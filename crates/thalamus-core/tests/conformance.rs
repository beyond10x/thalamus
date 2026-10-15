//! Runs the selected scenarios of `spec/suite.json` (written by `ess verify conform synthesize`)
//! in process against the generated behaviours over in-memory storage, through
//! `thalamus_core::conformance::run`. Every selected scenario must answer: an unknown step, value
//! kind, field shape or initial state fails the run, and nothing is skipped.
//!
//! The other tests hold the harness itself to failing, so that a green run means something.

use thalamus_core::conformance::{self, Report};
use thalamus_model::json::{self, Value};
use thalamus_model::server::memory::MemoryPorts;

/// The scenarios this suite runs: the definition and dimension domains, whose obligations
/// `thalamus_core::behaviour::definition` and `::dimension` meet. The other domains follow with
/// their own obligations.
const SELECTION: [&str; 2] = ["thalamus.definition.", "thalamus.dimension."];

fn selected(id: &str) -> bool {
    SELECTION.iter().any(|prefix| id.starts_with(prefix))
}

fn suite() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The ids of the suite's scenarios, read here and not by the harness, so the count the run is
/// held to is not the harness's own.
fn scenario_ids(suite: &str) -> Vec<String> {
    let parsed = json::parse(suite).expect("the suite is JSON");
    match parsed.member("scenarios") {
        Some(Value::Object(scenarios)) => scenarios.iter().map(|(id, _)| id.clone()).collect(),
        other => panic!("the suite holds no scenario map: {other:?}"),
    }
}

fn failures(report: &Report) -> String {
    report
        .failed
        .iter()
        .map(|failure| format!("{}: {}", failure.scenario, failure.reason))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_definition_and_dimension_scenario_of_the_suite_holds() {
    let suite = suite();
    let wanted: Vec<String> = scenario_ids(&suite)
        .into_iter()
        .filter(|id| selected(id))
        .collect();
    assert!(!wanted.is_empty(), "the selection names no scenario");

    let report = conformance::run(&suite, &selected, MemoryPorts::default).expect("the suite runs");
    println!(
        "conformance: {} scenario(s) selected, {} ran, {} failed",
        wanted.len(),
        report.ran.len(),
        report.failed.len()
    );
    assert_eq!(
        report.ran, wanted,
        "the run did not answer exactly the selected scenarios"
    );
    assert!(
        report.failed.is_empty(),
        "{} of {} scenarios failed:\n{}",
        report.failed.len(),
        report.ran.len(),
        failures(&report)
    );
}

/// A one-scenario suite around a `DeclareRegistry` call, with `extra` appended to its steps.
fn declare_registry_suite(initial: &str, extra: &str) -> String {
    format!(
        r#"{{"provenance": {{"scenario_initial_state": "{initial}"}},
            "scenarios": {{"probe": {{"purpose": "p", "source": [], "steps": [
              {{"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
                "actor": "thalamus.dimension.InstanceOperator",
                "input": {{"registry_id": {{"kind": "literal", "value": "r1"}},
                          "title": {{"kind": "literal", "value": "t"}}}}}}{extra}
            ]}}}}}}"#
    )
}

fn run_probe(suite: &str) -> Result<Report, String> {
    conformance::run(suite, &|_| true, MemoryPorts::default)
}

fn fails(extra: &str) -> String {
    let report = run_probe(&declare_registry_suite("empty", extra)).expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    match report.failed.as_slice() {
        [failure] => failure.reason.clone(),
        _ => panic!("the probe passed with the steps {extra}"),
    }
}

#[test]
fn the_harness_passes_a_scenario_whose_expectations_hold() {
    let report = run_probe(&declare_registry_suite(
        "empty",
        r#",
        {"step": "expect_outcome", "outcome": {"command": "thalamus.dimension.DeclareRegistry", "outcome": "declared"}},
        {"step": "expect_no_error"},
        {"step": "expect_event_values", "event": "thalamus.dimension.RegistryDeclared",
         "payload": {"registry_id": {"kind": "literal", "value": "r1"}},
         "shape": {"registry_id": {"holds": "primitive", "kind": "string"}}},
        {"step": "query_view", "view": "thalamus.dimension.Registries"},
        {"step": "expect_view", "view": "thalamus.dimension.Registries",
         "expectation": {"expect": "satisfies", "predicate": {"all": ["state == Declared", "defined(title)"]}}}"#,
    ))
    .expect("the probe suite runs");
    assert!(report.failed.is_empty(), "{}", failures(&report));
}

