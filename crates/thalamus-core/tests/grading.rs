//! `story:m1-verdict-grading`: one reading graded against one baseline yields exactly one verdict
//! of the ladder, its score and its reason (design § 2.3, § 3.1 rule 4, § 3.3), and an unknown
//! reading is never `OK` (L3, L5).

use std::collections::BTreeMap;

use thalamus_core::behaviour::evaluation::record_evaluation;
use thalamus_core::conformance::Ports;
use thalamus_core::grading::{self, Graded, REASON_PARTIAL, Refusal};
use thalamus_core::ladder;
use thalamus_core::rules::RuleId;
use thalamus_model::behaviour::SignalSampleStorage;
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId as ModelRuleId, SignalDefinitionData, SignalId,
    SignalKind, Source, Threshold, Unit,
};
use thalamus_model::evaluation::{
    Baseline, BaselineLabel, RecordEvaluation, RecordEvaluationOutcome, Verdict,
};
use thalamus_model::observation::{
    AnySignalSample, Bound, CompleteReading, Denominator, PartialReading, Provenance, QueryDigest,
    Reading, RunId, SampleId, SeriesKey, SignalSample, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_model::server::memory::MemoryPorts;

const RATIO: &str = "ratio_vs_pooled_baseline";
const HIGHER: &str = "higher_is_worse";
const LOWER: &str = "lower_is_worse";

fn line(x: Option<&str>, abs: Option<&str>, min_events: Option<i64>) -> Threshold {
    Threshold {
        x: x.map(|x| Decimal(x.into())),
        abs: abs.map(|abs| Decimal(abs.into())),
        min_events,
    }
}

/// The design's own example (§ 2.1): `warn: {x: 2, abs: 0.02, min_events: 10}`,
/// `crit: {x: 4, abs: 0.05}`.
fn design_lines() -> (Threshold, Threshold) {
    (
        line(Some("2"), Some("0.02"), Some(10)),
        line(Some("4"), Some("0.05"), None),
    )
}

fn define(rule: &str, kind: SignalKind, warn: Threshold, crit: Threshold) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.namespace.request_failure_ratio".into()),
        title: "Share of requests the platform failed".into(),
        family: Family("platform".into()),
        kind,
        unit: Some(Unit("1".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("platform-replica".into()),
        },
        extractor: ExtractorId("sql.ratio".into()),
        query: BTreeMap::new(),
        window: Duration("PT6H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Source,
        denominator: Some(DenominatorSpec {
            unit: Unit("{request}".into()),
            min_n: 20,
            since: "first_sample".into(),
        }),
        evaluation: EvaluationSpec {
            rule: ModelRuleId(rule.into()),
            baselines: vec![BaselineKind::Offset1d, BaselineKind::Offset7d],
            warn: Some(warn),
            crit: Some(crit),
            min_trend_window: None,
            fixed: None,
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:platform".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-10-01T00:00:00Z".into()),
        revised_by: Author("someone".into()),
        mute_reason: None,
        mute_until: None,
    }
}

fn ratio_definition() -> SignalDefinitionData {
    let (warn, crit) = design_lines();
    define(RATIO, SignalKind::Ratio, warn, crit)
}

fn sample(reading: Reading, n: Option<i64>) -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_1".into()),
        signal: SignalId("platform.namespace.request_failure_ratio".into()),
        series: SeriesKey("platform.namespace.request_failure_ratio".into()),
        dims: BTreeMap::new(),
        ts: Timestamp("2026-10-14T22:57:03Z".into()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T16:57:03Z".into()),
            end: Timestamp("2026-10-14T22:57:03Z".into()),
        },
        reading,
        unit: Unit("1".into()),
        denominator: n.map(|n| Denominator {
            n,
            unit: Unit("{request}".into()),
            since: Timestamp("2025-03-02T00:00:00Z".into()),
            age_s: 51_403_023,
        }),
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("platform-replica".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn partial(value: &str, bound: Bound) -> Reading {
    Reading::Partial(PartialReading {
        value: Decimal(value.into()),
        bound,
    })
}

fn unread(reason: &str) -> Reading {
    Reading::Unknown(UnreadReading {
        reason: reason.into(),
    })
}

fn baseline(reading: Reading) -> Baseline {
    Baseline {
        kind: BaselineLabel("pooled".into()),
        offsets: vec!["1d".into(), "7d".into()],
        reading,
    }
}

fn pooled(value: &str) -> Baseline {
    baseline(ok(value))
}

fn grade(
    definition: &SignalDefinitionData,
    reading: Reading,
    n: Option<i64>,
    base: &Baseline,
) -> Graded {
    grading::grade(definition, &sample(reading, n), base).expect("the definition grades")
}

fn score(graded: &Graded) -> Option<&str> {
    graded.score.as_ref().map(|score| score.0.as_str())
}

fn score_value(graded: &Graded) -> f64 {
    score(graded)
        .unwrap_or_else(|| panic!("{:?} carries no score", graded.verdict))
        .parse()
        .expect("a score is a decimal")
}

/// The band of the score each scored verdict admits (design § 3.3): below 1 is `OK`, 1 up to 2
/// is `WARN`, 2 and over is `CRIT`.
fn assert_agrees(graded: &Graded, context: &str) {
    match graded.verdict {
        Verdict::OK => assert!(score_value(graded) < 1.0, "{context}: {graded:?}"),
        Verdict::WARN => {
            let score = score_value(graded);
            assert!((1.0..2.0).contains(&score), "{context}: {graded:?}");
        }
        Verdict::CRIT => assert!(score_value(graded) >= 2.0, "{context}: {graded:?}"),
        Verdict::NA | Verdict::UNKNOWN => {
            assert_eq!(graded.score, None, "{context}: {graded:?}");
        }
    }
}

const ALL: [Verdict; 5] = [
    Verdict::OK,
    Verdict::NA,
    Verdict::WARN,
    Verdict::CRIT,
    Verdict::UNKNOWN,
];

#[test]
fn unknown_reading_is_unknown_never_ok() {
    for rule in [RATIO, HIGHER, LOWER] {
        let (warn, crit) = design_lines();
        let definition = define(rule, SignalKind::Ratio, warn, crit);
        for n in [None, Some(556)] {
            for base in [
                pooled("0.013"),
                baseline(unread("offset 1d unread: timeout")),
            ] {
                let graded = grade(&definition, unread("source timed out"), n, &base);
                assert_eq!(
                    graded.verdict,
                    Verdict::UNKNOWN,
                    "{rule}, n {n:?}: {graded:?}"
                );
                assert_eq!(graded.score, None, "{rule}: {graded:?}");
                assert!(
                    graded.reason.contains("source timed out"),
                    "the reason is carried from the reading: {graded:?}"
                );
            }
        }
    }
}

#[test]
fn unknown_baseline_is_unknown() {
    let definition = ratio_definition();
    for value in ["0", "0.013", "0.029", "0.9"] {
        let base = baseline(unread("baseline predates series"));
        let graded = grade(&definition, ok(value), Some(556), &base);
        assert_eq!(graded.verdict, Verdict::UNKNOWN, "{value}: {graded:?}");
        assert_eq!(graded.score, None);
        assert!(
            graded.reason.contains("baseline predates series"),
            "the reason is carried from the baseline: {graded:?}"
        );
        assert_eq!(
            graded.baseline, base,
            "the graded result carries its baseline"
        );
    }
}

#[test]
fn partial_lower_bound_over_crit_is_crit() {
    let definition = ratio_definition();
    // The CRIT line over a baseline of 0.013 is the stricter of 4 x 0.013 = 0.052 and 0.05.
    for bound in ["0.052", "0.06", "0.5"] {
        let graded = grade(
            &definition,
            partial(bound, Bound::Lower),
            Some(556),
            &pooled("0.013"),
        );
        assert_eq!(
            graded.verdict,
            Verdict::CRIT,
            "lower bound {bound}: {graded:?}"
        );
        assert_agrees(&graded, bound);
        assert!(graded.reason.contains("partial"), "{graded:?}");
    }
    // The CRIT line declares no `min_events`, so a sample without a population still crosses it,
    // although the WARN line's `min_events` cannot be counted.
    for reading in [partial("0.06", Bound::Lower), ok("0.06")] {
        let graded = grade(&definition, reading, None, &pooled("0.013"));
        assert_eq!(graded.verdict, Verdict::CRIT, "no population: {graded:?}");
    }
    // A lower-is-worse rule escalates on an upper bound already under its CRIT line.
    let lower = definition_lower();
    let graded = grade(
        &lower,
        partial("20", Bound::Upper),
        Some(556),
        &pooled("100"),
    );
    assert_eq!(graded.verdict, Verdict::CRIT, "{graded:?}");
    assert_agrees(&graded, "lower_is_worse upper bound 20");
}

fn definition_lower() -> SignalDefinitionData {
    define(
        LOWER,
        SignalKind::Gauge,
        line(Some("0.5"), None, None),
        line(Some("0.25"), None, None),
    )
}

#[test]
fn partial_reading_never_clears() {
    let definition = ratio_definition();
    let cases = [
        // Under the baseline, between the lines, at the WARN line, just under the CRIT line.
        ("0.001", Bound::Lower),
        ("0.03", Bound::Lower),
        ("0.026", Bound::Lower),
        ("0.0519", Bound::Lower),
        // An upper bound says nothing about how high the value is.
        ("0.9", Bound::Upper),
        ("0.001", Bound::Upper),
    ];
    for (value, bound) in cases {
        let graded = grade(
            &definition,
            partial(value, bound),
            Some(556),
            &pooled("0.013"),
        );
        assert_eq!(
            graded.verdict,
            Verdict::UNKNOWN,
            "{value} {bound:?}: {graded:?}"
        );
        assert_eq!(graded.reason, REASON_PARTIAL, "{graded:?}");
        assert_eq!(graded.score, None);
    }
    // Lower-is-worse: a lower bound never escalates, an upper bound above the line never clears.
    let lower = definition_lower();
    for (value, bound) in [
        ("10", Bound::Lower),
        ("200", Bound::Upper),
        ("40", Bound::Upper),
    ] {
        let graded = grade(&lower, partial(value, bound), None, &pooled("100"));
        assert_eq!(
            graded.verdict,
            Verdict::UNKNOWN,
            "{value} {bound:?}: {graded:?}"
        );
        assert_eq!(graded.reason, REASON_PARTIAL);
    }
}

#[test]
fn below_min_n_is_na_without_score() {
    let definition = ratio_definition();
    for reading in [
        ok("0.9"),
        unread("source timed out"),
        partial("0.9", Bound::Lower),
    ] {
        let graded = grade(&definition, reading, Some(12), &pooled("0.013"));
        assert_eq!(graded.verdict, Verdict::NA, "{graded:?}");
        assert_eq!(graded.score, None);
        assert!(
            graded.reason.contains("12") && graded.reason.contains("20"),
            "the reason names n and min_n: {graded:?}"
        );
    }
    // N/A is decided only when the source answered a population.
    let graded = grade(
        &definition,
        unread("source timed out"),
        None,
        &pooled("0.013"),
    );
    assert_eq!(graded.verdict, Verdict::UNKNOWN, "{graded:?}");
    // At min_n the sample is graded.
    let graded = grade(&definition, ok("0.013"), Some(20), &pooled("0.013"));
    assert_eq!(graded.verdict, Verdict::OK, "{graded:?}");
}

#[test]
fn score_is_0_1_2_at_baseline_warn_crit() {
    // Multiplier lines, log-linear: 0.013 x 2 and 0.013 x 4.
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), None, None),
        line(Some("4"), None, None),
    );
    for (value, verdict, expected) in [
        ("0.013", Verdict::OK, "0"),
        ("0.026", Verdict::WARN, "1"),
        ("0.052", Verdict::CRIT, "2"),
    ] {
        let graded = grade(&definition, ok(value), Some(10_000), &pooled("0.013"));
        assert_eq!(graded.verdict, verdict, "{value}: {graded:?}");
        assert_eq!(score(&graded), Some(expected), "{value}: {graded:?}");
    }
    let better = grade(&definition, ok("0.0065"), Some(10_000), &pooled("0.013"));
    assert_eq!(better.verdict, Verdict::OK);
    assert!(
        score_value(&better) < 0.0,
        "better than the baseline is negative: {better:?}"
    );

    // Absolute lines, linear, against a fixed hard zero.
    let definition = define(
        HIGHER,
        SignalKind::CountOfCases,
        line(None, Some("10"), None),
        line(None, Some("20"), None),
    );
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: ok("0"),
    };
    for (value, verdict, expected) in [
        ("0", Verdict::OK, "0"),
        ("5", Verdict::OK, "0.5"),
        ("10", Verdict::WARN, "1"),
        ("15", Verdict::WARN, "1.5"),
        ("20", Verdict::CRIT, "2"),
    ] {
        let graded = grade(&definition, ok(value), None, &fixed);
        assert_eq!(graded.verdict, verdict, "{value}: {graded:?}");
        assert_eq!(score(&graded), Some(expected), "{value}: {graded:?}");
    }

    // Lower is worse: half and a quarter of the baseline.
    let definition = definition_lower();
    for (value, verdict, expected) in [
        ("100", Verdict::OK, "0"),
        ("50", Verdict::WARN, "1"),
        ("25", Verdict::CRIT, "2"),
    ] {
        let graded = grade(&definition, ok(value), None, &pooled("100"));
        assert_eq!(graded.verdict, verdict, "{value}: {graded:?}");
        assert_eq!(score(&graded), Some(expected), "{value}: {graded:?}");
    }
}

