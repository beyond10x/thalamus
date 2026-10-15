//! Adversary pass 2 on the `http_probe` adapter: the catalog-read check on the operation's
//! description (`operations describe` names profile `generic-http`) and the pin of the invoke to
//! the very description checked. Each test writes its recordings for the Connectors double
//! (`src/bin/connectors-double.rs`) into its own directory under the build's temporary directory.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";
const DESCRIBE: &str = "operations+describe+a=forge+o=project.get.json";
const INVOKE: &str = "operations+invoke+a=forge+c=forge-main+o=project.get.json";

struct World {
    dir: PathBuf,
}

impl World {
    /// A directory holding the `forge-main` connection listing, the recordings `files`, and the
    /// `connectors` link to the double.
    fn new(name: &str, files: &[(&str, Value)]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv2_http_probe")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let listing = json!({"stdout": {"ok": true, "result": {"connections": [
            {"adapter": "forge", "connection": "forge-main", "state": "ready", "revision": "conn-rev-1"}
        ]}}});
        let listing = ("connections+list+a=forge.json", listing);
        for (file, recording) in std::iter::once(&listing).chain(files) {
            std::fs::write(dir.join(file), recording.to_string()).expect("write a recording");
        }
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_connectors-double"),
            dir.join("connectors"),
        )
        .expect("link the double");
        World { dir }
    }

    fn calls(&self, words: &[&str]) -> Vec<Vec<String>> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .map(|call| {
                call["argv"]
                    .as_array()
                    .expect("argv is a list")
                    .iter()
                    .map(|a| a.as_str().expect("argv element").to_string())
                    .collect::<Vec<_>>()
            })
            .filter(|argv| argv.iter().take(words.len()).eq(words.iter()))
            .collect()
    }

    fn read(&self) -> Read {
        let reads = registry::read(
            &Connectors::new(self.dir.join("connectors")),
            &probe(),
            &Timestamp(END.into()),
            &RunId("run_test".into()),
        );
        let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
            panic!("a probe reads one series, got {reads:#?}");
        });
        read
    }
}

