//! Settled verdict changes (design § 3.2, § 6.2, § 11 D6, D10, D13): the scheduler observes each
//! new evaluation through one hysteresis per series, records a settled change as a `Transition`
//! through `RecordTransition`, and emits it as a CloudEvents event in the tick.
//!
//! A fake source answers each slot with the value the test set for it; the fixture definition
//! grades against a fixed 0 with `WARN` above 100, so 0 reads `OK` and 150 reads `WARN`. The
//! default persistence holds (`k` 2, `k_crit` 1, `m` 3).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration as WallDuration;

use serde_json::Value;
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{CompleteReading, Reading, RunId, SeriesKey};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::transition::{RecordTransition, SeriesTransitions};
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_service::events::TRANSITION_TYPE;
use thalamus_service::scheduler::{Clock, Reader, Scheduler, Tick};
use thalamus_sources::adapter::{Read, unknown};
use thalamus_sources::connectors::Connectors;
use thalamus_store::eventlog::{EventlogStore, Locator};
use thalamus_store::transitions::Evidence;

/// A slot boundary of `PT5M`.
const T0: i64 = 1_792_018_500;
/// `PT5M`, in seconds.
const FIVE_MINUTES: i64 = 300;
/// The registry id of the fixture instance.
const INSTANCE: &str = "fixture";
const DEPTH: &str = "service.queue.depth";
const BUSY: &str = "service.worker.busy";

const OK: &str = "0";
const WARN: &str = "150";

#[derive(Clone)]
struct FakeClock(Arc<AtomicI64>);

impl FakeClock {
    fn set(&self, now: i64) {
        self.0.store(now, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }

    fn span(&self, seconds: i64) -> WallDuration {
        WallDuration::from_secs(u64::try_from(seconds).unwrap_or(0))
    }

    fn sleep_until(&self, at: i64) {
        self.set(at);
    }
}

/// The fake source: one complete read per slot, of the value set for that slot, with `dims`.
#[derive(Clone, Default)]
struct Source {
    values: Arc<Mutex<BTreeMap<String, String>>>,
    dims: BTreeMap<String, String>,
}

impl Reader for Source {
    fn read(
        &self,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Vec<Read> {
        let values = self.values.lock().expect("the values");
        let Some(value) = values.get(&end.0) else {
            return vec![unknown(definition, end, "no value set")];
        };
        let mut read = unknown(definition, end, "");
        read.reading = Reading::Ok(CompleteReading {
            value: Decimal(value.clone()),
        });
        read.dims.clone_from(&self.dims);
        vec![read]
    }
}

/// One test's directory: the instance, its store and the fake source.
struct World {
    root: PathBuf,
    clock: FakeClock,
    source: Source,
    /// The slot the next [`World::drive`] samples.
    next: i64,
}

impl World {
    fn new(name: &str, signal: &str, dimensions: &str, dims: &[(&str, &str)]) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("service_transitions")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scheduler"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("instance/definitions")).expect("definitions");
        std::fs::write(
            root.join(format!("instance/definitions/{signal}.yaml")),
            definition(signal, dimensions),
        )
        .expect("write the definition");
        Self {
            root,
            clock: FakeClock(Arc::new(AtomicI64::new(T0))),
            source: Source {
                values: Arc::default(),
                dims: dims
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                    .collect(),
            },
            next: T0,
        }
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.root.join("store.sqlite"))
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn scheduler(&self) -> Scheduler<FakeClock, Source> {
        Scheduler::open(
            &self.instance(),
            &self.locator(),
            self.clock.clone(),
            self.source.clone(),
            Connectors::new(self.root.join("no-connectors")),
        )
        .unwrap_or_else(|error| panic!("the scheduler opens: {error}"))
    }

    fn store(&self) -> EventlogStore {
        EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens")
    }

