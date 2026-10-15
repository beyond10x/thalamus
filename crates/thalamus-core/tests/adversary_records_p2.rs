//! Adversary pass 2 for `story:m0-conformance-records`: the identity sequence, the `ranked`
//! comparison of the conformance harness, and the refusals the suite reaches with one verdict
//! only. Each case asserts a literal expected value or verdict.
//!
//! The harness cases build a one-scenario suite from the steps of `spec/suite.json` and run it
//! through `thalamus_core::conformance::run`, so the comparison under test is the one a suite
//! run uses. The expected verdict of each `ranked` check is the ESS 0.60.0 reference runner's
//! (`compare_nodes`): two texts that are each an RFC 3339 `date-time` (`T` or `t`, seconds up to
//! 59, at most nine fraction digits, `Z` or `±HH:MM`) order by instant, any other pair of texts by
//! UTF-8 bytes, and numbers by exact value.

use std::collections::BTreeMap;

use thalamus_core::behaviour::observation::IdSequence;
use thalamus_core::conformance::{self, Ports};
use thalamus_model::assertion::{
    AskAssertion, Asker, AskerKind, AssertionId, AssertionSnapshot, Within,
};
use thalamus_model::behaviour::{
    AssertionStorage, Generated, SignalDefinitionStorage, SignalEvaluationStorage,
    SignalSampleStorage,
};
use thalamus_model::definition::obligations::{
    ActivateDefinitionBehavior, ProposeDefinitionBehavior,
};
use thalamus_model::definition::{
    ActivateDefinition, ActivateDefinitionOutcome, AdapterId, Author, ConnectionId,
    DenominatorSpec, EvaluationSpec, ExtractorId, Family, History, Links, Owner, ProposeDefinition,
    ProposeDefinitionOutcome, QueryText, RuleId, SignalDefinitionSnapshot, SignalId, SignalKind,
    Source, Unit,
};
use thalamus_model::dimension::{DimensionKey, DimensionValue};
use thalamus_model::evaluation::obligations::RecordEvaluationBehavior;
use thalamus_model::evaluation::{
    EvaluationId, RecordEvaluation, RecordEvaluationOutcome, SignalEvaluationSnapshot, Verdict,
};
use thalamus_model::json::{self, Value};
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::obligations::RecordSampleBehavior;
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RecordSample, RecordSampleOutcome, RunId,
    SampleId, SeriesKey, SignalSampleSnapshot, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

// ---------------------------------------------------------------------------------------------
// Probe suites built from the steps of spec/suite.json
// ---------------------------------------------------------------------------------------------

const SAMPLE_SCENARIO: &str = "thalamus.observation.RecordSample/outcome/recorded";
const EVALUATION_SCENARIO: &str = "thalamus.evaluation.RecordEvaluation/outcome/recorded";
const SERIES: &str = "adversary-series";

fn parsed_suite() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    json::parse(&text).expect("the suite is JSON")
}

