// generated from thalamus v1
// model digest 99b2e1b31aebb3cdde7af7bb46be2ec80e4af2ccd183cd598985f3a30326c48f
// contract digest 82328f7c71509b62eb250d7912f8913dadad64f2cf9aae2897c422dfba7f784c
// do not edit: regenerate with `ess synthesize --layout crate`

//! Signal definitions — `thalamus.definition`.
//!
//! What is measured and how. One copy of every threshold lives here and nowhere else.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// AdapterId — `thalamus.definition.AdapterId`: a distinct wrapper around `String`.
///
/// The source adapter that turns a connection operation into samples (sql, a metrics query, an orchestrator, version control, an HTTP probe, test-run events).
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterId(pub String);

/// Author — `thalamus.definition.Author`: a distinct wrapper around `String`.
///
/// Who made a revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author(pub String);

/// BaselineKind — `thalamus.definition.BaselineKind`: one of a closed set of names.
///
/// A baseline a sample is graded against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineKind {
    /// `offset_1d`.
    Offset1d,
    /// `offset_7d`.
    Offset7d,
    /// `pooled`.
    Pooled,
    /// `higher_of`.
    HigherOf,
    /// `lower_of`.
    LowerOf,
    /// `same_slot_median_4w`.
    SameSlotMedian4w,
    /// `rolling_median`.
    RollingMedian,
    /// `fixed`.
    Fixed,
}

/// ConnectionId — `thalamus.definition.ConnectionId`: a distinct wrapper around `String`.
///
/// A Connectors connection id. Thalamus holds no secret; the connection holds the credential.
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.:/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionId(pub String);

/// DenominatorSpec — `thalamus.definition.DenominatorSpec`.
///
/// The population a ratio or count is taken over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenominatorSpec {
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: Unit,
    /// `min_n` — `Integer`.
    pub min_n: i64,
    /// `since` — `String`.
    pub since: String,
}

/// EvaluationSpec — `thalamus.definition.EvaluationSpec`.
///
/// How a sample is graded. The only copy of the signal's thresholds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationSpec {
    /// `rule` — `thalamus.definition.RuleId`.
    pub rule: RuleId,
    /// `baselines` — `List<thalamus.definition.BaselineKind>`.
    pub baselines: Vec<BaselineKind>,
    /// `warn` — `Optional<thalamus.definition.Threshold>`.
    pub warn: Option<Threshold>,
    /// `crit` — `Optional<thalamus.definition.Threshold>`.
    pub crit: Option<Threshold>,
    /// `min_trend_window` — `Optional<Duration>`.
    pub min_trend_window: Option<crate::primitives::Duration>,
    /// `fixed` — `Optional<Decimal>`.
    pub fixed: Option<crate::primitives::Decimal>,
    /// `rolling_n` — `Optional<Integer>`.
    pub rolling_n: Option<i64>,
    /// `mapping` — `Optional<Map<String, thalamus.evaluation.Verdict>>`.
    pub mapping: Option<std::collections::BTreeMap<String, crate::evaluation::Verdict>>,
}

/// ExtractorId — `thalamus.definition.ExtractorId`: a distinct wrapper around `String`.
///
/// A built-in extractor (sql.ratio) or a plugin id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractorId(pub String);

/// Family — `thalamus.definition.Family`: a distinct wrapper around `String`.
///
/// An instance-defined grouping of signals.
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family(pub String);

/// History — `thalamus.definition.History`: one of a closed set of names.
///
/// source means the source can be re-queried at an offset; store means only thalamus remembers the series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    /// `source`.
    Source,
    /// `store`.
    Store,
}

/// Links — `thalamus.definition.Links`.
///
/// What the signal is evidence about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Links {
    /// `components` — `List<String>`.
    pub components: Vec<String>,
    /// `issues` — `List<String>`.
    pub issues: Vec<String>,
}

/// Owner — `thalamus.definition.Owner`: a distinct wrapper around `String`.
///
/// Who answers for the signal (team:<name>).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner(pub String);

/// Persistence — `thalamus.definition.Persistence`.
///
/// How many evaluations must agree before a verdict change is a transition (design § 3.2, § 11 D15). Absent on a definition, the service applies k 2, k_crit 1, m 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persistence {
    /// `k` — `Integer`.
    pub k: i64,
    /// `k_crit` — `Integer`.
    pub k_crit: i64,
    /// `m` — `Integer`.
    pub m: i64,
}

/// QueryText — `thalamus.definition.QueryText`: a distinct wrapper around `String`.
///
/// Query text, stored once and hashed into every sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryText(pub String);

/// RuleId — `thalamus.definition.RuleId`: a distinct wrapper around `String`.
///
/// The grading rule (ratio_vs_pooled_baseline).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleId(pub String);

/// The states of `thalamus.definition.SignalDefinition`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `SignalDefinition<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalDefinitionState {
    /// `Active`.
    Active,
    /// `Muted`.
    Muted,
    /// `Proposed`.
    Proposed,
    /// `Retired`.
    Retired,
}

/// SignalId — `thalamus.definition.SignalId`: a distinct wrapper around `String`.
///
/// Stable signal id, spelled <family>.<layer>.<name> in lower case.
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalId(pub String);

/// SignalKind — `thalamus.definition.SignalKind`: one of a closed set of names.
///
/// How a reading of the signal is formed. An instant signal gets no trend below its minimum trend window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    /// `gauge`.
    Gauge,
    /// `counter_increase`.
    CounterIncrease,
    /// `ratio`.
    Ratio,
    /// `instant`.
    Instant,
    /// `count_of_cases`.
    CountOfCases,
}

/// Source — `thalamus.definition.Source`.
///
/// Where a signal is read; a Connectors connection, never a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// `adapter` — `thalamus.definition.AdapterId`.
    pub adapter: AdapterId,
    /// `connection` — `thalamus.definition.ConnectionId`.
    pub connection: ConnectionId,
}

