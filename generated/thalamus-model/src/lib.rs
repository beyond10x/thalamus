// generated from thalamus v1
// model digest 9f981109ead94ade6b2b664c0234683eabf8a4e37dbb57b0fab92ac76ace7832
// contract digest e31e759ce615e6d5358d020e748a76c36fee60b1207d5e97f762de8391357b06
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
