---
format: aep.planning-md/3
id: review-result:adversary-m1-eventlog-store-pass-1
kind: review-result
status: active
title: Adversary, story:m1-eventlog-store, pass 1
relations:
- reviews: story:m1-eventlog-store
revision: 1
---
unit: story:m1-eventlog-store — uncommitted working tree of impl/m1-eventlog-store (base ca445f1), worktree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-eventlog-store
verdict: red (strongest NEEDS-CHANGE)
cases: executed 310→330, red 8
origin: introduced 8, pre-existing 1, undecided 0 (adversary)
wrote-outside-worktree: 24 paths under ~/.cache/thalamus-waves/20261015d/m1-eventlog-store/scratch/ (part 6)
needs-coordinator: yes — F1's fix is in docs/design/thalamus.md, outside the story's scope; F2 needs a decision: catch-up-on-conflict in the store, or a single-writer rule that design § 5.3 does not have

## 1. Diff stat

`git --no-pager diff --stat` after my work is the same as before it: `17 files changed, 1696 insertions(+), 43 deletions(-)`. I changed no tracked file. `git status` shows one new untracked file that is mine: `crates/thalamus-store/tests/adv_eventlog_store.rs` (908 lines, test file). `tests/store.rs` was already untracked and belongs to the implementor. No non-test path was touched. `b10x-gates scan-text` on my file returned exit 0.

## 2. Cases added

All cases are in `crates/thalamus-store/tests/adv_eventlog_store.rs`. Each was run alone with `cargo test --locked -p thalamus-store --test adv_eventlog_store -- --exact <name>` before any suite run. Logs: `scratch/adversary/case-<name>.log`.

| case | asserts | now |
|---|---|---|
| `adv_stored_records_have_the_shape_design_2_2_and_2_3_document` :157 | stored record members and JSON kinds = the design § 2.2 / § 2.3 json blocks | RED |
| `adv_range_read_orders_as_the_series_samples_view` :202 | `series_range` over all samples orders them as core `series_samples` does | RED |
| `adv_second_handle_never_answers_unknown_sample_for_a_stored_sample` :347 | a 2nd handle never answers `unknown-sample` for a sample on the stream | RED |
| `adv_long_lived_handle_records_after_another_handle_appended` :381 | a long-lived handle still appends after another handle appended (design § 5.3) | RED |
| `adv_sqlite_locator_is_a_file_or_refused` :411 | `sqlite::memory:` is durable across reopen, or refused | RED |
| `adv_sqlite_stream_with_a_value_altered_does_not_open_altered` :517 | a byte-altered SQLite value refuses open, or comes back unaltered | RED |
| `adv_foreign_unknown_reading_with_a_value_never_comes_back_with_it` :605 | a foreign `sample/1` whose unknown reading has a `value` is refused at open, or `records()` shows no value | RED |
| `adv_store_opened_for_an_async_caller_drops_without_panic` :756 | a store opened via `spawn_blocking` (as lib.rs advises), then dropped in async code, does not panic | RED |
| `adv_range_read_is_half_open_at_both_ends` :242 | `[from, to)`: a sample at `to` is excluded, one at `from` included | green (kills M1) |
| `adv_retry_writes_nothing_and_another_body_is_refused` :276 | a same-body retry is Ok with the head unmoved; a different body gets `ID_TAKEN`; samples and evaluations both | green (kills M2) |
| `adv_sequence_resumes_past_foreign_and_malformed_ids` :309 | ids `smp_12ab`, `smp_`, `nounderscore`, `evl_…50` as a sample id: the next smp number is 8, the next evl number is 51 | green |
| `adv_file_stream_with_its_last_line_removed_does_not_open_short` :490 | File provider, last line cut: open is loud or intact | green |
| `adv_file_stream_with_a_value_altered_does_not_open_altered` :507 | File provider, `0.029`→`0.039`: loud or intact | green |
| `adv_sqlite_file_truncated_does_not_open_short` :531 | SQLite file cut to half: loud or intact | green |
| `adv_foreign_schema_refuses_the_open` :631 | a `thalamus.sample/2` record refuses open, and the error names the schema | green |
| `adv_escaped_text_and_baselines_round_trip` :661 | escapes, U+2028, emoji, nested baseline Readings, `-0.000100`, a 41-digit decimal: exact, both providers | green |
| `adv_reopen_reads_past_one_page` :725 | 1001 records: 1001 samples after reopen, head 1001, next number 1002 | green (83 s) |
| `adv_open_inside_a_runtime_panics_as_documented` :785 | open inside a runtime panics, as lib.rs documents | green |
| `adv_whole_suite_holds_on_the_{file,sqlite}_provider` :901/:906 | all 73 suite scenarios over Eventlog-backed ports | green (7.2 s / 7.3 s) |

