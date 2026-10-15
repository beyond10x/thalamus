---
format: aep.planning-md/3
id: review-result:adversary-m3-kubernetes-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m3-kubernetes-adapter, pass 1
relations:
- reviews: story:m3-kubernetes-adapter
revision: 1
---
unit: story:m3-kubernetes-adapter, uncommitted working tree of impl/m3-kubernetes-adapter (base c2897fe) in ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-kubernetes-adapter
verdict: NEEDS-CHANGE
cases: executed 24→29, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (scratch logs and this report, listed in part 6)
needs-coordinator: none

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/kubernetes.rs | 338 +++++++++++++++++++++++++++++-
 1 file changed, 334 insertions(+), 4 deletions(-)
```

That line is the implementor's uncommitted change. I did not touch it. My additions are untracked, so `diff --stat` does not list them (`git status --short --untracked-files=all`):

```
?? crates/thalamus-sources/tests/adv_kubernetes.rs
?? crates/thalamus-sources/tests/fixtures/kubernetes/adv-no-namespace/operations+invoke+a=kubernetes+c=kubernetes-main+o=namespaces.list.json
?? crates/thalamus-sources/tests/fixtures/kubernetes/adv-repeated-cursor/operations+invoke+a=kubernetes+c=kubernetes-main+o=resources.list.json
```

All three are test files or fixtures. No implementation path was written.

## 2. Cases added (`crates/thalamus-sources/tests/adv_kubernetes.rs`)

| case | asserts | now |
|---|---|---|
| `adv_double_equals_predicate_is_never_a_complete_zero` | `status.phase==Running` over `per-namespace`: alpha reads `ok 3` or `unknown` | red: `ok 0` in every namespace |
| `adv_field_selector_conjunction_is_never_a_complete_zero` | `status.phase=Running,metadata.name=api-1`: alpha reads `ok 1` or `unknown` | red: `ok 0` in every namespace |
| `adv_quoted_predicate_value_is_never_a_complete_zero` | `status.phase="Running"`: alpha reads `ok 3` or `unknown` | red: `ok 0` in every namespace |
| `adv_no_configured_namespace_reads_unknown_not_nothing` | `namespaces.list` answers `items: []`, `complete: true`: at least one read, every read unknown | red: zero reads |
| `adv_repeated_continuation_is_partial_lower_bound_and_stops` | page 2 of alpha returns its own cursor again: alpha is `partial 3 lower`, after exactly 2 `resources.list` calls for alpha | green. It covers the repeated-cursor branch (`kubernetes.rs` `Value::String(next) =>` after `seen.insert`), which no existing case reaches. I did not run a mutant. |

Red output, captured when the cases were written, first run, this file alone (`cargo test --locked -p thalamus-sources --test adv_kubernetes`). The `all reads:` Debug tails are cut to alpha's read here; the full text is in the suite log, part 3.

```
running 5 tests
test adv_no_configured_namespace_reads_unknown_not_nothing ... FAILED
test adv_quoted_predicate_value_is_never_a_complete_zero ... FAILED
test adv_double_equals_predicate_is_never_a_complete_zero ... FAILED
test adv_field_selector_conjunction_is_never_a_complete_zero ... FAILED
test adv_repeated_continuation_is_partial_lower_bound_and_stops ... ok

---- adv_no_configured_namespace_reads_unknown_not_nothing stdout ----
thread 'adv_no_configured_namespace_reads_unknown_not_nothing' (581512) panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:198:5:
a definition read against a cluster holding none of its namespaces has no read at all

