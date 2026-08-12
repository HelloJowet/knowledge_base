use std::{collections::HashSet, hash::Hash, path::Path};

use chrono::{DateTime, NaiveDate};
use knowledge_base_models::{EntityId, ReferenceId};
use url::Url;

use super::diagnostics::{DiagnosticFactory, ValidationReport};
use crate::{CandidateValue, DatePrecision};

pub(crate) fn candidate_id(value: &str) -> bool {
    value
        .strip_prefix('C')
        .is_some_and(|number| number.len() >= 3 && number.bytes().all(|byte| byte.is_ascii_digit()))
}
pub(crate) fn evidence_id(value: &str) -> bool {
    value
        .strip_prefix('E')
        .is_some_and(|number| number.len() >= 3 && number.bytes().all(|byte| byte.is_ascii_digit()))
}
pub(crate) fn draft_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|number| !number.is_empty() && !number.starts_with('0') && number.bytes().all(|byte| byte.is_ascii_digit()))
}
pub(crate) fn is_identifier(value: &str, prefix: &str) -> bool {
    match prefix {
        "Q" => value.parse::<EntityId>().is_ok(),
        "R" => value.parse::<ReferenceId>().is_ok(),
        _ => false,
    }
}

pub(crate) fn require_unique<T: Eq + Hash + std::fmt::Display>(report: &mut ValidationReport, path: &Path, context: &str, values: &[T]) {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value) {
            report.push(DiagnosticFactory::domain(path, context, format!("contains duplicate value {value}")));
        }
    }
}

pub(crate) fn validate_text_list(report: &mut ValidationReport, path: &Path, context: &str, values: &[String]) {
    require_unique(report, path, context, values);
    for (index, value) in values.iter().enumerate() {
        report.require_nonempty(path, &format!("{context}[{index}]"), value);
    }
}

pub(crate) fn validate_candidate_value(report: &mut ValidationReport, path: &Path, context: &str, value: &CandidateValue) {
    match value {
        CandidateValue::Entity { id } if !is_identifier(id, "Q") && !candidate_id(id) => {
            report.push(DiagnosticFactory::domain(path, context, format!("invalid entity target {id}")))
        }
        CandidateValue::String { value } => report.require_nonempty(path, context, value),
        CandidateValue::Date { value, precision } => validate_date(report, path, context, value, *precision),
        CandidateValue::Datetime { value } if !(value.ends_with('Z') && DateTime::parse_from_rfc3339(value).is_ok_and(|date| date.offset().local_minus_utc() == 0)) => {
            report.push(DiagnosticFactory::domain(path, context, "datetime must be a valid UTC RFC 3339 timestamp ending in Z"))
        }
        CandidateValue::Url { value } if !Url::parse(value).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some() && value.starts_with("https://")) => {
            report.push(DiagnosticFactory::domain(path, context, "URL must be an absolute HTTPS URL"))
        }
        CandidateValue::Decimal { value } if !value.is_finite() => report.push(DiagnosticFactory::domain(path, context, "decimal must be finite")),
        CandidateValue::Coordinate { latitude, longitude, precision_m } => {
            if !latitude.is_finite() || !(-90.0..=90.0).contains(latitude) {
                report.push(DiagnosticFactory::domain(path, context, "latitude must be finite and between -90 and 90"));
            }
            if !longitude.is_finite() || !(-180.0..=180.0).contains(longitude) {
                report.push(DiagnosticFactory::domain(path, context, "longitude must be finite and between -180 and 180"));
            }
            if precision_m.is_some_and(|precision| !precision.is_finite() || precision <= 0.0) {
                report.push(DiagnosticFactory::domain(path, context, "precision_m must be finite and positive"));
            }
        }
        CandidateValue::Quantity { amount, unit } => {
            if !amount.is_finite() {
                report.push(DiagnosticFactory::domain(path, context, "quantity amount must be finite"));
            }
            report.require_nonempty(path, context, unit);
        }
        _ => {}
    }
}

fn validate_date(report: &mut ValidationReport, path: &Path, context: &str, value: &str, precision: DatePrecision) {
    let expected_length = match precision {
        DatePrecision::Year => 4,
        DatePrecision::Month => 7,
        DatePrecision::Day => 10,
    };
    let valid = match value.len() {
        4 => value.parse::<i32>().is_ok(),
        7 => value
            .get(..4)
            .and_then(|year| year.parse::<i32>().ok())
            .zip(value.get(5..).and_then(|month| month.parse::<u32>().ok()))
            .is_some_and(|(year, month)| value.as_bytes().get(4) == Some(&b'-') && NaiveDate::from_ymd_opt(year, month, 1).is_some()),
        10 => NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        _ => false,
    };
    if value.len() != expected_length || !valid {
        report.push(DiagnosticFactory::domain(path, context, format!("date does not match its {precision:?} precision")));
    }
}
