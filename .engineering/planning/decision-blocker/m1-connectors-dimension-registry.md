---
format: aep.planning-md/3
id: decision-blocker:m1-connectors-dimension-registry
kind: decision-blocker
status: cleared
title: 'Nobody has decided which Connectors read a connectors: dimension registry resolves its values from'
relations:
- blocks: epic:signal-contract
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:42:47Z", actor: "human:timo", revision: 3}
---
## Question

Which Connectors operation does a dimension whose values come from a `connectors:` registry read, and what in its answer is a dimension value?

## What the sources say

- design § 2.4, verbatim: "cluster:   { registry: connectors:inventory/clusters }", "namespace: { registry: connectors:inventory/namespaces, parent: cluster }".
- design § 10 question 4, verbatim: "Is the dimension registry read through Connectors, from files in the instance, or both?" Default: "both, as § 2.4 shows".
- `spec/domains/dimension.yaml`, type `thalamus.dimension.ValueSource`, variant `connectors: thalamus.dimension.Locator`; `thalamus.dimension.Locator` summary: "The inventory or registry file a key values are listed in (inventory/clusters, registries/components.json)".
- `spec/domains/observation.yaml:142` (`UNMAPPED:` on `dims`): "value resolution reads a Connectors inventory or a file outside the model".
- Connectors v0.45.0 exposes no operation called an inventory; its adapters answer typed reads (for example `namespaces.list` on the Kubernetes adapter, per its README). Nothing maps a `Locator` such as `inventory/namespaces` to a Connectors adapter instance, connection and operation.

## The relation

| field | state |
|---|---|
| entities | `thalamus.dimension.Dimension` (values `connectors:`) → a Connectors connection and operation |
| cardinality | not stated: one operation per key, or one per registry |
| ownership | not stated: whether thalamus caches the value list, and for how long |
| lifecycle coupling | not stated: whether a value the inventory drops refuses later samples carrying it |

## What it stops

Resolving dimension values through Connectors. The M1 stories admit `file:` and `open` registries and refuse a `connectors:` registry at instance load with a named error (`story:m1-instance-registry`).

## Clears when

The design names, for a `connectors:` locator, the adapter instance, connection and read operation, and which field of its answer is the value; then a story resolves it.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D17. Through M3 an instance's dimension registry is `file` or `open`. A `connectors:` registry is refused when the instance loads, with an error naming the missing Connectors inventory operation. Supporting it waits for that operation in Connectors and is a milestone after M3. This does not block M1: `story:m1-instance-registry` already refuses it.