/// Threshold — `thalamus.definition.Threshold`.
///
/// One line (WARN or CRIT) of a grading rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Threshold {
    /// `x` — `Optional<Decimal>`.
    pub x: Option<crate::primitives::Decimal>,
    /// `abs` — `Optional<Decimal>`.
    pub abs: Option<crate::primitives::Decimal>,
    /// `min_events` — `Optional<Integer>`.
    pub min_events: Option<i64>,
}

/// Unit — `thalamus.definition.Unit`: a distinct wrapper around `String`.
///
/// A UCUM unit, case-sensitive form (1, s, By, {request}, {pod}). A ratio has unit 1.
///
/// Every character is one of `!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit(pub String);

/// What SignalDefinition — `thalamus.definition.SignalDefinition` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`SignalDefinition<S>`], and at a boundary by [`SignalDefinitionSnapshot::state`].
///
/// Every value satisfies `rev >= 1` — checked by [`SignalDefinitionData::broken_invariant`].
/// Every value satisfies `(state == Proposed or state == Retired or (defined(unit) and defined(dimensions) and defined(denominator) and defined(owner)))` — checked by [`SignalDefinitionData::broken_invariant`].
/// Every value satisfies `((state == Muted and defined(mute_reason) and defined(mute_until)) or (state != Muted and not (defined(mute_reason)) and not (defined(mute_until))))` — checked by [`SignalDefinitionData::broken_invariant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDefinitionData {
    /// The identity: `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `title` — `String`.
    pub title: String,
    /// `family` — `thalamus.definition.Family`.
    pub family: Family,
    /// `kind` — `thalamus.definition.SignalKind`.
    pub kind: SignalKind,
    /// `unit` — `Optional<thalamus.definition.Unit>`.
    pub unit: Option<Unit>,
    /// `dimensions` — `Optional<List<thalamus.dimension.DimensionKey>>`.
    pub dimensions: Option<Vec<crate::dimension::DimensionKey>>,
    /// `source` — `thalamus.definition.Source`.
    pub source: Source,
    /// `extractor` — `thalamus.definition.ExtractorId`.
    pub extractor: ExtractorId,
    /// `query` — `Map<String, thalamus.definition.QueryText>`.
    pub query: std::collections::BTreeMap<String, QueryText>,
    /// `window` — `Duration`.
    pub window: crate::primitives::Duration,
    /// `cadence` — `Duration`.
    pub cadence: crate::primitives::Duration,
    /// `history` — `thalamus.definition.History`.
    pub history: History,
    /// `denominator` — `Optional<thalamus.definition.DenominatorSpec>`.
    pub denominator: Option<DenominatorSpec>,
    /// `evaluation` — `thalamus.definition.EvaluationSpec`.
    pub evaluation: EvaluationSpec,
    /// `persistence` — `Optional<thalamus.definition.Persistence>`.
    pub persistence: Option<Persistence>,
    /// `objective` — `Optional<Json>`.
    pub objective: Option<crate::json::Value>,
    /// `owner` — `Optional<thalamus.definition.Owner>`.
    pub owner: Option<Owner>,
    /// `links` — `thalamus.definition.Links`.
    pub links: Links,
    /// `rev` — `Integer`.
    pub rev: i64,
    /// `revised_at` — `Timestamp`.
    pub revised_at: crate::primitives::Timestamp,
    /// `revised_by` — `thalamus.definition.Author`.
    pub revised_by: Author,
    /// `mute_reason` — `Optional<String>`.
    pub mute_reason: Option<String>,
    /// `mute_until` — `Optional<Timestamp>`.
    pub mute_until: Option<crate::primitives::Timestamp>,
}

impl SignalDefinitionData {
    /// The first declared invariant of `thalamus.definition.SignalDefinition` this value breaks, as the specification declares it,
    /// or `None` when it breaks none.
    ///
    /// An invariant is broken only when it is false of this value. One that reads something
    /// absent — an empty `Optional`, a list position past the end, or `state`, which this
    /// type does not hold — decides nothing, as the conformance interpreter reads it.
    pub fn broken_invariant(&self) -> Option<&'static str> {
        use crate::primitives::invariant as iv;
        if iv::broken(iv::compare(Some(iv::Fact::integer(self.rev)), iv::Op::Ge, iv::Fact::number("1"), false, true)) {
            return Some("rev >= 1");
        }
        if iv::broken(iv::any([iv::compare(None, iv::Op::Eq, Some(iv::Fact::text("Proposed")), false, true), iv::compare(None, iv::Op::Eq, Some(iv::Fact::text("Retired")), false, true), iv::all([Some(self.unit.as_ref().map(|v| &v.0).is_some()), Some(self.dimensions.as_ref().is_some()), Some(self.denominator.as_ref().is_some()), Some(self.owner.as_ref().map(|v| &v.0).is_some())])])) {
            return Some("(state == Proposed or state == Retired or (defined(unit) and defined(dimensions) and defined(denominator) and defined(owner)))");
        }
        if iv::broken(iv::any([iv::all([iv::compare(None, iv::Op::Eq, Some(iv::Fact::text("Muted")), false, true), Some(self.mute_reason.as_ref().is_some()), Some(self.mute_until.as_ref().is_some())]), iv::all([iv::compare(None, iv::Op::Ne, Some(iv::Fact::text("Muted")), false, true), iv::not(Some(self.mute_reason.as_ref().is_some())), iv::not(Some(self.mute_until.as_ref().is_some()))])])) {
            return Some("((state == Muted and defined(mute_reason) and defined(mute_until)) or (state != Muted and not (defined(mute_reason)) and not (defined(mute_until))))");
        }
        None
    }
}

