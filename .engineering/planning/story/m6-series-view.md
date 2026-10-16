---
format: aep.planning-md/3
id: story:m6-series-view
kind: story
status: draft
title: The series view draws samples, evaluations and settled transitions of one series and appends live
relations:
- decomposes: epic:web-ui
- depends_on: story:m6-web-types
- depends_on: story:m6-web-skeleton
- depends_on: story:m4-service-api
- depends_on: story:m4-live
- depends_on: story:m2-transitions
scope:
- confidence: inferred
  path: crates/thalamus-service/src/api_transitions.rs
- confidence: inferred
  path: crates/thalamus-service/tests/api_transitions.rs
- confidence: inferred
  path: web/fixtures/transitions.json
- confidence: inferred
  path: web/src/lib/series.test.ts
- confidence: inferred
  path: web/src/lib/series.ts
- confidence: inferred
  path: web/src/views/Series.vue
revision: 4
---
## Context

`epic:web-ui` Ships: "A series view: samples, evaluations and settled transitions of one series over a time range, with the stream appending new points live." Samples and evaluations come from `GET /api/series` (`story:m4-service-api`). Settled transitions are recorded by `story:m2-transitions` and emitted as CloudEvents, but no route serves them, so this story adds one: `GET /api/transitions` (`story:m6-design-record` item 4), filling the stub `crates/thalamus-service/src/api_transitions.rs` whose route line and module line `story:m6-web-skeleton` lays. It reads the transitions of a series through the store read of `story:m2-transitions` (`crates/thalamus-store/src/transitions.rs`, `SeriesTransitions`) and adds no store read. Its HTTP tests use the in-process technique of `story:m4-skeleton`; it adds no dependency.

The browser half fills the stub `web/src/views/Series.vue`, through the client of `story:m6-web-types`.

## Work

- `crates/thalamus-service/src/api_transitions.rs`: `GET /api/transitions?key=<series key>&from=<ts>&to=<ts>` answers the `thalamus.transition.SeriesTransitions` rows (`spec/domains/transition.yaml:191`) with `settled_ts` in `[from, to)`, in settling order, default the last 7 days, `application/json`, numbers as the decimal strings of D11. A series with samples and no transition answers `[]`; a key with no sample in the store answers 404, as `/api/series` does. A transition whose evaluations have left raw retention is listed with its copied fields (D13).
- `crates/thalamus-service/tests/api_transitions.rs` (new), with the golden answer `web/fixtures/transitions.json` (`THALAMUS_BLESS=1` rewrites it).
- `web/src/lib/series.ts` (pure):
  - range: the last 7 days by default, 1 day and 28 days offered, and a `from`/`to` pair; the 35-day raw retention (design § 4) bounds what can be shown.
  - value line through `ok` readings; it breaks at every `unknown` sample (a missed slot is one, D16) and wherever one sample's `window.start` is after the previous sample's `window.end`; nothing is interpolated. A `partial` reading is a bound marker on its `bound` side, not a point of the line (D36).
  - evaluations: per sample the one at the highest `definition_rev` present (D35), drawn as a verdict mark and, when scored, a score point; `UNKNOWN` and `N/A` draw no score, never 0.
  - transitions: a mark at each `settled_ts` labelled with its `from_verdict` and `to_verdict`, spanning back to `first_ts`.
  - live: when the range ends now, a stream evaluation of this series rereads `series()` and `transitions()` from the newest `ts` shown and appends what they answer; an evaluation of another series is ignored.
- `web/src/lib/series.test.ts`; `web/src/views/Series.vue` (stub from `story:m6-web-skeleton`): SVG drawn in the template, the series key and its dims as the heading.
- `web/dist/` rebuilt (the coordinator rebuilds it at merge).

## Decisions taken here (decomposer calls, reversible)

- The transitions route mirrors `/api/series`: same parameters, half-open range, 7-day default and 404 for an unknown key.
- A live evaluation triggers a reread rather than being drawn from the stream alone, because the evaluation record carries no reading (it references its sample, `spec/domains/evaluation.yaml:110`).
- No baseline band and no WARN and CRIT lines: they are computed from the definition by the grading code (D36), which no API serves; the M4 sparklines view draws them.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id`; at most one evaluation per sample and revision | many-to-one | references | `spec/domains/evaluation.yaml:110`, `spec/domains/evaluation.yaml:228` (design § 11 D5) |
| `Transition` → `SignalEvaluation`, `evaluations` | many | references; the transition outlives them | `spec/domains/transition.yaml:56`, design § 11 D13 |
| `Transition` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/transition.yaml:61` |
| series key, the route parameter | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/api_transitions.rs` (stub from `story:m6-web-skeleton`)
- `crates/thalamus-service/tests/api_transitions.rs` (new)
- `web/fixtures/transitions.json` (new)
- `web/src/views/Series.vue` (stub from `story:m6-web-skeleton`)
- `web/src/lib/series.ts`, `web/src/lib/series.test.ts` (new)
- `web/dist/` is rebuilt by the coordinator at merge (`task web-build`) and, like `Cargo.lock`, is not in this story's typed scope, so the M6 view stories can share a wave

## Acceptance

`cargo test -p thalamus-service --test api_transitions` passes, against a seeded store with no source reachable, the failing-first tests `transitions_of_a_series_in_settling_order`, `transitions_range_is_half_open_on_settled_ts`, `transitions_default_to_the_last_7_days`, `known_series_without_transitions_answers_empty`, `unknown_series_answers_404`, `transition_with_expired_evaluations_keeps_copied_fields` and `transitions_answer_matches_golden`, and `npm test` in `web/` passes `transitions_fixture_validates_against_generated_schema`, `series_line_breaks_at_unknown_sample`, `series_line_breaks_where_windows_do_not_meet`, `partial_reading_is_a_bound_marker_not_a_point`, `unknown_and_na_evaluations_draw_no_score`, `highest_rev_evaluation_per_sample_is_drawn`, `transition_marked_at_settled_ts_with_from_and_to`, `stream_evaluation_of_this_series_rereads_and_appends` and `stream_evaluation_of_another_series_is_ignored`.
