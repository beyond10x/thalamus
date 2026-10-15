---
format: aep.planning-md/3
id: story:m1-live-pass
kind: story
status: draft
title: One read-only live pass against a real Prometheus measures the sizes in design section 4
relations:
- decomposes: epic:signal-contract
- depends_on: story:m1-cli-eval-show
- depends_on: story:m4-design-record
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 2
---
## Outcome

The size numbers of design § 4 are measured from one read-only pass of thalamus against a real Prometheus, not inferred.

## Context

`epic:signal-contract` acceptance: "one live read-only pass against a real instance; its line count and byte size replace the inferred numbers in design § 4". Design § 4 today: "about 300 B per sample (inferred)" and "about 2.4 MB a day (inferred)" for 26 scopes sampled hourly; "M1 replaces every inferred number with a measured one". Needs everything of M1 (`story:m1-cli-eval-show` and what it depends on) and an operator: a Connectors `prometheus` adapter instance with a saved, validated connection on the machine that runs the pass (Connectors `adapters/prometheus/README.md`). Thalamus holds no credential; the connection does.

## Work

- An instance directory outside this repository with at least five `Active` definitions read from that Prometheus (at least one `promql.ratio`, one `promql.value`), a `file:` or `open` dimension registry, and the File store provider (`--store file:<dir>`), so the stream is JSONL.
- `thalamus sample --all --once`, then `thalamus eval`, then `thalamus show`, run once; the Connectors double is not used.
- Read-only check: the operations Connectors invoked in the pass are only `series.query` and `series.query_range` (from the Connectors audit or `connections` listing, and the client allow-list of `story:m1-connectors-client`).
- Measure with `wc -l` and `wc -c` on the stream file: lines and bytes per sample line, per evaluation line, per pass; the daily volume for design § 4 fleet example is computed from the measured per-line bytes and says so.
- Edit design § 4 "Size": each number carries the date, the definition and series counts and the command it was read from; the word "inferred" goes from every number this pass measured. No host, company, customer or product name, and no instance data, enters the repository.

## Domain relations

- `thalamus.observation.SignalSample` → `thalamus.definition.SignalDefinition`, one, references via `signal`, and `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample`, one, references via `sample_id`. Inferable: `spec/domains/observation.yaml`, relation `definition`; `spec/domains/evaluation.yaml`, relation `sample`. One evaluation line per sample line at one revision (`spec/domains/evaluation.yaml:224`, design § 11 D5) is what makes per-pass size the sum of the two.
- `thalamus.definition.SignalDefinition.source` → one Connectors adapter instance and connection. Inferable, inferred from cortex `src/connectors.rs:209`, as recorded in `story:m1-connectors-client`.

## Scope

Planned (inferred):

- `docs/design/thalamus.md` § 4 (the file exists)
- Nothing else in this repository; the instance and its store stay outside it.

## Acceptance

Design § 4 states the bytes per sample line, the bytes per evaluation line and the lines per pass measured from one dated `thalamus sample --all --once` and `thalamus eval` run against a real Prometheus through Connectors, each naming the `wc` command it was read from, and `grep -n "inferred" docs/design/thalamus.md` shows no line of § 4 whose number that pass measured.

## Out of scope

The CloudEvents envelope size of design § 6.2 (M2); a fleet-wide pass; the scheduler.
