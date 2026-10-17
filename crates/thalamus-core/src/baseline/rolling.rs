//! `rolling_median` (design § 3.1): the median of the last N samples of the series, from its
//! stored samples, for a `history: store` signal.
//!
//! N is `evaluation.rolling_n`. The samples counted are the `ok` ones whose `ts` is before the
//! graded sample's, latest first; `partial` and `unknown` samples are skipped, and a `ts` that
//! names no instant cannot be placed before anything and is skipped too. One instant counts once among
//! the N, as its row appended last (design § 11 D6), so a re-read of an instant does not crowd an
//! earlier sample out. Fewer than N such samples
//! make the baseline `UNKNOWN` with reason [`REASON_PREDATES_SERIES`] (rule 2): the series is not
//! yet N samples long, and a median of fewer is not the baseline the definition names.

use thalamus_model::observation::{CompleteReading, Reading, SeriesSamples};
use thalamus_model::primitives::{Decimal, Timestamp};

use super::{REASON_PREDATES_SERIES, Refusal, median as median_of, unknown};
use crate::behaviour::observation::Instant;

/// The `rolling_median` baseline over the last `n` `ok` samples of `history` (the series' stored
/// samples, in any order) before `ts`.
///
/// # Errors
///
/// [`Refusal::Timestamp`] when `ts` is not an RFC 3339 instant.
pub fn median(ts: &Timestamp, history: &[SeriesSamples], n: usize) -> Result<Reading, Refusal> {
    let at = Instant::parse(&ts.0).ok_or_else(|| Refusal::Timestamp {
        field: "ts",
        value: ts.0.clone(),
    })?;
    let mut earlier: Vec<(Instant, &Decimal)> = history
        .iter()
        .filter_map(|row| match &row.reading {
            Reading::Ok(CompleteReading { value }) => Some((Instant::parse(&row.ts.0)?, value)),
            Reading::Partial(_) | Reading::Unknown(_) => None,
        })
        .filter(|(sampled, _)| *sampled < at)
        .collect();
    // Stable, so of two rows at one instant the one stored last stays last; then one instant
    // counts once, as its last row (design § 11 D6).
    earlier.sort_by_key(|(sampled, _)| *sampled);
    let mut once: Vec<(Instant, &Decimal)> = Vec::with_capacity(earlier.len());
    for entry in earlier {
        match once.last_mut() {
            Some(last) if last.0 == entry.0 => *last = entry,
            _ => once.push(entry),
        }
    }
    let earlier = once;
    if n == 0 || earlier.len() < n {
        return Ok(unknown(REASON_PREDATES_SERIES.to_owned()));
    }
    let last: Vec<&Decimal> = earlier[earlier.len() - n..]
        .iter()
        .map(|(_, value)| *value)
        .collect();
    Ok(median_of(&last).unwrap_or_else(|| unknown(REASON_PREDATES_SERIES.to_owned())))
}
