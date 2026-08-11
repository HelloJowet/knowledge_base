use crate::Error;
use fs2::FileExt;
use knowledge_base_validation::validate_repository;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempPath};

const LOCK_FILE: &str = ".knowledge-base.lock";

#[derive(Debug)]
pub(crate) struct FileEdit {
    pub(crate) path: PathBuf,
    pub(crate) original: Option<Vec<u8>>,
    pub(crate) replacement: Vec<u8>,
}

pub(crate) struct MutationLock {
    file: File,
}

impl MutationLock {
    pub(crate) fn acquire(root: &Path) -> Result<Self, Error> {
        let path = root.join(LOCK_FILE);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| Error::Write { path: path.clone(), source })?;
        file.lock_exclusive().map_err(|source| Error::Write { path, source })?;
        Ok(Self { file })
    }
}

impl Drop for MutationLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

pub(crate) fn validate_staged(root: &Path, edits: &[FileEdit]) -> Result<(), Error> {
    let staging = tempfile::tempdir().map_err(|source| Error::Write {
        path: std::env::temp_dir(),
        source,
    })?;
    copy_repository(root, staging.path())?;
    for edit in edits {
        let relative = edit
            .path
            .strip_prefix(root)
            .map_err(|_| Error::InvalidRequest(format!("resource path {} is outside the knowledge-base root", edit.path.display())))?;
        let path = staging.path().join(relative);
        fs::write(&path, &edit.replacement).map_err(|source| Error::Write { path, source })?;
    }
    let diagnostics = validate_repository(staging.path());
    if diagnostics.is_empty() { Ok(()) } else { Err(Error::Validation(diagnostics)) }
}

pub(crate) fn commit(edits: &[FileEdit]) -> Result<(), Error> {
    commit_with(edits, |temporary, path| temporary.persist(path).map(|_| ()).map_err(|error| error.error))
}

