---
format: aep.planning-md/3
id: review-result:adversary-m1-eventlog-store-pass-2
kind: review-result
status: active
title: Adversary, story:m1-eventlog-store, pass 2
relations:
- reviews: story:m1-eventlog-store
revision: 1
---
unit: story:m1-eventlog-store — uncommitted working tree of impl/m1-eventlog-store (base ca445f1), worktree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-eventlog-store
verdict: red (strongest NEEDS-CHANGE)
cases: executed 330→345, red 7
origin: introduced 8, pre-existing 0, undecided 0 (adversary)
wrote-outside-worktree: 52 log files under ~/.cache/thalamus-waves/20261015d/m1-eventlog-store/scratch/adversary-p2/, plus this report (part 6)
needs-coordinator: yes — two writers on one instance mint the same id (N1). Decide between the store-local sequence the unit adopted and a writer-unique id (the ULID that base design § 2.2 and the brief named). Concurrent PostgreSQL opens time out (N4), so `task check` goes red intermittently under load.

## 1. Diff stat

`git --no-pager diff --stat` reads `19 files changed, 2948 insertions(+), 88 deletions(-)`, before my work and after it. I changed no tracked file.

My one new file is `crates/thalamus-store/tests/adv_p2_eventlog_store.rs` (721 lines, test file, untracked). The other two untracked files are not mine: `adv_eventlog_store.rs` is pass 1's and `store.rs` is the implementor's. No non-test path was touched. `b10x-gates scan-text` on my file exited 0. The scratch copy and the containers are listed in part 6.

## 2. Cases added

All cases are in `crates/thalamus-store/tests/adv_p2_eventlog_store.rs` and run on SQLite and PostgreSQL. Each was run alone with `cargo test --locked -p thalamus-store --test adv_p2_eventlog_store -- --exact <name>` before any suite run, against my own disposable `postgres:17.6-alpine3.22` container. Logs: `scratch/adversary-p2/case-<name>.log`.

| case (line) | asserts | alone |
|---|---|---|
| `adv_p2_stale_handle_storing_a_taken_id_is_refused_id_taken` :135 | a stale handle that stores another body under a stored id gets `Refused{ID_TAKEN}`, as `append_sample` documents | RED |
| `adv_p2_two_handles_minting_then_recording_keep_both_samples` :160 | two handles: mint, mint, record, record. Both samples are stored under distinct ids | RED |
| `adv_p2_two_threads_minting_and_recording_lose_no_sample` :439 | 2 threads × 40, each mints from the store and then appends: 80 samples, 80 ids, no error | RED |
| `adv_p2_two_threads_recording_disjoint_ids_lose_no_sample` :454 | the same with caller-chosen disjoint ids, which isolates the single retry | RED |
| `adv_p2_postgres_concurrent_opens_all_succeed` :522 | 8 threads open one PostgreSQL database at once, and every open succeeds | RED |
| `adv_p2_postgres_sslmode_require_is_not_answered_with_plaintext` :567 | `?sslmode=require` against the server without TLS does not open | RED |
| `adv_p2_same_sample_from_two_handles_is_stored_once` :184 | a retry from a stale handle writes nothing; head is 2 after the next sample | green |
| `adv_p2_stale_handle_second_evaluation_at_one_rev_is_refused` :212 | D5 across handles: the stale handle gets `DUPLICATE_EVALUATION` after catch-up; 1 evaluation stored | green (kills M03, M04) |
| `adv_p2_stale_handle_reads_and_grades_what_another_handle_stored` :271 | a stale handle answers another handle's records through `first_sample`, `evaluation`, the evaluation port `get`/`list`, `newest_evaluations`, `unevaluated_samples`, `series_range` and `append_evaluation` | green (kills M05, M08, M10, M11) |
| `adv_p2_stale_handle_mints_past_what_the_stream_holds` :340 | after another handle stored smp 1–3, a stale handle mints 4 | green (kills M12) |
| `adv_p2_retry_of_another_handles_sample_is_taken_in` :358 | a retry Eventlog deduplicates is taken in: head 1, `samples()` holds the sample, 1 record | green alone (kills M13); red in the suite, see part 3 |
| `adv_p2_reads_racing_appends_answer_every_acknowledged_sample` :476 | while a writer thread appends 40 samples, the reader answers each acknowledged id through `sample`, `get` and `list`, with no duplicate | green |
| `adv_p2_postgres_password_in_the_query_is_refused_unrepeated` :548 | `?password=`, `user:pw@` and `user:@…?password=` are each refused `POSTGRES_PASSWORD`, and the message does not repeat the secret | green (kills M19) |
| `adv_p2_postgres_unusable_server_fails_loudly_and_soon` :584 | port 1, a missing database, `192.0.2.1` and a missing socket directory each fail within 5 s | green |
| `adv_p2_design_blocks_are_records_the_store_opens` :691 | the § 2.2 and § 2.3 json blocks, appended verbatim to a stream (the `…` digest filled with zeros), pass the open's decode→encode round-trip; the store answers reading `0.029` and score `1.158` | green |

