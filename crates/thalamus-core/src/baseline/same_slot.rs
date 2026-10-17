//! `same_slot_median_4w` (design § 3.1): the median of the same weekday and hour over the last 4
//! weeks, from the series' stored samples.
//!
//! The slots of a graded `ts` are the UTC hours holding `ts` minus 7, 14, 21 and 28 days, so the
//! weekday and hour are UTC, as rollup days are (design § 4), whatever offset a `ts` is written in.
//! A slot's reading is its `ok` sample closest to its instant (`ts` minus the weeks); of two
//! equally close, the earlier.
//!
//! Rule 1: a slot with no `ok` sample makes the baseline `UNKNOWN` with reason
//! [`REASON_SAME_SLOT_NOT_READ`]; a missing week is not a median of three. Rule 2: a slot whose hour
//! ends at or before the series began makes it `UNKNOWN` with reason [`REASON_PREDATES_SERIES`],
//! and an unknown series age with reason [`REASON_SERIES_AGE_UNKNOWN`]. Rule 2 is decided first.

use thalamus_model::observation::{CompleteReading, Reading, SeriesSamples};
use thalamus_model::primitives::{Decimal, Timestamp};

use super::{
    REASON_PREDATES_SERIES, REASON_SAME_SLOT_NOT_READ, REASON_SERIES_AGE_UNKNOWN, Refusal,
    median as median_of, unknown,
};
use crate::behaviour::observation::Instant;

/// The weeks before the graded sample the slots lie at.
pub const WEEKS: [i64; 4] = [1, 2, 3, 4];

/// How far before the graded sample the earliest slot can start, in seconds: 28 days and one
/// hour. A caller reading the series' history over this span before `ts` holds every slot.
pub const SPAN_S: i64 = 28 * DAY_S + HOUR_S;

const DAY_S: i64 = 86_400;
const HOUR_S: i64 = 3_600;
const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// The `same_slot_median_4w` baseline of a sample at `ts`, from `history`, the series' stored
/// samples in any order. `series_age_s` is the age of the series at `ts`, in seconds.
///
/// # Errors
///
/// [`Refusal::Timestamp`] when `ts` is not an RFC 3339 instant.
pub fn median(
    ts: &Timestamp,
    history: &[SeriesSamples],
    series_age_s: Option<i64>,
) -> Result<Reading, Refusal> {
    let at = Instant::parse(&ts.0)
        .ok_or_else(|| Refusal::Timestamp {
            field: "ts",
            value: ts.0.clone(),
        })?
        .nanoseconds();
    let Some(age_s) = series_age_s else {
        return Ok(unknown(REASON_SERIES_AGE_UNKNOWN.to_owned()));
    };
    let series_start = at - i128::from(age_s) * NANOS_PER_SECOND;
    let hour = i128::from(HOUR_S) * NANOS_PER_SECOND;
    let slots: Vec<(i128, i128)> = WEEKS
        .iter()
        .map(|weeks| {
            let instant = at - i128::from(weeks * 7 * DAY_S) * NANOS_PER_SECOND;
            (instant, instant - instant.rem_euclid(hour))
        })
        .collect();
    if slots
        .iter()
        .any(|&(_, slot_start)| slot_start + hour <= series_start)
    {
        return Ok(unknown(REASON_PREDATES_SERIES.to_owned()));
    }
    let readable: Vec<(i128, &Decimal)> = history
        .iter()
        .filter_map(|row| match &row.reading {
            Reading::Ok(CompleteReading { value }) => {
                Some((Instant::parse(&row.ts.0)?.nanoseconds(), value))
            }
            Reading::Partial(_) | Reading::Unknown(_) => None,
        })
        .collect();
    let mut values = Vec::with_capacity(slots.len());
    for (instant, slot_start) in slots {
        let closest = readable
            .iter()
            .filter(|(sampled, _)| slot_start <= *sampled && *sampled < slot_start + hour)
            // Reversed: of two rows at one instant the one appended last wins, as in rolling.
            .rev()
            .min_by_key(|(sampled, _)| ((*sampled - instant).abs(), *sampled));
        match closest {
            Some((_, value)) => values.push(*value),
            None => return Ok(unknown(REASON_SAME_SLOT_NOT_READ.to_owned())),
        }
    }
    Ok(median_of(&values).unwrap_or_else(|| unknown(REASON_SAME_SLOT_NOT_READ.to_owned())))
}
