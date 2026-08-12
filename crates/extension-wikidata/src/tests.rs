use super::*;
use knowledge_base_cli::Application;
use knowledge_base_extension_framework::manifest::ExtensionManifest;
use knowledge_base_extension_framework::registry::ExtensionRegistry;
use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::thread;

fn write_repository(root: &Path, next_reference: u64) {
    for directory in ["entities", "entity_types", "properties", "references"] {
        fs::create_dir_all(root.join(directory)).unwrap();
    }
    fs::write(
        root.join("id_allocation.yaml"),
        format!("version: 1\nnext: {{entity: 1, property: 1, reference: {next_reference}, entity_type: 1}}\n"),
    )
    .unwrap();
}

fn serve(status: &str, body: &str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let status = status.to_owned();
    let body = body.to_owned();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        String::from_utf8(request).unwrap()
    });
    (format!("http://{address}/w/api.php"), handle)
}

#[test]
fn accepts_only_canonical_wikidata_item_ids() {
    assert!(is_canonical_item_id("Q42"));
    for value in ["q42", "Q", "Q0", "Q042", "P42", "Q42|Q43"] {
        assert!(!is_canonical_item_id(value), "accepted {value}");
    }
}

#[test]
fn label_selection_prefers_english_and_preserves_a_usable_fallback() {
    let english = json!({"entities": {"Q1": {"labels": {
        "de": {"language": "de", "value": "Universum"},
        "en": {"language": "en", "value": "Universe"}
    }}}});
    assert_eq!(labels::select_label(&english, "Q1").unwrap(), Some(("Universe".to_owned(), "en".to_owned())));

    let fallback = json!({"entities": {"Q1": {"labels": {
        "en": {"language": "tr", "for-language": "en", "value": "Evren"}
    }}}});
    assert_eq!(labels::select_label(&fallback, "Q1").unwrap(), Some(("Evren".to_owned(), "tr".to_owned())));
}

#[test]
fn label_selection_rejects_missing_blank_and_api_error_responses() {
    let missing = json!({"entities": {"Q1": {"missing": ""}}});
    let blank = json!({"entities": {"Q1": {"labels": {"en": {"language": "en", "value": " "}}}}});
    assert_eq!(labels::select_label(&missing, "Q1").unwrap(), None);
    assert_eq!(labels::select_label(&blank, "Q1").unwrap(), None);
    assert!(labels::select_label(&json!({"error": {"code": "bad", "info": "no"}}), "Q1").is_err());
}

#[test]
fn label_http_client_sends_the_expected_request_and_records_utc_time() {
    let body = json!({"entities": {"Q42": {"labels": {"en": {"language": "en", "value": "Douglas Adams"}}}}}).to_string();
    let (url, server) = serve("200 OK", &body);
    let label = labels::get_entity_label_from(&url, "Q42").unwrap().unwrap();
    let request = server.join().unwrap();
    assert!(request.contains("action=wbgetentities"));
    assert!(request.contains("ids=Q42"));
    assert!(request.contains("languagefallback=1"));
    assert_eq!(label.value, "Douglas Adams");
    assert_eq!(label.language, "en");
    assert!(chrono::DateTime::parse_from_rfc3339(&label.retrieved_at).is_ok());
    assert!(label.retrieved_at.ends_with('Z'));
}

#[test]
fn label_http_client_rejects_http_and_malformed_responses() {
    let (failure_url, failure_server) = serve("500 Internal Server Error", "{}");
    assert!(labels::get_entity_label_from(&failure_url, "Q42").is_err());
    failure_server.join().unwrap();

    let (malformed_url, malformed_server) = serve("200 OK", "not json");
    assert!(labels::get_entity_label_from(&malformed_url, "Q42").is_err());
    malformed_server.join().unwrap();

    let (incomplete_url, incomplete_server) = serve("200 OK", "{}");
    assert!(labels::get_entity_label_from(&incomplete_url, "Q42").is_err());
    incomplete_server.join().unwrap();
}

