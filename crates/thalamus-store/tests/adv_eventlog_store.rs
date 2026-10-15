//! Adversary cases for the Eventlog store (wave 20261015d, pass 1): the store driven against the
//! design document it claims to write, against the core's own SeriesSamples view, across a second
//! handle, a damaged stream, a foreign record, an async caller and the boundaries its own suite
//! leaves open.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eventlog_core::{CommandMeta, EventStore, Expected, NewEvent, StreamId, TenantId};
use serde_json::Value;
use tempfile::TempDir;
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_core::conformance::{self, Report};
use thalamus_model::assertion;
use thalamus_model::behaviour::{
    AssertionStorage, DimensionRegistryStorage, DimensionStorage, SignalDefinitionStorage,
    SignalEvaluationStorage, SignalSampleStorage,
};
use thalamus_model::definition::{self, AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::{self, DimensionValue};
use thalamus_model::evaluation::{
    self, Baseline, BaselineLabel, EvaluationId, RecordEvaluation, RecordEvaluationOutcome,
    SignalEvaluationData, Verdict,
};
use thalamus_model::observation::{
    self, CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId,
    SeriesKey, SignalSampleData, TimeWindow, UnreadReading,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_store::eventlog::{EventlogStore, ID_TAKEN, Locator, StoreError};

const INSTANCE: &str = "instance-adv";
const SIGNAL: &str = "platform.namespace.request_failure_ratio";
const SERIES: &str = r#"platform.namespace.request_failure_ratio{namespace="shared-1"}"#;

/// The PostgreSQL database `THALAMUS_TEST_POSTGRES_URL` names.
fn postgres_locator() -> Locator {
    std::env::var("THALAMUS_TEST_POSTGRES_URL")
        .unwrap_or_else(|_| {
            panic!(
                "THALAMUS_TEST_POSTGRES_URL is not set: the PostgreSQL cases need a disposable \
                 local database (`task test-postgres` starts one)"
            )
        })
        .parse()
        .expect("THALAMUS_TEST_POSTGRES_URL is a PostgreSQL URL without a password")
}

/// Where a case's store lives and the instance stream it writes.
#[derive(Clone)]
struct Target {
    locator: Locator,
    instance: String,
}

impl std::fmt::Display for Target {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} ({})", self.locator, self.instance)
    }
}

/// An instance stream named after `root`, so a case never reads another case's records in the
/// one PostgreSQL database every case shares.
fn instance_of(root: &Path) -> String {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a tempdir name");
    format!("adv-{}{name}", std::process::id())
}

fn sqlite(root: &Path) -> Target {
    Target {
        locator: Locator::Sqlite(root.join("sqlite-store").join("thalamus.sqlite")),
        instance: instance_of(root),
    }
}

fn postgres(root: &Path) -> Target {
    Target {
        locator: postgres_locator(),
        instance: instance_of(root),
    }
}

fn locators(root: &Path) -> [Target; 2] {
    [sqlite(root), postgres(root)]
}

fn open(target: &Target) -> EventlogStore {
    EventlogStore::open(&target.locator, &target.instance)
        .unwrap_or_else(|error| panic!("{target}: {error}"))
}

fn sample_at(id: &str, ts: &str, reading: Reading) -> SignalSampleData {
    SignalSampleData {
        sample_id: SampleId(id.to_owned()),
        signal: SignalId(SIGNAL.to_owned()),
        series: SeriesKey(SERIES.to_owned()),
        dims: BTreeMap::from([(
            "namespace".to_owned(),
            DimensionValue("shared-1".to_owned()),
        )]),
        ts: Timestamp(ts.to_owned()),
        window: TimeWindow {
            start: Timestamp("2026-10-14T04:00:00Z".to_owned()),
            end: Timestamp(ts.to_owned()),
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
            run_id: RunId("run_000000000001".to_owned()),
        },
    }
}

