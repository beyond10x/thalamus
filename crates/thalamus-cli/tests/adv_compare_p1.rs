//! Adversary pass 1 on `thalamus compare`: window edges, the series-age rule, the parser's
//! limits and the cases that pin lines the acceptance tests leave free. Each test copies the
//! fixture instance into its own directory under the build's temporary directory, appends the
//! samples it needs to the instance's default store and compares. No source is read.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{
    CompleteReading, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

const INSTANCE: &str = "fixture";
/// `history: store`, no dimensions, cadence `PT5M`: a match tolerance of 150 seconds.
const DEPTH: &str = "service.queue.depth";
const AT: &str = "2026-10-14T23:00:00Z";
const NOW: &str = "2026-10-14T22:57:03Z";
/// Long before every slot: the series is older than a week.
const FIRST: &str = "2026-10-01T00:00:00Z";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_compare_p1")
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

    /// Dates the gauge's series from its first stored sample, in this test's own copy of the
    /// fixture, instead of the fixture's fixed `since`.
    fn since_first_sample(&self) {
        let path = self.instance().join("definitions/service.queue.depth.yaml");
        let text = std::fs::read_to_string(&path).expect("read the gauge definition");
        std::fs::write(
            &path,
            text.replace("since: 2026-01-01T00:00:00Z", "since: first_sample"),
        )
        .expect("write the gauge definition");
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn store_path(&self) -> PathBuf {
        self.instance().join(".thalamus/store.sqlite")
    }

    fn store(&mut self, ts: &str, value: &str) {
        let sample = SignalSampleData {
            sample_id: SampleId(format!("smp_{:012}", self.next)),
            signal: SignalId(DEPTH.to_owned()),
            series: SeriesKey(DEPTH.to_owned()),
            dims: BTreeMap::<String, DimensionValue>::new(),
            ts: Timestamp(ts.to_owned()),
            window: TimeWindow {
                start: Timestamp(ts.to_owned()),
                end: Timestamp(ts.to_owned()),
            },
            reading: Reading::Ok(CompleteReading {
                value: Decimal(value.to_owned()),
            }),
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
        let locator = Locator::Sqlite(self.store_path());
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

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// The story ("the offset before the first sample of the series prints `UNKNOWN` ...
/// `baseline predates series`") and design § 3.1 rule 2, as `thalamus_core`'s series rule reads
/// it (offset greater than the series age): the slot a week before `NOW` lies 60 s before the
/// series' first sample, so the week-ago baseline predates the series, even though that first
/// sample is inside the 150 s match tolerance.
#[test]
fn adv_compare_slot_before_first_sample_predates_even_when_a_sample_is_near() {
    let mut world = World::new("slot_before_first");
    world.since_first_sample();
    // The series begins 60 s after the slot `NOW - 7d` (2026-10-07T22:57:03Z).
    world.store("2026-10-07T22:58:03Z", "40");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(doc["reason"], "baseline predates series", "{doc}");
    assert_eq!(doc["result"], "UNKNOWN", "{doc}");
    assert!(doc.get("delta").is_none(), "{doc}");
    assert_eq!(code, Some(3), "{doc}");
}

/// The tolerance is half the cadence: 180 s off the slot is inside a full 5-minute cadence and
/// outside half of it. (Kills the mutant that widens the tolerance to the whole cadence.)
#[test]
fn adv_compare_three_minutes_off_a_5m_slot_is_no_sample_at_offset() {
    let mut world = World::new("three_minutes_off");
    world.store(FIRST, "1");
    world.store("2026-10-07T23:00:03Z", "40");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["reason"], "no stored sample at offset", "{doc}");
    assert!(doc["baseline"].is_null(), "{doc}");
}

/// Exactly half the cadence off the slot is "within half the cadence". (Kills `<=` -> `<`.)
#[test]
fn adv_compare_exactly_half_a_cadence_off_is_within() {
    let mut world = World::new("half_cadence_exact");
    world.store(FIRST, "1");
    world.store("2026-10-07T22:59:33Z", "40");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["baseline"]["ts"], "2026-10-07T22:59:33Z", "{doc}");
    assert_eq!(doc["delta"], "2", "{doc}");
}

/// An offset shorter than half the cadence never matches the current sample with itself.
/// (Kills `ts < now` -> `ts <= now`.)
#[test]
fn adv_compare_short_offset_never_matches_the_current_sample_itself() {
    let mut world = World::new("short_offset_self");
    world.store(FIRST, "1");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "1m", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["reason"], "no stored sample at offset", "{doc}");
    assert!(doc["baseline"].is_null(), "{doc}");
}

