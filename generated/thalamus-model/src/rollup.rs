// generated from thalamus v1
// model digest 73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed
// contract digest 093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061
// do not edit: regenerate with `ess synthesize --layout crate`

//! Daily rollups — `thalamus.rollup`.
//!
//! One row per series per UTC day, kept after the raw samples leave retention. It is what a source that cannot be re-queried (history store) leaves behind.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// The states of `thalamus.rollup.DailyRollup`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `DailyRollup<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DailyRollupState {
    /// `Recorded`.
    Recorded,
}

/// RollupId — `thalamus.rollup.RollupId`: a distinct wrapper around `String`.
///
/// A rollup id, spelled from the series key and the UTC day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollupId(pub String);

/// VerdictCounts — `thalamus.rollup.VerdictCounts`.
///
/// One count per verdict of the ladder, over the day's evaluations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictCounts {
    /// `ok` — `Integer`.
    pub ok: i64,
    /// `na` — `Integer`.
    pub na: i64,
    /// `warn` — `Integer`.
    pub warn: i64,
    /// `crit` — `Integer`.
    pub crit: i64,
    /// `unknown` — `Integer`.
    pub unknown: i64,
}

/// What DailyRollup — `thalamus.rollup.DailyRollup` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`DailyRollup<S>`], and at a boundary by [`DailyRollupSnapshot::state`].
///
/// Every value satisfies `unknown <= n` — checked by [`DailyRollupData::broken_invariant`].
/// Every value satisfies `partial <= n` — checked by [`DailyRollupData::broken_invariant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyRollupData {
    /// The identity: `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
    /// `signal` — `thalamus.definition.SignalId`.
    ///
    /// Carries `definition`: `thalamus.rollup.DailyRollup` references one `thalamus.definition.SignalDefinition`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `day` — `Timestamp`.
    pub day: crate::primitives::Timestamp,
    /// `n` — `Integer`.
    pub n: i64,
    /// `unknown` — `Integer`.
    pub unknown: i64,
    /// `partial` — `Integer`.
    pub partial: i64,
    /// `min` — `Optional<Decimal>`.
    pub min: Option<crate::primitives::Decimal>,
    /// `p50` — `Optional<Decimal>`.
    pub p50: Option<crate::primitives::Decimal>,
    /// `max` — `Optional<Decimal>`.
    pub max: Option<crate::primitives::Decimal>,
    /// `last` — `Optional<Decimal>`.
    pub last: Option<crate::primitives::Decimal>,
    /// `worst_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub worst_verdict: Option<crate::evaluation::Verdict>,
    /// `verdict_counts` — `thalamus.rollup.VerdictCounts`.
    pub verdict_counts: VerdictCounts,
    /// `min_score` — `Optional<Decimal>`.
    pub min_score: Option<crate::primitives::Decimal>,
    /// `max_score` — `Optional<Decimal>`.
    pub max_score: Option<crate::primitives::Decimal>,
}

impl DailyRollupData {
    /// The first declared invariant of `thalamus.rollup.DailyRollup` this value breaks, as the specification declares it,
    /// or `None` when it breaks none.
    ///
    /// An invariant is broken only when it is false of this value. One that reads something
    /// absent — an empty `Optional`, a list position past the end, or `state`, which this
    /// type does not hold — decides nothing, as the conformance interpreter reads it.
    pub fn broken_invariant(&self) -> Option<&'static str> {
        use crate::primitives::invariant as iv;
        if iv::broken(iv::compare(Some(iv::Fact::integer(self.unknown)), iv::Op::Le, Some(iv::Fact::integer(self.n)), false, true)) {
            return Some("unknown <= n");
        }
        if iv::broken(iv::compare(Some(iv::Fact::integer(self.partial)), iv::Op::Le, Some(iv::Fact::integer(self.n)), false, true)) {
            return Some("partial <= n");
        }
        None
    }
}

