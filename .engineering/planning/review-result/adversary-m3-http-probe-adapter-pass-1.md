---
format: aep.planning-md/3
id: review-result:adversary-m3-http-probe-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m3-http-probe-adapter, pass 1
relations:
- reviews: story:m3-http-probe-adapter
revision: 1
---
unit: story:m3-http-probe-adapter, uncommitted working tree on impl/m3-http-probe-adapter over 369a6a1
verdict: NEEDS-CHANGE
cases: executed 207→214, red 1
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 8 paths, listed in part 6
needs-coordinator: the story (revision 3, Behaviour line 33 and Acceptance) still says a provider 5xx reads 0; the implementation and § 11 D21 read it as unknown. Revise the story to match D21, or overrule D21.

## 1. `git --no-pager diff --stat`

```
 crates/thalamus-sources/src/adapter.rs    |   8 +
 crates/thalamus-sources/src/connectors.rs |  20 ++-
 crates/thalamus-sources/src/http_probe.rs | 265 +++++++++++++++++++++++++++++-
 crates/thalamus-sources/src/registry.rs   |   6 +-
 docs/design/thalamus.md                   |   1 +
 5 files changed, 289 insertions(+), 11 deletions(-)
```

The stat is the same as before this pass (289/11): those tracked changes belong to the implementor and the coordinator. Everything I added is untracked and is test-only: 13 files from `git status --short --untracked-files=all`, listed below.

- `crates/thalamus-sources/tests/adv_http_probe.rs`
- `crates/thalamus-sources/tests/fixtures/adv_http_probe/` with 12 recordings in the scenarios `common`, `cluster-ok`, `cluster-not-found`, `describe-refused-not-found-dispatch`, `describe-refused-timeout-dispatch`, `list-refused-timeout-dispatch`, `admission-not-found-with-service-code`, `service-not-found-at-readiness` and `ok-list-answer`

`http_probe.rs` in the worktree still has its original sha256 (`a7204b1a…491b8f`). I applied every mutant to a copy in scratch, never to the worktree. No non-test path was touched.

## 2. Cases added (`crates/thalamus-sources/tests/adv_http_probe.rs`)

| case | asserts | now |
|---|---|---|
| `adv_a_probe_that_can_read_0_can_also_read_1` | if a probe of an operation can read 0, it can also read 1. Two recorded answers for one operation, kubernetes `resources.get` on adapter instance `cluster`: the ok answer is the adapter's own document, with no numeric `status`; the refusal is `service_failure`/`not_found` at `dispatch` | **red** |
| `adv_guard_refuses_lookalikes_and_runs_nothing` | 17 ids are refused, `operations_for` is empty for each, and `operations` is never described or invoked. The ids: `x.get.delete`, `get`, `list`, `.get`, `.list`, `project.GET`, `project.Get`, a trailing `\n` or `\t`, a leading space, a zero-width space, Cyrillic `о` in the resource, Cyrillic `е` in the verb, `/../`, `;`, `%2e` and `project.list.create` | green |
| `adv_other_calls_and_stages_never_read_0` | reads unknown, not 0, on: describe refused `service_failure`/`not_found` at `dispatch`; describe refused `timeout` at `dispatch`; `connections list` refused `timeout` at `dispatch`; invoke refused `not_found` at `admission` carrying service code `not_found`; `service_failure`/`not_found` at `readiness`; an ok answer that is a list | green |
| `adv_hostile_input_reads_unknown_without_panicking` | `query.input` set to `null`, a string, a number, a bool, the empty text, a 20 000-deep array, a 20 000-deep object or a 400 kB object (over the per-argument limit, so the spawn fails) reads unknown, with no panic and no 0 | green |
| `adv_unplaceable_window_runs_nothing` | with window `P1M`, the read is unknown and nothing is described or invoked | green |
| `adv_operations_for_cannot_widen_other_adapters` | for every other registered adapter, `operations_for` equals `read_operations()`, including on a definition that names `operation: pods.exec` | green |
| `adv_query_digest_ignores_insertion_order` | both insertion orders of the query give the literal digest `a3628c27…98ec6c`, recomputed independently with `sha256sum` | green |

Red output of the red case run alone, captured before any suite run (`cargo test --locked -p thalamus-sources --test adv_http_probe -- adv_a_probe_that_can_read_0_can_also_read_1 --exact`):

