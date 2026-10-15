//! Adversary pass 2 for `story:m1-verdict-grading`: the rounded event count (design § 11 D22) at
//! exactly half an event and below zero, the score at the CRIT line for both directions, lines
//! with both `abs` and `x` over a baseline at or below 0 (design § 11 D23), monotonicity within
//! one such baseline, and a CRIT line `abs` decides while the WARN line is `x` only.

use std::collections::BTreeMap;

use thalamus_core::grading::{self, Graded};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId as ModelRuleId, SignalDefinitionData, SignalId,
    SignalKind, Source, Threshold, Unit,
};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, Denominator, PartialReading, Provenance, QueryDigest, Reading, RunId,
    SampleId, SeriesKey, SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

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
            min_n: 1,
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

fn sample(reading: Reading, n: Option<i64>) -> SignalSampleData {
    SignalSampleData {
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

fn pooled(value: &str) -> Baseline {
    Baseline {
        kind: BaselineLabel("pooled".into()),
        offsets: vec!["1d".into(), "7d".into()],
        reading: ok(value),
    }
}

fn grade_reading(
    definition: &SignalDefinitionData,
    reading: Reading,
    n: Option<i64>,
    base: &str,
) -> Graded {
    grading::grade(definition, &sample(reading, n), &pooled(base)).expect("the definition grades")
}

fn grade(definition: &SignalDefinitionData, value: &str, n: Option<i64>, base: &str) -> Graded {
    grade_reading(definition, ok(value), n, base)
}

fn score(graded: &Graded) -> Option<&str> {
    graded.score.as_ref().map(|score| score.0.as_str())
}

fn score_of(graded: &Graded) -> f64 {
    score(graded)
        .unwrap_or_else(|| panic!("{graded:?} carries no score"))
        .parse()
        .expect("a score is a decimal")
}

/// Design § 11 D23 says the `x` part is not used over a baseline of 0 or below, and that a line
/// declaring `abs` is then decided by `abs`. Code comment `threshold.rs` `try_decide`: "a crossed
/// CRIT line decides, whatever the WARN line could not count". § 3.1 rule 4: a lower bound
/// already over the CRIT line is `CRIT`. Here the CRIT line declares `abs: 20` and the value is 25
/// over a baseline of 0: the CRIT line is decidable and crossed, so the verdict is `CRIT` whatever
/// the `x`-only WARN line could have said. The grader answers `UNKNOWN` (exit 3, rank under
/// `WARN`) because it refuses the WARN line before it reads the CRIT line.
#[test]
fn adv_p2_a_crossed_abs_crit_line_decides_when_the_warn_line_is_x_only() {
    let count = define(
        HIGHER,
        SignalKind::Gauge,
        line(Some("2"), None, None),
        line(Some("4"), Some("20"), None),
    );
    let ratio = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), None, Some(10)),
        line(Some("4"), Some("0.05"), None),
    );
    let partial = Reading::Partial(PartialReading {
        value: Decimal("25".into()),
        bound: Bound::Lower,
    });
    let cases = [
        ("count 25 over 0", grade(&count, "25", None, "0")),
        ("count 25 over -5", grade(&count, "25", None, "-5")),
        (
            "partial lower bound 25 over 0",
            grade_reading(&count, partial, None, "0"),
        ),
        (
            "ratio 0.5 of 556 over 0",
            grade(&ratio, "0.5", Some(556), "0"),
        ),
    ];
    let wrong: Vec<String> = cases
        .iter()
        .filter(|(_, graded)| graded.verdict != Verdict::CRIT)
        .map(|(case, graded)| format!("{case}: {graded:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "a crossed, decidable CRIT line is CRIT:\n{}",
        wrong.join("\n")
    );
}

