# Changelog

Each release is the tag `v<version>`, where `<version>` is `version` in `Cargo.toml`. A change a
user sees adds a line under `## Unreleased` in the same change; a release turns that section into
`## <version> — <YYYY-MM-DD>` (see "Cutting a release" in [AGENTS.md](AGENTS.md)).

## Unreleased

- `thalamus-service run` records each settled verdict change of a series (design § 3.2) as a
  transition in the store and writes it to standard output as one CloudEvents JSON line of type
  `dev.beyond10x.thalamus.transition`, with the series key as `subject`; the hysteresis of each
  series is seeded from the store on start.

## 0.2.0 — 2026-10-17

- A `rolling_median` baseline over a series of `rolling_n` samples or more with fewer than `rolling_n` `ok` ones is `UNKNOWN` with reason `too few ok samples`, not `baseline predates series`.
- `thalamus-service run` fills at most 288 missed slots on its first tick, and none from before the revision in force, so a long-stopped or muted-across-restart definition no longer gets a sample per slot of the whole gap (design D47).
- A store on SQLite waits out a write lock another connection holds, for about 35 seconds at most, when it reserves an id or appends, instead of panicking with `database is locked`.
- `thalamus flaps --since <duration>` measures raw and settled verdict changes per family and
  series-day over stored evaluations, with `--rev`, `--k`, `--k-crit`, `--m` and `--sweep`.
- `thalamus-service run` samples a muted definition whose `mute_until` has passed as Active,
  records its `DefinitionUnmuted` once in the store as actor `thalamus.definition.Service`, and
  warns on every pass naming the definition file, which it never rewrites.

## 0.1.0 — 2026-10-17

Specification and model

- Five ESS domains under `spec/` specify signal definitions, dimensions, samples, evaluations and
  assertions; `task check` validates them and every synthesised conformance scenario passes.
- The Rust model (`thalamus-model`, `thalamus-spec-types`) is generated from the specification, and
  `task check` fails when it drifts.
- An unknown reading carries no value, and a unit is UCUM.

Grading

- One verdict, score and reason per reading on the ladder `OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`;
  an unknown reading is never `OK`.
- Offset, pooled, higher-of, lower-of and fixed baselines; a baseline older than its series is
  `UNKNOWN`.
- `same_slot_median_4w` and `rolling_median` baselines, computed from the series' stored samples;
  a missing week or fewer than `rolling_n` `ok` samples is `UNKNOWN`.
- Persistence settles verdict changes: `k` agreeing evaluations (`k_crit` for `CRIT`), and `WARN`
  clears after `m` `OK` evaluations, never on `N/A` or `UNKNOWN`.
- Other producers' vocabularies (colour ladders, detector exits, blocking gate exits) map onto the
  ladder.
- An advisory gate grades `WARN`, never `CRIT`.

Instances and storage

- Definitions and the dimension registry load and validate; only Active definitions whose
  dimensions are registered are sampled.
- Samples and evaluations persist in Eventlog, one append-only stream per instance, on the File or
  SQLite provider.

Sources

- One read-only Connectors client for every source, with a recorded double for tests.
- Prometheus, Loki, SQL, Kubernetes, Git hosting, HTTP probe and the host collector (CPU use,
  memory use, disk space, CPU temperature).

Command line

- `thalamus sample`, `thalamus eval`, `thalamus show` (exits with the worst current verdict) and
  `thalamus compare --vs <offset>`; a usage error exits 64, never a verdict code. `rollup` and
  `flaps` are parsed and still exit 70, not implemented.
- `thalamus-service run --instance <dir>` samples each Active definition on its cadence until
  stopped; a slot it missed is recorded as an `unknown` sample with reason `missed_slot`, and a read
  past its cadence is `unknown`, `read timed out`.

Project

- CI runs `task check` on every push and pull request.
- Each release is a tag `v<version>`, a CHANGELOG section and a GitHub Release with the built
  `thalamus` binary and `SHA256SUMS`.
