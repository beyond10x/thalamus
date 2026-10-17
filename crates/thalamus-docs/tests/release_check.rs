//! `thalamus-docs release-check` against fixture repositories, and the release workflow's order.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn fixture(name: &str, version: &str, changelog: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("thalamus-release-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"{version}\"\nedition = \"2024\"\n"
        ),
    )
    .unwrap();
    if let Some(text) = changelog {
        fs::write(dir.join("CHANGELOG.md"), text).unwrap();
    }
    dir
}

fn release_check(root: &Path, tag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_thalamus-docs"))
        .args(["release-check", "--tag", tag, "--root"])
        .arg(root)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const CHANGELOG: &str = "# Changelog\n\n## Unreleased\n\n- Something later.\n\n## 0.2.0 \u{2014} 2026-10-16\n\n- The sample command.\n\n## 0.1.0 \u{2014} 2026-10-15\n\n- The first release.\n";

#[test]
fn release_check_accepts_the_workspace_version_tag_with_a_changelog_section() {
    let root = fixture("accepts", "0.2.0", Some(CHANGELOG));
    let output = release_check(&root, "v0.2.0");
    assert!(output.status.success(), "{}", stderr(&output));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn release_check_refuses_a_tag_that_is_not_the_workspace_version() {
    let root = fixture("wrong-tag", "0.2.0", Some(CHANGELOG));
    for tag in ["v0.1.0", "0.2.0", "v0.2.0-rc1", "v0.2"] {
        let output = release_check(&root, tag);
        assert!(!output.status.success(), "{tag} was accepted");
        assert!(stderr(&output).contains("v0.2.0"), "{}", stderr(&output));
    }
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn release_check_refuses_a_version_without_a_changelog_section() {
    let root = fixture("no-section", "0.3.0", Some(CHANGELOG));
    let output = release_check(&root, "v0.3.0");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("## 0.3.0"), "{}", stderr(&output));
    let missing = fixture("no-file", "0.3.0", None);
    let output = release_check(&missing, "v0.3.0");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("CHANGELOG.md"));
    let unreleased = fixture(
        "unreleased-only",
        "0.3.0",
        Some("# Changelog\n\n## Unreleased\n\n- Work.\n"),
    );
    assert!(!release_check(&unreleased, "v0.3.0").status.success());
    for dir in [root, missing, unreleased] {
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn release_check_refuses_an_empty_changelog_section() {
    let root = fixture(
        "empty",
        "0.3.0",
        Some(
            "# Changelog\n\n## Unreleased\n\n## 0.3.0 \u{2014} 2026-10-17\n\n\n## 0.2.0 \u{2014} 2026-10-16\n\n- Work.\n",
        ),
    );
    let output = release_check(&root, "v0.3.0");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("empty"), "{}", stderr(&output));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn release_workflow_runs_release_check_before_building() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    let workflow = fs::read_to_string(&path).unwrap();
    let check = workflow
        .find("release-check")
        .expect("the workflow runs release-check");
    let build = workflow
        .find("cargo build --release --locked -p thalamus-cli")
        .expect("the workflow builds the release binary");
    assert!(check < build, "release-check must run before the build");
    let first_step = workflow.find("steps:").unwrap();
    let before_check = &workflow[first_step..check];
    assert!(
        !before_check.contains("cargo build"),
        "nothing builds before release-check"
    );
}
