---
format: aep.planning-md/3
id: review-result:adversary-m0-conformance-definitions-pass-2
kind: review-result
status: active
title: Adversary, story:m0-conformance-definitions, pass 2 (home paths shortened to ~)
relations:
- reviews: story:m0-conformance-definitions
revision: 1
---
```
unit: story:m0-conformance-definitions, uncommitted working tree on impl/m0-conformance-definitions (base b863833), pass 2 of 2
verdict: CONFIRMED and INFEASIBLE, notes only; nothing holds the unit
cases: executed 19→33, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (the scratch directory, part 6)
needs-coordinator: decide whether the 2 red notes are fixed now or recorded; neither is reached by the current suite
```

The correction holds. `SubjectShape` refuses every wrong union, enum, newtype, optional and nested struct I built. The one-row rule fails a view that serves its subject twice. All 12 pass-1 cases are green and 51/51 scenarios pass. What is left is 3 notes. Two are red cases on paths the suite never takes. The third is that the correction shipped without tests of its own.

**1. git diff --stat** (my files are untracked and do not show in `--stat`)
```
 Cargo.lock                                       |  155 ++
 crates/thalamus-core/Cargo.toml                  |    2 +-
 crates/thalamus-core/src/behaviour/definition.rs |  201 +++
 crates/thalamus-core/src/behaviour/dimension.rs  |   71 +
 crates/thalamus-core/src/conformance.rs          | 1756 ++++++++++++++++++++++
 crates/thalamus-core/tests/conformance.rs        |  220 +++
 6 files changed, 2404 insertions(+), 1 deletion(-)
?? crates/thalamus-core/tests/adv_conformance.rs         (mine, pass 1, unchanged since)
?? crates/thalamus-core/tests/adv_conformance_pass2.rs   (mine, pass 2)
```

**2. Cases added** in `~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m0-conformance-definitions/crates/thalamus-core/tests/adv_conformance_pass2.rs`. Each was run alone first; that output is in `p2-cases-alone.log`. Every shape is the suite's own shape for that view, changed in one place. Every refusal case also checks that the failure names the field.

| case | asserts | now |
|---|---|---|
| `the_suite_{dimension,definition}_shape_admits_its_own_row` :104, :253 | baseline: the unchanged shapes pass | green |
| union, 3 cases :118 :131 :144 | a unit variant served with a value, a carrying variant served without one, and an undeclared label are each refused | green |
| enum, newtype, optional, required-but-null :153 :166 :179 :192 | each is refused, naming its field | green |
| nested struct, 2 cases :258 :268 | an inner member of the wrong type, and a missing declared member, are each refused | green |
| `a_view_serving_the_subject_twice_fails_the_snapshot` :389 | storage that lists each row twice fails both real "already" scenarios with `matched 2 rows` | green |
| `a_decimal_served_as_a_json_number_is_refused` :281 | a `Decimal` served as a JSON number is refused, because under D11 it travels as a string | red: `a subject row breaking its declared rev passed` |
| `an_expectation_over_rows_read_before_the_last_command_fails` :298 | `expect_view` over rows read before a command fails, as in the ESS runner (`runtime.ts:4433`, ESS-CF-SUITE) | red: `excludes r1 held over rows read before r1 was declared` |

Mutants, run on a scratch copy. My pass-1 cases and the unit's 5 conformance tests stayed green under both, so only these pass-2 cases catch them.

| mutant in `SubjectShape::value` | unit's tests and pass-1 cases | caught by |
|---|---|---|
| unit variant with a value is admitted (`None => Ok(())`) | 12 + 5 passed | `a_unit_union_variant_served_with_a_value_is_refused` |
| the inner type of an optional is not checked | 12 + 5 passed | optional and nested-struct cases |

