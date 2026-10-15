---
format: aep.planning-md/3
id: story:m1-connectors-client
kind: story
status: draft
title: One read-only Connectors client, adapter trait and recorded Connectors double for every source
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m0-workspace
scope:
- confidence: inferred
  path: crates/thalamus-sources/Cargo.toml
- confidence: inferred
  path: crates/thalamus-sources/src/adapter.rs
- confidence: inferred
  path: crates/thalamus-sources/src/bin/connectors-double.rs
- confidence: inferred
  path: crates/thalamus-sources/src/connectors.rs
- confidence: inferred
  path: crates/thalamus-sources/src/git.rs
- confidence: inferred
  path: crates/thalamus-sources/src/http_probe.rs
- confidence: inferred
  path: crates/thalamus-sources/src/kubernetes.rs
- confidence: inferred
  path: crates/thalamus-sources/src/loki.rs
- confidence: inferred
  path: crates/thalamus-sources/src/prometheus.rs
- confidence: inferred
  path: crates/thalamus-sources/src/registry.rs
- confidence: inferred
  path: crates/thalamus-sources/src/sql.rs
- confidence: inferred
  path: crates/thalamus-sources/src/vocabulary.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/connectors_client.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/connectors
revision: 3
---
## Outcome

Every thalamus adapter reads a source through one Connectors client that holds no secret and refuses any operation that is not a read, and every adapter test runs against one recorded Connectors double.

## Context

`epic:signal-contract`; design § 5.2 (no secret enters thalamus; read-only against watched systems; "thalamus only invokes operations through Connectors, the pattern cortex uses"). The cortex client is the model: it runs the `connectors` binary with `--output json`, describes the operation for its schema and revision, invokes it, reads a refusal envelope from stderr, revalidates lapsed evidence once and retries a timeout at admission once (inferred from cortex `src/connectors.rs:117`, `:209`, `:244`). The M3 adapters (`story:m3-loki-adapter`, `story:m3-sql-adapter`, `story:m3-kubernetes-adapter`, `story:m3-git-adapter`, `story:m3-http-probe-adapter`) and `story:m1-prometheus-adapter` build on the adapter trait, the extractor registry and the double this story writes. Code is Rust (AGENTS.md).

The crate, its `lib.rs` module lines and its member line in the root `Cargo.toml` are `story:m0-workspace`'s (`depends_on`); this story touches no `members` line and no `lib.rs`. Its `Cargo.lock` change, and the one `story:m1-eventlog-store` makes, are resolved by the coordinator at merge by regenerating the lock.

## Work

- `crates/thalamus-sources/src/connectors.rs`: the client. Binary from `--connectors`, else `THALAMUS_CONNECTORS`, else `connectors` on `PATH`. A refusal becomes a typed error carrying Connectors `code` and `stage`. Each adapter declares its read operations; the client refuses any other operation before running anything.
- `crates/thalamus-sources/src/adapter.rs`: the adapter trait. Given a definition, a window end and a run id, an adapter answers one `Read` per series: `dims`, `reading`, `window`, optional `denominator` (`n` and unit; `since` and `age_s` are left to the pipeline, design § 11 D18), the provenance fields it owns (`adapter`, `connection`, `query_sha256`), `raw` (the producer's own text of the scalar it parsed, when there is one) and `retention_start` (only when the source exposes one). A Connectors failure becomes a `Read` of quality `unknown` whose reason names the code and stage, never an error that drops the series.
- `crates/thalamus-sources/src/registry.rs`: the extractor registry. It chooses an adapter by the definition `extractor` prefix, with one arm for each planned prefix: `promql.`, `loki.`, `sql.`, `kubernetes.`, `git.`, `http_probe.`. Each arm calls `adapter()` in that adapter's own file; this story writes in `prometheus.rs`, `loki.rs`, `sql.rs`, `kubernetes.rs`, `git.rs` and `http_probe.rs` a one-line `adapter()` that answers none, which the registry turns into an `unknown` read with reason `adapter not implemented`, and each adapter story replaces that line in its own file. Every read the registry returns passes through `vocabulary::map_reads(definition, reads)`, which this story writes in `vocabulary.rs` as the identity and `story:m3-vocabulary-mapping` replaces. So no adapter or vocabulary story edits this file.
- `crates/thalamus-sources/src/bin/connectors-double.rs` (a Rust test binary): answers `connections list`, `operations describe` and `operations invoke` from recorded JSON files in a fixture directory named by an environment variable, and appends every argv it received to a log the tests read.
- `crates/thalamus-sources/Cargo.toml`: the dependencies of the client.

## Domain relations

- `thalamus.definition.SignalDefinition.source` → one Connectors adapter instance and one connection. `source.adapter` is the Connectors adapter instance name and `source.connection` the Connectors connection id, as cortex `ConnectorsSource` names them; inferable, inferred from cortex `src/connectors.rs:209` (`invoke(adapter, connection, operation, input)`) and the `ConnectorsSource` struct of the cortex specification. The thalamus type `thalamus.definition.Source` (`spec/domains/definition.yaml:59`) has the same two fields, and its `AdapterId` summary ("the source adapter that turns a connection operation into samples") is read as that adapter instance name; the mapping code is chosen by `extractor`.
- A connection id is never a credential. Inferable: `spec/domains/definition.yaml:49` (`UNMAPPED:`, held by review and a Rust test).
- `thalamus.observation.Provenance` names `adapter` and `connection` (`spec/domains/observation.yaml:110`): "Names a connection, never a credential".

## Scope

Planned (inferred):

- `crates/thalamus-sources/Cargo.toml` (`Cargo.lock` is regenerated by the coordinator at merge)
- `crates/thalamus-sources/src/adapter.rs`, `crates/thalamus-sources/src/connectors.rs`, `crates/thalamus-sources/src/registry.rs`
- `crates/thalamus-sources/src/bin/connectors-double.rs`
- the one-line `adapter()` stubs in `crates/thalamus-sources/src/prometheus.rs`, `crates/thalamus-sources/src/loki.rs`, `crates/thalamus-sources/src/sql.rs`, `crates/thalamus-sources/src/kubernetes.rs`, `crates/thalamus-sources/src/git.rs`, `crates/thalamus-sources/src/http_probe.rs`, and the identity `map_reads` in `crates/thalamus-sources/src/vocabulary.rs`; each file's owning story depends on this one and replaces the stub
- `crates/thalamus-sources/tests/connectors_client.rs`, `crates/thalamus-sources/tests/fixtures/connectors`

## Acceptance

`cargo test -p thalamus-sources --test connectors_client` passes, against the double, the tests `write_operation_is_refused_before_invocation` (written and seen failing first), `argv_and_env_carry_no_credential`, `refusal_on_stderr_is_typed_with_code_and_stage`, `lapsed_evidence_revalidates_once_then_retries`, `admission_timeout_retries_once`, `connectors_failure_becomes_unknown_read_with_code_and_stage`, `registry_chooses_adapter_by_extractor_prefix` and `unimplemented_adapter_reads_unknown_never_zero`.

## Out of scope

Any concrete adapter (`story:m1-prometheus-adapter`; M3 for the others); producer vocabulary mapping (`story:m3-vocabulary-mapping`); creating or repairing Connectors connections (an operator step in Connectors).
