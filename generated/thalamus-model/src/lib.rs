// generated from thalamus v1
// model digest 6f2848b89ff6190d89c005ba09023b83056dbdfb2fe499f9e414dc63d7d866bd
// contract digest 52593762eb7c5911c4f82160d91abef9b7c9217fd30b29c6b5cb7e3ac76f0515
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
