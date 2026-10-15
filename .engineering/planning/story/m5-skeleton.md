---
format: aep.planning-md/3
id: story:m5-skeleton
kind: story
status: draft
title: Module lines, stubs and the assert, testrun, cortex and transitions subcommands for M5
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m0-workspace
- depends_on: story:m4-skeleton
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/mod.rs
- confidence: inferred
  path: crates/thalamus-cli/src/main.rs
- confidence: inferred
  path: crates/thalamus-core/src/lib.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/lib.rs
- confidence: inferred
  path: crates/thalamus-service/src/http.rs
- confidence: inferred
  path: crates/thalamus-service/src/lib.rs
- confidence: inferred
  path: crates/thalamus-service/tests/http.rs
- confidence: inferred
  path: crates/thalamus-sources/src/lib.rs
- confidence: inferred
  path: crates/thalamus-store/src/lib.rs
revision: 7
---
## Context

`story:m0-workspace` laid every module line, stub and subcommand of M0 to M3, so later stories add code to their own files and never edit a shared line. M5 adds files that `story:m0-workspace` did not declare. This story declares them all at once, the same way, so the M5 stories of one wave never touch the same `lib.rs`, `mod.rs`, subcommand enum or HTTP route table. It is a prefactor: it adds no behaviour.

## Work

- Module lines, each with a stub file that compiles (empty, or an empty `Args` and a `run` that prints `not implemented` and exits 70, as `story:m0-workspace` does; no `todo!()`):

| crate file that gets the module line | new module file (stub) | owner |
|---|---|---|
| `crates/thalamus-core/src/lib.rs` | `assertion.rs`, `declared.rs` | `story:m5-assert`, `story:m5-declared-sets` |
| `crates/thalamus-store/src/lib.rs` | `assertions.rs`, `transition_feed.rs` | `story:m5-assert`, `story:m5-conductor-contract` |
| `crates/thalamus-sources/src/lib.rs` | `testrun.rs` | `story:m5-testrun-ingest` |
| `crates/thalamus-pipeline/src/lib.rs` | `assert.rs`, `ingest.rs`, `testrun_cases.rs`, `cortex_export.rs` | `story:m5-assert`, `story:m5-testrun-ingest`, `story:m5-testrun-assertions`, `story:m5-cortex-export` |
| `crates/thalamus-service/src/lib.rs` | `assertions.rs` | `story:m5-service-answers` |
| `crates/thalamus-cli/src/cmd/mod.rs` | `cmd/assert.rs`, `cmd/testrun.rs`, `cmd/cortex.rs`, `cmd/transitions.rs` | `story:m5-assert`, `story:m5-testrun-ingest`, `story:m5-cortex-export`, `story:m5-conductor-contract` |

- `crates/thalamus-cli/src/main.rs`: the subcommand enum gains `assert`, `testrun` (with the nested subcommand `ingest`), `cortex` (with the nested subcommand `export`) and `transitions`, each variant holding the `Args` of its `cmd/<name>.rs` and calling its `run`. The nested enums live in `cmd/testrun.rs` and `cmd/cortex.rs`, so their owners add variants without touching `main.rs`. `cortex export` is a group of its own so it cannot collide with the `export` subcommand of `story:m4-skeleton`.
- `crates/thalamus-service/src/http.rs` (complete since `story:m4-skeleton`, which owns it): the route table gains `GET /api/assertions` and `GET /api/assertions/{id}`, routed to the handlers `assertions::list` and `assertions::get` of the stub `crates/thalamus-service/src/assertions.rs`, each answering 501 `not implemented` until `story:m5-service-answers` fills them. Other methods on both routes answer 405, as on every route. This is one edit of the route table, after `story:m4-skeleton` (`depends_on`).

Decision taken here (decomposer call, reversible): one skeleton story for M5 rather than a module line in each story, following the `story:m0-workspace` pattern. `story:m4-skeleton` adds module lines to the same `lib.rs`, `cmd/mod.rs` and `main.rs` files (its `export` and `deploys` subcommands, its `cloudevents` line in `crates/thalamus-pipeline/src/lib.rs`) and owns `http.rs`, so this story lands after it (`depends_on`).

## Domain relations

None: this story adds no behaviour and no record. The records the stub files will carry are declared by `story:m5-spec-integration-records`; the relation each owner rests on is in its own body.

## Scope

Planned (inferred):

- `crates/thalamus-core/src/lib.rs`, `crates/thalamus-store/src/lib.rs`, `crates/thalamus-sources/src/lib.rs`, `crates/thalamus-pipeline/src/lib.rs`, `crates/thalamus-service/src/lib.rs`, `crates/thalamus-cli/src/cmd/mod.rs`, `crates/thalamus-cli/src/main.rs` (all owned by `story:m0-workspace`; one line each; after `story:m4-skeleton`, which edits the same files)
- `crates/thalamus-service/src/http.rs` (owned by `story:m4-skeleton`; two route lines; after it)
- `crates/thalamus-service/tests/http.rs` (created by `story:m4-skeleton`; one test added; after it)
- the stub files of the table above, recorded in the scope of the story that owns each; every owner depends on this story

## Acceptance

- `task check` exits 0.
- `cargo run -q -p thalamus-cli -- --help` lists `assert`, `testrun`, `cortex` and `transitions` beside the six subcommands of `story:m0-workspace` and the two of `story:m4-skeleton`; `cargo run -q -p thalamus-cli -- assert`, `-- testrun ingest`, `-- cortex export` and `-- transitions` each exit 70 printing `not implemented`.
- `cargo test -p thalamus-service --test http` passes the failing-first test `assertion_routes_are_laid_and_answer_501` (`GET /api/assertions` and `GET /api/assertions/asr_x` answer 501, `POST /api/assertions` answers 405), through `http::router` and `tower::ServiceExt::oneshot`, beside the tests of `story:m4-skeleton`.
