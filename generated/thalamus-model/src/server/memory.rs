// generated from thalamus v1
// model digest 51d971079fa66242e2a2f34142dbe44422e1204f980cec980f14fbeb58009d94
// contract digest fd2d9e49183a79ea356f366023aa03cd159da99e8e343e0af5bc5233645b32af
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `thalamus.assertion.Assertion`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryAssertionStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::assertion::AssertionSnapshot>>>);
impl crate::behaviour::AssertionStorage for MemoryAssertionStorage {
fn get(&self, identity: &crate::assertion::AssertionId) -> Option<crate::assertion::AssertionSnapshot> { let key = memory_key_3(identity); self.0.borrow().iter().find(|row| memory_key_3(&row.data.assertion_id) == key).cloned() }
fn put(&mut self, snapshot: crate::assertion::AssertionSnapshot) { self.delete(&snapshot.data.assertion_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_3(&row.data.assertion_id).cmp(&memory_key_3(&other.data.assertion_id))); }
fn delete(&mut self, identity: &crate::assertion::AssertionId) { let key = memory_key_3(identity); self.0.borrow_mut().retain(|row| memory_key_3(&row.data.assertion_id) != key); }
fn list(&self) -> Vec<crate::assertion::AssertionSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::AssertionStorage for MemoryPorts {
fn get(&self, identity: &crate::assertion::AssertionId) -> Option<crate::assertion::AssertionSnapshot> { crate::behaviour::AssertionStorage::get(&self.assertion_storage, identity) }
fn put(&mut self, snapshot: crate::assertion::AssertionSnapshot) { crate::behaviour::AssertionStorage::put(&mut self.assertion_storage, snapshot); }
fn delete(&mut self, identity: &crate::assertion::AssertionId) { crate::behaviour::AssertionStorage::delete(&mut self.assertion_storage, identity); }
fn list(&self) -> Vec<crate::assertion::AssertionSnapshot> { crate::behaviour::AssertionStorage::list(&self.assertion_storage) }
}

/// Ephemeral storage of `thalamus.definition.SignalDefinition`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySignalDefinitionStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::definition::SignalDefinitionSnapshot>>>);
impl crate::behaviour::SignalDefinitionStorage for MemorySignalDefinitionStorage {
fn get(&self, identity: &crate::definition::SignalId) -> Option<crate::definition::SignalDefinitionSnapshot> { let key = memory_key_20(identity); self.0.borrow().iter().find(|row| memory_key_20(&row.data.signal_id) == key).cloned() }
fn put(&mut self, snapshot: crate::definition::SignalDefinitionSnapshot) { self.delete(&snapshot.data.signal_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_20(&row.data.signal_id).cmp(&memory_key_20(&other.data.signal_id))); }
fn delete(&mut self, identity: &crate::definition::SignalId) { let key = memory_key_20(identity); self.0.borrow_mut().retain(|row| memory_key_20(&row.data.signal_id) != key); }
fn list(&self) -> Vec<crate::definition::SignalDefinitionSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::SignalDefinitionStorage for MemoryPorts {
fn get(&self, identity: &crate::definition::SignalId) -> Option<crate::definition::SignalDefinitionSnapshot> { crate::behaviour::SignalDefinitionStorage::get(&self.signal_definition_storage, identity) }
fn put(&mut self, snapshot: crate::definition::SignalDefinitionSnapshot) { crate::behaviour::SignalDefinitionStorage::put(&mut self.signal_definition_storage, snapshot); }
fn delete(&mut self, identity: &crate::definition::SignalId) { crate::behaviour::SignalDefinitionStorage::delete(&mut self.signal_definition_storage, identity); }
fn list(&self) -> Vec<crate::definition::SignalDefinitionSnapshot> { crate::behaviour::SignalDefinitionStorage::list(&self.signal_definition_storage) }
}

/// Ephemeral storage of `thalamus.dimension.Dimension`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryDimensionStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::dimension::DimensionSnapshot>>>);
impl crate::behaviour::DimensionStorage for MemoryDimensionStorage {
fn get(&self, identity: &crate::dimension::DimensionKey) -> Option<crate::dimension::DimensionSnapshot> { let key = memory_key_26(identity); self.0.borrow().iter().find(|row| memory_key_26(&row.data.key) == key).cloned() }
fn put(&mut self, snapshot: crate::dimension::DimensionSnapshot) { self.delete(&snapshot.data.key); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_26(&row.data.key).cmp(&memory_key_26(&other.data.key))); }
fn delete(&mut self, identity: &crate::dimension::DimensionKey) { let key = memory_key_26(identity); self.0.borrow_mut().retain(|row| memory_key_26(&row.data.key) != key); }
fn list(&self) -> Vec<crate::dimension::DimensionSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::DimensionStorage for MemoryPorts {
fn get(&self, identity: &crate::dimension::DimensionKey) -> Option<crate::dimension::DimensionSnapshot> { crate::behaviour::DimensionStorage::get(&self.dimension_storage, identity) }
fn put(&mut self, snapshot: crate::dimension::DimensionSnapshot) { crate::behaviour::DimensionStorage::put(&mut self.dimension_storage, snapshot); }
fn delete(&mut self, identity: &crate::dimension::DimensionKey) { crate::behaviour::DimensionStorage::delete(&mut self.dimension_storage, identity); }
fn list(&self) -> Vec<crate::dimension::DimensionSnapshot> { crate::behaviour::DimensionStorage::list(&self.dimension_storage) }
}

/// Ephemeral storage of `thalamus.dimension.DimensionRegistry`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryDimensionRegistryStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::dimension::DimensionRegistrySnapshot>>>);
impl crate::behaviour::DimensionRegistryStorage for MemoryDimensionRegistryStorage {
fn get(&self, identity: &crate::dimension::RegistryId) -> Option<crate::dimension::DimensionRegistrySnapshot> { let key = memory_key_31(identity); self.0.borrow().iter().find(|row| memory_key_31(&row.data.registry_id) == key).cloned() }
fn put(&mut self, snapshot: crate::dimension::DimensionRegistrySnapshot) { self.delete(&snapshot.data.registry_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_31(&row.data.registry_id).cmp(&memory_key_31(&other.data.registry_id))); }
fn delete(&mut self, identity: &crate::dimension::RegistryId) { let key = memory_key_31(identity); self.0.borrow_mut().retain(|row| memory_key_31(&row.data.registry_id) != key); }
fn list(&self) -> Vec<crate::dimension::DimensionRegistrySnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::DimensionRegistryStorage for MemoryPorts {
fn get(&self, identity: &crate::dimension::RegistryId) -> Option<crate::dimension::DimensionRegistrySnapshot> { crate::behaviour::DimensionRegistryStorage::get(&self.dimension_registry_storage, identity) }
fn put(&mut self, snapshot: crate::dimension::DimensionRegistrySnapshot) { crate::behaviour::DimensionRegistryStorage::put(&mut self.dimension_registry_storage, snapshot); }
fn delete(&mut self, identity: &crate::dimension::RegistryId) { crate::behaviour::DimensionRegistryStorage::delete(&mut self.dimension_registry_storage, identity); }
fn list(&self) -> Vec<crate::dimension::DimensionRegistrySnapshot> { crate::behaviour::DimensionRegistryStorage::list(&self.dimension_registry_storage) }
}

/// Ephemeral storage of `thalamus.evaluation.SignalEvaluation`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySignalEvaluationStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::evaluation::SignalEvaluationSnapshot>>>);
impl crate::behaviour::SignalEvaluationStorage for MemorySignalEvaluationStorage {
fn get(&self, identity: &crate::evaluation::EvaluationId) -> Option<crate::evaluation::SignalEvaluationSnapshot> { let key = memory_key_35(identity); self.0.borrow().iter().find(|row| memory_key_35(&row.data.evaluation_id) == key).cloned() }
fn put(&mut self, snapshot: crate::evaluation::SignalEvaluationSnapshot) { self.delete(&snapshot.data.evaluation_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_35(&row.data.evaluation_id).cmp(&memory_key_35(&other.data.evaluation_id))); }
fn delete(&mut self, identity: &crate::evaluation::EvaluationId) { let key = memory_key_35(identity); self.0.borrow_mut().retain(|row| memory_key_35(&row.data.evaluation_id) != key); }
fn list(&self) -> Vec<crate::evaluation::SignalEvaluationSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::SignalEvaluationStorage for MemoryPorts {
fn get(&self, identity: &crate::evaluation::EvaluationId) -> Option<crate::evaluation::SignalEvaluationSnapshot> { crate::behaviour::SignalEvaluationStorage::get(&self.signal_evaluation_storage, identity) }
fn put(&mut self, snapshot: crate::evaluation::SignalEvaluationSnapshot) { crate::behaviour::SignalEvaluationStorage::put(&mut self.signal_evaluation_storage, snapshot); }
fn delete(&mut self, identity: &crate::evaluation::EvaluationId) { crate::behaviour::SignalEvaluationStorage::delete(&mut self.signal_evaluation_storage, identity); }
fn list(&self) -> Vec<crate::evaluation::SignalEvaluationSnapshot> { crate::behaviour::SignalEvaluationStorage::list(&self.signal_evaluation_storage) }
}

/// Ephemeral storage of `thalamus.observation.SignalSample`, shared by clones.
#[derive(Clone, Default)]
pub struct MemorySignalSampleStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::observation::SignalSampleSnapshot>>>);
impl crate::behaviour::SignalSampleStorage for MemorySignalSampleStorage {
fn get(&self, identity: &crate::observation::SampleId) -> Option<crate::observation::SignalSampleSnapshot> { let key = memory_key_46(identity); self.0.borrow().iter().find(|row| memory_key_46(&row.data.sample_id) == key).cloned() }
fn put(&mut self, snapshot: crate::observation::SignalSampleSnapshot) { self.delete(&snapshot.data.sample_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_46(&row.data.sample_id).cmp(&memory_key_46(&other.data.sample_id))); }
fn delete(&mut self, identity: &crate::observation::SampleId) { let key = memory_key_46(identity); self.0.borrow_mut().retain(|row| memory_key_46(&row.data.sample_id) != key); }
fn list(&self) -> Vec<crate::observation::SignalSampleSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::SignalSampleStorage for MemoryPorts {
fn get(&self, identity: &crate::observation::SampleId) -> Option<crate::observation::SignalSampleSnapshot> { crate::behaviour::SignalSampleStorage::get(&self.signal_sample_storage, identity) }
fn put(&mut self, snapshot: crate::observation::SignalSampleSnapshot) { crate::behaviour::SignalSampleStorage::put(&mut self.signal_sample_storage, snapshot); }
fn delete(&mut self, identity: &crate::observation::SampleId) { crate::behaviour::SignalSampleStorage::delete(&mut self.signal_sample_storage, identity); }
fn list(&self) -> Vec<crate::observation::SignalSampleSnapshot> { crate::behaviour::SignalSampleStorage::list(&self.signal_sample_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `thalamus.assertion.Assertion`.
pub assertion_storage: MemoryAssertionStorage,
/// Storage of `thalamus.definition.SignalDefinition`.
pub signal_definition_storage: MemorySignalDefinitionStorage,
/// Storage of `thalamus.dimension.Dimension`.
pub dimension_storage: MemoryDimensionStorage,
/// Storage of `thalamus.dimension.DimensionRegistry`.
pub dimension_registry_storage: MemoryDimensionRegistryStorage,
/// Storage of `thalamus.evaluation.SignalEvaluation`.
pub signal_evaluation_storage: MemorySignalEvaluationStorage,
/// Storage of `thalamus.observation.SignalSample`.
pub signal_sample_storage: MemorySignalSampleStorage,
}

impl crate::behaviour::TryContext for MemoryPorts {
/// The host's UTC clock, the command clock this network entry supplies: read once per decision.
fn try_command_clock(&mut self) -> Result<Option<crate::primitives::Timestamp>, crate::obligation::UnmetObligation> { Ok(Some(crate::primitives::Timestamp(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).expect("UTC time formats")))) }
}

fn memory_key_3(value: &crate::assertion::AssertionId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_20(value: &crate::definition::SignalId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_26(value: &crate::dimension::DimensionKey) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_31(value: &crate::dimension::RegistryId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_35(value: &crate::evaluation::EvaluationId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_46(value: &crate::observation::SampleId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

impl crate::assertion::obligations::AskAssertionBehavior for MemoryPorts {
fn ask_assertion(&mut self, input: crate::assertion::AskAssertion) -> Result<crate::assertion::AskAssertionOutcome, crate::obligation::UnmetObligation> { crate::assertion::obligations::AskAssertionBehavior::ask_assertion(&mut crate::assertion::obligations::Unimplemented, input) }
}
impl crate::definition::obligations::ProposeDefinitionBehavior for MemoryPorts {
fn propose_definition(&mut self, input: crate::definition::ProposeDefinition) -> Result<crate::definition::ProposeDefinitionOutcome, crate::obligation::UnmetObligation> { crate::definition::obligations::ProposeDefinitionBehavior::propose_definition(&mut crate::definition::obligations::Unimplemented, input) }
}
impl crate::definition::obligations::ReviseDefinitionBehavior for MemoryPorts {
fn revise_definition(&mut self, input: crate::definition::ReviseDefinition) -> Result<crate::definition::ReviseDefinitionOutcome, crate::obligation::UnmetObligation> { crate::definition::obligations::ReviseDefinitionBehavior::revise_definition(&mut crate::definition::obligations::Unimplemented, input) }
}
impl crate::dimension::obligations::RegisterDimensionBehavior for MemoryPorts {
fn register_dimension(&mut self, input: crate::dimension::RegisterDimension) -> Result<crate::dimension::RegisterDimensionOutcome, crate::obligation::UnmetObligation> { crate::dimension::obligations::RegisterDimensionBehavior::register_dimension(&mut crate::dimension::obligations::Unimplemented, input) }
}
impl crate::evaluation::obligations::RecordEvaluationBehavior for MemoryPorts {
fn record_evaluation(&mut self, input: crate::evaluation::RecordEvaluation) -> Result<crate::evaluation::RecordEvaluationOutcome, crate::obligation::UnmetObligation> { crate::evaluation::obligations::RecordEvaluationBehavior::record_evaluation(&mut crate::evaluation::obligations::Unimplemented, input) }
}
impl crate::observation::obligations::RecordSampleBehavior for MemoryPorts {
fn record_sample(&mut self, input: crate::observation::RecordSample) -> Result<crate::observation::RecordSampleOutcome, crate::obligation::UnmetObligation> { crate::observation::obligations::RecordSampleBehavior::record_sample(&mut crate::observation::obligations::Unimplemented, input) }
}
impl crate::evaluation::obligations::SeriesVerdictsQuery for MemoryPorts {
fn series_verdicts(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::evaluation::SeriesVerdicts>, crate::obligation::UnmetObligation> { crate::evaluation::obligations::SeriesVerdictsQuery::series_verdicts(&crate::evaluation::obligations::Unimplemented, series) }
}
impl crate::observation::obligations::SeriesSamplesQuery for MemoryPorts {
fn series_samples(&self, series: crate::observation::SeriesKey) -> Result<Vec<crate::observation::SeriesSamples>, crate::obligation::UnmetObligation> { crate::observation::obligations::SeriesSamplesQuery::series_samples(&crate::observation::obligations::Unimplemented, series) }
}
