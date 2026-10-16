//! Adversary pass 2 against `thalamus sample` (story:m1-cli-sample, wave 20261015e): the built
//! binary against a copy of the fixture instance and the Connectors double, with recordings this
//! file writes.
//!
//! The first group are findings (red when written). The second pins behaviour the coordinator's
//! decisions after pass 1 set and the suite leaves free; each names the mutant it catches.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{Value, json};
use thalamus_model::observation::SampleId;
use thalamus_model::primitives::Timestamp;
use thalamus_store::eventlog::{EventlogStore, Locator, SAMPLE_SCHEMA};

const T1: &str = "2026-10-14T22:57:03Z";
const T1_UNIX: i64 = 1_792_018_623;
const T2: &str = "2026-10-14T23:57:03Z";
const T2_UNIX: i64 = 1_792_022_223;

const INSTANCE: &str = "fixture";

const RATIO: &str = "service.api.request_failure_ratio";
const DEPTH: &str = "service.queue.depth";

const FAILED: &str = "sum by (component) (increase(requests_failed_total[6h]))";
const REQUESTS: &str = "sum by (component) (increase(requests_total[6h]))";
const QUEUE: &str = "sum(queue_depth)";

/// A text no refusal may print.
const MARK: &str = "mark3r";

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
            .join("adv_sample_p2")
            .join(name);
        if root.exists() {
            restore_modes(&root);
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

    fn database(&self) -> PathBuf {
        self.instance().join(".thalamus/store.sqlite")
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

    fn thalamus(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .env_remove("THALAMUS_INSTANCE")
            .env_remove("THALAMUS_STORE")
            .env_remove("THALAMUS_CONNECTORS")
            .env("CONNECTORS_DOUBLE_FIXTURES", self.recordings())
            .env("CONNECTORS_DOUBLE_LOG", self.calls())
            .arg("--instance")
            .arg(self.instance())
            .arg("--connectors")
            .arg(double())
            .args(args)
            .output()
            .expect("the thalamus binary runs")
    }

    fn stored(&self) -> Vec<Value> {
        let store = EventlogStore::open(&Locator::Sqlite(self.database()), INSTANCE)
            .expect("the store opens");
        store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(SAMPLE_SCHEMA))
            .map(|record| record.body().clone())
            .collect()
    }

    fn definition(&self, id: &str) -> String {
        std::fs::read_to_string(self.instance().join(format!("definitions/{id}.yaml")))
            .expect("the definition")
    }

    fn write_definition(&self, id: &str, text: &str) {
        std::fs::write(self.instance().join(format!("definitions/{id}.yaml")), text)
            .expect("write a definition");
    }

    /// Runs one SQL statement on the instance's default store with the `sqlite3` CLI; `false`
    /// when the CLI is not installed.
    fn sql(&self, statement: &str) -> bool {
        match Command::new("sqlite3")
            .arg(self.database())
            .arg(statement)
            .output()
        {
            Err(_) => false,
            Ok(out) => {
                assert!(
                    out.status.success(),
                    "sqlite3 `{statement}`: {}",
                    text(&out.stderr)
                );
                true
            }
        }
    }
}

