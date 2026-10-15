---
format: aep.planning-md/3
id: review-result:adversary-m2-spec-history-records-pass-1
kind: review-result
status: active
title: Adversary, story:m2-spec-history-records, pass 1
relations:
- reviews: story:m2-spec-history-records
revision: 1
---
unit: story:m2-spec-history-records, uncommitted working tree of impl/m2-spec-history-records (base 8a9be4e), worktree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m2-spec-history-records
verdict: red (strongest: NEEDS-CHANGE)
cases: executed 400→414, red 8
origin: introduced 13 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary/, this report)
needs-coordinator: an owner for reading evaluation.fixed, evaluation.rolling_n and persistence from the definition file (crates/thalamus-core/src/registry.rs belongs to story:m1-instance-registry; no M2 story's scope names it)

## 1. Diff (tracked files unchanged, so the bound held)

`git --no-pager diff --stat` is the same as the tree I was handed: `88 files changed, 32852 insertions(+), 16969 deletions(-)`, all the implementor's. My only change is one new untracked test file:

```
.../thalamus-core/tests/adv_history_records_p1.rs  | 676 +++++++++++++++++++++
1 file changed, 676 insertions(+)
```

## 2. Cases added (`crates/thalamus-core/tests/adv_history_records_p1.rs`)

| test | what it asserts | now |
|---|---|---|
| `a_transition_from_a_verdict_to_itself_is_not_recorded` (:162) | design § 3.2: OK→OK is not recorded; a first verdict (no `from`) is recorded | red |
| `a_transition_does_not_settle_before_its_first_agreeing_evaluation` (:201) | D13: `first_ts` after `settled_ts` is not recorded | red |
| `a_rolling_median_over_no_samples_is_not_proposed` (:524) | § 3.1: `rolling_n` 0 or -3 is not proposed | red |
| `a_persistence_of_no_evaluations_is_not_proposed` (:543) | § 3.2: k, k_crit or m of 0 or less is not proposed | red |
| `a_definition_file_carries_the_number_of_its_fixed_baseline` (:584) | D25 and D19: a definition file can declare `evaluation.fixed` | red |
| `a_definition_file_carries_its_persistence` (:602) | D15 and D19: a definition file can declare `persistence` | red |
| `a_definition_file_carries_the_n_of_its_rolling_median` (:656) | D25: a definition file can declare `evaluation.rolling_n` | red |
| `a_definition_file_naming_rolling_median_without_its_n_is_refused` (:670) | D25: the loader refuses what `rolling-n-missing` refuses | red |
| `a_transition_below_revision_one_is_unmet_and_stores_nothing` (:225) | the invariant guard answers `UnmetObligation` and stores nothing | green; kills M16 |
| `the_harness_orders_two_integer_fields` (:349) | lt, gt, gte and lte over 1 < 3 and 3 = 3, both ways | green; kills M10–M13 |
| `the_harness_orders_two_decimal_fields_by_value` (:369) | Decimal 9 < 10, although the text "9" sorts after "10" | green; kills M18 |
| `series_transitions_answer_one_series_oldest_first` (:381) | two series recorded out of time order: filter and `settled_ts asc` | green; kills M01, M02 |
| `series_rollups_answer_one_series_oldest_day_first` (:443) | id order is the reverse of day order: filter and `day asc` | green; kills M03, M04 |
| `propose_and_revise_keep_the_persistence_they_are_given` (:479) | propose and revise store `persistence` (no view publishes it) | green; kills M05, M06 |

Red output, captured when each case first ran on its own (`cargo test --locked -p thalamus-core --test adv_history_records_p1`, exit 101):

```
an OK -> OK transition was recorded: Ok(Recorded { ... from_verdict: Some(OK), to_verdict: OK, settled_ts: Timestamp("2026-01-01T01:00:00Z") } })
a transition settled a day before its first agreeing evaluation: Ok(Recorded { ... from_verdict: Some(OK), to_verdict: WARN, settled_ts: Timestamp("2026-01-01T00:00:00Z") } })
definitions/case.yaml: document shape: evaluation: unknown field `fixed`, expected one of `rule`, `baselines`, `warn`, `crit`, `min_trend_window` at line 17 column 3
definitions/case.yaml: document shape: unknown field `persistence`, expected one of `id`, ..., `mute_until` at line 20 column 1
definitions/case.yaml: document shape: evaluation: unknown field `rolling_n`, expected one of `rule`, `baselines`, `warn`, `crit`, `min_trend_window` at line 17 column 3
a rolling_median definition without rolling_n loaded
test result: FAILED. 3 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out
```
```
persistence k 0, k_crit 1, m 3 was proposed
rolling_n 0 was proposed
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 9 filtered out
```

Mutants were run on a scratch copy (deleted after `cargo clean`), with `cargo test --locked -p thalamus-core --no-fail-fast`:

| mutant | survived the tests that existed before | killed by |
|---|---|---|
| M01/M03: series-view filter removed (`transition.rs:102`, `rollup.rs:28`) | yes | my series tests |
| M02/M04: `in_time_order` removed from the two series views | yes | my series tests |
| M05/M06: `persistence: None` in propose (`definition.rs:82`) and in revise (`:180`) | yes | `propose_and_revise_keep_…` |
| M10/M11/M12: `gt`→`is_ge`, `lt`→`is_le`, `gte`→`is_gt` (`conformance.rs:1961`) | yes | `the_harness_orders_two_integer_fields` |
| M13: `number_order(right, left)` in `fact_truth` (`conformance.rs:1979`) | yes | same |
| M18: decimal-text ordering branch disabled (`conformance.rs:1980`) | yes | `the_harness_orders_two_decimal_fields_by_value` |
| M16: invariant guard disabled (`transition.rs:60`) | yes | `a_transition_below_revision_one_…` |
| M07, M08 (rolling-n guard), M09 (`lte`), M14 (`ne`), M15 (fold ignores `fixed`), M17 (unknown-signal) | no: killed by `conformance.rs`, `baselines.rs` or `adv_*` | — |

## 3. Suite run (after the cases existed)

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, exit 201:
- `ess specify validate`: `thalamus v1 — 9 file(s), valid`
- drift: `generated/ and spec/suite.json match spec/`
- fmt: passed. clippy `--workspace --all-targets -D warnings`: passed.
- test: stops at the first red binary: `Running tests/adv_history_records_p1.rs` → `test result: FAILED. 6 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out`. Every binary before it was `ok`.
- `task docs-check` (run on its own because check stopped first): exit 0.

`cargo test --locked --workspace --no-fail-fast`, exit 101: 414 tests ran, 406 passed, 8 failed, all 8 in `adv_history_records_p1`. The same run with that file left out of the count: 400 tests ran, 0 failed. That is the `before` figure: it comes from this run, not from a run before the cases existed.

## 4. Findings (all on the working tree above)

| # | file:line | finding | verdict | origin | reached by |
|---|---|---|---|---|---|
| 1 | `crates/thalamus-core/src/registry.rs:835` (`EvaluationDoc`, `deny_unknown_fields`), `:1268-1271` | A definition file cannot declare `evaluation.fixed`, `evaluation.rolling_n` or `persistence`: the loader refuses each as an unknown field and sets it to `None`. Under D19 the file is the only copy of a definition, and under D25 `fold` reads the number only from `evaluation.fixed`. So every `fixed` baseline loaded from a file reads UNKNOWN; D12's advisory gates are graded against exactly that baseline. The loader also admits `rolling_median` without `rolling_n`, which `rolling-n-missing` refuses. Neither story:m2-store-baselines nor story:m2-hysteresis-core has `registry.rs` in its scope. | NEEDS-CHANGE | introduced | `Instance::load` / `Definition::parse` is the only loader. No production caller exists in this tree yet; story:m1-cli-sample is the first. 4 red cases. |
| 2 | `spec/domains/transition.yaml:51` | The specification admits a "transition" from a verdict to itself (OK→OK; the suite's own `RecordTransition/outcome/recorded` fixture records exactly that) and one that settled before its first agreeing evaluation. ESS 0.60.0 validates and synthesizes both guards, `first_ts <= settled_ts` and `any: ["not defined(from_verdict)", "from_verdict != to_verdict"]`, with 0 refusals for the transition domain; I probed this on a scratch copy. Fix: add the two invariants and regenerate; the existing `broken_invariant` check then refuses. | NEEDS-CHANGE | introduced | `RecordTransition` is accepted over the network (`spec/components.yaml`, actor `Hysteresis`), and story:m2-transitions is its caller. 2 red cases. |
| 3 | `spec/domains/definition.yaml:139` | `rolling_n` of 0 or below is proposed (§ 3.1: "median of the last N samples"). An outcome guard of the same form as `rolling-n-missing` could refuse it; I did not probe that guard against ESS. | NEEDS-CHANGE | introduced | `ProposeDefinition` / `ReviseDefinition`. 1 red case. |
| 4 | `spec/domains/definition.yaml:145` | A `persistence` with k, k_crit or m of 0 or below is proposed (§ 3.2: settle after k agreeing evaluations). An entity invariant is refused by synthesis (ESS-SYNTH-011: no view publishes `persistence`), so the guard has to be an outcome guard (not probed). | NEEDS-CHANGE | introduced | same. 1 red case. |
| 5 | `crates/thalamus-core/src/conformance.rs:1961` | The new `fact_truth` ordering was held only by `eq`, `ne` and one `lte` over equal values (1 ≤ 1). Mutating `gt`, `lt` or `gte`, or the operand order, left the whole suite green. | CONFIRMED | introduced | every future `satisfies` with a field-to-field order. Closed by 2 green cases. |
| 6 | `crates/thalamus-core/src/behaviour/transition.rs:102` | The filter and order of `series_transitions` were untested. The generated `ranked`, `contains` and `counts` expectations pass without either, because both rows are one series and are stored in time order. | CONFIRMED | introduced | the view's only reader. Closed by a green case. |
| 7 | `crates/thalamus-core/src/behaviour/rollup.rs:28` | Same as row 6, for `series_rollups`. | CONFIRMED | introduced | Closed by a green case. |
| 8 | `crates/thalamus-core/src/behaviour/definition.rs:82` | No test saw whether propose (`:82`) and revise (`:180`) keep `persistence`, because no view publishes it. ESS notes this on every `refuses` scenario. | CONFIRMED | introduced | story:m2-hysteresis-core reads the field. Closed by a green case. |
| 9 | `crates/thalamus-core/src/behaviour/transition.rs:60` | The `definition_rev >= 1` guard was unreached by every test. | CONFIRMED | introduced | Closed by a green case. |
| 10 | `spec/domains/rollup.yaml:88` | The D27 split admits `n < unknown + partial <= 2n`. The suite's `RecordRollup/outcome/recorded` and `DailyRollup/invariant/...` both record n 1, unknown 1, partial 1 as conforming, and ESS's interpreted reference runner passes them (84/84). The same row also carries `worst_verdict: OK` beside `verdict_counts.crit: 1`, and a verdict-count sum of 5 over n 1, which D14 (one evaluation per sample) rules out. ESS 0.60.0 cannot state a sum: an offset magnitude is a literal (`ess-primitives/src/predicate.rs:774`). | INFEASIBLE | introduced | story:m2-rollup's builder; the stored row is unguarded. |
| 11 | `spec/domains/definition.yaml:296` | `thalamus.definition.Service` may `UnmuteDefinition` unconditionally. D9 and D19 allow it only once the mute's expiry has passed, and no `UNMAPPED:` marker names that guard or the test that holds it. | CONFIRMED | introduced | story:m2-auto-unmute. |
| 12 | `spec/domains/definition.yaml:136` | `fixed` without a `fixed` baseline, and `rolling_n` without `rolling_median`, are accepted and never read: a dead threshold, which contradicts "one copy of every threshold". A `fixed` baseline without its number is proposed and grades UNKNOWN forever, whereas the missing-N case is refused. Both are expressible with the `rolling-n-missing` construct. | CONFIRMED | introduced | `ProposeDefinition` / `ReviseDefinition`. |
| 13 | `crates/thalamus-core/src/conformance.rs:1979` | The harness and the reference runner (ess 0.60.0 `satisfies` over `row_facts`) agree on every form the suite uses: both pass 84/84. They differ only on forms the suite does not use. (a) For two text facts the reference has no order (`FactStore` default: no scales, `orders_text_by_bytes` false), while the harness orders decimal text by value and treats "1.0" `eq` "1" as true. (b) The reference binds only presence for a list or object, so `eq` over one is Unknown there; the harness compares it structurally. (c) Integers above 2^53 compare exactly in the harness and as f64 in the reference. I did not check at runtime how the interpreted runner represents a Decimal. | INFEASIBLE | introduced | nothing in today's suite. |

## 5. Attacked and not broken

- The story's acceptance: `ess specify validate` reports `9 file(s), valid`. All five `IR | jq -e` checks exit 0.
- Domains against the story's Work: fields, relations (`references`/many via `evaluations`; `definition` via `signal`), lifecycle `Recorded`, outcomes, `SeriesTransitions` `order_by: [settled_ts asc]` and the D13 copied fields are all present. The rollup fields match § 4. The Persistence summaries match § 3.2 (k 2, k_crit 1, m 3).
- `fold` reading `fixed`: no existing result changed. `fold` has no non-test caller, and M15 is killed by 5 `baselines.rs` tests.
- Drift: a hand edit to `generated/thalamus-model/src/rollup.rs` fails `task drift` (exit 201, with the diff). An added file fails it (`Only in`). A clean tree passes. `.ess-output` is excluded and gitignored.
- `rolling-n-missing` in propose and revise (M07, M08) and `unknown-signal` on transitions (M17) are killed by the existing tests.
- The ESS 0.60.0 reference runner (`ess verify conform run --target interpreted`) on `spec/suite.json`: 84 passed, 0 failed.
- `satisfies` over an empty view fails in the harness (`conformance.rs:840`), as it does in the reference.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary/`: logs (`cases-alone*.log`, `task-check.log`, `suite-nofailfast.log`, `docs-check.log`, `drift-*.log`, `synth-probe*.log`, `ref-run-*.{log,json}`), `mutate.sh`, `mutate2.sh`, `mutants.out`, `mutants2.out`, `mutant-logs/`, `pristine/`, `*.orig.yaml`, `rollup-gen.orig.rs`, `ids-*.txt`. 5.3 MB. The scratch copy `copy/` (1.1 GB with target) was `cargo clean`ed and deleted; three probe suites were deleted.
- `~/.cache/thalamus-waves/20261015e/m2-spec-history-records/scratch/adversary-pass-1.md` (this report).
- `task drift` made temp directories `~/.cache/claude-tmp/thalamus-drift.*`; its trap removed them, and 0 remain.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/registry.rs", "line": 835, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the definition file, the only copy of a definition (D19), cannot declare evaluation.fixed, evaluation.rolling_n or persistence (refused as unknown fields, then set to None), so every file-loaded fixed baseline is UNKNOWN and rolling_median without rolling_n loads"},
  {"file": "spec/domains/transition.yaml", "line": 51, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Transition admits from_verdict == to_verdict and first_ts > settled_ts although ESS 0.60.0 validates and synthesizes both invariants, and the suite's own recorded fixture is an OK->OK transition"},
  {"file": "spec/domains/definition.yaml", "line": 139, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "rolling_n of 0 or below is proposed; a median of the last 0 samples has no meaning"},
  {"file": "spec/domains/definition.yaml", "line": 145, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "persistence k, k_crit or m of 0 or below is proposed, settling or clearing on no evaluation at all"},
  {"file": "crates/thalamus-core/src/conformance.rs", "line": 1961, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "fact_truth's gt, lt and gte mapping, its operand order and its decimal branch survived every pre-existing test; the new green cases kill them"},
  {"file": "crates/thalamus-core/src/behaviour/transition.rs", "line": 102, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "removing the series filter or the settled_ts order of series_transitions left every pre-existing test green, including the generated ranked expectation"},
  {"file": "crates/thalamus-core/src/behaviour/rollup.rs", "line": 28, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "removing the series filter or the day order of series_rollups left every pre-existing test green"},
  {"file": "crates/thalamus-core/src/behaviour/definition.rs", "line": 82, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping persistence in propose or revise left every pre-existing test green because no view publishes it"},
  {"file": "crates/thalamus-core/src/behaviour/transition.rs", "line": 60, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the definition_rev >= 1 guard of record_transition was reached by no test"},
  {"file": "spec/domains/rollup.yaml", "line": 88, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "the D27 split admits n < unknown + partial <= 2n, and the suite records n 1, unknown 1, partial 1 with worst_verdict OK beside crit 1 as conforming; ESS 0.60.0 cannot state a sum"},
  {"file": "spec/domains/definition.yaml", "line": 296, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the Service actor may UnmuteDefinition unconditionally while D9 and D19 allow it only past the mute's expiry, and no UNMAPPED marker names that guard or its test"},
  {"file": "spec/domains/definition.yaml", "line": 136, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "fixed without a fixed baseline and rolling_n without rolling_median are accepted and never read, and a fixed baseline without its number is proposed to grade UNKNOWN forever, unlike the refused missing N"},
  {"file": "crates/thalamus-core/src/conformance.rs", "line": 1979, "category": "contract-drift", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "the harness agrees with the ESS 0.60.0 reference on every form the suite uses (84/84 each) but orders decimal text by value, equates 1.0 with 1 and compares lists structurally where the reference answers Unknown"}
]
```
