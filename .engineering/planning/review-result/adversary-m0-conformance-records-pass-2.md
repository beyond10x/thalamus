---
format: aep.planning-md/3
id: review-result:adversary-m0-conformance-records-pass-2
kind: review-result
status: active
title: Adversary, story:m0-conformance-records, pass 2
relations:
- reviews: story:m0-conformance-records
revision: 1
---
unit: story:m0-conformance-records, uncommitted working tree on impl/m0-conformance-records (base 5d0ee31), ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m0-conformance-records
verdict: red. Strongest is CONFIRMED: 9 new mutants of the corrections, behaviours and harness pass every pre-existing test. 4 red cases, all INFEASIBLE today
cases: executed 142→153, red 4
origin: introduced 9 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 scratch directory plus this report (part 6)
needs-coordinator: yes. Decide whether the 11 cases in adversary_records_p2.rs stay. The 7 green ones are the only guards on the 9 surviving mutants. The 4 red ones are INFEASIBLE and hold `task check` red until they are fixed or dropped. Status patch left applied.

## 1. git --no-pager diff --stat (final state)

```
 crates/thalamus-core/src/behaviour/assertion.rs   |  66 ++
 crates/thalamus-core/src/behaviour/evaluation.rs  | 138 +++
 crates/thalamus-core/src/behaviour/observation.rs | 165 ++++
 crates/thalamus-core/src/conformance.rs           | 981 +++++++++++++++++++---
 crates/thalamus-core/tests/adv_conformance.rs     |   5 +-
 crates/thalamus-core/tests/conformance.rs         | 582 ++++++++++++-
 crates/thalamus-docs/src/status.rs                |   8 +-
 website/data/status.json                          |   4 +-
 8 files changed, 1784 insertions(+), 165 deletions(-)
?? crates/thalamus-core/tests/adversary_records.rs      (pass 1, unchanged)
?? crates/thalamus-core/tests/adversary_records_p2.rs   (mine, 691 lines, untracked)
```

- The tracked stat is the one I was handed. Those files belong to the implementor and the coordinator, and I edited none of them.
- `cmp` against copies taken before the mutant run: `conformance.rs`, `observation.rs`, `evaluation.rs` and `assertion.rs` are all unchanged.
- The only path I added is a test file. I used no git write command.

## 2. Cases added: `crates/thalamus-core/tests/adversary_records_p2.rs`

- The harness cases build a one-scenario suite from the steps of `spec/suite.json` and run it through `conformance::run`.
- Their expected verdicts are those of the ESS 0.60.0 reference:
  - `compare_nodes`, at `runner.rs:3546` in pass 1's copy;
  - `Rfc3339Instant::parse_rfc3339`, at `~/beyond10x/ess/crates/specify/ess-primitives/src/time.rs:741`.

