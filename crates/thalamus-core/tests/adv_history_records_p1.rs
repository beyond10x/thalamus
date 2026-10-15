//! Adversary pass 1 on `story:m2-spec-history-records`.
//!
//! Each case drives the unit's code from a document outside it: design § 3.2 (a transition is a
//! verdict *change*, settled once k evaluations agree), design § 11 D13 (the first agreeing
//! evaluation precedes the settling one), D15 and D25 (persistence, `fixed` and `rolling_n` are
//! fields of the definition) and D19 (the definition file is the only copy of a definition), and
//! the two series views' own `filter` and `order_by`.

use std::collections::BTreeMap;

use thalamus_core::conformance::Ports;
use thalamus_core::registry::{Definition, LoadError, Registry};
use thalamus_model::behaviour::{Generated, SignalDefinitionStorage};
use thalamus_model::definition::obligations::{
    ProposeDefinitionBehavior, ReviseDefinitionBehavior,
};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, Persistence, ProposeDefinition, ProposeDefinitionOutcome,
    QueryText, ReviseDefinition, ReviseDefinitionOutcome, RuleId, SignalId, SignalKind, Source,
    Unit,
};
use thalamus_model::dimension::{DimensionKey, DimensionValue};
use thalamus_model::evaluation::{EvaluationId, Verdict};
use thalamus_model::observation::SeriesKey;
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::rollup::obligations::{RecordRollupBehavior, SeriesRollupsQuery};
use thalamus_model::rollup::{RecordRollup, RecordRollupOutcome, RollupId, VerdictCounts};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_model::transition::obligations::{RecordTransitionBehavior, SeriesTransitionsQuery};
use thalamus_model::transition::{RecordTransition, RecordTransitionOutcome};

type Service = Generated<Ports<MemoryPorts>>;

fn text<T>(wrap: fn(String) -> T, value: &str) -> T {
    wrap(value.to_owned())
}

fn service() -> Service {
    Generated::new(Ports::new(
        MemoryPorts::default(),
        Some(text(Timestamp, "2026-01-01T00:00:00Z")),
    ))
}

fn evaluation_spec() -> EvaluationSpec {
    EvaluationSpec {
        rule: text(RuleId, "threshold"),
        baselines: Vec::new(),
        warn: None,
        crit: None,
        min_trend_window: None,
        fixed: None,
        rolling_n: None,
        mapping: None,
    }
}

fn propose(signal: &str, persistence: Option<Persistence>) -> ProposeDefinition {
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
        evaluation: evaluation_spec(),
        persistence,
        objective: None,
        owner: Some(text(Owner, "team")),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        author: text(Author, "author"),
    }
}

fn revise(signal: &str, persistence: Option<Persistence>) -> ReviseDefinition {
    let proposal = propose(signal, persistence);
    ReviseDefinition {
        signal_id: proposal.signal_id,
        title: "Failure ratio, revised".to_owned(),
        family: proposal.family,
        kind: proposal.kind,
        unit: text(Unit, "1"),
        dimensions: vec![text(DimensionKey, "cluster")],
        source: proposal.source,
        extractor: proposal.extractor,
        query: proposal.query,
        window: proposal.window,
        cadence: proposal.cadence,
        history: proposal.history,
        denominator: proposal.denominator.expect("proposed with a denominator"),
        evaluation: proposal.evaluation,
        persistence: proposal.persistence,
        objective: None,
        owner: text(Owner, "team"),
        links: proposal.links,
        author: text(Author, "reviser"),
    }
}

fn defined(service: &mut Service, signal: &str) {
    let outcome = ProposeDefinitionBehavior::propose_definition(service, propose(signal, None))
        .expect("propose answers");
    assert!(
        matches!(outcome, ProposeDefinitionOutcome::Proposed { .. }),
        "{outcome:?}"
    );
}

