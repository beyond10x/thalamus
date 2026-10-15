---
format: aep.planning-md/3
id: review-result:adversary-m1-verdict-grading-pass-2
kind: review-result
status: active
title: Adversary, story:m1-verdict-grading, pass 2
relations:
- reviews: story:m1-verdict-grading
revision: 1
---
unit: story:m1-verdict-grading, uncommitted working tree of impl/m1-verdict-grading (base ca445f1) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-verdict-grading
verdict: CONFIRMED (one red case, INFEASIBLE until the D23 reading below is settled)
cases: executed 321→328, red 1
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths kept (5 logs, this report); 3 deleted (mutant copy after `cargo clean`, mutant runner, pristine source copy)
needs-coordinator: yes. Under D23, when the WARN line is x-only and the CRIT line's abs is crossed over a baseline ≤ 0, is the result CRIT or UNKNOWN? The code says UNKNOWN; my case expects CRIT.

## 1. `git --no-pager diff --stat` (worktree, at report time)

```
 crates/thalamus-core/src/grading.rs         | 148 ++++++
 crates/thalamus-core/src/ladder.rs          |  68 +++
 crates/thalamus-core/src/rules/mod.rs       | 141 ++++++
 crates/thalamus-core/src/rules/threshold.rs | 697 ++++++++++++++++++++++++++++
 crates/thalamus-docs/src/status.rs          |   6 +-
 docs/design/thalamus.md                     |   2 +
 website/data/status.json                    |   4 +-
 website/docs/concepts/score.md              |  15 +-
 website/docs/concepts/verdicts.md           |  21 +-
 9 files changed, 1090 insertions(+), 12 deletions(-)
```

All of the tracked diff is the implementor's. I added one untracked file, `crates/thalamus-core/tests/adv_grading_p2.rs`, the only path I wrote in the tree. I formatted it with `rustfmt` on that file alone. The untracked `grading.rs` and `adv_grading.rs` are unchanged by me.

## 2. Cases added (`crates/thalamus-core/tests/adv_grading_p2.rs`)

| case | what it asserts | now |
|---|---|---|
| `adv_p2_a_crossed_abs_crit_line_decides_when_the_warn_line_is_x_only` (:153) | warn `{x 2}`, crit `{x 4, abs 20}`: 25 over a baseline of 0 or -5 is CRIT, and so is a partial lower bound of 25 over 0. Same for the ratio warn `{x 2, min_events 10}`, crit `{x 4, abs 0.05}` at 0.5 of 556 over 0 | **red** |
| `adv_p2_half_an_event_rounds_up` (:198) | 0.5 of 19 is 9.5 events, rounds to 10, WARN, reason `, 10 events of 19`. 0.45 of 21 is 9.45, rounds to 9, OK, reason `, 9 events of 21` | green; kills M1, M3 |
| `adv_p2_half_an_event_rounds_away_from_zero_below_zero` (:231) | -0.5 of 19 counts `-10 events of 19` | green; kills M2 |
| `adv_p2_higher_is_worse_stays_log_linear_past_the_crit_line` (:250) | ratio and `higher_is_worse`, x 2 / x 4 over 0.013: 0.104 scores `3` (linear would score 4) | green; kills M4 |
| `adv_p2_lower_is_worse_score_between_and_past_its_marks` (:269) | x 0.5 / x 0.25 over 100: 70.71 scores `0.5`, 35.36 scores `1.5`, 25 → `2`, 1 → `2.96`, 0 → `3`, -25 → `4`. Across the CRIT line the score moves by at most 0.002 (25.01 / 24.99, and 0.05199 / 0.05201 for higher) | green; kills M5, M6 |
| `adv_p2_x_and_abs_lines_over_a_zero_baseline_are_decided_by_abs` (:311) | design lines over 0 and -0.013. 0.03 of 556 gives WARN, reason `0.03 >= 0.02, 17 events of 556 >= min_events 10`. 0.03 of 300 gives OK with score `0.999`, `too few events…`. 0.06 of 100 gives CRIT `0.06 >= 0.05, 6 events of 100`. 0.01 gives OK `under the WARN line: 0.01 < 0.02`. Lower is worse, `{x 0.5, abs 5}` / `{x 0.25, abs 2}` over 0 and -4: 6 is OK, 3 is WARN, 1 is CRIT, and no reason cites ` x ` | green; kills M7, M8, M9 |
| `adv_p2_grading_is_monotone_within_a_baseline_around_zero` (:379) | 14 line sets × baselines {-100, -5, -0.013, 0, 1e-18, 0.013, 3} × n {300, 1000} × 30 values from -1000 to 1000: a worse value never grades better in verdict or score. x-only sets over a baseline ≤ 0 must be UNKNOWN and are not ordered | green |