#[test]
fn registration_writes_new_reference_metadata_and_advances_allocation() {
    let temporary = tempfile::tempdir().unwrap();
    write_repository(temporary.path(), 7);
    let repository = knowledge_base_crud::KnowledgeBaseRepository::new(temporary.path());

    let outcome = register_reference(&repository, "Q42", |_| {
        Ok(Some(labels::RetrievedLabel {
            value: "Douglas Adams".to_owned(),
            language: "en".to_owned(),
            retrieved_at: "2026-08-03T13:20:03Z".to_owned(),
        }))
    })
    .unwrap();

    assert_eq!(outcome.status, ReferenceRegistrationStatus::Registered);
    assert_eq!(outcome.reference.to_string(), "R7");
    let reference = fs::read_to_string(temporary.path().join("references/R7.yaml")).unwrap();
    assert!(reference.contains("url: https://www.wikidata.org/wiki/Q42?uselang=en"));
    assert!(reference.contains("title: Douglas Adams"));
    assert!(reference.contains("publisher: Wikidata"));
    assert!(reference.contains("source_language: en"));
    assert!(reference.contains("retrieved_at: 2026-08-03T13:20:03Z"));
    assert!(fs::read_to_string(temporary.path().join("id_allocation.yaml")).unwrap().contains("reference: 8"));
}

#[test]
fn registration_reuses_the_exact_url_without_lookup_or_allocation_change() {
    let temporary = tempfile::tempdir().unwrap();
    write_repository(temporary.path(), 2);
    fs::write(
        temporary.path().join("references/R1.yaml"),
        "id: R1\nurl: https://www.wikidata.org/wiki/Q42?uselang=en\ntitle: Existing\npublisher: Wikidata\nsource_language: en\nretrieved_at: 2026-08-01T10:00:00Z\n",
    )
    .unwrap();
    let repository = knowledge_base_crud::KnowledgeBaseRepository::new(temporary.path());

    let outcome = register_reference(&repository, "Q42", |_| panic!("existing reference must not fetch a label")).unwrap();
    assert_eq!(outcome.status, ReferenceRegistrationStatus::Existing);
    assert_eq!(outcome.reference.to_string(), "R1");
    assert!(fs::read_to_string(temporary.path().join("id_allocation.yaml")).unwrap().contains("reference: 2"));
}

#[test]
fn failed_or_missing_label_registration_leaves_the_repository_unchanged() {
    let temporary = tempfile::tempdir().unwrap();
    write_repository(temporary.path(), 1);
    let repository = knowledge_base_crud::KnowledgeBaseRepository::new(temporary.path());
    assert!(register_reference(&repository, "Q42", |_| Ok(None)).is_err());
    assert!(register_reference(&repository, "Q42", |_| Err(anyhow::anyhow!("HTTP failed"))).is_err());
    assert!(!temporary.path().join("references/R1.yaml").exists());
    assert!(fs::read_to_string(temporary.path().join("id_allocation.yaml")).unwrap().contains("reference: 1"));
}

#[test]
fn extension_contract_uses_a_non_production_property_binding_and_command_is_namespaced() {
    let temporary = tempfile::tempdir().unwrap();
    write_repository(temporary.path(), 1);
    fs::write(
        temporary.path().join("extensions.yaml"),
        "version: 1\nextensions:\n  wikidata:\n    contract: 1\n    properties:\n      item_id_property: P99\n",
    )
    .unwrap();
    fs::write(
        temporary.path().join("properties/P99.yaml"),
        "id: P99\nlabels: {}\nsubject_types: []\nvalue_type: string\nusage: statement\ncardinality: one\n",
    )
    .unwrap();
    let core: Arc<dyn KnowledgeBaseExtension> = Arc::new(WikidataExtension::new());
    let registry = ExtensionRegistry::new(vec![core]).unwrap();
    ExtensionManifest::load_and_activate(temporary.path(), &registry).unwrap();

    let application = Application::builder().with_extension(WikidataExtension::new()).build().unwrap();
    assert!(
        application
            .command()
            .try_get_matches_from(["knowledge-base", "extension", "wikidata", "reference", "register", "Q42"])
            .is_ok()
    );
    assert!(
        application
            .command()
            .try_get_matches_from(["knowledge-base", "extension", "wikidata", "reference", "register", "q42"])
            .is_ok()
    );
    assert!(
        !knowledge_base_cli::Application::builder()
            .build()
            .unwrap()
            .command()
            .get_subcommands()
            .find(|command| command.get_name() == "extension")
            .unwrap()
            .get_subcommands()
            .any(|command| command.get_name() == "wikidata")
    );
}
