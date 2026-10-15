---
format: aep.planning-md/3
id: review-result:adversary-m1-baselines-pass-2
kind: review-result
status: active
title: Adversary, story:m1-baselines, pass 2 (home paths shortened to ~)
relations:
- reviews: story:m1-baselines
revision: 1
---
```
unit: story:m1-baselines, pass 2, uncommitted working tree on impl/m1-baselines (base b863833), thalamus-w15b-m1-baselines
verdict: green
cases: executed 15→18, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: 2 paths (part 6)
needs-coordinator: no
```

The correction holds. In 5,000 fixed-seed cases, pooled, higher_of and lower_of all match an independent exact `i128` oracle, and the pooled results at a population of `i64::MAX` are exact. Two mutants I put into the new arithmetic were each caught by the implementor's suite as well as mine. The one finding is a note: the new refusal only fires when a pass runs, not when the definition loads.

**1. `git --no-pager diff --stat`**
```
 Cargo.lock                               |   1 +
 crates/thalamus-core/Cargo.toml          |   1 +
 crates/thalamus-core/src/baseline/mod.rs | 774 +++++++++++++++++++++++++++++++
 3 files changed, 776 insertions(+)
```
All three tracked paths are the implementor's. This pass added one untracked test file, `crates/thalamus-core/tests/adv_baselines_pass2.rs`. `adv_baselines.rs` from pass 1 is unchanged. No non-test path was touched. `rustfmt --check`, `cargo clippy -p thalamus-core --all-targets -D warnings` and `b10x-gates scan-text` all exit 0.

**2. Cases added** (in `~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m1-baselines/crates/thalamus-core/tests/adv_baselines_pass2.rs`)

| case | asserts | now |
|---|---|---|
| `adv_pooled_and_bounds_agree_with_an_exact_oracle` | 5,000 seeded draws covering signs, zeros (including `0.000`), scales 0 to 6 and n from 0 to 10^6. Pooled must equal the `i128` oracle rounded half away from zero at 18 digits, or be unknown when the population is 0. higher_of and lower_of must equal the oracle's pick, the first of equal values. | green |
| `adv_pooled_at_the_largest_population` | `1`×MAX with `0`×MAX gives `0.5`; `-0.3`×MAX with `-0.1`×MAX gives `-0.2`; `0.029`×556 with `0.013`×MAX gives `0.013000000000000001` | green |
| `adv_bounds_compare_long_decimals_by_value` | 43-digit values compared with `0.1`, both signs; `-0` against `0.00` keeps the first | green |

Run alone (`cargo test --locked -p thalamus-core --test adv_baselines_pass2`, exit 0): `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s`.

To show the cases can fail, I ran two mutants of `mod.rs` in the scratch copy:

| mutant | my cases | implementor `baselines` |
|---|---|---|
| `compare`: dropped `order.reverse()` for negative values | oracle and long-compare failed | `higher_of_and_lower_of_pick_the_bound` failed (11/12 passed) |
| `plus`: mixed-sign `Less` branch took `self.negative` | oracle and max-population failed | 2 failed (10/12 passed) |

**3. Suite run, after the cases existed** (`cargo test --locked --no-fail-fast -p thalamus-core`, exit 0)
```
adv_baselines:            test result: ok. 3 passed; 0 failed
adv_baselines_pass2:      test result: ok. 3 passed; 0 failed
adv_conformance_feature:  test result: ok. 1 passed; 0 failed
baselines:                test result: ok. 12 passed; 0 failed
unittests, conformance, probes, doctests: test result: ok. 0 passed; 0 failed
```
Executed 15→18: the "before" count is `baselines` 12 plus `adv_baselines` 3, from the coordinator's message.

**4. Findings**

| # | file:line | verdict / origin | finding | what reaches it |
|---|---|---|---|---|
| G1 | mod.rs:368 | CONFIRMED / introduced | `PooledNeedsRatio` fires only in `plan` and `fold`. The definition validator in `story:m1-instance-registry` (story line 39) checks only `instant` baselines, so a gauge or count that declares `[offset_1d, offset_7d]` loads as Active and is then refused on every eval pass instead of when it loads. | `m1-cli-eval-show` calls `baseline::plan` for each Active definition |

This covers the uncommitted working tree on b863833.

**5. Attacked and not broken:**
- **Signs:** comparisons with mixed signs and two negatives work, and a mixed-sign sum or a cancellation never gives `-0`.
- **Zero:** `-0`, `0.00` and a numerator of 0 at any scale give `0`, and every n being 0 gives unknown.
- **Scale normalisation:** values are aligned to the largest scale and trailing zeros are dropped, with scales from 0 to 43 digits.
- **Rounding at the 18th digit:** half rounds away from zero for both signs and rounding carries correctly; the 5,000-case oracle and the pass-1 guard both cover it.
- **Large n:** a population of `i64::MAX` on both sides gives sums of 2^64 − 2 with no overflow, and a negative n gives unknown.
- **higher_of / lower_of across scales and long digit strings:** both pick correctly.
- **PooledNeedsRatio:** it fires for named pooled and for two bare offsets, on every kind that is not a ratio, in both `plan` and `fold`. higher_of and lower_of still fold for counts.
- **Not re-attacked:** pass-1 F4 (deferred to `story:m2-spec-history-records`) and F5 (the coordinator's documentation fix), because both were routed out of this unit.

**6. Paths written outside the worktree:**
- `~/.cache/thalamus-waves/20261015b/m1-baselines/scratch/adversary/mutant/`: the pass-1 scratch copy, reused. Its `mod.rs` now holds the last mutant (`plus`), its test files are copies, and its `target/` was left for you to remove.
- `~/.cache/thalamus-waves/20261015b/m1-baselines/scratch/adversary-pass-2.md`: this report.

**7.**
```findings
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 368
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: PooledNeedsRatio fires only in plan and fold while the story:m1-instance-registry validator checks only instant baselines, so a non-ratio definition with two bare offsets loads Active and is refused on every eval pass instead of at load
```
