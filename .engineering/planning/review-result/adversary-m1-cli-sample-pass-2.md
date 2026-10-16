---
format: aep.planning-md/3
id: review-result:adversary-m1-cli-sample-pass-2
kind: review-result
status: active
title: Adversary, story:m1-cli-sample, pass 2
relations:
- reviews: story:m1-cli-sample
revision: 1
---
unit: story:m1-cli-sample, working tree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15e-m1-cli-sample (impl/m1-cli-sample, base 69526d3, uncommitted)
verdict: NEEDS-CHANGE (suite red)
cases: executed 493→501, red 3
origin: introduced 10 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths kept (log directory, this report); 1 scratch directory created and removed
needs-coordinator: whether finding 2 (a non-instant stored `ts`) is fixed now or recorded; no writer reaches it today

## 1. Diff stat

```
$ git --no-pager diff --stat          # tracked files, against HEAD 69526d3
 Cargo.lock                                 |   1 +
 crates/thalamus-cli/Cargo.toml             |   3 +-
 crates/thalamus-cli/src/cmd/sample.rs      | 258 ++++++++++++++-
 crates/thalamus-cli/src/global.rs          |  45 ++-
 crates/thalamus-cli/tests/adv_stub_exit.rs |   4 +-
 crates/thalamus-docs/src/status.rs         |   6 +-
 crates/thalamus-pipeline/src/instance.rs   | 227 +++++++++++++
 crates/thalamus-pipeline/src/sample.rs     | 514 +++++++++++++++++++++++++++++
 website/data/status.json                   |   4 +-
 website/docs/concepts/samples.md           |  10 +-
 website/docs/getting-started.md            |  15 +-
 11 files changed, 1067 insertions(+), 20 deletions(-)
untracked: crates/thalamus-cli/tests/adv_sample_p2.rs   <- mine (585 lines)
```

- The tracked diff is the one I was handed, at 1067+/20-.
- My only change is the untracked test file. I touched no non-test path.
- `adv_sample_p1.rs` is unchanged.

## 2. Cases added: crates/thalamus-cli/tests/adv_sample_p2.rs

I captured the red output by running the file alone (`cargo test --locked -p thalamus-cli --test adv_sample_p2`) before any suite run. Then I ran `rustfmt`, which moved line numbers. The full log is `adversary-p2-logs/red-p2.log`.

| case | asserts | now |
|---|---|---|
| `a_store_eventlog_cannot_open_is_not_named_by_its_path` | a `sqlite:` locator Eventlog cannot open (a file in a mode-500 directory, or a mode-000 file) exits 74, and stderr does not contain the path's marker | RED |
| `a_stored_ts_that_names_no_instant_is_not_the_origin_of_a_series` | another writer appends a checkout sample with `ts` `2026-10-14 20:00:00Z`; the T2 sample then has `since` == T1 and `age_s` == 3600 | RED |
| `a_unix_socket_store_is_named_by_its_socket_directory` | `postgresql:///thalamus?host=<dir>` exits 74, and stderr names the database and the socket directory | RED |
| `an_admitted_postgres_url_is_named_by_host_and_database_only` | `postgresql://mark3r@127.0.0.1:1/thalamus?application_name=mark3r&user=mark3r` names host `127.0.0.1:1` and database `thalamus`, and prints no marker | green (catches M1) |
| `a_store_that_fails_mid_pass_prints_what_it_recorded_first` | an `sqlite3` trigger makes the store refuse the 2nd sample of a pass. JSON and text: exit 74, the 1 recorded id is printed, and the printed ids == the stored RATIO ids. The case skips itself, printing so, when no `sqlite3` CLI is installed | green (catches M5) |
| `since_is_the_earliest_of_several_stored_samples` | passes at T1, T2 and T2: the third pass has `since` T1 and `age_s` 3600, for both series | green (catches M2) |
| `collisions_in_two_definitions_of_one_pass_each_record_one_sample` | one `--all` pass: RATIO checkout with 3 numerator series and 1 denominator series, DEPTH with 2 series. Asserts 1 sample per series, `unknown` with no denominator, a reason starting `N reads of one pass fall on series <key>` and holding the adapter's reason exactly once, and search `ok` | green (catches M3, M6) |
| `a_proposed_draft_with_incomplete_lines_loads` | a Proposed copy of DEPTH with no `crit` exits 0; the same file made Active exits 65, naming the file | green (catches M4) |

The red output, verbatim:

```
thread 'a_unix_socket_store_is_named_by_its_socket_directory' panicked at crates/thalamus-cli/tests/adv_sample_p2.rs:391:5:
the refusal names no host: thalamus sample: store postgres host `` database `thalamus` refused: eventlog: event store is unavailable: PostgreSQL connection unavailable
thread 'a_store_eventlog_cannot_open_is_not_named_by_its_path' panicked at crates/thalamus-cli/tests/adv_sample_p2.rs:328:9:
the refusal prints the locator's path: thalamus sample: store sqlite refused: eventlog: event store is unavailable: unable to open database file: <tree>/target/tmp/adv_sample_p2/unopenable/mark3r/store.sqlite
thread 'a_stored_ts_that_names_no_instant_is_not_the_origin_of_a_series' panicked at crates/thalamus-cli/tests/adv_sample_p2.rs:370:5:
assertion `left == right` failed: the earliest stored instant is the origin: {
  left: String("2026-10-14 20:00:00Z")
 right: "2026-10-14T22:57:03Z"
test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
EXIT=101
```

I applied each mutant to a copy under `scratch/adversary-p2/mut`, never to the worktree, and ran `cargo test --locked -p thalamus-cli -p thalamus-pipeline --no-fail-fast` on it. The logs are `adversary-p2-logs/mutant-M*.log`.

| id | mutation | unit `sample.rs` (9) + `adv_sample_p1.rs` (12) | caught by |
|---|---|---|---|
| M1 | `cmd/sample.rs:190` `describe` keeps the query (`let before_query = rest;`) | all pass | `an_admitted_postgres_url_is_named_by_host_and_database_only` |
| M2 | `sample.rs:394` `.next()` → `.last()` (since from the latest stored sample) | all pass | `since_is_the_earliest_of_several_stored_samples` |
| M3 | `sample.rs:366` reason dedup dropped | all pass | `collisions_in_two_definitions_…` |
| M4 | `instance.rs:148` thresholds also checked for `Proposed` | all pass | `a_proposed_draft_with_incomplete_lines_loads` |
| M5 | `cmd/sample.rs:162` no `print` before exit 74 | all pass | `a_store_that_fails_mid_pass_prints_what_it_recorded_first` |
| M6 | `sample.rs:376` `{count} reads` → `several reads` | all pass | `collisions_in_two_definitions_…` |

## 3. Suite run, after the cases existed

```
$ task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess
specify validate, drift, fmt --check, clippy --all-targets -D warnings: pass
cargo test --locked --workspace --exclude thalamus-store:
  Running tests/adv_sample_p1.rs   test result: ok. 12 passed; 0 failed; 1 ignored
  Running tests/adv_sample_p2.rs   test result: FAILED. 5 passed; 3 failed; 0 ignored
task: Failed to run task "check": task: Failed to run task "test": exit status 101
EXIT=201

$ cargo test --locked --workspace --exclude thalamus-store --no-fail-fast   (to count past the red binary)
passed 446, failed 3, ignored 1, executed 449 over 62 binaries; EXIT=101
$ task test-postgres     (the step the gate did not reach; container removed by the task's trap, none left)
passed 52, failed 0, executed 52 over 6 binaries; EXIT=0
```

- Before: 493 executed, from the implementor's `scratch/check-p1.log` in the same gate scope.
- After: 449 + 52 = 501, which is 493 plus my 8 cases.
- A red suite is the result this pass was after.

## 4. Findings

The tree is uncommitted, on base 69526d3. At the base, `cmd/sample.rs`, `pipeline/sample.rs` and `instance.rs` are stubs, so every finding is `introduced`.

| # | file:line | verdict / origin / severity | what was measured | what reaches it | fix (named, not applied) |
|---|---|---|---|---|---|
| 1 | crates/thalamus-cli/src/cmd/sample.rs:204 | NEEDS-CHANGE / introduced / warning | `refusal()` prints a non-locator `StoreError` verbatim. Eventlog's SQLite open error carries rusqlite's `unable to open database file: <path>`, so the locator path is printed (red case 1). This contradicts the file header (`:13-14`), decision 4, and the unit's own test doc ("not a path"). | `--store` or `THALAMUS_STORE` naming an unwritable directory or an unreadable file. The default store under a read-only instance directory reaches it too. | At open, print the error kind without its text (for example `StoreError::Eventlog(Backend(_))` as "the store cannot be opened"), or cut the locator path out of the message |
| 2 | crates/thalamus-pipeline/src/sample.rs:394 | INFEASIBLE / introduced / note | `first_sample_ts` takes the first row of `series_samples`, which ranks a non-instant before the instants by bytes. `resolve_since` refuses the non-instant, and `.unwrap_or(first)` (`:423`) puts it back. Result: `since` `2026-10-14 20:00:00Z` and `age_s` 0 on every later sample of the series (red case 2). | I built this state: the case appends through `EventlogStore::append_sample`, which admits any `ts`. Nothing found that writes one: `--at` is checked (`cmd/sample.rs:76`) and the clock is RFC 3339. | Take the first row whose `ts` parses as an `Instant`, or have the store refuse a non-instant `ts` |
| 3 | crates/thalamus-cli/src/cmd/sample.rs:189 | NEEDS-CHANGE / introduced / note | `describe` reads the host only from the authority, so `postgresql:///thalamus?host=/run/…` prints host `` (red case 3) | the peer-authenticated Unix-socket locator, which the `postgres-password` refusal points operators to | Name the host from `tokio_postgres::Config::get_hosts()`, as the store already parses it |
| 4 | crates/thalamus-cli/src/cmd/sample.rs:159 | CONFIRMED / introduced / note | on a store failure mid-pass, the CLI's own line says only "it failed during the pass". The cause (`sample smp_… was not stored: eventlog: …`) reaches the operator only as a Rust panic message with a backtrace hint, from the default panic hook. Observed by hand with the case's trigger. | any mid-pass store failure. Exit 74 and the printed ids are correct. | Install a panic hook that captures the payload, and print it in the 74 line |
| M1–M6 | as in the mutant table | CONFIRMED / introduced / note | each mutant survives the unit suite and `adv_sample_p1.rs` | n/a | Adopt the green pin cases above |

