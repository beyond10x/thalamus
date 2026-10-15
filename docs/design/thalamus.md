# Thalamus — design

Last updated: 2026-10-15 · Status: proposed · Planning: one epic per milestone of § 9, `epic:specification` (M0) to `epic:assertions-and-integrations` (M5)

## 0. What thalamus is

Thalamus is the realtime perception layer over the systems an organization runs. It reads
numbers from the instruments those systems already expose (metrics, logs, databases, orchestrators,
version control, test runs), turns each reading into a typed **sample**, grades it into a
**verdict**, and answers one question at any moment: *what is out of bounds right now, compared with
what is normal for it?*

It is one half of a pair:

| | thalamus | cortex |
|---|---|---|
| role | perception: what the systems say now | memory: what the organization knows |
| time horizon | seconds to weeks | months to years |
| record | sample → evaluation, append-only, numeric | evidence-backed fact in an EKR store |
| retention | raw samples for a bounded window, daily rollups kept | every revision kept |
| question | "is it out of bounds, since when, against which baseline?" | "what do we know about it, and from which evidence?" |

The brain analogy is the reason for the name: the thalamus relays sensory input to the cortex and
filters it on the way. Thalamus does the same. Most samples never leave it. What reaches cortex is
a **verdict transition** (a signal that went out of bounds or came back), a daily rollup, and the
definition that explains both.

Both serve the periphery of the software factory (today: conductor and its controllers). A
release, a merge request or a wave can **assert** that named signals stay in baseline, and
thalamus answers with an evaluation, not a guess.

## 1. Lessons this design is built on

An earlier internal prototype grew thirteen producers of health numbers, verdicts and counts,
each with its own conventions. Reading them side by side gave nine failure modes. Every rule in
§ 2–§ 4 exists to remove one of them.

| # | failure mode | rule that removes it |
|---|---|---|
| L1 | four verdict vocabularies (`OK/WARN/CRIT`, colours, detector exit codes, gate exit codes) and three exit-code contracts | one verdict ladder (§ 2.3) with one exit-code mapping |
| L2 | the same threshold typed in two producers, one copy dead code | a threshold exists only in its definition (§ 2.1) |
| L3 | "not read" ranked the same as "green", so an overall verdict depended on key order | `UNKNOWN` is a verdict of its own and never folds into `OK` (§ 2.3) |
| L4 | a sample's value was sometimes a formatted string (`"2.9%"`) or `null` while `OK` | `value` is a number or `null`; a ratio is `0.029` with unit `1` (§ 2.2) |
| L5 | `NULL` and unparseable values became `0` | `quality: unknown` carries no value, enforced by the type (§ 2.2) |
| L6 | "week over week" compared against whatever row was oldest, even if the series was three days old | a baseline older than its series is `UNKNOWN` (§ 3.1) |
| L7 | two history series were started and abandoned; the one that survived was written by a step that always ran | sampling is owned by the service's scheduler, not by whoever remembers (§ 5.1) |
| L8 | an instant count (pods present now) was trended; two reads four minutes apart differed by 25 % | signals of kind `instant` get no trend below a declared window (§ 3.1) |
| L9 | a number measured for a shared environment was attributed to one tenant in it | a dimension is set only when the source carries it, never derived (§ 2.4) |

## 2. Model

Three records. A **definition** says what is measured and how. A **sample** is one reading. An
**evaluation** is one verdict over one sample, with the rule that decided it. Keeping the
evaluation apart from the sample lets a threshold change re-grade history without reading the source
again.

These three nouns are specified in ESS before any code (§ 8). The shapes below are the starting
point for that specification, not a second copy of it.

### 2.1 `SignalDefinition`

