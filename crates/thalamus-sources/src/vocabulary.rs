//! Producer vocabulary (design § 2.3, L1): a producer that already speaks another vocabulary (a
//! colour ladder, a detector's or a gate's exit code) is mapped onto the one ladder by the table
//! in its definition's `evaluation.mapping`. No table lives in code, and none is applied here.
//!
//! A read keeps its own quality as the adapter made it: a text cell is `unknown` with the adapter's
//! reason, an exit code `1` is `ok` with value 1. Under `producer_mapped` the sample records the
//! producer's own value, [`sample_text`], and `thalamus-core` grading maps it through the mapping
//! at the revision being graded, so a mapping revision re-grades stored history (design § 2, D5).

use thalamus_core::rules::mapped::ID;
use thalamus_model::definition::SignalDefinitionData;
use thalamus_model::observation::Reading;

use crate::adapter::{Read, Unread};

/// The reads of `definition`, as the adapter made them: a read is never rewritten before it is
/// stored. The extractor registry passes every read through here.
pub fn map_reads(_definition: &SignalDefinitionData, reads: Vec<Read>) -> Vec<Read> {
    reads
}

/// The producer text the sample of `read` records under `definition`: its `raw` text when the
/// rule is `producer_mapped` and the adapter read the value, an `ok` reading or an `unknown` one
/// typed [`Unread::Text`]; `None` under every other rule, and for a read the adapter could not
/// make (cut, colliding, NULL, unreachable) or only bounded (`partial`), whatever text it carries.
#[must_use]
pub fn sample_text(definition: &SignalDefinitionData, read: &Read) -> Option<String> {
    if definition.evaluation.rule.0 != ID {
        return None;
    }
    let read_the_value = match read.reading {
        Reading::Ok(_) => true,
        Reading::Unknown(_) => read.unread == Some(Unread::Text),
        Reading::Partial(_) => false,
    };
    read_the_value.then(|| read.raw.clone()).flatten()
}
