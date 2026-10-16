---
format: aep.planning-md/3
id: review-result:adversary-m2-compare-pass-1
kind: review-result
status: active
title: Adversary, story:m2-compare, pass 1
relations:
- reviews: story:m2-compare
revision: 1
---
unit: story:m2-compare, commit e5b6b9c
verdict: NEEDS-CHANGE
cases: executed 52→63 (`crates/thalamus-cli/tests/adv_compare_p1.rs` 11), red 3
origin: introduced 4 / pre-existing 1 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | must-change | `adv_compare_slot_before_first_sample_predates_even_when_a_sample_is_near` | a slot before the series' first sample, with that sample within half a cadence, used it and exited 0; design § 3.1 rule 2 says UNKNOWN "baseline predates series" (`compare.rs:310`) | the slot is checked against the first sample before the nearest-sample search |
| F2 | note | `adv_compare_at_the_last_rfc3339_instant_is_answered` | `--at 9999-12-31T23:59:59.999999999Z` was refused with exit 64 (`compare.rs:254`) | the last instant is answered |
| F3 | judgement | `adv_compare_on_a_fresh_instance_creates_no_store` | compare on an instance that never sampled created `.thalamus/store.sqlite` (`compare.rs:247`) | decided: a read command creates nothing; a missing store answers as an empty one |
| F4 | warning | five passing `adv_compare_p1` cases | five one-line mutants (tolerance widened to a cadence, exclusive tolerance bound, the current sample matching itself, a sample at `--at` excluded, half rounded down in the ratio) each passed `tests/compare.rs` | the pass-1 cases kill all five |
| F5 | note | judgement, pre-existing | `crates/thalamus-docs/src/status.rs:101` still said compare and sample exit 70 | the claim is corrected |

Attacked and held: unknown and partial readings print no number and exit 3; `--at` with an offset or in lower case answers as `Z`; the `--vs` parser at its limits; exit codes 64, 65 and 74 as `sample`; a baseline of 0 prints no ratio and exits 0.
