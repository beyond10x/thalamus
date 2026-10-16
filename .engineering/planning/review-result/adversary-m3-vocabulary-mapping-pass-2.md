---
format: aep.planning-md/3
id: review-result:adversary-m3-vocabulary-mapping-pass-2
kind: review-result
status: active
title: Adversary, story:m3-vocabulary-mapping, pass 2
relations:
- reviews: story:m3-vocabulary-mapping
revision: 1
---
unit: story:m3-vocabulary-mapping, uncommitted working tree of impl/m3-vocabulary-mapping on base 6b98444 (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m3-vocabulary-mapping)
verdict: INFEASIBLE
cases: executed 444→451, red 1
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (part 6)
needs-coordinator: finding 1 needs a design call: under producer_mapped, design § 2's "re-grade history" cannot hold unless a sample keeps the producer's text, which is a spec change outside this story

## 1. Diff

`git --no-pager diff --stat 6b98444`: `91 files changed, 7374 insertions(+), 1905 deletions(-)`, the same as before this pass, so no tracked file changed. `--stat` does not list untracked files. This pass added two files, both tests:

| file | cases |
|---|---|
| crates/thalamus-core/tests/adv_vocabulary_p2.rs | 4 |
| crates/thalamus-sources/tests/adv_vocabulary_p2.rs | 3 |

I touched no implementation file. The mutants were built in a scratch copy, never in the tree.

## 2. Cases added (each file run alone before the suite)

| case | asserts | now |
|---|---|---|
| core `adv_every_yaml_null_mapping_is_refused_as_mapping_missing` | `mapping:` written bare, `~`, `null`, `Null` or `NULL` under producer_mapped is refused as MappingMissing | green |
| core `adv_null_mapping_under_another_rule_is_absent_and_empty_one_unused` | under higher_is_worse, a bare, `~` or `null` mapping loads as `None`; `{}` and `{"1": N/A}` are refused as MappingUnused and nothing else | green; red on mutant 3 |
| core `adv_plain_yaml_keys_keep_the_text_written` | the keys `1.0`, `0x1`, `true`, `007` and `1e3` load as the text written | green |
| core `adv_partial_reading_under_producer_mapped_is_unknown_never_a_position` | a partial reading of 0, 1 or 2, with either bound, grades `(UNKNOWN, None, "partial read")` | green; red on mutant 1 |
| sources `adv_partial_read_carrying_mapped_text_keeps_its_reading` | a partial read whose raw text is `red`, `green` or `3`, typed `Text` or not, comes out of `map_reads` unchanged | green; red on mutant 2 |
| sources `adv_sql_whitespace_empty_and_padded_cells_map_only_by_exact_text` | end to end through `registry::read`: the cells `"  "`, `""`, `" green"` and `"green "` are typed `Text`, keep their raw text and read `unknown` with a reason naming the cell; a NULL cell keeps `raw: None, unread: None` and its own reason; `green` maps to `ok 0` | green |
| sources `adv_a_mapping_revision_re_grades_stored_history` | a sample mapped at rev 1 (`red: CRIT`, `purple` unlisted) and graded at rev 2 (`red: WARN`, `purple: OK`) gives rev 2's verdicts | **red** |

Red output, `cargo test --locked -p thalamus-sources --test adv_vocabulary_p2 adv_a_mapping_revision`:

```
thread 'adv_a_mapping_revision_re_grades_stored_history' (2147338) panicked at crates/thalamus-sources/tests/adv_vocabulary_p2.rs:287:5:
re-graded at rev 2, (producer value, rev 2's verdict, verdict given, reason): [("red", WARN, CRIT, "the producer's value maps to CRIT"), ("purple", OK, UNKNOWN, "producer value `purple` is not in evaluation.mapping")]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
```

After `rustfmt`, the same panic is at line 291. The other six cases passed when first run alone (`cargo test --locked -p thalamus-core --test adv_vocabulary_p2`: 4 passed; `-p thalamus-sources --test adv_vocabulary_p2`: 2 passed, before the third case was added).

Mutants. Each was built in a copy at `scratch/adversary-p2/mutant` with its own `target/` and run with `cargo test --locked --workspace --no-fail-fast`, one at a time:

