//! The `kubernetes` adapter, read through the extractor registry against the recorded Connectors
//! double (`src/bin/connectors-double.rs`).
//!
//! Each test copies recorded answers from `tests/fixtures/kubernetes/common`, then from its
//! scenarios, into a fresh directory under the build's own temporary directory and links
//! `connectors` there to the double. `common` lists the configured namespaces `alpha`, `beta` and
//! `gamma` over two pages; a scenario records the `resources.list` answers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, QueryDigest, Reading, RunId,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Read, window_ending};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry::{self, NOT_IMPLEMENTED, SourceKind};

const END: &str = "2026-10-14T22:57:03Z";
/// SHA-256 of the canonical JSON of the query map,
/// `{"kind":"pods","predicate":"status.phase=Running"}`.
const QUERY_SHA256: &str = "4481a8b1df7f49d73bdf739b7199e3b207161acdafe1e69876d09b0f0d16f1af";

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    /// Recordings from `common`, then from each scenario in order, a later file replacing an
    /// earlier one of the same name.
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("kubernetes")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/kubernetes");
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

    /// The `--operation` of every `operations invoke` the double received, in order.
    fn invoked(&self) -> Vec<String> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()
                    .expect("argv is a list")
                    .iter()
                    .map(|a| a.as_str().expect("argv element").to_string())
                    .collect();
                if argv.get(..2) != Some(&["operations".to_string(), "invoke".to_string()]) {
                    return None;
                }
                let at = argv.iter().position(|a| a == "--operation")?;
                argv.get(at + 1).cloned()
            })
            .collect()
    }
}

