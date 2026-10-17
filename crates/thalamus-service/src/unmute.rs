//! An expired mute (design § 11 D9, D19, D28): on every tick, before sampling, each `Muted`
//! definition whose `mute_until` is at or before now is unmuted for the pass by the generated
//! `UnmuteDefinition` (`thalamus_pipeline::definitions`), so it is sampled as `Active`. Its
//! `DefinitionUnmuted` is recorded once in the store, as actor `thalamus.definition.Service`, and
//! a warning naming the definition file goes into the tick's problems on every pass while the
//! file keeps the stale mute. A mute whose expiry is in the future stays unsampled.
//!
//! The service never mutes and never writes a definition file: the file is the one copy, and
//! someone edits it to drop the mute.

use thalamus_model::actor::Actor;
use thalamus_model::primitives::Timestamp;
use thalamus_pipeline::definitions;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_store::definition_events::UnmutedRecord;
use thalamus_store::eventlog::EventlogStore;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::scheduler::Tick;

/// Unmutes in `instance` every definition whose mute expired at or before `now` (Unix seconds),
/// records each unmute in `store`, and warns in `tick` about each stale definition file.
pub fn effective_definitions(
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    now: i64,
    tick: &mut Tick,
) {
    let Some(at) = OffsetDateTime::from_unix_timestamp(now)
        .ok()
        .and_then(|at| at.format(&Rfc3339).ok())
        .map(Timestamp)
    else {
        tick.problems.push(format!(
            "{now} names no instant; no expired mute was looked for"
        ));
        return;
    };
    for unmuted in definitions::unmute_expired(instance, &at) {
        tick.problems.push(format!(
            "{}: mute_until {} has passed; {} is sampled as Active, and the file still says \
             muted: edit it to drop the mute",
            unmuted.file, unmuted.mute_until.0, unmuted.event.signal_id.0
        ));
        let record = UnmutedRecord {
            rev: unmuted.rev,
            mute_until: unmuted.mute_until,
            actor: Actor::DefinitionService,
            at: at.clone(),
            event: unmuted.event,
        };
        if let Err(error) = store.append_definition_unmuted(&record) {
            tick.problems.push(format!(
                "{}: the unmute was not recorded: {error}",
                record.event.signal_id.0
            ));
        }
    }
}
