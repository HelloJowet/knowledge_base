use super::super::{write_content, CommandError};
use chrono::{DateTime, NaiveDate};
use knowledge_base_crud::{EntityFilter, KnowledgeBase};
use knowledge_base_models::{EntityId, Property, PropertyId, Value, ValueType};
use std::process::ExitCode;
use url::Url;

pub fn execute(knowledge_base: &KnowledgeBase, raw_filters: &[String], limit: usize, offset: usize) -> Result<ExitCode, CommandError> {
    let filters = raw_filters.iter().map(|filter| parse_filter(knowledge_base, filter)).collect::<Result<Vec<_>, _>>()?;
    let page = knowledge_base.entities().query(&filters, limit, offset)?;
    let output = serde_yaml::to_string(&page).map_err(CommandError::Serialization)?;
    write_content(&output)
}

fn parse_filter(knowledge_base: &KnowledgeBase, filter: &str) -> Result<EntityFilter, CommandError> {
    let (property, raw_value) = filter
        .split_once('=')
        .ok_or_else(|| CommandError::InvalidFilter(format!("invalid entity filter {filter:?}; expected P<n>=value")))?;
    if raw_value.is_empty() {
        return Err(CommandError::InvalidFilter(format!("invalid entity filter {filter:?}; value must not be empty")));
    }
    let property = property
        .parse::<PropertyId>()
        .map_err(|error| CommandError::InvalidFilter(format!("invalid entity filter {filter:?}: {error}")))?;
    let source = knowledge_base.properties().read(&property)?;
    let definition =
        serde_yaml::from_str::<Property>(&source).map_err(|error| CommandError::InvalidFilter(format!("cannot parse property {property} while resolving filter: {error}")))?;
    if definition.id != property {
        return Err(CommandError::InvalidFilter(format!(
            "property file {}.yaml declares identifier {}",
            property, definition.id
        )));
    }
    let value = parse_value(definition.value_type, raw_value).map_err(|message| CommandError::InvalidFilter(format!("invalid value for property {property}: {message}")))?;
    Ok(EntityFilter { property, value })
}

fn parse_value(value_type: ValueType, raw: &str) -> Result<Value, String> {
    match value_type {
        ValueType::Entity => raw.parse::<EntityId>().map(|value| Value::Entity { value }).map_err(|error| error.to_string()),
        ValueType::String => Ok(Value::String { value: raw.to_owned() }),
        ValueType::Integer => raw
            .parse::<i64>()
            .map(|value| Value::Integer { value })
            .map_err(|_| "expected a base-10 integer".to_owned()),
        ValueType::Decimal => canonical_decimal(raw)
            .then(|| Value::Decimal { value: raw.to_owned() })
            .ok_or_else(|| "expected canonical base-10 decimal syntax".to_owned()),
        ValueType::Quantity => {
            let (amount, unit) = raw.split_once(',').ok_or_else(|| "expected amount,unit".to_owned())?;
            if !canonical_decimal(amount) {
                return Err("amount must use canonical base-10 decimal syntax".to_owned());
            }
            if unit.trim().is_empty() {
                return Err("unit must not be empty".to_owned());
            }
            Ok(Value::Quantity {
                amount: amount.to_owned(),
                unit: unit.to_owned(),
            })
        }
        ValueType::Boolean => match raw {
            "true" => Ok(Value::Boolean { value: true }),
            "false" => Ok(Value::Boolean { value: false }),
            _ => Err("expected true or false".to_owned()),
        },
        ValueType::Date if valid_partial_date(raw) => Ok(Value::Date { value: raw.to_owned() }),
        ValueType::Date => Err("expected a real ISO 8601 calendar year, month, or day (YYYY, YYYY-MM, or YYYY-MM-DD)".to_owned()),
        ValueType::Datetime => DateTime::parse_from_rfc3339(raw)
            .map(|_| Value::Datetime { value: raw.to_owned() })
            .map_err(|_| "expected an RFC 3339 timestamp".to_owned()),
        ValueType::Url => Url::parse(raw)
            .map(|_| Value::Url { value: raw.to_owned() })
            .map_err(|_| "expected an absolute URL".to_owned()),
        ValueType::Coordinate => {
            let (latitude, longitude) = raw.split_once(',').ok_or_else(|| "expected latitude,longitude".to_owned())?;
            if !canonical_decimal(latitude) || !within_absolute_bound(latitude, 90) {
                return Err("latitude must be canonical decimal text between -90 and 90".to_owned());
            }
            if !canonical_decimal(longitude) || !within_absolute_bound(longitude, 180) {
                return Err("longitude must be canonical decimal text between -180 and 180".to_owned());
            }
            Ok(Value::Coordinate {
                latitude: latitude.to_owned(),
                longitude: longitude.to_owned(),
                precision: None,
            })
        }
    }
}

fn canonical_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    if unsigned.is_empty() || unsigned.starts_with('+') {
        return false;
    }
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && (integer == "0" || !integer.starts_with('0'))
        && (!unsigned.contains('.') || (!fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit())))
}

fn valid_partial_date(value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::parse_value;
    use knowledge_base_models::{Value, ValueType};

    #[test]
    fn parses_every_property_value_type() {
        let cases = [
            (ValueType::Entity, "Q43", Value::Entity { value: "Q43".parse().unwrap() }),
            (ValueType::String, "a=b", Value::String { value: "a=b".to_owned() }),
            (ValueType::Integer, "-42", Value::Integer { value: -42 }),
            (ValueType::Decimal, "-0.25", Value::Decimal { value: "-0.25".to_owned() }),
            (
                ValueType::Quantity,
                "12.5,km",
                Value::Quantity {
                    amount: "12.5".to_owned(),
                    unit: "km".to_owned(),
                },
            ),
            (ValueType::Boolean, "true", Value::Boolean { value: true }),
            (ValueType::Date, "2024-02", Value::Date { value: "2024-02".to_owned() }),
            (
                ValueType::Datetime,
                "2024-02-29T12:34:56Z",
                Value::Datetime {
                    value: "2024-02-29T12:34:56Z".to_owned(),
                },
            ),
            (
                ValueType::Url,
                "https://example.com/?a=b",
                Value::Url {
                    value: "https://example.com/?a=b".to_owned(),
                },
            ),
            (
                ValueType::Coordinate,
                "40.1,-29.2",
                Value::Coordinate {
                    latitude: "40.1".to_owned(),
                    longitude: "-29.2".to_owned(),
                    precision: None,
                },
            ),
        ];

        for (value_type, raw, expected) in cases {
            assert_eq!(parse_value(value_type, raw).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_invalid_typed_values() {
        let cases = [
            (ValueType::Entity, "P1"),
            (ValueType::Integer, "1.5"),
            (ValueType::Decimal, "01.2"),
            (ValueType::Quantity, "01.2,km"),
            (ValueType::Quantity, "1.2,   "),
            (ValueType::Boolean, "yes"),
            (ValueType::Date, "2023-02-29"),
            (ValueType::Date, "2024-2-1"),
            (ValueType::Datetime, "yesterday"),
            (ValueType::Url, "relative/path"),
            (ValueType::Coordinate, "91,0"),
            (ValueType::Coordinate, "0,181"),
        ];

        for (value_type, raw) in cases {
            assert!(parse_value(value_type, raw).is_err(), "{value_type:?} accepted {raw:?}");
        }
    }
}
