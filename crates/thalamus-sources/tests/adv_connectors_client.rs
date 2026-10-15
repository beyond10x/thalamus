//! Adversary cases for `story:m1-connectors-client`: the client and the recorded double held to
//! what the real `connectors` CLI does.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Adapter, Read, query_digest, unknown};
use thalamus_sources::connectors::{Connectors, ConnectorsError, ReadClient};
use thalamus_sources::registry;

const QUERY: &str = "sum(rate(requests_failed_total[5m])) / sum(rate(requests_total[5m]))";

/// Recordings from `common`, then from each scenario, in a fresh directory with `connectors`
/// linked to the double.
fn world(name: &str, scenarios: &[&str]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_connectors_client")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/connectors");
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
    dir
}

fn argvs(dir: &Path) -> Vec<Vec<String>> {
    std::fs::read_to_string(dir.join("calls.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).expect("one JSON line")["argv"]
                .as_array()
                .expect("argv")
                .iter()
                .map(|a| a.as_str().expect("argv element").to_string())
                .collect()
        })
        .collect()
}

/// Runs the double directly with `args`, reading recordings from `dir`.
fn double(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_connectors-double"))
        .args(args)
        .env("CONNECTORS_DOUBLE_FIXTURES", dir)
        .env("CONNECTORS_DOUBLE_LOG", dir.join("calls.jsonl"))
        .output()
        .expect("run the double")
}

/// Whether the double wrote a success envelope on stdout.
fn answered_ok(out: &Output) -> bool {
    serde_json::from_slice::<Value>(&out.stdout)
        .map(|doc| doc["ok"] == Value::Bool(true))
        .unwrap_or(false)
}

fn source() -> Source {
    Source {
        adapter: AdapterId("metrics".into()),
        connection: ConnectionId("metrics-main".into()),
    }
}

fn input() -> Value {
    json!({"query": QUERY})
}

fn definition() -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.request_failure_ratio".into()),
        title: "Share of requests that failed".into(),
        family: Family("platform".into()),
        kind: SignalKind::Ratio,
        unit: Some(Unit("1".into())),
        dimensions: Some(vec![DimensionKey("cluster".into())]),
        source: source(),
        extractor: ExtractorId("promql.value".into()),
        query: BTreeMap::from([("value".to_string(), QueryText(QUERY.into()))]),
        window: Duration("PT6H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Source,
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

/// Reads `series.query` and answers one read per returned series, as an adapter would.
struct Probe;

impl Adapter for Probe {
    fn read_operations(&self) -> &[&str] {
        &["series.query"]
    }

    fn read(
        &self,
        client: &ReadClient<'_>,
        definition: &SignalDefinitionData,
        end: &Timestamp,
        _run_id: &RunId,
    ) -> Result<Vec<Read>, ConnectorsError> {
        let answer = client.invoke(&definition.source, "series.query", &input())?;
        let series = answer["series"].as_array().cloned().unwrap_or_default();
        Ok(series
            .iter()
            .map(|s| {
                let text = s["value"].as_str().unwrap_or_default().to_string();
                let mut read = unknown(definition, end, "replaced below");
                read.reading = Reading::Ok(CompleteReading {
                    value: Decimal(text.clone()),
                });
                read.raw = Some(text);
                read
            })
            .collect())
    }
}

/// `connections list` pages (default limit 100, `next_cursor` on every page but the last). A
/// connection on the second page exists and is ready; the read must find it, not answer
/// `NoConnection`.
#[test]
fn adv_connection_on_a_later_listing_page_is_found() {
    let dir = world(
        "adv_connection_on_a_later_listing_page_is_found",
        &["adv-paged"],
    );
    let answer = Connectors::new(dir.join("connectors"))
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input());
    assert!(
        answer.is_ok(),
        "metrics-main is listed on page 2 (cursor c2): {answer:?}; calls {:?}",
        argvs(&dir)
    );
    let lists: Vec<Vec<String>> = argvs(&dir)
        .into_iter()
        .filter(|a| a.first().map(String::as_str) == Some("connections"))
        .collect();
    assert!(
        lists.iter().any(|a| a.iter().any(|w| w == "c2")),
        "the second page is asked for by its cursor: {lists:?}"
    );
}

