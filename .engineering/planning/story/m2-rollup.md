---
format: aep.planning-md/3
id: story:m2-rollup
kind: story
status: draft
title: thalamus rollup writes one sample rollup per series per UTC day
relations:
- decomposes: epic:history-and-transitions
- depends_on: epic:signal-contract
- depends_on: story:m2-spec-history-records
- depends_on: story:m2-scheduler
- depends_on: story:m2-auto-unmute
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/rollup.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/rollup.rs
- confidence: inferred
  path: crates/thalamus-core/src/rollup.rs
- confidence: inferred
  path: crates/thalamus-core/tests/rollup.rs
- confidence: inferred
  path: crates/thalamus-service/src/daily_rollup.rs
- confidence: inferred
  path: crates/thalamus-service/src/scheduler.rs
- confidence: inferred
  path: crates/thalamus-service/tests/daily_rollup.rs
- confidence: inferred
  path: crates/thalamus-store/src/rollups.rs
- confidence: inferred
  path: crates/thalamus-store/tests/conformance.rs
revision: 5
---
## Context

Design § 4: a daily rollup is "one row per series per UTC day: `n`, `unknown`, `partial`, `min/p50/max/last`, worst verdict, verdict counts, min/max score", kept after raw samples leave the retention window; it is "what a `history: store` source cannot replay". Design § 11 D14: a rollup counts each sample's evaluation at the highest `definition_rev` present when the rollup is built. `thalamus rollup` is in the CLI of design § 5.1. Lesson L7 puts recurring work on the scheduler, so the service rolls up each day that closes. The record `DailyRollup` is specified by `story:m2-spec-history-records`.

The scheduler hook line in `crates/thalamus-service/src/scheduler.rs` is the second of three: this story lands after `story:m2-auto-unmute` (`depends_on`), and `story:m2-transitions` after this one.

## Work

- `crates/thalamus-core/src/rollup.rs` (pure): from the samples of one series on one UTC day and their evaluations:
  - `n`, `unknown`, `partial`, and `min`, `p50`, `max`, `last` over the `ok` readings only; with no `ok` reading the four values are absent, never 0. `partial` readings are counted, not valued (a bound is not a value, design § 2.2);
  - D14: of each sample's evaluations, only the one at the highest `definition_rev` present is counted;
  - worst verdict (the ladder's worst-of, `story:m1-verdict-grading`), one count per ladder verdict, and `min_score` / `max_score` over the scored verdicts; absent, never 0, when the day has no evaluation or no scored one.
- `crates/thalamus-store/src/rollups.rs`: append and read of `DailyRollup`, refusing a second row for the same series and day (`RecordRollup` outcome `already-rolled-up`). The rollup id spelling is held by a Rust test, as the `UNMAPPED:` marker in `spec/domains/rollup.yaml` says.
- `crates/thalamus-store/tests/conformance.rs` (owned by `story:m1-eventlog-store`): one selection line adding the `RecordRollup` scenarios, so they run with the Eventlog storage.
- `crates/thalamus-cli/src/cmd/rollup.rs`: `thalamus rollup [--day YYYY-MM-DD] [--signal <id>]` rolls up every series with samples on a closed UTC day (default: yesterday); refuses a day that has not closed; a re-run writes nothing and prints the rows that exist.
- `crates/thalamus-service/src/daily_rollup.rs`: after each UTC midnight, roll up the day that closed; `crates/thalamus-service/src/scheduler.rs` (owned by `story:m2-scheduler`): one hook line calling it.
- Not here: pruning raw samples at the end of retention (not in M2).

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `DailyRollup` → `SignalDefinition` (`definition`, via `signal`) | many-to-one | references | declared by `story:m2-spec-history-records` in `spec/domains/rollup.yaml`, on the same ground as `SignalSample.definition` (`spec/domains/observation.yaml:161`) |
| `SignalSample` → `SignalDefinition` (`definition`, via `signal`): a rollup reads the samples of one series | many-to-one | references | `spec/domains/observation.yaml:161`, series key `spec/domains/observation.yaml:20` |
| `SignalEvaluation` → `SignalSample` (`sample`, via `sample_id`): the verdict fields read each sample's evaluation at the highest revision | many-to-one | references | `spec/domains/evaluation.yaml:109`; design § 11 D14 |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/rollup.rs`, `crates/thalamus-store/src/rollups.rs`, `crates/thalamus-service/src/daily_rollup.rs`, `crates/thalamus-cli/src/cmd/rollup.rs` (their module and subcommand lines are `story:m0-workspace`'s)
- `crates/thalamus-service/src/scheduler.rs` (one hook line; ordered after `story:m2-auto-unmute`)
- `crates/thalamus-store/tests/conformance.rs` (one selection line)
- `crates/thalamus-core/tests/rollup.rs`, `crates/thalamus-cli/tests/rollup.rs`, `crates/thalamus-service/tests/daily_rollup.rs` (new)

## Acceptance

- `cargo test -p thalamus-core --test rollup` passes the failing-first tests `rollup_counts_n_unknown_partial`, `rollup_values_from_ok_readings_only`, `rollup_without_ok_reading_has_no_values`, `rollup_day_is_utc`, `rollup_worst_verdict_and_verdict_counts`, `rollup_min_max_score_over_scored_verdicts_only` and `rollup_counts_highest_rev_evaluation_per_sample` (D14).
- `cargo test -p thalamus-cli --test rollup` passes `rollup_rerun_writes_nothing`, `rollup_refuses_open_day` and `rollup_id_spelling_is_series_key_and_day`.
- `cargo test -p thalamus-service --test daily_rollup` passes `service_rolls_up_closed_day`.
- `cargo test -p thalamus-store --test conformance` passes `thalamus.rollup.RecordRollup/outcome/recorded` and `thalamus.rollup.RecordRollup/outcome/already-rolled-up` with the Eventlog storage.
