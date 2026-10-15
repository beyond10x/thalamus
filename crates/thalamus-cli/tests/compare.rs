//! `thalamus compare`: the built binary against a store fixture. Each test copies the fixture
//! instance (`tests/fixtures/instance`) into its own directory under the build's temporary
//! directory, appends the samples it needs to the instance's default store
//! (`sqlite:<instance>/.thalamus/store.sqlite`) and compares. No source is read: `--connectors`
//! names a binary that does not exist and `PATH` is empty, so any source read would fail.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

/// `history: store`, no dimensions, cadence `PT5M`: a match tolerance of 150 seconds.
const DEPTH: &str = "service.queue.depth";
/// Dimension `component`, cadence `1h`: a match tolerance of 30 minutes.
const RATIO: &str = "service.api.request_failure_ratio";

/// The instant compared at, and the current sample's `ts` before it.
const AT: &str = "2026-10-14T23:00:00Z";
const NOW: &str = "2026-10-14T22:57:03Z";
/// `NOW` minus 7 days.
const WEEK_AGO: &str = "2026-10-07T22:57:03Z";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_compare")
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

    /// Appends one sample of `signal` with `dims` at `ts` to the instance's store.
    fn store(&mut self, signal: &str, dims: &[(&str, &str)], ts: &str, reading: Reading) {
        let dims: BTreeMap<String, DimensionValue> = dims
            .iter()
            .map(|(key, value)| ((*key).to_owned(), DimensionValue((*value).to_owned())))
            .collect();
        let series = if dims.is_empty() {
            signal.to_owned()
        } else {
            let labels: Vec<String> = dims
                .iter()
                .map(|(key, value)| format!("{key}=\"{}\"", value.0))
                .collect();
            format!("{signal}{{{}}}", labels.join(","))
        };
        let sample = SignalSampleData {
            sample_id: SampleId(format!("smp_{:012}", self.next)),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(series),
            dims,
            ts: Timestamp(ts.to_owned()),
            window: TimeWindow {
                start: Timestamp(ts.to_owned()),
                end: Timestamp(ts.to_owned()),
            },
            reading,
            text: None,
            unit: Unit("1".to_owned()),
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
        let locator = Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"));
        let mut store = EventlogStore::open(&locator, INSTANCE).expect("the store opens");
        store.append_sample(&sample).expect("the sample is stored");
    }

    fn thalamus(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(self.root.join("no-connectors-here"))
            .arg("compare")
            .args(args)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("PATH", self.root.join("empty-path"))
            .output()
            .expect("the thalamus binary runs")
    }

    /// `compare … --format json`, its exit code and its one JSON document.
    fn json(&self, args: &[&str]) -> (Option<i32>, Value) {
        let mut all = args.to_vec();
        all.extend(["--format", "json"]);
        let out = self.thalamus(&all);
        let doc = serde_json::from_slice(&out.stdout).unwrap_or_else(|error| {
            panic!(
                "stdout is one JSON document ({error}): {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.code(), doc)
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

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn compare_vs_7d_reads_stored_samples() {
    let mut world = World::new("vs_7d");
    world.store(DEPTH, &[], "2026-10-01T00:00:00Z", ok("1"));
    // 57 s after the slot a week earlier: inside half the 5-minute cadence.
    world.store(DEPTH, &[], "2026-10-07T22:58:00Z", ok("40"));
    // Further from the slot than the one above, still inside the tolerance.
    world.store(DEPTH, &[], "2026-10-07T22:55:00Z", ok("999"));
    world.store(DEPTH, &[], NOW, ok("42"));
    // After `--at`: not the current sample.
    world.store(DEPTH, &[], "2026-10-14T23:30:00Z", ok("77"));

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["series"], DEPTH, "{doc}");
    assert_eq!(doc["vs"], "7d", "{doc}");
    assert_eq!(doc["current"]["ts"], NOW, "{doc}");
    assert_eq!(doc["current"]["value"], "42", "{doc}");
    assert_eq!(doc["baseline"]["ts"], "2026-10-07T22:58:00Z", "{doc}");
    assert_eq!(doc["baseline"]["value"], "40", "{doc}");
    assert_eq!(doc["delta"], "2", "{doc}");
    assert_eq!(doc["ratio"], "1.05", "{doc}");

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    for expected in ["42", "40", "delta", "ratio", "1.05", NOW] {
        assert!(printed.contains(expected), "{expected} in {printed}");
    }
    assert!(!printed.contains("UNKNOWN"), "{printed}");
    assert!(
        !printed.contains("999") && !printed.contains("77"),
        "{printed}"
    );
}

#[test]
fn compare_missing_offset_sample_is_unknown_exit_3() {
    let mut world = World::new("missing_offset");
    world.store(DEPTH, &[], "2026-10-01T00:00:00Z", ok("1"));
    // Ten minutes after the slot a week earlier: outside half the 5-minute cadence.
    world.store(DEPTH, &[], "2026-10-07T23:07:03Z", ok("40"));
    world.store(DEPTH, &[], NOW, ok("42"));

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(printed.contains("UNKNOWN"), "{printed}");
    assert!(printed.contains("no stored sample at offset"), "{printed}");
    assert!(
        !printed.contains("delta") && !printed.contains("ratio"),
        "{printed}"
    );

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["result"], "UNKNOWN", "{doc}");
    assert_eq!(doc["reason"], "no stored sample at offset", "{doc}");
    assert!(doc["baseline"].is_null(), "{doc}");
    assert!(
        doc.get("delta").is_none() && doc.get("ratio").is_none(),
        "{doc}"
    );

    // No current sample at all: also UNKNOWN, exit 3.
    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", "2026-09-01T00:00:00Z"]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    assert!(text(&out).contains("UNKNOWN"), "{}", text(&out));
}

#[test]
fn compare_offset_before_series_is_unknown() {
    let mut world = World::new("before_series");
    // The series starts five days before NOW: a week earlier it did not exist.
    world.store(DEPTH, &[], "2026-10-09T22:57:03Z", ok("41"));
    world.store(DEPTH, &[], NOW, ok("42"));

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(printed.contains("UNKNOWN"), "{printed}");
    assert!(printed.contains("baseline predates series"), "{printed}");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["result"], "UNKNOWN", "{doc}");
    assert_eq!(doc["reason"], "baseline predates series", "{doc}");
    assert!(
        doc.get("delta").is_none() && doc.get("ratio").is_none(),
        "{doc}"
    );
}

#[test]
fn compare_unknown_side_never_prints_a_number() {
    let cases: [(&str, Reading, Reading, &str); 4] = [
        ("current_unknown", ok("40"), unknown("timeout"), "current"),
        ("baseline_unknown", unknown("timeout"), ok("42"), "baseline"),
        ("current_partial", ok("40"), partial("987.25"), "current"),
        ("baseline_partial", partial("123.5"), ok("42"), "baseline"),
    ];
    for (name, before, now, side) in cases {
        let mut world = World::new(name);
        world.store(DEPTH, &[], "2026-10-01T00:00:00Z", ok("1"));
        world.store(DEPTH, &[], WEEK_AGO, before);
        world.store(DEPTH, &[], NOW, now);

        let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
        assert_eq!(out.status.code(), Some(3), "{name}: {}", text(&out));
        let printed = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(printed.contains("UNKNOWN"), "{name}: {printed}");
        assert!(
            !printed.contains("delta") && !printed.contains("ratio"),
            "{name}: {printed}"
        );
        assert!(
            !printed.contains("987.25") && !printed.contains("123.5"),
            "{name}: a partial bound is printed: {printed}"
        );
        let line = printed
            .lines()
            .find(|line| line.starts_with(side))
            .unwrap_or_else(|| panic!("{name}: a {side} line in {printed}"));
        let shown = line
            .split_whitespace()
            .skip(2)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            !shown.chars().any(|c| c.is_ascii_digit()),
            "{name}: the {side} side shows a number: {line}"
        );

        let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
        assert_eq!(code, Some(3), "{name}: {doc}");
        assert_eq!(doc["result"], "UNKNOWN", "{name}: {doc}");
        assert!(doc[side].is_object(), "{name}: {doc}");
        assert!(
            doc[side].get("value").is_none(),
            "{name}: the {side} side carries a value: {doc}"
        );
        assert!(
            doc.get("delta").is_none() && doc.get("ratio").is_none(),
            "{name}: {doc}"
        );
    }
}

#[test]
fn compare_series_key_from_dims() {
    let mut world = World::new("dims");
    for (component, before, now) in [("checkout", "0.01", "0.03"), ("search", "0.25", "0.5")] {
        let dims = [("component", component)];
        world.store(RATIO, &dims, "2026-10-01T00:00:00Z", ok("0.1"));
        // 20 minutes off the slot: inside half the 1-hour cadence.
        world.store(RATIO, &dims, "2026-10-07T23:17:03Z", ok(before));
        world.store(RATIO, &dims, NOW, ok(now));
    }

    let (code, doc) = world.json(&[RATIO, "--dim", "component=search", "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(
        doc["series"],
        format!("{RATIO}{{component=\"search\"}}"),
        "{doc}"
    );
    assert_eq!(doc["current"]["value"], "0.5", "{doc}");
    assert_eq!(doc["baseline"]["value"], "0.25", "{doc}");
    assert_eq!(doc["delta"], "0.25", "{doc}");
    assert_eq!(doc["ratio"], "2", "{doc}");

    let (code, doc) = world.json(&[
        RATIO,
        "--dim",
        "component=checkout",
        "--vs",
        "7d",
        "--at",
        AT,
    ]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["delta"], "0.02", "{doc}");
    assert_eq!(doc["ratio"], "3", "{doc}");

    // No dims: the bare id, a series nothing was stored under.
    let (code, doc) = world.json(&[RATIO, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["series"], RATIO, "{doc}");
    assert_eq!(doc["result"], "UNKNOWN", "{doc}");

    // A key the definition does not declare is refused, not compared.
    let out = world.thalamus(&[RATIO, "--dim", "environment=prod", "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(65), "{}", text(&out));
    // A malformed --dim is a usage error.
    let out = world.thalamus(&[RATIO, "--dim", "component", "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(64), "{}", text(&out));
}
