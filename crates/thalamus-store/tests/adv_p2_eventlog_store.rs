//! Adversary cases for the Eventlog store (wave 20261015d, pass 2): two handles on one stream,
//! interleaved and in concurrent bursts, reads racing appends, the PostgreSQL locator, and the
//! design § 2.2 / § 2.3 blocks read as stored records.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use eventlog_core::{CommandMeta, EventStore, Expected, NewEvent, StreamId, TenantId};
use serde_json::Value;
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_model::behaviour::SignalSampleStorage;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::evaluation::EvaluationId;
use thalamus_model::observation::{
    CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, ID_TAKEN, Locator, POSTGRES_PASSWORD, StoreError};

const SIGNAL: &str = "platform.namespace.request_failure_ratio";
const SERIES: &str = r#"platform.namespace.request_failure_ratio{namespace="shared-1"}"#;

fn postgres_url() -> String {
    std::env::var("THALAMUS_TEST_POSTGRES_URL").unwrap_or_else(|_| {
        panic!(
            "THALAMUS_TEST_POSTGRES_URL is not set: the PostgreSQL cases need a disposable local \
             database (`task test-postgres` starts one)"
        )
    })
}

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

fn instance_of(root: &Path) -> String {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a tempdir name");
    format!("advp2-{}{name}", std::process::id())
}

fn targets(root: &Path) -> [Target; 2] {
    [
        Target {
            locator: Locator::Sqlite(root.join("thalamus.sqlite")),
            instance: instance_of(root),
        },
        Target {
            locator: postgres_url()
                .parse()
                .expect("a PostgreSQL URL without a password"),
            instance: instance_of(root),
        },
    ]
}

fn open(target: &Target) -> EventlogStore {
    EventlogStore::open(&target.locator, &target.instance)
        .unwrap_or_else(|error| panic!("{target}: {error}"))
}

