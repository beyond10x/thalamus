//! `thalamus sample`: the built binary against the fixture instance (`tests/fixtures/instance`)
//! and the Connectors double (`thalamus-sources`, `src/bin/connectors-double.rs`).
//!
//! Each test copies the instance into its own directory under the build's temporary directory,
//! so the default store (`sqlite:<instance>/.thalamus/store.sqlite`) and the double's call log
//! land there. The double's recordings are written per test, at the instants the test samples
//! at, because Prometheus answers name the instant they were evaluated at.

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_store::eventlog::{EventlogStore, Locator, SAMPLE_SCHEMA};

/// The first sampling instant, and its Unix seconds.
const T1: &str = "2026-10-14T22:57:03Z";
const T1_UNIX: i64 = 1_792_018_623;
/// One hour later.
const T2: &str = "2026-10-14T23:57:03Z";
const T2_UNIX: i64 = 1_792_022_223;

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

const RATIO: &str = "service.api.request_failure_ratio";
const DEPTH: &str = "service.queue.depth";
const MISSES: &str = "service.cache.miss_ratio";
const MUTED: &str = "service.batch.duration";

const FAILED: &str = "sum by (component) (increase(requests_failed_total[6h]))";
const REQUESTS: &str = "sum by (component) (increase(requests_total[6h]))";
const QUEUE: &str = "sum(queue_depth)";

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

/// One test's directory: the instance copy, the recordings and the call log.
struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_sample")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
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

    fn calls(&self) -> PathBuf {
        self.root.join("calls.jsonl")
    }

    fn write(&self, name: &str, answer: &Value) {
        std::fs::write(
            self.recordings().join(name),
            serde_json::to_string_pretty(answer).expect("JSON"),
        )
        .expect("write a recording");
    }

    /// The recordings of Connectors' `prometheus` adapter instance `metrics`: `metrics-main`
    /// answers every query at [`T1`] and [`T2`]; `metrics-down` refuses every invoke.
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
        let mut answers = Vec::new();
        for at in [T1_UNIX, T2_UNIX] {
            answers.push(vector(
                FAILED,
                at,
                &[("checkout", "15"), ("search", "3")],
                "component",
            ));
            answers.push(vector(
                REQUESTS,
                at,
                &[("checkout", "556"), ("search", "120")],
                "component",
            ));
            answers.push(vector(QUEUE, at, &[("", "42")], ""));
        }
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(answers),
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
            .env("CONNECTORS_DOUBLE_LOG", self.calls())
            .output()
            .expect("the thalamus binary runs")
    }

    /// Every sample record on the instance's stream, as stored, in the order appended.
    fn stored(&self) -> Vec<Value> {
        let locator = Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"));
        let store = EventlogStore::open(&locator, INSTANCE).expect("the store opens");
        store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(SAMPLE_SCHEMA))
            .map(|record| record.body().clone())
            .collect()
    }

    /// The double's call log, one line per call; empty when nothing was called.
    fn called(&self) -> Vec<Value> {
        std::fs::read_to_string(self.calls())
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).expect("a log line is JSON"))
            .collect()
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

/// A `series.query` answer to `query` at `at`: one series per `(label value, value)`, labelled
/// `label` (no labels where `label` is empty).
fn vector(query: &str, at: i64, series: &[(&str, &str)], label: &str) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(labelled, value)| {
            let labels = if label.is_empty() {
                json!({})
            } else {
                json!({ label: labelled })
            };
            json!({"labels": labels, "samples": [[at, value]], "samples_truncated": false})
        })
        .collect();
    let document = json!({
        "result_type": "vector", "series_truncated": false, "complete": true,
        "warnings": [], "infos": [], "time_unix_s": at,
        "provenance": {"instance": "prometheus-test", "resource": "prometheus:query",
                       "observed_at_unix_ms": 0, "source_revision": null},
        "series": series
    });
    json!({
        "input": {"query": query, "time_unix_s": at},
        "stdout": {"ok": true, "result": {
            "adapter": "metrics", "operation": "series.query", "revision": "op-rev-1",
            "result": document.to_string()
        }}
    })
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn series_of(sample: &Value) -> String {
    sample["series"].as_str().expect("series").to_owned()
}

