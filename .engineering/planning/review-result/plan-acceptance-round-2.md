---
format: aep.planning-md/3
id: review-result:plan-acceptance-round-2
kind: review-result
status: active
title: Acceptance critic, round 2
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
story:m2-live-observation — the first criterion names `aep plan artifact show story:m2-live-observation` as the command that lists the `health_observation`, `metric_observation` and `diff` records, but `show` prints frontmatter, body, findings and review outcomes only (no evidence), so nobody can run it and see those records; evidence is read with `aep plan artifact history <id>` — .engineering/planning/story/m2-live-observation.md:46 (cf. ~/beyond10x/aep/crates/edge/aep-cli/src/planning.rs:4285 `shown_from`, 706)

What I read: all 32 stories, whole bodies, with `aep plan artifact list --kind story`, `cat`/`awk` over `.engineering/planning/story/*.md` and the round-1 report. Also `aep plan artifact kinds` for the evidence kinds, `ess specify compile --path spec --format json` with the acceptance `jq` shapes run against today's IR, and `ess verify conform synthesize` into a scratch directory with the cited scenario ids grepped.

What I could not establish:
- Whether `aep plan artifact history` prints `health_observation`, `metric_observation` and `diff` evidence in a form a person can read. Nothing is recorded on the story yet, and I read it only from the source comment at `planning.rs:706`.
- Whether the new `RecordTransition` and `RecordRollup` obligations leave `task check` green in `story:m2-spec-history-records` before the behaviour code lands. The story scopes that code in the same Work list.
- Whether the post-change `jq` checks and scenario ids in `story:m2-spec-history-records` and `story:m3-vocabulary-mapping` pass. The spec is not changed yet; I confirmed only that the same query shapes return false today (exit 1) and run without error.

Out of my lane, and they did not set the verdict:
- `story:m3-one-vocabulary-across-sources` runs the SQL, Loki and other adapters through `connectors-double`, a binary of `thalamus-sources`. A `thalamus-pipeline` integration test cannot use `CARGO_BIN_EXE_*` for another package, so how it locates the double is a design and parallel-safety question.
- `story:m2-spec-history-records` lists the `rolling-n-missing` outcome and the `Service` actor in Work. Its acceptance does not check them; the work is conditional on whether ESS can express the rule, so I did not count it.

```findings
- file: .engineering/planning/story/m2-live-observation.md
  line: 46
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the first criterion names `aep plan artifact show story:m2-live-observation` as the command that lists the `health_observation`, `metric_observation` and `diff` records, but `show` prints frontmatter, body, findings and review outcomes only (no evidence), so nobody can run it and see those records; evidence is read with `aep plan artifact history <id>`"
```
