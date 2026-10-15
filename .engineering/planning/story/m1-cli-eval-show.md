---
format: aep.planning-md/3
id: story:m1-cli-eval-show
kind: story
status: draft
title: thalamus eval grades stored samples once per revision; thalamus show exits with the worst current verdict
relations:
- decomposes: epic:signal-contract
- depends_on: story:m1-cli-sample
- depends_on: story:m1-baselines
- depends_on: story:m1-verdict-grading
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/eval.rs
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/show.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/eval_show.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/fixtures/eval_show
- confidence: inferred
  path: crates/thalamus-pipeline/src/evaluate.rs
revision: 3
---
## Outcome

An operator runs `thalamus eval` to grade every stored sample not yet graded at its definition current revision, and `thalamus show` to see the current verdict of every series and exit with the worst one.

## Context

`epic:signal-contract`: "`thalamus eval`, `show`". Design § 2.3 (evaluation, ladder, exit codes), § 3.1, § 5.1, § 11 D5 and D6. Composes `story:m1-verdict-grading`, `story:m1-baselines`, `story:m1-prometheus-adapter` and the store of `story:m1-eventlog-store`. The grade → evaluate half of the pass lives in the library crate `thalamus-pipeline` (`story:m0-workspace`), beside the read → sample half of `story:m1-cli-sample`, so `story:m2-scheduler` calls the same code and `story:m2-store-baselines` changes one grading call site. `epic:history-and-transitions` builds `compare` from stored samples; here an `offset_*` baseline of a `history: source` signal is re-read from the source (`story:m2-compare` relies on that difference).

## Work

- `crates/thalamus-pipeline/src/evaluate.rs`: for every stored sample with no evaluation at the definition current `rev`:
  - baseline windows from `baseline::plan`; for `history: source` each window is read through the extractor registry at the shifted end, for `history: store` the stored samples of the series at the offset are read through the store's series range read; `fixed` reads nothing;
  - the series age is `denominator.age_s` when the sample has one, else the time since the first stored sample of the series;
  - `baseline::fold`, then `grade`; `prev` is the verdict of the newest earlier evaluation of the series (D6); the result is recorded through the generated `RecordEvaluation` over the store with an `evl_` id;
  - a second run records nothing new (D5). Returns the evaluations it recorded.
- `crates/thalamus-cli/src/cmd/eval.rs`: `eval [<signal-id>] [--run <run-id>]` over the pipeline; prints the evaluations it recorded; exits 0 when it recorded what it should; refusals use the codes of `story:m1-cli-sample`.
- `crates/thalamus-cli/src/cmd/show.rs`: `show [<signal-id>] [--format json|text]`: the newest evaluation of each series, worst first, with value, baseline, score and reason; exit code from the ladder (`story:m1-verdict-grading`): worst verdict, and 3 when every shown verdict is `N/A` or nothing is shown.

## Domain relations

- `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample`, one, references via `sample_id`. Inferable: `spec/domains/evaluation.yaml`, relation `sample`.
- `thalamus.evaluation.SignalEvaluation` → `thalamus.definition.SignalDefinition`, one, via `signal` at `definition_rev`. Inferable: `spec/domains/evaluation.yaml`, relation `definition`; at most one evaluation per `(sample, definition_rev)`, `spec/domains/evaluation.yaml:224` (design § 11 D5).
- `prev` is the previous raw verdict of the series. Inferable: `spec/domains/evaluation.yaml:325` (design § 11 D6).
- The current verdict of a series is its newest evaluation. Inferable: `spec/domains/evaluation.yaml:292` (`UNMAPPED:`; `SeriesVerdicts` summary: "its current verdict is the row with the newest ts").

## Scope

Planned (inferred):

- `crates/thalamus-pipeline/src/evaluate.rs`
- `crates/thalamus-cli/src/cmd/eval.rs`, `crates/thalamus-cli/src/cmd/show.rs`
- `crates/thalamus-cli/tests/eval_show.rs`, `crates/thalamus-cli/tests/fixtures/eval_show`

No subcommand enum line: `story:m0-workspace` declares `eval` and `show`.

## Acceptance

`cargo test -p thalamus-cli --test eval_show` passes, running the built binary against a fixture instance, a seeded store and the Connectors double, the tests `unknown_sample_shows_unknown_and_exits_3` (written and seen failing first), `eval_twice_records_each_sample_once_per_rev`, `offset_baseline_is_read_from_source_at_shifted_time` (the double argv log shows `time_unix_s` one day and seven days earlier), `baseline_older_than_series_shows_unknown`, `show_exits_with_worst_verdict` (`CRIT` exits 2, `WARN` 1), `show_all_na_exits_3` and `prev_is_previous_raw_verdict`.

## Out of scope

Hysteresis and settled transitions, CloudEvents, `compare`, `rollup` (M2); re-grading history after a revision beyond grading samples not yet graded at the current `rev`.
