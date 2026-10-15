//! Adversary cases for the `kubernetes` adapter, against the recorded Connectors double.
//!
//! Each case copies `tests/fixtures/kubernetes/common`, then its scenarios, into a fresh directory
//! under the build's temporary directory, as `tests/kubernetes.rs` does.

use std::path::{Path, PathBuf};

use serde_json::Value;
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, EvaluationSpec, ExtractorId, Family, History, Links, Owner,
    QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Bound, CompleteReading, PartialReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str, scenarios: &[&str]) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv-kubernetes")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/kubernetes");
        for scenario in std::iter::once(&"common").chain(scenarios) {
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
        World { dir }
    }

    /// The `--input-json` of every `operations invoke` of `operation`, in order.
    fn inputs(&self, operation: &str) -> Vec<Value> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect();
                let flag = |name: &str| {
                    let at = argv.iter().position(|a| a == name)?;
                    argv.get(at + 1).cloned()
                };
                if flag("--operation").as_deref() != Some(operation) {
                    return None;
                }
                serde_json::from_str(&flag("--input-json")?).ok()
            })
            .collect()
    }
}

fn definition(predicate: Option<&str>) -> SignalDefinitionData {
    let mut query = vec![("kind", "pods")];
    if let Some(predicate) = predicate {
        query.push(("predicate", predicate));
    }
    SignalDefinitionData {
        signal_id: SignalId("platform.namespace.running_pods".into()),
        title: "Pods running in a namespace".into(),
        family: Family("platform".into()),
        kind: SignalKind::Instant,
        unit: Some(Unit("{pod}".into())),
        dimensions: Some(vec![DimensionKey("namespace".into())]),
        source: Source {
            adapter: AdapterId("kubernetes".into()),
            connection: ConnectionId("kubernetes-main".into()),
        },
        extractor: ExtractorId("kubernetes.count".into()),
        query: query
            .iter()
            .map(|(k, v)| (k.to_string(), QueryText(v.to_string())))
            .collect(),
        window: Duration("PT5M".into()),
        cadence: Duration("PT5M".into()),
        history: History::Store,
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

fn read(world: &World, definition: &SignalDefinitionData) -> Vec<Read> {
    registry::read(
        &Connectors::new(world.dir.join("connectors")),
        definition,
        &Timestamp("2026-10-14T22:57:03Z".into()),
        &RunId("run_adv".into()),
    )
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn alpha(reads: &[Read]) -> Reading {
    reads
        .iter()
        .find(|r| r.dims.get("namespace").map(String::as_str) == Some("alpha"))
        .map(|r| r.reading.clone())
        .unwrap_or_else(|| {
            reads
                .first()
                .map(|r| r.reading.clone())
                .expect("at least one read")
        })
}

/// A predicate in a spelling the parser does not understand is either read as meant (`right`)
/// or refused (unknown). It is never a complete count of some other question.
fn assert_meant_or_unknown(predicate: &str, right: &str) {
    let name = format!(
        "adv_predicate_{}",
        predicate
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
    );
    let w = World::new(&name, &["per-namespace"]);
    let reads = read(&w, &definition(Some(predicate)));
    let reading = alpha(&reads);
    assert!(
        reading == ok(right) || matches!(reading, Reading::Unknown(_)),
        "predicate {predicate:?}: alpha has 3 Running pods of 4 (api-1 Running), so the read is \
         {right} or unknown, not {reading:?}; all reads: {reads:?}"
    );
}

/// Kubernetes field selectors spell equality `=` or `==` (`kubectl --field-selector
/// status.phase==Running`). `==` is split at its first `=`, so the value becomes `=Running` and
/// every namespace reads a complete 0.
#[test]
fn adv_double_equals_predicate_is_never_a_complete_zero() {
    assert_meant_or_unknown("status.phase==Running", "3");
}

/// Field selectors join requirements with `,`. The whole tail becomes the value, matching
/// nothing, and the read is a complete 0.
#[test]
fn adv_field_selector_conjunction_is_never_a_complete_zero() {
    assert_meant_or_unknown("status.phase=Running,metadata.name=api-1", "1");
}

/// A quoted value compares the quotes too: a complete 0.
#[test]
fn adv_quoted_predicate_value_is_never_a_complete_zero() {
    assert_meant_or_unknown("status.phase=\"Running\"", "3");
}

/// `namespaces.list` answers no configured namespace (every configured name is absent: the real
/// adapter omits a 404). The definition then has no read at all, so the pass carries no sample
/// for it and nothing reads UNKNOWN; a configuration naming no existing namespace is invisible.
#[test]
fn adv_no_configured_namespace_reads_unknown_not_nothing() {
    let w = World::new(
        "adv_no_configured_namespace_reads_unknown_not_nothing",
        &["adv-no-namespace"],
    );
    let reads = read(&w, &definition(Some("status.phase=Running")));
    assert!(
        !reads.is_empty(),
        "a definition read against a cluster holding none of its namespaces has no read at all"
    );
    assert!(
        reads
            .iter()
            .all(|r| matches!(r.reading, Reading::Unknown(_))),
        "{reads:?}"
    );
}

/// A continuation that comes back a second time ends the list as a lower bound after exactly
/// the two pages read, never a loop of `MAX_PAGES` calls. No existing case reaches the repeated
/// cursor branch.
#[test]
fn adv_repeated_continuation_is_partial_lower_bound_and_stops() {
    let w = World::new(
        "adv_repeated_continuation_is_partial_lower_bound_and_stops",
        &["adv-repeated-cursor"],
    );
    let reads = read(&w, &definition(Some("status.phase=Running")));
    assert_eq!(
        alpha(&reads),
        Reading::Partial(PartialReading {
            value: Decimal("3".into()),
            bound: Bound::Lower,
        }),
        "{reads:?}"
    );
    let alpha_calls = w
        .inputs("resources.list")
        .into_iter()
        .filter(|input| input["namespace"] == "alpha")
        .count();
    assert_eq!(alpha_calls, 2, "{:?}", w.inputs("resources.list"));
}

// Pass 2: the closed predicate grammar.

/// Each `(predicate, meant)` over `per-namespace`, where alpha lists api-1, api-2 and api-4
/// Running and api-3 Pending. Alpha reads `meant` when the spelling has one meaning, or unknown;
/// never a complete count of another question. Returns every spelling that broke that.
fn misread(name: &str, spellings: &[(&str, Option<&str>)]) -> Vec<String> {
    let w = World::new(name, &["per-namespace"]);
    let mut out = Vec::new();
    for (predicate, meant) in spellings {
        let reading = alpha(&read(&w, &definition(Some(predicate))));
        let fine = matches!(reading, Reading::Unknown(_))
            || meant.is_some_and(|meant| reading == ok(meant));
        if !fine {
            out.push(format!("{predicate:?} -> {reading:?} (meant {meant:?})"));
        }
    }
    out
}

/// Matcher spellings from the other sources thalamus reads (PromQL and LogQL `=~`), and
/// alternation or glob in the value, pass the value blacklist and read a complete 0.
#[test]
fn adv2_regex_glob_and_alternation_values_are_never_a_complete_count() {
    let broken = misread(
        "adv2_regex_glob_and_alternation_values_are_never_a_complete_count",
        &[
            ("status.phase=~Running", Some("3")),
            ("status.phase=~Running|Pending", Some("4")),
            ("status.phase=Running|Pending", Some("4")),
            ("metadata.name=api-*", Some("4")),
            ("status.phase=Run.*", Some("3")),
        ],
    );
    assert!(
        broken.is_empty(),
        "complete counts of another question: {broken:#?}"
    );
}

/// A value with whitespace or an invisible character inside passes and reads a complete 0.
#[test]
fn adv2_whitespace_and_invisible_characters_in_values_are_never_a_complete_count() {
    let broken = misread(
        "adv2_whitespace_and_invisible_characters_in_values_are_never_a_complete_count",
        &[
            ("status.phase=Running Pending", None),
            ("status.phase=Running or Pending", None),
            ("status.phase=Running\u{200b}", Some("3")),
            ("status.phase=Runn\u{0456}ng", None),
            ("status.phase=Running\tPending", None),
        ],
    );
    assert!(
        broken.is_empty(),
        "complete counts of another question: {broken:#?}"
    );
}

/// A field that is an object or a list compares its JSON text with the value: `status=Running`
/// reads a complete 0 and `status!=Running` counts every pod. Neither question has an answer.
#[test]
fn adv2_non_scalar_field_is_never_a_complete_count() {
    let broken = misread(
        "adv2_non_scalar_field_is_never_a_complete_count",
        &[
            ("status=Running", None),
            ("status!=Running", None),
            ("metadata=api-1", None),
        ],
    );
    assert!(
        broken.is_empty(),
        "complete counts of another question: {broken:#?}"
    );
}
