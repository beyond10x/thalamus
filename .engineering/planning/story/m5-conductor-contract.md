---
format: aep.planning-md/3
id: story:m5-conductor-contract
kind: story
status: draft
title: 'Conductor contract: a declared-set answer before a wave closes and a transition feed with a cursor'
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-declared-sets
- depends_on: story:m2-transitions
- depends_on: story:m5-skeleton
- depends_on: story:m4-live
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/transitions.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/conductor_contract.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/conductor_contract
- confidence: inferred
  path: crates/thalamus-store/src/transition_feed.rs
- confidence: inferred
  path: docs/integrations/conductor.md
revision: 7
---
## Outcome

Conductor can close a wave on a thalamus answer and read thalamus transitions between cycles, through a command-line contract thalamus pins with named contract tests. Thalamus changes nothing in the conductor repository.

## Context

Design § 6.1: "conductor / a controller: `thalamus assert --declared` before closing a wave, and the live verdict stream as an input to scheduling". What conductor exposes today (read 2026-10-15, conductor `README.md`, `docs/design/conductor.md`): no hook or command runs a check before a wave closes; a wave is closed by a repository controller following `.agents/repo-controller.md` ("A wave is done when integrated and clean"). Conductor reads every other system through command-line programs (`gh`, `glab`, `aep`, `ess`), and between cycles its watch (`conductor watch run`, design § 8) runs cheap probes and prints one line per change, keeping its state under `<state>/watch/`. So the thalamus side is a command-line contract: `thalamus assert --declared` (`story:m5-declared-sets`), `thalamus show --format json` (`story:m1-cli-eval-show`) and a transition feed with a cursor (this story).

The transition envelope comes from the one CloudEvents builder, `crates/thalamus-pipeline/src/cloudevents.rs`, which `story:m4-live` moves there from `crates/thalamus-service/src/events.rs` (coordinator decision C6, round 1); this story `depends_on` `story:m4-live` for it. `thalamus-cli` already depends on `thalamus-pipeline`, so this story adds no dependency to `crates/thalamus-cli/Cargo.toml` and never links `thalamus-service` into the CLI.

## Work

- `crates/thalamus-store/src/transition_feed.rs`: every transition recorded after a transition id, or at or after a time, across all series, in append order.
- `crates/thalamus-cli/src/cmd/transitions.rs`: `thalamus transitions [--after-id <trn_...> | --since <ts>] [--format cloudevents|json]`: one CloudEvents JSON line per transition, the envelope of design § 6.2 (`type` `dev.beyond10x.thalamus.transition`, `subject` the series key of D10, `time` the settling ts, `id` the transition id), built by `thalamus_pipeline::cloudevents::envelope`, the same call `crates/thalamus-service/src/events.rs` makes, so the CLI line and the service event are the same bytes for one transition.
- `crates/thalamus-cli/tests/fixtures/conductor_contract/`: golden files `assert-declared.json`, `transitions.jsonl` and `show.json`, whose key sets and value types are the contract; conductor can vendor them from a thalamus tag.
- `docs/integrations/conductor.md`: the contract (commands, exit codes 0, 1, 2, 3, 75, 64, 65, 74, the JSON shapes, the cursor) and the conductor-side work below.

## What conductor must add (conductor repository, not this story)

1. `.agents/repo-controller.md`, at a wave close: when the charter names a declared set, run `thalamus assert --declared <file> --after <merge instant of the wave> --asker controller:<repository> --wait --format json` before reporting the wave done. 0 is done; 1 or 2 is a breach reported to conductor naming the assertion ids; 3 is not a pass and is reported as unknown; 75 is only seen without `--wait` and means not yet answerable.
2. `conductor.config/1`: an optional per-instance block naming the thalamus instance directory and, per repository or in the charter, the declared-set file; `conductor doctor` lists `thalamus` as optional, `not needed` without the block.
3. A watch probe `signals`: each `cadence.watch`, `thalamus transitions --after-id <cursor> --format cloudevents`, printing `signal <to verdict>: <subject>` per event and keeping the cursor under `<state>/watch/`.
4. The compare step of the loop ("red signals", conductor design § 4) reads `thalamus show --format json`.
5. A conductor contract test over the vendored golden files of this story.

No decision in conductor is needed to build the thalamus side, so no blocker is filed.

## Decisions taken here (decomposer calls, reversible)

- The live verdict input is a polled feed with a cursor, not the server-sent-events stream of design § 5.1 (`GET /api/evaluations/stream`, `story:m4-live`): a poll with a cursor fits the probe model of the conductor watch and needs no running HTTP listener. Conductor may use the stream instead later; the feed stays.
- The contract is the command line, not an HTTP API, because every other input conductor reads is a program it runs.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `Transition` → `SignalEvaluation`, `evaluations` | many | references; outlives them (D13) | declared by `story:m2-spec-history-records`; design § 11 D13 |
| `Transition` → series (signal id plus dims) | many-to-one | key | `thalamus.observation.SeriesKey`, `spec/domains/observation.yaml:20` (D10) |
| `Asker.kind` `controller` → `Assertion` | one-to-many | value | `spec/domains/assertion.yaml:17` |

## Scope

Planned (inferred):

- `crates/thalamus-store/src/transition_feed.rs`, `crates/thalamus-cli/src/cmd/transitions.rs` (module and subcommand lines are `story:m5-skeleton`'s)
- `crates/thalamus-cli/tests/conductor_contract.rs`, `crates/thalamus-cli/tests/fixtures/conductor_contract` (new)
- `docs/integrations/conductor.md` (new)
- Not `crates/thalamus-service/src/events.rs` and not `crates/thalamus-cli/Cargo.toml`.

## Acceptance

- `cargo test -p thalamus-cli --test conductor_contract` passes, against the built binary and a seeded store, the failing-first test `transitions_feed_resumes_after_cursor_without_repeats` and `transitions_lines_are_cloudevents_with_series_subject`, `wave_close_declared_set_exit_codes_match_contract` (0, 1, 2, 3 and 75 from five seeded sets), `assert_declared_json_matches_golden_shape` and `show_json_matches_golden_shape`.
- `cargo tree -p thalamus-cli -e normal --depth 1` does not list `thalamus-service`.

## Revision after acceptance critic round 2

Added failing-first test: `transitions_since_timestamp_lists_every_transition_at_or_after_it` for the `--since <ts>` cursor form.