fn copy_repository(root: &Path, destination: &Path) -> Result<(), Error> {
    for directory in ["entities", "entity_types", "properties", "references"] {
        copy_directory(&root.join(directory), &destination.join(directory))?;
    }
    let context = root.join("entity_context");
    if context.exists() {
        copy_directory(&context, &destination.join("entity_context"))?;
    }
    copy_file(&root.join("id_allocation.yaml"), &destination.join("id_allocation.yaml"))
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), Error> {
    fs::create_dir(destination).map_err(|source| Error::Write {
        path: destination.to_path_buf(),
        source,
    })?;
    let entries = fs::read_dir(source).map_err(|source_error| Error::Read {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source_error| Error::Read {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        copy_file(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), Error> {
    fs::copy(source, destination).map(|_| ()).map_err(|source_error| Error::Read {
        path: source.to_path_buf(),
        source: source_error,
    })
}

fn stage_edit(edit: &FileEdit) -> Result<TempPath, Error> {
    let parent = edit.path.parent().expect("resource paths have a parent directory");
    let mut temporary = NamedTempFile::new_in(parent).map_err(|source| Error::Write {
        path: parent.to_path_buf(),
        source,
    })?;
    temporary
        .write_all(&edit.replacement)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|source| Error::Write {
            path: temporary.path().to_path_buf(),
            source,
        })?;
    Ok(temporary.into_temp_path())
}

fn commit_with(edits: &[FileEdit], mut persist: impl FnMut(TempPath, &Path) -> io::Result<()>) -> Result<(), Error> {
    for edit in edits {
        let current = match fs::read(&edit.path) {
            Ok(current) => Some(current),
            Err(source) if source.kind() == io::ErrorKind::NotFound => None,
            Err(source) => return Err(Error::Read { path: edit.path.clone(), source }),
        };
        if current != edit.original {
            return Err(Error::ConcurrentChange(edit.path.clone()));
        }
    }

    let staged = edits.iter().map(stage_edit).collect::<Result<Vec<_>, _>>()?;
    for (committed, (edit, temporary)) in edits.iter().zip(staged).enumerate() {
        if let Err(error) = persist(temporary, &edit.path) {
            let rollback = rollback(&edits[..committed]);
            let mut message = format!("cannot commit {}: {error}", edit.path.display());
            if let Err(rollback_error) = rollback {
                message.push_str(&format!("; rollback also failed: {rollback_error}"));
            }
            return Err(Error::Commit { message });
        }
    }
    Ok(())
}

fn rollback(edits: &[FileEdit]) -> Result<(), Error> {
    for edit in edits.iter().rev() {
        let Some(original) = &edit.original else {
            match fs::remove_file(&edit.path) {
                Ok(()) => continue,
                Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
                Err(source) => return Err(Error::Write { path: edit.path.clone(), source }),
            }
        };
        let parent = edit.path.parent().expect("resource paths have a parent directory");
        let mut temporary = NamedTempFile::new_in(parent).map_err(|source| Error::Write {
            path: parent.to_path_buf(),
            source,
        })?;
        temporary.write_all(original).and_then(|_| temporary.as_file().sync_all()).map_err(|source| Error::Write {
            path: temporary.path().to_path_buf(),
            source,
        })?;
        temporary.persist(&edit.path).map_err(|error| Error::Write {
            path: edit.path.clone(),
            source: error.error,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FileEdit, commit_with};
    use crate::Error;

    #[test]
    fn commit_rolls_back_when_a_later_replacement_fails() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("Q1.yaml");
        let second = root.path().join("Q2.yaml");
        std::fs::write(&first, "first original").unwrap();
        std::fs::write(&second, "second original").unwrap();
        let edits = vec![
            FileEdit {
                path: first.clone(),
                original: Some(b"first original".to_vec()),
                replacement: b"first replacement".to_vec(),
            },
            FileEdit {
                path: second.clone(),
                original: Some(b"second original".to_vec()),
                replacement: b"second replacement".to_vec(),
            },
        ];
        let mut calls = 0;

        let error = commit_with(&edits, |temporary, path| {
            calls += 1;
            if calls == 2 {
                Err(std::io::Error::other("injected failure"))
            } else {
                temporary.persist(path).map(|_| ()).map_err(|error| error.error)
            }
        })
        .unwrap_err();

        assert!(matches!(error, Error::Commit { .. }));
        assert_eq!(std::fs::read_to_string(first).unwrap(), "first original");
        assert_eq!(std::fs::read_to_string(second).unwrap(), "second original");
    }

    #[test]
    fn commit_detects_source_changes_before_staging() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Q1.yaml");
        std::fs::write(&path, "changed externally").unwrap();
        let edits = [FileEdit {
            path: path.clone(),
            original: Some(b"original".to_vec()),
            replacement: b"replacement".to_vec(),
        }];

        let error = commit_with(&edits, |_, _| unreachable!()).unwrap_err();

        assert!(matches!(error, Error::ConcurrentChange(changed) if changed == path));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "changed externally");
    }

    #[test]
    fn commit_detects_a_file_created_after_planning() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("R2.yaml");
        std::fs::write(&path, "created externally").unwrap();
        let edits = [FileEdit {
            path: path.clone(),
            original: None,
            replacement: b"planned reference".to_vec(),
        }];

        let error = commit_with(&edits, |_, _| unreachable!()).unwrap_err();

        assert!(matches!(error, Error::ConcurrentChange(changed) if changed == path));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "created externally");
    }

    #[test]
    fn commit_rolls_back_a_created_file_when_a_later_replacement_fails() {
        let root = tempfile::tempdir().unwrap();
        let created = root.path().join("R2.yaml");
        let allocation = root.path().join("id_allocation.yaml");
        std::fs::write(&allocation, "original allocation").unwrap();
        let edits = vec![
            FileEdit {
                path: created.clone(),
                original: None,
                replacement: b"new reference".to_vec(),
            },
            FileEdit {
                path: allocation.clone(),
                original: Some(b"original allocation".to_vec()),
                replacement: b"updated allocation".to_vec(),
            },
        ];
        let mut calls = 0;

        let error = commit_with(&edits, |temporary, path| {
            calls += 1;
            if calls == 2 {
                Err(std::io::Error::other("injected failure"))
            } else {
                temporary.persist(path).map(|_| ()).map_err(|error| error.error)
            }
        })
        .unwrap_err();

        assert!(matches!(error, Error::Commit { .. }));
        assert!(!created.exists());
        assert_eq!(std::fs::read_to_string(allocation).unwrap(), "original allocation");
    }
}
