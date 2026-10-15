//! The Loki adapter over recorded Connectors answers (`tests/fixtures/loki/<scenario>`), read
//! through the registry and the recorded double (`src/bin/connectors-double.rs`).
//!
//! Each test copies the recordings of `common` and its scenarios into a fresh directory under the
//! build's own temporary directory and links `connectors` there to the double, which appends every
//! call to `calls.jsonl` beside them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Bound, CompleteReading, PartialReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Population, Read, query_digest};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const VALUE_QUERY: &str =
    r#"sum by (cluster, namespace, app) (count_over_time({app="api"} |= "error" [5m]))"#;
/// SHA-256 of the canonical JSON of `{"value": VALUE_QUERY}`.
const VALUE_QUERY_SHA256: &str = "e516b7422141b3a112403d552239fd3298e4012d8c6c9a679af927cb44e49488";
const NUMERATOR: &str = r#"sum by (cluster) (count_over_time({app="api"} |= "error" [6h]))"#;
const DENOMINATOR: &str = r#"sum by (cluster) (count_over_time({app="api"} [6h]))"#;
const LINES_QUERY: &str = r#"{app="api"} |= "error""#;

const END: &str = "2026-10-14T22:57:03Z";
const START: &str = "2026-10-14T16:57:03Z";
/// `END` and `START` in Unix nanoseconds, as Connectors' Loki inputs take them.
const END_NS: &str = "1792018623000000000";
const START_NS: &str = "1791997023000000000";
/// The largest `limit` Connectors' `logs.query_range` admits (Loki profile § 11.3).
const LINE_LIMIT: usize = 1000;

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    /// Recordings from `common`, then from each scenario in order, a later file replacing an
    /// earlier one of the same name.
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("loki")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/loki");
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

    /// Writes one recording into this world, replacing any of the same name.
    fn record(&self, name: &str, recording: &Value) {
        std::fs::write(self.dir.join(name), recording.to_string()).expect("write a recording");
    }

    fn connectors(&self) -> Connectors {
        Connectors::new(self.dir.join("connectors"))
    }

    /// The `--operation` and parsed `--input-json` of every `operations invoke` call, in order.
    fn invokes(&self) -> Vec<(String, Value)> {
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
            .filter(|argv| argv.starts_with(&["operations".into(), "invoke".into()]))
            .map(|argv| {
                let flag = |name: &str| {
                    let at = argv.iter().position(|a| a == name).expect("flag present");
                    argv[at + 1].clone()
                };
                let input = serde_json::from_str(&flag("--input-json")).expect("input is JSON");
                (flag("--operation"), input)
            })
            .collect()
    }
}

fn definition(
    extractor: &str,
    kind: SignalKind,
    unit: &str,
    dimensions: &[&str],
    query: &[(&str, &str)],
) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.api_errors".into()),
        title: "Errors the API logged".into(),
        family: Family("platform".into()),
        kind,
        unit: Some(Unit(unit.into())),
        dimensions: Some(
            dimensions
                .iter()
                .map(|d| DimensionKey((*d).into()))
                .collect(),
        ),
        source: Source {
            adapter: AdapterId("logs".into()),
            connection: ConnectionId("logs-main".into()),
        },
        extractor: ExtractorId(extractor.into()),
        query: query
            .iter()
            .map(|(name, text)| ((*name).to_string(), QueryText((*text).into())))
            .collect(),
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

fn ratio_definition(unit: &str) -> SignalDefinitionData {
    let mut definition = definition(
        "loki.ratio",
        SignalKind::Ratio,
        unit,
        &["cluster"],
        &[("numerator", NUMERATOR), ("denominator", DENOMINATOR)],
    );
    definition.denominator = Some(DenominatorSpec {
        unit: Unit("{line}".into()),
        min_n: 20,
        since: "first_sample".into(),
    });
    definition
}

fn read(world: &World, definition: &SignalDefinitionData) -> Vec<Read> {
    registry::read(
        &world.connectors(),
        definition,
        &Timestamp(END.into()),
        &RunId("run_test".into()),
    )
}

