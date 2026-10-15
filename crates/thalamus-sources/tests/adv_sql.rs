//! Adversary cases for `story:m3-sql-adapter`: the `sql` adapter held to the story, design § 1
//! (L4, L5, L9), § 2.2, and what the Connectors `sql` adapter's `query.read` really answers.
//!
//! Recordings are built in the test, so that a cut answer can carry the 1,000 rows the real
//! adapter returns before it reports `truncated` (it cuts only at `rows == limit`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Reading, RunId};
use thalamus_model::primitives::{Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const END: &str = "2026-10-14T22:57:03Z";
const RECORDING: &str = "operations+invoke+a=sql+c=reports-replica+o=query.read.json";

/// One `query.read` answer as Connectors encodes it: the result object as a JSON string.
fn answer(query: &str, columns: &[&str], rows: &[Vec<Value>], truncated: bool) -> Value {
    let result = json!({
        "columns": columns.iter().map(|c| json!({"name": c, "native_type": "text"})).collect::<Vec<_>>(),
        "rows": rows,
        "truncated": truncated,
        "provenance": {"instance": "sql", "resource": "reports",
                       "observed_at_unix_ms": 1_792_018_623_000_u64, "source_revision": null},
    });
    json!({
        "input": {"query": query},
        "stdout": {"ok": true, "result": {
            "adapter": "sql", "operation": "query.read", "revision": "op-rev-1",
            "result": result.to_string(),
        }},
    })
}

/// A fresh directory with the `common` recordings, the given `query.read` answers, and
/// `connectors` linked to the double.
fn world(name: &str, answers: Vec<Value>) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_sql")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let common = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sql/common");
    for entry in std::fs::read_dir(common).expect("common exists") {
        let path = entry.expect("fixture entry").path();
        std::fs::copy(&path, dir.join(path.file_name().expect("file name"))).expect("copy");
    }
    std::fs::write(
        dir.join(RECORDING),
        serde_json::to_string(&Value::Array(answers)).expect("encode"),
    )
    .expect("write the recording");
    std::os::unix::fs::symlink(
        env!("CARGO_BIN_EXE_connectors-double"),
        dir.join("connectors"),
    )
    .expect("link the double");
    dir
}

fn read(dir: &Path, definition: &SignalDefinitionData) -> Vec<Read> {
    registry::read(
        &Connectors::new(dir.join("connectors")),
        definition,
        &Timestamp(END.into()),
        &RunId("run_test".into()),
    )
}

