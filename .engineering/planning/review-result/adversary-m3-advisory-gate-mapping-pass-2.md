---
format: aep.planning-md/3
id: review-result:adversary-m3-advisory-gate-mapping-pass-2
kind: review-result
status: active
title: Adversary, story:m3-advisory-gate-mapping, pass 2
relations:
- reviews: story:m3-advisory-gate-mapping
revision: 1
---
unit: story:m3-advisory-gate-mapping, correction 1, commit 2738823
verdict: NEEDS-CHANGE
cases: executed 282→287 (`crates/thalamus-core/tests/adv_advisory_p2.rs` 5), red 2
origin: introduced 4 / pre-existing 0 / undecided 0

## Pass-1 findings rechecked

| id | state | evidence |
|---|---|---|
| F1 | closed | trailing fractional zeros are dropped before the exact comparison (`warn_only.rs:139`); `adv_hundred_times_a_baseline_with_trailing_zeros_is_warn` passes |
| F2 | closed | the score is formed exactly on the decimals (`warn_only.rs:184-229`); `adv_score_is_zero_at_a_baseline_past_f64_integers` passes |
| F6 | closed | the story Scope names the paths the change needs |
| F7 | closed | a non-count kind is refused (`warn_only.rs:44`, outcome `advisory-not-count` on both commands); `advisory_non_count_kind_is_refused_at_load` and the propose case pass |

## Findings

| id | severity | test | finding |
|---|---|---|---|
| P2-F1 | warning | `adv_p2_count_above_a_tiny_baseline_the_loader_admits_is_warn` | a baseline of 10^-38, which the loader admits, grades count 2 UNKNOWN instead of WARN (`warn_only.rs:166`); the story says "at any magnitude" |
| P2-F2 | warning | `adv_p2_count_past_i128_above_the_baseline_is_warn` | a count of 10^39 against baseline 3 grades UNKNOWN instead of WARN (`warn_only.rs:142`) |
| P2-F3 | note | `adv_p2` score cases | four mutants of the score's rounding (round-half-down, a dropped carry, kept trailing zeros, `%` for `rem_euclid`; `warn_only.rs:187`, `:196`) survived the unit's suite, because no test expected a score with a fractional digit; the pass-2 cases kill all four |
| P2-F4 | warning | judgement | design § 11 D12 fixes the unit `{finding}`, but a `fixed_warn_only` definition with another unit still loads; the specification names no refusal for it |

Attacked and held: exponent and negative baseline spellings, revise after propose with the same refusals, contract order of the refusals in `spec/domains/definition.yaml` against `input_refusals`.