fn sample(id: &str) -> SignalSampleData {
    sample_at(id, "2026-10-14T10:00:01Z", ok("0.029"))
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.to_owned()),
    })
}

fn evaluation_of(id: &str, of: &SignalSampleData, rev: i64) -> SignalEvaluationData {
    SignalEvaluationData {
        evaluation_id: EvaluationId(id.to_owned()),
        sample_id: of.sample_id.clone(),
        signal: of.signal.clone(),
        series: of.series.clone(),
        dims: of.dims.clone(),
        ts: of.ts.clone(),
        definition_rev: rev,
        verdict: Verdict::OK,
        prev: None,
        score: Some(Decimal("0.25".to_owned())),
        baseline: None,
        reason: None,
    }
}

fn manifest(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

// ---------------------------------------------------------------------------------------------
// The unit's own contract: design § 2.2 and § 2.3 name the stored records.
// ---------------------------------------------------------------------------------------------

/// The first ```json block after `heading` in the design document, parsed.
fn design_record(heading: &str) -> Value {
    let path = manifest("../../docs/design/thalamus.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let after = &text[text
        .find(heading)
        .unwrap_or_else(|| panic!("no heading {heading}"))..];
    let start = after.find("```json\n").expect("a json block") + "```json\n".len();
    let block = &after[start..];
    let block = &block[..block.find("```").expect("the block ends")];
    serde_json::from_str(block).unwrap_or_else(|e| panic!("{heading}: {e}\n{block}"))
}

fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Every top-level member the document shows and the store writes, by name and JSON kind.
fn shape(record: &Value) -> BTreeMap<String, &'static str> {
    record
        .as_object()
        .expect("an object")
        .iter()
        .map(|(name, value)| (name.clone(), kind(value)))
        .collect()
}

/// The story: "records are the wire forms `thalamus.sample/1` and `thalamus.evaluation/1` of
/// design § 2.2 and § 2.3". The stored record and the document's record have the same members,
/// each of the same JSON kind.
#[test]
fn adv_stored_records_have_the_shape_design_2_2_and_2_3_document() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut store = open(&sqlite(root.path()));
    let mut s = sample("smp_000000000042");
    s.dims
        .insert("cluster".to_owned(), DimensionValue("prod-a".to_owned()));
    store.append_sample(&s).expect("stored");
    let mut e = evaluation_of("evl_000000000042", &s, 1);
    e.verdict = Verdict::WARN;
    e.prev = Some(Verdict::OK);
    e.score = Some(Decimal("1.3".to_owned()));
    e.reason = Some("0.029 >= 2 x 0.013".to_owned());
    e.baseline = Some(Baseline {
        kind: BaselineLabel("pooled".to_owned()),
        offsets: vec!["1d".to_owned(), "7d".to_owned()],
        reading: ok("0.013"),
    });
    store.append_evaluation(&e).expect("stored");
    let records: Vec<&Value> = store.records().iter().map(|r| r.body()).collect();
    let mut drift = Vec::new();
    for (heading, stored) in [
        ("### 2.2 `SignalSample`", records[0]),
        ("### 2.3 `SignalEvaluation`", records[1]),
    ] {
        let documented = shape(&design_record(heading));
        let written = shape(stored);
        if documented != written {
            drift.push(format!(
                "{heading}\n  documented: {documented:?}\n  stored:     {written:?}"
            ));
        }
    }
    assert!(drift.is_empty(), "{}", drift.join("\n"));
}

// ---------------------------------------------------------------------------------------------
// The range read against the core's SeriesSamples view.
// ---------------------------------------------------------------------------------------------

