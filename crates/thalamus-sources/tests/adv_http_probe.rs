//! Adversary cases for the `http_probe` adapter, read through the registry against the recorded
//! Connectors double. Recordings live in `tests/fixtures/adv_http_probe`; each test copies
//! `common`, then its scenarios, into its own directory under the build's temporary directory.

use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::observation::{CompleteReading, QueryDigest, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry::{self, SourceKind};

const END: &str = "2026-10-14T22:57:03Z";

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_http_probe")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adv_http_probe");
        for scenario in std::iter::once(&"common").chain(scenarios) {
            for entry in std::fs::read_dir(fixtures.join(scenario)).expect("scenario exists") {
                let path = entry.expect("fixture entry").path();
                std::fs::copy(&path, dir.join(path.file_name().expect("file name")))
                    .expect("copy a recording");
            }
        }
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_connectors-double"),
            dir.join("connectors"),
        )
        .expect("link the double");
        World { dir }
    }

    fn connectors(&self) -> Connectors {
        Connectors::new(self.dir.join("connectors"))
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
}

fn definition_on(adapter: &str, connection: &str, query: &[(&str, &str)]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.probe.up".into()),
        title: "A probe".into(),
        family: Family("platform".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("1".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId(adapter.into()),
            connection: ConnectionId(connection.into()),
        },
        extractor: ExtractorId("http_probe.up".into()),
        query: query
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
            fixed: None,
            rolling_n: None,
            mapping: None,
        },
        persistence: None,
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

fn forge(query: &[(&str, &str)]) -> SignalDefinitionData {
    definition_on("forge", "forge-main", query)
}

fn project_probe() -> SignalDefinitionData {
    forge(&[("operation", "project.get"), ("input", r#"{"id":"42"}"#)])
}

fn read(world: &World, definition: &SignalDefinitionData) -> Read {
    let reads = registry::read(
        &world.connectors(),
        definition,
        &Timestamp(END.into()),
        &RunId("run_test".into()),
    );
    let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
        panic!("a probe reads one series, got {reads:#?}");
    });
    read
}

fn is_value(reading: &Reading, v: &str) -> bool {
    *reading
        == Reading::Ok(CompleteReading {
            value: Decimal(v.into()),
        })
}

fn assert_unknown(read: &Read, context: &str) {
    assert!(
        matches!(read.reading, Reading::Unknown(_)),
        "{context}: expected unknown, got {:?} (raw {:?})",
        read.reading,
        read.raw
    );
    assert_eq!(read.raw, None, "{context}: an unknown read has no raw text");
}

/// A probe that can read 0 on an adapter must be able to read 1 on it. Connectors v0.45.0 answers
/// a non-catalog read (the kubernetes adapter's `resources.get`, the sql adapter's `schema.list`)
/// with the adapter's own document, which carries no numeric `status`; its refusals of a 404
/// (`upstream_json`, `.answered()`) or a statement timeout are the same `service_failure` /
/// `not_found` and `timeout` at `dispatch` as the catalog's, and a kubernetes 404 means the
/// resource is absent, not that the API is down. So the probe invokes only catalog operations
/// (coordinator decision F1, design § 11 D21): on a non-catalog operation both the served and
/// the refused answer read unknown, and the operation is never invoked.
///
/// Changed from the adversary's original, which asserted only "0 reachable implies 1
/// reachable", as decision F1 requires.
#[test]
fn adv_a_probe_that_can_read_0_can_also_read_1() {
    let probe = definition_on(
        "cluster",
        "cluster-main",
        &[
            ("operation", "resources.get"),
            (
                "input",
                r#"{"namespace":"prod","kind":"Deployment","name":"api"}"#,
            ),
        ],
    );
    for scenario in ["cluster-ok", "cluster-not-found"] {
        let w = World::new(
            &format!("adv_a_probe_that_can_read_0_can_also_read_1-{scenario}"),
            &[scenario],
        );
        let r = read(&w, &probe);
        assert!(
            !is_value(&r.reading, "0") && !is_value(&r.reading, "1"),
            "{scenario}: a non-catalog operation read {:?}",
            r.reading
        );
        assert_unknown(&r, scenario);
        assert!(
            w.calls(&["operations", "invoke"]).is_empty(),
            "{scenario}: a non-catalog operation is never invoked: {:?}",
            w.calls(&[])
        );
    }
}

/// Ids the guard must refuse before anything runs: a non-read verb in last place, no resource,
/// case, whitespace and control characters, and lookalike letters (Cyrillic `о` in the resource,
/// Cyrillic `е` in the verb, a zero-width space).
#[test]
fn adv_guard_refuses_lookalikes_and_runs_nothing() {
    for (i, operation) in [
        "x.get.delete",
        "get",
        "list",
        ".get",
        ".list",
        "project.GET",
        "project.Get",
        "project.get\n",
        "project.get\t",
        " project.get",
        "project.get\u{200b}",
        "pr\u{43e}ject.get",
        "project.g\u{435}t",
        "project.get/../delete",
        "project.get;delete",
        "project.get%2edelete",
        "project.list.create",
    ]
    .into_iter()
    .enumerate()
    {
        let w = World::new(
            &format!("adv_guard_refuses_lookalikes_and_runs_nothing-{i}"),
            &[],
        );
        let probe = forge(&[("operation", operation)]);
        let adapter = SourceKind::HttpProbe.adapter().expect("registered");
        assert!(
            adapter.operations_for(&probe).is_empty(),
            "{operation:?}: the client admits nothing"
        );
        let r = read(&w, &probe);
        assert_unknown(&r, operation);
        assert!(
            w.calls(&["operations"]).is_empty(),
            "{operation:?}: nothing is described or invoked"
        );
    }
}

/// Refusals that are not the probe's own dispatched invoke, or not at `dispatch`, never read 0.
#[test]
fn adv_other_calls_and_stages_never_read_0() {
    for scenario in [
        "describe-refused-not-found-dispatch",
        "describe-refused-timeout-dispatch",
        "list-refused-timeout-dispatch",
        "admission-not-found-with-service-code",
        "service-not-found-at-readiness",
        "ok-list-answer",
    ] {
        let w = World::new(
            &format!("adv_other_calls_and_stages_never_read_0-{scenario}"),
            &[scenario],
        );
        let r = read(&w, &project_probe());
        assert_unknown(&r, scenario);
    }
}

/// Input that is not an object, nested past the parser's limit, or too large for one argument
/// reads unknown; nothing panics and nothing reads 0.
#[test]
fn adv_hostile_input_reads_unknown_without_panicking() {
    let deep_array = format!("{}{}", "[".repeat(20_000), "]".repeat(20_000));
    let deep_object = format!("{}1{}", r#"{"a":"#.repeat(20_000), "}".repeat(20_000));
    let huge = format!(r#"{{"id":"{}"}}"#, "x".repeat(400_000));
    let cases: Vec<(&str, String)> = vec![
        ("null", "null".into()),
        ("string", r#""x""#.into()),
        ("number", "42".into()),
        ("bool", "true".into()),
        ("empty", String::new()),
        ("deep-array", deep_array),
        ("deep-object", deep_object),
        ("huge", huge),
    ];
    for (name, input) in cases {
        let w = World::new(
            &format!("adv_hostile_input_reads_unknown_without_panicking-{name}"),
            &[],
        );
        let r = read(
            &w,
            &forge(&[("operation", "project.get"), ("input", &input)]),
        );
        assert_unknown(&r, name);
    }
}

/// A window the adapter cannot place reads unknown and runs nothing.
#[test]
fn adv_unplaceable_window_runs_nothing() {
    let w = World::new("adv_unplaceable_window_runs_nothing", &[]);
    let probe = SignalDefinitionData {
        window: Duration("P1M".into()),
        ..project_probe()
    };
    let r = read(&w, &probe);
    assert_unknown(&r, "P1M");
    assert!(w.calls(&["operations"]).is_empty(), "{:?}", w.calls(&[]));
}

/// The default `operations_for` grants every other adapter exactly what it declared, whatever
/// operation the definition names.
#[test]
fn adv_operations_for_cannot_widen_other_adapters() {
    for kind in SourceKind::ALL {
        if kind == SourceKind::HttpProbe {
            continue;
        }
        let Some(adapter) = kind.adapter() else {
            continue;
        };
        let mut definition = forge(&[("operation", "pods.exec"), ("query", "up")]);
        definition.extractor = ExtractorId(format!("{}x", kind.prefix()));
        let declared: Vec<&str> = adapter.read_operations().to_vec();
        assert_eq!(adapter.operations_for(&definition), declared, "{kind:?}");
        assert!(
            !adapter.operations_for(&definition).contains(&"pods.exec"),
            "{kind:?}"
        );
    }
}

/// The digest does not depend on the order the query was written in.
#[test]
fn adv_query_digest_ignores_insertion_order() {
    let one = forge(&[("operation", "project.get"), ("input", r#"{"id":"42"}"#)]);
    let two = forge(&[("input", r#"{"id":"42"}"#), ("operation", "project.get")]);
    let w = World::new("adv_query_digest_ignores_insertion_order", &[]);
    let (a, b) = (read(&w, &one), read(&w, &two));
    let expected =
        QueryDigest("a3628c274c6f03606105a2ac9c68277c9e51dc91a00f7b8a6893a3a90c98ec6c".into());
    assert_eq!(a.query_sha256, expected);
    assert_eq!(b.query_sha256, expected);
}
