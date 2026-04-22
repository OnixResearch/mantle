use alloc::string::String;

use crate::generate::content_fingerprint;
use crate::generate::generate_inputs_ncl;
use crate::lock::Lockfile;

#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    InSync,
    Drifted {
        expected_fingerprint: u64,
        actual_fingerprint: u64,
    },
    Missing,
}

impl DriftStatus {
    pub fn is_ok(self) -> bool {
        matches!(self, DriftStatus::InSync)
    }
}

pub fn check_drift(lock: Lockfile, actual_content: Option<String>) -> DriftStatus {
    let actual_content = match actual_content {
        Some(content) => content,
        None => return DriftStatus::Missing,
    };

    let expected = generate_inputs_ncl(lock);
    let expected_fp = content_fingerprint(expected);
    let actual_fp = content_fingerprint(actual_content);

    if expected_fp == actual_fp {
        DriftStatus::InSync
    } else {
        DriftStatus::Drifted {
            expected_fingerprint: expected_fp,
            actual_fingerprint: actual_fp,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;

    use super::*;
    use crate::lock::*;
    use crate::manifest::HashAlgo;
    use crate::version::SchemaVersion;

    fn simple_lock() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert("foo".to_string(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/foo".to_string(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-abc=".to_string(),
            },
            patches: alloc::vec![],
            mirrors: alloc::vec![],
        });
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: BTreeMap::new(),
        }
    }

    #[test]
    fn in_sync() {
        let lock = simple_lock();
        let expected = generate_inputs_ncl(lock.clone());
        assert_eq!(check_drift(lock, Some(expected)), DriftStatus::InSync);
    }

    #[test]
    fn drifted() {
        let lock = simple_lock();
        let status = check_drift(lock, Some("stale content".to_string()));
        assert!(matches!(status, DriftStatus::Drifted { .. }));
    }

    #[test]
    fn missing_file() {
        let lock = simple_lock();
        assert_eq!(check_drift(lock, None), DriftStatus::Missing);
    }

    #[test]
    fn empty_lock_in_sync_with_empty_generated() {
        let lock = Lockfile::new();
        let expected = generate_inputs_ncl(lock.clone());
        assert_eq!(check_drift(lock, Some(expected)), DriftStatus::InSync);
    }
}
