//! Adversary pass 1 on the scheduler: cases the implementor's suite does not state. Each test runs
//! on a fake clock with an in-process reader, so no source and no Connectors double is involved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration as WallDuration;

use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::{CompleteReading, Reading, RunId, SignalSampleData};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_service::scheduler::{Clock, MISSED_SLOT, READ_TIMED_OUT, Reader, Scheduler};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

const T0: i64 = 1_792_018_500;
const FIVE_MINUTES: i64 = 300;
const INSTANCE: &str = "fixture";

#[derive(Clone)]
struct FakeClock {
    now: Arc<AtomicI64>,
    wall_ms_per_s: u64,
}

impl FakeClock {
    fn at(now: i64, wall_ms_per_s: u64) -> Self {
        Self {
            now: Arc::new(AtomicI64::new(now)),
            wall_ms_per_s,
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
        WallDuration::from_millis(u64::try_from(seconds).unwrap_or(0) * self.wall_ms_per_s)
    }

    fn sleep_until(&self, at: i64) {
        self.set(at);
    }
}

/// Answers `value` for every read, after the per-signal delay; logs `(signal, end)` per call.
#[derive(Clone, Default)]
struct Answering {
    delays: BTreeMap<String, WallDuration>,
    value: &'static str,
    calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl Reader for Answering {
    fn read(&self, definition: &SignalDefinitionData, end: &Timestamp, _: &RunId) -> Vec<Read> {
        self.calls
            .lock()
            .expect("log")
            .push((definition.signal_id.0.clone(), end.0.clone()));
        if let Some(delay) = self.delays.get(&definition.signal_id.0) {
            std::thread::sleep(*delay);
        }
        let mut read = unknown(definition, end, "");
        read.reading = Reading::Ok(CompleteReading {
            value: Decimal(self.value.to_owned()),
        });
        vec![read]
    }
}

fn definition(id: &str, cadence: &str) -> String {
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
cadence: {cadence}
history: store
denominator: {{ unit: \"{{message}}\", min_n: 0, since: 2026-01-01T00:00:00Z }}
evaluation:
  rule: higher_is_worse
  baselines: [fixed]
  fixed: \"0\"
  warn: {{ abs: \"100\" }}
  crit: {{ abs: \"1000\" }}
owner: team:service
lifecycle: active
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

/// An instance directory holding `definitions` (id, cadence), and its store's locator.
fn world(name: &str, definitions: &[(&str, &str)]) -> (PathBuf, Locator) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_scheduler_p1")
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
    for (id, cadence) in definitions {
        std::fs::write(
            instance.join("definitions").join(format!("{id}.yaml")),
            definition(id, cadence),
        )
        .expect("write a definition");
    }
    (instance, Locator::Sqlite(root.join("store.sqlite")))
}

fn samples(locator: &Locator) -> Vec<SignalSampleData> {
    EventlogStore::open(locator, INSTANCE)
        .expect("the store opens")
        .samples()
        .to_vec()
}

fn reason(sample: &SignalSampleData) -> Option<&str> {
    match &sample.reading {
        Reading::Unknown(unread) => Some(unread.reason.as_str()),
        _ => None,
    }
}

/// Design § 2.2: a read that has not finished within *its own* cadence is `unknown` with reason
/// `read timed out`. A PT1M read that takes longer than one minute must time out even when the
/// scheduler happens to wait first on a PT5M read that is slower still but inside its budget.
#[test]
fn late_short_cadence_read_times_out_even_behind_a_longer_wait() {
    const LONG: &str = "service.queue.alpha";
    const SHORT: &str = "service.queue.omega";
    let (instance, locator) = world("timeout_race", &[(LONG, "PT5M"), (SHORT, "PT1M")]);
    // 10 wall ms per clock second: PT1M is 600 ms, PT5M is 3 s.
    let clock = FakeClock::at(T0 + 1, 10);
    let reader = Answering {
        delays: BTreeMap::from([
            (LONG.to_owned(), WallDuration::from_millis(2_000)),
            (SHORT.to_owned(), WallDuration::from_millis(1_200)),
        ]),
        value: "0",
        ..Answering::default()
    };
    let mut scheduler = Scheduler::open(
        &instance,
        &locator,
        clock,
        reader,
        Connectors::new(instance.join("no-connectors")),
    )
    .expect("the scheduler opens");
    scheduler.tick();

    let samples = samples(&locator);
    let short = samples
        .iter()
        .find(|sample| sample.signal.0 == SHORT)
        .expect("the PT1M definition has a sample");
    assert_eq!(
        reason(short),
        Some(READ_TIMED_OUT),
        "a PT1M read that took 1.2 s of a 600 ms budget is recorded as its late answer: {short:#?}"
    );
    let long = samples
        .iter()
        .find(|sample| sample.signal.0 == LONG)
        .expect("the PT5M definition has a sample");
    assert!(matches!(long.reading, Reading::Ok(_)), "{long:#?}");
}

/// D16 (story: "every slot of an Active definition since its newest stored sample"): after the
/// clock steps backwards and then forwards again, no `missed_slot` sample is written for a slot
/// older than the newest stored sample; those slots were never missed by the service.
#[test]
fn backwards_clock_jump_writes_no_missed_slot_behind_the_newest_sample() {
    const DEPTH: &str = "service.queue.depth";
    let (instance, locator) = world("backwards", &[(DEPTH, "PT5M")]);
    let s3 = T0 + 3 * FIVE_MINUTES;
    let s4 = T0 + 4 * FIVE_MINUTES;
    let clock = FakeClock::at(s3 + 1, 1_000);
    let reader = Answering {
        value: "7",
        ..Answering::default()
    };
    let mut scheduler = Scheduler::open(
        &instance,
        &locator,
        clock.clone(),
        reader,
        Connectors::new(instance.join("no-connectors")),
    )
    .expect("the scheduler opens");
    scheduler.tick();
    // An NTP step or a VM resume moves the clock back three slots, then forward again.
    clock.set(T0 + 1);
    scheduler.tick();
    clock.set(s4 + 1);
    scheduler.tick();

    let samples = samples(&locator);
    let behind: Vec<(&str, Option<&str>)> = samples
        .iter()
        .filter(|sample| reason(sample) == Some(MISSED_SLOT))
        .map(|sample| (sample.ts.0.as_str(), reason(sample)))
        .collect();
    assert!(
        behind.is_empty(),
        "missed_slot samples written behind the newest stored sample ({s3}): {behind:?}"
    );
}