#[test]
fn the_harness_fails_every_expectation_that_does_not_hold() {
    for (rule, extra) in [
        (
            "another outcome",
            r#",{"step": "expect_outcome", "outcome": {"command": "thalamus.dimension.DeclareRegistry", "outcome": "already-declared"}}"#,
        ),
        (
            "an error where none was answered",
            r#",{"step": "expect_error", "error": "thalamus.dimension.RegistryExists"}"#,
        ),
        (
            "no event where one was published",
            r#",{"step": "expect_no_events"}"#,
        ),
        (
            "a payload value it does not carry",
            r#",{"step": "expect_event", "event": "thalamus.dimension.RegistryDeclared", "payload": {"registry_id": "r2"}, "shape": {"registry_id": {"holds": "primitive", "kind": "string"}}}"#,
        ),
        (
            "a payload shape it does not have",
            r#",{"step": "expect_event", "event": "thalamus.dimension.RegistryDeclared", "payload": {}, "shape": {"registry_id": {"holds": "primitive", "kind": "integer"}}}"#,
        ),
        (
            "a row the view does not hold",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "contains", "fields": {"registry_id": {"kind": "literal", "value": "r2"}}}}"#,
        ),
        (
            "a predicate a row breaks",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "satisfies", "predicate": "state != Declared"}}"#,
        ),
        (
            "a refusal where the grant admitted the caller",
            r#",{"step": "expect_not_granted", "actor": "thalamus.dimension.InstanceOperator", "unpublished": []}"#,
        ),
    ] {
        let reason = fails(extra);
        assert!(!reason.is_empty(), "{rule}: failed without a reason");
    }
}

#[test]
fn the_harness_refuses_what_it_does_not_know_rather_than_skipping_it() {
    for (rule, extra) in [
        ("an unknown step", r#",{"step": "expect_the_unexpected"}"#),
        (
            "an unknown member of a known step",
            r#",{"step": "expect_no_error", "within": 3}"#,
        ),
        (
            "an unknown value kind",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "contains", "fields": {"registry_id": {"kind": "guessed"}}}}"#,
        ),
        (
            "an unknown expectation",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "resembles"}}"#,
        ),
        (
            "an unknown field shape",
            r#",{"step": "expect_event", "event": "thalamus.dimension.RegistryDeclared", "payload": {}, "shape": {"registry_id": {"holds": "primitive", "kind": "money"}}}"#,
        ),
        (
            "an unknown predicate form",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "satisfies", "predicate": "title ~ t"}}"#,
        ),
        (
            "a capture naming an entity the model does not declare",
            r#",{"step": "capture_instance", "instance": "i", "entity": "thalamus.dimension.Registry", "event": "thalamus.dimension.RegistryDeclared", "field": "registry_id"}"#,
        ),
        (
            "a capture naming a field its entity does not declare",
            r#",{"step": "capture_instance", "instance": "i", "entity": "thalamus.definition.SignalDefinition", "event": "thalamus.dimension.RegistryDeclared", "field": "registry_id"}"#,
        ),
        (
            "a capture binding a name twice",
            r#",{"step": "capture_instance", "instance": "i", "entity": "thalamus.dimension.DimensionRegistry", "event": "thalamus.dimension.RegistryDeclared", "field": "registry_id"},{"step": "capture_instance", "instance": "i", "entity": "thalamus.dimension.DimensionRegistry", "event": "thalamus.dimension.RegistryDeclared", "field": "registry_id"}"#,
        ),
    ] {
        let reason = fails(extra);
        assert!(!reason.is_empty(), "{rule}: failed without a reason");
    }
    assert!(
        run_probe(&declare_registry_suite("seeded", "")).is_err(),
        "an unknown initial state ran"
    );
}

#[test]
fn an_ungranted_actor_is_refused_before_the_command_runs() {
    let suite = r#"{"provenance": {"scenario_initial_state": "empty"},
        "scenarios": {"probe": {"purpose": "p", "source": [], "steps": [
          {"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
           "actor": "thalamus.definition.InstanceOperator",
           "input": {"registry_id": {"kind": "literal", "value": "r1"},
                     "title": {"kind": "literal", "value": "t"}}},
          {"step": "expect_not_granted", "actor": "thalamus.definition.InstanceOperator",
           "unpublished": ["thalamus.dimension.RegistryDeclared"]},
          {"step": "query_view", "view": "thalamus.dimension.Registries"},
          {"step": "expect_view", "view": "thalamus.dimension.Registries",
           "expectation": {"expect": "excludes", "fields": {}}}
        ]}}}"#;
    let report = run_probe(suite).expect("the probe suite runs");
    assert!(report.failed.is_empty(), "{}", failures(&report));
}
