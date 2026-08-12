//! Composition and verification of extension ontology contracts.

use crate::bindings::ResolvedBindings;
use crate::contracts::{BindingReference, ExtensionId, PropertyRequirement};
use crate::registry::ActiveExtensions;
use knowledge_base_models::{Cardinality, EntityTypeId, Property, PropertyId, PropertyUsage, ValueType};
use knowledge_base_snapshot::RepositorySnapshot;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A field constrained by a partial property requirement.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OntologyContractField {
    ValueType,
    Usage,
    Cardinality,
    SubjectTypes,
    TargetTypes,
    AllowedQualifiers,
}

impl fmt::Display for OntologyContractField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ValueType => "value_type",
            Self::Usage => "usage",
            Self::Cardinality => "cardinality",
            Self::SubjectTypes => "subject_types",
            Self::TargetTypes => "target_types",
            Self::AllowedQualifiers => "allowed_qualifiers",
        })
    }
}

/// One deterministic failure while checking activated extension ontology contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyContractDiagnostic {
    pub extension: ExtensionId,
    pub binding: BindingReference,
    pub id: PropertyId,
    pub field: OntologyContractField,
    pub message: String,
}

impl fmt::Display for OntologyContractDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "extension {} binding {} resolves to property {} with incompatible {}: {}",
            self.extension, self.binding, self.id, self.field, self.message
        )
    }
}

/// Verifies all partial ontology requirements for an activated extension set.
///
/// Requirements are composed by resolved property ID. Scalar expectations must agree;
/// collection expectations are unioned and checked as subsets of the property record.
pub fn verify_ontology_contracts(snapshot: &RepositorySnapshot, active: &ActiveExtensions, bindings: &ResolvedBindings) -> Vec<OntologyContractDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut requirements = BTreeMap::<PropertyId, MergedPropertyRequirement>::new();

    for extension in active.extensions() {
        let metadata = extension.metadata();
        // Manifest activation has already confirmed every entity-type binding resolves
        // to a canonical record. EntityTypeRequirement has no further fields to check.
        for requirement in &metadata.ontology_requirements.properties {
            let id = bindings
                .property(metadata, &requirement.binding)
                .expect("active requirements have validated property bindings")
                .clone();
            requirements
                .entry(id.clone())
                .or_default()
                .merge(id, metadata.id.clone(), requirement, bindings, metadata, &mut diagnostics);
        }
    }

    for (id, requirement) in requirements {
        let property = snapshot.properties().get(&id).expect("manifest activation confirms all bound properties exist");
        requirement.verify(property, &mut diagnostics);
    }

    diagnostics.sort_by(|left, right| {
        (&left.binding, &left.id, left.field, &left.message, &left.extension).cmp(&(&right.binding, &right.id, right.field, &right.message, &right.extension))
    });
    diagnostics
}

#[derive(Default)]
struct MergedPropertyRequirement {
    value_type: Scalar<ValueType>,
    usage: Scalar<PropertyUsage>,
    cardinality: Scalar<Cardinality>,
    subject_types: BTreeMap<EntityTypeId, Contributor>,
    target_types: BTreeMap<EntityTypeId, Contributor>,
    allowed_qualifiers: BTreeMap<PropertyId, Contributor>,
}

struct Scalar<T> {
    expected: Option<(T, Contributor)>,
    conflict: bool,
}

impl<T> Default for Scalar<T> {
    fn default() -> Self {
        Self { expected: None, conflict: false }
    }
}

#[derive(Clone)]
struct Contributor {
    extension: ExtensionId,
    binding: BindingReference,
}

impl MergedPropertyRequirement {
    fn merge(
        &mut self,
        id: PropertyId,
        extension: ExtensionId,
        requirement: &PropertyRequirement,
        bindings: &ResolvedBindings,
        metadata: &crate::contracts::ExtensionMetadata,
        diagnostics: &mut Vec<OntologyContractDiagnostic>,
    ) {
        let contributor = Contributor {
            extension,
            binding: requirement.binding.clone(),
        };
        self.value_type
            .merge(id.clone(), requirement.value_type, OntologyContractField::ValueType, &contributor, diagnostics);
        self.usage.merge(id.clone(), requirement.usage, OntologyContractField::Usage, &contributor, diagnostics);
        self.cardinality
            .merge(id, requirement.cardinality, OntologyContractField::Cardinality, &contributor, diagnostics);
        for binding in &requirement.subject_types {
            let id = bindings
                .entity_type(metadata, binding)
                .expect("active requirements have validated entity-type bindings")
                .clone();
            self.subject_types.entry(id).or_insert_with(|| contributor.clone());
        }
        if let Some(target_types) = &requirement.target_types {
            for binding in target_types {
                let id = bindings
                    .entity_type(metadata, binding)
                    .expect("active requirements have validated entity-type bindings")
                    .clone();
                self.target_types.entry(id).or_insert_with(|| contributor.clone());
            }
        }
        for binding in &requirement.allowed_qualifiers {
            let id = bindings.property(metadata, binding).expect("active requirements have validated property bindings").clone();
            self.allowed_qualifiers.entry(id).or_insert_with(|| contributor.clone());
        }
    }

