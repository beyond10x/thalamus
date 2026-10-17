//! Adversary pass 1 of the service daily rollup: a tick that comes late after midnight, and a
//! midnight the service never saw. Destination: crates/thalamus-service/tests/.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration as WallDuration;

use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_service::scheduler::{Clock, Reader, Scheduler};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

/// 2026-10-16T00:00:00Z: a UTC midnight, and a slot boundary of `PT5M`.
const MIDNIGHT: i64 = 1_792_108_800;
const FIVE_MINUTES: i64 = 300;
const INSTANCE: &str = "fixture";
const DEPTH: &str = "service.queue.depth";

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
        WallDuration::from_secs(u64::try_from(seconds).unwrap_or(0))
    }

    fn sleep_until(&self, at: i64) {
        self.set(at);
    }
}

/// Answers 7 for every read: below the fixed baseline's `warn` of 100, so graded `OK`.
struct Seven;

impl Reader for Seven {
    fn read(&self, definition: &SignalDefinitionData, end: &Timestamp, _: &RunId) -> Vec<Read> {
        let mut read = unknown(definition, end, "");
        read.reading = Reading::Ok(CompleteReading {
            value: Decimal("7".to_owned()),
        });
        vec![read]
    }
}

fn definition() -> String {
    format!(
        "id: {DEPTH}
title: A signal of the daily rollup tests
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

/// An instance directory holding the depth definition, and its store's locator.
fn world(name: &str) -> (PathBuf, Locator) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("daily_rollup")
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
    std::fs::write(
        instance.join("definitions").join(format!("{DEPTH}.yaml")),
        definition(),
    )
    .expect("write a definition");
    (instance, Locator::Sqlite(root.join("store.sqlite")))
}

fn store(locator: &Locator) -> EventlogStore {
    EventlogStore::open(locator, INSTANCE).expect("the store opens")
}

const DAY: i64 = 86_400;

fn scheduler(instance: &Path, locator: &Locator, clock: &FakeClock) -> Scheduler<FakeClock, Seven> {
    Scheduler::open(
        instance,
        locator,
        clock.clone(),
        Seven,
        Connectors::new(instance.join("no-connectors")),
    )
    .expect("the scheduler opens")
}

/// The service misses 23:50, 23:55 and 00:00 (a host asleep, a restart) and ticks at 00:05. The
/// tick fills 23:50 and 23:55 of 2026-10-15 as `unknown` samples, so the day holds three samples,
/// two unknown. The row of 2026-10-15 is immutable (`already-rolled-up`), so it must count them.
#[test]
fn adv_late_tick_after_midnight_counts_the_missed_slots_of_the_closed_day() {
    let (instance, locator) = world("adv-late-tick");
    let clock = FakeClock::at(MIDNIGHT - 3 * FIVE_MINUTES);
    let mut scheduler = scheduler(&instance, &locator, &clock);
    scheduler.tick(); // 23:45, sampled
    for at in [MIDNIGHT + FIVE_MINUTES, MIDNIGHT + 2 * FIVE_MINUTES] {
        clock.set(at);
        scheduler.tick();
    }
    let held = store(&locator);
    let closed: Vec<_> = held
        .samples()
        .iter()
        .filter(|sample| sample.ts.0.starts_with("2026-10-15"))
        .cloned()
        .collect();
    assert_eq!(
        closed.len(),
        3,
        "23:45 read, 23:50 and 23:55 filled: {closed:?}"
    );
    let rows = held.rollups();
    let row = rows
        .iter()
        .find(|row| row.rollup_id.0 == "service.queue.depth@2026-10-15")
        .expect("the closed day is rolled up");
    assert_eq!(
        (row.n, row.unknown, row.partial),
        (3, 2, 0),
        "the rollup of 2026-10-15 counts every sample the day holds: {row:?}"
    );
    assert_eq!(row.verdict_counts.unknown, 2, "{row:?}");
}

/// The service is down from 2026-10-15 23:55 to 2026-10-17 00:05: the midnight closing
/// 2026-10-15 passes unseen. The day holds a sample and must still be rolled up once the
/// service runs again ("after each UTC midnight, roll up the day that closed").
#[test]
fn adv_service_rolls_up_a_day_whose_midnight_it_missed() {
    let (instance, locator) = world("adv-missed-midnight");
    let clock = FakeClock::at(MIDNIGHT - FIVE_MINUTES);
    let mut scheduler = scheduler(&instance, &locator, &clock);
    scheduler.tick(); // 2026-10-15 23:55, sampled
    for at in [
        MIDNIGHT + DAY + FIVE_MINUTES,
        MIDNIGHT + DAY + 2 * FIVE_MINUTES,
    ] {
        clock.set(at);
        scheduler.tick();
    }
    let ids: Vec<String> = store(&locator)
        .rollups()
        .into_iter()
        .map(|row| row.rollup_id.0)
        .collect();
    assert!(
        ids.iter().any(|id| id == "service.queue.depth@2026-10-15"),
        "2026-10-15 holds a sample and closed while the service was down: {ids:?}"
    );
}
