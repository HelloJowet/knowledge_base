use super::Validator;
use crate::diagnostic::Diagnostics;
use crate::input::{ContextDocument, Loaded};
use knowledge_base_models::{Reference, ReferenceId};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
struct FootnoteDefinition {
    line: usize,
    targets: Vec<String>,
}

pub(super) fn validate(validator: &mut Validator<'_>) {
    let entities = &validator.indexes.entities;
    let references = &validator.indexes.references;
    let diagnostics = &mut validator.diagnostics;

    for context in &validator.repository.contexts {
        validate_context(context, entities.contains_key(&context.entity_id), references, diagnostics);
    }
}

fn validate_context(context: &ContextDocument, entity_exists: bool, references: &BTreeMap<ReferenceId, &Loaded<Reference>>, diagnostics: &mut Diagnostics) {
    let owner = context.entity_id.to_string();
    if !entity_exists {
        diagnostics.provenance(&context.path, None, &owner, "context document names an entity that does not exist");
    }

    let mut definitions = BTreeMap::<String, Vec<FootnoteDefinition>>::new();
    let mut references_used = BTreeMap::<String, usize>::new();
    let mut current_definition: Option<(String, FootnoteDefinition)> = None;
    let options = Options::ENABLE_FOOTNOTES;
    for (event, range) in Parser::new_ext(&context.source, options).into_offset_iter() {
        let line = line_at(&context.source, range.start);
        match event {
            Event::Start(Tag::FootnoteDefinition(label)) => {
                current_definition = Some((label.to_string(), FootnoteDefinition { line, targets: Vec::new() }));
            }
            Event::End(pulldown_cmark::TagEnd::FootnoteDefinition) => {
                if let Some((label, definition)) = current_definition.take() {
                    definitions.entry(label).or_default().push(definition);
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some((_, definition)) = current_definition.as_mut() {
                    definition.targets.push(dest_url.to_string());
                }
            }
            Event::FootnoteReference(label) => {
                references_used.entry(label.to_string()).or_insert(line);
            }
            _ => {}
        }
    }

    for (label, line) in &references_used {
        parse_reference_label(label, &context.path, *line, &owner, references, diagnostics);
        match definitions.get(label).map(Vec::len).unwrap_or_default() {
            0 => diagnostics.provenance(&context.path, Some(*line), &owner, format!("footnote {label:?} has no definition")),
            1 => {}
            count => diagnostics.provenance(&context.path, Some(*line), &owner, format!("footnote {label:?} has {count} definitions")),
        }
    }

    for (label, entries) in definitions {
        if entries.len() > 1 && !references_used.contains_key(&label) {
            diagnostics.provenance(
                &context.path,
                Some(entries[0].line),
                &owner,
                format!("footnote {label:?} has {} definitions", entries.len()),
            );
        }
        for definition in &entries {
            let reference_id = parse_reference_label(&label, &context.path, definition.line, &owner, references, diagnostics);
            if let Some(reference_id) = reference_id {
                let expected = format!("../references/{reference_id}.yaml");
                if definition.targets.as_slice() != [expected.as_str()] {
                    diagnostics.provenance(
                        &context.path,
                        Some(definition.line),
                        &owner,
                        format!("footnote {label:?} must contain exactly one link to {expected}"),
                    );
                }
            }
        }
    }
}

fn parse_reference_label(
    label: &str,
    path: &Path,
    line: usize,
    owner: &str,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    diagnostics: &mut Diagnostics,
) -> Option<ReferenceId> {
    let reference_id = match serde_yaml::from_value::<ReferenceId>(label.into()) {
        Ok(reference_id) => reference_id,
        Err(_) => {
            diagnostics.provenance(path, Some(line), owner, format!("footnote label {label:?} is not a canonical reference identifier"));
            return None;
        }
    };
    if !references.contains_key(&reference_id) {
        diagnostics.provenance(path, Some(line), owner, format!("footnote {label:?} cites a reference that does not exist"));
    }
    Some(reference_id)
}

fn line_at(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())].bytes().filter(|byte| *byte == b'\n').count() + 1
}
