---
format: aep.planning-md/3
id: review-result:adversary-m1-instance-registry-pass-1
kind: review-result
status: active
title: Adversary, story:m1-instance-registry, pass 1
relations:
- reviews: story:m1-instance-registry
revision: 1
---
unit: story:m1-instance-registry, uncommitted tree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-instance-registry (branch impl/m1-instance-registry, base ca445f1)
verdict: NEEDS-CHANGE
cases: executed 321→341, red 12
origin: introduced 13 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths under ~/.cache/thalamus-waves/20261015d/m1-instance-registry/scratch/ (part 6)
needs-coordinator: yes. Finding 9: the generated RetireDefinition answers `Retired` over ports that cannot fail a write. Nothing in this unit's files can fix that; it needs a decision on the D19 write surface.

## 1. Diff

`git --no-pager diff --stat` is the same as the tree I was handed. Every path in it belongs to the implementor or the coordinator:

```
 Cargo.lock                           |   49 ++
 crates/thalamus-core/Cargo.toml      |    2 +
 crates/thalamus-core/src/registry.rs | 1363 ++++++++++++++++++++++++++++++++++
 crates/thalamus-core/src/series.rs   |  172 +++++
 crates/thalamus-core/src/since.rs    |  215 ++++++
 crates/thalamus-docs/src/status.rs   |    6 +-
 website/data/status.json             |    4 +-
 website/docs/concepts/signals.md     |   29 +-
 8 files changed, 1822 insertions(+), 18 deletions(-)
```

My changes are two new untracked test files, so `diff --stat` cannot show them. I changed no implementation file.

| file | lines |
|---|---|
| `crates/thalamus-core/tests/adv_registry.rs` | 423 |
| `crates/thalamus-sources/tests/adv_registry_window.rs` | 34 |

A throwaway probe, `crates/thalamus-core/tests/adv_probe_yaml.rs`, was created, run once and deleted.

## 2. Cases added

### 2a. Red cases

Twelve cases, red now. Each was run alone before any suite run. The logs are `scratch/adversary/adv-registry-red.log`, `adv-registry-window-red.log` and `adv-objective-nesting-red.log`. For each case the table gives the first line of the panic message verbatim, after `panicked at …:`.

| # | case | asserts | red output |
|---|---|---|---|
| 1 | `adv_registry.rs:69` `adv_registry_key_named_twice_is_refused` | a registry that names `component` twice (`file:` then `open`) does not load | `a key named twice loaded as Some(Open)` |
| 1 | `adv_registry.rs:86` `adv_query_name_given_twice_is_refused` | a `query` that names `numerator` twice does not load | `a query name given twice: loaded "service.api.request_failure_ratio" from` |
| 2 | `adv_registry.rs:99` `adv_connection_url_without_double_slash_is_refused` | `https:metrics.example/api`, `https:/metrics.example/api` and `file:/var/run/metrics.sock` are refused | `https:metrics.example/api: loaded "service.api.request_failure_ratio" from` |
| 3 | `adv_registry.rs:119` `adv_signal_id_first_segment_is_the_family` | id `service.api.…` with `family: platform` is refused | `id service.api.request_failure_ratio under family platform: loaded "service.api.request_failure_ratio" from` |
| 4 | `adv_registry.rs:131` `adv_window_no_adapter_reads_is_refused` | windows `P1M`, `P1Y`, `P1Y2M` and `99999999999999999999h` are refused | `P1M: loaded "service.api.request_failure_ratio" from` |
| 4 | `thalamus-sources/tests/adv_registry_window.rs:15` `adv_every_admitted_window_is_one_the_adapters_place` | every window the loader admits is one `window_ending` can place | `window P1M loaded as "P1M"; the adapters refuse it: window "P1M" is not an ISO 8601 duration of days, hours, minutes and seconds` / `window P1Y …` / `window 99999999999999999999h loaded as "PT99999999999999999999H"; the adapters refuse it: …` |
| 5 | `adv_registry.rs:143` `adv_zero_window_and_cadence_are_refused` | `0h`, `PT0S` and `P0D` are refused as `window` and as `cadence` | `window 0h: loaded "service.api.request_failure_ratio" from` |
| 6 | `adv_registry.rs:158` `adv_dimension_parent_cycle_is_refused` | `component: {parent: component}` and a parent cycle `cluster`↔`namespace` are refused | `a parent cycle loaded:` |
| 7 | `adv_registry.rs:173` `adv_file_locator_outside_the_instance_is_refused` | `file:/etc/passwd` and `file:../other-instance/…` are refused | `file:/etc/passwd loaded` |
| 8 | `adv_registry.rs:191` `adv_objective_numbers_are_json_numbers` | a loaded objective, written with `thalamus_model::json::push_value`, parses with `thalamus_model::json::parse` | `objective .nan loaded and is written as {"target":.nan}, which is not JSON` |
| 8 | `adv_registry.rs:219` `adv_objective_nesting_reads_back` | the same check for an objective nested 100 deep | `an objective nested 100 deep loaded, and its JSON is refused: at byte 65: expected a document nested less than 64 deep` |
| 9 | `adv_registry.rs:245` `adv_retire_through_the_file_ports_does_not_answer_retired` | generated `retire_definition` over `Instance` does not answer `Retired` | `the caller was told Retired { definition_retired: DefinitionRetired { signal_id: SignalId("service.api.request_failure_ratio") } }; the definition is still Some(Active)` |

