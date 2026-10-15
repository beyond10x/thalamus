//! Runs every scenario of `spec/suite.json` (written by `ess verify conform synthesize`) in
//! process against the generated behaviours over in-memory storage, through
//! `thalamus_core::conformance::run`. Every scenario must answer: an unknown step, value kind,
//! field shape or initial state fails the run, and nothing is skipped.
//!
//! `every_obligation_is_met` calls each obligation of the synthesis plan once, so an obligation
//! the suite stops reaching still fails here. The other tests hold the harness itself to failing,
//! so that a green run means something.

use std::collections::BTreeMap;

use thalamus_core::conformance::{self, Ports, Report};
use thalamus_model::assertion::obligations::AskAssertionBehavior;
use thalamus_model::assertion::{AskAssertion, AskAssertionOutcome, Asker, AskerKind, Within};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::{
    ActivateDefinitionBehavior, ProposeDefinitionBehavior, ReviseDefinitionBehavior,
};
use thalamus_model::definition::{
    ActivateDefinition, ActivateDefinitionOutcome, AdapterId, Author, ConnectionId,
    DenominatorSpec, EvaluationSpec, ExtractorId, Family, History, Links, Owner, ProposeDefinition,
    ProposeDefinitionOutcome, QueryText, ReviseDefinition, ReviseDefinitionOutcome, RuleId,
    SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::obligations::{DeclareRegistryBehavior, RegisterDimensionBehavior};
use thalamus_model::dimension::{
    DeclareRegistry, DimensionKey, DimensionRule, DimensionValue, RegisterDimension,
    RegisterDimensionOutcome, RegistryId, ValueSource,
};
use thalamus_model::evaluation::obligations::{RecordEvaluationBehavior, SeriesVerdictsQuery};
use thalamus_model::evaluation::{RecordEvaluation, RecordEvaluationOutcome, Verdict};
use thalamus_model::json::{self, Value};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::obligations::{RecordSampleBehavior, SeriesSamplesQuery};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RecordSample, RecordSampleOutcome, RunId,
    SeriesKey, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

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
fn every_scenario_of_the_suite_holds() {
    let suite = suite();
    let wanted = scenario_ids(&suite);
    assert!(!wanted.is_empty(), "the suite names no scenario");

    let report = conformance::run(&suite, &|_| true, MemoryPorts::default).expect("the suite runs");
    println!(
        "conformance: {} scenario(s) in the suite, {} ran, {} failed",
        wanted.len(),
        report.ran.len(),
        report.failed.len()
    );
    assert_eq!(
        report.ran, wanted,
        "the run did not answer exactly the suite's scenarios"
    );
    assert!(
        report.failed.is_empty(),
        "{} of {} scenarios failed:\n{}",
        report.failed.len(),
        report.ran.len(),
        failures(&report)
    );
}

// ---------------------------------------------------------------------------------------------
// Every obligation of the synthesis plan, called once.
// ---------------------------------------------------------------------------------------------

fn text<T>(wrap: fn(String) -> T, value: &str) -> T {
    wrap(value.to_owned())
}

fn evaluation_spec() -> EvaluationSpec {
    EvaluationSpec {
        rule: text(RuleId, "threshold"),
        baselines: Vec::new(),
        warn: None,
        crit: None,
        min_trend_window: None,
    }
}

fn source() -> Source {
    Source {
        adapter: text(AdapterId, "sql"),
        connection: text(ConnectionId, "replica"),
    }
}

fn denominator() -> DenominatorSpec {
    DenominatorSpec {
        unit: text(Unit, "{request}"),
        min_n: 20,
        since: "first_sample".to_owned(),
    }
}

fn query() -> BTreeMap<String, QueryText> {
    BTreeMap::from([("ratio".to_owned(), text(QueryText, "select 1"))])
}

fn links() -> Links {
    Links {
        components: Vec::new(),
        issues: Vec::new(),
    }
}

fn propose(signal: &str) -> ProposeDefinition {
    ProposeDefinition {
        signal_id: text(SignalId, signal),
        title: "Failure ratio".to_owned(),
        family: text(Family, "platform"),
        kind: SignalKind::Ratio,
        unit: Some(text(Unit, "1")),
        dimensions: Some(vec![text(DimensionKey, "cluster")]),
        source: source(),
        extractor: text(ExtractorId, "sql.ratio"),
        query: query(),
        window: text(Duration, "PT6H"),
        cadence: text(Duration, "PT1H"),
        history: History::Source,
        denominator: Some(denominator()),
        evaluation: evaluation_spec(),
        objective: None,
        owner: Some(text(Owner, "team")),
        links: links(),
        author: text(Author, "author"),
    }
}

fn revise(signal: &str) -> ReviseDefinition {
    ReviseDefinition {
        signal_id: text(SignalId, signal),
        title: "Failure ratio, revised".to_owned(),
        family: text(Family, "platform"),
        kind: SignalKind::Ratio,
        unit: text(Unit, "1"),
        dimensions: vec![text(DimensionKey, "cluster")],
        source: source(),
        extractor: text(ExtractorId, "sql.ratio"),
        query: query(),
        window: text(Duration, "PT6H"),
        cadence: text(Duration, "PT1H"),
        history: History::Source,
        denominator: denominator(),
        evaluation: evaluation_spec(),
        objective: None,
        owner: text(Owner, "team"),
        links: links(),
        author: text(Author, "reviser"),
    }
}

fn record_sample(signal: &str, series: &str) -> RecordSample {
    let at = |instant: &str| text(Timestamp, instant);
    RecordSample {
        signal: text(SignalId, signal),
        series: text(SeriesKey, series),
        dims: BTreeMap::from([("cluster".to_owned(), text(DimensionValue, "a"))]),
        ts: at("2026-01-01T00:00:00Z"),
        window: TimeWindow {
            start: at("2025-12-31T18:00:00Z"),
            end: at("2026-01-01T00:00:00Z"),
        },
        reading: Reading::Ok(CompleteReading {
            value: text(Decimal, "0.029"),
        }),
        unit: text(Unit, "1"),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".to_owned(),
            version: "0.1.0".to_owned(),
            definition_rev: 2,
            adapter: text(AdapterId, "sql"),
            connection: text(ConnectionId, "replica"),
            query_sha256: text(QueryDigest, "00ff"),
            run_id: text(RunId, "run_1"),
        },
    }
}

