//! Adversary pass 2 for `story:m1-connectors-client`: the double's new refusals held to the real
//! `connectors` CLI.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// Recordings from `common`, then from each scenario, in a fresh directory.
fn world(name: &str, scenarios: &[&str]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv2_connectors_client")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear the test directory");
    }
    std::fs::create_dir_all(&dir).expect("create the test directory");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/connectors");
    for scenario in std::iter::once(&"common").chain(scenarios) {
        for entry in std::fs::read_dir(fixtures.join(scenario)).expect("scenario exists") {
            let path = entry.expect("fixture entry").path();
            std::fs::copy(&path, dir.join(path.file_name().expect("file name")))
                .expect("copy a recording");
        }
    }
    dir
}

/// Runs the double directly with `args`, reading recordings from `dir`.
fn double(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_connectors-double"))
        .args(args)
        .env("CONNECTORS_DOUBLE_FIXTURES", dir)
        .env("CONNECTORS_DOUBLE_LOG", dir.join("calls.jsonl"))
        .output()
        .expect("run the double")
}

fn invoke(dir: &Path, operation: &str, revision: &str) -> Output {
    let input = json!({"query": "up"}).to_string();
    double(
        dir,
        &[
            "operations",
            "invoke",
            "--adapter",
            "metrics",
            "--connection",
            "metrics-main",
            "--operation",
            operation,
            "--schema",
            "series.query/v1alpha1",
            "--revision",
            revision,
            "--input-json",
            &input,
            "--output",
            "json",
        ],
    )
}

/// The real CLI maps `StaleDescription | DescriptionUnavailable` to stage `observation`, next
/// action `refresh_description` (connectors `apps/connectors/src/local.rs:178`). The double's
/// refusal must carry the same stage, or a test of an adapter's reason passes on a stage the real
/// CLI never answers.
#[test]
fn adv2_double_description_refusals_are_at_observation() {
    let dir = world("adv2_double_description_refusals_are_at_observation", &[]);
    for (operation, revision, code) in [
        ("series.query", "op-rev-stale", "stale_description"),
        ("series.other", "op-rev-1", "description_unavailable"),
    ] {
        let out = invoke(&dir, operation, revision);
        let refusal: Value = serde_json::from_slice(&out.stderr).unwrap_or_else(|e| {
            panic!(
                "{operation}: a JSON refusal on stderr: {e}: {}",
                String::from_utf8_lossy(&out.stderr)
            )
        });
        let data = &refusal["error"]["data"];
        assert_eq!(data["code"], code, "{refusal}");
        assert_eq!(
            (data["stage"].as_str(), data["next_action"].as_str()),
            (Some("observation"), Some("refresh_description")),
            "{refusal}"
        );
    }
}

/// The real CLI admits an invoke only at the description's current revision. Once the double's
/// recorded description has moved to `op-rev-2`, an invoke at `op-rev-1` is stale; the double
/// admits it because it accepts any revision any recorded answer ever named.
#[test]
fn adv2_double_admits_only_the_current_description() {
    let dir = world(
        "adv2_double_admits_only_the_current_description",
        &["adv2-described-twice"],
    );
    let describe = [
        "operations",
        "describe",
        "--adapter",
        "metrics",
        "--operation",
        "series.query",
        "--output",
        "json",
    ];
    let first = double(&dir, &describe);
    let second = double(&dir, &describe);
    let revision = |out: &Output| {
        serde_json::from_slice::<Value>(&out.stdout).expect("a description")["result"]["revision"]
            .clone()
    };
    assert_eq!(
        (revision(&first), revision(&second)),
        (json!("op-rev-1"), json!("op-rev-2"))
    );

    let out = invoke(&dir, "series.query", "op-rev-1");
    let answered_ok = serde_json::from_slice::<Value>(&out.stdout)
        .map(|doc| doc["ok"] == Value::Bool(true))
        .unwrap_or(false);
    assert!(
        !answered_ok,
        "op-rev-1 is no longer the description: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
