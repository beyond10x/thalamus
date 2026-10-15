---
format: aep.planning-md/3
id: review-result:plan-design-round-1
kind: review-result
status: active
title: Design critic, round 1
relations:
- reviews: story:m0-ci
- reviews: story:m0-conformance-definitions
- reviews: story:m0-conformance-records
- reviews: story:m0-probes
- reviews: story:m0-workspace
- reviews: story:m1-baselines
- reviews: story:m1-cli-eval-show
- reviews: story:m1-cli-sample
- reviews: story:m1-connectors-client
- reviews: story:m1-eventlog-store
- reviews: story:m1-instance-registry
- reviews: story:m1-live-pass
- reviews: story:m1-prometheus-adapter
- reviews: story:m1-verdict-grading
- reviews: story:m2-auto-unmute
- reviews: story:m2-compare
- reviews: story:m2-flap-measurement
- reviews: story:m2-hysteresis-core
- reviews: story:m2-rollup
- reviews: story:m2-scheduler
- reviews: story:m2-spec-history-records
- reviews: story:m2-store-baselines
- reviews: story:m2-transitions
- reviews: story:m3-advisory-gate-mapping
- reviews: story:m3-git-adapter
- reviews: story:m3-http-probe-adapter
- reviews: story:m3-kubernetes-adapter
- reviews: story:m3-loki-adapter
- reviews: story:m3-one-vocabulary-across-sources
- reviews: story:m3-sql-adapter
- reviews: story:m3-vocabulary-mapping
revision: 1
---
needs-revision
story:m2-hysteresis-core — its D15 acceptance test `persistence_block_overrides_defaults` reads `SignalDefinition.persistence`, which story:m2-spec-history-records creates, and no depends_on edge runs from it to story:m2-spec-history-records (the graph puts both in wave 1) — .engineering/planning/story/m2-hysteresis-core.md:50, .engineering/planning/story/m2-spec-history-records.md:79
story:m2-scheduler — its per-slot pass (adapter read, RecordSample, baseline fold, grade, RecordEvaluation) is the pipeline story:m1-cli-sample and story:m1-cli-eval-show put in the `thalamus-cli` binary crate (`sample.rs`, `eval.rs`), and no item moves it to a library crate `thalamus-service` can use; the body says only "M1 ships the adapter, `sample` and `eval` it calls", and story:m2-store-baselines assumes one shared grading call site — .engineering/planning/story/m2-scheduler.md:27, .engineering/planning/story/m1-cli-eval-show.md:43, .engineering/planning/story/m2-store-baselines.md:54
story:m2-transitions — its acceptance (`transition_recorded_when_hysteresis_settles`, `transition_survives_expired_evaluations`) reads stored transitions, but no story in the set puts `SettledTransition` behind a store port: story:m1-eventlog-store covers samples and evaluations, story:m2-rollup adds only `DailyRollup`, and this scope is only service files; add a store half here or an edge to a story that owns it — .engineering/planning/story/m2-transitions.md:43, .engineering/planning/story/m1-eventlog-store.md:23
story:m2-auto-unmute — it issues `UnmuteDefinition` as actor service and expects the conformance scenario `UnmuteDefinition/outcome/unmuted` to pass "against the service", while story:m1-instance-registry makes the definition ports read-only and refusing writes; the D19 store event it adds instead has no spec record or store port owner in the set — .engineering/planning/story/m2-auto-unmute.md:42, .engineering/planning/story/m2-auto-unmute.md:46, .engineering/planning/story/m1-instance-registry.md:28
story:m2-compare — it scopes "a series range read in thalamus-store/src if M1 does not ship one", but story:m1-eventlog-store already ships it, and story:m2-store-baselines claims the same store path; the body should name the one read it adds and which story owns it, or draw an edge to the other claimant — .engineering/planning/story/m2-compare.md:44, .engineering/planning/story/m1-eventlog-store.md:25, .engineering/planning/story/m2-store-baselines.md:54
story:m3-vocabulary-mapping — it maps producer values to verdicts inside `thalamus-sources` and says how a mapped verdict reaches an evaluation "is left to the implementor", yet verdicts are produced only by `thalamus-core` `grade()` against the closed rule-id list of story:m1-verdict-grading, so the seam between the two halves has no owner and no `thalamus-core` path is in scope — .engineering/planning/story/m3-vocabulary-mapping.md:30, .engineering/planning/story/m1-verdict-grading.md:29
story:m3-advisory-gate-mapping — its tests `advisory_count_above_baseline_is_warn` and `advisory_never_grades_crit` assert verdicts under `-p thalamus-sources --test vocabulary`, but its scope is only `vocabulary.rs` and no rule or `thalamus-core` change that grades a `fixed` baseline WARN-only is named, the same unowned seam as story:m3-vocabulary-mapping — .engineering/planning/story/m3-advisory-gate-mapping.md:19, .engineering/planning/story/m3-advisory-gate-mapping.md:35
story:m3-one-vocabulary-across-sources — its test `colour_na_through_sql_adapter_is_unknown` reads a colour mapping applied inside the SQL adapter that neither story:m3-sql-adapter (only `sql.value` and `sql.ratio`) nor story:m3-vocabulary-mapping (an "extractor any adapter applies") wires, and this scope claims no source file changes; the wiring needs an owner and a depends_on edge to it — .engineering/planning/story/m3-one-vocabulary-across-sources.md:36, .engineering/planning/story/m3-one-vocabulary-across-sources.md:45, .engineering/planning/story/m3-sql-adapter.md:22