/// "At or before `--at`": a sample whose `ts` is `--at` itself is the current sample. (Kills the
/// mutant that makes the range end at `--at` exclusive.)
#[test]
fn adv_compare_sample_exactly_at_at_is_the_current_sample() {
    let mut world = World::new("at_inclusive");
    world.store(FIRST, "1");
    world.store("2026-10-07T23:00:00Z", "40");
    world.store(AT, "50");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["current"]["ts"], AT, "{doc}");
    assert_eq!(doc["ratio"], "1.25", "{doc}");
}

/// The ratio keeps 18 fractional digits, the last rounded half away from zero (the module's
/// own contract): 5e-19 rounds to 1e-18 on both signs. (Kills `>= 5` -> `> 5`.)
#[test]
fn adv_compare_ratio_rounds_an_exact_half_away_from_zero() {
    for (name, current, ratio) in [
        (
            "half_positive",
            "0.0000000000000000005",
            "0.000000000000000001",
        ),
        (
            "half_negative",
            "-0.0000000000000000005",
            "-0.000000000000000001",
        ),
    ] {
        let mut world = World::new(name);
        world.store(FIRST, "1");
        world.store("2026-10-07T22:57:03Z", "1");
        world.store(NOW, current);

        let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
        assert_eq!(code, Some(0), "{name}: {doc}");
        assert_eq!(doc["ratio"], ratio, "{name}: {doc}");
    }
}

/// A baseline of 0: both sides were read, so the story's "otherwise exit 0" holds; the ratio is
/// `null` with its reason and the delta is the current value.
#[test]
fn adv_compare_zero_baseline_exits_0_with_no_ratio() {
    let mut world = World::new("zero_baseline");
    world.store(FIRST, "1");
    world.store("2026-10-07T22:57:03Z", "0");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["result"], "compared", "{doc}");
    assert_eq!(doc["delta"], "42", "{doc}");
    assert!(doc["ratio"].is_null(), "{doc}");
    assert_eq!(doc["ratio_reason"], "the baseline is 0", "{doc}");
}

/// `--at` with a UTC offset, or in lower case, names the same instant as its `Z` spelling.
#[test]
fn adv_compare_at_with_an_offset_names_the_same_instant() {
    let mut world = World::new("at_offset");
    world.store(FIRST, "1");
    world.store("2026-10-07T22:58:00Z", "40");
    world.store(NOW, "42");
    world.store("2026-10-14T23:30:00Z", "77");

    for at in [
        AT,
        "2026-10-15T01:00:00+02:00",
        "2026-10-14T13:30:00-09:30",
        "2026-10-14t23:00:00z",
    ] {
        let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", at]);
        assert_eq!(code, Some(0), "{at}: {doc}");
        assert_eq!(doc["current"]["ts"], NOW, "{at}: {doc}");
        assert_eq!(doc["baseline"]["value"], "40", "{at}: {doc}");
    }
}

/// The offset parser at its limits: the largest offset in seconds is a series-age answer, not a
/// crash; one past it, a zero, a capital unit and an overflowing week count are usage errors.
#[test]
fn adv_compare_offsets_at_the_parsers_limits() {
    let mut world = World::new("offset_limits");
    world.store(FIRST, "1");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "9223372036854775807s", "--at", AT]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["reason"], "baseline predates series", "{doc}");

    for vs in [
        "9223372036854775808s",
        "15250284452472w",
        "0s",
        "0w",
        "7D",
        "P7D",
        "-7d",
        "+7d",
        "7",
        "d",
        "",
    ] {
        let out = world.thalamus(&[DEPTH, "--vs", vs, "--at", AT]);
        assert_eq!(out.status.code(), Some(64), "--vs `{vs}`: {}", text(&out));
    }
}

/// `--at` at the last instant RFC 3339 can name is a valid timestamp: the newest stored sample
/// is at or before it, so the comparison is answered, not refused as a usage error.
#[test]
fn adv_compare_at_the_last_rfc3339_instant_is_answered() {
    let mut world = World::new("at_last_instant");
    world.store(FIRST, "1");
    world.store("2026-10-07T22:57:03Z", "40");
    world.store(NOW, "42");

    let out = world.thalamus(&[
        DEPTH,
        "--vs",
        "7d",
        "--at",
        "9999-12-31T23:59:59.999999999Z",
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
}

/// A comparison only reads: on an instance that has never sampled, it answers `UNKNOWN` (no
/// stored sample) and leaves the instance directory as it found it.
#[test]
fn adv_compare_on_a_fresh_instance_creates_no_store() {
    let world = World::new("fresh_instance");
    assert!(!world.store_path().exists(), "the fixture has no store");

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    assert!(text(&out).contains("UNKNOWN"), "{}", text(&out));
    assert!(
        !world.instance().join(".thalamus").exists(),
        "a read-only comparison created {}",
        world.instance().join(".thalamus").display()
    );
}
