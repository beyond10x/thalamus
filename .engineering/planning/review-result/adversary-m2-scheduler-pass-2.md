---
format: aep.planning-md/3
id: review-result:adversary-m2-scheduler-pass-2
kind: review-result
status: active
title: Adversary, story:m2-scheduler, pass 2
relations:
- reviews: story:m2-scheduler
revision: 1
---
unit: story:m2-scheduler, uncommitted tree on e07e28c, after the pass-1 corrections
verdict: NEEDS-CHANGE
cases: executed 10→13 (`crates/thalamus-service/tests/adv_scheduler_p2.rs` 3), red 3
origin: introduced 3 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `a_cli_sample_inside_the_current_slot_does_not_stop_the_first_tick` | the newest stored sample's raw instant is the start point (`scheduler.rs:269`); a hand-run `thalamus sample` off a slot boundary makes the current slot count as behind (`:282`), so it is neither read nor recorded missed | the newest stored sample is rounded down to its slot (`div_euclid(cadence) * cadence`) before it is a start point or a `behind` bound; a slot holding a sample is held |
| F2 | blocker | `a_cli_sample_before_a_late_tick_does_not_lose_the_slot` | the same raw instant, while running, loses a late tick's slot (`:267`, `:282`), against D16 | as F1 |
| F3 | warning | `a_definition_reactivated_after_a_backward_step_reads_no_slot_behind_its_newest_sample` | a mute drops the handled slot; after reactivation the `behind` check ignores the newest stored sample (`:264-267`, `:282`), against the rule at `:281` | `behind = current < max(handled, newest stored slot)`; a definition activated mid-run still gets no fill |
| F4 | note | judgement | a read whose thread panics is recorded at once as `read timed out` (`:349`) | accepted for M2 |

Attacked and held: an answer exactly at its budget is accepted; a reader that never answers times out at its deadline; a read finished in time but collected after a slower one is kept; arrival and deadline use one wall clock; the handled slot never decreases; a forward jump after a backward one fills only after the later of handled and newest stored; a definition activated mid-run is not filled.
