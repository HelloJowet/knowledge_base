use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdAllocation {
    pub version: u64,
    pub next: NextIds,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NextIds {
    pub entity: u64,
    pub property: u64,
    pub reference: u64,
    pub entity_type: u64,
}
