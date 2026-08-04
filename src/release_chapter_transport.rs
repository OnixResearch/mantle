use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use chapter_tgz::Compression;
use chapter_tgz::TgzReader;
use chapter_tgz::TgzWriter;
use crunch_release_core::CHAPTER_TRANSPORT_MANIFEST_PATH;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_INDEX_BYTES;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_MEMBER_BYTES;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT;
use crunch_release_core::CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES;
use crunch_release_core::CHAPTER_TRANSPORT_RESERVED_INDEX_PATH;
use crunch_release_core::ChapterTransportChapter;
use crunch_release_core::ChapterTransportEntryInput;
use crunch_release_core::ChapterTransportEntryKind;
use crunch_release_core::ChapterTransportIndex;
use crunch_release_core::ChapterTransportMember;
use crunch_release_core::ChapterTransportReceipt;
use crunch_release_core::ChapterTransportReceiptInput;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::build_chapter_transport_receipt;
use crunch_release_core::canonical_chapter_transport_index;
use crunch_release_core::canonical_chapter_transport_receipt;
use crunch_release_core::canonical_release_evidence_manifest;
use crunch_release_core::parse_canonical_chapter_transport_index;
use crunch_release_core::parse_canonical_chapter_transport_receipt;
use crunch_release_core::plan_chapter_transport;
use crunch_release_core::render_chapter_transport_blockers;
use serde::Serialize;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::release_tree_copy::PreparedTreeCopy;
use crate::release_tree_copy::hash_file_nofollow;
use crate::release_tree_copy::prepare_tree_copy;

pub(crate) const CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME: &str = "release.tgz";
pub(crate) const CHAPTER_TRANSPORT_RECEIPT_FILE_NAME: &str = "receipt.json";
const CHAPTER_TRANSPORT_PACK_OUTPUT_KIND: &str = "mantle-release-chapter-transport-pack-v1";
const CHAPTER_TRANSPORT_INSPECTION_OUTPUT_KIND: &str = "mantle-release-chapter-transport-inspect-v1";
const CHAPTER_TRANSPORT_UNPACK_OUTPUT_KIND: &str = "mantle-release-chapter-transport-unpack-v1";
const CHAPTER_TRANSPORT_GZIP_LEVEL: u32 = 6;
const CHAPTER_TRANSPORT_INDEX_MODE: u32 = 0o444;
const CHAPTER_TRANSPORT_RECEIPT_BYTES_MAX: u64 = 1_048_576;
const CHAPTER_TRANSPORT_COPY_BUFFER_BYTES: usize = 65_536;
const CHAPTER_TRANSPORT_MAX_GZIP_OVERHEAD_BYTES: u64 = 128 * 1_024 * 1_024;
const CHAPTER_TRANSPORT_STAGE_PREFIX: &str = ".mantle-chapter-transport-stage-";
const CHAPTER_TRANSPORT_UNPACK_STAGE_PREFIX: &str = ".mantle-chapter-unpack-stage-";
const CHAPTER_TRANSPORT_DIRECTORY_ENTRY_COUNT: usize = 2;
const CHAPTER_TRANSPORT_RESERVED_ENTRY_INDEX: u32 = 0;
const CHAPTER_TRANSPORT_MANIFEST_ENTRY_INDEX: u32 = 1;
// Audited chapter-tgz 0.1.0 dynamic-deflate marker prefixes. The first byte
// differs only for the final marker.
const CHAPTER_MARKER_NONFINAL_START: u8 = 0b0000_0100;
const CHAPTER_MARKER_FINAL_START: u8 = 0b0000_0101;
const CHAPTER_MARKER_SUFFIX: [u8; 8] = [0b1100_0000, 0b0001_0001, 1, 0, 0, 0, 0, 0b0010_0000];
#[cfg(unix)]
const UNIX_PERMISSION_BITS_MASK: u32 = 0o7_777;
#[cfg(not(unix))]
const UNIX_PERMISSION_BITS_MASK: u32 = 0;

const _: () = {
    assert!(CHAPTER_TRANSPORT_GZIP_LEVEL > 0);
    assert!(CHAPTER_TRANSPORT_RECEIPT_BYTES_MAX > 0);
    assert!(CHAPTER_TRANSPORT_COPY_BUFFER_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_MAX_GZIP_OVERHEAD_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_DIRECTORY_ENTRY_COUNT > 1);
    assert!(CHAPTER_MARKER_SUFFIX.len() > 1);
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReleaseTransportPackOutcome {
    pub kind: String,
    pub transport_dir: PathBuf,
    pub archive_path: PathBuf,
    pub receipt_path: PathBuf,
    pub receipt: ChapterTransportReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReleaseTransportInspection {
    pub kind: String,
    pub transport_dir: PathBuf,
    pub release_id: String,
    pub receipt: ChapterTransportReceipt,
    pub index: ChapterTransportIndex,
    pub compressed_chapter_sizes: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReleaseTransportUnpackOutcome {
    pub kind: String,
    pub transport_dir: PathBuf,
    pub destination_dir: PathBuf,
    pub release_id: String,
    pub archive_digest_blake3: String,
    pub chapter_count: u32,
    pub member_count: u32,
}

struct ControlChapter {
    manifest: ReleaseEvidenceManifest,
    index: ChapterTransportIndex,
    compressed_chapter_sizes: Vec<u64>,
}

struct VerifiedArchiveSnapshot {
    file: tempfile::NamedTempFile,
}

struct ArchiveCopyObservation {
    digest_blake3: String,
    chapter_marker_count: u32,
}

#[derive(Default)]
struct ChapterMarkerPrefixCounter {
    matched_bytes: usize,
    marker_count: u32,
}

impl ChapterMarkerPrefixCounter {
    fn observe(&mut self, bytes: &[u8]) -> Result<(), RunError> {
        for byte in bytes {
            self.observe_byte(*byte)?;
        }
        Ok(())
    }

    fn observe_byte(&mut self, byte: u8) -> Result<(), RunError> {
        if self.matched_bytes == 0 {
            self.matched_bytes = usize::from(is_chapter_marker_start(byte));
            return Ok(());
        }
        let suffix_index = self
            .matched_bytes
            .checked_sub(1)
            .ok_or_else(|| transport_error("chapter marker prefix state underflowed"))?;
        if CHAPTER_MARKER_SUFFIX.get(suffix_index).copied() == Some(byte) {
            self.matched_bytes = self
                .matched_bytes
                .checked_add(1)
                .ok_or_else(|| transport_error("chapter marker prefix state overflowed"))?;
            if self.matched_bytes == CHAPTER_MARKER_SUFFIX.len() + 1 {
                self.marker_count = self
                    .marker_count
                    .checked_add(1)
                    .ok_or_else(|| transport_error("chapter marker prefix count overflowed u32"))?;
                if self.marker_count > CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT {
                    return Err(transport_error("chapter marker prefix count exceeds the transport limit"));
                }
                self.matched_bytes = 0;
            }
        } else {
            self.matched_bytes = usize::from(is_chapter_marker_start(byte));
        }
        Ok(())
    }
}

fn is_chapter_marker_start(byte: u8) -> bool {
    byte == CHAPTER_MARKER_NONFINAL_START || byte == CHAPTER_MARKER_FINAL_START
}

// r[impl mantle.release_provenance.chapter_transport.pack]
// r[impl mantle.release_provenance.chapter_transport.receipt]
pub(crate) fn pack_release_transport(
    bundle_dir: &Path,
    transport_dir: &Path,
) -> Result<ReleaseTransportPackOutcome, RunError> {
    ensure_destination_absent(transport_dir, "chapter transport destination")?;
    let manifest = verify_release_evidence_bundle(bundle_dir)?;
    let manifest_bytes = canonical_release_evidence_manifest(manifest.clone())
        .map_err(|error| transport_error(format!("canonicalizing release manifest before transport: {error}")))?;
    let manifest_digest_blake3 = blake3::hash(&manifest_bytes).to_hex().to_string();
    let prepared = prepare_tree_copy(bundle_dir)?;
    let entries = prepared.chapter_transport_entries()?;
    let index = plan_chapter_transport(entries, manifest_digest_blake3.clone())
        .map_err(|blockers| transport_error(render_chapter_transport_blockers(&blockers)))?;
    let index_bytes = canonical_chapter_transport_index(&index)
        .map_err(|error| transport_error(format!("canonicalizing chapter transport index: {error}")))?;
    let stage = create_sibling_stage(transport_dir, CHAPTER_TRANSPORT_STAGE_PREFIX)?;
    let archive_path = stage.path().join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
    let receipt_path = stage.path().join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME);
    write_chapter_archive(&archive_path, &prepared, &index, &index_bytes)?;
    let (archive_size_bytes, archive_digest_blake3) = hash_file_nofollow(&archive_path)?;
    let receipt = build_chapter_transport_receipt(ChapterTransportReceiptInput {
        archive_size_bytes,
        archive_digest_blake3,
        source_manifest_digest_blake3: manifest_digest_blake3,
        transport_index_digest_blake3: blake3::hash(&index_bytes).to_hex().to_string(),
        chapter_count: index.chapter_count,
        member_count: index.member_count,
    })
    .map_err(|error| transport_error(format!("building chapter transport receipt: {error}")))?;
    write_receipt_file(&receipt_path, &receipt)?;
    let staged_inspection = inspect_release_transport(stage.path())?;
    if staged_inspection.receipt != receipt || staged_inspection.index != index {
        return Err(transport_error("staged chapter transport inspection did not reproduce its plan and receipt"));
    }
    validate_staged_transport_round_trip(stage.path(), &manifest)?;
    publish_stage_no_replace(stage, transport_dir)?;
    let outcome = ReleaseTransportPackOutcome {
        kind: CHAPTER_TRANSPORT_PACK_OUTPUT_KIND.to_string(),
        transport_dir: transport_dir.to_path_buf(),
        archive_path: transport_dir.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME),
        receipt_path: transport_dir.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME),
        receipt,
    };
    debug_assert_eq!(outcome.transport_dir, transport_dir);
    debug_assert_eq!(outcome.receipt.chapter_count, index.chapter_count);
    Ok(outcome)
}

