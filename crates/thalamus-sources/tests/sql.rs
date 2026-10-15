//! The `sql` adapter, read through the registry against recorded Connectors `query.read` answers
//! (`tests/fixtures/sql/<scenario>`, replayed by `src/bin/connectors-double.rs`).
//!
//! Each test copies the recordings of `common` and of its scenario into a fresh directory under
//! the build's own temporary directory and links `connectors` there to the double.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, QueryDigest, Reading, RunId,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Population, Read, query_digest};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;
use thalamus_sources::sql::ROW_LIMIT;

const VALUE_QUERY: &str = "SELECT cluster, value FROM queue_depth";
const DIMS_QUERY: &str = "SELECT cluster, tenant, value FROM queue_depth";
const NUMERATOR: &str = "SELECT count(*) FROM requests WHERE failed";
const DENOMINATOR: &str = "SELECT count(*) FROM requests";
const OPEN: &str = "SELECT id FROM open_incidents";
const RESOLVED: &str = "SELECT id FROM resolved_incidents";
const OPEN_BY_CLUSTER: &str = "SELECT cluster, id FROM open_incidents";
const RESOLVED_BY_CLUSTER: &str = "SELECT cluster, tenant, id FROM resolved_incidents";
const NUMERATOR_BY_CLUSTER: &str =
    "SELECT cluster, count(*) AS value FROM requests WHERE failed GROUP BY cluster";
const DENOMINATOR_BY_CLUSTER: &str =
    "SELECT cluster, count(*) AS value FROM requests GROUP BY cluster";
/// SHA-256 of `{"value":"SELECT cluster, value FROM queue_depth"}`, computed outside thalamus.
const VALUE_QUERY_SHA256: &str = "a4a4465b1c0f80686acd8fa0628ea4e4e8028a09bc870c9d2d48f91be09d93b3";
const END: &str = "2026-10-14T22:57:03Z";
const START: &str = "2026-10-14T16:57:03Z";

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str, scenario: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("sql")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sql");
        for scenario in ["common", scenario] {
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

    fn read(&self, definition: &SignalDefinitionData) -> Vec<Read> {
        registry::read(
            &Connectors::new(self.dir.join("connectors")),
            definition,
            &Timestamp(END.into()),
            &RunId("run_test".into()),
        )
    }

    /// The `--input-json` document of every `operations invoke` the double received.
    fn invoke_inputs(&self) -> Vec<Value> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect();
                if argv.first().map(String::as_str) != Some("operations")
                    || argv.get(1).map(String::as_str) != Some("invoke")
                {
                    return None;
                }
                let at = argv.iter().position(|a| a == "--input-json")?;
                serde_json::from_str(argv.get(at + 1)?).ok()
            })
            .collect()
    }
}

fn definition(
    extractor: &str,
    kind: SignalKind,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.queue_depth".into()),
        title: "Queue depth".into(),
        family: Family("platform".into()),
        kind,
        unit: Some(Unit("1".into())),
        dimensions: Some(
            dimensions
                .iter()
                .map(|k| DimensionKey((*k).into()))
                .collect(),
        ),
        source: Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("reports-replica".into()),
        },
        extractor: ExtractorId(extractor.into()),
        query: query
            .iter()
            .map(|(k, v)| ((*k).to_string(), QueryText((*v).into())))
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

fn value_definition(query: &str, dimensions: &[&str]) -> SignalDefinitionData {
    definition(
        "sql.value",
        SignalKind::Gauge,
        &[("value", query)],
        dimensions,
    )
}

fn ratio_definition() -> SignalDefinitionData {
    let mut definition = definition(
        "sql.ratio",
        SignalKind::Ratio,
        &[("numerator", NUMERATOR), ("denominator", DENOMINATOR)],
        &[],
    );
    definition.denominator = Some(DenominatorSpec {
        unit: Unit("{request}".into()),
        min_n: 20,
        since: "first_sample".into(),
    });
    definition
}

/// The read whose `cluster` dimension is `cluster`.
fn by_cluster<'a>(reads: &'a [Read], cluster: &str) -> &'a Read {
    reads
        .iter()
        .find(|r| r.dims.get("cluster").map(String::as_str) == Some(cluster))
        .unwrap_or_else(|| panic!("no read for cluster {cluster} in {reads:#?}"))
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

