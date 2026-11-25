use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use fs2::FileExt;

use crate::Error;

const LOCK_FILE_NAME: &str = "store-mutation.lock";
const MAX_OWNER_METADATA_BYTES: u64 = 1024;
const MAX_OWNER_ROLE_CHARS: usize = 128;
const LOCK_POLL_INTERVAL_MS: u64 = 25;
const DEFAULT_SYNC_WAIT_TIMEOUT_MS: u64 = 300_000;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct LockOwnerMetadata {
    pid: u32,
    role: String,
    acquired_unix_secs: i64,
}

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
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        const {
            assert!(DEFAULT_SYNC_WAIT_TIMEOUT_MS >= LOCK_POLL_INTERVAL_MS);
        }
        Self::acquire_wait_bounded(state_dir, "store-mutator", DEFAULT_SYNC_WAIT_TIMEOUT_MS)
    }

    pub fn acquire_wait_bounded(state_dir: &Path, role: &str, timeout_ms: u64) -> Result<Self, Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        assert!(!role.is_empty(), "role must not be empty");
        let max_polls = timeout_ms.checked_div(LOCK_POLL_INTERVAL_MS).unwrap_or(0).saturating_add(1);
        assert!(max_polls >= 1);
        for poll_index in 0..max_polls {
            if let Some(guard) = Self::try_acquire_nonblocking(state_dir, role)? {
                return Ok(guard);
            }
            if poll_index.saturating_add(1) >= max_polls {
                break;
            }
            std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
        }
        Err(Error::MutationLock(format!(
            "timed out waiting for store mutation lock ({})",
            Self::format_busy_diagnostic(state_dir, timeout_ms)
        )))
    }

    pub fn try_acquire(state_dir: &Path) -> Result<Self, Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        Self::try_acquire_for(state_dir, "store-mutator")
    }

    pub fn try_acquire_for(state_dir: &Path, role: &str) -> Result<Self, Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        assert!(!role.is_empty(), "role must not be empty");
        match Self::try_acquire_nonblocking(state_dir, role)? {
            Some(guard) => Ok(guard),
            None => Err(Error::MutationLock(format!(
                "another local build, substitution, or store mutation is active ({}): {}",
                state_dir.join(LOCK_FILE_NAME).display(),
                Self::read_owner_diagnostic(state_dir)
            ))),
        }
    }

    pub fn try_acquire_nonblocking(state_dir: &Path, role: &str) -> Result<Option<Self>, Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        assert!(!role.is_empty(), "role must not be empty");
        let (file, path) = Self::open_lock_file(state_dir)?;
        match file.try_lock_exclusive() {
            Ok(()) => {
                let mut guard = Self { file, path };
                guard.write_owner_metadata(role)?;
                Ok(Some(guard))
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(Error::MutationLock(format!("locking {}: {err}", path.display()))),
        }
    }

    pub fn format_busy_diagnostic(state_dir: &Path, wait_ms: u64) -> String {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        let lock_path = state_dir.join(LOCK_FILE_NAME);
        let owner = Self::read_owner_diagnostic(state_dir);
        debug_assert!(!owner.is_empty());
        format!("lock_path={} waited_ms={wait_ms} {owner}", lock_path.display())
    }

    fn open_lock_file(state_dir: &Path) -> Result<(File, PathBuf), Error> {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        std::fs::create_dir_all(state_dir)
            .map_err(|err| Error::MutationLock(format!("creating {}: {err}", state_dir.display())))?;
        let path = state_dir.join(LOCK_FILE_NAME);
        ensure_regular_lock_target(&path)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|err| Error::MutationLock(format!("opening {}: {err}", path.display())))?;
        let opened_meta = file
            .metadata()
            .map_err(|err| Error::MutationLock(format!("inspecting {}: {err}", path.display())))?;
        if !opened_meta.is_file() {
            return Err(Error::MutationLock(format!("refusing non-regular lock file {}", path.display())));
        }
        Ok((file, path))
    }

    fn write_owner_metadata(&mut self, role: &str) -> Result<(), Error> {
        assert!(!role.is_empty(), "role must not be empty");
        let bounded_role: String = role.chars().take(MAX_OWNER_ROLE_CHARS).collect();
        let acquired_unix_secs = crate::roots::current_unix_seconds()?;
        let metadata = LockOwnerMetadata {
            pid: std::process::id(),
            role: bounded_role,
            acquired_unix_secs,
        };
        let encoded = serde_json::to_vec(&metadata).map_err(|err| {
            Error::MutationLock(format!("encoding owner metadata for {}: {err}", self.path.display()))
        })?;
        let encoded_len = u64::try_from(encoded.len()).map_err(|err| {
            Error::MutationLock(format!("owner metadata size overflow for {}: {err}", self.path.display()))
        })?;
        assert!(encoded_len <= MAX_OWNER_METADATA_BYTES);
        self.file
            .set_len(0)
            .map_err(|err| Error::MutationLock(format!("truncating {}: {err}", self.path.display())))?;
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(|err| Error::MutationLock(format!("seeking {}: {err}", self.path.display())))?;
        self.file
            .write_all(&encoded)
            .map_err(|err| Error::MutationLock(format!("writing owner metadata {}: {err}", self.path.display())))?;
        self.file
            .flush()
            .map_err(|err| Error::MutationLock(format!("flushing owner metadata {}: {err}", self.path.display())))?;
        Ok(())
    }

    fn read_owner_diagnostic(state_dir: &Path) -> String {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        let path = state_dir.join(LOCK_FILE_NAME);
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            return "owner metadata unavailable".to_string();
        };
        if !meta.file_type().is_file() {
            return "owner metadata unavailable".to_string();
        }
        let Ok(file) = OpenOptions::new().read(true).open(&path) else {
            return "owner metadata unavailable".to_string();
        };
        let read_limit_bytes = MAX_OWNER_METADATA_BYTES.saturating_add(1);
        let mut bytes = Vec::new();
        if file.take(read_limit_bytes).read_to_end(&mut bytes).is_err() {
            return "owner metadata unavailable".to_string();
        }
        let Ok(read_len) = u64::try_from(bytes.len()) else {
            return "owner metadata unavailable".to_string();
        };
        if bytes.is_empty() || read_len > MAX_OWNER_METADATA_BYTES {
            return "owner metadata unavailable".to_string();
        }
        let Ok(owner) = serde_json::from_slice::<LockOwnerMetadata>(&bytes) else {
            return "owner metadata unavailable".to_string();
        };
        debug_assert!(!owner.role.is_empty());
        format!("owner pid={} role={} acquired_unix_s={}", owner.pid, owner.role, owner.acquired_unix_secs)
    }
}

