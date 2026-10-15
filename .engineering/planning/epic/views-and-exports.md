---
format: aep.planning-md/3
id: epic:views-and-exports
kind: epic
status: draft
title: M4 · Heatmap, live and before/after views, and dashboard exports
relations:
- depends_on: epic:history-and-transitions
revision: 2
---
## Outcome

An operator sees every signal of an instance in one heatmap, live and over weeks.

## Ships

- heatmap, sparklines, week vs week, before/after deploy, live view in `thalamus-service`
- dashboard JSON and Prometheus rule export for Prometheus and Loki definitions

## Acceptance

- a 4-week heatmap rendered from stored samples
- one known deploy visible in before/after

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
