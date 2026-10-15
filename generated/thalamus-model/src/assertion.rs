// generated from thalamus v1
// model digest 51d971079fa66242e2a2f34142dbe44422e1204f980cec980f14fbeb58009d94
// contract digest fd2d9e49183a79ea356f366023aa03cd159da99e8e343e0af5bc5233645b32af
// do not edit: regenerate with `ess synthesize --layout crate`

//! Assertions — `thalamus.assertion`.
//!
//! A caller (a release, a merge request, a test run, a controller) asks whether named signals stayed within baseline over a window, and thalamus answers with a result, not a guess. Unknown is not a pass.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Asker — `thalamus.assertion.Asker`.
///
/// Who asked; carried into the answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asker {
    /// `kind` — `thalamus.assertion.AskerKind`.
    pub kind: AskerKind,
    /// `ref` — `String`.
    pub r#ref: String,
}

/// AskerKind — `thalamus.assertion.AskerKind`: one of a closed set of names.
///
/// What kind of caller asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskerKind {
    /// `release`.
    Release,
    /// `merge_request`.
    MergeRequest,
    /// `test_run`.
    TestRun,
    /// `controller`.
    Controller,
}

/// The states of `thalamus.assertion.Assertion`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Assertion<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionState {
    /// `Answered`.
    Answered,
    /// `Asked`.
    Asked,
}

/// AssertionId — `thalamus.assertion.AssertionId`: a distinct wrapper around `String`.
///
/// An assertion id, minted by the service.
///
/// Every value starts with `asr_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionId(pub String);

/// AssertionResult — `thalamus.assertion.AssertionResult`: one of a closed set of names.
///
/// The answer to an assertion. Only within passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionResult {
    /// `within`.
    Within,
    /// `breach`.
    Breach,
    /// `unknown`.
    Unknown,
}

/// Within — `thalamus.assertion.Within`: one of a closed set of names.
///
/// What the signals must stay within.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Within {
    /// `baseline`.
    Baseline,
}

/// What Assertion — `thalamus.assertion.Assertion` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Assertion<S>`], and at a boundary by [`AssertionSnapshot::state`].
///
/// Every value satisfies `((state == Asked and not (defined(result)) and not (defined(worst)) and not (defined(reason))) or (state == Answered and defined(result) and defined(worst)))` — checked by [`AssertionData::broken_invariant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionData {
    /// The identity: `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `signals` — `List<thalamus.definition.SignalId>`.
    ///
    /// Carries `signals`: `thalamus.assertion.Assertion` references many `thalamus.definition.SignalDefinition`.
    pub signals: Vec<crate::definition::SignalId>,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `after` — `Timestamp`.
    pub after: crate::primitives::Timestamp,
    /// `horizon` — `Duration`.
    pub horizon: crate::primitives::Duration,
    /// `within` — `thalamus.assertion.Within`.
    pub within: Within,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
    /// `result` — `Optional<thalamus.assertion.AssertionResult>`.
    pub result: Option<AssertionResult>,
    /// `worst` — `Optional<thalamus.evaluation.Verdict>`.
    pub worst: Option<crate::evaluation::Verdict>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

impl AssertionData {
    /// The first declared invariant of `thalamus.assertion.Assertion` this value breaks, as the specification declares it,
    /// or `None` when it breaks none.
    ///
    /// An invariant is broken only when it is false of this value. One that reads something
    /// absent — an empty `Optional`, a list position past the end, or `state`, which this
    /// type does not hold — decides nothing, as the conformance interpreter reads it.
    pub fn broken_invariant(&self) -> Option<&'static str> {
        use crate::primitives::invariant as iv;
        if iv::broken(iv::any([iv::all([iv::compare(None, iv::Op::Eq, Some(iv::Fact::text("Asked")), false, true), iv::not(Some(self.result.as_ref().is_some())), iv::not(Some(self.worst.as_ref().is_some())), iv::not(Some(self.reason.as_ref().is_some()))]), iv::all([iv::compare(None, iv::Op::Eq, Some(iv::Fact::text("Answered")), false, true), Some(self.result.as_ref().is_some()), Some(self.worst.as_ref().is_some())])])) {
            return Some("((state == Asked and not (defined(result)) and not (defined(worst)) and not (defined(reason))) or (state == Answered and defined(result) and defined(worst)))");
        }
        None
    }
}

/// The states of `thalamus.assertion.Assertion`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](assertion_state::Marker), so [`Assertion<S>`](Assertion) can only ever rest in a real state.
pub mod assertion_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Answered {}
        impl Sealed for super::Asked {}
    }

    /// A declared state of `Assertion`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::AssertionState;
    }

    /// `Answered`. Terminal: an instance may rest here forever.
    pub struct Answered;

    impl Marker for Answered {
        const STATE: super::AssertionState = super::AssertionState::Answered;
    }

    /// `Asked`. Where a new instance starts.
    pub struct Asked;

    impl Marker for Asked {
        const STATE: super::AssertionState = super::AssertionState::Asked;
    }
}