/// Design § 11 D22: a ratio's event count is the number times `n` rounded to the nearest whole
/// event, half away from zero. 0.5 of 19 is 9.5 events, which rounds to 10 and crosses
/// `min_events: 10`; 0.45 of 21 is 9.45, which rounds to 9 and does not.
#[test]
fn adv_p2_half_an_event_rounds_up() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), Some("0.02"), Some(10)),
        line(Some("8"), Some("0.9"), None),
    );
    let graded = grade(&definition, "0.5", Some(19), "0.1");
    assert_eq!(
        graded.verdict,
        Verdict::WARN,
        "9.5 events round to 10: {graded:?}"
    );
    assert!(
        graded.reason.contains(", 10 events of 19"),
        "the reason states the rounded count: {graded:?}"
    );
    let graded = grade(&definition, "0.45", Some(21), "0.1");
    assert_eq!(
        graded.verdict,
        Verdict::OK,
        "9.45 events round to 9: {graded:?}"
    );
    assert!(
        graded.reason.contains(", 9 events of 21"),
        "the reason states the rounded count: {graded:?}"
    );
}

/// Half away from zero holds below zero too: -0.5 of 19 is -9.5 events and rounds to -10, not -9
/// and not -8. The WARN line is `abs: -1` over a baseline of 0, so the value reaches it and only
/// the count is short; the reason states the count.
#[test]
fn adv_p2_half_an_event_rounds_away_from_zero_below_zero() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(None, Some("-1"), Some(10)),
        line(None, Some("5"), None),
    );
    let graded = grade(&definition, "-0.5", Some(19), "0");
    assert_eq!(graded.verdict, Verdict::OK, "{graded:?}");
    assert!(
        graded.reason.contains(", -10 events of 19"),
        "-9.5 events round to -10: {graded:?}"
    );
}

/// Design § 3.3 and score.md: only the stretch past the CRIT line of a rule where lower is worse
/// turns linear. Under `higher_is_worse` and the ratio rule it stays log-linear: over 0.013 with
/// `x: 2` / `x: 4`, 0.104 is 8 times the baseline, 1 + log2(0.104 / 0.026) = 3 (linear would be 4).
#[test]
fn adv_p2_higher_is_worse_stays_log_linear_past_the_crit_line() {
    for rule in [RATIO, HIGHER] {
        let definition = define(
            rule,
            SignalKind::Ratio,
            line(Some("2"), None, None),
            line(Some("4"), None, None),
        );
        let graded = grade(&definition, "0.104", Some(10_000), "0.013");
        assert_eq!(graded.verdict, Verdict::CRIT, "{rule}: {graded:?}");
        assert_eq!(score(&graded), Some("3"), "{rule}: {graded:?}");
    }
}

/// Design § 11 D22: under `lower_is_worse` the score stays log-linear between the marks and turns
/// linear past the CRIT line (from 1 at the WARN line through 2 at the CRIT line). Over 100 with
/// `x: 0.5` / `x: 0.25`: 100 / sqrt 2 scores 0.5, 50 / sqrt 2 scores 1.5, 1 scores 2.96, 0 scores 3,
/// and the score is continuous across the CRIT line.
#[test]
fn adv_p2_lower_is_worse_score_between_and_past_its_marks() {
    let definition = define(
        LOWER,
        SignalKind::Gauge,
        line(Some("0.5"), None, None),
        line(Some("0.25"), None, None),
    );
    for (value, verdict, expected) in [
        ("70.710678118654752", Verdict::OK, "0.5"),
        ("35.355339059327376", Verdict::WARN, "1.5"),
        ("25", Verdict::CRIT, "2"),
        ("1", Verdict::CRIT, "2.96"),
        ("0", Verdict::CRIT, "3"),
        ("-25", Verdict::CRIT, "4"),
    ] {
        let graded = grade(&definition, value, None, "100");
        assert_eq!(graded.verdict, verdict, "{value}: {graded:?}");
        assert_eq!(score(&graded), Some(expected), "{value}: {graded:?}");
    }
    let above = score_of(&grade(&definition, "25.01", None, "100"));
    let below = score_of(&grade(&definition, "24.99", None, "100"));
    assert!(
        (below - above).abs() <= 0.002,
        "continuous at the CRIT line: 25.01 scores {above}, 24.99 scores {below}"
    );
    let higher = define(
        HIGHER,
        SignalKind::Gauge,
        line(Some("2"), None, None),
        line(Some("4"), None, None),
    );
    let under = score_of(&grade(&higher, "0.05199", None, "0.013"));
    let over = score_of(&grade(&higher, "0.05201", None, "0.013"));
    assert!(
        (over - under).abs() <= 0.002,
        "continuous at the CRIT line: 0.05199 scores {under}, 0.05201 scores {over}"
    );
}