Red output at first run, verbatim (`/home/<user>` shortened to `~`):

```
---- adv_stored_records_have_the_shape_design_2_2_and_2_3_document stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:189:5:
### 2.2 `SignalSample`
  documented: {"denominator": "object", "dims": "object", "id": "string", "provenance": "object", "reading": "object", "reason": "null", "schema": "string", "signal": "string", "ts": "string", "unit": "string", "window": "object"}
  stored:     {"denominator": "object", "dims": "object", "provenance": "object", "reading": "object", "sample_id": "string", "schema": "string", "series": "string", "signal": "string", "ts": "string", "unit": "string", "window": "object"}
### 2.3 `SignalEvaluation`
  documented: {"baseline": "object", "definition_rev": "number", "dims": "object", "prev": "string", "reason": "string", "sample": "string", "schema": "string", "score": "number", "signal": "string", "ts": "string", "verdict": "string"}
  stored:     {"baseline": "object", "definition_rev": "number", "dims": "object", "evaluation_id": "string", "prev": "string", "reason": "string", "sample_id": "string", "schema": "string", "score": "string", "series": "string", "signal": "string", "ts": "string", "verdict": "string"}

---- adv_range_read_orders_as_the_series_samples_view stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:227:9:
assertion `left == right` failed: file:~/.cache/claude-tmp/.tmpnWCNVo/file-store: range read and SeriesSamples view disagree
  left: ["smp_000000000002", "smp_000000000003", "smp_000000000001"]
 right: ["smp_000000000001", "smp_000000000002", "smp_000000000003"]

---- adv_second_handle_never_answers_unknown_sample_for_a_stored_sample stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:343:13:
file:~/.cache/claude-tmp/.tmpygTZFL/file-store: the second handle answered unknown-sample for smp_000000000001 which is on the stream

---- adv_long_lived_handle_records_after_another_handle_appended stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:369:9:
file:~/.cache/claude-tmp/.tmp8ByhYD/file-store: the long-lived handle cannot record after another handle appended: Err(Eventlog(Conflict { expected: 0, actual: 1 }))

---- adv_sqlite_locator_is_a_file_or_refused stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:373:5:
assertion `left == right` failed: sqlite::memory:: the sample recorded a moment ago is gone after a reopen; nothing is durable
  left: 0
 right: 1

---- adv_sqlite_stream_with_a_value_altered_does_not_open_altered stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:408:13:
assertion `left == right` failed: sqlite:~/.cache/claude-tmp/.tmpwiFW6h/thalamus.sqlite: after altering 0.029 to 0.039 in the SQLite file the store opened and answered other records
  left: ["0.5", "0.039"]
 right: ["0.5", "0.029"]

---- adv_foreign_unknown_reading_with_a_value_never_comes_back_with_it stdout ----
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:560:5:
an unknown reading came back from the store with a value: {"quality":"unknown","value":{"reason":"timeout","value":"0"}}

---- adv_store_opened_for_an_async_caller_drops_without_panic stdout ----
panicked at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.53.3/src/runtime/blocking/shutdown.rs:51:21:
Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context.
panicked at crates/thalamus-store/tests/adv_eventlog_store.rs:700:5:
dropping the store in the async caller panicked
```

The line numbers above are from before `rustfmt` ran on my file; the test bodies did not change. Every red run ended `test result: FAILED. 0 passed; 1 failed; … 18 filtered out` (19 for the case added second).

Mutants. A scratch copy was mutated; the worktree never was. Each row gives exit and result for the implementor's targets `store` and `conformance`, then for mine:

| mutant | store | conformance | adv_eventlog_store |
|---|---|---|---|
| M1 `range.rs:39` `at < to` → `at <= to` | 0, 10 passed | 0, 3 passed | 101, killed by `adv_range_read_is_half_open_at_both_ends` |
| M2 `samples.rs:35` `held == *sample` → `false` | 0, 10 passed | 0, 3 passed | 101, killed by `adv_retry_writes_nothing_and_another_body_is_refused` |
| M3 `eventlog.rs:186` `create_dir_all(parent)` → `metadata(parent)` | 0, 10 passed | 0, 3 passed | 101, killed by 13 cases |

## 3. Suite runs (after the cases existed)

Run 1: `task check ESS=~/.cache/ess/toolchains/0.60.0/ess` (log: `scratch/adversary/task-check.log`)
- `ess specify validate`: ok.
- drift: ok.
- `cargo fmt … -- --check`: ok.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: ok.
- `cargo test --locked --workspace`: stopped at the first red target:

