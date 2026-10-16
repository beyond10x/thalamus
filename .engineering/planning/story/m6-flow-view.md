---
format: aep.planning-md/3
id: story:m6-flow-view
kind: story
status: draft
title: The flow view shows each sample, evaluation and transition as it arrives, with pause, filter and lag state
relations:
- decomposes: epic:web-ui
- depends_on: story:m6-web-types
- depends_on: story:m6-web-skeleton
- depends_on: story:m4-live
- depends_on: story:m2-transitions
scope:
- confidence: inferred
  path: crates/thalamus-pipeline/src/cloudevents.rs
- confidence: inferred
  path: crates/thalamus-pipeline/tests/cloudevents.rs
- confidence: inferred
  path: crates/thalamus-service/src/flow.rs
- confidence: inferred
  path: crates/thalamus-service/tests/flow.rs
- confidence: inferred
  path: web/fixtures/flow-stream.txt
- confidence: inferred
  path: web/src/lib/flow.test.ts
- confidence: inferred
  path: web/src/lib/flow.ts
- confidence: inferred
  path: web/src/views/Flow.vue
revision: 4
---
## Context

`epic:web-ui` Ships: "A live flow view: the stream as it arrives (sample → evaluation → transition), with pause and filter, and a visible state when the stream lags or reconnects (D34: a lagging subscriber is closed and rereads `/api/verdicts`)." The M4 stream carries evaluations only (D34). An evaluation record carries no reading: it references its sample by `sample_id` (`spec/domains/evaluation.yaml:110`). Settled transitions are emitted by `crates/thalamus-service/src/events.rs` (`story:m2-transitions`), not served over HTTP. So the flow needs a server half: `GET /api/flow/stream` (`story:m6-design-record` item 4), filling the stub `crates/thalamus-service/src/flow.rs` whose route line and module line `story:m6-web-skeleton` lays.

No scheduler hook is added. `flow.rs` subscribes to the evaluation broadcast in `AppState` (`story:m4-skeleton`, published by `story:m4-live`), reads the sample of each evaluation from the store (`crates/thalamus-store/src/samples.rs:79`, `sample`) and the transitions of its series through the read of `story:m2-transitions`. The scheduler records a transition (the hook of `story:m2-transitions`) before it publishes the evaluation (the hook of `story:m4-live`, after it), so the transition an evaluation settled is in the store when the evaluation arrives. `crates/thalamus-service/src/scheduler.rs`, which `story:m5-service-answers` edits last, stays untouched.

`crates/thalamus-pipeline/src/cloudevents.rs` and its test file are owned by `story:m4-live`; this story adds one kind there, after it (`depends_on`). Its HTTP and SSE tests use the in-process technique of `story:m4-skeleton`; it adds no dependency.

The browser half fills the stub `web/src/views/Flow.vue`, through the client of `story:m6-web-types`.

## Work

- `crates/thalamus-pipeline/src/cloudevents.rs`: `kind` gains `dev.beyond10x.thalamus.sample`; the envelopes of the two existing kinds stay byte-identical.
- `crates/thalamus-service/src/flow.rs`: `GET /api/flow/stream`, `text/event-stream`. For each evaluation the broadcast carries, in this order:
  1. `event: dev.beyond10x.thalamus.sample`, `id:` the sample id, `data:` the envelope of the `SignalSample` record of `sample_id` (`subject` the series key, `time` the sample `ts`); nothing when the store no longer holds that sample;
  2. `event: dev.beyond10x.thalamus.evaluation`, the same message `/api/evaluations/stream` sends for it (D34);
  3. `event: dev.beyond10x.thalamus.transition`, the envelope `events.rs` emits, only when a transition of that series has `settled_ts` equal to the evaluation `ts` and lists the evaluation in `evaluations`.
  A subscriber that lags past the channel capacity gets one SSE comment naming the lag and is closed (D34). No replay: `Last-Event-ID` is ignored.
