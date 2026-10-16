//! The grade → evaluate half of the pass (design § 2.3, § 3.1): every stored sample with no
//! evaluation at its definition's current `rev` is graded once and recorded as one evaluation.
//!
//! Per sample, in `ts` order within each definition:
//!
//! 1. the baseline windows come from `thalamus_core::baseline::plan`. Under `history: source`
//!    each window is read from the source through the extractor registry at the shifted window
//!    end; under `history: store` the series' own stored sample at the offset is read through the
//!    store's series range read; `fixed` reads nothing;
//! 2. the series age is `denominator.age_s` when the sample carries one, else the time since the
//!    first stored sample of the series (by instant);
//! 3. `baseline::fold`, then `grading::grade`; `prev` is the verdict of the newest evaluation of
//!    the series whose `ts` is earlier than the sample's (design § 11 D6: the previous raw
//!    verdict);
//! 4. the result is recorded through the generated `RecordEvaluation` over the store, under an
//!    `evl_` id the store mints.
//!
//! Design § 11 D5: at most one evaluation per `(sample, definition_rev)`. Only the samples
//! `EventlogStore::unevaluated_samples` returns are graded, and the evaluation is appended through
//! `EventlogStore::append_evaluation`, so a second pass records nothing and a duplicate another
//! writer appended first is skipped, not a failure and not a panic.

use std::collections::BTreeMap;
use std::fmt;

use thalamus_core::baseline::{self, Offset, OffsetReading, WindowRequest};
use thalamus_core::behaviour::evaluation::record_evaluation;
use thalamus_core::behaviour::observation::{
    IdSequence, Instant, in_time_order, series_samples, timestamp_order,
};
use thalamus_core::grading;
use thalamus_core::registry::Definition;
use thalamus_core::series::series_key;
use thalamus_model::behaviour::{SignalEvaluationStorage, SignalSampleStorage};
use thalamus_model::definition::{History, SignalDefinitionData, SignalDefinitionState, SignalId};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{
    EvaluationId, RecordEvaluation, RecordEvaluationOutcome, SignalEvaluationSnapshot, Verdict,
};
use thalamus_model::observation::{
    Reading, RunId, SampleId, SeriesKey, SignalSampleData, SignalSampleSnapshot, UnreadReading,
};
use thalamus_model::primitives::Timestamp;
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;
use thalamus_store::evaluations::DUPLICATE_EVALUATION;
use thalamus_store::eventlog::{EventlogStore, StoreError};

use crate::instance::LoadedInstance;

/// The verdict as the ladder and the wire name it: `OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`.
#[must_use]
pub fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::OK => "OK",
        Verdict::NA => "N/A",
        Verdict::WARN => "WARN",
        Verdict::CRIT => "CRIT",
        Verdict::UNKNOWN => "UNKNOWN",
    }
}

/// Why a definition or a sample was not graded. Nothing is recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No definition has the id.
    UnknownSignal {
        /// The id asked for.
        signal: String,
    },
    /// The definition is neither `Active` nor `Muted`, the states whose samples are graded.
    NotGraded {
        /// The definition.
        signal: String,
        /// Its state.
        state: SignalDefinitionState,
    },
    /// The definition's baseline cannot be planned or folded for the sample.
    Baseline {
        /// The definition.
        signal: String,
        /// The sample.
        sample: String,
        /// Why.
        refusal: baseline::Refusal,
    },
    /// The definition's rule cannot grade any sample.
    Grading {
        /// The definition.
        signal: String,
        /// Why.
        refusal: grading::Refusal,
    },
    /// `RecordEvaluation` refused the graded result.
    Unrecorded {
        /// The sample.
        sample: String,
        /// The outcome's name.
        outcome: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSignal { signal } => write!(
                f,
                "{signal}: unknown-signal: no definition of the instance has this id; nothing \
                 was recorded"
            ),
            Self::NotGraded { signal, state } => write!(
                f,
                "{signal}: not-active: the definition is {state:?}, and only the samples of an \
                 Active or Muted definition are graded; nothing was recorded"
            ),
            Self::Baseline {
                signal,
                sample,
                refusal,
            } => write!(
                f,
                "{signal}: sample {sample}: baseline refused: {refusal}; nothing was recorded"
            ),
            Self::Grading { signal, refusal } => write!(
                f,
                "{signal}: grading rule refused: {refusal}; none of its samples was graded"
            ),
            Self::Unrecorded { sample, outcome } => write!(
                f,
                "sample {sample}: RecordEvaluation answered {outcome}; nothing was recorded"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// Why a pass could not go on.
#[derive(Debug)]
pub enum PassError {
    /// The store's `evl_` id sequence is spent.
    IdsSpent,
    /// The store could not append an evaluation.
    Store(StoreError),
}

impl fmt::Display for PassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdsSpent => f.write_str("the store's identity sequence of evl_ is spent"),
            Self::Store(error) => write!(f, "the store did not append an evaluation: {error}"),
        }
    }
}

