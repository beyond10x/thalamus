//! Adversary pass 1, `story:m3-vocabulary-mapping`, the `thalamus-sources` half: an `unknown`
//! read an adapter made for a reason other than its value text must stay `unknown` under
//! `producer_mapped`. Unknown is never OK (AGENTS.md), and `vocabulary.rs` promises "A read that
//! was not made keeps its own reason".

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::{QueryDigest, Reading, RunId, TimeWindow, UnreadReading};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;
use thalamus_sources::vocabulary::map_reads;

const END: &str = "2026-10-14T22:57:03Z";

fn mapped(
    extractor: &str,
    query: &str,
    dimensions: &[&str],
    mapping: &[(&str, Verdict)],
) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.probe".into()),
        title: "A mapped probe signal".into(),
        family: Family("platform".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("{verdict}".into())),
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
        query: BTreeMap::from([("value".to_owned(), QueryText(query.into()))]),
        window: Duration("PT6H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Source,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId("producer_mapped".into()),
            baselines: vec![BaselineKind::Fixed],
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: Some(Decimal("0".into())),
            rolling_n: None,
            mapping: Some(
                mapping
                    .iter()
                    .map(|(raw, verdict)| ((*raw).to_owned(), *verdict))
                    .collect(),
            ),
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

/// One test's directory with the Prometheus recordings and the Connectors double, as
/// `tests/prometheus.rs` builds it.
fn world(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_vocabulary_p1")
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
    dir
}

fn read_through_registry(dir: &Path, definition: &SignalDefinitionData) -> Vec<Read> {
    registry::read(
        &Connectors::new(dir.join("connectors")),
        definition,
        &Timestamp(END.into()),
        &RunId("run_test".into()),
    )
}

/// The unit-level shape: an `unknown` read that still carries the producer's text.
#[test]
fn adv_unknown_read_with_raw_text_stays_unknown() {
    let definition = mapped("promql.value", "x", &[], &[("0", Verdict::OK)]);
    let read = Read {
        dims: BTreeMap::new(),
        reading: Reading::Unknown(UnreadReading {
            reason: "the result is cut (complete: false)".into(),
        }),
        window: TimeWindow {
            start: Timestamp("2026-10-14T16:57:03Z".into()),
            end: Timestamp(END.into()),
        },
        denominator: None,
        adapter: AdapterId("metrics".into()),
        connection: ConnectionId("metrics-main".into()),
        query_sha256: QueryDigest("00".into()),
        raw: Some("0".into()),
        retention_start: None,
        unread: None,
    };
    let mapped = map_reads(&definition, vec![read]).remove(0);
    assert!(
        matches!(mapped.reading, Reading::Unknown(_)),
        "an unknown read the adapter made for a cut result is mapped onto the ladder: {:?}",
        mapped.reading
    );
}

/// End to end through the Prometheus adapter: a result Prometheus marks `complete: false` makes
/// every read `unknown` (`prometheus.rs` module doc, `tests/prometheus.rs`
/// `incomplete_result_makes_every_read_unknown`), and the adapter keeps each series' value text as
/// `raw` (`prometheus.rs:231`). Under `producer_mapped` those reads must stay `unknown`.
#[test]
fn adv_prometheus_cut_result_stays_unknown_under_producer_mapped() {
    let dir = world("cut");
    let definition = mapped(
        "promql.value",
        "sum by (cluster) (queue_depth_warned)",
        &["cluster"],
        &[("4", Verdict::OK), ("5", Verdict::OK)],
    );
    let reads = read_through_registry(&dir, &definition);
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for read in &reads {
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "a read of a cut Prometheus result is {:?} under producer_mapped (raw {:?})",
            read.reading,
            read.raw
        );
    }
}

/// End to end: two series that fall onto one `dims` are both `unknown` (`tests/prometheus.rs`
/// `colliding_series_are_unknown`); under `producer_mapped` they must stay so.
#[test]
fn adv_prometheus_colliding_series_stay_unknown_under_producer_mapped() {
    let dir = world("collision");
    let definition = mapped(
        "promql.value",
        "max by (cluster, pod) (queue_depth)",
        &["cluster"],
        &[
            ("1", Verdict::OK),
            ("2", Verdict::CRIT),
            ("3", Verdict::WARN),
        ],
    );
    let reads = read_through_registry(&dir, &definition);
    let alpha: Vec<&Read> = reads
        .iter()
        .filter(|read| read.dims.get("cluster").map(String::as_str) == Some("alpha"))
        .collect();
    assert_eq!(alpha.len(), 2, "{reads:#?}");
    for read in alpha {
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "a colliding series reads {:?} under producer_mapped (raw {:?})",
            read.reading,
            read.raw
        );
    }
}
