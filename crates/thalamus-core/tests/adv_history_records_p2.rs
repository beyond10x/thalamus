//! Adversary pass 2 on `story:m2-spec-history-records`.
//!
//! The coordinator's decisions after pass 1 are attacked here: the definition loader reading
//! `evaluation.fixed`, `evaluation.rolling_n` and `persistence` and refusing what propose and
//! revise refuse (design § 11 D25, D29), the two new `Transition` invariants (D29), and the
//! conformance harness's new `compare` form, which reads them.

use std::collections::BTreeMap;

use thalamus_core::baseline;
use thalamus_core::conformance::Ports;
use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::behaviour::Generated;
use thalamus_model::definition::obligations::ProposeDefinitionBehavior;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links,
    ProposeDefinition, ProposeDefinitionOutcome, RuleId, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{EvaluationId, Verdict};
use thalamus_model::json::{self, Value};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_model::server::wire;
use thalamus_model::transition::obligations::{RecordTransitionBehavior, SeriesTransitionsQuery};
use thalamus_model::transition::{RecordTransition, RecordTransitionOutcome};

type Service = Generated<Ports<MemoryPorts>>;

fn text<T>(wrap: fn(String) -> T, value: &str) -> T {
    wrap(value.to_owned())
}

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const QUEUE: &str = include_str!("fixtures/instance/definitions/service.queue.depth.yaml");
const QUEUE_FIXED: &str = "  baselines: [fixed]\n  fixed: \"0\"\n";

fn load(text: &str) -> Result<Definition, Vec<LoadError>> {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    Definition::parse("definitions/case.yaml", text, &registry)
}

