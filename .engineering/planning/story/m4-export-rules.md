---
format: aep.planning-md/3
id: story:m4-export-rules
kind: story
status: draft
title: Prometheus and Loki definitions export recording and alert rules
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-skeleton
- depends_on: story:m1-prometheus-adapter
- depends_on: story:m3-loki-adapter
- depends_on: story:m1-cli-sample
- depends_on: story:m1-verdict-grading
- depends_on: story:m2-hysteresis-core
- depends_on: story:m2-spec-history-records
- depends_on: story:m3-vocabulary-mapping
- depends_on: story:m3-advisory-gate-mapping
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/export.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/export_rules.rs
- confidence: inferred
  path: crates/thalamus-core/src/export/expr.rs
- confidence: inferred
  path: crates/thalamus-core/src/export/rules.rs
- confidence: inferred
  path: crates/thalamus-core/tests/export_rules.rs
- confidence: inferred
  path: crates/thalamus-core/tests/fixtures/export
revision: 4
---
## Context

Design § 6.5: "Prometheus recording and alert rules are generated from definitions as an **export**, only for definitions whose source is Prometheus or Loki". § 7: adopt OpenMetrics naming for export (`snake_case`, unit suffix, `_total`), labels = dimensions, and the `for:` semantics of § 3.2; do not adopt "rules written by hand beside the definitions", because of one copy of every threshold (L2). A definition reads Prometheus when its `extractor` starts `promql.` and Loki when it starts `loki.`: the extractor prefix chooses the mapping code (`story:m1-connectors-client`, registry arms), and the query texts and their keys (`value`, `numerator`, `denominator`) are those `story:m1-prometheus-adapter` and `story:m3-loki-adapter` run. The persistence values and their defaults are `Persistence` of `story:m2-hysteresis-core`; the threshold rule ids come from `story:m1-verdict-grading`. The two non-threshold rule ids the skip list names exist only once their stories land: `producer_mapped` (`story:m3-vocabulary-mapping`) and `fixed_warn_only` (`story:m3-advisory-gate-mapping`); this story `depends_on` both, because `non_threshold_rule_is_skipped_with_reason` reads both ids.

This story fills `crates/thalamus-core/src/export/expr.rs`, `crates/thalamus-core/src/export/rules.rs` and the CLI stub `crates/thalamus-cli/src/cmd/export.rs` (all from `story:m4-skeleton`). `story:m4-export-dashboard` reuses `expr.rs` without editing it and adds the `dashboard` arm to `cmd/export.rs` after this story.

## Work

- `expr.rs` (pure): per exported definition, the recorded series names `thalamus:<signal id, dots as underscores>:<part>` for each query part (`value`, or `numerator` and `denominator`), and for a ratio the value expression over the recorded parts. The value name carries the OpenMetrics unit suffix: unit `1` of a `ratio` gives `_ratio`, `s` gives `_seconds`, `By` gives `_bytes`, a `counter_increase` gives `_total`, a UCUM annotation such as `{request}` gives none. Baseline expressions over the recorded series with `offset`: `offset_1d`, `offset_7d`, `pooled` (sum of numerators over sum of denominators), `higher_of` (`max`), `lower_of` (`min`), `fixed` (the number in the definition).
- `rules.rs` (pure): `render(definitions, target) -> Export { yaml, skipped }`.
  - `--target prometheus`: one rule group per family; recording rules for every `promql.` definition; for every `promql.` and `loki.` definition whose rule is a threshold rule and whose baselines are all expressible above, two alert rules, `ThalamusWarn` and `ThalamusCrit`, labels `severity` (`warn`, `crit`) and `signal`, the expression the WARN or CRIT line of the definition over the baseline expression, guarded by `min_n` on the denominator and `min_events` on the event count where declared, and `for:` = `k` times the cadence (`k_crit` for CRIT).
  - `--target loki`: recording rules for every `loki.` definition, in the rule-group format the Loki ruler reads, LogQL as written in the definition.
  - Only `Active` definitions. Skipped, each with a reason: a definition whose source is neither Prometheus nor Loki; a store baseline (`same_slot_median_4w`, `rolling_median`); a rule other than a threshold rule (`producer_mapped`, `fixed_warn_only`).
- `crates/thalamus-cli/src/cmd/export.rs`: `thalamus export rules --target prometheus|loki [--out <file>]` loads the instance (`story:m1-cli-sample`), writes the YAML (stdout by default) and the skipped list with reasons on stderr; exit 0; refusals use the codes of `story:m1-cli-sample`.

## Decisions taken here (decomposer call, reversible)

- Baselines are expressed over recorded series with `offset`, because `offset` attaches to a selector and cannot be applied to an arbitrary query text. An offset baseline therefore has no value until the recording rules have run for that offset, so an exported alert stays silent, never firing on a missing baseline: the unknown-is-not-OK stance of § 3.1.
- Loki definitions record in the Loki ruler; their alerts run in Prometheus over the recorded series, so one alert file holds every exported threshold.
- `for:` = `k` times the cadence approximates `k` consecutive evaluations (§ 3.2); Prometheus evaluates on its own interval.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalDefinition.source` → one Connectors adapter instance and connection; the extractor prefix chooses Prometheus or Loki | one per definition | part of the definition | `thalamus.definition.Source`, `spec/domains/definition.yaml:59`; inferred from the registry arms recorded in `story:m1-connectors-client` |
| WARN and CRIT lines, baselines | one `EvaluationSpec` per definition | part of the definition | `spec/domains/definition.yaml:107`, `spec/domains/definition.yaml:122`; baseline kinds `thalamus.definition.BaselineKind`, `spec/domains/definition.yaml:95` |
| dims = labels | keys of the definition | registry owns the keys | `SignalDefinition.dimensions`, `spec/domains/definition.yaml:225` (design § 11 D8); `DimensionRegistry` → `Dimension`, `spec/domains/dimension.yaml:62` |
| `persistence: {k, k_crit, m}` | one per definition | part of the definition | design § 11 D15, declared by `story:m2-spec-history-records` |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/export/expr.rs`, `crates/thalamus-core/src/export/rules.rs` (stubs from `story:m4-skeleton`)
- `crates/thalamus-cli/src/cmd/export.rs` (stub from `story:m4-skeleton`; shared with `story:m4-export-dashboard`, which lands after this story)
- `crates/thalamus-core/tests/export_rules.rs`, `crates/thalamus-core/tests/fixtures/export`, `crates/thalamus-cli/tests/export_rules.rs` (new; neutral names only)

## Acceptance

- `cargo test -p thalamus-core --test export_rules` passes the failing-first tests `ratio_vs_pooled_baseline_known_answer` (the design § 2.1 definition against the golden file `crates/thalamus-core/tests/fixtures/export/ratio_pooled.rules.yaml`), `recorded_names_follow_openmetrics_naming`, `for_is_k_times_cadence_and_k_crit_for_crit`, `higher_of_and_lower_of_are_max_and_min_of_offsets`, `fixed_baseline_alert_compares_a_constant`, `min_n_and_min_events_guard_the_alert`, `store_baseline_is_skipped_with_reason`, `non_threshold_rule_is_skipped_with_reason`, `only_promql_and_loki_definitions_are_exported`, `loki_definition_records_in_loki_rules_and_alerts_in_prometheus_rules`, `only_active_definitions_are_exported` and `threshold_change_in_definition_changes_the_export`.
- `cargo test -p thalamus-cli --test export_rules` passes `export_rules_writes_rule_groups_and_lists_skipped_on_stderr` and `exported_rules_parse_as_rule_groups`.