Red output of the first case, run alone (`cargo test --locked -p thalamus-core --test adv_grading_p2 -- --exact adv_p2_a_crossed_abs_crit_line_decides_when_the_warn_line_is_x_only`). It is the first run of anything in this pass:

```
test adv_p2_a_crossed_abs_crit_line_decides_when_the_warn_line_is_x_only ... FAILED
a crossed, decidable CRIT line is CRIT:
count 25 over 0: Graded { verdict: UNKNOWN, score: None, reason: "baseline 0 is not above 0, so the x part of the WARN line cannot be decided and the line declares no abs", ... }
count 25 over -5: Graded { verdict: UNKNOWN, score: None, reason: "baseline -5 is not above 0, so the x part of the WARN line cannot be decided and the line declares no abs", ... }
partial lower bound 25 over 0: Graded { verdict: UNKNOWN, score: None, reason: "partial read", ... }
ratio 0.5 of 556 over 0: Graded { verdict: UNKNOWN, score: None, reason: "baseline 0 is not above 0, so the x part of the WARN line cannot be decided and the line declares no abs", ... }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

The other six cases, each run alone, gave `test result: ok. 1 passed` (`scratch/adversary-p2/cases-alone.log`).

Mutants were run on a scratch copy against `--test grading` (15 tests), `--test adv_grading` (pass 1, 9 tests) and `--test adv_grading_p2`:

| mutant (`threshold.rs`) | grading.rs | adv_grading (p1) | killed by p2 |
|---|---|---|---|
| M1 `:667` `>=` → `>` (half rounds toward zero) | ok 15 | ok 9 | half_an_event_rounds_up, …below_zero |
| M2 `:668` `+ signum()` → `+ 1` | ok 15 | ok 9 | …below_zero |
| M3 `:290` drop `Num::rounded` | ok 15 | FAILED 2 | half_an_event…, x_and_abs… |
| M4 `:480` past CRIT linear for both rules | ok 15 | ok 9 | higher_is_worse_stays_log_linear… |
| M5 `:478` `past_crit` inverted | ok 15 | FAILED 2 | lower_is_worse_score…, monotone… |
| M6 `:480` log past CRIT for lower too | ok 15 | FAILED 2 | lower_is_worse_score…, monotone… |
| M7 `:512` keep `x` when `abs` is declared, baseline ≤ 0 | ok 15 | ok 9 | x_and_abs_lines… |
| M8 `:512` `> 0` → `!= 0` | FAILED 1 | FAILED 1 | x_and_abs…, monotone… |
| M9 `:512` `> 0` → `>= 0` | FAILED 1 | FAILED 1 | x_and_abs…, monotone… |

## 3. Suite run (after the cases existed)

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess` finished with `EXIT=201`. Step by step:

- spec validate: `thalamus v1 — 7 file(s), valid`
- drift: `generated/ and spec/suite.json match spec/`
- fmt `--check`: passed
- clippy `-D warnings`: passed
- test: stopped at the first failing binary:

