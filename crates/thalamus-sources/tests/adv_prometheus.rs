//! Adversary pass 1 against the Prometheus adapter: the branches `tests/prometheus.rs` never
//! drives (a missing numerator or denominator series, a negative or zero denominator, an empty
//! answer, scalar, string and matrix answers) and the float text Prometheus really writes (Go's
//! `'f'` form, and `'e'` below 1e-6 and from 1e21 up, `-0`).
//!
//! Every answer in `tests/fixtures/adv_prometheus` has the shape Connectors' `prometheus` adapter
//! builds for `series.query` (`adapters/prometheus/src/lib.rs`, `query`, `series`): one sample per
//! vector or scalar series, `complete` false only with warnings, omitted notes or a cut.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

fn world(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_prometheus")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adv_prometheus");
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

fn read(name: &str, definition: &SignalDefinitionData) -> Vec<Read> {
    let dir = world(name);
    registry::read(
        &Connectors::new(dir.join("connectors")),
        definition,
        &Timestamp(END.into()),
        &RunId("run_adv".into()),
    )
}

fn definition(
    extractor: &str,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    let ratio = extractor == "promql.ratio";
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.adversary".into()),
        title: "An adversary probe".into(),
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

fn ratio(numerator: &str, denominator: &str) -> SignalDefinitionData {
    definition(
        "promql.ratio",
        &[("numerator", numerator), ("denominator", denominator)],
        &["cluster"],
    )
}

fn value(query: &str) -> SignalDefinitionData {
    definition("promql.value", &[("value", query)], &["cluster"])
}

fn of<'a>(reads: &'a [Read], cluster: &str) -> &'a Read {
    let found: Vec<&Read> = reads
        .iter()
        .filter(|r| r.dims.get("cluster").map(String::as_str) == Some(cluster))
        .collect();
    assert_eq!(found.len(), 1, "one read for {cluster}: {reads:#?}");
    found[0]
}

fn ok_text(read: &Read) -> &str {
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

fn population(n: i64) -> Option<Population> {
    Some(Population {
        n,
        unit: Unit("{request}".into()),
    })
}

/// The published decimal pattern (`generated/thalamus-model/src/json.rs`, `decimal_at`): an
/// optional `-`, digits without a leading zero, an optional `.` and digits.
fn is_wire_decimal(text: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    digits(whole) && !(whole.len() > 1 && whole.starts_with('0')) && fraction.is_none_or(digits)
}

#[test]
fn adv_missing_numerator_series_in_a_complete_answer_is_zero() {
    // A labelled counter child exists only after its first increment, so a cluster that never
    // failed has no numerator series. When both answers are complete, the denominator's series
    // with no numerator series reads numerator 0 (design § 11 D20).
    let reads = read(
        "missing_numerator",
        &ratio(
            "sum by (cluster) (increase(adv_lazy_failed_total[6h]))",
            "sum by (cluster) (increase(adv_lazy_total[6h]))",
        ),
    );
    assert_eq!(reads.len(), 2, "{reads:#?}");
    let alpha = of(&reads, "alpha");
    assert_eq!(ok_text(alpha), "0.02");
    assert_eq!(alpha.denominator, population(100));
    let beta = of(&reads, "beta");
    assert_eq!(ok_text(beta), "0", "{beta:#?}");
    assert_eq!(beta.denominator, population(80), "{beta:#?}");
    assert_eq!(beta.raw, None);
}

#[test]
fn adv_missing_denominator_series_is_unknown_without_population() {
    let reads = read(
        "missing_denominator",
        &ratio(
            "sum by (cluster) (increase(adv_orphan_failed_total[6h]))",
            "sum by (cluster) (increase(adv_orphan_total[6h]))",
        ),
    );
    assert_eq!(reads.len(), 2, "{reads:#?}");
    assert_eq!(ok_text(of(&reads, "alpha")), "0.02");
    let beta = of(&reads, "beta");
    assert!(reason(beta).contains("denominator"), "{beta:#?}");
    assert_eq!(beta.denominator, None, "{beta:#?}");
}

#[test]
fn adv_negative_and_zero_denominators() {
    let reads = read(
        "signed_denominator",
        &ratio(
            "sum by (cluster) (delta(adv_signed_failed[6h]))",
            "sum by (cluster) (delta(adv_signed_total[6h]))",
        ),
    );
    assert_eq!(reads.len(), 3, "{reads:#?}");

    // -40 is no population: unknown, and no negative `n` is claimed.
    let alpha = of(&reads, "alpha");
    assert!(reason(alpha).contains("negative"), "{alpha:#?}");
    assert_eq!(alpha.denominator, None, "{alpha:#?}");

    // 0 over -0 is still 0 / 0: the empty population, n = 0.
    let beta = of(&reads, "beta");
    assert_eq!(reason(beta), "empty population", "{beta:#?}");
    assert_eq!(beta.denominator, population(0));

    // 3 over 0 is unknown with n = 0, and it is not the empty population.
    let gamma = of(&reads, "gamma");
    let why = reason(gamma);
    assert_ne!(why, "empty population");
    assert!(why.contains('3') && why.contains('0'), "{why}");
    assert_eq!(gamma.denominator, population(0));
}

#[test]
fn adv_empty_answer_is_one_unknown_never_no_read() {
    let reads = read("empty_value", &value("sum by (cluster) (adv_nothing)"));
    assert_eq!(
        reads.len(),
        1,
        "an empty answer is one unknown read: {reads:#?}"
    );
    assert!(reads[0].dims.is_empty(), "{reads:#?}");
    assert!(!reason(&reads[0]).is_empty());
    assert_eq!(reads[0].raw, None);
}

#[test]
fn adv_prometheus_float_text_becomes_a_wire_decimal_of_the_same_number() {
    let reads = read("float_forms", &value("sum by (cluster) (adv_float_forms)"));
    assert_eq!(reads.len(), 8, "{reads:#?}");
    for (cluster, raw, expected) in [
        ("negzero", "-0", Some("0")),
        ("tiny", "1e-07", Some("0.0000001")),
        ("negtiny", "-1e-07", Some("-0.0000001")),
        ("huge", "1e+21", Some("1000000000000000000000")),
        ("subnormal", "5e-324", None),
        ("max", "1.7976931348623157e+308", None),
        ("sum", "0.30000000000000004", Some("0.30000000000000004")),
        (
            "big",
            "123456789012345680000",
            Some("123456789012345680000"),
        ),
    ] {
        let read = of(&reads, cluster);
        let text = ok_text(read);
        assert_eq!(read.raw.as_deref(), Some(raw), "{read:#?}");
        assert!(
            is_wire_decimal(text),
            "{cluster}: {text} is no wire decimal"
        );
        let back: f64 = text.parse().expect("a decimal");
        let want: f64 = raw.parse().expect("prometheus text");
        if want == 0.0 {
            assert_eq!(text, "0", "{cluster}: zero is written 0, never -0");
        } else {
            assert_eq!(
                back.to_bits(),
                want.to_bits(),
                "{cluster}: {text} is not {raw}"
            );
        }
        if let Some(expected) = expected {
            assert_eq!(text, expected, "{cluster}");
        }
    }
}

#[test]
fn adv_ratio_of_tiny_and_inexact_floats_is_a_wire_decimal() {
    let reads = read(
        "tiny_ratio",
        &ratio(
            "sum by (cluster) (increase(adv_tiny_failed_total[6h]))",
            "sum by (cluster) (increase(adv_tiny_total[6h]))",
        ),
    );
    assert_eq!(reads.len(), 2, "{reads:#?}");
    let alpha = ok_text(of(&reads, "alpha"));
    assert!(is_wire_decimal(alpha), "{alpha}");
    assert_eq!(
        alpha.parse::<f64>().expect("decimal"),
        1e-7 / 3.0,
        "{alpha}"
    );
    // 0.1 / 0.30000000000000004: the same float quotient Prometheus would answer for `a / b`.
    let beta = ok_text(of(&reads, "beta"));
    assert!(is_wire_decimal(beta), "{beta}");
    assert_eq!(
        beta.parse::<f64>().expect("decimal"),
        0.1 / 0.300_000_000_000_000_04,
        "{beta}"
    );
    assert_eq!(of(&reads, "beta").denominator, population(0));
}

#[test]
fn adv_scalar_answer_is_one_read_without_dims() {
    let reads = read("scalar", &value("scalar(sum(adv_queue))"));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reads[0].dims.is_empty(), "{reads:#?}");
    assert_eq!(ok_text(&reads[0]), "4");
    assert_eq!(reads[0].raw.as_deref(), Some("4"));
}

