---
format: aep.planning-md/3
id: review-result:adversary-m2-spec-history-records-pass-2
kind: review-result
status: active
title: Adversary, story:m2-spec-history-records, pass 2
relations:
- reviews: story:m2-spec-history-records
revision: 1
---
unit: story:m2-spec-history-records, uncommitted working tree of impl/m2-spec-history-records (base 8a9be4e) after the pass-1 changes, worktree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m2-spec-history-records
verdict: red (strongest: NEEDS-CHANGE, warning)
cases: executed 416→420, red 1
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary-p2/, this report)
needs-coordinator: whether tightening `is_decimal` to the contract pattern (no leading zero) also applies to `warn`/`crit` thresholds, where the same gap predates this unit

## 1. Diff

No tracked file changed by me. `git --no-pager diff --stat` is the implementor's tree: `91 files changed, 43141 insertions(+), 18031 deletions(-)`. My only change is one new untracked test file:

```
?? crates/thalamus-core/tests/adv_history_records_p2.rs   (new, 4 cases)
```

`crates/thalamus-core/tests/adv_history_records_p1.rs` is kept unchanged. All mutants were run on a scratch copy. `rustfmt --edition 2024` ran on my file alone.

## 2. Cases added (`crates/thalamus-core/tests/adv_history_records_p2.rs`)

| test | what it asserts | now |
|---|---|---|
| `a_fixed_number_the_contract_refuses_is_refused_at_load` (:111) | The wire decoder refuses `evaluation.fixed: "05"` and admits `"5"`. A file with `fixed` `"05"`, `"00"` or `"007.5"` is refused at load as `decimal` (D29). Control: `"5"` loads and folds to 5. | **red** |
| `an_instant_with_another_baseline_is_refused_once` (:165) | An instant with `baselines: [offset_1d]` (muted and proposed) gives exactly `["thalamus.definition.InstantNeedsFixedBaseline"]`, not that rule twice | green; kills ML1 |
| `a_transition_is_ordered_by_instant_whatever_its_spelling` (:256) | `first_ts <= settled_ts` compares instants: `+02:00` and `.5Z` spellings that sort the wrong way by bytes are refused, and the reverse is recorded | green |
| `the_harness_compares_two_timestamp_fields_as_instants` (:397) | The harness's new `compare` form, run over `first_ts` 01:30+02:00 and `settled_ts` 00:00Z, answers `lt`/`lte`/`gte`/`eq`/`ne`/`gt` as the instants do | green; kills MH1–MH4 |

Red output from the first run, this file alone (`cargo test --locked -p thalamus-core --test adv_history_records_p2`, exit 101):

