//! Adversary cases for the Loki adapter (wave 20261015c, pass 2), against the correction that
//! reads a label set absent from a complete numerator answer as numerator 0. Answers are shaped as
//! `adapters/loki/src/lib.rs` of Connectors v0.45.0 builds them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";
const END_NS: &str = "1792018623000000000";
const VALUE_QUERY: &str = r#"sum by (cluster) (count_over_time({app="api"} |= "error" [6h]))"#;
const NUMERATOR: &str = r#"sum(count_over_time({app="api"} |= "error" [6h]))"#;
const DENOMINATOR: &str = r#"sum by (cluster) (count_over_time({app="api"} [6h]))"#;
const METRIC_INVOKE: &str = "operations+invoke+a=logs+c=logs-main+o=logs.query_metric.json";

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv2_loki")
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

fn ratio_definition(dimensions: &[&str]) -> SignalDefinitionData {
    let mut definition = definition(
        "loki.ratio",
        SignalKind::Ratio,
        "1",
        dimensions,
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
        &RunId("run_adv2".into()),
    )
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn wrap(document: &Value) -> Value {
    json!({
        "ok": true,
        "result": {
            "adapter": "logs",
            "operation": "logs.query_metric",
            "revision": "op-rev-metric-1",
            "result": document.to_string(),
        }
    })
}

/// A `logs.query_metric` instant answer as Connectors builds it: `complete` is false exactly when
/// `truncation` names a cause.
fn metric_document(series: &[(Value, String)], truncation: &[&str]) -> Value {
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

fn s(labels: Value, value: &str) -> (Value, String) {
    (labels, value.to_string())
}

fn ratio_answers(
    numerator: &[(Value, String)],
    numerator_cut: &[&str],
    denominator: &[(Value, String)],
) -> Value {
    json!([
        {"input": {"query": NUMERATOR}, "stdout": wrap(&metric_document(numerator, numerator_cut))},
        {"input": {"query": DENOMINATOR}, "stdout": wrap(&metric_document(denominator, &[]))},
    ])
}

fn shown(reads: &[Read]) -> Vec<(BTreeMap<String, String>, Reading)> {
    reads
        .iter()
        .map(|r| (r.dims.clone(), r.reading.clone()))
        .collect()
}

/// The numerator aggregates over no label (`sum(...)`), the denominator `by (cluster)`: an
/// authoring slip LogQL's own `/` would answer with no series. The numerator's one series `{}` is
/// paired with nothing, and every denominator label set then reads numerator 0, so clusters that
/// logged 15 errors between them are each graded on 0 errors. Absence is evidence of zero only
/// when the two sides' label sets line up; here they do not.
#[test]
fn adv2_loki_ratio_numerator_without_by_is_not_zero_per_cluster() {
    let w = World::new("adv2_loki_ratio_numerator_without_by_is_not_zero_per_cluster");
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &[s(json!({}), "15")],
            &[],
            &[
                s(json!({"cluster": "alpha"}), "556"),
                s(json!({"cluster": "gamma"}), "8"),
            ],
        ),
    );

    let reads = read(&w, &ratio_definition(&["cluster"]));

    assert!(
        !reads.iter().any(|r| r.reading == ok("0")),
        "15 errors were read as 0 per cluster: {:?}",
        shown(&reads)
    );
}

/// The numerator is `by (cluster)`, the denominator `by (cluster, namespace)`, and the definition
/// declares both. The numerator's `{cluster: alpha}` pairs with nothing; each namespace of alpha
/// then reads numerator 0 although alpha logged 15 errors.
#[test]
fn adv2_loki_ratio_numerator_coarser_than_denominator_is_not_zero() {
    let w = World::new("adv2_loki_ratio_numerator_coarser_than_denominator_is_not_zero");
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &[s(json!({"cluster": "alpha"}), "15")],
            &[],
            &[
                s(json!({"cluster": "alpha", "namespace": "shop"}), "500"),
                s(json!({"cluster": "alpha", "namespace": "web"}), "56"),
            ],
        ),
    );

    let reads = read(&w, &ratio_definition(&["cluster", "namespace"]));

    assert!(
        !reads.iter().any(|r| r.reading == ok("0")),
        "alpha's 15 errors were read as 0 in each namespace: {:?}",
        shown(&reads)
    );
}

