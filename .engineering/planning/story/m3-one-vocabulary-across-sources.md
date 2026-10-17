---
format: aep.planning-md/3
id: story:m3-one-vocabulary-across-sources
kind: story
status: implemented
title: One pass over every source kind yields only ladder verdicts and the worst verdict's exit
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m3-loki-adapter
- depends_on: story:m3-sql-adapter
- depends_on: story:m3-kubernetes-adapter
- depends_on: story:m3-git-adapter
- depends_on: story:m3-http-probe-adapter
- depends_on: story:m3-vocabulary-mapping
- depends_on: story:m3-advisory-gate-mapping
- depends_on: story:m1-prometheus-adapter
- depends_on: story:m1-cli-eval-show
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/thalamus-pipeline/Cargo.toml
- confidence: inferred
  path: crates/thalamus-pipeline/tests/every_source_one_ladder.rs
- confidence: inferred
  path: crates/thalamus-pipeline/tests/fixtures/every-source
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T21:40:01Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-16T21:40:01Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-17T01:21:54Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Context

The epic acceptance is "one verdict vocabulary across all sources". Each adapter story proves its own reads; this story proves the whole set together: one pass over a fixture instance with one definition per source kind (Prometheus from M1, Loki, SQL, Kubernetes, Git, HTTP probe) and one producer-vocabulary definition (a colour read through the SQL adapter, rule `producer_mapped`), run through the read → sample → grade → evaluate pass of the library crate `thalamus-pipeline` (`story:m1-cli-sample`, `story:m1-cli-eval-show`) against the M1 Connectors test double with recorded answers. The colour reaches the pass through the extractor registry, which passes every read through `vocabulary::map_reads` (`story:m1-connectors-client`, `story:m3-vocabulary-mapping`), and the SQL adapter's `raw` cell text (`story:m3-sql-adapter`); no source file changes here. Code is Rust (AGENTS.md); the fixture instance holds no organization data.

## Behaviour

- Every evaluation of the pass carries one of the five ladder verdicts (`OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`); no adapter emits a verdict of its own vocabulary.
- An `unknown` reading from every source kind grades `UNKNOWN`, never `OK`.
- The pass exits with the ladder exit of its worst verdict (`CRIT > WARN > UNKNOWN > OK = N/A`, design § 2.3): a `CRIT` from SQL beside an `UNKNOWN` from Loki exits 2; `UNKNOWN` beside `OK` exits 3; a pass in which every verdict is `N/A` exits 3.
- Every sample and evaluation the pass records deserialises into the generated serde types of `generated/spec-types` (`thalamus.sample/1`, `thalamus.evaluation/1`) without loss.

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `SignalEvaluation → SignalSample` and `SignalEvaluation → SignalDefinition`, each many-to-one. Inferable: `spec/domains/evaluation.yaml`, entity `thalamus.evaluation.SignalEvaluation`, relations `sample` (via `sample_id`) and `definition` (via `signal`), both references, cardinality one.
- `DimensionRegistry → Dimension`, one-to-many, the registry owns its keys; the fixture instance declares one registry. Inferable: `spec/domains/dimension.yaml`, entity `thalamus.dimension.DimensionRegistry`, relation `dimensions` (owns, cardinality many, via `registry_id`).

## Acceptance

`cargo test -p thalamus-pipeline --test every_source_one_ladder` passes `every_source_kind_yields_only_ladder_verdicts`, `unknown_reading_from_every_source_grades_unknown`, `mixed_pass_exits_with_worst_verdict_exit`, `colour_na_through_sql_adapter_is_unknown` and `every_recorded_sample_and_evaluation_validates_against_generated_types` (each stored line of the pass deserialises into the `generated/spec-types` type of its `schema` and re-serialises to the same JSON).

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-pipeline/tests/every_source_one_ladder.rs` (new)
- `crates/thalamus-pipeline/tests/fixtures/every-source` (new: definitions, dimension registry, recorded Connectors answers)

No source file outside these paths is expected to change; a change one adapter needs belongs to that adapter story. The crate and its member line are `story:m0-workspace`'s.

## Amendment 1 (wave 20261016b)

`generated/spec-types` declares no type for a whole `thalamus.sample/1` or `thalamus.evaluation/1` line, only the `SampleRecorded` and `EvaluationRecorded` event subsets, and `ess generate types` selects types and events, not entities. The generated form of the two entities is their JSON Schema (`website/static/schemas/signal-sample.schema.json`, `signal-evaluation.schema.json`, from `ess generate --kind schema`, drift-checked by `task docs-check`). Decision (reversible): `every_recorded_sample_and_evaluation_validates_against_generated_types` validates each stored line of the pass against the JSON Schema of its entity, with the dev-dependency `boon` in `thalamus-pipeline`; no second record type is declared in `spec/`. Scope adds `crates/thalamus-pipeline/Cargo.toml` (dev-dependencies) and `Cargo.lock`.

## Amendment 2 (wave 20261016b)

A stored line is the body of the `SampleRecorded` or `EvaluationRecorded` event with the store envelope key `schema` (`crates/thalamus-store/src/samples.rs` `encode`), not the entity snapshot: the entity schemas require `state` and refuse extra keys. Both entities have the single state `Recorded` (`spec/domains/observation.yaml:170`), which the store supplies on read (`samples.rs` `snapshot`). Decision (reversible): the test reads each stored line as the snapshot the store serves (the `schema` key taken off, `state: "Recorded"` put on) and validates that against the entity JSON Schema; the stored line format does not change.
