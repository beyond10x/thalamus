---
format: aep.planning-md/3
id: review-result:adversary-m2-hysteresis-core-pass-2
kind: review-result
status: active
title: Adversary, story:m2-hysteresis-core, pass 2
relations:
- reviews: story:m2-hysteresis-core
revision: 1
---
unit: story:m2-hysteresis-core, working tree of ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m2-hysteresis-core (base 6b98444, uncommitted, after decisions 1-5)
verdict: NEEDS-CHANGE
cases: executed 446→454, red 6
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 18 paths (part 6)
needs-coordinator: whether a sample recorded late counts (decision 3 makes live and replay disagree, F2)

## 1. Diff stat

```
 crates/thalamus-core/src/hysteresis.rs | 268 +++++++++++++++++++++++++++++++++
 crates/thalamus-docs/src/status.rs     |  14 +-
 website/data/status.json               |  14 +-
 website/docs/concepts/verdicts.md      |   6 +-
 4 files changed, 287 insertions(+), 15 deletions(-)
?? crates/thalamus-core/tests/adv_hysteresis_p1.rs   (pass 1, unchanged)
?? crates/thalamus-core/tests/adv_hysteresis_p2.rs   (mine, new)
?? crates/thalamus-core/tests/hysteresis.rs           (implementor's)
```

I did not touch the 4 tracked paths. My only path is `crates/thalamus-core/tests/adv_hysteresis_p2.rs`. No non-test path is mine.

## 2. Cases added (`crates/thalamus-core/tests/adv_hysteresis_p2.rs`)

| test | asserts | now |
|---|---|---|
| `a_regraded_history_settles_what_its_highest_revisions_say` :82 | history re-graded after a revision (s1 r1 WARN, s1 r2 WARN, s2 r1 OK, s2 r2 WARN), replayed in minting order, settles WARN over `[evl_1_r2, evl_2_r2]` | RED |
| `a_regrade_that_restores_the_runs_verdict_keeps_the_run` :102 | live: s1 WARN, s2 r1 OK, s2 r2 WARN settles WARN | RED |
| `a_late_sample_counts_the_same_live_and_in_replay` :121 | live and `replay` of s1 WARN, s3 WARN, s2 OK (late) agree | RED |
| `a_stale_revision_is_ignored_after_its_sample_left_the_run` :137 | at one ts: s1 r2 OK, s2 WARN, s1 r1 WARN; the stale r1 settles nothing | RED |
| `an_evaluation_seen_earlier_at_the_same_ts_is_ignored` :152 | at one ts: s1 WARN, s2 OK, s1 WARN again, then s3 WARN settles nothing. Kills M3 and M5 | green |
| `a_change_holds_first_ts_before_settled_ts_for_any_timestamp_text` :170 | D29 `first_ts <= settled_ts` when a ts is not an instant | RED |
| `resume_then_the_same_verdict_settles_nothing` :196 | `resume(V)` then 8 × V emits nothing, for all 5 verdicts | green |
| `settled_changes_hold_the_transition_invariants_on_mixed_series` :232 | 3000 seeded series (LCG 202610152), with shared ts, late samples, re-grades at higher and lower revs, duplicates and `set_persistence`: 12 invariants (D29, D13, rule 1, decision 1) | RED |

Red output of this file run alone (`cargo test --locked -p thalamus-core --test adv_hysteresis_p2`, EXIT=101), captured before any suite run:

