//! The `http_probe` adapter, read through the extractor registry against the recorded Connectors
//! double (`src/bin/connectors-double.rs`).
//!
//! A probe samples the availability of one catalog read operation: the definition names the
//! operation (`query.operation`) and its input (`query.input`). Each test copies the recorded
//! answers of `tests/fixtures/http_probe/common`, then of its scenarios, into a fresh directory
//! under the build's own temporary directory and links `connectors` there to the double. The
//! refusal recordings carry the envelope Connectors v0.45.0 writes for each target answer
//! (`apps/connectors/src/local.rs`, `owner_failure`).

use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{CompleteReading, QueryDigest, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Read, window_ending};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry::{self, NOT_IMPLEMENTED, SourceKind};

const END: &str = "2026-10-14T22:57:03Z";
/// SHA-256 of the canonical JSON of the query map,
/// `{"input":"{\"id\":\"42\"}","operation":"project.get"}`.
const QUERY_SHA256: &str = "a3628c274c6f03606105a2ac9c68277c9e51dc91a00f7b8a6893a3a90c98ec6c";

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("http_probe")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/http_probe");
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

    /// The argv of every call the double received whose leading words are `words`.
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

fn definition(query: &[(&str, &str)]) -> SignalDefinitionData {
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

/// The probe of `project.get` for project 42.
fn project_probe() -> SignalDefinitionData {
    definition(&[("operation", "project.get"), ("input", r#"{"id":"42"}"#)])
}

fn end() -> Timestamp {
    Timestamp(END.into())
}

fn read(world: &World, definition: &SignalDefinitionData) -> Vec<Read> {
    registry::read(
        &world.connectors(),
        definition,
        &end(),
        &RunId("run_test".into()),
    )
}

/// The one read of a probe pass.
fn one(reads: Vec<Read>) -> Read {
    let [read] = <[Read; 1]>::try_from(reads).unwrap_or_else(|reads| {
        panic!("a probe reads one series, got {reads:#?}");
    });
    read
}

fn value(v: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(v.into()),
    })
}

fn reason(reading: &Reading) -> &str {
    match reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

/// The provenance every probe read carries, whatever it read.
fn assert_provenance(read: &Read) {
    assert_eq!(read.adapter, AdapterId("forge".into()));
    assert_eq!(read.connection, ConnectionId("forge-main".into()));
    assert_eq!(read.query_sha256, QueryDigest(QUERY_SHA256.into()));
    assert_eq!(
        read.window,
        window_ending(&Duration("PT5M".into()), &end()).expect("the window is placeable")
    );
    assert!(read.dims.is_empty(), "a probe derives no dims: {read:?}");
    assert_eq!(read.denominator, None);
    assert_eq!(read.retention_start, None);
}

fn assert_unknown_never_down(read: &Read, wanted: &[&str]) {
    let why = reason(&read.reading);
    for word in wanted {
        assert!(why.contains(word), "{word:?} missing from {why:?}");
    }
    assert_eq!(read.raw, None, "an unknown read has no producer text");
    assert_provenance(read);
}

#[test]
fn probe_ok_answer_is_up_1() {
    let w = World::new("probe_ok_answer_is_up_1", &["ok"]);
    let read = one(read(&w, &project_probe()));

    assert_eq!(read.reading, value("1"), "{read:#?}");
    assert_eq!(
        read.raw.as_deref(),
        Some("200"),
        "raw is the answered status"
    );
    assert_provenance(&read);

    let invokes = w.calls(&["operations", "invoke"]);
    assert_eq!(invokes.len(), 1, "{invokes:?}");
    let argv = &invokes[0];
    let flag = |name: &str| {
        let at = argv.iter().position(|a| a == name).expect(name);
        argv[at + 1].clone()
    };
    assert_eq!(flag("--operation"), "project.get");
    assert_eq!(
        serde_json::from_str::<Value>(&flag("--input-json")).expect("input is JSON"),
        serde_json::json!({"id": "42"}),
        "the definition's input is sent as written"
    );
}

#[test]
fn probe_provider_unavailable_is_up_0_with_reason() {
    // The target refused to serve the read: a 404 or 410 (`service_failure` with service code
    // `not_found`, dispatched), or a sent request whose deadline passed (`timeout` at
    // `dispatch`, the provider's origin).
    for (scenario, code) in [
        ("provider-not-found", "not_found"),
        ("provider-timeout", "timeout"),
    ] {
        let w = World::new(
            &format!("probe_provider_unavailable_is_up_0_with_reason-{scenario}"),
            &[scenario],
        );
        let read = one(read(&w, &project_probe()));
        assert_eq!(read.reading, value("0"), "{scenario}: {read:#?}");
        let why = read.raw.as_deref().unwrap_or_default();
        assert!(
            why.contains(code) && why.contains("dispatch"),
            "{scenario}: the reason names Connectors' code and stage: {why:?}"
        );
        assert_provenance(&read);
        assert_eq!(
            w.calls(&["operations", "invoke"]).len(),
            1,
            "{scenario}: a dispatched refusal is not retried"
        );
    }
}

#[test]
fn probe_admission_refusal_is_unknown_never_down() {
    // The connection, not the target, refused: Connectors never sent the request.
    for (scenario, code) in [
        ("admission-not-found", "not_found"),
        ("admission-not-granted", "not_granted"),
    ] {
        let w = World::new(
            &format!("probe_admission_refusal_is_unknown_never_down-{scenario}"),
            &[scenario],
        );
        let read = one(read(&w, &project_probe()));
        assert_unknown_never_down(&read, &[code, "admission"]);
    }
}

#[test]
fn probe_rate_limited_is_unknown() {
    let w = World::new("probe_rate_limited_is_unknown", &["rate-limited"]);
    let read = one(read(&w, &project_probe()));
    assert_unknown_never_down(&read, &["rate_limited", "dispatch"]);
}

#[test]
fn probe_refusal_of_the_credential_is_unknown() {
    // A 403 is the target answering: it is up, and the probe's credential is refused. That is an
    // instrument failure, never a 0.
    let w = World::new(
        "probe_refusal_of_the_credential_is_unknown",
        &["provider-forbidden"],
    );
    let read = one(read(&w, &project_probe()));
    assert_unknown_never_down(&read, &["forbidden", "dispatch"]);
}

#[test]
fn probe_5xx_on_the_connectors_wire_is_unknown() {
    // Connectors v0.45.0 projects a provider 5xx onto its own `unavailable` at `readiness`
    // (`crates/connectors-host/src/local/runtime.rs`, "a 5xx reads as unavailability"), the
    // envelope of the host's own unavailability. The probe cannot tell the two apart, so it
    // reads neither as 0.
    let w = World::new(
        "probe_5xx_on_the_connectors_wire_is_unknown",
        &["provider-5xx"],
    );
    let read = one(read(&w, &project_probe()));
    assert_unknown_never_down(&read, &["unavailable", "readiness"]);
}

#[test]
fn probe_of_an_operation_that_is_not_a_read_runs_nothing() {
    for operation in [
        "project.create",
        "project.delete",
        "project",
        "",
        "project.get ",
    ] {
        let w = World::new(
            &format!(
                "probe_of_an_operation_that_is_not_a_read_runs_nothing-{}",
                operation.replace(['.', ' '], "_")
            ),
            &["ok"],
        );
        let definition = definition(&[("operation", operation), ("input", r#"{"id":"42"}"#)]);
        let read = one(read(&w, &definition));
        let why = reason(&read.reading);
        assert!(why.contains("not a read"), "{operation:?}: {why}");
        assert!(
            w.calls(&["operations"]).is_empty(),
            "{operation:?}: no operation is described or invoked"
        );
    }
}

#[test]
fn probe_invokes_only_a_catalog_read_its_description_names() {
    // Connectors v0.45.0 describes a catalog read with profile `generic-http` and a catalog write
    // with `mutation` (`adapters/catalog/src/lib.rs`, `declare`); every other adapter names its
    // own operation profile. Only `generic-http` is invoked.
    for (scenario, word) in [("catalog-write", "mutation"), ("no-profile", "profile")] {
        let w = World::new(
            &format!("probe_invokes_only_a_catalog_read_its_description_names-{scenario}"),
            &["ok", scenario],
        );
        let read = one(read(&w, &project_probe()));
        let why = reason(&read.reading);
        assert!(
            why.contains(word) && why.contains("generic-http"),
            "{scenario}: {why}"
        );
        assert_eq!(read.raw, None);
        assert_eq!(
            w.calls(&["operations", "describe"]).len(),
            1,
            "{scenario}: described once"
        );
        assert!(
            w.calls(&["operations", "invoke"]).is_empty(),
            "{scenario}: never invoked"
        );
    }
}

#[test]
fn probe_definition_faults_read_unknown_without_invoking() {
    let cases: [(&str, SignalDefinitionData, &str); 5] = [
        (
            "no-operation",
            definition(&[("input", r#"{"id":"42"}"#)]),
            "operation",
        ),
        (
            "input-not-json",
            definition(&[("operation", "project.get"), ("input", "{id: 42")]),
            "input",
        ),
        (
            "input-not-object",
            definition(&[("operation", "project.get"), ("input", "[42]")]),
            "input",
        ),
        (
            "unit",
            SignalDefinitionData {
                unit: Some(Unit("s".into())),
                ..project_probe()
            },
            "unit",
        ),
        (
            "extractor",
            SignalDefinitionData {
                extractor: ExtractorId("http_probe.latency".into()),
                ..project_probe()
            },
            "http_probe.latency",
        ),
    ];
    for (name, definition, word) in cases {
        let w = World::new(
            &format!("probe_definition_faults_read_unknown_without_invoking-{name}"),
            &["ok"],
        );
        let read = one(read(&w, &definition));
        let why = reason(&read.reading);
        assert!(why.contains(word), "{name}: {word:?} missing from {why:?}");
        assert!(
            w.calls(&["operations", "invoke"]).is_empty(),
            "{name}: nothing is invoked"
        );
    }
}

#[test]
fn probe_without_input_sends_an_empty_object() {
    let w = World::new("probe_without_input_sends_an_empty_object", &["ok"]);
    // The recorded answer matches only input `{"id":"42"}`; another input finds no answer and
    // reads unknown, so this checks only what was sent.
    let _ = read(&w, &definition(&[("operation", "project.get")]));
    let invokes = w.calls(&["operations", "invoke"]);
    assert_eq!(invokes.len(), 1, "{invokes:?}");
    let argv = &invokes[0];
    let at = argv
        .iter()
        .position(|a| a == "--input-json")
        .expect("input");
    assert_eq!(argv[at + 1], "{}");
}

#[test]
fn probe_declared_dimensions_stay_unset() {
    let w = World::new("probe_declared_dimensions_stay_unset", &["ok"]);
    let definition = SignalDefinitionData {
        dimensions: Some(vec![DimensionKey("service".into())]),
        ..project_probe()
    };
    let read = one(read(&w, &definition));
    assert_eq!(read.reading, value("1"));
    assert!(read.dims.is_empty(), "no dim is derived: {:?}", read.dims);
}

#[test]
fn probe_is_registered_for_its_prefix() {
    let adapter = SourceKind::HttpProbe
        .adapter()
        .expect("the http probe adapter is registered");
    assert!(
        adapter.read_operations().is_empty(),
        "a probe declares no fixed operation"
    );
    let probe = project_probe();
    assert_eq!(adapter.operations_for(&probe), vec!["project.get"]);
    let reads = read(
        &World::new("probe_is_registered_for_its_prefix", &["ok"]),
        &probe,
    );
    assert!(
        reads
            .iter()
            .all(|r| !matches!(&r.reading, Reading::Unknown(u) if u.reason == NOT_IMPLEMENTED)),
        "{reads:?}"
    );
}
