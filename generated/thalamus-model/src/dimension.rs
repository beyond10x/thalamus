// generated from thalamus v1
// model digest b94b21e9ee6e74c9891c76f5a65bded2d7cf1a055917c6c83b204dfafa73425d
// contract digest 6426f4dedf49a1f3f982631528ebb33f128fbda5cbdbc34328b8d2e7a2b2c736
// do not edit: regenerate with `ess synthesize --layout crate`

//! Dimensions — `thalamus.dimension`.
//!
//! The closed, per-instance set of dimension keys a sample may carry, and where each key's values resolve.
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// The states of `thalamus.dimension.Dimension`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Dimension<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionState {
    /// `Registered`.
    Registered,
}

/// DimensionKey — `thalamus.dimension.DimensionKey`: a distinct wrapper around `String`.
///
/// A dimension key (cluster, namespace, tenant, component, suite).
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionKey(pub String);

/// The states of `thalamus.dimension.DimensionRegistry`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `DimensionRegistry<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionRegistryState {
    /// `Declared`.
    Declared,
}

/// DimensionRule — `thalamus.dimension.DimensionRule`: one of a closed set of names.
///
/// source_only means the key is set only when the source row carries it, never derived from a parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionRule {
    /// `source_only`.
    SourceOnly,
    /// `derived`.
    Derived,
}

/// DimensionValue — `thalamus.dimension.DimensionValue`: a distinct wrapper around `String`.
///
/// A value of one dimension key in one series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionValue(pub String);

/// Locator — `thalamus.dimension.Locator`: a distinct wrapper around `String`.
///
/// The inventory or registry file a key's values are listed in (inventory/clusters, registries/components.json).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locator(pub String);

/// RegistryId — `thalamus.dimension.RegistryId`: a distinct wrapper around `String`.
///
/// The instance whose dimension registry this is.
///
/// Every character is one of `abcdefghijklmnopqrstuvwxyz0123456789_-`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryId(pub String);

/// ValueSource — `thalamus.dimension.ValueSource`: one of a fixed set of shapes, tagged on the wire by `registry`.
///
/// The registry kind of a key. connectors is a Connectors inventory, file a registry file in the instance, open admits any value (test-run dimensions) and carries no locator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSource {
    /// Tagged `connectors` — `thalamus.dimension.Locator`.
    Connectors(Locator),
    /// Tagged `file` — `thalamus.dimension.Locator`.
    File(Locator),
    /// Tagged `open`, carrying nothing.
    Open,
}

/// What Dimension — `thalamus.dimension.Dimension` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Dimension<S>`], and at a boundary by [`DimensionSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionData {
    /// The identity: `key` — `thalamus.dimension.DimensionKey`.
    pub key: DimensionKey,
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    ///
    /// Carries `dimensions`: `thalamus.dimension.DimensionRegistry` owns many `thalamus.dimension.Dimension`.
    pub registry_id: RegistryId,
    /// `values` — `thalamus.dimension.ValueSource`.
    pub values: ValueSource,
    /// `rule` — `thalamus.dimension.DimensionRule`.
    pub rule: DimensionRule,
    /// `parent` — `Optional<thalamus.dimension.DimensionKey>`.
    ///
    /// Carries `parent_dimension`: `thalamus.dimension.Dimension` references one `thalamus.dimension.Dimension`.
    pub parent: Option<DimensionKey>,
    /// `otel_attribute` — `Optional<String>`.
    pub otel_attribute: Option<String>,
}

/// The states of `thalamus.dimension.Dimension`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](dimension_state::Marker), so [`Dimension<S>`](Dimension) can only ever rest in a real state.
pub mod dimension_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Registered {}
    }

    /// A declared state of `Dimension`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::DimensionState;
    }

    /// `Registered`. Where a new instance starts.
    pub struct Registered;

    impl Marker for Registered {
        const STATE: super::DimensionState = super::DimensionState::Registered;
    }
}

/// Dimension — `thalamus.dimension.Dimension` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Registered`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`DimensionSnapshot`]
/// and [`DimensionSnapshot::refine`].
pub struct Dimension<S: dimension_state::Marker> {
    data: DimensionData,
    state: core::marker::PhantomData<S>,
}

impl<S: dimension_state::Marker> Dimension<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> DimensionState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &DimensionData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> DimensionData {
        self.data
    }
}

