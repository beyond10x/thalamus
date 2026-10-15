---
format: aep.planning-md/3
id: story:m5-live-acceptance
kind: story
status: draft
title: 'Live: one release asserts its declared set, one test run carries assertions, one transition reaches cortex'
relations:
- decomposes: epic:assertions-and-integrations
- depends_on: story:m5-service-answers
- depends_on: story:m5-declared-sets
- depends_on: story:m5-testrun-assertions
- depends_on: story:m5-cortex-export
- depends_on: story:m2-live-observation
- depends_on: story:m4-live-acceptance
- depends_on: story:m5-design-record
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 6
---
## Outcome

The three acceptance lines of `epic:assertions-and-integrations` hold on real systems: one release asserts its declared set; one test run carries signal assertions and its cases appear as samples; one transition appears in cortex with its evaluation as evidence.

## Context

Every M5 command and its tests are in the other `m5-` stories; this story is the observation on real data, which needs an operator: a real instance sampled by a running `thalamus-service` (the one `story:m2-live-observation` runs, or another), saved Connectors connections on that machine, a cortex instance, and a test runner that writes `thalamus.testrun` events. Thalamus holds no credential. No instance data, series key, family, instance, repository, customer or host name enters this repository (AGENTS.md; the repository is public).

## Work

- Release: after one real rollout, `thalamus assert --declared <file> --after <rollout instant> --asker release:<tag> --dims <env-key>=<env> --wait --format json` for each environment the release reached, answered (not open). Recorded as `health_observation` evidence: the assertion ids, results, worst verdicts and exit codes, and the count of evaluations each cites; no names.
- Test run: one real run whose events are ingested with `thalamus testrun ingest`, and whose declared assertions are emitted into its stream with `--emit-cases` and ingested back. Recorded as `health_observation` evidence: the count of `test.case.passed` samples, the `test.run.pass_ratio` sample id, the assertion case events and their outcomes.
- Cortex: `thalamus cortex export` on the instance, a cortex instance with the sources of `docs/integrations/cortex-sources.yaml`, one `cortex run` of the transitions source, and the store read back (`ekr` query or `cortex` view) showing one `SignalTransition` node whose fact cites `record:files:...:<transition id>` and whose record carries the settling evaluation id. Recorded as `health_observation` evidence: transition id, evaluation id, the cortex evidence id.
- `docs/design/thalamus.md` § 9: one dated line per acceptance line, naming the command it was read from.

## What another repository or the operator must supply

- A test runner that writes `thalamus.testrun` events (`spec/domains/testrun.yaml`). None in this workspace does today; which runner adopts the format is the operator's choice. If none does, this line waits; the fallback is a converter story (for example from JUnit XML), not drafted here.
- Conductor is not needed for these three lines; its side is listed in `story:m5-conductor-contract`.

## Domain relations

| relation | cardinality | ownership | citation |
|---|---|---|---|
| `Assertion` → `SignalEvaluation`, `evaluations` | many | references | declared by `story:m5-spec-integration-records`; design § 11 D2 |
| `Transition` → `SignalEvaluation`, `evaluations` | many | references; outlives them | declared by `story:m2-spec-history-records`; design § 11 D13 |
| a `case.finished` event → a `test.case.passed` sample | one-to-one | references its definition | design § 6.3; `spec/domains/observation.yaml:161` |

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 9; shared; after `story:m4-live-acceptance` and `story:m5-design-record`, both `depends_on`, which edit the same file; the last story that edits it)
- Nothing else in this repository: the instance, the declared-set file, the test run, the cortex instance and their stores stay outside it.

## Acceptance

Each line is one outcome on its own; one can hold while another waits, and the story is done only when all four hold.

- Release: `aep plan artifact history story:m5-live-acceptance` lists a `health_observation` record from a real release (no Connectors double) whose declared set was answered `within` or `breach` (not open, not `unknown` alone), with each assertion id, its result, its worst verdict, its exit code and the count of evaluations it cites.
- Test run: it lists a `health_observation` record from one real test run whose ingested cases are `test.case.passed` samples (their count and the `test.run.pass_ratio` sample id) and which carries at least one assertion case event emitted with `--emit-cases` and ingested back, with its outcome.
- Cortex: it lists a `health_observation` record of one `SignalTransition` fact in a real cortex instance that cites its export record `record:files:...:<transition id>`, with the transition id, the settling evaluation id carried by that record and the cortex evidence id.
- Design: `docs/design/thalamus.md` § 9 carries one dated line for each of the three lines above, each naming the command it was read from.