fn transition(
    series: &str,
    from: Option<Verdict>,
    to: Verdict,
    first_ts: &str,
    settled_ts: &str,
) -> RecordTransition {
    RecordTransition {
        signal: text(SignalId, "platform.ratio"),
        series: text(SeriesKey, series),
        dims: BTreeMap::from([("cluster".to_owned(), text(DimensionValue, "a"))]),
        definition_rev: 1,
        from_verdict: from,
        to_verdict: to,
        first_ts: text(Timestamp, first_ts),
        settled_ts: text(Timestamp, settled_ts),
        settling_score: Some(text(Decimal, "1.3")),
        evaluations: vec![text(EvaluationId, "evl_1"), text(EvaluationId, "evl_2")],
    }
}

fn recorded(outcome: &Result<RecordTransitionOutcome, impl std::fmt::Debug>) -> bool {
    matches!(outcome, Ok(RecordTransitionOutcome::Recorded { .. }))
}

// ---------------------------------------------------------------------------------------------
// The transition record against design § 3.2 and § 11 D13.
// ---------------------------------------------------------------------------------------------

/// Design § 3.2: "A verdict change counts as a **transition** only after `k` consecutive
/// evaluations agree"; the domain's own summary: "A settled change of a series' verdict". A
/// record from `OK` to `OK` is no change, and the specification can say so: ESS 0.60.0 validates
/// and synthesizes `any: ["not defined(from_verdict)", "from_verdict != to_verdict"]` on
/// `Transition`. Today the record is accepted, and the suite's own `recorded` scenario records
/// exactly `OK` -> `OK`.
#[test]
fn a_transition_from_a_verdict_to_itself_is_not_recorded() {
    let mut service = service();
    defined(&mut service, "platform.ratio");

    let same = RecordTransitionBehavior::record_transition(
        &mut service,
        transition(
            "s",
            Some(Verdict::OK),
            Verdict::OK,
            "2026-01-01T00:00:00Z",
            "2026-01-01T01:00:00Z",
        ),
    );
    assert!(
        !recorded(&same),
        "an OK -> OK transition was recorded: {same:?}"
    );

    // The first settled verdict of a series has no `from` and is a transition.
    let first = RecordTransitionBehavior::record_transition(
        &mut service,
        transition(
            "s",
            None,
            Verdict::OK,
            "2026-01-01T00:00:00Z",
            "2026-01-01T01:00:00Z",
        ),
    );
    assert!(recorded(&first), "{first:?}");
}

/// Design § 11 D13 and the field summaries: `first_ts` is "the ts of the first of the agreeing
/// evaluations", `settled_ts` "the ts of the settling evaluation", the last of them. A transition
/// that settled before its first agreeing evaluation was taken is impossible; ESS 0.60.0
/// validates and synthesizes `first_ts <= settled_ts` (a timestamp comparison), and the
/// specification does not declare it.
#[test]
fn a_transition_does_not_settle_before_its_first_agreeing_evaluation() {
    let mut service = service();
    defined(&mut service, "platform.ratio");

    let backwards = RecordTransitionBehavior::record_transition(
        &mut service,
        transition(
            "s",
            Some(Verdict::OK),
            Verdict::WARN,
            "2026-01-02T00:00:00Z",
            "2026-01-01T00:00:00Z",
        ),
    );
    assert!(
        !recorded(&backwards),
        "a transition settled a day before its first agreeing evaluation: {backwards:?}"
    );
}

