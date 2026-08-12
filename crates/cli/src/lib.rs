//! Reusable command-line distribution support for a file-based knowledge base.
//!
//! The published binary is the base-only distribution. Downstream applications
//! can statically register their core and CLI extensions with [`DistributionBuilder`].

mod commands;

use clap::{ArgMatches, Command, CommandFactory, FromArgMatches};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_extension_framework::bindings::ResolvedBindings;
use knowledge_base_extension_framework::contracts::{ContractVersion, ExtensionId, KnowledgeBaseExtension};
use knowledge_base_extension_framework::manifest::{ExtensionManifest, ManifestActivation, ManifestError};
use knowledge_base_extension_framework::registry::ExtensionRegistry;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

const KNOWLEDGE_BASE_PATH: &str = "KNOWLEDGE_BASE_PATH";

/// A CLI error rendered by the distribution using the base CLI error convention.
#[derive(Debug)]
pub struct CliError {
    message: String,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CliError {}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self::new(format!("cannot write command output: {error}"))
    }
}

/// Context provided to an extension command after optional repository activation.
pub struct ExtensionCommandContext<'a> {
    repository: Option<&'a KnowledgeBaseRepository>,
    bindings: Option<&'a ResolvedBindings>,
}

impl<'a> ExtensionCommandContext<'a> {
    fn new(repository: Option<&'a KnowledgeBaseRepository>, bindings: Option<&'a ResolvedBindings>) -> Self {
        Self { repository, bindings }
    }

    /// Returns the activated repository for repository-dependent commands.
    pub fn repository(&self) -> Option<&'a KnowledgeBaseRepository> {
        self.repository
    }

    /// Returns semantic bindings for the activated extension set.
    pub fn bindings(&self) -> Option<&'a ResolvedBindings> {
        self.bindings
    }

    /// Writes command data to standard output.
    pub fn write_stdout(&self, content: &str) -> Result<(), CliError> {
        io::stdout().lock().write_all(content.as_bytes()).map_err(CliError::from)
    }

    /// Writes a diagnostic to standard error.
    pub fn write_stderr(&self, content: &str) -> Result<(), CliError> {
        io::stderr().lock().write_all(content.as_bytes()).map_err(CliError::from)
    }
}

/// CLI behavior for one statically compiled core extension.
pub trait KnowledgeBaseCliExtension: Send + Sync {
    /// The core extension identifier handled by this CLI extension.
    fn id(&self) -> &ExtensionId;
    /// The core extension contract supported by this CLI handler.
    fn contract(&self) -> ContractVersion;
    /// The static command inserted below `knowledge-base extension`.
    fn command(&self) -> Command;
    /// Whether the selected command requires a configured and activated repository.
    fn requires_repository(&self, matches: &ArgMatches) -> bool;
    /// Executes a parsed extension command.
    fn execute(&self, matches: &ArgMatches, context: ExtensionCommandContext<'_>) -> Result<ExitCode, CliError>;
}

/// Errors found while assembling one statically composed distribution.
#[derive(Debug)]
pub enum DistributionBuildError {
    Framework(knowledge_base_extension_framework::error::FrameworkError),
    DuplicateCliExtension(ExtensionId),
    MissingCoreExtension(ExtensionId),
    InconsistentCliExtension {
        extension: ExtensionId,
        core_contract: ContractVersion,
        cli_contract: ContractVersion,
    },
    InvalidCliCommand {
        extension: ExtensionId,
        command: String,
    },
}

impl fmt::Display for DistributionBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framework(error) => error.fmt(formatter),
            Self::DuplicateCliExtension(id) => write!(formatter, "CLI extension {id} is registered more than once"),
            Self::MissingCoreExtension(id) => write!(formatter, "CLI extension {id} has no matching core extension"),
            Self::InconsistentCliExtension {
                extension,
                core_contract,
                cli_contract,
            } => write!(
                formatter,
                "CLI extension {extension} supports contract {cli_contract}, but its core extension supports {core_contract}"
            ),
            Self::InvalidCliCommand { extension, command } => write!(formatter, "CLI extension {extension} must provide command {extension}, not {command}"),
        }
    }
}

impl Error for DistributionBuildError {}

/// Collects the static extensions that form one executable distribution.
#[derive(Default)]
pub struct DistributionBuilder {
    core_extensions: Vec<Arc<dyn KnowledgeBaseExtension>>,
    cli_extensions: Vec<Arc<dyn KnowledgeBaseCliExtension>>,
}

