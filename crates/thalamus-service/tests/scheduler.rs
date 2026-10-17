//! The scheduler of `thalamus-service` (design § 5.3, § 11 D16): on a fake clock, against the
//! fixture instance (`tests/fixtures/scheduler`, with the definitions each test writes) and the
//! Connectors double (`thalamus-sources`, `src/bin/connectors-double.rs`). No source is reachable.
//!
//! Each test gets its own directory under the build's temporary directory: the instance copy, a
//! SQLite store, and the double linked as `connectors` beside its recordings, so the double reads
//! its recordings and writes its call log there without any environment variable.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration as WallDuration;

use serde_json::{Value, json};
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{CompleteReading, Reading, RunId, SignalSampleData};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_service::scheduler::{
    Clock, ConnectorsReader, FIRST_TICK_FILL_SLOTS, MISSED_SLOT, READ_TIMED_OUT, Reader, Scheduler,
};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};

/// One day, in seconds.
const DAY: i64 = 86_400;

/// A slot boundary of every cadence the tests use (a multiple of 300 s and of 60 s).
const T0: i64 = 1_792_018_500;
/// `PT5M`, in seconds.
const FIVE_MINUTES: i64 = 300;

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

const DEPTH: &str = "service.queue.depth";
const BUSY: &str = "service.worker.busy";
const SLOW: &str = "service.queue.slow";

const QUEUE: &str = "sum(queue_depth)";
const WORKERS: &str = "sum by (component) (worker_busy)";

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

/// A clock the test sets. `span` maps one of its seconds to `wall_ms_per_s` wall milliseconds, so
/// a cadence budget is short enough to wait out in a test.
#[derive(Clone)]
struct FakeClock {
    now: Arc<AtomicI64>,
    wall_ms_per_s: u64,
}

