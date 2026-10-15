//! Adversary case: `story:m0-conformance-definitions` runs `cargo test -p thalamus-core --test
//! conformance`, and that test calls `thalamus_core::conformance::run`. The module is declared
//! behind the `conformance` feature, so this crate's own integration tests only see it when the
//! feature is on for them. That story owns neither `crates/thalamus-core/Cargo.toml` nor
//! `Taskfile.yml`, so the skeleton has to turn it on.

#[test]
fn the_conformance_module_is_compiled_for_this_crates_integration_tests() {
    assert!(
        std::hint::black_box(cfg!(feature = "conformance")),
        "thalamus-core's integration tests are built without the `conformance` feature, so \
         `thalamus_core::conformance` does not exist for tests/conformance.rs"
    );
}
