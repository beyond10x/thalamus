//! Adversary cases for `story:m1-verdict-grading`: the `min_events` boundary on a ratio the
//! adapters render, the reason's event count, a negative baseline, monotonicity, and the lines of
//! the grader no case of `tests/grading.rs` pins (the log-linear score, the stricter part of a
//! line, `worst([OK, N/A])`, a count exactly at `min_events`).

use std::collections::BTreeMap;

use thalamus_core::grading::{self, Graded};
use thalamus_core::ladder;
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId as ModelRuleId, SignalDefinitionData, SignalId,
    SignalKind, Source, Threshold, Unit,
};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
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
            min_n: 20,
            since: "first_sample".into(),
        }),
        evaluation: EvaluationSpec {
            rule: ModelRuleId(rule.into()),
            baselines: vec![BaselineKind::Offset1d, BaselineKind::Offset7d],
            warn: Some(warn),
            crit: Some(crit),
            min_trend_window: None,
        },
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

fn sample(value: &str, n: Option<i64>) -> SignalSampleData {
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
        reading: Reading::Ok(CompleteReading {
            value: Decimal(value.into()),
        }),
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

fn pooled(value: &str) -> Baseline {
    Baseline {
        kind: BaselineLabel("pooled".into()),
        offsets: vec!["1d".into(), "7d".into()],
        reading: Reading::Ok(CompleteReading {
            value: Decimal(value.into()),
        }),
    }
}

fn grade(definition: &SignalDefinitionData, value: &str, n: Option<i64>, base: &str) -> Graded {
    grading::grade(definition, &sample(value, n), &pooled(base)).expect("the definition grades")
}

fn score_of(graded: &Graded) -> f64 {
    graded
        .score
        .as_ref()
        .unwrap_or_else(|| panic!("{graded:?} carries no score"))
        .0
        .parse()
        .expect("a score is a decimal")
}

fn severity(verdict: Verdict) -> u8 {
    match verdict {
        Verdict::OK => 0,
        Verdict::WARN => 1,
        Verdict::CRIT => 2,
        Verdict::NA | Verdict::UNKNOWN => panic!("a complete reading with n is graded"),
    }
}

/// A ratio of exactly 10 failures in 30 requests, as the adapters render it, crosses a line
/// declaring `min_events: 10`: "the event count (value times `n`) is at least `min_events`".
/// The SQL, Loki and git adapters render a ratio at 18 fractional digits, rounded half away from
/// zero (`sql.rs` `quotient`, `loki.rs` `over`, `git.rs` `quotient`); the Prometheus adapter
/// renders an `f64` (`prometheus.rs` `render`). Both put `value x n` just under 10.
#[test]
fn adv_exact_event_count_at_min_events_crosses_the_line() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), Some("0.02"), Some(10)),
        line(Some("4"), Some("0.5"), None),
    );
    for rendered in ["0.333333333333333333", "0.3333333333333333"] {
        let graded = grade(&definition, rendered, Some(30), "0.1");
        assert_eq!(
            graded.verdict,
            Verdict::WARN,
            "10 failures of 30 (value {rendered}) meet min_events 10: {graded:?}"
        );
    }
}

/// Design § 2.3: the reason states the comparison, `0.029 >= 2 x 0.013 and >= 0.02, 15 failures
/// of 556`. With 15 failures of 556 rendered at 18 digits (`0.02697841726618705`), the count the
/// reason states is a whole number of events.
#[test]
fn adv_reason_states_a_whole_event_count() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), Some("0.02"), Some(10)),
        line(Some("4"), Some("0.05"), None),
    );
    let graded = grade(&definition, "0.02697841726618705", Some(556), "0.013");
    assert_eq!(graded.verdict, Verdict::WARN, "{graded:?}");
    assert!(
        graded.reason.contains(" 15 events of 556")
            || graded.reason.contains(" 15 failures of 556"),
        "the reason counts 15 events of 556: {:?}",
        graded.reason
    );
}