#[test]
fn the_reason_states_the_comparison() {
    let graded = grade(
        &ratio_definition(),
        ok("0.029"),
        Some(556),
        &pooled("0.013"),
    );
    assert_eq!(graded.verdict, Verdict::WARN, "{graded:?}");
    // The example evaluation of `website/docs/concepts/verdicts.md`, as the code writes it.
    assert_eq!(
        graded.reason, "0.029 >= 2 x 0.013 and >= 0.02, 16 events of 556 >= min_events 10",
        "design § 2.3 example: {graded:?}"
    );
    assert_eq!(score(&graded), Some("1.158"), "{graded:?}");
    // A lower-is-worse OK is above its WARN line, and says so.
    let graded = grade(&definition_lower(), ok("80"), None, &pooled("100"));
    assert_eq!(graded.verdict, Verdict::OK, "{graded:?}");
    assert_eq!(graded.reason, "above the WARN line: 80 > 0.5 x 100");
}

/// Design § 11 D23: an `x` part is decidable only over a baseline above 0. Over 0 or below, a
/// line that also declares `abs` is decided by `abs` alone; a line of `x` only is `UNKNOWN`.
#[test]
fn an_x_part_is_not_used_over_a_baseline_at_or_below_zero() {
    let both = define(
        HIGHER,
        SignalKind::Gauge,
        line(Some("2"), Some("10"), None),
        line(Some("4"), Some("20"), None),
    );
    for base in ["0", "-5"] {
        for (value, verdict) in [
            ("0", Verdict::OK),
            ("10", Verdict::WARN),
            ("20", Verdict::CRIT),
        ] {
            let graded = grade(&both, ok(value), None, &pooled(base));
            assert_eq!(graded.verdict, verdict, "{value} over {base}: {graded:?}");
            assert_agrees(&graded, value);
        }
    }
    let graded = grade(&both, ok("0"), None, &pooled("0"));
    assert_eq!(score(&graded), Some("0"), "0 at the baseline: {graded:?}");
    let x_only = define(
        HIGHER,
        SignalKind::Gauge,
        line(Some("2"), None, None),
        line(Some("4"), Some("20"), None),
    );
    for base in ["0", "-5"] {
        let graded = grade(&x_only, ok("0"), None, &pooled(base));
        assert_eq!(graded.verdict, Verdict::UNKNOWN, "{base}: {graded:?}");
        assert!(graded.reason.contains(base), "{graded:?}");
    }
}

