---
format: aep.planning-md/3
id: review-result:plan-m45-parallel-safety-round-1
kind: review-result
status: active
title: M4/M5 parallel-safety critic, round 1
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

story:m4-spec-deploy-instants — shares `generated/spec-types`, `generated/thalamus-model` and `spec/suite.json` with story:m3-vocabulary-mapping (cited, both bodies list them). No edge orders them, and line 34 calls story:m2-spec-history-records "the last story that changes" them, which is false. Remedies: an ordering edge to story:m3-vocabulary-mapping recording the regenerated files as the reason, or splitting the regeneration. An edge on this story also orders story:m5-spec-integration-records, whose line 52 repeats the claim — .engineering/planning/story/m4-spec-deploy-instants.md:34, .engineering/planning/story/m3-vocabulary-mapping.md:27-33
story:m5-spec-integration-records — shares `docs/design/thalamus.md` (§ 6.3, § 11) with story:m4-live-acceptance (§ 6.5, cited, both bodies). Both depend only on story:m4-design-record and neither names the other. Only the scope-derived wave placement orders them, and that holds M5 spec, and so all of M5, behind m4-live-acceptance's 28-day live observation. Remedies: an ordering edge, or moving the dated line out of the shared file — .engineering/planning/story/m5-spec-integration-records.md:92, .engineering/planning/story/m4-live-acceptance.md:39
story:m4-export-rules — its acceptance reads state that story:m3-vocabulary-mapping (`producer_mapped`) and story:m3-advisory-gate-mapping (`fixed_warn_only`) create. `non_threshold_rule_is_skipped_with_reason` needs both rule ids to exist. Neither is an ancestor, and story:m3-advisory-gate-mapping is in the same wave 8. The missing edges are `depends_on` story:m3-vocabulary-mapping and story:m3-advisory-gate-mapping, or drop the two rule ids from the story — .engineering/planning/story/m4-export-rules.md:43, .engineering/planning/story/m4-export-rules.md:9-15, .engineering/planning/story/m3-advisory-gate-mapping.md:31
story:m4-skeleton — inferred: it adds only `axum`, `tokio` and `tokio-stream` to `crates/thalamus-service/Cargo.toml`. The HTTP and SSE tests of story:m4-service-api, story:m4-heatmap, story:m4-live, story:m4-sparklines, story:m4-week-vs-week and story:m4-before-after then need a client or test-service dev-dependency, and none of those scopes lists that Cargo.toml. No M4 body names an HTTP test technique, so each story would edit it independently. The fix is for m4-skeleton to declare the service dev-dependencies once — .engineering/planning/story/m4-skeleton.md:51

What I read: all 23 `story:m4-*` and `story:m5-*` bodies, plus the scope, edges and the sharing bodies of story:m2-spec-history-records, story:m2-transitions, story:m2-rollup, story:m3-vocabulary-mapping, story:m3-advisory-gate-mapping and story:m1-connectors-client. Commands: `aep plan artifact waves --kind story --status draft`, `aep plan artifact list --kind story`, `aep plan artifact relations`, and a read-only transitive-`depends_on` scan of every story's scope, restricted to unordered pairs with at least one M4/M5 story.

Surfaces: 23 placed from paths the bodies cite (the store records every scope entry as `inferred`), 0 inferred by grep, 0 unplaceable.

Acceptance reads: about 37 traced to a producer in the set, about 35 recorded by a `depends_on` edge (direct or transitive). The two without an edge are the two in the story:m4-export-rules finding.

Pairs whose shared file the plan states and an edge orders (checked, not findings):
- `crates/thalamus-service/src/scheduler.rs`: the chain m2-scheduler → m2-auto-unmute → m2-rollup → m2-transitions → m4-live → m5-service-answers → m5-testrun-ingest.
- `crates/thalamus-store/tests/conformance.rs`: m1-eventlog-store → m2-rollup → m2-transitions → m4-deploy-instants → m5-assert.
- `crates/thalamus-cli/src/cmd/export.rs` and `crates/thalamus-cli/src/cmd/assert.rs`.
- `crates/thalamus-core/src/rules/threshold.rs`.
- The `lib.rs`, `cmd/mod.rs` and `main.rs` module lines shared by m4-skeleton and m5-skeleton.

What I could not establish:
- Whether story:m4-deploy-instants' reader trait can declare read operations to the story:m1-connectors-client allow-list without editing `connectors.rs` or `registry.rs`, which are not in its scope. The client's declaration point is the adapter trait (`m1-connectors-client.md:57`).
- Whether story:m5-cortex-export needs an all-series transition and rollup read that no earlier story provides. The nearest candidate is `transition_feed.rs` in story:m5-conductor-contract, which is later and unordered.
- Both are interface or design questions. They are out of my lane and did not set the verdict.
- Whether story:m5-conductor-contract's new `thalamus-service` dependency in `crates/thalamus-cli/Cargo.toml` collides with another dependency edit. No other story scopes that file.

```findings
[
  {"file": ".engineering/planning/story/m4-spec-deploy-instants.md", "line": 34, "category": "parallel-safety", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "story:m4-spec-deploy-instants and story:m3-vocabulary-mapping both regenerate generated/spec-types, generated/thalamus-model and spec/suite.json (cited, both bodies), no depends_on edge orders them, and line 34 wrongly calls story:m2-spec-history-records the last story that changes them; story:m5-spec-integration-records line 52 repeats the claim and is ordered only transitively through this story. Remedies: an ordering edge recording the shared generated files as reason, or splitting the surface."},
  {"file": ".engineering/planning/story/m5-spec-integration-records.md", "line": 92, "category": "parallel-safety", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "story:m5-spec-integration-records and story:m4-live-acceptance (m4-live-acceptance.md:39) both edit docs/design/thalamus.md (cited), neither names the other and no edge orders them; only the scope-derived wave placement does, which silently holds all of M5 behind a 28-day live observation. Remedies: an ordering edge with the shared file as reason, or moving one story's design-doc edit off the shared file."},
  {"file": ".engineering/planning/story/m4-export-rules.md", "line": 43, "category": "parallel-safety", "severity": "blocker", "verdict": "needs-revision", "origin": "introduced", "message": "the acceptance test non_threshold_rule_is_skipped_with_reason reads rule ids producer_mapped (created by story:m3-vocabulary-mapping) and fixed_warn_only (created by story:m3-advisory-gate-mapping, same wave 8), and the depends_on list at lines 9-15 has an edge to neither; add both edges or drop the two rule ids from the story."},
  {"file": ".engineering/planning/story/m4-skeleton.md", "line": 51, "category": "parallel-safety", "severity": "warning", "verdict": "needs-revision", "origin": "introduced", "message": "inferred: the skeleton adds only axum, tokio and tokio-stream to crates/thalamus-service/Cargo.toml and no M4 body names an HTTP test technique, so the HTTP and SSE tests of story:m4-service-api, story:m4-heatmap, story:m4-live, story:m4-sparklines, story:m4-week-vs-week and story:m4-before-after would each need a dev-dependency edit to a Cargo.toml none of them scopes; declare the service dev-dependencies once in the skeleton."}
]
```
