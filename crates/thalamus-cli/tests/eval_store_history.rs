//! `story:m2-store-baselines`: `thalamus eval` grades a `history: store` definition against
//! `rolling_median` and `same_slot_median_4w` from the series' stored samples (design § 3.1).
//!
//! Each test copies the fixture instance (`tests/fixtures/instance`) into its own directory under
//! the build's temporary directory, adds the definitions it grades to that copy, appends the
//! samples to the instance's default store (`sqlite:<instance>/.thalamus/store.sqlite`) and runs
//! `eval`. No source is read: `--connectors` names a binary that does not exist and `PATH` is
//! empty, so any source read would fail.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

/// The graded instant, a Wednesday.
const T: &str = "2026-10-14T22:57:03Z";

const ROLLING: &str = "service.queue.depth_rolling";
const SAME_SLOT: &str = "service.queue.depth_same_slot";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_eval_store_history")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("empty-path")).expect("create the empty PATH directory");
        Self { root, next: 1 }
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    /// Adds a `history: store` gauge graded `higher_is_worse` against `baselines`.
    fn define(&self, signal: &str, baselines: &str, rolling_n: Option<i64>) {
        let rolling_n = rolling_n.map_or(String::new(), |n| format!("  rolling_n: {n}\n"));
        let text = format!(
            "id: {signal}\n\
             title: Messages waiting in the work queue, against stored history\n\
             family: service\n\
             kind: gauge\n\
             unit: \"{{message}}\"\n\
             dimensions: []\n\
             source: {{ adapter: metrics, connection: metrics-main }}\n\
             extractor: promql.value\n\
             query: {{ value: sum(queue_depth) }}\n\
             window: PT5M\n\
             cadence: PT5M\n\
             history: store\n\
             denominator: {{ unit: \"{{message}}\", min_n: 0, since: first_sample }}\n\
             evaluation:\n\
             \x20 rule: higher_is_worse\n\
             \x20 baselines: [{baselines}]\n\
             {rolling_n}\
             \x20 warn: {{ x: 2 }}\n\
             \x20 crit: {{ x: 4 }}\n\
             owner: team:service\n\
             lifecycle: active\n\
             rev: 1\n\
             revised_at: 2026-09-01T00:00:00Z\n\
             revised_by: operator\n"
        );
        std::fs::write(
            self.instance()
                .join("definitions")
                .join(format!("{signal}.yaml")),
            text,
        )
        .expect("write the definition");
    }

    /// Appends one sample of `signal` at `ts` to the instance's store.
    fn store(&mut self, signal: &str, ts: &str, reading: Reading) {
        let sample = SignalSampleData {
            sample_id: SampleId(format!("smp_{:012}", self.next)),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(signal.to_owned()),
            dims: BTreeMap::new(),
            ts: Timestamp(ts.to_owned()),
            window: TimeWindow {
                start: Timestamp(ts.to_owned()),
                end: Timestamp(ts.to_owned()),
            },
            reading,
            text: None,
            unit: Unit("{message}".to_owned()),
            denominator: None,
            provenance: Provenance {
                producer: "thalamus-service".to_owned(),
                version: "0.1.0".to_owned(),
                definition_rev: 1,
                adapter: AdapterId("metrics".to_owned()),
                connection: ConnectionId("metrics-main".to_owned()),
                query_sha256: QueryDigest("0".repeat(64)),
                run_id: RunId(format!("run_{:012}", self.next)),
            },
        };
        self.next += 1;
        let mut store = EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens");
        store.append_sample(&sample).expect("the sample is stored");
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"))
    }

    /// `thalamus eval <signal> --format json`, which must exit 0.
    fn eval(&self, signal: &str) {
        let out: Output = Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(self.root.join("no-connectors-here"))
            .args(["eval", signal, "--format", "json"])
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("PATH", self.root.join("empty-path"))
            .output()
            .expect("the thalamus binary runs");
        assert_eq!(
            out.status.code(),
            Some(0),
            "eval {signal}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// The evaluation of `signal`'s sample at `ts`.
    fn evaluation_at(&self, signal: &str, ts: &str) -> SignalEvaluationData {
        let store = EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens");
        let rows = store.evaluations().to_vec();
        rows.iter()
            .find(|row| row.signal.0 == signal && row.ts.0 == ts)
            .unwrap_or_else(|| panic!("an evaluation of {signal} at {ts}: {rows:#?}"))
            .clone()
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create the directory");
    for entry in std::fs::read_dir(from).expect("read the fixture") {
        let entry = entry.expect("a fixture entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy a fixture file");
        }
    }
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    })
}

