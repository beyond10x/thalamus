---
format: aep.planning-md/3
id: review-result:adversary-m1-prometheus-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m1-prometheus-adapter, pass 2
relations:
- reviews: story:m1-prometheus-adapter
revision: 1
---
unit: story:m1-prometheus-adapter, uncommitted working tree of impl/m1-prometheus-adapter (base c2897fe, D20 correction) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m1-prometheus-adapter
verdict: CONFIRMED (one D20 edge case needs a decision; one suite gap, now covered)
cases: executed 39→43, red 0 (red against scratch mutant P3, which the unit's suite misses, and red in a scratch-only probe of the empty-numerator case)
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/adversary/ (part 6), plus this report file
needs-coordinator: G1. Under D20 as written, an entirely empty but complete numerator answer reads 0 for every denominator series. Decide whether D20 means that, or whether an empty numerator over a non-empty denominator is unknown. Keep tests/adv2_prometheus.rs and fixtures/adv2_prometheus/ in the commit.

## 1. Diff stat

```
$ git --no-pager diff --stat
 crates/thalamus-sources/src/prometheus.rs | 573 +++++++++++++++++++++++++++++-
 1 file changed, 569 insertions(+), 4 deletions(-)
$ git status --short
 M crates/thalamus-sources/src/prometheus.rs                 (implementor's, untouched by me)
?? crates/thalamus-sources/tests/adv2_prometheus.rs          (mine, pass 2)
?? crates/thalamus-sources/tests/adv_prometheus.rs           (mine, pass 1; one case rewritten by the implementor per D20)
?? crates/thalamus-sources/tests/fixtures/adv2_prometheus/   (mine, pass 2: 3 recordings)
?? crates/thalamus-sources/tests/fixtures/adv_prometheus/    (mine, pass 1)
?? crates/thalamus-sources/tests/fixtures/prometheus/        (implementor's)
?? crates/thalamus-sources/tests/prometheus.rs               (implementor's)
```

In this pass I added only test files and fixtures. Mutants and the probe ran on a scratch copy.

## 2. Cases added

File: `crates/thalamus-sources/tests/adv2_prometheus.rs`. Each recording has the shape of the real Connectors `series.query` answer.

| case | asserts | now | mutant it kills |
|---|---|---|---|
| `adv2_a_cut_numerator_never_infers_a_zero` | numerator `complete: false` (omitted notes, no warning) lacks `beta`; complete denominator has `alpha`, `beta` → every read `unknown`, reason names numerator, no population | green | P3 `prometheus.rs:256`, cut taken from the denominator only |
| `adv2_a_numerator_finer_by_a_declared_key_never_infers_a_zero` | numerator `{cluster,namespace}` vs denominator `{cluster}` → no `0` for `beta`; every read `unknown` naming `shared-1` | green | P1 `:293` (also caught by the unit's own test) |
| `adv2_a_numerator_coarser_on_one_series_never_infers_a_zero` | one numerator series lacks `namespace` (L9 unset), denominator splits `alpha` by namespace → every read `unknown` naming `alpha` | green | P1 |
| `adv2_a_numerator_collision_does_not_block_a_zero_elsewhere` | numerator splits `alpha` by undeclared `pod` (collision), `beta` absent → `alpha` ×2 collision `unknown`, `beta` `Ok("0")`, `n=80` | green | none (pins D20's positive case beside a collision) |

The first run of the file alone gave `test result: ok. 4 passed; 0 failed` (EXIT=0).

Mutant probe on a scratch copy of the pass-2 `prometheus.rs`, verbatim:

```
P0-unmutated --test prometheus exit=0 :: test result: ok. 11 passed; 0 failed; ...
P0-unmutated --test adv_prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
P0-unmutated --test adv2_prometheus exit=0 :: test result: ok. 4 passed; 0 failed; ...
P0-unmutated --test probe_empty_numerator exit=101 :: test result: FAILED. 0 passed; 1 failed; ...
P1-unshaped-ignored --test prometheus exit=101 :: test result: FAILED. 10 passed; 1 failed; ...
P1-unshaped-ignored --test adv2_prometheus exit=101 :: test result: FAILED. 2 passed; 2 failed; ...
P2-zero-condition-dropped --test prometheus exit=0 :: test result: ok. 11 passed; 0 failed; ...
P2-zero-condition-dropped --test adv_prometheus exit=101 :: test result: FAILED. 9 passed; 1 failed; ...
P3-numerator-cut-ignored --test prometheus exit=0 :: test result: ok. 11 passed; 0 failed; ...
P3-numerator-cut-ignored --test adv_prometheus exit=0 :: test result: ok. 10 passed; 0 failed; ...
P3-numerator-cut-ignored --test adv2_prometheus exit=101 :: test result: FAILED. 3 passed; 1 failed; ...
```

P3 red output, verbatim: `panicked at crates/thalamus-sources/tests/adv2_prometheus.rs:112:22: no read of this pass is a reading, got Ok(CompleteReading { value: Decimal("0.02") })`.

Scratch-only probe `probe_empty_numerator.rs`, unmutated code. Numerator `sum by (cluster) (increase(adv2_renamed_failed_totl[6h]))` answers `series: []`, complete. Denominator answers alpha 100, beta 80. Output, verbatim:

```
PROBE dims={"cluster": "alpha"} reading=Ok(CompleteReading { value: Decimal("0") }) denominator=Some(Population { n: 100, unit: Unit("{request}") })
PROBE dims={"cluster": "beta"} reading=Ok(CompleteReading { value: Decimal("0") }) denominator=Some(Population { n: 80, unit: Unit("{request}") })
```

I left this probe out of the tree because D20 as written allows the behaviour (G1).

## 3. Suite run (after the cases exist)

`cargo test --locked -p thalamus-sources` exited 0. Test results per target:

| target | passed |
|---|---|
| lib unit tests | 2 |
| connectors-double | 0 |
| adv2_connectors_client | 2 |
| adv2_prometheus | 4 |
| adv_connectors_client | 6 |
| adv_prometheus | 10 |
| connectors_client | 8 |
| prometheus | 11 |
| doc-tests | 0 |

| other check | result |
|---|---|
| clippy `-D warnings` | exit 0 |
| `cargo fmt -- --check` | exit 0 |
| `b10x-gates scan-text` (adv2 files) | "4 file(s) carry no private rule" |

`39→43`: the "before" count is this run without `adv2_prometheus.rs`, whose 4 cases I excluded.

## 4. Findings

These cover the working tree named in the header.

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| G1 | crates/thalamus-sources/src/prometheus.rs:283 | A complete numerator answer with no series at all meets "every numerator series has a twin" vacuously, so every denominator series reads `Ok("0")` with its full population. That matches D20 as written, but nothing tells this apart from a broken numerator query. | CONFIRMED | introduced | All reachable: a renamed metric, a typo in a numerator label matcher, or a numerator exporter down for longer than the window (Prometheus answers an empty vector, complete). The legitimate case looks the same: a counter that has never been incremented on any series. Not fixable without a decision. Options: an empty numerator over a non-empty denominator reads `unknown`, or the definition declares that the numerator may be absent. |
| G2 | crates/thalamus-sources/src/prometheus.rs:256 | If the cut check ignores the numerator side, the unit's suite stays green. That mutant turns a cut numerator's missing series into `0` (D20 requires the whole pass `unknown`). `adv2_a_cut_numerator_never_infers_a_zero` kills it. | CONFIRMED | introduced | A numerator answer `complete: false`: a provider warning, a partial store response, or more than 32 notes. The real adapter sets it (connectors lib.rs:418-421). |
| G3 | crates/thalamus-sources/src/prometheus.rs:286 | Dropping the `needs_zero &&` condition (an unmatched numerator alone makes the whole pass unknown) passes the unit's own suite. Only pass-1 `adv_missing_denominator_series_is_unknown_without_population` kills it, so this holds only while that file stays in the commit. | CONFIRMED | introduced | The numerator has a series that the denominator lacks while every denominator series has a numerator, e.g. a new cluster that has failures but no total yet. |

## 5. Attacked and not broken

- **Label-set mismatches:** a numerator finer by a declared key, coarser on one series, or global over per-cluster always gives the whole pass `unknown`. None of them produced a false 0.
- **Collisions next to a needed 0:** the colliding dims read `unknown` and the others read 0, as D20 says.
- **Cut answers:** a cut on the denominator, or on the numerator via omitted notes, makes every read `unknown` with no population.
- **0 inferred over a zero denominator:** a missing numerator over denominator `0` reads `empty population` with `n=0`, not `Ok`.
- **Replaced recordings (pass-1 F8):** `0.25`, `1e-07`, `1e+21`, `-0.5` and an infos-only complete answer are real shapes. The time-mismatch and `samples_truncated` checks are unit tests on `answer()` (`src/prometheus.rs` `mod tests`, 2 passed).
- **Residue, not raised:** `twelve` (Prometheus never writes it) and `queue_depth_many` (`series_truncated` with 1 series; the real adapter truncates only past 500) are still in the unit's recording. Both are harmless.
- **Pass-1 F1** no longer applies after the D20 decision. **Pass-1 F2–F7** are still covered by `adv_prometheus.rs`: 10 green on the unmutated code.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015c/m1-prometheus-adapter/scratch/`:

- `adversary/mutant-tree/`: re-synced copy of the tree; its own `target/` is left in place. It also holds the scratch-only `crates/thalamus-sources/tests/probe_empty_numerator.rs`, and its `tests/fixtures/adv2_prometheus/` recording has 2 extra probe entries.
- `adversary/prometheus.rs.pass2`
- `adversary/answers2.full.json`
- `adversary/adv2-alone.log`, `suite2.log`, `mutants2.log`
- `adversary/mutant2-P{0..3}-*-{prometheus,adv_prometheus,adv2_prometheus,probe_empty_numerator}.log` (16 files)
- `adversary-pass-2.md` (this report)
- Created and deleted: `adversary/mutants2.sh`, `build.jq`, `answers2.src.json`.
- Inside the worktree's own target: `target/tmp/adv2_prometheus/`.

Pass-1 paths are listed in `adversary-pass-1.md` and are unchanged.

## 7. Findings block

```findings
- file: crates/thalamus-sources/src/prometheus.rs
  line: 283
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a complete but empty numerator answer satisfies the D20 twin rule vacuously and reads Ok("0") with full population for every denominator series, indistinguishable from a renamed metric or a typo in the numerator query
- file: crates/thalamus-sources/src/prometheus.rs
  line: 256
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: ignoring a cut numerator answer (turning its missing series into 0) survives the unit's suite; adv2_a_cut_numerator_never_infers_a_zero kills it
- file: crates/thalamus-sources/src/prometheus.rs
  line: 286
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: dropping the needs_zero condition survives the unit's suite and is killed only by the pass-1 adversary case adv_missing_denominator_series_is_unknown_without_population
```