/// `range.rs` calls the range read "the store's read of the `thalamus.observation.SeriesSamples`
/// view, bounded in time". Over a range holding every sample, it orders them as the view does.
/// The generated decoder admits any text as a `Timestamp`; the view reads instants with the ESS
/// reference grammar (`Instant::parse`), the range read with the `time` crate, which also takes
/// a space separator and a leap second.
#[test]
fn adv_range_read_orders_as_the_series_samples_view() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let mut store = open(&locator);
        for (id, ts) in [
            ("smp_000000000001", "2026-10-14 10:00:05Z"),
            ("smp_000000000002", "2026-10-14T10:00:01+02:00"),
            ("smp_000000000003", "2026-10-14T09:00:00Z"),
        ] {
            store
                .append_sample(&sample_at(id, ts, ok("1")))
                .expect("stored");
        }
        let series = SeriesKey(SERIES.to_owned());
        let range = store.series_range(
            &series,
            &Timestamp("2026-10-14T00:00:00Z".to_owned()),
            &Timestamp("2026-10-15T00:00:00Z".to_owned()),
        );
        let ranged: Vec<String> = range
            .samples
            .iter()
            .map(|s| s.sample_id.0.clone())
            .collect();
        let view: Vec<String> =
            thalamus_core::behaviour::observation::series_samples(&store, &series)
                .expect("the view answers")
                .into_iter()
                .map(|row| row.sample_id.0)
                .collect();
        assert_eq!(
            ranged, view,
            "{locator}: range read and SeriesSamples view disagree"
        );
    }
}

/// `series_range` is `[from, to)`: a sample exactly at `to` is outside, one exactly at `from`
/// inside (the M4 API story's `series_range_is_half_open` rests on it).
#[test]
fn adv_range_read_is_half_open_at_both_ends() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let mut store = open(&locator);
        for (id, ts) in [
            ("smp_000000000001", "2026-10-14T10:00:00Z"),
            ("smp_000000000002", "2026-10-14T10:00:30Z"),
            ("smp_000000000003", "2026-10-14T12:00:30+02:00"),
        ] {
            store
                .append_sample(&sample_at(id, ts, ok("1")))
                .expect("stored");
        }
        let range = store.series_range(
            &SeriesKey(SERIES.to_owned()),
            &Timestamp("2026-10-14T10:00:00Z".to_owned()),
            &Timestamp("2026-10-14T10:00:30Z".to_owned()),
        );
        let ids: Vec<&str> = range
            .samples
            .iter()
            .map(|s| s.sample_id.0.as_str())
            .collect();
        assert_eq!(ids, ["smp_000000000001"], "{locator}");
    }
}

// ---------------------------------------------------------------------------------------------
// Retries and identity.
// ---------------------------------------------------------------------------------------------

/// `samples.rs`: "storing a sample already held with the same body is a retry and writes
/// nothing, and storing another body under a held id is refused". The same for evaluations.
#[test]
fn adv_retry_writes_nothing_and_another_body_is_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let mut store = open(&locator);
        let s = sample("smp_000000000001");
        store.append_sample(&s).expect("stored");
        store.append_sample(&s).expect("a retry is answered");
        assert_eq!(store.head(), 1, "{locator}: a retry wrote");
        let other = sample_at("smp_000000000001", "2026-10-14T10:00:02Z", ok("0.5"));
        match store.append_sample(&other) {
            Err(StoreError::Refused { code }) => assert_eq!(code, ID_TAKEN, "{locator}"),
            answer => panic!("{locator}: another body under a held sample id was {answer:?}"),
        }
        let e = evaluation_of("evl_000000000001", &s, 1);
        store.append_evaluation(&e).expect("stored");
        store.append_evaluation(&e).expect("a retry is answered");
        assert_eq!(store.head(), 2, "{locator}: a retry wrote");
        let mut changed = e.clone();
        changed.definition_rev = 2;
        match store.append_evaluation(&changed) {
            Err(StoreError::Refused { code }) => assert_eq!(code, ID_TAKEN, "{locator}"),
            answer => panic!("{locator}: another body under a held evaluation id was {answer:?}"),
        }
        drop(store);
        let store = open(&locator);
        assert_eq!(store.head(), 2, "{locator}");
        assert_eq!(store.sample(&s.sample_id), Some(s.clone()), "{locator}");
    }
}

