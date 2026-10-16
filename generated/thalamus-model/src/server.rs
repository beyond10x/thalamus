// generated from thalamus v1
// model digest 6f2848b89ff6190d89c005ba09023b83056dbdfb2fe499f9e414dc63d7d866bd
// contract digest 52593762eb7c5911c4f82160d91abef9b7c9217fd30b29c6b5cb7e3ac76f0515
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
