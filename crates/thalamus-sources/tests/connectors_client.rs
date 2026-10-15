//! The one Connectors client every adapter reads through, run against the recorded double
//! (`src/bin/connectors-double.rs`).
//!
//! Each test copies recorded answers from `tests/fixtures/connectors/<scenario>` into a fresh
//! directory under the build's own temporary directory and links `connectors` there to the double.
//! The double reads its recordings from the directory it was started from and appends every call
//! to `calls.jsonl` beside them, so the client runs with the test process's environment untouched.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Adapter, Read, unknown};
use thalamus_sources::connectors::{Connectors, ConnectorsError, ReadClient};
use thalamus_sources::registry::{self, SourceKind};

const QUERY: &str = "sum(rate(requests_failed_total[5m])) / sum(rate(requests_total[5m]))";
/// SHA-256 of the canonical JSON of the query map, `{"value":"<QUERY>"}`.
const QUERY_SHA256: &str = "dc00142179a8e3cfc1666b0b90c0dc5f9871b69946708d1659189bee5931fa7f";
const END: &str = "2026-10-14T22:57:03Z";

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    /// Recordings from `common`, then from each scenario in order, a later file replacing an
    /// earlier one of the same name.
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("connectors_client")
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
        World { dir }
    }

    fn connectors(&self) -> Connectors {
        Connectors::new(self.dir.join("connectors"))
    }

    /// Every call the double received, in order.
    fn calls(&self) -> Vec<Value> {
        match std::fs::read_to_string(self.dir.join("calls.jsonl")) {
            Ok(text) => text
                .lines()
                .map(|line| serde_json::from_str(line).expect("a call is one JSON line"))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn argvs(&self) -> Vec<Vec<String>> {
        self.calls()
            .iter()
            .map(|call| {
                call["argv"]
                    .as_array()
                    .expect("argv is a list")
                    .iter()
                    .map(|a| a.as_str().expect("argv element").to_string())
                    .collect()
            })
            .collect()
    }

    /// The calls whose first two words are `group verb`.
    fn count(&self, group: &str, verb: &str) -> usize {
        self.argvs()
            .iter()
            .filter(|argv| argv.first().map(String::as_str) == Some(group))
            .filter(|argv| argv.get(1).map(String::as_str) == Some(verb))
            .count()
    }
}

fn source() -> Source {
    Source {
        adapter: AdapterId("metrics".into()),
        connection: ConnectionId("metrics-main".into()),
    }
}

fn definition(extractor: &str) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.request_failure_ratio".into()),
        title: "Share of requests that failed".into(),
        family: Family("platform".into()),
        kind: SignalKind::Ratio,
        unit: Some(Unit("1".into())),
        dimensions: Some(vec![DimensionKey("cluster".into())]),
        source: source(),
        extractor: ExtractorId(extractor.into()),
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

fn end() -> Timestamp {
    Timestamp(END.into())
}

fn run_id() -> RunId {
    RunId("run_test".into())
}

fn input() -> Value {
    json!({"query": QUERY})
}

/// A test adapter that reads with `series.query` and answers one read per returned series.
struct Probe {
    /// The operation it actually invokes, so a test can make it ask for one it did not declare.
    invokes: &'static str,
}

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
        let answer = client.invoke(&definition.source, self.invokes, &input())?;
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

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

#[test]
fn write_operation_is_refused_before_invocation() {
    let w = World::new("write_operation_is_refused_before_invocation", &[]);
    let connectors = w.connectors();
    let client = connectors.reads(&["series.query"]);

    for write in ["rules.delete", "silences.create", "series.query_range", ""] {
        let refused = client.invoke(&source(), write, &input());
        assert_eq!(
            refused,
            Err(ConnectorsError::NotARead {
                operation: write.to_string()
            }),
            "{write:?} is not a declared read"
        );
    }
    assert!(w.calls().is_empty(), "nothing ran: {:?}", w.calls());

    // Through the registry the refusal is an unknown read, and still nothing runs.
    let reads = registry::read_with(
        Some(&Probe {
            invokes: "rules.delete",
        }),
        &connectors,
        &definition("promql.value"),
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reason(&reads[0]).contains("rules.delete"), "{reads:?}");
    assert!(w.calls().is_empty(), "nothing ran: {:?}", w.calls());

    // The declared read itself runs, so the empty log above is the refusal and not a dead double.
    client
        .invoke(&source(), "series.query", &input())
        .expect("a declared read is invoked");
    assert_eq!(w.count("operations", "invoke"), 1, "{:?}", w.argvs());
}

#[test]
fn argv_and_env_carry_no_credential() {
    let w = World::new("argv_and_env_carry_no_credential", &["lapsed"]);
    let connectors = w.connectors();
    connectors
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input())
        .expect("the read is answered after one revalidation");
    let calls = w.calls();
    assert!(
        w.count("connections", "list") >= 1
            && w.count("connections", "revalidate") == 1
            && w.count("operations", "describe") >= 1
            && w.count("operations", "invoke") == 2,
        "every kind of call was made: {:?}",
        w.argvs()
    );

    // The client adds no variable to the environment and removes none: Connectors gets exactly the
    // environment of the process that runs thalamus, and thalamus holds no secret to add.
    let parent: BTreeSet<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();
    for call in &calls {
        let child: BTreeSet<String> = call["env"]
            .as_array()
            .expect("env names")
            .iter()
            .map(|n| n.as_str().expect("env name").to_string())
            .collect();
        assert_eq!(child, parent, "environment of {call}");
    }

    // Every argv element is a fixed word, a fixed flag, or one of the values the read is made of.
    let words = [
        "connections",
        "list",
        "revalidate",
        "operations",
        "describe",
        "invoke",
        "json",
    ];
    let flags = [
        "--adapter",
        "--connection",
        "--operation",
        "--schema",
        "--revision",
        "--expected-revision",
        "--input-json",
        "--output",
    ];
    let input_json = serde_json::to_string(&input()).expect("plain JSON");
    let values = [
        "metrics",
        "metrics-main",
        "series.query",
        "series.query/v1alpha1",
        "op-rev-1",
        "conn-rev-1",
        input_json.as_str(),
    ];
    for argv in w.argvs() {
        for arg in &argv {
            let arg = arg.as_str();
            assert!(
                words.contains(&arg) || flags.contains(&arg) || values.contains(&arg),
                "{arg:?} in {argv:?} is not part of a read"
            );
            let lower = arg.to_ascii_lowercase();
            for secret in ["credential", "token", "secret", "password", "approval"] {
                assert!(!lower.contains(secret), "{arg:?} in {argv:?}");
            }
        }
    }
}

