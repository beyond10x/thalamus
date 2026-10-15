---
format: aep.planning-md/3
id: specification:wave-20261015a-workspace
kind: specification
status: approved
title: 'Wave 20261015a: workspace skeleton'
relations:
- specifies: story:m0-workspace
revision: 3
transitions:
- {from: "draft", to: "in_review", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 2}
- {from: "in_review", to: "approved", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 3}
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

(updated as units move)
