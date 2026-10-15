---
format: aep.planning-md/3
id: story:m0-conformance-records
kind: story
status: implemented
title: Every synthesised conformance scenario passes under task check
relations:
- decomposes: epic:specification
- depends_on: story:m0-conformance-definitions
scope:
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/assertion.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/evaluation.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/observation.rs
- confidence: inferred
  path: crates/thalamus-core/src/conformance.rs
- confidence: inferred
  path: crates/thalamus-core/tests/conformance.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T05:39:24Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-15T05:39:24Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-15T08:23:19Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Context

Completes the conformance suite the epic asks `task check` to run. `story:m0-conformance-definitions` built the harness and met the definition and dimension obligations; this story meets the remaining five obligations of the `ess generate synthesize` plan and lets the harness run every scenario.

## Work

- `crates/thalamus-core/src/behaviour/observation.rs` (inferred): obligation `thalamus.observation.RecordSample`, outcomes `unknown-signal` (error `thalamus.observation.UnknownSignal`), `not-active` (error `thalamus.observation.SignalNotActive`), `recorded`; and the view query `thalamus.observation.SeriesSamples` (rows of `SignalSample` where `series == param.series`).
- `crates/thalamus-core/src/behaviour/evaluation.rs` (inferred): obligation `thalamus.evaluation.RecordEvaluation`, outcomes `unknown-sample`, `reason-missing`, `score-not-admitted`, `recorded`, in `PLAN.md` selection precedence; and the view query `thalamus.evaluation.SeriesVerdicts`.
- `crates/thalamus-core/src/behaviour/assertion.rs` (inferred): obligation `thalamus.assertion.AskAssertion`, outcomes `no-signals`, `asked`.
- `crates/thalamus-core/tests/conformance.rs`: the id selection goes; every scenario runs, and the expected count is read from `spec/suite.json`, never written into the test. A new test `every_obligation_is_met` calls each of the eight `PLAN.md` obligations once and fails if any answers `UnmetObligation`.

Not here, because the specification does not state them and design § 11 assigns them to code in later milestones: every signal named by an assertion exists and is `Active` (D1, M5), an assertion is answered only after `after + horizon` (D3, M5), at most one evaluation per sample and definition revision (D5, store, M1), every `dims` key and value of a sample resolves in the registry (`spec/domains/observation.yaml:142`, M1).

## Domain relations

| relation | from → to | cardinality | ownership | used by | citation |
|---|---|---|---|---|---|
| `definition` | `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition` | one, via `signal` | references | `RecordSample` outcomes `unknown-signal`, `not-active` | `spec/domains/observation.yaml:160` |
| `sample` | `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample` | one, via `sample_id` | references | `RecordEvaluation` outcome `unknown-sample` | `spec/domains/evaluation.yaml:109` |
| `definition` | `thalamus.evaluation.SignalEvaluation` → `thalamus.definition.SignalDefinition` | one, via `signal` | references | stored on `recorded`; not checked by any outcome | `spec/domains/evaluation.yaml:115` |
| `signals` | `thalamus.assertion.Assertion` → `thalamus.definition.SignalDefinition` | many, via `signals` | references | stored on `asked`; `no-signals` reads only the count | `spec/domains/assertion.yaml:98` |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/behaviour/observation.rs`, `crates/thalamus-core/src/behaviour/evaluation.rs`, `crates/thalamus-core/src/behaviour/assertion.rs`
- `crates/thalamus-core/tests/conformance.rs` (owned by `story:m0-conformance-definitions`, which this story follows: the selection constant goes and `every_obligation_is_met` is added)

The module lines in `crates/thalamus-core/src/behaviour/mod.rs` are `story:m0-workspace`'s; this story does not touch them.

## Acceptance

`task check` exits 0 with `cargo test -p thalamus-core --test conformance` answering every scenario in `spec/suite.json` (73 at ess 0.59.0, none skipped) and its test `every_obligation_is_met` passing.

## Scope addition (2026-10-15, from story:m0-conformance-definitions)

The shared runner `crates/thalamus-core/src/conformance.rs` is this story's too, after `story:m0-conformance-definitions`: replace the five `Unimplemented` port impls (AskAssertion, RecordEvaluation, RecordSample, SeriesVerdicts, SeriesSamples) and support the steps the other 22 scenarios use: `expect_direct_response`, `expect_response_payload`, `query_view` with `params`, `expect_view` `counts` and `ranked`, and the `{field: {any_of: [...]}}` predicate.
