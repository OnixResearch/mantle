//! Schema migration for manifest and lockfile formats.
//!
//! Each version step is an explicit function. Migrations run in
//! sequence from the file's current version to `SchemaVersion::CURRENT`.

use crate::error::Error;
use crate::lock::Lockfile;
use crate::version::SchemaVersion;

/// Maximum number of migration steps to prevent infinite loops.
const MAX_MIGRATION_STEPS: u32 = 64;

/// Upgrade a lockfile to the current schema version.
///
/// Returns the lockfile unchanged if it is already at the current version.
/// Returns an error if the version is newer than what we support.
pub fn upgrade_lockfile(mut lock: Lockfile) -> Result<Lockfile, Error> {
    let target = SchemaVersion::CURRENT;

    if lock.version == target {
        return Ok(lock);
    }

    if lock.version.major > target.major {
        return Err(Error::Upgrade(format!(
            "lockfile version {} is newer than supported version {target}",
            lock.version
        )));
    }

    let mut steps: u32 = 0;
    while lock.version != target {
        assert!(steps < MAX_MIGRATION_STEPS, "migration loop detected");
        lock = migrate_one_step(lock)?;
        steps = steps.saturating_add(1);
    }

    assert_eq!(lock.version, target);
    Ok(lock)
}

/// Apply a single migration step.
///
/// Each match arm handles one version-to-version transition.
fn migrate_one_step(lock: Lockfile) -> Result<Lockfile, Error> {
    match (lock.version.major, lock.version.minor, lock.version.patch) {
        // Future migrations go here:
        // (0, 1, 0) => migrate_0_1_0_to_1_0_0(lock),
        (major, minor, patch) => Err(Error::Upgrade(format!(
            "no migration defined from version {major}.{minor}.{patch}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::Lockfile;

    #[test]
    fn upgrade_current_is_noop() {
        let lock = Lockfile::new();
        assert_eq!(lock.version, SchemaVersion::CURRENT);
        let upgraded = upgrade_lockfile(lock.clone()).unwrap();
        assert_eq!(upgraded, lock);
    }

    #[test]
    fn upgrade_future_version_fails() {
        let mut lock = Lockfile::new();
        lock.version = SchemaVersion::new(99, 0, 0);
        let result = upgrade_lockfile(lock);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("newer than supported"));
    }

    #[test]
    fn upgrade_unknown_old_version_fails() {
        let mut lock = Lockfile::new();
        lock.version = SchemaVersion::new(0, 1, 0);
        let result = upgrade_lockfile(lock);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no migration defined"));
    }
}
