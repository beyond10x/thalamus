//! The definition event port: `thalamus.definition.DefinitionUnmuted` on the instance's Eventlog
//! stream as `thalamus.definition-event/1` records (design § 11 D9, D19).
//!
//! The definition file is the only copy of a definition; the service records an unmute it acted
//! on here, so the change is visible, and never in the file. A record carries the event, the
//! definition's `rev`, the expired `mute_until` the unmute acted on, the actor and the instant.
//! One unmute is recorded once per `(signal, rev, mute_until)`: a later append of the same key
//! writes nothing, whatever its instant.

use serde_json::{Map, Value};
use thalamus_model::actor::Actor;
use thalamus_model::definition::{DefinitionUnmuted, SignalId};
use thalamus_model::primitives::Timestamp;
use thalamus_model::server::wire;

use crate::eventlog::{EventlogStore, StoreError, part, text, unpart};

/// The schema of a definition event record.
pub const DEFINITION_EVENT_SCHEMA: &str = "thalamus.definition-event/1";
/// The qualified name of the unmute event, as the specification spells it.
pub const DEFINITION_UNMUTED: &str = "thalamus.definition.DefinitionUnmuted";

/// The event name a definition event is appended under.
const EVENT: &str = "thalamus.definition-event";

/// One recorded unmute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmutedRecord {
    /// The event `UnmuteDefinition` published.
    pub event: DefinitionUnmuted,
    /// The definition's `rev` when it was unmuted.
    pub rev: i64,
    /// The expired `mute_until` the unmute acted on.
    pub mute_until: Timestamp,
    /// Who unmuted it.
    pub actor: Actor,
    /// When.
    pub at: Timestamp,
}

impl UnmutedRecord {
    /// The key one unmute is recorded once under: `(signal, rev, mute_until)`.
    fn key(&self) -> String {
        format!(
            "unmuted:{}:{}:{}",
            self.event.signal_id.0, self.rev, self.mute_until.0
        )
    }

    fn same_unmute(&self, other: &Self) -> bool {
        self.event == other.event && self.rev == other.rev && self.mute_until == other.mute_until
    }
}

impl EventlogStore {
    /// Appends `record` unless the stream holds an unmute of the same `(signal, rev,
    /// mute_until)`. Answers whether it was appended.
    ///
    /// # Errors
    ///
    /// [`StoreError::Record`] when it does not read back as itself from its wire form, or the
    /// Eventlog failure. Nothing is written on any error.
    pub fn append_definition_unmuted(
        &mut self,
        record: &UnmutedRecord,
    ) -> Result<bool, StoreError> {
        let body = encode(record)?;
        if decode(&body)? != *record {
            return Err(StoreError::Record(format!(
                "the unmute of {} does not read back as itself from its wire form",
                record.event.signal_id.0
            )));
        }
        let mut appended = false;
        let answer = self.append_caught_up(|store| {
            if store.holds_unmute(record) {
                return Ok(());
            }
            store.append(EVENT, &record.key(), body.clone(), None)?;
            appended = true;
            Ok(())
        });
        match answer {
            // Another writer recorded the same unmute under its own instant first.
            Err(StoreError::Refused { .. }) if self.holds_unmute(record) => Ok(false),
            Err(error) => Err(error),
            Ok(()) => Ok(appended),
        }
    }

    fn holds_unmute(&self, record: &UnmutedRecord) -> bool {
        self.definition_events(&record.event.signal_id)
            .iter()
            .any(|held| held.same_unmute(record))
    }

    /// Every unmute of `signal` stored, in the order it was appended, with those other writers
    /// appended since this handle last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn definition_events(&self, signal: &SignalId) -> Vec<UnmutedRecord> {
        let held = self
            .records()
            .iter()
            .filter(|record| record.schema() == Some(DEFINITION_EVENT_SCHEMA))
            .filter_map(|record| decode(record.body()).ok());
        held.chain(self.tail().unmuted)
            .filter(|record| record.event.signal_id == *signal)
            .collect()
    }
}

/// The `thalamus.definition-event/1` record of `record`.
pub(crate) fn encode(record: &UnmutedRecord) -> Result<Value, StoreError> {
    let mut body = Map::new();
    body.insert("schema".to_owned(), Value::from(DEFINITION_EVENT_SCHEMA));
    body.insert("event".to_owned(), Value::from(DEFINITION_UNMUTED));
    body.insert(
        "signal_id".to_owned(),
        part(
            &record.event.signal_id,
            wire::encode_thalamus_definition_signal_id,
        )?,
    );
    body.insert("rev".to_owned(), Value::from(record.rev));
    body.insert(
        "mute_until".to_owned(),
        Value::from(record.mute_until.0.as_str()),
    );
    body.insert("actor".to_owned(), Value::from(record.actor.name()));
    body.insert("at".to_owned(), Value::from(record.at.0.as_str()));
    Ok(Value::Object(body))
}

/// The unmute a `thalamus.definition-event/1` record holds.
pub(crate) fn decode(body: &Value) -> Result<UnmutedRecord, StoreError> {
    let event = text(body, "event")?;
    if event != DEFINITION_UNMUTED {
        return Err(StoreError::Record(format!(
            "the definition event `{event}` is not {DEFINITION_UNMUTED}"
        )));
    }
    let actor = text(body, "actor")?;
    let actor = Actor::ALL
        .iter()
        .copied()
        .find(|known| known.name() == actor)
        .ok_or_else(|| StoreError::Record(format!("`{actor}` is no declared actor")))?;
    let rev = body
        .get("rev")
        .and_then(Value::as_i64)
        .ok_or_else(|| StoreError::Record("the record has no integer `rev`".to_owned()))?;
    Ok(UnmutedRecord {
        event: DefinitionUnmuted {
            signal_id: unpart(
                body,
                "signal_id",
                wire::decode_thalamus_definition_signal_id,
            )?,
        },
        rev,
        mute_until: Timestamp(text(body, "mute_until")?),
        actor,
        at: Timestamp(text(body, "at")?),
    })
}
