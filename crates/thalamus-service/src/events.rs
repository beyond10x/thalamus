//! The CloudEvents emitter (design § 6.2): a recorded transition as one CloudEvents 1.0 event in
//! structured mode, a JSON object on one line. `thalamus-service run` writes each one to standard
//! output as it is recorded. Stored records are never wrapped (the store keeps the bare record).
//!
//! | attribute | value |
//! |---|---|
//! | `type` | [`TRANSITION_TYPE`] |
//! | `source` | the instance and the producer, `<instance>/<producer>` |
//! | `subject` | the series key (design § 11 D10) |
//! | `time` | the settling evaluation's `ts` |
//! | `id` | the transition id |
//!
//! `data` is the transition: its id, signal, series, dims, definition revision, verdicts, first
//! and settled `ts`, settling score and evaluation ids, verdicts and the score in their wire form.
//! An absent `from_verdict` or `settling_score` is an absent member, never `null` and never 0.

use serde_json::{Map, Value};
use thalamus_model::evaluation::Verdict;
use thalamus_model::server::wire;
use thalamus_model::transition::TransitionData;

/// The CloudEvents `type` of a transition.
pub const TRANSITION_TYPE: &str = "dev.beyond10x.thalamus.transition";
/// The CloudEvents specification version the events follow.
pub const SPEC_VERSION: &str = "1.0";

/// The CloudEvents event of `transition`, recorded in `instance` by `producer`, as one line of
/// JSON.
#[must_use]
pub fn transition(instance: &str, producer: &str, transition: &TransitionData) -> String {
    let mut data = Map::new();
    data.insert(
        "transition_id".to_owned(),
        Value::from(transition.transition_id.0.as_str()),
    );
    data.insert(
        "signal".to_owned(),
        Value::from(transition.signal.0.as_str()),
    );
    data.insert(
        "series".to_owned(),
        Value::from(transition.series.0.as_str()),
    );
    data.insert(
        "dims".to_owned(),
        Value::Object(
            transition
                .dims
                .iter()
                .map(|(key, value)| (key.clone(), Value::from(value.0.as_str())))
                .collect(),
        ),
    );
    data.insert(
        "definition_rev".to_owned(),
        Value::from(transition.definition_rev),
    );
    if let Some(from) = transition.from_verdict {
        data.insert("from_verdict".to_owned(), verdict(from));
    }
    data.insert("to_verdict".to_owned(), verdict(transition.to_verdict));
    data.insert(
        "first_ts".to_owned(),
        Value::from(transition.first_ts.0.as_str()),
    );
    data.insert(
        "settled_ts".to_owned(),
        Value::from(transition.settled_ts.0.as_str()),
    );
    if let Some(score) = &transition.settling_score {
        data.insert("settling_score".to_owned(), Value::from(score.0.as_str()));
    }
    data.insert(
        "evaluations".to_owned(),
        Value::Array(
            transition
                .evaluations
                .iter()
                .map(|id| Value::from(id.0.as_str()))
                .collect(),
        ),
    );
    let mut event = Map::new();
    event.insert("specversion".to_owned(), Value::from(SPEC_VERSION));
    event.insert(
        "id".to_owned(),
        Value::from(transition.transition_id.0.as_str()),
    );
    event.insert(
        "source".to_owned(),
        Value::from(format!("{instance}/{producer}")),
    );
    event.insert("type".to_owned(), Value::from(TRANSITION_TYPE));
    event.insert(
        "subject".to_owned(),
        Value::from(transition.series.0.as_str()),
    );
    event.insert(
        "time".to_owned(),
        Value::from(transition.settled_ts.0.as_str()),
    );
    event.insert(
        "datacontenttype".to_owned(),
        Value::from("application/json"),
    );
    event.insert("data".to_owned(), Value::Object(data));
    Value::Object(event).to_string()
}

/// The wire name of `verdict` (`N/A` for not applicable).
fn verdict(verdict: Verdict) -> Value {
    let mut text = String::new();
    wire::encode_thalamus_evaluation_verdict(&verdict, &mut text);
    serde_json::from_str(&text).unwrap_or(Value::Null)
}
