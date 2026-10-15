---
format: aep.planning-md/3
id: story:m4-live
kind: story
status: draft
title: Evaluations stream as server-sent CloudEvents and feed the live view
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-skeleton
- depends_on: story:m2-transitions
- depends_on: story:m1-cli-eval-show
scope:
- confidence: inferred
  path: crates/thalamus-pipeline/src/cloudevents.rs
- confidence: inferred
  path: crates/thalamus-pipeline/tests/cloudevents.rs
- confidence: inferred
  path: crates/thalamus-service/src/events.rs
- confidence: inferred
  path: crates/thalamus-service/src/live.rs
- confidence: inferred
  path: crates/thalamus-service/src/scheduler.rs
- confidence: inferred
  path: crates/thalamus-service/tests/live.rs
revision: 6
---
## Context

Design § 5.1: `thalamus-service` serves "a server-sent-events stream of evaluations"; § 6.2: every evaluation is available as a CloudEvents event (`type: dev.beyond10x.thalamus.evaluation`, `subject` the series key, `time` = `ts`); § 6.5: the live view is the "current verdict per series, updated from the event stream". The route lines, `AppState` and its broadcast sender come from `story:m4-skeleton`; this story fills `crates/thalamus-service/src/live.rs` and adds the publishing hook to the scheduler. `evaluate.rs` of `story:m1-cli-eval-show` returns the evaluations each pass recorded; the hook hands them to the broadcast.

`crates/thalamus-service/src/scheduler.rs` is owned by `story:m2-scheduler`; `story:m2-auto-unmute`, `story:m2-rollup` and `story:m2-transitions` each add one hook line in that order. This story adds a fourth, after grading, and lands after `story:m2-transitions` (`depends_on`).

One CloudEvents envelope (coordinator decision C6, round 1): `story:m2-transitions` builds the transition envelope in `crates/thalamus-service/src/events.rs`. This story moves that builder into `crates/thalamus-pipeline/src/cloudevents.rs` (stub and module line from `story:m4-skeleton`), so `events.rs` and `live.rs` both call it and `thalamus-cli` (`story:m5-conductor-contract`) can call it without linking the service. `events.rs` is owned by `story:m2-transitions` and edited here after it (`depends_on`).

Its HTTP and SSE tests use the in-process technique of `story:m4-skeleton` (`http::router`, `tower::ServiceExt::oneshot`, the SSE body read frame by frame with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `crates/thalamus-pipeline/src/cloudevents.rs` (pure): `envelope(kind, source, subject, time, id, data) -> serde_json::Value`, the CloudEvents 1.0 structured-mode envelope of design § 6.2 (`specversion` `1.0`, `type`, `source` the instance and producer, `subject`, `time`, `id`, `datacontenttype` `application/json`, `data`), with `kind` one of `dev.beyond10x.thalamus.transition` and `dev.beyond10x.thalamus.evaluation`.
- `crates/thalamus-service/src/events.rs` (owned by `story:m2-transitions`): its envelope code replaced by a call to `cloudevents::envelope`; the emitted transition events are byte-identical to before.
- `crates/thalamus-service/src/scheduler.rs`: one hook line after grading, `live::publish(&state, &recorded)`.
- `crates/thalamus-service/src/live.rs`:
  - `publish`: each recorded evaluation is sent on the broadcast channel; a send with no subscriber is not an error, and publishing never blocks the scheduler.
  - `GET /api/evaluations/stream`: `text/event-stream`; one SSE message per evaluation, `event: dev.beyond10x.thalamus.evaluation`, `id:` the evaluation id, `data:` the envelope from `cloudevents::envelope` (`subject` the series key of D10, `time` = `ts`, `data` the `thalamus.evaluation/1` record of design § 2.3). A subscriber that lags past the channel capacity gets one SSE comment naming the lag and is closed; it rereads `/api/verdicts`.
  - the live table: seeded on start from the newest evaluation of each series in the store, then updated from the broadcast only.
  - `GET /views/live`: one row per series, worst first, verdict, score, `ts` and reason; `UNKNOWN` hatched, `N/A` grey, never drawn as `OK`. The page refreshes itself every 10 s with `<meta http-equiv="refresh">` and carries no `<script>`.

## Decisions taken here (decomposer call, reversible)

- The live page is fed by the event stream on the server and refreshes itself, instead of running a browser `EventSource` script, because no script is committed (`story:m4-skeleton`, the Rust-only rule). A browser script is the obvious later change if the operator rules that browser code is outside that rule.
- No replay on reconnect (`Last-Event-ID` is ignored): the current state is `/api/verdicts`, the history is the store.
- The envelope builder lives in `thalamus-pipeline`, which `thalamus-service` and `thalamus-cli` already depend on, not in `thalamus-core` (which holds no wire format) and not in the service (coordinator decision C6).

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/evaluation.yaml:115` |
| the current verdict of a series is its newest evaluation | one per series | none | `spec/domains/evaluation.yaml:292` (`UNMAPPED:`) |
| series key, the CloudEvents `subject` | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/src/cloudevents.rs` (stub from `story:m4-skeleton`), `crates/thalamus-pipeline/tests/cloudevents.rs` (new)
- `crates/thalamus-service/src/events.rs` (owned by `story:m2-transitions`; the envelope code replaced by the call; ordered after it)
- `crates/thalamus-service/src/live.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-service/src/scheduler.rs` (one hook line; shared with `story:m2-scheduler`, `story:m2-auto-unmute`, `story:m2-rollup`, `story:m2-transitions`; ordered after `story:m2-transitions`; `story:m5-service-answers` adds the next line after this story)
- `crates/thalamus-service/tests/live.rs` (new)

## Acceptance

- `cargo test -p thalamus-pipeline --test cloudevents` passes the failing-first tests `evaluation_envelope_has_type_subject_time_and_record` and `transition_envelope_has_type_subject_and_settling_time`.
- `cargo test -p thalamus-service --test transitions` (`story:m2-transitions`) still passes unchanged, `transition_emitted_as_cloudevent_with_series_subject` included, and `grep -n '"specversion"' crates/thalamus-service/src/events.rs crates/thalamus-service/src/live.rs` prints nothing (the envelope is built in one place).
- `cargo test -p thalamus-service --test live` passes, on a fake clock and a fake source driving the scheduler, the failing-first tests `stream_carries_each_recorded_evaluation_as_cloudevent` (type, `subject` = series key, `time` = `ts`, `data` the evaluation record), `stream_unknown_evaluation_has_no_score`, `publish_without_subscriber_does_not_block_the_tick`, `lagging_subscriber_is_closed_with_a_comment`, `live_table_seeded_from_store_then_updated_from_stream`, `live_page_shows_unknown_hatched_never_ok`, `live_page_orders_worst_first` and `live_page_has_no_script_and_refreshes`.

Open points found by story:m4-design-record (wave 20261015e); coordinator defaults, each recorded as a design § 11 row when this story lands:

- The SSE channel capacity is 256 events per subscriber.
