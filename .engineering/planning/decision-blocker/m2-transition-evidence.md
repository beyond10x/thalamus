---
format: aep.planning-md/3
id: decision-blocker:m2-transition-evidence
kind: decision-blocker
status: cleared
title: Nobody has decided which evaluations a settled transition cites, or whether it outlives them
relations:
- blocks: epic:history-and-transitions
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:39:11Z", actor: "human:timo", revision: 3}
---
## Question

A settled transition (design § 3.2) is its own record (design § 11 D6, cited at `spec/domains/evaluation.yaml:329`). Which `thalamus.evaluation.SignalEvaluation` does it reference, and with which cardinality: the one evaluation that completed the run (one), or the k evaluations of the run (many)? And does the transition outlive what it cites?

## The relation

| field | state |
|---|---|
| entities | `SettledTransition` → `thalamus.evaluation.SignalEvaluation` |
| cardinality | not stated: one (the settling evaluation) or many (the run) |
| ownership | references; evaluations are append-only (`spec/domains/evaluation.yaml`, lifecycle `Recorded`) |
| lifecycle coupling | not stated: evaluations are retained for a bounded window (design § 4, default 35 days); a transition is what someone acts on and is forwarded to cortex (design § 6.4), so it may outlive the evaluations it cites |

No `ess/1` document declares the transition entity yet. What the prose says, verbatim:

- design § 6.4: "transitions become facts, each citing the evaluation that produced it"
- design § 4: "transitions | emitted as events (§ 6) and, for an instance connected to cortex, as facts with the evaluation as evidence"
- `epic:assertions-and-integrations` acceptance: "one transition appears in cortex with its evaluation as evidence"

All three read singular; none says whether the reference must survive retention (a dangling reference, a copy of the cited verdicts and timestamps carried on the transition, or evaluations behind a transition exempt from pruning).

## What it stops

The story that records settled transitions and emits them as CloudEvents (`dev.beyond10x.thalamus.transition`, design § 6.2), and with it the epic acceptance "one transition emitted and cleared by real data". That story is not drafted. `story:m2-spec-history-records` drafts the transition entity with this relation as an `UNMAPPED:` marker.

## Clears when

The answer is a `relations:` entry on the transition entity in `spec/domains/transition.yaml` (cardinality and lifecycle coupling stated), `ess specify validate --path spec` is valid, and the record-and-emit story is drafted against it.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D13. A settled transition references the k evaluations of the run that settled it (references, cardinality many). It copies what it needs to stand alone: series, definition_rev, from, to, the ts of the first and the settling evaluation, and the settling score. Transitions are durable like daily rollups and outlive raw-sample retention; their evaluation references may then dangle, and a reader must treat a missing evaluation as expired, not as an error. `story:m2-transitions` carries the work.