Red output at first run, verbatim (the home directory shortened to `~`; long failure lists are cut at `[…]`):

```
---- adv_p2_stale_handle_storing_a_taken_id_is_refused_id_taken stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:144:5:
a stale handle storing another body under a stored id was not refused thalamus.record.id-taken:
sqlite:~/.cache/claude-tmp/.tmpZH7Wx8/thalamus.sqlite (advp2-2486237.tmpZH7Wx8): Err(Eventlog(IdempotencyMismatch { key: "smp_000000000001" }))
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2486237.tmpZH7Wx8): Err(Eventlog(IdempotencyMismatch { key: "smp_000000000001" }))
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.06s

---- adv_p2_two_handles_minting_then_recording_keep_both_samples stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:173:5:
sqlite:~/.cache/claude-tmp/.tmpCmmRkb/thalamus.sqlite (advp2-2486305.tmpCmmRkb): minted 1 and 1; first Ok(()); second Err(Eventlog(IdempotencyMismatch { key: "smp_000000000001" })); 1 sample(s) on the stream
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2486305.tmpCmmRkb): minted 1 and 1; first Ok(()); second Err(Eventlog(IdempotencyMismatch { key: "smp_000000000001" })); 1 sample(s) on the stream
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.44s

---- adv_p2_two_threads_minting_and_recording_lose_no_sample stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:268:5:
44 failure(s):
sqlite:~/.cache/claude-tmp/.tmpki16sc/thalamus.sqlite (advp2-2487972.tmpki16sc): writer 1 step 0 smp_000000000001: Eventlog(IdempotencyMismatch { key: "smp_000000000001" })
sqlite:~/.cache/claude-tmp/.tmpki16sc/thalamus.sqlite (advp2-2487972.tmpki16sc): writer 1 step 1 smp_000000000016: Eventlog(IdempotencyMismatch { key: "smp_000000000016" })
sqlite:~/.cache/claude-tmp/.tmpki16sc/thalamus.sqlite (advp2-2487972.tmpki16sc): 78 records, 78 distinct ids, 78 distinct samples; 80 were recorded
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2487972.tmpki16sc): writer 2 step 0 smp_000000000001: Eventlog(IdempotencyMismatch { key: "smp_000000000001" })
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2487972.tmpki16sc): writer 2 step 1 smp_000000000002: Eventlog(IdempotencyMismatch { key: "smp_000000000002" })
[…]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 2.85s

---- adv_p2_two_threads_recording_disjoint_ids_lose_no_sample stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:283:5:
21 failure(s):
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2488422.tmpXDQdTe): writer 1 step 0 smp_000001000001: Eventlog(Conflict { expected: 1, actual: 2 })
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2488422.tmpXDQdTe): writer 1 step 1 smp_000001000002: Eventlog(Conflict { expected: 3, actual: 4 })
[…]
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2488422.tmpXDQdTe): writer 1 step 19 smp_000001000020: Eventlog(Conflict { expected: 39, actual: 40 })
postgresql://postgres@127.0.0.1:34127/postgres (advp2-2488422.tmpXDQdTe): 60 records, 60 distinct ids, 60 distinct samples; 80 were recorded
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 3.17s

---- adv_p2_postgres_sslmode_require_is_not_answered_with_plaintext stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:371:5:
postgresql://postgres@127.0.0.1:34127/postgres?sslmode=require: opened over plaintext although the locator requires TLS
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.87s

---- adv_p2_postgres_concurrent_opens_all_succeed stdout ----
panicked at crates/thalamus-store/tests/adv_p2_eventlog_store.rs:539:5:
advp2-4001878.tmpdsWqpg-1 after 2.050540043s: eventlog: event store deadline exceeded during database statement or lock
advp2-4001878.tmpdsWqpg-6 after 2.041458777s: eventlog: event store deadline exceeded during database statement or lock
advp2-4001878.tmpdsWqpg-7 after 2.044952415s: eventlog: event store deadline exceeded during database statement or lock
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 4.49s
```

