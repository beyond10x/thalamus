//! Adversary pass 2 for `story:m0-probes`: the nesting bound `parse_ucum` gained in the correction
//! round. Its own binary, because a stack overflow aborts the whole test process.
//!
//! The bound is written here as the literal 64 (design and `UnitErrorKind::TooDeep` doc), not as
//! `MAX_DEPTH`, so a change to the constant fails a case. Square brackets and annotations are loops,
//! not recursion, so they must neither count toward the bound nor overflow on their own.

use thalamus_core::unit::{UnitErrorKind, parse_ucum};

fn nested(levels: usize, inner: &str) -> String {
    format!("{}{inner}{}", "(".repeat(levels), ")".repeat(levels))
}

/// 64 levels of `(` admitted, the 65th refused as `TooDeep` at its own byte offset, whatever the
/// innermost component is: an atom, a bracketed atom, an annotation, an annotated prefixed atom
/// with an exponent, or an annotation holding brackets and parentheses.
#[test]
fn adv_p2_depth_bound_is_64_for_every_innermost_component() {
    let inners = [
        "m",
        "[in_i]",
        "{x}",
        "k[IU]2{y}",
        "m{[a](b)}",
        "10*-3",
        "%[slope]",
    ];
    for inner in inners {
        let deepest = nested(64, inner);
        assert!(
            parse_ucum(&deepest).is_ok(),
            "refused 64 levels around {inner:?}: {:?}",
            parse_ucum(&deepest)
        );
        let error = parse_ucum(&nested(65, inner)).expect_err("admitted 65 levels");
        assert_eq!(
            (error.kind, error.at),
            (UnitErrorKind::TooDeep, 64),
            "65 levels around {inner:?}"
        );
    }
}

/// The 65th `(` is named by its byte offset, also after a leading `/` and inside a product.
#[test]
fn adv_p2_too_deep_names_the_offset_of_the_65th_parenthesis() {
    let text = format!("/{}", nested(65, "m"));
    let error = parse_ucum(&text).expect_err("admitted 65 levels after `/`");
    assert_eq!((error.kind, error.at), (UnitErrorKind::TooDeep, 65));

    let text = format!("s.{}", nested(65, "m"));
    let error = parse_ucum(&text).expect_err("admitted 65 levels after `s.`");
    assert_eq!((error.kind, error.at), (UnitErrorKind::TooDeep, 66));
}

/// Depth is how many `(` enclose a position, not how many have been seen: a closed `(` releases its
/// level. 1 000 sibling groups at the top level, and 200 siblings each at level 64, are admitted.
#[test]
fn adv_p2_sibling_parentheses_release_their_level() {
    let siblings = vec!["(m)"; 1_000].join(".");
    assert!(
        parse_ucum(&siblings).is_ok(),
        "refused 1000 sibling groups at depth 1"
    );

    let inner = vec!["(m)"; 200].join("/");
    let text = nested(63, &inner);
    assert!(
        parse_ucum(&text).is_ok(),
        "refused 200 siblings at depth 64: {:?}",
        parse_ucum(&text)
    );

    let chains = vec![nested(64, "s"); 50].join(".");
    assert!(
        parse_ucum(&chains).is_ok(),
        "refused 50 chains each 64 deep: {:?}",
        parse_ucum(&chains)
    );
}

/// No other path recurses: half a million `[`, `{`, `(` (unclosed), mixed openers, exponent digits,
/// annotation bytes and product factors all return.
#[test]
fn adv_p2_no_input_overflows_the_stack() {
    let n = 500_000;
    let texts = [
        "[".repeat(n),
        "{".repeat(n),
        "(".repeat(n),
        "([{".repeat(n / 3),
        format!("m{}", "2".repeat(n)),
        format!("{{{}}}", "[".repeat(n)),
        format!("[{}]", "a".repeat(n)),
        vec!["m"; n / 2].join("."),
        vec!["(m)"; n / 4].join("/"),
        format!("{}m", "(".repeat(n)),
    ];
    for text in &texts {
        let returned = std::panic::catch_unwind(|| parse_ucum(text).is_ok());
        assert!(returned.is_ok(), "panicked on a {}-byte input", text.len());
    }
    let error = parse_ucum(&"(".repeat(n)).expect_err("admitted unclosed parentheses");
    assert_eq!((error.kind, error.at), (UnitErrorKind::TooDeep, 64));
}
