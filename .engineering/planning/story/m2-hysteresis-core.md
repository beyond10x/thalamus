---
format: aep.planning-md/3
id: story:m2-hysteresis-core
kind: story
status: implemented
title: Settle verdict changes with k/m persistence in thalamus-core
relations:
- decomposes: epic:history-and-transitions
- depends_on: epic:signal-contract
- depends_on: story:m2-spec-history-records
scope:
- confidence: inferred
  path: crates/thalamus-core/src/hysteresis.rs
- confidence: inferred
  path: crates/thalamus-core/tests/hysteresis.rs
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T12:33:40Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-15T12:33:40Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-15T13:32:04Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":7}}}
---
## Context

Design § 3.2: "A verdict change counts as a transition only after `k` consecutive evaluations agree (default `k = 2`, `k = 1` for `CRIT`). A `WARN` clears after `m = 3` `OK` evaluations." Design § 11 D15: `k`, `k_crit` and `m` live on the definition as `persistence: {k, k_crit, m}`, defaults 2, 1, 3; the field is added to the specification and the generated model by `story:m2-spec-history-records` (`depends_on`). `prev` on an evaluation stays the raw previous verdict (design § 11 D6); the settled verdict is computed here, purely, so the service, a replay of stored history and the flap measurement use one implementation (design § 5.1: `thalamus-core` has no I/O and no clock).

## Work

- `crates/thalamus-core/src/hysteresis.rs`: `Persistence { k, k_crit, m }` read from the definition's generated `persistence` field, each value absent falling back to the design default (2, 1, 3); `SeriesHysteresis::new(persistence)`; `SeriesHysteresis::observe(evaluation) -> Option<SettledChange>`, where a change carries what a transition records (design § 11 D13): `from: Option<Verdict>`, `to`, `first_ts` (the first of the agreeing evaluations), `settled_ts` (the settling evaluation `ts`), `settling_score`, `definition_rev` and the ids of the agreeing evaluations.
- Rules, decided here from § 3.2 and the spec verdict ladder (`thalamus.evaluation.Verdict`, `spec/domains/evaluation.yaml`), reversible:
  1. The settled verdict moves to V when the last n evaluations of the series all have verdict V: n = `k_crit` for `CRIT`, n = `m` for `OK`, n = `k` for `WARN`, `UNKNOWN` and `N/A`.
  2. Any differing verdict breaks a run. Time between evaluations does not: the count is of consecutive evaluations, not of cadence slots.
  3. `N/A` and `UNKNOWN` never clear a settled `WARN` or `CRIT`; only `OK` does (the ladder: `N/A` is "Not OK", `UNKNOWN` "Never folds into OK").
  4. The first settled verdict of a series is a change with `from` absent, as `prev` is absent on the first evaluation of a series (`thalamus.evaluation.SignalEvaluation.prev`).
- Recording and emitting a settled change is `story:m2-transitions`; fixing `k`, `k_crit` and `m` from a measured flap rate is `story:m2-live-observation`.

## Domain relations

No relation between entities is read or written. The input is the evaluations of one series in `ts` order, as `thalamus.evaluation.SeriesVerdicts` serves them (`spec/domains/evaluation.yaml:298`); a series is "a signal id together with its dims values" (`thalamus.observation.SeriesKey`, `spec/domains/observation.yaml:20`). The persistence values are fields of `thalamus.definition.SignalDefinition` (design § 11 D15).

## Scope

Planned (inferred):

- `crates/thalamus-core/src/hysteresis.rs` (its module line is `story:m0-workspace`'s)
- `crates/thalamus-core/tests/hysteresis.rs` (new)

## Acceptance

`cargo test -p thalamus-core --test hysteresis` passes the failing-first tests `warn_settles_after_two`, `crit_settles_after_one`, `warn_clears_after_three_ok`, `two_ok_do_not_clear_warn`, `unknown_does_not_clear_warn`, `na_does_not_clear_warn`, `alternating_ok_warn_never_settles`, `first_settled_verdict_has_no_from`, `replay_equals_live`, `persistence_block_overrides_defaults` (a definition with `persistence: {k: 3, k_crit: 2, m: 4}` settles `WARN` after three, `CRIT` after two and clears after four) and `settled_change_carries_d13_fields`.

## Note from story:m2-spec-history-records (2026-10-15)

The specification now declares `persistence`, `evaluation.fixed` and `evaluation.rolling_n` on a definition, but the definition-file loader (`crates/thalamus-core/src/registry.rs`) sets all three to `None`. This story also teaches the loader to read the field(s) it consumes, with the registry's rules (repeated keys refused, values validated), and a failing-first loader test; `website/docs/concepts/signals.md` lists the field.

## Note, superseding the one above (2026-10-15)

The loader now reads `evaluation.fixed`, `evaluation.rolling_n` and `persistence` in story:m2-spec-history-records (its adversary pass 1 found the gap with failing cases); this story no longer edits `crates/thalamus-core/src/registry.rs` for them.

Superseding the per-value fallback above (coordinator decision after review-result:adversary-m2-spec-history-records-pass-2, finding 7): the specification's `Persistence` requires all of `k`, `k_crit` and `m`, and the loader refuses a block missing any of them. A definition without a `persistence` block uses the design defaults 2, 1, 3 (design § 11 D15); a block present carries all three values, so `hysteresis.rs` has no per-value fallback.
