---
format: aep.planning-md/3
id: decision-blocker:m3-advisory-gate-verdict
kind: decision-blocker
status: cleared
title: Nobody has decided what an advisory gate's exit maps to, or what its baseline is
relations:
- blocks: epic:source-adapters
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T01:36:12Z", actor: "human:timo", revision: 3}
---
## Question

What does an advisory gate's exit map to on the verdict ladder?

Design § 2.3, verbatim: "a gate's non-zero exit → `CRIT` for a blocking gate, `WARN` above baseline for an advisory one."

Unclear, and nothing in this repository settles it:

1. What is the baseline of an advisory gate? An exit code is one run; "above baseline" needs a series (for example the count or share of non-zero runs over a window, against `offset_7d` or `higher_of`), and the design names none.
2. What does a non-zero exit at or below that baseline map to: `OK`, or something else?
3. Is a "gate" here the Gates tool, a CI job allowed to fail, or any producer that declares itself advisory?

## What it stops

The advisory-gate row of the vocabulary mapping (epic `epic:source-adapters`, "vocabulary mapping for existing producers"). `story:m3-vocabulary-mapping` maps colours, detector exits and blocking gates without it; no story for advisory gates is drafted until this is answered.

## Clears when

The design states the advisory-gate rule (baseline kind, the series it is taken over, and the verdict at or below it), and § 2.3 is updated to match.

## Answer

Decided by the coordinator on 2026-10-15, under the operator's standing rule to take the reversible
option and report it (design § 11 D12):

- An advisory gate's sample is its **finding count**, not its exit status: kind `count_of_cases`,
  unit `{finding}`.
- Its baseline is `fixed`, the count the definition declares as the gate's accepted baseline. A
  definition may use `rolling_median` instead once stored history exists (M2).
- Count above the baseline → `WARN`; at or below → `OK`. An advisory gate never grades `CRIT`.
- A run that printed no finding count is `UNKNOWN`, never 0.

`story:m3-advisory-gate-mapping` carries the work.