fn validate_staged_transport_round_trip(
    transport_stage: &Path,
    expected_manifest: &ReleaseEvidenceManifest,
) -> Result<(), RunError> {
    let validation_parent = tempfile::tempdir()
        .map_err(|error| transport_error(format!("creating chapter transport round-trip root: {error}")))?;
    let validation_destination = validation_parent.path().join("verified-release");
    let outcome = unpack_release_transport(transport_stage, &validation_destination)?;
    if outcome.release_id != expected_manifest.release_id {
        return Err(transport_error("staged chapter transport round trip changed the release identity"));
    }
    debug_assert!(validation_destination.is_dir());
    Ok(())
}

fn write_chapter_archive(
    archive_path: &Path,
    prepared: &PreparedTreeCopy,
    index: &ChapterTransportIndex,
    index_bytes: &[u8],
) -> Result<(), RunError> {
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(archive_path).map_err(|error| {
        transport_error(format!("creating chapter transport archive {}: {error}", archive_path.display()))
    })?;
    {
        let mut writer = TgzWriter::new(&mut file, Compression::new(CHAPTER_TRANSPORT_GZIP_LEVEL));
        for chapter in &index.chapters {
            write_one_chapter(&mut writer, prepared, chapter, index_bytes)?;
        }
        writer
            .into_inner()
            .map_err(|error| transport_error(format!("finishing chapter transport archive: {error}")))?;
    }
    file.flush()
        .map_err(|error| transport_error(format!("flushing chapter transport archive: {error}")))?;
    file.sync_all()
        .map_err(|error| transport_error(format!("syncing chapter transport archive: {error}")))?;
    debug_assert!(archive_path.is_file());
    debug_assert_eq!(index.chapter_count, u32::try_from(index.chapters.len()).unwrap_or(u32::MAX));
    Ok(())
}

fn write_one_chapter<W: Write>(
    writer: &mut TgzWriter<W>,
    prepared: &PreparedTreeCopy,
    chapter: &ChapterTransportChapter,
    index_bytes: &[u8],
) -> Result<(), RunError> {
    let mut builder = writer.create_chapter();
    builder.mode(tar::HeaderMode::Deterministic);
    if chapter.ordinal == 0 {
        append_transport_index(&mut builder, index_bytes)?;
    }
    for member in &chapter.members {
        prepared.append_chapter_transport_member(&mut builder, member)?;
    }
    builder
        .finish()
        .map_err(|error| transport_error(format!("finishing chapter {}: {error}", chapter.ordinal)))?;
    debug_assert!(!chapter.members.is_empty());
    debug_assert!(chapter.ordinal < CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT);
    Ok(())
}

fn append_transport_index<W: Write>(builder: &mut tar::Builder<W>, bytes: &[u8]) -> Result<(), RunError> {
    let size_bytes =
        u64::try_from(bytes.len()).map_err(|_| transport_error("chapter transport index size overflowed u64"))?;
    if size_bytes > CHAPTER_TRANSPORT_MAX_INDEX_BYTES {
        return Err(transport_error(format!(
            "chapter transport index size {size_bytes} exceeds {CHAPTER_TRANSPORT_MAX_INDEX_BYTES}"
        )));
    }
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(CHAPTER_TRANSPORT_INDEX_MODE);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(size_bytes);
    header.set_cksum();
    builder
        .append_data(&mut header, CHAPTER_TRANSPORT_RESERVED_INDEX_PATH, bytes)
        .map_err(|error| transport_error(format!("appending reserved chapter transport index: {error}")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert_eq!(header.size().ok(), Some(size_bytes));
    Ok(())
}

fn write_receipt_file(path: &Path, receipt: &ChapterTransportReceipt) -> Result<(), RunError> {
    let bytes = canonical_chapter_transport_receipt(receipt)
        .map_err(|error| transport_error(format!("canonicalizing chapter transport receipt: {error}")))?;
    let mut file =
        std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|error| {
            transport_error(format!("creating chapter transport receipt {}: {error}", path.display()))
        })?;
    file.write_all(&bytes)
        .map_err(|error| transport_error(format!("writing chapter transport receipt: {error}")))?;
    file.flush()
        .map_err(|error| transport_error(format!("flushing chapter transport receipt: {error}")))?;
    file.sync_all()
        .map_err(|error| transport_error(format!("syncing chapter transport receipt: {error}")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(path.is_file());
    Ok(())
}

// r[impl mantle.release_provenance.chapter_transport.inspect]
// r[impl mantle.release_provenance.chapter_transport.receipt]
pub(crate) fn inspect_release_transport(transport_dir: &Path) -> Result<ReleaseTransportInspection, RunError> {
    validate_transport_directory_shape(transport_dir)?;
    let receipt_path = transport_dir.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME);
    let archive_path = transport_dir.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
    let receipt_bytes = read_bounded_file_nofollow(&receipt_path, CHAPTER_TRANSPORT_RECEIPT_BYTES_MAX)?;
    let receipt = parse_canonical_chapter_transport_receipt(&receipt_bytes)
        .map_err(|error| transport_error(format!("validating chapter transport receipt: {error}")))?;
    let snapshot = snapshot_verified_archive(&archive_path, &receipt)?;
    let control = read_control_chapter(&snapshot, &receipt)?;
    validate_all_archive_metadata(&snapshot, &control.index)?;
    let inspection = ReleaseTransportInspection {
        kind: CHAPTER_TRANSPORT_INSPECTION_OUTPUT_KIND.to_string(),
        transport_dir: transport_dir.to_path_buf(),
        release_id: control.manifest.release_id,
        receipt,
        index: control.index,
        compressed_chapter_sizes: control.compressed_chapter_sizes,
    };
    debug_assert_eq!(inspection.receipt.chapter_count, inspection.index.chapter_count);
    debug_assert_eq!(inspection.receipt.member_count, inspection.index.member_count);
    Ok(inspection)
}

fn read_control_chapter(
    snapshot: &VerifiedArchiveSnapshot,
    receipt: &ChapterTransportReceipt,
) -> Result<ControlChapter, RunError> {
    let file = snapshot
        .file
        .reopen()
        .map_err(|error| transport_error(format!("reopening chapter transport snapshot: {error}")))?;
    let mut reader = TgzReader::open(file)
        .map_err(|error| transport_error(format!("opening chapter transport archive: {error}")))?;
    validate_reader_chapter_count(&reader, receipt)?;
    let compressed_chapter_sizes = compressed_chapter_sizes(&reader);
    let mut chapter = reader.jump_to_chapter(0);
    let mut entries = chapter
        .entries()
        .map_err(|error| transport_error(format!("reading chapter zero entries: {error}")))?;
    let mut index_entry = entries
        .next()
        .ok_or_else(|| transport_error("chapter zero is missing the reserved transport index"))?
        .map_err(|error| transport_error(format!("reading reserved transport index entry: {error}")))?;
    require_entry_path(&mut index_entry, CHAPTER_TRANSPORT_RESERVED_INDEX_PATH)?;
    let index_bytes = read_entry_bytes(&mut index_entry, CHAPTER_TRANSPORT_MAX_INDEX_BYTES)?;
    let mut manifest_entry = entries
        .next()
        .ok_or_else(|| transport_error("chapter zero is missing manifest.json"))?
        .map_err(|error| transport_error(format!("reading chapter zero manifest entry: {error}")))?;
    require_entry_path(&mut manifest_entry, CHAPTER_TRANSPORT_MANIFEST_PATH)?;
    let manifest_bytes = read_entry_bytes(&mut manifest_entry, CHAPTER_TRANSPORT_MAX_INDEX_BYTES)?;
    if entries.next().is_some() {
        return Err(transport_error("chapter zero contains unexpected entries after manifest.json"));
    }
    let index = parse_canonical_chapter_transport_index(&index_bytes)
        .map_err(|error| transport_error(format!("validating chapter transport index: {error}")))?;
    let manifest: ReleaseEvidenceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| transport_error(format!("parsing transported release manifest: {error}")))?;
    validate_control_identities(receipt, &index, &index_bytes, &manifest, &manifest_bytes)?;
    Ok(ControlChapter {
        manifest,
        index,
        compressed_chapter_sizes,
    })
}

fn validate_control_identities(
    receipt: &ChapterTransportReceipt,
    index: &ChapterTransportIndex,
    index_bytes: &[u8],
    manifest: &ReleaseEvidenceManifest,
    manifest_bytes: &[u8],
) -> Result<(), RunError> {
    let canonical_manifest = canonical_release_evidence_manifest(manifest.clone())
        .map_err(|error| transport_error(format!("validating transported release manifest: {error}")))?;
    if canonical_manifest != manifest_bytes {
        return Err(transport_error("transported release manifest is not canonical compact JSON"));
    }
    let manifest_digest = blake3::hash(manifest_bytes).to_hex().to_string();
    let index_digest = blake3::hash(index_bytes).to_hex().to_string();
    if manifest_digest != receipt.source_manifest_digest_blake3
        || manifest_digest != index.source_manifest_digest_blake3
    {
        return Err(transport_error("chapter transport source manifest digest does not match receipt and index"));
    }
    if index_digest != receipt.transport_index_digest_blake3 {
        return Err(transport_error("chapter transport index digest does not match receipt"));
    }
    if index.chapter_count != receipt.chapter_count || index.member_count != receipt.member_count {
        return Err(transport_error("chapter transport index counts do not match receipt"));
    }
    debug_assert_eq!(canonical_manifest, manifest_bytes);
    debug_assert_eq!(index.chapter_count, receipt.chapter_count);
    Ok(())
}

fn validate_reader_chapter_count<R: Read + Seek>(
    reader: &TgzReader<R>,
    receipt: &ChapterTransportReceipt,
) -> Result<(), RunError> {
    let count = reader.chapters();
    if count == 0 || count > CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT {
        return Err(transport_error(format!("chapter transport reader count {count} is outside the supported bound")));
    }
    if count != receipt.chapter_count {
        return Err(transport_error(format!(
            "chapter transport reader count {count} does not match receipt {}",
            receipt.chapter_count
        )));
    }
    Ok(())
}

fn compressed_chapter_sizes<R: Read + Seek>(reader: &TgzReader<R>) -> Vec<u64> {
    let mut sizes = Vec::with_capacity(usize::try_from(reader.chapters()).unwrap_or_default());
    for ordinal in 0..reader.chapters() {
        sizes.push(reader.compressed_size_of_chapter(ordinal));
    }
    debug_assert_eq!(sizes.len(), usize::try_from(reader.chapters()).unwrap_or_default());
    debug_assert!(sizes.iter().all(|size| *size > 0));
    sizes
}

fn validate_all_archive_metadata(
    snapshot: &VerifiedArchiveSnapshot,
    index: &ChapterTransportIndex,
) -> Result<(), RunError> {
    let file = snapshot
        .file
        .reopen()
        .map_err(|error| transport_error(format!("reopening transport snapshot for metadata validation: {error}")))?;
    let mut reader = TgzReader::open(file)
        .map_err(|error| transport_error(format!("opening transport snapshot for metadata validation: {error}")))?;
    if reader.chapters() != index.chapter_count {
        return Err(transport_error("chapter count changed between control and metadata validation"));
    }
    let mut all_inputs = Vec::with_capacity(usize::try_from(index.member_count).unwrap_or_default());
    for ordinal in 0..reader.chapters() {
        let expected = index
            .chapters
            .get(usize::try_from(ordinal).map_err(|_| transport_error("chapter ordinal overflowed usize"))?)
            .ok_or_else(|| transport_error(format!("chapter {ordinal} is missing from transport index")))?;
        let observed = observe_one_chapter(&mut reader, ordinal)?;
        if observed != expected.members {
            return Err(transport_error(format!("chapter {ordinal} members do not match the transport index")));
        }
        all_inputs.extend(observed.into_iter().map(input_from_member));
    }
    let observed_index = plan_chapter_transport(all_inputs, index.source_manifest_digest_blake3.clone())
        .map_err(|blockers| transport_error(render_chapter_transport_blockers(&blockers)))?;
    if &observed_index != index {
        return Err(transport_error("archive metadata does not reproduce the canonical transport index"));
    }
    debug_assert_eq!(observed_index.chapter_count, index.chapter_count);
    debug_assert_eq!(observed_index.member_count, index.member_count);
    Ok(())
}

fn observe_one_chapter<R: Read + Seek>(
    reader: &mut TgzReader<R>,
    ordinal: u32,
) -> Result<Vec<ChapterTransportMember>, RunError> {
    let mut chapter = reader.jump_to_chapter(ordinal);
    let entries = chapter
        .entries()
        .map_err(|error| transport_error(format!("reading chapter {ordinal} entries: {error}")))?;
    let mut observed = Vec::new();
    let mut entry_index = 0_u32;
    for entry in entries {
        let mut entry = entry.map_err(|error| transport_error(format!("reading chapter {ordinal} entry: {error}")))?;
        let path = entry_path(&mut entry)?;
        if path == CHAPTER_TRANSPORT_RESERVED_INDEX_PATH {
            if ordinal != 0 || entry_index != CHAPTER_TRANSPORT_RESERVED_ENTRY_INDEX {
                return Err(transport_error("reserved transport index appears outside chapter zero entry zero"));
            }
            drain_entry(&mut entry, CHAPTER_TRANSPORT_MAX_INDEX_BYTES)?;
        } else {
            let member = member_from_entry(&mut entry, path)?;
            drain_entry(&mut entry, member.size_bytes)?;
            observed.push(member);
        }
        entry_index =
            entry_index.checked_add(1).ok_or_else(|| transport_error("chapter entry count overflowed u32"))?;
        let original_count = u32::try_from(observed.len())
            .map_err(|_| transport_error("chapter observed member count overflowed u32"))?;
        if original_count > CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT {
            return Err(transport_error("chapter observed member count exceeds the transport limit"));
        }
    }
    if ordinal == 0 && entry_index <= CHAPTER_TRANSPORT_MANIFEST_ENTRY_INDEX {
        return Err(transport_error("chapter zero does not contain reserved index and manifest entries"));
    }
    Ok(observed)
}

fn input_from_member(member: ChapterTransportMember) -> ChapterTransportEntryInput {
    ChapterTransportEntryInput {
        relative_path: member.relative_path,
        kind: member.kind,
        mode: member.mode,
        size_bytes: member.size_bytes,
        symlink_target: member.symlink_target,
    }
}

fn member_from_entry<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    relative_path: String,
) -> Result<ChapterTransportMember, RunError> {
    let entry_type = entry.header().entry_type();
    let kind = if entry_type.is_file() {
        ChapterTransportEntryKind::File
    } else if entry_type.is_dir() {
        ChapterTransportEntryKind::Directory
    } else if entry_type.is_symlink() {
        ChapterTransportEntryKind::Symlink
    } else {
        return Err(transport_error(format!("unsupported chapter transport tar entry type for {relative_path}")));
    };
    let size_bytes = entry.size();
    if size_bytes > CHAPTER_TRANSPORT_MAX_MEMBER_BYTES {
        return Err(transport_error(format!(
            "chapter transport member {relative_path} size {size_bytes} exceeds {CHAPTER_TRANSPORT_MAX_MEMBER_BYTES}"
        )));
    }
    let mode = entry
        .header()
        .mode()
        .map_err(|error| transport_error(format!("reading tar mode for {relative_path}: {error}")))?
        & UNIX_PERMISSION_BITS_MASK;
    let symlink_target = entry_link_target(entry, kind, &relative_path)?;
    Ok(ChapterTransportMember {
        relative_path,
        kind,
        mode,
        size_bytes,
        symlink_target,
    })
}

