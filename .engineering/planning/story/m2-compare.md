---
format: aep.planning-md/3
id: story:m2-compare
kind: story
status: implemented
title: thalamus compare --vs <offset> answers from stored samples
relations:
- decomposes: epic:history-and-transitions
- depends_on: epic:signal-contract
- depends_on: story:m1-eventlog-store
- depends_on: story:m1-cli-sample
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/compare.rs
- confidence: cited
  path: crates/thalamus-cli/tests/adv_compare_p1.rs
- confidence: cited
  path: crates/thalamus-cli/tests/adv_compare_p2.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/compare.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T20:44:32Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-15T20:44:37Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-16T21:10:14Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Context

Epic acceptance: "`compare --vs 7d` answers from stored samples". `thalamus compare` is in the CLI of design § 5.1. It reads only the store: the point is that the comparison works for a `history: store` signal and when the source is unreachable, which separates it from the M1 `offset_7d` baseline that re-queries the source. It adds no store read: the series range read in `crates/thalamus-store/src/range.rs` is `story:m1-eventlog-store`'s (`depends_on`). The global options (`--instance`, `--store`) and the instance loading it reads the definition's cadence through are `story:m1-cli-sample`'s (`depends_on`), and the `compare` subcommand line is `story:m0-workspace`'s.

## Work

- `crates/thalamus-cli/src/cmd/compare.rs`: `thalamus compare <signal> [--dim key=value]... --vs <duration> [--at <timestamp>] [--format text|json]` (clap derive). The series key is built from the signal and dims (design § 11 D10). Through the store's series range read it takes the newest stored sample of the series at or before `--at` (default now) and the stored sample nearest to that `ts` minus the offset, within half the cadence of the definition; for `7d` the weekday matches by construction.
- Output: both readings, the delta and the ratio. Either side `unknown`, `partial` or missing, or the offset before the first sample of the series, prints `UNKNOWN` with a reason (`no stored sample at offset`, `baseline predates series`) and exits 3; otherwise exit 0 (the ladder exit codes, design § 2.3). An unknown side never prints a number.
- The command constructs no source adapter.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalSample` → `SignalDefinition` (`definition`, via `signal`): the cadence used for the match tolerance is read from the definition of the series | many-to-one | references | `spec/domains/observation.yaml:161` |

Samples are read as `thalamus.observation.SeriesSamples` (`spec/domains/observation.yaml:304`) serves them.

## Scope

Planned (inferred):

- `crates/thalamus-cli/src/cmd/compare.rs`
- `crates/thalamus-cli/tests/compare.rs` (new)

## Acceptance

`cargo test -p thalamus-cli --test compare` passes the failing-first tests `compare_vs_7d_reads_stored_samples` (a store fixture, no source connection configured), `compare_missing_offset_sample_is_unknown_exit_3`, `compare_offset_before_series_is_unknown`, `compare_unknown_side_never_prints_a_number` and `compare_series_key_from_dims`.

## Revision (adversary passes 1 and 2)

- "The first sample of the series" in the output line means the start of the series as design line 115 and § 11 D18 define it: the definition's resolved `denominator.since`, and the first stored sample only when `since` is `first_sample` or absent (adversary pass 2, P2-F2, P2-F3).
- A compare on an instance that has never sampled creates no store and answers as an empty one (pass 1, F3); a store the user cannot write is unavailable, exit 74, as for every command (pass 2, P2-F4).
