//! Adversary pass 1 on `story:m3-advisory-gate-mapping` (wave 20261015f): the rule
//! `fixed_warn_only` driven against what its own module documentation, the design row it wrote
//! and design § 3.1 rule 4 / § 3.3 say.
//!
//! - "The verdict is decided exactly, on the decimals": a baseline written with trailing
//!   fractional zeros is the same number, so a count 100 times it is `WARN`, as the acceptance
//!   statement's own example says.
//! - "0 at the baseline": the score at the baseline is 0 for every baseline the loader admits.
//! - "first whole count above the baseline": the score's unit step is measured to the first whole
//!   count above a non-integer baseline, not to the baseline plus 1.
//! - § 3.1 rule 4: a partial read never clears; it is `UNKNOWN` with reason `partial read` unless
//!   it escalates to `CRIT`, which this rule never grades.

use std::collections::BTreeMap;

use thalamus_core::grading::{self, Graded, REASON_PARTIAL};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Unit,
};
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

const SIGNAL: &str = "gate.ci.advisory_findings";

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
fn grade(baseline: &str, reading: Reading) -> Graded {
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: count(baseline),
    };
    grading::grade(&definition(baseline), &sample(reading), &fixed)
        .expect("a fixed_warn_only definition grades")
}

fn verdict_and_score(graded: &Graded) -> (Verdict, Option<&str>) {
    (
        graded.verdict,
        graded.score.as_ref().map(|score| score.0.as_str()),
    )
}

/// The acceptance's own example, a count 100 times the baseline, with the baseline written with
/// trailing fractional zeros (the loader's `is_decimal` admits it and `baseline::fold` passes it
/// verbatim). "Decided exactly, on the decimals": `3.000…0` is 3, so 300 is `WARN`, as it is
/// against `3`. The threshold rules normalise trailing zeros (`threshold::Num::normal`);
/// `warn_only::Exact` does not, so aligning 300 to 36 fractional digits overflows i128.
#[test]
fn adv_hundred_times_a_baseline_with_trailing_zeros_is_warn() {
    let plain = grade("3", count("300"));
    assert_eq!(
        verdict_and_score(&plain),
        (Verdict::WARN, Some("1")),
        "{plain:?}"
    );

    let zeros = format!("3.{}", "0".repeat(36));
    let graded = grade(&zeros, count("300"));
    assert_eq!(
        verdict_and_score(&graded),
        (Verdict::WARN, Some("1")),
        "count 300 against fixed baseline {zeros}: {graded:?}"
    );
}

/// "0 at the baseline" (module doc, design row, design § 3.3), for a whole-number baseline past
/// 2^53: `b.floor() + 1.0` rounds back to `b` in f64, the score divides 0 by 0, and `render` writes
/// the NaN as `f64::MIN`. One below the baseline is -1 (one whole count under it).
#[test]
fn adv_score_is_zero_at_a_baseline_past_f64_integers() {
    let baseline = "9007199254740992";
    let at = grade(baseline, count(baseline));
    assert_eq!(
        verdict_and_score(&at),
        (Verdict::OK, Some("0")),
        "at the baseline: {at:?}"
    );
    let below = grade(baseline, count("9007199254740991"));
    assert_eq!(
        verdict_and_score(&below),
        (Verdict::OK, Some("-1")),
        "one count under the baseline: {below:?}"
    );
}

/// The score at a non-integer baseline: 0 at the baseline, and the unit step runs to the first
/// whole count above it (design row: `(count - baseline) / (first whole count above the baseline
/// - baseline)`). Against 2.5 the first whole count above is 3, so 2 scores (2 - 2.5) / 0.5 = -1
/// and 0 scores -5. Every test of the unit uses the integer baseline 3, where `b + 1` and
/// `floor(b) + 1` agree, so the mutant `first_above = b + 1.0` survives the unit's suite.
#[test]
fn adv_non_integer_baseline_scores_to_the_first_whole_count_above() {
    let at = grade("2.5", count("2.5"));
    assert_eq!(verdict_and_score(&at), (Verdict::OK, Some("0")), "{at:?}");
    let below = grade("2.5", count("2"));
    assert_eq!(
        verdict_and_score(&below),
        (Verdict::OK, Some("-1")),
        "{below:?}"
    );
    let zero = grade("2.5", count("0"));
    assert_eq!(
        verdict_and_score(&zero),
        (Verdict::OK, Some("-5")),
        "{zero:?}"
    );
    let above = grade("2.5", count("3"));
    assert_eq!(
        verdict_and_score(&above),
        (Verdict::WARN, Some("1")),
        "{above:?}"
    );
}

/// Design § 3.1 rule 4: a partial sample never clears and, short of `CRIT`, is `UNKNOWN` with
/// reason `partial read`. The unit's own case asserts only `!= CRIT`, which a rule grading the
/// bound as a count (`WARN` for a lower bound of 300, `OK` for an upper bound of 1) also passes.
#[test]
fn adv_partial_read_is_unknown_partial_read_either_bound() {
    for (value, bound) in [
        ("300", Bound::Lower),
        ("1", Bound::Upper),
        ("3", Bound::Upper),
    ] {
        let graded = grade(
            "3",
            Reading::Partial(PartialReading {
                value: Decimal(value.into()),
                bound,
            }),
        );
        assert_eq!(
            (
                graded.verdict,
                graded.score.as_ref(),
                graded.reason.as_str()
            ),
            (Verdict::UNKNOWN, None, REASON_PARTIAL),
            "partial {value} {bound:?}: {graded:?}"
        );
    }
}

/// A negative fixed baseline is a decimal the loader admits; a count of 0 is above it, so `WARN`.
/// Kills the mutant that drops the sign in `warn_only::Exact::parse`.
#[test]
fn adv_count_above_a_negative_baseline_is_warn() {
    let graded = grade("-1", count("0"));
    assert_eq!(
        verdict_and_score(&graded),
        (Verdict::WARN, Some("1")),
        "{graded:?}"
    );
    let at = grade("-1", count("-1"));
    assert_eq!(verdict_and_score(&at), (Verdict::OK, Some("0")), "{at:?}");
}