/// Design § 11 D23 over a baseline of 0 or below: a line declaring `x` and `abs` is decided by
/// `abs` and `min_events` alone, so its reason does not cite an `x` part it did not use.
#[test]
fn adv_p2_x_and_abs_lines_over_a_zero_baseline_are_decided_by_abs() {
    let ratio = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), Some("0.02"), Some(10)),
        line(Some("4"), Some("0.05"), None),
    );
    for base in ["0", "-0.013"] {
        // 0.03 of 556 is 16.68 events, 17 rounded.
        let graded = grade(&ratio, "0.03", Some(556), base);
        assert_eq!(graded.verdict, Verdict::WARN, "{base}: {graded:?}");
        assert_eq!(
            graded.reason, "0.03 >= 0.02, 17 events of 556 >= min_events 10",
            "{base}"
        );
        // 0.03 of 300 is 9 events: past the WARN line's abs, short of its min_events.
        let graded = grade(&ratio, "0.03", Some(300), base);
        assert_eq!(graded.verdict, Verdict::OK, "{base}: {graded:?}");
        assert_eq!(score(&graded), Some("0.999"), "{base}: {graded:?}");
        assert!(
            graded
                .reason
                .starts_with("too few events for the WARN line"),
            "{base}: {graded:?}"
        );
        let graded = grade(&ratio, "0.06", Some(100), base);
        assert_eq!(graded.verdict, Verdict::CRIT, "{base}: {graded:?}");
        assert_eq!(graded.reason, "0.06 >= 0.05, 6 events of 100", "{base}");
        let graded = grade(&ratio, "0.01", Some(556), base);
        assert_eq!(graded.verdict, Verdict::OK, "{base}: {graded:?}");
        assert_eq!(graded.reason, "under the WARN line: 0.01 < 0.02", "{base}");
    }
    // Lower is worse over 0: abs 5 and abs 2 decide; the x parts (0 x 0.5, 0 x 0.25) do not.
    let lower = define(
        LOWER,
        SignalKind::Gauge,
        line(Some("0.5"), Some("5"), None),
        line(Some("0.25"), Some("2"), None),
    );
    for base in ["0", "-4"] {
        for (value, verdict) in [
            ("6", Verdict::OK),
            ("3", Verdict::WARN),
            ("1", Verdict::CRIT),
        ] {
            let graded = grade(&lower, value, None, base);
            assert_eq!(graded.verdict, verdict, "{value} over {base}: {graded:?}");
            assert!(
                !graded.reason.contains(" x "),
                "the reason cites no x part: {graded:?}"
            );
        }
    }
}

fn severity(verdict: Verdict, context: &str) -> u8 {
    match verdict {
        Verdict::OK => 0,
        Verdict::WARN => 1,
        Verdict::CRIT => 2,
        Verdict::NA | Verdict::UNKNOWN => panic!("{context}: graded {verdict:?}"),
    }
}

