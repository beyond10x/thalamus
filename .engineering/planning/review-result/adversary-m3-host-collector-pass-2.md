---
format: aep.planning-md/3
id: review-result:adversary-m3-host-collector-pass-2
kind: review-result
status: active
title: Adversary, story:m3-host-collector, pass 2
relations:
- reviews: story:m3-host-collector
revision: 1
---
unit: story:m3-host-collector, Amendment 1 and the pass-1 corrections, commit bb4b45d
verdict: NEEDS-CHANGE
cases: executed 0→10 (`crates/thalamus-sources/tests/adv_host_pass2.rs` 7, `crates/thalamus-cli/tests/adv_sample_breach_pass2.rs` 3), red 5
origin: introduced 5 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution |
|---|---|---|---|---|
| F7 | warning | `a_symlink_into_another_file_system_is_no_mount_point` | the mount check followed a symlink for the path but not its parent, so a link into another file system read that file system's space | a symlink is no mount point |
| F8 | warning | `a_dot_dot_path_below_a_mount_is_no_mount_point` | `/sys/fs/cgroup/..` was compared with its lexical parent and answered a mount point | paths with `.` or `..` components read unknown |
| F9 | warning | `disk_free_of_a_zero_size_file_system_reads_unknown` | `disk_free` of a zero-block file system read ok 0 bytes, which a floor grades CRIT | zero blocks read unknown; a full real disk stays a measured 0 |
| F10 | note | `one_mount_spelled_with_a_dot_is_one_series` | `/proc` and `/proc/.` gave two series | as F8 |
| F11 | note | `a_reading_exactly_at_the_floor_is_no_crit_breach` | a reading exactly at the floor printed CRIT, where Amendment 1 said "under" | decided: the threshold rule's at-or-under lines stay; the story says "at or under" (Amendment 2) and the test is inverted to `a_reading_exactly_at_the_floor_is_a_crit_breach` |
| F12 | note | judgement | the `sample` module doc gave a floor recipe without the `warn` line the loader requires | the doc names the warn line |

Attacked and held: `disk_free` overflow, zero fragment size, missing and relative mounts; mount spellings `//`, `/data//x`, `/data/x/`; the 300 s tolerance at ±300 and ±301 s, past and future; no breach for a definition not graded against `fixed`; the end-to-end host breach through `thalamus sample` with exit 0; `local-not-host` on both commands.
