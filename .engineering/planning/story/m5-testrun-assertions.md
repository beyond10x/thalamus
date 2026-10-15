---
format: aep.planning-md/3
id: story:m5-testrun-assertions
kind: story
status: draft
title: A test run carries signal assertions, each answer emitted as a case of the run
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-declared-sets
- depends_on: story:m5-testrun-ingest
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/assert.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/testrun_assertions
- confidence: inferred
  path: crates/thalamus-cli/tests/testrun_assertions.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/testrun_cases.rs
revision: 5
---
## Outcome

A release-day test run carries signal assertions: `thalamus assert --declared <file> --asker test_run:<run-id> --wait --emit-cases <path|-> --project <p> --target <t>` writes each answer as one `case.finished` event into the run's stream, so the test report shows "platform signals stayed in baseline" beside its scenarios, and those cases come back into thalamus as samples.

## Context

Design § 6.3 (thalamus → test run): "a release-day run carries signal assertions; each result is emitted as a case in the run's stream, so the test UI shows ... beside the scenarios". The event format is `story:m5-spec-integration-records`'s, the set is `story:m5-declared-sets`'s, and the way back (events → samples) is `story:m5-testrun-ingest`'s, which this story closes the loop with.

## Work

- `crates/thalamus-pipeline/src/testrun_cases.rs`: one answered assertion → one `dev.beyond10x.testrun.case.finished` event: `subject` and `data.run_id` the asker ref, `suite` `thalamus.assertions`, `case` the declared entry `name`, `started` = `after`, `finished` = the answer time, `outcome` `within` → `passed`, `breach` → `failed` with the reason and the assertion id as `message`, `unknown` → `errored` with the reason. CloudEvents `source` from `--emit-source` (default `thalamus`), `id` derived from the assertion id so a repeat emits the same event. An assertion still open emits nothing.
- `crates/thalamus-cli/src/cmd/assert.rs` (owned by `story:m5-assert`, extended by `story:m5-declared-sets`): the flags `--emit-cases`, `--emit-source`, `--project`, `--target`, accepted only with `--asker test_run:...`. Under any other asker kind each is a usage error: exit 64 naming the flag, checked before any ask, so nothing is recorded.

## Decisions taken here (decomposer calls, reversible)

- `unknown` becomes `errored`, not `failed`: it is an instrument failure, and ingest maps `errored` back to an `unknown` sample, so the round trip never turns an unknown into a 0 or a pass.
- The assertion cases go into the run's own stream (a file or stdout the runner appends), not to a test system API: no test runner in this workspace has one to write to.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| one answered `Assertion` → one `case.finished` event | one-to-one; an open one → none | none: the event copies the answer | design § 6.3; event type declared by `story:m5-spec-integration-records` |
| `Asker.kind` `test_run`, `ref` the run id | one per assertion | value | `spec/domains/assertion.yaml:17` |
| an emitted `case.finished` event → one `test.case.passed` sample when ingested | one-to-one | the sample references its definition | `spec/domains/observation.yaml:161`; `story:m5-testrun-ingest` |

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/src/testrun_cases.rs` (module line is `story:m5-skeleton`'s)
- `crates/thalamus-cli/src/cmd/assert.rs` (after `story:m5-declared-sets`)
- `crates/thalamus-cli/tests/testrun_assertions.rs`, `crates/thalamus-cli/tests/fixtures/testrun_assertions` (new)

## Acceptance

`cargo test -p thalamus-cli --test testrun_assertions` passes, against the built binary and a seeded store, the failing-first tests:

- `unknown_answer_is_an_errored_case_never_passed`, `each_answer_is_one_case_event`, `breach_is_a_failed_case_naming_the_assertion`, `open_assertion_emits_no_case` and `repeat_emits_the_same_event_id`;
- `emitted_cases_ingest_back_as_samples` (the emitted stream piped into `thalamus testrun ingest` records one `test.case.passed` sample per answer: 1 for `within`, 0 for `breach`, `unknown` without a value for `unknown`);
- `testrun_flags_refused_without_test_run_asker` (each of `--emit-cases`, `--emit-source`, `--project` and `--target` with `--asker release:v1` exits 64 naming the flag, and records no assertion).
