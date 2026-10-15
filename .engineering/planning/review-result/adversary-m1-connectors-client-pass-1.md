---
format: aep.planning-md/3
id: review-result:adversary-m1-connectors-client-pass-1
kind: review-result
status: active
title: Adversary, story:m1-connectors-client, pass 1 (home paths shortened to ~)
relations:
- reviews: story:m1-connectors-client
revision: 1
---
unit: story:m1-connectors-client, uncommitted tree `impl/m1-connectors-client` (base b863833) at ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15b-m1-connectors-client
verdict: red — two NEEDS-CHANGE (the recorded double answers calls the real `connectors` refuses), four INFEASIBLE
cases: executed 8→14, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (empty scratch directory)
needs-coordinator: none

**1. What I touched.** `git --no-pager diff --stat` shows only the implementor's 13 tracked files, with the same line counts as at the start. I edited no implementation file. My additions are untracked and all are test files:
- `crates/thalamus-sources/tests/adv_connectors_client.rs`
- `crates/thalamus-sources/tests/fixtures/connectors/adv-paged/connections-list.metrics.json`
- `crates/thalamus-sources/tests/fixtures/connectors/adv-no-document/operations-invoke.metrics.metrics-main.series.query.json`

I did not acquire a worktree lease; the brief named none.

**2. Cases added.** All are in `adv_connectors_client.rs`. Each was run alone first, and each was red (exit 101) for the reason claimed.

| Case (line) | Asserts | Red output, verbatim |
|---|---|---|
| `adv_connection_on_a_later_listing_page_is_found` (:163) | A connection listed on page 2 (`next_cursor: "c2"`) is found and read | `Err(NoConnection { adapter: "metrics", connection: "metrics-main" }); calls [["connections", "list", "--adapter", "metrics", "--output", "json"]]` |
| `adv_success_without_adapter_document_is_not_ok` (:189) | `ok:true` with no `result` gives `Unavailable`, and the registry returns 1 unknown read | `Ok(Object {"adapter": String("metrics"), "operation": String("series.query"), "revision": String("op-rev-1")})` |
| `adv_double_refuses_invoke_with_stale_description` (:218) | The double does not answer an invoke whose `--revision` or `--schema` is not the described one | revision `conn-rev-1` answered `{"ok":true,...}` |
| `adv_double_answers_json_only_when_asked` (:258) | The double does not answer JSON without `--output json` | answered `{"ok":true,"result":{...}}` |
| `adv_double_does_not_confuse_connection_and_operation` (:282) | `--connection metrics-main.series --operation query` is not answered from the `metrics-main`/`series.query` recording | answered `{"ok":true,...}` |
| `adv_query_digest_separates_one_text_from_a_map` (:316) | `{a:x,b:y}` and `{value:'{"a":"x","b":"y"}'}` get different digests | both `9280388d…eda3` |

**3. Suite run, after the cases existed.**
- `cargo test --locked -p thalamus-sources --no-fail-fast`, exit 101:
  - `adv_connectors_client`: `test result: FAILED. 0 passed; 6 failed`
  - `connectors_client`: `test result: ok. 8 passed; 0 failed`
  - lib, bin and doc-tests: 0 tests each
- The same with my file deselected (`--lib --bins --test connectors_client`): exit 0, `8 passed`. That is the source of the 8 in `before`.
- Clippy with `-D warnings`: clean. My file is rustfmt-clean. `b10x-gates scan-text` on my 3 files: exit 0.

**4. Findings: what reaches each one.**
- **F1** `src/connectors.rs:153`: the client reads only the first page of `connections list`.
  - Reached by the real CLI's paging (default limit 100, `next_cursor`; `~/beyond10x/connectors/apps/connectors/src/local/connections.rs:45-69`).
  - I found no adapter instance with more than 100 connections, so the verdict is INFEASIBLE.
  - Effect when it happens: a connection that exists always reads unknown, with a wrong reason.
  - Fix: follow `next_cursor`.
