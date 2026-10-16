//! `thalamus sample` reports a breach: a sample graded against a `fixed` baseline alone that
//! grades WARN or CRIT prints one `breach` line after the sample ids, and the same under
//! `breaches` in `--format json`. No evaluation is recorded and the exit code stays 0.
//!
//! The instance is `tests/fixtures/breach`: three free-space definitions in bytes, each graded
//! `lower_is_worse` against a synthetic floor `fixed: "200000000000"`, with `crit: {x: "1"}` (at
//! or under the floor) and `warn: {x: "1.25"}`. They read the Connectors double
//! (`thalamus-sources`, `src/bin/connectors-double.rs`): `platform.disk.low` answers one mount
//! under the floor and one between the floor and the WARN line, `platform.disk.ample` one mount
//! well over it, and `platform.disk.unread` names a connection whose every invoke fails.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_store::eventlog::{EventlogStore, Locator, SAMPLE_SCHEMA};

/// The sampling instant, and its Unix seconds.
const T1: &str = "2026-10-14T22:57:03Z";
const T1_UNIX: i64 = 1_792_018_623;

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

const LOW: &str = "platform.disk.low";
const AMPLE: &str = "platform.disk.ample";
const UNREAD: &str = "platform.disk.unread";

const LOW_QUERY: &str = "sum by (mount) (low_free_bytes)";
const AMPLE_QUERY: &str = "sum by (mount) (ample_free_bytes)";

/// The Connectors double, built once for the whole test binary beside the `thalamus` binary.
fn double() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let status = Command::new(env!("CARGO"))
                .args([
                    "build",
                    "--locked",
                    "--quiet",
                    "-p",
                    "thalamus-sources",
                    "--bin",
                    "connectors-double",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .status()
                .expect("cargo runs");
            assert!(status.success(), "the Connectors double builds");
            let bin = Path::new(env!("CARGO_BIN_EXE_thalamus")).with_file_name("connectors-double");
            assert!(bin.exists(), "{} exists", bin.display());
            bin
        })
        .clone()
}

/// One test's directory: the instance copy and the recordings.
struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_sample_breach")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/breach"),
            &root.join("instance"),
        );
        let world = Self { root };
        std::fs::create_dir_all(world.recordings()).expect("create the recordings directory");
        world.record_metrics();
        world
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn recordings(&self) -> PathBuf {
        self.root.join("connectors")
    }

    fn write(&self, name: &str, answer: &Value) {
        std::fs::write(
            self.recordings().join(name),
            serde_json::to_string_pretty(answer).expect("JSON"),
        )
        .expect("write a recording");
    }

    /// `metrics-main` answers the free bytes per mount at [`T1`]; `metrics-down` refuses every
    /// invoke.
    fn record_metrics(&self) {
        self.write(
            "connections+list+a=metrics.json",
            &json!({"stdout": {"ok": true, "result": {"connections": [
                {"adapter": "metrics", "connection": "metrics-main", "state": "ready", "revision": "conn-rev-1"},
                {"adapter": "metrics", "connection": "metrics-down", "state": "ready", "revision": "conn-rev-1"}
            ]}}}),
        );
        self.write(
            "operations+describe+a=metrics+o=series.query.json",
            &json!({"stdout": {"ok": true, "result": {
                "adapter": "metrics", "operation": "series.query",
                "schema": "series.query/v1alpha1", "revision": "op-rev-1"
            }}}),
        );
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(vec![
                // `/` is under the floor (CRIT); `/data` is over it, under the WARN line (WARN).
                vector(
                    LOW_QUERY,
                    &[("/", "150000000000"), ("/data", "240000000000")],
                ),
                vector(AMPLE_QUERY, &[("/", "900000000000")]),
            ]),
        );
        self.write(
            "operations+invoke+a=metrics+c=metrics-down+o=series.query.json",
            &json!({"stderr": {"ok": false, "error": {
                "code": "failure",
                "message": "the provider did not answer",
                "data": {"kind": "operational", "code": "unavailable", "stage": "dispatch"}
            }}}),
        );
    }

    /// Runs `thalamus --instance <copy> --connectors <double> <args>`.
    fn thalamus(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(double())
            .args(args)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("CONNECTORS_DOUBLE_FIXTURES", self.recordings())
            .env("CONNECTORS_DOUBLE_LOG", self.root.join("calls.jsonl"))
            .output()
            .expect("the thalamus binary runs")
    }

    /// The schema of every record on the instance's stream, in the order appended.
    fn schemas(&self) -> Vec<Option<String>> {
        let locator = Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"));
        let store = EventlogStore::open(&locator, INSTANCE).expect("the store opens");
        store
            .records()
            .iter()
            .map(|record| record.schema().map(str::to_owned))
            .collect()
    }

    /// Asserts the stream holds exactly `samples` sample records and nothing else: no evaluation.
    fn assert_only_samples(&self, samples: usize) {
        let schemas = self.schemas();
        assert!(
            schemas
                .iter()
                .all(|schema| schema.as_deref() == Some(SAMPLE_SCHEMA)),
            "sample records only, no evaluation: {schemas:?}"
        );
        assert_eq!(schemas.len(), samples, "{schemas:?}");
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory");
    for entry in std::fs::read_dir(from).expect("the fixture exists") {
        let path = entry.expect("a fixture entry").path();
        let target = to.join(path.file_name().expect("a file name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy a fixture file");
        }
    }
}

