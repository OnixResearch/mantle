use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const STAGEX_MES_SOURCE_COUNT: usize = 2;
const STAGEX_MES_SOURCE_FORMAT: &str = "mantle-stagex-mes-source-materialization-v1";
const STAGEX_MES_SOURCE_NON_CLAIM: &str =
    "Mes source materialization proves authenticated offline payload identity and unpacked fixed-output parity only";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MesSourceSpec {
    artifact_id: &'static str,
    record_name: &'static str,
    output_name: &'static str,
    record_content_blake3: &'static str,
}

const MES_SOURCE_SPECS: [MesSourceSpec; STAGEX_MES_SOURCE_COUNT] = [
    MesSourceSpec {
        artifact_id: "mes-0.27.1-source",
        record_name: "mes-src",
        output_name: "mes-0.27.1",
        record_content_blake3: "dd0e49258701c541d8fc097a899e4d00ae987d49951346ee22bd5743d5e81582",
    },
    MesSourceSpec {
        artifact_id: "nyacc-1.00.2-source",
        record_name: "nyacc-src",
        output_name: "nyacc",
        record_content_blake3: "3cdb842d8699276ce030e5fbbab455193d12ab41cfb36aabfc3f3bbdfa4c47db",
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesMaterializedSource {
    pub artifact_id: String,
    pub record_name: String,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub source_count: u32,
    pub sources: Vec<MesMaterializedSource>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum MesSourceError {
    Bundle(crate::RunError),
    InvalidAuthority(String),
    Materialization(String),
}

impl std::fmt::Display for MesSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "reading Mes source bundle: {error}"),
            Self::InvalidAuthority(message) => write!(formatter, "invalid Mes source authority: {message}"),
            Self::Materialization(message) => write!(formatter, "materializing Mes sources: {message}"),
        }
    }
}

impl std::error::Error for MesSourceError {}

impl From<crate::RunError> for MesSourceError {
    fn from(error: crate::RunError) -> Self {
        Self::Bundle(error)
    }
}

pub(crate) fn expected_mes_source_artifact_digests() -> [(&'static str, &'static str); STAGEX_MES_SOURCE_COUNT] {
    MES_SOURCE_SPECS.map(|spec| (spec.artifact_id, spec.record_content_blake3))
}