## 5. Attacked and not broken

- **Merge, collision count:** collisions in two definitions of one pass give 1 sample per series, the read count and the adapter reason once, no denominator, and the other series stays `ok`.
- **Merge, mixed ok and unknown:** no source reaches it. Prometheus (`prometheus.rs:222`, `:310`), SQL (`sql.rs:347`), Loki (`loki.rs:212`) and Git (`git.rs:678`) all mark every colliding read `unknown` with one reason. So the merge's `denominator = None` and `raw = None` lines have no reachable effect, and `raw` never reaches `RecordSample`.
- **Merge, reason length:** bounded by the distinct reasons, which is 1 for every adapter today.
- **since, equal instants in two spellings:** `is_lt` returns the sample's own `ts`; a later sample gets the stored spelling and the right `age_s`.
- **since, backfill:** handled, and pinned by the pass-1 case.
- **Redaction, passwords:** none printed. Covered forms: password in the userinfo, percent-encoded, empty, or in the query; `--store=…`; `THALAMUS_STORE`; markers in the user and in `application_name`.
- **Redaction, other paths:** clap errors (missing argument, unknown argument, conflicts, duplicate `--store`) and `--format json` refusals (stdout empty) print no locator. A non-loopback host and a host list show the host only.
- **Mid-pass store failure (deterministic `sqlite3` trigger):** text and JSON print exactly the stored ids before exit 74.
- **Rule-kind check:** `instance.rs:151` uses the same `RuleId::from_str` exact match as `grading::grade` (`grading.rs:74`). Only the core's own test fixture (`thalamus-core/tests/fixtures/instance`, Proposed `rule: threshold`) would now be refused, and no pipeline or CLI path loads it.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary-p2-logs/`, kept. It holds `red-p2.log`, `check.log`, `suite.log`, `test-postgres.log` and `mutant-M1.log` … `mutant-M6.log`.
- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary-pass-2.md`, kept. It is this report.
- `~/.cache/thalamus-waves/20261015e/m1-cli-sample/scratch/adversary-p2/`, removed. It held the mutant copy `mut/` (`cargo clean --manifest-path` removed 952.8 MiB first), `orig/` and `probe/` (modes restored, then removed).
- PostgreSQL: only the container `task test-postgres` started, removed by the task's trap. `docker ps -a --filter name=thalamus-test-postgres` is empty.
- Inside the worktree's build directory: `target/tmp/adv_sample_p2/` (`CARGO_TARGET_TMPDIR`), with modes restored.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 204, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a SQLite locator Eventlog cannot open is refused with rusqlite's message, which prints the locator path against decision 4 and the file's own header"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 394, "category": "property", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a stored ts naming no instant that sorts first by bytes becomes denominator.since with age_s 0 for every later sample of the series; no writer is known to store one"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 189, "category": "boundary", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "describe names a Unix-socket PostgreSQL store given as ?host= with an empty host"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 159, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a mid-pass store failure's cause reaches the operator only as a Rust panic message; the exit-74 line says only that it failed during the pass"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 190, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "describe printing the query of an admitted URL survives the suite"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 394, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "taking the latest instead of the earliest stored sample as origin survives the suite"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 366, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping the dedup of merged collision reasons survives the suite"},
  {"file": "crates/thalamus-pipeline/src/instance.rs", "line": 148, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "checking thresholds of Proposed definitions survives the suite"},
  {"file": "crates/thalamus-cli/src/cmd/sample.rs", "line": 162, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "printing nothing before exit 74 on a mid-pass store failure survives the suite"},
  {"file": "crates/thalamus-pipeline/src/sample.rs", "line": 376, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "dropping the read count from the merged collision reason survives the suite"}
]
```
