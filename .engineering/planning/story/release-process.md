---
format: aep.planning-md/3
id: story:release-process
kind: story
status: implemented
title: Each release is a tag v<version>, a CHANGELOG section and a GitHub Release with the built binary
relations:
- decomposes: epic:specification
scope:
- confidence: inferred
  path: .github/workflows/release.yml
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: README.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/thalamus-docs/src/main.rs
- confidence: inferred
  path: crates/thalamus-docs/src/release.rs
- confidence: inferred
  path: crates/thalamus-docs/tests/release_check.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T21:40:00Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-16T21:40:00Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-17T01:21:54Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Context

Thalamus has no release: no tag, no CHANGELOG, no release workflow, and `AGENTS.md` names no release step. Work that is merged and green on `main` is shipped as a release, roughly daily, batching what finished since the last one. Version `0.1.0` is the workspace version (`Cargo.toml`, `[workspace.package]`) and becomes the first release. The pattern is the sibling repositories': a tag `v<version>` equal to the workspace version, a GitHub Release on it, and a workflow that builds and attaches the binary when the Release is published.

The ESS specification under `spec/` models the signal domain (definitions, samples, evaluations, assertions); a release is repository tooling and has no noun there. No specification change.

## Work

- `CHANGELOG.md` (new): `## Unreleased` at the top, then one section per release, `## <version> — <YYYY-MM-DD>`. The first section, `## 0.1.0`, lists what `main` holds today by capability (the shipped items of `crates/thalamus-docs/src/status.rs`), no commit list.
- `crates/thalamus-docs`: a subcommand `thalamus-docs release-check --tag <tag>` (clap derive) that fails, naming the cause, when the tag is not `v<version>` of the workspace version, or when `CHANGELOG.md` has no `## <version>` section, or that section is empty. `task release-check TAG=<tag>` runs it.
- `.github/workflows/release.yml` (new): on `release: published`, and `workflow_dispatch` for a dry build. Runs `release-check` on the tag before anything else, refuses a prerelease and a tag whose commit is not on `origin/main`, builds `cargo build --release --locked -p thalamus-cli`, checks that `thalamus --version` prints `thalamus <version>`, and attaches `thalamus-<version>-x86_64-unknown-linux-gnu.tar.gz` (binary, `LICENSE`, `README.md`, reproducible tar flags) and `SHA256SUMS`. Only the upload job holds `contents: write`. Actions pinned by commit SHA at the pins `.github/workflows/check.yml` already uses.
- `AGENTS.md`: a section "Cutting a release" with the steps: bump `version` in `Cargo.toml` and `Cargo.lock` when the release changes it, turn `## Unreleased` into the dated version section, merge, then create the Release on the merged `main` commit through the bot route; verify tag, checks, release and assets. A release goes out when `main` is green with merged work, about daily. `README.md` links the CHANGELOG.
- Every later change that a user sees adds a line under `## Unreleased` in the same change (rule in `AGENTS.md`).

## Scope

- `CHANGELOG.md` (new), `.github/workflows/release.yml` (new)
- `crates/thalamus-docs/src/main.rs`, `crates/thalamus-docs/src/release.rs` (new), `crates/thalamus-docs/tests/release_check.rs` (new)
- `Taskfile.yml` (task `release-check`), `AGENTS.md`, `README.md`

## Acceptance

`cargo test -p thalamus-docs --test release_check` passes the failing-first tests `release_check_accepts_the_workspace_version_tag_with_a_changelog_section`, `release_check_refuses_a_tag_that_is_not_the_workspace_version`, `release_check_refuses_a_version_without_a_changelog_section`, `release_check_refuses_an_empty_changelog_section` and `release_workflow_runs_release_check_before_building` (reads `.github/workflows/release.yml`). After the merge, the GitHub Release `v0.1.0` exists on a commit of `main` with the tarball and `SHA256SUMS` attached by the workflow.
