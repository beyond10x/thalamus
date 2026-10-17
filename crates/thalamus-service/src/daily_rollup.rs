//! The daily rollup (design § 4, lesson L7): at the end of every tick, after the passes that
//! record samples (a missed slot's `unknown` sample belongs to the day it fills), every closed UTC
//! day, a day before the clock's UTC day, that holds a sample of a series with no rollup row yet
//! is rolled up (`EventlogStore::roll_up_day`, through the generated `RecordRollup`). That is the
//! day that closed at the last UTC midnight, and also a day whose midnight passed while the
//! service was down. A series already rolled up for a day is not rolled up again, so a tick
//! writes only what is missing; a series refused (a signal no definition has) and a store
//! failure go into the tick's problems.

use std::collections::BTreeSet;

use thalamus_core::rollup::{DAY_SECONDS, day_start, day_text, rollup_id};
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_store::eventlog::EventlogStore;

use crate::scheduler::Tick;

/// Rolls up in `store` every UTC day that closed before `now` (Unix seconds) and has a sample of
/// a series without a row.
pub fn closed_day(
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    now: i64,
    tick: &mut Tick,
) {
    let today = now.div_euclid(DAY_SECONDS) * DAY_SECONDS;
    let held: BTreeSet<String> = store
        .rollups()
        .into_iter()
        .map(|rollup| rollup.rollup_id.0)
        .collect();
    let days: BTreeSet<i64> = store
        .samples()
        .iter()
        .filter_map(|sample| Some((sample, day_start(&sample.ts.0)?)))
        .filter(|(sample, day)| *day < today && !held.contains(&rollup_id(&sample.series, *day).0))
        .map(|(_, day)| day)
        .collect();
    for day in days {
        match store.roll_up_day(instance.instance_mut(), day, None) {
            Ok(rolled) => tick.problems.extend(rolled.refused),
            Err(error) => tick.problems.push(format!(
                "the rollup of {} was not recorded: {error}",
                day_text(day)
            )),
        }
    }
}
