//! The options every subcommand takes: where the instance is, where its store is, and which
//! `connectors` binary reads its sources.

use std::path::{Path, PathBuf};

use clap::Args;

/// The store an instance keeps when neither `--store` nor `THALAMUS_STORE` names one, relative to
/// the instance directory.
pub const DEFAULT_STORE: &str = ".thalamus/store.sqlite";

#[derive(Debug, Args)]
pub struct Global {
    /// The instance directory: `dimensions.yaml`, `definitions/*.yaml` and the registry files.
    #[arg(
        long,
        global = true,
        env = "THALAMUS_INSTANCE",
        hide_env_values = true,
        value_name = "DIR"
    )]
    pub instance: Option<PathBuf>,
    /// The instance's store: `sqlite:<path>` or a loopback `postgres://` URL without a password.
    /// Default: `sqlite:<instance>/.thalamus/store.sqlite`.
    #[arg(
        long,
        global = true,
        env = "THALAMUS_STORE",
        hide_env_values = true,
        value_name = "LOCATOR"
    )]
    pub store: Option<String>,
    /// The `connectors` binary sources are read through (else `THALAMUS_CONNECTORS`, else
    /// `connectors` on `PATH`).
    #[arg(long, global = true, value_name = "BIN")]
    pub connectors: Option<PathBuf>,
}

impl Global {
    /// The store locator: `--store`, else `THALAMUS_STORE`, else the instance's default store.
    #[must_use]
    pub fn store_locator(&self, instance: &Path) -> String {
        self.store
            .clone()
            .unwrap_or_else(|| format!("sqlite:{}", instance.join(DEFAULT_STORE).display()))
    }
}