fn sample_at(id: &str, ts: &str) -> SignalSampleData {
    SignalSampleData {
        text: None,
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
        reading: Reading::Ok(CompleteReading {
            value: Decimal("0.029".to_owned()),
        }),
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

/// A distinct timestamp per (writer, step), so two writers never build one body.
fn ts_of(writer: u32, step: u32) -> String {
    format!(
        "2026-10-14T{:02}:{:02}:{:02}Z",
        writer,
        step / 60,
        step % 60
    )
}

fn smp(number: u64) -> String {
    format!("smp_{number:012}")
}

// ---------------------------------------------------------------------------------------------
// Two handles, interleaved.
// ---------------------------------------------------------------------------------------------

/// `append_sample` documents `StoreError::Refused` with `ID_TAKEN` "when its id is held with
/// another body". A handle that has not yet seen the other handle's sample gets that refusal too,
/// not a raw Eventlog error.
#[test]
fn adv_p2_stale_handle_storing_a_taken_id_is_refused_id_taken() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut wrong = Vec::new();
    for target in targets(root.path()) {
        let mut first = open(&target);
        let mut second = open(&target);
        first
            .append_sample(&sample_at(&smp(1), &ts_of(1, 0)))
            .unwrap_or_else(|error| panic!("{target}: {error}"));
        match second.append_sample(&sample_at(&smp(1), &ts_of(2, 0))) {
            Err(StoreError::Refused { code }) if code == ID_TAKEN => {}
            other => wrong.push(format!("{target}: {other:?}")),
        }
    }
    assert!(
        wrong.is_empty(),
        "a stale handle storing another body under a stored id was not refused {ID_TAKEN}:\n{}",
        wrong.join("\n")
    );
}

/// The store's `IdSequence` says neither a reopened store "nor a handle beside another writer
/// issues an identity again". Two handles that each mint a sample id and record a sample under
/// it, in the order two writers interleave (mint, mint, record, record), both keep their sample.
#[test]
fn adv_p2_two_handles_minting_then_recording_keep_both_samples() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut wrong = Vec::new();
    for target in targets(root.path()) {
        let mut first = open(&target);
        let mut second = open(&target);
        let a = first.next_number("smp_").expect("a number");
        let b = second.next_number("smp_").expect("a number");
        let stored_a = first.append_sample(&sample_at(&smp(a), &ts_of(1, 0)));
        let stored_b = second.append_sample(&sample_at(&smp(b), &ts_of(2, 0)));
        let kept = open(&target).samples().len();
        if a == b || stored_a.is_err() || stored_b.is_err() || kept != 2 {
            wrong.push(format!(
                "{target}: minted {a} and {b}; first {stored_a:?}; second {stored_b:?}; \
                 {kept} sample(s) on the stream"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// One sample stored from two handles, the second stale, is one record: the retry writes
/// nothing, and the stale handle records on afterwards.
#[test]
fn adv_p2_same_sample_from_two_handles_is_stored_once() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut first = open(&target);
        let mut second = open(&target);
        let one = sample_at(&smp(1), &ts_of(1, 0));
        first.append_sample(&one).expect("first stores");
        second
            .append_sample(&one)
            .unwrap_or_else(|error| panic!("{target}: the same sample again: {error}"));
        second
            .append_sample(&sample_at(&smp(2), &ts_of(1, 1)))
            .unwrap_or_else(|error| panic!("{target}: the next sample: {error}"));
        let fresh = open(&target);
        let ids: Vec<&str> = fresh
            .samples()
            .iter()
            .map(|s| s.sample_id.0.as_str())
            .collect();
        assert_eq!(ids, [smp(1), smp(2)], "{target}");
        assert_eq!(fresh.head(), 2, "{target}");
    }
}

/// D5 across two handles: the stale handle's evaluation of a sample the other handle already
/// graded at that revision is refused `DUPLICATE_EVALUATION` after its catch-up, and one
/// evaluation is stored.
#[test]
fn adv_p2_stale_handle_second_evaluation_at_one_rev_is_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut first = open(&target);
        let one = sample_at(&smp(1), &ts_of(1, 0));
        first.append_sample(&one).expect("the sample");
        let mut second = open(&target);
        first
            .append_sample(&sample_at(&smp(2), &ts_of(1, 1)))
            .expect("moves the head");
        let graded = |id: &str| thalamus_model::evaluation::SignalEvaluationData {
            evaluation_id: EvaluationId(id.to_owned()),
            sample_id: one.sample_id.clone(),
            signal: one.signal.clone(),
            series: one.series.clone(),
            dims: one.dims.clone(),
            ts: one.ts.clone(),
            definition_rev: 1,
            verdict: thalamus_model::evaluation::Verdict::OK,
            prev: None,
            score: None,
            baseline: None,
            reason: None,
        };
        first
            .append_evaluation(&graded("evl_000000000001"))
            .expect("the first grade");
        match second.append_evaluation(&graded("evl_000000000002")) {
            Err(StoreError::Refused { code })
                if code == thalamus_store::evaluations::DUPLICATE_EVALUATION => {}
            other => panic!("{target}: {other:?}"),
        }
        assert_eq!(open(&target).evaluations().len(), 1, "{target}");
    }
}

fn graded(id: &str, of: &SignalSampleData) -> thalamus_model::evaluation::SignalEvaluationData {
    thalamus_model::evaluation::SignalEvaluationData {
        evaluation_id: EvaluationId(id.to_owned()),
        sample_id: of.sample_id.clone(),
        signal: of.signal.clone(),
        series: of.series.clone(),
        dims: of.dims.clone(),
        ts: of.ts.clone(),
        definition_rev: 1,
        verdict: thalamus_model::evaluation::Verdict::OK,
        prev: None,
        score: None,
        baseline: None,
        reason: None,
    }
}

/// eventlog.rs: "A read that answers from what the stream holds (`sample`, `evaluation`, the
/// ranges, the generated ports) also reads the records appended since the handle last moved",
/// and "an evaluation of a sample the handle does not hold reads the stream first". A handle
/// opened on an empty stream answers, through every such read, what another handle stored
/// after it, and records an evaluation of that handle's sample.
#[test]
fn adv_p2_stale_handle_reads_and_grades_what_another_handle_stored() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut writer = open(&target);
        let stale = open(&target);
        let one = sample_at(&smp(1), &ts_of(1, 0));
        writer.append_sample(&one).expect("the sample");
        writer
            .append_evaluation(&graded("evl_000000000001", &one))
            .expect("its grade");
        let evaluation = EvaluationId("evl_000000000001".to_owned());
        let mut missing = Vec::new();
        if stale.first_sample(&one.series).is_none() {
            missing.push("first_sample");
        }
        if stale.evaluation(&evaluation).is_none() {
            missing.push("evaluation");
        }
        if thalamus_model::behaviour::SignalEvaluationStorage::get(&stale, &evaluation).is_none() {
            missing.push("SignalEvaluationStorage::get");
        }
        if thalamus_model::behaviour::SignalEvaluationStorage::list(&stale).len() != 1 {
            missing.push("SignalEvaluationStorage::list");
        }
        if stale.newest_evaluations().len() != 1 {
            missing.push("newest_evaluations");
        }
        if !stale.unevaluated_samples(&one.signal, 1).is_empty() {
            missing.push("unevaluated_samples (answers a graded sample as ungraded)");
        }
        let range = stale.series_range(
            &one.series,
            &Timestamp("2026-10-14T00:00:00Z".to_owned()),
            &Timestamp("2026-10-15T00:00:00Z".to_owned()),
        );
        if range.samples.len() != 1 || range.evaluations.len() != 1 {
            missing.push("series_range");
        }
        let two = sample_at(&smp(2), &ts_of(1, 1));
        writer.append_sample(&two).expect("the second sample");
        let mut stale_grader = open(&target);
        writer
            .append_sample(&sample_at(&smp(3), &ts_of(1, 2)))
            .expect("moves the head past the grader");
        if let Err(error) = stale_grader.append_evaluation(&graded("evl_000000000002", &two)) {
            missing.push(if matches!(&error, StoreError::Refused { code } if code == thalamus_store::evaluations::UNKNOWN_SAMPLE) {
                "append_evaluation refused unknown-sample"
            } else {
                "append_evaluation failed"
            });
        }
        let mut late = open(&target);
        drop(writer);
        let three = sample_at(&smp(4), &ts_of(1, 3));
        let mut other = open(&target);
        other.append_sample(&three).expect("the fourth sample");
        let answer = late.append_evaluation(&graded("evl_000000000003", &three));
        if matches!(&answer, Err(StoreError::Refused { code }) if code == thalamus_store::evaluations::UNKNOWN_SAMPLE)
        {
            missing
                .push("append_evaluation from a handle one record behind refused unknown-sample");
        }
        assert!(missing.is_empty(), "{target}: {}", missing.join(", "));
    }
}

