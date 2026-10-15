---
format: aep.planning-md/3
id: story:m4-week-vs-week
kind: story
status: draft
title: Week vs week compares each hour slot with the same slot seven days earlier
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-heatmap
scope:
- confidence: inferred
  path: crates/thalamus-service/src/views/week.rs
- confidence: inferred
  path: crates/thalamus-service/tests/week.rs
revision: 4
---
## Context

Design § 6.5, week vs week: "this week against the same slot last week", data: evaluations and `offset_7d`. § 3.1 rule 2 and § 1 L6: a baseline older than its series is `UNKNOWN` with reason `baseline predates series`. This story fills `crates/thalamus-service/src/views/week.rs` (stub and route line from `story:m4-skeleton`) and reuses the cell function and `svg.rs` of `story:m4-heatmap` without editing them. It reads the store through the series range read of `story:m1-eventlog-store`.

Its tests use the in-process HTTP technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `GET /views/week?at=<ts>&format=html|json`: for every series with an evaluation in either week, 168 UTC hour slots of this week (the 7 days ending at the start of the hour of `at`, default now) against the same slots 7 days earlier; the weekday and hour match by construction.
- Per slot: this week cell and last week cell from the heatmap cell function, and their score delta only when both cells are scored; otherwise no delta, never 0.
- A slot of last week before the first stored sample of the series is `UNKNOWN` with reason `baseline predates series`; a slot of last week with no evaluation is a gap.
- Rows ordered by the largest score delta of the week, then series key.

## Decisions taken here (decomposer call, reversible)

- "This week" is the rolling 7 days ending at `at`, not the calendar week, so the view is full on any day; `at` makes it reproducible.
- The comparison reads stored evaluations at the slot 7 days earlier (the M2 `compare` stance, `story:m2-compare`), not the `offset_7d` baseline recorded on each evaluation, which a `history: source` signal re-reads from its source and records only as a value.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/evaluation.yaml:115` |
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id`; one per sample and revision | many-to-one | references | `spec/domains/evaluation.yaml:110`, `spec/domains/evaluation.yaml:224` (design § 11 D5) |
| `SignalSample` → `SignalDefinition`: the first stored sample of a series dates the series | many-to-one | references | `spec/domains/observation.yaml:161`; `thalamus.observation.SeriesSamples`, `spec/domains/observation.yaml:304` |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/views/week.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-service/tests/week.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test week` passes, against a seeded store and no source reachable, the failing-first tests `slot_compared_with_same_slot_seven_days_earlier`, `week_has_168_hour_slots_ending_at_at`, `missing_last_week_slot_is_a_gap_not_zero`, `delta_only_between_two_scored_cells`, `series_younger_than_a_week_is_baseline_predates_series` and `rows_ordered_by_largest_score_delta`.
