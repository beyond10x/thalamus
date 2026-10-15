//! Adversary pass 2: the corrected complete-subject snapshot (`SubjectShape`, the one-row rule,
//! the preceding-query rule) on unions, enums, newtypes, optionals and nested structs, a view
//! serving duplicate rows, and the leniencies the correction chose.
//!
//! Each shape below is the one `spec/suite.json` carries for the view, changed in one place.

use thalamus_core::conformance::{self, Report};
use thalamus_model::assertion::{AssertionId, AssertionSnapshot};
use thalamus_model::behaviour::{
    AssertionStorage, DailyRollupStorage, DimensionRegistryStorage, DimensionStorage,
    SignalDefinitionStorage, SignalEvaluationStorage, SignalSampleStorage, TransitionStorage,
};
use thalamus_model::definition::{SignalDefinitionSnapshot, SignalId};
use thalamus_model::dimension::{
    DimensionKey, DimensionRegistrySnapshot, DimensionSnapshot, RegistryId,
};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationSnapshot};
use thalamus_model::observation::{SampleId, SignalSampleSnapshot};
use thalamus_model::rollup::{DailyRollupSnapshot, RollupId};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_model::transition::{TransitionId, TransitionSnapshot};

/// The `thalamus.dimension.Dimensions` subject shape of the suite.
const DIMENSION_SHAPE: &str = r#"{"identity_field":"key","fields":[{"name":"key","type":"thalamus.dimension.DimensionKey"},{"name":"registry_id","type":"thalamus.dimension.RegistryId"},{"name":"values","type":"thalamus.dimension.ValueSource"},{"name":"rule","type":"thalamus.dimension.DimensionRule"},{"name":"parent","type":"Optional<thalamus.dimension.DimensionKey>"},{"name":"otel_attribute","type":"Optional<String>"},{"name":"state","type":"thalamus.dimension.Dimension.State"}],"declarations":{"thalamus.dimension.Dimension.State":{"kind":"enum","variants":["Registered"]},"thalamus.dimension.DimensionKey":{"kind":"newtype","of":"String"},"thalamus.dimension.DimensionRule":{"kind":"enum","variants":["source_only","derived"]},"thalamus.dimension.Locator":{"kind":"newtype","of":"String"},"thalamus.dimension.RegistryId":{"kind":"newtype","of":"String"},"thalamus.dimension.ValueSource":{"kind":"union","tag":"registry","variants":{"connectors":"thalamus.dimension.Locator","file":"thalamus.dimension.Locator","open":null}}}}"#;

/// The `thalamus.definition.Definitions` subject shape of the suite.
const DEFINITION_SHAPE: &str = r#"{"identity_field":"signal_id","fields":[{"name":"signal_id","type":"thalamus.definition.SignalId"},{"name":"title","type":"String"},{"name":"family","type":"thalamus.definition.Family"},{"name":"kind","type":"thalamus.definition.SignalKind"},{"name":"unit","type":"Optional<thalamus.definition.Unit>"},{"name":"dimensions","type":"Optional<List<thalamus.dimension.DimensionKey>>"},{"name":"source","type":"thalamus.definition.Source"},{"name":"extractor","type":"thalamus.definition.ExtractorId"},{"name":"query","type":"Map<String, thalamus.definition.QueryText>"},{"name":"window","type":"Duration"},{"name":"cadence","type":"Duration"},{"name":"history","type":"thalamus.definition.History"},{"name":"denominator","type":"Optional<thalamus.definition.DenominatorSpec>"},{"name":"objective","type":"Optional<Json>"},{"name":"owner","type":"Optional<thalamus.definition.Owner>"},{"name":"links","type":"thalamus.definition.Links"},{"name":"rev","type":"Integer"},{"name":"revised_at","type":"Timestamp"},{"name":"revised_by","type":"thalamus.definition.Author"},{"name":"mute_reason","type":"Optional<String>"},{"name":"mute_until","type":"Optional<Timestamp>"},{"name":"state","type":"thalamus.definition.SignalDefinition.State"}],"declarations":{"thalamus.definition.AdapterId":{"kind":"newtype","of":"String"},"thalamus.definition.Author":{"kind":"newtype","of":"String"},"thalamus.definition.ConnectionId":{"kind":"newtype","of":"String"},"thalamus.definition.DenominatorSpec":{"kind":"struct","fields":[{"name":"unit","type":"thalamus.definition.Unit"},{"name":"min_n","type":"Integer"},{"name":"since","type":"String"}]},"thalamus.definition.ExtractorId":{"kind":"newtype","of":"String"},"thalamus.definition.Family":{"kind":"newtype","of":"String"},"thalamus.definition.History":{"kind":"enum","variants":["source","store"]},"thalamus.definition.Links":{"kind":"struct","fields":[{"name":"components","type":"List<String>"},{"name":"issues","type":"List<String>"}]},"thalamus.definition.Owner":{"kind":"newtype","of":"String"},"thalamus.definition.QueryText":{"kind":"newtype","of":"String"},"thalamus.definition.SignalDefinition.State":{"kind":"enum","variants":["Active","Muted","Proposed","Retired"]},"thalamus.definition.SignalId":{"kind":"newtype","of":"String"},"thalamus.definition.SignalKind":{"kind":"enum","variants":["gauge","counter_increase","ratio","instant","count_of_cases"]},"thalamus.definition.Source":{"kind":"struct","fields":[{"name":"adapter","type":"thalamus.definition.AdapterId"},{"name":"connection","type":"thalamus.definition.ConnectionId"}]},"thalamus.definition.Unit":{"kind":"newtype","of":"String"},"thalamus.dimension.DimensionKey":{"kind":"newtype","of":"String"}}}"#;

