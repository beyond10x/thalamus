//! The evaluation port: evaluations on the instance's Eventlog stream as `thalamus.evaluation/1`
//! records, and the generated `SignalEvaluationStorage` over them.
//!
//! D5 (design § 11): at most one evaluation per `(sample_id, definition_rev)`; a re-grade after a
//! revision is a new evaluation. The rule is held inside the append: an inline projection keeps
//! one row per pair, and a guard reading that row in the append transaction refuses a second
//! evaluation with [`DUPLICATE_EVALUATION`], so nothing is written. The sample is stored first:
//! an evaluation of a sample the stream does not hold is refused with [`UNKNOWN_SAMPLE`].

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use eventlog_core::{
    BoxFuture, EventLogError, Guard, ProjectionSpec, ProjectionStore, Projector, RecordedEvent,
    TenantId,
};
use serde_json::{Map, Value};
use thalamus_model::behaviour::SignalEvaluationStorage;
use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::{
    EvaluationId, SignalEvaluationData, SignalEvaluationSnapshot, SignalEvaluationState,
};
use thalamus_model::json;
use thalamus_model::observation::SignalSampleData;
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::server::wire;

use crate::eventlog::{
    EVALUATION_SCHEMA, EventlogStore, ID_TAKEN, StoreError, dims, dims_value, part, text, unpart,
    unpart_optional,
};
use thalamus_core::behaviour::observation::timestamp_order;

/// The refusal code of a second evaluation for one `(sample_id, definition_rev)` (D5).
pub const DUPLICATE_EVALUATION: &str = "thalamus.evaluation.duplicate";
/// The refusal code of an evaluation whose sample the stream does not hold.
pub const UNKNOWN_SAMPLE: &str = "thalamus.evaluation.unknown-sample";

/// The event name an evaluation is appended under.
const EVENT: &str = "thalamus.evaluation";

/// One row per `(sample_id, definition_rev)` an evaluation was stored for.
const KEYS: ProjectionSpec = ProjectionSpec {
    name: "thalamus_evaluation_keys",
    indexed: &[],
};
const KEY_PROJECTIONS: &[ProjectionSpec] = &[KEYS];

/// The inline projections the stream is opened with.
pub(crate) fn projectors() -> Vec<Arc<dyn Projector>> {
    vec![Arc::new(EvaluationKeys)]
}

/// The projection row key of one `(sample_id, definition_rev)`.
fn key(sample_id: &str, definition_rev: i64) -> String {
    format!("{sample_id}#{definition_rev}")
}

/// Keeps a [`KEYS`] row for every stored evaluation, in the append transaction.
struct EvaluationKeys;

impl Projector for EvaluationKeys {
    fn name(&self) -> &'static str {
        KEYS.name
    }

    fn projections(&self) -> &'static [ProjectionSpec] {
        KEY_PROJECTIONS
    }

    fn apply<'a>(
        &'a self,
        event: &'a RecordedEvent,
        store: &'a mut dyn ProjectionStore,
    ) -> BoxFuture<'a, Result<(), EventLogError>> {
        Box::pin(async move {
            if event.name != EVENT || event.is_redacted() {
                return Ok(());
            }
            let sample_id = event.data.get("sample_id").and_then(Value::as_str);
            let rev = event.data.get("definition_rev").and_then(Value::as_i64);
            let (Some(sample_id), Some(rev)) = (sample_id, rev) else {
                return Err(EventLogError::Invalid(format!(
                    "evaluation event {} names no sample_id and definition_rev",
                    event.event_id
                )));
            };
            let row = serde_json::json!({ "evaluation_id": event.data.get("evaluation_id") });
            store
                .upsert(&KEYS, &event.tenant, &key(sample_id, rev), &row)
                .await
        })
    }
}

/// Refuses the append when a [`KEYS`] row for its pair is already held.
struct OncePerRevision {
    tenant: TenantId,
    key: String,
}

impl Guard for OncePerRevision {
    fn check<'a>(
        &'a self,
        store: &'a mut dyn ProjectionStore,
    ) -> BoxFuture<'a, Result<(), EventLogError>> {
        Box::pin(async move {
            match store.get_for_update(&KEYS, &self.tenant, &self.key).await? {
                Some(_) => Err(EventLogError::GuardRefused {
                    code: DUPLICATE_EVALUATION.to_owned(),
                }),
                None => Ok(()),
            }
        })
    }
}

