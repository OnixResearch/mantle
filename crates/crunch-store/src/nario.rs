//! Version-bound Determinate Nix Nario v2 reader.
//!
//! Metadata and record-state decisions are pure. The async shell owns wire
//! reads, bounded NAR draining or ingest, castore staging, trust lookup, and
//! atomic PathInfo publication.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use nix_compat::narinfo::Signature;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::compress_hash;
use nix_compat::wire::de::NixRead;
use nix_compat::wire::de::NixReader;
use serde::Serialize;
use sha2::Digest;
use snix_store::nar::ingest_nar_and_hash;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::Error;
use crate::export::export_castore_to_disk;
use crate::handle::StoreHandle;

pub const NARIO_V2_FORMAT_NAME: &str = "nario-v2";
pub const NARIO_V2_FORMAT_VERSION: u32 = 2;
pub const NARIO_V2_MAGIC: u64 = 0x324f_4952_414e;
pub const NARIO_V2_PRODUCER_VERSION: &str = "Determinate Nix 3.12.0";
pub const NARIO_V2_PRODUCER_REVISION: &str = "9512828397f684d0f732ea76b7631f69a0db34f7";
pub const NARIO_V2_STORE_PREFIX: &str = "/nix/store";
pub const NARIO_V2_DIRECTION: &str = "list-and-import-only";
pub const NARIO_V2_NON_CLAIM: &str = "store-data-only; no derivation graph completeness, package recipes, package selection meaning, Nix source translation, evaluator parity, package correctness, reproducibility, output trust, or release eligibility";
pub const NARIO_V2_TRUSTED_KEYS_MAX: usize = MAX_SIGNATURES;
pub const NARIO_V2_UNSUPPORTED_METADATA: &[&str] = &[
    "non-v16-worker-metadata",
    "non-sha256-nar-hash",
    "unknown-content-address",
];

const RECORD_MARKER: u64 = 1;
const END_MARKER: u64 = 0;
const HASH_HEX_LENGTH: usize = 64;
const MAX_RECORDS: usize = 100_000;
const MAX_REFERENCES: usize = 16_384;
const MAX_SIGNATURES: usize = 4_096;
const MAX_METADATA_STRING_BYTES: usize = 1_048_576;
const MAX_NAR_BYTES: u64 = 68_719_476_736;
const MAX_TOTAL_NAR_BYTES: u64 = 1_099_511_627_776;
const IO_BUFFER_BYTES: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarioV2Limits {
    pub records_max: usize,
    pub references_max: usize,
    pub signatures_max: usize,
    pub metadata_string_bytes_max: usize,
    pub nar_bytes_max: u64,
    pub total_nar_bytes_max: u64,
}

