---
format: aep.planning-md/3
id: review-result:adversary-m2-flap-measurement-pass-1
kind: review-result
status: active
title: Adversary, story:m2-flap-measurement, pass 1
relations:
- reviews: story:m2-flap-measurement
revision: 1
---
unit: story:m2-flap-measurement, 68bfdac on impl/m2-flap-measurement
verdict: NEEDS-CHANGE
cases: `crates/thalamus-cli/tests/adv_flaps_p1.rs` 4 written, red 2
origin: introduced 4 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `adv_settled_counts_history_regraded_after_revision` | kept evaluations replay in id order (`flaps.rs:315`), so history re-graded after a revision carries higher ids than newer samples and the replay drops it as late: 0 settled changes where raw counts 4 and 3 are due | replay in sample ts order, the order the range read returns; within one kept revision each sample has one evaluation (D5) |
| F2 | warning | `adv_one_day_window_is_one_series_day` | series-days count distinct UTC calendar days (`flaps.rs:351`), so `--since 1d` spans two days and halves every per-series-day rate | a series-day is observed time: kept evaluations in the window times the definition's cadence over 24 h |
| F3 | note | judgement | a series settled WARN before `--since` that clears inside the window shows that change as its first settled verdict, not counted (`flaps.rs:319`), while raw counts it through `prev` | the settled replay is seeded with the series' kept evaluations before `--since`; only changes inside the window count |
| F4 | note | judgement | raw changes compare against the stored `prev`, which can be an older revision's verdict, so the first evaluation of a new revision counts as a change | accepted as the VerdictChanges view defines it; the verdicts page says so |
| F5 | note | judgement | the unit's expected settled figures call the same replay with the same sort (`tests/flaps.rs:262`) | expected figures become literal numbers |

Attacked and held: UNKNOWN and N/A in the ladder against literal figures (raw 8, settled 6), unknown never OK and never clearing a settled WARN; the store's bytes and mtime unchanged; text and JSON output byte-identical across runs; empty store, other-revision exclusion, `--sweep` refusing `--k` with exit 64.
