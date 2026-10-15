//! `story:m1-baselines`: the readings of earlier windows become the baseline a sample is graded
//! against (design § 3.1), and a baseline that could not be read, or that is older than its
//! series, is `UNKNOWN`, never 0.

use std::collections::BTreeMap;

use thalamus_core::baseline::{
    self, Offset, OffsetReading, REASON_PREDATES_SERIES, REASON_SERIES_AGE_UNKNOWN, Refusal,
    WindowRequest,
};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Source,
    Unit,
};
use thalamus_model::evaluation::Baseline;
use thalamus_model::observation::{
    CompleteReading, Denominator, PartialReading, Provenance, QueryDigest, Reading, RunId,
    SampleId, SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;

/// Wednesday 2026-10-14, a 6-hour window ending at the sample's timestamp.
const TS: &str = "2026-10-14T22:57:03Z";
const START: &str = "2026-10-14T16:57:03Z";

fn definition(kind: SignalKind, baselines: &[BaselineKind]) -> SignalDefinitionData {
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
            rule: RuleId("ratio_vs_pooled_baseline".into()),
            baselines: baselines.to_vec(),
            warn: None,
            crit: None,
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

fn ratio(baselines: &[BaselineKind]) -> SignalDefinitionData {
    definition(SignalKind::Ratio, baselines)
}

/// `definition` with `number` as its `evaluation.fixed`.
fn with_fixed(mut definition: SignalDefinitionData, number: &str) -> SignalDefinitionData {
    definition.evaluation.fixed = Some(Decimal(number.into()));
    definition
}

fn sample_at(start: &str, end: &str) -> SignalSampleData {
    SignalSampleData {
        sample_id: SampleId("smp_1".into()),
        signal: SignalId("platform.namespace.request_failure_ratio".into()),
        series: SeriesKey("platform.namespace.request_failure_ratio".into()),
        dims: BTreeMap::new(),
        ts: Timestamp(end.into()),
        window: TimeWindow {
            start: Timestamp(start.into()),
            end: Timestamp(end.into()),
        },
        reading: ok("0.029"),
        unit: Unit("1".into()),
        denominator: Some(Denominator {
            n: 556,
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

fn sample() -> SignalSampleData {
    sample_at(START, TS)
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn unread(reason: &str) -> Reading {
    Reading::Unknown(UnreadReading {
        reason: reason.into(),
    })
}

fn at(offset: Offset, value: &str, n: i64) -> OffsetReading {
    OffsetReading {
        offset,
        reading: ok(value),
        n: Some(n),
    }
}

/// A series old enough for every offset baseline of M1.
const OLD: Option<i64> = Some(365 * DAY);

fn fold(
    definition: &SignalDefinitionData,
    readings: &[OffsetReading],
    series_age_s: Option<i64>,
) -> Baseline {
    baseline::fold(definition, &sample(), readings, series_age_s)
        .expect("an M1 baseline kind folds")
}

fn value(baseline: &Baseline) -> &str {
    match &baseline.reading {
        Reading::Ok(reading) => &reading.value.0,
        other => panic!("expected an ok baseline reading, got {other:?}"),
    }
}

fn unknown_reason(baseline: &Baseline) -> &str {
    match &baseline.reading {
        Reading::Unknown(reading) => &reading.reason,
        other => panic!("expected an unknown baseline reading, never a value, got {other:?}"),
    }
}

fn offsets(baseline: &Baseline) -> Vec<&str> {
    baseline.offsets.iter().map(String::as_str).collect()
}

fn instant(text: &str) -> OffsetDateTime {
    OffsetDateTime::parse(text, &Rfc3339).expect("an RFC 3339 timestamp")
}

#[test]
fn baseline_older_than_series_is_unknown() {
    let week = ratio(&[BaselineKind::Offset7d]);
    let readings = [at(Offset::SevenDays, "0.013", 400)];

    // The design's L6 case: "week over week" on a series three days old.
    let three_days = fold(&week, &readings, Some(3 * DAY));
    assert_eq!(unknown_reason(&three_days), REASON_PREDATES_SERIES);
    assert_eq!(three_days.kind.0, "offset_7d");
    assert_eq!(offsets(&three_days), ["7d"]);

    // Exactly as old as the offset: the shifted window lies wholly before the series began.
    let exactly = fold(&week, &readings, Some(7 * DAY));
    assert_eq!(unknown_reason(&exactly), REASON_PREDATES_SERIES);

    // The shifted window starts one second before the series began.
    let one_short = fold(&week, &readings, Some(7 * DAY + 6 * HOUR - 1));
    assert_eq!(unknown_reason(&one_short), REASON_PREDATES_SERIES);

    // The shifted window starts exactly when the series began: it is inside the series.
    let inside = fold(&week, &readings, Some(7 * DAY + 6 * HOUR));
    assert_eq!(value(&inside), "0.013");

    // One offset of a pooled baseline predating the series makes the whole baseline unknown.
    let pooled = ratio(&[
        BaselineKind::Pooled,
        BaselineKind::Offset1d,
        BaselineKind::Offset7d,
    ]);
    let both = [
        at(Offset::OneDay, "0.02", 100),
        at(Offset::SevenDays, "0.01", 300),
    ];
    let young = fold(&pooled, &both, Some(2 * DAY));
    assert_eq!(unknown_reason(&young), REASON_PREDATES_SERIES);
    assert_eq!(young.kind.0, "pooled");
    assert_eq!(offsets(&young), ["1d", "7d"]);

    // The series age is checked before the readings: a reading of a window older than the series
    // is not trusted even when the source answered one.
    let day = ratio(&[BaselineKind::Offset1d]);
    let answered = fold(&day, &[at(Offset::OneDay, "0.5", 10)], Some(HOUR));
    assert_eq!(unknown_reason(&answered), REASON_PREDATES_SERIES);
}

#[test]
fn unreadable_offset_is_unknown_never_zero() {
    let day = ratio(&[BaselineKind::Offset1d]);

    let unread_day = fold(
        &day,
        &[OffsetReading {
            offset: Offset::OneDay,
            reading: unread("source timed out"),
            n: None,
        }],
        OLD,
    );
    let reason = unknown_reason(&unread_day);
    assert!(reason.contains("1d"), "reason names the offset: {reason}");
    assert!(
        reason.contains("source timed out"),
        "reason carries why: {reason}"
    );

    let missing_day = fold(&day, &[], OLD);
    let reason = unknown_reason(&missing_day);
    assert!(reason.contains("1d"), "reason names the offset: {reason}");

    let pooled = ratio(&[BaselineKind::Offset1d, BaselineKind::Offset7d]);
    let missing_week = fold(&pooled, &[at(Offset::OneDay, "0.02", 100)], OLD);
    let reason = unknown_reason(&missing_week);
    assert!(reason.contains("7d"), "reason names the offset: {reason}");
    assert!(!reason.contains("1d"), "the 1d offset was read: {reason}");

    for combiner in [BaselineKind::HigherOf, BaselineKind::LowerOf] {
        let definition = ratio(&[combiner]);
        let folded = fold(
            &definition,
            &[
                OffsetReading {
                    offset: Offset::OneDay,
                    reading: unread("query failed"),
                    n: None,
                },
                at(Offset::SevenDays, "120", 1),
            ],
            OLD,
        );
        let reason = unknown_reason(&folded);
        assert!(reason.contains("1d"), "reason names the offset: {reason}");
    }

    // A pooled constituent without a population cannot be pooled.
    let no_n = fold(
        &pooled,
        &[
            at(Offset::OneDay, "0.02", 100),
            OffsetReading {
                offset: Offset::SevenDays,
                reading: ok("0.01"),
                n: None,
            },
        ],
        OLD,
    );
    let reason = unknown_reason(&no_n);
    assert!(reason.contains("7d"), "reason names the offset: {reason}");

    // A partial read is a bound, not a baseline.
    let partial = fold(
        &day,
        &[OffsetReading {
            offset: Offset::OneDay,
            reading: Reading::Partial(PartialReading {
                value: Decimal("40".into()),
                bound: thalamus_model::observation::Bound::Lower,
            }),
            n: Some(40),
        }],
        OLD,
    );
    let reason = unknown_reason(&partial);
    assert!(reason.contains("1d"), "reason names the offset: {reason}");

    // A value that is not a decimal is not read as 0.
    let garbled = fold(&day, &[at(Offset::OneDay, "2.9%", 10)], OLD);
    let reason = unknown_reason(&garbled);
    assert!(reason.contains("1d"), "reason names the offset: {reason}");

    // No population at all in a pooled baseline is not a pooled 0.
    let empty = fold(
        &pooled,
        &[at(Offset::OneDay, "0", 0), at(Offset::SevenDays, "0", 0)],
        OLD,
    );
    assert!(matches!(empty.reading, Reading::Unknown(_)));
}

#[test]
fn unknown_series_age_makes_offsets_unknown() {
    let both = [
        at(Offset::OneDay, "0.02", 100),
        at(Offset::SevenDays, "0.01", 300),
    ];
    for kinds in [
        &[BaselineKind::Offset1d][..],
        &[BaselineKind::Offset7d],
        &[BaselineKind::Pooled],
        &[BaselineKind::HigherOf],
        &[BaselineKind::LowerOf],
    ] {
        let folded = fold(&ratio(kinds), &both, None);
        assert_eq!(
            unknown_reason(&folded),
            REASON_SERIES_AGE_UNKNOWN,
            "{kinds:?}"
        );
    }

    // A fixed baseline has no offset, so the series age does not matter to it.
    let fixed = baseline::fold(
        &with_fixed(ratio(&[BaselineKind::Fixed]), "0"),
        &sample(),
        &[],
        None,
    )
    .expect("fixed folds");
    assert_eq!(value(&fixed), "0");
}

#[test]
fn offset_7d_window_keeps_weekday() {
    let sample = sample();

    let week = baseline::plan(&ratio(&[BaselineKind::Offset7d]), &sample).expect("plans");
    assert_eq!(
        week,
        [WindowRequest {
            offset: Offset::SevenDays,
            window: TimeWindow {
                start: Timestamp("2026-10-07T16:57:03Z".into()),
                end: Timestamp("2026-10-07T22:57:03Z".into()),
            },
        }]
    );
    let shifted = instant(&week[0].window.end.0);
    let original = instant(TS);
    assert_eq!(shifted.weekday(), original.weekday());
    assert_eq!((original - shifted).whole_seconds(), 7 * DAY);

    let day = baseline::plan(&ratio(&[BaselineKind::Offset1d]), &sample).expect("plans");
    assert_eq!(
        day,
        [WindowRequest {
            offset: Offset::OneDay,
            window: TimeWindow {
                start: Timestamp("2026-10-13T16:57:03Z".into()),
                end: Timestamp("2026-10-13T22:57:03Z".into()),
            },
        }]
    );

    // A window across a month boundary, in a non-UTC offset, keeps its weekday and its offset.
    let early = sample_at("2026-03-02T01:00:00+02:00", "2026-03-02T07:00:00+02:00");
    let back = baseline::plan(&ratio(&[BaselineKind::Offset7d]), &early).expect("plans");
    let start = instant(&back[0].window.start.0);
    let end = instant(&back[0].window.end.0);
    assert_eq!(
        start.weekday(),
        instant("2026-03-02T01:00:00+02:00").weekday()
    );
    assert_eq!(end.offset(), instant("2026-03-02T07:00:00+02:00").offset());
    assert_eq!(
        (instant("2026-03-02T07:00:00+02:00") - end).whole_seconds(),
        7 * DAY
    );
    assert_eq!((end - start).whole_seconds(), 6 * HOUR);

    // A combined baseline plans each of its offsets, in order; a fixed one plans nothing.
    let pooled = baseline::plan(&ratio(&[BaselineKind::Pooled]), &sample).expect("plans");
    let planned: Vec<Offset> = pooled.iter().map(|request| request.offset).collect();
    assert_eq!(planned, [Offset::OneDay, Offset::SevenDays]);
    assert!(
        baseline::plan(&ratio(&[BaselineKind::Fixed]), &sample)
            .expect("plans")
            .is_empty()
    );
}

#[test]
fn pooled_sums_numerators_over_denominators() {
    // (0.02 x 100 + 0.01 x 300) / (100 + 300) = 5 / 400.
    let pooled = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "0.02", 100),
            at(Offset::SevenDays, "0.01", 300),
        ],
        OLD,
    );
    assert_eq!(value(&pooled), "0.0125");
    assert_eq!(pooled.kind.0, "pooled");
    assert_eq!(offsets(&pooled), ["1d", "7d"]);

    // Not the mean of the two values, which would be 0.015.
    assert_ne!(value(&pooled), "0.015");

    // Design § 11 D4: offsets listed without a combiner are recorded as pooled over [1d, 7d].
    let listed = fold(
        &ratio(&[BaselineKind::Offset1d, BaselineKind::Offset7d]),
        &[
            at(Offset::OneDay, "0.02", 100),
            at(Offset::SevenDays, "0.01", 300),
        ],
        OLD,
    );
    assert_eq!(listed, pooled);

    // A quotient that does not terminate is rounded at the 18th fractional digit.
    let thirds = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "0.01", 100),
            at(Offset::SevenDays, "0.02", 200),
        ],
        OLD,
    );
    assert_eq!(value(&thirds), "0.016666666666666667");

    // Whole counts pool to a whole number without a trailing fraction.
    let whole = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[at(Offset::OneDay, "3", 2), at(Offset::SevenDays, "3", 5)],
        OLD,
    );
    assert_eq!(value(&whole), "3");

    // Signs pool exactly, and a pooled 0 is written `0`, never `-0`.
    let negative = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "-0.3", 1),
            at(Offset::SevenDays, "-0.1", 1),
        ],
        OLD,
    );
    assert_eq!(value(&negative), "-0.2");
    let cancelled = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "-0.5", 1),
            at(Offset::SevenDays, "0.5", 1),
        ],
        OLD,
    );
    assert_eq!(value(&cancelled), "0");
}

