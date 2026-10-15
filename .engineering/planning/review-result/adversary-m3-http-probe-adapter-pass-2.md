---
format: aep.planning-md/3
id: review-result:adversary-m3-http-probe-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m3-http-probe-adapter, pass 2
relations:
- reviews: story:m3-http-probe-adapter
revision: 1
---
unit: story:m3-http-probe-adapter, uncommitted working tree on impl/m3-http-probe-adapter over 369a6a1 (http_probe.rs sha256 6aae99b5…f83f8f, connectors.rs ae3bf868…74e08d)
verdict: CONFIRMED
cases: executed 215→218, red 0
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths, listed in part 6
needs-coordinator: none

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/adapter.rs    |   8 +
 crates/thalamus-sources/src/connectors.rs |  51 +++++-
 crates/thalamus-sources/src/http_probe.rs | 294 +++++++++++++++++++++++++++++-
 crates/thalamus-sources/src/registry.rs   |   6 +-
 docs/design/thalamus.md                   |   1 +
 5 files changed, 348 insertions(+), 12 deletions(-)
```

These tracked changes are the implementor's, the same as before this pass. I added one untracked file, a test: `crates/thalamus-sources/tests/adv2_http_probe.rs`. Its recordings are written from the test body into `target/tmp/adv2_http_probe/`, so it adds no fixture files. I touched no non-test path. Both source hashes in the header are the same before and after this pass. Every mutant ran on a scratch copy.

## 2. Cases added (`crates/thalamus-sources/tests/adv2_http_probe.rs`)

| case | asserts | now |
|---|---|---|
| `adv2_only_the_exact_catalog_read_profile_is_invoked` | 16 descriptions, each with an ok invoke recorded: profile `""`, `Generic-HTTP`, `GENERIC-HTTP`, `generic-http␠`, `␠generic-http`, `generic-http\n`, `generic-http-page`, `generic_http`, `generic-http\0`, `generic‑http` (U+2011), `null`, `true`, `7`, `["generic-http"]`, `{"id":"generic-http"}`, and `generic-http` beside `operation` instead of inside it. Each reads unknown with `raw` None, is described once, and is never invoked | green |
| `adv2_the_invoke_pins_the_description_it_checked` | describe call 1 is `generic-http` at `rev-read`/`schema-read`; call 2 is `mutation` at `rev-write`/`schema-write`. Expected: read 1, one describe, and one invoke carrying `--revision rev-read --schema schema-read` | green |
| `adv2_a_retry_checks_the_description_it_describes_again` | invoke 1 is refused `timeout` at `admission`, which the client retries; describe 2 is `mutation` at `rev-write`. Expected: unknown, `raw` None, 2 describes, 1 invoke at `rev-read` | green |

First run of the cases alone, before any suite run (`cargo test --locked -p thalamus-sources --test adv2_http_probe`):

```
running 3 tests
test adv2_the_invoke_pins_the_description_it_checked ... ok
test adv2_a_retry_checks_the_description_it_describes_again ... ok
test adv2_only_the_exact_catalog_read_profile_is_invoked ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

None of the cases is red on this tree. They fail on mutants that the unit's suite and the pass-1 cases let through.

### Mutants, applied to a scratch copy

Log: `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/mutants.log`. The unit's suites are `--lib` (12 tests) and `--test http_probe` (12); the pass-1 cases are `--test adv_http_probe` (7).

| # | mutant | unit + pass 1 | killed by adv2 |
|---|---|---|---|
| M1 | `http_probe.rs:133` profile compared with `eq_ignore_ascii_case` | survives (12/12/7) | `adv2_only_the_exact…` |
| M2 | `:133` `starts_with("generic-http")`, which admits `generic-http-page` | survives | `adv2_only_the_exact…` |
| M3 | `:133` `p.trim() == …` | survives | `adv2_only_the_exact…` |
| M4 | `:132` falls back to a top-level `profile` | survives | `adv2_only_the_exact…` |
| M5 | `:132` missing profile admitted (`.or(Some(CATALOG_READ_PROFILE))`) | killed by `probe_invokes_only_a_catalog_read_its_description_names` | also `adv2_only_the_exact…` |
| M6 | `connectors.rs:276` describes again after the check and pins the second description | survives | `adv2_the_invoke_pins…`, `adv2_a_retry…` |
| M7 | `connectors.rs:394` checks only the first attempt, so a retry runs unchecked | survives | `adv2_a_retry…` |
| M8 | `connectors.rs:273-276` check dropped | killed by `probe_invokes_only…` and `adv_a_probe_that_can_read_0_can_also_read_1` | also two adv2 cases |
| M9 | `http_probe.rs:203` `down()` reads `NotAdmitted` as 0 | killed by `probe_invokes_only…` and `adv_a_probe…` | also two adv2 cases |

