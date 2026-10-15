//! The status data, `website/data/status.json` (`b10x-status/1`), from [`CAPABILITIES`].
//!
//! `shipped` means code on `main`. A shipped capability names what holds it: a test as
//! `path::function`, or a task of `Taskfile.yml` that the gate runs; generation fails when that
//! test or task does not exist, so a claim cannot outlive what it rests on. A capability that is not
//! shipped names the planning artifact that carries it (`story:<id>` or `epic:<id>`), which must
//! exist in `.engineering/planning/`; generation reports one whose artifact is already
//! `implemented`, because the item is then due to move to `shipped`.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The directory the status data lands in, relative to the repository root.
pub const DIR: &str = "website/data";
/// The status data's file name under [`DIR`].
pub const FILE: &str = "status.json";

/// The date [`CAPABILITIES`] was last checked against `main` and the planning store.
pub const AS_OF: &str = "2026-10-15";

/// What holds a shipped capability.
#[derive(Clone, Copy)]
pub enum Evidence {
    /// A test, written `path::function` relative to the repository root.
    Test(&'static str),
    /// A task of `Taskfile.yml` that the gate runs.
    Task(&'static str),
}

/// One capability on the status page.
pub struct Capability {
    pub area: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    /// `shipped`, `decided` or `planned`.
    pub status: &'static str,
    /// For a shipped capability, what holds it.
    pub evidence: Option<Evidence>,
    /// For a capability that is not shipped, the planning artifact that carries it.
    pub artifact: Option<&'static str>,
    pub href: &'static str,
}

const fn shipped(
    area: &'static str,
    label: &'static str,
    detail: &'static str,
    evidence: Evidence,
    href: &'static str,
) -> Capability {
    Capability {
        area,
        label,
        detail,
        status: "shipped",
        evidence: Some(evidence),
        artifact: None,
        href,
    }
}

const fn planned(
    area: &'static str,
    label: &'static str,
    detail: &'static str,
    artifact: &'static str,
    href: &'static str,
) -> Capability {
    Capability {
        area,
        label,
        detail,
        status: "planned",
        evidence: None,
        artifact: Some(artifact),
        href,
    }
}

/// Every capability the site claims, in the order the status table lists them.
pub const CAPABILITIES: &[Capability] = &[
    shipped(
        "Specification",
        "The five nouns are specified in ESS",
        "Five domains under spec/: signal definitions, dimensions, samples, evaluations and assertions. task check validates them.",
        Evidence::Task("check"),
        "/docs/reference/ess",
    ),
    shipped(
        "Specification",
        "The Rust model is generated from the specification",
        "generated/thalamus-model and generated/spec-types come from spec/ through ess; task check fails when they drift.",
        Evidence::Task("drift"),
        "/docs/reference/ess",
    ),
    shipped(
        "Command line",
        "A usage error exits 64, never a verdict code",
        "The thalamus binary parses its six subcommands (sample, eval, show, compare, rollup, flaps). Each still exits 70, not implemented.",
        Evidence::Test("crates/thalamus-cli/tests/cli.rs::an_unknown_flag_exits_64"),
        "/docs/getting-started",
    ),
    shipped(
        "Specification",
        "Definition and dimension scenarios pass",
        "A conformance harness runs every synthesised scenario of the definition and dimension domains.",
        Evidence::Test(
            "crates/thalamus-core/tests/conformance.rs::every_scenario_of_the_suite_holds",
        ),
        "/docs/reference/ess",
    ),
    shipped(
        "Specification",
        "Every synthesised conformance scenario passes",
        "Samples, evaluations and assertions join the harness under task check.",
        Evidence::Test(
            "crates/thalamus-core/tests/conformance.rs::every_scenario_of_the_suite_holds",
        ),
        "/docs/reference/ess",
    ),
    shipped(
        "Specification",
        "An unknown reading carries no value; a unit is UCUM",
        "Tests against both generated crates hold the tagged reading, and a validator holds the unit grammar ESS cannot state.",
        Evidence::Test("crates/thalamus-core/tests/probes.rs::unknown_reading_carries_no_value"),
        "/docs/concepts/samples",
    ),
    shipped(
        "Project",
        "CI runs task check",
        "On every push and every pull request, through .github/workflows/check.yml.",
        Evidence::Task("check"),
        "/docs/getting-started",
    ),
    shipped(
        "Grading",
        "One verdict, score and reason per reading",
        "thalamus-core grades a reading against a baseline onto the ladder; an unknown reading is never OK.",
        Evidence::Test(
            "crates/thalamus-core/tests/grading.rs::unknown_reading_is_unknown_never_ok",
        ),
        "/docs/concepts/verdicts",
    ),
    shipped(
        "Grading",
        "Offset, pooled, higher-of, lower-of and fixed baselines",
        "A baseline older than its series is UNKNOWN.",
        Evidence::Test(
            "crates/thalamus-core/tests/baselines.rs::baseline_older_than_series_is_unknown",
        ),
        "/docs/concepts/baselines",
    ),
    planned(
        "Grading",
        "Same-slot median and rolling median baselines",
        "Computed from stored samples.",
        "story:m2-store-baselines",
        "/docs/concepts/baselines",
    ),
    planned(
        "Grading",
        "Persistence settles verdict changes",
        "A change counts after k agreeing evaluations (k_crit for CRIT); WARN clears after m OK evaluations.",
        "story:m2-hysteresis-core",
        "/docs/concepts/verdicts#persistence",
    ),
    planned(
        "Grading",
        "Other producers' vocabularies map onto the ladder",
        "Colour ladders, detector exits and blocking gate exits.",
        "story:m3-vocabulary-mapping",
        "/docs/concepts/verdicts#other-vocabularies",
    ),
    planned(
        "Grading",
        "An advisory gate grades WARN, never CRIT",
        "Its finding count against a fixed baseline.",
        "story:m3-advisory-gate-mapping",
        "/docs/concepts/verdicts#other-vocabularies",
    ),
    shipped(
        "Instances",
        "Definitions and the dimension registry load and validate",
        "Only Active definitions whose dimensions are registered are sampled.",
        Evidence::Test(
            "crates/thalamus-core/tests/registry.rs::sample_of_non_active_definition_is_refused",
        ),
        "/docs/concepts/signals",
    ),
    shipped(
        "Instances",
        "Samples and evaluations persist in Eventlog",
        "One append-only stream per instance, on the File or SQLite provider.",
        Evidence::Test(
            "crates/thalamus-store/tests/store.rs::reopened_store_reads_series_in_ts_order",
        ),
        "/docs/concepts/samples#storage",
    ),
    planned(
        "Instances",
        "An expired mute ends by itself",
        "The service samples the definition as Active again and records the unmute.",
        "story:m2-auto-unmute",
        "/docs/concepts/signals#lifecycle",
    ),
    shipped(
        "Sources",
        "One read-only Connectors client for every source",
        "With the adapter trait every source implements and a recorded double for tests.",
        Evidence::Test(
            "crates/thalamus-sources/tests/connectors_client.rs::write_operation_is_refused_before_invocation",
        ),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "Prometheus",
        "PromQL reads become typed readings; a ratio is a decimal with unit 1, NaN is unknown.",
        Evidence::Test(
            "crates/thalamus-sources/tests/prometheus.rs::an_empty_numerator_answer_infers_no_zero",
        ),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "Loki",
        "LogQL metric queries.",
        Evidence::Test(
            "crates/thalamus-sources/tests/loki.rs::loki_nan_value_is_unknown_never_zero",
        ),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "SQL",
        "NULL and unparseable cells are unknown, never 0.",
        Evidence::Test("crates/thalamus-sources/tests/sql.rs::sql_null_cell_is_unknown_never_zero"),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "Kubernetes",
        "Object counts as instant samples with source-only dimensions.",
        Evidence::Test(
            "crates/thalamus-sources/tests/kubernetes.rs::kubernetes_definition_it_cannot_read_is_unknown_before_any_call",
        ),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "Git hosting",
        "GitLab-hosted repository records as count and ratio samples; an unfinished page walk is a lower bound.",
        Evidence::Test(
            "crates/thalamus-sources/tests/git.rs::git_unfinished_paging_is_partial_lower_bound",
        ),
        "/docs/concepts/signals#sources",
    ),
    shipped(
        "Sources",
        "HTTP probe",
        "The availability of one Connectors read operation.",
        Evidence::Test(
            "crates/thalamus-sources/tests/http_probe.rs::probe_provider_unavailable_is_up_0_with_reason",
        ),
        "/docs/concepts/signals#sources",
    ),
    planned(
        "Sources",
        "One vocabulary across every source",
        "A pass over every source kind yields only ladder verdicts and exits with the worst one.",
        "story:m3-one-vocabulary-across-sources",
        "/docs/concepts/verdicts",
    ),
    planned(
        "Command line",
        "thalamus sample",
        "One sample per series of every Active definition of an instance.",
        "story:m1-cli-sample",
        "/docs/concepts/samples",
    ),
    planned(
        "Command line",
        "thalamus eval and thalamus show",
        "eval grades stored samples once per definition revision; show exits with the worst current verdict.",
        "story:m1-cli-eval-show",
        "/docs/concepts/verdicts",
    ),
    planned(
        "Command line",
        "thalamus compare --vs <offset>",
        "Answers from stored samples.",
        "story:m2-compare",
        "/docs/concepts/baselines",
    ),
    planned(
        "Command line",
        "thalamus rollup",
        "One rollup per series per UTC day.",
        "story:m2-rollup",
        "/docs/concepts/samples#storage",
    ),
    planned(
        "Command line",
        "thalamus flaps",
        "Measures flap rates over stored evaluations before k and m are fixed.",
        "story:m2-flap-measurement",
        "/docs/concepts/verdicts#persistence",
    ),
    planned(
        "Service",
        "Each Active definition is sampled on its cadence",
        "The scheduler in thalamus-service owns sampling; a missed slot is an unknown sample.",
        "story:m2-scheduler",
        "/docs/concepts/samples",
    ),
    planned(
        "Service",
        "Settled transitions as CloudEvents",
        "A verdict change that persisted is recorded and emitted.",
        "story:m2-transitions",
        "/docs/concepts/interfaces#events",
    ),
    planned(
        "Views",
        "Heatmap, sparklines, week vs week, before/after a deploy, live",
        "Served by thalamus-service, with dashboard and alert-rule export for Prometheus and Loki definitions.",
        "epic:views-and-exports",
        "/docs/concepts/interfaces#views",
    ),
    planned(
        "Integrations",
        "thalamus assert",
        "A release, a merge request, a test run or a controller asks whether named signals stayed within baseline.",
        "epic:assertions-and-integrations",
        "/docs/concepts/interfaces#assertions",
    ),
    planned(
        "Integrations",
        "Transitions and rollups reach cortex as facts",
        "Each fact cites the evaluation that produced it; cortex never receives raw samples.",
        "epic:assertions-and-integrations",
        "/docs/concepts/interfaces#cortex",
    ),
    planned(
        "Integrations",
        "Test runs, both directions",
        "Finished cases become samples; a release-day run carries signal assertions.",
        "epic:assertions-and-integrations",
        "/docs/concepts/interfaces#test-runs",
    ),
];

/// Whether `source` defines a function called `name`.
fn defines(source: &str, name: &str) -> bool {
    source.contains(&format!("fn {name}("))
}

fn check_evidence(root: &Path, label: &str, evidence: Evidence) -> Result<(), String> {
    match evidence {
        Evidence::Test(reference) => {
            let (file, function) = reference
                .rsplit_once("::")
                .ok_or_else(|| format!("{label}: evidence {reference} is not path::function"))?;
            let source = fs::read_to_string(root.join(file))
                .map_err(|e| format!("{label}: evidence file {file}: {e}"))?;
            if !defines(&source, function) {
                return Err(format!("{label}: {file} defines no fn {function}"));
            }
        }
        Evidence::Task(task) => {
            let taskfile = fs::read_to_string(root.join("Taskfile.yml"))
                .map_err(|e| format!("{label}: Taskfile.yml: {e}"))?;
            if !taskfile.lines().any(|line| line == format!("  {task}:")) {
                return Err(format!("{label}: Taskfile.yml has no task {task}"));
            }
        }
    }
    Ok(())
}

/// The `status:` of a planning artifact, read from its front matter.
fn artifact_status(root: &Path, label: &str, artifact: &str) -> Result<String, String> {
    let (kind, name) = artifact
        .split_once(':')
        .filter(|(kind, _)| matches!(*kind, "story" | "epic"))
        .ok_or_else(|| format!("{label}: artifact {artifact} is not story:<id> or epic:<id>"))?;
    let path = root.join(format!(".engineering/planning/{kind}/{name}.md"));
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("{label}: planning artifact {artifact}: {e}"))?;
    text.lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .find_map(|line| line.strip_prefix("status:"))
        .map(|status| status.trim().to_owned())
        .ok_or_else(|| format!("{label}: {artifact} has no status"))
}

/// The status document for `capabilities` in the repository at `root`, and any warnings.
pub fn document_for(
    root: &Path,
    capabilities: &[Capability],
    as_of: &str,
) -> Result<(String, Vec<String>), String> {
    let mut items = Vec::new();
    let mut warnings = Vec::new();
    for capability in capabilities {
        let label = capability.label;
        let detail = match (capability.status, capability.evidence, capability.artifact) {
            ("shipped", Some(evidence), _) => {
                check_evidence(root, label, evidence)?;
                capability.detail.to_owned()
            }
            ("shipped", None, _) => {
                return Err(format!("{label}: a shipped capability names its evidence"));
            }
            ("decided" | "planned", Some(_), _) => {
                return Err(format!("{label}: only a shipped capability names evidence"));
            }
            ("decided" | "planned", None, Some(artifact)) => {
                if artifact_status(root, label, artifact)? == "implemented" {
                    warnings.push(format!(
                        "{label}: {artifact} is implemented; move the item to shipped"
                    ));
                }
                format!("{} Planned in {artifact}.", capability.detail)
            }
            ("decided" | "planned", None, None) => {
                return Err(format!(
                    "{label}: a {} capability names its planning artifact",
                    capability.status
                ));
            }
            (other, _, _) => {
                return Err(format!(
                    "{label}: status {other} is not shipped, decided or planned"
                ));
            }
        };
        let mut item = json!({
            "area": capability.area,
            "label": label,
            "detail": detail,
            "status": capability.status,
        });
        if !capability.href.is_empty() {
            item["href"] = Value::from(capability.href);
        }
        items.push(item);
    }
    let document = json!({
        "format": "b10x-status/1",
        "asOf": as_of,
        "source": "generated by thalamus-docs from its capability list, checked against main and the planning store",
        "items": items,
    });
    let text = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    Ok((text + "\n", warnings))
}

/// The status document of this repository.
pub fn document(root: &Path) -> Result<(String, Vec<String>), String> {
    document_for(root, CAPABILITIES, AS_OF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "thalamus-docs-status-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("crates/x/tests")).unwrap();
        fs::create_dir_all(dir.join(".engineering/planning/story")).unwrap();
        fs::create_dir_all(dir.join(".engineering/planning/epic")).unwrap();
        fs::write(
            dir.join("crates/x/tests/cli.rs"),
            "#[test]\nfn an_unknown_flag_exits_64() {}\n",
        )
        .unwrap();
        fs::write(
            dir.join("Taskfile.yml"),
            "version: '3'\ntasks:\n  drift:\n    cmds: []\n",
        )
        .unwrap();
        fs::write(
            dir.join(".engineering/planning/story/m1-a.md"),
            "---\nid: story:m1-a\nkind: story\nstatus: draft\n---\n",
        )
        .unwrap();
        fs::write(
            dir.join(".engineering/planning/story/m1-done.md"),
            "---\nid: story:m1-done\nkind: story\nstatus: implemented\n---\n",
        )
        .unwrap();
        fs::write(
            dir.join(".engineering/planning/epic/m4.md"),
            "---\nid: epic:m4\nkind: epic\nstatus: draft\n---\n",
        )
        .unwrap();
        dir
    }

    fn item(
        status: &'static str,
        evidence: Option<Evidence>,
        artifact: Option<&'static str>,
    ) -> Capability {
        Capability {
            area: "Area",
            label: "Label",
            detail: "Detail.",
            status,
            evidence,
            artifact,
            href: "/docs/x",
        }
    }

    #[test]
    fn shipped_items_need_a_test_or_task_that_exists() {
        let root = repo("shipped");
        let test = Evidence::Test("crates/x/tests/cli.rs::an_unknown_flag_exits_64");
        assert!(document_for(&root, &[item("shipped", Some(test), None)], "2026-10-15").is_ok());
        assert!(
            document_for(
                &root,
                &[item("shipped", Some(Evidence::Task("drift")), None)],
                "2026-10-15"
            )
            .is_ok()
        );
        for missing in [
            Evidence::Test("crates/x/tests/cli.rs::gone"),
            Evidence::Test("crates/x/tests/nowhere.rs::an_unknown_flag_exits_64"),
            Evidence::Test("crates/x/tests/cli.rs"),
            Evidence::Task("website"),
        ] {
            assert!(
                document_for(&root, &[item("shipped", Some(missing), None)], "2026-10-15").is_err()
            );
        }
        assert!(document_for(&root, &[item("shipped", None, None)], "2026-10-15").is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn planned_items_need_a_planning_artifact_that_exists_and_carry_no_evidence() {
        let root = repo("planned");
        assert!(
            document_for(
                &root,
                &[item("planned", None, Some("story:m1-a"))],
                "2026-10-15"
            )
            .is_ok()
        );
        assert!(
            document_for(
                &root,
                &[item("planned", None, Some("epic:m4"))],
                "2026-10-15"
            )
            .is_ok()
        );
        assert!(
            document_for(
                &root,
                &[item("planned", None, Some("story:gone"))],
                "2026-10-15"
            )
            .is_err()
        );
        assert!(
            document_for(
                &root,
                &[item("planned", None, Some("task:m1-a"))],
                "2026-10-15"
            )
            .is_err()
        );
        assert!(document_for(&root, &[item("planned", None, None)], "2026-10-15").is_err());
        let test = Evidence::Test("crates/x/tests/cli.rs::an_unknown_flag_exits_64");
        assert!(
            document_for(
                &root,
                &[item("planned", Some(test), Some("story:m1-a"))],
                "2026-10-15"
            )
            .is_err()
        );
        assert!(
            document_for(
                &root,
                &[item("done", None, Some("story:m1-a"))],
                "2026-10-15"
            )
            .is_err()
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_planned_item_whose_story_is_implemented_is_reported() {
        let root = repo("implemented");
        let (_, warnings) = document_for(
            &root,
            &[item("planned", None, Some("story:m1-done"))],
            "2026-10-15",
        )
        .unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("story:m1-done"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_document_is_b10x_status_and_names_the_artifact_of_a_planned_item() {
        let root = repo("document");
        let test = Evidence::Test("crates/x/tests/cli.rs::an_unknown_flag_exits_64");
        let (text, _) = document_for(
            &root,
            &[
                item("shipped", Some(test), None),
                item("planned", None, Some("story:m1-a")),
            ],
            "2026-10-15",
        )
        .unwrap();
        assert!(text.ends_with('\n'));
        let document: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(document["format"], "b10x-status/1");
        assert_eq!(document["asOf"], "2026-10-15");
        assert_eq!(document["items"][0]["status"], "shipped");
        assert_eq!(document["items"][0]["detail"], "Detail.");
        assert_eq!(document["items"][0]["area"], "Area");
        assert_eq!(document["items"][0]["href"], "/docs/x");
        assert_eq!(
            document["items"][1]["detail"],
            "Detail. Planned in story:m1-a."
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn every_capability_of_this_repository_checks() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (_, warnings) = document_for(&root, CAPABILITIES, "2026-10-15").unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(CAPABILITIES.iter().any(|c| c.status == "shipped"));
    }
}
