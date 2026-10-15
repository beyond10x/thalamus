---
format: aep.planning-md/3
id: review-result:adversary-m4-design-record-pass-1
kind: review-result
status: active
title: Adversary, story:m4-design-record, pass 1
relations:
- reviews: story:m4-design-record
revision: 1
---
unit: story:m4-design-record, working tree of impl/m4-design-record (base 6b98444, uncommitted change to docs/design/thalamus.md)
verdict: NEEDS-CHANGE
cases: executed 420→420, red 0 (no case added: the unit changes one document, and its checks are the greps below)
origin: introduced 5 / pre-existing 3 / undecided 0
wrote-outside-worktree: 16 paths (part 6)
needs-coordinator: F2 (§ 10 question 2 covers M1–M2, and D31 relies on it for M4); F3 (D35 says "every view" answers format=json, and story:m4-live gives /views/live no format)

## 1. git --no-pager diff --stat (worktree)

```
 docs/design/thalamus.md | 88 +++++++++++++++++++++++++++++++++++++++++++++----
 1 file changed, 81 insertions(+), 7 deletions(-)
```

That diff is the implementor's. The adversary wrote nothing in the worktree.

## 2. Cases added

None. The unit has no test surface. The acceptance checks are greps, and I ran them against both base and tree:

| check | base 6b98444 | tree |
|---|---|---|
| `grep -nE "export (rules\|dashboard)\|deploys (record\|list)\|--listen"` | 0 lines | § 5.1 :244, :245 (all three); also :356, :365, :370, :507, :508, :516 |
| `grep -nE "pod_created\|merged"` | 0 lines | § 6.5 :357, :358; :516 |
| `grep -c "story:m4-"` | 0 | 13 (D30–D42, one row each) |
| `b10x-gates scan-text … docs/design/thalamus.md` | — | `1 file(s) carry no private rule`, EXIT=0 |
| table column count (escaped pipes removed) in § 5.1, § 6.5, § 11 | — | 64 rows, all 3 columns |

## 3. Suite run

`task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess` gave `EXIT=0`: 62 `test result` lines, 420 passed, 0 failed. The last lines read `website/docs: no raw admonition titles`. No step of the gate reads docs/design/thalamus.md (the only "design" match in the log is `ucum_admits_design_units`), so this green says nothing about this unit.

## 4. Findings (tree: impl/m4-design-record working tree over 6b98444)

| # | finding | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | invented decision | docs/design/thalamus.md:507 (D30) puts `thalamus export dashboard [--out <file>]` in one parenthetical, "(stdout by default, the skipped list with reasons on stderr, exit 0, …)". story:m4-export-dashboard Work says only "`thalamus export dashboard [--out <file>]`; stdout by default; exit codes as `export rules`" (`grep -c stderr` = 0 in that story) | the implementor of story:m4-export-dashboard and its CLI test `export_dashboard_writes_json_that_parses` | NEEDS-CHANGE | introduced |
| F2 | contradicts § 10 | :254 "There is no authentication in M4 (§ 10 question 2)" and :508 (D31) "(§ 10 question 2: the service runs per operator machine)". § 10 at :436 reads "per machine for M1–M2; an in-cluster deployment is a later milestone". The default the row cites does not cover M4. story:m4-skeleton cites § 10 the same way, so the row is faithful to its story | the `--allow-remote` / no-auth stance of every M4 HTTP route | NEEDS-CHANGE (coordinator: extend § 10 default to M4, or reword D31) | pre-existing |
| F3 | stories disagree, and the design states one side | :320 says "Every view answers `format=html` or `format=json` … (D35)", from story:m4-heatmap ("`format=json` on every view"). story:m4-live Work line 36, "`GET /views/live`: one row per series …", names no `format`, and none of its acceptance tests asks for JSON | the m4-live implementor; the § 6.5 claim is false for /views/live unless m4-live adds it | NEEDS-CHANGE (coordinator) | pre-existing |
| F4 | misstated cell rule | :328 says "a column with no evaluation is a gap". story:m4-heatmap Cell function says "of the evaluations whose `ts` falls in the column … no evaluation is a gap", which is per cell (series × column). D35 at :512 says it correctly | a reader of § 6.5 who draws whole empty columns only | NEEDS-CHANGE | introduced |
| F5 | decision dropped | :344–345 "Per side: the highest score among its scored evaluations", and D38 at :515 "(the highest score per side)". story:m4-before-after Work says "the highest score among scored evaluations (one per sample at the highest revision present)". The heatmap bullet (:325–326) keeps that revision rule and before/after drops it | the m4-before-after implementor working from the design: a re-graded sample counts twice | NEEDS-CHANGE | introduced |
| F6 | overstated | :243 says "the one CloudEvents envelope builder (`cloudevents.rs`) the service and the CLI call". story:m4-live says "`thalamus-cli` (`story:m5-conductor-contract`) can call it", and D33 at :510 says "can call". In M4 the CLI does not call it | readers of § 5.1 | NEEDS-CHANGE | introduced |
| F7 | design left incomplete | § 5.1 :241 still reads `thalamus-store` "the append-only sample stream and rollups" and :242 `thalamus-sources` "adapters that turn a Connectors operation into samples". story:m4-skeleton adds `thalamus-store/src/deploys.rs` and `thalamus-sources/src/deploys/` (deploy readers, which produce no samples). § 4 (:217–222) has no record row for deploy instants, while D39 says "Kept without a retention limit" | readers of § 4 and § 5.1 | CONFIRMED | introduced |
| F8 | CLI list incomplete on a rewritten line | :245 lists the `thalamus` subcommands without `flaps`, and `crates/thalamus-cli/src/main.rs:36` has `Flaps(cmd::flaps::Args)` | readers of § 5.1 | CONFIRMED | pre-existing |

