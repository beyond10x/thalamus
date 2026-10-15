---
title: The verdict ladder
sidebar_position: 3
description: Every evaluation grades one sample into one verdict of one ladder (OK, N/A, WARN, CRIT, UNKNOWN), with one worst-of order and one exit-code mapping.
lede: One ladder for every source, in which UNKNOWN is a verdict of its own and never folds into OK.
source: spec/domains/evaluation.yaml (Verdict, SignalEvaluation), docs/design/thalamus.md § 2.3, § 3.1 rule 4, § 3.2, § 11 (D5, D6, D12, D15)
---

# The verdict ladder

An **evaluation** ([`SignalEvaluation`](../reference/ess/thalamus-evaluation.md#signalevaluation))
is one verdict over one sample, with the rule and the definition revision that decided it.

:::caution[Planned]

The ladder is specified: the generated `Verdict` type carries each verdict's exit code, rank and
whether it is scored. Grading samples into evaluations is planned; see the
[status page](/docs/status).

:::

## The ladder

| verdict | meaning | exit | rank | scored |
|---|---|---|---|---|
| `OK` | read, graded, inside bounds | 0 | 0 | yes |
| `N/A` | read, but too little to grade (`n < min_n`) | 0 | 0 | no |
| `WARN` | out of bounds | 1 | 2 | yes |
| `CRIT` | far out of bounds | 2 | 3 | yes |
| `UNKNOWN` | the sample is unknown, or its baseline is | 3 | 1 | no |

The specification spells `N/A` as `N_A`. The worst of several verdicts is the one with the highest
rank: `CRIT > WARN > UNKNOWN > OK = N/A`. A pass exits with its worst verdict's exit code, except
that a pass in which every verdict is `N/A` exits 3: having read nothing gradable is not a pass.

`N/A` is not `OK`, and `UNKNOWN` is never `OK`. An overall verdict therefore cannot depend on the
order in which signals were read.

## An evaluation

```json
{"evaluation_id": "evl_000000000042", "sample_id": "smp_000000000042",
 "signal": "platform.namespace.request_failure_ratio",
 "series": "platform.namespace.request_failure_ratio{cluster=\"prod-a\",namespace=\"shared-1\"}",
 "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z", "definition_rev": 1,
 "verdict": "WARN", "prev": "OK", "score": "1.3",
 "baseline": {"kind": "pooled", "offsets": ["1d", "7d"],
              "reading": {"quality": "ok", "value": {"value": "0.013"}}},
 "reason": "0.029 >= 2 x 0.013 and >= 0.02, 15 failures of 556"}
```

| field | rule |
|---|---|
| `reason` | required on every verdict other than `OK` |
| `score` | the [normalised score](./score.md); absent on `UNKNOWN` and `N/A` |
| `baseline` | the [baseline](./baselines.md) the sample was compared with: its kind, its offsets and its own reading |
| `prev` | the previous raw verdict of the same series, so a change is visible without a second read |
| `definition_rev` | the revision that decided it. At most one evaluation per sample and revision: re-grading after a threshold change writes a new evaluation and leaves the old one |

## How a reading is graded

In this order:

1. A sample whose source answered a population below `min_n` is `N/A`, with no score.
2. An `unknown` reading is `UNKNOWN`, with the reading's reason.
3. A baseline that could not be read, or that predates its series, is `UNKNOWN`.
4. A `partial` reading may escalate but never clear: a lower bound already over the CRIT line is
   `CRIT`; anything else is `UNKNOWN` with reason `partial read`.
5. Otherwise the definition's rule compares the value with the baseline and the WARN and CRIT
   lines. A line is crossed when every part it declares holds: at least `x` times the baseline, at
   least `abs`, and at least `min_events` events.

## Persistence

A verdict change counts as a **transition** only after `k` consecutive evaluations agree (`k_crit`
for `CRIT`), and a `WARN` clears after `m` consecutive `OK` evaluations. This is Prometheus' `for:`
applied to every source; without it a flapping signal opens and closes a downstream flag on every
pass. `k`, `k_crit` and `m` live on the definition, with defaults 2, 1 and 3. The defaults are a
starting point: flap rates are to be measured over stored evaluations (`thalamus flaps`, planned)
before they are fixed per family.

A transition is its own record: it references the `k` evaluations that settled it and copies the
series, the definition revision, the verdicts it moved between, and the first and settling
timestamps and score. It outlives raw retention.

## Other vocabularies

Producers that already speak another vocabulary are mapped at the adapter, so only ladder verdicts
leave it:

| producer | mapping |
|---|---|
| a colour ladder | `green`, `yellow`, `red`, `na` to `OK`, `WARN`, `CRIT`, `UNKNOWN`. `na` never ranks as `green`. |
| a detector exit code | `1` to `CRIT`, `2` to `UNKNOWN` |
| a blocking gate | a non-zero exit to `CRIT` |
| an advisory gate | its finding count (`count_of_cases`, unit `{finding}`) against a declared `fixed` baseline: above it `WARN`, at or below `OK`, never `CRIT`; no count printed is `UNKNOWN` |