fn probe() -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.forge.up".into()),
        title: "The forge answers a project read".into(),
        family: Family("platform".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("1".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("forge".into()),
            connection: ConnectionId("forge-main".into()),
        },
        extractor: ExtractorId("http_probe.up".into()),
        query: [("operation", "project.get"), ("input", r#"{"id":"42"}"#)]
            .iter()
            .map(|(k, v)| (k.to_string(), QueryText(v.to_string())))
            .collect(),
        window: Duration("PT5M".into()),
        cadence: Duration("PT5M".into()),
        history: History::Store,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId("threshold".into()),
            baselines: Vec::new(),
            warn: None,
            crit: None,
            min_trend_window: None,
        },
        objective: None,
        owner: Some(Owner("team:platform".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-10-01T00:00:00Z".into()),
        revised_by: Author("tester".into()),
        mute_reason: None,
        mute_until: None,
    }
}

/// The `operations describe` answer Connectors v0.45.0 writes (`apps/connectors/src/local/
/// operations.rs`, `describe`), its `operation.profile` set to `profile` (absent for `None`).
fn described(revision: &str, schema: &str, profile: Option<Value>) -> Value {
    let mut operation = json!({
        "id": "project.get",
        "description": "Read one project",
        "contract": "operations/v1alpha1"
    });
    if let Some(profile) = profile {
        operation["profile"] = profile;
    }
    json!({"ok": true, "result": {
        "adapter": "forge",
        "revision": revision,
        "schema": schema,
        "operation": operation,
        "source": "cached",
        "stale": true
    }})
}

/// The catalog's served answer: status 200.
fn served() -> Value {
    json!({"ok": true, "result": {
        "adapter": "forge",
        "operation": "project.get",
        "revision": "rev-read",
        "result": "{\"status\":200,\"body\":{\"id\":42},\"provenance\":{\"instance\":\"forge-main\",\"resource\":\"/projects/42\",\"observed_at_unix_ms\":1792018623000,\"source_revision\":null}}"
    }})
}

fn flag(argv: &[String], name: &str) -> String {
    let at = argv.iter().position(|a| a == name).expect(name);
    argv[at + 1].clone()
}

fn is_value(reading: &Reading, v: &str) -> bool {
    *reading
        == Reading::Ok(CompleteReading {
            value: Decimal(v.into()),
        })
}

/// A profile is an exact identifier. Every value that is not exactly `generic-http`, wherever it
/// differs (case, whitespace, a suffix such as the catalog contract's paged `generic-http-page`,
/// a lookalike separator, a non-text value, the right text one level up) is no catalog read: the
/// read is unknown with no `raw`, the operation is described once and never invoked, though an
/// invoke would have been served.
#[test]
fn adv2_only_the_exact_catalog_read_profile_is_invoked() {
    let mut cases: Vec<(String, Value)> = [
        Some(json!("")),
        Some(json!("Generic-HTTP")),
        Some(json!("GENERIC-HTTP")),
        Some(json!("generic-http ")),
        Some(json!(" generic-http")),
        Some(json!("generic-http\n")),
        Some(json!("generic-http-page")),
        Some(json!("generic_http")),
        Some(json!("generic-http\u{0}")),
        Some(json!("generic\u{2011}http")),
        Some(json!(null)),
        Some(json!(true)),
        Some(json!(7)),
        Some(json!(["generic-http"])),
        Some(json!({"id": "generic-http"})),
    ]
    .into_iter()
    .map(|profile| {
        (
            format!("{profile:?}"),
            described("rev-read", "schema-read", profile),
        )
    })
    .collect();
    let mut top_level = described("rev-read", "schema-read", None);
    top_level["result"]["profile"] = json!("generic-http");
    cases.push(("profile beside the operation".into(), top_level));

    for (i, (case, describe)) in cases.into_iter().enumerate() {
        let w = World::new(
            &format!("adv2_only_the_exact_catalog_read_profile_is_invoked-{i}"),
            &[
                (DESCRIBE, json!({"stdout": describe})),
                (INVOKE, json!({"stdout": served()})),
            ],
        );
        let r = w.read();
        assert!(
            matches!(r.reading, Reading::Unknown(_)),
            "{case}: expected unknown, got {:?} (raw {:?})",
            r.reading,
            r.raw
        );
        assert_eq!(r.raw, None, "{case}: an unknown read has no raw text");
        assert_eq!(
            w.calls(&["operations", "describe"]).len(),
            1,
            "{case}: described once"
        );
        assert!(
            w.calls(&["operations", "invoke"]).is_empty(),
            "{case}: never invoked: {:?}",
            w.calls(&[])
        );
    }
}

/// The invoke carries the schema and revision of the description the check accepted, never those
/// of a later one. Connectors answers a first describe `generic-http` and a second `mutation`
/// under another revision; a client that described again after the check would pin the write and
/// be served.
#[test]
fn adv2_the_invoke_pins_the_description_it_checked() {
    let w = World::new(
        "adv2_the_invoke_pins_the_description_it_checked",
        &[
            (
                DESCRIBE,
                json!([
                    {"call": 1, "stdout": described("rev-read", "schema-read", Some(json!("generic-http")))},
                    {"call": 2, "stdout": described("rev-write", "schema-write", Some(json!("mutation")))}
                ]),
            ),
            (INVOKE, json!({"stdout": served()})),
        ],
    );
    let r = w.read();
    assert!(is_value(&r.reading, "1"), "{r:#?}");
    assert_eq!(
        w.calls(&["operations", "describe"]).len(),
        1,
        "described once: {:?}",
        w.calls(&[])
    );
    let invokes = w.calls(&["operations", "invoke"]);
    assert_eq!(invokes.len(), 1, "{invokes:?}");
    assert_eq!(flag(&invokes[0], "--revision"), "rev-read");
    assert_eq!(flag(&invokes[0], "--schema"), "schema-read");
}

/// A retried invoke checks the description it describes again. The first description is a
/// catalog read and its invoke is refused at `admission` with `timeout`, which the client retries
/// once; the second description names `mutation` under a new revision. The retry must not invoke:
/// the read is unknown, and the only invoke is the first one, pinned to the read's revision.
#[test]
fn adv2_a_retry_checks_the_description_it_describes_again() {
    let admission_timeout = json!({"ok": false, "error": {"code": "failure", "message": "deadline",
        "data": {"kind": "operational", "code": "timeout", "stage": "admission", "next_action": "retry"}}});
    let w = World::new(
        "adv2_a_retry_checks_the_description_it_describes_again",
        &[
            (
                DESCRIBE,
                json!([
                    {"call": 1, "stdout": described("rev-read", "schema-read", Some(json!("generic-http")))},
                    {"call": 2, "stdout": described("rev-write", "schema-write", Some(json!("mutation")))}
                ]),
            ),
            (
                INVOKE,
                json!([
                    {"call": 1, "stderr": admission_timeout},
                    {"call": 2, "stdout": served()}
                ]),
            ),
        ],
    );
    let r = w.read();
    assert!(
        matches!(r.reading, Reading::Unknown(_)),
        "expected unknown, got {:?} (raw {:?}); calls {:?}",
        r.reading,
        r.raw,
        w.calls(&[])
    );
    assert_eq!(r.raw, None);
    assert_eq!(w.calls(&["operations", "describe"]).len(), 2);
    let invokes = w.calls(&["operations", "invoke"]);
    assert_eq!(invokes.len(), 1, "the retry is never invoked: {invokes:?}");
    assert_eq!(flag(&invokes[0], "--revision"), "rev-read");
}