/// `higher_is_worse` over a negative baseline: a value better than the baseline (lower) is not
/// out of bounds. With `x` lines over a baseline of -10 the CRIT line sits at -40, under the
/// baseline, so -11 reads as crossed.
///
/// Expectation changed by coordinator decision 3 (design § 11 D23): an `x` part is decidable
/// only over a baseline above 0, so x-only lines over -10 grade `UNKNOWN`, never `OK` and never
/// `CRIT`, with a reason naming the baseline.
#[test]
fn adv_value_better_than_a_negative_baseline_is_not_out_of_bounds() {
    let definition = define(
        HIGHER,
        SignalKind::Gauge,
        line(Some("2"), None, None),
        line(Some("4"), None, None),
    );
    let graded = grade(&definition, "-11", None, "-10");
    assert_eq!(
        graded.verdict,
        Verdict::UNKNOWN,
        "x-only lines over a baseline of -10 cannot be decided (D23): {graded:?}"
    );
    assert_eq!(graded.score, None, "{graded:?}");
    assert!(graded.reason.contains("-10"), "{graded:?}");
}

/// A count exactly at `min_events` crosses the line; one under it does not. Kills the mutant
/// `events < min_events` -> `events <= min_events` in `threshold.rs` `short`.
#[test]
fn adv_count_exactly_at_min_events_crosses() {
    let definition = define(
        HIGHER,
        SignalKind::CountOfCases,
        line(None, Some("5"), Some(10)),
        line(None, Some("100"), None),
    );
    let graded = grade(&definition, "10", None, "0");
    assert_eq!(graded.verdict, Verdict::WARN, "{graded:?}");
    let graded = grade(&definition, "9", None, "0");
    assert_eq!(graded.verdict, Verdict::OK, "{graded:?}");
}

/// `worst([OK, N/A])` is `N/A` in either order. Kills the mutant that breaks the tie toward `OK`.
#[test]
fn adv_worst_of_ok_and_na_is_na() {
    assert_eq!(ladder::worst([Verdict::OK, Verdict::NA]), Some(Verdict::NA));
    assert_eq!(ladder::worst([Verdict::NA, Verdict::OK]), Some(Verdict::NA));
    assert_eq!(
        ladder::worst([Verdict::OK, Verdict::NA, Verdict::OK]),
        Some(Verdict::NA)
    );
    assert_eq!(ladder::pass_exit([Verdict::NA, Verdict::UNKNOWN]), 3);
}

/// Design § 3.3: log-linear between the marks of a multiplier line. Over 0.013 with `x: 2` and
/// `x: 4`, 0.0195 is log2(1.5) = 0.585 and 0.039 is 1 + log2(1.5) = 1.585. Kills the mutant that
/// interpolates linearly (0.5 and 1.5).
#[test]
fn adv_score_is_log_linear_between_multiplier_marks() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), None, None),
        line(Some("4"), None, None),
    );
    for (value, verdict, expected) in [
        ("0.0195", Verdict::OK, "0.585"),
        ("0.039", Verdict::WARN, "1.585"),
    ] {
        let graded = grade(&definition, value, Some(10_000), "0.013");
        assert_eq!(graded.verdict, verdict, "{value}: {graded:?}");
        assert_eq!(
            graded.score.as_ref().map(|score| score.0.as_str()),
            Some(expected),
            "{value}: {graded:?}"
        );
    }
}

/// The WARN line of the design's lines over 0.013 is the stricter part, 2 x 0.013 = 0.026, not
/// `abs: 0.02`; 0.022 scores log2(0.022 / 0.013) = 0.759. Kills the mutant that takes the laxer
/// part (its score clamps to 0.999 and the band check alone stays green).
#[test]
fn adv_score_follows_the_stricter_part_of_a_line() {
    let definition = define(
        RATIO,
        SignalKind::Ratio,
        line(Some("2"), Some("0.02"), None),
        line(Some("4"), Some("0.05"), None),
    );
    let graded = grade(&definition, "0.022", Some(10_000), "0.013");
    assert_eq!(graded.verdict, Verdict::OK, "{graded:?}");
    assert_eq!(
        graded.score.as_ref().map(|score| score.0.as_str()),
        Some("0.759"),
        "{graded:?}"
    );
}

