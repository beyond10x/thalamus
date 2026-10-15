//! The Eventlog store on both providers it opens, SQLite and PostgreSQL: what an instance records
//! survives a restart in one stream, comes back per series in `ts` order, and keeps its numbers
//! and its unknowns exactly as they were recorded.
//!
//! The PostgreSQL cases run in the database `THALAMUS_TEST_POSTGRES_URL` names (`task
//! test-postgres` starts a disposable one); without it they fail, naming the variable. Every case
//! writes its own instance stream, so cases share the database and never each other's records.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::{
    EvaluationId, RecordEvaluation, RecordEvaluationOutcome, SignalEvaluationData, Verdict,
};
use thalamus_model::observation::{
    Bound, CompleteReading, Denominator, PartialReading, Provenance, QueryDigest, Reading, RunId,
    SampleId, SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::evaluations::DUPLICATE_EVALUATION;
use thalamus_store::eventlog::{
    EVALUATION_SCHEMA, EventlogStore, ID_TAKEN, LOCATOR_UNKNOWN, Locator, POSTGRES_PASSWORD,
    SAMPLE_SCHEMA, SQLITE_NOT_A_FILE, StoreError,
};

const SIGNAL: &str = "platform.namespace.request_failure_ratio";
const SERIES_A: &str = r#"platform.namespace.request_failure_ratio{namespace="shared-1"}"#;
const SERIES_B: &str = r#"platform.namespace.request_failure_ratio{namespace="shared-2"}"#;

/// The PostgreSQL database the PostgreSQL cases run in.
fn postgres_url() -> String {
    std::env::var("THALAMUS_TEST_POSTGRES_URL").unwrap_or_else(|_| {
        panic!(
            "THALAMUS_TEST_POSTGRES_URL is not set: the PostgreSQL cases need a disposable local \
             database (`task test-postgres` starts one)"
        )
    })
}

/// Where a case's store lives and the instance stream it writes.
#[derive(Clone)]
struct Target {
    locator: Locator,
    instance: String,
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.locator, self.instance)
    }
}

/// One target per provider. The instance is named after `root`, so every case writes its own
/// stream, also in the one PostgreSQL database every case shares.
fn targets(root: &Path) -> [Target; 2] {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a tempdir name");
    let instance = format!("case-{}{name}", std::process::id());
    [
        Target {
            locator: Locator::Sqlite(root.join("sqlite-store").join("thalamus.sqlite")),
            instance: instance.clone(),
        },
        Target {
            locator: postgres_url()
                .parse()
                .expect("THALAMUS_TEST_POSTGRES_URL is a PostgreSQL URL without a password"),
            instance,
        },
    ]
}

fn open(target: &Target) -> EventlogStore {
    EventlogStore::open(&target.locator, &target.instance)
        .unwrap_or_else(|error| panic!("{target}: {error}"))
}
fn ts(seconds: u32) -> Timestamp {
    Timestamp(format!("2026-10-14T10:00:{seconds:02}Z"))
}

fn sample(id: &str, series: &str, at: u32, reading: Reading) -> SignalSampleData {
    let namespace = if series == SERIES_A {
        "shared-1"
    } else {
        "shared-2"
    };
    SignalSampleData {
        text: None,
        sample_id: SampleId(id.to_owned()),
        signal: SignalId(SIGNAL.to_owned()),
        series: SeriesKey(series.to_owned()),
        dims: BTreeMap::from([("namespace".to_owned(), DimensionValue(namespace.to_owned()))]),
        ts: ts(at),
        window: TimeWindow {
            start: Timestamp("2026-10-14T04:00:00Z".to_owned()),
            end: ts(at),
        },
        reading,
        unit: Unit("1".to_owned()),
        denominator: Some(Denominator {
            n: 556,
            unit: Unit("{request}".to_owned()),
            since: Timestamp("2025-03-02T00:00:00Z".to_owned()),
            age_s: 51_403_023,
        }),
        provenance: Provenance {
            producer: "thalamus-service".to_owned(),
            version: "0.1.0".to_owned(),
            definition_rev: 1,
            adapter: AdapterId("sql".to_owned()),
            connection: ConnectionId("replica".to_owned()),
            query_sha256: QueryDigest("0".repeat(64)),
            run_id: RunId("run_01JA0000000000000000000000".to_owned()),
        },
    }
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    })
}

