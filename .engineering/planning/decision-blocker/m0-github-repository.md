---
format: aep.planning-md/3
id: decision-blocker:m0-github-repository
kind: decision-blocker
status: cleared
title: Nobody has decided that beyond10x/thalamus is created on GitHub and enrolled in common Gates
relations:
- blocks: story:m0-ci
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-15T02:07:53Z", actor: "human:timo", revision: 3}
---
## Question

Is the GitHub repository `beyond10x/thalamus` to be created (public, as `AGENTS.md` says the repository is), and is it enrolled in common Gates (shared source gates workflow, policy exceptions, the `B10X_GATES_POLICY` secret)?

## Why it blocks

`story:m0-ci` is accepted only by a green run of `.github/workflows/check.yml` on GitHub. On 2026-10-15 `gh repo view beyond10x/thalamus` answered "Could not resolve to a Repository", and the local checkout has no remote and no commit. Creating a repository is an operator decision, and Gates enrollment is explicit per repository; neither is guessable from this store.

## What does not wait

Every other M0 story lands and is accepted locally through `task check`. The workflow file of `story:m0-ci` can be written beforehand; only its acceptance waits.

## Default if unanswered

None taken. `story:m0-ci` stays unaccepted, and the shared source gates workflow is not added.

## Answer

Answered by the operator on 2026-10-15: create `beyond10x/thalamus` on GitHub (public), with the first
commit and push through the bot, after the planning store is decomposed and the specification
validates. Common Gates enrollment follows the organization's independent-repository route.

This blocker is cleared once the repository exists and the first bot-authored commit is on `main`.
