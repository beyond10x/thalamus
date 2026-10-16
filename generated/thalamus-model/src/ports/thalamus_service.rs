// generated from thalamus v1
// model digest 6f2e9f65e0ef065a249a30bb38bae16ef166a5b45577b7a8f20509f1f3eddea7
// contract digest 067f632c99cd362c9c2d5e3d29f9b5d30e6604352424ed4eca28522b738edbee
// do not edit: regenerate with `ess synthesize --layout crate`

//! thalamus-service — the `thalamus-service` component of `thalamus` v1.
//!
//! Samples each active definition on its cadence, grades every sample, answers assertions, and serves current verdicts, series and assertions over HTTP. Read-only against the systems it watches; holds no secret.
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `thalamus.assertion.AssertionAnswered`.
    AssertionAnswered(crate::assertion::AssertionAnswered),
    /// `thalamus.assertion.AssertionAsked`.
    AssertionAsked(crate::assertion::AssertionAsked),
    /// `thalamus.definition.DefinitionActivated`.
    DefinitionActivated(crate::definition::DefinitionActivated),
    /// `thalamus.definition.DefinitionMuted`.
    DefinitionMuted(crate::definition::DefinitionMuted),
    /// `thalamus.definition.DefinitionProposed`.
    DefinitionProposed(crate::definition::DefinitionProposed),
    /// `thalamus.definition.DefinitionRetired`.
    DefinitionRetired(crate::definition::DefinitionRetired),
    /// `thalamus.definition.DefinitionRevised`.
    DefinitionRevised(crate::definition::DefinitionRevised),
    /// `thalamus.definition.DefinitionUnmuted`.
    DefinitionUnmuted(crate::definition::DefinitionUnmuted),
    /// `thalamus.dimension.DimensionRegistered`.
    DimensionRegistered(crate::dimension::DimensionRegistered),
    /// `thalamus.dimension.RegistryDeclared`.
    RegistryDeclared(crate::dimension::RegistryDeclared),
    /// `thalamus.evaluation.EvaluationRecorded`.
    EvaluationRecorded(crate::evaluation::EvaluationRecorded),
    /// `thalamus.observation.SampleRecorded`.
    SampleRecorded(crate::observation::SampleRecorded),
    /// `thalamus.rollup.RollupRecorded`.
    RollupRecorded(crate::rollup::RollupRecorded),
    /// `thalamus.transition.TransitionRecorded`.
    TransitionRecorded(crate::transition::TransitionRecorded),
}

