//! The Prometheus adapter (`promql.value`, `promql.ratio`), run through the extractor registry
//! against recorded Connectors answers and the double (`src/bin/connectors-double.rs`).
//!
//! The recordings in `tests/fixtures/prometheus` are what Connectors' `prometheus` adapter answers
//! `series.query` (an instant query): the CLI envelope carries the adapter's document as a string,
//! sample values stay Prometheus's strings. Each answer is chosen by the query text it was asked,
//! so every test asks its own queries. Each test copies the recordings into a fresh directory under
//! the build's temporary directory and links `connectors` there to the double.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Reading, RunId};
use thalamus_model::primitives::{Duration, Timestamp};
use thalamus_sources::adapter::{Population, Read};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";
/// `END` in Unix seconds.
const END_UNIX_S: i64 = 1_792_018_623;
const NUMERATOR: &str = "sum by (cluster) (increase(requests_failed_total[6h]))";
const DENOMINATOR: &str = "sum by (cluster) (increase(requests_total[6h]))";

/// One test's directory: the recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("prometheus")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prometheus");
        for entry in std::fs::read_dir(fixtures).expect("the recordings exist") {
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

    fn connectors(&self) -> Connectors {
        Connectors::new(self.dir.join("connectors"))
    }

    /// The `--input-json` document of every `operations invoke`, in order.
    fn invoked(&self) -> Vec<Value> {
        let log = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        log.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("one JSON call per line"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect();
                if argv.get(..2) != Some(&["operations".to_string(), "invoke".to_string()]) {
                    return None;
                }
                let at = argv.iter().position(|a| a == "--input-json")?;
                serde_json::from_str(argv.get(at + 1)?).ok()
            })
            .collect()
    }

    /// The reads of `definition` for the window ending at `end`, through the registry.
    fn read_at(&self, definition: &SignalDefinitionData, end: &str) -> Vec<Read> {
        registry::read(
            &self.connectors(),
            definition,
            &Timestamp(end.into()),
            &RunId("run_test".into()),
        )
    }

    fn read(&self, definition: &SignalDefinitionData) -> Vec<Read> {
        self.read_at(definition, END)
    }
}

fn definition(
    extractor: &str,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    let ratio = extractor == "promql.ratio";
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.probe".into()),
        title: "A probe signal".into(),
        family: Family("platform".into()),
        kind: if ratio {
            SignalKind::Ratio
        } else {
            SignalKind::Gauge
        },
        unit: Some(Unit(if ratio { "1" } else { "{item}" }.into())),
        dimensions: Some(
            dimensions
                .iter()
                .map(|d| DimensionKey((*d).into()))
                .collect(),
        ),
        source: Source {
            adapter: AdapterId("metrics".into()),
            connection: ConnectionId("metrics-main".into()),
        },
        extractor: ExtractorId(extractor.into()),
        query: query
            .iter()
            .map(|(k, v)| ((*k).to_string(), QueryText((*v).into())))
            .collect(),
        window: Duration("PT6H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Source,
        denominator: ratio.then(|| DenominatorSpec {
            unit: Unit("{request}".into()),
            min_n: 20,
            since: "first_sample".into(),
        }),
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

fn value(query: &str, dimensions: &[&str]) -> SignalDefinitionData {
    definition("promql.value", &[("value", query)], dimensions)
}

fn ratio(numerator: &str, denominator: &str) -> SignalDefinitionData {
    definition(
        "promql.ratio",
        &[("numerator", numerator), ("denominator", denominator)],
        &["cluster"],
    )
}

fn dims(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// The read of the series whose `cluster` is `cluster`; exactly one.
fn of<'a>(reads: &'a [Read], cluster: &str) -> &'a Read {
    let mut found = reads
        .iter()
        .filter(|r| r.dims.get("cluster").map(String::as_str) == Some(cluster));
    let read = found
        .next()
        .unwrap_or_else(|| panic!("no read for cluster {cluster}: {reads:#?}"));
    assert!(
        found.next().is_none(),
        "two reads for {cluster}: {reads:#?}"
    );
    read
}

fn value_text(read: &Read) -> &str {
    match &read.reading {
        Reading::Ok(ok) => &ok.value.0,
        other => panic!("expected an ok reading, got {other:?} in {read:#?}"),
    }
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?} in {read:#?}"),
    }
}

fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn ratio_reading_is_a_decimal_with_unit_1() {
    let w = World::new("ratio_reading_is_a_decimal_with_unit_1");
    let reads = w.read(&ratio(NUMERATOR, DENOMINATOR));
    assert_eq!(reads.len(), 2, "{reads:#?}");

    // 15 failures of 556 requests: a fraction of 1, not a percentage and not Prometheus's text.
    let alpha = of(&reads, "alpha");
    let text = value_text(alpha);
    assert!(
        text.starts_with("0.0269"),
        "15 / 556 is 0.0269…, got {text}"
    );
    let parsed: f64 = text.parse().expect("the value is a decimal number");
    assert!((parsed - 15.0 / 556.0).abs() < 1e-15, "{text}");
    assert!(
        !text.starts_with("2.69") && !text.contains('%') && !text.contains('/'),
        "{text}"
    );
    assert_eq!(
        alpha.denominator,
        Some(Population {
            n: 556,
            unit: Unit("{request}".into())
        })
    );

    // `increase()` answers fractional counts: the population is the denominator rounded down.
    let beta = of(&reads, "beta");
    let parsed: f64 = value_text(beta).parse().expect("a decimal");
    assert!((parsed - 3.0 / 120.9).abs() < 1e-15, "{beta:#?}");
    assert_eq!(beta.denominator.as_ref().map(|p| p.n), Some(120));

    // Both queries were asked as instant queries at the window end, nothing but reads.
    let invoked = w.invoked();
    let queries: Vec<&str> = invoked.iter().filter_map(|i| i["query"].as_str()).collect();
    assert_eq!(queries, [NUMERATOR, DENOMINATOR], "{invoked:?}");
    for input in &invoked {
        assert_eq!(input["time_unix_s"], Value::from(END_UNIX_S), "{input}");
    }

    // A ratio is a fraction with unit 1: a definition declaring another unit would scale it, so it
    // is not read as one.
    let mut percent = ratio(NUMERATOR, DENOMINATOR);
    percent.unit = Some(Unit("%".into()));
    let reads = w.read(&percent);
    assert!(!reads.is_empty());
    for read in &reads {
        let why = reason(read);
        assert!(why.contains("unit") && why.contains('%'), "{why}");
    }
}

#[test]
fn nan_value_is_unknown_never_zero() {
    let w = World::new("nan_value_is_unknown_never_zero");
    let reads = w.read(&value("sum by (cluster) (queue_depth_odd)", &["cluster"]));
    assert_eq!(reads.len(), 5, "every series is kept: {reads:#?}");
    for (cluster, text) in [("a", "NaN"), ("b", "+Inf"), ("c", "-Inf"), ("d", "twelve")] {
        let read = of(&reads, cluster);
        let why = reason(read);
        assert!(
            why.contains(text),
            "{cluster}: the reason names {text}: {why}"
        );
        assert_eq!(read.raw.as_deref(), Some(text), "{read:#?}");
        assert_eq!(read.denominator, None);
    }
    assert_eq!(value_text(of(&reads, "e")), "0.5");

    // A ratio whose numerator is NaN is unknown too, while the population it was read over stays.
    let reads = w.read(&ratio(
        "sum by (cluster) (increase(nan_failed_total[6h]))",
        DENOMINATOR,
    ));
    let alpha = of(&reads, "alpha");
    assert!(reason(alpha).contains("NaN"), "{alpha:#?}");
    assert_eq!(alpha.denominator.as_ref().map(|p| p.n), Some(556));
}

#[test]
fn incomplete_result_makes_every_read_unknown() {
    let w = World::new("incomplete_result_makes_every_read_unknown");

    // A provider warning: `complete: false`, and the warning is named.
    let reads = w.read(&value(
        "sum by (cluster) (queue_depth_warned)",
        &["cluster"],
    ));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for read in &reads {
        let why = reason(read);
        assert!(why.contains("complete: false"), "{why}");
        assert!(
            why.contains("the query touched more samples than the configured soft limit"),
            "{why}"
        );
    }

    // Series past the bound were dropped.
    let reads = w.read(&value("sum by (cluster) (queue_depth_many)", &["cluster"]));
    assert!(!reads.is_empty());
    for read in &reads {
        assert!(reason(read).contains("series_truncated"), "{read:#?}");
    }

    // Infos alone do not make a result partial: the reads stand.
    let reads = w.read(&value("sum by (cluster) (queue_depth_noted)", &["cluster"]));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    assert_eq!(value_text(of(&reads, "alpha")), "4");
    assert_eq!(value_text(of(&reads, "beta")), "5");

    // A ratio whose denominator answer is cut is unknown, and claims no population.
    let reads = w.read(&ratio(
        "sum by (cluster) (increase(cut_failed_total[6h]))",
        "sum by (cluster) (increase(cut_total[6h]))",
    ));
    assert!(!reads.is_empty());
    for read in &reads {
        let why = reason(read);
        assert!(
            why.contains("denominator") && why.contains("partial response from a store"),
            "{why}"
        );
        assert_eq!(read.denominator, None, "{read:#?}");
    }
}

