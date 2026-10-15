//! Adversary pass 1 against `thalamus sample` (story:m1-cli-sample): the built binary against a
//! copy of the fixture instance and the Connectors double, with recordings this file writes.
//!
//! Cases named `*_red_*` in the report are the findings; the others pin behaviour the unit's own
//! suite leaves free (each names the mutant it catches).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_store::eventlog::{EventlogStore, Locator, SAMPLE_SCHEMA};

const T1: &str = "2026-10-14T22:57:03Z";
const T1_UNIX: i64 = 1_792_018_623;
const T2: &str = "2026-10-14T23:57:03Z";
const T2_UNIX: i64 = 1_792_022_223;
/// `created` of `checkout` in the fixture's `registries/components.yaml`.
const CHECKOUT_CREATED: &str = "2025-03-02T00:00:00Z";
const CHECKOUT_CREATED_UNIX: i64 = 1_740_873_600;

const INSTANCE: &str = "fixture";

const RATIO: &str = "service.api.request_failure_ratio";
const DEPTH: &str = "service.queue.depth";
const MUTED: &str = "service.batch.duration";

const FAILED: &str = "sum by (component) (increase(requests_failed_total[6h]))";
const REQUESTS: &str = "sum by (component) (increase(requests_total[6h]))";
const QUEUE: &str = "sum(queue_depth)";

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

struct World {
    root: PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_sample_p1")
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
        world.record_catalogue();
        world.record_invokes(&standard_answers());
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