| line | case | asserts | now | kills |
|---|---|---|---|---|
| 244 | `a_decimal_and_an_integer_rank_by_value_through_the_harness` | On Evaluations, score 9 then 10 and rev 9 then 10: `score asc` and `definition_rev asc` pass, `[definition_rev asc, score desc]` passes, `score desc` is refused | green | P4, P6, P7, P17 |
| 272 | `mixed_offsets_and_fractions_rank_by_instant_through_the_harness` | SeriesSamples with `09:00:03Z`, `09:00:02.5Z`, `10:00:01+02:00`, `10:00:02.25+01:00`: `ts asc` passes, `ts desc` is refused | green | P5 |
| 291 | `the_harness_accepts_the_order_series_samples_serves` | SeriesSamples with `2026-01-05T00:00:00Z` and `2026-01-04T00:00:00` (no offset) passes the harness's own `ts asc` | **red** | none (finding 7) |
| 307 | `a_text_the_reference_reads_as_no_instant_ranks_by_its_bytes` | On Evaluations, rows the reference refuses as `ts asc` (`…00.0000000001+02:00` then `…09:00:00Z`, and `…T09:00:03Z` then `2026-01-05 10:00:01Z`) are refused | **red** | none (finding 8) |
| 341 | `an_integer_ranks_exactly_past_two_to_the_fifty_three` | rev 9007199254740993 then 9007199254740992 is refused as `definition_rev asc` | **red** | none (finding 9) |
| 358 | `a_tie_on_the_first_key_is_decided_by_the_second` | equal rev with scores 10 then 9: `[rev asc, score desc]` passes and `[rev asc, score asc]` is refused | green | P13 |
| 519 | `a_second_ports_over_one_store_keeps_every_stored_sample` | a second `Ports` over a clone-shared `MemoryPorts` gets a new id, and the store holds all 3 samples | green | P1 |
| 542 | `a_second_ports_over_one_store_never_reissues_a_deleted_id` | after the newest sample is deleted, a second `Ports` over the same store does not reissue its id | **red** | none (finding 6) |
| 598 | `a_spent_sequence_answers_unmet_and_writes_nothing` | with an `IdSequence` that answers `None`, the 3 minting behaviours return `UnmetObligation{"identity sequence", "smp_"/"evl_"/"asr_"}`, and no sample, evaluation or assertion row is written | green | P2 |
| 641 | `every_verdict_but_ok_needs_a_reason` | WARN, CRIT, UNKNOWN and N/A without a reason give `ReasonMissing`; OK without a reason is `Recorded` | green | P9 |
| 670 | `a_revision_below_one_is_unmet_and_stores_nothing` | `definition_rev` 0 gives `UnmetObligation{"entity invariant", "definition_rev >= 1"}` and no row | green | P12 |

**First run, case file alone.** The command was `cargo test --locked -p thalamus-core --test adversary_records_p2`, with 9 cases. It exited 101. Verbatim, with long rows elided as `…`:

```
test a_second_ports_over_one_store_never_reissues_a_deleted_id ... FAILED
test the_harness_accepts_the_order_series_samples_serves ... FAILED
test an_integer_ranks_exactly_past_two_to_the_fifty_three ... FAILED
test a_text_the_reference_reads_as_no_instant_ranks_by_its_bytes ... FAILED
thread 'a_second_ports_over_one_store_never_reissues_a_deleted_id' (2850491) panicked at crates/thalamus-core/tests/adversary_records_p2.rs:517:5:
assertion `left != right` failed: a deleted sample's id was issued again by a second Ports over the store
  left: SampleId("smp_2")
 right: SampleId("smp_2")
thread 'the_harness_accepts_the_order_series_samples_serves' (2850497) panicked at crates/thalamus-core/tests/adversary_records_p2.rs:295:5:
assertion `left == right` failed: SeriesSamples serves rows its own ranked check refuses
  left: Err("step 23 ({\"step\":\"expect_view\",\"view\":\"thalamus.observation.SeriesSamples\",\"expectation\":{\"expect\":\"ranked\",\"order_by\":[\"ts asc\"]}}): thalamus.observation.SeriesSamples serves {\"sample_id\":\"smp_1\",\"ts\":\"2026-01-05T00:00:00Z\",…} then {\"sample_id\":\"smp_2\",\"ts\":\"2026-01-04T00:00:00\",…}, out of order by ts asc")
 right: Ok(())
thread 'a_text_the_reference_reads_as_no_instant_ranks_by_its_bytes' (2850493) panicked at crates/thalamus-core/tests/adversary_records_p2.rs:317:9:
2026-01-05T10:00:00.0000000001+02:00 then 2026-01-05T09:00:00Z passed ts asc; the reference orders them by bytes and refuses
thread 'an_integer_ranks_exactly_past_two_to_the_fifty_three' (2850494) panicked at crates/thalamus-core/tests/adversary_records_p2.rs:335:5:
9007199254740993 then 9007199254740992 passed definition_rev asc
test result: FAILED. 5 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

**Second run, alone.** I tightened the checks: `is_err` became "refused at the `ranked` step", and the text case now collects both pairs. The run exited 101 with `5 passed; 4 failed` and showed both pairs:

```
  left: ["2026-01-05T10:00:00.0000000001+02:00 then 2026-01-05T09:00:00Z: Ok(())", "2026-01-05T09:00:03Z then 2026-01-05 10:00:01Z: Ok(())"]
 right: []
