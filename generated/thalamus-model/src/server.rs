// generated from thalamus v1
// model digest 827e774bfdab74d78674ef1f90b801f077677fc03bc8c865ffa778e5b79217a0
// contract digest 13c4b1fcc0534ac1d06b37453f62fcff2b8a5610397a25feaf2dc9a5f5254988
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
