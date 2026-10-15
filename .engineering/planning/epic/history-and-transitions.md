---
format: aep.planning-md/3
id: epic:history-and-transitions
kind: epic
status: draft
title: M2 · History, week-over-week comparison and persistent transitions
relations:
- depends_on: epic:signal-contract
revision: 2
---
## Outcome

Thalamus compares a signal with last week from its own stored samples, and reports a verdict change only when it persists.

## Ships

- `thalamus compare`, `rollup`; baselines `same_slot_median_4w`, `rolling_median`
- hysteresis (design § 3.2)
- transitions as CloudEvents
- the scheduler in `thalamus-service`

## Acceptance

- `compare --vs 7d` answers from stored samples
- flap rate measured before `k` and `m` are fixed
- one transition emitted and cleared by real data

## Design

`docs/design/thalamus.md` § 9.

## Specification

The nouns this epic works with are declared in the ESS specification, which validates
(`ess specify validate --path spec`: `thalamus v1 — 7 file(s), valid`, 2026-10-15):

| noun | declared in |
|---|---|
| `SignalDefinition` | `spec/domains/definition.yaml` |
| `Dimension` (the registry) | `spec/domains/dimension.yaml` |
| `SignalSample`, `Reading` | `spec/domains/observation.yaml` |
| `SignalEvaluation`, `Verdict` | `spec/domains/evaluation.yaml` |
| `Assertion` | `spec/domains/assertion.yaml` |

`ess verify conform synthesize` gives 73 scenarios, 0 refusals. The open `UNMAPPED:` markers and the
decisions taken on them are in `docs/design/thalamus.md` § 11. A rule ESS cannot state is held by a
Rust test, and the marker stays until ESS can.