/// No reading anywhere is the number 0: unknown is never 0.
fn assert_no_zero(reads: &[Read]) {
    for read in reads {
        let value = match &read.reading {
            Reading::Ok(r) => Some(&r.value.0),
            Reading::Partial(r) => Some(&r.value.0),
            Reading::Unknown(_) => None,
        };
        assert!(
            value.is_none_or(|v| !v
                .trim_start_matches('-')
                .trim_matches(['0', '.'])
                .is_empty()),
            "a reading is 0: {read:#?}"
        );
    }
}

#[test]
fn sql_null_cell_is_unknown_never_zero() {
    let w = World::new("sql_null_cell_is_unknown_never_zero", "null-cell");
    let reads = w.read(&value_definition(VALUE_QUERY, &["cluster"]));

    assert_eq!(reads.len(), 2, "one read per row: {reads:#?}");
    assert_eq!(by_cluster(&reads, "alpha").reading, ok("4"));
    let beta = by_cluster(&reads, "beta");
    assert!(
        reason(beta).contains("NULL"),
        "the reason names the NULL cell: {}",
        reason(beta)
    );
    assert_eq!(beta.raw, None, "a NULL cell has no raw text");
    assert_no_zero(&reads);
}

#[test]
fn sql_non_numeric_cell_is_unknown() {
    let w = World::new("sql_non_numeric_cell_is_unknown", "non-numeric");
    let reads = w.read(&value_definition(VALUE_QUERY, &["cluster"]));

    assert_eq!(reads.len(), 5, "one read per row: {reads:#?}");
    for (cluster, text) in [
        ("alpha", "green"),
        ("beta", "NaN"),
        ("gamma", "Infinity"),
        ("delta", ""),
        ("epsilon", "12 ms"),
    ] {
        let read = by_cluster(&reads, cluster);
        assert!(
            reason(read).contains("not a number"),
            "{cluster}: the reason says the cell is not a number: {}",
            reason(read)
        );
        assert_eq!(
            read.raw.as_deref(),
            Some(text),
            "{cluster}: raw is the cell"
        );
    }
    assert_no_zero(&reads);
}

#[test]
fn sql_ratio_text_cells_give_exact_decimal_with_unit_1() {
    let w = World::new(
        "sql_ratio_text_cells_give_exact_decimal_with_unit_1",
        "ratio",
    );
    let definition = ratio_definition();
    let reads = w.read(&definition);

    assert_eq!(reads.len(), 1, "one series: {reads:#?}");
    let read = &reads[0];
    // 15 / 556 = 0.026978417266187050|3597…: 18 fractional digits, the next digit 3 rounds down,
    // the trailing 0 is dropped.
    assert_eq!(read.reading, ok("0.02697841726618705"));
    assert_eq!(definition.unit, Some(Unit("1".into())));
    assert_eq!(
        read.denominator,
        Some(Population {
            n: 556,
            unit: Unit("{request}".into()),
        })
    );
    assert!(read.dims.is_empty());
    assert_eq!(read.query_sha256, query_digest(&definition.query));
    let queries: Vec<Value> = w
        .invoke_inputs()
        .iter()
        .map(|input| input["query"].clone())
        .collect();
    assert_eq!(
        queries,
        [NUMERATOR, DENOMINATOR],
        "numerator, then denominator"
    );

    // The quotient has unit 1: a ratio definition declaring another unit is not read as one.
    let mut percent = ratio_definition();
    percent.unit = Some(Unit("%".into()));
    let w = World::new("sql_ratio_unit_other_than_1", "ratio");
    let reads = w.read(&percent);
    assert_eq!(reads.len(), 1);
    assert!(
        reason(&reads[0]).contains("unit"),
        "the reason names the unit: {}",
        reason(&reads[0])
    );
}

