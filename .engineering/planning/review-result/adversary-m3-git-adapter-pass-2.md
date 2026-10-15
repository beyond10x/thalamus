---
format: aep.planning-md/3
id: review-result:adversary-m3-git-adapter-pass-2
kind: review-result
status: active
title: Adversary, story:m3-git-adapter, pass 2
relations:
- reviews: story:m3-git-adapter
revision: 1
---
unit: story:m3-git-adapter, adversary pass 2, uncommitted working tree of impl/m3-git-adapter (base 77020c9) at ~/.local/state/worktree/trees/b10x/thalamus/thalamus-w15c-m3-git-adapter, with the F1 correction (`NO_MATCH`) and the status patch applied
verdict: red. CONFIRMED (warning): `NO_MATCH` fires once for the pass with empty dims, where SQL, Prometheus and Loki on main read one unknown per denominator series
cases: executed 221→228, red 1
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory and 1 report (part 6)
needs-coordinator: yes. Choose: make the git guard per series, like its peers (fix below; every existing case stays green under it), or keep it once per pass and mark my red case

## 1. `git --no-pager diff --stat` and untracked files

```
 crates/thalamus-docs/src/status.rs |   8 +-
 crates/thalamus-sources/src/git.rs | 880 ++++++++++++++++++++++++++++++++++++-
 website/data/status.json           |   4 +-
 3 files changed, 883 insertions(+), 9 deletions(-)
?? crates/thalamus-sources/tests/adv_git.rs       <- pass 1, unchanged
?? crates/thalamus-sources/tests/adv_git_p2.rs    <- mine, the only file I wrote
?? crates/thalamus-sources/tests/fixtures/git/    <- implementor's
?? crates/thalamus-sources/tests/git.rs           <- implementor's
```

- The tracked diff is the implementor's work plus the coordinator's status patch, which is still applied.
- `git.rs` in the worktree is byte-identical to the copy I took before mutating (`cmp`).
- I ran mutants only on a scratch copy.
- `adv_git_p2.rs` was formatted with `rustfmt --edition 2024` on that file alone. `b10x-gates scan-text` on it returned "1 file(s) carry no private rule", exit 0.
- I acquired and released my lease, `adv2-m3-git-20261015c`.

## 2. Cases added (`crates/thalamus-sources/tests/adv_git_p2.rs`, 7 cases)

| Case | What it asserts | Now |
|---|---|---|
| `adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers` | `ref` main/main/release/feature, no `failed`: one unknown per `ref`, carrying that `ref`, reason `NO_MATCH`, no population; exactly 3 reads | **red** |
| `adv2_git_ratio_no_match_has_the_no_match_reason_and_no_population` | no dims: 1 read, reason exactly `git.ratio: {NO_MATCH}`, `denominator None`, empty dims | green |
| `adv2_git_ratio_matches_only_outside_the_window_is_no_match` | `failed` records only before start or at end, `success` inside: `NO_MATCH` | green |
| `adv2_git_ratio_cut_walk_with_no_match_names_the_cut` | 100 `success`, then a refusal on page 2. With and without dims: 1 unknown naming `page 2`, not `NO_MATCH`, no population | green |
| `adv2_git_ratio_empty_window_keeps_population_zero` | only an out-of-window record. With and without dims: reason `EMPTY_POPULATION`, population `n 0` | green |
| `adv2_git_ratio_one_match_in_one_series_leaves_the_others_zero_over_their_population` | exactly 1 `failed` in 7 records. By `ref`: main `0.5` n2, release `0` n2, feature `0` n3. Without dims: `0.142857142857142857` n7 | green |
| `adv2_git_ratio_match_only_in_the_series_without_dims` | the only match lacks `ref`: series `{}` reads `1`, `ref=main` and `ref=release` read `0` | green |

Red output, captured when the case was written: `cargo test --locked -p thalamus-sources --test adv_git_p2 -- adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers --exact`

```
test adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers ... FAILED
thread 'adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers' (2824592) panicked at crates/thalamus-sources/tests/adv_git_p2.rs:207:5:
assertion `left == right` failed: series {"ref": "main"}: [
    Read {
        dims: {},
        reading: Unknown(
            UnreadReading {
                reason: "git.ratio: no record in the window matched `match`; a matcher that matches nothing is not read as 0",
  [...]
  left: 0
 right: 1
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s
```

### Mutants of the new guard (`git.rs:706-710`; scratch copy only, now cleaned and deleted)

| Mutant | `--lib --test git` (implementor) | `adv_git` (pass 1) | `adv_git_p2` kills it with |
|---|---|---|---|
| N1 guard deleted | **green** | red (D20 case) | no_match_reason, outside_window, by_ref |
| N2 guard carries `population(placed.len())` | **green** | **green** | no_match_reason, by_ref |
| N3 guard moved before the cut check | **green** | **green** | cut_walk_with_no_match, empty_window |
| N4 guard moved before the empty-window check | **green** | **green** | empty_window |
| N5 guard per series: a series with 0 matches reads unknown | red | green | one_match_in_one_series, match_only_in_series_without_dims |
| N6 `count(matches) <= 1` read as no match | **green** | red (deployments) | one_match_in_one_series, match_only_in_… |
| N7 `!all(matches)` | red | red | three cases |
| N8 the proposed fix (below) | green | green | all 7 green |

Source: `scratch/adversary-p2/mutants.txt` and `m-N*-{impl,adv1,adv2}.log`. The unmutated copy (N0) reproduced exactly the one red case.

## 3. Suite run (after the cases existed)

`task check ESS=~/.cache/ess/toolchains/0.60.0/ess`, log `scratch/adversary-p2/check.log`:

