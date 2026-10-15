---
format: aep.planning-md/3
id: story:m3-vocabulary-mapping
kind: story
status: active
title: Colour ladders, detector exits and blocking gate exits map onto the one verdict ladder
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-verdict-grading
- depends_on: story:m1-connectors-client
- depends_on: story:m2-spec-history-records
scope:
- confidence: inferred
  path: crates/thalamus-core/src/rules/mapped.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/mod.rs
- confidence: inferred
  path: crates/thalamus-core/tests/producer_mapped.rs
- confidence: inferred
  path: crates/thalamus-sources/src/vocabulary.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/vocabulary.json
- confidence: inferred
  path: crates/thalamus-sources/tests/vocabulary.rs
- confidence: inferred
  path: generated/spec-types
- confidence: inferred
  path: generated/thalamus-model
- confidence: inferred
  path: spec/domains/definition.yaml
- confidence: inferred
  path: spec/suite.json
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T12:33:41Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T12:33:41Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Context

Design § 1 L1: the prototype had four verdict vocabularies (`OK/WARN/CRIT`, colours, detector exit codes, gate exit codes). L3: "not read" ranked the same as "green", so an overall verdict depended on key order. Design § 2.3: "Producers that already speak another vocabulary are mapped at the adapter: colour ladders `green/yellow/red/na` → `OK/WARN/CRIT/UNKNOWN`; detector exit `1` → `CRIT`, `2` → `UNKNOWN`; a gate's non-zero exit → `CRIT` for a blocking gate, `WARN` above baseline for an advisory one."

Verdicts come only from `thalamus-core` `grade()` and its closed rule list (`story:m1-verdict-grading`), so the mapping has two halves with one seam, both owned here:

- In `thalamus-core`, a rule `producer_mapped`: the verdict a producer supplied in its own vocabulary, mapped by a table in the definition. It is one more variant and dispatch arm in the rule list of `crates/thalamus-core/src/rules/mod.rs` (owned by `story:m1-verdict-grading`; `story:m3-advisory-gate-mapping` adds the next one after this story).
- In `thalamus-sources`, the extractor wrapper that applies that table to any adapter's raw value. `story:m1-connectors-client` passes every read of the extractor registry through `vocabulary::map_reads` (the identity until now) and gives each `Read` the producer's `raw` text, so no adapter file and no registry line changes here.

The table needs a home in the definition, so this story adds one field to the specification first (`AGENTS.md`, ESS first). It follows `story:m2-spec-history-records` (`depends_on`), which changes the same file and regenerates the same output.

## Behaviour

The default tables, each written into a definition's `evaluation.mapping`, never into code:

| producer value | verdict | sample reading |
|---|---|---|
| colour `green` / `yellow` / `red` | `OK` / `WARN` / `CRIT` | `ok`, value 0 / 1 / 2 |
| colour `na` | `UNKNOWN` | `unknown`, reason names the producer value; no value |
| detector exit `1` | `CRIT` | `ok`, value 2 |
| detector exit `2` | `UNKNOWN` | `unknown`, reason |
| blocking gate exit non-zero | `CRIT` | `ok`, value 2 |
| detector or gate exit `0` | `OK` | `ok`, value 0 (inferred from the ladder exit column, design § 2.3: `OK` exits 0) |
| any value the table does not list | `UNKNOWN` | `unknown`, reason (AGENTS.md: unknown is never 0 and never OK) |

- A mapped `ok` reading's value is the verdict's position on the score scale of design § 3.3 (0 at `OK`, 1 at `WARN`, 2 at `CRIT`), unit `{verdict}`; `producer_mapped` grades it back to that verdict with that score, so the colour and the verdict cannot disagree.
- `na` maps to `UNKNOWN`, not to `N/A`: § 2.3 names it, and `N/A` means "read, too little to grade". Every mapped verdict other than `OK` carries a reason, and `UNKNOWN` carries no score (spec `thalamus.evaluation.SignalEvaluation` invariants).
- A definition with rule `producer_mapped` and no `evaluation.mapping`, or a mapping naming anything other than a ladder verdict, is refused at load.

