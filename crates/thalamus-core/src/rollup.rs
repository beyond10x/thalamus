//! The daily rollup (design § 4): one row per series per UTC day, built from that series' samples
//! of the day and their evaluations. Pure: the samples, the evaluations and the day are passed in.
//!
//! - `n` counts the day's samples, `unknown` and `partial` those of that quality; a sample is one
//!   of `ok`, `partial` and `unknown`, so `unknown + partial <= n` (design § 11 D27).
//! - `min`, `p50`, `max` and `last` are taken over the `ok` readings only. A `partial` reading is
//!   counted and never valued: its value is a bound (design § 2.2). With no `ok` reading the four
//!   are absent, never 0. `p50` is the exact median (the mean of the two middle values of an even
//!   count); `last` is the reading of the latest `ts`, of two at one instant the one given last.
//! - Of each sample's evaluations only the one at the highest `definition_rev` present is counted
//!   (design § 11 D14). From those: the worst verdict by the ladder's rank
//!   ([`crate::ladder::worst`]), one count per ladder verdict, and `min_score` / `max_score` over
//!   the scored verdicts ([`crate::ladder::scored`]) that carry a score; absent when there is none.
//!
//! A day is the UTC calendar day: from its midnight, included, to the next, excluded, whatever
//! offset a sample's `ts` is written with. A sample whose `ts` names no instant is in no day.
//!
//! A rollup's id is spelled `<series key>@<YYYY-MM-DD>` ([`rollup_id`]): ESS cannot derive a
//! field from two others (the `UNMAPPED:` marker in `spec/domains/rollup.yaml`).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{CompleteReading, Reading, SeriesKey, SignalSampleData};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::rollup::{RecordRollup, RollupId, VerdictCounts};

use crate::baseline::{decimal_order, median};
use crate::behaviour::observation::Instant;
use crate::ladder;

/// The seconds of a UTC day.
pub const DAY_SECONDS: i64 = 86_400;

/// The UTC midnight (Unix seconds) that starts the day holding the instant `ts` names; `None`
/// when `ts` names none.
#[must_use]
pub fn day_start(ts: &str) -> Option<i64> {
    Instant::parse(ts).map(|at| at.seconds().div_euclid(DAY_SECONDS) * DAY_SECONDS)
}

/// The UTC midnight (Unix seconds) of the day `text` names as `YYYY-MM-DD`; `None` for anything
/// else.
#[must_use]
pub fn parse_day(text: &str) -> Option<i64> {
    let shaped = text.len() == 10
        && text.bytes().enumerate().all(|(index, byte)| match index {
            4 | 7 => byte == b'-',
            _ => byte.is_ascii_digit(),
        });
    if !shaped {
        return None;
    }
    Instant::parse(&format!("{text}T00:00:00Z")).map(Instant::seconds)
}

/// The `YYYY-MM-DD` of the UTC day starting at `day` (Unix seconds).
#[must_use]
pub fn day_text(day: i64) -> String {
    let (year, month, date) = civil(day.div_euclid(DAY_SECONDS));
    format!("{year:04}-{month:02}-{date:02}")
}

/// The RFC 3339 instant of the UTC midnight `day` (Unix seconds): a rollup's `day`.
#[must_use]
pub fn day_timestamp(day: i64) -> Timestamp {
    Timestamp(format!("{}T00:00:00Z", day_text(day)))
}

/// The id of the rollup of `series` on the UTC day starting at `day`: `<series key>@<YYYY-MM-DD>`.
#[must_use]
pub fn rollup_id(series: &SeriesKey, day: i64) -> RollupId {
    RollupId(format!("{}@{}", series.0, day_text(day)))
}

