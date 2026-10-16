//! Adversary pass 2 on `thalamus compare`, against the correction of pass 1: the range end at the
//! last RFC 3339 instant, the series-age check, the missing-store path, and lines the suite
//! leaves free. Each test copies the fixture instance into its own directory under the build's
//! temporary directory, appends the samples it needs to the instance's default store and
//! compares. No source is read.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{
    CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

const INSTANCE: &str = "fixture";
/// `history: store`, no dimensions, cadence `PT5M`: a match tolerance of 150 seconds.
const DEPTH: &str = "service.queue.depth";
const AT: &str = "2026-10-14T23:00:00Z";
const NOW: &str = "2026-10-14T22:57:03Z";
/// `NOW` minus 7 days, to the second.
const WEEK_AGO: &str = "2026-10-07T22:57:03Z";
/// Long before every slot.
const FIRST: &str = "2026-10-01T00:00:00Z";
/// The last instant RFC 3339 can name.
const LAST: &str = "9999-12-31T23:59:59.999999999Z";

/// A ratio definition of the fixture's shape (cadence `1h`, dimension `component`) whose
/// `denominator.since` is `{since}`: the origin design § 2.2 reads a series' age from.
const RATIO_ID: &str = "service.api.error_ratio";
const RATIO_DEFINITION: &str = "id: service.api.error_ratio
title: Share of requests that errored
family: service
kind: ratio
unit: \"1\"
dimensions: [component]
source: { adapter: metrics, connection: metrics-main }
extractor: promql.ratio
query:
  numerator: sum by (component) (increase(requests_errored_total[1h]))
  denominator: sum by (component) (increase(requests_total[1h]))