/// For a drop, `min_events` counts the events at stake: the baseline's (design § 11 D22). A
/// count that drops from 100 to 0 crosses a line with `min_events: 10`; a drop from 3 to 0 does
/// not.
#[test]
fn a_drop_to_zero_crosses_when_the_baseline_had_enough_events() {
    let definition = define(
        LOWER,
        SignalKind::CountOfCases,
        line(Some("0.5"), None, Some(10)),
        line(Some("0.25"), None, Some(10)),
    );
    let graded = grade(&definition, ok("0"), None, &pooled("100"));
    assert_eq!(graded.verdict, Verdict::CRIT, "100 to 0: {graded:?}");
    assert_agrees(&graded, "100 to 0");
    assert!(graded.reason.contains("100 events"), "{graded:?}");
    let graded = grade(&definition, ok("0"), None, &pooled("3"));
    assert_eq!(graded.verdict, Verdict::OK, "3 to 0: {graded:?}");
    assert_agrees(&graded, "3 to 0");
    // The events at stake do not depend on the reading, so an upper bound of 0 escalates too.
    let graded = grade(
        &definition,
        partial("0", Bound::Upper),
        None,
        &pooled("100"),
    );
    assert_eq!(
        graded.verdict,
        Verdict::CRIT,
        "partial 100 to 0: {graded:?}"
    );
    let graded = grade(&definition, partial("0", Bound::Upper), None, &pooled("3"));
    assert_eq!(
        graded.verdict,
        Verdict::UNKNOWN,
        "partial 3 to 0: {graded:?}"
    );
}