- **F2** `src/connectors.rs:321`: the `_ => Ok(answer)` fallback turns a success that carries no adapter document into `Ok`. Through an adapter the series is then dropped (0 reads, no unknown read).
  - The real CLI always includes `result` on success (`apps/connectors/src/local/operations.rs:243-244`), so INFEASIBLE.
- **F3** `src/bin/connectors-double.rs:101`: the recording name ignores `--schema` and `--revision`. The real CLI refuses a mismatch with `stale_description` (`crates/connectors-host/src/local/owner.rs:373-377`).
  - Reached by every adapter story's tests, because they all run against this double.
  - Not measured, my reasoning only: if the client passed the connection revision as `--revision`, all 8 tests would stay green. `argv_and_env_carry_no_credential` accepts both `op-rev-1` and `conn-rev-1` as values.
- **F4** `src/bin/connectors-double.rs:101`: the double ignores `--output`. The real default is human text (`apps/connectors/src/local/session.rs:50-57`), so a client that drops `--output json` stays green. Same reach as F3.
- **F5** `src/bin/connectors-double.rs:108`: the recording name joins adapter, connection and operation with `.`, and `ConnectionId` allows `.`, `:` and `/`, so two different calls can read the same recording. No current test uses a dotted connection id, so INFEASIBLE.
- **F6** `src/adapter.rs:103`: for a multi-entry query, the canonical JSON is hashed in the same space as the plain text of a one-entry query. Reaching a collision needs a query text that is exactly sorted compact JSON; none found, so INFEASIBLE.

**5. Attacked and could not break.**
- Operation allow-list: exact match. Case variants, prefixes, `_range`, the empty string and trailing whitespace are all refused.
- Path traversal in fixture names: `/` becomes `_`, so `..` cannot escape the directory.
- Credentials: none enters the argv, a reason or provenance. A reason does carry the binary path (`connectors.rs:128`).
- Refusals and timeouts: `timeout` and `not_granted` at admission each retry once and then refuse. An unconfirmed revalidation becomes a typed refusal and then an unknown read.
- Window parsing: unit order, repeated units, `M` in the date part, fractions, `P`/`PT` with no number, and overflow are all refused.
- Registry prefix near-misses all return `None`.
- `QUERY_SHA256` matches the `sha256sum` of `QUERY`.

Two points I did not turn into cases:
- There is no deadline on the child process (`connectors.rs:122`), and the specification asks for none.
- A `ConnectionId` starting with `-` is passed as its own argv token. I could not build a path to a write from it.

**6. Paths written outside the worktree.** `~/.cache/thalamus-waves/20261015b/m1-connectors-client/scratch/adversary/` (created, empty). Test runs wrote under the worktree's own `target/tmp/adv_connectors_client/`.

```findings
[
{"file":"crates/thalamus-sources/src/connectors.rs","line":153,"category":"boundary","severity":"warning","verdict":"INFEASIBLE","origin":"introduced","message":"connections list is read one page only; a connection past the first page (real CLI pages at 100 via next_cursor) always reads unknown as NoConnection"},
{"file":"crates/thalamus-sources/src/connectors.rs","line":321,"category":"acceptance","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"a success envelope without an adapter document is returned as Ok (the wrapper or Null), so an adapter drops the series instead of reading unknown"},
{"file":"crates/thalamus-sources/src/bin/connectors-double.rs","line":101,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the double answers invoke with a stale --schema/--revision the real CLI refuses as stale_description, so no test can catch a client sending the wrong revision"},
{"file":"crates/thalamus-sources/src/bin/connectors-double.rs","line":101,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the double answers JSON without --output json while the real CLI defaults to human output, so dropping --output json stays green"},
{"file":"crates/thalamus-sources/src/bin/connectors-double.rs","line":108,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"recording names join adapter, connection and operation with '.', which ConnectionId and operation ids contain, so distinct calls read one recording"},
{"file":"crates/thalamus-sources/src/adapter.rs","line":103,"category":"property","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"query_digest hashes one-entry text and multi-entry canonical JSON in the same space, so a text equal to another map's canonical JSON collides"}
]
```
