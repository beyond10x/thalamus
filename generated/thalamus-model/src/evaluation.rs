// generated from thalamus v1
// model digest 6f2848b89ff6190d89c005ba09023b83056dbdfb2fe499f9e414dc63d7d866bd
// contract digest 52593762eb7c5911c4f82160d91abef9b7c9217fd30b29c6b5cb7e3ac76f0515
// do not edit: regenerate with `ess synthesize --layout crate`

//! Evaluations — `thalamus.evaluation`.
//!
//! One verdict over one sample, with the rule and the definition revision that decided it. Kept apart from the sample so a threshold change re-grades history without reading the source again.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Baseline — `thalamus.evaluation.Baseline`.
///
/// The baseline a sample was graded against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baseline {
    /// `kind` — `thalamus.evaluation.BaselineLabel`.
    pub kind: BaselineLabel,
    /// `offsets` — `List<String>`.
    pub offsets: Vec<String>,
    /// `reading` — `thalamus.observation.Reading`.
    pub reading: crate::observation::Reading,
}

/// BaselineLabel — `thalamus.evaluation.BaselineLabel`: a distinct wrapper around `String`.
///
/// Which baseline the sample was graded against (offset_pooled, fixed, rolling_median).
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineLabel(pub String);

/// EvaluationId — `thalamus.evaluation.EvaluationId`: a distinct wrapper around `String`.
///
/// An evaluation id, minted by the grader.
///
/// Every value starts with `evl_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationId(pub String);

/// The states of `thalamus.evaluation.SignalEvaluation`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SignalEvaluation<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalEvaluationState {
    /// `Recorded`.
    Recorded,
}

/// Verdict — `thalamus.evaluation.Verdict`: one of a closed set of names.
///
/// The one verdict ladder. exit is the process exit code; rank orders an aggregate worst-first (CRIT > WARN > UNKNOWN > OK = N/A); scored says whether the verdict carries a score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// `OK`.
    OK,
    /// `N_A`.
    NA,
    /// `WARN`.
    WARN,
    /// `CRIT`.
    CRIT,
    /// `UNKNOWN`.
    UNKNOWN,
}

/// What SignalEvaluation — `thalamus.evaluation.SignalEvaluation` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SignalEvaluation<S>`], and at a boundary by [`SignalEvaluationSnapshot::state`].
///
/// Every value satisfies `definition_rev >= 1` — checked by [`SignalEvaluationData::broken_invariant`].
/// Every value satisfies `(verdict == OK or defined(reason))` — checked by [`SignalEvaluationData::broken_invariant`].
/// Every value satisfies `(verdict.scored == true or not (defined(score)))` — checked by [`SignalEvaluationData::broken_invariant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEvaluationData {
    /// The identity: `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
    /// `sample_id` — `thalamus.observation.SampleId`.
    ///
    /// Carries `sample`: `thalamus.evaluation.SignalEvaluation` references one `thalamus.observation.SignalSample`.
    pub sample_id: crate::observation::SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    ///
    /// Carries `definition`: `thalamus.evaluation.SignalEvaluation` references one `thalamus.definition.SignalDefinition`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
    /// `score` — `Optional<Decimal>`.
    pub score: Option<crate::primitives::Decimal>,
    /// `baseline` — `Optional<thalamus.evaluation.Baseline>`.
    pub baseline: Option<Baseline>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

impl SignalEvaluationData {
    /// The first declared invariant of `thalamus.evaluation.SignalEvaluation` this value breaks, as the specification declares it,
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
        if iv::broken(iv::any([iv::compare(Some(iv::Fact::text(match self.verdict { Verdict::OK => "OK", Verdict::NA => "N_A", Verdict::WARN => "WARN", Verdict::CRIT => "CRIT", Verdict::UNKNOWN => "UNKNOWN" })), iv::Op::Eq, Some(iv::Fact::text("OK")), false, true), Some(self.reason.as_ref().is_some())])) {
            return Some("(verdict == OK or defined(reason))");
        }
        if iv::broken(iv::any([iv::any_of(Some(iv::Fact::text(match self.verdict { Verdict::OK => "OK", Verdict::NA => "N_A", Verdict::WARN => "WARN", Verdict::CRIT => "CRIT", Verdict::UNKNOWN => "UNKNOWN" })), &[Some(iv::Fact::text("OK")), Some(iv::Fact::text("WARN")), Some(iv::Fact::text("CRIT"))]), iv::not(Some(self.score.as_ref().is_some()))])) {
            return Some("(verdict.scored == true or not (defined(score)))");
        }
        None
    }
}