The line numbers quoted in the red output are from before `rustfmt` ran on my file. The test bodies did not change.

### Mutants

Mutants ran on a scratch copy; the worktree was never mutated. Each mutant ran `cargo test --locked -p thalamus-store --no-fail-fast` on SQLite and PostgreSQL. The table lists the tests that failed beyond the baseline reds (log: `mutants-round1.log`, then `mutants.log` for the rerun after my last three green cases). "Implementor" means `store.rs` plus `conformance.rs`; "pass 1" means `adv_eventlog_store.rs`.

| mutant | implementor | pass 1 | pass 2 |
|---|---|---|---|
| M01 `samples.rs:44` no catch-up retry | survives | killed (`adv_long_lived…`) | — |
| M02 `samples.rs:45` retry without `refresh` | survives | killed | — |
| M03 `evaluations.rs:144` no evaluation catch-up retry | survives | survives | killed (`…second_evaluation_at_one_rev…`) |
| M04 `evaluations.rs:145` evaluation retry without `refresh` | survives | survives | killed |
| M05 `evaluations.rs:167` no read before `unknown-sample` | survives | survives | killed (`…reads_and_grades…`) |
| M07 `samples.rs:86` `sample()` skips the tail | survives | killed | killed |
| M08 `samples.rs:107` `first_sample` skips the tail | survives | survives | killed |
| M09 `samples.rs:121` `current_samples` skips the tail | survives | survives | killed (`…reads_racing…`) |
| M10 `evaluations.rs:203` `evaluation()` skips the tail | survives | survives | killed |
| M11 `evaluations.rs:223` `current_evaluations` skips the tail | survives | survives | killed |
| M12 `eventlog.rs:591` `next_number` without `refresh` | survives | survives | killed (`…mints_past…`) |
| M13 `eventlog.rs:534` dedup returns `Ok` without taking in | survives | survives | killed (`…retry_of_another…`) |
| M14 `eventlog.rs:458` round-trip check off | survives | killed | — |
| M15 `eventlog.rs:464` hash check skipped | survives | killed | — |
| M16, M17, M18 `sqlite_file` refusals off | killed | killed (M16) | — |
| M19 `eventlog.rs:124` password check off | killed | — | killed |
| M20 `eventlog.rs:332` no password re-check at open | killed | — | — |
| M21 `eventlog.rs:420` first page only | survives | killed (`adv_reopen…`) | — |

In 4 of 25 store-suite runs, unrelated PostgreSQL cases also failed at `open` with `deadline exceeded`. That is finding N4, not a kill. I removed those from the table.

