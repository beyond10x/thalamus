//! Adversary pass 2, `story:m3-vocabulary-mapping`, the `thalamus-sources` half: a `partial` read
//! under `producer_mapped`, and the `sql` adapter's typing of whitespace-only, empty and padded
//! cells, read end to end through the extractor registry.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use thalamus_model::definition::{
    AdapterId, Author, BaselineKind, ConnectionId, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::evaluation::{Baseline, BaselineLabel, Verdict};
use thalamus_model::observation::{
    Bound, CompleteReading, PartialReading, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Read, Unread};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;
use thalamus_sources::vocabulary::map_reads;

const QUERY: &str = "SELECT cluster, value FROM board_colour";

fn colour_definition() -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("gate.ci.board_colour".into()),
        title: "Colour of the build board".into(),
        family: Family("gate".into()),
        kind: SignalKind::Gauge,
        unit: Some(Unit("{verdict}".into())),
        dimensions: Some(vec![DimensionKey("cluster".into())]),
        source: Source {
            adapter: AdapterId("sql".into()),
            connection: ConnectionId("reports-replica".into()),
        },
        extractor: ExtractorId("sql.value".into()),
        query: BTreeMap::from([("value".to_owned(), QueryText(QUERY.into()))]),
        window: Duration("PT1H".into()),
        cadence: Duration("PT1H".into()),
        history: History::Store,
        denominator: None,
        evaluation: EvaluationSpec {
            rule: RuleId("producer_mapped".into()),
            baselines: vec![BaselineKind::Fixed],
            warn: None,
            crit: None,
            min_trend_window: None,
            fixed: Some(Decimal("0".into())),
            rolling_n: None,
            mapping: Some(BTreeMap::from([
                ("green".to_owned(), Verdict::OK),
                ("yellow".to_owned(), Verdict::WARN),
                ("red".to_owned(), Verdict::CRIT),
                ("na".to_owned(), Verdict::UNKNOWN),
                ("3".to_owned(), Verdict::OK),
            ])),
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

fn partial_read(raw: &str, unread: Option<Unread>) -> Read {
    Read {
        dims: BTreeMap::new(),
        reading: Reading::Partial(PartialReading {
            value: Decimal("3".into()),
            bound: Bound::Lower,
        }),
        window: TimeWindow {
            start: Timestamp("2026-10-14T21:00:00Z".into()),
            end: Timestamp("2026-10-14T22:00:00Z".into()),
        },
        denominator: None,
        adapter: AdapterId("sql".into()),
        connection: ConnectionId("reports-replica".into()),
        query_sha256: QueryDigest("00".into()),
        raw: Some(raw.into()),
        retention_start: None,
        unread,
    }
}

/// D44: "Every other read, `partial` or `unknown` for any other reason ..., keeps its own reading
/// and reason whatever text it carries". A partial read whose `raw` is a colour the table maps
/// (`red`, `green`), or a number it maps (`3`), passes `map_reads` unchanged, with or without
/// the `Text` type. Holds the `Reading::Partial(_) => false` arm of `map_reads`; nothing else in
/// the suite hands it a partial read.
#[test]
fn adv_partial_read_carrying_mapped_text_keeps_its_reading() {
    let definition = colour_definition();
    for raw in ["red", "green", "3"] {
        for unread in [None, Some(Unread::Text)] {
            let read = partial_read(raw, unread);
            let mapped = map_reads(&definition, vec![read.clone()]);
            assert_eq!(
                mapped,
                vec![read],
                "partial, raw `{raw}`, unread {unread:?}"
            );
        }
    }
}

/// One test's directory: the `sql` recordings of `common`, one `query.read` answer of `rows`,
/// and the Connectors double, as `tests/vocabulary.rs` builds it.
fn sql_world(name: &str, rows: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv_vocabulary_p2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let common = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sql/common");
    for entry in std::fs::read_dir(common).expect("common recordings") {
        let path = entry.expect("fixture entry").path();
        std::fs::copy(&path, dir.join(path.file_name().expect("file name")))
            .expect("copy a recording");
    }
    let result = serde_json::json!({
        "columns": [
            {"name": "cluster", "native_type": "text"},
            {"name": "value", "native_type": "text"}
        ],
        "rows": serde_json::from_str::<serde_json::Value>(rows).expect("rows are JSON"),
        "truncated": false,
        "provenance": {
            "instance": "sql",
            "resource": "reports",
            "observed_at_unix_ms": 1_792_018_623_000_u64,
            "source_revision": null
        }
    });
    let recording = serde_json::json!({
        "input": {"query": QUERY},
        "stdout": {
            "ok": true,
            "result": {
                "adapter": "sql",
                "operation": "query.read",
                "revision": "op-rev-1",
                "result": result.to_string()
            }
        }
    });
    std::fs::write(
        dir.join("operations+invoke+a=sql+c=reports-replica+o=query.read.json"),
        recording.to_string(),
    )
    .expect("write the recording");
    std::os::unix::fs::symlink(
        env!("CARGO_BIN_EXE_connectors-double"),
        dir.join("connectors"),
    )
    .expect("link the double");
    dir
}

/// The reads of `rows` through the extractor registry, keyed by cluster.
fn sql_reads(name: &str, rows: &str) -> BTreeMap<String, Read> {
    let dir = sql_world(name, rows);
    registry::read(
        &Connectors::new(dir.join("connectors")),
        &colour_definition(),
        &Timestamp("2026-10-14T22:57:03Z".into()),
        &RunId("run_test".into()),
    )
    .into_iter()
    .map(|read| {
        let cluster = read.dims.get("cluster").cloned().unwrap_or_default();
        (cluster, read)
    })
    .collect()
}

/// D44: a cell "present and not a number" is `Unread::Text` and is matched exactly. A
/// whitespace-only cell, an empty one and a padded colour are text, are kept as written, and map
/// to `UNKNOWN` (none is in the table), never trimmed onto `green`. A NULL cell is no value and
/// keeps the adapter's own reason. The one exact colour maps.
#[test]
fn adv_sql_whitespace_empty_and_padded_cells_map_only_by_exact_text() {
    let reads = sql_reads(
        "whitespace",
        r#"[["a", "  "], ["b", ""], ["c", " green"], ["d", "green "], ["e", null], ["f", "green"]]"#,
    );
    assert_eq!(reads.len(), 6, "one read per row: {reads:#?}");
    for (cluster, raw) in [("a", "  "), ("b", ""), ("c", " green"), ("d", "green ")] {
        let read = &reads[cluster];
        assert_eq!(read.raw.as_deref(), Some(raw), "{cluster}: {read:?}");
        assert_eq!(read.unread, Some(Unread::Text), "{cluster}: {read:?}");
        let Reading::Unknown(unread) = &read.reading else {
            panic!("{cluster} `{raw}` reads {:?}", read.reading);
        };
        assert!(
            unread.reason.contains(&format!("`{raw}`")),
            "{cluster}: the reason names the cell: {unread:?}"
        );
    }
    let null = &reads["e"];
    assert_eq!((null.raw.as_deref(), null.unread), (None, None), "{null:?}");
    let Reading::Unknown(unread) = &null.reading else {
        panic!("NULL reads {:?}", null.reading);
    };
    assert!(
        !unread.reason.contains("evaluation.mapping"),
        "a NULL cell keeps the adapter's reason: {unread:?}"
    );
    let exact = &reads["f"];
    assert_eq!(
        (exact.raw.as_deref(), exact.unread),
        (Some("green"), Some(Unread::Text)),
        "{exact:?}"
    );
    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".into()),
        }),
    };
    let graded = thalamus_core::grading::grade(&colour_definition(), &stored(exact, 1), &fixed)
        .expect("the definition grades");
    assert_eq!(
        (graded.verdict, graded.score.clone()),
        (Verdict::OK, Some(Decimal("0".into()))),
        "{exact:?}: {graded:?}"
    );
}

