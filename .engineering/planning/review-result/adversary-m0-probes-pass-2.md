---
format: aep.planning-md/3
id: review-result:adversary-m0-probes-pass-2
kind: review-result
status: active
title: Adversary, story:m0-probes, pass 2
relations:
- reviews: story:m0-probes
revision: 1
---
unit: story:m0-probes, uncommitted working tree of impl/m0-probes over base 5d0ee31 (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m0-probes), after the pass-1 correction round
verdict: CONFIRMED (2 notes); all 7 pass-1 corrections hold, nothing blocks
cases: executed 144→150, red 1
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: 19 paths under ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/ (part 6)
needs-coordinator: my red case `adv_p2_generated_decoders_refuse_unit_outside_alphabet` shows a pre-existing ESS generator gap and keeps `task check` red. Route it out (ESS issue) and decide whether the case stays in this tree

## 1. Diff

`git --no-pager diff --stat` is unchanged from the tree I was handed: 52 files, 969 insertions, 111 deletions, all of them the implementor's or the coordinator's. I touched no tracked file. My files are untracked, and both are test files:

```
?? crates/thalamus-core/tests/adv_probes_p2.rs        (alphabet, 2 cases)
?? crates/thalamus-core/tests/adv_probes_p2_depth.rs  (nesting bound, 4 cases)
```

No non-test path changed. I ran `rustfmt --edition 2024` on my two files only. No `core.<pid>` file was produced (`find … -name 'core.[0-9]*'` outside `target/` is empty).

## 2. Cases added (each binary run alone, before the suite)

| case | asserts | now |
|---|---|---|
| `adv_probes_p2_depth.rs::adv_p2_depth_bound_is_64_for_every_innermost_component` | uses the literal 64, not `MAX_DEPTH`. 64 levels of `(` around `m`, `[in_i]`, `{x}`, `k[IU]2{y}`, `m{[a](b)}`, `10*-3` and `%[slope]` are admitted. 65 levels are refused with `(TooDeep, 64)` | green |
| `…::adv_p2_too_deep_names_the_offset_of_the_65th_parenthesis` | `/` + 65 levels gives `(TooDeep, 65)`. `s.` + 65 levels gives `(TooDeep, 66)` | green |
| `…::adv_p2_sibling_parentheses_release_their_level` | 1 000 sibling `(m)`, 200 siblings at depth 64, and 50 chains each 64 deep are all admitted | green |
| `…::adv_p2_no_input_overflows_the_stack` | 500 000 × `[`, `{`, `(`, `([{`, exponent digits, `[` inside an annotation, a long bracket atom, `m.m…`, `(m)/(m)…` and unclosed `(`… all return. Unclosed `(` ×500 000 gives `(TooDeep, 64)` | green |
| `adv_probes_p2.rs::adv_p2_unit_alphabet_is_exactly_ascii_33_to_126_everywhere` | the YAML scalar, unescaped (only `\"` and `\\` are admitted as escapes), equals ASCII 33–126 (94 characters). So does every generated copy: `source.schema.json` (1 copy), `thalamus-service.openapi.json` (2), `website/static/schemas/signal-definition.schema.json` (1) and `signal-sample.schema.json` (1) | green |
| `adv_probes_p2.rs::adv_p2_generated_decoders_refuse_unit_outside_alphabet` | `thalamus_model::server::wire::decode_thalamus_definition_unit` and `thalamus_spec_types::ThalamusDefinitionUnit` refuse `"m s"`, `"m\ts"`, `"m\0"`, `"m\u{7f}"`, `"µm"`, `""` and `" "` | **red** |

The depth binary on its first run, `cargo test --locked -p thalamus-core --test adv_probes_p2_depth`, exit 0:

```
test adv_p2_depth_bound_is_64_for_every_innermost_component ... ok
test adv_p2_too_deep_names_the_offset_of_the_65th_parenthesis ... ok
test adv_p2_sibling_parentheses_release_their_level ... ok
test adv_p2_no_input_overflows_the_stack ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

My first draft of `adv_p2_unit_alphabet_is_exactly_ascii_33_to_126_everywhere` was wrong. It expected ≥9 copies across 9 files, but only 4 files carry `Unit`, and `types-report.json` holds only digests. It failed with `found only 5 Unit alphabets in generated files`, which is my mistake, not a finding. I corrected it to the exact per-file copy counts before any suite run (draft kept at `scratch/adversary-p2/adv_probes_p2.first-draft.rs`). The rerun alone, `cargo test --locked -p thalamus-core --test adv_probes_p2`, exit 101:

```
test adv_p2_generated_decoders_refuse_unit_outside_alphabet ... FAILED
test adv_p2_unit_alphabet_is_exactly_ascii_33_to_126_everywhere ... ok
---- adv_p2_generated_decoders_refuse_unit_outside_alphabet stdout ----
thread 'adv_p2_generated_decoders_refuse_unit_outside_alphabet' (2294597) panicked at crates/thalamus-core/tests/adv_probes_p2.rs:127:5:
Unit texts outside ASCII 33-126 admitted: thalamus-model ["m s", "m\ts", "m\0", "m\u{7f}", "µm", "", " "], thalamus-spec-types ["m s", "m\ts", "m\0", "m\u{7f}", "µm", "", " "]
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Origin run against the base, not the tree: `git archive 5d0ee31 generated` was extracted to `scratch/adversary-p2/base/`, and the same case ran in a scratch crate (`scratch/adversary-p2/base-run/`, `cargo test --offline`), exit 101 with the identical message. The gap is pre-existing.

Mutant probe on copies, not on the tree. `unit.rs` was copied to `scratch/adversary-p2/mutants/{depth-drop,max-depth-8}/src/unit.rs`. Each copy ran the implementor's `ucum_bounds_nesting_at_64_levels` verbatim, plus my depth file:

| mutant | `ucum_bounds_nesting_at_64_levels` | my depth cases |
|---|---|---|
| `self.depth -= 1;` deleted (unit.rs:548) | ok (survives) | `sibling_parentheses_release_their_level` FAILED: `refused 1000 sibling groups at depth 1` |
| `MAX_DEPTH = 8` (unit.rs:65) | ok (survives) | all 4 FAILED, e.g. `left: (TooDeep, 9)` / `right: (TooDeep, 65)` |

## 3. Suite

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, exit 201, run after the cases existed. `specify validate` passed. `drift` printed `generated/ and spec/suite.json match spec/`. `fmt --check` and `clippy -D warnings` passed. `test` stopped at my red binary:

```
     Running tests/adv_probes.rs (target/debug/deps/adv_probes-ea19b91a0bea5efc)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
     Running tests/adv_probes_depth.rs (target/debug/deps/adv_probes_depth-d5a2aec76f169909)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/adv_probes_p2.rs (target/debug/deps/adv_probes_p2-4c8b7fa92cc35c7e)
test adv_p2_generated_decoders_refuse_unit_outside_alphabet ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
error: test failed, to rerun pass `-p thalamus-core --test adv_probes_p2`
```

The first attempt failed earlier, at `fmt --check`, on my two unformatted files. I formatted only those two and reran. That log was overwritten by the rerun.

`docs-check` did not run inside the gate, so I ran it separately: `task docs-check ESS=…`, exit 0, with `website/docs/reference/ess: 7`, `website/data/ess: 7`, `website/static/schemas: 6`, `website/data: 1` and `website/static/img: 1` `generated files current`. The website files match `docs-generate` exactly.