fn entry_link_target<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    kind: ChapterTransportEntryKind,
    relative_path: &str,
) -> Result<Option<String>, RunError> {
    let target = entry
        .link_name()
        .map_err(|error| transport_error(format!("reading tar link target for {relative_path}: {error}")))?;
    if kind != ChapterTransportEntryKind::Symlink {
        if target.is_some() {
            return Err(transport_error(format!("non-symlink tar member {relative_path} has a link target")));
        }
        return Ok(None);
    }
    let target = target.ok_or_else(|| transport_error(format!("symlink tar member {relative_path} has no target")))?;
    let target = target
        .to_str()
        .ok_or_else(|| transport_error(format!("tar link target is not valid UTF-8 for {relative_path}")))?;
    Ok(Some(target.to_string()))
}

fn entry_path<R: Read>(entry: &mut tar::Entry<'_, R>) -> Result<String, RunError> {
    let path = entry
        .path()
        .map_err(|error| transport_error(format!("reading chapter transport tar path: {error}")))?;
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| transport_error("chapter transport tar path is not valid UTF-8"))
}

fn require_entry_path<R: Read>(entry: &mut tar::Entry<'_, R>, expected: &str) -> Result<(), RunError> {
    let actual = entry_path(entry)?;
    if actual != expected {
        return Err(transport_error(format!("expected chapter zero entry {expected}, found {actual}")));
    }
    Ok(())
}

