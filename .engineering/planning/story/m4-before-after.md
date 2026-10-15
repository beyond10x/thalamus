---
format: aep.planning-md/3
id: story:m4-before-after
kind: story
status: draft
title: Before/after view ranks every series by score delta around one deploy instant
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-deploy-instants
- depends_on: story:m4-heatmap
scope:
- confidence: inferred
  path: crates/thalamus-service/src/views/deploy.rs
- confidence: inferred
  path: crates/thalamus-service/tests/before_after.rs
revision: 5
---
## Context

Design § 6.5, before/after a deploy: "every signal window before vs after one deploy instant, sorted by score delta", data: samples and deploy instants. Epic acceptance: "one known deploy visible in before/after". Deploy instants are recorded by `story:m4-deploy-instants` and read through its store port `crates/thalamus-store/src/deploys.rs`. This story fills `crates/thalamus-service/src/views/deploy.rs` (stub and route lines from `story:m4-skeleton`) and reuses `svg.rs` of `story:m4-heatmap`.

Its tests use the in-process HTTP technique of `story:m4-skeleton` (`http::router` over a seeded store, `tower::ServiceExt::oneshot`, bodies read with `http-body-util`); this story adds no dev-dependency and does not edit `crates/thalamus-service/Cargo.toml`.

## Work

- `GET /views/deploys?since=<duration>&format=html|json`: the recorded deploy instants, newest first, each with origin, `ts`, `workload`, `ref` and dims, linking to its page.
- `GET /views/deploys/{id}?window=<duration>&format=html|json`: for every series with an evaluation in either window, before = `[ts - window, ts)`, after = `[ts, ts + window)`. Per side: the highest score among scored evaluations (one per sample at the highest revision present) and the median of the `ok` readings of the samples. Delta = after score - before score. A side with no scored evaluation is `UNKNOWN` with a reason and has no delta, never 0. Rows sorted by delta descending, rows without a delta last. An unknown id answers 404.
- `window`: default 2h when the parameter is absent; at most 7d. A window over 7d, a zero or negative window, or an unparseable duration answers 400 with a body naming the parameter and the 7d cap; nothing is read from the store for it. The JSON answer carries the `window` it used.

## Decisions taken here (decomposer call, reversible)

- The sorted score delta is computed from the stored evaluations of each window (the highest score per side), and the sample values are shown beside it. The design data column names samples; grading a window of samples against another would re-run grading with a second baseline, which M4 does not need.
- Every series, not only series sharing a dimension value with the deploy: the design says "every signal". The deploy dims are shown so a reader can match them.
- Default window 2h, the horizon of the design § 6.1 assertion example. Cap 7d, one week either side, so a request cannot ask for more than the raw retention covers on both sides (design § 4, 35 days).

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `DeployInstant` → series | one instant, every series | none; joined by time only | no relation declared (`story:m4-spec-deploy-instants`); decomposer call |
| `SignalEvaluation` → `SignalSample`, `sample`, via `sample_id`; one per sample and revision | many-to-one | references | `spec/domains/evaluation.yaml:110`, `spec/domains/evaluation.yaml:224` (design § 11 D5) |
| `SignalSample` → `SignalDefinition`, `definition`, via `signal` | many-to-one | references | `spec/domains/observation.yaml:161` |

## Scope

Planned (inferred):

- `crates/thalamus-service/src/views/deploy.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-service/tests/before_after.rs` (new)

## Acceptance

`cargo test -p thalamus-service --test before_after` passes, against a store seeded with evaluations around one recorded deploy instant and no source reachable, the failing-first tests:

- `every_series_with_evaluations_in_either_window_is_listed`, `rows_sorted_by_score_delta_descending`, `side_without_scored_evaluation_is_unknown_never_zero`, `evaluation_at_the_instant_belongs_to_after`, `sample_medians_from_ok_readings_only`, `deploy_list_links_each_recorded_instant` and `unknown_deploy_id_answers_404`;
- `default_window_is_2h` (no `window` parameter: the JSON answer carries `window` 2h, and an evaluation 2h 1m before the instant is outside the before side while one 1h 59m before is inside);
- `window_over_7d_is_refused_with_400` (`window=7d` answers 200; `window=8d` answers 400 naming the 7d cap).

## Revision after acceptance critic round 2

Added failing-first tests: `zero_or_negative_window_is_refused_with_400` and `unparseable_window_is_refused_with_400`, each asserting nothing was read from the store.
