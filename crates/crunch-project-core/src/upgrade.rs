use alloc::format;

use crate::Error;
use crate::lock::Lockfile;
use crate::version::SchemaVersion;

const MAX_MIGRATION_STEPS: u32 = 64;

pub const OLDEST_SUPPORTED: SchemaVersion = SchemaVersion {
    major: 0,
    minor: 9,
    patch: 0,
};

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

fn migrate_one_step(lock: Lockfile) -> Result<Lockfile, Error> {
    match (lock.version.major, lock.version.minor, lock.version.patch) {
        (0, 9, 0) => Ok(Lockfile {
            version: SchemaVersion::CURRENT,
            ..lock
        }),
        (major, minor, patch) => {
            Err(Error::Upgrade(format!("no migration defined from version {major}.{minor}.{patch}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;

    use super::*;
    use crate::lock::LockEntry;
    use crate::lock::LockedHash;
    use crate::lock::LockedKind;
    use crate::lock::LockedPatch;
    use crate::lock::LockedPatchSource;
    use crate::manifest::HashAlgo;

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
        inputs.insert("example".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/file".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-abc=".into(),
            },
            patches: alloc::vec![],
            mirrors: alloc::vec![],
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
        });

        let lock = Lockfile {
            version: SchemaVersion::new(0, 9, 0),
            inputs,
            patches: BTreeMap::new(),
        };

        let upgraded = upgrade_lockfile(lock).unwrap();
        assert_eq!(upgraded.version, SchemaVersion::CURRENT);
        assert!(upgraded.inputs.contains_key("example"));
        assert_eq!(upgraded.inputs["example"].hash.value, "sha256-abc=");
    }

    #[test]
    fn upgrade_preserves_all_data() {
        let mut inputs = BTreeMap::new();
        inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::Git {
                repository: "https://github.com/user/repo.git".into(),
                rev: "abc123".into(),
                ref_name: Some("main".into()),
            },
            hash: LockedHash {
                algo: HashAlgo::Blake3,
                value: "blake3-xyz=".into(),
            },
            patches: alloc::vec!["fix1".into()],
            mirrors: alloc::vec!["https://mirror.example.com/repo.git".into()],
            fetch_policy: crate::InputFetchPolicy::GenerationMaterial,
        });

        let mut patches = BTreeMap::new();
        patches.insert("fix1".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "patches/fix1.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-patchhash=".into(),
            },
        });

        let lock = Lockfile {
            version: SchemaVersion::new(0, 9, 0),
            inputs,
            patches,
        };

        let upgraded = upgrade_lockfile(lock).unwrap();
        assert_eq!(upgraded.version, SchemaVersion::CURRENT);

        let pkg = &upgraded.inputs["pkg"];
        assert_eq!(pkg.patches, alloc::vec!["fix1"]);
        assert_eq!(pkg.mirrors.len(), 1);

        let patch = &upgraded.patches["fix1"];
        assert_eq!(patch.hash.value, "sha256-patchhash=");
    }
}