/// A part of a line the grader cannot count (an event count beyond its exact range) does not
/// block a line that declares no `min_events`.
#[test]
fn an_uncountable_event_count_does_not_block_a_line_without_min_events() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), None, None),
        line(Some("4"), None, None),
    );
    let huge = "1000000000000000000000000000000";
    let graded = grade(&definition, ok(huge), Some(1_000_000_000), &pooled("0.013"));
    assert_eq!(graded.verdict, Verdict::CRIT, "{graded:?}");
    assert_agrees(&graded, huge);
}

#[test]
fn score_and_verdict_agree() {
    let mut definitions = vec![ratio_definition()];
    let lines = [
        (line(Some("2"), None, None), line(Some("4"), None, None)),
        (
            line(None, Some("0.02"), None),
            line(None, Some("0.05"), None),
        ),
        (
            line(Some("2"), Some("0.02"), Some(10)),
            line(Some("4"), Some("0.05"), Some(20)),
        ),
        (
            line(Some("1.5"), Some("0.03"), None),
            line(Some("1.2"), None, None),
        ),
        (
            line(Some("0.5"), None, None),
            line(Some("3"), None, Some(5)),
        ),
    ];
    for rule in [RATIO, HIGHER] {
        for (warn, crit) in &lines {
            definitions.push(define(rule, SignalKind::Ratio, warn.clone(), crit.clone()));
        }
    }
    let lower = [
        (
            line(Some("0.5"), None, None),
            line(Some("0.25"), None, None),
        ),
        (
            line(None, Some("0.01"), None),
            line(None, Some("0.005"), None),
        ),
        (
            line(Some("0.5"), Some("0.01"), Some(2)),
            line(Some("0.25"), Some("0.004"), None),
        ),
    ];
    for (warn, crit) in lower {
        definitions.push(define(LOWER, SignalKind::Gauge, warn, crit));
    }
    let values = [
        "0", "0.0001", "0.001", "0.004", "0.005", "0.0065", "0.01", "0.013", "0.0195", "0.02",
        "0.026", "0.029", "0.03", "0.039", "0.05", "0.052", "0.06", "0.1", "0.5", "1", "7",
    ];
    let bases = ["0", "0.013", "0.02", "0.5"];
    let mut graded_count = 0;
    let mut reached = Vec::new();
    for definition in &definitions {
        for base in bases {
            for value in values {
                for n in [None, Some(20), Some(556), Some(100_000)] {
                    let context = format!(
                        "{} warn {:?} crit {:?}, value {value}, baseline {base}, n {n:?}",
                        definition.evaluation.rule.0,
                        definition.evaluation.warn,
                        definition.evaluation.crit
                    );
                    let graded = grade(definition, ok(value), n, &pooled(base));
                    assert_agrees(&graded, &context);
                    reached.push(graded.verdict);
                    graded_count += 1;
                }
            }
        }
    }
    assert!(
        graded_count > 1000,
        "the sweep graded {graded_count} results"
    );
    for verdict in [Verdict::OK, Verdict::WARN, Verdict::CRIT] {
        assert!(reached.contains(&verdict), "the sweep reaches {verdict:?}");
    }
}

