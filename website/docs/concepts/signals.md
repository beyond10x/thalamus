---
title: Signals
sidebar_position: 1
description: A signal definition says what is measured, from which source, in which unit and over which dimensions, and holds the only copy of its thresholds.
lede: A signal definition is the one place that says what is measured and how it is graded; every sample and every evaluation names the definition revision it came from.
source: spec/domains/definition.yaml, spec/domains/dimension.yaml, docs/design/thalamus.md § 2.1, § 2.4, § 5.2, § 11 (D8, D9, D10, D17, D18, D19, D25, D29), crates/thalamus-core/src/registry.rs
---

# Signals

A **signal** is one thing thalamus measures: the share of requests a platform failed, the number of
pods present, the pass ratio of a test run. Its **definition** says what is measured and how, and
it is the only copy of its thresholds.

An instance's definition files and its dimension registry are loaded and validated by
`thalamus_core::registry`. A file that breaks a rule is refused at load, and the error names the
file and the rule.

## A definition

One file per definition. The shape below follows the specification; the values are an example.

```yaml
id: platform.namespace.request_failure_ratio   # <family>.<layer>.<name>, lowercase, stable
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
persistence: {k: 2, k_crit: 1, m: 3}   # optional; these are the defaults
owner: team:platform
links: {components: [api], issues: []}
lifecycle: active               # proposed | active | muted | retired
rev: 1
revised_at: 2026-10-01T00:00:00Z
revised_by: someone
```

