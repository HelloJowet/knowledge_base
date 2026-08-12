use super::{Entities, query::load_entities};
use crate::Error;
use knowledge_base_models::Entity;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct EntitySearchPage {
    pub query: String,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    pub entities: Vec<Entity>,
}

impl Entities<'_> {
    pub fn search(&self, query: &str, limit: usize, offset: usize) -> Result<EntitySearchPage, Error> {
        let query = normalize(query);
        if query.is_empty() {
            return Err(Error::InvalidRequest("entity search query must not be empty".to_owned()));
        }
        if limit == 0 {
            return Err(Error::InvalidRequest("entity search limit must be greater than zero".to_owned()));
        }

        let mut entities = load_entities(self.repository.root())?
            .into_iter()
            .filter(|entity| entity.labels.values().any(|label| normalize(&label.text).contains(&query)))
            .collect::<Vec<_>>();
        entities.sort_by(|left, right| {
            has_exact_label(left, &query)
                .cmp(&has_exact_label(right, &query))
                .reverse()
                .then_with(|| left.id.number().cmp(&right.id.number()))
        });

        let total = entities.len();
        let entities = entities.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
        let returned_end = offset.saturating_add(entities.len());
        let next_offset = (returned_end < total).then_some(returned_end);

        Ok(EntitySearchPage {
            query,
            offset,
            limit,
            total,
            next_offset,
            entities,
        })
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn has_exact_label(entity: &Entity, query: &str) -> bool {
    entity.labels.values().any(|label| normalize(&label.text) == query)
}
