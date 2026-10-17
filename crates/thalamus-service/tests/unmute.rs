//! An expired mute (design § 11 D9, D19, D28): the service samples a `Muted` definition whose
//! `mute_until` has passed as `Active`, records `DefinitionUnmuted` once in its store as actor
//! `thalamus.definition.Service`, warns about the stale file on every pass, and never rewrites
//! the file. On a fake clock, against the fixture instance (`tests/fixtures/scheduler`) and the
//! Connectors double (`thalamus-sources`, `src/bin/connectors-double.rs`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration as WallDuration;

use serde_json::{Value, json};
use thalamus_model::actor::Actor;
use thalamus_model::definition::SignalId;
use thalamus_model::observation::{CompleteReading, Reading, SignalSampleData};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_pipeline::definitions;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_pipeline::sample::{self, Producer, Refusal};
use thalamus_service::scheduler::{Clock, ConnectorsReader, Scheduler, Tick};
use thalamus_sources::adapter::unknown;
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

/// A `Muted` definition file of a `PT5M` gauge whose mute ends at Unix second `until`.
fn muted(id: &str, query: &str, until: i64) -> String {
    format!(
        "id: {id}
title: A signal of the unmute tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: []
source: {{ adapter: metrics, connection: metrics-main }}
extractor: promql.value
query: {{ value: \"{query}\" }}
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
lifecycle: muted
mute_reason: maintenance
mute_until: {}
rev: 3
revised_at: 2026-10-02T00:00:00Z
revised_by: operator
",
        rfc3339(until)
    )
}

/// One test's directory: the instance copy, the store, the recordings and the call log.
struct World {
    root: PathBuf,
    answers: Vec<Value>,
}

impl World {
    fn new(name: &str, definitions: &[(&str, String)]) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("service_unmute")
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

fn not_active(tick: &Tick) -> Vec<&String> {
    tick.problems
        .iter()
        .filter(|problem| problem.contains("not-active"))
        .collect()
}

#[test]
fn expired_mute_is_sampled_as_active() {
    let mut world = World::new(
        "expired_mute_is_sampled_as_active",
        &[(DEPTH, muted(DEPTH, QUEUE, T0 - FIVE_MINUTES))],
    );
    world.answer(QUEUE, T0, "7");
    let clock = FakeClock::at(T0);
    let tick = world.scheduler(&clock).tick();

    assert!(not_active(&tick).is_empty(), "{:?}", tick.problems);
    let samples = world.samples(DEPTH);
    assert_eq!(
        samples.len(),
        1,
        "one sample of the expired mute: {samples:?}"
    );
    assert_eq!(samples[0].ts.0, rfc3339(T0));
    assert_eq!(
        samples[0].reading,
        Reading::Ok(CompleteReading {
            value: Decimal("7".to_owned())
        })
    );
    assert_eq!(samples[0].provenance.definition_rev, 3);
}

#[test]
fn unexpired_mute_stays_muted() {
    let mut world = World::new(
        "unexpired_mute_stays_muted",
        &[(LATER, muted(LATER, LATER_QUEUE, T0 + DAY))],
    );
    world.answer(LATER_QUEUE, T0, "7");
    let clock = FakeClock::at(T0);
    let mut scheduler = world.scheduler(&clock);
    let first = scheduler.tick();
    clock.set(T0 + FIVE_MINUTES);
    let second = scheduler.tick();

    assert!(first.runs.is_empty() && second.runs.is_empty());
    assert!(world.samples(LATER).is_empty());
    assert!(
        world
            .store()
            .definition_events(&SignalId(LATER.to_owned()))
            .is_empty()
    );
    let file = world.file(LATER).display().to_string();
    for tick in [&first, &second] {
        assert!(
            !tick.problems.iter().any(|problem| problem.contains(&file)),
            "an unexpired mute is not stale: {:?}",
            tick.problems
        );
    }
}

#[test]
fn expired_mute_unmutes_without_rewriting_the_file() {
    let mut world = World::new(
        "expired_mute_unmutes_without_rewriting_the_file",
        &[(DEPTH, muted(DEPTH, QUEUE, T0 - FIVE_MINUTES))],
    );
    world.answer(QUEUE, T0, "7");
    let file = world.file(DEPTH);
    let bytes = std::fs::read(&file).expect("read the definition");
    let modified = std::fs::metadata(&file)
        .and_then(|meta| meta.modified())
        .expect("the file's modification time");
    // A tick within the same second as the write could hide a rewrite from the mtime.
    std::thread::sleep(WallDuration::from_millis(1_100));

    let clock = FakeClock::at(T0);
    let tick = world.scheduler(&clock).tick();

    assert_eq!(tick.runs.len(), 1, "{tick:?}");
    assert_eq!(std::fs::read(&file).expect("read it again"), bytes);
    assert_eq!(
        std::fs::metadata(&file)
            .and_then(|meta| meta.modified())
            .expect("the modification time again"),
        modified
    );
    let reloaded = LoadedInstance::load(&world.instance()).expect("the instance loads");
    assert_eq!(
        reloaded
            .instance()
            .definition(DEPTH)
            .expect("the definition")
            .state(),
        thalamus_model::definition::SignalDefinitionState::Muted,
        "the file keeps its stale mute"
    );
}

#[test]
fn unmute_event_is_recorded_once_as_service_actor() {
    let mut world = World::new(
        "unmute_event_is_recorded_once_as_service_actor",
        &[(DEPTH, muted(DEPTH, QUEUE, T0 - FIVE_MINUTES))],
    );
    for slot in 0..3 {
        world.answer(QUEUE, T0 + slot * FIVE_MINUTES, "7");
    }
    let clock = FakeClock::at(T0);
    let mut scheduler = world.scheduler(&clock);
    for slot in 0..3 {
        clock.set(T0 + slot * FIVE_MINUTES);
        let tick = scheduler.tick();
        assert!(not_active(&tick).is_empty(), "{:?}", tick.problems);
    }

    let events = world.store().definition_events(&SignalId(DEPTH.to_owned()));
    assert_eq!(events.len(), 1, "one DefinitionUnmuted: {events:?}");
    let event = &events[0];
    assert_eq!(event.event.signal_id.0, DEPTH);
    assert_eq!(event.actor, Actor::DefinitionService);
    assert_eq!(event.actor.name(), "thalamus.definition.Service");
    assert_eq!(event.rev, 3);
    assert_eq!(event.mute_until.0, rfc3339(T0 - FIVE_MINUTES));
    assert_eq!(event.at.0, rfc3339(T0));
    assert_eq!(world.samples(DEPTH).len(), 3);
}

#[test]
fn stale_mute_is_logged_naming_the_file_each_pass() {
    let mut world = World::new(
        "stale_mute_is_logged_naming_the_file_each_pass",
        &[(DEPTH, muted(DEPTH, QUEUE, T0 - FIVE_MINUTES))],
    );
    for slot in 0..3 {
        world.answer(QUEUE, T0 + slot * FIVE_MINUTES, "7");
    }
    let file = world.file(DEPTH).display().to_string();
    let clock = FakeClock::at(T0);
    let mut scheduler = world.scheduler(&clock);
    for slot in 0..3 {
        clock.set(T0 + slot * FIVE_MINUTES);
        let tick = scheduler.tick();
        let warnings: Vec<&String> = tick
            .problems
            .iter()
            .filter(|problem| problem.contains(&file))
            .collect();
        assert_eq!(
            warnings.len(),
            1,
            "pass {slot}: one warning naming {file}: {:?}",
            tick.problems
        );
        assert!(
            warnings[0].contains("mute_until"),
            "the warning says what is stale: {}",
            warnings[0]
        );
    }
}

#[test]
fn expired_mute_passes_record_sample_as_active() {
    let world = World::new(
        "expired_mute_passes_record_sample_as_active",
        &[
            (DEPTH, muted(DEPTH, QUEUE, T0 - FIVE_MINUTES)),
            (LATER, muted(LATER, LATER_QUEUE, T0 + DAY)),
        ],
    );
    let mut instance = LoadedInstance::load(&world.instance()).expect("the instance loads");
    let mut store = world.store();
    let producer = Producer {
        name: "unmute-test".to_owned(),
        version: "0".to_owned(),
    };
    let end = Timestamp(rfc3339(T0));
    let read = |signal: &str, instance: &LoadedInstance| {
        let data = instance
            .instance()
            .definition(signal)
            .expect("the definition")
            .data()
            .clone();
        let mut read = unknown(&data, &end, "");
        read.reading = Reading::Ok(CompleteReading {
            value: Decimal("1".to_owned()),
        });
        vec![read]
    };

    // Before the overlay, the file lifecycle answers the guard: Muted, refused.
    let mut pass = sample::begin(&mut store).expect("a run id");
    let depth = SignalId(DEPTH.to_owned());
    let reads = read(DEPTH, &instance);
    sample::record_reads(
        &mut pass,
        &mut instance,
        &mut store,
        &depth,
        reads,
        &end,
        &producer,
    )
    .expect("the pass runs");
    assert!(pass.recorded.is_empty());
    assert!(matches!(
        pass.refused.as_slice(),
        [Refusal::NotActive { .. }]
    ));

    let unmuted = definitions::unmute_expired(&mut instance, &end);
    assert_eq!(unmuted.len(), 1, "{unmuted:?}");
    assert_eq!(unmuted[0].event.signal_id, depth);
    assert_eq!(unmuted[0].file, world.file(DEPTH).display().to_string());

    assert_eq!(sample::admit(&instance, DEPTH), Ok(depth.clone()));
    assert_eq!(sample::active(&instance), vec![depth.clone()]);
    let mut pass = sample::begin(&mut store).expect("a run id");
    let reads = read(DEPTH, &instance);
    sample::record_reads(
        &mut pass,
        &mut instance,
        &mut store,
        &depth,
        reads,
        &end,
        &producer,
    )
    .expect("the pass runs");
    assert!(pass.refused.is_empty(), "{:?}", pass.refused);
    assert_eq!(pass.recorded.len(), 1);

    // Every other definition answers its file lifecycle.
    let later = SignalId(LATER.to_owned());
    let reads = read(LATER, &instance);
    sample::record_reads(
        &mut pass,
        &mut instance,
        &mut store,
        &later,
        reads,
        &end,
        &producer,
    )
    .expect("the pass runs");
    assert_eq!(pass.recorded.len(), 1);
    assert!(matches!(
        pass.refused.as_slice(),
        [Refusal::NotActive { .. }]
    ));
    assert!(
        instance.instance().refused_writes().is_empty(),
        "nothing is written through the file ports: {:?}",
        instance.instance().refused_writes()
    );
}
