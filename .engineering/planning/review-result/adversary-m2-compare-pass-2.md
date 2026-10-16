---
format: aep.planning-md/3
id: review-result:adversary-m2-compare-pass-2
kind: review-result
status: active
title: Adversary, story:m2-compare, pass 2
relations:
- reviews: story:m2-compare
revision: 1
---
unit: story:m2-compare, correction 1, commit 38734b7
verdict: NEEDS-CHANGE
cases: executed 63→72 (`crates/thalamus-cli/tests/adv_compare_p2.rs` 9), red 4
origin: introduced 4 / pre-existing 0 / undecided 1

## Pass-1 findings rechecked

F1 (`compare.rs:314`), F2 (`compare.rs:263`), F3 (`compare.rs:251`), F4 (`adv_compare_p1.rs:164-222`) and F5 (`status.rs:101`) are closed.

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| P2-F1 | note | `adv2_compare_a_sample_at_the_last_instant_is_current_at_the_last_instant` | at the last RFC 3339 instant the range read excluded a sample stored at exactly `--at` (`compare.rs:263`) | the instant itself is read |
| P2-F2 | warning | `adv2_compare_offset_past_since_predates_even_when_a_sample_is_stored` | the series' age came from its first stored sample, not from `denominator.since`; with `since` after stored samples, `--vs 7d` compared against a sample before it and exited 0 (`compare.rs:312`) | decided: the design wins (line 115, § 11 D18); a series begins at its resolved `denominator.since`, else at its first stored sample |
| P2-F3 | note | `adv2_compare_age_comes_from_since_not_the_first_stored_sample` | with `since` before the first stored sample, the reason read `baseline predates series` instead of `no stored sample at offset` | as P2-F2 |
| P2-F4 | judgement | `adv2_compare_on_a_read_only_store_is_unavailable` | on a store the user cannot write, compare exits 74 | decided: no change; a store the user cannot write is unavailable for every command; the case asserts 74 |
| P2-F5 | warning | three green `adv_compare_p2` cases | three mutants (the `ts <= at` filter dropped, an inclusive age check, `0` printed for a ratio over a zero baseline) passed both compare test files | the pass-2 cases kill them |

Noted without a test: on a PostgreSQL store, a read on an instance that never sampled runs schema setup.
