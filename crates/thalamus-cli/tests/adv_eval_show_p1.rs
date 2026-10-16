//! Adversary pass 1 against `thalamus eval` and `thalamus show` (story:m1-cli-eval-show): the
//! built binary against a copy of the `eval_show` fixture instance and the Connectors double,
//! with recordings this file writes.
//!
//! Cases named `*_red_*` are findings; the others pin behaviour the unit's own suite leaves free
//! (each names the mutant it catches).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::Reading;
use thalamus_store::eventlog::{EventlogStore, Locator};

const DAY: i64 = 86_400;
const T_UNIX: i64 = 1_792_018_623;
const INSTANCE: &str = "fixture";

const RATIO: &str = "service.api.request_failure_ratio";
const MISSES: &str = "service.cache.miss_ratio";
const DEPTH: &str = "service.queue.depth";
const DEPTH_1D: &str = "service.queue.depth_vs_1d";

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

/// The RFC 3339 text of Unix second `at`, in UTC.
fn rfc3339(at: i64) -> String {
    let days = at.div_euclid(DAY);
    let second = at.rem_euclid(DAY);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        second / 3600,
        second % 3600 / 60,
        second % 60
    )
}

struct World {
    root: PathBuf,
    answers: Vec<Value>,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_eval_show_p1")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/eval_show"),
            &root.join("instance"),
        );
        let world = Self {
            root,
            answers: Vec::new(),
        };
        std::fs::create_dir_all(world.recordings()).expect("create the recordings directory");
        world.write(
            "connections+list+a=metrics.json",
            &json!({"stdout": {"ok": true, "result": {"connections": [
                {"adapter": "metrics", "connection": "metrics-main", "state": "ready", "revision": "conn-rev-1"},
                {"adapter": "metrics", "connection": "metrics-down", "state": "ready", "revision": "conn-rev-1"}
            ]}}}),
        );
        world.write(
            "operations+describe+a=metrics+o=series.query.json",
            &json!({"stdout": {"ok": true, "result": {
                "adapter": "metrics", "operation": "series.query",
                "schema": "series.query/v1alpha1", "revision": "op-rev-1"
            }}}),
        );
        world.write(
            "operations+invoke+a=metrics+c=metrics-down+o=series.query.json",
            &json!({"stderr": {"ok": false, "error": {
                "code": "failure",
                "message": "the provider did not answer",
                "data": {"kind": "operational", "code": "unavailable", "stage": "dispatch"}
            }}}),
        );
        world.flush();
        world
    }

    fn instance(&self) -> PathBuf {
        self.root.join("instance")
    }

    fn recordings(&self) -> PathBuf {
        self.root.join("connectors")
    }

    fn store_file(&self) -> PathBuf {
        self.instance().join(".thalamus/store.sqlite")
    }

    fn write(&self, name: &str, answer: &Value) {
        std::fs::write(
            self.recordings().join(name),
            serde_json::to_string_pretty(answer).expect("JSON"),
        )
        .expect("write a recording");
    }

    fn flush(&self) {
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(self.answers.clone()),
        );
    }

    fn ratio(&mut self, at: i64, checkout: (&str, &str), search: (&str, &str)) {
        self.answers.push(vector(
            FAILED,
            at,
            &[("checkout", checkout.0), ("search", search.0)],
            "component",
        ));
        self.answers.push(vector(
            REQUESTS,
            at,
            &[("checkout", checkout.1), ("search", search.1)],
            "component",
        ));
        self.flush();
    }

    fn depth(&mut self, at: i64, value: &str) {
        self.answers.push(vector(QUEUE, at, &[("", value)], ""));
        self.flush();
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_thalamus"));
        command
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(double())
            .args(args)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("CONNECTORS_DOUBLE_FIXTURES", self.recordings())
            .env("CONNECTORS_DOUBLE_LOG", self.root.join("calls.jsonl"));
        command
    }

    fn thalamus(&self, args: &[&str]) -> Output {
        self.command(args)
            .output()
            .expect("the thalamus binary runs")
    }

    fn sample(&self, signal: &str, at: i64) {
        let out = self.thalamus(&["sample", signal, "--at", &rfc3339(at), "--format", "json"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "sample {signal} at {at}: {}",
            text(&out.stderr)
        );
    }

    fn eval(&self, args: &[&str]) -> Vec<Value> {
        let mut argv = vec!["eval", "--format", "json"];
        argv.extend_from_slice(args);
        let out = self.thalamus(&argv);
        assert_eq!(
            out.status.code(),
            Some(0),
            "eval {args:?}: {}",
            text(&out.stderr)
        );
        let printed: Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
        printed["evaluations"]
            .as_array()
            .expect("evaluations")
            .clone()
    }

    fn show(&self, args: &[&str]) -> (Option<i32>, Vec<Value>) {
        let mut argv = vec!["show", "--format", "json"];
        argv.extend_from_slice(args);
        let out = self.thalamus(&argv);
        let printed: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|error| {
            panic!(
                "show {args:?} printed no JSON document ({error}): stdout {} stderr {}",
                text(&out.stdout),
                text(&out.stderr)
            )
        });
        (
            out.status.code(),
            printed["verdicts"].as_array().expect("verdicts").clone(),
        )
    }

    fn evaluations(&self) -> Vec<SignalEvaluationData> {
        let store = EventlogStore::open(&Locator::Sqlite(self.store_file()), INSTANCE)
            .expect("the store opens");
        store.evaluations().to_vec()
    }

    fn edit(&self, signal: &str, edit: impl Fn(String) -> String) {
        let file = self
            .instance()
            .join("definitions")
            .join(format!("{signal}.yaml"));
        let held = std::fs::read_to_string(&file).expect("the definition");
        let edited = edit(held.clone());
        assert_ne!(edited, held, "the edit changed {signal}");
        std::fs::write(&file, edited).expect("write the definition");
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn verdicts(shown: &[Value]) -> Vec<&str> {
    shown
        .iter()
        .map(|row| row["verdict"].as_str().expect("verdict"))
        .collect()
}

/// `history: store`, `offset_1d`, window `PT5M`: the shifted window of the sample at `T` is
/// `[T-1d-5m, T-1d]`. The only stored sample near it is the one at `T-1d-5m`, whose own window
/// is `[T-1d-10m, T-1d-5m]`: the slot before, touching the shifted window at one instant only.
/// It is not "the same window 1 day earlier" (design § 3.1), so the baseline is unread and the
/// verdict `UNKNOWN`. `from_store` takes it because `series_range` includes its start bound.
#[test]
fn store_offset_baseline_red_takes_the_previous_slot_on_the_window_start() {
    let mut world = World::new("store_window_start");
    let previous_slot = T_UNIX - DAY - 300;
    world.depth(previous_slot, "100");
    world.depth(T_UNIX, "300");
    world.sample(DEPTH_1D, previous_slot);
    world.sample(DEPTH_1D, T_UNIX);
    world.eval(&[DEPTH_1D]);

    let stored = world.evaluations();
    let row = stored
        .iter()
        .find(|row| row.ts.0 == rfc3339(T_UNIX))
        .unwrap_or_else(|| panic!("an evaluation at T: {stored:#?}"));
    let baseline = row.baseline.as_ref().expect("a baseline");
    assert!(
        matches!(baseline.reading, Reading::Unknown(_)),
        "the sample of the slot before the shifted window is no offset_1d baseline: {row:#?}"
    );
    assert_eq!(row.verdict, Verdict::UNKNOWN, "{row:#?}");
}

/// `history: store`, `offset_1d`: a series whose first stored sample is the one graded has no
/// sample one day earlier; the baseline is unknown (never 0) and the verdict `UNKNOWN` with no
/// score. Pins `from_store`'s `None` arm.
#[test]
fn store_offset_baseline_before_the_first_sample_is_unknown() {
    let mut world = World::new("store_before_first");
    world.depth(T_UNIX, "300");
    world.sample(DEPTH_1D, T_UNIX);
    world.eval(&[]);
    let (code, shown) = world.show(&[DEPTH_1D]);
    assert_eq!(code, Some(3), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["UNKNOWN"]);
    assert_eq!(shown[0]["baseline"]["reading"]["quality"], "unknown");
    assert!(shown[0]["baseline"]["reading"].get("value").is_none());
    assert!(shown[0]["score"].is_null());
}

/// `thalamus show` reads. On a valid instance that has never sampled, it shows nothing and exits
/// 3, and leaves no store behind: a read that creates `.thalamus/store.sqlite` writes to the
/// instance directory as a side effect.
#[test]
fn show_red_on_a_fresh_instance_creates_the_store() {
    let world = World::new("fresh_show");
    assert!(!world.store_file().exists());
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(3));
    assert!(shown.is_empty());
    assert!(
        !world.store_file().exists() && !world.instance().join(".thalamus").exists(),
        "show created {}",
        world.store_file().display()
    );
}

