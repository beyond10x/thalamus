// generated from thalamus v1
// model digest 51d971079fa66242e2a2f34142dbe44422e1204f980cec980f14fbeb58009d94
// contract digest fd2d9e49183a79ea356f366023aa03cd159da99e8e343e0af5bc5233645b32af
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
