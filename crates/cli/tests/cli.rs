use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

fn knowledge_base_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_knowledge-base"))
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
fn read_preserves_a_missing_trailing_newline_without_validating_the_repository() {
    let root = tempfile::tempdir().expect("temporary knowledge base");
    fs::create_dir(root.path().join("entities")).expect("create entity directory");
    fs::write(root.path().join("entities/Q1.yaml"), b"id: Q1").expect("write entity");

    let output = knowledge_base_command()
        .args(["entity", "read", "Q1"])
        .env("KNOWLEDGE_BASE_PATH", root.path())
        .output()
        .expect("run entity read");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"id: Q1");
}

#[test]
fn validate_uses_the_environment_configured_root() {
    let root = fixture();
    let output = knowledge_base_command().arg("validate").env("KNOWLEDGE_BASE_PATH", &root).output().expect("run validation");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("valid knowledge base: {}\n", root.display()));
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