#[test]
fn sql_truncated_result_is_partial() {
    let w = World::new("sql_truncated_result_is_partial", "truncated");

    // The recordings are cut as Connectors cuts: at exactly the requested limit.
    // `sql.count` of one series counts the rows the query returns; a cut result holds at least
    // that many.
    let open = w.read(&definition(
        "sql.count",
        SignalKind::CountOfCases,
        &[("value", OPEN)],
        &[],
    ));
    assert_eq!(open.len(), 1, "{open:#?}");
    assert_eq!(
        open[0].reading,
        Reading::Partial(PartialReading {
            value: Decimal(ROW_LIMIT.to_string()),
            bound: Bound::Lower,
        })
    );
    let limits: Vec<Value> = w
        .invoke_inputs()
        .iter()
        .map(|i| i["limit"].clone())
        .collect();
    assert_eq!(limits, [Value::from(ROW_LIMIT)], "the limit asked for");

    let resolved = w.read(&definition(
        "sql.count",
        SignalKind::CountOfCases,
        &[("value", RESOLVED)],
        &[],
    ));
    assert_eq!(resolved.len(), 1, "{resolved:#?}");
    assert_eq!(resolved[0].reading, ok("2"), "an uncut result is complete");

    // A cut answer of many series is one unknown read naming the cut: the series past it are not
    // read, and the ones before it are not passed off as the whole answer.
    let values = w.read(&value_definition(VALUE_QUERY, &["cluster"]));
    assert_eq!(values.len(), 1, "{values:#?}");
    assert!(values[0].dims.is_empty());
    let why = reason(&values[0]);
    assert!(
        why.contains(&format!("cut at {ROW_LIMIT} rows")),
        "the reason names the cut: {why}"
    );

    let by_cluster_cut = w.read(&definition(
        "sql.count",
        SignalKind::CountOfCases,
        &[("value", OPEN_BY_CLUSTER)],
        &["cluster"],
    ));
    assert_eq!(by_cluster_cut.len(), 1, "{by_cluster_cut:#?}");
    assert!(
        reason(&by_cluster_cut[0]).contains(&format!("cut at {ROW_LIMIT} rows")),
        "a cut count of many series names the cut: {}",
        reason(&by_cluster_cut[0])
    );
}

#[test]
fn sql_count_groups_rows_by_declared_dimensions() {
    let w = World::new("sql_count_groups_rows_by_declared_dimensions", "count-dims");
    // Declared: cluster. Returned: cluster, tenant and id; tenant is not declared.
    let reads = w.read(&definition(
        "sql.count",
        SignalKind::CountOfCases,
        &[("value", RESOLVED_BY_CLUSTER)],
        &["cluster"],
    ));

    assert_eq!(reads.len(), 3, "one read per series: {reads:#?}");
    assert_eq!(by_cluster(&reads, "alpha").reading, ok("2"));
    assert_eq!(by_cluster(&reads, "beta").reading, ok("1"));
    let unkeyed = reads
        .iter()
        .find(|r| r.dims.is_empty())
        .expect("the row with a NULL cluster counts in the series with no cluster key");
    assert_eq!(unkeyed.reading, ok("1"));
}

#[test]
fn sql_ratio_series_absent_from_complete_numerator_reads_0() {
    let w = World::new(
        "sql_ratio_series_absent_from_complete_numerator_reads_0",
        "ratio-dims",
    );
    let reads = w.read(&ratio_by_cluster());

    assert_eq!(reads.len(), 2, "{reads:#?}");
    assert_eq!(by_cluster(&reads, "alpha").reading, ok("0.1"));
    // A grouped count returns no row for a group with no case: a complete numerator answer
    // without the series is 0 cases of the denominator's 40.
    let beta = by_cluster(&reads, "beta");
    assert_eq!(beta.reading, ok("0"));
    assert_eq!(
        beta.denominator,
        Some(Population {
            n: 40,
            unit: Unit("{request}".into()),
        })
    );
}

#[test]
fn sql_ratio_numerator_series_without_denominator_twin_is_one_unknown() {
    // Design § 11 D20: numerator 0 for an absent series holds only when every numerator series
    // has a denominator twin. Here `gamma` has none, so no series is read.
    let w = World::new(
        "sql_ratio_numerator_series_without_denominator_twin_is_one_unknown",
        "ratio-unmatched",
    );
    let reads = w.read(&ratio_by_cluster());

    assert_eq!(reads.len(), 1, "one read for the whole pass: {reads:#?}");
    assert!(reads[0].dims.is_empty());
    let why = reason(&reads[0]);
    assert!(
        why.contains("gamma") && !why.contains("alpha") && !why.contains("beta"),
        "the reason names the unmatched numerator series, and only it: {why}"
    );
    assert_no_zero(&reads);
}

