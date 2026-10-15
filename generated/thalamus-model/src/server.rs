// generated from thalamus v1
// model digest 36486c1af727e3970e09eee21ce103728cca8a5870e0da6c3e5e285eb66f6345
// contract digest 743de6fde38e8322855a866475e38f9b1135e768965b29a2c1a9c9a5cce2d6eb
// do not edit: regenerate with `ess synthesize --layout crate`

//! The HTTP surface of `thalamus` v1, synthesised.
//!
//! One module per component the specification declares is reached over a network, each holding
//! that component's route table, its listener and the two documents it publishes about itself.
//! The routes are the ones the committed `OpenAPI` document declares, from the same mapping, so a
//! path served here and a path published there cannot be two different answers.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent is absent by
//! decision — no framework, no runtime, no second protocol, no concurrency, no authentication —
//! and each absence is argued in the `TARGET.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod entry;
pub mod http;
pub mod json;
pub mod wire;
pub mod thalamus_service;
pub mod memory;
mod static_assets;
