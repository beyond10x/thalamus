---
format: aep.planning-md/3
id: review-result:adversary-m3-host-collector-pass-1
kind: review-result
status: active
title: Adversary, story:m3-host-collector, pass 1
relations:
- reviews: story:m3-host-collector
revision: 1
---
unit: story:m3-host-collector base, commit 696be18
verdict: NEEDS-CHANGE
cases: executed 0→15 (`crates/thalamus-sources/tests/adv_host_pass1.rs`), red 6
origin: introduced 6 / pre-existing 0 / undecided 0

## Findings

| id | severity | test | finding | resolution (bb4b45d) |
|---|---|---|---|---|
| F1 | blocker | `disk_used_of_a_directory_that_is_no_mount_point_reads_unknown` | `statvfs` of a directory that is no mount point recorded the containing file system's usage as ok under that mount | a listed path must be `/` or a directory whose device differs from its parent's; else unknown |
| F2 | warning | `disk_used_of_a_regular_file_reads_unknown` | a regular file listed as a mount read ok | same check |
| F3 | warning | `cpu_counter_regression_never_reads_a_busy_share_outside_zero_to_one` | a non-idle `/proc/stat` counter going backwards gave busy −1, ok | every counter must be non-decreasing and busy within [0, 1]; else unknown |
| F4 | warning | `a_window_end_far_from_now_reads_unknown_not_this_moment` | the machine's current values were stamped with any window end, so `thalamus sample --at` a past time recorded today's values | the host reads unknown when the window end is more than 300 s from the machine's clock |
| F5 | note | `local_connection_is_reserved_for_the_host_adapter` | the specification reserves connection `local` for the host adapter, and a prometheus source on `local` was accepted | specification outcome `local-not-host` on ProposeDefinition and ReviseDefinition, refused at create and revise |
| F6 | note | `one_mount_spelled_twice_is_one_series` | `/` and `//` gave two series of one file system | mount spellings normalised before deduplication |

Attacked and held: zero and backwards CPU totals, malformed cpu lines, meminfo defects, empty or relative queries, zero statvfs counts, non-CPU hwmon sensors, unparseable sensor inputs, the set of paths read, determinism and `query_sha256`, non-local connections.

After the fixes all 15 cases pass (`cargo test -p thalamus-sources --test adv_host_pass1`, 15 passed).
