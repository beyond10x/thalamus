---
format: aep.planning-md/3
id: story:store-sqlite-id-reserve-locked
kind: story
status: implemented
title: Reserving an id on SQLite waits out a held write lock instead of panicking
relations:
- decomposes: epic:history-and-transitions
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T02:11:18Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-17T02:11:18Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-17T03:47:24Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2,"review_outcome":1}}}
---
## Context

The full gate of wave 20261016b (`task check`, `task test-postgres` step, SQLite target) failed once on `crates/thalamus-store/tests/adv_p2_eventlog_store.rs::adv_p2_two_threads_minting_and_recording_lose_no_sample` under host load:

```
thread '<unnamed>' panicked at crates/thalamus-store/src/eventlog.rs:862:13:
the next smp_ id cannot be reserved on the instance's id stream: eventlog: event store is unavailable: database is locked
```

The same step passed when rerun alone. A test that passes only on a rerun is a defect: two writers on one SQLite stream can still see `SQLITE_BUSY` while reserving an id, and `next_number` panics on it, so a concurrent `thalamus sample` and `thalamus-service run` could crash under load.

## Work

- Find why the id reservation (`EventlogStore::reserve`) gets `database is locked` instead of waiting on the busy handler, and make it wait or retry within a bounded time.
- Make the test fail on demand (for example by holding a write transaction from a third handle) so the fix is proven, not hoped.

## Scope

- `crates/thalamus-store/src/eventlog.rs`
- `crates/thalamus-store/tests/adv_p2_eventlog_store.rs`

## Acceptance

- `cargo test -p thalamus-store --test adv_p2_eventlog_store` passes a failing-first test in which a writer reserving an `smp_` id while another handle holds the SQLite write lock for less than the busy timeout gets its id and stores its sample, and `adv_p2_two_threads_minting_and_recording_lose_no_sample` passes 20 consecutive runs.
