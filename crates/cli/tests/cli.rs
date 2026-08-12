use chrono::DateTime;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

fn knowledge_base_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_knowledge-base"))
}

fn copied_fixture() -> tempfile::TempDir {
    let destination = tempfile::tempdir().expect("temporary knowledge base");
    for directory in ["entities", "entity_types", "properties", "references", "entity_context"] {
        let source_directory = fixture().join(directory);
        if !source_directory.exists() {
            continue;
        }
        fs::create_dir(destination.path().join(directory)).expect("create fixture directory");
        for entry in fs::read_dir(source_directory).expect("read fixture directory") {
            let entry = entry.expect("read fixture entry");
            fs::copy(entry.path(), destination.path().join(directory).join(entry.file_name())).expect("copy fixture file");
        }
    }
    fs::copy(fixture().join("id_allocation.yaml"), destination.path().join("id_allocation.yaml")).expect("copy allocation");
    fs::copy(fixture().join("extensions.yaml"), destination.path().join("extensions.yaml")).expect("copy extension manifest");
    destination
}

fn write_manifest(root: &Path, source: &str) -> PathBuf {
    let path = root.join("statement-manifest.yaml");
    fs::write(&path, source).expect("write statement manifest");
    path
}

#[test]
fn read_commands_print_files_exactly_as_stored() {
    let root = fixture();
    let cases = [
        (["entity", "read", "Q1"], "entities/Q1.yaml"),
        (["entity-type", "read", "T1"], "entity_types/T1.yaml"),
        (["property", "read", "P1"], "properties/P1.yaml"),
        (["reference", "read", "R1"], "references/R1.yaml"),
        (["entity-context", "read", "Q1"], "entity_context/Q1.md"),
    ];

    for (arguments, relative_path) in cases {
        let output = knowledge_base_command()
            .args(arguments)
            .env("KNOWLEDGE_BASE_PATH", &root)
            .output()
            .expect("run knowledge-base command");

        assert!(output.status.success(), "{}: {}", relative_path, String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, fs::read(root.join(relative_path)).expect("read expected file"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn read_fails_closed_when_the_extension_manifest_is_missing() {
    let root = tempfile::tempdir().expect("temporary knowledge base");
    fs::create_dir(root.path().join("entities")).expect("create entity directory");
    fs::write(root.path().join("entities/Q1.yaml"), b"id: Q1").expect("write entity");

    let output = knowledge_base_command()
        .args(["entity", "read", "Q1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run entity read");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("extensions.yaml"));
}

#[test]
fn validate_uses_the_environment_configured_root() {
    let root = fixture();
    let output = knowledge_base_command().arg("validate").env("KNOWLEDGE_BASE_PATH", &root).output().expect("run validation");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("valid knowledge base: {}\n", root.display()));
}

#[test]
fn ingestion_candidate_inventory_validation_uses_the_generic_cli() {
    let bundle = tempfile::tempdir().expect("temporary retrieval bundle");
    fs::write(bundle.path().join("page.html"), "page").expect("write source page");
    let inventory = bundle.path().join("ingestion_candidate_inventory.yaml");
    fs::write(
        &inventory,
        "source_reference: R1\nsource_file: page.html\nevidence: []\ndraft_entity_types: []\ndraft_properties: []\narticle_results: []\ncandidates: []\n",
    )
    .expect("write inventory");

    let output = knowledge_base_command()
        .args(["ingestion", "candidate-inventory", "validate"])
        .arg(&inventory)
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("validate ingestion candidate inventory");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("valid ingestion candidate inventory: {}\n", inventory.display())
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn ingestion_candidate_inventory_diagnostics_use_stderr_and_fail() {
    let bundle = tempfile::tempdir().expect("temporary retrieval bundle");
    fs::write(bundle.path().join("page.html"), "page").expect("write source page");
    let inventory = bundle.path().join("ingestion_candidate_inventory.yaml");
    fs::write(
        &inventory,
        "source_reference: R999\nsource_file: page.html\nevidence: []\ndraft_entity_types: []\ndraft_properties: []\narticle_results: []\ncandidates: []\n",
    )
    .expect("write inventory");

    let output = knowledge_base_command()
        .args(["ingestion", "candidate-inventory", "validate"])
        .arg(&inventory)
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("validate invalid ingestion candidate inventory");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unresolved production reference R999"));
}

#[test]
fn ingestion_candidate_inventory_rejects_legacy_filename_and_bad_snapshot() {
    let bundle = tempfile::tempdir().expect("temporary retrieval bundle");
    let legacy = bundle.path().join("candidate_inventory.yaml");
    fs::write(&legacy, "ignored").expect("write legacy inventory");
    let legacy_output = knowledge_base_command()
        .args(["ingestion", "candidate-inventory", "validate"])
        .arg(&legacy)
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("validate legacy inventory path");
    assert!(!legacy_output.status.success());
    assert!(legacy_output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&legacy_output.stderr).contains("filename must be ingestion_candidate_inventory.yaml"));

    let missing_root = bundle.path().join("missing-knowledge-base");
    let snapshot_output = knowledge_base_command()
        .args(["ingestion", "candidate-inventory", "validate"])
        .arg(bundle.path().join("ingestion_candidate_inventory.yaml"))
        .env("KNOWLEDGE_BASE_PATH", missing_root)
        .output()
        .expect("validate with missing snapshot");
    assert!(!snapshot_output.status.success());
    assert!(snapshot_output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&snapshot_output.stderr).contains("cannot read"));
}

#[test]
fn ingestion_candidate_inventory_help_is_available_without_configuration() {
    let output = knowledge_base_command()
        .args(["ingestion", "candidate-inventory", "validate", "--help"])
        .env_remove("KNOWLEDGE_BASE_PATH")
        .output()
        .expect("show ingestion validation help");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("ingestion_candidate_inventory.yaml"));
}

#[test]
fn ingestion_retrieval_commands_have_the_expected_configuration_boundary() {
    let fetch_help = knowledge_base_command()
        .args(["ingestion", "retrieval", "fetch", "--help"])
        .env_remove("KNOWLEDGE_BASE_PATH")
        .output()
        .expect("show retrieval fetch help");
    assert!(fetch_help.status.success(), "{}", String::from_utf8_lossy(&fetch_help.stderr));
    assert!(String::from_utf8_lossy(&fetch_help.stdout).contains("Web page URL to fetch"));

    let register_without_root = knowledge_base_command()
        .args(["ingestion", "retrieval", "register", "/tmp/bundle"])
        .env_remove("KNOWLEDGE_BASE_PATH")
        .output()
        .expect("register retrieval bundle without configuration");
    assert!(!register_without_root.status.success());
    assert!(register_without_root.stdout.is_empty());
    assert!(String::from_utf8_lossy(&register_without_root.stderr).contains("KNOWLEDGE_BASE_PATH must be set"));
}

#[test]
fn ingestion_retrieval_registers_and_previews_a_bundle() {
    let root = copied_fixture();
    let bundle = tempfile::tempdir().expect("temporary retrieval bundle");
    fs::write(bundle.path().join("page.html"), "page").expect("write page");
    fs::write(
        bundle.path().join("retrieval.yaml"),
        "schema_version: 1\nrequested_url: https://example.com/start\nurl: https://example.com/page\ntitle: Example\nsource_language: en\nretrieved_at: '2026-08-03T13:20:03Z'\n",
    )
    .expect("write metadata");

    let preview = knowledge_base_command()
        .args(["ingestion", "retrieval", "register"])
        .arg(bundle.path())
        .arg("--dry-run")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("preview bundle registration");
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    assert!(String::from_utf8_lossy(&preview.stdout).contains("status: previewed"));
    assert!(!root.path().join("references/R2.yaml").exists());

    let registered = knowledge_base_command()
        .args(["ingestion", "retrieval", "register"])
        .arg(bundle.path())
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("register bundle");
    assert!(registered.status.success(), "{}", String::from_utf8_lossy(&registered.stderr));
    assert!(String::from_utf8_lossy(&registered.stdout).contains("status: registered"));
    assert!(root.path().join("references/R2.yaml").is_file());
}

#[test]
fn configured_root_is_required_for_executable_commands() {
    for value in [None, Some("")] {
        let mut command = knowledge_base_command();
        command.arg("validate").env_remove("KNOWLEDGE_BASE_PATH");
        if let Some(value) = value {
            command.env("KNOWLEDGE_BASE_PATH", value);
        }

        let output = command.output().expect("run validation");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("KNOWLEDGE_BASE_PATH must be set"));
    }
}

