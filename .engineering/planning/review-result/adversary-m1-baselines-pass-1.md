---
format: aep.planning-md/3
id: review-result:adversary-m1-baselines-pass-1
kind: review-result
status: active
title: Adversary, story:m1-baselines, pass 1 (home paths shortened to ~)
relations:
- reviews: story:m1-baselines
revision: 1
---
```
unit: story:m1-baselines, uncommitted working tree on impl/m1-baselines (base b863833), thalamus-w15b-m1-baselines
verdict: red
cases: executed 10→13, red 2
origin: introduced 5, pre-existing 1, undecided 0
wrote-outside-worktree: 2 paths (part 6)
needs-coordinator: yes
```

The fold holds against the attacks the brief listed. Two of my new cases fail, both on wire-valid decimals that I built myself; nothing I could find produces them. The one gap that matters is upstream of this unit: the spec has no field for the fixed number, so every `fixed` baseline will read UNKNOWN in M1. That is why needs-coordinator is yes.

**1. `git --no-pager diff --stat`**
```
 Cargo.lock                               |   1 +
 crates/thalamus-core/Cargo.toml          |   1 +
 crates/thalamus-core/src/baseline/mod.rs | 631 +++++++++++++++++++++++++++++++
 3 files changed, 633 insertions(+)
```
All three tracked paths are the implementor's. Mine is one untracked test file, `crates/thalamus-core/tests/adv_baselines.rs`, so no non-test path was touched. `rustfmt --check` exits 0 and `b10x-gates scan-text` exits 0.

**2. Cases added** (in `~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m1-baselines/crates/thalamus-core/tests/adv_baselines.rs`)

| case | asserts | now |
|---|---|---|
| `adv_wire_valid_long_decimal_is_an_offset_baseline` | `offset_1d` with a 39-digit reading the wire grammar accepts returns that value | red |
| `adv_pooled_over_wire_valid_values_does_not_overflow` | pooled `0.0…01`×1 (37 fractional digits) and `5`×100 gives `4.950495049504950495…` | red |
| `adv_pooled_rounds_an_exact_half_away_from_zero` | a pooled exact half at digit 19 gives `±0.000000000000000001` | green; this is a mutant guard |