fn evaluation(id: &str, of: &SignalSampleData, rev: i64, verdict: Verdict) -> SignalEvaluationData {
    SignalEvaluationData {
        evaluation_id: EvaluationId(id.to_owned()),
        sample_id: of.sample_id.clone(),
        signal: of.signal.clone(),
        series: of.series.clone(),
        dims: of.dims.clone(),
        ts: of.ts.clone(),
        definition_rev: rev,
        verdict,
        prev: None,
        score: (verdict == Verdict::OK).then(|| Decimal("0.25".to_owned())),
        baseline: None,
        reason: (verdict != Verdict::OK).then(|| "over the line".to_owned()),
    }
}

fn ids_of_samples(samples: &[SignalSampleData]) -> Vec<&str> {
    samples.iter().map(|s| s.sample_id.0.as_str()).collect()
}

/// D5: at most one evaluation per `(sample, definition_rev)`. The second is refused inside the
/// append with a stable code, nothing is written, and a re-grade at the next revision is a new
/// evaluation.
#[test]
fn second_evaluation_at_same_rev_is_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        let first = sample("smp_01", SERIES_A, 1, ok("0.029"));
        {
            let mut store = open(&locator);
            store.append_sample(&first).expect("the sample is stored");
            store
                .append_evaluation(&evaluation("evl_01", &first, 1, Verdict::OK))
                .expect("the first evaluation at rev 1 is stored");
            let head = store.head();
            let records = store.records().len();

            let refused = store.append_evaluation(&evaluation("evl_02", &first, 1, Verdict::WARN));
            match refused {
                Err(StoreError::Refused { code }) => assert_eq!(code, DUPLICATE_EVALUATION),
                other => panic!("{locator}: a second evaluation at rev 1 was {other:?}"),
            }
            assert_eq!(store.head(), head, "{locator}: the stream did not move");
            assert_eq!(
                store.records().len(),
                records,
                "{locator}: nothing was kept"
            );
            assert!(
                store
                    .evaluation(&EvaluationId("evl_02".to_owned()))
                    .is_none()
            );
        }
        let mut store = open(&locator);
        let stored: Vec<_> = store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(EVALUATION_SCHEMA))
            .collect();
        assert_eq!(stored.len(), 1, "{locator}: one evaluation after a restart");
        match store.append_evaluation(&evaluation("evl_03", &first, 1, Verdict::CRIT)) {
            Err(StoreError::Refused { code }) => assert_eq!(code, DUPLICATE_EVALUATION),
            other => panic!("{locator}: after a restart the duplicate was {other:?}"),
        }
        store
            .append_evaluation(&evaluation("evl_04", &first, 2, Verdict::WARN))
            .expect("a re-grade at rev 2 is a new evaluation");
    }
}

#[test]
fn reopened_store_reads_series_in_ts_order() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        {
            let mut store = open(&locator);
            for (id, series, at) in [
                ("smp_03", SERIES_A, 3),
                ("smp_01", SERIES_A, 1),
                ("smp_09", SERIES_B, 2),
                ("smp_02", SERIES_A, 2),
            ] {
                store
                    .append_sample(&sample(id, series, at, ok("1")))
                    .expect("stored");
            }
        }
        let store = open(&locator);
        let range = store.series_range(&SeriesKey(SERIES_A.to_owned()), &ts(0), &ts(59));
        assert_eq!(
            ids_of_samples(&range.samples),
            ["smp_01", "smp_02", "smp_03"],
            "{locator}"
        );
        let view = thalamus_core::behaviour::observation::series_samples(
            &store,
            &SeriesKey(SERIES_A.to_owned()),
        )
        .expect("the SeriesSamples view answers");
        let view: Vec<_> = view.iter().map(|row| row.sample_id.0.as_str()).collect();
        assert_eq!(view, ["smp_01", "smp_02", "smp_03"], "{locator}");
    }
}

