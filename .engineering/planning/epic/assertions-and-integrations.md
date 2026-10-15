---
format: aep.planning-md/3
id: epic:assertions-and-integrations
kind: epic
status: draft
title: M5 · Assertions, conductor, test runs and cortex
relations:
- depends_on: epic:history-and-transitions
revision: 1
---
## Outcome

Releases, merge requests, test runs and conductor ask thalamus whether named signals stayed in baseline, and cortex remembers the transitions.

## Ships

- `thalamus assert`; `Asserts:` lines
- conductor integration
- test-run bridge both ways
- thalamus as a cortex source

## Acceptance

- one release asserts its declared set
- one test run carries signal assertions and its cases appear as samples
- one transition appears in cortex with its evaluation as evidence

## Design

`docs/design/thalamus.md` § 9.
