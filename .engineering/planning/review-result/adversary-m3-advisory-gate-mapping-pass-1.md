---
format: aep.planning-md/3
id: review-result:adversary-m3-advisory-gate-mapping-pass-1
kind: review-result
status: active
title: Adversary, story:m3-advisory-gate-mapping, pass 1
relations:
- reviews: story:m3-advisory-gate-mapping
revision: 1
---
unit: story:m3-advisory-gate-mapping, uncommitted working tree on impl/m3-advisory-gate-mapping (base 789270f), worktree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15f-m3-advisory-gate-mapping
verdict: red
cases: executed 275→280, red 2
origin: introduced 7, pre-existing 0, undecided 0
wrote-outside-worktree: 7 paths under ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/ (part 6)
needs-coordinator: yes (scope: the unit edits 5 tracked source and spec paths outside its story Scope, F6)

`executed <before>`: 275. That is the 280 cases of my suite run minus the 5 in my own test binary `adv_advisory_p1`, which I excluded by subtracting its count. It is not a separate run. Every other test binary is unchanged.

## 1. `git --no-pager diff --stat` (worktree)

```
 59 files changed, 6360 insertions(+), 3735 deletions(-)
```

This is identical to the stat before I started. I changed no tracked file. My only write in the tree is one new untracked test file:

```
?? crates/thalamus-core/tests/adv_advisory_p1.rs
```

It is a test path, so the charter held. The two red cases ran before anything else, as rule 3 requires; part 2 holds that run. After all runs I applied rustfmt to that one file. The change was layout only (3 hunks of line wrapping) and no assertion changed. `cargo fmt -p thalamus-core -- --check` now exits 0. `b10x-gates scan-text` on the file printed "1 file(s) carry no private rule".

## 2. Cases added (crates/thalamus-core/tests/adv_advisory_p1.rs)

| test | asserts | now |
|---|---|---|
| `adv_hundred_times_a_baseline_with_trailing_zeros_is_warn` | count 300 against fixed baseline `3.` followed by 36 zeros is `WARN`, score 1, the same as against `3`. The loader's `is_decimal` admits that text, and `baseline::fold` passes it through verbatim. | **red** |
| `adv_score_is_zero_at_a_baseline_past_f64_integers` | at baseline 9007199254740992 (2^53) the count equal to it is `OK` with score `0`, and one below is `-1` | **red** |
| `adv_non_integer_baseline_scores_to_the_first_whole_count_above` | at baseline 2.5: count 2.5 scores 0, count 2 scores -1, count 0 scores -5, and count 3 is `WARN` with score 1 | green; kills mutant M1 |
| `adv_partial_read_is_unknown_partial_read_either_bound` | partial reads (lower 300, upper 1, upper 3) are all `UNKNOWN`, with no score and reason `partial read` (design § 3.1 rule 4) | green; kills mutant M2 |
| `adv_count_above_a_negative_baseline_is_warn` | baseline -1: count 0 is `WARN`, and count -1 is `OK` with score 0 | green; kills mutant M3 |

The first run of these cases alone, verbatim (`cargo test --locked -p thalamus-core --test adv_advisory_p1`, log `scratch/adversary/p1-cases.log`):

```
running 5 tests
test adv_count_above_a_negative_baseline_is_warn ... ok
test adv_hundred_times_a_baseline_with_trailing_zeros_is_warn ... FAILED
test adv_non_integer_baseline_scores_to_the_first_whole_count_above ... ok
test adv_partial_read_is_unknown_partial_read_either_bound ... ok
test adv_score_is_zero_at_a_baseline_past_f64_integers ... FAILED

---- adv_hundred_times_a_baseline_with_trailing_zeros_is_warn stdout ----
thread 'adv_hundred_times_a_baseline_with_trailing_zeros_is_warn' (1115046) panicked at crates/thalamus-core/tests/adv_advisory_p1.rs:141:5:
assertion `left == right` failed: count 300 against fixed baseline 3.000000000000000000000000000000000000: Graded { verdict: UNKNOWN, score: None, reason: "count 300 against fixed baseline 3.000000000000000000000000000000000000 is beyond the range the grader compares exactly", baseline: Baseline { kind: BaselineLabel("fixed"), offsets: [], reading: Ok(CompleteReading { value: Decimal("3.000000000000000000000000000000000000") }) } }
  left: (UNKNOWN, None)
 right: (WARN, Some("1"))

---- adv_score_is_zero_at_a_baseline_past_f64_integers stdout ----
thread 'adv_score_is_zero_at_a_baseline_past_f64_integers' (1115049) panicked at crates/thalamus-core/tests/adv_advisory_p1.rs:155:5:
assertion `left == right` failed: at the baseline: Graded { verdict: OK, score: Some(Decimal("-179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368")), reason: "count 9007199254740992 <= fixed baseline 9007199254740992", baseline: Baseline { kind: BaselineLabel("fixed"), offsets: [], reading: Ok(CompleteReading { value: Decimal("9007199254740992") }) } }
  left: (OK, Some("-179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368"))
 right: (OK, Some("0"))

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT=101
```

