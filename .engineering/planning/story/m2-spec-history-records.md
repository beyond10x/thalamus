---
format: aep.planning-md/3
id: story:m2-spec-history-records
kind: story
status: draft
title: Specify settled transitions, daily rollups, the rolling-median window and the service unmute actor in ESS
relations:
- decomposes: epic:history-and-transitions
- depends_on: epic:signal-contract
- depends_on: story:m1-cli-eval-show
scope:
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/definition.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/rollup.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/transition.rs
- confidence: inferred
  path: crates/thalamus-core/tests/conformance.rs
- confidence: inferred
  path: docs/design/thalamus.md
- confidence: inferred
  path: generated/spec-types
- confidence: inferred
  path: generated/thalamus-model
- confidence: inferred
  path: spec/components.yaml
- confidence: inferred
  path: spec/domains/definition.yaml
- confidence: inferred
  path: spec/domains/rollup.yaml
- confidence: inferred
  path: spec/domains/transition.yaml
- confidence: inferred
  path: spec/ess-inputs.yaml
- confidence: inferred
  path: spec/suite.json
- confidence: inferred
  path: spec/system.yaml
revision: 7
---
## Context

M2 (design § 9) works with two records no `ess/1` document declares and four gaps in the existing specification. `AGENTS.md` (order of work) puts ESS first, so this story lands before any M2 story that uses them. It lands after M1 (`depends_on epic:signal-contract`, and `story:m1-cli-eval-show`, the last M1 code story): it regenerates `generated/thalamus-model`, `generated/spec-types` and `spec/suite.json`, which every M1 story compiles against, and adds behaviour to the conformance harness the M0 stories built.

| what | source | state today |
|---|---|---|
| transition | design § 3.2, § 11 D6 (`spec/domains/evaluation.yaml:329`): "settled transitions are their own record, emitted by hysteresis in M2"; D13 | no entity |
| daily rollup | design § 4: "one row per series per UTC day: `n`, `unknown`, `partial`, `min/p50/max/last`, worst verdict, verdict counts, min/max score"; D14 | no entity |
| persistence | design § 3.2, § 11 D15: `persistence: {k, k_crit, m}` on the definition, defaults 2, 1, 3 | no field |
| the N of `rolling_median` | design § 3.1: "median of the last N samples of this series" | `thalamus.definition.EvaluationSpec` (`spec/domains/definition.yaml:122`) has no field for N |
| the service as unmute actor | design § 11 D9 and D19 (`spec/domains/definition.yaml:241`) | `UnmuteDefinition` is granted only to `thalamus.definition.InstanceOperator` (`spec/domains/definition.yaml:261`) |

## Work

