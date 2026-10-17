//! The read → sample half of the pass (design § 2.2, § 5.3): one pass over a set of definitions
//! under one `run_` id, one recorded sample per series.
//!
//! Per read the extractor registry answers (`thalamus_sources::registry`): the `dims` are
//! admitted against the instance's registry (`thalamus_core::series::admit_dims`) and the series
//! key is spelled (D10). Reads that fall on one series key are merged into one `unknown` sample
//! naming the collision and the number of reads. Per series, `denominator.since` is resolved
//! (`thalamus_core::since::resolve_since`, D18) and `age_s` taken from it, and the generated
//! `RecordSample` behaviour (`thalamus_core::behaviour::observation::record_sample`) records the
//! sample over the file-backed definition ports and the store's sample port. The store mints the
//! `smp_` id, and the `run_` id of the pass, from its own id sequence (D24).
//!
//! `denominator.since` under `first_sample` is the earliest stored `ts` of the series by instant,
//! capped at the sample's own `ts`: the first sample of a series, or one sampled for a window
//! earlier than any stored, is its own origin. An origin the instance cannot give (a dimension
//! value whose registry entry carries no `created`) falls back the same way, the latest origin
//! the population could have, so `age_s` never overstates a series' age.
//!
//! An unknown read is recorded like any other: unknown is never dropped and never 0.
//!
//! The producer's value as printed, `text`, is recorded where
//! `thalamus_sources::vocabulary::sample_text` says the read carries one (a `producer_mapped`
//! definition whose value was read), so grading maps it at grade time. A merged collision sample
//! records none.

use std::collections::BTreeMap;
use std::fmt;

use thalamus_core::behaviour::observation::{
    IdSequence, Instant, record_sample, series_samples, timestamp_order,
};
use thalamus_core::registry::Definition;
use thalamus_core::series::{DimsRefusal, admit_dims, series_key};
use thalamus_core::since::resolve_since;
use thalamus_model::behaviour::{SignalDefinitionStorage, SignalSampleStorage};
use thalamus_model::definition::{SignalDefinitionSnapshot, SignalDefinitionState, SignalId};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{
    Denominator, Provenance, Reading, RecordSample, RecordSampleOutcome, RunId, SampleId,
    SeriesKey, SignalSampleSnapshot, UnreadReading,
};
use thalamus_model::primitives::Timestamp;
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::{registry, vocabulary};
use thalamus_store::eventlog::EventlogStore;

use crate::definitions::{self, Effective};
use crate::instance::LoadedInstance;

/// The id prefix of a sampling pass.
pub const RUN_PREFIX: &str = "run_";

/// Who records the samples: the provenance `producer` and `version`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Producer {
    /// The producer, such as `thalamus-cli`.
    pub name: String,
    /// Its version.
    pub version: String,
}

/// Why a definition or a read was not sampled. Nothing is recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// No definition has the id (`RecordSample`'s `unknown-signal`).
    UnknownSignal {
        /// The id asked for.
        signal: String,
    },
    /// The definition is not `Active` (`RecordSample`'s `not-active`).
    NotActive {
        /// The definition.
        signal: String,
        /// Its state.
        state: SignalDefinitionState,
    },
    /// The read's `dims` are not admitted by the instance's registry.
    Dims {
        /// The definition.
        signal: String,
        /// Why.
        refusal: DimsRefusal,
    },
    /// The definition declares no unit, so no sample of it can be written.
    NoUnit {
        /// The definition.
        signal: String,
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
            Self::NotActive { signal, state } => write!(
                f,
                "{signal}: not-active: the definition is {state:?}, and only an Active \
                 definition is sampled; nothing was recorded"
            ),
            Self::Dims { signal, refusal } => {
                write!(
                    f,
                    "{signal}: dims refused: {refusal}; the read was not recorded"
                )
            }
            Self::NoUnit { signal } => write!(
                f,
                "{signal}: the definition declares no unit; the read was not recorded"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// Why a pass could not run at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassError {
    /// The store's id sequence of `prefix` is spent.
    IdsSpent {
        /// `run_` or `smp_`.
        prefix: &'static str,
    },
}

impl fmt::Display for PassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdsSpent { prefix } => {
                write!(f, "the store's identity sequence of {prefix} is spent")
            }
        }
    }
}

impl std::error::Error for PassError {}