## 3. Suite run, made after the cases existed

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess` **exit 0**. Log: `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/task-check.log`.

```
task: [check] ~/.cache/ess/toolchains/0.60.0/ess specify validate --path spec --strict-requires
task: [drift] set -eu
task: [check] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo fmt -p thalamus-core -p thalamus-store -p thalamus-sources -p thalamus-pipeline -p thalamus-service -p thalamus-cli -p thalamus-docs -- --check
task: [check] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo clippy --locked --workspace --all-targets -- -D warnings
task: [test] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo test --locked --workspace
     Running tests/adv2_http_probe.rs (target/debug/deps/adv2_http_probe-7c77320fd7d405f9)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
task: [docs-check] env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nice -n 19 cargo run --locked -p thalamus-docs -- generate --check --ess ~/.cache/ess/toolchains/0.60.0/ess
```

Over 42 `test result` lines, 218 passed and 0 failed. The 215 before is the sum of `passed` in the implementor's `scratch/task-check-round2.log`, written at 08:41:35, after the last source edit at 08:40:56. The 3 new cases make 218. `b10x-gates scan-text` on the new file printed "1 file(s) carry no private rule" (exit 0).

## 4. Findings (working tree on impl/m3-http-probe-adapter over 369a6a1)

| # | file:line | category | verdict | origin | finding |
|---|---|---|---|---|---|
| 1 | `crates/thalamus-sources/src/http_probe.rs:133` | mutant | CONFIRMED | introduced | Four looser profile matches survive the unit's suite: case-insensitive, prefix (admits `generic-http-page`), trimmed, and a top-level fallback. Keep `adv2_only_the_exact_catalog_read_profile_is_invoked`. |
| 2 | `crates/thalamus-sources/src/connectors.rs:394` | mutant | CONFIRMED | introduced | Two pin mutants survive the unit's suite: a second describe after the check, and a check on the first attempt only. Keep both adv2 pin cases. |
| 3 | `docs/design/thalamus.md:423` | contract-drift | CONFIRMED | introduced | D21 says every other adapter names its credential profile. In v0.45.0 they name their own operation profiles. |
| 4 | `crates/thalamus-sources/src/http_probe.rs:131` | judgement | INFEASIBLE | introduced | `generic-http` identifies the catalog only among shipped adapters, and Connectors' revision pin does not hash the profile. |

**1 and 2. Measured:** see the mutant table and `mutants.log`. The code is correct today. **What reaches it:** a future edit that loosens the match or restructures `invoke_once` or its retry loop. Until this pass, nothing in the suite would have failed on one.

**3. Measured:**
- Connectors v0.45.0 `adapters/kubernetes/spec/adapter.json:302-305` describes `resources.get` with profile `kubernetes-get`.
- Its credential profile is `kubernetes.token` (`adapters/kubernetes/src/auth.rs:16`).
- Other adapters' operation profiles: `grafana-datasources`, `promql-range`, `logql-metric`, `postgresql-native-text` (`adapters/sql/src/lib.rs:218`) and `tavily/2026-10`. Writes are `mutation` (kubernetes `pods.exec`, `adapter.json:1852-1855`).
- None of them is `generic-http`, so the conclusion holds and only the reason is wrong.
- The same wrong reason appears at `http_probe.rs:17`, `:55` and `tests/http_probe.rs:322`.
- The pass-1 fixture `adv_http_probe/common/operations+describe+a=cluster+o=resources.get.json` says `kubeconfig` where Connectors sends `kubernetes-get`. Behaviour is the same; I did not edit it.

**What reaches it:** the next reader of D21. **Fix:** "every other adapter names its own operation profile (`kubernetes-get`, `promql-range`, …), never `generic-http`".

**4. Measured:**
- In v0.45.0, `generic-http` is emitted only at `adapters/catalog/src/lib.rs:2186` (`Effect::Read`) and `:2257` (`declare_download`, which `check_binary` restricts to reads). No shipped adapter's `spec/adapter.json` carries it.
- Connectors runs any operator-pinned executable as an adapter (`crates/connectors-host/src/local/config.rs:98-112`, `Executable { path, sha256 }`). Nothing outside the catalog refuses the string.
- `operations describe` names only the alias, not the adapter kind (`apps/connectors/src/local/operations.rs:85`).
- Connectors pins the descriptor `revision` and the schema digest (`crates/connectors-host/src/local/owner.rs:368-378`). The digest hashes instance, revision, operation id and input and output schemas, not the profile (`owner.rs:292-300`).
- The catalog revision hashes `configuration_revision` and the operation ids (`adapters/catalog/src/local.rs:1025-1028`). `configuration_revision` hashes `bundle_sha256`, `source_sha256` and the operations config (`local.rs:901-912`). So a catalog read that turns into a write does move the revision.
- Independently, Connectors refuses any non-read invoke without an approval proof (`owner.rs:436-438`, `operations.rs:140-160`).

**What reaches it:** nothing found. It needs an operator to pin a non-shipped adapter that declares `generic-http`. Note only.

## 5. Attacked and not broken

- **Profile matching:** a missing, empty, differently-cased, padded, suffixed, non-text or misplaced profile is never admitted. 16 variants, all unknown with no invoke.
- **Description changing between describe and invoke:** thalamus pins the schema and revision of the description it checked (one describe per attempt), and a retry re-checks the new description.
- **NotAdmitted:** never 0 or 1. `down()` (`http_probe.rs:195-213`) matches only `Refused`. The registry turns the error into unknown (`registry.rs:127`). No other code matches `NotAdmitted` (grep over `crates/`).
- **Other adapters:**
  - Only `http_probe.rs:105` calls `invoke_if`; every other adapter goes through `invoke` with an always-true check.
  - I ran the full `thalamus-sources` suite on a `git archive` of base 369a6a1 and on a copy of this tree. All 122 call logs that exist in both are byte-identical (738 calls, argv and env names). The 41 new logs belong only to `http_probe`, `adv_http_probe` and `adv2_http_probe`. Lists: `scratch/adversary-p2/base-logs.txt` and `copy-logs.txt`.
- **Fixture profile change justified:** for a catalog read, v0.45.0 `describe` writes `"operation": operation(selected)` (`operations.rs:85`), which carries `profile` (`operations.rs:5`), and the catalog sets it to `generic-http` for `Effect::Read` (`catalog/src/lib.rs:2185-2186`). Pass 1's `token` was a credential-profile id (the `Requirement.profile` in Connectors' own test fixture, `operations.rs:308`), not an operation profile.
- **Catalog feed reads** (`feed.containers`, `feed.items`) carry their declared feed profile. Their ids do not end in `.get` or `.list`, so the id guard refuses them first.

## 6. Paths written outside the worktree

1. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/mutants.log`
2. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/task-check.log`
3. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/base-logs.txt`
4. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-p2/copy-logs.txt`
5. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-pass-2.md`: this report.
6. The worktree lease record for session `adversary-m3-http-probe-p2`, under `~/.local/state/worktree`. I release it on return.

