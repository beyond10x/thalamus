//! Adversary cases, second pass, for the `git` adapter's ratio after the design § 11 D20
//! correction: a window that holds records of which none matches reads unknown with no population
//! ([`thalamus_sources::git::NO_MATCH`]).
//!
//! The harness is the one in `tests/adv_git.rs`: each case copies `tests/fixtures/git/common` into
//! a fresh directory under the build's temporary directory and writes the page answers it needs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{CompleteReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Population, Read};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::git::{EMPTY_POPULATION, NO_MATCH};
use thalamus_sources::registry;

const PROJECT: &str = "group/app";
const END: &str = "2026-10-14T22:57:03Z";
const INSIDE: &str = "2026-10-10T12:00:00.000Z";
/// Before the window's start: GitLab answered it, the adapter places it outside.
const BEFORE: &str = "2026-10-01T12:00:00.000Z";

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv-git-p2")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let common = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/git/common");
        for entry in std::fs::read_dir(common).expect("common recordings") {
            let path = entry.expect("fixture entry").path();
            std::fs::copy(&path, dir.join(path.file_name().expect("file name")))
                .expect("copy a recording");
        }
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_connectors-double"),
            dir.join("connectors"),
        )
        .expect("link the double");
        World { dir }
    }

    /// Records the answers of `pipelines.list`, page `n` answering `pages[n - 1]`.
    fn pipelines(&self, pages: Vec<Answer>) -> &World {
        let operation = "pipelines.list";
        let answers: Vec<Value> = pages
            .into_iter()
            .enumerate()
            .map(|(at, answer)| answer.recording(operation, at as u64 + 1))
            .collect();
        std::fs::write(
            self.dir.join(format!(
                "operations+invoke+a=forge+c=code-host+o={operation}.json"
            )),
            Value::Array(answers).to_string(),
        )
        .expect("write the invoke recording");
        self
    }

    fn read(&self, definition: &SignalDefinitionData) -> Vec<Read> {
        registry::read(
            &Connectors::new(self.dir.join("connectors")),
            definition,
            &Timestamp(END.into()),
            &RunId("run_test".into()),
        )
    }
}

enum Answer {
    Records(Vec<Value>),
    Refused,
}

impl Answer {
    fn recording(self, operation: &str, page: u64) -> Value {
        match self {
            Answer::Records(body) => {
                let document = json!({
                    "status": 200,
                    "body": body,
                    "provenance": {"instance": "forge", "observed_at_unix_ms": 1_792_018_623_000_u64,
                                   "resource": "/api/v4/projects/{id}/pipelines", "source_revision": "0f5662970b38"},
                });
                json!({"input": {"page": page}, "stdout": {"ok": true, "result": {
                    "adapter": "forge", "operation": operation, "revision": "op-rev-1",
                    "result": document.to_string()}}})
            }
            Answer::Refused => json!({"input": {"page": page}, "exit": 1, "stderr": {"ok": false,
                "error": {"code": "failure", "message": "the provider did not answer",
                          "data": {"kind": "operational", "code": "provider_unavailable",
                                   "stage": "dispatch", "next_action": "retry_later"}}}}),
        }
    }
}

fn pipeline_at(id: u64, status: &str, r#ref: &str, at: &str) -> Value {
    json!({"id": id, "iid": id, "project_id": 7, "status": status, "ref": r#ref,
           "created_at": at, "updated_at": at})
}

