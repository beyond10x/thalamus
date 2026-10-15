//! `story:m3-vocabulary-mapping`, the `thalamus-sources` half: a read keeps its own quality as the
//! adapter made it (`vocabulary::map_reads` rewrites nothing), and under `producer_mapped` the
//! sample records the producer's own value, `vocabulary::sample_text`, which `thalamus-core`
//! `grade()` maps through the definition's `evaluation.mapping` onto the one ladder (design § 2.3,
//! L1, L3).
//!
//! The known answers are `tests/fixtures/vocabulary.json`: the tables a definition writes, never
//! code, and per producer value the verdict and its score.

use std::collections::BTreeMap;

use serde_json::Value;
use thalamus_core::grading::{self, Graded};
use thalamus_core::ladder;
use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Threshold,
    Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Read, Unread};
use thalamus_sources::vocabulary::{map_reads, sample_text};

const FIXTURE: &str = include_str!("fixtures/vocabulary.json");
const SIGNAL: &str = "gate.ci.board_colour";
const MAPPED: &str = "producer_mapped";

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("the fixture is JSON")
}

fn verdict(name: &str) -> Verdict {
    match name {
        "OK" => Verdict::OK,
        "WARN" => Verdict::WARN,
        "CRIT" => Verdict::CRIT,
        "UNKNOWN" => Verdict::UNKNOWN,
        other => panic!("the fixture names no verdict {other}"),
    }
}

fn table(name: &str) -> BTreeMap<String, Verdict> {
    let fixture = fixture();
    let Some(Value::Object(table)) = fixture.get("tables").and_then(|tables| tables.get(name))
    else {
        panic!("the fixture has no table {name}");
    };
    table
        .iter()
        .map(|(raw, name)| (raw.clone(), verdict(name.as_str().expect("a verdict name"))))
        .collect()
}

/// One known answer.
#[derive(Debug)]
struct Case {
    table: String,
    raw: String,
    verdict: Verdict,
    /// The verdict's score; `None` when it carries none.
    score: Option<String>,
}

fn cases(of: &str) -> Vec<Case> {
    let fixture = fixture();
    let Some(Value::Array(cases)) = fixture.get("cases") else {
        panic!("the fixture has no cases");
    };
    let text = |case: &Value, key: &str| case.get(key).and_then(Value::as_str).map(str::to_owned);
    let picked: Vec<Case> = cases
        .iter()
        .map(|case| Case {
            table: text(case, "table").expect("table"),
            raw: text(case, "raw").expect("raw"),
            verdict: verdict(&text(case, "verdict").expect("verdict")),
            score: text(case, "score"),
        })
        .filter(|case| case.table == of)
        .collect();
    assert!(!picked.is_empty(), "the fixture has no case of {of}");
    picked
}

/// A definition under `rule`. A threshold rule carries `warn` and `crit` lines; `producer_mapped`
/// carries none (`thresholds-unused`).
fn definition(rule: &str, mapping: Option<BTreeMap<String, Verdict>>) -> SignalDefinitionData {
    let line = |abs: &str| {
        (rule != MAPPED).then(|| Threshold {
            x: None,
            abs: Some(Decimal(abs.into())),
            min_events: None,
        })
    };
    SignalDefinitionData {
        signal_id: SignalId(SIGNAL.into()),
        title: "Colour of the build board".into(),
        family: Family("gate".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("{verdict}".into())),
        dimensions: Some(Vec::new()),
        source: Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("ci-results".into()),
        },
        extractor: ExtractorId("sql.value".into()),
        query: BTreeMap::new(),
        window: Duration("PT1H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Store,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId(rule.into()),
            baselines: vec![BaselineKind::Fixed],
            warn: line("1"),
            crit: line("2"),
            min_trend_window: None,
            fixed: Some(Decimal("0".into())),
            rolling_n: None,
            mapping,
        },
        persistence: None,
        objective: None,
        owner: Some(Owner("team:gate".into())),
        links: Links {
            components: Vec::new(),
            issues: Vec::new(),
        },
        rev: 1,
        revised_at: Timestamp("2026-10-01T00:00:00Z".into()),
        revised_by: Author("someone".into()),
        mute_reason: None,
        mute_until: None,
    }
}

fn mapped(table_name: &str) -> SignalDefinitionData {
    definition(MAPPED, Some(table(table_name)))
}