```
test result: FAILED. 12 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 132.68s
error: test failed, to rerun pass `-p thalamus-store --test adv_eventlog_store`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
```

Exit status 201. docs-check did not run.

Run 2, for the count: `cargo test --locked --workspace --no-fail-fast` (log: `scratch/adversary/suite-no-fail-fast.log`). Summed `test result` lines: 322 passed, 8 failed, 330 executed. Exit 101.

`<before>` = 310. It is the sum of the `test result` lines in the implementor's gate log `scratch/task-check.log` (exit 0). I did not have the implementor's `cases:` line.

## 4. Findings

- **F1 contract-drift** (`docs/design/thalamus.md:94`, `:122`). NEEDS-CHANGE / introduced.
  - The story says records are "the wire forms … of design § 2.2 and § 2.3". The stored records differ from those blocks:
    - § 2.2 has `id` where the store writes `sample_id`, and a top-level `reason: null` the store does not write. It has no `series`.
    - § 2.3 has `sample` where the store writes `sample_id`, and no `evaluation_id` or `series`.
    - § 2.3 has `score` as a number (1.3), against D11. Its `baseline` holds `value`/`quality`, where the store writes `{kind, offsets, reading}`.
  - The unit edited both lines (the id examples) and left the members. `stored_line_is_not_cloudevents_wrapped` calls the `sample_id` key set "the § 2.2 sample".
  - Reaches: `story:m1-live-pass` measures stored lines against § 4, and every reader of § 2.
  - Fix: rewrite both blocks to the stored shape.
- **F2 concurrency** (`crates/thalamus-store/src/eventlog.rs:273`). CONFIRMED / introduced.
  - `head` is cached at open and never refreshed. After another handle appends, every append from this handle fails with `Conflict { expected: 0, actual: 1 }`, and `put` turns that into a panic. Reads stay stale: `record_evaluation` answers the declared outcome `unknown-sample` for a stored sample.
  - Reaches: design § 5.3 (the scheduler owns the cadence, and "the CLI can sample one definition on demand"). That is `story:m2-scheduler` beside `thalamus sample`. M1 has no long-lived handle.
  - Fix: on `Conflict`, read the stream after `head`, keep what it holds, and retry once. Re-read before answering "absent".
- **F3 judgement/async** (`eventlog.rs:157`, `src/lib.rs` doc). NEEDS-CHANGE / introduced.
  - Dropping the store's own `Runtime` inside async code panics, even when it was opened on a blocking thread as lib.rs advises.
  - Reaches: `story:m4-service-api` tests build a router "over a seeded store" with tokio (`oneshot`), and the service's shared state is dropped at shutdown.
  - Fix: hold `Option<Runtime>` and call `shutdown_background()` in `Drop`, or document "drop on a blocking thread".
- **F4 contract-drift** (`crates/thalamus-store/src/range.rs:26`). INFEASIBLE / introduced.
  - The range read parses with `time`'s RFC 3339, which also accepts a space separator and `:60`. The view uses core `Instant::parse`, the ESS grammar. Order differs, and ts texts that are not instants vanish from the range but stay in the view.
  - Reaches: nothing found. The pipeline writes `ts` itself, and the generated decoder admits any text.
  - Fix: use core's instant reader.
- **F5 boundary** (`eventlog.rs:184`). INFEASIBLE / introduced.
  - `sqlite::memory:` parses and opens a non-durable store. I infer, untested, that rusqlite's default `SQLITE_OPEN_URI` flag also lets `sqlite:file:…?mode=memory` through.
  - Reaches: an operator-typed locator. No caller parses a locator today.
- **F6 boundary** (`eventlog.rs:296`). INFEASIBLE / introduced.
  - `keep` decodes and does not check that the record re-encodes to its stored body. A foreign `sample/1` with an extra `value` in an unknown reading opens silently, and `Record::body()` hands the value back.
  - Reaches: only another writer on the stream.
  - Fix: the append's round-trip check, applied on open.
- **F7 boundary** (Eventlog SQLite provider, read side). INFEASIBLE / pre-existing.
  - A value altered in the SQLite file comes back altered. The File provider refuses the same edit. Inferred from behaviour; I did not read that code path.
- **F8 mutant** (`crates/thalamus-store/tests/store.rs:39`). CONFIRMED / introduced.
  - The test helper `open` creates the SQLite parent directory itself, so the store's own `create_dir_all` (`eventlog.rs:186`) is untested.
  - M1 and M2 also survive `store` and `conformance`. My green cases now kill all three.
