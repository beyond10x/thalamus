---
format: aep.planning-md/3
id: review-result:adversary-m3-git-adapter-pass-1
kind: review-result
status: active
title: Adversary, story:m3-git-adapter, pass 1
relations:
- reviews: story:m3-git-adapter
revision: 1
---
unit: story:m3-git-adapter, adversary pass 1, working tree of impl/m3-git-adapter (base 77020c9, uncommitted) at ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-git-adapter
verdict: red. NEEDS-CHANGE: a ratio whose `match` matches no record reads 0, against design § 11 D20
cases: executed 206→221, red 1
origin: introduced 8, pre-existing 0, undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: yes. Choose the D20 fix: change the code, or add a D20 row for git

## 1. `git --no-pager diff --stat` and untracked files

```
 crates/thalamus-docs/src/status.rs |   8 +-
 crates/thalamus-sources/src/git.rs | 867 ++++++++++++++++++++++++++++++++++++-
 website/data/status.json           |   4 +-
 3 files changed, 870 insertions(+), 9 deletions(-)
?? crates/thalamus-sources/tests/adv_git.rs            <- mine, the only file I wrote
?? crates/thalamus-sources/tests/fixtures/git/...      <- implementor's
?? crates/thalamus-sources/tests/git.rs                <- implementor's
```

| Path | Who changed it |
|---|---|
| `git.rs` | the implementor; I did not touch it (mutants ran on a scratch copy) |
| `status.rs`, `status.json` | `scratch/status-git-shipped.patch`, applied as the coordinator asked; it touches no test file |
| `adv_git.rs` | me. It is the only test file I wrote, formatted with `rustfmt --edition 2024` on that file alone |

State I left the tree in: the coordinator patch is applied. `task docs-generate ESS=~/.cache/ess/toolchains/0.60.0/ess` ran with exit 0 and changed nothing beyond the patch. `adv_git.rs` is untracked. Nothing is committed. I released my lease (`adv1-m3-git-20261015c`).

## 2. Cases added (`crates/thalamus-sources/tests/adv_git.rs`, 15 cases)

The cases write their own page recordings into the double's directory, so I added no fixture files.

| Case | What it asserts | Now |
|---|---|---|
| `adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20` | pipelines are 4 success and 1 canceled, `match {"status":"failed"}`: the read is unknown, not 0. The same holds per `ref` | **red** |
| `adv_git_full_page_then_empty_page_is_complete` | 100 records, then `[]`: `ok 100`, pages [1,2] | green |
| `adv_git_refusal_on_page_three_is_a_lower_bound` | 2 full pages, then a refusal: `partial 200 lower`, pages [1,2,3] | green |
| `adv_git_page_budget_hit_exactly` | 51 full pages recorded: `partial 5000 lower`, pages 1..=50 asked. 49 full pages and a 99: `ok 4999` | green |
| `adv_git_cut_walk_that_counted_nothing_is_unknown` | a full page with no match, then a refusal: unknown naming page 2 | green |
| `adv_git_record_repeated_across_pages_is_unknown` | id 100 appears on pages 1 and 2: unknown "twice", with and without dims | green |
| `adv_git_window_edges_with_offsets_and_fractions` | start at +02:00 is inside; start minus 1 ns is outside; end at +02:00 with `.000` is outside; end minus 1 ms is inside; end at -05:00 is outside; 2 far-outside records are outside. Result `ok 2` | green |
| `adv_git_dimension_values_of_each_json_type` | a number gives `"7"`, a bool gives `"false"`, null gives no key; a list gives unknown naming `labels` | green |
| `adv_git_dimensioned_count_with_no_match_is_one_unknown` | exactly 1 unknown read, not 0 reads | green |
| `adv_git_ratio_over_an_empty_window_is_one_unknown` | `[]`: exactly 1 unknown, with and without dims | green |
| `adv_git_ratio_with_the_match_field_absent_on_some_records_is_unknown` | 1 of 4 records lacks `status`: unknown naming `status` | green |
| `adv_git_tags_are_walked_whole_and_placed_by_commit_date` | page 1 is 100 tags, oldest first; page 2 is short. `ok 3`; inputs are only `id, page, per_page`. A null `commit` gives unknown | green |
| `adv_git_commits_are_bounded_by_since_and_until` | `since`=start, `until`=end, SHA ids: `ok 2` | green |
| `adv_git_deployments_default_to_updated_at` | `updated_after`/`updated_before` sent: ratio `0.5` | green |
| `adv_git_query_digest_is_canonical_and_sensitive_to_filter_and_match` | the digest equals SHA-256 of a literal canonical JSON, under 2 insertion orders; it differs when `filter` or `match` differ | green |