/// The read an adapter makes of the producer value `raw`, as the `sql` adapter does: a decimal is
/// an `ok` reading of that number; any other text is `unknown`, typed [`Unread::Text`], since the
/// value was read and is text. Either way the producer's text is kept as `raw`.
fn adapter_read(raw: &str) -> Read {
    let (reading, unread) = if is_decimal(raw) {
        let reading = Reading::Ok(CompleteReading {
            value: Decimal(raw.into()),
        });
        (reading, None)
    } else {
        let reading = Reading::Unknown(UnreadReading {
            reason: format!("value `{raw}` in column `value` is not a number"),
        });
        (reading, Some(Unread::Text))
    };
    Read {
        dims: BTreeMap::new(),
        reading,
        window: TimeWindow {
            start: Timestamp("2026-10-14T21:00:00Z".into()),
            end: Timestamp("2026-10-14T22:00:00Z".into()),
        },
        denominator: None,
        adapter: AdapterId("sql".into()),
        connection: ConnectionId("ci-results".into()),
        query_sha256: QueryDigest("00".into()),
        raw: Some(raw.into()),
        retention_start: None,
        unread,
    }
}

/// An optional `-`, digits, and optionally `.` and digits.
fn is_decimal(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, "0"));
    [whole, fraction]
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// The sample `read` becomes under `definition`: the read's own reading and the text
/// [`sample_text`] records.
fn sample_of(definition: &SignalDefinitionData, read: &Read) -> SignalSampleData {
    SignalSampleData {
        sample_id: SampleId("smp_1".into()),
        signal: SignalId(SIGNAL.into()),
        series: SeriesKey(SIGNAL.into()),
        dims: BTreeMap::new(),
        ts: read.window.end.clone(),
        window: read.window.clone(),
        reading: read.reading.clone(),
        text: sample_text(definition, read),
        unit: Unit("{verdict}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: definition.rev,
            adapter: read.adapter.clone(),
            connection: read.connection.clone(),
            query_sha256: read.query_sha256.clone(),
            run_id: RunId("run_1".into()),
        },
    }
}

fn grade(definition: &SignalDefinitionData, read: &Read) -> Graded {
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".into()),
        }),
    };
    grading::grade(definition, &sample_of(definition, read), &fixed).expect("the definition grades")
}

/// Every known answer of `table_name`: the read passes `map_reads` unchanged, its sample records
/// the producer value, and grading gives the case's verdict and score.
fn holds_known_answers(table_name: &str) {
    let definition = mapped(table_name);
    for case in cases(table_name) {
        let read = adapter_read(&case.raw);
        assert_eq!(
            map_reads(&definition, vec![read.clone()]),
            vec![read.clone()],
            "the read keeps its own quality: {case:?}"
        );
        assert_eq!(
            sample_text(&definition, &read).as_deref(),
            Some(case.raw.as_str()),
            "the sample records the producer value: {case:?}"
        );
        let graded = grade(&definition, &read);
        assert_eq!(graded.verdict, case.verdict, "{case:?}: {graded:?}");
        assert_eq!(
            graded.score,
            case.score.clone().map(Decimal),
            "{case:?}: {graded:?}"
        );
        if graded.verdict == Verdict::UNKNOWN {
            assert!(
                graded.reason.contains(&format!("`{}`", case.raw)),
                "the reason names the producer value: {case:?}: {graded:?}"
            );
        }
    }
}

#[test]
fn colour_ladder_maps_to_ok_warn_crit_unknown() {
    holds_known_answers("colour");
}

#[test]
fn detector_exit_1_is_crit_and_2_is_unknown() {
    holds_known_answers("detector");
}

#[test]
fn blocking_gate_nonzero_exit_is_crit() {
    holds_known_answers("blocking_gate");
    let definition = mapped("blocking_gate");
    for code in 1..=255 {
        let graded = grade(&definition, &adapter_read(&code.to_string()));
        assert_eq!(graded.verdict, Verdict::CRIT, "exit {code}: {graded:?}");
    }
}

/// L3: "not read" never ranks with "green". The worst of a green and an `na` is `UNKNOWN`,
/// whichever arrives first.
#[test]
fn colour_na_never_ranks_as_green() {
    let definition = mapped("colour");
    let verdict_of = |raw: &str| grade(&definition, &adapter_read(raw)).verdict;
    for order in [["green", "na"], ["na", "green"]] {
        let worst = ladder::worst(order.map(verdict_of));
        assert_eq!(worst, Some(Verdict::UNKNOWN), "worst of {order:?}");
    }
}