/// The sample `read` becomes at revision `rev`: the reading as `map_reads` left it, and the
/// producer's text as `vocabulary::sample_text` records it.
fn stored(read: &Read, rev: i64) -> SignalSampleData {
    SignalSampleData {
        text: thalamus_sources::vocabulary::sample_text(&colour_definition(), read),
        sample_id: SampleId("smp_1".into()),
        signal: SignalId("gate.ci.board_colour".into()),
        series: SeriesKey("gate.ci.board_colour".into()),
        dims: BTreeMap::new(),
        ts: read.window.end.clone(),
        window: read.window.clone(),
        reading: read.reading.clone(),
        unit: Unit("{verdict}".into()),
        denominator: None,
        provenance: Provenance {
            producer: "thalamus-service".into(),
            version: "0.1.0".into(),
            definition_rev: rev,
            adapter: read.adapter.clone(),
            connection: read.connection.clone(),
            query_sha256: read.query_sha256.clone(),
            run_id: RunId("run_1".into()),
        },
    }
}

/// Design § 2: "Keeping the evaluation apart from the sample lets a threshold change re-grade
/// history without reading the source again"; § 5.1: `thalamus-core` "re-grades history after a
/// threshold change"; D5: "a re-grade after a revision is a new evaluation". Under
/// `producer_mapped` the table is the threshold. A sample read at revision 1, `red` under
/// `red: CRIT`, re-graded at revision 2, whose table says `red: WARN`, must be `WARN`; and
/// `purple`, unlisted at revision 1 and `OK` at revision 2, must be `OK`. Today `map_reads`
/// applies revision 1's table before the sample is stored and the sample keeps no producer text,
/// so the re-grade at revision 2 returns revision 1's verdicts.
#[test]
fn adv_a_mapping_revision_re_grades_stored_history() {
    let rev1 = colour_definition();
    let mut rev2 = colour_definition();
    rev2.rev = 2;
    let table = rev2.evaluation.mapping.as_mut().expect("a table");
    table.insert("red".to_owned(), Verdict::WARN);
    table.insert("purple".to_owned(), Verdict::OK);

    let fixed = Baseline {
        kind: BaselineLabel("fixed".into()),
        offsets: Vec::new(),
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0".into()),
        }),
    };
    let mut seen = Vec::new();
    for (raw, at_rev2) in [("red", Verdict::WARN), ("purple", Verdict::OK)] {
        let mut read = partial_read(raw, Some(Unread::Text));
        read.reading = Reading::Unknown(UnreadReading {
            reason: format!("{raw:?} in column `value` is not a number"),
        });
        let read = map_reads(&rev1, vec![read]).remove(0);
        let regraded =
            thalamus_core::grading::grade(&rev2, &stored(&read, 1), &fixed).expect("rev 2 grades");
        seen.push((raw, at_rev2, regraded.verdict, regraded.reason));
    }
    assert!(
        seen.iter().all(|(_, want, got, _)| want == got),
        "re-graded at rev 2, (producer value, rev 2's verdict, verdict given, reason): {seen:?}"
    );
}
