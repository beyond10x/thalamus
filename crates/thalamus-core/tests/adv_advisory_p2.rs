//! Adversary pass 2 on `story:m3-advisory-gate-mapping` (wave 20261015g): the rule
//! `fixed_warn_only` after correction 1, driven against what the story's revision 6, the rule's
//! module documentation and the site's verdicts page say.
//!
//! - Story revision 6: "The verdict and the score are decided exactly on normalised decimals at any
//!   magnitude". A baseline the loader admits, however small, and a count, however large, still
//!   compare: a count above the baseline is `WARN`.
//! - `rules/warn_only.rs` and `website/docs/concepts/verdicts.md`: the `OK` score is
//!   `(count - baseline) / (floor(baseline) + 1 - baseline)`, "rounded only at the end, to three
//!   decimal places", half away from zero, never `-0`. No case of the unit's suite has a score
//!   with a fractional digit, so these pin the rendering and the step below a negative baseline.
//! - The wire grammar of a decimal: exponent forms, a `+` sign, a bare `.5` or `4.` are no
//!   decimal, so they grade `UNKNOWN`, never `OK` or `WARN`.

use std::collections::BTreeMap;

use thalamus_core::grading::{self, Graded};
use thalamus_core::registry::{Definition, Registry};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Unit,
};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

const SIGNAL: &str = "gate.ci.advisory_findings";
const REGISTRY: &str = include_str!("fixtures/instance/dimensions.yaml");
/// 10^-38: 38 fractional digits, the last one 1. The loader's `is_decimal` admits it.
const TINY: &str = "0.00000000000000000000000000000000000001";
/// 10^39 findings: 40 digits.
const HUGE: &str = "1000000000000000000000000000000000000000";