```yaml
id: platform.namespace.request_failure_ratio       # <family>.<layer>.<name>, lowercase, stable
title: Share of requests the platform failed
family: platform                                   # an instance-defined grouping
kind: ratio                                        # gauge | counter_increase | ratio | instant | count_of_cases
unit: "1"                                          # UCUM: 1, s, By, {request}, {pod}
dimensions: [cluster, namespace]                   # keys from the instance's dimension registry, § 2.4
source: {adapter: sql, connection: platform-replica}  # a Connectors connection; thalamus holds no secret
extractor: sql.ratio                               # a built-in extractor, or a plugin id
query: { numerator: "…", denominator: "…" }        # the query text, stored once, hashed into every sample
window: 6h
cadence: 1h                                        # how often the service samples it
history: source                                    # source: re-queryable at an offset · store: only thalamus remembers it
denominator: { unit: "{request}", min_n: 20, since: dimension.created }
evaluation: { rule: ratio_vs_pooled_baseline, baselines: [offset_1d, offset_7d],
              warn: { x: 2, abs: 0.02, min_events: 10 }, crit: { x: 4, abs: 0.05 } }
objective: null                                    # optional OpenSLO-shaped block, § 3.4
owner: team:platform
links: { components: [api], issues: [] }           # what this signal is evidence about
lifecycle: active                                  # proposed | active | muted | retired
rev: 1
```

The lifecycle is an Entity Runtime definition. Its invariants: an `active` definition declares
`unit`, `dimensions`, `denominator` and `owner`; a `muted` one carries a reason and an expiry date.
A threshold lives in the definition and nowhere else (L2). A definition change is a revision with a
date and an author, so an evaluation names the `rev` it was graded against.

### 2.2 `SignalSample`

```json
{"schema": "thalamus.sample/1",
 "id": "smp_01JA…",
 "signal": "platform.namespace.request_failure_ratio",
 "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z",
 "window": {"start": "2026-10-14T16:57:03Z", "end": "2026-10-14T22:57:03Z"},
 "reading": {"quality": "ok", "value": {"value": "0.029"}}, "unit": "1",
 "reason": null,
 "denominator": {"n": 556, "unit": "{request}", "since": "2025-03-02T00:00:00Z", "age_s": 51403023},
 "provenance": {"producer": "thalamus-service", "version": "0.1.0", "definition_rev": 1,
                "adapter": "sql", "connection": "platform-replica",
                "query_sha256": "…", "run_id": "run_01JA…"}}
```

| field | rule |
|---|---|
| `reading` | a tagged union on `quality` (spec `thalamus.observation.Reading`): `ok` carries a number, `partial` a number and its `bound`, `unknown` no value at all. Never a formatted string (L4) |
| `quality` | `ok`: read completely. `partial`: read but cut (a full page, a paged read that could not finish); `value` is then a bound and `bound` says `lower` or `upper`. `unknown`: not read; there is no `value` and `reason` says why |
| unknown is not 0 | `quality: unknown` carries no value. In Rust this is an enum, so the illegal pair cannot be built (L5) |
| `denominator.since`, `age_s` | the earliest moment the population could contain anything: the dimension's creation time, the source's retention start, the first sample of the series. A baseline whose shifted window starts before the series began (offset plus window span greater than `age_s`) is `UNKNOWN` (§ 3.1) |
| `provenance` | enough to re-run the read: producer and version, definition revision, adapter and connection, the SHA-256 of the query map's canonical JSON, and the run id grouping one sampling pass |
| ids | `dims` keys come from the dimension registry and every value must resolve in it (§ 2.4) |

### 2.3 `SignalEvaluation`

```json
{"schema": "thalamus.evaluation/1", "sample": "smp_01JA…",
 "signal": "platform.namespace.request_failure_ratio", "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z", "definition_rev": 1,
 "verdict": "WARN", "prev": "OK", "score": 1.3,
 "baseline": {"kind": "pooled", "offsets": ["1d", "7d"], "value": 0.013, "quality": "ok"},
 "reason": "0.029 >= 2 x 0.013 and >= 0.02, 15 failures of 556"}
```

One verdict ladder (L1):

| verdict | meaning | exit |
|---|---|---|
| `OK` | read, graded, inside bounds | 0 |
| `N/A` | read, too little to grade (`n < min_n`). Not `OK`: a pass in which every signal is `N/A` exits 3 | 0 |
| `WARN` | out of bounds | 1 |
| `CRIT` | far out of bounds | 2 |
| `UNKNOWN` | the sample is unknown, or its baseline is | 3 |