## 3. Suite run, after the cases existed

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess` in the worktree (log: `adversary-p2/task-check.log`, exit 201):

- `ess specify validate`: ok
- drift: ok
- `cargo fmt … --check`: ok
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: ok
- `cargo test --locked --workspace --exclude thalamus-store`: ok
- `test-postgres`: started its own container, ran `adv_eventlog_store` (`test result: ok. 19 passed; …`), then stopped at mine:

```
test result: FAILED. 8 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.18s
error: test failed, to rerun pass `-p thalamus-store --test adv_p2_eventlog_store`
task: Failed to run task "check": task: Failed to run task "test": task: Failed to run task "test-postgres": exit status 101
```

Its container was removed (`docker ps -a --filter name=thalamus-test-postgres` is empty). `docs-check` did not run.

Two earlier gate runs failed on my own file, not on the unit. Run 1 failed `fmt` (`task-check-1-fmt.log`). Run 3 failed clippy `cloned_ref_to_slice_refs` (`task-check-3-clippy.log`). Both are fixed; the run above is after the fixes.

For the count, I ran `cargo test --locked -p thalamus-store --no-fail-fast` against my container (`store-no-fail-fast.log`, exit 101):

```
adv_eventlog_store: test result: ok. 19 passed; 0 failed
adv_p2_eventlog_store: test result: FAILED. 8 passed; 7 failed
conformance: test result: ok. 3 passed; 0 failed
store: test result: ok. 11 passed; 0 failed
```

The 7 red are my 6 RED cases plus `adv_p2_retry_of_another_handles_sample_is_taken_in`. That case is green alone. In both suite runs it failed at `open` (`:74`) with `deadline exceeded`, beside `…concurrent_opens…`: collateral of N4, not its own assertion.

The count: `<before>` = 330, the sum of the `test result` lines in the coordinator's `scratch/coord-check.log`, which exited 0. `<after>` = 345: the gate's 331 executed, plus the 14 cases of `conformance` and `store` that its fail-fast skipped, taken from the no-fail-fast run.

## 4. Findings

Each finding covers the working tree named in the header.

- **N1 concurrency, `crates/thalamus-store/src/eventlog.rs:589`. CONFIRMED / introduced, warning.**
  - Two writers on one stream mint the same id.
  - Ids are a store-local `max+1` read before each number (design § 2.2 as rewritten; the base said "a durable store mints a ULID").
  - The second writer's append fails with `Eventlog(IdempotencyMismatch)`, and the sample is lost. The generated `put` panics on that error.
  - Measured: deterministic with two handles. In 2 threads × 40, SQLite lost 1–2 of 80 and PostgreSQL lost 40 of 80 (gate run: 79/80 and 40/80).
  - Reaches: § 5.3 "the scheduler owns the cadence … the CLI can sample on demand … `thalamus sample --all --once`", and D24 "PostgreSQL for a shared one". No M1 caller runs two writers.
  - Fix: a writer-unique id (ULID), or a number claimed in the append transaction.
- **N2 contract-drift, `crates/thalamus-store/src/eventlog.rs:494`. NEEDS-CHANGE / introduced, warning.**
  - `append_sample` and `append_evaluation` document `Refused{ID_TAKEN}` for an id held with another body.
  - From a stale handle they return the raw `Eventlog(IdempotencyMismatch)`. Eventlog checks the idempotency key before the expected version, so the Conflict catch-up never runs.
  - Reaches: every N1 collision.
  - Fix: on `IdempotencyMismatch`, refresh and rerun `try_append_*`, which answers `ID_TAKEN`.
- **N3 concurrency, `crates/thalamus-store/src/samples.rs:44` (the same at `evaluations.rs:144`). CONFIRMED / introduced, warning.**
  - One retry is not enough under sustained contention.
  - Disjoint ids, 2 × 40: on PostgreSQL, writer 1 lost every second append (`Conflict { expected: n, actual: n+1 }` after the retry; 19–21 lost). On SQLite, 0 or 1 lost (`Conflict { expected: 6, actual: 40 }`).
  - Reaches: concurrent writers on a shared PostgreSQL instance (D24).
  - Fix: retry while `Conflict`, bounded.
- **N4 concurrency, `crates/thalamus-store/src/eventlog.rs:335`. NEEDS-CHANGE / introduced, warning.**
  - Every open runs Eventlog's `local` migration under `PoolOptions::default()` (`lock_timeout` 2 s).
  - 3 of 8 simultaneous opens fail after 2.04–2.05 s with `deadline exceeded during database statement or lock`.
  - The store's own suite (4 test threads, all on one database) hit it at `open` in 4 of 25 runs on this machine (load average 10–13).
  - Reaches: `task check` on any loaded runner (an intermittent red gate), and shared-instance processes that start together.
  - Fix: retry the open on `Deadline`, or raise `lock_timeout` for the migration.
- **N5 boundary, `crates/thalamus-store/src/eventlog.rs:333`. CONFIRMED / introduced, note.**
  - `?sslmode=require` is accepted and silently opened over plaintext: Eventlog's `isolated` forces `SslMode::Disable`.
  - The host is loopback-only, so nothing leaves the machine. Still, the operator asked for TLS and did not get it.
  - Fix: refuse a locator whose `sslmode` asks for TLS, and name D24.
- **N6 judgement, `crates/thalamus-store/src/eventlog.rs:467`. CONFIRMED / introduced, note.**
  - Open makes one `recorded_command` round trip per record. Debug build, 5000 records (2500 samples + 2500 evaluations), 3 opens each:

    | provider | open | open with the lookup skipped |
    |---|---|---|
    | SQLite | 1.93–2.03 s | 0.59–0.74 s |
    | PostgreSQL | 6.68–7.07 s | 0.88–0.99 s |

  - Cost is linear in records; every CLI invocation pays it.
  - Inferred, not measured: a 35-day window of the § 4 fleet estimate is roughly 10⁵–10⁶ records, which would be minutes per open.
  - Fix: check hashes in one batched query, or only for records past a checkpoint.
- **N7 mutant, `crates/thalamus-store/tests/store.rs`. CONFIRMED / introduced, note.**
  - 8 mutants of the catch-up and refresh code (M03–M05, M08, M10–M13) survived the implementor's suite and pass 1.
  - My green cases now kill all 8.
- **N8 judgement, `Taskfile.yml:60`. INFEASIBLE / introduced, note.**
  - `test-postgres` leaves its container running when the task is SIGKILLed: the `trap … EXIT` cannot run, and `--rm` acts only on stop.
  - Measured on a copy whose container prefix was changed to mine.
  - Reaches: `kill -9` or the OOM killer. Nothing I could show runs that way; CI uses the service container instead.

## 5. Attacked, not broken

| attack | measured result |
|---|---|
| Container cleanup | Failing tests, SIGINT to the process group and SIGTERM to `task` each left 0 containers (`cleanup-{fail,int,term}.log`). |
| CI path | With `THALAMUS_TEST_POSTGRES_URL` set, `task test-postgres` starts no container and runs the tests (`ci-path.log`). |
| CI service container | `check.yml` uses the Taskfile's image digest, trust auth, `127.0.0.1:5432` and database `postgres`, and sets the URL on the `task check` step only, as the tests expect. |
| PostgreSQL locator | A password in the user part or in `?password=` is refused, and the refusal does not repeat it. |
| Unusable servers | Port 1 failed in 0.6 ms, a missing database in 2.5 ms (`SQLSTATE 3D000`), `192.0.2.1` (not local) in 0.1 ms and a missing socket directory in 0.2 ms, all loudly. |
| Design § 2.2 / § 2.3 | The blocks, read as records, pass the open's round-trip check. |
| Same sample from two handles | Stored once. |
| D5 across handles | `DUPLICATE_EVALUATION`; 1 evaluation stored. |
| Reads racing appends | 40/40 acknowledged samples answered, no duplicate in `list`. |
| Pass-1 fixes still holding | F3 (async drop), F4 (range uses `Instant::parse`), F5 (`:memory:`, `file:`, directory), F6 (round-trip on open, M14) and the hash check (M15): their cases are green, and M14–M18 are killed. |

Not observed, only inferred: `--health-cmd "pg_isready -U postgres"` has no `-h 127.0.0.1`, so it may report healthy while the image's init-phase server listens on the socket only. The tests start minutes later in any case.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015d/m1-eventlog-store/scratch/`:

- `adversary-p2/` (52 files):
  - `case-<name>.log` (15)
  - `mutant-M00…M21.out` (21)
  - `mutants.log`, `mutants-round1.log`
  - `probe-open-cost.log`, `probe-open-cost-nohash.log`
  - `cleanup-fail.log`, `cleanup-int.log`, `cleanup-term.log`, `cleanup-kill.log`, `ci-path.log`
  - `task-check.log`, `task-check-1-fmt.log`, `task-check-2.log`, `task-check-3-clippy.log`
  - `store-no-fail-fast.log`, `store-no-fail-fast-1.log`, `store-no-fail-fast-2.log`
- `adversary-pass-2.md`: this report.

Created and already deleted:

- `adversary-p2/copy/`, the scratch copy with its probe `tests/zz_probe_open_cost.rs`. `cargo clean` removed 756.1 MiB, then I deleted the directory.
- `adversary-p2/mutants.sh` and `adversary-p2/pg.id`.

Docker:

- My PostgreSQL container `adv-p2-m1-pg-*` (id 8895ab53a3ca) was removed by id.
- Four `adv-p2-cleanup-*` containers came from the cleanup probes. Three removed themselves; the SIGKILL one (9ca5b78adc80) I removed by id, after killing its two orphaned processes by PID.

