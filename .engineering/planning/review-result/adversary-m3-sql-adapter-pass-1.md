---
format: aep.planning-md/3
id: review-result:adversary-m3-sql-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m3-sql-adapter, pass 1
relations:
- reviews: story:m3-sql-adapter
revision: 1
---
unit: story:m3-sql-adapter, working tree of impl/m3-sql-adapter (base c2897fe, implementation uncommitted)
verdict: NEEDS-CHANGE
cases: executed 27→40, red 6
origin: introduced 8, pre-existing 0, undecided 0
wrote-outside-worktree: 8 paths under ~/.cache/thalamus-waves/20261015c/m3-sql-adapter/scratch/ (part 6)
needs-coordinator: yes (finding J1 is a design fork: a numerator row that GROUP BY never returns)

`executed 27→40`: 27 is the `thalamus-sources` package run with my file `tests/adv_sql.rs` deselected (the same run, summed without that binary: 3+0+2+6+8+8+0). 40 adds its 13 cases.

## 1. `git --no-pager diff --stat` (worktree)

```
 crates/thalamus-sources/src/sql.rs | 699 ++++++++++++++++++++++++++++++++++++-
 1 file changed, 695 insertions(+), 4 deletions(-)
```

That line is the implementor's change, and I did not touch the file. `git status --short`:
```
 M crates/thalamus-sources/src/sql.rs
?? crates/thalamus-sources/tests/adv_sql.rs          <- mine, the only path I wrote (575 lines, test file)
?? crates/thalamus-sources/tests/fixtures/sql/       <- implementor's
?? crates/thalamus-sources/tests/sql.rs              <- implementor's
```
No path outside the tests was written. `sql.rs` in the tree equals the pre-mutation copy (`mutants.diff` shows only the six mutated lines). `adv_sql.rs` passes rustfmt, clippy `-D warnings` (exit 0) and `b10x-gates scan-text` ("1 file(s) carry no private rule").

## 2. Cases added: `crates/thalamus-sources/tests/adv_sql.rs`

The recordings are built inside the tests, so a cut answer carries 1,000 rows. The real adapter sets `truncated` only when `rows.len() == limit` (`~/beyond10x/connectors/adapters/sql/src/lib.rs:432`, same in `mysql.rs`), and thalamus sends `limit: 1000`.

| case | asserts | now |
|---|---|---|
| `adv_sql_value_cut_result_is_not_silently_complete` | a `sql.value` answer with `truncated: true` and 1,000 rows yields at least one read that is not `ok` | **red** |
| `adv_sql_ratio_cut_result_is_not_silently_complete` | the same for `sql.ratio`, both sides cut | **red** |
| `adv_sql_two_value_columns_are_ambiguous` | columns `[cluster, value, value]` do not read `ok` | **red** |
| `adv_sql_two_dimension_columns_of_one_name_are_ambiguous` | columns `[cluster, cluster, value]` with `alpha`/`beta` are not read `ok` as `{cluster: alpha}` | **red** |
| `adv_sql_ratio_null_dimension_does_not_pair_with_an_absent_column` | a numerator row with a NULL tenant is not paired with a denominator that returns no tenant column | **red** |
| `adv_sql_ratio_tiny_numerator_is_read` | `1e-40 / 1` reads `ok`, as the unit test's `1e-20 / 1` reads `0` | **red** |
| `adv_sql_type_tricks_never_read_a_number` | JSON number, JSON bool, PG `t`, JSON object cells: no read is `ok` | green |
| `adv_sql_huge_integers_stay_exact` | a 60-digit integer reads `ok` with every digit | green |
| `adv_sql_ratio_rounds_an_exact_half_away_from_zero` | `1 / 2e18` gives `0.000000000000000001`; `-1 / 2e18` gives `-0.000000000000000001` | green (mutant catcher) |
| `adv_sql_ratio_series_twice_in_one_side_is_unknown` | a numerator series returned twice reads unknown "twice" | green (mutant catcher) |
| `adv_sql_ratio_unpaired_series_are_unknown_never_zero` | a series on one side only reads unknown and never 0 | green (mutant catcher) |
| `adv_sql_empty_answers_read_one_unknown` | 0 rows give exactly one unknown read, for value and for ratio | green (mutant catcher) |
| `adv_sql_two_candidate_value_columns_are_unknown` | with no `value` column and two non-dimension columns, the read is unknown | green (mutant catcher) |

