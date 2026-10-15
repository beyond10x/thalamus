---
format: aep.planning-md/3
id: story:m5-testrun-ingest
kind: story
status: draft
title: Test-run events become test.case.passed and test.run.pass_ratio samples
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-spec-integration-records
- depends_on: story:m5-skeleton
- depends_on: story:m1-cli-sample
- depends_on: story:m1-cli-eval-show
- depends_on: story:m1-instance-registry
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/testrun.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/testrun
- confidence: inferred
  path: crates/thalamus-cli/tests/testrun_ingest.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/ingest.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/sample.rs
- confidence: inferred
  path: crates/thalamus-service/tests/testrun_not_scheduled.rs
- confidence: inferred
  path: crates/thalamus-sources/src/testrun.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/testrun.rs
revision: 5
---
## Outcome

A test runner hands thalamus its run as `thalamus.testrun` events (`thalamus testrun ingest --events <path|->`, JSON Lines), and every finished case becomes a graded sample of `test.case.passed{project,suite,case,target}` and every finished run one of `test.run.pass_ratio{project,target}`, so a red suite on release day shows beside the platform signals.

## Context

Design § 6.3 (test run → thalamus): "a finished case becomes a sample of `test.case.passed{project,suite,case,target}` (1 or 0); a run becomes `test.run.pass_ratio`. An instrument failure maps to `quality: unknown`, a defect to a failing value." Design § 2.4: `registry: open` "is meant for test-run dimensions". The event format and the mapping decisions (skipped, errored, the run ratio, the `testrun` source) are `story:m5-spec-integration-records`'s. Test-run events are pushed, not read through Connectors, so a definition with `source.adapter` `testrun` is never sampled by the scheduler or by `thalamus sample --all`.

The push-fed skip is stated once, in the pipeline pass (coordinator decision C7, round 1). `story:m2-scheduler` runs, at each slot, the read → sample → grade → evaluate pass of `thalamus-pipeline` and "adds no pass of its own", and its missed-slot record (design § 11 D16) fills "one sample per series of that definition's newest pass". So a skip in `crates/thalamus-pipeline/src/sample.rs` reaches `thalamus sample --all` and the scheduler alike, and a push-fed definition has no pass, hence no series to fill. This story does not edit `crates/thalamus-service/src/scheduler.rs` and does not wait on `story:m5-service-answers`.

## Work

- `crates/thalamus-sources/src/testrun.rs` (pure): one event to at most one read. `passed` → value 1, `failed` → value 0, `errored` → `unknown` with reason `instrument failure` plus the message, never 0, `skipped` → nothing; `run.finished` → the ratio and its quality as decided in the spec story, with denominator `n` = passed + failed + errored and unit `{case}`. An envelope that is not CloudEvents 1.0 or has an unknown `type` is refused naming its line.
- `crates/thalamus-pipeline/src/ingest.rs`: for each read, the `Active` definition of the signal whose `source.adapter` is `testrun` and whose `source.connection` equals the event `source`; none is a refusal naming the event (a sample names an `Active` definition). Dims admitted through the instance registry; `ts` = `finished`, `window` = `started` to `finished`; provenance adapter `testrun`, connection the event `source`. Recorded through the generated `RecordSample`, then graded through `crates/thalamus-pipeline/src/evaluate.rs` (`story:m1-cli-eval-show`) unchanged. An event whose `source` and `id` were ingested before records nothing (CloudEvents: `source` plus `id` is unique).
- `crates/thalamus-cli/src/cmd/testrun.rs`: the `testrun` group and `ingest`; exit 0 when every line was recorded or was a repeat, 65 on a refused line (nothing after it is recorded), 74 for the store.
- `crates/thalamus-pipeline/src/sample.rs` (owned by `story:m1-cli-sample`): one selection line, where the pass picks the definitions it reads, skipping every definition whose `source.adapter` is `testrun` before any read and before any sample is recorded. This is the only place the rule is written.
- If `push_fed_definition_gets_no_sample_and_no_missed_slot` fails because the scheduler selects definitions outside the pipeline pass, the story stops and reports it against `story:m2-scheduler` rather than adding a second skip to `scheduler.rs`.
- A fixture instance with the two test-run definitions and an `open` registry for `project`, `suite`, `case`, `target`, neutral names only.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalSample` → `SignalDefinition`, `definition`; only an `Active` one is sampled | many-to-one | references | `spec/domains/observation.yaml:161`; `RecordSample` outcome `not-active` |
| a `case.finished` event → a `test.case.passed` sample; a `run.finished` event → a `test.run.pass_ratio` sample; a skipped case → none | one-to-one or zero | the sample references its definition | design § 6.3; decided in `story:m5-spec-integration-records` |
| test-run dims `project`, `suite`, `case`, `target` → registry kind `open` | many | registry admits any value | `spec/domains/dimension.yaml:48`; design § 2.4 |
| `SignalEvaluation` → `SignalSample`, `sample` | many-to-one | references | `spec/domains/evaluation.yaml:109` |

## Scope

Planned (inferred):

- `crates/thalamus-sources/src/testrun.rs`, `crates/thalamus-pipeline/src/ingest.rs`, `crates/thalamus-cli/src/cmd/testrun.rs` (module and subcommand lines are `story:m5-skeleton`'s)
- `crates/thalamus-pipeline/src/sample.rs` (one selection line; owned by `story:m1-cli-sample`; after it)
- `crates/thalamus-sources/tests/testrun.rs`, `crates/thalamus-cli/tests/testrun_ingest.rs`, `crates/thalamus-cli/tests/fixtures/testrun`, `crates/thalamus-service/tests/testrun_not_scheduled.rs` (new)
- Not `crates/thalamus-service/src/scheduler.rs`.

## Acceptance

- `cargo test -p thalamus-sources --test testrun` passes the failing-first test `errored_case_reads_unknown_without_value` and `passed_case_reads_1`, `failed_case_reads_0`, `skipped_case_reads_nothing`, `run_pass_ratio_from_its_counts`, `run_with_errored_cases_is_partial_upper_bound`, `run_without_gradable_case_is_unknown` and `wire_event_matches_golden_envelope`.
- `cargo test -p thalamus-cli --test testrun_ingest` passes `cases_appear_as_samples_and_evaluations`, `event_from_undeclared_source_is_refused_with_65`, `reingesting_an_event_records_nothing`, `malformed_line_is_refused_naming_it` and `sample_all_skips_push_fed_definitions`.
- `cargo test -p thalamus-service --test testrun_not_scheduled` passes `push_fed_definition_gets_no_sample_and_no_missed_slot` with `crates/thalamus-service/src/scheduler.rs` unchanged by this story (`git diff --stat <base> -- crates/thalamus-service/src/scheduler.rs` prints nothing).
