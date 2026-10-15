---
format: aep.planning-md/3
id: story:m2-live-observation
kind: story
status: draft
title: 'Seven days of real sampling: one transition opened and cleared, flap rate measured, k and m fixed'
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-flap-measurement
- depends_on: story:m2-transitions
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 3
---
## Outcome

On one real instance, scheduled sampling has run for at least 7 days, one transition has been opened and cleared by real data, the flap rate is measured, and `k`, `k_crit` and `m` are fixed in the definitions from that measurement.

## Context

`epic:history-and-transitions` acceptance: "flap rate measured before `k` and `m` are fixed" and "one transition emitted and cleared by real data". Design § 3.2: "The defaults are guesses; milestone M2 measures flap rates and fixes them per family"; § 11 D15 puts `persistence: {k, k_crit, m}` on each definition. The commands and their tests are `story:m2-flap-measurement` and `story:m2-transitions` (`depends_on` both); this story is the observation on real data, which needs an operator: a running `thalamus-service` with a saved, validated Connectors connection on the machine that runs it, as `story:m1-live-pass` does. Thalamus holds no credential; the connection does.

## Work

- An instance directory outside this repository, its definitions `Active`, sampled by `thalamus-service run` for at least 7 days on the File or SQLite store provider.
- The transition: from the service's CloudEvents output, one `dev.beyond10x.thalamus.transition` event that opens `WARN` or `CRIT` on a series and a later one that clears it to `OK`, both from real reads (the Connectors double is not used). Recorded as `health_observation` evidence on this story: the two event ids, their `time`, the from and to verdicts, and the instance-neutral count of evaluations each references; no series key, family name, instance name or host.
- The flap rate: `thalamus flaps --since 7d --sweep --format json` after the 7 days; its aggregate counts are recorded as `metric_observation` evidence on this story, with no series key, family name, instance name or host (the repository is public).
- The fix: an edit of the instance's definitions that sets `persistence: {k, k_crit, m}` from the sweep, whose commit message cites the `metric_observation` record. Recorded as `diff` evidence on this story naming the instance commit id only.
- Design § 3.2 gets one dated line: the measured settled changes per series-day at the default and at the chosen values, and the values chosen, each naming the command it was read from. No instance data enters the repository.

## Domain relations

- `Transition` → `SignalEvaluation`, references, many (design § 11 D13), declared by `story:m2-spec-history-records`: the observed transition cites the real evaluations that settled it.
- `SignalDefinition.persistence` (design § 11 D15): the field the fix sets.

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 3.2, the file exists)
- Nothing else in this repository; the instance, its definitions and its store stay outside it.

## Acceptance

- `aep plan artifact history story:m2-live-observation` lists a `health_observation` record naming one opening and one clearing transition event from real reads.
- It lists a `metric_observation` record carrying the `thalamus flaps --since 7d --sweep` aggregate over at least 7 days of scheduled sampling of one real instance.
- It lists a `diff` record naming the instance commit that sets `persistence` on the definitions and cites that `metric_observation` record.
- Design § 3.2 states the measured flap rates and the chosen `k`, `k_crit` and `m`, dated and naming the command each was read from.
