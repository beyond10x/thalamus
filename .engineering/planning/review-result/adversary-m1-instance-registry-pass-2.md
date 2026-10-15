---
format: aep.planning-md/3
id: review-result:adversary-m1-instance-registry-pass-2
kind: review-result
status: active
title: Adversary, story:m1-instance-registry, pass 2
relations:
- reviews: story:m1-instance-registry
revision: 1
---
unit: story:m1-instance-registry, uncommitted tree ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15d-m1-instance-registry (branch impl/m1-instance-registry, base ca445f1), after the pass-1 corrections
verdict: CONFIRMED
cases: executed 349→365, red 5
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths under ~/.cache/thalamus-waves/20261015d/m1-instance-registry/scratch/ (part 6)
needs-coordinator: yes, finding 2. The implementor's test asserts that `P1W2D` loads and is stored as written. The generated schemas give every `Duration` the JSON Schema `format: duration`, whose grammar (RFC 3339 Appendix A) refuses `P1W2D`. Someone has to decide which grammar a stored `Duration` follows.

## 1. Diff

`git --no-pager diff --stat` is the same as the tree I was handed: 9 tracked files, 2073 insertions and 22 deletions, all of them the implementor's or the coordinator's.

```
 Cargo.lock                           |   49 ++
 crates/thalamus-core/Cargo.toml      |    2 +
 crates/thalamus-core/src/registry.rs | 1610 ++++++++++++++++++++++++++++++++++
 crates/thalamus-core/src/series.rs   |  172 ++++
 crates/thalamus-core/src/since.rs    |  215 +++++
 crates/thalamus-docs/src/status.rs   |    6 +-
 docs/design/thalamus.md              |    2 +-
 website/data/status.json             |    4 +-
 website/docs/concepts/signals.md     |   35 +-
 9 files changed, 2073 insertions(+), 22 deletions(-)
```

My changes are two new untracked test files. I changed no implementation file. `rustfmt` ran on these two files only. `b10x-gates scan-text` on them: `2 file(s) carry no private rule`.

| file | lines |
|---|---|
| `crates/thalamus-core/tests/adv_registry_p2.rs` | 630 |
| `crates/thalamus-sources/tests/adv_registry_window_p2.rs` | 63 |

## 2. Cases added

16 cases. Each file was run alone before any suite run. Logs: `adv-registry-p2-red.log`, `adv-registry-window-p2-red.log` and `adv-since-catchers.log`, all in `scratch/adversary-p2/`.

### 2a. Red now (5)

| # | case | asserts | red output, verbatim (after `panicked at …:`) |
|---|---|---|---|
| 1 | `thalamus-sources/tests/adv_registry_window_p2.rs:36` `adv_p2_an_admitted_long_window_is_placed_at_a_present_end` | A window the loader admits is placed by `window_ending` at a 2026 end. | `window P800000D loaded as "P800000D"; ending 2026-10-14T22:57:03Z the adapters refuse it: window "P800000D" reaches before any representable instant` (also `P4000000D` and `P110000W`) |
| 2 | `adv_registry_p2.rs:454` `adv_p2_every_admitted_duration_is_an_rfc_3339_duration` | Every admitted `window` and `cadence` is stored as an RFC 3339 App. A `duration`, the `format: duration` of `generated/spec-types/source.schema.json` and `thalamus-service.openapi.json`. | `durations outside RFC 3339 Appendix A loaded:` / `window "P1W2D" loaded as "P1W2D"` / `window "PT1H1S" loaded as "PT1H1S"` / `window "P1DT1H1S" loaded as "P1DT1H1S"` / `window "P1W1DT1H1M1S" loaded as "P1W1DT1H1M1S"`, and the same for `cadence` |
| 3 | `adv_registry_p2.rs:355` `adv_p2_the_served_definitions_view_reads_back` | The `Definitions` view that `Generated<Instance>` answers, served as `{"rows":[…]}` the way `thalamus_service.rs` `run_thalamus_definition_definitions` serves it, parses with `json::parse`. | `objective nested 62 deep loaded; the served Definitions body is refused: at byte 653: expected a document nested less than 64 deep` (the same at 64; 61 passes) |
| 4 | `adv_registry_p2.rs:104` `adv_p2_every_special_url_scheme_is_refused` | The WHATWG special schemes left out of the correction are refused. | `connection ids a WHATWG parser reads as URLs loaded: ["ftp:metrics.example/api", "ws:metrics.example/api", "wss:metrics.example/api", "WSS:metrics.example"]` |
| 5 | `adv_registry_p2.rs:153` `adv_p2_a_locator_naming_a_directory_is_refused` | A locator naming a directory is refused, as `registries/` already is. | `locators naming a directory loaded: [".", "registries/."]` |

