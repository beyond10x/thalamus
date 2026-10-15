---
format: aep.planning-md/3
id: story:m3-loki-empty-numerator
kind: story
status: implemented
title: An empty Loki numerator answer infers no zero
relations:
- decomposes: epic:source-adapters
- depends_on: story:m3-loki-adapter
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/loki.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/loki.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T05:42:58Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T05:42:58Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-15T06:13:36Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The Loki ratio applies design § 11 D20 as extended on 2026-10-15: a complete numerator answer with no
series at all, over a non-empty denominator, reads unknown for every series instead of 0. Found by
the Prometheus adapter's second adversary pass (`review-result:adversary-m1-prometheus-adapter-pass-2`,
G1); `story:m3-loki-adapter` landed before the extension.

## Acceptance

`cargo test -p thalamus-sources --test loki` passes the failing-first test
`loki_empty_numerator_answer_infers_no_zero`.

## Scope

- `crates/thalamus-sources/src/loki.rs` — inferred
- `crates/thalamus-sources/tests/loki.rs` — inferred