/// One recorded sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// Its id, minted by the store.
    pub sample_id: SampleId,
    /// The definition.
    pub signal: SignalId,
    /// The series.
    pub series: SeriesKey,
    /// `ok`, `partial` or `unknown`.
    pub quality: &'static str,
}

/// What one pass recorded and refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pass {
    /// The pass's run id, in the provenance of every sample it recorded.
    pub run_id: RunId,
    /// The samples recorded, in the order recorded.
    pub recorded: Vec<Recorded>,
    /// The reads refused.
    pub refused: Vec<Refusal>,
}

/// The definition `signal` names, when it may be sampled: the precedence of `RecordSample`,
/// `unknown-signal` before `not-active`.
///
/// # Errors
///
/// [`Refusal::UnknownSignal`] or [`Refusal::NotActive`].
pub fn admit(instance: &LoadedInstance, signal: &str) -> Result<SignalId, Refusal> {
    let Some(definition) = instance.instance().definition(signal) else {
        return Err(Refusal::UnknownSignal {
            signal: signal.to_owned(),
        });
    };
    match definitions::state(instance, signal).unwrap_or(definition.state()) {
        SignalDefinitionState::Active => Ok(definition.data().signal_id.clone()),
        state => Err(Refusal::NotActive {
            signal: signal.to_owned(),
            state,
        }),
    }
}

/// Every `Active` definition of the instance, in id order, by its effective lifecycle
/// (`crate::definitions`).
#[must_use]
pub fn active(instance: &LoadedInstance) -> Vec<SignalId> {
    instance
        .instance()
        .definitions()
        .filter(|definition| {
            definitions::state(instance, &definition.data().signal_id.0)
                == Some(SignalDefinitionState::Active)
        })
        .map(|definition| definition.data().signal_id.clone())
        .collect()
}

/// One pass over `signals`: [`begin`], then [`sample_into`].
///
/// # Errors
///
/// [`PassError::IdsSpent`] when the store mints no run id or no sample id.
///
/// # Panics
///
/// When the store cannot be read or written: the generated sample port answers no error.
pub fn run(
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    connectors: &Connectors,
    signals: &[SignalId],
    end: &Timestamp,
    producer: &Producer,
) -> Result<Pass, PassError> {
    let mut pass = begin(store)?;
    sample_into(
        &mut pass, instance, store, connectors, signals, end, producer,
    )?;
    Ok(pass)
}

/// A pass with nothing recorded yet, under a run id the store mints.
///
/// # Errors
///
/// [`PassError::IdsSpent`] when the store mints no run id.
///
/// # Panics
///
/// When the store's id stream cannot be read or written.
pub fn begin(store: &mut EventlogStore) -> Result<Pass, PassError> {
    let number = store
        .next_number(RUN_PREFIX)
        .ok_or(PassError::IdsSpent { prefix: RUN_PREFIX })?;
    Ok(Pass {
        run_id: RunId(format!("{RUN_PREFIX}{number:012}")),
        recorded: Vec::new(),
        refused: Vec::new(),
    })
}

/// Every read of each of `signals`, for the window ending at `end`, recorded into `pass` under
/// its run id: one sample per series. Reads of one definition that fall on one series key are
/// recorded as one `unknown` sample naming the collision and the number of reads ([`merge`]).
///
/// Each sample is added to `pass` as soon as it is recorded, so a store that fails in the middle
/// of the pass leaves `pass` holding every sample recorded before it failed.
///
/// # Errors
///
/// [`PassError::IdsSpent`] when the store mints no sample id.
///
/// # Panics
///
/// When the store cannot be read or written: the generated sample port answers no error.
pub fn sample_into(
    pass: &mut Pass,
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    connectors: &Connectors,
    signals: &[SignalId],
    end: &Timestamp,
    producer: &Producer,
) -> Result<(), PassError> {
    let run_id = pass.run_id.clone();
    for signal in signals {
        let reads = match instance.instance().definition(&signal.0) {
            Some(definition) => registry::read(connectors, definition.data(), end, &run_id),
            None => Vec::new(),
        };
        record_reads(pass, instance, store, signal, reads, end, producer)?;
    }
    Ok(())
}