#[test]
fn worst_of_orders_crit_warn_unknown_ok() {
    let ranked = [
        (Verdict::CRIT, 3, 2),
        (Verdict::WARN, 2, 1),
        (Verdict::UNKNOWN, 1, 3),
        (Verdict::OK, 0, 0),
        (Verdict::NA, 0, 0),
    ];
    for (verdict, rank, exit) in ranked {
        assert_eq!(ladder::rank(verdict), rank, "{verdict:?}");
        assert_eq!(ladder::exit(verdict), exit, "{verdict:?}");
        assert_eq!(
            ladder::scored(verdict),
            !matches!(verdict, Verdict::NA | Verdict::UNKNOWN),
            "{verdict:?}"
        );
    }
    assert_eq!(
        ladder::worst([Verdict::OK, Verdict::UNKNOWN]),
        Some(Verdict::UNKNOWN)
    );
    assert_eq!(
        ladder::worst([Verdict::UNKNOWN, Verdict::WARN]),
        Some(Verdict::WARN)
    );
    assert_eq!(
        ladder::worst([Verdict::CRIT, Verdict::WARN]),
        Some(Verdict::CRIT)
    );
    assert_eq!(
        ladder::worst([Verdict::NA, Verdict::UNKNOWN]),
        Some(Verdict::UNKNOWN)
    );
    assert_eq!(ladder::worst(Vec::<Verdict>::new()), None);
    // The worst of a set does not depend on the order it was read in (L3).
    for a in ALL {
        for b in ALL {
            for c in ALL {
                let worst = ladder::worst([a, b, c]).expect("three verdicts");
                assert_eq!(ladder::worst([c, b, a]), Some(worst), "{a:?} {b:?} {c:?}");
                assert_eq!(ladder::worst([b, a, c]), Some(worst), "{a:?} {b:?} {c:?}");
                for verdict in [a, b, c] {
                    assert!(ladder::rank(worst) >= ladder::rank(verdict));
                }
            }
        }
    }
    assert_eq!(ladder::pass_exit([Verdict::OK, Verdict::UNKNOWN]), 3);
    assert_eq!(ladder::pass_exit([Verdict::UNKNOWN, Verdict::WARN]), 1);
    assert_eq!(
        ladder::pass_exit([Verdict::OK, Verdict::CRIT, Verdict::WARN]),
        2
    );
    assert_eq!(ladder::pass_exit([Verdict::OK, Verdict::OK]), 0);
}