fn ensure_regular_lock_target(path: &Path) -> Result<(), Error> {
    assert!(!path.as_os_str().is_empty(), "lock path must not be empty");
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.file_type().is_file() {
                return Err(Error::MutationLock(format!(
                    "refusing symlink or non-regular lock path {}",
                    path.display()
                )));
            }
            debug_assert!(meta.file_type().is_file());
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Error::MutationLock(format!("inspecting {}: {err}", path.display()))),
    }
}

impl Drop for StoreMutationGuard {
    fn drop(&mut self) {
        let _ = self.file.set_len(0);
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

    #[test]
    fn try_acquire_for_reports_owner_pid_and_role_and_clears_on_drop() {
        let state_dir = tempfile::tempdir().unwrap();
        let guard = StoreMutationGuard::try_acquire_for(state_dir.path(), "build-role").unwrap();

        let err = StoreMutationGuard::try_acquire_for(state_dir.path(), "gc-role").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("active"), "{message}");
        assert!(message.contains(&format!("pid={}", std::process::id())), "{message}");
        assert!(message.contains("role=build-role"), "{message}");

        drop(guard);
        let lock_bytes = std::fs::read(state_dir.path().join(LOCK_FILE_NAME)).unwrap();
        assert!(lock_bytes.is_empty(), "owner metadata must clear on drop");

        let next = StoreMutationGuard::try_acquire_nonblocking(state_dir.path(), "gc-role")
            .unwrap()
            .expect("lock must be free after drop");
        drop(next);
    }

    #[test]
    fn acquire_wait_bounded_times_out_with_busy_diagnostic() {
        let state_dir = tempfile::tempdir().unwrap();
        let _guard = StoreMutationGuard::try_acquire_for(state_dir.path(), "watch").unwrap();

        let err = StoreMutationGuard::acquire_wait_bounded(state_dir.path(), "build", 60).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("timed out"), "{message}");
        assert!(message.contains("waited_ms=60"), "{message}");
        assert!(message.contains("role=watch"), "{message}");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_lock_path_is_rejected() {
        let state_dir = tempfile::tempdir().unwrap();
        let target = state_dir.path().join("outside.txt");
        std::fs::write(&target, b"keep").unwrap();
        std::os::unix::fs::symlink(&target, state_dir.path().join(LOCK_FILE_NAME)).unwrap();

        let err = StoreMutationGuard::try_acquire_for(state_dir.path(), "build").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("refusing symlink or non-regular lock path"), "{message}");
        assert_eq!(std::fs::read(&target).unwrap(), b"keep");
    }
}
