//! Adversary pass 2 against the Prometheus adapter: the numerator-0 rule of design § 11 D20. A
//! missing numerator series reads 0 only when both answers are complete and every numerator series
//! has a denominator twin; a label-set mismatch between the two queries must never turn into a 0.
//!
//! Every answer in `tests/fixtures/adv2_prometheus` has the shape Connectors' `prometheus` adapter
//! builds for `series.query`.

use std::path::Path;

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

fn read(name: &str, definition: &SignalDefinitionData) -> Vec<Read> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv2_prometheus")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adv2_prometheus");
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
    registry::read(
        &Connectors::new(dir.join("connectors")),
        definition,
        &Timestamp(END.into()),
        &RunId("run_adv2".into()),
    )
}

fn ratio(numerator: &str, denominator: &str, dimensions: &[&str]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.adversary_two".into()),
        title: "An adversary probe".into(),
        family: Family("platform".into()),
        kind: SignalKind::Ratio,
        unit: Some(Unit("1".into())),
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
        extractor: ExtractorId("promql.ratio".into()),
        query: [("numerator", numerator), ("denominator", denominator)]
            .iter()
            .map(|(k, v)| ((*k).to_string(), QueryText((*v).into())))
            .collect(),
        window: Duration("PT6H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Source,
        denominator: Some(DenominatorSpec {
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

fn assert_every_read_unknown(reads: &[Read], why_contains: &str) {
    assert!(!reads.is_empty());
    for read in reads {
        match &read.reading {
            Reading::Unknown(unread) => {
                assert!(
                    unread.reason.contains(why_contains),
                    "the reason names {why_contains:?}: {read:#?}"
                );
                assert_eq!(read.denominator, None, "{read:#?}");
            }
            other => panic!("no read of this pass is a reading, got {other:?} in {read:#?}"),
        }
    }
}

#[test]
fn adv2_a_cut_numerator_never_infers_a_zero() {
    // The numerator answer is `complete: false` (Connectors dropped notes past its bound, no
    // warning); the denominator is complete and carries `beta`, which the numerator lacks.
    let reads = read(
        "numerator_cut",
        &ratio(
            "sum by (cluster) (increase(adv2_numcut_failed_total[6h]))",
            "sum by (cluster) (increase(adv2_numcut_total[6h]))",
            &["cluster"],
        ),
    );
    assert_eq!(reads.len(), 2, "{reads:#?}");
    assert_every_read_unknown(&reads, "numerator");
}

#[test]
fn adv2_a_numerator_finer_by_a_declared_key_never_infers_a_zero() {
    // The numerator splits by `namespace`, the denominator does not: `{cluster="alpha",
    // namespace="shared-1"}` has no denominator twin, so `beta` is no count of 0.
    let reads = read(
        "numerator_finer",
        &ratio(
            "sum by (cluster, namespace) (increase(adv2_ns_failed_total[6h]))",
            "sum by (cluster) (increase(adv2_ns_total[6h]))",
            &["cluster", "namespace"],
        ),
    );
    assert_every_read_unknown(&reads, "shared-1");
}

#[test]
fn adv2_a_numerator_coarser_on_one_series_never_infers_a_zero() {
    // One numerator series lacks `namespace` (L9: left unset); the denominator splits `alpha` by
    // namespace. The two namespaces of `alpha` are not counts of 0.
    let reads = read(
        "numerator_coarser",
        &ratio(
            "sum by (cluster, namespace) (increase(adv2_mixed_failed_total[6h]))",
            "sum by (cluster, namespace) (increase(adv2_mixed_total[6h]))",
            &["cluster", "namespace"],
        ),
    );
    assert_every_read_unknown(&reads, "alpha");
}

#[test]
fn adv2_a_numerator_collision_does_not_block_a_zero_elsewhere() {
    // `alpha` collides in the numerator (split by an undeclared `pod`); `beta` has no numerator
    // series and a twin-complete shape, so it reads 0 over its population.
    let reads = read(
        "numerator_collision",
        &ratio(
            "sum by (cluster, pod) (increase(adv2_pod_failed_total[6h]))",
            "sum by (cluster) (increase(adv2_pod_total[6h]))",
            &["cluster"],
        ),
    );
    let alpha: Vec<&Read> = reads
        .iter()
        .filter(|r| r.dims.get("cluster").map(String::as_str) == Some("alpha"))
        .collect();
    assert_eq!(alpha.len(), 2, "{reads:#?}");
    for read in alpha {
        assert!(
            matches!(&read.reading, Reading::Unknown(u) if u.reason.contains("collid")),
            "{read:#?}"
        );
    }
    let beta: Vec<&Read> = reads
        .iter()
        .filter(|r| r.dims.get("cluster").map(String::as_str) == Some("beta"))
        .collect();
    assert_eq!(beta.len(), 1, "{reads:#?}");
    assert!(
        matches!(&beta[0].reading, Reading::Ok(ok) if ok.value.0 == "0"),
        "{reads:#?}"
    );
    assert_eq!(
        beta[0].denominator,
        Some(Population {
            n: 80,
            unit: Unit("{request}".into())
        })
    );
}
