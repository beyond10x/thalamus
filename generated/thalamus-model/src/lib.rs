// generated from thalamus v1
// model digest 6f2e9f65e0ef065a249a30bb38bae16ef166a5b45577b7a8f20509f1f3eddea7
// contract digest 067f632c99cd362c9c2d5e3d29f9b5d30e6604352424ed4eca28522b738edbee
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
