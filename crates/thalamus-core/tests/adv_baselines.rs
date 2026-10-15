//! Adversary cases for `story:m1-baselines`: wire-valid decimals the baseline fold must not turn
//! into `unknown`, and the rounding of a pooled quotient at an exact half.

use std::collections::BTreeMap;

use thalamus_core::baseline::{self, Offset, OffsetReading};
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId,
    Family, History, Links, Owner, RuleId, SignalDefinitionData, SignalId, SignalKind, Source,
    Unit,
};
use thalamus_model::evaluation::Baseline;
use thalamus_model::observation::{
    CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};

const OLD: Option<i64> = Some(365 * 86_400);

fn ratio(baselines: &[BaselineKind]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.namespace.request_failure_ratio".into()),
        title: "Share of requests the platform failed".into(),
        family: Family("platform".into()),
        kind: SignalKind::Ratio,
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

fn sample() -> SignalSampleData {
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

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn at(offset: Offset, value: &str, n: i64) -> OffsetReading {
    OffsetReading {
        offset,
        reading: ok(value),
        n: Some(n),
    }
}

fn fold(definition: &SignalDefinitionData, readings: &[OffsetReading]) -> Baseline {
    baseline::fold(definition, &sample(), readings, OLD).expect("an M1 baseline kind folds")
}

fn value(baseline: &Baseline) -> &str {
    match &baseline.reading {
        Reading::Ok(reading) => &reading.value.0,
        other => panic!("expected an ok baseline reading, got {other:?}"),
    }
}

/// The generated wire decoder (`generated/thalamus-model/src/json.rs`, `decimal_at`) admits a
/// decimal of any length. An offset baseline is "the reading of that window" (story Work), so a
/// wire-valid reading with 39 digits is that baseline, not `unknown` with reason "is not a
/// decimal".
#[test]
fn adv_wire_valid_long_decimal_is_an_offset_baseline() {
    let long = "0.12345678901234567890123456789012345678";
    let folded = fold(
        &ratio(&[BaselineKind::Offset1d]),
        &[at(Offset::OneDay, long, 10)],
    );
    assert_eq!(value(&folded), long);
}

/// A pooled baseline over two wire-valid values and a population of 100 is computable; it is not
/// `unknown` because aligning the scales overflowed an `i128`.
#[test]
fn adv_pooled_over_wire_valid_values_does_not_overflow() {
    // (0.0000000000000000000000000000000000001 x 1 + 5 x 100) / 101 = 4.9504950495…
    let folded = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "0.0000000000000000000000000000000000001", 1),
            at(Offset::SevenDays, "5", 100),
        ],
    );
    let pooled = value(&folded);
    assert!(
        pooled.starts_with("4.950495049504950495"),
        "pooled value {pooled}"
    );
}

/// The unit's own contract (`POOLED_SCALE`): the 18th fractional digit is rounded half away from
/// zero. An exact half at the 19th digit must round up in magnitude, for both signs. Catches the
/// mutant `>= 5` -> `> 5` in `divide`, which no case of `tests/baselines.rs` exercises.
#[test]
fn adv_pooled_rounds_an_exact_half_away_from_zero() {
    let positive = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "0.000000000000000001", 1),
            at(Offset::SevenDays, "0", 1),
        ],
    );
    assert_eq!(value(&positive), "0.000000000000000001");

    let negative = fold(
        &ratio(&[BaselineKind::Pooled]),
        &[
            at(Offset::OneDay, "-0.000000000000000001", 1),
            at(Offset::SevenDays, "0", 1),
        ],
    );
    assert_eq!(value(&negative), "-0.000000000000000001");
}