Worst-of ordering for an aggregate: `CRIT > WARN > UNKNOWN > OK = N/A`. `reason` is required on
every verdict other than `OK`. `prev` makes a transition visible without a second read. `score` is
the normalised position used by every view (§ 3.3).

Producers that already speak another vocabulary are mapped at the adapter: colour ladders
`green/yellow/red/na` → `OK/WARN/CRIT/UNKNOWN`; detector exit `1` → `CRIT`, `2` → `UNKNOWN`; a gate's
non-zero exit → `CRIT` for a blocking gate, `WARN` above baseline for an advisory one.

### 2.4 Dimensions

The dimension set is closed **per instance**: an instance declares its dimension registry, and a
sample with an unregistered key or value is refused.

```yaml
dimensions:
  cluster:   { registry: connectors:inventory/clusters }
  namespace: { registry: connectors:inventory/namespaces, parent: cluster }
  tenant:    { registry: connectors:inventory/tenants, rule: source_only }
  component: { registry: file:registries/components.json }
  suite:     { registry: open }
```

`rule: source_only` means the key is set only when the source row carries it, never derived from
a parent (L9). `registry: open` admits any value and is meant for test-run dimensions
(`project`, `suite`, `case`). A dimension may declare an OpenTelemetry attribute it maps to
(`cluster` → `k8s.cluster.name`, `namespace` → `k8s.namespace.name`, `component` → `service.name`).

## 3. Evaluation

### 3.1 Baselines

| kind | what | when |
|---|---|---|
| `offset_1d`, `offset_7d` | the same window 1 day or 7 days earlier, weekday-matched for 7 days | `history: source` |
| `pooled` | sum of the baselines' numerators over the sum of their denominators | ratios |
| `higher_of`, `lower_of` | tolerant of daily and weekly shape: errors against the higher, volume drops against the lower | counts |
| `same_slot_median_4w` | median of the same weekday and hour over the last 4 weeks | 4 weeks of samples or source history |
| `rolling_median` | median of the last N samples of this series | `history: store` |
| `fixed` | a number in the definition | a hard zero, or no history yet |

Rules:

1. A baseline that could not be read is `UNKNOWN`, never 0.
2. A baseline older than its series is `UNKNOWN` with reason `baseline predates series` (L6).
3. A signal of kind `instant` gets no delta and no trend below its `min_trend_window`; it is graded
   against `fixed` only (L8).
4. A `partial` sample may escalate but never clear. A lower bound already over the CRIT line is
   `CRIT`; otherwise the verdict is `UNKNOWN` with reason `partial read`.

### 3.2 Persistence and hysteresis

A verdict change counts as a **transition** only after `k` consecutive evaluations agree (default
`k = 2`, `k = 1` for `CRIT`). A `WARN` clears after `m = 3` `OK` evaluations. This is Prometheus'
`for:` applied to every source. Without it a flapping signal opens and closes a downstream flag on
every pass. The defaults are guesses; milestone M2 measures flap rates and fixes them per family.

### 3.3 The normalised score

Every evaluation records `score`: 0 at the baseline, 1 at the WARN line, 2 at the CRIT line, linear
in between (log-linear for multiplier rules), negative when the value is better than the baseline.
`UNKNOWN` and `N/A` have no score; views draw them hatched or grey, never as 0. The score is what
lets signals with different units share one heatmap and one sort order. It derives from the rule that
decided the verdict, so the colour and the verdict cannot disagree.

### 3.4 Objectives

Optional, for signals whose good/total split is meaningful:

```yaml
objective: { target: 0.995, window: 28d rolling, indicator: { good: "requests not failed", total: "requests" },
             budgeting: occurrences,
             burn_alerts: [ { long: 1h, short: 5m, rate: 14.4, verdict: CRIT },
                            { long: 6h, short: 30m, rate: 6, verdict: WARN } ] }
```

This is the OpenSLO `SLO` shape (target, time window, ratio metric, budgeting method) plus
multi-window burn-rate pairs. Thalamus sets no targets; an instance does.