/// Each of the eight obligations of `generated/thalamus-model/PLAN.md`, called once through the
/// ports the conformance harness runs over, in an order that reaches each accepting outcome. An
/// obligation answering [`UnmetObligation`] fails the test, and every one is reported, not only
/// the first.
#[test]
fn every_obligation_is_met() {
    let mut generated = Generated::new(Ports::new(
        MemoryPorts::default(),
        Some(text(Timestamp, "2026-01-01T00:00:00Z")),
    ));
    let mut unmet: Vec<UnmetObligation> = Vec::new();
    let mut answered: Vec<String> = Vec::new();
    let mut note = |name: &str, answer: Result<bool, UnmetObligation>| match answer {
        Ok(true) => {}
        Ok(false) => answered.push(name.to_owned()),
        Err(refusal) => unmet.push(refusal),
    };

    DeclareRegistryBehavior::declare_registry(
        &mut generated,
        DeclareRegistry {
            registry_id: text(RegistryId, "instance"),
            title: "Instance".to_owned(),
        },
    )
    .expect("the generated DeclareRegistry answers");
    note(
        "thalamus.dimension.RegisterDimension",
        RegisterDimensionBehavior::register_dimension(
            &mut generated,
            RegisterDimension {
                key: text(DimensionKey, "cluster"),
                registry_id: text(RegistryId, "instance"),
                values: ValueSource::Open,
                rule: DimensionRule::SourceOnly,
                parent: None,
                otel_attribute: None,
            },
        )
        .map(|outcome| matches!(outcome, RegisterDimensionOutcome::Registered { .. })),
    );
    note(
        "thalamus.definition.ProposeDefinition",
        ProposeDefinitionBehavior::propose_definition(&mut generated, propose("platform.ratio"))
            .map(|outcome| matches!(outcome, ProposeDefinitionOutcome::Proposed { .. })),
    );
    note(
        "thalamus.definition.ReviseDefinition",
        ReviseDefinitionBehavior::revise_definition(&mut generated, revise("platform.ratio"))
            .map(|outcome| matches!(outcome, ReviseDefinitionOutcome::Revised { .. })),
    );
    let activated = ActivateDefinitionBehavior::activate_definition(
        &mut generated,
        ActivateDefinition {
            signal_id: text(SignalId, "platform.ratio"),
        },
    )
    .expect("the generated ActivateDefinition answers");
    assert!(
        matches!(activated, ActivateDefinitionOutcome::Activated { .. }),
        "the definition was not activated: {activated:?}"
    );

    let series = r#"platform.ratio{cluster="a"}"#;
    let sample = RecordSampleBehavior::record_sample(
        &mut generated,
        record_sample("platform.ratio", series),
    );
    let sample_id = match &sample {
        Ok(RecordSampleOutcome::Recorded { response, .. }) => Some(response.sample_id.clone()),
        _ => None,
    };
    note(
        "thalamus.observation.RecordSample",
        sample.map(|_| sample_id.is_some()),
    );
    note(
        "thalamus.observation.SeriesSamples",
        SeriesSamplesQuery::series_samples(&generated, text(SeriesKey, series))
            .map(|rows| rows.len() == 1 && sample_id.as_ref() == Some(&rows[0].sample_id)),
    );
    let evaluation = RecordEvaluationBehavior::record_evaluation(
        &mut generated,
        RecordEvaluation {
            sample_id: sample_id
                .unwrap_or_else(|| text(thalamus_model::observation::SampleId, "smp_none")),
            definition_rev: 2,
            verdict: Verdict::WARN,
            prev: Some(Verdict::OK),
            score: Some(text(Decimal, "1.3")),
            baseline: None,
            reason: Some("0.029 >= 2 x 0.013".to_owned()),
        },
    );
    let evaluation_id = match &evaluation {
        Ok(RecordEvaluationOutcome::Recorded { response, .. }) => {
            Some(response.evaluation_id.clone())
        }
        _ => None,
    };
    note(
        "thalamus.evaluation.RecordEvaluation",
        evaluation.map(|_| evaluation_id.is_some()),
    );
    note(
        "thalamus.evaluation.SeriesVerdicts",
        SeriesVerdictsQuery::series_verdicts(&generated, text(SeriesKey, series)).map(|rows| {
            rows.len() == 1
                && evaluation_id.as_ref() == Some(&rows[0].evaluation_id)
                && rows[0].verdict == Verdict::WARN
        }),
    );
    note(
        "thalamus.assertion.AskAssertion",
        AskAssertionBehavior::ask_assertion(
            &mut generated,
            AskAssertion {
                signals: vec![text(SignalId, "platform.ratio")],
                dims: BTreeMap::new(),
                after: text(Timestamp, "2026-01-01T00:00:00Z"),
                horizon: text(Duration, "PT1H"),
                within: Within::Baseline,
                asker: Asker {
                    kind: AskerKind::Release,
                    r#ref: "v1".to_owned(),
                },
            },
        )
        .map(|outcome| matches!(outcome, AskAssertionOutcome::Asked { .. })),
    );

    assert!(
        unmet.is_empty(),
        "{} obligation(s) answered UnmetObligation: {unmet:?}",
        unmet.len()
    );
    assert!(
        answered.is_empty(),
        "these obligations met their contract but not with the accepting outcome or rows this \
         sequence reaches: {answered:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The harness held to failing.
// ---------------------------------------------------------------------------------------------

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

/// A one-scenario suite around an `AskAssertion` call that answers `asked`, with `extra`
/// appended to its steps.
fn ask_assertion_suite(extra: &str) -> String {
    format!(
        r#"{{"provenance": {{"scenario_initial_state": "empty"}},
            "scenarios": {{"probe": {{"purpose": "p", "source": [], "steps": [
              {{"step": "execute_command", "command": "thalamus.assertion.AskAssertion",
                "actor": "thalamus.assertion.Caller",
                "input": {{"signals": {{"kind": "literal", "value": ["s1"]}},
                          "dims": {{"kind": "literal", "value": {{}}}},
                          "after": {{"kind": "literal", "value": "2026-01-01T00:00:00Z"}},
                          "horizon": {{"kind": "literal", "value": "PT1H"}},
                          "within": {{"kind": "literal", "value": "baseline"}},
                          "asker": {{"kind": "literal", "value": {{"kind": "release", "ref": "v1"}}}}}}}}{extra}
            ]}}}}}}"#
    )
}