fn dims(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

/// A `logs.query_range` answer as the Connectors CLI wraps it: `lines` lines of one stream.
fn range_answer(lines: usize, complete: bool, causes: &[&str]) -> Value {
    let lines: Vec<Value> = (0..lines)
        .map(|i| {
            json!({
                "timestamp_unix_ns": format!("{}", 1_792_018_622_000_000_000_u64 - i as u64),
                "stream": {"app": "api"},
                "line": "request failed",
                "line_truncated": false,
                "redacted": false,
                "source": null,
            })
        })
        .collect();
    let saturated = causes.contains(&"provider_limit");
    let document = json!({
        "lines": lines,
        "selection": {"kind": "loki-range", "start_unix_ns": START_NS, "end_unix_ns": END_NS, "direction": "backward"},
        "order": "timestamp-backward",
        "complete": complete,
        "truncation": {
            "causes": causes,
            "occurrences_dropped": if saturated { Value::Null } else { json!(0) },
            "stream_groups_dropped": if saturated { Value::Null } else { json!(0) },
        },
        "next_cursor": null,
        "provenance": {"instance": "loki-test", "resource": "loki:query_range", "observed_at_unix_ms": 1_792_018_623_000_u64, "source_revision": null},
    });
    json!({
        "stdout": {
            "ok": true,
            "result": {
                "adapter": "logs",
                "operation": "logs.query_range",
                "revision": "op-rev-range-1",
                "result": document.to_string(),
            }
        }
    })
}

const RANGE_INVOKE: &str = "operations+invoke+a=logs+c=logs-main+o=logs.query_range.json";
const METRIC_INVOKE: &str = "operations+invoke+a=logs+c=logs-main+o=logs.query_metric.json";

/// A complete `logs.query_metric` instant answer of `query` as the Connectors CLI wraps it, one
/// series per label set and value.
fn metric_answer(query: &str, series: &[(Value, &str)]) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(labels, value)| {
            json!({"labels": labels, "samples": [{"timestamp_unix_ns": END_NS, "value": value}]})
        })
        .collect();
    let document = json!({
        "result_type": "vector",
        "series": series,
        "complete": true,
        "truncation": [],
        "provenance": {"instance": "loki-test", "profile": "logql-metric", "observed_at_unix_ms": 1_792_018_623_000_u64},
    });
    json!({
        "input": {"query": query},
        "stdout": {
            "ok": true,
            "result": {
                "adapter": "logs",
                "operation": "logs.query_metric",
                "revision": "op-rev-metric-1",
                "result": document.to_string(),
            }
        }
    })
}

#[test]
fn loki_metric_series_become_samples_with_source_labels_only() {
    let w = World::new(
        "loki_metric_series_become_samples_with_source_labels_only",
        &["series"],
    );
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster", "namespace"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 2, "one read per returned series: {reads:?}");
    // `app` is a label of both series but no dimension of the definition: dropped. `beta` carries
    // no `namespace`, so its read has none; nothing derives it (L9).
    assert_eq!(
        reads[0].dims,
        dims(&[("cluster", "alpha"), ("namespace", "shop")])
    );
    assert_eq!(reads[0].reading, ok("42"));
    assert_eq!(reads[0].raw.as_deref(), Some("42"));
    assert_eq!(reads[1].dims, dims(&[("cluster", "beta")]));
    assert_eq!(reads[1].reading, ok("7.5"));
    assert_eq!(reads[1].raw.as_deref(), Some("7.5"));
    for read in &reads {
        assert_eq!(read.adapter, AdapterId("logs".into()));
        assert_eq!(read.connection, ConnectionId("logs-main".into()));
        assert_eq!(read.query_sha256.0, VALUE_QUERY_SHA256);
        assert_eq!(read.query_sha256, query_digest(&definition.query));
        assert_eq!(read.window.start, Timestamp(START.into()));
        assert_eq!(read.window.end, Timestamp(END.into()));
        assert_eq!(read.denominator, None);
        assert_eq!(read.retention_start, None);
    }

    // One instant metric query at the window end, the definition's text unchanged.
    assert_eq!(
        w.invokes(),
        [(
            "logs.query_metric".to_string(),
            json!({"query": VALUE_QUERY, "time_unix_ns": END_NS})
        )]
    );
}

