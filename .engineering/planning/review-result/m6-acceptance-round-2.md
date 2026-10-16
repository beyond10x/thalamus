---
format: aep.planning-md/3
id: review-result:m6-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, epic:web-ui, round 2
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

story:m6-design-record — the § 6.5 check `sed -n '/^### 6.5 Views/,/^### 6.6/p' … | grep -cE "signal board|series view|flow view"` has an end pattern that matches nothing (`docs/design/thalamus.md` has no `### 6.6`; § 6 ends at `## 7. Standards`, line 391), so the range runs to end of file and the count of at least 3 passes on mentions in § 9 or § 11 without any § 6.5 row — `aep plan artifact show story:m6-design-record`, "## Acceptance" third bullet; `sed -n '/^### 6.5 Views/,/^### 6.6/p' docs/design/thalamus.md | wc -l` prints 215 and the one baseline match is a § 11 line

story:m6-design-record — the first two checks (`grep -nE "/ui/|/api/transitions|/api/flow/stream"` and `grep -n "dev.beyond10x.thalamus.sample"`) search the whole file while claiming "a line of § 5.1" and "a line of § 6.2", so a § 11 row alone satisfies them and the § 5.1 and § 6.2 edits stay unchecked, as in round 1; scope them as the later bullets are, e.g. `sed -n '/^### 5.1/,/^### 5.2/p'` and `sed -n '/^### 6.2/,/^### 6.3/p'` — `aep plan artifact show story:m6-design-record`, "## Acceptance" first and second bullets; § 5.1 spans lines 248-271 and § 6.2 lines 305-311 of `docs/design/thalamus.md`

What I read: 7 of 7 ids (story:m6-design-record, m6-web-skeleton, m6-web-types, m6-signal-board, m6-series-view, m6-flow-view, m6-web-e2e), whole bodies via `aep plan artifact show story:<id>`, plus `review-result:m6-acceptance-round-1`. I ran the design-record greps and seds against `docs/design/thalamus.md` to get before-state counts.

Round-1 outcomes:
- m6-web-e2e: the docs finding is fixed. The acceptance now has a grep on `website/docs/concepts/interfaces.md` and a status-item clause. The file currently has no `/ui` line.
- m6-design-record: the § 9 and § 11 checks are fixed. Both are correctly scoped with baseline 0.
- m6-design-record: the § 6.5 and § 5.1/§ 6.2 checks are not fixed, per the two findings above.

What I could not establish:
- Whether `task web-check`, `task drift` and `task test-web-e2e` exist yet. They are new tasks, and I did not check the Taskfile.
- Whether the named failing-first tests exist. I assumed they are new.
- Whether `ess generate types` accepts an entity or view as `--root`. This is an open probe in m6-web-types.
- Unease, not findings: the hatched `UNKNOWN` and grey `N/A` marks in `Board.vue` have no test beyond the no-score logic, and the e2e checks only the absence of score text. The m6-design-record `scan-text` bullet passes before the change, which contradicts "fails before". The m6-web-e2e clause `cargo test -p thalamus-docs passes` does not by itself separate before from after.
- Out of my lane, not counted in the verdict: the shared `Taskfile.yml` (m6-web-types and m6-web-e2e) and `status.rs` plus `website/data/status.json` (parallel safety). The `depends_on story:m5-spec-integration-records` added to m6-web-types is a design and parallel-safety matter.

```findings
- file: "aep plan artifact show story:m6-design-record (## Acceptance, third bullet)"
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the § 6.5 check `sed -n '/^### 6.5 Views/,/^### 6.6/p' … | grep -cE \"signal board|series view|flow view\"` ends on a `### 6.6` heading that does not exist, so the range runs to end of file and the count of at least 3 passes on § 9 or § 11 mentions without any § 6.5 row; end the range at `^## 7\\.`"
- file: "aep plan artifact show story:m6-design-record (## Acceptance, first and second bullets)"
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the first two checks grep the whole file while claiming a line of § 5.1 and of § 6.2, so a § 11 row alone satisfies them and the § 5.1 and § 6.2 edits are unchecked; scope them with sed ranges as the § 9 and § 11 bullets are"
```
