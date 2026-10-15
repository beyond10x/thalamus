//! The dimension obligation of the synthesis plan (`generated/thalamus-model/PLAN.md`):
//! `thalamus.dimension.RegisterDimension`.
//!
//! It reads and writes only through the generated storage ports. The plan keeps it an obligation
//! because its `when_related:` branches read two related rows in one command; it selects in the
//! plan's precedence for such a command: the addressed key already registered, then each related
//! row in the declaration order of its `exists: false` branch (registry, then parent), then the
//! accepting branch.

use thalamus_model::behaviour::{DimensionRegistryStorage, DimensionStorage};
use thalamus_model::dimension::{
    AnyDimension, Dimension, DimensionData, DimensionExists, DimensionRegistered,
    RegisterDimension, RegisterDimensionOutcome, UnknownParent, UnknownRegistry,
};
use thalamus_model::obligation::UnmetObligation;

/// `thalamus.dimension.RegisterDimension`: `already-registered`, then `no-registry`, then
/// `no-parent` where a parent is named, then `registered`.
///
/// # Errors
///
/// None today: every input selects a declared outcome. The `Result` is the obligation's
/// contract, which a refusal of the model would answer through.
pub fn register_dimension<P>(
    ports: &mut P,
    input: RegisterDimension,
) -> Result<RegisterDimensionOutcome, UnmetObligation>
where
    P: DimensionStorage + DimensionRegistryStorage + ?Sized,
{
    if DimensionStorage::get(ports, &input.key).is_some() {
        return Ok(RegisterDimensionOutcome::AlreadyRegistered {
            error: DimensionExists { key: input.key },
        });
    }
    if DimensionRegistryStorage::get(ports, &input.registry_id).is_none() {
        return Ok(RegisterDimensionOutcome::NoRegistry {
            error: UnknownRegistry {
                registry_id: input.registry_id,
            },
        });
    }
    if let Some(parent) = &input.parent
        && DimensionStorage::get(ports, parent).is_none()
    {
        return Ok(RegisterDimensionOutcome::NoParent {
            error: UnknownParent {
                parent: input.parent,
            },
        });
    }
    let answer = RegisterDimensionOutcome::Registered {
        dimension_registered: DimensionRegistered {
            key: input.key.clone(),
            registry_id: input.registry_id.clone(),
            rule: input.rule,
        },
    };
    let data = DimensionData {
        key: input.key,
        registry_id: input.registry_id,
        values: input.values,
        rule: input.rule,
        parent: input.parent,
        otel_attribute: input.otel_attribute,
    };
    DimensionStorage::put(
        ports,
        AnyDimension::Registered(Dimension::new(data)).snapshot(),
    );
    Ok(answer)
}