## 5. Attacked and not broken

- Each "Decisions taken here" item of the ten cited stories (skeleton 4, live 3, heatmap 3, sparklines 2, week 2, before-after 3, spec-deploy 3, deploy 3, export-rules 3, export-dashboard 2) has a row in D30–D42. No decision is missing.
- story:m4-service-api is not cited, and it records no "Decisions taken here"; its routes are in § 5.1 :251.
- No new row contradicts D1–D29. D14, D10, D12 and D19 are cited and quoted correctly.
- § 6.2 envelope fields agree with D33 and D34.
- Heatmap scale, live meta refresh, the before/after window bounds (2h default, 7d cap, 400/404), the week slots (168, start of hour of `at`, `baseline predates series`), the deploy origins and id derivation, export naming, `for:`, the skip list and the Grafana model all match their stories.
- Ownership of `thalamus.deploy` by `thalamus-service`, with the CLI as writer, follows the existing pattern in spec/components.yaml (samples are the same).
- The acceptance greps are not trivial: all fail on base 6b98444. The `grep -c "story:m4-" >= 9` check is weaker than "one row per decision", but the rows exist.
- Table integrity holds: 64 rows, 3 columns each.
- scan-text is clean.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary/{epic,m1-cli-sample,m4-before-after,m4-deploy-instants,m4-export-dashboard,m4-export-rules,m4-heatmap,m4-live-acceptance,m4-live,m4-service-api,m4-skeleton,m4-sparklines,m4-spec-deploy-instants,m4-week-vs-week}.md (14 story dumps)
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary/task-check.log
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-pass-1.md (this report)

Build output of `task check` went to the worktree's own `target/`.

## 7. Findings block

```findings
- file: docs/design/thalamus.md
  line: 507
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: D30 gives `export dashboard` a skipped list on stderr that story:m4-export-dashboard never states (it says only stdout by default and exit codes as export rules)
- file: docs/design/thalamus.md
  line: 508
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: D31 and :254 rest no-auth in M4 on § 10 question 2, whose default at :436 covers only M1–M2
- file: docs/design/thalamus.md
  line: 320
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: § 6.5 says every view answers format=json (D35, story:m4-heatmap) but story:m4-live gives /views/live no format parameter and no JSON test
- file: docs/design/thalamus.md
  line: 328
  category: acceptance
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: § 6.5 says a column with no evaluation is a gap where story:m4-heatmap makes it a cell with no evaluation
- file: docs/design/thalamus.md
  line: 344
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: § 6.5 and D38 drop the before/after rule "one per sample at the highest revision present" that story:m4-before-after states for each side's score
- file: docs/design/thalamus.md
  line: 243
  category: acceptance
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: § 5.1 says the CLI calls the CloudEvents builder, while story:m4-live and D33 say it can call it, from M5
- file: docs/design/thalamus.md
  line: 241
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: § 5.1 store and sources rows and the § 4 storage table omit the deploy-instant store port, the deploy readers and the unlimited retention D39 records
- file: docs/design/thalamus.md
  line: 245
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: the rewritten thalamus-cli line omits the `flaps` subcommand that crates/thalamus-cli/src/main.rs:36 has
```
