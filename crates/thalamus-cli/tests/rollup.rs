//! `thalamus rollup [--day YYYY-MM-DD] [--signal <id>]`: the built binary against a store fixture.
//! Each test copies the fixture instance (`tests/fixtures/instance`) into its own directory under
//! the build's temporary directory, appends the samples and evaluations it needs to the
//! instance's default store and rolls up. No source is read: `--connectors` names a binary that
//! does not exist and `PATH` is empty.
//!
//! Every expected figure is a literal worked out by hand from the stored samples.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SignalSampleData,
    TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};
use time::OffsetDateTime;

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

fn unknown() -> Reading {
    Reading::Unknown(UnreadReading {
        reason: "read timed out".to_owned(),
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
fn rollup_rerun_writes_nothing() {
    let mut world = World::new("rerun");
    let first = world.sample(DEPTH, &[], "2026-01-01T00:00:00Z", ok("4"));
    let second = world.sample(DEPTH, &[], "2026-01-01T12:00:00Z", ok("2"));
    world.sample(DEPTH, &[], "2026-01-01T18:00:00Z", unknown());
    world.sample(DEPTH, &[], "2026-01-02T00:00:00Z", ok("99"));
    world.evaluate(&first, Verdict::OK, Some("0"));
    world.evaluate(&second, Verdict::WARN, Some("1.5"));

    let doc = world.json(&["--day", "2026-01-01"]);
    assert_eq!(ids(&doc, "recorded"), ["service.queue.depth@2026-01-01"]);
    assert_eq!(ids(&doc, "held"), Vec::<String>::new());
    let row = &doc["recorded"][0];
    assert_eq!(row["day"], "2026-01-01T00:00:00Z");
    assert_eq!(
        (&row["n"], &row["unknown"], &row["partial"]),
        (&3.into(), &1.into(), &0.into())
    );
    assert_eq!(
        (&row["min"], &row["p50"], &row["max"], &row["last"]),
        (&"2".into(), &"3".into(), &"4".into(), &"2".into())
    );
    assert_eq!(row["worst_verdict"], "WARN");
    assert_eq!(
        (&row["min_score"], &row["max_score"]),
        (&"0".into(), &"1.5".into())
    );
    let head = world.store().head();
    assert_eq!(world.store().rollups().len(), 1);

    // The re-run writes nothing and prints the row that exists.
    let again = world.json(&["--day", "2026-01-01"]);
    assert_eq!(ids(&again, "recorded"), Vec::<String>::new());
    assert_eq!(ids(&again, "held"), ["service.queue.depth@2026-01-01"]);
    assert_eq!(again["held"][0], doc["recorded"][0]);
    assert_eq!(world.store().head(), head, "the re-run appended nothing");
    assert_eq!(world.store().rollups().len(), 1);

    // A sample added to the day after its rollup does not change or double the row.
    world.sample(DEPTH, &[], "2026-01-01T20:00:00Z", ok("1"));
    let late = world.json(&["--day", "2026-01-01"]);
    assert_eq!(ids(&late, "recorded"), Vec::<String>::new());
    assert_eq!(late["held"][0], doc["recorded"][0]);
    assert_eq!(world.store().rollups().len(), 1);

    // The text form names the row on a re-run too.
    let out = world.thalamus(&["--day", "2026-01-01"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("service.queue.depth@2026-01-01"),
        "the text form names the held row: {text}"
    );
}

#[test]
fn rollup_refuses_open_day() {
    let mut world = World::new("open-day");
    let now = OffsetDateTime::now_utc();
    let today = format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        u8::from(now.month()),
        now.day()
    );
    let tomorrow_at = now + time::Duration::days(1);
    let tomorrow = format!(
        "{:04}-{:02}-{:02}",
        tomorrow_at.year(),
        u8::from(tomorrow_at.month()),
        tomorrow_at.day()
    );
    world.sample(DEPTH, &[], &format!("{today}T00:00:00Z"), ok("1"));
    let head = world.store().head();
    for day in [&today, &tomorrow] {
        let out = world.thalamus(&["--day", day]);
        assert_eq!(out.status.code(), Some(64), "rollup --day {day}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("has not closed"),
            "rollup --day {day} says why: {stderr}"
        );
        assert!(out.stdout.is_empty(), "nothing is printed for {day}");
    }
    assert_eq!(world.store().head(), head, "an open day writes nothing");
    assert!(world.store().rollups().is_empty());

    // A day that is not YYYY-MM-DD is a usage error too.
    for day in ["2026-1-01", "2026-02-30", "yesterday"] {
        let out = world.thalamus(&["--day", day]);
        assert_eq!(out.status.code(), Some(64), "rollup --day {day}");
    }
}

#[test]
fn rollup_id_spelling_is_series_key_and_day() {
    let mut world = World::new("id-spelling");
    world.sample(
        MISS,
        &[("component", "search")],
        "2026-03-04T05:00:00Z",
        ok("0.5"),
    );
    world.sample(
        MISS,
        &[("component", "checkout")],
        "2026-03-04T06:00:00Z",
        ok("0.25"),
    );
    world.sample(DEPTH, &[], "2026-03-04T07:00:00Z", ok("3"));
    world.sample(DEPTH, &[], "2026-03-05T07:00:00Z", ok("3"));

    // --signal rolls up that signal's series only.
    let doc = world.json(&["--day", "2026-03-04", "--signal", MISS]);
    assert_eq!(
        ids(&doc, "recorded"),
        [
            r#"service.cache.miss_ratio{component="checkout"}@2026-03-04"#,
            r#"service.cache.miss_ratio{component="search"}@2026-03-04"#,
        ]
    );
    for row in doc["recorded"].as_array().expect("rows") {
        assert_eq!(row["signal"], MISS);
        let series = row["series"].as_str().expect("a series key");
        assert_eq!(
            row["rollup_id"].as_str().expect("an id"),
            format!("{series}@2026-03-04"),
            "the id is the series key, `@`, and the UTC day"
        );
    }

    let doc = world.json(&["--day", "2026-03-04"]);
    assert_eq!(ids(&doc, "recorded"), ["service.queue.depth@2026-03-04"]);
    assert_eq!(ids(&doc, "held").len(), 2);

    let stored: Vec<(String, String, String)> = world
        .store()
        .rollups()
        .into_iter()
        .map(|row| (row.rollup_id.0, row.series.0, row.day.0))
        .collect();
    for (id, series, day) in &stored {
        assert_eq!(*id, format!("{series}@{}", &day[..10]));
        assert_eq!(&day[10..], "T00:00:00Z");
    }
    assert_eq!(stored.len(), 3);
}
