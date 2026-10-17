---
format: aep.planning-md/3
id: story:m2-transitions
kind: story
status: implemented
title: Settled verdict changes are recorded as transitions and emitted as CloudEvents
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-hysteresis-core
- depends_on: story:m2-scheduler
- depends_on: story:m2-spec-history-records
- depends_on: story:m2-rollup
scope:
- confidence: inferred
  path: crates/thalamus-service/src/events.rs
- confidence: inferred
  path: crates/thalamus-service/src/scheduler.rs
- confidence: inferred
  path: crates/thalamus-service/src/transitions.rs
- confidence: inferred
  path: crates/thalamus-service/tests/transitions.rs
- confidence: inferred
  path: crates/thalamus-store/src/transitions.rs
- confidence: inferred
  path: crates/thalamus-store/tests/conformance.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T04:25:48Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-17T04:25:49Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-17T07:57:04Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Outcome

A settled verdict change (design § 3.2) is recorded as a transition and emitted as a CloudEvents event, so a consumer learns that a signal went out of bounds or came back without reading samples (design § 6.2, § 11 D6, D13).

## Work

- `crates/thalamus-store/src/transitions.rs`: the store port for `Transition` (specified by `story:m2-spec-history-records`): append through the generated `RecordTransition`, and read the transitions of a series (`SeriesTransitions`). A reader whose referenced evaluations have left raw retention gets the copied fields and the evaluations marked expired, not an error (D13).
- `crates/thalamus-store/tests/conformance.rs` (owned by `story:m1-eventlog-store`): one selection line adding the `RecordTransition` scenarios, so they run with the Eventlog storage.
- `crates/thalamus-service/src/transitions.rs`: one `SeriesHysteresis` (`story:m2-hysteresis-core`) per series, seeded on start by replaying the series' stored evaluations through the store's series range read; each new evaluation is observed, and a settled change is recorded as a `Transition` referencing its k evaluations.
- `crates/thalamus-service/src/events.rs`: each recorded transition emitted as a CloudEvents event, type `dev.beyond10x.thalamus.transition`, `subject` the series key of D10, `time` the settling evaluation's ts.
- `crates/thalamus-service/src/scheduler.rs` (owned by `story:m2-scheduler`): one hook line after grading calling `transitions::observe`. The third of three hook lines; this story lands after `story:m2-rollup` (`depends_on`).
- Observing a transition on real data is `story:m2-live-observation`; this story ends at its tests.

## Acceptance

- `cargo test -p thalamus-service --test transitions` passes these named tests, each red before the change: `transition_recorded_when_hysteresis_settles`, `transition_references_the_k_evaluations_of_its_run`, `transition_emitted_as_cloudevent_with_series_subject` (type `dev.beyond10x.thalamus.transition`, `subject` = the series key of D10, `time` = the settling evaluation's ts), `transition_survives_expired_evaluations` (a reader of a transition whose evaluations were retired gets the copied fields, not an error), `hysteresis_state_survives_restart` and `warn_opened_and_cleared_end_to_end` (a fake source drives one series through the scheduler until it has settled OK, then OK → WARN → OK; exactly two transitions are recorded and emitted by that drive. The series' first settled OK is itself a transition with no `from`, by hysteresis rule 4, design § 3.2).
- `cargo test -p thalamus-store --test conformance` passes `thalamus.transition.RecordTransition/outcome/recorded` with the Eventlog storage.

## Domain relations

- `Transition` → `SignalEvaluation`, references, many (design § 11 D13); declared by `story:m2-spec-history-records`.
- `Transition` → `SignalDefinition` via `signal`; a series is a signal id plus its dims (`SeriesKey`, `spec/domains/observation.yaml:20`).

## Scope

Planned (inferred):

- `crates/thalamus-store/src/transitions.rs`, `crates/thalamus-service/src/transitions.rs`, `crates/thalamus-service/src/events.rs` (their module lines are `story:m0-workspace`'s)
- `crates/thalamus-service/src/scheduler.rs` (one hook line; ordered after `story:m2-rollup`)
- `crates/thalamus-store/tests/conformance.rs` (one selection line; ordered after `story:m2-rollup`)
- `crates/thalamus-service/tests/transitions.rs`