    /// One tick per value, at consecutive slots: each slot is sampled with its value.
    fn drive(&mut self, scheduler: &mut Scheduler<FakeClock, Source>, values: &[&str]) -> Tick {
        let mut all = Tick::default();
        for value in values {
            self.source
                .values
                .lock()
                .expect("the values")
                .insert(rfc3339(self.next), (*value).to_owned());
            self.clock.set(self.next);
            let tick = scheduler.tick();
            assert!(
                tick.problems.is_empty(),
                "the tick at {} reported {:?}",
                self.next,
                tick.problems
            );
            all.runs.extend(tick.runs);
            all.events.extend(tick.events);
            self.next += FIVE_MINUTES;
        }
        all
    }

    /// The stored evaluations of `series`, in `ts` order.
    fn evaluations(&self, series: &str) -> Vec<SignalEvaluationData> {
        let mut evaluations: Vec<SignalEvaluationData> = self
            .store()
            .evaluations()
            .iter()
            .filter(|evaluation| evaluation.series.0 == series)
            .cloned()
            .collect();
        evaluations.sort_by(|a, b| a.ts.0.cmp(&b.ts.0));
        evaluations
    }

    fn transitions(&self, series: &str) -> Vec<SeriesTransitions> {
        self.store()
            .series_transitions(&SeriesKey(series.to_owned()))
    }
}

/// A `promql.value` gauge graded against a fixed 0: `WARN` above 100, `CRIT` above 1000.
fn definition(id: &str, dimensions: &str) -> String {
    format!(
        "id: {id}
title: A signal of the transition tests
family: service
kind: gauge
unit: \"{{message}}\"
dimensions: {dimensions}
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

/// The RFC 3339 text of Unix second `at`, in UTC.
fn rfc3339(at: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(at)
        .expect("an instant")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC 3339")
}

fn ids(evaluations: &[SignalEvaluationData]) -> Vec<EvaluationId> {
    evaluations
        .iter()
        .map(|evaluation| evaluation.evaluation_id.clone())
        .collect()
}

/// Three `OK` samples settle the series' first verdict (`m` = 3) and two `WARN` samples settle
/// the change to `WARN` (`k` = 2); one `WARN` alone settles nothing.
#[test]
fn transition_recorded_when_hysteresis_settles() {
    let mut world = World::new("settles", DEPTH, "[]", &[]);
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK]);
    assert!(world.transitions(DEPTH).is_empty(), "two OK settle nothing");
    world.drive(&mut scheduler, &[OK]);
    let first = world.transitions(DEPTH);
    assert_eq!(first.len(), 1, "the third OK settles: {first:?}");
    assert_eq!(first[0].from_verdict, None);
    assert_eq!(first[0].to_verdict, Verdict::OK);
    world.drive(&mut scheduler, &[WARN]);
    assert_eq!(
        world.transitions(DEPTH).len(),
        1,
        "one WARN settles nothing"
    );
    world.drive(&mut scheduler, &[WARN]);
    let transitions = world.transitions(DEPTH);
    assert_eq!(
        transitions.len(),
        2,
        "the second WARN settles: {transitions:?}"
    );
    assert_eq!(transitions[1].from_verdict, Some(Verdict::OK));
    assert_eq!(transitions[1].to_verdict, Verdict::WARN);
    assert_eq!(transitions[1].signal.0, DEPTH);
    assert_eq!(transitions[1].definition_rev, 1);
    assert!(transitions[1].transition_id.0.starts_with("trn_"));
}

/// The settled `WARN` references exactly the two `WARN` evaluations of its run, oldest first, and
/// copies the first and the settling evaluation's `ts` and the settling score.
#[test]
fn transition_references_the_k_evaluations_of_its_run() {
    let mut world = World::new("references", DEPTH, "[]", &[]);
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK, OK, WARN, WARN]);
    let evaluations = world.evaluations(DEPTH);
    assert_eq!(evaluations.len(), 5);
    let warns = &evaluations[3..];
    assert!(warns.iter().all(|e| e.verdict == Verdict::WARN));
    let transitions = world.transitions(DEPTH);
    let warn = transitions.last().expect("a transition");
    assert_eq!(warn.to_verdict, Verdict::WARN);
    assert_eq!(warn.evaluations, ids(warns));
    assert_eq!(warn.first_ts, warns[0].ts);
    assert_eq!(warn.settled_ts, warns[1].ts);
    assert_eq!(warn.settling_score, warns[1].score);
    let ok = &transitions[0];
    assert_eq!(ok.evaluations, ids(&evaluations[..3]));
}