```
     Running tests/adv_grading_p2.rs (target/debug/deps/adv_grading_p2-f4a077a0a8bcd941)
test adv_p2_a_crossed_abs_crit_line_decides_when_the_warn_line_is_x_only ... FAILED
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

The binary before it was `tests/adv_grading.rs`: `test result: ok. 9 passed; 0 failed`. docs-check did not run. An earlier gate run, made after the case existed, failed at fmt `--check` on my unformatted file and ran no tests (`task-check-run1-fmt.log`). I then formatted the file and ran the gate above.

Count run: `cargo test --locked --workspace --no-fail-fast` gave `EXIT=101`. It printed 54 `test result` lines, and `adv_grading_p2` is the only `FAILED` one. 328 cases ran (327 passed, 1 failed), and `tests/grading.rs` gave `ok. 15 passed`. The before figure, 321, is 328 minus the 7 cases of `adv_grading_p2`. I got it by subtraction, not from a second run.

## 4. Findings (working tree above)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| G1 | crates/thalamus-core/src/rules/threshold.rs:245 | INFEASIBLE | introduced | `try_decide` calls `usable(&self.warn, …)?` before it reads the CRIT line. An x-only WARN line over a baseline ≤ 0 therefore returns UNKNOWN even when the CRIT line is decidable by its `abs` and crossed (25 against abs 20; 0.5 against abs 0.05). The partial lower bound of 25 also reads `partial read`, where § 3.1 rule 4 says a lower bound already over the CRIT line is CRIT. This contradicts the function's own rule at :249: "a crossed CRIT line decides, whatever the WARN line could not count" (`adv_grading_p2.rs:153`, red). UNKNOWN exits 3 and ranks below WARN, so the aggregate understates a CRIT. | The parser admits this line shape (only lines with neither x nor abs are refused, :126). The unit's own test builds it (`grading.rs:473`, warn `{x 2}`, crit `{x 4, abs 20}`, value 0). A zero baseline is the common case for a failure ratio. The repository has no shipped definitions, so I found nothing that reaches it. Fix: settle the CRIT line first (its `usable`, then `short`) and return CRIT when it is crossed; require the WARN line only after that. |
| G2 | crates/thalamus-core/tests/grading.rs:368 | CONFIRMED | introduced | 7 of 9 mutants of the new code pass all 15 `grading.rs` tests (M1–M7). Four of them (half-away-from-zero rounding, its sign, log-linear past the CRIT line for higher is worse, D23's abs-only decision) also pass every pass-1 case. Only the p2 cases kill them. | The p2 cases in part 2 kill all nine. |
| G3 | crates/thalamus-core/src/rules/threshold.rs:394 | CONFIRMED | introduced | The doc comments at :394 and :446 still give `16.124 events of 556` as the reason form, which rounding (D22) no longer produces. The `verdicts.md:50` example reason, `0.029 >= 2 x 0.013 and >= 0.02, 15 failures of 556`, is not what the code writes for that input. It writes `…, 16 events of 556 >= min_events 10` (0.029 × 556 = 16.12; design § 2.3 carries the same inconsistent example). | Anyone reading the docs or the comments for the reason format. Nothing parses the reason. |

## 5. Attacked and not broken

- Rounding at exactly .5 is half away from zero for both signs, and the rounded count is used both against `min_events` and in the reason (p2 cases, green).
- A fractional n cannot be built, because `Denominator.n` is an i64. A negative n is N/A under any `min_n` ≥ 0. I did not build a definition without a denominator.
- n past 10^17: by arithmetic, an 18-digit ratio times n ≥ 10^18 can miss the true count by ≥ 0.5. No population that size was found, and I wrote no case.
- Rounding a partial lower bound gives at most the ceiling of the bound's count, so it never over-escalates. I checked this by reading, not with a case.
- The score is continuous at the CRIT line in both directions. Past CRIT it is linear for lower is worse (D22's values 2.96 / 3) and stays log-linear for higher is worse.
- Within one baseline from -100 to 3, including 0 and 1e-18, with values from -1000 to 1000: no verdict or score inversion across all three rules.
- Lines with both `abs` and `x` over 0 or below, with and without `min_events`, are decided by `abs` and the count alone. No reason cites the dropped `x` part.
- Reason text, branch by branch: crossed, under the line (higher), above the line (lower, F7), too few events, D23 UNKNOWN (names the baseline), no population, out of range, and the partial prefix. Every one is non-empty and none contradicts § 2.3 beyond G3.
- `score.md` (band clamps, 0.999 under an unmet `min_events`, linear on the better side and past the CRIT line of a drop) and `verdicts.md` (the order of rules 1–5, the D23 paragraph, the rounding sentence) match the code.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/cases-alone.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/task-check-run1-fmt.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/task-check.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/suite-nofailfast.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/mutants.log (kept)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/copy/ (mutant copy with its own target/; `cargo clean` removed 210.8 MiB, then the directory was deleted)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-p2/mutate.sh and threshold.rs.orig (throwaway mutant runner and pristine source, deleted)
- ~/.cache/thalamus-waves/20261015d/m1-verdict-grading/scratch/adversary-pass-2.md (this report)
- The harness kept background-task output files under ~/.cache/claude-tmp/ (owned by the harness)

My session lease `adv-m1-verdict-grading-p2` was acquired on the worktree and has been released.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 245, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "an x-only WARN line over a baseline at or below 0 is refused before the CRIT line is read, so a CRIT line decidable by abs and crossed (25 against abs 20, also as a partial lower bound) grades UNKNOWN instead of CRIT"},
  {"file": "crates/thalamus-core/tests/grading.rs", "line": 368, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "7 of 9 mutants of the pass-2 changes (rounding, past-CRIT score, D23 abs-only decision) pass all 15 grading tests; four also pass every pass-1 case and are killed only by adv_grading_p2.rs"},
  {"file": "crates/thalamus-core/src/rules/threshold.rs", "line": 394, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "doc comments still show '16.124 events of 556' and verdicts.md:50 shows '15 failures of 556' for 0.029, while the code writes '16 events of 556 >= min_events 10'"}
]
```
