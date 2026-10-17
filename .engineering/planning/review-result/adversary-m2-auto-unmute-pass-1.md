---
format: aep.planning-md/3
id: review-result:adversary-m2-auto-unmute-pass-1
kind: review-result
status: active
title: Adversary, story:m2-auto-unmute, pass 1
relations:
- reviews: story:m2-auto-unmute
revision: 1
---
unit: story:m2-auto-unmute, 9f30afd as merged at wave/20261017a a067c30
verdict: NEEDS-CHANGE
cases: `crates/thalamus-service/tests/adv_unmute_p1.rs` and `crates/thalamus-cli/tests/adv_unmute_cli_p1.rs`, 6 executed, red 3
origin: introduced 4 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `adv_expired_mute_beside_hourly_definition_is_sampled_each_slot` | `next_slot` (`scheduler.rs:218`) lists definitions without the unmute overlay, so beside an hourly Active definition an expired mute is sampled once an hour and 11 of its 5-minute slots are recorded missed | the next wake-up is chosen over the effective lifecycle |
| F2 | warning | `adv_restart_past_expiry_does_not_fill_the_muted_span` | after a restart past the expiry the first-tick fill skips only slots before `revised_at` (D47), which an automatic unmute leaves unchanged, so 23 muted slots are recorded missed (`scheduler.rs:288`) | the first-tick fill also skips slots before `mute_until`; D47 says so |
| F3 | warning | `adv_cli_sample_of_an_expired_mute_is_not_refused` | `thalamus sample` refuses an expired mute as not-active while the service samples it as Active (`cmd/sample.rs:127,134`) | the CLI applies the same overlay and records no unmute event, which stays the service's |
| F4 | note | judgement | an earlier binary refuses a store holding a `thalamus.definition-event/1` record as unknown schema, so a downgrade fails (`eventlog.rs:633`) | accepted: a store is not downgraded |

Attacked and held: one `DefinitionUnmuted` across two services ticking one store and a restart; a mute set again after expiry honoured and its own expiry recorded; `mute_until` exactly at the tick is expired, one second later is not; the definition file's bytes and mtime unchanged.
