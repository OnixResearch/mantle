use std::collections::BTreeMap;

use crunch_source_core::GitObjectFormat;
use crunch_source_core::ImmutableRevisionWire;
use crunch_source_core::LegacySourceObservationFacts;
use crunch_source_core::LegacySourceProjection;
use crunch_source_core::LocatorClass;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservationWire;
use crunch_source_core::SourceRecordFact;
use crunch_source_core::SourceRecordProvenance;
use crunch_source_core::project_legacy_source_observation;

use super::FETCH_ENV_REV_KEY;
use super::FETCH_ENV_UNPACK_KEY;
use super::RECORD_METADATA_URL_KEY;
use super::SourceObservationImportSummary;
use super::SourceRecord;
use super::SourceRecordKind;
use super::digest_source_entries;
use crate::errors::RunError;

pub(super) const OBSERVATION_COMPLETE: &str = "complete";
pub(super) const OBSERVATION_PROVENANCE_UNAVAILABLE: &str = "provenance-unavailable-legacy-v1";

#[derive(Debug, Clone)]
pub(super) struct ProjectedSourceRecord {
    pub fact: SourceRecordFact,
    pub observation: Option<SourceObservationWire>,
    pub summary: SourceObservationImportSummary,
}

pub(super) fn project_source_record(record: &SourceRecord) -> Result<ProjectedSourceRecord, RunError> {
    let measured_content_blake3 = measured_content_blake3(record)?;
    let projection = project_legacy_source_observation(legacy_facts(record, measured_content_blake3.as_deref()))
        .map_err(|error| {
            RunError::Internal(format!("source observation rejected record {}: {error:?}", record.identity))
        })?;
    let (observation, disposition, observation_blake3, provenance) = match projection {
        LegacySourceProjection::Complete(observation) => {
            let identity = Some(observation.observation_blake3.clone());
            (Some(observation), OBSERVATION_COMPLETE, identity, SourceRecordProvenance::ObservedV1)
        }
        LegacySourceProjection::ProvenanceUnavailable(_) => {
            (None, OBSERVATION_PROVENANCE_UNAVAILABLE, None, SourceRecordProvenance::LegacyV1)
        }
    };
    let record_bytes = serde_json::to_vec(record)
        .map_err(|error| RunError::Internal(format!("serializing source record for ingest: {error}")))?;
    let record_bytes_blake3 = blake3::hash(&record_bytes).to_hex().to_string();
    let fact = SourceRecordFact {
        record_identity: record.identity.clone(),
        content_blake3: record.content_blake3.clone(),
        record_bytes_blake3,
        observation_blake3: observation_blake3.clone(),
        provenance,
    };
    let summary = SourceObservationImportSummary {
        record_identity: record.identity.clone(),
        disposition: disposition.to_string(),
        observation_blake3,
        content_blake3: measured_content_blake3.unwrap_or_else(|| record.content_blake3.clone()),
    };
    debug_assert_eq!(fact.record_identity, summary.record_identity);
    debug_assert!(!summary.content_blake3.is_empty());
    Ok(ProjectedSourceRecord {
        fact,
        observation,
        summary,
    })
}

fn measured_content_blake3(record: &SourceRecord) -> Result<Option<String>, RunError> {
    if record.files.is_empty() {
        return Ok(None);
    }
    digest_source_entries(&record.files).map(Some)
}

fn legacy_facts(record: &SourceRecord, measured_content_blake3: Option<&str>) -> LegacySourceObservationFacts {
    let source_kind = source_kind(record.kind.clone());
    LegacySourceObservationFacts {
        source_kind,
        locator_class: Some(locator_class(source_kind)),
        locator_hint: locator_hint(record, source_kind),
        immutable_revision: immutable_revision(record),
        normalized_projection: Some(".".to_string()),
        snapshot_profile: measured_content_blake3.and_then(|_| snapshot_profile(record)),
        content_blake3: measured_content_blake3.unwrap_or(&record.content_blake3).to_string(),
        mutable_reference_hint: mutable_reference_hint(record),
    }
}

fn source_kind(kind: SourceRecordKind) -> SourceKind {
    match kind {
        SourceRecordKind::FixedUrl | SourceRecordKind::BootstrapArchive => SourceKind::FixedUrl,
        SourceRecordKind::VcsSnapshot => SourceKind::VcsSnapshot,
        SourceRecordKind::LocalPath | SourceRecordKind::ProviderManifest | SourceRecordKind::ToolchainSourceRoot => {
            SourceKind::LocalLogical
        }
        SourceRecordKind::PackageMirror => SourceKind::PackageMirror,
        SourceRecordKind::ProofInput => SourceKind::OpaqueAdapter,
    }
}

