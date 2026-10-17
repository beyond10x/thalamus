//! `story:m3-one-vocabulary-across-sources`: one read → sample → grade → evaluate pass over a
//! fixture instance holding one definition per source kind (Prometheus, Loki, SQL, Kubernetes,
//! Git, HTTP probe) and one producer-vocabulary definition (a colour read through the SQL
//! adapter, rule `producer_mapped`), against the Connectors double with recorded answers.
//!
//! The instance is `tests/fixtures/every-source/instance`. Each test copies it and one set of
//! recordings (`tests/fixtures/every-source/recordings/common`, then a scenario that replaces
//! files of the same name) into its own directory, links `connectors` there to the double, and
//! runs the pass of `thalamus_pipeline::sample` and `thalamus_pipeline::evaluate` into its own
//! SQLite store. Every definition grades against a `fixed` baseline, so grading reads nothing
//! more from a source and each verdict is the sample's own.
//!
//! The exit of a pass is `thalamus_core::ladder::pass_exit` over the verdicts the pass recorded,
//! the rule `thalamus eval` and `thalamus show` exit by (design § 2.3).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use thalamus_core::ladder;
use thalamus_model::definition::SignalId;
use thalamus_model::evaluation::Verdict;
use thalamus_model::observation::Reading;
use thalamus_model::primitives::Timestamp;
use thalamus_pipeline::evaluate;
use thalamus_pipeline::instance::LoadedInstance;
use thalamus_pipeline::sample::{self, Producer};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry::{self, SourceKind};
use thalamus_store::eventlog::{EVALUATION_SCHEMA, EventlogStore, Locator, SAMPLE_SCHEMA};

/// The window end of every pass: 1792018623 in Unix seconds, the instant the recordings answer.
const END: &str = "2026-10-14T22:57:03Z";

const PROMETHEUS: &str = "service.api.failure_ratio";
const LOKI: &str = "service.logs.error_ratio";
const SQL: &str = "service.queue.depth";
const KUBERNETES: &str = "platform.pods.pending";
const GIT: &str = "delivery.changes.merged";
const HTTP_PROBE: &str = "service.api.up";
const COLOUR: &str = "gate.ci.colour";

/// Every definition of the fixture instance.
const ALL: [&str; 7] = [PROMETHEUS, LOKI, SQL, KUBERNETES, GIT, HTTP_PROBE, COLOUR];

/// The source kinds the story names, by the extractor prefix that chooses each.
const KINDS: [SourceKind; 6] = [
    SourceKind::Prometheus,
    SourceKind::Loki,
    SourceKind::Sql,
    SourceKind::Kubernetes,
    SourceKind::Git,
    SourceKind::HttpProbe,
];

/// The five ladder verdicts as the wire names them (design § 2.3).
const LADDER: [&str; 5] = ["OK", "N/A", "WARN", "CRIT", "UNKNOWN"];

/// The Connectors double, built once: it is a binary of `thalamus-sources`, so the build of this
/// package's tests does not make it.
fn double() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let status = Command::new(env!("CARGO"))
                .args([
                    "build",
                    "--locked",
                    "--quiet",
                    "-p",
                    "thalamus-sources",
                    "--bin",
                    "connectors-double",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .status()
                .expect("cargo runs");
            assert!(status.success(), "the Connectors double builds");
            // This test runs from `<target>/<profile>/deps/`; the double lands in `<profile>/`.
            let bin = std::env::current_exe()
                .expect("the test knows its path")
                .parent()
                .and_then(Path::parent)
                .expect("the test runs from a profile's deps directory")
                .join("connectors-double");
            assert!(bin.exists(), "{} exists", bin.display());
            bin
        })
        .clone()
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/every-source")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory");
    for entry in std::fs::read_dir(from).expect("the fixture exists") {
        let path = entry.expect("a fixture entry").path();
        let target = to.join(path.file_name().expect("a file name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy a fixture file");
        }
    }
}

/// One test's directory: the instance, the recordings with the `connectors` link, the store.
struct World {
    root: PathBuf,
}