/// thalamus-service — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct ThalamusService<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> ThalamusService<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> ThalamusService<B>
where
    B: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    /// Accepts `thalamus.assertion.AnswerAssertion`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn answer_assertion(&mut self, input: crate::assertion::AnswerAssertion) -> Result<crate::assertion::AnswerAssertionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.answer_assertion(input)?;
        match &outcome {
            crate::assertion::AnswerAssertionOutcome::ReasonMissing { .. } => {}
            crate::assertion::AnswerAssertionOutcome::ContradictsVerdict { .. } => {}
            crate::assertion::AnswerAssertionOutcome::Answered { assertion_answered, .. } => {
                self.outbox.push(PublishedEvent::AssertionAnswered(assertion_answered.clone()));
            }
            crate::assertion::AnswerAssertionOutcome::WrongState { .. } => {}
            crate::assertion::AnswerAssertionOutcome::NoSuchAssertion { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.assertion.AskAssertion`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn ask_assertion(&mut self, input: crate::assertion::AskAssertion) -> Result<crate::assertion::AskAssertionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.ask_assertion(input)?;
        match &outcome {
            crate::assertion::AskAssertionOutcome::NoSignals { .. } => {}
            crate::assertion::AskAssertionOutcome::Asked { assertion_asked, .. } => {
                self.outbox.push(PublishedEvent::AssertionAsked(assertion_asked.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.ActivateDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn activate_definition(&mut self, input: crate::definition::ActivateDefinition) -> Result<crate::definition::ActivateDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.activate_definition(input)?;
        match &outcome {
            crate::definition::ActivateDefinitionOutcome::Incomplete { .. } => {}
            crate::definition::ActivateDefinitionOutcome::Activated { definition_activated, .. } => {
                self.outbox.push(PublishedEvent::DefinitionActivated(definition_activated.clone()));
            }
            crate::definition::ActivateDefinitionOutcome::WrongState { .. } => {}
            crate::definition::ActivateDefinitionOutcome::NoSuchDefinition { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.MuteDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn mute_definition(&mut self, input: crate::definition::MuteDefinition) -> Result<crate::definition::MuteDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.mute_definition(input)?;
        match &outcome {
            crate::definition::MuteDefinitionOutcome::ExpiryPassed { .. } => {}
            crate::definition::MuteDefinitionOutcome::Muted { definition_muted, .. } => {
                self.outbox.push(PublishedEvent::DefinitionMuted(definition_muted.clone()));
            }
            crate::definition::MuteDefinitionOutcome::WrongState { .. } => {}
            crate::definition::MuteDefinitionOutcome::NoSuchDefinition { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.ProposeDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn propose_definition(&mut self, input: crate::definition::ProposeDefinition) -> Result<crate::definition::ProposeDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.propose_definition(input)?;
        match &outcome {
            crate::definition::ProposeDefinitionOutcome::HostNotLocal { .. } => {}
            crate::definition::ProposeDefinitionOutcome::LocalNotHost { .. } => {}
            crate::definition::ProposeDefinitionOutcome::InstantNotFixed { .. } => {}
            crate::definition::ProposeDefinitionOutcome::RollingNMissing { .. } => {}
            crate::definition::ProposeDefinitionOutcome::RollingNUnused { .. } => {}
            crate::definition::ProposeDefinitionOutcome::FixedNumberMissing { .. } => {}
            crate::definition::ProposeDefinitionOutcome::FixedNumberUnused { .. } => {}
            crate::definition::ProposeDefinitionOutcome::PersistenceBelowOne { .. } => {}
            crate::definition::ProposeDefinitionOutcome::MappingMissing { .. } => {}
            crate::definition::ProposeDefinitionOutcome::MappingUnused { .. } => {}
            crate::definition::ProposeDefinitionOutcome::ThresholdsUnused { .. } => {}
            crate::definition::ProposeDefinitionOutcome::AdvisoryNotCount { .. } => {}
            crate::definition::ProposeDefinitionOutcome::AdvisoryNotFixed { .. } => {}
            crate::definition::ProposeDefinitionOutcome::Proposed { definition_proposed, .. } => {
                self.outbox.push(PublishedEvent::DefinitionProposed(definition_proposed.clone()));
            }
            crate::definition::ProposeDefinitionOutcome::AlreadyDefined { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.RetireDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn retire_definition(&mut self, input: crate::definition::RetireDefinition) -> Result<crate::definition::RetireDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.retire_definition(input)?;
        match &outcome {
            crate::definition::RetireDefinitionOutcome::Retired { definition_retired, .. } => {
                self.outbox.push(PublishedEvent::DefinitionRetired(definition_retired.clone()));
            }
            crate::definition::RetireDefinitionOutcome::WrongState { .. } => {}
            crate::definition::RetireDefinitionOutcome::NoSuchDefinition { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.ReviseDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn revise_definition(&mut self, input: crate::definition::ReviseDefinition) -> Result<crate::definition::ReviseDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.revise_definition(input)?;
        match &outcome {
            crate::definition::ReviseDefinitionOutcome::Retired { .. } => {}
            crate::definition::ReviseDefinitionOutcome::HostNotLocal { .. } => {}
            crate::definition::ReviseDefinitionOutcome::LocalNotHost { .. } => {}
            crate::definition::ReviseDefinitionOutcome::InstantNotFixed { .. } => {}
            crate::definition::ReviseDefinitionOutcome::RollingNMissing { .. } => {}
            crate::definition::ReviseDefinitionOutcome::RollingNUnused { .. } => {}
            crate::definition::ReviseDefinitionOutcome::FixedNumberMissing { .. } => {}
            crate::definition::ReviseDefinitionOutcome::FixedNumberUnused { .. } => {}
            crate::definition::ReviseDefinitionOutcome::PersistenceBelowOne { .. } => {}
            crate::definition::ReviseDefinitionOutcome::MappingMissing { .. } => {}
            crate::definition::ReviseDefinitionOutcome::MappingUnused { .. } => {}
            crate::definition::ReviseDefinitionOutcome::ThresholdsUnused { .. } => {}
            crate::definition::ReviseDefinitionOutcome::AdvisoryNotCount { .. } => {}
            crate::definition::ReviseDefinitionOutcome::AdvisoryNotFixed { .. } => {}
            crate::definition::ReviseDefinitionOutcome::Revised { definition_revised, .. } => {
                self.outbox.push(PublishedEvent::DefinitionRevised(definition_revised.clone()));
            }
            crate::definition::ReviseDefinitionOutcome::NoSuchDefinition { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.definition.UnmuteDefinition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn unmute_definition(&mut self, input: crate::definition::UnmuteDefinition) -> Result<crate::definition::UnmuteDefinitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.unmute_definition(input)?;
        match &outcome {
            crate::definition::UnmuteDefinitionOutcome::Unmuted { definition_unmuted, .. } => {
                self.outbox.push(PublishedEvent::DefinitionUnmuted(definition_unmuted.clone()));
            }
            crate::definition::UnmuteDefinitionOutcome::WrongState { .. } => {}
            crate::definition::UnmuteDefinitionOutcome::NoSuchDefinition { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.dimension.DeclareRegistry`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn declare_registry(&mut self, input: crate::dimension::DeclareRegistry) -> Result<crate::dimension::DeclareRegistryOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.declare_registry(input)?;
        match &outcome {
            crate::dimension::DeclareRegistryOutcome::Declared { registry_declared, .. } => {
                self.outbox.push(PublishedEvent::RegistryDeclared(registry_declared.clone()));
            }
            crate::dimension::DeclareRegistryOutcome::AlreadyDeclared { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.dimension.RegisterDimension`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn register_dimension(&mut self, input: crate::dimension::RegisterDimension) -> Result<crate::dimension::RegisterDimensionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.register_dimension(input)?;
        match &outcome {
            crate::dimension::RegisterDimensionOutcome::NoRegistry { .. } => {}
            crate::dimension::RegisterDimensionOutcome::NoParent { .. } => {}
            crate::dimension::RegisterDimensionOutcome::Registered { dimension_registered, .. } => {
                self.outbox.push(PublishedEvent::DimensionRegistered(dimension_registered.clone()));
            }
            crate::dimension::RegisterDimensionOutcome::AlreadyRegistered { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.evaluation.RecordEvaluation`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_evaluation(&mut self, input: crate::evaluation::RecordEvaluation) -> Result<crate::evaluation::RecordEvaluationOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_evaluation(input)?;
        match &outcome {
            crate::evaluation::RecordEvaluationOutcome::UnknownSample { .. } => {}
            crate::evaluation::RecordEvaluationOutcome::ReasonMissing { .. } => {}
            crate::evaluation::RecordEvaluationOutcome::ScoreNotAdmitted { .. } => {}
            crate::evaluation::RecordEvaluationOutcome::Recorded { evaluation_recorded, .. } => {
                self.outbox.push(PublishedEvent::EvaluationRecorded(evaluation_recorded.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.observation.RecordSample`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_sample(&mut self, input: crate::observation::RecordSample) -> Result<crate::observation::RecordSampleOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_sample(input)?;
        match &outcome {
            crate::observation::RecordSampleOutcome::UnknownSignal { .. } => {}
            crate::observation::RecordSampleOutcome::NotActive { .. } => {}
            crate::observation::RecordSampleOutcome::Recorded { sample_recorded, .. } => {
                self.outbox.push(PublishedEvent::SampleRecorded(sample_recorded.clone()));
            }
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.rollup.RecordRollup`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_rollup(&mut self, input: crate::rollup::RecordRollup) -> Result<crate::rollup::RecordRollupOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_rollup(input)?;
        match &outcome {
            crate::rollup::RecordRollupOutcome::UnknownSignal { .. } => {}
            crate::rollup::RecordRollupOutcome::Recorded { rollup_recorded, .. } => {
                self.outbox.push(PublishedEvent::RollupRecorded(rollup_recorded.clone()));
            }
            crate::rollup::RecordRollupOutcome::AlreadyRolledUp { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `thalamus.transition.RecordTransition`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn record_transition(&mut self, input: crate::transition::RecordTransition) -> Result<crate::transition::RecordTransitionOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.record_transition(input)?;
        match &outcome {
            crate::transition::RecordTransitionOutcome::UnknownSignal { .. } => {}
            crate::transition::RecordTransitionOutcome::Recorded { transition_recorded, .. } => {
                self.outbox.push(PublishedEvent::TransitionRecorded(transition_recorded.clone()));
            }
        }
        Ok(outcome)
    }

    /// Serves `thalamus.assertion.Assertions` at `read_your_writes` consistency, from the owed projection.
    pub fn assertions(&self) -> Result<Vec<crate::assertion::Assertions>, crate::obligation::UnmetObligation> {
        self.behaviors.assertions()
    }

    /// Serves `thalamus.assertion.OpenAssertions` at `read_your_writes` consistency, from the owed projection.
    pub fn open_assertions(&self) -> Result<Vec<crate::assertion::OpenAssertions>, crate::obligation::UnmetObligation> {
        self.behaviors.open_assertions()
    }

    /// Serves `thalamus.definition.ActiveDefinitions` at `read_your_writes` consistency, from the owed projection.
    pub fn active_definitions(&self) -> Result<Vec<crate::definition::ActiveDefinitions>, crate::obligation::UnmetObligation> {
        self.behaviors.active_definitions()
    }

    /// Serves `thalamus.definition.Definitions` at `read_your_writes` consistency, from the owed projection.
    pub fn definitions(&self) -> Result<Vec<crate::definition::Definitions>, crate::obligation::UnmetObligation> {
        self.behaviors.definitions()
    }

    /// Serves `thalamus.dimension.Dimensions` at `read_your_writes` consistency, from the owed projection.
    pub fn dimensions(&self) -> Result<Vec<crate::dimension::Dimensions>, crate::obligation::UnmetObligation> {
        self.behaviors.dimensions()
    }

    /// Serves `thalamus.dimension.Registries` at `read_your_writes` consistency, from the owed projection.
    pub fn registries(&self) -> Result<Vec<crate::dimension::Registries>, crate::obligation::UnmetObligation> {
        self.behaviors.registries()
    }

    /// Serves `thalamus.evaluation.Evaluations` at `read_your_writes` consistency, from the owed projection.
    pub fn evaluations(&self) -> Result<Vec<crate::evaluation::Evaluations>, crate::obligation::UnmetObligation> {
        self.behaviors.evaluations()
    }

    /// Serves `thalamus.evaluation.SeriesVerdicts` at `read_your_writes` consistency, from the owed projection.
    pub fn series_verdicts(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::evaluation::SeriesVerdicts>, crate::obligation::UnmetObligation> {
        self.behaviors.series_verdicts(series)
    }

    /// Serves `thalamus.evaluation.VerdictChanges` at `read_your_writes` consistency, from the owed projection.
    pub fn verdict_changes(&self) -> Result<Vec<crate::evaluation::VerdictChanges>, crate::obligation::UnmetObligation> {
        self.behaviors.verdict_changes()
    }

    /// Serves `thalamus.observation.Samples` at `read_your_writes` consistency, from the owed projection.
    pub fn samples(&self) -> Result<Vec<crate::observation::Samples>, crate::obligation::UnmetObligation> {
        self.behaviors.samples()
    }

    /// Serves `thalamus.observation.SeriesSamples` at `read_your_writes` consistency, from the owed projection.
    pub fn series_samples(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::observation::SeriesSamples>, crate::obligation::UnmetObligation> {
        self.behaviors.series_samples(series)
    }

    /// Serves `thalamus.rollup.Rollups` at `read_your_writes` consistency, from the owed projection.
    pub fn rollups(&self) -> Result<Vec<crate::rollup::Rollups>, crate::obligation::UnmetObligation> {
        self.behaviors.rollups()
    }

    /// Serves `thalamus.rollup.SeriesRollups` at `read_your_writes` consistency, from the owed projection.
    pub fn series_rollups(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::rollup::SeriesRollups>, crate::obligation::UnmetObligation> {
        self.behaviors.series_rollups(series)
    }

    /// Serves `thalamus.transition.SeriesTransitions` at `read_your_writes` consistency, from the owed projection.
    pub fn series_transitions(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::transition::SeriesTransitions>, crate::obligation::UnmetObligation> {
        self.behaviors.series_transitions(series)
    }

    /// Serves `thalamus.transition.Transitions` at `read_your_writes` consistency, from the owed projection.
    pub fn transitions(&self) -> Result<Vec<crate::transition::Transitions>, crate::obligation::UnmetObligation> {
        self.behaviors.transitions()
    }
}
