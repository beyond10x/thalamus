---
format: aep.planning-md/3
id: story:m5-service-answers
kind: story
status: draft
title: The service answers every open assertion once its horizon has passed
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-assert
- depends_on: story:m2-transitions
- depends_on: story:m4-live
scope:
- confidence: inferred
  path: crates/thalamus-service/src/assertions.rs
- confidence: inferred
  path: crates/thalamus-service/src/scheduler.rs
- confidence: inferred
  path: crates/thalamus-service/tests/assertions.rs
revision: 5
---
## Outcome

An assertion nobody re-runs is still answered: the service answers every open assertion once its horizon has passed, so a merge request, a release or a controller reads a recorded answer later, from the CLI or from the service's HTTP API.

## Context

Design § 11 D3: "the service answers only after `after + horizon`". The `Answerer` actor of `spec/domains/assertion.yaml` is "the service, which answers an assertion from the evaluations over its window". Lesson L7 (design § 1) gives recurring work to the scheduler, not to whoever remembers. The answering logic is `story:m5-assert`'s (`crates/thalamus-pipeline/src/assert.rs`, `answer_open`); this story only calls it on each tick.

Design § 5.1 gives the service "the HTTP API (current verdicts, series, assertions)". `story:m4-service-api` ships the first two and leaves the assertions to M5; this story fills them. `story:m5-skeleton` lays the routes `GET /api/assertions` and `GET /api/assertions/{id}` in the route table of `story:m4-skeleton` with stub handlers in `crates/thalamus-service/src/assertions.rs`.

`crates/thalamus-service/src/scheduler.rs` is owned by `story:m2-scheduler`; three M2 stories each add one hook line, the last `story:m2-transitions`, and `story:m4-live` adds a fourth after it. This story adds the fifth, after `story:m4-live` (`depends_on`). No later story edits the file (`story:m5-testrun-ingest` puts its skip in `crates/thalamus-pipeline/src/sample.rs`).

Its HTTP tests use the in-process technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `crates/thalamus-service/src/assertions.rs`:
  - on each tick, after grading and after the transition hook, read `OpenAssertions` and answer each whose `after + horizon <= now` through `answer_open`. An assertion the CLI answered first (a state conflict) is skipped silently.
  - `GET /api/assertions[?asker=<kind>:<ref>&state=asked|answered]`: the `thalamus.assertion.Assertions` view rows (`spec/domains/assertion.yaml:295`), newest `after` first, then assertion id, filtered by asker and state when given; `application/json`.
  - `GET /api/assertions/{id}`: one assertion with its result, worst, reason and the ids of the evaluations it cites; an open one is returned with no result; an unknown id answers 404.
  - No write route: asking stays the CLI's (`story:m5-assert`), and every other method answers 405 (`story:m4-skeleton`).
- `crates/thalamus-service/src/scheduler.rs`: one hook line calling the tick function, after the line of `story:m4-live`.
- Clock injected, as in `story:m2-scheduler`.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `Assertion` → `SignalEvaluation`, `evaluations`, set on `answer` | many | references | declared by `story:m5-spec-integration-records`; design § 11 D2 |
| `Assertion` lifecycle `Asked` → `Answered`, terminal | one move per assertion | the assertion owns its answer | `spec/domains/assertion.yaml` (`lifecycle`, transition `answer`) |
| `Assertion` → `Asker` (`kind`, `ref`): the list filter | one per assertion | value | `spec/domains/assertion.yaml:17` (`AskerKind`) |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/assertions.rs` (module line and route stubs are `story:m5-skeleton`'s)
- `crates/thalamus-service/src/scheduler.rs` (one hook line; after `story:m4-live`; the last story that edits it)
- `crates/thalamus-service/tests/assertions.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test assertions` passes, on a fake clock with no source reachable, the failing-first tests:

- `open_assertion_not_answered_before_its_horizon`, `open_assertion_answered_on_first_tick_after_its_horizon`, `answer_is_not_repeated_after_restart` and `assertion_answered_by_cli_first_is_left_alone`;
- `api_assertions_lists_every_assertion_filtered_by_asker_and_state`, `api_assertion_by_id_carries_result_worst_and_cited_evaluations`, `api_open_assertion_has_no_result` and `api_unknown_assertion_id_answers_404`.
