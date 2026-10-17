//! The scheduler (design § 5.3, lesson L7): the service, not whoever remembers, owns the cadence.
//!
//! Slots are multiples of a definition's cadence from the Unix epoch, UTC. On each tick the
//! instance is loaded again from its directory, so an activation, a revision, a mute or a
//! retirement takes effect at the next slot, and every `Active` definition (the
//! `thalamus.definition.ActiveDefinitions` view: `state == Active`) whose current slot holds no
//! sample yet goes through one pipeline pass, under one `run_` id per pass
//! (`thalamus.observation.Provenance`): its read, `RecordSample` per series
//! (`thalamus_pipeline::sample::record_reads`), then grading and `RecordEvaluation` of what the
//! pass recorded (`thalamus_pipeline::evaluate`). The scheduler adds no pass of its own.
//! `Proposed`, `Muted` and `Retired` definitions are neither read nor recorded.
//!
//! Each read runs on its own thread and is given the definition's cadence (on the wall, as the
//! [`Clock`] maps it). A read that has not answered by then is recorded as one `unknown` read with
//! reason [`READ_TIMED_OUT`] (design § 2.2), never as 0; its late answer is dropped, so nothing it
//! says reaches the store.
//!
//! Missed slots (design § 11 D16) are recorded, never read and never interpolated. On the first
//! tick after start, every slot of an `Active` definition after its newest stored sample and
//! before the current slot is missed; on a later tick, every slot after the last slot this
//! scheduler handled for the definition (a clock jump, a tick that came late). Each missed slot
//! holds one `unknown` sample with reason [`MISSED_SLOT`] per series of the definition's newest
//! pass, recorded in a pass of their own before the current slot is sampled. A slot that already
//! holds a sample of the definition (another writer's, or this one's before a restart) is never
//! recorded again. A definition that becomes `Active` while the scheduler runs starts at its
//! current slot: the slots it spent `Proposed` or `Muted` were not missed.
//!
//! The store is used from the scheduler's own thread, never from inside a Tokio runtime, as
//! `thalamus_store::eventlog` requires.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant as WallInstant;

use thalamus_core::behaviour::observation::Instant;
use thalamus_model::definition::{SignalDefinitionData, SignalId};
use thalamus_model::observation::RunId;
use thalamus_model::primitives::Timestamp;
use thalamus_pipeline::evaluate;
use thalamus_pipeline::instance::{InstanceError, LoadedInstance};
use thalamus_pipeline::sample::{self, Producer};
use thalamus_sources::adapter::{Read, unknown, window_ending};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;
use thalamus_store::eventlog::{EventlogStore, Locator, StoreError};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// The reason of a missed slot's sample (design § 11 D16).
pub const MISSED_SLOT: &str = "missed_slot";
/// The reason of a read that did not finish within the cadence (design § 2.2).
pub const READ_TIMED_OUT: &str = "read timed out";
/// The provenance `producer` of every sample the scheduler records.
pub const PRODUCER: &str = "thalamus-service";

/// The scheduler's time, injected so a test runs on a fake clock.
pub trait Clock {
    /// Now, in Unix seconds.
    fn now(&self) -> i64;
    /// How long `seconds` of this clock's time last on the wall: the budget of a read.
    fn span(&self, seconds: i64) -> std::time::Duration;
    /// Waits until the clock reads `at` (Unix seconds).
    fn sleep_until(&self, at: i64);
}

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> i64 {
        OffsetDateTime::now_utc().unix_timestamp()
    }

    fn span(&self, seconds: i64) -> std::time::Duration {
        std::time::Duration::from_secs(u64::try_from(seconds).unwrap_or(0))
    }

    fn sleep_until(&self, at: i64) {
        let wait = at - self.now();
        if wait > 0 {
            std::thread::sleep(self.span(wait));
        }
    }
}

/// The source side of a pass, injected so a test runs with no source.
pub trait Reader: Send + Sync + 'static {
    /// One read per series of `definition` for the window ending at `end`.
    fn read(&self, definition: &SignalDefinitionData, end: &Timestamp, run_id: &RunId)
    -> Vec<Read>;
}

/// Reads through the extractor registry and the `connectors` binary.
#[derive(Debug, Clone)]
pub struct ConnectorsReader(pub Connectors);

impl Reader for ConnectorsReader {
    fn read(
        &self,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        run_id: &RunId,
    ) -> Vec<Read> {
        registry::read(&self.0, definition, end, run_id)
    }
}

