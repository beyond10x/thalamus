// generated from thalamus v1
// model digest 6f2e9f65e0ef065a249a30bb38bae16ef166a5b45577b7a8f20509f1f3eddea7
// contract digest 067f632c99cd362c9c2d5e3d29f9b5d30e6604352424ed4eca28522b738edbee
// do not edit: regenerate with `ess synthesize --layout crate`

//! Transitions — `thalamus.transition`.
//!
//! A settled change of a series' verdict, emitted by the hysteresis once k consecutive evaluations agree. It copies what a reader needs from the evaluations that settled it, so it outlives their raw retention.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// The states of `thalamus.transition.Transition`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Transition<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionState {
    /// `Recorded`.
    Recorded,
}

/// TransitionId — `thalamus.transition.TransitionId`: a distinct wrapper around `String`.
///
/// A transition id, minted by the service.
///
/// Every value starts with `trn_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionId(pub String);

/// What Transition — `thalamus.transition.Transition` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Transition<S>`], and at a boundary by [`TransitionSnapshot::state`].
///
/// Every value satisfies `definition_rev >= 1` — checked by [`TransitionData::broken_invariant`].
/// Every value satisfies `first_ts <= settled_ts` — checked by [`TransitionData::broken_invariant`].
/// Every value satisfies `(not (defined(from_verdict)) or from_verdict != to_verdict)` — checked by [`TransitionData::broken_invariant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionData {
    /// The identity: `transition_id` — `thalamus.transition.TransitionId`.
    pub transition_id: TransitionId,
    /// `signal` — `thalamus.definition.SignalId`.
    ///
    /// Carries `definition`: `thalamus.transition.Transition` references one `thalamus.definition.SignalDefinition`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `from_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub from_verdict: Option<crate::evaluation::Verdict>,
    /// `to_verdict` — `thalamus.evaluation.Verdict`.
    pub to_verdict: crate::evaluation::Verdict,
    /// `first_ts` — `Timestamp`.
    pub first_ts: crate::primitives::Timestamp,
    /// `settled_ts` — `Timestamp`.
    pub settled_ts: crate::primitives::Timestamp,
    /// `settling_score` — `Optional<Decimal>`.
    pub settling_score: Option<crate::primitives::Decimal>,
    /// `evaluations` — `List<thalamus.evaluation.EvaluationId>`.
    ///
    /// Carries `evaluations`: `thalamus.transition.Transition` references many `thalamus.evaluation.SignalEvaluation`.
    pub evaluations: Vec<crate::evaluation::EvaluationId>,
}