### 2b. Mutant catchers

Eight cases, green now. They were written after the mutation run in part 2c.

| finding | cases | mutants they kill |
|---|---|---|
| 10 | `adv_registry.rs:281` `adv_character_sets_are_held` | m01 |
| 10 | `:302` `adv_timestamps_are_held` | m02 |
| 10 | `:314` `adv_decimals_are_held` | m03 |
| 10 | `:327` `adv_unknown_parent_is_refused` | m04 |
| 10 | `:335` `adv_value_listed_twice_is_refused` | m05 |
| 10 | `:340` `adv_rev_below_one_is_refused` | m06 |
| 10 | `:345` `adv_dimension_declared_twice_is_refused` | m07 |
| 11 | `:360` `adv_every_field_loads_as_written` | m11 to m21 |

### 2c. Mutation runs

All runs used a scratch copy, never the worktree. The copy has since been cleaned and deleted.

Run 1 ran the unit's suite, `--test registry --lib`. Every mutant survived except m09, which did not build:

| mutant | change |
|---|---|
| m01 | `Errors::alphabet` disabled |
| m02 | `Errors::timestamp` disabled |
| m03 | `Errors::decimal` disabled |
| m04 | `UnknownParent` disabled |
| m05 | `DuplicateValue` disabled |
| m06 | `rev < 1` changed to `rev < 0` |
| m07 | `DuplicateDimension` disabled |
| m08 | `if sampled` changed to `if true` |
| m10 | the `since.rs:153` guard disabled |
| m11 | `source_only` mapped to `derived` |
| m12 | `otel_attribute` mapped to `None` |
| m13 | `count_of_cases` mapped to `gauge` |
| m14 | `min_n` mapped to 0 |
| m15 | `lower_of` mapped to `higher_of` |
| m16 | `min_events` mapped to `None` |
| m17 | query text mapped to empty |
| m18 | `history: source` mapped to `store` |
| m19 | `links.components` mapped to empty |
| m20 | `revised_by` mapped to empty |
| m21 | YAML `true` mapped to `false` |

Run 2 ran the 8 catchers. On the unmutated copy: `test result: ok. 8 passed; 0 failed`. Against the mutants, m01 to m07 and m11 to m21 were all killed; m08 and m10 survived. Logs: `mutants.log`, `mutants-catchers.log`, `mutants-control.log`.

## 3. Suite run

The suite ran after the cases existed.

