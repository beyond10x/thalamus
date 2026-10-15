---
format: aep.planning-md/3
id: review-result:adversary-m3-vocabulary-mapping-pass-1
kind: review-result
status: active
title: Adversary, story:m3-vocabulary-mapping, pass 1
relations:
- reviews: story:m3-vocabulary-mapping
revision: 1
---
unit: story:m3-vocabulary-mapping, uncommitted working tree of impl/m3-vocabulary-mapping on base 6b98444 (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m3-vocabulary-mapping)
verdict: NEEDS-CHANGE
cases: executed 436→441, red 4
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: none

## 1. Diff

`git --no-pager diff --stat 6b98444`: `88 files changed, 6625 insertions(+), 3218 deletions(-)`. That matches the tracked diff before this pass, so no tracked file changed. `--stat` does not list untracked files. This pass added two, both test files:

| file | lines |
|---|---|
| crates/thalamus-core/tests/adv_vocabulary_p1.rs | 90 |
| crates/thalamus-sources/tests/adv_vocabulary_p1.rs | 197 |

No implementation file was touched. No mutant copies were made.

## 2. Cases added (each run alone before the suite)

| case | asserts | now |
|---|---|---|
| core `adv_bare_mapping_key_is_refused_as_mapping_missing_like_json_null` | `mapping:` with nothing after it (YAML null) is refused as `MappingMissing`, as `"mapping": null` / `None` is on ProposeDefinition | red |
| core `adv_unquoted_integer_keys_load_as_their_text` | `{0: OK, 1: CRIT}` loads as keys `"0"`, `"1"`; `{1: CRIT, "1": OK}` is refused as a duplicate | green |
| sources `adv_unknown_read_with_raw_text_stays_unknown` | an `unknown` read with `raw: Some("0")` stays `unknown` through `map_reads` | red |
| sources `adv_prometheus_cut_result_stays_unknown_under_producer_mapped` | through `registry::read` and the recorded Prometheus double, a `complete: false` result's reads stay `unknown` | red |
| sources `adv_prometheus_colliding_series_stay_unknown_under_producer_mapped` | two series that collide on `dims` stay `unknown` | red |

Red output, `cargo test --locked -p thalamus-core --test adv_vocabulary_p1`:

```
thread 'adv_bare_mapping_key_is_refused_as_mapping_missing_like_json_null' panicked at crates/thalamus-core/tests/adv_vocabulary_p1.rs:57:23:
`mapping:` (YAML null) loads, with mapping Some({}), so every read of this blocking gate grades UNKNOWN forever; ProposeDefinition refuses the same definition as mapping-missing
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Red output, `cargo test --locked -p thalamus-sources --test adv_vocabulary_p1`:

```
thread 'adv_unknown_read_with_raw_text_stays_unknown' panicked at crates/thalamus-sources/tests/adv_vocabulary_p1.rs:136:5:
an unknown read the adapter made for a cut result is mapped onto the ladder: Ok(CompleteReading { value: Decimal("0") })
thread 'adv_prometheus_colliding_series_stay_unknown_under_producer_mapped' panicked at crates/thalamus-sources/tests/adv_vocabulary_p1.rs:190:9:
a colliding series reads Ok(CompleteReading { value: Decimal("0") }) under producer_mapped (raw Some("1"))
thread 'adv_prometheus_cut_result_stays_unknown_under_producer_mapped' panicked at crates/thalamus-sources/tests/adv_vocabulary_p1.rs:159:9:
a read of a cut Prometheus result is Ok(CompleteReading { value: Decimal("0") }) under producer_mapped (raw Some("4"))
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

The first run of the core file failed `cargo fmt --check`, in my own file only. I fixed it with `rustfmt --edition 2024` on my two files; no assertion changed.

## 3. Suite

| step | command | result |
|---|---|---|
| validate, drift, fmt, clippy `-D warnings` | `task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess` | each passed |
| test | same | stopped at the first red binary: `test result: FAILED. 1 passed; 1 failed` (core adv_vocabulary_p1); `task: Failed to run task "check": task: Failed to run task "test": exit status 101` (exit 201) |
| all cases | `cargo test --locked --workspace --no-fail-fast` | exit 101; 437 passed, 4 failed, 0 ignored; the 4 failures are the 4 red cases above |

`<before>` 436 is the sum of the `test result` lines in the implementor's `scratch/gate.log`. 436 + the 5 cases added = 441 executed.

The acceptance command `ess specify compile --path spec --format json | jq -e '…index("mapping") != null'` exits 0 (`true`).