`cargo test --locked --workspace --no-fail-fast`, exit 101. The only failure is the case above. Executed: 150 in total, and 144 with my two binaries deselected (their 6 cases subtracted from the same run's `test result` lines). Logs: `scratch/adversary-p2/task-check.log`, `docs-check.log`, `suite-nofailfast.log`.

## 4. Findings (tree: impl/m0-probes working tree over 5d0ee31)

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/thalamus-core/tests/probes.rs:164 | `ucum_bounds_nesting_at_64_levels` builds its inputs from `MAX_DEPTH`, so `MAX_DEPTH = 8` survives it. It nests a single chain, so deleting `self.depth -= 1` also survives. The code is correct today. My green cases in `adv_probes_p2_depth.rs` catch both mutants | CONFIRMED | introduced | `parse_ucum`. No runtime caller until `story:m1-instance-registry` |
| 2 | generated/thalamus-model/src/server/wire.rs:638 | Neither generated decoder applies the `Unit` alphabet: `decode_thalamus_definition_unit` is `text_at` only, and spec-types `ThalamusDefinitionUnit` is `#[serde(transparent)]`. So space, tab, NUL, DEL, `µ` and the empty string are admitted. Design § 8 (docs/design/thalamus.md:358, new in this unit) says `Unit` "holds only the character set (ASCII 33–126)", which is true of the spec and false of both decoders. Fix outside this unit: an ESS generator issue. In-unit option: qualify § 8 ("declared, not enforced by the generated decoders; `parse_ucum` refuses these") | INFEASIBLE | pre-existing | `decode_command_thalamus_definition_propose_definition` (wire.rs:2549 → :2575), `…revise_definition` (:2856 → :2881) and `decode_command_thalamus_observation_record_sample` (→ :3535). All of them sit behind the model server's `POST` routes. Run at base 5d0ee31: same red |

## 5. Attacked and not broken

- Depth 64 for `(`, `[` and `{`: only `(` recurses (`component` → `term`, unit.rs:540–547). `[` and `{` are loops (unit.rs:565–600) and do not count. Exactly 64 levels are admitted and the 65th is refused at its own byte offset, whatever the innermost component (4 cases, green).
- No other recursion path overflows: 500 000-byte inputs of every opener, of exponent digits, and annotation and bracket bytes all return. The pass-1 `adv_probes_depth` (500 000 levels) is now green.
- `ATOMS` against the UCUM 2.2 essence (`scratch/adversary/essence/ucum-essence.xml`): all 312 codes and every `isMetric` flag are identical, `diff` exit 0, base units counted as metric. `PREFIXES` against the essence: all 24 are identical, `diff` exit 0. `atm` and `[lbf_av]` are present. `b` and `att` are non-metric. R, Q, r and q are gone.
- Widened alphabet: exactly ASCII 33–126 in the YAML, unescaped, and in all 5 generated copies. There is no whitespace, no control character and no stray escape (green case).
- Regeneration (`git diff generated/ spec/suite.json`): the changed lines are the model, contract, source and projection digests, the `x-ess-alphabet` values, and the alphabet in the `Unit` doc comment and in `thalamus-service.docs.md`. Nothing else changed.
- The decoder test pins the corrected § 8 sentence. `probes.rs::unknown_reading_carries_no_value` asserts spec-types refuses all 7 inputs, and that the model decoder returns `Ok(Reading::Unknown(UnreadReading { reason }))` with `reason == "source timed out"` through an exhaustive pattern. The coordinator-rewritten `adv_probes.rs::adv_model_decoder_drops_value_under_unknown` asserts the same. Read in `wire.rs:1171–1306`, not run separately: the model decoder requires `reason` and so refuses the flat, numeric and reason-less forms. That matches "ignores the extra member and drops it".
- `annotation_char` (unit.rs:484) admits `[`, `]`, `(`, `'`, `"`, `\` and `,`, and refuses only braces. It interacts cleanly with bracket scanning: `m{[a](b)}` is admitted at depth 64, and `{x}]` and `(m){x}` are refused (read).
- Website files: `docs-check` exit 0 (part 3).

## 6. Paths written outside the worktree

All are under ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/:

- adversary-p2/essence-units.txt
- adversary-p2/essence-base.txt
- adversary-p2/essence-prefixes.txt
- adversary-p2/essence-all.txt
- adversary-p2/impl-atoms.txt
- adversary-p2/impl-prefixes.txt
- adversary-p2/p2-depth.log
- adversary-p2/p2.log (the first, wrong draft's run)
- adversary-p2/p2-rerun.log
- adversary-p2/task-check.log
- adversary-p2/docs-check.log
- adversary-p2/suite-nofailfast.log
- adversary-p2/adv_probes_p2.first-draft.rs
- adversary-p2/base/ (1.2M, `git archive 5d0ee31 generated`)
- adversary-p2/base-run/ (161M, scratch crate with its own `target/` and a copy of `Cargo.lock`)
- adversary-p2/mutants/depth-drop/ (22M, crate with its own `target/`)
- adversary-p2/mutants/max-depth-8/ (22M, crate with its own `target/`)
- adversary-p2/ (the directory itself)
- adversary-pass-2.md (this report)

The worktree session lease `adversary-m0-probes-p2-20261015c` was acquired and has been released.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/tests/probes.rs", "line": 164, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "ucum_bounds_nesting_at_64_levels derives its inputs from MAX_DEPTH and nests one chain, so MAX_DEPTH = 8 and a dropped depth decrement both survive it; adv_probes_p2_depth.rs catches both."},
  {"file": "generated/thalamus-model/src/server/wire.rs", "line": 638, "category": "contract-drift", "severity": "note", "verdict": "INFEASIBLE", "origin": "pre-existing", "message": "Neither generated decoder enforces the Unit alphabet, so whitespace, control characters, non-ASCII and the empty string are admitted although design section 8 says Unit holds only ASCII 33-126."}
]
```