## 4. Storage

| record | where | why |
|---|---|---|
| definitions | the instance's definition files, versioned in the instance's own repository | one copy of every threshold, reviewed like code |
| samples and evaluations | append-only stream per instance, retained for a configured window (default 35 days: enough for `offset_7d` and a 4-week same-slot median) | volume; the source can replay most of it |
| daily rollups | one row per series per UTC day: `n`, `unknown`, `partial`, `min/p50/max/last`, worst verdict, verdict counts, min/max score | what a `history: store` source cannot replay |
| transitions | emitted as events (§ 6) and, for an instance connected to cortex, as facts with the evaluation as evidence | what someone has to act on |

Raw samples never go into a fold log or a knowledge store. They have no lifecycle, and a fleet pass
writes hundreds of them. Only the **change** someone has to act on is forwarded.

Size, from the prototype (one environment, 23 signals): 2,835 B of core fields, about 123 B per
signal. With provenance by reference, about 300 B per sample (inferred). A fleet of 26 scopes
sampled hourly writes about 2.4 MB a day (inferred). M1 replaces every inferred number with a
measured one.

## 5. Architecture

### 5.1 Crates

| crate | kind | holds |
|---|---|---|
| `thalamus-model` | library, generated | the types and behaviours ESS synthesises from `spec/` into `generated/thalamus-model`; never edited by hand. ESS names the package `thalamus`; the workspace depends on it renamed `thalamus-model` |
| `thalamus-spec-types` | library, generated | the serde types of every model type and event (`generated/spec-types`); never edited by hand |
| `thalamus-core` | library | definition registry and validation, baselines, the verdict ladder, hysteresis, score, rollups. No I/O, no clock: time and samples are passed in |
| `thalamus-store` | library | the append-only sample stream and rollups behind one trait. First provider: Eventlog (File, SQLite, PostgreSQL) |
| `thalamus-sources` | library | adapters that turn a Connectors operation into samples: Prometheus/PromQL, Loki/LogQL, SQL, Kubernetes, Git, HTTP probe, test-run events |
| `thalamus-pipeline` | library | the read → sample → grade → evaluate pass the CLI and the service share |
| `thalamus-service` | binary | the scheduler that samples each definition on its cadence, the HTTP API (current verdicts, series, assertions), a server-sent-events stream of evaluations, and the CloudEvents emitter |
| `thalamus-cli` | binary `thalamus` (clap derive) | `define validate`, `sample`, `eval`, `show`, `compare --vs 7d`, `assert`, `rollup`, `view` |

`thalamus-core` stays pure so the same code grades a live sample, re-grades history after a
threshold change, and runs in a test without a source.

### 5.2 Boundaries

- **No secret enters thalamus.** A source names a Connectors connection; thalamus only invokes
  operations through Connectors, the pattern cortex uses.
- **Read-only against the systems it watches.** Thalamus never writes to a source.
- **Instances, not a hosted product.** An instance is a directory: its definitions, its dimension
  registry, its store location. Instance data never enters this repository.

### 5.3 Who samples

The service's scheduler owns the cadence (L7). The CLI can sample one definition on demand, and a
CI job or a controller can run a one-shot pass with `thalamus sample --all --once`. A gap in
sampling is recorded as a gap and drawn as one, never interpolated.

## 6. Interfaces

### 6.1 Assertions

```console
$ thalamus assert platform.namespace.request_failure_ratio --dims namespace=shared-1 \
      --after 2026-10-14T20:08:00Z --for 2h --within baseline
```

Exit codes are the ladder's: 0 within, 1 or 2 breach, 3 unknown. **Unknown is not a pass.** The
result is an evaluation with an `assertion` block naming who asked: a release, a merge request, a
test run, a controller.

| asker | how |
|---|---|
| a release | after the rollout, assert the release's declared set for every environment it reached |
| a merge request | an `Asserts:` line in its description names signals and a horizon; the factory re-runs it when the horizon passes |
| conductor / a controller | `thalamus assert --declared` before closing a wave, and the live verdict stream as an input to scheduling |

