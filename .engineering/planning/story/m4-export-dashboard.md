---
format: aep.planning-md/3
id: story:m4-export-dashboard
kind: story
status: draft
title: Prometheus and Loki definitions export one dashboard JSON
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-export-rules
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/export.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/export_dashboard.rs
- confidence: inferred
  path: crates/thalamus-core/src/export/dashboard.rs
- confidence: inferred
  path: crates/thalamus-core/tests/export_dashboard.rs
- confidence: inferred
  path: crates/thalamus-core/tests/fixtures/export_dashboard
revision: 3
---
## Context

Design § 6.5: "Dashboard JSON ... generated from definitions as an **export**, only for definitions whose source is Prometheus or Loki; a dashboard tool cannot see SQL, orchestrator or version-control signals, so it is never the only view." This story fills `crates/thalamus-core/src/export/dashboard.rs` (stub from `story:m4-skeleton`) over the recorded series names and baseline expressions of `crates/thalamus-core/src/export/expr.rs` (`story:m4-export-rules`, not edited here), and adds the `dashboard` arm to `crates/thalamus-cli/src/cmd/export.rs`, which `story:m4-export-rules` fills first (`depends_on`).

## Work

- `dashboard.rs` (pure): one dashboard per instance in the Grafana dashboard JSON model, schema version pinned by the golden file. One row per layer, one panel per exported `Active` definition: the value target over its recorded series, one target per offset baseline, `fixed` WARN and CRIT lines as threshold steps read from the definition; the panel unit mapped from the UCUM unit (`1` of a ratio as percent unit, `s`, `By`; an annotation as none); one template variable per declared dimension (`label_values` over the recorded value series); the data source a template variable of type `prometheus` with no URL and no host. Definitions that are not exported (`story:m4-export-rules` skip rules) get no panel.
- `crates/thalamus-cli/src/cmd/export.rs`: `thalamus export dashboard [--out <file>]`; stdout by default; exit codes as `export rules`.

## Decisions taken here (decomposer call, reversible)

- The dashboard JSON format is the Grafana model: the dashboard tool that reads both Prometheus and Loki. The design names no tool.
- Panels read the recorded series of the rules export, so the dashboard and the alerts show one expression; the dashboard is empty for a signal until those recording rules run.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `SignalDefinition.source`, Prometheus or Loki by extractor prefix | one per definition | part of the definition | `spec/domains/definition.yaml:59`; inferred from `story:m1-connectors-client` registry arms |
| WARN and CRIT lines of a `fixed` baseline | one `EvaluationSpec` per definition | part of the definition | `spec/domains/definition.yaml:107`, `spec/domains/definition.yaml:122` |
| dims = template variables | keys of the definition | registry owns the keys | `spec/domains/definition.yaml:225` (design § 11 D8); `spec/domains/dimension.yaml:62` |
| layer from the signal id | one per definition | none | `spec/domains/definition.yaml:10` (`UNMAPPED:`, held by a Rust test) |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/export/dashboard.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-cli/src/cmd/export.rs` (the `dashboard` arm; shared with `story:m4-export-rules`, ordered after it)
- `crates/thalamus-core/tests/export_dashboard.rs`, `crates/thalamus-core/tests/fixtures/export_dashboard`, `crates/thalamus-cli/tests/export_dashboard.rs` (new; neutral names only)

## Acceptance

- `cargo test -p thalamus-core --test export_dashboard` passes the failing-first tests `dashboard_known_answer` (golden file `crates/thalamus-core/tests/fixtures/export_dashboard/dashboard.json`), `one_panel_per_exported_definition_grouped_by_layer`, `non_exported_definition_has_no_panel`, `dimension_becomes_template_variable`, `fixed_lines_become_threshold_steps`, `ucum_unit_maps_to_panel_unit` and `datasource_is_a_variable_without_url_or_host`.
- `cargo test -p thalamus-cli --test export_dashboard` passes `export_dashboard_writes_json_that_parses`.