fn unknown(reason: &str) -> Reading {
    Reading::Unknown(UnreadReading {
        reason: reason.to_owned(),
    })
}

fn partial(value: &str) -> Reading {
    Reading::Partial(PartialReading {
        value: Decimal(value.to_owned()),
        bound: Bound::Lower,
    })
}

fn baseline_value(row: &SignalEvaluationData) -> &str {
    match row.baseline.as_ref().map(|baseline| &baseline.reading) {
        Some(Reading::Ok(CompleteReading { value })) => &value.0,
        _ => panic!("an ok baseline: {row:#?}"),
    }
}

#[test]
fn eval_grades_store_history_definition_from_stored_samples() {
    let mut world = World::new("store_history");

    // rolling_median over the last 3 ok samples: 10, 20 and 30, skipping the unknown and the
    // partial ones; the baseline is 20, and 100 is at least 4 x 20.
    world.define(ROLLING, "rolling_median", Some(3));
    world.store(ROLLING, "2026-10-14T18:57:03Z", ok("10"));
    world.store(ROLLING, "2026-10-14T19:57:03Z", unknown("source down"));
    world.store(ROLLING, "2026-10-14T20:57:03Z", ok("20"));
    world.store(ROLLING, "2026-10-14T21:27:03Z", partial("500"));
    world.store(ROLLING, "2026-10-14T21:57:03Z", ok("30"));
    world.store(ROLLING, T, ok("100"));
    world.eval(ROLLING);
    let graded = world.evaluation_at(ROLLING, T);
    let baseline = graded.baseline.as_ref().expect("a baseline");
    assert_eq!(baseline.kind.0, "rolling_median", "{graded:#?}");
    assert_eq!(baseline_value(&graded), "20", "{graded:#?}");
    assert_eq!(graded.verdict, Verdict::CRIT, "{graded:#?}");
    // Its first sample had no 3 earlier ok samples: unknown, never graded against 0.
    let first = world.evaluation_at(ROLLING, "2026-10-14T18:57:03Z");
    assert_eq!(first.verdict, Verdict::UNKNOWN, "{first:#?}");
    assert!(
        first
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("baseline predates series")),
        "{first:#?}"
    );

    // same_slot_median_4w over the Wednesdays 22:00 to 23:00 UTC of the four weeks before: 10,
    // 20, 30 and 40; the baseline is 25, and 60 is at least 2 x 25 and under 4 x 25.
    world.define(SAME_SLOT, "same_slot_median_4w", None);
    world.store(SAME_SLOT, "2026-09-16T22:57:03Z", ok("40"));
    world.store(SAME_SLOT, "2026-09-23T22:50:00Z", ok("30"));
    world.store(SAME_SLOT, "2026-09-30T22:57:03Z", ok("20"));
    // Outside the 7-day slot's hour: not its reading.
    world.store(SAME_SLOT, "2026-10-07T21:59:00Z", ok("999"));
    world.store(SAME_SLOT, "2026-10-07T22:57:03Z", ok("10"));
    world.store(SAME_SLOT, T, ok("60"));
    world.eval(SAME_SLOT);
    let graded = world.evaluation_at(SAME_SLOT, T);
    let baseline = graded.baseline.as_ref().expect("a baseline");
    assert_eq!(baseline.kind.0, "same_slot_median_4w", "{graded:#?}");
    assert_eq!(baseline_value(&graded), "25", "{graded:#?}");
    assert_eq!(graded.verdict, Verdict::WARN, "{graded:#?}");
    // A week earlier the series had no four weeks behind it.
    let early = world.evaluation_at(SAME_SLOT, "2026-10-07T22:57:03Z");
    assert_eq!(early.verdict, Verdict::UNKNOWN, "{early:#?}");
}
