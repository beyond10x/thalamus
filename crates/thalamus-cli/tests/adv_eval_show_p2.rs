//! Adversary pass 2 against `thalamus eval` and `thalamus show` (story:m1-cli-eval-show), at the
//! unit after its first correction: the built binary against a copy of the `eval_show` fixture
//! instance and the Connectors double, with recordings this file writes.
//!
//! Cases named `*_red_*` are findings; the others pin behaviour the unit's suite leaves free
//! (each names the mutant it catches).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_model::evaluation::{SignalEvaluationData, Verdict};
use thalamus_model::observation::{CompleteReading, Reading};
use thalamus_model::primitives::Decimal;
use thalamus_store::eventlog::{EventlogStore, Locator};

const DAY: i64 = 86_400;
const T_UNIX: i64 = 1_792_018_623;
const INSTANCE: &str = "fixture";

const DEPTH: &str = "service.queue.depth";
const DEPTH_1D: &str = "service.queue.depth_vs_1d";

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
            .join("adv_eval_show_p2")
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

impl World {
    /// The source fails the queue-depth query at `at`: the sample there is `unknown`.
    fn depth_unread(&mut self, at: i64) {
        self.answers.push(json!({
            "input": {"query": QUEUE, "time_unix_s": at},
            "stderr": {"ok": false, "error": {
                "code": "failure",
                "message": "the provider did not answer",
                "data": {"kind": "operational", "code": "unavailable", "stage": "dispatch"}
            }}
        }));
        self.flush();
    }

    fn at_the_time(&self, at: i64) -> SignalEvaluationData {
        let stored = self.evaluations();
        stored
            .iter()
            .find(|row| row.ts.0 == rfc3339(at))
            .cloned()
            .unwrap_or_else(|| panic!("an evaluation at {}: {stored:#?}", rfc3339(at)))
    }
}

/// `history: store`, `offset_1d`, window and cadence `PT5M`. Yesterday's slots were stamped on
/// the cadence, at `T-1d-5m` (100) and `T-1d` (300); today's pass ran 30 s early, at `T-30s`
/// (300), as an unaligned `thalamus sample` (default `--at` now) or a cron job does. The shifted
/// window is `[T-1d-5m30s, T-1d-30s]`: the sample at `T-1d` covers 4 m 30 s of it, the one at
/// `T-1d-5m` 30 s. The correction's rule (`ts` in `(start, end]`) drops the first for being 30 s
/// past the end and takes the second, the previous slot, which is the defect of pass 1's F1 again
/// at any jitter: the verdict is `WARN` against 100 where the same window a day earlier read 300.
#[test]
fn store_offset_red_takes_the_previous_slot_when_the_pass_runs_early() {
    let mut world = World::new("store_offset_early");
    let (previous_slot, same_slot, today) = (T_UNIX - DAY - 300, T_UNIX - DAY, T_UNIX - 30);
    // The series began two days ago, so no baseline predates it (rule 2).
    let began = T_UNIX - 2 * DAY;
    world.depth(began, "300");
    world.depth(previous_slot, "100");
    world.depth(same_slot, "300");
    world.depth(today, "300");
    for at in [began, previous_slot, same_slot, today] {
        world.sample(DEPTH_1D, at);
    }
    world.eval(&[DEPTH_1D]);

    let row = world.at_the_time(today);
    let baseline = row.baseline.as_ref().expect("a baseline");
    assert_ne!(
        baseline.reading,
        Reading::Ok(CompleteReading {
            value: Decimal("100".to_owned())
        }),
        "the slot before yesterday's same slot is no offset_1d baseline: {row:#?}"
    );
    assert_ne!(row.verdict, Verdict::WARN, "{row:#?}");
}

/// `history: store`, `offset_1d`: the stored sample one day earlier is itself `unknown` (the
/// source failed then). The baseline is unknown, never 0, and the verdict `UNKNOWN` with no
/// score; `show` exits 3. Pins `from_store` handing the stored reading through as it is.
#[test]
fn store_offset_of_an_unknown_stored_sample_is_unknown() {
    let mut world = World::new("store_offset_unknown");
    world.depth_unread(T_UNIX - DAY);
    world.depth(T_UNIX, "300");
    world.sample(DEPTH_1D, T_UNIX - DAY);
    world.sample(DEPTH_1D, T_UNIX);
    world.eval(&[DEPTH_1D]);

    let row = world.at_the_time(T_UNIX);
    let baseline = row.baseline.as_ref().expect("a baseline");
    assert!(matches!(baseline.reading, Reading::Unknown(_)), "{row:#?}");
    assert_eq!(row.verdict, Verdict::UNKNOWN, "{row:#?}");
    assert!(row.score.is_none(), "{row:#?}");
    let (code, shown) = world.show(&[DEPTH_1D]);
    assert_eq!(code, Some(3), "{shown:#?}");
    assert_eq!(verdicts(&shown), ["UNKNOWN"]);
}

