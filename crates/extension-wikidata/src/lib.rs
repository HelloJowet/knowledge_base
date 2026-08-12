//! Public Wikidata support for statically composed knowledge-base distributions.
//!
//! Register [`WikidataExtension`] with a
//! [`knowledge_base_cli::Application`] to make the contract and the
//! `knowledge-base extension wikidata` command available.

#![forbid(unsafe_code)]

mod labels;

use clap::{Arg, ArgMatches, Command};
use knowledge_base_cli::{CliError, ExtensionCommandContext, KnowledgeBaseCliExtension};
use knowledge_base_crud::write::{ReferenceDraft, ReferenceRegistrationOutcome, ReferenceRegistrationStatus, WriteMode};
use knowledge_base_extension_framework::contracts::{
    BindingDeclaration, BindingKey, BindingKind, BindingReference, ContractVersion, ExtensionId, ExtensionMetadata, KnowledgeBaseExtension, OntologyRequirements,
    PropertyRequirement,
};
use knowledge_base_models::{Cardinality, PropertyUsage, ValueType};
use std::process::ExitCode;

const EXTENSION_NAME: &str = "wikidata";
const EXTENSION_ID: ExtensionId = ExtensionId::from_static(EXTENSION_NAME);
const ITEM_ID_PROPERTY: BindingKey = BindingKey::from_static("item_id_property");
const ITEM_ID_PROPERTY_BINDING: BindingReference = BindingReference::from_static("wikidata", "item_id_property");

/// Returns whether `value` is a canonical Wikidata item identifier (`Q` plus
/// a positive integer without leading zeroes).
pub fn is_canonical_item_id(value: &str) -> bool {
    let Some(digits) = value.strip_prefix('Q') else {
        return false;
    };
    digits.parse::<u64>().ok().is_some_and(|number| number > 0 && number.to_string() == digits)
}

/// Wikidata extension contract and CLI capability.
#[derive(Clone, Debug)]
pub struct WikidataExtension {
    metadata: ExtensionMetadata,
}

impl Default for WikidataExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl WikidataExtension {
    /// Constructs the version-1 Wikidata extension contract.
    pub fn new() -> Self {
        Self {
            metadata: ExtensionMetadata {
                id: EXTENSION_ID.clone(),
                contract: ContractVersion::new(1),
                dependencies: Vec::new(),
                bindings: vec![BindingDeclaration {
                    key: ITEM_ID_PROPERTY.clone(),
                    kind: BindingKind::Property,
                }],
                ontology_requirements: OntologyRequirements {
                    entity_types: Vec::new(),
                    properties: vec![PropertyRequirement {
                        binding: ITEM_ID_PROPERTY_BINDING.clone(),
                        value_type: Some(ValueType::String),
                        usage: Some(PropertyUsage::Statement),
                        cardinality: Some(Cardinality::One),
                        subject_types: Default::default(),
                        target_types: None,
                        allowed_qualifiers: Default::default(),
                    }],
                },
            },
        }
    }
}

impl KnowledgeBaseExtension for WikidataExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        &self.metadata
    }
}

impl KnowledgeBaseCliExtension for WikidataExtension {
    fn command(&self) -> Command {
        Command::new(EXTENSION_NAME).about("Register references for Wikidata items").subcommand(
            Command::new("reference").about("Work with Wikidata references").subcommand(
                Command::new("register")
                    .about("Register or reuse a Wikidata item reference")
                    .arg(Arg::new("item_id").value_name("QID").help("Canonical Wikidata item identifier, such as Q42").required(true)),
            ),
        )
    }

    fn requires_repository(&self, _: &ArgMatches) -> bool {
        true
    }

    fn execute(&self, matches: &ArgMatches, context: ExtensionCommandContext<'_>) -> Result<ExitCode, CliError> {
        let Some(("reference", reference_matches)) = matches.subcommand() else {
            return Err(CliError::new("a Wikidata subcommand is required"));
        };
        let Some(("register", register_matches)) = reference_matches.subcommand() else {
            return Err(CliError::new("a Wikidata reference subcommand is required"));
        };
        let item_id = register_matches.get_one::<String>("item_id").expect("required by Clap");
        let repository = context.repository().expect("repository-required extension commands receive a repository");
        let outcome = register_reference(repository, item_id, labels::get_entity_label).map_err(|error| CliError::new(error.to_string()))?;
        let output = serde_yaml::to_string(&outcome).map_err(|error| CliError::new(format!("cannot serialize command output: {error}")))?;
        context.write_stdout(&output)?;
        Ok(ExitCode::SUCCESS)
    }
}

fn register_reference(
    repository: &knowledge_base_crud::KnowledgeBaseRepository,
    item_id: &str,
    lookup_label: impl FnOnce(&str) -> anyhow::Result<Option<labels::RetrievedLabel>>,
) -> anyhow::Result<ReferenceRegistrationOutcome> {
    anyhow::ensure!(is_canonical_item_id(item_id), "Wikidata item ID must use canonical uppercase Q<number> form");
    let url = format!("https://www.wikidata.org/wiki/{item_id}?uselang=en");

    if let Some(reference) = repository
        .read()
        .snapshot()?
        .references()
        .values()
        .find(|reference| reference.url == url)
        .map(|reference| reference.id.clone())
    {
        return Ok(ReferenceRegistrationOutcome {
            status: ReferenceRegistrationStatus::Existing,
            reference,
        });
    }

    let label = lookup_label(item_id)?.ok_or_else(|| anyhow::anyhow!("Wikidata item {item_id} has no usable English or fallback label"))?;
    repository
        .write()
        .references()
        .register(
            &ReferenceDraft {
                url,
                title: label.value,
                publisher: Some("Wikidata".to_owned()),
                publication_date: None,
                source_language: Some(label.language),
                retrieved_at: label.retrieved_at,
                archive_url: None,
            },
            WriteMode::Commit,
        )
        .map_err(Into::into)
}

#[cfg(test)]
mod tests;
