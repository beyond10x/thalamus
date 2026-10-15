---
title: Assertions, events and views
sidebar_position: 6
description: What callers get out of thalamus — assertions answered with the ladder's exit codes, transitions as CloudEvents, facts in cortex, test runs in both directions, and views.
lede: A release, a merge request, a test run or a controller can ask whether named signals stayed within baseline, and thalamus answers with a result, not a guess; unknown is not a pass.
source: spec/domains/assertion.yaml, docs/design/thalamus.md § 6, § 11 (D1, D2, D3, D13)
---

# Assertions, events and views

:::caution[Planned]

Everything on this page is planned. The assertion is specified (`thalamus.assertion.Assertion`);
the command, the event stream, the cortex source, the test-run bridge and the views are not built.
See the [status page](/docs/status).

:::

## Assertions

An **assertion** ([`Assertion`](../reference/ess/thalamus-assertion.md#assertion)) asks whether
named signals stay within baseline over a window after an instant:

```console
$ thalamus assert platform.namespace.request_failure_ratio --dims namespace=shared-1 \
      --after 2026-10-14T20:08:00Z --for 2h --within baseline
```

The exit code is the ladder's: 0 within, 1 or 2 breach, 3 unknown. **Unknown is not a pass.**

- Every signal an assertion names must exist and be `Active`; otherwise the ask is refused.
- The answer comes only after `after + horizon` has passed. A window in which every evaluation is
  `N/A` answers `unknown`.
- The answer is part of the assertion record: its result (`within`, `breach`, `unknown`), the
  worst verdict, a reason, and the evaluations it read.
- The record names who asked: a `release`, a `merge_request`, a `test_run` or a `controller`.

| asker | how |
|---|---|
| a release | after the rollout, assert the release's declared set for every environment it reached |
| a merge request | an `Asserts:` line in its description names signals and a horizon; it is re-run when the horizon passes |
| [conductor](https://beyond10x.github.io/conductor/) ([GitHub](https://github.com/beyond10x/conductor)) or a controller | assert before closing a wave, and read the live verdict stream as an input to scheduling |

## Events

Every evaluation and every transition is available as a CloudEvents event on the wire:

| attribute | value |
|---|---|
| `type` | `dev.beyond10x.thalamus.evaluation`, `dev.beyond10x.thalamus.transition` |
| `source` | the instance and the producer |
| `subject` | the series key |
| `time` | the evaluation's `ts` |

Stored lines are not wrapped in the envelope: no reader inside the store needs it.

## Cortex

For an instance connected to [cortex](https://beyond10x.github.io/cortex/)
([GitHub](https://github.com/beyond10x/cortex)), thalamus is a cortex source: definitions, daily
rollups and transitions become facts, each citing the evaluation that produced it. Cortex never
receives raw samples.

## Test runs

| direction | mapping |
|---|---|
| test run to thalamus | a finished case becomes a sample of `test.case.passed{project,suite,case,target}` (1 or 0); a run becomes `test.run.pass_ratio`. An instrument failure is `unknown`, a defect a failing value. A red suite on release day shows in the same heatmap as the platform signals. |
| thalamus to test run | a release-day run carries signal assertions; each result is emitted as a case in the run's stream, so the test report shows "platform signals stayed in baseline" beside the scenarios |

## Views

Served by `thalamus-service`:

| view | shape |
|---|---|
| heatmap | rows are series grouped by layer, columns hours or days, each cell the [score](./score.md) on a diverging scale; unknown hatched, `N/A` grey |
| sparklines | the value line, the baseline band, the WARN and CRIT lines; gaps drawn as gaps |
| before and after a deploy | every signal's window before against after one deploy instant, sorted by score change |
| week vs week | this week against the same slot last week |
| live | the current verdict per series, updated from the event stream |

A **deploy instant** is the time a workload actually changed (pod creation, merge time), never an
announcement.

Dashboard JSON and Prometheus recording and alert rules are generated from definitions as an
**export**, and only for definitions whose source is Prometheus or Loki. A dashboard tool cannot see
SQL, orchestrator or version-control signals, so it is never the only view.