impl World {
    /// The fixture instance, and the `common` recordings overlaid by those of `scenario`.
    fn new(name: &str, scenario: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("every_source_one_ladder")
            .join(name);
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("clear the test directory");
        }
        copy_dir(&fixtures().join("instance"), &root.join("instance"));
        let recordings = root.join("recordings");
        for set in ["common", scenario] {
            copy_dir(&fixtures().join("recordings").join(set), &recordings);
        }
        std::os::unix::fs::symlink(double(), recordings.join("connectors"))
            .expect("link the double");
        Self { root }
    }

    fn locator(&self) -> Locator {
        Locator::Sqlite(self.root.join("store.sqlite"))
    }

    /// One pass over `signals`: sample them at [`END`], then grade what that run sampled. Nothing
    /// may be refused: every read is recorded and every sample graded.
    fn pass(&self, signals: &[&str]) -> Ran {
        let mut instance = LoadedInstance::load(&self.root.join("instance"))
            .unwrap_or_else(|errors| panic!("the fixture instance loads: {errors:#?}"));
        let mut store =
            EventlogStore::open(&self.locator(), instance.name()).expect("the store opens");
        let connectors = Connectors::new(self.root.join("recordings/connectors"));
        let ids: Vec<SignalId> = signals
            .iter()
            .map(|signal| sample::admit(&instance, signal).expect("an active definition"))
            .collect();
        let producer = Producer {
            name: "thalamus-test".to_owned(),
            version: "0.0.0".to_owned(),
        };
        let sampled = sample::run(
            &mut instance,
            &mut store,
            &connectors,
            &ids,
            &Timestamp(END.to_owned()),
            &producer,
        )
        .expect("the sampling pass runs");
        assert!(
            sampled.refused.is_empty(),
            "no read is refused: {:#?}",
            sampled.refused
        );
        let evaluated = evaluate::run(
            &instance,
            &mut store,
            &connectors,
            &ids,
            Some(&sampled.run_id),
        )
        .expect("the grading pass runs");
        assert!(
            evaluated.refused.is_empty(),
            "no sample is refused: {:#?}",
            evaluated.refused
        );
        drop(store);
        let store = EventlogStore::open(&self.locator(), instance.name()).expect("the store opens");
        Ran {
            instance,
            sampled,
            evaluated,
            store,
        }
    }
}

/// What one pass recorded, and the store it recorded into, opened again.
struct Ran {
    instance: LoadedInstance,
    sampled: sample::Pass,
    evaluated: evaluate::Pass,
    store: EventlogStore,
}

impl Ran {
    /// The one evaluation the pass recorded for `signal`.
    fn evaluation(&self, signal: &str) -> &evaluate::Recorded {
        let mut of = self
            .evaluated
            .recorded
            .iter()
            .filter(|recorded| recorded.signal.0 == signal);
        let (Some(one), None) = (of.next(), of.next()) else {
            panic!("one evaluation of {signal}: {:#?}", self.evaluated.recorded);
        };
        one
    }

    fn verdict(&self, signal: &str) -> Verdict {
        self.evaluation(signal).verdict
    }

    /// The verdict of every signal the pass graded.
    fn verdicts(&self) -> BTreeMap<String, Verdict> {
        self.evaluated
            .recorded
            .iter()
            .map(|recorded| (recorded.signal.0.clone(), recorded.verdict))
            .collect()
    }

    /// The exit code of the pass: the ladder exit of its worst verdict, 3 when every one is N/A.
    fn exit(&self) -> i32 {
        ladder::pass_exit(
            self.evaluated
                .recorded
                .iter()
                .map(|recorded| recorded.verdict),
        )
    }

    /// The `verdict` member of every stored evaluation line, as the store holds it.
    fn stored_verdicts(&self) -> Vec<String> {
        self.store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(EVALUATION_SCHEMA))
            .map(|record| {
                record.body()["verdict"]
                    .as_str()
                    .unwrap_or_else(|| {
                        panic!("a stored evaluation names its verdict: {}", record.body())
                    })
                    .to_owned()
            })
            .collect()
    }

    fn stored_count(&self, schema: &str) -> usize {
        self.store
            .records()
            .iter()
            .filter(|record| record.schema() == Some(schema))
            .count()
    }
}

