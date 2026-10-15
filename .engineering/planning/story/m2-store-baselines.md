---
format: aep.planning-md/3
id: story:m2-store-baselines
kind: story
status: draft
title: Grade against same_slot_median_4w and rolling_median from stored samples
relations:
- decomposes: epic:history-and-transitions
- depends_on: epic:signal-contract
- depends_on: story:m2-spec-history-records
- depends_on: story:m1-cli-eval-show
scope:
- confidence: inferred
  path: crates/thalamus-cli/tests/eval_store_history.rs
- confidence: inferred
  path: crates/thalamus-core/src/baseline/mod.rs
- confidence: inferred
  path: crates/thalamus-core/src/baseline/rolling.rs
- confidence: inferred
  path: crates/thalamus-core/src/baseline/same_slot.rs
- confidence: inferred
  path: crates/thalamus-core/tests/store_baselines.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/evaluate.rs
revision: 9
---
## Context

Design § 3.1 adds two baselines that only stored history can answer: `same_slot_median_4w`, "median of the same weekday and hour over the last 4 weeks", and `rolling_median`, "median of the last N samples of this series", for `history: store` signals (`thalamus.definition.History`, `spec/domains/definition.yaml:198`). M1 ships the other six kinds (`story:m1-baselines`). Rules 1 and 2 of § 3.1 hold here: a baseline that could not be read is `UNKNOWN`, never 0, and one older than its series is `UNKNOWN` with reason `baseline predates series`. `rolling_n` is the field `story:m2-spec-history-records` adds to `EvaluationSpec` (`depends_on`). The grading call site is in `crates/thalamus-pipeline/src/evaluate.rs`, owned by `story:m1-cli-eval-show` (`depends_on`), so `thalamus eval` and the scheduler grade with both kinds from that one change.

## Work

- `crates/thalamus-core/src/baseline/same_slot.rs`: the median of the `ok` readings at the same UTC weekday and hour 7, 14, 21 and 28 days before the graded `ts` (the sample closest to each instant within that hour). Any of the four slots without an `ok` reading makes the baseline `UNKNOWN` (reason `same slot not read`); a slot before the first sample of the series makes it `UNKNOWN` (reason `baseline predates series`).
- `crates/thalamus-core/src/baseline/rolling.rs`: the median of the last `rolling_n` `ok` readings before the graded sample; `partial` and `unknown` samples are skipped; fewer than `rolling_n` makes it `UNKNOWN` (reason `baseline predates series`).
- The median of an even count is the mean of the two middle values, in `Decimal`. Pure: the caller passes the samples.
- `crates/thalamus-core/src/baseline/mod.rs` (owned by `story:m1-baselines`): the two "not in M1" refusal arms of the dispatch become calls into `same_slot.rs` and `rolling.rs`.
- `crates/thalamus-pipeline/src/evaluate.rs` (owned by `story:m1-cli-eval-show`): for these two kinds the grading call site fetches the series history through the store's series range read (`crates/thalamus-store/src/range.rs`, `story:m1-eventlog-store`): 28 days and one hour, or the last `rolling_n` `ok` samples. No store file changes.
- Decisions taken here (reversible): slots are UTC, as rollup days are (design § 4); a missing week is `UNKNOWN`, not a median of three, by § 3.1 rule 1.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalSample` → `SignalDefinition` (`definition`, via `signal`): the history read is of one series of the graded signal | many-to-one | references | `spec/domains/observation.yaml:161`; series key `spec/domains/observation.yaml:20` (D10) |

Samples are read as `thalamus.observation.SeriesSamples` serves them (`spec/domains/observation.yaml:304`): one series, `ts` ascending, a gap left a gap.

## Scope

Planned (inferred):

- `crates/thalamus-core/src/baseline/same_slot.rs`, `crates/thalamus-core/src/baseline/rolling.rs`
- `crates/thalamus-core/src/baseline/mod.rs` (two dispatch arms)
- `crates/thalamus-pipeline/src/evaluate.rs` (the grading call site)
- `crates/thalamus-core/tests/store_baselines.rs`, `crates/thalamus-cli/tests/eval_store_history.rs` (new)

## Acceptance

- `cargo test -p thalamus-core --test store_baselines` passes the failing-first tests `same_slot_median_of_four_weeks`, `same_slot_missing_week_is_unknown`, `same_slot_before_series_is_unknown`, `same_slot_uses_utc_weekday_and_hour`, `rolling_median_of_last_n_ok`, `rolling_median_skips_unknown_and_partial` and `rolling_median_too_few_is_unknown`.
- `cargo test -p thalamus-cli --test eval_store_history` passes `eval_grades_store_history_definition_from_stored_samples`.

## Note from story:m2-spec-history-records (2026-10-15)

The specification now declares `persistence`, `evaluation.fixed` and `evaluation.rolling_n` on a definition, but the definition-file loader (`crates/thalamus-core/src/registry.rs`) sets all three to `None`. This story also teaches the loader to read the field(s) it consumes, with the registry's rules (repeated keys refused, values validated), and a failing-first loader test; `website/docs/concepts/signals.md` lists the field.

## Note, superseding the one above (2026-10-15)

The loader now reads `evaluation.fixed`, `evaluation.rolling_n` and `persistence` in story:m2-spec-history-records (its adversary pass 1 found the gap with failing cases); this story no longer edits `crates/thalamus-core/src/registry.rs` for them.
