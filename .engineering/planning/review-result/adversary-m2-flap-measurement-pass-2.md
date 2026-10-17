---
format: aep.planning-md/3
id: review-result:adversary-m2-flap-measurement-pass-2
kind: review-result
status: active
title: Adversary, story:m2-flap-measurement, pass 2
relations:
- reviews: story:m2-flap-measurement
revision: 1
---
unit: story:m2-flap-measurement, corrections cc12a60 as merged at wave/20261017a da67fef
verdict: NEEDS-CHANGE
cases: `crates/thalamus-cli/tests/adv_flaps_p2.rs` 3 written, red 3
origin: introduced 3 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | warning | `adv_series_days_after_cadence_revision_stay_within_the_window`, `adv_rev_of_an_older_cadence_counts_its_own_observed_time` | series-days price every kept evaluation at the current definition's cadence (`flaps.rs:272`), so after a cadence revision or with `--rev` of an older one a one-day window counts 12 series-days | observed time comes from the series' sample timestamps in the window: last minus first plus the median spacing, capped at the window |
| F2 | note | `adv_zero_series_days_carry_no_rate` | a rate is computed from unrounded series-days, so 0 series-days print a rate of 2880 (`flaps.rs:399,410`) | rates come from the rounded series-days and are left out at 0 |
| F3 | note | judgement | the replay seed before `--since` is unbounded (`flaps.rs:247`) | the seed holds the newest max(k, k_crit, m) kept evaluations before the window |

Attacked and held: replay order by parsed instant with append order at equal instants; one evaluation per sample per kept revision; the seed limited to the selected revision; an all-UNKNOWN seed; missed slots counted as observed time consistently with the replay; half-up rounding; no evaluations, no rates.