```

**Cases at 358 and 670.** I added them after the mutant run showed P12 and P13 surviving. Run alone they exited 0 (`2 passed; 0 failed; 9 filtered out`). On the mutated copy, P12 failed `a_revision_below_one…` and P13 failed `a_tie_on_the_first_key…`.

**Mutant run.** It ran on a scratch copy, never on the worktree. The command was `cargo test --locked --no-fail-fast -p thalamus-core`. Diffs and logs are in `scratch/adversary-p2/logs/P*.{diff,log}`. Baseline P0: every binary passes except the 4 red cases above.

| mutant | change | pre-existing tests | my cases |
|---|---|---|---|
| P1 | `observation.rs:162` `taken` check dropped | all ok | keeps_every_stored_sample FAILED |
| P2 | `observation.rs:155` spent sequence answers 0 | all ok | spent_sequence FAILED |
| P4 | `conformance.rs:855` the Row-schema decimal lookup returns false | all ok | decimal_and_integer FAILED |
| P5 | `conformance.rs:1652` instants compared as bytes | all ok | mixed_offsets FAILED |
| P6 | `conformance.rs:1642` number branch dropped | all ok | decimal_and_integer FAILED |
| P7 | `conformance.rs:1621` decided key `break` → `continue` | all ok | decimal_and_integer FAILED |
| P9 | `evaluation.rs:44` reason required only for N_A | all ok | every_verdict FAILED |
| P12 | `evaluation.rs:76` invariant check dropped | all ok | revision_below_one FAILED |
| P13 | `conformance.rs:1618` first-key tie `continue` → `break` | all ok | tie_on_the_first_key FAILED |
| P8, P16, P17 | `decimal_order` trailing-zero trim, `-0` sign, whole-length compare | lib unit test FAILED | P17 also FAILED in mine |
| P10 | `evaluation.rs:136` WARN not scored | `every_obligation_is_met` FAILED | n/a |
| P11 | `assertion.rs:59` invariant check dropped | all ok | all ok: an equivalent mutant (part 5) |
| P14 | `observation.rs:120-121` non-instants sorted first | all ok | turns the case at 291 green (it is finding 7, not a new guard) |

## 3. Gate run, after the cases existed

The command was `task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, with the status patch applied. It exited **201**:

