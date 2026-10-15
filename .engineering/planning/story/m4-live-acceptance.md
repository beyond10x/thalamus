---
format: aep.planning-md/3
id: story:m4-live-acceptance
kind: story
status: draft
title: A 4-week heatmap and one known deploy in before/after on a real instance
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-heatmap
- depends_on: story:m4-before-after
- depends_on: story:m4-deploy-instants
- depends_on: story:m4-design-record
- depends_on: story:m2-live-observation
- depends_on: story:m1-live-pass
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 4
---
## Context

`epic:views-and-exports` acceptance: "a 4-week heatmap rendered from stored samples" and "one known deploy visible in before/after". The views and their tests are `story:m4-heatmap`, `story:m4-deploy-instants` and `story:m4-before-after` (`depends_on`); this story is the observation on real data, which needs an operator: a `thalamus-service` that has sampled one real instance on its scheduler for at least 28 days, the instance of `story:m2-live-observation` continued, with saved, validated Connectors connections on the machine that runs it. Thalamus holds no credential; the connection does.

## Work

- The instance directory stays outside this repository; it is sampled by `thalamus-service run` for at least 28 days on the File or SQLite store provider.
- Heatmap: `thalamus-service run --instance <dir> --listen 127.0.0.1:<port>`, then `GET /views/heatmap?days=28&step=day&format=json` and the HTML of the same view. Recorded as `health_observation` evidence on this story: the date, the column count (28), the row count, the count of cells per kind (scored, hatched, grey, gap) and the command; no series key, family name, instance name or host.
- Deploy: one deploy the operator knows from its own record (a rollout or a merge with a known time), read with `thalamus deploys record` and drawn by `GET /views/deploys/<id>`. Recorded as `health_observation` evidence: the deploy id, its origin, the difference in seconds between the recorded instant and the time the operator knows (zero for a merge, the pod scheduling delay for a rollout), the window, the count of series listed and the count with a delta; no workload, project, series or host name.
- Design § 6.5 gets one dated line naming the two commands and the counts above.

## Domain relations

- `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id` (`spec/domains/evaluation.yaml:110`), and `SignalSample` → `SignalDefinition`, `definition`, via `signal` (`spec/domains/observation.yaml:161`): the heatmap draws real evaluations of real samples.
- `DeployInstant` joined to every series by time only: no relation declared (`story:m4-spec-deploy-instants`, decomposer call).

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 6.5, one dated line; shared; after `story:m4-design-record`, `story:m1-live-pass` and `story:m2-live-observation`, each a `depends_on` with this file as the reason; before `story:m5-design-record` and `story:m5-live-acceptance`)
- Nothing else in this repository; the instance, its definitions and its store stay outside it.

## Acceptance

- `aep plan artifact history story:m4-live-acceptance` lists a `health_observation` record of a 28-column day heatmap of one real instance drawn from stored evaluations after at least 28 days of scheduled sampling.
- It lists a `health_observation` record naming one recorded deploy id, its origin and the offset from the known deploy time, drawn by the before/after view.
- Design § 6.5 states both, dated and naming the command each was read from.
