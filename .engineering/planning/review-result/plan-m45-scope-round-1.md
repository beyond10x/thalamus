---
format: aep.planning-md/3
id: review-result:plan-m45-scope-round-1
kind: review-result
status: active
title: M4/M5 scope critic, round 1
relations:
- reviews: story:m4-before-after
- reviews: story:m4-deploy-instants
- reviews: story:m4-design-record
- reviews: story:m4-export-dashboard
- reviews: story:m4-export-rules
- reviews: story:m4-heatmap
- reviews: story:m4-live
- reviews: story:m4-live-acceptance
- reviews: story:m4-service-api
- reviews: story:m4-skeleton
- reviews: story:m4-sparklines
- reviews: story:m4-spec-deploy-instants
- reviews: story:m4-week-vs-week
- reviews: story:m5-assert
- reviews: story:m5-conductor-contract
- reviews: story:m5-cortex-export
- reviews: story:m5-declared-sets
- reviews: story:m5-live-acceptance
- reviews: story:m5-service-answers
- reviews: story:m5-skeleton
- reviews: story:m5-spec-integration-records
- reviews: story:m5-testrun-assertions
- reviews: story:m5-testrun-ingest
revision: 1
---
needs-revision
story:m5-declared-sets — the epic promises that "Releases, merge requests, test runs and conductor ask thalamus", but the body states no ask for the release pipeline or the factory, which are other repositories that must pass the description text and `--after` and re-run at the horizon; only conductor gets a "What conductor must add" section (add one naming what each asker repository does) — .engineering/planning/epic/assertions-and-integrations.md:13, .engineering/planning/story/m5-declared-sets.md:42, .engineering/planning/story/m5-conductor-contract.md:44
story:m5-service-answers — design § 5.1 gives the service an HTTP API of "current verdicts, series, assertions", and `story:m4-service-api` hands the assertions part to M5, but no M5 story claims an assertions route and `story:m4-skeleton` lays none (this story would most naturally take `GET /api/assertions` and `GET /api/assertions/{id}`, or record the drop) — .engineering/planning/story/m4-service-api.md:21, docs/design/thalamus.md:244

Read: 2 epics, 23 stories (13 m4, 10 m5), design § 5, 6, 9 and 11, the graph. I extracted 9 promises from `epic:views-and-exports` and 9 from `epic:assertions-and-integrations`, and traced all 18 to a story. M4 has no gap and no reach. Its narrowings are recorded in story bodies: the export skips store baselines and non-threshold rules, the heatmap reads raw evaluations only, and before/after and week-vs-week use evaluations rather than `offset_7d`. Cross-repo: conductor has a stated ask, cortex needs none, and the test-runner ask is stated only in `story:m5-live-acceptance`.

Not established, out of my lane: whether the cortex acceptance "with its evaluation as evidence" is met when the fact cites the export record that copies the evaluation (acceptance critic); the `settled_by` relation targets `SignalEvaluation` nodes no source creates (design critic).

```findings
- file: .engineering/planning/story/m5-declared-sets.md
  line: 42
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the epic promises that \"Releases, merge requests, test runs and conductor ask thalamus\", but the body states no ask for the release pipeline or the factory, which are other repositories that must pass the description text and --after and re-run at the horizon; only conductor gets a \"What conductor must add\" section (add one naming what each asker repository does)"
- file: .engineering/planning/story/m4-service-api.md
  line: 21
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "design § 5.1 gives the service an HTTP API of current verdicts, series, assertions, and story:m4-service-api hands the assertions part to M5, but no M5 story claims an assertions route and story:m4-skeleton lays none (story:m5-service-answers would most naturally take GET /api/assertions and GET /api/assertions/{id}, or record the drop)"
```