```
running 1 test
test adv_a_probe_that_can_read_0_can_also_read_1 ... FAILED

---- adv_a_probe_that_can_read_0_can_also_read_1 stdout ----

thread 'adv_a_probe_that_can_read_0_can_also_read_1' (2309775) panicked at crates/thalamus-sources/tests/adv_http_probe.rs:183:5:
on the same operation a refusal reads 0 and a served answer reads Unknown(UnreadReading { reason: "the answer names no status" }): the probe can say down and never up

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s
```

That run was before rustfmt; in the suite run below the same assertion is reported at line 186.

### Mutants, applied to a scratch copy of the tree

Log: `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/mutants2.log`.

| mutant on `http_probe.rs` | unit's suite (`--lib`, `--test http_probe`) | killed by |
|---|---|---|
| `:123` byte-class check disabled (`valid = true`) | survives (12/12, 11/11) | `adv_guard_refuses_lookalikes_and_runs_nothing` (Cyrillic `о` in the resource) |
| `:129` `!resource.is_empty()` disabled | survives | `adv_guard_refuses_lookalikes_and_runs_nothing` (`.get`, `.list`) |
| `:83` window check disabled | survives (11/11) | `adv_unplaceable_window_runs_nothing` |
| `:176` `refused_call == call` disabled | killed only by the `--lib` unit test; `--test http_probe` stays 11/11 | also `adv_other_calls_and_stages_never_read_0` |

## 3. Suite run, made after the cases existed

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess` exited **201**. `specify validate`, `drift`, `fmt --check` and `clippy -D warnings` passed. The `test` step failed, and `docs-check` was not reached:

```
test adv_a_probe_that_can_read_0_can_also_read_1 ... FAILED
thread 'adv_a_probe_that_can_read_0_can_also_read_1' (2335125) panicked at crates/thalamus-sources/tests/adv_http_probe.rs:186:5:
on the same operation a refusal reads 0 and a served answer reads Unknown(UnreadReading { reason: "the answer names no status" }): the probe can say down and never up
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
error: test failed, to rerun pass `-p thalamus-sources --test adv_http_probe`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

`cargo test --locked --workspace` stops at the first failing binary, so I also ran `cargo test --locked --workspace --no-fail-fast` to get the full count. It exited **101**: 213 passed, 1 failed, 214 executed. The only failure is `adv_a_probe_that_can_read_0_can_also_read_1`. The 207 before is the sum of `passed` across the implementor's `scratch/task-check.log`, and the 7 new cases make 214.

## 4. Findings (working tree on impl/m3-http-probe-adapter over 369a6a1)

| # | file:line | category | verdict | origin | finding |
|---|---|---|---|---|---|
| 1 | `crates/thalamus-sources/src/http_probe.rs:160` | property | NEEDS-CHANGE | introduced | On every adapter that is not catalog, the probe can read 0 but never 1. |
| 2 | `.engineering/planning/story/m3-http-probe-adapter.md:33` vs `crates/thalamus-sources/tests/http_probe.rs:215` | acceptance | NEEDS-CHANGE | introduced | The story says a provider 5xx reads 0. The code reads it as unknown. The acceptance test named `probe_provider_unavailable_is_up_0_with_reason` asserts 0 for `not_found` and `timeout`, while Connectors' `unavailable` reads unknown. |
| 3 | `crates/thalamus-sources/src/http_probe.rs:123`, `:129`, `:83` | mutant | CONFIRMED | introduced | Three guards survived the unit's own suite. My cases now kill them; keep those cases. |
| 4 | `crates/thalamus-sources/src/http_probe.rs:129` | judgement | CONFIRMED | introduced | The read-only guard is a naming rule and never checks the operation's `effect`. |

**1. Measured:** `adv_http_probe.rs:186` fails with exit 101. The served answer reads `Unknown("the answer names no status")` and the 404 reads `0`.

**What reaches it:**
- The guard (D21) admits any `.get`/`.list` on any adapter instance, and nothing in thalamus checks the adapter kind.
- Connectors v0.45.0 ships these non-catalog reads: kubernetes `resources.get` and `namespaces.list`, sql `schema.list` and `table.list`, grafana `datasources.list`, prometheus `rules.list`.
- Their 404 is `.answered()` (`crates/connectors-sdk/src/lib.rs:331-343`), so it maps to `ProviderNotFound` and then to `service_failure`/`not_found` at `dispatch`, which reads 0.
- A sql statement timeout (`adapters/sql/src/lib.rs:681-692`) maps to `timeout` at `dispatch`, which also reads 0.
- Their ok documents carry no integer `status`, so a served read reads unknown.

No definition shipped in the repository does this; any definition file can. **Fix:** read 1 on an ok answer that has no `status` key (raw `None`), keep unknown for a `status` that is present and not an integer 2xx, and update the module table and the D21 row.