#[test]
fn higher_of_and_lower_of_pick_the_bound() {
    let readings = [
        at(Offset::OneDay, "120", 1),
        at(Offset::SevenDays, "95.5", 1),
    ];

    let higher = fold(&ratio(&[BaselineKind::HigherOf]), &readings, OLD);
    assert_eq!(value(&higher), "120");
    assert_eq!(higher.kind.0, "higher_of");
    assert_eq!(offsets(&higher), ["1d", "7d"]);

    let lower = fold(&ratio(&[BaselineKind::LowerOf]), &readings, OLD);
    assert_eq!(value(&lower), "95.5");
    assert_eq!(lower.kind.0, "lower_of");
    assert_eq!(offsets(&lower), ["1d", "7d"]);

    // Compared as numbers, not as text: "0.5" is higher than "0.45" and "10" than "9".
    let fractions = [
        at(Offset::OneDay, "0.45", 1),
        at(Offset::SevenDays, "0.5", 1),
    ];
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::HigherOf]), &fractions, OLD)),
        "0.5"
    );
    let counts = [at(Offset::OneDay, "9", 1), at(Offset::SevenDays, "10", 1)];
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::HigherOf]), &counts, OLD)),
        "10"
    );
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::LowerOf]), &counts, OLD)),
        "9"
    );
    let below_zero = [
        at(Offset::OneDay, "-1.5", 1),
        at(Offset::SevenDays, "-0.9", 1),
    ];
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::HigherOf]), &below_zero, OLD)),
        "-0.9"
    );
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::LowerOf]), &below_zero, OLD)),
        "-1.5"
    );
}