impl DistributionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_core_extension(mut self, extension: Arc<dyn KnowledgeBaseExtension>) -> Self {
        self.core_extensions.push(extension);
        self
    }

    pub fn with_cli_extension(mut self, extension: Arc<dyn KnowledgeBaseCliExtension>) -> Self {
        self.cli_extensions.push(extension);
        self
    }

    pub fn build(self) -> Result<Distribution, DistributionBuildError> {
        let registry = ExtensionRegistry::new(self.core_extensions).map_err(DistributionBuildError::Framework)?;
        let mut cli_extensions = BTreeMap::new();
        for extension in self.cli_extensions {
            let id = extension.id().clone();
            let core = registry.metadata(&id).ok_or_else(|| DistributionBuildError::MissingCoreExtension(id.clone()))?;
            if core.contract != extension.contract() {
                return Err(DistributionBuildError::InconsistentCliExtension {
                    extension: id,
                    core_contract: core.contract,
                    cli_contract: extension.contract(),
                });
            }
            let command = extension.command();
            if command.get_name() != extension.id().as_str() {
                return Err(DistributionBuildError::InvalidCliCommand {
                    extension: extension.id().clone(),
                    command: command.get_name().to_owned(),
                });
            }
            if cli_extensions.insert(extension.id().clone(), extension).is_some() {
                return Err(DistributionBuildError::DuplicateCliExtension(id));
            }
        }
        Ok(Distribution { registry, cli_extensions })
    }
}

/// A validated static CLI distribution.
pub struct Distribution {
    registry: ExtensionRegistry,
    cli_extensions: BTreeMap<ExtensionId, Arc<dyn KnowledgeBaseCliExtension>>,
}

impl Distribution {
    /// Parses and executes a command using process arguments.
    pub fn run(&self) -> ExitCode {
        self.run_from(env::args_os())
    }

    /// Parses and executes a command from supplied arguments.
    pub fn run_from<I, T>(&self, arguments: I) -> ExitCode
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = match self.command().try_get_matches_from(arguments) {
            Ok(matches) => matches,
            Err(error) => error.exit(),
        };
        match self.execute_matches(&matches) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    }

    /// Returns the fully composed Clap command for help and embedding.
    pub fn command(&self) -> Command {
        let extension = self.cli_extensions.values().fold(
            Command::new("extension")
                .about("Inspect and run knowledge-base extensions")
                .subcommand(Command::new("list").about("List declared and compiled extensions"))
                .subcommand(Command::new("check").about("Check configured extensions")),
            |command, extension| command.subcommand(extension.command()),
        );
        commands::Cli::command().subcommand(extension)
    }

    fn execute_matches(&self, matches: &ArgMatches) -> Result<ExitCode, CliError> {
        if let Some(("extension", extension_matches)) = matches.subcommand() {
            return self.execute_extension(extension_matches);
        }
        let cli = commands::Cli::from_arg_matches(matches).map_err(|error| CliError::new(error.to_string()))?;
        if cli.command.requires_knowledge_base() {
            let context = self.repository_context(&knowledge_base_path().map_err(CliError::new)?)?;
            commands::execute(cli.command, Some(&commands::RepositoryContext { repository: context.repository() })).map_err(|error| CliError::new(error.to_string()))
        } else {
            commands::execute(cli.command, None).map_err(|error| CliError::new(error.to_string()))
        }
    }

    fn execute_extension(&self, matches: &ArgMatches) -> Result<ExitCode, CliError> {
        match matches.subcommand() {
            Some(("list", _)) => self.list_extensions(&knowledge_base_path().map_err(CliError::new)?),
            Some(("check", _)) => self.check_extensions(&knowledge_base_path().map_err(CliError::new)?),
            Some((name, command_matches)) => {
                let id: ExtensionId = name.parse().expect("registered extension command names are canonical IDs");
                let extension = self.cli_extensions.get(&id).expect("registered extension command has a handler");
                let context = if extension.requires_repository(command_matches) {
                    Some(self.repository_context(&knowledge_base_path().map_err(CliError::new)?)?)
                } else {
                    None
                };
                extension.execute(
                    command_matches,
                    ExtensionCommandContext::new(context.as_ref().map(RepositoryContext::repository), context.as_ref().map(RepositoryContext::bindings)),
                )
            }
            None => Err(CliError::new("an extension subcommand is required")),
        }
    }

    fn repository_context(&self, root: &Path) -> Result<RepositoryContext, CliError> {
        let activation = ExtensionManifest::load_and_activate(root, &self.registry).map_err(manifest_error)?;
        let validators = activation.active().validators(activation.bindings()).map_err(|error| CliError::new(error.to_string()))?;
        Ok(RepositoryContext {
            repository: KnowledgeBaseRepository::with_validators(root.to_path_buf(), validators),
            activation,
        })
    }

    fn list_extensions(&self, root: &Path) -> Result<ExitCode, CliError> {
        let manifest = ExtensionManifest::load(root).map_err(manifest_error)?;
        let activation = manifest.activate(root, &self.registry);
        let report = ExtensionList::new(&manifest, &self.registry, activation.is_ok());
        let output = serde_yaml::to_string(&report).map_err(|error| CliError::new(format!("cannot serialize command output: {error}")))?;
        write_stdout(&output)?;
        match activation {
            Ok(_) => Ok(ExitCode::SUCCESS),
            Err(error) => {
                eprintln!("{}", manifest_error(error));
                Ok(ExitCode::FAILURE)
            }
        }
    }

    fn check_extensions(&self, root: &Path) -> Result<ExitCode, CliError> {
        self.repository_context(root)?;
        let output = serde_yaml::to_string(&ExtensionCheck { version: 1, status: "valid" }).map_err(|error| CliError::new(format!("cannot serialize command output: {error}")))?;
        write_stdout(&output)?;
        Ok(ExitCode::SUCCESS)
    }
}

