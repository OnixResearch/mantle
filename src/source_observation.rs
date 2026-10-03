//! Monotonic ingest for source-bundle records.
//!
//! Only `Add` writes; identical reuse is write-free, and a conflicting record
//! is rejected before durable state changes.

use std::path::Path;

use crunch_source_core::ExistingObservation;
use crunch_source_core::LocatorClass;
use crunch_source_core::ProjectionPath;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservation;

use crate::RunError;
use crate::source_bundle::SourceRecord;

/// Snapshot profile recorded for source-bundle observations.
pub const SOURCE_BUNDLE_SNAPSHOT_PROFILE: &str = "mantle-source-snapshot";

/// Snapshot profile version recorded for source-bundle observations.
pub const SOURCE_BUNDLE_SNAPSHOT_PROFILE_VERSION: u32 = 1;

/// Stable projection for canonical stored-record comparisons.
const DEFAULT_PROJECTION: &str = "source";

/// Outcome of planning one record against the durable state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordIngestOutcome {
    /// The identity is absent; the caller may write.
    Add,
    /// Canonical bytes already exist; no write occurs.
    ReuseIdentical,
}

/// Read the canonical record already stored under one content identity.
fn read_existing_record(target: &Path) -> Result<ExistingObservation, RunError> {
    let bytes = match std::fs::read(target) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ExistingObservation::Absent);
        }
        Err(error) => {
            return Err(RunError::Internal(format!("reading existing source record {}: {error}", target.display())));
        }
    };
    let existing: SourceRecord = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing existing source record {}: {error}", target.display())))?;
    Ok(ExistingObservation::Present(Box::new(canonical_record_observation(&existing))))
}

/// Canonical observation-shaped facts for one stored record.
///
/// Records without sufficient adapter facts still need a stable identity for
/// reuse comparison, so the canonical byte digest and content identity carry
/// the comparison instead of a derived observation.
fn canonical_record_observation(record: &SourceRecord) -> SourceObservation {
    let digest =
        crunch_source_core::Blake3Digest::from_slice(serde_json::to_vec(record).unwrap_or_default().as_slice());
    SourceObservation {
        schema: String::from("mantle-source-record-canonical-v1"),
        kind: SourceKind::Opaque,
        locator: LocatorClass::Opaque {
            label: record.content_blake3.clone(),
        },
        mutable_ref_hint: None,
        git_object_format: None,
        git_revision: None,
        projection: ProjectionPath::parse(DEFAULT_PROJECTION)
            .unwrap_or_else(|_| unreachable!("default projection always parses")),
        snapshot_profile: SnapshotProfile {
            name: String::from(SOURCE_BUNDLE_SNAPSHOT_PROFILE),
            version: SOURCE_BUNDLE_SNAPSHOT_PROFILE_VERSION,
        },
        payload_blake3: digest.clone(),
        encoding_version: crunch_source_core::SOURCE_OBSERVATION_ENCODING_VERSION,
        observation_blake3: digest,
    }
}

/// Plan one record against the record already stored at its content identity.
pub fn plan_record_ingest(target: &Path, record: &SourceRecord) -> Result<RecordIngestOutcome, RunError> {
    let existing = read_existing_record(target)?;
    let incoming = canonical_record_observation(record);
    match existing {
        ExistingObservation::Absent => {
            debug_assert!(!record.content_blake3.is_empty());
            Ok(RecordIngestOutcome::Add)
        }
        ExistingObservation::Present(existing_observation) => {
            if existing_observation.observation_blake3 == incoming.observation_blake3 {
                Ok(RecordIngestOutcome::ReuseIdentical)
            } else {
                Err(RunError::Internal(format!(
                    "source record identity conflict for {}: stored bytes differ from the incoming canonical record",
                    record.content_blake3
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::source_bundle::SourceFileType;
    use crate::source_bundle::SourceRecordKind;

    fn record(kind: SourceRecordKind, identity: &str, metadata: Vec<(&str, &str)>) -> SourceRecord {
        SourceRecord {
            kind,
            identity: identity.to_string(),
            store_prefix: None,
            adapter: None,
            metadata: metadata
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<BTreeMap<_, _>>(),
            payload_bytes: 16,
            content_blake3: "a".repeat(64),
            files: vec![crate::source_bundle::SourceFileEntry {
                path: "source".to_string(),
                file_type: SourceFileType::Regular,
                executable: false,
                size: 16,
                content_hex: Some(String::from("00")),
                symlink_target: None,
                chunk_index: None,
                chunk_count: None,
                blake3: "b".repeat(64),
            }],
        }
    }

    fn temp_target(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("mantle-source-record-{label}-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn absent_record_plans_an_add() {
        let target = temp_target("absent");
        let plan = plan_record_ingest(&target, &record(SourceRecordKind::FixedUrl, "https://x.invalid", vec![]))
            .expect("planning succeeds");
        assert_eq!(plan, RecordIngestOutcome::Add);
        assert!(!target.exists());
    }

    #[test]
    fn identical_record_reuses_without_writing() {
        let target = temp_target("identical");
        let stored = record(SourceRecordKind::FixedUrl, "https://x.invalid", vec![]);
        std::fs::write(&target, serde_json::to_vec(&stored).expect("serializes")).expect("write fixture");
        let plan = plan_record_ingest(&target, &stored).expect("planning succeeds");
        assert_eq!(plan, RecordIngestOutcome::ReuseIdentical);
        let _ = std::fs::remove_file(&target);
    }

    #[test]
    fn conflicting_record_rejects_and_leaves_state_unchanged() {
        let target = temp_target("conflict");
        let stored = record(SourceRecordKind::FixedUrl, "https://x.invalid", vec![]);
        std::fs::write(&target, serde_json::to_vec(&stored).expect("serializes")).expect("write fixture");
        let before = std::fs::read(&target).expect("read before");
        let mut conflicting = stored.clone();
        conflicting.payload_bytes = 32;
        let outcome = plan_record_ingest(&target, &conflicting);
        assert!(outcome.is_err());
        let after = std::fs::read(&target).expect("read after");
        assert_eq!(before, after);
        let _ = std::fs::remove_file(&target);
    }
}
