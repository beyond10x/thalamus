//! The daily rollup builder (`thalamus_core::rollup`, design § 4, § 11 D14, D27): one row per
//! series per UTC day from that series' samples and their evaluations.
//!
//! Every expected figure is a literal worked out by hand from the samples below, never
//! recomputed through the builder.

use std::collections::BTreeMap;

use thalamus_core::rollup::{build, day_start, day_text, rollup_id};
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::rollup::{RecordRollup, VerdictCounts};

const SIGNAL: &str = "service.queue.depth";
/// 2026-10-16T00:00:00Z.
const DAY: i64 = 1_792_108_800;

fn series() -> SeriesKey {
    SeriesKey(SIGNAL.to_owned())
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    })
}

/// A partial reading: its value is a bound, never counted as a value (design § 2.2).
fn partial(value: &str) -> Reading {
    Reading::Partial(PartialReading {
        value: Decimal(value.to_owned()),
        bound: Bound::Lower,
    })
}

fn unknown() -> Reading {
    Reading::Unknown(UnreadReading {
        reason: "read timed out".to_owned(),
    })
}

fn sample(id: u64, ts: &str, reading: Reading) -> SignalSampleData {
    SignalSampleData {
        sample_id: SampleId(format!("smp_{id:012}")),
        signal: SignalId(SIGNAL.to_owned()),
        series: series(),
        dims: BTreeMap::new(),
        ts: Timestamp(ts.to_owned()),
        window: TimeWindow {
            start: Timestamp(ts.to_owned()),
            end: Timestamp(ts.to_owned()),
        },
        reading,
        text: None,
        unit: Unit("{message}".to_owned()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".to_owned(),
            version: "0".to_owned(),
            definition_rev: 1,
            adapter: AdapterId("metrics".to_owned()),
            connection: ConnectionId("metrics-main".to_owned()),
            query_sha256: QueryDigest("0".repeat(64)),
            run_id: RunId("run_000000000001".to_owned()),
        },
    }
}

fn evaluation(
    id: u64,
    of: &SignalSampleData,
    rev: i64,
    verdict: Verdict,
    score: Option<&str>,
) -> SignalEvaluationData {
    SignalEvaluationData {
        evaluation_id: EvaluationId(format!("evl_{id:012}")),
        sample_id: of.sample_id.clone(),
        signal: of.signal.clone(),
        series: of.series.clone(),
        dims: of.dims.clone(),
        ts: of.ts.clone(),
        definition_rev: rev,
        verdict,
        prev: None,
        score: score.map(|score| Decimal(score.to_owned())),
        baseline: None,
        reason: (verdict != Verdict::OK).then(|| "graded".to_owned()),
    }
}

fn roll(samples: &[SignalSampleData], evaluations: &[SignalEvaluationData]) -> RecordRollup {
    build(
        &SignalId(SIGNAL.to_owned()),
        &series(),
        DAY,
        samples,
        evaluations,
    )
}

fn decimal(text: &str) -> Option<Decimal> {
    Some(Decimal(text.to_owned()))
}

/// The all-zero counts.
fn zero_counts() -> VerdictCounts {
    VerdictCounts {
        ok: 0,
        na: 0,
        warn: 0,
        crit: 0,
        unknown: 0,
    }
}

/// D27: every row thalamus writes holds `unknown + partial <= n`.
fn holds_the_sum(row: &RecordRollup) {
    assert!(
        row.unknown + row.partial <= row.n,
        "unknown {} + partial {} <= n {}",
        row.unknown,
        row.partial,
        row.n
    );
}

#[test]
fn rollup_counts_n_unknown_partial() {
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", ok("1")),
        sample(2, "2026-10-16T01:00:00Z", unknown()),
        sample(3, "2026-10-16T02:00:00Z", partial("9")),
        sample(4, "2026-10-16T03:00:00Z", unknown()),
        sample(5, "2026-10-16T04:00:00Z", ok("2")),
    ];
    let row = roll(&samples, &[]);
    assert_eq!((row.n, row.unknown, row.partial), (5, 2, 1));
    holds_the_sum(&row);
    assert_eq!(row.day.0, "2026-10-16T00:00:00Z");
    assert_eq!(row.signal.0, SIGNAL);
    assert_eq!(row.series, series());
}

