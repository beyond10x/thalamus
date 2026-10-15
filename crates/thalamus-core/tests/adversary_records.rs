//! Adversary cases for `story:m0-conformance-records`: behaviour the specification states and the
//! synthesised suite does not reach. Each case drives the obligation through the same ports the
//! conformance harness runs over and asserts a literal expected value.

use std::collections::BTreeMap;

use thalamus_core::conformance::Ports;
use thalamus_model::assertion::obligations::AskAssertionBehavior;
use thalamus_model::assertion::{AskAssertion, AskAssertionOutcome, Asker, AskerKind, Within};
use thalamus_model::behaviour::{
    AssertionStorage, Generated, SignalEvaluationStorage, SignalSampleStorage,
};
use thalamus_model::definition::obligations::{
    ActivateDefinitionBehavior, MuteDefinitionBehavior, ProposeDefinitionBehavior,
    RetireDefinitionBehavior,
};
use thalamus_model::definition::{
    ActivateDefinition, ActivateDefinitionOutcome, AdapterId, Author, ConnectionId,
    DenominatorSpec, EvaluationSpec, ExtractorId, Family, History, Links, MuteDefinition,
    MuteDefinitionOutcome, Owner, ProposeDefinition, ProposeDefinitionOutcome, QueryText,
    RetireDefinition, RetireDefinitionOutcome, RuleId, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::{DimensionKey, DimensionValue};
use thalamus_model::evaluation::obligations::{RecordEvaluationBehavior, SeriesVerdictsQuery};
use thalamus_model::evaluation::{RecordEvaluation, RecordEvaluationOutcome, Verdict};
use thalamus_model::observation::obligations::{RecordSampleBehavior, SeriesSamplesQuery};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RecordSample, RecordSampleOutcome, RunId,
    SampleId, SeriesKey, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

type System = Generated<Ports<MemoryPorts>>;

fn text<T>(wrap: fn(String) -> T, value: &str) -> T {
    wrap(value.to_owned())
}

fn system() -> System {
    Generated::new(Ports::new(
        MemoryPorts::default(),
        Some(text(Timestamp, "2026-01-01T00:00:00Z")),
    ))
}

fn propose(signal: &str) -> ProposeDefinition {
    ProposeDefinition {
        signal_id: text(SignalId, signal),
        title: "Failure ratio".to_owned(),
        family: text(Family, "platform"),
        kind: SignalKind::Ratio,
        unit: Some(text(Unit, "1")),
        dimensions: Some(vec![text(DimensionKey, "cluster")]),
        source: Source {
            adapter: text(AdapterId, "sql"),
            connection: text(ConnectionId, "replica"),
        },
        extractor: text(ExtractorId, "sql.ratio"),
        query: BTreeMap::from([("ratio".to_owned(), text(QueryText, "select 1"))]),
        window: text(Duration, "PT6H"),
        cadence: text(Duration, "PT1H"),
        history: History::Source,
        denominator: Some(DenominatorSpec {
            unit: text(Unit, "{request}"),
            min_n: 20,
            since: "first_sample".to_owned(),
        }),
        evaluation: EvaluationSpec {
            rule: text(RuleId, "threshold"),
            baselines: Vec::new(),
            warn: None,
            crit: None,
            min_trend_window: None,
        },
        objective: None,
        owner: Some(text(Owner, "team")),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        author: text(Author, "author"),
    }
}

/// Proposes and activates `signal`.
fn active(system: &mut System, signal: &str) {
    let proposed = ProposeDefinitionBehavior::propose_definition(system, propose(signal))
        .expect("ProposeDefinition answers");
    assert!(
        matches!(proposed, ProposeDefinitionOutcome::Proposed { .. }),
        "{signal} was not proposed: {proposed:?}"
    );
    let activated = ActivateDefinitionBehavior::activate_definition(
        system,
        ActivateDefinition {
            signal_id: text(SignalId, signal),
        },
    )
    .expect("ActivateDefinition answers");
    assert!(
        matches!(activated, ActivateDefinitionOutcome::Activated { .. }),
        "{signal} was not activated: {activated:?}"
    );
}

fn sample(signal: &str, series: &str, ts: &str) -> RecordSample {
    let at = |instant: &str| text(Timestamp, instant);
    RecordSample {
        signal: text(SignalId, signal),
        series: text(SeriesKey, series),
        dims: BTreeMap::from([("cluster".to_owned(), text(DimensionValue, "a"))]),
        ts: at(ts),
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
            definition_rev: 1,
            adapter: text(AdapterId, "sql"),
            connection: text(ConnectionId, "replica"),
            query_sha256: text(QueryDigest, "00ff"),
            run_id: text(RunId, "run_1"),
        },
    }
}