### 6.2 Events

Every evaluation and every transition is available as a CloudEvents event on the wire (`type:
dev.beyond10x.thalamus.evaluation`, `…transition`; `source` = instance and producer; `subject` =
series key; `time` = `ts`). Stored lines are not wrapped; the envelope costs about 120 B a line for
no reader inside the store (inferred).

### 6.3 Test runs, both directions

| direction | mapping |
|---|---|
| test run → thalamus | a finished case becomes a sample of `test.case.passed{project,suite,case,target}` (1 or 0); a run becomes `test.run.pass_ratio`. An instrument failure maps to `quality: unknown`, a defect to a failing value. A red suite on release day shows in the same heatmap as the platform signals |
| thalamus → test run | a release-day run carries signal assertions; each result is emitted as a case in the run's stream, so the test UI shows "platform signals stayed in baseline" beside the scenarios |

### 6.4 Cortex

For an instance connected to cortex, thalamus is a cortex source: definitions, daily rollups and
transitions become facts, each citing the evaluation that produced it. Cortex never receives raw
samples.

### 6.5 Views

| view | shape | data |
|---|---|---|
| heatmap | rows = series grouped by layer, columns = hours or days, cell = `score` on a diverging scale; unknown hatched, N/A grey | evaluations |
| sparklines | value line, baseline band, WARN and CRIT lines; gaps drawn as gaps | samples, evaluations |
| before/after a deploy | every signal's window before vs after one deploy instant, sorted by score delta | samples, deploy instants |
| week vs week | this week against the same slot last week | evaluations, `offset_7d` |
| live | current verdict per series, updated from the event stream | evaluations |

A **deploy instant** is the time a workload actually changed (pod creation, merge time), never an
announcement.

Views are served by `thalamus-service`. Dashboard JSON and Prometheus recording and alert rules are
generated from definitions as an **export**, only for definitions whose source is Prometheus or
Loki; a dashboard tool cannot see SQL, orchestrator or version-control signals, so it is never the
only view.

## 7. Standards

| standard | adopt | do not adopt | why |
|---|---|---|---|
| OpenMetrics / Prometheus data model | naming for export (`snake_case`, unit suffix, `_total`), labels = dimensions, gauge/counter split | Prometheus as the store, Pushgateway | most signals come from SQL, orchestrators and Git, not exporters |
| Prometheus recording and alert rules | an export for Prometheus/Loki definitions; `for:` semantics (§ 3.2) | rules written by hand beside the definitions | one copy of every threshold (L2) |
| OpenSLO | the `SLO` object shape for `objective` | `Service`, `AlertPolicy`, the OpenSLO toolchain | nothing here consumes them; keep the block exportable |
| OpenTelemetry | UCUM units; declared mapping of dimensions to resource attributes | OTLP as transport or store | verdicts, quality and denominators have no OTel field |
| CloudEvents | the envelope on the wire | wrapping stored lines | no reader inside the store needs it |
| AEP evidence | an evaluation can be recorded as a `metric_observation` for a protocol guard | — | lets an AEP move cite a signal |

## 8. ESS

`SignalDefinition`, `SignalSample`, `SignalEvaluation`, `DimensionRegistry` and `Assertion` are
specified in an ESS domain under `spec/` **before** the first story is implemented, and
`thalamus-model` is generated from it. Each story's acceptance is a set of named conformance
scenarios. The prototype deferred ESS until a second consumer existed; here the product has three
from the start (conductor, cortex, test runs), and the workspace rule is that a new noun gets its
ESS domain first.

Two constructs to probe first, because the model depends on them:

1. a nullable number whose nullness is tied to an enum (`quality: unknown` ⇔ `value: null`);
2. a UCUM-typed unit.

If ESS cannot express either, the gap is reported to the ESS repository and the rule is held by a
Rust test with an `UNMAPPED:` marker in the spec, the pattern cortex uses.

## 9. Milestones