#[test]
fn fixed_is_the_definition_number() {
    let fixed = with_fixed(ratio(&[BaselineKind::Fixed]), "0.05");

    let folded = baseline::fold(&fixed, &sample(), &[], OLD).expect("fixed folds");
    assert_eq!(value(&folded), "0.05");
    assert_eq!(folded.kind.0, "fixed");
    assert!(folded.offsets.is_empty());

    // Earlier readings handed in by mistake do not move a fixed baseline.
    let ignored =
        baseline::fold(&fixed, &sample(), &[at(Offset::OneDay, "7", 1)], OLD).expect("fixed folds");
    assert_eq!(value(&ignored), "0.05");

    // A fixed baseline that declares no number is unknown, never 0.
    let absent =
        baseline::fold(&ratio(&[BaselineKind::Fixed]), &sample(), &[], OLD).expect("fixed folds");
    assert!(matches!(absent.reading, Reading::Unknown(_)));
}

/// Design § 11 D25: the number of a `fixed` baseline is `evaluation.fixed`, read from the
/// definition, its one copy of every threshold. Without it the baseline is unknown, never 0.
#[test]
fn fixed_baseline_reads_the_definition_number() {
    let mut fixed = ratio(&[BaselineKind::Fixed]);
    fixed.evaluation.fixed = Some(Decimal("0.05".into()));
    let folded = baseline::fold(&fixed, &sample(), &[], OLD).expect("fixed folds");
    assert_eq!(value(&folded), "0.05");
    assert_eq!(folded.kind.0, "fixed");

    fixed.evaluation.fixed = None;
    let absent = baseline::fold(&fixed, &sample(), &[], OLD).expect("fixed folds");
    assert!(matches!(absent.reading, Reading::Unknown(_)), "{absent:?}");
}

