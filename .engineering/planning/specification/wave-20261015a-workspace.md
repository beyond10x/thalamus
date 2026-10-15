---
format: aep.planning-md/3
id: specification:wave-20261015a-workspace
kind: specification
status: implemented
title: 'Wave 20261015a: workspace skeleton'
relations:
- specifies: story:m0-workspace
revision: 5
transitions:
- {from: "draft", to: "in_review", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 2}
- {from: "in_review", to: "approved", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 3}
- {from: "approved", to: "implemented", at: "2026-10-15T03:07:23Z", actor: "human:timo", revision: 5}
---
## Wave 20261015a: workspace skeleton

Opened 2026-10-15, `aep:implementing` wave mode (skill 0.22.11), toward the operator's goal "M3
done". Standing approval: the operator asked for M0–M3 to be implemented in AEP waves with multiple
agents; each wave records its own page and does not stop at stage 1.

`aep plan artifact waves --kind story --status draft` (2026-10-15) places `story:m0-workspace` alone
in wave 1; every other story depends on it. N = 1.

Integration branch `wave/20261015a`.

## Units

| unit | story | branch | worktree | build dir | scratch |
|---|---|---|---|---|---|
| ws | `story:m0-workspace` | `impl/m0-workspace` | see Progress | `<tree>/target` | `~/.cache/thalamus-waves/20261015a/ws/scratch` |

Agents: `aep:implementor`, then `aep:adversary`.

## Commits approval authorises

One commit per unit through `b10x-gates bot`, the merge into `wave/20261015a`, the closing store
commit, the merge into `main`, and the push of `main` through `b10x-gates bot` (the operator
approved publishing this repository on 2026-10-15). No tag, no release.

## Progress

| unit | commit | stages |
|---|---|---|
| ws | `b863833` | implementor green → adversary pass 1 red, 3 findings (`review-result:adversary-m0-workspace-pass-1`: 1 fixed, 1 fixed by keeping the adversary's test, 1 no-op) → correction green → coordinator-verified → merged fast-forward into `main` |

Deviation: the correction (one self dev-dependency line in `crates/thalamus-core/Cargo.toml`, one
`Cargo.lock` line, two `AGENTS.md` lines) got no second adversary pass; the coordinator read the
diff and re-ran the whole gate instead.

Full gate on `b863833`: `task check` exit 0 — `thalamus v1 — 7 file(s), valid`; drift `generated/ and
spec/suite.json match spec/`; fmt, clippy exit 0; tests 8 passed, 0 failed (cli 6,
adv_stub_exit 1, adv_conformance_feature 1). Pushed to `origin/main` through `b10x-gates bot`.

Cost: implementor 177,896 tokens over two rounds (67 tool uses, about 27 min); adversary 88,471
tokens (49 tool uses, about 30 min).

Cleanup: tree `thalamus-w15a-ws` finished and removed by `worktree gc`; branch `impl/m0-workspace`
deleted (merged); scratch removed.