Red output from the first run of the file alone (`cargo test --locked -p thalamus-sources --test adv_sql`, exit 101), verbatim:
```
---- adv_sql_two_value_columns_are_ambiguous stdout ----
thread 'adv_sql_two_value_columns_are_ambiguous' (633702) panicked at crates/thalamus-sources/tests/adv_sql.rs:226:5:
two columns named `value`, and the first was read as the value: Ok(CompleteReading { value: Decimal("1") })
---- adv_sql_two_dimension_columns_of_one_name_are_ambiguous stdout ----
thread 'adv_sql_two_dimension_columns_of_one_name_are_ambiguous' (633701) panicked at crates/thalamus-sources/tests/adv_sql.rs:253:5:
two `cluster` columns (alpha, beta); the read was attributed to the first: Read {
    dims: {
        "cluster": "alpha",
    },
    reading: Ok(
        CompleteReading {
            value: Decimal(
                "5",
            ),
        },
    ),
    [...window, adapter, connection, digest elided...]
    raw: Some(
        "5",
    ),
---- adv_sql_ratio_null_dimension_does_not_pair_with_an_absent_column stdout ----
thread 'adv_sql_ratio_null_dimension_does_not_pair_with_an_absent_column' (633699) panicked at crates/thalamus-sources/tests/adv_sql.rs:294:5:
the NULL-tenant numerator row was paired with a denominator that has no tenant column: Ok(CompleteReading { value: Decimal("0.03") })
---- adv_sql_ratio_tiny_numerator_is_read stdout ----
thread 'adv_sql_ratio_tiny_numerator_is_read' (633700) panicked at crates/thalamus-sources/tests/adv_sql.rs:317:5:
1e-40 / 1 was not read: Unknown(UnreadReading { reason: "sql.ratio: the denominator 1 has more digits than an exact division takes" })
---- adv_sql_value_cut_result_is_not_silently_complete stdout ----
thread 'adv_sql_value_cut_result_is_not_silently_complete' (633704) panicked at crates/thalamus-sources/tests/adv_sql.rs:177:5:
a result Connectors reported truncated was read as 1000 complete `ok` reads; nothing records the cut
---- adv_sql_ratio_cut_result_is_not_silently_complete stdout ----
thread 'adv_sql_ratio_cut_result_is_not_silently_complete' (633698) panicked at crates/thalamus-sources/tests/adv_sql.rs:199:5:
two results Connectors reported truncated were read as 1000 complete `ok` ratios
test result: FAILED. 2 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```
I added the five catchers next and ran the file alone again: `test result: FAILED. 7 passed; 6 failed`, the same six red.

**Mutant probe.** The probe ran on a copy at `scratch/adversary/mutant-tree`, never on the tree under attack. It changed six lines (`scratch/adversary/mutants.diff`):
- `step(0) >= 5` became `> 5`
- the duplicate check in `side()` was disabled
- a missing ratio side was read as `number(Some("0"))`
- the guard `series.is_empty()` was disabled
- the guard `answer.rows.is_empty()` in `value()` was disabled
- `[at] => Ok(*at)` became `[at, ..]`

Results on the mutated copy:
- Implementor's suite: `--test sql` gave `test result: ok. 8 passed; 0 failed`. The unit tests gave `test result: ok. 3 passed; 0 failed`.
- Mine: each of the five catchers failed. For example, `expected unknown, got Ok(CompleteReading { value: Decimal("0") })` for the missing side read as 0, and `... Decimal("0.2")` for the duplicate numerator.

## 3. Suite run (after the cases existed)

`cargo test --locked -p thalamus-sources --no-fail-fast` gave exit 101:
```
     Running unittests src/lib.rs (target/debug/deps/thalamus_sources-4d66c42b8590df3d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/bin/connectors-double.rs (target/debug/deps/connectors_double-ba87b135b333fe63)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/adv2_connectors_client.rs (target/debug/deps/adv2_connectors_client-440cff7c6b4b7553)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adv_connectors_client.rs (target/debug/deps/adv_connectors_client-c57a1285dbf980f2)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/adv_sql.rs (target/debug/deps/adv_sql-47b8b54cfd19e05c)
test result: FAILED. 7 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
error: test failed, to rerun pass `-p thalamus-sources --test adv_sql`
     Running tests/connectors_client.rs (target/debug/deps/connectors_client-db1ed0bd6892c612)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sql.rs (target/debug/deps/sql-edbb80ea08f6fac1)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```
