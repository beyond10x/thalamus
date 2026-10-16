---
format: aep.planning-md/3
id: story:m3-advisory-gate-mapping
kind: story
status: implemented
title: An advisory gate's finding count grades WARN above its declared baseline, never CRIT
relations:
- decomposes: epic:source-adapters
- depends_on: story:m3-vocabulary-mapping
scope:
- confidence: cited
  path: crates/thalamus-core/src/behaviour/definition.rs
- confidence: cited
  path: crates/thalamus-core/src/registry.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/mod.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/warn_only.rs
- confidence: cited
  path: crates/thalamus-core/tests/adv_advisory_p1.rs
- confidence: cited
  path: crates/thalamus-core/tests/adv_advisory_p2.rs
- confidence: inferred
  path: crates/thalamus-core/tests/advisory_gate.rs
- confidence: cited
  path: crates/thalamus-core/tests/conformance.rs
- confidence: cited
  path: crates/thalamus-docs/src/status.rs
- confidence: cited
  path: crates/thalamus-sources/tests/adv_host_pass1.rs
- confidence: cited
  path: docs/design/thalamus.md
- confidence: cited
  path: spec/domains/definition.yaml
- confidence: cited
  path: website/docs/concepts/verdicts.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T14:23:57Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T14:23:57Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-16T21:10:13Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Outcome

An advisory gate's result becomes a typed sample and verdict on the one ladder: its finding count is the value, graded against a declared `fixed` baseline (design § 11 D12): above it `WARN`, at or below it `OK`, never `CRIT`; no count printed `UNKNOWN`.

## Context

Design § 2.3: "a gate's non-zero exit → … `WARN` above baseline for an advisory one"; design § 11 D12 decides the shape: kind `count_of_cases`, unit `{finding}`, a `fixed` baseline (`story:m1-baselines` folds it). Verdicts come only from `thalamus-core` `grade()` and its closed rule list (`story:m1-verdict-grading`), so the grading is a rule, `fixed_warn_only`, in `thalamus-core`. The count itself is an ordinary `ok` reading from whichever adapter reads the gate's output; a gate that printed no count is an `unknown` reading, which grading rule 2 already makes `UNKNOWN`.

The rule is one more variant and dispatch arm in `crates/thalamus-core/src/rules/mod.rs` (owned by `story:m1-verdict-grading`), after the one `story:m3-vocabulary-mapping` adds; this story depends on it for that shared line.

## Work

- `crates/thalamus-core/src/rules/warn_only.rs`: `fixed_warn_only`: against the `fixed` baseline, a count above it is `WARN` with a reason naming the count and the baseline, at or below it `OK`; never `CRIT`. A definition with this rule that declares a `crit` line, or a baseline other than `fixed`, is refused at load. Score per design § 3.3: 0 at the baseline, 1 at the first count above it, never above 1.
- `crates/thalamus-core/src/rules/mod.rs`: one variant `FixedWarnOnly` and one dispatch arm.

## Acceptance

`cargo test -p thalamus-core --test advisory_gate` passes these named tests, each red before the change:

- `advisory_count_above_baseline_is_warn`
- `advisory_count_at_baseline_is_ok`
- `advisory_never_grades_crit` (a count one hundred times the baseline is `WARN`)
- `advisory_missing_count_is_unknown_not_zero`
- `advisory_crit_line_is_refused_at_load`
- `advisory_non_count_kind_is_refused_at_load`
- `advisory_non_count_kind_is_refused_on_propose_and_revise`
- `advisory_non_finding_unit_is_refused_at_load`
- `advisory_non_finding_unit_is_refused_on_propose_and_revise`

and `cargo test -p thalamus-core --test adv_advisory_p1 --test adv_advisory_p2` passes the adversaries' cases.

## Domain relations

- `SignalSample` → `SignalDefinition` (`spec/domains/observation.yaml`, relation `definition`): the sample names the advisory gate's definition, kind `count_of_cases`.
- `SignalEvaluation` → `SignalSample` (`spec/domains/evaluation.yaml`, relation `sample`).

## Scope

Cited from the unit's working tree (adversary pass 1, F6: the "refused at load" acceptance needs the loader and the specified outcome):

- `crates/thalamus-core/src/rules/warn_only.rs`
- `crates/thalamus-core/src/rules/mod.rs` (one variant, one arm; shared with `story:m3-vocabulary-mapping`, which landed first)
- `crates/thalamus-core/tests/advisory_gate.rs` (new), `crates/thalamus-core/tests/adv_advisory_p1.rs` (new, the adversary's cases)
- `crates/thalamus-core/src/behaviour/definition.rs` and `crates/thalamus-core/src/registry.rs`: the one `input_refusals` that `ProposeDefinition`, `ReviseDefinition` and the loader share (design § 11 D29)
- `crates/thalamus-core/tests/conformance.rs`: the conformance cases of the new outcomes
- `spec/domains/definition.yaml`: the outcomes `advisory-not-fixed` and, after correction 1, the kind refusal; with the regenerated `generated/`, `spec/suite.json` and `website/` output
- `crates/thalamus-docs/src/status.rs`: the capability item
- `crates/thalamus-core/tests/adv_advisory_p2.rs` (new, the pass-2 adversary's cases)
- `crates/thalamus-sources/tests/adv_host_pass1.rs`: its one call of `input_refusals` passes the unit
- `docs/design/thalamus.md`: § 11 row D45
- `website/docs/concepts/verdicts.md`: the refusals of the rule

## Revision 6 (adversary pass 1)

- The rule is restricted to kind `count_of_cases` (F7, design § 11 D12): a definition under `fixed_warn_only` with another kind is refused alike by propose, revise and the loader.
- The verdict and the score are decided exactly on normalised decimals at any magnitude (F1, F2): trailing fractional zeros do not change a grade, and the score at the baseline is exactly 0.

## Revision 8 (adversary pass 2)

- The verdict and the score are computed on the decimal digits as written at any magnitude, with no fixed-width integer and no float (adversary pass 2, P2-F1, P2-F2): a count above a baseline of 10^-38, or a count of 10^39 above 3, is `WARN`. Only a count or baseline that is not a decimal is `UNKNOWN`, with a reason.
- The rule is restricted to unit `{finding}` (P2-F4, design § 11 D12): a declared other unit is refused as `advisory-not-finding-unit` alike by propose, revise and the loader, through the one `input_refusals`, which now takes the unit; the outcome is declared right after `advisory-not-count`.
- The score's rounding is held by cases with a fractional digit (P2-F3).
- Design § 11 row D45 records the rule.
