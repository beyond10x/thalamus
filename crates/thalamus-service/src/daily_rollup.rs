//! The daily rollup (design § 4, lesson L7): on every tick, the day before the clock's UTC day,
//! the day that closed at the last UTC midnight, is rolled up for every series with a sample on
//! it (`EventlogStore::roll_up_day`, through the generated `RecordRollup`). A series already
//! rolled up for that day is not rolled up again, so only the first tick after midnight writes;
//! a series refused (a signal no definition has) and a store failure go into the tick's problems.

use thalamus_core::rollup::{DAY_SECONDS, day_text};
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_store::eventlog::EventlogStore;

use crate::scheduler::Tick;

/// Rolls up in `store` the UTC day that closed before `now` (Unix seconds).
pub fn closed_day(
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    now: i64,
    tick: &mut Tick,
) {
    let day = now.div_euclid(DAY_SECONDS) * DAY_SECONDS - DAY_SECONDS;
    match store.roll_up_day(instance.instance_mut(), day, None) {
        Ok(rolled) => tick.problems.extend(rolled.refused),
        Err(error) => tick.problems.push(format!(
            "the rollup of {} was not recorded: {error}",
            day_text(day)
        )),
    }
}
