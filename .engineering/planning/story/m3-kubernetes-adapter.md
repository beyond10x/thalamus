---
format: aep.planning-md/3
id: story:m3-kubernetes-adapter
kind: story
status: draft
title: Kubernetes object counts become instant samples with source-only dimensions
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/kubernetes.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/kubernetes
- confidence: inferred
  path: crates/thalamus-sources/tests/kubernetes.rs
revision: 3
---
## Context

Design § 5.1 lists Kubernetes among the `thalamus-sources` adapters. § 1 L8: an instant count (pods present now) is not trended; § 1 L9: a dimension is set only when the source carries it, never derived. § 5.2: read only through a Connectors connection, no secret, never a write. This story adds the `kubernetes` adapter on the M1 adapter trait and Connectors client (`epic:signal-contract`). Code is Rust (AGENTS.md).

## Connectors

Exists. Connectors v0.45.0 (`94e0805ed`), `adapters/kubernetes/` (`adapters/kubernetes/design.md`): `resources.list` (kinds `pods`, `services`, `deployments`, `endpointslices`, `replicasets`, `events`, within the configured namespaces, with continuation), `resources.get`, `namespaces.list`, `deployments.history`. `adapters/kubernetes/design.md:21`: the untyped item schema of `resources.list` does not guarantee a status projection. `pods.exec` is a write and is never invoked.

## Behaviour

- `kubernetes.count`: the number of listed objects of one kind that match the definition predicate, one sample per namespace. The signal kind (`instant`) comes from the definition; the adapter computes no delta or trend (L8).
- A list whose continuation could not be finished: `partial`, bound `lower`.
- An object that lacks the status field the predicate reads: the sample is `quality: unknown` with reason `status not projected`; it is never counted as 0 or as a match.
- `namespace` comes from the object metadata. No other key is derived: `tenant` is never set from a namespace (L9, `rule: source_only`). A `cluster` key taken from the connection rather than the object is not decided; until it is, the adapter sets no `cluster` key.
- A Connectors refusal or provider failure: `quality: unknown`; the reason carries the Connectors code and stage.

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one; the sample references the definition, which exists first and outlives it. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `Dimension → Dimension` (namespace under cluster), many-to-one; the child references its parent key. Inferable: `spec/domains/dimension.yaml`, entity `thalamus.dimension.Dimension`, relation `parent_dimension` (references, cardinality one, via `parent`). The rule that a `source_only` key is never derived from its parent is the type `thalamus.dimension.DimensionRule` in the same file.
- `DimensionRegistry → Dimension`, one-to-many, the registry owns its keys. Inferable: `spec/domains/dimension.yaml`, entity `thalamus.dimension.DimensionRegistry`, relation `dimensions` (owns, cardinality many, via `registry_id`).
- `dims` keys → registered `Dimension`: no `relations:` entry; `spec/domains/observation.yaml` field `dims` carries an `UNMAPPED:` marker, decided by design § 2.4 and held by an M1 Rust test. Relied on, not implemented here.

## Acceptance

`cargo test -p thalamus-sources --test kubernetes` passes, with these known-answer tests over recorded Connectors answers in `crates/thalamus-sources/tests/fixtures/kubernetes/`, each seen failing before the adapter exists: `kubernetes_pod_count_per_namespace_known_answer`, `kubernetes_unfinished_continuation_is_partial_lower_bound`, `kubernetes_missing_status_is_unknown_never_zero`, `kubernetes_tenant_never_derived_from_namespace`, `kubernetes_refused_read_is_unknown_with_reason`.

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-sources/src/kubernetes.rs` (replaces the one-line `adapter()` stub `story:m1-connectors-client` leaves there; the module line is `story:m0-workspace`'s and the registry arm `story:m1-connectors-client`'s, so no other file changes)
- `crates/thalamus-sources/tests/kubernetes.rs` (new)
- `crates/thalamus-sources/tests/fixtures/kubernetes` (new)

## Not in this story

Deploy instants from `deployments.history` and pod creation (M4 before/after view, design § 6.5).
