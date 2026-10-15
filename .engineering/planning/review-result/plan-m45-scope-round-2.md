---
format: aep.planning-md/3
id: review-result:plan-m45-scope-round-2
kind: review-result
status: active
title: M4/M5 scope critic, round 2
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
- reviews: story:m5-design-record
- reviews: story:m5-live-acceptance
- reviews: story:m5-service-answers
- reviews: story:m5-skeleton
- reviews: story:m5-spec-integration-records
- reviews: story:m5-testrun-assertions
- reviews: story:m5-testrun-ingest
revision: 1
---
approve

**What I read:** 2 epics and 24 stories (13 `m4-`, 11 `m5-`), whole bodies, with `aep plan artifact show` and `aep plan artifact list`. I also read `docs/design/thalamus.md` § 6 and § 9, the `graph` (24 stories decompose these two epics, none outside `m4-`/`m5-`) and my round-1 report. I extracted 10 promises from `epic:views-and-exports` and 8 from `epic:assertions-and-integrations`, 18 in all, and traced all 18 to a story.

- **Round-1 fixes:** the asker-repository sections are now in `story:m5-declared-sets`, and `GET /api/assertions` is laid by `story:m5-skeleton` and filled by `story:m5-service-answers`.
- **Narrowings recorded in story bodies:** these are not findings.
  - The export skips store baselines and non-threshold rules.
  - Week-vs-week and before/after read evaluations rather than `offset_7d`.
  - Conductor gets a polled transition feed instead of the SSE stream.

**What I could not establish (out of my lane):**
- Whether the cortex acceptance "with its evaluation as evidence" is met by a fact that cites the export record copying the evaluation (acceptance critic).
- Whether the `settled_by` relation in `cortex-sources.yaml` targets `SignalEvaluation` nodes that no source creates (design critic).

```findings
[]
```
