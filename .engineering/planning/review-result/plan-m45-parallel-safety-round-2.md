---
format: aep.planning-md/3
id: review-result:plan-m45-parallel-safety-round-2
kind: review-result
status: active
title: M4/M5 parallel-safety critic, round 2
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

What I read: all 24 `story:m4-*` and `story:m5-*` bodies, scopes and `depends_on` edges. I also read the relations of M0–M3 stories on shared surfaces: `m1-live-pass`, `m2-live-observation`, `m2-spec-history-records`, `m2-scheduler`, `m2-auto-unmute`, `m2-rollup`, `m2-transitions` and `m1-cli-eval-show`. Commands: `aep plan artifact waves --kind story --status draft` (13 waves, no unassessed items), `aep plan artifact show` on each story, `rg` over the story files, and the round-1 record `review-result:plan-m45-parallel-safety-round-1`.

Round-1 findings, all four fixed:
- `m4-spec-deploy-instants` now has `depends_on story:m3-vocabulary-mapping`, and `m5-spec-integration-records` is ordered after it.
- `m5-spec-integration-records` no longer scopes `docs/design/thalamus.md`.
- `m4-export-rules` now has `depends_on` edges to `story:m3-vocabulary-mapping` and `story:m3-advisory-gate-mapping`.
- `m4-skeleton` declares the HTTP test dev-dependencies once. The seven M4/M5 service-test stories (`m4-service-api`, `m4-heatmap`, `m4-live`, `m4-sparklines`, `m4-week-vs-week`, `m4-before-after`, `m5-service-answers`) each say they add none.

Collisions:
- **M4/M5 pairs:** every pair the waves output lists is ordered by a recorded `depends_on` path or a body that names the other story. This covers the design-doc chain, the conformance and spec files, the `scheduler.rs` chain, `cmd/assert.rs`, `cmd/export.rs` and the skeleton files.
- **M4/M5 against M0–M3:** every shared file is ordered the same way. This covers `m1-live-pass` and `m2-live-observation` (each depends on `m4-design-record`), `m2-spec-history-records`, `m3-vocabulary-mapping`, `m1-cli-sample`, `m1-verdict-grading` and `m2-transitions`.

What I could not establish:
- Surfaces: 24 cited from the bodies, 0 inferred by grep, 0 unplaceable. The store labels every scope entry `inferred`.
- Acceptance reads: about 37 traced to a producer in the set, all recorded by a `depends_on` edge, directly or transitively.
- Out of my lane, did not set the verdict: whether `m4-deploy-instants` can add its reader to the `m1-connectors-client` allow-list without editing files outside its scope. The Connectors client file was not re-read this round.
- Out of my lane: whether `m5-cortex-export` has a store read that enumerates transitions and rollups across all series. `m2-transitions` and `m2-rollup` do not state one, and `m5-conductor-contract`'s `transition_feed.rs` is later and unordered.

```findings
[]
```
