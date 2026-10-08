//! Source-observation adapters and monotonic ingest for source-bundle records.
//!
//! Adapter inputs (fixed URL, Git, local logical, package mirror, opaque) map
//! onto admitted observations from `crunch-source-core`. Ingest planning is
//! monotonic: only `Add` writes, identical reuse is write-free, and a
//! conflicting record is rejected before any durable state changes.

use std::path::Path;

use crunch_source_core::ExistingObservation;
use crunch_source_core::GitObjectFormat;
use crunch_source_core::LocatorBoundaryPolicy;
use crunch_source_core::LocatorClass;
use crunch_source_core::ProjectionPath;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservation;
use crunch_source_core::SourceObservationRequest;
use crunch_source_core::admit_source_observation;

use crate::RunError;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;

/// Snapshot profile recorded for source-bundle observations.
pub const SOURCE_BUNDLE_SNAPSHOT_PROFILE: &str = "mantle-source-snapshot";

/// Snapshot profile version recorded for source-bundle observations.
pub const SOURCE_BUNDLE_SNAPSHOT_PROFILE_VERSION: u32 = 1;

/// Default projection when a record does not name one.
const DEFAULT_PROJECTION: &str = "source";

/// Map one source record onto an admitted observation when its adapter facts
/// are sufficient. Insufficient or unrecognized facts yield `None`, which the
/// caller reports as `provenance-unavailable` rather than backfilling.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "I9 source observation awaits external source-review and Leviathan V2 prerequisites"
    )
)]
pub fn source_observation_for_record(record: &SourceRecord) -> Result<Option<SourceObservation>, RunError> {
    let Some(request) = observation_request_for_record(record) else {
        return Ok(None);
    };
    let policy = LocatorBoundaryPolicy {
        allowed_query_fields: Vec::new(),
    };
    let admission = admit_source_observation(&request, &policy);
    if let Some(observation) = admission.observation {
        debug_assert_eq!(observation.schema, crunch_source_core::SOURCE_OBSERVATION_SCHEMA);
        return Ok(Some(observation));
    }
    // The record stays valid; only its provenance projection is unavailable.
    debug_assert!(!admission.diagnostics.is_empty());
    Ok(None)
}

fn observation_request_for_record(record: &SourceRecord) -> Option<SourceObservationRequest> {
    let projection = ProjectionPath::parse(&projection_of(record)).ok()?;
    let snapshot_profile = SnapshotProfile {
        name: String::from(SOURCE_BUNDLE_SNAPSHOT_PROFILE),
        version: SOURCE_BUNDLE_SNAPSHOT_PROFILE_VERSION,
    };
    let payload_blake3 = parse_content_digest(&record.content_blake3)?;
    let (kind, locator, git_object_format, git_revision) = match record.kind {
        SourceRecordKind::VcsSnapshot => {
            let remote = record.identity.strip_prefix("git+")?.to_string();
            let revision = metadata_value(record, "commit")?;
            let git_object_format = match revision.len() {
                40 => GitObjectFormat::Sha1,
                64 => GitObjectFormat::Sha256,
                _ => return None,
            };
            (SourceKind::Git, LocatorClass::GitRemote { remote }, Some(git_object_format), Some(revision))
        }
        SourceRecordKind::FixedUrl | SourceRecordKind::BootstrapArchive => (
            SourceKind::FixedUrl,
            LocatorClass::FixedUrl {
                url: record.identity.clone(),
            },
            None,
            None,
        ),
        SourceRecordKind::LocalPath => (
            SourceKind::LocalLogical,
            LocatorClass::LocalLogical {
                path: record.identity.clone(),
            },
            None,
            None,
        ),
        SourceRecordKind::PackageMirror => (
            SourceKind::PackageMirror,
            LocatorClass::PackageMirror {
                mirror: record.identity.clone(),
            },
            None,
            None,
        ),
        SourceRecordKind::ProviderManifest | SourceRecordKind::ToolchainSourceRoot | SourceRecordKind::ProofInput => (
            SourceKind::Opaque,
            LocatorClass::Opaque {
                label: record.identity.clone(),
            },
            None,
            None,
        ),
    };
    Some(SourceObservationRequest {
        kind,
        locator,
        mutable_ref_hint: metadata_value(record, "reference"),
        git_object_format,
        git_revision,
        projection,
        snapshot_profile,
        payload_blake3,
    })
}

fn projection_of(record: &SourceRecord) -> String {
    metadata_value(record, "projection").unwrap_or_else(|| String::from(DEFAULT_PROJECTION))
}

fn metadata_value(record: &SourceRecord, key: &str) -> Option<String> {
    record
        .metadata
        .get(key)
        .cloned()
        .or_else(|| record.adapter.as_ref().and_then(|adapter| adapter.extra.get(key).cloned()))
}

fn parse_content_digest(value: &str) -> Option<crunch_source_core::Blake3Digest> {
    crunch_source_core::Blake3Digest::parse(value.to_string()).ok()
}

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
    fn git_records_project_to_git_observations() {
        let git_record = record(SourceRecordKind::VcsSnapshot, "git+https://example.invalid/source.git", vec![(
            "commit",
            "1320a983e6c3d1e2fb53dd2464b084b4903b1426",
        )]);
        // 40-character revision keeps the Git SHA-1 interoperability class.
        let observation = source_observation_for_record(&git_record)
            .expect("adapter mapping runs")
            .expect("git facts are sufficient");
        assert_eq!(observation.kind, SourceKind::Git);
        assert_eq!(observation.git_object_format, Some(GitObjectFormat::Sha1));
        assert_eq!(observation.git_revision.as_deref().map(str::len), Some(40));
    }

    #[test]
    fn url_and_opaque_records_project_or_report_unavailable() {
        let url_record = record(SourceRecordKind::FixedUrl, "https://example.invalid/a.tar", vec![]);
        assert!(source_observation_for_record(&url_record).expect("mapping runs").is_some());

        let opaque_record = record(SourceRecordKind::ProofInput, "proof-input-1", vec![]);
        assert!(source_observation_for_record(&opaque_record).expect("mapping runs").is_some());

        let insufficient = record(SourceRecordKind::VcsSnapshot, "git+https://example.invalid/x.git", vec![]);
        assert!(source_observation_for_record(&insufficient).expect("mapping runs").is_none());
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