/// Why the scheduler could not start.
#[derive(Debug)]
pub enum OpenError {
    /// The instance was refused.
    Instance(Vec<InstanceError>),
    /// The store could not be opened.
    Store(StoreError),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instance(errors) => {
                f.write_str("invalid instance")?;
                for error in errors {
                    write!(f, "; {error}")?;
                }
                Ok(())
            }
            Self::Store(error) => write!(f, "store refused: {error}"),
        }
    }
}

impl std::error::Error for OpenError {}

/// What one tick did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tick {
    /// The run id of each pass the tick recorded, in order: the missed slots' pass first.
    pub runs: Vec<RunId>,
    /// What went wrong, one line each: an instance refused, a read or grade refused, a store
    /// failure. Nothing here stops the next tick.
    pub problems: Vec<String>,
}

/// Samples each `Active` definition of one instance on its cadence.
pub struct Scheduler<C, R> {
    dir: PathBuf,
    store: EventlogStore,
    clock: C,
    reader: Arc<R>,
    connectors: Connectors,
    producer: Producer,
    /// Whether a tick has run since start.
    started: bool,
    /// The last slot handled per `Active` definition, in Unix seconds.
    handled: BTreeMap<String, i64>,
}

impl<C: Clock, R: Reader> Scheduler<C, R> {
    /// Opens the instance in `dir` and its store. `connectors` serves the source reads grading
    /// makes for a `history: source` baseline.
    ///
    /// # Errors
    ///
    /// [`OpenError`].
    pub fn open(
        dir: &Path,
        locator: &Locator,
        clock: C,
        reader: R,
        connectors: Connectors,
    ) -> Result<Self, OpenError> {
        let instance = LoadedInstance::load(dir).map_err(OpenError::Instance)?;
        let store = EventlogStore::open(locator, instance.name()).map_err(OpenError::Store)?;
        Ok(Self {
            dir: dir.to_owned(),
            store,
            clock,
            reader: Arc::new(reader),
            connectors,
            producer: Producer {
                name: PRODUCER.to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            started: false,
            handled: BTreeMap::new(),
        })
    }

    /// Ticks at each slot boundary until `stop` answers true.
    pub fn run(&mut self, stop: impl Fn() -> bool) {
        while !stop() {
            let tick = self.tick();
            for problem in &tick.problems {
                eprintln!("thalamus-service: {problem}");
            }
            let next = self.next_slot();
            self.clock.sleep_until(next);
        }
    }

    /// The earliest slot boundary after now among the `Active` definitions; a minute from now
    /// when there is none.
    fn next_slot(&self) -> i64 {
        let now = self.clock.now();
        LoadedInstance::load(&self.dir)
            .ok()
            .and_then(|instance| {
                sample::active(&instance)
                    .iter()
                    .filter_map(|signal| instance.instance().definition(&signal.0))
                    .filter_map(|definition| cadence_seconds(definition.data()))
                    .map(|cadence| (now.div_euclid(cadence) + 1) * cadence)
                    .min()
            })
            .unwrap_or(now + 60)
    }

    /// One tick at the clock's now: the missed slots' pass, then the current slots' pass.
    pub fn tick(&mut self) -> Tick {
        let mut tick = Tick::default();
        let now = self.clock.now();
        let mut instance = match LoadedInstance::load(&self.dir) {
            Ok(instance) => instance,
            Err(errors) => {
                tick.problems.extend(
                    errors
                        .iter()
                        .map(|error| format!("invalid instance: {error}")),
                );
                return tick;
            }
        };
        if let Err(error) = self.store.refresh() {
            tick.problems.push(format!("store refused: {error}"));
            return tick;
        }

        let held = Held::read(&self.store);
        let mut missed: Vec<(SignalId, i64, Vec<Read>)> = Vec::new();
        let mut due: Vec<(SignalId, i64)> = Vec::new();
        let mut active = BTreeMap::new();
        for signal in sample::active(&instance) {
            let Some(definition) = instance.instance().definition(&signal.0) else {
                continue;
            };
            let data = definition.data();
            let Some(cadence) = cadence_seconds(data) else {
                tick.problems.push(format!(
                    "{}: cadence {} is no positive length; not sampled",
                    signal.0, data.cadence.0
                ));
                continue;
            };
            let current = now.div_euclid(cadence) * cadence;
            // The newest stored sample counts by its slot: a hand-run sample off a slot boundary
            // puts no later slot behind, and the slot itself is still read, since no sample sits
            // at its instant.
            let newest = held
                .newest(&signal.0)
                .map(|newest| newest.div_euclid(cadence) * cadence);
            let handled = self.handled.get(&signal.0).copied();
            // On the first tick, from the newest stored slot; after it, from the later of the
            // handled slot and the newest stored one, so a clock that steps back never brings a
            // slot behind either into reach again. A definition that became Active while running
            // has no handled slot and is not filled.
            let since = if self.started {
                handled.map(|handled| handled.max(newest.unwrap_or(handled)))
            } else {
                newest
            };
            if let Some(since) = since {
                let series = held.newest_pass_dims(&signal.0);
                let mut slot = since + cadence;
                while slot < current {
                    if !held.has(&signal.0, slot) {
                        missed.push((signal.clone(), slot, missed_reads(data, &series, slot)));
                    }
                    slot += cadence;
                }
            }
            // A slot older than the newest stored slot, or than the handled slot, is not read.
            let bound = handled.max(newest);
            let behind = bound.is_some_and(|bound| current < bound);
            if !behind && !held.has(&signal.0, current) {
                due.push((signal.clone(), current));
            }
            // The handled slot never decreases.
            active.insert(
                signal.0.clone(),
                bound.map_or(current, |bound| bound.max(current)),
            );
        }
        self.started = true;
        self.handled = active;

        if !missed.is_empty()
            && let Some(pass) = self.begin(&mut tick)
        {
            self.record(pass, &mut instance, missed, &mut tick);
        }
        if !due.is_empty()
            && let Some(pass) = self.begin(&mut tick)
        {
            let reads = self.read(&instance, &due, &pass.run_id);
            self.record(pass, &mut instance, reads, &mut tick);
        }
        tick
    }

    /// A pass under a run id the store mints.
    fn begin(&mut self, tick: &mut Tick) -> Option<sample::Pass> {
        sample::begin(&mut self.store)
            .map_err(|error| tick.problems.push(error.to_string()))
            .ok()
    }

    /// Every due read, each on its own thread with its own deadline: the instant it was started
    /// plus its definition's cadence. A read whose answer arrived after its deadline, or has not
    /// arrived by it, is one `unknown` read saying [`READ_TIMED_OUT`], whatever order the reads
    /// are collected in.
    fn read(
        &self,
        instance: &LoadedInstance,
        due: &[(SignalId, i64)],
        run_id: &RunId,
    ) -> Vec<(SignalId, i64, Vec<Read>)> {
        let pending: Vec<_> = due
            .iter()
            .filter_map(|(signal, slot)| {
                let data = instance.instance().definition(&signal.0)?.data().clone();
                let budget = self.clock.span(cadence_seconds(&data)?);
                let end = Timestamp(rfc3339(*slot));
                let (send, receive) = mpsc::channel();
                let reader = Arc::clone(&self.reader);
                let (thread_data, thread_end, run) = (data.clone(), end.clone(), run_id.clone());
                let deadline = WallInstant::now() + budget;
                std::thread::spawn(move || {
                    let reads = reader.read(&thread_data, &thread_end, &run);
                    let _ = send.send((reads, WallInstant::now()));
                });
                Some((signal.clone(), *slot, data, end, deadline, receive))
            })
            .collect();
        pending
            .into_iter()
            .map(|(signal, slot, data, end, deadline, receive)| {
                let left = deadline.saturating_duration_since(WallInstant::now());
                let reads = match receive.recv_timeout(left) {
                    Ok((reads, arrived)) if arrived <= deadline => reads,
                    _ => vec![unknown(&data, &end, READ_TIMED_OUT)],
                };
                (signal, slot, reads)
            })
            .collect()
    }

    /// The rest of `pass`: `RecordSample` for every read, then grading of what the pass
    /// recorded.
    fn record(
        &mut self,
        mut pass: sample::Pass,
        instance: &mut LoadedInstance,
        reads: Vec<(SignalId, i64, Vec<Read>)>,
        tick: &mut Tick,
    ) {
        let mut signals: Vec<SignalId> = Vec::new();
        for (signal, slot, reads) in reads {
            let end = Timestamp(rfc3339(slot));
            if let Err(error) = sample::record_reads(
                &mut pass,
                instance,
                &mut self.store,
                &signal,
                reads,
                &end,
                &self.producer,
            ) {
                tick.problems.push(error.to_string());
                break;
            }
            if !signals.contains(&signal) {
                signals.push(signal);
            }
        }
        tick.problems
            .extend(pass.refused.iter().map(ToString::to_string));
        match evaluate::run(
            instance,
            &mut self.store,
            &self.connectors,
            &signals,
            Some(&pass.run_id),
        ) {
            Ok(graded) => tick
                .problems
                .extend(graded.refused.iter().map(ToString::to_string)),
            Err(error) => tick.problems.push(error.to_string()),
        }
        tick.runs.push(pass.run_id);
    }
}

/// What the store holds, per definition: the slots holding a sample, the newest sample's slot
/// and run, and the dims of each series of that run.
struct Held {
    slots: BTreeMap<String, BTreeSet<i64>>,
    newest: BTreeMap<String, (i128, i64, RunId)>,
    dims: BTreeMap<(String, String), Vec<BTreeMap<String, String>>>,
}

impl Held {
    fn read(store: &EventlogStore) -> Self {
        let mut slots: BTreeMap<String, BTreeSet<i64>> = BTreeMap::new();
        let mut newest: BTreeMap<String, (i128, i64, RunId)> = BTreeMap::new();
        for sample in store.samples() {
            let Some(at) = Instant::parse(&sample.ts.0) else {
                continue;
            };
            slots
                .entry(sample.signal.0.clone())
                .or_default()
                .insert(at.seconds());
            let entry = (
                at.nanoseconds(),
                at.seconds(),
                sample.provenance.run_id.clone(),
            );
            match newest.get(&sample.signal.0) {
                Some(held) if held.0 >= entry.0 => {}
                _ => {
                    newest.insert(sample.signal.0.clone(), entry);
                }
            }
        }
        let mut dims: BTreeMap<(String, String), Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for sample in store.samples() {
            let Some((_, _, run)) = newest.get(&sample.signal.0) else {
                continue;
            };
            if sample.provenance.run_id != *run
                || !seen.insert((sample.signal.0.clone(), sample.series.0.clone()))
            {
                continue;
            }
            dims.entry((sample.signal.0.clone(), run.0.clone()))
                .or_default()
                .push(
                    sample
                        .dims
                        .iter()
                        .map(|(key, value)| (key.clone(), value.0.clone()))
                        .collect(),
                );
        }
        Self {
            slots,
            newest,
            dims,
        }
    }

