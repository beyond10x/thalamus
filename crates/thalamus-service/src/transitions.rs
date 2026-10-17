//! Settled verdict changes (design § 3.2, § 11 D6, D13): one
//! [`SeriesHysteresis`](thalamus_core::hysteresis::SeriesHysteresis) per series observes each new
//! evaluation, and a change it settles is recorded as a `Transition` through the generated
//! `RecordTransition`, referencing the k evaluations of its run, and emitted as a CloudEvents
//! event ([`crate::events`]) in the tick.
//!
//! On start each series is seeded from the store, through its series range read: resumed from
//! the `to_verdict` of its newest recorded transition and fed its stored evaluations after that
//! transition's `settled_ts`, or, with no transition, fed every stored evaluation. The stored
//! evaluations up to that `settled_ts` fill the series' window and newest `ts` first, so after a
//! restart a re-grade of a sample in the window is a re-grade and one of an older sample is
//! dropped as late. A change the evaluations after the transition settle (stored while the
//! service was down, or not observed before a crash) is not lost: seeding keeps it, and the next
//! [`observe`] records and emits it. A change that could not be recorded is kept the same way and
//! tried again on the next call. Evaluations are fed in minting order (evaluation id order), as
//! the live service took them, so a restart neither loses a run in progress nor records a change
//! twice.
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
    /// Settled changes not recorded yet, oldest first: those seeding settled, and those whose
    /// record failed.
    pending: Vec<Pending>,
}

/// A settled change with the evaluation that settled it, which names its series.
#[derive(Debug, Clone)]
struct Pending {
    evaluation: SignalEvaluationData,
    change: SettledChange,
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
        let mut pending = Vec::new();
        for (key, signal) in keys {
            let persistence = persistence(instance, signal);
            let key = SeriesKey(key.to_owned());
            let newest = store.series_transitions(&key).pop();
            let mut hysteresis = SeriesHysteresis::new(persistence);
            let after = newest
                .as_ref()
                .map(|transition| transition.settled_ts.clone());
            let after = after.as_ref().and_then(|ts| Instant::parse(&ts.0));
            let mut stored: Vec<SignalEvaluationData> = store
                .series_range(
                    &key,
                    &Timestamp(BEGINNING.to_owned()),
                    &Timestamp(END.to_owned()),
                )
                .evaluations
                .into_iter()
                .filter(|evaluation| held.contains(evaluation.evaluation_id.0.as_str()))
                .collect();
            stored.sort_by(|a, b| a.evaluation_id.0.cmp(&b.evaluation_id.0));
            let is_after = |evaluation: &SignalEvaluationData| {
                after.is_none_or(|after| {
                    Instant::parse(&evaluation.ts.0).is_some_and(|at| at > after)
                })
            };
            if let Some(transition) = &newest {
                hysteresis = SeriesHysteresis::resume_from(
                    persistence,
                    transition.to_verdict,
                    stored.iter().filter(|evaluation| !is_after(evaluation)),
                );
            }
            for evaluation in stored.iter().filter(|evaluation| is_after(evaluation)) {
                pending.extend(hysteresis.observe(evaluation).map(|change| Pending {
                    evaluation: evaluation.clone(),
                    change,
                }));
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
            pending,
        }
    }
}

/// Observes every evaluation the store handle took in since the last call: each settled change,
/// those seeding settled and those a failed record kept included, is recorded as a transition and
/// its CloudEvents event pushed onto `tick.events`; a change that could not be recorded is a
/// problem of the tick and is tried again, with those after it, on the next call.
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
        transitions
            .pending
            .extend(changes.into_iter().map(|change| Pending {
                evaluation: evaluation.clone(),
                change,
            }));
    }
    let pending = std::mem::take(&mut transitions.pending);
    let mut kept = Vec::new();
    for item in pending {
        // Once one failed, the changes after it wait, so the series' transitions keep their order.
        if kept.is_empty() && record(store, instance, &item.evaluation, &item.change, tick) {
            continue;
        }
        kept.push(item);
    }
    transitions.pending = kept;
}

/// Records `change` of `evaluation`'s series and emits it; whether it was recorded.
fn record(
    store: &mut EventlogStore,
    instance: &LoadedInstance,
    evaluation: &SignalEvaluationData,
    change: &SettledChange,
    tick: &mut Tick,
) -> bool {
    let change = change.clone();
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
        Ok(transition) => {
            tick.events.push(crate::events::transition(
                instance.name(),
                PRODUCER,
                &transition,
            ));
            true
        }
        Err(error) => {
            tick.problems.push(format!(
                "{}: the settled change to {} was not recorded: {error}",
                evaluation.series.0,
                thalamus_pipeline::evaluate::verdict_name(change.to)
            ));
            false
        }
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
