---
format: aep.planning-md/3
id: review-result:plan-m45-acceptance-round-2
kind: review-result
status: active
title: M4/M5 acceptance critic, round 2
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
- reviews: story:m5-design-record
- reviews: story:m5-live-acceptance
- reviews: story:m5-service-answers
- reviews: story:m5-skeleton
- reviews: story:m5-spec-integration-records
- reviews: story:m5-testrun-assertions
- reviews: story:m5-testrun-ingest
revision: 1
---
needs-revision
story:m4-before-after — Work makes a zero or negative window and an unparseable duration answer 400 with nothing read from the store, but the only refusal test is `window_over_7d_is_refused_with_400` (8d), so those two refusals can be absent while every named test passes — .engineering/planning/story/m4-before-after.md:28
story:m5-conductor-contract — Work gives `thalamus transitions` a second cursor form `--since <ts>` (every transition at or after a time), but Acceptance names no test for it, only the `--after-id` cursor test `transitions_feed_resumes_after_cursor_without_repeats` — .engineering/planning/story/m5-conductor-contract.md:77

What I read: all 24 stories (13 `story:m4-*`, 11 `story:m5-*` bodies) by `cat -n`, plus `review-result:plan-m45-acceptance-round-1`, the story lifecycle, and `grep` of `spec/domains/assertion.yaml` (the D2 marker is on line 93, count 1 today, so that check does change with the work).
All six round-1 findings are fixed. The two above are the only gaps of the same kind that remain, and I raised them only where Work states a refusal or a flag.

Could not establish: the `ess specify validate` file count in `story:m4-spec-deploy-instants` (`spec/` holds 5 domain files, M1–M3 spec stories have not landed). Out of my lane: the test runner `story:m5-live-acceptance` needs does not exist yet.

```findings
- file: .engineering/planning/story/m4-before-after.md
  line: 28
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work makes a zero or negative window and an unparseable duration answer 400 with nothing read from the store, but the only refusal test is window_over_7d_is_refused_with_400 (8d), so those two refusals can be absent while every named test passes"
- file: .engineering/planning/story/m5-conductor-contract.md
  line: 77
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Work gives thalamus transitions a second cursor form --since <ts> (every transition at or after a time), but Acceptance names no test for it, only the --after-id cursor test transitions_feed_resumes_after_cursor_without_repeats"
```
