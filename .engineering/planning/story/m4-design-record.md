---
format: aep.planning-md/3
id: story:m4-design-record
kind: story
status: draft
title: Record the M4 decisions in the design
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m0-workspace
- depends_on: story:m0-probes
- depends_on: story:m2-spec-history-records
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 4
---
## Context

The M4 stories take decisions the design does not state yet: new CLI subcommands and service options, the heatmap cell rule, the before/after windows, the deploy instant record and the export shapes. `docs/design/thalamus.md` is the design every later reader starts from (`AGENTS.md`), so the decisions go into it, once, in one story. It changes no code and depends on no M4 code story: it records the decisions as the M4 stories state them.

`docs/design/thalamus.md` is shared. Earlier code and spec stories that edit it come first (`depends_on`): `story:m0-workspace`, `story:m0-probes`, `story:m2-spec-history-records`. The operator-run live stories that also edit it come after this one, never before, so a live observation never gates a code or spec story: `story:m1-live-pass`, `story:m2-live-observation` and `story:m4-live-acceptance` each `depends_on` this story, with this file as the shared surface. `story:m5-design-record` lands after this story as well.

## Work

- § 5.1: the CLI line gains `export rules|dashboard` and `deploys record|list`; the `thalamus-service` line names `--listen` (loopback unless `--allow-remote`), the routes of `story:m4-skeleton` and the SSE stream; views live in `thalamus-service`, exports in `thalamus-core` (`export/`), the one CloudEvents envelope builder in `thalamus-pipeline` (`cloudevents.rs`, `story:m4-live`).
- § 6.5: the heatmap cell rule (`story:m4-heatmap`); the sparkline band and lines (`story:m4-sparklines`); the rolling week (`story:m4-week-vs-week`); the before/after windows and score delta (`story:m4-before-after`); deploy instants as stored `thalamus.deploy.DeployInstant` records with origins `pod_created` and `merged` (`story:m4-spec-deploy-instants`, `story:m4-deploy-instants`); the live page without a script (`story:m4-live`); recorded series, offset baselines and `for:` of the export (`story:m4-export-rules`), the Grafana dashboard model (`story:m4-export-dashboard`).
- § 11: one row per decision above, numbered from the next free `D` when this story lands, each citing the story that took it, under a heading dated the day it lands.

## Domain relations

None read or written. The relations each decision rests on are cited in the story that took it.

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 5.1, § 6.5, § 11; shared; after `story:m0-workspace`, `story:m0-probes` and `story:m2-spec-history-records`, before `story:m1-live-pass`, `story:m2-live-observation`, `story:m4-live-acceptance` and `story:m5-design-record`)

## Acceptance

- `grep -nE "export (rules|dashboard)|deploys (record|list)|--listen" docs/design/thalamus.md` prints at least one line of § 5.1 for each of the three.
- `grep -nE "pod_created|merged" docs/design/thalamus.md` prints a line of § 6.5, and § 11 has one row per decision listed under Work, each naming its `story:m4-` id (`grep -c "story:m4-" docs/design/thalamus.md` is at least 9).
- `b10x-gates scan-text` on the changed file passes (no organization-specific name).