#[test]
fn range_read_returns_samples_and_evaluations_of_one_series() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        let mut store = open(&locator);
        let a1 = sample("smp_a1", SERIES_A, 10, ok("1"));
        let a2 = sample("smp_a2", SERIES_A, 5, ok("2"));
        let a3 = sample("smp_a3", SERIES_A, 40, ok("3"));
        let b1 = sample("smp_b1", SERIES_B, 7, ok("4"));
        for s in [&a1, &a2, &a3, &b1] {
            store.append_sample(s).expect("stored");
        }
        for e in [
            evaluation("evl_a1", &a1, 1, Verdict::OK),
            evaluation("evl_a2", &a2, 1, Verdict::WARN),
            evaluation("evl_a3", &a3, 1, Verdict::OK),
            evaluation("evl_b1", &b1, 1, Verdict::CRIT),
        ] {
            store.append_evaluation(&e).expect("stored");
        }
        // [00:05, 00:30): a2 and a1 of series A; a3 is past the end, b1 another series.
        let range = store.series_range(&SeriesKey(SERIES_A.to_owned()), &ts(5), &ts(30));
        assert_eq!(
            ids_of_samples(&range.samples),
            ["smp_a2", "smp_a1"],
            "{locator}"
        );
        let evaluations: Vec<_> = range
            .evaluations
            .iter()
            .map(|e| e.evaluation_id.0.as_str())
            .collect();
        assert_eq!(evaluations, ["evl_a2", "evl_a1"], "{locator}");
        assert_eq!(
            range.samples[0], a2,
            "{locator}: the whole sample comes back"
        );
    }
}

#[test]
fn first_stored_sample_of_series() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        {
            let mut store = open(&locator);
            assert!(
                store
                    .first_sample(&SeriesKey(SERIES_A.to_owned()))
                    .is_none()
            );
            for (id, series, at) in [
                ("smp_b", SERIES_B, 1),
                ("smp_a_first", SERIES_A, 20),
                ("smp_a_earlier_ts", SERIES_A, 10),
            ] {
                store
                    .append_sample(&sample(id, series, at, ok("1")))
                    .expect("stored");
            }
        }
        let store = open(&locator);
        let first = store
            .first_sample(&SeriesKey(SERIES_A.to_owned()))
            .expect("series A has samples");
        assert_eq!(first.sample_id.0, "smp_a_first", "{locator}");
        let first_b = store.first_sample(&SeriesKey(SERIES_B.to_owned()));
        assert_eq!(
            first_b.map(|s| s.sample_id.0),
            Some("smp_b".to_owned()),
            "{locator}"
        );
        assert!(store.first_sample(&SeriesKey("other".to_owned())).is_none());
    }
}

/// The record a sample is stored as, by id, from the provider's own history.
fn stored_record<'a>(store: &'a EventlogStore, id: &str) -> &'a Value {
    store
        .records()
        .iter()
        .find(|record| record.body().get("sample_id").and_then(Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("no record holds {id}"))
        .body()
}

#[test]
fn unknown_reading_round_trips_without_value() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        let unknown = sample(
            "smp_unknown",
            SERIES_A,
            1,
            Reading::Unknown(UnreadReading {
                reason: "timeout".to_owned(),
            }),
        );
        open(&locator).append_sample(&unknown).expect("stored");
        let store = open(&locator);
        let back = store
            .sample(&SampleId("smp_unknown".to_owned()))
            .expect("the sample is there after a restart");
        assert_eq!(back, unknown, "{locator}");
        let reading = &stored_record(&store, "smp_unknown")["reading"];
        assert_eq!(
            reading,
            &serde_json::json!({"quality": "unknown", "value": {"reason": "timeout"}}),
            "{locator}: an unknown reading is stored with no value at all"
        );
    }
}