#[test]
fn labels_outside_dimensions_are_dropped() {
    let w = World::new("labels_outside_dimensions_are_dropped");
    let reads = w.read(&value("up", &["cluster", "namespace"]));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    let alpha = of(&reads, "alpha");
    assert_eq!(
        alpha.dims,
        dims(&[("cluster", "alpha"), ("namespace", "shared-1")])
    );
    assert_eq!(value_text(alpha), "1");
    // A declared key the series does not carry stays unset; it is not filled in.
    let beta = of(&reads, "beta");
    assert_eq!(beta.dims, dims(&[("cluster", "beta")]));
    assert_eq!(value_text(beta), "0");
}

#[test]
fn colliding_series_are_unknown() {
    let w = World::new("colliding_series_are_unknown");
    let reads = w.read(&value("max by (cluster, pod) (queue_depth)", &["cluster"]));
    assert_eq!(reads.len(), 3, "no series is dropped: {reads:#?}");
    let alpha: Vec<&Read> = reads
        .iter()
        .filter(|r| r.dims == dims(&[("cluster", "alpha")]))
        .collect();
    assert_eq!(alpha.len(), 2, "{reads:#?}");
    for read in alpha {
        let why = reason(read);
        assert!(
            why.contains("collid") && why.contains("cluster") && why.contains("alpha"),
            "{why}"
        );
    }
    assert_eq!(value_text(of(&reads, "beta")), "3");

    // In a ratio, a numerator collision makes the reads of those dims unknown as well.
    let reads = w.read(&ratio(
        "sum by (cluster, pod) (increase(collide_failed_total[6h]))",
        "sum by (cluster) (increase(collide_total[6h]))",
    ));
    let alpha: Vec<&Read> = reads
        .iter()
        .filter(|r| r.dims == dims(&[("cluster", "alpha")]))
        .collect();
    assert_eq!(alpha.len(), 2, "{reads:#?}");
    for read in alpha {
        assert!(reason(read).contains("collid"), "{read:#?}");
    }
    assert_eq!(value_text(of(&reads, "beta")), "0.03");
}

#[test]
fn empty_population_is_unknown_with_n_0() {
    let w = World::new("empty_population_is_unknown_with_n_0");
    let reads = w.read(&ratio(
        "sum by (cluster) (increase(empty_failed_total[6h]))",
        "sum by (cluster) (increase(empty_total[6h]))",
    ));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    let alpha = of(&reads, "alpha");
    assert_eq!(reason(alpha), "empty population");
    assert_eq!(
        alpha.denominator,
        Some(Population {
            n: 0,
            unit: Unit("{request}".into())
        })
    );
}

#[test]
fn unmatched_numerator_series_make_the_pass_unknown() {
    // A global numerator against a per-cluster denominator: the numerator series has no
    // denominator twin, so a cluster without a numerator series is not a count of 0 (design § 11
    // D20), and nothing is attributed to a cluster from the global count.
    let w = World::new("unmatched_numerator_series_make_the_pass_unknown");
    let reads = w.read(&ratio(
        "sum(increase(global_failed_total[6h]))",
        "sum by (cluster) (increase(global_total[6h]))",
    ));
    assert_eq!(reads.len(), 3, "{reads:#?}");
    for read in &reads {
        let why = reason(read);
        assert!(
            why.contains("numerator") && why.contains("{}"),
            "the reason names the unmatched numerator series: {why}"
        );
        assert_eq!(read.denominator, None, "{read:#?}");
    }
}

#[test]
fn an_empty_numerator_answer_infers_no_zero() {
    // A complete numerator answer with no series at all over a non-empty denominator: nothing shows
    // the numerator shaped like the denominator, so no series reads 0 (design § 11 D20).
    let w = World::new("an_empty_numerator_answer_infers_no_zero");
    let reads = w.read(&ratio(
        "sum by (cluster) (increase(quiet_failed_total[6h]))",
        "sum by (cluster) (increase(quiet_total[6h]))",
    ));
    assert_eq!(
        reads.len(),
        2,
        "one read per denominator series: {reads:#?}"
    );
    for cluster in ["alpha", "beta"] {
        let read = of(&reads, cluster);
        let why = reason(read);
        assert!(why.contains("the numerator answered no series"), "{why}");
        assert_eq!(read.denominator, None, "{read:#?}");
    }
}

