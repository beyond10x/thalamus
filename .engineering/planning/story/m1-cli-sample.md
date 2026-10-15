---
format: aep.planning-md/3
id: story:m1-cli-sample
kind: story
status: draft
title: thalamus sample records one sample per series of every Active definition of an instance
relations:
- decomposes: epic:signal-contract
- depends_on: story:m1-instance-registry
- depends_on: story:m1-eventlog-store
- depends_on: story:m1-prometheus-adapter
- depends_on: story:m1-verdict-grading
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/sample.rs
- confidence: inferred
  path: crates/thalamus-cli/src/global.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/instance
- confidence: inferred
  path: crates/thalamus-cli/tests/sample.rs
- confidence: inferred
  path: crates/thalamus-pipeline/Cargo.toml
- confidence: inferred
  path: crates/thalamus-pipeline/src/instance.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/sample.rs
revision: 3
---
## Outcome

An operator runs `thalamus sample <signal-id>` or `thalamus sample --all --once` against an instance directory and gets one recorded sample per series of every `Active` definition, unknown where the source could not be read.

## Context

`epic:signal-contract`: "`thalamus sample`". Design § 5.2 (an instance is a directory: definitions, dimension registry, store location), § 5.3 (the CLI samples one definition on demand; a one-shot pass is `thalamus sample --all --once`), § 2.2 (provenance, run id, `denominator.since`), § 11 D18. `story:m0-workspace` names the CLI package `thalamus-cli` with binary `thalamus`, declares the subcommand enum and the stub files, and adds the library crate `thalamus-pipeline`, which holds the read → sample → grade → evaluate pass the CLI and the service (`story:m2-scheduler`) share. This story writes the read → sample half of it; `story:m1-cli-eval-show` writes the grade → evaluate half. It is where the M1 crates meet: instance validation (`story:m1-instance-registry`), reads (`story:m1-prometheus-adapter`), the store (`story:m1-eventlog-store`).

## Work

- `crates/thalamus-pipeline/src/instance.rs`: instance layout `<instance>/definitions/*.yaml`, `<instance>/dimensions.yaml`, registry value files under `<instance>/` as their `file:` locators name. Loading reads the files and hands their contents to `thalamus-core`; every refusal names the file and the rule. The grading rule of each definition is parsed at load too (`story:m1-verdict-grading`), so an unknown rule id is refused before any read.
- `crates/thalamus-pipeline/src/sample.rs`: one pass over a set of definitions with one `run_` id; per read from the extractor registry: `admit_dims`, the series key, `denominator.since` from `resolve_since` with the first stored sample of the series and the read's `retention_start` (`story:m1-instance-registry`), `age_s` from it, a `smp_` id, provenance (`producer`, `version`, `definition_rev`, run id), then the generated `RecordSample` over the file-backed definition ports and the store sample port.
- `crates/thalamus-pipeline/Cargo.toml`: the dependencies of the pass (id generation).
- `crates/thalamus-cli/src/global.rs`: the global options `--instance <dir>` (else `THALAMUS_INSTANCE`), `--store <locator>` (else `THALAMUS_STORE`, else `sqlite:<instance>/.thalamus/store.sqlite`), `--connectors <bin>`.
- `crates/thalamus-cli/src/cmd/sample.rs`: `sample <signal-id>` and `sample --all --once` over the pipeline; prints the recorded sample ids (`--format json|text`). Exit codes: 0 when every read was recorded, unknown ones included. Refusals never use a ladder code (0 to 3, design § 2.3): an invalid instance 65, a refused sample 65, an unreachable store 74; a usage error exits 64 from `story:m0-workspace`'s `main.rs`.

## Domain relations

- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`; only an `Active` definition is sampled. Inferable: `spec/domains/observation.yaml`, relation `definition`, outcomes `unknown-signal`, `not-active` of `RecordSample`.
- `thalamus.observation.Provenance.run_id` groups the samples of one pass. Inferable: `spec/domains/observation.yaml`, type `thalamus.observation.RunId` ("Groups the samples of one sampling pass").
- `dims` admission and the series key. Inferable: `spec/domains/observation.yaml:142` and `:17` (design § 11 D10), applied through `story:m1-instance-registry`.
- Definitions and the dimension registry are read from the instance files. Inferable, inferred from `docs/design/thalamus.md` § 4 (row "definitions") and § 5.2; no `ess/1` document declares the instance layout.

## Decisions taken (reversible)

- The instance layout and the default store location above. If the instance later gets a configuration file, it is an ESS type first (the cortex `cortex.instance` precedent), not a story-level format.
- Refusal exit codes 64, 65 and 74 (sysexits), kept off the ladder.

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/Cargo.toml`, `crates/thalamus-pipeline/src/instance.rs`, `crates/thalamus-pipeline/src/sample.rs`
- `crates/thalamus-cli/src/global.rs`, `crates/thalamus-cli/src/cmd/sample.rs`
- `crates/thalamus-cli/tests/sample.rs`, `crates/thalamus-cli/tests/fixtures/instance` (neutral names only)

No `members` line, `lib.rs` module line or subcommand enum line: those are `story:m0-workspace`'s.

## Acceptance

`cargo test -p thalamus-cli --test sample` passes, running the built `thalamus` binary against a fixture instance and the Connectors double, the tests `sample_all_once_records_one_sample_per_series`, `unreadable_source_records_unknown_without_value` (written and seen failing first; the stored line has `"quality": "unknown"` and no `value` key), `muted_definition_is_refused_and_nothing_is_recorded`, `invalid_instance_exits_65_naming_file_and_rule` and `since_first_sample_is_the_first_stored_sample`.

## Out of scope

Grading and `show` (`story:m1-cli-eval-show`); the scheduler (M2); `define validate` and the other commands of design § 5.1.
