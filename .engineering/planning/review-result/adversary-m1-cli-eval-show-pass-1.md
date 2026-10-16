---
format: aep.planning-md/3
id: review-result:adversary-m1-cli-eval-show-pass-1
kind: review-result
status: active
title: Adversary, story:m1-cli-eval-show, pass 1
relations:
- reviews: story:m1-cli-eval-show
revision: 1
---
unit: story:m1-cli-eval-show, commit 0987708
verdict: NEEDS-CHANGE
cases: executed 53→60 (`crates/thalamus-cli/tests/adv_eval_show_p1.rs` 7), red 3 (one intermittent)
origin: introduced 3 / pre-existing 0 / undecided 1

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `store_offset_baseline_red_takes_the_previous_slot_on_the_window_start` | an `offset_1d` baseline of a `history: store` definition took the previous slot's sample on the start of the shifted window when the slot itself was missing, and graded WARN where it is UNKNOWN (`evaluate.rs:473`) | only a stored sample in the shifted window is the baseline |
| F2 | warning | `show_red_on_a_fresh_instance_creates_the_store` | `thalamus show` on an instance that never sampled created `.thalamus/store.sqlite` (`show.rs:53`) | decided: a read command creates nothing; a missing store answers as an empty one |
| F3 | warning | `two_concurrent_evals_grade_each_sample_once_per_revision` | under load one of two concurrent `eval`s sometimes exited 74 on opening the store; open retries only a lock timeout (`eventlog.rs:463`); once-per-revision held in every run | open waits for the busy error within the lock-timeout budget |
| F4 | warning | `regrade_takes_prev_from_the_earlier_sample_at_the_new_revision` | the mutant `is_lt` → `is_le` at `evaluate.rs:528` (a re-grade takes its own old verdict as `prev`) passed `eval_show` | the pass-1 case kills it |

Attacked and held: one grade per (sample, revision); unknown never 0 or OK; the order of `show`'s exit codes; the refusal codes 64, 65 and 74.