Gate: `task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, run from the worktree. Exit 201. Log: `scratch/adversary/task-check.log`.

| step | result |
|---|---|
| specify validate | `thalamus v1 — 7 file(s), valid` |
| drift | `generated/ and spec/suite.json match spec/` |
| fmt `--check` | exit 0 |
| clippy `-D warnings` | `Finished`, exit 0 |
| test | stops at the first red binary: `test result: FAILED. 8 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out` (adv_registry), then `task: Failed to run task "check": task: Failed to run task "test": exit status 101` |
| docs-check | not reached |

Count: `cargo test --locked --workspace --no-fail-fast`, exit 101. Log: `scratch/adversary/suite-no-fail-fast.log`.

- 54 binaries ran: 329 passed, 12 failed, 341 executed.
- Before: 321 in the implementor's `scratch/task-check.log`, 52 binaries. 321 + 20 new cases = 341.
- The 12 failures:
  - `adv_registry`: `test result: FAILED. 8 passed; 11 failed`
  - `adv_registry_window`: `test result: FAILED. 0 passed; 1 failed`

## 4. Findings

Every finding covers the uncommitted tree on base ca445f1. All are `introduced`: `registry.rs`, `since.rs` and `series.rs` are one empty line at ca445f1.

| # | file:line | verdict | sev | what was measured | what reaches it | fix |
|---|---|---|---|---|---|---|
| 1 | `crates/thalamus-core/src/registry.rs:499` (also `:549`) | NEEDS-CHANGE | warning | With duplicate mapping keys, the last entry wins with no error. A `file:` key silently becomes `open`, and the closed registry stops checking values. A duplicate query name drops one text that would be hashed into provenance. | An operator's `dimensions.yaml` or definition file, through `Instance::load`. `story:m1-cli-sample` loads them. | Deserialize both maps through a visitor that refuses a repeated key. serde_yaml_ng's own `Mapping` does: the probe got `duplicate entry with key "a"` on `objective`. |
| 2 | `crates/thalamus-core/src/registry.rs:485` | NEEDS-CHANGE | warning | `is_connection_id` refuses only `://` and a leading `/`. It admits `https:host/api`, `https:/host/api` and `file:/path`, which a WHATWG parser and RFC 8089 read as URLs. | Any definition's `source.connection`, passed to Connectors by every adapter. | Also refuse `:/` anywhere, and the scheme prefixes `http:`, `https:` and `file:`. |
| 3 | `crates/thalamus-core/src/registry.rs:479` | NEEDS-CHANGE | warning | `is_signal_id` does not compare the `<family>` segment with `family`, so `service.api.x` loads under `family: platform`. | An operator file. Every consumer that groups by family. | Refuse when the first segment differs from `family`. |
| 4 | `crates/thalamus-core/src/registry.rs:425` | NEEDS-CHANGE | warning | `iso_duration` admits `Y` and `M` date parts and counts past `i64` seconds. `window_ending` refuses all of them, so such a definition loads `Active` and reads unknown on every pass. | All five adapters call `window_ending(&definition.window, …)`: prometheus:72, sql:84, git:124, kubernetes:72, http_probe:96. | Refuse `Y`/`M` date components and any total past `i64` seconds. |
| 5 | `crates/thalamus-core/src/registry.rs:425` | CONFIRMED | note | `0h`, `PT0S` and `P0D` load as `window` and `cadence`. A zero window is `[end, end]`, an empty population. | `window_ending` accepts it. Nothing reads `cadence` yet (scheduler: `story:m2-scheduler`). | Refuse a zero total. |
| 6 | `crates/thalamus-core/src/registry.rs:717` | CONFIRMED | warning | The parent check asks only "is it in the file", so a self-parent and a two-key cycle load. The spec's `RegisterDimension` refuses a parent not registered yet, so no command sequence can build either. | An operator's `dimensions.yaml`. Nothing walks parents yet (L9 derivation). | Refuse a key that reaches itself through `parent`. |
| 7 | `crates/thalamus-core/src/registry.rs:695` | CONFIRMED | warning | `file:/etc/passwd`, `file:../…` and `file:https://example/x` load. The spec's `ValueSource` says "a registry file in the instance", and design § 5.2 says an instance is a directory. | `story:m1-cli-sample` reads `<instance>/` + locator. `Path::join` with an absolute locator reads the host path. | Refuse an absolute locator, `..` segments and URL schemes. |
| 8 | `crates/thalamus-core/src/registry.rs:1125` (`json`, `:1120`) | INFEASIBLE | warning | `.nan`, `.inf` and `-.inf` become `Json::Number(".nan")` and the like. Nesting 65 to about 127 deep loads. `push_value` then writes JSON that `json::parse` refuses. Its doc says "a caller building one by hand owns its spelling". | Nothing found today. No path serializes an `Instance` definition: the conformance server uses its own ports. | Refuse non-finite numbers and nesting past 64 in `json()`. |
| 9 | `crates/thalamus-core/src/registry.rs:1261` | INFEASIBLE | warning | Generated `retire_definition` over `Instance` answers `Retired` with a `DefinitionRetired` event while the file stays `Active`. The refusal is only an entry in `refused_writes()`. The story's Work says a write is "refused with a named error". The unit's own `definition_ports_refuse_writes` (`tests/registry.rs:894`) checks state but not the answer ("Whatever the command answered"). | Nothing found. `story:m2-auto-unmute` says "this story issues no command against them", and no service wires `Generated<Instance>`. | None inside the generated `put(&mut self, _)`, which cannot fail. The options are to panic on write or to stop exposing `Generated<Instance>`. This needs the coordinator. |
| 10 | `crates/thalamus-core/tests/registry.rs:1033` | CONFIRMED | warning | The unit's suite has no case for these rules, so mutants m01 to m07 leave it green: character sets, timestamps (`revised_at`, `created`), decimals, `UnknownParent`, `DuplicateValue`, `rev >= 1` and `DuplicateDimension`. | The rules guard every operator file. | Catchers added (part 2b). They kill m01 to m07. |
| 11 | `crates/thalamus-core/tests/registry.rs:1033` | CONFIRMED | warning | The unit's suite does not read back these fields, so mutants m11 to m21 leave it green: `rule`, `otel_attribute`, `count_of_cases`, `min_n`, `lower_of`, `min_events`, query text, `history`, `links`, `revised_by` and objective booleans. | Every consumer of the loaded definition. | `adv_every_field_loads_as_written` added. It kills all eleven. |
| 12 | `crates/thalamus-core/src/since.rs:130` | CONFIRMED | note | The loader admits a fourth form, `<key>.created`, and the fixture uses it. Design § 11 D18 names only `dimension.created`. `signals.md` documents the new form, but D18 does not. | The implementor's stated call. | Add the form to D18, or drop it. Decide at the design, not here. |
| 13 | `crates/thalamus-core/src/registry.rs:1043` | CONFIRMED | note | No case holds "the baseline plan runs only for Active and Muted": mutant m08 (`if true`) survives both runs. | The implementor's stated call. | A case where a `Proposed` gauge with `pooled` loads. |

