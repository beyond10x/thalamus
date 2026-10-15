// generated from thalamus v1
// model digest 51d971079fa66242e2a2f34142dbe44422e1204f980cec980f14fbeb58009d94
// contract digest fd2d9e49183a79ea356f366023aa03cd159da99e8e343e0af5bc5233645b32af
// do not edit: regenerate with `ess synthesize --layout crate`

//! Samples — `thalamus.observation`.
//!
//! One reading of one signal for one series. Append-only; a sample has no lifecycle beyond being recorded.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Bound — `thalamus.observation.Bound`: one of a closed set of names.
///
/// Which side a partial reading's value bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// `lower`.
    Lower,
    /// `upper`.
    Upper,
}

/// CompleteReading — `thalamus.observation.CompleteReading`.
///
/// Read completely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteReading {
    /// `value` — `Decimal`.
    pub value: crate::primitives::Decimal,
}

/// Denominator — `thalamus.observation.Denominator`.
///
/// The population behind a ratio or count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denominator {
    /// `n` — `Integer`.
    pub n: i64,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: crate::definition::Unit,
    /// `since` — `Timestamp`.
    pub since: crate::primitives::Timestamp,
    /// `age_s` — `Integer`.
    pub age_s: i64,
}

/// PartialReading — `thalamus.observation.PartialReading`.
///
/// Read but cut (a full page, a paged read that could not finish).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialReading {
    /// `value` — `Decimal`.
    pub value: crate::primitives::Decimal,
    /// `bound` — `thalamus.observation.Bound`.
    pub bound: Bound,
}

/// Provenance — `thalamus.observation.Provenance`.
///
/// Enough to re-run the read. Names a connection, never a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// `producer` — `String`.
    pub producer: String,
    /// `version` — `String`.
    pub version: String,
    /// `definition_rev` — `Integer`.
    pub definition_rev: i64,
    /// `adapter` — `thalamus.definition.AdapterId`.
    pub adapter: crate::definition::AdapterId,
    /// `connection` — `thalamus.definition.ConnectionId`.
    pub connection: crate::definition::ConnectionId,
    /// `query_sha256` — `thalamus.observation.QueryDigest`.
    pub query_sha256: QueryDigest,
    /// `run_id` — `thalamus.observation.RunId`.
    pub run_id: RunId,
}

/// QueryDigest — `thalamus.observation.QueryDigest`: a distinct wrapper around `String`.
///
/// SHA-256 of the canonical JSON of the definition's query map (keys sorted, no whitespace), lower-case hex.
///
/// Every character is one of `0123456789abcdef`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryDigest(pub String);

/// Reading — `thalamus.observation.Reading`: one of a fixed set of shapes, tagged on the wire by `quality`.
///
/// The value and its quality as one type. quality unknown has no value and a reason; partial has a value and its bound; ok has a value. The illegal pairs cannot be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// Tagged `ok` — `thalamus.observation.CompleteReading`.
    Ok(CompleteReading),
    /// Tagged `partial` — `thalamus.observation.PartialReading`.
    Partial(PartialReading),
    /// Tagged `unknown` — `thalamus.observation.UnreadReading`.
    Unknown(UnreadReading),
}

/// RunId — `thalamus.observation.RunId`: a distinct wrapper around `String`.
///
/// Groups the samples of one sampling pass.
///
/// Every value starts with `run_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunId(pub String);

/// SampleId — `thalamus.observation.SampleId`: a distinct wrapper around `String`.
///
/// A sample id, minted by the producer.
///
/// Every value starts with `smp_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleId(pub String);

/// SeriesKey — `thalamus.observation.SeriesKey`: a distinct wrapper around `String`.
///
/// One series, a signal id together with its dims values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesKey(pub String);

/// The states of `thalamus.observation.SignalSample`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SignalSample<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalSampleState {
    /// `Recorded`.
    Recorded,
}

/// TimeWindow — `thalamus.observation.TimeWindow`.
///
/// The span a reading covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWindow {
    /// `start` — `Timestamp`.
    pub start: crate::primitives::Timestamp,
    /// `end` — `Timestamp`.
    pub end: crate::primitives::Timestamp,
}

/// UnreadReading — `thalamus.observation.UnreadReading`.
///
/// Not read. Carries no value at all, so unknown can never become 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadReading {
    /// `reason` — `String`.
    pub reason: String,
}

