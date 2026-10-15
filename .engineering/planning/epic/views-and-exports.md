---
format: aep.planning-md/3
id: epic:views-and-exports
kind: epic
status: draft
title: M4 · Heatmap, live and before/after views, and dashboard exports
relations:
- depends_on: epic:history-and-transitions
revision: 1
---
## Outcome

An operator sees every signal of an instance in one heatmap, live and over weeks.

## Ships

- heatmap, sparklines, week vs week, before/after deploy, live view in `thalamus-service`
- dashboard JSON and Prometheus rule export for Prometheus and Loki definitions

## Acceptance

- a 4-week heatmap rendered from stored samples
- one known deploy visible in before/after

## Design

`docs/design/thalamus.md` § 9.