/// The states of `thalamus.evaluation.SignalEvaluation`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](signal_evaluation_state::Marker), so [`SignalEvaluation<S>`](SignalEvaluation) can only ever rest in a real state.
pub mod signal_evaluation_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Recorded {}
    }

    /// A declared state of `SignalEvaluation`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::SignalEvaluationState;
    }

    /// `Recorded`. Where a new instance starts.
    pub struct Recorded;

    impl Marker for Recorded {
        const STATE: super::SignalEvaluationState = super::SignalEvaluationState::Recorded;
    }
}

/// SignalEvaluation — `thalamus.evaluation.SignalEvaluation` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Recorded`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`SignalEvaluationSnapshot`]
/// and [`SignalEvaluationSnapshot::refine`].
pub struct SignalEvaluation<S: signal_evaluation_state::Marker> {
    data: SignalEvaluationData,
    state: core::marker::PhantomData<S>,
}

impl<S: signal_evaluation_state::Marker> SignalEvaluation<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> SignalEvaluationState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &SignalEvaluationData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> SignalEvaluationData {
        self.data
    }
}

impl SignalEvaluation<signal_evaluation_state::Recorded> {
    /// A new instance, resting in `Recorded` — the only state the lifecycle starts one in.
    pub fn new(data: SignalEvaluationData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.evaluation.SignalEvaluation` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`SignalEvaluationSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEvaluationSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: SignalEvaluationState,
    /// What it holds.
    pub data: SignalEvaluationData,
}

/// An `SignalEvaluation` in whichever declared state it was found.
pub enum AnySignalEvaluation {
    /// Resting in `Recorded`.
    Recorded(SignalEvaluation<signal_evaluation_state::Recorded>),
}

impl SignalEvaluationSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `SignalEvaluationState` cannot spell one.
    pub fn refine(self) -> AnySignalEvaluation {
        match self.state {
            SignalEvaluationState::Recorded => AnySignalEvaluation::Recorded(SignalEvaluation {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnySignalEvaluation {
    /// The state, as the runtime value.
    pub fn state(&self) -> SignalEvaluationState {
        match self {
            Self::Recorded(_) => SignalEvaluationState::Recorded,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> SignalEvaluationSnapshot {
        match self {
            Self::Recorded(instance) => SignalEvaluationSnapshot {
                state: SignalEvaluationState::Recorded,
                data: instance.into_data(),
            },
        }
    }
}

/// Record an evaluation — the input of `thalamus.evaluation.RecordEvaluation`.
///
/// Append one verdict over one sample, graded against one definition revision.
///
/// Everything it can result in is [`RecordEvaluationOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEvaluation {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: crate::observation::SampleId,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
    /// `score` — `Optional<Decimal>`.
    pub score: Option<crate::primitives::Decimal>,
    /// `baseline` — `Optional<thalamus.evaluation.Baseline>`.
    pub baseline: Option<Baseline>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

/// Actual typed response of `thalamus.evaluation.RecordEvaluation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEvaluationResponse {
    /// `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
}

/// Everything `thalamus.evaluation.RecordEvaluation` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordEvaluationOutcome {
    /// `unknown-sample` — when no `thalamus.observation.SignalSample` carries the identity `input.sample_id` names.
    ///
    /// No sample has this id; nothing was recorded.
    UnknownSample {
        /// Why it was refused: `thalamus.evaluation.UnknownSample`.
        error: UnknownSample,
    },
    /// `reason-missing` — when `(verdict != OK and not (defined(reason)))`.
    ///
    /// A verdict other than OK came without a reason; nothing was recorded.
    ReasonMissing {
        /// Why it was refused: `thalamus.evaluation.ReasonRequired`.
        error: ReasonRequired,
    },
    /// `score-not-admitted` — when `(verdict in [N_A, UNKNOWN] and defined(score))`.
    ///
    /// UNKNOWN or N/A came with a score; nothing was recorded.
    ScoreNotAdmitted {
        /// Why it was refused: `thalamus.evaluation.ScoreNotAdmitted`.
        error: ScoreNotAdmitted,
    },
    /// `recorded` — otherwise.
    ///
    /// The evaluation is recorded and its id returned.
    Recorded {
        /// Actual response returned by this branch.
        response: RecordEvaluationResponse,
        /// The `thalamus.evaluation.EvaluationRecorded` this outcome publishes.
        evaluation_recorded: EvaluationRecorded,
    },
}

impl RecordEvaluationOutcome {
    /// Compare the actual returned response with response-mapped emitted fields.
    pub fn response_payload_matches(&self) -> bool {
        match self {
            Self::UnknownSample { .. } => true,
            Self::ReasonMissing { .. } => true,
            Self::ScoreNotAdmitted { .. } => true,
            Self::Recorded { response, evaluation_recorded, .. } => evaluation_recorded.evaluation_id == response.evaluation_id.clone(),
        }
    }
}

/// EvaluationRecorded — the event `thalamus.evaluation.EvaluationRecorded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationRecorded {
    /// `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: crate::observation::SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
}

/// The declared error `thalamus.evaluation.ReasonRequired`.
///
/// Every verdict other than OK carries a reason; none was given, so nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonRequired {
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
}

/// The declared error `thalamus.evaluation.ScoreNotAdmitted`.
///
/// UNKNOWN and N/A carry no score; one was given, so nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreNotAdmitted {
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
}

/// The declared error `thalamus.evaluation.UnknownSample`.
///
/// No sample has this id; nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSample {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: crate::observation::SampleId,
}

/// Evaluations — one row of the view `thalamus.evaluation.Evaluations`.
///
/// Projects `thalamus.evaluation.SignalEvaluation` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluations {
    /// `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: crate::observation::SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
    /// `score` — `Optional<Decimal>`.
    pub score: Option<crate::primitives::Decimal>,
    /// `baseline` — `Optional<thalamus.evaluation.Baseline>`.
    pub baseline: Option<Baseline>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
    /// `state` — `thalamus.evaluation.SignalEvaluation.State`.
    pub state: SignalEvaluationState,
}

