---
format: aep.planning-md/3
id: story:m2-flap-measurement
kind: story
status: implemented
title: Measure flap rates over stored evaluations before k and m are fixed
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-hysteresis-core
- depends_on: story:m2-scheduler
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/flaps.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/flaps.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T02:11:18Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-17T02:11:18Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-17T03:47:24Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Context

Epic acceptance: "flap rate measured before `k` and `m` are fixed". Design § 3.2: "The defaults are guesses; milestone M2 measures flap rates and fixes them per family." Design § 11 D15 puts `k`, `k_crit` and `m` on each definition. This story builds the measuring command and its tests; running it on one real instance and fixing the values from what it shows is `story:m2-live-observation`. The `flaps` subcommand line is `story:m0-workspace`'s.

## Work

- `crates/thalamus-cli/src/cmd/flaps.rs`: `thalamus flaps --since <duration> [--rev current|<n>] [--k <n> --k-crit <n> --m <n>] [--sweep] [--format text|json]`. It reads the stored evaluations of every series through the store's series range read, keeps those at the selected definition revision (default: the current revision of each definition) and counts the rest as `excluded`. It replays them through `thalamus-core` hysteresis (`story:m2-hysteresis-core`) and reports per family: series, series-days, raw verdict changes (as `thalamus.evaluation.VerdictChanges` defines them, `spec/domains/evaluation.yaml:330`), settled changes, and both per series-day. Without `--k`, `--k-crit`, `--m` each definition's own `persistence` applies. `--sweep` covers `k` in 1 to 3 and `m` in 1 to 4. It writes nothing.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalEvaluation` → `SignalDefinition` (`definition`, via `signal`): family and current revision are read from the definition | many-to-one | references | `spec/domains/evaluation.yaml:115`; `family` at `spec/domains/definition.yaml:177` |

## Scope

Planned (inferred):

- `crates/thalamus-cli/src/cmd/flaps.rs`
- `crates/thalamus-cli/tests/flaps.rs` (new)

## Acceptance

`cargo test -p thalamus-cli --test flaps` passes the failing-first tests `flaps_raw_changes_match_verdict_changes_view`, `flaps_settled_changes_match_hysteresis_replay`, `flaps_excludes_other_revisions`, `flaps_sweep_covers_k_and_m_grid` and `flaps_writes_nothing`.
