---
format: aep.planning-md/3
id: story:m3-one-vocabulary-across-sources
kind: story
status: draft
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
- confidence: inferred
  path: crates/thalamus-pipeline/tests/every_source_one_ladder.rs
- confidence: inferred
  path: crates/thalamus-pipeline/tests/fixtures/every-source
revision: 3
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