fn read_entry_bytes<R: Read>(entry: &mut tar::Entry<'_, R>, limit_bytes: u64) -> Result<Vec<u8>, RunError> {
    let expected_bytes = entry.size();
    if expected_bytes == 0 || expected_bytes > limit_bytes {
        return Err(transport_error(format!(
            "chapter transport control entry size {expected_bytes} is outside limit {limit_bytes}"
        )));
    }
    let capacity = usize::try_from(expected_bytes)
        .map_err(|_| transport_error("chapter transport control entry size overflowed usize"))?;
    let mut bytes = Vec::with_capacity(capacity);
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| transport_error(format!("reading chapter transport control entry: {error}")))?;
    if bytes.len() != capacity {
        return Err(transport_error(format!(
            "chapter transport control entry read {} bytes, expected {expected_bytes}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn drain_entry<R: Read>(entry: &mut tar::Entry<'_, R>, limit_bytes: u64) -> Result<(), RunError> {
    let expected_bytes = entry.size();
    if expected_bytes > limit_bytes || expected_bytes > CHAPTER_TRANSPORT_MAX_MEMBER_BYTES {
        return Err(transport_error(format!(
            "chapter transport entry size {expected_bytes} exceeds its validated bound"
        )));
    }
    let copied = std::io::copy(entry, &mut std::io::sink())
        .map_err(|error| transport_error(format!("draining chapter transport entry: {error}")))?;
    if copied != expected_bytes {
        return Err(transport_error(format!(
            "chapter transport entry yielded {copied} bytes, expected {expected_bytes}"
        )));
    }
    Ok(())
}

fn snapshot_verified_archive(
    archive_path: &Path,
    receipt: &ChapterTransportReceipt,
) -> Result<VerifiedArchiveSnapshot, RunError> {
    let mut source = open_file_nofollow(archive_path)?;
    let metadata = source
        .metadata()
        .map_err(|error| transport_error(format!("reading chapter transport archive metadata: {error}")))?;
    if !metadata.is_file() {
        return Err(transport_error(format!(
            "chapter transport archive is not a regular file: {}",
            archive_path.display()
        )));
    }
    if metadata.len() != receipt.archive_size_bytes || metadata.len() > CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES {
        return Err(transport_error(format!(
            "chapter transport archive size {} does not match receipt {} or exceeds its bound",
            metadata.len(),
            receipt.archive_size_bytes
        )));
    }
    let mut snapshot = tempfile::NamedTempFile::new()
        .map_err(|error| transport_error(format!("creating private chapter transport snapshot: {error}")))?;
    let observation = copy_and_hash_exact(&mut source, snapshot.as_file_mut(), metadata.len())?;
    if observation.digest_blake3 != receipt.archive_digest_blake3 {
        return Err(transport_error("chapter transport compressed archive BLAKE3 does not match receipt"));
    }
    if observation.chapter_marker_count != receipt.chapter_count {
        return Err(transport_error(format!(
            "chapter marker prefix count {} does not match receipt {}",
            observation.chapter_marker_count, receipt.chapter_count
        )));
    }
    snapshot
        .as_file_mut()
        .sync_all()
        .map_err(|error| transport_error(format!("syncing private chapter transport snapshot: {error}")))?;
    validate_complete_gzip(&snapshot, receipt.archive_size_bytes)?;
    snapshot
        .as_file_mut()
        .seek(SeekFrom::Start(0))
        .map_err(|error| transport_error(format!("rewinding private chapter transport snapshot: {error}")))?;
    debug_assert_eq!(metadata.len(), receipt.archive_size_bytes);
    debug_assert_eq!(observation.digest_blake3, receipt.archive_digest_blake3);
    debug_assert_eq!(observation.chapter_marker_count, receipt.chapter_count);
    Ok(VerifiedArchiveSnapshot { file: snapshot })
}

fn copy_and_hash_exact<R: Read, W: Write>(
    source: &mut R,
    destination: &mut W,
    expected_bytes: u64,
) -> Result<ArchiveCopyObservation, RunError> {
    let mut buffer = [0_u8; CHAPTER_TRANSPORT_COPY_BUFFER_BYTES];
    let mut copied_bytes = 0_u64;
    let mut hasher = blake3::Hasher::new();
    let mut marker_counter = ChapterMarkerPrefixCounter::default();
    while copied_bytes < expected_bytes {
        let remaining = expected_bytes
            .checked_sub(copied_bytes)
            .ok_or_else(|| transport_error("chapter transport snapshot remaining byte count underflowed"))?;
        let capacity = usize::try_from(remaining.min(CHAPTER_TRANSPORT_COPY_BUFFER_BYTES as u64))
            .map_err(|_| transport_error("chapter transport snapshot read capacity overflowed usize"))?;
        let count = source
            .read(&mut buffer[..capacity])
            .map_err(|error| transport_error(format!("reading chapter transport archive: {error}")))?;
        if count == 0 {
            return Err(transport_error(format!(
                "chapter transport archive ended after {copied_bytes} bytes, expected {expected_bytes}"
            )));
        }
        destination
            .write_all(&buffer[..count])
            .map_err(|error| transport_error(format!("writing private chapter transport snapshot: {error}")))?;
        hasher.update(&buffer[..count]);
        marker_counter.observe(&buffer[..count])?;
        copied_bytes = copied_bytes
            .checked_add(u64::try_from(count).map_err(|_| transport_error("snapshot read count overflowed u64"))?)
            .ok_or_else(|| transport_error("snapshot byte count overflowed u64"))?;
    }
    let mut trailing = [0_u8; 1];
    let trailing_count = source
        .read(&mut trailing)
        .map_err(|error| transport_error(format!("checking chapter transport archive growth: {error}")))?;
    if trailing_count != 0 {
        return Err(transport_error("chapter transport archive grew while it was measured"));
    }
    debug_assert_eq!(copied_bytes, expected_bytes);
    debug_assert_eq!(trailing_count, 0);
    Ok(ArchiveCopyObservation {
        digest_blake3: hasher.finalize().to_hex().to_string(),
        chapter_marker_count: marker_counter.marker_count,
    })
}

fn validate_complete_gzip(snapshot: &tempfile::NamedTempFile, archive_size_bytes: u64) -> Result<(), RunError> {
    let file = snapshot
        .reopen()
        .map_err(|error| transport_error(format!("reopening transport snapshot for gzip validation: {error}")))?;
    let mut decoder = flate2::bufread::GzDecoder::new(BufReader::new(file));
    let decoded_limit = CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES
        .checked_add(CHAPTER_TRANSPORT_MAX_GZIP_OVERHEAD_BYTES)
        .ok_or_else(|| transport_error("chapter transport gzip validation limit overflowed u64"))?;
    let read_limit = decoded_limit
        .checked_add(1)
        .ok_or_else(|| transport_error("chapter transport gzip validation read limit overflowed u64"))?;
    let decoded_bytes = std::io::copy(&mut decoder.by_ref().take(read_limit), &mut std::io::sink())
        .map_err(|error| transport_error(format!("validating complete chapter transport gzip stream: {error}")))?;
    if decoded_bytes > decoded_limit {
        return Err(transport_error(format!(
            "chapter transport gzip output exceeds the validation bound {decoded_limit}"
        )));
    }
    let mut buffered = decoder.into_inner();
    let consumed_bytes = buffered
        .stream_position()
        .map_err(|error| transport_error(format!("measuring validated chapter transport gzip stream: {error}")))?;
    if consumed_bytes != archive_size_bytes {
        return Err(transport_error(format!(
            "chapter transport gzip stream consumed {consumed_bytes} bytes, expected {archive_size_bytes}"
        )));
    }
    debug_assert!(decoded_bytes <= decoded_limit);
    debug_assert_eq!(consumed_bytes, archive_size_bytes);
    Ok(())
}

// r[impl mantle.release_provenance.chapter_transport.unpack]
pub(crate) fn unpack_release_transport(
    transport_dir: &Path,
    destination_dir: &Path,
) -> Result<ReleaseTransportUnpackOutcome, RunError> {
    ensure_destination_absent(destination_dir, "chapter transport unpack destination")?;
    validate_transport_directory_shape(transport_dir)?;
    let receipt_path = transport_dir.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME);
    let archive_path = transport_dir.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
    let receipt_bytes = read_bounded_file_nofollow(&receipt_path, CHAPTER_TRANSPORT_RECEIPT_BYTES_MAX)?;
    let receipt = parse_canonical_chapter_transport_receipt(&receipt_bytes)
        .map_err(|error| transport_error(format!("validating chapter transport receipt: {error}")))?;
    let snapshot = snapshot_verified_archive(&archive_path, &receipt)?;
    let control = read_control_chapter(&snapshot, &receipt)?;
    validate_all_archive_metadata(&snapshot, &control.index)?;
    let stage = create_sibling_stage(destination_dir, CHAPTER_TRANSPORT_UNPACK_STAGE_PREFIX)?;
    extract_snapshot(&snapshot, &control.index, stage.path())?;
    let verified_manifest = verify_release_evidence_bundle(stage.path())?;
    if verified_manifest != control.manifest {
        return Err(transport_error("unpacked release manifest does not match transported manifest"));
    }
    publish_stage_no_replace(stage, destination_dir)?;
    let outcome = ReleaseTransportUnpackOutcome {
        kind: CHAPTER_TRANSPORT_UNPACK_OUTPUT_KIND.to_string(),
        transport_dir: transport_dir.to_path_buf(),
        destination_dir: destination_dir.to_path_buf(),
        release_id: verified_manifest.release_id,
        archive_digest_blake3: receipt.archive_digest_blake3,
        chapter_count: receipt.chapter_count,
        member_count: receipt.member_count,
    };
    debug_assert_eq!(outcome.chapter_count, control.index.chapter_count);
    debug_assert_eq!(outcome.member_count, control.index.member_count);
    Ok(outcome)
}

fn extract_snapshot(
    snapshot: &VerifiedArchiveSnapshot,
    index: &ChapterTransportIndex,
    stage_path: &Path,
) -> Result<(), RunError> {
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeDestination, stage_path)
        .map_err(|error| transport_error(format!("opening private transport extraction stage: {error}")))?;
    if !root
        .is_empty()
        .map_err(|error| transport_error(format!("reading private transport extraction stage: {error}")))?
    {
        return Err(transport_error("private transport extraction stage is not empty"));
    }
    let file = snapshot
        .file
        .reopen()
        .map_err(|error| transport_error(format!("reopening transport snapshot for extraction: {error}")))?;
    let mut reader = TgzReader::open(file)
        .map_err(|error| transport_error(format!("opening transport snapshot for extraction: {error}")))?;
    let mut directories = Vec::new();
    for ordinal in 0..reader.chapters() {
        let expected = index
            .chapters
            .get(usize::try_from(ordinal).map_err(|_| transport_error("extract chapter ordinal overflowed usize"))?)
            .ok_or_else(|| transport_error(format!("extract chapter {ordinal} is absent from index")))?;
        extract_one_chapter(&mut reader, ordinal, expected, &root, &mut directories)?;
    }
    finalize_extracted_directory_modes(&root, &mut directories)?;
    debug_assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    debug_assert_eq!(reader.chapters(), index.chapter_count);
    Ok(())
}

fn extract_one_chapter<R: Read + Seek>(
    reader: &mut TgzReader<R>,
    ordinal: u32,
    expected: &ChapterTransportChapter,
    root: &ReleaseCapabilityRoot,
    directories: &mut Vec<(String, u32)>,
) -> Result<(), RunError> {
    let mut chapter = reader.jump_to_chapter(ordinal);
    let entries = chapter
        .entries()
        .map_err(|error| transport_error(format!("reading extraction chapter {ordinal}: {error}")))?;
    let mut member_index = 0_usize;
    let mut entry_index = 0_u32;
    for entry in entries {
        let mut entry = entry.map_err(|error| transport_error(format!("reading extraction entry: {error}")))?;
        let path = entry_path(&mut entry)?;
        if path == CHAPTER_TRANSPORT_RESERVED_INDEX_PATH {
            if ordinal != 0 || entry_index != CHAPTER_TRANSPORT_RESERVED_ENTRY_INDEX {
                return Err(transport_error("reserved index moved during transport extraction"));
            }
            drain_entry(&mut entry, CHAPTER_TRANSPORT_MAX_INDEX_BYTES)?;
        } else {
            let expected_member = expected
                .members
                .get(member_index)
                .ok_or_else(|| transport_error(format!("chapter {ordinal} has more members than its index")))?;
            let observed = member_from_entry(&mut entry, path)?;
            if &observed != expected_member {
                return Err(transport_error(format!("chapter {ordinal} member changed before extraction")));
            }
            extract_member(&mut entry, expected_member, root, directories)?;
            member_index = member_index
                .checked_add(1)
                .ok_or_else(|| transport_error("extraction member index overflowed usize"))?;
        }
        entry_index =
            entry_index.checked_add(1).ok_or_else(|| transport_error("extraction entry index overflowed u32"))?;
    }
    if member_index != expected.members.len() {
        return Err(transport_error(format!("chapter {ordinal} has fewer members than its index")));
    }
    debug_assert!(member_index <= usize::try_from(CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT).unwrap_or_default());
    debug_assert_eq!(expected.ordinal, ordinal);
    Ok(())
}

fn extract_member<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    member: &ChapterTransportMember,
    root: &ReleaseCapabilityRoot,
    directories: &mut Vec<(String, u32)>,
) -> Result<(), RunError> {
    let path = ValidatedReleasePath::new(&member.relative_path).map_err(|error| {
        transport_error(format!("invalid chapter transport extraction path {}: {error:?}", member.relative_path))
    })?;
    match member.kind {
        ChapterTransportEntryKind::Directory => {
            root.create_dir_all_relative_nofollow(ReleaseRootKind::ReleaseTreeDestination, &path)
                .map_err(|error| {
                    transport_error(format!("creating extracted directory {}: {error}", member.relative_path))
                })?;
            directories.push((member.relative_path.clone(), member.mode));
            drain_entry(entry, 0)
        }
        ChapterTransportEntryKind::File => extract_file(entry, member, root, &path),
        ChapterTransportEntryKind::Symlink => {
            let target = member
                .symlink_target
                .as_deref()
                .ok_or_else(|| transport_error(format!("extracted symlink {} has no target", member.relative_path)))?;
            root.create_symlink_nofollow(&path, target).map_err(|error| {
                transport_error(format!("creating extracted symlink {}: {error}", member.relative_path))
            })?;
            drain_entry(entry, 0)
        }
        ChapterTransportEntryKind::Unsupported => {
            Err(transport_error(format!("unsupported extracted member kind for {}", member.relative_path)))
        }
    }
}

