---
format: aep.planning-md/3
id: review-result:adversary-m3-kubernetes-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m3-kubernetes-adapter, pass 2
relations:
- reviews: story:m3-kubernetes-adapter
revision: 1
---
unit: story:m3-kubernetes-adapter, uncommitted working tree of impl/m3-kubernetes-adapter (base c2897fe) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-kubernetes-adapter, after the implementor's F1/F2 correction
verdict: NEEDS-CHANGE
cases: executed 29→32, red 3
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (suite log and this report, listed in part 6)
needs-coordinator: none

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/kubernetes.rs | 372 +++++++++++++++++++++++++++++-
 1 file changed, 368 insertions(+), 4 deletions(-)
```

That line is the implementor's uncommitted change. I did not touch it. In this pass I wrote only to `crates/thalamus-sources/tests/adv_kubernetes.rs`, my own pass-1 file, which is untracked: I added three `adv2_` cases and a helper, and ran rustfmt on that one file. No implementation path was written.

## 2. Cases added (`crates/thalamus-sources/tests/adv_kubernetes.rs`, section "Pass 2")

Each case runs its spellings over `per-namespace`. In that fixture alpha has 3 Running pods and 1 Pending. The case asserts that alpha reads either the count the spelling means or `unknown`. A complete count of a different question fails the case.

| case | spellings | now |
|---|---|---|
| `adv2_regex_glob_and_alternation_values_are_never_a_complete_count` | `status.phase=~Running`, `status.phase=~Running\|Pending`, `status.phase=Running\|Pending`, `metadata.name=api-*`, `status.phase=Run.*` | red: all 5 read `ok 0` |
| `adv2_whitespace_and_invisible_characters_in_values_are_never_a_complete_count` | `status.phase=Running Pending`, `…=Running or Pending`, `…=Running\u{200b}`, `…=Runn\u{0456}ng` (Cyrillic i), `…=Running\tPending` | red: all 5 read `ok 0` |
| `adv2_non_scalar_field_is_never_a_complete_count` | `status=Running`, `status!=Running`, `metadata=api-1` | red: `ok 0`, `ok 4`, `ok 0` |

Red output from the first run of these cases alone (`cargo test --locked -p thalamus-sources --test adv_kubernetes adv2_`), before rustfmt. rustfmt changed only whitespace, which moved the panic lines to 269, 288 and 306. A rerun after formatting gave the same 3 failures.

```
test adv2_non_scalar_field_is_never_a_complete_count ... FAILED
test adv2_whitespace_and_invisible_characters_in_values_are_never_a_complete_count ... FAILED
test adv2_regex_glob_and_alternation_values_are_never_a_complete_count ... FAILED

---- adv2_non_scalar_field_is_never_a_complete_count stdout ----
panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:300:5:
complete counts of another question: [
    "\"status=Running\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
    "\"status!=Running\" -> Ok(CompleteReading { value: Decimal(\"4\") }) (meant None)",
    "\"metadata=api-1\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
]
---- adv2_whitespace_and_invisible_characters_in_values_are_never_a_complete_count stdout ----
panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:285:5:
complete counts of another question: [
    "\"status.phase=Running Pending\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
    "\"status.phase=Running or Pending\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
    "\"status.phase=Running\\u{200b}\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"3\"))",
    "\"status.phase=Runnіng\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
    "\"status.phase=Running\\tPending\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant None)",
]
---- adv2_regex_glob_and_alternation_values_are_never_a_complete_count stdout ----
panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:269:5:
complete counts of another question: [
    "\"status.phase=~Running\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"3\"))",
    "\"status.phase=~Running|Pending\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"4\"))",
    "\"status.phase=Running|Pending\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"4\"))",
    "\"metadata.name=api-*\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"4\"))",
    "\"status.phase=Run.*\" -> Ok(CompleteReading { value: Decimal(\"0\") }) (meant Some(\"3\"))",
]
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.46s
```

Checks on the file: `rustfmt --check` exit 0 after formatting. `cargo clippy --locked -p thalamus-sources --test adv_kubernetes -- -D warnings` is clean. `b10x-gates scan-text` reports "1 file(s) carry no private rule".

## 3. Suite run (after the cases existed)

`cargo test --locked -p thalamus-sources --no-fail-fast`, exit 101. Full log: `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/suite-pass-2.log`.

```
     Running tests/adv2_connectors_client.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adv_connectors_client.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/adv_kubernetes.rs
test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
error: test failed, to rerun pass `-p thalamus-sources --test adv_kubernetes`
     Running tests/connectors_client.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/kubernetes.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
