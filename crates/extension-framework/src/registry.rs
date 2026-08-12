use crate::bindings::{BindingValue, ResolvedBindings};
use crate::contracts::{BindingKind, BindingReference, ExtensionId, ExtensionMetadata, KnowledgeBaseExtension};
use crate::error::FrameworkError;
use knowledge_base_validation::KnowledgeBaseValidator;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The statically compiled extensions available in one distribution.
pub struct ExtensionRegistry {
    extensions: BTreeMap<ExtensionId, Arc<dyn KnowledgeBaseExtension>>,
}

impl ExtensionRegistry {
    /// Registers extensions after validating their metadata and declared dependencies.
    pub fn new(extensions: impl IntoIterator<Item = Arc<dyn KnowledgeBaseExtension>>) -> Result<Self, FrameworkError> {
        let mut registered = BTreeMap::new();
        for extension in extensions {
            let metadata = extension.metadata();
            validate_metadata(metadata)?;
            let id = metadata.id.clone();
            if registered.insert(id.clone(), extension).is_some() {
                return Err(FrameworkError::DuplicateExtension(id));
            }
        }

        let registry = Self { extensions: registered };
        for extension in registry.extensions.values() {
            registry.validate_dependencies(extension.metadata())?;
            registry.validate_requirements(extension.metadata())?;
        }
        Ok(registry)
    }

    /// Resolves an explicitly activated set into stable dependency-first order.
    pub fn resolve_active(&self, requested: impl IntoIterator<Item = ExtensionId>) -> Result<ActiveExtensions, FrameworkError> {
        let requested = requested.into_iter().collect::<BTreeSet<_>>();
        let mut ordered = Vec::new();
        let mut state = BTreeMap::new();
        for id in &requested {
            self.visit(id, &requested, &mut state, &mut ordered, &mut Vec::new())?;
        }
        Ok(ActiveExtensions { ordered })
    }

    /// Returns compiled extension metadata by canonical identifier.
    pub fn metadata(&self, id: &ExtensionId) -> Option<&ExtensionMetadata> {
        self.extensions.get(id).map(|extension| extension.metadata())
    }

    /// Returns every compiled extension in canonical identifier order.
    pub fn extensions(&self) -> impl Iterator<Item = &dyn KnowledgeBaseExtension> {
        self.extensions.values().map(AsRef::as_ref)
    }

    fn validate_dependencies(&self, metadata: &ExtensionMetadata) -> Result<(), FrameworkError> {
        for dependency in &metadata.dependencies {
            let available = self.extensions.get(&dependency.id).ok_or_else(|| FrameworkError::MissingDependency {
                extension: metadata.id.clone(),
                dependency: dependency.id.clone(),
            })?;
            if available.metadata().contract != dependency.contract {
                return Err(FrameworkError::UnsupportedContract {
                    extension: metadata.id.clone(),
                    required: dependency.contract,
                    available: available.metadata().contract,
                });
            }
        }
        Ok(())
    }

    fn validate_requirements(&self, metadata: &ExtensionMetadata) -> Result<(), FrameworkError> {
        for requirement in &metadata.ontology_requirements.entity_types {
            self.validate_requirement_binding(metadata, &requirement.binding, BindingKind::EntityType)?;
        }
        for requirement in &metadata.ontology_requirements.properties {
            self.validate_requirement_binding(metadata, &requirement.binding, BindingKind::Property)?;
            for binding in &requirement.subject_types {
                self.validate_requirement_binding(metadata, binding, BindingKind::EntityType)?;
            }
            if let Some(target_types) = &requirement.target_types {
                for binding in target_types {
                    self.validate_requirement_binding(metadata, binding, BindingKind::EntityType)?;
                }
            }
            for binding in &requirement.allowed_qualifiers {
                self.validate_requirement_binding(metadata, binding, BindingKind::Property)?;
            }
        }
        Ok(())
    }

    fn validate_requirement_binding(&self, metadata: &ExtensionMetadata, reference: &BindingReference, expected: BindingKind) -> Result<(), FrameworkError> {
        if reference.extension_id() != &metadata.id && !metadata.dependencies.iter().any(|dependency| dependency.id == *reference.extension_id()) {
            return Err(FrameworkError::InaccessibleBinding {
                extension: metadata.id.clone(),
                binding: reference.clone(),
            });
        }
        let owner = self.extensions.get(reference.extension_id()).expect("dependency availability was validated").metadata();
        let declaration = owner
            .bindings
            .iter()
            .find(|binding| binding.key == *reference.key())
            .ok_or_else(|| FrameworkError::UndeclaredBinding {
                extension: owner.id.clone(),
                binding: reference.clone(),
            })?;
        if declaration.kind != expected {
            return Err(FrameworkError::InvalidRequirement {
                extension: metadata.id.clone(),
                binding: reference.clone(),
                expected,
            });
        }
        Ok(())
    }

