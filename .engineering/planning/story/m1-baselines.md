---
format: aep.planning-md/3
id: story:m1-baselines
kind: story
status: implemented
title: Offset, pooled, higher/lower-of and fixed baselines; one older than its series is UNKNOWN
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m0-workspace
scope:
- confidence: inferred
  path: crates/thalamus-core/src/baseline/mod.rs
- confidence: inferred
  path: crates/thalamus-core/tests/baselines.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T03:07:33Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T03:07:33Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-15T04:28:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Outcome

`thalamus-core` turns the readings of earlier windows into the baseline a sample is graded against, and a baseline that could not be read, or that is older than its series, comes out `UNKNOWN`, never 0.

## Context

`epic:signal-contract`; design § 3.1 (table and rules 1 to 3), § 1 L6 and L8. Pure: the caller passes the earlier readings and the series age in; this story plans which windows to read and folds what comes back. The generated `thalamus.definition.BaselineKind` and `thalamus.evaluation.Baseline` are its interface. Its module file exists after `story:m0-workspace` (`depends_on`). This story, `story:m1-verdict-grading` and `story:m1-instance-registry` write disjoint files, so no shared line orders them: the `crates/thalamus-core/src/lib.rs` module lines are `story:m0-workspace`'s, and none of the three touches them.

## Work

- `crates/thalamus-core/src/baseline/mod.rs`:
  - `plan(definition, sample) -> Vec<WindowRequest>`: for `offset_1d` and `offset_7d` the sample window shifted back 1 day and 7 days (a 7-day shift keeps the weekday); nothing for `fixed`.
  - `fold(definition, sample, readings, series_age) -> thalamus.evaluation.Baseline`:
    - `offset_1d`, `offset_7d`: the reading of that window;
    - `pooled`: sum of the offsets numerators (value times `n`) over the sum of their denominators;
    - `higher_of`, `lower_of`: the higher or lower of the offset values;
    - `fixed`: the number in the definition;
    - the recorded kind and offsets follow design § 11 D4 (`pooled` over `[1d, 7d]`).
  - Rule 1: any constituent reading `unknown`, or missing, makes the baseline reading `unknown` with a reason naming the offset.
  - Rule 2: an offset longer than the series age makes the baseline `unknown` with reason `baseline predates series`; a series whose age is not known makes an offset baseline `unknown` with reason `series age unknown`.
  - Rule 3: a definition of kind `instant` is folded against `fixed` only; any other kind requested for it is refused.
- `same_slot_median_4w` and `rolling_median` answer a named "not in M1" refusal from the dispatch in `baseline/mod.rs`; `story:m2-store-baselines` replaces those two arms with calls into `baseline/same_slot.rs` and `baseline/rolling.rs`.

## Domain relations

- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`. Inferable: `spec/domains/observation.yaml`, entity `SignalSample`, relation `definition`. The baselines a sample is graded against come from that definition, `evaluation.baselines`.
- A sample series origin is `denominator.since` and `age_s` (`spec/domains/observation.yaml`, type `thalamus.observation.Denominator`). Its forms are design § 11 D18 (`first_sample`, a fixed timestamp, `dimension.created` for a `file` registry entry carrying `created`), resolved by `story:m1-instance-registry` and the pipeline; this story only takes an age, never resolves one.
- `instant` admits only `fixed`. Inferable: `spec/domains/definition.yaml`, `ProposeDefinition` and `ReviseDefinition` outcome `instant-not-fixed`.

## Decisions taken (reversible)

- `pooled`, `higher_of` and `lower_of` are `unknown` when any constituent is, rather than folding the readable ones.
- An unknown series age makes every offset baseline `unknown` (L6 read conservatively).

## Scope

Planned (inferred):

- `crates/thalamus-core/src/baseline/mod.rs` (its module line is `story:m0-workspace`'s)
- `crates/thalamus-core/tests/baselines.rs`

## Acceptance

`cargo test -p thalamus-core --test baselines` passes the tests `baseline_older_than_series_is_unknown` (written and seen failing first), `unreadable_offset_is_unknown_never_zero`, `unknown_series_age_makes_offsets_unknown`, `offset_7d_window_keeps_weekday`, `pooled_sums_numerators_over_denominators`, `higher_of_and_lower_of_pick_the_bound`, `fixed_is_the_definition_number` and `instant_refuses_non_fixed_baseline`.

## Out of scope

`same_slot_median_4w`, `rolling_median` (`story:m2-store-baselines`); reading any window (the adapter and the pipeline); resolving `denominator.since` (`story:m1-instance-registry`).