/// A stream holding malformed ids and an id of the other prefix: the sequence of each prefix
/// resumes after the highest well-formed number it holds, and ignores the rest.
#[test]
fn adv_sequence_resumes_past_foreign_and_malformed_ids() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        {
            let mut store = open(&locator);
            for id in [
                "smp_12ab",
                "evl_000000000050",
                "smp_000000000007",
                "smp_",
                "nounderscore",
            ] {
                store.append_sample(&sample(id)).expect("stored");
            }
        }
        let mut store = open(&locator);
        assert_eq!(
            IdSequence::next_number(&mut store, "smp_"),
            Some(8),
            "{locator}"
        );
        assert_eq!(
            IdSequence::next_number(&mut store, "evl_"),
            Some(51),
            "{locator}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Two handles on one instance.
// ---------------------------------------------------------------------------------------------

/// Two handles on one instance's stream, as `thalamus eval` beside a running scheduler would
/// hold: a sample one handle stored is never answered `unknown-sample` by the other. Either the
/// other handle sees the sample, or it fails loudly; a declared outcome that says the sample does
/// not exist is a wrong answer.
#[test]
fn adv_second_handle_never_answers_unknown_sample_for_a_stored_sample() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let mut first = open(&locator);
        let mut second = open(&locator);
        let s = sample("smp_000000000001");
        first.append_sample(&s).expect("stored by the first handle");
        let answer = catch_unwind(AssertUnwindSafe(|| {
            thalamus_core::behaviour::evaluation::record_evaluation(
                &mut second,
                RecordEvaluation {
                    sample_id: s.sample_id.clone(),
                    definition_rev: 1,
                    verdict: Verdict::OK,
                    prev: None,
                    score: None,
                    baseline: None,
                    reason: None,
                },
            )
        }));
        if let Ok(Ok(RecordEvaluationOutcome::UnknownSample { .. })) = answer {
            panic!(
                "{locator}: the second handle answered unknown-sample for {} which is on the stream",
                s.sample_id.0
            );
        }
    }
}

/// Design § 5.3: the service's scheduler owns the cadence and "the CLI can sample one definition
/// on demand", so a long-lived handle shares the stream with a short-lived one. After the
/// short-lived handle appended, the long-lived one still records the next sample, and sees both.
#[test]
fn adv_long_lived_handle_records_after_another_handle_appended() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let mut service = open(&locator);
        {
            let mut cli = open(&locator);
            cli.append_sample(&sample("smp_000000000001"))
                .expect("stored by the on-demand handle");
        }
        let answer = service.append_sample(&sample_at(
            "smp_000000000002",
            "2026-10-14T10:00:02Z",
            ok("0.5"),
        ));
        assert!(
            answer.is_ok(),
            "{locator}: the long-lived handle cannot record after another handle appended: {answer:?}"
        );
        assert_eq!(service.samples().len(), 2, "{locator}");
    }
}

// ---------------------------------------------------------------------------------------------
// The SQLite locator.
// ---------------------------------------------------------------------------------------------

/// `Locator::Sqlite` is "the SQLite provider, in this database file". `sqlite::memory:` parses
/// to a path SQLite reads as an in-memory database: a store opened there either keeps what it
/// recorded across a reopen or is refused at open.
#[test]
fn adv_sqlite_locator_is_a_file_or_refused() {
    let locator: Locator = "sqlite::memory:".parse().expect("the locator parses");
    let Ok(mut store) = EventlogStore::open(&locator, INSTANCE) else {
        return;
    };
    store
        .append_sample(&sample("smp_000000000001"))
        .expect("stored");
    drop(store);
    let reopened = EventlogStore::open(&locator, INSTANCE).expect("reopens");
    assert!(
        !Path::new(":memory:").exists(),
        "the probe created a file named :memory: in the crate directory"
    );
    assert_eq!(
        reopened.samples().len(),
        1,
        "{locator}: the sample recorded a moment ago is gone after a reopen; nothing is durable"
    );
}

