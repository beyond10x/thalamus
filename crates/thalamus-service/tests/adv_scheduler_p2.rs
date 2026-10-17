//! Adversary pass 2 on the scheduler: the pass-1 corrections (per-read deadlines, missed slots
//! counted from the later of the handled slot and the newest stored sample, a "behind" slot that is
//! not read). Each test runs on a fake clock with an in-process reader.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration as WallDuration;

use thalamus_model::definition::{SignalDefinitionData, SignalId};
use thalamus_model::observation::{CompleteReading, Reading, RunId, SignalSampleData};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_pipeline::sample::{self, Producer};
use thalamus_service::scheduler::{Clock, Reader, Scheduler};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

const T0: i64 = 1_792_018_500;
const FIVE_MINUTES: i64 = 300;
const INSTANCE: &str = "fixture";
const DEPTH: &str = "service.queue.depth";
const ACTIVE: &str = "lifecycle: active";
const MUTED: &str = "lifecycle: muted\nmute_reason: maintenance\nmute_until: 2099-01-01T00:00:00Z";

#[derive(Clone)]
struct FakeClock {
    now: Arc<AtomicI64>,
}

impl FakeClock {
    fn at(now: i64) -> Self {
        Self {
            now: Arc::new(AtomicI64::new(now)),
        }
    }

    fn set(&self, now: i64) {
        self.now.store(now, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }

    fn span(&self, seconds: i64) -> WallDuration {
        WallDuration::from_millis(u64::try_from(seconds).unwrap_or(0) * 1_000)
    }

    fn sleep_until(&self, at: i64) {
        self.set(at);
    }
}

fn ok_read(definition: &SignalDefinitionData, end: &Timestamp, value: &str) -> Read {
    let mut read = unknown(definition, end, "");
    read.reading = Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    });
    read
}

/// Answers 7 at once for every read.
#[derive(Clone, Default)]
struct Answering;

impl Reader for Answering {
    fn read(&self, definition: &SignalDefinitionData, end: &Timestamp, _: &RunId) -> Vec<Read> {
        vec![ok_read(definition, end, "7")]
    }
}