/// A store the user cannot write (the file mode 0444: another user's instance, a read-only copy)
/// is unavailable for every command, `show` included: exit 74, as `compare` answers it.
#[test]
fn show_on_a_store_it_may_not_write_is_unavailable() {
    use std::os::unix::fs::PermissionsExt;
    let mut world = World::new("read_only_store");
    world.depth(T_UNIX, "500");
    world.sample(DEPTH, T_UNIX);
    world.eval(&[DEPTH]);
    let (code, _) = world.show(&[]);
    assert_eq!(code, Some(1), "500 >= warn 100");

    let file = world.store_file();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444))
        .expect("make the store read-only");
    let out = world.thalamus(&["show", "--format", "json"]);
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644))
        .expect("make the store writable again");
    assert_eq!(
        out.status.code(),
        Some(74),
        "a read-only store is unavailable: stdout {} stderr {}",
        text(&out.stdout),
        text(&out.stderr)
    );
}

/// `show` answers a store that does not exist as empty only when the locator names an SQLite
/// file; a locator that names no file (a directory, an in-memory name, a path under a regular
/// file) is refused 74 as `eval` refuses it, and nothing is created. Catches the
/// `sqlite_file(path).is_ok()` guard of `Locator::names_a_missing_sqlite_file` dropped, and its
/// `NotFound` match widened to any error.
#[test]
fn show_refuses_a_locator_that_names_no_sqlite_file() {
    let world = World::new("show_locators");
    let plain = world.root.join("plain.txt");
    std::fs::write(&plain, "not a directory").expect("write a plain file");
    let missing_dir = world.root.join("nowhere");
    let refused = [
        format!("sqlite:{}/", missing_dir.display()),
        "sqlite::memory:".to_owned(),
        format!("sqlite:{}/store.sqlite", plain.display()),
    ];
    for locator in &refused {
        for command in ["show", "eval"] {
            let out = world.thalamus(&[command, "--store", locator]);
            assert_eq!(
                out.status.code(),
                Some(74),
                "{command} --store {locator}: stdout {} stderr {}",
                text(&out.stdout),
                text(&out.stderr)
            );
        }
    }
    assert!(
        !missing_dir.exists(),
        "nothing created at {}",
        missing_dir.display()
    );

    let absent = world.root.join("absent");
    let out = world.thalamus(&[
        "show",
        "--format",
        "json",
        "--store",
        &format!("sqlite:{}/store.sqlite", absent.display()),
    ]);
    assert_eq!(out.status.code(), Some(3), "{}", text(&out.stderr));
    assert!(!absent.exists(), "show created {}", absent.display());
}

/// `sample`, two `eval`s and `show` started together on an instance with no store yet, eight
/// times: the open retry holds for all four. None exits 74; `sample` and both `eval`s exit 0,
/// `show` 0 (the OK verdict) or 3 (nothing graded yet); the store holds one sample and at most one
/// evaluation of it.
#[test]
fn sample_eval_and_show_at_once_on_a_new_store_all_open() {
    for round in 0..8 {
        let mut world = World::new(&format!("all_at_once_{round}"));
        world.depth(T_UNIX, "42");
        let at = rfc3339(T_UNIX);
        let spawn = |args: &[&str]| {
            world
                .command(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("starts")
        };
        let children = [
            ("sample", spawn(&["sample", DEPTH, "--at", &at])),
            ("eval", spawn(&["eval"])),
            ("eval", spawn(&["eval"])),
            ("show", spawn(&["show"])),
        ];
        for (command, child) in children {
            let out = child.wait_with_output().expect("ends");
            let allowed: &[i32] = if command == "show" { &[0, 3] } else { &[0] };
            assert!(
                out.status
                    .code()
                    .is_some_and(|code| allowed.contains(&code)),
                "round {round}: {command} exited {:?}: {}",
                out.status.code(),
                text(&out.stderr)
            );
        }
        let stored = world.evaluations();
        assert!(stored.len() <= 1, "round {round}: {stored:#?}");
        let samples: BTreeSet<String> = stored.iter().map(|row| row.sample_id.0.clone()).collect();
        assert_eq!(samples.len(), stored.len(), "round {round}");
    }
}

/// A `Muted` definition's stored samples are graded (the implementor's choice: only an `Active`
/// definition is sampled, so this finishes samples taken before the mute) and shown, and `show`'s
/// exit counts them.
/// Catches `Muted` dropped from `evaluate::gradable` or `evaluate::admit`.
#[test]
fn muted_definition_samples_are_graded_and_shown() {
    let mut world = World::new("muted");
    world.depth(T_UNIX, "500");
    world.sample(DEPTH, T_UNIX);
    world.edit(DEPTH, |held| {
        held.replace(
            "lifecycle: active\n",
            "lifecycle: muted\nmute_reason: queue migration\nmute_until: 2026-11-01T00:00:00Z\n",
        )
    });
    let graded = world.eval(&[]);
    assert_eq!(graded.len(), 1, "{graded:#?}");
    assert_eq!(graded[0]["verdict"], "WARN");
    let (code, shown) = world.show(&[DEPTH]);
    assert_eq!(code, Some(1), "{shown:#?}");
    let named = world.eval(&[DEPTH]);
    assert!(named.is_empty(), "graded once: {named:#?}");
}
