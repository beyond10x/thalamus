// generated from thalamus v1
// model digest 73865b5d22d3d964014ec5c52cb6db35af4760818bbeefd77df0a2551b8fd3ed
// contract digest 093adf023da1db0be3662fe2c9b1542d25415a595e3023e37377255b2d0a3061
// do not edit: regenerate with `ess synthesize --layout crate`

//! Semantic types synthesised from the `thalamus` specification, v1.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod actor;
pub mod assertion;
pub mod behaviour;
pub mod definition;
pub mod dimension;
pub mod evaluation;
pub mod json;
pub mod obligation;
pub mod observation;
pub mod primitives;
pub mod rollup;
pub mod transition;

pub mod ports;
pub mod system;
#[cfg(feature = "server")]
pub mod server;
