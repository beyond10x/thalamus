---
format: aep.planning-md/3
id: review-result:adversary-m3-sql-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m3-sql-adapter, pass 2
relations:
- reviews: story:m3-sql-adapter
revision: 1
---
unit: story:m3-sql-adapter, working tree of impl/m3-sql-adapter (base c2897fe, implementation uncommitted, pass-2 correction)
verdict: NEEDS-CHANGE
cases: executed 43→50, red 3
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: 3 new paths under ~/.cache/thalamus-waves/20261015c/m3-sql-adapter/scratch/ (part 6)
needs-coordinator: no

`executed 43→50`: 43 is the package run with my 7 pass-2 cases deselected. The run below shows 30 outside `adv_sql.rs`, and 13 is the pass-1 count of `adv_sql.rs`. 50 is the run below.

## 1. `git --no-pager diff --stat` (worktree)

```
 crates/thalamus-sources/src/sql.rs | 892 ++++++++++++++++++++++++++++++++++++-
 1 file changed, 888 insertions(+), 4 deletions(-)
```

That line is the implementor's change, and I did not touch it. `git status --short` shows `?? crates/thalamus-sources/tests/adv_sql.rs`, which is mine and the only path I wrote. The other untracked paths are the implementor's: `tests/sql.rs` and `tests/fixtures/sql/`. Every path I changed is a test.

`adv_sql.rs` passes `cargo fmt -p thalamus-sources -- --check` (exit 0), clippy `--all-targets -D warnings` (exit 0) and `b10x-gates scan-text` ("1 file(s) carry no private rule"). The coordinator's rewrite of `adv_sql_ratio_unpaired_series_are_unknown_never_zero` is unchanged.

## 2. Cases added in pass 2 (`crates/thalamus-sources/tests/adv_sql.rs`)

| case | asserts | now |
|---|---|---|
| `adv_sql_ratio_empty_grouped_numerator_reads_every_series_unknown` | Extended D20 (main `docs/design/thalamus.md:422`). The numerator is a GROUP BY whose WHERE matches nothing: whole, 0 rows. The denominator is alpha=100, beta=50. No read may be `ok`. | **red** |
| `adv_sql_ratio_empty_ungrouped_numerator_is_not_zero` | A numerator with no dimension columns answers 0 rows; the denominator answers 556. The read may not be `ok`. | **red** |
| `adv_sql_ratio_empty_numerator_without_dimension_columns_is_not_zero` | The same, with `cluster` declared but returned by neither query. | **red** |
| `adv_sql_ratio_near_miss_numerator_keys_never_infer_zero` | A numerator key that is NULL, or padded by `char(n)` (`"alpha     "`), has no twin. The result is one unknown and nothing reads 0. | green |
| `adv_sql_ratio_cut_denominator_infers_no_zero` | A whole numerator with a denominator cut at 1,000 rows gives one unknown naming the cut. | green |
| `adv_sql_ratio_significant_digit_division_is_exact` | 8 hand-worked quotients, each listed below. | green |
| `adv_sql_count_groups_by_declared_dimensions_only` | An undeclared `reason` column splits nothing. The count is rows, not the `value` cell: alpha=2, beta=1. | green |

The 8 quotients are:
- `9.999999999999999999999/1` = `10` (round-up carry when the numerator is shortened)
- `1e300/1e300` = `1`
- `7/0.07` = `100`
- `1/0.0400` = `25`
- `15/556000` = `0.000026978417266187`
- `-2/3` = `-0.666666666666666667`
- `1/3e-18` = `333333333333333333.333333333333333333`
- `1.5e-05/3e-05` = `0.5`