What I read: 45 artifacts (31 stories, 6 epics, 8 decision-blockers). Commands: `aep plan artifact relations`, `graph`, `validate` (valid), `waves`, and `show` on every story by reading its file. I read the design §§ 5.1, 5.3, 9 and 11 for the checks above.

What I could not establish:
- Cycles: none found. I walked all 99 declared edges, including those to epics and decision-blockers outside the set.
- Chain: there is no serialising chain. The longest story-level path is 4 deep, m0-workspace to conformance-definitions to conformance-records, and m1-baselines to cli-eval-show to live-pass.
- Acceptance reads: I traced about 40 reads to a producer in the set. Where the reader and producer sit in different epics, the recorded edge goes to the epic, not the producing story (for example M1 `RecordSample` and `RecordEvaluation` tests read story:m0-conformance-records, and the M3 adapters read story:m1-connectors-client and its double). Whether `depends_on` an epic means "all children done" is not stated in the CLI output, so I did not count these as unrecorded. The edges the findings name are the ones with no recorded path at all.
- Out of my lane, for the acceptance and scope critics:
  - story:m3-one-vocabulary-across-sources runs `cargo test -p thalamus` and writes to `crates/thalamus/`, but story:m0-workspace names the CLI package `thalamus-cli` and the generated model package is also `thalamus`.
  - story:m2-spec-history-records Work still says `SettledTransition` with an UNMAPPED relation, while its "Decisions applied" paragraph says `Transition` with a relation to `SignalEvaluation`.
  - story:m2-scheduler Work says "nothing is synthesised" for a missed slot, against D16 `missed_slot`.
  - story:m2-rollup is not updated for D14.
  - The extractor prefix is `logql.` in story:m1-connectors-client and `loki.` in story:m3-loki-adapter.

```findings
- file: .engineering/planning/story/m2-hysteresis-core.md
  line: 50
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its D15 acceptance test persistence_block_overrides_defaults reads SignalDefinition.persistence, which story:m2-spec-history-records creates, and no depends_on edge runs from it to story:m2-spec-history-records"
- file: .engineering/planning/story/m2-scheduler.md
  line: 27
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its per-slot read, RecordSample, grade and RecordEvaluation pass is the pipeline M1 puts in the thalamus-cli binary crate (sample.rs, eval.rs), and no item moves it to a library crate the service can use"
- file: .engineering/planning/story/m2-transitions.md
  line: 43
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads stored transitions but no story in the set puts SettledTransition behind a store port; story:m1-eventlog-store covers samples and evaluations and story:m2-rollup only DailyRollup"
- file: .engineering/planning/story/m2-auto-unmute.md
  line: 42
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it expects the UnmuteDefinition/outcome/unmuted scenario to pass against the service while story:m1-instance-registry makes definition ports read-only, and the D19 store event it records instead has no spec record or store port owner in the set"
- file: .engineering/planning/story/m2-compare.md
  line: 44
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it scopes a series range read in thalamus-store if M1 does not ship one, but story:m1-eventlog-store already ships it and story:m2-store-baselines claims the same store path; name the one read each adds or draw an edge"
- file: .engineering/planning/story/m3-vocabulary-mapping.md
  line: 30
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it maps producer values to verdicts in thalamus-sources and leaves how a mapped verdict reaches an evaluation to the implementor, while verdicts come only from thalamus-core grade() and its closed rule-id list in story:m1-verdict-grading, so the seam has no owner"
- file: .engineering/planning/story/m3-advisory-gate-mapping.md
  line: 19
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its verdict tests run under thalamus-sources --test vocabulary but its scope names no rule or thalamus-core change that grades a fixed baseline WARN-only, the same unowned seam as story:m3-vocabulary-mapping"
- file: .engineering/planning/story/m3-one-vocabulary-across-sources.md
  line: 36
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its test colour_na_through_sql_adapter_is_unknown reads a colour mapping applied inside the SQL adapter that neither story:m3-sql-adapter nor story:m3-vocabulary-mapping wires, and its scope claims no source file changes"
```
