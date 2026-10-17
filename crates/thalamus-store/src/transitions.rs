//! The transition port: settled verdict changes (design § 3.2, § 11 D13) on the instance's
//! Eventlog stream as `thalamus.transition/1` records, the generated `TransitionStorage` over
//! them, `thalamus.transition.RecordTransition` through the generated command, and the
//! `thalamus.transition.SeriesTransitions` read.
//!
//! A transition copies what a reader needs from the evaluations that settled it, so it outlives
//! their raw retention: [`EventlogStore::series_transition_evidence`] answers each referenced
//! evaluation as held or as [`Evidence::Expired`], never as an error.

use serde_json::{Map, Value};
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_core::behaviour::transition::{record_transition, series_transitions};
use thalamus_model::behaviour::{SignalDefinitionStorage, TransitionStorage};
use thalamus_model::definition::{SignalDefinitionSnapshot, SignalId};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData};
use thalamus_model::json;
use thalamus_model::observation::SeriesKey;
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::server::wire;
use thalamus_model::transition::{
    RecordTransition, RecordTransitionOutcome, SeriesTransitions, TransitionData, TransitionId,
    TransitionSnapshot, TransitionState,
};

use crate::eventlog::{
    EventlogStore, ID_TAKEN, StoreError, dims, dims_value, part, text, unpart, unpart_optional,
};

/// The wire schema of a stored transition.
pub const TRANSITION_SCHEMA: &str = "thalamus.transition/1";

/// The event name a transition is appended under.
const EVENT: &str = "thalamus.transition";

/// One evaluation a transition references, as a reader finds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    /// The evaluation is held.
    Held(Box<SignalEvaluationData>),
    /// The evaluation has left raw retention: only its id is left, in the transition (D13).
    Expired(EvaluationId),
}

/// One transition of a series with the evaluations it references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionEvidence {
    /// The transition's row of `thalamus.transition.SeriesTransitions`: the copied fields.
    pub transition: SeriesTransitions,
    /// Its evaluations, in the order it names them.
    pub evaluations: Vec<Evidence>,
}

/// Why `RecordTransition` stored nothing.
#[derive(Debug)]
pub enum RecordError {
    /// The command decided an outcome other than `recorded` (`unknown-signal`).
    Refused(Box<RecordTransitionOutcome>),
    /// The command met an obligation it could not meet: the identity sequence is spent, or the
    /// transition would break an entity invariant.
    Unmet {
        /// What could not be met.
        capability: &'static str,
        /// The invariant or sequence it names.
        source: &'static str,
    },
    /// The store refused the append or failed.
    Store(StoreError),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(outcome) => match &**outcome {
                RecordTransitionOutcome::UnknownSignal { error } => {
                    write!(f, "no definition has the signal {}", error.signal.0)
                }
                RecordTransitionOutcome::Recorded { .. } => f.write_str("recorded"),
            },
            Self::Unmet { capability, source } => write!(f, "{capability}: {source}"),
            Self::Store(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RecordError {}

impl EventlogStore {
    /// Records `input` through the generated `RecordTransition`, with `definitions` answering
    /// whether its signal is defined, under a `trn_` id this store mints. Answers the stored
    /// transition.
    ///
    /// # Errors
    ///
    /// [`RecordError`]; nothing is stored on any error.
    pub fn record_transition<D: SignalDefinitionStorage + ?Sized>(
        &mut self,
        definitions: &D,
        input: RecordTransition,
    ) -> Result<TransitionData, RecordError> {
        let mut ports = Ports {
            store: self,
            definitions,
            failure: None,
        };
        let outcome = record_transition(&mut ports, input);
        let failure = ports.failure.take();
        match (outcome, failure) {
            (_, Some(error)) => Err(RecordError::Store(error)),
            (Err(unmet), None) => Err(RecordError::Unmet {
                capability: unmet.capability,
                source: unmet.source,
            }),
            (Ok(RecordTransitionOutcome::Recorded { response, .. }), None) => {
                self.transition(&response.transition_id).ok_or_else(|| {
                    RecordError::Store(StoreError::Record(format!(
                        "transition {} was recorded and does not read back",
                        response.transition_id.0
                    )))
                })
            }
            (Ok(other), None) => Err(RecordError::Refused(Box::new(other))),
        }
    }

    /// Appends `transition` to the stream.
    ///
    /// # Errors
    ///
    /// [`StoreError::Refused`] with [`ID_TAKEN`] when its id is held with another body;
    /// [`StoreError::Record`] when it does not read back as itself from its wire form; or the
    /// Eventlog failure. Nothing is written on any error.
    pub fn append_transition(&mut self, transition: &TransitionData) -> Result<(), StoreError> {
        let body = encode(transition)?;
        if decode(&body)? != *transition {
            return Err(StoreError::Record(format!(
                "transition {} does not read back as itself from its wire form",
                transition.transition_id.0
            )));
        }
        self.append_caught_up(|store| match store.transition(&transition.transition_id) {
            Some(held) if held == *transition => Ok(()),
            Some(_) => Err(StoreError::Refused {
                code: ID_TAKEN.to_owned(),
            }),
            None => store.append(EVENT, &transition.transition_id.0, body.clone(), None),
        })
    }

    /// Every stored transition, in the order it was appended, with those other writers appended
    /// since this handle last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn transitions(&self) -> Vec<TransitionData> {
        let held = self
            .records()
            .iter()
            .filter(|record| record.schema() == Some(TRANSITION_SCHEMA))
            .filter_map(|record| decode(record.body()).ok());
        held.chain(self.tail().transitions).collect()
    }

    /// The transition stored under `id`.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn transition(&self, id: &TransitionId) -> Option<TransitionData> {
        self.transitions()
            .into_iter()
            .find(|transition| transition.transition_id == *id)
    }

