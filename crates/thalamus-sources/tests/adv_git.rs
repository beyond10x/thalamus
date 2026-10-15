//! Adversary cases for the `git` adapter, against the recorded Connectors double.
//!
//! Each case copies `tests/fixtures/git/common` into a fresh directory under the build's temporary
//! directory, as `tests/git.rs` does, and writes the page answers it needs there itself: the page
//! walks under attack (a full page then an empty one, the 50-page budget, a refusal on page 3) are
//! too long to keep as recordings and are built from one record template each.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thalamus_model::definition::{
    AdapterId, Author, ConnectionId, DenominatorSpec, EvaluationSpec, ExtractorId, Family, History,
    Links, Owner, QueryText, RuleId, SignalDefinitionData, SignalId, SignalKind, Source, Unit,
};
use thalamus_model::dimension::DimensionKey;
use thalamus_model::observation::{Bound, CompleteReading, PartialReading, Reading, RunId};
use thalamus_model::primitives::{Decimal, Duration, Timestamp};
use thalamus_sources::adapter::Read;
use thalamus_sources::connectors::Connectors;
use thalamus_sources::registry;

const PROJECT: &str = "group/app";
const END: &str = "2026-10-14T22:57:03Z";
const START: &str = "2026-10-07T22:57:03Z";
const INSIDE: &str = "2026-10-10T12:00:00.000Z";

struct World {
    dir: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adv-git")
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
        for operation in ["tags.list", "commits.list", "deployments.list"] {
            let describe = json!({"stdout": {"ok": true, "result": {
                "adapter": "forge", "operation": operation,
                "schema": format!("{operation}/v1"), "revision": "op-rev-1"}}});
            std::fs::write(
                dir.join(format!("operations+describe+a=forge+o={operation}.json")),
                describe.to_string(),
            )
            .expect("write a describe recording");
        }
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_connectors-double"),
            dir.join("connectors"),
        )
        .expect("link the double");
        World { dir }
    }

    /// Records the answers of `operation`, page `n` answering `pages[n - 1]`.
    fn pages(&self, operation: &str, pages: Vec<Answer>) -> &World {
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

    /// The `--input-json` of every `operations invoke`, in order.
    fn inputs(&self) -> Vec<Value> {
        let text = std::fs::read_to_string(self.dir.join("calls.jsonl")).unwrap_or_default();
        text.lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("a call is one JSON line"))
            .filter_map(|call| {
                let argv: Vec<String> = call["argv"]
                    .as_array()?
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect();
                if argv.first().map(String::as_str) != Some("operations")
                    || argv.get(1).map(String::as_str) != Some("invoke")
                {
                    return None;
                }
                let at = argv.iter().position(|a| a == "--input-json")?;
                serde_json::from_str(argv.get(at + 1)?).ok()
            })
            .collect()
    }

    fn pages_asked(&self) -> Vec<u64> {
        self.inputs()
            .iter()
            .map(|i| i["page"].as_u64().expect("a page number"))
            .collect()
    }
}

/// One page's answer.
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
                                   "resource": "/api/v4/projects/{id}/records", "source_revision": "0f5662970b38"},
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

/// A merged merge request.
fn mr(id: u64, merged_at: &str, target_branch: &str) -> Value {
    json!({"id": id, "iid": id, "project_id": 7, "state": "merged", "target_branch": target_branch,
           "created_at": "2026-10-01T09:00:00.000Z", "updated_at": merged_at, "merged_at": merged_at})
}

/// `n` merged merge requests inside the window, ids from `from`.
fn mrs(from: u64, n: usize) -> Answer {
    Answer::Records(
        (0..n as u64)
            .map(|i| mr(from + i, INSIDE, "main"))
            .collect(),
    )
}

fn pipeline(id: u64, status: &str, r#ref: &str) -> Value {
    json!({"id": id, "iid": id, "project_id": 7, "status": status, "ref": r#ref,
           "created_at": INSIDE, "updated_at": INSIDE})
}

