//! The sample port: samples on the instance's Eventlog stream as `thalamus.sample/1` records, and
//! the generated `SignalSampleStorage` over them.
//!
//! A sample has no lifecycle beyond `Recorded` (`spec/domains/observation.yaml`), so the port only
//! appends: storing a sample already held with the same body is a retry and writes nothing, and
//! storing another body under a held id is refused.

use serde_json::{Map, Value};
use thalamus_model::behaviour::SignalSampleStorage;
use thalamus_model::observation::{
    SampleId, SeriesKey, SignalSampleData, SignalSampleSnapshot, SignalSampleState,
};
use thalamus_model::primitives::Timestamp;
use thalamus_model::server::wire;

use crate::eventlog::{
    EventlogStore, ID_TAKEN, SAMPLE_SCHEMA, StoreError, dims, dims_value, optional_text, part,
    text, unpart, unpart_optional,
};

/// The event name a sample is appended under.
const EVENT: &str = "thalamus.sample";

impl EventlogStore {
    /// Appends `sample` to the stream.
    ///
    /// # Errors
    ///
    /// [`StoreError::Refused`] with [`ID_TAKEN`] when its id is held with another body;
    /// [`StoreError::Record`] when it does not read back as itself from its wire form (a
    /// `Decimal` outside the published pattern, an id without its prefix); or the Eventlog
    /// failure. Nothing is written on any error.
    ///
    /// An append that meets another writer's append takes its records in and is retried, up to
    /// [`crate::eventlog::CATCH_UP_ATTEMPTS`] times ([`StoreError::Contended`] after that); one
    /// that meets its id stored with another body takes the stream in and is refused
    /// [`ID_TAKEN`], from a stale handle too.
    pub fn append_sample(&mut self, sample: &SignalSampleData) -> Result<(), StoreError> {
        let body = encode(sample)?;
        if decode(&body)? != *sample {
            return Err(StoreError::Record(format!(
                "sample {} does not read back as itself from its wire form",
                sample.sample_id.0
            )));
        }
        self.append_caught_up(|store| store.try_append_sample(sample, body.clone()))
    }

    fn try_append_sample(
        &mut self,
        sample: &SignalSampleData,
        body: Value,
    ) -> Result<(), StoreError> {
        if let Some(held) = self.held_sample(&sample.sample_id) {
            return if held == sample {
                Ok(())
            } else {
                Err(StoreError::Refused {
                    code: ID_TAKEN.to_owned(),
                })
            };
        }
        self.append(EVENT, &sample.sample_id.0, body, None)
    }

    /// The sample this handle has taken in under `id`.
    pub(crate) fn held_sample(&self, id: &SampleId) -> Option<&SignalSampleData> {
        self.sample_index
            .get(&id.0)
            .and_then(|index| self.samples.get(*index))
    }

    /// The sample stored under `id`, also when another writer appended it since this handle last
    /// moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn sample(&self, id: &SampleId) -> Option<SignalSampleData> {
        if let Some(held) = self.held_sample(id) {
            return Some(held.clone());
        }
        self.tail()
            .samples
            .into_iter()
            .find(|sample| sample.sample_id == *id)
    }

    /// Every sample this handle has taken in, in the order it was appended: the stream as of the
    /// handle's last call that took `&mut self`.
    pub fn samples(&self) -> &[SignalSampleData] {
        &self.samples
    }

    /// The first sample stored for `series`: the earliest appended, whatever its `ts`.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn first_sample(&self, series: &SeriesKey) -> Option<SignalSampleData> {
        if let Some(first) = self.samples.iter().find(|sample| sample.series == *series) {
            return Some(first.clone());
        }
        self.tail()
            .samples
            .into_iter()
            .find(|sample| sample.series == *series)
    }

    /// Every stored sample, in the order it was appended, with those other writers appended
    /// since this handle last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn current_samples(&self) -> Vec<SignalSampleData> {
        let mut all = self.samples.clone();
        all.extend(self.tail().samples);
        all
    }
}

