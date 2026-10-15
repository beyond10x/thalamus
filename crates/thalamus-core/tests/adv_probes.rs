//! Adversary cases for `story:m0-probes`, written against the UCUM 2.2 essence
//! (`ucum-essence.xml`, version 2.2, revision-date 2024-06-17) and the UCUM specification text
//! (§ 3 character set, § 4 prefixes, § 6 curly braces), and against the design § 8 claim about
//! how the generated model decoder treats an unknown reading carrying a value.

use thalamus_core::unit::parse_ucum;
use thalamus_model::observation::{Reading, UnreadReading};
use thalamus_model::server::json;
use thalamus_model::server::wire::decode_thalamus_observation_reading;

fn refused(text: &str) -> bool {
    parse_ucum(text).is_err()
}

/// UCUM 2.2 defines 24 prefixes: Y Z E P T G M k h da d c m u n p f a z y Ki Mi Gi Ti. The SI
/// prefixes ronna, quetta, ronto and quecto (R, Q, r, q) are not among them.
#[test]
fn adv_ucum_refuses_prefixes_outside_ucum_2_2() {
    let admitted: Vec<&str> = ["Rm", "Qg", "rm", "qs"]
        .into_iter()
        .filter(|text| !refused(text))
        .collect();
    assert!(
        admitted.is_empty(),
        "admitted prefixes UCUM 2.2 does not define: {admitted:?}"
    );
}

/// The essence marks `att` (technical atmosphere) and `b` (barn) `isMetric="no"`, so neither takes
/// a prefix.
#[test]
fn adv_ucum_refuses_prefix_on_non_metric_att_and_barn() {
    let admitted: Vec<&str> = ["katt", "matt", "mb", "kb"]
        .into_iter()
        .filter(|text| !refused(text))
        .collect();
    assert!(
        admitted.is_empty(),
        "admitted a prefix on an atom UCUM 2.2 marks non-metric: {admitted:?}"
    );
}

/// `atm` (standard atmosphere) and `[lbf_av]` (pound force) are UCUM 2.2 atoms.
#[test]
fn adv_ucum_admits_atm_and_pound_force() {
    let refused_units: Vec<&str> = ["atm", "[lbf_av]", "[lbf_av]/[sin_i]"]
        .into_iter()
        .filter(|text| refused(text))
        .collect();
    assert!(
        refused_units.is_empty(),
        "refused UCUM 2.2 atoms: {refused_units:?}"
    );
}

/// § 6 ■1: "The full range of characters 33–126 can be used within a pair of curly braces";
/// only nesting of curly braces is excluded (■5). Square brackets are in that range.
#[test]
fn adv_ucum_admits_square_brackets_inside_annotation() {
    let refused_units: Vec<&str> = ["{a[b]}", "m{[x]}", "{]}"]
        .into_iter()
        .filter(|text| refused(text))
        .collect();
    assert!(
        refused_units.is_empty(),
        "refused annotations UCUM § 6 admits: {refused_units:?}"
    );
}

/// The model crate's server decoder reads the reading of `POST /samples/commands/record`
/// (`decode_command_thalamus_observation_record_sample`). Design § 8: it ignores a value under
/// `unknown` and drops it, so the decoded `Reading::Unknown` carries a reason and no value.
///
/// Rewritten in the correction round (wave 20261015c coordinator): this case first asserted the
/// decoder refuses the reading. The decoder is generated ESS code, not edited here, and design
/// § 8 now states what it does.
#[test]
fn adv_model_decoder_drops_value_under_unknown() {
    let text = r#"{"quality":"unknown","value":{"reason":"source timed out","value":"0"}}"#;
    let value = json::parse(text).expect("well-formed JSON");
    let decoded = decode_thalamus_observation_reading(&value, "reading");
    match decoded {
        // Exhaustive: a field added to `UnreadReading` breaks this pattern.
        Ok(Reading::Unknown(UnreadReading { reason })) => {
            assert_eq!(reason, "source timed out");
        }
        other => panic!("the model decoder did not return Unknown with a reason only: {other:?}"),
    }
}

/// The `alphabet` of `thalamus.definition.Unit` in `spec/domains/definition.yaml`.
fn unit_alphabet() -> &'static str {
    let spec = include_str!("../../../spec/domains/definition.yaml");
    let after = spec
        .split("- name: thalamus.definition.Unit\n")
        .nth(1)
        .expect("thalamus.definition.Unit is declared");
    let line = after
        .lines()
        .find(|line| line.trim_start().starts_with("alphabet:"))
        .expect("Unit declares an alphabet");
    let quoted = line.trim_start().trim_start_matches("alphabet:").trim();
    quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("the alphabet is a double-quoted scalar")
}

/// The spec says `Unit` holds the character set of a UCUM unit and `parse_ucum` the grammar. A
/// unit both UCUM and `parse_ucum` admit must therefore fit the character set.
#[test]
fn adv_unit_alphabet_admits_what_parse_ucum_admits() {
    let alphabet = unit_alphabet();
    let units = ["(m/s)/s", "[m/s2/Hz^(1/2)]", "{a,b}"];
    let mut outside = Vec::new();
    for unit in units {
        assert!(parse_ucum(unit).is_ok(), "parse_ucum refused {unit:?}");
        if !unit.chars().all(|c| alphabet.contains(c)) {
            outside.push(unit);
        }
    }
    assert!(
        outside.is_empty(),
        "UCUM units parse_ucum admits fall outside the Unit alphabet {alphabet:?}: {outside:?}"
    );
}

/// No input panics: every string of up to four characters over an alphabet of the grammar's
/// delimiters, digits, letters, a multi-byte character and a space, and a fixed-seed sample of
/// longer strings over all of ASCII and three multi-byte characters.
#[test]
fn adv_parse_ucum_never_panics() {
    let alphabet: Vec<char> = "()[]{}./*^+-0129mskBy%' µ".chars().collect();
    let mut text = String::new();
    fn walk(alphabet: &[char], text: &mut String, depth: usize) {
        let _ = parse_ucum(text);
        if depth == 4 {
            return;
        }
        for &c in alphabet {
            text.push(c);
            walk(alphabet, text, depth + 1);
            text.pop();
        }
    }
    walk(&alphabet, &mut text, 0);

    let extra = ['µ', 'é', '€'];
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for _ in 0..200_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let len = (state % 16) as usize;
        let mut sample = String::new();
        let mut s = state;
        for _ in 0..len {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let pick = (s % 131) as u8;
            if pick < 128 {
                sample.push(char::from(pick));
            } else {
                sample.push(extra[usize::from(pick - 128)]);
            }
        }
        if let Ok(unit) = parse_ucum(&sample) {
            assert!(
                unit.as_str().bytes().all(|b| (33..=126).contains(&b)),
                "admitted a unit outside ASCII 33-126: {sample:?}"
            );
        }
    }
}
