---
format: aep.planning-md/3
id: story:m5-declared-sets
kind: story
status: draft
title: 'thalamus assert asks a declared set and the Asserts: lines of a merge-request description'
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-assert
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/assert.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/assert_declared.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/assert_declared
- confidence: inferred
  path: crates/thalamus-core/src/declared.rs
- confidence: inferred
  path: crates/thalamus-core/tests/declared.rs
revision: 4
---
## Outcome

A release asks its declared set for one environment with `thalamus assert --declared <file> --after <rollout> --asker release:<tag> --dims <env-key>=<env>`, and a merge request is asked from the `Asserts:` lines of its description with `thalamus assert --from-description <path|-> --after <merge time> --asker merge_request:<ref>`. Running the same command again after the horizon answers the same assertions: that is the re-run of design § 6.1.

## Context

Design § 6.1: "a release: after the rollout, assert the release's declared set for every environment it reached"; "a merge request: an `Asserts:` line in its description names signals and a horizon; the factory re-runs it when the horizon passes". The types `DeclaredSet` (`thalamus.asserts/1`) and the line grammar are `story:m5-spec-integration-records`'s; asking and answering one assertion is `story:m5-assert`'s. A merge instant is the merge time, never an announcement (design § 6.5, deploy instant). `epic:assertions-and-integrations` promises that releases, merge requests, test runs and conductor ask thalamus: the release pipeline and the merge-request factory are other repositories, and what each must add is stated below; conductor's part is in `story:m5-conductor-contract`, the test run's in `story:m5-testrun-assertions`.

## Work

- `crates/thalamus-core/src/declared.rs`: parse a `thalamus.asserts/1` file (YAML or JSON) into the generated `DeclaredSet`; parse every `Asserts:` line of a text (grammar: `Asserts: <signal>[,<signal>...] for <duration> [where <key>=<value>[,<key>=<value>...]]`, line starting at column 0); a malformed line is refused naming its line number; an empty set is refused (no signals, as `AskAssertion` outcome `no-signals`).
- `crates/thalamus-cli/src/cmd/assert.rs` (owned by `story:m5-assert`): the flags `--declared <file>`, `--from-description <path|->` (`-` reads the description text from stdin) and `--list` (every recorded assertion of the one `--asker` given, with its id, state, result and worst; no ask is made). `--dims` is merged into every entry, so one environment is one call. Each entry is one ask through `story:m5-assert`; an identical ask returns the recorded assertion, so a re-run never duplicates.
- Combined exit over a set: any breach gives the worst breach code (2 over 1); else any `unknown` gives 3; else any assertion still open gives 75; else 0. `--format json` prints every assertion with its id, result, worst, reason and cited evaluations.

## What each asking repository must add (other repositories, not this story)

Release pipeline (the repository that rolls a release out):

1. Keep the release's declared set as a `thalamus.asserts/1` file in the released repository or its pipeline.
2. After the rollout reaches an environment, take the rollout instant (pod creation, never the announcement) and run `thalamus assert --declared <file> --after <rollout instant> --asker release:<tag> --dims <env-key>=<env> --format json`, once per environment reached.
3. Ask again at the horizon: a pipeline job scheduled at `after + horizon` reruns the same command, or the first call carries `--wait` and blocks until then. The rerun returns the same assertion ids (an identical ask).
4. Read the exit code: 0 within; 1 or 2 breach (fail the release step and name the assertion ids); 3 unknown, never a pass; 75 asked, not yet answerable (only without `--wait`; ask again later); 64, 65, 74 a usage, refused-signal or store error, which is not an answer.

Merge-request factory (the repository that opens and merges merge requests):

1. Pass the description text on stdin, never a forge URL: `thalamus assert --from-description - --after <merge time> --asker merge_request:<ref> --format json`, run after the merge with the merge time as `--after`.
2. Ask at the horizon: rerun the same command once `after + horizon` has passed (the factory's own timer), or use `--wait`; the rerun answers the recorded assertions.
3. Read the exit code as the release pipeline does; a description without an `Asserts:` line exits 0 with `asked 0`.

Thalamus does not call either repository and holds no credential for one.

## Decisions taken here (decomposer calls, reversible)

- The declared set is a file the asker owns (in the released repository or its pipeline), not a thalamus instance file: the release declares what it promises.
- A description with no `Asserts:` line asks nothing and exits 0 with `asked 0`; a declared-set file with no entry is refused (65). A merge request with nothing to assert is not a failure; an empty declared set is a mistake.
- An answered `unknown` (3) outranks an open assertion (75): it is final, the open one is not.
- Thalamus never reads a forge: the asker (CI or a conductor controller) passes the description text in, so thalamus stays read-only against every system it does not watch.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `DeclaredSet` → `DeclaredAssertion` | one-to-many, at least one | the set owns its entries (a file) | declared by `story:m5-spec-integration-records` |
| one `DeclaredAssertion` or `Asserts:` line, asked once per `after`, `asker` and `dims` → one `Assertion` | one-to-one per ask | none: the assertion outlives the file and the description | decomposer call in `story:m5-spec-integration-records` (identical ask returns the recorded id) |
| `Assertion` → `SignalDefinition`, `signals` | many | references | `spec/domains/assertion.yaml:98` |
| `Asker.kind` `release`, `merge_request` | one per assertion | value | `spec/domains/assertion.yaml:17` (`AskerKind`) |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/declared.rs` (module line is `story:m5-skeleton`'s)
- `crates/thalamus-cli/src/cmd/assert.rs` (owned by `story:m5-assert`; this story adds the three flags and the combined exit, after it and before `story:m5-testrun-assertions`)
- `crates/thalamus-core/tests/declared.rs`, `crates/thalamus-cli/tests/assert_declared.rs`, `crates/thalamus-cli/tests/fixtures/assert_declared` (new)

## Acceptance

- `cargo test -p thalamus-core --test declared` passes the failing-first tests `malformed_asserts_line_is_refused_with_its_line_number`, `asserts_line_parses_signals_horizon_and_dims`, `declared_set_parses_into_the_generated_type` and `empty_declared_set_is_refused`.
- `cargo test -p thalamus-cli --test assert_declared` passes `declared_set_exit_is_the_worst_breach`, `one_unknown_entry_makes_the_set_exit_3_never_0`, `rerun_after_horizon_answers_the_same_assertion_ids` (first run exits 75, the second after the horizon exits by the ladder with the same ids), `description_without_asserts_lines_asks_nothing` and `dims_flag_narrows_every_entry`, and:
  - `list_prints_every_assertion_of_one_asker_only` (assertions of `release:v1` and `release:v2` recorded: `--list --asker release:v1` prints exactly the `release:v1` ids and records nothing);
  - `from_description_dash_reads_stdin` (the description piped on stdin with `--from-description -` asks the same assertions as the same text passed as a file).
