---
format: aep.planning-md/3
id: story:m0-ci
kind: story
status: implemented
title: GitHub Actions runs task check on every push and pull request
relations:
- decomposes: epic:specification
- depends_on: story:m0-workspace
scope:
- confidence: inferred
  path: .github/workflows/check.yml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T03:07:32Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-15T03:07:32Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-15T03:38:20Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Context

`epic:specification` ships "GitHub Actions runs `task check` on every push and pull request"; this story is that line. The gate `task check` (`story:m0-workspace`) runs on GitHub for every push and pull request, as the cortex repository runs its own (`.github/workflows/check.yml` there).

## Work

- `.github/workflows/check.yml`, one job `task check` on `ubuntu-latest`: triggers `pull_request`, `push` to `main`, `workflow_dispatch`; `permissions: contents: read`; concurrency group per ref; every action pinned by commit SHA; `actions/checkout` with `persist-credentials: false`.
- Toolchain: Rust 1.99.0 with `rustfmt` and `clippy`; go-task 3.54.0 verified by SHA-256; `ess` at the version `spec/ess-inputs.yaml` `requires` (0.59.0 today) and `aep`, each downloaded from its GitHub release and verified against that release `SHA256SUMS`; `Swatinem/rust-cache` keyed on the toolchain.
- Steps after setup: `aep plan artifact validate`, then `task check`. Nothing else gates.

Not here: the shared source gates workflow and Gates policy enrollment (see `decision-blocker:m0-github-repository`), release workflows, documentation publishing.

## Domain relations

None. The workflow rests on no relation between entities; it runs `task check`, which validates every `spec/domains/*.yaml` with `--strict-requires`.

## Scope

Planned (inferred):

- `.github/workflows/check.yml`

## Acceptance

The first run of `.github/workflows/check.yml` on `https://github.com/beyond10x/thalamus` concludes `success`, as `gh run list --repo beyond10x/thalamus --workflow check.yml` shows.