/// Assertion — `thalamus.assertion.Assertion` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Asked`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`AssertionSnapshot`]
/// and [`AssertionSnapshot::refine`].
pub struct Assertion<S: assertion_state::Marker> {
    data: AssertionData,
    state: core::marker::PhantomData<S>,
}

impl<S: assertion_state::Marker> Assertion<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> AssertionState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &AssertionData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> AssertionData {
        self.data
    }
}

impl Assertion<assertion_state::Asked> {
    /// A new instance, resting in `Asked` — the only state the lifecycle starts one in.
    pub fn new(data: AssertionData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Assertion<assertion_state::Asked> {
    /// `answer` — `Asked` → `Answered`. Taken by the `answered` outcome of `thalamus.assertion.AnswerAssertion`.
    pub fn answer(self) -> Assertion<assertion_state::Answered> {
        Assertion {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.assertion.Assertion` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`AssertionSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: AssertionState,
    /// What it holds.
    pub data: AssertionData,
}

/// An `Assertion` in whichever declared state it was found.
pub enum AnyAssertion {
    /// Resting in `Answered`.
    Answered(Assertion<assertion_state::Answered>),
    /// Resting in `Asked`.
    Asked(Assertion<assertion_state::Asked>),
}

impl AssertionSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `AssertionState` cannot spell one.
    pub fn refine(self) -> AnyAssertion {
        match self.state {
            AssertionState::Answered => AnyAssertion::Answered(Assertion {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            AssertionState::Asked => AnyAssertion::Asked(Assertion {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyAssertion {
    /// The state, as the runtime value.
    pub fn state(&self) -> AssertionState {
        match self {
            Self::Answered(_) => AssertionState::Answered,
            Self::Asked(_) => AssertionState::Asked,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> AssertionSnapshot {
        match self {
            Self::Answered(instance) => AssertionSnapshot {
                state: AssertionState::Answered,
                data: instance.into_data(),
            },
            Self::Asked(instance) => AssertionSnapshot {
                state: AssertionState::Asked,
                data: instance.into_data(),
            },
        }
    }
}

/// Answer an assertion — the input of `thalamus.assertion.AnswerAssertion`.
///
/// Close an open assertion with its result and the worst verdict over its window.
///
/// Everything it can result in is [`AnswerAssertionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerAssertion {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `result` — `thalamus.assertion.AssertionResult`.
    pub result: AssertionResult,
    /// `worst` — `thalamus.evaluation.Verdict`.
    pub worst: crate::evaluation::Verdict,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

/// Everything `thalamus.assertion.AnswerAssertion` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerAssertionOutcome {
    /// `reason-missing` — when `(result != within and not (defined(reason)))`.
    ///
    /// breach or unknown came without a reason; nothing was answered.
    ReasonMissing {
        /// Why it was refused: `thalamus.assertion.ReasonRequired`.
        error: ReasonRequired,
    },
    /// `contradicts-verdict` — when `((result == breach and worst in [OK, N_A, UNKNOWN]) or (result != breach and worst in [WARN, CRIT]) or (result == within and worst == UNKNOWN))`.
    ///
    /// The result and the worst verdict disagree; nothing was answered.
    ContradictsVerdict {
        /// Why it was refused: `thalamus.assertion.ResultContradictsVerdict`.
        error: ResultContradictsVerdict,
    },
    /// `answered` — otherwise.
    ///
    /// The assertion is answered; the asker reads the result, and only within passes.
    Answered {
        /// The `thalamus.assertion.AssertionAnswered` this outcome publishes.
        assertion_answered: AssertionAnswered,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The assertion is already answered; nothing changed.
    WrongState {
        /// Why it was refused: `thalamus.assertion.AssertionStateConflict`.
        error: AssertionStateConflict,
    },
    /// `no-such-assertion` — for an identity no record carries.
    ///
    /// No assertion has this id; nothing changed.
    NoSuchAssertion {
        /// Why it was refused: `thalamus.assertion.UnknownAssertion`.
        error: UnknownAssertion,
    },
}

/// Ask an assertion — the input of `thalamus.assertion.AskAssertion`.
///
/// Ask whether the named signals stay within baseline after an instant, for a horizon.
///
/// Everything it can result in is [`AskAssertionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskAssertion {
    /// `signals` — `List<thalamus.definition.SignalId>`.
    pub signals: Vec<crate::definition::SignalId>,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `after` — `Timestamp`.
    pub after: crate::primitives::Timestamp,
    /// `horizon` — `Duration`.
    pub horizon: crate::primitives::Duration,
    /// `within` — `thalamus.assertion.Within`.
    pub within: Within,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
}

/// Actual typed response of `thalamus.assertion.AskAssertion`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskAssertionResponse {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
}

/// Everything `thalamus.assertion.AskAssertion` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskAssertionOutcome {
    /// `no-signals` — when `signals.count == 0`.
    ///
    /// No signal was named; nothing was asked.
    NoSignals {
        /// Why it was refused: `thalamus.assertion.NoSignals`.
        error: NoSignals,
    },
    /// `asked` — otherwise.
    ///
    /// The assertion is open and its id returned; it is answered once its horizon has passed.
    Asked {
        /// Actual response returned by this branch.
        response: AskAssertionResponse,
        /// The `thalamus.assertion.AssertionAsked` this outcome publishes.
        assertion_asked: AssertionAsked,
    },
}

impl AskAssertionOutcome {
    /// Compare the actual returned response with response-mapped emitted fields.
    pub fn response_payload_matches(&self) -> bool {
        match self {
            Self::NoSignals { .. } => true,
            Self::Asked { response, assertion_asked, .. } => assertion_asked.assertion_id == response.assertion_id.clone(),
        }
    }
}

/// AssertionAnswered — the event `thalamus.assertion.AssertionAnswered`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionAnswered {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
    /// `result` — `thalamus.assertion.AssertionResult`.
    pub result: AssertionResult,
    /// `worst` — `thalamus.evaluation.Verdict`.
    pub worst: crate::evaluation::Verdict,
}

/// AssertionAsked — the event `thalamus.assertion.AssertionAsked`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionAsked {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
}

/// The declared error `thalamus.assertion.AssertionStateConflict`.
///
/// The assertion is already answered; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionStateConflict {
    /// `current` — `thalamus.assertion.Assertion.State`.
    pub current: AssertionState,
}

/// The declared error `thalamus.assertion.NoSignals`.
///
/// An assertion names at least one signal; none was named, so nothing was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSignals;

/// The declared error `thalamus.assertion.ReasonRequired`.
///
/// A breach or unknown result carries a reason; none was given, so nothing was answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonRequired {
    /// `result` — `thalamus.assertion.AssertionResult`.
    pub result: AssertionResult,
}

/// The declared error `thalamus.assertion.ResultContradictsVerdict`.
///
/// breach goes with a worst verdict of WARN or CRIT and only then, and within never goes with UNKNOWN; the answer broke that, so nothing was answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultContradictsVerdict {
    /// `result` — `thalamus.assertion.AssertionResult`.
    pub result: AssertionResult,
    /// `worst` — `thalamus.evaluation.Verdict`.
    pub worst: crate::evaluation::Verdict,
}

/// The declared error `thalamus.assertion.UnknownAssertion`.
///
/// No assertion has this id; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAssertion {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
}

/// Assertions — one row of the view `thalamus.assertion.Assertions`.
///
/// Projects `thalamus.assertion.Assertion` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assertions {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `signals` — `List<thalamus.definition.SignalId>`.
    pub signals: Vec<crate::definition::SignalId>,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `after` — `Timestamp`.
    pub after: crate::primitives::Timestamp,
    /// `horizon` — `Duration`.
    pub horizon: crate::primitives::Duration,
    /// `within` — `thalamus.assertion.Within`.
    pub within: Within,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
    /// `result` — `Optional<thalamus.assertion.AssertionResult>`.
    pub result: Option<AssertionResult>,
    /// `worst` — `Optional<thalamus.evaluation.Verdict>`.
    pub worst: Option<crate::evaluation::Verdict>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
    /// `state` — `thalamus.assertion.Assertion.State`.
    pub state: AssertionState,
}

/// Open assertions — one row of the view `thalamus.assertion.OpenAssertions`.
///
/// Projects `thalamus.assertion.Assertion` at `read_your_writes` consistency, containing instances where `state == Asked`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAssertions {
    /// `assertion_id` — `thalamus.assertion.AssertionId`.
    pub assertion_id: AssertionId,
    /// `signals` — `List<thalamus.definition.SignalId>`.
    pub signals: Vec<crate::definition::SignalId>,
    /// `after` — `Timestamp`.
    pub after: crate::primitives::Timestamp,
    /// `horizon` — `Duration`.
    pub horizon: crate::primitives::Duration,
    /// `asker` — `thalamus.assertion.Asker`.
    pub asker: Asker,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.assertion.AnswerAssertion` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait AnswerAssertionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.assertion.AnswerAssertion`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn answer_assertion(&mut self, input: super::AnswerAssertion) -> Result<super::AnswerAssertionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.assertion.AskAssertion` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `thalamus.assertion.AskAssertion` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `no-signals` when `signals.count == 0`, error `thalamus.assertion.NoSignals`; `asked` otherwise, creates `thalamus.assertion.Assertion`, emits `thalamus.assertion.AssertionAsked`.
    pub trait AskAssertionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.assertion.AskAssertion`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn ask_assertion(&mut self, input: super::AskAssertion) -> Result<super::AskAssertionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.assertion.Assertions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait AssertionsQuery {
        /// Serves `thalamus.assertion.Assertions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn assertions(&self) -> Result<Vec<super::Assertions>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.assertion.OpenAssertions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait OpenAssertionsQuery {
        /// Serves `thalamus.assertion.OpenAssertions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn open_assertions(&self) -> Result<Vec<super::OpenAssertions>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl AskAssertionBehavior for Unimplemented {
        fn ask_assertion(&mut self, _input: super::AskAssertion) -> Result<super::AskAssertionOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.assertion.AskAssertion" })
        }
    }
}
