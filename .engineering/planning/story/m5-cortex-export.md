---
format: aep.planning-md/3
id: story:m5-cortex-export
kind: story
status: draft
title: Definitions, daily rollups and transitions export as cortex structured-source records
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-spec-integration-records
- depends_on: story:m5-skeleton
- depends_on: story:m2-transitions
- depends_on: story:m2-rollup
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/cortex.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/cortex_export.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/cortex_export
- confidence: inferred
  path: crates/thalamus-pipeline/src/cortex_export.rs
- confidence: inferred
  path: crates/thalamus-store/src/rollups.rs
- confidence: inferred
  path: crates/thalamus-store/src/transitions.rs
- confidence: inferred
  path: docs/integrations/cortex-sources.yaml
- confidence: inferred
  path: docs/integrations/cortex.md
revision: 6
---
## Outcome

An instance connected to cortex runs `thalamus cortex export --out <dir>`, and a cortex instance whose spec carries the three documented `structured` sources turns definitions, daily rollups and transitions into facts; each transition fact cites the evaluations that settled it. Cortex never receives a raw sample.

## Context

Design § 6.4: "thalamus is a cortex source: definitions, daily rollups and transitions become facts, each citing the evaluation that produced it. Cortex never receives raw samples." Design § 4: only the change someone has to act on is forwarded. How cortex adds a source (read 2026-10-15, cortex `README.md` and `website/docs/spec-file.md`, `kind: structured`): a `structured` source with `from: files` reads JSON files matching a glob, takes the record array at `records`, maps each record by a fixed `mapping` (`node_type`, `id`, `name`, `aliases`, `properties`, `relations`) with no model call, and cites each fact to `record:<identity>`, the identity being `files:<paths>:<glob>:<id>`. So cortex needs no new source kind and no code change: thalamus writes files in that shape. The record types (`thalamus.cortex-export/1`) are `story:m5-spec-integration-records`'s.

## Work

- `crates/thalamus-pipeline/src/cortex_export.rs`: three files, `definitions.json`, `rollups.json`, `transitions.json`, each the generated envelope with its `records`:
  - a definition record per definition revision found in the instance files, with the digest of the file;
  - a rollup record per `DailyRollup` in the store;
  - a transition record per `Transition`, with its `k` evaluation ids and the settling evaluation copied as `evaluation`; once that evaluation has left raw retention, `evaluation_state` is `expired` and the ids stay (D13).
  Records sorted by id, no export time inside, each file written to a temporary name and renamed, so an unchanged store gives byte-identical files and cortex sees no change.
- `crates/thalamus-cli/src/cmd/cortex.rs`: the `cortex` group and `export --out <dir>` (default `<instance>/.thalamus/export/cortex`).
- `docs/integrations/cortex-sources.yaml`: the three cortex sources (`kind: structured`, `from: files`, `records: $.records`; node types `Signal`, `SignalRollup`, `SignalTransition`; relation `of_signal` to `Signal` by `$.signal`; relation `settled_by` to `SignalEvaluation` by `$.evaluations`; property `evaluation_id` from `$.evaluation.evaluation_id`). No comments in the file.
- `docs/integrations/cortex.md`: how an instance wires it, what a fact cites, and that the export is run from a timer beside the cortex source timers.

## What cortex must add

Nothing. An operator adds the sources of `docs/integrations/cortex-sources.yaml` to a cortex instance spec. Optional for cortex: a fixture test running a `structured` source over a thalamus export sample, so a change in the mapping semantics shows in cortex first.

## Decisions taken here (decomposer calls, reversible)

- Files read by the existing `structured` kind, not a new cortex source kind or a Connectors adapter for thalamus: it needs no change in another repository.
- The export is a command run on a timer, not a scheduler hook: it is recomputed from the store each time, so a missed run loses nothing (lesson L7 is about samples nobody takes).
- Only transition facts cite evaluations; definition and rollup facts cite their own record (decided in `story:m5-spec-integration-records`). Design § 6.4 says every fact cites "the evaluation that produced it"; a definition is produced by no evaluation and a rollup by many it outlives.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `Transition` → `SignalEvaluation`, `evaluations` | many | references; outlives them, a missing one reads as expired | declared by `story:m2-spec-history-records`; design § 11 D13 |
| `DailyRollup` → `SignalDefinition`, `definition` | many-to-one | references | declared by `story:m2-spec-history-records` |
| one export record → one cortex node and its facts, cited as `record:files:<paths>:<glob>:<id>` | one-to-one | cortex keeps every revision; the export does not own it | inferred from cortex `website/docs/spec-file.md` (`kind: structured`, Identity); no `ess/1` document declares it |

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/src/cortex_export.rs`, `crates/thalamus-cli/src/cmd/cortex.rs` (module and subcommand lines are `story:m5-skeleton`'s)
- `docs/integrations/cortex-sources.yaml`, `docs/integrations/cortex.md` (new)
- `crates/thalamus-cli/tests/cortex_export.rs`, `crates/thalamus-cli/tests/fixtures/cortex_export` (new)

## Acceptance

`cargo test -p thalamus-cli --test cortex_export` passes, against the built binary and a seeded store, the failing-first test `transition_record_cites_its_settling_evaluation` and `expired_evaluation_is_marked_expired_not_dropped`, `export_is_byte_identical_when_nothing_changed`, `export_holds_no_raw_sample`, `one_definition_record_per_revision` and the contract test `every_documented_mapping_path_resolves_on_every_record` (reads `docs/integrations/cortex-sources.yaml` and resolves its `records`, `id`, `name`, property and relation paths on every exported record).

## Coordinator note after round 2 (2026-10-15)

Two critics could not establish an all-series read for the export. This story adds the reads it needs, `transitions_since(ts)` and `rollups_for_day(day)` across all series, to `crates/thalamus-store/src/transitions.rs` and `crates/thalamus-store/src/rollups.rs` (owned by `story:m2-transitions` and `story:m2-rollup`, both ancestors of this story), with the named test `export_reads_every_series_transitions_and_rollups`.