impl std::error::Error for PassError {}

/// One recorded evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// Its id, minted by the store.
    pub evaluation_id: EvaluationId,
    /// The sample graded.
    pub sample_id: SampleId,
    /// The definition.
    pub signal: SignalId,
    /// The series.
    pub series: SeriesKey,
    /// The sample's `ts`.
    pub ts: Timestamp,
    /// The definition revision graded at.
    pub definition_rev: i64,
    /// The verdict.
    pub verdict: Verdict,
    /// The previous raw verdict of the series.
    pub prev: Option<Verdict>,
}

/// What one pass recorded and refused.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pass {
    /// The evaluations recorded, in the order recorded.
    pub recorded: Vec<Recorded>,
    /// The definitions and samples refused.
    pub refused: Vec<Refusal>,
}

/// The definition `signal` names, when its samples may be graded: `Active` or `Muted`.
///
/// # Errors
///
/// [`Refusal::UnknownSignal`] or [`Refusal::NotGraded`].
pub fn admit(instance: &LoadedInstance, signal: &str) -> Result<SignalId, Refusal> {
    let Some(definition) = instance.instance().definition(signal) else {
        return Err(Refusal::UnknownSignal {
            signal: signal.to_owned(),
        });
    };
    match definition.state() {
        SignalDefinitionState::Active | SignalDefinitionState::Muted => {
            Ok(definition.data().signal_id.clone())
        }
        state => Err(Refusal::NotGraded {
            signal: signal.to_owned(),
            state,
        }),
    }
}

/// Every definition of the instance whose samples are graded (`Active` or `Muted`), in id order.
#[must_use]
pub fn gradable(instance: &LoadedInstance) -> Vec<SignalId> {
    instance
        .instance()
        .definitions()
        .filter(|definition| {
            matches!(
                definition.state(),
                SignalDefinitionState::Active | SignalDefinitionState::Muted
            )
        })
        .map(|definition| definition.data().signal_id.clone())
        .collect()
}

/// One pass: [`evaluate_into`] a fresh [`Pass`].
///
/// # Errors
///
/// [`PassError`] when the store mints no id or appends no evaluation.
///
/// # Panics
///
/// When the store cannot be read: its reads answer no error.
pub fn run(
    instance: &LoadedInstance,
    store: &mut EventlogStore,
    connectors: &Connectors,
    signals: &[SignalId],
    run: Option<&RunId>,
) -> Result<Pass, PassError> {
    let mut pass = Pass::default();
    evaluate_into(&mut pass, instance, store, connectors, signals, run)?;
    Ok(pass)
}

/// Grades every stored sample of each of `signals` with no evaluation at the definition's
/// current `rev` (only those of `run`, when given), and records each result into `pass` as soon
/// as it is stored, so a store that fails in the middle leaves `pass` holding everything recorded
/// before it failed.
///
/// # Errors
///
/// [`PassError`] when the store mints no id or appends no evaluation.
///
/// # Panics
///
/// When the store cannot be read: its reads answer no error.
pub fn evaluate_into(
    pass: &mut Pass,
    instance: &LoadedInstance,
    store: &mut EventlogStore,
    connectors: &Connectors,
    signals: &[SignalId],
    run: Option<&RunId>,
) -> Result<(), PassError> {
    for signal in signals {
        let Some(definition) = instance.instance().definition(&signal.0) else {
            pass.refused.push(Refusal::UnknownSignal {
                signal: signal.0.clone(),
            });
            continue;
        };
        evaluate_definition(pass, definition, store, connectors, run)?;
    }
    Ok(())
}

