---
format: aep.planning-md/3
id: story:m6-design-record
kind: story
status: draft
title: 'Record the web UI decisions in the design: the D32 exception, the committed build, the generated TypeScript types'
relations:
- decomposes: epic:web-ui
- depends_on: story:m4-design-record
scope:
- confidence: inferred
  path: docs/design/thalamus.md
revision: 4
---
## Context

`epic:web-ui` adds a Vue 3 + TypeScript layer beside the M4 views and names three decisions design § 11 must record before it is built: the exception to D32 for `web/`, where the built files live and how their drift is checked, and how the TypeScript types are generated from `spec/`. Its sibling stories take further calls that change what § 5.1, § 6.2 and § 6.5 say: two routes, one CloudEvents type, how the browser reads a stream, and how the end-to-end check drives a browser. `docs/design/thalamus.md` is the design every later reader starts from (`AGENTS.md`), so all of them go into it, once, here, the way `story:m4-design-record` recorded M4. It changes no code. The M6 code stories depend on it, because they implement what it records.

`docs/design/thalamus.md` is shared. `story:m4-design-record` (implemented) came first. `story:m1-live-pass`, `story:m2-live-observation`, `story:m4-live-acceptance`, `story:m5-design-record` and `story:m5-live-acceptance` also edit it. This story carries no `depends_on` edge to them, so M6 does not wait behind the 28-day M4 observation; the shared path in its scope keeps it out of any wave that holds one of them (decomposer call).

## Work: the decisions to record (decomposer calls, reversible)

1. **The D32 exception.** D32 and D40 hold for the M4 views: they stay rendered by Rust, with no `<script>` element and the 10 s refresh. `web/` is the one browser-script layer: a Vue 3 + TypeScript app built at development time with Node and npm (the package manager `website/` already uses), committed as built files and served by Rust. Node is needed to change or check `web/`, never to build or run the service: `cargo build` alone yields a binary that serves the UI. `task check` runs the web checks, so CI installs a pinned Node. The only TypeScript in the repository is the browser app under `web/`; no TypeScript or Node runs in the service, a test harness or a gate (the end-to-end check is a Rust test, item 6).
2. **Built files.** `vite build` writes `web/dist/`, which is committed and embedded in `thalamus-service` at compile time (`include_dir`): no `--assets` option and no file beside the binary. Served under `/ui/`: `GET /ui` redirects to `/ui/`, `GET /ui/{*path}` answers one embedded file or 404. The app routes by URL hash (`/ui/#/series/<key>`), so the server never maps an unknown path to `index.html`. Every `/ui/` answer carries `Content-Security-Policy: default-src 'self'`, so the page fetches nothing but the serving `thalamus-service`; other methods answer 405 as on every route (D31). Drift: `task web-check` runs `npm ci`, the type check (`vue-tsc`), the unit tests and a fresh `vite build` into a scratch directory, then `diff -r` against `web/dist/`; any difference fails `task check`. `web/dist/` is rebuilt by the coordinator at merge, as `Cargo.lock` is regenerated.
3. **TypeScript types.** ESS's TypeScript output, as uilab uses it: `task generate` runs `ess generate types --target typescript` with one `--root` per record the API and the streams carry (the records `SignalSample`, `SignalEvaluation`, `Transition`, `Assertion`; the view rows `Evaluations`, `SeriesSamples`, `SeriesTransitions`, `Assertions`) into `web/src/generated/`, and `task drift` diffs it as it diffs `generated/`. One generator for the Rust and the TypeScript side; no generator in `thalamus-docs`. The CloudEvents 1.0 envelope (D33) and the `{samples, evaluations}` answer of `/api/series` are generic containers written once in `web/src/lib/wire.ts` over generated record types: transport, not records, so no ESS noun is added. The Rust handler tests write their answers as golden files under `web/fixtures/`, and the web unit tests validate each against the generated JSON Schema, so the two sides cannot drift apart unnoticed. If ESS refuses an entity or a view as a root, the gap is filed in the ESS repository and the types wait for the release that closes it.
4. **Two routes and one event type for the UI.** `GET /api/transitions?key=<series key>&from=<ts>&to=<ts>`: the `SeriesTransitions` rows of one series over `[from, to)` on `settled_ts`, default the last 7 days, an unknown series key 404 (`story:m6-series-view`). `GET /api/flow/stream`: for each recorded evaluation, its sample, the evaluation and, when that evaluation settled a transition, the transition, as three CloudEvents in that order; the type `dev.beyond10x.thalamus.sample` joins the two kinds of D33; the D34 lag rule applies; `/api/evaluations/stream` is unchanged (`story:m6-flow-view`).
5. **Reading a stream in the browser.** `fetch` with a streaming server-sent-events parser, not `EventSource`, because `EventSource` drops SSE comments and the D34 lag comment must show as its own state, distinct from a reconnect (`story:m6-web-types`).
6. **Checks.** Logic lives in plain `web/src/lib/*.ts` modules tested with `node --test`; `.vue` files stay thin and are checked by `vue-tsc` and by the end-to-end check. No test framework, router or chart library: a hash router in `web/src/lib/route.ts`, SVG drawn in templates. The end-to-end check is a Rust test of `thalamus-service`, behind the cargo feature `web-e2e`, that serves a fixture instance on a loopback port and drives headless Chromium over the DevTools protocol; the browser is a disposable container pinned by digest, as `task test-postgres` runs PostgreSQL, or the one `THALAMUS_TEST_CHROME_URL` names (`story:m6-web-skeleton`, `story:m6-web-e2e`).

Where each goes:

- § 5.1: the `thalamus-service` line and the route paragraph gain `/ui/`, `/api/transitions` and `/api/flow/stream`; one paragraph names `web/` (items 1 to 3).
- § 6.2: the sample event type (item 4).
- § 6.5: one row each for the signal board, the series view and the flow view.
- § 9: an M6 row naming `epic:web-ui`.
- § 11: one row per item above, numbered from the next free `D` when this story lands, each citing the `story:m6-` id that implements it, under a heading dated the day it lands. D32 and D40 are amended by reference, not rewritten.

## Domain relations

None read or written. The relations each M6 story reads are cited in that story.

## Scope

Planned (inferred):

- `docs/design/thalamus.md` (§ 5.1, § 6.2, § 6.5, § 9, § 11; shared; after `story:m4-design-record`; kept apart from `story:m4-live-acceptance`, `story:m5-design-record` and `story:m5-live-acceptance` by scope, not by an edge)

## Acceptance

Each of these fails before the change and passes after it (revised after review-result:m6-acceptance-round-1 and review-result:m6-acceptance-round-2); every check is scoped to its section:

- `sed -n '/^### 5\.1 Crates/,/^### 5\.2/p' docs/design/thalamus.md | grep -cE "/ui/|/api/transitions|/api/flow/stream"` is at least 3, one line of § 5.1 for each;
- `sed -n '/^### 6\.2 Events/,/^### 6\.3/p' docs/design/thalamus.md | grep -c "dev.beyond10x.thalamus.sample"` is at least 1;
- `sed -n '/^### 6\.5 Views/,/^## 7\./p' docs/design/thalamus.md | grep -cE "signal board|series view|flow view"` is at least 3, one row per view;
- `sed -n '/^## 9\. Milestones/,/^## 10\./p' docs/design/thalamus.md | grep -c "epic:web-ui"` is at least 1;
- `sed -n '/^## 11\./,$p' docs/design/thalamus.md | grep -c "story:m6-"` is at least 6, one § 11 row per decision under Work.

`b10x-gates scan-text docs/design/thalamus.md` also passes after the change (a guard, not a failing-first check).
