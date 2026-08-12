use knowledge_base_models::ValueType;
use serde::{Deserialize, Serialize};

/// The precision explicitly supplied with an inventory date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateStatement {
    pub property: String,
    pub value: CandidateValue,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<CandidateQualifier>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateQualifier {
    pub property: String,
    pub value: CandidateValue,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CandidateValue {
    Entity {
        id: String,
    },
    String {
        value: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    Date {
        value: String,
        precision: DatePrecision,
    },
    Datetime {
        value: String,
    },
    Url {
        value: String,
    },
    Coordinate {
        latitude: f64,
        longitude: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        precision_m: Option<f64>,
    },
    Quantity {
        amount: f64,
        unit: String,
    },
}

impl CandidateValue {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Entity { .. } => ValueType::Entity,
            Self::String { .. } => ValueType::String,
            Self::Integer { .. } => ValueType::Integer,
            Self::Decimal { .. } => ValueType::Decimal,
            Self::Boolean { .. } => ValueType::Boolean,
            Self::Date { .. } => ValueType::Date,
            Self::Datetime { .. } => ValueType::Datetime,
            Self::Url { .. } => ValueType::Url,
            Self::Coordinate { .. } => ValueType::Coordinate,
            Self::Quantity { .. } => ValueType::Quantity,
        }
    }
    pub fn entity_id(&self) -> Option<&str> {
        match self {
            Self::Entity { id } => Some(id),
            _ => None,
        }
    }
}
