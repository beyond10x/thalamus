//! The `git` adapter, read through the registry against recorded Connectors answers of the GitLab
//! catalog provider (`tests/fixtures/git/<scenario>`, replayed by `src/bin/connectors-double.rs`).
//!
//! Each test copies the recordings of `common` and of its scenario into a fresh directory under
//! the build's own temporary directory and links `connectors` there to the double.
//!
//! The recordings: `merged` holds two pages of merge requests, a full one of 100 and a short one of
//! 4, two of which lie on the window's edges; `cut` holds the same first page and a refusal of the
//! second; `pipelines` holds one short page of 10 pipelines, one of them at the window end;
//! `refused` refuses the first page.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Bound, CompleteReading, PartialReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::{Population, Read, query_digest};
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const PROJECT: &str = "group/app";
const END: &str = "2026-10-14T22:57:03Z";
const START: &str = "2026-10-07T22:57:03Z";
const MERGED: &str = r#"{"state":"merged"}"#;
const FAILED: &str = r#"{"status":"failed"}"#;

/// One test's directory: recordings, the `connectors` link to the double, and the call log.
struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str, scenario: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("git")
            .join(name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear the test directory");
        }
        std::fs::create_dir_all(&dir).expect("create the test directory");
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/git");
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
        World { dir }
    }

    fn read(&self, definition: &SignalDefinitionData) -> Vec<Read> {
        registry::read(
            &Connectors::new(self.dir.join("connectors")),
            definition,
            &Timestamp(END.into()),
            &RunId("run_test".into()),
        )
    }

    /// Every call the double received, as its argv.
    fn calls(&self) -> Vec<Vec<String>> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .map(|call| {
                call["argv"]
                    .as_array()
                    .expect("argv")
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect()
            })
            .collect()
    }

    /// The `--input-json` document of every `operations invoke` the double received.
    fn invoke_inputs(&self) -> Vec<Value> {
        self.calls()
            .into_iter()
            .filter(|argv| argv.first().map(String::as_str) == Some("operations"))
            .filter(|argv| argv.get(1).map(String::as_str) == Some("invoke"))
            .filter_map(|argv| {
                let at = argv.iter().position(|a| a == "--input-json")?;
                serde_json::from_str(argv.get(at + 1)?).ok()
            })
            .collect()
    }
}

