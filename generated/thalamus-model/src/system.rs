// generated from thalamus v1
// model digest 9f981109ead94ade6b2b664c0234683eabf8a4e37dbb57b0fab92ac76ace7832
// contract digest e31e759ce615e6d5358d020e748a76c36fee60b1207d5e97f762de8391357b06
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `thalamus` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
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

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::AssertionAnswered(_) => "thalamus.assertion.AssertionAnswered",
            Self::AssertionAsked(_) => "thalamus.assertion.AssertionAsked",
            Self::DefinitionActivated(_) => "thalamus.definition.DefinitionActivated",
            Self::DefinitionMuted(_) => "thalamus.definition.DefinitionMuted",
            Self::DefinitionProposed(_) => "thalamus.definition.DefinitionProposed",
            Self::DefinitionRetired(_) => "thalamus.definition.DefinitionRetired",
            Self::DefinitionRevised(_) => "thalamus.definition.DefinitionRevised",
            Self::DefinitionUnmuted(_) => "thalamus.definition.DefinitionUnmuted",
            Self::DimensionRegistered(_) => "thalamus.dimension.DimensionRegistered",
            Self::RegistryDeclared(_) => "thalamus.dimension.RegistryDeclared",
            Self::EvaluationRecorded(_) => "thalamus.evaluation.EvaluationRecorded",
            Self::SampleRecorded(_) => "thalamus.observation.SampleRecorded",
            Self::RollupRecorded(_) => "thalamus.rollup.RollupRecorded",
            Self::TransitionRecorded(_) => "thalamus.transition.TransitionRecorded",
        }
    }
}

impl From<crate::ports::thalamus_service::PublishedEvent> for SystemEvent {
    fn from(event: crate::ports::thalamus_service::PublishedEvent) -> Self {
        match event {
            crate::ports::thalamus_service::PublishedEvent::AssertionAnswered(event) => Self::AssertionAnswered(event),
            crate::ports::thalamus_service::PublishedEvent::AssertionAsked(event) => Self::AssertionAsked(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionActivated(event) => Self::DefinitionActivated(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionMuted(event) => Self::DefinitionMuted(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionProposed(event) => Self::DefinitionProposed(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionRetired(event) => Self::DefinitionRetired(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionRevised(event) => Self::DefinitionRevised(event),
            crate::ports::thalamus_service::PublishedEvent::DefinitionUnmuted(event) => Self::DefinitionUnmuted(event),
            crate::ports::thalamus_service::PublishedEvent::DimensionRegistered(event) => Self::DimensionRegistered(event),
            crate::ports::thalamus_service::PublishedEvent::RegistryDeclared(event) => Self::RegistryDeclared(event),
            crate::ports::thalamus_service::PublishedEvent::EvaluationRecorded(event) => Self::EvaluationRecorded(event),
            crate::ports::thalamus_service::PublishedEvent::SampleRecorded(event) => Self::SampleRecorded(event),
            crate::ports::thalamus_service::PublishedEvent::RollupRecorded(event) => Self::RollupRecorded(event),
            crate::ports::thalamus_service::PublishedEvent::TransitionRecorded(event) => Self::TransitionRecorded(event),
        }
    }
}

/// The `thalamus` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<ThalamusServiceBehaviors> {
    /// The `thalamus-service` component.
    pub thalamus_service: crate::ports::thalamus_service::ThalamusService<ThalamusServiceBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<ThalamusServiceBehaviors> System<ThalamusServiceBehaviors> {
    /// Assembles the system from its components.
    pub fn new(thalamus_service: crate::ports::thalamus_service::ThalamusService<ThalamusServiceBehaviors>) -> Self {
        Self {
            thalamus_service,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<ThalamusServiceBehaviors> System<ThalamusServiceBehaviors>
where
    ThalamusServiceBehaviors: crate::assertion::obligations::AnswerAssertionBehavior + crate::assertion::obligations::AskAssertionBehavior + crate::definition::obligations::ActivateDefinitionBehavior + crate::definition::obligations::MuteDefinitionBehavior + crate::definition::obligations::ProposeDefinitionBehavior + crate::definition::obligations::RetireDefinitionBehavior + crate::definition::obligations::ReviseDefinitionBehavior + crate::definition::obligations::UnmuteDefinitionBehavior + crate::dimension::obligations::DeclareRegistryBehavior + crate::dimension::obligations::RegisterDimensionBehavior + crate::evaluation::obligations::RecordEvaluationBehavior + crate::observation::obligations::RecordSampleBehavior + crate::rollup::obligations::RecordRollupBehavior + crate::transition::obligations::RecordTransitionBehavior + crate::assertion::obligations::AssertionsQuery + crate::assertion::obligations::OpenAssertionsQuery + crate::definition::obligations::ActiveDefinitionsQuery + crate::definition::obligations::DefinitionsQuery + crate::dimension::obligations::DimensionsQuery + crate::dimension::obligations::RegistriesQuery + crate::evaluation::obligations::EvaluationsQuery + crate::evaluation::obligations::SeriesVerdictsQuery + crate::evaluation::obligations::VerdictChangesQuery + crate::observation::obligations::SamplesQuery + crate::observation::obligations::SeriesSamplesQuery + crate::rollup::obligations::RollupsQuery + crate::rollup::obligations::SeriesRollupsQuery + crate::transition::obligations::SeriesTransitionsQuery + crate::transition::obligations::TransitionsQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), crate::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.thalamus_service.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