fn rule_names(text: &str) -> Vec<String> {
    match load(text) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .iter()
            .map(|error| error.rule.name().to_owned())
            .collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// The loader's `evaluation.fixed` against the contract's decimal pattern.
// ---------------------------------------------------------------------------------------------

/// The `fixed` fixture with its number replaced by `number`.
fn queue_with_fixed(number: &str) -> String {
    let changed = QUEUE.replace(
        QUEUE_FIXED,
        &format!("  baselines: [fixed]\n  fixed: \"{number}\"\n"),
    );
    assert_ne!(changed, QUEUE, "the fixture declares a fixed baseline");
    changed
}

/// A sample of the `service.queue.depth` fixture, for folding its baseline.
fn queue_sample() -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_000000000001".into()),
        signal: SignalId("service.queue.depth".into()),
        series: SeriesKey("service.queue.depth".into()),
        dims: BTreeMap::new(),
        ts: Timestamp("2026-10-14T22:55:00Z".into()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T22:50:00Z".into()),
            end: Timestamp("2026-10-14T22:55:00Z".into()),
        },
        reading: Reading::Ok(CompleteReading {
            value: Decimal("12".into()),
        }),
        unit: Unit("{message}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("prometheus".into()),
            connection: ConnectionId("metrics-main".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

/// D29: the loader refuses "a number that is not a decimal ... as everywhere else in the file",
/// and refuses what `ProposeDefinition` refuses. D25: a `fixed` baseline without a number it can
/// read is refused (`fixed-number-missing`), because it would grade `UNKNOWN` forever.
///
/// The contract's decimal is "an optional `-`, digits without a leading zero, then an optional
/// `.` and digits" (`generated/thalamus-model/src/json.rs`, `decimal_at`), and the generated
/// wire decoder refuses `"05"` in `evaluation.fixed`, so propose never sees it. The loader's
/// `is_decimal` (`crates/thalamus-core/src/registry.rs`) admits a leading zero, so the file
/// loads, and `baseline::fold` (`Dec::parse`, the same pattern as the wire) reads that `fixed`
/// baseline as `UNKNOWN` on every sample: the state `fixed-number-missing` exists to refuse.
#[test]
fn a_fixed_number_the_contract_refuses_is_refused_at_load() {
    // The contract: the wire decoder of `EvaluationSpec` refuses a leading zero, admits `5`.
    let decode = |number: &str| {
        let body =
            format!(r#"{{"rule": "threshold", "baselines": ["fixed"], "fixed": "{number}"}}"#);
        let value = json::parse(&body).expect("the probe is JSON");
        wire::decode_thalamus_definition_evaluation_spec(&value, "$.evaluation")
    };
    assert!(decode("5").is_ok(), "{:?}", decode("5"));
    assert!(
        decode("05").is_err(),
        "the wire admits 05: {:?}",
        decode("05")
    );

    // Control: a contract decimal loads and folds into its own number.
    let loaded = load(&queue_with_fixed("5")).expect("fixed 5 loads");
    let folded = baseline::fold(loaded.data(), &queue_sample(), &[], None).expect("fixed folds");
    assert_eq!(
        folded.reading,
        Reading::Ok(CompleteReading {
            value: Decimal("5".into())
        })
    );

    for number in ["05", "00", "007.5"] {
        match load(&queue_with_fixed(number)) {
            Err(errors) => assert!(
                errors.iter().any(|error| error.rule.name() == "decimal"),
                "fixed {number}: refused, but not as a decimal: {errors:?}"
            ),
            Ok(loaded) => {
                let folded =
                    baseline::fold(loaded.data(), &queue_sample(), &[], None).expect("fixed folds");
                panic!(
                    "a definition file with fixed {number:?}, which the wire refuses, loaded; \
                     its fixed baseline folds to {:?}",
                    folded.reading
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The loader reports instant-not-fixed once.
// ---------------------------------------------------------------------------------------------

/// `input_refusals` includes `instant-not-fixed`, and the loader already refuses it as
/// `Rule::InstantNeedsFixed`, so it skips that refusal in the shared list
/// (`registry.rs`, `if refusal != InputRefusal::InstantNotFixed`). The registry tests check only
/// that some error is `InstantNeedsFixed` (`refused` uses `any`), and the conformance table has
/// no instant case, so dropping the skip, which reports the one broken rule twice, was not seen.
#[test]
fn an_instant_with_another_baseline_is_refused_once() {
    let offset = QUEUE.replace(QUEUE_FIXED, "  baselines: [offset_1d]\n");
    assert_ne!(offset, QUEUE);
    let proposed = offset
        .replace("lifecycle: muted\n", "lifecycle: proposed\n")
        .replace("mute_reason: queue migration in progress\n", "")
        .replace("mute_until: 2026-11-01T00:00:00Z\n", "");
    assert_ne!(proposed, offset);
    for file in [&offset, &proposed] {
        assert_eq!(
            rule_names(file),
            ["thalamus.definition.InstantNeedsFixedBaseline"],
            "{file}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The Transition invariants, over instants written in different offsets.
// ---------------------------------------------------------------------------------------------

fn service() -> Service {
    Generated::new(Ports::new(
        MemoryPorts::default(),
        Some(text(Timestamp, "2026-01-01T00:00:00Z")),
    ))
}

fn define(service: &mut Service) {
    let proposal = ProposeDefinition {
        signal_id: text(SignalId, "platform.ratio"),
        title: "Failure ratio".to_owned(),
        family: text(Family, "platform"),
        kind: SignalKind::Gauge,
        unit: Some(text(Unit, "s")),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: text(AdapterId, "sql"),
            connection: text(ConnectionId, "replica"),
        },
        extractor: text(ExtractorId, "sql.gauge"),
        query: BTreeMap::new(),
        window: text(Duration, "PT6H"),
        cadence: text(Duration, "PT1H"),
        history: History::Source,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: text(RuleId, "threshold"),
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
        owner: None,
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        author: text(Author, "author"),
    };
    let outcome =
        ProposeDefinitionBehavior::propose_definition(service, proposal).expect("propose answers");
    assert!(
        matches!(outcome, ProposeDefinitionOutcome::Proposed { .. }),
        "{outcome:?}"
    );
}

fn transition(first_ts: &str, settled_ts: &str) -> RecordTransition {
    RecordTransition {
        signal: text(SignalId, "platform.ratio"),
        series: text(SeriesKey, "s"),
        dims: BTreeMap::from([("cluster".to_owned(), text(DimensionValue, "a"))]),
        definition_rev: 1,
        from_verdict: Some(Verdict::OK),
        to_verdict: Verdict::WARN,
        first_ts: text(Timestamp, first_ts),
        settled_ts: text(Timestamp, settled_ts),
        settling_score: None,
        evaluations: vec![text(EvaluationId, "evl_1")],
    }
}

/// D13, D29: `first_ts <= settled_ts` compares instants. A settling instant written in another
/// offset, or with a fraction, is ordered by the instant it names, not by its spelling: by bytes
/// the two refused pairs below are in order, and the admitted pair is not.
#[test]
fn a_transition_is_ordered_by_instant_whatever_its_spelling() {
    for (first, settled) in [
        // 23:00Z the day before, after midnight by bytes.
        ("2026-01-01T00:00:00Z", "2026-01-01T01:00:00+02:00"),
        // half a second after settled_ts; "." sorts before "Z".
        ("2026-01-01T00:00:00.5Z", "2026-01-01T00:00:00Z"),
    ] {
        let mut service = service();
        define(&mut service);
        let answer =
            RecordTransitionBehavior::record_transition(&mut service, transition(first, settled));
        match answer {
            Err(unmet) => assert_eq!(unmet.source, "first_ts <= settled_ts", "{unmet:?}"),
            Ok(outcome) => panic!("first_ts {first} after settled_ts {settled}: {outcome:?}"),
        }
        let rows = SeriesTransitionsQuery::series_transitions(&service, text(SeriesKey, "s"))
            .expect("the view answers");
        assert!(rows.is_empty(), "{rows:?}");
    }

    // 23:30Z the day before is before midnight, although it sorts after it by bytes.
    let mut service = service();
    define(&mut service);
    let answer = RecordTransitionBehavior::record_transition(
        &mut service,
        transition("2026-01-01T01:30:00+02:00", "2026-01-01T00:00:00Z"),
    );
    assert!(
        matches!(answer, Ok(RecordTransitionOutcome::Recorded { .. })),
        "{answer:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The harness's `compare` form, over two timestamps that differ.
// ---------------------------------------------------------------------------------------------

const INVARIANT: &str =
    "thalamus.transition.Transition/invariant/after/thalamus.transition.RecordTransition/recorded";

fn literal_text(value: &str) -> Value {
    Value::Object(vec![
        ("kind".to_owned(), Value::Text("literal".to_owned())),
        ("value".to_owned(), Value::Text(value.to_owned())),
    ])
}

/// The suite's transition invariant scenario with its `RecordTransition` input moved to
/// `first_ts` 2026-01-01T01:30:00+02:00 (23:30Z the day before) and `settled_ts`
/// 2026-01-01T00:00:00Z, its transition view expectations dropped, and one `satisfies` over
/// `predicate` on `SeriesTransitions` added.
fn transition_probe(predicate: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    let suite = json::parse(&std::fs::read_to_string(&path).expect("the suite reads"))
        .expect("the suite is JSON");
    let scenario = suite
        .member("scenarios")
        .and_then(|all| all.member(INVARIANT))
        .expect("the suite holds the transition invariant scenario");
    let Some(Value::Array(steps)) = scenario.member("steps") else {
        panic!("the scenario has no steps");
    };
    let mut kept = Vec::new();
    let mut moved = 0;
    for step in steps {
        let view = match step.member("view") {
            Some(Value::Text(view)) => Some(view.as_str()),
            _ => None,
        };
        if view.is_some_and(|view| view.starts_with("thalamus.transition.")) {
            continue;
        }
        let records = matches!(step.member("command"), Some(Value::Text(command)) if command == "thalamus.transition.RecordTransition");
        let Value::Object(members) = step else {
            panic!("a step is not an object");
        };
        let mut members = members.clone();
        if records {
            for (name, value) in &mut members {
                let Value::Object(input) = value else {
                    continue;
                };
                if name != "input" {
                    continue;
                }
                for (field, given) in input.iter_mut() {
                    match field.as_str() {
                        "first_ts" => *given = literal_text("2026-01-01T01:30:00+02:00"),
                        "settled_ts" => *given = literal_text("2026-01-01T00:00:00Z"),
                        _ => continue,
                    }
                    moved += 1;
                }
            }
        }
        kept.push(Value::Object(members));
    }
    assert_eq!(moved, 2, "the scenario records one transition");
    let mut out = String::new();
    for step in &kept {
        out.push_str(if out.is_empty() { "" } else { "," });
        json::push_value(&mut out, step);
    }
    let mut provenance = String::new();
    json::push_value(
        &mut provenance,
        suite
            .member("provenance")
            .expect("the suite has a provenance"),
    );
    format!(
        r#"{{"provenance": {provenance}, "scenarios": {{"probe": {{"purpose": "p", "source": [], "steps": [{out},
          {{"step": "query_view", "view": "thalamus.transition.SeriesTransitions", "params": {{"series": {{"kind": "literal", "value": "series"}}}}}},
          {{"step": "expect_view", "view": "thalamus.transition.SeriesTransitions", "expectation": {{"expect": "satisfies", "predicate": {predicate}}}}}
        ]}}}}}}"#
    )
}

fn compare(left: &str, op: &str, right: &str) -> String {
    format!(
        r#"{{"compare": {{"as": "timestamp", "left": "{left}", "op": "{op}", "right": {{"fact": "{right}"}}}}}}"#
    )
}

fn probe_holds(predicate: &str) -> bool {
    let report = thalamus_core::conformance::run(
        &transition_probe(predicate),
        &|_| true,
        MemoryPorts::default,
    )
    .expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    report.failed.is_empty()
}

/// The harness's new `{compare: {as: timestamp, left, op, right: {fact}}}` form
/// (`conformance.rs`, `compare_truth`) is reached by the suite only with `first_ts` equal to
/// `settled_ts` (both `2020-01-01T00:00:00Z`) under `lte`, so an operand swap, an ignored `op`,
/// a byte order, or a constant `true` all pass it. Over two instants that differ, and whose byte
/// order is the reverse of their instant order, each keyword answers as the instants do.
#[test]
fn the_harness_compares_two_timestamp_fields_as_instants() {
    for (predicate, holds) in [
        (compare("first_ts", "lt", "settled_ts"), true),
        (compare("first_ts", "lte", "settled_ts"), true),
        (compare("first_ts", "gte", "settled_ts"), false),
        (compare("first_ts", "eq", "settled_ts"), false),
        (compare("first_ts", "ne", "settled_ts"), true),
        (compare("settled_ts", "gt", "first_ts"), true),
        (compare("settled_ts", "lte", "first_ts"), false),
    ] {
        assert_eq!(probe_holds(&predicate), holds, "{predicate}");
    }
}