The red output from running these cases alone (`cargo test --locked -p thalamus-core --test adv_baselines`, exit 101):
```
expected an ok baseline reading, got Unknown(UnreadReading { reason: "pooled baseline overflows" })
expected an ok baseline reading, got Unknown(UnreadReading { reason: "offset 1d value `0.12345678901234567890123456789012345678` is not a decimal" })
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
To check the guard, I changed `>= 5` to `> 5` at mod.rs:499 in a scratch copy. My guard case failed (`adv_baselines.rs:162`). The implementor's `--test baselines` still passed: `test result: ok. 10 passed; 0 failed`.

**3. Suite run, after the cases existed** (`cargo test --locked --no-fail-fast -p thalamus-core`, exit 101)
```
adv_baselines:            test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
adv_conformance_feature:  test result: ok. 1 passed; 0 failed
baselines:                test result: ok. 10 passed; 0 failed
conformance, probes, unittests, doctests: test result: ok. 0 passed; 0 failed
```
The "before" count of 10 comes from the implementor's report.

**4. Findings**

| # | file:line | verdict / origin | finding | what reaches it |
|---|---|---|---|---|
| F1 | mod.rs:605 | INFEASIBLE / introduced | `Dec::parse` caps a decimal at 38 digits. The wire grammar (`json.rs:537`) has no cap, so a valid baseline becomes unknown with a reason that calls it "not a decimal". | Nothing found. No adapter exists yet. |
| F2 | mod.rs:472 | INFEASIBLE / introduced | Aligning scales in `i128` overflows on pooled inputs that can be computed, and the baseline becomes unknown. | Nothing found. It needs about 37 fractional digits. |
| F3 | mod.rs:499 | CONFIRMED / introduced | The half-rounding mutant survives the unit's suite. My guard case now covers it. | Not applicable; this is a gap in the tests. |
| F4 | spec/domains/definition.yaml:122, mod.rs:222 | INFEASIBLE / pre-existing | `EvaluationSpec` has no field for the fixed number. `story:m1-cli-eval-show` (line 38) calls `baseline::fold` and has nowhere to read the number from, so every `fixed` baseline will be UNKNOWN. That covers all `instant` signals and the advisory gates in decision D12. The fix is a spec field, which is outside this unit. | `m1-cli-eval-show`, `m3-advisory-gate-mapping` |
| F5 | mod.rs:347 | CONFIRMED / introduced | Rule 2 is applied as offset + (ts − window.start) > age. Design § 2.2 and the story say "offset exceeds age_s". Between the two, for ages from the offset up to offset + window span, the code says UNKNOWN and the documents say there is a value. The code's reading is the safer one, so the documents should say what it does. | Any `offset_*` definition on a series younger than offset + window |
| F6 | mod.rs:336 | CONFIRMED / introduced | Two offsets listed without a combiner become `pooled` whatever the signal kind. Design § 3.1 limits pooled to ratios, so a count or gauge silently gets an n-weighted mean, or UNKNOWN when it has no n. | The `instance-registry` validation in the plan checks only `instant` baselines |

All of these cover the uncommitted working tree on b863833.

**5. Attacked and not broken:**
- **Unknown, partial, missing or duplicate constituent:** the result is unknown and the reason names the offset.
- **n = 0, None or negative:** an unknown reading carries no value, so it is never 0 or OK.
- **higher_of / lower_of:** they pick the right side for negative values, mixed scales and comparisons that look backwards as text.
- **Pooled arithmetic:** numerator over denominator is correct, `-0` is avoided and rounding carries correctly.
- **7-day shift:** weekday and UTC offset are kept, including across a month boundary.
- **Rule 2:** the boundaries are exact, any offset that predates the series wins, and a series of unknown age makes offsets unknown.
- **Rule 3:** `instant` is refused for every non-fixed kind and for a mix of fixed and another kind.
- **Mixed shapes:** `fixed` beside another kind is refused, as are two combiners.

**6. Paths written outside the worktree:**
- `~/.cache/thalamus-waves/20261015b/m1-baselines/scratch/adversary/mutant/`: the mutated copy and its `target/`, 111M. I left the `target/` for you to remove, per the rule against deleting one by hand.
- `~/.cache/claude-tmp/claude-1000/-home-timo-beyond10x/31ebfb59-e4dc-47a7-84f7-7da6eb1ee418/tasks/bt1h0u78y.output`: the harness's log of the background mutant run.

**7.**
```findings
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 605
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: Dec::parse refuses wire-valid decimals over 38 digits, so a readable offset baseline folds to unknown with the false reason "is not a decimal"; no producer of such values found
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 472
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: pooled scale alignment in i128 overflows on wire-valid inputs with a computable result, giving unknown "pooled baseline overflows"; no producer of such values found
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 499
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the mutant >= 5 to > 5 in divide survives tests/baselines.rs; adv_pooled_rounds_an_exact_half_away_from_zero now kills it
- file: spec/domains/definition.yaml
  line: 122
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: EvaluationSpec has no field for the fixed number, so the fold caller in story:m1-cli-eval-show has no source and every fixed baseline (all instant signals, D12 advisory gates) will be UNKNOWN in M1
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 347
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: rule 2 uses offset plus window span over series age while design 2.2 and the story say offset exceeds age_s; the code is the safer reading and the documents should state it
- file: crates/thalamus-core/src/baseline/mod.rs
  line: 336
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: two offsets listed without a combiner resolve to pooled for every signal kind, while design 3.1 restricts pooled to ratios, so a count or gauge gets an n-weighted mean or UNKNOWN without being refused
```
