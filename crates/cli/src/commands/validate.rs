use knowledge_base_validation::validate_repository;
use std::path::Path;
use std::process::ExitCode;

pub fn execute(root: &Path) -> ExitCode {
    let diagnostics = validate_repository(root);
    if diagnostics.is_empty() {
        println!("valid knowledge base: {}", root.display());
        ExitCode::SUCCESS
    } else {
        for diagnostic in diagnostics {
            eprintln!("{diagnostic}");
        }
        ExitCode::FAILURE
    }
}
