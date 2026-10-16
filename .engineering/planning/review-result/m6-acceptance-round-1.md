---
format: aep.planning-md/3
id: review-result:m6-acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, epic:web-ui, round 1
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
story:m6-design-record — the acceptance count `grep -c "story:m6-" … is at least 6` is met by lines anywhere in the file (§ 6.5 and § 9 rows also cite `story:m6-`), so it cannot show the six § 11 rows, and no check covers the § 6.5 and § 9 edits listed under Work — `aep plan artifact show story:m6-design-record`, section "## Acceptance"; baseline `grep -c "story:m6-" docs/design/thalamus.md` prints 0
story:m6-web-e2e — the acceptance names no check for the `website/docs/concepts/interfaces.md` section on the web UI that Work and Context promise, so the story can close without it; only the `status.rs` item is covered, indirectly, by the evidence check in `crates/thalamus-docs/src/status.rs:362` — `aep plan artifact show story:m6-web-e2e`, "## Acceptance" and "## Work" last bullet

What I read: 7 of 7 ids (story:m6-design-record, m6-web-skeleton, m6-web-types, m6-signal-board, m6-series-view, m6-flow-view, m6-web-e2e), whole bodies via `aep plan artifact show story:<id>`. I also ran `grep` on `docs/design/thalamus.md` and `crates/thalamus-docs/src/status.rs` to confirm the design record's before-state and the status evidence check.

What I could not establish:
- Whether `web-check`, `task drift` and `task test-web-e2e` exist yet. They are new tasks the stories create, and the Taskfile was not checked.
- Whether the named failing-first test functions exist. I assumed they are new, and did not grep the tree for each name.
- Whether `ess generate types` accepts an entity or view as `--root`. `story:m6-web-types` records this as an open probe.
- Out of my lane, not counted in the verdict: the shared `Taskfile.yml` (`story:m6-web-types` and `story:m6-web-e2e`), `status.rs` and `website/data/status.json` (parallel safety); the claim that `m4-live` owns `cloudevents.rs` (design).
- Unease, not findings: no test names the hatched `UNKNOWN` and grey `N/A` marks in `Board.vue`, or the `first_ts` span of a transition mark.

```findings
- file: "aep plan artifact show story:m6-design-record (## Acceptance)"
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance count `grep -c \"story:m6-\" docs/design/thalamus.md` is at least 6 is satisfied by lines anywhere in the file, so it cannot show one § 11 row per item, and no check covers the § 6.5 and § 9 edits listed under Work"
- file: "aep plan artifact show story:m6-web-e2e (## Acceptance, ## Work last bullet)"
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names no check for the website/docs/concepts/interfaces.md web UI section that Work and Context promise, so the story can close without it"
```