/// Records a sample that must be accepted, and returns its id.
fn recorded(system: &mut System, signal: &str, series: &str, ts: &str) -> SampleId {
    match RecordSampleBehavior::record_sample(system, sample(signal, series, ts)) {
        Ok(RecordSampleOutcome::Recorded { response, .. }) => response.sample_id,
        other => panic!("the sample at {ts} on {series} was not recorded: {other:?}"),
    }
}

fn evaluation(sample_id: SampleId, verdict: Verdict, score: Option<&str>) -> RecordEvaluation {
    RecordEvaluation {
        sample_id,
        definition_rev: 1,
        verdict,
        prev: None,
        score: score.map(|score| text(Decimal, score)),
        baseline: None,
        reason: Some("adversary".to_owned()),
    }
}

/// `SeriesSamples` declares `filter: series == param.series` (`spec/domains/observation.yaml`).
/// The suite's only query of it reads one series that holds every sample of its scenario, and
/// `every_obligation_is_met` stores one sample, so neither sees a row of another series.
#[test]
fn series_samples_serves_only_the_named_series() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let a = recorded(
        &mut system,
        "platform.ratio",
        "series-a",
        "2026-01-01T00:00:00Z",
    );
    let _b = recorded(
        &mut system,
        "platform.ratio",
        "series-ab",
        "2026-01-01T00:00:00Z",
    );
    let rows = SeriesSamplesQuery::series_samples(&system, text(SeriesKey, "series-a"))
        .expect("SeriesSamples answers");
    let ids: Vec<&str> = rows.iter().map(|row| row.sample_id.0.as_str()).collect();
    assert_eq!(ids, vec![a.0.as_str()], "SeriesSamples(series-a)");
}

/// `SeriesVerdicts` declares `filter: series == param.series` (`spec/domains/evaluation.yaml`).
/// The suite checks it with `contains` only, which a view serving every series also passes.
#[test]
fn series_verdicts_serves_only_the_named_series() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let a = recorded(
        &mut system,
        "platform.ratio",
        "series-a",
        "2026-01-01T00:00:00Z",
    );
    let b = recorded(
        &mut system,
        "platform.ratio",
        "series-ab",
        "2026-01-01T00:00:00Z",
    );
    let mut ids = Vec::new();
    for sample_id in [a, b] {
        match RecordEvaluationBehavior::record_evaluation(
            &mut system,
            evaluation(sample_id, Verdict::OK, None),
        ) {
            Ok(RecordEvaluationOutcome::Recorded { response, .. }) => {
                ids.push(response.evaluation_id);
            }
            other => panic!("the evaluation was not recorded: {other:?}"),
        }
    }
    let rows = SeriesVerdictsQuery::series_verdicts(&system, text(SeriesKey, "series-a"))
        .expect("SeriesVerdicts answers");
    let served: Vec<&str> = rows
        .iter()
        .map(|row| row.evaluation_id.0.as_str())
        .collect();
    assert_eq!(served, vec![ids[0].0.as_str()], "SeriesVerdicts(series-a)");
}

/// `SeriesSamples` declares `order_by: [ts asc]`. The suite's one `ranked` check records its
/// samples in time order, so store order already passes it. Here the samples arrive out of
/// order and one carries an offset whose text sorts after an instant it precedes.
#[test]
fn series_samples_orders_by_instant_not_by_arrival_or_text() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let late = recorded(&mut system, "platform.ratio", "s", "2026-01-05T09:00:03Z");
    let offset = recorded(
        &mut system,
        "platform.ratio",
        "s",
        "2026-01-05T10:00:01+02:00",
    );
    let early = recorded(&mut system, "platform.ratio", "s", "2026-01-04T00:00:00Z");
    let rows = SeriesSamplesQuery::series_samples(&system, text(SeriesKey, "s"))
        .expect("SeriesSamples answers");
    let ids: Vec<&str> = rows.iter().map(|row| row.sample_id.0.as_str()).collect();
    assert_eq!(
        ids,
        vec![early.0.as_str(), offset.0.as_str(), late.0.as_str()],
        "SeriesSamples in ts order (2026-01-04, 08:00:01Z, 09:00:03Z)"
    );
}

