---
format: aep.planning-md/3
id: story:m4-deploy-instants
kind: story
status: draft
title: Deploy instants are read from pod creation and merge time and recorded
relations:
- decomposes: epic:views-and-exports
- depends_on: story:m4-spec-deploy-instants
- depends_on: story:m4-skeleton
- depends_on: story:m1-connectors-client
- depends_on: story:m1-cli-sample
- depends_on: story:m1-instance-registry
- depends_on: story:m2-transitions
scope:
- confidence: inferred
  path: crates/thalamus-cli/src/cmd/deploys.rs
- confidence: inferred
  path: crates/thalamus-cli/tests/deploys.rs
- confidence: inferred
  path: crates/thalamus-sources/src/deploys/gitlab.rs
- confidence: inferred
  path: crates/thalamus-sources/src/deploys/kubernetes.rs
- confidence: inferred
  path: crates/thalamus-sources/src/deploys/reader.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/deploys.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/fixtures/deploys
- confidence: inferred
  path: crates/thalamus-store/src/deploys.rs
- confidence: inferred
  path: crates/thalamus-store/tests/conformance.rs
revision: 4
---
## Context

Design § 6.5: "A **deploy instant** is the time a workload actually changed (pod creation, merge time), never an announcement." `story:m4-spec-deploy-instants` specifies `thalamus.deploy.DeployInstant`; this story reads instants through Connectors and records them, so `story:m4-before-after` can draw one. § 5.2: read only through a Connectors connection, no secret, never a write. The Connectors client, its read-operation allow-list and the recorded double come from `story:m1-connectors-client`; the global options and the instance loader come from `story:m1-cli-sample`; dimension admission comes from `story:m1-instance-registry`. The M3 adapters name these reads and leave deploy instants to M4: `story:m3-kubernetes-adapter` ("Deploy instants from `deployments.history` and pod creation (M4)") and `story:m3-git-adapter` ("merge time as a deploy instant (M4)"). This story edits neither adapter file.

## Connectors

As those two stories record at Connectors v0.45.0: the Kubernetes adapter offers `resources.list` (kinds include `pods` and `replicasets`); GitLab-hosted repositories are read through the catalog provider, `merge_requests.list`, one page per call. Both are reads.

## Work

- `crates/thalamus-sources/src/deploys/reader.rs`: the deploy reader trait; each reader declares its read operations to the client allow-list; `deploy_id` derived from origin, connection and `ref` (the `UNMAPPED:` rule of `spec/domains/deploy.yaml`).
- `crates/thalamus-sources/src/deploys/kubernetes.rs`: origin `pod_created`. Pods and ReplicaSets listed in the requested namespaces; the pods of one ReplicaSet are one rollout; its instant is the earliest pod `metadata.creationTimestamp`; `workload` the owning Deployment, `ref` the ReplicaSet name and its revision annotation. A rollout whose pods are no longer listed is not recorded, and an object without a creation timestamp is never given a guessed one.
- `crates/thalamus-sources/src/deploys/gitlab.rs`: origin `merged`. Merge requests in state merged over the requested window, paged until the window is covered; instant `merged_at`; `workload` the project as the source names it, `ref` the merge request reference. Open or closed-unmerged merge requests are not deploys.
- dims on an instant are set only from a field the source object carries and only for a key the instance registry declares (`namespace` from Kubernetes metadata); a value that does not resolve in the registry refuses that instant, which is reported and not recorded (L9, design § 2.4).
- `crates/thalamus-store/src/deploys.rs`: the store port, append through the generated `RecordDeployInstant`; read one by id; list over a time range in `ts` order.
- `crates/thalamus-store/tests/conformance.rs` (owned by `story:m1-eventlog-store`, last changed by `story:m2-transitions`): one selection line adding the `RecordDeployInstant` scenarios with the Eventlog storage.
- `crates/thalamus-cli/src/cmd/deploys.rs`: `thalamus deploys record --reader kubernetes|gitlab --source <adapter> --connection <id> [--namespace <ns>]... [--project <p>] [--since <duration>]` prints each recorded or already-recorded id; `thalamus deploys list [--since <duration>] [--format text|json]`. Exit 0 when every read instant was recorded or already recorded; refusals use the codes of `story:m1-cli-sample` (65, 74).

## Decisions taken here (decomposer call, reversible)

- Pod creation is read from the pods still listed; a ReplicaSet creation time is not used as a stand-in, because the design names pod creation. Instants are recorded by running `thalamus deploys record` after a deploy; recording them on the scheduler is left for later.
- GitLab-hosted repositories only, the one repository surface Connectors offers (`story:m3-git-adapter`).
- The design § 2.4 dimension rule applies to deploy instants as to samples (`story:m4-spec-deploy-instants`).

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `DeployInstant.dims` keys → registered `Dimension` | many keys per instant | none | the `UNMAPPED:` marker on `dims` in `spec/domains/deploy.yaml` (`story:m4-spec-deploy-instants`), the pattern of `spec/domains/observation.yaml:142`; decomposer call |
| `Dimension` → `Dimension`, `parent_dimension`, via `parent`: a `source_only` key is never derived from its parent | many-to-one | references | `spec/domains/dimension.yaml:92` |
| `DimensionRegistry` → `Dimension`, `dimensions`, via `registry_id` | one-to-many | registry owns | `spec/domains/dimension.yaml:62` |

## Scope

Planned (inferred):

- `crates/thalamus-sources/src/deploys/reader.rs`, `crates/thalamus-sources/src/deploys/kubernetes.rs`, `crates/thalamus-sources/src/deploys/gitlab.rs` (stubs from `story:m4-skeleton`)
- `crates/thalamus-store/src/deploys.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-store/tests/conformance.rs` (one selection line; ordered after `story:m2-transitions`)
- `crates/thalamus-cli/src/cmd/deploys.rs` (stub from `story:m4-skeleton`)
- `crates/thalamus-sources/tests/deploys.rs`, `crates/thalamus-sources/tests/fixtures/deploys`, `crates/thalamus-cli/tests/deploys.rs` (new; neutral names only)

## Acceptance

- `cargo test -p thalamus-sources --test deploys` passes, over recorded Connectors answers in `crates/thalamus-sources/tests/fixtures/deploys/`, the failing-first tests `earliest_pod_creation_of_a_rollout_is_its_instant`, `merge_time_is_the_instant`, `unmerged_merge_request_is_not_a_deploy`, `object_without_creation_timestamp_is_not_recorded`, `deploy_dims_only_from_source_fields`, `deploy_id_is_stable_for_the_same_change` and `deploy_readers_invoke_only_read_operations` (the double argv log).
- `cargo test -p thalamus-cli --test deploys` passes `record_twice_reports_already_recorded`, `unresolved_dim_value_refuses_the_instant` and `list_is_in_ts_order`.
- `cargo test -p thalamus-store --test conformance` passes `thalamus.deploy.RecordDeployInstant/outcome/recorded` and `thalamus.deploy.RecordDeployInstant/outcome/already-recorded` with the Eventlog storage.

Open points found by story:m4-design-record (wave 20261015e); coordinator defaults, each recorded as a design § 11 row when this story lands:

- The spelling of `deploy_id` from its stated inputs is the implementor's; the § 11 row states it.
