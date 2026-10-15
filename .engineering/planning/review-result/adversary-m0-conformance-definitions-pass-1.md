---
format: aep.planning-md/3
id: review-result:adversary-m0-conformance-definitions-pass-1
kind: review-result
status: active
title: Adversary, story:m0-conformance-definitions, pass 1 (home paths shortened to ~)
relations:
- reviews: story:m0-conformance-definitions
revision: 1
---
```
unit: story:m0-conformance-definitions, uncommitted working tree on impl/m0-conformance-definitions (base b863833)
verdict: CONFIRMED (2 warnings, harness snapshot steps), plus 1 note
cases: executed 7→19, red 4
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 path (the scratch directory, part 6)
needs-coordinator: the implementor needs to fix the snapshot steps (the fix is named in part 4); the 4 red cases stay red until then
```

The behaviours hold up: no wrong outcome and no forbidden lifecycle move. The 51-scenario green is honest for outcomes, events and views. Where the harness fails is its complete-subject snapshot. It reports green when the subject is missing, and it never checks the declared shape. So the two "leaves the row as the first call made it" scenarios pass even when the views serve nothing.

**1. git diff --stat** (my change is one new test file; it does not show in `--stat`)
```
 Cargo.lock                                       |  155 +++
 crates/thalamus-core/Cargo.toml                  |    2 +-
 crates/thalamus-core/src/behaviour/definition.rs |  201 ++++
 crates/thalamus-core/src/behaviour/dimension.rs  |   71 ++
 crates/thalamus-core/src/conformance.rs          | 1402 ++++++++++++++++++++++
 crates/thalamus-core/tests/conformance.rs        |  208 ++++
 6 files changed, 2038 insertions(+), 1 deletion(-)
?? crates/thalamus-core/tests/adv_conformance.rs   (mine, the only path I touched)
```

**2. Cases added** in `~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m0-conformance-definitions/crates/thalamus-core/tests/adv_conformance.rs`. Each was run alone first; that output is in `cases-alone.log`.

| case | asserts | now |
|---|---|---|
| `a_snapshot_of_a_subject_the_view_does_not_serve_fails` :102 | a snapshot that matches 0 rows fails, as the ESS runner requires (`runtime.ts:4321-4327`) | red: `left: 0 right: 1` |
| `a_snapshot_row_lacking_a_declared_field_fails` :121 | a row missing a required field the shape declares fails (`response.ts:625` `admitSubjectRow`) | red: `left: 0 right: 1` |
| `a_snapshot_row_breaking_a_declared_field_type_fails` :137 | a string `title` declared `Integer` fails | red: `left: 0 right: 1` |
| `views_that_serve_nothing_turn_the_already_scenarios_red` :310 | with storage whose `list()` returns nothing, the real `ProposeDefinition/outcome/already-defined` and `RegisterDimension/outcome/already-registered` must fail | red: `...already-defined passed over a planted defect; failed were:` (an empty list) |
| `lost_definition_updates` / `a_dropped_parent` / `a_hidden_retired_definition` `_turn_the_run_red` | each defect planted in storage turns the named real scenarios red | green: the harness catches these |
| `a_revised_active_definition_stays_active`, `a_revised_muted_definition_stays_muted_with_its_mute`, `the_instant_guard_precedes_the_retired_subject`, `a_second_revision_counts_from_the_first`, `a_revision_is_stamped_with_its_own_instant` | revise behaviour the 51 scenarios never reach | green now; red on the mutants below |

Mutants, run on a scratch copy (the tree itself was not edited). The 51 scenarios stayed green under M1 to M4.

| mutant | 51 scenarios | caught by |
|---|---|---|
| M1: revise resets the state to Proposed | 0 failed | active and muted cases |
| M2: revise keeps the old `revised_at` | 0 failed | stamp case |
| M3: `retired` checked before `instant-not-fixed` | 0 failed | precedence case |
| M4: revise refuses any subject that is not Proposed | 0 failed | active and muted cases |
| M5: a missing parent is accepted | **1 failed** | the 51 scenarios themselves |

