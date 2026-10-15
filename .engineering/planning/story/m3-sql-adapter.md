---
format: aep.planning-md/3
id: story:m3-sql-adapter
kind: story
status: active
title: SQL reads become typed samples; NULL and unparseable cells are unknown, never 0
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/sql.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/sql
- confidence: inferred
  path: crates/thalamus-sources/tests/sql.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T04:58:56Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-15T04:58:56Z", actor: "human:timo", revision: 7}
---
## Context

Design § 5.1 lists SQL among the `thalamus-sources` adapters, and the § 2.1 example definition reads `source: {adapter: sql, connection: …}` with `extractor: sql.ratio`. § 5.2: thalamus reads only through a Connectors connection, holds no secret and never writes. This story adds the `sql` adapter on the M1 adapter trait and Connectors client (`epic:signal-contract`). Code is Rust (AGENTS.md).

## Connectors

Exists. Connectors v0.45.0 (`94e0805ed`), `adapters/sql/`, profile in `adapters/sql/contracts/reads/v1alpha1/semantics.md`: `query.read` against PostgreSQL or MySQL (per-connection `engine`), in a read-only transaction or session, at most 1,000 rows and 8 KiB of query text, every non-NULL cell as lossless native text with its type name, NULL as JSON null, truncation reported explicitly.

## Behaviour

- `sql.value`: one numeric column per row; one sample per row. A `dims` key is set only from a column the row returns and the definition declares (L9).
- `sql.ratio`: the definition `numerator` and `denominator` queries; the value is their quotient as a `Decimal` with unit `1` (L4); `denominator.n` is the denominator count.
- A NULL cell: `quality: unknown` with a reason, never 0 (L5). A cell whose text does not parse as a number: the same.
- Every read keeps the cell's native text as `raw` (the `Read` field of `story:m1-connectors-client`), so `vocabulary::map_reads` can map a producer value such as a colour (`story:m3-vocabulary-mapping`); a NULL cell has no `raw`.
- Numeric text is parsed into `Decimal` exactly, never through a binary float (design § 11).
- A result the adapter reports as truncated: `partial`, with the bound the extractor states (`lower` for a count).
- A Connectors refusal or provider failure: `quality: unknown`; the reason carries the Connectors code and stage.
- `provenance.query_sha256` is the SHA-256 of the exact query text; with two queries, of their concatenation in the order `numerator`, `denominator` (inferred; the implementor may choose another stable rule and state it).

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one; the sample references the definition, which exists first and outlives it. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `dims` keys → registered `Dimension`: no `relations:` entry; `spec/domains/observation.yaml` field `dims` carries an `UNMAPPED:` marker, decided by design § 2.4 and held by an M1 Rust test. Relied on, not implemented here.
- `SignalDefinition.source` → a Connectors connection: the typed field `thalamus.definition.Source` in `spec/domains/definition.yaml`, not an entity relation.

## Acceptance

`cargo test -p thalamus-sources --test sql` passes, with these known-answer tests over recorded Connectors answers in `crates/thalamus-sources/tests/fixtures/sql`, each seen failing before the adapter exists: `sql_null_cell_is_unknown_never_zero`, `sql_non_numeric_cell_is_unknown`, `sql_ratio_text_cells_give_exact_decimal_with_unit_1` (numerator `15`, denominator `556`), `sql_truncated_result_is_partial`, `sql_dims_only_from_returned_columns`, `sql_read_keeps_raw_cell_text`, `sql_refused_read_is_unknown_with_reason`.

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-sources/src/sql.rs` (replaces the one-line `adapter()` stub `story:m1-connectors-client` leaves there; the module line is `story:m0-workspace`'s and the registry arm `story:m1-connectors-client`'s, so no other file changes)
- `crates/thalamus-sources/tests/sql.rs` (new)
- `crates/thalamus-sources/tests/fixtures/sql` (new)
