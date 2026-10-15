//! `thalamus eval` and `thalamus show`: the built binary against the fixture instance
//! (`tests/fixtures/eval_show`), a store seeded by `thalamus sample`, and the Connectors double
//! (`thalamus-sources`, `src/bin/connectors-double.rs`).
//!
//! Each test copies the instance into its own directory under the build's temporary directory,
//! so the default store (`sqlite:<instance>/.thalamus/store.sqlite`) and the double's call log
//! land there. The double's recordings are written per test, at the instants the test samples
//! and grades at, because Prometheus answers name the instant they were evaluated at.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{CompleteReading, Reading};
use thalamus_store::eventlog::{EventlogStore, Locator};

/// One day, in seconds.
const DAY: i64 = 86_400;

/// The grading instant of most tests, and its Unix seconds.
const T: &str = "2026-10-14T22:57:03Z";
const T_UNIX: i64 = 1_792_018_623;

/// The registry id of the fixture instance: the name its store stream is kept under.
const INSTANCE: &str = "fixture";

const RATIO: &str = "service.api.request_failure_ratio";
const MISSES: &str = "service.cache.miss_ratio";
const DEPTH: &str = "service.queue.depth";
const DEPTH_1D: &str = "service.queue.depth_vs_1d";

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

/// The RFC 3339 text of Unix second `at`, in UTC.
fn rfc3339(at: i64) -> String {
    let days = at.div_euclid(DAY);
    let second = at.rem_euclid(DAY);
    // Civil from days (Howard Hinnant's algorithm), for the proleptic Gregorian calendar.
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

/// One test's directory: the instance copy, the recordings and the call log.
struct World {
    root: PathBuf,
    /// The `series.query` answers of `metrics-main`, one per query and instant.
    answers: Vec<Value>,
}

impl World {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("cli_eval_show")
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
        world.record_connections();
        world.flush();
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
    /// answers what [`World::ratio`] and [`World::depth`] recorded; `metrics-down` refuses every
    /// invoke.
    fn record_connections(&self) {
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

    fn flush(&self) {
        self.write(
            "operations+invoke+a=metrics+c=metrics-main+o=series.query.json",
            &Value::Array(self.answers.clone()),
        );
    }

    /// The failure ratio at `at`: `(failed, requests)` of `checkout` and of `search`.
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

    /// The queue depth at `at`.
    fn depth(&mut self, at: i64, value: &str) {
        self.answers.push(vector(QUEUE, at, &[("", value)], ""));
        self.flush();
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

    /// `thalamus sample <signal> --at <at>`, which must exit 0; the run id it printed.
    fn sample(&self, signal: &str, at: i64) -> String {
        let out = self.thalamus(&["sample", signal, "--at", &rfc3339(at), "--format", "json"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "sample {signal} at {at}: {}",
            text(&out.stderr)
        );
        let printed: Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
        printed["run_id"].as_str().expect("run_id").to_owned()
    }

    /// `thalamus eval <args>`, which must exit 0; the evaluations it printed.
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

    /// `thalamus show <args> --format json`: its exit code and the verdicts it printed.
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

    /// Every evaluation on the instance's stream, in the order appended.
    fn evaluations(&self) -> Vec<SignalEvaluationData> {
        let locator = Locator::Sqlite(self.instance().join(".thalamus/store.sqlite"));
        let store = EventlogStore::open(&locator, INSTANCE).expect("the store opens");
        store.evaluations().to_vec()
    }

    /// The `time_unix_s` of every `series.query` invoke the double received, in call order.
    fn invoked_at(&self) -> Vec<i64> {
        std::fs::read_to_string(self.calls())
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a log line is JSON"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|arg| arg.as_str().map(str::to_owned))
                    .collect();
                if !argv.starts_with(&["operations".to_owned(), "invoke".to_owned()]) {
                    return None;
                }
                let at = argv.iter().position(|arg| arg == "--input-json")?;
                let input: Value = serde_json::from_str(argv.get(at + 1)?).ok()?;
                input["time_unix_s"].as_i64()
            })
            .collect()
    }

    /// Rewrites the definition file of `signal` with `edit`.
    fn edit(&self, signal: &str, edit: impl Fn(String) -> String) {
        let file = self
            .instance()
            .join("definitions")
            .join(format!("{signal}.yaml"));
        let held = std::fs::read_to_string(&file).expect("the definition");
        std::fs::write(&file, edit(held)).expect("write the definition");
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

fn checkout() -> String {
    format!("{RATIO}{{component=\"checkout\"}}")
}

fn search() -> String {
    format!("{RATIO}{{component=\"search\"}}")
}

fn verdicts(shown: &[Value]) -> Vec<&str> {
    shown
        .iter()
        .map(|row| row["verdict"].as_str().expect("verdict"))
        .collect()
}

#[test]
fn rfc3339_writes_the_instants_the_tests_use() {
    assert_eq!(rfc3339(T_UNIX), T);
    assert_eq!(rfc3339(T_UNIX - 7 * DAY), "2026-10-07T22:57:03Z");
    assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
}

/// An unknown sample is graded `UNKNOWN`, never `OK` and never 0, and `show` exits 3.
#[test]
fn unknown_sample_shows_unknown_and_exits_3() {
    let world = World::new("unknown");
    world.sample(MISSES, T_UNIX);

    let recorded = world.eval(&[]);
    assert_eq!(recorded.len(), 1, "{recorded:#?}");
    assert_eq!(recorded[0]["series"], MISSES);
    assert_eq!(recorded[0]["verdict"], "UNKNOWN");
    assert!(
        recorded[0]["evaluation_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("evl_")),
        "{recorded:#?}"
    );

    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(3), "{shown:#?}");
    assert_eq!(shown.len(), 1, "{shown:#?}");
    let row = &shown[0];
    assert_eq!(row["series"], MISSES);
    assert_eq!(row["verdict"], "UNKNOWN");
    assert_eq!(row["value"]["quality"], "unknown", "{row:#}");
    assert!(row["value"].get("value").is_none(), "no value: {row:#}");
    assert!(row["score"].is_null(), "an UNKNOWN carries no score: {row:#}");
    assert!(
        row["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("unavailable")),
        "{row:#}"
    );

    let out = world.thalamus(&["show"]);
    assert_eq!(out.status.code(), Some(3));
    let shown = text(&out.stdout);
    assert!(shown.contains("UNKNOWN") && shown.contains(MISSES), "{shown}");

    let stored = world.evaluations();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].verdict, Verdict::UNKNOWN);
    assert_eq!(stored[0].score, None);
}

/// D5: one evaluation per sample and definition revision. A second run records nothing and does
/// not fail; a revision grades every sample again, once; `--run` grades one pass only.
#[test]
fn eval_twice_records_each_sample_once_per_rev() {
    let mut world = World::new("twice");
    world.depth(T_UNIX - 3600, "42");
    world.depth(T_UNIX, "42");
    world.sample(DEPTH, T_UNIX - 3600);
    let run = world.sample(DEPTH, T_UNIX);

    let recorded = world.eval(&["--run", &run]);
    assert_eq!(recorded.len(), 1, "only the samples of {run}: {recorded:#?}");
    assert_eq!(recorded[0]["verdict"], "OK");

    let recorded = world.eval(&[DEPTH]);
    assert_eq!(recorded.len(), 1, "the sample not yet graded: {recorded:#?}");

    let again = world.eval(&[]);
    assert!(again.is_empty(), "a second run records nothing: {again:#?}");
    let stored = world.evaluations();
    assert_eq!(stored.len(), 2, "{stored:#?}");
    let samples: BTreeSet<&str> = stored.iter().map(|row| row.sample_id.0.as_str()).collect();
    assert_eq!(samples.len(), 2, "each sample once: {stored:#?}");
    assert!(stored.iter().all(|row| row.definition_rev == 1));

    world.edit(DEPTH, |held| held.replace("rev: 1\n", "rev: 2\n"));
    let regraded = world.eval(&[]);
    assert_eq!(regraded.len(), 2, "each sample again at rev 2: {regraded:#?}");
    assert!(world.eval(&[]).is_empty(), "and only once");
    let stored = world.evaluations();
    assert_eq!(stored.len(), 4);
    assert_eq!(
        stored.iter().filter(|row| row.definition_rev == 2).count(),
        2
    );
}

/// `history: source`: an offset baseline is read from the source at the shifted window end, one
/// day and seven days earlier, and pooled.
#[test]
fn offset_baseline_is_read_from_source_at_shifted_time() {
    let mut world = World::new("offset_source");
    let first = T_UNIX - 8 * DAY;
    world.ratio(first, ("10", "500"), ("2", "100"));
    world.ratio(T_UNIX - DAY, ("10", "500"), ("2", "100"));
    world.ratio(T_UNIX - 7 * DAY, ("6", "300"), ("2", "100"));
    world.ratio(T_UNIX, ("15", "556"), ("3", "120"));
    world.sample(RATIO, first);
    world.sample(RATIO, T_UNIX);
    let before = world.invoked_at().len();

    world.eval(&[RATIO]);
    let read: BTreeSet<i64> = world.invoked_at().split_off(before).into_iter().collect();
    assert!(
        read.contains(&(T_UNIX - DAY)) && read.contains(&(T_UNIX - 7 * DAY)),
        "the source is read one day and seven days earlier: {read:?}"
    );

    let stored = world.evaluations();
    let at_t = |series: &str| {
        stored
            .iter()
            .find(|row| row.series.0 == series && row.ts.0 == T)
            .unwrap_or_else(|| panic!("an evaluation of {series} at {T}: {stored:#?}"))
    };
    for series in [checkout(), search()] {
        let row = at_t(&series);
        let baseline = row.baseline.as_ref().expect("a baseline");
        assert_eq!(baseline.kind.0, "pooled");
        assert_eq!(baseline.offsets, ["1d", "7d"]);
        assert_eq!(
            baseline.reading,
            Reading::Ok(CompleteReading {
                value: thalamus_model::primitives::Decimal("0.02".to_owned())
            }),
            "{row:#?}"
        );
        assert_eq!(row.verdict, Verdict::OK, "{row:#?}");
    }

    let (code, shown) = world.show(&[RATIO]);
    assert_eq!(code, Some(0), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["OK", "OK"]);
    assert_eq!(shown[0]["baseline"]["reading"]["value"], "0.02", "{shown:#?}");
}

/// `history: store`: an offset baseline is the stored sample of the series at the offset, and
/// the source is not read for it.
#[test]
fn offset_baseline_of_store_history_is_read_from_the_store() {
    let mut world = World::new("offset_store");
    world.depth(T_UNIX - 2 * DAY, "100");
    world.depth(T_UNIX - DAY, "100");
    world.depth(T_UNIX, "300");
    for at in [T_UNIX - 2 * DAY, T_UNIX - DAY, T_UNIX] {
        world.sample(DEPTH_1D, at);
    }
    let sampled = world.invoked_at();

    world.eval(&[DEPTH_1D]);
    assert_eq!(world.invoked_at(), sampled, "no source read for the baseline");
    let stored = world.evaluations();
    let row = stored
        .iter()
        .find(|row| row.ts.0 == T)
        .unwrap_or_else(|| panic!("an evaluation at {T}: {stored:#?}"));
    let baseline = row.baseline.as_ref().expect("a baseline");
    assert_eq!(baseline.kind.0, "offset_1d");
    assert_eq!(
        baseline.reading,
        Reading::Ok(CompleteReading {
            value: thalamus_model::primitives::Decimal("100".to_owned())
        }),
        "{row:#?}"
    );
    assert_eq!(row.verdict, Verdict::WARN, "300 >= 2 x 100: {row:#?}");
}

/// Rule 2 (L6): a baseline window that starts before the series began is `UNKNOWN`.
#[test]
fn baseline_older_than_series_shows_unknown() {
    let mut world = World::new("predates");
    world.ratio(T_UNIX, ("15", "556"), ("3", "120"));
    world.sample(RATIO, T_UNIX);
    world.eval(&[]);

    let (code, shown) = world.show(&[RATIO]);
    assert_eq!(code, Some(3), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["UNKNOWN", "UNKNOWN"]);
    for row in &shown {
        assert!(
            row["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("baseline predates series")),
            "{row:#}"
        );
        assert_eq!(row["baseline"]["reading"]["quality"], "unknown", "{row:#}");
        assert!(row["score"].is_null(), "{row:#}");
    }
}

/// The ladder: the worst current verdict is the exit code, `WARN` 1 and `CRIT` 2, and the rows
/// are shown worst first.
#[test]
fn show_exits_with_worst_verdict() {
    let mut world = World::new("worst");
    let t1 = T_UNIX - 3600;
    world.depth(t1, "500");
    world.depth(T_UNIX, "5000");

    world.sample(DEPTH, t1);
    world.eval(&[]);
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(1), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["WARN"]);
    assert!(shown[0]["score"].is_string(), "a WARN is scored: {shown:#?}");
    assert_eq!(shown[0]["value"]["value"], "500", "{shown:#?}");

    world.sample(DEPTH, T_UNIX);
    world.sample(MISSES, T_UNIX);
    world.eval(&[]);
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(2), "{shown:#?}");
    assert_eq!(
        verdicts(&shown),
        ["CRIT", "UNKNOWN"],
        "the newest verdict of each series, worst first"
    );
    assert_eq!(shown[0]["series"], DEPTH);
    assert_eq!(shown[0]["ts"], T);

    let (code, shown) = world.show(&[MISSES]);
    assert_eq!(code, Some(3), "one signal's verdicts only: {shown:#?}");
    assert_eq!(verdicts(&shown), ["UNKNOWN"]);

    let out = world.thalamus(&["show", "service.no.such_signal"]);
    assert_eq!(out.status.code(), Some(65), "{}", text(&out.stderr));
    assert!(text(&out.stderr).contains("unknown-signal"));
}

/// A pass in which every shown verdict is `N/A`, or nothing is shown, exits 3.
#[test]
fn show_all_na_exits_3() {
    let mut world = World::new("all_na");
    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(3), "nothing shown: {shown:#?}");
    assert!(shown.is_empty());

    world.ratio(T_UNIX, ("1", "5"), ("0", "3"));
    world.sample(RATIO, T_UNIX);
    world.eval(&[]);
    let (code, shown) = world.show(&[RATIO]);
    assert_eq!(code, Some(3), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["N/A", "N/A"]);
    assert!(shown.iter().all(|row| row["score"].is_null()), "{shown:#?}");
}

/// D6: `prev` is the previous raw verdict of the series, by `ts`, across runs of `eval` and
/// whatever order the samples were stored in.
#[test]
fn prev_is_previous_raw_verdict() {
    let mut world = World::new("prev");
    let at: Vec<i64> = (0..4).map(|step| T_UNIX + step * 300).collect();
    for (instant, value) in at.iter().zip(["42", "500", "5000", "42"]) {
        world.depth(*instant, value);
    }
    world.sample(DEPTH, at[0]);
    world.sample(DEPTH, at[1]);
    world.eval(&[]);
    world.sample(DEPTH, at[3]);
    world.sample(DEPTH, at[2]);
    world.eval(&[]);

    let stored = world.evaluations();
    let graded: Vec<(String, Verdict, Option<Verdict>)> = at
        .iter()
        .map(|instant| {
            let ts = rfc3339(*instant);
            let row = stored
                .iter()
                .find(|row| row.ts.0 == ts)
                .unwrap_or_else(|| panic!("an evaluation at {ts}: {stored:#?}"));
            (ts, row.verdict, row.prev)
        })
        .collect();
    let expected = [
        (Verdict::OK, None),
        (Verdict::WARN, Some(Verdict::OK)),
        (Verdict::CRIT, Some(Verdict::WARN)),
        (Verdict::OK, Some(Verdict::CRIT)),
    ];
    for ((ts, verdict, prev), (want, want_prev)) in graded.iter().zip(expected) {
        assert_eq!((*verdict, *prev), (want, want_prev), "at {ts}");
    }

    let (code, shown) = world.show(&[]);
    assert_eq!(code, Some(0), "{shown:#?}");
    assert_eq!(shown[0]["prev"], "CRIT", "{shown:#?}");
}