/// `sql.ratio` grouped by `cluster`.
fn ratio_by_cluster() -> SignalDefinitionData {
    let mut definition = ratio_definition();
    definition.query = BTreeMap::from([
        (
            "numerator".to_string(),
            QueryText(NUMERATOR_BY_CLUSTER.into()),
        ),
        (
            "denominator".to_string(),
            QueryText(DENOMINATOR_BY_CLUSTER.into()),
        ),
    ]);
    definition.dimensions = Some(vec![DimensionKey("cluster".into())]);
    definition
}

#[test]
fn sql_dims_only_from_returned_columns() {
    let w = World::new("sql_dims_only_from_returned_columns", "dims");
    // Declared: cluster and namespace. Returned: cluster, tenant and value.
    let reads = w.read(&value_definition(DIMS_QUERY, &["cluster", "namespace"]));

    assert_eq!(reads.len(), 2, "one read per row: {reads:#?}");
    assert_eq!(
        reads[0].dims,
        BTreeMap::from([("cluster".to_string(), "alpha".to_string())]),
        "cluster is declared and returned; namespace is not returned; tenant is not declared"
    );
    assert_eq!(reads[0].reading, ok("7"));
    assert_eq!(
        reads[1].dims,
        BTreeMap::new(),
        "a NULL cluster cell sets no key"
    );
    assert_eq!(reads[1].reading, ok("9"));
}

#[test]
fn sql_read_keeps_raw_cell_text() {
    let w = World::new("sql_read_keeps_raw_cell_text", "raw");
    let definition = value_definition(VALUE_QUERY, &["cluster"]);
    let reads = w.read(&definition);

    assert_eq!(reads.len(), 5, "one read per row: {reads:#?}");
    for (cluster, raw, value) in [
        ("alpha", "0.0250", "0.0250"),
        ("beta", "1e-4", "0.0001"),
        ("gamma", "-1.5e+3", "-1500"),
        (
            "delta",
            "123456789012345678901234567890.000000000000000000001",
            "123456789012345678901234567890.000000000000000000001",
        ),
    ] {
        let read = by_cluster(&reads, cluster);
        assert_eq!(read.raw.as_deref(), Some(raw), "{cluster}: raw is the cell");
        assert_eq!(read.reading, ok(value), "{cluster}: the value is exact");
    }
    let epsilon = by_cluster(&reads, "epsilon");
    assert_eq!(epsilon.raw, None, "a NULL cell has no raw text");
    assert!(matches!(epsilon.reading, Reading::Unknown(_)));

    for read in &reads {
        assert_eq!(
            read.query_sha256,
            QueryDigest(VALUE_QUERY_SHA256.into()),
            "the digest of the query map's canonical JSON"
        );
        assert_eq!(read.window.start, Timestamp(START.into()));
        assert_eq!(read.window.end, Timestamp(END.into()));
        assert_eq!(read.adapter, AdapterId("sql".into()));
        assert_eq!(read.connection, ConnectionId("reports-replica".into()));
    }
    let inputs = w.invoke_inputs();
    assert_eq!(inputs.len(), 1, "one query.read: {inputs:#?}");
    assert_eq!(inputs[0]["query"], VALUE_QUERY);
    assert_eq!(inputs[0]["limit"], ROW_LIMIT);
}

#[test]
fn sql_refused_read_is_unknown_with_reason() {
    let w = World::new("sql_refused_read_is_unknown_with_reason", "refused");
    let definition = value_definition(VALUE_QUERY, &["cluster"]);
    let reads = w.read(&definition);

    assert_eq!(reads.len(), 1, "{reads:#?}");
    let why = reason(&reads[0]);
    assert!(
        why.contains("forbidden") && why.contains("dispatch"),
        "the reason carries Connectors' code and stage: {why}"
    );
    assert!(
        why.contains("query.read"),
        "the reason names the call: {why}"
    );
    assert_eq!(reads[0].raw, None);
    assert_eq!(reads[0].query_sha256, query_digest(&definition.query));
}

#[test]
fn sql_series_returned_twice_is_unknown() {
    let w = World::new("sql_series_returned_twice_is_unknown", "duplicate");
    let reads = w.read(&value_definition(VALUE_QUERY, &["cluster"]));

    assert_eq!(reads.len(), 2, "one read per series: {reads:#?}");
    let alpha = by_cluster(&reads, "alpha");
    assert!(
        reason(alpha).contains("twice"),
        "neither of two rows of one series is its value: {}",
        reason(alpha)
    );
    assert_eq!(alpha.raw, None);
    assert_eq!(by_cluster(&reads, "beta").reading, ok("3"));
}
