---
format: aep.planning-md/3
id: review-result:plan-m45-design-round-2
kind: review-result
status: active
title: M4/M5 design critic, round 2
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

**What I read:** all 24 `story:m4-*` and `story:m5-*` bodies in full, plus `review-result:plan-m45-design-round-1`. Commands: `aep plan artifact relations`, `validate` (valid, 85 artifacts), `graph`, `waves`, `list`.

**Edges walked:** 175 `depends_on` edges across the whole store, including those outside the set, through `tsort`. There are no cycles.

**Acceptance reads traced:** about 30 acceptance reads traced to a producer in the set. Each has a direct or transitive `depends_on` edge. Every `waves` collision between M4/M5 stories is covered by an edge that names the shared file.

**Round-1 findings:** all four are fixed.
- The spec and design records no longer queue M5 behind the 28-day observation.
- `story:m1-live-pass` and `story:m2-live-observation` now depend on `story:m4-design-record`.
- `story:m5-testrun-ingest` puts its skip in `sample.rs` and no longer depends on `story:m5-service-answers`.
- `story:m5-conductor-contract` takes the envelope builder from `thalamus-pipeline`, moved by `story:m4-live`.

**What I could not establish:**
- Out of my lane (scope): design § 5.1's "assertions" HTTP surface is now covered by `story:m5-skeleton` and `story:m5-service-answers`. I found no gap there.
- Out of my lane (parallel safety): `story:m4-sparklines` still hedges its `threshold.rs` edit ("if the grader computes the line inline"), as in round 1.
- Unease, not a finding: `story:m5-declared-sets` bundles a read-only `--list` into the ask command.

```findings
[]
```