pub(crate) fn materialize_authenticated_mes_sources(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    source_root: &Path,
) -> Result<MesSourceMaterializationReport, MesSourceError> {
    let manifest = read_source_bundle(bundle_path)?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(MesSourceError::InvalidAuthority(format!(
            "source bundle BLAKE3 mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    materialize_mes_sources_from_manifest(&manifest, source_root)
}

fn materialize_mes_sources_from_manifest(
    manifest: &SourceBundleManifest,
    source_root: &Path,
) -> Result<MesSourceMaterializationReport, MesSourceError> {
    let records = select_mes_source_records(&manifest.records)?;
    fs::create_dir(source_root).map_err(|error| {
        MesSourceError::Materialization(format!(
            "creating create-new Mes source root {}: {error}",
            source_root.display()
        ))
    })?;
    let mut sources = Vec::with_capacity(STAGEX_MES_SOURCE_COUNT);
    for (spec, record) in MES_SOURCE_SPECS.iter().zip(records) {
        let output_path = source_root.join(spec.output_name);
        if output_path.exists() {
            return Err(MesSourceError::Materialization(format!(
                "create-new Mes source target already exists: {}",
                output_path.display()
            )));
        }
        materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
            MesSourceError::Materialization(format!(
                "materializing {} at {}: {error}",
                spec.record_name,
                output_path.display()
            ))
        })?;
        sources.push(MesMaterializedSource {
            artifact_id: spec.artifact_id.to_string(),
            record_name: spec.record_name.to_string(),
            record_identity: record.identity.clone(),
            record_content_blake3: record.content_blake3.clone(),
            output_path,
        });
    }
    assert_eq!(sources.len(), STAGEX_MES_SOURCE_COUNT);
    assert!(sources.iter().all(|source| source.output_path.starts_with(source_root)));
    Ok(MesSourceMaterializationReport {
        format: STAGEX_MES_SOURCE_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3.clone(),
        source_count: u32::try_from(sources.len())
            .map_err(|_| MesSourceError::Materialization("Mes source count does not fit u32".to_string()))?,
        sources,
        non_claim: STAGEX_MES_SOURCE_NON_CLAIM,
    })
}

fn select_mes_source_records(records: &[SourceRecord]) -> Result<Vec<&SourceRecord>, MesSourceError> {
    let mut by_name = BTreeMap::new();
    for record in records {
        let Some(name) = record.metadata.get(SOURCE_RECORD_NAME_KEY) else {
            continue;
        };
        if !MES_SOURCE_SPECS.iter().any(|spec| spec.record_name == name) {
            continue;
        }
        if by_name.insert(name.as_str(), record).is_some() {
            return Err(MesSourceError::InvalidAuthority(format!(
                "source bundle contains duplicate Mes record '{name}'"
            )));
        }
    }
    let mut selected = Vec::with_capacity(STAGEX_MES_SOURCE_COUNT);
    for spec in MES_SOURCE_SPECS {
        let record = by_name.get(spec.record_name).copied().ok_or_else(|| {
            MesSourceError::InvalidAuthority(format!("source bundle is missing Mes record '{}'", spec.record_name))
        })?;
        validate_mes_source_record(spec, record)?;
        selected.push(record);
    }
    assert_eq!(selected.len(), STAGEX_MES_SOURCE_COUNT);
    assert!(selected.iter().all(|record| record.kind == SourceRecordKind::FixedUrl));
    Ok(selected)
}

fn validate_mes_source_record(spec: MesSourceSpec, record: &SourceRecord) -> Result<(), MesSourceError> {
    if record.kind != SourceRecordKind::FixedUrl {
        return Err(MesSourceError::InvalidAuthority(format!(
            "Mes source '{}' is not a fixed URL record",
            spec.record_name
        )));
    }
    if record.files.is_empty() {
        return Err(MesSourceError::InvalidAuthority(format!(
            "Mes source '{}' has no authenticated archive payload",
            spec.record_name
        )));
    }
    if record.content_blake3 != spec.record_content_blake3 {
        return Err(MesSourceError::InvalidAuthority(format!(
            "Mes source '{}' content BLAKE3 mismatch: expected {}, observed {}",
            spec.record_name, spec.record_content_blake3, record.content_blake3
        )));
    }
    assert!(record.payload_bytes > 0);
    assert!(!record.identity.is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_bundle::SourceFileEntry;
    use crate::source_bundle::SourceFileType;

    const TEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const MES_SOURCE_SCRATCH_ENV: &str = "MANTLE_STAGE_X_MES_SOURCE_SCRATCH";

    fn record(spec: MesSourceSpec) -> SourceRecord {
        SourceRecord {
            kind: SourceRecordKind::FixedUrl,
            identity: format!("fixed-url-{}", spec.record_name),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), spec.record_name.to_string())]),
            payload_bytes: 1,
            content_blake3: spec.record_content_blake3.to_string(),
            files: vec![SourceFileEntry {
                path: "archive".to_string(),
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

    fn records() -> Vec<SourceRecord> {
        MES_SOURCE_SPECS.iter().copied().map(record).collect()
    }

    #[test]
    fn selects_exact_mes_and_nyacc_sources() {
        let mut input = records();
        input.reverse();

        let selected = select_mes_source_records(&input).unwrap();

        assert_eq!(selected.len(), STAGEX_MES_SOURCE_COUNT);
        assert_eq!(selected[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some("mes-src"));
        assert_eq!(selected[1].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some("nyacc-src"));
    }

    #[test]
    fn rejects_substituted_or_missing_mes_source() {
        let mut substituted = records();
        substituted[0].content_blake3 = TEST_BLAKE3.to_string();
        let substituted_error = select_mes_source_records(&substituted).unwrap_err();
        assert!(substituted_error.to_string().contains("content BLAKE3 mismatch"));

        let mut missing = records();
        missing.pop();
        let missing_error = select_mes_source_records(&missing).unwrap_err();
        assert!(missing_error.to_string().contains("missing Mes record 'nyacc-src'"));
    }

    #[test]
    #[ignore = "requires explicit authenticated source bundle and create-new scratch"]
    fn materializes_retained_mes_sources() {
        let bundle = PathBuf::from(std::env::var(SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(MES_SOURCE_SCRATCH_ENV).unwrap());

        let report = materialize_authenticated_mes_sources(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        let bytes = serde_json::to_vec_pretty(&report).unwrap();
        fs::write(scratch.join("mes-source-materialization.json"), bytes).unwrap();

        assert_eq!(report.source_count, u32::try_from(STAGEX_MES_SOURCE_COUNT).unwrap());
        assert!(report.sources.iter().all(|source| source.output_path.is_dir()));
        assert!(scratch.join("mes-source-materialization.json").is_file());
    }
}
