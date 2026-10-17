//! Adversary cases for story:store-sqlite-id-reserve-locked (wave 20261017a, pass 1): an append,
//! and an id reservation, that meet a write lock another SQLite connection holds.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use thalamus_core::behaviour::observation::IdSequence;
use thalamus_model::definition::{AdapterId, ConnectionId, SignalId, Unit};
use thalamus_model::dimension::DimensionValue;
use thalamus_model::observation::{
    CompleteReading, Denominator, Provenance, QueryDigest, Reading, RunId, SampleId, SeriesKey,
    SignalSampleData, TimeWindow,
};
use thalamus_model::primitives::{Decimal, Timestamp};
use thalamus_store::eventlog::{EventlogStore, Locator};

const SIGNAL: &str = "platform.namespace.request_failure_ratio";
const SERIES: &str = r#"platform.namespace.request_failure_ratio{namespace="shared-1"}"#;

fn instance_of(root: &Path) -> String {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a tempdir name");
    format!("advp1lock-{}{name}", std::process::id())
}

fn smp(number: u64) -> String {
    format!("smp_{number:012}")
}

fn sample_at(id: &str, second: u32) -> SignalSampleData {
    let ts = format!("2026-10-17T04:00:{second:02}Z");
    SignalSampleData {
        text: None,
        sample_id: SampleId(id.to_owned()),
        signal: SignalId(SIGNAL.to_owned()),
        series: SeriesKey(SERIES.to_owned()),
        dims: BTreeMap::from([(
            "namespace".to_owned(),
            DimensionValue("shared-1".to_owned()),
        )]),
        ts: Timestamp(ts.clone()),
        window: TimeWindow {
            start: Timestamp("2026-10-17T03:00:00Z".to_owned()),
            end: Timestamp(ts),
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

struct Fixture {
    _root: tempfile::TempDir,
    path: std::path::PathBuf,
    locator: Locator,
    instance: String,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("thalamus.sqlite");
    let instance = instance_of(root.path());
    Fixture {
        locator: Locator::Sqlite(path.clone()),
        path,
        instance,
        _root: root,
    }
}

impl Fixture {
    fn open(&self) -> EventlogStore {
        EventlogStore::open(&self.locator, &self.instance).expect("the store opens")
    }

    /// A third connection that takes the write lock now and commits after `hold`.
    fn hold_write_lock(&self, hold: Duration) -> std::thread::JoinHandle<()> {
        let holder = rusqlite::Connection::open(&self.path).expect("the holder opens the file");
        holder
            .execute_batch("BEGIN IMMEDIATE")
            .expect("the holder takes the write lock");
        std::thread::spawn(move || {
            std::thread::sleep(hold);
            holder.execute_batch("COMMIT").expect("the holder commits");
        })
    }

    fn stored_count(&self, id: &str) -> usize {
        self.open()
            .current_samples()
            .iter()
            .filter(|sample| sample.sample_id.0 == id)
            .count()
    }
}

/// The append half of the fix (`append_caught_up`'s locked arm): no case of the unit's reaches
/// it, since its sample is appended only after the reservation has outwaited the lock. A writer
/// whose id is already reserved, appending while another connection holds the write lock past
/// the driver's 5 s busy handler, stores its sample once.
#[test]
fn adv_p1_appending_under_a_held_write_lock_waits_and_stores_once() {
    const HOLD: Duration = Duration::from_secs(7);
    let fx = fixture();
    let mut writer = fx.open();
    let number = writer.next_number("smp_").expect("a number");
    let release = fx.hold_write_lock(HOLD);
    let started = Instant::now();
    let stored = writer.append_sample(&sample_at(&smp(number), 1));
    let waited = started.elapsed();
    release.join().expect("the holder did not panic");
    stored.expect("the sample is stored after the lock is released");
    assert!(
        waited >= HOLD / 2,
        "the append did not meet the held lock (waited {waited:?})"
    );
    assert_eq!(fx.stored_count(&smp(number)), 1);
}

/// Two writers reserving and appending while a third connection holds the write lock past the
/// busy handler: each gets its own number, the two numbers are the next two, and each sample is
/// stored exactly once.
#[test]
fn adv_p1_two_writers_under_a_held_write_lock_get_distinct_ids_and_store_once() {
    const HOLD: Duration = Duration::from_secs(7);
    let fx = Arc::new(fixture());
    let mut first = fx.open();
    let seed = first.next_number("smp_").expect("a seed number");
    first
        .append_sample(&sample_at(&smp(seed), 0))
        .expect("the seed sample is stored");
    drop(first);
    let release = fx.hold_write_lock(HOLD);
    let barrier = Arc::new(Barrier::new(2));
    let writers: Vec<_> = (0..2u32)
        .map(|writer| {
            let fx = Arc::clone(&fx);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let mut store = fx.open();
                barrier.wait();
                let number = store.next_number("smp_").expect("a number");
                store
                    .append_sample(&sample_at(&smp(number), 10 + writer))
                    .map(|()| number)
            })
        })
        .collect();
    let numbers: Vec<u64> = writers
        .into_iter()
        .map(|writer| {
            writer
                .join()
                .expect("the writer did not panic")
                .expect("the writer stored its sample")
        })
        .collect();
    release.join().expect("the holder did not panic");
    let mut sorted = numbers.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, vec![seed + 1, seed + 2], "numbers {numbers:?}");
    for number in sorted {
        assert_eq!(fx.stored_count(&smp(number)), 1, "{}", smp(number));
    }
    assert_eq!(fx.open().current_samples().len(), 3);
}

/// A lock held past the store's 30 s budget is answered as the lock error within the budget
/// plus one busy-handler wait, not swallowed until the holder lets go; nothing is written, and
/// the same append once the lock is free stores the sample exactly once.
#[test]
fn adv_p1_a_lock_held_past_the_budget_is_answered_not_waited_out_forever() {
    const HOLD: Duration = Duration::from_secs(45);
    let fx = fixture();
    let mut writer = fx.open();
    let number = writer.next_number("smp_").expect("a number");
    let sample = sample_at(&smp(number), 2);
    let release = fx.hold_write_lock(HOLD);
    let started = Instant::now();
    let answer = writer.append_sample(&sample);
    let waited = started.elapsed();
    let error = answer.expect_err("an append under a lock held past the budget fails");
    assert!(
        error.to_string().contains("database is locked"),
        "the failure names the lock: {error}"
    );
    assert!(
        waited >= Duration::from_secs(30) && waited < Duration::from_secs(40),
        "the append gave up after {waited:?}, not after the 30 s budget"
    );
    release.join().expect("the holder did not panic");
    assert_eq!(fx.stored_count(&smp(number)), 0, "the refused append wrote");
    writer
        .append_sample(&sample)
        .expect("the same append once the lock is free");
    assert_eq!(fx.stored_count(&smp(number)), 1);
}