#[test]
fn refusal_on_stderr_is_typed_with_code_and_stage() {
    let w = World::new(
        "refusal_on_stderr_is_typed_with_code_and_stage",
        &["refused"],
    );
    let refused = w
        .connectors()
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input())
        .expect_err("the recorded answer is a refusal");
    match &refused {
        ConnectorsError::Refused { code, stage, .. } => {
            assert_eq!(
                (code.as_str(), stage.as_str()),
                ("provider_unavailable", "dispatch")
            );
        }
        other => panic!("expected a typed refusal, got {other:?}"),
    }
    let text = refused.to_string();
    assert!(
        text.contains("provider_unavailable") && text.contains("dispatch"),
        "{text}"
    );
    assert_eq!(
        w.count("operations", "invoke"),
        1,
        "a refusal past admission is not retried: {:?}",
        w.argvs()
    );
}

#[test]
fn lapsed_evidence_revalidates_once_then_retries() {
    let w = World::new("lapsed_evidence_revalidates_once_then_retries", &["lapsed"]);
    let answer = w
        .connectors()
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input())
        .expect("answered after one revalidation");
    assert_eq!(answer, json!({"complete": true, "series": []}));
    assert_eq!(w.count("operations", "invoke"), 2, "{:?}", w.argvs());
    assert_eq!(w.count("connections", "revalidate"), 1, "{:?}", w.argvs());
    let order: Vec<String> = w
        .argvs()
        .iter()
        .map(|a| a[1].clone())
        .filter(|verb| verb == "invoke" || verb == "revalidate")
        .collect();
    assert_eq!(order, ["invoke", "revalidate", "invoke"]);
    let revalidate = w
        .argvs()
        .into_iter()
        .find(|a| a[1] == "revalidate")
        .expect("one revalidation");
    let at = revalidate
        .iter()
        .position(|a| a == "--expected-revision")
        .expect("names the revision it renews");
    assert_eq!(revalidate[at + 1], "conn-rev-1");

    // Lapsed again after the renewal: one revalidation is all, and the refusal is the answer.
    let w = World::new(
        "lapsed_evidence_revalidates_once_then_retries_again",
        &["lapsed-again"],
    );
    let refused =
        w.connectors()
            .reads(&["series.query"])
            .invoke(&source(), "series.query", &input());
    assert!(
        matches!(&refused, Err(ConnectorsError::Refused { code, stage, .. })
            if code == "not_granted" && stage == "admission"),
        "{refused:?}"
    );
    assert_eq!(w.count("operations", "invoke"), 2, "{:?}", w.argvs());
    assert_eq!(w.count("connections", "revalidate"), 1, "{:?}", w.argvs());
}

#[test]
fn admission_timeout_retries_once() {
    let w = World::new("admission_timeout_retries_once", &["timeout"]);
    let answer = w
        .connectors()
        .reads(&["series.query"])
        .invoke(&source(), "series.query", &input())
        .expect("answered on the retry");
    assert_eq!(answer, json!({"complete": true, "series": []}));
    let invokes: Vec<Vec<String>> = w.argvs().into_iter().filter(|a| a[1] == "invoke").collect();
    assert_eq!(invokes.len(), 2, "{:?}", w.argvs());
    assert_eq!(invokes[0], invokes[1], "the same read both times");
    assert_eq!(w.count("connections", "revalidate"), 0, "{:?}", w.argvs());

    let w = World::new("admission_timeout_retries_once_again", &["timeout-again"]);
    let refused =
        w.connectors()
            .reads(&["series.query"])
            .invoke(&source(), "series.query", &input());
    assert!(
        matches!(&refused, Err(ConnectorsError::Refused { code, stage, .. })
            if code == "timeout" && stage == "admission"),
        "{refused:?}"
    );
    assert_eq!(
        w.count("operations", "invoke"),
        2,
        "one retry, no more: {:?}",
        w.argvs()
    );
}

