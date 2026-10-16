//! The extractor registry: the definition `extractor` prefix chooses the adapter.
//!
//! Each arm calls `adapter()` in that adapter's own file, and every read the registry returns
//! passes through [`vocabulary::map_reads`], so an adapter or vocabulary story changes only its
//! own file. A read that could not be made is an `unknown` read with a reason, never a dropped
//! series and never 0.

use thalamus_model::definition::{ExtractorId, SignalDefinitionData};
use thalamus_model::observation::RunId;
use thalamus_model::primitives::Timestamp;

use crate::adapter::{Adapter, Read, unknown};
use crate::connectors::Connectors;
use crate::{git, host, http_probe, kubernetes, loki, prometheus, sql, vocabulary};

/// The reason of a read whose adapter is not written yet.
pub const NOT_IMPLEMENTED: &str = "adapter not implemented";

/// The source kinds the registry has an arm for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// `promql.`
    Prometheus,
    /// `loki.`
    Loki,
    /// `sql.`
    Sql,
    /// `kubernetes.`
    Kubernetes,
    /// `git.`
    Git,
    /// `http_probe.`
    HttpProbe,
    /// `host.`: this machine's own counters, no Connectors operation (design § 11 D46).
    Host,
}

impl SourceKind {
    /// Every kind, in the order the registry tries their prefixes.
    pub const ALL: [SourceKind; 7] = [
        SourceKind::Prometheus,
        SourceKind::Loki,
        SourceKind::Sql,
        SourceKind::Kubernetes,
        SourceKind::Git,
        SourceKind::HttpProbe,
        SourceKind::Host,
    ];

    /// The extractor prefix this kind takes, dot included.
    pub fn prefix(self) -> &'static str {
        match self {
            SourceKind::Prometheus => "promql.",
            SourceKind::Loki => "loki.",
            SourceKind::Sql => "sql.",
            SourceKind::Kubernetes => "kubernetes.",
            SourceKind::Git => "git.",
            SourceKind::HttpProbe => "http_probe.",
            SourceKind::Host => "host.",
        }
    }

    /// The adapter of this kind, or `None` while its file holds the stub.
    pub fn adapter(self) -> Option<Box<dyn Adapter>> {
        match self {
            SourceKind::Prometheus => prometheus::adapter(),
            SourceKind::Loki => loki::adapter(),
            SourceKind::Sql => sql::adapter(),
            SourceKind::Kubernetes => kubernetes::adapter(),
            SourceKind::Git => git::adapter(),
            SourceKind::HttpProbe => http_probe::adapter(),
            SourceKind::Host => host::adapter(),
        }
    }
}

/// The kind whose prefix `extractor` starts with, followed by a name; `None` when no arm takes it.
pub fn choose(extractor: &ExtractorId) -> Option<SourceKind> {
    SourceKind::ALL.into_iter().find(|kind| {
        extractor
            .0
            .strip_prefix(kind.prefix())
            .is_some_and(|name| !name.is_empty())
    })
}

/// The reads of `definition` for the window ending at `end`, through the adapter its `extractor`
/// chooses.
pub fn read(
    connectors: &Connectors,
    definition: &SignalDefinitionData,
    end: &Timestamp,
    run_id: &RunId,
) -> Vec<Read> {
    match choose(&definition.extractor) {
        Some(kind) => read_with(
            kind.adapter().as_deref(),
            connectors,
            definition,
            end,
            run_id,
        ),
        None => vocabulary::map_reads(
            definition,
            vec![unknown(
                definition,
                end,
                format!("no adapter for extractor {}", definition.extractor.0),
            )],
        ),
    }
}

/// The reads of `definition` through `adapter`, or one `unknown` read saying
/// [`NOT_IMPLEMENTED`] when there is none. The adapter reads through a client admitting only the
/// operations it declared for `definition` ([`Adapter::operations_for`]); a Connectors failure
/// becomes one `unknown` read naming it.
pub fn read_with(
    adapter: Option<&dyn Adapter>,
    connectors: &Connectors,
    definition: &SignalDefinitionData,
    end: &Timestamp,
    run_id: &RunId,
) -> Vec<Read> {
    let reads = match adapter {
        None => vec![unknown(definition, end, NOT_IMPLEMENTED)],
        Some(adapter) => {
            let operations = adapter.operations_for(definition);
            let client = connectors.reads(&operations);
            adapter
                .read(&client, definition, end, run_id)
                .unwrap_or_else(|failure| vec![unknown(definition, end, failure.to_string())])
        }
    };
    vocabulary::map_reads(definition, reads)
}