/// Verdicts of a series — one row of the view `thalamus.evaluation.SeriesVerdicts`.
///
/// Projects `thalamus.evaluation.SignalEvaluation` at `read_your_writes` consistency, containing instances where `series == param.series`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesVerdicts {
    /// `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

/// Verdict changes — one row of the view `thalamus.evaluation.VerdictChanges`.
///
/// Projects `thalamus.evaluation.SignalEvaluation` at `read_your_writes` consistency, containing instances where `(defined(prev) and verdict != {fact: prev})`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictChanges {
    /// `evaluation_id` — `thalamus.evaluation.EvaluationId`.
    pub evaluation_id: EvaluationId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `verdict` — `thalamus.evaluation.Verdict`.
    pub verdict: Verdict,
    /// `prev` — `Optional<thalamus.evaluation.Verdict>`.
    pub prev: Option<Verdict>,
    /// `reason` — `Optional<String>`.
    pub reason: Option<String>,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.evaluation.RecordEvaluation` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `thalamus.evaluation.RecordEvaluation` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-sample` when no `thalamus.observation.SignalSample` carries the identity `input.sample_id` names, error `thalamus.evaluation.UnknownSample`; `reason-missing` when `(verdict != OK and not (defined(reason)))`, error `thalamus.evaluation.ReasonRequired`; `score-not-admitted` when `(verdict in [N_A, UNKNOWN] and defined(score))`, error `thalamus.evaluation.ScoreNotAdmitted`; `recorded` otherwise, creates `thalamus.evaluation.SignalEvaluation`, emits `thalamus.evaluation.EvaluationRecorded`.
    pub trait RecordEvaluationBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.evaluation.RecordEvaluation`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn record_evaluation(&mut self, input: super::RecordEvaluation) -> Result<super::RecordEvaluationOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.evaluation.Evaluations` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait EvaluationsQuery {
        /// Serves `thalamus.evaluation.Evaluations` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn evaluations(&self) -> Result<Vec<super::Evaluations>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.evaluation.SeriesVerdicts` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a view parameter (`params:`), which a generated query does not apply.
    ///
    /// Contract: a query answering `thalamus.evaluation.SeriesVerdicts` with rows projected from `thalamus.evaluation.SignalEvaluation` at `read_your_writes` consistency, containing instances where `series == param.series`.
    pub trait SeriesVerdictsQuery {
        /// Serves `thalamus.evaluation.SeriesVerdicts` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn series_verdicts(&self, series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesVerdicts>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.evaluation.VerdictChanges` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait VerdictChangesQuery {
        /// Serves `thalamus.evaluation.VerdictChanges` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn verdict_changes(&self) -> Result<Vec<super::VerdictChanges>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl RecordEvaluationBehavior for Unimplemented {
        fn record_evaluation(&mut self, _input: super::RecordEvaluation) -> Result<super::RecordEvaluationOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.evaluation.RecordEvaluation" })
        }
    }

    impl SeriesVerdictsQuery for Unimplemented {
        fn series_verdicts(&self, _series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesVerdicts>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "thalamus.evaluation.SeriesVerdicts" })
        }
    }
}