#[test]
fn decimal_round_trips_exactly() {
    let root = tempfile::tempdir().expect("tempdir");
    let values = [
        "0.0290",
        "-0.000",
        "12345678901234567890.123456789012345678901",
        "0.0000001",
    ];
    for locator in targets(root.path()) {
        let mut stored = Vec::new();
        {
            let mut store = open(&locator);
            for (index, value) in values.iter().enumerate() {
                let id = format!("smp_d{index}");
                let reading = if index % 2 == 0 {
                    ok(value)
                } else {
                    Reading::Partial(PartialReading {
                        value: Decimal((*value).to_owned()),
                        bound: Bound::Lower,
                    })
                };
                let s = sample(&id, SERIES_A, 1, reading);
                store.append_sample(&s).expect("stored");
                let mut e = evaluation(&format!("evl_d{index}"), &s, 1, Verdict::OK);
                e.score = Some(Decimal((*value).to_owned()));
                store.append_evaluation(&e).expect("stored");
                stored.push((s, e));
            }
        }
        let store = open(&locator);
        for (index, (s, e)) in stored.iter().enumerate() {
            assert_eq!(store.sample(&s.sample_id).as_ref(), Some(s), "{locator}");
            assert_eq!(
                store.evaluation(&e.evaluation_id).as_ref(),
                Some(e),
                "{locator}"
            );
            let value = &stored_record(&store, &s.sample_id.0)["reading"]["value"]["value"];
            assert_eq!(
                value,
                &Value::String(values[index].to_owned()),
                "{locator}: a Decimal is stored as its decimal string (D11)"
            );
        }
        // A number the published decimal pattern refuses would be written and never read back:
        // it is refused before the append, and nothing is written.
        let mut store = open(&locator);
        let head = store.head();
        let exponent = sample("smp_exponent", SERIES_A, 1, ok("1e-7"));
        assert!(
            matches!(store.append_sample(&exponent), Err(StoreError::Record(_))),
            "{locator}"
        );
        assert_eq!(store.head(), head, "{locator}");
        assert!(store.sample(&exponent.sample_id).is_none(), "{locator}");
    }
}

/// The first object in `value` whose `schema` member is `schema`, depth first.
fn find_schema<'a>(value: &'a Value, schema: &str) -> Option<&'a Value> {
    match value {
        Value::Object(members) => {
            if members.get("schema").and_then(Value::as_str) == Some(schema) {
                return Some(value);
            }
            members
                .values()
                .find_map(|member| find_schema(member, schema))
        }
        Value::Array(items) => items.iter().find_map(|item| find_schema(item, schema)),
        _ => None,
    }
}

/// Whether any object in `value` carries the member `key`.
fn has_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(members) => {
            members.contains_key(key) || members.values().any(|member| has_key(member, key))
        }
        Value::Array(items) => items.iter().any(|item| has_key(item, key)),
        _ => false,
    }
}