fn extract_file<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    member: &ChapterTransportMember,
    root: &ReleaseCapabilityRoot,
    path: &ValidatedReleasePath,
) -> Result<(), RunError> {
    let mut file = root
        .open_new_file_nofollow(path)
        .map_err(|error| transport_error(format!("creating extracted file {}: {error}", member.relative_path)))?;
    let copied = std::io::copy(entry, &mut file)
        .map_err(|error| transport_error(format!("writing extracted file {}: {error}", member.relative_path)))?;
    if copied != member.size_bytes {
        return Err(transport_error(format!(
            "extracted file {} wrote {copied} bytes, expected {}",
            member.relative_path, member.size_bytes
        )));
    }
    file.flush()
        .map_err(|error| transport_error(format!("flushing extracted file {}: {error}", member.relative_path)))?;
    set_extracted_file_mode(&file, member.mode, &member.relative_path)?;
    debug_assert_eq!(copied, member.size_bytes);
    debug_assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    Ok(())
}

fn finalize_extracted_directory_modes(
    root: &ReleaseCapabilityRoot,
    directories: &mut [(String, u32)],
) -> Result<(), RunError> {
    directories
        .sort_by(|left, right| path_depth(&right.0).cmp(&path_depth(&left.0)).then_with(|| right.0.cmp(&left.0)));
    for (relative_path, mode) in directories.iter() {
        let path = ValidatedReleasePath::new(relative_path)
            .map_err(|error| transport_error(format!("invalid extracted directory path {relative_path}: {error:?}")))?;
        let dir = root
            .create_dir_all_relative_nofollow(ReleaseRootKind::ReleaseTreeDestination, &path)
            .map_err(|error| transport_error(format!("opening extracted directory {relative_path}: {error}")))?;
        set_extracted_directory_mode(dir.dir(), *mode, relative_path)?;
    }
    debug_assert!(directories.windows(2).all(|pair| path_depth(&pair[0].0) >= path_depth(&pair[1].0)));
    debug_assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    Ok(())
}

fn path_depth(path: &str) -> u32 {
    path.bytes().fold(1_u32, |depth, byte| if byte == b'/' { depth.saturating_add(1) } else { depth })
}

#[cfg(unix)]
fn set_extracted_file_mode(file: &cap_std::fs::File, mode: u32, relative_path: &str) -> Result<(), RunError> {
    use cap_std::fs::Permissions;
    use cap_std::fs::PermissionsExt;

    file.set_permissions(Permissions::from_mode(mode & UNIX_PERMISSION_BITS_MASK))
        .map_err(|error| transport_error(format!("setting extracted file mode for {relative_path}: {error}")))
}

#[cfg(not(unix))]
fn set_extracted_file_mode(_file: &cap_std::fs::File, _mode: u32, _relative_path: &str) -> Result<(), RunError> {
    Ok(())
}

#[cfg(unix)]
fn set_extracted_directory_mode(dir: &cap_std::fs::Dir, mode: u32, relative_path: &str) -> Result<(), RunError> {
    use cap_std::fs::Permissions;
    use cap_std::fs::PermissionsExt;

    dir.set_permissions(".", Permissions::from_mode(mode & UNIX_PERMISSION_BITS_MASK))
        .map_err(|error| transport_error(format!("setting extracted directory mode for {relative_path}: {error}")))
}

#[cfg(not(unix))]
fn set_extracted_directory_mode(_dir: &cap_std::fs::Dir, _mode: u32, _relative_path: &str) -> Result<(), RunError> {
    Ok(())
}

fn validate_transport_directory_shape(transport_dir: &Path) -> Result<(), RunError> {
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, transport_dir).map_err(
        |error| transport_error(format!("opening chapter transport directory {}: {error}", transport_dir.display())),
    )?;
    let mut names = root
        .dir()
        .entries()
        .map_err(|error| transport_error(format!("reading chapter transport directory: {error}")))?
        .map(|entry| {
            entry
                .map_err(|error| transport_error(format!("reading chapter transport directory entry: {error}")))?
                .file_name()
                .into_string()
                .map_err(|_| transport_error("chapter transport directory entry is not valid UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let mut expected = vec![
        CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME.to_string(),
        CHAPTER_TRANSPORT_RECEIPT_FILE_NAME.to_string(),
    ];
    expected.sort();
    if names != expected {
        return Err(transport_error(format!(
            "chapter transport directory must contain only {} and {}",
            CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME, CHAPTER_TRANSPORT_RECEIPT_FILE_NAME
        )));
    }
    debug_assert_eq!(names.len(), CHAPTER_TRANSPORT_DIRECTORY_ENTRY_COUNT);
    debug_assert_eq!(root.kind(), ReleaseRootKind::ReleaseEvidence);
    Ok(())
}

fn read_bounded_file_nofollow(path: &Path, limit_bytes: u64) -> Result<Vec<u8>, RunError> {
    let mut file = open_file_nofollow(path)?;
    let metadata = file
        .metadata()
        .map_err(|error| transport_error(format!("reading no-follow file metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > limit_bytes {
        return Err(transport_error(format!(
            "file {} size {} is outside the supported bound {limit_bytes}",
            path.display(),
            metadata.len()
        )));
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| transport_error(format!("file size overflowed usize for {}", path.display())))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes)
        .map_err(|error| transport_error(format!("reading no-follow file {}: {error}", path.display())))?;
    if bytes.len() != capacity {
        return Err(transport_error(format!("file {} changed size while reading", path.display())));
    }
    Ok(bytes)
}

fn open_file_nofollow(path: &Path) -> Result<cap_std::fs::File, RunError> {
    let absolute = std::path::absolute(path)
        .map_err(|error| transport_error(format!("resolving file path {}: {error}", path.display())))?;
    let parent = absolute
        .parent()
        .ok_or_else(|| transport_error(format!("file path has no parent: {}", absolute.display())))?;
    let file_name = absolute
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| transport_error(format!("file name is not valid UTF-8: {}", absolute.display())))?;
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeSource, parent)
        .map_err(|error| transport_error(format!("opening no-follow file parent {}: {error}", parent.display())))?;
    let relative = ValidatedReleasePath::new(file_name)
        .map_err(|error| transport_error(format!("invalid file name {file_name}: {error:?}")))?;
    let file = root
        .open_file_read_nofollow(&relative)
        .map_err(|error| transport_error(format!("opening no-follow file {}: {error}", absolute.display())))?;
    debug_assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeSource);
    debug_assert!(!relative.as_str().is_empty());
    Ok(file)
}

fn create_sibling_stage(destination: &Path, prefix: &str) -> Result<tempfile::TempDir, RunError> {
    let absolute = std::path::absolute(destination).map_err(|error| {
        transport_error(format!("resolving transport destination {}: {error}", destination.display()))
    })?;
    let parent = absolute
        .parent()
        .ok_or_else(|| transport_error(format!("transport destination has no parent: {}", absolute.display())))?;
    ReleaseCapabilityRoot::create_ambient_dir_all_nofollow(ReleaseRootKind::ReleaseTreeDestination, parent).map_err(
        |error| transport_error(format!("opening transport destination parent {}: {error}", parent.display())),
    )?;
    let stage = tempfile::Builder::new().prefix(prefix).tempdir_in(parent).map_err(|error| {
        transport_error(format!("creating private transport stage in {}: {error}", parent.display()))
    })?;
    debug_assert_eq!(stage.path().parent(), Some(parent));
    debug_assert_ne!(stage.path(), absolute);
    Ok(stage)
}

fn publish_stage_no_replace(stage: tempfile::TempDir, destination: &Path) -> Result<(), RunError> {
    let stage_path = stage.path().to_path_buf();
    let absolute_destination = std::path::absolute(destination)
        .map_err(|error| transport_error(format!("resolving transport publication destination: {error}")))?;
    crate::linux_rename::rename_path_no_replace(&stage_path, &absolute_destination).map_err(|error| {
        transport_error(format!(
            "publishing transport output without replacement to {}: {error}",
            absolute_destination.display()
        ))
    })?;
    let _kept_stage_path = stage.keep();
    sync_parent_directory(&absolute_destination)?;
    debug_assert!(!stage_path.exists());
    debug_assert!(absolute_destination.is_dir());
    Ok(())
}