Red output, captured when the case was written: `cargo test --locked -p thalamus-sources --test adv_git -- adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20 --exact`

```
test adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20 ... FAILED
thread 'adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20' (2273386) panicked at crates/thalamus-sources/tests/adv_git.rs:277:9:
expected unknown, got Ok(CompleteReading { value: Decimal("0") }) at {}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.01s
```

The line was :277 before rustfmt and :281 after.

### Mutants (scratch copy `scratch/adversary/mut`, `git.rs` never edited)

| Mutant | `--lib --test git` | Killed by an `adv_git` case other than D20 |
|---|---|---|
| M1 the cut with 0 counted returns partial 0 (`:637` guard off) | **green** | `cut_walk_that_counted_nothing` |
| M2 a number dim is refused (`:603`) | **green** | `dimension_values_of_each_json_type` |
| M3 `PAGE_LIMIT` 50→60 (`:73`) | **green** | `page_budget_hit_exactly` |
| M4 tag identity `name`→`id` | **green** | `tags_are_walked_whole…` |
| M5 commits `since`/`until` swapped | **green** | `commits_are_bounded…` |
| M6 a ratio over an empty window returns no read (`:691` guard off) | **green** | `ratio_over_an_empty_window…` |
| M7 a dims count with no match returns no read (`:661` guard off) | **green** | `dimensioned_count_with_no_match…` |
| M12 deployments default to `finished_at` | **green** | `deployments_default_to_updated_at` |
| M8 start exclusive, M9 end inclusive, M10 `<=` short page, M11 page ≤3 fails whole, M13 duplicate check off, M14 ratio cut ignored | red | M8–M13: yes. M14: no, only the implementor's cut-ratio test kills it |

Source: `scratch/adversary/mutants.txt` and `m-*-{impl,adv}.log`.

