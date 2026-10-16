//! Adapters that turn a Connectors operation into samples.

pub mod adapter;
pub mod connectors;
pub mod git;
pub mod host;
pub mod http_probe;
pub mod kubernetes;
pub mod loki;
pub mod prometheus;
pub mod registry;
pub mod sql;
pub mod vocabulary;
