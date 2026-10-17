//! Adversary pass 1 on `story:m2-flap-measurement`: `thalamus flaps` against store fixtures,
//! asserting literal figures worked out by hand from design § 3.2, never figures recomputed by
//! the implementation's own `replay` call.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::evaluation::{EvaluationId, SignalEvaluationData, Verdict};
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

const INSTANCE: &str = "fixture";
/// Family `service`, rev 1, no `persistence` block: k 2, k_crit 1, m 3.
const DEPTH: &str = "service.queue.depth";

use Verdict::{CRIT, NA, OK, UNKNOWN, WARN};

struct World {
    root: PathBuf,
    next: u64,
    now: OffsetDateTime,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_flaps_p1")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("empty-path")).expect("create the empty PATH directory");
        let now = OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("a whole second");
        Self { root, next: 1, now }
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"))
    }

    /// Revises the gauge to `rev` in this test's copy, as an operator's edit does.
    fn revise(&self, rev: i64) {
        let path = self.instance().join(format!("definitions/{DEPTH}.yaml"));
        let text = std::fs::read_to_string(&path).expect("read the definition");
        assert!(text.contains("\nrev: 1\n"), "the fixture is at rev 1");
        std::fs::write(
            &path,
            text.replace("\nrev: 1\n", &format!("\nrev: {rev}\n")),
        )
        .expect("write the definition");
    }

    fn at(&self, minutes_ago: i64) -> String {
        (self.now - Duration::minutes(minutes_ago))
            .format(&Rfc3339)
            .expect("an RFC 3339 instant")
    }

    fn sample(&mut self, ts: &str) -> SampleId {
        let sample_id = SampleId(format!("smp_{:012}", self.next));
        self.next += 1;
        let sample = SignalSampleData {
            sample_id: sample_id.clone(),
            signal: SignalId(DEPTH.to_owned()),
            series: SeriesKey(DEPTH.to_owned()),
            dims: BTreeMap::new(),
            ts: Timestamp(ts.to_owned()),
            window: TimeWindow {
                start: Timestamp(ts.to_owned()),
                end: Timestamp(ts.to_owned()),
            },
            reading: Reading::Ok(CompleteReading {
                value: Decimal("1".to_owned()),
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
        let mut store = EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens");
        store.append_sample(&sample).expect("the sample is stored");
        sample_id
    }

    fn evaluate(
        &mut self,
        sample: &SampleId,
        ts: &str,
        rev: i64,
        verdict: Verdict,
        prev: Option<Verdict>,
    ) -> SignalEvaluationData {
        let evaluation = SignalEvaluationData {
            evaluation_id: EvaluationId(format!("evl_{:012}", self.next)),
            sample_id: sample.clone(),
            signal: SignalId(DEPTH.to_owned()),
            series: SeriesKey(DEPTH.to_owned()),
            dims: BTreeMap::new(),
            ts: Timestamp(ts.to_owned()),
            definition_rev: rev,
            verdict,
            prev,
            score: None,
            baseline: None,
            reason: (verdict != OK).then(|| "graded by the test".to_owned()),
        };
        self.next += 1;
        let mut store = EventlogStore::open(&self.locator(), INSTANCE).expect("the store opens");
        store
            .append_evaluation(&evaluation)
            .expect("the evaluation is stored");
        evaluation
    }

    /// A sample per verdict, `step` minutes apart, the last `end` minutes ago, each graded at
    /// rev 1 with the raw previous verdict as `prev`. Returns (sample, ts) per verdict.
    fn series(&mut self, end: i64, step: i64, verdicts: &[Verdict]) -> Vec<(SampleId, String)> {
        let mut prev = None;
        let mut out = Vec::new();
        for (index, verdict) in verdicts.iter().enumerate() {
            let back = i64::try_from(verdicts.len() - 1 - index).expect("few");
            let ts = self.at(end + back * step);
            let sample = self.sample(&ts);
            self.evaluate(&sample, &ts, 1, *verdict, prev);
            prev = Some(*verdict);
            out.push((sample, ts));
        }
        out
    }

    fn thalamus(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(self.root.join("no-connectors-here"))
            .arg("flaps")
            .args(args)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("PATH", self.root.join("empty-path"))
            .output()
            .expect("the thalamus binary runs")
    }

    fn json(&self, args: &[&str]) -> Value {
        let mut all = args.to_vec();
        all.extend(["--format", "json"]);
        let out = self.thalamus(&all);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{all:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).expect("one JSON document")
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

fn service(doc: &Value) -> &Value {
    doc["families"]
        .as_array()
        .expect("families is a list")
        .iter()
        .find(|family| family["family"] == "service")
        .unwrap_or_else(|| panic!("family service in {doc}"))
}

/// The documented re-grade after a revision: the service grades only its own run's samples
/// (`scheduler.rs:391`, `Some(&pass.run_id)`), so the new samples are graded at rev 2 first and
/// `thalamus eval` re-grades the older history at rev 2 afterwards, with later ids. Measuring rev
/// 2 must measure the rev-2 verdicts of every kept sample: raw changes and series-days count the
/// re-graded history, so settled changes must too.
///
/// Rev-2 verdicts in `ts` order, default persistence (k 2, m 3):
/// OK WARN WARN | OK OK OK | WARN WARN | OK OK OK | OK OK OK OK
/// settles WARN (first, not counted), OK (1), WARN (2), OK (3): 3 settled changes.
#[test]
fn adv_settled_counts_history_regraded_after_revision() {
    let mut world = World::new("regraded_history");
    let history = world.series(600, 60, &[OK; 11]);
    world.revise(2);
    // The service, at rev 2, grades its new samples as they arrive.
    let mut prev = Some(OK);
    for minutes in [240, 180, 120, 60] {
        let ts = world.at(minutes);
        let sample = world.sample(&ts);
        world.evaluate(&sample, &ts, 2, OK, prev);
        prev = Some(OK);
    }
    // `thalamus eval` then re-grades the history at rev 2.
    let regrade = [OK, WARN, WARN, OK, OK, OK, WARN, WARN, OK, OK, OK];
    let mut prev = None;
    for ((sample, ts), verdict) in history.iter().zip(regrade) {
        world.evaluate(sample, ts, 2, verdict, prev);
        prev = Some(verdict);
    }

    let doc = world.json(&["--since", "2d"]);
    assert_eq!(doc["excluded"], 11, "{doc}");
    let row = service(&doc);
    assert_eq!(
        row["raw_changes"], 4,
        "the re-graded history is counted raw: {row}"
    );
    assert_eq!(
        row["settled"][0]["settled_changes"], 3,
        "settled must be measured over the same rev-2 samples raw counts: {row}"
    );
}

/// A series sampled on its 5-minute cadence for the last 23h55m: 288 kept evaluations, one day
/// of observation (288 x 5 min / 24 h). A `--since 1d` run reads one day, so its rate per
/// series-day is per one day. Counting distinct UTC calendar days counts two for almost every
/// wall-clock time the command runs at, halving every per-series-day rate the k/m decision reads.
#[test]
fn adv_one_day_window_is_one_series_day() {
    let mut world = World::new("one_day");
    let verdicts: Vec<Verdict> = (0..288)
        .map(|i| if i % 24 == 12 { WARN } else { OK })
        .collect();
    world.series(0, 5, &verdicts);

    let doc = world.json(&["--since", "1d"]);
    let row = service(&doc);
    assert_eq!(row["raw_changes"], 24, "{row}");
    assert_eq!(
        row["series_days"], 1,
        "23h55m of one series at its cadence is one series-day: {row}"
    );
    assert_eq!(row["raw_per_series_day"].as_f64(), Some(24.0), "{row}");
}

/// The ladder, by hand (default k 2, k_crit 1, m 3): UNKNOWN and N/A settle from OK after k and
/// are never OK; neither clears a settled WARN; CRIT settles on one.
/// OK OK OK (first) | UNKNOWN UNKNOWN (1) | OK OK OK (2) | NA NA (3) | WARN WARN (4)
/// | UNKNOWN UNKNOWN NA NA (blocked) | CRIT (5) | OK OK OK (6)
#[test]
fn adv_unknown_and_na_in_the_ladder() {
    let mut world = World::new("ladder");
    world.series(
        60,
        60,
        &[
            OK, OK, OK, UNKNOWN, UNKNOWN, OK, OK, OK, NA, NA, WARN, WARN, UNKNOWN, UNKNOWN, NA, NA,
            CRIT, OK, OK, OK,
        ],
    );
    let doc = world.json(&["--since", "2d"]);
    let row = service(&doc);
    // Raw: OK>U, U>OK, OK>NA, NA>W, W>U, U>NA, NA>C, C>OK.
    assert_eq!(row["raw_changes"], 8, "{row}");
    assert_eq!(row["settled"][0]["settled_changes"], 6, "{row}");
}

/// `flaps` writes nothing: every file under the store's directory keeps its bytes and its
/// mtime, and no file appears; and two runs print the same bytes.
#[test]
fn adv_store_files_untouched_and_output_stable() {
    let mut world = World::new("untouched");
    world.series(60, 60, &[OK, WARN, WARN, OK, OK, OK, CRIT, OK, OK, OK]);
    let dir = world.instance().join(".thalamus");
    let snapshot = || {
        let mut files = BTreeMap::new();
        for entry in std::fs::read_dir(&dir).expect("read the store directory") {
            let entry = entry.expect("an entry");
            let meta = entry.metadata().expect("metadata");
            files.insert(
                entry.file_name().to_string_lossy().into_owned(),
                (
                    std::fs::read(entry.path()).expect("read a store file"),
                    meta.modified().expect("an mtime"),
                ),
            );
        }
        files
    };
    let before = snapshot();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let first = world.thalamus(&["--since", "7d", "--sweep"]);
    let second = world.thalamus(&["--since", "7d", "--sweep"]);
    let json_a = world.thalamus(&["--since", "7d", "--format", "json"]);
    let json_b = world.thalamus(&["--since", "7d", "--format", "json"]);
    assert_eq!(first.status.code(), Some(0));
    let after = snapshot();
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "no store file appears or vanishes"
    );
    for (name, (bytes, mtime)) in &before {
        let (bytes_after, mtime_after) = &after[name];
        assert!(bytes == bytes_after, "{name}: bytes changed");
        assert_eq!(mtime, mtime_after, "{name}: mtime changed");
    }
    assert_eq!(first.stdout, second.stdout, "text output is stable");
    assert_eq!(json_a.stdout, json_b.stdout, "JSON output is stable");
}
