// generated from thalamus v1
// model digest 99b2e1b31aebb3cdde7af7bb46be2ec80e4af2ccd183cd598985f3a30326c48f
// contract digest 82328f7c71509b62eb250d7912f8913dadad64f2cf9aae2897c422dfba7f784c
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