fn suite() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn probe(steps: &[String]) -> String {
    format!(
        r#"{{"provenance": {{"scenario_initial_state": "empty"}},
            "scenarios": {{"probe": {{"purpose": "p", "source": [], "steps": [{}]}}}}}}"#,
        steps.join(",")
    )
}

/// The one reason the one-scenario probe failed with, or `None` where it passed.
fn probe_failure(steps: &[String]) -> Option<String> {
    let report = conformance::run(&probe(steps), &|_| true, MemoryPorts::default)
        .expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    report.failed.first().map(|failure| failure.reason.clone())
}

fn lit(value: &str) -> String {
    format!(r#"{{"kind": "literal", "value": {value}}}"#)
}

fn replaced(shape: &str, from: &str, to: &str) -> String {
    assert!(shape.contains(from), "the shape carries no {from}");
    shape.replacen(from, to, 1)
}

// ---------------------------------------------------------------------------------------------
// Dimensions.
// ---------------------------------------------------------------------------------------------

/// A registry `r1` and one dimension `k1` in it, with `values` and `otel_attribute` as given,
/// then a fresh query of `Dimensions` and a snapshot of `k1` under `shape`.
fn dimension_snapshot(values: &str, shape: &str) -> Vec<String> {
    vec![
        format!(
            r#"{{"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
                "actor": "thalamus.dimension.InstanceOperator",
                "input": {{"registry_id": {}, "title": {}}}}}"#,
            lit("\"r1\""),
            lit("\"t\"")
        ),
        format!(
            r#"{{"step": "execute_command", "command": "thalamus.dimension.RegisterDimension",
                "actor": "thalamus.dimension.InstanceOperator",
                "input": {{"key": {}, "registry_id": {}, "rule": {}, "values": {},
                          "otel_attribute": {}}}}}"#,
            lit("\"k1\""),
            lit("\"r1\""),
            lit("\"source_only\""),
            lit(values),
            lit("\"service.name\"")
        ),
        r#"{"step": "expect_outcome", "outcome": {"command": "thalamus.dimension.RegisterDimension", "outcome": "registered"}}"#.to_owned(),
        r#"{"step": "query_view", "view": "thalamus.dimension.Dimensions"}"#.to_owned(),
        format!(
            r#"{{"step": "snapshot_complete_subject", "view": "thalamus.dimension.Dimensions",
                "subject": {{"key": {}}}, "shape": {shape}}}"#,
            lit("\"k1\"")
        ),
    ]
}

fn assert_refused(steps: &[String], naming: &str) {
    match probe_failure(steps) {
        Some(reason) => assert!(
            reason.contains(naming),
            "refused, but not for {naming}: {reason}"
        ),
        None => panic!("a subject row breaking its declared {naming} passed"),
    }
}

#[test]
fn the_suite_dimension_shape_admits_its_own_row() {
    for values in [
        r#"{"registry": "open"}"#,
        r#"{"registry": "file", "value": "registries/x.json"}"#,
    ] {
        assert_eq!(
            probe_failure(&dimension_snapshot(values, DIMENSION_SHAPE)),
            None,
            "{values}"
        );
    }
}

#[test]
fn a_unit_union_variant_served_with_a_value_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""file":"thalamus.dimension.Locator""#,
        r#""file":null"#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "file", "value": "x"}"#, &shape),
        "values",
    );
}

#[test]
fn a_carrying_union_variant_served_without_its_value_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""open":null"#,
        r#""open":"thalamus.dimension.Locator""#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "values",
    );
}

#[test]
fn an_undeclared_union_label_is_refused() {
    let shape = replaced(DIMENSION_SHAPE, r#","open":null"#, "");
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "values",
    );
}

