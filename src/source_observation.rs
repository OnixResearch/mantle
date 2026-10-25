//! Monotonic ingest for source-bundle records.
//!
//! Only `Add` writes; identical reuse is write-free, and a conflicting record
//! is rejected before durable state changes.

use std::path::Path;

use crate::RunError;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;

/// Project only facts explicitly present in a v1 record. This never rewrites
/// the record or invents a profile/projection for historical source cohorts.
pub fn observation_for_record(
    record: &SourceRecord,
) -> Result<Option<crunch_source_core::SourceObservation>, RunError> {
    use crunch_source_core::LocatorClass;
    use crunch_source_core::SourceKind;
    let metadata = &record.metadata;
    let (Some(projection), Some(profile), Some(version)) = (
        metadata.get("projection"),
        metadata.get("snapshot_profile"),
        metadata.get("snapshot_profile_version"),
    ) else {
        return Ok(None);
    };
    let (kind, locator) = match record.kind {
        SourceRecordKind::FixedUrl | SourceRecordKind::BootstrapArchive => {
            let Some(url) = metadata.get("url") else {
                return Ok(None);
            };
            (SourceKind::FixedUrl, LocatorClass::FixedUrl { url: url.clone() })
        }
        SourceRecordKind::VcsSnapshot => {
            let Some(remote) = metadata.get("url") else {
                return Ok(None);
            };
            (SourceKind::Git, LocatorClass::GitRemote { remote: remote.clone() })
        }
        SourceRecordKind::LocalPath => (SourceKind::LocalLogical, LocatorClass::LocalLogical {
            path: record.identity.clone(),
        }),
        SourceRecordKind::PackageMirror => {
            let Some(mirror) = metadata.get("url") else {
                return Ok(None);
            };
            (SourceKind::PackageMirror, LocatorClass::PackageMirror { mirror: mirror.clone() })
        }
        SourceRecordKind::ProviderManifest | SourceRecordKind::ToolchainSourceRoot | SourceRecordKind::ProofInput => {
            (SourceKind::Opaque, LocatorClass::Opaque {
                label: record.identity.clone(),
            })
        }
    };
    let (git_object_format, git_revision) = if kind == SourceKind::Git {
        let (Some(format), Some(revision)) = (metadata.get("git_object_format"), metadata.get("rev")) else {
            return Ok(None);
        };
        let format = match format.as_str() {
            "sha1" => crunch_source_core::GitObjectFormat::Sha1,
            "sha256" => crunch_source_core::GitObjectFormat::Sha256,
            _ => return Err(observation_error("unsupported Git object format")),
        };
        (Some(format), Some(revision.clone()))
    } else {
        if metadata.contains_key("git_object_format") || metadata.contains_key("rev") {
            return Err(observation_error("Git facts require a Git source record"));
        }
        (None, None)
    };
    let request = crunch_source_core::SourceObservationRequest {
        kind,
        locator,
        mutable_ref_hint: None,
        git_object_format,
        git_revision,
        projection: crunch_source_core::ProjectionPath::parse(projection)
            .map_err(|_| observation_error("unsafe source projection"))?,
        snapshot_profile: crunch_source_core::SnapshotProfile {
            name: profile.clone(),
            version: version.parse().map_err(|_| observation_error("invalid snapshot profile version"))?,
        },
        payload_blake3: crunch_source_core::Blake3Digest::parse(record.content_blake3.clone())
            .map_err(|_| observation_error("invalid source content identity"))?,
    };
    let admission = crunch_source_core::admit_source_observation(&request, &Default::default());
    admission
        .observation
        .map(Some)
        .ok_or_else(|| observation_error("source facts failed observation admission"))
}

fn observation_error(message: &str) -> RunError {
    RunError::Internal(format!("source observation rejected: {message}"))
}

/// Outcome of planning one record against the durable state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordIngestOutcome {
    /// The identity is absent; the caller may write.
    Add,
    /// Canonical bytes already exist; no write occurs.
    ReuseIdentical,
}

/// Plan one record against the record already stored at its content identity.
/// Comparison is over the actual record, not a fabricated provenance observation.
pub fn plan_record_ingest(target: &Path, record: &SourceRecord) -> Result<RecordIngestOutcome, RunError> {
    // Complete explicit provenance is checked before any import mutation.
    // Absent legacy provenance remains unavailable, not silently backfilled.
    observation_for_record(record)?;
    let bytes = match std::fs::read(target) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RecordIngestOutcome::Add);
        }
        Err(error) => {
            return Err(RunError::Internal(format!("reading existing source record {}: {error}", target.display())));
        }
    };
    let existing: SourceRecord = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing existing source record {}: {error}", target.display())))?;
    if &existing == record {
        return Ok(RecordIngestOutcome::ReuseIdentical);
    }
    Err(RunError::Internal(format!(
        "source record identity conflict for {}: stored bytes differ from the incoming canonical record",
        record.content_blake3
    )))
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
            store_path_attestation: None,
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

    #[test]
    fn explicit_git_provenance_binds_revision_and_never_backfills_legacy_facts() {
        let revision = "1".repeat(64);
        let mut source = record(SourceRecordKind::VcsSnapshot, "git-source", vec![
            ("url", "https://example.invalid/source.git"),
            ("git_object_format", "sha256"),
            ("rev", &revision),
            ("projection", "source"),
            ("snapshot_profile", "mantle-source-snapshot"),
            ("snapshot_profile_version", "1"),
        ]);
        let original = observation_for_record(&source).unwrap().unwrap();
        source.metadata.insert("rev".into(), "2".repeat(64));
        let changed = observation_for_record(&source).unwrap().unwrap();
        assert_ne!(original.observation_blake3, changed.observation_blake3);
        source.metadata.remove("git_object_format");
        assert!(observation_for_record(&source).unwrap().is_none());
        source.metadata.remove("projection");
        assert!(observation_for_record(&source).unwrap().is_none());
    }

    #[test]
    fn invalid_explicit_provenance_rejects_before_existing_state_changes() {
        let scratch = tempfile::tempdir().unwrap();
        let target = scratch.path().join("record.json");
        let stored = record(SourceRecordKind::LocalPath, "logical-source", vec![]);
        std::fs::write(&target, serde_json::to_vec(&stored).unwrap()).unwrap();
        let before = std::fs::read(&target).unwrap();
        let mut incoming = stored;
        incoming.metadata.extend([
            ("projection".into(), "../escape".into()),
            ("snapshot_profile".into(), "mantle-source-snapshot".into()),
            ("snapshot_profile_version".into(), "1".into()),
        ]);
        assert!(plan_record_ingest(&target, &incoming).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), before);
        let absent = scratch.path().join("absent.json");
        assert!(plan_record_ingest(&absent, &incoming).is_err());
        assert!(!absent.exists());
    }
}