```
task: [check] ~/.cache/ess/toolchains/0.60.0/ess specify validate --path spec --strict-requires
task: [drift] set -eu
task: [check] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo fmt -p thalamus-core ... -- --check
task: [check] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo clippy --locked --workspace --all-targets -- -D warnings
task: [test] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo test --locked --workspace
test result: ok. 8 passed; 0 failed; ... (adversary_records)
test a_second_ports_over_one_store_never_reissues_a_deleted_id ... FAILED
test an_integer_ranks_exactly_past_two_to_the_fifty_three ... FAILED
test a_text_the_reference_reads_as_no_instant_ranks_by_its_bytes ... FAILED
test the_harness_accepts_the_order_series_samples_serves ... FAILED
test result: FAILED. 7 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
error: test failed, to rerun pass `-p thalamus-core --test adversary_records_p2`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

- **Steps before the tests.** validate, drift, fmt and clippy all passed.
- **Executed count.** `cargo test --locked --workspace --no-fail-fast` exited 101: 149 passed, 4 failed, 153 ran.
  - Without my 11 cases that is 142, which matches the implementor's `scratch/task-check-2.log` (142 passed).
  - `every_scenario_of_the_suite_holds ... ok` and `every_obligation_is_met ... ok`.
- **docs-check.** The gate stops before docs-check, so I ran `task docs-check ESS=…0.60.0/ess` separately: exit 0.

## 4. Findings

Covers the working tree above.

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/thalamus-core/src/behaviour/observation.rs:155 | CONFIRMED | introduced | P1 (`taken` skip dropped) and P2 (spent sequence answers 0) leave every pre-existing test green. Pass 1's id cases use one `Ports` and never exhaust it. | A store holding rows that its `Ports` did not mint, and sequence exhaustion. Caught by cases 519 and 598. |
| 2 | crates/thalamus-core/src/conformance.rs:855 | CONFIRMED | introduced | P4 (Row-schema decimal lookup returns false) stays green. The unit test at :2415 passes a hand-built `Ranking`, so nothing drives the lookup, which is the fix for pass-1 finding 7. | Any suite `ranked` on a Decimal field. Caught by case 244. |
| 3 | crates/thalamus-core/src/conformance.rs:1617 | CONFIRMED | introduced | P5 (instant branch), P6 (number branch), P7 (decided key continues) and P13 (tie breaks) all stay green. The suite's one `ranked` (ts asc) compares equal instants. | Every `ranked` expectation. Caught by cases 244, 272 and 358. |
| 4 | crates/thalamus-core/src/behaviour/evaluation.rs:44 | CONFIRMED | introduced | P9 (reason required only for N_A) stays green. The suite's reason-missing scenario uses N_A only. | A grader recording WARN, CRIT or UNKNOWN without a reason. Caught by case 641. |
| 5 | crates/thalamus-core/src/behaviour/evaluation.rs:76 | CONFIRMED | introduced | P12 (invariant check dropped) stays green, and a rev-0 evaluation is stored. | `definition_rev` 0 or below on the wire: `integer_at` takes any i64. Caught by case 670. |
| 6 | crates/thalamus-core/src/behaviour/observation.rs:134 | INFEASIBLE | introduced | The `IdSequence` doc says the sequence is "held by the store". It is held by `Ports` (conformance.rs:74). A second `Ports` over a clone-shared `MemoryPorts` reissues a deleted id (`smp_2` twice; red at adversary_records_p2.rs:551). | Nothing: `run` builds one `Ports` per scenario over fresh storage, and nothing deletes. It is the residue of pass-1 finding 6, which is narrowed but still reachable once a store outlives its `Ports`. |
| 7 | crates/thalamus-core/src/behaviour/observation.rs:116 | INFEASIBLE | introduced | `instant_order` sorts a non-instant ts after every instant. The unit's own `rank_order`, like the reference, orders a mixed pair by bytes, so the harness refuses SeriesSamples' own answer (red at :295). | The generated decoder admits any text as `Timestamp` (wire.rs `text_at`), but no suite or producer sends a ts without an offset. |
| 8 | crates/thalamus-core/src/conformance.rs:1647 | INFEASIBLE | introduced | `rank_order` and `instant_order` read instants with the `time` crate, which accepts a space separator, more than 9 fraction digits and `:60`. The reference reads none of these as an instant. The harness passes 2 orders the reference refuses (red at :326). | No suite ts has these shapes. A future sampler passing a SQL `YYYY-MM-DD HH:MM:SS` ts would reach it. |
| 9 | crates/thalamus-core/src/conformance.rs:1642 | INFEASIBLE | introduced | Numbers rank as `f64`, so 9007199254740993 and 9007199254740992 tie and pass `definition_rev asc` (red at :348). The reference compares exactly. | No suite `ranked` on an Integer, and no revision above 2^53. |

Fixes I name but have not applied:
- **1 to 5:** keep cases like mine, because `spec/suite.json` cannot be edited.
- **6:** keep the sequence in the store (or mint a ULID), or reword the doc to say the sequence is per-`Ports`.
- **7 and 8:** use one instant reader, the reference's grammar, in `instant_order` and `rank_order`, and make the behaviour's mixed-pair order the checker's.
- **9:** compare integer spellings exactly.

## 5. Attacked and not broken

- **Spent sequence and partial writes.** A spent sequence writes no row in any of the 3 behaviours (case 598). `mint` runs before every `put`. In `record_evaluation`, a broken invariant after `mint` only skips a number.
- **u64::MAX.** `last.checked_add(1)?` returns before it assigns, so the counter stays at MAX and keeps answering `None`; it never wraps (conformance.rs:95-96). Tests cannot reach it because `Sequences` is private.
- **Two `Ports` over one store, nothing deleted.** No collision: `mint` skips taken ids (case 519).
- **Decimal ranking.** `-0` equals `0` and `0.0`, and `0.10` equals `0.1` (lib test :2431 and the code).
  - Exponent, `+` and leading-zero forms are refused by both `decimal()` and the generated `decimal_at`, so they never reach a row.
  - Non-decimal text, or text against a number, in a decimal field gives `None`, and `ranked` fails closed.
- **Mixed offsets and fractions of up to 9 digits.** These rank by instant correctly (case 272).
- **`.Row` lookup.** It fails open (falls back to bytes) only when `{view}.Row` or the field is missing. All 11 views declare a Row with `additionalProperties: false`, and an undeclared field fails earlier with "do not both hold".
- **P11 is an equivalent mutant.** `ask_assertion` builds `Asked` with result, worst and reason all `None`, so the invariant cannot break.
- **Already caught.** P8, P16, P17 (`decimal_order`) and P10 (WARN scored) are caught by existing tests.
- **Ranked message (pass-1 finding 8).** It now prints the later row, and the lib test at :2458 asserts this.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary-p2/` (480K, left in place):
  - `mutants.out`
  - `pristine/{conformance,observation,evaluation,assertion}.rs`
  - `logs/{cases-alone,cases-alone-2,cases-alone-3,P0-baseline,task-check,workspace-tests,docs-check}.log`
  - `logs/P{1,2,4,5,6,7,8,9,10,11,12,13,14,16,17}.{diff,log}` and `logs/P{12,13}-rerun.log`