fn restore_modes(dir: &Path) {
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                restore_modes(&path);
            } else {
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
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

/// One Prometheus `series.query` answer at `at`, each series with its own label set.
fn answer(query: &str, at: i64, series: &[(Value, &str)]) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(labels, value)| {
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

fn standard_answers() -> Vec<Value> {
    let mut answers = Vec::new();
    for at in [T1_UNIX, T2_UNIX] {
        answers.push(answer(
            FAILED,
            at,
            &[
                (json!({"component": "checkout"}), "15"),
                (json!({"component": "search"}), "3"),
            ],
        ));
        answers.push(answer(
            REQUESTS,
            at,
            &[
                (json!({"component": "checkout"}), "556"),
                (json!({"component": "search"}), "120"),
            ],
        ));
        answers.push(answer(QUEUE, at, &[(json!({}), "42")]));
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

fn checkout() -> String {
    format!("{RATIO}{{component=\"checkout\"}}")
}

fn search() -> String {
    format!("{RATIO}{{component=\"search\"}}")
}

// ---------------------------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------------------------

/// Coordinator decision 4 and `cmd/sample.rs`'s own header: "A store refusal names the store by
/// kind ... never by its locator text." The unit's `no_store_refusal_prints_the_locator_text`
/// lists "not a path". Every case it tries is refused by the locator parser; a SQLite locator
/// that parses but that Eventlog cannot open is refused with Eventlog's own error, which
/// `refusal()` prints verbatim, and rusqlite names the file in it.
#[test]
fn a_store_eventlog_cannot_open_is_not_named_by_its_path() {
    let world = World::new("unopenable");
    let unwritable = world.root.join(MARK);
    std::fs::create_dir_all(&unwritable).expect("create a directory");
    std::fs::set_permissions(&unwritable, std::fs::Permissions::from_mode(0o500))
        .expect("make it read-only");
    let unreadable = world.root.join(format!("{MARK}.sqlite"));
    std::fs::write(&unreadable, b"").expect("create a file");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000))
        .expect("make it unreadable");

    let mut outs = Vec::new();
    for locator in [
        format!("sqlite:{}", unwritable.join("store.sqlite").display()),
        format!("sqlite:{}", unreadable.display()),
    ] {
        outs.push((
            locator.clone(),
            world.thalamus(&["--store", &locator, "sample", DEPTH, "--at", T1]),
        ));
    }
    restore_modes(&world.root);

    for (locator, out) in outs {
        let stderr = text(&out.stderr);
        assert_eq!(out.status.code(), Some(74), "{locator}: {stderr}");
        assert!(
            !stderr.contains(MARK),
            "the refusal prints the locator's path: {stderr}"
        );
    }
}

/// Design row (coordinator decision 2): "`denominator.since` under `first_sample` is the earliest
/// stored `ts` of the series by instant". The core says the generated decoder admits any text as
/// a `Timestamp` (`in_time_order`), and the store appends it. A stored `ts` that names no instant
/// but sorts before the instants by bytes (here a space for the `T`) becomes the origin: the
/// `resolve_since` refusal of a non-instant falls back to that same text, so every later sample of
/// the series carries a `since` that names no instant and an `age_s` of 0.
#[test]
fn a_stored_ts_that_names_no_instant_is_not_the_origin_of_a_series() {
    let world = World::new("non_instant");
    let out = world.thalamus(&["sample", RATIO, "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));

    let mut store =
        EventlogStore::open(&Locator::Sqlite(world.database()), INSTANCE).expect("the store opens");
    let mut forged = store
        .samples()
        .iter()
        .find(|sample| sample.series.0 == checkout())
        .expect("the checkout sample")
        .clone();
    forged.sample_id = SampleId("smp_000000000900".to_owned());
    forged.ts = Timestamp("2026-10-14 20:00:00Z".to_owned());
    store
        .append_sample(&forged)
        .expect("the store appends a sample whose ts names no instant");
    drop(store);

    let out = world.thalamus(&["sample", RATIO, "--at", T2]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let stored = world.stored();
    let at_t2: Vec<&Value> = of_series(&stored, &checkout())
        .into_iter()
        .filter(|sample| sample["ts"] == T2)
        .collect();
    assert_eq!(at_t2.len(), 1, "{stored:#?}");
    assert_eq!(
        at_t2[0]["denominator"]["since"], T1,
        "the earliest stored instant is the origin: {:#}",
        at_t2[0]
    );
    assert_eq!(at_t2[0]["denominator"]["age_s"], T2_UNIX - T1_UNIX);
}

/// Decision 4: a PostgreSQL store is named by "host and database". The peer-authenticated
/// Unix-socket locator the password refusal points an operator to
/// (`postgresql:///<db>?host=<socket dir>`) is named with an empty host, because `describe` reads
/// the host only from the authority.
#[test]
fn a_unix_socket_store_is_named_by_its_socket_directory() {
    let world = World::new("unix_socket");
    let socket = world.root.join("no-server-here");
    let locator = format!("postgresql:///thalamus?host={}", socket.display());
    let out = world.thalamus(&["--store", &locator, "sample", DEPTH, "--at", T1]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(74), "{stderr}");
    assert!(stderr.contains("database `thalamus`"), "{stderr}");
    assert!(
        stderr.contains(&socket.display().to_string()),
        "the refusal names no host: {stderr}"
    );
}

// ---------------------------------------------------------------------------------------------
// Pins of the pass-1 decisions the suite leaves free
// ---------------------------------------------------------------------------------------------

/// Decision 4 on an admitted URL. Every case of the unit's test is refused by the parser before
/// `describe` runs, so `describe` returning the URL verbatim survives the suite. A user and a
/// query parameter carry the marker here; host and database are named.
#[test]
fn an_admitted_postgres_url_is_named_by_host_and_database_only() {
    let world = World::new("admitted_url");
    let locator =
        format!("postgresql://{MARK}@127.0.0.1:1/thalamus?application_name={MARK}&user={MARK}");
    let out = world.thalamus(&["--store", &locator, "sample", DEPTH, "--at", T1]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(74), "{stderr}");
    assert!(
        stderr.contains("host `127.0.0.1:1` database `thalamus`"),
        "{stderr}"
    );
    assert!(!stderr.contains(MARK), "{stderr}");
}

/// Decision 6: "On exit 74, the ids already recorded are printed first." A SQLite trigger makes
/// the store refuse the second sample of the pass, deterministically: the first is recorded and
/// printed, the second is not, and what was printed is what was stored. Text and JSON.
#[test]
fn a_store_that_fails_mid_pass_prints_what_it_recorded_first() {
    let world = World::new("mid_pass");
    let out = world.thalamus(&["sample", DEPTH, "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let refuse_from = |count: usize| {
        format!(
            "DROP TRIGGER IF EXISTS adversary_refuses; \
             CREATE TRIGGER adversary_refuses BEFORE INSERT ON thalamus_events \
             WHEN NEW.event_name = 'thalamus.sample' AND \
             (SELECT count(*) FROM thalamus_events WHERE event_name = 'thalamus.sample') >= {count} \
             BEGIN SELECT RAISE(ABORT, 'adversary refuses this append'); END;"
        )
    };
    if !world.sql(&refuse_from(2)) {
        eprintln!("skipped: no sqlite3 CLI to install the refusing trigger");
        return;
    }

    let out = world.thalamus(&["sample", RATIO, "--at", T2, "--format", "json"]);
    assert_eq!(out.status.code(), Some(74), "stderr: {}", text(&out.stderr));
    let printed: Value = serde_json::from_slice(&out.stdout).expect("stdout is one JSON document");
    let samples = printed["samples"].as_array().expect("samples");
    assert_eq!(samples.len(), 1, "{printed:#}");
    assert_eq!(samples[0]["series"], checkout().as_str());
    let printed_json = samples[0]["sample_id"].as_str().expect("id").to_owned();

    assert!(world.sql(&refuse_from(3)));
    let out = world.thalamus(&["sample", RATIO, "--at", T1]);
    assert_eq!(out.status.code(), Some(74), "stderr: {}", text(&out.stderr));
    let lines: Vec<String> = text(&out.stdout).lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    let printed_text = lines[0].split('\t').next().expect("an id").to_owned();

    assert!(world.sql("DROP TRIGGER adversary_refuses;"));
    let stored = world.stored();
    let ratio: Vec<&str> = stored
        .iter()
        .filter(|sample| sample["signal"] == RATIO)
        .map(|sample| sample["sample_id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        ratio,
        vec![printed_json.as_str(), printed_text.as_str()],
        "{stored:#?}"
    );
}

/// Decision 2 with more than one stored sample. The unit's case stores one sample before the
/// second pass, so the first and the last stored sample are the same one and `.next()` read as
/// `.last()` survives. A third pass at T2 reads T1 as its origin, not T2.
#[test]
fn since_is_the_earliest_of_several_stored_samples() {
    let world = World::new("several");
    for at in [T1, T2, T2] {
        let out = world.thalamus(&["sample", RATIO, "--at", at]);
        assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    }
    let stored = world.stored();
    for series in [checkout(), search()] {
        let samples = of_series(&stored, &series);
        assert_eq!(samples.len(), 3, "{series}");
        let third = samples[2];
        assert_eq!(third["ts"], T2);
        assert_eq!(third["denominator"]["since"], T1, "{third:#}");
        assert_eq!(third["denominator"]["age_s"], T2_UNIX - T1_UNIX);
    }
}

/// Decision 1 over two definitions in one `--all` pass: a ratio whose numerator answers three
/// series for `checkout` (they differ only in `instance`) and a gauge answering two. One sample
/// per series; the merged reason names the collision and the read count, and carries the
/// adapter's reason once; the untouched series stays `ok`.
#[test]
fn collisions_in_two_definitions_of_one_pass_each_record_one_sample() {
    let world = World::new("two_collisions");
    world.record_invokes(&[
        answer(
            FAILED,
            T1_UNIX,
            &[
                (json!({"component": "checkout", "instance": "a"}), "5"),
                (json!({"component": "checkout", "instance": "b"}), "6"),
                (json!({"component": "checkout", "instance": "c"}), "7"),
                (json!({"component": "search"}), "3"),
            ],
        ),
        answer(
            REQUESTS,
            T1_UNIX,
            &[
                (json!({"component": "checkout"}), "556"),
                (json!({"component": "search"}), "120"),
            ],
        ),
        answer(
            QUEUE,
            T1_UNIX,
            &[
                (json!({"instance": "a"}), "1"),
                (json!({"instance": "b"}), "2"),
            ],
        ),
    ]);
    let out = world.thalamus(&["sample", "--all", "--once", "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    let stored = world.stored();

    let reason = |series: &str, reads: usize, adapter: &str| {
        let samples = of_series(&stored, series);
        assert_eq!(samples.len(), 1, "{series}: {samples:#?}");
        let sample = samples[0];
        assert_eq!(sample["reading"]["quality"], "unknown", "{sample:#}");
        assert!(sample.get("denominator").is_none(), "{sample:#}");
        let reason = sample["reading"]["value"]["reason"]
            .as_str()
            .expect("a reason")
            .to_owned();
        assert!(
            reason.starts_with(&format!(
                "{reads} reads of one pass fall on series {series}"
            )),
            "{reason}"
        );
        assert_eq!(reason.matches(adapter).count(), 1, "{reason}");
    };
    reason(
        &checkout(),
        3,
        "3 numerator and 1 denominator series collide on",
    );
    reason(DEPTH, 2, "2 result series collide on");

    let search = of_series(&stored, &search());
    assert_eq!(search.len(), 1);
    assert_eq!(search[0]["reading"]["quality"], "ok", "{:#}", search[0]);
}

/// Decision 3: thresholds are checked only for `Active` and `Muted` definitions. A `Proposed`
/// draft naming a known rule with no CRIT line loads; the same file made `Active` is refused.
/// Catches the threshold check run for every lifecycle state.
#[test]
fn a_proposed_draft_with_incomplete_lines_loads() {
    let world = World::new("draft_lines");
    let id = "service.queue.draft_depth";
    let draft = world
        .definition(DEPTH)
        .replace("id: service.queue.depth", &format!("id: {id}"))
        .replace("lifecycle: active", "lifecycle: proposed")
        .replace("  crit: { abs: \"1000\" }\n", "");
    assert!(!draft.contains("crit"), "{draft}");
    world.write_definition(id, &draft);
    let out = world.thalamus(&["sample", DEPTH, "--at", T1]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));

    world.write_definition(
        id,
        &draft.replace("lifecycle: proposed", "lifecycle: active"),
    );
    let out = world.thalamus(&["sample", DEPTH, "--at", T1]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(65), "{stderr}");
    assert!(stderr.contains("draft_depth.yaml"), "{stderr}");
}