fn pipeline(id: u64, status: &str, r#ref: &str) -> Value {
    pipeline_at(id, status, r#ref, INSIDE)
}

/// `git.ratio` of failed pipelines over pipelines, population unit `{pipeline}`.
fn failure_ratio(dimensions: &[&str]) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("delivery.project.adversary".into()),
        title: "Adversary".into(),
        family: Family("delivery".into()),
        kind: SignalKind::Ratio,
        unit: Some(Unit("1".into())),
        dimensions: Some(
            dimensions
                .iter()
                .map(|k| DimensionKey((*k).into()))
                .collect(),
        ),
        source: Source {
            adapter: AdapterId("forge".into()),
            connection: ConnectionId("code-host".into()),
        },
        extractor: ExtractorId("git.ratio".into()),
        query: [
            ("project", PROJECT),
            ("records", "pipelines"),
            ("match", r#"{"status":"failed"}"#),
        ]
        .iter()
        .map(|(k, v)| ((*k).to_string(), QueryText((*v).into())))
        .collect(),
        window: Duration("P7D".into()),
        cadence: Duration("P1D".into()),
        history: History::Source,
        denominator: Some(DenominatorSpec {
            unit: Unit("{pipeline}".into()),
            min_n: 1,
            since: "first_sample".into(),
        }),
        evaluation: EvaluationSpec {
            rule: RuleId("threshold".into()),
            baselines: Vec::new(),
            warn: None,
            crit: None,
            min_trend_window: None,
        },
        objective: None,
        owner: Some(Owner("team:delivery".into())),
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

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn population(n: i64) -> Option<Population> {
    Some(Population {
        n,
        unit: Unit("{pipeline}".into()),
    })
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

fn dims(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// The read of the series `want`, of which there is exactly one.
fn series<'a>(reads: &'a [Read], want: &BTreeMap<String, String>) -> &'a Read {
    let found: Vec<&Read> = reads.iter().filter(|r| &r.dims == want).collect();
    assert_eq!(found.len(), 1, "series {want:?}: {reads:#?}");
    found[0]
}

// --- The guard, per series or once ------------------------------------------------------------

/// Every peer adapter on `main` reads the D20 "numerator with no series over a non-empty
/// denominator" case as one `unknown` per denominator series, with that series' dims and no
/// population: `sql.rs:505-512` (`denominator.keys().map(..)`), `prometheus.rs:294-305` (every
/// joined series), and `loki.rs` ("reads every denominator label set `unknown`",
/// `story:m3-loki-empty-numerator`: "reads unknown for every series instead of 0"). In the git
/// adapter the denominator series are the `ref`s of the window's records. The guard instead
/// returns one read with empty dims, so the series `ref=main`, `ref=release`, `ref=feature` get no
/// read at all in this pass.
#[test]
fn adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers() {
    let w = World::new("adv2_no_match_by_ref");
    w.pipelines(vec![Answer::Records(vec![
        pipeline(1, "success", "main"),
        pipeline(2, "success", "main"),
        pipeline(3, "success", "release"),
        pipeline(4, "canceled", "feature"),
    ])]);
    let reads = w.read(&failure_ratio(&["ref"]));
    for r in ["main", "release", "feature"] {
        let read = series(&reads, &dims(&[("ref", r)]));
        assert_eq!(read.denominator, None, "{r}: no population: {read:#?}");
        assert!(reason(read).contains(NO_MATCH), "{r}: {read:#?}");
    }
    assert_eq!(
        reads.len(),
        3,
        "one read per series, nothing else: {reads:#?}"
    );
}

// --- The guard's reading: reason and population -----------------------------------------------

/// Without dimensions: exactly one `unknown`, reason `git.ratio: NO_MATCH`, and no population
/// (module doc: "reads one `unknown` with no population"). A population of 4 would hand a count
/// to the evaluator for a ratio nobody read.
#[test]
fn adv2_git_ratio_no_match_has_the_no_match_reason_and_no_population() {
    let w = World::new("adv2_no_match_plain");
    w.pipelines(vec![Answer::Records(vec![
        pipeline(1, "success", "main"),
        pipeline(2, "success", "main"),
        pipeline(3, "canceled", "release"),
        pipeline(4, "running", "feature"),
    ])]);
    let reads = w.read(&failure_ratio(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reason(&reads[0]), format!("git.ratio: {NO_MATCH}"));
    assert_eq!(reads[0].denominator, None, "{reads:#?}");
    assert!(reads[0].dims.is_empty(), "{reads:#?}");
}

/// Matching records GitLab answered outside the window do not count as a match: the guard looks
/// at the placed records only.
#[test]
fn adv2_git_ratio_matches_only_outside_the_window_is_no_match() {
    let w = World::new("adv2_match_outside_window");
    w.pipelines(vec![Answer::Records(vec![
        pipeline_at(1, "failed", "main", BEFORE),
        pipeline_at(2, "failed", "main", END),
        pipeline(3, "success", "main"),
        pipeline(4, "success", "main"),
    ])]);
    let reads = w.read(&failure_ratio(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reason(&reads[0]), format!("git.ratio: {NO_MATCH}"));
}

/// A cut walk with no match names the cut, not the matcher: records past the cut may match, so
/// "the matcher matches nothing" would be a claim the read cannot make.
#[test]
fn adv2_git_ratio_cut_walk_with_no_match_names_the_cut() {
    let w = World::new("adv2_cut_no_match");
    let full: Vec<Value> = (1..=100)
        .map(|id| pipeline(id, "success", "main"))
        .collect();
    w.pipelines(vec![Answer::Records(full), Answer::Refused]);
    for dimensions in [&[][..], &["ref"][..]] {
        let reads = w.read(&failure_ratio(dimensions));
        assert_eq!(reads.len(), 1, "{dimensions:?}: {reads:#?}");
        let why = reason(&reads[0]);
        assert!(why.contains("page 2"), "{dimensions:?}: {why}");
        assert!(!why.contains(NO_MATCH), "{dimensions:?}: {why}");
        assert_eq!(reads[0].denominator, None, "{dimensions:?}");
    }
}

/// A window holding no record keeps its own reason and population 0; the no-match guard does not
/// swallow it.
#[test]
fn adv2_git_ratio_empty_window_keeps_population_zero() {
    let w = World::new("adv2_empty_window");
    w.pipelines(vec![Answer::Records(vec![pipeline_at(
        1, "failed", "main", BEFORE,
    )])]);
    for dimensions in [&[][..], &["ref"][..]] {
        let reads = w.read(&failure_ratio(dimensions));
        assert_eq!(reads.len(), 1, "{dimensions:?}: {reads:#?}");
        assert_eq!(reason(&reads[0]), format!("git.ratio: {EMPTY_POPULATION}"));
        assert_eq!(reads[0].denominator, population(0), "{dimensions:?}");
    }
}

// --- dims x match: the legitimate 0 -----------------------------------------------------------

/// D20's legitimate 0: one series matches, so the matcher is shown to work, and every other
/// series is a ratio of 0 over the population the adapter read whole. Exactly one matching record
/// in the whole window, so a guard counting "at most one match" as none is caught too.
#[test]
fn adv2_git_ratio_one_match_in_one_series_leaves_the_others_zero_over_their_population() {
    let w = World::new("adv2_one_series_matches");
    w.pipelines(vec![Answer::Records(vec![
        pipeline(1, "failed", "main"),
        pipeline(2, "success", "main"),
        pipeline(3, "success", "release"),
        pipeline(4, "success", "release"),
        pipeline(5, "success", "feature"),
        pipeline(6, "canceled", "feature"),
        pipeline(7, "success", "feature"),
    ])]);
    let reads = w.read(&failure_ratio(&["ref"]));
    assert_eq!(reads.len(), 3, "{reads:#?}");
    for (r, value, n) in [("main", "0.5", 2), ("release", "0", 2), ("feature", "0", 3)] {
        let read = series(&reads, &dims(&[("ref", r)]));
        assert_eq!(read.reading, ok(value), "{r}");
        assert_eq!(read.denominator, population(n), "{r}");
    }

    // Without dimensions the same window is 1 of 7.
    let reads = w.read(&failure_ratio(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("0.142857142857142857"));
    assert_eq!(reads[0].denominator, population(7));
}

/// A record without the dimension field belongs to the series with no dims. When it is the only
/// match, the matcher is shown to work and the `ref` series read 0 over their counts.
#[test]
fn adv2_git_ratio_match_only_in_the_series_without_dims() {
    let w = World::new("adv2_match_in_undimensioned_series");
    let mut no_ref = pipeline(1, "failed", "main");
    no_ref.as_object_mut().expect("object").remove("ref");
    w.pipelines(vec![Answer::Records(vec![
        no_ref,
        pipeline(2, "success", "main"),
        pipeline(3, "success", "release"),
    ])]);
    let reads = w.read(&failure_ratio(&["ref"]));
    assert_eq!(reads.len(), 3, "{reads:#?}");
    assert_eq!(series(&reads, &BTreeMap::new()).reading, ok("1"));
    assert_eq!(
        series(&reads, &dims(&[("ref", "main")])).reading,
        ok("0"),
        "{reads:#?}"
    );
    assert_eq!(
        series(&reads, &dims(&[("ref", "release")])).reading,
        ok("0"),
        "{reads:#?}"
    );
}
