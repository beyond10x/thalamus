---
format: aep.planning-md/3
id: story:m3-host-collector
kind: story
status: implemented
title: A local host collector samples CPU, memory, disk space and CPU temperature when invoked
relations:
- decomposes: epic:source-adapters
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/sample.rs
- confidence: cited
  path: crates/thalamus-cli/tests/adv_sample_breach_pass2.rs
- confidence: cited
  path: crates/thalamus-cli/tests/fixtures/adv_breach_pass2
- confidence: cited
  path: crates/thalamus-cli/tests/fixtures/breach
- confidence: inferred
  path: crates/thalamus-cli/tests/sample_breach.rs
- confidence: inferred
  path: crates/thalamus-core/src/behaviour/definition.rs
- confidence: inferred
  path: crates/thalamus-core/src/registry.rs
- confidence: cited
  path: crates/thalamus-core/tests/conformance.rs
- confidence: inferred
  path: crates/thalamus-docs/src/status.rs
- confidence: inferred
  path: crates/thalamus-pipeline/src/sample.rs
- confidence: inferred
  path: crates/thalamus-sources/Cargo.toml
- confidence: inferred
  path: crates/thalamus-sources/src/host.rs
- confidence: inferred
  path: crates/thalamus-sources/src/lib.rs
- confidence: inferred
  path: crates/thalamus-sources/src/registry.rs
- confidence: cited
  path: crates/thalamus-sources/tests/adv_host_pass1.rs
- confidence: cited
  path: crates/thalamus-sources/tests/adv_host_pass2.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/host
- confidence: inferred
  path: crates/thalamus-sources/tests/host.rs
- confidence: inferred
  path: docs/design/thalamus.md
- confidence: inferred
  path: generated
- confidence: inferred
  path: spec/domains/definition.yaml
- confidence: inferred
  path: spec/suite.json
- confidence: cited
  path: website/docs/concepts/samples.md
- confidence: inferred
  path: website/docs/concepts/signals.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T07:58:25Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-16T07:58:25Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-16T10:25:28Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Context

The operator's direction (2026-10-16): the Prometheus ingestion is proven and stays, but host data (the machine thalamus runs on) does not come from a Prometheus and node-exporter stack. It comes from a small local host collector in thalamus that reads exactly four things: CPU use, memory use, disk space and CPU temperature, and nothing else. It runs only when invoked (`thalamus sample` reads it like any other source); it is no service, no timer and no daemon (AGENTS.md, "Nothing outlives the session that started it").

Design § 5.2 says thalamus invokes operations only through Connectors. Reading the local machine's own counters holds no secret and writes nothing, so the host collector is the one named exception: a source kind that reads this machine's kernel interfaces directly and never another machine. The design records it as decision D46 and amends § 5.1 and § 5.2.

## Specification first

Model the host source in `spec/domains/definition.yaml` before any code: adapter id `host`, the reserved connection `local` (the host collector reads only the machine it runs on; any other connection is refused, never read), and the four extractors below in the naming text. Add conformance scenarios where ESS can express them (a host definition naming a connection other than `local` is refused at create). Validate with `ESS=$HOME/.cache/ess/toolchains/0.60.0/ess`, run `task generate`, implement against the generated model. What ESS cannot express is reported, not hand-modelled.

## Behaviour

Source `{adapter: host, connection: local}`, extractor prefix `host.`, a new `SourceKind::Host` in the registry. The adapter takes no Connectors operation (`read_operations` is empty) and reads only these files, under a root that tests replace with fixtures:

| extractor | kind, unit | read | dims |
|---|---|---|---|
| `host.cpu_busy` | gauge, `1` | two reads of the `cpu` line of `/proc/stat` a short fixed interval apart; busy = 1 − (Δidle + Δiowait) / Δtotal | none |
| `host.memory_used` | gauge, `1` | `/proc/meminfo`: (MemTotal − MemAvailable) / MemTotal | none |
| `host.disk_used` | gauge, `1` | `statvfs` of each mount point the definition's `query` lists, used = (f_blocks − f_bfree) / (f_blocks − f_bfree + f_bavail), as `df` computes it | `mount` |
| `host.cpu_temperature` | gauge, `Cel` | the hottest `temp*_input` of the CPU sensor under `/sys/class/hwmon` (`coretemp`, `k10temp`, `zenpower`, `cpu_thermal`), else a `/sys/class/thermal` zone of type `x86_pkg_temp` or `cpu-thermal`; millidegrees ÷ 1000 | none |

- Unknown is never 0 and never OK: a missing file, a missing field, a zero total, a mount that does not exist or a machine with no CPU sensor reads `quality: unknown` with a reason naming what was missing, and no value.
- A host definition whose connection is not `local` reads unknown with the reason "host reads only this machine (connection local)", never this machine's values.
- `raw` is the producer's text of the scalar (the millidegree string, the meminfo line values); `query_sha256` as every adapter.
- Nothing else is read: no process list, no network, no load average, no other `/proc` or `/sys` path.

## Acceptance