/// A `series.query` answer to `query` at [`T1`]: one series per `(mount, value)`.
fn vector(query: &str, series: &[(&str, &str)]) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(mount, value)| {
            json!({"labels": {"mount": mount}, "samples": [[T1_UNIX, value]], "samples_truncated": false})
        })
        .collect();
    let document = json!({
        "result_type": "vector", "series_truncated": false, "complete": true,
        "warnings": [], "infos": [], "time_unix_s": T1_UNIX,
        "provenance": {"instance": "prometheus-test", "resource": "prometheus:query",
                       "observed_at_unix_ms": 0, "source_revision": null},
        "series": series
    });
    json!({
        "input": {"query": query, "time_unix_s": T1_UNIX},
        "stdout": {"ok": true, "result": {
            "adapter": "metrics", "operation": "series.query", "revision": "op-rev-1",
            "result": document.to_string()
        }}
    })
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The `breach` lines of a text run.
fn breach_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.starts_with("breach "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_reading_under_the_fixed_floor_prints_a_crit_breach() {
    let world = World::new("under_floor");
    let out = world.thalamus(&["sample", LOW, "--at", T1]);
    let stdout = text(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a breach is not a failure of the run; stdout {stdout} stderr {}",
        text(&out.stderr)
    );

    let lines: Vec<&str> = stdout.lines().collect();
    let first_breach = lines
        .iter()
        .position(|line| line.starts_with("breach "))
        .expect("a breach line");
    assert_eq!(
        first_breach, 2,
        "the breach lines follow the two sample ids: {stdout}"
    );
    assert!(
        lines[..first_breach]
            .iter()
            .all(|line| line.starts_with("smp_")),
        "{stdout}"
    );

    assert_eq!(
        breach_lines(&stdout),
        [
            "breach CRIT platform.disk.low mount=/ 150000000000 By 150000000000 <= 1 x \
             200000000000",
            "breach WARN platform.disk.low mount=/data 240000000000 By 240000000000 <= 1.25 x \
             200000000000",
        ],
        "{stdout}"
    );
    world.assert_only_samples(2);
}

#[test]
fn a_reading_over_the_floor_prints_no_breach() {
    let world = World::new("over_floor");
    let out = world.thalamus(&["sample", AMPLE, "--at", T1]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert_eq!(
        stdout.lines().count(),
        1,
        "one sample id and nothing else: {stdout}"
    );
    assert!(
        stdout.lines().all(|line| line.starts_with("smp_")),
        "{stdout}"
    );
    assert!(stdout.ends_with("\tok\n"), "{stdout}");
    assert_eq!(breach_lines(&stdout), Vec::<String>::new());
    world.assert_only_samples(1);
}

#[test]
fn an_unknown_reading_prints_no_breach_and_is_never_ok() {
    let world = World::new("unknown");
    let out = world.thalamus(&["sample", UNREAD, "--at", T1, "--format", "json"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    let samples = printed["samples"].as_array().expect("samples");
    assert_eq!(samples.len(), 1, "{printed}");
    assert_eq!(
        samples[0]["quality"], "unknown",
        "unknown is never 0 and never OK"
    );
    assert_eq!(
        printed["breaches"],
        json!([]),
        "an unknown reading is no breach: {printed}"
    );
    assert!(
        !text(&out.stdout).contains("\"OK\""),
        "an unknown reading is never reported OK: {printed}"
    );

    let out = world.thalamus(&["sample", UNREAD, "--at", T1]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert_eq!(breach_lines(&stdout), Vec::<String>::new(), "{stdout}");
    assert!(stdout.ends_with("\tunknown\n"), "{stdout}");
    world.assert_only_samples(2);
}

#[test]
fn breaches_appear_in_json_output() {
    let world = World::new("json");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1, "--format", "json"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    assert_eq!(
        printed["samples"].as_array().expect("samples").len(),
        4,
        "low twice, ample and unread once: {printed}"
    );
    let low_root = printed["samples"]
        .as_array()
        .expect("samples")
        .iter()
        .find(|sample| sample["series"] == "platform.disk.low{mount=\"/\"}")
        .expect("the low / series")["sample_id"]
        .clone();
    assert_eq!(
        printed["breaches"],
        json!([
            {
                "verdict": "CRIT",
                "sample_id": low_root,
                "signal": LOW,
                "series": "platform.disk.low{mount=\"/\"}",
                "dims": {"mount": "/"},
                "value": "150000000000",
                "unit": "By",
                "line": "150000000000 <= 1 x 200000000000",
            },
            {
                "verdict": "WARN",
                "sample_id": printed["samples"]
                    .as_array()
                    .expect("samples")
                    .iter()
                    .find(|sample| sample["series"] == "platform.disk.low{mount=\"/data\"}")
                    .expect("the low /data series")["sample_id"],
                "signal": LOW,
                "series": "platform.disk.low{mount=\"/data\"}",
                "dims": {"mount": "/data"},
                "value": "240000000000",
                "unit": "By",
                "line": "240000000000 <= 1.25 x 200000000000",
            },
        ]),
        "{printed}"
    );
    world.assert_only_samples(4);
}