// ---------------------------------------------------------------------------------------------
// Damaged streams: open fails loudly, never answers fewer or altered records.
// ---------------------------------------------------------------------------------------------

/// Two samples on a fresh store at `locator`, the second with reading `0.029`.
fn seeded(locator: &Target) {
    let mut store = open(locator);
    store
        .append_sample(&sample_at(
            "smp_000000000001",
            "2026-10-14T10:00:00Z",
            ok("0.5"),
        ))
        .expect("stored");
    store
        .append_sample(&sample("smp_000000000002"))
        .expect("stored");
}

/// What a reopen answers after the damage: an error, or the two samples exactly as recorded.
fn assert_loud_or_intact(locator: &Target, what: &str) {
    match EventlogStore::open(&locator.locator, &locator.instance) {
        Err(_) => {}
        Ok(store) => {
            let values: Vec<String> = store
                .samples()
                .iter()
                .map(|s| match &s.reading {
                    Reading::Ok(reading) => reading.value.0.clone(),
                    other => format!("{other:?}"),
                })
                .collect();
            assert_eq!(
                values,
                ["0.5", "0.029"],
                "{locator}: after {what} the store opened and answered other records"
            );
        }
    }
}

fn replace_bytes(path: &Path, from: &[u8], to: &[u8]) -> bool {
    let Ok(mut bytes) = std::fs::read(path) else {
        return false;
    };
    let mut changed = false;
    let mut at = 0;
    while let Some(found) = bytes[at..].windows(from.len()).position(|w| w == from) {
        let start = at + found;
        bytes[start..start + from.len()].copy_from_slice(to);
        at = start + from.len();
        changed = true;
    }
    std::fs::write(path, bytes).expect("rewrite");
    changed
}

#[test]
fn adv_postgres_stream_with_a_value_altered_does_not_open_altered() {
    let root = tempfile::tempdir().expect("tempdir");
    let target = postgres(root.path());
    seeded(&target);
    let Locator::Postgres(url) = &target.locator else {
        unreachable!("a PostgreSQL target")
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let altered = runtime.block_on(async {
        let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls)
            .await
            .expect("the database answers");
        tokio::spawn(connection);
        client
            .execute(
                "UPDATE thalamus_events \
                 SET data = jsonb_set(data, '{reading,value,value}', '\"0.039\"') \
                 WHERE tenant_id = $1 AND data #>> '{reading,value,value}' = '0.029'",
                &[&target.instance],
            )
            .await
            .expect("the row is updated")
    });
    assert_eq!(altered, 1, "the value is in one row");
    assert_loud_or_intact(&target, "altering 0.029 to 0.039 in the PostgreSQL row");
}

#[test]
fn adv_sqlite_stream_with_a_value_altered_does_not_open_altered() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("thalamus.sqlite");
    let locator = Target {
        locator: Locator::Sqlite(path.clone()),
        instance: INSTANCE.to_owned(),
    };
    seeded(&locator);
    let mut altered = false;
    for file in [path.clone(), root.path().join("thalamus.sqlite-wal")] {
        altered |= replace_bytes(&file, b"0.029", b"0.039");
    }
    assert!(altered, "the value is in the database file");
    assert_loud_or_intact(&locator, "altering 0.029 to 0.039 in the SQLite file");
}

#[test]
fn adv_sqlite_file_truncated_does_not_open_short() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("thalamus.sqlite");
    let locator = Target {
        locator: Locator::Sqlite(path.clone()),
        instance: INSTANCE.to_owned(),
    };
    seeded(&locator);
    let length = std::fs::metadata(&path).expect("the file").len();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|file| file.set_len(length / 2))
        .expect("truncate");
    assert_loud_or_intact(&locator, "truncating the SQLite file to half");
}

