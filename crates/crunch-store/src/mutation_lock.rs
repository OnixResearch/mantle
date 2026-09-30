use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

use fs2::FileExt;

use crate::Error;

const LOCK_FILE_NAME: &str = "store-mutation.lock";

#[derive(Debug)]
pub struct StoreMutationGuard {
    file: File,
    path: PathBuf,
}

impl StoreMutationGuard {
    /// Confirm this guard owns the exact selected state directory's lock.
    pub(crate) fn protects_state_dir(&self, state_dir: &Path) -> bool {
        self.path == state_dir.join(LOCK_FILE_NAME)
    }

    pub fn acquire_wait(state_dir: &Path) -> Result<Self, Error> {
        let guard = Self::open(state_dir)?;
        guard
            .file
            .lock_exclusive()
            .map_err(|err| Error::MutationLock(format!("locking {}: {err}", guard.path.display())))?;
        Ok(guard)
    }

    pub fn try_acquire(state_dir: &Path) -> Result<Self, Error> {
        let guard = Self::open(state_dir)?;
        guard.file.try_lock_exclusive().map_err(|err| {
            Error::MutationLock(format!(
                "another local build, substitution, or store mutation is active ({}): {err}",
                guard.path.display()
            ))
        })?;
        Ok(guard)
    }

    fn open(state_dir: &Path) -> Result<Self, Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        std::fs::create_dir_all(state_dir)
            .map_err(|err| Error::MutationLock(format!("creating {}: {err}", state_dir.display())))?;
        let path = state_dir.join(LOCK_FILE_NAME);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|err| Error::MutationLock(format!("opening {}: {err}", path.display())))?;
        Ok(Self { file, path })
    }
}

impl Drop for StoreMutationGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_acquire_rejects_second_mutator() {
        let state_dir = tempfile::tempdir().unwrap();
        let _guard = StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();

        let err = StoreMutationGuard::try_acquire(state_dir.path()).unwrap_err();

        assert!(matches!(err, Error::MutationLock(msg) if msg.contains("active")));
    }
}
