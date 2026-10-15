---
format: aep.planning-md/3
id: review-result:adversary-m0-probes-pass-1
kind: review-result
status: active
title: Adversary, story:m0-probes, pass 1
relations:
- reviews: story:m0-probes
revision: 1
---
unit: story:m0-probes, uncommitted working tree of impl/m0-probes over base 5d0ee31 (~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m0-probes)
verdict: NEEDS-CHANGE
cases: executed 135→143, red 7
origin: introduced 5 / pre-existing 1 / undecided 0
wrote-outside-worktree: 11 paths (part 6)
needs-coordinator: (a) widening the `thalamus.definition.Unit` alphabet is a spec change that moves the model digest and is outside this unit's comment-only scope; (b) design § 8's "refused on the wire" claim: correct the doc, or fix the ESS generator's model decoder

## 1. Diff

`git --no-pager diff --stat` (tracked files; unchanged from the implementor's tree, I touched none of them):

```
 Cargo.lock                           |   1 +
 crates/thalamus-core/Cargo.toml      |   1 +
 crates/thalamus-core/src/unit.rs     | 611 +++++++++++++++++++++++++++++++++++
 crates/thalamus-core/tests/probes.rs | 180 +++++++++++
 docs/design/thalamus.md              |  12 +
 spec/domains/definition.yaml         |   6 +-
 6 files changed, 808 insertions(+), 3 deletions(-)
```

Untracked, mine (`git status --short`), both test files:

```
?? crates/thalamus-core/tests/adv_probes.rs        (167 lines)
?? crates/thalamus-core/tests/adv_probes_depth.rs  (19 lines)
```

No non-test path was changed. The aborting depth case wrote two 4.7 MB core dumps (`crates/thalamus-core/core.<pid>`). They were my runs' and I deleted them. Every run of `adv_probes_depth` makes another until the fix lands, so do not `git add -A` the crate directory.

## 2. Cases added (each binary run alone, before the suite)

Reference: the UCUM 2.2 essence (`ucum-essence.xml`, version 2.2, revision-date 2024-06-17), fetched from github.com/ucum-org/ucum, and the spec page ucum.org (§ 3, § 4, § 5, § 6, BNF). A diff of `ATOMS` against the essence (code and isMetric) found 310 entries against 312. The only differences are `atm` and `[lbf_av]` missing and `att` and `b` flagged metric. `[FNU]` is in 2.2. The essence has 24 prefixes and none is R, Q, r or q.

| case | asserts | now |
|---|---|---|
| `adv_probes.rs::adv_ucum_refuses_prefixes_outside_ucum_2_2` | `Rm`, `Qg`, `rm`, `qs` refused (not UCUM 2.2 prefixes) | red |
| `adv_probes.rs::adv_ucum_refuses_prefix_on_non_metric_att_and_barn` | `katt`, `matt`, `mb`, `kb` refused (essence: `att`, `b` isMetric="no") | red |
| `adv_probes.rs::adv_ucum_admits_atm_and_pound_force` | `atm`, `[lbf_av]`, `[lbf_av]/[sin_i]` admitted | red |
| `adv_probes.rs::adv_ucum_admits_square_brackets_inside_annotation` | `{a[b]}`, `m{[x]}`, `{]}` admitted (§ 6 ■1: "The full range of characters 33–126 can be used within a pair of curly braces") | red |
| `adv_probes.rs::adv_model_decoder_refuses_value_under_unknown` | `thalamus_model::server::wire::decode_thalamus_observation_reading` refuses `{"quality":"unknown","value":{"reason":…,"value":"0"}}` | red |
| `adv_probes.rs::adv_unit_alphabet_admits_what_parse_ucum_admits` | `(m/s)/s`, `[m/s2/Hz^(1/2)]`, `{a,b}` (UCUM and `parse_ucum` admit them) fit the `Unit` alphabet read from `spec/domains/definition.yaml` | red |
| `adv_probes.rs::adv_parse_ucum_never_panics` | no panic on every string ≤4 chars over `()[]{}./*^+-0129mskBy%' µ`, nor on 200 000 fixed-seed strings ≤15 chars over ASCII 0–127 plus `µ é €`, and every admitted text is ASCII 33–126 | green |
| `adv_probes_depth.rs::adv_parse_ucum_returns_on_deep_nesting` | `parse_ucum` returns (Ok or Err) on 500 000 nested `(`…`)` around `m` | red (SIGABRT) |

`cargo test --locked -p thalamus-core --test adv_probes`, exit 101:

```
test adv_model_decoder_refuses_value_under_unknown ... FAILED
test adv_ucum_admits_atm_and_pound_force ... FAILED
test adv_ucum_admits_square_brackets_inside_annotation ... FAILED
test adv_ucum_refuses_prefixes_outside_ucum_2_2 ... FAILED
test adv_ucum_refuses_prefix_on_non_metric_att_and_barn ... FAILED
test adv_unit_alphabet_admits_what_parse_ucum_admits ... FAILED
test adv_parse_ucum_never_panics ... ok
---- adv_model_decoder_refuses_value_under_unknown stdout ----
thread 'adv_model_decoder_refuses_value_under_unknown' (2095240) panicked at crates/thalamus-core/tests/adv_probes.rs:77:5:
the model decoder admitted an unknown reading carrying a value: Ok(Unknown(UnreadReading { reason: "source timed out" }))
---- adv_ucum_admits_atm_and_pound_force stdout ----
thread 'adv_ucum_admits_atm_and_pound_force' (2095242) panicked at crates/thalamus-core/tests/adv_probes.rs:49:5:
refused UCUM 2.2 atoms: ["atm", "[lbf_av]", "[lbf_av]/[sin_i]"]
---- adv_ucum_admits_square_brackets_inside_annotation stdout ----
thread 'adv_ucum_admits_square_brackets_inside_annotation' (2095243) panicked at crates/thalamus-core/tests/adv_probes.rs:63:5:
refused annotations UCUM § 6 admits: ["{a[b]}", "m{[x]}", "{]}"]
---- adv_ucum_refuses_prefixes_outside_ucum_2_2 stdout ----
thread 'adv_ucum_refuses_prefixes_outside_ucum_2_2' (2095245) panicked at crates/thalamus-core/tests/adv_probes.rs:22:5:
admitted prefixes UCUM 2.2 does not define: ["Rm", "Qg", "rm", "qs"]
---- adv_ucum_refuses_prefix_on_non_metric_att_and_barn stdout ----
thread 'adv_ucum_refuses_prefix_on_non_metric_att_and_barn' (2095244) panicked at crates/thalamus-core/tests/adv_probes.rs:36:5:
admitted a prefix on an atom UCUM 2.2 marks non-metric: ["katt", "matt", "mb", "kb"]
---- adv_unit_alphabet_admits_what_parse_ucum_admits stdout ----
thread 'adv_unit_alphabet_admits_what_parse_ucum_admits' (2095246) panicked at crates/thalamus-core/tests/adv_probes.rs:114:5:
UCUM units parse_ucum admits fall outside the Unit alphabet "!#$%&'*+-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~": ["(m/s)/s", "[m/s2/Hz^(1/2)]", "{a,b}"]
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
```

`cargo test --locked -p thalamus-core --test adv_probes_depth`, exit 101:

```
running 1 test

thread 'adv_parse_ucum_returns_on_deep_nesting' (2097006) has overflowed its stack
fatal runtime error: stack overflow, aborting
  process didn't exit successfully: `…/target/debug/deps/adv_probes_depth-d5a2aec76f169909` (signal: 6, SIGABRT: process abort signal)
```

## 3. Suite

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, exit 201. `specify validate`, `drift`, `fmt --check` and `clippy -D warnings` passed. `test` failed at the first red binary, so `docs-check` did not run:

```
     Running tests/adv_probes.rs (target/debug/deps/adv_probes-ea19b91a0bea5efc)
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
error: test failed, to rerun pass `-p thalamus-core --test adv_probes`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

To get the count, I then ran `cargo test --locked --workspace --no-fail-fast` (exit 101). Every other binary was green, including `tests/probes.rs` (`test result: ok. 4 passed`). Executed: 135 cases outside my two files (the sum of every other `test result` line in the same run) and 143 with them (7 in `adv_probes`, plus 1 in `adv_probes_depth`, which aborted). Full logs: `scratch/adversary/task-check.log` and `suite-nofailfast.log`.

## 4. Findings (tree: impl/m0-probes working tree over 5d0ee31)

| # | file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/thalamus-core/src/unit.rs:103 | `PREFIXES` holds R, Q, r, q. UCUM 2.2 defines 24 prefixes and none of these four. Fix: delete them | NEEDS-CHANGE | introduced | the story's Work line ("prefixes checked against an embedded table of UCUM case-sensitive codes"). No runtime caller yet; `story:m1-instance-registry` will be the first |
| 2 | crates/thalamus-core/src/unit.rs:420 | `("b", true)` and `("att", true)`: the essence has `isMetric="no"` for both, so `mb` and `katt` are admitted. Fix: `false` | NEEDS-CHANGE | introduced | same as 1 |
| 3 | crates/thalamus-core/src/unit.rs:108 | `ATOMS` lacks `atm` (non-metric) and `[lbf_av]` (non-metric), so both are refused. Fix: add them | NEEDS-CHANGE | introduced | same as 1 |
| 4 | crates/thalamus-core/src/unit.rs:463 | `enclosed_char` excludes `[` and `]` inside annotations. UCUM § 6 ■1 admits 33–126 there; only nested braces are excluded (■5). Fix: give annotations their own predicate that excludes only `{` and `}` | NEEDS-CHANGE | introduced | same as 1 |
| 5 | docs/design/thalamus.md:355 | The new § 8 text says an unknown reading carrying a value "is refused on the wire" by both generated crates. The model crate's decoder (`generated/thalamus-model/src/server/wire.rs:1298`, pre-existing) ignores extra members: it admits `{"quality":"unknown","value":{"reason":…,"value":"0"}}` and drops the value. The probes tests decode only through `thalamus-spec-types`. The in-memory `Reading::Unknown` still carries no value, so the invariant holds and only the claim is false | NEEDS-CHANGE | introduced | `POST /samples/commands/record` → `decode_command_thalamus_observation_record_sample` → `decode_thalamus_observation_reading` (`generated/thalamus-model/src/server/thalamus_service.rs:1913`, `wire.rs:3530`) |
| 6 | spec/domains/definition.yaml:38 | The `Unit` alphabet omits `(`, `)`, `,`, `"` and `\`. UCUM and `parse_ucum` admit `(m/s)/s`, the atom `[m/s2/Hz^(1/2)]` and `{a,b}`, so the "character set" the UNMAPPED comment and the `unit.rs` module doc describe is narrower than UCUM. Fix: widen the alphabet to 33–126, which moves the digest, so needs-coordinator (a) | CONFIRMED | pre-existing | the alphabet is identical at 5d0ee31 (`git show`). Neither generated decoder enforces it today (`wire.rs:638`, spec-types `ThalamusDefinitionUnit` is `transparent`) |
| 7 | crates/thalamus-core/src/unit.rs:515 | Recursion is unbounded: `component` → `term` → `component` per `(`. 500 000 levels overflow the 2 MiB test-thread stack and abort the process (SIGABRT, not a panic). The generated JSON parser bounds its own recursion (`DEPTH` = 64); the server admits 1 MiB bodies (`MAX_BODY`). Fix: a depth bound returning `UnitError` | INFEASIBLE | introduced | nothing calls `parse_ucum` today. It becomes reachable when `story:m1-instance-registry` validates a proposed definition's unit |

## 5. Attacked and not broken

- `unknown_reading_carries_no_value` against spec-types (read, not mutated): case 1 is refused only through `deny_unknown_fields` on `ThalamusObservationUnreadReading` (`types.rs:899`). Dropping that attribute turns case 1 Ok, so the test fails. The re-serialisation assertion compares against a literal. The test can fail.
- Panics: none, over the exhaustive 4-char sweep and 200 000 random strings including multi-byte characters (`adv_parse_ucum_never_panics`, green). Every slice index is at an ASCII boundary.
- Read, not run: `kmin`, `kh` and `k[in_i]` are refused. `10*3`, `10*-3`, `10^3`, `m+2` and `m-1` are admitted. `m^2`, `m*2`, `//s`, `s/`, `.s`, `M` and `MM` are refused, `Mm` is admitted. `%` is admitted and `m%` refused. `{request}/s` is admitted. Nested braces, whitespace, non-ASCII and an annotation after a factor or `)` are refused (per the BNF).
- `ATOMS` against the essence: the other 306 codes and their metric flags match. `[FNU]` is correct.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/essence/ucum-essence.xml (UCUM 2.2 essence, downloaded)
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/essence/ucum-spec.html (ucum.org specification page, downloaded)
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/essence-oneline.txt
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/ucum-atoms.txt
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/ucum-sorted.txt
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/impl-sorted.txt
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/adv_probes.log
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/adv_probes_depth.log
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/task-check.log
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary/suite-nofailfast.log
- ~/.cache/thalamus-waves/20261015c/m0-probes/scratch/adversary-pass-1.md (this report)

The worktree session lease `adversary-m0-probes-p1-20261015c` was acquired and is released when this report returns.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/unit.rs", "line": 103, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "PREFIXES admits R, Q, r and q, which UCUM 2.2 does not define, so Rm, Qg, rm and qs parse."},
  {"file": "crates/thalamus-core/src/unit.rs", "line": 420, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "b (barn) and att are flagged metric although the UCUM 2.2 essence marks both isMetric=no, so mb and katt parse."},
  {"file": "crates/thalamus-core/src/unit.rs", "line": 108, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "ATOMS lacks the UCUM 2.2 atoms atm and [lbf_av], so both are refused."},
  {"file": "crates/thalamus-core/src/unit.rs", "line": 463, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Annotations refuse square brackets although UCUM section 6 admits every character 33-126 except curly braces inside them."},
  {"file": "docs/design/thalamus.md", "line": 355, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Design section 8 claims both generated crates refuse an unknown reading carrying a value, but the model server decoder behind POST /samples/commands/record admits it and drops the value."},
  {"file": "spec/domains/definition.yaml", "line": 38, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "The Unit alphabet omits parentheses, comma, double quote and backslash, so it refuses UCUM units parse_ucum admits such as (m/s)/s and [m/s2/Hz^(1/2)]."},
  {"file": "crates/thalamus-core/src/unit.rs", "line": 515, "category": "boundary", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "Unbounded recursion per parenthesis aborts the process with a stack overflow at 500000 levels; no caller exists until story:m1-instance-registry."}
]
```