`cargo test -p thalamus-sources --test host` passes, with known-answer tests over fixtures in `crates/thalamus-sources/tests/fixtures/host/`, each seen failing before the adapter exists: `cpu_busy_is_one_minus_idle_share_of_the_delta`, `memory_used_is_total_minus_available_over_total`, `disk_used_matches_df_per_listed_mount`, `cpu_temperature_is_hottest_cpu_sensor_in_celsius`, `no_cpu_sensor_reads_unknown_never_zero`, `missing_meminfo_field_reads_unknown`, `unknown_mount_reads_unknown`, `connection_other_than_local_reads_unknown`; and `ess specify validate` and `task drift` are green on the changed specification.

## Scope

- `spec/domains/definition.yaml`, `generated/`, `spec/suite.json` (regenerated)
- `crates/thalamus-sources/src/host.rs` (new), `crates/thalamus-sources/src/lib.rs` (module line), `crates/thalamus-sources/src/registry.rs` (the `Host` arm), `crates/thalamus-sources/Cargo.toml` if `statvfs` needs a dependency (`rustix` or `nix`, whichever the workspace already locks)
- `crates/thalamus-sources/tests/host.rs`, `crates/thalamus-sources/tests/fixtures/host/` (new)
- `crates/thalamus-docs/src/status.rs` (one `shipped` item), `website/docs/concepts/signals.md` (the host source under Sources), then `task docs-generate`
- `docs/design/thalamus.md`: § 5.1 row of `thalamus-sources`, § 5.2 exception, § 11 row D46

## Not in this story

Any other host metric; reading another machine; a Prometheus or node-exporter stack for host data; running the collector on a schedule (the scheduler samples it like any definition when it exists).

## Amendment 1 (2026-10-16): free space and the breach `thalamus sample` reports

The operator's addition to this wave: a host keeps a floor of free space on a mount (for the operator's own host, 200 GB on `/`), and `thalamus sample` reports when a reading is at or under it. The floor is instance data: it lives in the instance's definition, never in this repository (AGENTS.md, "No instance data in Git", "One copy of every threshold"); tests and docs use synthetic numbers.

- A fifth extractor, `host.disk_free`: kind gauge, unit `By`, dims `mount`, the bytes available to an unprivileged writer (`f_bavail × f_frsize`, the "Avail" column of `df`), for each mount point the `query` lists. Unknown, never 0, as the other four. It reads only `statvfs`, so the "only CPU, memory, disk space and CPU temperature" boundary holds.
- The floor is expressed with what the model already has: `evaluation.rule: lower_is_worse`, `baselines: [fixed]`, `evaluation.fixed` the floor in bytes, and the WARN or CRIT line the threshold rule derives from it (`crates/thalamus-core/src/rules/threshold.rs`). If the existing lines cannot say "at or under the fixed number is CRIT", that is a specification change made first, not a special case in code.
- `thalamus sample` reports a breach: after recording, every sample whose definition is graded against `fixed` alone (no stored history needed) is graded through `thalamus_core::grading`, and each WARN or CRIT prints one line on stdout after the sample ids, `breach <VERDICT> <signal> <dims> <value> <unit> <line>`, and the same as `breaches` in `--format json`. No evaluation is recorded (that is `thalamus eval`'s, `story:m1-cli-eval-show`), the exit code stays 0 (codes 0 to 3 are the verdict ladder's, and a breach is not a failure of the run), and nothing runs outside the command.
- If the CLI's output is described in the specification, the breach line is specified there first.

Acceptance, in addition: `cargo test -p thalamus-sources --test host` adds `disk_free_is_avail_bytes_per_listed_mount` and `disk_free_of_unknown_mount_reads_unknown`; `cargo test -p thalamus-cli --test sample_breach` passes `a_reading_under_the_fixed_floor_prints_a_crit_breach`, `a_reading_over_the_floor_prints_no_breach`, `an_unknown_reading_prints_no_breach_and_is_never_ok`, `breaches_appear_in_json_output`, each seen failing first.

Scope, in addition: `crates/thalamus-cli/src/cmd/sample.rs`, `crates/thalamus-cli/tests/sample_breach.rs` (new) and its fixtures, `crates/thalamus-pipeline/src/sample.rs` if the pass must hand back the definition per sample.

## Amendment 2 (2026-10-16): review corrections

- A reading exactly at the floor is a breach: the threshold rule's lines are "at or under" (`threshold.rs`), and the floor follows them rather than a special case (adversary pass 2, F11).
- The host source reads unknown, never another file system's numbers, for a listed path that is no mount point, a regular file, a symlink, or a path with `.` or `..` components; for a file system of zero blocks; for a `/proc/stat` counter that went backwards; and for a window end more than 300 s from the machine's clock. Connection `local` is refused for any adapter but `host` (`local-not-host`). Adversary passes 1 and 2, `crates/thalamus-sources/tests/adv_host_pass1.rs`, `adv_host_pass2.rs`, `crates/thalamus-cli/tests/adv_sample_breach_pass2.rs`.
