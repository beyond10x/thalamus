---
format: aep.planning-md/3
id: story:m3-loki-adapter
kind: story
status: active
title: Loki metric queries become typed samples through a Connectors connection
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/loki.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/loki
- confidence: inferred
  path: crates/thalamus-sources/tests/loki.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T04:58:56Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T04:58:56Z", actor: "human:timo", revision: 5}
---
## Context

Design § 5.1 lists Loki/LogQL among the `thalamus-sources` adapters; § 5.2: thalamus reads a source only through a Connectors connection, holds no secret and never writes to a source. This story adds the `loki` adapter on the M1 adapter trait and Connectors client (`epic:signal-contract`); the client invokes `connectors operations invoke`, the pattern cortex uses. Code is Rust (AGENTS.md).

## Connectors

Exists. Connectors v0.45.0 (`94e0805ed`), `adapters/loki/` (`adapters/loki/README.md`): `logs.query_metric` (an instant or range metric query), `logs.query_range` (log lines, bounded by `limit`), `logs.labels`. Validation evidence lasts 60 s; a scheduled read revalidates first (same README, *Evidence lifetime*).

## Behaviour

- `logs.query_metric`: one sample per returned series. A `dims` key is set only from a label the series carries and the definition declares (L9).
- `loki.ratio` reads the definition `numerator` and `denominator` queries; the value is their quotient, a `Decimal` with unit `1` (L4, design § 11).
- A value Loki returns as `NaN`, `+Inf` or text that does not parse: `quality: unknown` with a reason, never 0 (L5).
- `logs.query_range` counted as `count_of_cases`: a result that reached the request `limit` is `partial`, bound `lower` (design § 2.2).
- A Connectors refusal or provider failure: `quality: unknown`; the reason carries the Connectors code and stage.
- `provenance.query_sha256` is the SHA-256 of the exact LogQL text.

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one; the sample references the definition, which exists first and outlives it. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `dims` keys → registered `Dimension`: no `relations:` entry; `spec/domains/observation.yaml` field `dims` carries an `UNMAPPED:` marker, decided by design § 2.4 and held by an M1 Rust test. This story relies on it and does not implement it.
- `SignalDefinition.source` → a Connectors connection: the typed field `thalamus.definition.Source` in `spec/domains/definition.yaml` (one adapter, one connection id), not an entity relation.

## Acceptance

`cargo test -p thalamus-sources --test loki` passes, with these known-answer tests over recorded Connectors answers in `crates/thalamus-sources/tests/fixtures/loki/`, each seen failing before the adapter exists: `loki_metric_series_become_samples_with_source_labels_only`, `loki_ratio_is_decimal_with_unit_1`, `loki_nan_value_is_unknown_never_zero`, `loki_line_limit_reached_is_partial_lower_bound`, `loki_refused_read_is_unknown_with_reason`.

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-sources/src/loki.rs` (replaces the one-line `adapter()` stub `story:m1-connectors-client` leaves there; the module line is `story:m0-workspace`'s and the registry arm `story:m1-connectors-client`'s, so no other file changes)
- `crates/thalamus-sources/tests/loki.rs` (new)
- `crates/thalamus-sources/tests/fixtures/loki` (new)

## Not in this story

Dashboard and alert-rule export for Loki definitions (M4, design § 6.5).
