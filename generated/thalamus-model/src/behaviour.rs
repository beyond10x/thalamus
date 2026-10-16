// generated from thalamus v1
// model digest 99b2e1b31aebb3cdde7af7bb46be2ec80e4af2ccd183cd598985f3a30326c48f
// contract digest 82328f7c71509b62eb250d7912f8913dadad64f2cf9aae2897c422dfba7f784c
// do not edit: regenerate with `ess synthesize --layout crate`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! preserves the trait; generated network entries supply an ephemeral store. `Context` carries the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `thalamus.assertion.Assertion` is stored — a port the implementor provides.
///
/// Keyed by the identity `assertion_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait AssertionStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::assertion::AssertionId) -> Option<crate::assertion::AssertionSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::assertion::AssertionSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::assertion::AssertionId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::assertion::AssertionSnapshot>;
}

/// Where `thalamus.definition.SignalDefinition` is stored — a port the implementor provides.
///
/// Keyed by the identity `signal_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait SignalDefinitionStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::definition::SignalId) -> Option<crate::definition::SignalDefinitionSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::definition::SignalDefinitionSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::definition::SignalId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::definition::SignalDefinitionSnapshot>;
}

/// Where `thalamus.dimension.Dimension` is stored — a port the implementor provides.
///
/// Keyed by the identity `key`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait DimensionStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::dimension::DimensionKey) -> Option<crate::dimension::DimensionSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::dimension::DimensionSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::dimension::DimensionKey);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::dimension::DimensionSnapshot>;
}

/// Where `thalamus.dimension.DimensionRegistry` is stored — a port the implementor provides.
///
/// Keyed by the identity `registry_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait DimensionRegistryStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::dimension::RegistryId) -> Option<crate::dimension::DimensionRegistrySnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::dimension::DimensionRegistrySnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::dimension::RegistryId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::dimension::DimensionRegistrySnapshot>;
}

/// Where `thalamus.evaluation.SignalEvaluation` is stored — a port the implementor provides.
///
/// Keyed by the identity `evaluation_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait SignalEvaluationStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::evaluation::EvaluationId) -> Option<crate::evaluation::SignalEvaluationSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::evaluation::SignalEvaluationSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::evaluation::EvaluationId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::evaluation::SignalEvaluationSnapshot>;
}

/// Where `thalamus.observation.SignalSample` is stored — a port the implementor provides.
///
/// Keyed by the identity `sample_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait SignalSampleStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::observation::SampleId) -> Option<crate::observation::SignalSampleSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::observation::SignalSampleSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::observation::SampleId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::observation::SignalSampleSnapshot>;
}

/// Where `thalamus.rollup.DailyRollup` is stored — a port the implementor provides.
///
/// Keyed by the identity `rollup_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait DailyRollupStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::rollup::RollupId) -> Option<crate::rollup::DailyRollupSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::rollup::DailyRollupSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::rollup::RollupId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::rollup::DailyRollupSnapshot>;
}

/// Where `thalamus.transition.Transition` is stored — a port the implementor provides.
///
/// Keyed by the identity `transition_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait TransitionStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::transition::TransitionId) -> Option<crate::transition::TransitionSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::transition::TransitionSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::transition::TransitionId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::transition::TransitionSnapshot>;
}

/// What the specification leaves to the implementor's context — a port the implementor provides.
///
/// The caller's attributes, the values the model says the implementation assigns, and the answer
/// to each `external:` branch.
pub trait Context {
    /// The instant of the command decision being made now, or `None` where this context has no
    /// clock. Read once per decision, before any guard: every guard ordering an instant against
    /// the current time reads that one instant, and a decision that needs it and has none is
    /// refused naming the command clock, never decided with another instant.
    fn command_clock(&mut self) -> Option<crate::primitives::Timestamp>;
}