/// The states of `thalamus.definition.SignalDefinition`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](signal_definition_state::Marker), so [`SignalDefinition<S>`](SignalDefinition) can only ever rest in a real state.
pub mod signal_definition_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Active {}
        impl Sealed for super::Muted {}
        impl Sealed for super::Proposed {}
        impl Sealed for super::Retired {}
    }

    /// A declared state of `SignalDefinition`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::SignalDefinitionState;
    }

    /// `Active`.
    pub struct Active;

    impl Marker for Active {
        const STATE: super::SignalDefinitionState = super::SignalDefinitionState::Active;
    }

    /// `Muted`.
    pub struct Muted;

    impl Marker for Muted {
        const STATE: super::SignalDefinitionState = super::SignalDefinitionState::Muted;
    }

    /// `Proposed`. Where a new instance starts.
    pub struct Proposed;

    impl Marker for Proposed {
        const STATE: super::SignalDefinitionState = super::SignalDefinitionState::Proposed;
    }

    /// `Retired`. Terminal: an instance may rest here forever.
    pub struct Retired;

    impl Marker for Retired {
        const STATE: super::SignalDefinitionState = super::SignalDefinitionState::Retired;
    }
}

/// SignalDefinition — `thalamus.definition.SignalDefinition` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Proposed`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`SignalDefinitionSnapshot`]
/// and [`SignalDefinitionSnapshot::refine`].
pub struct SignalDefinition<S: signal_definition_state::Marker> {
    data: SignalDefinitionData,
    state: core::marker::PhantomData<S>,
}

impl<S: signal_definition_state::Marker> SignalDefinition<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> SignalDefinitionState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &SignalDefinitionData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> SignalDefinitionData {
        self.data
    }
}

