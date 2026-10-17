---
format: aep.planning-md/3
id: story:m2-series-range-read
kind: story
status: implemented
title: Read one series' samples in a range without scanning the store
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

Grading a sample against a store baseline reads the series' history through `series_range`, which clones every sample and evaluation of the store whatever its bound (`crates/thalamus-pipeline/src/evaluate.rs`, `crates/thalamus-store/src/range.rs`). Each graded store-baseline sample therefore adds a scan of the whole store. Found by adversary pass 1 of `story:m2-store-baselines` (finding F4), accepted for M2.

## Work

- A samples-only range read of one series between two instants that does not clone evaluations or other series; `thalamus eval` uses it for store baselines.

## Scope

- `crates/thalamus-store/src/range.rs`
- `crates/thalamus-pipeline/src/evaluate.rs`

## Acceptance

- `cargo test -p thalamus-store` passes a failing-first test in which a range read of one series returns only that series' samples inside the bound, and `cargo test -p thalamus-cli --test eval_store_history` stays green.