#[test]
fn instant_refuses_non_fixed_baseline() {
    for kind in [
        BaselineKind::Offset1d,
        BaselineKind::Offset7d,
        BaselineKind::Pooled,
        BaselineKind::HigherOf,
        BaselineKind::LowerOf,
        BaselineKind::SameSlotMedian4w,
        BaselineKind::RollingMedian,
    ] {
        let definition = with_fixed(definition(SignalKind::Instant, &[kind]), "3");
        assert_eq!(
            baseline::plan(&definition, &sample()),
            Err(Refusal::InstantNeedsFixed { kind }),
        );
        assert_eq!(
            baseline::fold(&definition, &sample(), &[], OLD),
            Err(Refusal::InstantNeedsFixed { kind }),
        );
    }

    // Fixed next to another kind is still refused for an instant signal.
    let mixed = definition(
        SignalKind::Instant,
        &[BaselineKind::Fixed, BaselineKind::Offset1d],
    );
    assert_eq!(
        baseline::plan(&mixed, &sample()),
        Err(Refusal::InstantNeedsFixed {
            kind: BaselineKind::Offset1d
        }),
    );

    let fixed = with_fixed(definition(SignalKind::Instant, &[BaselineKind::Fixed]), "3");
    assert!(baseline::plan(&fixed, &sample()).expect("plans").is_empty());
    let folded = baseline::fold(&fixed, &sample(), &[], None)
        .expect("an instant signal folds against fixed");
    assert_eq!(value(&folded), "3");
}