    /// The `thalamus.transition.SeriesTransitions` rows of `series`: its transitions in
    /// `settled_ts` order.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn series_transitions(&self, series: &SeriesKey) -> Vec<SeriesTransitions> {
        series_transitions(self, series).unwrap_or_default()
    }

    /// The transitions of `series`, in `settled_ts` order, each with the evaluations it
    /// references: one the store no longer holds reads as [`Evidence::Expired`] (D13).
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn series_transition_evidence(&self, series: &SeriesKey) -> Vec<TransitionEvidence> {
        self.series_transitions(series)
            .into_iter()
            .map(|transition| {
                let evaluations = transition
                    .evaluations
                    .iter()
                    .map(|id| {
                        self.evaluation(id).map_or_else(
                            || Evidence::Expired(id.clone()),
                            |held| Evidence::Held(Box::new(held)),
                        )
                    })
                    .collect();
                TransitionEvidence {
                    transition,
                    evaluations,
                }
            })
            .collect()
    }
}

/// The store, with the definitions `RecordTransition` reads, as the generated ports.
struct Ports<'a, D: ?Sized> {
    store: &'a mut EventlogStore,
    definitions: &'a D,
    failure: Option<StoreError>,
}

impl<D: SignalDefinitionStorage + ?Sized> SignalDefinitionStorage for Ports<'_, D> {
    fn get(&self, identity: &SignalId) -> Option<SignalDefinitionSnapshot> {
        self.definitions.get(identity)
    }

    /// `RecordTransition` writes no definition.
    fn put(&mut self, snapshot: SignalDefinitionSnapshot) {
        panic!(
            "RecordTransition does not write the definition {}",
            snapshot.data.signal_id.0
        );
    }

    /// `RecordTransition` removes no definition.
    fn delete(&mut self, identity: &SignalId) {
        panic!(
            "RecordTransition does not remove the definition {}",
            identity.0
        );
    }

    fn list(&self) -> Vec<SignalDefinitionSnapshot> {
        self.definitions.list()
    }
}

impl<D: ?Sized> TransitionStorage for Ports<'_, D> {
    fn get(&self, identity: &TransitionId) -> Option<TransitionSnapshot> {
        TransitionStorage::get(&*self.store, identity)
    }

    fn put(&mut self, snapshot: TransitionSnapshot) {
        if let Err(error) = self.store.append_transition(&snapshot.data) {
            self.failure = Some(error);
        }
    }

    fn delete(&mut self, identity: &TransitionId) {
        TransitionStorage::delete(self.store, identity);
    }

    fn list(&self) -> Vec<TransitionSnapshot> {
        TransitionStorage::list(&*self.store)
    }
}

impl<D: ?Sized> IdSequence for Ports<'_, D> {
    fn next_number(&mut self, prefix: &'static str) -> Option<u64> {
        self.store.next_number(prefix)
    }
}

fn snapshot(data: TransitionData) -> TransitionSnapshot {
    TransitionSnapshot {
        state: TransitionState::Recorded,
        data,
    }
}

/// The generated port over the stream. Its methods return nothing, so a refusal or a store
/// failure while storing panics with the reason rather than losing the transition;
/// [`EventlogStore::append_transition`] answers the same refusal as a value.
impl TransitionStorage for EventlogStore {
    fn get(&self, identity: &TransitionId) -> Option<TransitionSnapshot> {
        self.transition(identity).map(snapshot)
    }

