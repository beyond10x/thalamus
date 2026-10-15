//! Adversary case for `story:m0-probes`, in its own binary because a stack overflow aborts the
//! whole test process. `parse_ucum` recurses once per `(`; the generated JSON parser bounds its own
//! recursion (`DEPTH` = 64 in `generated/thalamus-model/src/json.rs`) and the server reads bodies up
//! to `MAX_BODY` = 1 048 576 bytes, so a unit of about 500 000 nested parentheses fits one request.

use thalamus_core::unit::parse_ucum;

#[test]
fn adv_parse_ucum_returns_on_deep_nesting() {
    let depth = 500_000;
    let text = format!("{}m{}", "(".repeat(depth), ")".repeat(depth));
    // UCUM sets no nesting limit, so admitting it is correct and refusing it is a defensible
    // bound; what is not acceptable is not returning at all.
    let returned = std::panic::catch_unwind(|| parse_ucum(&text).is_ok());
    assert!(
        returned.is_ok(),
        "parse_ucum panicked on {depth} nested parentheses"
    );
}
