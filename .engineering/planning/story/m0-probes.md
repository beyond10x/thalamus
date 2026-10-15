---
format: aep.planning-md/3
id: story:m0-probes
kind: story
status: implemented
title: 'Answer the design § 8 probes: an unknown reading carries no value, a unit is UCUM'
relations:
- decomposes: epic:specification
- depends_on: story:m0-workspace
- depends_on: story:m0-conformance-definitions
scope:
- confidence: inferred
  path: crates/thalamus-core/Cargo.toml
- confidence: inferred
  path: crates/thalamus-core/src/unit.rs
- confidence: inferred
  path: crates/thalamus-core/tests/probes.rs
- confidence: inferred
  path: docs/design/thalamus.md
- confidence: inferred
  path: spec/domains/definition.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T05:39:23Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-15T05:39:23Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-15T06:37:57Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Context

Design § 8 names two constructs to probe before anything is built on the model:

1. `quality: unknown` ⇔ no value. Answered in ESS: `thalamus.observation.Reading` (`spec/domains/observation.yaml:73`) is a union tagged on `quality`; its `unknown` variant is `thalamus.observation.UnreadReading`, which carries `reason` and nothing else (design § 11). This story holds the answer with tests against both generated crates, so a later spec change that reintroduces a value on `unknown` fails the build.
2. A UCUM-typed unit. Not expressible: ESS has no pattern or grammar-typed string, so `thalamus.definition.Unit` (`spec/domains/definition.yaml:35`) holds only the character set, and the `UNMAPPED:` marker at `spec/domains/definition.yaml:32` says the rule is held by a Rust test. This story writes that test and the validator it exercises.

It follows `story:m0-conformance-definitions` (`depends_on`), because both add dev-dependencies to `crates/thalamus-core/Cargo.toml`.

## Work

- `crates/thalamus-core/src/unit.rs`: `parse_ucum(&str) -> Result<UcumUnit, UnitError>` over the case-sensitive UCUM syntax (the unity `1`, annotations `{…}`, metric prefixes, integer exponents, `.` products, `/` quotients, `10*n` factors), atoms and prefixes checked against an embedded table of UCUM case-sensitive codes. Pure: no I/O, no clock. Not yet called from definition validation (`story:m1-instance-registry` calls it).
- `crates/thalamus-core/tests/probes.rs`, against the wire form `generated/spec-types` emits:
  - `unknown_reading_carries_no_value`: a reading with `"quality": "unknown"`, a `reason` and a `value` is refused by `thalamus-spec-types` deserialisation; the same reading without `value` is admitted and re-serialises with no `value` key; the generated model variant for `unknown` is built with an exhaustive struct literal naming `reason` only, so a field added to it breaks the test.
  - `ok_reading_requires_value`: `{"quality": "ok"}` is refused.
  - `ucum_admits_design_units`: `1`, `s`, `By`, `{request}`, `{pod}`, `ms`, `kBy`, `{request}/s`, `m2`, `10*3`.
  - `ucum_refuses_non_units`: the empty string, `{request`, `m^2`, `sec`, `s s`.
- `crates/thalamus-core/Cargo.toml`: dev-dependencies on `thalamus-spec-types` and `serde_json`.
- Edit the `UNMAPPED:` marker at `spec/domains/definition.yaml:32` so it names `crates/thalamus-core/tests/probes.rs` and the two `ucum_*` tests. A comment does not change the model digest; `task drift` stays green.
- Design § 8 records the two answers: probe 1 expressed in ESS, probe 2 held by a Rust test with an `UNMAPPED:` marker.

Not here: reporting the missing grammar type to the ESS repository (listed with the other gaps in design § 11), and rejecting a definition whose unit fails `parse_ucum` (`story:m1-instance-registry`).

## Domain relations

None. This story rests on two types and one field, not on a relation between entities: `thalamus.observation.Reading` and its variants (`spec/domains/observation.yaml:46-80`), the `reading` field of `thalamus.observation.SignalSample` (`spec/domains/observation.yaml:131`), and `thalamus.definition.Unit` (`spec/domains/definition.yaml:35`), the type of `thalamus.definition.SignalDefinition.unit`.

## Scope

Planned (inferred) unless marked:

- `crates/thalamus-core/src/unit.rs` (the module line is `story:m0-workspace`'s)
- `crates/thalamus-core/Cargo.toml` (dev-dependencies; shared with `story:m0-conformance-definitions`, which lands first)
- `crates/thalamus-core/tests/probes.rs`
- `spec/domains/definition.yaml` lines 32-34 (comment only; the file exists)
- `docs/design/thalamus.md` § 8 (the file exists)

## Acceptance

`cargo test -p thalamus-core --test probes` passes the tests `unknown_reading_carries_no_value`, `ok_reading_requires_value`, `ucum_admits_design_units` and `ucum_refuses_non_units`.

## Correction (2026-10-15, from the implementation)

The generated wire format carries the tag beside the variant's own record: an unknown reading is `{"quality":"unknown","value":{"reason":"…"}}`, and `value` is required on every variant (design § 11 D11). `unknown_reading_carries_no_value` therefore checks that the unknown record holds only `reason`, and refuses a number inside it, the flat form, a number in place of the record and a bare `{"quality":"unknown"}`.
