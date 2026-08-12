mod candidate;
mod diagnostics;
mod index;
mod inventory;
mod rules;
mod summary;

use std::{fs, path::Path};

use knowledge_base_snapshot::RepositorySnapshot;
use knowledge_base_validation::Diagnostic;

use crate::IngestionCandidateInventory;
use diagnostics::{DiagnosticFactory, ValidationReport};

/// Validates one strict inventory YAML file against the canonical repository snapshot.
pub fn validate_ingestion_candidate_inventory(inventory_path: &Path, snapshot: &RepositorySnapshot) -> Vec<Diagnostic> {
    let mut report = ValidationReport::default();
    if inventory_path.file_name().and_then(|name| name.to_str()) != Some("ingestion_candidate_inventory.yaml") {
        report.push(DiagnosticFactory::at_path(inventory_path, "filename must be ingestion_candidate_inventory.yaml"));
        return report.into_diagnostics();
    }
    let yaml = match fs::read_to_string(inventory_path) {
        Ok(yaml) => yaml,
        Err(error) => {
            report.push(DiagnosticFactory::at_path(inventory_path, format!("could not read ingestion candidate inventory: {error}")));
            return report.into_diagnostics();
        }
    };
    let inventory: IngestionCandidateInventory = match serde_yaml::from_str(&yaml) {
        Ok(inventory) => inventory,
        Err(error) => {
            report.push(DiagnosticFactory::at_path(inventory_path, format!("invalid ingestion candidate inventory YAML: {error}")));
            return report.into_diagnostics();
        }
    };
    inventory::validate(inventory_path, &inventory, snapshot, &mut report);
    report.into_diagnostics()
}

#[cfg(test)]
mod tests;
