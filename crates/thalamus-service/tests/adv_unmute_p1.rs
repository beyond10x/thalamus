//! Adversary pass 1 on story:m2-auto-unmute. Helpers copied from tests/unmute.rs.
//! An expired mute (design § 11 D9, D19, D28): the service samples a `Muted` definition whose
//! `mute_until` has passed as `Active`, records `DefinitionUnmuted` once in its store as actor
//! `thalamus.definition.Service`, warns about the stale file on every pass, and never rewrites
//! the file. On a fake clock, against the fixture instance (`tests/fixtures/scheduler`) and the
//! Connectors double (`thalamus-sources`, `src/bin/connectors-double.rs`).

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Barrier, OnceLock};
use std::time::Duration as WallDuration;

use serde_json::{Value, json};
use thalamus_model::definition::SignalId;
use thalamus_model::observation::{Reading, SignalSampleData};
use thalamus_service::scheduler::{Clock, ConnectorsReader, MISSED_SLOT, Scheduler};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

/// One day, in seconds.
const DAY: i64 = 86_400;
/// A slot boundary of `PT5M`.
const T0: i64 = 1_792_018_500;
/// `PT5M`, in seconds.
const FIVE_MINUTES: i64 = 300;

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

const DEPTH: &str = "service.queue.depth";
const QUEUE: &str = "sum(queue_depth)";
const LATER: &str = "service.queue.later";
const LATER_QUEUE: &str = "sum(later_depth)";

/// The Connectors double, built once for the whole test binary beside the service binary.
fn double() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let status = Command::new(env!("CARGO"))
                .args([
                    "build",
                    "--locked",
                    "--quiet",
                    "-p",
                    "thalamus-sources",
                    "--bin",
                    "connectors-double",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .status()
                .expect("cargo runs");
            assert!(status.success(), "the Connectors double builds");
            let bin = Path::new(env!("CARGO_BIN_EXE_thalamus-service"))
                .with_file_name("connectors-double");
            assert!(bin.exists(), "{} exists", bin.display());
            bin
        })
        .clone()
}

/// The RFC 3339 text of Unix second `at`, in UTC.
fn rfc3339(at: i64) -> String {
    let days = at.div_euclid(DAY);
    let second = at.rem_euclid(DAY);
    // Civil from days (Howard Hinnant's algorithm), for the proleptic Gregorian calendar.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        second / 3600,
        second % 3600 / 60,
        second % 60
    )
}

/// A clock the test sets.
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

/// One test's directory: the instance copy, the store, the recordings and the call log.
struct World {
    root: PathBuf,
    answers: Vec<Value>,
}

impl World {
    fn new(name: &str, definitions: &[(&str, String)]) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("service_adv_unmute_p1")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scheduler"),
            &root.join("instance"),
        );
        let world = Self {
            root,
            answers: Vec::new(),
        };
        std::fs::create_dir_all(world.instance().join("definitions"))
            .expect("create the definitions directory");
        for (id, text) in definitions {
            std::fs::write(world.file(id), text).expect("write a definition");
        }
        std::fs::create_dir_all(world.recordings()).expect("create the recordings directory");
        std::os::unix::fs::symlink(double(), world.connectors()).expect("link the double");
        world.write(
            "connections+list+a=metrics.json",
            &json!({"stdout": {"ok": true, "result": {"connections": [
                {"adapter": "metrics", "connection": "metrics-main", "state": "ready", "revision": "conn-rev-1"}
            ]}}}),
        );
        world.write(
            "operations+describe+a=metrics+o=series.query.json",
            &json!({"stdout": {"ok": true, "result": {
                "adapter": "metrics", "operation": "series.query",
                "schema": "series.query/v1alpha1", "revision": "op-rev-1"
            }}}),
        );
        world.flush();
        world
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn file(&self, id: &str) -> PathBuf {
        self.instance()
            .join("definitions")
            .join(format!("{id}.yaml"))
    }

    fn recordings(&self) -> PathBuf {
        self.root.join("connectors")
    }

    fn connectors(&self) -> PathBuf {
        self.recordings().join("connectors")
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.root.join("store.sqlite"))
    }

    fn write(&self, name: &str, answer: &Value) {
        std::fs::write(
            self.recordings().join(name),
            serde_json::to_string_pretty(answer).expect("JSON"),
        )
        .expect("write a recording");
    }

    fn flush(&self) {
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(self.answers.clone()),
        );
    }

    /// `query` answers `value` at `at`, one series with no labels.
    fn answer(&mut self, query: &str, at: i64, value: &str) {
        let document = json!({
            "result_type": "vector", "series_truncated": false, "complete": true,
            "warnings": [], "infos": [], "time_unix_s": at,
            "provenance": {"instance": "prometheus-test", "resource": "prometheus:query",
                           "observed_at_unix_ms": 0, "source_revision": null},
            "series": [{"labels": {}, "samples": [[at, value]], "samples_truncated": false}]
        });
        self.answers.push(json!({
            "input": {"query": query, "time_unix_s": at},
            "stdout": {"ok": true, "result": {
                "adapter": "metrics", "operation": "series.query", "revision": "op-rev-1",
                "result": document.to_string()
            }}
        }));
        self.flush();
    }

    fn scheduler(&self, clock: &FakeClock) -> Scheduler<FakeClock, ConnectorsReader> {
        Scheduler::open(
            &self.instance(),
            &self.locator(),
            clock.clone(),
            ConnectorsReader(Connectors::new(self.connectors())),
            Connectors::new(self.connectors()),
        )
        .unwrap_or_else(|error| panic!("the scheduler opens: {error:?}"))
    }

    fn store(&self) -> EventlogStore {
        EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens")
    }

    fn samples(&self, signal: &str) -> Vec<SignalSampleData> {
        self.store()
            .samples()
            .iter()
            .filter(|sample| sample.signal.0 == signal)
            .cloned()
            .collect()
    }
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

