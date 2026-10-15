<!--
  generated from thalamus v1
  model digest 73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed
  contract digest 093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061
  do not edit: regenerate with `ess synthesize --layout crate`
-->
# Synthesis plan — thalamus v1

Scope: `component-skeletons`, laid out as `crate`, planned by `ess-synth`. Regenerate with `ess synthesize --layout crate`.

180 capabilities: **169 generated**, **11 obligations**, **0 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `thalamus.assertion.Asker` |
| domain type | `thalamus.assertion.AskerKind` |
| domain type | `thalamus.assertion.Assertion.State` |
| domain type | `thalamus.assertion.AssertionId` |
| domain type | `thalamus.assertion.AssertionResult` |
| domain type | `thalamus.assertion.Within` |
| domain type | `thalamus.definition.AdapterId` |
| domain type | `thalamus.definition.Author` |
| domain type | `thalamus.definition.BaselineKind` |
| domain type | `thalamus.definition.ConnectionId` |
| domain type | `thalamus.definition.DenominatorSpec` |
| domain type | `thalamus.definition.EvaluationSpec` |
| domain type | `thalamus.definition.ExtractorId` |
| domain type | `thalamus.definition.Family` |
| domain type | `thalamus.definition.History` |
| domain type | `thalamus.definition.Links` |
| domain type | `thalamus.definition.Owner` |
| domain type | `thalamus.definition.Persistence` |
| domain type | `thalamus.definition.QueryText` |
| domain type | `thalamus.definition.RuleId` |
| domain type | `thalamus.definition.SignalDefinition.State` |
| domain type | `thalamus.definition.SignalId` |
| domain type | `thalamus.definition.SignalKind` |
| domain type | `thalamus.definition.Source` |
| domain type | `thalamus.definition.Threshold` |
| domain type | `thalamus.definition.Unit` |
| domain type | `thalamus.dimension.Dimension.State` |
| domain type | `thalamus.dimension.DimensionKey` |
| domain type | `thalamus.dimension.DimensionRegistry.State` |
| domain type | `thalamus.dimension.DimensionRule` |
| domain type | `thalamus.dimension.DimensionValue` |
| domain type | `thalamus.dimension.Locator` |
| domain type | `thalamus.dimension.RegistryId` |
| domain type | `thalamus.dimension.ValueSource` |
| domain type | `thalamus.evaluation.Baseline` |
| domain type | `thalamus.evaluation.BaselineLabel` |
| domain type | `thalamus.evaluation.EvaluationId` |
| domain type | `thalamus.evaluation.SignalEvaluation.State` |
| domain type | `thalamus.evaluation.Verdict` |
| domain type | `thalamus.observation.Bound` |
| domain type | `thalamus.observation.CompleteReading` |
| domain type | `thalamus.observation.Denominator` |
| domain type | `thalamus.observation.PartialReading` |
| domain type | `thalamus.observation.Provenance` |
| domain type | `thalamus.observation.QueryDigest` |
| domain type | `thalamus.observation.Reading` |
| domain type | `thalamus.observation.RunId` |
| domain type | `thalamus.observation.SampleId` |
| domain type | `thalamus.observation.SeriesKey` |
| domain type | `thalamus.observation.SignalSample.State` |
| domain type | `thalamus.observation.TimeWindow` |
| domain type | `thalamus.observation.UnreadReading` |
| domain type | `thalamus.rollup.DailyRollup.State` |
| domain type | `thalamus.rollup.RollupId` |
| domain type | `thalamus.rollup.VerdictCounts` |
| domain type | `thalamus.transition.Transition.State` |
| domain type | `thalamus.transition.TransitionId` |
| entity lifecycle | `thalamus.assertion.Assertion` |
| entity lifecycle | `thalamus.definition.SignalDefinition` |
| entity lifecycle | `thalamus.dimension.Dimension` |
| entity lifecycle | `thalamus.dimension.DimensionRegistry` |
| entity lifecycle | `thalamus.evaluation.SignalEvaluation` |
| entity lifecycle | `thalamus.observation.SignalSample` |
| entity lifecycle | `thalamus.rollup.DailyRollup` |
| entity lifecycle | `thalamus.transition.Transition` |
| command contract | `thalamus.assertion.AnswerAssertion` |
| command behaviour | `thalamus.assertion.AnswerAssertion` |
| command contract | `thalamus.assertion.AskAssertion` |
| command contract | `thalamus.definition.ActivateDefinition` |
| command behaviour | `thalamus.definition.ActivateDefinition` |
| command contract | `thalamus.definition.MuteDefinition` |
| command behaviour | `thalamus.definition.MuteDefinition` |
| command contract | `thalamus.definition.ProposeDefinition` |
| command contract | `thalamus.definition.RetireDefinition` |
| command behaviour | `thalamus.definition.RetireDefinition` |
| command contract | `thalamus.definition.ReviseDefinition` |
| command contract | `thalamus.definition.UnmuteDefinition` |
| command behaviour | `thalamus.definition.UnmuteDefinition` |
| command contract | `thalamus.dimension.DeclareRegistry` |
| command behaviour | `thalamus.dimension.DeclareRegistry` |
| command contract | `thalamus.dimension.RegisterDimension` |
| command contract | `thalamus.evaluation.RecordEvaluation` |
| command contract | `thalamus.observation.RecordSample` |
| command contract | `thalamus.rollup.RecordRollup` |
| command behaviour | `thalamus.rollup.RecordRollup` |
| command contract | `thalamus.transition.RecordTransition` |
| event type | `thalamus.assertion.AssertionAnswered` |
| event type | `thalamus.assertion.AssertionAsked` |
| event type | `thalamus.definition.DefinitionActivated` |
| event type | `thalamus.definition.DefinitionMuted` |
| event type | `thalamus.definition.DefinitionProposed` |
| event type | `thalamus.definition.DefinitionRetired` |
| event type | `thalamus.definition.DefinitionRevised` |
| event type | `thalamus.definition.DefinitionUnmuted` |
| event type | `thalamus.dimension.DimensionRegistered` |
| event type | `thalamus.dimension.RegistryDeclared` |
| event type | `thalamus.evaluation.EvaluationRecorded` |
| event type | `thalamus.observation.SampleRecorded` |
| event type | `thalamus.rollup.RollupRecorded` |
| event type | `thalamus.transition.TransitionRecorded` |
| error type | `thalamus.assertion.AssertionStateConflict` |
| error type | `thalamus.assertion.NoSignals` |
| error type | `thalamus.assertion.ReasonRequired` |
| error type | `thalamus.assertion.ResultContradictsVerdict` |
| error type | `thalamus.assertion.UnknownAssertion` |
| error type | `thalamus.definition.DefinitionExists` |
| error type | `thalamus.definition.DefinitionStateConflict` |
| error type | `thalamus.definition.FixedNumberMissing` |
| error type | `thalamus.definition.FixedNumberUnused` |
| error type | `thalamus.definition.IncompleteDefinition` |
| error type | `thalamus.definition.InstantNeedsFixedBaseline` |
| error type | `thalamus.definition.MappingMissing` |
| error type | `thalamus.definition.MappingUnused` |
| error type | `thalamus.definition.MuteExpiryPassed` |
| error type | `thalamus.definition.PersistenceTooSmall` |
| error type | `thalamus.definition.RetiredDefinition` |
| error type | `thalamus.definition.RollingWindowMissing` |
| error type | `thalamus.definition.RollingWindowUnused` |
| error type | `thalamus.definition.ThresholdsUnused` |
| error type | `thalamus.definition.UnknownDefinition` |
| error type | `thalamus.dimension.DimensionExists` |
| error type | `thalamus.dimension.RegistryExists` |
| error type | `thalamus.dimension.UnknownParent` |
| error type | `thalamus.dimension.UnknownRegistry` |
| error type | `thalamus.evaluation.ReasonRequired` |
| error type | `thalamus.evaluation.ScoreNotAdmitted` |
| error type | `thalamus.evaluation.UnknownSample` |
| error type | `thalamus.observation.SignalNotActive` |
| error type | `thalamus.observation.UnknownSignal` |
| error type | `thalamus.rollup.RollupExists` |
| error type | `thalamus.rollup.UnknownSignal` |
| error type | `thalamus.transition.UnknownSignal` |
| view type | `thalamus.assertion.Assertions` |
| view query | `thalamus.assertion.Assertions` |
| view type | `thalamus.assertion.OpenAssertions` |
| view query | `thalamus.assertion.OpenAssertions` |
| view type | `thalamus.definition.ActiveDefinitions` |
| view query | `thalamus.definition.ActiveDefinitions` |
| view type | `thalamus.definition.Definitions` |
| view query | `thalamus.definition.Definitions` |
| view type | `thalamus.dimension.Dimensions` |
| view query | `thalamus.dimension.Dimensions` |
| view type | `thalamus.dimension.Registries` |
| view query | `thalamus.dimension.Registries` |
| view type | `thalamus.evaluation.Evaluations` |
| view query | `thalamus.evaluation.Evaluations` |
| view type | `thalamus.evaluation.SeriesVerdicts` |
| view type | `thalamus.evaluation.VerdictChanges` |
| view query | `thalamus.evaluation.VerdictChanges` |
| view type | `thalamus.observation.Samples` |
| view query | `thalamus.observation.Samples` |
| view type | `thalamus.observation.SeriesSamples` |
| view type | `thalamus.rollup.Rollups` |
| view query | `thalamus.rollup.Rollups` |
| view type | `thalamus.rollup.SeriesRollups` |
| view type | `thalamus.transition.SeriesTransitions` |
| view type | `thalamus.transition.Transitions` |
| view query | `thalamus.transition.Transitions` |
| actor grants | `thalamus.assertion.Answerer` |
| actor grants | `thalamus.assertion.Caller` |
| actor grants | `thalamus.definition.InstanceOperator` |
| actor grants | `thalamus.definition.Service` |
| actor grants | `thalamus.dimension.InstanceOperator` |
| actor grants | `thalamus.evaluation.Grader` |
| actor grants | `thalamus.observation.Sampler` |
| actor grants | `thalamus.rollup.Roller` |
| actor grants | `thalamus.transition.Hysteresis` |
| component port | `thalamus-service` |
| component transport | `thalamus-service` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `thalamus.assertion.AskAssertion` | kept an obligation by a typed response (`response:`) | given `thalamus.assertion.AskAssertion` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `no-signals` when `signals.count == 0`, error `thalamus.assertion.NoSignals`; `asked` otherwise, creates `thalamus.assertion.Assertion`, emits `thalamus.assertion.AssertionAsked` |
| command behaviour | `thalamus.definition.ProposeDefinition` | kept an obligation by the guard `exists b in evaluation.baselines: (b != fixed)`, in `instant-not-fixed` | given `thalamus.definition.ProposeDefinition` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `instant-not-fixed` when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.InstantNeedsFixedBaseline`; `rolling-n-missing` when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`, error `thalamus.definition.RollingWindowMissing`; `rolling-n-unused` when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`, error `thalamus.definition.RollingWindowUnused`; `fixed-number-missing` when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`, error `thalamus.definition.FixedNumberMissing`; `fixed-number-unused` when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.FixedNumberUnused`; `persistence-below-one` when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`, error `thalamus.definition.PersistenceTooSmall`; `mapping-missing` when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`, error `thalamus.definition.MappingMissing`; `mapping-unused` when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`, error `thalamus.definition.MappingUnused`; `thresholds-unused` when `(evaluation.rule == producer_mapped and (defined(evaluation.warn) or defined(evaluation.crit)))`, error `thalamus.definition.ThresholdsUnused`; `proposed` otherwise, creates `thalamus.definition.SignalDefinition`, emits `thalamus.definition.DefinitionProposed`; `already-defined` for an identity a record already carries, error `thalamus.definition.DefinitionExists` |
| command behaviour | `thalamus.definition.ReviseDefinition` | kept an obligation by the guard `exists b in evaluation.baselines: (b != fixed)`, in `instant-not-fixed` | given `thalamus.definition.ReviseDefinition` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `retired` when the existing subject's stored fields satisfy `state == Retired`, error `thalamus.definition.RetiredDefinition`; `instant-not-fixed` when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.InstantNeedsFixedBaseline`; `rolling-n-missing` when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`, error `thalamus.definition.RollingWindowMissing`; `rolling-n-unused` when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`, error `thalamus.definition.RollingWindowUnused`; `fixed-number-missing` when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`, error `thalamus.definition.FixedNumberMissing`; `fixed-number-unused` when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.FixedNumberUnused`; `persistence-below-one` when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`, error `thalamus.definition.PersistenceTooSmall`; `mapping-missing` when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`, error `thalamus.definition.MappingMissing`; `mapping-unused` when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`, error `thalamus.definition.MappingUnused`; `thresholds-unused` when `(evaluation.rule == producer_mapped and (defined(evaluation.warn) or defined(evaluation.crit)))`, error `thalamus.definition.ThresholdsUnused`; `revised` otherwise, updates `thalamus.definition.SignalDefinition`, emits `thalamus.definition.DefinitionRevised`; `no-such-definition` for an identity no record carries, error `thalamus.definition.UnknownDefinition` |
| command behaviour | `thalamus.dimension.RegisterDimension` | kept an obligation by `when_related:` reading several related rows in one command (beyond10x/ess#283) | given `thalamus.dimension.RegisterDimension` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` (read the related rows in the declaration order of their `exists: false` branches, and the first missing one answers) before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then choose the first declared present `when_related:` predicate refusal whose predicate and optional input guard hold, across rows; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `no-registry` when no `thalamus.dimension.DimensionRegistry` carries the identity `input.registry_id` names, error `thalamus.dimension.UnknownRegistry`; `no-parent` when no `thalamus.dimension.Dimension` carries the identity `input.parent` names, error `thalamus.dimension.UnknownParent`; `registered` otherwise, creates `thalamus.dimension.Dimension`, emits `thalamus.dimension.DimensionRegistered`; `already-registered` for an identity a record already carries, error `thalamus.dimension.DimensionExists` |
| command behaviour | `thalamus.evaluation.RecordEvaluation` | kept an obligation by a typed response (`response:`) | given `thalamus.evaluation.RecordEvaluation` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-sample` when no `thalamus.observation.SignalSample` carries the identity `input.sample_id` names, error `thalamus.evaluation.UnknownSample`; `reason-missing` when `(verdict != OK and not (defined(reason)))`, error `thalamus.evaluation.ReasonRequired`; `score-not-admitted` when `(verdict in [N_A, UNKNOWN] and defined(score))`, error `thalamus.evaluation.ScoreNotAdmitted`; `recorded` otherwise, creates `thalamus.evaluation.SignalEvaluation`, emits `thalamus.evaluation.EvaluationRecorded` |
| command behaviour | `thalamus.observation.RecordSample` | kept an obligation by a typed response (`response:`) | given `thalamus.observation.RecordSample` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-signal` when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names, error `thalamus.observation.UnknownSignal`; `not-active` when the `thalamus.definition.SignalDefinition` that `input.signal` names satisfies `state != Active`, error `thalamus.observation.SignalNotActive`; `recorded` otherwise, creates `thalamus.observation.SignalSample`, emits `thalamus.observation.SampleRecorded` |
| command behaviour | `thalamus.transition.RecordTransition` | kept an obligation by a typed response (`response:`) | given `thalamus.transition.RecordTransition` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-signal` when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names, error `thalamus.transition.UnknownSignal`; `recorded` otherwise, creates `thalamus.transition.Transition`, emits `thalamus.transition.TransitionRecorded` |
| view query | `thalamus.evaluation.SeriesVerdicts` | kept an obligation by a view parameter (`params:`), which a generated query does not apply | a query answering `thalamus.evaluation.SeriesVerdicts` with rows projected from `thalamus.evaluation.SignalEvaluation` at `read_your_writes` consistency, containing instances where `series == param.series` |
| view query | `thalamus.observation.SeriesSamples` | kept an obligation by a view parameter (`params:`), which a generated query does not apply | a query answering `thalamus.observation.SeriesSamples` with rows projected from `thalamus.observation.SignalSample` at `read_your_writes` consistency, containing instances where `series == param.series` |
| view query | `thalamus.rollup.SeriesRollups` | kept an obligation by a view parameter (`params:`), which a generated query does not apply | a query answering `thalamus.rollup.SeriesRollups` with rows projected from `thalamus.rollup.DailyRollup` at `read_your_writes` consistency, containing instances where `series == param.series` |
| view query | `thalamus.transition.SeriesTransitions` | kept an obligation by a view parameter (`params:`), which a generated query does not apply | a query answering `thalamus.transition.SeriesTransitions` with rows projected from `thalamus.transition.Transition` at `read_your_writes` consistency, containing instances where `series == param.series` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
