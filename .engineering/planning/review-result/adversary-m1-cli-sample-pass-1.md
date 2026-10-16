---
format: aep.planning-md/3
id: review-result:adversary-m1-cli-sample-pass-1
kind: review-result
status: active
title: Adversary, story:m1-cli-sample, pass 1
relations:
- reviews: story:m1-cli-sample
revision: 1
---
unit: story:m1-cli-sample, working tree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m1-cli-sample (impl/m1-cli-sample, base 69526d3, uncommitted)
verdict: NEEDS-CHANGE (suite red)
cases: executed 427→439, red 5
origin: introduced 13 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths (one log directory and this report); one mutant copy directory, now removed
needs-coordinator: yes. 1) The acceptance text "no `value` key" contradicts design § 2.2 (row 6): amend the story or record the conflict. 2) Rows 1, 2 and 3 need a call on intended behaviour before the fix.

## 1. Diff stat

```
$ git --no-pager diff --stat          # tracked files, against HEAD 69526d3
 Cargo.lock                                 |   1 +
 crates/thalamus-cli/Cargo.toml             |   3 +-
 crates/thalamus-cli/src/cmd/sample.rs      | 187 +++++++++++++-
 crates/thalamus-cli/src/global.rs          |  33 ++-
 crates/thalamus-cli/tests/adv_stub_exit.rs |   4 +-
 crates/thalamus-docs/src/status.rs         |   6 +-
 crates/thalamus-pipeline/src/instance.rs   | 227 +++++++++++++++++
 crates/thalamus-pipeline/src/sample.rs     | 397 +++++++++++++++++++++++++++++
 website/data/status.json                   |   4 +-
 website/docs/concepts/samples.md           |  10 +-
 website/docs/getting-started.md            |  15 +-
 11 files changed, 867 insertions(+), 20 deletions(-)
untracked: crates/thalamus-cli/tests/adv_sample_p1.rs   <- mine (713 lines)
```

The tracked diff is the same as when I was handed the tree (867+/20-). It is the implementor's and the coordinator's. My only change is the untracked test file. I touched no non-test path.

## 2. Cases added: crates/thalamus-cli/tests/adv_sample_p1.rs

Red output below was captured from each case run alone, before any suite run. After that I ran `rustfmt` on the file, which shifted line numbers by one.

| case | asserts | now |
|---|---|---|
| `a_prometheus_collision_records_one_sample_per_series` | two PromQL series that differ only in a label that is not a dimension (`instance`, on `service.queue.depth`, which declares no dims): one pass records exactly 1 sample for the series | RED |
| `a_backfilled_sample_does_not_begin_after_itself` | sample `RATIO` at T2, then `--at T1`: the T1 sample's `denominator.since` == T1 | RED |
| `an_unknown_rule_on_a_proposed_definition_is_refused_at_load` | a `Proposed` definition with `rule: no_such_rule` makes the pass exit 65, naming the file and the rule, with no source read | RED |
| `a_refused_store_url_does_not_print_its_password` | `--store postgresql://reader:<pw>@127.0.0.1:1/thalamus` exits 74 and stderr does not contain `<pw>` | RED |
| `help_does_not_print_the_store_url_from_the_environment` | with `THALAMUS_STORE=<url with pw>`, `thalamus --help` stdout does not contain `<pw>` | RED |
| `a_backfilled_sample_has_no_negative_age` | `age_s >= 0` after the backfill | green (catches mutant M1) |
| `proposed_and_retired_definitions_are_never_sampled` | `sample <proposed>` / `sample <retired>` exit 65 `not-active`, with no read and nothing recorded; `--all --once` samples only the 3 Active signals | green (M2) |
| `an_unknown_rule_on_a_muted_definition_is_refused_at_load` | a Muted definition with an unknown rule exits 65 at load | green (M3) |
| `since_from_a_dimension_created_and_its_fallback` | `since: component.created`: checkout's since is 2025-03-02 with the exact `age_s`; search (no `created`) has since T1 on both passes | green (M4) |
| `text_format_is_id_series_quality_per_line` | the text lines are exactly {(id, series, quality)} as stored | green (M5) |
| `instance_and_store_come_from_the_environment_and_the_flag_wins` | `THALAMUS_INSTANCE` and `THALAMUS_STORE` are honoured, and `--store` beats `THALAMUS_STORE` | green (M6) |
| `a_read_refused_for_its_dims_is_reported_and_the_other_reads_are_recorded` | an unregistered `billing` read gives exit 65; JSON `refused` has 1 entry naming it; 2 samples are recorded and printed | green (M7) |
| `a_postgres_store_records_one_pass_under_one_run_id` | `--store` is a PostgreSQL URL; 2 passes give 8 samples, 2 run ids and 8 distinct ids; the default SQLite store is untouched | `#[ignore]` without `THALAMUS_TEST_POSTGRES_URL`; **passed** against a disposable postgres:17.6 container |

