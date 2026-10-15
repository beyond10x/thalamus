---
format: aep.planning-md/3
id: story:m5-design-record
kind: story
status: draft
title: Record the M5 test-run event types and mapping decisions in the design
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-spec-integration-records
- depends_on: story:m4-design-record
- depends_on: story:m4-live-acceptance
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 2
---
## Context

`story:m5-spec-integration-records` declares the test-run event types and takes four mapping and export decisions. `docs/design/thalamus.md` is the design every later reader starts from (`AGENTS.md`), so they go into it, once, in this story, the way `story:m4-design-record` records M4. The spec story does not edit the design (coordinator decision C2, round 1), so no M5 code story waits on this one. It changes no code.

`docs/design/thalamus.md` is shared, and this story is ordered against every other story that edits it:

- after `story:m5-spec-integration-records` (`depends_on`): it records that story's decisions as stated there;
- after `story:m4-design-record` (`depends_on`, shared file);
- after `story:m4-live-acceptance` (`depends_on`, shared file), which itself lands after `story:m1-live-pass` and `story:m2-live-observation`. This puts the M5 design record behind the 28-day M4 observation; nothing but `story:m5-live-acceptance` depends on this story, and that story waits on `story:m4-live-acceptance` already, so the order delays no code or spec story (decomposer call, round-1 revision);
- before `story:m5-live-acceptance` (it `depends_on` this story).

## Work

- § 6.3: names the two event types `dev.beyond10x.testrun.case.finished` and `dev.beyond10x.testrun.run.finished`, the domain `thalamus.testrun` and its file `spec/domains/testrun.yaml`.
- § 11: one row per decision of `story:m5-spec-integration-records` (`errored` is `unknown`, `failed` is 0, `skipped` records nothing; the `test.run.pass_ratio` formula and its `partial` upper bound; the `testrun` source label; which export record cites what), numbered from the next free `D` when this story lands, each citing `story:m5-spec-integration-records`, under a heading dated the day it lands.

## Domain relations

None read or written. The relations each decision rests on are cited in `story:m5-spec-integration-records`.

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 6.3, § 11; shared; after `story:m4-design-record` and `story:m4-live-acceptance`, before `story:m5-live-acceptance`)

## Acceptance

- `grep -n "dev.beyond10x.testrun.case.finished" docs/design/thalamus.md` and `grep -n "dev.beyond10x.testrun.run.finished" docs/design/thalamus.md` each print a line of § 6.3.
- § 11 has one row per decision listed under Work, each naming the spec story: `grep -c "story:m5-spec-integration-records" docs/design/thalamus.md` is at least 4.
- `b10x-gates scan-text` on the changed file passes (no organization-specific name).