/// `not-active` refuses every state but `Active` (`predicate: state != Active`); the suite's
/// `not-active` scenario reaches only `Proposed`.
#[test]
fn muted_and_retired_definitions_refuse_a_sample() {
    let mut system = system();
    active(&mut system, "platform.muted");
    active(&mut system, "platform.retired");
    let muted = MuteDefinitionBehavior::mute_definition(
        &mut system,
        MuteDefinition {
            signal_id: text(SignalId, "platform.muted"),
            reason: "maintenance".to_owned(),
            until: text(Timestamp, "2026-02-01T00:00:00Z"),
        },
    )
    .expect("MuteDefinition answers");
    assert!(
        matches!(muted, MuteDefinitionOutcome::Muted { .. }),
        "not muted: {muted:?}"
    );
    let retired = RetireDefinitionBehavior::retire_definition(
        &mut system,
        RetireDefinition {
            signal_id: text(SignalId, "platform.retired"),
        },
    )
    .expect("RetireDefinition answers");
    assert!(
        matches!(retired, RetireDefinitionOutcome::Retired { .. }),
        "not retired: {retired:?}"
    );
    for signal in ["platform.muted", "platform.retired"] {
        let answer = RecordSampleBehavior::record_sample(
            &mut system,
            sample(signal, signal, "2026-01-01T00:00:00Z"),
        );
        assert!(
            matches!(answer, Ok(RecordSampleOutcome::NotActive { .. })),
            "{signal}: {answer:?}"
        );
    }
}

/// The verdict's `scored` attribute: `CRIT` carries a score, `UNKNOWN` never does. The suite
/// reaches `OK` and `N/A` only, and `every_obligation_is_met` adds `WARN`.
#[test]
fn crit_carries_a_score_and_unknown_refuses_one() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let id = recorded(&mut system, "platform.ratio", "s", "2026-01-01T00:00:00Z");
    let crit = RecordEvaluationBehavior::record_evaluation(
        &mut system,
        evaluation(id.clone(), Verdict::CRIT, Some("2.4")),
    );
    assert!(
        matches!(crit, Ok(RecordEvaluationOutcome::Recorded { .. })),
        "CRIT with a score: {crit:?}"
    );
    let unknown = RecordEvaluationBehavior::record_evaluation(
        &mut system,
        evaluation(id, Verdict::UNKNOWN, Some("0")),
    );
    assert!(
        matches!(
            unknown,
            Ok(RecordEvaluationOutcome::ScoreNotAdmitted { .. })
        ),
        "UNKNOWN with a score: {unknown:?}"
    );
}

/// An id `mint` issued is issued again once the newest row is deleted, because `mint` counts
/// rows rather than remembering what it issued. Design § 2.3 and D13: evaluations and
/// transitions reference a sample or evaluation by id and outlive raw retention.
#[test]
fn an_issued_sample_id_is_never_issued_again() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let _first = recorded(&mut system, "platform.ratio", "s", "2026-01-01T00:00:00Z");
    let newest = recorded(&mut system, "platform.ratio", "s", "2026-01-01T01:00:00Z");
    SignalSampleStorage::delete(&mut system.ports, &newest);
    let next = recorded(&mut system, "platform.ratio", "s", "2026-01-01T02:00:00Z");
    assert_ne!(
        next, newest,
        "a deleted sample's id was issued to a new sample"
    );
}

/// The `evl_` twin of `an_issued_sample_id_is_never_issued_again`: an evaluation id, once
/// issued, is not issued again after the newest evaluation is deleted.
#[test]
fn an_issued_evaluation_id_is_never_issued_again() {
    let mut system = system();
    active(&mut system, "platform.ratio");
    let sample_id = recorded(&mut system, "platform.ratio", "s", "2026-01-01T00:00:00Z");
    let evaluate = |system: &mut System| match RecordEvaluationBehavior::record_evaluation(
        system,
        evaluation(sample_id.clone(), Verdict::OK, None),
    ) {
        Ok(RecordEvaluationOutcome::Recorded { response, .. }) => response.evaluation_id,
        other => panic!("the evaluation was not recorded: {other:?}"),
    };
    let _first = evaluate(&mut system);
    let newest = evaluate(&mut system);
    SignalEvaluationStorage::delete(&mut system.ports, &newest);
    let next = evaluate(&mut system);
    assert_ne!(
        next, newest,
        "a deleted evaluation's id was issued to a new evaluation"
    );
}

/// The `asr_` twin of `an_issued_sample_id_is_never_issued_again`: an assertion id, once
/// issued, is not issued again after the newest assertion is deleted.
#[test]
fn an_issued_assertion_id_is_never_issued_again() {
    let mut system = system();
    let ask = |system: &mut System| match AskAssertionBehavior::ask_assertion(
        system,
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
    ) {
        Ok(AskAssertionOutcome::Asked { response, .. }) => response.assertion_id,
        other => panic!("the assertion was not asked: {other:?}"),
    };
    let _first = ask(&mut system);
    let newest = ask(&mut system);
    AssertionStorage::delete(&mut system.ports, &newest);
    let next = ask(&mut system);
    assert_ne!(
        next, newest,
        "a deleted assertion's id was issued to a new assertion"
    );
}
