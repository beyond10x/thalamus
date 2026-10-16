# Host collector fixtures

Synthetic machine trees for `tests/host.rs`. Each directory is a root the host collector reads
under in place of `/`: `proc/stat`, `proc/meminfo`, `sys/class/hwmon/*` and
`sys/class/thermal/*`, with the numbers the known answers are computed from.

- `proc/stat.after` is the second read of `proc/stat` (the test machine answers it on the second
  read; the collector itself never names it).
- `statvfs` stands in for the `statvfs(3)` call: one line per mount point,
  `<mount> <f_blocks> <f_bfree> <f_bavail>`. A mount it does not list does not exist.