#[test]
fn adv_string_and_matrix_answers_are_unknown() {
    for (name, query) in [("string", "\"adv-text\""), ("matrix", "adv_queue[5m]")] {
        let reads = read(name, &value(query));
        assert!(!reads.is_empty(), "{name}");
        for read in &reads {
            assert!(
                matches!(read.reading, Reading::Unknown(_)),
                "{name}: a {name} answer is no instant value: {read:#?}"
            );
        }
    }
}

#[test]
fn adv_ratio_joins_on_dims_and_leaks_no_label() {
    // The numerator carries `code`, which is no dimension: it is dropped and the series join.
    let reads = read(
        "code_join",
        &ratio(
            "sum by (cluster, code) (increase(adv_code_failed_total{code=\"500\"}[6h]))",
            "sum by (cluster) (increase(adv_code_total[6h]))",
        ),
    );
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for (cluster, text, n) in [("alpha", "0.1", 50), ("beta", "0.05", 20)] {
        let read = of(&reads, cluster);
        let want: BTreeMap<String, String> = [("cluster".to_string(), cluster.to_string())].into();
        assert_eq!(read.dims, want, "{read:#?}");
        assert_eq!(ok_text(read), text);
        assert_eq!(read.denominator, population(n));
    }

    // A numerator without the dimension never joins a denominator that has it (L9): nothing is
    // attributed to `alpha` from a global count.
    let reads = read(
        "global_numerator",
        &ratio(
            "sum(increase(adv_global_failed_total[6h]))",
            "sum by (cluster) (increase(adv_global_total[6h]))",
        ),
    );
    assert!(!reads.is_empty());
    for read in &reads {
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "a global numerator over a per-cluster denominator is no reading: {read:#?}"
        );
    }
}

#[test]
fn adv_omitted_notes_without_warnings_make_the_read_unknown() {
    // Connectors sets `complete: false` when it dropped notes past its bound, with no warning.
    let reads = read("noisy", &value("sum by (cluster) (adv_noisy)"));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("complete: false"), "{reads:#?}");
}
