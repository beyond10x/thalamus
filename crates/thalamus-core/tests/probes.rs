//! The two constructs design § 8 probes before anything is built on the model.
//!
//! 1. `quality: unknown` carries no value. ESS states it: `thalamus.observation.Reading` is a union
//!    tagged on `quality`, and its `unknown` variant, `UnreadReading`, holds a `reason` and nothing
//!    else. These tests hold that answer against both generated crates, so a spec change that puts a
//!    value back on `unknown` fails the build.
//! 2. A unit is UCUM. ESS has no grammar-typed string, so `thalamus.definition.Unit` holds only the
//!    character set (`UNMAPPED:` in `spec/domains/definition.yaml`) and the grammar is held here,
//!    through `thalamus_core::unit::parse_ucum`.
//!
//! The wire form of a reading is the one ESS generates (design § 11, D11): the tag beside the
//! variant's own record, `{"quality": "unknown", "value": {"reason": "…"}}`. The `value` member is
//! that record, never a number; for `unknown` it holds `reason` and nothing else.

use serde_json::{Value, json};
use thalamus_core::unit::{MAX_DEPTH, UnitErrorKind, parse_ucum};
use thalamus_model::observation::{Reading, UnreadReading};
use thalamus_model::server::json as model_json;
use thalamus_model::server::wire::{
    decode_thalamus_observation_reading, encode_thalamus_observation_reading,
};
use thalamus_spec_types::{
    ThalamusObservationReading, ThalamusObservationReadingVariant2,
    ThalamusObservationReadingVariant2Quality, ThalamusObservationUnreadReading,
};

fn decode(reading: &Value) -> Result<ThalamusObservationReading, serde_json::Error> {
    serde_json::from_value(reading.clone())
}

#[test]
fn unknown_reading_carries_no_value() {
    let refused = [
        // A number beside the reason, inside the unknown variant's record.
        json!({"quality": "unknown", "value": {"reason": "source timed out", "value": "0"}}),
        // The flat form of the first draft: reason and value beside the tag.
        json!({"quality": "unknown", "reason": "source timed out", "value": "0"}),
        // A number in place of the variant's record.
        json!({"quality": "unknown", "value": "0"}),
        json!({"quality": "unknown", "value": 0}),
        // A value and no reason.
        json!({"quality": "unknown", "value": {"value": "0"}}),
        // The ok record under the unknown tag.
        json!({"quality": "unknown", "value": {"value": "0.029"}}),
        // A bare unknown with nothing to say why.
        json!({"quality": "unknown"}),
    ];
    for reading in &refused {
        assert!(
            decode(reading).is_err(),
            "admitted an unknown reading that carries a value or lacks a reason: {reading}"
        );
    }

    let admitted = json!({"quality": "unknown", "value": {"reason": "source timed out"}});
    let decoded = decode(&admitted).expect("an unknown reading with only a reason is admitted");
    let ThalamusObservationReading::V2(ThalamusObservationReadingVariant2 { quality, value }) =
        &decoded
    else {
        panic!("decoded as another variant: {decoded:?}");
    };
    assert_eq!(*quality, ThalamusObservationReadingVariant2Quality::V0);
    // Exhaustive: a field added to the unknown record breaks this pattern.
    let ThalamusObservationUnreadReading { reason } = value.as_ref();
    assert_eq!(reason, "source timed out");

    let reserialised = serde_json::to_value(&decoded).expect("re-serialises");
    assert_eq!(reserialised, admitted);
    let record = reserialised["value"]
        .as_object()
        .expect("the unknown variant's record is an object");
    assert_eq!(
        record.keys().collect::<Vec<_>>(),
        ["reason"],
        "the unknown record holds a reason and nothing else"
    );

    // The model's unknown variant, built with an exhaustive struct literal: a field added to
    // `UnreadReading` stops this compiling.
    let model = Reading::Unknown(UnreadReading {
        reason: "source timed out".to_owned(),
    });
    let mut encoded = String::new();
    encode_thalamus_observation_reading(&model, &mut encoded);
    let encoded: Value = serde_json::from_str(&encoded).expect("the model encodes JSON");
    assert_eq!(
        encoded, admitted,
        "the model and the spec types agree on the wire form"
    );

    // The model's server decoder ignores a value under `unknown` and drops it: what it builds is
    // `Reading::Unknown` with a reason and nothing else.
    let smuggled = model_json::parse(&refused[0].to_string()).expect("well-formed JSON");
    match decode_thalamus_observation_reading(&smuggled, "reading") {
        Ok(Reading::Unknown(UnreadReading { reason })) => assert_eq!(reason, "source timed out"),
        other => panic!("the model decoder did not return Unknown with a reason only: {other:?}"),
    }
}

#[test]
fn ok_reading_requires_value() {
    let refused = [
        json!({"quality": "ok"}),
        json!({"quality": "ok", "value": {}}),
        json!({"quality": "ok", "value": {"reason": "source timed out"}}),
    ];
    for reading in &refused {
        assert!(
            decode(reading).is_err(),
            "admitted an ok reading without a value: {reading}"
        );
    }
    let admitted = json!({"quality": "ok", "value": {"value": "0.029"}});
    assert!(
        matches!(decode(&admitted), Ok(ThalamusObservationReading::V0(_))),
        "refused an ok reading with a value: {admitted}"
    );
}

#[test]
fn ucum_admits_design_units() {
    let units = [
        "1",
        "s",
        "By",
        "{request}",
        "{pod}",
        "ms",
        "kBy",
        "{request}/s",
        "m2",
        "10*3",
        // Beyond the design's examples: the rest of the syntax the parser claims.
        "{finding}",
        "%",
        "/s",
        "m-1",
        "m.s-2",
        "kg.m/s2",
        "10^3",
        "10*-3.mol",
        "KiBy",
        "dam",
        "[in_i]2",
        "k[IU]",
        "mm[Hg]",
        "B[10.nV]",
        "s{elapsed}",
        "(m/s)/s",
        "100",
        "[m/s2/Hz^(1/2)]",
    ];
    for unit in units {
        match parse_ucum(unit) {
            Ok(parsed) => assert_eq!(parsed.as_str(), unit),
            Err(error) => panic!("refused the UCUM unit {unit:?}: {error}"),
        }
    }
}

#[test]
fn ucum_bounds_nesting_at_64_levels() {
    let nested = |levels: usize| format!("{}m{}", "(".repeat(levels), ")".repeat(levels));
    let deepest = nested(MAX_DEPTH);
    assert!(
        parse_ucum(&deepest).is_ok(),
        "refused {MAX_DEPTH} nested parentheses"
    );
    let error = parse_ucum(&nested(MAX_DEPTH + 1)).expect_err("admitted 65 nested parentheses");
    assert_eq!(error.kind, UnitErrorKind::TooDeep);
    assert_eq!(error.at, MAX_DEPTH, "the error names the 65th `(`");
}

#[test]
fn ucum_refuses_non_units() {
    let texts = [
        "",
        "{request",
        "m^2",
        "sec",
        "s s",
        // Beyond the story's list.
        "request}",
        "{re{quest}",
        "S/",
        "//s",
        "m..s",
        "(m",
        "m)",
        "()",
        "kmin",
        "kh",
        "[in_i",
        "M",
        "2m",
        "ms{a b}",
        "seconds",
        "Sec",
    ];
    for text in texts {
        if let Ok(parsed) = parse_ucum(text) {
            panic!("admitted {text:?} as the UCUM unit {:?}", parsed.as_str());
        }
    }
}