/// Unknown is never OK: a value no table lists, in any table, is `UNKNOWN`; a sample with a number
/// and no producer text is never graded on its number; and a read the adapter could not make
/// (cut, colliding, unreachable) records no text, whatever `raw` it carries, and keeps its own
/// reason.
#[test]
fn unrecognised_producer_value_is_unknown_never_ok() {
    for table_name in ["colour", "detector", "blocking_gate"] {
        let definition = mapped(table_name);
        for raw in ["unlisted", "OK", "", " 0", "0 ", "00", "+1"] {
            let graded = grade(&definition, &adapter_read(raw));
            assert_eq!(graded.verdict, Verdict::UNKNOWN, "{table_name} `{raw}`");
            assert!(graded.reason.contains(&format!("`{raw}`")), "{graded:?}");
        }

        let mut textless = adapter_read("0");
        textless.raw = None;
        assert_eq!(sample_text(&definition, &textless), None);
        let graded = grade(&definition, &textless);
        assert_eq!(
            graded.verdict,
            Verdict::UNKNOWN,
            "{table_name}: a number with no producer text is not graded: {graded:?}"
        );

        for raw in [None, Some("green"), Some("0")] {
            let mut unread = adapter_read("x");
            unread.raw = raw.map(str::to_owned);
            unread.unread = None;
            unread.reading = Reading::Unknown(UnreadReading {
                reason: "the result is cut".into(),
            });
            assert_eq!(sample_text(&definition, &unread), None, "raw {raw:?}");
            let graded = grade(&definition, &unread);
            assert_eq!(
                (graded.verdict, graded.reason.as_str()),
                (Verdict::UNKNOWN, "the result is cut"),
                "{table_name}, raw {raw:?}"
            );
        }

        let mut partial = adapter_read("green");
        partial.reading = Reading::Partial(PartialReading {
            value: Decimal("0".into()),
            bound: Bound::Lower,
        });
        assert_eq!(
            sample_text(&definition, &partial),
            None,
            "a bound is no value"
        );
    }
}

/// `map_reads` rewrites no read, under any rule; a sample records producer text only under
/// `producer_mapped`, so under every other rule a sample is as it was.
#[test]
fn other_rules_pass_reads_unchanged() {
    let reads: Vec<Read> = ["green", "1", "0.25", "na"]
        .into_iter()
        .map(adapter_read)
        .collect();
    for rule in [
        "ratio_vs_pooled_baseline",
        "higher_is_worse",
        "lower_is_worse",
        "unknown_rule",
        MAPPED,
    ] {
        for mapping in [None, Some(table("colour"))] {
            let definition = definition(rule, mapping.clone());
            assert_eq!(
                map_reads(&definition, reads.clone()),
                reads,
                "{rule}, mapping {mapping:?}"
            );
            if rule != MAPPED {
                for read in &reads {
                    assert_eq!(sample_text(&definition, read), None, "{rule}: {read:?}");
                }
            }
        }
    }
}

