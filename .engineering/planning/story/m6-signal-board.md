---
format: aep.planning-md/3
id: story:m6-signal-board
kind: story
status: draft
title: The signal board shows every series with verdict, score and age, grouped and filtered from the data, live
relations:
- decomposes: epic:web-ui
- depends_on: story:m6-web-types
- depends_on: story:m4-service-api
- depends_on: story:m4-live
- depends_on: story:m5-skeleton
scope:
- confidence: inferred
  path: web/src/lib/board.test.ts
- confidence: inferred
  path: web/src/lib/board.ts
- confidence: inferred
  path: web/src/views/Board.vue
revision: 5
---
## Context

`epic:web-ui` Ships: "A generic signal board: every series of the instance with its current verdict, score and age, grouped and filtered by family, signal and dimension values read from the data, updated live." It reads `GET /api/verdicts` (`story:m4-service-api`: the newest evaluation of each series, worst first, an `UNKNOWN` or `N/A` row with no `score` key), the evaluation stream (`story:m4-live`, D34) and, when present, `GET /api/assertions` (route laid by `story:m5-skeleton`, answering 501 until `story:m5-service-answers` fills it), all through the client of `story:m6-web-types`. It fills the stub `web/src/views/Board.vue` of `story:m6-web-skeleton`. Nothing in it names a signal, a family or a dimension: every group, filter and value comes from the rows.

## Work

- `web/src/lib/board.ts` (pure, no DOM):
  - rows: one per series from `verdicts()`: signal, series key, dims, verdict, score, `ts`, reason; the age is now minus `ts`, recomputed every second from an injected clock.
  - grouping: by family, the first segment of the signal id, then by signal; inside a group worst first by ladder rank, then series key.
  - filters: family, signal, and per dimension key the values present in the current rows; the values offered are read from the rows, never from a list in code.
  - live: a stream evaluation replaces its series row when its `ts` is newer than the row's, adds a row for a series not on the board, and is ignored when older. On `lagged`, and on `reconnecting` turning `live`, the board rereads `verdicts()` and replaces every row (D34).
  - `UNKNOWN` and `N/A` rows show no score, never 0; `UNKNOWN` is marked hatched and `N/A` grey, the marks of the M4 views (D35, D40).
  - assertions: when `assertions()` is present, each signal group lists the assertions whose `signals` name the signal, with state, result, worst, asker, `after` and `horizon`; when it is `absent` there is no assertion column.
- `web/src/lib/board.test.ts`.
- `web/src/views/Board.vue`: draws the above; each series key links to `#/series/<key>`; the stream state of `sse.ts` is shown in the header.
- `web/dist/` rebuilt (the coordinator rebuilds it at merge).

## Decisions taken here (decomposer calls, reversible)

- Assertions are listed per signal, not matched to series. `Assertion.dims` "narrows" the series (`spec/domains/assertion.yaml:72`, field summary), but no ESS relation or design row states the matching rule, so the board shows an assertion's dims as text and does not decide which series it covers.
- Age is computed in the browser from `ts`; no server time is read.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/evaluation.yaml:115` |
| the current verdict of a series is its newest evaluation | one per series | none | `spec/domains/evaluation.yaml:292` (`UNMAPPED:`; the reader picks the newest `ts`); `/api/verdicts` applies it |
| the family of a signal is the first segment of its id | one family per signal | none | inferred from `crates/thalamus-core/src/registry.rs:1186` (the loader refuses an id whose first segment is not the definition's `family`); the id grammar itself is `UNMAPPED:` at `spec/domains/definition.yaml:10` |
| `Assertion` → `SignalDefinition`, `signals` | many | references | `spec/domains/assertion.yaml:99` |
| series key | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `web/src/views/Board.vue` (stub from `story:m6-web-skeleton`)
- `web/src/lib/board.ts`, `web/src/lib/board.test.ts` (new)
- `web/dist/` is rebuilt by the coordinator at merge (`task web-build`) and, like `Cargo.lock`, is not in this story's typed scope, so the M6 view stories can share a wave

## Acceptance

`npm test` in `web/` passes the failing-first tests `board_groups_by_family_read_from_signal_id`, `board_orders_worst_first_within_a_group`, `board_filter_values_are_read_from_rows`, `stream_evaluation_replaces_older_row_of_its_series`, `stream_evaluation_of_new_series_adds_a_row`, `older_stream_evaluation_is_ignored`, `lag_or_reconnect_rereads_verdicts`, `unknown_and_na_rows_show_no_score_never_zero`, `age_counts_from_ts_on_an_injected_clock`, `assertions_listed_under_each_signal_they_name` and `absent_assertions_show_no_column`, and `npx vue-tsc --noEmit` in `web/` exits 0 in the implementor's tree.

`task web-check` (the diff of a fresh build against the committed `web/dist/`) is not part of this story's acceptance: `web/dist/` is rebuilt by the coordinator at merge (revised after review-result:m6-parallel-safety-round-1), and the merge gate runs `task check`, which includes `web-check`.