/// What SignalSample — `thalamus.observation.SignalSample` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SignalSample<S>`], and at a boundary by [`SignalSampleSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalSampleData {
    /// The identity: `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    ///
    /// Carries `definition`: `thalamus.observation.SignalSample` references one `thalamus.definition.SignalDefinition`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `window` — `thalamus.observation.TimeWindow`.
    pub window: TimeWindow,
    /// `reading` — `thalamus.observation.Reading`.
    pub reading: Reading,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: crate::definition::Unit,
    /// `denominator` — `Optional<thalamus.observation.Denominator>`.
    pub denominator: Option<Denominator>,
    /// `provenance` — `thalamus.observation.Provenance`.
    pub provenance: Provenance,
}

/// The states of `thalamus.observation.SignalSample`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](signal_sample_state::Marker), so [`SignalSample<S>`](SignalSample) can only ever rest in a real state.
pub mod signal_sample_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Recorded {}
    }

    /// A declared state of `SignalSample`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::SignalSampleState;
    }

    /// `Recorded`. Where a new instance starts.
    pub struct Recorded;

    impl Marker for Recorded {
        const STATE: super::SignalSampleState = super::SignalSampleState::Recorded;
    }
}

/// SignalSample — `thalamus.observation.SignalSample` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Recorded`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`SignalSampleSnapshot`]
/// and [`SignalSampleSnapshot::refine`].
pub struct SignalSample<S: signal_sample_state::Marker> {
    data: SignalSampleData,
    state: core::marker::PhantomData<S>,
}

impl<S: signal_sample_state::Marker> SignalSample<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> SignalSampleState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &SignalSampleData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> SignalSampleData {
        self.data
    }
}

impl SignalSample<signal_sample_state::Recorded> {
    /// A new instance, resting in `Recorded` — the only state the lifecycle starts one in.
    pub fn new(data: SignalSampleData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.observation.SignalSample` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`SignalSampleSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalSampleSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: SignalSampleState,
    /// What it holds.
    pub data: SignalSampleData,
}

/// An `SignalSample` in whichever declared state it was found.
pub enum AnySignalSample {
    /// Resting in `Recorded`.
    Recorded(SignalSample<signal_sample_state::Recorded>),
}

impl SignalSampleSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `SignalSampleState` cannot spell one.
    pub fn refine(self) -> AnySignalSample {
        match self.state {
            SignalSampleState::Recorded => AnySignalSample::Recorded(SignalSample {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnySignalSample {
    /// The state, as the runtime value.
    pub fn state(&self) -> SignalSampleState {
        match self {
            Self::Recorded(_) => SignalSampleState::Recorded,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> SignalSampleSnapshot {
        match self {
            Self::Recorded(instance) => SignalSampleSnapshot {
                state: SignalSampleState::Recorded,
                data: instance.into_data(),
            },
        }
    }
}

/// Record a sample — the input of `thalamus.observation.RecordSample`.
///
/// Append one reading of an active signal. Unknown is recorded as unknown, never as 0.
///
/// Everything it can result in is [`RecordSampleOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSample {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `window` — `thalamus.observation.TimeWindow`.
    pub window: TimeWindow,
    /// `reading` — `thalamus.observation.Reading`.
    pub reading: Reading,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: crate::definition::Unit,
    /// `denominator` — `Optional<thalamus.observation.Denominator>`.
    pub denominator: Option<Denominator>,
    /// `provenance` — `thalamus.observation.Provenance`.
    pub provenance: Provenance,
}

/// Actual typed response of `thalamus.observation.RecordSample`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSampleResponse {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: SampleId,
}

/// Everything `thalamus.observation.RecordSample` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordSampleOutcome {
    /// `unknown-signal` — when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names.
    ///
    /// No definition has this id; nothing was recorded.
    UnknownSignal {
        /// Why it was refused: `thalamus.observation.UnknownSignal`.
        error: UnknownSignal,
    },
    /// `not-active` — when the `thalamus.definition.SignalDefinition` that `input.signal` names satisfies `state != Active`.
    ///
    /// The definition is not Active; nothing was recorded.
    NotActive {
        /// Why it was refused: `thalamus.observation.SignalNotActive`.
        error: SignalNotActive,
    },
    /// `recorded` — otherwise.
    ///
    /// The sample is recorded and its id returned.
    Recorded {
        /// Actual response returned by this branch.
        response: RecordSampleResponse,
        /// The `thalamus.observation.SampleRecorded` this outcome publishes.
        sample_recorded: SampleRecorded,
    },
}

impl RecordSampleOutcome {
    /// Compare the actual returned response with response-mapped emitted fields.
    pub fn response_payload_matches(&self) -> bool {
        match self {
            Self::UnknownSignal { .. } => true,
            Self::NotActive { .. } => true,
            Self::Recorded { response, sample_recorded, .. } => sample_recorded.sample_id == response.sample_id.clone(),
        }
    }
}

/// SampleRecorded — the event `thalamus.observation.SampleRecorded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleRecorded {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: SeriesKey,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
}

/// The declared error `thalamus.observation.SignalNotActive`.
///
/// Only an Active definition is sampled; this one is Proposed, Muted or Retired, so nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalNotActive {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
}

/// The declared error `thalamus.observation.UnknownSignal`.
///
/// No definition has this signal id; nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSignal {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
}

/// Samples — one row of the view `thalamus.observation.Samples`.
///
/// Projects `thalamus.observation.SignalSample` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Samples {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: SampleId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: SeriesKey,
    /// `dims` — `Map<String, thalamus.dimension.DimensionValue>`.
    pub dims: std::collections::BTreeMap<String, crate::dimension::DimensionValue>,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `window` — `thalamus.observation.TimeWindow`.
    pub window: TimeWindow,
    /// `reading` — `thalamus.observation.Reading`.
    pub reading: Reading,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: crate::definition::Unit,
    /// `denominator` — `Optional<thalamus.observation.Denominator>`.
    pub denominator: Option<Denominator>,
    /// `provenance` — `thalamus.observation.Provenance`.
    pub provenance: Provenance,
    /// `state` — `thalamus.observation.SignalSample.State`.
    pub state: SignalSampleState,
}