#[test]
fn loki_ratio_is_decimal_with_unit_1() {
    let w = World::new("loki_ratio_is_decimal_with_unit_1", &["ratio"]);

    let reads = read(&w, &ratio_definition("1"));

    assert_eq!(reads.len(), 3, "{reads:?}");
    let by_cluster: BTreeMap<&str, &Read> = reads
        .iter()
        .map(|r| (r.dims["cluster"].as_str(), r))
        .collect();
    // 15 / 556 = 0.026978417266187050359…: a decimal string, never a percentage, never text.
    let alpha = by_cluster["alpha"];
    assert_eq!(alpha.reading, ok("0.02697841726618705"));
    assert_eq!(
        alpha.denominator,
        Some(Population {
            n: 556,
            unit: Unit("{line}".into())
        })
    );
    assert_eq!(by_cluster["gamma"].reading, ok("0.375"));
    assert_eq!(
        by_cluster["gamma"].denominator.as_ref().map(|p| p.n),
        Some(8)
    );
    // `count_over_time` emits nothing for a cluster without an error line: absent from the
    // complete numerator answer and present in the denominator's, it is 0 errors of its lines.
    let beta = by_cluster["beta"];
    assert_eq!(beta.reading, ok("0"), "{beta:?}");
    assert_eq!(beta.denominator.as_ref().map(|p| p.n), Some(40));
    for read in &reads {
        assert_eq!(
            read.query_sha256,
            query_digest(&ratio_definition("1").query)
        );
    }

    // Both queries, at the same instant.
    assert_eq!(
        w.invokes(),
        [
            (
                "logs.query_metric".to_string(),
                json!({"query": NUMERATOR, "time_unix_ns": END_NS})
            ),
            (
                "logs.query_metric".to_string(),
                json!({"query": DENOMINATOR, "time_unix_ns": END_NS})
            ),
        ]
    );

    // A ratio has unit `1`. A `loki.ratio` declaring another unit is not read as one.
    let w = World::new("loki_ratio_is_decimal_with_unit_1_other_unit", &["ratio"]);
    let reads = read(&w, &ratio_definition("{line}"));
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reason(&reads[0]).contains("unit 1"), "{reads:?}");
}

/// A complete numerator answer with no series at all, over a denominator that answered some,
/// reads unknown for every denominator series, without a population (design § 11 D20). A renamed
/// stream label or a broken matcher answers no series exactly as a healthy instance with no error
/// line does, and no numerator series is there to show the two queries group alike.
#[test]
fn loki_empty_numerator_answer_infers_no_zero() {
    let w = World::new("loki_empty_numerator_answer_infers_no_zero", &[]);
    w.record(
        METRIC_INVOKE,
        &json!([
            metric_answer(NUMERATOR, &[]),
            metric_answer(
                DENOMINATOR,
                &[
                    (json!({"cluster": "alpha"}), "556"),
                    (json!({"cluster": "beta"}), "40"),
                ]
            ),
        ]),
    );

    let reads = read(&w, &ratio_definition("1"));

    assert_eq!(reads.len(), 2, "one read per denominator series: {reads:?}");
    assert_eq!(reads[0].dims, dims(&[("cluster", "alpha")]));
    assert_eq!(reads[1].dims, dims(&[("cluster", "beta")]));
    for read in &reads {
        assert!(
            reason(read).contains("the numerator answered no series"),
            "{read:?}"
        );
        assert_eq!(read.denominator, None, "{read:?}");
        assert_eq!(read.raw, None, "{read:?}");
        assert_eq!(read.window.start, Timestamp(START.into()));
        assert_eq!(
            read.query_sha256,
            query_digest(&ratio_definition("1").query)
        );
    }

    // Without dimensions the one denominator series is the whole instance: unknown, never 0.
    let w = World::new("loki_empty_numerator_answer_infers_no_zero_whole", &[]);
    w.record(
        METRIC_INVOKE,
        &json!([
            metric_answer(NUMERATOR, &[]),
            metric_answer(DENOMINATOR, &[(json!({}), "40")]),
        ]),
    );
    let mut whole = ratio_definition("1");
    whole.dimensions = None;
    let reads = read(&w, &whole);
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reads[0].dims.is_empty(), "{reads:?}");
    assert!(
        reason(&reads[0]).contains("the numerator answered no series"),
        "{reads:?}"
    );
    assert_eq!(reads[0].denominator, None, "{reads:?}");
}

#[test]
fn loki_nan_value_is_unknown_never_zero() {
    let w = World::new("loki_nan_value_is_unknown_never_zero", &["nan"]);
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 4, "every series is kept: {reads:?}");
    let by_cluster: BTreeMap<&str, &Read> = reads
        .iter()
        .map(|r| (r.dims["cluster"].as_str(), r))
        .collect();
    // The non-finite values Loki writes (Go's `FormatFloat(v, 'f', -1, 64)`); other text that is
    // not a decimal is held by `loki::tests::only_the_decimal_grammar_parses`.
    for (cluster, text) in [("alpha", "NaN"), ("beta", "+Inf"), ("gamma", "-Inf")] {
        let read = by_cluster[cluster];
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "{cluster}: {read:?}"
        );
        assert!(reason(read).contains(text), "{cluster}: {read:?}");
        assert_eq!(read.raw.as_deref(), Some(text), "{cluster}");
    }
    assert_eq!(by_cluster["epsilon"].reading, ok("3"));
    for read in &reads {
        assert_ne!(read.reading, ok("0"), "{read:?}");
    }
}

