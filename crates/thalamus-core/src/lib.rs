//! Definitions, baselines, the verdict ladder, hysteresis and rollups. No I/O and no clock: time
//! and samples are passed in.

pub mod baseline;
pub mod behaviour;
#[cfg(feature = "conformance")]
pub mod conformance;
pub mod grading;
pub mod hysteresis;
pub mod ladder;
pub mod registry;
pub mod rollup;
pub mod rules;
pub mod series;
pub mod since;
pub mod unit;
