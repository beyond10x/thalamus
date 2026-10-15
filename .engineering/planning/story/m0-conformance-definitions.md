---
format: aep.planning-md/3
id: story:m0-conformance-definitions
kind: story
status: active
title: Conformance harness passes every definition and dimension scenario
relations:
- decomposes: epic:specification
- depends_on: story:m0-workspace
scope:
- confidence: inferred
  path: crates/thalamus-core/Cargo.toml
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/definition.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/dimension.rs
- confidence: inferred
  path: crates/thalamus-core/src/conformance.rs
- confidence: inferred
  path: crates/thalamus-core/tests/conformance.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T03:07:32Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T03:07:32Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Context

The epic asks for a conformance suite synthesised from `spec/` and run by `task check`. `ess generate synthesize` leaves eight capabilities as obligations (its `PLAN.md`, "Obligations — yours to implement"); until they are met their stubs answer `UnmetObligation` and most scenarios cannot pass, because nearly every scenario sets up through `ProposeDefinition`. This story builds the harness and meets the three obligations of the definition and dimension domains; `story:m0-conformance-records` meets the other five and removes the selection.

## Work

- `crates/thalamus-core/tests/conformance.rs`: reads `spec/suite.json` and runs each selected scenario in process against the generated behaviours (`thalamus_model::behaviour::Generated<_>`) over in-memory storage ports, forcing external branches and the command clock through the context port, as the cortex harness does. An unknown step, value kind, field shape or initial state fails the test; nothing is skipped. The selection is one named constant: scenario ids beginning `thalamus.definition.` or `thalamus.dimension.`.
- `crates/thalamus-core/src/behaviour/definition.rs` (inferred): the obligations `thalamus.definition.ProposeDefinition` and `thalamus.definition.ReviseDefinition`, including the guard `instant-not-fixed` (`kind == instant and exists b in evaluation.baselines: b != fixed`, error `thalamus.definition.InstantNeedsFixedBaseline`), `retired` and `no-such-definition` for revise, `already-defined` for propose, each exactly as `PLAN.md` states its contract.
- `crates/thalamus-core/src/behaviour/dimension.rs` (inferred): the obligation `thalamus.dimension.RegisterDimension` with outcomes `no-registry`, `no-parent`, `registered`, `already-registered` in the selection precedence `PLAN.md` gives for `when_related:` commands.
- The obligations read and write only through the generated storage ports; no I/O, no clock of their own.

Not here: rules the specification leaves `UNMAPPED:` (the signal id grammar at `spec/domains/definition.yaml:10`, dimension keys of a definition at `spec/domains/definition.yaml:225`, unmute on expiry, design § 11 D8 and D9). They belong to definition validation in M1 and the scheduler in M2.

## Domain relations

| relation | from → to | cardinality | ownership | used by | citation |
|---|---|---|---|---|---|
| `dimensions` | `thalamus.dimension.DimensionRegistry` → `thalamus.dimension.Dimension` | many, via `registry_id` | registry owns | `RegisterDimension` outcome `no-registry` | `spec/domains/dimension.yaml:62` |
| `parent_dimension` | `thalamus.dimension.Dimension` → `thalamus.dimension.Dimension` | one, via `parent` | references | `RegisterDimension` outcome `no-parent` | `spec/domains/dimension.yaml:92` |

`thalamus.definition.SignalDefinition` declares no relation; `ProposeDefinition` and `ReviseDefinition` read only its own fields and lifecycle (`spec/domains/definition.yaml:168`).

## Scope

Planned (inferred):

- `crates/thalamus-core/tests/conformance.rs`
- `crates/thalamus-core/src/behaviour/definition.rs`, `crates/thalamus-core/src/behaviour/dimension.rs`
- `crates/thalamus-core/Cargo.toml` (dev-dependencies such as `serde_json`); `story:m0-probes` adds its dev-dependencies to the same file after this story (`depends_on`)

The module lines in `crates/thalamus-core/src/lib.rs` and `crates/thalamus-core/src/behaviour/mod.rs` are `story:m0-workspace`'s; this story does not touch them.

## Acceptance

`cargo test -p thalamus-core --test conformance` passes, answering every scenario in `spec/suite.json` whose id begins `thalamus.definition.` or `thalamus.dimension.` (51 at ess 0.59.0) with none skipped.

## Revision after design critic round 2

The scenario runner and its in-memory storage swap live in the library module `crates/thalamus-core/src/conformance.rs`, compiled under the feature `conformance`, so other crates reuse it rather than copy it. `crates/thalamus-core/tests/conformance.rs` only calls `thalamus_core::conformance::run` with in-memory storage; `story:m1-eventlog-store`, `story:m2-rollup` and `story:m2-transitions` call the same function from `crates/thalamus-store/tests/conformance.rs` with Eventlog storage. The `conformance` module line and feature are `story:m0-workspace`'s.
