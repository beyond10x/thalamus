---
format: aep.planning-md/3
id: review-result:plan-scope-round-2
kind: review-result
status: active
title: Scope critic, round 2
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
approve

What I read: 4 epics and 32 stories whole, plus design § 2–§ 11 (D1–D19) and `review-result:plan-scope-round-1`. I used `aep plan artifact list`, `aep plan artifact validate` (50 artifacts, valid), `cat` and Read, and spot-checked `spec/domains/definition.yaml:32`. I extracted 38 promises from the four parents (M0 6, M1 14, M2 10, M3 8) and traced all 38 to a story. For M0, the ESS domain was already on disk when the plan was drafted, so I counted it as covered.

Round-1 findings, all fixed:
- **Real-data transition:** `story:m2-live-observation` claims it, with `health_observation` evidence from real reads.
- **Rollup verdict fields:** `story:m2-rollup` now rolls up worst verdict, verdict counts and min/max score at the highest `definition_rev` (D14), and `story:m2-spec-history-records` specifies them.
- **D17/D18 in M1:** `story:m1-instance-registry` claims the D17 refusal naming the missing inventory operation and all three D18 `since` origins.
- **Prometheus `since`:** `story:m1-prometheus-adapter` leaves `since` to the registry and pipeline.
- **Fixing k and m:** `story:m2-live-observation` records the fix from a 7-day `flaps --sweep` (D15), and `story:m2-flap-measurement` no longer cites the cleared blocker.
- **CI traceability:** `epic:specification` revision 3 now ships the GitHub Actions line that `story:m0-ci` claims.

What I could not establish:
- Out of my lane, not setting the verdict:
  - `story:m0-ci` is `blocked: decision` on the still-open `decision-blocker:m0-github-repository`. The drafter named this, so it is not a gap.
  - `story:m1-connectors-client` puts one-line stubs in the M3 adapter files. That is a parallel-safety question.
- Unease, not findings:
  - `thalamus flaps` and the `thalamus-pipeline` crate appear in neither design § 5.1 nor an epic Ships line. `story:m0-workspace` records the crate in § 5.1 but nothing records the command there. Both trace to epic acceptance and M1 ships.
  - `story:m2-auto-unmute` rests on D9 and D19, not on any epic sentence. It sits under "the scheduler in `thalamus-service`", so I did not call it reach.
- Round-1 out-of-lane items on D16/D19 text and the `thalamus-cli` package name are not re-checked here.

```findings
[]
```
