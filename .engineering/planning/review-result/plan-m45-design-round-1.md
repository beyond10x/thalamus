---
format: aep.planning-md/3
id: review-result:plan-m45-design-round-1
kind: review-result
status: active
title: M4/M5 design critic, round 1
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
story:m5-spec-integration-records — its `docs/design/thalamus.md` edit (§ 6.3, § 11) is the only reason it queues behind `story:m4-design-record` and, through the wave collision with no edge, behind `story:m4-live-acceptance`, so every M5 code story waits on a 28-day real-sampling observation. Trade-off: keep the order, or move the § 6.3 and § 11 rows into a late M5 design-record story as M4 did, so the spec story depends only on `story:m4-spec-deploy-instants`. — `aep plan artifact waves` (wave 14 `story:m4-live-acceptance`, wave 15 this story, collision `story:m4-live-acceptance story:m5-spec-integration-records docs/design/thalamus.md`); `.engineering/planning/story/m5-spec-integration-records.md:67`
story:m4-design-record — its `depends_on` edges to `story:m1-live-pass` and `story:m2-live-observation` exist only because those two operator-run stories also edit `docs/design/thalamus.md`, yet this story gates `story:m4-live-acceptance` and, through `story:m5-spec-integration-records`, all of M5. Trade-off: keep the edges and accept the operator wait, or reverse them so the live stories land after the design record and the record waits only on `story:m0-workspace`, `story:m0-probes` and `story:m2-spec-history-records`. — `.engineering/planning/story/m4-design-record.md:11,13,21`
story:m5-testrun-ingest — its `depends_on story:m5-service-answers` exists only for one selection line in `crates/thalamus-service/src/scheduler.rs`, and it puts the test-run bridge behind the whole assert chain (spec, assert, service-answers, ingest, testrun-assertions, live-acceptance). Trade-off: keep that order, or split the shared surface so the push-fed skip lives in `crates/thalamus-pipeline/src/sample.rs` and the scheduler picks it up. The skip rule is currently stated twice, in `sample.rs` and in `scheduler.rs`. — `.engineering/planning/story/m5-testrun-ingest.md:14,49,67`
story:m5-conductor-contract — it makes `thalamus-cli` depend on `thalamus-service` to reuse the envelope function in `events.rs`. `story:m4-skeleton` gives that crate axum and tokio, and `story:m4-live` builds a second CloudEvents envelope in `live.rs`, so the CLI would link an HTTP server and the envelope would live in two service files. The body should say the envelope builder moves to `thalamus-pipeline` or `thalamus-core`, which `thalamus-cli` already depends on, and name which story moves it. — `.engineering/planning/story/m5-conductor-contract.md:40,72`; `.engineering/planning/story/m4-live.md:40`; `.engineering/planning/story/m4-skeleton.md:51`; design § 5.1 crate table, `docs/design/thalamus.md:236-245`

**What I read:** all 23 stories in full, and the bodies of `m1-live-pass`, `m2-live-observation`, `m0-probes`, `m1-verdict-grading`, `m2-transitions`, `m3-git-adapter` and `m3-kubernetes-adapter`. Commands: `aep plan artifact relations`, `graph`, `validate` (valid), `waves`, `list`, and `show` via the story files.

**Edges walked:** 83 `depends_on` edges from the set, and all 167 in the store, including those outside the set, through `tsort`. There are no cycles. About 20 acceptance reads were traced to a producer in the set, and each has a direct or transitive `depends_on` edge. The one unrecorded ordering is the design-file collision in finding 1.

**What I could not establish:**
- Whether `testrun` definitions load without a `testrun.` arm in `registry.rs`. The extractor is free-form per `spec/domains/definition.yaml`, and `m5-testrun-ingest` names no extractor. Its acceptance test `sample_all_skips_push_fed_definitions` may cover it.
- Out of my lane (scope): design § 5.1 lists "assertions" among the HTTP API surfaces. `m4-service-api` defers them to M5, and no M5 story adds a route or `/api/assertions`. `m4-skeleton` fixes the route table as complete.
- Out of my lane (parallel safety): `m4-sparklines` hedges an edit to `m1-verdict-grading`'s `rules/threshold.rs` ("if the grader computes the line inline"), so its scope is conditional.

```findings
- file: .engineering/planning/story/m5-spec-integration-records.md
  line: 67
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its docs/design/thalamus.md edit (§ 6.3, § 11) is the only reason it queues behind story:m4-design-record and, through the wave collision with no edge, behind story:m4-live-acceptance, so every M5 code story waits on a 28-day real-sampling observation; trade-off: keep the order, or move the § 6.3 and § 11 rows into a late M5 design-record story as M4 did so the spec story depends only on story:m4-spec-deploy-instants"
- file: .engineering/planning/story/m4-design-record.md
  line: 11
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its depends_on edges to story:m1-live-pass and story:m2-live-observation exist only because those operator-run stories also edit docs/design/thalamus.md, yet this story gates story:m4-live-acceptance and, through story:m5-spec-integration-records, all of M5; trade-off: keep the edges, or reverse them so the live stories land after the design record"
- file: .engineering/planning/story/m5-testrun-ingest.md
  line: 14
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its depends_on story:m5-service-answers exists only for one selection line in crates/thalamus-service/src/scheduler.rs and puts the test-run bridge behind the whole assert chain; trade-off: keep that order, or put the push-fed skip in crates/thalamus-pipeline/src/sample.rs so the scheduler picks it up, since the skip rule is stated twice"
- file: .engineering/planning/story/m5-conductor-contract.md
  line: 40
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it makes thalamus-cli depend on thalamus-service to reuse the CloudEvents envelope in events.rs, linking the axum and tokio server that story:m4-skeleton adds into the CLI while story:m4-live builds a second envelope in live.rs; the body should say the envelope builder moves to thalamus-pipeline or thalamus-core and name the story that moves it"
```