struct RepositoryContext {
    repository: KnowledgeBaseRepository,
    activation: ManifestActivation,
}

impl RepositoryContext {
    fn repository(&self) -> &KnowledgeBaseRepository {
        &self.repository
    }
    fn bindings(&self) -> &ResolvedBindings {
        self.activation.bindings()
    }
}

#[derive(serde::Serialize)]
struct ExtensionList {
    version: u32,
    extensions: BTreeMap<ExtensionId, ExtensionListEntry>,
}

#[derive(serde::Serialize)]
struct ExtensionListEntry {
    declared_contract: Option<ContractVersion>,
    available_contract: Option<ContractVersion>,
    active: bool,
    incompatible: bool,
}

impl ExtensionList {
    fn new(manifest: &ExtensionManifest, registry: &ExtensionRegistry, activation_succeeded: bool) -> Self {
        let mut ids = manifest.extensions.keys().cloned().collect::<std::collections::BTreeSet<_>>();
        ids.extend(registry.extensions().map(|extension| extension.metadata().id.clone()));
        let extensions = ids
            .into_iter()
            .map(|id| {
                let declared = manifest.extensions.get(&id).map(|extension| extension.contract);
                let available = registry.metadata(&id).map(|extension| extension.contract);
                let compatible = declared.zip(available).is_some_and(|(declared, available)| declared == available);
                (
                    id,
                    ExtensionListEntry {
                        declared_contract: declared,
                        available_contract: available,
                        active: activation_succeeded && compatible,
                        incompatible: declared.is_some() && (!activation_succeeded || !compatible),
                    },
                )
            })
            .collect();
        Self { version: 1, extensions }
    }
}

#[derive(serde::Serialize)]
struct ExtensionCheck {
    version: u32,
    status: &'static str,
}

fn manifest_error(error: ManifestError) -> CliError {
    CliError::new(error.to_string())
}

fn write_stdout(content: &str) -> Result<(), CliError> {
    io::stdout().lock().write_all(content.as_bytes()).map_err(CliError::from)
}

fn knowledge_base_path() -> Result<PathBuf, &'static str> {
    match env::var_os(KNOWLEDGE_BASE_PATH) {
        Some(value) if !value.is_empty() => Ok(value.into()),
        _ => Err("KNOWLEDGE_BASE_PATH must be set to the knowledge-base root directory"),
    }
}

/// Runs the base-only published distribution.
pub fn run() -> ExitCode {
    DistributionBuilder::new().build().expect("base distribution is valid").run()
}