/// A success envelope that carries no adapter document is nothing a read can use: the client
/// answers `Unavailable`, and through the registry the series stays as one `unknown` read.
#[test]
fn adv_success_without_adapter_document_is_not_ok() {
    let dir = world(
        "adv_success_without_adapter_document_is_not_ok",
        &["adv-no-document"],
    );
    let connectors = Connectors::new(dir.join("connectors"));
    let answer = connectors
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input());
    assert!(
        matches!(answer, Err(ConnectorsError::Unavailable { .. })),
        "an answer without `result` is not a read: {answer:?}"
    );

    let reads = registry::read_with(
        Some(&Probe),
        &connectors,
        &definition(),
        &Timestamp("2026-10-14T22:57:03Z".into()),
        &RunId("run_test".into()),
    );
    assert_eq!(reads.len(), 1, "the series is kept as one read: {reads:?}");
    assert!(matches!(reads[0].reading, Reading::Unknown(_)), "{reads:?}");
}

/// The real CLI refuses an invoke whose `--revision` or `--schema` is not the current
/// description's (`stale_description`). The double answers it, so a client that sends the wrong
/// revision stays green.
#[test]
fn adv_double_refuses_invoke_with_stale_description() {
    let dir = world("adv_double_refuses_invoke_with_stale_description", &[]);
    let input = serde_json::to_string(&input()).expect("plain JSON");
    for (schema, revision) in [
        ("series.query/v1alpha1", "conn-rev-1"),
        ("series.query/v0", "op-rev-1"),
        ("", ""),
    ] {
        let out = double(
            &dir,
            &[
                "operations",
                "invoke",
                "--adapter",
                "metrics",
                "--connection",
                "metrics-main",
                "--operation",
                "series.query",
                "--schema",
                schema,
                "--revision",
                revision,
                "--input-json",
                &input,
                "--output",
                "json",
            ],
        );
        assert!(
            !answered_ok(&out),
            "schema {schema:?} revision {revision:?} is not the described op-rev-1: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

/// Without `--output json` the real CLI writes its human form. The double answers the JSON
/// envelope anyway, so a client that drops `--output json` stays green.
#[test]
fn adv_double_answers_json_only_when_asked() {
    let dir = world("adv_double_answers_json_only_when_asked", &[]);
    let out = double(
        &dir,
        &[
            "operations",
            "describe",
            "--adapter",
            "metrics",
            "--operation",
            "series.query",
        ],
    );
    assert!(
        !answered_ok(&out),
        "no --output json was given: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// A recording name joins adapter, connection and operation with `.`, which both a connection id
/// and an operation id may contain. A call naming another connection and another operation must
/// not be answered from the recording of `metrics-main` / `series.query`.
#[test]
fn adv_double_does_not_confuse_connection_and_operation() {
    let dir = world("adv_double_does_not_confuse_connection_and_operation", &[]);
    let input = serde_json::to_string(&input()).expect("plain JSON");
    let out = double(
        &dir,
        &[
            "operations",
            "invoke",
            "--adapter",
            "metrics",
            "--connection",
            "metrics-main.series",
            "--operation",
            "query",
            "--schema",
            "series.query/v1alpha1",
            "--revision",
            "op-rev-1",
            "--input-json",
            &input,
            "--output",
            "json",
        ],
    );
    assert!(
        !answered_ok(&out),
        "connection metrics-main.series has no recording: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Two different query maps must not share a digest. A one-entry query whose text is the
/// canonical JSON of a two-entry map hashes to the same bytes as that map.
#[test]
fn adv_query_digest_separates_one_text_from_a_map() {
    let map = BTreeMap::from([
        ("a".to_string(), QueryText("x".into())),
        ("b".to_string(), QueryText("y".into())),
    ]);
    let text = BTreeMap::from([(
        "value".to_string(),
        QueryText(r#"{"a":"x","b":"y"}"#.into()),
    )]);
    assert_ne!(
        query_digest(&map),
        query_digest(&text),
        "different queries, one digest"
    );
}
