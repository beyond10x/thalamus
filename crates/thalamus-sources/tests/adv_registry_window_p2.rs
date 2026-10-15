//! Adversary pass 2 on `story:m1-instance-registry`: the length bound the correction added.
//!
//! `registry::placeable` admits a window when "a window of it ending at any instant from 1970 on
//! starts at an instant `window_ending` can write". `window_ending` writes the start as RFC 3339,
//! whose year is four digits, 0000 to 9999; the bound the loader applies is the `time` crate's
//! year -9999. A window between the two loads `Active` and reads unknown on every pass.

use thalamus_core::registry::{Definition, Registry};
use thalamus_model::primitives::Timestamp;
use thalamus_sources::adapter::window_ending;

const REGISTRY: &str = include_str!("../../thalamus-core/tests/fixtures/instance/dimensions.yaml");
const RATIO: &str = include_str!(
    "../../thalamus-core/tests/fixtures/instance/definitions/service.api.request_failure_ratio.yaml"
);

fn placed(window: &str, end: &str) -> Result<Option<String>, String> {
    let registry =
        Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry loads");
    let text = RATIO.replace("window: 6h", &format!("window: \"{window}\""));
    assert_ne!(text, RATIO, "the fixture's window line moved");
    let Ok(definition) = Definition::parse("definitions/case.yaml", &text, &registry) else {
        return Ok(None);
    };
    window_ending(&definition.data().window, &Timestamp(end.to_owned()))
        .map(|window| Some(window.start.0))
        .map_err(|why| {
            format!(
                "window {window} loaded as {:?}; ending {end} the adapters refuse it: {why}",
                definition.data().window.0
            )
        })
}

#[test]
fn adv_p2_an_admitted_long_window_is_placed_at_a_present_end() {
    let mut unplaced = Vec::new();
    for window in ["P800000D", "P4000000D", "P110000W"] {
        if let Err(why) = placed(window, "2026-10-14T22:57:03Z") {
            unplaced.push(why);
        }
    }
    assert!(unplaced.is_empty(), "{}", unplaced.join("\n"));
}

/// The shorthand and the edges the correction admits are placed, as the definition stores them.
#[test]
fn adv_p2_admitted_edges_are_placed() {
    for (window, start) in [
        ("6h", "2026-10-14T16:57:03Z"),
        ("1d", "2026-10-13T22:57:03Z"),
        ("1w", "2026-10-07T22:57:03Z"),
        ("P1W2D", "2026-10-05T22:57:03Z"),
        ("PT1S", "2026-10-14T22:57:02Z"),
        ("0006h", "2026-10-14T16:57:03Z"),
    ] {
        assert_eq!(
            placed(window, "2026-10-14T22:57:03Z"),
            Ok(Some(start.to_owned())),
            "{window}"
        );
    }
}
