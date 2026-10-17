---
format: aep.planning-md/3
id: review-result:adversary-store-sqlite-id-reserve-locked-pass-1
kind: review-result
status: active
title: Adversary, story:store-sqlite-id-reserve-locked, pass 1
relations:
- reviews: story:store-sqlite-id-reserve-locked
revision: 1
---
unit: story:store-sqlite-id-reserve-locked, 9713a8d on impl/store-sqlite-id-reserve-locked
verdict: NEEDS-CHANGE
cases: `crates/thalamus-store/tests/adv_p1_store_lock.rs` 3 written, 3 pass on the tree; 1 red on a scratch mutant with the append retry arm deleted
origin: introduced 3 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | warning | `adv_p1_appending_under_a_held_write_lock_waits_and_stores_once` | no case of the unit reaches the append retry arm (`eventlog.rs:734`); its test appends only after the reservation waited out the lock | the adversary case stays in the suite |
| F2 | note | `adv_p1_a_lock_held_past_the_budget_is_answered_not_waited_out_forever` | the 30 s budget is checked before a further 5 s driver busy wait, so a call can block about 35 s, not the 30 s the CHANGELOG line states (`eventlog.rs:792`) | the CHANGELOG line says about 35 seconds |
| F3 | note | judgement | `is_sqlite_locked` also retries "database table is locked" and "database schema is locked" (SQLITE_LOCKED), which waiting cannot clear on a private connection (`eventlog.rs:113`) | only SQLITE_BUSY's "database is locked" is retried |

Attacked and held: duplicate event or id after a refused append (`BEGIN IMMEDIATE` before any write, failed `COMMIT` rolled back, a repeated key deduplicated); two writers under a held lock get distinct ids and store once; starvation bounded by the budget; errors other than busy returned at once; the PostgreSQL path untouched; WAL reads never take the write lock.