Test tempdirs under `~/.cache/claude-tmp/.tmp*` were dropped by `tempfile`; 0 remain. My lease `adv-m1-eventlog-store-p2` is released.

```findings
[
 {"file":"crates/thalamus-store/src/eventlog.rs","line":589,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Two handles on one stream mint the same store-local id (read max, add one), so the second writer's sample is lost with IdempotencyMismatch and the generated put panics; measured deterministically with two handles and as up to 40 of 80 lost in a two-thread burst."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":494,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"A stale handle storing another body under a stored id gets the raw Eventlog IdempotencyMismatch instead of the documented Refused ID_TAKEN, because Eventlog checks the idempotency key before the expected version and the conflict catch-up never runs."},
 {"file":"crates/thalamus-store/src/samples.rs","line":44,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The single catch-up retry loses appends under sustained contention: two PostgreSQL writers with disjoint ids lost 19 to 21 of 80 samples to Conflict after the retry, SQLite 0 or 1."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":335,"category":"concurrency","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Concurrent PostgreSQL opens time out after the 2 s lock_timeout of the migration every open runs (3 of 8 simultaneous opens failed), which made the store suite fail at open in 4 of 25 runs."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":333,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A PostgreSQL locator with sslmode=require is accepted and silently opened over plaintext, because Eventlog's isolated configuration forces SslMode::Disable."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":467,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Open makes one recorded_command round trip per record: 5000 records open in 1.9 to 2.0 s on SQLite and 6.7 to 7.1 s on PostgreSQL (debug), against 0.6 to 1.0 s without the lookup, linear in history."},
 {"file":"crates/thalamus-store/tests/store.rs","category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Eight mutants of the catch-up and refresh code (evaluation retry and its refresh, read-before-unknown-sample, first_sample, evaluation and current_evaluations tails, next_number refresh, dedup take-in) survived the implementor's suite and pass 1; the pass-2 green cases kill them."},
 {"file":"Taskfile.yml","line":60,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"task test-postgres leaves its PostgreSQL container running when the task is SIGKILLed, because the EXIT trap cannot run and --rm acts only on stop."}
]
```
