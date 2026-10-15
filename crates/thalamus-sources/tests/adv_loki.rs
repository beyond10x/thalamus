//! Adversary cases for the Loki adapter (wave 20261015c, pass 1), over Connectors answers shaped
//! as `adapters/loki/src/lib.rs` of Connectors v0.45.0 builds them, through the registry and the
//! recorded double.

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
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";
const START_NS: &str = "1791997023000000000";
const END_NS: &str = "1792018623000000000";
const VALUE_QUERY: &str = r#"sum by (cluster) (count_over_time({app="api"} |= "error" [6h]))"#;
const NUMERATOR: &str = r#"sum by (cluster) (count_over_time({app="api"} |= "error" [6h]))"#;
const DENOMINATOR: &str = r#"sum by (cluster) (count_over_time({app="api"} [6h]))"#;
const LINES_QUERY: &str = r#"{app="api"} |= "error""#;
const METRIC_INVOKE: &str = "operations+invoke+a=logs+c=logs-main+o=logs.query_metric.json";
const RANGE_INVOKE: &str = "operations+invoke+a=logs+c=logs-main+o=logs.query_range.json";

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv_loki")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let common = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/loki/common");
        for entry in std::fs::read_dir(common).expect("common recordings") {
            let path = entry.expect("fixture entry").path();
            std::fs::copy(&path, dir.join(path.file_name().expect("file name")))
                .expect("copy a recording");
        }
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_connectors-double"),
            dir.join("connectors"),
        )
        .expect("link the double");
        World { dir }
    }

    fn record(&self, name: &str, recording: &Value) {
        std::fs::write(self.dir.join(name), recording.to_string()).expect("write a recording");
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

fn ratio_definition() -> SignalDefinitionData {
    let mut definition = definition(
        "loki.ratio",
        SignalKind::Ratio,
        "1",
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
        &Connectors::new(world.dir.join("connectors")),
        definition,
        &Timestamp(END.into()),
        &RunId("run_adv".into()),
    )
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn wrap(operation: &str, revision: &str, document: &Value) -> Value {
    json!({
        "ok": true,
        "result": {
            "adapter": "logs",
            "operation": operation,
            "revision": revision,
            "result": document.to_string(),
        }
    })
}

/// A `logs.query_metric` instant answer as Connectors' `query_metric` builds it: `complete` is
/// false exactly when `truncation` names a cause.
fn metric_document(series: &[(Value, &str)], truncation: &[&str]) -> Value {
    let series: Vec<Value> = series
        .iter()
        .map(|(labels, value)| {
            json!({"labels": labels, "samples": [{"timestamp_unix_ns": END_NS, "value": value}]})
        })
        .collect();
    json!({
        "result_type": "vector",
        "series": series,
        "complete": truncation.is_empty(),
        "truncation": truncation,
        "provenance": {"instance": "loki-test", "profile": "logql-metric", "observed_at_unix_ms": 1_792_018_623_000_u64},
    })
}

fn metric_answer(series: &[(Value, &str)], truncation: &[&str]) -> Value {
    json!({"stdout": wrap("logs.query_metric", "op-rev-metric-1", &metric_document(series, truncation))})
}

fn ratio_answers(numerator: &[(Value, &str)], denominator: &[(Value, &str)]) -> Value {
    json!([
        {"input": {"query": NUMERATOR}, "stdout": wrap("logs.query_metric", "op-rev-metric-1", &metric_document(numerator, &[]))},
        {"input": {"query": DENOMINATOR}, "stdout": wrap("logs.query_metric", "op-rev-metric-1", &metric_document(denominator, &[]))},
    ])
}

/// LogQL's `count_over_time` emits no point for a stream with no entry in its range, and `sum by`
/// of nothing is nothing: a cluster that logged lines and no error is absent from the numerator's
/// complete (exhaustive, Loki profile § 11.1) answer and present in the denominator's. That is
/// zero errors over its lines, read completely. It is the answer every healthy cluster gives the
/// design's own failure-ratio signal (§ 2.1), and nothing in the story, the design or the
/// Connectors profile makes it unread.
#[test]
fn adv_loki_ratio_cluster_without_errors_is_zero_not_unknown() {
    let w = World::new("adv_loki_ratio_cluster_without_errors_is_zero_not_unknown");
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &[(json!({"cluster": "alpha"}), "15")],
            &[
                (json!({"cluster": "alpha"}), "556"),
                (json!({"cluster": "gamma"}), "8"),
            ],
        ),
    );

    let reads = read(&w, &ratio_definition());

    let by_cluster: BTreeMap<&str, &Read> = reads
        .iter()
        .map(|r| (r.dims["cluster"].as_str(), r))
        .collect();
    assert_eq!(by_cluster["alpha"].reading, ok("0.02697841726618705"));
    assert_eq!(
        by_cluster["gamma"].reading,
        ok("0"),
        "gamma logged 8 lines and no error: {:?}",
        by_cluster["gamma"]
    );
    assert_eq!(
        by_cluster["gamma"].denominator.as_ref().map(|p| p.n),
        Some(8)
    );
}