impl EventlogStore {
    /// Appends `evaluation` to the stream.
    ///
    /// # Errors
    ///
    /// [`StoreError::Refused`] with [`ID_TAKEN`] when its id is held with another body, with
    /// [`UNKNOWN_SAMPLE`] when its sample is not stored, and with [`DUPLICATE_EVALUATION`] when an
    /// evaluation for its `(sample_id, definition_rev)` is (D5); [`StoreError::Record`] when it
    /// does not read back as itself from its wire form; or the Eventlog failure. Nothing is
    /// written on any error.
    ///
    /// An append that meets another writer's append takes its records in and is retried, up to
    /// [`crate::eventlog::CATCH_UP_ATTEMPTS`] times ([`StoreError::Contended`] after that); one
    /// that meets its id stored with another body takes the stream in and is refused
    /// [`ID_TAKEN`], from a stale handle too. A sample this
    /// handle does not hold is looked for on the stream before the refusal.
    pub fn append_evaluation(
        &mut self,
        evaluation: &SignalEvaluationData,
    ) -> Result<(), StoreError> {
        let body = encode(evaluation)?;
        if decode(&body)? != *evaluation {
            return Err(StoreError::Record(format!(
                "evaluation {} does not read back as itself from its wire form",
                evaluation.evaluation_id.0
            )));
        }
        self.append_caught_up(|store| store.try_append_evaluation(evaluation, body.clone()))
    }

    fn try_append_evaluation(
        &mut self,
        evaluation: &SignalEvaluationData,
        body: Value,
    ) -> Result<(), StoreError> {
        if let Some(held) = self.held_evaluation(&evaluation.evaluation_id) {
            return if held == evaluation {
                Ok(())
            } else {
                Err(StoreError::Refused {
                    code: ID_TAKEN.to_owned(),
                })
            };
        }
        if self.held_sample(&evaluation.sample_id).is_none() {
            self.refresh()?;
        }
        if self.held_sample(&evaluation.sample_id).is_none() {
            return Err(StoreError::Refused {
                code: UNKNOWN_SAMPLE.to_owned(),
            });
        }
        let guard = OncePerRevision {
            tenant: self.tenant().clone(),
            key: key(&evaluation.sample_id.0, evaluation.definition_rev),
        };
        self.append(
            EVENT,
            &evaluation.evaluation_id.0,
            body,
            Some(Arc::new(guard)),
        )
    }

    /// The evaluation this handle has taken in under `id`.
    fn held_evaluation(&self, id: &EvaluationId) -> Option<&SignalEvaluationData> {
        self.evaluation_index
            .get(&id.0)
            .and_then(|index| self.evaluations.get(*index))
    }