#[test]
fn help_and_version_do_not_require_configuration() {
    for argument in ["--help", "--version"] {
        let output = knowledge_base_command()
            .arg(argument)
            .env_remove("KNOWLEDGE_BASE_PATH")
            .output()
            .expect("run informational command");

        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn extension_list_and_check_report_a_valid_base_only_manifest() {
    let root = fixture();
    let list = knowledge_base_command()
        .args(["extension", "list"])
        .env("KNOWLEDGE_BASE_PATH", &root)
        .output()
        .expect("list extensions");
    assert!(list.status.success(), "{}", String::from_utf8_lossy(&list.stderr));
    assert_eq!(String::from_utf8(list.stdout).unwrap(), "version: 1\nextensions: {}\n");
    assert!(list.stderr.is_empty());

    let check = knowledge_base_command()
        .args(["extension", "check"])
        .env("KNOWLEDGE_BASE_PATH", &root)
        .output()
        .expect("check extensions");
    assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));
    assert_eq!(String::from_utf8(check.stdout).unwrap(), "version: 1\nstatus: valid\n");
    assert!(check.stderr.is_empty());
}

#[test]
fn extension_list_reports_unavailable_extensions_and_repository_commands_fail_closed() {
    let root = copied_fixture();
    fs::write(root.path().join("extensions.yaml"), "version: 1\nextensions:\n  unavailable:\n    contract: 1\n").expect("write invalid extension manifest");

    let list = knowledge_base_command()
        .args(["extension", "list"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("list extensions");
    assert!(!list.status.success());
    assert_eq!(
        String::from_utf8(list.stdout).unwrap(),
        "version: 1\nextensions:\n  unavailable:\n    declared_contract: 1\n    available_contract: null\n    active: false\n    incompatible: true\n"
    );
    assert!(String::from_utf8_lossy(&list.stderr).contains("not compiled"));

    let check = knowledge_base_command()
        .args(["extension", "check"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("check extensions");
    assert!(!check.status.success());
    assert!(check.stdout.is_empty());
    assert!(String::from_utf8_lossy(&check.stderr).contains("not compiled"));

    let read = knowledge_base_command()
        .args(["entity", "read", "Q1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("read with unavailable extension");
    assert!(!read.status.success());
    assert!(read.stdout.is_empty());
    assert!(String::from_utf8_lossy(&read.stderr).contains("not compiled"));
}

#[test]
fn malformed_or_wrong_prefix_identifiers_are_usage_errors() {
    for identifier in ["P1", "../Q1", "Q1.yaml"] {
        let output = knowledge_base_command()
            .args(["entity", "read", identifier])
            .env("KNOWLEDGE_BASE_PATH", fixture())
            .output()
            .expect("run entity read");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid identifier"));
    }
}

#[test]
fn missing_records_and_optional_context_are_read_errors() {
    for arguments in [["entity", "read", "Q999"], ["entity-context", "read", "Q2"]] {
        let output = knowledge_base_command()
            .args(arguments)
            .env("KNOWLEDGE_BASE_PATH", fixture())
            .output()
            .expect("run read command");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn relationships_show_direct_incoming_and_outgoing_edges() {
    let cases = [
        (
            "Q2",
            "entity: Q2\noffset: 0\nlimit: 100\ntotal: 1\nrelationships:\n- direction: incoming\n  entity:\n    id: Q1\n    labels:\n      tr: BİLECİK\n  property: P3\n  statement: S3\n",
        ),
        (
            "Q1",
            "entity: Q1\noffset: 0\nlimit: 100\ntotal: 1\nrelationships:\n- direction: outgoing\n  entity:\n    id: Q2\n    labels:\n      en: Türkiye\n  property: P3\n  statement: S3\n",
        ),
    ];

    for (id, expected) in cases {
        let output = knowledge_base_command()
            .args(["entity", "relationships", id])
            .env("KNOWLEDGE_BASE_PATH", fixture())
            .output()
            .expect("run entity relationships");

        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn entity_query_returns_full_entities_with_filter_and_pagination_metadata() {
    let output = knowledge_base_command()
        .args(["entity", "query", "--filter", "P3=Q2"])
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("run entity query");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let page: serde_yaml::Value = serde_yaml::from_slice(&output.stdout).unwrap();
    assert_eq!(page["filters"][0]["property"], "P3");
    assert_eq!(page["filters"][0]["value"]["type"], "entity");
    assert_eq!(page["filters"][0]["value"]["value"], "Q2");
    assert_eq!(page["offset"], 0);
    assert_eq!(page["limit"], 100);
    assert_eq!(page["total"], 1);
    assert!(page.get("next_offset").is_none());
    assert_eq!(page["entities"][0]["id"], "Q1");
    assert_eq!(page["entities"][0]["statements"][2]["id"], "S3");
    assert!(output.stderr.is_empty());
}

#[test]
fn entity_search_returns_ordered_paginated_canonical_entities() {
    let root = copied_fixture();
    fs::write(
        root.path().join("entities/Q3.yaml"),
        "id: Q3\nlabels:\n  en:\n    text: Türkiye Cumhuriyeti\n    references: [R1]\nentity_types: []\nstatements: []\n",
    )
    .expect("write search fixture entity");

    let output = knowledge_base_command()
        .args(["entity", "search", " türkiye ", "--limit", "1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run entity search");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let page: serde_yaml::Value = serde_yaml::from_slice(&output.stdout).unwrap();
    assert_eq!(page["query"], "türkiye");
    assert_eq!(page["offset"], 0);
    assert_eq!(page["limit"], 1);
    assert_eq!(page["total"], 2);
    assert_eq!(page["next_offset"], 1);
    assert_eq!(page["entities"][0]["id"], "Q2");
    assert!(page["entities"][0].get("statements").is_some());
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_entity_searches_fail_without_output() {
    for arguments in [vec!["entity", "search", " "], vec!["entity", "search", "Türkiye", "--limit", "0"]] {
        let output = knowledge_base_command()
            .args(arguments)
            .env("KNOWLEDGE_BASE_PATH", fixture())
            .output()
            .expect("run invalid entity search");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn entity_query_ands_filters_and_returns_empty_pages() {
    let matching = knowledge_base_command()
        .args(["entity", "query", "--filter", "P3=Q2", "--filter", "P1=228334", "--limit", "1"])
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("run matching entity query");
    assert!(matching.status.success(), "{}", String::from_utf8_lossy(&matching.stderr));
    let matching: serde_yaml::Value = serde_yaml::from_slice(&matching.stdout).unwrap();
    assert_eq!(matching["total"], 1);
    assert_eq!(matching["entities"][0]["id"], "Q1");

    let empty = knowledge_base_command()
        .args(["entity", "query", "--filter", "P3=Q2", "--offset", "99"])
        .env("KNOWLEDGE_BASE_PATH", fixture())
        .output()
        .expect("run empty entity query page");
    assert!(empty.status.success(), "{}", String::from_utf8_lossy(&empty.stderr));
    let empty: serde_yaml::Value = serde_yaml::from_slice(&empty.stdout).unwrap();
    assert_eq!(empty["total"], 1);
    assert_eq!(empty["entities"], serde_yaml::Value::Sequence(Vec::new()));
}

#[test]
fn invalid_entity_queries_fail_without_output() {
    let cases = [
        vec!["entity", "query"],
        vec!["entity", "query", "--filter", "P3"],
        vec!["entity", "query", "--filter", "P3="],
        vec!["entity", "query", "--filter", "P3=P1"],
        vec!["entity", "query", "--filter", "P999=Q2"],
        vec!["entity", "query", "--filter", "P3=Q2", "--limit", "0"],
    ];

    for arguments in cases {
        let output = knowledge_base_command()
            .args(arguments)
            .env("KNOWLEDGE_BASE_PATH", fixture())
            .output()
            .expect("run invalid entity query");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn relationships_are_one_hop_and_paginated_after_canonical_sorting() {
    let root = copied_fixture();
    fs::write(
        root.path().join("entities/Q3.yaml"),
        "id: Q3\nlabels:\n  en:\n    text: District\n    references: [R1]\nentity_types:\n  - value: T1\n    references: [R1]\nstatements:\n  - id: S1\n    property: P3\n    value:\n      type: entity\n      value: Q1\n    references: [R1]\n",
    )
    .unwrap();
    fs::write(
        root.path().join("entities/Q4.yaml"),
        "id: Q4\nlabels:\n  en:\n    text: Ankara\n    references: [R1]\nentity_types:\n  - value: T1\n    references: [R1]\nstatements:\n  - id: S1\n    property: P3\n    value:\n      type: entity\n      value: Q2\n    references: [R1]\n",
    )
    .unwrap();

    let first = knowledge_base_command()
        .args(["entity", "relationships", "Q2", "--limit", "1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run first relationship page");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let first = String::from_utf8(first.stdout).unwrap();
    assert!(first.contains("total: 2\nnext_offset: 1\n"));
    assert!(first.contains("id: Q1"));
    assert!(!first.contains("id: Q3"));
    assert!(!first.contains("id: Q4"));

    let second = knowledge_base_command()
        .args(["entity", "relationships", "Q2", "--limit", "1", "--offset", "1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run second relationship page");
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    let second = String::from_utf8(second.stdout).unwrap();
    assert!(second.contains("total: 2\nrelationships:"));
    assert!(!second.contains("next_offset"));
    assert!(second.contains("id: Q4"));
    assert!(!second.contains("id: Q3"));

    let empty = knowledge_base_command()
        .args(["entity", "relationships", "Q2", "--offset", "99"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run empty relationship page");
    assert!(empty.status.success(), "{}", String::from_utf8_lossy(&empty.stderr));
    assert!(String::from_utf8(empty.stdout).unwrap().ends_with("total: 2\nrelationships: []\n"));
}

#[test]
fn invalid_relationship_queries_fail_without_output() {
    let malformed_root = copied_fixture();
    fs::write(malformed_root.path().join("entities/Q2.yaml"), "not: an entity\n").unwrap();
    let fixture_root = fixture();
    let cases = [
        (vec!["entity", "relationships", "Q999"], fixture_root.as_path()),
        (vec!["entity", "relationships", "Q2"], malformed_root.path()),
        (vec!["entity", "relationships", "Q2", "--limit", "0"], fixture_root.as_path()),
    ];

    for (arguments, root) in cases {
        let output = knowledge_base_command()
            .args(arguments)
            .env("KNOWLEDGE_BASE_PATH", root)
            .output()
            .expect("run invalid relationship query");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn non_utf8_resources_are_read_errors() {
    let root = tempfile::tempdir().expect("temporary knowledge base");
    fs::create_dir(root.path().join("entities")).expect("create entity directory");
    fs::write(root.path().join("entities/Q1.yaml"), [0xff]).expect("write invalid UTF-8");

    let output = knowledge_base_command()
        .args(["entity", "read", "Q1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run entity read");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
    assert!(output.stdout.is_empty());
}

#[test]
fn statement_apply_dry_run_reports_planned_addition_without_changing_entity() {
    let root = copied_fixture();
    let manifest = write_manifest(
        root.path(),
        "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 123456789 }\n    references: [R1]\n",
    );
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();

    let output = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .arg("--dry-run")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run statement dry-run");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "outcome: previewed\nresults:\n- index: 1\n  entity: Q1\n  property: P1\n  statement: S4\n  status: would_add\n"
    );
    assert_eq!(fs::read(entity_path).unwrap(), before);
}

#[test]
fn statement_apply_preserves_text_and_repeated_apply_is_rejected() {
    let root = copied_fixture();
    let manifest_source = "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 123456789 }\n    references: [R1]\n";
    let manifest = write_manifest(root.path(), manifest_source);
    let entity_path = root.path().join("entities/Q1.yaml");
    let source = fs::read_to_string(&entity_path).unwrap().replace("statements:\n", "# retained comment\nstatements:\n");
    fs::write(&entity_path, &source).unwrap();

    let applied = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("apply statement manifest");
    assert!(applied.status.success(), "{}", String::from_utf8_lossy(&applied.stderr));
    let changed = fs::read_to_string(&entity_path).unwrap();
    assert!(changed.starts_with(&source));
    assert!(changed.contains("  - id: S4\n    property: P1\n"));
    assert!(changed.contains("# retained comment"));
    assert_eq!(fs::read_to_string(&manifest).unwrap(), manifest_source);

    let repeated = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .arg("--dry-run")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("repeat statement manifest");
    assert!(!repeated.status.success());
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("status: already_present"));
    assert_eq!(fs::read_to_string(&entity_path).unwrap(), changed);
}

#[test]
fn statement_apply_serializes_qualifiers_and_repeated_apply_is_rejected() {
    let root = copied_fixture();
    let manifest = write_manifest(
        root.path(),
        "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 123456789 }\n    qualifiers:\n      - property: P2\n        value: { type: date, value: 2024-01-01 }\n    references: [R1]\n",
    );
    let entity_path = root.path().join("entities/Q1.yaml");

    let applied = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("apply qualified statement manifest");
    assert!(applied.status.success(), "{}", String::from_utf8_lossy(&applied.stderr));
    assert!(fs::read_to_string(&entity_path).unwrap().contains("    qualifiers:\n      - property: P2\n"));

    let repeated = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .arg("--dry-run")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("repeat qualified statement manifest");
    assert!(!repeated.status.success());
    assert!(String::from_utf8_lossy(&repeated.stdout).contains("status: already_present"));
}

#[test]
fn duplicate_manifest_rows_are_reported_and_never_written() {
    let root = copied_fixture();
    let manifest = write_manifest(
        root.path(),
        "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    references: [R1]\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    references: [R1]\n",
    );
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();

    let output = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("apply duplicate statement manifest");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("statement: S4\n  status: would_add"));
    assert!(stdout.contains("statement: S4\n  status: already_present"));
    assert!(stdout.starts_with("outcome: not_applied\n"));
    assert_eq!(fs::read(entity_path).unwrap(), before);
}

#[test]
fn invalid_statement_manifests_fail_without_results_or_writes() {
    let cases = [
        (
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: string, value: wrong-type }\n    references: [R1]\n",
            "requires integer values",
        ),
        (
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    references: []\n",
            "references must not be empty",
        ),
        (
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    qualifiers:\n      - property: P2\n        value: { type: date, value: 2024-01-01 }\n      - property: P2\n        value: { type: date, value: 2024-01-01 }\n    references: [R1]\n",
            "contains duplicate property/value entry",
        ),
        (
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    qualifiers:\n      - property: P2\n        value: { type: string, value: wrong-type }\n    references: [R1]\n",
            "property P2 requires date values",
        ),
    ];

    for (manifest_source, expected_error) in cases {
        let root = copied_fixture();
        let manifest = write_manifest(root.path(), manifest_source);
        let entity_path = root.path().join("entities/Q1.yaml");
        let before = fs::read(&entity_path).unwrap();
        let output = knowledge_base_command()
            .args(["entity", "statement", "apply"])
            .arg(&manifest)
            .arg("--dry-run")
            .env("KNOWLEDGE_BASE_PATH", root.path())
            .output()
            .expect("run invalid statement manifest");

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected_error));
        assert_eq!(fs::read(entity_path).unwrap(), before);
    }
}

#[test]
fn statement_apply_commits_multiple_entities_after_complete_validation() {
    let root = copied_fixture();
    fs::write(
        root.path().join("properties/P4.yaml"),
        "id: P4\nlabels:\n  en:\n    text: external identifier\n    references: [R1]\nsubject_types: [T2]\nvalue_type: string\nusage: statement\ncardinality: one\n",
    )
    .unwrap();
    fs::write(
        root.path().join("id_allocation.yaml"),
        "version: 1\nnext:\n  entity: 3\n  property: 5\n  reference: 2\n  entity_type: 3\n",
    )
    .unwrap();
    let manifest = write_manifest(
        root.path(),
        "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 42 }\n    references: [R1]\n  - entity: Q2\n    property: P4\n    value: { type: string, value: Q987654 }\n    references: [R1]\n",
    );

    let output = knowledge_base_command()
        .args(["entity", "statement", "apply"])
        .arg(&manifest)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("apply multi-entity statement manifest");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("entity: Q1\n  property: P1\n  statement: S4\n  status: added"));
    assert!(stdout.contains("entity: Q2\n  property: P4\n  statement: S1\n  status: added"));
    assert!(stdout.starts_with("outcome: applied\n"));
    assert!(fs::read_to_string(root.path().join("entities/Q1.yaml")).unwrap().contains("value: 42"));
    assert!(fs::read_to_string(root.path().join("entities/Q2.yaml")).unwrap().contains("value: Q987654"));

    let validation = knowledge_base_command()
        .arg("validate")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("validate applied repository");
    assert!(validation.status.success(), "{}", String::from_utf8_lossy(&validation.stderr));
}

#[test]
fn reference_register_previews_registers_and_reuses_exact_urls() {
    let root = copied_fixture();
    let reference_path = root.path().join("references/R2.yaml");
    let allocation_path = root.path().join("id_allocation.yaml");
    let before = fs::read(&allocation_path).unwrap();
    let arguments = [
        "reference",
        "register",
        "--url",
        "https://example.org/new-source",
        "--title",
        "New source",
        "--publisher",
        "Example Publisher",
        "--publication-date",
        "2026-08",
        "--source-language",
        "en",
        "--archive-url",
        "https://archive.example.org/new-source",
    ];

    let preview = knowledge_base_command()
        .args(arguments)
        .arg("--dry-run")
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("preview reference registration");
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    assert_eq!(String::from_utf8(preview.stdout).unwrap(), "status: previewed\nreference: R2\n");
    assert!(!reference_path.exists());
    assert_eq!(fs::read(&allocation_path).unwrap(), before);

    let registered = knowledge_base_command()
        .args(arguments)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("register reference");
    assert!(registered.status.success(), "{}", String::from_utf8_lossy(&registered.stderr));
    assert_eq!(String::from_utf8(registered.stdout).unwrap(), "status: registered\nreference: R2\n");
    let stored = fs::read_to_string(&reference_path).unwrap();
    assert!(stored.contains("publisher: Example Publisher\n"));
    assert!(stored.contains("publication_date: 2026-08\n"));
    assert!(stored.contains("source_language: en\n"));
    let timestamp = stored.lines().find_map(|line| line.strip_prefix("retrieved_at: ")).expect("retrieval timestamp");
    assert!(DateTime::parse_from_rfc3339(timestamp).is_ok());

    let reused = knowledge_base_command()
        .args(arguments)
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("reuse reference");
    assert!(reused.status.success(), "{}", String::from_utf8_lossy(&reused.stderr));
    assert_eq!(String::from_utf8(reused.stdout).unwrap(), "status: existing\nreference: R2\n");
    assert_eq!(fs::read_to_string(allocation_path).unwrap().matches("reference: 3").count(), 1);
}

#[test]
fn reference_register_rejects_invalid_metadata_and_allocation_exhaustion_without_writes() {
    let root = copied_fixture();
    let invalid = knowledge_base_command()
        .args(["reference", "register", "--url", "https://example.org/source", "--title", " "])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("register invalid reference");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("title must not be empty"));
    assert!(!root.path().join("references/R2.yaml").exists());

    let allocation_path = root.path().join("id_allocation.yaml");
    fs::write(
        &allocation_path,
        format!("version: 1\nnext:\n  entity: 3\n  property: 4\n  reference: {}\n  entity_type: 3\n", u64::MAX),
    )
    .unwrap();
    let before = fs::read(&allocation_path).unwrap();
    let exhausted = knowledge_base_command()
        .args(["reference", "register", "--url", "https://example.org/source", "--title", "Source"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("register exhausted reference");
    assert!(!exhausted.status.success());
    assert!(String::from_utf8_lossy(&exhausted.stderr).contains("cannot allocate another reference identifier"));
    assert_eq!(fs::read(&allocation_path).unwrap(), before);
}
