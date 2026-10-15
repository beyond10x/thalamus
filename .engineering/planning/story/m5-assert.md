---
format: aep.planning-md/3
id: story:m5-assert
kind: story
status: draft
title: thalamus assert answers from stored evaluations with the ladder exit; unknown is not a pass
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-spec-integration-records
- depends_on: story:m5-skeleton
- depends_on: story:m1-cli-eval-show
- depends_on: story:m1-verdict-grading
- depends_on: story:m2-auto-unmute
- depends_on: story:m2-transitions
- depends_on: story:m4-deploy-instants
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/assert.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/assert.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/assert
- confidence: inferred
  path: crates/thalamus-core/src/assertion.rs
- confidence: inferred
  path: crates/thalamus-core/tests/assertion.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/assert.rs
- confidence: inferred
  path: crates/thalamus-store/src/assertions.rs
- confidence: inferred
  path: crates/thalamus-store/tests/conformance.rs
revision: 5
---
## Outcome

A caller runs `thalamus assert <signal>... --dims k=v --after <ts> --for <horizon> --within baseline --asker <kind>:<ref>` and gets the answer as an exit code of the verdict ladder: 0 within, 1 or 2 breach, 3 unknown. Unknown is not a pass.

## Context

Design § 6.1, § 2.3 (ladder, worst-of `CRIT > WARN > UNKNOWN > OK = N/A`), § 11 D1, D2, D3, D19. The record is `thalamus.assertion.Assertion` (`spec/domains/assertion.yaml`), extended with `evaluations` by `story:m5-spec-integration-records`. The ask/answer pass lives in `thalamus-pipeline` so the CLI and the service (`story:m5-service-answers`) run the same code. Code is Rust, clap derive (AGENTS.md).

## Work

- `crates/thalamus-core/src/assertion.rs` (pure, no clock: `now` is passed in):
  - D1: every named signal exists and is `Active` (a `Muted` definition whose expiry has passed counts as `Active`, D19, through the overlay of `story:m2-auto-unmute`); otherwise the ask is refused naming each signal, and nothing is recorded.
  - the window: for each named signal, every stored series whose dims contain every `--dims` pair; the evaluations with `after <= ts < after + horizon`, one per sample, at its highest `definition_rev` (as D14 counts).
  - D3: not answerable while `now < after + horizon`.
  - the answer: worst-of over the window; `WARN` or `CRIT` is `breach` with that worst; `UNKNOWN` is `unknown`; every evaluation `N/A` is `unknown` (D3); no matching series or no evaluation in the window is `unknown` with worst `UNKNOWN`; otherwise `within`. `reason` names the series and evaluation that decided it. The answer lists every evaluation it read (D2).
- `crates/thalamus-store/src/assertions.rs`: the store port for `Assertion` and the generated storage over it (`AskAssertion`, `AnswerAssertion`, views `Assertions` and `OpenAssertions`); the lookup that makes an identical ask return the existing id (the `UNMAPPED:` rule of `story:m5-spec-integration-records`). A second `AnswerAssertion` that meets `AssertionStateConflict` is read back, not reported as a failure.
- `crates/thalamus-store/tests/conformance.rs` (owned by `story:m1-eventlog-store`): one selection line adding the `AskAssertion` and `AnswerAssertion` scenarios with the Eventlog storage; ordered after `story:m4-deploy-instants`, the last earlier story adding a line (after `story:m2-transitions`).
- `crates/thalamus-pipeline/src/assert.rs`: ask, then answer at once when `now >= after + horizon`; `answer_open(now)` for the service.
- `crates/thalamus-cli/src/cmd/assert.rs`: the flags above, `--format json|text`, `--wait` (block until `after + horizon`, then answer), and `--id <asr_...>` (print a recorded assertion). Exit codes: `within` 0, `breach` the worst verdict exit (1 or 2), `unknown` 3; an assertion still open (horizon not passed, no `--wait`) exits 75 printing its id; a D1 refusal 65; an unreachable store 74; a usage error 64. Refusals and the open state never use a ladder code.