Red output from the first run of the file with these cases (`cargo test --locked -p thalamus-sources --test adv_sql`, exit 101), verbatim:
```
thread 'adv_sql_ratio_empty_ungrouped_numerator_is_not_zero' (861464) panicked at crates/thalamus-sources/tests/adv_sql.rs:593:5:
an ungrouped numerator answered no row, and the ratio was read as Ok(CompleteReading { value: Decimal("0") }) with population Some(Population { n: 556, unit: Unit("{request}") })
thread 'adv_sql_ratio_empty_numerator_without_dimension_columns_is_not_zero' (861463) panicked at crates/thalamus-sources/tests/adv_sql.rs:617:5:
Ok(CompleteReading { value: Decimal("0") })
thread 'adv_sql_ratio_empty_grouped_numerator_reads_every_series_unknown' (861462) panicked at crates/thalamus-sources/tests/adv_sql.rs:647:9:
an empty numerator answer read Ok(CompleteReading { value: Decimal("0") }) for {"cluster": "alpha"}
test result: FAILED. 17 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```
All 13 pass-1 cases are green on the correction, including the six that were red in pass 1.

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
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adv_sql.rs (target/debug/deps/adv_sql-47b8b54cfd19e05c)
test result: FAILED. 17 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
error: test failed, to rerun pass `-p thalamus-sources --test adv_sql`
     Running tests/connectors_client.rs (target/debug/deps/connectors_client-db1ed0bd6892c612)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sql.rs (target/debug/deps/sql-edbb80ea08f6fac1)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```
The suite is red. That is the outcome this pass was after.

## 4. Findings (working tree of impl/m3-sql-adapter, pass-2 correction)

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| P2-F1 | `crates/thalamus-sources/src/sql.rs:504-529` (`unmatched` guard, then `None => Ok(Dec::zero())`) | A whole numerator with 0 rows over a non-empty denominator reads `ok` `0` with the denominator's population. That holds for each series of a grouped ratio (alpha, and beta), for a ratio with no dimensions, and for one whose declared dimensions neither query returns. Extended D20 requires unknown for every series. An empty key set passes the subset guard, so nothing stops it. Three red cases: `adv_sql.rs:593`, `:617` and `:647`. | A numerator whose WHERE matches nothing: a renamed status value, a renamed table behind a view, a roll-up row not yet written (`WHERE day = current_date`), or a row-level-security policy hiding every row. There is a second route for ratios with no grouping columns. An aggregate without GROUP BY always returns one row, so an empty answer there can never mean zero cases. That route is outside D20's stated "SQL `GROUP BY`" premise even before the extension. | NEEDS-CHANGE | introduced |
| P2-J1 | `sql.rs:402-406` | Judgement, not a red case. A whole `sql.count` answer with declared dimension columns and 0 rows reads one unknown ("no series to count"). The same zero rows without dimension columns read `ok` `0`. A healthy dimensioned count of cases with zero open cases therefore reads UNKNOWN (exit 3) on every pass. A series whose count drops to 0 stops getting a read; it does not read 0. This is an asymmetry, not a false 0. | Any dimensioned `count_of_cases` definition in its healthy state. | CONFIRMED | introduced |

The fix, named and not applied: in `ratio()`, when `numerator.is_empty() && !denominator.is_empty()`, return an unknown for every denominator series (extended D20) before inferring zeros. That one branch turns all three red cases green.

## 5. Attacked and not broken

- **D20 subset guard:** a NULL numerator key, a `char(n)`-padded key and an unmatched `gamma` each give one unknown, never 0. Duplicate numerator series read unknown.
- **D20 with a cut on either side:** one unknown naming the cut. A cut `sql.value`, a cut `sql.ratio` and a cut dimensioned `sql.count` each give one unknown. A cut dimensionless `sql.count` reads `partial` `lower` 1000.
- **Significant-digit division:** I checked the identity in the source comment by hand (p = 18 + sb − z − sa + 1, and floor(floor(A/10^k)/B) when p < 0) and found it correct. All 8 hand-worked quotients pass through the adapter. The implementor's unit cases (`1e+60/1e+50`, `1/1e+50`) would catch removal of the shortening step and of the zero-strip.
- **`sql.count` grouping:** counts per declared series; undeclared columns and `value` cells are ignored; NULL dimension cells count into the series with no key (implementor's case). A duplicate dimension column gives one unknown.
- **Pass-1 corrections:** duplicate `value` column, duplicate dimension column, mismatched dimension columns between numerator and denominator, and recordings cut at exactly 1,000 rows all hold.

## 6. Paths written outside the worktree (pass 2)

All under `~/.cache/thalamus-waves/20261015c/m3-sql-adapter/scratch/`:
- `adversary/p2-adv_sql-alone.log`
- `adversary/p2-suite.log`
- `adversary-pass-2.md` (this report)

The pass-1 paths are still there, including `adversary/mutant-tree/`, 152M with its own `target/`.

Disclosure: I read the cut fixtures' row counts with one inline `python3 -I -c` (it wrote nothing and is not in the repository). I should have used `jq`.

```findings
[
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 529, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a whole numerator answer with no rows over a non-empty denominator reads ok 0 for every series, against extended D20, which requires unknown"},
  {"file": "crates/thalamus-sources/src/sql.rs", "line": 402, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a whole dimensioned sql.count answer with no rows reads unknown while the same answer without dimension columns reads 0, so a healthy zero-case state grades UNKNOWN"}
]
```