/// The `thalamus.sample/1` record of `sample`. An absent denominator is an absent member.
pub(crate) fn encode(sample: &SignalSampleData) -> Result<Value, StoreError> {
    let mut body = Map::new();
    body.insert("schema".to_owned(), Value::from(SAMPLE_SCHEMA));
    body.insert(
        "sample_id".to_owned(),
        part(
            &sample.sample_id,
            wire::encode_thalamus_observation_sample_id,
        )?,
    );
    body.insert(
        "signal".to_owned(),
        part(&sample.signal, wire::encode_thalamus_definition_signal_id)?,
    );
    body.insert(
        "series".to_owned(),
        part(&sample.series, wire::encode_thalamus_observation_series_key)?,
    );
    body.insert("dims".to_owned(), dims_value(&sample.dims));
    body.insert("ts".to_owned(), Value::from(sample.ts.0.as_str()));
    body.insert(
        "window".to_owned(),
        part(
            &sample.window,
            wire::encode_thalamus_observation_time_window,
        )?,
    );
    body.insert(
        "reading".to_owned(),
        part(&sample.reading, wire::encode_thalamus_observation_reading)?,
    );
    if let Some(text) = &sample.text {
        body.insert("text".to_owned(), Value::from(text.as_str()));
    }
    body.insert(
        "unit".to_owned(),
        part(&sample.unit, wire::encode_thalamus_definition_unit)?,
    );
    if let Some(denominator) = &sample.denominator {
        body.insert(
            "denominator".to_owned(),
            part(denominator, wire::encode_thalamus_observation_denominator)?,
        );
    }
    body.insert(
        "provenance".to_owned(),
        part(
            &sample.provenance,
            wire::encode_thalamus_observation_provenance,
        )?,
    );
    Ok(Value::Object(body))
}

/// The sample a `thalamus.sample/1` record holds.
pub(crate) fn decode(body: &Value) -> Result<SignalSampleData, StoreError> {
    Ok(SignalSampleData {
        sample_id: unpart(
            body,
            "sample_id",
            wire::decode_thalamus_observation_sample_id,
        )?,
        signal: unpart(body, "signal", wire::decode_thalamus_definition_signal_id)?,
        series: unpart(body, "series", wire::decode_thalamus_observation_series_key)?,
        dims: dims(body)?,
        ts: Timestamp(text(body, "ts")?),
        window: unpart(
            body,
            "window",
            wire::decode_thalamus_observation_time_window,
        )?,
        reading: unpart(body, "reading", wire::decode_thalamus_observation_reading)?,
        text: optional_text(body, "text")?,
        unit: unpart(body, "unit", wire::decode_thalamus_definition_unit)?,
        denominator: unpart_optional(
            body,
            "denominator",
            wire::decode_thalamus_observation_denominator,
        )?,
        provenance: unpart(
            body,
            "provenance",
            wire::decode_thalamus_observation_provenance,
        )?,
    })
}

fn snapshot(data: SignalSampleData) -> SignalSampleSnapshot {
    SignalSampleSnapshot {
        state: SignalSampleState::Recorded,
        data,
    }
}

/// The generated port over the stream. Its methods return nothing, so a store failure while
/// storing panics with the reason rather than losing the sample; [`EventlogStore::append_sample`]
/// answers the same failure as a value.
impl SignalSampleStorage for EventlogStore {
    fn get(&self, identity: &SampleId) -> Option<SignalSampleSnapshot> {
        self.sample(identity).map(snapshot)
    }

    fn put(&mut self, snapshot: SignalSampleSnapshot) {
        if let Err(error) = self.append_sample(&snapshot.data) {
            panic!(
                "sample {} was not stored: {error}",
                snapshot.data.sample_id.0
            );
        }
    }

    /// The stream is append-only: a held sample is never removed (retention is M2).
    fn delete(&mut self, identity: &SampleId) {
        assert!(
            self.sample(identity).is_none(),
            "sample {} is on an append-only stream and is not deleted",
            identity.0
        );
    }

    fn list(&self) -> Vec<SignalSampleSnapshot> {
        self.current_samples().into_iter().map(snapshot).collect()
    }
}
