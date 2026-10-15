---
format: aep.planning-md/3
id: epic:source-adapters
kind: epic
status: draft
title: M3 · Every source kind speaks one verdict vocabulary
relations:
- depends_on: epic:signal-contract
revision: 2
---
## Outcome

Every source kind an instance needs speaks the one verdict vocabulary.

## Ships

- Loki, SQL, Kubernetes, Git and HTTP probe adapters
- vocabulary mapping for existing producers (design § 2.3)

## Acceptance

- one verdict vocabulary across all sources
- failing-first test: a colour ladder's `na` cannot rank as `green`

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
