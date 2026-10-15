---
format: aep.planning-md/3
id: review-result:adversary-m3-loki-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m3-loki-adapter, pass 1
relations:
- reviews: story:m3-loki-adapter
revision: 1
---
unit: story:m3-loki-adapter, uncommitted working tree on impl/m3-loki-adapter over base c2897fe (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-loki-adapter)
verdict: red, strongest finding NEEDS-CHANGE (a ratio whose numerator series is absent reads unknown)
cases: executed 24→30, red 3
origin: introduced 9 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 tree, ~/.cache/thalamus-waves/20261015c/m3-loki-adapter/scratch/adversary/ (logs, a mutated source copy and its 147M target), plus this report
needs-coordinator: yes. Decide whether a numerator series that is absent from a complete answer, while its denominator series is present, reads 0 (my case asserts that) or unknown (the implementor's call)

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/loki.rs | 612 +++++++++++++++++++++++++++++++++++-
 1 file changed, 608 insertions(+), 4 deletions(-)
```

That is the implementor's tracked change only. My one path is untracked, so `git status --short` shows it:

```
 M crates/thalamus-sources/src/loki.rs
?? crates/thalamus-sources/tests/adv_loki.rs              <- mine, 392 lines, test file
?? crates/thalamus-sources/tests/fixtures/loki/           <- implementor's
?? crates/thalamus-sources/tests/loki.rs                  <- implementor's
```

No non-test path was touched. `rustfmt --check`, `cargo clippy -p thalamus-sources --all-targets -D warnings` and `b10x-gates scan-text` are clean on `adv_loki.rs`.

## 2. Cases added (`crates/thalamus-sources/tests/adv_loki.rs`)

Every answer is built the way Connectors v0.45.0 `adapters/loki/src/lib.rs` (`query_metric`, `query_range`) builds it: `complete` is false exactly when a cause is present. The red output below was captured from running each case alone (`cargo test --locked -p thalamus-sources --test adv_loki -- --exact <name>`), before any suite run.

| case | asserts | now |
|---|---|---|
| `adv_loki_ratio_cluster_without_errors_is_zero_not_unknown` (:194) | numerator has `alpha=15` only; denominator has `alpha=556`, `gamma=8`. Expects `gamma` to read `ok("0")` with n=8 | **red** |
| `adv_loki_value_negative_zero_is_not_a_second_zero` (:231) | Loki value `-0` reads `ok("0")`, as `render` does for a ratio | **red** |
| `adv_loki_ratio_pairs_only_series_with_the_same_labels` (:374) | numerator `{cluster:alpha,app:api}`, denominator `{cluster:alpha,app:web}`: not divided | **red** |
| `adv_loki_value_series_colliding_on_declared_dims_are_unknown` (:255) | two series that differ only in an undeclared label produce one unknown read saying "2 series" | green, kills mutant M1 |
| `adv_loki_metric_series_limit_is_not_reported_complete` (:289) | 500 series with `series_limit`, `complete:false`: no read is `ok` | green, kills mutant M2 |
| `adv_loki_count_stream_limit_below_line_cap_is_partial` (:322) | 500 lines with `stream_limit`, `complete:false`: `partial`, `lower`, 500 | green, kills mutant M3 |

Red output of each red case, run alone:

```
---- adv_loki_ratio_cluster_without_errors_is_zero_not_unknown stdout ----
panicked at crates/thalamus-sources/tests/adv_loki.rs:214:5:
assertion `left == right` failed: gamma logged 8 lines and no error: Read { dims: {"cluster": "gamma"}, reading: Unknown(UnreadReading { reason: "no numerator series for {\"cluster\":\"gamma\"}" }), ... denominator: None, ... }
  left: Unknown(UnreadReading { reason: "no numerator series for {\"cluster\":\"gamma\"}" })
 right: Ok(CompleteReading { value: Decimal("0") })
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out

---- adv_loki_value_negative_zero_is_not_a_second_zero stdout ----
panicked at crates/thalamus-sources/tests/adv_loki.rs:248:5:
  left: Ok(CompleteReading { value: Decimal("-0") })
 right: Ok(CompleteReading { value: Decimal("0") })
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out

---- adv_loki_ratio_pairs_only_series_with_the_same_labels stdout ----
panicked at crates/thalamus-sources/tests/adv_loki.rs:386:5:
series of different label sets were divided: [Read { dims: {"cluster": "alpha"}, reading: Ok(CompleteReading { value: Decimal("0.02697841726618705") }), ... denominator: Some(Population { n: 556, unit: Unit("{line}") }), ... }]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out
```

The three green cases each passed alone (`exit=0`). I then ran a mutation probe on a copy under scratch (`mutant/`); the tree under attack was never edited. Mutants (`scratch/adversary/mutants.diff`):

```
263c263  M3  let cut = (!complete && false) || lines.len() as u64 >= LINE_LIMIT;
312c312  M2  match answer["complete"].as_bool().map(|_| true) {
378c378  M1  1.. => Ok(texts.remove(0)),
```

Against the mutated copy, `cargo test --locked --no-fail-fast -p thalamus-sources --test loki --test adv_loki` gave:

```
     Running tests/adv_loki.rs
test result: FAILED. 0 passed; 6 failed
     Running tests/loki.rs
test loki_nan_value_is_unknown_never_zero ... ok
test loki_refused_read_is_unknown_with_reason ... ok
test loki_metric_series_become_samples_with_source_labels_only ... ok
test loki_ratio_is_decimal_with_unit_1 ... ok
test loki_line_limit_reached_is_partial_lower_bound ... ok
test result: ok. 5 passed; 0 failed
```

`cargo test -p thalamus-sources --lib loki::` on the copy: `test result: ok. 3 passed; 0 failed`. The unit's 5 known-answer tests and 3 unit tests stay green with all three mutants in place, and M1, M2 and M3 each turn one of my guards red (:280, :305, :360).

## 3. Suite run, in the worktree, after the cases existed

`cargo test --locked --no-fail-fast -p thalamus-sources` gave **exit 101**. Full log: `scratch/adversary/suite.log`.

```
unittests src/lib.rs                 test result: ok. 3 passed; 0 failed
unittests connectors-double          test result: ok. 0 passed; 0 failed
tests/adv2_connectors_client.rs      test result: ok. 2 passed; 0 failed
tests/adv_connectors_client.rs       test result: ok. 6 passed; 0 failed
tests/adv_loki.rs                    test result: FAILED. 3 passed; 3 failed
    adv_loki_ratio_cluster_without_errors_is_zero_not_unknown
    adv_loki_ratio_pairs_only_series_with_the_same_labels
    adv_loki_value_negative_zero_is_not_a_second_zero
tests/connectors_client.rs           test result: ok. 8 passed; 0 failed
tests/loki.rs                        test result: ok. 5 passed; 0 failed
doc-tests                            test result: ok. 0 passed; 0 failed
```

`executed 24→30`: 24 is this same run with `tests/adv_loki.rs` excluded (3+0+2+6+8+5); 30 includes it. The suite is red, which is the outcome this pass was after.

## 4. Findings

Every finding covers the uncommitted working tree over c2897fe. At c2897fe `src/loki.rs` is a stub whose `adapter()` returns `None` (`git show c2897fe:crates/thalamus-sources/src/loki.rs`), so every origin is `introduced`.

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | `src/loki.rs:166` | `adv_loki.rs:214` red: a denominator series without a numerator series reads `unknown` "no numerator series" and gets no population | **Inferred from LogQL semantics, not observed against a running Loki:** `count_over_time` emits no point for a stream with no entry in range, and `sum by` of nothing is nothing. So every dims that logged lines and no error has no numerator series in a complete answer, which Connectors § 11.1 calls exhaustive. That is the design's own failure-ratio signal (§ 2.1). Each healthy dims then reads UNKNOWN, and UNKNOWN ranks above OK in worst-of (§ 2.3). An author can work around it in LogQL (`… or 0 * <denominator>`) | NEEDS-CHANGE | introduced |
| F2 | `src/loki.rs:314` | no case in `tests/loki.rs` gives an incomplete metric answer. Mutant M2 (ignore `complete`) stays green under the unit's suite and fails `adv_loki.rs:305`. Under M2, 500 of an unknown number of series read `ok` | any instant query returning more than 500 series (Connectors `query_metric` adds `series_limit`) | CONFIRMED | introduced |
| F3 | `src/loki.rs:263` | the suite cuts counts only at the 1,000-line cap. Mutant M3 (drop `!complete`) stays green and fails `adv_loki.rs:360`, which means a partial count reported as complete | `logs.query_range` over more than 500 streams (for example per-pod streams of `{app="api"}`): Connectors keeps 500 groups, adds `stream_limit` and sets `complete:false` below the line cap; `response_bytes` (4 MiB) does the same | CONFIRMED | introduced |
| F4 | `src/loki.rs:378` | no answer in the suite collides on declared dims. Mutant M1 (take the first series) stays green and fails `adv_loki.rs:280`, so a colliding series is read as one series' value | any query whose `by (…)` keeps a label the definition does not declare | CONFIRMED | introduced |
| F5 | `src/loki.rs:122` | `adv_loki.rs:248` red: value `-0` is stored as `Decimal("-0")`, while a ratio is rendered "never `-0`" (:541). `Decimal` compares by rendering | Go `FormatFloat(-0,'f',-1,64)` prints `-0`, but it needs a LogQL expression that evaluates to negative zero. No realistic definition found | INFEASIBLE | introduced |
| F6 | `src/loki.rs:341` | `adv_loki.rs:386` red: series are paired after undeclared labels are dropped, so `{app:api}` errors are divided by `{app:web}` lines. LogQL's `/` matches identical label sets only. Collisions (F4) and disjoint dims read unknown correctly | only numerator and denominator queries whose undeclared label values differ. A realistic `\| json \| level="error"` numerator over a `by (cluster)` denominator is paired correctly by this rule. No realistic definition found | INFEASIBLE | introduced |
| F7 | `tests/loki.rs:324` | the ratio fixture returns `beta` numerator `0` and denominator `0` for plain `count_over_time` queries; the nan fixture (:377) returns `0x10` | **inferred, not observed:** these queries return no `beta` series rather than `0` (see F1), and Loki's `'f'` float format never emits hex. So the 0/0 known-answer pins a shape the real source does not return, and the shape it does return (F1) is untested. The at-cap `complete:true` answer (:429) is also one Connectors never builds (exact cap sets `provider_limit`), but it is labelled defensive | CONFIRMED | introduced |
| F8 | `src/loki.rs:202` | population `n` is `floor(denominator)`, so `0.5` gives `n=0` (`floor_is_the_whole_part` asserts `0.4`→`0`) and the ratio still reads `ok`. No case added | a `loki.ratio` with a `denominator` spec whose denominator query is a `rate()` or `bytes_rate()`: n < min_n on every pass, so it is always `N/A` | CONFIRMED | introduced |
| F9 | `src/loki.rs:358` | every read's `window` is `[end − definition.window, end]`, whatever range selector the LogQL carries. The unit's own `VALUE_QUERY` counts `[5m]` under window `PT6H`, and `loki.rs:205` asserts the 6h start | any definition whose LogQL range differs from its `window`. Baseline eligibility (§ 3.1, `age_s`) then reasons over a span that was not measured. No parser exists to check it (Connectors § 11.3), so this is a validation or documentation fix | INFEASIBLE | introduced |

Fixes named, not applied:
- **F1:** in `ratio()`, read a dims present in the denominator and absent from a complete numerator answer as numerator `0`.
- **F2, F3, F4:** keep my three guards.
- **F5:** normalise a value through `parse` and `render`.
- **F6:** pair on full label sets, or document the projection.
- **F7:** replace the beta 0/0 recording with the absent-series shape.
- **F8:** set no population when the denominator is not an integer.

## 5. Attacked and not broken

- Undeclared labels in `dims`: dropped, both for `query_metric` series and `query_range` streams. Nothing derives a tenant: the binding sends no tenant header, and dims come only from returned labels.
- `NaN`, `+Inf`, `-Inf`, `1e3`, `+1`, `.5`, `5.`, `01` and empty text: all read unknown, never 0.
- Incomplete metric answer (`complete:false`): the whole pass reads unknown, never `ok` (guarded by my case).
- Count at the cap with `complete:true`: `partial lower`. Count with `complete:false` below the cap: `partial lower` (guarded by my case).
- Ratio 0/0 and n/0: unknown. Negative denominator: unknown. Disjoint dims on the two sides: both unknown. A numerator or denominator answer that is refused or incomplete: the whole pass reads unknown.
- Exact division: 1/3, 2/3, −2/3, 1/0.001 and 15/556 are exact to 18 places and round half away from zero.
- Connectors refusal: one unknown read naming code `rate_limited` and stage `dispatch`, with no dims.
- Request shapes: `query_metric` sends `{query, time_unix_ns}` and `query_range` sends `{query, start_unix_ns, end_unix_ns, limit:1000}`. Both are accepted by Connectors' `deny_unknown_fields` structs (`Metric`, `Range`), and limit 1000 is that binding's maximum (§ 11.3).

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015c/m3-loki-adapter/scratch/`:

- `adversary/build.log`
- `adversary/alone-<case>.log`, one for each of the 6 cases
- `adversary/suite.log`
- `adversary/mutants.diff`
- `adversary/mutant-run.log`
- `adversary/mutant-lib.log`
- `adversary/mutant/`: an rsync copy of the worktree without `target/` and `.git`, with three mutants in `crates/thalamus-sources/src/loki.rs`
- `adversary/mutant/target/`: its build directory, 147M. I did not delete it
- `adversary-pass-1.md`: this report

Inside the worktree, besides `tests/adv_loki.rs`, cargo wrote build output to `target/` (including `target/tmp/adv_loki/`).

## 7. Findings block

```findings
- file: crates/thalamus-sources/src/loki.rs
  line: 166
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a dims present in the denominator and absent from a complete numerator answer (LogQL count_over_time emits nothing for zero errors) reads unknown instead of ratio 0, so every healthy dims of a Loki failure ratio grades UNKNOWN
- file: crates/thalamus-sources/src/loki.rs
  line: 314
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: ignoring complete:false on logs.query_metric survives the unit's suite, so a series_limit-cut answer could be reported complete without a test failing
- file: crates/thalamus-sources/src/loki.rs
  line: 263
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping the !complete term of the count cut survives the unit's suite, so a stream_limit or response_bytes answer below 1000 lines could be counted complete without a test failing
- file: crates/thalamus-sources/src/loki.rs
  line: 378
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: taking the first of several series that collide on declared dims survives the unit's suite
- file: crates/thalamus-sources/src/loki.rs
  line: 122
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a value of -0 is stored as Decimal("-0") while ratios render zero as "0", giving one number two renderings
- file: crates/thalamus-sources/src/loki.rs
  line: 341
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: ratio sides are paired after undeclared labels are dropped, so series with different label sets are divided where LogQL would not match them
- file: crates/thalamus-sources/tests/loki.rs
  line: 324
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the 0/0 and 0x10 recordings are answers the queried Loki would not return, while the absent-numerator shape it does return is untested
- file: crates/thalamus-sources/src/loki.rs
  line: 202
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the population is the floor of the denominator, so a rate denominator such as 0.5 gives n=0 and the ratio is always N/A under min_n
- file: crates/thalamus-sources/src/loki.rs
  line: 358
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: each read claims the definition window regardless of the LogQL range selector, and the unit's own test pairs a [5m] query with a 6h window
```
