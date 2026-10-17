//! Settled verdict changes (design § 3.2, § 11 D6, D13): one
//! [`SeriesHysteresis`](thalamus_core::hysteresis::SeriesHysteresis) per series observes each new
//! evaluation, and a change it settles is recorded as a `Transition` through the generated
//! `RecordTransition`, referencing the k evaluations of its run, and emitted as a CloudEvents
//! event ([`crate::events`]) in the tick.
//!
//! On start each series is seeded from the store, through its series range read: resumed from
//! the `to_verdict` of its newest recorded transition and fed its stored evaluations after that
//! transition's `settled_ts`, or, with no transition, fed every stored evaluation. Seeding records
//! nothing. Evaluations are fed in minting order (evaluation id order), as the live service took
//! them, so a restart neither loses a run in progress nor records a change twice.
//!
//! The persistence of a series is its definition's (`thalamus_core::hysteresis::Persistence::of`);
//! a revision that changes it applies from the next evaluation of the series on.

use std::collections::{BTreeMap, BTreeSet};

use thalamus_core::behaviour::observation::Instant;
use thalamus_core::hysteresis::{Persistence, SeriesHysteresis, SettledChange};
use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::SignalEvaluationData;
use thalamus_model::observation::SeriesKey;
use thalamus_model::primitives::Timestamp;
use thalamus_model::transition::RecordTransition;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_store::eventlog::EventlogStore;

use crate::scheduler::{PRODUCER, Tick};

/// The start of the range a series is seeded from: before any sample thalamus records.
const BEGINNING: &str = "1970-01-01T00:00:00Z";
/// The end of that range: after any sample thalamus records.
const END: &str = "9999-12-31T23:59:59Z";

/// The hysteresis of every series the store holds evaluations of, and how far into the store's
/// evaluations it has read.
#[derive(Debug, Clone, Default)]
pub struct Transitions {
    series: BTreeMap<String, Tracked>,
    /// How many of the store handle's evaluations have been observed or seeded.
    cursor: usize,
}

#[derive(Debug, Clone)]
struct Tracked {
    hysteresis: SeriesHysteresis,
    persistence: Persistence,
}

impl Transitions {
    /// Every series of `store` seeded from its stored evaluations and its newest transition.
    ///
    /// # Panics
    ///
    /// When the store cannot be read: its reads answer no error.
    #[must_use]
    pub fn seed(store: &EventlogStore, instance: &LoadedInstance) -> Self {
        let taken = store.evaluations();
        let held: BTreeSet<&str> = taken
            .iter()
            .map(|evaluation| evaluation.evaluation_id.0.as_str())
            .collect();
        let mut keys: BTreeMap<&str, &SignalId> = BTreeMap::new();
        for evaluation in taken {
            keys.entry(evaluation.series.0.as_str())
                .or_insert(&evaluation.signal);
        }
        let mut series = BTreeMap::new();
        for (key, signal) in keys {
            let persistence = persistence(instance, signal);
            let key = SeriesKey(key.to_owned());
            let newest = store.series_transitions(&key).pop();
            let (mut hysteresis, after) = match &newest {
                Some(transition) => (
                    SeriesHysteresis::resume(persistence, transition.to_verdict),
                    Some(transition.settled_ts.clone()),
                ),
                None => (SeriesHysteresis::new(persistence), None),
            };
            let from = after.clone().unwrap_or(Timestamp(BEGINNING.to_owned()));
            let after = after.as_ref().and_then(|ts| Instant::parse(&ts.0));
            let mut evaluations: Vec<SignalEvaluationData> = store
                .series_range(&key, &from, &Timestamp(END.to_owned()))
                .evaluations
                .into_iter()
                .filter(|evaluation| held.contains(evaluation.evaluation_id.0.as_str()))
                .filter(|evaluation| {
                    after.is_none_or(|after| {
                        Instant::parse(&evaluation.ts.0).is_some_and(|at| at > after)
                    })
                })
                .collect();
            evaluations.sort_by(|a, b| a.evaluation_id.0.cmp(&b.evaluation_id.0));
            for evaluation in &evaluations {
                let _seeded = hysteresis.observe(evaluation);
            }
            series.insert(
                key.0,
                Tracked {
                    hysteresis,
                    persistence,
                },
            );
        }
        Self {
            series,
            cursor: taken.len(),
        }
    }
}

/// Observes every evaluation the store handle took in since the last call: each settled change is
/// recorded as a transition and its CloudEvents event pushed onto `tick.events`; a change that
/// could not be recorded is a problem of the tick.
///
/// # Panics
///
/// When the store cannot be read: its reads answer no error.
pub fn observe(
    transitions: &mut Transitions,
    store: &mut EventlogStore,
    instance: &LoadedInstance,
    tick: &mut Tick,
) {
    let fresh: Vec<SignalEvaluationData> = store
        .evaluations()
        .get(transitions.cursor..)
        .unwrap_or_default()
        .to_vec();
    transitions.cursor = store.evaluations().len();
    for evaluation in fresh {
        let persistence = persistence(instance, &evaluation.signal);
        let tracked = transitions
            .series
            .entry(evaluation.series.0.clone())
            .or_insert_with(|| Tracked {
                hysteresis: SeriesHysteresis::new(persistence),
                persistence,
            });
        let mut changes = Vec::new();
        if tracked.persistence != persistence {
            tracked.persistence = persistence;
            changes.extend(tracked.hysteresis.set_persistence(persistence));
        }
        changes.extend(tracked.hysteresis.observe(&evaluation));
        for change in changes {
            record(store, instance, &evaluation, change, tick);
        }
    }
}

/// Records `change` of `evaluation`'s series and emits it.
fn record(
    store: &mut EventlogStore,
    instance: &LoadedInstance,
    evaluation: &SignalEvaluationData,
    change: SettledChange,
    tick: &mut Tick,
) {
    let input = RecordTransition {
        signal: evaluation.signal.clone(),
        series: evaluation.series.clone(),
        dims: evaluation.dims.clone(),
        definition_rev: change.definition_rev,
        from_verdict: change.from,
        to_verdict: change.to,
        first_ts: change.first_ts,
        settled_ts: change.settled_ts,
        settling_score: change.settling_score,
        evaluations: change.evaluations,
    };
    match store.record_transition(instance.instance(), input) {
        Ok(transition) => tick.events.push(crate::events::transition(
            instance.name(),
            PRODUCER,
            &transition,
        )),
        Err(error) => tick.problems.push(format!(
            "{}: the settled change to {} was not recorded: {error}",
            evaluation.series.0,
            thalamus_pipeline::evaluate::verdict_name(change.to)
        )),
    }
}

/// The persistence of `signal`'s definition; the defaults when the instance has none.
fn persistence(instance: &LoadedInstance, signal: &SignalId) -> Persistence {
    instance
        .instance()
        .definition(&signal.0)
        .map_or(Persistence::DEFAULT, |definition| {
            Persistence::of(definition.data())
        })
}
