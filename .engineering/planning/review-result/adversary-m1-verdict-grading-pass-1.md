---
format: aep.planning-md/3
id: review-result:adversary-m1-verdict-grading-pass-1
kind: review-result
status: active
title: Adversary, story:m1-verdict-grading, pass 1
relations:
- reviews: story:m1-verdict-grading
revision: 1
---
unit: story:m1-verdict-grading, uncommitted working tree of impl/m1-verdict-grading (base ca445f1) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-verdict-grading
verdict: NEEDS-CHANGE
cases: executed 311→320, red 5
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths kept (2 logs, this report); 2 deleted (a mutant copy after `cargo clean`, a mutant script)
needs-coordinator: F6 needs a design call: is an x-only line over a zero baseline refused, or is CRIT at the baseline accepted? F4: are negative gauges in scope?

## 1. `git --no-pager diff --stat` (worktree, at report time)

```
 crates/thalamus-core/src/grading.rs         | 148 +++++++
 crates/thalamus-core/src/ladder.rs          |  68 +++
 crates/thalamus-core/src/rules/mod.rs       | 141 ++++++
 crates/thalamus-core/src/rules/threshold.rs | 644 ++++++++++++++++++++++++++++
 crates/thalamus-docs/src/status.rs          |   6 +-
 docs/design/thalamus.md                     |   1 +
 website/data/status.json                    |   4 +-
 website/docs/concepts/score.md              |  13 +-
 website/docs/concepts/verdicts.md           |  16 +-
 9 files changed, 1029 insertions(+), 12 deletions(-)
```

This stat matches the one taken before I started, so all of it is the implementor's. I added one untracked test file, `crates/thalamus-core/tests/adv_grading.rs` (396 lines). It is the only path I wrote in the tree, and no implementation file was touched.

## 2. Cases added (`crates/thalamus-core/tests/adv_grading.rs`)

| case | asserts | now |
|---|---|---|
| `adv_exact_event_count_at_min_events_crosses_the_line` (:149) | 10 failures of 30, rendered as the adapters render them (`0.333333333333333333` from SQL, Loki and git; `0.3333333333333333` from the Prometheus f64), with warn `{x 2, abs 0.02, min_events 10}` over a baseline of 0.1, is WARN | **red** |
| `adv_reason_states_a_whole_event_count` (:170) | the design § 2.3 example, 15 of 556 rendered at 18 digits: the reason counts `15 events of 556` | **red** |
| `adv_value_better_than_a_negative_baseline_is_not_out_of_bounds` (:191) | `higher_is_worse` x 2 / x 4, value -11 against a baseline of -10, is OK | **red** |
| `adv_drop_to_zero_scores_no_better_than_a_drop_to_one` (:379) | `lower_is_worse` x 0.5 / x 0.25 over 100: the score of 0 is at least the score of 1 | **red** |
| `adv_grading_is_monotone_in_the_value` (:282) | across 2015 adjacent pairs (8 line sets × 5 baselines × 32 values, n 1000), a worse value never grades better in verdict or score | **red** (4 score pairs, all at value 0) |
| `adv_count_exactly_at_min_events_crosses` (:209) | a count of 10 crosses a line with `min_events: 10`; a count of 9 does not | green (kills M2) |
| `adv_worst_of_ok_and_na_is_na` (:224) | `worst([OK, NA])` and `worst([NA, OK])` both return NA; `pass_exit([NA, UNKNOWN])` is 3 | green (kills M1) |
| `adv_score_is_log_linear_between_multiplier_marks` (:238) | over 0.013 with x 2 / x 4: 0.0195 scores `0.585` and 0.039 scores `1.585` | green (kills M8a, M8b) |
| `adv_score_follows_the_stricter_part_of_a_line` (:263) | with the design's lines, 0.022 over 0.013 scores `0.759` (2 × 0.013 is the stricter part, not abs 0.02) | green (kills M10) |

Red output, each case run alone (`cargo test --locked -p thalamus-core --test adv_grading -- --exact <case>`) before anything else ran:

```
test adv_exact_event_count_at_min_events_crosses_the_line ... FAILED
assertion `left == right` failed: 10 failures of 30 (value 0.333333333333333333) meet min_events 10: Graded { verdict: OK, score: Some(Decimal("0.999")), reason: "under the WARN line: 0.333333333333333333, 9.99999999999999999 events of 30 < min_events 10", ... }
  left: OK
 right: WARN
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```
```
test adv_reason_states_a_whole_event_count ... FAILED
the reason counts 15 events of 556: "0.02697841726618705 >= 2 x 0.013 and >= 0.02, 14.9999999999999998 events of 556 >= min_events 10"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```
```
test adv_value_better_than_a_negative_baseline_is_not_out_of_bounds ... FAILED
assertion `left == right` failed: -11 is better than a baseline of -10 when higher is worse: Graded { verdict: CRIT, score: Some(Decimal("2")), reason: "-11 >= 4 x -10", ... }
  left: CRIT
 right: OK
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

I expected the monotone sweep to come back green. Its first run, which stopped at the first violation, was red:

```
test adv_grading_is_monotone_in_the_value ... FAILED
score improved (3.7 then 3): lower_is_worse warn Some(Threshold { x: Some(Decimal("0.5")), abs: None, min_events: None }) crit Some(Threshold { x: Some(Decimal("0.25")), abs: None, min_events: None }), baseline 0.013: 0.001 then 0: Graded { verdict: CRIT, score: Some(Decimal("3")), reason: "0 <= 0.25 x 0.013", ... }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.05s
```

I then changed my own sweep to collect every violation and added the focused case for D22's own example. Both were run alone:

```
test adv_drop_to_zero_scores_no_better_than_a_drop_to_one ... FAILED
a drop to 0 scores Some(Decimal("3")), a drop to 1 scores Some(Decimal("6.644"))
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

test adv_grading_is_monotone_in_the_value ... FAILED
4 of 2015 pairs grade a worse value better:
score improved (3.7 then 3): lower_is_worse ... x 0.5 ... x 0.25 ..., baseline 0.013: 0.001 then 0
score improved (8.966 then 3): ... baseline 0.5: 0.001 then 0
score improved (11.551 then 3): ... baseline 3: 0.001 then 0
score improved (16.61 then 3): ... baseline 100: 0.001 then 0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s
```

The sweep found no verdict inversion in any of the 2015 pairs.

Mutants were run on a scratch copy of the tree, one at a time, against the unit's suite (`--test grading`) and then the matching case of mine:

| mutant | change | `grading.rs` (14 tests) | killer |
|---|---|---|---|
| M1 | `ladder.rs` `worst` tie-break `verdict == Verdict::NA` → `== Verdict::OK` | ok, 14 passed | FAILED |
| M2 | `threshold.rs:381` `order == Ordering::Less` → `order != Ordering::Greater` | ok, 14 passed | FAILED |
| M8a | `threshold.rs:477` `warn.multiplier && !better_than_baseline` → `false` | ok, 14 passed | FAILED |
| M8b | `threshold.rs:465` `crit.multiplier` → `false` | ok, 14 passed | FAILED |
| M10 | `threshold.rs:319` `!self.reaches(m, a)?` → `self.reaches(m, a)?` | ok, 14 passed | FAILED |

## 3. Suite run (after the cases existed)

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`: `EXIT=201`. Steps:

- spec validate: `thalamus v1 — 7 file(s), valid`
- drift: `generated/ and spec/suite.json match spec/`
- fmt `--check`: passed
- clippy `-D warnings`: passed
- test: stopped at the first failing binary:

