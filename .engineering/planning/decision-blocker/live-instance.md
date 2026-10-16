---
format: aep.planning-md/3
id: decision-blocker:live-instance
kind: decision-blocker
status: cleared
title: Nobody has named the real instance the live stories watch, nor who runs their weeks of sampling
relations:
- blocks: story:m1-live-pass
- blocks: story:m2-live-observation
- blocks: story:m4-live-acceptance
- blocks: story:m5-live-acceptance
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T20:48:46Z", actor: "human:timo", revision: 3}
---
## Question

Which real instance do the live stories read, and who runs the multi-week sampling? Four stories are accepted only by observations of a real, watched system over real time: `story:m1-live-pass` (one pass against a real Prometheus through Connectors), `story:m2-live-observation` (7 days of scheduled sampling, one transition opened and cleared), `story:m4-live-acceptance` (28 days of scheduled sampling, one known deploy), `story:m5-live-acceptance` (a real release, a real test run, a real cortex instance).

## Why it blocks

Their acceptance is a `health_observation` or `metric_observation` record of a real instance; a fixture or a double does not count. Which instance, which Connectors connection and which deploys are watched is the operator's choice, and no instance data may enter this repository.

## Steps for the operator

1. Name the instance to watch: a Prometheus reachable through a Connectors connection (the local demo Prometheus counts only if the operator says so), and where its instance directory and store live, outside this repository.
2. For `story:m1-live-pass`: once `story:m1-cli-eval-show` is on `main`, run `thalamus sample --all --once` and `thalamus eval` against it (or let the controller run them, read-only), and keep the `wc -c` / `wc -l` outputs of the sample and evaluation lines.
3. For `story:m2-live-observation`: once `story:m2-scheduler` and `story:m2-transitions` are on `main`, run `thalamus serve` (the scheduler) on that instance for at least 7 days.
4. For `story:m4-live-acceptance`: keep it sampling for at least 28 days and name one deploy with its known time.
5. For `story:m5-live-acceptance`: name one release with a declared set, one test run, and the cortex instance the export goes to.

## What does not wait

Every other story of M1–M5 is accepted locally by its named tests and lands without these.

## Default if unanswered

None taken; the four stories stay draft.

## Answer

Decided 2026-10-15 by the operator: the local demonstration Prometheus (the host's node exporter behind its Connectors connection) is the live instance.

- It is read only; its containers and its sampling timer are not changed.
- The controller runs the `story:m1-live-pass` pass itself, read-only, once `story:m1-cli-eval-show` is on `main`.
- For `story:m2-live-observation` (7 days) and `story:m4-live-acceptance` (28 days) the controller runs `thalamus serve` under a user-level unit of its own, outside the repository, with its instance directory and store outside the repository; the deploys and releases watched are thalamus's own.
- For `story:m5-live-acceptance` a local scratch cortex instance receives the export.
- No instance data enters the repository: evidence carries sizes and counts only.