fn definition(id: &str, lifecycle: &str) -> String {
    format!(
        "id: {id}
title: A signal of the adversary tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: []
source: {{ adapter: metrics, connection: metrics-main }}
extractor: promql.value
query: {{ value: \"sum(x)\" }}
window: PT5M
cadence: PT5M
history: store
denominator: {{ unit: \"{{message}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}
evaluation:
  rule: higher_is_worse
  baselines: [fixed]
  fixed: \"0\"
  warn: {{ abs: \"100\" }}
  crit: {{ abs: \"1000\" }}
owner: team:service
{lifecycle}
rev: 1
revised_at: 2026-10-02T00:00:00Z
revised_by: operator
"
    )
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory");
    for entry in std::fs::read_dir(from).expect("the fixture exists") {
        let path = entry.expect("a fixture entry").path();
        let target = to.join(path.file_name().expect("a file name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy a fixture file");
        }
    }
}

fn define(instance: &Path, lifecycle: &str) {
    std::fs::write(
        instance.join("definitions").join(format!("{DEPTH}.yaml")),
        definition(DEPTH, lifecycle),
    )
    .expect("write a definition");
}

/// An instance directory holding the depth definition, and its store's locator.
fn world(name: &str) -> (PathBuf, Locator) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_scheduler_p2")
        .join(name);
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("clear the test directory");
    }
    let instance = root.join("instance");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scheduler"),
        &instance,
    );
    std::fs::create_dir_all(instance.join("definitions")).expect("definitions directory");
    define(&instance, ACTIVE);
    (instance, Locator::Sqlite(root.join("store.sqlite")))
}

fn scheduler(
    instance: &Path,
    locator: &Locator,
    clock: &FakeClock,
) -> Scheduler<FakeClock, Answering> {
    Scheduler::open(
        instance,
        locator,
        clock.clone(),
        Answering,
        Connectors::new(instance.join("no-connectors")),
    )
    .expect("the scheduler opens")
}

fn samples(locator: &Locator) -> Vec<SignalSampleData> {
    EventlogStore::open(locator, INSTANCE)
        .expect("the store opens")
        .samples()
        .to_vec()
}

fn rfc3339(at: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(at)
        .expect("an instant")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("formats")
}

/// What `thalamus sample` with no `--at` does (story:m1-cli-sample): one pass, recorded through
/// `thalamus_pipeline::sample`, for the window ending at the second it runs, which is no slot
/// boundary.
fn cli_sample(instance: &Path, locator: &Locator, at: i64) {
    let mut loaded = LoadedInstance::load(instance).expect("the instance loads");
    let mut store = EventlogStore::open(locator, loaded.name()).expect("the store opens");
    let mut pass = sample::begin(&mut store).expect("a run id");
    let end = Timestamp(rfc3339(at));
    let data = loaded
        .instance()
        .definition(DEPTH)
        .expect("defined")
        .data()
        .clone();
    sample::record_reads(
        &mut pass,
        &mut loaded,
        &mut store,
        &SignalId(DEPTH.to_owned()),
        vec![ok_read(&data, &end, "3")],
        &end,
        &Producer {
            name: "thalamus".to_owned(),
            version: "0".to_owned(),
        },
    )
    .expect("recorded");
    assert!(pass.refused.is_empty(), "{:?}", pass.refused);
}

fn at_slot(locator: &Locator, slot: i64) -> Vec<SignalSampleData> {
    let ts = rfc3339(slot);
    samples(locator)
        .into_iter()
        .filter(|sample| sample.signal.0 == DEPTH && sample.ts.0 == ts)
        .collect()
}

/// Module doc: every `Active` definition "whose current slot holds no sample yet" goes through
/// one pass. An operator's `thalamus sample` at T0+137 stores a sample at an instant that is no
/// slot; the service started at T0+200 must still sample slot T0, which holds nothing. The
/// correction's `behind` (`current < since`, with `since` the unaligned newest instant) skips it,
/// and nothing records it as missed either.
#[test]
fn a_cli_sample_inside_the_current_slot_does_not_stop_the_first_tick() {
    let (instance, locator) = world("cli_first_tick");
    cli_sample(&instance, &locator, T0 + 137);
    let clock = FakeClock::at(T0 + 200);
    let mut scheduler = scheduler(&instance, &locator, &clock);
    let tick = scheduler.tick();

    assert_eq!(
        at_slot(&locator, T0).len(),
        1,
        "slot {} holds no sample after the first tick (tick: {tick:?})",
        rfc3339(T0)
    );
}

/// The same while running: the service handled T0; the tick for T0+300 comes 40 s late (a slow
/// pass, a hung read held to its cadence); an operator's `thalamus sample` ran at T0+330. Slot
/// T0+300 is neither read nor recorded as missed: it is lost, against D16.
#[test]
fn a_cli_sample_before_a_late_tick_does_not_lose_the_slot() {
    let (instance, locator) = world("cli_late_tick");
    let clock = FakeClock::at(T0 + 1);
    let mut scheduler = scheduler(&instance, &locator, &clock);
    scheduler.tick();
    assert_eq!(at_slot(&locator, T0).len(), 1, "slot T0 sampled");

    cli_sample(&instance, &locator, T0 + FIVE_MINUTES + 30);
    clock.set(T0 + FIVE_MINUTES + 40);
    let tick = scheduler.tick();

    assert_eq!(
        at_slot(&locator, T0 + FIVE_MINUTES).len(),
        1,
        "slot {} holds no sample, read or missed (tick: {tick:?})",
        rfc3339(T0 + FIVE_MINUTES)
    );
}

/// The correction's rule (scheduler.rs: "A slot older than the newest stored sample, or than the
/// handled slot, is not read"; pass 1 F2) holds on the first tick and for a definition that stayed
/// Active, but not for one re-activated while running: a mute drops its handled slot, and the
/// mid-run activation path ignores the newest stored sample. The service samples T0+900; the
/// definition is muted; the clock steps back to T0; the definition is unmuted: slot T0, behind
/// the newest stored sample, is read.
#[test]
fn a_definition_reactivated_after_a_backward_step_reads_no_slot_behind_its_newest_sample() {
    let (instance, locator) = world("reactivated_behind");
    let s3 = T0 + 3 * FIVE_MINUTES;
    let clock = FakeClock::at(s3 + 1);
    let mut scheduler = scheduler(&instance, &locator, &clock);
    scheduler.tick();
    assert_eq!(at_slot(&locator, s3).len(), 1, "slot s3 sampled");

    define(&instance, MUTED);
    scheduler.tick();
    clock.set(T0 + 1);
    scheduler.tick();
    define(&instance, ACTIVE);
    let tick = scheduler.tick();

    let behind = at_slot(&locator, T0);
    assert!(
        behind.is_empty(),
        "slot {} behind the newest stored sample ({}) was read: {behind:#?} (tick: {tick:?})",
        rfc3339(T0),
        rfc3339(s3)
    );
}