- Created and deleted:
  - `scratch/adversary-p2/mutant-tree/`: a copy of the worktree without `.git`. I ran `cargo clean` (289.2MiB) and then deleted it.
  - `scratch/adversary-p2/probe/`: a `time` 0.3.45 parse probe. I ran `cargo clean` and then deleted it.
  - `scratch/adversary-p2/mutants.sh`
- `~/.cache/thalamus-waves/20261015c/m0-conformance-records/scratch/adversary-pass-2.md`: this report.
- Worktree session lease `adversary-m0-conformance-records-p2`: acquired and released.
- Inside the worktree I wrote build output to its own `target/`, plus my test file.

## 7. Findings

```findings
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 155
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping mint's taken-id skip or answering an id from a spent sequence leaves every pre-existing test green
- file: crates/thalamus-core/src/conformance.rs
  line: 855
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the Row-schema lookup that makes a Decimal rank by value is driven by no test, so disabling it stays green
- file: crates/thalamus-core/src/conformance.rs
  line: 1617
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: four ranked/rank_order mutants (instant branch, number branch, decided-key break, first-key tie) survive the suite
- file: crates/thalamus-core/src/behaviour/evaluation.rs
  line: 44
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: requiring a reason only for N_A stays green because the suite's reason-missing scenario uses N_A alone
- file: crates/thalamus-core/src/behaviour/evaluation.rs
  line: 76
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping the definition_rev >= 1 invariant check stays green and stores a revision-0 evaluation
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 134
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the sequence lives in Ports, not the store, so a second Ports over a shared store reissues a deleted id (smp_2 twice)
- file: crates/thalamus-core/src/behaviour/observation.rs
  line: 116
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: SeriesSamples orders a non-instant ts after every instant while the unit's own ranked check orders the pair by bytes and refuses it
- file: crates/thalamus-core/src/conformance.rs
  line: 1647
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the time crate reads space-separated, over-nine-digit and :60 texts as instants that the reference orders by bytes, so the harness passes orders the reference refuses
- file: crates/thalamus-core/src/conformance.rs
  line: 1642
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: numbers rank as f64, so integers past 2^53 tie and an out-of-order pair passes where the reference compares exactly
```