impl Dimension<dimension_state::Registered> {
    /// A new instance, resting in `Registered` — the only state the lifecycle starts one in.
    pub fn new(data: DimensionData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.dimension.Dimension` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`DimensionSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: DimensionState,
    /// What it holds.
    pub data: DimensionData,
}

/// An `Dimension` in whichever declared state it was found.
pub enum AnyDimension {
    /// Resting in `Registered`.
    Registered(Dimension<dimension_state::Registered>),
}

impl DimensionSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `DimensionState` cannot spell one.
    pub fn refine(self) -> AnyDimension {
        match self.state {
            DimensionState::Registered => AnyDimension::Registered(Dimension {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyDimension {
    /// The state, as the runtime value.
    pub fn state(&self) -> DimensionState {
        match self {
            Self::Registered(_) => DimensionState::Registered,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> DimensionSnapshot {
        match self {
            Self::Registered(instance) => DimensionSnapshot {
                state: DimensionState::Registered,
                data: instance.into_data(),
            },
        }
    }
}

/// What DimensionRegistry — `thalamus.dimension.DimensionRegistry` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`DimensionRegistry<S>`], and at a boundary by [`DimensionRegistrySnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionRegistryData {
    /// The identity: `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `title` — `String`.
    pub title: String,
}

/// The states of `thalamus.dimension.DimensionRegistry`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](dimension_registry_state::Marker), so [`DimensionRegistry<S>`](DimensionRegistry) can only ever rest in a real state.
pub mod dimension_registry_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Declared {}
    }

    /// A declared state of `DimensionRegistry`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::DimensionRegistryState;
    }

    /// `Declared`. Where a new instance starts.
    pub struct Declared;

    impl Marker for Declared {
        const STATE: super::DimensionRegistryState = super::DimensionRegistryState::Declared;
    }
}

/// DimensionRegistry — `thalamus.dimension.DimensionRegistry` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Declared`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`DimensionRegistrySnapshot`]
/// and [`DimensionRegistrySnapshot::refine`].
pub struct DimensionRegistry<S: dimension_registry_state::Marker> {
    data: DimensionRegistryData,
    state: core::marker::PhantomData<S>,
}

impl<S: dimension_registry_state::Marker> DimensionRegistry<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> DimensionRegistryState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &DimensionRegistryData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> DimensionRegistryData {
        self.data
    }
}

impl DimensionRegistry<dimension_registry_state::Declared> {
    /// A new instance, resting in `Declared` — the only state the lifecycle starts one in.
    pub fn new(data: DimensionRegistryData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `thalamus.dimension.DimensionRegistry` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`DimensionRegistrySnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionRegistrySnapshot {
    /// Where the instance is in its lifecycle.
    pub state: DimensionRegistryState,
    /// What it holds.
    pub data: DimensionRegistryData,
}

/// An `DimensionRegistry` in whichever declared state it was found.
pub enum AnyDimensionRegistry {
    /// Resting in `Declared`.
    Declared(DimensionRegistry<dimension_registry_state::Declared>),
}

impl DimensionRegistrySnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `DimensionRegistryState` cannot spell one.
    pub fn refine(self) -> AnyDimensionRegistry {
        match self.state {
            DimensionRegistryState::Declared => AnyDimensionRegistry::Declared(DimensionRegistry {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyDimensionRegistry {
    /// The state, as the runtime value.
    pub fn state(&self) -> DimensionRegistryState {
        match self {
            Self::Declared(_) => DimensionRegistryState::Declared,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> DimensionRegistrySnapshot {
        match self {
            Self::Declared(instance) => DimensionRegistrySnapshot {
                state: DimensionRegistryState::Declared,
                data: instance.into_data(),
            },
        }
    }
}

/// Declare a dimension registry — the input of `thalamus.dimension.DeclareRegistry`.
///
/// Declare an instance's dimension registry, holding no keys yet.
///
/// Everything it can result in is [`DeclareRegistryOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclareRegistry {
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `title` — `String`.
    pub title: String,
}

/// Everything `thalamus.dimension.DeclareRegistry` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclareRegistryOutcome {
    /// `declared` — otherwise.
    ///
    /// The registry exists and admits no key yet.
    Declared {
        /// The `thalamus.dimension.RegistryDeclared` this outcome publishes.
        registry_declared: RegistryDeclared,
    },
    /// `already-declared` — for an identity a record already carries.
    ///
    /// The id is taken; nothing changed.
    AlreadyDeclared {
        /// Why it was refused: `thalamus.dimension.RegistryExists`.
        error: RegistryExists,
    },
}

/// Register a dimension key — the input of `thalamus.dimension.RegisterDimension`.
///
/// Admit one key to a registry, with its rule and where its values resolve.
///
/// Everything it can result in is [`RegisterDimensionOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterDimension {
    /// `key` — `thalamus.dimension.DimensionKey`.
    pub key: DimensionKey,
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `values` — `thalamus.dimension.ValueSource`.
    pub values: ValueSource,
    /// `rule` — `thalamus.dimension.DimensionRule`.
    pub rule: DimensionRule,
    /// `parent` — `Optional<thalamus.dimension.DimensionKey>`.
    pub parent: Option<DimensionKey>,
    /// `otel_attribute` — `Optional<String>`.
    pub otel_attribute: Option<String>,
}

/// Everything `thalamus.dimension.RegisterDimension` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterDimensionOutcome {
    /// `no-registry` — when no `thalamus.dimension.DimensionRegistry` carries the identity `input.registry_id` names.
    ///
    /// The registry is not declared; nothing was registered.
    NoRegistry {
        /// Why it was refused: `thalamus.dimension.UnknownRegistry`.
        error: UnknownRegistry,
    },
    /// `no-parent` — when no `thalamus.dimension.Dimension` carries the identity `input.parent` names.
    ///
    /// A parent key was named and is not registered; nothing was registered.
    NoParent {
        /// Why it was refused: `thalamus.dimension.UnknownParent`.
        error: UnknownParent,
    },
    /// `registered` — otherwise.
    ///
    /// The key is registered; samples may carry it.
    Registered {
        /// The `thalamus.dimension.DimensionRegistered` this outcome publishes.
        dimension_registered: DimensionRegistered,
    },
    /// `already-registered` — for an identity a record already carries.
    ///
    /// The key is already registered; nothing changed.
    AlreadyRegistered {
        /// Why it was refused: `thalamus.dimension.DimensionExists`.
        error: DimensionExists,
    },
}

/// DimensionRegistered — the event `thalamus.dimension.DimensionRegistered`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionRegistered {
    /// `key` — `thalamus.dimension.DimensionKey`.
    pub key: DimensionKey,
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `rule` — `thalamus.dimension.DimensionRule`.
    pub rule: DimensionRule,
}

