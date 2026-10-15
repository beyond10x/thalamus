---
format: aep.planning-md/3
id: review-result:adversary-m0-conformance-records-pass-1
kind: review-result
status: active
title: Adversary, story:m0-conformance-records, pass 1
relations:
- reviews: story:m0-conformance-records
revision: 1
---
unit: story:m0-conformance-records, uncommitted working tree on impl/m0-conformance-records (base 5d0ee31), ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m0-conformance-records
verdict: red. Strongest is CONFIRMED: 7 behaviour mutants pass the whole suite, and 1 red property case (INFEASIBLE today)
cases: executed 133→139, red 1
origin: introduced 8 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 scratch directory plus this report (part 6)
needs-coordinator: yes. I reversed status-shipped.patch after the gate ran, so re-apply it. Decide whether the 6 cases stay in the unit (the 5 green ones guard lines the suite misses)

## 1. git --no-pager diff --stat (final state)

```
 crates/thalamus-core/src/behaviour/assertion.rs   |  67 ++
 crates/thalamus-core/src/behaviour/evaluation.rs  | 138 ++++
 crates/thalamus-core/src/behaviour/observation.rs | 141 ++++
 crates/thalamus-core/src/conformance.rs           | 837 ++++++++++++++++++----
 crates/thalamus-core/tests/conformance.rs         | 582 ++++++++++++++-
 5 files changed, 1613 insertions(+), 152 deletions(-)
?? crates/thalamus-core/tests/adversary_records.rs   (326 lines, mine, untracked)
```

- This stat is the same as the one at the start: those 5 files are the implementor's, and I did not edit them.
- `cmp` against the pristine copies I took before the mutant run shows `observation.rs` and `evaluation.rs` unchanged.
- The only path I added is a test file.
- The coordinator patch (`crates/thalamus-docs/src/status.rs`, `website/data/status.json`) was applied for the gate and then reversed with `git apply -R`. The tree is left without it.

## 2. Cases added: `crates/thalamus-core/tests/adversary_records.rs`

| case | asserts | now | red against |
|---|---|---|---|
| `series_samples_serves_only_the_named_series` | SeriesSamples(`series-a`) returns exactly the sample on `series-a`, not the one on `series-ab` | green | M10 |
| `series_verdicts_serves_only_the_named_series` | SeriesVerdicts(`series-a`) returns exactly that series' evaluation | green | M9 |
| `series_samples_orders_by_instant_not_by_arrival_or_text` | samples recorded at 09:00:03Z, then 10:00:01+02:00, then 01-04 come back in the order 01-04, +02:00, 09:00:03Z | green | M1, M2 |
| `muted_and_retired_definitions_refuse_a_sample` | RecordSample on a Muted signal and on a Retired signal answers `NotActive` | green | M5 |
| `crit_carries_a_score_and_unknown_refuses_one` | CRIT with a score is `Recorded`; UNKNOWN with a score is `ScoreNotAdmitted` | green | M3, M4 |
| `an_issued_sample_id_is_never_issued_again` | after the newest sample is deleted through the storage port, the next sample does not get its id | **red** | the tree as handed |

Red output from the first run of the case file alone (`cargo test --locked -p thalamus-core --test adversary_records`, exit 101, before rustfmt moved the assert to line 322):

```
---- an_issued_sample_id_is_never_issued_again stdout ----

thread 'an_issued_sample_id_is_never_issued_again' (2227602) panicked at crates/thalamus-core/tests/adversary_records.rs:289:5:
assertion `left != right` failed: a deleted sample's id was issued to a new sample
  left: SampleId("smp_2")
 right: SampleId("smp_2")
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Mutant run.** The scratch copy is `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary/mutant-tree`; the worktree was never mutated. Command: `cargo test --locked --no-fail-fast -p thalamus-core --test conformance --test adversary_records`. Each mutant's diff is in `scratch/adversary/logs/<M>.diff`.

| mutant | change | conformance binary (suite, `every_obligation_is_met`, harness tests) | my cases |
|---|---|---|---|
| M1 | `observation.rs:110` sort removed | `test result: ok. 6 passed; 0 failed` | ordering case FAILED |
| M2 | `observation.rs:110` sort by `ts` text | `ok. 6 passed; 0 failed` | ordering case FAILED |
| M3 | `evaluation.rs:136` CRIT not scored | `ok. 6 passed; 0 failed` | crit/unknown case FAILED |
| M4 | `evaluation.rs:137` UNKNOWN scored | `ok. 6 passed; 0 failed` | crit/unknown case FAILED |
| M5 | `observation.rs:43` `!= Active` changed to `== Proposed` | `ok. 6 passed; 0 failed` | muted/retired case FAILED |
| M9 | `evaluation.rs:119` series filter changed to `true` | `ok. 6 passed; 0 failed` | verdicts-filter case FAILED |
| M10 | `observation.rs:100` series filter changed to `true` | `ok. 6 passed; 0 failed` | samples-filter case FAILED |

## 3. Gate run, after the cases existed

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, with status-shipped.patch applied, exited **201**:

```
task: [check] ~/.cache/ess/toolchains/0.60.0/ess specify validate --path spec --strict-requires
generated/ and spec/suite.json match spec/
task: [check] ... cargo fmt -p thalamus-core ... -- --check
task: [check] ... cargo clippy --locked --workspace --all-targets -- -D warnings
task: [test] ... cargo test --locked --workspace
     Running tests/adversary_records.rs (target/debug/deps/adversary_records-25cb5e5924da846e)