/// The one guard `record_transition` writes itself: the entity invariant `definition_rev >= 1`,
/// which no declared outcome refuses, answers `UnmetObligation` and stores nothing. No scenario
/// and no other test sends a revision below 1, so deleting the guard left every test green.
#[test]
fn a_transition_below_revision_one_is_unmet_and_stores_nothing() {
    let mut service = service();
    defined(&mut service, "platform.ratio");
    let mut input = transition(
        "s",
        None,
        Verdict::WARN,
        "2026-01-01T00:00:00Z",
        "2026-01-01T01:00:00Z",
    );
    input.definition_rev = 0;

    let answer = RecordTransitionBehavior::record_transition(&mut service, input);
    match answer {
        Err(unmet) => assert_eq!(unmet.capability, "entity invariant", "{unmet:?}"),
        Ok(outcome) => panic!("a transition at revision 0 answered {outcome:?}"),
    }
    let rows = SeriesTransitionsQuery::series_transitions(&service, text(SeriesKey, "s"))
        .expect("the view answers");
    assert!(
        rows.is_empty(),
        "the refused transition was stored: {rows:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The harness's field-to-field comparison, over values that differ.
// ---------------------------------------------------------------------------------------------

/// The suite's rollup invariant scenario, its `RecordRollup` input moved to `n` 3, `unknown` 1,
/// `partial` 2, `min` 9 and `max` 10, and its `SeriesRollups` expectations replaced by one
/// `satisfies` over `predicate`.
fn rollup_probe(predicate: &str) -> String {
    use thalamus_model::json::{self, Value};
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    let suite = json::parse(&std::fs::read_to_string(&path).expect("the suite reads"))
        .expect("the suite is JSON");
    let scenario = suite
        .member("scenarios")
        .and_then(|all| {
            all.member(
                "thalamus.rollup.DailyRollup/invariant/after/thalamus.rollup.RecordRollup/recorded",
            )
        })
        .expect("the suite holds the rollup invariant scenario");
    let Some(Value::Array(steps)) = scenario.member("steps") else {
        panic!("the scenario has no steps");
    };
    let literal = |number: &str| {
        Value::Object(vec![
            ("kind".to_owned(), Value::Text("literal".to_owned())),
            ("value".to_owned(), Value::Number(number.to_owned())),
        ])
    };
    let mut kept = Vec::new();
    for step in steps {
        let view = step.member("view").and_then(|view| match view {
            Value::Text(view) => Some(view.as_str()),
            _ => None,
        });
        if view == Some("thalamus.rollup.SeriesRollups") {
            continue;
        }
        let records = matches!(step.member("command"), Some(Value::Text(command)) if command == "thalamus.rollup.RecordRollup");
        let Value::Object(members) = step else {
            panic!("a step is not an object");
        };
        let mut members = members.clone();
        if records {
            for (name, value) in &mut members {
                if name != "input" {
                    continue;
                }
                let Value::Object(input) = value else {
                    panic!("the input is not an object");
                };
                for (field, given) in input.iter_mut() {
                    match field.as_str() {
                        "n" => *given = literal("3"),
                        "unknown" => *given = literal("1"),
                        "partial" => *given = literal("2"),
                        "min" => *given = literal("9"),
                        "max" => *given = literal("10"),
                        _ => {}
                    }
                }
            }
        }
        kept.push(Value::Object(members));
    }
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
          {{"step": "query_view", "view": "thalamus.rollup.SeriesRollups", "params": {{"series": {{"kind": "literal", "value": "series"}}}}}},
          {{"step": "expect_view", "view": "thalamus.rollup.SeriesRollups", "expectation": {{"expect": "satisfies", "predicate": {predicate}}}}}
        ]}}}}}}"#
    )
}

fn probe_holds(predicate: &str) -> bool {
    let report =
        thalamus_core::conformance::run(&rollup_probe(predicate), &|_| true, MemoryPorts::default)
            .expect("the probe suite runs");
    assert_eq!(report.ran, ["probe"]);
    report.failed.is_empty()
}