| # | mutation | pre-existing cases red | caught by |
|---|---|---|---|
| 1 | crates/thalamus-core/src/rules/mod.rs:142 `Prepared::Mapped(_) => None` → `Some(rule.decide(value))` | 0 of 444 | core partial case: `partial 0 Lower: Graded { verdict: OK, score: Some(Decimal("0")), reason: "the producer's value maps to OK", … }` |
| 2 | crates/thalamus-sources/src/vocabulary.rs:39 `Reading::Partial(_) => false` → `true` | 0 of 444 | sources partial case (adv_vocabulary_p2.rs:108) |
| 3 | crates/thalamus-core/src/rules/mapped.rs:79 `mapping.is_some()` → `is_some_and(\|m\| !m.is_empty())` | 0 of 444 | core null/unused case (adv_vocabulary_p2.rs:93) |

Each mutant run reported 449 passed and 2 failed. The 2 failures were the catching case and the re-grade case, which fails with or without a mutant. Logs: `scratch/adversary-p2/mutant-{escalate,partial,unused-empty}.log`. The copy was removed after `cargo clean --manifest-path …/mutant/Cargo.toml`, which removed 1.0 GiB.

My first gate run failed clippy `clone_on_copy` in my own core file (`bound.clone()`). I fixed it (`bound`) and ran `rustfmt --edition 2024` on my sources file. No assertion changed.

## 3. Suite

| step | command | result |
|---|---|---|
| validate, drift, fmt, clippy `-D warnings` | `task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess` | each passed |
| test | same | `test adv_a_mapping_revision_re_grades_stored_history ... FAILED`, `test result: FAILED. 2 passed; 1 failed`, `task: Failed to run task "check": task: Failed to run task "test": exit status 101` (EXIT=201) |
| all cases | `cargo test --locked --workspace --no-fail-fast` | exit 101; 450 passed, 1 failed, 0 ignored |

The `<before>` of 444 is the sum of the `test result` lines in the implementor's `scratch/gate-c2.log` (EXIT=0). 444 + the 7 cases added = 451 executed.

## 4. Findings (tree: the uncommitted working tree above, base 6b98444)

| # | file:line | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| 1 | crates/thalamus-sources/src/vocabulary.rs:43 | `map_reads` applies the table before the sample exists, and `SignalSampleData` has no field for the producer's text. So a stored sample holds the position under the table in force when it was read. Graded at rev 2, it keeps rev 1's verdict (`red` gives CRIT, not WARN), and a value rev 2 adds stays UNKNOWN with a false reason ("`purple` is not in evaluation.mapping"). This contradicts design § 2 ("lets a threshold change re-grade history without reading the source again"), § 5.1 and D5. Red case at adv_vocabulary_p2.rs:291. | Nothing today: `grade()` has no production caller. `story:m1-cli-eval-show` (planned) grades "every stored sample not yet graded at its definition current revision", which is this path. A fix needs the producer text on the sample, a spec change to `thalamus.observation.SignalSample`, or a design statement that a mapping revision never re-grades. Either is outside this story. | INFEASIBLE | introduced |
| 2 | crates/thalamus-core/src/rules/mod.rs:142 | No pre-existing case grades a partial reading under producer_mapped. Mutant 1 (escalate to the position) left all 444 green, and grades a partial 0 as OK. The new core case catches it and is green now. | The `sql.count` truncated read and the `loki.count` cut read are partial, and `map_reads` passes them through. | CONFIRMED | introduced |
| 3 | crates/thalamus-sources/src/vocabulary.rs:39 | No pre-existing case hands `map_reads` a partial read, so D44's "partial … keeps its own reading … whatever text it carries" was unheld. Mutant 2 left all 444 green. The new sources case catches it. | No adapter today makes a partial read that carries raw text, so the mutant changes only reasons on real reads. | CONFIRMED | introduced |
| 4 | crates/thalamus-core/src/rules/mapped.rs:79 | `mapping: {}` under another rule had no case, because the conformance witnesses and `a_mapping_under_any_other_rule_is_refused_as_unused` use a non-empty table. Mutant 3 left all 444 green. The new core case catches it. | The loader, on a definition file carrying `mapping: {}` under a threshold rule. | CONFIRMED | introduced |
| 5 | crates/thalamus-core/src/behaviour/definition.rs:285 | `warn` and `crit` lines under producer_mapped are numbers nothing reads, and they load and propose without refusal. That is the inverse of `mapping-unused`, which coordinator decision 4 justified by the same "a number nothing reads" argument. Shown by the core p2 `definition_file`, which loads with lines under producer_mapped, and by `tests/vocabulary.rs::definition`, which builds that shape. | The loader, `ProposeDefinition` and `ReviseDefinition`. Judgement only; no rule requires it. | CONFIRMED | introduced |

