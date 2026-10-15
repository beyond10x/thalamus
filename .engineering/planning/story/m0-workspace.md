---
format: aep.planning-md/3
id: story:m0-workspace
kind: story
status: active
title: Cargo workspace, generated model and the task check gate
relations:
- decomposes: epic:specification
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/thalamus-cli/Cargo.toml
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/mod.rs
- confidence: inferred
  path: crates/thalamus-cli/src/main.rs
- confidence: inferred
  path: crates/thalamus-core/Cargo.toml
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/mod.rs
- confidence: inferred
  path: crates/thalamus-core/src/lib.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/mod.rs
- confidence: inferred
  path: crates/thalamus-pipeline/Cargo.toml
- confidence: inferred
  path: crates/thalamus-pipeline/src/lib.rs
- confidence: inferred
  path: crates/thalamus-service/Cargo.toml
- confidence: inferred
  path: crates/thalamus-service/src/lib.rs
- confidence: inferred
  path: crates/thalamus-sources/Cargo.toml
- confidence: inferred
  path: crates/thalamus-sources/src/lib.rs
- confidence: inferred
  path: crates/thalamus-store/Cargo.toml
- confidence: inferred
  path: crates/thalamus-store/src/lib.rs
- confidence: inferred
  path: docs/design/thalamus.md
- confidence: inferred
  path: generated/spec-types
- confidence: inferred
  path: generated/thalamus-model
- confidence: inferred
  path: spec/suite.json
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Context

M0 of `docs/design/thalamus.md` § 9. The repository holds the ESS specification under `spec/` (`ess specify validate --path spec`: valid; `ess verify conform synthesize`: 73 scenarios, 0 refusals at ess 0.59.0) and no code. This story lays the Rust workspace around it, following the sibling pattern of the cortex repository: a `Taskfile.yml` gate, generated crates committed under `generated/`, excluded from the workspace and depended on by path.

It also lays the **whole skeleton** of the planned code: every crate as a workspace member and every module file of the ownership table below already declared, so a later story adds code to its own files and never edits the root `Cargo.toml` `members`, a `crates/*/src/lib.rs` module line, a `mod.rs` module line or the CLI's subcommand enum. That is what lets stories in one wave land without touching a shared line.

## Work