**3. Package test run, after the cases existed**: `cargo test --locked --no-fail-fast -p thalamus-core`, exit 101.
```
test result: ok. 1 passed; 0 failed; ...      (lib)
test result: ok. 12 passed; 0 failed; ...     (adv_conformance)
test result: ok. 1 passed; 0 failed; ...      (adv_conformance_feature)
test result: FAILED. 12 passed; 2 failed; ... (adv_conformance_pass2)
test result: ok. 5 passed; 0 failed; ...      (conformance; 51 selected, 51 ran, 0 failed)
test result: ok. 0 passed; 0 failed; ...      (probes, doc)
```
The "before" count of 19 comes from the same run with `adv_conformance_pass2` left out (exit 0). Clippy `--all-targets -D warnings` is clean, `fmt --check` exits 0, and scan-text reports `1 file(s) carry no private rule`. I did not run `task check`.

**4. Findings**
- `crates/thalamus-core/src/conformance.rs:713-717` (note, CONFIRMED, introduced): `expect_view` reads `self.views[view]` without the `queried` check the correction added for subject steps. So rows read before a command still answer an expectation about the state after it. What reaches it: nothing today. In all 73 scenarios, every `expect_view` comes after its own `query_view` with no command in between (checked with `jq`). Fix: require `queried == Some(view)`, as `subject_row` does.
- `crates/thalamus-core/src/conformance.rs:1140` (note, INFEASIBLE, introduced): `Decimal` accepts a JSON number as well as a decimal string. That covers both the reference runner (number) and D11 (string), so a `Decimal` served as a number would pass. What reaches it: nothing. No snapshot shape in any of the 73 scenarios declares a `Decimal`, and the generated encoder writes decimal strings.
- `crates/thalamus-core/src/conformance.rs:1062` (note, CONFIRMED, introduced): the unit added no test of `SubjectShape`'s type checks. The two mutants above survive its 5 tests and my 12 pass-1 cases. The pass-2 file now covers them.

**5. Attacked and could not break**
- Union admission: a unit variant with a value, a carrying variant without one, and an undeclared label are all refused.
- Enum, newtype, optional, required-null and nested-struct mismatches are all refused.
- A duplicate subject row fails with `matched 2 rows`.
- A subject step needs a query after the last command.
- `capture_instance.entity` is checked against `x-ess-entities`.
- The `Json` leniency, accepting any non-null value, cannot let a wrong type through, because `Json` declares no type.
- `Timestamp` and `Duration` accept any string, which matches the reference (`response.ts:1243-1245`).
- `Integer` accepting `2.0` matches the reference's `exactInteger`.

**6. Paths written outside the worktree**
- `~/.cache/thalamus-waves/20261015b/m0-conformance-definitions/scratch/adversary/` (185M), which holds:
  - pass-1 contents, plus `mut-tree/` (re-synced with the current tree, with its own `target/`; `conformance.rs` restored there)
  - `p1-adv.rs`
  - `p2-cases-alone.log`, `p2-suite-after.log`, `p2-suite-deselected.log`
  - `mutant-p2-m1-unit-variant-value-admitted.log`, `mutant-p2-m2-optional-inner-unchecked.log`
  - `run-mutant.sh` (edited)
- `~/.cache/thalamus-waves/20261015b/m0-conformance-definitions/scratch/adversary-pass-1.md` and `adversary-pass-2.md`

**7.**
```findings
- file: crates/thalamus-core/src/conformance.rs
  line: 713
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: expect_view reads the rows of the last query of that view even when a command ran after it, where the ESS reference runner fails ESS-CF-SUITE; no scenario of the suite takes that path
- file: crates/thalamus-core/src/conformance.rs
  line: 1140
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a Decimal subject field admits a JSON number as well as the decimal string D11 publishes, so a Decimal served as a number would pass; no snapshot shape in the suite declares a Decimal
- file: crates/thalamus-core/src/conformance.rs
  line: 1062
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: SubjectShape's type checks shipped without a test of their own, so two mutants of SubjectShape::value survive the unit's tests and the pass-1 cases; adv_conformance_pass2.rs now catches both
```
