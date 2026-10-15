---
format: aep.planning-md/3
id: epic:signal-contract
kind: epic
status: draft
title: M1 · One source produces typed samples and evaluations end to end
relations:
- depends_on: epic:specification
revision: 3
---
## Outcome

One source kind produces typed samples and evaluations end to end, and the rules that keep unknown apart from 0 are tests.

## Ships

- `thalamus-core`: the verdict ladder, baselines `offset_1d`, `offset_7d`, `pooled`, `higher_of`, `lower_of`, `fixed`, the normalised score
- `thalamus-store` on Eventlog
- the Prometheus adapter in `thalamus-sources`
- `thalamus sample`, `eval`, `show`

## Acceptance

- failing-first tests: an unknown sample carries no value and is never `OK`; a ratio is numeric with unit `1`; every sample names an `Active` definition; a baseline older than its series is `UNKNOWN`
- one live read-only pass against a real instance; its line count and byte size replace the inferred numbers in design § 4

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
