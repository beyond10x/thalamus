---
format: aep.planning-md/3
id: story:m3-loki-empty-numerator
kind: story
status: draft
title: An empty Loki numerator answer infers no zero
relations:
- decomposes: epic:source-adapters
- depends_on: story:m3-loki-adapter
scope:
- confidence: inferred
  path: crates/thalamus-sources/src/loki.rs
- confidence: inferred
  path: crates/thalamus-sources/tests/loki.rs
revision: 3
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