I created and then deleted these: `scratch/adversary-p2/copy/` (a copy of the tree), `scratch/adversary-p2/base/` (a `git archive` of 369a6a1), `scratch/adversary-p2/pristine/`, and the throwaway `scratch/adversary-p2/run-mutant.sh`. I ran `cargo clean` on `copy` (369.4 MiB) and on `base` (330.5 MiB) before deleting them. Inside the worktree, the tests wrote under `target/tmp/adv2_http_probe/`.

## 7. Findings

```findings
[
  {"file": "crates/thalamus-sources/src/http_probe.rs", "line": 133, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "case-insensitive, prefix (admits generic-http-page), trimmed and top-level-fallback profile matches all survive the unit's suite and pass 1; adv2_only_the_exact_catalog_read_profile_is_invoked kills them"},
  {"file": "crates/thalamus-sources/src/connectors.rs", "line": 394, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "re-describing after the check and pinning the second description, or checking only the first attempt of the retry loop, survive the unit's suite; the two adv2 pin cases kill them"},
  {"file": "docs/design/thalamus.md", "line": 423, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "D21 (and http_probe.rs:17, :55, tests/http_probe.rs:322) says every other adapter describes its operations with its credential profile, while Connectors v0.45.0 gives them their own operation profiles such as kubernetes-get, never generic-http"},
  {"file": "crates/thalamus-sources/src/http_probe.rs", "line": 131, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "generic-http identifies the catalog only among shipped adapters, since any operator-pinned executable may declare it and Connectors' revision/schema pin does not hash the profile; nothing found reaches this"}
]
```