fn evaluate_definition(
    pass: &mut Pass,
    definition: &Definition,
    store: &mut EventlogStore,
    connectors: &Connectors,
    run: Option<&RunId>,
) -> Result<(), PassError> {
    let data = definition.data();
    let samples: Vec<SignalSampleData> = store
        .unevaluated_samples(&data.signal_id, data.rev)
        .into_iter()
        .filter(|sample| run.is_none_or(|run| sample.provenance.run_id == *run))
        .collect();
    let samples = in_time_order(samples, |sample| sample.ts.0.as_str());
    let mut source = SourceReads::default();
    for sample in samples {
        let requests = match baseline::plan(data, &sample) {
            Ok(requests) => requests,
            Err(refusal) => {
                let per_sample = matches!(refusal, baseline::Refusal::Timestamp { .. });
                pass.refused.push(Refusal::Baseline {
                    signal: data.signal_id.0.clone(),
                    sample: sample.sample_id.0.clone(),
                    refusal,
                });
                if per_sample {
                    continue;
                }
                return Ok(());
            }
        };
        let readings: Vec<OffsetReading> = requests
            .iter()
            .map(|request| match data.history {
                History::Source => source.read(connectors, data, &sample, request),
                History::Store => from_store(store, &sample, request),
            })
            .collect();
        let age = series_age(store, &sample);
        let folded = match baseline::fold(data, &sample, &readings, age) {
            Ok(folded) => folded,
            Err(refusal) => {
                pass.refused.push(Refusal::Baseline {
                    signal: data.signal_id.0.clone(),
                    sample: sample.sample_id.0.clone(),
                    refusal,
                });
                continue;
            }
        };
        let graded = match grading::grade(data, &sample, &folded) {
            Ok(graded) => graded,
            Err(refusal) => {
                pass.refused.push(Refusal::Grading {
                    signal: data.signal_id.0.clone(),
                    refusal,
                });
                return Ok(());
            }
        };
        let prev = previous_verdict(store, &sample);
        let input = RecordEvaluation {
            sample_id: sample.sample_id.clone(),
            definition_rev: data.rev,
            verdict: graded.verdict,
            prev,
            score: graded.score,
            baseline: Some(graded.baseline),
            reason: Some(graded.reason),
        };
        let mut ports = Ports {
            store,
            failure: None,
        };
        let outcome = record_evaluation(&mut ports, input);
        let failure = ports.failure.take();
        match (outcome, failure) {
            (Err(unmet), _) if unmet.capability == "identity sequence" => {
                return Err(PassError::IdsSpent);
            }
            (Err(unmet), _) => pass.refused.push(Refusal::Unrecorded {
                sample: sample.sample_id.0.clone(),
                outcome: format!("{}: {}", unmet.capability, unmet.source),
            }),
            // Another writer graded this sample at this revision first (D5): nothing to record.
            (Ok(_), Some(StoreError::Refused { code })) if code == DUPLICATE_EVALUATION => {}
            (Ok(_), Some(error)) => return Err(PassError::Store(error)),
            (Ok(RecordEvaluationOutcome::Recorded { response, .. }), None) => {
                pass.recorded.push(Recorded {
                    evaluation_id: response.evaluation_id,
                    sample_id: sample.sample_id.clone(),
                    signal: sample.signal.clone(),
                    series: sample.series.clone(),
                    ts: sample.ts.clone(),
                    definition_rev: data.rev,
                    verdict: graded.verdict,
                    prev,
                });
            }
            (Ok(other), None) => pass.refused.push(Refusal::Unrecorded {
                sample: sample.sample_id.0.clone(),
                outcome: outcome_name(&other).to_owned(),
            }),
        }
    }
    Ok(())
}

fn outcome_name(outcome: &RecordEvaluationOutcome) -> &'static str {
    match outcome {
        RecordEvaluationOutcome::Recorded { .. } => "recorded",
        RecordEvaluationOutcome::UnknownSample { .. } => "unknown-sample",
        RecordEvaluationOutcome::ReasonMissing { .. } => "reason-missing",
        RecordEvaluationOutcome::ScoreNotAdmitted { .. } => "score-not-admitted",
    }
}

/// The source reads of one definition in one pass, by window end: the samples of one sampling
/// pass share their `ts`, so each shifted window is read once for all of its series.
#[derive(Default)]
struct SourceReads {
    reads: BTreeMap<String, Vec<Read>>,
}

impl SourceReads {
    /// The reading of `sample`'s series in the window `request` names, read from the source at
    /// the shifted window end.
    fn read(
        &mut self,
        connectors: &Connectors,
        definition: &SignalDefinitionData,
        sample: &SignalSampleData,
        request: &WindowRequest,
    ) -> OffsetReading {
        let end = &request.window.end;
        let reads = self.reads.entry(end.0.clone()).or_insert_with(|| {
            registry::read(connectors, definition, end, &sample.provenance.run_id)
        });
        let mut matching = reads.iter().filter(|read| {
            let dims: BTreeMap<String, DimensionValue> = read
                .dims
                .iter()
                .map(|(key, value)| (key.clone(), DimensionValue(value.clone())))
                .collect();
            series_key(&definition.signal_id, &dims) == sample.series
        });
        let label = request.offset.label();
        match (matching.next(), matching.next()) {
            (Some(read), None) => OffsetReading {
                offset: request.offset,
                reading: read.reading.clone(),
                n: read.denominator.as_ref().map(|population| population.n),
            },
            (Some(_), Some(_)) => unknown(
                request.offset,
                format!(
                    "the source answered more than one read of series {} at {}",
                    sample.series.0, end.0
                ),
            ),
            (None, _) => {
                // A read that could not be made at all names no series; its reason is the one.
                let unread = reads.iter().find_map(|read| match &read.reading {
                    Reading::Unknown(UnreadReading { reason }) if read.dims.is_empty() => {
                        Some(reason.clone())
                    }
                    _ => None,
                });
                unknown(
                    request.offset,
                    unread.unwrap_or_else(|| {
                        format!(
                            "the source answered no read of series {} for the {label} window \
                             ending at {}",
                            sample.series.0, end.0
                        )
                    }),
                )
            }
        }
    }
}