/// Context answers that may be unavailable, without fabricated values.
/// Existing `Context` implementations receive the blanket adapter.
pub trait TryContext {
/// The decision's one instant, `None` where the context has no clock, or a named unavailable answer.
fn try_command_clock(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation>;
}

impl<T: Context + ?Sized> TryContext for T {
fn try_command_clock(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation> { Ok(Context::command_clock(self)) }
}

/// An unavailable runtime context answer, rather than a new planned capability.
pub fn unmet_context(source: &'static str) -> UnmetObligation { UnmetObligation { capability: "context answer", source } }

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

/// `thalamus.assertion.AnswerAssertion`, generated: every outcome is one the specification fully determines.
impl<P> crate::assertion::obligations::AnswerAssertionBehavior for Generated<P>
where
    P: AssertionStorage,
{
    fn answer_assertion(&mut self, input: crate::assertion::AnswerAssertion) -> Result<crate::assertion::AnswerAssertionOutcome, UnmetObligation> {
        let _ = &input;
        // `reason-missing`: an input-guarded refusal, before the addressed subject is loaded.
        if decided(all(&[equal(Some(&input.result).map(|value| match value { crate::assertion::AssertionResult::Within => "within", crate::assertion::AssertionResult::Breach => "breach", crate::assertion::AssertionResult::Unknown => "unknown" }.to_owned()), Some("within".to_owned())).map(|value| !value), (Some(Some(&input.reason).and_then(|value| value.as_ref()).is_some())).map(|value| !value)]), "thalamus.assertion.AnswerAssertion")? {
            return Ok(crate::assertion::AnswerAssertionOutcome::ReasonMissing { error: crate::assertion::ReasonRequired { result: input.result.clone() } });
        }
        // `contradicts-verdict`: an input-guarded refusal, before the addressed subject is loaded.
        if decided(any(&[all(&[equal(Some(&input.result).map(|value| match value { crate::assertion::AssertionResult::Within => "within", crate::assertion::AssertionResult::Breach => "breach", crate::assertion::AssertionResult::Unknown => "unknown" }.to_owned()), Some("breach".to_owned())), any(&[equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("OK".to_owned())), equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("N_A".to_owned())), equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("UNKNOWN".to_owned()))])]), all(&[equal(Some(&input.result).map(|value| match value { crate::assertion::AssertionResult::Within => "within", crate::assertion::AssertionResult::Breach => "breach", crate::assertion::AssertionResult::Unknown => "unknown" }.to_owned()), Some("breach".to_owned())).map(|value| !value), any(&[equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("WARN".to_owned())), equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("CRIT".to_owned()))])]), all(&[equal(Some(&input.result).map(|value| match value { crate::assertion::AssertionResult::Within => "within", crate::assertion::AssertionResult::Breach => "breach", crate::assertion::AssertionResult::Unknown => "unknown" }.to_owned()), Some("within".to_owned())), equal(Some(&input.worst).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some("UNKNOWN".to_owned()))])]), "thalamus.assertion.AnswerAssertion")? {
            return Ok(crate::assertion::AnswerAssertionOutcome::ContradictsVerdict { error: crate::assertion::ResultContradictsVerdict { result: input.result.clone(), worst: input.worst.clone() } });
        }
        // `answered`: the default.
        let Some(held) = AssertionStorage::get(&self.ports, &input.assertion_id) else {
            return Ok(crate::assertion::AnswerAssertionOutcome::NoSuchAssertion { error: crate::assertion::UnknownAssertion { assertion_id: input.assertion_id.clone() } });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::assertion::AnyAssertion::Asked(instance) => crate::assertion::AnyAssertion::Answered(instance.answer()),
            _ => return Ok(crate::assertion::AnswerAssertionOutcome::WrongState { error: crate::assertion::AssertionStateConflict { current: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.result = Some(input.result.clone());
        next.data.worst = Some(input.worst.clone());
        next.data.reason = input.reason.clone();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::assertion::AnswerAssertionOutcome::Answered { assertion_answered: crate::assertion::AssertionAnswered { assertion_id: input.assertion_id.clone(), asker: before.asker.clone(), result: input.result.clone(), worst: input.worst.clone() } };
        AssertionStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::assertion::obligations::AskAssertionBehavior> crate::assertion::obligations::AskAssertionBehavior for Generated<P> {
    fn ask_assertion(&mut self, input: crate::assertion::AskAssertion) -> Result<crate::assertion::AskAssertionOutcome, UnmetObligation> {
        crate::assertion::obligations::AskAssertionBehavior::ask_assertion(&mut self.ports, input)
    }
}

/// `thalamus.definition.ActivateDefinition`, generated: every outcome is one the specification fully determines.
impl<P> crate::definition::obligations::ActivateDefinitionBehavior for Generated<P>
where
    P: SignalDefinitionStorage,
{
    fn activate_definition(&mut self, input: crate::definition::ActivateDefinition) -> Result<crate::definition::ActivateDefinitionOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row, read before the branches that select by it.
        let Some(held) = SignalDefinitionStorage::get(&self.ports, &input.signal_id) else {
            return Ok(crate::definition::ActivateDefinitionOutcome::NoSuchDefinition { error: crate::definition::UnknownDefinition { signal_id: input.signal_id.clone() } });
        };
        let _ = &held;
        // `incomplete`: selected by the addressed row.
        if decided(any(&[(Some(Some(&held.data.unit).and_then(|value| value.as_ref()).map(|value| &value.0).is_some())).map(|value| !value), (Some(Some(&held.data.dimensions).and_then(|value| value.as_ref()).is_some())).map(|value| !value), (Some(Some(&held.data.denominator).and_then(|value| value.as_ref()).is_some())).map(|value| !value), (Some(Some(&held.data.owner).and_then(|value| value.as_ref()).map(|value| &value.0).is_some())).map(|value| !value)]), "thalamus.definition.ActivateDefinition")? {
            return Ok(crate::definition::ActivateDefinitionOutcome::Incomplete { error: crate::definition::IncompleteDefinition { signal_id: input.signal_id.clone() } });
        }
        // `activated`: the default.
        let Some(held) = SignalDefinitionStorage::get(&self.ports, &input.signal_id) else {
            return Ok(crate::definition::ActivateDefinitionOutcome::NoSuchDefinition { error: crate::definition::UnknownDefinition { signal_id: input.signal_id.clone() } });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::definition::AnySignalDefinition::Proposed(instance) => crate::definition::AnySignalDefinition::Active(instance.activate()),
            _ => return Ok(crate::definition::ActivateDefinitionOutcome::WrongState { error: crate::definition::DefinitionStateConflict { current: held_state } }),
        };
        let next = moved.snapshot();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::definition::ActivateDefinitionOutcome::Activated { definition_activated: crate::definition::DefinitionActivated { signal_id: input.signal_id.clone() } };
        SignalDefinitionStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `thalamus.definition.MuteDefinition`, generated: every outcome is one the specification fully determines.
impl<P> crate::definition::obligations::MuteDefinitionBehavior for Generated<P>
where
    P: TryContext + SignalDefinitionStorage,
{
    fn mute_definition(&mut self, input: crate::definition::MuteDefinition) -> Result<crate::definition::MuteDefinitionOutcome, UnmetObligation> {
        let _ = &input;
        // The decision's one instant: read once, before any guard, and read by every guard that
        // orders an instant against the current time.
        let now = self.ports.try_command_clock();
        // `expiry-passed`: an input-guarded refusal, before the addressed subject is loaded.
        if decided_at(compare_with_now(Some(&input.until).map(|value| value.0.clone()), &now, 0, core::cmp::Ordering::is_le), &now, "thalamus.definition.MuteDefinition")? {
            return Ok(crate::definition::MuteDefinitionOutcome::ExpiryPassed { error: crate::definition::MuteExpiryPassed { until: input.until.clone() } });
        }
        // `muted`: the default.
        let Some(held) = SignalDefinitionStorage::get(&self.ports, &input.signal_id) else {
            return Ok(crate::definition::MuteDefinitionOutcome::NoSuchDefinition { error: crate::definition::UnknownDefinition { signal_id: input.signal_id.clone() } });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::definition::AnySignalDefinition::Active(instance) => crate::definition::AnySignalDefinition::Muted(instance.mute()),
            _ => return Ok(crate::definition::MuteDefinitionOutcome::WrongState { error: crate::definition::DefinitionStateConflict { current: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.mute_reason = Some(input.reason.clone());
        next.data.mute_until = Some(input.until.clone());
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::definition::MuteDefinitionOutcome::Muted { definition_muted: crate::definition::DefinitionMuted { signal_id: input.signal_id.clone(), reason: input.reason.clone(), until: input.until.clone() } };
        SignalDefinitionStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::definition::obligations::ProposeDefinitionBehavior> crate::definition::obligations::ProposeDefinitionBehavior for Generated<P> {
    fn propose_definition(&mut self, input: crate::definition::ProposeDefinition) -> Result<crate::definition::ProposeDefinitionOutcome, UnmetObligation> {
        crate::definition::obligations::ProposeDefinitionBehavior::propose_definition(&mut self.ports, input)
    }
}

/// `thalamus.definition.RetireDefinition`, generated: every outcome is one the specification fully determines.
impl<P> crate::definition::obligations::RetireDefinitionBehavior for Generated<P>
where
    P: SignalDefinitionStorage,
{
    fn retire_definition(&mut self, input: crate::definition::RetireDefinition) -> Result<crate::definition::RetireDefinitionOutcome, UnmetObligation> {
        let _ = &input;
        // `retired`: the default.
        let Some(held) = SignalDefinitionStorage::get(&self.ports, &input.signal_id) else {
            return Ok(crate::definition::RetireDefinitionOutcome::NoSuchDefinition { error: crate::definition::UnknownDefinition { signal_id: input.signal_id.clone() } });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::definition::AnySignalDefinition::Active(instance) => crate::definition::AnySignalDefinition::Retired(instance.retire()),
            crate::definition::AnySignalDefinition::Muted(instance) => crate::definition::AnySignalDefinition::Retired(instance.retire()),
            crate::definition::AnySignalDefinition::Proposed(instance) => crate::definition::AnySignalDefinition::Retired(instance.retire()),
            _ => return Ok(crate::definition::RetireDefinitionOutcome::WrongState { error: crate::definition::DefinitionStateConflict { current: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.mute_reason = None;
        next.data.mute_until = None;
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::definition::RetireDefinitionOutcome::Retired { definition_retired: crate::definition::DefinitionRetired { signal_id: input.signal_id.clone() } };
        SignalDefinitionStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::definition::obligations::ReviseDefinitionBehavior> crate::definition::obligations::ReviseDefinitionBehavior for Generated<P> {
    fn revise_definition(&mut self, input: crate::definition::ReviseDefinition) -> Result<crate::definition::ReviseDefinitionOutcome, UnmetObligation> {
        crate::definition::obligations::ReviseDefinitionBehavior::revise_definition(&mut self.ports, input)
    }
}

/// `thalamus.definition.UnmuteDefinition`, generated: every outcome is one the specification fully determines.
impl<P> crate::definition::obligations::UnmuteDefinitionBehavior for Generated<P>
where
    P: SignalDefinitionStorage,
{
    fn unmute_definition(&mut self, input: crate::definition::UnmuteDefinition) -> Result<crate::definition::UnmuteDefinitionOutcome, UnmetObligation> {
        let _ = &input;
        // `unmuted`: the default.
        let Some(held) = SignalDefinitionStorage::get(&self.ports, &input.signal_id) else {
            return Ok(crate::definition::UnmuteDefinitionOutcome::NoSuchDefinition { error: crate::definition::UnknownDefinition { signal_id: input.signal_id.clone() } });
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::definition::AnySignalDefinition::Muted(instance) => crate::definition::AnySignalDefinition::Active(instance.unmute()),
            _ => return Ok(crate::definition::UnmuteDefinitionOutcome::WrongState { error: crate::definition::DefinitionStateConflict { current: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.mute_reason = None;
        next.data.mute_until = None;
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::definition::UnmuteDefinitionOutcome::Unmuted { definition_unmuted: crate::definition::DefinitionUnmuted { signal_id: input.signal_id.clone() } };
        SignalDefinitionStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `thalamus.dimension.DeclareRegistry`, generated: every outcome is one the specification fully determines.
impl<P> crate::dimension::obligations::DeclareRegistryBehavior for Generated<P>
where
    P: DimensionRegistryStorage,
{
    fn declare_registry(&mut self, input: crate::dimension::DeclareRegistry) -> Result<crate::dimension::DeclareRegistryOutcome, UnmetObligation> {
        let _ = &input;
        // `already-declared`: an identity a record already carries, before any branch is taken.
        if DimensionRegistryStorage::get(&self.ports, &input.registry_id).is_some() {
            return Ok(crate::dimension::DeclareRegistryOutcome::AlreadyDeclared { error: crate::dimension::RegistryExists { registry_id: input.registry_id.clone() } });
        }
        // `declared`: the default.
        let identity: crate::dimension::RegistryId = input.registry_id.clone();
        let data = crate::dimension::DimensionRegistryData {
            registry_id: identity.clone(),
            title: input.title.clone(),
        };
        let answer = crate::dimension::DeclareRegistryOutcome::Declared { registry_declared: crate::dimension::RegistryDeclared { registry_id: identity.clone() } };
        DimensionRegistryStorage::put(&mut self.ports, crate::dimension::AnyDimensionRegistry::Declared(crate::dimension::DimensionRegistry::new(data)).snapshot());
        return Ok(answer);
    }
}

impl<P: crate::dimension::obligations::RegisterDimensionBehavior> crate::dimension::obligations::RegisterDimensionBehavior for Generated<P> {
    fn register_dimension(&mut self, input: crate::dimension::RegisterDimension) -> Result<crate::dimension::RegisterDimensionOutcome, UnmetObligation> {
        crate::dimension::obligations::RegisterDimensionBehavior::register_dimension(&mut self.ports, input)
    }
}

impl<P: crate::evaluation::obligations::RecordEvaluationBehavior> crate::evaluation::obligations::RecordEvaluationBehavior for Generated<P> {
    fn record_evaluation(&mut self, input: crate::evaluation::RecordEvaluation) -> Result<crate::evaluation::RecordEvaluationOutcome, UnmetObligation> {
        crate::evaluation::obligations::RecordEvaluationBehavior::record_evaluation(&mut self.ports, input)
    }
}

impl<P: crate::observation::obligations::RecordSampleBehavior> crate::observation::obligations::RecordSampleBehavior for Generated<P> {
    fn record_sample(&mut self, input: crate::observation::RecordSample) -> Result<crate::observation::RecordSampleOutcome, UnmetObligation> {
        crate::observation::obligations::RecordSampleBehavior::record_sample(&mut self.ports, input)
    }
}

/// `thalamus.rollup.RecordRollup`, generated: every outcome is one the specification fully determines.
impl<P> crate::rollup::obligations::RecordRollupBehavior for Generated<P>
where
    P: SignalDefinitionStorage + DailyRollupStorage,
{
    fn record_rollup(&mut self, input: crate::rollup::RecordRollup) -> Result<crate::rollup::RecordRollupOutcome, UnmetObligation> {
        let _ = &input;
        // `already-rolled-up`: an identity a record already carries, before any branch is taken.
        if DailyRollupStorage::get(&self.ports, &input.rollup_id).is_some() {
            return Ok(crate::rollup::RecordRollupOutcome::AlreadyRolledUp { error: crate::rollup::RollupExists { rollup_id: input.rollup_id.clone() } });
        }
        // `when_related:` reads the `thalamus.definition.SignalDefinition` row `input.signal` names, through its storage port; an absent
        // reference reads no row and selects no related branch.
        let reference = Some(&input.signal);
        let related = reference.and_then(|identity| SignalDefinitionStorage::get(&self.ports, identity));
        // `unknown-signal`: the reference names an identity no row carries.
        if reference.is_some() && related.is_none() {
            return Ok(crate::rollup::RecordRollupOutcome::UnknownSignal { error: crate::rollup::UnknownSignal { signal: input.signal.clone() } });
        }
        let _ = &related;
        // `recorded`: the default.
        let identity: crate::rollup::RollupId = input.rollup_id.clone();
        let data = crate::rollup::DailyRollupData {
            rollup_id: identity.clone(),
            signal: input.signal.clone(),
            series: input.series.clone(),
            day: input.day.clone(),
            n: input.n.clone(),
            unknown: input.unknown.clone(),
            partial: input.partial.clone(),
            min: input.min.clone(),
            p50: input.p50.clone(),
            max: input.max.clone(),
            last: input.last.clone(),
            worst_verdict: input.worst_verdict.clone(),
            verdict_counts: input.verdict_counts.clone(),
            min_score: input.min_score.clone(),
            max_score: input.max_score.clone(),
        };
        if let Some(broken) = data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::rollup::RecordRollupOutcome::Recorded { rollup_recorded: crate::rollup::RollupRecorded { rollup_id: identity.clone(), signal: input.signal.clone(), series: input.series.clone(), day: input.day.clone() } };
        DailyRollupStorage::put(&mut self.ports, crate::rollup::AnyDailyRollup::Recorded(crate::rollup::DailyRollup::new(data)).snapshot());
        return Ok(answer);
    }
}

impl<P: crate::transition::obligations::RecordTransitionBehavior> crate::transition::obligations::RecordTransitionBehavior for Generated<P> {
    fn record_transition(&mut self, input: crate::transition::RecordTransition) -> Result<crate::transition::RecordTransitionOutcome, UnmetObligation> {
        crate::transition::obligations::RecordTransitionBehavior::record_transition(&mut self.ports, input)
    }
}

/// `thalamus.assertion.Assertions`, generated: every row is one the specification fully determines from the stored `thalamus.assertion.Assertion`s.
impl<P> crate::assertion::obligations::AssertionsQuery for Generated<P>
where
    P: AssertionStorage,
{
    fn assertions(&self) -> Result<Vec<crate::assertion::Assertions>, UnmetObligation> {
        let admitted = AssertionStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::assertion::Assertions {
                assertion_id: held.data.assertion_id,
                signals: held.data.signals,
                dims: held.data.dims,
                after: held.data.after,
                horizon: held.data.horizon,
                within: held.data.within,
                asker: held.data.asker,
                result: held.data.result,
                worst: held.data.worst,
                reason: held.data.reason,
                state: held.state,
            })
            .collect())
    }
}

/// `thalamus.assertion.OpenAssertions`, generated: every row is one the specification fully determines from the stored `thalamus.assertion.Assertion`s.
impl<P> crate::assertion::obligations::OpenAssertionsQuery for Generated<P>
where
    P: AssertionStorage,
{
    fn open_assertions(&self) -> Result<Vec<crate::assertion::OpenAssertions>, UnmetObligation> {
        let mut admitted = AssertionStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| equal(Some(&held.state).map(|value| match value { crate::assertion::AssertionState::Answered => "Answered", crate::assertion::AssertionState::Asked => "Asked" }.to_owned()), Some("Asked".to_owned())) == Some(true));
        Ok(admitted
            .into_iter()
            .map(|held| crate::assertion::OpenAssertions {
                assertion_id: held.data.assertion_id,
                signals: held.data.signals,
                after: held.data.after,
                horizon: held.data.horizon,
                asker: held.data.asker,
            })
            .collect())
    }
}

/// `thalamus.definition.ActiveDefinitions`, generated: every row is one the specification fully determines from the stored `thalamus.definition.SignalDefinition`s.
impl<P> crate::definition::obligations::ActiveDefinitionsQuery for Generated<P>
where
    P: SignalDefinitionStorage,
{
    fn active_definitions(&self) -> Result<Vec<crate::definition::ActiveDefinitions>, UnmetObligation> {
        let mut admitted = SignalDefinitionStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| equal(Some(&held.state).map(|value| match value { crate::definition::SignalDefinitionState::Active => "Active", crate::definition::SignalDefinitionState::Muted => "Muted", crate::definition::SignalDefinitionState::Proposed => "Proposed", crate::definition::SignalDefinitionState::Retired => "Retired" }.to_owned()), Some("Active".to_owned())) == Some(true));
        Ok(admitted
            .into_iter()
            .map(|held| crate::definition::ActiveDefinitions {
                signal_id: held.data.signal_id,
                title: held.data.title,
                family: held.data.family,
                kind: held.data.kind,
                unit: held.data.unit,
                dimensions: held.data.dimensions,
                source: held.data.source,
                cadence: held.data.cadence,
                owner: held.data.owner,
                rev: held.data.rev,
            })
            .collect())
    }
}

/// `thalamus.definition.Definitions`, generated: every row is one the specification fully determines from the stored `thalamus.definition.SignalDefinition`s.
impl<P> crate::definition::obligations::DefinitionsQuery for Generated<P>
where
    P: SignalDefinitionStorage,
{
    fn definitions(&self) -> Result<Vec<crate::definition::Definitions>, UnmetObligation> {
        let admitted = SignalDefinitionStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::definition::Definitions {
                signal_id: held.data.signal_id,
                title: held.data.title,
                family: held.data.family,
                kind: held.data.kind,
                unit: held.data.unit,
                dimensions: held.data.dimensions,
                source: held.data.source,
                extractor: held.data.extractor,
                query: held.data.query,
                window: held.data.window,
                cadence: held.data.cadence,
                history: held.data.history,
                denominator: held.data.denominator,
                objective: held.data.objective,
                owner: held.data.owner,
                links: held.data.links,
                rev: held.data.rev,
                revised_at: held.data.revised_at,
                revised_by: held.data.revised_by,
                mute_reason: held.data.mute_reason,
                mute_until: held.data.mute_until,
                state: held.state,
            })
            .collect())
    }
}

/// `thalamus.dimension.Dimensions`, generated: every row is one the specification fully determines from the stored `thalamus.dimension.Dimension`s.
impl<P> crate::dimension::obligations::DimensionsQuery for Generated<P>
where
    P: DimensionStorage,
{
    fn dimensions(&self) -> Result<Vec<crate::dimension::Dimensions>, UnmetObligation> {
        let admitted = DimensionStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::dimension::Dimensions {
                key: held.data.key,
                registry_id: held.data.registry_id,
                values: held.data.values,
                rule: held.data.rule,
                parent: held.data.parent,
                otel_attribute: held.data.otel_attribute,
                state: held.state,
            })
            .collect())
    }
}

/// `thalamus.dimension.Registries`, generated: every row is one the specification fully determines from the stored `thalamus.dimension.DimensionRegistry`s.
impl<P> crate::dimension::obligations::RegistriesQuery for Generated<P>
where
    P: DimensionRegistryStorage,
{
    fn registries(&self) -> Result<Vec<crate::dimension::Registries>, UnmetObligation> {
        let admitted = DimensionRegistryStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::dimension::Registries {
                registry_id: held.data.registry_id,
                title: held.data.title,
                state: held.state,
            })
            .collect())
    }
}

/// `thalamus.evaluation.Evaluations`, generated: every row is one the specification fully determines from the stored `thalamus.evaluation.SignalEvaluation`s.
impl<P> crate::evaluation::obligations::EvaluationsQuery for Generated<P>
where
    P: SignalEvaluationStorage,
{
    fn evaluations(&self) -> Result<Vec<crate::evaluation::Evaluations>, UnmetObligation> {
        let admitted = SignalEvaluationStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::evaluation::Evaluations {
                evaluation_id: held.data.evaluation_id,
                sample_id: held.data.sample_id,
                signal: held.data.signal,
                series: held.data.series,
                dims: held.data.dims,
                ts: held.data.ts,
                definition_rev: held.data.definition_rev,
                verdict: held.data.verdict,
                prev: held.data.prev,
                score: held.data.score,
                baseline: held.data.baseline,
                reason: held.data.reason,
                state: held.state,
            })
            .collect())
    }
}

impl<P: crate::evaluation::obligations::SeriesVerdictsQuery> crate::evaluation::obligations::SeriesVerdictsQuery for Generated<P> {
    fn series_verdicts(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::evaluation::SeriesVerdicts>, UnmetObligation> {
        crate::evaluation::obligations::SeriesVerdictsQuery::series_verdicts(&self.ports, series)
    }
}

/// `thalamus.evaluation.VerdictChanges`, generated: every row is one the specification fully determines from the stored `thalamus.evaluation.SignalEvaluation`s.
impl<P> crate::evaluation::obligations::VerdictChangesQuery for Generated<P>
where
    P: SignalEvaluationStorage,
{
    fn verdict_changes(&self) -> Result<Vec<crate::evaluation::VerdictChanges>, UnmetObligation> {
        let mut admitted = SignalEvaluationStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| all(&[Some(Some(&held.data.prev).and_then(|value| value.as_ref()).is_some()), equal(Some(&held.data.verdict).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned()), Some(&held.data.prev).and_then(|value| value.as_ref()).map(|value| match value { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" }.to_owned())).map(|value| !value)]) == Some(true));
        Ok(admitted
            .into_iter()
            .map(|held| crate::evaluation::VerdictChanges {
                evaluation_id: held.data.evaluation_id,
                signal: held.data.signal,
                series: held.data.series,
                ts: held.data.ts,
                verdict: held.data.verdict,
                prev: held.data.prev,
                reason: held.data.reason,
            })
            .collect())
    }
}

/// `thalamus.observation.Samples`, generated: every row is one the specification fully determines from the stored `thalamus.observation.SignalSample`s.
impl<P> crate::observation::obligations::SamplesQuery for Generated<P>
where
    P: SignalSampleStorage,
{
    fn samples(&self) -> Result<Vec<crate::observation::Samples>, UnmetObligation> {
        let admitted = SignalSampleStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::observation::Samples {
                sample_id: held.data.sample_id,
                signal: held.data.signal,
                series: held.data.series,
                dims: held.data.dims,
                ts: held.data.ts,
                window: held.data.window,
                reading: held.data.reading,
                text: held.data.text,
                unit: held.data.unit,
                denominator: held.data.denominator,
                provenance: held.data.provenance,
                state: held.state,
            })
            .collect())
    }
}

impl<P: crate::observation::obligations::SeriesSamplesQuery> crate::observation::obligations::SeriesSamplesQuery for Generated<P> {
    fn series_samples(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::observation::SeriesSamples>, UnmetObligation> {
        crate::observation::obligations::SeriesSamplesQuery::series_samples(&self.ports, series)
    }
}

/// `thalamus.rollup.Rollups`, generated: every row is one the specification fully determines from the stored `thalamus.rollup.DailyRollup`s.
impl<P> crate::rollup::obligations::RollupsQuery for Generated<P>
where
    P: DailyRollupStorage,
{
    fn rollups(&self) -> Result<Vec<crate::rollup::Rollups>, UnmetObligation> {
        let admitted = DailyRollupStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::rollup::Rollups {
                rollup_id: held.data.rollup_id,
                signal: held.data.signal,
                series: held.data.series,
                day: held.data.day,
                n: held.data.n,
                state: held.state,
            })
            .collect())
    }
}

impl<P: crate::rollup::obligations::SeriesRollupsQuery> crate::rollup::obligations::SeriesRollupsQuery for Generated<P> {
    fn series_rollups(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::rollup::SeriesRollups>, UnmetObligation> {
        crate::rollup::obligations::SeriesRollupsQuery::series_rollups(&self.ports, series)
    }
}

impl<P: crate::transition::obligations::SeriesTransitionsQuery> crate::transition::obligations::SeriesTransitionsQuery for Generated<P> {
    fn series_transitions(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::transition::SeriesTransitions>, UnmetObligation> {
        crate::transition::obligations::SeriesTransitionsQuery::series_transitions(&self.ports, series)
    }
}

/// `thalamus.transition.Transitions`, generated: every row is one the specification fully determines from the stored `thalamus.transition.Transition`s.
impl<P> crate::transition::obligations::TransitionsQuery for Generated<P>
where
    P: TransitionStorage,
{
    fn transitions(&self) -> Result<Vec<crate::transition::Transitions>, UnmetObligation> {
        let admitted = TransitionStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::transition::Transitions {
                transition_id: held.data.transition_id,
                signal: held.data.signal,
                series: held.data.series,
                from_verdict: held.data.from_verdict,
                to_verdict: held.data.to_verdict,
                settled_ts: held.data.settled_ts,
                state: held.state,
            })
            .collect())
    }
}

/// The typed refusal of a request the model declares no outcome for.
fn undeclared(source: &'static str) -> UnmetObligation {
    let capability = "command behaviour";
    UnmetObligation { capability, source }
}

/// A guard's truth, where it has one: Unknown selects no branch, so the model declares no outcome.
fn decided(truth: Option<bool>, command: &'static str) -> Result<bool, UnmetObligation> {
    truth.ok_or_else(|| undeclared(command))
}

/// A guard's truth, where it has one. Unknown with no instant to read is the command clock this
/// decision needs and its context did not supply: the context's own unavailable answer where it
/// named one, otherwise the missing command clock, named as such. Any other Unknown selects no
/// branch, so the model declares no outcome.
fn decided_at(
    truth: Option<bool>,
    now: &Result<Option<crate::primitives::Timestamp>, UnmetObligation>,
    command: &'static str,
) -> Result<bool, UnmetObligation> {
    match (truth, now) {
        (Some(truth), _) => Ok(truth),
        (None, Err(unavailable)) => Err(unavailable.clone()),
        (None, Ok(None)) => Err(UnmetObligation { capability: "command clock", source: command }),
        (None, Ok(Some(_))) => Err(undeclared(command)),
    }
}

/// Three-valued conjunction: false wins, then Unknown.
fn all(truths: &[Option<bool>]) -> Option<bool> {
    if truths.contains(&Some(false)) {
        Some(false)
    } else if truths.contains(&None) {
        None
    } else {
        Some(true)
    }
}

/// Three-valued disjunction: true wins, then Unknown.
fn any(truths: &[Option<bool>]) -> Option<bool> {
    if truths.contains(&Some(true)) {
        Some(true)
    } else if truths.contains(&None) {
        None
    } else {
        Some(false)
    }
}

/// Equality of two read values; an unread one is Unknown.
fn equal<T: PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {
    Some(left? == right?)
}

/// Orders an RFC 3339 rendering against the decision's one instant moved by `seconds`, by the
/// instants each names; an unread value, no instant to read, or one that names no instant is
/// Unknown — never ordered by its spelling, and never read from another clock.
fn compare_with_now(
    value: Option<String>,
    now: &Result<Option<crate::primitives::Timestamp>, UnmetObligation>,
    seconds: i64,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let (at, nanos) = instant_of(&value?)?;
    let (decided, decided_nanos) = instant_of(&now.as_ref().ok()?.as_ref()?.0)?;
    Some(accepts((at, nanos).cmp(&(decided.checked_add(seconds)?, decided_nanos))))
}

/// The instant an RFC 3339 `date-time` names, as seconds from the epoch and nanoseconds.
fn instant_of(text: &str) -> Option<(i64, u32)> {
    let bytes = text.as_bytes();
    let digits = |from: usize, to: usize| -> Option<u32> {
        let slice = bytes.get(from..to)?;
        if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        slice
            .iter()
            .try_fold(0u32, |total, digit| Some(total * 10 + u32::from(digit - b'0')))
    };
    let at = |index: usize, expected: &[u8]| bytes.get(index).is_some_and(|b| expected.contains(b));
    if !(at(4, b"-") && at(7, b"-") && at(10, b"Tt") && at(13, b":") && at(16, b":")) {
        return None;
    }
    let (year, month, day) = (i64::from(digits(0, 4)?), digits(5, 7)?, digits(8, 10)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let length = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
    if day < 1 || day > length || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut position = 19;
    let mut nanos = 0u32;
    if at(position, b".") {
        let mut end = position + 1;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        let width = end - position - 1;
        if width == 0 || width > 9 {
            return None;
        }
        nanos = digits(position + 1, end)? * 10u32.pow(u32::try_from(9 - width).ok()?);
        position = end;
    }
    let offset = match bytes.get(position..)? {
        b"Z" | b"z" => 0i64,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (hours, minutes) = (digits(position + 1, position + 3)?, digits(position + 4, position + 6)?);
            if hours > 23 || minutes > 59 {
                return None;
            }
            let magnitude = i64::from(hours * 3600 + minutes * 60);
            if *sign == b'-' { -magnitude } else { magnitude }
        }
        _ => return None,
    };
    let shifted = if month <= 2 { year - 1 } else { year };
    let era = shifted.div_euclid(400);
    let of_era = shifted - era * 400;
    let march = i64::from(if month > 2 { month - 3 } else { month + 9 });
    let of_year = (153 * march + 2) / 5 + i64::from(day) - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    let days = era * 146_097 + of_cycle - 719_468;
    Some((days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset, nanos))
}
