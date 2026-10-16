---
format: aep.planning-md/3
id: review-result:m6-design-round-2
kind: review-result
status: active
title: design critic, epic:web-ui, round 2
relations:
- reviews: story:m6-design-record
- reviews: story:m6-web-skeleton
- reviews: story:m6-web-types
- reviews: story:m6-signal-board
- reviews: story:m6-series-view
- reviews: story:m6-flow-view
- reviews: story:m6-web-e2e
revision: 1
---
approve

The round-1 finding is closed: `story:m6-web-types` now declares `depends_on story:m5-spec-integration-records` (`aep plan artifact graph`). I found no new shape defect.

- **Cycle:** none. The new edge adds no cycle. The Assertion spec it needs already exists in `spec/domains/assertion.yaml`, and `m5-spec-integration-records` depends only on spec stories.
- **Chain:** the order is design-record, then skeleton, then types, then board, series and flow in parallel, then e2e. That is a fan-out and a join, not a queue.
- **Hidden reads:** none remain after the rewritten acceptances.
  - **`m6-design-record`:** reads only `docs/design/thalamus.md`, which it writes.
  - **`m6-signal-board`:** `vue-tsc --noEmit` reads the client and the generated types from `m6-web-types`, a direct edge.
  - **`m6-web-e2e`:**
    - It reads the `web-e2e` feature and the `check` wiring from the skeleton.
    - It reads `/api/transitions` from the series view and `/api/flow/stream` from the flow view.
    - It reads the three views' `web/dist`, rebuilt by the coordinator at merge.
    - Each of these reaches its producer through a declared edge, directly or through the board, series and flow stories.
- **Edges into M2, M4 and M5:**
  - **`m6-flow-view` on `cloudevents.rs`:** the edge to `m4-live` is recorded.
  - **`m6-web-skeleton` on `http.rs`, `lib.rs` and `Cargo.toml`:** its edges to `m4-skeleton` and `m5-skeleton` match who owns those files.
  - **`m6-web-types` on the `transition.yaml` records:** it reaches `m2-spec-history-records` through `m4-live`, then `m2-transitions`.
  - **`/api/assertions`:** the board's edge to `m5-skeleton` covers the route, and the 501 case is handled as absent.

Read: 7 M6 stories in full, plus the round-1 design record, the M5 spec story, and the dependency edges of `m4-live`, `m4-service-api`, `m4-skeleton`, `m5-skeleton`, `m2-transitions` and `m5-service-answers`. Commands: `aep plan artifact show`, `relations`, `graph`, `validate`. I walked about 60 declared edges, including neighbours outside the set, and found no cycle. `validate` reports `valid`. I traced about 22 acceptance reads to a producer in the set, and every one is recorded by an edge, directly or through a declared chain.

Could not establish:
- **Not verified:** whether `ess generate types` accepts entity or view roots. `m6-web-types` records that probe as its own first step.
- **Out of my lane (acceptance):** `m6-design-record`'s § 6.5 check is `sed -n '/^### 6.5 Views/,/^### 6.6/p'`, but `docs/design/thalamus.md` has no `### 6.6` heading (headings at lines 325 and 391). The range therefore runs to end of file and can pass on text outside § 6.5. Whether it fails before the change depends on whether § 9 or § 11 already says "signal board", "series view" or "flow view". I did not check.
- **Out of my lane (parallel safety):** `m6-web-e2e` and the other stories that move a status item share `crates/thalamus-docs/src/status.rs` and `website/data/status.json`. Nothing in the set serialises them.

```findings
[]
```