impl Default for NarioV2Limits {
    fn default() -> Self {
        Self {
            records_max: MAX_RECORDS,
            references_max: MAX_REFERENCES,
            signatures_max: MAX_SIGNATURES,
            metadata_string_bytes_max: MAX_METADATA_STRING_BYTES,
            nar_bytes_max: MAX_NAR_BYTES,
            total_nar_bytes_max: MAX_TOTAL_NAR_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NarioMetadata {
    store_path: StorePath<String>,
    deriver: Option<StorePath<String>>,
    nar_sha256: [u8; 32],
    references: Vec<StorePath<String>>,
    registration_time: u64,
    nar_size: u64,
    ultimate: bool,
    signatures: Vec<Signature<String>>,
    ca: Option<CAHash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NarioV2ListedPath {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256_hex: String,
    pub references: Vec<String>,
    pub signatures: Vec<String>,
    pub deriver: Option<String>,
    pub ca: Option<String>,
    pub registration_time: u64,
    pub ultimate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NarioV2ListReport {
    pub format: &'static str,
    pub format_version: u32,
    pub producer_version: &'static str,
    pub producer_revision: &'static str,
    pub supported_direction: &'static str,
    pub store_prefix: &'static str,
    pub archive_blake3: String,
    pub record_count: u32,
    pub total_nar_bytes: u64,
    pub unsupported_metadata_classes: &'static [&'static str],
    pub non_claim: &'static str,
    pub paths: Vec<NarioV2ListedPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NarioV2ImportReport {
    pub format: &'static str,
    pub format_version: u32,
    pub producer_version: &'static str,
    pub producer_revision: &'static str,
    pub supported_direction: &'static str,
    pub store_prefix: &'static str,
    pub archive_blake3: String,
    pub imported_count: u32,
    pub skipped_already_present_count: u32,
    pub total_nar_bytes: u64,
    pub unsupported_metadata_classes: &'static [&'static str],
    pub non_claim: &'static str,
    pub paths: Vec<NarioV2ListedPath>,
}

#[derive(Debug, Clone)]
pub struct NarioV2ImportOptions {
    pub trust_unsigned: bool,
    pub trusted_public_keys: Vec<VerifyingKey>,
    pub materialize: bool,
}

#[derive(Debug)]
struct StagedRecord {
    listed: NarioV2ListedPath,
    path_info: Option<PathInfo>,
    present: bool,
}

struct StagedMaterialization {
    staged_path: PathBuf,
    destination_path: PathBuf,
}

async fn stage_materializations(
    handle: &StoreHandle,
    path_infos: &[PathInfo],
) -> Result<(Option<tempfile::TempDir>, Vec<StagedMaterialization>), Error> {
    assert!(path_infos.len() <= NarioV2Limits::default().records_max);
    assert_eq!(handle.store_dir(), NARIO_V2_STORE_PREFIX);
    if path_infos.is_empty() {
        return Ok((None, Vec::new()));
    }
    let output_dir = Path::new(handle.output_dir_str());
    fs::create_dir_all(output_dir).map_err(|error| nario_error(format!("nario-v2-materialization-root: {error}")))?;
    let staging_dir = tempfile::Builder::new()
        .prefix(".mantle-nario-v2-")
        .tempdir_in(output_dir)
        .map_err(|error| nario_error(format!("nario-v2-materialization-staging: {error}")))?;
    let mut materializations = Vec::with_capacity(path_infos.len());
    for path_info in path_infos {
        let basename = path_info.store_path.to_string();
        let destination_path = output_dir.join(&basename);
        if destination_path.exists() {
            return Err(nario_error(format!("nario-v2-materialization-conflict: {}", destination_path.display())));
        }
        let staged_path = staging_dir.path().join(&basename);
        export_castore_to_disk(
            &path_info.node,
            &staged_path.to_string_lossy(),
            &handle.blob_service(),
            &handle.directory_service(),
        )
        .await
        .map_err(|error| nario_error(format!("nario-v2-materialization-stage for {basename}: {error}")))?;
        materializations.push(StagedMaterialization {
            staged_path,
            destination_path,
        });
    }
    Ok((Some(staging_dir), materializations))
}

async fn validate_reference_closure(handle: &StoreHandle, staged: &[StagedRecord]) -> Result<(), Error> {
    assert!(staged.len() <= NarioV2Limits::default().records_max);
    assert_eq!(handle.store_dir(), NARIO_V2_STORE_PREFIX);
    let archive_paths = staged.iter().map(|record| record.listed.store_path.clone()).collect::<BTreeSet<_>>();
    for record in staged {
        for reference in &record.listed.references {
            if archive_paths.contains(reference) {
                continue;
            }
            let parsed = StorePath::from_absolute_path(reference.as_bytes())
                .map_err(|error| nario_error(format!("nario-v2-reference-path {reference}: {error}")))?;
            let local = handle
                .pathinfo_service()
                .get(*parsed.digest())
                .await
                .map_err(|error| nario_error(format!("nario-v2-reference-lookup {reference}: {error}")))?;
            if local.as_ref().is_some_and(|path_info| path_info.store_path == parsed) {
                continue;
            }
            return Err(nario_error(format!(
                "nario-v2-missing-reference: {} -> {reference}",
                record.listed.store_path
            )));
        }
    }
    Ok(())
}

fn publish_materializations(materializations: &[StagedMaterialization]) -> Result<(), Error> {
    assert!(materializations.len() <= NarioV2Limits::default().records_max);
    assert!(materializations.iter().all(|item| item.staged_path.is_absolute()));
    let mut published_count = 0usize;
    for materialization in materializations {
        if let Err(error) = fs::rename(&materialization.staged_path, &materialization.destination_path) {
            if let Err(rollback_error) = rollback_materializations(&materializations[..published_count]) {
                return Err(nario_error(format!(
                    "nario-v2-materialization-publication for {}: {error}; {rollback_error}",
                    materialization.destination_path.display()
                )));
            }
            return Err(nario_error(format!(
                "nario-v2-materialization-publication for {}: {error}",
                materialization.destination_path.display()
            )));
        }
        published_count = published_count.saturating_add(1);
    }
    Ok(())
}

fn rollback_materializations(materializations: &[StagedMaterialization]) -> Result<(), Error> {
    assert!(materializations.len() <= NarioV2Limits::default().records_max);
    assert!(materializations.iter().all(|item| item.destination_path.is_absolute()));
    let mut first_failure = None;
    for materialization in materializations.iter().rev() {
        let metadata = match fs::symlink_metadata(&materialization.destination_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                first_failure.get_or_insert_with(|| format!("{}: {error}", materialization.destination_path.display()));
                continue;
            }
        };
        let removal = if metadata.is_dir() {
            fs::remove_dir_all(&materialization.destination_path)
        } else {
            fs::remove_file(&materialization.destination_path)
        };
        if let Err(error) = removal {
            first_failure.get_or_insert_with(|| format!("{}: {error}", materialization.destination_path.display()));
        }
    }
    match first_failure {
        Some(failure) => Err(nario_error(format!("nario-v2-rollback-failed: {failure}"))),
        None => Ok(()),
    }
}

pub fn validate_nario_record_state(
    marker: u64,
    record_index: usize,
    seen_paths: &BTreeSet<String>,
    path: Option<&str>,
    limits: &NarioV2Limits,
) -> Result<bool, String> {
    assert!(limits.records_max > 0, "Nario record limit must be positive");
    assert!(limits.metadata_string_bytes_max > 0, "Nario metadata limit must be positive");
    match marker {
        END_MARKER => Ok(false),
        RECORD_MARKER => {
            if record_index >= limits.records_max {
                return Err(format!("nario-v2-record-limit: record count exceeds {}", limits.records_max));
            }
            let path = path.ok_or_else(|| "nario-v2-missing-path: record marker has no path metadata".to_string())?;
            if seen_paths.contains(path) {
                return Err(format!("nario-v2-duplicate-path: {path}"));
            }
            Ok(true)
        }
        other => Err(format!("nario-v2-invalid-record-marker: expected 0 or 1, got {other}")),
    }
}

fn validate_metadata(metadata: &NarioMetadata, limits: &NarioV2Limits) -> Result<(), String> {
    assert!(limits.nar_bytes_max > 0, "Nario NAR byte limit must be positive");
    assert!(limits.total_nar_bytes_max >= limits.nar_bytes_max, "Nario total limit must cover one NAR");
    if metadata.nar_size == 0 {
        return Err("nario-v2-empty-nar: NAR size must be positive".to_string());
    }
    if metadata.nar_size > limits.nar_bytes_max {
        return Err(format!("nario-v2-nar-limit: {} exceeds {}", metadata.nar_size, limits.nar_bytes_max));
    }
    if metadata.references.len() > limits.references_max {
        return Err(format!(
            "nario-v2-reference-limit: {} exceeds {}",
            metadata.references.len(),
            limits.references_max
        ));
    }
    if metadata.signatures.len() > limits.signatures_max {
        return Err(format!(
            "nario-v2-signature-limit: {} exceeds {}",
            metadata.signatures.len(),
            limits.signatures_max
        ));
    }
    let mut references = BTreeSet::new();
    for reference in &metadata.references {
        if !references.insert(reference.to_string()) {
            return Err(format!("nario-v2-duplicate-reference: {reference}"));
        }
    }
    Ok(())
}

fn checked_total(current: u64, next: u64, limits: &NarioV2Limits) -> Result<u64, Error> {
    let total = current.checked_add(next).ok_or_else(|| nario_error("nario-v2-total-size-overflow"))?;
    if total > limits.total_nar_bytes_max {
        return Err(nario_error(format!("nario-v2-total-limit: {total} exceeds {}", limits.total_nar_bytes_max)));
    }
    Ok(total)
}

pub async fn list_nario_v2<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<NarioV2ListReport, Error> {
    let limits = NarioV2Limits::default();
    let mut hashing_reader = ArchiveHashReader::new(reader);
    let mut wire = nix_compat::wire::de::NixReader::builder()
        .set_max_buf_size(limits.metadata_string_bytes_max)
        .set_reserved_buf_size(IO_BUFFER_BYTES)
        .build(&mut hashing_reader);
    require_magic(&mut wire).await?;
    let mut seen_paths = BTreeSet::new();
    let mut paths = Vec::new();
    let mut total_nar_bytes = 0u64;
    for record_index in 0..=limits.records_max {
        let marker = wire.read_number().await.map_err(read_error("nario-v2-record-marker"))?;
        if marker == END_MARKER {
            require_eof(&mut wire).await?;
            drop(wire);
            let archive_blake3 = hashing_reader.finish();
            return Ok(NarioV2ListReport {
                format: NARIO_V2_FORMAT_NAME,
                format_version: NARIO_V2_FORMAT_VERSION,
                producer_version: NARIO_V2_PRODUCER_VERSION,
                producer_revision: NARIO_V2_PRODUCER_REVISION,
                supported_direction: NARIO_V2_DIRECTION,
                store_prefix: NARIO_V2_STORE_PREFIX,
                archive_blake3,
                record_count: checked_u32(paths.len(), "nario-v2 list record count")?,
                total_nar_bytes,
                unsupported_metadata_classes: NARIO_V2_UNSUPPORTED_METADATA,
                non_claim: NARIO_V2_NON_CLAIM,
                paths,
            });
        }
        let metadata = read_metadata(&mut wire, &limits).await?;
        validate_nario_record_state(marker, record_index, &seen_paths, Some(&metadata.store_path.to_string()), &limits)
            .map_err(nario_error)?;
        validate_metadata(&metadata, &limits).map_err(nario_error)?;
        require_metadata_ca_identity(&metadata)?;
        let path = metadata.store_path.to_string();
        seen_paths.insert(path);
        total_nar_bytes = checked_total(total_nar_bytes, metadata.nar_size, &limits)?;
        drain_and_verify_nar(&mut wire, &metadata).await?;
        paths.push(listed_from_metadata(&metadata));
    }
    Err(nario_error("nario-v2-record-limit"))
}

async fn finalize_nario_import(
    handle: &StoreHandle,
    options: &NarioV2ImportOptions,
    staged: Vec<StagedRecord>,
    total_nar_bytes: u64,
    archive_blake3: String,
) -> Result<NarioV2ImportReport, Error> {
    assert!(staged.len() <= NarioV2Limits::default().records_max);
    assert_eq!(handle.store_dir(), NARIO_V2_STORE_PREFIX);
    validate_reference_closure(handle, &staged).await?;
    let path_infos = staged.iter().filter_map(|record| record.path_info.clone()).collect::<Vec<_>>();
    let (materialization_dir, materializations) = if options.materialize {
        stage_materializations(handle, &path_infos).await?
    } else {
        (None, Vec::new())
    };
    publish_materializations(&materializations)?;
    if let Err(error) = handle.pathinfo_service().put_batch_atomic(path_infos.clone()).await {
        if let Err(rollback_error) = rollback_materializations(&materializations) {
            return Err(nario_error(format!("nario-v2-atomic-publication: {error}; {rollback_error}")));
        }
        return Err(nario_error(format!("nario-v2-atomic-publication: {error}")));
    }
    drop(materialization_dir);
    let paths = staged.iter().map(|record| record.listed.clone()).collect::<Vec<_>>();
    let imported_count = checked_u32(path_infos.len(), "nario-v2 imported count")?;
    let skipped = staged.iter().filter(|record| record.present).count();
    Ok(NarioV2ImportReport {
        format: NARIO_V2_FORMAT_NAME,
        format_version: NARIO_V2_FORMAT_VERSION,
        producer_version: NARIO_V2_PRODUCER_VERSION,
        producer_revision: NARIO_V2_PRODUCER_REVISION,
        supported_direction: NARIO_V2_DIRECTION,
        store_prefix: NARIO_V2_STORE_PREFIX,
        archive_blake3,
        imported_count,
        skipped_already_present_count: checked_u32(skipped, "nario-v2 skipped count")?,
        total_nar_bytes,
        unsupported_metadata_classes: NARIO_V2_UNSUPPORTED_METADATA,
        non_claim: NARIO_V2_NON_CLAIM,
        paths,
    })
}

pub async fn import_nario_v2<R: AsyncRead + Unpin + Send>(
    handle: &StoreHandle,
    reader: &mut R,
    options: &NarioV2ImportOptions,
) -> Result<NarioV2ImportReport, Error> {
    if handle.store_dir() != NARIO_V2_STORE_PREFIX {
        return Err(nario_error(format!(
            "nario-v2-store-prefix-mismatch: direct import requires {NARIO_V2_STORE_PREFIX}, configured {}",
            handle.store_dir()
        )));
    }
    let limits = NarioV2Limits::default();
    let mut hashing_reader = ArchiveHashReader::new(reader);
    let mut wire = NixReader::builder()
        .set_max_buf_size(limits.metadata_string_bytes_max)
        .set_reserved_buf_size(IO_BUFFER_BYTES)
        .build(&mut hashing_reader);
    require_magic(&mut wire).await?;
    let mut staged = Vec::new();
    let mut seen_paths = BTreeSet::new();
    let mut total_nar_bytes = 0u64;
    for record_index in 0..=limits.records_max {
        let marker = wire.read_number().await.map_err(read_error("nario-v2-record-marker"))?;
        if marker == END_MARKER {
            require_eof(&mut wire).await?;
            drop(wire);
            let archive_blake3 = hashing_reader.finish();
            return finalize_nario_import(handle, options, staged, total_nar_bytes, archive_blake3).await;
        }
        let metadata = read_metadata(&mut wire, &limits).await?;
        validate_nario_record_state(marker, record_index, &seen_paths, Some(&metadata.store_path.to_string()), &limits)
            .map_err(nario_error)?;
        validate_metadata(&metadata, &limits).map_err(nario_error)?;
        require_metadata_ca_identity(&metadata)?;
        require_trusted(&metadata, options)?;
        let path = metadata.store_path.to_string();
        seen_paths.insert(path);
        total_nar_bytes = checked_total(total_nar_bytes, metadata.nar_size, &limits)?;
        let local = handle
            .pathinfo_service()
            .get(*metadata.store_path.digest())
            .await
            .map_err(|error| nario_error(format!("nario-v2-local-pathinfo: {error}")))?;
        let listed = listed_from_metadata(&metadata);
        if let Some(local) = local {
            validate_existing_metadata(&metadata, &local)?;
            drain_and_verify_nar(&mut wire, &metadata).await?;
            staged.push(StagedRecord {
                listed,
                path_info: None,
                present: true,
            });
            continue;
        }
        let expected_ca_content_hash = None;
        let mut limited = (&mut wire).take(metadata.nar_size);
        let (node, actual_hash, actual_size) = ingest_nar_and_hash(
            handle.blob_service(),
            handle.directory_service(),
            &mut limited,
            &expected_ca_content_hash,
        )
        .await
        .map_err(|error| nario_error(format!("nario-v2-nar-ingest for {}: {error}", metadata.store_path)))?;
        if actual_size != metadata.nar_size || limited.limit() != 0 {
            return Err(nario_error(format!("nario-v2-nar-size-mismatch for {}", metadata.store_path)));
        }
        if actual_hash != metadata.nar_sha256 {
            return Err(nario_error(format!("nario-v2-nar-hash-mismatch for {}", metadata.store_path)));
        }
        let path_info = PathInfo {
            store_path: metadata.store_path.clone(),
            node,
            references: metadata.references.clone(),
            nar_size: metadata.nar_size,
            nar_sha256: metadata.nar_sha256,
            signatures: metadata.signatures.clone(),
            deriver: metadata.deriver.clone(),
            ca: metadata.ca.clone(),
        };
        require_standard_nix_ca_identity(&path_info)?;
        staged.push(StagedRecord {
            listed,
            path_info: Some(path_info),
            present: false,
        });
    }
    Err(nario_error("nario-v2-record-limit"))
}

async fn require_magic<R: AsyncRead + Unpin + Send>(reader: &mut NixReader<R>) -> Result<(), Error> {
    let magic = reader.read_number().await.map_err(read_error("nario-v2-magic"))?;
    if magic != NARIO_V2_MAGIC {
        return Err(nario_error(format!("nario-v2-wrong-magic: expected {NARIO_V2_MAGIC:#x}, got {magic:#x}")));
    }
    Ok(())
}

async fn read_metadata<R: AsyncRead + Unpin + Send>(
    reader: &mut NixReader<R>,
    limits: &NarioV2Limits,
) -> Result<NarioMetadata, Error> {
    let store_path = parse_store_path(read_string(reader, "path").await?, "path")?;
    let deriver_text = read_string(reader, "deriver").await?;
    let deriver = if deriver_text.is_empty() {
        None
    } else {
        Some(parse_store_path(deriver_text, "deriver")?)
    };
    let nar_hash_text = read_string(reader, "nar-hash").await?;
    let nar_sha256 = parse_sha256(&nar_hash_text)?;
    let reference_count = read_count(reader, "references", limits.references_max).await?;
    let mut references = Vec::with_capacity(reference_count);
    for _ in 0..reference_count {
        references.push(parse_store_path(read_string(reader, "reference").await?, "reference")?);
    }
    let registration_time = reader.read_number().await.map_err(read_error("nario-v2-registration-time"))?;
    let nar_size = reader.read_number().await.map_err(read_error("nario-v2-nar-size"))?;
    let ultimate_raw = reader.read_number().await.map_err(read_error("nario-v2-ultimate"))?;
    let ultimate = match ultimate_raw {
        0 => false,
        1 => true,
        other => return Err(nario_error(format!("nario-v2-invalid-boolean: ultimate={other}"))),
    };
    let signature_count = read_count(reader, "signatures", limits.signatures_max).await?;
    let mut signatures = Vec::with_capacity(signature_count);
    for _ in 0..signature_count {
        let signature_text = read_string(reader, "signature").await?;
        signatures.push(
            Signature::<String>::parse(&signature_text)
                .map_err(|error| nario_error(format!("nario-v2-invalid-signature: {error}")))?,
        );
    }
    let ca_text = read_string(reader, "content-address").await?;
    let ca = if ca_text.is_empty() {
        None
    } else {
        Some(CAHash::from_nix_hex_str(&ca_text).ok_or_else(|| nario_error("nario-v2-unsupported-content-address"))?)
    };
    let metadata = NarioMetadata {
        store_path,
        deriver,
        nar_sha256,
        references,
        registration_time,
        nar_size,
        ultimate,
        signatures,
        ca,
    };
    validate_metadata(&metadata, limits).map_err(nario_error)?;
    Ok(metadata)
}

async fn read_string<R: AsyncRead + Unpin + Send>(reader: &mut NixReader<R>, field: &str) -> Result<String, Error> {
    reader.read_value::<String>().await.map_err(read_error(&format!("nario-v2-{field}")))
}

async fn read_count<R: AsyncRead + Unpin + Send>(
    reader: &mut NixReader<R>,
    field: &str,
    maximum: usize,
) -> Result<usize, Error> {
    let raw = reader.read_number().await.map_err(read_error("nario-v2-collection-count"))?;
    let count = usize::try_from(raw).map_err(|_| nario_error(format!("nario-v2-{field}-limit")))?;
    if count > maximum {
        return Err(nario_error(format!("nario-v2-{field}-limit: {count} exceeds {maximum}")));
    }
    Ok(count)
}

fn parse_store_path(text: String, field: &str) -> Result<StorePath<String>, Error> {
    StorePath::from_str(&text).map_err(|error| nario_error(format!("nario-v2-invalid-{field}: {error}")))
}

fn parse_sha256(text: &str) -> Result<[u8; 32], Error> {
    if text.starts_with("sha256:") {
        return Err(nario_error("nario-v2-unsupported-nar-hash-encoding"));
    }
    let digest_text = text;
    if digest_text.len() != HASH_HEX_LENGTH {
        return Err(nario_error("nario-v2-invalid-nar-hash-length"));
    }
    let bytes = data_encoding::HEXLOWER
        .decode(digest_text.as_bytes())
        .map_err(|error| nario_error(format!("nario-v2-invalid-nar-hash: {error}")))?;
    bytes.try_into().map_err(|_| nario_error("nario-v2-invalid-nar-hash-length"))
}

fn require_trusted(metadata: &NarioMetadata, options: &NarioV2ImportOptions) -> Result<(), Error> {
    if options.trust_unsigned {
        return Ok(());
    }
    if metadata.signatures.is_empty() {
        return Err(nario_error(format!("nario-v2-untrusted-signature: {} is unsigned", metadata.store_path)));
    }
    let refs = metadata.references.iter().map(StorePath::as_ref).collect::<Vec<_>>();
    let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
        &metadata.store_path.as_ref(),
        &metadata.nar_sha256,
        metadata.nar_size,
        refs.iter(),
        NARIO_V2_STORE_PREFIX,
    );
    let trusted = metadata.signatures.iter().any(|signature| {
        options
            .trusted_public_keys
            .iter()
            .any(|key| key.name() == signature.name() && key.verify(&fingerprint, &signature.as_ref()))
    });
    if !trusted {
        return Err(nario_error(format!("nario-v2-untrusted-signature: {}", metadata.store_path)));
    }
    Ok(())
}

fn require_metadata_ca_identity(metadata: &NarioMetadata) -> Result<(), Error> {
    let Some(ca) = metadata.ca.as_ref() else {
        return Ok(());
    };
    let references = metadata.references.iter().map(ToString::to_string).collect::<Vec<_>>();
    let candidate = standard_nix_ca_path(metadata.store_path.name(), ca, &references)?;
    if candidate != metadata.store_path {
        return Err(nario_error(format!("nario-v2-content-address-identity-mismatch: {}", metadata.store_path)));
    }
    Ok(())
}

fn require_standard_nix_ca_identity(path_info: &PathInfo) -> Result<(), Error> {
    let metadata = NarioMetadata {
        store_path: path_info.store_path.clone(),
        deriver: path_info.deriver.clone(),
        nar_sha256: path_info.nar_sha256,
        references: path_info.references.clone(),
        registration_time: 0,
        nar_size: path_info.nar_size,
        ultimate: false,
        signatures: path_info.signatures.clone(),
        ca: path_info.ca.clone(),
    };
    require_metadata_ca_identity(&metadata)
}

fn standard_nix_ca_path(name: &str, ca: &CAHash, references: &[String]) -> Result<StorePath<String>, Error> {
    let (ty, inner_digest) = match ca {
        CAHash::Text(digest) => (reference_type("text", references), *digest),
        CAHash::Nar(NixHash::Sha256(digest)) => (reference_type("source", references), *digest),
        CAHash::Nar(hash) => {
            if !references.is_empty() || matches!(hash, NixHash::Blake3(_)) {
                return Err(nario_error("nario-v2-unsupported-content-address"));
            }
            ("output:out".to_string(), fixed_output_digest("fixed:out:r", hash))
        }
        CAHash::Flat(hash) => {
            if !references.is_empty() || matches!(hash, NixHash::Blake3(_)) {
                return Err(nario_error("nario-v2-unsupported-content-address"));
            }
            ("output:out".to_string(), fixed_output_digest("fixed:out", hash))
        }
    };
    let fingerprint =
        format!("{ty}:sha256:{}:{NARIO_V2_STORE_PREFIX}:{name}", data_encoding::HEXLOWER.encode(&inner_digest));
    let digest: [u8; 32] = sha2::Sha256::digest(fingerprint.as_bytes()).into();
    StorePath::from_name_and_digest_fixed(name, compress_hash(&digest))
        .map_err(|error| nario_error(format!("nario-v2-content-address-path: {error}")))
}

fn fixed_output_digest(prefix: &str, hash: &NixHash) -> [u8; 32] {
    let fingerprint = format!("{prefix}:{}:", hash.to_nix_lowerhex_string());
    sha2::Sha256::digest(fingerprint.as_bytes()).into()
}

fn reference_type(prefix: &str, references: &[String]) -> String {
    let mut value = prefix.to_string();
    for reference in references {
        value.push(':');
        value.push_str(reference);
    }
    value
}

fn validate_existing_metadata(metadata: &NarioMetadata, local: &PathInfo) -> Result<(), Error> {
    if local.store_path != metadata.store_path
        || local.references != metadata.references
        || local.nar_size != metadata.nar_size
        || local.nar_sha256 != metadata.nar_sha256
        || local.signatures != metadata.signatures
        || local.deriver != metadata.deriver
        || local.ca != metadata.ca
    {
        return Err(nario_error(format!("nario-v2-existing-path-conflict: {}", metadata.store_path)));
    }
    Ok(())
}

async fn drain_and_verify_nar<R: AsyncRead + Unpin>(reader: &mut R, metadata: &NarioMetadata) -> Result<(), Error> {
    let mut limited = reader.take(metadata.nar_size);
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0u8; IO_BUFFER_BYTES];
    let mut read_total = 0u64;
    loop {
        let count = limited.read(&mut buffer).await.map_err(read_error("nario-v2-nar-read"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        read_total = read_total.saturating_add(count as u64);
    }
    if read_total != metadata.nar_size || limited.limit() != 0 {
        return Err(nario_error(format!("nario-v2-truncated-nar: {}", metadata.store_path)));
    }
    let actual: [u8; 32] = hasher.finalize().into();
    if actual != metadata.nar_sha256 {
        return Err(nario_error(format!("nario-v2-nar-hash-mismatch: {}", metadata.store_path)));
    }
    Ok(())
}

async fn require_eof<R: AsyncRead + Unpin>(reader: &mut R) -> Result<(), Error> {
    let mut trailing = [0u8; 1];
    let count = reader.read(&mut trailing).await.map_err(read_error("nario-v2-trailing-byte"))?;
    if count != 0 {
        return Err(nario_error("nario-v2-trailing-data"));
    }
    Ok(())
}

fn listed_from_metadata(metadata: &NarioMetadata) -> NarioV2ListedPath {
    NarioV2ListedPath {
        store_path: format!("{NARIO_V2_STORE_PREFIX}/{}", metadata.store_path),
        nar_size: metadata.nar_size,
        nar_sha256_hex: data_encoding::HEXLOWER.encode(&metadata.nar_sha256),
        references: metadata.references.iter().map(|path| format!("{NARIO_V2_STORE_PREFIX}/{path}")).collect(),
        signatures: metadata.signatures.iter().map(ToString::to_string).collect(),
        deriver: metadata.deriver.as_ref().map(|path| format!("{NARIO_V2_STORE_PREFIX}/{path}")),
        ca: metadata.ca.as_ref().map(CAHash::to_nix_nixbase32_string),
        registration_time: metadata.registration_time,
        ultimate: metadata.ultimate,
    }
}

fn checked_u32(value: usize, field: &str) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| nario_error(format!("{field} exceeds u32")))
}

fn read_error(context: &str) -> impl FnOnce(std::io::Error) -> Error {
    let context = context.to_string();
    move |error| nario_error(format!("{context}: {error}"))
}

fn nario_error(message: impl Into<String>) -> Error {
    Error::Store(message.into())
}

struct ArchiveHashReader<R> {
    inner: R,
    hasher: blake3::Hasher,
}

impl<R> ArchiveHashReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
        }
    }

    fn finish(self) -> String {
        blake3::Hasher::finalize(&self.hasher).to_hex().to_string()
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ArchiveHashReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let before = buffer.filled().len();
        let poll = std::pin::Pin::new(&mut self.inner).poll_read(context, buffer);
        if let std::task::Poll::Ready(Ok(())) = &poll {
            let after = buffer.filled().len();
            self.hasher.update(&buffer.filled()[before..after]);
        }
        poll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STORE_PATH_DIGEST_BYTES: usize = 20;
    const SHA256_DIGEST_BYTES: usize = 32;

    #[test]
    fn state_core_accepts_record_and_end() {
        let limits = NarioV2Limits::default();
        let seen = BTreeSet::new();
        assert!(validate_nario_record_state(RECORD_MARKER, 0, &seen, Some("abc-path"), &limits).unwrap());
        assert!(!validate_nario_record_state(END_MARKER, 1, &seen, None, &limits).unwrap());
    }

    #[test]
    fn state_core_rejects_marker_duplicate_and_limit() {
        let mut limits = NarioV2Limits::default();
        limits.records_max = 1;
        let seen = BTreeSet::from(["abc-path".to_string()]);
        assert!(
            validate_nario_record_state(2, 0, &seen, None, &limits)
                .unwrap_err()
                .contains("invalid-record-marker")
        );
        assert!(
            validate_nario_record_state(RECORD_MARKER, 0, &seen, Some("abc-path"), &limits)
                .unwrap_err()
                .contains("duplicate-path")
        );
        assert!(
            validate_nario_record_state(RECORD_MARKER, 1, &BTreeSet::new(), Some("def-path"), &limits)
                .unwrap_err()
                .contains("record-limit")
        );
    }

    #[test]
    fn sha256_parser_accepts_pinned_raw_hex_and_rejects_other_encodings() {
        let text = "00".repeat(SHA256_DIGEST_BYTES);
        assert_eq!(parse_sha256(&text).unwrap(), [0u8; SHA256_DIGEST_BYTES]);
        assert!(parse_sha256("sha256:00").unwrap_err().to_string().contains("unsupported-nar-hash-encoding"));
        assert!(
            parse_sha256(&"gg".repeat(SHA256_DIGEST_BYTES))
                .unwrap_err()
                .to_string()
                .contains("invalid-nar-hash")
        );
    }

    #[test]
    fn ca_identity_core_accepts_standard_path_and_rejects_mismatch() {
        let ca = CAHash::Nar(NixHash::Sha256([1u8; SHA256_DIGEST_BYTES]));
        let store_path = standard_nix_ca_path("nario-ca", &ca, &[]).unwrap();
        let mut metadata = NarioMetadata {
            store_path,
            deriver: None,
            nar_sha256: [1u8; SHA256_DIGEST_BYTES],
            references: Vec::new(),
            registration_time: 0,
            nar_size: 1,
            ultimate: false,
            signatures: Vec::new(),
            ca: Some(ca),
        };

        require_metadata_ca_identity(&metadata).unwrap();
        metadata.store_path =
            StorePath::from_name_and_digest_fixed("nario-ca", [1u8; STORE_PATH_DIGEST_BYTES]).unwrap();
        assert!(
            require_metadata_ca_identity(&metadata)
                .unwrap_err()
                .to_string()
                .contains("content-address-identity-mismatch")
        );
    }

    #[test]
    fn trust_core_accepts_matching_key_and_rejects_wrong_key() {
        const KEYPAIR: &str =
            "nario-test-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
        let (signing_key, verifying_key) = nix_compat::narinfo::parse_keypair(KEYPAIR).unwrap();
        let mut metadata = NarioMetadata {
            store_path: StorePath::from_name_and_digest_fixed("nario-trust", [1u8; STORE_PATH_DIGEST_BYTES]).unwrap(),
            deriver: None,
            nar_sha256: [1u8; SHA256_DIGEST_BYTES],
            references: Vec::new(),
            registration_time: 0,
            nar_size: 1,
            ultimate: false,
            signatures: Vec::new(),
            ca: None,
        };
        let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
            &metadata.store_path.as_ref(),
            &metadata.nar_sha256,
            metadata.nar_size,
            std::iter::empty::<&nix_compat::store_path::StorePathRef>(),
            NARIO_V2_STORE_PREFIX,
        );
        metadata.signatures.push(signing_key.sign(fingerprint.as_bytes()).to_owned());
        let trusted = NarioV2ImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![verifying_key],
            materialize: false,
        };
        let untrusted = NarioV2ImportOptions {
            trust_unsigned: false,
            trusted_public_keys: Vec::new(),
            materialize: false,
        };

        require_trusted(&metadata, &trusted).unwrap();
        assert!(require_trusted(&metadata, &untrusted).unwrap_err().to_string().contains("untrusted-signature"));
    }

    #[tokio::test]
    async fn reference_closure_accepts_archive_member_and_rejects_missing_member() {
        const ROOT_PATH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-root";
        const REFERENCE_PATH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-reference";
        let temp = tempfile::tempdir().unwrap();
        let handle = StoreHandle::open(crate::StoreConfig {
            state_dir: temp.path().join("state"),
            output_dir: temp.path().join("store"),
            remote_cache_urls: Vec::new(),
            base_state_dirs: Vec::new(),
            fallback_mode: crate::StoreFallbackMode::Strict,
            store_dir: NARIO_V2_STORE_PREFIX.to_string(),
        })
        .await
        .unwrap();
        let root = StagedRecord {
            listed: listed_fixture(ROOT_PATH, vec![format!("{NARIO_V2_STORE_PREFIX}/{REFERENCE_PATH}")]),
            path_info: None,
            present: true,
        };
        let reference = StagedRecord {
            listed: listed_fixture(REFERENCE_PATH, Vec::new()),
            path_info: None,
            present: true,
        };

        validate_reference_closure(&handle, &[root, reference]).await.unwrap();
        let missing = StagedRecord {
            listed: listed_fixture(ROOT_PATH, vec![format!("{NARIO_V2_STORE_PREFIX}/{REFERENCE_PATH}")]),
            path_info: None,
            present: true,
        };
        let error = validate_reference_closure(&handle, &[missing]).await.unwrap_err();
        assert!(error.to_string().contains("missing-reference"));
    }

    fn listed_fixture(store_path: &str, references: Vec<String>) -> NarioV2ListedPath {
        NarioV2ListedPath {
            store_path: format!("{NARIO_V2_STORE_PREFIX}/{store_path}"),
            nar_size: 1,
            nar_sha256_hex: "00".repeat(SHA256_DIGEST_BYTES),
            references,
            signatures: Vec::new(),
            deriver: None,
            ca: None,
            registration_time: 0,
            ultimate: false,
        }
    }

    #[test]
    fn materialization_publication_moves_all_staged_paths() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join("staged");
        let destination = temp.path().join("destination");
        fs::write(&staged, b"payload").unwrap();
        let materializations = vec![StagedMaterialization {
            staged_path: staged.clone(),
            destination_path: destination.clone(),
        }];

        publish_materializations(&materializations).unwrap();

        assert!(!staged.exists());
        assert_eq!(fs::read(destination).unwrap(), b"payload");
    }

    #[test]
    fn materialization_publication_rolls_back_prior_moves_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let first_staged = temp.path().join("first-staged");
        let first_destination = temp.path().join("first-destination");
        fs::write(&first_staged, b"payload").unwrap();
        let missing_staged = temp.path().join("missing-staged");
        let second_destination = temp.path().join("second-destination");
        let materializations = vec![
            StagedMaterialization {
                staged_path: first_staged,
                destination_path: first_destination.clone(),
            },
            StagedMaterialization {
                staged_path: missing_staged,
                destination_path: second_destination.clone(),
            },
        ];

        let error = publish_materializations(&materializations).unwrap_err();

        assert!(error.to_string().contains("materialization-publication"));
        assert!(!first_destination.exists());
        assert!(!second_destination.exists());
    }
}