## 5. Attacked, not broken

| attack | result |
|---|---|
| every place a `Read` is built: adapter.rs `unknown`, sql.rs `Reads::read`/`unknown`, git.rs, prometheus.rs `read`, loki.rs `read`, http_probe.rs `known`, kubernetes via `unknown` | only sql.rs:371-374 sets `Text`, only on a non-numeric present cell; every other site leaves `None` |
| SQL NULL, duplicate, cut, ambiguous column, empty answer | `raw: None, unread: None`; the existing cases plus the new whitespace case |
| SQL whitespace-only, empty, padded cells | typed `Text`, matched exactly, never trimmed (new case, green) |
| Prometheus cut or colliding reads; Loki non-decimal or duplicate reads | `unread: None`, so not mapped (p1 cases, now green) |
| Kubernetes and Git reads (no raw) | an ok read becomes UNKNOWN "no producer value", never graded on its number |
| HTTP probe | an ok read carries raw (the status, or the refusal text for a down target), so it maps by that text; consistent with D44's "an ok reading" |
| loader parity: bare / `~` / `null` / `Null` / `NULL`, `{}`, N/A, under producer_mapped and under another rule | matches propose/revise (new cases, green) |
| plain YAML keys (`1.0`, `0x1`, `true`, `007`, `1e3`) | kept as written |
| JSON duplicate keys on the wire | refused by generated/thalamus-model/src/json.rs:40, as the loader refuses them |
| new refusals against existing fixtures | no fixture or instance file sets `mapping`; existing tests changed only by `mapping: None` lines and outcome-name arms |
| worst-of across mixed mapped and unmapped reads | `ladder::worst` is unchanged; a mapped read is a scored verdict or UNKNOWN, never N/A |
| D44 row against the code | every clause matches: Text only in sql, mapping-missing (bare, null, empty, N/A), all-UNKNOWN kept, mapping-unused, exact match, no wildcard, decision by type |

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-pass-2.md
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p2-gate.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p2-suite.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p2/mutant-escalate.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p2/mutant-partial.log
- ~/.cache/thalamus-waves/20261015e/m3-vocabulary-mapping/scratch/adversary-p2/mutant-unused-empty.log

The mutant copy `scratch/adversary-p2/mutant/` was cleaned with `cargo clean` and removed. Inside the worktree's `target/`, the sources cases write `target/tmp/adv_vocabulary_p2/whitespace` (CARGO_TARGET_TMPDIR).

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-sources/src/vocabulary.rs", "line": 43, "category": "contract-drift", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "the table is applied before the sample is stored and the sample keeps no producer text, so a mapping revision cannot re-grade history (design § 2, § 5.1, D5): rev 2 grades a rev-1 sample with rev 1's verdict and a false 'not in evaluation.mapping' reason."},
  {"file": "crates/thalamus-core/src/rules/mod.rs", "line": 142, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "no pre-existing case grades a partial reading under producer_mapped; escalating it to its position left all 444 cases green and grades a partial 0 as OK."},
  {"file": "crates/thalamus-sources/src/vocabulary.rs", "line": 39, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "no pre-existing case hands map_reads a partial read; mapping partial reads left all 444 cases green."},
  {"file": "crates/thalamus-core/src/rules/mapped.rs", "line": 79, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "no pre-existing case refuses an empty mapping under another rule as mapping-unused; exempting empty tables left all 444 cases green."},
  {"file": "crates/thalamus-core/src/behaviour/definition.rs", "line": 285, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "warn and crit lines under producer_mapped are numbers nothing reads and are not refused, the inverse of the new mapping-unused refusal."}
]
```