impl FakeClock {
    fn at(now: i64) -> Self {
        Self {
            now: Arc::new(AtomicI64::new(now)),
            wall_ms_per_s: 1_000,
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

/// A reader whose reads of `signal` take `delay` and then answer 0; every other read goes to
/// `inner`.
struct Slow<R> {
    inner: R,
    signal: &'static str,
    delay: WallDuration,
}

impl<R: Reader> Reader for Slow<R> {
    fn read(
        &self,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        run_id: &RunId,
    ) -> Vec<Read> {
        if definition.signal_id.0 != self.signal {
            return self.inner.read(definition, end, run_id);
        }
        std::thread::sleep(self.delay);
        let mut read = unknown(definition, end, "");
        read.reading = Reading::Ok(CompleteReading {
            value: Decimal("0".to_owned()),
        });
        vec![read]
    }
}

/// A definition file: a `promql.value` gauge over `metrics-main`, graded against a fixed 0.
fn definition(
    id: &str,
    query: &str,
    dimensions: &str,
    cadence: &str,
    lifecycle: &str,
    rev: i64,
) -> String {
    format!(
        "id: {id}
title: A signal of the scheduler tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: {dimensions}
source: {{ adapter: metrics, connection: metrics-main }}
extractor: promql.value
query: {{ value: \"{query}\" }}
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
{lifecycle}
rev: {rev}
revised_at: 2026-10-02T00:00:00Z
revised_by: operator
"
    )
}

const ACTIVE: &str = "lifecycle: active";

fn depth(rev: i64, query: &str) -> (String, String) {
    (
        DEPTH.to_owned(),
        definition(DEPTH, query, "[]", "PT5M", ACTIVE, rev),
    )
}

fn busy() -> (String, String) {
    (
        BUSY.to_owned(),
        definition(BUSY, WORKERS, "[component]", "PT5M", ACTIVE, 1),
    )
}

/// One test's directory: the instance copy, the store, the recordings and the call log.
struct World {
    root: PathBuf,
    /// The `series.query` answers of `metrics-main`, one per query and instant.
    answers: Vec<Value>,
}

impl World {
    fn new(name: &str, definitions: &[(String, String)]) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("service_scheduler")
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
            world.define(id, text);
        }
        std::fs::create_dir_all(world.recordings()).expect("create the recordings directory");
        std::os::unix::fs::symlink(double(), world.connectors()).expect("link the double");
        world.record_connections();
        world.flush();
        world
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn recordings(&self) -> PathBuf {
        self.root.join("connectors")
    }

    /// The double, linked into its recordings directory: it reads its recordings and writes its
    /// call log beside the path it was started from.
    fn connectors(&self) -> PathBuf {
        self.recordings().join("connectors")
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.root.join("store.sqlite"))
    }

    fn define(&self, id: &str, text: &str) {
        std::fs::write(
            self.instance()
                .join("definitions")
                .join(format!("{id}.yaml")),
            text,
        )
        .expect("write a definition");
    }

    fn write(&self, name: &str, answer: &Value) {
        std::fs::write(
            self.recordings().join(name),
            serde_json::to_string_pretty(answer).expect("JSON"),
        )
        .expect("write a recording");
    }

    fn record_connections(&self) {
        self.write(
            "connections+list+a=metrics.json",
            &json!({"stdout": {"ok": true, "result": {"connections": [
                {"adapter": "metrics", "connection": "metrics-main", "state": "ready", "revision": "conn-rev-1"}
            ]}}}),
        );
        self.write(
            "operations+describe+a=metrics+o=series.query.json",
            &json!({"stdout": {"ok": true, "result": {
                "adapter": "metrics", "operation": "series.query",
                "schema": "series.query/v1alpha1", "revision": "op-rev-1"
            }}}),
        );
    }

    fn flush(&self) {
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(self.answers.clone()),
        );
    }

    /// `query` answers `value` at `at`, one series with no labels.
    fn answer(&mut self, query: &str, at: i64, value: &str) {
        self.answers.push(vector(query, at, &[("", value)], ""));
        self.flush();
    }

    /// The busy workers at `at`: `checkout` and `search`.
    fn workers(&mut self, at: i64, checkout: &str, search: &str) {
        self.answers.push(vector(
            WORKERS,
            at,
            &[("checkout", checkout), ("search", search)],
            "component",
        ));
        self.flush();
    }

    fn scheduler(&self, clock: &FakeClock) -> Scheduler<FakeClock, ConnectorsReader> {
        self.scheduler_with(clock, ConnectorsReader(Connectors::new(self.connectors())))
    }

    fn scheduler_with<R: Reader>(&self, clock: &FakeClock, reader: R) -> Scheduler<FakeClock, R> {
        Scheduler::open(
            &self.instance(),
            &self.locator(),
            clock.clone(),
            reader,
            Connectors::new(self.connectors()),
        )
        .unwrap_or_else(|error| panic!("the scheduler opens: {error:?}"))
    }

    fn store(&self) -> EventlogStore {
        EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens")
    }

    fn samples(&self) -> Vec<SignalSampleData> {
        self.store().samples().to_vec()
    }

    fn evaluations(&self) -> Vec<SignalEvaluationData> {
        self.store().evaluations().to_vec()
    }

    /// `(query, time_unix_s)` of every `series.query` invoke the double received, in call order.
    fn invoked(&self) -> Vec<(String, i64)> {
        std::fs::read_to_string(self.recordings().join("calls.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a log line is JSON"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|arg| arg.as_str().map(str::to_owned))
                    .collect();
                if !argv.starts_with(&["operations".to_owned(), "invoke".to_owned()]) {
                    return None;
                }
                let at = argv.iter().position(|arg| arg == "--input-json")?;
                let input: Value = serde_json::from_str(argv.get(at + 1)?).ok()?;
                Some((
                    input["query"].as_str()?.to_owned(),
                    input["time_unix_s"].as_i64()?,
                ))
            })
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

/// A `series.query` answer to `query` at `at`: one series per `(label value, value)`, labelled
/// `label` (no labels where `label` is empty).
fn vector(query: &str, at: i64, series: &[(&str, &str)], label: &str) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(labelled, value)| {
            let labels = if label.is_empty() {
                json!({})
            } else {
                json!({ label: labelled })
            };
            json!({"labels": labels, "samples": [[at, value]], "samples_truncated": false})
        })
        .collect();
    let document = json!({
        "result_type": "vector", "series_truncated": false, "complete": true,
        "warnings": [], "infos": [], "time_unix_s": at,
        "provenance": {"instance": "prometheus-test", "resource": "prometheus:query",
                       "observed_at_unix_ms": 0, "source_revision": null},
        "series": series
    });
    json!({
        "input": {"query": query, "time_unix_s": at},
        "stdout": {"ok": true, "result": {
            "adapter": "metrics", "operation": "series.query", "revision": "op-rev-1",
            "result": document.to_string()
        }}
    })
}

fn checkout() -> String {
    format!("{BUSY}{{component=\"checkout\"}}")
}

fn search() -> String {
    format!("{BUSY}{{component=\"search\"}}")
}

/// `(series, ts)` of every sample, sorted.
fn slots(samples: &[SignalSampleData]) -> Vec<(String, String)> {
    let mut slots: Vec<(String, String)> = samples
        .iter()
        .map(|sample| (sample.series.0.clone(), sample.ts.0.clone()))
        .collect();
    slots.sort();
    slots
}

/// `(series, ts)` of each series of `depth` and `busy` at each of `at`, sorted.
fn every_series_at(at: &[i64]) -> Vec<(String, String)> {
    let mut slots: Vec<(String, String)> = at
        .iter()
        .flat_map(|at| {
            [DEPTH.to_owned(), checkout(), search()]
                .into_iter()
                .map(move |series| (series, rfc3339(*at)))
        })
        .collect();
    slots.sort();
    slots
}

fn reason(sample: &SignalSampleData) -> Option<&str> {
    match &sample.reading {
        Reading::Unknown(unread) => Some(unread.reason.as_str()),
        _ => None,
    }
}

#[test]
fn rfc3339_writes_the_instants_the_tests_use() {
    assert_eq!(rfc3339(T0), "2026-10-14T22:55:00Z");
    assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
    assert_eq!(T0 % FIVE_MINUTES, 0);
}

/// Slots are multiples of the cadence from the Unix epoch: however often the scheduler ticks
/// inside a slot, each series of each `Active` definition gets one sample per slot, stamped at the
/// slot, read once, and graded once.
#[test]
fn samples_each_active_definition_once_per_cadence_slot() {
    let mut world = World::new("once_per_slot", &[depth(1, QUEUE), busy()]);
    for slot in [T0, T0 + FIVE_MINUTES] {
        world.answer(QUEUE, slot, "7");
        world.workers(slot, "3", "4");
    }
    let clock = FakeClock::at(T0 + 7);
    let mut scheduler = world.scheduler(&clock);
    for at in [
        T0 + 7,
        T0 + 120,
        T0 + FIVE_MINUTES + 3,
        T0 + FIVE_MINUTES + 299,
    ] {
        clock.set(at);
        scheduler.tick();
    }

    let samples = world.samples();
    assert_eq!(
        slots(&samples),
        every_series_at(&[T0, T0 + FIVE_MINUTES]),
        "{samples:#?}"
    );
    assert!(
        samples
            .iter()
            .all(|sample| matches!(sample.reading, Reading::Ok(_))),
        "{samples:#?}"
    );
    let mut invoked = world.invoked();
    invoked.sort();
    let mut expected = vec![
        (QUEUE.to_owned(), T0),
        (QUEUE.to_owned(), T0 + FIVE_MINUTES),
        (WORKERS.to_owned(), T0),
        (WORKERS.to_owned(), T0 + FIVE_MINUTES),
    ];
    expected.sort();
    assert_eq!(invoked, expected, "one read per definition and slot");

    let evaluations = world.evaluations();
    let graded: BTreeSet<&str> = evaluations
        .iter()
        .map(|evaluation| evaluation.sample_id.0.as_str())
        .collect();
    assert_eq!(evaluations.len(), samples.len(), "{evaluations:#?}");
    assert_eq!(graded.len(), samples.len(), "one evaluation per sample");
}

/// `Proposed`, `Muted` and `Retired` definitions are neither read nor recorded.
#[test]
fn does_not_sample_proposed_muted_or_retired() {
    let mut world = World::new(
        "not_active",
        &[
            depth(1, QUEUE),
            (
                "service.queue.proposed".to_owned(),
                definition(
                    "service.queue.proposed",
                    "sum(proposed_depth)",
                    "[]",
                    "PT5M",
                    "lifecycle: proposed",
                    1,
                ),
            ),
            (
                "service.queue.muted".to_owned(),
                definition(
                    "service.queue.muted",
                    "sum(muted_depth)",
                    "[]",
                    "PT5M",
                    "lifecycle: muted\nmute_reason: maintenance\nmute_until: 2099-01-01T00:00:00Z",
                    1,
                ),
            ),
            (
                "service.queue.retired".to_owned(),
                definition(
                    "service.queue.retired",
                    "sum(retired_depth)",
                    "[]",
                    "PT5M",
                    "lifecycle: retired",
                    1,
                ),
            ),
        ],
    );
    for slot in [T0, T0 + FIVE_MINUTES] {
        world.answer(QUEUE, slot, "7");
    }
    let clock = FakeClock::at(T0 + 1);
    let mut scheduler = world.scheduler(&clock);
    scheduler.tick();
    clock.set(T0 + FIVE_MINUTES + 1);
    scheduler.tick();

    let samples = world.samples();
    assert_eq!(samples.len(), 2, "{samples:#?}");
    assert!(
        samples.iter().all(|sample| sample.signal.0 == DEPTH),
        "{samples:#?}"
    );
    let queries: BTreeSet<String> = world
        .invoked()
        .into_iter()
        .map(|(query, _)| query)
        .collect();
    assert_eq!(queries, BTreeSet::from([QUEUE.to_owned()]));
}

/// D16: a missed slot is never re-read and never interpolated. On a restart, and after a clock
/// jump, every slot since the newest stored sample gets one `unknown` sample with reason
/// `missed_slot` per series of the definition's newest pass; the current slot is then sampled.
#[test]
fn missed_slot_is_recorded_unknown_with_reason() {
    let missed = [T0 + FIVE_MINUTES, T0 + 2 * FIVE_MINUTES];
    let now = T0 + 3 * FIVE_MINUTES;
    for restart in [true, false] {
        let name = if restart {
            "missed_restart"
        } else {
            "missed_jump"
        };
        let mut world = World::new(name, &[depth(1, QUEUE), busy()]);
        for slot in [T0, now] {
            world.answer(QUEUE, slot, "7");
            world.workers(slot, "3", "4");
        }
        let clock = FakeClock::at(T0 + 1);
        let mut scheduler = world.scheduler(&clock);
        scheduler.tick();
        clock.set(now + 1);
        if restart {
            drop(scheduler);
            world.scheduler(&clock).tick();
        } else {
            scheduler.tick();
        }

        let samples = world.samples();
        assert_eq!(
            slots(&samples),
            every_series_at(&[T0, missed[0], missed[1], now]),
            "{name}: {samples:#?}"
        );
        for sample in &samples {
            let at_missed = missed.iter().any(|slot| sample.ts.0 == rfc3339(*slot));
            if at_missed {
                assert_eq!(reason(sample), Some(MISSED_SLOT), "{name}: {sample:#?}");
            } else {
                assert!(
                    matches!(sample.reading, Reading::Ok(_)),
                    "{name}: {sample:#?}"
                );
            }
        }
        let read_at: BTreeSet<i64> = world.invoked().into_iter().map(|(_, at)| at).collect();
        assert_eq!(
            read_at,
            BTreeSet::from([T0, now]),
            "{name}: a missed slot is never read"
        );
    }
}

/// A restart inside a slot already sampled reads nothing and records nothing.
#[test]
fn restart_does_not_resample_a_sampled_slot() {
    let mut world = World::new("restart", &[depth(1, QUEUE), busy()]);
    world.answer(QUEUE, T0, "7");
    world.workers(T0, "3", "4");
    let clock = FakeClock::at(T0 + 5);
    world.scheduler(&clock).tick();
    clock.set(T0 + 200);
    world.scheduler(&clock).tick();

    let samples = world.samples();
    assert_eq!(slots(&samples), every_series_at(&[T0]), "{samples:#?}");
    assert_eq!(world.invoked().len(), 2, "{:?}", world.invoked());
}

/// A read that has not finished within the cadence is recorded `unknown` with reason
/// `read timed out`, never as the 0 it answers late; the other definitions of the pass are not
/// held up by it.
#[test]
fn slow_read_is_unknown_not_zero() {
    let mut world = World::new(
        "slow",
        &[
            depth(1, QUEUE),
            (
                SLOW.to_owned(),
                definition(SLOW, "sum(slow_depth)", "[]", "PT1M", ACTIVE, 1),
            ),
        ],
    );
    world.answer(QUEUE, T0, "7");
    let mut clock = FakeClock::at(T0 + 1);
    // PT1M is 600 ms on the wall, PT5M 3 s; the slow read takes 4 s.
    clock.wall_ms_per_s = 10;
    let reader = Slow {
        inner: ConnectorsReader(Connectors::new(world.connectors())),
        signal: SLOW,
        delay: WallDuration::from_secs(4),
    };
    let started = std::time::Instant::now();
    world.scheduler_with(&clock, reader).tick();
    assert!(
        started.elapsed() < WallDuration::from_secs(4),
        "the tick did not wait for the slow read: {:?}",
        started.elapsed()
    );

    let samples = world.samples();
    assert_eq!(samples.len(), 2, "{samples:#?}");
    let slow = samples
        .iter()
        .find(|sample| sample.signal.0 == SLOW)
        .expect("the slow definition has a sample");
    assert_eq!(reason(slow), Some(READ_TIMED_OUT), "{slow:#?}");
    assert_eq!(slow.ts.0, rfc3339(T0));
    let depth = samples
        .iter()
        .find(|sample| sample.signal.0 == DEPTH)
        .expect("the depth definition has a sample");
    assert!(matches!(depth.reading, Reading::Ok(_)), "{depth:#?}");

    let evaluations = world.evaluations();
    let graded = evaluations
        .iter()
        .find(|evaluation| evaluation.sample_id == slow.sample_id)
        .expect("the timed-out sample is graded");
    assert_eq!(graded.verdict, Verdict::UNKNOWN);
}

/// Every sample of one pass carries the pass's one `run_` id; the next pass has another.
#[test]
fn one_run_id_per_pass() {
    let mut world = World::new("run_id", &[depth(1, QUEUE), busy()]);
    for slot in [T0, T0 + FIVE_MINUTES] {
        world.answer(QUEUE, slot, "7");
        world.workers(slot, "3", "4");
    }
    let clock = FakeClock::at(T0 + 1);
    let mut scheduler = world.scheduler(&clock);
    scheduler.tick();
    clock.set(T0 + FIVE_MINUTES + 1);
    scheduler.tick();

    let samples = world.samples();
    assert_eq!(samples.len(), 6, "{samples:#?}");
    let runs = |at: i64| -> BTreeSet<String> {
        samples
            .iter()
            .filter(|sample| sample.ts.0 == rfc3339(at))
            .map(|sample| sample.provenance.run_id.0.clone())
            .collect()
    };
    let first = runs(T0);
    let second = runs(T0 + FIVE_MINUTES);
    assert_eq!(first.len(), 1, "one run id in the first pass: {first:?}");
    assert_eq!(second.len(), 1, "one run id in the second pass: {second:?}");
    assert_ne!(first, second, "each pass has its own run id");
    assert!(
        first
            .iter()
            .chain(&second)
            .all(|run| run.starts_with("run_")),
        "{first:?} {second:?}"
    );
}

/// Definitions are read again on every tick: a revision is sampled and graded at its new `rev`
/// from the next slot, and the slot already sampled is not sampled again.
#[test]
fn revision_applies_at_next_slot() {
    const REVISED: &str = "max(queue_depth)";
    let mut world = World::new("revision", &[depth(1, QUEUE)]);
    world.answer(QUEUE, T0, "7");
    world.answer(REVISED, T0 + FIVE_MINUTES, "9");
    let clock = FakeClock::at(T0 + 1);
    let mut scheduler = world.scheduler(&clock);
    scheduler.tick();
    let (_, revised) = depth(2, REVISED);
    world.define(DEPTH, &revised);
    clock.set(T0 + 100);
    scheduler.tick();
    clock.set(T0 + FIVE_MINUTES + 1);
    scheduler.tick();

    let samples = world.samples();
    let revs: Vec<(String, i64)> = samples
        .iter()
        .map(|sample| (sample.ts.0.clone(), sample.provenance.definition_rev))
        .collect();
    assert_eq!(
        revs,
        vec![(rfc3339(T0), 1), (rfc3339(T0 + FIVE_MINUTES), 2)],
        "{samples:#?}"
    );
    assert_eq!(
        world.invoked(),
        vec![
            (QUEUE.to_owned(), T0),
            (REVISED.to_owned(), T0 + FIVE_MINUTES)
        ]
    );
    let graded = world
        .evaluations()
        .into_iter()
        .find(|evaluation| evaluation.sample_id == samples[1].sample_id)
        .expect("the revised sample is graded");
    assert_eq!(graded.definition_rev, 2);
}

/// D47: the first tick fills at most `FIRST_TICK_FILL_SLOTS` slots, the newest, when the store was
/// last written far behind; the older slots stay empty.
#[test]
fn first_tick_fill_is_bounded_when_the_store_is_far_behind() {
    let now = T0 + 10 * DAY;
    let mut world = World::new("first_tick_bound", &[depth(1, QUEUE)]);
    for slot in [T0, now] {
        world.answer(QUEUE, slot, "7");
    }
    let clock = FakeClock::at(T0 + 1);
    world.scheduler(&clock).tick();
    clock.set(now + 1);
    world.scheduler(&clock).tick();

    let samples = world.samples();
    let missed: Vec<&SignalSampleData> = samples
        .iter()
        .filter(|sample| reason(sample) == Some(MISSED_SLOT))
        .collect();
    assert_eq!(
        missed.len() as i64,
        FIRST_TICK_FILL_SLOTS,
        "at most the bound's slots are filled"
    );
    let oldest = now - FIRST_TICK_FILL_SLOTS * FIVE_MINUTES;
    let expected: Vec<String> = (0..FIRST_TICK_FILL_SLOTS)
        .map(|n| rfc3339(oldest + n * FIVE_MINUTES))
        .collect();
    let mut got: Vec<String> = missed.iter().map(|sample| sample.ts.0.clone()).collect();
    got.sort();
    assert_eq!(got, expected, "the newest slots are the ones filled");
    assert_eq!(
        samples.len() as i64,
        FIRST_TICK_FILL_SLOTS + 2,
        "{samples:#?}"
    );
}

/// A definition muted across a restart and active again at the first tick: the slots of the
/// span it was muted are not missed; the slots after the unmuting revision are.
#[test]
fn first_tick_does_not_fill_the_span_muted_across_a_restart() {
    let unmuted = T0 + 2 * FIVE_MINUTES + 10;
    let now = T0 + 5 * FIVE_MINUTES;
    let mut world = World::new("first_tick_muted_span", &[depth(1, QUEUE)]);
    for slot in [T0, now] {
        world.answer(QUEUE, slot, "7");
    }
    let clock = FakeClock::at(T0 + 1);
    world.scheduler(&clock).tick();
    // Muted after T0, unmuted at `unmuted`; the restart is later.
    world.define(
        DEPTH,
        &definition(DEPTH, QUEUE, "[]", "PT5M", ACTIVE, 3)
            .replace("2026-10-02T00:00:00Z", &rfc3339(unmuted)),
    );
    clock.set(now + 1);
    world.scheduler(&clock).tick();

    let samples = world.samples();
    let filled: Vec<String> = slots(&samples).into_iter().map(|(_, ts)| ts).collect();
    let expected: Vec<String> = [T0, T0 + 3 * FIVE_MINUTES, T0 + 4 * FIVE_MINUTES, now]
        .iter()
        .map(|slot| rfc3339(*slot))
        .collect();
    assert_eq!(filled, expected, "{samples:#?}");
}
