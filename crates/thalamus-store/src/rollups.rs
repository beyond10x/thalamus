//! The rollup port: daily rollups on the instance's Eventlog stream as `thalamus.rollup/1`
//! records, the generated `DailyRollupStorage` over them, and the rollup of one closed UTC day
//! through the generated `RecordRollup` (design § 4, § 11 D14, D26).
//!
//! A record's body is the `RecordRollup` input in its wire form with a `schema` member: an absent
//! optional is an absent member. One row per series and day: the id is spelled from both
//! (`thalamus_core::rollup::rollup_id`), and a second append under a held id writes nothing,
//! whatever its body, so a re-run never doubles or replaces a row (`already-rolled-up`). The
//! Eventlog idempotency key is a digest of the id, since a series key may hold bytes, or a
//! length, that Eventlog does not store as a key.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use thalamus_core::rollup::{build, day_start};
use thalamus_model::behaviour::{DailyRollupStorage, Generated, SignalDefinitionStorage};
use thalamus_model::definition::{SignalDefinitionSnapshot, SignalId};
use thalamus_model::json;
use thalamus_model::observation::SeriesKey;
use thalamus_model::rollup::obligations::RecordRollupBehavior;
use thalamus_model::rollup::{
    DailyRollupData, DailyRollupSnapshot, DailyRollupState, RecordRollup, RecordRollupOutcome,
    RollupId,
};
use thalamus_model::server::wire;

use crate::eventlog::{EventlogStore, StoreError, part};

/// The schema of a rollup record.
pub const ROLLUP_SCHEMA: &str = "thalamus.rollup/1";

/// The event name a rollup is appended under.
const EVENT: &str = "thalamus.rollup";

/// What rolling up one day did, series by series in series-key order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RolledUp {
    /// The rows this call recorded.
    pub recorded: Vec<DailyRollupData>,
    /// The rows that were already held (`already-rolled-up`): nothing was written for them.
    pub held: Vec<DailyRollupData>,
    /// One line per series that was refused (`unknown-signal`, or a row the model refuses).
    pub refused: Vec<String>,
}

impl EventlogStore {
    /// Appends `rollup` unless a rollup with its id is stored. Answers whether it was appended.
    ///
    /// # Errors
    ///
    /// [`StoreError::Record`] when it does not read back as itself from its wire form, or the
    /// Eventlog failure. Nothing is written on any error.
    pub fn append_rollup(&mut self, rollup: &DailyRollupData) -> Result<bool, StoreError> {
        let body = encode(rollup)?;
        if decode(&body)? != *rollup {
            return Err(StoreError::Record(format!(
                "rollup {} does not read back as itself from its wire form",
                rollup.rollup_id.0
            )));
        }
        let key = key(&rollup.rollup_id)?;
        let mut appended = false;
        let answer = self.append_caught_up(|store| {
            if store.rollup(&rollup.rollup_id).is_some() {
                return Ok(());
            }
            store.append(EVENT, &key, body.clone(), None)?;
            appended = true;
            Ok(())
        });
        match answer {
            // Another writer stored a row under this id first, with its own body.
            Err(StoreError::Refused { .. }) if self.rollup(&rollup.rollup_id).is_some() => {
                Ok(false)
            }
            Err(error) => Err(error),
            Ok(()) => Ok(appended),
        }
    }

