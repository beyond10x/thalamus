//! Kubernetes reads. The adapter is `story:m3-kubernetes-adapter`'s; until it lands the registry reads `unknown`.

/// The Kubernetes adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
