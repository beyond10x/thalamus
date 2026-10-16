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

`thalamus-core` grades a reading against its baseline onto this ladder and computes a pass's exit
code. Recording evaluations from a sampling pass (`thalamus eval`) is planned; see the
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
 "verdict": "WARN", "prev": "OK", "score": "1.158",
 "baseline": {"kind": "pooled", "offsets": ["1d", "7d"],
              "reading": {"quality": "ok", "value": {"value": "0.013"}}},
 "reason": "0.029 >= 2 x 0.013 and >= 0.02, 16 events of 556 >= min_events 10"}
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
   `CRIT` (an upper bound already under it, for a rule where lower is worse); anything else is
   `UNKNOWN` with reason `partial read`.
5. Otherwise the definition's rule compares the value with the baseline and the WARN and CRIT
   lines. A line is crossed when every part it declares holds: at least `x` times the baseline, at
   least `abs`, and at least `min_events` events at stake.

The rules are `ratio_vs_pooled_baseline` (higher is worse; its events are the value times the
population `n`, rounded to the nearest whole event, so 10 of 30 counts 10), `higher_is_worse` (its
events are the value) and `lower_is_worse` (a drop: a line is crossed at most `x` times the
baseline and at most `abs`, and the events at stake are the baseline's, so a count that drops from
100 to 0 crosses a line with `min_events: 10` and a drop from 3 to 0 does not). A definition naming
any other rule is refused. Both `warn` and `crit` must be declared.

An `x` part is decided only over a baseline above 0. Over a baseline of 0 or below, a line that
also declares `abs` is decided by `abs` (and `min_events`) alone, and a line that declares only `x`
grades `UNKNOWN`, with a reason naming the baseline: the instrument cannot decide, and unknown is
never `OK`.

## Persistence

A verdict change counts as a **transition** only after `k` consecutive samples agree (`k_crit`
for `CRIT`), and a `WARN` clears after `m` consecutive `OK` samples. This is Prometheus' `for:`
applied to every source; without it a flapping signal opens and closes a downstream flag on every
pass. Only `OK` clears a settled `WARN` or `CRIT`: a run of `N/A` or `UNKNOWN` never does. The count
is of consecutive samples, not of time, and any differing verdict breaks a run. `k`, `k_crit` and
`m` live on the definition as `persistence: {k, k_crit, m}`, all three or none, with defaults 2, 1
and 3. The defaults are a starting point: flap rates are to be measured over stored evaluations
(`thalamus flaps`, planned) before they are fixed per family.

Each series keeps its newest `max(k, k_crit, m)` samples. A sample reads the verdict of its
evaluation at the highest definition revision seen: when a revision re-grades history, the
re-grade replaces that sample's verdict in place, and the run is the longest stretch of the newest
samples that read one verdict. A re-grade at a revision no higher than the one counted is ignored,
and so is any evaluation of a sample that has left those newest samples. A new sample older than
the newest one taken (a sample recorded late) is dropped. Live grading and a replay of stored
evaluations read them in the order they were minted, so both drop the same late samples and settle
the same changes.

A transition is its own record: it references the `k` evaluations that settled it, one per sample,
and copies the series, the verdicts it moved between, the timestamp of its oldest sample, and the
timestamp, definition revision and score of the settling evaluation, the one that completed the
run (a re-grade of an older sample included). It outlives raw retention.

## Other vocabularies

Producers that already speak another vocabulary are mapped at the adapter, so only ladder verdicts
leave it:

| producer | mapping |
|---|---|
| a colour ladder | `green`, `yellow`, `red`, `na` to `OK`, `WARN`, `CRIT`, `UNKNOWN`. `na` never ranks as `green`. |
| a detector exit code | `1` to `CRIT`, `2` to `UNKNOWN` |
| a blocking gate | a non-zero exit to `CRIT` |
| an advisory gate | its finding count (`count_of_cases`, unit `{finding}`) against a declared `fixed` baseline: above it `WARN`, at or below `OK`, never `CRIT`; no count printed is `UNKNOWN` |

The first three are tables in the definition's own `evaluation.mapping` under the rule
`producer_mapped`, never in code: each producer value, as the source printed it, names `OK`,
`WARN`, `CRIT` or `UNKNOWN`. The sample records the value the adapter read as its `text` and
keeps its own reading; grading maps the text at the revision it grades, so revising the table
re-grades history. A read the adapter could not make records no text and stays `UNKNOWN` with its
own reason, and a value the table does not list is `UNKNOWN`. A definition naming
`producer_mapped` without a mapping, with an empty one, with one naming `N/A`, or with `warn` or
`crit` lines is refused, and so is a mapping under any other rule. A mapped verdict's score is its
position: 0 at `OK`, 1 at `WARN`, 2 at `CRIT`.

An advisory gate is graded by the rule `fixed_warn_only` instead: its finding count is an ordinary
reading, compared exactly with the definition's `fixed` baseline; trailing fractional zeros do
not count, so `3.000` is `3`. A count above it is `WARN`, scored 1. A count at or below it is
`OK`, scored `(count - baseline) / (first whole count above the baseline - baseline)`: 0 at the
baseline, however large, and below 0 under it. The score is worked out exactly and rounded only at
the end, to three decimal places. A count or baseline too long to compare or score exactly grades
`UNKNOWN` with a reason. The rule has no CRIT line, so no count, however large, and no partial read
grades `CRIT`. A partial read grades `UNKNOWN` (`partial read`). A gate that printed no count is an
`unknown` reading and grades `UNKNOWN`, never `OK` as a count of 0 would. A definition naming
`fixed_warn_only` is refused when it is proposed, revised or loaded if it has a `warn` or `crit`
line (`thresholds-unused`), a kind other than `count_of_cases` (`advisory-not-count`) or a
baseline list that is not `fixed` alone (`advisory-not-fixed`).
