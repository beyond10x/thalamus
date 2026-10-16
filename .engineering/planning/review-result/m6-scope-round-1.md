---
format: aep.planning-md/3
id: review-result:m6-scope-round-1
kind: review-result
status: active
title: Scope critic, epic:web-ui, round 1
relations:
- reviews: story:m6-design-record
- reviews: story:m6-web-skeleton
- reviews: story:m6-web-types
- reviews: story:m6-signal-board
- reviews: story:m6-series-view
- reviews: story:m6-flow-view
- reviews: story:m6-web-e2e
- reviews: epic:web-ui
revision: 1
---
approve

**What I read:** 8 artifacts, whole bodies: `epic:web-ui` and the 7 `story:m6-*` drafts. I ran `aep plan artifact show` on each and `aep plan artifact graph`. The graph shows no other artifact decomposing `epic:web-ui`, so nothing outside the set claims any of its promises.

I extracted 14 promises from the epic and traced all 14 to an item:

| Promises | Claimed by |
|---|---|
| `web/` Vue app, built and committed, served under one route prefix, no Node to run the service | `story:m6-web-skeleton` |
| TypeScript types generated from the specification, drift-checked, none hand-written | `story:m6-web-types` |
| Signal board, generic, grouped and filtered from the data | `story:m6-signal-board` |
| Board live updates | `story:m6-signal-board` |
| Board shows assertions when present | `story:m6-signal-board` |
| Series view over a time range | `story:m6-series-view` |
| Series view appends live | `story:m6-series-view` |
| Flow view | `story:m6-flow-view` |
| Flow pause and filter | `story:m6-flow-view` |
| Visible lag and reconnect state | `story:m6-web-types` (reader states), `story:m6-flow-view` and `story:m6-signal-board` (display) |
| Type check and unit tests in `task check` | `story:m6-web-skeleton` (`web-check`) |
| End-to-end check from a running service | `story:m6-web-e2e` |
| Read-only, and no fetch other than the serving origin | `story:m6-web-skeleton` (CSP), `story:m6-web-e2e` (`page_requests_only_the_serving_origin`) |
| The three decisions the epic needs recorded | `story:m6-design-record` items 1 to 3 |

**Reach beyond the parent:** none. Two things I checked and cleared:
- `GET /api/transitions` and `GET /api/flow/stream` are new routes, but they serve the epic's "settled transitions" and "sample → evaluation → transition" lines. Both stories explain why the existing routes cannot carry that data.
- The status item and `website/docs/concepts/interfaces.md` page in `story:m6-web-e2e` are required by `AGENTS.md` ("A story that ships a capability moves its item to `shipped`"), so they are not unrequested work.

**Two items claiming one outcome:** none. Each of the two cross-story overlaps is a reuse of another story's output:
- Lag handling in the board and in the flow view both use the `sse.ts` states.
- The design record describes decisions that the code stories implement.

**Narrowed promises:** none. The drafter named the one deliberate narrowing: the board lists assertions per signal rather than matching them to series (`story:m6-signal-board`, "Decisions taken here").

**What I could not establish:**
- Out of my lane, and not a finding: `epic:web-ui` says its data comes "all from M4 and M5 stories", but the set adds two routes. The epic body is stale on that line, which `story:m6-design-record` item 4 does record.
- Out of my lane: `story:m6-web-types` is blocked on a probe, since it is unknown whether `ess generate types` accepts an entity or a view as `--root`. That is a design and dependency question.

```findings
[]
```