- **F9 judgement** (`crates/thalamus-store/src/evaluations.rs:342`). CONFIRMED / introduced.
  - A D5 duplicate through the generated `put` panics. `story:m1-cli-eval-show` records "through the generated RecordEvaluation". Two `eval` runs racing, or F2's stale handle, crash the process instead of returning a refusal. The brief already discloses this.

## 5. Attacked, not broken

- Ids:
  - No reissue across reopen.
  - Ids that are malformed, have an empty number, or belong to the other prefix are ignored.
  - Past `smp_999999999999`, `mint` returns `UnmetObligation`. `checked_add` covers `u64::MAX`.
- Ordering: ts ties keep append order; mixed offsets are compared as instants; order is the same after reopen; reads past the 1000-record page are correct.
- Reads: `first_sample` is the first appended. `newest_evaluations` lets a re-grade at the same instant win. `unevaluated_samples` is per rev and per signal.
- Decimal: `-0.000`, `-0.000100`, trailing zeros and 41-digit values round-trip exactly on both providers; `1e-7` is refused before any write.
- Damage on File (line cut, byte altered) and a SQLite file cut to half are all loud.
- Schema `/2` refuses open and names the schema. Unknown readings, including nested baseline readings, carry no value.
- All 73 suite scenarios pass over Eventlog on both providers.
- No panic is reachable from the public reads (`get`, `list`, `sample`, `evaluation`, `series_range`, `newest_evaluations`, `unevaluated_samples`, `first_sample`).
- Open inside a runtime panics, as documented.
- The idempotency key is scoped per stream, and the D5 projection key `sample#rev` cannot collide.
- Append cost (debug build, one run, scratch probe since deleted):
  - File: 102–220 ms per append, reopen of 1200 records 2983 ms.
  - SQLite: 16–30 ms per append, reopen 133 ms.
  - No quadratic growth seen up to 1200 records. Scale beyond that: I don't know.
- Not tested: a SQLite file held locked by another process. Holding an fcntl lock needs `libc`/`rusqlite` in dev-dependencies, and I may not change `Cargo.toml`.

## 6. Paths written outside the worktree

All under `~/.cache/thalamus-waves/20261015d/m1-eventlog-store/scratch/`:
- `adversary/case-<name>.log`: 20 files, one per case.
- `adversary/mutants.log`
- `adversary/task-check.log`
- `adversary/suite-no-fail-fast.log`
- `adversary-pass-1.md`: this report.

Created and already deleted: `adversary/mutant-tree/` (scratch copy, `cargo clean` removed 597.7 MiB, then deleted), its probe `tests/zz_probe_append_cost.rs`, and `adversary/mutants.sh`.

Test runs created tempdirs under `~/.cache/claude-tmp/.tmp*`. `tempfile` removes them on drop; the `:memory:` probe created no file in the crate directory (checked).

My worktree lease `adv-m1-eventlog-store-p1` is released.

```findings
[
 {"file":"docs/design/thalamus.md","line":94,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Stored sample/evaluation records do not match the design § 2.2/§ 2.3 records the story names (id/sample vs sample_id, missing series and evaluation_id, reason null, score and baseline value as JSON numbers against D11)."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":273,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"A handle never refreshes its cached head: after another handle appends, every append fails with Conflict forever and reads answer unknown-sample for a stored sample, which the design § 5.3 scheduler-plus-CLI workflow reaches."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":157,"category":"judgement","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Dropping the store's private Tokio runtime in async code panics even when opened on a blocking thread as lib.rs advises, which the M4 service and its tokio tests will hit."},
 {"file":"crates/thalamus-store/src/range.rs","line":26,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"The range read parses ts with the time crate's wider RFC 3339 grammar, so its order and membership differ from the SeriesSamples view it claims to bound."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":184,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"The locator sqlite::memory: opens a non-durable in-memory store silently instead of a database file or a refusal."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":296,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"Open does not round-trip-check stored records, so a foreign record with extra members (a value in an unknown reading) opens silently and Record::body returns it."},
 {"file":"crates/thalamus-store/src/eventlog.rs","line":184,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"pre-existing","message":"On the SQLite provider a value altered in the database file is returned altered where the File provider refuses the open."},
 {"file":"crates/thalamus-store/tests/store.rs","line":39,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The unit's suite survives three mutants (range upper bound inclusive, same-body retry refused, no SQLite parent creation, masked by the test helper creating the directory); the adversary cases now kill them."},
 {"file":"crates/thalamus-store/src/evaluations.rs","line":342,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A D5 duplicate through the generated put panics the process, and the eval story records through that port."}
]
```
