---
title: Samples
sidebar_position: 2
description: A sample is one typed reading of one signal for one series. Its reading is ok, partial or unknown, and an unknown reading carries no value at all.
lede: A sample records what was read and how completely; unknown is a reading of its own, never 0 and never a formatted string.
source: spec/domains/observation.yaml, docs/design/thalamus.md § 2.2, § 4, § 5.3, § 11 (D10, D11, D16)
---

# Samples

A **sample** ([`SignalSample`](../reference/ess/thalamus-observation.md#signalsample)) is one
reading of one signal for one series at one instant. Samples are append-only: a sample is recorded
and never changed. Grading it is a separate record, the [evaluation](./verdicts.md).

`thalamus sample <signal-id>` records one sample per series of one `Active` definition, and
`thalamus sample --all --once` does so for every `Active` definition of the instance, all under one
run id. An unreadable source is recorded as an `unknown` sample, and the command exits 0. An invalid
instance, or a definition that is not `Active`, exits 65 and records nothing; a store that cannot be
opened exits 74. A sample under a definition's fixed floor is printed as a `breach` line after the
sample ids, without changing the exit code; see [a floor of free space](./signals.md#a-floor-of-free-space).

:::caution[Planned]

The service's scheduler, which samples each definition on its cadence, is planned; see the
[status page](/docs/status).

:::

## A sample on the wire

```json
{"sample_id": "smp_000000000042",
 "signal": "platform.namespace.request_failure_ratio",
 "series": "platform.namespace.request_failure_ratio{cluster=\"prod-a\",namespace=\"shared-1\"}",
 "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z",
 "window": {"start": "2026-10-14T16:57:03Z", "end": "2026-10-14T22:57:03Z"},
 "reading": {"quality": "ok", "value": {"value": "0.029"}},
 "unit": "1",
 "denominator": {"n": 556, "unit": "{request}", "since": "2025-03-02T00:00:00Z", "age_s": 51403023},
 "provenance": {"producer": "thalamus-service", "version": "0.1.0", "definition_rev": 1,
                "adapter": "sql", "connection": "platform-replica",
                "query_sha256": "…", "run_id": "run_01JA…"}}
```

## The reading

`reading` is a tagged union on `quality`
([`Reading`](../reference/ess/thalamus-observation.md#reading)):

| quality | on the wire | meaning |
|---|---|---|
| `ok` | `{"quality": "ok", "value": {"value": "0.029"}}` | read completely |
| `partial` | `{"quality": "partial", "value": {"value": "500", "bound": "lower"}}` | read but cut short (a full page, a paged read that could not finish); the number is a bound and `bound` says `lower` or `upper` |
| `unknown` | `{"quality": "unknown", "value": {"reason": "…"}}` | not read; there is no number, only the reason |

An unknown reading carries no number at all. Each quality is its own variant in the generated Rust
types, so the illegal pair (`unknown` with a number) cannot be built, and `NULL`, `NaN` or an
unparseable cell can never become `0`.

Numbers are decimals, and on the wire a decimal is a string: `{"value": "0.029"}`. That is the form
the generated serde types use; it is exact, and no consumer needs a JSON precision flag. A ratio is
`0.029` with unit `1`, never `"2.9%"`.

## The other fields

| field | rule |
|---|---|
| `series` | the signal id with its dimension values: `<signal-id>{k1="v1",k2="v2"}`, keys sorted ascending, values escaped as OpenMetrics label values. A sample with no dimensions has the bare signal id as its series key. |
| `dims` | every key comes from the instance's [dimension registry](./signals.md#dimensions) and every value resolves in it |
| `unit` | the definition's UCUM unit |
| `denominator` | the population the value was taken over: `n`, its unit, `since` (the earliest moment the population could contain anything) and `age_s`. A baseline whose offset exceeds `age_s` is `UNKNOWN`; see [Baselines](./baselines.md). |
| `provenance` | enough to re-run the read: producer and version, definition revision, adapter and connection, the SHA-256 of the exact query, and the run id grouping one sampling pass |

## Who samples

The service's scheduler owns the cadence: it samples each `Active` definition every `cadence`. The
command line samples on demand. A missed slot is recorded as a sample with quality `unknown` and
reason `missed_slot`; it is never re-read and never interpolated, and views draw it as a gap.

## Storage

| record | where | retention |
|---|---|---|
| samples and evaluations | an append-only stream per instance; the first store is [Eventlog](https://beyond10x.github.io/ecosystem/eventlog/) ([GitHub](https://github.com/beyond10x/eventlog)) (File, SQLite) | a configured window, by default 35 days: enough for a 7-day offset and a 4-week same-slot median |
| daily rollups | one row per series per UTC day: count, unknown and partial counts, min, median, max and last value, worst verdict, verdict counts, min and max score | kept |
| transitions | emitted as events and, for an instance connected to cortex, as facts | see [Assertions, events and views](./interfaces.md) |

Raw samples never go into a knowledge store: they have no lifecycle, and a pass over a fleet writes
hundreds of them. Only the change someone has to act on is forwarded.
