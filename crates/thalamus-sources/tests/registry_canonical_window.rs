//! `story:m1-instance-registry`, adversary pass 2, finding 2: the instance loader stores every
//! `window` in the canonical RFC 3339 Appendix A form, and the adapters place that form exactly
//! as they place the form the operator wrote.

use thalamus_core::registry::{Definition, Registry};
use thalamus_model::primitives::{Duration, Timestamp};
use thalamus_sources::adapter::window_ending;

const REGISTRY: &str = include_str!("../../thalamus-core/tests/fixtures/instance/dimensions.yaml");
const RATIO: &str = include_str!(
    "../../thalamus-core/tests/fixtures/instance/definitions/service.api.request_failure_ratio.yaml"
);

/// What the operator wrote, the ISO 8601 form `window_ending` reads it as (the shorthand of
/// design § 2.1 written out), and the canonical form the loader stores.
const CASES: [(&str, &str, &str); 12] = [
    ("6h", "PT6H", "PT6H"),
    ("0006h", "PT6H", "PT6H"),
    ("90m", "PT90M", "PT90M"),
    ("1w", "P1W", "P1W"),
    ("P2W", "P2W", "P2W"),
    ("P1W2D", "P1W2D", "P9D"),
    ("PT1H1S", "PT1H1S", "PT1H0M1S"),
    ("P1DT1H1S", "P1DT1H1S", "P1DT1H0M1S"),
    ("P1W1DT1H1M1S", "P1W1DT1H1M1S", "P8DT1H1M1S"),
    ("P1WT1H", "P1WT1H", "P7DT1H"),
    ("P1DT12H", "P1DT12H", "P1DT12H"),
    ("PT1S", "PT1S", "PT1S"),
];

const ENDS: [&str; 3] = [
    "1970-01-01T00:00:00Z",
    "2026-10-14T22:57:03Z",
    "2024-02-29T23:59:59Z",
];

#[test]
fn the_canonical_window_is_placed_as_the_written_one() {
    let registry =
        Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry loads");
    for (written, iso, canonical) in CASES {
        let text = RATIO.replace("window: 6h", &format!("window: \"{written}\""));
        assert_ne!(text, RATIO, "the fixture's window line moved");
        let definition = Definition::parse("definitions/case.yaml", &text, &registry)
            .unwrap_or_else(|errors| panic!("{written}: {errors:?}"));
        let stored = &definition.data().window;
        assert_eq!(stored.0, canonical, "{written}");
        for end in ENDS {
            let end = Timestamp(end.to_owned());
            assert_eq!(
                window_ending(stored, &end),
                window_ending(&Duration(iso.to_owned()), &end),
                "{written} stored as {canonical}, ending {}",
                end.0
            );
            assert!(window_ending(stored, &end).is_ok(), "{written}");
        }
    }
}