/// `reads` of `signal`, made for the window ending at `end` by whoever read them (the extractor
/// registry in [`sample_into`], or a caller that bounds or replaces the read, such as the
/// service's scheduler), recorded into `pass` under its run id exactly as [`sample_into`] records
/// its own: one sample per series, reads that fall on one series key merged ([`merge`]).
///
/// # Errors
///
/// [`PassError::IdsSpent`] when the store mints no sample id.
///
/// # Panics
///
/// When the store cannot be read or written: the generated sample port answers no error.
pub fn record_reads(
    pass: &mut Pass,
    instance: &mut LoadedInstance,
    store: &mut EventlogStore,
    signal: &SignalId,
    reads: Vec<Read>,
    end: &Timestamp,
    producer: &Producer,
) -> Result<(), PassError> {
    let run_id = pass.run_id.clone();
    let Some(definition) = instance.instance().definition(&signal.0).cloned() else {
        pass.refused.push(Refusal::UnknownSignal {
            signal: signal.0.clone(),
        });
        return Ok(());
    };
    for (series, dims, reads) in by_series(instance, &definition, reads, &mut pass.refused) {
        let text = match reads.as_slice() {
            [read] => vocabulary::sample_text(definition.data(), read),
            _ => None,
        };
        let read = merge(&series, reads);
        let input = match sample_input(
            instance,
            store,
            &definition,
            (series, dims, read, text),
            end,
            &run_id,
            producer,
        ) {
            Ok(input) => input,
            Err(refusal) => {
                pass.refused.push(refusal);
                continue;
            }
        };
        let quality = quality(&input.reading);
        let series = input.series.clone();
        let (files, overlay) = instance.parts_mut();
        let mut ports = Ports {
            definitions: Effective { files, overlay },
            samples: store,
        };
        match record_sample(&mut ports, input) {
            Err(_) => return Err(PassError::IdsSpent { prefix: "smp_" }),
            Ok(RecordSampleOutcome::Recorded { response, .. }) => {
                pass.recorded.push(Recorded {
                    sample_id: response.sample_id,
                    signal: signal.clone(),
                    series,
                    quality,
                });
            }
            Ok(RecordSampleOutcome::UnknownSignal { .. }) => {
                pass.refused.push(Refusal::UnknownSignal {
                    signal: signal.0.clone(),
                });
            }
            Ok(RecordSampleOutcome::NotActive { .. }) => {
                pass.refused.push(Refusal::NotActive {
                    signal: signal.0.clone(),
                    state: definitions::state(instance, &signal.0).unwrap_or(definition.state()),
                });
            }
        }
    }
    Ok(())
}

/// The reads of `definition` grouped by series key, in the order each series was first read.
/// A read whose `dims` the instance does not admit is added to `refused` and left out.
fn by_series(
    instance: &LoadedInstance,
    definition: &Definition,
    reads: Vec<Read>,
    refused: &mut Vec<Refusal>,
) -> Vec<(SeriesKey, BTreeMap<String, DimensionValue>, Vec<Read>)> {
    let data = definition.data();
    let mut groups: Vec<(SeriesKey, BTreeMap<String, DimensionValue>, Vec<Read>)> = Vec::new();
    for read in reads {
        let dims: BTreeMap<String, DimensionValue> = read
            .dims
            .iter()
            .map(|(key, value)| (key.clone(), DimensionValue(value.clone())))
            .collect();
        if let Err(refusal) = admit_dims(
            instance.instance().registry(),
            data,
            &dims,
            instance.values(),
        ) {
            refused.push(Refusal::Dims {
                signal: data.signal_id.0.clone(),
                refusal,
            });
            continue;
        }
        let series = series_key(&data.signal_id, &dims);
        match groups.iter_mut().find(|(held, _, _)| *held == series) {
            Some((_, _, reads)) => reads.push(read),
            None => groups.push((series, dims, vec![read])),
        }
    }
    groups
}

/// The one read a pass records for `series`: the read itself when there is one; when several
/// reads fall on the series, the first of them made `unknown`, with no population, no raw text
/// and no [`Unread`](thalamus_sources::adapter::Unread) type, so it records no producer `text`;
/// its reason names the collision, the number of reads and each distinct reason they
/// carried.
fn merge(series: &SeriesKey, mut reads: Vec<Read>) -> Read {
    let count = reads.len();
    let mut reasons: Vec<String> = Vec::new();
    for read in &reads {
        if let Reading::Unknown(unread) = &read.reading
            && !unread.reason.is_empty()
            && !reasons.contains(&unread.reason)
        {
            reasons.push(unread.reason.clone());
        }
    }
    let mut read = reads.swap_remove(0);
    if count == 1 {
        return read;
    }
    let mut reason = format!(
        "{count} reads of one pass fall on series {}; one pass records one sample per series",
        series.0
    );
    for held in reasons {
        reason.push_str("; ");
        reason.push_str(&held);
    }
    read.reading = Reading::Unknown(UnreadReading { reason });
    read.denominator = None;
    read.raw = None;
    read.unread = None;
    read
}