```
     Running tests/adv_grading.rs (target/debug/deps/adv_grading-18a1bfc45b2f4769)
test adv_drop_to_zero_scores_no_better_than_a_drop_to_one ... FAILED
test adv_exact_event_count_at_min_events_crosses_the_line ... FAILED
test adv_reason_states_a_whole_event_count ... FAILED
test adv_value_better_than_a_negative_baseline_is_not_out_of_bounds ... FAILED
test adv_grading_is_monotone_in_the_value ... FAILED
test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The run before it in the same gate was `tests/grading.rs`: `test result: ok. 14 passed; 0 failed`. docs-check did not run.

Count run: `cargo test --locked --workspace --no-fail-fast`, `EXIT=101`. It has 53 test binaries and 53 result lines, and `adv_grading` is the only `FAILED` among them. 320 cases ran. The before figure, 311, is the same run minus the `adv_grading` binary's 9 cases. I got it by subtraction, not from a second run with my file deselected.

## 4. Findings (working tree above)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| F1 | crates/thalamus-core/src/rules/threshold.rs:283 | NEEDS-CHANGE | introduced | The event count of a ratio is value × n. For a ratio rounded down, that falls just under the true count: 10 of 30 gives `9.99999999999999999`, which fails `min_events 10`, so the result is OK where WARN is due (`adv_grading.rs:149`, red). | Every ratio adapter rounds at 18 digits, half away from zero: `sql.rs` `quotient`, `loki.rs` `over`, `git.rs` `quotient`. Prometheus renders an f64. Roughly half of all non-terminating k/n round down, and the design's own warn line declares `min_events: 10`. Fix: round value × n to the nearest whole event before comparing. It stays exact while n is under 10^17. |
| F2 | crates/thalamus-core/src/rules/threshold.rs:441 | NEEDS-CHANGE | introduced | For 15 of 556, the reason prints `14.9999999999999998 events of 556`. Design § 2.3 prints `15 failures of 556` (`adv_grading.rs:170`, red). | Every WARN and CRIT on a ratio. The F1 fix covers this too: print the rounded count. |
| F3 | crates/thalamus-core/src/rules/threshold.rs:491 | NEEDS-CHANGE | introduced | Under `lower_is_worse` with an x CRIT line, the score is log-linear while v > 0 and falls back to linear at v = 0. Over a baseline of 100, a drop to 1 scores 6.644 and a drop to 0 scores 3, so a total outage sorts below a near-outage (`adv_grading.rs:379` and `:282`, red). | D22's own example, a count dropping from 100 to 0. The score is what the views sort by (§ 3.3). Verdicts agree (both CRIT). Fix options: score lower-is-worse linearly past the CRIT line, or give v ≤ 0 on a log segment the segment's upper limit. |
| F4 | crates/thalamus-core/src/rules/threshold.rs:303 | INFEASIBLE | introduced | x × a negative baseline puts the line on the better side of the baseline. Under `higher_is_worse`, -11 against a baseline of -10 is CRIT with the reason `-11 >= 4 x -10` (`adv_grading.rs:191`, red). | Nothing found. No definition or adapter was shown to produce a negative gauge under a threshold rule. |
| F5 | crates/thalamus-core/tests/grading.rs:368 | CONFIRMED | introduced | 5 mutants pass all 14 `grading.rs` tests (M1, M2, M8a, M8b, M10). The log-linear score, the stricter part of a line, the `min_events` equality and `worst([OK, N/A]) = N/A` are not pinned by any of them. `score_and_verdict_agree` checks only the bands. | The four green cases in part 2 kill all five mutants. |
| F6 | crates/thalamus-core/src/rules/threshold.rs:298 | INFEASIBLE | introduced | Over a baseline of 0, an x-only line sits at 0, so a value equal to the baseline is CRIT with score 2. § 3.3 says the score is 0 at the baseline. The line rule and the score rule contradict each other here. | A pooled ratio baseline of 0 is common: no failures in either offset. Lines declaring x without abs are accepted, because only lines with neither x nor abs are refused (`threshold.rs:125`). The unit's own tests use them, and the design's own example always pairs x with abs. I found no shipped definition with x-only lines. This needs a design call. |
| F7 | crates/thalamus-core/src/rules/threshold.rs:432 | CONFIRMED | introduced | An OK reason under `lower_is_worse` reads `under the WARN line: 80 > 0.5 x 100`, when the value is above that line. | Every OK verdict of a `lower_is_worse` signal. |

## 5. Attacked and not broken

- Ladder: rank, exit and scored for all 5 verdicts; `worst` gives the same answer in every order of 125 triples. `pass_exit` returns 3 for an empty pass, an all-N/A pass and `[NA, UNKNOWN]`; `[NA, OK]` returns 0.
- Verdict monotonicity: no inversion in 2015 adjacent pairs across all three rules.
- Score against verdict at the exact lines: rounding then clamping keeps every score inside its verdict's band, `0.999` and `1.999` included.
- Partial readings for all three rules: a lower bound escalates only past CRIT; an upper bound escalates only under `lower_is_worse`.
- N/A ordering: n below min_n is N/A before the reading is checked, and a missing n is never N/A.
- A ratio with no n blocks only a line that declares `min_events`.
- Decimal overflow: unreachable at the adapters' 18-digit scale. An out-of-range input reads UNKNOWN, never OK.
- Rule ids: the three ids parse; anything else is a `Refusal::UnknownRule`.
- Incomplete lines are refused.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary/task-check.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary/suite-nofailfast.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary/copy/ (mutant copy with its own target/; `cargo clean` removed 202.0 MiB, then the directory was deleted)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary/mutate.sh (throwaway mutant runner, deleted)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-pass-1.md (this report)

My session lease `adv-m1-verdict-grading-p1` on the worktree is released when this report is returned.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 283, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the event count of a ratio is value x n on an adapter-rounded ratio, so exactly min_events events (10 of 30) count as 9.99999999999999999 and the line is not crossed: OK where WARN is due"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 441, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the reason states a fractional event count (14.9999999999999998 events of 556) where design 2.3 states 15 failures of 556"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 491, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "under lower_is_worse the log-linear score falls back to linear at value 0, so a drop to 0 scores 3 while a drop to 1 scores 6.644"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 303, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "x times a negative baseline puts the line on the better side, so -11 against a baseline of -10 is CRIT under higher_is_worse; no negative gauge shown to reach it"},
  {"file": "crates/thalamus-core/tests/grading.rs", "line": 368, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "five mutants (worst tie-break, min_events equality, both log-linear segments, stricter-part choice) pass all 14 grading tests; the adversary file's green cases kill them"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 298, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "over a zero baseline an x-only line sits at 0, so a value equal to the baseline is CRIT with score 2, contradicting design 3.3 (0 at the baseline); needs a design call"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 432, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the OK reason of lower_is_worse reads 'under the WARN line: 80 > 0.5 x 100' when the value is above the line"}
]
```