/// D5 under two writers: two `thalamus eval` processes over the same stored samples, three
/// revisions in a row. Both exit 0, and every sample is graded exactly once per revision.
#[test]
fn two_concurrent_evals_grade_each_sample_once_per_revision() {
    let mut world = World::new("concurrent");
    let at: Vec<i64> = (0..12).map(|step| T_UNIX + step * 300).collect();
    for instant in &at {
        world.depth(*instant, "42");
    }
    for instant in &at {
        world.sample(DEPTH, *instant);
    }
    for rev in 1..=3_i64 {
        if rev > 1 {
            world.edit(DEPTH, |held| {
                held.replace(&format!("rev: {}\n", rev - 1), &format!("rev: {rev}\n"))
            });
        }
        let first = world
            .command(&["eval", "--format", "json"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("first eval starts");
        let second = world
            .command(&["eval", "--format", "json"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("second eval starts");
        let mut printed = 0;
        for child in [first, second] {
            let out = child.wait_with_output().expect("eval ends");
            assert_eq!(
                out.status.code(),
                Some(0),
                "rev {rev}: {}",
                text(&out.stderr)
            );
            let document: Value = serde_json::from_slice(&out.stdout).expect("JSON");
            printed += document["evaluations"]
                .as_array()
                .expect("evaluations")
                .len();
        }
        let stored: Vec<SignalEvaluationData> = world
            .evaluations()
            .into_iter()
            .filter(|row| row.definition_rev == rev)
            .collect();
        let samples: BTreeSet<&str> = stored.iter().map(|row| row.sample_id.0.as_str()).collect();
        assert_eq!(stored.len(), at.len(), "rev {rev}: {stored:#?}");
        assert_eq!(samples.len(), at.len(), "rev {rev}");
        assert_eq!(
            printed,
            at.len(),
            "rev {rev}: each recorded once, by one of them"
        );
    }
}

/// D6 across a revision: the re-grade of a sample at rev 2 takes `prev` from the newest
/// *earlier* evaluation of the series, never from its own rev-1 evaluation at the same `ts`, and
/// the next sample's `prev` is the rev-2 verdict of the earlier sample, not its rev-1 one. Then
/// `show` answers the rev-2 verdict at the shared `ts`. Catches `previous_verdict`'s `is_lt()`
/// mutated to `is_le()`, which `eval_show.rs` lets through.
#[test]
fn regrade_takes_prev_from_the_earlier_sample_at_the_new_revision() {
    let mut world = World::new("prev_regrade");
    let (t1, t2) = (T_UNIX, T_UNIX + 300);
    world.depth(t1, "500");
    world.depth(t2, "42");
    world.sample(DEPTH, t1);
    world.eval(&[]);
    let (code, _) = world.show(&[]);
    assert_eq!(code, Some(1), "500 >= warn 100");

    world.edit(DEPTH, |held| {
        held.replace("rev: 1\n", "rev: 2\n")
            .replace("warn: { abs: \"100\" }", "warn: { abs: \"600\" }")
    });
    let regraded = world.eval(&[]);
    assert_eq!(regraded.len(), 1, "{regraded:#?}");
    assert_eq!(regraded[0]["verdict"], "OK");
    assert_eq!(
        regraded[0]["prev"],
        Value::Null,
        "no earlier sample: {regraded:#?}"
    );
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(0), "the rev-2 verdict is current: {shown:#?}");
    assert_eq!(shown[0]["definition_rev"], 2);

    world.sample(DEPTH, t2);
    let next = world.eval(&[]);
    assert_eq!(next.len(), 1, "{next:#?}");
    assert_eq!(
        next[0]["prev"], "OK",
        "the earlier sample's rev-2 verdict, not its rev-1 WARN: {next:#?}"
    );
}

/// The ladder over mixed series (design § 2.3, `CRIT > WARN > UNKNOWN > OK = N/A`): `OK` and
/// `N/A` exit 0; an `UNKNOWN` beside them exits 3; a `WARN` beside an `UNKNOWN` exits 1.
#[test]
fn show_exit_follows_the_ladder_over_mixed_series() {
    let mut world = World::new("ladder");
    world.ratio(T_UNIX, ("1", "5"), ("0", "3"));
    world.depth(T_UNIX, "42");
    world.depth(T_UNIX + 300, "500");
    world.sample(RATIO, T_UNIX);
    world.sample(DEPTH, T_UNIX);
    world.eval(&[]);
    let (code, shown) = world.show(&[]);
    assert_eq!(verdicts(&shown).len(), 3, "{shown:#?}");
    assert_eq!(code, Some(0), "OK and N/A: {shown:#?}");

    world.sample(MISSES, T_UNIX);
    world.eval(&[]);
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(3), "UNKNOWN beside OK and N/A: {shown:#?}");
    assert_eq!(verdicts(&shown)[0], "UNKNOWN", "worst first: {shown:#?}");

    world.sample(DEPTH, T_UNIX + 300);
    world.eval(&[]);
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(1), "WARN beside UNKNOWN: {shown:#?}");
    assert_eq!(verdicts(&shown)[..2], ["WARN", "UNKNOWN"], "{shown:#?}");
}

/// The refusal codes of `thalamus sample`, never a ladder code: no instance 64, an unknown or
/// ungraded signal 65, an invalid instance 65, a store locator no build opens 74.
#[test]
fn eval_and_show_refusals_use_the_codes_of_sample() {
    let world = World::new("codes");
    for command in ["eval", "show"] {
        let out = Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg(command)
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .output()
            .expect("runs");
        assert_eq!(
            out.status.code(),
            Some(64),
            "{command}: {}",
            text(&out.stderr)
        );

        let out = world.thalamus(&[command, "--store", "bogus:locator"]);
        assert_eq!(
            out.status.code(),
            Some(74),
            "{command}: {}",
            text(&out.stderr)
        );
    }
    let out = world.thalamus(&["eval", "service.no.such_signal"]);
    assert_eq!(out.status.code(), Some(65), "{}", text(&out.stderr));
    assert!(text(&out.stderr).contains("unknown-signal"));

    world.edit(DEPTH, |held| {
        held.replace("lifecycle: active", "lifecycle: proposed")
    });
    let out = world.thalamus(&["eval", DEPTH]);
    assert_eq!(out.status.code(), Some(65), "{}", text(&out.stderr));

    std::fs::write(
        world.instance().join("definitions/broken.yaml"),
        "id: [not, a, definition\n",
    )
    .expect("write a broken definition");
    for command in ["eval", "show"] {
        let out = world.thalamus(&[command]);
        assert_eq!(
            out.status.code(),
            Some(65),
            "{command}: {}",
            text(&out.stderr)
        );
    }
}
