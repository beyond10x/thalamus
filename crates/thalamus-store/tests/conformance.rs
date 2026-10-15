//! The named scenarios of `spec/suite.json`, run by `thalamus_core::conformance::run_sequenced`
//! with the Eventlog store in place of the in-memory one for samples and evaluations, on both
//! SQLite and the PostgreSQL provider. Definitions, dimensions and assertions stay in memory; every id
//! is minted from the store's own sequence.
//!
//! [`SCENARIOS`] is the one selection: a later story that puts another port on Eventlog adds its
//! scenarios there, one line each.

use std::path::Path;

use tempfile::TempDir;
use thalamus_core::behaviour::observation::IdSequence;
use thalamus_core::conformance::{self, Ports, Report};
use thalamus_model::behaviour::{
    AssertionStorage, DimensionRegistryStorage, DimensionStorage, SignalDefinitionStorage,
    SignalEvaluationStorage, SignalSampleStorage,
};
use thalamus_model::server::memory::MemoryPorts;
use thalamus_model::{assertion, definition, dimension, evaluation, observation};
use thalamus_store::eventlog::{EventlogStore, Locator};

/// The scenarios run against the Eventlog storage.
const SCENARIOS: &[&str] = &[
    "thalamus.observation.RecordSample/outcome/recorded",
    "thalamus.evaluation.RecordEvaluation/outcome/recorded",
    "thalamus.evaluation.RecordEvaluation/outcome/unknown-sample",
    "thalamus.evaluation.SignalEvaluation/invariant/after/thalamus.evaluation.RecordEvaluation/recorded",
];

/// Samples and evaluations in a fresh Eventlog store, everything else in memory. The directory
/// is dropped after the store.
struct EventlogPorts {
    store: EventlogStore,
    memory: MemoryPorts,
    _dir: TempDir,
}

impl EventlogPorts {
    fn new(locator: fn(&Path) -> Locator) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let instance = format!(
            "scenario-{}{}",
            std::process::id(),
            dir.path()
                .file_name()
                .and_then(|name| name.to_str())
                .expect("a tempdir name")
        );
        let store =
            EventlogStore::open(&locator(dir.path()), &instance).expect("the Eventlog store opens");
        Self {
            store,
            memory: MemoryPorts::default(),
            _dir: dir,
        }
    }
}

/// Every id comes from the Eventlog store's sequence, which it resumes on open.
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

/// The PostgreSQL database `THALAMUS_TEST_POSTGRES_URL` names; each scenario writes its own
/// instance stream in it.
fn postgres() -> Locator {
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

fn suite() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/suite.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn run(locator: fn(&Path) -> Locator) -> Report {
    let report = conformance::run_sequenced(&suite(), &|id| SCENARIOS.contains(&id), || {
        EventlogPorts::new(locator)
    })
    .expect("the suite runs");
    let failures: Vec<String> = report
        .failed
        .iter()
        .map(|failure| format!("{}: {}", failure.scenario, failure.reason))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let mut ran: Vec<&str> = report.ran.iter().map(String::as_str).collect();
    let mut wanted = SCENARIOS.to_vec();
    ran.sort_unstable();
    wanted.sort_unstable();
    assert_eq!(
        ran, wanted,
        "the run answered exactly the selected scenarios"
    );
    report
}

#[test]
fn named_scenarios_hold_on_the_postgres_provider() {
    let report = run(|_| postgres());
    println!("conformance (postgres): {} ran", report.ran.len());
}

#[test]
fn named_scenarios_hold_on_the_sqlite_provider() {
    let report = run(|dir| Locator::Sqlite(dir.join("thalamus.sqlite")));
    println!("conformance (sqlite): {} ran", report.ran.len());
}

/// The harness ports a sequenced run builds draw every id from the store, not from the
/// process-wide sequence the in-memory runs share: after the store has issued 41 sample numbers,
/// the ports answer 42.
#[test]
fn harness_ports_draw_ids_from_the_store() {
    let mut storage = EventlogPorts::new(|dir| Locator::Sqlite(dir.join("thalamus.sqlite")));
    for _ in 0..41 {
        storage.store.next_number("smp_").expect("a number");
    }
    let mut ports = Ports::sequenced(storage, None);
    assert_eq!(IdSequence::next_number(&mut ports, "smp_"), Some(42));
    assert_eq!(
        IdSequence::next_number(&mut ports.storage, "smp_"),
        Some(43)
    );
}
