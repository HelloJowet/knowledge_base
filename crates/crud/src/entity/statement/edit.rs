use crate::Error;
use knowledge_base_models::{Entity, PropertyId, Qualifier, ReferenceId, Statement, StatementId, Value};
use serde::Serialize;
use std::path::Path;
use std::str::FromStr;
use yaml_edit::YamlFile;

pub(super) fn append_statements(source: &str, statements: &[Statement], path: &Path) -> Result<String, Error> {
    let yaml = YamlFile::from_str(source).map_err(|error| Error::Edit {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let document = yaml.document().ok_or_else(|| Error::Edit {
        path: path.to_path_buf(),
        message: "expected exactly one YAML document".to_owned(),
    })?;
    let sequence = document.get_sequence("statements").ok_or_else(|| Error::Edit {
        path: path.to_path_buf(),
        message: "statements must be a YAML sequence".to_owned(),
    })?;
    let range = sequence.byte_range();
    let start = range.start as usize;
    let end = range.end as usize;
    let existing = &source[start..end];
    let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
    let leading = &source[line_start..start];
    let mapping_indent = leading.chars().take_while(|character| character.is_whitespace()).collect::<String>();
    let entry_indent = if existing.trim() == "[]" { format!("{mapping_indent}  ") } else { leading.to_owned() };
    let additions = render_statement_entries(statements, &entry_indent, path)?;

    let mut replacement = String::with_capacity(source.len() + additions.len() + 1);
    replacement.push_str(&source[..start]);
    if existing.trim() == "[]" {
        replacement.push('\n');
        replacement.push_str(&additions);
        replacement.push_str(&source[end..]);
    } else if existing.trim_start().starts_with('[') {
        return Err(Error::Edit {
            path: path.to_path_buf(),
            message: "non-empty flow-style statements sequences cannot be extended without reformatting".to_owned(),
        });
    } else {
        replacement.push_str(existing);
        if !existing.ends_with('\n') {
            replacement.push('\n');
        }
        replacement.push_str(&additions);
        replacement.push_str(&source[end..]);
    }
    serde_yaml::from_str::<Entity>(&replacement).map_err(|source| Error::ParseEntity { path: path.to_path_buf(), source })?;
    Ok(replacement)
}

#[derive(Serialize)]
struct StoredStatement<'a> {
    id: &'a StatementId,
    property: &'a PropertyId,
    value: &'a Value,
    #[serde(skip_serializing_if = "slice_is_empty")]
    qualifiers: &'a [Qualifier],
    references: &'a [ReferenceId],
}

fn slice_is_empty<T>(items: &[T]) -> bool {
    items.is_empty()
}

fn render_statement_entries(statements: &[Statement], indentation: &str, path: &Path) -> Result<String, Error> {
    let mut output = String::new();
    for statement in statements {
        let stored = StoredStatement {
            id: &statement.id,
            property: &statement.property,
            value: &statement.value,
            qualifiers: &statement.qualifiers,
            references: &statement.references,
        };
        let generated = serde_yaml::to_string(&stored).map_err(|error| Error::Edit {
            path: path.to_path_buf(),
            message: format!("cannot serialize statement: {error}"),
        })?;
        for (line_index, line) in generated.trim_end_matches('\n').lines().enumerate() {
            output.push_str(indentation);
            output.push_str(if line_index == 0 { "- " } else { "  " });
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::append_statements;
    use knowledge_base_models::{PropertyId, Qualifier, ReferenceId, Statement, StatementId, Value};
    use std::path::Path;

    fn statement(id: &str, value: i64) -> Statement {
        Statement {
            id: id.parse::<StatementId>().unwrap(),
            property: "P1".parse::<PropertyId>().unwrap(),
            value: Value::Integer { value },
            qualifiers: Vec::new(),
            references: vec!["R1".parse::<ReferenceId>().unwrap()],
        }
    }

    #[test]
    fn lossless_append_preserves_existing_text() {
        let source = "id: Q1\n# keep this comment\nlabels: {}\nentity_types: []\nstatements:\n  - id: S1 # keep inline\n    property: P1\n    value: { type: integer, value: 1 }\n    references: [R1]\n";
        let replacement = append_statements(source, &[statement("S2", 2)], Path::new("Q1.yaml")).unwrap();

        assert!(replacement.starts_with(source));
        assert!(replacement.contains("  - id: S2\n    property: P1\n"));
        assert!(replacement.contains("# keep this comment"));
        assert!(replacement.contains("id: S1 # keep inline"));
    }

    #[test]
    fn empty_inline_sequence_is_changed_only_at_the_statements_value() {
        let source = "id: Q1\nlabels: {}\nentity_types: []\nstatements: []\n";
        let replacement = append_statements(source, &[statement("S1", 2)], Path::new("Q1.yaml")).unwrap();

        assert!(replacement.starts_with("id: Q1\nlabels: {}\nentity_types: []\nstatements: \n  - id: S1\n"));
    }

    #[test]
    fn appended_qualifiers_are_serialized_in_manifest_order() {
        let source = "id: Q1\nlabels: {}\nentity_types: []\nstatements: []\n";
        let mut appended = statement("S1", 2);
        appended.qualifiers = vec![
            Qualifier {
                property: "P3".parse().unwrap(),
                value: Value::String { value: "first".to_owned() },
            },
            Qualifier {
                property: "P2".parse().unwrap(),
                value: Value::Date { value: "2024-01-01".to_owned() },
            },
        ];

        let replacement = append_statements(source, &[appended], Path::new("Q1.yaml")).unwrap();

        let first = replacement.find("property: P3").unwrap();
        let second = replacement.find("property: P2").unwrap();
        assert!(replacement.contains("qualifiers:"));
        assert!(first < second);
    }
}
