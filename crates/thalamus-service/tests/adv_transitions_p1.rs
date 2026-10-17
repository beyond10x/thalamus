//! Adversary pass 1 on story:m2-transitions. Destined for `crates/thalamus-service/tests/`.
//!
//! The fixture and the fake source are those of `tests/transitions.rs`; the definition is graded
//! against a fixed 0, `WARN` above the `warn` number, with the default persistence (`k` 2,
//! `k_crit` 1, `m` 3).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration as WallDuration;

use thalamus_model::definition::{SignalDefinitionData, SignalId};
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::{CompleteReading, Reading, RunId, SeriesKey};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::transition::SeriesTransitions;
use thalamus_pipeline::evaluate;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_pipeline::sample::{self, Producer};
use thalamus_service::scheduler::{Clock, Reader, Scheduler, Tick};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

const T0: i64 = 1_792_018_500;
const FIVE_MINUTES: i64 = 300;
const INSTANCE: &str = "fixture";
const DEPTH: &str = "service.queue.depth";
const OK: &str = "0";
const WARN: &str = "150";

#[derive(Clone)]
struct FakeClock(Arc<AtomicI64>);

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn span(&self, seconds: i64) -> WallDuration {
        WallDuration::from_secs(u64::try_from(seconds).unwrap_or(0))
    }
    fn sleep_until(&self, at: i64) {
        self.0.store(at, Ordering::SeqCst);
    }
}

#[derive(Clone, Default)]
struct Source {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

fn read_of(definition: &SignalDefinitionData, end: &Timestamp, value: &str) -> Read {
    let mut read = unknown(definition, end, "");
    read.reading = Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    });
    read
}

impl Reader for Source {
    fn read(&self, definition: &SignalDefinitionData, end: &Timestamp, _: &RunId) -> Vec<Read> {
        let values = self.values.lock().expect("the values");
        match values.get(&end.0) {
            Some(value) => vec![read_of(definition, end, value)],
            None => vec![unknown(definition, end, "no value set")],
        }
    }
}

struct World {
    root: PathBuf,
    clock: FakeClock,
    source: Source,
    next: i64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("service_adv_transitions")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scheduler"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("instance/definitions")).expect("definitions");
        let world = Self {
            root,
            clock: FakeClock(Arc::new(AtomicI64::new(T0))),
            source: Source::default(),
            next: T0,
        };
        world.define(1, "100", "1000");
        world
    }

    fn define(&self, rev: i64, warn: &str, crit: &str) {
        std::fs::write(
            self.root.join(format!("instance/definitions/{DEPTH}.yaml")),
            definition(rev, warn, crit),
        )
        .expect("write the definition");
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.root.join("store.sqlite"))
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn connectors(&self) -> Connectors {
        Connectors::new(self.root.join("no-connectors"))
    }

    fn scheduler(&self) -> Scheduler<FakeClock, Source> {
        Scheduler::open(
            &self.instance(),
            &self.locator(),
            self.clock.clone(),
            self.source.clone(),
            self.connectors(),
        )
        .unwrap_or_else(|error| panic!("the scheduler opens: {error}"))
    }

    fn store(&self) -> EventlogStore {
        EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens")
    }

    fn load(&self) -> LoadedInstance {
        LoadedInstance::load(&self.instance())
            .unwrap_or_else(|errors| panic!("the instance loads: {} error(s)", errors.len()))
    }

    fn drive(&mut self, scheduler: &mut Scheduler<FakeClock, Source>, values: &[&str]) -> Tick {
        let mut all = Tick::default();
        for value in values {
            self.source
                .values
                .lock()
                .expect("the values")
                .insert(rfc3339(self.next), (*value).to_owned());
            self.clock.0.store(self.next, Ordering::SeqCst);
            let tick = scheduler.tick();
            assert!(tick.problems.is_empty(), "{:?}", tick.problems);
            all.events.extend(tick.events);
            self.next += FIVE_MINUTES;
        }
        all
    }

    /// What `thalamus sample` followed by `thalamus eval --run` does while the service is down:
    /// one sample per value at the next slots, graded, and no transition observed.
    fn hand_run(&mut self, values: &[&str]) {
        let mut instance = self.load();
        let mut store = self.store();
        let signal = SignalId(DEPTH.to_owned());
        let producer = Producer {
            name: "thalamus-cli".to_owned(),
            version: "0".to_owned(),
        };
        let mut pass = sample::begin(&mut store).unwrap_or_else(|e| panic!("begin: {e}"));
        for value in values {
            let end = Timestamp(rfc3339(self.next));
            let data = instance
                .instance()
                .definition(DEPTH)
                .expect("the definition")
                .data()
                .clone();
            let reads = vec![read_of(&data, &end, value)];
            sample::record_reads(
                &mut pass,
                &mut instance,
                &mut store,
                &signal,
                reads,
                &end,
                &producer,
            )
            .unwrap_or_else(|e| panic!("record: {e}"));
            self.next += FIVE_MINUTES;
        }
        evaluate::run(
            &instance,
            &mut store,
            &self.connectors(),
            &[signal],
            Some(&pass.run_id),
        )
        .unwrap_or_else(|e| panic!("grade: {e}"));
    }

    fn transitions(&self) -> Vec<SeriesTransitions> {
        self.store()
            .series_transitions(&SeriesKey(DEPTH.to_owned()))
    }
}