#[test]
fn connectors_failure_becomes_unknown_read_with_code_and_stage() {
    let w = World::new(
        "connectors_failure_becomes_unknown_read_with_code_and_stage",
        &["refused"],
    );
    let definition = definition("promql.value");
    let reads = registry::read_with(
        Some(&Probe {
            invokes: "series.query",
        }),
        &w.connectors(),
        &definition,
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "the series is kept as one read: {reads:?}");
    let read = &reads[0];
    let why = reason(read);
    assert!(
        why.contains("provider_unavailable") && why.contains("dispatch"),
        "{why}"
    );
    assert_eq!(read.adapter, AdapterId("metrics".into()));
    assert_eq!(read.connection, ConnectionId("metrics-main".into()));
    assert_eq!(read.query_sha256.0, QUERY_SHA256);
    assert_eq!(read.window.start, Timestamp("2026-10-14T16:57:03Z".into()));
    assert_eq!(read.window.end, end());
    assert_eq!(read.denominator, None);
    assert_eq!(read.raw, None);
    assert_eq!(read.retention_start, None);

    // A Connectors that cannot be run at all is unknown too, never a dropped series.
    let missing = Connectors::new(w.dir.join("no-such-connectors"));
    let reads = registry::read_with(
        Some(&Probe {
            invokes: "series.query",
        }),
        &missing,
        &definition,
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reason(&reads[0]).contains("connectors"), "{reads:?}");

    // The answered read, for contrast, carries the value and the producer's text of it.
    let w = World::new("connectors_failure_contrast", &[]);
    let reads = registry::read_with(
        Some(&Probe {
            invokes: "series.query",
        }),
        &w.connectors(),
        &definition,
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(
        reads[0].reading,
        Reading::Ok(CompleteReading {
            value: Decimal("0.25".into())
        })
    );
    assert_eq!(reads[0].raw.as_deref(), Some("0.25"));
}

#[test]
fn registry_chooses_adapter_by_extractor_prefix() {
    let cases = [
        ("promql.value", SourceKind::Prometheus),
        ("promql.ratio", SourceKind::Prometheus),
        ("loki.count", SourceKind::Loki),
        ("sql.ratio", SourceKind::Sql),
        ("kubernetes.ready_ratio", SourceKind::Kubernetes),
        ("git.commits", SourceKind::Git),
        ("http_probe.status", SourceKind::HttpProbe),
    ];
    for (extractor, kind) in cases {
        assert_eq!(
            registry::choose(&ExtractorId(extractor.into())),
            Some(kind),
            "{extractor}"
        );
    }
    for near_miss in [
        "promql",
        "promqlx.value",
        "PROMQL.value",
        "sqlite.ratio",
        "http.status",
        ".value",
        "",
        "plugin.custom",
    ] {
        assert_eq!(
            registry::choose(&ExtractorId(near_miss.into())),
            None,
            "{near_miss:?}"
        );
    }

    // An extractor no arm takes is an unknown read naming it, and nothing runs.
    let w = World::new("registry_chooses_adapter_by_extractor_prefix", &[]);
    let reads = registry::read(
        &w.connectors(),
        &definition("plugin.custom"),
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reason(&reads[0]).contains("plugin.custom"), "{reads:?}");
    assert!(w.calls().is_empty(), "{:?}", w.calls());
}

#[test]
fn unimplemented_adapter_reads_unknown_never_zero() {
    let w = World::new("unimplemented_adapter_reads_unknown_never_zero", &[]);
    let connectors = w.connectors();

    let reads = registry::read_with(
        None,
        &connectors,
        &definition("promql.value"),
        &end(),
        &run_id(),
    );
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(reason(&reads[0]), "adapter not implemented");
    assert_eq!(reads[0].raw, None);
    assert_eq!(reads[0].denominator, None);

    // Every arm whose adapter is still the stub reads the same way through the registry. An adapter
    // story replaces its stub, and its arm then leaves this loop.
    let extractors = [
        "promql.value",
        "loki.count",
        "sql.ratio",
        "kubernetes.ready_ratio",
        "git.commits",
        "http_probe.status",
    ];
    for extractor in extractors {
        let kind = registry::choose(&ExtractorId(extractor.into())).expect("an arm");
        if kind.adapter().is_some() {
            continue;
        }
        let reads = registry::read(&connectors, &definition(extractor), &end(), &run_id());
        assert_eq!(reads.len(), 1, "{extractor}: {reads:?}");
        assert!(
            matches!(&reads[0].reading, Reading::Unknown(u) if u.reason == "adapter not implemented"),
            "{extractor}: {reads:?}"
        );
    }
    assert!(w.calls().is_empty(), "{:?}", w.calls());
}