- `spec/domains/transition.yaml`, domain `thalamus.transition`: entity `Transition` (identity `transition_id`, prefix `trn_`; fields `signal`, `series`, `dims`, `definition_rev`, `from_verdict: Optional<Verdict>` (absent on a series' first settled verdict), `to_verdict`, `first_ts` (the first of the agreeing evaluations), `settled_ts` (the settling evaluation `ts`), `settling_score: Optional<Decimal>`, `evaluations` (the ids of the k evaluations that settled it)); relation `evaluations` → `thalamus.evaluation.SignalEvaluation`, `references`, cardinality many, via `evaluations` (D13); relation `definition` → `thalamus.definition.SignalDefinition`, via `signal`; lifecycle `Recorded`; actor `thalamus.transition.Hysteresis`; command `RecordTransition` with outcomes `unknown-signal` and `recorded`; event `TransitionRecorded`; view `SeriesTransitions` (param `series`, `order_by: [settled_ts asc]`). The copied fields are what a reader gets once the evaluations have left raw retention (D13: a missing evaluation reads as expired).
- `spec/domains/rollup.yaml`, domain `thalamus.rollup`: entity `DailyRollup` (identity `rollup_id`; fields `signal`, `series`, `day` (UTC midnight), `n`, `unknown`, `partial`, `min`, `p50`, `max`, `last` as `Optional<Decimal>`, absent when the day has no `ok` reading, never 0; `worst_verdict: Optional<Verdict>`, `verdict_counts` (one count per ladder verdict), `min_score`, `max_score` as `Optional<Decimal>`); invariant `unknown + partial <= n`; command `RecordRollup` with outcomes `unknown-signal`, `already-rolled-up` and `recorded`; event `RollupRecorded`; view `SeriesRollups`. Two rules ESS cannot state are `UNMAPPED:` markers held by Rust tests in `story:m2-rollup`: the rollup id spelling (series key and day), and D14 (a rollup counts each sample's evaluation at the highest `definition_rev` present when it is built).
- `spec/domains/definition.yaml`: `SignalDefinition.persistence: Optional<Persistence>` with `Persistence {k, k_crit, m}` (D15; defaults applied in `thalamus-core`, `story:m2-hysteresis-core`); `EvaluationSpec.rolling_n: Optional<Integer>`; `ProposeDefinition` and `ReviseDefinition` refuse a definition whose baselines contain `rolling_median` without `rolling_n` (outcome `rolling-n-missing`; if ESS cannot select into the struct, an `UNMAPPED:` marker and a Rust test); actor `thalamus.definition.Service`, granted `UnmuteDefinition`, whose event the service records in its store and never in the definition file (D19).
- `spec/system.yaml`, `spec/ess-inputs.yaml` (the two domains), `spec/components.yaml` (`thalamus-service` owns both domains, accepts `RecordTransition` and `RecordRollup`, publishes `TransitionRecorded` and `RollupRecorded`).
- `task generate`; `generated/` and `spec/suite.json` regenerated, not edited.
- `crates/thalamus-core/src/behaviour/transition.rs` and `crates/thalamus-core/src/behaviour/rollup.rs`: the obligations `RecordTransition` and `RecordRollup` the regenerated `PLAN.md` names; `crates/thalamus-core/src/behaviour/definition.rs`: the `rolling-n-missing` outcome of propose and revise; `crates/thalamus-core/tests/conformance.rs`: the new obligations in `every_obligation_is_met`. The count of scenarios is still read from `spec/suite.json`.
- Design § 11: one row per decision taken here.

Decisions taken here (reversible): N is a field of the definition, because `AGENTS.md` keeps "one copy of every threshold, in its definition"; the transition and rollup records are their own domains, so neither changes the digest of `thalamus.evaluation` or `thalamus.observation`.

## Domain relations

| relation | cardinality | ownership | lifecycle | citation |
|---|---|---|---|---|
| `Transition` → `SignalEvaluation`, declared here as `evaluations`, via `evaluations` | many | references | the transition outlives its evaluations; a missing one reads as expired | design § 11 D13 |
| `Transition` → `SignalDefinition`, declared here as `definition`, via `signal` | many-to-one | references | the definition exists first and is never deleted (`Retired` is terminal, `spec/domains/definition.yaml:245`) | a transition is per series, and a series is "a signal id together with its dims values" (`thalamus.observation.SeriesKey`, `spec/domains/observation.yaml:20`, D10); declared as `SignalEvaluation.definition` is (`spec/domains/evaluation.yaml:115`) |
| `DailyRollup` → `SignalDefinition`, declared here as `definition`, via `signal` | many-to-one | references | as above; the rollup outlives its samples (design § 4) and the definition outlives both | same citation |
| `DailyRollup` → `SignalEvaluation` (verdict fields) | none declared | none | the evaluation counted is the one at the highest `definition_rev` present when the rollup is built | design § 11 D14, an `UNMAPPED:` marker held by a Rust test |

## Scope

Planned (inferred):

- `spec/domains/transition.yaml`, `spec/domains/rollup.yaml` (new)
- `spec/domains/definition.yaml`, `spec/system.yaml`, `spec/components.yaml`, `spec/ess-inputs.yaml`
- `generated/thalamus-model`, `generated/spec-types`, `spec/suite.json` (generated)
- `crates/thalamus-core/src/behaviour/transition.rs`, `crates/thalamus-core/src/behaviour/rollup.rs`, `crates/thalamus-core/src/behaviour/definition.rs` (the `rolling-n-missing` outcome; owned by `story:m0-conformance-definitions`, which lands first)
- `crates/thalamus-core/tests/conformance.rs` (owned by `story:m0-conformance-definitions`; the new obligations in `every_obligation_is_met`)
- `docs/design/thalamus.md` (§ 11)

## Acceptance

Each check runs on the changed specification; `IR` stands for `ess specify compile --path spec --format json`.

- `ess specify validate --path spec` reports `thalamus v1 — 9 file(s), valid`.
- `SignalDefinition.persistence`: `IR | jq -e '.entities["thalamus.definition.SignalDefinition"].fields | map(.name) | index("persistence") != null'` exits 0.
- `rolling_n`: `IR | jq -e '.types["thalamus.definition.EvaluationSpec"].body.fields | map(.name) | index("rolling_n") != null'` exits 0.
- The `Transition` → `SignalEvaluation` relation: `IR | jq -e '.entities["thalamus.transition.Transition"].relations | any(.target == "thalamus.evaluation.SignalEvaluation" and .kind == "references" and .cardinality == "many")'` exits 0.
- The D13 copied fields: `IR | jq -e '(.entities["thalamus.transition.Transition"].fields | map(.name)) as $f | ["series", "definition_rev", "from_verdict", "to_verdict", "first_ts", "settled_ts", "settling_score"] - $f == []'` exits 0.
- The D14 rollup verdict fields: `IR | jq -e '(.entities["thalamus.rollup.DailyRollup"].fields | map(.name)) as $f | ["worst_verdict", "verdict_counts", "min_score", "max_score"] - $f == []'` exits 0.
- `task check` exits 0 on the regenerated model, with `cargo test -p thalamus-core --test conformance` passing the scenarios `thalamus.transition.RecordTransition/outcome/recorded`, `thalamus.rollup.RecordRollup/outcome/recorded` and `thalamus.rollup.RecordRollup/outcome/already-rolled-up` and `every_obligation_is_met`.

## Decision applied (2026-10-15, wave 20261015b)

`story:m1-baselines` found that `EvaluationSpec` has no field for the number a `fixed` baseline names (`spec/domains/definition.yaml:122-137`) and passes it as a fifth `fold` argument. This story adds `fixed: Optional<Decimal>` to `EvaluationSpec` (patch drafted at the coordinator's scratch, `evaluation-spec-fixed.patch`), regenerates, and switches `fold` to read it; named test `fixed_baseline_reads_the_definition_number`.
