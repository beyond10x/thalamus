---
format: aep.planning-md/3
id: story:m3-git-adapter
kind: story
status: implemented
title: GitLab-hosted repository records become count and ratio samples
relations:
- decomposes: epic:source-adapters
- depends_on: epic:signal-contract
- depends_on: story:m1-connectors-client
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/git.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/git
- confidence: inferred
  path: crates/thalamus-sources/tests/git.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T05:44:44Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T05:44:44Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-15T06:54:17Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Context

Design § 5.1 lists Git among the `thalamus-sources` adapters; § 0 names version control among the instruments thalamus reads. § 5.2: read only through a Connectors connection, no secret, never a write. This story adds the `git` adapter on the M1 adapter trait and Connectors client (`epic:signal-contract`). Code is Rust (AGENTS.md).

## Connectors

Partly exists. Connectors v0.45.0 (`94e0805ed`) has no native Git adapter and no GitHub provider: `docs/fluxplane-plugin-parity.md:842` reads "Connectors has no local git adapter. The only repository surface is the GitLab-hosted reads." GitLab-hosted repositories are served by the catalog provider, selection set `adapters/catalog/providers/gitlab/operations.json`: reads `commits.list`, `merge_requests.list`, `pipelines.list`, `tags.list`, `deployments.list`, one page per call (`per_page` at most 100).

This story builds the adapter over those GitLab reads. A GitHub-hosted or local repository needs a Connectors adapter that does not exist; that work belongs to the Connectors repository and is not part of this story.

## Behaviour

- `git.count`: the number of records inside the definition window that match its predicate (merged merge requests, commits, tags), one sample per series.
- `git.ratio`: matching records over all records in the window, for example failed pipelines over pipelines; a `Decimal` with unit `1` (L4); `denominator.n` is the record count.
- Pages are read until the window is covered. A read that stops before it is covered (page budget, refusal mid-way): `partial`, bound `lower` for a count.
- A `dims` key is set only from a field the record carries and the definition declares (L9).
- A Connectors refusal or provider failure before any page: `quality: unknown`; the reason carries the Connectors code and stage.

## Domain relations

- `SignalSample → SignalDefinition`, many-to-one; the sample references the definition, which exists first and outlives it. Inferable: `spec/domains/observation.yaml`, entity `thalamus.observation.SignalSample`, relation `definition` (references, cardinality one, via `signal`).
- `dims` keys → registered `Dimension`: no `relations:` entry; `spec/domains/observation.yaml` field `dims` carries an `UNMAPPED:` marker, decided by design § 2.4 and held by an M1 Rust test. Relied on, not implemented here.
- `SignalDefinition.source` → a Connectors connection: the typed field `thalamus.definition.Source` in `spec/domains/definition.yaml`, not an entity relation.

## Acceptance

`cargo test -p thalamus-sources --test git` passes, with these known-answer tests over recorded GitLab catalog answers in `crates/thalamus-sources/tests/fixtures/git/`, each seen failing before the adapter exists: `git_merged_count_in_window_known_answer`, `git_pipeline_failure_ratio_is_decimal_with_unit_1`, `git_unfinished_paging_is_partial_lower_bound`, `git_refused_read_is_unknown_with_reason`.

## Scope

All paths `inferred` (no code exists yet):

- `crates/thalamus-sources/src/git.rs` (replaces the one-line `adapter()` stub `story:m1-connectors-client` leaves there; the module line is `story:m0-workspace`'s and the registry arm `story:m1-connectors-client`'s, so no other file changes)
- `crates/thalamus-sources/tests/git.rs` (new)
- `crates/thalamus-sources/tests/fixtures/git` (new)

## Not in this story

GitHub-hosted and local repositories (no Connectors adapter); merge time as a deploy instant (M4, design § 6.5).
