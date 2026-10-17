---
format: aep.planning-md/3
id: review-result:adversary-m2-transitions-pass-1
kind: review-result
status: active
title: Adversary, story:m2-transitions, pass 1
relations:
- reviews: story:m2-transitions
revision: 1
---
unit: story:m2-transitions, d16a3f7 on impl/m2-transitions
verdict: NEEDS-CHANGE
cases: `crates/thalamus-service/tests/adv_transitions_p1.rs`, written from reading, run by the correction
origin: introduced 3 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F1 | blocker | `adv_change_settled_while_down_is_recorded_on_restart` | seeding replays stored evaluations and discards what they settle (`transitions.rs:210`), so a change settled by evaluations stored while the service was down, or before a crash between grading and observing, is never recorded or emitted; a failed record is lost the same way | seeding records what the replay settles after the newest stored transition; a failed record is not lost |
| F2 | warning | `adv_regrade_after_restart_does_not_clear_before_the_open` | a series resumed from a transition has no newest ts (`transitions.rs:190`, `hysteresis.rs:170`), so a re-grade of older samples after a restart settles a clear dated before the transition it clears | a resumed series starts at its transition's settled ts |
| F3 | warning | `adv_ok_warn_ok_records_exactly_two_transitions` | the OK, WARN, OK drive records three transitions, the first being the series' first settled OK | accepted: the first settled verdict of a series is a transition (hysteresis rule 4, design § 3.2); the acceptance counts the two transitions of the drive after the series settled OK, and the story says so |

Attacked and held: transition ids after a restart; ties on equal settled ts; the cursor against refresh and record appends; persistence changes mid-run; the CloudEvents required attributes, subject the D10 series key, time the settled ts; several series in one tick.
