---
format: aep.planning-md/3
id: decision-blocker:m2-sampling-gap-record
kind: decision-blocker
status: cleared
title: Nobody has decided whether a missed sampling slot is a record of its own or only a missing sample
relations:
- blocks: epic:history-and-transitions
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:39:12Z", actor: "human:timo", revision: 3}
---
## Question

Is a missed sampling slot a record of its own, or only the absence of a sample at a cadence slot?

## What the sources say

- design § 5.3, verbatim: "A gap in sampling is recorded as a gap and drawn as one, never interpolated."
- `spec/domains/observation.yaml:328` (`thalamus.observation.SeriesSamples`), verbatim: "A gap in sampling is a gap here, never interpolated."

The second settles that no value is synthesised and that the series view shows the gap as missing rows. Neither settles whether the scheduler writes anything when a slot passes unsampled (service down, clock jump, a pass that never started). "Recorded as a gap" can mean a gap record; it can also mean that the absence is what is recorded. Unclear; the design owner has to say.

## The relation, if a gap is a record

| field | state |
|---|---|
| entities | `SamplingGap` → `thalamus.definition.SignalDefinition` (and the series it belongs to) |
| cardinality | not stated: one gap per missed slot, or one per contiguous run of missed slots |
| ownership | not stated |
| lifecycle coupling | not stated: whether a gap is written when it is detected (on restart) or closed when sampling resumes |

No `ess/1` document declares a gap noun.

## What it stops

Recording gaps beyond their absence. `story:m2-scheduler` is drafted on what both readings share: a missed slot is not backfilled and no sample is synthesised for it. Writing a gap record, and counting gaps in a rollup, is not drafted.

## Clears when

Either the design states that a gap is the absence of a sample (and this blocker clears with no further story), or a gap noun is declared in `spec/domains/` with its relations, `ess specify validate --path spec` is valid, and a story records it from the scheduler.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D16. A missed slot is a sample with quality `unknown` and reason `missed_slot`, written by the scheduler when it next runs; there is no separate gap record. An unknown sample has no value, so views draw it as a gap and never interpolate it. `story:m2-scheduler` carries it.
