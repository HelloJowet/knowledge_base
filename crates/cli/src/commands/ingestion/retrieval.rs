use super::super::{CommandError, write_content};
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_crud::write::WriteMode;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Fetch a web page and save it as a retrieval bundle.
    Fetch {
        /// Web page URL to fetch.
        url: String,
        /// Directory in which uniquely named retrieval bundles are created.
        #[arg(long, value_name = "PATH", default_value = "temp/retrievals")]
        output_root: PathBuf,
    },
    /// Register or reuse the reference described by a retrieval bundle.
    Register {
        /// Directory containing page.html and retrieval.yaml.
        bundle_directory: PathBuf,
        /// Validate and report the registration without writing repository files.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn requires_knowledge_base(command: &Command) -> bool {
    matches!(command, Command::Register { .. })
}

pub fn execute(command: Command, repository: Option<&KnowledgeBaseRepository>) -> Result<ExitCode, CommandError> {
    match command {
        Command::Fetch { url, output_root } => {
            let path = knowledge_base_ingestion_retrieval::fetch_to_bundle(&url, &output_root)?;
            write_content(&format!("{}\n", path.display()))
        }
        Command::Register { bundle_directory, dry_run } => {
            let mode = if dry_run { WriteMode::Preview } else { WriteMode::Commit };
            let outcome = knowledge_base_ingestion_retrieval::register_bundle(&bundle_directory, repository.expect("retrieval registration requires a knowledge base"), mode)?;
            let output = serde_yaml::to_string(&outcome).map_err(CommandError::Serialization)?;
            write_content(&output)
        }
    }
}
