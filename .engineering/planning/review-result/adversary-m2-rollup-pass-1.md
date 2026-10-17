---
format: aep.planning-md/3
id: review-result:adversary-m2-rollup-pass-1
kind: review-result
status: active
title: Adversary, story:m2-rollup, pass 1
relations:
- reviews: story:m2-rollup
revision: 1
---
unit: story:m2-rollup, 5d16c33 as merged at wave/20261017b 753dc08
verdict: NEEDS-CHANGE
cases: `crates/thalamus-service/tests/adv_rollup_p1.rs` and `crates/thalamus-cli/tests/adv_rollup_p1.rs`, written from reading, run by the correction
origin: introduced 4 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `adv_late_tick_after_midnight_counts_the_missed_slots_of_the_closed_day` | the rollup hook (`scheduler.rs:266`) runs before the missed-slot pass, so a late first tick after midnight writes the closed day's row without the unknown samples it backfills, and the row cannot be written again | the hook runs after both recording passes |
| F2 | warning | `adv_service_rolls_up_a_day_whose_midnight_it_missed` | only the day before now is rolled up (`daily_rollup.rs:20`), so a day whose midnight passed while the service was down never gets a row from the service | every closed day with samples and no row is rolled up |
| F3 | note | judgement | every tick builds each series' rollup over a full sample and evaluation scan before learning the day is already rolled up (`rollups.rs:127-145`) | a series and day that already has a row is skipped before building |
| F4 | note | `adv_rollup_signal_no_definition_has_is_refused` | `--signal` naming no definition rolls up nothing and exits 0 (`cmd/rollup.rs:95`) | refused as `UnknownSignal` |

Attacked and held: UTC day boundaries, offsets and a sample at exactly 00:00:00Z; D14 highest revision in any evaluation order; values from `ok` readings only, absent without one; p50 of an even count; N/A and UNKNOWN in the worst verdict and counts; min and max score over scored verdicts only; a re-run writes nothing; midnight counts as closed; rollup ids cannot collide; two services rolling up at once.