/// The `IdSequence` doc: the sequence is "read again before every number". A handle opened
/// before another handle stored three samples mints the number after them.
#[test]
fn adv_p2_stale_handle_mints_past_what_the_stream_holds() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut writer = open(&target);
        let mut stale = open(&target);
        for step in 0..3 {
            let number = writer.next_number("smp_").expect("a number");
            writer
                .append_sample(&sample_at(&smp(number), &ts_of(1, step)))
                .expect("stored");
        }
        assert_eq!(stale.next_number("smp_"), Some(4), "{target}");
    }
}

/// A retry that Eventlog answers as already stored (the other handle stored the same body) is
/// taken in: after the call that took `&mut self`, the handle's slices and head hold it.
#[test]
fn adv_p2_retry_of_another_handles_sample_is_taken_in() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut first = open(&target);
        let mut second = open(&target);
        let one = sample_at(&smp(1), &ts_of(1, 0));
        first.append_sample(&one).expect("first stores");
        second.append_sample(&one).expect("the retry");
        assert_eq!(second.head(), 1, "{target}: head");
        assert_eq!(
            second.samples(),
            std::slice::from_ref(&one),
            "{target}: samples()"
        );
        assert_eq!(second.records().len(), 1, "{target}: records()");
    }
}

// ---------------------------------------------------------------------------------------------
// Two threads, a burst.
// ---------------------------------------------------------------------------------------------

const BURST: u32 = 40;

