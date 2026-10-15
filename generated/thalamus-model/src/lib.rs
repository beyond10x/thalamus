// generated from thalamus v1
// model digest 827e774bfdab74d78674ef1f90b801f077677fc03bc8c865ffa778e5b79217a0
// contract digest 13c4b1fcc0534ac1d06b37453f62fcff2b8a5610397a25feaf2dc9a5f5254988
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

pub mod ports;
pub mod system;
#[cfg(feature = "server")]
pub mod server;