The suite is red, and that is the outcome this pass was after. After rustfmt reflowed my file, it was re-run alone: `7 passed; 6 failed`.

## 4. Findings (working tree of impl/m3-sql-adapter)

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | `crates/thalamus-sources/src/sql.rs:273` (`value()`), doc `:27-28` | A 1,000-row answer with `truncated: true` gives 1,000 plain `ok` reads. Nothing records the cut. Series past the cut vanish, which contradicts `registry.rs:5-6` ("never a dropped series"). The last series reads `ok` even when its further rows were cut, so a one-row duplicate escapes the "twice" rule. The story's Behaviour bullet says "A result the adapter reports as truncated: `partial`" (design § 2.2: `partial` = "read but cut"). `adv_sql.rs:167`, exit 101. | Any `sql.value` query whose result exceeds 1,000 rows, for example a per-tenant gauge over more than 1,000 tenants. Connectors cuts at `rows == limit` (`lib.rs:432`), and thalamus always sends `limit: 1000` (`sql.rs` `ROW_LIMIT`). | NEEDS-CHANGE | introduced |
| F2 | `sql.rs:370` (`ratio()`) | Both sides cut at 1,000 rows give 1,000 `ok` ratios. Series past the cut vanish without an `unknown` read. `adv_sql.rs:193`, exit 101. | The same as F1: any per-series ratio with more than 1,000 groups on both sides. | NEEDS-CHANGE | introduced |
| F3 | `sql.rs:229` | With columns `[cluster, value, value]`, the first `value` is read `ok` ("1") and the second is ignored. `adv_sql.rs:215`. | A join or `SELECT *` whose result has two columns named `value`. Connectors passes duplicate names through: names come from the prepared original (`lib.rs:377-383`), and positional aliases exist so that "duplicate result names cannot collide" (`mysql.rs` comment above `wrapped`). | NEEDS-CHANGE | introduced |
| F4 | `sql.rs:252` | With columns `[cluster, cluster, value]` holding `alpha`/`beta`, the read is `ok` and attributed to `{cluster: alpha}` (L9 misattribution). `adv_sql.rs:247`. | The same path as F3: a self-join or a two-table join where both tables have the dimension column. | NEEDS-CHANGE | introduced |
| F5 | `sql.rs:260` (`dims_of`), `sql.rs:352` (`side`) | A NULL dimension cell and a column the query does not return both give "no key", so ratio pairing cannot tell them apart. A numerator row with tenant NULL (3) was divided by the whole-cluster denominator (100) and read `ok` `0.03` for `{cluster: alpha}`. `adv_sql.rs:282`. | A ratio definition whose numerator and denominator return different dimension columns. That is an authoring mistake, and nothing in the adapter or definition load refuses it. | NEEDS-CHANGE | introduced |
| F6 | `sql.rs:543` | `1e-40 / 1` reads unknown with the reason "the denominator 1 has more digits than an exact division takes": the numerator's scale is blamed on the denominator. `1e-20 / 1` reads `0` (the implementor's unit test). `adv_sql.rs:322`. | I found nothing that reaches it. It needs a numerator scale of 36 or more against a small denominator, for example a float8 below about 1e-35. | INFEASIBLE | introduced |
| F7 | `crates/thalamus-sources/tests/sql.rs` (whole suite) | The implementor's 8 integration tests and 3 unit tests stay green under six mutants: exact-half rounding, the ratio duplicate check, a missing ratio side read as **0** (an L5 violation that the suite would not catch), the empty-ratio guard, the empty-value guard, and the column-candidate ambiguity. Five green catchers in `adv_sql.rs:387-528` kill all six mutants. | Any regression in those lines ships green. | CONFIRMED | introduced |
| J1 | `sql.rs:412` | Judgement. A series present only in the denominator reads unknown. Design § 2.1's example (`request_failure_ratio`, dims `[cluster, namespace]`) needs `GROUP BY` queries, and a GROUP BY numerator returns no row for a namespace with zero failures. Every healthy series would read `UNKNOWN` (exit 3) on every pass unless authors write `LEFT JOIN … COALESCE`. Deciding whether an absent row in an uncut answer is 0 is a design fork against L5 ("NULL and unparseable", not "absent"), so the coordinator decides. | The design's own example definition, written in the natural SQL form. | NEEDS-CHANGE | introduced |
| J2 | `tests/fixtures/sql/truncated/…query.read.json`, `tests/sql.rs:334` | Judgement. Recordings have `truncated: true` with 2 or 3 rows under `limit: 1000`, a shape the real adapter never returns. `sql_truncated_result_is_partial` also asserts that a cut `sql.value` result reads `ok`, which pins F1 as intended behaviour. | Only the fixtures. | CONFIRMED | introduced |
| J3 | `sql.rs:94` | Judgement. `sql.count` is an extractor that the story neither names nor scopes. It ignores declared dimensions, so a per-tenant count definition gets one instance-wide read. | A definition naming `sql.count`. The story only says "`lower` for a count". | CONFIRMED | introduced |

