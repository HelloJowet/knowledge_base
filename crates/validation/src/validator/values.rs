use crate::diagnostic::{Diagnostics, ValidationLayer};
use crate::input::Loaded;
use chrono::{DateTime, NaiveDate};
use knowledge_base_models::{LocalizedMap, Reference, ReferenceId, Value, ValueType};
use language_tags::LanguageTag;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::LazyLock;
use url::Url;

static DECIMAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?(0|[1-9][0-9]*)(\.[0-9]+)?$").expect("valid regex"));

#[derive(Clone, Copy)]
pub(super) struct LocalizedMapRules {
    pub required: bool,
    pub references_required: bool,
}

pub(super) fn validate_nonempty_metadata<T>(item: &Loaded<T>, id: &str, field: &str, value: &str, diagnostics: &mut Diagnostics) -> bool {
    if value.trim().is_empty() {
        diagnostics.schema(item, id, format!("{field} must not be empty"));
        false
    } else {
        true
    }
}

pub(super) fn valid_partial_date(value: &str) -> bool {
    match value.len() {
        4 => value.bytes().all(|byte| byte.is_ascii_digit()),
        7 => value
            .get(..4)
            .zip(value.get(5..))
            .filter(|(year, month)| year.bytes().all(|byte| byte.is_ascii_digit()) && month.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|(year, month)| year.parse::<i32>().ok().zip(month.parse::<u32>().ok()))
            .is_some_and(|(year, month)| value.as_bytes().get(4) == Some(&b'-') && NaiveDate::from_ymd_opt(year, month, 1).is_some()),
        10 => NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        _ => false,
    }
}

pub(super) fn validate_value(path: &Path, owner: &str, value: &Value, diagnostics: &mut Diagnostics) {
    let message = match value {
        Value::Decimal { value } if !DECIMAL.is_match(value) => Some("decimal value must use canonical quoted base-10 syntax"),
        Value::Quantity { amount, unit: _ } if !DECIMAL.is_match(amount) => Some("quantity amount must use canonical quoted base-10 syntax"),
        Value::Quantity { unit, .. } if unit.trim().is_empty() => Some("quantity unit must not be empty"),
        Value::Date { value } if !valid_partial_date(value) => Some("date value must be a real ISO 8601 calendar year, month, or day"),
        Value::Datetime { value } if DateTime::parse_from_rfc3339(value).is_err() => Some("datetime value must be an RFC 3339 timestamp"),
        Value::Url { value } if Url::parse(value).is_err() => Some("url value must be an absolute URL"),
        Value::Coordinate { latitude, longitude, precision } => {
            if !DECIMAL.is_match(latitude) || !within_absolute_bound(latitude, 90) {
                Some("coordinate latitude must be canonical decimal text between -90 and 90")
            } else if !DECIMAL.is_match(longitude) || !within_absolute_bound(longitude, 180) {
                Some("coordinate longitude must be canonical decimal text between -180 and 180")
            } else if precision.as_ref().is_some_and(|precision| !positive_decimal(precision)) {
                Some("coordinate precision must be positive canonical decimal text in metres")
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(message) = message {
        diagnostics.push(ValidationLayer::Schema, path.to_path_buf(), None, Some(owner.to_owned()), message);
    }
}

pub(super) fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Entity => "entity",
        ValueType::String => "string",
        ValueType::Integer => "integer",
        ValueType::Decimal => "decimal",
        ValueType::Quantity => "quantity",
        ValueType::Boolean => "boolean",
        ValueType::Date => "date",
        ValueType::Datetime => "datetime",
        ValueType::Url => "url",
        ValueType::Coordinate => "coordinate",
    }
}

pub(super) fn validate_localized_map(
    path: &Path,
    owner: &str,
    field: &str,
    values: &LocalizedMap,
    rules: LocalizedMapRules,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    diagnostics: &mut Diagnostics,
) {
    if rules.required && values.is_empty() {
        diagnostics.push(
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} must not be empty"),
        );
    }
    let mut normalized = BTreeSet::new();
    for (locale, value) in values {
        if locale.parse::<LanguageTag>().is_err() {
            diagnostics.push(
                ValidationLayer::Schema,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field} locale {locale:?} is not a well-formed BCP 47 tag"),
            );
        }
        if !normalized.insert(locale.to_ascii_lowercase()) {
            diagnostics.push(
                ValidationLayer::Schema,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field} contains locale {locale:?} more than once ignoring case"),
            );
        }
        if value.text.trim().is_empty() {
            diagnostics.push(
                ValidationLayer::Schema,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field}.{locale} text must not be empty"),
            );
        }
        if rules.references_required {
            validate_provenance(path, owner, &format!("{field}.{locale}"), &value.references, references, diagnostics);
        } else {
            validate_optional_provenance(path, owner, &format!("{field}.{locale}"), &value.references, references, diagnostics);
        }
    }
}

