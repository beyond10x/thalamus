---
format: aep.planning-md/3
id: review-result:m6-design-round-1
kind: review-result
status: active
title: Design critic, epic:web-ui, round 1
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
needs-revision

m6-web-types — its generated Assertion and Assertions types read `spec/domains/assertion.yaml`, which `story:m5-spec-integration-records` rewrites (adds `Assertion.evaluations`), and no depends_on edge orders them. The board's typed assertion literals and `task drift` break if the M5 spec lands second. Add `depends_on story:m5-spec-integration-records`. Its own prerequisites are all spec stories, so the edge makes no cycle and adds no M4 or M5 code dependency. — .engineering/planning/story/m6-web-types.md:45, .engineering/planning/story/m6-web-types.md:56, .engineering/planning/story/m5-spec-integration-records.md:57

**Verdict**
- One finding, a warning: an unrecorded edge, not a wrong split.
- The seven stories are otherwise the right shape: no cycle, and the chain is design record, skeleton, types, then board, series and flow in parallel, then e2e.
- No story is cut as a layer.

**Edges into M2, M4 and M5**
- **`cloudevents.rs`:** the m4-live and m6-flow-view edit order is recorded (`m6-flow-view depends_on m4-live`).
- **m5-conductor-contract:** it only calls `envelope` and its scope excludes the file. m6-flow-view adds one kind and keeps the two existing envelopes byte-identical (m6-flow-view.md:44). So the m5 acceptance is unaffected, and m5-conductor-contract needs no edge to m6-flow-view.
- **`scheduler.rs`:** m6-flow-view leaves it untouched.
- **`http.rs`, `lib.rs`, `Cargo.toml`:** the m6-web-skeleton edges to m4-skeleton and m5-skeleton match these files' ownership.
- **m5-service-answers:** the `/api/assertions` 501 and 404 handling is recorded.

**What I read**
- **Artifacts:** 7 M6 stories in full, plus `epic:web-ui` and 8 neighbours (m4-live, m4-skeleton, m4-service-api, m2-transitions, m5-skeleton, m5-conductor-contract, m5-spec-integration-records, m5-service-answers).
- **Commands:** `aep plan artifact show`, `relations`, `graph` and `validate`.
- **Edges walked:** about 45, outside the set as well as inside it. I found no cycle.
- **Validator:** it reports `valid`.
- **Acceptance reads:** I traced about 18 to a producer, counted by hand. All but the one finding are recorded, directly or through a declared chain.

**Could not establish**
- **Out of my lane:** acceptance and parallel-safety critics own these.
  - **Stale `web/dist`:** the board's `task web-check` acceptance (m6-signal-board.md:Acceptance) needs `web/dist/` rebuilt, yet the body defers the rebuild to the coordinator at merge.
  - **Duplicated SSE framing:** m6-flow-view's `flow.rs` must produce the same evaluation SSE message as m4-live's `live.rs`, and the body does not say whether it shares the code or duplicates the framing. The equality test guards it, so I did not file it.
  - **Ordering through the chain:** the `Taskfile.yml` and `Cargo.toml` ordering in m6-web-e2e holds only through the chain of edges.
- **Unverified:** I did not check whether `ess generate types` accepts entity or view roots. m6-web-types.md:Context records that probe as its own first step.

```findings
- file: .engineering/planning/story/m6-web-types.md
  line: 45
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its generated Assertion and Assertions types and the board's typed literals read spec/domains/assertion.yaml, which story:m5-spec-integration-records rewrites (adds Assertion.evaluations, m5-spec-integration-records.md:57), and no depends_on edge runs from story:m6-web-types to it, so task drift and the typed literals break if the M5 spec lands second; add depends_on story:m5-spec-integration-records"
```
