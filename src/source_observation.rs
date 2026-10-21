//! Monotonic ingest for source-bundle records.
//!
//! Only `Add` writes; identical reuse is write-free, and a conflicting record
//! is rejected before durable state changes.

use std::path::Path;

use crate::RunError;
use crate::source_bundle::SourceRecord;

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
}
