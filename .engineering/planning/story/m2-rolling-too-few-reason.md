---
format: aep.planning-md/3
id: story:m2-rolling-too-few-reason
kind: story
status: implemented
title: A rolling baseline short of ok samples says so, not that it predates its series
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-store-baselines
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T02:11:19Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-17T02:11:19Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-17T03:47:24Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Context

A `rolling_median` baseline over fewer than `evaluation.rolling_n` `ok` samples is `UNKNOWN` with reason `baseline predates series`. For a series that is old but mostly `unknown` that reason is wrong: the series does not predate the baseline, it lacks readable samples. Found by adversary pass 1 of `story:m2-store-baselines` (finding F3), accepted for M2.

## Work

- A distinct reason for a rolling baseline whose series is older than `rolling_n` samples but holds fewer than `rolling_n` `ok` ones; `baseline predates series` stays for a series with fewer than `rolling_n` samples of any status.

## Scope

- `crates/thalamus-core/src/baseline/rolling.rs`, `crates/thalamus-core/src/baseline/mod.rs`
- `crates/thalamus-core/tests/store_baselines.rs`
- `website/docs/concepts/baselines.md`

## Acceptance

- `cargo test -p thalamus-core --test store_baselines` passes a failing-first test in which a series of `rolling_n` or more samples, fewer than `rolling_n` of them `ok`, folds to `UNKNOWN` with the new reason, and one in which a series of fewer than `rolling_n` samples keeps `baseline predates series`.