```
---- a_regrade_that_restores_the_runs_verdict_keeps_the_run stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:111:5:
  left: []
 right: [(None, WARN, ["evl_1_r1", "evl_2_r2"])]
---- a_change_holds_first_ts_before_settled_ts_for_any_timestamp_text stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:188:9:
first_ts Timestamp("2026-01-05T10:00:01+02:00") after settled_ts Timestamp("2026-01-05T09:x")
---- a_late_sample_counts_the_same_live_and_in_replay stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:129:5:
assertion `left == right` failed: live and replay of the same evaluations disagree
  left: [(None, WARN, ["evl_1_r1", "evl_3_r1"])]
 right: []
---- a_stale_revision_is_ignored_after_its_sample_left_the_run stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:145:5:
assertion `left == right` failed: revision 1 of sample 1 counted after revision 2 was taken
  left: Some(SettledChange { from: None, to: WARN, first_ts: Timestamp("2026-01-01T00:00:01Z"), settled_ts: Timestamp("2026-01-01T00:00:01Z"), settling_score: Some(Decimal("0.5")), definition_rev: 1, evaluations: [EvaluationId("evl_2_r1"), EvaluationId("evl_1_r1")] })
 right: None
---- a_regraded_history_settles_what_its_highest_revisions_say stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:92:5:
assertion `left == right` failed: samples 1 and 2 are both WARN at revision 2
  left: []
 right: [(None, WARN, ["evl_1_r2", "evl_2_r2"])]
---- settled_changes_hold_the_transition_invariants_on_mixed_series stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p2.rs:389:5:
2 invariants violated over 5955 changes:
- decision 1 a stale revision settles nothing: 2x, first case 309: SettledChange { from: Some(NA), to: WARN, ..., definition_rev: 4, evaluations: [EvaluationId("evl_1_r4")] }
  after ["evl_1_r1 2026-01-01T00:00:02Z r1 OK", "evl_1_r5 2026-01-01T00:00:02Z r5 UNKNOWN", "evl_2_r1 2026-01-01T00:00:02Z r1 NA", "evl_1_r4 2026-01-01T00:00:02Z r4 WARN"]
- decision 1 no listed evaluation is a stale revision: 2x, first case 309: (same)
test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```

Mutants, in a scratch copy, run against `--test hysteresis --test adv_hysteresis_p1 --test adv_hysteresis_p2`:

| mutant | the 26 existing tests | mine |
|---|---|---|
| M3 remove the same-ts id check (hysteresis.rs:176) | 26 pass | killed by `an_evaluation_seen_earlier_at_the_same_ts_is_ignored` |
| M5 keep only the last id at one ts (hysteresis.rs:206, `&& false`) | 26 pass | killed by the same case |
| M4 `<= rev` → `< rev` (hysteresis.rs:193) | 26 pass | survives. Equivalent under D5: one evaluation per (sample, rev) |
| M6 `settled_ts` from the last listed member (hysteresis.rs:241) | 26 pass | survives. Equivalent: the settling evaluation was the last listed in all 5955 changes |
| M7 drop `.max(1)` from `keep` (hysteresis.rs:228) | 26 pass | survives. Equivalent: the values are already ≥ 1 |
| M8 drop `blocked` (hysteresis.rs:222) | killed (4 unit + 1 p1) | killed |

## 3. Suite run (after the cases existed)

`task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess`, EXIT=201: validate ok, drift `generated/ and spec/suite.json match spec/`, fmt ok, clippy ok, then

