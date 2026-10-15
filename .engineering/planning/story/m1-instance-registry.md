---
format: aep.planning-md/3
id: story:m1-instance-registry
kind: story
status: draft
title: Instance definitions and dimension registry validate, and only Active definitions with registered dims are sampled
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m0-probes
- depends_on: story:m0-conformance-records
scope:
- confidence: inferred
  path: crates/thalamus-core/Cargo.toml
- confidence: inferred
  path: crates/thalamus-core/src/registry.rs
- confidence: inferred
  path: crates/thalamus-core/src/series.rs
- confidence: inferred
  path: crates/thalamus-core/src/since.rs
- confidence: inferred
  path: crates/thalamus-core/tests/fixtures/instance
- confidence: inferred
  path: crates/thalamus-core/tests/registry.rs
revision: 4
---
## Outcome

An instance operator whose definition files or dimension registry break a rule is told which rule at load, a sample is admitted only for an `Active` definition with `dims` keys and values the registry knows, and every sample's `denominator.since` resolves from one of the three origins design § 11 D18 admits.

## Context

`epic:signal-contract` acceptance: "every sample names an `Active` definition". Design § 2.1 (definition invariants), § 2.2 (`denominator.since`, `age_s`), § 2.4 (dimension registry), § 4 (definitions live in the instance definition files), § 5.2 (an instance is a directory), § 11 D8, D10, D17 and D18. `story:m0-conformance-records` meets the `RecordSample` obligation over in-memory ports, and `story:m0-probes` writes `parse_ucum`; this story depends on both. It implements the rules the specification marks `UNMAPPED:` and leaves to M1: the signal id grammar (`spec/domains/definition.yaml:10`), registered dimension keys of a definition (`spec/domains/definition.yaml:225`, `:357`), the UCUM check, and the `dims` of a sample (`spec/domains/observation.yaml:142`). Pure: the caller passes file contents, the first stored sample time and the source's retention start in. Its module files exist after `story:m0-workspace`; the `crates/thalamus-core/src/lib.rs` module lines are that story's.

## Work

- `crates/thalamus-core/src/registry.rs`:
  - parse one definition document (YAML, the design § 2.1 shape: `id`, `lifecycle` and the `SignalDefinition` fields of `spec/domains/definition.yaml:168`, with `revised_at`, `revised_by`, `mute_reason`, `mute_until`) and one dimension registry document (design § 2.4 shape, plus `registry_id`, `title`, and an optional `created` per entry of a `file:` registry value list);
  - validate: the `SignalDefinition` invariants; `Active` or `Muted` declares `unit`, `dimensions`, `denominator`, `owner` (error named after `thalamus.definition.IncompleteDefinition`); the id is `<family>.<layer>.<name>`; `unit` passes `parse_ucum`; kind `ratio` has unit `1`; kind `instant` declares only `fixed`; every key in `dimensions` is registered; an empty `dimensions` list is zero dimensions and an absent one is undeclared (D8); `source.connection` is an identifier, never a URL;
  - D17: a dimension whose registry is `connectors:…` is refused at load; the error names the locator and says that Connectors offers no inventory operation to resolve it. Only `file:` and `open` registries load;
  - D18: `denominator.since` admits `first_sample`, a fixed RFC 3339 timestamp, and `dimension.created` only when the named dimension's registry is `file:`; any other value is refused naming the three admitted forms;
  - every error names the file and the rule.
- `crates/thalamus-core/src/since.rs`: `resolve_since(definition, dims, registry_values, first_stored: Option<Timestamp>, retention_start: Option<Timestamp>) -> Option<Timestamp>`:
  - `first_sample`: the timestamp of the first stored sample of the series (the pipeline passes it from the store); none stored yet resolves to none, so `age_s` is unknown and an offset baseline is `UNKNOWN` (`story:m1-baselines`);
  - a fixed timestamp: that timestamp;
  - `dimension.created`: the `created` of the sample's value in the `file:` registry; a value whose entry carries no `created` resolves to none;
  - when the adapter reported a retention start, the result is never earlier than it, because the source cannot be read before it.