error: 1 target failed:
```

(The lib, bin and doc-test targets ran 0 tests each.)

The 29 is this same run with my three `adv2_` cases deselected: 2 + 6 + 5 + 8 + 8. All 5 pass-1 cases are now green, and so are the implementor's 8. The suite is red on 3 cases, which is the outcome this pass aims for.

## 4. Findings

**G1: the value check is a blocklist, and regex, glob and alternation spellings get past it, each reading a complete 0.** NEEDS-CHANGE, blocker, introduced. This is the residue of pass-1 F1, in the same function.
- **Measured:** `crates/thalamus-sources/src/kubernetes.rs:218` (`NOT_IN_VALUE`, 9 characters) does not include `~`, `|` or `*`. Red at `adv_kubernetes.rs:269`, 5 of 5 spellings.
- **What reaches it:** whoever writes a definition. `=~` is the regex matcher in PromQL and LogQL, the sources behind thalamus's sibling `prometheus` and `loki` adapters, so the same authors type it. `a|b` and `api-*` are the common ways to write "either" and "prefix".
- **Fix (not applied):** replace the blocklist with an allowlist of value characters, for example ASCII `[A-Za-z0-9._:/-]`, so that anything else reads unknown before any call. The same change closes G2.

**G2: whitespace and invisible characters inside a value are accepted, and the read is a complete 0.** NEEDS-CHANGE, warning, introduced.
- **Measured:** `kubernetes.rs:255` trims only the ends of the value. Interior spaces and tabs, U+200B and a Cyrillic look-alike all pass. Red at `adv_kubernetes.rs:288`, 5 of 5.
- **What reaches it:** "Running or Pending" as an author's attempt at "either". U+200B arrives by copy-paste from rendered docs or chat. The homoglyph is something I built and is not shown to occur; it rides on the same fix.

**G3: a field that is an object or a list is compared by its JSON text, which gives a count of a question that has no answer.** NEEDS-CHANGE, warning, introduced.
- **Measured:** `kubernetes.rs:280`, the branch `other => other.to_string()`. `status=Running` reads `ok 0` and `status!=Running` reads `ok 4` (every pod). Red at `adv_kubernetes.rs:306`.
- **What reaches it:** an author who names a container field, e.g. `status.conditions=Ready`, where `conditions` is a list in real pod objects.
- **Fix (not applied):** treat `Value::Object` and `Value::Array` as unknown ("field is not a scalar"), or refuse such a path once the first object shows it.

**G4: Kubernetes leaves zero-valued status counters out, and the story's "missing → status not projected" rule reads those as unknown.** INFEASIBLE, note, introduced. No case written, because a red case would contradict the story's own acceptance statement.
- **Measured:** `kubernetes.rs:272` (`Predicate::test`) returns `None` for an absent field.
- **Not verified in this tree:** Kubernetes `apps/v1` `DeploymentStatus` declares `readyReplicas`, `availableReplicas` and `unavailableReplicas` as `omitempty`. I know that from the Kubernetes API types; I did not observe it here, so it is inferred.
- **Effect:** `kind: deployments`, `predicate: status.unavailableReplicas!=0` would read unknown in every namespace whose deployments are all healthy, permanently, though the answer is a number. Changing this needs a story decision, not an implementor fix.

## 5. Attacked and not broken

- **Grammar holes asked about:** trailing dot (`status.phase.=`), a leading dot, `status..phase`, whitespace inside the path, a Unicode letter or digit in the path, `= =`, `! =`, `!==`, `===`, `a=b!=c`, and a 10,000-character value. Each is refused before any call or reads correctly; the long value reads `ok 0` because nothing has that value, and no other question is being answered.
- **`==`:** reads `=` (pass-1 case green, alpha `ok 3`).
- **Empty-reads guard against a legitimate zero-namespace definition:**
  - The guard fires only when the namespace listing finished, named no namespace and held no unnamed item.
  - A definition that declares no `namespace` dimension, including `dimensions: []`, is refused in `Count::of` before any call, so it never reaches the guard.
  - A cluster that does hold a configured namespace always produces a per-namespace read.
  - So no definition that should read a number gets that unknown (pass-1 case green).
- **Pass-1 cases:** all 5 are green.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/suite-pass-2.log`
- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary-pass-2.md` (this report)
- Worktree lease `adv2-m3-k8s-20261015c`, held through `worktree hook session-start` and released at return.
- Inside the worktree, not in git: `target/tmp/adv-kubernetes/adv2_*`.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 218, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the value check is a 9-character blocklist, so `=~Running`, `Running|Pending`, `api-*` and `Run.*` parse as literal values and read a complete 0 instead of unknown"},
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 255, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "interior whitespace, a tab, U+200B or a homoglyph in the value is accepted and reads a complete 0"},
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 280, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an object or list field is compared by its JSON text, so `status=Running` reads a complete 0 and `status!=Running` counts every object"},
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 272, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "Kubernetes omits zero-valued status counters (omitempty, inferred from the API types), so the story's missing-field rule reads predicates on them as unknown permanently where the answer is a number; needs a story decision"}
]
```
