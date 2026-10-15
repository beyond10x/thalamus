---
format: aep.planning-md/3
id: review-result:plan-design-round-2
kind: review-result
status: active
title: Design critic, round 2
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
- reviews: story:m2-live-observation
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
story:m2-auto-unmute — it hands an expired-mute definition to the pass "as Active", but the pass records through the generated `RecordSample` over file-backed definition ports that answer `not-active` from the file's `Muted` lifecycle, and no story's scope holds the overlay that makes the port answer Active (its scope has no `thalamus-pipeline` or `thalamus-core` path); name the owner, or add the pipeline/port path here with an edge to story:m1-cli-sample — .engineering/planning/story/m2-auto-unmute.md:31, .engineering/planning/story/m1-instance-registry.md:49, .engineering/planning/story/m1-cli-sample.md:41
story:m0-conformance-definitions — the scenario runner is written inside the integration-test file `crates/thalamus-core/tests/conformance.rs`, which another crate cannot import, yet story:m1-eventlog-store (and later story:m2-rollup and story:m2-transitions) run the same scenarios from `crates/thalamus-store/tests/conformance.rs` over Eventlog storage; no story puts the runner and its in-memory storage swap in a shared library module, so the second runner is a copy — .engineering/planning/story/m0-conformance-definitions.md:27, .engineering/planning/story/m1-eventlog-store.md:45

What I read: 51 artifacts (32 stories, 6 epics, 8 decision-blockers, 5 review-results). I ran `aep plan artifact relations`, `graph` (walked all declared edges, including those to epics and decision-blockers outside the set), `waves`, `validate` (valid) and `show` on every story, and read the round-1 report. Cycles: none. Chain: none; the longest path is 12 waves, and every hook-line or shared-file edge in M2/M3 states its reason (scheduler.rs, rules/mod.rs, definition.yaml), so it is a trade-off already taken. Acceptance reads: about 45 traced to a producer in the set, all but the one above recorded by an edge (direct or transitive); edges to an epic rather than the producing story are not counted as missing. All 8 round-1 findings are fixed.

What I could not establish:
- Whether the existing "newest evaluation of each series" read in story:m1-eventlog-store is meant to enumerate the series of a signal for story:m2-scheduler (D16 missed slots) and story:m2-rollup (series with samples on a day); no body states it. I did not count this as a finding.
- Out of my lane, not counted: story:m3-vocabulary-mapping lists detector and gate exit codes as producer values, but no adapter in the set reads an exit code, so this is for the scope critic. The sink for the CloudEvents that story:m2-transitions emits is unnamed, so this is for the acceptance critic.

```findings
- file: .engineering/planning/story/m2-auto-unmute.md
  line: 31
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it hands an expired-mute definition to the pass as Active, but the pass records through the generated RecordSample over file-backed definition ports that answer not-active from the file's Muted lifecycle, and no story's scope holds the overlay that makes the port answer Active; its scope has no thalamus-pipeline or thalamus-core path, so name the owner or add the path here with an edge to story:m1-cli-sample"
- file: .engineering/planning/story/m0-conformance-definitions.md
  line: 27
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scenario runner is written inside the integration-test file crates/thalamus-core/tests/conformance.rs, which another crate cannot import, yet story:m1-eventlog-store, story:m2-rollup and story:m2-transitions run the same scenarios from crates/thalamus-store/tests/conformance.rs over Eventlog storage; no story puts the runner and its storage swap in a shared library module, so the second runner is a copy"
```
