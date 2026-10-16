---
format: aep.planning-md/3
id: epic:web-ui
kind: epic
status: draft
title: M6 · A Vue web UI shows the data flowing through thalamus, generically and live
summary: A read-only Vue 3 + TypeScript layer, served by thalamus-service, renders every sample, evaluation and transition from the API and the evaluation stream without per-signal code.
relations:
- depends_on: epic:views-and-exports
- informed_by: epic:assertions-and-integrations
revision: 1
---
## Outcome

An operator opens one browser page served by `thalamus-service` and watches the data flowing
through an instance as it happens: every sample, evaluation and settled transition, for every
signal, without a line of per-signal UI code. The page is generic (it renders whatever the
instance's definitions and the specification's types say) and live (it updates from the
evaluation stream, not by reloading).

Operator request, 2026-10-15: "plan for a dedicated webui layer (vue) which shows the data in a
generic and realtime nature … I mean the data flowing through thalamus".

## Context

- M4 ships server-rendered views with no browser script: design § 11 D32 ("HTML and SVG are
  rendered by Rust … no Node build and no `<script>` element") and D40 (the live page refreshes
  every 10 s by `<meta http-equiv="refresh">`; "a browser script is the obvious later change").
  This epic is that later change, as a separate layer beside the M4 views, not a rewrite of them.
- Precedent in the organisation: `uilab` keeps a Vue 3 + TypeScript browser app in `widget/`,
  "served as built files by the Rust binary" (`~/beyond10x/uilab/AGENTS.md` § Rules), with wire
  types generated for TypeScript (`widget/src/generated/`). Running code stays Rust; the browser
  bundle is the one TypeScript part, as there.
- Data the layer reads, all from M4 and M5 stories:
  - `GET /api/verdicts` and `GET /api/series` (story:m4-service-api),
  - `GET /api/evaluations/stream`, `text/event-stream`, one CloudEvents envelope per recorded
    evaluation, no replay on reconnect (story:m4-live, D33, D34),
  - `GET /api/assertions` (story:m5-skeleton).
- The service listens on loopback only unless `--allow-remote`, with no authentication through M5
  (D31, D43).

## Ships

- A `web/` Vue 3 + TypeScript app, built at development time; the built files are committed and
  served by `thalamus-service` under one route prefix, so running the service needs no Node.
- TypeScript types for every record the API and the stream carry, generated from the thalamus ESS
  specification and drift-checked by `task check`; no hand-written record type.
- A generic signal board: every series of the instance with its current verdict, score and age,
  grouped and filtered by family, signal and dimension values read from the data, updated live.
- A series view: samples, evaluations and settled transitions of one series over a time range,
  with the stream appending new points live.
- A live flow view: the stream as it arrives (sample → evaluation → transition), with pause and
  filter, and a visible state when the stream lags or reconnects (D34: a lagging subscriber is
  closed and rereads `/api/verdicts`).
- Its own checks in `task check`: type check, unit tests, and an end-to-end check that the built
  page renders a fixture instance's data from a running service.

## Not in scope

- Editing definitions, muting or any write: the definition file is the only copy (D19); the UI
  is read-only.
- Authentication, remote exposure beyond `--allow-remote`, multi-instance views.
- Replacing the M4 server-rendered views or the dashboard export.
- A CDN or any network fetch other than the serving `thalamus-service`.

## Decisions this epic needs recorded (design § 11)

- D32's "no `<script>`" and "no Node build" hold for the M4 views; the `web/` layer is the one
  exception, built at development time with Node, committed as built files, served by Rust.
- Where the built files live and how drift between `web/` sources and the built files is checked.
- How the TypeScript types are generated from the specification (ESS's TypeScript output, as
  uilab does, or a generator in `thalamus-docs`).

## Depends on

story:m4-skeleton, story:m4-service-api and story:m4-live (routes, API, stream); story:m5-skeleton
for the assertion routes, which the board shows when present.