### 2b. Green now (11)

These hold the corrections. Several also catch mutants (part 2c).

| case | holds |
|---|---|
| `:81` `adv_p2_connectors_connection_refs_load` | These Connectors refs load: `conn_<32hex>_<32hex>` (the form `connectors-host/src/local/registry/lifecycle.rs:364` mints), `forge-local/connection-1`, `configured-main`, `connection-1`, `conn-team`, `conn_child_7` and `c1` |
| `:128` `adv_p2_nested_and_unicode_locators_load` | These locators load and are kept as written: `registries/teams.yaml`, `registries/2026/teams.yaml`, `registries/équipes.yaml`, `チーム.yaml`, `registries/..teams.yaml` |
| `:176` `adv_p2_each_locator_clause_refuses_on_its_own` | Each clause of `is_instance_path` alone gives a `LocatorOutsideInstance` naming the locator |
| `:207` `adv_p2_repeated_keys_are_refused_at_every_depth` | A repeated key is refused in each of these places: a nested map, a map inside a sequence, `source`, `links`, `evaluation.warn`, `denominator`, the registry entry's fields, and a `FileValues` entry |
| `:252` `adv_p2_parent_chains_of_three_and_more` | For parent chains: <ul><li>a cycle through every key, of 3 and of 4 keys, is refused</li><li>a lead-in key is not flagged</li><li>a chain into an unregistered key gives only `UnknownParent`</li><li>a 5-key chain loads</li></ul> |
| `:326` `adv_p2_objective_depth_matches_the_reader_at_64_and_65` | 64 levels load and read back, 65 are refused. This holds through maps, through sequences, and through a YAML tag |
| `:513` `adv_p2_duration_forms_outside_the_grammar_are_refused` | Each of these is refused as `Rule::Duration`: <ul><li>`PT0.5S` and `PT0,5S`</li><li>`pt6h`, `p1d`, `P1d` and `Pt6H`</li><li>`6H` and `1D`</li><li>`+6h`, `6 h`, `6hh` and `h6`</li><li>`P-1D` and `PT-1H`</li><li>`P1DT`</li><li>`P1D1D`, `PT1S1S` and `PT1H1H`</li></ul> |
| `:537` `adv_p2_the_signals_page_example_loads` | The example in `signals.md`, read verbatim from the page, loads with the values the page shows |
| `:579` `adv_p2_a_key_named_dimension_is_named_by_its_own_created` | Under D18 clause 1, a declared `file` key named `dimension` is what `dimension.created` names |
| `:618` `adv_p2_a_dotted_name_is_no_form` | `component.x.created` is `SinceRefusal::Form` |
| `window_p2.rs:48` `adv_p2_admitted_edges_are_placed` | `6h`, `1d`, `1w`, `P1W2D`, `PT1S` and `0006h` are each placed at the exact start expected |

### 2c. Mutation runs

All mutation runs used a scratch copy, never the worktree. The copy has been `cargo clean`ed and deleted.

Runs:

- Control on the unmutated copy: `--lib` 9 ok, `adv_registry` 19 ok, `registry` 29 ok, and `adv_registry_p2` with only the 4 red cases of part 2a failing.
- 25 mutants against `--lib --test registry --test adv_registry --test adv_registry_p2`.
- Log: `mutants.log`.

Kills:

- The unit's suite kills m01–m03, m05–m07, m09–m18 and m22.
- Only adversary cases kill these 5 mutants:

| mutant | change | killed by |
|---|---|---|
| m08 | drop `!segment.is_empty()`, so `registries/` and `a//b` load | `adv_p2_each_locator_clause_refuses_on_its_own` |
| m21 | a YAML tag counts as one level | `adv_p2_objective_depth_matches_the_reader_at_64_and_65` |
| m23 | `next += at` (from `at + 1`), so `P1D1D` and `PT1H1H` load, which `window_ending` refuses | `adv_p2_duration_forms_outside_the_grammar_are_refused` |
| m24 | `since.rs:153`: drop `name.contains('.')` | `adv_p2_a_dotted_name_is_no_form`. The re-run is in `mutants-since.log`: `test adv_p2_a_dotted_name_is_no_form ... FAILED` |
| m25 | `since.rs:137`: drop `&& !is_declared(name)` | `adv_p2_a_key_named_dimension_is_named_by_its_own_created`. The re-run is in `mutants-since.log`: `... FAILED` |