#[test]
fn an_undeclared_enum_variant_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""variants":["source_only","derived"]"#,
        r#""variants":["derived"]"#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "rule",
    );
}

#[test]
fn a_newtype_over_another_primitive_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""thalamus.dimension.RegistryId":{"kind":"newtype","of":"String"}"#,
        r#""thalamus.dimension.RegistryId":{"kind":"newtype","of":"Integer"}"#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "registry_id",
    );
}

#[test]
fn an_optional_holding_another_type_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""type":"Optional<String>""#,
        r#""type":"Optional<Integer>""#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "otel_attribute",
    );
}

#[test]
fn a_required_field_served_absent_or_null_is_refused() {
    let shape = replaced(
        DIMENSION_SHAPE,
        r#""type":"Optional<thalamus.dimension.DimensionKey>""#,
        r#""type":"thalamus.dimension.DimensionKey""#,
    );
    assert_refused(
        &dimension_snapshot(r#"{"registry": "open"}"#, &shape),
        "parent",
    );
}

// ---------------------------------------------------------------------------------------------
// Definitions.
// ---------------------------------------------------------------------------------------------

fn propose_and_snapshot(shape: &str) -> Vec<String> {
    let body = format!(
        r#"{{"author": {}, "cadence": {}, "denominator": {}, "dimensions": {}, "evaluation": {},
            "extractor": {}, "family": {}, "history": {}, "kind": {}, "links": {},
            "objective": {}, "owner": {}, "query": {}, "signal_id": {}, "source": {},
            "title": {}, "unit": {}, "window": {}}}"#,
        lit("\"author\""),
        lit("\"PT1S\""),
        lit(r#"{"min_n": 1.0, "since": "denominator.since", "unit": "denominator.unit"}"#),
        lit(r#"["dimensionsj0"]"#),
        lit(
            r#"{"baselines": [], "crit": {"abs": 2.0, "min_events": 1.0, "x": 2.0},
                "min_trend_window": "PT1S", "rule": "evaluation.rule",
                "warn": {"abs": 1.0, "min_events": 1.0, "x": 1.0}}"#
        ),
        lit("\"extractor\""),
        lit("\"family\""),
        lit("\"source\""),
        lit("\"gauge\""),
        lit(r#"{"components": ["c"], "issues": ["i"]}"#),
        lit(r#"{"objective": "objective"}"#),
        lit("\"owner\""),
        lit(r#"{"query": "query.0"}"#),
        lit("\"sig_n\""),
        lit(r#"{"adapter": "sourcejadapter", "connection": "source.connection"}"#),
        lit("\"title\""),
        lit("\"unit\""),
        lit("\"PT1S\""),
    );
    vec![
        format!(
            r#"{{"step": "execute_command", "command": "thalamus.definition.ProposeDefinition",
                "actor": "thalamus.definition.InstanceOperator", "input": {body}}}"#
        ),
        r#"{"step": "expect_outcome", "outcome": {"command": "thalamus.definition.ProposeDefinition", "outcome": "proposed"}}"#.to_owned(),
        r#"{"step": "query_view", "view": "thalamus.definition.Definitions"}"#.to_owned(),
        format!(
            r#"{{"step": "snapshot_complete_subject", "view": "thalamus.definition.Definitions",
                "subject": {{"signal_id": {}}}, "shape": {shape}}}"#,
            lit("\"sig_n\"")
        ),
    ]
}

#[test]
fn the_suite_definition_shape_admits_its_own_row() {
    assert_eq!(probe_failure(&propose_and_snapshot(DEFINITION_SHAPE)), None);
}

#[test]
fn a_nested_struct_member_of_another_type_is_refused() {
    let shape = replaced(
        DEFINITION_SHAPE,
        r#"{"name":"min_n","type":"Integer"}"#,
        r#"{"name":"min_n","type":"String"}"#,
    );
    assert_refused(&propose_and_snapshot(&shape), "denominator");
}

#[test]
fn a_nested_struct_lacking_a_declared_member_is_refused() {
    let shape = replaced(
        DEFINITION_SHAPE,
        r#"{"name":"connection","type":"thalamus.definition.ConnectionId"}"#,
        r#"{"name":"connection","type":"thalamus.definition.ConnectionId"},{"name":"region","type":"String"}"#,
    );
    assert_refused(&propose_and_snapshot(&shape), "source");
}

/// D11: a `Decimal` crosses the thalamus wire as a decimal string; the OpenAPI document declares
/// every `Decimal` `type: string, format: decimal`. A row serving a JSON number where the shape
/// declares a `Decimal` is not that wire form.
#[test]
fn a_decimal_served_as_a_json_number_is_refused() {
    let shape = replaced(
        DEFINITION_SHAPE,
        r#"{"name":"rev","type":"Integer"}"#,
        r#"{"name":"rev","type":"Decimal"}"#,
    );
    assert_refused(&propose_and_snapshot(&shape), "rev");
}