#[test]
fn all_na_pass_exits_3() {
    assert_eq!(ladder::pass_exit([Verdict::NA]), 3);
    assert_eq!(
        ladder::pass_exit([Verdict::NA, Verdict::NA, Verdict::NA]),
        3
    );
    assert_eq!(
        ladder::pass_exit(Vec::<Verdict>::new()),
        3,
        "an empty pass graded nothing"
    );
    assert_eq!(ladder::pass_exit([Verdict::NA, Verdict::OK]), 0);
    assert_eq!(ladder::pass_exit([Verdict::NA, Verdict::WARN]), 1);
}

#[test]
fn unknown_rule_id_is_a_parse_error() {
    for id in [RATIO, HIGHER, LOWER] {
        let rule: RuleId = id.parse().expect("a known rule id parses");
        assert_eq!(rule.as_str(), id);
    }
    let error = "no_such_rule"
        .parse::<RuleId>()
        .expect_err("an unknown rule id is refused");
    assert!(error.to_string().contains("no_such_rule"), "{error}");
    let (warn, crit) = design_lines();
    let definition = define("no_such_rule", SignalKind::Ratio, warn, crit);
    let refused = grading::grade(
        &definition,
        &sample(ok("0.029"), Some(556)),
        &pooled("0.013"),
    );
    assert!(
        matches!(&refused, Err(Refusal::UnknownRule(unknown)) if unknown.id == "no_such_rule"),
        "{refused:?}"
    );
}