Red output, verbatim. The full logs are in `adversary-p1-logs/`.

```
thread 'a_prometheus_collision_records_one_sample_per_series' panicked at crates/thalamus-cli/tests/adv_sample_p1.rs:311:5:
assertion `left == right` failed: one pass records one sample per series; exit Some(0), stdout smp_000000000001	service.queue.depth	unknown
smp_000000000002	service.queue.depth	unknown
  left: 2
 right: 1
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out

thread 'a_backfilled_sample_does_not_begin_after_itself' panicked at crates/thalamus-cli/tests/adv_sample_p1.rs:336:9:
assertion `left == right` failed: the earliest sample of the series is its own origin, not a later one: {
  "denominator": { "age_s": 0, "n": 556, "since": "2026-10-14T23:57:03Z", "unit": "{request}" } ...
  left: String("2026-10-14T23:57:03Z")
 right: "2026-10-14T22:57:03Z"

thread 'an_unknown_rule_on_a_proposed_definition_is_refused_at_load' panicked at crates/thalamus-cli/tests/adv_sample_p1.rs:356:5:
assertion `left == right` failed: stderr: 
  left: Some(0)
 right: Some(65)

thread 'a_refused_store_url_does_not_print_its_password' panicked at crates/thalamus-cli/tests/adv_sample_p1.rs:384:5:
the password is printed: thalamus sample: store postgresql://reader:hunter2-not-printed@127.0.0.1:1/thalamus: thalamus.store.postgres-password: the URL carries a password; thalamus reads no credential, ...

thread 'help_does_not_print_the_store_url_from_the_environment' panicked at crates/thalamus-cli/tests/adv_sample_p1.rs:408:5:
      --store <LOCATOR>   ... [env: THALAMUS_STORE=postgresql://reader:hunter2-not-printed@127.0.0.1:1/thalamus]
```

Mutants. Each was applied to a scratch copy and never to the worktree. Each one leaves the unit's suite (`--test sample`, 7 tests) green, and the case named above goes red.

| id | mutation | unit suite | caught by |
|---|---|---|---|
| M1 | `sample.rs:338` drop `.max(0)` | 7 passed | `a_backfilled_sample_has_no_negative_age` |
| M2 | `sample.rs:185` `== Active` → `!= Muted` | 7 passed | `proposed_and_retired_definitions_are_never_sampled` |
| M3 | `instance.rs:147` drop `\| Muted` | 7 passed | `an_unknown_rule_on_a_muted_definition_is_refused_at_load` |
| M4 | `sample.rs:306` `.unwrap_or(first)` → `.unwrap_or(end.clone())` | 7 passed | `since_from_a_dimension_created_and_its_fallback` |
| M5 | `cmd/sample.rs:166` swap series and quality | 7 passed | `text_format_is_id_series_quality_per_line` |
| M6 | `global.rs:19` drop `env = "THALAMUS_STORE"` | 7 passed | `instance_and_store_come_from_the_environment_and_the_flag_wins` |
| M7 | `cmd/sample.rs:183` empty `refused` | 7 passed | `a_read_refused_for_its_dims_is_reported_and_the_other_reads_are_recorded` |

