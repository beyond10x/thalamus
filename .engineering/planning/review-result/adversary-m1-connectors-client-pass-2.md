---
format: aep.planning-md/3
id: review-result:adversary-m1-connectors-client-pass-2
kind: review-result
status: active
title: Adversary, story:m1-connectors-client, pass 2 (home paths shortened to ~)
relations:
- reviews: story:m1-connectors-client
revision: 1
---
unit: story:m1-connectors-client pass 2, uncommitted tree `impl/m1-connectors-client` (base b863833) at ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m1-connectors-client
verdict: red — two NEEDS-CHANGE (the double's new refusals differ from the real CLI) and one NEEDS-CHANGE contract drift (the F6 fix contradicts the spec)
cases: executed 14→16, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (this report file)
needs-coordinator: yes — F9 needs a change in `spec/`, which this unit does not own, or my pass-1 F6 case has to be dropped

**1. What I touched.** `git --no-pager diff --stat` shows `13 files changed, 1082 insertions(+), 2 deletions(-)`. That is all the implementor's work; I edited no implementation file. I added two untracked test files:
- `crates/thalamus-sources/tests/adv2_connectors_client.rs`
- `crates/thalamus-sources/tests/fixtures/connectors/adv2-described-twice/operations+describe+a=metrics+o=series.query.json`

**2. Cases added.** Both are in `adv2_connectors_client.rs`. Each was run alone first, and each was red (exit 101).

| Case (line) | Asserts | Red output, verbatim |
|---|---|---|
| `adv2_double_description_refusals_are_at_observation` (:68) | The double's `stale_description` and `description_unavailable` refusals carry stage `observation` and next action `refresh_description`, as the real CLI's do | `left: (Some("admission"), Some("describe_operation"))` / `right: (Some("observation"), Some("refresh_description"))` |
| `adv2_double_admits_only_the_current_description` (:96) | Describe answers `op-rev-1` on the 1st call and `op-rev-2` on the 2nd; an invoke at `op-rev-1` after that is not answered `ok` | `op-rev-1 is no longer the description: {"ok":true,"result":{...}}` |

**3. Suite run, after the cases existed.**
- `cargo test --locked -p thalamus-sources --no-fail-fast`, exit 101:
  - `adv2_connectors_client`: `FAILED. 0 passed; 2 failed`
  - `adv_connectors_client`: `ok. 6 passed`
  - `connectors_client`: `ok. 8 passed`
  - lib, bin and doc-tests: 0 tests each
- The same with `adv2` deselected: exit 0, 6 + 8 = 14 passed. That is the source of the 14 in `before`.
- Clippy with `-D warnings`: clean. rustfmt check on `adv2`: exit 0. `b10x-gates scan-text` on my 2 files: exit 0.
- I did not run `task check`.

**4. Findings: what reaches each one.**
- **F7** `src/bin/connectors-double.rs:229`: the double's description refusal says stage `admission`, next action `describe_operation`. The real CLI maps `StaleDescription | DescriptionUnavailable` to `("observation", "refresh_description")` (`~/beyond10x/connectors/apps/connectors/src/local.rs:178`).
  - Reached by every adapter test that asserts a stale-description reason. The client retries only `not_granted` and `timeout` at `admission`, so its behaviour does not change today.
  - Risk: a later "retry at admission" rule would pass against the double and do nothing against the real CLI.
- **F8** `src/bin/connectors-double.rs:199-212`: the double admits any (schema, revision) pair that any successful describe answer in the recording ever named. The real CLI admits only the current descriptor revision (`crates/connectors-host/src/local/owner.rs:373-377`).
  - Reached by a test that records a description moving between describe and invoke. None exists yet, and the double cannot express that case.
- **F9** `src/adapter.rs:106`: since the F6 fix, the digest is SHA-256 of the whole map's canonical JSON. The spec still says "SHA-256 of the exact query text" (`spec/domains/observation.yaml:38`), and design § 2.2 says "the hash of the exact query".
  - Reached by every sample's `provenance.query_sha256`. Hashing the query text with `sha256sum` no longer reproduces it: the test constant moved from `6d986bf1…` to `dc001421…`.
  - No digest can satisfy both the spec text and injectivity (my pass-1 F6), because any byte string can be a query text. So either the spec summary changes or F6's case goes.
  - My call (reversible): amend the `QueryDigest` summary to say the digest is of the query map's canonical JSON. That edit is in `spec/`, which is not this unit's.

**5. Attacked and could not break.**
- Escaping injectivity: `+`, `=` and `%` are always escaped inside a value; tags appear in a fixed order and only after the words. No two calls that differ in adapter, connection or operation map to one name. The one exception I found is a flag value equal to a flag name (`--connection --operation`), which the real CLI would refuse anyway.
- Paging loop: a repeated cursor gives `Unavailable`, as does a non-text cursor or a page without `connections`. The connection is found on the first page that holds it. The real cursor flag is `--cursor`. Registry cursors are ids (`new_id()`), so none starts with `-`.
- New `Unavailable` paths: a null, numeric or missing `result` and a non-boolean `ok` each become `Unavailable`, and the registry turns that into one unknown read. I found no path where one becomes 0 or `Ok`.
- An `ok:true` answer with a non-zero exit is still accepted. The real CLI never produces it, so I did not write a case.
- Revalidation: the double ignores `--expected-revision`, but `lapsed_evidence_revalidates_once_then_retries` asserts `conn-rev-1` as a literal, so the client side is covered.

**6. Paths written outside the worktree.** `~/.cache/thalamus-waves/20261015b/m1-connectors-client/scratch/adversary-pass-2.md` (this report). The pass-1 paths are unchanged. Test runs wrote under the worktree's own `target/tmp/adv2_connectors_client/`.

```findings
[
{"file":"crates/thalamus-sources/src/bin/connectors-double.rs","line":229,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the double refuses stale_description/description_unavailable at stage admission with next_action describe_operation; the real CLI answers stage observation, next_action refresh_description"},
{"file":"crates/thalamus-sources/src/bin/connectors-double.rs","line":199,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the double admits an invoke at any revision any recorded describe answer ever named, while the real CLI admits only the current descriptor revision"},
{"file":"crates/thalamus-sources/src/adapter.rs","line":106,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"query_digest now hashes the canonical JSON of the query map while spec QueryDigest says SHA-256 of the exact query text; the spec summary or the F6 case must change"}
]
```