/// Every branch of the grader, recorded through `RecordEvaluation`: none trips
/// `thalamus.evaluation.RecordEvaluation/outcome/reason-missing` or
/// `thalamus.evaluation.RecordEvaluation/outcome/score-not-admitted`.
#[test]
fn every_graded_result_is_admitted_by_record_evaluation() {
    let readings = [
        ok("0"),
        ok("0.013"),
        ok("0.029"),
        ok("0.06"),
        partial("0.06", Bound::Lower),
        partial("0.02", Bound::Lower),
        partial("0.06", Bound::Upper),
        unread("source timed out"),
    ];
    let bases = [
        pooled("0.013"),
        pooled("0"),
        baseline(unread("baseline predates series")),
    ];
    let definitions = [
        ratio_definition(),
        define(
            HIGHER,
            SignalKind::Gauge,
            line(None, Some("0.02"), None),
            line(None, Some("0.05"), None),
        ),
        definition_lower(),
    ];
    let mut ports = Ports::new(MemoryPorts::default(), None);
    let mut seen = Vec::new();
    for definition in &definitions {
        for base in &bases {
            for reading in &readings {
                for n in [None, Some(5), Some(556)] {
                    let sample = sample(reading.clone(), n);
                    let graded = grading::grade(definition, &sample, base).expect("grades");
                    SignalSampleStorage::put(
                        &mut ports,
                        AnySignalSample::Recorded(SignalSample::new(sample.clone())).snapshot(),
                    );
                    let input = RecordEvaluation {
                        sample_id: sample.sample_id.clone(),
                        definition_rev: definition.rev,
                        verdict: graded.verdict,
                        prev: None,
                        score: graded.score.clone(),
                        baseline: Some(graded.baseline.clone()),
                        reason: Some(graded.reason.clone()),
                    };
                    let outcome = record_evaluation(&mut ports, input).expect("an obligation met");
                    assert!(
                        matches!(outcome, RecordEvaluationOutcome::Recorded { .. }),
                        "{graded:?} was refused: {outcome:?}"
                    );
                    assert!(!graded.reason.is_empty(), "{graded:?}");
                    seen.push(graded.verdict);
                }
            }
        }
    }
    for verdict in ALL {
        assert!(seen.contains(&verdict), "the cases reach {verdict:?}");
    }
}
