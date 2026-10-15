---
format: aep.planning-md/3
id: decision-blocker:m1-dimension-value-origin
kind: decision-blocker
status: cleared
title: 'Nobody has decided where a dimension value creation time comes from for since: dimension.created'
relations:
- blocks: epic:signal-contract
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:42:48Z", actor: "human:timo", revision: 3}
---
## Question

Where does a dimension value creation time come from, so that a definition declaring `denominator.since: dimension.created` can give its samples a `since` and an `age_s`?

## What the sources say

- design § 2.1 example, verbatim: "denominator: { unit: \"{request}\", min_n: 20, since: dimension.created }".
- design § 2.2, verbatim: "`denominator.since`, `age_s` | the earliest moment the population could contain anything: the dimension creation time, the source retention start, the first sample of the series. A baseline whose offset exceeds `age_s` is `UNKNOWN`".
- `spec/domains/definition.yaml`, `thalamus.definition.DenominatorSpec.since` is a `String`, "as a reference (dimension.created, source retention start, first sample)"; the reference vocabulary is not closed.
- `spec/domains/dimension.yaml`: `thalamus.dimension.Dimension` is a key, not a value; `thalamus.dimension.DimensionValue` is a newtype with no entity, so no record carries a value creation time.

## The relation

| field | state |
|---|---|
| entities | `thalamus.observation.SignalSample.denominator.since` → the creation time of one `thalamus.dimension.DimensionValue` |
| cardinality | not stated: which value when a sample has several dims (the leaf, the oldest, the newest) |
| ownership | not stated: a field of an inventory read, or of a registry file |
| lifecycle coupling | not stated: what a value re-created under the same name does to `since` |

## What it stops

Rule 2 of design § 3.1 (a baseline older than its series is `UNKNOWN`) for definitions that declare `since: dimension.created`. The M1 stories hold rule 2 for `since: first_sample` and refuse other references at instance load with a named error (`story:m1-instance-registry`).

## Clears when

The design states where a value creation time is read and which value a multi-dimension sample uses, and `spec/domains/dimension.yaml` declares it if it is a record; then a story resolves `dimension.created`.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D18. Through M3, `denominator.since` admits `first_sample` and a fixed RFC 3339 timestamp. `dimension.created` is admitted only for a `file` registry entry that carries an optional `created` timestamp; an entry without one is refused at load with a named error. Until a series has a week of samples, its 1-day and 7-day baselines grade UNKNOWN, as rule 2 of § 3.1 says.