// ---------------------------------------------------------------------------------------------
// The preceding-query rule and the one-row rule.
// ---------------------------------------------------------------------------------------------

/// The reference runner refuses `expect_view` unless the view last read is the one it names,
/// and a command clears that read (`runtime.ts` `expectView`, ESS-CF-SUITE): rows read before a
/// command say nothing about the state after it.
#[test]
fn an_expectation_over_rows_read_before_the_last_command_fails() {
    let failure = probe_failure(&[
        r#"{"step": "query_view", "view": "thalamus.dimension.Registries"}"#.to_owned(),
        format!(
            r#"{{"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
                "actor": "thalamus.dimension.InstanceOperator",
                "input": {{"registry_id": {}, "title": {}}}}}"#,
            lit("\"r1\""),
            lit("\"t\"")
        ),
        r#"{"step": "expect_outcome", "outcome": {"command": "thalamus.dimension.DeclareRegistry", "outcome": "declared"}}"#.to_owned(),
        format!(
            r#"{{"step": "expect_view", "view": "thalamus.dimension.Registries",
                "expectation": {{"expect": "excludes", "fields": {{"registry_id": {}}}}}}}"#,
            lit("\"r1\"")
        ),
    ]);
    assert!(
        failure.is_some(),
        "excludes r1 held over rows read before r1 was declared"
    );
}

/// Storage whose `list` serves every definition and dimension twice.
struct Doubled(MemoryPorts);

macro_rules! plain {
    ($port:ident, $identity:ty, $snapshot:ty) => {
        impl $port for Doubled {
            fn get(&self, identity: &$identity) -> Option<$snapshot> {
                $port::get(&self.0, identity)
            }
            fn put(&mut self, snapshot: $snapshot) {
                $port::put(&mut self.0, snapshot);
            }
            fn delete(&mut self, identity: &$identity) {
                $port::delete(&mut self.0, identity);
            }
            fn list(&self) -> Vec<$snapshot> {
                $port::list(&self.0)
            }
        }
    };
}

macro_rules! doubled {
    ($port:ident, $identity:ty, $snapshot:ty) => {
        impl $port for Doubled {
            fn get(&self, identity: &$identity) -> Option<$snapshot> {
                $port::get(&self.0, identity)
            }
            fn put(&mut self, snapshot: $snapshot) {
                $port::put(&mut self.0, snapshot);
            }
            fn delete(&mut self, identity: &$identity) {
                $port::delete(&mut self.0, identity);
            }
            fn list(&self) -> Vec<$snapshot> {
                let once = $port::list(&self.0);
                once.iter().cloned().chain(once.iter().cloned()).collect()
            }
        }
    };
}

plain!(AssertionStorage, AssertionId, AssertionSnapshot);
plain!(
    DimensionRegistryStorage,
    RegistryId,
    DimensionRegistrySnapshot
);
plain!(
    SignalEvaluationStorage,
    EvaluationId,
    SignalEvaluationSnapshot
);
plain!(SignalSampleStorage, SampleId, SignalSampleSnapshot);
plain!(TransitionStorage, TransitionId, TransitionSnapshot);
plain!(DailyRollupStorage, RollupId, DailyRollupSnapshot);
doubled!(SignalDefinitionStorage, SignalId, SignalDefinitionSnapshot);
doubled!(DimensionStorage, DimensionKey, DimensionSnapshot);

fn doubled_run(scenarios: &[&str]) -> Report {
    let report = conformance::run(&suite(), &|id| scenarios.contains(&id), || {
        Doubled(MemoryPorts::default())
    })
    .expect("the suite runs");
    assert_eq!(report.ran.len(), scenarios.len());
    report
}

/// A view serving the subject twice fails the one-row rule on the real "already" scenarios.
#[test]
fn a_view_serving_the_subject_twice_fails_the_snapshot() {
    let scenarios = [
        "thalamus.definition.ProposeDefinition/outcome/already-defined",
        "thalamus.dimension.RegisterDimension/outcome/already-registered",
    ];
    let report = doubled_run(&scenarios);
    for scenario in scenarios {
        let failure = report
            .failed
            .iter()
            .find(|failure| failure.scenario == scenario)
            .unwrap_or_else(|| panic!("{scenario} passed over a view serving its subject twice"));
        assert!(
            failure.reason.contains("matched 2 rows"),
            "{scenario} failed for another reason: {}",
            failure.reason
        );
    }
}