/// Design § 6.2: stored lines are not CloudEvents-wrapped. Each provider hands the record back as
/// the event body, and the row the database itself holds (SQLite's `data` text, PostgreSQL's
/// `data` JSONB) is the `thalamus.sample/1` record, with no `specversion` envelope around it.
#[test]
fn stored_line_is_not_cloudevents_wrapped() {
    let root = tempfile::tempdir().expect("tempdir");
    let s = sample("smp_line", SERIES_A, 1, ok("0.029"));
    for locator in targets(root.path()) {
        let mut store = open(&locator);
        store.append_sample(&s).expect("stored");
        let body = stored_record(&store, "smp_line");
        assert_eq!(body["schema"], SAMPLE_SCHEMA, "{locator}");
        assert!(!has_key(body, "specversion"), "{locator}: {body}");
        let keys: Vec<&str> = body
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        let mut expected = vec![
            "denominator",
            "dims",
            "provenance",
            "reading",
            "sample_id",
            "schema",
            "series",
            "signal",
            "ts",
            "unit",
            "window",
        ];
        let mut keys = keys;
        keys.sort_unstable();
        expected.sort_unstable();
        assert_eq!(keys, expected, "{locator}: the record is the § 2.2 sample");
        drop(store);
        let rows = raw_rows(&locator);
        assert!(
            !rows.is_empty(),
            "{locator}: no stored row holds the sample"
        );
        for row in rows {
            let parsed: Value = serde_json::from_str(&row).expect("a stored row is JSON");
            assert!(
                !has_key(&parsed, "specversion"),
                "{locator}: a CloudEvents envelope: {row}"
            );
            assert_eq!(
                find_schema(&parsed, SAMPLE_SCHEMA).map(|record| record == &parsed),
                Some(true),
                "{locator}: the stored row is not the thalamus.sample/1 record itself: {row}"
            );
            assert_eq!(parsed["sample_id"], "smp_line", "{locator}");
        }
    }
}

/// The `data` column of every event row of `target`'s instance, as the database stores it.
fn raw_rows(target: &Target) -> Vec<String> {
    match &target.locator {
        Locator::Sqlite(path) => {
            let connection = rusqlite::Connection::open(path).expect("the database opens");
            let mut query = connection
                .prepare("SELECT data FROM thalamus_events WHERE tenant_id = ?1")
                .expect("the events table");
            query
                .query_map([&target.instance], |row| row.get::<_, String>(0))
                .expect("rows")
                .collect::<Result<_, _>>()
                .expect("text rows")
        }
        Locator::Postgres(url) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            runtime.block_on(async {
                let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
                    .await
                    .expect("the database answers");
                tokio::spawn(connection);
                client
                    .query(
                        "SELECT data::text FROM thalamus_events WHERE tenant_id = $1",
                        &[&target.instance],
                    )
                    .await
                    .expect("the events table")
                    .into_iter()
                    .map(|row| row.get::<_, String>(0))
                    .collect()
            })
        }
    }
}

/// The newest evaluation of each series (latest `ts`, a re-grade at one instant winning), and the
/// samples a re-grade at a revision still has to evaluate; an evaluation of a sample the stream
/// does not hold is refused.
#[test]
fn newest_evaluations_and_unevaluated_samples() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        {
            let mut store = open(&locator);
            let a1 = sample("smp_a1", SERIES_A, 1, ok("1"));
            let a2 = sample("smp_a2", SERIES_A, 2, ok("2"));
            let b1 = sample("smp_b1", SERIES_B, 1, ok("3"));
            for s in [&a2, &a1, &b1] {
                store.append_sample(s).expect("stored");
            }
            for e in [
                evaluation("evl_a2r1", &a2, 1, Verdict::OK),
                evaluation("evl_a1r1", &a1, 1, Verdict::WARN),
                evaluation("evl_a2r2", &a2, 2, Verdict::CRIT),
                evaluation("evl_b1r1", &b1, 1, Verdict::OK),
            ] {
                store.append_evaluation(&e).expect("stored");
            }
            let missing = sample("smp_missing", SERIES_A, 3, ok("1"));
            match store.append_evaluation(&evaluation("evl_missing", &missing, 1, Verdict::OK)) {
                Err(StoreError::Refused { code }) => {
                    assert_eq!(code, thalamus_store::evaluations::UNKNOWN_SAMPLE);
                }
                other => panic!("{locator}: an evaluation of no stored sample was {other:?}"),
            }
        }
        let store = open(&locator);
        let newest: Vec<_> = store
            .newest_evaluations()
            .into_iter()
            .map(|e| e.evaluation_id.0)
            .collect();
        assert_eq!(newest, ["evl_a2r2", "evl_b1r1"], "{locator}");
        let signal = SignalId(SIGNAL.to_owned());
        assert_eq!(
            ids_of_samples(&store.unevaluated_samples(&signal, 2)),
            ["smp_a1", "smp_b1"],
            "{locator}"
        );
        assert!(
            store.unevaluated_samples(&signal, 1).is_empty(),
            "{locator}"
        );
        assert!(
            store
                .unevaluated_samples(&SignalId("other.signal".to_owned()), 1)
                .is_empty(),
            "{locator}"
        );
    }
}