| Step | Result |
|---|---|
| validate | `ess specify validate`: "thalamus v1 — 7 file(s), valid" |
| drift | "generated/ and spec/suite.json match spec/" |
| fmt, clippy | passed |
| test | red, quoted below |

```
test adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers ... FAILED
thread 'adv2_git_ratio_no_match_by_ref_reads_each_series_unknown_like_its_peers' (2831629) panicked at crates/thalamus-sources/tests/adv_git_p2.rs:207:5:
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
error: test failed, to rerun pass `-p thalamus-sources --test adv_git_p2`
task: Failed to run task "check": task: Failed to run task "test": exit status 101
EXIT=201
```

- `cargo test --locked --workspace --no-fail-fast` ran 228 cases, 227 passed and 1 failed, exit 101. All of `adv_git` (15), `git` and the lib unit tests passed.
- 221 is the same suite without `adv_git_p2`: `--workspace --exclude thalamus-sources` plus `-p thalamus-sources` with every test target except `adv_git_p2`. It passed 221 and failed 0, exit 0. This matches pass 1's after-count.

## 4. Findings

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| G1 | `crates/thalamus-sources/src/git.rs:709` | CONFIRMED / introduced, warning | `adv_git_p2.rs:207`, exit 101. With dims, a window of records none of which match gives 1 read with dims `{}`; the series `ref=main/release/feature` get no read. On main, every peer reads this D20 case once per denominator series, with dims and no population: `sql.rs:505-511` (`denominator.keys().map(..)`), `prometheus.rs:296-305` (`empty` goes into `unread`, then one read per joined series), `loki.rs:197` (per label set; `story:m3-loki-empty-numerator`: "reads unknown for every series instead of 0"). D20's own text says only "reads unknown". Without dims, git matches its peers (1 read, `{}`, no population) | every `git.ratio` with declared `dimensions` whose `match` matches no in-window record. No consumer of `Read.dims` exists in the tree yet (no pipeline code reads it), so the per-series effect downstream (design § 4 daily rollups, `thalamus.md:221`: "one row per series per UTC day: … unknown") is not observed: I don't know |
| G2 | `git.rs:708` | CONFIRMED / introduced, note | N1 (guard deleted) survives the implementor's `--lib --test git`. Only `adv_git`'s D20 case holds the F1 correction. `tests/git.rs` has no no-match ratio test | the guard's whole purpose; the coordinator keeps `adv_git.rs` |
| G3 | `git.rs:709` | CONFIRMED / introduced, note | N2, N3, N4 survive both the implementor's suite and `adv_git`. N6 survives the implementor's suite. `adv_git_p2` (green cases) now kills all four | a population on a no-match read; a cut or empty window misreported as a broken matcher |

Fix for G1, not applied; N8 shows it keeps every existing case and all 7 of mine green. Change `git.rs:709` to one unknown per distinct `dims` of the placed records:

```rust
let keys: BTreeSet<&BTreeMap<String, String>> = placed.iter().map(|p| &p.dims).collect();
return keys.into_iter().map(|d| reads.unknown(d.clone(), NO_MATCH.to_string(), None)).collect();
```

If the fix is taken, the module doc at `git.rs:38-40` ("reads one `unknown`") needs to say "one `unknown` per series".

## 5. Attacked and could not break

- **The legitimate 0 (dims × match):** with ≥1 match anywhere, a series with population and no match reads `0` over its own `n`. This holds with exactly 1 match in the window and when the only match is in the undimensioned series (`{}`).
- **Ratio over a cut walk with no match:** the cut wins over `NO_MATCH`, with and without dims, no population. The guard sits after the cut check (`git.rs:692` before `:708`).
- **Empty window:** keeps `EMPTY_POPULATION` and `n 0`, with and without dims. The guard sits after `:699`.
- **Matches GitLab answered outside `[start, end)`:** they do not count. The guard reads placed records only.
- **The guard's own reading:** reason exactly `git.ratio: NO_MATCH`, `denominator None`, dims `{}` when no dims are declared.
- **A per-series guard (N5)** would break D20's legitimate 0, and the implementor's by-`ref` test catches it.

## 6. Paths written outside the worktree

- `~/.cache/thalamus-waves/20261015c/m3-git-adapter/scratch/adversary-p2/` holds:
  - `red-per-series.txt`, `check.log`, `suite-nofailfast.log`, `suite-other.log`, `suite-without-p2.log`
  - `mutants.txt` and 27 `m-N*-{impl,adv1,adv2}.log`
- Deleted after use: the mutant copy `adversary-p2/mut/` (after `cargo clean` removed its `target/`: 626 files, 209.5 MiB), `mutate.sh` and `git.rs.orig`.
- `~/.cache/thalamus-waves/20261015c/m3-git-adapter/scratch/adversary-pass-2.md`: this report.
- The worktree's own `target/` gained the `adv_git_p2` build and `target/tmp/adv-git-p2/` test directories, as pass 1's did.
- Not mine: pass 1's `scratch/adversary/mut/` with its `target/` (168M) is still there.

```findings
[
  {"file": "crates/thalamus-sources/src/git.rs", "line": 709, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "with declared dimensions, a git.ratio whose match matches no in-window record reads one unknown with empty dims, where the SQL, Prometheus and Loki adapters on main read one unknown per denominator series carrying its dims"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 708, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "deleting the NO_MATCH guard leaves the implementor's own suite green; only adv_git.rs holds the F1 correction"},
  {"file": "crates/thalamus-sources/src/git.rs", "line": 709, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the guard carrying a population, or moved ahead of the cut or empty-window checks, survived the implementor's suite and adv_git.rs; adv_git_p2.rs now kills each"}
]
```