- `crates/thalamus-service/tests/flow.rs` (new), with the golden answer `web/fixtures/flow-stream.txt` (`THALAMUS_BLESS=1` rewrites it); one test added to `crates/thalamus-pipeline/tests/cloudevents.rs`.
- `web/src/lib/flow.ts` (pure):
  - the log: newest first, at most 1000 entries, the oldest dropped; each entry its type, series key, `ts`, and for an evaluation its verdict and score, none for `UNKNOWN` and `N/A`. The sample, evaluation and transition of one evaluation form one group.
  - pause: arriving events are counted and held, within the same cap, and appended in arrival order on resume.
  - filter: event type, family (the first segment of the signal id), signal, and dimension values, each offered from the events received, never from a list in code.
  - stream state from `sse.ts`: `lagged` inserts a marker "events missed here" into the log and shows the state; `reconnecting` is shown in the header until the stream is `live` again.
- `web/src/lib/flow.test.ts`; `web/src/views/Flow.vue` (stub from `story:m6-web-skeleton`); each series key links to `#/series/<key>`.
- `web/dist/` rebuilt (the coordinator rebuilds it at merge).

## Decisions taken here (decomposer calls, reversible)

- A second stream route instead of more event types on `/api/evaluations/stream`, so the D34 contract and its consumers stay unchanged.
- The flow is derived from the evaluation broadcast plus two store reads, not from new publishing hooks, so neither `scheduler.rs` nor `events.rs` is edited.
- After a lag the flow view does not reread `/api/verdicts`: it is a log, not a state, so it marks the missed events and continues; the board rereads (`story:m6-signal-board`).
- The 1000-entry cap bounds the page's memory on a busy instance.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id` | many-to-one | references | `spec/domains/evaluation.yaml:110` |
| `SignalSample` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/observation.yaml:164` |
| `Transition` → `SignalEvaluation`, `evaluations` | many | references; the transition outlives them | `spec/domains/transition.yaml:56`, design § 11 D13 |
| the evaluation that settled a transition is the one at its `settled_ts` | one per transition | none | `spec/domains/transition.yaml:42` (`settled_ts`, "The ts of the settling evaluation"), design § 11 D13 |
| the family of a signal is the first segment of its id | one family per signal | none | inferred from `crates/thalamus-core/src/registry.rs:1186`; the id grammar is `UNMAPPED:` at `spec/domains/definition.yaml:10` |
| series key, the CloudEvents `subject` | one key per signal and dims | none | `spec/domains/observation.yaml:17` (design § 11 D10) |

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/src/cloudevents.rs`, `crates/thalamus-pipeline/tests/cloudevents.rs` (owned by `story:m4-live`; one kind and one test; after it)
- `crates/thalamus-service/src/flow.rs` (stub from `story:m6-web-skeleton`)
- `crates/thalamus-service/tests/flow.rs` (new)
- `web/fixtures/flow-stream.txt` (new)
- `web/src/views/Flow.vue` (stub from `story:m6-web-skeleton`)
- `web/src/lib/flow.ts`, `web/src/lib/flow.test.ts` (new)
- `web/dist/` is rebuilt by the coordinator at merge (`task web-build`) and, like `Cargo.lock`, is not in this story's typed scope, so the M6 view stories can share a wave

## Acceptance

`cargo test -p thalamus-pipeline --test cloudevents` passes the failing-first test `sample_envelope_has_type_subject_and_sample_time` beside the unchanged tests of `story:m4-live`, `cargo test -p thalamus-service --test flow` passes, on a fake clock and a fake source driving the scheduler, the failing-first tests `flow_sends_sample_then_evaluation_then_transition`, `flow_sends_no_transition_when_none_settled`, `flow_evaluation_message_equals_the_evaluation_stream_message`, `flow_lagging_subscriber_is_closed_with_a_comment`, `evaluation_stream_is_unchanged_by_flow` and `flow_stream_matches_golden`, and `npm test` in `web/` passes `flow_fixture_validates_against_generated_schema`, `flow_groups_sample_evaluation_and_transition_of_one_evaluation`, `pause_holds_events_and_resume_appends_in_order`, `log_is_capped_at_1000_entries`, `filter_values_are_read_from_events`, `lagged_state_inserts_a_missed_events_marker`, `reconnecting_state_is_shown_until_live` and `unknown_evaluation_shows_no_score`.