/// Two writers, each its own handle on one stream and its own thread, record `BURST` samples
/// each as the pipeline does (mint an id from the store, then store the sample). Every sample is
/// stored, under its own id, and no call fails.
fn burst(target: &Target, mint: bool) -> Vec<String> {
    let start = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for writer in 1..=2u32 {
        let mut store = open(target);
        let start = Arc::clone(&start);
        handles.push(std::thread::spawn(move || {
            let mut failures = Vec::new();
            start.wait();
            for step in 0..BURST {
                let number = if mint {
                    store.next_number("smp_").expect("a number")
                } else {
                    u64::from(writer) * 1_000_000 + u64::from(step) + 1
                };
                if let Err(error) =
                    store.append_sample(&sample_at(&smp(number), &ts_of(writer, step)))
                {
                    failures.push(format!(
                        "writer {writer} step {step} {}: {error:?}",
                        smp(number)
                    ));
                }
            }
            failures
        }));
    }
    let mut failures: Vec<String> = handles
        .into_iter()
        .flat_map(|handle| handle.join().expect("the writer thread did not panic"))
        .collect();
    let fresh = open(target);
    let ids: BTreeSet<&str> = fresh
        .samples()
        .iter()
        .map(|s| s.sample_id.0.as_str())
        .collect();
    let stamps: BTreeSet<&str> = fresh.samples().iter().map(|s| s.ts.0.as_str()).collect();
    let want = 2 * BURST as usize;
    if fresh.samples().len() != want || ids.len() != want || stamps.len() != want {
        failures.push(format!(
            "{} records, {} distinct ids, {} distinct samples; {want} were recorded",
            fresh.samples().len(),
            ids.len(),
            stamps.len()
        ));
    }
    failures
        .into_iter()
        .map(|failure| format!("{target}: {failure}"))
        .collect()
}