| field | rule |
|---|---|
| `id` | three non-empty segments of lower-case letters, digits and `_`, joined by dots; the first is the `family`. |
| `kind` | `gauge`, `counter_increase`, `ratio`, `instant` or `count_of_cases`. An `instant` count gets no trend below its `min_trend_window`, and is graded against `fixed` only. |
| `unit` | a UCUM unit. A ratio is a number with unit `1`, never a formatted percentage. |
| `dimensions` | keys from the instance's dimension registry. An empty list declares a signal of the whole instance; an absent list is undeclared. |
| `source` | an adapter and a Connectors connection id: an identifier, never a URL or a path (no `:/`, no leading `/`, no `http:`, `https:`, `file:`, `ftp:`, `ws:` or `wss:` prefix). thalamus holds no secret. |
| `window`, `cadence` | `<n>s`, `<n>m`, `<n>h`, `<n>d`, `<n>w`, or an ISO 8601 duration of weeks, days, hours, minutes and seconds, each unit at most once. More than zero and at most 719528 days, with no years or months, whose length depends on the calendar. It is stored in the canonical RFC 3339 form of the same length: `6h` as `PT6H`, `P1W2D` as `P9D`, `PT1H1S` as `PT1H0M1S`. `min_trend_window` follows the same rule. |
| `history` | `source` when the source can be re-queried at an offset, `store` when only thalamus remembers past values. It decides which [baselines](./baselines.md) are possible. |
| `denominator` | the population a ratio or count is taken over, and `min_n`, below which the sample is graded `N/A`. `since` is `first_sample`, a fixed RFC 3339 timestamp, or `<key>.created`: the `created` of the sample's value in the `file` registry of a declared dimension (`dimension.created` when exactly one declared dimension has a `file` registry). The origin is never earlier than the source's retention start. |
| `evaluation` | the rule, the baselines and the WARN and CRIT lines. This is the only copy of every threshold. An `Active` or `Muted` definition whose baselines name no single baseline thalamus computes is refused at load. |
| `evaluation.fixed` | the number a `fixed` baseline is, as a decimal (`fixed: "0"`). Required when `baselines` names `fixed`, and refused when it does not. |
| `evaluation.rolling_n` | the N of a `rolling_median` baseline, the median of the last N samples of the series: an integer of at least 1. Required when `baselines` names `rolling_median`, and refused when it does not. The `rolling_median` baseline itself is planned for M2. |
| `persistence` | `{k, k_crit, m}`, each an integer of at least 1: a verdict change settles into a transition after `k` agreeing evaluations (`k_crit` for a change to `CRIT`), and a `WARN` clears after `m` `OK` evaluations. Optional; absent, the service applies `k` 2, `k_crit` 1, `m` 3. The loader reads and checks it now; the hysteresis that applies it is planned for M2. |
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
registered key ([`Dimension`](../reference/ess/thalamus-dimension.md#dimension)), named once, says where its
values come from, how it may be set, and optionally its parent key.

| field | values | meaning |
|---|---|---|
| `values` | `file` | values listed in a registry file in the instance: a relative path naming a file inside the instance directory, with no empty, `.` or `..` segment, scheme or drive. An entry may carry `created` |
| | `open` | any value; meant for test-run dimensions (`project`, `suite`, `case`) |
| | `connectors` | values read from a Connectors inventory operation. Specified, but refused when an instance loads until Connectors offers such an operation |
| `rule` | `source_only` | set only when the source row carries it, never derived from a parent: a number measured for a shared environment is never attributed to one tenant in it |
| | `derived` | the alternative: the key's value may be derived rather than read from the row |
| `parent` | a key | the key this one sits under (`namespace` under `cluster`). It is registered, and no key reaches itself through its parents |
| `otel_attribute` | a name | the OpenTelemetry resource attribute the key maps to (`cluster` to `k8s.cluster.name`) |

## Sources

Every source but one is read through a [Connectors](https://beyond10x.github.io/connectors/)
([GitHub](https://github.com/beyond10x/connectors)) connection. thalamus invokes read operations
only, never writes to a system it watches, and never sees a credential. The exception is the host
collector, which reads the machine thalamus runs on directly (below). Which adapters have shipped
is on the [status page](/docs/status).

| source | reads |
|---|---|
| Prometheus | PromQL; `NaN` is unknown |
| Loki | LogQL metric queries |
| SQL | queries; `NULL` and unparseable cells are unknown, never 0 |
| Kubernetes | object counts, as instant samples with source-only dimensions |
| Git hosting | repository records as counts and ratios |
| HTTP probe | the availability of one Connectors read operation |
| Host | this machine's CPU use, memory use, disk space (used share and free bytes) and CPU temperature, no Connectors connection |

### The host collector

Source `{adapter: host, connection: local}` reads the machine thalamus runs on, and only when a
definition is sampled: it is no service, timer or daemon, and nothing it starts outlives the read.
It reads these five extractors and nothing else (design D46):

| extractor | unit | reads | dimensions |
|---|---|---|---|
| `host.cpu_busy` | `1` | the `cpu` line of `/proc/stat`, twice, 250 ms apart: 1 − (Δidle + Δiowait) / Δtotal | none |
| `host.memory_used` | `1` | `/proc/meminfo`: (MemTotal − MemAvailable) / MemTotal | none |
| `host.disk_used` | `1` | `statvfs` of each mount point the `query` lists (every value one mount), used as `df` computes it | `mount` |
| `host.disk_free` | `By` | `statvfs` of each listed mount point: f_bavail × f_frsize, the bytes an unprivileged writer may still take (`df`'s Avail) | `mount` |
| `host.cpu_temperature` | `Cel` | the hottest CPU sensor under `/sys/class/hwmon` (`coretemp`, `k10temp`, `zenpower`, `cpu_thermal`), else a `/sys/class/thermal` zone of type `x86_pkg_temp` or `cpu-thermal` | none |

```yaml
source: { adapter: host, connection: local }
extractor: host.disk_used
query: { root: /, data: /srv }
```

A missing file or field, a zero total, a CPU counter that went backwards, a mount that cannot be
read, a listed path that is no mount point (a plain directory or a file) and a machine with no CPU
sensor read unknown, naming what was wrong, never 0. A mount is spelled once: `//` is `/` and
`/srv/` is `/srv`, and `mount` carries that spelling. A host read measures the machine now, so a
window that does not end now (within 300 seconds of the machine's clock, as with
`thalamus sample --at` in the past) reads unknown ("host reads only this moment"). A host
definition naming any connection other than `local` is refused when it is proposed or revised,
and reads unknown if one is loaded anyway; a definition of any other adapter naming the reserved
connection `local` is refused too.

#### A floor of free space

A floor is a threshold, so it lives in the instance's definition, never in thalamus. It is said
with the lines every definition has: grade lower-is-worse against the fixed number, and put the
CRIT line at the floor itself. A reading at or under a line is on it, so a reading exactly at the
floor is a CRIT breach (synthetic numbers):

```yaml
kind: gauge
unit: By
source: { adapter: host, connection: local }
extractor: host.disk_free
query: { root: / }
evaluation:
  rule: lower_is_worse
  baselines: [fixed]
  fixed: "200000000000"   # the floor, in bytes
  warn: { x: "1.25" }     # WARN at or under 1.25 times the floor
  crit: { x: "1" }        # CRIT at or under the floor
```

`thalamus sample` grades every sample it recorded whose definition is graded against `fixed`
alone, since that needs no stored history, and prints each WARN or CRIT after the sample ids as one
line, `breach <VERDICT> <signal> <dims> <value> <unit> <line>`:

```text
breach CRIT host.disk.free mount=/ 150000000000 By 150000000000 <= 1 x 200000000000
```

`--format json` carries the same under `breaches`, one object per breach with `verdict`,
`sample_id`, `signal`, `series`, `dims`, `value`, `unit` and `line`. No evaluation is recorded, an
unknown reading is never a breach and never OK, and the exit code is unchanged: a breach is not a
failure of the run.

Producers that already speak another verdict vocabulary are mapped onto the ladder at the adapter;
see [other vocabularies](./verdicts.md#other-vocabularies).
