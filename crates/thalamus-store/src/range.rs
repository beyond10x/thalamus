//! The series range read: the samples and the evaluations of one series over a time range, in
//! `ts` order. It is the store's read of the `thalamus.observation.SeriesSamples` and
//! `thalamus.evaluation.SeriesVerdicts` views, bounded in time, and the one read of history that
//! comparison, store-backed baselines and the hysteresis replay go through.
//!
//! It reads `ts` exactly as the SeriesSamples view does: with `thalamus_core`'s instant reader
//! (the ESS reference grammar) and in the view's order (`in_time_order`). Over a range holding
//! every instant of a series it answers the view's rows in the view's order.

use thalamus_core::behaviour::observation::{Instant, in_time_order};
use thalamus_model::evaluation::SignalEvaluationData;
use thalamus_model::observation::{SeriesKey, SignalSampleData};
use thalamus_model::primitives::Timestamp;

use crate::eventlog::EventlogStore;

/// What one series holds over a time range, each list in `ts` order; rows at one instant keep
/// the order they were appended in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeriesRange {
    /// The series' samples in the range.
    pub samples: Vec<SignalSampleData>,
    /// The series' evaluations in the range.
    pub evaluations: Vec<SignalEvaluationData>,
}

/// Whether `ts` is in `[from, to)`. A `ts` that names no instant cannot be placed in time and is
/// in every range, where the view shows it too, rather than vanishing from history.
fn within(ts: &Timestamp, from: Instant, to: Instant) -> bool {
    Instant::parse(&ts.0).is_none_or(|at| from <= at && at < to)
}

impl EventlogStore {
    /// The samples and evaluations of `series` whose `ts` is at or after `from` and before `to`,
    /// each in the view's `ts` order. A row whose `ts` names no instant is in every range, placed
    /// as the view places it; a bound that names no instant makes the range empty.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::current_samples`]).
    pub fn series_range(
        &self,
        series: &SeriesKey,
        from: &Timestamp,
        to: &Timestamp,
    ) -> SeriesRange {
        let (Some(from), Some(to)) = (Instant::parse(&from.0), Instant::parse(&to.0)) else {
            return SeriesRange::default();
        };
        let samples = self
            .current_samples()
            .into_iter()
            .filter(|sample| sample.series == *series && within(&sample.ts, from, to))
            .collect();
        let evaluations = self
            .current_evaluations()
            .into_iter()
            .filter(|evaluation| evaluation.series == *series && within(&evaluation.ts, from, to))
            .collect();
        SeriesRange {
            samples: in_time_order(samples, |sample| sample.ts.0.as_str()),
            evaluations: in_time_order(evaluations, |evaluation| evaluation.ts.0.as_str()),
        }
    }
}