fn sync_parent_directory(path: &Path) -> Result<(), RunError> {
    let parent = path
        .parent()
        .ok_or_else(|| transport_error(format!("published path has no parent: {}", path.display())))?;
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| transport_error(format!("syncing published transport parent {}: {error}", parent.display())))
}

fn ensure_destination_absent(path: &Path, label: &str) -> Result<(), RunError> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(transport_error(format!("observing {label} {}: {error}", path.display()))),
        Ok(_) => Err(transport_error(format!("{label} must be absent: {}", path.display()))),
    }
}

fn transport_error(message: impl Into<String>) -> RunError {
    RunError::Internal(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERIC_READER_EXPECTED_MINIMUM_MEMBERS: usize = 4;
    const PRIVILEGED_FILE_MODE: u32 = 0o4_755;
    const MODE_TAMPER_MASK: u32 = 0o100;

    #[derive(Clone, Copy)]
    enum HeaderTamper {
        Mode,
        SpecialType,
    }

    fn create_verified_bundle(root: &Path, name: &str) -> PathBuf {
        let request = crate::release_evidence::tests::publication_fixture(root, name, name);
        let bundle = request.bundle_dir.clone();
        crate::release_evidence::create_release_evidence_bundle(&request).unwrap();
        bundle
    }

    fn write_transport_with_untrusted_index(transport: &Path, index: &ChapterTransportIndex) {
        const MANIFEST_BYTES: &[u8] = b"{}";
        std::fs::create_dir(transport).unwrap();
        let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let file = std::fs::File::create(&archive_path).unwrap();
        let mut writer = TgzWriter::new(file, Compression::new(CHAPTER_TRANSPORT_GZIP_LEVEL));
        let mut chapter = writer.create_chapter();
        let index_bytes = serde_json::to_vec(index).unwrap();
        append_transport_index(&mut chapter, &index_bytes).unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(MANIFEST_BYTES.len()).unwrap());
        header.set_mode(CHAPTER_TRANSPORT_INDEX_MODE);
        header.set_cksum();
        chapter.append_data(&mut header, CHAPTER_TRANSPORT_MANIFEST_PATH, MANIFEST_BYTES).unwrap();
        chapter.finish().unwrap();
        drop(chapter);
        writer.into_inner().unwrap();
        let (archive_size_bytes, archive_digest_blake3) = hash_file_nofollow(&archive_path).unwrap();
        let receipt = build_chapter_transport_receipt(ChapterTransportReceiptInput {
            archive_size_bytes,
            archive_digest_blake3,
            source_manifest_digest_blake3: index.source_manifest_digest_blake3.clone(),
            transport_index_digest_blake3: blake3::hash(&index_bytes).to_hex().to_string(),
            chapter_count: index.chapter_count,
            member_count: index.member_count,
        })
        .unwrap();
        write_receipt_file(&transport.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME), &receipt).unwrap();
    }

    fn rewrite_archive_header(source_path: &Path, destination_path: &Path, target_path: &str, tamper: HeaderTamper) {
        let mut reader = TgzReader::open(std::fs::File::open(source_path).unwrap()).unwrap();
        let output = std::fs::File::create(destination_path).unwrap();
        let mut writer = TgzWriter::new(output, Compression::new(CHAPTER_TRANSPORT_GZIP_LEVEL));
        let mut tampered = false;
        for ordinal in 0..reader.chapters() {
            let mut source_chapter = reader.jump_to_chapter(ordinal);
            let entries = source_chapter.entries().unwrap();
            let mut destination_chapter = writer.create_chapter();
            for entry in entries {
                let mut entry = entry.unwrap();
                let path = entry.path().unwrap().into_owned();
                let mut header = entry.header().clone();
                if path == Path::new(target_path) {
                    match tamper {
                        HeaderTamper::Mode => {
                            let original_mode = header.mode().unwrap();
                            header.set_mode(original_mode ^ MODE_TAMPER_MASK);
                        }
                        HeaderTamper::SpecialType => header.set_entry_type(tar::EntryType::Fifo),
                    }
                    header.set_cksum();
                    tampered = true;
                }
                destination_chapter.append(&header, &mut entry).unwrap();
            }
            destination_chapter.finish().unwrap();
        }
        writer.into_inner().unwrap();
        assert!(tampered);
    }

    fn rebind_receipt_to_archive(transport: &Path) {
        let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let receipt_path = transport.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME);
        let receipt_bytes = std::fs::read(&receipt_path).unwrap();
        let mut receipt = parse_canonical_chapter_transport_receipt(&receipt_bytes).unwrap();
        let (archive_size_bytes, archive_digest_blake3) = hash_file_nofollow(&archive_path).unwrap();
        receipt.archive_size_bytes = archive_size_bytes;
        receipt.archive_digest_blake3 = archive_digest_blake3;
        std::fs::write(receipt_path, canonical_chapter_transport_receipt(&receipt).unwrap()).unwrap();
    }

    fn invalid_index_member(
        relative_path: &str,
        kind: ChapterTransportEntryKind,
        size_bytes: u64,
        symlink_target: Option<&str>,
    ) -> ChapterTransportIndex {
        let manifest_digest = blake3::hash(b"{}").to_hex().to_string();
        let members = vec![
            ChapterTransportMember {
                relative_path: CHAPTER_TRANSPORT_MANIFEST_PATH.to_string(),
                kind: ChapterTransportEntryKind::File,
                mode: CHAPTER_TRANSPORT_INDEX_MODE,
                size_bytes: 1,
                symlink_target: None,
            },
            ChapterTransportMember {
                relative_path: relative_path.to_string(),
                kind,
                mode: CHAPTER_TRANSPORT_INDEX_MODE,
                size_bytes,
                symlink_target: symlink_target.map(ToString::to_string),
            },
        ];
        ChapterTransportIndex {
            schema: crunch_release_core::CHAPTER_TRANSPORT_INDEX_SCHEMA.to_string(),
            format: crunch_release_core::CHAPTER_TRANSPORT_FORMAT.to_string(),
            source_manifest_digest_blake3: manifest_digest,
            chapter_count: 1,
            member_count: u32::try_from(members.len()).unwrap(),
            total_uncompressed_file_bytes: members
                .iter()
                .filter(|member| member.kind == ChapterTransportEntryKind::File)
                .map(|member| member.size_bytes)
                .sum(),
            chapters: vec![ChapterTransportChapter {
                ordinal: 0,
                role: crunch_release_core::ChapterTransportRole::Control,
                group: "control".to_string(),
                uncompressed_file_bytes: 1,
                members,
            }],
        }
    }

    // r[verify mantle.release_provenance.chapter_transport.pack]
    // r[verify mantle.release_provenance.chapter_transport.inspect]
    #[test]
    fn pack_is_deterministic_and_standard_readers_see_every_member() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = create_verified_bundle(temp.path(), "chapter-pack");
        let first = temp.path().join("first-transport");
        let second = temp.path().join("second-transport");

        let first_outcome = pack_release_transport(&bundle, &first).unwrap();
        let second_outcome = pack_release_transport(&bundle, &second).unwrap();
        let first_bytes = std::fs::read(&first_outcome.archive_path).unwrap();
        let second_bytes = std::fs::read(&second_outcome.archive_path).unwrap();
        let inspection = inspect_release_transport(&first).unwrap();
        let decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(&first_bytes));
        let mut archive = tar::Archive::new(decoder);
        let paths = archive
            .entries()
            .unwrap()
            .map(|entry| entry.unwrap().path().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_outcome.kind, CHAPTER_TRANSPORT_PACK_OUTPUT_KIND);
        assert_eq!(first_outcome.receipt, second_outcome.receipt);
        assert_eq!(inspection.kind, CHAPTER_TRANSPORT_INSPECTION_OUTPUT_KIND);
        assert_eq!(inspection.release_id, "chapter-pack");
        assert_eq!(paths[0], CHAPTER_TRANSPORT_RESERVED_INDEX_PATH);
        assert!(paths.contains(&CHAPTER_TRANSPORT_MANIFEST_PATH.to_string()));
        assert!(paths.len() >= GENERIC_READER_EXPECTED_MINIMUM_MEMBERS);
    }

    // r[verify mantle.release_provenance.chapter_transport.unpack]
    #[test]
    #[cfg(unix)]
    fn unpack_round_trip_preserves_verified_tree_and_internal_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let request =
            crate::release_evidence::tests::publication_fixture(temp.path(), "chapter-round-trip", "round-trip");
        symlink("manifest.json", request.proof_bundle_dir.join("manifest-link.json")).unwrap();
        let bundle = request.bundle_dir.clone();
        crate::release_evidence::create_release_evidence_bundle(&request).unwrap();
        let transport = temp.path().join("transport");
        let unpacked = temp.path().join("unpacked");

        pack_release_transport(&bundle, &transport).unwrap();
        let outcome = unpack_release_transport(&transport, &unpacked).unwrap();
        let original_identity = crate::release_tree_copy::hash_directory_tree(&bundle).unwrap();
        let unpacked_identity = crate::release_tree_copy::hash_directory_tree(&unpacked).unwrap();

        assert_eq!(outcome.kind, CHAPTER_TRANSPORT_UNPACK_OUTPUT_KIND);
        assert_eq!(outcome.release_id, "chapter-round-trip");
        assert_eq!(original_identity, unpacked_identity);
        assert_eq!(
            std::fs::read_link(unpacked.join("proof/self-hosting/manifest-link.json")).unwrap(),
            Path::new("manifest.json")
        );
        assert!(!unpacked.join(CHAPTER_TRANSPORT_RESERVED_INDEX_PATH).exists());
    }

    // r[verify mantle.release_provenance.chapter_transport.receipt]
    #[test]
    fn truncation_and_receipt_replacement_fail_before_unpack_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = create_verified_bundle(temp.path(), "chapter-negative");
        let transport = temp.path().join("transport");
        let unpacked = temp.path().join("unpacked");
        pack_release_transport(&bundle, &transport).unwrap();
        let archive = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let mut bytes = std::fs::read(&archive).unwrap();
        bytes.pop();
        std::fs::write(&archive, bytes).unwrap();

        let inspect_error = inspect_release_transport(&transport).unwrap_err();
        let unpack_error = unpack_release_transport(&transport, &unpacked).unwrap_err();

        assert!(inspect_error.to_string().contains("size") || inspect_error.to_string().contains("BLAKE3"));
        assert!(unpack_error.to_string().contains("size") || unpack_error.to_string().contains("BLAKE3"));
        assert!(!unpacked.exists());
    }

    #[test]
    fn compressed_payload_tamper_fails_digest_validation() {
        const MIDDLE_DIVISOR: usize = 2;
        const TAMPER_MASK: u8 = 0b0000_0001;

        let temp = tempfile::tempdir().unwrap();
        let bundle = create_verified_bundle(temp.path(), "chapter-tamper-negative");
        let transport = temp.path().join("transport");
        pack_release_transport(&bundle, &transport).unwrap();
        let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let mut archive_bytes = std::fs::read(&archive_path).unwrap();
        let middle = archive_bytes.len() / MIDDLE_DIVISOR;
        archive_bytes[middle] ^= TAMPER_MASK;
        std::fs::write(&archive_path, archive_bytes).unwrap();

        let error = inspect_release_transport(&transport).unwrap_err();

        assert!(error.to_string().contains("BLAKE3"), "{error}");
    }

    #[test]
    fn truncated_gzip_with_rebound_receipt_fails_complete_stream_validation() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = create_verified_bundle(temp.path(), "chapter-gzip-negative");
        let transport = temp.path().join("transport");
        pack_release_transport(&bundle, &transport).unwrap();
        let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let receipt_path = transport.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME);
        let mut archive_bytes = std::fs::read(&archive_path).unwrap();
        archive_bytes.pop();
        std::fs::write(&archive_path, &archive_bytes).unwrap();
        let receipt_bytes = std::fs::read(&receipt_path).unwrap();
        let mut receipt = parse_canonical_chapter_transport_receipt(&receipt_bytes).unwrap();
        receipt.archive_size_bytes = u64::try_from(archive_bytes.len()).unwrap();
        receipt.archive_digest_blake3 = blake3::hash(&archive_bytes).to_hex().to_string();
        let rebound_receipt = canonical_chapter_transport_receipt(&receipt).unwrap();
        std::fs::write(&receipt_path, rebound_receipt).unwrap();

        let error = inspect_release_transport(&transport).unwrap_err();

        assert!(error.to_string().contains("gzip stream"), "{error}");
    }

    #[test]
    fn marker_counter_handles_chunk_boundaries_and_rejects_excess_markers() {
        let mut prefix = vec![CHAPTER_MARKER_NONFINAL_START];
        prefix.extend_from_slice(&CHAPTER_MARKER_SUFFIX);
        let mut counter = ChapterMarkerPrefixCounter::default();
        counter.observe(&prefix[..1]).unwrap();
        counter.observe(&prefix[1..]).unwrap();
        assert_eq!(counter.marker_count, 1);

        let mut excessive = ChapterMarkerPrefixCounter::default();
        for _ in 0..CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT {
            excessive.observe(&prefix).unwrap();
        }
        let error = excessive.observe(&prefix).unwrap_err();
        assert!(error.to_string().contains("exceeds the transport limit"));
    }

    // r[verify mantle.release_provenance.chapter_transport.inspect]
    #[test]
    fn ordinary_tgz_without_reserved_index_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let transport = temp.path().join("legacy-transport");
        std::fs::create_dir(&transport).unwrap();
        let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let encoder = flate2::GzBuilder::new().mtime(0).write(
            std::fs::File::create(&archive_path).unwrap(),
            flate2::Compression::new(CHAPTER_TRANSPORT_GZIP_LEVEL),
        );
        let mut archive = tar::Builder::new(encoder);
        let manifest_bytes = b"{}";
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(manifest_bytes.len()).unwrap());
        header.set_mode(CHAPTER_TRANSPORT_INDEX_MODE);
        header.set_cksum();
        archive
            .append_data(&mut header, CHAPTER_TRANSPORT_MANIFEST_PATH, manifest_bytes.as_slice())
            .unwrap();
        archive.finish().unwrap();
        archive.into_inner().unwrap().finish().unwrap();
        let (archive_size_bytes, archive_digest_blake3) = hash_file_nofollow(&archive_path).unwrap();
        let receipt = build_chapter_transport_receipt(ChapterTransportReceiptInput {
            archive_size_bytes,
            archive_digest_blake3,
            source_manifest_digest_blake3: blake3::hash(manifest_bytes).to_hex().to_string(),
            transport_index_digest_blake3: blake3::hash(b"missing-index").to_hex().to_string(),
            chapter_count: 1,
            member_count: 1,
        })
        .unwrap();
        write_receipt_file(&transport.join(CHAPTER_TRANSPORT_RECEIPT_FILE_NAME), &receipt).unwrap();

        let error = inspect_release_transport(&transport).unwrap_err();

        assert!(error.to_string().contains("marker prefix count"), "{error}");
        assert_eq!(TgzReader::open(std::fs::File::open(archive_path).unwrap()).unwrap().chapters(), 1);
    }

    // r[verify mantle.release_provenance.chapter_transport.unpack]
    #[test]
    fn stale_metadata_and_special_tar_type_fail_with_rebound_receipts() {
        let temp = tempfile::tempdir().unwrap();
        let cases = [HeaderTamper::Mode, HeaderTamper::SpecialType];
        for (case_index, tamper) in cases.into_iter().enumerate() {
            let bundle = create_verified_bundle(temp.path(), &format!("chapter-header-{case_index}"));
            let transport = temp.path().join(format!("transport-{case_index}"));
            pack_release_transport(&bundle, &transport).unwrap();
            let archive_path = transport.join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
            let replacement_path = transport.join(format!("replacement-{case_index}.tgz"));
            rewrite_archive_header(&archive_path, &replacement_path, CHAPTER_TRANSPORT_MANIFEST_PATH, tamper);
            std::fs::remove_file(&archive_path).unwrap();
            std::fs::rename(&replacement_path, &archive_path).unwrap();
            rebind_receipt_to_archive(&transport);

            let error = inspect_release_transport(&transport).unwrap_err();

            assert!(
                error.to_string().contains("members do not match")
                    || error.to_string().contains("unsupported chapter transport tar entry type"),
                "{error}"
            );
        }
    }

    // r[verify mantle.release_provenance.chapter_transport.unpack]
    #[test]
    fn unsafe_index_matrix_fails_before_unpack_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let mut privileged = invalid_index_member("privileged", ChapterTransportEntryKind::File, 1, None);
        privileged.chapters[0].members[1].mode = PRIVILEGED_FILE_MODE;
        let cases = [
            invalid_index_member("../escape", ChapterTransportEntryKind::File, 1, None),
            invalid_index_member("/absolute", ChapterTransportEntryKind::File, 1, None),
            invalid_index_member("link", ChapterTransportEntryKind::Symlink, 0, Some("../outside")),
            invalid_index_member(
                "oversized",
                ChapterTransportEntryKind::File,
                CHAPTER_TRANSPORT_MAX_MEMBER_BYTES.saturating_add(1),
                None,
            ),
            invalid_index_member("unsupported", ChapterTransportEntryKind::Unsupported, 0, None),
            privileged,
        ];
        for (case_index, index) in cases.iter().enumerate() {
            let transport = temp.path().join(format!("transport-{case_index}"));
            let destination = temp.path().join(format!("destination-{case_index}"));
            write_transport_with_untrusted_index(&transport, index);

            let inspect_error = inspect_release_transport(&transport).unwrap_err();
            let unpack_error = unpack_release_transport(&transport, &destination).unwrap_err();

            assert!(inspect_error.to_string().contains("transport index"), "{inspect_error}");
            assert!(unpack_error.to_string().contains("transport index"), "{unpack_error}");
            assert!(!destination.exists());
        }
    }

    #[cfg(unix)]
    struct PositionedFile {
        file: std::sync::Arc<std::fs::File>,
        position: u64,
    }

    #[cfg(unix)]
    impl PositionedFile {
        fn open(path: &Path) -> std::io::Result<Self> {
            Ok(Self {
                file: std::sync::Arc::new(std::fs::File::open(path)?),
                position: 0,
            })
        }
    }

    #[cfg(unix)]
    impl Read for PositionedFile {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            use std::os::unix::fs::FileExt;

            let count = self.file.read_at(buffer, self.position)?;
            self.position = self
                .position
                .checked_add(u64::try_from(count).map_err(std::io::Error::other)?)
                .ok_or_else(|| std::io::Error::other("positioned chapter read offset overflowed u64"))?;
            Ok(count)
        }
    }

    #[cfg(unix)]
    impl Seek for PositionedFile {
        fn seek(&mut self, seek: SeekFrom) -> std::io::Result<u64> {
            let length = self.file.metadata()?.len();
            self.position = resolve_positioned_seek(self.position, length, seek)?;
            Ok(self.position)
        }
    }

    #[cfg(unix)]
    impl chapter_tgz::IndependentRead for PositionedFile {
        fn independent_clone(&self) -> std::io::Result<Self> {
            Ok(Self {
                file: self.file.clone(),
                position: self.position,
            })
        }
    }

    #[cfg(unix)]
    fn resolve_positioned_seek(current: u64, length: u64, seek: SeekFrom) -> std::io::Result<u64> {
        match seek {
            SeekFrom::Start(position) => Ok(position),
            SeekFrom::Current(delta) => add_positioned_seek_delta(current, delta),
            SeekFrom::End(delta) => add_positioned_seek_delta(length, delta),
        }
    }

    #[cfg(unix)]
    fn add_positioned_seek_delta(base: u64, delta: i64) -> std::io::Result<u64> {
        let position = i128::from(base) + i128::from(delta);
        u64::try_from(position).map_err(|_| std::io::Error::other("positioned chapter seek is outside u64"))
    }

    #[cfg(unix)]
    fn drain_tar_archive<R: Read>(archive: &mut tar::Archive<R>) -> std::io::Result<u64> {
        let mut payload_bytes = 0_u64;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let copied = std::io::copy(&mut entry, &mut std::io::sink())?;
            payload_bytes = payload_bytes
                .checked_add(copied)
                .ok_or_else(|| std::io::Error::other("chapter payload byte count overflowed u64"))?;
        }
        Ok(payload_bytes)
    }

    #[cfg(unix)]
    fn drain_chapters_sequentially(archive_path: &Path) -> std::io::Result<u64> {
        let mut reader = TgzReader::open(PositionedFile::open(archive_path)?)?;
        let mut payload_bytes = 0_u64;
        for ordinal in 0..reader.chapters() {
            let mut chapter = reader.jump_to_chapter(ordinal);
            payload_bytes = payload_bytes
                .checked_add(drain_tar_archive(&mut chapter)?)
                .ok_or_else(|| std::io::Error::other("sequential chapter payload count overflowed u64"))?;
        }
        Ok(payload_bytes)
    }

    #[cfg(unix)]
    fn drain_chapters_in_parallel(archive_path: &Path) -> std::io::Result<u64> {
        let reader = TgzReader::open(PositionedFile::open(archive_path)?)?;
        std::thread::scope(|scope| {
            let mut workers = Vec::with_capacity(usize::try_from(reader.chapters()).unwrap_or_default());
            for ordinal in 0..reader.chapters() {
                let reader_ref = &reader;
                workers.push(scope.spawn(move || -> std::io::Result<u64> {
                    let mut chapter = reader_ref.independent_read_chapter(ordinal)?;
                    drain_tar_archive(&mut chapter)
                }));
            }
            let mut payload_bytes = 0_u64;
            for worker in workers {
                let chapter_bytes = worker
                    .join()
                    .map_err(|_| std::io::Error::other("parallel chapter benchmark worker panicked"))??;
                payload_bytes = payload_bytes
                    .checked_add(chapter_bytes)
                    .ok_or_else(|| std::io::Error::other("parallel chapter payload count overflowed u64"))?;
            }
            Ok(payload_bytes)
        })
    }

    #[cfg(unix)]
    fn write_benchmark_source(path: &Path) {
        const CHUNK_BYTES: usize = 1_024 * 1_024;
        const CHUNK_COUNT: u32 = 64;
        const XOR_SHIFT_LEFT: u32 = 13;
        const XOR_SHIFT_RIGHT: u32 = 7;
        const XOR_SHIFT_FINAL: u32 = 17;
        const XOR_SEED: u64 = 0x4d61_6e74_6c65_7631;

        let mut state = XOR_SEED;
        let mut chunk = vec![0_u8; CHUNK_BYTES];
        for byte in &mut chunk {
            state ^= state << XOR_SHIFT_LEFT;
            state ^= state >> XOR_SHIFT_RIGHT;
            state ^= state << XOR_SHIFT_FINAL;
            *byte = state.to_le_bytes()[0];
        }
        let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        for _ in 0..CHUNK_COUNT {
            file.write_all(&chunk).unwrap();
        }
        file.flush().unwrap();
        file.into_inner().unwrap().sync_all().unwrap();
    }

    fn median_micros(mut samples: Vec<u128>) -> u128 {
        const MEDIAN_DIVISOR: usize = 2;

        assert!(!samples.is_empty());
        samples.sort_unstable();
        samples[samples.len() / MEDIAN_DIVISOR]
    }

    #[cfg(unix)]
    #[test]
    fn positioned_file_seek_math_accepts_valid_offsets_and_rejects_underflow() {
        const CURRENT_POSITION: u64 = 8;
        const FILE_LENGTH: u64 = 16;
        const FORWARD_DELTA: i64 = 4;
        const BACKWARD_DELTA: i64 = -4;
        const UNDERFLOW_DELTA: i64 = -17;

        assert_eq!(
            resolve_positioned_seek(CURRENT_POSITION, FILE_LENGTH, SeekFrom::Current(FORWARD_DELTA)).unwrap(),
            CURRENT_POSITION + u64::try_from(FORWARD_DELTA).unwrap()
        );
        assert_eq!(
            resolve_positioned_seek(CURRENT_POSITION, FILE_LENGTH, SeekFrom::End(BACKWARD_DELTA)).unwrap(),
            FILE_LENGTH - BACKWARD_DELTA.unsigned_abs()
        );
        assert!(resolve_positioned_seek(CURRENT_POSITION, FILE_LENGTH, SeekFrom::End(UNDERFLOW_DELTA)).is_err());
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "representative release transport benchmark"]
    fn representative_release_transport_benchmark() {
        const SAMPLE_COUNT: u32 = 3;

        let temp = tempfile::tempdir().unwrap();
        let request =
            crate::release_evidence::tests::publication_fixture(temp.path(), "chapter-benchmark", "benchmark");
        write_benchmark_source(&request.source_archive_path);
        let bundle = request.bundle_dir.clone();
        crate::release_evidence::create_release_evidence_bundle(&request).unwrap();

        let mut pack_samples = Vec::new();
        let mut transports = Vec::new();
        for sample in 0..SAMPLE_COUNT {
            let transport = temp.path().join(format!("transport-{sample}"));
            let started = std::time::Instant::now();
            pack_release_transport(&bundle, &transport).unwrap();
            pack_samples.push(started.elapsed().as_micros());
            transports.push(transport);
        }
        let archive_path = transports[0].join(CHAPTER_TRANSPORT_ARCHIVE_FILE_NAME);
        let mut inspect_samples = Vec::new();
        let mut unpack_samples = Vec::new();
        let mut sequential_samples = Vec::new();
        let mut parallel_samples = Vec::new();
        let mut generic_gzip_samples = Vec::new();
        let mut control_access_samples = Vec::new();
        let mut expected_payload_bytes = None;
        for sample in 0..SAMPLE_COUNT {
            let inspect_started = std::time::Instant::now();
            let inspection = inspect_release_transport(&transports[0]).unwrap();
            inspect_samples.push(inspect_started.elapsed().as_micros());

            let unpacked = temp.path().join(format!("unpacked-{sample}"));
            let unpack_started = std::time::Instant::now();
            unpack_release_transport(&transports[0], &unpacked).unwrap();
            unpack_samples.push(unpack_started.elapsed().as_micros());

            let sequential_started = std::time::Instant::now();
            let sequential_payload = drain_chapters_sequentially(&archive_path).unwrap();
            sequential_samples.push(sequential_started.elapsed().as_micros());

            let parallel_started = std::time::Instant::now();
            let parallel_payload = drain_chapters_in_parallel(&archive_path).unwrap();
            parallel_samples.push(parallel_started.elapsed().as_micros());
            assert_eq!(parallel_payload, sequential_payload);
            assert!(sequential_payload >= inspection.index.total_uncompressed_file_bytes);
            assert!(expected_payload_bytes.is_none_or(|expected| expected == sequential_payload));
            expected_payload_bytes = Some(sequential_payload);

            let generic_started = std::time::Instant::now();
            let decoder = flate2::read::GzDecoder::new(std::fs::File::open(&archive_path).unwrap());
            std::io::copy(&mut std::io::BufReader::new(decoder), &mut std::io::sink()).unwrap();
            generic_gzip_samples.push(generic_started.elapsed().as_micros());

            let control_started = std::time::Instant::now();
            let mut reader = TgzReader::open(PositionedFile::open(&archive_path).unwrap()).unwrap();
            let mut control = reader.jump_to_chapter(0);
            drain_tar_archive(&mut control).unwrap();
            control_access_samples.push(control_started.elapsed().as_micros());
        }

        let report = serde_json::json!({
            "kind": "mantle-release-chapter-transport-benchmark-v1",
            "sample_count": SAMPLE_COUNT,
            "source_bytes": std::fs::metadata(&request.source_archive_path).unwrap().len(),
            "archive_bytes": std::fs::metadata(&archive_path).unwrap().len(),
            "chapter_count": inspect_release_transport(&transports[0]).unwrap().receipt.chapter_count,
            "pack_median_micros": median_micros(pack_samples),
            "inspect_median_micros": median_micros(inspect_samples),
            "unpack_median_micros": median_micros(unpack_samples),
            "sequential_chapter_read_median_micros": median_micros(sequential_samples),
            "parallel_chapter_read_median_micros": median_micros(parallel_samples),
            "generic_gzip_read_median_micros": median_micros(generic_gzip_samples),
            "control_chapter_access_median_micros": median_micros(control_access_samples),
            "payload_bytes": expected_payload_bytes.unwrap(),
        });
        eprintln!("{}", serde_json::to_string_pretty(&report).unwrap());
        eprintln!("benchmark_summary={}", serde_json::to_string(&report).unwrap());
    }

    // r[verify mantle.release_provenance.chapter_transport.unpack]
    #[test]
    fn publication_race_preserves_competing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("destination");
        let stage = create_sibling_stage(&destination, CHAPTER_TRANSPORT_STAGE_PREFIX).unwrap();
        std::fs::write(stage.path().join("payload"), b"stage").unwrap();
        std::fs::create_dir(&destination).unwrap();
        std::fs::write(destination.join("sentinel"), b"competitor").unwrap();

        let error = publish_stage_no_replace(stage, &destination).unwrap_err();

        assert!(error.to_string().contains("without replacement"));
        assert_eq!(std::fs::read(destination.join("sentinel")).unwrap(), b"competitor");
    }

    // r[verify mantle.release_provenance.chapter_transport.unpack]
    #[test]
    fn competing_unpack_destination_is_not_replaced() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = create_verified_bundle(temp.path(), "chapter-no-clobber");
        let transport = temp.path().join("transport");
        let destination = temp.path().join("destination");
        pack_release_transport(&bundle, &transport).unwrap();
        std::fs::create_dir(&destination).unwrap();
        std::fs::write(destination.join("sentinel"), b"competitor").unwrap();

        let error = unpack_release_transport(&transport, &destination).unwrap_err();

        assert!(error.to_string().contains("must be absent"));
        assert_eq!(std::fs::read(destination.join("sentinel")).unwrap(), b"competitor");
    }
}