#[test]
fn loki_line_limit_reached_is_partial_lower_bound() {
    let definition = definition(
        "loki.count",
        SignalKind::CountOfCases,
        "{line}",
        &[],
        &[("value", LINES_QUERY)],
    );

    // Connectors saturated at the limit: the count is at least the lines returned.
    let w = World::new("loki_line_limit_reached_is_partial_lower_bound", &[]);
    w.record(
        RANGE_INVOKE,
        &range_answer(LINE_LIMIT, false, &["provider_limit"]),
    );
    let reads = read(&w, &definition);
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(
        reads[0].reading,
        Reading::Partial(PartialReading {
            value: Decimal(LINE_LIMIT.to_string()),
            bound: Bound::Lower,
        })
    );
    // The request names the window and the limit it is counted against.
    assert_eq!(
        w.invokes(),
        [(
            "logs.query_range".to_string(),
            json!({"query": LINES_QUERY, "start_unix_ns": START_NS, "end_unix_ns": END_NS, "limit": LINE_LIMIT})
        )]
    );

    // An answer that reached the limit is a lower bound even when it calls itself complete.
    let w = World::new("loki_line_limit_reached_is_partial_lower_bound_at_cap", &[]);
    w.record(RANGE_INVOKE, &range_answer(LINE_LIMIT, true, &[]));
    let reads = read(&w, &definition);
    assert!(
        matches!(
            &reads[..],
            [Read {
                reading: Reading::Partial(PartialReading {
                    bound: Bound::Lower,
                    ..
                }),
                ..
            }]
        ),
        "{reads:?}"
    );

    // Below the limit the count is complete, per series of the declared dimensions; clipped line
    // text does not cut the count.
    let w = World::new(
        "loki_line_limit_reached_is_partial_lower_bound_below",
        &["lines"],
    );
    let mut by_cluster = definition.clone();
    by_cluster.dimensions = Some(vec![DimensionKey("cluster".into())]);
    let reads = read(&w, &by_cluster);
    assert_eq!(reads.len(), 2, "{reads:?}");
    assert_eq!(reads[0].dims, dims(&[("cluster", "alpha")]));
    assert_eq!(reads[0].reading, ok("2"));
    assert_eq!(reads[1].dims, dims(&[("cluster", "beta")]));
    assert_eq!(reads[1].reading, ok("1"));

    // No line in the window, read completely: a count of the whole instance is 0; a dimensioned
    // signal has no series to put a 0 on, so it is unknown, never 0 at no dims.
    let w = World::new("loki_line_limit_reached_is_partial_lower_bound_empty", &[]);
    w.record(RANGE_INVOKE, &range_answer(0, true, &[]));
    let reads = read(&w, &definition);
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reads[0].dims.is_empty(), "{reads:?}");
    assert_eq!(reads[0].reading, ok("0"));
    let reads = read(&w, &by_cluster);
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(
        reason(&reads[0]).contains("no series in the window"),
        "{reads:?}"
    );
}

#[test]
fn loki_refused_read_is_unknown_with_reason() {
    let value = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );
    for definition in [value, ratio_definition("1")] {
        let extractor = definition.extractor.0.clone();
        let w = World::new(
            &format!("loki_refused_read_is_unknown_with_reason_{extractor}"),
            &["refused"],
        );

        let reads = read(&w, &definition);

        assert_eq!(reads.len(), 1, "{extractor}: {reads:?}");
        let read = &reads[0];
        let why = reason(read);
        for part in ["logs.query_metric", "rate_limited", "dispatch"] {
            assert!(why.contains(part), "{extractor}: {part} in {why:?}");
        }
        assert_eq!(read.raw, None, "{extractor}");
        assert_eq!(read.denominator, None, "{extractor}");
        assert!(read.dims.is_empty(), "{extractor}");
        assert_eq!(read.query_sha256, query_digest(&definition.query));
        assert_eq!(read.window.start, Timestamp(START.into()));
    }
}