    /// The rollup stored under `id`, also when another writer appended it since this handle last
    /// moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn rollup(&self, id: &RollupId) -> Option<DailyRollupData> {
        self.rollups()
            .into_iter()
            .find(|rollup| rollup.rollup_id == *id)
    }

    /// Every stored rollup, in the order it was appended, with those other writers appended since
    /// this handle last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn rollups(&self) -> Vec<DailyRollupData> {
        let held = self
            .records()
            .iter()
            .filter(|record| record.schema() == Some(ROLLUP_SCHEMA))
            .filter_map(|record| decode(record.body()).ok());
        held.chain(self.tail().rollups).collect()
    }

    /// Rolls up every series with a sample on the UTC day starting at `day` (Unix seconds), of
    /// `signal` only when one is given, through the generated `RecordRollup` over `definitions`
    /// and this store. A series already rolled up for the day is answered as held and nothing is
    /// written for it. Whether the day has closed is the caller's to decide.
    ///
    /// # Errors
    ///
    /// The store failure that stopped the pass; the rows recorded before it stay recorded.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn roll_up_day<D: SignalDefinitionStorage>(
        &mut self,
        definitions: &mut D,
        day: i64,
        signal: Option<&SignalId>,
    ) -> Result<RolledUp, StoreError> {
        self.refresh()?;
        let samples: Vec<_> = self
            .samples()
            .iter()
            .filter(|sample| day_start(&sample.ts.0) == Some(day))
            .filter(|sample| signal.is_none_or(|signal| sample.signal == *signal))
            .cloned()
            .collect();
        let evaluations: Vec<_> = self
            .evaluations()
            .iter()
            .filter(|evaluation| {
                samples
                    .iter()
                    .any(|sample| sample.sample_id == evaluation.sample_id)
            })
            .cloned()
            .collect();
        let series: BTreeMap<String, (SeriesKey, SignalId)> = samples
            .iter()
            .map(|sample| {
                (
                    sample.series.0.clone(),
                    (sample.series.clone(), sample.signal.clone()),
                )
            })
            .collect();
        let mut rolled = RolledUp::default();
        for (series, signal) in series.into_values() {
            let input = build(&signal, &series, day, &samples, &evaluations);
            let id = input.rollup_id.clone();
            let mut behaviour = Generated::new(Ports {
                definitions: &mut *definitions,
                store: &mut *self,
                failed: None,
            });
            let outcome = behaviour.record_rollup(input);
            if let Some(error) = behaviour.ports.failed.take() {
                return Err(error);
            }
            match outcome {
                Ok(RecordRollupOutcome::Recorded { .. }) => {
                    rolled.recorded.extend(self.rollup(&id));
                }
                Ok(RecordRollupOutcome::AlreadyRolledUp { .. }) => {
                    rolled.held.extend(self.rollup(&id));
                }
                Ok(RecordRollupOutcome::UnknownSignal { error }) => rolled.refused.push(format!(
                    "{}: no definition has the signal id {}; not rolled up",
                    series.0, error.signal.0
                )),
                Err(unmet) => rolled
                    .refused
                    .push(format!("{}: refused by the model: {unmet}", series.0)),
            }
        }
        Ok(rolled)
    }
}

/// The ports `RecordRollup` runs over: the caller's definitions and this store's rollups. A store
/// failure while storing is kept, for [`EventlogStore::roll_up_day`] to answer, never a panic.
struct Ports<'a, D> {
    definitions: &'a mut D,
    store: &'a mut EventlogStore,
    failed: Option<StoreError>,
}

impl<D: SignalDefinitionStorage> SignalDefinitionStorage for Ports<'_, D> {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        self.definitions.get(identity)
    }

    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        self.definitions.put(snapshot);
    }

    fn delete(&mut self, identity: &SignalId) {
        self.definitions.delete(identity);
    }

    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        self.definitions.list()
    }
}

impl<D> DailyRollupStorage for Ports<'_, D> {
    fn get(&self, identity: &RollupId) -> Option<DailyRollupSnapshot> {
        DailyRollupStorage::get(&*self.store, identity)
    }

    fn put(&mut self, snapshot: DailyRollupSnapshot) {
        if let Err(error) = self.store.append_rollup(&snapshot.data) {
            self.failed = Some(error);
        }
    }

    fn delete(&mut self, identity: &RollupId) {
        DailyRollupStorage::delete(self.store, identity);
    }

    fn list(&self) -> Vec<DailyRollupSnapshot> {
        DailyRollupStorage::list(&*self.store)
    }
}

/// The Eventlog idempotency key of the rollup `id`: a digest, which Eventlog stores as a key
/// whatever bytes the series key holds.
fn key(id: &RollupId) -> Result<String, StoreError> {
    Ok(format!(
        "rollup:{}",
        eventlog_core::request_hash(&Value::from(id.0.as_str()))?
    ))
}

