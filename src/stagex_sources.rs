use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_payload;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const SOURCE_RECORD_REVISION_KEY: &str = "rev";
const STAGEX_SOURCE_RECORD_COUNT: usize = 7;
pub(crate) const STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3: &str =
    "6d6e2f0ae906c3a421b90e220fed77e6f90b556cd7982731352ce28f39f5a796";
const STAGEX_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-source-materialization-v1";
const STAGEX_SOURCE_NON_CLAIM: &str = "source materialization proves authenticated payload identity and placement only; it does not prove executable lineage or provider admission";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StagexSourceSpec {
    artifact_id: &'static str,
    record_name: &'static str,
    output_name: &'static str,
    content_blake3: &'static str,
}

const STAGEX_SOURCE_SPECS: [StagexSourceSpec; STAGEX_SOURCE_RECORD_COUNT] = [
    StagexSourceSpec {
        artifact_id: "stage0-posix-amd64-source",
        record_name: "stage0-posix-amd64",
        output_name: "AMD64",
        content_blake3: "f983a7b79acdd53fd4d31c8959c62ca24eabbea491327aaa113c4bc11549e5f9",
    },
    StagexSourceSpec {
        artifact_id: "bootstrap-seeds-source",
        record_name: "bootstrap-seeds",
        output_name: "bootstrap-seeds",
        content_blake3: "30d135a3e1c83495b521ae969d2c0e97da44baff7e143a16bcab0919fb0a36d2",
    },
    StagexSourceSpec {
        artifact_id: "m2-planet-source",
        record_name: "M2-Planet",
        output_name: "M2-Planet",
        content_blake3: "a4b081c01854e735697908471cd6170e270f5084170a64e51c5a20302c842d89",
    },
    StagexSourceSpec {
        artifact_id: "m2libc-source",
        record_name: "M2libc",
        output_name: "M2libc",
        content_blake3: "13450e1e9a31edfbeccd0adc30c52b2d085d4c3c6b58f42a1b6f902e903f3b4a",
    },
    StagexSourceSpec {
        artifact_id: "mescc-tools-source",
        record_name: "mescc-tools",
        output_name: "mescc-tools",
        content_blake3: "2dcf2577f60e195efa112a8d8978b41542a13f25ef0ee3d9ab4068256096ce94",
    },
    StagexSourceSpec {
        artifact_id: "mescc-tools-extra-source",
        record_name: "mescc-tools-extra",
        output_name: "mescc-tools-extra",
        content_blake3: "0cc000667e857118296d9abd7959a323024eed0fc09231d307985e9ffe7a6ae6",
    },
    StagexSourceSpec {
        artifact_id: "m2-mesoplanet-source",
        record_name: "M2-Mesoplanet",
        output_name: "M2-Mesoplanet",
        content_blake3: "f2c5526d58cc9424f32e78f6d96e9c271237998a7c8ac007d80fdd118d50fa47",
    },
];

pub(crate) fn expected_stagex_source_artifact_digests() -> [(&'static str, &'static str); STAGEX_SOURCE_RECORD_COUNT] {
    STAGEX_SOURCE_SPECS.map(|spec| (spec.artifact_id, spec.content_blake3))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StagexMaterializedSource {
    pub record_name: String,
    pub output_name: String,
    pub record_identity: String,
    pub revision: String,
    pub content_blake3: String,
    pub payload_bytes: u64,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StagexSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub source_count: u32,
    pub sources: Vec<StagexMaterializedSource>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum StagexSourceError {
    Bundle(crate::RunError),
    InvalidAuthority(String),
    Materialization(String),
}

impl std::fmt::Display for StagexSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "reading authenticated StageX source bundle: {error}"),
            Self::InvalidAuthority(message) => write!(formatter, "invalid StageX source authority: {message}"),
            Self::Materialization(message) => write!(formatter, "materializing StageX sources: {message}"),
        }
    }
}

impl std::error::Error for StagexSourceError {}

impl From<crate::RunError> for StagexSourceError {
    fn from(error: crate::RunError) -> Self {
        Self::Bundle(error)
    }
}

pub(crate) fn materialize_authenticated_stagex_sources(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    source_root: &Path,
) -> Result<StagexSourceMaterializationReport, StagexSourceError> {
    let manifest = read_source_bundle(bundle_path)?;
    require_manifest_authority(&manifest, expected_manifest_blake3)?;
    materialize_stagex_sources_from_manifest(&manifest, source_root)
}