/// Under `higher_is_worse` (and the ratio rule) a larger value never grades better, in verdict
/// or score; under `lower_is_worse` the mirror holds. Non-negative baselines and values.
#[test]
fn adv_grading_is_monotone_in_the_value() {
    let higher_lines = [
        (line(Some("2"), None, None), line(Some("4"), None, None)),
        (
            line(Some("2"), Some("0.02"), Some(10)),
            line(Some("4"), Some("0.05"), Some(20)),
        ),
        (
            line(None, Some("0.02"), None),
            line(None, Some("0.05"), None),
        ),
        (
            line(Some("1.5"), Some("3"), Some(2)),
            line(Some("3"), None, None),
        ),
        (line(None, Some("10"), None), line(Some("4"), None, Some(5))),
    ];
    let lower_lines = [
        (
            line(Some("0.5"), None, None),
            line(Some("0.25"), None, None),
        ),
        (
            line(Some("0.5"), Some("40"), Some(10)),
            line(Some("0.25"), Some("10"), Some(10)),
        ),
        (line(None, Some("50"), None), line(None, Some("20"), None)),
    ];
    let values = [
        "0", "0.001", "0.005", "0.01", "0.013", "0.019", "0.02", "0.022", "0.026", "0.03", "0.04",
        "0.05", "0.052", "0.1", "0.5", "1", "2", "3", "5", "9", "10", "20", "25", "40", "49", "50",
        "60", "99", "100", "150", "400", "1000",
    ];
    let bases = ["0", "0.013", "0.5", "3", "100"];
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
    let mut compared = 0;
    let mut violations = Vec::new();
    for (lower_is_worse, definition) in &definitions {
        for base in bases {
            // Coordinator decision 3 (design § 11 D23): over a baseline at or below 0 a line
            // declaring only `x` cannot be decided, so every value grades `UNKNOWN` with no score
            // and a reason naming the baseline; there is no order to compare.
            let x_only = |threshold: &Option<thalamus_model::definition::Threshold>| {
                threshold
                    .as_ref()
                    .is_some_and(|line| line.x.is_some() && line.abs.is_none())
            };
            let evaluation = &definition.evaluation;
            if base.parse::<f64>().expect("a baseline") <= 0.0
                && (x_only(&evaluation.warn) || x_only(&evaluation.crit))
            {
                for value in values {
                    let graded = grade(definition, value, Some(1_000), base);
                    assert_eq!(graded.verdict, Verdict::UNKNOWN, "D23 {base}: {graded:?}");
                    assert_eq!(graded.score, None, "D23 {base}: {graded:?}");
                    assert!(graded.reason.contains(base), "D23 {base}: {graded:?}");
                }
                continue;
            }
            let mut previous: Option<(String, u8, f64)> = None;
            let ordered: Vec<&str> = if *lower_is_worse {
                values.iter().rev().copied().collect()
            } else {
                values.to_vec()
            };
            // `ordered` runs from better to worse.
            for value in ordered {
                let graded = grade(definition, value, Some(1_000), base);
                let (rank, score) = (severity(graded.verdict), score_of(&graded));
                if let Some((before, previous_rank, previous_score)) = &previous {
                    let context = format!(
                        "{} warn {:?} crit {:?}, baseline {base}: {before} then {value}",
                        definition.evaluation.rule.0,
                        definition.evaluation.warn,
                        definition.evaluation.crit
                    );
                    if rank < *previous_rank {
                        violations.push(format!("verdict improved: {context}: {graded:?}"));
                    }
                    if score < *previous_score {
                        violations.push(format!(
                            "score improved ({previous_score} then {score}): {context}"
                        ));
                    }
                    compared += 1;
                }
                previous = Some((value.to_owned(), rank, score));
            }
        }
    }
    assert!(compared > 500, "the sweep compared {compared} pairs");
    assert!(
        violations.is_empty(),
        "{} of {compared} pairs grade a worse value better:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

/// Design § 11 D22's own drop: a count falling from 100 to 0 is at least as bad as one falling to
/// one. Under `x: 0.5` / `x: 0.25` the score is log-linear past the CRIT line while the value is
/// positive and falls back to linear at 0, so 1 scores 6.644 and 0 scores 3.
#[test]
fn adv_drop_to_zero_scores_no_better_than_a_drop_to_one() {
    let definition = define(
        LOWER,
        SignalKind::CountOfCases,
        line(Some("0.5"), None, None),
        line(Some("0.25"), None, None),
    );
    let to_one = grade(&definition, "1", None, "100");
    let to_zero = grade(&definition, "0", None, "100");
    assert_eq!(to_one.verdict, Verdict::CRIT, "{to_one:?}");
    assert_eq!(to_zero.verdict, Verdict::CRIT, "{to_zero:?}");
    assert!(
        score_of(&to_zero) >= score_of(&to_one),
        "a drop to 0 scores {:?}, a drop to 1 scores {:?}",
        to_zero.score,
        to_one.score
    );
}
