---
format: aep.planning-md/3
id: review-result:adversary-m4-design-record-pass-2
kind: review-result
status: active
title: Adversary, story:m4-design-record, pass 2
relations:
- reviews: story:m4-design-record
revision: 1
---
unit: story:m4-design-record, working tree of impl/m4-design-record (base 6b98444, uncommitted docs/design/thalamus.md), after the coordinator's F1–F8 edits
verdict: NEEDS-CHANGE
cases: executed 420→420 (task check), red 5 (adversary-p2/checks.sh: 31 checks, 26 green, 5 red)
origin: introduced 3 / pre-existing 2 / undecided 0
wrote-outside-worktree: 8 paths (part 6)
needs-coordinator: P1 (merge conflict with main's D24 on the § 5.1 store row); P3 (§ 10 q2 stops at M4, but M5 adds routes to the same unauthenticated listener); P4 and P5 (planning store text, which is not this unit's file)

## 1. git --no-pager diff --stat (worktree)

```
 docs/design/thalamus.md | 99 ++++++++++++++++++++++++++++++++++++++++++++-----
 1 file changed, 89 insertions(+), 10 deletions(-)
```

The implementor made that diff. The adversary wrote nothing in the worktree.

## 2. Checks added (scratch, not in the suite)

`adversary-p2/checks.sh`: read-only greps of the design against the stories in `~/beyond10x/thalamus/.engineering/planning/story/` (the coordinator's current text), plus a three-way `git merge-file` of tree, base 6b98444 and `main`. The red lines from its first run, verbatim (the F5 check was red at first because my threshold was wrong; I corrected the check, not the doc):

```
FAIL F3-residue story:m4-heatmap Decisions still say 'format=json on every view' while D35 says 'every view except /views/live'
FAIL D43 'authentication comes with the in-cluster deployment' appears in no story (coordinator decision as briefed: q2 through M4, D43 ties D31 no-auth to it)
FAIL Q2-M5 section 10 q2 default ends at M4; story:m5-skeleton adds routes to the same no-auth listener in M5
FAIL BA-open-point story:m4-before-after open point (no ok reading -> no score) disagrees with design 6.5 / story Work (no scored evaluation -> UNKNOWN); a partial CRIT is scored with no ok reading
FAIL merge with main: 1 conflict hunk(s) (crate table); tree row keeps 'Eventlog (File, SQLite, PostgreSQL)', main D24 says 'no File provider'
EXIT=1
```

## 3. Suite run

`task check ESS=$HOME/.cache/ess/toolchains/0.60.0/ess` in the worktree, run after the checks: `EXIT=0`, 62 `test result` lines, 420 passed, 0 failed. The last line is `website/docs: no raw admonition titles`. No step reads docs/design/thalamus.md (`grep design/thalamus` on the log finds 0 lines). Log: `adversary-p2/task-check.log`.

## 4. Findings (tree: impl/m4-design-record working tree over 6b98444)

| # | finding | what was measured | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| P1 | merge conflict, and a resolution that reintroduces File | :242 "`thalamus-store` … First provider: Eventlog (File, SQLite, PostgreSQL)". `main` (f164345) changed the same row to "Eventlog (SQLite, PostgreSQL)" and added D24: "no File provider: an instance stores samples and evaluations in Eventlog on SQLite … and on PostgreSQL". `git merge-file` of tree/base/main exits 1 with one hunk (the crate table). Taking the tree side contradicts D24 | the coordinator's merge of this unit into main | NEEDS-CHANGE | introduced |
| P2 | D43 claims what no source states | :524 D43 "… and authentication comes with the in-cluster deployment". No story contains it (`grep -ri "authentication comes with"` in the store: 0). The briefed decision is only "§ 10 question 2 defaults to per operator machine through M4, and D43 ties D31's no-authentication rule to it". story:m4-skeleton:75 says only "there is no authentication in M4 (design § 10 question 2: the service runs per operator machine)" | readers of the design who plan the in-cluster milestone | NEEDS-CHANGE | introduced |
| P3 | the F2 fix moves the gap to M5 | :440 "per operator machine through M4; an in-cluster deployment is a later milestone (D43)". story:m5-skeleton:50 "the route table gains `GET /api/assertions` and `GET /api/assertions/{id}`" on the same listener. story:m5-service-answers:41 adds "No write route", but no M5 story states where the service runs. In M5, unauthenticated HTTP has no § 10 default behind it | the M5 assertion routes behind `--allow-remote` | CONFIRMED | introduced |
| P4 | the story D35 cites still says the old rule | D35 at :516 "`format=json` on every view except `/views/live` … (coordinator, wave 20261015e …)". story:m4-heatmap.md:41 still says "`format=json` on every view, so the views are testable …". Each other view story declares its own `format=html\|json`, so no implementor is misled; the planning store contradicts the design | planning-store readers | NEEDS-CHANGE | pre-existing |
| P5 | a coordinator open point contradicts the design line and the story's own Work line | :350 "a side with no scored evaluation is `UNKNOWN` with a reason and has no delta" (= story:m4-before-after.md:27). The same story, :65 (open point, wave 20261015e): "A side with no `ok` reading shows no score and the row has no delta". A side made only of partial reads can be scored: `crates/thalamus-core/tests/grading.rs:252` `partial_lower_bound_over_crit_is_crit` asserts `CRIT` with score >= 2 (`assert_agrees`, :191). The two rules disagree for that side. "shows no score" is unclear: it may mean the median column. Ask the coordinator | the m4-before-after implementor | NEEDS-CHANGE | pre-existing |

## 5. Attacked and not broken

- F1: D30 :511 quotes story:m4-export-dashboard:30 exactly ("stdout by default, exit codes as `export rules`"). The `export rules` parenthetical matches story:m4-export-rules:46.
- F2: § 10 q2 :440 and D31 :512 now cite D43. :255–256 agrees with them, and so does main's D24 ("waits for the in-cluster deployment (§ 10 question 2)").
- F3: :322 holds. heatmap, sparklines, week, `/views/deploys` and `/views/deploys/{id}` each carry `format=html|json` in their stories, and story:m4-live gives `/views/live` none. m4-live-acceptance reads only the heatmap's JSON.
- F4: :331 says "a cell with no evaluation is a gap", and the column wording is gone.
- F5: § 6.5 before/after :348 and D38 :519 both carry "one per sample at the highest revision present".
- F6: :244 "the CLI can call it from M5 (`story:m5-conductor-contract`)". That story builds the transition line with `thalamus_pipeline::cloudevents::envelope`.
- F7: § 4 :223, store :242 and sources :243 are present. "a few per day" comes from story:m4-spec-deploy-instants:59. "behind one trait" matches `crates/thalamus-store/src/lib.rs:1`.
- F8: every `Command` variant in `crates/thalamus-cli/src/main.rs` (sample, eval, show, compare, rollup, flaps) appears in :246.
- D1–D29 are byte-identical to base. D30–D43 each appear once, in order. Every § 11 row has 3 cells. The acceptance greps pass (story:m4- count 14).
- § 11 intro :506 ("unless the row names a coordinator decision") covers D35's amendment and D43.

## 6. Paths written outside the worktree

- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/checks.sh
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/checks.log
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/design.base.md
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/design.main.md
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/design.tree.md
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/design.merged.md (holds the conflict markers)
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-p2/task-check.log
- ~/.cache/thalamus-waves/20261015e/m4-design-record/scratch/adversary-pass-2.md (this report)

`task check` wrote its build output to the worktree's own `target/`.

## 7. Findings block

```findings
- file: docs/design/thalamus.md
  line: 242
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the rewritten store row conflicts with main's row and keeps "Eventlog (File, SQLite, PostgreSQL)", which main's D24 ("no File provider") rules out, so the merge must take "(SQLite, PostgreSQL)"
- file: docs/design/thalamus.md
  line: 524
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: D43 adds "authentication comes with the in-cluster deployment", which no story and not the briefed coordinator decision states
- file: docs/design/thalamus.md
  line: 440
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: § 10 q2 now defaults only through M4, while story:m5-skeleton adds assertion routes to the same unauthenticated listener in M5
- file: docs/design/thalamus.md
  line: 516
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: D35 says format=json on every view except /views/live, while story:m4-heatmap.md:41, which D35 cites, still says "on every view"
- file: docs/design/thalamus.md
  line: 350
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the coordinator's open point at story:m4-before-after.md:65 (no ok reading means no score) contradicts the design and the story's Work line (no scored evaluation means UNKNOWN) for a side of partial-escalated CRIT evaluations, which carry a score
```
