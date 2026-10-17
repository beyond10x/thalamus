---
format: aep.planning-md/3
id: review-result:adversary-m2-store-baselines-pass-1
kind: review-result
status: active
title: Adversary, story:m2-store-baselines, pass 1
relations:
- reviews: story:m2-store-baselines
revision: 1
---
unit: story:m2-store-baselines, uncommitted tree on e07e28c
verdict: NEEDS-CHANGE
cases: executed 299→305 (`crates/thalamus-core/tests/adv_store_baselines_p1.rs` 6), red 1; `crates/thalamus-cli/tests/adv_eval_store_history_p1.rs` 2 cases written, not run (build refused for disk)
origin: introduced 4 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | warning | `same_slot_reads_the_row_appended_last_at_one_instant_as_rolling_does` | of two `ok` rows at one instant the same-slot pick takes the one appended first (`same_slot.rs:83`), while `rolling_median` and design § 11 D6 take the one appended last; a re-read moves one store baseline and not the other | the row appended last at one instant wins; of two different instants equally close, the earlier still wins |
| F2 | note | judgement | `rolling_median` counts two rows at one instant as two of its last N samples (`rolling.rs:27-35`) | decided: one instant counts once, its row appended last (D6); stated in the doc comment |
| F3 | note | judgement | fewer than N `ok` samples is always `baseline predates series`, also for an old series of mostly unknown samples (`rolling.rs:38`) | accepted: the story names this reason; a distinct reason is follow-up work |
| F4 | note | judgement | `series_range` clones every sample and evaluation of the store whatever its bound, so each graded store-baseline sample adds a full-store scan (`evaluate.rs:531`, `range.rs:50-59`) | accepted for M2; a samples-only range read is follow-up work |

The concurrent-eval failure seen once under load (`adv_eval_show_p1.rs::two_concurrent_evals_grade_each_sample_once_per_revision`) cannot come from this change: its fixture names no store baseline, and the panic path (`eventlog.rs:558-560`) is reached before it.

Attacked and held: UTC weekday and hour under offsets; slot edges; ties between instants; partial-only slots; even-count medians in Decimal (never -0); rolling_n 0, 1, equal to the history, i64::MAX; unknown never 0; the graded sample never in its own baseline; predates before not-read; a store baseline beside another kind refused Ambiguous.
