//! Git reads. The adapter is `story:m3-git-adapter`'s; until it lands the registry reads `unknown`.

/// The Git adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