/// The suite's `expect_direct_response` for `AskAssertion`/`asked`, with `prefix` as the
/// identity's declared prefix and `expected` as its literal expectations.
fn direct_response(outcome: &str, prefix: &str, expected: &str) -> String {
    format!(
        r#",{{"step": "expect_direct_response", "response": {{
            "command": "thalamus.assertion.AskAssertion",
            "outcome": {{"command": "thalamus.assertion.AskAssertion", "outcome": "{outcome}"}},
            "fields": [{{"name": "assertion_id", "type": "thalamus.assertion.AssertionId"}}],
            "declarations": {{"thalamus.assertion.AssertionId": {{"kind": "newtype", "of": "String"}}}},
            "constraints": {{"thalamus.assertion.AssertionId": {{"alphabet": null, "prefix": "{prefix}", "invariants": []}}}},
            "expected": {expected}}}}}"#
    )
}

/// The suite's `expect_response_payload` for `AskAssertion`/`asked`, mapping the event field
/// `target` to the response field `assertion_id`.
fn response_payload(event: &str, target: &str, prefix: &str) -> String {
    format!(
        r#",{{"step": "expect_response_payload", "response": {{
            "command": "thalamus.assertion.AskAssertion",
            "outcome": {{"command": "thalamus.assertion.AskAssertion", "outcome": "asked"}},
            "event": "{event}",
            "fields": [{{"name": "assertion_id", "type": "thalamus.assertion.AssertionId"}}],
            "declarations": {{"thalamus.assertion.AssertionId": {{"kind": "newtype", "of": "String"}}}},
            "mappings": {{"{target}": "assertion_id"}},
            "targets": [{{"name": "{target}", "type": "thalamus.assertion.AssertionId"}}],
            "constraints": {{"thalamus.assertion.AssertionId": {{"alphabet": null, "prefix": "{prefix}", "invariants": []}}}}}}}}"#
    )
}