/// The `RecordRollup` input that records `rollup`: the same fields.
fn input(rollup: &DailyRollupData) -> RecordRollup {
    let rollup = rollup.clone();
    RecordRollup {
        rollup_id: rollup.rollup_id,
        signal: rollup.signal,
        series: rollup.series,
        day: rollup.day,
        n: rollup.n,
        unknown: rollup.unknown,
        partial: rollup.partial,
        min: rollup.min,
        p50: rollup.p50,
        max: rollup.max,
        last: rollup.last,
        worst_verdict: rollup.worst_verdict,
        verdict_counts: rollup.verdict_counts,
        min_score: rollup.min_score,
        max_score: rollup.max_score,
    }
}

/// The `thalamus.rollup/1` record of `rollup`.
pub(crate) fn encode(rollup: &DailyRollupData) -> Result<Value, StoreError> {
    let Value::Object(fields) = part(
        &input(rollup),
        wire::encode_command_thalamus_rollup_record_rollup,
    )?
    else {
        return Err(StoreError::Record(format!(
            "rollup {} is not a JSON object in its wire form",
            rollup.rollup_id.0
        )));
    };
    let mut body = Map::new();
    body.insert("schema".to_owned(), Value::from(ROLLUP_SCHEMA));
    body.extend(fields);
    Ok(Value::Object(body))
}

/// The rollup a `thalamus.rollup/1` record holds.
pub(crate) fn decode(body: &Value) -> Result<DailyRollupData, StoreError> {
    let mut fields = body
        .as_object()
        .cloned()
        .ok_or_else(|| StoreError::Record("the rollup record is not an object".to_owned()))?;
    if fields.remove("schema").as_ref().and_then(Value::as_str) != Some(ROLLUP_SCHEMA) {
        return Err(StoreError::Record(format!(
            "the record's schema is not {ROLLUP_SCHEMA}"
        )));
    }
    let parsed = json::parse(&Value::Object(fields).to_string())
        .map_err(|error| StoreError::Record(format!("rollup: {error}")))?;
    let input = wire::decode_command_thalamus_rollup_record_rollup(&parsed, "")
        .map_err(|error| StoreError::Record(error.to_string()))?;
    Ok(DailyRollupData {
        rollup_id: input.rollup_id,
        signal: input.signal,
        series: input.series,
        day: input.day,
        n: input.n,
        unknown: input.unknown,
        partial: input.partial,
        min: input.min,
        p50: input.p50,
        max: input.max,
        last: input.last,
        worst_verdict: input.worst_verdict,
        verdict_counts: input.verdict_counts,
        min_score: input.min_score,
        max_score: input.max_score,
    })
}

fn snapshot(data: DailyRollupData) -> DailyRollupSnapshot {
    DailyRollupSnapshot {
        state: DailyRollupState::Recorded,
        data,
    }
}

/// The generated port over the stream. Its methods return nothing, so a store failure while
/// storing panics with the reason rather than losing the row; [`EventlogStore::append_rollup`]
/// answers it as a value. A row under a held id is never replaced: the generated `RecordRollup`
/// answers `already-rolled-up` before it stores.
impl DailyRollupStorage for EventlogStore {
    fn get(&self, identity: &RollupId) -> Option<DailyRollupSnapshot> {
        self.rollup(identity).map(snapshot)
    }

    fn put(&mut self, snapshot: DailyRollupSnapshot) {
        if let Err(error) = self.append_rollup(&snapshot.data) {
            panic!(
                "rollup {} was not stored: {error}",
                snapshot.data.rollup_id.0
            );
        }
    }

    /// The stream is append-only: a held rollup is never removed.
    fn delete(&mut self, identity: &RollupId) {
        assert!(
            self.rollup(identity).is_none(),
            "rollup {} is on an append-only stream and is not deleted",
            identity.0
        );
    }

    fn list(&self) -> Vec<DailyRollupSnapshot> {
        self.rollups().into_iter().map(snapshot).collect()
    }
}