/// The code a refused locator answers.
fn refusal_code<T>(answer: Result<T, StoreError>) -> Option<&'static str> {
    match answer {
        Err(StoreError::Locator { code, .. }) => Some(code),
        _ => None,
    }
}

#[test]
fn locator_names_the_provider() {
    assert_eq!(
        "sqlite:instance/thalamus.sqlite"
            .parse::<Locator>()
            .expect("a sqlite locator"),
        Locator::Sqlite(PathBuf::from("instance/thalamus.sqlite"))
    );
    for url in [
        "postgres://thalamus@localhost:5432/thalamus",
        "postgresql://thalamus@127.0.0.1/thalamus",
    ] {
        assert_eq!(
            url.parse::<Locator>().expect("a PostgreSQL locator"),
            Locator::Postgres(url.to_owned())
        );
    }
    for unknown in [
        "file:instance/store",
        "sqlite:",
        "instance/store",
        "mysql://h/db",
    ] {
        assert_eq!(
            refusal_code(unknown.parse::<Locator>()),
            Some(LOCATOR_UNKNOWN),
            "{unknown}"
        );
    }
    for password in [
        "postgres://thalamus:secret@localhost/thalamus",
        "postgresql://thalamus:@localhost/thalamus",
    ] {
        assert_eq!(
            refusal_code(password.parse::<Locator>()),
            Some(POSTGRES_PASSWORD),
            "{password}"
        );
        assert_eq!(
            refusal_code(EventlogStore::open(
                &Locator::Postgres(password.to_owned()),
                "instance-a"
            )),
            Some(POSTGRES_PASSWORD),
            "{password}: a locator built without parsing is refused at open too"
        );
    }
    assert_eq!(
        Locator::Sqlite(PathBuf::from("a/b.sqlite")).to_string(),
        "sqlite:a/b.sqlite"
    );
}

/// A SQLite locator that names no database file is refused at open with its code, and nothing is
/// created: an in-memory database or a URI would not keep what the instance records.
#[test]
fn sqlite_locator_naming_no_file_is_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    let directory = root.path().join("a-directory");
    std::fs::create_dir(&directory).expect("mkdir");
    let mut refused = vec![
        ":memory:".to_owned(),
        "file:thalamus.sqlite?mode=memory".to_owned(),
        format!("{}/", root.path().join("ends-in-a-slash").display()),
        directory.display().to_string(),
    ];
    refused.push(String::new());
    for path in refused {
        assert_eq!(
            refusal_code(EventlogStore::open(
                &Locator::Sqlite(PathBuf::from(&path)),
                "instance-a"
            )),
            Some(SQLITE_NOT_A_FILE),
            "sqlite:{path}"
        );
    }
    assert!(!Path::new(":memory:").exists());
    assert!(!root.path().join("ends-in-a-slash").exists());
}

/// The id `prefix` and a sequence number make, as the behaviours mint it.
fn minted(prefix: &str, number: u64) -> String {
    format!("{prefix}{number:012}")
}