    fn has(&self, signal: &str, slot: i64) -> bool {
        self.slots
            .get(signal)
            .is_some_and(|slots| slots.contains(&slot))
    }

    /// The slot of the newest stored sample of `signal`.
    fn newest(&self, signal: &str) -> Option<i64> {
        self.newest.get(signal).map(|(_, seconds, _)| *seconds)
    }

    /// The dims of each series of the newest pass of `signal`.
    fn newest_pass_dims(&self, signal: &str) -> Vec<BTreeMap<String, String>> {
        self.newest
            .get(signal)
            .and_then(|(_, _, run)| self.dims.get(&(signal.to_owned(), run.0.clone())))
            .cloned()
            .unwrap_or_default()
    }
}

/// One `unknown` read per series in `series` at `slot`, saying [`MISSED_SLOT`]: nothing is read.
fn missed_reads(
    definition: &SignalDefinitionData,
    series: &[BTreeMap<String, String>],
    slot: i64,
) -> Vec<Read> {
    let end = Timestamp(rfc3339(slot));
    series
        .iter()
        .map(|dims| {
            let mut read = unknown(definition, &end, MISSED_SLOT);
            read.dims.clone_from(dims);
            read
        })
        .collect()
}

/// The length of `definition`'s cadence in seconds, when it is positive.
fn cadence_seconds(definition: &SignalDefinitionData) -> Option<i64> {
    let end = Timestamp("2000-01-01T00:00:00Z".to_owned());
    let window = window_ending(&definition.cadence, &end).ok()?;
    let start = Instant::parse(&window.start.0)?.seconds();
    let end = Instant::parse(&window.end.0)?.seconds();
    (end > start).then_some(end - start)
}

/// The RFC 3339 text of Unix second `at`, in UTC.
fn rfc3339(at: i64) -> String {
    OffsetDateTime::from_unix_timestamp(at)
        .ok()
        .and_then(|at| at.format(&Rfc3339).ok())
        .unwrap_or_default()
}
