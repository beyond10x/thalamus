---
format: aep.planning-md/3
id: review-result:adversary-m3-loki-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m3-loki-adapter, pass 2
relations:
- reviews: story:m3-loki-adapter
revision: 1
---
unit: story:m3-loki-adapter, uncommitted working tree on impl/m3-loki-adapter over base c2897fe (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-loki-adapter), after the implementor's pass-1 corrections
verdict: red, strongest finding NEEDS-CHANGE (the absent-means-0 rule reads real errors as 0 when the two sides' label sets do not line up)
cases: executed 32→37, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/thalamus-waves/20261015c/m3-loki-adapter/scratch/adversary/pass2/ (logs, mutants.diff), the reused mutated copy ~/.cache/thalamus-waves/20261015c/m3-loki-adapter/scratch/adversary/mutant/ (re-synced, target 159M), and this report
needs-coordinator: no. The fix below keeps the coordinator's F1 decision and narrows only where it applies

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/loki.rs | 699 +++++++++++++++++++++++++++++++++++-
 1 file changed, 695 insertions(+), 4 deletions(-)
```

That is the implementor's tracked change only. My paths are untracked (`git status --short`):

```
?? crates/thalamus-sources/tests/adv2_loki.rs   <- mine, new this pass, test file
?? crates/thalamus-sources/tests/adv_loki.rs    <- mine, pass 1, unchanged (byte-identical to its pass-1 copy)
```

No non-test path was touched. `rustfmt --check`, `cargo clippy -p thalamus-sources --all-targets -D warnings` and `b10x-gates scan-text` are clean on both files.

## 2. Cases added (`crates/thalamus-sources/tests/adv2_loki.rs`)

Each case was run alone (`cargo test --locked -p thalamus-sources --test adv2_loki -- --exact <name>`) before the suite ran. Answers are shaped as Connectors v0.45.0 `query_metric` builds them.

| case | asserts | now |
|---|---|---|
| `adv2_loki_ratio_numerator_without_by_is_not_zero_per_cluster` (:201) | numerator `sum(...)` returns `{}`=15; denominator `by (cluster)` returns alpha=556 and gamma=8. No read is `ok("0")` | **red** |
| `adv2_loki_ratio_numerator_coarser_than_denominator_is_not_zero` (:228) | numerator `{cluster:alpha}`=15; denominator `{alpha,shop}`=500 and `{alpha,web}`=56; declared `[cluster, namespace]`. No read is `ok("0")` | **red** |
| `adv2_loki_ratio_incomplete_numerator_is_not_read_as_absent` (:256) | numerator cut at 500 series (`series_limit`); denominator c000 and c900. Every read is unknown | green, catches mutant M4 |
| `adv2_loki_ratio_two_pairs_on_one_dims_are_one_unknown` (:289) | pairs api 15/100 and web absent/200 both project onto `{cluster:alpha}`: one unknown saying "2 series", no population | green, catches mutant M6 |
| `adv2_loki_value_long_decimals_render_once` (:316) | a 309-digit integer and the 323-zero subnormal stay unchanged; `7.5` plus 40 zeros becomes `7.5`; `-0.` plus 40 zeros becomes `0`; `-<subnormal>` keeps its sign; `100.000` becomes `100`; `raw` keeps the provider text | green |

Red output, each case run alone:

```
thread 'adv2_loki_ratio_numerator_without_by_is_not_zero_per_cluster' panicked at crates/thalamus-sources/tests/adv2_loki.rs:217:5:
15 errors were read as 0 per cluster: [({}, Unknown(UnreadReading { reason: "no denominator series for {}" })), ({"cluster": "alpha"}, Ok(CompleteReading { value: Decimal("0") })), ({"cluster": "gamma"}, Ok(CompleteReading { value: Decimal("0") }))]

thread 'adv2_loki_ratio_numerator_coarser_than_denominator_is_not_zero' panicked at crates/thalamus-sources/tests/adv2_loki.rs:244:5:
alpha's 15 errors were read as 0 in each namespace: [({"cluster": "alpha"}, Unknown(UnreadReading { reason: "no denominator series for {\"cluster\":\"alpha\"}" })), ({"cluster": "alpha", "namespace": "shop"}, Ok(CompleteReading { value: Decimal("0") })), ({"cluster": "alpha", "namespace": "web"}, Ok(CompleteReading { value: Decimal("0") }))]
```

### Mutation probe

The probe ran on the scratch copy, re-synced from the tree; the tree under attack was not edited. Mutants (`pass2/mutants.diff`):

```
161  M4  Err(why) => return Ok(Err(format!("numerator: {why}"))),  ->  Err(_) => BTreeMap::new(),
197  M6  if pairs.len() > 1 {                                       ->  if pairs.len() > 2 {
```

`cargo test --locked --no-fail-fast -p thalamus-sources --test loki --test adv_loki --test adv2_loki` against the copy:
- `tests/loki.rs`: ok, 5 passed.
- lib `loki::`: ok, 5 passed.
- `tests/adv_loki.rs` (pass 1): ok, 6 passed. Pass 1's pairing case orders its unknown pair first, so it misses M6.
- `tests/adv2_loki.rs`: 4 failed, including:
  - `:275` `a cut numerator was read: [({"cluster":"c000"}, Ok("0")), ({"cluster":"c900"}, Ok("0"))]` (M4)
  - `:308` `two pairs on one dims became Ok(CompleteReading { value: Decimal("0.15") })` (M6)

Under M4, a cut numerator becomes "every cluster has 0 errors", and the unit's whole suite plus pass 1 stay green.

## 3. Suite run, in the worktree, after the cases existed

`cargo test --locked --no-fail-fast -p thalamus-sources` gave **exit 101**. Full log: `pass2/suite.log`.

```
unittests src/lib.rs                 test result: ok. 5 passed; 0 failed
unittests connectors-double          test result: ok. 0 passed; 0 failed
tests/adv2_connectors_client.rs      test result: ok. 2 passed; 0 failed
tests/adv2_loki.rs                   test result: FAILED. 3 passed; 2 failed
    adv2_loki_ratio_numerator_coarser_than_denominator_is_not_zero
    adv2_loki_ratio_numerator_without_by_is_not_zero_per_cluster
tests/adv_connectors_client.rs       test result: ok. 6 passed; 0 failed
tests/adv_loki.rs                    test result: ok. 6 passed; 0 failed
tests/connectors_client.rs           test result: ok. 8 passed; 0 failed
tests/loki.rs                        test result: ok. 5 passed; 0 failed
doc-tests                            test result: ok. 0 passed; 0 failed
```

`32→37`: 32 is this run with `adv2_loki.rs` excluded (5+0+2+6+6+8+5); 37 includes it. All six pass-1 cases are green, as the implementor reported.

## 4. Findings

All cover the working tree over c2897fe. `loki.rs` is a stub there, so every origin is `introduced`.

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| G1 | `src/loki.rs:180` | `adv2_loki.rs:217` and `:244` red. A numerator series whose label set has no denominator twin reads unknown, and every denominator label set then reads numerator 0 (`ok("0")`). That is 15 errors graded as 0 per cluster or namespace: unknown became 0 (L5) | any `loki.ratio` whose numerator groups by fewer labels than its denominator: a missing `by`, `by (cluster)` against `by (cluster, namespace)`, or a numerator pipeline that drops a label. LogQL's own `/` returns nothing for such a pair. Pass-1 code read it unknown; the F1 correction made it false-green | NEEDS-CHANGE | introduced |
| G2 | `src/loki.rs:161` | the unit's suite has no incomplete numerator. Mutant M4 (failed numerator treated as empty) stays green and reads every cluster 0; `adv2_loki.rs:275` catches it | any numerator over 500 series (`series_limit`); the guard now carries the absent-means-0 rule | CONFIRMED | introduced |
| G3 | `src/loki.rs:197` | the unit's suite has no ratio collision, and pass 1's pairing case misses M6 (first pair kept). `adv2_loki.rs:308` catches it | any ratio whose label sets differ only in an undeclared label | CONFIRMED | introduced |
| G4 | `src/loki.rs:243` | read: an empty complete `query_range` answer gives one read at dims `{}` = 0, even when the definition declares dimensions. A non-empty answer gives no read at all for a quiet cluster. So a quiet window writes 0 to the bare-id series (D10), not to the per-cluster series it never otherwise feeds. No case added; the right shape is a design call | every `loki.count` with declared dimensions in a window with no matching line. The 0 itself is right: a validated empty answer establishes exhaustion (Connectors § 4.3) | CONFIRMED | introduced |

Fix for G1, not applied: apply absent-means-0 only when every numerator label set has a denominator twin (numerator keys ⊆ denominator keys); otherwise read the whole pass unknown, naming the unmatched numerator label set. With that change, G1's two cases pass, pass 1's pairing case still passes, and the F1 case still reads 0. G2 and G3: keep the guards.

## 5. Attacked and not broken

- **Absent-means-0 when a side is cut by `series_limit`:** a cut numerator makes the whole pass unknown (guarded, G2). A cut denominator makes it unknown too: the `?` path is a whole-pass `Err`.
- **Both answers empty:** reads unknown ("no series"), not 0.
- **Denominator series 0 with the numerator absent:** 0/0 reads unknown "empty population", with population 0.
- **Collision after pairing:** one unknown with no population. The reason names the projected dims, not the colliding label sets.
- **Canonicalisation:** a 309-digit integer, the 323-zero subnormal, long zero tails, `-0.000…` becomes `0`, and a negative subnormal keeps its sign. `raw` keeps the provider text.
- **Population:** `8.0` gives 8; `8.9`, `0.4` and over-i64 values give none; it is never a floor.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015c/m3-loki-adapter/scratch/`:

- `adversary/pass2/build.log`
- `adversary/pass2/alone-<case>.log` (5 files)
- `adversary/pass2/suite.log`
- `adversary/pass2/mutants.diff`
- `adversary/pass2/mutant-run.log`
- `adversary/pass2/mutant-lib.log`
- `adversary/mutant/`: re-synced from the tree with `rsync --delete`, now holding the pass-2 mutants. Pass 1's mutants survive only in `adversary/mutants.diff`
- `adversary/mutant/target/`: 159M, not deleted
- `adversary-pass-2.md`: this report

Inside the worktree, cargo wrote under `target/` (including `target/tmp/adv2_loki/`).

## 7. Findings block

```findings
- file: crates/thalamus-sources/src/loki.rs
  line: 180
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: absent-means-0 fires even when a numerator label set has no denominator twin, so a numerator grouped by fewer labels than its denominator turns real errors into ok 0 per cluster
- file: crates/thalamus-sources/src/loki.rs
  line: 161
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: reading a failed or cut numerator as an empty answer survives the unit's suite and pass 1, and would grade every cluster 0
- file: crates/thalamus-sources/src/loki.rs
  line: 197
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: keeping the first of two pairs projected onto one dims survives the unit's suite and pass 1's pairing case
- file: crates/thalamus-sources/src/loki.rs
  line: 243
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: an empty count answer writes 0 to the dimensionless series even when the definition declares dimensions, a series no non-empty answer feeds
```
