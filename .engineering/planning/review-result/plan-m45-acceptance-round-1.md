---
format: aep.planning-md/3
id: review-result:plan-m45-acceptance-round-1
kind: review-result
status: active
title: M4/M5 acceptance critic, round 1
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
story:m5-assert — Work names `--wait`, the `--dims` series filter, the highest-`definition_rev` count and the expired-mute overlay, but Acceptance names no test for any of them, so each can be absent while every named test passes — .engineering/planning/story/m5-assert.md:53
story:m5-declared-sets — Work adds `--list` (every assertion of one `--asker`) and `--from-description -` (stdin), but Acceptance names no test for either — .engineering/planning/story/m5-declared-sets.md:34
story:m5-testrun-assertions — Work says `--emit-cases`, `--emit-source`, `--project` and `--target` are "accepted only with `--asker test_run:...`", but no named test checks the refusal under any other asker — .engineering/planning/story/m5-testrun-assertions.md:33
story:m5-spec-integration-records — Work edits `docs/design/thalamus.md` (§ 6.3 event types, § 11 rows) and removes the `UNMAPPED:` marker at `assertion.yaml:93`, but Acceptance checks neither, unlike `story:m4-design-record`, which greps its § 11 rows — .engineering/planning/story/m5-spec-integration-records.md:96
story:m5-live-acceptance — Acceptance is one sentence joining three independent real-system outcomes (release, test run, cortex transition) and the § 9 lines, so one can pass while another fails and the story is neither done nor not done; `story:m4-live-acceptance` splits its outcomes into bullets — .engineering/planning/story/m5-live-acceptance.md:57
story:m4-before-after — Work caps `window` at 7d (default 2h), but no named test checks the cap or the default, so Acceptance reads the same whether or not the cap exists — .engineering/planning/story/m4-before-after.md:25

What I read: 23 of 23 artifacts (11 `story:m4-*`, 12 `story:m5-*`), by `cat -n` of each story file in `.engineering/planning/story/`, plus `aep plan artifact kinds` and `aep plan artifact lifecycle story`. I also grepped `docs/design/thalamus.md` and `spec/` to see whether the M4 acceptance checks fail before the work.

Could not establish:
- `story:m4-spec-deploy-instants` says `ess specify validate` reports "10 files when nothing else changed". I cannot check that count: `spec/` holds only the M0 files, so the M1–M3 spec stories have not landed.
- The jq enum selector in `story:m4-spec-deploy-instants` carries "adjust the selector to the IR shape". I treated that as acceptable because the asserted variant set is fixed.
- `story:m4-design-record`'s `grep -nE` for `export (rules|dashboard)` cannot tell which of the three § 5.1 items matched. I left it, since the output carries line numbers and § 5.1 sits at lines 234–250. Today those greps print nothing, so the check does change with the work.
- The `docs/integrations/*.md` deliverables of `story:m5-conductor-contract` and `story:m5-cortex-export` have no check beyond the golden files and the cortex mapping-path test. I did not raise them: the contract is pinned by the named fixtures.
- Out of my lane: `story:m5-live-acceptance` depends on a test runner that does not exist yet. That is a coverage or design question for another critic.

```findings
- file: .engineering/planning/story/m5-assert.md
  line: 53
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work names `--wait`, the `--dims` series filter, the highest-`definition_rev` count and the expired-mute overlay, but Acceptance names no test for any of them, so each can be absent while every named test passes"
- file: .engineering/planning/story/m5-declared-sets.md
  line: 34
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work adds `--list` (every assertion of one `--asker`) and `--from-description -` (stdin), but Acceptance names no test for either"
- file: .engineering/planning/story/m5-testrun-assertions.md
  line: 33
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work says `--emit-cases`, `--emit-source`, `--project` and `--target` are \"accepted only with `--asker test_run:...`\", but no named test checks the refusal under any other asker"
- file: .engineering/planning/story/m5-spec-integration-records.md
  line: 96
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work edits `docs/design/thalamus.md` (§ 6.3 event types, § 11 rows) and removes the `UNMAPPED:` marker at `assertion.yaml:93`, but Acceptance checks neither, unlike `story:m4-design-record`, which greps its § 11 rows"
- file: .engineering/planning/story/m5-live-acceptance.md
  line: 57
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Acceptance is one sentence joining three independent real-system outcomes (release, test run, cortex transition) and the § 9 lines, so one can pass while another fails and the story is neither done nor not done; `story:m4-live-acceptance` splits its outcomes into bullets"
- file: .engineering/planning/story/m4-before-after.md
  line: 25
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work caps `window` at 7d (default 2h), but no named test checks the cap or the default, so Acceptance reads the same whether or not the cap exists"
```