## 3. Suite run, after the cases existed

```
$ task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess
specify validate, drift, fmt --check, clippy -D warnings: pass
cargo test --locked --workspace --exclude thalamus-store:
  Running tests/adv_sample_p1.rs
  test result: FAILED. 7 passed; 5 failed; 1 ignored
  task: Failed to run task "check": task: Failed to run task "test": exit status 101
EXIT=201

$ cargo test --locked --workspace --exclude thalamus-store --no-fail-fast    (to count past the first red binary)
passed 434, failed 5, ignored 1, executed 439 over 61 test binaries; EXIT=101
tests/sample.rs: 7 passed
```

- Before: 427 executed. That is the implementor's `scratch/check.log`, the same workspace scope, before `test-postgres`.
- After: 439, which is 427 plus my 12 executed cases.
- A red suite is the result this pass was after.

## 4. Findings

The working tree is uncommitted, on base 69526d3. Origin: base `instance.rs`, `sample.rs` and `global.rs` are empty stubs, so every code finding is `introduced`.

| # | file:line | verdict / origin / severity | what was measured | what reaches it | fix (named, not applied) |
|---|---|---|---|---|---|
| 1 | crates/thalamus-pipeline/src/sample.rs:224 | NEEDS-CHANGE / introduced / blocker | 2 samples for series `service.queue.depth` in one pass (red case 1) | any PromQL answer with a label the definition does not declare. The Prometheus adapter documents the collision (`prometheus.rs:19-20`) and makes every colliding read `unknown`. A `sum by (component, instance)` query or an un-aggregated query reaches it. | Merge the reads of a pass that share a series key into one `unknown` sample naming the collision, or have the adapter emit one read per dims |
| 2 | crates/thalamus-pipeline/src/sample.rs:297 | NEEDS-CHANGE / introduced / warning | backfilled T1 sample has `since` = T2 > `ts`, which contradicts the spec ("earliest moment the population could contain anything") | `--at`, added by this unit, for a window earlier than one already stored. `EventlogStore::first_sample` is "the earliest appended, whatever its ts" (`samples.rs:96`). | `since` = the earliest stored `ts` of the series by instant, capped at the sample's own `ts` |
| 3 | crates/thalamus-pipeline/src/instance.rs:145 | NEEDS-CHANGE / introduced / warning | a `Proposed` definition with an unknown rule loads, and the pass exits 0 | story Work: "The grading rule of **each** definition is parsed at load". An operator writing a Proposed definition reaches it. The narrowing to Active and Muted matches `registry.rs` "sampled", not the story text. | Check every definition, or have the coordinator amend the story text |
| 4 | crates/thalamus-cli/src/cmd/sample.rs:111 | NEEDS-CHANGE / introduced / blocker | the password from `--store` appears in stderr | an operator passing a URL with a password: the store refuses it (74) and the CLI echoes it. This breaks the invariant "never read, print or store a credential". Lines 134 and 138 print the same `{locator}`, but only for locators already admitted. | Print the parsed `Locator` (password-free by construction) or omit it on a parse refusal |
| 5 | crates/thalamus-cli/src/global.rs:19 | NEEDS-CHANGE / introduced / warning | `--help` prints `[env: THALAMUS_STORE=<url with password>]` | `THALAMUS_STORE` set to a URL with a password, then any `--help` | `hide_env_values = true` on `--store` (and `--instance`) |
| 6 | story:m1-cli-sample `## Acceptance` | CONFIRMED / pre-existing / note | the stored unknown line is `{"quality":"unknown","value":{"reason":…}}`, so it has a `value` key. The acceptance says "no `value` key". | Design § 2.2 (`docs/design/thalamus.md:113`) spells exactly this wire form, and the generated encoder writes it. The unit's test comment attributes it to D11, but D11 is about Decimal strings. | Amend the acceptance text. I wrote no case, because it would assert against the design. |
| 7 | crates/thalamus-cli/src/cmd/sample.rs:121 | INFEASIBLE / introduced / note | from reading the code: a store failure mid-pass panics, `catch_unwind` exits 74, and stdout lists nothing. Samples already appended stay under the run id, with no partial-pass marker. | a PostgreSQL restart or a full disk mid-pass. I found no deterministic trigger: SQLite data cannot fail one append and not the next, and the store type cannot be injected. | Print the ids recorded so far before exiting 74, or document that a 74 can follow partial writes |
| M1–M7 | as in the mutant table | CONFIRMED / introduced / note | each mutant survives the unit's 7 tests | n/a | Adopt the green pin cases above |