/// A numerator answer cut at 500 series (`series_limit`) proves nothing about the series it left
/// out, so no denominator label set may read numerator 0 from it. The unit's suite has no
/// incomplete numerator: a copy that reads a failed numerator as an empty one stays green there
/// and reads `c900` as 0 here.
#[test]
fn adv2_loki_ratio_incomplete_numerator_is_not_read_as_absent() {
    let w = World::new("adv2_loki_ratio_incomplete_numerator_is_not_read_as_absent");
    let numerator: Vec<(Value, String)> = (0..500)
        .map(|i| s(json!({"cluster": format!("c{i:03}")}), "1"))
        .collect();
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &numerator,
            &["series_limit"],
            &[
                s(json!({"cluster": "c000"}), "10"),
                s(json!({"cluster": "c900"}), "10"),
            ],
        ),
    );

    let reads = read(&w, &ratio_definition(&["cluster"]));

    assert!(
        reads
            .iter()
            .all(|r| matches!(r.reading, Reading::Unknown(_))),
        "a cut numerator was read: {:?}",
        shown(&reads)
    );
}

/// Two full-label pairs (`app="api"`: 15/100, `app="web"`: absent/200) project onto one declared
/// dims. That is one `unknown` read naming the collision, without a population. The unit's suite
/// has no ratio collision, and pass 1's pairing case orders its unknown pair first, so a copy that
/// keeps the first pair stays green there and reads 0.15 here.
#[test]
fn adv2_loki_ratio_two_pairs_on_one_dims_are_one_unknown() {
    let w = World::new("adv2_loki_ratio_two_pairs_on_one_dims_are_one_unknown");
    w.record(
        METRIC_INVOKE,
        &ratio_answers(
            &[s(json!({"cluster": "alpha", "app": "api"}), "15")],
            &[],
            &[
                s(json!({"cluster": "alpha", "app": "api"}), "100"),
                s(json!({"cluster": "alpha", "app": "web"}), "200"),
            ],
        ),
    );

    let reads = read(&w, &ratio_definition(&["cluster"]));

    assert_eq!(reads.len(), 1, "{:?}", shown(&reads));
    match &reads[0].reading {
        Reading::Unknown(unread) => assert!(unread.reason.contains("2 series"), "{unread:?}"),
        other => panic!("two pairs on one dims became {other:?}"),
    }
    assert_eq!(reads[0].denominator, None);
}

/// Canonicalisation at the lengths Go's `FormatFloat(v, 'f', -1, 64)` reaches: a 309-digit whole
/// number, the smallest subnormal (323 zeros after the point), and long zero tails.
#[test]
fn adv2_loki_value_long_decimals_render_once() {
    let huge = format!("1{}", "0".repeat(308));
    let tiny = format!("0.{}5", "0".repeat(323));
    let cases = [
        ("a", huge.clone(), huge.clone()),
        ("b", tiny.clone(), tiny.clone()),
        ("c", format!("7.5{}", "0".repeat(40)), "7.5".to_string()),
        ("d", format!("-0.{}", "0".repeat(40)), "0".to_string()),
        ("e", format!("-{tiny}"), format!("-{tiny}")),
        ("f", "100.000".to_string(), "100".to_string()),
    ];
    let w = World::new("adv2_loki_value_long_decimals_render_once");
    let series: Vec<(Value, String)> = cases
        .iter()
        .map(|(cluster, text, _)| (json!({"cluster": cluster}), text.clone()))
        .collect();
    w.record(
        METRIC_INVOKE,
        &json!({"stdout": wrap(&metric_document(&series, &[]))}),
    );
    let definition = definition(
        "loki.value",
        SignalKind::Gauge,
        "{line}",
        &["cluster"],
        &[("value", VALUE_QUERY)],
    );

    let reads = read(&w, &definition);

    let by_cluster: BTreeMap<&str, &Read> = reads
        .iter()
        .map(|r| (r.dims["cluster"].as_str(), r))
        .collect();
    for (cluster, text, rendered) in &cases {
        let read = by_cluster[cluster];
        assert_eq!(read.reading, ok(rendered), "{cluster}");
        assert_eq!(read.raw.as_deref(), Some(text.as_str()), "{cluster}");
    }
}
