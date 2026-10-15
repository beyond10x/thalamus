//! An instance directory (design § 5.2): its definitions, its dimension registry and the registry
//! value files, read from disk and handed to `thalamus_core::registry`, which validates them.
//!
//! Layout:
//!
//! | path | holds |
//! |---|---|
//! | `<instance>/dimensions.yaml` | the dimension registry |
//! | `<instance>/definitions/*.yaml` | one definition per file |
//! | `<instance>/<locator>` | the values of each dimension whose registry is `file:<locator>` |
//!
//! Loading also parses the grading rule of every definition, whatever its lifecycle state, so a
//! rule id that names no rule refuses the instance before any source is read. The rule of a
//! definition that is sampled (`Active` or `Muted`) is also bound to the definition's lines
//! (`thalamus_core::rules`). Every refusal names the file and the rule.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use thalamus_core::grading;
use thalamus_core::registry::{FileValues, Instance, LoadError};
use thalamus_core::rules::RuleId;
use thalamus_model::definition::SignalDefinitionState;
use thalamus_model::dimension::ValueSource;

/// The registry file, relative to the instance directory.
pub const REGISTRY_FILE: &str = "dimensions.yaml";
/// The definitions directory, relative to the instance directory.
pub const DEFINITIONS_DIR: &str = "definitions";

/// One refusal of an instance: the file and the rule it breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceError {
    /// The file or directory could not be read.
    Unreadable {
        /// The path.
        file: String,
        /// What the system said.
        error: String,
    },
    /// `thalamus_core::registry` refused the file.
    Load(LoadError),
    /// The definition's grading rule is unknown, or its lines cannot be graded against.
    Grading {
        /// The definition file.
        file: String,
        /// Why.
        refusal: grading::Refusal,
    },
}

impl InstanceError {
    /// The file the refusal is about.
    #[must_use]
    pub fn file(&self) -> &str {
        match self {
            Self::Unreadable { file, .. } | Self::Grading { file, .. } => file,
            Self::Load(error) => &error.file,
        }
    }
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { file, error } => {
                write!(f, "{file}: readable instance file: {error}")
            }
            Self::Load(error) => error.fmt(f),
            Self::Grading { file, refusal } => write!(f, "{file}: grading rule: {refusal}"),
        }
    }
}

impl std::error::Error for InstanceError {}

/// A loaded instance: the validated definitions and registry, and the parsed values of every
/// `file:` registry, by dimension key.
#[derive(Debug, Clone)]
pub struct LoadedInstance {
    dir: PathBuf,
    instance: Instance,
    values: BTreeMap<String, FileValues>,
}

impl LoadedInstance {
    /// Reads and validates the instance in `dir`.
    ///
    /// # Errors
    ///
    /// Every [`InstanceError`] found: an unreadable file, every refusal of
    /// `thalamus_core::registry`, every refused grading rule. A refused registry stops the load
    /// before the definitions are judged against it.
    pub fn load(dir: &Path) -> Result<Self, Vec<InstanceError>> {
        let registry_file = dir.join(REGISTRY_FILE);
        let registry_name = registry_file.display().to_string();
        let registry_text = read(&registry_file).map_err(|error| vec![error])?;

        let mut errors = Vec::new();
        let definitions_dir = dir.join(DEFINITIONS_DIR);
        let mut files: Vec<PathBuf> = match std::fs::read_dir(&definitions_dir) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file()
                        && path
                            .extension()
                            .is_some_and(|extension| extension == "yaml" || extension == "yml")
                })
                .collect(),
            Err(error) => {
                errors.push(InstanceError::Unreadable {
                    file: definitions_dir.display().to_string(),
                    error: error.to_string(),
                });
                Vec::new()
            }
        };
        files.sort();
        let mut texts = Vec::new();
        for file in &files {
            match read(file) {
                Ok(text) => texts.push((file.display().to_string(), text)),
                Err(error) => errors.push(error),
            }
        }
        let definitions: Vec<(&str, &str)> = texts
            .iter()
            .map(|(file, text)| (file.as_str(), text.as_str()))
            .collect();

        let instance = match Instance::load((&registry_name, &registry_text), &definitions) {
            Ok(instance) => Some(instance),
            Err(refused) => {
                errors.extend(refused.into_iter().map(InstanceError::Load));
                None
            }
        };
        let Some(instance) = instance else {
            return Err(errors);
        };

        for definition in instance.definitions() {
            let sampled = matches!(
                definition.state(),
                SignalDefinitionState::Active | SignalDefinitionState::Muted
            );
            let evaluation = &definition.data().evaluation;
            let refusal = match evaluation.rule.0.parse::<RuleId>() {
                Err(unknown) => Some(grading::Refusal::UnknownRule(unknown)),
                Ok(rule) if sampled => rule.prepare(evaluation).err().map(grading::Refusal::Lines),
                Ok(_) => None,
            };
            if let Some(refusal) = refusal {
                errors.push(InstanceError::Grading {
                    file: definition.file().to_owned(),
                    refusal,
                });
            }
        }

        let mut values = BTreeMap::new();
        for dimension in instance.registry().dimensions() {
            let ValueSource::File(locator) = &dimension.values else {
                continue;
            };
            let file = dir.join(&locator.0);
            let name = file.display().to_string();
            match read(&file) {
                Err(error) => errors.push(error),
                Ok(text) => match FileValues::parse(&name, &text) {
                    Ok(parsed) => {
                        values.insert(dimension.key.0.clone(), parsed);
                    }
                    Err(refused) => errors.extend(refused.into_iter().map(InstanceError::Load)),
                },
            }
        }

        if errors.is_empty() {
            Ok(Self {
                dir: dir.to_owned(),
                instance,
                values,
            })
        } else {
            Err(errors)
        }
    }

    /// The instance directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The instance's name: its registry id, which its store stream is kept under.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.instance.registry().data().registry_id.0
    }

    /// The validated definitions and registry.
    #[must_use]
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// The validated definitions and registry, for the generated ports that take them mutably.
    pub fn instance_mut(&mut self) -> &mut Instance {
        &mut self.instance
    }

    /// The values of every `file:` registry, by dimension key.
    #[must_use]
    pub fn values(&self) -> &BTreeMap<String, FileValues> {
        &self.values
    }
}

fn read(path: &Path) -> Result<String, InstanceError> {
    std::fs::read_to_string(path).map_err(|error| InstanceError::Unreadable {
        file: path.display().to_string(),
        error: error.to_string(),
    })
}