### Mutant probe

The probe ran in a scratch copy of the tree; the worktree was never edited. The patch is `scratch/adversary/mut1.patch`.

| id | mutation | killed by the unit's suite? | killed by |
|---|---|---|---|
| M1 | `warn_only.rs:82` `b.floor() + 1.0` → `b + 1.0` | no | `adv_non_integer_baseline_…` (score `-0.5`, expected `-1`) |
| M2 | `mod.rs:158` `Prepared::WarnOnly(_) => None` → `Some(rule.decide(value, baseline))` | no; `advisory_never_grades_crit` stays green | `adv_partial_read_…` (`partial 300 Lower` graded `WARN` 1, expected `UNKNOWN` `partial read`) |
| M3 | `warn_only.rs:144` `mantissa = -mantissa` → `mantissa.abs()` (sign dropped) | no | `adv_count_above_a_negative_baseline_is_warn` (count 0 vs -1 graded `OK` -1) |

The whole `thalamus-core` suite on the mutated copy (`cargo test --locked -p thalamus-core --no-fail-fast`, log `scratch/adversary/mut1-suite.log`) gave `passed 275 failed 5`. The 5 failures are all in `adv_advisory_p1`. Every other case stays green, including `advisory_gate` (6 passed) and `conformance` (10 passed).

## 3. Suite run (after the cases existed)

Command: `cargo test --locked -p thalamus-core --no-fail-fast`. Log: `scratch/adversary/p1-suite.log`.

```
     Running tests/adv_advisory_p1.rs (target/debug/deps/adv_advisory_p1-cc8a669fbd04a3c0)
test adv_score_is_zero_at_a_baseline_past_f64_integers ... FAILED
test adv_hundred_times_a_baseline_with_trailing_zeros_is_warn ... FAILED
test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/advisory_gate.rs (target/debug/deps/advisory_gate-6cab6e051eec47ff)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/conformance.rs (target/debug/deps/conformance-d742183280d5fc95)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     [every other binary: test result: ok; totals over all binaries: passed 278 failed 2]
error: 1 target failed:
EXIT=101
```

Clippy: `cargo clippy --locked -p thalamus-core --all-targets -- -D warnings` exited 0. Per the coordinator's rule change, I did not run the full `task check`.

## 4. Findings

| id | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| F1 | crates/thalamus-core/src/rules/warn_only.rs:153 | INFEASIBLE | introduced | `Exact` keeps trailing fractional zeros, unlike `threshold::Num::normal` (threshold.rs:654), so `compare` overflows i128. Count 300 against `3.` + 36 zeros grades `UNKNOWN` where `3` grades `WARN` (`adv_hundred_times_…`, exit 101). The module doc says the verdict is "decided exactly, on the decimals". | The loader and `baseline::fold` pass any `is_decimal` text through. Nothing I found writes a baseline with dozens of trailing zeros. Fix: normalise in `Exact::parse`, or reuse `threshold::Num`. |
| F2 | crates/thalamus-core/src/rules/warn_only.rs:82 | INFEASIBLE | introduced | The score is computed in f64. At baselines of 2^53 or more, `floor(b)+1 == b`, the division is 0/0, and `render` (line 105) writes the NaN as `f64::MIN`: a 309-digit negative score at the baseline, where "0 at the baseline" was promised (`adv_score_is_zero_…`, exit 101). | No realistic finding-count baseline is that large. Fix: compute `count - baseline` and `floor(baseline) + 1 - baseline` exactly on `Exact`, then divide; never map NaN to `f64::MIN`. |
| F3 | crates/thalamus-core/tests/advisory_gate.rs:199 | CONFIRMED | introduced | The partial-read case asserts only `!= CRIT`. Mutant M2 (a partial bound graded as a count, so it clears to `OK` or grades `WARN`) passes the whole suite. | Every partial read under `fixed_warn_only` goes through `Prepared::escalate`. `adv_partial_read_…` closes the gap. |
| F4 | crates/thalamus-core/tests/advisory_gate.rs:42 | CONFIRMED | introduced | Every case uses the integer baseline 3, so mutant M1 (`b + 1.0`) survives. The design row's "first whole count above the baseline" is untested. | Any non-integer `evaluation.fixed`, which the loader admits. `adv_non_integer_baseline_…` closes the gap. |
| F5 | crates/thalamus-core/src/rules/warn_only.rs:126 | CONFIRMED | introduced | Mutant M3 (sign dropped in `Exact::parse`) survives the suite. | Negative fixed baselines or counts. `adv_count_above_a_negative_baseline_is_warn` closes the gap. |
| F6 | crates/thalamus-core/src/behaviour/definition.rs | CONFIRMED | introduced | The story's Scope names 3 paths. The tree also changes `behaviour/definition.rs`, `registry.rs`, `tests/conformance.rs`, `thalamus-docs/src/status.rs` and `spec/domains/definition.yaml`, plus the generated output and website files. Meanwhile `scratch/outside-scope.patch` carries registry.rs, status.rs, status.json and verdicts.md as if they were not applied. | The story's own Work ("refused at load") needs the loader and the ESS outcome, so the Scope was too narrow. The coordinator decides whether to accept the out-of-scope edits. |
| F7 | crates/thalamus-core/src/rules/warn_only.rs:227 | CONFIRMED | introduced | No refusal ties `fixed_warn_only` to kind `count_of_cases` (D12 names that kind). `input_refusals` checks no kind for this rule, so a ratio or gauge definition with this rule loads. Its score step "to the first whole count above" then has no meaning. | Read from code only; I wrote no case. Note only. |