// ---------------------------------------------------------------------------------------------
// A record another writer put on the stream.
// ---------------------------------------------------------------------------------------------

/// Appends `body` to the instance stream of the SQLite store at `path`, as another writer would.
fn foreign_append(path: &Path, body: Value) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let store: Arc<dyn EventStore> = Arc::new(
            eventlog_sqlite::SqliteEventStore::open(path.to_str().expect("UTF-8"), "thalamus")
                .await
                .expect("opens"),
        );
        let stream = StreamId::new(
            TenantId::new(INSTANCE).expect("tenant"),
            "thalamus.instance",
            INSTANCE,
        )
        .expect("stream");
        let head = store
            .read_stream(&stream, 0, eventlog_core::MAX_READ_LIMIT)
            .await
            .expect("reads")
            .events
            .len() as u64;
        let meta = CommandMeta {
            idempotency_key: "foreign-1".to_owned(),
            request_hash: eventlog_core::request_hash(&body).expect("hash"),
            subject: "thalamus".to_owned(),
            actor: "another-writer".to_owned(),
            request_id: "foreign-1".to_owned(),
            trace_id: "foreign-1".to_owned(),
            causation_id: None,
            causation_depth: 0,
            occurred_at: time::OffsetDateTime::now_utc(),
            claim: None,
        };
        let expected = if head == 0 {
            Expected::NoStream
        } else {
            Expected::Exact(head)
        };
        store
            .append(
                &stream,
                expected,
                &[NewEvent::new("thalamus.sample", 1, body).expect("event")],
                &meta,
            )
            .await
            .expect("appended");
    });
}

/// A `thalamus.sample/1` record whose unknown reading carries a value is not answered as a
/// stored record holding that value: the open refuses it, or what the store hands back as the
/// record carries no value.
#[test]
fn adv_foreign_unknown_reading_with_a_value_never_comes_back_with_it() {
    let root = tempfile::tempdir().expect("tempdir");
    let dir = root.path().join("thalamus.sqlite");
    let locator = Locator::Sqlite(dir.clone());
    let mut body = thalamus_store_body(&sample_at(
        "smp_000000000001",
        "2026-10-14T10:00:00Z",
        Reading::Unknown(UnreadReading {
            reason: "timeout".to_owned(),
        }),
    ));
    body["reading"]["value"]["value"] = Value::from("0");
    foreign_append(&dir, body);
    let Ok(store) = EventlogStore::open(&locator, INSTANCE) else {
        return;
    };
    let record = store.records()[0].body();
    assert!(
        record["reading"]["value"].get("value").is_none(),
        "an unknown reading came back from the store with a value: {}",
        record["reading"]
    );
}

/// A record of a schema this build does not know refuses the open, naming the schema.
#[test]
fn adv_foreign_schema_refuses_the_open() {
    let root = tempfile::tempdir().expect("tempdir");
    let dir = root.path().join("thalamus.sqlite");
    let mut body = thalamus_store_body(&sample("smp_000000000001"));
    body["schema"] = Value::from("thalamus.sample/2");
    foreign_append(&dir, body);
    match EventlogStore::open(&Locator::Sqlite(dir), INSTANCE) {
        Err(StoreError::Record(message)) => {
            assert!(message.contains("thalamus.sample/2"), "{message}")
        }
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(_) => panic!("a thalamus.sample/2 record opened"),
    }
}

/// The record body the store itself writes for `sample`, read back from a scratch store.
fn thalamus_store_body(sample: &SignalSampleData) -> Value {
    let root = tempfile::tempdir().expect("tempdir");
    let mut store = open(&sqlite(root.path()));
    store.append_sample(sample).expect("stored");
    store.records()[0].body().clone()
}

// ---------------------------------------------------------------------------------------------
// Text and nested numbers round-trip.
// ---------------------------------------------------------------------------------------------