    fn verify(&self, property: &Property, diagnostics: &mut Vec<OntologyContractDiagnostic>) {
        self.value_type.verify(property, OntologyContractField::ValueType, property.value_type, diagnostics);
        self.usage.verify(property, OntologyContractField::Usage, property.usage, diagnostics);
        self.cardinality.verify(property, OntologyContractField::Cardinality, property.cardinality, diagnostics);
        verify_collection(
            property,
            OntologyContractField::SubjectTypes,
            &self.subject_types,
            property.subject_types.iter().cloned().collect(),
            diagnostics,
        );
        if !self.target_types.is_empty() && property.value_type != ValueType::Entity {
            let contributor = self.target_types.values().next().expect("non-empty map has a contributor");
            diagnostics.push(diagnostic(
                contributor,
                property,
                OntologyContractField::TargetTypes,
                format!("property value_type is {}, but target types require entity", render(property.value_type)),
            ));
        } else {
            verify_collection(
                property,
                OntologyContractField::TargetTypes,
                &self.target_types,
                property.target_types.clone().unwrap_or_default().into_iter().collect(),
                diagnostics,
            );
        }
        verify_collection(
            property,
            OntologyContractField::AllowedQualifiers,
            &self.allowed_qualifiers,
            property.allowed_qualifiers.iter().cloned().collect(),
            diagnostics,
        );
    }
}

impl<T> Scalar<T>
where
    T: Copy + Eq + fmt::Debug,
{
    fn merge(&mut self, id: PropertyId, expected: Option<T>, field: OntologyContractField, contributor: &Contributor, diagnostics: &mut Vec<OntologyContractDiagnostic>) {
        let Some(expected) = expected else { return };
        match &self.expected {
            None => self.expected = Some((expected, contributor.clone())),
            Some((existing, _first)) if *existing == expected => {}
            Some((existing, first)) => {
                self.conflict = true;
                diagnostics.push(OntologyContractDiagnostic {
                    extension: contributor.extension.clone(),
                    binding: contributor.binding.clone(),
                    id,
                    field,
                    message: format!(
                        "requires {}, conflicting with {} required by extension {} binding {}",
                        render(expected),
                        render(*existing),
                        first.extension,
                        first.binding
                    ),
                });
            }
        }
    }

    fn verify(&self, property: &Property, field: OntologyContractField, actual: T, diagnostics: &mut Vec<OntologyContractDiagnostic>) {
        let Some((expected, contributor)) = &self.expected else { return };
        if !self.conflict && *expected != actual {
            diagnostics.push(diagnostic(
                contributor,
                property,
                field,
                format!("expected {}, found {}", render(*expected), render(actual)),
            ));
        }
    }
}

fn verify_collection<T>(
    property: &Property,
    field: OntologyContractField,
    expected: &BTreeMap<T, Contributor>,
    actual: BTreeSet<T>,
    diagnostics: &mut Vec<OntologyContractDiagnostic>,
) where
    T: Ord + fmt::Display,
{
    for (id, contributor) in expected {
        if !actual.contains(id) {
            diagnostics.push(diagnostic(contributor, property, field, format!("missing required entry {id}")));
        }
    }
}

fn diagnostic(contributor: &Contributor, property: &Property, field: OntologyContractField, message: String) -> OntologyContractDiagnostic {
    OntologyContractDiagnostic {
        extension: contributor.extension.clone(),
        binding: contributor.binding.clone(),
        id: property.id.clone(),
        field,
        message,
    }
}

fn render(value: impl fmt::Debug) -> String {
    format!("{value:?}").to_lowercase()
}