## 5. Attacked and not broken

- The empty baseline list is refused by propose, revise and load, all as `advisory-not-fixed` when `fixed` is absent. With `fixed` present, `fixed-number-unused` answers first, consistent with the declared order.
- Outcome order: `thresholds-unused` comes before `advisory-not-fixed`, and both come after `fixed-number-*`. Spec and Rust agree (spec/domains/definition.yaml outcomes 9 and 10 of both commands).
- Loader and propose parity: the loader runs the same `input_refusals`, and every advisory input the unit tests is refused by both.
- `grade()` not refusing: nothing outside `thalamus-core` calls `grading::grade`, so no unrefused definition reaches it.
- Missing count (unknown reading) and unknown baseline both grade `UNKNOWN` with no score. A printed 0 is `OK`.
- Huge counts that fit i128 (1e24) are `WARN` with score 1. Counts beyond i128 are `UNKNOWN`, the same as in the threshold rules. They are never `CRIT`.
- The OK score is never positive, because f64 rounding is monotone and `render` clamps it.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/p1-cases.log
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/p1-suite.log
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/p1-clippy.log
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/mut1-suite.log
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/mut1-clean.log
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary/mut1.patch
- ~/.cache/thalamus-waves/20261015f/m3-advisory-gate-mapping/scratch/adversary-pass-1.md (this report)
- The mutant copy at `scratch/adversary/mut1/`, including its `target/`, was cleaned with `cargo clean --manifest-path` through the host build runner (594.5 MiB) and then removed.

The `target/` build output in the worktree grew from my test, clippy and case runs. I did not remove it.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/rules/warn_only.rs", "line": 153, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "F1 (adv_hundred_times_a_baseline_with_trailing_zeros_is_warn): Exact keeps trailing fractional zeros, so a count 100 times a baseline written with 36 trailing zeros overflows the comparison and grades UNKNOWN instead of WARN"},
  {"file": "crates/thalamus-core/src/rules/warn_only.rs", "line": 82, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "F2 (adv_score_is_zero_at_a_baseline_past_f64_integers): the f64 score divides 0 by 0 at baselines from 2^53 and renders the NaN as f64::MIN, a 309-digit negative score where 0 at the baseline is promised"},
  {"file": "crates/thalamus-core/tests/advisory_gate.rs", "line": 199, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "F3 (adv_partial_read_is_unknown_partial_read_either_bound): the partial-read case asserts only != CRIT, so a rule that clears or warns on a partial bound passes the whole suite"},
  {"file": "crates/thalamus-core/tests/advisory_gate.rs", "line": 42, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "F4 (adv_non_integer_baseline_scores_to_the_first_whole_count_above): every case uses the integer baseline 3, so replacing floor(b)+1 with b+1 in the score survives the suite"},
  {"file": "crates/thalamus-core/src/rules/warn_only.rs", "line": 126, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F5 (adv_count_above_a_negative_baseline_is_warn): dropping the sign in Exact::parse survives the suite"},
  {"file": "crates/thalamus-core/src/behaviour/definition.rs", "line": null, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F6: the unit edits definition.rs, registry.rs, conformance.rs, status.rs and spec/domains/definition.yaml outside its story Scope while outside-scope.patch presents part of that as unapplied"},
  {"file": "crates/thalamus-core/src/rules/warn_only.rs", "line": 227, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F7: nothing ties fixed_warn_only to kind count_of_cases, so a ratio or gauge under it loads and gets a whole-count score step that means nothing"}
]
```