```
     Running tests/adv_hysteresis_p2.rs (target/debug/deps/adv_hysteresis_p2-aba64ebb0f1d9232)
test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

docs-check was not reached. Count from `cargo test --locked --workspace --no-fail-fast` (EXIT=101): 448 passed, 6 failed, 454 in total. The before count, 446, comes from the implementor's `scratch/c1-task-check.log`.

## 4. Findings (working tree above; base `hysteresis.rs` is empty, so every origin is `introduced`)

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | crates/thalamus-core/src/hysteresis.rs:198 | p2.rs:92 and :111 are red. When a re-grade changes the verdict, the code starts a new run and forgets the samples before it. A later re-grade back to the run's verdict does not restore them. A history re-graded to WARN, WARN settles nothing. This contradicts decision 1, D14 and verdicts.md:92-93 ("consecutive samples ... counts once, with its newest evaluation") | The `Grader` runs "again over history after a definition revision" (evaluation.yaml, actor summary). The memory store lists rows in id order (`memory.rs:77`), and ids are minted in sequence (`observation.rs:297`), so a replay reads each ts tie in the order graded. Nothing calls `replay` or `observe` yet. Fix: keep the last max(k, k_crit, m) samples at their highest revision, whatever their verdict, and take the run as the longest suffix with one verdict | NEEDS-CHANGE | introduced |
| F2 | crates/thalamus-core/src/hysteresis.rs:175 | p2.rs:129 is red. Live drops the late s2 OK and settles WARN over s1, s3; `replay` sorts first and settles nothing. So "None, state unchanged" loses an evaluation that `replay` counts | `RecordSample` sets `ts: input.ts` with no ordering outcome (observation.yaml:249-271). A restart that replays, and the flap measurement, then disagree with what live recorded. Nothing calls `observe` yet. Fix, one of: `RecordSample` refuses a ts older than the series' newest, or `replay` reads in minting order without sorting (live drops the same rows then) | CONFIRMED | introduced |
| F3 | crates/thalamus-core/src/hysteresis.rs:193 | p2.rs:145 and the property (case 309, 2 of 5955 changes) are red. The revision check reads only the current run, so a lower revision of a sample that has left the run is counted again and settles | Two samples at one ts are admitted (`observation.rs:82`). It needs revisions handed in out of order, and minting order prevents that in the memory store. No path found | INFEASIBLE | introduced |
| F4 | crates/thalamus-core/src/hysteresis.rs:174 | p2.rs:188 is red. The texts `…T10:00:01+02:00`, `…T09:00:03Z` and `…T09:x` each pass the check against the previous one, yet first_ts is after settled_ts in `timestamp_order`. D29 refuses that | The decoder admits any text as a Timestamp (`observation.rs:112`). The sources emit RFC 3339. No path found | INFEASIBLE | introduced |
| F5 | crates/thalamus-core/src/hysteresis.rs:176 | mutants M3 and M5 (also :206) survive all 26 existing tests. `an_evaluation_seen_earlier_at_the_same_ts_is_ignored` now kills both | Two samples at one ts, plus an evaluation seen twice (replay then live, story:m2-transitions) | CONFIRMED | introduced |
| J1 | crates/thalamus-core/src/hysteresis.rs:160 | judgement. After `set_persistence` shrinks n, a run already long enough settles only if the next evaluation continues it. If that evaluation breaks the run, or is ignored, the run never settles. Decision 3 says "settles on the next evaluation". Fix: `set_persistence` returns `Option<SettledChange>` for the run as it stands | `ReviseDefinition` changing `persistence`. No caller yet | CONFIRMED | introduced |
| J2 | crates/thalamus-core/src/hysteresis.rs:148 | judgement. `resume` takes no ts, and its doc does not say that the caller must feed only evaluations after the resumed transition's settled_ts. Feeding retained history from before that point re-emits older changes | story:m2-transitions seeding on restart. No caller yet | CONFIRMED | introduced |

## 5. Attacked and not broken

- Over 5955 changes on 3000 mixed series, with valid instants, these held: D29 (from ≠ to, first_ts ≤ settled_ts), one evaluation per sample, every listed verdict = to, oldest first, the settling evaluation listed and copied, and exactly n under the current persistence (including after `set_persistence` mid-run, raising and lowering).
- `resume(V)` followed by V, for all 5 verdicts.
- Equal ts with `in_time_order`: ties keep minting order. Nothing broke beyond F1 and F3.
- `replay_equals_live` now compares against a literal list, so it can fail.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/`:

- `adversary-p2/red.log`, `task-check.log`, `suite-nff.log`, `mutants.sh`, `clean.log`
- `adversary-p2/m3..m8.log` and `m3..m8.diff` (12 files)
- `adversary-pass-2.md` (this report)
- The mutant copy `adversary-p2/mutant` was cleaned with `cargo clean --manifest-path` (243.4 MiB) and then removed.

## 7. Findings block

```findings
- file: crates/thalamus-core/src/hysteresis.rs
  line: 198
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a re-grade that changes the verdict forgets the run before it, so a history whose newest revisions read WARN, WARN never settles WARN, contrary to decision 1 and verdicts.md
- file: crates/thalamus-core/src/hysteresis.rs
  line: 175
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a sample recorded late is dropped live but counted by replay, so live settles a WARN that replay of the same evaluations does not
- file: crates/thalamus-core/src/hysteresis.rs
  line: 193
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: with several samples at one ts, a lower revision of a sample that left the run is counted again and settles a change its higher revision contradicts
- file: crates/thalamus-core/src/hysteresis.rs
  line: 174
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: with a ts that is not an instant, the check against the last evaluation alone lets a change have first_ts after settled_ts
- file: crates/thalamus-core/src/hysteresis.rs
  line: 176
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: removing the same-ts id check, or keeping only the last id at a ts, survives all 26 existing tests; a pass-2 case now kills both
- file: crates/thalamus-core/src/hysteresis.rs
  line: 160
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: after set_persistence shrinks n, a run already long enough never settles if the next evaluation breaks it or is ignored
- file: crates/thalamus-core/src/hysteresis.rs
  line: 148
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: resume does not say that the caller must feed only evaluations after the resumed settled_ts; feeding older retained history re-emits changes
```