/// The states of `thalamus.rollup.DailyRollup`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](daily_rollup_state::Marker), so [`DailyRollup<S>`](DailyRollup) can only ever rest in a real state.
pub mod daily_rollup_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Recorded {}
    }

    /// A declared state of `DailyRollup`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::DailyRollupState;
    }

    /// `Recorded`. Where a new instance starts.
    pub struct Recorded;

    impl Marker for Recorded {
        const STATE: super::DailyRollupState = super::DailyRollupState::Recorded;
    }
}

/// DailyRollup — `thalamus.rollup.DailyRollup` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Recorded`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`DailyRollupSnapshot`]
/// and [`DailyRollupSnapshot::refine`].
pub struct DailyRollup<S: daily_rollup_state::Marker> {
    data: DailyRollupData,
    state: core::marker::PhantomData<S>,
}

impl<S: daily_rollup_state::Marker> DailyRollup<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> DailyRollupState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &DailyRollupData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> DailyRollupData {
        self.data
    }
}

impl DailyRollup<daily_rollup_state::Recorded> {
    /// A new instance, resting in `Recorded` — the only state the lifecycle starts one in.
    pub fn new(data: DailyRollupData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.rollup.DailyRollup` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`DailyRollupSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyRollupSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: DailyRollupState,
    /// What it holds.
    pub data: DailyRollupData,
}

/// An `DailyRollup` in whichever declared state it was found.
pub enum AnyDailyRollup {
    /// Resting in `Recorded`.
    Recorded(DailyRollup<daily_rollup_state::Recorded>),
}

impl DailyRollupSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `DailyRollupState` cannot spell one.
    pub fn refine(self) -> AnyDailyRollup {
        match self.state {
            DailyRollupState::Recorded => AnyDailyRollup::Recorded(DailyRollup {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyDailyRollup {
    /// The state, as the runtime value.
    pub fn state(&self) -> DailyRollupState {
        match self {
            Self::Recorded(_) => DailyRollupState::Recorded,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> DailyRollupSnapshot {
        match self {
            Self::Recorded(instance) => DailyRollupSnapshot {
                state: DailyRollupState::Recorded,
                data: instance.into_data(),
            },
        }
    }
}

/// Record a daily rollup — the input of `thalamus.rollup.RecordRollup`.
///
/// Append the one row of a series for a closed UTC day.
///
/// Everything it can result in is [`RecordRollupOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordRollup {
    /// `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `day` — `Timestamp`.
    pub day: crate::primitives::Timestamp,
    /// `n` — `Integer`.
    pub n: i64,
    /// `unknown` — `Integer`.
    pub unknown: i64,
    /// `partial` — `Integer`.
    pub partial: i64,
    /// `min` — `Optional<Decimal>`.
    pub min: Option<crate::primitives::Decimal>,
    /// `p50` — `Optional<Decimal>`.
    pub p50: Option<crate::primitives::Decimal>,
    /// `max` — `Optional<Decimal>`.
    pub max: Option<crate::primitives::Decimal>,
    /// `last` — `Optional<Decimal>`.
    pub last: Option<crate::primitives::Decimal>,
    /// `worst_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub worst_verdict: Option<crate::evaluation::Verdict>,
    /// `verdict_counts` — `thalamus.rollup.VerdictCounts`.
    pub verdict_counts: VerdictCounts,
    /// `min_score` — `Optional<Decimal>`.
    pub min_score: Option<crate::primitives::Decimal>,
    /// `max_score` — `Optional<Decimal>`.
    pub max_score: Option<crate::primitives::Decimal>,
}

/// Everything `thalamus.rollup.RecordRollup` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordRollupOutcome {
    /// `unknown-signal` — when no `thalamus.definition.SignalDefinition` carries the identity `input.signal` names.
    ///
    /// No definition has this id; nothing was recorded.
    UnknownSignal {
        /// Why it was refused: `thalamus.rollup.UnknownSignal`.
        error: UnknownSignal,
    },
    /// `recorded` — otherwise.
    ///
    /// The day's row is recorded.
    Recorded {
        /// The `thalamus.rollup.RollupRecorded` this outcome publishes.
        rollup_recorded: RollupRecorded,
    },
    /// `already-rolled-up` — for an identity a record already carries.
    ///
    /// The series already has a row for this day; nothing changed.
    AlreadyRolledUp {
        /// Why it was refused: `thalamus.rollup.RollupExists`.
        error: RollupExists,
    },
}

/// RollupRecorded — the event `thalamus.rollup.RollupRecorded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollupRecorded {
    /// `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `day` — `Timestamp`.
    pub day: crate::primitives::Timestamp,
}

/// The declared error `thalamus.rollup.RollupExists`.
///
/// This series is already rolled up for this day; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollupExists {
    /// `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
}

/// The declared error `thalamus.rollup.UnknownSignal`.
///
/// No definition has this signal id; nothing was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSignal {
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
}

/// Rollups — one row of the view `thalamus.rollup.Rollups`.
///
/// Projects `thalamus.rollup.DailyRollup` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollups {
    /// `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `series` — `thalamus.observation.SeriesKey`.
    pub series: crate::observation::SeriesKey,
    /// `day` — `Timestamp`.
    pub day: crate::primitives::Timestamp,
    /// `n` — `Integer`.
    pub n: i64,
    /// `state` — `thalamus.rollup.DailyRollup.State`.
    pub state: DailyRollupState,
}

/// Rollups of a series — one row of the view `thalamus.rollup.SeriesRollups`.
///
/// Projects `thalamus.rollup.DailyRollup` at `read_your_writes` consistency, containing instances where `series == param.series`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesRollups {
    /// `rollup_id` — `thalamus.rollup.RollupId`.
    pub rollup_id: RollupId,
    /// `signal` — `thalamus.definition.SignalId`.
    pub signal: crate::definition::SignalId,
    /// `day` — `Timestamp`.
    pub day: crate::primitives::Timestamp,
    /// `n` — `Integer`.
    pub n: i64,
    /// `unknown` — `Integer`.
    pub unknown: i64,
    /// `partial` — `Integer`.
    pub partial: i64,
    /// `min` — `Optional<Decimal>`.
    pub min: Option<crate::primitives::Decimal>,
    /// `p50` — `Optional<Decimal>`.
    pub p50: Option<crate::primitives::Decimal>,
    /// `max` — `Optional<Decimal>`.
    pub max: Option<crate::primitives::Decimal>,
    /// `last` — `Optional<Decimal>`.
    pub last: Option<crate::primitives::Decimal>,
    /// `worst_verdict` — `Optional<thalamus.evaluation.Verdict>`.
    pub worst_verdict: Option<crate::evaluation::Verdict>,
    /// `verdict_counts` — `thalamus.rollup.VerdictCounts`.
    pub verdict_counts: VerdictCounts,
    /// `min_score` — `Optional<Decimal>`.
    pub min_score: Option<crate::primitives::Decimal>,
    /// `max_score` — `Optional<Decimal>`.
    pub max_score: Option<crate::primitives::Decimal>,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.rollup.RecordRollup` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RecordRollupBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.rollup.RecordRollup`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn record_rollup(&mut self, input: super::RecordRollup) -> Result<super::RecordRollupOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.rollup.Rollups` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait RollupsQuery {
        /// Serves `thalamus.rollup.Rollups` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn rollups(&self) -> Result<Vec<super::Rollups>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.rollup.SeriesRollups` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a view parameter (`params:`), which a generated query does not apply.
    ///
    /// Contract: a query answering `thalamus.rollup.SeriesRollups` with rows projected from `thalamus.rollup.DailyRollup` at `read_your_writes` consistency, containing instances where `series == param.series`.
    pub trait SeriesRollupsQuery {
        /// Serves `thalamus.rollup.SeriesRollups` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn series_rollups(&self, series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesRollups>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl SeriesRollupsQuery for Unimplemented {
        fn series_rollups(&self, _series: crate::observation::SeriesKey) -> Result<Vec<super::SeriesRollups>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "thalamus.rollup.SeriesRollups" })
        }
    }
}
