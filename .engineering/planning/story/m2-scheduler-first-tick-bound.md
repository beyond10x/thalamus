---
format: aep.planning-md/3
id: story:m2-scheduler-first-tick-bound
kind: story
status: draft
title: Bound the scheduler's first-tick missed-slot fill
relations:
- decomposes: epic:history-and-transitions
- depends_on: story:m2-scheduler
revision: 1
---
## Context

`thalamus-service run` fills every slot an `Active` definition missed since its newest stored sample with one `unknown` sample per series, reason `missed_slot` (design D16). On the first tick that fill has no upper bound: an instance whose store was last written months ago gets one sample per slot per series for the whole gap. A definition muted across a restart is filled as missed on the first tick, while the running scheduler skips a muted span. Found by adversary pass 1 of `story:m2-scheduler` (finding F3), accepted for M2.

## Work

- Bound the first-tick fill (a number of slots or a span, decided in the design's § 11 table) and say what the scheduler records for slots past the bound.
- Make the first tick and the running path agree on a mute that spans a restart.

## Scope

- `crates/thalamus-service/src/scheduler.rs`
- `crates/thalamus-service/tests/scheduler.rs`
- `docs/design/thalamus.md` (§ 11 row)

## Acceptance

- `cargo test -p thalamus-service --test scheduler` passes a failing-first test in which a store last written far behind the bound is filled with at most the bound's slots, and one in which a definition muted across a restart records no `missed_slot` sample for the muted span.
