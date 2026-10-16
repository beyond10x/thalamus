---
title: The score
sidebar_position: 5
description: Every scored evaluation carries a normalised score, 0 at the baseline, 1 at the WARN line and 2 at the CRIT line, so signals with different units share one scale.
lede: The score is derived from the rule that decided the verdict, so a signal's colour and its verdict cannot disagree.
source: spec/domains/evaluation.yaml (SignalEvaluation.score, Verdict.scored), docs/design/thalamus.md § 3.3
---

# The score

Every `OK`, `WARN` and `CRIT` evaluation records a **score**: where the value sits between its
baseline and its thresholds.

`thalamus-core` computes the score when it grades a reading; `thalamus eval` records it with the
evaluation and `thalamus show` prints it (see [eval and show](./verdicts.md#eval-and-show)).

| value | position |
|---|---|
| below 0 | better than the baseline |
| 0 | at the baseline |
| 1 | at the WARN line |
| 2 | at the CRIT line |
| above 2 | beyond the CRIT line |

Between the marks the score is linear for an absolute line (`abs`) and log-linear for a multiplier
line (`x`); on the better side of the baseline it is linear, and so is the stretch past the CRIT
line of a rule where lower is worse, so a drop to 0 never scores better than a drop to 1. The
score never improves as the value gets worse. A line is the stricter of its `x` and
`abs` parts, the same line that decides the verdict, so the colour a view draws and the verdict
cannot disagree.

The score is rounded to three fractional digits and held inside its verdict's band: below 1 for
`OK`, from 1 to under 2 for `WARN`, 2 and over for `CRIT`. A value past a line whose `min_events`
is not met is not over the line, so its score stops just under it (0.999 under the WARN line).

## Why one scale

A failure ratio in `1`, a latency in `s` and a pod count in `{pod}` cannot share an axis. Their
scores can: one heatmap, one sort order, one "what moved most" list after a deploy. The
[views](./interfaces.md#views) are built on it.

## No score is not zero

`UNKNOWN` and `N/A` carry no score. Views draw them hatched or grey, never as 0: a signal that was
not read is not a signal at its baseline.

The score travels as a decimal string, like every number on the wire (`"score": "1.3"`).
