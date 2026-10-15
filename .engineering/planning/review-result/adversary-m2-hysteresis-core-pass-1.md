---
format: aep.planning-md/3
id: review-result:adversary-m2-hysteresis-core-pass-1
kind: review-result
status: active
title: Adversary, story:m2-hysteresis-core, pass 1
relations:
- reviews: story:m2-hysteresis-core
revision: 1
---
unit: story:m2-hysteresis-core, working tree of ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m2-hysteresis-core (base 6b98444, uncommitted)
verdict: NEEDS-CHANGE
cases: executed 431→438, red 3
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: whether re-grade dedup and ts-order guarding belong in thalamus-core or in story:m2-transitions' caller

## 1. Diff stat

```
 crates/thalamus-core/src/hysteresis.rs | 171 +++++++++++++++++++++++++++++++++
 crates/thalamus-docs/src/status.rs     |  14 +--
 website/data/status.json               |  14 +--
 website/docs/concepts/verdicts.md      |   5 +-
 4 files changed, 189 insertions(+), 15 deletions(-)
?? crates/thalamus-core/tests/adv_hysteresis_p1.rs   (mine, untracked)
?? crates/thalamus-core/tests/hysteresis.rs           (implementor's, untracked)
```

I did not touch the 4 tracked paths. They are the implementor's and coordinator's changes. My only path is the untracked test file `crates/thalamus-core/tests/adv_hysteresis_p1.rs`. No non-test path is mine.

## 2. Cases added (`crates/thalamus-core/tests/adv_hysteresis_p1.rs`)

| test | asserts | now |
|---|---|---|
| `one_sample_graded_at_two_revisions_does_not_settle_alone` :56 | one sample graded at rev 1 and rev 2 (D5 re-grade) does not settle WARN with k = 2 | RED |
| `the_same_evaluation_observed_twice_counts_once` :70 | the same evaluation id observed twice does not count twice | RED |
| `a_settled_change_always_satisfies_the_transition_invariant` :85 | a returned change holds `first_ts <= settled_ts` (D29) for input out of `ts` order | RED |
| `a_persistence_of_zero_settles_on_one_evaluation` :101 | `Persistence {0,0,0}` settles every verdict on one evaluation | green |
| `persistence_of_reads_values_below_one_as_one` :124 | `Persistence::of` maps `{0, -1, i64::MIN}` to `{1,1,1}`. This kills mutant M2, which the unit's 11 tests miss | green |
| `crit_clears_after_m_ok_and_steps_down_after_k_warn` :147 | with k 2 / m 4, CRIT clears on the 4th OK, and CRIT→WARN takes 2 WARN | green |
| `settled_changes_match_the_rules_on_seeded_series` :221 | 2000 seeded series (LCG seed 20261015) agree with an oracle written from rules 1–4: from≠to, first_ts≤settled_ts, exactly n evaluations ending at the settling one | green |

Red output of this file run alone (`cargo test --locked -p thalamus-core --test adv_hysteresis_p1`), captured verbatim before any suite run:

```
running 7 tests
test a_persistence_of_zero_settles_on_one_evaluation ... ok
test a_settled_change_always_satisfies_the_transition_invariant ... FAILED
test crit_clears_after_m_ok_and_steps_down_after_k_warn ... ok
test one_sample_graded_at_two_revisions_does_not_settle_alone ... FAILED
test the_same_evaluation_observed_twice_counts_once ... FAILED
test persistence_of_reads_values_below_one_as_one ... ok
test settled_changes_match_the_rules_on_seeded_series ... ok

---- a_settled_change_always_satisfies_the_transition_invariant stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p1.rs:89:9:
first_ts Timestamp("2026-01-01T00:05:00Z") after settled_ts Timestamp("2026-01-01T00:04:00Z"): RecordTransition refuses this

---- one_sample_graded_at_two_revisions_does_not_settle_alone stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p1.rs:59:5:
assertion `left == right` failed: one sample, re-graded, settled a change with k = 2
  left: [SettledChange { from: None, to: WARN, first_ts: Timestamp("2026-01-01T00:01:00Z"), settled_ts: Timestamp("2026-01-01T00:01:00Z"), settling_score: Some(Decimal("0.5")), definition_rev: 2, evaluations: [EvaluationId("evl_1_r1"), EvaluationId("evl_1_r2")] }]
 right: []

---- the_same_evaluation_observed_twice_counts_once stdout ----
panicked at crates/thalamus-core/tests/adv_hysteresis_p1.rs:75:5:
assertion `left == right` failed: the same evaluation id counted twice toward k = 2
  left: Some(SettledChange { from: None, to: WARN, first_ts: Timestamp("2026-01-01T00:01:00Z"), settled_ts: Timestamp("2026-01-01T00:01:00Z"), settling_score: Some(Decimal("0.5")), definition_rev: 1, evaluations: [EvaluationId("evl_1_r1"), EvaluationId("evl_1_r1")] })
 right: None

test result: FAILED. 4 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

Mutants, each run in a scratch copy of the workspace (removed afterwards with `cargo clean --manifest-path`) against `--test hysteresis --test adv_hysteresis_p1`:

| mutant | unit's 11 tests | mine |
|---|---|---|
| M1 `run_for`: `.max(1)` removed (hysteresis.rs:67) | 11 pass | survives. Equivalent: the first member of a run is never trimmed (:135), so needed 0 behaves as 1 |
| M2 `Persistence::of`: `map_or(1, \|v\| v.max(1))` → `map_or(0, \|v\| v)` (hysteresis.rs:48) | 11 pass (`test result: ok. 11 passed`) | killed by `persistence_of_reads_values_below_one_as_one` |

## 3. Suite run (after the cases existed)

`task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess` exited 201:

| step | result |
|---|---|
| ess specify validate | ok |
| drift | `generated/ and spec/suite.json match spec/` |
| cargo fmt --check | ok |
| cargo clippy -D warnings | ok |
| cargo test --workspace | `test result: FAILED. 4 passed; 3 failed` in adv_hysteresis_p1, then `task: Failed to run task "check": task: Failed to run task "test": exit status 101` |
| docs-check | not reached |

The count comes from `cargo test --locked --workspace --no-fail-fast`, exit 101: 435 passed, 3 failed, 438 in total. The before count is 431, from the implementor's `scratch/task-check.log`.

## 4. Findings (all cover the working tree above; the base `hysteresis.rs` is empty, so every origin is `introduced`)

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | crates/thalamus-core/src/hysteresis.rs:128 | adv_hysteresis_p1.rs:59 is red. One sample graded at two revisions settles WARN with k = 2, and first_ts equals settled_ts | Documented workflow: § 5.1 says the core "re-grades history after a threshold change", and D5 says a re-grade is a new evaluation of the same sample. story:m2-transitions seeds by replaying the stored evaluations. No re-grade code exists yet. Fix: a run counts distinct samples and keeps the highest `definition_rev`, as D14 does, or the doc comment states that the caller passes one evaluation per sample | NEEDS-CHANGE | introduced |
| F2 | crates/thalamus-core/src/hysteresis.rs:128 | adv_hysteresis_p1.rs:75 is red. The same evaluation id is counted twice, and the change references `[evl_1_r1, evl_1_r1]` | Nothing found. It needs a replay-then-live overlap in story:m2-transitions, which is not written yet. The F1 dedup fixes it | INFEASIBLE | introduced |
| F3 | crates/thalamus-core/src/hysteresis.rs:146 | adv_hysteresis_p1.rs:89 is red. Input out of order gives first_ts 00:05 > settled_ts 00:04, which breaks the `Transition` invariant (spec/domains/transition.yaml:53) | `SeriesVerdicts` returns rows "in the store's order" (behaviour/evaluation.rs:103-105). A re-grade appends older ts. Nothing calls `observe` yet. Fix: `observe` ignores or refuses an evaluation whose ts is earlier than the run's last member | CONFIRMED | introduced |
| F4 | crates/thalamus-core/src/hysteresis.rs:48 | mutant M2 survives all 11 unit tests, and a green case now kills it | The loader refuses values below 1 (D29 `persistence-below-one`). Only a hand-built `SignalDefinitionData` reaches it | INFEASIBLE | introduced |
| J1 | crates/thalamus-core/src/hysteresis.rs:110 | judgement: `Persistence` is fixed when a `SeriesHysteresis` is built. A revision that changes `persistence` either keeps the stale value or drops the settled verdict. A replay on restart applies today's values to history graded under older revisions, so replay ≠ live. A run spanning revisions reports only the rev that settled it | story:m2-transitions `hysteresis_state_survives_restart`, and `ReviseDefinition` | CONFIRMED | introduced |
| J2 | crates/thalamus-core/src/hysteresis.rs:110 | judgement: no constructor resumes from a recorded settled verdict. D13 lets transitions outlive raw retention, so a replay of the retained evaluations starts at settled None. On restart it emits a second change with no `from` | story:m2-transitions seeding after retention. No caller yet | CONFIRMED | introduced |

## 5. Attacked and not broken

- k/k_crit/m semantics: 2000 seeded series agree with an independent oracle of rules 1–4.
- CRIT clears only after m OK (not k), and CRIT→WARN steps down after k WARN.
- UNKNOWN and N/A never clear a settled WARN or CRIT, including long runs.
- `from != to` held on every change. On input in order, `first_ts <= settled_ts` held, as did exactly n evaluations ending at the settling one and the `settled_ts`, `settling_score` and `definition_rev` of the settling evaluation.
- Values below 1 behave as 1, both through `run_for` and through `of`.
- Replay equals live for input in order. It is the same code path: the restart-split loop in `replay_equals_live` is self-referential and could not fail for a defect in `observe`. Its literal `.to` list is what gives it force.
- The docs patch (verdicts.md, status.rs/status.json) agrees with the behaviour.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/adversary/m1.log
- ~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/adversary/m2.log
- ~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/adversary/task-check.log
- ~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/adversary/suite-nff.log
- ~/.cache/thalamus-waves/20261015e/m2-hysteresis-core/scratch/adversary-pass-1.md (this report)
- The mutant workspace copy `scratch/adversary/mutant` and its `target/` are removed (`cargo clean --manifest-path`, then `rm -rf`).

## 7. Findings block

```findings
- file: crates/thalamus-core/src/hysteresis.rs
  line: 128
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a sample re-graded at a second definition revision counts twice toward k, so one reading settles a WARN with first_ts equal to settled_ts
- file: crates/thalamus-core/src/hysteresis.rs
  line: 128
  category: concurrency
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the same evaluation observed twice counts twice and the settled change references one evaluation id twice
- file: crates/thalamus-core/src/hysteresis.rs
  line: 146
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: input out of ts order yields a settled change with first_ts after settled_ts, which RecordTransition's invariant refuses
- file: crates/thalamus-core/src/hysteresis.rs
  line: 48
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: removing the below-1 clamp in Persistence::of survives all 11 unit tests; adv_hysteresis_p1 now kills it
- file: crates/thalamus-core/src/hysteresis.rs
  line: 110
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: persistence is fixed per SeriesHysteresis, so a revision changing persistence cannot apply without losing the settled verdict, and a restart replay applies today's values to older revisions
- file: crates/thalamus-core/src/hysteresis.rs
  line: 110
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no way to resume from a recorded settled verdict, so a replay after raw retention re-emits a change with no from
```
