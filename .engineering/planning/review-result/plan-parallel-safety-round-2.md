---
format: aep.planning-md/3
id: review-result:plan-parallel-safety-round-2
kind: review-result
status: active
title: Parallel-safety critic, round 2
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
story:m1-verdict-grading — its acceptance test `every_graded_result_is_admitted_by_record_evaluation` runs results through `RecordEvaluation` outcomes `reason-missing` and `score-not-admitted`, which story:m0-conformance-records implements in `behaviour/evaluation.rs`, and no `depends_on` path runs from the grader to that story (it sits in wave 2, the producer in wave 3), so a stand-in `RecordEvaluation` would be built; add `depends_on: story:m0-conformance-records`, or say in the body that the test asserts those two invariants directly on the result and never calls the behaviour module (inferred, the body names no mechanism for the admission check) — .engineering/planning/story/m1-verdict-grading.md:67, .engineering/planning/story/m0-conformance-records.md:28

What I read: all 32 stories, the 5 epics' `depends_on` edges via `aep plan artifact graph`, and the revised front matter (`scope`, `relations`) of every story. I also read the Scope, Work and Acceptance sections of the stories that share files, the ownership table in m0-workspace, `aep plan artifact waves --kind story --status draft` (12 waves, 60 collisions, 0 unassessed), and round-1 `review-result:plan-parallel-safety-round-1`.

Counts:
- **Placement:** 32 cited (every body names its paths and every story now records `scope`), 0 inferred, 0 unplaceable. The `scope` field marks each path `inferred`, but the body cites it.
- **Round 1:** all 12 findings are fixed. Skeleton ownership in m0-workspace removes the shared `lib.rs`, `mod.rs`, `members` and subcommand-enum lines. Every shared file is now named in both bodies with an ordering edge, and the scope of m2-store-baselines, m2-transitions and m3-one-vocabulary-across-sources is narrowed. The 60 collisions that `waves` prints all sit between stories connected by `depends_on`, directly or through a chain, and no two stories in one wave collide.
- **Reads traced to a producer:** about 27 acceptance reads. 26 are ordered by a `depends_on` edge or chain. 1 is not (the finding above). I did not count each edge exactly.

What I could not establish:
- `docs/design/thalamus.md` is edited by m0-workspace (§5.1), m0-probes (§8), m1-live-pass (§4), m2-spec-history-records (§11) and m2-live-observation (§3.2). The bodies name different sections, and `waves` already puts m1-live-pass (wave 7), m2-spec-history-records (wave 8) and m2-live-observation (wave 12) in separate waves. I raised no finding, but the m1-live-pass pair has no edge, so it stays separated only while `waves` is followed.
- `Cargo.lock` is rewritten by several stories. The bodies of m1-connectors-client, m1-eventlog-store, m2-scheduler and m0-workspace name it, and the coordinator regenerates it at merge. I counted that as named and deliberate.
- Whether m3-vocabulary-mapping's new `EvaluationSpec.mapping` field adds scenarios to `spec/suite.json` that `behaviour/definition.rs` would have to answer is unclear. That is acceptance territory (out of my lane), and it did not set the verdict.
- m1-live-pass and m2-live-observation need a real Prometheus and seven days of sampling. That is scheduling and acceptance, out of my lane.

```findings
[
  {"file": ".engineering/planning/story/m1-verdict-grading.md", "line": 67, "category": "parallel-safety", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "its acceptance test every_graded_result_is_admitted_by_record_evaluation reads the RecordEvaluation outcomes reason-missing and score-not-admitted that story:m0-conformance-records implements in crates/thalamus-core/src/behaviour/evaluation.rs (wave 3, this story is wave 2), and no depends_on path runs from this story to it; add depends_on story:m0-conformance-records or state that the test asserts the two invariants directly and never calls the behaviour module (inferred: the body names no admission mechanism)"}
]
```
