---
format: aep.planning-md/3
id: story:m1-eventlog-store
kind: story
status: draft
title: Samples and evaluations persist in one Eventlog stream per instance (File, SQLite)
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m0-conformance-records
scope:
- confidence: inferred
  path: crates/thalamus-store/Cargo.toml
- confidence: inferred
  path: crates/thalamus-store/src/evaluations.rs
- confidence: inferred
  path: crates/thalamus-store/src/eventlog.rs
- confidence: inferred
  path: crates/thalamus-store/src/range.rs
- confidence: inferred
  path: crates/thalamus-store/src/samples.rs
- confidence: inferred
  path: crates/thalamus-store/tests/conformance.rs
- confidence: inferred
  path: crates/thalamus-store/tests/store.rs
revision: 4
---
## Outcome

Samples and evaluations an instance records survive a restart in one append-only Eventlog stream per instance, on the File or the SQLite provider, and come back per series in time order.

## Context

`epic:signal-contract`: "`thalamus-store` on Eventlog". Design § 4 (append-only stream per instance; stored lines are not CloudEvents-wrapped, § 6.2), § 5.1 (first provider Eventlog), § 10 question 1 (default: Eventlog, SQLite provider), § 11 D5. Eventlog is a Rust library with File, SQLite and PostgreSQL providers, pinned by bare Git tag (`0.9.0` today, its README). The File provider writes JSONL, which `story:m1-live-pass` measures in lines and bytes. `story:m0-conformance-records` meets the `RecordSample` / `RecordEvaluation` obligations over in-memory storage (`depends_on`); this story puts Eventlog behind the sample and evaluation ports.

The crate, its `lib.rs` module lines and its member line in the root `Cargo.toml` are `story:m0-workspace`'s; this story touches no `members` line and no `lib.rs`. It adds Eventlog to `crates/thalamus-store/Cargo.toml` only; the resulting `Cargo.lock` change, and the one `story:m1-connectors-client` makes to it, are resolved by the coordinator at merge by regenerating the lock. Each store port is its own file, so the M2 stories that add ports (`story:m2-transitions`, `story:m2-rollup`, `story:m2-auto-unmute`) add files rather than edit these.

## Work

- `crates/thalamus-store/src/eventlog.rs`: opening the one stream of an instance, provider chosen by a locator (`file:<dir>` or `sqlite:<path>`); records are the wire forms `thalamus.sample/1` and `thalamus.evaluation/1` of design § 2.2 and § 2.3; the shared append and read helpers the port files use.
- `crates/thalamus-store/src/samples.rs`: the sample port and the generated `SignalSampleStorage` (`thalamus_model::behaviour`, inferred from `ess generate synthesize --target rust` at ess 0.59.0) over it; read a sample by id; the first stored sample of a series.
- `crates/thalamus-store/src/evaluations.rs`: the evaluation port and the generated `SignalEvaluationStorage` over it. D5: a second evaluation for the same `(sample_id, definition_rev)` is refused inside the append, with a stable refusal code, and nothing is written. Reads: the newest evaluation of each series; the samples with no evaluation at a given definition revision.
- `crates/thalamus-store/src/range.rs`: the series range read, the samples and the evaluations of one series in `ts` order over a time range (the `thalamus.observation.SeriesSamples` and `thalamus.evaluation.SeriesVerdicts` views). `story:m2-compare`, `story:m2-store-baselines` and the hysteresis replay read history through it and add no read of their own.
- Numbers: `Decimal` values round-trip exactly. Eventlog keeps `serde_json/arbitrary_precision` off its workspace because it changes every stored `request_hash` (inferred from the comment on `eventlog-capture-types` in its root `Cargo.toml`); this story checks the feature set of `thalamus-model` against that before the first append.
- `crates/thalamus-store/tests/conformance.rs`: runs the named scenarios of `spec/suite.json` with the Eventlog storage in place of the in-memory one, from one selection constant that later stories extend by one line each.

## Domain relations

- `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample`, one, references via `sample_id`; the sample is stored first. Inferable: `spec/domains/evaluation.yaml`, entity `SignalEvaluation`, relation `sample`; outcome `unknown-sample` of `RecordEvaluation`.
- At most one evaluation per sample and definition revision. Inferable: `spec/domains/evaluation.yaml:224` (`UNMAPPED:`, decided as design § 11 D5, held by a store constraint and a Rust test).
- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`; samples and evaluations outlive a retired definition. Inferable: `spec/domains/observation.yaml`, relation `definition`; `spec/domains/definition.yaml`, `RetireDefinition` summary.
- Samples and evaluations have no lifecycle beyond `Recorded`. Inferable: `spec/domains/observation.yaml` and `spec/domains/evaluation.yaml`, lifecycle `initial: Recorded`, `terminal: [Recorded]`.

## Decisions taken (reversible)

- File and SQLite in M1; PostgreSQL (hosted composition: verified TLS, separate migration authority, connection budget, per the Eventlog README) is left for an in-cluster deployment (design § 10 question 2).
- One stream per instance holding every record kind.

## Scope

Planned (inferred):

- `crates/thalamus-store/Cargo.toml` (Eventlog; `Cargo.lock` is regenerated by the coordinator at merge)
- `crates/thalamus-store/src/eventlog.rs`, `crates/thalamus-store/src/samples.rs`, `crates/thalamus-store/src/evaluations.rs`, `crates/thalamus-store/src/range.rs`
- `crates/thalamus-store/tests/store.rs`, `crates/thalamus-store/tests/conformance.rs`

## Acceptance

`cargo test -p thalamus-store` passes, on both the File and the SQLite provider: the conformance scenarios `thalamus.observation.RecordSample/outcome/recorded`, `thalamus.evaluation.RecordEvaluation/outcome/recorded`, `thalamus.evaluation.RecordEvaluation/outcome/unknown-sample` and `thalamus.evaluation.SignalEvaluation/invariant/after/thalamus.evaluation.RecordEvaluation/recorded` from `spec/suite.json` with the Eventlog storage in place of the in-memory one, and the tests `second_evaluation_at_same_rev_is_refused` (written and seen failing first), `reopened_store_reads_series_in_ts_order`, `range_read_returns_samples_and_evaluations_of_one_series`, `first_stored_sample_of_series`, `unknown_reading_round_trips_without_value`, `decimal_round_trips_exactly` and `stored_line_is_not_cloudevents_wrapped`.

## Out of scope

Retention (35 days) and daily rollups (design § 4; M2); PostgreSQL; CloudEvents on the wire (M2); transitions and definition events (M2).

## Revision after design critic round 2

The store conformance test reuses the runner: `crates/thalamus-store/tests/conformance.rs` calls `thalamus_core::conformance::run` (dev-dependency `thalamus-core` with feature `conformance`) with Eventlog storage. It writes no runner of its own.
