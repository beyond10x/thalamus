//! Adversary pass 1 on `story:m2-store-baselines`: the edges of `same_slot_median_4w` and
//! `rolling_median` (design § 3.1) the unit's own suite does not state.

use std::collections::BTreeMap;

use thalamus_core::baseline::{self, REASON_SERIES_AGE_UNKNOWN};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::evaluation::Baseline;
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey, SeriesSamples,
    SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

const DAY: i64 = 86_400;
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

fn row(id: &str, ts: &str, reading: Reading) -> SeriesSamples {
    SeriesSamples {
        sample_id: SampleId(id.into()),
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

/// One ok row in each of the four Wednesday 22:00 UTC slots of `2026-10-14T22:xx`, at `minute`.
fn slots(minute: &str, values: [&str; 4]) -> Vec<SeriesSamples> {
    ["2026-10-07", "2026-09-30", "2026-09-23", "2026-09-16"]
        .iter()
        .zip(values)
        .map(|(day, value)| {
            let ts = format!("{day}T22:{minute}Z");
            row(&format!("smp_{ts}"), &ts, ok(value))
        })
        .collect()
}

/// A series sampled twice at one instant (`thalamus sample --at` run again for the same window,
/// a re-read after a source outage) has two rows there, in the order they were appended. The
/// rolling baseline (`rolling.rs:35`) and the project's convention for one instant (design § 11
/// D6, `evaluate.rs` `previous_verdict`, `evaluations.rs:225`) take the row appended last; the
/// same-slot baseline must read the series the same way, or a corrected read changes one
/// store baseline and not the other.
#[test]
fn same_slot_reads_the_row_appended_last_at_one_instant_as_rolling_does() {
    let graded = "2026-10-14T22:30:00Z";
    let mut history = slots("30:00", ["10", "20", "30", "40"]);
    // The 7-day slot's instant is read again; the re-read is appended after the first read.
    history.insert(1, row("smp_reread", "2026-10-07T22:30:00Z", ok("90")));

    // rolling_median over 1, graded just after the 7-day slot: the re-read, appended last.
    let latest = fold(&rolling(1), "2026-10-07T22:35:00Z", &history, OLD);
    assert_eq!(value(&latest), "90", "{latest:#?}");

    // same_slot_median_4w: sorted 20, 30, 40, 90 -> 35 when the 7-day slot reads the re-read
    // as rolling does; 25 (10, 20, 30, 40) when it reads the first, superseded row.
    let folded = fold(&same_slot(), graded, &history, OLD);
    assert_eq!(
        value(&folded),
        "35",
        "the 7-day slot read the row appended first, not the re-read rolling_median reads: \
         {folded:#?}"
    );
}

/// A graded `ts` exactly on the hour: its slot is the hour it opens, so a stored row exactly at
/// the slot instant is the slot's reading, and one a second before it is the previous hour's.
#[test]
fn same_slot_on_the_hour_reads_the_hour_it_opens() {
    let graded = "2026-10-14T22:00:00Z";
    let mut history = slots("00:00", ["1", "2", "3", "4"]);
    history.push(row("smp_before", "2026-10-07T21:59:59Z", ok("1000")));
    let folded = fold(&same_slot(), graded, &history, OLD);
    assert_eq!(value(&folded), "2.5", "{folded:#?}");

    // Only the row a second before the slot: the slot is not read.
    let only_before: Vec<SeriesSamples> = history
        .iter()
        .filter(|row| !row.ts.0.starts_with("2026-10-07T22"))
        .cloned()
        .collect();
    let folded = fold(&same_slot(), graded, &only_before, OLD);
    assert_eq!(reason(&folded), "same slot not read", "{folded:#?}");
}

/// Two ok rows equally far from the slot instant, one either side: the earlier, as documented.
#[test]
fn same_slot_tie_takes_the_earlier() {
    let graded = "2026-10-14T22:30:00Z";
    let mut history = slots("10:00", ["7", "7", "7", "7"]);
    history.retain(|row| !row.ts.0.starts_with("2026-10-07"));
    history.push(row("smp_late", "2026-10-07T22:40:00Z", ok("100")));
    history.push(row("smp_early", "2026-10-07T22:20:00Z", ok("1")));
    // Slots: 1, 7, 7, 7 -> median 7.
    let folded = fold(&same_slot(), graded, &history, OLD);
    assert_eq!(value(&folded), "7", "{folded:#?}");
    // Make the earlier one decisive: 1, 2, 7, 7 -> 4.5; with the later it would be 7.
    history.retain(|row| !row.ts.0.starts_with("2026-09-30"));
    history.push(row("smp_w2", "2026-09-30T22:30:00Z", ok("2")));
    let folded = fold(&same_slot(), graded, &history, OLD);
    assert_eq!(value(&folded), "4.5", "{folded:#?}");
}

/// Rule 2 with an unknown age names it, and is decided before the slots: never a number.
#[test]
fn same_slot_unknown_age_names_its_reason() {
    let folded = fold(
        &same_slot(),
        "2026-10-14T22:30:00Z",
        &slots("30:00", ["1", "2", "3", "4"]),
        None,
    );
    assert_eq!(reason(&folded), REASON_SERIES_AGE_UNKNOWN, "{folded:#?}");
}

/// The even-count median renders in the wire grammar: no trailing zero, never `-0`.
#[test]
fn even_median_renders_in_the_wire_grammar() {
    let ts = "2026-10-14T22:00:00Z";
    let history = |values: &[&str]| -> Vec<SeriesSamples> {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let at = format!("2026-10-14T1{index}:00:00Z");
                row(&format!("smp_{index}"), &at, ok(value))
            })
            .collect()
    };
    for (values, expected) in [
        (vec!["10", "20"], "15"),
        (vec!["-0.5", "0.5"], "0"),
        (vec!["-3", "-2"], "-2.5"),
        (vec!["0.001", "0.002"], "0.0015"),
        (
            vec![
                "99999999999999999999999999999",
                "99999999999999999999999999999",
            ],
            "99999999999999999999999999999",
        ),
    ] {
        let folded = fold(&rolling(2), ts, &history(&values), OLD);
        assert_eq!(value(&folded), expected, "{values:?}: {folded:#?}");
    }
}

/// `rolling_n` far above any history, and of exactly the history's length.
#[test]
fn rolling_n_at_and_beyond_the_history() {
    let ts = "2026-10-14T22:00:00Z";
    let history = vec![
        row("smp_a", "2026-10-14T20:00:00Z", ok("4")),
        row("smp_b", "2026-10-14T21:00:00Z", ok("8")),
    ];
    assert_eq!(value(&fold(&rolling(2), ts, &history, OLD)), "6");
    let folded = fold(&rolling(i64::MAX), ts, &history, OLD);
    assert_eq!(reason(&folded), "baseline predates series", "{folded:#?}");
}
