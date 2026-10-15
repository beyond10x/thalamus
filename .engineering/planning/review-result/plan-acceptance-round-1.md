---
format: aep.planning-md/3
id: review-result:plan-acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, round 1
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
story:m3-one-vocabulary-across-sources — the acceptance runs `cargo test -p thalamus --test every_source_one_ladder`, but package `thalamus` is the generated model, outside the workspace, and the CLI is `thalamus-cli`, so the command cannot run as written; the Scope's `crates/thalamus/tests/` also names no crate — .engineering/planning/story/m3-one-vocabulary-across-sources.md:36 (cf. .engineering/planning/story/m0-workspace.md:27)
story:m3-one-vocabulary-across-sources — "every sample and evaluation the pass records is admitted under conformance scenarios `…RecordSample/outcome/recorded`" names no test or command that checks admission; it is a fixed suite scenario, not a check of the pass's records — .engineering/planning/story/m3-one-vocabulary-across-sources.md:36
story:m2-auto-unmute — the clause "conformance scenario `thalamus.definition.UnmuteDefinition/outcome/unmuted` passes against the service" reads the same before and after the work, because the scenario already passes under the M0 `thalamus.definition.` selection and no service-side conformance harness is named — .engineering/planning/story/m2-auto-unmute.md:42 (cf. .engineering/planning/story/m0-conformance-definitions.md:44)
story:m2-auto-unmute — the Acceptance sentence omits the D19 test `expired_mute_unmutes_without_rewriting_the_file`, which sits only in "Decisions applied", so the acceptance passes under an implementation that rewrites the file — .engineering/planning/story/m2-auto-unmute.md:47
story:m2-scheduler — the Acceptance sentence omits `missed_slot_is_recorded_unknown_with_reason`, and `missed_slots_are_not_backfilled` passes both with Work's "nothing is synthesised" and with D16's recorded `unknown` sample, so it cannot tell which was built — .engineering/planning/story/m2-scheduler.md:57
story:m2-hysteresis-core — the Acceptance sentence omits `persistence_block_overrides_defaults` (D15), so it passes while `k`, `k_crit` and `m` stay hard-coded at 2, 1, 3 — .engineering/planning/story/m2-hysteresis-core.md:46
story:m2-spec-history-records — the Acceptance names only validity, three scenarios and `task check`, so it passes without `SignalDefinition.persistence`, the `Transition` → `SignalEvaluation` relation with its copied fields (D13) or the D14 rollup rule that "Decisions applied" adds — .engineering/planning/story/m2-spec-history-records.md:75
story:m2-flap-measurement — one acceptance sentence joins a test suite to a measurement on one real instance after at least 7 days of scheduled sampling, so the tests can pass while the measurement is absent and the story is neither done nor not done — .engineering/planning/story/m2-flap-measurement.md:44
story:m0-probes — one acceptance sentence joins four passing tests to an edit of the `UNMAPPED:` marker in `spec/domains/definition.yaml`, two independent outcomes that can pass or fail separately — .engineering/planning/story/m0-probes.md:48

What I read: 31 of 31 stories (`cat -n` of each body under `.engineering/planning/story/`). I also ran `aep plan artifact kinds` and `aep plan artifact lifecycle story`, `ess specify validate --path spec` (7 files today, so 9 after `story:m2-spec-history-records` is a real change) and `ess verify conform synthesize` into a temp directory (73 scenarios, 51 under `thalamus.definition.` or `thalamus.dimension.`; all seven scenario ids cited by the stories exist).

What I could not establish:
- "Written and seen failing first" and "each red before the change" cannot be checked afterwards, so I took them as process claims and did not count them as findings.
- Whether the new `RecordTransition` and `RecordRollup` commands become `UnmetObligation` stubs, which would make `task check exits 0` in `story:m2-spec-history-records` unsatisfiable without behaviour code in `thalamus-core`. No story scopes that code. I could not generate a modified spec to test it.
- `spec/suite.json` and `Cargo.toml` do not exist yet. The 73 and 51 counts and the commands are therefore checked against `ess` output, not against a built workspace.

Out of my lane, and they did not set the verdict:
- `story:m1-instance-registry` still refuses every `since` other than `first_sample` and any `connectors:` registry, naming blockers that D17 and D18 have since decided. D18 now admits a fixed timestamp and `dimension.created` for a file registry.
- `story:m2-spec-history-records` calls the entity `SettledTransition` in Work and `Transition` in "Decisions applied" and in `story:m2-transitions`.
- `story:m3-advisory-gate-mapping` and `story:m3-vocabulary-mapping` share `vocabulary.rs`, and the M3 adapter stories share `lib.rs`. That is a parallel-safety question.

```findings
- file: .engineering/planning/story/m3-one-vocabulary-across-sources.md
  line: 36
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance runs `cargo test -p thalamus --test every_source_one_ladder`, but package `thalamus` is the generated model, outside the workspace, and the CLI is `thalamus-cli`, so the command cannot run as written; the Scope's `crates/thalamus/tests/` also names no crate"
- file: .engineering/planning/story/m3-one-vocabulary-across-sources.md
  line: 36
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "\"every sample and evaluation the pass records is admitted under conformance scenarios `...RecordSample/outcome/recorded`\" names no test or command that checks admission; it is a fixed suite scenario, not a check of the pass's records"
- file: .engineering/planning/story/m2-auto-unmute.md
  line: 42
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the clause \"conformance scenario `thalamus.definition.UnmuteDefinition/outcome/unmuted` passes against the service\" reads the same before and after the work, because the scenario already passes under the M0 `thalamus.definition.` selection and no service-side conformance harness is named"
- file: .engineering/planning/story/m2-auto-unmute.md
  line: 47
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance sentence omits the D19 test `expired_mute_unmutes_without_rewriting_the_file`, which sits only in \"Decisions applied\", so the acceptance passes under an implementation that rewrites the file"
- file: .engineering/planning/story/m2-scheduler.md
  line: 57
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance sentence omits `missed_slot_is_recorded_unknown_with_reason`, and `missed_slots_are_not_backfilled` passes both with Work's \"nothing is synthesised\" and with D16's recorded `unknown` sample, so it cannot tell which was built"
- file: .engineering/planning/story/m2-hysteresis-core.md
  line: 46
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance sentence omits `persistence_block_overrides_defaults` (D15), so it passes while `k`, `k_crit` and `m` stay hard-coded at 2, 1, 3"
- file: .engineering/planning/story/m2-spec-history-records.md
  line: 75
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Acceptance names only validity, three scenarios and `task check`, so it passes without `SignalDefinition.persistence`, the `Transition` to `SignalEvaluation` relation with its copied fields (D13) or the D14 rollup rule that \"Decisions applied\" adds"
- file: .engineering/planning/story/m2-flap-measurement.md
  line: 44
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "one acceptance sentence joins a test suite to a measurement on one real instance after at least 7 days of scheduled sampling, so the tests can pass while the measurement is absent and the story is neither done nor not done"
- file: .engineering/planning/story/m0-probes.md
  line: 48
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "one acceptance sentence joins four passing tests to an edit of the `UNMAPPED:` marker in `spec/domains/definition.yaml`, two independent outcomes that can pass or fail separately"
```
