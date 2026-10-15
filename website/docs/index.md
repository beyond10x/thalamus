---
slug: /
title: Overview
sidebar_label: Overview
sidebar_position: 1
description: What thalamus is, what it is not, and how it sits beside cortex, Connectors, conductor, Eventlog and ESS.
---

# thalamus

![thalamus](/img/logo.svg)

thalamus is the realtime perception layer over the systems an organization runs. It reads numbers
from the instruments those systems already expose (metrics, logs, databases, orchestrators,
version control, test runs), records each reading as a typed **sample**, grades the sample into a
**verdict** against a **baseline**, and answers one question at any moment: *what is out of bounds
right now, compared with what is normal for it?*

:::caution[Specified, not yet implemented]

The specification of thalamus is on `main`: five ESS domains and the Rust model generated from
them. The libraries, the service and the command line that implement it are planned. Every page
says which parts exist; the [status page](/docs/status) lists each capability.

:::

```text
signal definition   what is measured, from which source, graded by which rule
   ↓  read through a Connectors connection, read-only
sample              one typed reading per series: ok, partial or unknown
   ↓  compared with a baseline
evaluation          one verdict of the ladder, its score and its reason
   ↓  persisted over k evaluations
transition          the change someone has to act on: an event, a fact in cortex
```

## Three records

| record | holds | page |
|---|---|---|
| signal definition | what is measured and how: the source, the query, the unit, the dimensions, the baseline and the thresholds | [Signals](./concepts/signals.md) |
| sample | one reading of one signal for one series at one instant | [Samples](./concepts/samples.md) |
| evaluation | one verdict over one sample, with the rule and the definition revision that decided it | [The verdict ladder](./concepts/verdicts.md) |

Keeping the evaluation apart from the sample lets a threshold change re-grade history without
reading the source again.

## thalamus and cortex

thalamus is one half of a pair. [cortex](https://beyond10x.github.io/cortex/)
([GitHub](https://github.com/beyond10x/cortex)) is the other.

| | thalamus | cortex |
|---|---|---|
| role | perception: what the systems say now | memory: what the organization knows |
| time horizon | seconds to weeks | months to years |
| record | sample, then evaluation; append-only, numeric | evidence-backed fact in a knowledge store |
| retention | raw samples for a bounded window, daily rollups kept | every revision kept |
| question | is it out of bounds, since when, against which baseline? | what do we know about it, and from which evidence? |

The name is the brain's: the thalamus relays sensory input to the cortex and filters it on the
way. Most samples never leave thalamus. What reaches cortex is a verdict transition (a signal that
went out of bounds or came back), a daily rollup, and the definition that explains both.

## What thalamus is not

- **Not a store of everything.** Raw samples stay for a bounded window (35 days by default);
  only transitions and daily rollups are forwarded.
- **Not a writer.** thalamus never writes to a system it watches.
- **Not a credential holder.** A source names a [Connectors](https://beyond10x.github.io/connectors/)
  ([GitHub](https://github.com/beyond10x/connectors)) connection; Connectors holds the credential.
- **Not a target setter.** An instance declares its thresholds and objectives; thalamus sets none.
- **Not a hosted product.** An instance is a directory with its definitions, its dimension
  registry and its store location. Instance data never enters the thalamus repository.
- **Not a dashboard tool.** Dashboard and alert-rule exports are planned for Prometheus and Loki
  definitions only; a dashboard tool cannot see SQL, orchestrator or version-control signals.

## Where it sits

| neighbour | relation |
|---|---|
| [cortex](https://beyond10x.github.io/cortex/) ([GitHub](https://github.com/beyond10x/cortex)) | remembers what thalamus perceives: transitions and daily rollups are planned to reach it as facts |
| [Connectors](https://beyond10x.github.io/connectors/) ([GitHub](https://github.com/beyond10x/connectors)) | every source is read through it; it holds every credential |
| [conductor](https://beyond10x.github.io/conductor/) ([GitHub](https://github.com/beyond10x/conductor)) | thalamus's first consumer: it will assert signals before closing a wave and read the live verdict stream |
| [ESS](https://beyond10x.github.io/ess/) ([GitHub](https://github.com/beyond10x/ess)) | specifies thalamus; the Rust model and the conformance suite are generated from `spec/` |
| [Eventlog](https://beyond10x.github.io/ecosystem/eventlog/) ([GitHub](https://github.com/beyond10x/eventlog)) | the first store of samples and evaluations (File, SQLite) |

## Where to go next

- [Getting started](./getting-started.md): build the workspace and run its gate.
- [Signals](./concepts/signals.md), [Samples](./concepts/samples.md),
  [The verdict ladder](./concepts/verdicts.md), [Baselines](./concepts/baselines.md) and
  [The score](./concepts/score.md): the model, one idea per page.
- [Assertions, events and views](./concepts/interfaces.md): what callers will get out of it.
- [ESS specification](/docs/reference/ess): every domain, as graphs and as reference pages.
- [Status](/docs/status): shipped and planned, item by item.
