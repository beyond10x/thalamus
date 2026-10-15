---
format: aep.planning-md/3
id: story:m4-sparklines
kind: story
status: draft
title: Sparklines with value line, baseline band and WARN and CRIT lines computed from the definition
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-heatmap
- depends_on: story:m1-verdict-grading
- depends_on: story:m3-vocabulary-mapping
- depends_on: story:m3-advisory-gate-mapping
scope:
- confidence: inferred
  path: crates/thalamus-core/src/lines.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/threshold.rs
- confidence: inferred
  path: crates/thalamus-core/tests/lines.rs
- confidence: inferred
  path: crates/thalamus-service/src/views/sparklines.rs
- confidence: inferred
  path: crates/thalamus-service/tests/sparklines.rs
revision: 4
---
## Context

Design § 6.5, sparklines: "value line, baseline band, WARN and CRIT lines; gaps drawn as gaps", data: samples and evaluations. § 5.3: "A gap in sampling is recorded as a gap and drawn as one, never interpolated"; § 11 D16: a missed slot is a sample with quality `unknown` and reason `missed_slot`. § 1 L2 and `AGENTS.md`: one copy of every threshold, in its definition, so the WARN and CRIT lines are computed from the definition by the same code that grades, never typed again.

The line computation lives in the threshold rule of `story:m1-verdict-grading` (`crates/thalamus-core/src/rules/threshold.rs`: "a line is the stricter of its `x` and `abs` parts"). This story makes that computation public there, a visibility change and, if the grader computes the line inline, its extraction into one function the grader then calls; nothing in the grading result changes. That file is named in both bodies and this story lands after `story:m1-verdict-grading`. It adds `crates/thalamus-core/src/lines.rs` (stub from `story:m4-skeleton`), which dispatches on the rule id parsed by `crates/thalamus-core/src/rules/mod.rs` without editing it, and fills `crates/thalamus-service/src/views/sparklines.rs`. It reuses `svg.rs` of `story:m4-heatmap`.

Its HTTP tests use the in-process technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `crates/thalamus-core/src/rules/threshold.rs` (owned by `story:m1-verdict-grading`): the WARN and CRIT line values for a threshold and a baseline value made `pub`.
- `crates/thalamus-core/src/lines.rs`: `lines(definition, baseline) -> Lines { warn, crit }`, each optional: the threshold rules give both from the definition `evaluation.warn` and `evaluation.crit`; the advisory rule `fixed_warn_only` of `story:m3-advisory-gate-mapping` gives a WARN line at the first count above its fixed baseline and no CRIT line (design § 11 D12); the rule `producer_mapped` of `story:m3-vocabulary-mapping` gives none; an `unknown` baseline gives none.
- `GET /views/sparklines?signal=<id>&days=<n>&format=html|json`: one sparkline per series of the signal, default 7 days:
  - the value line through `ok` readings; it breaks at every `unknown` sample (a missed slot included) and at every slot with no sample; a `partial` reading is a bound marker (a caret on its `bound` side), not a point of the line;
  - the baseline band from the recorded baseline of each evaluation up to its WARN line; no band where the baseline is unknown;
  - WARN and CRIT lines from `lines`, drawn only at points whose evaluation `definition_rev` equals the current revision of the definition (the definition file holds only the current one, design § 11 D19); elsewhere no line.

## Decisions taken here (decomposer call, reversible)

- The baseline band spans from the baseline to the WARN line, the inside-bounds zone.
- Lines only where the evaluation was graded at the current revision, instead of drawing current thresholds over history graded at older ones.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalSample` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/observation.yaml:161` |
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id`: pairs each value with its baseline | many-to-one | references | `spec/domains/evaluation.yaml:110` |
| `SignalEvaluation` → `SignalDefinition` at `definition_rev` | many-to-one | references | `spec/domains/evaluation.yaml:115` |
| recorded baseline: a closed kind, its offsets and a `Reading` | one per evaluation | part of the evaluation | `thalamus.evaluation.Baseline`, `spec/domains/evaluation.yaml:57` (design § 11 D4) |
| WARN and CRIT lines read from the definition | one `Threshold` each | part of the definition | `thalamus.definition.Threshold` and `EvaluationSpec`, `spec/domains/definition.yaml:107`, `spec/domains/definition.yaml:122` |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/lines.rs` (stub from `story:m4-skeleton`), `crates/thalamus-core/tests/lines.rs` (new)
- `crates/thalamus-core/src/rules/threshold.rs` (shared with `story:m1-verdict-grading`; visibility only; ordered after it)
- `crates/thalamus-service/src/views/sparklines.rs` (stub from `story:m4-skeleton`), `crates/thalamus-service/tests/sparklines.rs` (new)

## Acceptance

- `cargo test -p thalamus-core --test lines` passes the failing-first tests `value_on_warn_line_grades_score_1_and_crit_line_score_2` (for every threshold rule, through `grade`), `line_is_the_stricter_of_x_and_abs`, `fixed_warn_only_has_warn_line_above_fixed_baseline_and_no_crit`, `producer_mapped_has_no_lines` and `unknown_baseline_has_no_lines`.
- `cargo test -p thalamus-service --test sparklines` passes `missing_slot_is_a_gap_in_the_value_line`, `unknown_sample_breaks_the_line_never_drawn_at_zero`, `partial_reading_is_a_bound_marker_not_a_point`, `baseline_band_from_recorded_baseline`, `lines_only_where_evaluation_rev_is_current` and `one_sparkline_per_series_of_the_signal`.
- `cargo test -p thalamus-core --test grading` still passes unchanged.