pub(super) fn validate_provenance(
    path: &Path,
    owner: &str,
    field: &str,
    reference_ids: &[ReferenceId],
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    diagnostics: &mut Diagnostics,
) {
    if reference_ids.is_empty() {
        diagnostics.push(
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} references must not be empty"),
        );
    }
    for reference_id in reference_ids {
        if !references.contains_key(reference_id) {
            diagnostics.provenance(path, None, owner, format!("{field} cites missing reference {reference_id}"));
        }
    }
}

pub(super) fn validate_optional_provenance(
    path: &Path,
    owner: &str,
    field: &str,
    reference_ids: &[ReferenceId],
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    diagnostics: &mut Diagnostics,
) {
    for reference_id in reference_ids {
        if !references.contains_key(reference_id) {
            diagnostics.provenance(path, None, owner, format!("{field} cites missing reference {reference_id}"));
        }
    }
}

pub(super) fn validate_url(path: &Path, owner: &str, field: &str, value: &str, diagnostics: &mut Diagnostics) {
    if Url::parse(value).is_err() {
        diagnostics.push(
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} must be an absolute URL"),
        );
    }
}

fn within_absolute_bound(value: &str, bound: u64) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    match integer.len().cmp(&bound.to_string().len()) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => match integer.parse::<u64>() {
            Ok(integer) if integer < bound => true,
            Ok(integer) if integer == bound => fraction.bytes().all(|byte| byte == b'0'),
            _ => false,
        },
    }
}

fn positive_decimal(value: &str) -> bool {
    DECIMAL.is_match(value) && !value.starts_with('-') && value.bytes().any(|byte| matches!(byte, b'1'..=b'9'))
}

#[cfg(test)]
mod tests {
    use super::{valid_partial_date, validate_value};
    use crate::diagnostic::Diagnostics;
    use knowledge_base_models::Value;
    use std::path::Path;

    #[test]
    fn validates_partial_dates() {
        for value in ["2025", "2025-02", "2024-02-29"] {
            assert!(valid_partial_date(value), "expected {value} to be valid");
        }
        for value in ["25", "2025-13", "2025-02-29", "202é-1"] {
            assert!(!valid_partial_date(value), "expected {value} to be invalid");
        }
    }

    #[test]
    fn validates_every_value_type() {
        let values = [
            Value::Entity { value: "Q1".parse().unwrap() },
            Value::String { value: "Bilecik".to_owned() },
            Value::Integer { value: 42 },
            Value::Decimal { value: "12.5".to_owned() },
            Value::Quantity {
                amount: "12.5".to_owned(),
                unit: "km".to_owned(),
            },
            Value::Boolean { value: true },
            Value::Date { value: "2024".to_owned() },
            Value::Date { value: "2024-02".to_owned() },
            Value::Date { value: "2024-02-29".to_owned() },
            Value::Datetime {
                value: "2024-02-29T12:34:56Z".to_owned(),
            },
            Value::Url {
                value: "https://example.org/".to_owned(),
            },
            Value::Coordinate {
                latitude: "40.1419".to_owned(),
                longitude: "29.9793".to_owned(),
                precision: Some("10".to_owned()),
            },
        ];

        for value in values {
            let mut diagnostics = Diagnostics::default();
            validate_value(Path::new("entity.yaml"), "Q1/S1", &value, &mut diagnostics);
            assert!(diagnostics.finish().is_empty(), "{value:?} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_validated_value_types() {
        let values = [
            Value::Decimal { value: "01.2".to_owned() },
            Value::Quantity {
                amount: "1e2".to_owned(),
                unit: "km".to_owned(),
            },
            Value::Quantity {
                amount: "1".to_owned(),
                unit: "  ".to_owned(),
            },
            Value::Date { value: "2023-02-29".to_owned() },
            Value::Datetime {
                value: "not a timestamp".to_owned(),
            },
            Value::Url {
                value: "relative/path".to_owned(),
            },
            Value::Coordinate {
                latitude: "91".to_owned(),
                longitude: "0".to_owned(),
                precision: None,
            },
            Value::Coordinate {
                latitude: "0".to_owned(),
                longitude: "0".to_owned(),
                precision: Some("0.00".to_owned()),
            },
        ];

        for value in values {
            let mut diagnostics = Diagnostics::default();
            validate_value(Path::new("entity.yaml"), "Q1/S1", &value, &mut diagnostics);
            assert_eq!(diagnostics.finish().len(), 1, "{value:?} should be invalid");
        }
    }
}
