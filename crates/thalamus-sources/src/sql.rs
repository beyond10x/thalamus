//! SQL reads. The adapter is `story:m3-sql-adapter`'s; until it lands the registry reads `unknown`.

/// The SQL adapter, or `None` while this stub stands.
pub fn adapter() -> Option<Box<dyn crate::adapter::Adapter>> {
    None
}