/// Samples of a series — one row of the view `thalamus.observation.SeriesSamples`.
///
/// Projects `thalamus.observation.SignalSample` at `read_your_writes` consistency, containing instances where `series == param.series`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesSamples {
    /// `sample_id` — `thalamus.observation.SampleId`.
    pub sample_id: SampleId,
    /// `ts` — `Timestamp`.
    pub ts: crate::primitives::Timestamp,
    /// `window` — `thalamus.observation.TimeWindow`.
    pub window: TimeWindow,
    /// `reading` — `thalamus.observation.Reading`.
    pub reading: Reading,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: crate::definition::Unit,
    /// `denominator` — `Optional<thalamus.observation.Denominator>`.
    pub denominator: Option<Denominator>,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.observation.RecordSample` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `thalamus.observation.RecordSample` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `unknown-signal` when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names, error `thalamus.observation.UnknownSignal`; `not-active` when the `thalamus.definition.SignalDefinition` that `input.signal` names satisfies `state != Active`, error `thalamus.observation.SignalNotActive`; `recorded` otherwise, creates `thalamus.observation.SignalSample`, emits `thalamus.observation.SampleRecorded`.
    pub trait RecordSampleBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.observation.RecordSample`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn record_sample(&mut self, input: super::RecordSample) -> Result<super::RecordSampleOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.observation.Samples` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait SamplesQuery {
        /// Serves `thalamus.observation.Samples` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn samples(&self) -> Result<Vec<super::Samples>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.observation.SeriesSamples` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a view parameter (`params:`), which a generated query does not apply.
    ///
    /// Contract: a query answering `thalamus.observation.SeriesSamples` with rows projected from `thalamus.observation.SignalSample` at `read_your_writes` consistency, containing instances where `series == param.series`.
    pub trait SeriesSamplesQuery {
        /// Serves `thalamus.observation.SeriesSamples` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn series_samples(&self, series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesSamples>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl RecordSampleBehavior for Unimplemented {
        fn record_sample(&mut self, _input: super::RecordSample) -> Result<super::RecordSampleOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.observation.RecordSample" })
        }
    }

    impl SeriesSamplesQuery for Unimplemented {
        fn series_samples(&self, _series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesSamples>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "thalamus.observation.SeriesSamples" })
        }
    }
}
