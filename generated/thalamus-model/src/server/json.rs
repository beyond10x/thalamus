// generated from thalamus v1
// model digest 99b2e1b31aebb3cdde7af7bb46be2ec80e4af2ccd183cd598985f3a30326c48f
// contract digest 82328f7c71509b62eb250d7912f8913dadad64f2cf9aae2897c422dfba7f784c
// do not edit: regenerate with `ess synthesize --layout crate`

//! JSON at this system's boundary: the types crate's own module, re-exported.
//!
//! This model uses `Json`, so a semantic type holds a `json::Value`, and the value this crate
//! decodes off the wire has to be that type rather than a second one beside it.

pub use crate::json::*;
