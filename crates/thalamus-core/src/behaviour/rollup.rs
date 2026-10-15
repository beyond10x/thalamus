//! The rollup obligation of the synthesis plan (`generated/thalamus-model/PLAN.md`): the view
//! query `thalamus.rollup.SeriesRollups`. `thalamus.rollup.RecordRollup` is generated.
//!
//! It reads only through the generated storage port: no I/O and no clock of its own.

use thalamus_model::behaviour::DailyRollupStorage;
use thalamus_model::obligation::UnmetObligation;
use thalamus_model::observation::SeriesKey;
use thalamus_model::rollup::SeriesRollups;

use super::observation::in_time_order;

/// `thalamus.rollup.SeriesRollups`: the daily rows whose `series` is `series`, oldest day first
/// as the view declares (`order_by: [day asc]`). Two rows of one day keep the store's order.
///
/// # Errors
///
/// None today. The `Result` is the obligation's contract.
pub fn series_rollups<P>(
    ports: &P,
    series: &SeriesKey,
) -> Result<Vec<SeriesRollups>, UnmetObligation>
where
    P: DailyRollupStorage + ?Sized,
{
    let rows: Vec<SeriesRollups> = DailyRollupStorage::list(ports)
        .into_iter()
        .filter(|rollup| rollup.data.series == *series)
        .map(|rollup| SeriesRollups {
            rollup_id: rollup.data.rollup_id,
            signal: rollup.data.signal,
            day: rollup.data.day,
            n: rollup.data.n,
            unknown: rollup.data.unknown,
            partial: rollup.data.partial,
            min: rollup.data.min,
            p50: rollup.data.p50,
            max: rollup.data.max,
            last: rollup.data.last,
            worst_verdict: rollup.data.worst_verdict,
            verdict_counts: rollup.data.verdict_counts,
            min_score: rollup.data.min_score,
            max_score: rollup.data.max_score,
        })
        .collect();
    Ok(in_time_order(rows, |row| row.day.0.as_str()))
}