fn steps_of(suite: &Value, scenario: &str) -> Vec<Value> {
    match suite
        .member("scenarios")
        .and_then(|scenarios| scenarios.member(scenario))
        .and_then(|scenario| scenario.member("steps"))
    {
        Some(Value::Array(steps)) => steps.clone(),
        other => panic!("{scenario} holds no steps: {other:?}"),
    }
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn object(members: &[(&str, Value)]) -> Value {
    Value::Object(
        members
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn literal(value: Value) -> Value {
    object(&[("kind", text("literal")), ("value", value)])
}

fn instance(name: &str) -> Value {
    object(&[("kind", text("instance")), ("instance", text(name))])
}

fn executes(step: &Value, command: &str) -> bool {
    step.member("step") == Some(&text("execute_command"))
        && step.member("command") == Some(&text(command))
}

/// Replaces the input field `field` of an `execute_command` step.
fn with_input(mut step: Value, field: &str, value: Value) -> Value {
    let Value::Object(members) = &mut step else {
        panic!("a step is not an object");
    };
    let (_, input) = members
        .iter_mut()
        .find(|(key, _)| key == "input")
        .expect("an execute_command step carries its input");
    let Value::Object(fields) = input else {
        panic!("an input is not an object");
    };
    match fields.iter_mut().find(|(key, _)| key == field) {
        Some((_, held)) => *held = value,
        None => fields.push((field.to_owned(), value)),
    }
    step
}

fn outcome(command: &str, outcome: &str) -> Value {
    object(&[
        ("step", text("expect_outcome")),
        (
            "outcome",
            object(&[("command", text(command)), ("outcome", text(outcome))]),
        ),
    ])
}

/// A scenario that proposes and activates one signal, as the suite's own `recorded` scenario
/// does, and nothing else yet.
struct Probe {
    steps: Vec<Value>,
    sample: Value,
    evaluation: Value,
}

impl Probe {
    fn new() -> Self {
        let suite = parsed_suite();
        let steps = steps_of(&suite, SAMPLE_SCENARIO);
        let at = steps
            .iter()
            .position(|step| executes(step, "thalamus.observation.RecordSample"))
            .expect("the recorded scenario records a sample");
        let evaluation = steps_of(&suite, EVALUATION_SCENARIO)
            .into_iter()
            .find(|step| executes(step, "thalamus.evaluation.RecordEvaluation"))
            .expect("the recorded scenario records an evaluation");
        Self {
            sample: with_input(steps[at].clone(), "series", literal(text(SERIES))),
            steps: steps[..at].to_vec(),
            evaluation,
        }
    }

    /// Records a sample at `ts` on the probe's series and captures its id as `name`.
    fn sample(mut self, name: &str, ts: &str) -> Self {
        self.steps
            .push(with_input(self.sample.clone(), "ts", literal(text(ts))));
        self.steps
            .push(outcome("thalamus.observation.RecordSample", "recorded"));
        self.steps.push(object(&[
            ("step", text("capture_instance")),
            ("instance", text(name)),
            ("entity", text("thalamus.observation.SignalSample")),
            ("event", text("thalamus.observation.SampleRecorded")),
            ("field", text("sample_id")),
        ]));
        self
    }

    /// Records an `OK` evaluation of the sample captured as `sample` with this revision and score.
    fn evaluation(mut self, sample: &str, definition_rev: &str, score: &str) -> Self {
        let step = with_input(self.evaluation.clone(), "sample_id", instance(sample));
        let step = with_input(step, "verdict", literal(text("OK")));
        let step = with_input(
            step,
            "definition_rev",
            literal(Value::Number(definition_rev.to_owned())),
        );
        let step = with_input(step, "score", literal(Value::Number(score.to_owned())));
        self.steps.push(step);
        self.steps
            .push(outcome("thalamus.evaluation.RecordEvaluation", "recorded"));
        self
    }

    /// The verdict of a run that queries `view` and expects it ranked by `order_by`: `Ok` when the
    /// scenario passes, the harness's reason when it fails.
    fn ranked(&self, view: &str, order_by: &[&str]) -> Result<(), String> {
        let mut steps = self.steps.clone();
        let mut query = vec![("step", text("query_view")), ("view", text(view))];
        if view == "thalamus.observation.SeriesSamples" {
            query.push(("params", object(&[("series", literal(text(SERIES)))])));
        }
        steps.push(object(&query));
        steps.push(object(&[
            ("step", text("expect_view")),
            ("view", text(view)),
            (
                "expectation",
                object(&[
                    ("expect", text("ranked")),
                    (
                        "order_by",
                        Value::Array(order_by.iter().map(|key| text(key)).collect()),
                    ),
                ]),
            ),
        ]));
        let suite = object(&[
            (
                "provenance",
                object(&[("scenario_initial_state", text("empty"))]),
            ),
            (
                "scenarios",
                object(&[(
                    "probe",
                    object(&[
                        ("purpose", text("adversary probe")),
                        ("source", Value::Array(Vec::new())),
                        ("steps", Value::Array(steps)),
                    ]),
                )]),
            ),
        ]);
        let mut rendered = String::new();
        json::push_value(&mut rendered, &suite);
        let report = conformance::run(&rendered, &|_| true, MemoryPorts::default)
            .expect("the probe suite runs");
        assert_eq!(report.ran, ["probe"], "the probe ran");
        match report.failed.as_slice() {
            [] => Ok(()),
            [failure] => Err(failure.reason.clone()),
            more => panic!("one scenario failed {} times", more.len()),
        }
    }
}

/// `Evaluations.score` is a `Decimal` and `definition_rev` an `Integer`; both rank by value
/// (`9` before `10`), where their bytes put `10` first. The implementor's unit test hands
/// `ranked` a hand-built `Ranking`, so nothing else drives the `{view}.Row` schema lookup that
/// decides whether a field ranks as a decimal, or a ranking with a second key.
#[test]
fn a_decimal_and_an_integer_rank_by_value_through_the_harness() {
    let probe = Probe::new()
        .sample("first", "2026-01-05T09:00:00Z")
        .sample("second", "2026-01-05T10:00:00Z")
        .evaluation("first", "9", "9")
        .evaluation("second", "10", "10");
    let view = "thalamus.evaluation.Evaluations";
    assert_eq!(probe.ranked(view, &["score asc"]), Ok(()), "score asc");
    assert_eq!(
        probe.ranked(view, &["definition_rev asc"]),
        Ok(()),
        "definition_rev asc"
    );
    assert_eq!(
        probe.ranked(view, &["definition_rev asc", "score desc"]),
        Ok(()),
        "the first key decides, so the second is not read"
    );
    assert!(
        refused_by_ranked(&probe.ranked(view, &["score desc"])),
        "9 then 10 is not score desc"
    );
}

/// `SeriesSamples` is `order_by: [ts asc]`. Samples with mixed offsets and fractions are served
/// in instant order, and the harness ranks them by instant: by bytes,
/// `2026-01-05T10:00:02.25+01:00` would sort after `2026-01-05T09:00:02.5Z`.
#[test]
fn mixed_offsets_and_fractions_rank_by_instant_through_the_harness() {
    let probe = Probe::new()
        .sample("a", "2026-01-05T09:00:03Z")
        .sample("b", "2026-01-05T09:00:02.5Z")
        .sample("c", "2026-01-05T10:00:01+02:00")
        .sample("d", "2026-01-05T10:00:02.25+01:00");
    let view = "thalamus.observation.SeriesSamples";
    assert_eq!(probe.ranked(view, &["ts asc"]), Ok(()), "ts asc");
    assert!(
        refused_by_ranked(&probe.ranked(view, &["ts desc"])),
        "four distinct instants cannot also be ts desc"
    );
}

/// `series_samples` serves a ts that is not an instant after every instant
/// (`instant_order`), while the harness's `ranked`, like the reference, orders an instant and a
/// non-instant by bytes. A ts without an offset passes the generated decoder, so the unit's own
/// harness refuses the order the unit's own view serves.
#[test]
fn the_harness_accepts_the_order_series_samples_serves() {
    let probe = Probe::new()
        .sample("instant", "2026-01-05T00:00:00Z")
        .sample("no-offset", "2026-01-04T00:00:00");
    assert_eq!(
        probe.ranked("thalamus.observation.SeriesSamples", &["ts asc"]),
        Ok(()),
        "SeriesSamples serves rows its own ranked check refuses"
    );
}

/// The harness reads instants with the `time` crate, which also admits a space separator and
/// more than nine fraction digits. The reference reads neither as an instant and orders such a
/// text by bytes. On `Evaluations` (no declared order, so rows come in the order recorded) each
/// pair below is out of `ts asc` order for the reference and in order for the harness.
#[test]
fn a_text_the_reference_reads_as_no_instant_ranks_by_its_bytes() {
    let mut passed = Vec::new();
    for (earlier, later) in [
        (
            "2026-01-05T10:00:00.0000000001+02:00",
            "2026-01-05T09:00:00Z",
        ),
        ("2026-01-05T09:00:03Z", "2026-01-05 10:00:01Z"),
    ] {
        let probe = Probe::new()
            .sample("earlier", earlier)
            .sample("later", later)
            .evaluation("earlier", "1", "1")
            .evaluation("later", "1", "1");
        let verdict = probe.ranked("thalamus.evaluation.Evaluations", &["ts asc"]);
        if !refused_by_ranked(&verdict) {
            passed.push(format!("{earlier} then {later}: {verdict:?}"));
        }
    }
    assert_eq!(
        passed,
        Vec::<String>::new(),
        "pairs the reference refuses as ts asc (bytes) and the harness did not"
    );
}

/// `true` when the probe failed at its `ranked` expectation, the last step, and not before it.
fn refused_by_ranked(verdict: &Result<(), String>) -> bool {
    matches!(verdict, Err(reason) if reason.contains(r#""expect":"ranked""#))
}

/// The reference ranks numbers by exact value. 9007199254740993 then 9007199254740992 is out of
/// `definition_rev asc` order; as `f64` the two are one value, a tie, and a tie passes.
#[test]
fn an_integer_ranks_exactly_past_two_to_the_fifty_three() {
    let probe = Probe::new()
        .sample("first", "2026-01-05T09:00:00Z")
        .sample("second", "2026-01-05T10:00:00Z")
        .evaluation("first", "9007199254740993", "1")
        .evaluation("second", "9007199254740992", "1");
    let verdict = probe.ranked("thalamus.evaluation.Evaluations", &["definition_rev asc"]);
    assert!(
        refused_by_ranked(&verdict),
        "9007199254740993 then 9007199254740992 passed definition_rev asc: {verdict:?}"
    );
}

/// A tie on the first key leaves the order to the second (reference `ranked`: "Equal on this key
/// decides nothing; the next key does"). Equal revisions with scores 10 then 9 are out of
/// `[definition_rev asc, score asc]` order.
#[test]
fn a_tie_on_the_first_key_is_decided_by_the_second() {
    let probe = Probe::new()
        .sample("first", "2026-01-05T09:00:00Z")
        .sample("second", "2026-01-05T10:00:00Z")
        .evaluation("first", "1", "10")
        .evaluation("second", "1", "9");
    let view = "thalamus.evaluation.Evaluations";
    assert_eq!(
        probe.ranked(view, &["definition_rev asc", "score desc"]),
        Ok(()),
        "a tie on definition_rev, then 10 before 9 is score desc"
    );
    let verdict = probe.ranked(view, &["definition_rev asc", "score asc"]);
    assert!(
        refused_by_ranked(&verdict),
        "a tie on definition_rev, then 10 before 9, passed score asc: {verdict:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The identity sequence
// ---------------------------------------------------------------------------------------------

type System = Generated<Ports<MemoryPorts>>;

fn wrap<T>(make: fn(String) -> T, value: &str) -> T {
    make(value.to_owned())
}

/// A system over `store`. `MemoryPorts` clones share their rows, so two systems over clones of
/// one store write to one store, each with its own `Ports` and so its own identity sequence.
fn system_over(store: &MemoryPorts) -> System {
    Generated::new(Ports::new(
        store.clone(),
        Some(wrap(Timestamp, "2026-01-01T00:00:00Z")),
    ))
}

fn propose(signal: &str) -> ProposeDefinition {
    ProposeDefinition {
        signal_id: wrap(SignalId, signal),
        title: "Failure ratio".to_owned(),
        family: wrap(Family, "platform"),
        kind: SignalKind::Ratio,
        unit: Some(wrap(Unit, "1")),
        dimensions: Some(vec![wrap(DimensionKey, "cluster")]),
        source: Source {
            adapter: wrap(AdapterId, "sql"),
            connection: wrap(ConnectionId, "replica"),
        },
        extractor: wrap(ExtractorId, "sql.ratio"),
        query: BTreeMap::from([("ratio".to_owned(), wrap(QueryText, "select 1"))]),
        window: wrap(Duration, "PT6H"),
        cadence: wrap(Duration, "PT1H"),
        history: History::Source,
        denominator: Some(DenominatorSpec {
            unit: wrap(Unit, "{request}"),
            min_n: 20,
            since: "first_sample".to_owned(),
        }),
        evaluation: EvaluationSpec {
            rule: wrap(RuleId, "threshold"),
            baselines: Vec::new(),
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: None,
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(wrap(Owner, "team")),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        author: wrap(Author, "author"),
    }
}

fn active(system: &mut System, signal: &str) {
    let proposed = ProposeDefinitionBehavior::propose_definition(system, propose(signal))
        .expect("ProposeDefinition answers");
    assert!(
        matches!(proposed, ProposeDefinitionOutcome::Proposed { .. }),
        "{proposed:?}"
    );
    let activated = ActivateDefinitionBehavior::activate_definition(
        system,
        ActivateDefinition {
            signal_id: wrap(SignalId, signal),
        },
    )
    .expect("ActivateDefinition answers");
    assert!(
        matches!(activated, ActivateDefinitionOutcome::Activated { .. }),
        "{activated:?}"
    );
}

fn sample(ts: &str) -> RecordSample {
    let at = |instant: &str| wrap(Timestamp, instant);
    RecordSample {
        text: None,
        signal: wrap(SignalId, "platform.ratio"),
        series: wrap(SeriesKey, "s"),
        dims: BTreeMap::from([("cluster".to_owned(), wrap(DimensionValue, "a"))]),
        ts: at(ts),
        window: TimeWindow {
            start: at("2025-12-31T18:00:00Z"),
            end: at("2026-01-01T00:00:00Z"),
        },
        reading: Reading::Ok(CompleteReading {
            value: wrap(Decimal, "0.029"),
        }),
        unit: wrap(Unit, "1"),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".to_owned(),
            version: "0.1.0".to_owned(),
            definition_rev: 1,
            adapter: wrap(AdapterId, "sql"),
            connection: wrap(ConnectionId, "replica"),
            query_sha256: wrap(QueryDigest, "00ff"),
            run_id: wrap(RunId, "run_1"),
        },
    }
}

fn recorded(system: &mut System, ts: &str) -> SampleId {
    match RecordSampleBehavior::record_sample(system, sample(ts)) {
        Ok(RecordSampleOutcome::Recorded { response, .. }) => response.sample_id,
        other => panic!("the sample at {ts} was not recorded: {other:?}"),
    }
}

fn evaluation(sample_id: SampleId, verdict: Verdict, reason: Option<&str>) -> RecordEvaluation {
    RecordEvaluation {
        sample_id,
        definition_rev: 1,
        verdict,
        prev: None,
        score: None,
        baseline: None,
        reason: reason.map(str::to_owned),
    }
}

fn ask() -> AskAssertion {
    AskAssertion {
        signals: vec![wrap(SignalId, "platform.ratio")],
        dims: BTreeMap::new(),
        after: wrap(Timestamp, "2026-01-01T00:00:00Z"),
        horizon: wrap(Duration, "PT1H"),
        within: Within::Baseline,
        asker: Asker {
            kind: AskerKind::Release,
            r#ref: "v1".to_owned(),
        },
    }
}

/// A second `Ports` over the store starts its sequence at the beginning; `mint` must still skip
/// the ids the store holds, or the new sample replaces a stored one.
#[test]
fn a_second_ports_over_one_store_keeps_every_stored_sample() {
    let store = MemoryPorts::default();
    let mut first = system_over(&store);
    active(&mut first, "platform.ratio");
    let a = recorded(&mut first, "2026-01-01T00:00:00Z");
    let b = recorded(&mut first, "2026-01-01T01:00:00Z");
    let mut second = system_over(&store);
    let c = recorded(&mut second, "2026-01-01T02:00:00Z");
    assert!(c != a && c != b, "{c:?} reissued one of {a:?}, {b:?}");
    let mut ids: Vec<String> = SignalSampleStorage::list(&store)
        .into_iter()
        .map(|row| row.data.sample_id.0)
        .collect();
    ids.sort();
    let mut expected = vec![a.0, b.0, c.0];
    expected.sort();
    assert_eq!(ids, expected, "the store holds all three samples");
}

/// `IdSequence` says an identity is never issued twice, also after its row is deleted, and that
/// the sequence is held by the store. It is held by `Ports`, beside the store: a second `Ports`
/// over the same (clone-shared) store reissues the id of a deleted newest row.
#[test]
fn a_second_ports_over_one_store_never_reissues_a_deleted_id() {
    let store = MemoryPorts::default();
    let mut first = system_over(&store);
    active(&mut first, "platform.ratio");
    let _a = recorded(&mut first, "2026-01-01T00:00:00Z");
    let newest = recorded(&mut first, "2026-01-01T01:00:00Z");
    SignalSampleStorage::delete(&mut first.ports, &newest);
    let mut second = system_over(&store);
    let next = recorded(&mut second, "2026-01-01T02:00:00Z");
    assert_ne!(
        next, newest,
        "a deleted sample's id was issued again by a second Ports over the store"
    );
}

/// Ports over a store whose identity sequence is spent: every storage call goes to the store,
/// and `next_number` answers `None`.
struct Spent(MemoryPorts);

impl IdSequence for Spent {
    fn next_number(&mut self, _prefix: &'static str) -> Option<u64> {
        None
    }
}

macro_rules! delegate {
    ($port:ident, $identity:ty, $snapshot:ty) => {
        impl $port for Spent {
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

delegate!(SignalDefinitionStorage, SignalId, SignalDefinitionSnapshot);
delegate!(SignalSampleStorage, SampleId, SignalSampleSnapshot);
delegate!(
    SignalEvaluationStorage,
    EvaluationId,
    SignalEvaluationSnapshot
);
delegate!(AssertionStorage, AssertionId, AssertionSnapshot);

/// A spent sequence answers `UnmetObligation` naming the identity sequence and its prefix, and
/// none of the three behaviours that mint writes a row.
#[test]
fn a_spent_sequence_answers_unmet_and_writes_nothing() {
    use thalamus_core::behaviour::{assertion, evaluation as evaluations, observation};
    let store = MemoryPorts::default();
    let mut system = system_over(&store);
    active(&mut system, "platform.ratio");
    let sample_id = recorded(&mut system, "2026-01-01T00:00:00Z");
    let mut spent = Spent(store.clone());
    let unmet = |source: &'static str| UnmetObligation {
        capability: "identity sequence",
        source,
    };

    assert_eq!(
        observation::record_sample(&mut spent, sample("2026-01-01T01:00:00Z")),
        Err(unmet("smp_"))
    );
    assert_eq!(
        evaluations::record_evaluation(
            &mut spent,
            evaluation(sample_id, Verdict::OK, Some("spent"))
        ),
        Err(unmet("evl_"))
    );
    assert_eq!(
        assertion::ask_assertion(&mut spent, ask()),
        Err(unmet("asr_"))
    );
    assert_eq!(SignalSampleStorage::list(&store).len(), 1, "samples");
    assert_eq!(
        SignalEvaluationStorage::list(&store).len(),
        0,
        "evaluations"
    );
    assert_eq!(AssertionStorage::list(&store).len(), 0, "assertions");
}

// ---------------------------------------------------------------------------------------------
// Refusals the suite reaches with one verdict only
// ---------------------------------------------------------------------------------------------

/// `reason-missing` refuses every verdict but `OK` without a reason (`verdict != OK and
/// reason == null`). The suite's `reason-missing` scenario uses `N/A` only.
#[test]
fn every_verdict_but_ok_needs_a_reason() {
    let store = MemoryPorts::default();
    let mut system = system_over(&store);
    active(&mut system, "platform.ratio");
    let sample_id = recorded(&mut system, "2026-01-01T00:00:00Z");
    for verdict in [Verdict::WARN, Verdict::CRIT, Verdict::UNKNOWN, Verdict::NA] {
        let answer = RecordEvaluationBehavior::record_evaluation(
            &mut system,
            evaluation(sample_id.clone(), verdict, None),
        );
        assert!(
            matches!(answer, Ok(RecordEvaluationOutcome::ReasonMissing { .. })),
            "{verdict:?} without a reason: {answer:?}"
        );
    }
    let ok = RecordEvaluationBehavior::record_evaluation(
        &mut system,
        evaluation(sample_id, Verdict::OK, None),
    );
    assert!(
        matches!(ok, Ok(RecordEvaluationOutcome::Recorded { .. })),
        "OK without a reason: {ok:?}"
    );
}

/// `SignalEvaluation` declares `definition_rev >= 1` and no `RecordEvaluation` outcome refuses a
/// lower revision, so `record_evaluation` answers `UnmetObligation` naming the invariant (its doc
/// comment) and stores nothing. The suite records revisions of 1 and up only.
#[test]
fn a_revision_below_one_is_unmet_and_stores_nothing() {
    let store = MemoryPorts::default();
    let mut system = system_over(&store);
    active(&mut system, "platform.ratio");
    let sample_id = recorded(&mut system, "2026-01-01T00:00:00Z");
    let mut input = evaluation(sample_id, Verdict::OK, None);
    input.definition_rev = 0;
    let answer = RecordEvaluationBehavior::record_evaluation(&mut system, input);
    let unmet = UnmetObligation {
        capability: "entity invariant",
        source: "definition_rev >= 1",
    };
    assert!(
        matches!(&answer, Err(refused) if *refused == unmet),
        "definition_rev 0: {answer:?}"
    );
    assert_eq!(
        SignalEvaluationStorage::list(&store).len(),
        0,
        "evaluations"
    );
}
