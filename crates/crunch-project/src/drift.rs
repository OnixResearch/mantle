//! Drift detection between lockfile and generated inputs file.
//!
//! Compares what `.crunch/inputs.ncl` should contain (from the current
//! lockfile) against what it actually contains on disk. Pure logic
//! except for reading the generated file.

use crate::generate::{content_fingerprint, generate_inputs_ncl};
use crate::lock::Lockfile;

/// Result of a drift check.
#[derive(Debug, Clone, PartialEq)]
pub enum DriftStatus {
    /// The generated file matches the lockfile.
    InSync,
    /// The generated file does not match the lockfile.
    Drifted {
        expected_fingerprint: u64,
        actual_fingerprint: u64,
    },
    /// The generated file does not exist.
    Missing,
}

impl DriftStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, DriftStatus::InSync)
    }
}

/// Check whether a generated inputs file matches the lockfile.
///
/// `actual_content` is the current on-disk content of `.crunch/inputs.ncl`
/// (or `None` if the file does not exist). The expected content is
/// regenerated from the lockfile and compared by fingerprint.
pub fn check_drift(lock: &Lockfile, actual_content: Option<&str>) -> DriftStatus {
    let actual_content = match actual_content {
        Some(c) => c,
        None => return DriftStatus::Missing,
    };

    let expected = generate_inputs_ncl(lock);
    let expected_fp = content_fingerprint(&expected);
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
    use super::*;
    use crate::lock::*;
    use crate::manifest::HashAlgo;
    use crate::version::SchemaVersion;
    use std::collections::BTreeMap;

    fn simple_lock() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert(
            "foo".to_string(),
            LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/foo".to_string(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-abc=".to_string(),
                },
                patches: vec![],
                mirrors: vec![],
            },
        );
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: BTreeMap::new(),
        }
    }

    #[test]
    fn in_sync() {
        let lock = simple_lock();
        let expected = generate_inputs_ncl(&lock);
        assert_eq!(check_drift(&lock, Some(&expected)), DriftStatus::InSync);
    }

    #[test]
    fn drifted() {
        let lock = simple_lock();
        let status = check_drift(&lock, Some("stale content"));
        assert!(matches!(status, DriftStatus::Drifted { .. }));
    }

    #[test]
    fn missing_file() {
        let lock = simple_lock();
        assert_eq!(check_drift(&lock, None), DriftStatus::Missing);
    }

    #[test]
    fn empty_lock_in_sync_with_empty_generated() {
        let lock = Lockfile::new();
        let expected = generate_inputs_ncl(&lock);
        assert_eq!(check_drift(&lock, Some(&expected)), DriftStatus::InSync);
    }
}
