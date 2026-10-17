---
format: aep.planning-md/3
id: review-result:adversary-m2-scheduler-pass-1
kind: review-result
status: active
title: Adversary, story:m2-scheduler, pass 1
relations:
- reviews: story:m2-scheduler
revision: 1
---
unit: story:m2-scheduler, uncommitted tree on e07e28c
verdict: NEEDS-CHANGE
cases: executed 8→10 (`crates/thalamus-service/tests/adv_scheduler_p1.rs` 2), red 2
origin: introduced 4 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `late_short_cadence_read_times_out_even_behind_a_longer_wait` | reads are collected one after another against one shared start; a short-cadence read that overran its budget while a longer read was awaited is taken with `recv_timeout(0)` and recorded `ok` with its late value (`scheduler.rs:327-331`), against design § 2.2 | each read carries its own deadline (spawn time + budget); an answer that arrived after it is `unknown`, reason `read timed out` |
| F2 | warning | `backwards_clock_jump_writes_no_missed_slot_behind_the_newest_sample` | a backward clock step lowers the in-memory handled slot, and the next forward tick writes `missed_slot` samples older than the newest stored sample (`scheduler.rs:260`, `:280`) | missed slots count from the later of the handled slot and the newest stored slot; the handled slot never decreases; a slot older than the newest stored sample is not read |
| F3 | note | judgement | the first-tick fill has no upper bound, and a mute that spans a restart is filled as missed while the running path skips it (`scheduler.rs:258-262`) | accepted for M2, D16 records every missed slot; a bound is follow-up work |
| F4 | note | judgement | a hung read holds the tick up to its own cadence, and a timed-out reader thread is detached, never joined (`scheduler.rs:320-331`) | per-read deadlines (F1) bound the wait; the detached thread is accepted for M2 |

Attacked and held: fill across a restart from the newest stored sample; a forward jump while running; no duplicate sample for one slot; unknown never 0 for `missed_slot` and a lone timeout; one `run_id` per pass; a definition activated mid-run starts at its current slot; a cadence change between ticks; the `record_reads` refactor; one write per line in the Connectors double.
