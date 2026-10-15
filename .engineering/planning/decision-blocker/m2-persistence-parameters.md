---
format: aep.planning-md/3
id: decision-blocker:m2-persistence-parameters
kind: decision-blocker
status: cleared
title: Nobody has decided whether hysteresis k and m live on each definition or on its family
relations:
- blocks: epic:history-and-transitions
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:39:12Z", actor: "human:timo", revision: 3}
---
## Question

Where do the hysteresis parameters `k` and `m` (design § 3.2) live once they are fixed: on each definition, or on its family?

## The relation

| field | state |
|---|---|
| entities | `SignalDefinition` → family, many-to-one; the parameters are owned by one side |
| cardinality | many definitions per family (`thalamus.definition.SignalDefinition.family`, `spec/domains/definition.yaml:177`) |
| ownership | not stated: the family owns `k` and `m`, or each definition owns its own |
| lifecycle coupling | not stated: a family is a newtype (`thalamus.definition.Family`, `spec/domains/definition.yaml:19`), not an entity, so it has no lifecycle and no revision to carry a parameter change |

What the sources say, verbatim:

- design § 3.2: "The defaults are guesses; milestone M2 measures flap rates and fixes them per family."
- `AGENTS.md`: "One copy of every threshold, in its definition."

"Per family" reads as a family-level home, which needs a family entity (a new noun) or an instance file; "in its definition" reads as a field on `thalamus.definition.EvaluationSpec`, set alike on every definition of a family. The two produce different schemas and a different answer to what a revision of `k` is.

## What it stops

Fixing `k` and `m` from the measurement: the second half of the epic acceptance "flap rate measured before `k` and `m` are fixed". Not drafted. Drafted around it: `story:m2-hysteresis-core` takes `k` and `m` as parameters with the design defaults, and `story:m2-flap-measurement` produces the measurement this decision should read.

## Clears when

The home is declared in the ESS specification (a field on `EvaluationSpec`, or a family entity with a `relations:` entry from `SignalDefinition`), `ess specify validate --path spec` is valid, and a story writes the measured values there.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D15. k and m live on each definition as `persistence: {k, k_crit, m}`; an absent block means the defaults k = 2, k_crit = 1, m = 3. Family-level defaults are not built before M3. `story:m2-hysteresis-core` reads them from the definition.