Equivalent mutants, which survive everything:

- m04: the leading-`/` clause. The empty-segment clause already refuses a leading `/`.
- m19 and m20: `is_finite` and `is_json_number`. Each makes the other redundant.

## 3. Suite run

The suite ran after the cases existed.

Gate: `task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, run from the worktree. Exit 201. Log: `scratch/adversary-p2/task-check.log`.

| step | result |
|---|---|
| specify validate | `thalamus v1 — 7 file(s), valid` |
| drift | `generated/ and spec/suite.json match spec/` |
| fmt `--check` | passed |
| clippy `-D warnings` | `Finished`, passed |
| test | `test result: FAILED. 10 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out` (adv_registry_p2), then `task: Failed to run task "check": task: Failed to run task "test": exit status 101` |
| docs-check | not reached |

Count: `cargo test --locked --workspace --no-fail-fast`, exit 101. Log: `scratch/adversary-p2/suite-no-fail-fast.log`.

- 56 binaries ran: 360 passed, 5 failed, 365 executed.
- Before: 349 in 54 binaries, from the implementor's `scratch/task-check-round2.log`. 349 + 16 = 365.
- The failures:
  - `adv_registry_p2`: `test result: FAILED. 10 passed; 4 failed`
  - `adv_registry_window_p2`: `test result: FAILED. 1 passed; 1 failed`
- All 19 pass-1 cases in `adv_registry` now pass.

## 4. Findings

Every finding covers the uncommitted tree on base ca445f1. All are `introduced`.

| # | file:line | verdict | sev | what was measured | what reaches it | fix |
|---|---|---|---|---|---|---|
| 1 | `crates/thalamus-core/src/registry.rs:540` | CONFIRMED | note | `placeable` bounds at the `time` crate's year -9999. `window_ending` writes the start as RFC 3339, so its year must be 0000 or later. Windows of about 1970 to 11 969 years load, and every adapter refuses them (case 1). The doc comment at `:538` claims the bound is the one `window_ending` writes. The unit test asserts `placeable(11_000 * 365 * 86_400)`. | Only an operator file with a window of millennia. | Bound at 0000-01-01T00:00:00Z (719 528 days before 1970), not year -9999. |
| 2 | `crates/thalamus-core/src/registry.rs:482` | INFEASIBLE | note | `P1W2D`, `PT1H1S`, `P1DT1H1S` and `P1W1DT1H1M1S` load and are stored verbatim. The generated schemas declare `format: duration`, whose grammar (RFC 3339 App. A) refuses all four (case 2). `window_ending` accepts them, so the adapters are unaffected. | Nothing today. `Generated<Instance>`'s `Definitions` view would carry them, but no service serves an `Instance` (`story:m4-service-api`). | Needs the coordinator. Either store a canonical form (`P9D`, `PT3601S`), or refuse weeks mixed with other units and time parts with gaps. Otherwise record that `Duration` follows `window_ending`'s grammar, not the schema's. |
| 3 | `crates/thalamus-core/src/registry.rs:1281` | INFEASIBLE | note | The 64-level limit holds for the objective alone. The served `Definitions` body puts it 3 levels deep, so an objective of 62 to 64 levels yields a body that `json::parse` refuses (case 3). | Nothing today, for the same reason as finding 2. | Cap at 61 levels, or document that the limit counts the objective alone. |
| 4 | `crates/thalamus-core/src/registry.rs:554` | CONFIRMED | note | The correction refuses `http:`, `https:` and `file:` but not the other WHATWG special schemes. `ftp:host/p`, `ws:host/p` and `wss:host/p` load (case 4). Connectors never mints such a ref: it mints `conn_<hex>_<hex>`, and none of its fixtures holds a `:`. | Only an operator file. | Add `ftp:`, `ws:` and `wss:` to `URL_SCHEMES`. |
| 5 | `crates/thalamus-core/src/registry.rs:568` | CONFIRMED | note | `file:.` and `file:registries/.` load, while `file:registries/` is refused. Both name a directory, not a registry file (case 5). | An operator's `dimensions.yaml`. `story:m1-cli-sample` would then read a directory. | Refuse a `.` segment, or at least a final one. |
| 6 | `crates/thalamus-core/tests/registry.rs:1257` | CONFIRMED | note | The unit's suite leaves 5 mutants of the new checks green: m08, m21, m23, m24 and m25 (part 2c). m23 admits `P1D1D`, which the adapters refuse. | Each guards an operator file. | Adversary cases now kill all 5 (part 2b). |

## 5. Attacked and not broken

- **Connection ids:**
  - Every connection form Connectors mints or uses as a fixture loads: `conn_<hex>_<hex>`, `forge-local/connection-1` and `configured-main`.
  - Connectors types `connection` as an unconstrained `String` (`ess/domains/cli.yaml:420`). No ref containing `:` exists in its source or contracts.
- **Duration grammar:**
  - `PT0.5S`, the lower-case and mixed-case forms, `6H`, `1D`, signs, `P1DT` and repeated units are refused.
  - The adapters place `6h`, `1d`, `1w`, `P1W2D` and `0006h` exactly.
  - The loader and `window_ending` agree on every form except finding 1's lengths.
- **`file:` locators:**
  - Nested, unicode and dotted-name locators load as written.
  - Each refusal clause works alone, apart from the leading-`/` clause (m04), which the empty-segment clause makes redundant.
- **Duplicate keys:** refused at every depth: nested maps, maps inside sequences, struct-shaped entries, the registry entry's fields and `FileValues` entries.
- **Parent chains:**
  - Cycles of 3 and 4 keys are refused, including one through every key.
  - A lead-in key is not flagged.
  - A chain into an undefined key gives only `UnknownParent`.
  - A 5-key chain loads.
- **Objective depth:** 64 loads and round-trips through `json::parse`, and 65 is refused, for maps, sequences and tags. Non-finite numbers are refused.
- **`signals.md`:** the example loads verbatim. The `id`, `source`, `window`/`cadence`, `denominator.since`, `evaluation`, `file`-locator and `parent` rows match the code.
- **D18 as amended:** matches `since.rs`, including the case of a key named `dimension`.

## 6. Paths written outside the worktree

All are under `~/.cache/thalamus-waves/20261015d/m1-instance-registry/scratch/`:

- `adversary-p2/adv-registry-p2-red.log`
- `adversary-p2/adv-registry-window-p2-red.log`
- `adversary-p2/adv-since-catchers.log`
- `adversary-p2/mutants-control.log`
- `adversary-p2/mutants.log`
- `adversary-p2/mutants-since.log`
- `adversary-p2/task-check.log`
- `adversary-p2/suite-no-fail-fast.log`
- `adversary-pass-2.md` (this report)

Already deleted:

- `adversary-p2/mutants/`: the scratch copy, after `cargo clean` (1.2 GiB)
- `adversary-p2/mutate.sh`
- `adversary-p2/mutate-since.sh`

The worktree session lease `adversary-m1-instance-registry-p2` was acquired and then released.

## 7. Findings block

```findings
[
  {"file": "crates/thalamus-core/src/registry.rs", "line": 540, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "placeable bounds windows at year -9999 while window_ending can only write a start from year 0000, so windows of about 1970 to 11969 years load and every adapter refuses them"},
  {"file": "crates/thalamus-core/src/registry.rs", "line": 482, "category": "contract-drift", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "P1W2D, PT1H1S, P1DT1H1S and P1W1DT1H1M1S load and are stored verbatim, outside the RFC 3339 format: duration the generated schemas declare for Duration"},
  {"file": "crates/thalamus-core/src/registry.rs", "line": 1281, "category": "contract-drift", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "an objective of 62 to 64 levels loads, and the served Definitions view body carrying it is refused by json::parse"},
  {"file": "crates/thalamus-core/src/registry.rs", "line": 554, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "connection ids with the WHATWG special schemes ftp:, ws: and wss: load although the correction refuses http:, https: and file:"},
  {"file": "crates/thalamus-core/src/registry.rs", "line": 568, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "file:. and file:registries/. load as locators although they name a directory, while file:registries/ is refused"},
  {"file": "crates/thalamus-core/tests/registry.rs", "line": 1257, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "five mutants of the new checks (empty-segment locator clause, tag depth, repeated duration designator, dotted since name, since key named dimension) leave the unit's suite green"}
]
```