/// `{field: {op: {fact: other}}}` under every ordering keyword, over two Integer fields that
/// differ (1 < 3) and over a field and itself (3 = 3). The suite's own two such predicates compare
/// equal values only (`unknown` and `partial` against `n`, all 1), so swapping `lt` for `lte`,
/// `gt` for `gte`, `gte` for `gt`, or the operands of the number comparison, left every test
/// green.
#[test]
fn the_harness_orders_two_integer_fields() {
    for (predicate, holds) in [
        (r#"{"unknown": {"lt": {"fact": "n"}}}"#, true),
        (r#"{"n": {"lt": {"fact": "unknown"}}}"#, false),
        (r#"{"n": {"lt": {"fact": "n"}}}"#, false),
        (r#"{"n": {"gt": {"fact": "partial"}}}"#, true),
        (r#"{"partial": {"gt": {"fact": "n"}}}"#, false),
        (r#"{"n": {"gt": {"fact": "n"}}}"#, false),
        (r#"{"n": {"gte": {"fact": "n"}}}"#, true),
        (r#"{"unknown": {"gte": {"fact": "partial"}}}"#, false),
        (r#"{"partial": {"lte": {"fact": "n"}}}"#, true),
        (r#"{"n": {"lte": {"fact": "partial"}}}"#, false),
    ] {
        assert_eq!(probe_holds(predicate), holds, "{predicate}");
    }
}

/// Two `Decimal` fields, which the wire carries as decimal strings (design § 11 D11), compare by
/// value and not by their spelling: 9 < 10, although the text "9" sorts after "10".
#[test]
fn the_harness_orders_two_decimal_fields_by_value() {
    assert!(probe_holds(r#"{"min": {"lt": {"fact": "max"}}}"#));
    assert!(!probe_holds(r#"{"max": {"lt": {"fact": "min"}}}"#));
}

// ---------------------------------------------------------------------------------------------
// The two series views: their `filter` and `order_by`.
// ---------------------------------------------------------------------------------------------

/// `SeriesTransitions`: `filter: series == param.series`, `order_by: [settled_ts asc]`. Two
/// series, recorded out of time order: each read answers its own series only, oldest first.
#[test]
fn series_transitions_answer_one_series_oldest_first() {
    let mut service = service();
    defined(&mut service, "platform.ratio");
    for (series, from, to, settled) in [
        (
            "b",
            Some(Verdict::OK),
            Verdict::WARN,
            "2026-01-02T00:00:00Z",
        ),
        ("a", None, Verdict::OK, "2026-01-03T00:00:00Z"),
        ("b", None, Verdict::OK, "2026-01-01T00:00:00Z"),
    ] {
        let outcome = RecordTransitionBehavior::record_transition(
            &mut service,
            transition(series, from, to, "2025-12-31T00:00:00Z", settled),
        );
        assert!(recorded(&outcome), "{outcome:?}");
    }

    let rows = SeriesTransitionsQuery::series_transitions(&service, text(SeriesKey, "b"))
        .expect("the view answers");
    let settled: Vec<_> = rows.iter().map(|row| row.settled_ts.0.as_str()).collect();
    assert_eq!(settled, ["2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z"]);
    assert_eq!(rows[1].from_verdict, Some(Verdict::OK));
    assert_eq!(rows[1].to_verdict, Verdict::WARN);

    let rows = SeriesTransitionsQuery::series_transitions(&service, text(SeriesKey, "a"))
        .expect("the view answers");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].settled_ts.0, "2026-01-03T00:00:00Z");
}

fn rollup(id: &str, series: &str, day: &str, n: i64) -> RecordRollup {
    RecordRollup {
        rollup_id: text(RollupId, id),
        signal: text(SignalId, "platform.ratio"),
        series: text(SeriesKey, series),
        day: text(Timestamp, day),
        n,
        unknown: 0,
        partial: 0,
        min: None,
        p50: None,
        max: None,
        last: None,
        worst_verdict: None,
        verdict_counts: VerdictCounts {
            ok: 0,
            na: 0,
            warn: 0,
            crit: 0,
            unknown: 0,
        },
        min_score: None,
        max_score: None,
    }
}

/// `SeriesRollups`: `filter: series == param.series`, `order_by: [day asc]`. The ids are chosen
/// so that id order is the reverse of day order, and a second series sits between them.
#[test]
fn series_rollups_answer_one_series_oldest_day_first() {
    let mut service = service();
    defined(&mut service, "platform.ratio");
    for (id, series, day, n) in [
        ("x-1", "b", "2026-01-02T00:00:00Z", 2),
        ("x-2", "a", "2026-01-03T00:00:00Z", 3),
        ("x-3", "b", "2026-01-01T00:00:00Z", 1),
    ] {
        let outcome = RecordRollupBehavior::record_rollup(&mut service, rollup(id, series, day, n))
            .expect("record answers");
        assert!(
            matches!(outcome, RecordRollupOutcome::Recorded { .. }),
            "{outcome:?}"
        );
    }

    let rows = SeriesRollupsQuery::series_rollups(&service, text(SeriesKey, "b"))
        .expect("the view answers");
    let days: Vec<_> = rows.iter().map(|row| (row.day.0.as_str(), row.n)).collect();
    assert_eq!(
        days,
        [("2026-01-01T00:00:00Z", 1), ("2026-01-02T00:00:00Z", 2)]
    );
    let rows = SeriesRollupsQuery::series_rollups(&service, text(SeriesKey, "a"))
        .expect("the view answers");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].rollup_id.0, "x-2");
}

// ---------------------------------------------------------------------------------------------
// Persistence is kept by propose and revise (D15).
// ---------------------------------------------------------------------------------------------

/// D15: `persistence: {k, k_crit, m}` lives on the definition. No view publishes it, so no
/// scenario of the suite can see whether propose and revise keep it; this reads the store.
#[test]
fn propose_and_revise_keep_the_persistence_they_are_given() {
    let mut service = service();
    let held = |service: &Service| {
        SignalDefinitionStorage::get(&service.ports, &text(SignalId, "platform.p"))
            .expect("the definition is stored")
            .data
            .persistence
    };
    let given = |k, k_crit, m| Persistence { k, k_crit, m };

    let proposed = ProposeDefinitionBehavior::propose_definition(
        &mut service,
        propose("platform.p", Some(given(3, 2, 4))),
    )
    .expect("propose answers");
    assert!(matches!(
        proposed,
        ProposeDefinitionOutcome::Proposed { .. }
    ));
    assert_eq!(held(&service), Some(given(3, 2, 4)));

    let revised = ReviseDefinitionBehavior::revise_definition(
        &mut service,
        revise("platform.p", Some(given(5, 1, 6))),
    )
    .expect("revise answers");
    assert!(matches!(revised, ReviseDefinitionOutcome::Revised { .. }));
    assert_eq!(held(&service), Some(given(5, 1, 6)));

    let cleared =
        ReviseDefinitionBehavior::revise_definition(&mut service, revise("platform.p", None))
            .expect("revise answers");
    assert!(matches!(cleared, ReviseDefinitionOutcome::Revised { .. }));
    assert_eq!(held(&service), None);
}

// ---------------------------------------------------------------------------------------------
// Boundaries of the two new counts: N of rolling_median, and k, k_crit, m.
// ---------------------------------------------------------------------------------------------

/// Design § 3.1: `rolling_median` is "median of the last N samples of this series". A median of
/// the last 0, or of the last -3, samples has no meaning; `rolling-n-missing` refuses an absent N
/// and admits these. ESS states the guard the same way it states `rolling-n-missing`
/// (`when: evaluation.rolling_n < 1`).
#[test]
fn a_rolling_median_over_no_samples_is_not_proposed() {
    for n in [0, -3] {
        let mut service = service();
        let mut proposal = propose("platform.rolling", None);
        proposal.evaluation.baselines = vec![BaselineKind::RollingMedian];
        proposal.evaluation.rolling_n = Some(n);
        let outcome = ProposeDefinitionBehavior::propose_definition(&mut service, proposal)
            .expect("propose answers");
        assert!(
            !matches!(outcome, ProposeDefinitionOutcome::Proposed { .. }),
            "rolling_n {n} was proposed"
        );
    }
}

/// Design § 3.2: a change settles "only after `k` consecutive evaluations agree"; a `WARN`
/// "clears after `m` `OK` evaluations". k, k_crit or m of 0 or below settles or clears on no
/// evaluation at all, which is the flapping § 3.2 exists to stop. The specification admits it.
#[test]
fn a_persistence_of_no_evaluations_is_not_proposed() {
    for (k, k_crit, m) in [(0, 1, 3), (2, 0, 3), (2, 1, 0), (-1, 1, 3)] {
        let mut service = service();
        let outcome = ProposeDefinitionBehavior::propose_definition(
            &mut service,
            propose("platform.p", Some(Persistence { k, k_crit, m })),
        )
        .expect("propose answers");
        assert!(
            !matches!(outcome, ProposeDefinitionOutcome::Proposed { .. }),
            "persistence k {k}, k_crit {k_crit}, m {m} was proposed"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The definition file, the only copy of a definition (D19), carries the new fields.
// ---------------------------------------------------------------------------------------------

const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
const QUEUE: &str = include_str!("fixtures/instance/definitions/service.queue.depth.yaml");

fn load(text: &str) -> Result<Definition, Vec<LoadError>> {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    Definition::parse("definitions/case.yaml", text, &registry)
}

fn shown(errors: &[LoadError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// D25: "a `fixed` baseline is the definition's own number, `evaluation.fixed`", and
/// `baseline::fold` now reads it from nowhere else. D19: the definition file is the only copy of
/// a definition. So the file must be able to carry the number; today the loader refuses the
/// field (`deny_unknown_fields`) and sets it to `None`, which makes every `fixed` baseline loaded
/// from a file `UNKNOWN` (D12's advisory gates are graded against exactly that baseline).
#[test]
fn a_definition_file_carries_the_number_of_its_fixed_baseline() {
    // Coordinator decision 4 (D25): a fixed baseline without its number is refused, so the
    // fixture now carries `fixed: "0"`; the case replaces that number with another.
    let with_number = QUEUE.replace(
        "  baselines: [fixed]\n  fixed: \"0\"\n",
        "  baselines: [fixed]\n  fixed: \"100\"\n",
    );
    assert_ne!(with_number, QUEUE, "the fixture names a fixed baseline");
    let loaded = load(&with_number).unwrap_or_else(|errors| {
        panic!(
            "a definition declaring evaluation.fixed does not load:\n{}",
            shown(&errors)
        )
    });
    assert_eq!(loaded.data().evaluation.fixed, Some(text(Decimal, "100")));
}

/// D15 and `story:m2-hysteresis-core` ("read from the definition's generated `persistence`
/// field"): the definition file carries `persistence: {k, k_crit, m}`.
#[test]
fn a_definition_file_carries_its_persistence() {
    let with_persistence = QUEUE.replace(
        "owner: team:service\n",
        "persistence: { k: 3, k_crit: 2, m: 4 }\nowner: team:service\n",
    );
    assert_ne!(with_persistence, QUEUE);
    let loaded = load(&with_persistence).unwrap_or_else(|errors| {
        panic!(
            "a definition declaring persistence does not load:\n{}",
            shown(&errors)
        )
    });
    assert_eq!(
        loaded.data().persistence,
        Some(Persistence {
            k: 3,
            k_crit: 2,
            m: 4
        })
    );
}

/// A proposed (not yet sampled) store-history gauge graded against `rolling_median`.
fn rolling(rolling_n_line: &str) -> String {
    format!(
        "id: service.queue.age\n\
         title: Age of the oldest waiting message\n\
         family: service\n\
         kind: gauge\n\
         unit: s\n\
         dimensions: []\n\
         source: {{ adapter: prometheus, connection: metrics-main }}\n\
         extractor: promql.instant\n\
         query: {{ value: max(queue_oldest_age_seconds) }}\n\
         window: PT5M\n\
         cadence: PT5M\n\
         history: store\n\
         denominator: {{ unit: \"{{message}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}\n\
         evaluation:\n\
         \x20 rule: threshold\n\
         \x20 baselines: [rolling_median]\n\
         {rolling_n_line}\
         \x20 warn: {{ x: 2 }}\n\
         owner: team:service\n\
         links: {{ components: [], issues: [] }}\n\
         lifecycle: proposed\n\
         rev: 1\n\
         revised_at: 2026-10-02T00:00:00Z\n\
         revised_by: operator\n"
    )
}

/// D25: `evaluation.rolling_n` is the N of `rolling_median`, carried by the definition.
#[test]
fn a_definition_file_carries_the_n_of_its_rolling_median() {
    let loaded = load(&rolling("  rolling_n: 12\n")).unwrap_or_else(|errors| {
        panic!(
            "a definition declaring evaluation.rolling_n does not load:\n{}",
            shown(&errors)
        )
    });
    assert_eq!(loaded.data().evaluation.rolling_n, Some(12));
}

/// D25: "`ProposeDefinition` and `ReviseDefinition` refuse a definition whose baselines name
/// `rolling_median` without `rolling_n`". Under D19 the file is how a definition enters the
/// service, and the loader admits exactly what the commands refuse.
#[test]
fn a_definition_file_naming_rolling_median_without_its_n_is_refused() {
    let loaded = load(&rolling(""));
    assert!(
        loaded.is_err(),
        "a rolling_median definition without rolling_n loaded"
    );
}