fn definition(
    extractor: &str,
    kind: SignalKind,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    SignalDefinitionData {
        signal_id: SignalId("delivery.project.merged_changes".into()),
        title: "Merged changes".into(),
        family: Family("delivery".into()),
        kind,
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
        extractor: ExtractorId(extractor.into()),
        query: query
            .iter()
            .map(|(k, v)| ((*k).to_string(), QueryText((*v).into())))
            .collect(),
        window: Duration("P7D".into()),
        cadence: Duration("P1D".into()),
        history: History::Source,
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

/// `git.count` of merged merge requests.
fn merged_count(dimensions: &[&str]) -> SignalDefinitionData {
    definition(
        "git.count",
        SignalKind::CountOfCases,
        &[
            ("project", PROJECT),
            ("records", "merge_requests"),
            ("filter", MERGED),
        ],
        dimensions,
    )
}

/// `git.ratio` of failed pipelines over pipelines.
fn failure_ratio(dimensions: &[&str]) -> SignalDefinitionData {
    let mut definition = definition(
        "git.ratio",
        SignalKind::Ratio,
        &[
            ("project", PROJECT),
            ("records", "pipelines"),
            ("match", FAILED),
        ],
        dimensions,
    );
    definition.denominator = Some(DenominatorSpec {
        unit: Unit("{pipeline}".into()),
        min_n: 5,
        since: "first_sample".into(),
    });
    definition
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

/// The read whose `key` dimension is `value`.
fn by<'a>(reads: &'a [Read], key: &str, value: &str) -> &'a Read {
    reads
        .iter()
        .find(|r| r.dims.get(key).map(String::as_str) == Some(value))
        .unwrap_or_else(|| panic!("no read for {key}={value} in {reads:#?}"))
}

/// No reading anywhere is the number 0: unknown is never 0.
fn assert_no_zero(reads: &[Read]) {
    for read in reads {
        let value = match &read.reading {
            Reading::Ok(r) => Some(&r.value.0),
            Reading::Partial(r) => Some(&r.value.0),
            Reading::Unknown(_) => None,
        };
        assert!(
            value.is_none_or(|v| !v
                .trim_start_matches('-')
                .trim_matches(['0', '.'])
                .is_empty()),
            "a reading is 0: {read:#?}"
        );
    }
}

fn pages_asked(inputs: &[Value]) -> Vec<Value> {
    inputs.iter().map(|i| i["page"].clone()).collect()
}

#[test]
fn git_merged_count_in_window_known_answer() {
    let w = World::new("git_merged_count_in_window_known_answer", "merged");
    let definition = merged_count(&[]);
    let reads = w.read(&definition);

    // 100 on the full first page, then of the short second page the one merged at the window
    // start (inside) and one more; the one merged at the window end and the one a millisecond
    // before the start lie outside.
    assert_eq!(reads.len(), 1, "{reads:#?}");
    let read = &reads[0];
    assert_eq!(read.reading, ok("102"));
    assert!(read.dims.is_empty());
    assert_eq!(read.window.start, Timestamp(START.into()));
    assert_eq!(read.window.end, Timestamp(END.into()));
    assert_eq!(read.query_sha256, query_digest(&definition.query));
    assert_eq!(read.adapter, AdapterId("forge".into()));
    assert_eq!(read.connection, ConnectionId("code-host".into()));
    assert_eq!(read.raw, None);
    assert_eq!(read.denominator, None);

    // The pages are read until a short one; each asks for the window and GitLab's largest page.
    let inputs = w.invoke_inputs();
    assert_eq!(pages_asked(&inputs), [json!(1), json!(2)], "{inputs:#?}");
    for input in &inputs {
        assert_eq!(input["id"], PROJECT);
        assert_eq!(input["state"], "merged", "the filter is sent as written");
        assert_eq!(input["merged_after"], START);
        assert_eq!(input["merged_before"], END);
        assert_eq!(input["per_page"], 100);
    }

    // Declared: target_branch and tenant. The records carry target_branch (null on one of them)
    // and source_branch, which is not declared; none carries tenant.
    let w = World::new("git_merged_count_by_target_branch", "merged");
    let reads = w.read(&merged_count(&["target_branch", "tenant"]));
    assert_eq!(reads.len(), 3, "one read per series: {reads:#?}");
    assert_eq!(by(&reads, "target_branch", "main").reading, ok("76"));
    assert_eq!(by(&reads, "target_branch", "release").reading, ok("25"));
    let unkeyed = reads
        .iter()
        .find(|r| r.dims.is_empty())
        .expect("the record with a null target_branch counts in the series with no key");
    assert_eq!(unkeyed.reading, ok("1"));
    for read in &reads {
        assert!(
            read.dims.keys().all(|key| key == "target_branch"),
            "only a declared key the record carries: {:?}",
            read.dims
        );
    }
}

#[test]
fn git_pipeline_failure_ratio_is_decimal_with_unit_1() {
    let w = World::new(
        "git_pipeline_failure_ratio_is_decimal_with_unit_1",
        "pipelines",
    );
    let definition = failure_ratio(&[]);
    let reads = w.read(&definition);

    // 10 pipelines on one short page; the one created at the window end is outside. 2 of the 9
    // failed: 0.2222…, 18 fractional digits.
    assert_eq!(reads.len(), 1, "{reads:#?}");
    let read = &reads[0];
    assert_eq!(read.reading, ok("0.222222222222222222"));
    assert_eq!(definition.unit, Some(Unit("1".into())));
    assert_eq!(
        read.denominator,
        Some(Population {
            n: 9,
            unit: Unit("{pipeline}".into()),
        }),
        "the population is the record count"
    );
    assert_eq!(read.raw, None);
    assert_eq!(read.query_sha256, query_digest(&definition.query));
    let inputs = w.invoke_inputs();
    assert_eq!(
        pages_asked(&inputs),
        [json!(1)],
        "one short page: {inputs:#?}"
    );
    assert_eq!(inputs[0]["id"], PROJECT);
    assert_eq!(inputs[0]["created_after"], START);
    assert_eq!(inputs[0]["created_before"], END);
    assert_eq!(
        inputs[0]["status"],
        Value::Null,
        "the match is not sent: {inputs:#?}"
    );

    // Per ref, every series is a population the adapter read whole, so a ref without a failure
    // is a ratio of 0 over its own count, not an inference.
    let w = World::new("git_pipeline_failure_ratio_by_ref", "pipelines");
    let reads = w.read(&failure_ratio(&["ref"]));
    assert_eq!(reads.len(), 3, "{reads:#?}");
    let expect = [
        ("main", "0.25", 4),
        ("release", "0.333333333333333333", 3),
        ("feature", "0", 2),
    ];
    for (r, value, n) in expect {
        let read = by(&reads, "ref", r);
        assert_eq!(read.reading, ok(value), "{r}");
        assert_eq!(
            read.denominator,
            Some(Population {
                n,
                unit: Unit("{pipeline}".into()),
            }),
            "{r}"
        );
    }

    // The quotient has unit 1: a ratio definition declaring another unit is not read as one.
    let mut percent = failure_ratio(&[]);
    percent.unit = Some(Unit("%".into()));
    let w = World::new("git_ratio_unit_other_than_1", "pipelines");
    let reads = w.read(&percent);
    assert_eq!(reads.len(), 1);
    assert!(
        reason(&reads[0]).contains("unit"),
        "the reason names the unit: {}",
        reason(&reads[0])
    );
}

#[test]
fn git_unfinished_paging_is_partial_lower_bound() {
    let w = World::new("git_unfinished_paging_is_partial_lower_bound", "cut");
    let reads = w.read(&merged_count(&[]));

    // The first page is full and the second is refused: the window holds at least the 100 read.
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(
        reads[0].reading,
        Reading::Partial(PartialReading {
            value: Decimal("100".into()),
            bound: Bound::Lower,
        })
    );
    assert_eq!(pages_asked(&w.invoke_inputs()), [json!(1), json!(2)]);

    // A cut count of many series is one unknown read naming the cut: the series past it are not
    // read, and the ones before it are not passed off as the whole answer.
    let w = World::new("git_unfinished_paging_by_target_branch", "cut");
    let reads = w.read(&merged_count(&["target_branch"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reads[0].dims.is_empty());
    let why = reason(&reads[0]);
    assert!(
        why.contains("page 2") && why.contains("provider_unavailable"),
        "the reason names the page and the refusal: {why}"
    );

    // A ratio over a cut population has no bound: unknown.
    let w = World::new("git_unfinished_paging_ratio", "cut");
    let mut ratio = definition(
        "git.ratio",
        SignalKind::Ratio,
        &[
            ("project", PROJECT),
            ("records", "merge_requests"),
            ("filter", MERGED),
            ("match", r#"{"target_branch":"release"}"#),
        ],
        &[],
    );
    ratio.denominator = Some(DenominatorSpec {
        unit: Unit("{merge_request}".into()),
        min_n: 5,
        since: "first_sample".into(),
    });
    let reads = w.read(&ratio);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("page 2"), "{reads:#?}");
    assert_eq!(reads[0].denominator, None);
    assert_no_zero(&reads);
}

#[test]
fn git_refused_read_is_unknown_with_reason() {
    let w = World::new("git_refused_read_is_unknown_with_reason", "refused");
    let definition = merged_count(&["target_branch"]);
    let reads = w.read(&definition);

    assert_eq!(reads.len(), 1, "{reads:#?}");
    let why = reason(&reads[0]);
    assert!(
        why.contains("forbidden") && why.contains("dispatch"),
        "the reason carries Connectors' code and stage: {why}"
    );
    assert!(
        why.contains("merge_requests.list"),
        "the reason names the call: {why}"
    );
    assert!(reads[0].dims.is_empty());
    assert_eq!(reads[0].raw, None);
    assert_eq!(reads[0].denominator, None);
    assert_eq!(reads[0].query_sha256, query_digest(&definition.query));
    assert_eq!(pages_asked(&w.invoke_inputs()), [json!(1)]);
}

/// An extractor, a query map, and a word the reason must name.
type Unreadable = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
);

#[test]
fn git_definition_it_cannot_read_is_unknown_before_any_call() {
    let cases: [Unreadable; 9] = [
        ("git.count", &[("records", "merge_requests")], "project"),
        ("git.count", &[("project", PROJECT)], "records"),
        (
            "git.count",
            &[("project", PROJECT), ("records", "issues")],
            "issues",
        ),
        (
            "git.count",
            &[
                ("project", PROJECT),
                ("records", "merge_requests"),
                ("time", "closed_at"),
            ],
            "closed_at",
        ),
        (
            "git.count",
            &[
                ("project", PROJECT),
                ("records", "merge_requests"),
                ("filter", r#"{"page":3}"#),
            ],
            "page",
        ),
        (
            "git.count",
            &[
                ("project", PROJECT),
                ("records", "merge_requests"),
                ("filter", "state=merged"),
            ],
            "filter",
        ),
        (
            "git.count",
            &[
                ("project", PROJECT),
                ("records", "merge_requests"),
                ("match", r#"["merged"]"#),
            ],
            "match",
        ),
        (
            "git.ratio",
            &[("project", PROJECT), ("records", "pipelines")],
            "match",
        ),
        (
            "git.commits",
            &[("project", PROJECT), ("records", "commits")],
            "git.commits",
        ),
    ];
    for (extractor, query, named) in cases {
        let w = World::new("git_definition_it_cannot_read", "refused");
        let reads = w.read(&definition(extractor, SignalKind::CountOfCases, query, &[]));
        assert_eq!(reads.len(), 1, "{extractor} {query:?}: {reads:#?}");
        let why = reason(&reads[0]);
        assert!(why.contains(named), "{extractor} {query:?}: {why}");
        assert_ne!(why, registry::NOT_IMPLEMENTED);
        assert!(
            w.invoke_inputs().is_empty(),
            "{extractor} {query:?}: nothing is invoked"
        );
    }
}

#[test]
fn git_match_on_a_field_the_records_lack_is_unknown() {
    // A predicate over a field no record carries looks the same as a renamed field: no record is
    // counted as a non-match, and no ratio of 0 is read.
    let w = World::new(
        "git_match_on_a_field_the_records_lack_is_unknown",
        "pipelines",
    );
    let mut definition = failure_ratio(&[]);
    definition.query = BTreeMap::from([
        ("project".to_string(), QueryText(PROJECT.into())),
        ("records".to_string(), QueryText("pipelines".into())),
        (
            "match".to_string(),
            QueryText(r#"{"result":"failed"}"#.into()),
        ),
    ]);
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("result"), "{reads:#?}");
    assert_no_zero(&reads);
}
