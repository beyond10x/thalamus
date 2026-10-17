//! Adversary pass 1 of `thalamus rollup --signal`. Destination: crates/thalamus-cli/tests/.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SignalSampleData,
    TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";
/// No dimensions, rev 1, active.
const DEPTH: &str = "service.queue.depth";
/// Dimension `component`, whose registry lists `checkout` and `search`; rev 1, active.
const MISS: &str = "service.cache.miss_ratio";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_rollup")
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

    fn store_file(&self) -> PathBuf {
        self.instance().join(".thalamus/store.sqlite")
    }

    fn store(&self) -> EventlogStore {
        EventlogStore::open(&Locator::Sqlite(self.store_file()), INSTANCE).expect("the store opens")
    }

    /// Stores one sample of `signal` with `dims` at `ts` and returns it.
    fn sample(
        &mut self,
        signal: &str,
        dims: &[(&str, &str)],
        ts: &str,
        reading: Reading,
    ) -> SignalSampleData {
        let dims: BTreeMap<String, DimensionValue> = dims
            .iter()
            .map(|(key, value)| ((*key).to_owned(), DimensionValue((*value).to_owned())))
            .collect();
        let series = thalamus_core::series::series_key(&SignalId(signal.to_owned()), &dims);
        let sample = SignalSampleData {
            sample_id: SampleId(format!("smp_{:012}", self.next)),
            signal: SignalId(signal.to_owned()),
            series,
            dims,
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
                producer: "thalamus".to_owned(),
                version: "0.1.0".to_owned(),
                definition_rev: 1,
                adapter: AdapterId("metrics".to_owned()),
                connection: ConnectionId("metrics-main".to_owned()),
                query_sha256: QueryDigest("0".repeat(64)),
                run_id: RunId(format!("run_{:012}", self.next)),
            },
        };
        self.next += 1;
        self.store()
            .append_sample(&sample)
            .expect("the sample is stored");
        sample
    }

    /// Stores one evaluation of `sample` at rev 1.
    fn evaluate(&mut self, sample: &SignalSampleData, verdict: Verdict, score: Option<&str>) {
        let evaluation = SignalEvaluationData {
            evaluation_id: EvaluationId(format!("evl_{:012}", self.next)),
            sample_id: sample.sample_id.clone(),
            signal: sample.signal.clone(),
            series: sample.series.clone(),
            dims: sample.dims.clone(),
            ts: sample.ts.clone(),
            definition_rev: 1,
            verdict,
            prev: None,
            score: score.map(|score| Decimal(score.to_owned())),
            baseline: None,
            reason: (verdict != Verdict::OK).then(|| "graded by the test".to_owned()),
        };
        self.next += 1;
        self.store()
            .append_evaluation(&evaluation)
            .expect("the evaluation is stored");
    }

    fn thalamus(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(self.root.join("no-connectors-here"))
            .arg("rollup")
            .args(args)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("PATH", self.root.join("empty-path"))
            .output()
            .expect("the thalamus binary runs")
    }

    /// `rollup … --format json`: exit 0 and its one JSON document.
    fn json(&self, args: &[&str]) -> Value {
        let mut all = args.to_vec();
        all.extend(["--format", "json"]);
        let out = self.thalamus(&all);
        assert_eq!(
            out.status.code(),
            Some(0),
            "thalamus rollup {all:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap_or_else(|error| {
            panic!(
                "stdout is one JSON document ({error}): {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        })
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

/// The ids of the rows of `doc[list]`.
fn ids(doc: &Value, list: &str) -> Vec<String> {
    doc[list]
        .as_array()
        .unwrap_or_else(|| panic!("`{list}` is a list: {doc}"))
        .iter()
        .map(|row| row["rollup_id"].as_str().expect("a rollup id").to_owned())
        .collect()
}

#[test]
fn adv_rollup_signal_no_definition_has_is_refused() {
    let mut world = World::new("adv-unknown-signal");
    world.sample(DEPTH, &[], "2026-03-04T07:00:00Z", ok("3"));
    let head = world.store().head();
    // A typo of the signal id: no definition has it (thalamus.rollup.UnknownSignal), and the
    // command must not answer "every series of the day is rolled up" (exit 0) with nothing done.
    let out = world.thalamus(&["--day", "2026-03-04", "--signal", "service.queue.dpeth"]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "stdout {} stderr {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(world.store().head(), head, "nothing is written");
}
