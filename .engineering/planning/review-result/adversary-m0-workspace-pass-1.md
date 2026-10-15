---
format: aep.planning-md/3
id: review-result:adversary-m0-workspace-pass-1
kind: review-result
status: active
title: Adversary, story:m0-workspace, pass 1
relations:
- reviews: story:m0-workspace
revision: 1
---
unit: story:m0-workspace (uncommitted working tree on impl/m0-workspace, base 911a539)
verdict: red
cases: executed 6→8, red 1
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/thalamus-waves/20261015a/ws/scratch/adversary/ (see part 6)
needs-coordinator: yes. Finding 1 needs a fix in `crates/thalamus-core/Cargo.toml`, a file that only this unit owns. Finding 2's test checks stubs that later stories replace, so you decide whether to keep it.

The unit is red: `task check` fails because the `conformance` feature is never switched on for thalamus-core's own tests, and the next story's test needs it. A second gap is that no test checks the exit-70 stub contract.

**1. Diff stat.** My two test files are new and untracked, so they appear in `git status` and not in `git diff --stat`. No implementation file was touched.
```
 .gitignore              |  1 +
 AGENTS.md               | 16 ++++++++++++----
 docs/design/thalamus.md |  6 ++++--
 3 files changed, 17 insertions(+), 6 deletions(-)
?? crates/thalamus-core/tests/adv_conformance_feature.rs   (mine)
?? crates/thalamus-cli/tests/adv_stub_exit.rs              (mine)
```

**2. Cases added (each run alone before the suite)**

| file | asserts | now |
|---|---|---|
| `crates/thalamus-core/tests/adv_conformance_feature.rs` | thalamus-core's integration tests are built with `feature = "conformance"` | red |
| `crates/thalamus-cli/tests/adv_stub_exit.rs` | each of the six subcommands exits 70 and prints `not implemented` | green; red on a mutant |

- **First case, alone** (`cargo test --locked -p thalamus-core --test adv_conformance_feature`, exit 101):
  ```
  panicked at crates/thalamus-core/tests/adv_conformance_feature.rs:9:5:
  thalamus-core's integration tests are built without the `conformance` feature, so `thalamus_core::conformance` does not exist for tests/conformance.rs
  test result: FAILED. 0 passed; 1 failed
  ```
  My first version used a plain `assert!(cfg!(..))`. Clippy rejected it (`assertions_on_constants`), so I wrapped it in `black_box`. That was my lint error, not a finding. The red result above was captured after the fix.
- **Mutant probe in a scratch copy:** I changed `sample.rs` to return `ExitCode::SUCCESS`.
  - The implementor's `cli.rs` stays green: `6 passed; 0 failed`.
  - `adv_stub_exit` goes red: `thalamus sample  left: Some(0) right: Some(70)`.
- **Fix probe in the same scratch copy:** I added `[dev-dependencies] thalamus-core = { path = ".", features = ["conformance"] }`. `adv_conformance_feature` then passes (`1 passed`). `Cargo.lock` gains one line, `"thalamus-core",` under thalamus-core's dependencies.

**3. Suite run, after the cases existed:** `task check` exited 201. The steps ran in order: validate, drift, fmt, clippy (all passed), then test. In the test step: `adv_stub_exit` 1 passed, `cli.rs` 6 passed, `adv_conformance_feature` FAILED, then `Failed to run task "test": exit status 101`.

**4. Findings**

| # | file:line | finding | verdict / origin |
|---|---|---|---|
| 1 | `crates/thalamus-core/Cargo.toml:9-10` | The `conformance` feature is declared, but nothing turns it on for tests, and clippy never lints the module. | NEEDS-CHANGE / introduced |
| 2 | `crates/thalamus-cli/tests/cli.rs` | No test checks the "exits 70, prints `not implemented`" acceptance line, so a stub that exits 0 stays green. | CONFIRMED / introduced |
| 3 | `Taskfile.yml:31` | `diff -r` ignores file-mode changes: after a `chmod +x` on a generated file, drift still exits 0. | INFEASIBLE / introduced |

- **Finding 1, what reaches it:** `story:m0-conformance-definitions` says its `tests/conformance.rs` calls `thalamus_core::conformance::run`, and its acceptance is `cargo test -p thalamus-core --test conformance`. That story's scope does not include `thalamus-core/Cargo.toml` or `Taskfile.yml`. It would have to edit a file it doesn't own, or put the test behind `#![cfg(feature)]`, which would make it pass while running nothing. The fix is the self dev-dependency from the probe.
- **Finding 2, what reaches it:** the story's acceptance line. My test checks stubs that later stories replace, so each owner story has to remove its subcommand from `STUBS`.
- **Finding 3, what reaches it:** I found nothing that would cause such a change.

**5. Attacked and could not break**
- **Drift detection:** it fails (exit 201) on each of these:
  - an edit to `generated/thalamus-model/src/lib.rs`, `types.rs`, `plan.json` or the generated `Cargo.toml`
  - an emptied `PLAN.md`
  - an edited or deleted `spec/suite.json`
  - an added file, a deleted file, or a stray directory under `generated/`
  - an edit to `spec/` without regenerating
- **Drift is reproducible:** a copy at a different path gave identical output, and no `thalamus-drift.*` temp directories were left after 15 runs.
- **The gate cannot skip steps:** in a copy with a drift failure, `task check` stopped at drift and never built `target/`.
- **The module-ownership table matches the files:** 60 paths, none missing. The only extra files are the `lib.rs`/`mod.rs`/`main.rs` files, `conformance.rs` and `cli.rs`.
- **`cargo metadata` lists exactly the six crates.**
- **Usage errors exit 64:** unknown flag, unknown subcommand, missing subcommand, and an unknown flag on each subcommand. `--help` exits 0.
- **No personal paths or org names:** a grep for `/home/`, `/tmp/`, `.cache/`, company names and `timo` over every new or changed file found nothing.
- **The generated `server` feature compiles:** `cargo check --features thalamus-model/server` finished.

**6. Paths written outside the worktree** (under `~/.cache/thalamus-waves/20261015a/ws/scratch/adversary/`)
- `copy/`: a clean copy without `target/`
- `mut/`: 224M, including its own `target/`
- logs: `check.log`, `m-*.log` (14)
- `table.txt`, `files.txt`

I deleted the probe script. My worktree session lease is released.

```findings
- file: crates/thalamus-core/Cargo.toml
  line: 9
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the conformance feature is never enabled for thalamus-core's own integration tests or the clippy gate, so the next story's tests/conformance.rs cannot reach thalamus_core::conformance without editing a file outside its scope; fix is a self dev-dependency with features = ["conformance"]
- file: crates/thalamus-cli/tests/cli.rs
  line: 1
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no test asserts the acceptance's exit-70 "not implemented" stub contract, so a stub returning ExitCode::SUCCESS keeps all 6 CLI tests green (caught by adv_stub_exit.rs)
- file: Taskfile.yml
  line: 31
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: drift's diff -r ignores file-mode changes, so a chmod +x on a generated file passes drift although git would commit it; no workflow found that produces one
```