/// Each recorded transition is emitted in its tick as one CloudEvents 1.0 event: type
/// `dev.beyond10x.thalamus.transition`, `subject` the D10 series key, `time` the settling
/// evaluation's `ts`, `id` the transition id.
#[test]
fn transition_emitted_as_cloudevent_with_series_subject() {
    let mut world = World::new(
        "cloudevent",
        BUSY,
        "[component]",
        &[("component", "checkout")],
    );
    let series = format!("{BUSY}{{component=\"checkout\"}}");
    let mut scheduler = world.scheduler();
    let quiet = world.drive(&mut scheduler, &[WARN]);
    assert!(
        quiet.events.is_empty(),
        "nothing settled: {:?}",
        quiet.events
    );
    let tick = world.drive(&mut scheduler, &[WARN]);
    assert_eq!(tick.events.len(), 1, "{:?}", tick.events);
    let event: Value = serde_json::from_str(&tick.events[0]).expect("a JSON event");
    let transitions = world.transitions(&series);
    assert_eq!(transitions.len(), 1);
    let transition = &transitions[0];
    let settling = world.evaluations(&series);
    assert_eq!(event["specversion"], "1.0");
    assert_eq!(event["type"], TRANSITION_TYPE);
    assert_eq!(TRANSITION_TYPE, "dev.beyond10x.thalamus.transition");
    assert_eq!(event["subject"], series.as_str());
    assert_eq!(event["time"], settling[1].ts.0.as_str());
    assert_eq!(event["time"], transition.settled_ts.0.as_str());
    assert_eq!(event["id"], transition.transition_id.0.as_str());
    let source = event["source"].as_str().expect("a source");
    assert!(
        source.contains(INSTANCE) && source.contains("thalamus-service"),
        "{source}"
    );
    assert_eq!(event["data"]["to_verdict"], "WARN");
    assert_eq!(event["data"]["series"], series.as_str());
    assert!(event["data"].get("from_verdict").is_none());
    assert_eq!(
        event["data"]["evaluations"].as_array().map(Vec::len),
        Some(2)
    );
}

/// A transition whose evaluations the store no longer holds (they left raw retention) reads back
/// with its copied fields and each evaluation marked expired, not as an error (D13).
///
/// The store has no raw retention yet, so nothing retires an evaluation: the transition names
/// evaluation ids the store never held, which a reader meets exactly as it would meet retired
/// ones.
#[test]
fn transition_survives_expired_evaluations() {
    let world = World::new("expired", DEPTH, "[]", &[]);
    let instance = LoadedInstance::load(&world.instance()).expect("the instance loads");
    let mut store = world.store();
    let retired = vec![
        EvaluationId("evl_000000000901".to_owned()),
        EvaluationId("evl_000000000902".to_owned()),
    ];
    let recorded = store
        .record_transition(
            instance.instance(),
            RecordTransition {
                signal: thalamus_model::definition::SignalId(DEPTH.to_owned()),
                series: SeriesKey(DEPTH.to_owned()),
                dims: BTreeMap::<String, DimensionValue>::new(),
                definition_rev: 1,
                from_verdict: Some(Verdict::OK),
                to_verdict: Verdict::WARN,
                first_ts: Timestamp(rfc3339(T0)),
                settled_ts: Timestamp(rfc3339(T0 + FIVE_MINUTES)),
                settling_score: Some(Decimal("1.5".to_owned())),
                evaluations: retired.clone(),
            },
        )
        .expect("the transition is recorded");
    drop(store);
    let read = world
        .store()
        .series_transition_evidence(&SeriesKey(DEPTH.to_owned()));
    assert_eq!(read.len(), 1);
    let row = &read[0].transition;
    assert_eq!(row.transition_id, recorded.transition_id);
    assert_eq!(row.from_verdict, Some(Verdict::OK));
    assert_eq!(row.to_verdict, Verdict::WARN);
    assert_eq!(row.first_ts.0, rfc3339(T0));
    assert_eq!(row.settled_ts.0, rfc3339(T0 + FIVE_MINUTES));
    assert_eq!(row.settling_score, Some(Decimal("1.5".to_owned())));
    assert_eq!(row.evaluations, retired);
    assert_eq!(
        read[0].evaluations,
        retired
            .into_iter()
            .map(Evidence::Expired)
            .collect::<Vec<_>>()
    );
}