/// Every evaluation of the pass is one of the five ladder verdicts, in the pass result and in
/// every stored line, and the pass covers every source kind with one sample and one evaluation
/// per definition. The colour producer's own word (`green`) never reaches a verdict: the sample
/// keeps it as `text` and grading maps it to `OK`.
#[test]
fn every_source_kind_yields_only_ladder_verdicts() {
    let ran = World::new("every_source_kind", "mixed").pass(&ALL);

    let kinds: BTreeSet<&str> = ALL
        .iter()
        .map(|signal| {
            let definition = ran
                .instance
                .instance()
                .definition(signal)
                .expect("the definition exists");
            registry::choose(&definition.data().extractor)
                .unwrap_or_else(|| panic!("{signal}: an adapter takes its extractor"))
                .prefix()
        })
        .collect();
    let named: BTreeSet<&str> = KINDS.iter().map(|kind| kind.prefix()).collect();
    assert!(
        named.is_subset(&kinds),
        "the instance holds a definition of every source kind: {kinds:?}"
    );

    assert_eq!(
        ran.sampled.recorded.len(),
        ALL.len(),
        "one sample per definition: {:#?}",
        ran.sampled.recorded
    );
    assert_eq!(
        ran.evaluated.recorded.len(),
        ALL.len(),
        "one evaluation per definition: {:#?}",
        ran.evaluated.recorded
    );
    assert_eq!(ran.stored_count(SAMPLE_SCHEMA), ALL.len());
    assert_eq!(ran.stored_count(EVALUATION_SCHEMA), ALL.len());

    let stored = ran.stored_verdicts();
    assert_eq!(stored.len(), ALL.len(), "{stored:?}");
    for verdict in &stored {
        assert!(
            LADDER.contains(&verdict.as_str()),
            "a stored verdict is a ladder verdict: {verdict:?} in {stored:?}"
        );
    }
    for recorded in &ran.evaluated.recorded {
        assert!(
            LADDER.contains(&evaluate::verdict_name(recorded.verdict)),
            "{recorded:#?}"
        );
    }

    let expected = BTreeMap::from([
        (PROMETHEUS.to_owned(), Verdict::OK),
        (LOKI.to_owned(), Verdict::UNKNOWN),
        (SQL.to_owned(), Verdict::CRIT),
        (KUBERNETES.to_owned(), Verdict::WARN),
        (GIT.to_owned(), Verdict::OK),
        (HTTP_PROBE.to_owned(), Verdict::OK),
        (COLOUR.to_owned(), Verdict::OK),
    ]);
    assert_eq!(ran.verdicts(), expected);

    let colour = ran
        .store
        .sample(&ran.evaluation(COLOUR).sample_id)
        .expect("the colour sample is stored");
    assert_eq!(colour.text.as_deref(), Some("green"), "{colour:#?}");
    assert!(
        matches!(colour.reading, Reading::Unknown(_)),
        "the SQL adapter keeps a text cell unknown; only grading maps it: {colour:#?}"
    );
}

/// A read refused by Connectors, from every source kind, is recorded as an `unknown` sample and
/// graded `UNKNOWN` with a reason and no score; never `OK`, never `N/A`, never a 0.
#[test]
fn unknown_reading_from_every_source_grades_unknown() {
    let ran = World::new("unknown_reading", "unknown").pass(&ALL);

    assert_eq!(
        ran.sampled.recorded.len(),
        ALL.len(),
        "an unknown read is recorded, not dropped: {:#?}",
        ran.sampled.recorded
    );
    for recorded in &ran.sampled.recorded {
        assert_eq!(recorded.quality, "unknown", "{recorded:#?}");
        let stored = ran
            .store
            .sample(&recorded.sample_id)
            .expect("the sample is stored");
        assert!(
            matches!(stored.reading, Reading::Unknown(_)),
            "an unknown reading carries no value: {stored:#?}"
        );
    }

    for signal in ALL {
        assert_eq!(ran.verdict(signal), Verdict::UNKNOWN, "{signal}");
    }
    for evaluation in ran.store.current_evaluations() {
        assert_eq!(evaluation.verdict, Verdict::UNKNOWN, "{evaluation:#?}");
        assert!(
            evaluation.score.is_none(),
            "UNKNOWN has no score: {evaluation:#?}"
        );
        assert!(
            evaluation
                .reason
                .as_deref()
                .is_some_and(|reason| !reason.is_empty()),
            "UNKNOWN names its reason: {evaluation:#?}"
        );
    }
    assert_eq!(ran.stored_verdicts(), vec!["UNKNOWN"; ALL.len()]);
    assert_eq!(ran.exit(), 3);
}

/// The pass exits with the ladder exit of its worst verdict, `CRIT > WARN > UNKNOWN > OK = N/A`:
/// a `CRIT` from SQL beside an `UNKNOWN` from Loki exits 2; `UNKNOWN` beside `OK` exits 3; a pass
/// in which every verdict is `N/A` exits 3.
#[test]
fn mixed_pass_exits_with_worst_verdict_exit() {
    let every = World::new("mixed_every", "mixed").pass(&ALL);
    assert_eq!(every.verdict(SQL), Verdict::CRIT);
    assert_eq!(every.verdict(LOKI), Verdict::UNKNOWN);
    assert_eq!(
        ladder::worst(every.verdicts().into_values()),
        Some(Verdict::CRIT)
    );
    assert_eq!(every.exit(), 2, "{:#?}", every.verdicts());

    let unknown_beside_ok = World::new("mixed_unknown_ok", "mixed").pass(&[LOKI, PROMETHEUS]);
    assert_eq!(unknown_beside_ok.verdict(LOKI), Verdict::UNKNOWN);
    assert_eq!(unknown_beside_ok.verdict(PROMETHEUS), Verdict::OK);
    assert_eq!(
        unknown_beside_ok.exit(),
        3,
        "{:#?}",
        unknown_beside_ok.verdicts()
    );

    let all_na = World::new("mixed_all_na", "sparse").pass(&[PROMETHEUS, LOKI]);
    assert_eq!(all_na.verdict(PROMETHEUS), Verdict::NA);
    assert_eq!(all_na.verdict(LOKI), Verdict::NA);
    assert_eq!(all_na.exit(), 3, "{:#?}", all_na.verdicts());
}

