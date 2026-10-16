//! `release-check`: a release is the tag `v<version>` of the workspace version and a non-empty
//! `## <version>` section of `CHANGELOG.md`.
use std::{fs, path::Path};

use crate::{Result, io};

/// `version` of `[workspace.package]` in the text of a workspace `Cargo.toml`.
fn workspace_version(manifest: &str) -> Result<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[workspace.package]";
        } else if in_package
            && let Some(rest) = line.strip_prefix("version")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            let value = value.trim().trim_matches('"');
            if !value.is_empty() {
                return Ok(value.to_owned());
            }
        }
    }
    Err("Cargo.toml has no [workspace.package] version".into())
}

/// The body of the `## <version>` section of `changelog`, or `None` when there is no such heading.
fn section(changelog: &str, version: &str) -> Option<String> {
    let heading = format!("## {version}");
    let mut lines = changelog.lines();
    lines.find(|line| {
        line.strip_prefix(&heading)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
    })?;
    let body: Vec<&str> = lines.take_while(|line| !line.starts_with("## ")).collect();
    Some(body.join("\n"))
}

/// Check `tag` against the repository at `root`.
pub fn check(root: &Path, tag: &str) -> Result<()> {
    let manifest_path = root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(io(manifest_path.display()))?;
    let version = workspace_version(&manifest)?;
    if tag != format!("v{version}") {
        return Err(format!(
            "the tag is {tag}, but the workspace version is {version} (expected v{version})"
        ));
    }
    let path = root.join("CHANGELOG.md");
    let changelog = fs::read_to_string(&path).map_err(io(path.display()))?;
    match section(&changelog, &version) {
        None => Err(format!("CHANGELOG.md has no `## {version}` section")),
        Some(body) if body.trim().is_empty() => Err(format!(
            "the `## {version}` section of CHANGELOG.md is empty"
        )),
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workspace_version_is_read_from_workspace_package() {
        let manifest = "[workspace]\nmembers = []\n\n[dependencies]\nversion = \"9\"\n\n[workspace.package]\nversion = \"1.2.3\"\n";
        assert_eq!(workspace_version(manifest).unwrap(), "1.2.3");
        assert!(workspace_version("[package]\nversion = \"1\"\n").is_err());
    }

    #[test]
    fn a_section_ends_at_the_next_heading_and_matches_the_whole_version() {
        let text = "## Unreleased\n\n## 0.10.0 \u{2014} d\n\n- a\n\n## 0.1.0\n- b\n";
        assert_eq!(section(text, "0.1.0").unwrap().trim(), "- b");
        assert_eq!(section(text, "0.10.0").unwrap().trim(), "- a");
        assert!(section(text, "0.1").is_none());
    }
}