**2. Measured:** the story text at line 33, the Acceptance list at line 44, and D21 at `docs/design/thalamus.md:423`. I checked the wire facts in D21 against Connectors v0.45.0 and they hold:
- catalog `read_body` sends 5xx to `Unavailable`; `from_service` keeps it `Unavailable`, which maps to `readiness`;
- 404 and 410 map to `ProviderNotFound` at `dispatch`;
- 401 maps to `service_failure`/`unauthorized`;
- 403 maps to `forbidden` at `dispatch`;
- 429 maps to `rate_limited`.

**What reaches it:** whoever reads the story or the test name. The fix is a story revision, plus renaming or extending the test so that its name does not say "unavailable is 0".

**3. Measured:** the mutant table in part 2. **What reaches it:** a definition whose operation is `.get`, holds a lookalike letter, or has an unplaceable window. Connectors refuses unknown ids at `admission`, so each guard is defence in depth.

**4. Measured:**
- In the v0.45.0 catalog selection sets, all 45 `*.get`/`*.list` ids are `effect: read`.
- In every other adapter, each `.get` and `.list` is `Effect::Read`; the only write effect beside them is kubernetes `pods.exec` (`adapters/kubernetes/src/local.rs:207-221`).
- Connectors requires an approval proof for a write effect (`apps/connectors/src/local/operations.rs:140-160`), and thalamus never sends one.

**What reaches it:** only a future selection set that names a write `*.get` or `*.list`. Note only.

## 5. Attacked and not broken

- A refusal at `readiness`, at `admission`, from `describe`, from `connections list`, or with no `service_code` never reads 0 or 1.
- `not_found` at `admission`, even with service code `not_found`, reads unknown and is never taken for a target 404.
- Guard bypasses: none of the 17 ids in part 2 passes, whether by case, whitespace, unicode, an empty resource or a non-read last segment.
- `operations_for`: it cannot widen any other adapter, and the default equals `read_operations()` for every registered adapter.
- An ok answer without a 2xx status, with a string status, or as a list is never up. The unit tests and `ok-list-answer` cover these.
- `query.input` that is not an object, deeply nested, or 400 kB reads unknown without panicking.
- The query digest matches the literal for both insertion orders, and the `QUERY_SHA256` in `tests/http_probe.rs` matches `sha256sum`.
- A dispatched `timeout` and `not_found` are not retried (1 invoke). `timeout` at `admission` is retried once and then reads unknown.
- The `Display` change to `Refused` is not parsed by any repository test, document or spec.

## 6. Paths written outside the worktree

1. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/mutant/`: a source copy of the tree (6.9 MB), with its own `target/` (155 MB). Leave the deletion to the coordinator.
2. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/mutants.log`
3. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/mutants2.log`
4. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/task-check.log`
5. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary/test-no-fail-fast.log`
6. `~/.cache/thalamus-waves/20261015c/m3-http-probe-adapter/scratch/adversary-pass-1.md`: this report.
7. The harness's background-task output file under `~/.cache/claude-tmp/claude-1000/…/tasks/bghlk45ow.output`.
8. The worktree lease record for session `adversary-m3-http-probe-p1`, under `~/.local/state/worktree`, which I released at the end.

I wrote two throwaway mutant scripts and a pristine copy of `http_probe.rs` in `scratch/adversary/` and deleted them. Inside the worktree, the tests wrote under `target/tmp/adv_http_probe/`.

## 7. Findings

```findings
[
  {"file": "crates/thalamus-sources/src/http_probe.rs", "line": 160, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the guard admits any adapter's .get/.list; a non-catalog served read has no integer status and reads unknown while its 404 or dispatched timeout reads 0, so the probe can say down and never up"},
  {"file": ".engineering/planning/story/m3-http-probe-adapter.md", "line": 33, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "story revision 3 says a provider 5xx reads 0 and names the test probe_provider_unavailable_is_up_0_with_reason, while the code and D21 read Connectors' unavailable (the 5xx) as unknown and the test of that name asserts 0 only for not_found and timeout"},
  {"file": "crates/thalamus-sources/src/http_probe.rs", "line": 123, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the byte-class check (:123), the empty-resource check (:129) and the window check (:83) each survived the unit's suite when disabled; the adversary guard and window cases kill all three"},
  {"file": "crates/thalamus-sources/src/http_probe.rs", "line": 129, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the read-only guard is a naming rule that never checks the operation's effect; it holds for every Connectors v0.45.0 get/list id and Connectors still requires an approval proof for a write"}
]
```
