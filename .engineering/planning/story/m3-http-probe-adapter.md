---
format: aep.planning-md/3
id: story:m3-http-probe-adapter
kind: story
status: implemented
title: An HTTP probe samples the availability of one Connectors read operation
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/http_probe.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/http_probe
- confidence: inferred
  path: crates/thalamus-sources/tests/http_probe.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T06:14:41Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T06:14:42Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-15T08:20:44Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Context

Design § 5.1 lists an HTTP probe among the `thalamus-sources` adapters. § 5.2: thalamus invokes operations only through Connectors, holds no secret and never writes, so a probe is a Connectors read, not an HTTP client inside thalamus. This story adds the `http_probe` adapter on the M1 adapter trait and Connectors client (`epic:signal-contract`). Code is Rust (AGENTS.md).

## Connectors

No probe operation exists. Connectors v0.45.0 (`94e0805ed`) has no adapter that requests an arbitrary URL and reports status and latency. The closest surface is the catalog provider: every selected read answers `{status, body, provenance}` (`adapters/catalog/src/lib.rs:2199-2221`, `result_schema`). A non-2xx answer becomes an error envelope instead, its code taken from the status (`adapters/catalog/src/lib.rs:1854-1878`: 5xx is `unavailable`, 404 and 410 `not_found`, 429 `rate_limited`). Each probed API needs a pinned OpenAPI bundle compiled into Connectors (`adapters/catalog/design.md:138`), and no answer carries a latency.

This story therefore samples **availability** of one admitted catalog read operation. Latency and probes of arbitrary endpoints need a Connectors probe operation that is missing; that work belongs to the Connectors repository and is not part of this story.

## Behaviour

- `http_probe.up`: kind `gauge`, unit `1`. An `ok` answer (2xx) is value 1.
- A provider-stage error (the request reached the target, which answered 5xx, 404 or another refusal) is value 0, with the Connectors code in the reason. This rule is inferred from `adapters/catalog/src/lib.rs:1854-1878` and design § 6.3 ("an instrument failure maps to `quality: unknown`, a defect to a failing value").
- An admission-stage refusal (`not_found`, `not_granted`: the connection, not the target) and `rate_limited`: `quality: unknown` with a reason, never 0 and never 1.
- No `dims` key is derived; the definition may declare none (an empty list, design § 11 D8).

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one; the sample references the definition, which exists first and outlives it. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `SignalDefinition.source` → a Connectors connection: the typed field `thalamus.definition.Source` in `spec/domains/definition.yaml`, not an entity relation.

## Acceptance

`cargo test -p thalamus-sources --test http_probe` passes, with these known-answer tests over recorded catalog answers in `crates/thalamus-sources/tests/fixtures/http_probe/`, each seen failing before the adapter exists: `probe_ok_answer_is_up_1`, `probe_provider_unavailable_is_up_0_with_reason`, `probe_admission_refusal_is_unknown_never_down`, `probe_rate_limited_is_unknown`.

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-sources/src/http_probe.rs` (replaces the one-line `adapter()` stub `story:m1-connectors-client` leaves there; the module line is `story:m0-workspace`'s and the registry arm `story:m1-connectors-client`'s, so no other file changes)
- `crates/thalamus-sources/tests/http_probe.rs` (new)
- `crates/thalamus-sources/tests/fixtures/http_probe` (new)

## Not in this story

Probe latency and arbitrary-URL probes (no Connectors operation).

## Correction (2026-10-15, from the implementation)

- Scope: a probe names its operation per definition, so `adapter.rs` gains `Adapter::operations_for(definition)` (defaults to `read_operations`) and `registry.rs` calls it; `connectors.rs` keeps the Connectors `service_code` on a refusal, because on the v0.45.0 wire a 404 and `rate_limited` both arrive as `service_failure` at `dispatch`.
- Behaviour: design § 11 D21 replaces "a provider-stage error is 0". Down (0) is a `not_found` or `timeout` refusal of the probe's own invoke at `dispatch`; a provider 5xx, 401 and 403 read unknown. `probe_provider_unavailable_is_up_0_with_reason` records a 404 and a dispatch timeout.

## Correction 2 (2026-10-15, from the adversary passes)

- A probe invokes only a catalog read: the `operations describe` answer must name profile `generic-http`, and the invoke pins the revision of the description it checked. `connectors.rs` gains `ReadClient::invoke_if` and `ConnectorsError::NotAdmitted`; every other adapter goes through `invoke` unchanged. Design § 11 D21 says so.
