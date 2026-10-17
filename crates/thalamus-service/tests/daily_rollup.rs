//! The service's daily rollup (design § 4, lesson L7): after each UTC midnight the scheduler
//! rolls up the day that closed, once. On a fake clock, against the fixture instance
//! (`tests/fixtures/scheduler`) and an in-process reader that answers 7 for every read.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration as WallDuration;

use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::evaluation::Verdict;
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

#[test]
fn service_rolls_up_closed_day() {
    let (instance, locator) = world("closed-day");
    let clock = FakeClock::at(MIDNIGHT - 2 * FIVE_MINUTES);
    let mut scheduler = Scheduler::open(
        &instance,
        &locator,
        clock.clone(),
        Seven,
        Connectors::new(instance.join("no-connectors")),
    )
    .expect("the scheduler opens");

    // 23:50 and 23:55 of 2026-10-15: sampled and graded; the day is open, nothing is rolled up.
    for at in [MIDNIGHT - 2 * FIVE_MINUTES, MIDNIGHT - FIVE_MINUTES] {
        clock.set(at);
        let tick = scheduler.tick();
        assert!(tick.problems.is_empty(), "{:?}", tick.problems);
    }
    let held = store(&locator);
    assert_eq!(held.samples().len(), 2);
    assert_eq!(held.evaluations().len(), 2);
    assert!(held.rollups().is_empty(), "an open day is not rolled up");

    // The first tick after midnight rolls up the day that closed.
    clock.set(MIDNIGHT);
    let tick = scheduler.tick();
    assert!(tick.problems.is_empty(), "{:?}", tick.problems);
    let rows = store(&locator).rollups();
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    assert_eq!(row.rollup_id.0, "service.queue.depth@2026-10-15");
    assert_eq!(row.series.0, DEPTH);
    assert_eq!(row.day.0, "2026-10-15T00:00:00Z");
    assert_eq!((row.n, row.unknown, row.partial), (2, 0, 0));
    assert_eq!(row.last, Some(Decimal("7".to_owned())));
    assert_eq!(row.worst_verdict, Some(Verdict::OK));
    assert_eq!(row.verdict_counts.ok, 2);

    // Later ticks of the next day write no second row and leave the first as it was.
    for at in [MIDNIGHT + FIVE_MINUTES, MIDNIGHT + 2 * FIVE_MINUTES] {
        clock.set(at);
        let tick = scheduler.tick();
        assert!(tick.problems.is_empty(), "{:?}", tick.problems);
    }
    let after = store(&locator).rollups();
    assert_eq!(after, rows, "the closed day is rolled up once");
    assert_eq!(store(&locator).samples().len(), 5);
}