/// A restart between the first and the second `WARN` loses no hysteresis state: the scheduler
/// seeds each series from its stored evaluations and its last transition, so the second `WARN`
/// after the restart settles `OK` → `WARN` over both `WARN` evaluations.
#[test]
fn hysteresis_state_survives_restart() {
    let mut world = World::new("restart", DEPTH, "[]", &[]);
    let mut scheduler = world.scheduler();
    world.drive(&mut scheduler, &[OK, OK, OK, WARN]);
    assert_eq!(world.transitions(DEPTH).len(), 1);
    drop(scheduler);
    let mut scheduler = world.scheduler();
    let tick = world.drive(&mut scheduler, &[WARN]);
    let transitions = world.transitions(DEPTH);
    assert_eq!(transitions.len(), 2, "{transitions:?}");
    assert_eq!(transitions[1].from_verdict, Some(Verdict::OK));
    assert_eq!(transitions[1].to_verdict, Verdict::WARN);
    let evaluations = world.evaluations(DEPTH);
    assert_eq!(transitions[1].evaluations, ids(&evaluations[3..5]));
    assert_eq!(tick.events.len(), 1, "{:?}", tick.events);
}

/// End to end through the scheduler: one series reads `OK` until its first verdict has settled
/// (`m` = 3, recorded as a change with no `from`, hysteresis rule 4), then `WARN`, then `OK`.
/// That drive records and emits exactly two transitions: `WARN` opened (`OK` -> `WARN`, `k` = 2)
/// and `WARN` cleared (`WARN` -> `OK`, `m` = 3).
#[test]
fn warn_opened_and_cleared_end_to_end() {
    let mut world = World::new("end-to-end", DEPTH, "[]", &[]);
    let mut scheduler = world.scheduler();
    let settle = world.drive(&mut scheduler, &[OK, OK, OK]);
    let settled = world.transitions(DEPTH);
    assert_eq!(settled.len(), 1, "the series settled OK: {settled:?}");
    assert_eq!(settled[0].from_verdict, None);
    assert_eq!(settled[0].to_verdict, Verdict::OK);
    assert_eq!(settle.events.len(), 1, "{:?}", settle.events);
    let tick = world.drive(&mut scheduler, &[WARN, WARN, OK, OK, OK]);
    let all = world.transitions(DEPTH);
    assert_eq!(all.len(), 3, "{all:?}");
    let transitions = &all[1..];
    assert_eq!(
        transitions
            .iter()
            .map(|t| (t.from_verdict, t.to_verdict))
            .collect::<Vec<_>>(),
        vec![
            (Some(Verdict::OK), Verdict::WARN),
            (Some(Verdict::WARN), Verdict::OK)
        ]
    );
    assert_eq!(tick.events.len(), 2, "{:?}", tick.events);
    let emitted: Vec<Value> = tick
        .events
        .iter()
        .map(|event| serde_json::from_str(event).expect("a JSON event"))
        .collect();
    for (event, transition) in emitted.iter().zip(transitions) {
        assert_eq!(event["type"], TRANSITION_TYPE);
        assert_eq!(event["id"], transition.transition_id.0.as_str());
        assert_eq!(event["subject"], DEPTH);
        assert_eq!(event["time"], transition.settled_ts.0.as_str());
    }
}