fn definition(baseline: &str) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId(SIGNAL.into()),
        title: "Findings of the advisory lint gate".into(),
        family: Family("gate".into()),
        kind: SignalKind::CountOfCases,
        unit: Some(Unit("{finding}".into())),
        dimensions: Some(Vec::new()),
        source: thalamus_model::definition::Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("ci-results".into()),
        },
        extractor: ExtractorId("sql.value".into()),
        query: BTreeMap::new(),
        window: Duration("PT1H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Store,
        denominator: Some(DenominatorSpec {
            unit: Unit("{run}".into()),
            min_n: 0,
            since: "2026-01-01T00:00:00Z".into(),
        }),
        evaluation: EvaluationSpec {
            rule: RuleId("fixed_warn_only".into()),
            baselines: vec![BaselineKind::Fixed],
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: Some(Decimal(baseline.into())),
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:gate".into())),
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

fn sample(reading: Reading) -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_1".into()),
        signal: SignalId(SIGNAL.into()),
        series: SeriesKey(SIGNAL.into()),
        dims: BTreeMap::new(),
        ts: Timestamp("2026-10-14T22:00:00Z".into()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T21:00:00Z".into()),
            end: Timestamp("2026-10-14T22:00:00Z".into()),
        },
        reading,
        unit: Unit("{finding}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("ci-results".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

fn count(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

/// `value` graded against the `fixed` baseline `baseline`, the definition declaring the same
/// number (as `baseline::fold` hands it over verbatim).
fn grade(baseline: &str, value: &str) -> Graded {
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: count(baseline),
    };
    grading::grade(&definition(baseline), &sample(count(value)), &fixed)
        .expect("a fixed_warn_only definition grades")
}

fn verdict_and_score(graded: &Graded) -> (Verdict, Option<&str>) {
    (
        graded.verdict,
        graded.score.as_ref().map(|score| score.0.as_str()),
    )
}

/// An advisory definition file with the fixed baseline `baseline`, as an operator writes it.
fn definition_file(baseline: &str) -> String {
    format!(
        "id: {SIGNAL}\n\
         title: Findings of the advisory lint gate\n\
         family: gate\n\
         kind: count_of_cases\n\
         unit: \"{{finding}}\"\n\
         dimensions: []\n\
         source: {{ adapter: sql, connection: ci-results }}\n\
         extractor: sql.value\n\
         query: {{ value: select findings from gate_runs }}\n\
         window: PT1H\n\
         cadence: PT1H\n\
         history: store\n\
         denominator: {{ unit: \"{{run}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}\n\
         evaluation:\n\
         \x20 rule: fixed_warn_only\n\
         \x20 baselines: [fixed]\n\
         \x20 fixed: \"{baseline}\"\n\
         owner: team:gate\n\
         links: {{ components: [], issues: [] }}\n\
         lifecycle: active\n\
         rev: 1\n\
         revised_at: 2026-10-02T00:00:00Z\n\
         revised_by: operator\n"
    )
}

/// Story revision 6 ("at any magnitude"): the loader admits the baseline 10^-38, so a count of 2
/// findings against it is above it and `WARN`, scored 1. Aligning 2 to 38 fractional digits
/// overflows `i128`, and the rule answers `UNKNOWN` for a comparison a string of digits decides.
#[test]
fn adv_p2_count_above_a_tiny_baseline_the_loader_admits_is_warn() {
    let registry = Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry");
    let text = definition_file(TINY);
    Definition::parse("definitions/case.yaml", &text, &registry)
        .unwrap_or_else(|errors| panic!("the loader refuses {TINY}: {errors:?}"));

    let graded = grade(TINY, "2");
    assert_eq!(
        verdict_and_score(&graded),
        (Verdict::WARN, Some("1")),
        "count 2 against fixed baseline {TINY}: {graded:?}"
    );
}

/// Story revision 6 ("at any magnitude") and the acceptance (`advisory_never_grades_crit`: a count
/// far above the baseline is `WARN`): 10^39 findings against the baseline 3 is above it.
#[test]
fn adv_p2_count_past_i128_above_the_baseline_is_warn() {
    let graded = grade("3", HUGE);
    assert_eq!(
        verdict_and_score(&graded),
        (Verdict::WARN, Some("1")),
        "count {HUGE} against fixed baseline 3: {graded:?}"
    );
}

/// The `OK` score's fractional digits: three at most, half away from zero, trailing zeros dropped,
/// a carry into the whole part, and a score that rounds to nothing is `0`, never `-0`.
#[test]
fn adv_p2_fractional_ok_scores_round_half_away_to_three_digits() {
    for (baseline, value, expected) in [
        // step 0.75: -0.25 / 0.75 = -0.3333…
        ("2.25", "2", "-0.333"),
        // -0.5 / 0.75 = -0.6666…
        ("2.25", "1.75", "-0.667"),
        // step 0.5: -0.2 / 0.5 = -0.4, written without trailing zeros
        ("2.5", "2.3", "-0.4"),
        // -0.00025 / 0.5 = -0.0005 exactly: half, away from zero
        ("2.5", "2.49975", "-0.001"),
        // -0.000245 / 0.5 = -0.00049: below half, so 0, and never -0
        ("2.5", "2.499755", "0"),
        // -0.49975 / 0.5 = -0.9995: half, carried into the whole part
        ("2.5", "2.00025", "-1"),
        // -1.24975 / 0.5 = -2.4995: half, carried into the fraction
        ("2.5", "1.25025", "-2.5"),
    ] {
        let graded = grade(baseline, value);
        assert_eq!(
            verdict_and_score(&graded),
            (Verdict::OK, Some(expected)),
            "count {value} against fixed baseline {baseline}: {graded:?}"
        );
    }
}

/// Below a negative non-integer baseline the step is to the first whole count above it:
/// `floor(-1.5) + 1 - (-1.5) = 0.5`, so count -2 scores -1 and -1.75 scores -0.5; count -1 is
/// above it and `WARN`.
#[test]
fn adv_p2_negative_fractional_baseline_steps_to_the_next_whole_count() {
    for (value, expected) in [
        ("-2", (Verdict::OK, Some("-1"))),
        ("-1.75", (Verdict::OK, Some("-0.5"))),
        ("-1.5", (Verdict::OK, Some("0"))),
        ("-1", (Verdict::WARN, Some("1"))),
    ] {
        let graded = grade("-1.5", value);
        assert_eq!(
            verdict_and_score(&graded),
            expected,
            "count {value} against fixed baseline -1.5: {graded:?}"
        );
    }
}

/// Text outside the wire grammar of a decimal is no count: `UNKNOWN`, with no score, whichever
/// side of the baseline it would read as.
#[test]
fn adv_p2_non_decimal_count_forms_are_unknown() {
    for value in [
        "4e0", "1E2", "+4", ".5", "4.", "", "-", "4 ", "0x4", "--4", "1.2.3",
    ] {
        let graded = grade("3", value);
        assert_eq!(
            verdict_and_score(&graded),
            (Verdict::UNKNOWN, None),
            "count {value:?} against fixed baseline 3: {graded:?}"
        );
    }
}