## Work

- `spec/domains/definition.yaml`: `EvaluationSpec.mapping: Optional<Map<String, String>>`, producer value → ladder verdict name (the verdict names held by the load-time refusal and a Rust test if ESS cannot type a map value across domains); `task generate` regenerates `generated/thalamus-model`, `generated/spec-types` and `spec/suite.json`.
- `crates/thalamus-core/src/rules/mapped.rs`: the rule `producer_mapped`, and its load-time check of `evaluation.mapping`.
- `crates/thalamus-core/src/rules/mod.rs` (owned by `story:m1-verdict-grading`): one variant `ProducerMapped` and one dispatch arm.
- `crates/thalamus-sources/src/vocabulary.rs`: `map_reads(definition, reads)`; for a definition whose rule is `producer_mapped`, each read's `raw` is looked up in `evaluation.mapping` and the reading replaced as the table above says; for any other rule the reads pass unchanged.

## Domain relations

- `SignalEvaluation → SignalSample`, many-to-one; the evaluation references the sample it grades, which exists first. Inferable: `spec/domains/evaluation.yaml`, entity `thalamus.evaluation.SignalEvaluation`, relation `sample` (references, cardinality one, via `sample_id`).
- `SignalEvaluation → SignalDefinition`, many-to-one. Inferable: same file and entity, relation `definition` (references, cardinality one, via `signal`). The mapping table is a field of that definition, its only copy (L2).
- `SignalSample → SignalDefinition`, many-to-one. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition`.
- The ladder itself (exit, rank, scored per verdict) is the enum `thalamus.evaluation.Verdict` in `spec/domains/evaluation.yaml`: `UNKNOWN` rank 1, `OK` rank 0.

## Acceptance

- `ess specify compile --path spec --format json | jq -e '.types["thalamus.definition.EvaluationSpec"].body.fields | map(.name) | index("mapping") != null'` exits 0, and `task check` exits 0 on the regenerated model.
- `cargo test -p thalamus-core --test producer_mapped` passes `mapped_reading_grades_to_its_verdict_and_score`, `unknown_mapped_reading_grades_unknown_without_score` and `producer_mapped_without_mapping_is_refused_at_load`.
- `cargo test -p thalamus-sources --test vocabulary` passes over the known-answer table `crates/thalamus-sources/tests/fixtures/vocabulary.json`, with `colour_na_never_ranks_as_green` (worst-of `[green, na]` and `[na, green]` are both `UNKNOWN`, never `OK`) seen failing first against a stub that maps `na` like `green`, beside `colour_ladder_maps_to_ok_warn_crit_unknown`, `detector_exit_1_is_crit_and_2_is_unknown`, `blocking_gate_nonzero_exit_is_crit`, `unrecognised_producer_value_is_unknown_never_ok`, `other_rules_pass_reads_unchanged` and `mapped_non_ok_verdict_carries_reason_and_unknown_no_score` (the rules behind conformance scenarios `thalamus.evaluation.RecordEvaluation/outcome/reason-missing` and `thalamus.evaluation.RecordEvaluation/outcome/score-not-admitted`).

## Scope

All paths `inferred` (no code exists yet):

- `spec/domains/definition.yaml`; `generated/thalamus-model`, `generated/spec-types`, `spec/suite.json` (generated)
- `crates/thalamus-core/src/rules/mapped.rs`; `crates/thalamus-core/src/rules/mod.rs` (one variant, one arm)
- `crates/thalamus-core/tests/producer_mapped.rs` (new)
- `crates/thalamus-sources/src/vocabulary.rs` (replaces the identity stub of `story:m1-connectors-client`)
- `crates/thalamus-sources/tests/vocabulary.rs`, `crates/thalamus-sources/tests/fixtures/vocabulary.json` (new)

No `lib.rs` module line: those are `story:m0-workspace`'s.

## Not in this story

The advisory gate (design § 11 D12), graded on its finding count by the rule `fixed_warn_only` of `story:m3-advisory-gate-mapping`. The prototype producers migrating as an instance (design § 10 question 7) stay outside this repository.