#[test]
fn rollup_values_from_ok_readings_only() {
    // ok values 5, 1, 10, 3 in time order; the partial 1000 and the partial -7 are bounds, not
    // values. min 1, max 10, p50 the mean of 3 and 5 = 4 (the baseline median's spelling), last the 03:00 reading 3.
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", ok("5")),
        sample(2, "2026-10-16T00:30:00Z", partial("1000")),
        sample(3, "2026-10-16T01:00:00Z", ok("1")),
        sample(4, "2026-10-16T02:00:00Z", ok("10")),
        sample(5, "2026-10-16T02:30:00Z", partial("-7")),
        sample(6, "2026-10-16T03:00:00Z", ok("3")),
        sample(7, "2026-10-16T04:00:00Z", unknown()),
    ];
    let row = roll(&samples, &[]);
    assert_eq!(row.min, decimal("1"));
    assert_eq!(row.max, decimal("10"));
    assert_eq!(row.p50, decimal("4"));
    assert_eq!(row.last, decimal("3"));
    assert_eq!((row.n, row.unknown, row.partial), (7, 1, 2));
    holds_the_sum(&row);

    // An odd count's p50 is its middle value; `last` follows the timestamps, not the order the
    // samples arrive in.
    let shuffled = [
        sample(3, "2026-10-16T09:00:00Z", ok("2.5")),
        sample(1, "2026-10-16T07:00:00Z", ok("-1")),
        sample(2, "2026-10-16T08:00:00Z", ok("40")),
    ];
    let row = roll(&shuffled, &[]);
    assert_eq!(row.p50, decimal("2.5"));
    assert_eq!(row.min, decimal("-1"));
    assert_eq!(row.max, decimal("40"));
    assert_eq!(row.last, decimal("2.5"));
}

#[test]
fn rollup_without_ok_reading_has_no_values() {
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", unknown()),
        sample(2, "2026-10-16T01:00:00Z", partial("12")),
    ];
    let row = roll(&samples, &[]);
    assert_eq!(
        (&row.min, &row.p50, &row.max, &row.last),
        (&None, &None, &None, &None)
    );
    assert_eq!((row.n, row.unknown, row.partial), (2, 1, 1));
    holds_the_sum(&row);

    let empty = roll(&[], &[]);
    assert_eq!(empty.n, 0);
    assert_eq!(
        (empty.min, empty.p50, empty.max, empty.last),
        (None, None, None, None)
    );
    assert_eq!(empty.worst_verdict, None);
    assert_eq!((empty.min_score, empty.max_score), (None, None));
}

#[test]
fn rollup_day_is_utc() {
    // 2026-10-16T23:30:00-02:00 is 2026-10-17T01:30Z: the next day. 2026-10-17T00:30:00+02:00
    // is 2026-10-16T22:30Z: this day. The day's bounds are UTC midnights, start included, end
    // excluded.
    let samples = [
        sample(1, "2026-10-15T23:59:59Z", ok("100")),
        sample(2, "2026-10-16T00:00:00Z", ok("1")),
        sample(3, "2026-10-16T23:30:00-02:00", ok("200")),
        sample(4, "2026-10-17T00:30:00+02:00", ok("2")),
        sample(5, "2026-10-16T23:59:59.999Z", ok("3")),
        sample(6, "2026-10-17T00:00:00Z", ok("300")),
    ];
    let row = roll(&samples, &[]);
    assert_eq!(row.n, 3);
    assert_eq!(row.max, decimal("3"));
    assert_eq!(row.last, decimal("3"));
    assert_eq!(row.day.0, "2026-10-16T00:00:00Z");

    assert_eq!(day_start("2026-10-17T00:30:00+02:00"), Some(DAY));
    assert_eq!(day_start("2026-10-16T23:30:00-02:00"), Some(DAY + 86_400));
    assert_eq!(day_text(DAY), "2026-10-16");
    assert_eq!(
        rollup_id(&series(), DAY).0,
        "service.queue.depth@2026-10-16"
    );
}

#[test]
fn rollup_worst_verdict_and_verdict_counts() {
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", ok("1")),
        sample(2, "2026-10-16T01:00:00Z", ok("150")),
        sample(3, "2026-10-16T02:00:00Z", unknown()),
        sample(4, "2026-10-16T03:00:00Z", ok("2")),
        sample(5, "2026-10-16T04:00:00Z", ok("3")),
        sample(6, "2026-10-16T05:00:00Z", ok("4")),
    ];
    let evaluations = [
        evaluation(1, &samples[0], 1, Verdict::OK, Some("0")),
        evaluation(2, &samples[1], 1, Verdict::WARN, Some("1.5")),
        evaluation(3, &samples[2], 1, Verdict::UNKNOWN, None),
        evaluation(4, &samples[3], 1, Verdict::NA, None),
        evaluation(5, &samples[4], 1, Verdict::OK, Some("0")),
        // Sample 6 has no evaluation: counted in n, in no verdict.
    ];
    let row = roll(&samples, &evaluations);
    assert_eq!(row.worst_verdict, Some(Verdict::WARN));
    assert_eq!(
        row.verdict_counts,
        VerdictCounts {
            ok: 2,
            na: 1,
            warn: 1,
            crit: 0,
            unknown: 1,
        }
    );

    let with_crit = [
        evaluation(1, &samples[0], 1, Verdict::CRIT, Some("12")),
        evaluation(2, &samples[1], 1, Verdict::WARN, Some("1.5")),
    ];
    let row = roll(&samples, &with_crit);
    assert_eq!(row.worst_verdict, Some(Verdict::CRIT));
    assert_eq!(
        row.verdict_counts,
        VerdictCounts {
            ok: 0,
            na: 0,
            warn: 1,
            crit: 1,
            unknown: 0,
        }
    );

    // Only UNKNOWN and OK: UNKNOWN ranks above OK.
    let unknown_only = [
        evaluation(1, &samples[0], 1, Verdict::OK, Some("0")),
        evaluation(3, &samples[2], 1, Verdict::UNKNOWN, None),
    ];
    assert_eq!(
        roll(&samples, &unknown_only).worst_verdict,
        Some(Verdict::UNKNOWN)
    );

    // An evaluation of a sample outside the day, or of another series, is not counted.
    let other_day = sample(9, "2026-10-17T05:00:00Z", ok("4"));
    let outside = [evaluation(9, &other_day, 1, Verdict::CRIT, Some("99"))];
    let row = roll(&samples, &outside);
    assert_eq!(row.worst_verdict, None);
    assert_eq!(row.verdict_counts, zero_counts());
}

