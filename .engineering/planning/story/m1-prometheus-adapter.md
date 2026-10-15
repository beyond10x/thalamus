---
format: aep.planning-md/3
id: story:m1-prometheus-adapter
kind: story
status: implemented
title: Prometheus reads through Connectors become typed readings; a ratio is a decimal with unit 1, NaN is unknown
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/prometheus.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/prometheus
- confidence: inferred
  path: crates/thalamus-sources/tests/prometheus.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T04:58:55Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T04:58:55Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-15T05:41:40Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Outcome

A Prometheus signal is read through a Connectors connection into typed readings: a ratio is a number with unit `1`, and a value Prometheus could not give is `unknown`, never 0.

## Context

`epic:signal-contract`: "the Prometheus adapter in `thalamus-sources`" and the failing-first test "a ratio is numeric with unit `1`". Design § 2.2 (reading, provenance), § 1 L4, L5, L9. Connectors v0.45.0 ships a `prometheus` adapter with the reads `series.query` (instant, optional `time_unix_s`), `series.query_range` (at most 7 days and 2,000 points per series) and `rules.list`; values stay Prometheus strings (`NaN`, `+Inf`) and a provider warning makes a result `complete: false` (Connectors `adapters/prometheus/README.md`). Built on the client, adapter trait and extractor registry of `story:m1-connectors-client`, which leaves a one-line `adapter()` stub in `prometheus.rs` that this story replaces.

## Work

- `crates/thalamus-sources/src/prometheus.rs`, read operation `series.query` only:
  - `promql.value` reads `query.value` with `series.query` at the window end; `promql.ratio` reads `query.numerator` and `query.denominator` at the same instant and divides them per series; the denominator, rounded down, is `denominator.n` with the unit of the definition `denominator`.
  - Result labels named as one of the definition `dimensions` become `dims`; other labels are dropped; a declared key the series does not carry stays unset (L9). Two result series that fall onto the same `dims` make both reads `unknown`, naming the collision.
  - `NaN`, `+Inf`, `-Inf` or text that does not parse as a decimal: `unknown` with a reason. `complete: false`, `series_truncated` or `samples_truncated`: every read of the pass `unknown`, reason naming the cut and the warnings.
  - `0 / 0`: reading `unknown` with reason `empty population` and `denominator.n = 0`, so grading answers `N/A` (`story:m1-verdict-grading`).
  - `raw` is the Prometheus value string as returned; `query_sha256`: SHA-256 of the exact query text, for a ratio of the canonical JSON of the `query` map (keys sorted).
  - Window: `[end - window, end]`; the query text is used as written, so an offset read is the same text at `end - offset`.
  - `denominator.since` and `age_s` are not set here: the registry and the pipeline resolve them (`story:m1-instance-registry`, design § 11 D18). The adapter reports the source's retention start only if the source exposes one; none of the Connectors v0.45.0 `prometheus` reads does, so `retention_start` stays unset.

## Domain relations

- `thalamus.definition.SignalDefinition.source` → one Connectors adapter instance and connection. Inferable, inferred from cortex `src/connectors.rs:209`, as recorded in `story:m1-connectors-client`.
- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`. Inferable: `spec/domains/observation.yaml`, relation `definition`. The adapter produces reads; the sample is recorded by the caller.
- `dims` keys come from the definition `dimensions` and are set only when the source carries them. Inferable: `spec/domains/observation.yaml:142` (`UNMAPPED:`) and `spec/domains/dimension.yaml`, `thalamus.dimension.DimensionRule` `source_only`.
- The reading is the tagged union `thalamus.observation.Reading` (`spec/domains/observation.yaml:73`); a ratio has unit `1` (`thalamus.definition.Unit` summary, `spec/domains/definition.yaml:35`).

## Decisions taken (reversible)

- The adapter never emits `partial`: Prometheus does not say which side a cut result bounds, so a cut read is `unknown`.
- `denominator.n` is the denominator rounded down; `increase()` answers fractional counts.

## Scope

Planned (inferred):

- `crates/thalamus-sources/src/prometheus.rs` (its module line and registry arm are `story:m0-workspace`'s and `story:m1-connectors-client`'s)
- `crates/thalamus-sources/tests/prometheus.rs`, `crates/thalamus-sources/tests/fixtures/prometheus` (recorded answers with neutral labels)

## Acceptance

`cargo test -p thalamus-sources --test prometheus` passes, over recorded Connectors answers through the double, the tests `ratio_reading_is_a_decimal_with_unit_1` (written and seen failing first; `15 / 556` is `0.0269…`, never `2.69` and never a string), `nan_value_is_unknown_never_zero` (written and seen failing first), `incomplete_result_makes_every_read_unknown`, `labels_outside_dimensions_are_dropped`, `colliding_series_are_unknown`, `empty_population_is_unknown_with_n_0`, `query_sha256_is_of_the_exact_text`, `offset_read_shifts_time_only`, `raw_is_the_prometheus_value_text` and `since_and_retention_start_are_left_unset`.

## Out of scope

`rules.list` and `series.query_range`; resolving `denominator.since` (`story:m1-instance-registry`, `story:m1-cli-sample`); Grafana-mediated placement; Loki and every other source (M3); export of recording and alert rules (M4).

## Decision applied (2026-10-15, wave 20261015b)

`query_sha256` is the SHA-256 of the canonical JSON of the whole query map (keys sorted, compact), for one entry or many, as `story:m1-connectors-client` implements it after its adversary pass (a bare-text digest cannot be told apart from some map's canonical JSON). The test `query_sha256_is_of_the_exact_text` becomes `query_sha256_is_of_the_canonical_query_map`.