    /// The evaluation stored under `id`, also when another writer appended it since this handle
    /// last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn evaluation(&self, id: &EvaluationId) -> Option<SignalEvaluationData> {
        if let Some(held) = self.held_evaluation(id) {
            return Some(held.clone());
        }
        self.tail()
            .evaluations
            .into_iter()
            .find(|evaluation| evaluation.evaluation_id == *id)
    }

    /// Every evaluation this handle has taken in, in the order it was appended: the stream as of
    /// the handle's last call that took `&mut self`.
    pub fn evaluations(&self) -> &[SignalEvaluationData] {
        &self.evaluations
    }

    /// Every stored evaluation, in the order it was appended, with those other writers appended
    /// since this handle last moved.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn current_evaluations(&self) -> Vec<SignalEvaluationData> {
        let mut all = self.evaluations.clone();
        all.extend(self.tail().evaluations);
        all
    }

    /// The newest evaluation of each series, by series key: the one with the latest `ts` in the
    /// order the SeriesSamples view reads `ts` (`timestamp_order`), and of two at one instant the
    /// one appended last (a re-grade).
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn newest_evaluations(&self) -> Vec<SignalEvaluationData> {
        let mut newest: BTreeMap<String, SignalEvaluationData> = BTreeMap::new();
        for evaluation in self.current_evaluations() {
            let replaces = newest
                .get(&evaluation.series.0)
                .is_none_or(|held| timestamp_order(&evaluation.ts.0, &held.ts.0) != Ordering::Less);
            if replaces {
                newest.insert(evaluation.series.0.clone(), evaluation);
            }
        }
        newest.into_values().collect()
    }

    /// The samples of `signal` with no evaluation at `definition_rev`, in the order they were
    /// appended: what a re-grade after a revision still has to evaluate.
    ///
    /// # Panics
    ///
    /// When the stream cannot be read ([`EventlogStore::tail`]).
    pub fn unevaluated_samples(
        &self,
        signal: &SignalId,
        definition_rev: i64,
    ) -> Vec<SignalSampleData> {
        let evaluated: HashSet<String> = self
            .current_evaluations()
            .into_iter()
            .filter(|evaluation| evaluation.definition_rev == definition_rev)
            .map(|evaluation| evaluation.sample_id.0)
            .collect();
        self.current_samples()
            .into_iter()
            .filter(|sample| sample.signal == *signal && !evaluated.contains(&sample.sample_id.0))
            .collect()
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

/// The `thalamus.evaluation/1` record of `evaluation`. An absent optional is an absent member.
pub(crate) fn encode(evaluation: &SignalEvaluationData) -> Result<Value, StoreError> {
    let mut body = Map::new();
    body.insert("schema".to_owned(), Value::from(EVALUATION_SCHEMA));
    body.insert(
        "evaluation_id".to_owned(),
        part(
            &evaluation.evaluation_id,
            wire::encode_thalamus_evaluation_evaluation_id,
        )?,
    );
    body.insert(
        "sample_id".to_owned(),
        part(
            &evaluation.sample_id,
            wire::encode_thalamus_observation_sample_id,
        )?,
    );
    body.insert(
        "signal".to_owned(),
        part(
            &evaluation.signal,
            wire::encode_thalamus_definition_signal_id,
        )?,
    );
    body.insert(
        "series".to_owned(),
        part(
            &evaluation.series,
            wire::encode_thalamus_observation_series_key,
        )?,
    );
    body.insert("dims".to_owned(), dims_value(&evaluation.dims));
    body.insert("ts".to_owned(), Value::from(evaluation.ts.0.as_str()));
    body.insert(
        "definition_rev".to_owned(),
        Value::from(evaluation.definition_rev),
    );
    body.insert(
        "verdict".to_owned(),
        part(
            &evaluation.verdict,
            wire::encode_thalamus_evaluation_verdict,
        )?,
    );
    if let Some(prev) = &evaluation.prev {
        body.insert(
            "prev".to_owned(),
            part(prev, wire::encode_thalamus_evaluation_verdict)?,
        );
    }
    if let Some(score) = &evaluation.score {
        body.insert("score".to_owned(), part(score, encode_decimal)?);
    }
    if let Some(baseline) = &evaluation.baseline {
        body.insert(
            "baseline".to_owned(),
            part(baseline, wire::encode_thalamus_evaluation_baseline)?,
        );
    }
    if let Some(reason) = &evaluation.reason {
        body.insert("reason".to_owned(), Value::from(reason.as_str()));
    }
    Ok(Value::Object(body))
}

/// The evaluation a `thalamus.evaluation/1` record holds.
pub(crate) fn decode(body: &Value) -> Result<SignalEvaluationData, StoreError> {
    Ok(SignalEvaluationData {
        evaluation_id: unpart(
            body,
            "evaluation_id",
            wire::decode_thalamus_evaluation_evaluation_id,
        )?,
        sample_id: unpart(
            body,
            "sample_id",
            wire::decode_thalamus_observation_sample_id,
        )?,
        signal: unpart(body, "signal", wire::decode_thalamus_definition_signal_id)?,
        series: unpart(body, "series", wire::decode_thalamus_observation_series_key)?,
        dims: dims(body)?,
        ts: Timestamp(text(body, "ts")?),
        definition_rev: unpart(body, "definition_rev", decode_integer)?,
        verdict: unpart(body, "verdict", wire::decode_thalamus_evaluation_verdict)?,
        prev: unpart_optional(body, "prev", wire::decode_thalamus_evaluation_verdict)?,
        score: unpart_optional(body, "score", decode_decimal)?,
        baseline: unpart_optional(body, "baseline", wire::decode_thalamus_evaluation_baseline)?,
        reason: match body.get("reason") {
            None => None,
            Some(_) => Some(text(body, "reason")?),
        },
    })
}

fn snapshot(data: SignalEvaluationData) -> SignalEvaluationSnapshot {
    SignalEvaluationSnapshot {
        state: SignalEvaluationState::Recorded,
        data,
    }
}

/// The generated port over the stream. Its methods return nothing, so a refusal or a store
/// failure while storing panics with the reason rather than losing or doubling the evaluation;
/// [`EventlogStore::append_evaluation`] answers the same refusal as a value. `RecordEvaluation`
/// declares no error a D5 refusal could answer (`spec/domains/evaluation.yaml`, the `UNMAPPED:`
/// marker before `recorded`), so the behaviour cannot refuse a second evaluation itself; a caller
/// that may grade one sample twice at one revision checks first.
impl SignalEvaluationStorage for EventlogStore {
    fn get(&self, identity: &EvaluationId) -> Option<SignalEvaluationSnapshot> {
        self.evaluation(identity).map(snapshot)
    }

    fn put(&mut self, snapshot: SignalEvaluationSnapshot) {
        if let Err(error) = self.append_evaluation(&snapshot.data) {
            panic!(
                "evaluation {} was not stored: {error}",
                snapshot.data.evaluation_id.0
            );
        }
    }

    /// The stream is append-only: a held evaluation is never removed (retention is M2).
    fn delete(&mut self, identity: &EvaluationId) {
        assert!(
            self.evaluation(identity).is_none(),
            "evaluation {} is on an append-only stream and is not deleted",
            identity.0
        );
    }

    fn list(&self) -> Vec<SignalEvaluationSnapshot> {
        self.current_evaluations()
            .into_iter()
            .map(snapshot)
            .collect()
    }
}
