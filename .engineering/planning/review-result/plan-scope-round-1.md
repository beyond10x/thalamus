---
format: aep.planning-md/3
id: review-result:plan-scope-round-1
kind: review-result
status: active
title: Scope critic, round 1
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
epic:history-and-transitions — "one transition emitted and cleared by real data" is promised and the only claiming story, `story:m2-transitions`, proves it with `warn_opened_and_cleared_end_to_end` on a fake source, so no item runs one transition through real data (that story is the natural owner) — .engineering/planning/epic/history-and-transitions.md:26
story:m2-rollup — design § 4 gives the daily rollup "worst verdict, verdict counts, min/max score", and the story drops them as "Not here" citing `decision-blocker:m2-rollup-revision`, which is cleared by D14 (highest `definition_rev` per sample) — .engineering/planning/story/m2-rollup.md:43
story:m1-instance-registry — it refuses every `denominator.since` but `first_sample` and every `connectors:` registry by naming cleared blockers, so D18 (fixed RFC 3339 timestamp; `dimension.created` for a `file` entry carrying `created`) and D17's error naming the missing Connectors inventory operation are claimed by no M1 story — .engineering/planning/story/m1-instance-registry.md:24
story:m1-prometheus-adapter — it resolves `since` only by probing `first_sample` and leaves `dimension.created` to the cleared `decision-blocker:m1-dimension-value-origin`, so D18's fixed-timestamp and file-registry-`created` origins are not read — .engineering/planning/story/m1-prometheus-adapter.md:37
story:m2-flap-measurement — it says "fixing the values waits on `decision-blocker:m2-persistence-parameters`", which D15 cleared, and no item records the fixing of `k`, `k_crit` and `m` that the epic's "before `k` and `m` are fixed" presupposes — .engineering/planning/story/m2-flap-measurement.md:22
story:m0-ci — a GitHub Actions workflow is claimed, but `epic:specification` ships only the `Taskfile.yml` `task check` gate and no CI, so the story is not traceable to any promise of the parent — .engineering/planning/epic/specification.md:17

What I read: 4 epics, 31 stories, 8 decision-blockers, design § 2.3, § 3, § 5, § 8, § 9, § 10 and § 11 (D1–D19), through `aep plan artifact graph` and `cat`. From the parents I extracted 36 promises (M0 5, M1 13, M2 10, M3 8) and traced 34 in full. The two partial ones are the real-data transition and the rollup verdict fields.

What I could not establish:
- Git and HTTP probe are narrowed to GitLab-hosted reads and catalog-read availability. The stories say so and name the missing Connectors operations, so I did not call this a gap.
- Out of my lane (design or acceptance critic):
  - `story:m2-scheduler` and `story:m2-auto-unmute` keep Work text that D16 and D19 contradict ("nothing is synthesised for a missed slot"; "sampled from its next slot").
  - `story:m3-one-vocabulary-across-sources` tests `-p thalamus` and `crates/thalamus/`, while `story:m0-workspace` names the CLI package `thalamus-cli`.
  - `story:m3-vocabulary-mapping` still defers the advisory gate to a cleared blocker; `story:m3-advisory-gate-mapping` does cover it.
- Unease only: `story:m2-flap-measurement` adds a `thalamus flaps` command that appears in neither design § 5.1 nor the epic's Ships.

```findings
- file: .engineering/planning/epic/history-and-transitions.md
  line: 26
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "\"one transition emitted and cleared by real data\" is promised and the only claiming story, story:m2-transitions, proves it with warn_opened_and_cleared_end_to_end on a fake source, so no item runs one transition through real data (that story is the natural owner)"
- file: .engineering/planning/story/m2-rollup.md
  line: 43
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "design § 4 gives the daily rollup \"worst verdict, verdict counts, min/max score\", and the story drops them as \"Not here\" citing decision-blocker:m2-rollup-revision, which is cleared by D14 (highest definition_rev per sample)"
- file: .engineering/planning/story/m1-instance-registry.md
  line: 24
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it refuses every denominator.since but first_sample and every connectors: registry by naming cleared blockers, so D18 (fixed RFC 3339 timestamp; dimension.created for a file entry carrying created) and D17's error naming the missing Connectors inventory operation are claimed by no M1 story"
- file: .engineering/planning/story/m1-prometheus-adapter.md
  line: 37
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it resolves since only by probing first_sample and leaves dimension.created to the cleared decision-blocker:m1-dimension-value-origin, so D18's fixed-timestamp and file-registry-created origins are not read"
- file: .engineering/planning/story/m2-flap-measurement.md
  line: 22
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it says \"fixing the values waits on decision-blocker:m2-persistence-parameters\", which D15 cleared, and no item records the fixing of k, k_crit and m that the epic's \"before k and m are fixed\" presupposes"
- file: .engineering/planning/epic/specification.md
  line: 17
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:m0-ci claims a GitHub Actions workflow, but epic:specification ships only the Taskfile.yml task check gate and no CI, so the story is not traceable to any promise of the parent"
```
