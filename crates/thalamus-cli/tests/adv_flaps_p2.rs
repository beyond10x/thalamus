//! Adversary pass 2 on `story:m2-flap-measurement`: `thalamus flaps` against store fixtures,
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

use Verdict::{OK, WARN};

struct World {
    root: PathBuf,
    next: u64,
    now: OffsetDateTime,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_flaps_p2")
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

    /// Revises the gauge to `rev` with `cadence` in this test's copy, as an operator's edit does.
    fn revise_cadence(&self, rev: i64, cadence: &str) {
        let path = self.instance().join(format!("definitions/{DEPTH}.yaml"));
        let text = std::fs::read_to_string(&path).expect("read the definition");
        assert!(
            text.contains("\ncadence: PT5M\n"),
            "the fixture samples every 5 minutes"
        );
        std::fs::write(
            &path,
            text.replace("\nrev: 1\n", &format!("\nrev: {rev}\n"))
                .replace("\ncadence: PT5M\n", &format!("\ncadence: {cadence}\n")),
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

/// A day of 5-minute samples (rev 1, cadence PT5M), then the operator revises the definition to
/// rev 2 with `cadence: PT1H` and re-grades the history at rev 2 (the workflow pass 1 F1 names).
/// The kept rev-2 evaluations cover 23h55m of wall time in a `--since 1d` window: one
/// series-day. `flaps.rs:272` prices every kept evaluation at the current definition's cadence,
/// so 288 evaluations x 1 h = 12 series-days inside a one-day window, and every rate the k/m
/// decision reads is divided by 12.
#[test]
fn adv_series_days_after_cadence_revision_stay_within_the_window() {
    let mut world = World::new("cadence_regraded");
    let verdicts: Vec<Verdict> = (0..288)
        .map(|i| if i % 24 == 12 { WARN } else { OK })
        .collect();
    let history = world.series(0, 5, &verdicts);
    world.revise_cadence(2, "PT1H");
    let mut prev = None;
    for ((sample, ts), verdict) in history.iter().zip(&verdicts) {
        world.evaluate(sample, ts, 2, *verdict, prev);
        prev = Some(*verdict);
    }

    let doc = world.json(&["--since", "1d"]);
    let row = service(&doc);
    assert_eq!(row["raw_changes"], 24, "{row}");
    let days = row["series_days"]
        .as_f64()
        .expect("series_days is a number");
    assert!(
        days <= 1.0,
        "one series observed for 23h55m inside a 1d window is at most one series-day, got {days}: {row}"
    );
}

/// The same root through `--rev`: the definition is now rev 2 with `cadence: PT1H`; rev 1 was
/// sampled every 5 minutes for the last 23h55m. `--rev 1 --since 1d` measures those 288 rev-1
/// evaluations, which observed one day, and reports 12 series-days (rev 2's cadence).
#[test]
fn adv_rev_of_an_older_cadence_counts_its_own_observed_time() {
    let mut world = World::new("cadence_rev");
    let verdicts: Vec<Verdict> = (0..288)
        .map(|i| if i % 24 == 12 { WARN } else { OK })
        .collect();
    world.series(0, 5, &verdicts);
    world.revise_cadence(2, "PT1H");

    let doc = world.json(&["--rev", "1", "--since", "1d"]);
    let row = service(&doc);
    assert_eq!(row["raw_changes"], 24, "{row}");
    let days = row["series_days"]
        .as_f64()
        .expect("series_days is a number");
    assert!(
        days <= 1.0,
        "288 rev-1 evaluations of a 5-minute cadence are one series-day, got {days}: {row}"
    );
}

/// `flaps.rs:21` promises each per-series-day count is "absent when there are no series-days,
/// never 0". One evaluation of a 30-second cadence observes 30 s, which `series_days` rounds to
/// 0 (`flaps.rs:399`) while `per_day` (`flaps.rs:410`) divides by the unrounded 30 s: the
/// document says 0 series-days and 2880 raw changes per series-day.
#[test]
fn adv_zero_series_days_carry_no_rate() {
    let mut world = World::new("zero_days");
    world.revise_cadence(1, "PT30S");
    let ts = world.at(1);
    let sample = world.sample(&ts);
    world.evaluate(&sample, &ts, 1, WARN, Some(OK));

    let doc = world.json(&["--since", "1h"]);
    let row = service(&doc);
    assert_eq!(row["raw_changes"], 1, "{row}");
    if row["series_days"] == 0 {
        assert!(
            row["raw_per_series_day"].is_null(),
            "0 series-days reported with a rate per series-day: {row}"
        );
    }
}
