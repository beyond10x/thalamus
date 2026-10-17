//! The effective lifecycle of an instance's definitions (design § 11 D9, D19).
//!
//! The definition file is the only copy of a definition, its lifecycle included, and nothing here
//! writes it. A `Muted` definition whose `mute_until` is at or before now is unmuted by the
//! generated `thalamus.definition.UnmuteDefinition` behaviour ([`unmute_expired`]) over a port
//! whose writes land in an in-memory [`Overlay`] of the loaded instance, never in the file. The
//! file-backed definition port seen through that overlay ([`Effective`]) answers the moved
//! snapshot, `Active` with no mute, to every reader of the loaded instance, the generated
//! `RecordSample` guard among them; every other definition answers its file lifecycle.

use std::collections::BTreeMap;

use thalamus_core::behaviour::observation::timestamp_order;
use thalamus_core::registry::Instance;
use thalamus_model::behaviour::{Generated, SignalDefinitionStorage};
use thalamus_model::definition::obligations::UnmuteDefinitionBehavior;
use thalamus_model::definition::{
    DefinitionUnmuted, SignalDefinitionSnapshot, SignalDefinitionState, SignalId, UnmuteDefinition,
    UnmuteDefinitionOutcome,
};
use thalamus_model::primitives::Timestamp;

use crate::instance::LoadedInstance;

/// The snapshots that override the files of a loaded instance, by definition id: the definitions
/// unmuted in memory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overlay {
    held: BTreeMap<String, SignalDefinitionSnapshot>,
}

impl Overlay {
    /// The snapshot that overrides the file of `signal`, if any.
    #[must_use]
    pub fn get(&self, signal: &str) -> Option<&SignalDefinitionSnapshot> {
        self.held.get(signal)
    }
}

/// One definition unmuted because its mute had expired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unmuted {
    /// The event `UnmuteDefinition` published.
    pub event: DefinitionUnmuted,
    /// The definition's `rev`.
    pub rev: i64,
    /// The expired `mute_until` the file still carries.
    pub mute_until: Timestamp,
    /// The definition file, which keeps its stale mute.
    pub file: String,
}

/// The effective lifecycle state of `signal`: its overlay snapshot's, or else its file's.
#[must_use]
pub fn state(instance: &LoadedInstance, signal: &str) -> Option<SignalDefinitionState> {
    instance
        .overlay()
        .get(signal)
        .map(|snapshot| snapshot.state)
        .or_else(|| {
            instance
                .instance()
                .definition(signal)
                .map(thalamus_core::registry::Definition::state)
        })
}

/// Every `Muted` definition of `instance` whose `mute_until` is at or before `now`, unmuted by
/// the generated `UnmuteDefinition` into the instance's overlay, in id order. A definition
/// already unmuted in the overlay is not unmuted again; the file is never written.
pub fn unmute_expired(instance: &mut LoadedInstance, now: &Timestamp) -> Vec<Unmuted> {
    let expired: Vec<(SignalId, i64, Timestamp, String)> = instance
        .instance()
        .definitions()
        .filter(|definition| {
            state(instance, &definition.data().signal_id.0) == Some(SignalDefinitionState::Muted)
        })
        .filter_map(|definition| {
            let data = definition.data();
            let until = data.mute_until.clone()?;
            timestamp_order(&until.0, &now.0).is_le().then(|| {
                (
                    data.signal_id.clone(),
                    data.rev,
                    until,
                    definition.file().to_owned(),
                )
            })
        })
        .collect();
    let mut unmuted = Vec::new();
    for (signal_id, rev, mute_until, file) in expired {
        let (files, overlay) = instance.parts_mut();
        let mut behaviour = Generated::new(Unmuting {
            effective: Effective { files, overlay },
        });
        if let Ok(UnmuteDefinitionOutcome::Unmuted { definition_unmuted }) =
            behaviour.unmute_definition(UnmuteDefinition { signal_id })
        {
            unmuted.push(Unmuted {
                event: definition_unmuted,
                rev,
                mute_until,
                file,
            });
        }
    }
    unmuted
}

/// The file-backed definition port of a loaded instance seen through its [`Overlay`]: reads
/// answer the overlay's snapshot where there is one; writes go to the files' port, which refuses
/// and records them.
pub struct Effective<'a> {
    pub(crate) files: &'a mut Instance,
    pub(crate) overlay: &'a mut Overlay,
}

impl SignalDefinitionStorage for Effective<'_> {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        self.overlay
            .get(&identity.0)
            .cloned()
            .or_else(|| SignalDefinitionStorage::get(&*self.files, identity))
    }

    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        SignalDefinitionStorage::put(self.files, snapshot);
    }

    fn delete(&mut self, identity: &SignalId) {
        SignalDefinitionStorage::delete(self.files, identity);
    }

    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        SignalDefinitionStorage::list(&*self.files)
            .into_iter()
            .map(|snapshot| {
                self.overlay
                    .get(&snapshot.data.signal_id.0)
                    .cloned()
                    .unwrap_or(snapshot)
            })
            .collect()
    }
}

/// The port `UnmuteDefinition` runs over: [`Effective`], whose one write, the unmuted snapshot,
/// is held in the overlay instead of the file.
struct Unmuting<'a> {
    effective: Effective<'a>,
}

impl SignalDefinitionStorage for Unmuting<'_> {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        self.effective.get(identity)
    }

    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        self.effective
            .overlay
            .held
            .insert(snapshot.data.signal_id.0.clone(), snapshot);
    }

    fn delete(&mut self, identity: &SignalId) {
        self.effective.delete(identity);
    }

    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        self.effective.list()
    }
}