#[test]
fn sample_all_once_records_one_sample_per_series() {
    let world = World::new("all_once");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1, "--format", "json"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));

    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    let run_id = printed["run_id"].as_str().expect("run_id").to_owned();
    assert!(run_id.starts_with("run_"), "{run_id}");

    let stored = world.stored();
    let series: BTreeSet<String> = stored.iter().map(series_of).collect();
    let expected: BTreeSet<String> = [
        format!("{RATIO}{{component=\"checkout\"}}"),
        format!("{RATIO}{{component=\"search\"}}"),
        DEPTH.to_owned(),
        MISSES.to_owned(),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        series, expected,
        "one sample per series of every Active definition"
    );
    assert_eq!(
        stored.len(),
        4,
        "exactly one sample per series: {stored:#?}"
    );
    assert!(
        stored.iter().all(|sample| sample["signal"] != MUTED),
        "a Muted definition is not sampled"
    );

    for sample in &stored {
        let provenance = &sample["provenance"];
        assert_eq!(provenance["run_id"], run_id.as_str(), "one run id per pass");
        assert_eq!(provenance["producer"], "thalamus-cli");
        assert_eq!(provenance["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(provenance["adapter"], "metrics");
        assert_eq!(sample["ts"], T1);
        assert!(
            sample["sample_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("smp_")),
            "{sample}"
        );
    }
    let rev = |signal: &str| {
        stored
            .iter()
            .find(|sample| sample["signal"] == signal)
            .map(|sample| sample["provenance"]["definition_rev"].clone())
    };
    assert_eq!(rev(RATIO), Some(json!(3)));
    assert_eq!(rev(DEPTH), Some(json!(1)));

    let stored_ids: BTreeSet<String> = stored
        .iter()
        .map(|sample| sample["sample_id"].as_str().expect("id").to_owned())
        .collect();
    let printed_ids: BTreeSet<String> = printed["samples"]
        .as_array()
        .expect("samples")
        .iter()
        .map(|sample| sample["sample_id"].as_str().expect("id").to_owned())
        .collect();
    assert_eq!(
        printed_ids, stored_ids,
        "the printed ids are the stored ones"
    );

    let checkout = stored
        .iter()
        .find(|sample| sample["dims"]["component"] == "checkout")
        .expect("the checkout series");
    assert_eq!(checkout["reading"]["quality"], "ok");
    assert_eq!(checkout["denominator"]["n"], 556);
    assert_eq!(checkout["denominator"]["unit"], "{request}");
    assert_eq!(checkout["unit"], "1");

    let depth = stored
        .iter()
        .find(|sample| sample["signal"] == DEPTH)
        .expect("the depth series");
    assert_eq!(
        depth["reading"],
        json!({"quality": "ok", "value": {"value": "42"}})
    );
    assert_eq!(depth["dims"], json!({}));
}

#[test]
fn unreadable_source_records_unknown_without_value() {
    let world = World::new("unreadable");
    let out = world.thalamus(&["sample", MISSES, "--at", T1]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "an unknown read is recorded, so the pass exits 0; stderr: {}",
        text(&out.stderr)
    );

    let stored = world.stored();
    assert_eq!(stored.len(), 1, "{stored:#?}");
    let sample = &stored[0];
    assert_eq!(sample["signal"], MISSES);
    assert_eq!(sample["series"], MISSES);
    let reading = &sample["reading"];
    assert_eq!(reading["quality"], "unknown", "{reading}");
    // The reading's union arm carries only its reason: no value of any kind (D11 wire form
    // `{"quality": "unknown", "value": {"reason": …}}`).
    let arm = reading["value"].as_object().expect("the unknown arm");
    assert_eq!(
        arm.keys().map(String::as_str).collect::<Vec<_>>(),
        ["reason"],
        "{reading}"
    );
    assert!(arm.get("value").is_none(), "{reading}");
    let reason = arm["reason"].as_str().expect("reason");
    assert!(reason.contains("unavailable"), "{reason}");
    assert!(sample.get("denominator").is_none(), "{sample}");
    assert!(
        text(&out.stdout).contains(sample["sample_id"].as_str().expect("id")),
        "the recorded id is printed: {}",
        text(&out.stdout)
    );
}

#[test]
fn muted_definition_is_refused_and_nothing_is_recorded() {
    let world = World::new("muted");
    let out = world.thalamus(&["sample", MUTED, "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(stderr.contains(MUTED), "{stderr}");
    assert!(stderr.contains("not-active"), "{stderr}");
    assert!(world.stored().is_empty(), "nothing is recorded");
    assert!(world.called().is_empty(), "the source is not read");

    let out = world.thalamus(&["sample", "service.no.such_signal", "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    assert!(text(&out.stderr).contains("unknown-signal"));
    assert!(world.stored().is_empty(), "nothing is recorded");
}

#[test]
fn invalid_instance_exits_65_naming_file_and_rule() {
    let world = World::new("invalid");
    let broken = world.instance().join("definitions/broken.yaml");
    let ratio = std::fs::read_to_string(
        world
            .instance()
            .join("definitions/service.api.request_failure_ratio.yaml"),
    )
    .expect("the ratio definition");
    std::fs::write(
        &broken,
        ratio.replace("id: service.api.request_failure_ratio", "id: Broken-Id"),
    )
    .expect("write a broken definition");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(stderr.contains("definitions/broken.yaml"), "{stderr}");
    assert!(stderr.contains("signal id grammar"), "{stderr}");

    // An unknown grading rule is refused at load, before any read.
    std::fs::write(
        &broken,
        ratio
            .replace(
                "id: service.api.request_failure_ratio",
                "id: service.api.copy",
            )
            .replace("rule: ratio_vs_pooled_baseline", "rule: no_such_rule"),
    )
    .expect("write a definition naming no rule");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(stderr.contains("definitions/broken.yaml"), "{stderr}");
    assert!(stderr.contains("grading rule"), "{stderr}");
    assert!(stderr.contains("no_such_rule"), "{stderr}");

    // A registry file a `file:` locator names that does not exist.
    std::fs::remove_file(&broken).expect("remove the broken definition");
    std::fs::remove_file(world.instance().join("registries/components.yaml"))
        .expect("remove the registry file");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains("registries/components.yaml"),
        "{}",
        text(&out.stderr)
    );

    assert!(world.called().is_empty(), "no source is read");
}

#[test]
fn since_first_sample_is_the_first_stored_sample() {
    let world = World::new("since");
    for at in [T1, T2] {
        let out = world.thalamus(&["sample", RATIO, "--at", at]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    let stored = world.stored();
    assert_eq!(stored.len(), 4, "two series, two passes: {stored:#?}");
    for component in ["checkout", "search"] {
        let samples: Vec<&Value> = stored
            .iter()
            .filter(|sample| sample["dims"]["component"] == component)
            .collect();
        assert_eq!(samples.len(), 2, "{component}");
        let (first, second) = (samples[0], samples[1]);
        assert_eq!(first["ts"], T1);
        assert_eq!(second["ts"], T2);
        assert_eq!(
            first["denominator"]["since"], T1,
            "the first sample of a series is its own origin"
        );
        assert_eq!(first["denominator"]["age_s"], 0);
        assert_eq!(
            second["denominator"]["since"], T1,
            "since is the first stored sample of the series"
        );
        assert_eq!(second["denominator"]["age_s"], T2_UNIX - T1_UNIX);
        assert_ne!(
            first["provenance"]["run_id"], second["provenance"]["run_id"],
            "each pass has its own run id"
        );
    }
}

#[test]
fn a_one_shot_pass_is_all_and_once_together() {
    let world = World::new("usage");
    for args in [
        &["sample", "--all", "--at", T1][..],
        &["sample", "--once", RATIO, "--at", T1][..],
        &["sample", RATIO, "--all", "--once", "--at", T1][..],
        &["sample", "--at", T1][..],
        &["sample", RATIO, "--at", "yesterday"][..],
    ] {
        let out = world.thalamus(args);
        assert_eq!(out.status.code(), Some(64), "thalamus {args:?}");
    }
    assert!(world.called().is_empty(), "no source is read");
}

/// A store refusal never prints the locator text: not a user, not a password, not a query, not a
/// path. Every refusal path of a locator carries the marker, and none may print it.
#[test]
fn no_store_refusal_prints_the_locator_text() {
    const MARK: &str = "mark3r";
    let world = World::new("locator_text");
    let dir = world.root.join(MARK);
    std::fs::create_dir_all(&dir).expect("create a directory");
    // Refused at open, not at parse: a database file the store cannot open, and a PostgreSQL
    // server that does not answer.
    let unreadable = world.root.join(format!("{MARK}.sqlite"));
    std::fs::write(&unreadable, b"").expect("create a file");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000))
        .expect("make it unreadable");
    for (locator, code) in [
        (
            format!("sqlite:{}", unreadable.display()),
            "the store is unavailable",
        ),
        (
            format!("postgresql://{MARK}@127.0.0.1:1/thalamus?application_name={MARK}"),
            "the store is unavailable",
        ),
        (
            format!("postgresql://{MARK}:{MARK}@127.0.0.1:1/thalamus"),
            "thalamus.store.postgres-password",
        ),
        (
            format!("postgresql://127.0.0.1:1/thalamus?sslmode=require&application_name={MARK}"),
            "thalamus.store.postgres-tls",
        ),
        (
            format!("mysql://{MARK}@127.0.0.1/thalamus"),
            "thalamus.store.locator-unknown",
        ),
        (
            format!("sqlite:{}", dir.display()),
            "thalamus.store.sqlite-not-a-file",
        ),
    ] {
        let out = world.thalamus(&["--store", &locator, "sample", RATIO, "--at", T1]);
        assert_eq!(out.status.code(), Some(74), "{locator}");
        let stderr = text(&out.stderr);
        assert!(stderr.contains(code), "{locator}: {stderr}");
        assert!(!stderr.contains(MARK), "{locator}: {stderr}");
    }
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o600))
        .expect("restore the mode");
    assert!(world.called().is_empty(), "no source is read");
}

/// No help text prints the value of an environment variable an option reads: every `[env: NAME]`
/// help names, set to a marker, stays out of the help.
#[test]
fn no_help_prints_an_environment_value() {
    const MARK: &str = "mark3r";
    let help = |args: &[&str], env: &[String]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_thalamus"));
        command.args(args);
        for name in env {
            command.env(name, MARK);
        }
        let out = command.output().expect("the thalamus binary runs");
        assert_eq!(out.status.code(), Some(0), "{args:?}");
        text(&out.stdout)
    };
    for args in [&["--help"][..], &["sample", "--help"][..]] {
        let names: Vec<String> = help(args, &[])
            .split("[env: ")
            .skip(1)
            .filter_map(|rest| rest.split(['=', ']']).next())
            .map(str::to_owned)
            .collect();
        assert!(
            names.iter().any(|name| name == "THALAMUS_STORE"),
            "{args:?}: {names:?}"
        );
        let shown = help(args, &names);
        assert!(!shown.contains(MARK), "{args:?}: {shown}");
    }
}

#[test]
fn an_unusable_store_exits_74() {
    let world = World::new("store");
    let out = world.thalamus(&[
        "--store",
        &format!("sqlite:{}", world.root.display()),
        "sample",
        RATIO,
        "--at",
        T1,
    ]);
    assert_eq!(out.status.code(), Some(74), "stderr: {}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains("thalamus.store.sqlite-not-a-file"),
        "{}",
        text(&out.stderr)
    );
}
