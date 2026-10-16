---
format: aep.planning-md/3
id: review-result:adversary-m1-cli-eval-show-pass-2
kind: review-result
status: active
title: Adversary, story:m1-cli-eval-show, pass 2
relations:
- reviews: story:m1-cli-eval-show
revision: 1
---
unit: story:m1-cli-eval-show, correction 1, commit 8e742af
verdict: NEEDS-CHANGE
cases: executed 60→66 (`crates/thalamus-cli/tests/adv_eval_show_p2.rs` 6), red 2
origin: introduced 2 / pre-existing 0 / undecided 1

## Pass-1 findings rechecked

F1 to F4 are closed; their pass-1 cases pass. The open retry keeps to its 30 s budget: with a write lock held elsewhere, `show` waited 30.09 s and exited 74.

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| P2-F1 | blocker | `store_offset_red_takes_the_previous_slot_when_the_pass_runs_early` | when today's pass is stamped earlier in its slot than yesterday's, the same-slot sample falls just outside the shifted window and the previous slot is taken (`evaluate.rs:480`); `thalamus sample` without `--at` stamps the current time, so passes are never aligned | decided: the stored sample nearest to `ts - offset` within half the cadence, as `thalamus compare` matches; none is `no stored sample at offset` |
| P2-F2 | warning | `show_on_a_store_it_may_not_write_is_unavailable` | `show` opens the store read-write and exits 74 on a store the user cannot write; the website said it "only reads" | decided: a store the user cannot write is unavailable, exit 74, for every command; the page says `show` records nothing |
| P2-F3 | note | judgement | on a PostgreSQL store with no tables, `show` would create them (`eventlog.rs:442`) | no change |
| P2-F4 | warning | pass-2 pin cases | three mutants survived the unit's suites: two in the missing-store check (`eventlog.rs:140`, `:143`) and one dropping Muted definitions from grading (`evaluate.rs:220`) | the pass-2 cases kill them |
