---
format: aep.planning-md/3
id: story:m4-heatmap
kind: story
status: draft
title: 'Heatmap of every series by layer: score per cell, unknown hatched, N/A grey, gaps empty'
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-skeleton
- depends_on: story:m1-eventlog-store
- depends_on: story:m1-verdict-grading
- depends_on: story:m1-instance-registry
scope:
- confidence: inferred
  path: crates/thalamus-service/src/views/heatmap.rs
- confidence: inferred
  path: crates/thalamus-service/src/views/svg.rs
- confidence: inferred
  path: crates/thalamus-service/tests/heatmap.rs
revision: 6
---
## Context

Design § 6.5, heatmap: "rows = series grouped by layer, columns = hours or days, cell = `score` on a diverging scale; unknown hatched, N/A grey", data: evaluations. § 3.3: `UNKNOWN` and `N/A` have no score and are "never drawn as 0"; the score "lets signals with different units share one heatmap". Epic acceptance: "a 4-week heatmap rendered from stored samples". This story fills `crates/thalamus-service/src/views/heatmap.rs` and `crates/thalamus-service/src/views/svg.rs` (stubs and route line from `story:m4-skeleton`). It reads the store through the newest-evaluation-per-series read and the series range read of `story:m1-eventlog-store`; it adds no read.

`svg.rs` holds the drawing primitives the other views reuse without editing it: the diverging score scale, the `UNKNOWN` hatch pattern, the `N/A` grey and the gap. `story:m4-sparklines`, `story:m4-week-vs-week` and `story:m4-before-after` depend on this story for it, and for the public cell function below.

Its tests use the in-process HTTP technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `GET /views/heatmap?days=<n>&step=day|hour&format=html|json`: default `days=28`, `step=day`; columns are UTC days or UTC hours ending with the current one; `format=json` returns the cell grid the SVG is drawn from.
- Rows: one per series that has an evaluation in the range, grouped by layer, the second segment of the signal id `<family>.<layer>.<name>`; inside a layer ordered by worst cell, then series key.
- Cell function (`pub`, reused by `story:m4-week-vs-week`): of the evaluations whose `ts` falls in the column, one per sample, the one at the highest `definition_rev` present; the cell verdict is their worst-of by ladder rank; a scored worst verdict (`OK`, `WARN`, `CRIT`) colours the cell with the highest score among them; worst `UNKNOWN` is hatched; every evaluation `N/A` is grey; no evaluation is a gap, drawn empty and distinct from all three.
- `svg.rs`: a fixed diverging scale with 0 at its neutral midpoint, negative scores toward the better side, 1 at the WARN colour, 2 at the CRIT colour, clamped beyond 3 and below -2; the hatch pattern; the grey; the gap. A cell carries its verdict, score and series key as a `<title>`.

## Decisions taken here (decomposer call, reversible)

- The cell rule above: worst-of by the ladder (design § 2.3) so the colour and the verdict cannot disagree (§ 3.3), and one evaluation per sample at the highest revision present, the rule design § 11 D14 sets for rollups.
- The heatmap reads raw evaluations only. The default retention of 35 days (design § 4) covers 4 weeks; drawing older columns from daily rollups is left for later.
- `format=json` on every view, so the views are testable and the live acceptance can count columns without parsing SVG.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id`; at most one evaluation per sample and revision | many-to-one | references | `spec/domains/evaluation.yaml:110`, `spec/domains/evaluation.yaml:224` (design § 11 D5) |
| `SignalEvaluation` → `SignalDefinition`, `definition`, via `signal`; the layer is read from the signal id | many-to-one | references | `spec/domains/evaluation.yaml:115`; id grammar `spec/domains/definition.yaml:10` (`UNMAPPED:`, held by a Rust test) |
| verdict rank and `scored` | one ladder | none | `thalamus.evaluation.Verdict`, `spec/domains/evaluation.yaml:17` |
| series key | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/views/heatmap.rs`, `crates/thalamus-service/src/views/svg.rs` (stubs from `story:m4-skeleton`)
- `crates/thalamus-service/tests/heatmap.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test heatmap` passes, against a store seeded with 28 days of hourly evaluations and no source reachable, the failing-first tests `four_week_heatmap_has_28_utc_day_columns_from_stored_evaluations`, `hour_step_has_24_columns_per_day`, `cell_takes_worst_verdict_and_its_highest_score`, `unknown_cell_is_hatched_never_coloured_as_zero`, `all_na_cell_is_grey`, `column_without_evaluation_is_a_gap`, `cell_counts_highest_rev_evaluation_per_sample`, `rows_grouped_by_layer_of_signal_id` and `score_zero_is_the_scale_midpoint`.

Open points found by story:m4-design-record (wave 20261015e); coordinator defaults, each recorded as a design § 11 row when this story lands:

- The colours of the scale between its stated fixed points: the implementor picks them, the golden file pins them, the § 11 row lists them.

From review-result:adversary-m4-design-record-pass-2, P4: `format=json` applies to every view except `/views/live`, whose machine-readable form is the evaluation stream (design § 6.5, D34, D35). This supersedes "format=json on every view" above.
