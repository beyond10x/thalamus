---
format: aep.planning-md/3
id: story:m1-verdict-grading
kind: story
status: draft
title: A reading graded against a baseline yields one ladder verdict, score and reason; unknown is never OK
relations:
- decomposes: epic:signal-contract
- depends_on: epic:specification
- depends_on: story:m0-workspace
- depends_on: story:m0-conformance-records
scope:
- confidence: inferred
  path: crates/thalamus-core/src/grading.rs
- confidence: inferred
  path: crates/thalamus-core/src/ladder.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/mod.rs
- confidence: inferred
  path: crates/thalamus-core/src/rules/threshold.rs
- confidence: inferred
  path: crates/thalamus-core/tests/grading.rs
revision: 3
---
## Outcome

Any caller of `thalamus-core` gets, for one reading and one baseline, exactly one verdict of the ladder, its score and its reason, and an unknown reading can never come out `OK`.

## Context

`epic:signal-contract`; design § 2.3 (ladder), § 3.1 rule 4 (partial), § 3.3 (score), § 1 L1, L3, L5. `thalamus-core` is pure: no I/O, no clock (design § 5.1). The crate and every module file this story fills exist after `story:m0-workspace` (`depends_on`); the generated `Verdict` enum carries `exit`, `rank` and `scored` (`spec/domains/evaluation.yaml:17`). Baselines are computed by `story:m1-baselines`; this story takes a recorded `thalamus.evaluation.Baseline` as input. This story, `story:m1-baselines` and `story:m1-instance-registry` write disjoint files, so no shared line orders them: the `crates/thalamus-core/src/lib.rs` module lines are `story:m0-workspace`'s, and none of the three touches them.

## Work

- `crates/thalamus-core/src/ladder.rs`: worst-of over verdicts by `rank` (`CRIT > WARN > UNKNOWN > OK = N/A`); the exit code of a pass: the worst verdict `exit`, except a pass in which every verdict is `N/A`, or which is empty, exits 3.
- `crates/thalamus-core/src/grading.rs`: `grade(definition, sample, baseline) -> Graded { verdict, score, reason, baseline }`, in this order:
  1. a sample whose `denominator.n` was read and is below `min_n` is `N/A` with a reason naming `n` and `min_n`, no score;
  2. a reading of quality `unknown` is `UNKNOWN`, reason carried from the reading;
  3. a baseline whose reading is `unknown` is `UNKNOWN`, reason carried from the baseline (rules 1 and 2 of § 3.1 arrive here as an unknown baseline);
  4. a `partial` reading may escalate but never clear: a lower bound at or over the CRIT line is `CRIT`, anything else `UNKNOWN` with reason `partial read`;
  5. otherwise the rule of the definition decides (below).
- `crates/thalamus-core/src/rules/mod.rs`: the closed rule list, an enum `RuleId` parsed from `evaluation.rule`, and the dispatch from each id to its rule. An unknown id is a parse error the instance loader reports. Each later rule adds one variant and one dispatch arm here (`story:m3-vocabulary-mapping`, then `story:m3-advisory-gate-mapping`).
- `crates/thalamus-core/src/rules/threshold.rs`: `ratio_vs_pooled_baseline` (design § 2.1) and one higher-is-worse and one lower-is-worse rule for counts and gauges graded against `higher_of`, `lower_of` or `fixed`; their ids are chosen here. The WARN and CRIT lines of `evaluation.warn` / `evaluation.crit` decide: a line is crossed when the value is at least `x` times the baseline, at least `abs`, and the event count (value times `n` for a ratio) is at least `min_events`, each where declared; the reason states the comparison (design § 2.3 example: `0.029 >= 2 x 0.013 and >= 0.02, 15 failures of 556`).
- Score (§ 3.3): 0 at the baseline, 1 at the WARN line, 2 at the CRIT line, log-linear between them for a multiplier line, linear for an absolute one, negative when better than the baseline; a line is the stricter of its `x` and `abs` parts, so the score and the verdict cannot disagree. `UNKNOWN` and `N/A` carry no score.

## Domain relations

- `thalamus.evaluation.SignalEvaluation` → `thalamus.observation.SignalSample`, one, references via `sample_id`; the sample exists first. Inferable: `spec/domains/evaluation.yaml`, entity `SignalEvaluation`, relation `sample`. The grader reads the sample, never changes it.
- `thalamus.evaluation.SignalEvaluation` → `thalamus.definition.SignalDefinition`, one, references via `signal`, at the revision `definition_rev`. Inferable: `spec/domains/evaluation.yaml`, relation `definition`; the thresholds live only in the definition (design § 1 L2).
- The recorded baseline is a closed kind plus its offsets and a `Reading`. Inferable: `spec/domains/evaluation.yaml:46` (`UNMAPPED:` marker, decided as design § 11 D4).
- The rules an evaluation must satisfy to be recorded: `reason` on every verdict other than `OK`, no score on an unscored verdict. Inferable: `spec/domains/evaluation.yaml`, `SignalEvaluation` invariants and `RecordEvaluation` outcomes `reason-missing`, `score-not-admitted`. The grading rules on the sample quality are held here by test (`spec/domains/evaluation.yaml:171`, `UNMAPPED:`).

## Decisions taken (reversible)

- `N/A` is decided before the quality checks only when the source answered a population (`denominator.n` present); an unknown reading with no denominator is `UNKNOWN`.
- The ids of the two count/gauge rules are chosen in this story.

## Scope

Planned (inferred):

- `crates/thalamus-core/src/ladder.rs`, `crates/thalamus-core/src/grading.rs`
- `crates/thalamus-core/src/rules/mod.rs` (the rule list; its module lines are `story:m0-workspace`'s), `crates/thalamus-core/src/rules/threshold.rs`
- `crates/thalamus-core/tests/grading.rs`

## Acceptance

`cargo test -p thalamus-core --test grading` passes the tests `unknown_reading_is_unknown_never_ok` (written and seen failing first), `unknown_baseline_is_unknown`, `partial_lower_bound_over_crit_is_crit`, `partial_reading_never_clears`, `below_min_n_is_na_without_score`, `score_is_0_1_2_at_baseline_warn_crit`, `score_and_verdict_agree`, `worst_of_orders_crit_warn_unknown_ok`, `all_na_pass_exits_3`, `unknown_rule_id_is_a_parse_error` and `every_graded_result_is_admitted_by_record_evaluation` (no result trips `thalamus.evaluation.RecordEvaluation/outcome/reason-missing` or `thalamus.evaluation.RecordEvaluation/outcome/score-not-admitted`).

## Out of scope

Computing baselines (`story:m1-baselines`); hysteresis and settled transitions (M2, design § 3.2, § 11 D6); producer vocabulary mapping and the advisory-gate rule (M3); objectives and burn rates (design § 3.4, § 11 D7).