/// Records an evaluation of `sample` at `rev` through the behaviour, which mints its id from the
/// store's own sequence, and answers that id.
fn record_through_behaviour(store: &mut EventlogStore, sample: &SampleId, rev: i64) -> String {
    let outcome = thalamus_core::behaviour::evaluation::record_evaluation(
        store,
        RecordEvaluation {
            sample_id: sample.clone(),
            definition_rev: rev,
            verdict: Verdict::OK,
            prev: None,
            score: None,
            baseline: None,
            reason: None,
        },
    )
    .expect("the behaviour answers");
    match outcome {
        RecordEvaluationOutcome::Recorded { response, .. } => response.evaluation_id.0,
        other => panic!("not recorded: {other:?}"),
    }
}

/// Ids are store-local: `smp_`/`evl_` and a never-decreasing per-prefix sequence the store
/// resumes on open. Three handles, one after the other, each opened from a freshly parsed
/// locator as a new process would, never issue an id the stream already passed: every id is
/// above every id stored before it, including a sample id whose lower numbers the stream never
/// held (expired rows, D13).
#[test]
fn reopened_store_never_reissues_an_id() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        let text = locator.locator.to_string();
        let fresh = || {
            open(&Target {
                locator: text.parse::<Locator>().expect("the locator parses again"),
                instance: locator.instance.clone(),
            })
        };
        let seeded = SampleId(minted("smp_", 7));
        let mut issued: Vec<String> = Vec::new();
        {
            let mut store = fresh();
            store
                .append_sample(&sample(&seeded.0, SERIES_A, 1, ok("1")))
                .expect("stored");
            issued.push(record_through_behaviour(&mut store, &seeded, 1));
        }
        for rev in [2, 3] {
            let mut store = fresh();
            let number = IdSequence::next_number(&mut store, "smp_").expect("a number");
            let id = minted("smp_", number);
            assert!(
                id > seeded.0,
                "{locator}: {id} was issued after the stream held {}",
                seeded.0
            );
            store
                .append_sample(&sample(&id, SERIES_A, 2, ok("2")))
                .expect("stored");
            issued.push(id.clone());
            issued.push(record_through_behaviour(&mut store, &seeded, rev));
            issued.push(record_through_behaviour(&mut store, &SampleId(id), rev));
        }
        let evaluations: Vec<&String> = issued.iter().filter(|id| id.starts_with("evl_")).collect();
        let samples: Vec<&String> = issued.iter().filter(|id| id.starts_with("smp_")).collect();
        for ids in [&evaluations, &samples] {
            for pair in ids.windows(2) {
                assert!(pair[0] < pair[1], "{locator}: {} then {}", pair[0], pair[1]);
            }
        }
        let store = fresh();
        assert_eq!(store.evaluations().len(), evaluations.len(), "{locator}");
        assert_eq!(store.samples().len(), samples.len() + 1, "{locator}");
    }
}

/// D24: a number is reserved on the instance's own id stream before it is answered, so a number
/// one handle answered is never answered to another handle, stored or not, and a reservation is
/// never a record on the instance stream (the § 2.2 and § 2.3 shapes stay the only records, and
/// the open replays nothing more).
#[test]
fn minting_reserves_each_number_off_the_instance_stream() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut first = open(&target);
        let mut second = open(&target);
        let reserved: Vec<u64> = (0..3)
            .map(|_| IdSequence::next_number(&mut first, "smp_").expect("a number"))
            .collect();
        assert_eq!(reserved, [1, 2, 3], "{target}");
        assert_eq!(
            IdSequence::next_number(&mut second, "smp_"),
            Some(4),
            "{target}: a number another handle reserved was answered again"
        );
        assert_eq!(
            IdSequence::next_number(&mut second, "evl_"),
            Some(1),
            "{target}: each prefix keeps its own sequence"
        );
        assert_eq!(
            IdSequence::next_number(&mut first, "smp_"),
            Some(5),
            "{target}"
        );
        let fresh = open(&target);
        assert_eq!(
            (fresh.head(), fresh.records().len()),
            (0, 0),
            "{target}: a reservation is a record on the instance stream"
        );
    }
}