/// The rules behind `thalamus.evaluation.RecordEvaluation/outcome/reason-missing` and
/// `.../score-not-admitted`: every mapped verdict other than `OK` carries a reason, and a verdict
/// that is not scored, `UNKNOWN` here, carries no score.
#[test]
fn mapped_non_ok_verdict_carries_reason_and_unknown_no_score() {
    for table_name in ["colour", "detector", "blocking_gate"] {
        let definition = mapped(table_name);
        for case in cases(table_name) {
            let graded = grade(&definition, &adapter_read(&case.raw));
            if graded.verdict != Verdict::OK {
                assert!(!graded.reason.is_empty(), "{case:?}: {graded:?}");
            }
            if ladder::scored(graded.verdict) {
                assert!(graded.score.is_some(), "{case:?}: {graded:?}");
            } else {
                assert_eq!(graded.score, None, "{case:?}: {graded:?}");
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// End to end: a recorded source answer, through the extractor registry, the sample and grade().
// ---------------------------------------------------------------------------------------------

/// One test's directory with the `sql` recordings of `common` and `scenario` and the Connectors
/// double, as `tests/sql.rs` builds it.
fn sql_world(name: &str, scenario: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("vocabulary")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sql");
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
    dir
}

/// The reads of recording `scenario` through the extractor registry, under `producer_mapped`
/// with `mapping`, each with the grade of the sample it becomes.
fn sql_graded(
    name: &str,
    scenario: &str,
    mapping: BTreeMap<String, Verdict>,
) -> Vec<(Read, Graded)> {
    let dir = sql_world(name, scenario);
    let mut definition = definition(MAPPED, Some(mapping));
    definition.source.connection = ConnectionId("reports-replica".into());
    definition.dimensions = Some(vec![DimensionKey("cluster".into())]);
    definition.query = BTreeMap::from([(
        "value".to_owned(),
        QueryText("SELECT cluster, value FROM queue_depth".into()),
    )]);
    thalamus_sources::registry::read(
        &thalamus_sources::connectors::Connectors::new(dir.join("connectors")),
        &definition,
        &Timestamp("2026-10-14T22:57:03Z".into()),
        &RunId("run_test".into()),
    )
    .into_iter()
    .map(|read| {
        let graded = grade(&definition, &read);
        (read, graded)
    })
    .collect()
}

/// The `sql` adapter reads a text cell `green` (recording `tests/fixtures/sql/non-numeric`,
/// cluster `alpha`) as an `unknown` read whose only defect is that the value is text, and keeps
/// it so. Under `producer_mapped` with the colour table its sample grades `OK`; the other cells
/// of the same answer (`NaN`, `Infinity`, empty, `12 ms`) are values the table does not list, so
/// `UNKNOWN`, each reason naming its cell.
#[test]
fn sql_text_cell_green_grades_ok_end_to_end() {
    let graded = sql_graded(
        "sql_text_cell_green_grades_ok_end_to_end",
        "non-numeric",
        table("colour"),
    );
    assert_eq!(graded.len(), 5, "one read per row: {graded:#?}");
    for (read, graded) in &graded {
        let cluster = read.dims.get("cluster").map(String::as_str);
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "the read keeps the adapter's quality: {read:?}"
        );
        assert_eq!(read.unread, Some(Unread::Text), "{read:?}");
        let raw = read.raw.as_deref().expect("the cell is kept");
        if cluster == Some("alpha") {
            assert_eq!(raw, "green");
            assert_eq!(graded.verdict, Verdict::OK, "{read:?}: {graded:?}");
            assert_eq!(graded.score, Some(Decimal("0".into())));
        } else {
            assert_eq!(graded.verdict, Verdict::UNKNOWN, "{read:?}: {graded:?}");
            assert!(
                graded.reason.contains(&format!("`{raw}`")),
                "{cluster:?}: {graded:?}"
            );
        }
    }
}

/// Only a read whose one defect is text records it. A NULL cell (`tests/fixtures/sql/null-cell`)
/// and a series the query returned twice (`tests/fixtures/sql/duplicate`) stay `UNKNOWN` with the
/// adapter's own reason, though the table maps every number those answers hold to `OK`; the
/// cells that were read, numbers here, map by their text.
#[test]
fn sql_null_and_duplicate_cells_stay_unknown_under_producer_mapped() {
    let every_number: BTreeMap<String, Verdict> = ["1", "2", "3", "4"]
        .into_iter()
        .map(|n| (n.to_owned(), Verdict::OK))
        .collect();
    for (scenario, unread, read_ok) in [
        ("null-cell", "beta", "alpha"),
        ("duplicate", "alpha", "beta"),
    ] {
        let graded = sql_graded(
            &format!("sql_{scenario}_under_producer_mapped"),
            scenario,
            every_number.clone(),
        );
        for (read, graded) in &graded {
            let cluster = read.dims.get("cluster").map(String::as_str);
            if cluster == Some(unread) {
                assert_eq!(read.unread, None, "{scenario}: {read:?}");
                assert_eq!(
                    graded.verdict,
                    Verdict::UNKNOWN,
                    "{scenario}: {read:?}: {graded:?}"
                );
                assert!(
                    !graded.reason.contains("evaluation.mapping"),
                    "{scenario}: {graded:?}"
                );
            } else {
                assert_eq!(cluster, Some(read_ok), "{scenario}: {read:?}");
                assert_eq!(
                    graded.verdict,
                    Verdict::OK,
                    "{scenario}: {read:?}: {graded:?}"
                );
            }
        }
        assert!(
            graded
                .iter()
                .any(|(read, _)| read.dims.get("cluster").map(String::as_str) == Some(unread)),
            "{scenario}: no read of {unread}: {graded:#?}"
        );
    }
}
