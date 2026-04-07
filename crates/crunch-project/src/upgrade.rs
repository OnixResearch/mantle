//! Schema migration for manifest and lockfile formats.
//!
//! Each version step is an explicit function. Migrations run in
//! sequence from the file's current version to `SchemaVersion::CURRENT`.

use crate::error::Error;
use crate::lock::Lockfile;
use crate::version::SchemaVersion;

/// Maximum number of migration steps to prevent infinite loops.
const MAX_MIGRATION_STEPS: u32 = 64;

/// The oldest schema version we can migrate from.
pub const OLDEST_SUPPORTED: SchemaVersion = SchemaVersion {
    major: 0,
    minor: 9,
    patch: 0,
};

/// Upgrade a lockfile to the current schema version.
///
/// Returns the lockfile unchanged if it is already at the current version.
/// Returns an error if the version is newer than what we support or
/// older than the oldest we have a migration for.
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
        // 0.9.0 -> 1.0.0: first supported migration.
        // The 0.9.0 format is identical in structure to 1.0.0 — only
        // the version field changes. This exists so the upgrade path
        // is exercised from the start.
        (0, 9, 0) => Ok(Lockfile {
            version: SchemaVersion::CURRENT,
            ..lock
        }),
        (major, minor, patch) => Err(Error::Upgrade(format!(
            "no migration defined from version {major}.{minor}.{patch}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::*;
    use crate::manifest::HashAlgo;
    use std::collections::BTreeMap;

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

    #[test]
    fn upgrade_0_9_0_to_1_0_0() {
        let mut inputs = BTreeMap::new();
        inputs.insert(
            "example".into(),
            LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/file".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-abc=".into(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        );

        let lock = Lockfile {
            version: SchemaVersion::new(0, 9, 0),
            inputs,
            patches: BTreeMap::new(),
        };

        let upgraded = upgrade_lockfile(lock).unwrap();
        assert_eq!(upgraded.version, SchemaVersion::CURRENT);
        // Data preserved through migration
        assert!(upgraded.inputs.contains_key("example"));
        assert_eq!(upgraded.inputs["example"].hash.value, "sha256-abc=");
    }

    #[test]
    fn upgrade_preserves_all_data() {
        let mut inputs = BTreeMap::new();
        inputs.insert(
            "pkg".into(),
            LockEntry {
                kind: LockedKind::Git {
                    repository: "https://github.com/user/repo.git".into(),
                    rev: "abc123".into(),
                    ref_name: Some("main".into()),
                },
                hash: LockedHash {
                    algo: HashAlgo::Blake3,
                    value: "blake3-xyz=".into(),
                },
                patches: vec!["fix1".into()],
                mirrors: vec!["https://mirror.example.com/repo.git".into()],
            },
        );

        let mut patches = BTreeMap::new();
        patches.insert(
            "fix1".into(),
            LockedPatch {
                source: LockedPatchSource::Local {
                    path: "patches/fix1.patch".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-patchhash=".into(),
                },
            },
        );

        let lock = Lockfile {
            version: SchemaVersion::new(0, 9, 0),
            inputs,
            patches,
        };

        let upgraded = upgrade_lockfile(lock).unwrap();
        assert_eq!(upgraded.version, SchemaVersion::CURRENT);

        let pkg = &upgraded.inputs["pkg"];
        assert_eq!(pkg.patches, vec!["fix1"]);
        assert_eq!(pkg.mirrors.len(), 1);

        let patch = &upgraded.patches["fix1"];
        assert_eq!(patch.hash.value, "sha256-patchhash=");
    }
}