    /// Depth-first traversal detects cycles and appends each node after its dependencies.
    fn visit(
        &self,
        id: &ExtensionId,
        requested: &BTreeSet<ExtensionId>,
        state: &mut BTreeMap<ExtensionId, Visit>,
        ordered: &mut Vec<Arc<dyn KnowledgeBaseExtension>>,
        stack: &mut Vec<ExtensionId>,
    ) -> Result<(), FrameworkError> {
        match state.get(id) {
            Some(Visit::Done) => return Ok(()),
            Some(Visit::Visiting) => {
                let start = stack.iter().position(|item| item == id).expect("visiting extension is on stack");
                let mut cycle = stack[start..].to_vec();
                cycle.push(id.clone());
                return Err(FrameworkError::DependencyCycle(cycle));
            }
            None => {}
        }
        let extension = self
            .extensions
            .get(id)
            .ok_or_else(|| FrameworkError::MissingDependency {
                extension: id.clone(),
                dependency: id.clone(),
            })?
            .clone();
        state.insert(id.clone(), Visit::Visiting);
        stack.push(id.clone());
        for dependency in &extension.metadata().dependencies {
            if !requested.contains(&dependency.id) {
                return Err(FrameworkError::InactiveDependency {
                    extension: id.clone(),
                    dependency: dependency.id.clone(),
                });
            }
            self.visit(&dependency.id, requested, state, ordered, stack)?;
        }
        stack.pop();
        state.insert(id.clone(), Visit::Done);
        ordered.push(extension);
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Visit {
    Visiting,
    Done,
}

/// The dependency-first extensions active for one repository configuration.
pub struct ActiveExtensions {
    ordered: Vec<Arc<dyn KnowledgeBaseExtension>>,
}

impl ActiveExtensions {
    pub fn extensions(&self) -> &[Arc<dyn KnowledgeBaseExtension>] {
        &self.ordered
    }

    /// Constructs every active extension validator in dependency-first order.
    pub fn validators(&self, bindings: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError> {
        let mut validators = Vec::new();
        for extension in &self.ordered {
            validators.extend(extension.validators(bindings)?);
        }
        Ok(validators)
    }

    /// Verifies that every active declared binding has a value of its declared kind.
    pub fn resolve_bindings(&self, declared: BTreeMap<BindingReference, BindingValue>) -> Result<ResolvedBindings, FrameworkError> {
        let metadata = self
            .ordered
            .iter()
            .map(|extension| (extension.metadata().id.clone(), extension.metadata()))
            .collect::<BTreeMap<_, _>>();
        let mut resolved = ResolvedBindings::default();
        for (reference, value) in declared {
            let owner = metadata.get(reference.extension_id()).ok_or_else(|| FrameworkError::MissingBinding(reference.clone()))?;
            let declaration = owner
                .bindings
                .iter()
                .find(|item| item.key == *reference.key())
                .ok_or_else(|| FrameworkError::UndeclaredBinding {
                    extension: owner.id.clone(),
                    binding: reference.clone(),
                })?;
            if declaration.kind != value.kind() {
                return Err(FrameworkError::BindingKindMismatch {
                    binding: reference,
                    expected: declaration.kind,
                    actual: value.kind(),
                });
            }
            resolved.values.insert(reference, value);
        }
        for extension in &self.ordered {
            for declaration in &extension.metadata().bindings {
                let reference = BindingReference::new(extension.metadata().id.clone(), declaration.key.clone());
                if !resolved.values.contains_key(&reference) {
                    return Err(FrameworkError::MissingBinding(reference));
                }
            }
        }
        Ok(resolved)
    }
}

fn validate_metadata(metadata: &ExtensionMetadata) -> Result<(), FrameworkError> {
    let mut dependencies = BTreeSet::new();
    for dependency in &metadata.dependencies {
        if !dependencies.insert(dependency.id.clone()) {
            return Err(FrameworkError::DuplicateDependency {
                extension: metadata.id.clone(),
                dependency: dependency.id.clone(),
            });
        }
    }
    let mut bindings = BTreeMap::new();
    for binding in &metadata.bindings {
        if bindings.insert(binding.key.clone(), binding.kind).is_some() {
            return Err(FrameworkError::DuplicateBinding {
                extension: metadata.id.clone(),
                key: binding.key.clone(),
            });
        }
    }
    Ok(())
}