/// Loki renders a sample with Go's `FormatFloat(v, 'f', -1, 64)`, which spells negative zero
/// `-0`. A ratio is rendered "never `-0`" (`render` in `src/loki.rs`); a value is the provider's
/// text unchanged, so the same number reaches the store under two renderings, and `Decimal`
/// compares by rendering.
#[test]
fn adv_loki_value_negative_zero_is_not_a_second_zero() {
    let w = World::new("adv_loki_value_negative_zero_is_not_a_second_zero");
    w.record(
        METRIC_INVOKE,
        &metric_answer(&[(json!({"cluster": "alpha"}), "-0")], &[]),
    );
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(reads[0].reading, ok("0"), "{reads:?}");
}

/// Two series that differ only in a label the definition does not declare fall onto one dims.
/// The unit's suite has no answer that does this, so `one_of`'s refusal is never exercised: a
/// copy whose `one_of` takes the first series stays green under `tests/loki.rs`.
#[test]
fn adv_loki_value_series_colliding_on_declared_dims_are_unknown() {
    let w = World::new("adv_loki_value_series_colliding_on_declared_dims_are_unknown");
    w.record(
        METRIC_INVOKE,
        &metric_answer(
            &[
                (json!({"cluster": "alpha", "app": "api"}), "3"),
                (json!({"cluster": "alpha", "app": "web"}), "4"),
            ],
            &[],
        ),
    );
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 1, "{reads:?}");
    match &reads[0].reading {
        Reading::Unknown(unread) => assert!(unread.reason.contains("2 series"), "{reads:?}"),
        other => panic!("two series on one dims became {other:?}"),
    }
}

/// Connectors cuts an instant answer at 500 series with cause `series_limit` and `complete:
/// false` (Loki profile § 11.1). The unit's suite has no incomplete metric answer, so a copy that
/// ignores `complete` on `logs.query_metric` stays green under `tests/loki.rs`, and 500 of an
/// unknown number of series are then reported complete.
#[test]
fn adv_loki_metric_series_limit_is_not_reported_complete() {
    let w = World::new("adv_loki_metric_series_limit_is_not_reported_complete");
    let series: Vec<(Value, &str)> = (0..500)
        .map(|i| (json!({"cluster": format!("c{i:03}")}), "1"))
        .collect();
    w.record(METRIC_INVOKE, &metric_answer(&series, &["series_limit"]));
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    assert!(
        reads
            .iter()
            .all(|r| matches!(r.reading, Reading::Unknown(_))),
        "a cut metric answer was read as {} complete series",
        reads
            .iter()
            .filter(|r| matches!(r.reading, Reading::Ok(_)))
            .count()
    );
}

/// Connectors keeps the first 500 stream groups and calls the answer incomplete with cause
/// `stream_limit` (Loki profile § 4.3, § 5); 501 groups of one line each return 500 lines, below
/// the limit. The unit's suite cuts only answers at the 1,000-line cap, so a copy that tests the
/// line count alone stays green under `tests/loki.rs` and reports this count complete.
#[test]
fn adv_loki_count_stream_limit_below_line_cap_is_partial() {
    let w = World::new("adv_loki_count_stream_limit_below_line_cap_is_partial");
    let lines: Vec<Value> = (0..500)
        .map(|i| {
            json!({
                "timestamp_unix_ns": format!("{}", 1_792_018_622_000_000_000_u64 - i as u64),
                "stream": {"app": "api", "pod": format!("p{i:03}")},
                "line": "request failed",
                "line_truncated": false,
                "redacted": false,
                "source": null,
            })
        })
        .collect();
    let document = json!({
        "lines": lines,
        "selection": {"kind": "loki-range", "start_unix_ns": START_NS, "end_unix_ns": END_NS, "direction": "backward"},
        "order": "timestamp-backward",
        "complete": false,
        "truncation": {"causes": ["stream_limit"], "occurrences_dropped": 1, "stream_groups_dropped": 1},
        "next_cursor": null,
        "provenance": {"instance": "loki-test", "resource": "loki:query_range", "observed_at_unix_ms": 1_792_018_623_000_u64, "source_revision": null},
    });
    w.record(
        RANGE_INVOKE,
        &json!({"stdout": wrap("logs.query_range", "op-rev-range-1", &document)}),
    );
    let definition = definition(
        "loki.count",
        SignalKind::CountOfCases,
        "{line}",
        &[],
        &[("value", LINES_QUERY)],
    );

    let reads = read(&w, &definition);

    assert_eq!(reads.len(), 1, "{reads:?}");
    assert_eq!(
        reads[0].reading,
        Reading::Partial(PartialReading {
            value: Decimal("500".into()),
            bound: Bound::Lower,
        })
    );
}

/// A numerator series and a denominator series whose label sets differ (`app="api"` against
/// `app="web"`) are divided because the undeclared label is dropped before they are paired.
/// LogQL's own `/` matches identical label sets only. Constructed: no realistic definition was
/// found that produces it.
#[test]
fn adv_loki_ratio_pairs_only_series_with_the_same_labels() {
    let w = World::new("adv_loki_ratio_pairs_only_series_with_the_same_labels");
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &[(json!({"cluster": "alpha", "app": "api"}), "15")],
            &[(json!({"cluster": "alpha", "app": "web"}), "556")],
        ),
    );

    let reads = read(&w, &ratio_definition());

    assert!(
        reads
            .iter()
            .all(|r| matches!(r.reading, Reading::Unknown(_))),
        "series of different label sets were divided: {reads:?}"
    );
}
