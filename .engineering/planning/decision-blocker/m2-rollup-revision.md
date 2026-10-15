---
format: aep.planning-md/3
id: decision-blocker:m2-rollup-revision
kind: decision-blocker
status: cleared
title: Nobody has decided which evaluation of a sample a daily rollup counts when the sample was graded at more than one revision
relations:
- blocks: epic:history-and-transitions
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:39:12Z", actor: "human:timo", revision: 3}
---
## Question

One sample can carry evaluations at more than one definition revision: "at most one evaluation per (sample, definition_rev); a re-grade after a revision is a new evaluation" (design § 11 D5, cited at `spec/domains/evaluation.yaml:228`). A daily rollup records "worst verdict, verdict counts, min/max score" per series per UTC day (design § 4). Which evaluations of a sample does it count?

## The relation

| field | state |
|---|---|
| entities | `DailyRollup` → `thalamus.evaluation.SignalEvaluation` (aggregates) |
| cardinality | many evaluations per row; which subset is not stated |
| ownership | none; the rollup derives from evaluations and owns nothing |
| lifecycle coupling | the rollup is kept after its samples and evaluations are pruned (design § 4), so it cannot be recomputed later; the choice is final per day |

Candidate answers, none taken: the highest `definition_rev` present for each sample at rollup time; the revision in force when the sample was taken (`provenance.definition_rev`, `spec/domains/observation.yaml`); one row per series, day and revision (which changes the "one row per series per UTC day" of design § 4). Each also decides whether the row records the revision it counted.

The same question reaches every per-series reader of evaluations (current verdict, hysteresis input). The live path is single-revision because the scheduler grades at the current revision; a re-grade of history is what makes it multi-revision.

## What it stops

The verdict fields of the daily rollup (worst verdict, verdict counts, min/max score). `story:m2-rollup` covers the sample-derived fields only, and `story:m2-spec-history-records` leaves the verdict fields as an `UNMAPPED:` marker.

## Clears when

The rule is stated on `DailyRollup` in `spec/domains/rollup.yaml` (as a relation or a field with its revision), `ess specify validate --path spec` is valid, and a story adds the verdict fields to `thalamus rollup`.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it.

Design § 11 D14. A daily rollup counts, for each sample, only its evaluation at the highest definition_rev present when the rollup is built. Re-running a rollup after a revision rebuilds the day.