| step | ships | acceptance |
|---|---|---|
| **M0. Specification** | ESS domain for the five nouns; generated `thalamus-model`; `task check` running spec validity and drift | the two probes in § 8 answered; conformance suite synthesised |
| **M1. Contract and first source** | `thalamus-core` (ladder, baselines `offset_*`/`pooled`/`higher_of`/`lower_of`/`fixed`, score); `thalamus-store` on Eventlog; the Prometheus adapter; `thalamus sample`, `eval`, `show` | failing-first tests: unknown ⇒ `value: null` and never `OK`; a ratio is numeric with unit `1`; every sample names an `active` definition; a baseline older than its series is `UNKNOWN`. One live read-only pass against a real instance, its byte size recorded in § 4 |
| **M2. History and transitions** | `compare`, `rollup`, `same_slot_median_4w`, `rolling_median`; hysteresis; transitions as CloudEvents; the scheduler in `thalamus-service` | `compare --vs 7d` answers from stored samples; flap rate measured before `k` and `m` are fixed; one transition emitted and cleared by real data |
| **M3. Every source kind** | Loki, SQL, Kubernetes, Git, HTTP probe adapters; vocabulary mapping for existing producers | one verdict vocabulary across all sources; a colour ladder's `na` cannot rank as `green` (failing-first test) |
| **M4. Views and exports** | heatmap, sparklines, week vs week, before/after deploy, live; dashboard and rule export | a 4-week heatmap from stored samples; one known deploy visible in before/after |
| **M5. Assertions and integrations** | `assert`; `Asserts:` lines; conductor integration; test-run bridge both ways; cortex source | one release asserts its declared set; one test run carries signal assertions and its cases appear as samples; one transition appears in cortex with its evaluation as evidence |

M0 comes first because every later step reads the types it generates. M1 touches one adapter and
turns L4, L5 and L6 into tests.

## 10. Open questions

| # | question | default if unanswered |
|---|---|---|
| 1 | Is Eventlog the first store provider, or a plain append-only file per day? | Eventlog, SQLite provider |
| 2 | Does the service run per operator machine, or in-cluster next to the systems it watches? | per machine for M1–M2; an in-cluster deployment is a later milestone |
| 3 | Where do transitions go besides the event stream: cortex, conductor, a chat post? | event stream and cortex only |
| 4 | Is the dimension registry read through Connectors, from files in the instance, or both? | both, as § 2.4 shows |
| 5 | Is the live view part of `thalamus-service`, or a uilab surface that reads its API? | in the service for M4; moved if uilab adopts it |
| 6 | Which signals get an objective, and with which target? | none set by thalamus |
| 7 | Is the prototype the first instance, migrated source by source in M3? | yes, kept outside this repository |

## 11. Specification decisions (2026-10-15)

The first draft of `spec/` left 17 `UNMAPPED:` markers. Seven are constructs ESS cannot state yet;
the rule is held by a Rust test and the marker stays. Ten are decisions, taken here and cited from
the marker.

| # | marker | decision |
|---|---|---|
| D1 | `spec/domains/assertion.yaml:66` | every signal an assertion names must exist and be `Active`; otherwise the ask is refused. Held by a Rust test |
| D2 | `spec/domains/assertion.yaml:92` | an answer is part of the `Assertion` record, not an evaluation with an assertion block. It cites the evaluations it read: `references`, cardinality many |
| D3 | `spec/domains/assertion.yaml:223` | the service answers only after `after + horizon`; a window in which every evaluation is `N/A` answers `unknown`. Held by `thalamus-core` tests |
| D4 | `spec/domains/evaluation.yaml:46` | a recorded baseline is a closed kind from § 3.1 plus its offsets. The example in § 2.3 is `pooled` over `[1d, 7d]` |
| D5 | `spec/domains/evaluation.yaml:223` | at most one evaluation per `(sample, definition_rev)`; a re-grade after a revision is a new evaluation. Held by a store constraint and a Rust test |
| D6 | `spec/domains/evaluation.yaml:323` | `prev` is the previous raw verdict of the series. A settled transition (§ 3.2) is its own record, emitted by the hysteresis in M2 |
| D7 | `spec/domains/definition.yaml:204` | `objective` stays untyped JSON until a milestone after M3 types it |
| D8 | `spec/domains/definition.yaml:224` | an empty `dimensions` list declares zero dimensions (a signal of the whole instance); an absent one is undeclared, and `ActivateDefinition` refuses it |
| D9 | `spec/domains/definition.yaml:237` | the service unmutes a definition whose expiry has passed, issuing `UnmuteDefinition` as actor `service` |
| D10 | `spec/domains/observation.yaml:17` | a series key is `<signal-id>{k1="v1",k2="v2"}`, keys sorted ascending, values escaped as OpenMetrics label values; a sample with no dimensions has the bare id as its key |