/// The `RecordRollup` input of `series` (of `signal`) on the UTC day starting at `day` (Unix
/// seconds). `samples` and `evaluations` may hold other series and other days: only this series'
/// samples of this day, and evaluations of those samples, are counted.
#[must_use]
pub fn build(
    signal: &SignalId,
    series: &SeriesKey,
    day: i64,
    samples: &[SignalSampleData],
    evaluations: &[SignalEvaluationData],
) -> RecordRollup {
    let mut kept: Vec<(Instant, &SignalSampleData)> = samples
        .iter()
        .filter(|sample| sample.series == *series)
        .filter_map(|sample| Some((Instant::parse(&sample.ts.0)?, sample)))
        .filter(|(at, _)| at.seconds().div_euclid(DAY_SECONDS) * DAY_SECONDS == day)
        .collect();
    // Stable: of two samples at one instant, the one given last stays last.
    kept.sort_by_key(|(at, _)| *at);

    let mut unknown = 0;
    let mut partial = 0;
    let mut values: Vec<&Decimal> = Vec::new();
    for (_, sample) in &kept {
        match &sample.reading {
            Reading::Ok(CompleteReading { value }) => {
                if decimal_order(value, value).is_some() {
                    values.push(value);
                }
            }
            Reading::Partial(_) => partial += 1,
            Reading::Unknown(_) => unknown += 1,
        }
    }
    let last = values.last().map(|value| (*value).clone());
    let min = extreme(values.iter().copied(), Ordering::Less);
    let max = extreme(values.iter().copied(), Ordering::Greater);
    let p50 = median(&values).and_then(|reading| match reading {
        Reading::Ok(CompleteReading { value }) => Some(value),
        Reading::Partial(_) | Reading::Unknown(_) => None,
    });

    let counted = highest_rev(&kept, evaluations);
    let mut verdict_counts = VerdictCounts {
        ok: 0,
        na: 0,
        warn: 0,
        crit: 0,
        unknown: 0,
    };
    for evaluation in &counted {
        let count = match evaluation.verdict {
            Verdict::OK => &mut verdict_counts.ok,
            Verdict::NA => &mut verdict_counts.na,
            Verdict::WARN => &mut verdict_counts.warn,
            Verdict::CRIT => &mut verdict_counts.crit,
            Verdict::UNKNOWN => &mut verdict_counts.unknown,
        };
        *count += 1;
    }
    let scores: Vec<&Decimal> = counted
        .iter()
        .filter(|evaluation| ladder::scored(evaluation.verdict))
        .filter_map(|evaluation| evaluation.score.as_ref())
        .filter(|score| decimal_order(score, score).is_some())
        .collect();

    RecordRollup {
        rollup_id: rollup_id(series, day),
        signal: signal.clone(),
        series: series.clone(),
        day: day_timestamp(day),
        n: count(kept.len()),
        unknown,
        partial,
        min,
        p50,
        max,
        last,
        worst_verdict: ladder::worst(counted.iter().map(|evaluation| evaluation.verdict)),
        verdict_counts,
        min_score: extreme(scores.iter().copied(), Ordering::Less),
        max_score: extreme(scores.iter().copied(), Ordering::Greater),
    }
}

/// Of each kept sample's evaluations, the one at the highest `definition_rev` (D14); of two at
/// one revision, the one given last.
fn highest_rev<'a>(
    kept: &[(Instant, &SignalSampleData)],
    evaluations: &'a [SignalEvaluationData],
) -> Vec<&'a SignalEvaluationData> {
    let mut highest: BTreeMap<&str, &SignalEvaluationData> = kept
        .iter()
        .map(|(_, sample)| sample.sample_id.0.as_str())
        .filter_map(|id| {
            evaluations
                .iter()
                .filter(|evaluation| evaluation.sample_id.0 == id)
                .fold(
                    None,
                    |held: Option<&SignalEvaluationData>, evaluation| match held {
                        Some(held) if held.definition_rev > evaluation.definition_rev => Some(held),
                        _ => Some(evaluation),
                    },
                )
                .map(|evaluation| (id, evaluation))
        })
        .collect();
    // One evaluation per sample, in the order the samples were kept.
    kept.iter()
        .filter_map(|(_, sample)| highest.remove(sample.sample_id.0.as_str()))
        .collect()
}

/// The least (`Ordering::Less`) or greatest (`Ordering::Greater`) of `values`, by numeric order.
fn extreme<'a>(values: impl Iterator<Item = &'a Decimal>, wanted: Ordering) -> Option<Decimal> {
    values
        .fold(None, |held: Option<&Decimal>, value| match held {
            Some(held) if decimal_order(value, held) != Some(wanted) => Some(held),
            _ => Some(value),
        })
        .cloned()
}

/// A count as the `Integer` the record holds.
fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// The civil date of `days` after 1970-01-01 (Howard Hinnant's algorithm).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let date = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2), month, date)
}