    fn record_catalogue(&self) {
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
            "operations+invoke+a=metrics+c=metrics-down+o=series.query.json",
            &json!({"stderr": {"ok": false, "error": {
                "code": "failure",
                "message": "the provider did not answer",
                "data": {"kind": "operational", "code": "unavailable", "stage": "dispatch"}
            }}}),
        );
    }

    fn record_invokes(&self, answers: &[Value]) {
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(answers.to_vec()),
        );
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_thalamus"));
        command
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("CONNECTORS_DOUBLE_FIXTURES", self.recordings())
            .env("CONNECTORS_DOUBLE_LOG", self.calls());
        command
    }

    fn thalamus(&self, args: &[&str]) -> Output {
        self.command()
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(double())
            .args(args)
            .output()
            .expect("the thalamus binary runs")
    }

    fn stored_at(&self, locator: &Locator, instance: &str) -> Vec<Value> {
        let store = EventlogStore::open(locator, instance).expect("the store opens");
        store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(SAMPLE_SCHEMA))
            .map(|record| record.body().clone())
            .collect()
    }

    fn stored(&self) -> Vec<Value> {
        self.stored_at(
            &Locator::Sqlite(self.instance().join(".thalamus/store.sqlite")),
            INSTANCE,
        )
    }

    fn called(&self) -> Vec<Value> {
        std::fs::read_to_string(self.calls())
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).expect("a log line is JSON"))
            .collect()
    }

    /// Writes `definitions/<id>.yaml`: the depth definition under another id and lifecycle.
    fn add_depth_copy(&self, id: &str, lifecycle: &str) {
        let depth = self.definition(DEPTH);
        let text = depth
            .replace("id: service.queue.depth", &format!("id: {id}"))
            .replace("lifecycle: active", &format!("lifecycle: {lifecycle}"));
        std::fs::write(self.instance().join(format!("definitions/{id}.yaml")), text)
            .expect("write a definition");
    }

    fn definition(&self, id: &str) -> String {
        std::fs::read_to_string(self.instance().join(format!("definitions/{id}.yaml")))
            .expect("the definition")
    }

    fn rewrite_definition(&self, id: &str, from: &str, to: &str) {
        let text = self.definition(id);
        assert!(text.contains(from), "{id} holds `{from}`");
        std::fs::write(
            self.instance().join(format!("definitions/{id}.yaml")),
            text.replace(from, to),
        )
        .expect("rewrite a definition");
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

/// The unit's own recordings: every query answered at T1 and T2.
fn standard_answers() -> Vec<Value> {
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
    answers
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn of_series<'a>(stored: &'a [Value], series: &str) -> Vec<&'a Value> {
    stored
        .iter()
        .filter(|sample| sample["series"] == series)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------------------------

/// Acceptance and story Outcome: "one recorded sample per series". A PromQL answer whose series
/// differ only in labels that are not dimensions (here `instance`, dropped because `service.queue.
/// depth` declares no dimensions) falls onto one series; the adapter reads every one of them
/// `unknown`, naming the collision, and the pass records each read: two samples for one series in
/// one pass.
#[test]
fn a_prometheus_collision_records_one_sample_per_series() {
    let world = World::new("collision");
    world.record_invokes(&[vector(
        QUEUE,
        T1_UNIX,
        &[("a", "1"), ("b", "2")],
        "instance",
    )]);
    let out = world.thalamus(&["sample", DEPTH, "--at", T1]);
    let stored = world.stored();
    let depth = of_series(&stored, DEPTH);
    assert!(
        !depth.is_empty(),
        "the collision is recorded as unknown, never dropped: {stored:#?}"
    );
    assert!(
        depth
            .iter()
            .all(|sample| sample["reading"]["quality"] == "unknown"),
        "{depth:#?}"
    );
    assert_eq!(
        depth.len(),
        1,
        "one pass records one sample per series; exit {:?}, stdout {}, stored {depth:#?}",
        out.status.code(),
        text(&out.stdout)
    );
}

/// `denominator.since` is "the earliest moment the population could contain anything" and
/// `age_s` is "seconds from since to the sample" (`spec/domains/observation.yaml`, Denominator).
/// `--at` (added by this unit) samples a past window; when a later window of the series is
/// already stored, `first_sample` resolves to the earliest *appended* sample
/// (`EventlogStore::first_sample`), so the backfilled sample's population begins after the sample
/// itself.
#[test]
fn a_backfilled_sample_does_not_begin_after_itself() {
    let world = World::new("backfill");
    for at in [T2, T1] {
        let out = world.thalamus(&["sample", RATIO, "--at", at]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    let stored = world.stored();
    assert_eq!(stored.len(), 4, "{stored:#?}");
    for sample in stored.iter().filter(|sample| sample["ts"] == T1) {
        assert_eq!(
            sample["denominator"]["since"], T1,
            "the earliest sample of the series is its own origin, not a later one: {sample:#}"
        );
    }
}

/// Story Work, instance.rs: "The grading rule of each definition is parsed at load too, so an
/// unknown rule id is refused before any read." The loader checks only `Active` and `Muted`
/// definitions, so a `Proposed` one naming no rule loads and the pass exits 0.
#[test]
fn an_unknown_rule_on_a_proposed_definition_is_refused_at_load() {
    let world = World::new("proposed_rule");
    world.add_depth_copy("service.queue.proposed_depth", "proposed");
    world.rewrite_definition(
        "service.queue.proposed_depth",
        "rule: higher_is_worse",
        "rule: no_such_rule",
    );
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(
        stderr.contains("definitions/service.queue.proposed_depth.yaml"),
        "{stderr}"
    );
    assert!(stderr.contains("no_such_rule"), "{stderr}");
    assert!(world.called().is_empty(), "no source is read");
}

/// Repository invariant: "No secret enters thalamus: ... never read, print or store a
/// credential." The store refuses a PostgreSQL URL carrying a password and does not repeat it
/// (`thalamus.store.postgres-password`), but `cmd/sample.rs` prints the locator it was given
/// verbatim in front of the refusal.
#[test]
fn a_refused_store_url_does_not_print_its_password() {
    let world = World::new("password_stderr");
    let out = world.thalamus(&[
        "--store",
        "postgresql://reader:hunter2-not-printed@127.0.0.1:1/thalamus",
        "sample",
        DEPTH,
        "--at",
        T1,
    ]);
    assert_eq!(out.status.code(), Some(74), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(stderr.contains("postgres-password"), "{stderr}");
    assert!(
        !stderr.contains("hunter2-not-printed"),
        "the password is printed: {stderr}"
    );
}

/// The same invariant through `THALAMUS_STORE`: clap shows an `env` option's current value in
/// `--help` unless told not to, so a password-bearing URL in the environment is printed by
/// `thalamus --help`.
#[test]
fn help_does_not_print_the_store_url_from_the_environment() {
    let world = World::new("password_help");
    let out = world
        .command()
        .env(
            "THALAMUS_STORE",
            "postgresql://reader:hunter2-not-printed@127.0.0.1:1/thalamus",
        )
        .arg("--help")
        .output()
        .expect("runs");
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("THALAMUS_STORE"), "{stdout}");
    assert!(
        !stdout.contains("hunter2-not-printed"),
        "the password is printed: {stdout}"
    );
}

// ---------------------------------------------------------------------------------------------
// Pins the unit's suite leaves free (green now; each names the mutant it catches)
// ---------------------------------------------------------------------------------------------

/// Catches `.max(0)` dropped from `age_s` in `thalamus-pipeline/src/sample.rs`.
#[test]
fn a_backfilled_sample_has_no_negative_age() {
    let world = World::new("backfill_age");
    for at in [T2, T1] {
        let out = world.thalamus(&["sample", RATIO, "--at", at]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    for sample in world.stored() {
        let age = sample["denominator"]["age_s"].as_i64().expect("age_s");
        assert!(age >= 0, "{sample:#}");
    }
}

/// Catches `admit` or `active` admitting `Proposed` or `Retired` (only `Muted` is pinned by the
/// unit's suite).
#[test]
fn proposed_and_retired_definitions_are_never_sampled() {
    let world = World::new("lifecycles");
    world.add_depth_copy("service.queue.proposed_depth", "proposed");
    world.add_depth_copy("service.queue.retired_depth", "retired");
    for (id, state) in [
        ("service.queue.proposed_depth", "Proposed"),
        ("service.queue.retired_depth", "Retired"),
    ] {
        let out = world.thalamus(&["sample", id, "--at", T1]);
        assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
        let stderr = text(&out.stderr);
        assert!(stderr.contains("not-active"), "{stderr}");
        assert!(stderr.contains(state), "{stderr}");
        assert!(world.called().is_empty(), "{id}: the source is not read");
    }
    assert!(world.stored().is_empty(), "nothing is recorded");

    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let signals: BTreeSet<String> = world
        .stored()
        .iter()
        .map(|sample| sample["signal"].as_str().expect("signal").to_owned())
        .collect();
    assert_eq!(
        signals,
        [RATIO, "service.cache.miss_ratio", DEPTH]
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    );
}

/// Catches `Muted` dropped from the load-time grading check in
/// `thalamus-pipeline/src/instance.rs` (the unit's suite only breaks an `Active` rule).
#[test]
fn an_unknown_rule_on_a_muted_definition_is_refused_at_load() {
    let world = World::new("muted_rule");
    world.rewrite_definition(MUTED, "rule: higher_is_worse", "rule: no_such_rule");
    let out = world.thalamus(&["sample", RATIO, "--at", T1]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let stderr = text(&out.stderr);
    assert!(
        stderr.contains("definitions/service.batch.duration.yaml"),
        "{stderr}"
    );
    assert!(stderr.contains("no_such_rule"), "{stderr}");
    assert!(world.stored().is_empty());
    assert!(world.called().is_empty());
}

/// Catches any reordering or loss of the text format's three fields: the unit's suite only
/// checks the id appears somewhere on stdout.
#[test]
fn text_format_is_id_series_quality_per_line() {
    let world = World::new("text");
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let printed: BTreeSet<(String, String, String)> = text(&out.stdout)
        .lines()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 3, "{line}");
            (
                fields[0].to_owned(),
                fields[1].to_owned(),
                fields[2].to_owned(),
            )
        })
        .collect();
    let stored: BTreeSet<(String, String, String)> = world
        .stored()
        .iter()
        .map(|sample| {
            (
                sample["sample_id"].as_str().expect("id").to_owned(),
                sample["series"].as_str().expect("series").to_owned(),
                sample["reading"]["quality"]
                    .as_str()
                    .expect("quality")
                    .to_owned(),
            )
        })
        .collect();
    assert_eq!(printed, stored);
    assert!(
        printed.iter().any(|(_, _, quality)| quality == "unknown"),
        "{printed:?}"
    );
}

/// Catches the JSON `refused` list being dropped, a dims refusal aborting the other reads, or
/// a refused read being recorded: none of which the unit's suite exercises.
#[test]
fn a_read_refused_for_its_dims_is_reported_and_the_other_reads_are_recorded() {
    let world = World::new("dims_refused");
    world.record_invokes(&[
        vector(
            FAILED,
            T1_UNIX,
            &[("checkout", "15"), ("search", "3"), ("billing", "1")],
            "component",
        ),
        vector(
            REQUESTS,
            T1_UNIX,
            &[("checkout", "556"), ("search", "120"), ("billing", "40")],
            "component",
        ),
    ]);
    let out = world.thalamus(&["sample", RATIO, "--at", T1, "--format", "json"]);
    assert_eq!(out.status.code(), Some(65), "stderr: {}", text(&out.stderr));
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    let refused = printed["refused"].as_array().expect("refused");
    assert_eq!(refused.len(), 1, "{printed:#}");
    assert!(
        refused[0]
            .as_str()
            .is_some_and(|why| why.contains("billing")),
        "{printed:#}"
    );
    let stored = world.stored();
    let series: BTreeSet<String> = stored
        .iter()
        .map(|sample| sample["series"].as_str().expect("series").to_owned())
        .collect();
    assert_eq!(
        series,
        [
            format!("{RATIO}{{component=\"checkout\"}}"),
            format!("{RATIO}{{component=\"search\"}}"),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        printed["samples"].as_array().map(Vec::len),
        Some(2),
        "{printed:#}"
    );
    assert!(text(&out.stderr).contains("billing"));
}

/// Catches `env = "THALAMUS_INSTANCE"` / `env = "THALAMUS_STORE"` dropped from `global.rs`, or
/// the environment winning over `--store`: the unit's suite removes both variables.
#[test]
fn instance_and_store_come_from_the_environment_and_the_flag_wins() {
    let world = World::new("environment");
    let from_env = world.root.join("from-env.sqlite");
    let from_flag = world.root.join("from-flag.sqlite");
    let out = world
        .command()
        .env("THALAMUS_INSTANCE", world.instance())
        .env("THALAMUS_STORE", format!("sqlite:{}", from_env.display()))
        .env("THALAMUS_CONNECTORS", double())
        .args(["sample", DEPTH, "--at", T1])
        .output()
        .expect("runs");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert_eq!(
        world
            .stored_at(&Locator::Sqlite(from_env.clone()), INSTANCE)
            .len(),
        1
    );
    assert!(
        !world.instance().join(".thalamus/store.sqlite").exists(),
        "THALAMUS_STORE replaces the default store"
    );

    let out = world
        .command()
        .env("THALAMUS_INSTANCE", world.instance())
        .env("THALAMUS_STORE", format!("sqlite:{}", from_env.display()))
        .env("THALAMUS_CONNECTORS", double())
        .args(["--store", &format!("sqlite:{}", from_flag.display())])
        .args(["sample", DEPTH, "--at", T1])
        .output()
        .expect("runs");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert_eq!(
        world.stored_at(&Locator::Sqlite(from_flag), INSTANCE).len(),
        1
    );
    assert_eq!(
        world.stored_at(&Locator::Sqlite(from_env), INSTANCE).len(),
        1,
        "--store wins over THALAMUS_STORE"
    );
}

/// `<key>.created` (D18): `checkout` carries `created`, so its origin is that instant; `search`
/// carries none, so (design row of this unit) it falls back to the first stored sample of the
/// series, else the sample itself. Catches the fallback being `end` instead of the first stored
/// sample, and `resolve_since` being bypassed.
#[test]
fn since_from_a_dimension_created_and_its_fallback() {
    let world = World::new("created");
    world.rewrite_definition(RATIO, "since: first_sample", "since: component.created");
    for at in [T1, T2] {
        let out = world.thalamus(&["sample", RATIO, "--at", at]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    let stored = world.stored();
    for sample in &stored {
        let ts_unix = if sample["ts"] == T1 { T1_UNIX } else { T2_UNIX };
        match sample["dims"]["component"].as_str() {
            Some("checkout") => {
                assert_eq!(
                    sample["denominator"]["since"], CHECKOUT_CREATED,
                    "{sample:#}"
                );
                assert_eq!(
                    sample["denominator"]["age_s"],
                    ts_unix - CHECKOUT_CREATED_UNIX,
                    "{sample:#}"
                );
            }
            Some("search") => {
                assert_eq!(sample["denominator"]["since"], T1, "{sample:#}");
                assert_eq!(
                    sample["denominator"]["age_s"],
                    ts_unix - T1_UNIX,
                    "{sample:#}"
                );
            }
            other => panic!("unexpected component {other:?}"),
        }
    }
    assert_eq!(stored.len(), 4);
}

/// A PostgreSQL store locator, as `thalamus-store` tests it: the database
/// `THALAMUS_TEST_POSTGRES_URL` names. Ignored without it; run with `--ignored`.
#[test]
#[ignore = "needs THALAMUS_TEST_POSTGRES_URL (a disposable local PostgreSQL)"]
fn a_postgres_store_records_one_pass_under_one_run_id() {
    let url = std::env::var("THALAMUS_TEST_POSTGRES_URL")
        .expect("THALAMUS_TEST_POSTGRES_URL names a disposable local PostgreSQL");
    let world = World::new("postgres");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("after the epoch")
        .as_nanos();
    let instance = format!("adv-pg-{}-{nanos}", std::process::id());
    let registry = world.instance().join("dimensions.yaml");
    let text_of = std::fs::read_to_string(&registry).expect("registry");
    std::fs::write(
        &registry,
        text_of.replace("registry_id: fixture", &format!("registry_id: {instance}")),
    )
    .expect("rename the instance");
    for _ in 0..2 {
        let out = world.thalamus(&[
            "--store", &url, "sample", "--all", "--once", "--at", T1, "--format", "json",
        ]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    assert!(
        !world.instance().join(".thalamus").exists(),
        "the default SQLite store is not touched"
    );
    let stored = world.stored_at(&url.parse().expect("a locator"), &instance);
    assert_eq!(stored.len(), 8, "{stored:#?}");
    let runs: BTreeSet<String> = stored
        .iter()
        .map(|sample| {
            sample["provenance"]["run_id"]
                .as_str()
                .expect("run")
                .to_owned()
        })
        .collect();
    assert_eq!(runs.len(), 2, "one run id per pass: {runs:?}");
    let ids: BTreeSet<&str> = stored
        .iter()
        .map(|sample| sample["sample_id"].as_str().expect("id"))
        .collect();
    assert_eq!(ids.len(), 8, "every sample id is distinct");
}
