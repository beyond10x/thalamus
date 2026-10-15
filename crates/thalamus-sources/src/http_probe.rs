//! HTTP probe reads. The adapter is `story:m3-http-probe-adapter`'s; until it lands the registry reads `unknown`.

/// The HTTP probe adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