impl SignalDefinition<signal_definition_state::Proposed> {
    /// A new instance, resting in `Proposed` — the only state the lifecycle starts one in.
    pub fn new(data: SignalDefinitionData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl SignalDefinition<signal_definition_state::Active> {
    /// `mute` — `Active` → `Muted`. Taken by the `muted` outcome of `thalamus.definition.MuteDefinition`.
    pub fn mute(self) -> SignalDefinition<signal_definition_state::Muted> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `retire` — `Active` → `Retired`. Taken by the `retired` outcome of `thalamus.definition.RetireDefinition`.
    pub fn retire(self) -> SignalDefinition<signal_definition_state::Retired> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl SignalDefinition<signal_definition_state::Muted> {
    /// `unmute` — `Muted` → `Active`. Taken by the `unmuted` outcome of `thalamus.definition.UnmuteDefinition`.
    pub fn unmute(self) -> SignalDefinition<signal_definition_state::Active> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `retire` — `Muted` → `Retired`. Taken by the `retired` outcome of `thalamus.definition.RetireDefinition`.
    pub fn retire(self) -> SignalDefinition<signal_definition_state::Retired> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl SignalDefinition<signal_definition_state::Proposed> {
    /// `activate` — `Proposed` → `Active`. Taken by the `activated` outcome of `thalamus.definition.ActivateDefinition`.
    pub fn activate(self) -> SignalDefinition<signal_definition_state::Active> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `retire` — `Proposed` → `Retired`. Taken by the `retired` outcome of `thalamus.definition.RetireDefinition`.
    pub fn retire(self) -> SignalDefinition<signal_definition_state::Retired> {
        SignalDefinition {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.definition.SignalDefinition` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`SignalDefinitionSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDefinitionSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: SignalDefinitionState,
    /// What it holds.
    pub data: SignalDefinitionData,
}

/// An `SignalDefinition` in whichever declared state it was found.
pub enum AnySignalDefinition {
    /// Resting in `Active`.
    Active(SignalDefinition<signal_definition_state::Active>),
    /// Resting in `Muted`.
    Muted(SignalDefinition<signal_definition_state::Muted>),
    /// Resting in `Proposed`.
    Proposed(SignalDefinition<signal_definition_state::Proposed>),
    /// Resting in `Retired`.
    Retired(SignalDefinition<signal_definition_state::Retired>),
}

impl SignalDefinitionSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `SignalDefinitionState` cannot spell one.
    pub fn refine(self) -> AnySignalDefinition {
        match self.state {
            SignalDefinitionState::Active => AnySignalDefinition::Active(SignalDefinition {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            SignalDefinitionState::Muted => AnySignalDefinition::Muted(SignalDefinition {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            SignalDefinitionState::Proposed => AnySignalDefinition::Proposed(SignalDefinition {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            SignalDefinitionState::Retired => AnySignalDefinition::Retired(SignalDefinition {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnySignalDefinition {
    /// The state, as the runtime value.
    pub fn state(&self) -> SignalDefinitionState {
        match self {
            Self::Active(_) => SignalDefinitionState::Active,
            Self::Muted(_) => SignalDefinitionState::Muted,
            Self::Proposed(_) => SignalDefinitionState::Proposed,
            Self::Retired(_) => SignalDefinitionState::Retired,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> SignalDefinitionSnapshot {
        match self {
            Self::Active(instance) => SignalDefinitionSnapshot {
                state: SignalDefinitionState::Active,
                data: instance.into_data(),
            },
            Self::Muted(instance) => SignalDefinitionSnapshot {
                state: SignalDefinitionState::Muted,
                data: instance.into_data(),
            },
            Self::Proposed(instance) => SignalDefinitionSnapshot {
                state: SignalDefinitionState::Proposed,
                data: instance.into_data(),
            },
            Self::Retired(instance) => SignalDefinitionSnapshot {
                state: SignalDefinitionState::Retired,
                data: instance.into_data(),
            },
        }
    }
}

/// Activate a definition — the input of `thalamus.definition.ActivateDefinition`.
///
/// Start sampling a proposed definition.
///
/// Everything it can result in is [`ActivateDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// Everything `thalamus.definition.ActivateDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivateDefinitionOutcome {
    /// `incomplete` — when the existing subject's stored fields satisfy `(not (defined(unit)) or not (defined(dimensions)) or not (defined(denominator)) or not (defined(owner)))`.
    ///
    /// Unit, dimensions, denominator or owner is missing; the definition stays Proposed.
    Incomplete {
        /// Why it was refused: `thalamus.definition.IncompleteDefinition`.
        error: IncompleteDefinition,
    },
    /// `activated` — otherwise.
    ///
    /// The definition is Active; the scheduler samples it on its cadence.
    Activated {
        /// The `thalamus.definition.DefinitionActivated` this outcome publishes.
        definition_activated: DefinitionActivated,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The definition is not Proposed; nothing changed.
    WrongState {
        /// Why it was refused: `thalamus.definition.DefinitionStateConflict`.
        error: DefinitionStateConflict,
    },
    /// `no-such-definition` — for an identity no record carries.
    ///
    /// No definition has this id; nothing changed.
    NoSuchDefinition {
        /// Why it was refused: `thalamus.definition.UnknownDefinition`.
        error: UnknownDefinition,
    },
}

/// Mute a definition — the input of `thalamus.definition.MuteDefinition`.
///
/// Stop grading an active definition until an expiry, for a stated reason.
///
/// Everything it can result in is [`MuteDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuteDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `reason` — `String`.
    pub reason: String,
    /// `until` — `Timestamp`.
    pub until: crate::primitives::Timestamp,
}

/// Everything `thalamus.definition.MuteDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MuteDefinitionOutcome {
    /// `expiry-passed` — when `until <= now`.
    ///
    /// The expiry is not in the future; nothing was muted.
    ExpiryPassed {
        /// Why it was refused: `thalamus.definition.MuteExpiryPassed`.
        error: MuteExpiryPassed,
    },
    /// `muted` — otherwise.
    ///
    /// The definition is Muted and carries its reason and expiry.
    Muted {
        /// The `thalamus.definition.DefinitionMuted` this outcome publishes.
        definition_muted: DefinitionMuted,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The definition is not Active; nothing changed.
    WrongState {
        /// Why it was refused: `thalamus.definition.DefinitionStateConflict`.
        error: DefinitionStateConflict,
    },
    /// `no-such-definition` — for an identity no record carries.
    ///
    /// No definition has this id; nothing changed.
    NoSuchDefinition {
        /// Why it was refused: `thalamus.definition.UnknownDefinition`.
        error: UnknownDefinition,
    },
}

/// Propose a definition — the input of `thalamus.definition.ProposeDefinition`.
///
/// Add a signal definition at revision 1, not yet sampled.
///
/// Everything it can result in is [`ProposeDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposeDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `title` — `String`.
    pub title: String,
    /// `family` — `thalamus.definition.Family`.
    pub family: Family,
    /// `kind` — `thalamus.definition.SignalKind`.
    pub kind: SignalKind,
    /// `unit` — `Optional<thalamus.definition.Unit>`.
    pub unit: Option<Unit>,
    /// `dimensions` — `Optional<List<thalamus.dimension.DimensionKey>>`.
    pub dimensions: Option<Vec<crate::dimension::DimensionKey>>,
    /// `source` — `thalamus.definition.Source`.
    pub source: Source,
    /// `extractor` — `thalamus.definition.ExtractorId`.
    pub extractor: ExtractorId,
    /// `query` — `Map<String, thalamus.definition.QueryText>`.
    pub query: std::collections::BTreeMap<String, QueryText>,
    /// `window` — `Duration`.
    pub window: crate::primitives::Duration,
    /// `cadence` — `Duration`.
    pub cadence: crate::primitives::Duration,
    /// `history` — `thalamus.definition.History`.
    pub history: History,
    /// `denominator` — `Optional<thalamus.definition.DenominatorSpec>`.
    pub denominator: Option<DenominatorSpec>,
    /// `evaluation` — `thalamus.definition.EvaluationSpec`.
    pub evaluation: EvaluationSpec,
    /// `persistence` — `Optional<thalamus.definition.Persistence>`.
    pub persistence: Option<Persistence>,
    /// `objective` — `Optional<Json>`.
    pub objective: Option<crate::json::Value>,
    /// `owner` — `Optional<thalamus.definition.Owner>`.
    pub owner: Option<Owner>,
    /// `links` — `thalamus.definition.Links`.
    pub links: Links,
    /// `author` — `thalamus.definition.Author`.
    pub author: Author,
}

/// Everything `thalamus.definition.ProposeDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposeDefinitionOutcome {
    /// `instant-not-fixed` — when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`.
    ///
    /// An instant signal declared a baseline other than fixed; nothing was proposed.
    InstantNotFixed {
        /// Why it was refused: `thalamus.definition.InstantNeedsFixedBaseline`.
        error: InstantNeedsFixedBaseline,
    },
    /// `rolling-n-missing` — when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`.
    ///
    /// A rolling_median baseline was declared without evaluation.rolling_n; nothing was proposed.
    RollingNMissing {
        /// Why it was refused: `thalamus.definition.RollingWindowMissing`.
        error: RollingWindowMissing,
    },
    /// `rolling-n-unused` — when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`.
    ///
    /// evaluation.rolling_n was declared without a rolling_median baseline to read it; nothing was proposed.
    RollingNUnused {
        /// Why it was refused: `thalamus.definition.RollingWindowUnused`.
        error: RollingWindowUnused,
    },
    /// `fixed-number-missing` — when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`.
    ///
    /// A fixed baseline was declared without evaluation.fixed; nothing was proposed.
    FixedNumberMissing {
        /// Why it was refused: `thalamus.definition.FixedNumberMissing`.
        error: FixedNumberMissing,
    },
    /// `fixed-number-unused` — when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`.
    ///
    /// evaluation.fixed was declared without a fixed baseline to read it; nothing was proposed.
    FixedNumberUnused {
        /// Why it was refused: `thalamus.definition.FixedNumberUnused`.
        error: FixedNumberUnused,
    },
    /// `persistence-below-one` — when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`.
    ///
    /// persistence k, k_crit or m is below 1, settling or clearing on no evaluation; nothing was proposed.
    PersistenceBelowOne {
        /// Why it was refused: `thalamus.definition.PersistenceTooSmall`.
        error: PersistenceTooSmall,
    },
    /// `mapping-missing` — when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`.
    ///
    /// The rule producer_mapped was named without evaluation.mapping; nothing was proposed.
    MappingMissing {
        /// Why it was refused: `thalamus.definition.MappingMissing`.
        error: MappingMissing,
    },
    /// `mapping-unused` — when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`.
    ///
    /// evaluation.mapping was declared under a rule other than producer_mapped; nothing was proposed.
    MappingUnused {
        /// Why it was refused: `thalamus.definition.MappingUnused`.
        error: MappingUnused,
    },
    /// `thresholds-unused` — when `((evaluation.rule == producer_mapped or evaluation.rule == fixed_warn_only) and (defined(evaluation.warn) or defined(evaluation.crit)))`.
    ///
    /// A warn or crit line was declared under producer_mapped or fixed_warn_only; nothing was proposed.
    ThresholdsUnused {
        /// Why it was refused: `thalamus.definition.ThresholdsUnused`.
        error: ThresholdsUnused,
    },
    /// `advisory-not-count` — when `(evaluation.rule == fixed_warn_only and kind != count_of_cases)`.
    ///
    /// The rule fixed_warn_only was named for a signal whose kind is not count_of_cases; nothing was proposed.
    AdvisoryNotCount {
        /// Why it was refused: `thalamus.definition.AdvisoryNeedsCountOfCases`.
        error: AdvisoryNeedsCountOfCases,
    },
    /// `advisory-not-fixed` — when `(evaluation.rule == fixed_warn_only and exists b in evaluation.baselines: (b != fixed))`.
    ///
    /// The rule fixed_warn_only was named with a baseline list that is not fixed alone; nothing was proposed.
    AdvisoryNotFixed {
        /// Why it was refused: `thalamus.definition.AdvisoryNeedsFixedBaseline`.
        error: AdvisoryNeedsFixedBaseline,
    },
    /// `proposed` — otherwise.
    ///
    /// The definition exists at revision 1 and is Proposed.
    Proposed {
        /// The `thalamus.definition.DefinitionProposed` this outcome publishes.
        definition_proposed: DefinitionProposed,
    },
    /// `already-defined` — for an identity a record already carries.
    ///
    /// The id is taken; nothing changed.
    AlreadyDefined {
        /// Why it was refused: `thalamus.definition.DefinitionExists`.
        error: DefinitionExists,
    },
}

/// Retire a definition — the input of `thalamus.definition.RetireDefinition`.
///
/// Stop the signal for good. Its samples and evaluations stay.
///
/// Everything it can result in is [`RetireDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetireDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// Everything `thalamus.definition.RetireDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetireDefinitionOutcome {
    /// `retired` — otherwise.
    ///
    /// The definition is Retired; it is never sampled again.
    Retired {
        /// The `thalamus.definition.DefinitionRetired` this outcome publishes.
        definition_retired: DefinitionRetired,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The definition is already Retired; nothing changed.
    WrongState {
        /// Why it was refused: `thalamus.definition.DefinitionStateConflict`.
        error: DefinitionStateConflict,
    },
    /// `no-such-definition` — for an identity no record carries.
    ///
    /// No definition has this id; nothing changed.
    NoSuchDefinition {
        /// Why it was refused: `thalamus.definition.UnknownDefinition`.
        error: UnknownDefinition,
    },
}

/// Revise a definition — the input of `thalamus.definition.ReviseDefinition`.
///
/// Replace a definition's body and count one revision, with its date and author. A revision always declares unit, dimensions, denominator and owner.
///
/// Everything it can result in is [`ReviseDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviseDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `title` — `String`.
    pub title: String,
    /// `family` — `thalamus.definition.Family`.
    pub family: Family,
    /// `kind` — `thalamus.definition.SignalKind`.
    pub kind: SignalKind,
    /// `unit` — `thalamus.definition.Unit`.
    pub unit: Unit,
    /// `dimensions` — `List<thalamus.dimension.DimensionKey>`.
    pub dimensions: Vec<crate::dimension::DimensionKey>,
    /// `source` — `thalamus.definition.Source`.
    pub source: Source,
    /// `extractor` — `thalamus.definition.ExtractorId`.
    pub extractor: ExtractorId,
    /// `query` — `Map<String, thalamus.definition.QueryText>`.
    pub query: std::collections::BTreeMap<String, QueryText>,
    /// `window` — `Duration`.
    pub window: crate::primitives::Duration,
    /// `cadence` — `Duration`.
    pub cadence: crate::primitives::Duration,
    /// `history` — `thalamus.definition.History`.
    pub history: History,
    /// `denominator` — `thalamus.definition.DenominatorSpec`.
    pub denominator: DenominatorSpec,
    /// `evaluation` — `thalamus.definition.EvaluationSpec`.
    pub evaluation: EvaluationSpec,
    /// `persistence` — `Optional<thalamus.definition.Persistence>`.
    pub persistence: Option<Persistence>,
    /// `objective` — `Optional<Json>`.
    pub objective: Option<crate::json::Value>,
    /// `owner` — `thalamus.definition.Owner`.
    pub owner: Owner,
    /// `links` — `thalamus.definition.Links`.
    pub links: Links,
    /// `author` — `thalamus.definition.Author`.
    pub author: Author,
}

/// Everything `thalamus.definition.ReviseDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviseDefinitionOutcome {
    /// `retired` — when the existing subject's stored fields satisfy `state == Retired`.
    ///
    /// The definition is retired; nothing changed.
    Retired {
        /// Why it was refused: `thalamus.definition.RetiredDefinition`.
        error: RetiredDefinition,
    },
    /// `instant-not-fixed` — when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`.
    ///
    /// An instant signal declared a baseline other than fixed; nothing changed.
    InstantNotFixed {
        /// Why it was refused: `thalamus.definition.InstantNeedsFixedBaseline`.
        error: InstantNeedsFixedBaseline,
    },
    /// `rolling-n-missing` — when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`.
    ///
    /// A rolling_median baseline was declared without evaluation.rolling_n; nothing changed.
    RollingNMissing {
        /// Why it was refused: `thalamus.definition.RollingWindowMissing`.
        error: RollingWindowMissing,
    },
    /// `rolling-n-unused` — when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`.
    ///
    /// evaluation.rolling_n was declared without a rolling_median baseline to read it; nothing changed.
    RollingNUnused {
        /// Why it was refused: `thalamus.definition.RollingWindowUnused`.
        error: RollingWindowUnused,
    },
    /// `fixed-number-missing` — when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`.
    ///
    /// A fixed baseline was declared without evaluation.fixed; nothing changed.
    FixedNumberMissing {
        /// Why it was refused: `thalamus.definition.FixedNumberMissing`.
        error: FixedNumberMissing,
    },
    /// `fixed-number-unused` — when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`.
    ///
    /// evaluation.fixed was declared without a fixed baseline to read it; nothing changed.
    FixedNumberUnused {
        /// Why it was refused: `thalamus.definition.FixedNumberUnused`.
        error: FixedNumberUnused,
    },
    /// `persistence-below-one` — when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`.
    ///
    /// persistence k, k_crit or m is below 1, settling or clearing on no evaluation; nothing changed.
    PersistenceBelowOne {
        /// Why it was refused: `thalamus.definition.PersistenceTooSmall`.
        error: PersistenceTooSmall,
    },
    /// `mapping-missing` — when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`.
    ///
    /// The rule producer_mapped was named without evaluation.mapping; nothing changed.
    MappingMissing {
        /// Why it was refused: `thalamus.definition.MappingMissing`.
        error: MappingMissing,
    },
    /// `mapping-unused` — when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`.
    ///
    /// evaluation.mapping was declared under a rule other than producer_mapped; nothing changed.
    MappingUnused {
        /// Why it was refused: `thalamus.definition.MappingUnused`.
        error: MappingUnused,
    },
    /// `thresholds-unused` — when `((evaluation.rule == producer_mapped or evaluation.rule == fixed_warn_only) and (defined(evaluation.warn) or defined(evaluation.crit)))`.
    ///
    /// A warn or crit line was declared under producer_mapped or fixed_warn_only; nothing changed.
    ThresholdsUnused {
        /// Why it was refused: `thalamus.definition.ThresholdsUnused`.
        error: ThresholdsUnused,
    },
    /// `advisory-not-count` — when `(evaluation.rule == fixed_warn_only and kind != count_of_cases)`.
    ///
    /// The rule fixed_warn_only was named for a signal whose kind is not count_of_cases; nothing changed.
    AdvisoryNotCount {
        /// Why it was refused: `thalamus.definition.AdvisoryNeedsCountOfCases`.
        error: AdvisoryNeedsCountOfCases,
    },
    /// `advisory-not-fixed` — when `(evaluation.rule == fixed_warn_only and exists b in evaluation.baselines: (b != fixed))`.
    ///
    /// The rule fixed_warn_only was named with a baseline list that is not fixed alone; nothing changed.
    AdvisoryNotFixed {
        /// Why it was refused: `thalamus.definition.AdvisoryNeedsFixedBaseline`.
        error: AdvisoryNeedsFixedBaseline,
    },
    /// `revised` — otherwise.
    ///
    /// The body is replaced and rev is one higher; evaluations already recorded keep the rev they name.
    Revised {
        /// The `thalamus.definition.DefinitionRevised` this outcome publishes.
        definition_revised: DefinitionRevised,
    },
    /// `no-such-definition` — for an identity no record carries.
    ///
    /// No definition has this id; nothing changed.
    NoSuchDefinition {
        /// Why it was refused: `thalamus.definition.UnknownDefinition`.
        error: UnknownDefinition,
    },
}

/// Unmute a definition — the input of `thalamus.definition.UnmuteDefinition`.
///
/// Return a muted definition to Active and drop its mute.
///
/// Everything it can result in is [`UnmuteDefinitionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmuteDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// Everything `thalamus.definition.UnmuteDefinition` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnmuteDefinitionOutcome {
    /// `unmuted` — otherwise.
    ///
    /// The definition is Active again and carries no mute.
    Unmuted {
        /// The `thalamus.definition.DefinitionUnmuted` this outcome publishes.
        definition_unmuted: DefinitionUnmuted,
    },
    /// `wrong-state` — from a state no declared move starts in.
    ///
    /// The definition is not Muted; nothing changed.
    WrongState {
        /// Why it was refused: `thalamus.definition.DefinitionStateConflict`.
        error: DefinitionStateConflict,
    },
    /// `no-such-definition` — for an identity no record carries.
    ///
    /// No definition has this id; nothing changed.
    NoSuchDefinition {
        /// Why it was refused: `thalamus.definition.UnknownDefinition`.
        error: UnknownDefinition,
    },
}

/// DefinitionActivated — the event `thalamus.definition.DefinitionActivated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionActivated {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// DefinitionMuted — the event `thalamus.definition.DefinitionMuted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionMuted {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `reason` — `String`.
    pub reason: String,
    /// `until` — `Timestamp`.
    pub until: crate::primitives::Timestamp,
}

/// DefinitionProposed — the event `thalamus.definition.DefinitionProposed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionProposed {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `revised_by` — `thalamus.definition.Author`.
    pub revised_by: Author,
}

/// DefinitionRetired — the event `thalamus.definition.DefinitionRetired`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionRetired {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// DefinitionRevised — the event `thalamus.definition.DefinitionRevised`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionRevised {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `previous_rev` — `Integer`.
    pub previous_rev: i64,
    /// `revised_by` — `thalamus.definition.Author`.
    pub revised_by: Author,
}

/// DefinitionUnmuted — the event `thalamus.definition.DefinitionUnmuted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionUnmuted {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.AdvisoryNeedsCountOfCases`.
///
/// The rule fixed_warn_only grades an advisory gate's finding count, a signal of kind count_of_cases, and its score steps to the first whole count above the baseline; another kind was declared, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryNeedsCountOfCases {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.AdvisoryNeedsFixedBaseline`.
///
/// The rule fixed_warn_only grades an advisory gate's count against the definition's fixed baseline, and evaluation.baselines is not fixed alone; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryNeedsFixedBaseline {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.DefinitionExists`.
///
/// A definition with this id exists; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionExists {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.DefinitionStateConflict`.
///
/// The definition is not in a state this command acts from; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionStateConflict {
    /// `current` — `thalamus.definition.SignalDefinition.State`.
    pub current: SignalDefinitionState,
}

/// The declared error `thalamus.definition.FixedNumberMissing`.
///
/// A fixed baseline is the definition's own number, and evaluation.fixed declares none; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedNumberMissing {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.FixedNumberUnused`.
///
/// evaluation.fixed is declared, and no fixed baseline reads it; a threshold nothing reads is not kept, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedNumberUnused {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.IncompleteDefinition`.
///
/// An active definition declares unit, dimensions, denominator and owner; at least one is missing, so it stays Proposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.InstantNeedsFixedBaseline`.
///
/// A signal of kind instant is graded against a fixed baseline only; another baseline was declared, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantNeedsFixedBaseline {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.MappingMissing`.
///
/// The rule producer_mapped grades a producer's own value through evaluation.mapping, and the definition declares no mapping onto OK, WARN, CRIT or UNKNOWN; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingMissing {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.MappingUnused`.
///
/// evaluation.mapping is declared, and the rule is not producer_mapped, so nothing reads it; a table nothing reads is not kept, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingUnused {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.MuteExpiryPassed`.
///
/// A mute carries an expiry in the future; this one has passed, so nothing was muted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuteExpiryPassed {
    /// `until` — `Timestamp`.
    pub until: crate::primitives::Timestamp,
}

/// The declared error `thalamus.definition.PersistenceTooSmall`.
///
/// persistence k, k_crit or m is below 1, which settles or clears a verdict on no evaluation; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceTooSmall {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.RetiredDefinition`.
///
/// A retired definition takes no revision; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.RollingWindowMissing`.
///
/// A rolling_median baseline is the median of the last N samples, and evaluation.rolling_n declares no N of at least 1; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollingWindowMissing {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.RollingWindowUnused`.
///
/// evaluation.rolling_n is declared, and no rolling_median baseline reads it; a threshold nothing reads is not kept, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollingWindowUnused {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.ThresholdsUnused`.
///
/// The rule producer_mapped grades by evaluation.mapping alone, and fixed_warn_only by its fixed baseline alone, and evaluation.warn or evaluation.crit is declared; a line nothing reads is not kept, so nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThresholdsUnused {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// The declared error `thalamus.definition.UnknownDefinition`.
///
/// No definition has this id; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownDefinition {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
}

/// Active definitions — one row of the view `thalamus.definition.ActiveDefinitions`.
///
/// Projects `thalamus.definition.SignalDefinition` at `read_your_writes` consistency, containing instances where `state == Active`.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveDefinitions {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `title` — `String`.
    pub title: String,
    /// `family` — `thalamus.definition.Family`.
    pub family: Family,
    /// `kind` — `thalamus.definition.SignalKind`.
    pub kind: SignalKind,
    /// `unit` — `Optional<thalamus.definition.Unit>`.
    pub unit: Option<Unit>,
    /// `dimensions` — `Optional<List<thalamus.dimension.DimensionKey>>`.
    pub dimensions: Option<Vec<crate::dimension::DimensionKey>>,
    /// `source` — `thalamus.definition.Source`.
    pub source: Source,
    /// `cadence` — `Duration`.
    pub cadence: crate::primitives::Duration,
    /// `owner` — `Optional<thalamus.definition.Owner>`.
    pub owner: Option<Owner>,
    /// `rev` — `Integer`.
    pub rev: i64,
}

/// Definitions — one row of the view `thalamus.definition.Definitions`.
///
/// Projects `thalamus.definition.SignalDefinition` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definitions {
    /// `signal_id` — `thalamus.definition.SignalId`.
    pub signal_id: SignalId,
    /// `title` — `String`.
    pub title: String,
    /// `family` — `thalamus.definition.Family`.
    pub family: Family,
    /// `kind` — `thalamus.definition.SignalKind`.
    pub kind: SignalKind,
    /// `unit` — `Optional<thalamus.definition.Unit>`.
    pub unit: Option<Unit>,
    /// `dimensions` — `Optional<List<thalamus.dimension.DimensionKey>>`.
    pub dimensions: Option<Vec<crate::dimension::DimensionKey>>,
    /// `source` — `thalamus.definition.Source`.
    pub source: Source,
    /// `extractor` — `thalamus.definition.ExtractorId`.
    pub extractor: ExtractorId,
    /// `query` — `Map<String, thalamus.definition.QueryText>`.
    pub query: std::collections::BTreeMap<String, QueryText>,
    /// `window` — `Duration`.
    pub window: crate::primitives::Duration,
    /// `cadence` — `Duration`.
    pub cadence: crate::primitives::Duration,
    /// `history` — `thalamus.definition.History`.
    pub history: History,
    /// `denominator` — `Optional<thalamus.definition.DenominatorSpec>`.
    pub denominator: Option<DenominatorSpec>,
    /// `objective` — `Optional<Json>`.
    pub objective: Option<crate::json::Value>,
    /// `owner` — `Optional<thalamus.definition.Owner>`.
    pub owner: Option<Owner>,
    /// `links` — `thalamus.definition.Links`.
    pub links: Links,
    /// `rev` — `Integer`.
    pub rev: i64,
    /// `revised_at` — `Timestamp`.
    pub revised_at: crate::primitives::Timestamp,
    /// `revised_by` — `thalamus.definition.Author`.
    pub revised_by: Author,
    /// `mute_reason` — `Optional<String>`.
    pub mute_reason: Option<String>,
    /// `mute_until` — `Optional<Timestamp>`.
    pub mute_until: Option<crate::primitives::Timestamp>,
    /// `state` — `thalamus.definition.SignalDefinition.State`.
    pub state: SignalDefinitionState,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.definition.ActivateDefinition` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait ActivateDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.ActivateDefinition`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn activate_definition(&mut self, input: super::ActivateDefinition) -> Result<super::ActivateDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.definition.MuteDefinition` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait MuteDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.MuteDefinition`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn mute_definition(&mut self, input: super::MuteDefinition) -> Result<super::MuteDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.definition.ProposeDefinition` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by the guard `exists b in evaluation.baselines: (b != fixed)`, in `instant-not-fixed`.
    ///
    /// Contract: given `thalamus.definition.ProposeDefinition` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `instant-not-fixed` when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.InstantNeedsFixedBaseline`; `rolling-n-missing` when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`, error `thalamus.definition.RollingWindowMissing`; `rolling-n-unused` when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`, error `thalamus.definition.RollingWindowUnused`; `fixed-number-missing` when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`, error `thalamus.definition.FixedNumberMissing`; `fixed-number-unused` when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.FixedNumberUnused`; `persistence-below-one` when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`, error `thalamus.definition.PersistenceTooSmall`; `mapping-missing` when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`, error `thalamus.definition.MappingMissing`; `mapping-unused` when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`, error `thalamus.definition.MappingUnused`; `thresholds-unused` when `((evaluation.rule == producer_mapped or evaluation.rule == fixed_warn_only) and (defined(evaluation.warn) or defined(evaluation.crit)))`, error `thalamus.definition.ThresholdsUnused`; `advisory-not-count` when `(evaluation.rule == fixed_warn_only and kind != count_of_cases)`, error `thalamus.definition.AdvisoryNeedsCountOfCases`; `advisory-not-fixed` when `(evaluation.rule == fixed_warn_only and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.AdvisoryNeedsFixedBaseline`; `proposed` otherwise, creates `thalamus.definition.SignalDefinition`, emits `thalamus.definition.DefinitionProposed`; `already-defined` for an identity a record already carries, error `thalamus.definition.DefinitionExists`.
    pub trait ProposeDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.ProposeDefinition`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn propose_definition(&mut self, input: super::ProposeDefinition) -> Result<super::ProposeDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.definition.RetireDefinition` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RetireDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.RetireDefinition`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn retire_definition(&mut self, input: super::RetireDefinition) -> Result<super::RetireDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.definition.ReviseDefinition` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by the guard `exists b in evaluation.baselines: (b != fixed)`, in `instant-not-fixed`.
    ///
    /// Contract: given `thalamus.definition.ReviseDefinition` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `retired` when the existing subject's stored fields satisfy `state == Retired`, error `thalamus.definition.RetiredDefinition`; `instant-not-fixed` when `(kind == instant and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.InstantNeedsFixedBaseline`; `rolling-n-missing` when `(exists b in evaluation.baselines: (b == rolling_median) and not (defined(evaluation.rolling_n)))`, error `thalamus.definition.RollingWindowMissing`; `rolling-n-unused` when `(defined(evaluation.rolling_n) and forall b in evaluation.baselines: (b != rolling_median))`, error `thalamus.definition.RollingWindowUnused`; `fixed-number-missing` when `(exists b in evaluation.baselines: (b == fixed) and not (defined(evaluation.fixed)))`, error `thalamus.definition.FixedNumberMissing`; `fixed-number-unused` when `(defined(evaluation.fixed) and forall b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.FixedNumberUnused`; `persistence-below-one` when `(persistence.k < 1 or persistence.k_crit < 1 or persistence.m < 1)`, error `thalamus.definition.PersistenceTooSmall`; `mapping-missing` when `(evaluation.rule == producer_mapped and not (defined(evaluation.mapping)))`, error `thalamus.definition.MappingMissing`; `mapping-unused` when `(defined(evaluation.mapping) and evaluation.rule != producer_mapped)`, error `thalamus.definition.MappingUnused`; `thresholds-unused` when `((evaluation.rule == producer_mapped or evaluation.rule == fixed_warn_only) and (defined(evaluation.warn) or defined(evaluation.crit)))`, error `thalamus.definition.ThresholdsUnused`; `advisory-not-count` when `(evaluation.rule == fixed_warn_only and kind != count_of_cases)`, error `thalamus.definition.AdvisoryNeedsCountOfCases`; `advisory-not-fixed` when `(evaluation.rule == fixed_warn_only and exists b in evaluation.baselines: (b != fixed))`, error `thalamus.definition.AdvisoryNeedsFixedBaseline`; `revised` otherwise, updates `thalamus.definition.SignalDefinition`, emits `thalamus.definition.DefinitionRevised`; `no-such-definition` for an identity no record carries, error `thalamus.definition.UnknownDefinition`.
    pub trait ReviseDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.ReviseDefinition`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn revise_definition(&mut self, input: super::ReviseDefinition) -> Result<super::ReviseDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.definition.UnmuteDefinition` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait UnmuteDefinitionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.definition.UnmuteDefinition`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn unmute_definition(&mut self, input: super::UnmuteDefinition) -> Result<super::UnmuteDefinitionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.definition.ActiveDefinitions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait ActiveDefinitionsQuery {
        /// Serves `thalamus.definition.ActiveDefinitions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn active_definitions(&self) -> Result<Vec<super::ActiveDefinitions>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.definition.Definitions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait DefinitionsQuery {
        /// Serves `thalamus.definition.Definitions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn definitions(&self) -> Result<Vec<super::Definitions>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl ProposeDefinitionBehavior for Unimplemented {
        fn propose_definition(&mut self, _input: super::ProposeDefinition) -> Result<super::ProposeDefinitionOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.definition.ProposeDefinition" })
        }
    }

    impl ReviseDefinitionBehavior for Unimplemented {
        fn revise_definition(&mut self, _input: super::ReviseDefinition) -> Result<super::ReviseDefinitionOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.definition.ReviseDefinition" })
        }
    }
}