## 5. Attacked and not broken

- **YAML hostility:**
  - Alias bomb: refused, `repetition limit exceeded`, in 0.32 s.
  - Nesting of 20000 or 200: refused, `recursion limit exceeded`.
  - Merge keys, unknown fields and a duplicate struct field: refused.
  - Multi-document and an empty file: refused.
  - A 20 MB title loads in 1.2 s.
  - Tags are ignored.
- **Ids:** these are refused:
  - Unicode (Cyrillic `і`, fullwidth dot)
  - uppercase
  - an empty segment
  - two or four segments
- **UCUM:** `unit` and `denominator.unit` both go through `parse_ucum`, in every lifecycle state.
- **Dimensions:** keys that differ only in case or whitespace are refused. Values are matched exactly. Empty versus absent behaves as D8 says.
- **Lifecycle:** Proposed, Muted and Retired are refused for sampling through the generated `RecordSample`.
- **D17:** `connectors:`, `Connectors:`, `Open`, `file:` and `connectors:` are each refused.
- **`resolve_since`:** stays at or after retention start, including with offsets. An unparseable retention start gives `None`.
- **D10 series key:** vectors hold, including escapes and key order.
- **Durations:** `1.5d`, `-6h`, `6H` and `PT1.5S` are refused.

## 6. Paths written outside the worktree

All are under `~/.cache/thalamus-waves/20261015d/m1-instance-registry/scratch/`:

- `adversary/adv-registry-red.log`
- `adversary/adv-registry-window-red.log`
- `adversary/adv-objective-nesting-red.log`
- `adversary/mutants.log`
- `adversary/mutants-catchers.log`
- `adversary/mutants-control.log`
- `adversary/task-check.log`
- `adversary/suite-no-fail-fast.log`
- `adversary-pass-1.md` (this report)

Already deleted:

- `adversary/mutants/`: the scratch copy, after `cargo clean`
- `adversary/mutate.sh`

The worktree session lease `adversary-m1-instance-registry-p1` was acquired and then released.

## 7. Findings block

```findings
- file: crates/thalamus-core/src/registry.rs
  line: 499
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a dimension key or query name repeated in a YAML mapping loads silently as its last entry, so a file registry key can become open
- file: crates/thalamus-core/src/registry.rs
  line: 485
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: source.connection admits https:host/path, https:/host/path and file:/path, which URL parsers read as URLs
- file: crates/thalamus-core/src/registry.rs
  line: 479
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the id's <family> segment is not compared with the family field
- file: crates/thalamus-core/src/registry.rs
  line: 425
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: windows with years, months or more than i64 seconds load while every adapter's window_ending refuses them, so the definition reads unknown on every pass
- file: crates/thalamus-core/src/registry.rs
  line: 425
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a zero window or cadence (0h, PT0S, P0D) loads
- file: crates/thalamus-core/src/registry.rs
  line: 717
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a dimension that is its own parent, or a parent cycle, loads although RegisterDimension cannot build either
- file: crates/thalamus-core/src/registry.rs
  line: 695
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: file registry locators outside the instance directory (absolute, .., URL) load
- file: crates/thalamus-core/src/registry.rs
  line: 1125
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: objective NaN or infinity, or nesting past 64, loads and is written as JSON the model's reader refuses
- file: crates/thalamus-core/src/registry.rs
  line: 1261
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: generated RetireDefinition over the file-backed ports answers Retired while nothing changes; the refusal is only a side log
- file: crates/thalamus-core/tests/registry.rs
  line: 1033
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no case states the character set, timestamp, decimal, UnknownParent, DuplicateValue, rev and DuplicateDimension rules; mutants m01 to m07 survive
- file: crates/thalamus-core/tests/registry.rs
  line: 1033
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: eleven loaded fields are never read back; mutants m11 to m21 survive
- file: crates/thalamus-core/src/since.rs
  line: 130
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: denominator.since admits <key>.created, a form design § 11 D18 does not name
- file: crates/thalamus-core/src/registry.rs
  line: 1043
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no case holds that the baseline plan is skipped for Proposed and Retired definitions; mutant m08 survives
```
