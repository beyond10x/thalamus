// generated from thalamus v1
// model digest 9f981109ead94ade6b2b664c0234683eabf8a4e37dbb57b0fab92ac76ace7832
// contract digest e31e759ce615e6d5358d020e748a76c36fee60b1207d5e97f762de8391357b06
// do not edit: regenerate with `ess synthesize --layout crate`
//! Every generated declaration, as JSON, in the renderings the published wire contracts fix.
//!
//! Generated from the model beside the types it crosses, so a field renamed in the specification
//! is renamed here in the same regeneration. An absent optional field is omitted rather
//! than sent as `null`, which is what the `required` list of the published schema says.

use crate::server::json;

/// Writes `thalamus.assertion.Asker` as JSON.
pub fn encode_thalamus_assertion_asker(value: &crate::assertion::Asker, out: &mut String) {
    out.push('{');
    json::member(out, "kind");
    encode_thalamus_assertion_asker_kind(&value.kind, out);
    json::member(out, "ref");
    json::push_text(out, &value.r#ref);
    out.push('}');
}

/// Reads `thalamus.assertion.Asker` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_asker(value: &json::Value, at: &str) -> Result<crate::assertion::Asker, json::DecodeError> {
    Ok(crate::assertion::Asker {
        kind: {
            let at0 = json::nested(at, "kind");
            let member0 = json::member_at(value, at, "kind")?;
            decode_thalamus_assertion_asker_kind(member0, &at0)?
        },
        r#ref: {
            let at1 = json::nested(at, "ref");
            let member1 = json::member_at(value, at, "ref")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes `thalamus.assertion.AskerKind` as JSON.
pub fn encode_thalamus_assertion_asker_kind(value: &crate::assertion::AskerKind, out: &mut String) {
    match value {
        crate::assertion::AskerKind::Release => json::push_text(out, "release"),
        crate::assertion::AskerKind::MergeRequest => json::push_text(out, "merge_request"),
        crate::assertion::AskerKind::TestRun => json::push_text(out, "test_run"),
        crate::assertion::AskerKind::Controller => json::push_text(out, "controller"),
    }
}

/// Reads `thalamus.assertion.AskerKind` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_asker_kind(value: &json::Value, at: &str) -> Result<crate::assertion::AskerKind, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `release`, `merge_request`, `test_run`, `controller`")? {
        "release" => crate::assertion::AskerKind::Release,
        "merge_request" => crate::assertion::AskerKind::MergeRequest,
        "test_run" => crate::assertion::AskerKind::TestRun,
        "controller" => crate::assertion::AskerKind::Controller,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `release`, `merge_request`, `test_run`, `controller`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.assertion.Assertion.State` as JSON.
pub fn encode_thalamus_assertion_assertion_state(value: &crate::assertion::AssertionState, out: &mut String) {
    match value {
        crate::assertion::AssertionState::Answered => json::push_text(out, "Answered"),
        crate::assertion::AssertionState::Asked => json::push_text(out, "Asked"),
    }
}

/// Reads `thalamus.assertion.Assertion.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_assertion_state(value: &json::Value, at: &str) -> Result<crate::assertion::AssertionState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Answered`, `Asked`")? {
        "Answered" => crate::assertion::AssertionState::Answered,
        "Asked" => crate::assertion::AssertionState::Asked,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Answered`, `Asked`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.assertion.AssertionId` as JSON.
pub fn encode_thalamus_assertion_assertion_id(value: &crate::assertion::AssertionId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.assertion.AssertionId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_assertion_id(value: &json::Value, at: &str) -> Result<crate::assertion::AssertionId, json::DecodeError> {
    Ok(crate::assertion::AssertionId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.assertion.AssertionResult` as JSON.
pub fn encode_thalamus_assertion_assertion_result(value: &crate::assertion::AssertionResult, out: &mut String) {
    match value {
        crate::assertion::AssertionResult::Within => json::push_text(out, "within"),
        crate::assertion::AssertionResult::Breach => json::push_text(out, "breach"),
        crate::assertion::AssertionResult::Unknown => json::push_text(out, "unknown"),
    }
}

/// Reads `thalamus.assertion.AssertionResult` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_assertion_result(value: &json::Value, at: &str) -> Result<crate::assertion::AssertionResult, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `within`, `breach`, `unknown`")? {
        "within" => crate::assertion::AssertionResult::Within,
        "breach" => crate::assertion::AssertionResult::Breach,
        "unknown" => crate::assertion::AssertionResult::Unknown,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `within`, `breach`, `unknown`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.assertion.Within` as JSON.
pub fn encode_thalamus_assertion_within(value: &crate::assertion::Within, out: &mut String) {
    match value {
        crate::assertion::Within::Baseline => json::push_text(out, "baseline"),
    }
}

/// Reads `thalamus.assertion.Within` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_assertion_within(value: &json::Value, at: &str) -> Result<crate::assertion::Within, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `baseline`")? {
        "baseline" => crate::assertion::Within::Baseline,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `baseline`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.definition.AdapterId` as JSON.
pub fn encode_thalamus_definition_adapter_id(value: &crate::definition::AdapterId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.AdapterId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_adapter_id(value: &json::Value, at: &str) -> Result<crate::definition::AdapterId, json::DecodeError> {
    Ok(crate::definition::AdapterId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.Author` as JSON.
pub fn encode_thalamus_definition_author(value: &crate::definition::Author, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.Author` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_author(value: &json::Value, at: &str) -> Result<crate::definition::Author, json::DecodeError> {
    Ok(crate::definition::Author(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.BaselineKind` as JSON.
pub fn encode_thalamus_definition_baseline_kind(value: &crate::definition::BaselineKind, out: &mut String) {
    match value {
        crate::definition::BaselineKind::Offset1d => json::push_text(out, "offset_1d"),
        crate::definition::BaselineKind::Offset7d => json::push_text(out, "offset_7d"),
        crate::definition::BaselineKind::Pooled => json::push_text(out, "pooled"),
        crate::definition::BaselineKind::HigherOf => json::push_text(out, "higher_of"),
        crate::definition::BaselineKind::LowerOf => json::push_text(out, "lower_of"),
        crate::definition::BaselineKind::SameSlotMedian4w => json::push_text(out, "same_slot_median_4w"),
        crate::definition::BaselineKind::RollingMedian => json::push_text(out, "rolling_median"),
        crate::definition::BaselineKind::Fixed => json::push_text(out, "fixed"),
    }
}

/// Reads `thalamus.definition.BaselineKind` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_baseline_kind(value: &json::Value, at: &str) -> Result<crate::definition::BaselineKind, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `offset_1d`, `offset_7d`, `pooled`, `higher_of`, `lower_of`, `same_slot_median_4w`, `rolling_median`, `fixed`")? {
        "offset_1d" => crate::definition::BaselineKind::Offset1d,
        "offset_7d" => crate::definition::BaselineKind::Offset7d,
        "pooled" => crate::definition::BaselineKind::Pooled,
        "higher_of" => crate::definition::BaselineKind::HigherOf,
        "lower_of" => crate::definition::BaselineKind::LowerOf,
        "same_slot_median_4w" => crate::definition::BaselineKind::SameSlotMedian4w,
        "rolling_median" => crate::definition::BaselineKind::RollingMedian,
        "fixed" => crate::definition::BaselineKind::Fixed,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `offset_1d`, `offset_7d`, `pooled`, `higher_of`, `lower_of`, `same_slot_median_4w`, `rolling_median`, `fixed`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.definition.ConnectionId` as JSON.
pub fn encode_thalamus_definition_connection_id(value: &crate::definition::ConnectionId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.ConnectionId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_connection_id(value: &json::Value, at: &str) -> Result<crate::definition::ConnectionId, json::DecodeError> {
    Ok(crate::definition::ConnectionId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.DenominatorSpec` as JSON.
pub fn encode_thalamus_definition_denominator_spec(value: &crate::definition::DenominatorSpec, out: &mut String) {
    out.push('{');
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    json::member(out, "min_n");
    json::push_integer(out, value.min_n);
    json::member(out, "since");
    json::push_text(out, &value.since);
    out.push('}');
}

/// Reads `thalamus.definition.DenominatorSpec` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_denominator_spec(value: &json::Value, at: &str) -> Result<crate::definition::DenominatorSpec, json::DecodeError> {
    Ok(crate::definition::DenominatorSpec {
        unit: {
            let at0 = json::nested(at, "unit");
            let member0 = json::member_at(value, at, "unit")?;
            decode_thalamus_definition_unit(member0, &at0)?
        },
        min_n: {
            let at1 = json::nested(at, "min_n");
            let member1 = json::member_at(value, at, "min_n")?;
            json::integer_at(member1, &at1, "an integer")?
        },
        since: {
            let at2 = json::nested(at, "since");
            let member2 = json::member_at(value, at, "since")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
    })
}

/// Writes `thalamus.definition.EvaluationSpec` as JSON.
pub fn encode_thalamus_definition_evaluation_spec(value: &crate::definition::EvaluationSpec, out: &mut String) {
    out.push('{');
    json::member(out, "rule");
    encode_thalamus_definition_rule_id(&value.rule, out);
    json::member(out, "baselines");
    out.push('[');
    for (index0, item0) in value.baselines.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_definition_baseline_kind(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.warn {
        json::member(out, "warn");
        encode_thalamus_definition_threshold(&*held0, out);
    }
    if let Some(held0) = &value.crit {
        json::member(out, "crit");
        encode_thalamus_definition_threshold(&*held0, out);
    }
    if let Some(held0) = &value.min_trend_window {
        json::member(out, "min_trend_window");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.fixed {
        json::member(out, "fixed");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.rolling_n {
        json::member(out, "rolling_n");
        json::push_integer(out, *held0);
    }
    if let Some(held0) = &value.mapping {
        json::member(out, "mapping");
        out.push('{');
        for (index1, (key1, item1)) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, key1);
            out.push(':');
            encode_thalamus_evaluation_verdict(&*item1, out);
        }
        out.push('}');
    }
    out.push('}');
}

/// Reads `thalamus.definition.EvaluationSpec` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_evaluation_spec(value: &json::Value, at: &str) -> Result<crate::definition::EvaluationSpec, json::DecodeError> {
    Ok(crate::definition::EvaluationSpec {
        rule: {
            let at0 = json::nested(at, "rule");
            let member0 = json::member_at(value, at, "rule")?;
            decode_thalamus_definition_rule_id(member0, &at0)?
        },
        baselines: {
            let at1 = json::nested(at, "baselines");
            let member1 = json::member_at(value, at, "baselines")?;
            {
                let mut items1 = Vec::new();
                for (index1, element1) in json::items_at(member1, &at1, "an array")?.iter().enumerate() {
                    let nested1 = json::nested(&at1, &index1.to_string());
                    items1.push(decode_thalamus_definition_baseline_kind(element1, &nested1)?);
                }
                items1
            }
        },
        warn: match value.member("warn") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "warn");
                Some(decode_thalamus_definition_threshold(member2, &at2)?)
            }
        },
        crit: match value.member("crit") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "crit");
                Some(decode_thalamus_definition_threshold(member3, &at3)?)
            }
        },
        min_trend_window: match value.member("min_trend_window") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "min_trend_window");
                Some(crate::primitives::Duration(json::text_at(member4, &at4, "an ISO 8601 duration")?.to_owned()))
            }
        },
        fixed: match value.member("fixed") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "fixed");
                Some(crate::primitives::Decimal(json::decimal_at(member5, &at5, "a decimal string")?.to_owned()))
            }
        },
        rolling_n: match value.member("rolling_n") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "rolling_n");
                Some(json::integer_at(member6, &at6, "an integer")?)
            }
        },
        mapping: match value.member("mapping") {
            None | Some(json::Value::Null) => None,
            Some(member7) => {
                let at7 = json::nested(at, "mapping");
                Some({
                let mut entries7 = std::collections::BTreeMap::new();
                for (key7, element7) in json::members_at(member7, &at7, "an object")? {
                    let nested7 = json::nested(&at7, key7);
                    entries7.insert(key7.clone(), decode_thalamus_evaluation_verdict(element7, &nested7)?);
                }
                entries7
            })
            }
        },
    })
}

/// Writes `thalamus.definition.ExtractorId` as JSON.
pub fn encode_thalamus_definition_extractor_id(value: &crate::definition::ExtractorId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.ExtractorId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_extractor_id(value: &json::Value, at: &str) -> Result<crate::definition::ExtractorId, json::DecodeError> {
    Ok(crate::definition::ExtractorId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.Family` as JSON.
pub fn encode_thalamus_definition_family(value: &crate::definition::Family, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.Family` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_family(value: &json::Value, at: &str) -> Result<crate::definition::Family, json::DecodeError> {
    Ok(crate::definition::Family(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.History` as JSON.
pub fn encode_thalamus_definition_history(value: &crate::definition::History, out: &mut String) {
    match value {
        crate::definition::History::Source => json::push_text(out, "source"),
        crate::definition::History::Store => json::push_text(out, "store"),
    }
}

/// Reads `thalamus.definition.History` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_history(value: &json::Value, at: &str) -> Result<crate::definition::History, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `source`, `store`")? {
        "source" => crate::definition::History::Source,
        "store" => crate::definition::History::Store,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `source`, `store`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.definition.Links` as JSON.
pub fn encode_thalamus_definition_links(value: &crate::definition::Links, out: &mut String) {
    out.push('{');
    json::member(out, "components");
    out.push('[');
    for (index0, item0) in value.components.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "issues");
    out.push('[');
    for (index0, item0) in value.issues.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    out.push('}');
}

/// Reads `thalamus.definition.Links` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_links(value: &json::Value, at: &str) -> Result<crate::definition::Links, json::DecodeError> {
    Ok(crate::definition::Links {
        components: {
            let at0 = json::nested(at, "components");
            let member0 = json::member_at(value, at, "components")?;
            {
                let mut items0 = Vec::new();
                for (index0, element0) in json::items_at(member0, &at0, "an array")?.iter().enumerate() {
                    let nested0 = json::nested(&at0, &index0.to_string());
                    items0.push(json::text_at(element0, &nested0, "a string")?.to_owned());
                }
                items0
            }
        },
        issues: {
            let at1 = json::nested(at, "issues");
            let member1 = json::member_at(value, at, "issues")?;
            {
                let mut items1 = Vec::new();
                for (index1, element1) in json::items_at(member1, &at1, "an array")?.iter().enumerate() {
                    let nested1 = json::nested(&at1, &index1.to_string());
                    items1.push(json::text_at(element1, &nested1, "a string")?.to_owned());
                }
                items1
            }
        },
    })
}

/// Writes `thalamus.definition.Owner` as JSON.
pub fn encode_thalamus_definition_owner(value: &crate::definition::Owner, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.Owner` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_owner(value: &json::Value, at: &str) -> Result<crate::definition::Owner, json::DecodeError> {
    Ok(crate::definition::Owner(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.Persistence` as JSON.
pub fn encode_thalamus_definition_persistence(value: &crate::definition::Persistence, out: &mut String) {
    out.push('{');
    json::member(out, "k");
    json::push_integer(out, value.k);
    json::member(out, "k_crit");
    json::push_integer(out, value.k_crit);
    json::member(out, "m");
    json::push_integer(out, value.m);
    out.push('}');
}

/// Reads `thalamus.definition.Persistence` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_persistence(value: &json::Value, at: &str) -> Result<crate::definition::Persistence, json::DecodeError> {
    Ok(crate::definition::Persistence {
        k: {
            let at0 = json::nested(at, "k");
            let member0 = json::member_at(value, at, "k")?;
            json::integer_at(member0, &at0, "an integer")?
        },
        k_crit: {
            let at1 = json::nested(at, "k_crit");
            let member1 = json::member_at(value, at, "k_crit")?;
            json::integer_at(member1, &at1, "an integer")?
        },
        m: {
            let at2 = json::nested(at, "m");
            let member2 = json::member_at(value, at, "m")?;
            json::integer_at(member2, &at2, "an integer")?
        },
    })
}

/// Writes `thalamus.definition.QueryText` as JSON.
pub fn encode_thalamus_definition_query_text(value: &crate::definition::QueryText, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.QueryText` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_query_text(value: &json::Value, at: &str) -> Result<crate::definition::QueryText, json::DecodeError> {
    Ok(crate::definition::QueryText(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.RuleId` as JSON.
pub fn encode_thalamus_definition_rule_id(value: &crate::definition::RuleId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.RuleId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_rule_id(value: &json::Value, at: &str) -> Result<crate::definition::RuleId, json::DecodeError> {
    Ok(crate::definition::RuleId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.SignalDefinition.State` as JSON.
pub fn encode_thalamus_definition_signal_definition_state(value: &crate::definition::SignalDefinitionState, out: &mut String) {
    match value {
        crate::definition::SignalDefinitionState::Active => json::push_text(out, "Active"),
        crate::definition::SignalDefinitionState::Muted => json::push_text(out, "Muted"),
        crate::definition::SignalDefinitionState::Proposed => json::push_text(out, "Proposed"),
        crate::definition::SignalDefinitionState::Retired => json::push_text(out, "Retired"),
    }
}

/// Reads `thalamus.definition.SignalDefinition.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_signal_definition_state(value: &json::Value, at: &str) -> Result<crate::definition::SignalDefinitionState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Active`, `Muted`, `Proposed`, `Retired`")? {
        "Active" => crate::definition::SignalDefinitionState::Active,
        "Muted" => crate::definition::SignalDefinitionState::Muted,
        "Proposed" => crate::definition::SignalDefinitionState::Proposed,
        "Retired" => crate::definition::SignalDefinitionState::Retired,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Active`, `Muted`, `Proposed`, `Retired`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.definition.SignalId` as JSON.
pub fn encode_thalamus_definition_signal_id(value: &crate::definition::SignalId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.SignalId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_signal_id(value: &json::Value, at: &str) -> Result<crate::definition::SignalId, json::DecodeError> {
    Ok(crate::definition::SignalId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.definition.SignalKind` as JSON.
pub fn encode_thalamus_definition_signal_kind(value: &crate::definition::SignalKind, out: &mut String) {
    match value {
        crate::definition::SignalKind::Gauge => json::push_text(out, "gauge"),
        crate::definition::SignalKind::CounterIncrease => json::push_text(out, "counter_increase"),
        crate::definition::SignalKind::Ratio => json::push_text(out, "ratio"),
        crate::definition::SignalKind::Instant => json::push_text(out, "instant"),
        crate::definition::SignalKind::CountOfCases => json::push_text(out, "count_of_cases"),
    }
}

/// Reads `thalamus.definition.SignalKind` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_signal_kind(value: &json::Value, at: &str) -> Result<crate::definition::SignalKind, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `gauge`, `counter_increase`, `ratio`, `instant`, `count_of_cases`")? {
        "gauge" => crate::definition::SignalKind::Gauge,
        "counter_increase" => crate::definition::SignalKind::CounterIncrease,
        "ratio" => crate::definition::SignalKind::Ratio,
        "instant" => crate::definition::SignalKind::Instant,
        "count_of_cases" => crate::definition::SignalKind::CountOfCases,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `gauge`, `counter_increase`, `ratio`, `instant`, `count_of_cases`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.definition.Source` as JSON.
pub fn encode_thalamus_definition_source(value: &crate::definition::Source, out: &mut String) {
    out.push('{');
    json::member(out, "adapter");
    encode_thalamus_definition_adapter_id(&value.adapter, out);
    json::member(out, "connection");
    encode_thalamus_definition_connection_id(&value.connection, out);
    out.push('}');
}

/// Reads `thalamus.definition.Source` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_source(value: &json::Value, at: &str) -> Result<crate::definition::Source, json::DecodeError> {
    Ok(crate::definition::Source {
        adapter: {
            let at0 = json::nested(at, "adapter");
            let member0 = json::member_at(value, at, "adapter")?;
            decode_thalamus_definition_adapter_id(member0, &at0)?
        },
        connection: {
            let at1 = json::nested(at, "connection");
            let member1 = json::member_at(value, at, "connection")?;
            decode_thalamus_definition_connection_id(member1, &at1)?
        },
    })
}

/// Writes `thalamus.definition.Threshold` as JSON.
pub fn encode_thalamus_definition_threshold(value: &crate::definition::Threshold, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.x {
        json::member(out, "x");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.abs {
        json::member(out, "abs");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.min_events {
        json::member(out, "min_events");
        json::push_integer(out, *held0);
    }
    out.push('}');
}

/// Reads `thalamus.definition.Threshold` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_threshold(value: &json::Value, at: &str) -> Result<crate::definition::Threshold, json::DecodeError> {
    Ok(crate::definition::Threshold {
        x: match value.member("x") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "x");
                Some(crate::primitives::Decimal(json::decimal_at(member0, &at0, "a decimal string")?.to_owned()))
            }
        },
        abs: match value.member("abs") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "abs");
                Some(crate::primitives::Decimal(json::decimal_at(member1, &at1, "a decimal string")?.to_owned()))
            }
        },
        min_events: match value.member("min_events") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "min_events");
                Some(json::integer_at(member2, &at2, "an integer")?)
            }
        },
    })
}

/// Writes `thalamus.definition.Unit` as JSON.
pub fn encode_thalamus_definition_unit(value: &crate::definition::Unit, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.definition.Unit` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_definition_unit(value: &json::Value, at: &str) -> Result<crate::definition::Unit, json::DecodeError> {
    Ok(crate::definition::Unit(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.dimension.Dimension.State` as JSON.
pub fn encode_thalamus_dimension_dimension_state(value: &crate::dimension::DimensionState, out: &mut String) {
    match value {
        crate::dimension::DimensionState::Registered => json::push_text(out, "Registered"),
    }
}

/// Reads `thalamus.dimension.Dimension.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_dimension_state(value: &json::Value, at: &str) -> Result<crate::dimension::DimensionState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Registered`")? {
        "Registered" => crate::dimension::DimensionState::Registered,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Registered`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.dimension.DimensionKey` as JSON.
pub fn encode_thalamus_dimension_dimension_key(value: &crate::dimension::DimensionKey, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.dimension.DimensionKey` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_dimension_key(value: &json::Value, at: &str) -> Result<crate::dimension::DimensionKey, json::DecodeError> {
    Ok(crate::dimension::DimensionKey(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.dimension.DimensionRegistry.State` as JSON.
pub fn encode_thalamus_dimension_dimension_registry_state(value: &crate::dimension::DimensionRegistryState, out: &mut String) {
    match value {
        crate::dimension::DimensionRegistryState::Declared => json::push_text(out, "Declared"),
    }
}

/// Reads `thalamus.dimension.DimensionRegistry.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_dimension_registry_state(value: &json::Value, at: &str) -> Result<crate::dimension::DimensionRegistryState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Declared`")? {
        "Declared" => crate::dimension::DimensionRegistryState::Declared,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Declared`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.dimension.DimensionRule` as JSON.
pub fn encode_thalamus_dimension_dimension_rule(value: &crate::dimension::DimensionRule, out: &mut String) {
    match value {
        crate::dimension::DimensionRule::SourceOnly => json::push_text(out, "source_only"),
        crate::dimension::DimensionRule::Derived => json::push_text(out, "derived"),
    }
}

/// Reads `thalamus.dimension.DimensionRule` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_dimension_rule(value: &json::Value, at: &str) -> Result<crate::dimension::DimensionRule, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `source_only`, `derived`")? {
        "source_only" => crate::dimension::DimensionRule::SourceOnly,
        "derived" => crate::dimension::DimensionRule::Derived,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `source_only`, `derived`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.dimension.DimensionValue` as JSON.
pub fn encode_thalamus_dimension_dimension_value(value: &crate::dimension::DimensionValue, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.dimension.DimensionValue` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_dimension_value(value: &json::Value, at: &str) -> Result<crate::dimension::DimensionValue, json::DecodeError> {
    Ok(crate::dimension::DimensionValue(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.dimension.Locator` as JSON.
pub fn encode_thalamus_dimension_locator(value: &crate::dimension::Locator, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.dimension.Locator` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_locator(value: &json::Value, at: &str) -> Result<crate::dimension::Locator, json::DecodeError> {
    Ok(crate::dimension::Locator(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.dimension.RegistryId` as JSON.
pub fn encode_thalamus_dimension_registry_id(value: &crate::dimension::RegistryId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.dimension.RegistryId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_registry_id(value: &json::Value, at: &str) -> Result<crate::dimension::RegistryId, json::DecodeError> {
    Ok(crate::dimension::RegistryId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.dimension.ValueSource` as JSON.
pub fn encode_thalamus_dimension_value_source(value: &crate::dimension::ValueSource, out: &mut String) {
    match value {
        crate::dimension::ValueSource::Connectors(held) => {
            out.push('{');
            json::member(out, "registry");
            json::push_text(out, "connectors");
            json::member(out, "value");
            encode_thalamus_dimension_locator(&*held, out);
            out.push('}');
        }
        crate::dimension::ValueSource::File(held) => {
            out.push('{');
            json::member(out, "registry");
            json::push_text(out, "file");
            json::member(out, "value");
            encode_thalamus_dimension_locator(&*held, out);
            out.push('}');
        }
        crate::dimension::ValueSource::Open => {
            out.push('{');
            json::member(out, "registry");
            json::push_text(out, "open");
            out.push('}');
        }
    }
}

/// Reads `thalamus.dimension.ValueSource` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_dimension_value_source(value: &json::Value, at: &str) -> Result<crate::dimension::ValueSource, json::DecodeError> {
    let tag = json::member_at(value, at, "registry")?;
    let at_tag = json::nested(at, "registry");
    Ok(match json::text_at(tag, &at_tag, "one of `connectors`, `file`, `open`")? {
        "connectors" => crate::dimension::ValueSource::Connectors({
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            decode_thalamus_dimension_locator(member0, &at0)?
        }),
        "file" => crate::dimension::ValueSource::File({
            let at1 = json::nested(at, "value");
            let member1 = json::member_at(value, at, "value")?;
            decode_thalamus_dimension_locator(member1, &at1)?
        }),
        "open" => {
            if let Some(found) = value.member("value") {
                return Err(json::DecodeError::of(&json::nested(at, "value"), "no `value`: `open` carries nothing", found));
            }
            crate::dimension::ValueSource::Open
        }
        other => return Err(json::DecodeError { at: at_tag.clone(), expected: "one of `connectors`, `file`, `open`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.evaluation.Baseline` as JSON.
pub fn encode_thalamus_evaluation_baseline(value: &crate::evaluation::Baseline, out: &mut String) {
    out.push('{');
    json::member(out, "kind");
    encode_thalamus_evaluation_baseline_label(&value.kind, out);
    json::member(out, "offsets");
    out.push('[');
    for (index0, item0) in value.offsets.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "reading");
    encode_thalamus_observation_reading(&value.reading, out);
    out.push('}');
}

/// Reads `thalamus.evaluation.Baseline` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_evaluation_baseline(value: &json::Value, at: &str) -> Result<crate::evaluation::Baseline, json::DecodeError> {
    Ok(crate::evaluation::Baseline {
        kind: {
            let at0 = json::nested(at, "kind");
            let member0 = json::member_at(value, at, "kind")?;
            decode_thalamus_evaluation_baseline_label(member0, &at0)?
        },
        offsets: {
            let at1 = json::nested(at, "offsets");
            let member1 = json::member_at(value, at, "offsets")?;
            {
                let mut items1 = Vec::new();
                for (index1, element1) in json::items_at(member1, &at1, "an array")?.iter().enumerate() {
                    let nested1 = json::nested(&at1, &index1.to_string());
                    items1.push(json::text_at(element1, &nested1, "a string")?.to_owned());
                }
                items1
            }
        },
        reading: {
            let at2 = json::nested(at, "reading");
            let member2 = json::member_at(value, at, "reading")?;
            decode_thalamus_observation_reading(member2, &at2)?
        },
    })
}

/// Writes `thalamus.evaluation.BaselineLabel` as JSON.
pub fn encode_thalamus_evaluation_baseline_label(value: &crate::evaluation::BaselineLabel, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.evaluation.BaselineLabel` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_evaluation_baseline_label(value: &json::Value, at: &str) -> Result<crate::evaluation::BaselineLabel, json::DecodeError> {
    Ok(crate::evaluation::BaselineLabel(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.evaluation.EvaluationId` as JSON.
pub fn encode_thalamus_evaluation_evaluation_id(value: &crate::evaluation::EvaluationId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.evaluation.EvaluationId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_evaluation_evaluation_id(value: &json::Value, at: &str) -> Result<crate::evaluation::EvaluationId, json::DecodeError> {
    Ok(crate::evaluation::EvaluationId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.evaluation.SignalEvaluation.State` as JSON.
pub fn encode_thalamus_evaluation_signal_evaluation_state(value: &crate::evaluation::SignalEvaluationState, out: &mut String) {
    match value {
        crate::evaluation::SignalEvaluationState::Recorded => json::push_text(out, "Recorded"),
    }
}

/// Reads `thalamus.evaluation.SignalEvaluation.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_evaluation_signal_evaluation_state(value: &json::Value, at: &str) -> Result<crate::evaluation::SignalEvaluationState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Recorded`")? {
        "Recorded" => crate::evaluation::SignalEvaluationState::Recorded,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Recorded`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.evaluation.Verdict` as JSON.
pub fn encode_thalamus_evaluation_verdict(value: &crate::evaluation::Verdict, out: &mut String) {
    match value {
        crate::evaluation::Verdict::OK => json::push_text(out, "OK"),
        crate::evaluation::Verdict::NA => json::push_text(out, "N/A"),
        crate::evaluation::Verdict::WARN => json::push_text(out, "WARN"),
        crate::evaluation::Verdict::CRIT => json::push_text(out, "CRIT"),
        crate::evaluation::Verdict::UNKNOWN => json::push_text(out, "UNKNOWN"),
    }
}

/// Reads `thalamus.evaluation.Verdict` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_evaluation_verdict(value: &json::Value, at: &str) -> Result<crate::evaluation::Verdict, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`")? {
        "OK" => crate::evaluation::Verdict::OK,
        "N/A" => crate::evaluation::Verdict::NA,
        "WARN" => crate::evaluation::Verdict::WARN,
        "CRIT" => crate::evaluation::Verdict::CRIT,
        "UNKNOWN" => crate::evaluation::Verdict::UNKNOWN,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `OK`, `N/A`, `WARN`, `CRIT`, `UNKNOWN`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.observation.Bound` as JSON.
pub fn encode_thalamus_observation_bound(value: &crate::observation::Bound, out: &mut String) {
    match value {
        crate::observation::Bound::Lower => json::push_text(out, "lower"),
        crate::observation::Bound::Upper => json::push_text(out, "upper"),
    }
}

/// Reads `thalamus.observation.Bound` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_bound(value: &json::Value, at: &str) -> Result<crate::observation::Bound, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `lower`, `upper`")? {
        "lower" => crate::observation::Bound::Lower,
        "upper" => crate::observation::Bound::Upper,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `lower`, `upper`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.observation.CompleteReading` as JSON.
pub fn encode_thalamus_observation_complete_reading(value: &crate::observation::CompleteReading, out: &mut String) {
    out.push('{');
    json::member(out, "value");
    json::push_text(out, &value.value.0);
    out.push('}');
}

/// Reads `thalamus.observation.CompleteReading` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_complete_reading(value: &json::Value, at: &str) -> Result<crate::observation::CompleteReading, json::DecodeError> {
    Ok(crate::observation::CompleteReading {
        value: {
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            crate::primitives::Decimal(json::decimal_at(member0, &at0, "a decimal string")?.to_owned())
        },
    })
}

/// Writes `thalamus.observation.Denominator` as JSON.
pub fn encode_thalamus_observation_denominator(value: &crate::observation::Denominator, out: &mut String) {
    out.push('{');
    json::member(out, "n");
    json::push_integer(out, value.n);
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    json::member(out, "since");
    json::push_text(out, &value.since.0);
    json::member(out, "age_s");
    json::push_integer(out, value.age_s);
    out.push('}');
}

/// Reads `thalamus.observation.Denominator` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_denominator(value: &json::Value, at: &str) -> Result<crate::observation::Denominator, json::DecodeError> {
    Ok(crate::observation::Denominator {
        n: {
            let at0 = json::nested(at, "n");
            let member0 = json::member_at(value, at, "n")?;
            json::integer_at(member0, &at0, "an integer")?
        },
        unit: {
            let at1 = json::nested(at, "unit");
            let member1 = json::member_at(value, at, "unit")?;
            decode_thalamus_definition_unit(member1, &at1)?
        },
        since: {
            let at2 = json::nested(at, "since");
            let member2 = json::member_at(value, at, "since")?;
            crate::primitives::Timestamp(json::text_at(member2, &at2, "an RFC 3339 instant")?.to_owned())
        },
        age_s: {
            let at3 = json::nested(at, "age_s");
            let member3 = json::member_at(value, at, "age_s")?;
            json::integer_at(member3, &at3, "an integer")?
        },
    })
}

/// Writes `thalamus.observation.PartialReading` as JSON.
pub fn encode_thalamus_observation_partial_reading(value: &crate::observation::PartialReading, out: &mut String) {
    out.push('{');
    json::member(out, "value");
    json::push_text(out, &value.value.0);
    json::member(out, "bound");
    encode_thalamus_observation_bound(&value.bound, out);
    out.push('}');
}

/// Reads `thalamus.observation.PartialReading` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_partial_reading(value: &json::Value, at: &str) -> Result<crate::observation::PartialReading, json::DecodeError> {
    Ok(crate::observation::PartialReading {
        value: {
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            crate::primitives::Decimal(json::decimal_at(member0, &at0, "a decimal string")?.to_owned())
        },
        bound: {
            let at1 = json::nested(at, "bound");
            let member1 = json::member_at(value, at, "bound")?;
            decode_thalamus_observation_bound(member1, &at1)?
        },
    })
}

/// Writes `thalamus.observation.Provenance` as JSON.
pub fn encode_thalamus_observation_provenance(value: &crate::observation::Provenance, out: &mut String) {
    out.push('{');
    json::member(out, "producer");
    json::push_text(out, &value.producer);
    json::member(out, "version");
    json::push_text(out, &value.version);
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    json::member(out, "adapter");
    encode_thalamus_definition_adapter_id(&value.adapter, out);
    json::member(out, "connection");
    encode_thalamus_definition_connection_id(&value.connection, out);
    json::member(out, "query_sha256");
    encode_thalamus_observation_query_digest(&value.query_sha256, out);
    json::member(out, "run_id");
    encode_thalamus_observation_run_id(&value.run_id, out);
    out.push('}');
}

/// Reads `thalamus.observation.Provenance` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_provenance(value: &json::Value, at: &str) -> Result<crate::observation::Provenance, json::DecodeError> {
    Ok(crate::observation::Provenance {
        producer: {
            let at0 = json::nested(at, "producer");
            let member0 = json::member_at(value, at, "producer")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        version: {
            let at1 = json::nested(at, "version");
            let member1 = json::member_at(value, at, "version")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        definition_rev: {
            let at2 = json::nested(at, "definition_rev");
            let member2 = json::member_at(value, at, "definition_rev")?;
            json::integer_at(member2, &at2, "an integer")?
        },
        adapter: {
            let at3 = json::nested(at, "adapter");
            let member3 = json::member_at(value, at, "adapter")?;
            decode_thalamus_definition_adapter_id(member3, &at3)?
        },
        connection: {
            let at4 = json::nested(at, "connection");
            let member4 = json::member_at(value, at, "connection")?;
            decode_thalamus_definition_connection_id(member4, &at4)?
        },
        query_sha256: {
            let at5 = json::nested(at, "query_sha256");
            let member5 = json::member_at(value, at, "query_sha256")?;
            decode_thalamus_observation_query_digest(member5, &at5)?
        },
        run_id: {
            let at6 = json::nested(at, "run_id");
            let member6 = json::member_at(value, at, "run_id")?;
            decode_thalamus_observation_run_id(member6, &at6)?
        },
    })
}

/// Writes `thalamus.observation.QueryDigest` as JSON.
pub fn encode_thalamus_observation_query_digest(value: &crate::observation::QueryDigest, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.observation.QueryDigest` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_query_digest(value: &json::Value, at: &str) -> Result<crate::observation::QueryDigest, json::DecodeError> {
    Ok(crate::observation::QueryDigest(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.observation.Reading` as JSON.
pub fn encode_thalamus_observation_reading(value: &crate::observation::Reading, out: &mut String) {
    match value {
        crate::observation::Reading::Ok(held) => {
            out.push('{');
            json::member(out, "quality");
            json::push_text(out, "ok");
            json::member(out, "value");
            encode_thalamus_observation_complete_reading(&*held, out);
            out.push('}');
        }
        crate::observation::Reading::Partial(held) => {
            out.push('{');
            json::member(out, "quality");
            json::push_text(out, "partial");
            json::member(out, "value");
            encode_thalamus_observation_partial_reading(&*held, out);
            out.push('}');
        }
        crate::observation::Reading::Unknown(held) => {
            out.push('{');
            json::member(out, "quality");
            json::push_text(out, "unknown");
            json::member(out, "value");
            encode_thalamus_observation_unread_reading(&*held, out);
            out.push('}');
        }
    }
}

/// Reads `thalamus.observation.Reading` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_reading(value: &json::Value, at: &str) -> Result<crate::observation::Reading, json::DecodeError> {
    let tag = json::member_at(value, at, "quality")?;
    let at_tag = json::nested(at, "quality");
    Ok(match json::text_at(tag, &at_tag, "one of `ok`, `partial`, `unknown`")? {
        "ok" => crate::observation::Reading::Ok({
            let at0 = json::nested(at, "value");
            let member0 = json::member_at(value, at, "value")?;
            decode_thalamus_observation_complete_reading(member0, &at0)?
        }),
        "partial" => crate::observation::Reading::Partial({
            let at1 = json::nested(at, "value");
            let member1 = json::member_at(value, at, "value")?;
            decode_thalamus_observation_partial_reading(member1, &at1)?
        }),
        "unknown" => crate::observation::Reading::Unknown({
            let at2 = json::nested(at, "value");
            let member2 = json::member_at(value, at, "value")?;
            decode_thalamus_observation_unread_reading(member2, &at2)?
        }),
        other => return Err(json::DecodeError { at: at_tag.clone(), expected: "one of `ok`, `partial`, `unknown`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.observation.RunId` as JSON.
pub fn encode_thalamus_observation_run_id(value: &crate::observation::RunId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.observation.RunId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_run_id(value: &json::Value, at: &str) -> Result<crate::observation::RunId, json::DecodeError> {
    Ok(crate::observation::RunId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.observation.SampleId` as JSON.
pub fn encode_thalamus_observation_sample_id(value: &crate::observation::SampleId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.observation.SampleId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_sample_id(value: &json::Value, at: &str) -> Result<crate::observation::SampleId, json::DecodeError> {
    Ok(crate::observation::SampleId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.observation.SeriesKey` as JSON.
pub fn encode_thalamus_observation_series_key(value: &crate::observation::SeriesKey, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.observation.SeriesKey` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_series_key(value: &json::Value, at: &str) -> Result<crate::observation::SeriesKey, json::DecodeError> {
    Ok(crate::observation::SeriesKey(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.observation.SignalSample.State` as JSON.
pub fn encode_thalamus_observation_signal_sample_state(value: &crate::observation::SignalSampleState, out: &mut String) {
    match value {
        crate::observation::SignalSampleState::Recorded => json::push_text(out, "Recorded"),
    }
}

/// Reads `thalamus.observation.SignalSample.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_signal_sample_state(value: &json::Value, at: &str) -> Result<crate::observation::SignalSampleState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Recorded`")? {
        "Recorded" => crate::observation::SignalSampleState::Recorded,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Recorded`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.observation.TimeWindow` as JSON.
pub fn encode_thalamus_observation_time_window(value: &crate::observation::TimeWindow, out: &mut String) {
    out.push('{');
    json::member(out, "start");
    json::push_text(out, &value.start.0);
    json::member(out, "end");
    json::push_text(out, &value.end.0);
    out.push('}');
}

/// Reads `thalamus.observation.TimeWindow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_time_window(value: &json::Value, at: &str) -> Result<crate::observation::TimeWindow, json::DecodeError> {
    Ok(crate::observation::TimeWindow {
        start: {
            let at0 = json::nested(at, "start");
            let member0 = json::member_at(value, at, "start")?;
            crate::primitives::Timestamp(json::text_at(member0, &at0, "an RFC 3339 instant")?.to_owned())
        },
        end: {
            let at1 = json::nested(at, "end");
            let member1 = json::member_at(value, at, "end")?;
            crate::primitives::Timestamp(json::text_at(member1, &at1, "an RFC 3339 instant")?.to_owned())
        },
    })
}

/// Writes `thalamus.observation.UnreadReading` as JSON.
pub fn encode_thalamus_observation_unread_reading(value: &crate::observation::UnreadReading, out: &mut String) {
    out.push('{');
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    out.push('}');
}

/// Reads `thalamus.observation.UnreadReading` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_observation_unread_reading(value: &json::Value, at: &str) -> Result<crate::observation::UnreadReading, json::DecodeError> {
    Ok(crate::observation::UnreadReading {
        reason: {
            let at0 = json::nested(at, "reason");
            let member0 = json::member_at(value, at, "reason")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
    })
}

/// Writes `thalamus.rollup.DailyRollup.State` as JSON.
pub fn encode_thalamus_rollup_daily_rollup_state(value: &crate::rollup::DailyRollupState, out: &mut String) {
    match value {
        crate::rollup::DailyRollupState::Recorded => json::push_text(out, "Recorded"),
    }
}

/// Reads `thalamus.rollup.DailyRollup.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_rollup_daily_rollup_state(value: &json::Value, at: &str) -> Result<crate::rollup::DailyRollupState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Recorded`")? {
        "Recorded" => crate::rollup::DailyRollupState::Recorded,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Recorded`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.rollup.RollupId` as JSON.
pub fn encode_thalamus_rollup_rollup_id(value: &crate::rollup::RollupId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.rollup.RollupId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_rollup_rollup_id(value: &json::Value, at: &str) -> Result<crate::rollup::RollupId, json::DecodeError> {
    Ok(crate::rollup::RollupId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `thalamus.rollup.VerdictCounts` as JSON.
pub fn encode_thalamus_rollup_verdict_counts(value: &crate::rollup::VerdictCounts, out: &mut String) {
    out.push('{');
    json::member(out, "ok");
    json::push_integer(out, value.ok);
    json::member(out, "na");
    json::push_integer(out, value.na);
    json::member(out, "warn");
    json::push_integer(out, value.warn);
    json::member(out, "crit");
    json::push_integer(out, value.crit);
    json::member(out, "unknown");
    json::push_integer(out, value.unknown);
    out.push('}');
}

/// Reads `thalamus.rollup.VerdictCounts` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_rollup_verdict_counts(value: &json::Value, at: &str) -> Result<crate::rollup::VerdictCounts, json::DecodeError> {
    Ok(crate::rollup::VerdictCounts {
        ok: {
            let at0 = json::nested(at, "ok");
            let member0 = json::member_at(value, at, "ok")?;
            json::integer_at(member0, &at0, "an integer")?
        },
        na: {
            let at1 = json::nested(at, "na");
            let member1 = json::member_at(value, at, "na")?;
            json::integer_at(member1, &at1, "an integer")?
        },
        warn: {
            let at2 = json::nested(at, "warn");
            let member2 = json::member_at(value, at, "warn")?;
            json::integer_at(member2, &at2, "an integer")?
        },
        crit: {
            let at3 = json::nested(at, "crit");
            let member3 = json::member_at(value, at, "crit")?;
            json::integer_at(member3, &at3, "an integer")?
        },
        unknown: {
            let at4 = json::nested(at, "unknown");
            let member4 = json::member_at(value, at, "unknown")?;
            json::integer_at(member4, &at4, "an integer")?
        },
    })
}

/// Writes `thalamus.transition.Transition.State` as JSON.
pub fn encode_thalamus_transition_transition_state(value: &crate::transition::TransitionState, out: &mut String) {
    match value {
        crate::transition::TransitionState::Recorded => json::push_text(out, "Recorded"),
    }
}

/// Reads `thalamus.transition.Transition.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_transition_transition_state(value: &json::Value, at: &str) -> Result<crate::transition::TransitionState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Recorded`")? {
        "Recorded" => crate::transition::TransitionState::Recorded,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Recorded`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `thalamus.transition.TransitionId` as JSON.
pub fn encode_thalamus_transition_transition_id(value: &crate::transition::TransitionId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `thalamus.transition.TransitionId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_thalamus_transition_transition_id(value: &json::Value, at: &str) -> Result<crate::transition::TransitionId, json::DecodeError> {
    Ok(crate::transition::TransitionId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes the event `thalamus.assertion.AssertionAnswered` as JSON.
pub fn encode_event_thalamus_assertion_assertion_answered(value: &crate::assertion::AssertionAnswered, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    json::member(out, "asker");
    encode_thalamus_assertion_asker(&value.asker, out);
    json::member(out, "result");
    encode_thalamus_assertion_assertion_result(&value.result, out);
    json::member(out, "worst");
    encode_thalamus_evaluation_verdict(&value.worst, out);
    out.push('}');
}

/// Writes the event `thalamus.assertion.AssertionAsked` as JSON.
pub fn encode_event_thalamus_assertion_assertion_asked(value: &crate::assertion::AssertionAsked, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    json::member(out, "asker");
    encode_thalamus_assertion_asker(&value.asker, out);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionActivated` as JSON.
pub fn encode_event_thalamus_definition_definition_activated(value: &crate::definition::DefinitionActivated, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionMuted` as JSON.
pub fn encode_event_thalamus_definition_definition_muted(value: &crate::definition::DefinitionMuted, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "until");
    json::push_text(out, &value.until.0);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionProposed` as JSON.
pub fn encode_event_thalamus_definition_definition_proposed(value: &crate::definition::DefinitionProposed, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "revised_by");
    encode_thalamus_definition_author(&value.revised_by, out);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionRetired` as JSON.
pub fn encode_event_thalamus_definition_definition_retired(value: &crate::definition::DefinitionRetired, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionRevised` as JSON.
pub fn encode_event_thalamus_definition_definition_revised(value: &crate::definition::DefinitionRevised, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "previous_rev");
    json::push_integer(out, value.previous_rev);
    json::member(out, "revised_by");
    encode_thalamus_definition_author(&value.revised_by, out);
    out.push('}');
}

/// Writes the event `thalamus.definition.DefinitionUnmuted` as JSON.
pub fn encode_event_thalamus_definition_definition_unmuted(value: &crate::definition::DefinitionUnmuted, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the event `thalamus.dimension.DimensionRegistered` as JSON.
pub fn encode_event_thalamus_dimension_dimension_registered(value: &crate::dimension::DimensionRegistered, out: &mut String) {
    out.push('{');
    json::member(out, "key");
    encode_thalamus_dimension_dimension_key(&value.key, out);
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    json::member(out, "rule");
    encode_thalamus_dimension_dimension_rule(&value.rule, out);
    out.push('}');
}

/// Writes the event `thalamus.dimension.RegistryDeclared` as JSON.
pub fn encode_event_thalamus_dimension_registry_declared(value: &crate::dimension::RegistryDeclared, out: &mut String) {
    out.push('{');
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    out.push('}');
}

/// Writes the event `thalamus.evaluation.EvaluationRecorded` as JSON.
pub fn encode_event_thalamus_evaluation_evaluation_recorded(value: &crate::evaluation::EvaluationRecorded, out: &mut String) {
    out.push('{');
    json::member(out, "evaluation_id");
    encode_thalamus_evaluation_evaluation_id(&value.evaluation_id, out);
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    if let Some(held0) = &value.prev {
        json::member(out, "prev");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    out.push('}');
}

/// Writes the event `thalamus.observation.SampleRecorded` as JSON.
pub fn encode_event_thalamus_observation_sample_recorded(value: &crate::observation::SampleRecorded, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    out.push('}');
}

/// Writes the event `thalamus.rollup.RollupRecorded` as JSON.
pub fn encode_event_thalamus_rollup_rollup_recorded(value: &crate::rollup::RollupRecorded, out: &mut String) {
    out.push('{');
    json::member(out, "rollup_id");
    encode_thalamus_rollup_rollup_id(&value.rollup_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "day");
    json::push_text(out, &value.day.0);
    out.push('}');
}

/// Writes the event `thalamus.transition.TransitionRecorded` as JSON.
pub fn encode_event_thalamus_transition_transition_recorded(value: &crate::transition::TransitionRecorded, out: &mut String) {
    out.push('{');
    json::member(out, "transition_id");
    encode_thalamus_transition_transition_id(&value.transition_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    if let Some(held0) = &value.from_verdict {
        json::member(out, "from_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "to_verdict");
    encode_thalamus_evaluation_verdict(&value.to_verdict, out);
    json::member(out, "settled_ts");
    json::push_text(out, &value.settled_ts.0);
    out.push('}');
}

/// Writes the declared error `thalamus.assertion.AssertionStateConflict` as JSON.
pub fn encode_error_thalamus_assertion_assertion_state_conflict(value: &crate::assertion::AssertionStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "current");
    encode_thalamus_assertion_assertion_state(&value.current, out);
    out.push('}');
}

/// Writes the declared error `thalamus.assertion.NoSignals` as JSON.
pub fn encode_error_thalamus_assertion_no_signals(_value: &crate::assertion::NoSignals, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `thalamus.assertion.ReasonRequired` as JSON.
pub fn encode_error_thalamus_assertion_reason_required(value: &crate::assertion::ReasonRequired, out: &mut String) {
    out.push('{');
    json::member(out, "result");
    encode_thalamus_assertion_assertion_result(&value.result, out);
    out.push('}');
}

/// Writes the declared error `thalamus.assertion.ResultContradictsVerdict` as JSON.
pub fn encode_error_thalamus_assertion_result_contradicts_verdict(value: &crate::assertion::ResultContradictsVerdict, out: &mut String) {
    out.push('{');
    json::member(out, "result");
    encode_thalamus_assertion_assertion_result(&value.result, out);
    json::member(out, "worst");
    encode_thalamus_evaluation_verdict(&value.worst, out);
    out.push('}');
}

/// Writes the declared error `thalamus.assertion.UnknownAssertion` as JSON.
pub fn encode_error_thalamus_assertion_unknown_assertion(value: &crate::assertion::UnknownAssertion, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.DefinitionExists` as JSON.
pub fn encode_error_thalamus_definition_definition_exists(value: &crate::definition::DefinitionExists, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.DefinitionStateConflict` as JSON.
pub fn encode_error_thalamus_definition_definition_state_conflict(value: &crate::definition::DefinitionStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "current");
    encode_thalamus_definition_signal_definition_state(&value.current, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.FixedNumberMissing` as JSON.
pub fn encode_error_thalamus_definition_fixed_number_missing(value: &crate::definition::FixedNumberMissing, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.FixedNumberUnused` as JSON.
pub fn encode_error_thalamus_definition_fixed_number_unused(value: &crate::definition::FixedNumberUnused, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.HostConnectionNotLocal` as JSON.
pub fn encode_error_thalamus_definition_host_connection_not_local(value: &crate::definition::HostConnectionNotLocal, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.IncompleteDefinition` as JSON.
pub fn encode_error_thalamus_definition_incomplete_definition(value: &crate::definition::IncompleteDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.InstantNeedsFixedBaseline` as JSON.
pub fn encode_error_thalamus_definition_instant_needs_fixed_baseline(value: &crate::definition::InstantNeedsFixedBaseline, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.MappingMissing` as JSON.
pub fn encode_error_thalamus_definition_mapping_missing(value: &crate::definition::MappingMissing, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.MappingUnused` as JSON.
pub fn encode_error_thalamus_definition_mapping_unused(value: &crate::definition::MappingUnused, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.MuteExpiryPassed` as JSON.
pub fn encode_error_thalamus_definition_mute_expiry_passed(value: &crate::definition::MuteExpiryPassed, out: &mut String) {
    out.push('{');
    json::member(out, "until");
    json::push_text(out, &value.until.0);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.PersistenceTooSmall` as JSON.
pub fn encode_error_thalamus_definition_persistence_too_small(value: &crate::definition::PersistenceTooSmall, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.RetiredDefinition` as JSON.
pub fn encode_error_thalamus_definition_retired_definition(value: &crate::definition::RetiredDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.RollingWindowMissing` as JSON.
pub fn encode_error_thalamus_definition_rolling_window_missing(value: &crate::definition::RollingWindowMissing, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.RollingWindowUnused` as JSON.
pub fn encode_error_thalamus_definition_rolling_window_unused(value: &crate::definition::RollingWindowUnused, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.ThresholdsUnused` as JSON.
pub fn encode_error_thalamus_definition_thresholds_unused(value: &crate::definition::ThresholdsUnused, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.definition.UnknownDefinition` as JSON.
pub fn encode_error_thalamus_definition_unknown_definition(value: &crate::definition::UnknownDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.dimension.DimensionExists` as JSON.
pub fn encode_error_thalamus_dimension_dimension_exists(value: &crate::dimension::DimensionExists, out: &mut String) {
    out.push('{');
    json::member(out, "key");
    encode_thalamus_dimension_dimension_key(&value.key, out);
    out.push('}');
}

/// Writes the declared error `thalamus.dimension.RegistryExists` as JSON.
pub fn encode_error_thalamus_dimension_registry_exists(value: &crate::dimension::RegistryExists, out: &mut String) {
    out.push('{');
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.dimension.UnknownParent` as JSON.
pub fn encode_error_thalamus_dimension_unknown_parent(value: &crate::dimension::UnknownParent, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.parent {
        json::member(out, "parent");
        encode_thalamus_dimension_dimension_key(&*held0, out);
    }
    out.push('}');
}

/// Writes the declared error `thalamus.dimension.UnknownRegistry` as JSON.
pub fn encode_error_thalamus_dimension_unknown_registry(value: &crate::dimension::UnknownRegistry, out: &mut String) {
    out.push('{');
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.evaluation.ReasonRequired` as JSON.
pub fn encode_error_thalamus_evaluation_reason_required(value: &crate::evaluation::ReasonRequired, out: &mut String) {
    out.push('{');
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    out.push('}');
}

/// Writes the declared error `thalamus.evaluation.ScoreNotAdmitted` as JSON.
pub fn encode_error_thalamus_evaluation_score_not_admitted(value: &crate::evaluation::ScoreNotAdmitted, out: &mut String) {
    out.push('{');
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    out.push('}');
}

/// Writes the declared error `thalamus.evaluation.UnknownSample` as JSON.
pub fn encode_error_thalamus_evaluation_unknown_sample(value: &crate::evaluation::UnknownSample, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.observation.SignalNotActive` as JSON.
pub fn encode_error_thalamus_observation_signal_not_active(value: &crate::observation::SignalNotActive, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    out.push('}');
}

/// Writes the declared error `thalamus.observation.UnknownSignal` as JSON.
pub fn encode_error_thalamus_observation_unknown_signal(value: &crate::observation::UnknownSignal, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    out.push('}');
}

/// Writes the declared error `thalamus.rollup.RollupExists` as JSON.
pub fn encode_error_thalamus_rollup_rollup_exists(value: &crate::rollup::RollupExists, out: &mut String) {
    out.push('{');
    json::member(out, "rollup_id");
    encode_thalamus_rollup_rollup_id(&value.rollup_id, out);
    out.push('}');
}

/// Writes the declared error `thalamus.rollup.UnknownSignal` as JSON.
pub fn encode_error_thalamus_rollup_unknown_signal(value: &crate::rollup::UnknownSignal, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    out.push('}');
}

/// Writes the declared error `thalamus.transition.UnknownSignal` as JSON.
pub fn encode_error_thalamus_transition_unknown_signal(value: &crate::transition::UnknownSignal, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.assertion.Assertions` as JSON.
pub fn encode_view_thalamus_assertion_assertions(value: &crate::assertion::Assertions, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    json::member(out, "signals");
    out.push('[');
    for (index0, item0) in value.signals.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_definition_signal_id(&*item0, out);
    }
    out.push(']');
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "after");
    json::push_text(out, &value.after.0);
    json::member(out, "horizon");
    json::push_text(out, &value.horizon.0);
    json::member(out, "within");
    encode_thalamus_assertion_within(&value.within, out);
    json::member(out, "asker");
    encode_thalamus_assertion_asker(&value.asker, out);
    if let Some(held0) = &value.result {
        json::member(out, "result");
        encode_thalamus_assertion_assertion_result(&*held0, out);
    }
    if let Some(held0) = &value.worst {
        json::member(out, "worst");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    json::member(out, "state");
    encode_thalamus_assertion_assertion_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.assertion.OpenAssertions` as JSON.
pub fn encode_view_thalamus_assertion_open_assertions(value: &crate::assertion::OpenAssertions, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    json::member(out, "signals");
    out.push('[');
    for (index0, item0) in value.signals.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_definition_signal_id(&*item0, out);
    }
    out.push(']');
    json::member(out, "after");
    json::push_text(out, &value.after.0);
    json::member(out, "horizon");
    json::push_text(out, &value.horizon.0);
    json::member(out, "asker");
    encode_thalamus_assertion_asker(&value.asker, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.definition.ActiveDefinitions` as JSON.
pub fn encode_view_thalamus_definition_active_definitions(value: &crate::definition::ActiveDefinitions, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    json::member(out, "family");
    encode_thalamus_definition_family(&value.family, out);
    json::member(out, "kind");
    encode_thalamus_definition_signal_kind(&value.kind, out);
    if let Some(held0) = &value.unit {
        json::member(out, "unit");
        encode_thalamus_definition_unit(&*held0, out);
    }
    if let Some(held0) = &value.dimensions {
        json::member(out, "dimensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_thalamus_dimension_dimension_key(&*item1, out);
        }
        out.push(']');
    }
    json::member(out, "source");
    encode_thalamus_definition_source(&value.source, out);
    json::member(out, "cadence");
    json::push_text(out, &value.cadence.0);
    if let Some(held0) = &value.owner {
        json::member(out, "owner");
        encode_thalamus_definition_owner(&*held0, out);
    }
    json::member(out, "rev");
    json::push_integer(out, value.rev);
    out.push('}');
}

/// Writes one row of the view `thalamus.definition.Definitions` as JSON.
pub fn encode_view_thalamus_definition_definitions(value: &crate::definition::Definitions, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    json::member(out, "family");
    encode_thalamus_definition_family(&value.family, out);
    json::member(out, "kind");
    encode_thalamus_definition_signal_kind(&value.kind, out);
    if let Some(held0) = &value.unit {
        json::member(out, "unit");
        encode_thalamus_definition_unit(&*held0, out);
    }
    if let Some(held0) = &value.dimensions {
        json::member(out, "dimensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_thalamus_dimension_dimension_key(&*item1, out);
        }
        out.push(']');
    }
    json::member(out, "source");
    encode_thalamus_definition_source(&value.source, out);
    json::member(out, "extractor");
    encode_thalamus_definition_extractor_id(&value.extractor, out);
    json::member(out, "query");
    out.push('{');
    for (index0, (key0, item0)) in value.query.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_definition_query_text(&*item0, out);
    }
    out.push('}');
    json::member(out, "window");
    json::push_text(out, &value.window.0);
    json::member(out, "cadence");
    json::push_text(out, &value.cadence.0);
    json::member(out, "history");
    encode_thalamus_definition_history(&value.history, out);
    if let Some(held0) = &value.denominator {
        json::member(out, "denominator");
        encode_thalamus_definition_denominator_spec(&*held0, out);
    }
    if let Some(held0) = &value.objective {
        json::member(out, "objective");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.owner {
        json::member(out, "owner");
        encode_thalamus_definition_owner(&*held0, out);
    }
    json::member(out, "links");
    encode_thalamus_definition_links(&value.links, out);
    json::member(out, "rev");
    json::push_integer(out, value.rev);
    json::member(out, "revised_at");
    json::push_text(out, &value.revised_at.0);
    json::member(out, "revised_by");
    encode_thalamus_definition_author(&value.revised_by, out);
    if let Some(held0) = &value.mute_reason {
        json::member(out, "mute_reason");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.mute_until {
        json::member(out, "mute_until");
        json::push_text(out, &(*held0).0);
    }
    json::member(out, "state");
    encode_thalamus_definition_signal_definition_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.dimension.Dimensions` as JSON.
pub fn encode_view_thalamus_dimension_dimensions(value: &crate::dimension::Dimensions, out: &mut String) {
    out.push('{');
    json::member(out, "key");
    encode_thalamus_dimension_dimension_key(&value.key, out);
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    json::member(out, "values");
    encode_thalamus_dimension_value_source(&value.values, out);
    json::member(out, "rule");
    encode_thalamus_dimension_dimension_rule(&value.rule, out);
    if let Some(held0) = &value.parent {
        json::member(out, "parent");
        encode_thalamus_dimension_dimension_key(&*held0, out);
    }
    if let Some(held0) = &value.otel_attribute {
        json::member(out, "otel_attribute");
        json::push_text(out, &*held0);
    }
    json::member(out, "state");
    encode_thalamus_dimension_dimension_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.dimension.Registries` as JSON.
pub fn encode_view_thalamus_dimension_registries(value: &crate::dimension::Registries, out: &mut String) {
    out.push('{');
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    json::member(out, "state");
    encode_thalamus_dimension_dimension_registry_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.evaluation.Evaluations` as JSON.
pub fn encode_view_thalamus_evaluation_evaluations(value: &crate::evaluation::Evaluations, out: &mut String) {
    out.push('{');
    json::member(out, "evaluation_id");
    encode_thalamus_evaluation_evaluation_id(&value.evaluation_id, out);
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    if let Some(held0) = &value.prev {
        json::member(out, "prev");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    if let Some(held0) = &value.score {
        json::member(out, "score");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.baseline {
        json::member(out, "baseline");
        encode_thalamus_evaluation_baseline(&*held0, out);
    }
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    json::member(out, "state");
    encode_thalamus_evaluation_signal_evaluation_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.evaluation.SeriesVerdicts` as JSON.
pub fn encode_view_thalamus_evaluation_series_verdicts(value: &crate::evaluation::SeriesVerdicts, out: &mut String) {
    out.push('{');
    json::member(out, "evaluation_id");
    encode_thalamus_evaluation_evaluation_id(&value.evaluation_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    if let Some(held0) = &value.prev {
        json::member(out, "prev");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the parameters of `thalamus.evaluation.SeriesVerdicts` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_thalamus_evaluation_series_verdicts(value: &json::Value, at: &str) -> Result<(crate::observation::SeriesKey,), json::DecodeError> {
    Ok((
        {
            let at0 = json::nested(at, "series");
            let member0 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member0, &at0)?
        },
    ))
}

/// Writes one row of the view `thalamus.evaluation.VerdictChanges` as JSON.
pub fn encode_view_thalamus_evaluation_verdict_changes(value: &crate::evaluation::VerdictChanges, out: &mut String) {
    out.push('{');
    json::member(out, "evaluation_id");
    encode_thalamus_evaluation_evaluation_id(&value.evaluation_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    if let Some(held0) = &value.prev {
        json::member(out, "prev");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Writes one row of the view `thalamus.observation.Samples` as JSON.
pub fn encode_view_thalamus_observation_samples(value: &crate::observation::Samples, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "window");
    encode_thalamus_observation_time_window(&value.window, out);
    json::member(out, "reading");
    encode_thalamus_observation_reading(&value.reading, out);
    if let Some(held0) = &value.text {
        json::member(out, "text");
        json::push_text(out, &*held0);
    }
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    if let Some(held0) = &value.denominator {
        json::member(out, "denominator");
        encode_thalamus_observation_denominator(&*held0, out);
    }
    json::member(out, "provenance");
    encode_thalamus_observation_provenance(&value.provenance, out);
    json::member(out, "state");
    encode_thalamus_observation_signal_sample_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.observation.SeriesSamples` as JSON.
pub fn encode_view_thalamus_observation_series_samples(value: &crate::observation::SeriesSamples, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "window");
    encode_thalamus_observation_time_window(&value.window, out);
    json::member(out, "reading");
    encode_thalamus_observation_reading(&value.reading, out);
    if let Some(held0) = &value.text {
        json::member(out, "text");
        json::push_text(out, &*held0);
    }
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    if let Some(held0) = &value.denominator {
        json::member(out, "denominator");
        encode_thalamus_observation_denominator(&*held0, out);
    }
    out.push('}');
}

/// Reads the parameters of `thalamus.observation.SeriesSamples` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_thalamus_observation_series_samples(value: &json::Value, at: &str) -> Result<(crate::observation::SeriesKey,), json::DecodeError> {
    Ok((
        {
            let at0 = json::nested(at, "series");
            let member0 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member0, &at0)?
        },
    ))
}

/// Writes one row of the view `thalamus.rollup.Rollups` as JSON.
pub fn encode_view_thalamus_rollup_rollups(value: &crate::rollup::Rollups, out: &mut String) {
    out.push('{');
    json::member(out, "rollup_id");
    encode_thalamus_rollup_rollup_id(&value.rollup_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "day");
    json::push_text(out, &value.day.0);
    json::member(out, "n");
    json::push_integer(out, value.n);
    json::member(out, "state");
    encode_thalamus_rollup_daily_rollup_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `thalamus.rollup.SeriesRollups` as JSON.
pub fn encode_view_thalamus_rollup_series_rollups(value: &crate::rollup::SeriesRollups, out: &mut String) {
    out.push('{');
    json::member(out, "rollup_id");
    encode_thalamus_rollup_rollup_id(&value.rollup_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "day");
    json::push_text(out, &value.day.0);
    json::member(out, "n");
    json::push_integer(out, value.n);
    json::member(out, "unknown");
    json::push_integer(out, value.unknown);
    json::member(out, "partial");
    json::push_integer(out, value.partial);
    if let Some(held0) = &value.min {
        json::member(out, "min");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.p50 {
        json::member(out, "p50");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.max {
        json::member(out, "max");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.last {
        json::member(out, "last");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.worst_verdict {
        json::member(out, "worst_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "verdict_counts");
    encode_thalamus_rollup_verdict_counts(&value.verdict_counts, out);
    if let Some(held0) = &value.min_score {
        json::member(out, "min_score");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.max_score {
        json::member(out, "max_score");
        json::push_text(out, &(*held0).0);
    }
    out.push('}');
}

/// Reads the parameters of `thalamus.rollup.SeriesRollups` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_thalamus_rollup_series_rollups(value: &json::Value, at: &str) -> Result<(crate::observation::SeriesKey,), json::DecodeError> {
    Ok((
        {
            let at0 = json::nested(at, "series");
            let member0 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member0, &at0)?
        },
    ))
}

/// Writes one row of the view `thalamus.transition.SeriesTransitions` as JSON.
pub fn encode_view_thalamus_transition_series_transitions(value: &crate::transition::SeriesTransitions, out: &mut String) {
    out.push('{');
    json::member(out, "transition_id");
    encode_thalamus_transition_transition_id(&value.transition_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    if let Some(held0) = &value.from_verdict {
        json::member(out, "from_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "to_verdict");
    encode_thalamus_evaluation_verdict(&value.to_verdict, out);
    json::member(out, "first_ts");
    json::push_text(out, &value.first_ts.0);
    json::member(out, "settled_ts");
    json::push_text(out, &value.settled_ts.0);
    if let Some(held0) = &value.settling_score {
        json::member(out, "settling_score");
        json::push_text(out, &(*held0).0);
    }
    json::member(out, "evaluations");
    out.push('[');
    for (index0, item0) in value.evaluations.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_evaluation_evaluation_id(&*item0, out);
    }
    out.push(']');
    out.push('}');
}

/// Reads the parameters of `thalamus.transition.SeriesTransitions` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_thalamus_transition_series_transitions(value: &json::Value, at: &str) -> Result<(crate::observation::SeriesKey,), json::DecodeError> {
    Ok((
        {
            let at0 = json::nested(at, "series");
            let member0 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member0, &at0)?
        },
    ))
}

/// Writes one row of the view `thalamus.transition.Transitions` as JSON.
pub fn encode_view_thalamus_transition_transitions(value: &crate::transition::Transitions, out: &mut String) {
    out.push('{');
    json::member(out, "transition_id");
    encode_thalamus_transition_transition_id(&value.transition_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    if let Some(held0) = &value.from_verdict {
        json::member(out, "from_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "to_verdict");
    encode_thalamus_evaluation_verdict(&value.to_verdict, out);
    json::member(out, "settled_ts");
    json::push_text(out, &value.settled_ts.0);
    json::member(out, "state");
    encode_thalamus_transition_transition_state(&value.state, out);
    out.push('}');
}

/// Writes the input of `thalamus.assertion.AnswerAssertion` as JSON.
pub fn encode_command_thalamus_assertion_answer_assertion(value: &crate::assertion::AnswerAssertion, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    json::member(out, "result");
    encode_thalamus_assertion_assertion_result(&value.result, out);
    json::member(out, "worst");
    encode_thalamus_evaluation_verdict(&value.worst, out);
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `thalamus.assertion.AnswerAssertion` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_assertion_answer_assertion(value: &json::Value, at: &str) -> Result<crate::assertion::AnswerAssertion, json::DecodeError> {
    Ok(crate::assertion::AnswerAssertion {
        assertion_id: {
            let at0 = json::nested(at, "assertion_id");
            let member0 = json::member_at(value, at, "assertion_id")?;
            decode_thalamus_assertion_assertion_id(member0, &at0)?
        },
        result: {
            let at1 = json::nested(at, "result");
            let member1 = json::member_at(value, at, "result")?;
            decode_thalamus_assertion_assertion_result(member1, &at1)?
        },
        worst: {
            let at2 = json::nested(at, "worst");
            let member2 = json::member_at(value, at, "worst")?;
            decode_thalamus_evaluation_verdict(member2, &at2)?
        },
        reason: match value.member("reason") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "reason");
                Some(json::text_at(member3, &at3, "a string")?.to_owned())
            }
        },
    })
}

/// Writes the outcome of `thalamus.assertion.AnswerAssertion` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_assertion_answer_assertion(value: &crate::assertion::AnswerAssertionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::assertion::AnswerAssertionOutcome::ReasonMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "reason-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.assertion.ReasonRequired");
            json::member(out, "payload");
            encode_error_thalamus_assertion_reason_required(error, out);
            out.push('}');
        }
        crate::assertion::AnswerAssertionOutcome::ContradictsVerdict { error } => {
            json::member(out, "outcome");
            json::push_text(out, "contradicts-verdict");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.assertion.ResultContradictsVerdict");
            json::member(out, "payload");
            encode_error_thalamus_assertion_result_contradicts_verdict(error, out);
            out.push('}');
        }
        crate::assertion::AnswerAssertionOutcome::Answered { assertion_answered } => {
            json::member(out, "outcome");
            json::push_text(out, "answered");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.assertion.AssertionAnswered");
            json::member(out, "payload");
            encode_event_thalamus_assertion_assertion_answered(assertion_answered, out);
            out.push('}');
            out.push(']');
        }
        crate::assertion::AnswerAssertionOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.assertion.AssertionStateConflict");
            json::member(out, "payload");
            encode_error_thalamus_assertion_assertion_state_conflict(error, out);
            out.push('}');
        }
        crate::assertion::AnswerAssertionOutcome::NoSuchAssertion { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-assertion");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.assertion.UnknownAssertion");
            json::member(out, "payload");
            encode_error_thalamus_assertion_unknown_assertion(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.assertion.AskAssertion` as JSON.
pub fn encode_command_thalamus_assertion_ask_assertion(value: &crate::assertion::AskAssertion, out: &mut String) {
    out.push('{');
    json::member(out, "signals");
    out.push('[');
    for (index0, item0) in value.signals.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_definition_signal_id(&*item0, out);
    }
    out.push(']');
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "after");
    json::push_text(out, &value.after.0);
    json::member(out, "horizon");
    json::push_text(out, &value.horizon.0);
    json::member(out, "within");
    encode_thalamus_assertion_within(&value.within, out);
    json::member(out, "asker");
    encode_thalamus_assertion_asker(&value.asker, out);
    out.push('}');
}

/// Reads the input of `thalamus.assertion.AskAssertion` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_assertion_ask_assertion(value: &json::Value, at: &str) -> Result<crate::assertion::AskAssertion, json::DecodeError> {
    Ok(crate::assertion::AskAssertion {
        signals: {
            let at0 = json::nested(at, "signals");
            let member0 = json::member_at(value, at, "signals")?;
            {
                let mut items0 = Vec::new();
                for (index0, element0) in json::items_at(member0, &at0, "an array")?.iter().enumerate() {
                    let nested0 = json::nested(&at0, &index0.to_string());
                    items0.push(decode_thalamus_definition_signal_id(element0, &nested0)?);
                }
                items0
            }
        },
        dims: {
            let at1 = json::nested(at, "dims");
            let member1 = json::member_at(value, at, "dims")?;
            {
                let mut entries1 = std::collections::BTreeMap::new();
                for (key1, element1) in json::members_at(member1, &at1, "an object")? {
                    let nested1 = json::nested(&at1, key1);
                    entries1.insert(key1.clone(), decode_thalamus_dimension_dimension_value(element1, &nested1)?);
                }
                entries1
            }
        },
        after: {
            let at2 = json::nested(at, "after");
            let member2 = json::member_at(value, at, "after")?;
            crate::primitives::Timestamp(json::text_at(member2, &at2, "an RFC 3339 instant")?.to_owned())
        },
        horizon: {
            let at3 = json::nested(at, "horizon");
            let member3 = json::member_at(value, at, "horizon")?;
            crate::primitives::Duration(json::text_at(member3, &at3, "an ISO 8601 duration")?.to_owned())
        },
        within: {
            let at4 = json::nested(at, "within");
            let member4 = json::member_at(value, at, "within")?;
            decode_thalamus_assertion_within(member4, &at4)?
        },
        asker: {
            let at5 = json::nested(at, "asker");
            let member5 = json::member_at(value, at, "asker")?;
            decode_thalamus_assertion_asker(member5, &at5)?
        },
    })
}

/// Writes the response of `thalamus.assertion.AskAssertion` as JSON.
pub fn encode_response_thalamus_assertion_ask_assertion(value: &crate::assertion::AskAssertionResponse, out: &mut String) {
    out.push('{');
    json::member(out, "assertion_id");
    encode_thalamus_assertion_assertion_id(&value.assertion_id, out);
    out.push('}');
}

/// Writes the outcome of `thalamus.assertion.AskAssertion` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_assertion_ask_assertion(value: &crate::assertion::AskAssertionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::assertion::AskAssertionOutcome::NoSignals { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-signals");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.assertion.NoSignals");
            json::member(out, "payload");
            encode_error_thalamus_assertion_no_signals(error, out);
            out.push('}');
        }
        crate::assertion::AskAssertionOutcome::Asked { assertion_asked, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "asked");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.assertion.AssertionAsked");
            json::member(out, "payload");
            encode_event_thalamus_assertion_assertion_asked(assertion_asked, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.ActivateDefinition` as JSON.
pub fn encode_command_thalamus_definition_activate_definition(value: &crate::definition::ActivateDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Reads the input of `thalamus.definition.ActivateDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_activate_definition(value: &json::Value, at: &str) -> Result<crate::definition::ActivateDefinition, json::DecodeError> {
    Ok(crate::definition::ActivateDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `thalamus.definition.ActivateDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_activate_definition(value: &crate::definition::ActivateDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::ActivateDefinitionOutcome::Incomplete { error } => {
            json::member(out, "outcome");
            json::push_text(out, "incomplete");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.IncompleteDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_incomplete_definition(error, out);
            out.push('}');
        }
        crate::definition::ActivateDefinitionOutcome::Activated { definition_activated } => {
            json::member(out, "outcome");
            json::push_text(out, "activated");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionActivated");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_activated(definition_activated, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::ActivateDefinitionOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.DefinitionStateConflict");
            json::member(out, "payload");
            encode_error_thalamus_definition_definition_state_conflict(error, out);
            out.push('}');
        }
        crate::definition::ActivateDefinitionOutcome::NoSuchDefinition { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-definition");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.UnknownDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_unknown_definition(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.MuteDefinition` as JSON.
pub fn encode_command_thalamus_definition_mute_definition(value: &crate::definition::MuteDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "until");
    json::push_text(out, &value.until.0);
    out.push('}');
}

/// Reads the input of `thalamus.definition.MuteDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_mute_definition(value: &json::Value, at: &str) -> Result<crate::definition::MuteDefinition, json::DecodeError> {
    Ok(crate::definition::MuteDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
        reason: {
            let at1 = json::nested(at, "reason");
            let member1 = json::member_at(value, at, "reason")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        until: {
            let at2 = json::nested(at, "until");
            let member2 = json::member_at(value, at, "until")?;
            crate::primitives::Timestamp(json::text_at(member2, &at2, "an RFC 3339 instant")?.to_owned())
        },
    })
}

/// Writes the outcome of `thalamus.definition.MuteDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_mute_definition(value: &crate::definition::MuteDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::MuteDefinitionOutcome::ExpiryPassed { error } => {
            json::member(out, "outcome");
            json::push_text(out, "expiry-passed");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.MuteExpiryPassed");
            json::member(out, "payload");
            encode_error_thalamus_definition_mute_expiry_passed(error, out);
            out.push('}');
        }
        crate::definition::MuteDefinitionOutcome::Muted { definition_muted } => {
            json::member(out, "outcome");
            json::push_text(out, "muted");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionMuted");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_muted(definition_muted, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::MuteDefinitionOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.DefinitionStateConflict");
            json::member(out, "payload");
            encode_error_thalamus_definition_definition_state_conflict(error, out);
            out.push('}');
        }
        crate::definition::MuteDefinitionOutcome::NoSuchDefinition { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-definition");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.UnknownDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_unknown_definition(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.ProposeDefinition` as JSON.
pub fn encode_command_thalamus_definition_propose_definition(value: &crate::definition::ProposeDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    json::member(out, "family");
    encode_thalamus_definition_family(&value.family, out);
    json::member(out, "kind");
    encode_thalamus_definition_signal_kind(&value.kind, out);
    if let Some(held0) = &value.unit {
        json::member(out, "unit");
        encode_thalamus_definition_unit(&*held0, out);
    }
    if let Some(held0) = &value.dimensions {
        json::member(out, "dimensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_thalamus_dimension_dimension_key(&*item1, out);
        }
        out.push(']');
    }
    json::member(out, "source");
    encode_thalamus_definition_source(&value.source, out);
    json::member(out, "extractor");
    encode_thalamus_definition_extractor_id(&value.extractor, out);
    json::member(out, "query");
    out.push('{');
    for (index0, (key0, item0)) in value.query.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_definition_query_text(&*item0, out);
    }
    out.push('}');
    json::member(out, "window");
    json::push_text(out, &value.window.0);
    json::member(out, "cadence");
    json::push_text(out, &value.cadence.0);
    json::member(out, "history");
    encode_thalamus_definition_history(&value.history, out);
    if let Some(held0) = &value.denominator {
        json::member(out, "denominator");
        encode_thalamus_definition_denominator_spec(&*held0, out);
    }
    json::member(out, "evaluation");
    encode_thalamus_definition_evaluation_spec(&value.evaluation, out);
    if let Some(held0) = &value.persistence {
        json::member(out, "persistence");
        encode_thalamus_definition_persistence(&*held0, out);
    }
    if let Some(held0) = &value.objective {
        json::member(out, "objective");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.owner {
        json::member(out, "owner");
        encode_thalamus_definition_owner(&*held0, out);
    }
    json::member(out, "links");
    encode_thalamus_definition_links(&value.links, out);
    json::member(out, "author");
    encode_thalamus_definition_author(&value.author, out);
    out.push('}');
}

/// Reads the input of `thalamus.definition.ProposeDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_propose_definition(value: &json::Value, at: &str) -> Result<crate::definition::ProposeDefinition, json::DecodeError> {
    Ok(crate::definition::ProposeDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
        title: {
            let at1 = json::nested(at, "title");
            let member1 = json::member_at(value, at, "title")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        family: {
            let at2 = json::nested(at, "family");
            let member2 = json::member_at(value, at, "family")?;
            decode_thalamus_definition_family(member2, &at2)?
        },
        kind: {
            let at3 = json::nested(at, "kind");
            let member3 = json::member_at(value, at, "kind")?;
            decode_thalamus_definition_signal_kind(member3, &at3)?
        },
        unit: match value.member("unit") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "unit");
                Some(decode_thalamus_definition_unit(member4, &at4)?)
            }
        },
        dimensions: match value.member("dimensions") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "dimensions");
                Some({
                let mut items5 = Vec::new();
                for (index5, element5) in json::items_at(member5, &at5, "an array")?.iter().enumerate() {
                    let nested5 = json::nested(&at5, &index5.to_string());
                    items5.push(decode_thalamus_dimension_dimension_key(element5, &nested5)?);
                }
                items5
            })
            }
        },
        source: {
            let at6 = json::nested(at, "source");
            let member6 = json::member_at(value, at, "source")?;
            decode_thalamus_definition_source(member6, &at6)?
        },
        extractor: {
            let at7 = json::nested(at, "extractor");
            let member7 = json::member_at(value, at, "extractor")?;
            decode_thalamus_definition_extractor_id(member7, &at7)?
        },
        query: {
            let at8 = json::nested(at, "query");
            let member8 = json::member_at(value, at, "query")?;
            {
                let mut entries8 = std::collections::BTreeMap::new();
                for (key8, element8) in json::members_at(member8, &at8, "an object")? {
                    let nested8 = json::nested(&at8, key8);
                    entries8.insert(key8.clone(), decode_thalamus_definition_query_text(element8, &nested8)?);
                }
                entries8
            }
        },
        window: {
            let at9 = json::nested(at, "window");
            let member9 = json::member_at(value, at, "window")?;
            crate::primitives::Duration(json::text_at(member9, &at9, "an ISO 8601 duration")?.to_owned())
        },
        cadence: {
            let at10 = json::nested(at, "cadence");
            let member10 = json::member_at(value, at, "cadence")?;
            crate::primitives::Duration(json::text_at(member10, &at10, "an ISO 8601 duration")?.to_owned())
        },
        history: {
            let at11 = json::nested(at, "history");
            let member11 = json::member_at(value, at, "history")?;
            decode_thalamus_definition_history(member11, &at11)?
        },
        denominator: match value.member("denominator") {
            None | Some(json::Value::Null) => None,
            Some(member12) => {
                let at12 = json::nested(at, "denominator");
                Some(decode_thalamus_definition_denominator_spec(member12, &at12)?)
            }
        },
        evaluation: {
            let at13 = json::nested(at, "evaluation");
            let member13 = json::member_at(value, at, "evaluation")?;
            decode_thalamus_definition_evaluation_spec(member13, &at13)?
        },
        persistence: match value.member("persistence") {
            None | Some(json::Value::Null) => None,
            Some(member14) => {
                let at14 = json::nested(at, "persistence");
                Some(decode_thalamus_definition_persistence(member14, &at14)?)
            }
        },
        objective: match value.member("objective") {
            None | Some(json::Value::Null) => None,
            Some(member15) => {
                let at15 = json::nested(at, "objective");
                Some(json::value_at(member15, &at15))
            }
        },
        owner: match value.member("owner") {
            None | Some(json::Value::Null) => None,
            Some(member16) => {
                let at16 = json::nested(at, "owner");
                Some(decode_thalamus_definition_owner(member16, &at16)?)
            }
        },
        links: {
            let at17 = json::nested(at, "links");
            let member17 = json::member_at(value, at, "links")?;
            decode_thalamus_definition_links(member17, &at17)?
        },
        author: {
            let at18 = json::nested(at, "author");
            let member18 = json::member_at(value, at, "author")?;
            decode_thalamus_definition_author(member18, &at18)?
        },
    })
}

/// Writes the outcome of `thalamus.definition.ProposeDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_propose_definition(value: &crate::definition::ProposeDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::ProposeDefinitionOutcome::HostNotLocal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "host-not-local");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.HostConnectionNotLocal");
            json::member(out, "payload");
            encode_error_thalamus_definition_host_connection_not_local(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::InstantNotFixed { error } => {
            json::member(out, "outcome");
            json::push_text(out, "instant-not-fixed");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.InstantNeedsFixedBaseline");
            json::member(out, "payload");
            encode_error_thalamus_definition_instant_needs_fixed_baseline(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::RollingNMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "rolling-n-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.RollingWindowMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_rolling_window_missing(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::RollingNUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "rolling-n-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.RollingWindowUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_rolling_window_unused(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::FixedNumberMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "fixed-number-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.FixedNumberMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_fixed_number_missing(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::FixedNumberUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "fixed-number-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.FixedNumberUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_fixed_number_unused(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::PersistenceBelowOne { error } => {
            json::member(out, "outcome");
            json::push_text(out, "persistence-below-one");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.PersistenceTooSmall");
            json::member(out, "payload");
            encode_error_thalamus_definition_persistence_too_small(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::MappingMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "mapping-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.MappingMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_mapping_missing(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::MappingUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "mapping-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.MappingUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_mapping_unused(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::ThresholdsUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "thresholds-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.ThresholdsUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_thresholds_unused(error, out);
            out.push('}');
        }
        crate::definition::ProposeDefinitionOutcome::Proposed { definition_proposed } => {
            json::member(out, "outcome");
            json::push_text(out, "proposed");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionProposed");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_proposed(definition_proposed, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::ProposeDefinitionOutcome::AlreadyDefined { error } => {
            json::member(out, "outcome");
            json::push_text(out, "already-defined");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.DefinitionExists");
            json::member(out, "payload");
            encode_error_thalamus_definition_definition_exists(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.RetireDefinition` as JSON.
pub fn encode_command_thalamus_definition_retire_definition(value: &crate::definition::RetireDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Reads the input of `thalamus.definition.RetireDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_retire_definition(value: &json::Value, at: &str) -> Result<crate::definition::RetireDefinition, json::DecodeError> {
    Ok(crate::definition::RetireDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `thalamus.definition.RetireDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_retire_definition(value: &crate::definition::RetireDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::RetireDefinitionOutcome::Retired { definition_retired } => {
            json::member(out, "outcome");
            json::push_text(out, "retired");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionRetired");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_retired(definition_retired, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::RetireDefinitionOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.DefinitionStateConflict");
            json::member(out, "payload");
            encode_error_thalamus_definition_definition_state_conflict(error, out);
            out.push('}');
        }
        crate::definition::RetireDefinitionOutcome::NoSuchDefinition { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-definition");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.UnknownDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_unknown_definition(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.ReviseDefinition` as JSON.
pub fn encode_command_thalamus_definition_revise_definition(value: &crate::definition::ReviseDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    json::member(out, "family");
    encode_thalamus_definition_family(&value.family, out);
    json::member(out, "kind");
    encode_thalamus_definition_signal_kind(&value.kind, out);
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    json::member(out, "dimensions");
    out.push('[');
    for (index0, item0) in value.dimensions.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_dimension_dimension_key(&*item0, out);
    }
    out.push(']');
    json::member(out, "source");
    encode_thalamus_definition_source(&value.source, out);
    json::member(out, "extractor");
    encode_thalamus_definition_extractor_id(&value.extractor, out);
    json::member(out, "query");
    out.push('{');
    for (index0, (key0, item0)) in value.query.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_definition_query_text(&*item0, out);
    }
    out.push('}');
    json::member(out, "window");
    json::push_text(out, &value.window.0);
    json::member(out, "cadence");
    json::push_text(out, &value.cadence.0);
    json::member(out, "history");
    encode_thalamus_definition_history(&value.history, out);
    json::member(out, "denominator");
    encode_thalamus_definition_denominator_spec(&value.denominator, out);
    json::member(out, "evaluation");
    encode_thalamus_definition_evaluation_spec(&value.evaluation, out);
    if let Some(held0) = &value.persistence {
        json::member(out, "persistence");
        encode_thalamus_definition_persistence(&*held0, out);
    }
    if let Some(held0) = &value.objective {
        json::member(out, "objective");
        json::push_value(out, &*held0);
    }
    json::member(out, "owner");
    encode_thalamus_definition_owner(&value.owner, out);
    json::member(out, "links");
    encode_thalamus_definition_links(&value.links, out);
    json::member(out, "author");
    encode_thalamus_definition_author(&value.author, out);
    out.push('}');
}

/// Reads the input of `thalamus.definition.ReviseDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_revise_definition(value: &json::Value, at: &str) -> Result<crate::definition::ReviseDefinition, json::DecodeError> {
    Ok(crate::definition::ReviseDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
        title: {
            let at1 = json::nested(at, "title");
            let member1 = json::member_at(value, at, "title")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        family: {
            let at2 = json::nested(at, "family");
            let member2 = json::member_at(value, at, "family")?;
            decode_thalamus_definition_family(member2, &at2)?
        },
        kind: {
            let at3 = json::nested(at, "kind");
            let member3 = json::member_at(value, at, "kind")?;
            decode_thalamus_definition_signal_kind(member3, &at3)?
        },
        unit: {
            let at4 = json::nested(at, "unit");
            let member4 = json::member_at(value, at, "unit")?;
            decode_thalamus_definition_unit(member4, &at4)?
        },
        dimensions: {
            let at5 = json::nested(at, "dimensions");
            let member5 = json::member_at(value, at, "dimensions")?;
            {
                let mut items5 = Vec::new();
                for (index5, element5) in json::items_at(member5, &at5, "an array")?.iter().enumerate() {
                    let nested5 = json::nested(&at5, &index5.to_string());
                    items5.push(decode_thalamus_dimension_dimension_key(element5, &nested5)?);
                }
                items5
            }
        },
        source: {
            let at6 = json::nested(at, "source");
            let member6 = json::member_at(value, at, "source")?;
            decode_thalamus_definition_source(member6, &at6)?
        },
        extractor: {
            let at7 = json::nested(at, "extractor");
            let member7 = json::member_at(value, at, "extractor")?;
            decode_thalamus_definition_extractor_id(member7, &at7)?
        },
        query: {
            let at8 = json::nested(at, "query");
            let member8 = json::member_at(value, at, "query")?;
            {
                let mut entries8 = std::collections::BTreeMap::new();
                for (key8, element8) in json::members_at(member8, &at8, "an object")? {
                    let nested8 = json::nested(&at8, key8);
                    entries8.insert(key8.clone(), decode_thalamus_definition_query_text(element8, &nested8)?);
                }
                entries8
            }
        },
        window: {
            let at9 = json::nested(at, "window");
            let member9 = json::member_at(value, at, "window")?;
            crate::primitives::Duration(json::text_at(member9, &at9, "an ISO 8601 duration")?.to_owned())
        },
        cadence: {
            let at10 = json::nested(at, "cadence");
            let member10 = json::member_at(value, at, "cadence")?;
            crate::primitives::Duration(json::text_at(member10, &at10, "an ISO 8601 duration")?.to_owned())
        },
        history: {
            let at11 = json::nested(at, "history");
            let member11 = json::member_at(value, at, "history")?;
            decode_thalamus_definition_history(member11, &at11)?
        },
        denominator: {
            let at12 = json::nested(at, "denominator");
            let member12 = json::member_at(value, at, "denominator")?;
            decode_thalamus_definition_denominator_spec(member12, &at12)?
        },
        evaluation: {
            let at13 = json::nested(at, "evaluation");
            let member13 = json::member_at(value, at, "evaluation")?;
            decode_thalamus_definition_evaluation_spec(member13, &at13)?
        },
        persistence: match value.member("persistence") {
            None | Some(json::Value::Null) => None,
            Some(member14) => {
                let at14 = json::nested(at, "persistence");
                Some(decode_thalamus_definition_persistence(member14, &at14)?)
            }
        },
        objective: match value.member("objective") {
            None | Some(json::Value::Null) => None,
            Some(member15) => {
                let at15 = json::nested(at, "objective");
                Some(json::value_at(member15, &at15))
            }
        },
        owner: {
            let at16 = json::nested(at, "owner");
            let member16 = json::member_at(value, at, "owner")?;
            decode_thalamus_definition_owner(member16, &at16)?
        },
        links: {
            let at17 = json::nested(at, "links");
            let member17 = json::member_at(value, at, "links")?;
            decode_thalamus_definition_links(member17, &at17)?
        },
        author: {
            let at18 = json::nested(at, "author");
            let member18 = json::member_at(value, at, "author")?;
            decode_thalamus_definition_author(member18, &at18)?
        },
    })
}

/// Writes the outcome of `thalamus.definition.ReviseDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_revise_definition(value: &crate::definition::ReviseDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::ReviseDefinitionOutcome::Retired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "retired");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.RetiredDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_retired_definition(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::HostNotLocal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "host-not-local");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.HostConnectionNotLocal");
            json::member(out, "payload");
            encode_error_thalamus_definition_host_connection_not_local(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::InstantNotFixed { error } => {
            json::member(out, "outcome");
            json::push_text(out, "instant-not-fixed");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.InstantNeedsFixedBaseline");
            json::member(out, "payload");
            encode_error_thalamus_definition_instant_needs_fixed_baseline(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::RollingNMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "rolling-n-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.RollingWindowMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_rolling_window_missing(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::RollingNUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "rolling-n-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.RollingWindowUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_rolling_window_unused(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::FixedNumberMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "fixed-number-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.FixedNumberMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_fixed_number_missing(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::FixedNumberUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "fixed-number-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.FixedNumberUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_fixed_number_unused(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::PersistenceBelowOne { error } => {
            json::member(out, "outcome");
            json::push_text(out, "persistence-below-one");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.PersistenceTooSmall");
            json::member(out, "payload");
            encode_error_thalamus_definition_persistence_too_small(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::MappingMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "mapping-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.MappingMissing");
            json::member(out, "payload");
            encode_error_thalamus_definition_mapping_missing(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::MappingUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "mapping-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.MappingUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_mapping_unused(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::ThresholdsUnused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "thresholds-unused");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.ThresholdsUnused");
            json::member(out, "payload");
            encode_error_thalamus_definition_thresholds_unused(error, out);
            out.push('}');
        }
        crate::definition::ReviseDefinitionOutcome::Revised { definition_revised } => {
            json::member(out, "outcome");
            json::push_text(out, "revised");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionRevised");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_revised(definition_revised, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::ReviseDefinitionOutcome::NoSuchDefinition { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-definition");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.UnknownDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_unknown_definition(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.definition.UnmuteDefinition` as JSON.
pub fn encode_command_thalamus_definition_unmute_definition(value: &crate::definition::UnmuteDefinition, out: &mut String) {
    out.push('{');
    json::member(out, "signal_id");
    encode_thalamus_definition_signal_id(&value.signal_id, out);
    out.push('}');
}

/// Reads the input of `thalamus.definition.UnmuteDefinition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_definition_unmute_definition(value: &json::Value, at: &str) -> Result<crate::definition::UnmuteDefinition, json::DecodeError> {
    Ok(crate::definition::UnmuteDefinition {
        signal_id: {
            let at0 = json::nested(at, "signal_id");
            let member0 = json::member_at(value, at, "signal_id")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
    })
}

/// Writes the outcome of `thalamus.definition.UnmuteDefinition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_definition_unmute_definition(value: &crate::definition::UnmuteDefinitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::definition::UnmuteDefinitionOutcome::Unmuted { definition_unmuted } => {
            json::member(out, "outcome");
            json::push_text(out, "unmuted");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.definition.DefinitionUnmuted");
            json::member(out, "payload");
            encode_event_thalamus_definition_definition_unmuted(definition_unmuted, out);
            out.push('}');
            out.push(']');
        }
        crate::definition::UnmuteDefinitionOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.DefinitionStateConflict");
            json::member(out, "payload");
            encode_error_thalamus_definition_definition_state_conflict(error, out);
            out.push('}');
        }
        crate::definition::UnmuteDefinitionOutcome::NoSuchDefinition { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-such-definition");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.definition.UnknownDefinition");
            json::member(out, "payload");
            encode_error_thalamus_definition_unknown_definition(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.dimension.DeclareRegistry` as JSON.
pub fn encode_command_thalamus_dimension_declare_registry(value: &crate::dimension::DeclareRegistry, out: &mut String) {
    out.push('{');
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    json::member(out, "title");
    json::push_text(out, &value.title);
    out.push('}');
}

/// Reads the input of `thalamus.dimension.DeclareRegistry` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_dimension_declare_registry(value: &json::Value, at: &str) -> Result<crate::dimension::DeclareRegistry, json::DecodeError> {
    Ok(crate::dimension::DeclareRegistry {
        registry_id: {
            let at0 = json::nested(at, "registry_id");
            let member0 = json::member_at(value, at, "registry_id")?;
            decode_thalamus_dimension_registry_id(member0, &at0)?
        },
        title: {
            let at1 = json::nested(at, "title");
            let member1 = json::member_at(value, at, "title")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `thalamus.dimension.DeclareRegistry` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_dimension_declare_registry(value: &crate::dimension::DeclareRegistryOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::dimension::DeclareRegistryOutcome::Declared { registry_declared } => {
            json::member(out, "outcome");
            json::push_text(out, "declared");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.dimension.RegistryDeclared");
            json::member(out, "payload");
            encode_event_thalamus_dimension_registry_declared(registry_declared, out);
            out.push('}');
            out.push(']');
        }
        crate::dimension::DeclareRegistryOutcome::AlreadyDeclared { error } => {
            json::member(out, "outcome");
            json::push_text(out, "already-declared");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.dimension.RegistryExists");
            json::member(out, "payload");
            encode_error_thalamus_dimension_registry_exists(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.dimension.RegisterDimension` as JSON.
pub fn encode_command_thalamus_dimension_register_dimension(value: &crate::dimension::RegisterDimension, out: &mut String) {
    out.push('{');
    json::member(out, "key");
    encode_thalamus_dimension_dimension_key(&value.key, out);
    json::member(out, "registry_id");
    encode_thalamus_dimension_registry_id(&value.registry_id, out);
    json::member(out, "values");
    encode_thalamus_dimension_value_source(&value.values, out);
    json::member(out, "rule");
    encode_thalamus_dimension_dimension_rule(&value.rule, out);
    if let Some(held0) = &value.parent {
        json::member(out, "parent");
        encode_thalamus_dimension_dimension_key(&*held0, out);
    }
    if let Some(held0) = &value.otel_attribute {
        json::member(out, "otel_attribute");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `thalamus.dimension.RegisterDimension` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_dimension_register_dimension(value: &json::Value, at: &str) -> Result<crate::dimension::RegisterDimension, json::DecodeError> {
    Ok(crate::dimension::RegisterDimension {
        key: {
            let at0 = json::nested(at, "key");
            let member0 = json::member_at(value, at, "key")?;
            decode_thalamus_dimension_dimension_key(member0, &at0)?
        },
        registry_id: {
            let at1 = json::nested(at, "registry_id");
            let member1 = json::member_at(value, at, "registry_id")?;
            decode_thalamus_dimension_registry_id(member1, &at1)?
        },
        values: {
            let at2 = json::nested(at, "values");
            let member2 = json::member_at(value, at, "values")?;
            decode_thalamus_dimension_value_source(member2, &at2)?
        },
        rule: {
            let at3 = json::nested(at, "rule");
            let member3 = json::member_at(value, at, "rule")?;
            decode_thalamus_dimension_dimension_rule(member3, &at3)?
        },
        parent: match value.member("parent") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "parent");
                Some(decode_thalamus_dimension_dimension_key(member4, &at4)?)
            }
        },
        otel_attribute: match value.member("otel_attribute") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "otel_attribute");
                Some(json::text_at(member5, &at5, "a string")?.to_owned())
            }
        },
    })
}

/// Writes the outcome of `thalamus.dimension.RegisterDimension` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_dimension_register_dimension(value: &crate::dimension::RegisterDimensionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::dimension::RegisterDimensionOutcome::NoRegistry { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-registry");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.dimension.UnknownRegistry");
            json::member(out, "payload");
            encode_error_thalamus_dimension_unknown_registry(error, out);
            out.push('}');
        }
        crate::dimension::RegisterDimensionOutcome::NoParent { error } => {
            json::member(out, "outcome");
            json::push_text(out, "no-parent");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.dimension.UnknownParent");
            json::member(out, "payload");
            encode_error_thalamus_dimension_unknown_parent(error, out);
            out.push('}');
        }
        crate::dimension::RegisterDimensionOutcome::Registered { dimension_registered } => {
            json::member(out, "outcome");
            json::push_text(out, "registered");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.dimension.DimensionRegistered");
            json::member(out, "payload");
            encode_event_thalamus_dimension_dimension_registered(dimension_registered, out);
            out.push('}');
            out.push(']');
        }
        crate::dimension::RegisterDimensionOutcome::AlreadyRegistered { error } => {
            json::member(out, "outcome");
            json::push_text(out, "already-registered");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.dimension.DimensionExists");
            json::member(out, "payload");
            encode_error_thalamus_dimension_dimension_exists(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.evaluation.RecordEvaluation` as JSON.
pub fn encode_command_thalamus_evaluation_record_evaluation(value: &crate::evaluation::RecordEvaluation, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    json::member(out, "verdict");
    encode_thalamus_evaluation_verdict(&value.verdict, out);
    if let Some(held0) = &value.prev {
        json::member(out, "prev");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    if let Some(held0) = &value.score {
        json::member(out, "score");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.baseline {
        json::member(out, "baseline");
        encode_thalamus_evaluation_baseline(&*held0, out);
    }
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `thalamus.evaluation.RecordEvaluation` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_evaluation_record_evaluation(value: &json::Value, at: &str) -> Result<crate::evaluation::RecordEvaluation, json::DecodeError> {
    Ok(crate::evaluation::RecordEvaluation {
        sample_id: {
            let at0 = json::nested(at, "sample_id");
            let member0 = json::member_at(value, at, "sample_id")?;
            decode_thalamus_observation_sample_id(member0, &at0)?
        },
        definition_rev: {
            let at1 = json::nested(at, "definition_rev");
            let member1 = json::member_at(value, at, "definition_rev")?;
            json::integer_at(member1, &at1, "an integer")?
        },
        verdict: {
            let at2 = json::nested(at, "verdict");
            let member2 = json::member_at(value, at, "verdict")?;
            decode_thalamus_evaluation_verdict(member2, &at2)?
        },
        prev: match value.member("prev") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "prev");
                Some(decode_thalamus_evaluation_verdict(member3, &at3)?)
            }
        },
        score: match value.member("score") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "score");
                Some(crate::primitives::Decimal(json::decimal_at(member4, &at4, "a decimal string")?.to_owned()))
            }
        },
        baseline: match value.member("baseline") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "baseline");
                Some(decode_thalamus_evaluation_baseline(member5, &at5)?)
            }
        },
        reason: match value.member("reason") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "reason");
                Some(json::text_at(member6, &at6, "a string")?.to_owned())
            }
        },
    })
}

/// Writes the response of `thalamus.evaluation.RecordEvaluation` as JSON.
pub fn encode_response_thalamus_evaluation_record_evaluation(value: &crate::evaluation::RecordEvaluationResponse, out: &mut String) {
    out.push('{');
    json::member(out, "evaluation_id");
    encode_thalamus_evaluation_evaluation_id(&value.evaluation_id, out);
    out.push('}');
}

/// Writes the outcome of `thalamus.evaluation.RecordEvaluation` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_evaluation_record_evaluation(value: &crate::evaluation::RecordEvaluationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::evaluation::RecordEvaluationOutcome::UnknownSample { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unknown-sample");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.evaluation.UnknownSample");
            json::member(out, "payload");
            encode_error_thalamus_evaluation_unknown_sample(error, out);
            out.push('}');
        }
        crate::evaluation::RecordEvaluationOutcome::ReasonMissing { error } => {
            json::member(out, "outcome");
            json::push_text(out, "reason-missing");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.evaluation.ReasonRequired");
            json::member(out, "payload");
            encode_error_thalamus_evaluation_reason_required(error, out);
            out.push('}');
        }
        crate::evaluation::RecordEvaluationOutcome::ScoreNotAdmitted { error } => {
            json::member(out, "outcome");
            json::push_text(out, "score-not-admitted");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.evaluation.ScoreNotAdmitted");
            json::member(out, "payload");
            encode_error_thalamus_evaluation_score_not_admitted(error, out);
            out.push('}');
        }
        crate::evaluation::RecordEvaluationOutcome::Recorded { evaluation_recorded, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "recorded");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.evaluation.EvaluationRecorded");
            json::member(out, "payload");
            encode_event_thalamus_evaluation_evaluation_recorded(evaluation_recorded, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.observation.RecordSample` as JSON.
pub fn encode_command_thalamus_observation_record_sample(value: &crate::observation::RecordSample, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "ts");
    json::push_text(out, &value.ts.0);
    json::member(out, "window");
    encode_thalamus_observation_time_window(&value.window, out);
    json::member(out, "reading");
    encode_thalamus_observation_reading(&value.reading, out);
    if let Some(held0) = &value.text {
        json::member(out, "text");
        json::push_text(out, &*held0);
    }
    json::member(out, "unit");
    encode_thalamus_definition_unit(&value.unit, out);
    if let Some(held0) = &value.denominator {
        json::member(out, "denominator");
        encode_thalamus_observation_denominator(&*held0, out);
    }
    json::member(out, "provenance");
    encode_thalamus_observation_provenance(&value.provenance, out);
    out.push('}');
}

/// Reads the input of `thalamus.observation.RecordSample` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_observation_record_sample(value: &json::Value, at: &str) -> Result<crate::observation::RecordSample, json::DecodeError> {
    Ok(crate::observation::RecordSample {
        signal: {
            let at0 = json::nested(at, "signal");
            let member0 = json::member_at(value, at, "signal")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
        series: {
            let at1 = json::nested(at, "series");
            let member1 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member1, &at1)?
        },
        dims: {
            let at2 = json::nested(at, "dims");
            let member2 = json::member_at(value, at, "dims")?;
            {
                let mut entries2 = std::collections::BTreeMap::new();
                for (key2, element2) in json::members_at(member2, &at2, "an object")? {
                    let nested2 = json::nested(&at2, key2);
                    entries2.insert(key2.clone(), decode_thalamus_dimension_dimension_value(element2, &nested2)?);
                }
                entries2
            }
        },
        ts: {
            let at3 = json::nested(at, "ts");
            let member3 = json::member_at(value, at, "ts")?;
            crate::primitives::Timestamp(json::text_at(member3, &at3, "an RFC 3339 instant")?.to_owned())
        },
        window: {
            let at4 = json::nested(at, "window");
            let member4 = json::member_at(value, at, "window")?;
            decode_thalamus_observation_time_window(member4, &at4)?
        },
        reading: {
            let at5 = json::nested(at, "reading");
            let member5 = json::member_at(value, at, "reading")?;
            decode_thalamus_observation_reading(member5, &at5)?
        },
        text: match value.member("text") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "text");
                Some(json::text_at(member6, &at6, "a string")?.to_owned())
            }
        },
        unit: {
            let at7 = json::nested(at, "unit");
            let member7 = json::member_at(value, at, "unit")?;
            decode_thalamus_definition_unit(member7, &at7)?
        },
        denominator: match value.member("denominator") {
            None | Some(json::Value::Null) => None,
            Some(member8) => {
                let at8 = json::nested(at, "denominator");
                Some(decode_thalamus_observation_denominator(member8, &at8)?)
            }
        },
        provenance: {
            let at9 = json::nested(at, "provenance");
            let member9 = json::member_at(value, at, "provenance")?;
            decode_thalamus_observation_provenance(member9, &at9)?
        },
    })
}

/// Writes the response of `thalamus.observation.RecordSample` as JSON.
pub fn encode_response_thalamus_observation_record_sample(value: &crate::observation::RecordSampleResponse, out: &mut String) {
    out.push('{');
    json::member(out, "sample_id");
    encode_thalamus_observation_sample_id(&value.sample_id, out);
    out.push('}');
}

/// Writes the outcome of `thalamus.observation.RecordSample` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_observation_record_sample(value: &crate::observation::RecordSampleOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::observation::RecordSampleOutcome::UnknownSignal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unknown-signal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.observation.UnknownSignal");
            json::member(out, "payload");
            encode_error_thalamus_observation_unknown_signal(error, out);
            out.push('}');
        }
        crate::observation::RecordSampleOutcome::NotActive { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-active");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.observation.SignalNotActive");
            json::member(out, "payload");
            encode_error_thalamus_observation_signal_not_active(error, out);
            out.push('}');
        }
        crate::observation::RecordSampleOutcome::Recorded { sample_recorded, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "recorded");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.observation.SampleRecorded");
            json::member(out, "payload");
            encode_event_thalamus_observation_sample_recorded(sample_recorded, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.rollup.RecordRollup` as JSON.
pub fn encode_command_thalamus_rollup_record_rollup(value: &crate::rollup::RecordRollup, out: &mut String) {
    out.push('{');
    json::member(out, "rollup_id");
    encode_thalamus_rollup_rollup_id(&value.rollup_id, out);
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "day");
    json::push_text(out, &value.day.0);
    json::member(out, "n");
    json::push_integer(out, value.n);
    json::member(out, "unknown");
    json::push_integer(out, value.unknown);
    json::member(out, "partial");
    json::push_integer(out, value.partial);
    if let Some(held0) = &value.min {
        json::member(out, "min");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.p50 {
        json::member(out, "p50");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.max {
        json::member(out, "max");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.last {
        json::member(out, "last");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.worst_verdict {
        json::member(out, "worst_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "verdict_counts");
    encode_thalamus_rollup_verdict_counts(&value.verdict_counts, out);
    if let Some(held0) = &value.min_score {
        json::member(out, "min_score");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.max_score {
        json::member(out, "max_score");
        json::push_text(out, &(*held0).0);
    }
    out.push('}');
}

/// Reads the input of `thalamus.rollup.RecordRollup` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_rollup_record_rollup(value: &json::Value, at: &str) -> Result<crate::rollup::RecordRollup, json::DecodeError> {
    Ok(crate::rollup::RecordRollup {
        rollup_id: {
            let at0 = json::nested(at, "rollup_id");
            let member0 = json::member_at(value, at, "rollup_id")?;
            decode_thalamus_rollup_rollup_id(member0, &at0)?
        },
        signal: {
            let at1 = json::nested(at, "signal");
            let member1 = json::member_at(value, at, "signal")?;
            decode_thalamus_definition_signal_id(member1, &at1)?
        },
        series: {
            let at2 = json::nested(at, "series");
            let member2 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member2, &at2)?
        },
        day: {
            let at3 = json::nested(at, "day");
            let member3 = json::member_at(value, at, "day")?;
            crate::primitives::Timestamp(json::text_at(member3, &at3, "an RFC 3339 instant")?.to_owned())
        },
        n: {
            let at4 = json::nested(at, "n");
            let member4 = json::member_at(value, at, "n")?;
            json::integer_at(member4, &at4, "an integer")?
        },
        unknown: {
            let at5 = json::nested(at, "unknown");
            let member5 = json::member_at(value, at, "unknown")?;
            json::integer_at(member5, &at5, "an integer")?
        },
        partial: {
            let at6 = json::nested(at, "partial");
            let member6 = json::member_at(value, at, "partial")?;
            json::integer_at(member6, &at6, "an integer")?
        },
        min: match value.member("min") {
            None | Some(json::Value::Null) => None,
            Some(member7) => {
                let at7 = json::nested(at, "min");
                Some(crate::primitives::Decimal(json::decimal_at(member7, &at7, "a decimal string")?.to_owned()))
            }
        },
        p50: match value.member("p50") {
            None | Some(json::Value::Null) => None,
            Some(member8) => {
                let at8 = json::nested(at, "p50");
                Some(crate::primitives::Decimal(json::decimal_at(member8, &at8, "a decimal string")?.to_owned()))
            }
        },
        max: match value.member("max") {
            None | Some(json::Value::Null) => None,
            Some(member9) => {
                let at9 = json::nested(at, "max");
                Some(crate::primitives::Decimal(json::decimal_at(member9, &at9, "a decimal string")?.to_owned()))
            }
        },
        last: match value.member("last") {
            None | Some(json::Value::Null) => None,
            Some(member10) => {
                let at10 = json::nested(at, "last");
                Some(crate::primitives::Decimal(json::decimal_at(member10, &at10, "a decimal string")?.to_owned()))
            }
        },
        worst_verdict: match value.member("worst_verdict") {
            None | Some(json::Value::Null) => None,
            Some(member11) => {
                let at11 = json::nested(at, "worst_verdict");
                Some(decode_thalamus_evaluation_verdict(member11, &at11)?)
            }
        },
        verdict_counts: {
            let at12 = json::nested(at, "verdict_counts");
            let member12 = json::member_at(value, at, "verdict_counts")?;
            decode_thalamus_rollup_verdict_counts(member12, &at12)?
        },
        min_score: match value.member("min_score") {
            None | Some(json::Value::Null) => None,
            Some(member13) => {
                let at13 = json::nested(at, "min_score");
                Some(crate::primitives::Decimal(json::decimal_at(member13, &at13, "a decimal string")?.to_owned()))
            }
        },
        max_score: match value.member("max_score") {
            None | Some(json::Value::Null) => None,
            Some(member14) => {
                let at14 = json::nested(at, "max_score");
                Some(crate::primitives::Decimal(json::decimal_at(member14, &at14, "a decimal string")?.to_owned()))
            }
        },
    })
}

/// Writes the outcome of `thalamus.rollup.RecordRollup` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_rollup_record_rollup(value: &crate::rollup::RecordRollupOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::rollup::RecordRollupOutcome::UnknownSignal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unknown-signal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.rollup.UnknownSignal");
            json::member(out, "payload");
            encode_error_thalamus_rollup_unknown_signal(error, out);
            out.push('}');
        }
        crate::rollup::RecordRollupOutcome::Recorded { rollup_recorded } => {
            json::member(out, "outcome");
            json::push_text(out, "recorded");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.rollup.RollupRecorded");
            json::member(out, "payload");
            encode_event_thalamus_rollup_rollup_recorded(rollup_recorded, out);
            out.push('}');
            out.push(']');
        }
        crate::rollup::RecordRollupOutcome::AlreadyRolledUp { error } => {
            json::member(out, "outcome");
            json::push_text(out, "already-rolled-up");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.rollup.RollupExists");
            json::member(out, "payload");
            encode_error_thalamus_rollup_rollup_exists(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `thalamus.transition.RecordTransition` as JSON.
pub fn encode_command_thalamus_transition_record_transition(value: &crate::transition::RecordTransition, out: &mut String) {
    out.push('{');
    json::member(out, "signal");
    encode_thalamus_definition_signal_id(&value.signal, out);
    json::member(out, "series");
    encode_thalamus_observation_series_key(&value.series, out);
    json::member(out, "dims");
    out.push('{');
    for (index0, (key0, item0)) in value.dims.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_thalamus_dimension_dimension_value(&*item0, out);
    }
    out.push('}');
    json::member(out, "definition_rev");
    json::push_integer(out, value.definition_rev);
    if let Some(held0) = &value.from_verdict {
        json::member(out, "from_verdict");
        encode_thalamus_evaluation_verdict(&*held0, out);
    }
    json::member(out, "to_verdict");
    encode_thalamus_evaluation_verdict(&value.to_verdict, out);
    json::member(out, "first_ts");
    json::push_text(out, &value.first_ts.0);
    json::member(out, "settled_ts");
    json::push_text(out, &value.settled_ts.0);
    if let Some(held0) = &value.settling_score {
        json::member(out, "settling_score");
        json::push_text(out, &(*held0).0);
    }
    json::member(out, "evaluations");
    out.push('[');
    for (index0, item0) in value.evaluations.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_thalamus_evaluation_evaluation_id(&*item0, out);
    }
    out.push(']');
    out.push('}');
}

/// Reads the input of `thalamus.transition.RecordTransition` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_thalamus_transition_record_transition(value: &json::Value, at: &str) -> Result<crate::transition::RecordTransition, json::DecodeError> {
    Ok(crate::transition::RecordTransition {
        signal: {
            let at0 = json::nested(at, "signal");
            let member0 = json::member_at(value, at, "signal")?;
            decode_thalamus_definition_signal_id(member0, &at0)?
        },
        series: {
            let at1 = json::nested(at, "series");
            let member1 = json::member_at(value, at, "series")?;
            decode_thalamus_observation_series_key(member1, &at1)?
        },
        dims: {
            let at2 = json::nested(at, "dims");
            let member2 = json::member_at(value, at, "dims")?;
            {
                let mut entries2 = std::collections::BTreeMap::new();
                for (key2, element2) in json::members_at(member2, &at2, "an object")? {
                    let nested2 = json::nested(&at2, key2);
                    entries2.insert(key2.clone(), decode_thalamus_dimension_dimension_value(element2, &nested2)?);
                }
                entries2
            }
        },
        definition_rev: {
            let at3 = json::nested(at, "definition_rev");
            let member3 = json::member_at(value, at, "definition_rev")?;
            json::integer_at(member3, &at3, "an integer")?
        },
        from_verdict: match value.member("from_verdict") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "from_verdict");
                Some(decode_thalamus_evaluation_verdict(member4, &at4)?)
            }
        },
        to_verdict: {
            let at5 = json::nested(at, "to_verdict");
            let member5 = json::member_at(value, at, "to_verdict")?;
            decode_thalamus_evaluation_verdict(member5, &at5)?
        },
        first_ts: {
            let at6 = json::nested(at, "first_ts");
            let member6 = json::member_at(value, at, "first_ts")?;
            crate::primitives::Timestamp(json::text_at(member6, &at6, "an RFC 3339 instant")?.to_owned())
        },
        settled_ts: {
            let at7 = json::nested(at, "settled_ts");
            let member7 = json::member_at(value, at, "settled_ts")?;
            crate::primitives::Timestamp(json::text_at(member7, &at7, "an RFC 3339 instant")?.to_owned())
        },
        settling_score: match value.member("settling_score") {
            None | Some(json::Value::Null) => None,
            Some(member8) => {
                let at8 = json::nested(at, "settling_score");
                Some(crate::primitives::Decimal(json::decimal_at(member8, &at8, "a decimal string")?.to_owned()))
            }
        },
        evaluations: {
            let at9 = json::nested(at, "evaluations");
            let member9 = json::member_at(value, at, "evaluations")?;
            {
                let mut items9 = Vec::new();
                for (index9, element9) in json::items_at(member9, &at9, "an array")?.iter().enumerate() {
                    let nested9 = json::nested(&at9, &index9.to_string());
                    items9.push(decode_thalamus_evaluation_evaluation_id(element9, &nested9)?);
                }
                items9
            }
        },
    })
}

/// Writes the response of `thalamus.transition.RecordTransition` as JSON.
pub fn encode_response_thalamus_transition_record_transition(value: &crate::transition::RecordTransitionResponse, out: &mut String) {
    out.push('{');
    json::member(out, "transition_id");
    encode_thalamus_transition_transition_id(&value.transition_id, out);
    out.push('}');
}

/// Writes the outcome of `thalamus.transition.RecordTransition` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_thalamus_transition_record_transition(value: &crate::transition::RecordTransitionOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::transition::RecordTransitionOutcome::UnknownSignal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "unknown-signal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "thalamus.transition.UnknownSignal");
            json::member(out, "payload");
            encode_error_thalamus_transition_unknown_signal(error, out);
            out.push('}');
        }
        crate::transition::RecordTransitionOutcome::Recorded { transition_recorded, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "recorded");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "thalamus.transition.TransitionRecorded");
            json::member(out, "payload");
            encode_event_thalamus_transition_transition_recorded(transition_recorded, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes any event on the system's log as JSON: its qualified name and its payload,
/// `{"event": …, "payload": {…}}`, the envelope a command's answer lists it in.
pub fn encode_system_event(value: &crate::system::SystemEvent) -> String {
    let mut out = String::from("{");
    json::member(&mut out, "event");
    json::push_text(&mut out, value.name());
    json::member(&mut out, "payload");
    match value {
        crate::system::SystemEvent::AssertionAnswered(event) => encode_event_thalamus_assertion_assertion_answered(event, &mut out),
        crate::system::SystemEvent::AssertionAsked(event) => encode_event_thalamus_assertion_assertion_asked(event, &mut out),
        crate::system::SystemEvent::DefinitionActivated(event) => encode_event_thalamus_definition_definition_activated(event, &mut out),
        crate::system::SystemEvent::DefinitionMuted(event) => encode_event_thalamus_definition_definition_muted(event, &mut out),
        crate::system::SystemEvent::DefinitionProposed(event) => encode_event_thalamus_definition_definition_proposed(event, &mut out),
        crate::system::SystemEvent::DefinitionRetired(event) => encode_event_thalamus_definition_definition_retired(event, &mut out),
        crate::system::SystemEvent::DefinitionRevised(event) => encode_event_thalamus_definition_definition_revised(event, &mut out),
        crate::system::SystemEvent::DefinitionUnmuted(event) => encode_event_thalamus_definition_definition_unmuted(event, &mut out),
        crate::system::SystemEvent::DimensionRegistered(event) => encode_event_thalamus_dimension_dimension_registered(event, &mut out),
        crate::system::SystemEvent::RegistryDeclared(event) => encode_event_thalamus_dimension_registry_declared(event, &mut out),
        crate::system::SystemEvent::EvaluationRecorded(event) => encode_event_thalamus_evaluation_evaluation_recorded(event, &mut out),
        crate::system::SystemEvent::SampleRecorded(event) => encode_event_thalamus_observation_sample_recorded(event, &mut out),
        crate::system::SystemEvent::RollupRecorded(event) => encode_event_thalamus_rollup_rollup_recorded(event, &mut out),
        crate::system::SystemEvent::TransitionRecorded(event) => encode_event_thalamus_transition_transition_recorded(event, &mut out),
    }
    out.push('}');
    out
}