/// A second registry, so a view holds two rows to count and rank: `r1` and `r2`.
const SECOND_REGISTRY: &str = r#",
    {"step": "execute_command", "command": "thalamus.dimension.DeclareRegistry",
     "actor": "thalamus.dimension.InstanceOperator",
     "input": {"registry_id": {"kind": "literal", "value": "r2"},
               "title": {"kind": "literal", "value": "t"}}},
    {"step": "query_view", "view": "thalamus.dimension.Registries"}"#;

fn run_probe(suite: &str) -> Result<Report, String> {
    conformance::run(suite, &|_| true, MemoryPorts::default)
}

fn failed_reason(suite: &str, extra: &str) -> String {
    let report = run_probe(suite).expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    match report.failed.as_slice() {
        [failure] => failure.reason.clone(),
        _ => panic!("the probe passed with the steps {extra}"),
    }
}

fn fails(extra: &str) -> String {
    failed_reason(&declare_registry_suite("empty", extra), extra)
}

fn ask_fails(extra: &str) -> String {
    failed_reason(&ask_assertion_suite(extra), extra)
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
         "expectation": {"expect": "satisfies", "predicate": {"all": ["state == Declared", "defined(title)"]}}},
        {"step": "expect_view", "view": "thalamus.dimension.Registries",
         "expectation": {"expect": "satisfies", "predicate": {"registry_id": {"any_of": ["r1", "r9"]}}}},
        {"step": "expect_view", "view": "thalamus.dimension.Registries",
         "expectation": {"expect": "counts", "at_least": 1, "at_most": 1}}"#,
    ))
    .expect("the probe suite runs");
    assert!(report.failed.is_empty(), "{}", failures(&report));

    let report = run_probe(&declare_registry_suite(
        "empty",
        &format!(
            r#"{SECOND_REGISTRY},
            {{"step": "expect_view", "view": "thalamus.dimension.Registries",
              "expectation": {{"expect": "counts", "at_least": 2}}}},
            {{"step": "expect_view", "view": "thalamus.dimension.Registries",
              "expectation": {{"expect": "ranked", "order_by": ["registry_id asc"]}}}},
            {{"step": "query_view", "view": "thalamus.observation.SeriesSamples",
              "params": {{"series": {{"kind": "literal", "value": "s"}}}}}},
            {{"step": "expect_view", "view": "thalamus.observation.SeriesSamples",
              "expectation": {{"expect": "counts", "at_most": 0}}}}"#
        ),
    ))
    .expect("the probe suite runs");
    assert!(report.failed.is_empty(), "{}", failures(&report));

    let report = run_probe(&ask_assertion_suite(&format!(
        r#",{{"step": "expect_outcome", "outcome": {{"command": "thalamus.assertion.AskAssertion", "outcome": "asked"}}}}{}{}"#,
        direct_response("asked", "asr_", "{}"),
        response_payload("thalamus.assertion.AssertionAsked", "assertion_id", "asr_"),
    )))
    .expect("the probe suite runs");
    assert!(report.failed.is_empty(), "{}", failures(&report));

    // A shape reads dotted leaves, and leaves out the response-mapped identity it checks
    // through `expect_response_payload`, as the suite's own shapes do.
    let report = run_probe(&ask_assertion_suite(
        r#",{"step": "expect_event", "event": "thalamus.assertion.AssertionAsked",
             "payload": {"asker": {"kind": "release", "ref": "v1"}},
             "shape": {"asker.kind": {"holds": "enum", "variants": ["release", "test_run"]},
                       "asker.ref": {"holds": "primitive", "kind": "string"}}}"#,
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
            "a value outside an any_of a row must be in",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "satisfies", "predicate": {"registry_id": {"any_of": ["r2", "r3"]}}}}"#,
        ),
        (
            "an any_of over a field the row does not hold",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "satisfies", "predicate": {"parent": {"any_of": ["r1"]}}}}"#,
        ),
        (
            "fewer rows than a count's floor",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "counts", "at_least": 2}}"#,
        ),
        (
            "more rows than a count's ceiling",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "counts", "at_most": 0}}"#,
        ),
        (
            "a refusal where the grant admitted the caller",
            r#",{"step": "expect_not_granted", "actor": "thalamus.dimension.InstanceOperator", "unpublished": []}"#,
        ),
        (
            "a response from a command that returns none",
            r#",{"step": "expect_direct_response", "response": {"command": "thalamus.dimension.DeclareRegistry", "outcome": {"command": "thalamus.dimension.DeclareRegistry", "outcome": "declared"}, "fields": [], "declarations": {}, "constraints": {}, "expected": {}}}"#,
        ),
    ] {
        let reason = fails(extra);
        assert!(!reason.is_empty(), "{rule}: failed without a reason");
    }

    for (rule, extra) in [
        (
            "rows out of the declared order",
            format!(
                r#"{SECOND_REGISTRY},{{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {{"expect": "ranked", "order_by": ["registry_id desc"]}}}}"#
            ),
        ),
        (
            "an order over a field the rows do not hold",
            format!(
                r#"{SECOND_REGISTRY},{{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {{"expect": "ranked", "order_by": ["parent asc"]}}}}"#
            ),
        ),
    ] {
        let reason = fails(&extra);
        assert!(!reason.is_empty(), "{rule}: failed without a reason");
    }

    for (rule, extra) in [
        (
            "a response identity without its declared prefix",
            direct_response("asked", "zzz_", "{}"),
        ),
        (
            "a response of another outcome",
            direct_response("no-signals", "asr_", "{}"),
        ),
        (
            "a response literal it does not carry",
            direct_response("asked", "asr_", r#"{"assertion_id": "asr_none"}"#),
        ),
        (
            "a response-mapped event the command did not publish",
            response_payload("thalamus.assertion.AssertionAnswered", "assertion_id", "asr_"),
        ),
        (
            "a response-mapped event field that differs from the response",
            response_payload("thalamus.assertion.AssertionAsked", "asker", "asr_"),
        ),
        (
            "a response-mapped identity without its declared prefix",
            response_payload("thalamus.assertion.AssertionAsked", "assertion_id", "zzz_"),
        ),
        (
            "a dotted leaf outside its stated variants",
            r#",{"step": "expect_event", "event": "thalamus.assertion.AssertionAsked", "payload": {}, "shape": {"asker.kind": {"holds": "enum", "variants": ["controller"]}}}"#.to_owned(),
        ),
        (
            "a dotted leaf the payload does not carry",
            r#",{"step": "expect_event", "event": "thalamus.assertion.AssertionAsked", "payload": {}, "shape": {"asker.owner": {"holds": "primitive", "kind": "string"}}}"#.to_owned(),
        ),
        (
            "a dotted leaf under a field that is no object",
            r#",{"step": "expect_event", "event": "thalamus.assertion.AssertionAsked", "payload": {}, "shape": {"assertion_id.kind": {"holds": "primitive", "kind": "string", "optional": true}}}"#.to_owned(),
        ),
    ] {
        let reason = ask_fails(&extra);
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
            "an any_of that is not a list",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "satisfies", "predicate": {"registry_id": {"any_of": "r1"}}}}"#,
        ),
        (
            "a count with neither bound",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "counts"}}"#,
        ),
        (
            "a count bound that is not a whole number",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "counts", "at_least": 1.5}}"#,
        ),
        (
            "an order in no direction",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "ranked", "order_by": ["registry_id sideways"]}}"#,
        ),
        (
            "an order by nothing",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries"},{"step": "expect_view", "view": "thalamus.dimension.Registries", "expectation": {"expect": "ranked", "order_by": []}}"#,
        ),
        (
            "a parameter the view does not declare",
            r#",{"step": "query_view", "view": "thalamus.dimension.Registries", "params": {"series": {"kind": "literal", "value": "s"}}}"#,
        ),
        (
            "a parameterised view queried without its parameter",
            r#",{"step": "query_view", "view": "thalamus.observation.SeriesSamples"}"#,
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

    for (rule, extra) in [
        (
            "an unknown member of a direct response",
            direct_response("asked", "asr_", "{}").replace(
                r#""expected": {}"#,
                r#""expected": {}, "undeclared_fields": "ignored""#,
            ),
        ),
        (
            "a response constraint with an invariant it cannot decide",
            direct_response("asked", "asr_", "{}")
                .replace(r#""invariants": []"#, r#""invariants": ["value != asr_0"]"#),
        ),
        (
            "a response field of a type it cannot resolve",
            direct_response("asked", "asr_", "{}").replace(
                r#""type": "thalamus.assertion.AssertionId"}]"#,
                r#""type": "thalamus.assertion.Unknown"}]"#,
            ),
        ),
        (
            "a mapping onto a target it does not declare",
            response_payload("thalamus.assertion.AssertionAsked", "assertion_id", "asr_").replace(
                r#""targets": [{"name": "assertion_id""#,
                r#""targets": [{"name": "other""#,
            ),
        ),
    ] {
        let reason = ask_fails(&extra);
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