/// The earliest `ts` stored for `series`, by instant, but never later than `end`: the origin of a
/// `first_sample` series for a sample at `end`. A stored `ts` that names no instant is skipped,
/// never an origin. `end` itself when nothing earlier is stored.
fn first_sample_ts(store: &EventlogStore, series: &SeriesKey, end: &Timestamp) -> Timestamp {
    series_samples(store, series)
        .ok()
        .and_then(|rows| {
            rows.into_iter()
                .find(|row| Instant::parse(&row.ts.0).is_some())
        })
        .map(|row| row.ts)
        .filter(|first| timestamp_order(&first.0, &end.0).is_lt())
        .unwrap_or_else(|| end.clone())
}

/// The `RecordSample` input of the one read of `series`.
fn sample_input(
    instance: &LoadedInstance,
    store: &EventlogStore,
    definition: &Definition,
    (series, dims, read, text): (
        SeriesKey,
        BTreeMap<String, DimensionValue>,
        Read,
        Option<String>,
    ),
    end: &Timestamp,
    run_id: &RunId,
    producer: &Producer,
) -> Result<RecordSample, Refusal> {
    let data = definition.data();
    let unit = data.unit.clone().ok_or_else(|| Refusal::NoUnit {
        signal: data.signal_id.0.clone(),
    })?;
    let denominator = read.denominator.map(|population| {
        let first = first_sample_ts(store, &series, end);
        let since = resolve_since(
            definition,
            &dims,
            instance.values(),
            Some(first.clone()),
            read.retention_start.clone(),
        )
        .unwrap_or(first);
        Denominator {
            n: population.n,
            unit: population.unit,
            age_s: age_s(&since, end),
            since,
        }
    });
    Ok(RecordSample {
        signal: data.signal_id.clone(),
        series,
        dims,
        ts: end.clone(),
        window: read.window,
        reading: read.reading,
        text,
        unit,
        denominator,
        provenance: Provenance {
            producer: producer.name.clone(),
            version: producer.version.clone(),
            definition_rev: data.rev,
            adapter: read.adapter,
            connection: read.connection,
            query_sha256: read.query_sha256,
            run_id: run_id.clone(),
        },
    })
}

/// Whole seconds from `since` to `ts`, never below 0; 0 where either names no instant.
fn age_s(since: &Timestamp, ts: &Timestamp) -> i64 {
    match (Instant::parse(&since.0), Instant::parse(&ts.0)) {
        (Some(since), Some(ts)) => (ts.seconds() - since.seconds()).max(0),
        _ => 0,
    }
}

fn quality(reading: &Reading) -> &'static str {
    match reading {
        Reading::Ok(_) => "ok",
        Reading::Partial(_) => "partial",
        Reading::Unknown(_) => "unknown",
    }
}

/// The ports `RecordSample` runs over: the instance's file-backed definitions seen through its
/// overlay (an expired mute answers `Active`, `crate::definitions`) and the store's samples and
/// id sequence.
struct Ports<'a> {
    definitions: Effective<'a>,
    samples: &'a mut EventlogStore,
}

impl SignalDefinitionStorage for Ports<'_> {
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

impl SignalSampleStorage for Ports<'_> {
    fn get(&self, identity: &SampleId) -> Option<SignalSampleSnapshot> {
        SignalSampleStorage::get(&*self.samples, identity)
    }

    fn put(&mut self, snapshot: SignalSampleSnapshot) {
        SignalSampleStorage::put(self.samples, snapshot);
    }

    fn delete(&mut self, identity: &SampleId) {
        SignalSampleStorage::delete(self.samples, identity);
    }

    fn list(&self) -> Vec<SignalSampleSnapshot> {
        SignalSampleStorage::list(&*self.samples)
    }
}

impl IdSequence for Ports<'_> {
    fn next_number(&mut self, prefix: &'static str) -> Option<u64> {
        self.samples.next_number(prefix)
    }
}