const HOURLY: &str = "service.queue.hourly";
const HOURLY_QUEUE: &str = "sum(hourly_depth)";
const HOUR: i64 = 3_600;

/// A `PT5M` (or `cadence`) gauge with the given lifecycle block, `rev` and `revised_at`.
fn definition(
    id: &str,
    query: &str,
    cadence: &str,
    lifecycle: &str,
    rev: i64,
    revised_at: i64,
) -> String {
    format!(
        "id: {id}
title: A signal of the adversary unmute tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: []
source: {{ adapter: metrics, connection: metrics-main }}
extractor: promql.value
query: {{ value: \"{query}\" }}
window: {cadence}
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
{lifecycle}
rev: {rev}
revised_at: {}
revised_by: operator
",
        rfc3339(revised_at)
    )
}

fn muted_until(until: i64) -> String {
    format!(
        "lifecycle: muted\nmute_reason: maintenance\nmute_until: {}",
        rfc3339(until)
    )
}

fn reason(sample: &SignalSampleData) -> Option<&str> {
    match &sample.reading {
        Reading::Unknown(unread) => Some(unread.reason.as_str()),
        _ => None,
    }
}

/// The running service wakes at `next_slot`, which reads the file lifecycle: an expired mute
/// beside an hourly Active definition is not woken for, and its 5-minute slots between the hourly
/// ticks are recorded as `missed_slot` though nothing stopped the service from sampling them.
#[test]
fn adv_expired_mute_beside_hourly_definition_is_sampled_each_slot() {
    let mut world = World::new(
        "adv_expired_mute_beside_hourly_definition_is_sampled_each_slot",
        &[
            (
                DEPTH,
                definition(
                    DEPTH,
                    QUEUE,
                    "PT5M",
                    &muted_until(T0 - FIVE_MINUTES),
                    3,
                    T0 - DAY,
                ),
            ),
            (
                HOURLY,
                definition(
                    HOURLY,
                    HOURLY_QUEUE,
                    "PT1H",
                    "lifecycle: active",
                    1,
                    T0 - DAY,
                ),
            ),
        ],
    );
    for slot in -12..=14 {
        world.answer(QUEUE, T0 + slot * FIVE_MINUTES, "7");
    }
    for at in [
        T0 - 11 * FIVE_MINUTES,
        T0 + FIVE_MINUTES,
        T0 + FIVE_MINUTES + HOUR,
    ] {
        world.answer(HOURLY_QUEUE, at, "1");
    }
    let clock = FakeClock::at(T0);
    let mut scheduler = world.scheduler(&clock);
    let ticks = Cell::new(0);
    scheduler.run(|| {
        ticks.set(ticks.get() + 1);
        // Thirteen ticks: one per 5-minute slot from T0 through the hour.
        ticks.get() > 13
    });
    assert!(
        clock.now() >= T0 + HOUR,
        "the run reached the next hour: {}",
        clock.now()
    );
    let missed: Vec<String> = world
        .samples(DEPTH)
        .iter()
        .filter(|sample| reason(sample) == Some(MISSED_SLOT))
        .map(|sample| sample.ts.0.clone())
        .collect();
    assert!(
        missed.is_empty(),
        "the running service recorded {} slots of the expired mute as missed: {missed:?}",
        missed.len()
    );
}

/// D47 floors the first-tick fill at `revised_at`; the auto-unmute leaves the file (and its
/// `revised_at`, the mute's) unchanged, so a restart past the expiry fills the muted span with
/// `missed_slot` samples.
#[test]
fn adv_restart_past_expiry_does_not_fill_the_muted_span() {
    let muted_at = T0 - 4 * HOUR;
    let until = T0 - 2 * HOUR;
    let mut world = World::new(
        "adv_restart_past_expiry_does_not_fill_the_muted_span",
        &[(
            DEPTH,
            definition(
                DEPTH,
                QUEUE,
                "PT5M",
                "lifecycle: active",
                2,
                muted_at - HOUR,
            ),
        )],
    );
    world.answer(QUEUE, muted_at - FIVE_MINUTES, "7");
    world.answer(QUEUE, T0, "7");
    let clock = FakeClock::at(muted_at - FIVE_MINUTES);
    world.scheduler(&clock).tick();
    assert_eq!(world.samples(DEPTH).len(), 1, "one sample before the mute");

    std::fs::write(
        world.file(DEPTH),
        definition(DEPTH, QUEUE, "PT5M", &muted_until(until), 3, muted_at),
    )
    .expect("mute the definition");
    clock.set(T0);
    world.scheduler(&clock).tick();

    let muted_span: Vec<String> = world
        .samples(DEPTH)
        .iter()
        .filter(|sample| reason(sample) == Some(MISSED_SLOT))
        .map(|sample| sample.ts.0.clone())
        // RFC 3339 in one UTC format sorts as its instants do.
        .filter(|ts| *ts >= rfc3339(muted_at) && *ts < rfc3339(until))
        .collect();
    assert!(
        muted_span.is_empty(),
        "{} slots of the muted span were recorded missed_slot: {muted_span:?}",
        muted_span.len()
    );
}

/// One `DefinitionUnmuted` across two services on one store and a restart.
#[test]
fn adv_unmute_recorded_once_across_two_services_and_a_restart() {
    let mut world = World::new(
        "adv_unmute_recorded_once_across_two_services_and_a_restart",
        &[(
            DEPTH,
            definition(
                DEPTH,
                QUEUE,
                "PT5M",
                &muted_until(T0 - FIVE_MINUTES),
                3,
                T0 - DAY,
            ),
        )],
    );
    for slot in 0..3 {
        world.answer(QUEUE, T0 + slot * FIVE_MINUTES, "7");
    }
    let world = Arc::new(world);
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let world = Arc::clone(&world);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let clock = FakeClock::at(T0);
                let mut scheduler = world.scheduler(&clock);
                barrier.wait();
                scheduler.tick()
            })
        })
        .collect();
    let ticks: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("a tick"))
        .collect();
    let clock = FakeClock::at(T0 + FIVE_MINUTES);
    let mut restarted = world.scheduler(&clock);
    restarted.tick();
    clock.set(T0 + 2 * FIVE_MINUTES);
    restarted.tick();
    let events = world.store().definition_events(&SignalId(DEPTH.to_owned()));
    assert_eq!(
        events.len(),
        1,
        "one DefinitionUnmuted: {events:?}; ticks {ticks:?}"
    );
    for tick in &ticks {
        assert!(
            !tick.problems.iter().any(|p| p.contains("not recorded")),
            "{:?}",
            tick.problems
        );
    }
}

