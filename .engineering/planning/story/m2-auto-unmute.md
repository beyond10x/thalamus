---
format: aep.planning-md/3
id: story:m2-auto-unmute
kind: story
status: draft
title: The service unmutes a definition whose mute has expired
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-scheduler
- depends_on: story:m2-spec-history-records
- depends_on: story:m1-cli-sample
scope:
- confidence: inferred
  path: crates/thalamus-pipeline/src/definitions.rs
- confidence: inferred
  path: crates/thalamus-service/src/scheduler.rs
- confidence: inferred
  path: crates/thalamus-service/src/unmute.rs
- confidence: inferred
  path: crates/thalamus-service/tests/unmute.rs
- confidence: inferred
  path: crates/thalamus-store/src/definition_events.rs
revision: 8
---
## Context

Design § 11 D9 (`spec/domains/definition.yaml:241`): "the service unmutes a definition whose expiry has passed, issuing `UnmuteDefinition` as actor `service`". Design § 11 D19 settles where that state lives: the definition file is the only copy of a definition, lifecycle included, and the service never rewrites it. A `Muted` definition whose expiry has passed is sampled as `Active`; the service records the unmute as an event in its store so the change is visible, and logs a warning naming the stale file on each pass. ESS has no time-triggered move (the `UNMAPPED:` marker above that line), so the rule is held by the scheduler hook and Rust tests. `story:m2-spec-history-records` adds the actor `thalamus.definition.Service`; the event is the existing `thalamus.definition.DefinitionUnmuted` (`spec/domains/definition.yaml:346`).

The definition ports of `story:m1-instance-registry` stay read-only: this story issues no command against them. The store port for the event is this story's own file.

## Work

- `crates/thalamus-store/src/definition_events.rs`: a store port that appends a `DefinitionUnmuted` event (signal id, the definition `rev`, the expired `mute_until`, actor `thalamus.definition.Service`, the tick time) to the instance stream and reads the events of a definition; an event for the same `(signal, rev, mute_until)` is appended once.
- `crates/thalamus-service/src/unmute.rs`: on every tick, before sampling, each `Muted` definition whose `mute_until` is at or before now is handed to the pass as `Active`, its `DefinitionUnmuted` event is recorded once, and a warning naming the definition file is logged on every pass while the file keeps the stale mute. A `Muted` definition whose `mute_until` is in the future stays unsampled. The service never mutes and never writes a definition file.
- `crates/thalamus-service/src/scheduler.rs` (owned by `story:m2-scheduler`): one hook line in the tick, before sampling, calling `unmute::effective_definitions`. The first of the three hook lines; `story:m2-rollup` adds the next after this story (`depends_on`).

## Domain relations

No relation between entities. The story reads one entity's lifecycle and records one of its events: `SignalDefinition` transition `unmute`, `Muted` → `Active` (`spec/domains/definition.yaml`, lifecycle and `UnmuteDefinition`, `spec/domains/definition.yaml:638`); `mute_until` is the field at `spec/domains/definition.yaml:223`. Per D19 the transition takes effect for sampling and in the store, never in the definition file.

## Scope

Planned (inferred):

- `crates/thalamus-service/src/unmute.rs`, `crates/thalamus-store/src/definition_events.rs` (their module lines are `story:m0-workspace`'s)
- `crates/thalamus-service/src/scheduler.rs` (one hook line; shared with `story:m2-scheduler`, `story:m2-rollup`, `story:m2-transitions`, ordered by `depends_on`)
- `crates/thalamus-service/tests/unmute.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test unmute` passes the failing-first tests `expired_mute_is_sampled_as_active`, `unexpired_mute_stays_muted`, `expired_mute_unmutes_without_rewriting_the_file` (the definition file's bytes and modification time are unchanged after the tick), `unmute_event_is_recorded_once_as_service_actor` (one `DefinitionUnmuted` event in the store, actor `thalamus.definition.Service`, after three ticks) and `stale_mute_is_logged_naming_the_file_each_pass`.

## Revision after design critic round 2

The effective-lifecycle overlay is this story's: `crates/thalamus-pipeline/src/definitions.rs` wraps the file-backed definition port so that a `Muted` definition whose expiry has passed answers `Active` to the generated `RecordSample` guard, and every other definition answers its file lifecycle. Added failing-first test `expired_mute_passes_record_sample_as_active` in `crates/thalamus-service/tests/unmute.rs`. Depends on `story:m1-cli-sample`, which creates the pass this overlay wraps.
