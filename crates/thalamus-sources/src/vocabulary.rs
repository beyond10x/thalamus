//! Producer vocabulary mapping. The identity until `story:m3-vocabulary-mapping` replaces it.

use thalamus_model::definition::SignalDefinitionData;

use crate::adapter::Read;

/// The reads of `definition` in thalamus's own vocabulary.
pub fn map_reads(_definition: &SignalDefinitionData, reads: Vec<Read>) -> Vec<Read> {
    reads
}