fn definition(rev: i64, warn: &str, crit: &str) -> String {
    format!(
        "id: {DEPTH}
title: A signal of the transition tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: []
source: {{ adapter: metrics, connection: metrics-main }}
extractor: promql.value
query: {{ value: \"sum(queue_depth)\" }}
window: PT5M
cadence: PT5M
history: store
denominator: {{ unit: \"{{message}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}
evaluation:
  rule: higher_is_worse
  baselines: [fixed]
  fixed: \"0\"
  warn: {{ abs: \"{warn}\" }}
  crit: {{ abs: \"{crit}\" }}
owner: team:service
lifecycle: active
rev: {rev}
revised_at: 2026-10-0{rev}T00:00:00Z
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

fn rfc3339(at: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(at)
        .expect("an instant")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC 3339")
}

fn pairs(transitions: &[SeriesTransitions]) -> Vec<(Option<Verdict>, Verdict)> {
    transitions
        .iter()
        .map(|t| (t.from_verdict, t.to_verdict))
        .collect()
}

/// F1. Two `WARN` evaluations stored while the service was down (a hand-run sample and grade, or
/// a crash between grading and `transitions::observe`) settle `OK` -> `WARN`. Seeding on restart
/// replays them, settles that change and discards it (`let _seeded`), so it is never recorded or
/// emitted: the consumer never learns the signal went out of bounds.
#[test]
fn adv_change_settled_while_down_is_recorded_on_restart() {
    let mut world = World::new("settled-while-down");
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK, OK]);
    assert_eq!(pairs(&world.transitions()), vec![(None, Verdict::OK)]);
    drop(scheduler);
    world.hand_run(&[WARN, WARN]);
    let mut scheduler = world.scheduler();
    let tick = world.drive(&mut scheduler, &[WARN]);
    assert_eq!(
        pairs(&world.transitions()),
        vec![(None, Verdict::OK), (Some(Verdict::OK), Verdict::WARN)],
        "the OK -> WARN change settled by the stored evaluations is never recorded"
    );
    assert_eq!(tick.events.len(), 1, "{:?}", tick.events);
}

/// F2. After a restart the resumed hysteresis has an empty window and no newest `ts`, so a
/// re-grade of samples older than the newest transition (`thalamus eval` after a definition
/// revision) is taken as new samples instead of being dropped as late. Three re-graded `OK`
/// samples from before the `WARN` opened settle `WARN` -> `OK` with a `settled_ts` older than
/// the `WARN` transition it clears, so `SeriesTransitions` (settled_ts order) ends on `WARN`, and
/// the next restart resumes the series as `WARN`. Without the restart the same re-grade settles
/// `WARN` -> `OK` at the newest sample.
#[test]
fn adv_regrade_after_restart_does_not_clear_before_the_open() {
    let mut world = World::new("regrade-after-restart");
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK, OK, WARN, WARN]);
    let opened = world.transitions();
    assert_eq!(opened.last().map(|t| t.to_verdict), Some(Verdict::WARN));
    let warn_settled = opened.last().expect("the WARN").settled_ts.clone();
    drop(scheduler);
    let mut scheduler = world.scheduler();
    // A revision under which 150 reads OK, then `thalamus eval` (no `--run`): every stored
    // sample is re-graded at rev 2.
    world.define(2, "1000", "2000");
    {
        let instance = world.load();
        let mut store = world.store();
        evaluate::run(
            &instance,
            &mut store,
            &world.connectors(),
            &[SignalId(DEPTH.to_owned())],
            None,
        )
        .unwrap_or_else(|e| panic!("re-grade: {e}"));
    }
    world.drive(&mut scheduler, &[OK]);
    let all = world.transitions();
    let cleared = all
        .iter()
        .find(|t| t.from_verdict == Some(Verdict::WARN))
        .expect("the WARN is cleared");
    assert!(
        cleared.settled_ts.0 >= warn_settled.0,
        "WARN -> OK settled at {} before the WARN it clears ({}): {all:?}",
        cleared.settled_ts.0,
        warn_settled.0
    );
    assert_eq!(
        all.last().map(|t| t.to_verdict),
        Some(Verdict::OK),
        "the series reads as WARN in settled_ts order: {all:?}"
    );
}

/// F3 (as decided: the first settled verdict from absent is a transition, hysteresis rule 4). One
/// series driven OK -> WARN -> OK records and emits exactly two transitions after the series
/// settled `OK`.
#[test]
fn adv_ok_warn_ok_records_exactly_two_transitions_after_settling() {
    let mut world = World::new("acceptance-two");
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK, OK]);
    assert_eq!(pairs(&world.transitions()), vec![(None, Verdict::OK)]);
    let tick = world.drive(&mut scheduler, &[WARN, WARN, OK, OK, OK]);
    assert_eq!(
        pairs(&world.transitions()),
        vec![
            (None, Verdict::OK),
            (Some(Verdict::OK), Verdict::WARN),
            (Some(Verdict::WARN), Verdict::OK)
        ]
    );
    assert_eq!(tick.events.len(), 2, "{:?}", tick.events);
}