/// The reading of `sample`'s series in the window `request` names, from the store: the newest
/// stored sample of the series whose `ts` falls in the shifted window, after its start and at or
/// before its end. A sample whose `ts` is the shifted window's start closes the slot before it
/// (its own window ends there), so it is never the baseline: a missing slot is unknown.
fn from_store(
    store: &EventlogStore,
    sample: &SignalSampleData,
    request: &WindowRequest,
) -> OffsetReading {
    let (start, end) = (&request.window.start, &request.window.end);
    let in_window = Instant::parse(&start.0)
        .zip(Instant::parse(&end.0))
        .and_then(|(after, end)| {
            store
                .series_range(&sample.series, start, &sample.ts)
                .samples
                .into_iter()
                .rfind(|row| Instant::parse(&row.ts.0).is_some_and(|at| after < at && at <= end))
        });
    match in_window {
        Some(row) => OffsetReading {
            offset: request.offset,
            reading: row.reading,
            n: row.denominator.map(|denominator| denominator.n),
        },
        None => unknown(
            request.offset,
            format!(
                "no stored sample of series {} in the {} window {} to {}",
                sample.series.0,
                request.offset.label(),
                start.0,
                end.0
            ),
        ),
    }
}

fn unknown(offset: Offset, reason: String) -> OffsetReading {
    OffsetReading {
        offset,
        reading: Reading::Unknown(UnreadReading { reason }),
        n: None,
    }
}

/// The age of the series at `sample`, in whole seconds: `denominator.age_s` when the sample
/// carries one, else the time from the earliest stored sample of the series (by instant) to the
/// sample, never below 0. `None` when neither can be told.
fn series_age(store: &EventlogStore, sample: &SignalSampleData) -> Option<i64> {
    if let Some(denominator) = &sample.denominator {
        return Some(denominator.age_s);
    }
    let at = Instant::parse(&sample.ts.0)?;
    let first = series_samples(store, &sample.series)
        .ok()?
        .into_iter()
        .find_map(|row| Instant::parse(&row.ts.0))?;
    Some((at.seconds() - first.seconds()).max(0))
}

/// The verdict of the newest evaluation of `sample`'s series whose `ts` is earlier than the
/// sample's; of two at one instant, the one appended last (design § 11 D6).
fn previous_verdict(store: &EventlogStore, sample: &SignalSampleData) -> Option<Verdict> {
    let earlier = store
        .current_evaluations()
        .into_iter()
        .filter(|evaluation| {
            evaluation.series == sample.series
                && timestamp_order(&evaluation.ts.0, &sample.ts.0).is_lt()
        })
        .collect();
    in_time_order(earlier, |evaluation| evaluation.ts.0.as_str())
        .pop()
        .map(|evaluation| evaluation.verdict)
}

/// The ports `RecordEvaluation` runs over: the store's samples, evaluations and id sequence. An
/// evaluation is appended through `EventlogStore::append_evaluation`, whose refusal is kept as a
/// value rather than the panic of the store's own port.
struct Ports<'a> {
    store: &'a mut EventlogStore,
    failure: Option<StoreError>,
}

impl SignalSampleStorage for Ports<'_> {
    fn get(&self, identity: &SampleId) -> Option<SignalSampleSnapshot> {
        SignalSampleStorage::get(&*self.store, identity)
    }

    fn put(&mut self, snapshot: SignalSampleSnapshot) {
        SignalSampleStorage::put(self.store, snapshot);
    }

    fn delete(&mut self, identity: &SampleId) {
        SignalSampleStorage::delete(self.store, identity);
    }

    fn list(&self) -> Vec<SignalSampleSnapshot> {
        SignalSampleStorage::list(&*self.store)
    }
}

impl SignalEvaluationStorage for Ports<'_> {
    fn get(&self, identity: &EvaluationId) -> Option<SignalEvaluationSnapshot> {
        SignalEvaluationStorage::get(&*self.store, identity)
    }

    fn put(&mut self, snapshot: SignalEvaluationSnapshot) {
        if let Err(error) = self.store.append_evaluation(&snapshot.data) {
            self.failure = Some(error);
        }
    }

    fn delete(&mut self, identity: &EvaluationId) {
        SignalEvaluationStorage::delete(self.store, identity);
    }

    fn list(&self) -> Vec<SignalEvaluationSnapshot> {
        SignalEvaluationStorage::list(&*self.store)
    }
}

impl IdSequence for Ports<'_> {
    fn next_number(&mut self, prefix: &'static str) -> Option<u64> {
        self.store.next_number(prefix)
    }
}
