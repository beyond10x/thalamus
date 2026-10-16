---
format: aep.planning-md/3
id: review-result:m6-parallel-safety-round-1
kind: review-result
status: active
title: Parallel-safety critic, epic:web-ui, round 1
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

story:m6-signal-board — its acceptance ends with "`task web-check` exits 0", and `web-check` diffs a fresh `vite build` against the committed `web/dist/`. The body gives `web/dist/` to the coordinator at merge, and it is outside this story's scope. A `Board.vue` change makes that diff non-zero in the implementor's own tree, so the line cannot pass before the merge. Either replace it with a line the story can reach (for example `npm test` plus `task typecheck`), or state that it is checked only after the coordinator's `task web-build`. — .engineering/planning/story/m6-signal-board.md:64 (acceptance), .engineering/planning/story/m6-signal-board.md:60 (`web/dist/` left to the coordinator), .engineering/planning/story/m6-web-skeleton.md:86 (`web-check` defined as the diff against `web/dist/`)

The exclusion of `web/dist` from the typed scope of types, board, series-view and flow-view is sound for the three view stories. All three share wave 14, and each would otherwise rewrite the same hashed bundle. The skeleton keeps `web/dist` in its own scope and all four bodies name the coordinator's rebuild, so I did not report it as a collision.

No other pair collides. `aep plan artifact waves` prints no collision between two m6 stories.

| Shared surface | Stories | Ordered by |
|---|---|---|
| `Taskfile.yml` | skeleton, types, e2e | `depends_on` edges (e2e reaches types through board) |
| `crates/thalamus-service/Cargo.toml` | skeleton, e2e | `depends_on` edges (e2e reaches the skeleton through the views) |
| `flow.rs`, `api_transitions.rs`, `Board.vue`, `Series.vue`, `Flow.vue` | skeleton, then the one view story that fills each | `depends_on` edges, and the skeleton says so |
| `web/fixtures` | types (the directory), series-view (`transitions.json`), flow-view (`flow-stream.txt`) | both views depend on types; the files differ |
| `docs/design/thalamus.md` | design-record only, within the set | skeleton depends on design-record |

Readers beside producers, traced within the set: 8, all recorded by a `depends_on` edge. The edges are:
- Skeleton to design-record.
- Types to skeleton.
- Board to types.
- Series-view to types and to skeleton.
- Flow-view to types and to skeleton.
- E2e to the three views.
- The view stories read the generated schema and `sse.ts`/`api.ts` from types.
- The view stories fill the stubs the skeleton lays.

The one read that no edge can record is the one in the finding: `web/dist`, produced by the coordinator.

What I read: 7 stories (`aep plan artifact show` for each), `aep plan artifact waves` for the full store, and the critic rubric and procedure. The waves put the m6 stories at 6 (design-record), 11 (skeleton), 13 (types), 14 (board, series-view, flow-view) and 15 (e2e). Surfaces placed: 7 cited (paths in the Work bullets) and 0 inferred. The store labels every scope entry "inferred", but each story's Work names the paths, so I counted them cited. 0 unplaceable.

What I could not establish:
- None of the paths exist yet: there is no `web/` and no `http.rs` in `thalamus-service/src`. Collisions with other stories rest on the bodies alone.
- Collisions with stories outside the set are not assessed (for example `docs/design/thalamus.md` against the m4 and m5 stories, and `status.rs` against stories that move a status item).
- Out of my lane: whether the board's unreachable `web-check` line is an acceptance defect for the acceptance critic is their call.

```findings
- file: .engineering/planning/story/m6-signal-board.md
  line: 64
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance ends with `task web-check` exits 0, which diffs a fresh build against the committed web/dist/, but web/dist/ is outside this story's scope and rebuilt by the coordinator at merge (line 60), so the line cannot pass in the implementor's tree after Board.vue changes; replace it with a reachable check or state it is checked only after the coordinator's web-build (cited)"
```