- `Taskfile.yml` (go-task):
  - `generate`: `ess specify validate --path spec --strict-requires`; `ess generate synthesize --path spec --target rust --layout crate --out generated/thalamus-model --strict-requires`; `ess generate types --path spec --target rust --all-types --all-events --package thalamus-spec-types --out generated/spec-types --strict-requires`; `ess verify conform synthesize --path spec --out spec/suite.json --strict-requires`.
  - `drift`: regenerate all of the above into a `mktemp -d` directory under `$TMPDIR`, then `diff -r` against `generated/` and `cmp` against `spec/suite.json`; fail on any difference.
  - `check`: spec validity with `--strict-requires`, `drift`, `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Root `Cargo.toml`: a virtual workspace with `members` = the six crates `thalamus-core`, `thalamus-store`, `thalamus-sources`, `thalamus-pipeline`, `thalamus-service`, `thalamus-cli` (all under `crates/`), and `exclude = ["generated/thalamus-model", "generated/spec-types"]` (each generated manifest declares its own `[workspace]`); `[workspace.lints]` forbids `unsafe_code`. `Cargo.lock` committed; every cargo call is `--locked`.
- Each crate's `Cargo.toml` with its path dependencies on the other workspace crates and on the generated crates (`thalamus-model = { package = "thalamus", path = "../../generated/thalamus-model" }`, `thalamus-spec-types`), so the generated code is compiled by `task check` from the first commit. `thalamus-cli` also gets `clap` (derive) and `serde_json`. A later story that needs a third-party dependency adds it to its own crate's `Cargo.toml`; the coordinator regenerates `Cargo.lock` at merge.
- `crates/thalamus-core/src/lib.rs`, `crates/thalamus-core/src/behaviour/mod.rs`, `crates/thalamus-core/src/rules/mod.rs`, `crates/thalamus-store/src/lib.rs`, `crates/thalamus-sources/src/lib.rs`, `crates/thalamus-pipeline/src/lib.rs`, `crates/thalamus-service/src/lib.rs`, `crates/thalamus-cli/src/cmd/mod.rs`: the module lines for every file of the table below, and nothing else. `rules/mod.rs` gets its module lines here and its rule list from `story:m1-verdict-grading`.
- `crates/thalamus-cli/src/main.rs` (package `thalamus-cli`, `[[bin]] name = "thalamus"`): the clap derive root with the global options struct from `crates/thalamus-cli/src/global.rs` flattened in, and the subcommand enum `sample`, `eval`, `show`, `compare`, `rollup`, `flaps`, each variant holding the `Args` struct of its `cmd/<name>.rs` and calling its `run`. A usage error exits 64, never a ladder code (0 to 3, design § 2.3) and never clap's default 2.
- Every module file of the table is created as a stub that compiles: empty, or for `cmd/<name>.rs` an empty `Args` struct and a `run` that prints `not implemented` and exits 70; no `todo!()`. `crates/thalamus-service/src/main.rs` and `crates/thalamus-sources/src/bin/connectors-double.rs` print `not implemented` and exit 70.
- `AGENTS.md`: the command table names `task generate`, `task drift` and `task check`; the line saying `task check` arrives with the first crate goes; the planned-crates line names the six workspace crates and `thalamus-model`.

Decisions taken here (reversible):

- `ess generate synthesize --layout crate` names its package `thalamus`; ess 0.59.0 offers no option to change it. The CLI crate of design § 5.1 therefore becomes package `thalamus-cli` with `[[bin]] name = "thalamus"`, as cortex does (`cortex-cli`, binary `cortex`), and the generated dependency is renamed `thalamus-model`. § 5.1 gets that line and a row for `thalamus-pipeline`, the library holding the read → sample → grade → evaluate pass that the CLI and the service share.
- `ess generate types` uses `--all-types --all-events`: an entity name is not a type root at ess 0.59.0 (`unknown_root`), and the wire records (`thalamus.sample/1`, `thalamus.evaluation/1`, the CloudEvents payloads) span every domain.

## Module ownership

Every file below is created by this story as a stub and filled by exactly one story. A file a second story also changes is named in both bodies with the line it changes, and the two are ordered by `depends_on`.

| story | files it owns (paths under `crates/`) |
|---|---|
| `story:m0-conformance-definitions` | `thalamus-core/src/behaviour/definition.rs`, `thalamus-core/src/behaviour/dimension.rs`, `thalamus-core/tests/conformance.rs` |
| `story:m0-conformance-records` | `thalamus-core/src/behaviour/observation.rs`, `thalamus-core/src/behaviour/evaluation.rs`, `thalamus-core/src/behaviour/assertion.rs` |
| `story:m0-probes` | `thalamus-core/src/unit.rs`, `thalamus-core/tests/probes.rs` |
| `story:m1-verdict-grading` | `thalamus-core/src/ladder.rs`, `thalamus-core/src/grading.rs`, the rule list in `thalamus-core/src/rules/mod.rs`, `thalamus-core/src/rules/threshold.rs` |
| `story:m1-baselines` | `thalamus-core/src/baseline/mod.rs` |
| `story:m1-instance-registry` | `thalamus-core/src/registry.rs`, `thalamus-core/src/series.rs`, `thalamus-core/src/since.rs` |
| `story:m1-eventlog-store` | `thalamus-store/src/eventlog.rs`, `thalamus-store/src/samples.rs`, `thalamus-store/src/evaluations.rs`, `thalamus-store/src/range.rs`, `thalamus-store/tests/conformance.rs` |
| `story:m1-connectors-client` | `thalamus-sources/src/adapter.rs`, `thalamus-sources/src/connectors.rs`, `thalamus-sources/src/registry.rs`, `thalamus-sources/src/bin/connectors-double.rs` |
| `story:m1-prometheus-adapter` | `thalamus-sources/src/prometheus.rs` |
| `story:m1-cli-sample` | `thalamus-pipeline/src/instance.rs`, `thalamus-pipeline/src/sample.rs`, `thalamus-cli/src/global.rs`, `thalamus-cli/src/cmd/sample.rs` |
| `story:m1-cli-eval-show` | `thalamus-pipeline/src/evaluate.rs`, `thalamus-cli/src/cmd/eval.rs`, `thalamus-cli/src/cmd/show.rs` |
| `story:m2-spec-history-records` | `thalamus-core/src/behaviour/transition.rs`, `thalamus-core/src/behaviour/rollup.rs` |
| `story:m2-hysteresis-core` | `thalamus-core/src/hysteresis.rs` |
| `story:m2-store-baselines` | `thalamus-core/src/baseline/same_slot.rs`, `thalamus-core/src/baseline/rolling.rs` |
| `story:m2-scheduler` | `thalamus-service/src/main.rs`, `thalamus-service/src/scheduler.rs` |
| `story:m2-auto-unmute` | `thalamus-service/src/unmute.rs`, `thalamus-store/src/definition_events.rs` |
| `story:m2-rollup` | `thalamus-core/src/rollup.rs`, `thalamus-store/src/rollups.rs`, `thalamus-service/src/daily_rollup.rs`, `thalamus-cli/src/cmd/rollup.rs` |
| `story:m2-transitions` | `thalamus-store/src/transitions.rs`, `thalamus-service/src/transitions.rs`, `thalamus-service/src/events.rs` |
| `story:m2-compare` | `thalamus-cli/src/cmd/compare.rs` |
| `story:m2-flap-measurement` | `thalamus-cli/src/cmd/flaps.rs` |
| `story:m3-vocabulary-mapping` | `thalamus-core/src/rules/mapped.rs`, `thalamus-sources/src/vocabulary.rs` |
| `story:m3-advisory-gate-mapping` | `thalamus-core/src/rules/warn_only.rs` |
| `story:m3-loki-adapter` | `thalamus-sources/src/loki.rs` |
| `story:m3-sql-adapter` | `thalamus-sources/src/sql.rs` |
| `story:m3-kubernetes-adapter` | `thalamus-sources/src/kubernetes.rs` |
| `story:m3-git-adapter` | `thalamus-sources/src/git.rs` |
| `story:m3-http-probe-adapter` | `thalamus-sources/src/http_probe.rs` |
| `story:m3-one-vocabulary-across-sources` | `thalamus-pipeline/tests/every_source_one_ladder.rs` |

## Domain relations

The generated model carries these relations as typed fields. This story adds no behaviour over them. Each is declared in an `ess/1` document:

| relation | from → to | cardinality | ownership | citation |
|---|---|---|---|---|
| `definition` | `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition` | one, via `signal` | references | `spec/domains/observation.yaml:160` |
| `sample` | `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample` | one, via `sample_id` | references | `spec/domains/evaluation.yaml:109` |
| `definition` | `thalamus.evaluation.SignalEvaluation` → `thalamus.definition.SignalDefinition` | one, via `signal` | references | `spec/domains/evaluation.yaml:115` |
| `dimensions` | `thalamus.dimension.DimensionRegistry` → `thalamus.dimension.Dimension` | many, via `registry_id` | registry owns | `spec/domains/dimension.yaml:62` |
| `parent_dimension` | `thalamus.dimension.Dimension` → `thalamus.dimension.Dimension` | one, via `parent` | references | `spec/domains/dimension.yaml:92` |
| `signals` | `thalamus.assertion.Assertion` → `thalamus.definition.SignalDefinition` | many, via `signals` | references | `spec/domains/assertion.yaml:98` |

## Scope

Every path is planned (inferred); none exists yet.

- `Taskfile.yml`, `Cargo.toml`, `Cargo.lock`
- `crates/thalamus-core/Cargo.toml`, `crates/thalamus-store/Cargo.toml`, `crates/thalamus-sources/Cargo.toml`, `crates/thalamus-pipeline/Cargo.toml`, `crates/thalamus-service/Cargo.toml`, `crates/thalamus-cli/Cargo.toml`
- `crates/thalamus-core/src/lib.rs`, `crates/thalamus-core/src/behaviour/mod.rs`, `crates/thalamus-core/src/rules/mod.rs`, `crates/thalamus-store/src/lib.rs`, `crates/thalamus-sources/src/lib.rs`, `crates/thalamus-pipeline/src/lib.rs`, `crates/thalamus-service/src/lib.rs`, `crates/thalamus-cli/src/main.rs`, `crates/thalamus-cli/src/cmd/mod.rs`
- `generated/thalamus-model`, `generated/spec-types`, `spec/suite.json` (generated)
- `AGENTS.md` (command table, planned crates), `docs/design/thalamus.md` (§ 5.1)

The stub module files of the ownership table are created here too, and recorded in the scope of the story that owns them: every owner depends on this story, directly or through its chain, so the two never share a wave.

## Acceptance

- `task check` exits 0 on a clean checkout, and `task drift` exits non-zero after any one file under `generated/` or `spec/suite.json` is edited by hand.
- `cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sort` prints exactly `thalamus-cli`, `thalamus-core`, `thalamus-pipeline`, `thalamus-service`, `thalamus-sources`, `thalamus-store`.
- `cargo run -q -p thalamus-cli -- --help` lists the subcommands `sample`, `eval`, `show`, `compare`, `rollup`, `flaps`; `cargo run -q -p thalamus-cli -- <subcommand>` exits 70 printing `not implemented` for each of the six; `cargo run -q -p thalamus-cli -- --no-such-flag` exits 64.

## Revision after design critic round 2

The skeleton also declares `pub mod conformance;` in `crates/thalamus-core/src/lib.rs` behind a `conformance` Cargo feature (an empty module), owned afterwards by `story:m0-conformance-definitions`.