#[test]
fn query_sha256_is_of_the_canonical_query_map() {
    let w = World::new("query_sha256_is_of_the_canonical_query_map");

    // A ratio: the canonical JSON of the whole map, keys sorted, no whitespace.
    let canonical = format!(
        "{{\"denominator\":{},\"numerator\":{}}}",
        serde_json::to_string(DENOMINATOR).expect("text"),
        serde_json::to_string(NUMERATOR).expect("text"),
    );
    let reads = w.read(&ratio(NUMERATOR, DENOMINATOR));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for read in &reads {
        assert_eq!(read.query_sha256.0, sha256_hex(&canonical), "{canonical}");
    }

    // One entry is a map too: never the bare text's digest.
    let query = "up";
    let reads = w.read(&value(query, &["cluster", "namespace"]));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for read in &reads {
        assert_eq!(read.query_sha256.0, sha256_hex("{\"value\":\"up\"}"));
        assert_ne!(read.query_sha256.0, sha256_hex(query));
    }
}

#[test]
fn offset_read_shifts_time_only() {
    let w = World::new("offset_read_shifts_time_only");
    let definition = value("sum by (cluster) (queue_depth_now)", &["cluster"]);
    let now = w.read_at(&definition, END);
    let day_before = w.read_at(&definition, "2026-10-13T22:57:03Z");

    // The same text, asked one day earlier.
    let invoked = w.invoked();
    assert_eq!(invoked.len(), 2, "{invoked:?}");
    assert_eq!(invoked[0]["query"], invoked[1]["query"]);
    assert_eq!(invoked[0]["time_unix_s"], Value::from(END_UNIX_S));
    assert_eq!(invoked[1]["time_unix_s"], Value::from(END_UNIX_S - 86_400));

    assert_eq!(value_text(of(&now, "alpha")), "7");
    assert_eq!(value_text(of(&day_before, "alpha")), "6");
    let (now, before) = (of(&now, "alpha"), of(&day_before, "alpha"));
    assert_eq!(now.window.start, Timestamp("2026-10-14T16:57:03Z".into()));
    assert_eq!(now.window.end, Timestamp(END.into()));
    assert_eq!(
        before.window.start,
        Timestamp("2026-10-13T16:57:03Z".into())
    );
    assert_eq!(before.window.end, Timestamp("2026-10-13T22:57:03Z".into()));
    assert_eq!(now.query_sha256, before.query_sha256);
}

#[test]
fn raw_is_the_prometheus_value_text() {
    let w = World::new("raw_is_the_prometheus_value_text");
    let reads = w.read(&value("sum by (cluster) (queue_depth_text)", &["cluster"]));
    assert_eq!(reads.len(), 4, "{reads:#?}");
    // Prometheus writes a float below 1e-6 or from 1e21 up with an exponent; the reading is the
    // same number in the wire grammar of a decimal, the raw text is Prometheus's own.
    for (cluster, raw, parsed) in [
        ("alpha", "0.25", "0.25"),
        ("beta", "1e-07", "0.0000001"),
        ("gamma", "1e+21", "1000000000000000000000"),
        ("delta", "-0.5", "-0.5"),
    ] {
        let read = of(&reads, cluster);
        assert_eq!(read.raw.as_deref(), Some(raw), "{read:#?}");
        assert_eq!(value_text(read), parsed, "{read:#?}");
    }
}

#[test]
fn since_and_retention_start_are_left_unset() {
    let w = World::new("since_and_retention_start_are_left_unset");
    let mut reads = w.read(&ratio(NUMERATOR, DENOMINATOR));
    reads.extend(w.read(&value("up", &["cluster", "namespace"])));
    assert_eq!(reads.len(), 4, "{reads:#?}");
    for read in &reads {
        // No `series.query` answer exposes the source's retention start.
        assert_eq!(read.retention_start, None, "{read:#?}");
        // The population is `n` and its unit only; `since` and `age_s` are the pipeline's.
        if let Some(Population { n, unit }) = &read.denominator {
            assert!(*n > 0, "{read:#?}");
            assert_eq!(unit, &Unit("{request}".into()));
        }
    }
    assert_eq!(
        reads.iter().filter(|r| r.denominator.is_some()).count(),
        2,
        "{reads:#?}"
    );
}
