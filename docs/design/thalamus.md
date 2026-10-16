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

The record the store writes, one event body per sample, in the generated wire form (numbers that
are `Decimal` are decimal strings, D11; an absent optional is an absent member):

```json
{"schema": "thalamus.sample/1",
 "sample_id": "smp_000000000042",
 "signal": "platform.namespace.request_failure_ratio",
 "series": "platform.namespace.request_failure_ratio{cluster=\"prod-a\",namespace=\"shared-1\"}",
 "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z",
 "window": {"start": "2026-10-14T16:57:03Z", "end": "2026-10-14T22:57:03Z"},
 "reading": {"quality": "ok", "value": {"value": "0.029"}}, "unit": "1",
 "denominator": {"n": 556, "unit": "{request}", "since": "2025-03-02T00:00:00Z", "age_s": 51403023},
 "provenance": {"producer": "thalamus-service", "version": "0.1.0", "definition_rev": 1,
                "adapter": "sql", "connection": "platform-replica",
                "query_sha256": "…", "run_id": "run_000000000042"}}
```

| field | rule |
|---|---|
| `reading` | a tagged union on `quality` (spec `thalamus.observation.Reading`): `ok` carries a decimal, `partial` a decimal and its `bound`, `unknown` no value at all. Never a formatted string (L4) |
| `quality` | `ok`: read completely. `partial`: read but cut (a full page, a paged read that could not finish); `value` is then a bound and `bound` says `lower` or `upper`. `unknown`: not read; the reading carries only its `reason` (`{"quality": "unknown", "value": {"reason": "timeout"}}`) |
| unknown is not 0 | `quality: unknown` carries no value. In Rust this is an enum, so the illegal pair cannot be built (L5) |
| `denominator.since`, `age_s` | the earliest moment the population could contain anything: the dimension's creation time, the source's retention start, the first sample of the series. A baseline whose shifted window starts before the series began (offset plus window span greater than `age_s`) is `UNKNOWN` (§ 3.1) |
| `provenance` | enough to re-run the read: producer and version, definition revision, adapter and connection, the SHA-256 of the query map's canonical JSON, and the run id grouping one sampling pass |
| `text` | optional: the producer's value exactly as the source printed it (a text cell, an exit code, an HTTP status). Recorded only under a `producer_mapped` definition and only when the adapter read the value (an `ok` reading, or an `unknown` text cell the adapter types as text). The sample keeps its own `reading` all the same; grading maps `text` through the definition's `evaluation.mapping` at the revision being graded, so a mapping revision re-grades history (§ 2.3, D44) |
| ids | `dims` keys come from the dimension registry and every value must resolve in it (§ 2.4) |

Sample, evaluation and assertion ids (`smp_`, `evl_`, `asr_` and twelve digits) are store-local,
from a never-decreasing per-prefix sequence the store resumes on open.

### 2.3 `SignalEvaluation`

The record the store writes for the evaluation of that sample, in the same wire form:

```json
{"schema": "thalamus.evaluation/1",
 "evaluation_id": "evl_000000000042", "sample_id": "smp_000000000042",
 "signal": "platform.namespace.request_failure_ratio",
 "series": "platform.namespace.request_failure_ratio{cluster=\"prod-a\",namespace=\"shared-1\"}",
 "dims": {"cluster": "prod-a", "namespace": "shared-1"},
 "ts": "2026-10-14T22:57:03Z", "definition_rev": 1,
 "verdict": "WARN", "prev": "OK", "score": "1.158",
 "baseline": {"kind": "pooled", "offsets": ["1d", "7d"],
              "reading": {"quality": "ok", "value": {"value": "0.013"}}},
 "reason": "0.029 >= 2 x 0.013 and >= 0.02, 16 events of 556 >= min_events 10"}
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
| deploy instants | `thalamus.deploy.DeployInstant` records in the instance store, kept without a retention limit (D39) | a few per day; the before/after view works when the source is unreachable and after the source has dropped the object (a replaced pod) |

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
| `thalamus-core` | library | definition registry and validation, baselines, the verdict ladder, hysteresis, score, rollups, and the dashboard and rule export (`export/`: definitions in, text out). No I/O, no clock: time and samples are passed in |
| `thalamus-store` | library | the append-only sample stream, rollups and deploy instants (`deploys.rs`, D39) behind one trait. First provider: Eventlog (SQLite, PostgreSQL) |
| `thalamus-sources` | library | adapters that turn a Connectors operation into samples: Prometheus/PromQL, Loki/LogQL, SQL, Kubernetes, Git, HTTP probe, test-run events; the local host collector (`host.`), which reads this machine's CPU, memory, disk space and CPU temperature and no Connectors operation (D46); and the deploy readers (`deploys/`) that turn a Connectors read into deploy instants, origins `pod_created` and `merged` (D39) |
| `thalamus-pipeline` | library | the read → sample → grade → evaluate pass the CLI and the service share, and the one CloudEvents envelope builder (`cloudevents.rs`, D33): the service calls it from M4, and the CLI can call it from M5 (`story:m5-conductor-contract`) without linking the service |
| `thalamus-service` | binary | the scheduler that samples each definition on its cadence, the HTTP API (current verdicts, series, assertions), the views (`src/views/`), a server-sent-events stream of evaluations, and the CloudEvents emitter. `thalamus-service run --instance <dir> [--listen <addr>] [--allow-remote]`: without `--listen` no HTTP is served; `--listen` binds a loopback address only unless `--allow-remote` is given |
| `thalamus-cli` | binary `thalamus` (clap derive) | `define validate`, `sample`, `eval`, `show`, `compare --vs 7d`, `assert`, `rollup`, `flaps`, `view`, `export rules`, `export dashboard`, `deploys record`, `deploys list` |

`thalamus-core` stays pure so the same code grades a live sample, re-grades history after a
threshold change, and runs in a test without a source.

The M4 routes of `thalamus-service`, every one `GET` (D31): `/` (an index linking every view),
`/api/verdicts`, `/api/series`, `/api/evaluations/stream` (the server-sent-events stream, D34),
`/views/live`, `/views/heatmap`, `/views/sparklines`, `/views/week`, `/views/deploys` and
`/views/deploys/{id}`. Any other method on any route answers 405: the service never accepts a write
over HTTP. There is no authentication in M4: the service runs per operator machine (§ 10 question
2) and binds loopback only unless `--allow-remote` is given (D43). HTML and SVG are rendered by Rust,
with no `<script>` element (D32).

### 5.2 Boundaries

- **No secret enters thalamus.** A source names a Connectors connection; thalamus only invokes
  operations through Connectors, the pattern cortex uses. The one exception is the local host
  collector (D46): source `{adapter: host, connection: local}` reads the kernel interfaces of the
  machine thalamus runs on (`/proc/stat`, `/proc/meminfo`, `statvfs`, the CPU sensor under
  `/sys/class/hwmon` or `/sys/class/thermal`) directly, holds no secret, writes nothing and never
  reads another machine. It runs only inside a read; it is no service, timer or daemon.
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
| before/after a deploy | every signal's window before vs after one deploy instant, sorted by score delta | evaluations, samples, deploy instants |
| week vs week | this week against the same slot last week | evaluations at the slot 7 days earlier |
| live | current verdict per series, updated from the event stream | evaluations |

Every view except `/views/live` answers `format=html` or `format=json`; the JSON is what the view
is drawn from (D35). The live page's machine-readable form is the evaluation stream (D34).

- **Heatmap** (D35). Columns are UTC days (default 28) or UTC hours, ending with the current one.
  Rows are the series with an evaluation in the range, grouped by layer (the second segment of
  `<family>.<layer>.<name>`), inside a layer by worst cell, then series key. A cell reads the
  evaluations whose `ts` falls in its column, one per sample at the highest `definition_rev`
  present (the rule D14 sets for rollups). Its verdict is their worst by ladder rank; a scored
  worst verdict colours it with the highest score among them; a worst `UNKNOWN` is hatched; a cell
  whose every evaluation is `N/A` is grey; a cell with no evaluation is a gap, drawn empty and
  distinct from all three. The scale is fixed and diverging: 0 at its neutral midpoint, 1 at the
  WARN colour, 2 at the CRIT colour, clamped beyond 3 and below -2. The heatmap reads raw
  evaluations only; the 35-day retention (§ 4) covers 4 weeks.
- **Sparklines** (D36). The value line runs through `ok` readings and breaks at every `unknown`
  sample (a missed slot included) and every slot with no sample; a `partial` reading is a bound
  marker on its `bound` side, not a point of the line. The baseline band spans from the recorded
  baseline of each evaluation up to its WARN line, and is absent where the baseline is unknown.
  The WARN and CRIT lines are computed from the definition by the code that grades, never typed
  again (L2), and are drawn only at points graded at the current revision of the definition.
- **Week vs week** (D37). This week is the rolling 7 days ending at the start of the hour of `at`
  (default now): 168 UTC hour slots against the same slots 7 days earlier. A slot's score delta
  exists only when both cells are scored, never 0. A slot of last week before the first stored
  sample of the series is `UNKNOWN` with reason `baseline predates series`; a slot with no
  evaluation is a gap.
- **Before/after a deploy** (D38). Before is `[ts - window, ts)`, after is `[ts, ts + window)`;
  `window` defaults to 2h and is at most 7d. Per side: the highest score among its scored
  evaluations (one per sample at the highest revision present), and beside it the median of its
  `ok` readings. Delta = after score - before score;
  a side with no scored evaluation is `UNKNOWN` with a reason and has no delta, never 0. Every
  series with an evaluation in either window is listed, sorted by delta descending, rows without a
  delta last.
- **Live** (D40). Seeded on start from the newest evaluation of each series in the store, then
  updated from the evaluation stream only. One row per series, worst first; `UNKNOWN` hatched,
  `N/A` grey, never drawn as `OK`. The page refreshes itself every 10 s with
  `<meta http-equiv="refresh">` and carries no `<script>`.

A **deploy instant** is the time a workload actually changed (pod creation, merge time), never an
announcement. It is a stored `thalamus.deploy.DeployInstant` record (D39), read through Connectors
by `thalamus deploys record` and never from the source when a view is drawn. Its origin is one of
two: `pod_created`, the earliest pod creation of one rollout (the pods of one ReplicaSet), and
`merged`, the merge time of a merged merge request. Another origin is a specification change.

Views are served by `thalamus-service`. Dashboard JSON and Prometheus recording and alert rules are
generated from definitions as an **export**, only for definitions whose source is Prometheus or
Loki; a dashboard tool cannot see SQL, orchestrator or version-control signals, so it is never the
only view.

- **Rules** (`thalamus export rules --target prometheus` or `--target loki`, D41). Each exported
  definition's query parts are recorded as `thalamus:<signal id, dots as underscores>:<part>`;
  baselines are expressions over those recorded series with `offset`. An alert's `for:` is `k`
  times the cadence, `k_crit` for `CRIT`. Definitions that cannot be exported are skipped, each
  with a reason.
- **Dashboard** (`thalamus export dashboard`, D42). One Grafana dashboard JSON model per instance:
  one row per layer, one panel per exported definition, reading the recorded series of the rules
  export, so the dashboard and the alerts show one expression.

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

The answers (`story:m0-probes`):

1. **Expressed in ESS.** `thalamus.observation.Reading` is a union tagged on `quality` (§ 11); its
   `unknown` variant, `UnreadReading`, carries a `reason` and nothing else, so there is no nullable
   number at all. `thalamus-spec-types` refuses a value under `unknown` (`deny_unknown_fields`).
   The model's server decoder ignores the extra member and drops it, so the in-memory
   `Reading::Unknown` never carries a value. `crates/thalamus-core/tests/probes.rs`
   (`unknown_reading_carries_no_value`, `ok_reading_requires_value`) holds this.
2. **Not expressible.** ESS has no pattern or grammar-typed string, so `thalamus.definition.Unit`
   declares only the character set (ASCII 33–126), under an `UNMAPPED:` marker. Neither generated
   decoder enforces that set (an ESS generator gap), so a space, a control character, non-ASCII
   text or an empty unit decodes; `parse_ucum` refuses all of them. The grammar is
   `thalamus_core::unit::parse_ucum`, held by `ucum_admits_design_units` and
   `ucum_refuses_non_units` in the same file. The gap is listed in § 11.

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
| 2 | Does the service run per operator machine, or in-cluster next to the systems it watches? | per operator machine through M5; an in-cluster deployment is a later milestone (D43) |
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
| D18 | `decision-blocker:m1-dimension-value-origin` | `denominator.since` admits `first_sample`, a fixed timestamp, and `<key>.created` for a declared dimension with a `file` registry entry carrying `created`. `dimension.created` is accepted when exactly one declared dimension has a `file:` registry |
| D19 | M1 decomposer: where a definition's lifecycle state lives once the service runs | the definition file is the only copy of a definition, its lifecycle field included. The service never rewrites it: a `Muted` definition whose expiry has passed is sampled as `Active`, and the service records an `UnmuteDefinition` event in the store (D9) so the change is visible. The file keeps its stale mute until someone edits it, and the service logs a warning naming it on each pass |
| D20 | adversary passes on the Loki, SQL and Prometheus adapters (wave 20261015c) | in a ratio, a series the complete denominator answer carries and the complete numerator answer lacks reads numerator 0, but only when every numerator series has a denominator twin (numerator keys a subset of denominator keys); otherwise the whole pass reads unknown naming the unmatched numerator series (the Prometheus adapter narrows this to the passes where a 0 would be inferred; an unmatched numerator series alone reads unknown with no population). Counting queries (LogQL `count_over_time`, SQL `GROUP BY`, a Prometheus counter child never incremented) emit nothing for zero. A cut or incomplete answer on either side reads unknown; a series missing from the denominator reads unknown with no population; a complete numerator answer with no series at all over a non-empty denominator reads unknown (a renamed metric or a broken matcher looks the same as zero everywhere) |
| D21 | `story:m3-http-probe-adapter` implementor (wave 20261015c), from the Connectors v0.45.0 source, not observed at runtime | `http_probe.up` reads 0 only when the probe's own invoke is refused at `dispatch` with service code `not_found` or with `timeout`; every other refusal reads unknown. Connectors folds a provider 5xx into its own `unavailable` at `readiness`, the same envelope as its own outage, so a 5xx reads unknown until Connectors tells the two apart. 401 and 403 read unknown: the target answered and the credential was refused, an instrument failure. A probe invokes only a catalog operation whose id ends in `.get` or `.list` (every such operation in the v0.45.0 catalog is `effect: read`); any other id reads unknown and nothing runs. The operation is invoked only when its `operations describe` answer names profile `generic-http`, which the catalog gives exactly its reads (a catalog write is `mutation`; every other adapter names its own operation profile, such as `kubernetes-get` or `promql-range`, never `generic-http`); otherwise it reads unknown and is described, never invoked. A non-catalog 404 (a kubernetes resource that does not exist) says nothing about availability |
| D22 | `story:m1-verdict-grading` implementor and coordinator (wave 20261015d) | `min_events` counts the events at stake. For `higher_is_worse` and `ratio_vs_pooled_baseline` they are the current value's (value times `n` for a ratio, the value for a count); for `lower_is_worse`, a drop, they are the baseline's (baseline times `n` for a ratio, the baseline value for a count). A count that drops from 100 to 0 crosses a line with `min_events: 10`; a drop from 3 to 0 does not. Since the baseline's events do not depend on the reading, a partial read's upper bound already under the CRIT line of a drop escalates to `CRIT` whatever its `min_events`. A ratio's event count is the number times `n` rounded to the nearest whole event, half away from zero, both against `min_events` and in the reason: 10 of 30 crosses `min_events: 10`, and 15 of 556 reads `15 events of 556`. The score is monotone in the value: past the CRIT line of a drop it is linear (from 1 at the WARN line through 2 at the CRIT line), so a value of 0 or below scores no better than any positive value; between the marks it stays log-linear for a multiplier line (§ 3.3) |
| D23 | `story:m1-verdict-grading` adversary and coordinator (wave 20261015d) | an `x` part of a line is decidable only over a baseline above 0. Over a baseline of 0 or below the `x` part is not used: a line that also declares `abs` is decided by `abs` and `min_events` alone, and a line that declares only `x` grades `UNKNOWN` with a reason naming the baseline, because the instrument cannot decide; unknown is never `OK`. The CRIT line is read first: when it is decidable and crossed the verdict is `CRIT` even if the WARN line cannot be decided (a partial lower bound by § 3.1 rule 4), and only below a decidable CRIT line does an undecidable WARN line give `UNKNOWN` |
| D24 | operator, 2026-10-15 | no File provider: an instance stores samples and evaluations in Eventlog on SQLite for a local instance and on PostgreSQL for a shared one (`sqlite:<path>` or a PostgreSQL URL). PostgreSQL in M1 is a loopback or Unix-socket server that authenticates the connection itself; the locator carries no password, because no credential enters thalamus. The hosted composition (verified TLS, a separate migration role, a connection budget) waits for the in-cluster deployment (§ 10 question 2). Amended after the second adversary pass (wave 20261015d): ids stay `smp_`/`evl_` and twelve digits (§ 2.2), and writers sharing a store never mint one twice. Before a number is answered, it is reserved on the instance's id stream (`thalamus.ids`, beside the instance stream), at the version the handle last read. A writer that meets another writer's reservation reads the id stream again and takes the next number. The instance stream keeps only the § 2.2 and § 2.3 records, and the open does not read the id stream. An append that meets another writer's append takes that writer's records in and retries, up to 64 attempts, and then answers an error naming the count. An id already stored with another body is refused `thalamus.record.id-taken` from every handle, including a stale one. An open that waits on another opener's schema lock retries for up to 30 s. TLS, Eventlog's `verified` connection, is not wired yet, so a locator whose `sslmode` is anything but absent or `disable` (or that asks for direct TLS negotiation) is refused `thalamus.store.postgres-tls`; it is not silently answered in plaintext |
| D25 | `story:m2-spec-history-records`: the N of `rolling_median` and the number of `fixed` (§ 3.1) had no field | both are fields of the definition's grading block, its one copy of every threshold: `evaluation.rolling_n` and `evaluation.fixed` (`spec/domains/definition.yaml`, `thalamus.definition.EvaluationSpec`). A number belongs with the baseline that reads it, and nothing else holds one: `ProposeDefinition` and `ReviseDefinition` refuse `rolling_median` without a `rolling_n` of at least 1 (`rolling-n-missing`), `rolling_n` without `rolling_median` (`rolling-n-unused`), a `fixed` baseline without `evaluation.fixed` (`fixed-number-missing`) and `evaluation.fixed` without a `fixed` baseline (`fixed-number-unused`). ESS 0.60.0 cannot state "an N below 1" as a guard synthesis accepts (ESS-SYNTH-002 on the optional N, ESS-SYNTH-022 once it is guarded by `defined`), so that part of `rolling-n-missing` is an `UNMAPPED:` marker held by `crates/thalamus-core/tests/conformance.rs::rolling_median_over_fewer_than_one_sample_is_refused`. `thalamus_core::baseline::fold` reads the `fixed` number from the definition, and a `fixed` baseline handed no number is still `UNKNOWN`, never 0. `persistence: {k, k_crit, m}` (D15) is an optional field of the definition, its defaults applied by `thalamus-core` (D15). A `persistence` block declares all three of `k`, `k_crit` and `m`; only the whole block may be absent, and then the defaults 2, 1, 3 apply |
| D26 | `story:m2-spec-history-records`: where the settled transition (D6, D13) and the daily rollup (§ 4) live | each is its own domain, `thalamus.transition` and `thalamus.rollup`, so neither changes the digest of `thalamus.evaluation` or `thalamus.observation`. A transition is minted by the service (`trn_`) through `RecordTransition`, granted to the actor `thalamus.transition.Hysteresis`. A rollup's id is given by its writer, spelled from the series key and the day (an `UNMAPPED:` marker), so `RecordRollup` refuses a second row for one series and day (`already-rolled-up`). Each domain also declares a view of every row with its identity and state: synthesis observes `already-rolled-up` through `thalamus.rollup.Rollups` (ESS-SYNTH-001), and `thalamus.transition.Transitions` is what gives a transition a generated storage port, since its only command and its series view are obligations |
| D27 | `story:m2-spec-history-records`: the rollup invariant `unknown + partial <= n` | an ESS predicate compares a fact with a fact or a literal, not with a sum, so the specification holds `unknown <= n` and `partial <= n` and the sum is an `UNMAPPED:` marker held by a Rust test. A gap to report to the ESS repository beside those listed above. The synthesized conformance fixtures may therefore hold rows that break the sum (`n` 1, `unknown` 1, `partial` 1); The same fixtures may also pair a `worst_verdict` with `verdict_counts` that contradict it (`OK` beside a `crit` count of 1) and hold verdict counts whose sum exceeds `n`, which D14 rules out. `story:m2-rollup`'s builder test holds the sum, the worst verdict and the counts for every row thalamus writes. `verdict_counts` is one count per ladder verdict (`ok`, `na`, `warn`, `crit`, `unknown`) |
| D28 | `story:m2-spec-history-records`: D9 and D19 name the service as the actor of an expired mute's `UnmuteDefinition` | the actor is `thalamus.definition.Service`, granted `UnmuteDefinition` and nothing else. The grant is unconditional in ESS, which has no guard on a grant and no outcome that tells the service from an `InstanceOperator`; that the service unmutes only past the mute's expiry is an `UNMAPPED:` marker on the actor, held by the test of `story:m2-auto-unmute` |
| D29 | adversary pass 1 on `story:m2-spec-history-records` (coordinator, 2026-10-15) | the definition file (D19) carries `evaluation.fixed`, `evaluation.rolling_n` and `persistence`, and the loader (`thalamus_core::registry`) refuses what `ProposeDefinition` and `ReviseDefinition` refuse (D25), by the same rule (`thalamus_core::behaviour::definition::input_refusals`); a repeated key and a number that is not a decimal are refused as everywhere else in the file. `persistence` with `k`, `k_crit` or `m` below 1 is refused (`persistence-below-one`). A transition is a change: `Transition` holds `first_ts <= settled_ts` and `from_verdict` absent or different from `to_verdict` |

### Views and exports, M4 (2026-10-15)

Decisions the M4 stories take, as each states it: decomposer calls, reversible, unless the row names
a coordinator decision. § 5.1 and § 6.5 follow them.

| # | story | decision |
|---|---|---|
| D30 | `story:m4-skeleton`; the forms from `story:m4-export-rules`, `story:m4-export-dashboard`, `story:m4-deploy-instants` | `thalamus` gains `export` and `deploys`: `thalamus export rules --target prometheus\|loki [--out <file>]` (stdout by default, the skipped list with reasons on stderr, exit 0, refusals with the codes of `story:m1-cli-sample`) and `thalamus export dashboard [--out <file>]` (stdout by default, exit codes as `export rules`); `thalamus deploys record --reader kubernetes\|gitlab --source <adapter> --connection <id> [--namespace <ns>]... [--project <p>] [--since <duration>]`, which prints each recorded or already-recorded id, and `thalamus deploys list [--since <duration>] [--format text\|json]`; exit 0 when every read instant was recorded or already recorded, refusals 65 and 74 |
| D31 | `story:m4-skeleton` | `thalamus-service run --instance <dir> [--listen <addr>] [--allow-remote]`. Without `--listen` no HTTP is served, exactly as in M2; with it the HTTP server runs on its own thread with its own tokio runtime beside the scheduler. `--listen` binds a loopback address only unless `--allow-remote` is given; there is no authentication in M4 (§ 10 question 2: the service runs per operator machine; D43). The route table is § 5.1's; any other method on any route answers 405, and the service never accepts a write over HTTP. HTTP tests run in process through `tower::ServiceExt::oneshot`, with no port and no client crate (coordinator decision C5) |
| D32 | `story:m4-skeleton` | views live in `thalamus-service` (`src/views/`), the dashboard and rule export in `thalamus-core` (`src/export/`, pure: definitions in, text out). No new crate, so § 5.1 keeps its crate table. HTML and SVG are rendered by Rust; no Node build and no `<script>` element (running code is Rust) |
| D33 | `story:m4-live` (coordinator decision C6) | one CloudEvents envelope builder, `thalamus-pipeline` `cloudevents.rs`: the CloudEvents 1.0 structured-mode envelope of § 6.2, `kind` one of `dev.beyond10x.thalamus.transition` and `dev.beyond10x.thalamus.evaluation`. The transition emitter and the evaluation stream both call it, and `thalamus-cli` can call it without linking the service; not in `thalamus-core`, which holds no wire format, and not in the service. The emitted transition events stay byte-identical |
| D34 | `story:m4-live` | `GET /api/evaluations/stream` is `text/event-stream`: one message per recorded evaluation, `event: dev.beyond10x.thalamus.evaluation`, `id:` the evaluation id, `data:` the D33 envelope (`subject` the series key of D10, `time` = `ts`, `data` the evaluation record). Publishing never blocks the scheduler, and a send with no subscriber is not an error. A subscriber that lags past the channel capacity gets one SSE comment naming the lag and is closed; it rereads `/api/verdicts`. No replay on reconnect (`Last-Event-ID` is ignored): the current state is `/api/verdicts`, the history is the store |
| D35 | `story:m4-heatmap` | the heatmap cell rule of § 6.5: one evaluation per sample at the highest revision present (D14's rule for rollups), the cell verdict their worst-of by the ladder so the colour and the verdict cannot disagree (§ 3.3), coloured by the highest score among them; worst `UNKNOWN` hatched, all `N/A` grey, no evaluation a gap. The heatmap reads raw evaluations only; drawing older columns from daily rollups is left for later. `format=json` on every view except `/views/live`, so the views are testable and the live acceptance can count columns without parsing SVG; the live page's machine-readable form is the evaluation stream (D34; coordinator, wave 20261015e, adversary pass 1 on `story:m4-design-record`) |
| D36 | `story:m4-sparklines` | the baseline band spans from the baseline to the WARN line, the inside-bounds zone. WARN and CRIT lines come from one public line computation in the threshold rule that grades (`thalamus-core` `lines`): the threshold rules give both from `evaluation.warn` and `evaluation.crit`; `fixed_warn_only` gives a WARN line at the first count above its fixed baseline and no CRIT line (D12); `producer_mapped` gives none; an `unknown` baseline gives none. Lines are drawn only where the evaluation was graded at the current revision (the definition file holds only the current one, D19), instead of drawing current thresholds over history graded at older ones |
| D37 | `story:m4-week-vs-week` | "this week" is the rolling 7 days ending at `at`, not the calendar week, so the view is full on any day; `at` makes it reproducible. The comparison reads stored evaluations at the slot 7 days earlier (the M2 `compare` stance), not the `offset_7d` baseline recorded on each evaluation, which a `history: source` signal re-reads from its source and records only as a value. Rows ordered by the largest score delta of the week, then series key |
| D38 | `story:m4-before-after` | the sorted score delta is computed from the stored evaluations of each window (the highest score per side, among scored evaluations, one per sample at the highest revision present), and the sample values are shown beside it: grading a window of samples against another would re-run grading with a second baseline, which M4 does not need. Every series, not only series sharing a dimension value with the deploy (§ 6.5 says "every signal"); the deploy dims are shown so a reader can match them. Default window 2h, the horizon of the § 6.1 assertion example; cap 7d, one week either side, so a request cannot ask for more than the raw retention covers on both sides (§ 4, 35 days). A window over 7d, a zero or negative window, or an unparseable duration answers 400 naming the parameter and the 7d cap, and nothing is read from the store. An unknown deploy id answers 404 |
| D39 | `story:m4-spec-deploy-instants`, `story:m4-deploy-instants` | a deploy instant is a stored record, not a value read from the source each time the view is drawn: the view then works when the source is unreachable and after the source has dropped the object (a replaced pod), and a deploy can be cited by id. Domain `thalamus.deploy`, entity `DeployInstant` (`ts`, `origin`, `workload`, `ref`, `dims`, `provenance`; lifecycle `Recorded` only), command `RecordDeployInstant` with outcomes `recorded` and `already-recorded`; `deploy_id` (prefix `dep_`) is derived from origin, connection and `ref`, so a second read of the same change is `already-recorded`. Two origins only, the two the design names: `pod_created` and `merged`; another origin is a spec change. Kept without a retention limit. `pod_created` is read from the pods still listed, the earliest pod `metadata.creationTimestamp` of one ReplicaSet; a ReplicaSet creation time is not used as a stand-in, and an object without a creation timestamp is never given a guessed one. `merged` is `merged_at` of a merged merge request, GitLab-hosted repositories only, the one repository surface Connectors offers. The § 2.4 dimension rule applies to deploy instants as to samples: a value that does not resolve refuses that instant. Instants are recorded by running `thalamus deploys record` after a deploy; recording them on the scheduler is left for later |
| D40 | `story:m4-live` | the live page is fed by the event stream on the server and refreshes itself every 10 s with `<meta http-equiv="refresh">`, instead of running a browser `EventSource` script, because no script is committed (D32). A browser script is the obvious later change if the operator rules that browser code is outside that rule |
| D41 | `story:m4-export-rules` | recorded series `thalamus:<signal id, dots as underscores>:<part>` per query part (`value`, or `numerator` and `denominator`), the value name with its OpenMetrics unit suffix. Baselines are expressed over recorded series with `offset` (`offset_1d`, `offset_7d`; `pooled` the sum of numerators over the sum of denominators; `higher_of` `max`, `lower_of` `min`; `fixed` the number in the definition), because `offset` attaches to a selector and cannot be applied to an arbitrary query text. An offset baseline therefore has no value until the recording rules have run for that offset, so an exported alert stays silent, never firing on a missing baseline (§ 3.1). Alerts `ThalamusWarn` and `ThalamusCrit`, one rule group per family, guarded by `min_n` and `min_events` where declared. `for:` = `k` times the cadence (`k_crit` for `CRIT`) approximates `k` consecutive evaluations (§ 3.2); Prometheus evaluates on its own interval. Loki definitions record in the Loki ruler; their alerts run in Prometheus over the recorded series, so one alert file holds every exported threshold. Only `Active` definitions; skipped with a reason: a source neither Prometheus nor Loki, a store baseline (`same_slot_median_4w`, `rolling_median`), a rule other than a threshold rule (`producer_mapped`, `fixed_warn_only`) |
| D42 | `story:m4-export-dashboard` | the dashboard JSON format is the Grafana model, the dashboard tool that reads both Prometheus and Loki; the design names no tool. Panels read the recorded series of the rules export (D41), so the dashboard and the alerts show one expression; the dashboard is empty for a signal until those recording rules run. One target per offset baseline; `fixed` WARN and CRIT lines as threshold steps read from the definition; the panel unit mapped from the UCUM unit; one template variable per declared dimension; the data source a template variable of type `prometheus` with no URL and no host. Definitions that are not exported get no panel |
| D43 | coordinator, wave 20261015e, adversary passes 1 and 2 on `story:m4-design-record`; `story:m5-skeleton` | the service runs per operator machine through M5 (§ 10 question 2); an in-cluster deployment comes later. D31's no-authentication rule depends on it: the service binds loopback only unless `--allow-remote` is given, and authentication comes with the in-cluster deployment. The M5 assertion routes `story:m5-skeleton` adds (`GET /api/assertions`, `GET /api/assertions/{id}`) are served on the same loopback-only listener |
| D44 | Producer vocabulary mapping (story:m3-vocabulary-mapping) | `evaluation.mapping` is `Optional<Map<String, thalamus.evaluation.Verdict>>`: a producer value, matched exactly as the source printed it, to a ladder verdict, so a name that is no verdict is refused by type. The mapping is applied at grading, not at sampling, so a mapping revision re-grades stored history (§ 2, § 5.1, D5). `thalamus.observation.SignalSample` gains optional `text`, the producer's value exactly as the source printed it (a text cell, an exit code, an HTTP status); it is recorded only under a `producer_mapped` definition and only when the adapter read the value: an `ok` reading with producer text, or an `unknown` read typed `Unread::Text` where the adapter builds it (today the `sql` adapter, for a cell that is present and not a number). The sample keeps its own reading: a text cell is `unknown` with the adapter's reason, an exit code `1` is `ok` with value 1; no read is rewritten before it is stored. A read the adapter could not make (cut, colliding, NULL, duplicate, unreachable) or only bounded (`partial`) records no text whatever `raw` it carries; the decision is the type, never the reason text. Under `producer_mapped`, grading maps `sample.text` through the mapping of the revision being graded and scores an `OK`, `WARN` or `CRIT` at its position (0, 1, 2); `UNKNOWN` is reserved for a sample with no text (its own reason, or "no producer value to map" for a number without text) and for a text the mapping maps to `UNKNOWN` or does not list, whose reason names the value and the revision. Every other rule ignores `text`. `mapping-missing` refuses, in the loader, `ProposeDefinition` and `ReviseDefinition` alike: no mapping (a bare `mapping:` or YAML null included, as JSON null on the wire), an empty mapping, and a mapping onto `N/A`; the last two are held in Rust because ESS 0.60.0 leaves a second child of the guard unwitnessed (ESS-SYNTH-022). A mapping whose every value is `UNKNOWN` is an explicit choice and is kept. `mapping-unused` refuses a mapping under any rule other than `producer_mapped`, and `thresholds-unused` refuses a `warn` or `crit` line under `producer_mapped`, which grades by the mapping alone; both are ESS guards. A table lists every value it maps: a blocking gate's table lists exits 1 to 255 as `CRIT`; there is no wildcard key. grade() still needs the baseline read, and a fixed baseline of 0, the position of `OK`, is the one the tests use. |
| D46 | Local host collector (story:m3-host-collector) | host data (the machine thalamus runs on) comes from a collector in `thalamus-sources`, not from a Prometheus and node-exporter stack: adapter id `host`, the reserved connection `local`, extractors `host.cpu_busy` (two reads of the `cpu` line of `/proc/stat` 250 ms apart, 1 − (Δidle + Δiowait) / Δtotal), `host.memory_used` ((MemTotal − MemAvailable) / MemTotal of `/proc/meminfo`), `host.disk_used` (`statvfs` of each mount the `query` lists, every value one mount, (f_blocks − f_bfree) / (f_blocks − f_bfree + f_bavail) as `df` computes it, dimension `mount`), `host.disk_free` (`statvfs` of each listed mount, f_bavail × f_frsize bytes, unit `By`, dimension `mount`) and `host.cpu_temperature` (the hottest `temp*_input` of a `coretemp`, `k10temp`, `zenpower` or `cpu_thermal` hwmon chip, else of a thermal zone of type `x86_pkg_temp` or `cpu-thermal`, in `Cel`), and nothing else. It is the one source that takes no Connectors operation (§ 5.2): reading this machine's own counters holds no secret and writes nothing. `ProposeDefinition` and `ReviseDefinition` refuse a `host` source whose connection is not `local` (`host-not-local`, an ESS guard), and the loader refuses it too; the adapter still reads such a definition `unknown` ("host reads only this machine (connection local)") and reads nothing. A source of any other adapter on connection `local` is refused (`local-not-host`). A missing file or field, a zero total, a CPU counter that went backwards, a mount that cannot be read, a listed path that is no mount point and a machine with no CPU sensor read `unknown` naming what was wrong, never 0; mounts are deduplicated by one spelling (`//` is `/`, no trailing slash). A host read measures now, so a window end more than 300 s from the machine's clock reads `unknown` ("host reads only this moment"). A free-space floor is instance data, said with `lower_is_worse`, `baselines: [fixed]` and `crit: {x: 1}`; `thalamus sample` reports a sample graded against `fixed` alone that grades WARN or CRIT as a `breach` line, records no evaluation and keeps its exit code. It runs only when sampled, like every definition; nothing it starts outlives the read |
