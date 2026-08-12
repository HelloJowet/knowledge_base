use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

const API_URL: &str = "https://www.wikidata.org/w/api.php";
const USER_AGENT: &str = "JonasWikidataClient/1.0 (mailto:contact@jonasfrei.de)";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetrievedLabel {
    pub(crate) value: String,
    pub(crate) language: String,
    pub(crate) retrieved_at: String,
}

pub(crate) fn get_entity_label(id: &str) -> Result<Option<RetrievedLabel>> {
    get_entity_label_from(API_URL, id)
}

pub(crate) fn get_entity_label_from(api_url: &str, id: &str) -> Result<Option<RetrievedLabel>> {
    let response = request(api_url, id)?;
    let retrieved_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    Ok(select_label(&response, id)?.map(|(value, language)| RetrievedLabel { value, language, retrieved_at }))
}

fn request(api_url: &str, id: &str) -> Result<Value> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("could not create Wikidata HTTP client")?;
    client
        .get(api_url)
        .query(&[
            ("action", "wbgetentities"),
            ("ids", id),
            ("props", "labels"),
            ("languagefallback", "1"),
            ("format", "json"),
            ("formatversion", "2"),
        ])
        .send()
        .context("could not query Wikidata")?
        .error_for_status()
        .context("Wikidata returned an HTTP error")?
        .json()
        .context("could not decode the Wikidata response")
}

pub(crate) fn select_label(response: &Value, id: &str) -> Result<Option<(String, String)>> {
    if let Some(error) = response.get("error") {
        let code = error.get("code").and_then(Value::as_str).unwrap_or("unknown");
        let info = error.get("info").and_then(Value::as_str).unwrap_or("unspecified error");
        bail!("Wikidata API returned {code}: {info}");
    }
    let entities = response.get("entities").context("Wikidata response is missing entities")?;
    let entity = if let Some(entities) = entities.as_object() {
        entities.get(id)
    } else if let Some(entities) = entities.as_array() {
        entities.iter().find(|entity| entity.get("id").and_then(Value::as_str) == Some(id))
    } else {
        bail!("Wikidata response has invalid entities")
    };
    let Some(entity) = entity else {
        return Ok(None);
    };
    if entity.get("missing").is_some() {
        return Ok(None);
    }
    let Some(labels) = entity.get("labels").and_then(Value::as_object) else {
        return Ok(None);
    };
    Ok(labels.get("en").and_then(parse_label).or_else(|| labels.values().find_map(parse_label)))
}

fn parse_label(label: &Value) -> Option<(String, String)> {
    let value = label.get("value")?.as_str()?.trim();
    let language = label.get("language")?.as_str()?.trim();
    if value.is_empty() || language.is_empty() {
        return None;
    }
    Some((value.to_owned(), language.to_owned()))
}
