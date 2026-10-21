//! Short coordinator-save fencing. The mutation guard protects admission-sized
//! transactions; this separate lock protects only revision check + durable save,
//! including saves made by runners that did not retain that mutation guard.
use std::fs::File;
use std::fs::{self};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use fs2::FileExt;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use super::RemoteCoordinatorState;
use crate::errors::RunError;

const SAVE_LOCK_FILE: &str = "remote-coordinator-save.lock";

/// Cloning a coordinator copies its observed revision, not a shared revision
/// cell. Committing one clone must never authorize another stale clone's save.
#[derive(Debug, Default)]
pub(crate) struct CoordinatorRevision(AtomicU64);

impl Clone for CoordinatorRevision {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.get()))
    }
}

// Revision is persistence metadata, not part of the coordinator's domain value.
impl PartialEq for CoordinatorRevision {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for CoordinatorRevision {}

impl<'de> Deserialize<'de> for CoordinatorRevision {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(|revision| Self(AtomicU64::new(revision)))
    }
}

impl CoordinatorRevision {
    pub(crate) fn get(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Deserialize, Default)]
struct PersistedRevision {
    #[serde(default)]
    persistence_revision: u64,
}

/// The root state skips serializing its revision field. Only this borrowed
/// envelope supplies the next revision, without copying jobs or event histories.
#[derive(Serialize)]
pub(crate) struct RevisionedState<'a> {
    #[serde(flatten)]
    state: &'a RemoteCoordinatorState,
    persistence_revision: u64,
}

pub(crate) struct CoordinatorSave<'a> {
    _guard: File,
    state: &'a RemoteCoordinatorState,
    next_revision: u64,
}

/// Check disk authority without calling the normal load path: loading may
/// migrate/reconcile and save, which would recursively acquire this save lock.
pub(crate) fn begin_save<'a>(
    state_dir: &Path,
    state: &'a RemoteCoordinatorState,
) -> Result<CoordinatorSave<'a>, RunError> {
    fs::create_dir_all(state_dir)
        .map_err(|error| RunError::Internal(format!("creating coordinator save directory: {error}")))?;
    let guard = super::open_coordinator_lock_no_follow(&state_dir.join(SAVE_LOCK_FILE)).map_err(RunError::Internal)?;
    FileExt::lock_exclusive(&guard)
        .map_err(|error| RunError::Internal(format!("locking coordinator save: {error}")))?;
    let current = read_persisted_revision(state_dir)?;
    let expected = state.persistence_revision.get();
    let next_revision = next_revision(expected, current).map_err(RunError::Internal)?;
    debug_assert!(next_revision > expected);
    debug_assert_eq!(current, expected);
    Ok(CoordinatorSave {
        _guard: guard,
        state,
        next_revision,
    })
}

fn read_persisted_revision(state_dir: &Path) -> Result<u64, RunError> {
    let path = super::coordinator_state_path(state_dir);
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(RunError::Internal(format!("opening coordinator save revision: {error}"))),
    };
    let revision: PersistedRevision = serde_json::from_reader(std::io::BufReader::new(file))
        .map_err(|error| RunError::Internal(format!("reading coordinator save revision: {error}")))?;
    Ok(revision.persistence_revision)
}

fn next_revision(expected: u64, current: u64) -> Result<u64, String> {
    if expected != current {
        return Err(format!(
            "remote-coordinator-save-conflict: expected revision {expected}, persisted revision {current}; reload coordinator state before applying the transition"
        ));
    }
    expected.checked_add(1).ok_or_else(|| "remote-coordinator-save-revision-exhausted".into())
}

impl CoordinatorSave<'_> {
    pub(crate) fn snapshot(&self) -> RevisionedState<'_> {
        RevisionedState {
            state: self.state,
            persistence_revision: self.next_revision,
        }
    }

    /// Call only after file fsync, atomic rename, and parent-directory fsync.
    /// On every earlier error, dropping this transaction leaves the observed
    /// revision unchanged. Do not automatically retry a stale transition.
    pub(crate) fn commit(self) {
        self.state.persistence_revision.0.store(self.next_revision, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::next_revision;

    #[test]
    fn revision_fence_rejects_stale_and_exhausted_authority() {
        assert_eq!(next_revision(4, 4), Ok(5));
        assert!(next_revision(4, 5).unwrap_err().starts_with("remote-coordinator-save-conflict:"));
        assert!(next_revision(5, 4).unwrap_err().starts_with("remote-coordinator-save-conflict:"));
        assert_eq!(next_revision(u64::MAX, u64::MAX).unwrap_err(), "remote-coordinator-save-revision-exhausted");
    }
}