## Decisions taken here (decomposer calls, reversible)

- An assertion reads raw evaluations, not settled transitions: one `WARN` evaluation in the window is a breach, because the answer cites evaluations (D2) and hysteresis exists to stop flags flapping, not to hide an out-of-bounds reading.
- Exit 75 (`EX_TEMPFAIL`) for "asked, not yet answerable": off the ladder, non-zero, so no caller reads it as a pass.
- The CLI answers as the `Answerer` when the horizon has already passed; the service answers the rest (`story:m5-service-answers`).

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `Assertion` → `SignalDefinition`, `signals`; each must exist and be `Active` at ask | many | references | `spec/domains/assertion.yaml:98`; design § 11 D1 (`spec/domains/assertion.yaml:66`) |
| `Assertion` → `SignalEvaluation`, `evaluations`: the evaluations the answer read | many | references | declared by `story:m5-spec-integration-records`; design § 11 D2 |
| `SignalEvaluation` → `SignalSample`, `sample`; one counted per sample at its highest revision | many-to-one | references | `spec/domains/evaluation.yaml:109`; design § 11 D5, D14 |
| the series of a signal: signal id plus dims | one signal, many series | key, not an entity | `thalamus.observation.SeriesKey`, `spec/domains/observation.yaml:20` (D10) |

## Scope

Planned (inferred):

- `crates/thalamus-core/src/assertion.rs`, `crates/thalamus-store/src/assertions.rs`, `crates/thalamus-pipeline/src/assert.rs`, `crates/thalamus-cli/src/cmd/assert.rs` (module and subcommand lines are `story:m5-skeleton`'s)
- `crates/thalamus-store/tests/conformance.rs` (one selection line; after `story:m4-deploy-instants`)
- `crates/thalamus-core/tests/assertion.rs`, `crates/thalamus-cli/tests/assert.rs`, `crates/thalamus-cli/tests/fixtures/assert` (new)

`crates/thalamus-cli/src/cmd/assert.rs` is later extended by `story:m5-declared-sets` and then `story:m5-testrun-assertions`, each after this story.

## Acceptance

- `cargo test -p thalamus-core --test assertion` passes the failing-first tests `all_na_window_answers_unknown` (D3), `unknown_in_window_is_not_a_pass`, `warn_in_window_is_breach_with_exit_1`, `crit_in_window_is_breach_with_exit_2`, `no_evaluation_in_window_answers_unknown`, `answer_cites_every_evaluation_it_read` (D2), `not_answerable_before_after_plus_horizon` (D3) and `missing_or_inactive_signal_is_refused` (D1), and:
  - `dims_select_only_series_containing_every_pair` (two series of one signal, `env=a` CRIT and `env=b` OK: `--dims env=b` answers `within` and cites only `env=b` evaluations);
  - `highest_definition_rev_counts_once_per_sample` (one sample graded CRIT at rev 1 and OK at rev 2 answers `within` and cites only the rev-2 evaluation);
  - `expired_mute_reads_as_active` (a `Muted` definition whose expiry is before `now` is asked and recorded, D19), beside `unexpired_mute_is_refused` (expiry after `now`: refused as inactive, D1).
- `cargo test -p thalamus-cli --test assert` passes, against the built binary and a seeded store, `assert_exit_codes_follow_the_ladder` (0, 1, 2, 3), `open_assertion_exits_75_with_its_id`, `same_ask_twice_records_one_assertion`, `assert_id_reads_the_recorded_answer`, `refused_signal_exits_65_and_records_nothing`, and `wait_blocks_until_horizon_then_answers` (`--for 3s --wait` with `after` = now: the process does not exit before `after + 3s`, then exits by the ladder, never 75, and `--id` reads the same answer).
- `cargo test -p thalamus-store --test conformance` passes `thalamus.assertion.AskAssertion/outcome/asked` and `thalamus.assertion.AnswerAssertion/outcome/answered` with the Eventlog storage.