Fixes, named and not applied:
- F1/F2: on a cut answer, mark every read `partial` or `unknown`, or add one `unknown` read naming the cut. Either way, `sql_truncated_result_is_partial`'s `ok("4")`/`ok("6")` assertions must change, which is the implementor's call.
- F3/F4: treat a duplicate column name for `value` or for a declared dimension as an unknown with a reason.
- F5: pair only rows whose sides returned the same declared dimension columns, or refuse a ratio whose two answers return different dimension column sets.
- F6: bound by the numerator's scale and name the right operand, or rescale before the 37-digit check.

## 5. Attacked and not broken

- Cells that are not text or null (JSON number, bool, object) refuse the whole answer, which becomes one unknown. PG `t` reads unknown and never 0.
- A 60-digit integer keeps every digit. Exponent text expands exactly (implementor's cases plus mine).
- NaN, Infinity, empty text and words read unknown with `raw` kept. A NULL cell reads unknown with no `raw`.
- Two NULL dimension cells in one `sql.value` answer collapse into one series, but they read unknown ("twice") rather than silently wrong. An SQL `GROUP BY` yields only one NULL group, so this is not reachable silently.
- Ratio with denominator 0 or `0.000` reads unknown. A unit other than `1` reads unknown. Rounding of 15/556 is correct, and exact-half rounding is correct on both signs.
- The digest follows design § 2.2's canonical JSON of the query map. The story allowed the implementor this rule.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015c/m3-sql-adapter/scratch/`:
- `adversary/adv_sql-alone.log`
- `adversary/adv_sql-alone-2.log`
- `adversary/suite.log`
- `adversary/mutants.diff`
- `adversary/mutant-run.log`
- `adversary/mutant-run-sql.log`
- `adversary/mutant-tree/`: a copy of the worktree without `.git` plus its own `target/`, 152M. Not deleted, per the rule against deleting a `target/` by hand.
- `adversary-pass-1.md` (this report)

Inside the worktree, the tests also wrote `target/tmp/adv_sql/*`, which is build-scratch.

```findings
[
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 273, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a sql.value answer Connectors reports truncated is read as 1,000 complete ok reads, so the cut is invisible and series past it are dropped without an unknown read"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 370, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a sql.ratio whose sides are both truncated reads every pair ok and drops the series past the cut silently"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 229, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "two result columns named value are resolved silently to the first, which reads ok"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 252, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "two result columns named for one declared dimension set the key from the first silently, misattributing the read (L9)"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 352, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a NULL dimension cell and an absent dimension column give the same dims, so a ratio pairs a NULL-tenant numerator with a whole-cluster denominator and reads ok"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 543, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a numerator with 36 or more fractional digits makes the ratio unknown with a reason blaming the denominator, and no reachable source producing it was found"},
  {"file": "crates/thalamus-sources/tests/sql.rs", "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the suite stays green under six mutants including a missing ratio side read as 0, and adv_sql.rs carries the five catchers"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 412, "category": "judgement", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a GROUP BY numerator omits zero-failure series, so the design's own ratio example reads every healthy series UNKNOWN on every pass; a design fork for the coordinator"},
  {"file": "crates/thalamus-sources/tests/fixtures/sql/truncated", "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the truncated recordings carry 2 to 3 rows under limit 1000, which the real adapter never answers, and the test built on them pins F1's ok reads as intended"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 94, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "sql.count is an extractor outside the story that ignores declared dimensions"}
]
```