    fn put(&mut self, snapshot: TransitionSnapshot) {
        if let Err(error) = self.append_transition(&snapshot.data) {
            panic!(
                "transition {} was not stored: {error}",
                snapshot.data.transition_id.0
            );
        }
    }

    /// The stream is append-only: a held transition is never removed.
    fn delete(&mut self, identity: &TransitionId) {
        assert!(
            self.transition(identity).is_none(),
            "transition {} is on an append-only stream and is not deleted",
            identity.0
        );
    }

    fn list(&self) -> Vec<TransitionSnapshot> {
        self.transitions().into_iter().map(snapshot).collect()
    }
}

fn encode_decimal(value: &Decimal, out: &mut String) {
    json::push_text(out, &value.0);
}

fn decode_decimal(value: &json::Value, at: &str) -> Result<Decimal, json::DecodeError> {
    json::decimal_at(value, at, "a decimal string").map(|text| Decimal(text.to_owned()))
}

fn decode_integer(value: &json::Value, at: &str) -> Result<i64, json::DecodeError> {
    json::integer_at(value, at, "an integer")
}

/// The `thalamus.transition/1` record of `transition`. An absent optional is an absent member.
pub(crate) fn encode(transition: &TransitionData) -> Result<Value, StoreError> {
    let mut body = Map::new();
    body.insert("schema".to_owned(), Value::from(TRANSITION_SCHEMA));
    body.insert(
        "transition_id".to_owned(),
        part(
            &transition.transition_id,
            wire::encode_thalamus_transition_transition_id,
        )?,
    );
    body.insert(
        "signal".to_owned(),
        part(
            &transition.signal,
            wire::encode_thalamus_definition_signal_id,
        )?,
    );
    body.insert(
        "series".to_owned(),
        part(
            &transition.series,
            wire::encode_thalamus_observation_series_key,
        )?,
    );
    body.insert("dims".to_owned(), dims_value(&transition.dims));
    body.insert(
        "definition_rev".to_owned(),
        Value::from(transition.definition_rev),
    );
    if let Some(from) = &transition.from_verdict {
        body.insert(
            "from_verdict".to_owned(),
            part(from, wire::encode_thalamus_evaluation_verdict)?,
        );
    }
    body.insert(
        "to_verdict".to_owned(),
        part(
            &transition.to_verdict,
            wire::encode_thalamus_evaluation_verdict,
        )?,
    );
    body.insert(
        "first_ts".to_owned(),
        Value::from(transition.first_ts.0.as_str()),
    );
    body.insert(
        "settled_ts".to_owned(),
        Value::from(transition.settled_ts.0.as_str()),
    );
    if let Some(score) = &transition.settling_score {
        body.insert("settling_score".to_owned(), part(score, encode_decimal)?);
    }
    let evaluations = transition
        .evaluations
        .iter()
        .map(|id| part(id, wire::encode_thalamus_evaluation_evaluation_id))
        .collect::<Result<Vec<_>, _>>()?;
    body.insert("evaluations".to_owned(), Value::Array(evaluations));
    Ok(Value::Object(body))
}

/// The transition a `thalamus.transition/1` record holds.
pub(crate) fn decode(body: &Value) -> Result<TransitionData, StoreError> {
    let Some(Value::Array(evaluations)) = body.get("evaluations") else {
        return Err(StoreError::Record(
            "the record has no `evaluations` list".to_owned(),
        ));
    };
    let evaluations = evaluations
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let parsed = json::parse(&id.to_string())
                .map_err(|error| StoreError::Record(format!("`evaluations/{index}`: {error}")))?;
            wire::decode_thalamus_evaluation_evaluation_id(&parsed, "evaluations")
                .map_err(|error| StoreError::Record(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TransitionData {
        transition_id: unpart(
            body,
            "transition_id",
            wire::decode_thalamus_transition_transition_id,
        )?,
        signal: unpart(body, "signal", wire::decode_thalamus_definition_signal_id)?,
        series: unpart(body, "series", wire::decode_thalamus_observation_series_key)?,
        dims: dims(body)?,
        definition_rev: unpart(body, "definition_rev", decode_integer)?,
        from_verdict: unpart_optional(
            body,
            "from_verdict",
            wire::decode_thalamus_evaluation_verdict,
        )?,
        to_verdict: unpart(body, "to_verdict", wire::decode_thalamus_evaluation_verdict)?,
        first_ts: Timestamp(text(body, "first_ts")?),
        settled_ts: Timestamp(text(body, "settled_ts")?),
        settling_score: unpart_optional(body, "settling_score", decode_decimal)?,
        evaluations,
    })
}
