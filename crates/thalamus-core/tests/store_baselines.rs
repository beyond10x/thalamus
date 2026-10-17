//! `story:m2-store-baselines`: the two baselines only stored history answers (design § 3.1),
//! `same_slot_median_4w` and `rolling_median`, computed from the samples of the series the caller
//! passes. A slot or a count that could not be read is `UNKNOWN`, never 0 and never a median of
//! fewer values (rule 1); one older than the series is `UNKNOWN` with reason `baseline predates
//! series` (rule 2).

use std::collections::BTreeMap;

use thalamus_core::baseline::{
    self, REASON_PREDATES_SERIES, REASON_SAME_SLOT_NOT_READ, Refusal, StoredHistory,
};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::evaluation::Baseline;
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SeriesSamples, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

const DAY: i64 = 86_400;

/// Wednesday 2026-10-14, 22:57:03 UTC: its slots are the Wednesdays before, 22:00 to 23:00 UTC.
const TS: &str = "2026-10-14T22:57:03Z";

/// A series far older than four weeks.
const OLD: Option<i64> = Some(400 * DAY);

const SIGNAL: &str = "service.queue.depth";

fn definition(baselines: &[BaselineKind], rolling_n: Option<i64>) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId(SIGNAL.into()),
        title: "Messages waiting in the work queue".into(),
        family: Family("service".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("{message}".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("metrics".into()),
            connection: ConnectionId("metrics-main".into()),
        },
        extractor: ExtractorId("promql.value".into()),
        query: BTreeMap::new(),
        window: Duration("PT5M".into()),
        cadence: Duration("PT5M".into()),
        history: History::Store,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId("higher_is_worse".into()),
            baselines: baselines.to_vec(),
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: None,
            rolling_n,
            mapping: None,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:service".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-09-01T00:00:00Z".into()),
        revised_by: Author("someone".into()),
        mute_reason: None,
        mute_until: None,
    }
}

fn same_slot() -> SignalDefinitionData {
    definition(&[BaselineKind::SameSlotMedian4w], None)
}

fn rolling(n: i64) -> SignalDefinitionData {
    definition(&[BaselineKind::RollingMedian], Some(n))
}

/// The graded sample, at `ts`.
fn sample(ts: &str) -> SignalSampleData {
    SignalSampleData {
        text: None,
        sample_id: SampleId("smp_graded".into()),
        signal: SignalId(SIGNAL.into()),
        series: SeriesKey(SIGNAL.into()),
        dims: BTreeMap::new(),
        ts: Timestamp(ts.into()),
        window: TimeWindow {
            start: Timestamp(ts.into()),
            end: Timestamp(ts.into()),
        },
        reading: ok("50"),
        unit: Unit("{message}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: 1,
            adapter: AdapterId("metrics".into()),
            connection: ConnectionId("metrics-main".into()),
            query_sha256: QueryDigest("00".into()),
            run_id: RunId("run_1".into()),
        },
    }
}