window: 1h
cadence: 1h
history: source
denominator: { unit: \"{request}\", min_n: 20, since: \"{since}\" }
evaluation:
  rule: ratio_vs_pooled_baseline
  baselines: [pooled, offset_1d, offset_7d]
  warn: { x: 2, abs: 0.02, min_events: 10 }
  crit: { x: 4, abs: 0.05 }
owner: team:service
links: { components: [api], issues: [] }
lifecycle: active
rev: 1
revised_at: 2026-10-01T00:00:00Z
revised_by: operator
";

struct World {
    root: PathBuf,
    next: u64,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_compare_p2")
            .join(name);
        if root.exists() {
            make_writable(&root);
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

    fn store_path(&self) -> PathBuf {
        self.instance().join(".thalamus/store.sqlite")
    }

    fn define_ratio(&self, since: &str) {
        std::fs::write(
            self.instance()
                .join("definitions")
                .join(format!("{RATIO_ID}.yaml")),
            RATIO_DEFINITION.replace("{since}", since),
        )
        .expect("write the ratio definition");
    }

    fn append(&mut self, sample: SignalSampleData) {
        let locator = Locator::Sqlite(self.store_path());
        let mut store = EventlogStore::open(&locator, INSTANCE).expect("the store opens");
        store.append_sample(&sample).expect("the sample is stored");
    }

    fn store(&mut self, ts: &str, value: &str) {
        let sample = self.sample(DEPTH, DEPTH, BTreeMap::new(), ts, value, None);
        self.append(sample);
    }

    /// A sample of the ratio definition for `component=checkout`, carrying the population and its
    /// origin as `thalamus sample` records them.
    fn store_ratio(&mut self, ts: &str, value: &str, since: &str, age_s: i64) {
        let dims = BTreeMap::from([(
            "component".to_owned(),
            DimensionValue("checkout".to_owned()),
        )]);
        let series = format!("{RATIO_ID}{{component=\"checkout\"}}");
        let denominator = Denominator {
            n: 1000,
            unit: Unit("{request}".to_owned()),
            since: Timestamp(since.to_owned()),
            age_s,
        };
        let sample = self.sample(RATIO_ID, &series, dims, ts, value, Some(denominator));
        self.append(sample);
    }

    fn sample(
        &mut self,
        signal: &str,
        series: &str,
        dims: BTreeMap<String, DimensionValue>,
        ts: &str,
        value: &str,
        denominator: Option<Denominator>,
    ) -> SignalSampleData {
        let sample = SignalSampleData {
            sample_id: SampleId(format!("smp_{:012}", self.next)),
            signal: SignalId(signal.to_owned()),
            series: SeriesKey(series.to_owned()),
            dims,
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
            denominator,
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
        sample
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

/// Gives the owner write permission on `path` and everything below it, so a test directory a
/// read-only case left behind can be removed.
fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let mut permissions = meta.permissions();
    permissions.set_mode(permissions.mode() | 0o700);
    let _ = std::fs::set_permissions(path, permissions);
    if meta.is_dir() {
        for entry in std::fs::read_dir(path).into_iter().flatten().flatten() {
            make_writable(&entry.path());
        }
    }
}

fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("set the mode");
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// The correction of pass 1 ends the range read at the last RFC 3339 instant when `--at` has no
/// successor, but the range end is exclusive: a sample stored at that instant is "at or before
/// `--at`" (the story) and must be the current sample.
#[test]
fn adv2_compare_a_sample_at_the_last_instant_is_current_at_the_last_instant() {
    let mut world = World::new("sample_at_last_instant");
    world.store(FIRST, "1");
    world.store(NOW, "42");
    world.store(LAST, "50");

    let (_, doc) = world.json(&[DEPTH, "--vs", "1s", "--at", LAST]);
    assert_eq!(doc["current"]["ts"], LAST, "{doc}");
    assert_eq!(doc["current"]["value"], "50", "{doc}");
}

/// `--at` whose UTC instant lies before year 0 (`0000-01-01T00:30:00+01:00`): the range end then
/// has no RFC 3339 spelling and falls back to the last instant, so only the `ts <= --at` filter
/// keeps later samples out. Nothing is at or before it: exit 3, no current sample. (Kills the
/// mutant that drops that filter.)
#[test]
fn adv2_compare_at_before_year_zero_finds_no_current_sample() {
    let mut world = World::new("at_before_year_zero");
    world.store(FIRST, "1");
    world.store(WEEK_AGO, "40");
    world.store(NOW, "42");

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", "0000-01-01T00:30:00+01:00"]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", "0000-01-01T00:30:00+01:00"]);
    assert_eq!(code, Some(3), "{doc}");
    assert!(doc["current"].is_null(), "{doc}");
    assert_eq!(doc["result"], "UNKNOWN", "{doc}");
}

/// A slot that is exactly the first sample's `ts` does not predate the series: the offset equals
/// the series age, and only an offset greater than the age predates it (`thalamus_core`'s series
/// rule, design § 2.2). (Kills `target < first` -> `target <= first`.)
#[test]
fn adv2_compare_slot_exactly_at_the_first_sample_is_compared() {
    let mut world = World::new("slot_at_first");
    world.store(WEEK_AGO, "40");
    world.store(NOW, "42");

    let (code, doc) = world.json(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(code, Some(0), "{doc}");
    assert_eq!(doc["baseline"]["ts"], WEEK_AGO, "{doc}");
    assert_eq!(doc["delta"], "2", "{doc}");
}

/// A baseline of 0 in text: the ratio line says it is undefined and prints no number for it.
/// (The JSON form is pinned by pass 1; the text line is pinned by nothing.)
#[test]
fn adv2_compare_zero_baseline_text_prints_no_ratio_number() {
    let mut world = World::new("zero_baseline_text");
    world.store(FIRST, "1");
    world.store(WEEK_AGO, "0");
    world.store(NOW, "42");

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    let ratio = printed
        .lines()
        .find(|line| line.starts_with("ratio"))
        .unwrap_or_else(|| panic!("a ratio line in {printed}"));
    assert_eq!(ratio, "ratio    undefined: the baseline is 0", "{printed}");
    let delta = printed
        .lines()
        .find(|line| line.starts_with("delta"))
        .unwrap_or_else(|| panic!("a delta line in {printed}"));
    assert_eq!(delta, "delta    42", "{printed}");
}

/// Design § 2.2 and the baselines page ("How old is a series"): a series' age is `age_s`, the
/// time since `denominator.since`, and only an offset greater than it predates the series. Here
/// `since` is the `checkout` value's `created` (2025-03-02), so a week-ago slot is well inside the
/// series; thalamus simply stored nothing there. The answer is `UNKNOWN` with reason `no stored
/// sample at offset`, not `baseline predates series`.
#[test]
fn adv2_compare_age_comes_from_since_not_the_first_stored_sample() {
    let mut world = World::new("since_older_than_store");
    world.define_ratio("component.created");
    let since = "2025-03-02T00:00:00Z";
    // Stored from 2026-10-12 on; the series is 591 days old at NOW (age_s as sampled).
    world.store_ratio("2026-10-12T22:57:03Z", "0.01", since, 50_972_223);
    world.store_ratio(NOW, "0.02", since, 51_145_023);

    let (code, doc) = world.json(&[
        RATIO_ID,
        "--dim",
        "component=checkout",
        "--vs",
        "7d",
        "--at",
        AT,
    ]);
    assert_eq!(code, Some(3), "{doc}");
    assert_eq!(doc["reason"], "no stored sample at offset", "{doc}");
}

/// The other direction: `denominator.since` is a fixed instant (2026-10-10) after the first
/// stored sample, so at `NOW` the series is under 5 days old and a 7-day baseline predates it
/// (design § 3.1 rule 2, L6). Compare answers with the stored sample from before `since` and
/// exits 0.
#[test]
fn adv2_compare_offset_past_since_predates_even_when_a_sample_is_stored() {
    let mut world = World::new("since_newer_than_store");
    let since = "2026-10-10T00:00:00Z";
    world.define_ratio(since);
    world.store_ratio(FIRST, "0.5", since, 0);
    world.store_ratio(WEEK_AGO, "0.4", since, 0);
    // 4 days, 22:57:03 after `since`.
    world.store_ratio(NOW, "0.02", since, 428_223);

    let (code, doc) = world.json(&[
        RATIO_ID,
        "--dim",
        "component=checkout",
        "--vs",
        "7d",
        "--at",
        AT,
    ]);
    assert_eq!(doc["reason"], "baseline predates series", "{doc}");
    assert_eq!(code, Some(3), "{doc}");
}

/// A read on an instance whose store directory is read-only (a read-only mount of a sampled
/// instance): a store the user cannot write is "store unavailable", exit 74, as for every command.
#[test]
fn adv2_compare_on_a_read_only_store_is_unavailable() {
    let mut world = World::new("read_only_store");
    world.store(FIRST, "1");
    world.store(WEEK_AGO, "40");
    world.store(NOW, "42");
    let dir = world.instance().join(".thalamus");
    for entry in std::fs::read_dir(&dir).expect("list the store directory") {
        set_mode(&entry.expect("a store file").path(), 0o444);
    }
    set_mode(&dir, 0o555);

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", AT]);
    make_writable(&dir);
    assert_eq!(out.status.code(), Some(74), "{}", text(&out));
}

/// `--store` naming a SQLite file in a directory that does not exist: compare answers `UNKNOWN`
/// (nothing stored) and creates neither the file nor its directory.
#[test]
fn adv2_compare_with_a_store_locator_that_does_not_exist_creates_nothing() {
    let world = World::new("store_locator_missing");
    let missing = world.root.join("elsewhere/store.sqlite");
    let locator = format!("sqlite:{}", missing.display());

    let out = world.thalamus(&["--store", &locator, DEPTH, "--vs", "7d", "--at", AT]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
    assert!(text(&out).contains("UNKNOWN"), "{}", text(&out));
    assert!(!world.root.join("elsewhere").exists(), "{}", text(&out));
}

/// `--at` at the last instant of year 9999 in a UTC offset west of Greenwich is a valid RFC 3339
/// timestamp the crate's instant reader accepts: it is answered, not refused as a usage error.
#[test]
fn adv2_compare_at_past_year_9999_in_utc_is_answered() {
    let mut world = World::new("at_past_9999_utc");
    world.store(FIRST, "1");
    world.store(NOW, "42");

    let out = world.thalamus(&[DEPTH, "--vs", "7d", "--at", "9999-12-31T23:30:00-01:00"]);
    assert_ne!(out.status.code(), Some(64), "{}", text(&out));
    assert_eq!(out.status.code(), Some(3), "{}", text(&out));
}