test an_issued_sample_id_is_never_issued_again ... FAILED
thread 'an_issued_sample_id_is_never_issued_again' (2277131) panicked at crates/thalamus-core/tests/adversary_records.rs:322:5:
assertion `left != right` failed: a deleted sample's id was issued to a new sample
  left: SampleId("smp_2")
 right: SampleId("smp_2")
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p thalamus-core --test adversary_records`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

- **First attempt.** fmt failed on my own file only. I ran rustfmt on that file alone and nothing else.
- **Executed count.** `cargo test --locked --workspace --no-fail-fast` exited 101: 138 passed, 1 failed, 139 ran.
  - The implementor's `scratch/task-check.log` has 133 passed, so my 6 cases account for the difference.
  - `every_scenario_of_the_suite_holds ... ok` and `every_obligation_is_met ... ok`.
- **docs-check.** The gate stopped before docs-check, so I ran `task docs-check` separately with the patch applied: exit 0.

## 4. Findings

Covers the working tree above.

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/thalamus-core/src/behaviour/observation.rs:100 | CONFIRMED | introduced | M10 drops the SeriesSamples series filter and the whole suite stays green. Its one query reads a series holding every sample of the scenario. | Every SeriesSamples reader. Caught by `series_samples_serves_only_the_named_series`. |
| 2 | crates/thalamus-core/src/behaviour/evaluation.rs:119 | CONFIRMED | introduced | M9 drops the SeriesVerdicts filter and the suite stays green. It is checked only with `contains`, and `every_obligation_is_met` stores 1 row. | The view's "current verdict" reader. Caught by `series_verdicts_serves_only_the_named_series`. |
| 3 | crates/thalamus-core/src/behaviour/observation.rs:110 | CONFIRMED | introduced | M1 (no sort) and M2 (sort by text) both stay green. The suite's one `ranked` check records its samples in time order, and that matches store/id order. | `order_by: [ts asc]`, which samplers backfilling or using mixed offsets rely on. Caught by the ordering case. |
| 4 | crates/thalamus-core/src/behaviour/observation.rs:43 | CONFIRMED | introduced | M5 (refuse only Proposed) stays green. The not-active scenario reaches Proposed only. | The Muted/Retired lifecycle. Caught by `muted_and_retired_definitions_refuse_a_sample`. |
| 5 | crates/thalamus-core/src/behaviour/evaluation.rs:136 | CONFIRMED | introduced | M3 and M4 (CRIT/UNKNOWN `scored` flipped) stay green. The suite uses OK and N_A only. | The grader recording CRIT or UNKNOWN. Caught by `crit_carries_a_score_and_unknown_refuses_one`. |
| 6 | crates/thalamus-core/src/behaviour/observation.rs:133 | INFEASIBLE | introduced | `mint` counts rows, so after the newest row is deleted it issues that id again (`smp_2` twice; red case at `adversary_records.rs:322`). Two stores also mint the same ids. | Nothing today, because no command deletes. Design § 1 retention ("raw samples for a bounded window"), § 2.2 `smp_01JA…` and D13 (references outlive retention) make it reachable when retention ships. The same applies to `evl_` and `asr_`. |
| 7 | crates/thalamus-core/src/conformance.rs:1602 | INFEASIBLE | introduced | `rank_order` compares wire Decimals (text) byte by byte ("10" < "9"). The ESS 0.60.0 runner (`compare_nodes`, runner.rs:3546) ranks semantic numbers. | No suite `ranked` uses a Decimal field (the only one is `ts asc`). |
| 8 | crates/thalamus-core/src/conformance.rs:1577 | CONFIRMED | introduced | The out-of-order message prints `show(right)` (a field value) where it means `show(later)` (the row). | Any `ranked` failure, which then names the wrong thing. |
| 9 | generated/thalamus-model/src/server/thalamus-service.openapi.json | CONFIRMED | pre-existing | `x-ess-attributes` carry `{"$serde_json::private::Number": "2"}` instead of numbers: 20 occurrences at 5d0ee31. | Any consumer of `/openapi.json`. It comes from the ESS generator, not from this unit. |

Fix for 1 to 5: add cases like mine to the unit's tests, because `spec/suite.json` cannot be edited. 6 is a design call: mint a unique id (ULID per design § 2.2), or record that ids are store-local and never reused.

## 5. Attacked and not broken

- **Extra event fields admitted.** This matches the ESS 0.60.0 `expect_payload` (runner.rs:2159, "nothing in the model closes an event's payload"). It is not exploitable here: the generated event structs are closed, and the suite checks every field of SampleRecorded, EvaluationRecorded and AssertionAsked, by payload value or by response mapping.
- **Dotted leaves.** Same semantics as the reference `reach_into`: null on the path is absence, and a non-object blocks.
- **Enum-name mapping.** Verdict `N/A` is the only spelling that is not an identifier, and it maps uniquely. A vacuous `excludes` would need an ambiguous mapping, and the contract has none.
- **counts, any_of, satisfies.** They compare what they claim. A count with no bound is refused. `satisfies` with no rows fails. `any_of` on a missing field is undecided and fails.
- **ranked.** Same adjacent-pair algorithm as the reference runner.rs:3491, apart from finding 7.
- **RecordEvaluation precedence.** The suite pins it: N_A with no reason and a score expects reason-missing, and an unknown sample with N_A and no reason expects unknown-sample.
- **Id collisions within one store.** `mint` skips ids that are taken.
- **SeriesSamples ordering.** Mixed offsets order by instant. Ties keep store (id) order, and the spec declares no tie-break.
- **Vacuous scenarios.** None: every scenario has at least 1 positive check, and the run asserts that it ran exactly the suite's 73 ids.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary/` (about 560M):
  - `runner-0.60.0.rs`
  - `cases-alone.log`, `task-check.log`, `workspace-tests.log`, `docs-check.log`, `mutants.out`
  - `logs/M{1,2,3,4,5,9,10}-*.{diff,log}`
  - `pristine/{observation,evaluation}.rs`
  - `mutant-tree/`: a full copy of the worktree without `.git`, including its own `target/`. I left it in place because of the rule against deleting a `target/` by hand, so it needs cleanup.