fn definition(
    extractor: &str,
    kind: SignalKind,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("platform.cluster.adversary".into()),
        title: "Adversary".into(),
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

fn ratio(numerator: &str, denominator: &str, dimensions: &[&str]) -> SignalDefinitionData {
    let mut d = definition(
        "sql.ratio",
        SignalKind::Ratio,
        &[("numerator", numerator), ("denominator", denominator)],
        dimensions,
    );
    d.denominator = Some(DenominatorSpec {
        unit: Unit("{request}".into()),
        min_n: 20,
        since: "first_sample".into(),
    });
    d
}

fn is_ok(read: &Read) -> bool {
    matches!(read.reading, Reading::Ok(_))
}

/// 1,000 rows, one per series `c000`…`c998` and a last series `omega`: the most the real adapter
/// returns before it reports the result cut.
fn thousand_rows(value: &str) -> Vec<Vec<Value>> {
    let mut rows: Vec<Vec<Value>> = (0..999)
        .map(|i| vec![json!(format!("c{i:03}")), json!(value)])
        .collect();
    rows.push(vec![json!("omega"), json!(value)]);
    rows
}

/// Story Behaviour: "A result the adapter reports as truncated: `partial`". Design § 2.2:
/// `partial` is "read but cut". The registry's own contract: "never a dropped series".
/// A cut `sql.value` answer is today read as 1,000 complete `ok` reads; the series past the cut
/// vanish, and the last series (whose further rows, if any, were cut) reads `ok`. Nothing in the
/// reads says the result was cut.
#[test]
fn adv_sql_value_cut_result_is_not_silently_complete() {
    const Q: &str = "SELECT cluster, value FROM queue_depth";
    let dir = world(
        "value_cut",
        vec![answer(Q, &["cluster", "value"], &thousand_rows("4"), true)],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.value",
            SignalKind::Gauge,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    assert!(
        reads.iter().any(|r| !is_ok(r)),
        "a result Connectors reported truncated was read as {} complete `ok` reads; \
         nothing records the cut",
        reads.len()
    );
}

/// The same for `sql.ratio`: both sides cut at 1,000 rows with the same series. Every pair reads
/// `ok`; the series past the cut vanish without an `unknown` read.
#[test]
fn adv_sql_ratio_cut_result_is_not_silently_complete() {
    const N: &str = "SELECT cluster, count(*) AS value FROM requests WHERE failed GROUP BY cluster";
    const D: &str = "SELECT cluster, count(*) AS value FROM requests GROUP BY cluster";
    let dir = world(
        "ratio_cut",
        vec![
            answer(N, &["cluster", "value"], &thousand_rows("3"), true),
            answer(D, &["cluster", "value"], &thousand_rows("100"), true),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert!(
        reads.iter().any(|r| !is_ok(r)),
        "two results Connectors reported truncated were read as {} complete `ok` ratios",
        reads.len()
    );
}

/// PostgreSQL and MySQL both return two result columns of one name (`SELECT a.value, b.value …`),
/// and Connectors keeps both names (`mysql.rs`: positional aliases so "duplicate result names
/// cannot collide"). Which one is the value is then ambiguous; today the first is read `ok`.
#[test]
fn adv_sql_two_value_columns_are_ambiguous() {
    const Q: &str =
        "SELECT q.cluster, q.value, r.value FROM queue_depth q JOIN retries r USING (cluster)";
    let dir = world(
        "two_value_columns",
        vec![answer(
            Q,
            &["cluster", "value", "value"],
            &[vec![json!("alpha"), json!("1"), json!("2")]],
            false,
        )],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.value",
            SignalKind::Gauge,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        !is_ok(&reads[0]),
        "two columns named `value`, and the first was read as the value: {:?}",
        reads[0].reading
    );
}

/// The same ambiguity on a dimension column: two columns named `cluster` holding different values.
/// Today the first sets the key and the read is `ok`, attributed to `alpha` (L9).
#[test]
fn adv_sql_two_dimension_columns_of_one_name_are_ambiguous() {
    const Q: &str =
        "SELECT a.cluster, b.cluster, a.value FROM queue_depth a JOIN peers b USING (link)";
    let dir = world(
        "two_dimension_columns",
        vec![answer(
            Q,
            &["cluster", "cluster", "value"],
            &[vec![json!("alpha"), json!("beta"), json!("5")]],
            false,
        )],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.value",
            SignalKind::Gauge,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        !(is_ok(&reads[0])
            && reads[0].dims == BTreeMap::from([("cluster".to_string(), "alpha".to_string())])),
        "two `cluster` columns (alpha, beta); the read was attributed to the first: {:#?}",
        reads[0]
    );
}

/// A NULL dimension cell sets no key, and a query that does not return the column sets no key
/// either, so the two are one series to the ratio's pairing. The numerator's NULL-tenant row (3
/// failures of untenanted traffic) is divided by the denominator's whole-cluster count (100) and
/// read `ok` as 0.03 for `{cluster: alpha}`.
#[test]
fn adv_sql_ratio_null_dimension_does_not_pair_with_an_absent_column() {
    const N: &str =
        "SELECT cluster, tenant, count(*) AS value FROM requests WHERE failed GROUP BY 1, 2";
    const D: &str = "SELECT cluster, count(*) AS value FROM requests GROUP BY 1";
    let dir = world(
        "ratio_null_dimension",
        vec![
            answer(
                N,
                &["cluster", "tenant", "value"],
                &[
                    vec![json!("alpha"), Value::Null, json!("3")],
                    vec![json!("alpha"), json!("t1"), json!("2")],
                ],
                false,
            ),
            answer(
                D,
                &["cluster", "value"],
                &[vec![json!("alpha"), json!("100")]],
                false,
            ),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster", "tenant"]));
    let alpha = reads
        .iter()
        .find(|r| r.dims == BTreeMap::from([("cluster".to_string(), "alpha".to_string())]))
        .expect("a read for {cluster: alpha}");
    assert!(
        !is_ok(alpha),
        "the NULL-tenant numerator row was paired with a denominator that has no tenant column: \
         {:?}",
        alpha.reading
    );
}

/// Exactness boundary: the implementor's own unit test reads `1e-20 / 1` as `0`; `1e-40 / 1` is the
/// same kind of value and should read the same, not `unknown` blaming the denominator `1`.
#[test]
fn adv_sql_ratio_tiny_numerator_is_read() {
    const N: &str = "SELECT sum(weight) AS value FROM failures";
    const D: &str = "SELECT count(*) AS value FROM requests";
    let dir = world(
        "ratio_tiny_numerator",
        vec![
            answer(N, &["value"], &[vec![json!("1e-40")]], false),
            answer(D, &["value"], &[vec![json!("1")]], false),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        is_ok(&reads[0]),
        "1e-40 / 1 was not read: {:?}",
        reads[0].reading
    );
}

/// Type tricks the real adapter never sends (JSON numbers, booleans) and one it does
/// (PostgreSQL's `t` for a boolean): none reads as a number, none reads 0.
#[test]
fn adv_sql_type_tricks_never_read_a_number() {
    const Q: &str = "SELECT cluster, value FROM flags";
    for (name, cell) in [
        ("json_number", json!(5)),
        ("json_bool", json!(true)),
        ("pg_bool", json!("t")),
        ("json_object", json!({"value": "5"})),
    ] {
        let dir = world(
            &format!("type_{name}"),
            vec![answer(
                Q,
                &["cluster", "value"],
                &[vec![json!("alpha"), cell.clone()]],
                false,
            )],
        );
        let reads = read(
            &dir,
            &definition(
                "sql.value",
                SignalKind::Gauge,
                &[("value", Q)],
                &["cluster"],
            ),
        );
        assert!(!reads.is_empty(), "{name}: no read at all");
        for r in &reads {
            assert!(!is_ok(r), "{name}: cell {cell} read as {:?}", r.reading);
        }
    }
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(u) => &u.reason,
        other => panic!("expected unknown, got {other:?}"),
    }
}

/// Mutant catcher (`step(0) >= 5` → `> 5`): the 19th fractional digit is exactly 5, so half away
/// from zero rounds the 18th up, on either sign. No test in the suite lands on a 5.
#[test]
fn adv_sql_ratio_rounds_an_exact_half_away_from_zero() {
    const N: &str = "SELECT n AS value FROM a";
    const NN: &str = "SELECT -n AS value FROM a";
    const D: &str = "SELECT d AS value FROM b";
    for (name, numerator, cell, expected) in [
        ("positive", N, "1", "0.000000000000000001"),
        ("negative", NN, "-1", "-0.000000000000000001"),
    ] {
        let dir = world(
            &format!("half_{name}"),
            vec![
                answer(numerator, &["value"], &[vec![json!(cell)]], false),
                answer(D, &["value"], &[vec![json!("2000000000000000000")]], false),
            ],
        );
        let reads = read(&dir, &ratio(numerator, D, &[]));
        assert_eq!(reads.len(), 1, "{reads:#?}");
        match &reads[0].reading {
            Reading::Ok(r) => assert_eq!(r.value.0, expected, "{name}"),
            other => panic!("{name}: {other:?}"),
        }
    }
}

/// Mutant catcher (drop the duplicate check in `side`): a numerator that returns one series twice
/// has no one value for it.
#[test]
fn adv_sql_ratio_series_twice_in_one_side_is_unknown() {
    const N: &str = "SELECT cluster, value FROM failures";
    const D: &str = "SELECT cluster, value FROM requests";
    let dir = world(
        "ratio_twice",
        vec![
            answer(
                N,
                &["cluster", "value"],
                &[
                    vec![json!("alpha"), json!("1")],
                    vec![json!("alpha"), json!("2")],
                ],
                false,
            ),
            answer(
                D,
                &["cluster", "value"],
                &[vec![json!("alpha"), json!("10")]],
                false,
            ),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        reason(&reads[0]).contains("twice"),
        "{:?}",
        reads[0].reading
    );
}

/// Mutant catcher (a missing side read as 0): a numerator series without a denominator twin
/// makes the pass unknown; no series reads 0.
#[test]
fn adv_sql_ratio_unpaired_series_are_unknown_never_zero() {
    const N: &str = "SELECT cluster, value FROM failures";
    const D: &str = "SELECT cluster, value FROM requests";
    let dir = world(
        "ratio_unpaired",
        vec![
            answer(
                N,
                &["cluster", "value"],
                &[
                    vec![json!("alpha"), json!("1")],
                    vec![json!("gamma"), json!("1")],
                ],
                false,
            ),
            answer(
                D,
                &["cluster", "value"],
                &[
                    vec![json!("alpha"), json!("10")],
                    vec![json!("beta"), json!("10")],
                ],
                false,
            ),
        ],
    );
    // Design § 11 D20: `gamma` has no denominator twin, so the whole pass is one unknown read
    // naming it; neither `alpha` nor `beta` (numerator 0 only when every numerator series is
    // paired) is read, and nothing is 0.
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    let why = reason(&reads[0]);
    assert!(
        why.contains("gamma") && why.contains("denominator"),
        "{why}"
    );
}

/// Mutant catchers (drop the empty-answer branches): no rows read one `unknown`, never nothing.
#[test]
fn adv_sql_empty_answers_read_one_unknown() {
    const Q: &str = "SELECT cluster, value FROM nothing";
    const N: &str = "SELECT cluster, value FROM no_failures";
    const D: &str = "SELECT cluster, value FROM no_requests";
    let dir = world(
        "empty_value",
        vec![answer(Q, &["cluster", "value"], &[], false)],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.value",
            SignalKind::Gauge,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("no rows"));

    let dir = world(
        "empty_ratio",
        vec![
            answer(N, &["cluster", "value"], &[], false),
            answer(D, &["cluster", "value"], &[], false),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("no rows"));
}

/// Mutant catcher (take the first candidate column): no column named `value` and two columns that
/// are not declared dimensions is ambiguous, and reads `unknown`.
#[test]
fn adv_sql_two_candidate_value_columns_are_unknown() {
    const Q: &str = "SELECT cluster, failures, total FROM requests";
    let dir = world(
        "two_candidates",
        vec![answer(
            Q,
            &["cluster", "failures", "total"],
            &[vec![json!("alpha"), json!("1"), json!("10")]],
            false,
        )],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.value",
            SignalKind::Gauge,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        reason(&reads[0]).contains("`value`"),
        "{:?}",
        reads[0].reading
    );
}

/// Huge integers keep every digit; a huge denominator is either exact or unknown, never rounded
/// through a float.
#[test]
fn adv_sql_huge_integers_stay_exact() {
    const Q: &str = "SELECT value FROM big";
    let huge = "123456789012345678901234567890123456789012345678901234567890";
    let dir = world(
        "huge_value",
        vec![answer(Q, &["value"], &[vec![json!(huge)]], false)],
    );
    let reads = read(
        &dir,
        &definition("sql.value", SignalKind::Gauge, &[("value", Q)], &[]),
    );
    assert_eq!(reads.len(), 1);
    match &reads[0].reading {
        Reading::Ok(r) => assert_eq!(r.value.0, huge),
        other => panic!("{other:?}"),
    }
}

// ---- Pass 2: the correction (D20, significant-digit division, cut handling, sql.count dims) ----

/// D20 infers numerator 0 because "counting queries (… SQL `GROUP BY` …) emit nothing for zero".
/// A numerator with no dimension column is not grouped: an aggregate without `GROUP BY` returns
/// exactly one row even over no input (`count` → `0`, `sum` → NULL). An empty answer from it is
/// therefore never "zero cases"; it is a query that selects a stored row that is not there
/// (a daily roll-up not yet written, a row-level-security filter). Today that reads `ok` `0`.
#[test]
fn adv_sql_ratio_empty_ungrouped_numerator_is_not_zero() {
    const N: &str = "SELECT failures AS value FROM daily_rollup WHERE day = current_date";
    const D: &str = "SELECT count(*) AS value FROM requests";
    let dir = world(
        "ratio_empty_ungrouped_numerator",
        vec![
            answer(N, &["value"], &[], false),
            answer(D, &["value"], &[vec![json!("556")]], false),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        !is_ok(&reads[0]),
        "an ungrouped numerator answered no row, and the ratio was read as {:?} with population \
         {:?}",
        reads[0].reading,
        reads[0].denominator
    );
}

/// The same with declared dimensions the queries do not return: both sides key every row `{}`,
/// and an empty numerator again reads `ok` `0`.
#[test]
fn adv_sql_ratio_empty_numerator_without_dimension_columns_is_not_zero() {
    const N: &str = "SELECT failures AS value FROM daily_rollup WHERE day = current_date";
    const D: &str = "SELECT count(*) AS value FROM requests";
    let dir = world(
        "ratio_empty_numerator_declared_dims",
        vec![
            answer(N, &["value"], &[], false),
            answer(D, &["value"], &[vec![json!("556")]], false),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(!is_ok(&reads[0]), "{:?}", reads[0].reading);
}

/// D20 as extended (design § 11, main `docs/design/thalamus.md:422`): "a complete numerator
/// answer with no series at all over a non-empty denominator reads unknown (a renamed metric or a
/// broken matcher looks the same as zero everywhere)". A grouped numerator whose `WHERE` matches
/// nothing (a renamed status value) answers no row; every series must read unknown, none 0.
#[test]
fn adv_sql_ratio_empty_grouped_numerator_reads_every_series_unknown() {
    const N: &str =
        "SELECT cluster, count(*) AS value FROM requests WHERE status = 'FAILED' GROUP BY cluster";
    const D: &str = "SELECT cluster, count(*) AS value FROM requests GROUP BY cluster";
    let dir = world(
        "ratio_empty_grouped_numerator",
        vec![
            answer(N, &["cluster", "value"], &[], false),
            answer(
                D,
                &["cluster", "value"],
                &[
                    vec![json!("alpha"), json!("100")],
                    vec![json!("beta"), json!("50")],
                ],
                false,
            ),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert!(!reads.is_empty(), "no read at all");
    for r in &reads {
        assert!(
            !is_ok(r),
            "an empty numerator answer read {:?} for {:?}",
            r.reading,
            r.dims
        );
    }
}

/// D20's guard holds against near-miss keys: a numerator NULL-cluster row the denominator has no
/// twin for, and a numerator key padded by a `char(n)` column, each make the pass one unknown,
/// and nothing reads 0.
#[test]
fn adv_sql_ratio_near_miss_numerator_keys_never_infer_zero() {
    const N: &str = "SELECT cluster, count(*) AS value FROM failures GROUP BY cluster";
    const D: &str = "SELECT cluster, count(*) AS value FROM requests GROUP BY cluster";
    for (name, key) in [("null", Value::Null), ("padded", json!("alpha     "))] {
        let dir = world(
            &format!("near_miss_{name}"),
            vec![
                answer(
                    N,
                    &["cluster", "value"],
                    &[vec![key.clone(), json!("3")]],
                    false,
                ),
                answer(
                    D,
                    &["cluster", "value"],
                    &[
                        vec![json!("alpha"), json!("100")],
                        vec![json!("beta"), json!("50")],
                    ],
                    false,
                ),
            ],
        );
        let reads = read(&dir, &ratio(N, D, &["cluster"]));
        assert_eq!(reads.len(), 1, "{name}: {reads:#?}");
        assert!(!is_ok(&reads[0]), "{name}: {:?}", reads[0].reading);
    }
}

/// D20 needs both answers whole: a whole numerator and a cut denominator infer nothing.
#[test]
fn adv_sql_ratio_cut_denominator_infers_no_zero() {
    const N: &str = "SELECT cluster, count(*) AS value FROM failures GROUP BY cluster";
    const D: &str = "SELECT cluster, count(*) AS value FROM requests GROUP BY cluster";
    let dir = world(
        "ratio_cut_denominator",
        vec![
            answer(
                N,
                &["cluster", "value"],
                &[vec![json!("c000"), json!("3")]],
                false,
            ),
            answer(D, &["cluster", "value"], &thousand_rows("100"), true),
        ],
    );
    let reads = read(&dir, &ratio(N, D, &["cluster"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("cut"), "{:?}", reads[0].reading);
}

/// The significant-digit division, through the adapter, against quotients worked by hand.
#[test]
fn adv_sql_ratio_significant_digit_division_is_exact() {
    for (name, n, d, expected) in [
        ("round_up_carry", "9.999999999999999999999", "1", "10"),
        ("both_huge", "1e300", "1e300", "1"),
        ("scaled_divisor", "7", "0.07", "100"),
        ("trailing_zero_divisor", "1", "0.0400", "25"),
        ("small_quotient", "15", "556000", "0.000026978417266187"),
        ("negative", "-2", "3", "-0.666666666666666667"),
        (
            "large_quotient",
            "1",
            "3e-18",
            "333333333333333333.333333333333333333",
        ),
        ("float_text", "1.5e-05", "3e-05", "0.5"),
    ] {
        let nq = format!("SELECT n AS value FROM t /* {name} */");
        let dq = format!("SELECT d AS value FROM t /* {name} */");
        let dir = world(
            &format!("division_{name}"),
            vec![
                answer(&nq, &["value"], &[vec![json!(n)]], false),
                answer(&dq, &["value"], &[vec![json!(d)]], false),
            ],
        );
        let reads = read(&dir, &ratio(&nq, &dq, &[]));
        assert_eq!(reads.len(), 1, "{name}: {reads:#?}");
        match &reads[0].reading {
            Reading::Ok(r) => assert_eq!(r.value.0, expected, "{name}: {n} / {d}"),
            other => panic!("{name}: {n} / {d} read {other:?}"),
        }
    }
}

/// `sql.count` grouping: rows are counted per declared-dimension series, an undeclared column
/// splits nothing, and the count is the row count, not the value of any cell.
#[test]
fn adv_sql_count_groups_by_declared_dimensions_only() {
    const Q: &str = "SELECT cluster, reason, value FROM open_incidents";
    let dir = world(
        "count_groups",
        vec![answer(
            Q,
            &["cluster", "reason", "value"],
            &[
                vec![json!("alpha"), json!("oom"), json!("7")],
                vec![json!("alpha"), json!("timeout"), json!("7")],
                vec![json!("beta"), json!("oom"), json!("0")],
            ],
            false,
        )],
    );
    let reads = read(
        &dir,
        &definition(
            "sql.count",
            SignalKind::CountOfCases,
            &[("value", Q)],
            &["cluster"],
        ),
    );
    let by = |c: &str| {
        reads
            .iter()
            .find(|r| r.dims.get("cluster").map(String::as_str) == Some(c))
            .unwrap_or_else(|| panic!("no read for {c}: {reads:#?}"))
    };
    assert_eq!(reads.len(), 2, "{reads:#?}");
    for (c, n) in [("alpha", "2"), ("beta", "1")] {
        match &by(c).reading {
            Reading::Ok(r) => assert_eq!(r.value.0, n, "{c}"),
            other => panic!("{c}: {other:?}"),
        }
    }
}