/// A mute set again after the expiry, under a new revision, is honoured, and its own expiry is
/// recorded as a second unmute.
#[test]
fn adv_mute_set_again_after_expiry_is_honoured() {
    let mut world = World::new(
        "adv_mute_set_again_after_expiry_is_honoured",
        &[(
            DEPTH,
            definition(
                DEPTH,
                QUEUE,
                "PT5M",
                &muted_until(T0 - FIVE_MINUTES),
                3,
                T0 - DAY,
            ),
        )],
    );
    for slot in 0..3 {
        world.answer(QUEUE, T0 + slot * FIVE_MINUTES, "7");
    }
    let clock = FakeClock::at(T0);
    let mut scheduler = world.scheduler(&clock);
    scheduler.tick();
    std::fs::write(
        world.file(DEPTH),
        definition(
            DEPTH,
            QUEUE,
            "PT5M",
            &muted_until(T0 + 2 * FIVE_MINUTES),
            4,
            T0 + 60,
        ),
    )
    .expect("mute again");
    clock.set(T0 + FIVE_MINUTES);
    let second = scheduler.tick();
    assert!(
        second.runs.is_empty(),
        "the re-muted definition is not sampled: {second:?}"
    );
    clock.set(T0 + 2 * FIVE_MINUTES);
    scheduler.tick();
    let stamps: Vec<String> = world
        .samples(DEPTH)
        .iter()
        .map(|s| s.ts.0.clone())
        .collect();
    assert_eq!(stamps, vec![rfc3339(T0), rfc3339(T0 + 2 * FIVE_MINUTES)]);
    let events = world.store().definition_events(&SignalId(DEPTH.to_owned()));
    assert_eq!(
        events.iter().map(|e| e.rev).collect::<Vec<_>>(),
        vec![3, 4],
        "{events:?}"
    );
}

/// `mute_until` exactly at the tick instant is expired; one second later is not.
#[test]
fn adv_mute_until_exactly_at_the_tick_is_expired() {
    let mut world = World::new(
        "adv_mute_until_exactly_at_the_tick_is_expired",
        &[
            (
                DEPTH,
                definition(DEPTH, QUEUE, "PT5M", &muted_until(T0), 3, T0 - DAY),
            ),
            (
                LATER,
                definition(
                    LATER,
                    LATER_QUEUE,
                    "PT5M",
                    &muted_until(T0 + 1),
                    3,
                    T0 - DAY,
                ),
            ),
        ],
    );
    world.answer(QUEUE, T0, "7");
    world.answer(LATER_QUEUE, T0, "7");
    let clock = FakeClock::at(T0);
    world.scheduler(&clock).tick();
    assert_eq!(world.samples(DEPTH).len(), 1);
    assert!(world.samples(LATER).is_empty());
}
