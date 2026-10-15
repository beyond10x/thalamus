//! Adversary pass 2 on `story:m1-baselines`: the arbitrary-length decimal arithmetic of the
//! correction, checked against an independent `i128` oracle over a fixed-seed sample of signs,
//! zeros, scales and populations, plus populations at `i64::MAX`.

use std::cmp::Ordering;
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
const FRACTION: u32 = 18;

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

fn sample() -> SignalSampleData {
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
            value: Decimal("0.029".into()),
        }),
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

fn at(offset: Offset, value: &str, n: i64) -> OffsetReading {
    OffsetReading {
        offset,
        reading: Reading::Ok(CompleteReading {
            value: Decimal(value.into()),
        }),
        n: Some(n),
    }
}

fn fold(kind: BaselineKind, readings: &[OffsetReading]) -> Baseline {
    baseline::fold(&ratio(&[kind]), &sample(), readings, OLD, None).expect("an M1 baseline folds")
}

fn value(baseline: &Baseline) -> String {
    match &baseline.reading {
        Reading::Ok(reading) => reading.value.0.clone(),
        other => panic!("expected an ok baseline reading, got {other:?}"),
    }
}

/// A fixed-seed linear congruential generator, so every run draws the same cases.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// One drawn constituent: `mantissa / 10^scale`, its wire text, and its population.
struct Drawn {
    mantissa: i128,
    scale: u32,
    text: String,
    n: i64,
}

/// The wire text of `mantissa / 10^scale`, keeping every fractional digit (so `0.000` and `1.50`
/// occur), and `0` never signed.
fn text(mantissa: i128, scale: u32) -> String {
    let sign = if mantissa < 0 { "-" } else { "" };
    let magnitude = mantissa.unsigned_abs();
    if scale == 0 {
        return format!("{sign}{magnitude}");
    }
    let unit = 10_u128.pow(scale);
    format!(
        "{sign}{}.{:0width$}",
        magnitude / unit,
        magnitude % unit,
        width = scale as usize
    )
}

fn draw(rng: &mut Lcg) -> Drawn {
    let scale = u32::try_from(rng.below(7)).expect("small");
    let magnitude = match rng.below(4) {
        0 => 0,
        1 => i128::from(rng.below(10)),
        _ => i128::from(rng.below(1_000_000)),
    };
    let mantissa = if rng.below(3) == 0 {
        -magnitude
    } else {
        magnitude
    };
    let n = match rng.below(5) {
        0 => 0,
        1 => 1,
        _ => i64::try_from(rng.below(1_000_000)).expect("small"),
    };
    Drawn {
        mantissa,
        scale,
        text: text(mantissa, scale),
        n,
    }
}

/// The pooled quotient by the oracle: `None` when the population is 0, else the decimal rounded
/// half away from zero at 18 fractional digits, trailing zeros dropped, never `-0`.
fn oracle_pooled(values: &[&Drawn]) -> Option<String> {
    let scale = values.iter().map(|value| value.scale).max().unwrap_or(0);
    let numerator: i128 = values
        .iter()
        .map(|value| value.mantissa * 10_i128.pow(scale - value.scale) * i128::from(value.n))
        .sum();
    let denominator: i128 = values.iter().map(|value| i128::from(value.n)).sum();
    if denominator == 0 {
        return None;
    }
    let scaled = numerator.unsigned_abs() * 10_u128.pow(FRACTION - scale);
    let divisor = denominator.unsigned_abs();
    let mut quotient = scaled / divisor;
    if (scaled % divisor) * 2 >= divisor {
        quotient += 1;
    }
    let unit = 10_u128.pow(FRACTION);
    let whole = quotient / unit;
    let part = format!("{:018}", quotient % unit);
    let part = part.trim_end_matches('0');
    let sign = if numerator < 0 && quotient != 0 {
        "-"
    } else {
        ""
    };
    Some(if part.is_empty() {
        format!("{sign}{whole}")
    } else {
        format!("{sign}{whole}.{part}")
    })
}