/// Within one baseline at or near the D23 boundary (negative, 0, just above 0), a worse value never
/// grades better, in verdict or score. Values run through negatives. x-only line sets over a
/// baseline at or below 0 grade `UNKNOWN` throughout and are not ordered.
#[test]
fn adv_p2_grading_is_monotone_within_a_baseline_around_zero() {
    let higher_lines = [
        (
            line(Some("2"), Some("0.02"), Some(10)),
            line(Some("4"), Some("0.05"), None),
        ),
        (
            line(Some("2"), Some("0.02"), Some(10)),
            line(Some("4"), Some("0.05"), Some(20)),
        ),
        (
            line(None, Some("10"), Some(5)),
            line(Some("4"), Some("20"), None),
        ),
        (
            line(Some("1.5"), Some("-1"), None),
            line(Some("3"), Some("5"), None),
        ),
        (line(Some("2"), None, None), line(Some("4"), None, None)),
    ];
    let lower_lines = [
        (
            line(Some("0.5"), Some("40"), Some(10)),
            line(Some("0.25"), Some("10"), Some(10)),
        ),
        (
            line(Some("0.5"), Some("5"), None),
            line(Some("0.25"), Some("2"), None),
        ),
        (line(None, Some("50"), None), line(None, Some("-20"), None)),
        (
            line(Some("0.5"), None, None),
            line(Some("0.25"), None, None),
        ),
    ];
    let values = [
        "-1000", "-50", "-20", "-10", "-1", "-0.05", "-0.001", "0", "0.001", "0.01", "0.013",
        "0.02", "0.026", "0.03", "0.05", "0.052", "0.1", "0.5", "1", "2", "3", "5", "9", "10",
        "20", "25", "40", "50", "100", "1000",
    ];
    let bases = [
        "-100",
        "-5",
        "-0.013",
        "0",
        "0.000000000000000001",
        "0.013",
        "3",
    ];
    let mut definitions = Vec::new();
    for rule in [RATIO, HIGHER] {
        for (warn, crit) in &higher_lines {
            definitions.push((
                false,
                define(rule, SignalKind::Ratio, warn.clone(), crit.clone()),
            ));
        }
    }
    for (warn, crit) in &lower_lines {
        definitions.push((
            true,
            define(LOWER, SignalKind::Gauge, warn.clone(), crit.clone()),
        ));
    }
    let x_only = |threshold: &Option<Threshold>| {
        threshold
            .as_ref()
            .is_some_and(|line| line.x.is_some() && line.abs.is_none())
    };
    let mut compared = 0;
    let mut violations = Vec::new();
    for (lower_is_worse, definition) in &definitions {
        let evaluation = &definition.evaluation;
        for base in bases {
            for n in [Some(300), Some(1_000)] {
                let undecidable = base.parse::<f64>().expect("a baseline") <= 0.0
                    && (x_only(&evaluation.warn) || x_only(&evaluation.crit));
                let ordered: Vec<&str> = if *lower_is_worse {
                    values.iter().rev().copied().collect()
                } else {
                    values.to_vec()
                };
                let mut previous: Option<(&str, u8, f64)> = None;
                for value in ordered {
                    let graded = grade(definition, value, n, base);
                    let context = format!(
                        "{} warn {:?} crit {:?}, baseline {base}, n {n:?}, value {value}",
                        evaluation.rule.0, evaluation.warn, evaluation.crit
                    );
                    if undecidable {
                        assert_eq!(graded.verdict, Verdict::UNKNOWN, "{context}: {graded:?}");
                        continue;
                    }
                    let (rank, score) = (severity(graded.verdict, &context), score_of(&graded));
                    if let Some((before, previous_rank, previous_score)) = previous {
                        if rank < previous_rank {
                            violations.push(format!("verdict improved after {before}: {context}"));
                        }
                        if score < previous_score {
                            violations.push(format!(
                                "score improved ({previous_score} then {score}) after {before}: {context}"
                            ));
                        }
                        compared += 1;
                    }
                    previous = Some((value, rank, score));
                }
            }
        }
    }
    assert!(compared > 1_000, "the sweep compared {compared} pairs");
    assert!(
        violations.is_empty(),
        "{} of {compared} pairs grade a worse value better:\n{}",
        violations.len(),
        violations.join("\n")
    );
}
