// generated from thalamus v1
// model digest b94b21e9ee6e74c9891c76f5a65bded2d7cf1a055917c6c83b204dfafa73425d
// contract digest 6426f4dedf49a1f3f982631528ebb33f128fbda5cbdbc34328b8d2e7a2b2c736
// do not edit: regenerate with `ess synthesize --layout crate`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `thalamus.assertion.Answerer`.
    AssertionAnswerer,
    /// `thalamus.assertion.Caller`.
    AssertionCaller,
    /// `thalamus.definition.InstanceOperator`.
    DefinitionInstanceOperator,
    /// `thalamus.dimension.InstanceOperator`.
    DimensionInstanceOperator,
    /// `thalamus.evaluation.Grader`.
    EvaluationGrader,
    /// `thalamus.observation.Sampler`.
    ObservationSampler,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::AssertionAnswerer,
        Actor::AssertionCaller,
        Actor::DefinitionInstanceOperator,
        Actor::DimensionInstanceOperator,
        Actor::EvaluationGrader,
        Actor::ObservationSampler,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::AssertionAnswerer => "thalamus.assertion.Answerer",
            Actor::AssertionCaller => "thalamus.assertion.Caller",
            Actor::DefinitionInstanceOperator => "thalamus.definition.InstanceOperator",
            Actor::DimensionInstanceOperator => "thalamus.dimension.InstanceOperator",
            Actor::EvaluationGrader => "thalamus.evaluation.Grader",
            Actor::ObservationSampler => "thalamus.observation.Sampler",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::AssertionAnswerer => &[
            "thalamus.assertion.AnswerAssertion",
        ],
        Actor::AssertionCaller => &[
            "thalamus.assertion.AskAssertion",
        ],
        Actor::DefinitionInstanceOperator => &[
            "thalamus.definition.ActivateDefinition",
            "thalamus.definition.MuteDefinition",
            "thalamus.definition.ProposeDefinition",
            "thalamus.definition.RetireDefinition",
            "thalamus.definition.ReviseDefinition",
            "thalamus.definition.UnmuteDefinition",
        ],
        Actor::DimensionInstanceOperator => &[
            "thalamus.dimension.DeclareRegistry",
            "thalamus.dimension.RegisterDimension",
        ],
        Actor::EvaluationGrader => &[
            "thalamus.evaluation.RecordEvaluation",
        ],
        Actor::ObservationSampler => &[
            "thalamus.observation.RecordSample",
        ],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}
