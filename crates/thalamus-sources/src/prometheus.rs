//! Prometheus (PromQL) reads. The adapter is `story:m1-prometheus-adapter`'s; until it lands the registry reads `unknown`.

/// The Prometheus (PromQL) adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
