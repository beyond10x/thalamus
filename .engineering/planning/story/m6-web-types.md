---
format: aep.planning-md/3
id: story:m6-web-types
kind: story
status: draft
title: TypeScript record types are generated from spec/ and drift-checked; one typed client reads the API and the streams
relations:
- decomposes: epic:web-ui
- depends_on: story:m6-web-skeleton
- depends_on: story:m4-service-api
- depends_on: story:m4-live
- depends_on: story:m5-spec-integration-records
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/thalamus-service/tests/web_fixtures.rs
- confidence: inferred
  path: web/fixtures
- confidence: inferred
  path: web/src/generated
- confidence: inferred
  path: web/src/lib/api.test.ts
- confidence: inferred
  path: web/src/lib/api.ts
- confidence: inferred
  path: web/src/lib/fixtures.test.ts
- confidence: inferred
  path: web/src/lib/sse.test.ts
- confidence: inferred
  path: web/src/lib/sse.ts
- confidence: inferred
  path: web/src/lib/wire.ts
revision: 4
---
## Context

`epic:web-ui` Ships: "TypeScript types for every record the API and the stream carry, generated from the thalamus ESS specification and drift-checked by `task check`; no hand-written record type." The decisions are `story:m6-design-record` items 3 and 5. Precedent: uilab's `Taskfile.yml` runs `ess generate types --target typescript --out widget/src/generated` in `generate` and diffs a fresh run in `drift`.

This story also writes the one typed client every view reads, so no view hand-types an answer: `GET /api/verdicts` and `GET /api/series` (`story:m4-service-api`), `GET /api/evaluations/stream` (`story:m4-live`, D34), and `GET /api/assertions` (laid by `story:m5-skeleton`, filled by `story:m5-service-answers`), plus the two routes M6 adds (`story:m6-series-view`, `story:m6-flow-view`), typed here from the spec and checked against their golden answers in those stories.

Risk found while decomposing, not a guess at the answer: `task generate` runs `ess generate types --all-types --all-events --target rust`, and the `generated/spec-types/types.rs` it writes declares no entity record and no view row (no `SignalEvaluation`, no `Evaluations`); the rows the API serialises are the `thalamus-model` types `ess generate synthesize` writes. Whether `ess generate types` accepts an entity or a view as `--root` was not observed: the installed `ess` was 0.59.0 and `spec/` requires 0.60.0. The first step of this story is that probe. If ESS refuses, the story stops: the gap is filed in the ESS repository and the story waits for the release that closes it (design item 3); no record type is written by hand meanwhile.

## Work

- `Taskfile.yml` `generate` and `drift`: one `ess generate types --path spec --target typescript --strict-requires` with `--root` for `thalamus.observation.SignalSample`, `thalamus.evaluation.SignalEvaluation`, `thalamus.transition.Transition`, `thalamus.assertion.Assertion`, `thalamus.evaluation.Evaluations`, `thalamus.observation.SeriesSamples`, `thalamus.transition.SeriesTransitions` and `thalamus.assertion.Assertions`, out `web/src/generated/`; `drift` diffs a fresh run against it, as it diffs `generated/`.
- `web/src/generated/` (generated, never edited by hand).
- `web/src/lib/wire.ts`: `CloudEvent<T>` (the D33 envelope fields) and `SeriesAnswer` (`samples` and `evaluations` over the generated row types). Nothing else is typed by hand.
- `web/src/lib/api.ts`: `verdicts(signal?)`, `series(key, from?, to?)`, `transitions(key, from?, to?)` and `assertions()`, each returning generated types over same-origin relative URLs; `assertions()` answers `absent` on 501 or 404, not an error.
- `web/src/lib/sse.ts`: a `fetch`-based server-sent-events reader that parses `event`, `id`, `data` and comment lines. States: `connecting`, `live`, `lagged` (a comment, then the server closed the stream: D34), `reconnecting` (any other close or network error, retried after 1 s, doubling to at most 30 s). It never sends `Last-Event-ID` (D34: no replay).
- `crates/thalamus-service/tests/web_fixtures.rs` (new): over a seeded store, through `http::router` and `tower::ServiceExt::oneshot`, compares the golden answers `web/fixtures/verdicts.json`, `web/fixtures/series.json`, `web/fixtures/evaluation-stream.txt` (two evaluations, one `UNKNOWN`) and `web/fixtures/evaluation-stream-lagged.txt` (a lagging subscriber) with what the handlers answer; `THALAMUS_BLESS=1` rewrites them.
- `web/fixtures/` (new), `web/src/lib/api.test.ts`, `web/src/lib/sse.test.ts`, `web/src/lib/fixtures.test.ts` (each golden answer validated against `web/src/generated/source.schema.json`; each stream fixture read through `sse.ts`).
- `web/dist/` rebuilt (the coordinator rebuilds it at merge).

## Decisions taken here (decomposer calls, reversible)

- No golden answer for `/api/assertions` here: `story:m5-service-answers` need not have landed, and this story does not wait on the M5 assertion chain. Its rows are typed from the spec; the board tests use typed literals.
- A 501 from `/api/assertions` means the M5 answers are not built yet, so the client reports `absent`, which the board shows as no assertions, never as an error.

## Domain relations

None joined here: the client returns rows as the API answers them. The relations each view joins are cited in that view's story.

## Scope

Planned (inferred):

- `Taskfile.yml` (`generate`, `drift`; after `story:m6-web-skeleton`)
- `web/src/generated` (new, generated)
- `web/src/lib/wire.ts`, `web/src/lib/api.ts`, `web/src/lib/sse.ts`, `web/src/lib/api.test.ts`, `web/src/lib/sse.test.ts`, `web/src/lib/fixtures.test.ts` (new)
- `web/fixtures` (new)
- `crates/thalamus-service/tests/web_fixtures.rs` (new)
- `web/dist/` is rebuilt by the coordinator at merge (`task web-build`) and, like `Cargo.lock`, is not in this story's typed scope, so the M6 view stories can share a wave

## Acceptance

`task drift` passes and fails after any byte under `web/src/generated/` is changed, `cargo test -p thalamus-service --test web_fixtures` passes the failing-first tests `verdicts_answer_matches_golden`, `series_answer_matches_golden`, `evaluation_stream_matches_golden` and `lagged_evaluation_stream_matches_golden`, and `npm test` in `web/` passes the failing-first tests `golden_answers_validate_against_generated_schema`, `unknown_evaluation_fixture_has_no_score_key`, `sse_reader_parses_event_id_and_data`, `lag_comment_then_close_is_lagged_not_reconnecting`, `network_close_is_reconnecting_with_backoff`, `sse_reader_never_sends_last_event_id` and `assertions_501_is_absent_not_an_error`.
