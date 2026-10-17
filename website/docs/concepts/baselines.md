---
title: Baselines
sidebar_position: 4
description: A sample is graded against what is normal for it, a baseline. A baseline that could not be read, or that is older than its series, is UNKNOWN.
lede: Every comparison names its baseline, and a baseline that cannot be trusted makes the verdict UNKNOWN instead of quietly comparing against too little history.
source: spec/domains/definition.yaml (BaselineKind, History), spec/domains/evaluation.yaml (Baseline), docs/design/thalamus.md § 3.1, § 11 (D4, D18)
---

# Baselines

A sample is graded against a **baseline**: what is normal for this series. A definition lists the
baseline kinds it is graded against (`evaluation.baselines`), and every evaluation records the
baseline it used: its kind, its offsets and its own [reading](./samples.md#the-reading).

## Kinds

| kind | what | needs |
|---|---|---|
| `offset_1d`, `offset_7d` | the same window 1 day or 7 days earlier, weekday-matched for 7 days | `history: source`: the source can be re-queried at an offset |
| `pooled` | the sum of the baselines' numerators over the sum of their denominators | a ratio |
| `higher_of`, `lower_of` | the higher or the lower of the listed baselines, tolerant of daily and weekly shape: errors against the higher, volume drops against the lower | a count |
| `same_slot_median_4w` | the median of the same weekday and hour over the last 4 weeks | 4 weeks of samples or source history |
| `rolling_median` | the median of the last N samples of this series | `history: store`: only thalamus remembers past values |
| `fixed` | a number in the definition | a hard zero, or no history yet |

The design's example grades a failure ratio against `pooled` over the offsets `1d` and `7d`: the
failures of yesterday's and last week's windows over their requests.

## Rules

1. A baseline that could not be read is `UNKNOWN`, never 0.
2. A baseline older than its series is `UNKNOWN` with reason `baseline predates series`. A
   "week over week" comparison against a series that is three days old compares against nothing.
3. A signal of kind `instant` (pods present now) gets no delta and no trend below its
   `min_trend_window`; it is graded against `fixed` only. Two reads of an instant count minutes
   apart can differ by a quarter.
4. A `partial` sample may escalate but never clear; see
   [How a reading is graded](./verdicts.md#how-a-reading-is-graded).

## Store baselines

`same_slot_median_4w` and `rolling_median` are computed from the series' own stored samples, read
through the store, never from the source. A definition that names one of them names no other
baseline beside it.

- **`same_slot_median_4w`**: the slots are the UTC hours holding the sample's time minus 7, 14,
  21 and 28 days, so the weekday and the hour are UTC, whatever offset a timestamp is written in.
  Each slot's value is its `ok` sample closest to that time; the baseline is the median of the
  four, recorded with offsets `7d`, `14d`, `21d` and `28d`. A slot with no `ok` sample makes the
  baseline `UNKNOWN` with reason `same slot not read`: a missing week is not a median of three. A
  slot whose hour ends before the series began makes it `UNKNOWN` with reason `baseline predates
  series`.
- **`rolling_median`**: the median of the last `evaluation.rolling_n` `ok` samples before the
  graded one; `partial` and `unknown` samples are skipped. A series of fewer than
  `rolling_n` samples of any status makes the baseline `UNKNOWN` with reason `baseline predates
  series`; a series of `rolling_n` samples or more with fewer than `rolling_n` `ok` ones makes it
  `UNKNOWN` with reason `too few ok samples`.

The median of an even count is the mean of the two middle values, computed exactly in decimal
(the median of `0.1` and `0.2` is `0.15`).

## How old is a series

A sample's `denominator.since` is the earliest moment its population could contain anything, and
`age_s` is the time since. A baseline whose offset exceeds `age_s` predates the series. `since` is
one of:

| value | when |
|---|---|
| `first_sample` | the first sample of the series |
| a fixed timestamp | a source whose retention starts at a known time |
| `dimension.created` | only for a dimension value from a registry file whose entry carries `created` |

## Comparing a series with its past

`thalamus compare <signal> [--dim key=value]... --vs 7d [--at <timestamp>]` answers from stored
samples only, so it works for a `history: store` signal and while the source is unreachable. It
takes the newest stored sample of the series at or before `--at` (default now) and the stored
sample nearest to that sample's time minus the offset, within half the definition's cadence; for
`7d` the weekday matches by construction. It prints both readings, the delta and the ratio, and
exits 0. When either side is missing, `unknown` or `partial`, or the offset reaches before the
first sample of the series, it prints `UNKNOWN` with a reason (`no stored sample at offset`,
`baseline predates series`) and exits 3; an unknown side never prints a number. The [week vs week view](./interfaces.md#views) draws the same
comparison over a whole week.