#[test]
fn store_baselines_answer_not_in_m1() {
    for kind in [BaselineKind::SameSlotMedian4w, BaselineKind::RollingMedian] {
        let definition = ratio(&[kind]);
        let refusal = baseline::plan(&definition, &sample()).expect_err("not in M1");
        assert_eq!(refusal, Refusal::NotInM1 { kind });
        assert!(refusal.to_string().contains("not in M1"), "{refusal}");
        assert_eq!(
            baseline::fold(&definition, &sample(), &[], OLD),
            Err(Refusal::NotInM1 { kind }),
        );
    }
}

#[test]
fn a_definition_names_one_baseline() {
    assert_eq!(
        baseline::plan(&ratio(&[]), &sample()),
        Err(Refusal::NoBaseline)
    );
    let two_combiners = ratio(&[BaselineKind::Pooled, BaselineKind::HigherOf]);
    assert!(matches!(
        baseline::plan(&two_combiners, &sample()),
        Err(Refusal::Ambiguous { .. })
    ));
    let fixed_and_offset = ratio(&[BaselineKind::Fixed, BaselineKind::Offset1d]);
    assert!(matches!(
        baseline::fold(&fixed_and_offset, &sample(), &[], OLD),
        Err(Refusal::Ambiguous { .. })
    ));
}