---- adv_quoted_predicate_value_is_never_a_complete_zero stdout ----
thread 'adv_quoted_predicate_value_is_never_a_complete_zero' (581513) panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:160:5:
predicate "status.phase=\"Running\"": alpha has 3 Running pods of 4 (api-1 Running), so the read is 3 or unknown, not Ok(CompleteReading { value: Decimal("0") }); all reads: [Read { dims: {"namespace": "alpha"}, reading: Ok(CompleteReading { value: Decimal("0") }), …

---- adv_double_equals_predicate_is_never_a_complete_zero stdout ----
thread 'adv_double_equals_predicate_is_never_a_complete_zero' (581510) panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:160:5:
predicate "status.phase==Running": alpha has 3 Running pods of 4 (api-1 Running), so the read is 3 or unknown, not Ok(CompleteReading { value: Decimal("0") }); all reads: [Read { dims: {"namespace": "alpha"}, reading: Ok(CompleteReading { value: Decimal("0") }), …

---- adv_field_selector_conjunction_is_never_a_complete_zero stdout ----
thread 'adv_field_selector_conjunction_is_never_a_complete_zero' (581511) panicked at crates/thalamus-sources/tests/adv_kubernetes.rs:160:5:
predicate "status.phase=Running,metadata.name=api-1": alpha has 3 Running pods of 4 (api-1 Running), so the read is 1 or unknown, not Ok(CompleteReading { value: Decimal("0") }); all reads: [Read { dims: {"namespace": "alpha"}, reading: Ok(CompleteReading { value: Decimal("0") }), …

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Checks on the added files: `rustfmt --edition 2024 --check` exit 0, `cargo clippy --locked -p thalamus-sources --test adv_kubernetes -- -D warnings` clean, `b10x-gates scan-text` "3 file(s) carry no private rule", exit 0.

## 3. Suite run (after the cases existed)

`cargo test --locked -p thalamus-sources --no-fail-fast`, exit 101. Full log: `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/suite-pass-1-nofailfast.log`.

```
     Running unittests src/lib.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/bin/connectors-double.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/adv2_connectors_client.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adv_connectors_client.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adv_kubernetes.rs
test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p thalamus-sources --test adv_kubernetes`
     Running tests/connectors_client.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/kubernetes.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```

Where `executed 24→29` comes from: 24 is the same run with `tests/adv_kubernetes.rs` deselected (2 + 6 + 8 + 8). The implementor's own count, 8 in `tests/kubernetes.rs`, is unchanged and still green. The suite is red on 4 cases, which is the outcome this pass aims for. An earlier run with fail-fast (`scratch/adversary/suite-pass-1.log`) stopped at `adv_kubernetes` and is superseded by this one.

## 4. Findings

**F1: a predicate the parser misreads becomes a complete count, not a refusal.** NEEDS-CHANGE, blocker, introduced. Covers the working tree.
- **Measured:** `crates/thalamus-sources/src/kubernetes.rs:212` (`Predicate::parse`) splits at the first `!=`, else at the first `=`. It accepts whatever is left as the value, including a leading `=`, `,`, or quotes. Each such spelling reads `ok 0` (complete) in every namespace instead of `unknown`. Red: `adv_kubernetes.rs:160`, three cases. This is lesson L5 again, an unparseable input becoming 0. Under § 3.1 a complete 0 grades as a real reading.
- **Reaches it:** whoever writes a definition. Kubernetes field selectors, the predicate syntax a Kubernetes operator already knows (`kubectl --field-selector`), allow `==` as equality and `,` as conjunction. So `status.phase==Running` and `status.phase=Running,spec.nodeName=…` are ordinary spellings. Nothing between the definition and this parser checks the text. `Count::of` is the only validation, and it calls this function.
- **Fix, not applied:** refuse at parse time any value that starts with `=`, or contains `,`, `=` or `"`. Accepting `==` as `=` is also enough to turn the first case green.

**F2: a namespace listing with no configured namespace yields no read at all.** NEEDS-CHANGE, warning, introduced. Covers the working tree.
- **Measured:** `kubernetes.rs:111`. If `namespaces.list` completes with `items: []`, `names` is empty, `unfinished` is `None` and `unnamed` is false, so `read` returns `Ok(vec![])`. Red: `adv_kubernetes.rs:198`. The definition then has no sample in the pass, so nothing reads UNKNOWN. The signal disappears silently instead of reading "not read".
- **Reaches it:** the real adapter. `~/beyond10x/connectors/adapters/kubernetes/src/lib.rs:479-481` (`configured_namespaces`) skips each configured namespace that answers 404 and returns `complete: next_cursor.is_none()`. A connection whose configured namespaces do not exist (yet, or any more) therefore answers exactly this page. No pipeline caller of `registry::read` exists yet in this tree, so the downstream effect of zero reads is inferred, not observed.
- **Fix, not applied:** when the listing is complete and `names` is empty, push one `unknown` read, for example "no configured namespace exists in the cluster".

**F3: optional fields cannot be used in a predicate.** INFEASIBLE, note, introduced. No case: the semantics are a design call, and a case would assert my preference.
- **Measured:** `kubernetes.rs:238` (`Predicate::test`) treats any absent path as "not projected". A predicate on a label, such as `metadata.labels.app=api`, therefore reads `unknown` whenever one listed object has no such label. A missing label is an ordinary non-match in Kubernetes label-selector semantics.
- **Measured, second part:** the dotted path cannot address label keys that contain dots (`app.kubernetes.io/name`), so those predicates are always `unknown`. The direction is safe (unknown, never a number), but the predicate grammar can count nothing keyed by a recommended Kubernetes label.
- **Reaches it:** any definition that counts by label. The story's examples only name `status` fields, so whether this is in scope is unclear. The coordinator should decide.

## 5. Attacked and not broken

- **Partial results:** an unfinished continuation (`complete: false` with a null cursor, a refused page 2, a repeated cursor) reads `partial` with bound `lower`, never `ok`. A first-page failure reads `unknown` with code and stage, never `partial 0`.
- **Missing status:** an absent, `null` or non-object `status`, or a missing `status.phase`, reads `unknown` "status not projected" for both `=` and `!=`. It is never 0 and never a match, including inside a partial list.
- **Dimensions (L9):** dims come only from the Namespace object's `metadata.name`. `tenant` and `cluster` labels on the namespace and on pods never reach `dims` (existing tenant test, read in code).
- **Instant counts (L8, § 3.1 rule 3):** the adapter keeps no state and computes no delta or trend; the kind is left to the definition.
- **Predicate edge cases that stay safe:** spaces around `=`/`!=` are trimmed and parse correctly. A value containing `=` after `!=`, or after the first `=`, parses correctly. `a=b!=c` mis-splits into the path `…x=a`, which reads `unknown`, not a number. `..` and an empty path are refused before any call.
- **Recorded answers against the real adapter:** the shapes match `adapters/kubernetes/spec/adapter.json`. Inputs are `namespace`, `kind`, `limit` ≤ 100 and `cursor`; outputs are `items`, `next_cursor`, `complete` and `provenance`. `complete: false` with a null cursor is a real answer (`lib.rs:322`, `Carried::Partial`). The error codes and stages exist in Connectors.
- **One deviation in the recordings:** the real `provenance.resource` is `<namespace>/<kind>` (`lib.rs:396`), while the fixtures record `pods`. The adapter does not read that field, so behaviour is unaffected.
- **Missing status in the real adapter:** `resources.list` clones the provider items unprojected (`lib.rs:374-382`), so a missing `status` is schema-legal (`design.md:21`) but not produced by today's adapter. The `missing-status` fixture is a constructed state, and that is fine for a conservative path.
- **Read-only:** only `namespaces.list` and `resources.list` are declared, so `pods.exec` is refused by the client before running.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/` (directory, created)
- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/suite-pass-1.log`
- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary/suite-pass-1-nofailfast.log`
- `~/.cache/thalamus-waves/20261015c/m3-kubernetes-adapter/scratch/adversary-pass-1.md` (this report)
- Also: the worktree lease `adv1-m3-k8s-20261015c`, recorded by `worktree hook session-start` in the worktree CLI's own state. It is released at return.
- Inside the worktree, not in git: `target/tmp/adv-kubernetes/` (test directories under `CARGO_TARGET_TMPDIR`).

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 212, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Predicate::parse accepts `==`, a `,` conjunction and a quoted value as a literal value, so a malformed predicate reads a complete 0 in every namespace instead of unknown"},
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 111, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a complete namespaces.list with no items (the real adapter's answer when every configured namespace is absent) returns zero reads, so the definition silently has no sample instead of an unknown"},
  {"file": "crates/thalamus-sources/src/kubernetes.rs", "line": 238, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "any absent predicate path reads unknown and dotted label keys cannot be addressed, so a count keyed by a label is unknown whenever one object lacks the label; whether label predicates are in scope is undecided"}
]
```