impl TransitionData {
    /// The first declared invariant of `thalamus.transition.Transition` this value breaks, as the specification declares it,
    /// or `None` when it breaks none.
    ///
    /// An invariant is broken only when it is false of this value. One that reads something
    /// absent — an empty `Optional`, a list position past the end, or `state`, which this
    /// type does not hold — decides nothing, as the conformance interpreter reads it.
    pub fn broken_invariant(&self) -> Option<&'static str> {
        use crate::primitives::invariant as iv;
        if iv::broken(iv::compare(Some(iv::Fact::integer(self.definition_rev)), iv::Op::Ge, iv::Fact::number("1"), false, true)) {
            return Some("definition_rev >= 1");
        }
        if iv::broken(iv::compare(Some(iv::Fact::text(&self.first_ts.0)), iv::Op::Le, Some(iv::Fact::text(&self.settled_ts.0)), true, true)) {
            return Some("first_ts <= settled_ts");
        }
        if iv::broken(iv::any([iv::not(Some(self.from_verdict.as_ref().is_some())), iv::compare(self.from_verdict.as_ref().map(|v| iv::Fact::text(match v { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" })), iv::Op::Ne, Some(iv::Fact::text(match self.to_verdict { crate::evaluation::Verdict::OK => "OK", crate::evaluation::Verdict::NA => "N_A", crate::evaluation::Verdict::WARN => "WARN", crate::evaluation::Verdict::CRIT => "CRIT", crate::evaluation::Verdict::UNKNOWN => "UNKNOWN" })), false, true)])) {
            return Some("(not (defined(from_verdict)) or from_verdict != to_verdict)");
        }
        None
    }
}

/// The states of `thalamus.transition.Transition`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](transition_state::Marker), so [`Transition<S>`](Transition) can only ever rest in a real state.
pub mod transition_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Recorded {}
    }

    /// A declared state of `Transition`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::TransitionState;
    }

    /// `Recorded`. Where a new instance starts.
    pub struct Recorded;

    impl Marker for Recorded {
        const STATE: super::TransitionState = super::TransitionState::Recorded;
    }
}

/// Transition — `thalamus.transition.Transition` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Recorded`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`TransitionSnapshot`]
/// and [`TransitionSnapshot::refine`].
pub struct Transition<S: transition_state::Marker> {
    data: TransitionData,
    state: core::marker::PhantomData<S>,
}

impl<S: transition_state::Marker> Transition<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> TransitionState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &TransitionData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> TransitionData {
        self.data
    }
}

impl Transition<transition_state::Recorded> {
    /// A new instance, resting in `Recorded` — the only state the lifecycle starts one in.
    pub fn new(data: TransitionData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.transition.Transition` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`TransitionSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: TransitionState,
    /// What it holds.
    pub data: TransitionData,
}

/// An `Transition` in whichever declared state it was found.
pub enum AnyTransition {
    /// Resting in `Recorded`.
    Recorded(Transition<transition_state::Recorded>),
}

impl TransitionSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `TransitionState` cannot spell one.
    pub fn refine(self) -> AnyTransition {
        match self.state {
            TransitionState::Recorded => AnyTransition::Recorded(Transition {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyTransition {
    /// The state, as the runtime value.
    pub fn state(&self) -> TransitionState {
        match self {
            Self::Recorded(_) => TransitionState::Recorded,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> TransitionSnapshot {
        match self {
            Self::Recorded(instance) => TransitionSnapshot {
                state: TransitionState::Recorded,
                data: instance.into_data(),
            },
        }
    }
}

/// Record a transition — the input of `thalamus.transition.RecordTransition`.
///
/// Append one settled verdict change of one series.
///
/// Everything it can result in is [`RecordTransitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTransition {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `from_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub from_verdict: Option<crate::evaluation::Verdict>,
    /// `to_verdict` — `thalamus.evaluation.Verdict`.
    pub to_verdict: crate::evaluation::Verdict,
    /// `first_ts` — `Timestamp`.
    pub first_ts: crate::primitives::Timestamp,
    /// `settled_ts` — `Timestamp`.
    pub settled_ts: crate::primitives::Timestamp,
    /// `settling_score` — `Optional<Decimal>`.
    pub settling_score: Option<crate::primitives::Decimal>,
    /// `evaluations` — `List<thalamus.evaluation.EvaluationId>`.
    pub evaluations: Vec<crate::evaluation::EvaluationId>,
}

/// Actual typed response of `thalamus.transition.RecordTransition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTransitionResponse {
    /// `transition_id` — `thalamus.transition.TransitionId`.
    pub transition_id: TransitionId,
}

/// Everything `thalamus.transition.RecordTransition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordTransitionOutcome {
    /// `unknown-signal` — when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names.
    ///
    /// No definition has this id; nothing was recorded.
    UnknownSignal {
        /// Why it was refused: `thalamus.transition.UnknownSignal`.
        error: UnknownSignal,
    },
    /// `recorded` — otherwise.
    ///
    /// The transition is recorded and its id returned.
    Recorded {
        /// Actual response returned by this branch.
        response: RecordTransitionResponse,
        /// The `thalamus.transition.TransitionRecorded` this outcome publishes.
        transition_recorded: TransitionRecorded,
    },
}

impl RecordTransitionOutcome {
    /// Compare the actual returned response with response-mapped emitted fields.
    pub fn response_payload_matches(&self) -> bool {
        match self {
            Self::UnknownSignal { .. } => true,
            Self::Recorded { response, transition_recorded, .. } => transition_recorded.transition_id == response.transition_id.clone(),
        }
    }
}

/// TransitionRecorded — the event `thalamus.transition.TransitionRecorded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRecorded {
    /// `transition_id` — `thalamus.transition.TransitionId`.
    pub transition_id: TransitionId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `from_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub from_verdict: Option<crate::evaluation::Verdict>,
    /// `to_verdict` — `thalamus.evaluation.Verdict`.
    pub to_verdict: crate::evaluation::Verdict,
    /// `settled_ts` — `Timestamp`.
    pub settled_ts: crate::primitives::Timestamp,
}

/// The declared error `thalamus.transition.UnknownSignal`.
///
/// No definition has this signal id; nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSignal {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
}

/// Transitions of a series — one row of the view `thalamus.transition.SeriesTransitions`.
///
/// Projects `thalamus.transition.Transition` at `read_your_writes` consistency, containing instances where `series == param.series`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesTransitions {
    /// `transition_id` — `thalamus.transition.TransitionId`.
    pub transition_id: TransitionId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `from_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub from_verdict: Option<crate::evaluation::Verdict>,
    /// `to_verdict` — `thalamus.evaluation.Verdict`.
    pub to_verdict: crate::evaluation::Verdict,
    /// `first_ts` — `Timestamp`.
    pub first_ts: crate::primitives::Timestamp,
    /// `settled_ts` — `Timestamp`.
    pub settled_ts: crate::primitives::Timestamp,
    /// `settling_score` — `Optional<Decimal>`.
    pub settling_score: Option<crate::primitives::Decimal>,
    /// `evaluations` — `List<thalamus.evaluation.EvaluationId>`.
    pub evaluations: Vec<crate::evaluation::EvaluationId>,
}

/// Transitions — one row of the view `thalamus.transition.Transitions`.
///
/// Projects `thalamus.transition.Transition` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transitions {
    /// `transition_id` — `thalamus.transition.TransitionId`.
    pub transition_id: TransitionId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `from_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub from_verdict: Option<crate::evaluation::Verdict>,
    /// `to_verdict` — `thalamus.evaluation.Verdict`.
    pub to_verdict: crate::evaluation::Verdict,
    /// `settled_ts` — `Timestamp`.
    pub settled_ts: crate::primitives::Timestamp,
    /// `state` — `thalamus.transition.Transition.State`.
    pub state: TransitionState,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.transition.RecordTransition` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `thalamus.transition.RecordTransition` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-signal` when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names, error `thalamus.transition.UnknownSignal`; `recorded` otherwise, creates `thalamus.transition.Transition`, emits `thalamus.transition.TransitionRecorded`.
    pub trait RecordTransitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.transition.RecordTransition`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn record_transition(&mut self, input: super::RecordTransition) -> Result<super::RecordTransitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.transition.SeriesTransitions` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a view parameter (`params:`), which a generated query does not apply.
    ///
    /// Contract: a query answering `thalamus.transition.SeriesTransitions` with rows projected from `thalamus.transition.Transition` at `read_your_writes` consistency, containing instances where `series == param.series`.
    pub trait SeriesTransitionsQuery {
        /// Serves `thalamus.transition.SeriesTransitions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn series_transitions(&self, series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesTransitions>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.transition.Transitions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait TransitionsQuery {
        /// Serves `thalamus.transition.Transitions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn transitions(&self) -> Result<Vec<super::Transitions>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl RecordTransitionBehavior for Unimplemented {
        fn record_transition(&mut self, _input: super::RecordTransition) -> Result<super::RecordTransitionOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.transition.RecordTransition" })
        }
    }

    impl SeriesTransitionsQuery for Unimplemented {
        fn series_transitions(&self, _series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesTransitions>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "thalamus.transition.SeriesTransitions" })
        }
    }
}