```
test a_transition_is_ordered_by_instant_whatever_its_spelling ... ok
test an_instant_with_another_baseline_is_refused_once ... ok
test a_fixed_number_the_contract_refuses_is_refused_at_load ... FAILED
test the_harness_compares_two_timestamp_fields_as_instants ... ok
thread 'a_fixed_number_the_contract_refuses_is_refused_at_load' (3934823) panicked at crates/thalamus-core/tests/adv_history_records_p2.rs:140:17:
a definition file with fixed "05", which the wire refuses, loaded; its fixed baseline folds to Unknown(UnreadReading { reason: "fixed baseline `05` is not a decimal" })
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

Mutants, each run on a scratch copy with `cargo test --locked -p thalamus-core --no-fail-fast`. In the unmutated copy only the red case above fails, out of 214 core tests.

| mutant | site | pre-existing tests | killed by |
|---|---|---|---|
| MH1: operands swapped, `timestamp_order(&right, &left)` | `conformance.rs:2029` | all green | `the_harness_compares_…` |
| MH2: `op` ignored, always `lte` | `conformance.rs:2019` | all green | same |
| MH3: byte order instead of instant order | `conformance.rs:2029` | all green | same |
| MH4: `Ok(Some(true))` | `conformance.rs:2029` | all green | same |
| ML1: `instant-not-fixed` skip removed, so the rule is reported twice | `registry.rs:1353` | all green | `an_instant_with_…` |

## 3. Suite run (after the cases existed)

`task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess`, exit 201:
- `ess specify validate --path spec --strict-requires`: `thalamus v1 — 9 file(s), valid`
- drift: `generated/ and spec/suite.json match spec/`
- fmt `--check`: passed after `rustfmt` of my file; the first run failed on my file's formatting only. clippy `--workspace --all-targets -D warnings`: passed.
- test: stops at the first red binary. 12 binaries were `ok` before it, `adv_history_records_p1` among them. Then `Running tests/adv_history_records_p2.rs` → `test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s`

`cargo test --locked --workspace --no-fail-fast`, exit 101: 420 ran, 419 passed, 1 failed (`a_fixed_number_the_contract_refuses_is_refused_at_load`). The `before` count of 416 is this same run with the 4 tests of `adv_history_records_p2` left out of the count.

ESS 0.60.0 reference runner (`ess verify conform run --target interpreted --path spec --suite spec/suite.json --report-format 2`), exit 0: `92 scenarios: 92 passed, 0 failed, 0 error, 0 unsupported`.

## 4. Findings (working tree above)

| # | file:line | finding | verdict | origin | reached by |
|---|---|---|---|---|---|
| 1 | `crates/thalamus-core/src/registry.rs:512` (`is_decimal`), used for `evaluation.fixed` at `:1282` | The loader admits a decimal with a leading zero (`"05"`, `"00"`, `"007.5"`). The contract pattern (`generated/thalamus-model/src/json.rs:537`, `decimal_at`) and `baseline::fold` (`Dec::parse`, `baseline/mod.rs:567`) both refuse it. So the file loads, propose over the wire would refuse the same number, and the `fixed` baseline reads `UNKNOWN` on every sample. That is the state `fixed-number-missing` (D25) exists to refuse, and it contradicts D29 ("a number that is not a decimal [is] refused"). Fix: give `is_decimal` the contract's no-leading-zero rule. | NEEDS-CHANGE | introduced (the `fixed` path is new; the same gap for `warn`/`crit` predates this unit) | An operator writing `fixed: "05"`, read through `Definition::parse` / `Instance::load`. No production loader caller in this tree yet. 1 red case. |
| 2 | `crates/thalamus-core/src/conformance.rs:1995` (`compare_truth`) | The harness's new timestamp `compare` form is reached by the suite only with `first_ts == settled_ts` (`2020-01-01T00:00:00Z`) under `lte`. An operand swap, an ignored `op`, byte order and constant `true` all left every pre-existing test green. | CONFIRMED | introduced | Every `Transition` invariant scenario. Closed by a green case. |
| 3 | `crates/thalamus-core/src/registry.rs:1353` | The skip that stops `instant-not-fixed` being reported twice is untested: `registry.rs`'s `refused` helper matches with `any`, and the conformance table has no instant row. | CONFIRMED | introduced | Any instant definition with a non-`fixed` baseline. Closed by a green case. |
| 4 | `crates/thalamus-core/src/behaviour/transition.rs:27` | The doc says the entity-invariant error is reached "only [by] a `definition_rev` below 1". After decision 2, `first_ts > settled_ts` and `from_verdict == to_verdict` reach it too. | CONFIRMED | introduced | Readers of `record_transition`, i.e. story:m2-transitions. |
| 5 | `generated/thalamus-model/src/server/entry.rs:42` | A broken invariant is an `UnmetObligation`, which the generated server answers as `501`. So an OK→OK or backwards `RecordTransition` over the network gets "not implemented" and no declared error. Read from the code; not run. | INFEASIBLE | introduced | `spec/components.yaml` accepts `RecordTransition` over the network; no caller posts it today. Only a declared refusal outcome would change it, and the coordinator chose invariants. |
| 6 | `docs/design/thalamus.md:441` | D25 says persistence defaults are "applied by `thalamus-core` (D29)". D29 says nothing about defaults; D15 holds them. | CONFIRMED | introduced | Readers of § 11. |
| 7 | `spec/domains/definition.yaml:145` | `Persistence` makes `k`, `k_crit` and `m` all required (the loader refuses `{k: 2, k_crit: 1}` as document shape). story:m2-hysteresis-core's Work still says "each value absent falling back to the design default". Only the whole block can be absent. | CONFIRMED | introduced | story:m2-hysteresis-core, when it implements the fallback. |

### Pass-1 findings now closed

| pass-1 # | state |
|---|---|
| 1 (loader cannot carry `fixed`, `rolling_n`, `persistence`) | closed: the 4 pass-1 cases are green, and the loader shares `input_refusals` |
| 2 (OK→OK and backwards transitions) | closed: both invariants are in the spec and both pass-1 cases are green. Offset and fraction spellings hold (new green case). |
| 3 (`rolling_n` < 1) | closed: refused by propose, revise and the loader, including `i64::MIN`, behind an `UNMAPPED:` marker held by `conformance.rs::rolling_median_over_fewer_than_one_sample_is_refused` |
| 4 (persistence < 1) | closed: `persistence-below-one`, all three fields, in propose, revise and the loader |
| 5–9 (mutants) | stay closed: the pass-1 green cases still pass |
| 10 (rollup sum, INFEASIBLE) | partly addressed: D27 now names synthesized fixtures that break the sum. The `worst_verdict` and `verdict_counts` parts are unmentioned; not re-raised |
| 11 (Service unmute without a marker) | closed: `UNMAPPED:` marker at `definition.yaml:296`, D28, held by m2-auto-unmute's `unexpired_mute_stays_muted` |
| 12 (dead `fixed` and `rolling_n`) | closed: `rolling-n-unused`, `fixed-number-missing`, `fixed-number-unused` |
| 13 (harness vs reference on unused forms, INFEASIBLE) | unchanged; the new `compare` form is covered by finding 2 |

## 5. Attacked and not broken

- `input_refusals` order matches the spec's declaration order. Propose, revise and the loader agree on every row of the conformance table.
- The `Transition` invariants compare instants, not bytes (`+02:00`, `.5Z`).
- ESS reference runner: 92/92. Spec validate and drift both pass.
- `website/docs/concepts/signals.md` rows for `fixed`, `rolling_n` and `persistence` match the loader.
- `baseline::fold` reads a loaded `fixed: "5"` as 5.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary-p2/`: `cases-alone.log`, `mutate.sh`, `mutants.out`, `mutant-logs/` (6 logs), `conformance.pristine.rs`, `registry.pristine.rs`, `ref-run.log`, `task-check.log`, `suite-nofailfast.log`. The scratch copy `copy/` was `cargo clean`ed (484.5 MiB) and deleted.
- `~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary-pass-2.md` (this report).
- The harness kept two background-task output files under `~/.cache/claude-tmp/claude-1000/`.