fn require_manifest_authority(
    manifest: &SourceBundleManifest,
    expected_manifest_blake3: &str,
) -> Result<(), StagexSourceError> {
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexSourceError::InvalidAuthority(format!(
            "source bundle manifest BLAKE3 mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    assert!(!manifest.records.is_empty());
    assert_eq!(manifest.manifest_blake3.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn materialize_stagex_sources_from_manifest(
    manifest: &SourceBundleManifest,
    source_root: &Path,
) -> Result<StagexSourceMaterializationReport, StagexSourceError> {
    let selected = select_stagex_source_records(&manifest.records)?;
    fs::create_dir(source_root).map_err(|error| {
        StagexSourceError::Materialization(format!(
            "creating create-new source root {}: {error}",
            source_root.display()
        ))
    })?;
    let mut sources = Vec::with_capacity(STAGEX_SOURCE_RECORD_COUNT);
    for (spec, record) in STAGEX_SOURCE_SPECS.iter().zip(selected) {
        let output_path = source_root.join(spec.output_name);
        if output_path.exists() {
            return Err(StagexSourceError::Materialization(format!(
                "create-new source target already exists: {}",
                output_path.display()
            )));
        }
        materialize_source_record_payload(record, &output_path).map_err(|error| {
            StagexSourceError::Materialization(format!(
                "materializing source record {} at {}: {error}",
                record.identity,
                output_path.display()
            ))
        })?;
        sources.push(materialized_source(spec, record, output_path)?);
    }
    assert_eq!(sources.len(), STAGEX_SOURCE_RECORD_COUNT);
    assert!(sources.iter().all(|source| source.output_path.starts_with(source_root)));
    Ok(StagexSourceMaterializationReport {
        format: STAGEX_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3.clone(),
        source_count: u32::try_from(sources.len())
            .map_err(|_| StagexSourceError::Materialization("source count does not fit in u32".to_string()))?,
        sources,
        non_claim: STAGEX_SOURCE_NON_CLAIM,
    })
}

fn select_stagex_source_records(records: &[SourceRecord]) -> Result<Vec<&SourceRecord>, StagexSourceError> {
    let mut by_name = BTreeMap::new();
    for record in records {
        let Some(name) = record.metadata.get(SOURCE_RECORD_NAME_KEY) else {
            continue;
        };
        if !STAGEX_SOURCE_SPECS.iter().any(|spec| spec.record_name == name) {
            continue;
        }
        if by_name.insert(name.as_str(), record).is_some() {
            return Err(StagexSourceError::InvalidAuthority(format!(
                "source bundle contains duplicate required record '{name}'"
            )));
        }
    }
    let mut selected = Vec::with_capacity(STAGEX_SOURCE_RECORD_COUNT);
    for spec in STAGEX_SOURCE_SPECS {
        let record = by_name.get(spec.record_name).copied().ok_or_else(|| {
            StagexSourceError::InvalidAuthority(format!(
                "source bundle is missing required record '{}'",
                spec.record_name
            ))
        })?;
        validate_selected_record(spec, record)?;
        selected.push(record);
    }
    assert_eq!(selected.len(), STAGEX_SOURCE_RECORD_COUNT);
    assert!(selected.iter().all(|record| record.kind == SourceRecordKind::VcsSnapshot));
    Ok(selected)
}

fn validate_selected_record(spec: StagexSourceSpec, record: &SourceRecord) -> Result<(), StagexSourceError> {
    if record.kind != SourceRecordKind::VcsSnapshot {
        return Err(StagexSourceError::InvalidAuthority(format!(
            "required source '{}' is not a VCS snapshot",
            spec.record_name
        )));
    }
    if record.files.is_empty() {
        return Err(StagexSourceError::InvalidAuthority(format!(
            "required source '{}' has no authenticated payload files",
            spec.record_name
        )));
    }
    if record.content_blake3 != spec.content_blake3 {
        return Err(StagexSourceError::InvalidAuthority(format!(
            "required source '{}' content BLAKE3 mismatch: expected {}, observed {}",
            spec.record_name, spec.content_blake3, record.content_blake3
        )));
    }
    let revision = record.metadata.get(SOURCE_RECORD_REVISION_KEY).ok_or_else(|| {
        StagexSourceError::InvalidAuthority(format!("required source '{}' has no revision identity", spec.record_name))
    })?;
    if revision.is_empty() {
        return Err(StagexSourceError::InvalidAuthority(format!(
            "required source '{}' has an empty revision identity",
            spec.record_name
        )));
    }
    assert!(!record.content_blake3.is_empty());
    assert!(record.payload_bytes > 0);
    Ok(())
}

fn materialized_source(
    spec: &StagexSourceSpec,
    record: &SourceRecord,
    output_path: PathBuf,
) -> Result<StagexMaterializedSource, StagexSourceError> {
    let revision = record.metadata.get(SOURCE_RECORD_REVISION_KEY).cloned().ok_or_else(|| {
        StagexSourceError::InvalidAuthority(format!(
            "required source '{}' lost its revision identity",
            spec.record_name
        ))
    })?;
    assert_eq!(record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some(spec.record_name));
    assert!(!record.files.is_empty());
    Ok(StagexMaterializedSource {
        record_name: spec.record_name.to_string(),
        output_name: spec.output_name.to_string(),
        record_identity: record.identity.clone(),
        revision,
        content_blake3: record.content_blake3.clone(),
        payload_bytes: record.payload_bytes,
        output_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_bundle::SourceFileEntry;
    use crate::source_bundle::SourceFileType;

    const TEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TEST_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_SCRATCH_ENV: &str = "MANTLE_STAGE_X_SOURCE_SCRATCH";
    const FULL_SOURCE_CLOSURE_BLAKE3: &str = "7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45";

    fn required_record(name: &str) -> SourceRecord {
        let content_blake3 = STAGEX_SOURCE_SPECS
            .iter()
            .find(|spec| spec.record_name == name)
            .map(|spec| spec.content_blake3)
            .unwrap();
        SourceRecord {
            kind: SourceRecordKind::VcsSnapshot,
            identity: format!("vcs-snapshot-{name}"),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([
                (SOURCE_RECORD_NAME_KEY.to_string(), name.to_string()),
                (SOURCE_RECORD_REVISION_KEY.to_string(), TEST_REVISION.to_string()),
            ]),
            payload_bytes: 1,
            content_blake3: content_blake3.to_string(),
            files: vec![SourceFileEntry {
                path: "README".to_string(),
                file_type: SourceFileType::Regular,
                executable: false,
                size: 1,
                content_hex: Some("78".to_string()),
                symlink_target: None,
                chunk_index: None,
                chunk_count: None,
                blake3: TEST_BLAKE3.to_string(),
            }],
        }
    }

    fn required_records() -> Vec<SourceRecord> {
        STAGEX_SOURCE_SPECS.iter().map(|spec| required_record(spec.record_name)).collect()
    }

    #[test]
    fn selects_complete_authenticated_stagex_source_set_in_fixed_order() {
        let mut records = required_records();
        records.reverse();

        let selected = select_stagex_source_records(&records).unwrap();

        assert_eq!(selected.len(), STAGEX_SOURCE_RECORD_COUNT);
        assert_eq!(selected[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some("stage0-posix-amd64"));
        assert_eq!(
            selected[STAGEX_SOURCE_RECORD_COUNT - 1].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str),
            Some("M2-Mesoplanet")
        );
    }

    #[test]
    fn rejects_missing_required_stagex_source() {
        let mut records = required_records();
        records.pop();

        let error = select_stagex_source_records(&records).unwrap_err();

        assert!(error.to_string().contains("missing required record 'M2-Mesoplanet'"));
        assert!(!error.to_string().contains("materializing"));
    }

    #[test]
    fn rejects_duplicate_required_stagex_source() {
        let mut records = required_records();
        records.push(required_record("M2-Planet"));

        let error = select_stagex_source_records(&records).unwrap_err();

        assert!(error.to_string().contains("duplicate required record 'M2-Planet'"));
        assert!(!error.to_string().contains("missing required record"));
    }

    #[test]
    fn rejects_non_vcs_or_payload_free_required_source() {
        let mut non_vcs = required_records();
        non_vcs[0].kind = SourceRecordKind::LocalPath;
        let non_vcs_error = select_stagex_source_records(&non_vcs).unwrap_err();
        assert!(non_vcs_error.to_string().contains("is not a VCS snapshot"));

        let mut payload_free = required_records();
        payload_free[0].files.clear();
        let payload_error = select_stagex_source_records(&payload_free).unwrap_err();
        assert!(payload_error.to_string().contains("has no authenticated payload files"));
    }

    #[test]
    fn rejects_substituted_source_bundle_manifest_authority() {
        let manifest = SourceBundleManifest {
            format: "mantle-source-bundle-v1".to_string(),
            version: 1,
            store_prefix: "/mantle/store".to_string(),
            roots: Vec::new(),
            records: required_records(),
            manifest_blake3: TEST_BLAKE3.to_string(),
            non_claim: "test".to_string(),
        };

        let error =
            require_manifest_authority(&manifest, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
                .unwrap_err();

        assert!(error.to_string().contains("manifest BLAKE3 mismatch"));
        assert!(error.to_string().contains(TEST_BLAKE3));
    }

    #[test]
    #[ignore = "requires an explicitly authenticated retained source bundle and create-new scratch"]
    fn materializes_retained_authenticated_stagex_sources() {
        let bundle_path = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_SCRATCH_ENV).unwrap());

        let report =
            materialize_authenticated_stagex_sources(&bundle_path, FULL_SOURCE_CLOSURE_BLAKE3, &scratch).unwrap();
        let report_path = scratch.join("source-materialization.json");
        let report_bytes = serde_json::to_vec_pretty(&report).unwrap();
        fs::write(&report_path, report_bytes).unwrap();

        assert_eq!(report.source_count, u32::try_from(STAGEX_SOURCE_RECORD_COUNT).unwrap());
        assert_eq!(report.source_bundle_manifest_blake3, FULL_SOURCE_CLOSURE_BLAKE3);
        assert!(report.sources.iter().all(|source| source.output_path.is_dir()));
        assert!(report_path.is_file());
    }
}
