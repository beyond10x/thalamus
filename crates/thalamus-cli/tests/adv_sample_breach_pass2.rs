//! Adversarial pass 2 on the breach report of `thalamus sample` (Amendment 1 of
//! story:m3-host-collector). The instance is `tests/fixtures/adv_breach_pass2`; the harness is
//! the one of `tests/sample_breach.rs`.

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

const ATFLOOR: &str = "platform.disk.atfloor";
const MIXED: &str = "platform.disk.mixed";
const HOST_FREE: &str = "platform.host.free";

const ATFLOOR_QUERY: &str = "sum by (mount) (atfloor_free_bytes)";
const MIXED_QUERY: &str = "sum by (mount) (mixed_free_bytes)";

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
            .join("adv_sample_breach_pass2")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adv_breach_pass2"),
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
                vector(ATFLOOR_QUERY, &[("/", "200000000000")]),
                vector(MIXED_QUERY, &[("/", "1")]),
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

/// A reading exactly at the floor is at the line and grades CRIT
/// (`200000000000 <= 1 x 200000000000`).
#[test]
fn a_reading_exactly_at_the_floor_is_a_crit_breach() {
    let world = World::new("at_floor");
    let out = world.thalamus(&["sample", ATFLOOR, "--at", T1]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let crit: Vec<String> = breach_lines(&stdout)
        .into_iter()
        .filter(|line| line.starts_with("breach CRIT "))
        .collect();
    assert_eq!(
        crit.len(),
        1,
        "a reading at the floor is a CRIT breach: {stdout}"
    );
    world.assert_only_samples(1);
}

/// Green guard: a definition graded against a baseline other than `fixed` needs history and is
/// not graded at sample time, however low it reads. (`fixed` beside another baseline is refused by
/// the loader, exit 65, so that case cannot be reached.)
#[test]
fn a_definition_not_graded_against_fixed_prints_no_breach() {
    let world = World::new("mixed");
    let out = world.thalamus(&["sample", MIXED, "--at", T1, "--format", "json"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let printed: Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(
        printed["samples"].as_array().expect("samples").len(),
        1,
        "{printed}"
    );
    assert_eq!(printed["breaches"], json!([]), "{printed}");
    world.assert_only_samples(1);
}

/// The operator's own case, end to end: a `host.disk_free` definition of `/` under a floor no
/// disk reaches, sampled now, prints one CRIT breach for `mount=/` in bytes. No test of the unit
/// drives a host definition through `thalamus sample`.
#[test]
fn host_disk_free_under_the_floor_prints_a_crit_breach_end_to_end() {
    let world = World::new("host_free");
    let out = world.thalamus(&["sample", HOST_FREE]);
    let stdout = text(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout {stdout} stderr {}",
        text(&out.stderr)
    );
    assert!(
        stdout.lines().next().is_some_and(|l| l.ends_with("\tok")),
        "an ok sample: {stdout}"
    );
    let lines = breach_lines(&stdout);
    assert_eq!(lines.len(), 1, "{stdout}");
    assert!(
        lines[0].starts_with("breach CRIT platform.host.free mount=/ ")
            && lines[0].contains(" By ")
            && lines[0].ends_with("<= 1 x 1000000000000000000"),
        "{stdout}"
    );
    world.assert_only_samples(1);
}
