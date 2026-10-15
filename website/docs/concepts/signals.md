---
title: Signals
sidebar_position: 1
description: A signal definition says what is measured, from which source, in which unit and over which dimensions, and holds the only copy of its thresholds.
lede: A signal definition is the one place that says what is measured and how it is graded; every sample and every evaluation names the definition revision it came from.
source: spec/domains/definition.yaml, spec/domains/dimension.yaml, docs/design/thalamus.md § 2.1, § 2.4, § 5.2, § 11 (D8, D9, D17, D19)
---

# Signals

A **signal** is one thing thalamus measures: the share of requests a platform failed, the number of
pods present, the pass ratio of a test run. Its **definition** says what is measured and how, and
it is the only copy of its thresholds.

:::caution[Planned]

The definition is specified (`thalamus.definition.SignalDefinition`) and its Rust type is
generated. Loading and validating an instance's definitions is planned; see the
[status page](/docs/status).

:::

## A definition

The shape below follows the specification; the values are an example.

```yaml
signal_id: platform.namespace.request_failure_ratio   # <family>.<layer>.<name>, lowercase, stable
title: Share of requests the platform failed
family: platform
kind: ratio                     # gauge | counter_increase | ratio | instant | count_of_cases
unit: "1"                       # UCUM: 1, s, By, {request}, {pod}
dimensions: [cluster, namespace]
source: {adapter: sql, connection: platform-replica}
extractor: sql.ratio
query: {numerator: "…", denominator: "…"}
window: 6h
cadence: 1h                     # how often the service samples it
history: source                 # source: re-queryable at an offset · store: only thalamus remembers it
denominator: {unit: "{request}", min_n: 20, since: first_sample}
evaluation:
  rule: ratio_vs_pooled_baseline
  baselines: [offset_1d, offset_7d]
  warn: {x: 2, abs: 0.02, min_events: 10}
  crit: {x: 4, abs: 0.05}
owner: team:platform
links: {components: [api], issues: []}
rev: 1
```

| field | rule |
|---|---|
| `kind` | `gauge`, `counter_increase`, `ratio`, `instant` or `count_of_cases`. An `instant` count gets no trend below its `min_trend_window`. |
| `unit` | a UCUM unit. A ratio is a number with unit `1`, never a formatted percentage. |
| `dimensions` | keys from the instance's dimension registry. An empty list declares a signal of the whole instance; an absent list is undeclared. |
| `source` | an adapter and a Connectors connection id. thalamus holds no secret. |
| `history` | `source` when the source can be re-queried at an offset, `store` when only thalamus remembers past values. It decides which [baselines](./baselines.md) are possible. |
| `denominator` | the population a ratio or count is taken over, and `min_n`, below which the sample is graded `N/A`. |
| `evaluation` | the rule, the baselines and the WARN and CRIT lines. This is the only copy of every threshold. |
| `rev` | a definition change is a new revision with a date and an author, so an evaluation names the revision it was graded against. |

The full type, with every field, is
[`SignalDefinition`](../reference/ess/thalamus-definition.md#signaldefinition) in the reference.

## Lifecycle

A definition is created `Proposed` and becomes `Active` through `ActivateDefinition`. An `Active`
definition can be `Muted` (`MuteDefinition`) and return to `Active` (`UnmuteDefinition`). Any of
the three can be `Retired`, which is terminal. The
[reference page](../reference/ess/thalamus-definition.md#signaldefinition) draws the lifecycle
from `spec/`.

- An `Active` definition declares `unit`, `dimensions`, `denominator` and `owner`; activation
  refuses one that does not.
- A `Muted` definition carries a reason and an expiry; no other state carries either.
- Only `Active` definitions are sampled.
- The definition file in the instance is the only copy of a definition, its lifecycle included.
  The service never rewrites it: once a mute has expired, the service samples the definition as
  `Active` again and records the unmute as an event, so the change is visible.

## Dimensions

A sample's dimensions (`cluster`, `namespace`, `component`, a test `suite`) come from a closed
registry declared **per instance**. A sample with an unregistered key or value is refused. Each
registered key ([`Dimension`](../reference/ess/thalamus-dimension.md#dimension)) says where its
values come from, how it may be set, and optionally its parent key.

| field | values | meaning |
|---|---|---|
| `values` | `file` | values listed in a registry file in the instance |
| | `open` | any value; meant for test-run dimensions (`project`, `suite`, `case`) |
| | `connectors` | values read from a Connectors inventory operation. Specified, but refused when an instance loads until Connectors offers such an operation |
| `rule` | `source_only` | set only when the source row carries it, never derived from a parent: a number measured for a shared environment is never attributed to one tenant in it |
| | `derived` | the alternative: the key's value may be derived rather than read from the row |
| `parent` | a key | the key this one sits under (`namespace` under `cluster`) |
| `otel_attribute` | a name | the OpenTelemetry resource attribute the key maps to (`cluster` to `k8s.cluster.name`) |

## Sources

Every source is read through a [Connectors](https://beyond10x.github.io/connectors/)
([GitHub](https://github.com/beyond10x/connectors)) connection. thalamus invokes read operations
only, never writes to a system it watches, and never sees a credential.

| source | reads | status |
|---|---|---|
| Prometheus | PromQL; `NaN` is unknown | planned |
| Loki | LogQL metric queries | planned |
| SQL | queries; `NULL` and unparseable cells are unknown, never 0 | planned |
| Kubernetes | object counts, as instant samples with source-only dimensions | planned |
| Git hosting | repository records as counts and ratios | planned |
| HTTP probe | the availability of one Connectors read operation | planned |

Producers that already speak another verdict vocabulary are mapped onto the ladder at the adapter;
see [other vocabularies](./verdicts.md#other-vocabularies).