fn locator_class(kind: SourceKind) -> LocatorClass {
    match kind {
        SourceKind::FixedUrl => LocatorClass::Url,
        SourceKind::VcsSnapshot => LocatorClass::GitRemote,
        SourceKind::LocalLogical => LocatorClass::LogicalPath,
        SourceKind::PackageMirror => LocatorClass::Mirror,
        SourceKind::OpaqueAdapter => LocatorClass::Opaque,
    }
}

fn locator_hint(record: &SourceRecord, kind: SourceKind) -> Option<String> {
    match kind {
        SourceKind::FixedUrl | SourceKind::VcsSnapshot | SourceKind::PackageMirror => {
            checked_url_hint(&record.metadata)
        }
        SourceKind::LocalLogical => Some(record.identity.clone()),
        SourceKind::OpaqueAdapter => record.adapter.as_ref().map(|adapter| adapter.adapter.clone()),
    }
}

fn checked_url_hint(metadata: &BTreeMap<String, String>) -> Option<String> {
    let value = metadata.get(RECORD_METADATA_URL_KEY)?;
    crunch_build::FetchUrl::new(value.clone()).ok().map(|url| url.as_str().to_string())
}

fn immutable_revision(record: &SourceRecord) -> Option<ImmutableRevisionWire> {
    if record.kind != SourceRecordKind::VcsSnapshot {
        return None;
    }
    let revision = record.metadata.get(FETCH_ENV_REV_KEY)?;
    let checked = crunch_build::GitRevision::new(revision.clone()).ok()?;
    let object_format = match checked.as_str().len() {
        crunch_source_core::GIT_SHA1_HEX_CHARS => GitObjectFormat::Sha1,
        crunch_source_core::GIT_SHA256_HEX_CHARS => GitObjectFormat::Sha256,
        _ => return None,
    };
    Some(ImmutableRevisionWire {
        object_format,
        value: checked.as_str().to_string(),
    })
}

fn mutable_reference_hint(record: &SourceRecord) -> Option<String> {
    if record.kind != SourceRecordKind::VcsSnapshot {
        return None;
    }
    let revision = record.metadata.get(FETCH_ENV_REV_KEY)?;
    if immutable_revision(record).is_some() {
        return None;
    }
    Some(revision.clone())
}

fn snapshot_profile(record: &SourceRecord) -> Option<SnapshotProfile> {
    match &record.kind {
        SourceRecordKind::FixedUrl => Some(fixed_url_profile(record)),
        SourceRecordKind::BootstrapArchive => Some(SnapshotProfile::ArchiveTreeV1),
        SourceRecordKind::VcsSnapshot | SourceRecordKind::ToolchainSourceRoot => Some(SnapshotProfile::CanonicalTreeV1),
        SourceRecordKind::PackageMirror => Some(package_mirror_profile(record)),
        SourceRecordKind::ProviderManifest => Some(SnapshotProfile::FlatFileV1),
        SourceRecordKind::ProofInput => Some(SnapshotProfile::OpaqueV1),
        SourceRecordKind::LocalPath => local_path_profile(record),
    }
}

fn fixed_url_profile(record: &SourceRecord) -> SnapshotProfile {
    if record.metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) == Some("1") {
        SnapshotProfile::ArchiveTreeV1
    } else {
        SnapshotProfile::FlatFileV1
    }
}

fn package_mirror_profile(record: &SourceRecord) -> SnapshotProfile {
    if record.metadata.get(super::RECORD_METADATA_PAYLOAD_ENCODING_KEY).map(String::as_str)
        == Some(super::TARBALL_ARCHIVE_PAYLOAD_ENCODING)
    {
        SnapshotProfile::ArchiveTreeV1
    } else {
        SnapshotProfile::CanonicalTreeV1
    }
}

fn local_path_profile(record: &SourceRecord) -> Option<SnapshotProfile> {
    if record.metadata.get(super::FOREIGN_PAYLOAD_SHAPE_METADATA_KEY).map(String::as_str)
        == Some(super::FOREIGN_PAYLOAD_SHAPE_FLAT_FILE)
    {
        return Some(SnapshotProfile::FlatFileV1);
    }
    let tree_shape_is_explicit = record.files.len() > 1 || record.files.iter().any(|file| file.path.contains('/'));
    tree_shape_is_explicit.then_some(SnapshotProfile::CanonicalTreeV1)
}