## 7. Findings block

```findings
- file: crates/thalamus-core/src/registry.rs
  line: 512
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the loader admits evaluation.fixed with a leading zero ("05"), which the wire contract and baseline::fold refuse, so the file loads and its fixed baseline grades UNKNOWN on every sample
- file: crates/thalamus-core/src/conformance.rs
  line: 1995
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: compare_truth's operand order, op, instant ordering and result survived every pre-existing test because the suite compares equal timestamps only
- file: crates/thalamus-core/src/registry.rs
  line: 1353
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: removing the instant-not-fixed skip, which reports that rule twice, left every pre-existing test green
- file: crates/thalamus-core/src/behaviour/transition.rs
  line: 27
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the doc says only definition_rev below 1 reaches the invariant error, but first_ts after settled_ts and from_verdict equal to to_verdict now reach it too
- file: generated/thalamus-model/src/server/entry.rs
  line: 42
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an invariant-breaking RecordTransition over the network is answered 501 with no declared error, since invariants were chosen over refusal outcomes
- file: docs/design/thalamus.md
  line: 441
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: D25 cites D29 for the persistence defaults, which D29 does not state; D15 does
- file: spec/domains/definition.yaml
  line: 145
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: Persistence requires all of k, k_crit and m, while story:m2-hysteresis-core plans a per-value fallback to the defaults
```
