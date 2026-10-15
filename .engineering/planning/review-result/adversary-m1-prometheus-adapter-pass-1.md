---
format: aep.planning-md/3
id: review-result:adversary-m1-prometheus-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m1-prometheus-adapter, pass 1
relations:
- reviews: story:m1-prometheus-adapter
revision: 1
---
unit: story:m1-prometheus-adapter, uncommitted working tree of impl/m1-prometheus-adapter (base c2897fe) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m1-prometheus-adapter
verdict: CONFIRMED (suite gaps only; no defect in the implementation found)
cases: executed 26→36, red 0 (red 7 of 7 against scratch mutants, which the unit's own suite misses)
origin: introduced 8 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory tree, ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/ (part 6), plus this report file
needs-coordinator: keep crates/thalamus-sources/tests/adv_prometheus.rs and tests/fixtures/adv_prometheus/ in the unit's commit; they hold the guards the unit's suite lacks

## 1. Diff stat

```
$ git --no-pager diff --stat
 crates/thalamus-sources/src/prometheus.rs | 504 +++++++++++++++++++++++++++++-
 1 file changed, 500 insertions(+), 4 deletions(-)
$ git status --short
 M crates/thalamus-sources/src/prometheus.rs                 (implementor's, untouched by me)
?? crates/thalamus-sources/tests/adv_prometheus.rs           (mine)
?? crates/thalamus-sources/tests/fixtures/adv_prometheus/    (mine: 3 recordings)
?? crates/thalamus-sources/tests/fixtures/prometheus/        (implementor's, untouched)
?? crates/thalamus-sources/tests/prometheus.rs               (implementor's, untouched)
```

All my paths are test files or fixtures. I mutated only a scratch copy, never the tree.

## 2. Cases added

File: `crates/thalamus-sources/tests/adv_prometheus.rs`. Fixtures: `tests/fixtures/adv_prometheus/`. Each answer is in the shape `~/beyond10x/connectors/adapters/prometheus/src/lib.rs` builds in `query` (:404-432) and `series` (:288-347).

| case | asserts | now | mutant it kills (scratch copy) |
|---|---|---|---|
| `adv_missing_numerator_series_is_unknown_never_zero` | numerator lacks `beta`, denominator 80 → `unknown`, `n=80`, never `Ok("0")` | green | M1 `prometheus.rs:340` missing numerator → `Ok("0")` |
| `adv_missing_denominator_series_is_unknown_without_population` | denominator lacks `beta` → `unknown`, `denominator: None` | green | M7 `:309` claims `n=0` |
| `adv_negative_and_zero_denominators` | `-40` → `unknown` and no population; `0/-0` → `empty population`, `n=0`; `3/0` → not `empty population`, `n=0` | green | M2 `:319` guard dropped (→ `Ok("-0.025")`); M3 `:351` x/0 → `empty population` |
| `adv_empty_answer_is_one_unknown_never_no_read` | `series: []` → exactly 1 `unknown` read, no dims | green | M4 `:200` → `Ok(vec![])` |
| `adv_prometheus_float_text_becomes_a_wire_decimal_of_the_same_number` | `-0`, `1e-07`, `-1e-07`, `1e+21`, `5e-324`, `1.7976931348623157e+308`, `0.30000000000000004`, `123456789012345680000` → `decimal_at` grammar, same f64 bits, `raw` kept | green | M6 `:497` writes `-0` |
| `adv_ratio_of_tiny_and_inexact_floats_is_a_wire_decimal` | `1e-07/3` and `0.1/0.30000000000000004` → grammar decimal equal to the f64 quotient | green | none |
| `adv_scalar_answer_is_one_read_without_dims` | `result_type: scalar` → 1 read, dims `{}`, `"4"` | green | none |
| `adv_string_and_matrix_answers_are_unknown` | `string` and 1-sample `matrix` → `unknown` | green | M5 `:394` admits `matrix` |
| `adv_ratio_joins_on_dims_and_leaks_no_label` | `code` is dropped and the series join (dims exactly `{cluster}`); a global numerator over a per-cluster denominator gives no reading (L9) | green | M1 as well |
| `adv_omitted_notes_without_warnings_make_the_read_unknown` | `complete: false` with `warnings: []` (Connectors' omitted notes, lib.rs:418-421) → `unknown` | green | none |

First run of the file alone (before any suite run), verbatim tail:

```
$ cargo test --locked -p thalamus-sources --test adv_prometheus
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
EXIT=0
```

Mutant probe on a scratch copy of the tree, one mutant at a time (perl substitution on the copy's `prometheus.rs`), verbatim:

```
M1-missing-numerator-is-0 --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M1-missing-numerator-is-0 --test adv_prometheus exit=101 :: test result: FAILED. 8 passed; 2 failed; ...
M2-negative-denominator-admitted --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M2-negative-denominator-admitted --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
M3-x-over-0-is-empty-population --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M3-x-over-0-is-empty-population --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
M4-empty-answer-is-no-read --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M4-empty-answer-is-no-read --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
M5-matrix-admitted --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M5-matrix-admitted --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
M6-negative-zero-kept --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M6-negative-zero-kept --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
M7-missing-denominator-claims-n-0 --test prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
M7-missing-denominator-claims-n-0 --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
```

M1 red output, verbatim: `panicked at crates/thalamus-sources/tests/adv_prometheus.rs:149:18: expected an unknown reading, got Ok(CompleteReading { value: Decimal("0") })`. M2: `got Ok(CompleteReading { value: Decimal("-0.025") })`. M4: `an empty answer is one unknown read: [] left: 0 right: 1`. M6: `negzero: zero is written 0, never -0 left: "-0" right: "0"`. M7: `left: Some(Population { n: 0, unit: Unit("{request}") }) right: None`.

## 3. Suite run (after the cases exist)

```
$ cargo test --locked -p thalamus-sources
test result: ok. 0 passed; ... (lib unittests)
test result: ok. 0 passed; ... (connectors-double)
test result: ok. 2 passed; ... (adv2_connectors_client)
test result: ok. 6 passed; ... (adv_connectors_client)
test result: ok. 10 passed; ... (adv_prometheus)
test result: ok. 8 passed; ... (connectors_client)
test result: ok. 10 passed; ... (prometheus)
test result: ok. 0 passed; ... (doc-tests)
EXIT=0
$ cargo clippy --locked -p thalamus-sources --all-targets -- -D warnings   → exit 0
$ cargo fmt -p thalamus-sources -- --check                                → exit 0
$ b10x-gates scan-text … adv_prometheus.rs fixtures/adv_prometheus/*      → "4 file(s) carry no private rule"
```

`executed 26→36`: 26 is this run without `adv_prometheus.rs` (its 10 excluded). The suite is green, so I found no defect in the code. Everything found is a gap in the unit's suite.

## 4. Findings

Covers the working tree named in the header.

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| F1 | crates/thalamus-sources/src/prometheus.rs:340 | Nothing in the unit's suite drives "missing numerator → unknown". Changing it to `Ok("0")` (the L5 failure) leaves all 10 green. | CONFIRMED | introduced | Common in real use: a labelled counter child exists only after its first increment, so a cluster with no failures has no numerator series under `sum by (cluster)(increase(…_failed_total[…]))`. |
| F2 | crates/thalamus-sources/src/prometheus.rs:319 | The negative-denominator guard is untested. Without it a ratio reads `Ok(-0.025)` with `n = -40`. | CONFIRMED | introduced | A denominator over `delta()` or any gauge expression. `increase()` and `rate()` are never negative. |
| F3 | crates/thalamus-sources/src/prometheus.rs:200 | "Empty answer → one unknown" is untested. Returning no reads instead (the signal vanishes from the pass) leaves the suite green. | CONFIRMED | introduced | A selector that matches nothing at the instant (series gone stale, a typo in a label value). Real adapter answers `series: []`. |
| F4 | crates/thalamus-sources/src/prometheus.rs:351 | "x/0 is not the empty population" is untested. Only `0/0` is driven. | CONFIRMED | introduced | `sum(increase(failed))` > 0 while the total's counter reset or lags. Grading result is N/A either way (n=0), only the reason differs. |
| F5 | crates/thalamus-sources/src/prometheus.rs:394 | Refusing `matrix`/`string` result types is untested. Admitting `matrix` makes a 1-sample range selector an `ok` reading. | CONFIRMED | introduced | A definition whose `query.value` is a range selector (`x[5m]`). Real adapter returns `result_type: matrix` for it (lib.rs:295). |
| F6 | crates/thalamus-sources/src/prometheus.rs:497 | The "never `-0`" branch of `render` is untested. The wire grammar (`decimal_at`) admits `-0`, so only an explicit assertion catches it. | CONFIRMED | introduced | Prometheus writes `-0` for a negative-zero result (e.g. `-1 * 0`). |
| F7 | crates/thalamus-sources/src/prometheus.rs:309 | "Missing denominator → no population" is untested. Claiming `n = 0` there would turn an unread reading into N/A (`story:m1-verdict-grading` rule 1). | CONFIRMED | introduced | The numerator has a series for dims the denominator query lacks (different `by` lists or label drift). |
| F8 | crates/thalamus-sources/tests/fixtures/prometheus/operations+invoke+a=metrics+c=metrics-main+o=series.query.json:116 | Three recordings are states the real adapter never returns: `samples_truncated: true` on an instant vector (:116, the adapter keeps exactly 1 sample and `max_samples ≥ 1`, connectors lib.rs:304, :369), `time_unix_s` ≠ asked (:258, the adapter echoes the input `time`, lib.rs:429), and `1e+03`/`0.250` (:214). The real exponent forms were untested until F6's case. | CONFIRMED | introduced | Nothing reaches these three states. The branches they drive are defensive and harmless. Prometheus writes `e` form only below 1e-6 and from 1e21 up (my recollection of its `jsonutil.MarshalFloat`, not verified locally). |

## 5. Attacked and not broken

- NaN, `+Inf`, `-Inf` as a value, numerator or denominator, and overflow of `n/d`: always `unknown`, never 0 or `ok`.
- Float to decimal: Rust `Display` writes the shortest round-trip digits with no exponent, so `1e+21`, `1e-07`, `5e-324` and `f64::MAX` all come out in the wire grammar with the same bits. `0.1+0.2`-style quotients equal the quotient Prometheus itself would answer for `a / b`.
- Label-set matching: the join is on narrowed dims. An undeclared label (`code`, `pod`, `__name__`) never reaches `dims`. A global numerator never joins a per-cluster denominator (L9). Duplicates on one side are collisions.
- `complete: false` with no warnings (Connectors' omitted notes) and scalar answers are handled.
- Unit check: kind `ratio` with unit other than `1` on `promql.value` is outside this unit. `story:m1-instance-registry` validates "kind `ratio` has unit `1`".
- Underflow to `0` (`5e-324 / 3`) is IEEE rounding, and no real population produces it.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/mutant-tree/ (rsync copy of the tree without target/.git/website/.engineering; contains its own `target/`, 138M; left in place, not deleted by hand)
- ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/prometheus.rs.orig
- ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/adv-alone.log, suite.log, mutants.log
- ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/mutant-M{1..7}-*-{prometheus,adv_prometheus}.log (14 files)
- ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary-pass-1.md (this report)
- Created and deleted: scratch/adversary/mutants.sh, build.jq, answers.src.json (probes).
- Inside the worktree's own target/: `target/tmp/adv_prometheus/` (test worlds, created by the cases).

## 7. Findings block

```findings
- file: crates/thalamus-sources/src/prometheus.rs
  line: 340
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: missing numerator series returning Ok("0") (the L5 failure) survives the unit's suite; adv_missing_numerator_series_is_unknown_never_zero kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 319
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping the negative-denominator guard (ratio Ok(-0.025), n=-40) survives the unit's suite; adv_negative_and_zero_denominators kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 200
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: an empty answer yielding no read instead of one unknown survives the unit's suite; adv_empty_answer_is_one_unknown_never_no_read kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 351
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: x/0 reported as "empty population" survives the unit's suite; adv_negative_and_zero_denominators kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 394
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: admitting a matrix result as an instant value survives the unit's suite; adv_string_and_matrix_answers_are_unknown kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 497
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: writing -0 instead of 0 survives the unit's suite and the wire grammar; adv_prometheus_float_text_becomes_a_wire_decimal_of_the_same_number kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 309
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a missing denominator claiming n=0 (turning unknown into N/A) survives the unit's suite; adv_missing_denominator_series_is_unknown_without_population kills it
- file: crates/thalamus-sources/tests/fixtures/prometheus/operations+invoke+a=metrics+c=metrics-main+o=series.query.json
  line: 116
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: recordings for samples_truncated on an instant vector, a mismatched time_unix_s and 1e+03/0.250 are answers the real Connectors adapter never returns, while its real exponent forms went untested
```