fn definition(dimensions: &[&str], query: &[(&str, &str)]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.namespace.running_pods".into()),
        title: "Pods running in a namespace".into(),
        family: Family("platform".into()),
        kind: SignalKind::Instant,
        unit: Some(Unit("{pod}".into())),
        dimensions: Some(
            dimensions
                .iter()
                .map(|d| DimensionKey(d.to_string()))
                .collect(),
        ),
        source: Source {
            adapter: AdapterId("kubernetes".into()),
            connection: ConnectionId("kubernetes-main".into()),
        },
        extractor: ExtractorId("kubernetes.count".into()),
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

/// Running pods, one series per namespace, `cluster` declared but not carried by any object.
fn running_pods() -> SignalDefinitionData {
    definition(
        &["cluster", "namespace"],
        &[("kind", "pods"), ("predicate", "status.phase=Running")],
    )
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

/// The reads keyed by their `namespace` dimension; every read must carry one, once.
fn by_namespace(reads: &[Read]) -> BTreeMap<String, Reading> {
    let mut out = BTreeMap::new();
    for read in reads {
        let namespace = read
            .dims
            .get("namespace")
            .unwrap_or_else(|| panic!("a read without a namespace: {reads:?}"))
            .clone();
        assert!(
            out.insert(namespace, read.reading.clone()).is_none(),
            "two reads of one namespace: {reads:?}"
        );
    }
    out
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn reason(reading: &Reading) -> &str {
    match reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

#[test]
fn kubernetes_pod_count_per_namespace_known_answer() {
    let w = World::new(
        "kubernetes_pod_count_per_namespace_known_answer",
        &["per-namespace"],
    );
    let definition = running_pods();
    let reads = read(&w, &definition);

    // alpha: 3 running of 4 over two pages; beta: 1 of 2; gamma lists no pod, a complete 0.
    assert_eq!(
        by_namespace(&reads),
        BTreeMap::from([
            ("alpha".to_string(), ok("3")),
            ("beta".to_string(), ok("1")),
            ("gamma".to_string(), ok("0")),
        ]),
        "{reads:?}"
    );
    let window = window_ending(&definition.window, &end()).expect("a placeable window");
    for read in &reads {
        assert_eq!(read.adapter, AdapterId("kubernetes".into()));
        assert_eq!(read.connection, ConnectionId("kubernetes-main".into()));
        assert_eq!(read.query_sha256, QueryDigest(QUERY_SHA256.into()));
        assert_eq!(read.window, window);
        assert_eq!(
            read.retention_start, None,
            "Kubernetes exposes no retention"
        );
    }

    // Read-only: the adapter declares only list reads, and only they ran — the namespace listing
    // over its two pages and one `resources.list` per page of each namespace.
    let adapter = SourceKind::Kubernetes
        .adapter()
        .expect("the adapter exists");
    assert_eq!(
        adapter.read_operations().iter().collect::<BTreeSet<_>>(),
        BTreeSet::from([&"namespaces.list", &"resources.list"])
    );
    let invoked = w.invoked();
    let count = |op: &str| invoked.iter().filter(|o| *o == op).count();
    assert_eq!(
        (
            count("namespaces.list"),
            count("resources.list"),
            invoked.len()
        ),
        (2, 4, 6),
        "{invoked:?}"
    );
}

#[test]
fn kubernetes_unfinished_continuation_is_partial_lower_bound() {
    let w = World::new(
        "kubernetes_unfinished_continuation_is_partial_lower_bound",
        &["unfinished"],
    );
    let reads = read(&w, &running_pods());
    let lower = |value: &str| {
        Reading::Partial(PartialReading {
            value: Decimal(value.into()),
            bound: Bound::Lower,
        })
    };

    // alpha: the provider ended without finishing (`complete: false`, no cursor) after 3.
    // beta: the continuation of its second page was refused after 1.
    // gamma: one complete page.
    assert_eq!(
        by_namespace(&reads),
        BTreeMap::from([
            ("alpha".to_string(), lower("3")),
            ("beta".to_string(), lower("1")),
            ("gamma".to_string(), ok("1")),
        ]),
        "{reads:?}"
    );
}

#[test]
fn kubernetes_missing_status_is_unknown_never_zero() {
    let w = World::new(
        "kubernetes_missing_status_is_unknown_never_zero",
        &["missing-status"],
    );
    let reads = read(&w, &running_pods());
    let readings = by_namespace(&reads);
    assert_eq!(readings.len(), 3, "{reads:?}");

    // alpha: one Pending pod and one with no `status` at all; counted as no match it would read
    // 0, as a match 1. beta: one Running pod and one whose `status` has no `phase`.
    for namespace in ["alpha", "beta"] {
        let reading = &readings[namespace];
        assert!(
            reason(reading).starts_with("status not projected"),
            "{namespace}: {reading:?}"
        );
        assert!(
            reason(reading).contains("status.phase"),
            "the reason names the field: {reading:?}"
        );
    }
    let alpha = reads
        .iter()
        .find(|r| r.dims.get("namespace").map(String::as_str) == Some("alpha"))
        .expect("alpha read");
    assert_eq!(alpha.raw, None, "an unknown read carries no value text");
    assert_eq!(readings["gamma"], ok("1"));
}

#[test]
fn kubernetes_tenant_never_derived_from_namespace() {
    let w = World::new(
        "kubernetes_tenant_never_derived_from_namespace",
        &["tenant"],
    );
    // Every pod, no predicate. The namespace is named like a tenant and both it and its pods carry
    // `tenant` and `cluster` labels; neither key is a dimension the source carries for the series.
    let definition = definition(&["cluster", "namespace", "tenant"], &[("kind", "pods")]);
    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(
        reads[0].dims,
        BTreeMap::from([("namespace".to_string(), "tenant-a".to_string())]),
        "only the namespace, from the object metadata"
    );
    assert_eq!(reads[0].reading, ok("2"));
}

#[test]
fn kubernetes_refused_read_is_unknown_with_reason() {
    // A refused `resources.list`: each namespace's read is unknown, naming code and stage.
    let w = World::new(
        "kubernetes_refused_read_is_unknown_with_reason",
        &["refused"],
    );
    let reads = read(&w, &running_pods());
    let readings = by_namespace(&reads);
    assert_eq!(
        readings.keys().map(String::as_str).collect::<Vec<_>>(),
        ["alpha", "beta", "gamma"],
        "{reads:?}"
    );
    for (namespace, reading) in &readings {
        let reason = reason(reading);
        assert!(
            reason.contains("provider_unavailable") && reason.contains("dispatch"),
            "{namespace}: {reason}"
        );
        assert_ne!(reason, NOT_IMPLEMENTED);
    }

    // A refused namespace listing: one unknown read for the definition, naming code and stage.
    let w = World::new(
        "kubernetes_refused_read_is_unknown_with_reason_namespaces",
        &["namespaces-refused"],
    );
    let reads = read(&w, &running_pods());
    assert_eq!(reads.len(), 1, "{reads:?}");
    let reason = reason(&reads[0].reading);
    assert!(
        reason.contains("forbidden") && reason.contains("admission"),
        "{reason}"
    );
    assert_eq!(w.invoked(), ["namespaces.list"], "nothing listed after");
}

#[test]
fn kubernetes_unfinished_namespace_listing_is_unknown() {
    // The second page of the namespace listing is refused: alpha is read, and the namespaces that
    // may be missing are one unknown read with no dimension, never a silently dropped series.
    let w = World::new(
        "kubernetes_unfinished_namespace_listing_is_unknown",
        &["per-namespace", "namespaces-unfinished"],
    );
    let reads = read(&w, &running_pods());
    let (named, unnamed): (Vec<Read>, Vec<Read>) = reads
        .into_iter()
        .partition(|r| r.dims.contains_key("namespace"));
    assert_eq!(
        by_namespace(&named),
        BTreeMap::from([("alpha".to_string(), ok("3"))])
    );
    assert_eq!(unnamed.len(), 1, "{unnamed:?}");
    assert!(unnamed[0].dims.is_empty());
    let reason = reason(&unnamed[0].reading);
    assert!(
        reason.contains("namespace listing unfinished") && reason.contains("stale_cursor"),
        "{reason}"
    );
}

#[test]
fn kubernetes_predicate_not_equal_counts_the_rest() {
    let w = World::new(
        "kubernetes_predicate_not_equal_counts_the_rest",
        &["per-namespace"],
    );
    let definition = definition(
        &["namespace"],
        &[("kind", "pods"), ("predicate", "status.phase != Running")],
    );
    assert_eq!(
        by_namespace(&read(&w, &definition)),
        BTreeMap::from([
            ("alpha".to_string(), ok("1")),
            ("beta".to_string(), ok("1")),
            ("gamma".to_string(), ok("0")),
        ])
    );
}

#[test]
fn kubernetes_definition_it_cannot_read_is_unknown_before_any_call() {
    let w = World::new(
        "kubernetes_definition_it_cannot_read_is_unknown_before_any_call",
        &[],
    );
    let running = ("predicate", "status.phase=Running");
    let mut other_extractor = running_pods();
    other_extractor.extractor = ExtractorId("kubernetes.sum".into());
    let cases = [
        (other_extractor, "no extractor kubernetes.sum"),
        (
            definition(&["cluster"], &[("kind", "pods"), running]),
            "declares no dimension namespace",
        ),
        (
            definition(&["namespace"], &[("kind", "pods"), ("predicates", "x=y")]),
            "not \"predicates\"",
        ),
        (definition(&["namespace"], &[running]), "names no kind"),
        (
            definition(&["namespace"], &[("kind", "pods"), ("predicate", "status")]),
            "is not <field>=<value>",
        ),
        (
            definition(
                &["namespace"],
                &[("kind", "pods"), ("predicate", "status..phase=Running")],
            ),
            "dotted path",
        ),
    ];
    // Every spelling outside the closed grammar `<field>(=|==|!=)<value>` is refused, never read
    // with the rest as a literal value: conjunctions, set and comparison operators, quotes, a
    // field that is no dotted path of plain segments, and an empty value.
    let refused = [
        ("status.phase=Running,metadata.name=api-1", "','"),
        ("status.phase=\"Running\"", "'\"'"),
        ("status.phase='Running'", "'\\''"),
        (
            "status.phase in (Running,Pending)",
            "is not <field>=<value>",
        ),
        ("status.restartCount>=3", "dotted path"),
        ("status.restartCount<3", "is not <field>=<value>"),
        ("status.phase!==Running", "'='"),
        ("status.phase===Running", "'='"),
        ("status.phase=!Running", "'!'"),
        ("status.phase=(Running)", "'('"),
        ("status.phase=", "empty value"),
        ("metadata.labels[app]=api", "dotted path"),
        ("status phase=Running", "dotted path"),
        ("status.phase=~Running", "'~'"),
        ("status.phase=Running|Pending", "'|'"),
        ("metadata.name=api-*", "'*'"),
        ("status.phase=Running Pending", "' '"),
        ("status.phase=Running\tPending", "'\\t'"),
        ("status.phase=Running\u{200b}", "'\\u{200b}'"),
        ("status.phase=Runn\u{0456}ng", "'\u{0456}'"),
        ("status.phase=Run;ning", "';'"),
        ("status.phase=Run\\ning", "'\\\\'"),
    ];
    let cases = cases
        .into_iter()
        .chain(refused.iter().map(|(predicate, why)| {
            (
                definition(
                    &["namespace"],
                    &[("kind", "pods"), ("predicate", predicate)],
                ),
                *why,
            )
        }));
    for (definition, expected) in cases {
        let reads = read(&w, &definition);
        assert_eq!(reads.len(), 1, "{reads:?}");
        assert!(reads[0].dims.is_empty(), "{reads:?}");
        assert!(reason(&reads[0].reading).contains(expected), "{reads:?}");
    }
    assert!(w.invoked().is_empty(), "nothing ran: {:?}", w.invoked());
}