/// The JSON Schema of an entity (`ess generate --kind schema`), compiled for validation.
fn compile(file: &str) -> (boon::Schemas, boon::SchemaIndex) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../website/static/schemas")
        .join(file);
    let text = std::fs::read_to_string(&path).expect("the generated schema exists");
    let document: serde_json::Value = serde_json::from_str(&text).expect("the schema is JSON");
    let url = format!("https://schemas.invalid/{file}");
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler
        .add_resource(&url, document)
        .expect("the schema is a resource");
    let index = compiler
        .compile(&url, &mut schemas)
        .unwrap_or_else(|error| panic!("the schema compiles: {error}"));
    (schemas, index)
}

/// A stored line as the entity snapshot the store serves: the event body without the envelope key
/// `schema`, with the one state both entities have, `Recorded`, which the store supplies on read.
fn entity(body: &serde_json::Value) -> serde_json::Value {
    let mut line = body.clone();
    line.as_object_mut()
        .expect("a stored line is an object")
        .remove("schema");
    line.as_object_mut()
        .expect("a stored line is an object")
        .insert("state".to_owned(), serde_json::json!("Recorded"));
    line
}

/// Every stored sample and evaluation line of the pass validates against the generated JSON Schema
/// of its entity (`signal-sample.schema.json`, `signal-evaluation.schema.json`). The check is not
/// vacuous: a line with a required field removed, and one with a wrong type, are refused.
#[test]
fn every_recorded_sample_and_evaluation_validates_against_generated_types() {
    let ran = World::new("validates", "mixed").pass(&ALL);
    let (sample_schemas, sample) = compile("signal-sample.schema.json");
    let (evaluation_schemas, evaluation) = compile("signal-evaluation.schema.json");

    let (mut samples, mut evaluations) = (0, 0);
    for record in ran.store.records() {
        let (schemas, index) = match record.schema() {
            Some(SAMPLE_SCHEMA) => {
                samples += 1;
                (&sample_schemas, sample)
            }
            Some(EVALUATION_SCHEMA) => {
                evaluations += 1;
                (&evaluation_schemas, evaluation)
            }
            _ => continue,
        };
        let line = entity(record.body());
        if let Err(error) = schemas.validate(&line, index) {
            panic!("a stored line validates: {error:#}\n{line}");
        }

        // Negative cases: the same line with a field removed, and with a wrong type.
        let first = "signal";
        let mut removed = line.clone();
        removed.as_object_mut().expect("an object").remove(first);
        assert!(
            schemas.validate(&removed, index).is_err(),
            "a line without `{first}` is refused"
        );
        let mut retyped = line.clone();
        retyped["signal"] = serde_json::json!(7);
        assert!(
            schemas.validate(&retyped, index).is_err(),
            "a line with a numeric `signal` is refused"
        );
    }
    assert_eq!(samples, ALL.len());
    assert_eq!(evaluations, ALL.len());
}

/// The colour `na`, read through the SQL adapter as a text cell, is recorded with its text and
/// graded `UNKNOWN` by the definition's mapping, with a reason naming the value; never `N/A`.
#[test]
fn colour_na_through_sql_adapter_is_unknown() {
    let ran = World::new("colour_na", "colour-na").pass(&[COLOUR]);

    let evaluation = ran.evaluation(COLOUR);
    let sample = ran
        .store
        .sample(&evaluation.sample_id)
        .expect("the colour sample is stored");
    assert_eq!(sample.provenance.adapter.0, "sql", "{sample:#?}");
    assert_eq!(sample.text.as_deref(), Some("na"), "{sample:#?}");
    assert!(matches!(sample.reading, Reading::Unknown(_)), "{sample:#?}");

    assert_eq!(evaluation.verdict, Verdict::UNKNOWN);
    let stored = ran
        .store
        .current_evaluations()
        .into_iter()
        .find(|stored| stored.sample_id == evaluation.sample_id)
        .expect("the evaluation is stored");
    assert!(stored.score.is_none(), "{stored:#?}");
    assert!(
        stored
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("`na`")),
        "the reason names the producer value: {stored:#?}"
    );
    assert_eq!(ran.stored_verdicts(), vec!["UNKNOWN"]);
    assert_eq!(ran.exit(), 3);
}
