//! Adversary pass 1 on `story:m2-store-baselines`: `thalamus eval` over stored history, at the
//! edges the unit's own CLI test does not state: a series whose first sample lies inside the
//! 28-day window (rule 2 through the pipeline's own series age), and a graded sample that shares
//! its instant with another sample of the series (no sample is its own baseline).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

const INSTANCE: &str = "fixture";
const T: &str = "2026-10-14T22:57:03Z";
const ROLLING: &str = "service.queue.depth_adv_rolling";
const SAME_SLOT: &str = "service.queue.depth_adv_same_slot";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_eval_store_history_p1")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("empty-path")).expect("create the empty PATH");
        Self { root, next: 1 }
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

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

    fn store(&mut self, signal: &str, ts: &str, value: &str) -> String {
        let id = format!("smp_{:012}", self.next);
        let sample = SignalSampleData {
            sample_id: SampleId(id.clone()),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(signal.to_owned()),
            dims: BTreeMap::new(),
            ts: Timestamp(ts.to_owned()),
            window: TimeWindow {
                start: Timestamp(ts.to_owned()),
                end: Timestamp(ts.to_owned()),
            },
            reading: Reading::Ok(CompleteReading {
                value: Decimal(value.to_owned()),
            }),
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
        id
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"))
    }

    fn eval(&self, signal: &str) {
        let out = Command::new(env!("CARGO_BIN_EXE_thalamus"))
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

    fn evaluation_of(&self, sample_id: &str) -> SignalEvaluationData {
        let store = EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens");
        let rows = store.evaluations().to_vec();
        rows.iter()
            .find(|row| row.sample_id.0 == sample_id)
            .unwrap_or_else(|| panic!("an evaluation of {sample_id}: {rows:#?}"))
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

/// The series began 27 days before `T`: its 28-day slot lies before it. Rule 2 is decided from
/// the pipeline's own series age (first stored sample), before the empty slot: the reason is
/// `baseline predates series`, the baseline carries no value, and the evaluation no score.
#[test]
fn same_slot_series_younger_than_four_weeks_predates_through_eval() {
    let mut world = World::new("young_series");
    world.define(SAME_SLOT, "same_slot_median_4w", None);
    world.store(SAME_SLOT, "2026-09-17T22:57:03Z", "5");
    world.store(SAME_SLOT, "2026-09-23T22:57:03Z", "30");
    world.store(SAME_SLOT, "2026-09-30T22:57:03Z", "20");
    world.store(SAME_SLOT, "2026-10-07T22:57:03Z", "10");
    let graded = world.store(SAME_SLOT, T, "60");
    world.eval(SAME_SLOT);
    let row = world.evaluation_of(&graded);
    assert_eq!(row.verdict, Verdict::UNKNOWN, "{row:#?}");
    assert!(
        row.reason
            .as_deref()
            .is_some_and(|reason| reason.contains("baseline predates series")),
        "{row:#?}"
    );
    assert!(
        matches!(
            row.baseline.as_ref().map(|baseline| &baseline.reading),
            Some(Reading::Unknown(_))
        ),
        "{row:#?}"
    );
    assert_eq!(row.score, None, "unknown is never scored as 0: {row:#?}");
}

/// Two samples of the series at the graded instant (a re-read of one window): neither is the
/// other's rolling baseline, and neither is its own. Both are graded against the 10 before them.
#[test]
fn rolling_never_reads_a_sample_at_the_graded_instant() {
    let mut world = World::new("same_instant");
    world.define(ROLLING, "rolling_median", Some(1));
    world.store(ROLLING, "2026-10-14T21:57:03Z", "10");
    let first = world.store(ROLLING, T, "100");
    let second = world.store(ROLLING, T, "35");
    world.eval(ROLLING);
    for (id, verdict) in [(first, Verdict::CRIT), (second, Verdict::WARN)] {
        let row = world.evaluation_of(&id);
        match row.baseline.as_ref().map(|baseline| &baseline.reading) {
            Some(Reading::Ok(CompleteReading { value })) => {
                assert_eq!(value.0, "10", "{row:#?}");
            }
            other => panic!("an ok baseline of 10, got {other:?}: {row:#?}"),
        }
        assert_eq!(row.verdict, verdict, "{row:#?}");
    }
}
