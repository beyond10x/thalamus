---
format: aep.planning-md/3
id: story:m3-advisory-gate-mapping
kind: story
status: draft
title: An advisory gate's finding count grades WARN above its declared baseline, never CRIT
relations:
- decomposes: epic:source-adapters
- depends_on: story:m3-vocabulary-mapping
scope:
- confidence: inferred
  path: crates/thalamus-core/src/rules/mod.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/warn_only.rs
- confidence: inferred
  path: crates/thalamus-core/tests/advisory_gate.rs
revision: 3
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

## Domain relations

- `SignalSample` → `SignalDefinition` (`spec/domains/observation.yaml`, relation `definition`): the sample names the advisory gate's definition, kind `count_of_cases`.
- `SignalEvaluation` → `SignalSample` (`spec/domains/evaluation.yaml`, relation `sample`).

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-core/src/rules/warn_only.rs`
- `crates/thalamus-core/src/rules/mod.rs` (one variant, one arm; shared with `story:m3-vocabulary-mapping`, which lands first)
- `crates/thalamus-core/tests/advisory_gate.rs` (new)
