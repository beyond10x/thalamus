//! Adversary pass 1 on `story:m1-instance-registry`: every window the instance loader admits is
//! one the adapters can place. A definition that loads `Active` with a window no adapter reads
//! reads unknown on every pass.

use thalamus_core::registry::{Definition, Registry};
use thalamus_model::primitives::Timestamp;
use thalamus_sources::adapter::window_ending;

const REGISTRY: &str = include_str!("../../thalamus-core/tests/fixtures/instance/dimensions.yaml");
const RATIO: &str = include_str!(
    "../../thalamus-core/tests/fixtures/instance/definitions/service.api.request_failure_ratio.yaml"
);

#[test]
fn adv_every_admitted_window_is_one_the_adapters_place() {
    let registry =
        Registry::parse("dimensions.yaml", REGISTRY).expect("the fixture registry loads");
    let end = Timestamp("2026-10-14T22:57:03Z".into());
    let mut unplaced = Vec::new();
    for window in ["6h", "P1M", "P1Y", "99999999999999999999h"] {
        let text = RATIO.replace("window: 6h", &format!("window: \"{window}\""));
        assert_ne!(text, RATIO, "the fixture's window line moved");
        let Ok(definition) = Definition::parse("definitions/case.yaml", &text, &registry) else {
            continue;
        };
        if let Err(why) = window_ending(&definition.data().window, &end) {
            unplaced.push(format!(
                "window {window} loaded as {:?}; the adapters refuse it: {why}",
                definition.data().window.0
            ));
        }
    }
    assert!(unplaced.is_empty(), "{}", unplaced.join("\n"));
}