#[test]
fn rollup_min_max_score_over_scored_verdicts_only() {
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", ok("1")),
        sample(2, "2026-10-16T01:00:00Z", ok("150")),
        sample(3, "2026-10-16T02:00:00Z", ok("1500")),
        sample(4, "2026-10-16T03:00:00Z", unknown()),
        sample(5, "2026-10-16T04:00:00Z", ok("0")),
    ];
    let evaluations = [
        evaluation(1, &samples[0], 1, Verdict::OK, Some("0.25")),
        evaluation(2, &samples[1], 1, Verdict::WARN, Some("1.5")),
        evaluation(3, &samples[2], 1, Verdict::CRIT, Some("15")),
        evaluation(4, &samples[3], 1, Verdict::UNKNOWN, None),
        evaluation(5, &samples[4], 1, Verdict::NA, None),
    ];
    let row = roll(&samples, &evaluations);
    assert_eq!(row.min_score, decimal("0.25"));
    assert_eq!(row.max_score, decimal("15"));

    // Numeric order, not text order: "9" < "10".
    let numeric = [
        evaluation(1, &samples[0], 1, Verdict::WARN, Some("9")),
        evaluation(2, &samples[1], 1, Verdict::CRIT, Some("10")),
    ];
    let row = roll(&samples, &numeric);
    assert_eq!(row.min_score, decimal("9"));
    assert_eq!(row.max_score, decimal("10"));

    // Evaluations, none of them scored: absent, never 0.
    let unscored = [
        evaluation(4, &samples[3], 1, Verdict::UNKNOWN, None),
        evaluation(5, &samples[4], 1, Verdict::NA, None),
    ];
    let row = roll(&samples, &unscored);
    assert_eq!((row.min_score, row.max_score), (None, None));
    assert_eq!(row.worst_verdict, Some(Verdict::UNKNOWN));

    // No evaluation at all: absent too.
    let row = roll(&samples, &[]);
    assert_eq!((row.min_score, row.max_score), (None, None));
    assert_eq!(row.worst_verdict, None);
}

#[test]
fn rollup_counts_highest_rev_evaluation_per_sample() {
    // D14: of each sample's evaluations, the one at the highest definition_rev present counts.
    // Sample 1: rev 1 CRIT, rev 3 OK, rev 2 WARN -> OK (rev 3), whatever the order they arrive in.
    // Sample 2: rev 1 OK only -> OK. Sample 3: rev 2 WARN, rev 1 OK -> WARN.
    let samples = [
        sample(1, "2026-10-16T00:00:00Z", ok("1")),
        sample(2, "2026-10-16T01:00:00Z", ok("2")),
        sample(3, "2026-10-16T02:00:00Z", ok("3")),
    ];
    let evaluations = [
        evaluation(1, &samples[0], 1, Verdict::CRIT, Some("40")),
        evaluation(2, &samples[0], 3, Verdict::OK, Some("0.5")),
        evaluation(3, &samples[0], 2, Verdict::WARN, Some("2")),
        evaluation(4, &samples[1], 1, Verdict::OK, Some("0.75")),
        evaluation(5, &samples[2], 2, Verdict::WARN, Some("1.25")),
        evaluation(6, &samples[2], 1, Verdict::OK, Some("0.1")),
    ];
    let row = roll(&samples, &evaluations);
    assert_eq!(
        row.verdict_counts,
        VerdictCounts {
            ok: 2,
            na: 0,
            warn: 1,
            crit: 0,
            unknown: 0,
        }
    );
    assert_eq!(row.worst_verdict, Some(Verdict::WARN));
    assert_eq!(row.min_score, decimal("0.5"));
    assert_eq!(row.max_score, decimal("1.25"));
    // One counted evaluation per sample, never more than n.
    let counts = &row.verdict_counts;
    assert_eq!(
        counts.ok + counts.na + counts.warn + counts.crit + counts.unknown,
        3
    );
    assert_eq!(rollup_id(&series(), DAY), row.rollup_id);
}
