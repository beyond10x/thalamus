//! Adversary pass 2 for `story:m0-probes`: the widened `thalamus.definition.Unit` alphabet and the
//! claim design § 8 makes about it ("`thalamus.definition.Unit` holds only the character set
//! (ASCII 33–126)").

use serde_json::{Value, json};
use thalamus_model::server::json as model_json;
use thalamus_model::server::wire::decode_thalamus_definition_unit;
use thalamus_spec_types::ThalamusDefinitionUnit;

/// ASCII 33–126 in order: 94 characters, no space, no control character.
fn ascii_33_to_126() -> String {
    (33u8..=126).map(char::from).collect()
}

/// Every `x-ess-alphabet` string in `value` that holds a curly brace: only `Unit`'s does.
fn unit_alphabets(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, member) in map {
                if key == "x-ess-alphabet"
                    && let Some(text) = member.as_str()
                    && text.contains('{')
                {
                    out.push(text.to_owned());
                }
                unit_alphabets(member, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| unit_alphabets(item, out)),
        _ => {}
    }
}

/// The YAML double-quoted scalar of `Unit`'s alphabet, unescaped. Only `\"` and `\\` are expected;
/// any other escape (`\t`, `\x20`, `\u0000`) fails the case.
fn yaml_unit_alphabet() -> String {
    let spec = include_str!("../../../spec/domains/definition.yaml");
    let after = spec
        .split("- name: thalamus.definition.Unit\n")
        .nth(1)
        .expect("Unit is declared");
    let line = after
        .lines()
        .find(|line| line.trim_start().starts_with("alphabet:"))
        .expect("Unit declares an alphabet");
    let quoted = line.trim_start().trim_start_matches("alphabet:").trim();
    let inner = quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("double-quoted");
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(escaped @ ('"' | '\\')) => out.push(escaped),
                other => panic!("unexpected YAML escape \\{other:?} in the Unit alphabet"),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The source alphabet and every generated copy of it are exactly ASCII 33–126: nothing missing,
/// nothing doubled, no whitespace or control character.
#[test]
fn adv_p2_unit_alphabet_is_exactly_ascii_33_to_126_everywhere() {
    let expected = ascii_33_to_126();
    assert_eq!(expected.len(), 94);
    assert_eq!(
        yaml_unit_alphabet(),
        expected,
        "spec/domains/definition.yaml"
    );

    // Every file that carries `Unit`'s alphabet, with how many copies it holds.
    let files: [(&str, &str, usize); 4] = [
        (
            "generated/spec-types/source.schema.json",
            include_str!("../../../generated/spec-types/source.schema.json"),
            1,
        ),
        (
            "generated/thalamus-model/src/server/thalamus-service.openapi.json",
            include_str!(
                "../../../generated/thalamus-model/src/server/thalamus-service.openapi.json"
            ),
            2,
        ),
        (
            "website/static/schemas/signal-definition.schema.json",
            include_str!("../../../website/static/schemas/signal-definition.schema.json"),
            1,
        ),
        (
            "website/static/schemas/signal-sample.schema.json",
            include_str!("../../../website/static/schemas/signal-sample.schema.json"),
            1,
        ),
    ];
    for (path, text, copies) in files {
        let value: Value = serde_json::from_str(text).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut found = Vec::new();
        unit_alphabets(&value, &mut found);
        assert_eq!(found.len(), copies, "{path}: copies of the Unit alphabet");
        for alphabet in &found {
            assert_eq!(alphabet, &expected, "{path}");
        }
    }
}

/// Design § 8: neither generated decoder enforces the `Unit` character set (the model's sits
/// behind `POST` of `ProposeDefinition`, `ReviseDefinition` and `RecordSample`), and
/// `parse_ucum` refuses every text outside it. Pass 2 found the decoders admitting these texts
/// (pre-existing, an ESS generator gap); the coordinator recorded the gap in § 8 instead of
/// claiming the decoders refuse them. This case fails once ESS enforces the set, so § 8 is
/// rewritten then.
#[test]
fn adv_p2_generated_decoders_admit_unit_outside_alphabet_and_parse_ucum_refuses() {
    let outside = ["m s", "m\ts", "m\u{0}", "m\u{7f}", "µm", "", " "];
    let mut model_admitted = Vec::new();
    let mut types_admitted = Vec::new();
    for text in outside {
        let wire = serde_json::to_string(&json!(text)).expect("encodes");
        let parsed = model_json::parse(&wire).expect("well-formed JSON");
        if decode_thalamus_definition_unit(&parsed, "unit").is_ok() {
            model_admitted.push(text);
        }
        if serde_json::from_value::<ThalamusDefinitionUnit>(json!(text)).is_ok() {
            types_admitted.push(text);
        }
    }
    assert_eq!(model_admitted, outside, "thalamus-model decoder");
    assert_eq!(types_admitted, outside, "thalamus-spec-types decoder");
    let parsed: Vec<&str> = outside
        .into_iter()
        .filter(|text| thalamus_core::unit::parse_ucum(text).is_ok())
        .collect();
    assert!(parsed.is_empty(), "parse_ucum admitted {parsed:?}");
}
