---
format: aep.planning-md/3
id: epic:specification
kind: epic
status: draft
title: M0 · The thalamus nouns are specified in ESS and the model is generated
revision: 3
---
## Outcome

The five nouns of thalamus have one ESS domain, and the Rust model types are generated from it.

## Ships

- ESS domain under `spec/` for `SignalDefinition`, `SignalSample`, `SignalEvaluation`, `DimensionRegistry`, `Assertion`
- generated `thalamus-model`
- `Taskfile.yml` with `task check` running spec validity and generated-code drift
- GitHub Actions runs `task check` on every push and pull request

## Acceptance

- the two probes of design § 8 answered: `quality: unknown` ⇔ `value: null` and a UCUM-typed unit, each expressed in ESS or held by a Rust test with an `UNMAPPED:` marker
- a conformance suite synthesised from `spec/`, run by `task check`

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
