//! Loki (LogQL) reads. The adapter is `story:m3-loki-adapter`'s; until it lands the registry reads `unknown`.

/// The Loki (LogQL) adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