/// Text that needs escaping and a baseline (a nested `Reading`) come back exactly, both
/// providers, and an unknown baseline reading carries no value on the stream.
#[test]
fn adv_escaped_text_and_baselines_round_trip() {
    let root = tempfile::tempdir().expect("tempdir");
    let awkward =
        "quote \" backslash \\ newline \n tab \t bell \u{7} e\u{301} \u{2028} crab \u{1f980}";
    for locator in locators(root.path()) {
        let mut s = sample("smp_000000000001");
        s.dims
            .insert("namespace".to_owned(), DimensionValue(awkward.to_owned()));
        s.reading = Reading::Unknown(UnreadReading {
            reason: awkward.to_owned(),
        });
        let mut e = evaluation_of("evl_000000000001", &s, 1);
        e.verdict = Verdict::UNKNOWN;
        e.score = None;
        e.reason = Some(awkward.to_owned());
        e.baseline = Some(Baseline {
            kind: BaselineLabel("offset_pooled".to_owned()),
            offsets: vec![awkward.to_owned(), "7d".to_owned()],
            reading: Reading::Unknown(UnreadReading {
                reason: awkward.to_owned(),
            }),
        });
        let mut e2 = evaluation_of("evl_000000000002", &s, 2);
        e2.score = Some(Decimal("-0.000100".to_owned()));
        e2.baseline = Some(Baseline {
            kind: BaselineLabel("rolling_median".to_owned()),
            offsets: Vec::new(),
            reading: ok("12345678901234567890.000000000000000000010"),
        });
        {
            let mut store = open(&locator);
            store.append_sample(&s).expect("stored");
            store.append_evaluation(&e).expect("stored");
            store.append_evaluation(&e2).expect("stored");
        }
        let store = open(&locator);
        assert_eq!(store.sample(&s.sample_id), Some(s.clone()), "{locator}");
        assert_eq!(
            store.evaluation(&e.evaluation_id),
            Some(e.clone()),
            "{locator}"
        );
        assert_eq!(
            store.evaluation(&e2.evaluation_id),
            Some(e2.clone()),
            "{locator}"
        );
        let stored = store.records()[1].body();
        assert!(
            stored["baseline"]["reading"]["value"]
                .get("value")
                .is_none(),
            "{locator}: {}",
            stored["baseline"]
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Volume: the open reads the stream in pages.
// ---------------------------------------------------------------------------------------------

/// More records than one read page (`MAX_READ_LIMIT`, 1000) come back once each after a reopen.
#[test]
fn adv_reopen_reads_past_one_page() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        {
            let mut store = open(&locator);
            for number in 1..=1001 {
                store
                    .append_sample(&sample(&format!("smp_{number:012}")))
                    .expect("stored");
            }
        }
        let mut store = open(&locator);
        assert_eq!(store.samples().len(), 1001, "{locator}");
        assert_eq!(store.head(), 1001, "{locator}");
        assert_eq!(
            IdSequence::next_number(&mut store, "smp_"),
            Some(1002),
            "{locator}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// An async caller.
// ---------------------------------------------------------------------------------------------

/// `lib.rs`: "none may be made from inside another Tokio runtime (an async caller moves the store
/// onto a blocking thread)". An async caller that follows that, opening and using the store on a
/// blocking thread, then lets the store go where it holds it (a server's shared state dropped in
/// an async test or at shutdown), does not panic.
#[test]
fn adv_store_opened_for_an_async_caller_drops_without_panic() {
    let root = tempfile::tempdir().expect("tempdir");
    for locator in locators(root.path()) {
        let caller = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime");
        let dropped = catch_unwind(AssertUnwindSafe(|| {
            caller.block_on(async {
                let locator = locator.clone();
                let store = tokio::task::spawn_blocking(move || {
                    let mut store = open(&locator);
                    store
                        .append_sample(&sample("smp_000000000001"))
                        .expect("stored");
                    store
                })
                .await
                .expect("the blocking open");
                drop(store);
            });
        }));
        assert!(
            dropped.is_ok(),
            "{locator}: dropping the store in the async caller panicked"
        );
    }
}

/// What a call from inside a runtime does: documented, not a defect (`lib.rs`).
#[test]
fn adv_open_inside_a_runtime_panics_as_documented() {
    let root = tempfile::tempdir().expect("tempdir");
    let locator = Locator::Sqlite(root.path().join("thalamus.sqlite"));
    let caller = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime");
    let opened = catch_unwind(AssertUnwindSafe(|| {
        caller.block_on(async { EventlogStore::open(&locator, INSTANCE).map(|_| ()) })
    }));
    assert!(
        opened.is_err(),
        "open inside a runtime did not panic: {opened:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The whole suite over the Eventlog store.
// ---------------------------------------------------------------------------------------------

struct EventlogPorts {
    store: EventlogStore,
    memory: MemoryPorts,
    _dir: TempDir,
}

impl EventlogPorts {
    fn new(locator: fn(&Path) -> Locator) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let store =
            EventlogStore::open(&locator(dir.path()), &instance_of(dir.path())).expect("opens");
        Self {
            store,
            memory: MemoryPorts::default(),
            _dir: dir,
        }
    }
}

impl IdSequence for EventlogPorts {
    fn next_number(&mut self, prefix: &'static str) -> Option<u64> {
        self.store.next_number(prefix)
    }
}

macro_rules! delegate {
    ($port:ident, $to:ident, $identity:ty, $snapshot:ty) => {
        impl $port for EventlogPorts {
            fn get(&self, identity: &$identity) -> Option<$snapshot> {
                $port::get(&self.$to, identity)
            }
            fn put(&mut self, snapshot: $snapshot) {
                $port::put(&mut self.$to, snapshot);
            }
            fn delete(&mut self, identity: &$identity) {
                $port::delete(&mut self.$to, identity);
            }
            fn list(&self) -> Vec<$snapshot> {
                $port::list(&self.$to)
            }
        }
    };
}

delegate!(
    SignalSampleStorage,
    store,
    observation::SampleId,
    observation::SignalSampleSnapshot
);
delegate!(
    SignalEvaluationStorage,
    store,
    evaluation::EvaluationId,
    evaluation::SignalEvaluationSnapshot
);
delegate!(
    SignalDefinitionStorage,
    memory,
    definition::SignalId,
    definition::SignalDefinitionSnapshot
);
delegate!(
    DimensionStorage,
    memory,
    dimension::DimensionKey,
    dimension::DimensionSnapshot
);
delegate!(
    DimensionRegistryStorage,
    memory,
    dimension::RegistryId,
    dimension::DimensionRegistrySnapshot
);
delegate!(
    AssertionStorage,
    memory,
    assertion::AssertionId,
    assertion::AssertionSnapshot
);

fn whole_suite(locator: fn(&Path) -> Locator) -> Report {
    let suite = std::fs::read_to_string(manifest("../../spec/suite.json")).expect("suite");
    let report = conformance::run_sequenced(&suite, &|_| true, || EventlogPorts::new(locator))
        .expect("the suite runs");
    let failures: Vec<String> = report
        .failed
        .iter()
        .map(|failure| format!("{}: {}", failure.scenario, failure.reason))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(report.ran.len(), 73, "every scenario of the suite ran");
    report
}

/// Every scenario of `spec/suite.json`, not only the four the story names, holds with samples and
/// evaluations on Eventlog: the store is a drop-in for the in-memory one.
#[test]
fn adv_whole_suite_holds_on_the_postgres_provider() {
    whole_suite(|_| postgres_locator());
}

#[test]
fn adv_whole_suite_holds_on_the_sqlite_provider() {
    whole_suite(|dir| Locator::Sqlite(dir.join("thalamus.sqlite")));
}
