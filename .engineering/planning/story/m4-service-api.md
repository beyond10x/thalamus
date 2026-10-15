---
format: aep.planning-md/3
id: story:m4-service-api
kind: story
status: draft
title: thalamus-service serves current verdicts and series over HTTP
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-skeleton
- depends_on: story:m1-eventlog-store
- depends_on: story:m1-cli-eval-show
scope:
- confidence: inferred
  path: crates/thalamus-service/src/api.rs
- confidence: inferred
  path: crates/thalamus-service/tests/api.rs
revision: 7
---
## Context

Design § 5.1: `thalamus-service` serves "the HTTP API (current verdicts, series, assertions)". M4 ships the current verdicts and the series. The assertions part is M5: `story:m5-skeleton` adds the routes `GET /api/assertions` and `GET /api/assertions/{id}` to the route table of `story:m4-skeleton`, and `story:m5-service-answers` fills them. The route lines and the shared state come from `story:m4-skeleton`; this story fills the stub `crates/thalamus-service/src/api.rs`. It adds no store read: the newest evaluation of each series and the series range read come from `story:m1-eventlog-store` (`crates/thalamus-store/src/evaluations.rs`, `crates/thalamus-store/src/range.rs`), and the worst-first order is the ladder of `story:m1-verdict-grading`.

Its tests use the in-process HTTP technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `GET /api/verdicts[?signal=<id>]`: the newest evaluation of every series, worst first by ladder rank, then by series key; each row is the generated `thalamus.evaluation.Evaluations` view row (`spec/domains/evaluation.yaml:258`). An `UNKNOWN` or `N/A` row carries no `score` key at all, never 0.
- `GET /api/series?key=<series key>&from=<ts>&to=<ts>`: `{"samples": [...], "evaluations": [...]}`, the `thalamus.observation.SeriesSamples` rows (`spec/domains/observation.yaml:304`) and the `Evaluations` rows of that series in `ts` order over `[from, to)`; default range the last 7 days. A sampling gap stays a gap: no row is synthesised. An unknown series key answers 404.
- Both answer `application/json`; numbers are the decimal strings of design § 11 D11.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id` | many-to-one | references | `spec/domains/evaluation.yaml:110` |
| `SignalEvaluation` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/evaluation.yaml:115` |
| `SignalSample` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/observation.yaml:161` |
| the current verdict of a series is its newest evaluation | one per series | none | `spec/domains/evaluation.yaml:292` (`UNMAPPED:`, latest row per group; the reader picks the newest `ts`) |
| series key spelling | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/api.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-service/tests/api.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test api` passes, against a seeded store with no source reachable, the failing-first tests `verdicts_are_newest_evaluation_per_series_worst_first`, `unknown_verdict_has_no_score_key`, `verdicts_filter_by_signal`, `series_returns_samples_and_evaluations_in_ts_order`, `series_gap_is_not_filled`, `series_range_is_half_open` and `unknown_series_answers_404`.

## Note from story:m1-instance-registry (2026-10-15)

Design § 11 D19: the definition file is the only copy of a definition. Over the file-backed `Instance` ports a generated `ProposeDefinition`, `ReviseDefinition`, `MuteDefinition`, `UnmuteDefinition` or `RetireDefinition` changes nothing, yet the generated behaviour answers success (the ports' `put` cannot fail; the refusal is only an `Instance::refused_writes()` entry). The service API never routes those commands to an `Instance`: it answers them with a refusal naming D19 (definitions change by editing the file), with a failing-first test that a retire request leaves the response a refusal and the file unchanged. `UnmuteDefinition` recorded by the scheduler (D19) goes to the Eventlog store, not to the `Instance`.

From review-result:adversary-m2-spec-history-records-pass-2, finding 5: a network `RecordTransition` that breaks a `Transition` invariant (`first_ts <= settled_ts`, `from_verdict` absent or different from `to_verdict`, `definition_rev >= 1`) is answered 501 with no declared error by the generated server (`generated/thalamus-model/src/server/entry.rs`). When this story routes commands to an instance, such a request is answered with a declared refusal, and a test shows it.

Open points found by story:m4-design-record (wave 20261015e); coordinator defaults, each recorded as a design § 11 row when this story lands:

- This story's API decisions (no `score` key on `UNKNOWN`, 404 for an unknown series, 7-day default range) get a § 11 row when it lands.