/// Design § 3.1: `pooled` is the baseline of ratios. A signal of another kind is refused it,
/// whether it names `pooled` or lists two offsets without a combiner (which reads as `pooled`).
#[test]
fn pooled_is_refused_for_a_signal_that_is_not_a_ratio() {
    for kind in [
        SignalKind::Gauge,
        SignalKind::CounterIncrease,
        SignalKind::CountOfCases,
    ] {
        for baselines in [
            &[BaselineKind::Offset1d, BaselineKind::Offset7d][..],
            &[BaselineKind::Pooled],
            &[
                BaselineKind::Pooled,
                BaselineKind::Offset1d,
                BaselineKind::Offset7d,
            ],
        ] {
            let definition = definition(kind, baselines);
            assert_eq!(
                baseline::plan(&definition, &sample()),
                Err(Refusal::PooledNeedsRatio { kind }),
                "{kind:?} {baselines:?}"
            );
            let refusal = baseline::fold(
                &definition,
                &sample(),
                &[at(Offset::OneDay, "3", 1), at(Offset::SevenDays, "4", 1)],
                OLD,
            )
            .expect_err("pooled is for ratios");
            assert_eq!(refusal, Refusal::PooledNeedsRatio { kind });
            assert!(refusal.to_string().contains("ratio"), "{refusal}");
        }

        // The combiners § 3.1 names for counts still fold.
        for combiner in [BaselineKind::HigherOf, BaselineKind::LowerOf] {
            let folded = baseline::fold(
                &definition(
                    kind,
                    &[combiner, BaselineKind::Offset1d, BaselineKind::Offset7d],
                ),
                &sample(),
                &[at(Offset::OneDay, "3", 1), at(Offset::SevenDays, "4", 1)],
                OLD,
            )
            .expect("higher_of and lower_of fold for any kind");
            assert!(matches!(folded.reading, Reading::Ok(_)));
        }
    }
}

/// The wire grammar of a decimal (`generated/thalamus-model/src/json.rs`, `decimal_at`) has no
/// length limit, so no baseline kind may refuse or overflow on a wire-valid decimal.
#[test]
fn wire_valid_decimals_of_any_length_fold() {
    let long = "123456789012345678901234567890.123456789012345678901234567890";
    let offset = fold(
        &ratio(&[BaselineKind::Offset7d]),
        &[at(Offset::SevenDays, long, 1)],
        OLD,
    );
    assert_eq!(value(&offset), long);

    let tenth = "100000000000000000000000000000000000000000000000000.1";
    let hundredth = "100000000000000000000000000000000000000000000000000.01";
    let both = [
        at(Offset::OneDay, hundredth, 1),
        at(Offset::SevenDays, tenth, 1),
    ];
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::HigherOf]), &both, OLD)),
        tenth
    );
    assert_eq!(
        value(&fold(&ratio(&[BaselineKind::LowerOf]), &both, OLD)),
        hundredth
    );

    // Products past 128 bits and a population past 64 bits pool exactly.
    let nines = "99999999999999999999999999999999999999999";
    let pooled = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, nines, i64::MAX),
            at(Offset::SevenDays, nines, i64::MAX),
        ],
        OLD,
    );
    assert_eq!(value(&pooled), nines);

    // Opposite signs of one long magnitude cancel to 0.
    let cancelled = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(
                Offset::OneDay,
                "-1000000000000000000000000000000000000000.5",
                3,
            ),
            at(
                Offset::SevenDays,
                "1000000000000000000000000000000000000000.5",
                3,
            ),
        ],
        OLD,
    );
    assert_eq!(value(&cancelled), "0");

    let fixed = baseline::fold(
        &with_fixed(ratio(&[BaselineKind::Fixed]), long),
        &sample(),
        &[],
        OLD,
    )
    .expect("fixed folds");
    assert_eq!(value(&fixed), long);
}