## 3. Suite run (after the cases existed)

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`: `ess specify validate`, drift, fmt and clippy passed. Test stage:

```
test adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20 ... FAILED
thread 'adv_git_ratio_with_no_matching_record_anywhere_is_unknown_d20' (2344217) panicked at crates/thalamus-sources/tests/adv_git.rs:281:9:
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.21s
error: test failed, to rerun pass `-p thalamus-sources --test adv_git`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
EXIT=201
```

`cargo test` stops at the first failing binary. So I ran `cargo test --locked --workspace --no-fail-fast`: exit 101, 220 passed and 1 failed. All of git.rs (6) and the lib unit tests passed.

- `executed 206→221`: 206 is the same run without `adv_git`'s 15 cases.
- `task docs-check`: exit 0, "1 generated files current".

## 4. Findings

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| F1 | `crates/thalamus-sources/src/git.rs:705` | NEEDS-CHANGE / introduced | `adv_git.rs:281`, exit 101. A ratio with no matching record in a non-empty window reads `ok 0` in every series. D20: "a complete numerator answer with no series at all over a non-empty denominator reads unknown (… a broken matcher looks the same as zero everywhere)". SQL (`sql.rs:505`) and Prometheus (`prometheus.rs:296`) apply this rule. The field-presence check catches a misspelt field, not a misspelt or mistyped value (`"faild"`, `7` vs `"7"`) | every `git.ratio` definition, which `match` is required for |
| F2 | `git.rs:637` | CONFIRMED / introduced | mutant M1 survives the unit's suite | a cut walk with 0 counted, which the unit's tests never build |
| F3 | `git.rs:661` | CONFIRMED / introduced | mutant M7 survives: a dropped series goes undetected | any dims count with no match |
| F4 | `git.rs:691` | CONFIRMED / introduced | mutant M6 survives: an empty-window ratio could return no read | any quiet window |
| F5 | `git.rs:603` | CONFIRMED / introduced | mutant M2 survives: number and bool dims are untested | declared dims such as `project_id` or `draft` |
| F6 | `git.rs:73` | CONFIRMED / introduced | mutant M3 survives: no test reaches the 50-page budget | projects with more than 5000 records a window |
| F7 | `git.rs:168` | CONFIRMED / introduced | M4, M5 and M12 survive: `tags`, `commits` and `deployments` have no test in `tests/git.rs` | definitions with `records` set to those kinds |
| F8 | `git.rs:208` | INFEASIBLE / introduced | not measured, a hypothesis: GitLab's DeploymentsFinder may refuse `updated_after`/`updated_before` without `order_by=updated_at`. If so, every default deployments read is a page-1 refusal and reads unknown, never a wrong value. The adapter does not send `order_by`, and the module doc does not say a filter must | needs one live GitLab call to settle; I don't know |

Fix for F1, not applied: in `ratio()` after `:697`, if no placed record matches, return one unknown with no population ("no record in the window matched `match`; no series is a ratio of 0"). The other option is a D20 row that exempts git because it reads one population whole; that is the coordinator's call. F2–F7 are covered by `adv_git.rs` if the coordinator keeps it.

## 5. Attacked and could not break

- Exactly 100 records then an empty page: complete.
- A refusal on page 2 or 3: partial lower bound, or unknown when 0 were counted.
- Exactly 50 full pages: cut with no page 51; a short 50th page is complete.
- Window edges with offsets, fractions down to 1 ns, and records GitLab returned outside the window.
- A duplicate across pages reads unknown. A page shift that loses a record without repeating one cannot be detected from the answers. It is not testable, and the count stays a valid lower bound.
- Dims that are null, number, bool, list or undeclared; only declared top-level keys are set.
- A ratio with 0 records reads unknown; a ratio where some records lack the match field reads unknown.
- Tags read in ascending order: no early stop, no window parameters.
- The query digest does not change with key order and does change with `filter` or `match`. JSON inside `filter` is hashed as written, as in SQL.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m3-git-adapter/scratch/adversary/`: logs, `mutate.sh`, `mutants.txt`, `git.rs.orig`, `red-d20.txt`, `check.log`, `suite-nofailfast.{log,summary}`, `adv-rest.txt`.
- `~/.cache/thalamus-waves/20261015c/m3-git-adapter/scratch/adversary/mut/`: a mutant copy of the tree with its own `target/` (168M). I left it in place because your rule forbids deleting a `target/` by hand.
- `~/.cache/thalamus-waves/20261015c/m3-git-adapter/scratch/adversary-pass-1.md`: this report.

```findings
[
  {"file": "crates/thalamus-sources/src/git.rs", "line": 705, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a git.ratio whose match matches no record over a non-empty window reads ok 0 in every series, where design § 11 D20 requires unknown"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 637, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the guard reading a cut walk that counted nothing as unknown survives mutation under the unit's own suite"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 661, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the guard emitting one unknown for a dimensioned count with no match survives mutation to an empty read list"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 691, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the guard emitting one unknown for a ratio over an empty window survives mutation to an empty read list"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 603, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "number and bool dimension values have no test in the unit's suite"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 73, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "PAGE_LIMIT can change without any test of the unit failing"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 168, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the tags, commits and deployments rows of KINDS have no test in the unit's suite"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 208, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "unverified: GitLab may refuse deployments updated_after/updated_before without order_by=updated_at, making every default deployments read unknown"}
]
```