#[test]
fn adv_p2_two_threads_minting_and_recording_lose_no_sample() {
    let root = tempfile::tempdir().expect("tempdir");
    let failures: Vec<String> = targets(root.path())
        .iter()
        .flat_map(|target| burst(target, true))
        .collect();
    assert!(
        failures.is_empty(),
        "{} failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A writer reserving an `smp_` id while another handle holds the SQLite write lock, for longer
/// than the driver's own 5 s busy handler waits but less than the store's lock budget, waits the
/// lock out: it gets its id and stores its sample, and neither call fails or panics.
#[test]
fn adv_p2_reserving_an_id_under_a_held_write_lock_waits_and_stores() {
    const HOLD: Duration = Duration::from_secs(7);
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("thalamus.sqlite");
    let target = Target {
        locator: Locator::Sqlite(path.clone()),
        instance: instance_of(root.path()),
    };
    let mut reserver = open(&target);
    let first = reserver.next_number("smp_").expect("a first number");
    reserver
        .append_sample(&sample_at(&smp(first), &ts_of(1, 0)))
        .expect("the first sample is stored");
    let holder = rusqlite::Connection::open(&path).expect("the holder opens the file");
    holder
        .execute_batch("BEGIN IMMEDIATE")
        .expect("the holder takes the write lock");
    let release = std::thread::spawn(move || {
        std::thread::sleep(HOLD);
        holder.execute_batch("COMMIT").expect("the holder commits");
    });
    let started = Instant::now();
    let number = reserver
        .next_number("smp_")
        .expect("a number after the lock is released");
    let stored = reserver.append_sample(&sample_at(&smp(number), &ts_of(1, 1)));
    release.join().expect("the holder did not panic");
    assert!(
        started.elapsed() >= HOLD / 2,
        "the reservation did not meet the held lock"
    );
    assert_eq!(number, first + 1);
    stored.expect("the sample is stored under the reserved id");
    let fresh = open(&target);
    assert!(fresh.sample(&SampleId(smp(number))).is_some());
}

#[test]
fn adv_p2_two_threads_recording_disjoint_ids_lose_no_sample() {
    let root = tempfile::tempdir().expect("tempdir");
    let failures: Vec<String> = targets(root.path())
        .iter()
        .flat_map(|target| burst(target, false))
        .collect();
    assert!(
        failures.is_empty(),
        "{} failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------------------------
// Reads racing appends.
// ---------------------------------------------------------------------------------------------

/// A reader handle opened before a writer starts answers every sample the writer has been told
/// is stored, through `sample`, the generated `get` and `list`, and `list` never holds one id
/// twice, while the writer keeps appending.
#[test]
fn adv_p2_reads_racing_appends_answer_every_acknowledged_sample() {
    let root = tempfile::tempdir().expect("tempdir");
    for target in targets(root.path()) {
        let mut writer = open(&target);
        let reader = open(&target);
        let (sent, received) = mpsc::channel::<String>();
        let write = std::thread::spawn(move || {
            for step in 0..BURST {
                let id = smp(u64::from(step) + 1);
                writer
                    .append_sample(&sample_at(&id, &ts_of(1, step)))
                    .expect("the writer stores");
                sent.send(id).expect("the reader listens");
            }
        });
        let mut acknowledged = Vec::new();
        for id in received {
            acknowledged.push(id.clone());
            let wanted = SampleId(id.clone());
            assert!(reader.sample(&wanted).is_some(), "{target}: sample({id})");
            assert!(
                SignalSampleStorage::get(&reader, &wanted).is_some(),
                "{target}: get({id})"
            );
            let listed: Vec<String> = SignalSampleStorage::list(&reader)
                .into_iter()
                .map(|snapshot| snapshot.data.sample_id.0)
                .collect();
            let distinct: BTreeSet<&String> = listed.iter().collect();
            assert_eq!(distinct.len(), listed.len(), "{target}: list repeats an id");
            for held in &acknowledged {
                assert!(listed.contains(held), "{target}: list misses {held}");
            }
        }
        write.join().expect("the writer did not panic");
        assert_eq!(acknowledged.len(), BURST as usize, "{target}");
    }
}

// ---------------------------------------------------------------------------------------------
// The PostgreSQL locator.
// ---------------------------------------------------------------------------------------------

/// Eight processes' worth of handles opening one PostgreSQL database at once (a shared instance
/// whose writers start together, or the store suite's parallel test threads) all open.
#[test]
fn adv_p2_postgres_concurrent_opens_all_succeed() {
    let root = tempfile::tempdir().expect("tempdir");
    let start = Arc::new(Barrier::new(8));
    let opens: Vec<_> = (0..8)
        .map(|n| {
            let start = Arc::clone(&start);
            let instance = format!("{}-{n}", instance_of(root.path()));
            std::thread::spawn(move || {
                let locator: Locator = postgres_url().parse().expect("a locator");
                start.wait();
                let started = Instant::now();
                EventlogStore::open(&locator, &instance)
                    .map(|_| ())
                    .map_err(|error| format!("{instance} after {:?}: {error}", started.elapsed()))
            })
        })
        .collect();
    let failed: Vec<String> = opens
        .into_iter()
        .filter_map(|open| open.join().expect("no panic").err())
        .collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// A password in the query string is a password: refused, and the refusal does not repeat it.
#[test]
fn adv_p2_postgres_password_in_the_query_is_refused_unrepeated() {
    for url in [
        "postgresql://postgres@127.0.0.1:5432/postgres?password=hunter2-secret",
        "postgres://postgres:hunter2-secret@127.0.0.1:5432/postgres",
        "postgresql://postgres:@127.0.0.1:5432/postgres?password=hunter2-secret",
    ] {
        match url.parse::<Locator>() {
            Err(error @ StoreError::Locator { code, .. }) => {
                assert_eq!(code, POSTGRES_PASSWORD, "{url}");
                assert!(!error.to_string().contains("hunter2"), "{url}: {error}");
            }
            other => panic!("{url}: {other:?}"),
        }
    }
}

/// An operator who writes `sslmode=require` asked for TLS. The disposable server has no TLS, so
/// an open that honoured the request fails; an open that succeeds went plaintext silently.
#[test]
fn adv_p2_postgres_sslmode_require_is_not_answered_with_plaintext() {
    let root = tempfile::tempdir().expect("tempdir");
    let url = format!("{}?sslmode=require", postgres_url());
    let locator: Locator = match url.parse() {
        Ok(locator) => locator,
        Err(_) => return,
    };
    let opened = EventlogStore::open(&locator, &instance_of(root.path()));
    assert!(
        opened.is_err(),
        "{url}: opened over plaintext although the locator requires TLS"
    );
}

/// An unreachable server, a database that does not exist and a host that is not local each fail
/// the open with an error, within five seconds.
#[test]
fn adv_p2_postgres_unusable_server_fails_loudly_and_soon() {
    let root = tempfile::tempdir().expect("tempdir");
    let base = postgres_url();
    let (prefix, _database) = base.rsplit_once('/').expect("a database path");
    for url in [
        "postgresql://postgres@127.0.0.1:1/postgres".to_owned(),
        format!("{prefix}/advp2_no_such_database"),
        "postgresql://postgres@192.0.2.1:5432/postgres".to_owned(),
        "postgresql://postgres@%2Fnonexistent-socket-dir/postgres".to_owned(),
    ] {
        let started = Instant::now();
        let opened = url
            .parse::<Locator>()
            .and_then(|locator| EventlogStore::open(&locator, &instance_of(root.path())));
        let took = started.elapsed();
        if let Err(error) = &opened {
            println!("{url}: {took:?}: {error}");
        }
        assert!(opened.is_err(), "{url}: opened");
        assert!(
            took < Duration::from_secs(5),
            "{url}: failed after {took:?}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The design blocks, read as stored records.
// ---------------------------------------------------------------------------------------------

fn manifest(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// The first json block after `heading` in the design document, with the elided digest filled.
fn design_record(heading: &str) -> Value {
    let path = manifest("../../docs/design/thalamus.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let after = &text[text.find(heading).expect("the heading")..];
    let start = after.find("```json\n").expect("a json block") + "```json\n".len();
    let block = &after[start..];
    let block = &block[..block.find("```").expect("the block ends")];
    let mut record: Value =
        serde_json::from_str(block).unwrap_or_else(|e| panic!("{heading}: {e}\n{block}"));
    if let Some(digest) = record.pointer_mut("/provenance/query_sha256") {
        *digest = Value::from("0".repeat(64));
    }
    record
}

/// Appends `body` under `name` to the instance stream of the SQLite store at `path`, as another
/// writer of the same stream would.
fn foreign_append(path: &Path, instance: &str, name: &str, key: &str, body: Value) {
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
            TenantId::new(instance).expect("tenant"),
            "thalamus.instance",
            instance,
        )
        .expect("stream");
        let head = store
            .read_stream(&stream, 0, eventlog_core::MAX_READ_LIMIT)
            .await
            .expect("reads")
            .events
            .len() as u64;
        let meta = CommandMeta {
            idempotency_key: key.to_owned(),
            request_hash: eventlog_core::request_hash(&body).expect("hash"),
            subject: "thalamus".to_owned(),
            actor: "another-writer".to_owned(),
            request_id: key.to_owned(),
            trace_id: key.to_owned(),
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
                &[NewEvent::new(name, 1, body).expect("event")],
                &meta,
            )
            .await
            .expect("appended");
    });
}

/// The § 2.2 and § 2.3 blocks, put on a stream as they are written, are records the store opens:
/// each decodes and encodes back to exactly the block (the open's round-trip check), and the
/// store answers the values the blocks show.
#[test]
fn adv_p2_design_blocks_are_records_the_store_opens() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("thalamus.sqlite");
    let instance = "advp2-design";
    let sample = design_record("### 2.2 `SignalSample`");
    let evaluation = design_record("### 2.3 `SignalEvaluation`");
    foreign_append(&path, instance, "thalamus.sample", "doc-sample", sample);
    foreign_append(
        &path,
        instance,
        "thalamus.evaluation",
        "doc-evaluation",
        evaluation,
    );
    let store = EventlogStore::open(&Locator::Sqlite(path), instance)
        .unwrap_or_else(|error| panic!("the design blocks do not open as records: {error}"));
    let held = store
        .sample(&SampleId("smp_000000000042".to_owned()))
        .expect("the § 2.2 sample");
    assert_eq!(
        held.reading,
        Reading::Ok(CompleteReading {
            value: Decimal("0.029".to_owned())
        })
    );
    let graded = store
        .evaluation(&EvaluationId("evl_000000000042".to_owned()))
        .expect("the § 2.3 evaluation");
    assert_eq!(graded.score, Some(Decimal("1.158".to_owned())));
    assert_eq!(graded.sample_id, held.sample_id);
}