- `crates/thalamus-core/src/series.rs`: the series key of D10, `<signal-id>{k1="v1",k2="v2"}`, keys ascending, values escaped as OpenMetrics label values, the bare id with no dims; and `admit_dims(registry, definition, dims, file_values)`: every key registered and declared by the definition; every value listed in its `file:` registry (contents passed in) or admitted by `open`.
- The generated storage ports for definitions and dimensions (`SignalDefinitionStorage`, `DimensionStorage`, `DimensionRegistryStorage` in `thalamus_model::behaviour`, inferred from `ess generate synthesize --target rust` at ess 0.59.0), served read-only from the loaded instance; a write through them is refused with a named error, because the files are the one copy (design § 4, § 11 D19). The generated `RecordSample` over them answers `unknown-signal` and `not-active` from the files.
- `crates/thalamus-core/Cargo.toml`: a YAML parser (shared with the two M0 stories that add dev-dependencies; this story follows both).

## Domain relations

- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`; the definition exists first and outlives its samples (`RetireDefinition`: "Its samples and evaluations stay"). Inferable: `spec/domains/observation.yaml`, entity `SignalSample`, relation `definition`; outcomes `unknown-signal`, `not-active` of `RecordSample`.
- `thalamus.dimension.DimensionRegistry` → `thalamus.dimension.Dimension`, one-to-many, the registry owns its keys. Inferable: `spec/domains/dimension.yaml`, entity `DimensionRegistry`, relation `dimensions`.
- `thalamus.dimension.Dimension` → `thalamus.dimension.Dimension`, zero or one parent, references via `parent`. Inferable: `spec/domains/dimension.yaml`, entity `Dimension`, relation `parent_dimension`.
- `thalamus.definition.SignalDefinition.dimensions` → `thalamus.dimension.Dimension` keys, many, no ownership. Inferable: `spec/domains/definition.yaml:225` and `:357` (`UNMAPPED:` markers; empty versus absent decided as design § 11 D8).
- `thalamus.observation.SignalSample.dims` → registered keys and resolvable values. Inferable: `spec/domains/observation.yaml:142` (`UNMAPPED:`, held by a Rust test); `file` and `open` registries only, design § 11 D17.
- `thalamus.observation.Denominator.since` → the series origin. Inferable: `spec/domains/observation.yaml`, type `thalamus.observation.Denominator`; its three forms are design § 11 D18.
- A series is a signal id plus its dims values. Inferable: `spec/domains/observation.yaml:17` (decided as design § 11 D10).
- Where a definition lifecycle state is kept: the definition file field `lifecycle`, the only copy (design § 11 D19).

## Scope

Planned (inferred):

- `crates/thalamus-core/src/registry.rs`, `crates/thalamus-core/src/series.rs`, `crates/thalamus-core/src/since.rs`
- `crates/thalamus-core/Cargo.toml` (a YAML parser)
- `crates/thalamus-core/tests/registry.rs`, `crates/thalamus-core/tests/fixtures/instance` (an example instance with neutral names only)

## Acceptance

`cargo test -p thalamus-core --test registry` passes the tests `sample_of_non_active_definition_is_refused` (written and seen failing first; a `Muted`, a `Proposed` and a `Retired` definition each answer `thalamus.observation.SignalNotActive` through the generated `RecordSample` over the file-backed ports), `sample_of_unknown_signal_is_refused`, `unregistered_dims_key_is_refused`, `unlisted_dims_value_is_refused`, `active_definition_without_dimensions_is_invalid`, `empty_dimensions_is_zero_dimensions`, `signal_id_needs_three_segments`, `ratio_unit_must_be_1`, `non_ucum_unit_is_invalid`, `connection_url_is_refused`, `connectors_registry_is_refused_naming_the_missing_inventory_operation` (D17), `since_admits_first_sample_fixed_timestamp_and_file_dimension_created` (D18), `since_dimension_created_on_open_registry_is_refused` (D18), `since_other_value_is_refused_naming_admitted_forms` (D18), `resolve_since_first_sample_is_first_stored_ts`, `resolve_since_fixed_timestamp`, `resolve_since_dimension_created_reads_file_entry`, `resolve_since_never_precedes_retention_start`, `definition_ports_refuse_writes` and `series_key_matches_d10_vectors`.

## Out of scope

Reading files from disk and the instance layout (`story:m1-cli-sample`); definition lifecycle commands and expiry handling (`story:m2-auto-unmute`); `connectors:` registries beyond the refusal (D17: until Connectors offers an inventory operation).

## Addition (2026-10-15, from the m1-baselines adversary pass 2)

Loading a definition also runs `thalamus_core::baseline::plan` and refuses the definition at load when it returns a refusal (for example `PooledNeedsRatio` for a gauge or count that declares two offsets without a combiner), so it never loads Active and fails every pass. Named test: `pooled_on_a_non_ratio_is_refused_at_load`.