/// Runs the base-only published distribution with supplied arguments.
pub fn run_from<I, T>(arguments: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    DistributionBuilder::new().build().expect("base distribution is valid").run_from(arguments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use knowledge_base_extension_framework::bindings::ResolvedBindings;
    use knowledge_base_extension_framework::contracts::{ExtensionMetadata, OntologyRequirements};
    use knowledge_base_extension_framework::error::FrameworkError;
    use knowledge_base_validation::{Diagnostic, KnowledgeBaseValidator, ValidationContext, ValidationLayer};
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct CoreExtension(ExtensionMetadata);

    impl KnowledgeBaseExtension for CoreExtension {
        fn metadata(&self) -> &ExtensionMetadata {
            &self.0
        }

        fn validators(&self, _: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError> {
            Ok(Vec::new())
        }
    }

    struct TestCliExtension {
        id: ExtensionId,
        contract: ContractVersion,
        called: Arc<AtomicBool>,
    }

    impl KnowledgeBaseCliExtension for TestCliExtension {
        fn id(&self) -> &ExtensionId {
            &self.id
        }
        fn contract(&self) -> ContractVersion {
            self.contract
        }
        fn command(&self) -> Command {
            Command::new("demo")
        }
        fn requires_repository(&self, _: &ArgMatches) -> bool {
            false
        }
        fn execute(&self, _: &ArgMatches, context: ExtensionCommandContext<'_>) -> Result<ExitCode, CliError> {
            assert!(context.repository().is_none());
            assert!(context.bindings().is_none());
            self.called.store(true, Ordering::SeqCst);
            Ok(ExitCode::SUCCESS)
        }
    }

    fn id(value: &str) -> ExtensionId {
        value.parse().unwrap()
    }

    fn core(name: &str) -> Arc<dyn KnowledgeBaseExtension> {
        Arc::new(CoreExtension(ExtensionMetadata {
            id: id(name),
            contract: ContractVersion::new(1),
            dependencies: Vec::new(),
            bindings: Vec::new(),
            ontology_requirements: OntologyRequirements::default(),
        }))
    }

    struct RejectingExtension(ExtensionMetadata);

    impl KnowledgeBaseExtension for RejectingExtension {
        fn metadata(&self) -> &ExtensionMetadata {
            &self.0
        }

        fn validators(&self, _: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError> {
            Ok(vec![Arc::new(|_: &ValidationContext<'_>| {
                vec![Diagnostic {
                    layer: ValidationLayer::Domain,
                    path: "entities/Q1.yaml".into(),
                    line: None,
                    identifier: Some("Q1".to_owned()),
                    message: "test extension rejects staged repository".to_owned(),
                }]
            })])
        }
    }

    fn copy_fixture(destination: &Path) {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal");
        for directory in ["entities", "entity_types", "properties", "references", "entity_context"] {
            fs::create_dir(destination.join(directory)).unwrap();
            for entry in fs::read_dir(source.join(directory)).unwrap() {
                let entry = entry.unwrap();
                fs::copy(entry.path(), destination.join(directory).join(entry.file_name())).unwrap();
            }
        }
        fs::copy(source.join("id_allocation.yaml"), destination.join("id_allocation.yaml")).unwrap();
    }

    #[test]
    fn builder_routes_repository_independent_extension_commands() {
        let called = Arc::new(AtomicBool::new(false));
        let distribution = DistributionBuilder::new()
            .with_core_extension(core("demo"))
            .with_cli_extension(Arc::new(TestCliExtension {
                id: id("demo"),
                contract: ContractVersion::new(1),
                called: called.clone(),
            }))
            .build()
            .unwrap();
        assert_eq!(distribution.run_from(["knowledge-base", "extension", "demo"]), ExitCode::SUCCESS);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn builder_rejects_cli_handlers_without_a_matching_core_extension() {
        let result = DistributionBuilder::new()
            .with_cli_extension(Arc::new(TestCliExtension {
                id: id("demo"),
                contract: ContractVersion::new(1),
                called: Arc::new(AtomicBool::new(false)),
            }))
            .build();
        let Err(error) = result else { panic!("builder unexpectedly succeeded") };
        assert!(matches!(error, DistributionBuildError::MissingCoreExtension(extension) if extension == id("demo")));
    }

    #[test]
    fn activated_extension_validators_reject_staged_statement_mutations_atomically() {
        let root = tempfile::tempdir().unwrap();
        copy_fixture(root.path());
        fs::write(root.path().join("extensions.yaml"), "version: 1\nextensions:\n  rejector:\n    contract: 1\n").unwrap();
        fs::write(
            root.path().join("statements.yaml"),
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 999 }\n    references: [R1]\n",
        )
        .unwrap();
        let extension = Arc::new(RejectingExtension(ExtensionMetadata {
            id: id("rejector"),
            contract: ContractVersion::new(1),
            dependencies: Vec::new(),
            bindings: Vec::new(),
            ontology_requirements: OntologyRequirements::default(),
        }));
        let distribution = DistributionBuilder::new().with_core_extension(extension).build().unwrap();
        let context = distribution.repository_context(root.path()).unwrap();
        let entity_before = fs::read(root.path().join("entities/Q1.yaml")).unwrap();
        let allocation_before = fs::read(root.path().join("id_allocation.yaml")).unwrap();
        let batch = knowledge_base_crud::write::StatementBatch::read(root.path().join("statements.yaml")).unwrap();
        let error = context
            .repository()
            .write()
            .statements()
            .apply(&batch, knowledge_base_crud::write::WriteMode::Commit)
            .unwrap_err();
        assert!(error.to_string().contains("test extension rejects staged repository"));
        assert_eq!(fs::read(root.path().join("entities/Q1.yaml")).unwrap(), entity_before);
        assert_eq!(fs::read(root.path().join("id_allocation.yaml")).unwrap(), allocation_before);
    }
}