fn definition(
    extractor: &str,
    query: &[(&str, &str)],
    dimensions: &[&str],
) -> SignalDefinitionData {
    let ratio = extractor == "git.ratio";
    SignalDefinitionData {
        signal_id: SignalId("delivery.project.adversary".into()),
        title: "Adversary".into(),
        family: Family("delivery".into()),
        kind: if ratio {
            SignalKind::Ratio
        } else {
            SignalKind::CountOfCases
        },
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
        denominator: ratio.then(|| DenominatorSpec {
            unit: Unit("{record}".into()),
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

fn merged_count(dimensions: &[&str]) -> SignalDefinitionData {
    definition(
        "git.count",
        &[
            ("project", PROJECT),
            ("records", "merge_requests"),
            ("filter", r#"{"state":"merged"}"#),
        ],
        dimensions,
    )
}

fn failure_ratio(dimensions: &[&str]) -> SignalDefinitionData {
    definition(
        "git.ratio",
        &[
            ("project", PROJECT),
            ("records", "pipelines"),
            ("match", r#"{"status":"failed"}"#),
        ],
        dimensions,
    )
}

fn ok(value: &str) -> Reading {
    Reading::Ok(CompleteReading {
        value: Decimal(value.into()),
    })
}

fn lower(value: &str) -> Reading {
    Reading::Partial(PartialReading {
        value: Decimal(value.into()),
        bound: Bound::Lower,
    })
}

fn reason(read: &Read) -> &str {
    match &read.reading {
        Reading::Unknown(unread) => &unread.reason,
        other => panic!("expected an unknown reading, got {other:?}"),
    }
}

/// Every read is `unknown`: no value, 0 least of all.
fn assert_all_unknown(reads: &[Read]) {
    assert!(
        !reads.is_empty(),
        "a read that could not be made is one unknown read, not none"
    );
    for read in reads {
        assert!(
            matches!(read.reading, Reading::Unknown(_)),
            "expected unknown, got {:?} at {:?}",
            read.reading,
            read.dims
        );
    }
}

// --- Design § 11 D20 -------------------------------------------------------------------------

/// D20: "a complete numerator answer with no series at all over a non-empty denominator reads
/// unknown (a renamed metric or a broken matcher looks the same as zero everywhere)". In the git
/// adapter the numerator is the matching records of the one walk: a `match` whose value no record
/// in the window holds (a typo `faild`, or `7` against a text `"7"`) reads 0 everywhere today.
#[test]
fn adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20() {
    let w = World::new("adv_git_ratio_no_match_d20");
    w.pages(
        "pipelines.list",
        vec![Answer::Records(vec![
            pipeline(1, "success", "main"),
            pipeline(2, "success", "main"),
            pipeline(3, "success", "release"),
            pipeline(4, "canceled", "release"),
            pipeline(5, "success", "feature"),
        ])],
    );
    let reads = w.read(&failure_ratio(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_all_unknown(&reads);

    // The same with a dimension: every series would read 0.
    let w = World::new("adv_git_ratio_no_match_d20_by_ref");
    w.pages(
        "pipelines.list",
        vec![Answer::Records(vec![
            pipeline(1, "success", "main"),
            pipeline(2, "success", "release"),
            pipeline(3, "success", "feature"),
        ])],
    );
    let reads = w.read(&failure_ratio(&["ref"]));
    assert_all_unknown(&reads);
}

// --- Paging boundaries ------------------------------------------------------------------------

/// A full page of exactly 100 followed by an empty page: covered, a complete 100.
#[test]
fn adv_git_full_page_then_empty_page_is_complete() {
    let w = World::new("adv_git_full_then_empty");
    w.pages(
        "merge_requests.list",
        vec![mrs(1, 100), Answer::Records(Vec::new())],
    );
    let reads = w.read(&merged_count(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("100"));
    assert_eq!(w.pages_asked(), [1, 2]);
}

/// A refusal on page 3 after two full pages: at least the 200 read, bound lower.
#[test]
fn adv_git_refusal_on_page_three_is_a_lower_bound() {
    let w = World::new("adv_git_refused_page_three");
    w.pages(
        "merge_requests.list",
        vec![mrs(1, 100), mrs(101, 100), Answer::Refused],
    );
    let reads = w.read(&merged_count(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, lower("200"));
    assert_eq!(w.pages_asked(), [1, 2, 3]);
}

/// The 50-page budget hit exactly: 50 full pages are a cut walk (a 51st is never asked), and a
/// short 50th page is a covered one.
#[test]
fn adv_git_page_budget_hit_exactly() {
    let w = World::new("adv_git_budget_spent");
    w.pages(
        "merge_requests.list",
        (0..51).map(|p| mrs(p * 100 + 1, 100)).collect(),
    );
    let reads = w.read(&merged_count(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, lower("5000"));
    assert_eq!(w.pages_asked(), (1..=50).collect::<Vec<u64>>());

    let w = World::new("adv_git_budget_last_page_short");
    let mut pages: Vec<Answer> = (0..49).map(|p| mrs(p * 100 + 1, 100)).collect();
    pages.push(mrs(4901, 99));
    w.pages("merge_requests.list", pages);
    let reads = w.read(&merged_count(&[]));
    assert_eq!(reads[0].reading, ok("4999"));
    assert_eq!(w.pages_asked().len(), 50);
}

/// A cut walk that counted nothing reads unknown, never a lower bound of 0.
#[test]
fn adv_git_cut_walk_that_counted_nothing_is_unknown() {
    let w = World::new("adv_git_cut_counted_nothing");
    w.pages("merge_requests.list", vec![mrs(1, 100), Answer::Refused]);
    let definition = definition(
        "git.count",
        &[
            ("project", PROJECT),
            ("records", "merge_requests"),
            ("match", r#"{"target_branch":"release"}"#),
        ],
        &[],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("page 2"), "{reads:#?}");
}

/// A record answered twice across pages reads unknown, with or without dimensions.
#[test]
fn adv_git_record_repeated_across_pages_is_unknown() {
    for dimensions in [&[][..], &["target_branch"][..]] {
        let w = World::new("adv_git_repeated_across_pages");
        let mut second = vec![mr(100, INSIDE, "main")];
        second.extend((0..3).map(|i| mr(200 + i, INSIDE, "main")));
        w.pages(
            "merge_requests.list",
            vec![mrs(1, 100), Answer::Records(second)],
        );
        let reads = w.read(&merged_count(dimensions));
        assert_eq!(reads.len(), 1, "{reads:#?}");
        assert!(reason(&reads[0]).contains("twice"), "{reads:#?}");
    }
}

// --- Window boundaries ------------------------------------------------------------------------

/// `[start, end)`, compared as instants: offsets and fractions do not move a record across an
/// edge, and a record GitLab returned outside the window is not counted.
#[test]
fn adv_git_window_edges_with_offsets_and_fractions() {
    let w = World::new("adv_git_window_edges");
    w.pages(
        "merge_requests.list",
        vec![Answer::Records(vec![
            // the start, written at +02:00: inside
            mr(1, "2026-10-08T00:57:03+02:00", "main"),
            // a nanosecond before the start: outside
            mr(2, "2026-10-07T22:57:02.999999999Z", "main"),
            // the end, written at +02:00 with a fraction: outside
            mr(3, "2026-10-15T00:57:03.000+02:00", "main"),
            // a millisecond before the end: inside
            mr(4, "2026-10-14T22:57:02.999Z", "main"),
            // the end at -05:00: outside
            mr(5, "2026-10-14T17:57:03-05:00", "main"),
            // returned by GitLab although far outside: outside
            mr(6, "2026-09-01T00:00:00Z", "main"),
            mr(7, "2026-10-20T00:00:00Z", "main"),
        ])],
    );
    let reads = w.read(&merged_count(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("2"));
}

// --- Dimensions -------------------------------------------------------------------------------

/// A declared dimension holding a number or a boolean is a key; null is no key; a list reads
/// unknown.
#[test]
fn adv_git_dimension_values_of_each_json_type() {
    let w = World::new("adv_git_dims_types");
    let record = |id: u64, project_id: Value, draft: Value, milestone: Value| {
        json!({"id": id, "state": "merged", "merged_at": INSIDE, "project_id": project_id,
               "draft": draft, "milestone": milestone, "labels": ["a", "b"]})
    };
    w.pages(
        "merge_requests.list",
        vec![Answer::Records(vec![
            record(1, json!(7), json!(false), json!(null)),
            record(2, json!(7), json!(false), json!(null)),
            record(3, json!(8), json!(true), json!(null)),
        ])],
    );
    let reads = w.read(&merged_count(&["project_id", "draft", "milestone"]));
    assert_eq!(reads.len(), 2, "{reads:#?}");
    let seven = BTreeMap::from([
        ("draft".to_string(), "false".to_string()),
        ("project_id".to_string(), "7".to_string()),
    ]);
    let eight = BTreeMap::from([
        ("draft".to_string(), "true".to_string()),
        ("project_id".to_string(), "8".to_string()),
    ]);
    let by = |dims: &BTreeMap<String, String>| {
        reads
            .iter()
            .find(|r| &r.dims == dims)
            .unwrap_or_else(|| panic!("no read at {dims:?}: {reads:#?}"))
    };
    assert_eq!(by(&seven).reading, ok("2"));
    assert_eq!(by(&eight).reading, ok("1"));

    let reads = w.read(&merged_count(&["labels"]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("labels"), "{reads:#?}");
}

/// A dimensioned count with no matching record has no series: one unknown, not none and not 0.
#[test]
fn adv_git_dimensioned_count_with_no_match_is_one_unknown() {
    let w = World::new("adv_git_dims_no_match");
    w.pages("merge_requests.list", vec![mrs(1, 3)]);
    let definition = definition(
        "git.count",
        &[
            ("project", PROJECT),
            ("records", "merge_requests"),
            ("match", r#"{"target_branch":"release"}"#),
        ],
        &["target_branch"],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_all_unknown(&reads);
}

// --- Ratios -----------------------------------------------------------------------------------

/// A window holding no record: one unknown, with or without dimensions; never 0, never no read.
#[test]
fn adv_git_ratio_over_an_empty_window_is_one_unknown() {
    for dimensions in [&[][..], &["ref"][..]] {
        let w = World::new("adv_git_ratio_empty");
        w.pages("pipelines.list", vec![Answer::Records(Vec::new())]);
        let reads = w.read(&failure_ratio(dimensions));
        assert_eq!(reads.len(), 1, "{dimensions:?}: {reads:#?}");
        assert_all_unknown(&reads);
    }
}

/// A ratio over records some of which lack the match field: unknown, naming the field.
#[test]
fn adv_git_ratio_with_the_match_field_absent_on_some_records_is_unknown() {
    let w = World::new("adv_git_ratio_match_field_partly_absent");
    let mut lacking = pipeline(4, "failed", "main");
    lacking.as_object_mut().expect("object").remove("status");
    w.pages(
        "pipelines.list",
        vec![Answer::Records(vec![
            pipeline(1, "failed", "main"),
            pipeline(2, "success", "main"),
            pipeline(3, "success", "main"),
            lacking,
        ])],
    );
    let reads = w.read(&failure_ratio(&[]));
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(reason(&reads[0]).contains("status"), "{reads:#?}");
}

// --- Record kinds without a test in tests/git.rs ----------------------------------------------

/// Tags: no window parameter is sent, every page is read whatever order the tags come in, and a
/// tag is placed by its commit's date.
#[test]
fn adv_git_tags_are_walked_whole_and_placed_by_commit_date() {
    let tag = |name: String, at: &str| {
        json!({"name": name, "message": "", "target": "abc", "protected": false,
               "commit": {"id": "abc", "committed_date": at, "created_at": at}})
    };
    // Page 1: 100 tags, oldest first and every one before the window (a walk that stops at the
    // first tag older than the window, assuming newest-first order, would read none of page 2).
    let first: Vec<Value> = (0..100)
        .map(|i| tag(format!("v0.{i}"), "2026-01-01T00:00:00Z"))
        .collect();
    // Page 2: 3 inside, one at the end (outside).
    let second = vec![
        tag("v1.0".into(), "2026-10-08T00:00:00+00:00"),
        tag("v1.1".into(), "2026-10-09T00:00:00.5Z"),
        tag("v1.2".into(), START),
        tag("v1.3".into(), END),
    ];
    let w = World::new("adv_git_tags");
    w.pages(
        "tags.list",
        vec![Answer::Records(first), Answer::Records(second)],
    );
    let definition = definition(
        "git.count",
        &[("project", PROJECT), ("records", "tags")],
        &[],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("3"));
    let inputs = w.inputs();
    assert_eq!(w.pages_asked(), [1, 2]);
    for input in &inputs {
        let keys: Vec<&str> = input
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            ["id", "page", "per_page"],
            "no window parameter: {input}"
        );
    }

    // A tag whose commit is null cannot be placed: unknown.
    let w = World::new("adv_git_tags_null_commit");
    w.pages(
        "tags.list",
        vec![Answer::Records(vec![
            tag("v1.0".into(), INSIDE),
            json!({"name": "v1.1", "commit": null}),
        ])],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert!(
        reason(&reads[0]).contains("commit.committed_date"),
        "{reads:#?}"
    );
}

/// Commits: bounded by `since` and `until`, identified by their text id.
#[test]
fn adv_git_commits_are_bounded_by_since_and_until() {
    let commit = |id: &str, at: &str| json!({"id": id, "short_id": &id[..8], "committed_date": at});
    let w = World::new("adv_git_commits");
    w.pages(
        "commits.list",
        vec![Answer::Records(vec![
            commit("0123456789abcdef0123456789abcdef01234567", INSIDE),
            commit("1123456789abcdef0123456789abcdef01234567", INSIDE),
            commit("2123456789abcdef0123456789abcdef01234567", END),
        ])],
    );
    let definition = definition(
        "git.count",
        &[("project", PROJECT), ("records", "commits")],
        &[],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("2"));
    let inputs = w.inputs();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0]["since"], START);
    assert_eq!(inputs[0]["until"], END);
}

/// Deployments: placed on `updated_at` by default, bounded by `updated_after`/`updated_before`.
#[test]
fn adv_git_deployments_default_to_updated_at() {
    let w = World::new("adv_git_deployments");
    w.pages(
        "deployments.list",
        vec![Answer::Records(vec![
            json!({"id": 1, "status": "success", "updated_at": INSIDE, "finished_at": null}),
            json!({"id": 2, "status": "failed", "updated_at": START, "finished_at": null}),
            json!({"id": 3, "status": "failed", "updated_at": "2026-10-01T00:00:00Z"}),
        ])],
    );
    let definition = definition(
        "git.ratio",
        &[
            ("project", PROJECT),
            ("records", "deployments"),
            ("match", r#"{"status":"failed"}"#),
        ],
        &[],
    );
    let reads = w.read(&definition);
    assert_eq!(reads.len(), 1, "{reads:#?}");
    assert_eq!(reads[0].reading, ok("0.5"));
    let inputs = w.inputs();
    assert_eq!(inputs[0]["updated_after"], START);
    assert_eq!(inputs[0]["updated_before"], END);
}

// --- Query digest -----------------------------------------------------------------------------

/// The digest is SHA-256 of the canonical JSON of the query map, keys sorted: written here as a
/// literal, so it does not lean on the function under test. A different `filter` or `match`
/// changes it.
#[test]
fn adv_git_query_digest_is_canonical_and_sensitive_to_filter_and_match() {
    let canonical =
        r#"{"filter":"{\"state\":\"merged\"}","project":"group/app","records":"merge_requests"}"#;
    let expected: String = Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let digest_of = |name: &str, query: &[(&str, &str)]| {
        let w = World::new(name);
        w.pages("merge_requests.list", vec![mrs(1, 2)]);
        let reads = w.read(&definition("git.count", query, &[]));
        reads[0].query_sha256.0.clone()
    };
    let one = digest_of(
        "adv_git_digest_a",
        &[
            ("records", "merge_requests"),
            ("filter", r#"{"state":"merged"}"#),
            ("project", PROJECT),
        ],
    );
    let two = digest_of(
        "adv_git_digest_b",
        &[
            ("project", PROJECT),
            ("filter", r#"{"state":"merged"}"#),
            ("records", "merge_requests"),
        ],
    );
    assert_eq!(one, expected);
    assert_eq!(two, expected);

    let other_filter = digest_of(
        "adv_git_digest_c",
        &[
            ("project", PROJECT),
            ("filter", r#"{"state":"closed"}"#),
            ("records", "merge_requests"),
        ],
    );
    let with_match = digest_of(
        "adv_git_digest_d",
        &[
            ("project", PROJECT),
            ("filter", r#"{"state":"merged"}"#),
            ("match", r#"{"target_branch":"main"}"#),
            ("records", "merge_requests"),
        ],
    );
    assert_ne!(other_filter, expected);
    assert_ne!(with_match, expected);
    assert_ne!(with_match, other_filter);
}
