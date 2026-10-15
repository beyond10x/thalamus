---
format: aep.planning-md/3
id: epic:assertions-and-integrations
kind: epic
status: draft
title: M5 · Assertions, conductor, test runs and cortex
relations:
- depends_on: epic:history-and-transitions
revision: 2
---
## Outcome

Releases, merge requests, test runs and conductor ask thalamus whether named signals stayed in baseline, and cortex remembers the transitions.

## Ships

- `thalamus assert`; `Asserts:` lines
- conductor integration
- test-run bridge both ways
- thalamus as a cortex source

## Acceptance

- one release asserts its declared set
- one test run carries signal assertions and its cases appear as samples
- one transition appears in cortex with its evaluation as evidence

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