Two more decisions came out of the draft itself, and § 2.2 follows them:

- **The reading is a tagged union.** `quality` is the tag; `ok` carries a value, `partial` a value
  and its bound, `unknown` nothing. On the wire: `{"quality": "ok", "value": {"value": "0.029"}}` (D11).
  The flat `value: null` form of the first draft is gone, because the union makes the illegal pair
  unbuildable in Rust.
- **Numbers are `Decimal`.** `Binary64` is refused by ESS conformance and by its Rust target.

ESS gaps found by the draft, to report to the ESS repository: a pattern or grammar type for a
string (the id grammar, UCUM); a guard over every element of an input list or map
(D1, dimension registration); a union-tag predicate selector (grading rules); a latest-row-per-group
view; a guard on `now` against a stored timestamp plus a stored duration (D3); and `Decimal` in an
immediate view, which costs 15 conformance scenarios
(`Decimal and Binary64 replay results are unsupported`).

### Decisions taken during decomposition (2026-10-15)

| # | blocker or finding | decision |
|---|---|---|
| D11 | M0 decomposer: generated serde types carry `Decimal` as a JSON string | the wire form of a number is the one ESS generates: a decimal string (`{"value": "0.029"}`). Exact, and no `serde_json` precision flag in consumers |
| D12 | `decision-blocker:m3-advisory-gate-verdict` | an advisory gate is graded on its finding count (`count_of_cases`, unit `{finding}`) against a declared `fixed` baseline: above → `WARN`, at or below → `OK`, never `CRIT`; no count printed → `UNKNOWN` |
| D13 | `decision-blocker:m2-transition-evidence` | a transition references the k evaluations that settled it and copies series, definition_rev, from, to, first and settling ts and settling score; it outlives raw retention, and a missing evaluation reads as expired |
| D14 | `decision-blocker:m2-rollup-revision` | a rollup counts each sample's evaluation at the highest definition_rev present when the rollup is built |
| D15 | `decision-blocker:m2-persistence-parameters` | k, k_crit and m live on the definition as `persistence: {k, k_crit, m}`, defaults 2, 1, 3 |
| D16 | `decision-blocker:m2-sampling-gap-record` | a missed slot is a sample with quality `unknown` and reason `missed_slot`; no separate gap record, never re-read, never interpolated |
| D17 | `decision-blocker:m1-connectors-dimension-registry` | through M3 a dimension registry is `file` or `open`; a `connectors:` registry is refused at load until Connectors offers an inventory operation |
| D18 | `decision-blocker:m1-dimension-value-origin` | `denominator.since` admits `first_sample`, a fixed timestamp, and `dimension.created` only for a `file` registry entry carrying `created` |
| D19 | M1 decomposer: where a definition's lifecycle state lives once the service runs | the definition file is the only copy of a definition, its lifecycle field included. The service never rewrites it: a `Muted` definition whose expiry has passed is sampled as `Active`, and the service records an `UnmuteDefinition` event in the store (D9) so the change is visible. The file keeps its stale mute until someone edits it, and the service logs a warning naming it on each pass |
| D20 | adversary passes on the Loki, SQL and Prometheus adapters (wave 20261015c) | in a ratio, a series the complete denominator answer carries and the complete numerator answer lacks reads numerator 0: counting queries (LogQL `count_over_time`, SQL `GROUP BY`, a Prometheus counter child never incremented) emit nothing for zero. A cut or incomplete answer on either side reads unknown; a series missing from the denominator reads unknown with no population |
