# AGENTS.md — thalamus

Instructions for agents changing this repository. Humans start at [README.md](README.md); the
design is [docs/design/thalamus.md](docs/design/thalamus.md).

## What this is

Thalamus samples signals from the systems an organization runs, grades each sample into a verdict
(`OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`) against a baseline, and serves the live verdicts,
history, assertions and events. Cortex is its long-term memory; conductor is its first consumer.

| path | holds |
|---|---|
| `docs/design/` | the design; read it before planning or code |
| `.engineering/` | the AEP planning store |
| `assets/` | the logo |

Planned (design § 5.1): `spec/` (ESS specification), `generated/` (never edited by hand) and the
crates `thalamus-model`, `thalamus-core`, `thalamus-store`, `thalamus-sources`, `thalamus-service`
and `thalamus`.

## Commands

| command | does |
|---|---|
| `aep plan artifact validate` | validate the planning store |

`task check` arrives with the first crate (`Taskfile.yml`, go-task).

## Order of work

1. ESS first: every noun (`SignalDefinition`, `SignalSample`, `SignalEvaluation`,
   `DimensionRegistry`, `Assertion`) is specified under `spec/` before a story that uses it is
   implemented. A story's acceptance is a set of named conformance scenarios.
2. The AEP planning store is the plan. Work is recorded there, never only in a chat or a file.

## Hard rules

- **Committed code is Rust**, command lines with clap derive.
- **No secret enters thalamus.** A source names a Connectors connection id.
- **Read-only against watched systems.** Thalamus never writes to a source.
- **Unknown is never 0 and never OK.** `quality: unknown` carries no value (the `Reading` union); keep it a type-level
  rule, not a convention.
- **One copy of every threshold**, in its definition.
- **No instance data in Git**, and nothing organization-specific: no company, customer, product or
  host names in code, docs, examples, tests or the planning store. The repository is public.
- **Every commit and GitHub write goes through the bot** (`b10x-gates bot`, `b10x-gates api`), after
  `b10x-gates scan-text` on every staged file.