/// RegistryDeclared — the event `thalamus.dimension.RegistryDeclared`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryDeclared {
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
}

/// The declared error `thalamus.dimension.DimensionExists`.
///
/// The key is already registered; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionExists {
    /// `key` — `thalamus.dimension.DimensionKey`.
    pub key: DimensionKey,
}

/// The declared error `thalamus.dimension.RegistryExists`.
///
/// A registry with this id is already declared; nothing changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryExists {
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
}

/// The declared error `thalamus.dimension.UnknownParent`.
///
/// The parent key is not registered; nothing was registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownParent {
    /// `parent` — `Optional<thalamus.dimension.DimensionKey>`.
    pub parent: Option<DimensionKey>,
}

/// The declared error `thalamus.dimension.UnknownRegistry`.
///
/// No registry with this id is declared; nothing was registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRegistry {
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
}

/// Dimension keys — one row of the view `thalamus.dimension.Dimensions`.
///
/// Projects `thalamus.dimension.Dimension` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimensions {
    /// `key` — `thalamus.dimension.DimensionKey`.
    pub key: DimensionKey,
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `values` — `thalamus.dimension.ValueSource`.
    pub values: ValueSource,
    /// `rule` — `thalamus.dimension.DimensionRule`.
    pub rule: DimensionRule,
    /// `parent` — `Optional<thalamus.dimension.DimensionKey>`.
    pub parent: Option<DimensionKey>,
    /// `otel_attribute` — `Optional<String>`.
    pub otel_attribute: Option<String>,
    /// `state` — `thalamus.dimension.Dimension.State`.
    pub state: DimensionState,
}

/// Dimension registries — one row of the view `thalamus.dimension.Registries`.
///
/// Projects `thalamus.dimension.DimensionRegistry` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registries {
    /// `registry_id` — `thalamus.dimension.RegistryId`.
    pub registry_id: RegistryId,
    /// `title` — `String`.
    pub title: String,
    /// `state` — `thalamus.dimension.DimensionRegistry.State`.
    pub state: DimensionRegistryState,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `thalamus.dimension.DeclareRegistry` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait DeclareRegistryBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.dimension.DeclareRegistry`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn declare_registry(&mut self, input: super::DeclareRegistry) -> Result<super::DeclareRegistryOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `thalamus.dimension.RegisterDimension` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by `when_related:` reading several related rows in one command (beyond10x/ess#283).
    ///
    /// Contract: given `thalamus.dimension.RegisterDimension` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` (read the related rows in the declaration order of their `exists: false` branches, and the first missing one answers) before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then choose the first declared present `when_related:` predicate refusal whose predicate and optional input guard hold, across rows; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `no-registry` when no `thalamus.dimension.DimensionRegistry` carries the identity `input.registry_id` names, error `thalamus.dimension.UnknownRegistry`; `no-parent` when no `thalamus.dimension.Dimension` carries the identity `input.parent` names, error `thalamus.dimension.UnknownParent`; `registered` otherwise, creates `thalamus.dimension.Dimension`, emits `thalamus.dimension.DimensionRegistered`; `already-registered` for an identity a record already carries, error `thalamus.dimension.DimensionExists`.
    pub trait RegisterDimensionBehavior {
        /// Decides and enacts exactly one declared outcome of `thalamus.dimension.RegisterDimension`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn register_dimension(&mut self, input: super::RegisterDimension) -> Result<super::RegisterDimensionOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.dimension.Dimensions` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait DimensionsQuery {
        /// Serves `thalamus.dimension.Dimensions` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn dimensions(&self) -> Result<Vec<super::Dimensions>, crate::obligation::UnmetObligation>;
    }

    /// The query `thalamus.dimension.Registries` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait RegistriesQuery {
        /// Serves `thalamus.dimension.Registries` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn registries(&self) -> Result<Vec<super::Registries>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl RegisterDimensionBehavior for Unimplemented {
        fn register_dimension(&mut self, _input: super::RegisterDimension) -> Result<super::RegisterDimensionOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "thalamus.dimension.RegisterDimension" })
        }
    }
}
