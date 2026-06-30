//! Mantle-native streaming store archive transport.
//!
//! The pure layer validates frame order, deterministic export planning, prefix
//! binding, signature policy, and import action selection. The shell layer owns
//! PathInfo lookup, NAR rendering/ingest, filesystem materialization, and
//! reader/writer streaming.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use snix_store::nar::ingest_nar_and_hash;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::ReadBuf;

use crate::Error;
use crate::export::export_castore_to_disk;
use crate::handle::StoreHandle;

pub const ARCHIVE_FORMAT_NAME: &str = "mantle-store-archive-v1";
pub const ARCHIVE_VERSION: u32 = 1;
pub const ARCHIVE_COMPATIBILITY: &str = "mantle-native; nario-v2 byte compatibility unproven";
pub const MAX_ARCHIVE_RECORDS: usize = 1_000_000;
pub const MAX_ARCHIVE_METADATA_BYTES: usize = 1_048_576;
pub const ARCHIVE_IO_BUFFER_BYTES: usize = 65_536;

const ARCHIVE_MAGIC: &[u8] = b"mantle-store-archive-v1\n";
const FRAME_LEN_BYTES: usize = 4;
const TEMP_ARCHIVE_RENDER_DIR: &str = "archive-render-tmp";
const HASH_HEX_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveExportOptions {
    pub trust_unsigned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveImportOptions {
    pub trust_unsigned: bool,
    pub trusted_public_keys: Vec<VerifyingKey>,
    pub materialize: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveExportReport {
    pub exported_count: u32,
    pub total_payload_bytes: u64,
    pub paths: Vec<ArchivePathSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveImportReport {
    pub imported_count: u32,
    pub skipped_already_present_count: u32,
    pub total_payload_bytes: u64,
    pub paths: Vec<ArchivePathSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveListReport {
    pub store_prefix: String,
    pub record_count: u32,
    pub total_payload_bytes: u64,
    pub compatibility: String,
    pub paths: Vec<ArchiveListedPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchivePathSummary {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveListedPath {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256_hex: String,
    pub reference_count: u32,
    pub signature_count: u32,
    pub ca: Option<String>,
    pub root: bool,
    pub payload_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "frame", rename_all = "kebab-case")]
enum ArchiveFrame {
    Header(ArchiveHeaderFrame),
    Path(ArchivePathFrame),
    End(ArchiveEndFrame),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchiveHeaderFrame {
    format: String,
    version: u32,
    store_prefix: String,
    record_count: u32,
    roots: Vec<String>,
    compatibility: String,
    payload_len: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchivePathFrame {
    path_info: PathInfo,
    root: bool,
    payload_len: u64,
    payload_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArchiveEndFrame {
    record_count: u32,
    total_payload_bytes: u64,
    payload_len: u64,
}

impl ArchiveFrame {
    fn payload_len(&self) -> u64 {
        match self {
            Self::Header(frame) => frame.payload_len,
            Self::Path(frame) => frame.payload_len,
            Self::End(frame) => frame.payload_len,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveImportAction {
    Import,
    SkipExisting,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveImportDecision {
    pub action: ArchiveImportAction,
    pub reason: &'static str,
}

pub fn plan_archive_import_action(
    record: &ArchiveListedPath,
    local_present: bool,
    store_prefix_matches: bool,
    signatures_trusted: bool,
) -> ArchiveImportDecision {
    assert!(!record.store_path.is_empty(), "archive import record path must be present");
    assert_eq!(record.nar_sha256_hex.len(), HASH_HEX_BYTES, "nar sha256 hex must be canonical length");
    if !store_prefix_matches {
        return ArchiveImportDecision {
            action: ArchiveImportAction::Reject,
            reason: "store-prefix-mismatch",
        };
    }
    if !signatures_trusted {
        return ArchiveImportDecision {
            action: ArchiveImportAction::Reject,
            reason: "untrusted-signature",
        };
    }
    if local_present {
        return ArchiveImportDecision {
            action: ArchiveImportAction::SkipExisting,
            reason: "already-present",
        };
    }
    ArchiveImportDecision {
        action: ArchiveImportAction::Import,
        reason: "missing-locally",
    }
}

pub async fn export_store_archive<W: AsyncWrite + Unpin + Send>(
    handle: &StoreHandle,
    roots: &[PathInfo],
    writer: &mut W,
    options: &ArchiveExportOptions,
) -> Result<ArchiveExportReport, Error> {
    assert!(roots.len() <= MAX_ARCHIVE_RECORDS, "archive root count exceeds limit");
    if roots.is_empty() {
        return Err(Error::Export("archive export requires at least one root PathInfo".to_string()));
    }

    let planned = plan_export_closure(handle, roots, options).await?;
    let root_set = roots.iter().map(|pi| pi.store_path.to_string()).collect::<BTreeSet<_>>();
    let record_count = checked_record_count(planned.len())?;
    let root_labels = roots.iter().map(|pi| pi.store_path.to_string()).collect::<Vec<_>>();

    writer.write_all(ARCHIVE_MAGIC).await.map_err(write_archive_error("writing archive magic"))?;
    write_frame(
        writer,
        &ArchiveFrame::Header(ArchiveHeaderFrame {
            format: ARCHIVE_FORMAT_NAME.to_string(),
            version: ARCHIVE_VERSION,
            store_prefix: handle.store_dir().to_string(),
            record_count,
            roots: root_labels,
            compatibility: ARCHIVE_COMPATIBILITY.to_string(),
            payload_len: 0,
        }),
    )
    .await?;

    let mut report = ArchiveExportReport {
        exported_count: 0,
        total_payload_bytes: 0,
        paths: Vec::with_capacity(planned.len()),
    };

    for path_info in &planned {
        let rendered = render_payload_to_temp(handle, path_info).await?;
        write_frame(
            writer,
            &ArchiveFrame::Path(ArchivePathFrame {
                path_info: path_info.clone(),
                root: root_set.contains(&path_info.store_path.to_string()),
                payload_len: rendered.payload_len,
                payload_blake3: rendered.payload_blake3.clone(),
            }),
        )
        .await?;
        copy_payload_file(writer, &rendered.path).await?;
        let _ = tokio::fs::remove_file(&rendered.path).await;

        report.exported_count = report.exported_count.saturating_add(1);
        report.total_payload_bytes = report.total_payload_bytes.saturating_add(rendered.payload_len);
        report.paths.push(summary_for_pathinfo(path_info));
    }

    write_frame(
        writer,
        &ArchiveFrame::End(ArchiveEndFrame {
            record_count: report.exported_count,
            total_payload_bytes: report.total_payload_bytes,
            payload_len: 0,
        }),
    )
    .await?;
    writer.flush().await.map_err(write_archive_error("flushing archive"))?;

    Ok(report)
}

pub async fn list_store_archive<R: AsyncRead + Unpin>(reader: &mut R) -> Result<ArchiveListReport, Error> {
    read_magic(reader).await?;
    let header = read_header(reader).await?;
    let mut report = ArchiveListReport {
        store_prefix: header.store_prefix,
        record_count: header.record_count,
        total_payload_bytes: 0,
        compatibility: header.compatibility,
        paths: Vec::with_capacity(header.record_count as usize),
    };

    let mut seen = BTreeSet::new();
    for record_index in 0..MAX_ARCHIVE_RECORDS {
        let frame = read_frame(reader).await?;
        match frame {
            ArchiveFrame::Path(path_frame) => {
                let listed = listed_path_from_frame(&path_frame)?;
                if !seen.insert(listed.store_path.clone()) {
                    return Err(Error::Store(format!("archive contains duplicate path record {}", listed.store_path)));
                }
                drain_payload(reader, path_frame.payload_len).await?;
                report.total_payload_bytes = report.total_payload_bytes.saturating_add(path_frame.payload_len);
                report.paths.push(listed);
            }
            ArchiveFrame::End(end) => {
                validate_end_frame(&report, &end)?;
                return Ok(report);
            }
            ArchiveFrame::Header(_) => {
                return Err(Error::Store("archive header frame repeated after first frame".to_string()));
            }
        }
        if record_index + 1 >= MAX_ARCHIVE_RECORDS {
            return Err(Error::Store(format!("archive record count exceeded {MAX_ARCHIVE_RECORDS}")));
        }
    }

    Err(Error::Store("archive ended without end marker".to_string()))
}

pub async fn import_store_archive<R: AsyncRead + Unpin + Send>(
    handle: &StoreHandle,
    reader: &mut R,
    options: &ArchiveImportOptions,
) -> Result<ArchiveImportReport, Error> {
    read_magic(reader).await?;
    let header = read_header(reader).await?;
    validate_header_store_prefix(&header, handle.store_dir())?;

    let mut report = ArchiveImportReport {
        imported_count: 0,
        skipped_already_present_count: 0,
        total_payload_bytes: 0,
        paths: Vec::with_capacity(header.record_count as usize),
    };
    let mut seen = BTreeSet::new();

    for record_index in 0..MAX_ARCHIVE_RECORDS {
        let frame = read_frame(reader).await?;
        match frame {
            ArchiveFrame::Path(path_frame) => {
                let listed = listed_path_from_frame(&path_frame)?;
                if !seen.insert(listed.store_path.clone()) {
                    return Err(Error::Store(format!("archive contains duplicate path record {}", listed.store_path)));
                }
                import_or_skip_path(handle, reader, path_frame, listed, options, &mut report).await?;
            }
            ArchiveFrame::End(end) => {
                validate_import_end(&report, &end, header.record_count)?;
                return Ok(report);
            }
            ArchiveFrame::Header(_) => {
                return Err(Error::Store("archive header frame repeated after first frame".to_string()));
            }
        }
        if record_index + 1 >= MAX_ARCHIVE_RECORDS {
            return Err(Error::Store(format!("archive record count exceeded {MAX_ARCHIVE_RECORDS}")));
        }
    }

    Err(Error::Store("archive ended without end marker".to_string()))
}

async fn plan_export_closure(
    handle: &StoreHandle,
    roots: &[PathInfo],
    options: &ArchiveExportOptions,
) -> Result<Vec<PathInfo>, Error> {
    let mut selected = BTreeMap::<String, PathInfo>::new();
    let mut queue = VecDeque::<StorePath<String>>::new();
    for root in roots {
        validate_export_candidate(root, options, handle).await?;
        queue.push_back(root.store_path.clone());
        selected.insert(root.store_path.to_string(), root.clone());
    }

    while let Some(next) = queue.pop_front() {
        let key = next.to_string();
        let path_info = selected
            .get(&key)
            .cloned()
            .ok_or_else(|| Error::Export(format!("internal archive closure miss for {key}")))?;
        for reference in &path_info.references {
            let reference_key = reference.to_string();
            if selected.contains_key(&reference_key) {
                continue;
            }
            let referenced = load_pathinfo(handle.pathinfo_service().as_ref(), reference).await?;
            validate_export_candidate(&referenced, options, handle).await?;
            queue.push_back(reference.clone());
            selected.insert(reference_key, referenced);
        }
    }

    assert!(selected.len() <= MAX_ARCHIVE_RECORDS, "archive closure size exceeds limit");
    Ok(selected.into_values().collect())
}

async fn validate_export_candidate(
    path_info: &PathInfo,
    options: &ArchiveExportOptions,
    handle: &StoreHandle,
) -> Result<(), Error> {
    assert!(!path_info.store_path.name().is_empty(), "archive path name must not be empty");
    if path_info.signatures.is_empty() && !options.trust_unsigned {
        return Err(Error::Export(format!(
            "archive export refuses unsigned path {} without trust_unsigned",
            path_info.store_path
        )));
    }
    if !handle.castore_has_content(&path_info.node).await? {
        return Err(Error::Export(format!("archive export missing payload for {}", path_info.store_path)));
    }
    Ok(())
}

async fn load_pathinfo(pathinfo: &dyn PathInfoService, store_path: &StorePath<String>) -> Result<PathInfo, Error> {
    let loaded = pathinfo
        .get(*store_path.digest())
        .await
        .map_err(|err| Error::PathInfoService(format!("loading closure member {store_path}: {err}")))?;
    let Some(loaded) = loaded else {
        return Err(Error::MissingClosureFacts {
            path: store_path.clone(),
            store_dir: nix_compat::store_path::STORE_DIR.to_string(),
            detail: "PathInfo missing for archive export".to_string(),
        });
    };
    if loaded.store_path != *store_path {
        return Err(Error::PathInfoService(format!(
            "PathInfo digest collision for {store_path}: stored path was {}",
            loaded.store_path
        )));
    }
    Ok(loaded)
}

struct RenderedPayload {
    path: PathBuf,
    payload_len: u64,
    payload_blake3: String,
}

async fn render_payload_to_temp(handle: &StoreHandle, path_info: &PathInfo) -> Result<RenderedPayload, Error> {
    let temp_dir = handle.state_dir().join(TEMP_ARCHIVE_RENDER_DIR);
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|err| Error::Export(format!("creating archive temp dir {}: {err}", temp_dir.display())))?;
    let temp_path = temp_dir.join(format!("{}.nar", path_info.store_path));
    let temp_file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|err| Error::Export(format!("creating archive payload {}: {err}", temp_path.display())))?;
    let mut writer = HashingAsyncWriter::new(tokio::io::BufWriter::new(temp_file));
    handle.render_nar(&path_info.node, &mut writer).await?;
    writer.flush().await.map_err(write_archive_error("flushing archive payload"))?;
    let (payload_len, payload_blake3) = writer.finish();
    if payload_len != path_info.nar_size {
        return Err(Error::Export(format!(
            "rendered NAR size mismatch for {}: rendered {payload_len}, PathInfo {}",
            path_info.store_path, path_info.nar_size
        )));
    }
    Ok(RenderedPayload {
        path: temp_path,
        payload_len,
        payload_blake3,
    })
}

async fn copy_payload_file<W: AsyncWrite + Unpin>(writer: &mut W, payload_path: &std::path::Path) -> Result<(), Error> {
    let mut file = tokio::fs::File::open(payload_path)
        .await
        .map_err(|err| Error::Export(format!("opening archive payload {}: {err}", payload_path.display())))?;
    tokio::io::copy(&mut file, writer)
        .await
        .map_err(|err| Error::Export(format!("copying archive payload {}: {err}", payload_path.display())))?;
    Ok(())
}

async fn import_or_skip_path<R: AsyncRead + Unpin + Send>(
    handle: &StoreHandle,
    reader: &mut R,
    path_frame: ArchivePathFrame,
    listed: ArchiveListedPath,
    options: &ArchiveImportOptions,
    report: &mut ArchiveImportReport,
) -> Result<(), Error> {
    let local_state = local_archive_path_state(handle, &path_frame.path_info).await?;
    if local_state == LocalArchivePathState::ConflictingMetadata {
        return Err(Error::Store(format!("local PathInfo for {} conflicts with archive metadata", listed.store_path)));
    }
    let local_present = local_state == LocalArchivePathState::Present;
    let signatures_trusted = signatures_trusted(&path_frame.path_info, handle.store_dir(), options);
    let decision = plan_archive_import_action(&listed, local_present, true, signatures_trusted);
    match decision.action {
        ArchiveImportAction::SkipExisting => {
            drain_payload(reader, path_frame.payload_len).await?;
            report.skipped_already_present_count = report.skipped_already_present_count.saturating_add(1);
            Ok(())
        }
        ArchiveImportAction::Reject => {
            Err(Error::Store(format!("archive record {} rejected: {}", listed.store_path, decision.reason)))
        }
        ArchiveImportAction::Import => import_missing_path(handle, reader, path_frame, listed, options, report).await,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalArchivePathState {
    Missing,
    Present,
    ConflictingMetadata,
}

async fn local_archive_path_state(handle: &StoreHandle, path_info: &PathInfo) -> Result<LocalArchivePathState, Error> {
    assert!(!path_info.store_path.name().is_empty(), "archive import path name must not be empty");
    let local = handle
        .pathinfo_service()
        .get(*path_info.store_path.digest())
        .await
        .map_err(|err| Error::PathInfoService(format!("checking local archive import path: {err}")))?;
    let Some(local) = local else {
        return Ok(LocalArchivePathState::Missing);
    };
    if local.store_path != path_info.store_path {
        return Err(Error::PathInfoService(format!(
            "PathInfo digest collision for {}: stored path was {}",
            path_info.store_path, local.store_path
        )));
    }
    if local != *path_info {
        return Ok(LocalArchivePathState::ConflictingMetadata);
    }
    if handle.castore_has_content(&local.node).await? {
        return Ok(LocalArchivePathState::Present);
    }
    Ok(LocalArchivePathState::Missing)
}

async fn import_missing_path<R: AsyncRead + Unpin + Send>(
    handle: &StoreHandle,
    reader: &mut R,
    path_frame: ArchivePathFrame,
    listed: ArchiveListedPath,
    options: &ArchiveImportOptions,
    report: &mut ArchiveImportReport,
) -> Result<(), Error> {
    let blob_service = handle.blob_service();
    let directory_service = handle.directory_service();
    let mut limited_reader = Blake3AsyncReader::new(reader.take(path_frame.payload_len));
    let (node, actual_nar_sha256, actual_nar_size) = ingest_nar_and_hash(
        blob_service.clone(),
        directory_service.clone(),
        &mut limited_reader,
        &path_frame.path_info.ca,
    )
    .await
    .map_err(|err| Error::Store(format!("ingesting archive payload for {}: {err}", listed.store_path)))?;
    let (bytes_read, payload_blake3) = limited_reader.finish();
    validate_imported_payload(&path_frame, &node, actual_nar_sha256, actual_nar_size, bytes_read, &payload_blake3)?;

    let mut imported = path_frame.path_info.clone();
    imported.node = node.clone();
    handle.pathinfo_service().put(imported).await.map_err(|err| {
        Error::PathInfoService(format!("persisting archive PathInfo for {}: {err}", listed.store_path))
    })?;

    if options.materialize {
        let abs_path = path_frame.path_info.store_path.to_absolute_path_with_prefix(handle.output_dir_str());
        if !std::path::Path::new(&abs_path).exists() {
            export_castore_to_disk(&node, &abs_path, &blob_service, &directory_service)
                .await
                .map_err(|err| Error::Export(format!("materializing archive path {abs_path}: {err}")))?;
        }
    }

    report.imported_count = report.imported_count.saturating_add(1);
    report.total_payload_bytes = report.total_payload_bytes.saturating_add(actual_nar_size);
    report.paths.push(listed_summary(&listed));
    Ok(())
}

fn validate_imported_payload(
    path_frame: &ArchivePathFrame,
    node: &Node,
    actual_nar_sha256: [u8; 32],
    actual_nar_size: u64,
    bytes_read: u64,
    payload_blake3: &str,
) -> Result<(), Error> {
    if bytes_read != path_frame.payload_len {
        return Err(Error::Store(format!(
            "archive payload length mismatch for {}: read {bytes_read}, expected {}",
            path_frame.path_info.store_path, path_frame.payload_len
        )));
    }
    if payload_blake3 != path_frame.payload_blake3 {
        return Err(Error::Store(format!("archive payload BLAKE3 mismatch for {}", path_frame.path_info.store_path)));
    }
    if actual_nar_sha256 != path_frame.path_info.nar_sha256 {
        return Err(Error::Store(format!("archive payload SHA-256 mismatch for {}", path_frame.path_info.store_path)));
    }
    if actual_nar_size != path_frame.path_info.nar_size {
        return Err(Error::Store(format!("archive payload NAR size mismatch for {}", path_frame.path_info.store_path)));
    }
    if node != &path_frame.path_info.node {
        return Err(Error::Store(format!("archive payload node mismatch for {}", path_frame.path_info.store_path)));
    }
    Ok(())
}

fn signatures_trusted(path_info: &PathInfo, store_dir: &str, options: &ArchiveImportOptions) -> bool {
    if options.trust_unsigned {
        return true;
    }
    if path_info.signatures.is_empty() {
        return false;
    }
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let refs: Vec<StorePathRef> = path_info.references.iter().map(StorePath::as_ref).collect();
    let fp = nix_compat::narinfo::fingerprint_with_store_dir(
        &store_path_ref,
        &path_info.nar_sha256,
        path_info.nar_size,
        refs.iter(),
        store_dir,
    );
    path_info
        .signatures
        .iter()
        .any(|signature| options.trusted_public_keys.iter().any(|key| key.verify(&fp, &signature.as_ref())))
}

fn listed_path_from_frame(path_frame: &ArchivePathFrame) -> Result<ArchiveListedPath, Error> {
    if path_frame.payload_blake3.len() != HASH_HEX_BYTES {
        return Err(Error::Store(format!(
            "archive record {} has invalid BLAKE3 digest length",
            path_frame.path_info.store_path
        )));
    }
    Ok(ArchiveListedPath {
        store_path: path_frame.path_info.store_path.to_string(),
        nar_size: path_frame.path_info.nar_size,
        nar_sha256_hex: data_encoding::HEXLOWER.encode(&path_frame.path_info.nar_sha256),
        reference_count: checked_reference_count(path_frame.path_info.references.len())?,
        signature_count: checked_signature_count(path_frame.path_info.signatures.len())?,
        ca: path_frame.path_info.ca.as_ref().map(|ca| format!("{ca:?}")),
        root: path_frame.root,
        payload_blake3: path_frame.payload_blake3.clone(),
    })
}

fn validate_header_store_prefix(header: &ArchiveHeaderFrame, expected_store_prefix: &str) -> Result<(), Error> {
    if header.store_prefix != expected_store_prefix {
        return Err(Error::Store(format!(
            "archive store prefix mismatch: expected {expected_store_prefix}, archive declares {}",
            header.store_prefix
        )));
    }
    Ok(())
}

fn validate_end_frame(report: &ArchiveListReport, end: &ArchiveEndFrame) -> Result<(), Error> {
    if end.payload_len != 0 {
        return Err(Error::Store("archive end frame unexpectedly declares payload bytes".to_string()));
    }
    if end.record_count != report.record_count {
        return Err(Error::Store(format!(
            "archive header/end record count mismatch: header says {}, end says {}",
            report.record_count, end.record_count
        )));
    }
    if end.record_count != report.paths.len() as u32 {
        return Err(Error::Store(format!(
            "archive end record count mismatch: saw {}, end says {}",
            report.paths.len(),
            end.record_count
        )));
    }
    if end.total_payload_bytes != report.total_payload_bytes {
        return Err(Error::Store(format!(
            "archive end payload byte mismatch: saw {}, end says {}",
            report.total_payload_bytes, end.total_payload_bytes
        )));
    }
    Ok(())
}

fn validate_import_end(
    report: &ArchiveImportReport,
    end: &ArchiveEndFrame,
    expected_record_count: u32,
) -> Result<(), Error> {
    if end.payload_len != 0 {
        return Err(Error::Store("archive end frame unexpectedly declares payload bytes".to_string()));
    }
    if end.record_count != expected_record_count {
        return Err(Error::Store(format!(
            "archive header/end record count mismatch: header says {expected_record_count}, end says {}",
            end.record_count
        )));
    }
    let processed = report.imported_count.saturating_add(report.skipped_already_present_count);
    if end.record_count != processed {
        return Err(Error::Store(format!(
            "archive end record count mismatch: processed {processed}, end says {}",
            end.record_count
        )));
    }
    Ok(())
}

fn summary_for_pathinfo(path_info: &PathInfo) -> ArchivePathSummary {
    ArchivePathSummary {
        store_path: path_info.store_path.to_string(),
        nar_size: path_info.nar_size,
        nar_sha256_hex: data_encoding::HEXLOWER.encode(&path_info.nar_sha256),
    }
}

fn listed_summary(listed: &ArchiveListedPath) -> ArchivePathSummary {
    ArchivePathSummary {
        store_path: listed.store_path.clone(),
        nar_size: listed.nar_size,
        nar_sha256_hex: listed.nar_sha256_hex.clone(),
    }
}

async fn read_magic<R: AsyncRead + Unpin>(reader: &mut R) -> Result<(), Error> {
    let mut magic = vec![0u8; ARCHIVE_MAGIC.len()];
    reader.read_exact(&mut magic).await.map_err(read_archive_error("reading archive magic"))?;
    if magic != ARCHIVE_MAGIC {
        return Err(Error::Store("unsupported archive magic; expected mantle-store-archive-v1".to_string()));
    }
    Ok(())
}

async fn read_header<R: AsyncRead + Unpin>(reader: &mut R) -> Result<ArchiveHeaderFrame, Error> {
    match read_frame(reader).await? {
        ArchiveFrame::Header(header) => {
            validate_header(&header)?;
            Ok(header)
        }
        ArchiveFrame::Path(_) => Err(Error::Store("archive starts with path frame before header".to_string())),
        ArchiveFrame::End(_) => Err(Error::Store("archive starts with end frame before header".to_string())),
    }
}

fn validate_header(header: &ArchiveHeaderFrame) -> Result<(), Error> {
    if header.format != ARCHIVE_FORMAT_NAME {
        return Err(Error::Store(format!("unsupported archive format {}", header.format)));
    }
    if header.version != ARCHIVE_VERSION {
        return Err(Error::Store(format!("unsupported archive version {}", header.version)));
    }
    if header.payload_len != 0 {
        return Err(Error::Store("archive header unexpectedly declares payload bytes".to_string()));
    }
    if header.record_count as usize > MAX_ARCHIVE_RECORDS {
        return Err(Error::Store(format!("archive record count exceeds {MAX_ARCHIVE_RECORDS}")));
    }
    if !header.store_prefix.starts_with('/') {
        return Err(Error::Store(format!("archive store prefix is not absolute: {}", header.store_prefix)));
    }
    Ok(())
}

async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &ArchiveFrame) -> Result<(), Error> {
    let payload_len = frame.payload_len();
    if !matches!(frame, ArchiveFrame::Path(_)) && payload_len != 0 {
        return Err(Error::Export("only path frames may carry payload bytes".to_string()));
    }
    let encoded = serde_json::to_vec(frame).map_err(|err| Error::Export(format!("encoding archive frame: {err}")))?;
    if encoded.len() > MAX_ARCHIVE_METADATA_BYTES {
        return Err(Error::Export(format!("archive metadata frame exceeded {MAX_ARCHIVE_METADATA_BYTES} bytes")));
    }
    let frame_len = checked_frame_len(encoded.len())?;
    writer
        .write_all(&frame_len.to_le_bytes())
        .await
        .map_err(write_archive_error("writing archive frame length"))?;
    writer.write_all(&encoded).await.map_err(write_archive_error("writing archive frame metadata"))?;
    Ok(())
}

async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<ArchiveFrame, Error> {
    let mut len_bytes = [0u8; FRAME_LEN_BYTES];
    reader
        .read_exact(&mut len_bytes)
        .await
        .map_err(read_archive_error("reading archive frame length"))?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    if len == 0 || len > MAX_ARCHIVE_METADATA_BYTES {
        return Err(Error::Store(format!(
            "archive metadata frame length {len} is outside 1..={MAX_ARCHIVE_METADATA_BYTES}"
        )));
    }
    let mut metadata = vec![0u8; len];
    reader
        .read_exact(&mut metadata)
        .await
        .map_err(read_archive_error("reading archive frame metadata"))?;
    serde_json::from_slice(&metadata).map_err(|err| Error::Store(format!("parsing archive frame metadata: {err}")))
}

async fn drain_payload<R: AsyncRead + Unpin>(reader: &mut R, payload_len: u64) -> Result<(), Error> {
    let mut remaining = payload_len;
    let mut buffer = vec![0u8; ARCHIVE_IO_BUFFER_BYTES];
    while remaining > 0 {
        let read_limit = remaining.min(ARCHIVE_IO_BUFFER_BYTES as u64) as usize;
        let n = reader
            .read(&mut buffer[..read_limit])
            .await
            .map_err(read_archive_error("draining archive payload"))?;
        if n == 0 {
            return Err(Error::Store(format!("archive ended while draining {remaining} payload bytes")));
        }
        remaining = remaining.saturating_sub(n as u64);
    }
    Ok(())
}

fn checked_frame_len(len: usize) -> Result<u32, Error> {
    u32::try_from(len).map_err(|_| Error::Export(format!("archive frame length does not fit in u32: {len}")))
}

fn checked_record_count(count: usize) -> Result<u32, Error> {
    u32::try_from(count).map_err(|_| Error::Export(format!("archive record count does not fit in u32: {count}")))
}

fn checked_reference_count(count: usize) -> Result<u32, Error> {
    u32::try_from(count).map_err(|_| Error::Store(format!("archive reference count does not fit in u32: {count}")))
}

fn checked_signature_count(count: usize) -> Result<u32, Error> {
    u32::try_from(count).map_err(|_| Error::Store(format!("archive signature count does not fit in u32: {count}")))
}

fn read_archive_error(context: &'static str) -> impl FnOnce(std::io::Error) -> Error {
    move |err| Error::Store(format!("{context}: {err}"))
}

fn write_archive_error(context: &'static str) -> impl FnOnce(std::io::Error) -> Error {
    move |err| Error::Export(format!("{context}: {err}"))
}

struct HashingAsyncWriter<W> {
    inner: W,
    hasher: blake3::Hasher,
    bytes_written: u64,
}

impl<W> HashingAsyncWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
            bytes_written: 0,
        }
    }

    fn finish(self) -> (u64, String) {
        (self.bytes_written, self.hasher.finalize().to_hex().to_string())
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for HashingAsyncWriter<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        let written = futures::ready!(Pin::new(&mut self.inner).poll_write(cx, buf))?;
        if written > 0 {
            self.hasher.update(&buf[..written]);
            self.bytes_written = self.bytes_written.saturating_add(written as u64);
        }
        Poll::Ready(Ok(written))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

struct Blake3AsyncReader<R> {
    inner: R,
    hasher: blake3::Hasher,
    bytes_read: u64,
}

impl<R> Blake3AsyncReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
            bytes_read: 0,
        }
    }

    fn finish(self) -> (u64, String) {
        (self.bytes_read, self.hasher.finalize().to_hex().to_string())
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for Blake3AsyncReader<R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let filled_before = buf.filled().len();
        futures::ready!(Pin::new(&mut self.inner).poll_read(cx, buf))?;
        let filled_after = buf.filled().len();
        if filled_after > filled_before {
            let new_bytes = &buf.filled()[filled_before..filled_after];
            self.hasher.update(new_bytes);
            self.bytes_read = self.bytes_read.saturating_add(new_bytes.len() as u64);
        }
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
#[allow(clippy::cloned_ref_to_slice_refs)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use nix_compat::narinfo::SigningKey;
    use nix_compat::store_path::StorePath;
    use sha2::Digest;
    use snix_castore::Node;
    use snix_store::pathinfoservice::LruPathInfoService;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::StoreConfig;
    use crate::StoreFallbackMode;

    async fn open_test_store(dir: &std::path::Path, store_dir: &str) -> StoreHandle {
        StoreHandle::open(StoreConfig {
            state_dir: dir.join("state"),
            output_dir: dir.join("store"),
            remote_cache_url: None,
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: store_dir.to_string(),
        })
        .await
        .unwrap()
    }

    async fn signed_pathinfo(handle: &StoreHandle, name: &str, content: &[u8]) -> PathInfo {
        let mut writer = handle.blob_service().open_write().await;
        writer.write_all(content).await.unwrap();
        let blob_digest = writer.close().await.unwrap();
        let node = Node::File {
            digest: blob_digest,
            size: content.len() as u64,
            executable: false,
        };
        let nar_buf = render_nar_bytes(handle, &node).await;
        let nar_sha256: [u8; 32] = sha2::Sha256::digest(&nar_buf).into();
        let mut digest = [0u8; 20];
        for (index, byte) in name.as_bytes().iter().enumerate().take(20) {
            digest[index] = *byte;
        }
        let mut path_info = PathInfo {
            store_path: StorePath::from_name_and_digest_fixed(name, digest).unwrap(),
            node,
            references: vec![],
            nar_size: nar_buf.len() as u64,
            nar_sha256,
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        sign_pathinfo(&mut path_info);
        path_info
    }

    async fn render_nar_bytes(handle: &StoreHandle, node: &Node) -> Vec<u8> {
        use snix_store::nar::write_nar;
        use tokio::io::AsyncReadExt;

        let (mut reader, writer) = tokio::io::duplex(ARCHIVE_IO_BUFFER_BYTES);
        let blob_service = handle.blob_service();
        let directory_service = handle.directory_service();
        let node = node.clone();
        let write_task = tokio::spawn(async move { write_nar(writer, &node, blob_service, directory_service).await });
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        write_task.await.unwrap().unwrap();
        buf
    }

    fn test_keypair() -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
        let raw_signing_key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let verifying_key = VerifyingKey::new("archive-test-1".to_string(), raw_signing_key.verifying_key());
        let signing_key = SigningKey::new("archive-test-1".to_string(), raw_signing_key);
        (signing_key, verifying_key)
    }

    fn sign_pathinfo(path_info: &mut PathInfo) {
        let (signing_key, _) = test_keypair();
        let store_path_ref: StorePathRef = path_info.store_path.as_ref();
        let refs: Vec<StorePathRef> = path_info.references.iter().map(StorePath::as_ref).collect();
        let fp = nix_compat::narinfo::fingerprint_with_store_dir(
            &store_path_ref,
            &path_info.nar_sha256,
            path_info.nar_size,
            refs.iter(),
            "/mantle/store",
        );
        path_info.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
    }

    fn wrong_verifying_key() -> VerifyingKey {
        const WRONG_KEY_BYTE: u8 = 7;
        let raw_signing_key = ed25519_dalek::SigningKey::from_bytes(&[WRONG_KEY_BYTE; 32]);
        VerifyingKey::new("archive-test-1".to_string(), raw_signing_key.verifying_key())
    }

    fn first_payload_range(archive: &[u8]) -> std::ops::Range<usize> {
        let mut offset = ARCHIVE_MAGIC.len();
        offset = skip_metadata_frame(archive, offset);
        let path_frame_len = frame_len_at(archive, offset);
        let path_frame_start = offset + FRAME_LEN_BYTES;
        let path_frame_end = path_frame_start + path_frame_len;
        let frame: ArchiveFrame = serde_json::from_slice(&archive[path_frame_start..path_frame_end]).unwrap();
        let ArchiveFrame::Path(path_frame) = frame else {
            panic!("expected first record frame to be a path frame");
        };
        let payload_len = usize::try_from(path_frame.payload_len).unwrap();
        path_frame_end..path_frame_end + payload_len
    }

    fn skip_metadata_frame(archive: &[u8], offset: usize) -> usize {
        offset + FRAME_LEN_BYTES + frame_len_at(archive, offset)
    }

    fn frame_len_at(archive: &[u8], offset: usize) -> usize {
        let len_end = offset + FRAME_LEN_BYTES;
        let len_bytes: [u8; FRAME_LEN_BYTES] = archive[offset..len_end].try_into().unwrap();
        u32::from_le_bytes(len_bytes) as usize
    }

    fn tamper_payload_content(archive: &mut [u8], needle: &[u8]) {
        const PAYLOAD_TAMPER_MASK: u8 = 1;
        let range = first_payload_range(archive);
        let relative = archive[range.clone()].windows(needle.len()).position(|window| window == needle).unwrap();
        archive[range.start + relative] ^= PAYLOAD_TAMPER_MASK;
    }

    fn truncate_first_payload(archive: &mut Vec<u8>) {
        const TRUNCATED_PAYLOAD_BYTES: usize = 1;
        let range = first_payload_range(archive);
        archive.truncate(range.end - TRUNCATED_PAYLOAD_BYTES);
    }

    fn replace_first_path_frame_metadata(archive: &mut Vec<u8>, mutate: impl FnOnce(&mut serde_json::Value)) {
        let mut offset = ARCHIVE_MAGIC.len();
        offset = skip_metadata_frame(archive, offset);
        let path_frame_len = frame_len_at(archive, offset);
        let path_frame_start = offset + FRAME_LEN_BYTES;
        let path_frame_end = path_frame_start + path_frame_len;
        let mut value: serde_json::Value = serde_json::from_slice(&archive[path_frame_start..path_frame_end]).unwrap();
        mutate(&mut value);
        let encoded = serde_json::to_vec(&value).unwrap();
        assert!(encoded.len() <= MAX_ARCHIVE_METADATA_BYTES);
        let frame_len = u32::try_from(encoded.len()).unwrap();
        let mut replacement = Vec::with_capacity(FRAME_LEN_BYTES + encoded.len());
        replacement.extend_from_slice(&frame_len.to_le_bytes());
        replacement.extend_from_slice(&encoded);
        archive.splice(offset..path_frame_end, replacement);
    }

    struct ChunkedAsyncRead {
        bytes: Vec<u8>,
        pos: usize,
        chunk_bytes: usize,
    }

    impl ChunkedAsyncRead {
        fn new(bytes: Vec<u8>, chunk_bytes: usize) -> Self {
            assert!(chunk_bytes > 0, "chunked reader chunk size must be positive");
            Self {
                bytes,
                pos: 0,
                chunk_bytes,
            }
        }
    }

    impl AsyncRead for ChunkedAsyncRead {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            if self.pos >= self.bytes.len() {
                return Poll::Ready(Ok(()));
            }
            let remaining = self.bytes.len() - self.pos;
            let writable = buf.remaining().min(self.chunk_bytes).min(remaining);
            let start = self.pos;
            let end = start + writable;
            buf.put_slice(&self.bytes[start..end]);
            self.pos = end;
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn archive_export_list_round_trip_preserves_metadata_before_payload() {
        let temp = tempfile::tempdir().unwrap();
        let store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&store, "archive-root", b"hello archive").await;
        store.pathinfo_service().put(path_info.clone()).await.unwrap();

        let mut archive = Vec::new();
        let report =
            export_store_archive(&store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
                trust_unsigned: false,
            })
            .await
            .unwrap();
        assert_eq!(report.exported_count, 1);
        assert!(archive.starts_with(ARCHIVE_MAGIC));

        let mut reader = std::io::Cursor::new(archive);
        let listed = list_store_archive(&mut reader).await.unwrap();
        assert_eq!(listed.store_prefix, "/mantle/store");
        assert_eq!(listed.paths.len(), 1);
        assert_eq!(listed.paths[0].store_path, path_info.store_path.to_string());
        assert_eq!(listed.paths[0].signature_count, 1);
    }

    #[tokio::test]
    async fn archive_import_rejects_store_prefix_mismatch_before_persisting() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "prefix-root", b"wrong prefix").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/nix/store").await;
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("store prefix mismatch"));
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn archive_import_round_trip_and_skip_existing_are_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "import-root", b"import me").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let mut first_reader = std::io::Cursor::new(archive.clone());
        let first = import_store_archive(&dest_store, &mut first_reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: true,
        })
        .await
        .unwrap();
        assert_eq!(first.imported_count, 1);
        assert_eq!(first.skipped_already_present_count, 0);
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_some());

        let mut second_reader = std::io::Cursor::new(archive);
        let second = import_store_archive(&dest_store, &mut second_reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: true,
        })
        .await
        .unwrap();
        assert_eq!(second.imported_count, 0);
        assert_eq!(second.skipped_already_present_count, 1);
    }

    #[test]
    fn import_action_core_rejects_untrusted_and_skips_present() {
        let record = ArchiveListedPath {
            store_path: "/mantle/store/example".to_string(),
            nar_size: 1,
            nar_sha256_hex: "0".repeat(HASH_HEX_BYTES),
            reference_count: 0,
            signature_count: 1,
            ca: None,
            root: true,
            payload_blake3: "1".repeat(HASH_HEX_BYTES),
        };
        let rejected = plan_archive_import_action(&record, false, true, false);
        assert_eq!(rejected.action, ArchiveImportAction::Reject);
        assert_eq!(rejected.reason, "untrusted-signature");
        let skipped = plan_archive_import_action(&record, true, true, true);
        assert_eq!(skipped.action, ArchiveImportAction::SkipExisting);
        assert_eq!(skipped.reason, "already-present");
    }

    #[tokio::test]
    async fn archive_list_rejects_bad_magic() {
        let mut reader = std::io::Cursor::new(b"not-an-archive".to_vec());
        let err = list_store_archive(&mut reader).await.unwrap_err();
        assert!(err.to_string().contains("archive magic"));
    }

    #[tokio::test]
    async fn archive_list_rejects_header_end_count_mismatch() {
        const DECLARED_RECORD_COUNT: u32 = 1;
        const ACTUAL_RECORD_COUNT: u32 = 0;
        let mut archive = Vec::new();
        archive.extend_from_slice(ARCHIVE_MAGIC);
        write_frame(
            &mut archive,
            &ArchiveFrame::Header(ArchiveHeaderFrame {
                format: ARCHIVE_FORMAT_NAME.to_string(),
                version: ARCHIVE_VERSION,
                store_prefix: "/mantle/store".to_string(),
                record_count: DECLARED_RECORD_COUNT,
                roots: vec![],
                compatibility: ARCHIVE_COMPATIBILITY.to_string(),
                payload_len: 0,
            }),
        )
        .await
        .unwrap();
        write_frame(
            &mut archive,
            &ArchiveFrame::End(ArchiveEndFrame {
                record_count: ACTUAL_RECORD_COUNT,
                total_payload_bytes: 0,
                payload_len: 0,
            }),
        )
        .await
        .unwrap();

        let mut reader = std::io::Cursor::new(archive);
        let err = list_store_archive(&mut reader).await.unwrap_err();
        assert!(err.to_string().contains("header/end record count mismatch"));
    }

    #[tokio::test]
    async fn export_closure_includes_references_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let store = open_test_store(temp.path(), "/mantle/store").await;
        let dep = signed_pathinfo(&store, "closure-dep", b"dep").await;
        let mut root = signed_pathinfo(&store, "closure-root", b"root").await;
        root.references.push(dep.store_path.clone());
        sign_pathinfo(&mut root);
        store.pathinfo_service().put(root.clone()).await.unwrap();
        store.pathinfo_service().put(dep.clone()).await.unwrap();

        let plan =
            plan_export_closure(&store, std::slice::from_ref(&root), &ArchiveExportOptions { trust_unsigned: false })
                .await
                .unwrap();
        let paths = plan.iter().map(|pi| pi.store_path.to_string()).collect::<Vec<_>>();
        assert_eq!(paths, vec![dep.store_path.to_string(), root.store_path.to_string()]);
    }

    #[tokio::test]
    async fn missing_closure_reference_fails_export_plan() {
        let temp = tempfile::tempdir().unwrap();
        let store = open_test_store(temp.path(), "/mantle/store").await;
        let missing = StorePath::from_name_and_digest_fixed("missing-dep", [9u8; 20]).unwrap();
        let mut root = signed_pathinfo(&store, "missing-root", b"root").await;
        root.references.push(missing.clone());
        sign_pathinfo(&mut root);
        store.pathinfo_service().put(root.clone()).await.unwrap();

        let err =
            plan_export_closure(&store, std::slice::from_ref(&root), &ArchiveExportOptions { trust_unsigned: false })
                .await
                .unwrap_err();
        assert!(err.to_string().contains("missing closure facts"));
    }

    #[tokio::test]
    async fn archive_export_refuses_unsigned_without_escape_hatch() {
        let temp = tempfile::tempdir().unwrap();
        let store = open_test_store(temp.path(), "/mantle/store").await;
        let mut path_info = signed_pathinfo(&store, "unsigned-root", b"unsigned archive").await;
        path_info.signatures.clear();
        store.pathinfo_service().put(path_info.clone()).await.unwrap();

        let mut rejected_archive = Vec::new();
        let rejected = export_store_archive(
            &store,
            std::slice::from_ref(&path_info),
            &mut rejected_archive,
            &ArchiveExportOptions { trust_unsigned: false },
        )
        .await
        .unwrap_err();
        assert!(rejected.to_string().contains("unsigned"));
        assert!(rejected_archive.is_empty());

        let mut accepted_archive = Vec::new();
        let accepted = export_store_archive(
            &store,
            std::slice::from_ref(&path_info),
            &mut accepted_archive,
            &ArchiveExportOptions { trust_unsigned: true },
        )
        .await
        .unwrap();
        assert_eq!(accepted.exported_count, 1);
        assert!(accepted_archive.starts_with(ARCHIVE_MAGIC));
    }

    #[tokio::test]
    async fn archive_import_rejects_untrusted_signature_without_persisting() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "untrusted-root", b"untrusted archive").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![wrong_verifying_key()],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("untrusted-signature"));
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn archive_import_rejects_tampered_payload_without_persisting() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let payload = b"tamper me";
        let path_info = signed_pathinfo(&source_store, "tamper-root", payload).await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
        tamper_payload_content(&mut archive, payload);

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("mismatch") || err.to_string().contains("ingesting"));
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn archive_import_rejects_unsupported_ca_metadata_without_persisting() {
        const UNSUPPORTED_CA_HASH_ALGO: &str = "unsupported";
        const VALID_CA_HASH_BASE32: &str = "1fnf2m46ya7r7afkcb8ba2j0sc4a85m749sh9jz64g4hx6z3r088";
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "unsupported-ca-root", b"unsupported ca archive").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
        replace_first_path_frame_metadata(&mut archive, |frame| {
            frame["path_info"]["ca"] = serde_json::json!({
                "hash": VALID_CA_HASH_BASE32,
                "hashAlgo": UNSUPPORTED_CA_HASH_ALGO,
            });
        });

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("parsing archive frame metadata"));
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn archive_import_rejects_truncated_payload_without_persisting() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "truncated-root", b"truncated archive").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
        truncate_first_payload(&mut archive);

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("ingesting") || err.to_string().contains("mismatch"));
        assert!(dest_store.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn archive_import_rejects_conflicting_local_pathinfo() {
        let temp = tempfile::tempdir().unwrap();
        let source_store = open_test_store(temp.path(), "/mantle/store").await;
        let path_info = signed_pathinfo(&source_store, "conflict-root", b"archive content").await;
        source_store.pathinfo_service().put(path_info.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&source_store, std::slice::from_ref(&path_info), &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let dest_temp = tempfile::tempdir().unwrap();
        let dest_store = open_test_store(dest_temp.path(), "/mantle/store").await;
        let stale = signed_pathinfo(&dest_store, "conflict-root", b"stale local content").await;
        dest_store.pathinfo_service().put(stale.clone()).await.unwrap();
        let mut reader = std::io::Cursor::new(archive);
        let err = import_store_archive(&dest_store, &mut reader, &ArchiveImportOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_keypair().1],
            materialize: false,
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("conflicts with archive metadata"));
        let retained = dest_store.pathinfo_service().get(*stale.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(retained.nar_sha256, stale.nar_sha256);
    }

    #[tokio::test]
    async fn archive_list_drains_non_seekable_payloads_in_bounded_chunks() {
        const NON_SEEK_CHUNK_BYTES: usize = 7;
        let temp = tempfile::tempdir().unwrap();
        let store = open_test_store(temp.path(), "/mantle/store").await;
        let first = signed_pathinfo(&store, "stream-a", b"first streamed payload").await;
        let second = signed_pathinfo(&store, "stream-b", b"second streamed payload").await;
        store.pathinfo_service().put(first.clone()).await.unwrap();
        store.pathinfo_service().put(second.clone()).await.unwrap();
        let mut archive = Vec::new();
        export_store_archive(&store, &[first.clone(), second.clone()], &mut archive, &ArchiveExportOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let mut reader = ChunkedAsyncRead::new(archive, NON_SEEK_CHUNK_BYTES);
        let listed = list_store_archive(&mut reader).await.unwrap();
        let paths = listed.paths.iter().map(|path| path.store_path.clone()).collect::<BTreeSet<_>>();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&first.store_path.to_string()));
        assert!(paths.contains(&second.store_path.to_string()));
    }

    #[test]
    fn pathinfo_fixture_service_is_bounded() {
        let service = LruPathInfoService::with_capacity("archive-fixture".to_string(), NonZeroUsize::new(32).unwrap());
        let _: Arc<dyn PathInfoService> = Arc::new(service);
        assert!(MAX_ARCHIVE_RECORDS > 0);
        assert!(MAX_ARCHIVE_METADATA_BYTES >= FRAME_LEN_BYTES);
    }
}