/// The constituent `higher_of` or `lower_of` picks by the oracle: the first of equal ones.
fn oracle_pick<'a>(values: &[&'a Drawn], wanted: Ordering) -> &'a Drawn {
    let scale = values.iter().map(|value| value.scale).max().unwrap_or(0);
    let aligned = |value: &Drawn| value.mantissa * 10_i128.pow(scale - value.scale);
    let mut best = values[0];
    for candidate in &values[1..] {
        if aligned(candidate).cmp(&aligned(best)) == wanted {
            best = candidate;
        }
    }
    best
}

#[test]
fn adv_pooled_and_bounds_agree_with_an_exact_oracle() {
    let mut rng = Lcg(0x5EED_2026_1015);
    for case in 0..5_000 {
        let (day, week) = (draw(&mut rng), draw(&mut rng));
        let readings = [
            at(Offset::OneDay, &day.text, day.n),
            at(Offset::SevenDays, &week.text, week.n),
        ];
        let both = [&day, &week];
        let context = format!(
            "case {case}: 1d {} x {}, 7d {} x {}",
            day.text, day.n, week.text, week.n
        );

        let pooled = fold(BaselineKind::Pooled, &readings);
        match oracle_pooled(&both) {
            Some(expected) => assert_eq!(value(&pooled), expected, "pooled, {context}"),
            None => assert!(
                matches!(pooled.reading, Reading::Unknown(_)),
                "pooled over no population, {context}: {:?}",
                pooled.reading
            ),
        }

        let higher = fold(BaselineKind::HigherOf, &readings);
        assert_eq!(
            value(&higher),
            oracle_pick(&both, Ordering::Greater).text,
            "higher_of, {context}"
        );
        let lower = fold(BaselineKind::LowerOf, &readings);
        assert_eq!(
            value(&lower),
            oracle_pick(&both, Ordering::Less).text,
            "lower_of, {context}"
        );
    }
}

#[test]
fn adv_pooled_at_the_largest_population() {
    let max = i64::MAX;
    let half = fold(
        BaselineKind::Pooled,
        &[
            at(Offset::OneDay, "1", max),
            at(Offset::SevenDays, "0", max),
        ],
    );
    assert_eq!(value(&half), "0.5");

    let mean = fold(
        BaselineKind::Pooled,
        &[
            at(Offset::OneDay, "-0.3", max),
            at(Offset::SevenDays, "-0.1", max),
        ],
    );
    assert_eq!(value(&mean), "-0.2");

    // (0.029 x 556 + 0.013 x (2^63 - 1)) / (556 + 2^63 - 1), rounded at the 18th digit.
    let skewed = fold(
        BaselineKind::Pooled,
        &[
            at(Offset::OneDay, "0.029", 556),
            at(Offset::SevenDays, "0.013", max),
        ],
    );
    assert_eq!(value(&skewed), "0.013000000000000001");
}

#[test]
fn adv_bounds_compare_long_decimals_by_value() {
    let long = "0.1000000000000000000000000000000000000000001";
    let readings = [at(Offset::OneDay, "0.1", 1), at(Offset::SevenDays, long, 1)];
    assert_eq!(value(&fold(BaselineKind::HigherOf, &readings)), long);
    assert_eq!(value(&fold(BaselineKind::LowerOf, &readings)), "0.1");

    let negative_long = "-0.1000000000000000000000000000000000000000001";
    let readings = [
        at(Offset::OneDay, negative_long, 1),
        at(Offset::SevenDays, "-0.1", 1),
    ];
    assert_eq!(value(&fold(BaselineKind::HigherOf, &readings)), "-0.1");
    assert_eq!(
        value(&fold(BaselineKind::LowerOf, &readings)),
        negative_long
    );

    // `-0` and `0.00` are both zero: neither is higher, so the first is kept.
    let zeros = [
        at(Offset::OneDay, "-0", 1),
        at(Offset::SevenDays, "0.00", 1),
    ];
    assert_eq!(value(&fold(BaselineKind::HigherOf, &zeros)), "-0");
    assert_eq!(value(&fold(BaselineKind::LowerOf, &zeros)), "-0");
}
