---
format: aep.planning-md/3
id: review-result:m6-parallel-safety-round-2
kind: review-result
status: active
title: parallel-safety critic, epic:web-ui, round 2
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

I found no collision and no unrecorded reader-beside-producer pair. Both round 1 changes landed.

- **Round 1 finding (`web-check` unreachable in the signal-board tree): fixed.** The acceptance now ends with `npx vue-tsc --noEmit` in `web/`, which the implementor's tree can reach. It says `task web-check` runs at the merge gate after the coordinator's rebuild of `web/dist/` (`.engineering/planning/story/m6-signal-board.md`, Acceptance, lines 264 and 266).
- **New edge `story:m6-web-types` → `story:m5-spec-integration-records`: sound.** `story:m6-web-types` roots `thalamus.assertion.Assertion` and `thalamus.assertion.Assertions` in `ess generate types`, and `story:m5-spec-integration-records` writes `spec/domains/assertion.yaml`. The edge does not change the waves: `story:m5-spec-integration-records` is in wave 11 and `story:m6-web-types` stays in wave 13.
- **Intersecting surfaces are all ordered by `depends_on`.** `aep plan artifact waves` reports 238 collision lines, 17 of them between two M6 stories. Each of the 17 is one of these:
  - skeleton against the one view story that fills a stub (`Board.vue`, `Series.vue`, `Flow.vue`, `flow.rs`, `api_transitions.rs`);
  - skeleton, types and e2e on `Taskfile.yml`;
  - skeleton and e2e on `Cargo.toml`.
- **View stories do not collide with each other.** `story:m6-signal-board`, `story:m6-series-view` and `story:m6-flow-view` share wave 14 with disjoint files. `web/dist/` is outside their typed scope in all of them, and each body names the coordinator's rebuild.
- **`story:m6-flow-view` edits `story:m4-live`'s `cloudevents.rs` and its test file.** `story:m6-flow-view` depends on `story:m4-live`, and its scope names the ownership.
- **Readers beside producers: 12 traced, 12 recorded by `depends_on`.**
  - Every M6 story depends on the skeleton and on `story:m6-web-types`, directly or transitively.
  - `story:m6-web-types` also depends on `story:m4-service-api`, `story:m4-live` and `story:m5-spec-integration-records`.
  - `story:m6-signal-board` also depends on `story:m5-skeleton`; `story:m6-series-view` and `story:m6-flow-view` on `story:m2-transitions`; `story:m6-flow-view` on `story:m4-live`.
  - `story:m6-web-e2e` reads the embedded `web/dist/`, rebuilt by the coordinator as each view merges. It depends on all three views, so the dist it embeds is complete when it runs.
  - The one read no edge can record is `web/dist/`, produced by the coordinator at merge. It is stated in every body that reads it.

What I read: 7 stories with `aep plan artifact show`, `aep plan artifact waves` (full output and its M6 collision lines), `review-result:m6-parallel-safety-round-1`, and the heads of `story:m4-live`, `story:m5-skeleton`, `story:m5-spec-integration-records` and `story:m4-service-api`.

Surface counts: 7 cited (paths named in the Work bullets; the store labels every scope entry "inferred"), 0 inferred, 0 unplaceable.

What I could not establish:
- None of the paths exist yet (no `web/` in the tree), so collisions rest on the bodies alone.
- Collisions with stories outside the set are not assessed. That covers `docs/design/thalamus.md` against `story:m4-live-acceptance`, `story:m5-design-record` and `story:m5-live-acceptance`, and `status.rs` against other status movers.
- Out of my lane: `story:m6-web-types` `fixtures.test.ts` is worded as validating "each golden answer". If that test globs `web/fixtures/`, it also picks up the files the view stories add. That is for the acceptance critic.

```findings
[]
```
