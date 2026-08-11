use crate::ReferenceId;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub type LocalizedMap = BTreeMap<String, LocalizedText>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    pub text: String,
    pub references: Vec<ReferenceId>,
}

pub(crate) fn deserialize_optional_non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
