use knowledge_base_crud::KnowledgeBaseRepository;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository) -> ExitCode {
    let diagnostics = repository.validate();
    if diagnostics.is_empty() {
        println!("valid knowledge base: {}", repository.root().display());
        ExitCode::SUCCESS
    } else {
        for diagnostic in diagnostics {
            eprintln!("{diagnostic}");
        }
        ExitCode::FAILURE
    }
}