- `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary/mutants.sh`: created, run, deleted.
- `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary-pass-1.md`: this report.
- Worktree session lease `adversary-m0-conformance-records-p1`: acquired and released.
- Inside the worktree I wrote build output to its own `target/` and nothing else.

## 7. Findings

```findings
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 100
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: removing the SeriesSamples series filter leaves all 73 scenarios and every_obligation_is_met green
- file: crates/thalamus-core/src/behaviour/evaluation.rs
  line: 119
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: removing the SeriesVerdicts series filter leaves the suite green because it is checked only with contains
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 110
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping the ts sort or sorting by text leaves the suite green because its only ranked check records samples already in order
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 43
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: refusing only Proposed definitions leaves the suite green, so Muted and Retired signals accepting samples goes unnoticed
- file: crates/thalamus-core/src/behaviour/evaluation.rs
  line: 136
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: flipping scored for CRIT or UNKNOWN leaves the suite green because it reaches only OK and N_A
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 133
  category: property
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: row-count minting reissues a deleted newest id (smp_2 twice), reachable only once retention deletes rows
- file: crates/thalamus-core/src/conformance.rs
  line: 1602
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: ranked orders wire Decimals by bytes where the ESS 0.60.0 runner orders them by value, and no suite expectation ranks a Decimal
- file: crates/thalamus-core/src/conformance.rs
  line: 1577
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the ranked failure message prints the later field value where it means the later row
- file: generated/thalamus-model/src/server/thalamus-service.openapi.json
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: x-ess-attributes publish serde_json private Number wrappers instead of numbers, 20 occurrences at base 5d0ee31
```