**3. Package test run, after the cases existed**: `cargo test --locked --no-fail-fast -p thalamus-core`, exit 101.
```
test result: ok. 1 passed; 0 failed; ...                    (lib)
test result: FAILED. 8 passed; 4 failed; ...                (adv_conformance)
test result: ok. 1 passed; 0 failed; ...                    (adv_conformance_feature)
test result: ok. 5 passed; 0 failed; ...                    (conformance)
test result: ok. 0 passed; 0 failed; ...                    (probes, doc)
```
The "before" count of 7 comes from the same run with `adv_conformance` left out (exit 0). Clippy `-D warnings` and `fmt --check` are clean, and `b10x-gates scan-text` on my file reports `1 file(s) carry no private rule`.

**4. Findings**
- `crates/thalamus-core/src/conformance.rs:798,831` (warning, CONFIRMED, introduced): when no row matches, the snapshot stores `None`, and `(None, None) => true` then passes "unchanged". What reaches it: the two real scenarios above pass with storage whose views are empty. `run` is the harness the Eventlog-storage stories (m1-eventlog-store, m2-*) will reuse; the in-memory store does not reach this today. Fix: fail unless exactly one row matches, at both the snapshot and the "unchanged" step.
- `crates/thalamus-core/src/conformance.rs:791-807` (warning, CONFIRMED, introduced): the step accepts the shape's field `type`s and `declarations` but never reads them. It checks only for extra fields, so a missing required field or a wrongly typed one passes. What reaches it: the 20 snapshot steps in the selected suite; today's in-memory rows conform, so no result changes now. Fix: check each declared field for presence and type, as `admitSubjectRow` does. The same "accepted but never read" pattern covers `capture_instance.entity` (:415).
- `spec/suite.json` (note, CONFIRMED, pre-existing): the scenarios ESS generates never revise from Active or Muted, never check `revised_at`, and never send an instant body to a retired definition. M1 to M4 survive the 51 scenarios. The harness's fixed clock (`conformance.rs:41`) would also hide M2 even if `revised_at` were checked. My five revise cases now catch these mutants.

**5. Attacked and could not break**
- The selection: `ran == wanted` is computed independently, and the count is 51 by `jq`.
- Unknown steps, members, kinds and predicate forms are refused, not skipped.
- `expect_outcome` compares the command as well as the outcome; view `contains`/`excludes`/`satisfies` are all compared.
- The precedence orders in ProposeDefinition, ReviseDefinition and RegisterDimension match PLAN.md and the suite's own precedence probes.
- A missing parent is refused (M5 goes red).
- Lost updates, a dropped parent and a hidden retired definition are each caught.

**6. Paths written outside the worktree**
- `~/.cache/thalamus-waves/20261015b/m0-conformance-definitions/scratch/adversary/` (173M), which holds:
  - `mut-tree/`: the mutated copy, including its own `target/`
  - `run-mutant.sh`
  - `cases-alone.log`, `suite-after.log`, `suite-deselected.log`
  - `mutant-m0..m5-*.log`

**7.**
```findings
- file: crates/thalamus-core/src/conformance.rs
  line: 831
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: snapshot_complete_subject accepts a subject no row carries and expect_complete_subject_unchanged passes None==None, so already-defined and already-registered pass over storage whose views serve nothing
- file: crates/thalamus-core/src/conformance.rs
  line: 791
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the complete-subject shape's field types and declarations are admitted but never applied, so a row missing a required declared field or breaking its type passes where the ESS reference runner fails it
- file: spec/suite.json
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: the synthesized scenarios never revise from Active or Muted, never read revised_at and never send an instant body to a retired definition, so mutants M1-M4 in revise_definition survive the 51-scenario run; adv_conformance.rs now catches each
```