/// One stored row of the series, as `thalamus.observation.SeriesSamples` serves it.
fn row(ts: &str, reading: Reading) -> SeriesSamples {
    SeriesSamples {
        sample_id: SampleId(format!("smp_{ts}")),
        ts: Timestamp(ts.into()),
        window: TimeWindow {
            start: Timestamp(ts.into()),
            end: Timestamp(ts.into()),
        },
        reading,
        text: None,
        unit: Unit("{message}".into()),
        denominator: None,
    }
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn unknown(reason: &str) -> Reading {
    Reading::Unknown(UnreadReading {
        reason: reason.into(),
    })
}

fn partial(value: &str) -> Reading {
    Reading::Partial(PartialReading {
        value: Decimal(value.into()),
        bound: Bound::Lower,
    })
}

fn fold(
    definition: &SignalDefinitionData,
    ts: &str,
    history: &[SeriesSamples],
    age: Option<i64>,
) -> Baseline {
    baseline::fold_with_history(definition, &sample(ts), &[], history, age)
        .expect("the store baseline folds")
}

fn value(baseline: &Baseline) -> &str {
    match &baseline.reading {
        Reading::Ok(CompleteReading { value }) => &value.0,
        other => panic!("expected an ok baseline, got {other:?}"),
    }
}

fn reason(baseline: &Baseline) -> &str {
    match &baseline.reading {
        Reading::Unknown(UnreadReading { reason }) => reason,
        other => panic!("expected an unknown baseline, never a number, got {other:?}"),
    }
}

/// The four slots of [`TS`] and a decoy in or beside each.
fn four_weeks(values: [&str; 4]) -> Vec<SeriesSamples> {
    let days = ["2026-10-07", "2026-09-30", "2026-09-23", "2026-09-16"];
    let mut history = Vec::new();
    for (day, value) in days.iter().zip(values) {
        // In the slot's hour, further from the instant than the one below: not the slot's reading.
        history.push(row(&format!("{day}T22:05:00Z"), ok("999")));
        // The sample closest to the instant within the hour.
        history.push(row(&format!("{day}T22:56:00Z"), ok(value)));
        // Closer to the instant, but in the next hour: outside the slot.
        history.push(row(&format!("{day}T23:00:00Z"), ok("777")));
    }
    history.sort_by(|left, right| left.ts.0.cmp(&right.ts.0));
    history
}

#[test]
fn same_slot_median_of_four_weeks() {
    // Sorted 10, 21, 30, 40: an even count is the mean of the two middle values, in decimal.
    let folded = fold(&same_slot(), TS, &four_weeks(["10", "40", "21", "30"]), OLD);
    assert_eq!(folded.kind.0, "same_slot_median_4w");
    assert_eq!(value(&folded), "25.5", "{folded:#?}");

    // Decimal, not float: the mean of 0.1 and 0.2 is exactly 0.15.
    let folded = fold(
        &same_slot(),
        TS,
        &four_weeks(["0.1", "0.2", "0.1", "0.2"]),
        OLD,
    );
    assert_eq!(value(&folded), "0.15", "{folded:#?}");

    // Negative values and differing scales.
    let folded = fold(
        &same_slot(),
        TS,
        &four_weeks(["-3", "-1.25", "7", "-2"]),
        OLD,
    );
    assert_eq!(value(&folded), "-1.625", "{folded:#?}");
}

#[test]
fn same_slot_missing_week_is_unknown() {
    let full = four_weeks(["10", "40", "21", "30"]);

    // The 21-day slot holds no sample at all.
    let missing: Vec<SeriesSamples> = full
        .iter()
        .filter(|row| !row.ts.0.starts_with("2026-09-23"))
        .cloned()
        .collect();
    let folded = fold(&same_slot(), TS, &missing, OLD);
    assert_eq!(reason(&folded), REASON_SAME_SLOT_NOT_READ, "{folded:#?}");
    assert_eq!(folded.kind.0, "same_slot_median_4w");

    // The 14-day slot holds only an unknown and a partial reading: neither is an `ok` reading,
    // so the slot was not read, and three weeks are not a four-week median.
    let mut unread: Vec<SeriesSamples> = full
        .iter()
        .filter(|row| !row.ts.0.starts_with("2026-09-30"))
        .cloned()
        .collect();
    unread.push(row("2026-09-30T22:56:00Z", unknown("source down")));
    unread.push(row("2026-09-30T22:40:00Z", partial("12")));
    let folded = fold(&same_slot(), TS, &unread, OLD);
    assert_eq!(reason(&folded), REASON_SAME_SLOT_NOT_READ, "{folded:#?}");

    // No history at all.
    let folded = fold(&same_slot(), TS, &[], OLD);
    assert_eq!(reason(&folded), REASON_SAME_SLOT_NOT_READ, "{folded:#?}");
}

#[test]
fn same_slot_before_series_is_unknown() {
    let full = four_weeks(["10", "40", "21", "30"]);

    // A series 20 days old: the 21- and 28-day slots lie before it began.
    let folded = fold(&same_slot(), TS, &full, Some(20 * DAY));
    assert_eq!(reason(&folded), REASON_PREDATES_SERIES, "{folded:#?}");

    // The series began at 2026-09-16T23:27:03Z, after the 28-day slot's hour ended.
    let folded = fold(&same_slot(), TS, &full, Some(28 * DAY - 1800));
    assert_eq!(reason(&folded), REASON_PREDATES_SERIES, "{folded:#?}");

    // The series began at 2026-09-16T22:27:03Z, inside the 28-day slot's hour: the slot is read.
    let folded = fold(&same_slot(), TS, &full, Some(28 * DAY + 1800));
    assert_eq!(value(&folded), "25.5", "{folded:#?}");

    // An unknown series age is not an old one.
    let folded = fold(&same_slot(), TS, &full, None);
    assert!(
        matches!(folded.reading, Reading::Unknown(_)),
        "an unknown age reads nothing: {folded:#?}"
    );
}

#[test]
fn same_slot_uses_utc_weekday_and_hour() {
    // Wednesday 2026-11-04 10:30 at +01:00 is 09:30 UTC. The slots are 09:00 to 10:00 UTC on the
    // four Wednesdays before, whatever offset a stored `ts` is written in; three of them fall
    // before a change of local offset, where local 10:30 is 08:30 UTC.
    let graded = "2026-11-04T10:30:00+01:00";
    let history = vec![
        row("2026-10-07T10:30:00+02:00", ok("100")),
        row("2026-10-07T11:30:00+02:00", ok("4")),
        row("2026-10-14T10:30:00+02:00", ok("100")),
        row("2026-10-14T09:30:00Z", ok("3")),
        row("2026-10-21T10:30:00+02:00", ok("100")),
        row("2026-10-21T11:30:00+02:00", ok("2")),
        row("2026-10-28T10:30:00+01:00", ok("1")),
        // Thursday 2026-10-29 at 09:30 UTC: the hour matches, the weekday does not.
        row("2026-10-29T09:30:00Z", ok("100")),
    ];
    let folded = fold(&same_slot(), graded, &history, OLD);
    assert_eq!(value(&folded), "2.5", "{folded:#?}");

    // A ts written at an offset that moves it to the previous UTC day: Thursday 00:30 at +02:00
    // is Wednesday 22:30 UTC, so its slots are the Wednesdays of `four_weeks`.
    let folded = fold(
        &same_slot(),
        "2026-10-15T00:30:00+02:00",
        &[
            row("2026-10-07T22:30:00Z", ok("1")),
            row("2026-09-30T22:30:00Z", ok("2")),
            row("2026-09-23T22:30:00Z", ok("3")),
            row("2026-09-16T22:30:00Z", ok("5")),
        ],
        OLD,
    );
    assert_eq!(value(&folded), "2.5", "{folded:#?}");
}

/// Hourly rows ending one hour before [`TS`], oldest first.
fn hourly(readings: Vec<Reading>) -> Vec<SeriesSamples> {
    let count = readings.len();
    readings
        .into_iter()
        .enumerate()
        .map(|(index, reading)| {
            let hours_before = count - index;
            let hour = 22 - hours_before;
            row(&format!("2026-10-14T{hour:02}:57:03Z"), reading)
        })
        .collect()
}

#[test]
fn rolling_median_of_last_n_ok() {
    let mut history = hourly(vec![ok("1"), ok("2"), ok("3"), ok("4"), ok("5")]);
    // The graded sample's own instant and later: not before the graded sample.
    history.push(row(TS, ok("999")));
    history.push(row("2026-10-14T23:30:00Z", ok("999")));

    let folded = fold(&rolling(3), TS, &history, OLD);
    assert_eq!(folded.kind.0, "rolling_median");
    assert_eq!(value(&folded), "4", "the last 3 are 3, 4, 5: {folded:#?}");

    let folded = fold(&rolling(4), TS, &history, OLD);
    assert_eq!(
        value(&folded),
        "3.5",
        "the last 4 are 2, 3, 4, 5: {folded:#?}"
    );

    let folded = fold(&rolling(1), TS, &history, OLD);
    assert_eq!(value(&folded), "5", "{folded:#?}");
}

#[test]
fn rolling_median_skips_unknown_and_partial() {
    let history = hourly(vec![
        ok("1"),
        ok("2"),
        unknown("source down"),
        ok("3"),
        partial("100"),
        ok("6"),
        unknown("timeout"),
    ]);
    let folded = fold(&rolling(3), TS, &history, OLD);
    assert_eq!(
        value(&folded),
        "3",
        "the last 3 ok are 2, 3, 6: {folded:#?}"
    );
}

#[test]
fn rolling_median_too_few_is_unknown() {
    let history = hourly(vec![
        unknown("source down"),
        ok("2"),
        partial("100"),
        ok("6"),
    ]);
    let folded = fold(&rolling(3), TS, &history, OLD);
    assert_eq!(reason(&folded), REASON_PREDATES_SERIES, "{folded:#?}");

    let folded = fold(&rolling(1), TS, &[], OLD);
    assert_eq!(reason(&folded), REASON_PREDATES_SERIES, "{folded:#?}");
}

#[test]
fn store_baselines_plan_no_window_and_name_their_history() {
    let sample = sample(TS);
    for definition in [same_slot(), rolling(3)] {
        assert!(
            baseline::plan(&definition, &sample)
                .expect("a store baseline plans")
                .is_empty(),
            "a store baseline reads no source window"
        );
    }
    assert_eq!(
        baseline::stored_history(&same_slot()),
        Ok(Some(StoredHistory::SameSlot))
    );
    assert_eq!(
        baseline::stored_history(&rolling(3)),
        Ok(Some(StoredHistory::Rolling { n: 3 }))
    );
    assert_eq!(
        baseline::stored_history(&definition(&[BaselineKind::Offset1d], None)),
        Ok(None)
    );
}

#[test]
fn rolling_median_needs_an_n_of_at_least_one() {
    for n in [None, Some(0), Some(-3)] {
        let definition = definition(&[BaselineKind::RollingMedian], n);
        assert_eq!(
            baseline::plan(&definition, &sample(TS)),
            Err(Refusal::RollingNeedsN { n }),
        );
    }
}

#[test]
fn a_store_baseline_beside_another_kind_is_ambiguous() {
    for kinds in [
        vec![BaselineKind::SameSlotMedian4w, BaselineKind::Offset1d],
        vec![BaselineKind::RollingMedian, BaselineKind::Fixed],
        vec![BaselineKind::RollingMedian, BaselineKind::SameSlotMedian4w],
        vec![BaselineKind::SameSlotMedian4w, BaselineKind::HigherOf],
    ] {
        let definition = definition(&kinds, Some(3));
        assert_eq!(
            baseline::plan(&definition, &sample(TS)),
            Err(Refusal::Ambiguous { kinds }),
        );
    }
}

#[test]
fn rolling_median_counts_one_instant_once() {
    let mut history = hourly(vec![ok("1"), ok("2"), ok("3"), ok("4"), ok("5")]);
    // A second row at the latest instant, appended last: that instant counts once, as this row.
    history.push(row("2026-10-14T21:57:03Z", ok("50")));

    let folded = fold(&rolling(3), TS, &history, OLD);
    assert_eq!(
        value(&folded),
        "4",
        "the last 3 instants are 3, 4, 50: {folded:#?}"
    );
}
