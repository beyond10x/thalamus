//! `thalamus flaps`: the built binary against a store fixture. Each test copies the fixture
//! instance (`tests/fixtures/instance`) into its own directory under the build's temporary
//! directory, appends the samples and evaluations it needs to the instance's default store and
//! measures. No source is read: `--connectors` names a binary that does not exist and `PATH` is
//! empty.
//!
//! Every expected figure is a literal worked out by hand from design § 3.2 and the stored
//! verdicts, never recomputed through the implementation's own replay. Both definitions are
//! sampled hourly here (`cadence: PT1H`), so 24 kept evaluations of one series are one series-day.

use std::collections::{BTreeMap, BTreeSet};
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

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";
/// Family `service`, no dimensions, rev 1, no `persistence` block, cadence one hour here.
const DEPTH: &str = "service.queue.depth";
/// Family `platform`, no dimensions, rev 1: a copy of the gauge added by [`World::new`].
const PLATFORM: &str = "platform.queue.depth";

use Verdict::{CRIT, OK, WARN};

struct World {
    root: PathBuf,
    next: u64,
    now: OffsetDateTime,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_flaps")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
            &root.join("instance"),
        );
        std::fs::create_dir_all(root.join("empty-path")).expect("create the empty PATH directory");
        let definitions = root.join("instance/definitions");
        let gauge = std::fs::read_to_string(definitions.join(format!("{DEPTH}.yaml")))
            .expect("read the gauge definition")
            .replace("cadence: PT5M", "cadence: PT1H");
        std::fs::write(definitions.join(format!("{DEPTH}.yaml")), &gauge)
            .expect("write the gauge definition");
        std::fs::write(
            definitions.join(format!("{PLATFORM}.yaml")),
            gauge
                .replace(&format!("id: {DEPTH}"), &format!("id: {PLATFORM}"))
                .replace("family: service", "family: platform"),
        )
        .expect("write the platform definition");
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

    /// Declares `persistence` on `signal`'s definition, in this test's copy.
    fn persistence(&self, signal: &str, k: usize, k_crit: usize, m: usize) {
        let path = self.instance().join(format!("definitions/{signal}.yaml"));
        let text = std::fs::read_to_string(&path).expect("read the definition");
        std::fs::write(
            &path,
            text.replace(
                "owner: ",
                &format!("persistence: {{ k: {k}, k_crit: {k_crit}, m: {m} }}\nowner: "),
            ),
        )
        .expect("write the definition");
    }

    /// The RFC 3339 instant `hours` before now.
    fn ago(&self, hours: i64) -> String {
        (self.now - Duration::hours(hours))
            .format(&Rfc3339)
            .expect("an RFC 3339 instant")
    }

    /// Stores one sample of `signal` (no dimensions) at `ts` and returns its id.
    fn sample(&mut self, signal: &str, ts: &str) -> SampleId {
        let sample_id = SampleId(format!("smp_{:012}", self.next));
        self.next += 1;
        let sample = SignalSampleData {
            sample_id: sample_id.clone(),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(signal.to_owned()),
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

    /// Stores one evaluation of `sample` and returns it.
    fn evaluate(
        &mut self,
        sample: &SampleId,
        signal: &str,
        ts: &str,
        rev: i64,
        verdict: Verdict,
        prev: Option<Verdict>,
    ) -> SignalEvaluationData {
        let evaluation = SignalEvaluationData {
            evaluation_id: EvaluationId(format!("evl_{:012}", self.next)),
            sample_id: sample.clone(),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(signal.to_owned()),
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

    /// One series of `signal`: a sample per verdict, one hour apart and ending `end` hours ago,
    /// each evaluated once at `rev` with the raw previous verdict as `prev`.
    fn series(
        &mut self,
        signal: &str,
        end: i64,
        rev: i64,
        verdicts: &[Verdict],
    ) -> Vec<SignalEvaluationData> {
        let mut prev = None;
        let mut stored = Vec::new();
        for (index, verdict) in verdicts.iter().enumerate() {
            let hours = end + i64::try_from(verdicts.len() - 1 - index).expect("few verdicts");
            let ts = self.ago(hours);
            let sample = self.sample(signal, &ts);
            stored.push(self.evaluate(&sample, signal, &ts, rev, *verdict, prev));
            prev = Some(*verdict);
        }
        stored
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

    /// `flaps … --format json`: exit 0 and its one JSON document.
    fn json(&self, args: &[&str]) -> Value {
        let mut all = args.to_vec();
        all.extend(["--format", "json"]);
        let out = self.thalamus(&all);
        assert_eq!(
            out.status.code(),
            Some(0),
            "thalamus flaps {all:?}: {}",
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

/// The family `name` of a report.
fn family<'a>(doc: &'a Value, name: &str) -> &'a Value {
    doc["families"]
        .as_array()
        .expect("families is a list")
        .iter()
        .find(|family| family["family"] == name)
        .unwrap_or_else(|| panic!("family {name} in {doc}"))
}

/// `field` of `row` as a number.
fn number(row: &Value, field: &str) -> Option<f64> {
    row[field].as_f64()
}

const FLAPPY: [Verdict; 10] = [OK, WARN, OK, WARN, WARN, OK, CRIT, OK, OK, OK];

/// Raw changes of `FLAPPY` under `prev`: OK>WARN, WARN>OK, OK>WARN, WARN>OK, OK>CRIT, CRIT>OK, six
/// per block; one block's last OK to the next block's first OK is no change.
#[test]
fn flaps_raw_changes_match_verdict_changes_view() {
    let mut world = World::new("raw_changes");
    // 30 hours, so the series spans two UTC days or more.
    let mut long = FLAPPY.repeat(3);
    long.truncate(30);
    world.series(DEPTH, 1, 1, &long);
    world.series(PLATFORM, 2, 1, &[OK, OK, CRIT, CRIT, OK]);
    // Before --since: read only to seed the settled replay, neither counted nor excluded.
    let old = world.ago(24 * 40);
    let sample = world.sample(DEPTH, &old);
    world.evaluate(&sample, DEPTH, &old, 1, WARN, Some(OK));

    let doc = world.json(&["--since", "30d"]);
    assert_eq!(doc["excluded"], 0, "{doc}");
    // 30 hourly evaluations are 1.25 series-days; 18 raw changes over them are 14.4 a day.
    let row = family(&doc, "service");
    assert_eq!(row["series"], 1, "{row}");
    assert_eq!(number(row, "series_days"), Some(1.25), "{row}");
    assert_eq!(row["raw_changes"], 18, "{row}");
    assert_eq!(number(row, "raw_per_series_day"), Some(14.4), "{row}");
    // OK>CRIT and CRIT>OK over 5 hourly evaluations: 5/24 = 0.208 series-days, 9.615 a day.
    let row = family(&doc, "platform");
    assert_eq!(row["series"], 1, "{row}");
    assert_eq!(number(row, "series_days"), Some(0.208), "{row}");
    assert_eq!(row["raw_changes"], 2, "{row}");
    assert_eq!(number(row, "raw_per_series_day"), Some(9.615), "{row}");
}

/// `FLAPPY` twice is the runs OK1 W1 OK1 W2 OK1 C1 OK4 W1 OK1 W2 OK1 C1 OK3 (20 hourly
/// evaluations, 0.833 series-days, 12 raw changes). A run of V settles V once it is as long as V
/// needs and V is not already settled; the first settled verdict is a start, not counted.
#[test]
fn flaps_settled_changes_match_hysteresis_replay() {
    let mut world = World::new("settled_changes");
    world.persistence(PLATFORM, 3, 2, 1);
    world.series(DEPTH, 1, 1, &FLAPPY.repeat(2));
    world.series(PLATFORM, 1, 1, &FLAPPY.repeat(2));

    // Each definition's own persistence.
    let doc = world.json(&["--since", "7d"]);
    // Defaults k 2, k_crit 1, m 3: W2 (start), C1, OK4, W2, C1, OK3 settle: 5, 6.002 a day.
    let row = family(&doc, "service");
    let runs = row["settled"].as_array().expect("settled is a list");
    assert_eq!(runs.len(), 1, "{row}");
    assert_eq!(number(row, "series_days"), Some(0.833), "{row}");
    assert_eq!(row["raw_changes"], 12, "{row}");
    assert_eq!(runs[0]["settled_changes"], 5, "{row}");
    assert_eq!(
        number(&runs[0], "settled_per_series_day"),
        Some(6.002),
        "{row}"
    );
    assert!(runs[0]["k"].is_null(), "{row}");
    // Declared k 3, k_crit 2, m 1: OK1 settles first; no WARN run reaches 3 and no CRIT run 2.
    let row = family(&doc, "platform");
    let runs = row["settled"].as_array().expect("settled is a list");
    assert_eq!(runs.len(), 1, "{row}");
    assert_eq!(runs[0]["settled_changes"], 0, "{row}");
    assert_eq!(
        number(&runs[0], "settled_per_series_day"),
        Some(0.0),
        "{row}"
    );
    assert!(runs[0]["k"].is_null(), "{row}");

    // --k, --k-crit and --m override every definition's own. k 1, k_crit 1, m 2: W1 (start), C1,
    // OK4, W1, C1, OK3 settle: 5 for both.
    let doc = world.json(&["--since", "7d", "--k", "1", "--k-crit", "1", "--m", "2"]);
    for name in ["service", "platform"] {
        let row = family(&doc, name);
        let run = &row["settled"][0];
        assert_eq!(run["settled_changes"], 5, "{name}: {row}");
        assert_eq!(
            (&run["k"], &run["k_crit"], &run["m"]),
            (&1.into(), &1.into(), &2.into())
        );
    }
}

/// A WARN that settled before `--since` and clears inside it is a settled change of the window:
/// the replay is seeded with the series' kept evaluations before the window and counts only the
/// changes that settle inside it.
///
/// Hourly, 9 h to 1 h ago: OK OK OK WARN WARN WARN | OK OK OK, the window the last 3.5 h. Before
/// it OK settles (start) and WARN settles (before the window, not counted); inside it OK settles
/// from WARN: 1 settled change, and 1 raw change (the first OK, `prev` WARN).
#[test]
fn flaps_settled_counts_a_clear_of_a_warn_settled_before_the_window() {
    let mut world = World::new("seeded");
    world.series(DEPTH, 1, 1, &[OK, OK, OK, WARN, WARN, WARN, OK, OK, OK]);

    let doc = world.json(&["--since", "210m"]);
    assert_eq!(doc["excluded"], 0, "{doc}");
    let row = family(&doc, "service");
    assert_eq!(row["settled"][0]["settled_changes"], 1, "{row}");
    assert_eq!(row["series"], 1, "{row}");
    assert_eq!(row["raw_changes"], 1, "{row}");
    assert_eq!(number(row, "series_days"), Some(0.125), "{row}");
}

#[test]
fn flaps_excludes_other_revisions() {
    let mut world = World::new("revisions");
    let current = world.series(DEPTH, 10, 1, &[OK, WARN, WARN, OK, OK, OK]);
    // Re-grades of three of those samples at revision 2, which the instance does not have yet.
    let mut prev = None;
    for (evaluation, verdict) in current.iter().take(3).zip([CRIT, CRIT, OK]) {
        let ts = evaluation.ts.0.clone();
        world.evaluate(&evaluation.sample_id, DEPTH, &ts, 2, verdict, prev);
        prev = Some(verdict);
    }

    // Rev 1: OK>WARN and WARN>OK raw; WARN settles (start), then OK after three: 1 settled.
    let doc = world.json(&["--since", "2d"]);
    assert_eq!(doc["rev"], "current", "{doc}");
    assert_eq!(doc["excluded"], 3, "{doc}");
    let row = family(&doc, "service");
    assert_eq!(row["raw_changes"], 2, "{row}");
    assert_eq!(row["settled"][0]["settled_changes"], 1, "{row}");

    // Rev 2: CRIT CRIT OK, so CRIT>OK is the one raw change; 3 hourly evaluations, 0.125 days.
    let doc = world.json(&["--since", "2d", "--rev", "2"]);
    assert_eq!(doc["rev"], 2, "{doc}");
    assert_eq!(doc["excluded"], 6, "{doc}");
    let row = family(&doc, "service");
    assert_eq!(row["raw_changes"], 1, "{row}");
    assert_eq!(number(row, "series_days"), Some(0.125), "{row}");

    // A revision nothing was graded at keeps nothing and reports no family.
    let doc = world.json(&["--since", "2d", "--rev", "7"]);
    assert_eq!(doc["excluded"], 9, "{doc}");
    assert_eq!(doc["families"], Value::Array(Vec::new()), "{doc}");
}

/// The sweep over the runs of `FLAPPY` twice (see `flaps_settled_changes_match_hysteresis_replay`),
/// by hand, `(k, m) => settled`. The gauge keeps its own k_crit 1, so every CRIT run settles; the
/// copy declares k_crit 3, so none does.
#[test]
fn flaps_sweep_covers_k_and_m_grid() {
    let mut world = World::new("sweep");
    world.persistence(PLATFORM, 2, 3, 3);
    world.series(DEPTH, 1, 1, &FLAPPY.repeat(2));
    world.series(PLATFORM, 1, 1, &FLAPPY.repeat(2));

    let service: BTreeMap<(u64, u64), u64> = [
        ((1, 1), 12),
        ((1, 2), 5),
        ((1, 3), 5),
        ((1, 4), 4),
        ((2, 1), 8),
        ((2, 2), 5),
        ((2, 3), 5),
        ((2, 4), 4),
        ((3, 1), 4),
        ((3, 2), 3),
        ((3, 3), 3),
        ((3, 4), 2),
    ]
    .into();
    let platform: BTreeMap<(u64, u64), u64> = [
        ((1, 1), 8),
        ((1, 2), 3),
        ((1, 3), 3),
        ((1, 4), 2),
        ((2, 1), 4),
        ((2, 2), 3),
        ((2, 3), 3),
        ((2, 4), 2),
        ((3, 1), 0),
        ((3, 2), 0),
        ((3, 3), 0),
        ((3, 4), 0),
    ]
    .into();

    let doc = world.json(&["--since", "7d", "--sweep"]);
    for (name, expected) in [("service", &service), ("platform", &platform)] {
        let row = family(&doc, name);
        let runs = row["settled"].as_array().expect("settled is a list");
        let mut measured = BTreeMap::new();
        for run in runs {
            let k = run["k"].as_u64().expect("k is given");
            let m = run["m"].as_u64().expect("m is given");
            assert!(run["k_crit"].is_null(), "{name}: {run}");
            let settled = run["settled_changes"].as_u64().expect("a count");
            assert!(measured.insert((k, m), settled).is_none(), "{name}: {row}");
        }
        assert_eq!(runs.len(), 12, "{name}: {row}");
        assert_eq!(&measured, expected, "{name}: {row}");
        let grid: BTreeSet<(u64, u64)> = measured.keys().copied().collect();
        let full: BTreeSet<(u64, u64)> =
            (1..=3).flat_map(|k| (1..=4).map(move |m| (k, m))).collect();
        assert_eq!(grid, full, "{name}: {row}");
    }

    // --sweep chooses k and m itself.
    let out = world.thalamus(&["--since", "7d", "--sweep", "--k", "2"]);
    assert_eq!(out.status.code(), Some(64), "{out:?}");
}

#[test]
fn flaps_writes_nothing() {
    let mut world = World::new("writes_nothing");
    world.series(DEPTH, 1, 1, &FLAPPY);
    let before = EventlogStore::open(&world.locator(), INSTANCE).expect("the store opens");
    let (head, records) = (before.head(), before.records().len());
    drop(before);

    for args in [
        &["--since", "7d"][..],
        &["--since", "7d", "--sweep", "--format", "json"],
        &[
            "--since", "7d", "--rev", "1", "--k", "1", "--k-crit", "1", "--m", "1",
        ],
    ] {
        let out = world.thalamus(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let after = EventlogStore::open(&world.locator(), INSTANCE).expect("the store opens");
    assert_eq!((after.head(), after.records().len()), (head, records));

    // An instance with no store yet: an empty report, and no store is created.
    let empty = World::new("writes_nothing_empty");
    let doc = empty.json(&["--since", "7d"]);
    assert_eq!(doc["families"], Value::Array(Vec::new()), "{doc}");
    assert_eq!(doc["excluded"], 0, "{doc}");
    assert!(!empty.instance().join(".thalamus").exists());
}