## 4. Findings

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| 1 | crates/thalamus-sources/src/vocabulary.rs:47 | `map_one` re-maps every read that has `raw` text, including `unknown` reads the adapter made for a reason other than the value text. A cut or colliding Prometheus read becomes `ok` at position 0, so it grades OK. This breaks "unknown is never OK" and the module's own line "A read that was not made keeps its own reason". 3 red cases. | crates/thalamus-sources/src/prometheus.rs:231 keeps `series.text` as `raw` on `complete: false`, truncated and collision reads. Real recordings (`queue_depth_warned`, `max by (cluster, pod) (queue_depth)`) drive it through `registry::read`, the documented entry that every read passes through. The pipeline that will call it is still a stub (`crates/thalamus-pipeline/src/sample.rs`). Fix (not applied), either: in `map_one`, keep an `Unknown` read unless the adapter marks it as "value text not a number"; or have prometheus.rs:231 pass `raw: None` when the reading is `Err` for cut or collision. | NEEDS-CHANGE | introduced |
| 2 | crates/thalamus-core/src/registry.rs:916 | `optional_distinct_keys` sends an empty plain scalar through `deserialize_map`, and serde_yaml_ng reads that as an empty map. So `mapping:` loads as `Some({})`, is not refused, and every read grades UNKNOWN. `mapping: ~` / `null` is refused as a parse error. `"mapping": null` on ProposeDefinition is refused as `mapping-missing`. In the same block, `fixed:` with nothing after it is `None`. 1 red case. | `Definition::parse`, the instance loader, on a definition file with a stub `mapping:` line. Fix (not applied): treat a null or empty scalar as `None` before `distinct_keys`, for example by deserializing `Option<…>` through a wrapper. | CONFIRMED | introduced |
| 3 | crates/thalamus-core/src/rules/mapped.rs:45 | `mapping: {}` (and a table whose every value is `UNKNOWN`) passes `mapping_missing` in loader, propose and revise. A blocking gate then grades UNKNOWN forever. The spec guard is `not defined(...)`, so this matches the spec; the story's "no evaluation.mapping" is open to either reading. | Judgement, no red case. | INFEASIBLE (spec allows it) | introduced |
| 4 | crates/thalamus-core/tests/producer_mapped.rs:390 | No `mapping-unused` refusal. A `mapping` under `higher_is_worse` is silently ignored, unlike `fixed-number-unused` and `rolling-n-unused`. The test's doc says "with or without one" but it only exercises "without". | Judgement, no red case. | CONFIRMED | introduced |
| 5 | crates/thalamus-core/tests/producer_mapped.rs:165 | `unknown_mapped_reading_grades_unknown_without_score` asserts the reason contains `` `na` ``, a reason the test wrote into the reading itself. The case passes through grading step 2 whatever `mapped.rs` does: it is a "fixture asserts fixture" case. | Acceptance-named test. It should map `na` through `map_reads` first, as vocabulary.rs does. | CONFIRMED | introduced |

## 5. Attacked, not broken

| attack | result |
|---|---|
| exact match: case, leading or trailing space, `00`, `+1`, `1.0`, `-1`, `256`, empty string | UNKNOWN, as designed (fixture and existing tests) |
| unquoted YAML integer keys and the `1` / `"1"` duplicate | load as text; the duplicate is refused (green case) |
| na never ranks as green, worst-of in both orders | holds |
| N/A in mapping refused in loader, propose and revise; producer_mapped without mapping refused in all three | holds |
| N/A produced via `denominator.min_n` on a mapped signal | no adapter that sets `raw` sets a denominator (sql/prometheus/loki value reads pass `None`): not reachable |
| baseline unknown masking a mapped CRIT | grade() has no production caller yet: not reachable |
| score (§ 3.3 marks 0/1/2; D23 does not apply, since mapped has no lines) | positions match the marks; UNKNOWN has no score |
| mutants: `mapping_missing` `&&`→`\|\|` or `is_none`, VerdictDoc NA→UNKNOWN, dropping the refusal row, the identity under other rules | each caught by an existing test, judged by reading; no mutant copies built |

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p1-gate.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p1-suite.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-pass-1.md

Inside the worktree's `target/`, the Prometheus cases also write `target/tmp/adv_vocabulary_p1/{cut,collision}` (CARGO_TARGET_TMPDIR, as tests/prometheus.rs does).

## 7. Findings block

```findings
- file: crates/thalamus-sources/src/vocabulary.rs
  line: 47
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "map_reads turns an unknown read that still carries raw text (a cut or colliding Prometheus result, prometheus.rs:231) into an ok reading, so an unknown grades OK under producer_mapped."
- file: crates/thalamus-core/src/registry.rs
  line: 916
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a bare `mapping:` (YAML null) loads as an empty table and is not refused, while ProposeDefinition refuses JSON null as mapping-missing and the loader refuses `mapping: ~` as a parse error."
- file: crates/thalamus-core/src/rules/mapped.rs
  line: 45
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "an empty mapping, or one whose every value is UNKNOWN, is accepted everywhere, so a producer_mapped signal can be configured to grade UNKNOWN forever; the spec guard (not defined) allows it."
- file: crates/thalamus-core/tests/producer_mapped.rs
  line: 390
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a mapping under a non-mapped rule is silently ignored rather than refused like fixed-number-unused and rolling-n-unused, and the test claiming \"with or without one\" only exercises \"without\"."
- file: crates/thalamus-core/tests/producer_mapped.rs
  line: 165
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "unknown_mapped_reading_grades_unknown_without_score asserts a reason the test itself wrote into the reading and passes whatever mapped.rs does."
```