/// `append_evaluation` documents `ID_TAKEN` for an id held with another body. A handle that has
/// not yet seen the other handle's evaluation answers that refusal, not a raw Eventlog error;
/// the same body from a stale handle is a retry and stores nothing.
#[test]
fn stale_handle_storing_a_taken_evaluation_id_is_refused_id_taken() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut first = open(&target);
        let graded = sample("smp_graded", SERIES_A, 1, ok("1"));
        first.append_sample(&graded).expect("stored");
        let mut other_body = open(&target);
        let mut same_body = open(&target);
        let stored = evaluation("evl_taken", &graded, 1, Verdict::OK);
        first.append_evaluation(&stored).expect("stored");
        match other_body.append_evaluation(&evaluation("evl_taken", &graded, 2, Verdict::WARN)) {
            Err(StoreError::Refused { code }) if code == ID_TAKEN => {}
            other => panic!("{target}: another body under a stored evaluation id: {other:?}"),
        }
        same_body
            .append_evaluation(&stored)
            .unwrap_or_else(|error| panic!("{target}: the same evaluation again: {error}"));
        let fresh = open(&target);
        assert_eq!(fresh.evaluations(), [stored], "{target}");
        assert_eq!(fresh.head(), 2, "{target}");
    }
}

/// The refusal code of a PostgreSQL locator that asks for TLS.
const POSTGRES_TLS: &str = "thalamus.store.postgres-tls";

/// D24: TLS, Eventlog's `verified` connection, is not wired yet, so a locator that asks for it
/// (any `sslmode` but `disable`, or a direct TLS negotiation) is refused rather than answered in
/// plaintext. An absent `sslmode` and `sslmode=disable` open as before.
#[test]
fn postgres_locator_asking_for_tls_is_refused() {
    let base = "postgresql://thalamus@127.0.0.1/thalamus";
    for query in [
        "?sslmode=require",
        "?sslmode=prefer",
        "?sslmode=verify-full",
        "?application_name=thalamus&sslmode=require",
        "?ssl%6Dode=require",
        "?sslmode=disable&sslmode=require",
        "?sslnegotiation=direct",
    ] {
        let url = format!("{base}{query}");
        match url.parse::<Locator>() {
            Err(error @ StoreError::Locator { code, .. }) => {
                assert_eq!(code, POSTGRES_TLS, "{url}");
                assert!(error.to_string().contains("verified"), "{url}: {error}");
            }
            other => panic!("{url}: {other:?}"),
        }
        assert_eq!(
            refusal_code(EventlogStore::open(
                &Locator::Postgres(url.clone()),
                "instance-a"
            )),
            Some(POSTGRES_TLS),
            "{url}: a locator built without parsing is refused at open too"
        );
    }
    for url in [base.to_owned(), format!("{base}?sslmode=disable")] {
        assert_eq!(
            url.parse::<Locator>().expect("admitted"),
            Locator::Postgres(url.clone())
        );
    }
}

#[test]
fn producer_text_round_trips_and_is_absent_when_none() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in targets(root.path()) {
        let mut mapped = sample(
            "smp_text",
            SERIES_A,
            1,
            Reading::Unknown(UnreadReading {
                reason: "value `green` is not a number".to_owned(),
            }),
        );
        mapped.text = Some("green".to_owned());
        let plain = sample("smp_plain", SERIES_A, 2, ok("1"));
        let mut writer = open(&locator);
        writer.append_sample(&mapped).expect("stored");
        writer.append_sample(&plain).expect("stored");
        let store = open(&locator);
        let back = store
            .sample(&SampleId("smp_text".to_owned()))
            .expect("the sample is there after a restart");
        assert_eq!(back, mapped, "{locator}");
        assert_eq!(
            stored_record(&store, "smp_text")["text"],
            serde_json::json!("green"),
            "{locator}"
        );
        assert!(
            !has_key(stored_record(&store, "smp_plain"), "text"),
            "{locator}: a sample without producer text stores no text member"
        );
    }
}