## 5. Attacked and not broken

- One `run_` per pass: the PostgreSQL case and the unit suite both show distinct run ids across passes and one within a pass.
- Provenance: `producer`, `version` and `definition_rev` are already pinned by the unit. `connection` and `query_sha256` come straight from the read.
- `--all` with Muted, Proposed and Retired definitions: none is sampled, and none is read.
- Store locator: relative instance paths, `--store` beats env beats default, PostgreSQL works. Bad, unreachable and password-bearing locators exit 74.
- `retention_start`: no adapter sets it (`prometheus.rs:27-28`, `git.rs:57`), so the clamp and the fallback skipping the clamp are unreachable today.
- A dims refusal records the other reads and exits 65; JSON `refused` is populated.
- Exit 64 on usage errors, including a bad `--at`, is pinned by the unit.
- `age_s` never goes negative.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary-p1-logs/` holds the red logs, the mutant logs (`mutant-*.log`, `mutant-unit-*.log`), `check.log`, `suite.log`, `pg.log`, `pg.id` and `loc.err`.
- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary-pass-1.md` is this report.
- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary/` was the mutant copy plus `mutate.sh`. I ran `cargo clean --manifest-path` on it (875 MiB) and then removed it.
- Docker container `b28a6d73fb80`: removed by its id.
- Inside the worktree's build directory: `target/tmp/adv_sample_p1/` (`CARGO_TARGET_TMPDIR`).

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 224, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a Prometheus collision on undeclared labels records one unknown sample per colliding read, so one pass writes several samples for one series"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 297, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "first_sample resolves to the earliest appended sample, so a --at backfill gets denominator.since later than its own ts"},
  {"file": "crates/thalamus-pipeline/src/instance.rs", "line": 145, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the load-time grading-rule check skips Proposed and Retired definitions although the story says each definition"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 111, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a refused store URL is echoed verbatim to stderr, password included, against the no-credential invariant"},
  {"file": "crates/thalamus-cli/src/global.rs", "line": 19, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "clap shows the THALAMUS_STORE value in --help, printing any password it carries"},
  {"file": "story:m1-cli-sample", "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "the acceptance's 'no value key' for an unknown sample contradicts the design 2.2 wire form the store writes; the story text needs amending"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 121, "category": "concurrency", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a mid-pass store failure exits 74 printing none of the samples already appended under the run id; no deterministic trigger found"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 338, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping the age_s clamp survives the unit suite"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 185, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "sampling Proposed and Retired in --all survives the unit suite"},
  {"file": "crates/thalamus-pipeline/src/instance.rs", "line": 147, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping Muted from the load-time rule check survives the unit suite"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 306, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "falling back to the window end instead of the first stored sample survives the unit suite"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 166, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "reordering the text-format fields survives the unit suite"},
  {"file": "crates/thalamus-cli/src/global.rs", "line": 19, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "removing the THALAMUS_STORE env binding survives the unit suite"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 183, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "emptying the JSON refused list survives the unit suite"}
]
```
