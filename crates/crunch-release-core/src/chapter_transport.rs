use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::TreeCopyLimits;
use crate::TreeEntryKind;
use crate::TreeEntryObservation;
use crate::plan_tree_copy;

pub const CHAPTER_TRANSPORT_FORMAT: &str = "chapter-tgz-v1";
pub const CHAPTER_TRANSPORT_INDEX_SCHEMA: &str = "mantle-release-chapter-transport-index-v1";
pub const CHAPTER_TRANSPORT_RECEIPT_SCHEMA: &str = "mantle-release-chapter-transport-receipt-v1";
pub const CHAPTER_TRANSPORT_RESERVED_INDEX_PATH: &str = "__mantle_release_transport_index__.json";
pub const CHAPTER_TRANSPORT_MANIFEST_PATH: &str = "manifest.json";
pub const CHAPTER_TRANSPORT_CHAPTER_TGZ_VERSION: &str = "0.1.0";
pub const CHAPTER_TRANSPORT_CLAIM_SCOPE: &str = "compressed-transport-identity-and-safe-materialization";
pub const CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT: u32 = 256;
pub const CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT: u32 = 4_096;
pub const CHAPTER_TRANSPORT_MAX_PATH_BYTES: u32 = 4_096;
pub const CHAPTER_TRANSPORT_MAX_INDEX_BYTES: u64 = 32 * 1_024 * 1_024;
pub const CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES: u64 = 1_u64 << 40;
pub const CHAPTER_TRANSPORT_MAX_MEMBER_BYTES: u64 = 1_u64 << 40;
pub const CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES: u64 = 1_u64 << 40;
pub const CHAPTER_TRANSPORT_PERMISSION_MODE_MASK: u32 = 0o777;
const BLAKE3_HEX_CHARS: usize = 64;
const CONTROL_GROUP: &str = "control";
const SOURCE_GROUP: &str = "source";
const BINARIES_GROUP: &str = "binaries";
const NON_CLAIM_BUILD_CORRECTNESS: &str = "does not prove build correctness";
const NON_CLAIM_SOURCE_CORRECTNESS: &str = "does not prove source correctness";
const NON_CLAIM_SEMANTICS: &str = "does not prove semantic correctness";
const NON_CLAIM_REPRODUCIBILITY: &str = "does not prove reproducibility";
const NON_CLAIM_DEPLOYMENT: &str = "does not prove deployment safety";
const NON_CLAIM_RELEASE_ELIGIBILITY: &str = "does not prove universal release eligibility";

const _: () = {
    assert!(CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT > 1);
    assert!(CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT > CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT);
    assert!(CHAPTER_TRANSPORT_MAX_PATH_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_MAX_INDEX_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_MAX_MEMBER_BYTES > 0);
    assert!(CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES >= CHAPTER_TRANSPORT_MAX_MEMBER_BYTES);
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChapterTransportEntryKind {
    Directory,
    File,
    Symlink,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChapterTransportRole {
    Control,
    Source,
    Binary,
    ArtifactGroup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTransportEntryInput {
    pub relative_path: String,
    pub kind: ChapterTransportEntryKind,
    pub mode: u32,
    pub size_bytes: u64,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterTransportMember {
    pub relative_path: String,
    pub kind: ChapterTransportEntryKind,
    pub mode: u32,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterTransportChapter {
    pub ordinal: u32,
    pub role: ChapterTransportRole,
    pub group: String,
    pub uncompressed_file_bytes: u64,
    pub members: Vec<ChapterTransportMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterTransportIndex {
    pub schema: String,
    pub format: String,
    pub source_manifest_digest_blake3: String,
    pub chapter_count: u32,
    pub member_count: u32,
    pub total_uncompressed_file_bytes: u64,
    pub chapters: Vec<ChapterTransportChapter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChapterTransportReceipt {
    pub schema: String,
    pub format: String,
    pub chapter_tgz_version: String,
    pub archive_size_bytes: u64,
    pub archive_digest_blake3: String,
    pub source_manifest_digest_blake3: String,
    pub transport_index_digest_blake3: String,
    pub chapter_count: u32,
    pub member_count: u32,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTransportReceiptInput {
    pub archive_size_bytes: u64,
    pub archive_digest_blake3: String,
    pub source_manifest_digest_blake3: String,
    pub transport_index_digest_blake3: String,
    pub chapter_count: u32,
    pub member_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChapterTransportBlockerKind {
    InvalidManifestDigest,
    EntryCountOverflow,
    EntryLimitExceeded,
    ReservedIndexCollision,
    ManifestMissing,
    ManifestInvalid,
    InvalidEntrySize,
    InvalidEntryMode,
    InvalidEntryShape,
    TreePolicy,
    ChapterCountOverflow,
    ChapterLimitExceeded,
    UncompressedBytesOverflow,
    UncompressedBytesLimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTransportBlocker {
    pub kind: ChapterTransportBlockerKind,
    pub relative_path: Option<String>,
    pub message: String,
}

impl fmt::Display for ChapterTransportBlocker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTransportError {
    pub message: String,
}

impl fmt::Display for ChapterTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

// r[impl mantle.release_provenance.chapter_transport.plan]
pub fn plan_chapter_transport(
    mut entries: Vec<ChapterTransportEntryInput>,
    source_manifest_digest_blake3: String,
) -> Result<ChapterTransportIndex, Vec<ChapterTransportBlocker>> {
    let mut blockers = validate_plan_header(&entries, &source_manifest_digest_blake3);
    blockers.extend(validate_input_shapes(&entries));
    let tree_plan = plan_transport_tree(&entries);
    if let Err(tree_blockers) = tree_plan {
        blockers.extend(tree_blockers);
    }
    sort_blockers(&mut blockers);
    if !blockers.is_empty() {
        return Err(blockers);
    }
    entries.sort_by(member_order);
    let chapters = build_chapters(entries)?;
    let totals = chapter_totals(&chapters)?;
    let index = ChapterTransportIndex {
        schema: CHAPTER_TRANSPORT_INDEX_SCHEMA.to_string(),
        format: CHAPTER_TRANSPORT_FORMAT.to_string(),
        source_manifest_digest_blake3,
        chapter_count: totals.chapter_count,
        member_count: totals.member_count,
        total_uncompressed_file_bytes: totals.file_bytes,
        chapters,
    };
    debug_assert_eq!(index.chapter_count, u32::try_from(index.chapters.len()).unwrap_or(u32::MAX));
    debug_assert!(index.chapters.first().is_some_and(|chapter| chapter.ordinal == 0));
    Ok(index)
}

fn validate_plan_header(entries: &[ChapterTransportEntryInput], digest: &str) -> Vec<ChapterTransportBlocker> {
    let mut blockers = Vec::new();
    if !is_blake3_hex(digest) {
        blockers.push(blocker(
            ChapterTransportBlockerKind::InvalidManifestDigest,
            None,
            "chapter transport source manifest digest must be 64 lowercase hexadecimal characters",
        ));
    }
    let entry_count = match u32::try_from(entries.len()) {
        Ok(count) => count,
        Err(_) => {
            blockers.push(blocker(
                ChapterTransportBlockerKind::EntryCountOverflow,
                None,
                "chapter transport member count overflowed u32",
            ));
            return blockers;
        }
    };
    if entry_count > CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT {
        blockers.push(blocker(
            ChapterTransportBlockerKind::EntryLimitExceeded,
            None,
            format!("chapter transport member count {entry_count} exceeds {CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT}"),
        ));
    }
    blockers
}

fn validate_input_shapes(entries: &[ChapterTransportEntryInput]) -> Vec<ChapterTransportBlocker> {
    let mut blockers = Vec::new();
    let mut manifest_count = 0_u32;
    for entry in entries {
        if entry.relative_path == CHAPTER_TRANSPORT_RESERVED_INDEX_PATH {
            blockers.push(blocker(
                ChapterTransportBlockerKind::ReservedIndexCollision,
                Some(entry.relative_path.clone()),
                "release tree contains the reserved chapter transport index path",
            ));
        }
        if entry.relative_path == CHAPTER_TRANSPORT_MANIFEST_PATH {
            manifest_count = manifest_count.saturating_add(1);
            if entry.kind != ChapterTransportEntryKind::File {
                blockers.push(blocker(
                    ChapterTransportBlockerKind::ManifestInvalid,
                    Some(entry.relative_path.clone()),
                    "chapter transport manifest member must be a regular file",
                ));
            }
        }
        validate_entry_shape(entry, &mut blockers);
    }
    if manifest_count == 0 {
        blockers.push(blocker(
            ChapterTransportBlockerKind::ManifestMissing,
            Some(CHAPTER_TRANSPORT_MANIFEST_PATH.to_string()),
            "chapter transport input is missing manifest.json",
        ));
    }
    blockers
}

fn validate_entry_shape(entry: &ChapterTransportEntryInput, blockers: &mut Vec<ChapterTransportBlocker>) {
    if entry.mode & !CHAPTER_TRANSPORT_PERMISSION_MODE_MASK != 0 {
        blockers.push(blocker(
            ChapterTransportBlockerKind::InvalidEntryMode,
            Some(entry.relative_path.clone()),
            format!(
                "chapter transport member {} mode {:#o} contains privileged or unsupported bits",
                entry.relative_path, entry.mode
            ),
        ));
    }
    if entry.size_bytes > CHAPTER_TRANSPORT_MAX_MEMBER_BYTES {
        blockers.push(blocker(
            ChapterTransportBlockerKind::InvalidEntrySize,
            Some(entry.relative_path.clone()),
            format!(
                "chapter transport member {} size {} exceeds {}",
                entry.relative_path, entry.size_bytes, CHAPTER_TRANSPORT_MAX_MEMBER_BYTES
            ),
        ));
    }
    let shape_is_valid = match entry.kind {
        ChapterTransportEntryKind::File => entry.symlink_target.is_none(),
        ChapterTransportEntryKind::Directory => entry.size_bytes == 0 && entry.symlink_target.is_none(),
        ChapterTransportEntryKind::Symlink => entry.size_bytes == 0 && entry.symlink_target.is_some(),
        ChapterTransportEntryKind::Unsupported => false,
    };
    if !shape_is_valid {
        blockers.push(blocker(
            ChapterTransportBlockerKind::InvalidEntryShape,
            Some(entry.relative_path.clone()),
            format!("chapter transport member {} has inconsistent kind, size, or link target", entry.relative_path),
        ));
    }
}

fn plan_transport_tree(entries: &[ChapterTransportEntryInput]) -> Result<(), Vec<ChapterTransportBlocker>> {
    let observations = entries
        .iter()
        .map(|entry| TreeEntryObservation {
            relative_path: entry.relative_path.clone(),
            kind: tree_kind(entry.kind),
            mode: entry.mode,
            symlink_target: entry.symlink_target.clone(),
        })
        .collect::<Vec<_>>();
    plan_tree_copy(observations, TreeCopyLimits::RELEASE_BUNDLE).map(|_| ()).map_err(|blockers| {
        blockers
            .into_iter()
            .map(|tree| blocker(ChapterTransportBlockerKind::TreePolicy, tree.relative_path, tree.message))
            .collect()
    })
}

fn tree_kind(kind: ChapterTransportEntryKind) -> TreeEntryKind {
    match kind {
        ChapterTransportEntryKind::Directory => TreeEntryKind::Directory,
        ChapterTransportEntryKind::File => TreeEntryKind::File,
        ChapterTransportEntryKind::Symlink => TreeEntryKind::Symlink,
        ChapterTransportEntryKind::Unsupported => TreeEntryKind::Unsupported,
    }
}

fn member_order(left: &ChapterTransportEntryInput, right: &ChapterTransportEntryInput) -> core::cmp::Ordering {
    let left_group = group_for_path(&left.relative_path);
    let right_group = group_for_path(&right.relative_path);
    left_group
        .cmp(&right_group)
        .then_with(|| member_phase(left.kind).cmp(&member_phase(right.kind)))
        .then_with(|| left.relative_path.cmp(&right.relative_path))
}

fn member_phase(kind: ChapterTransportEntryKind) -> u8 {
    match kind {
        ChapterTransportEntryKind::Directory => 0,
        ChapterTransportEntryKind::File => 1,
        ChapterTransportEntryKind::Symlink => 2,
        ChapterTransportEntryKind::Unsupported => u8::MAX,
    }
}

fn build_chapters(
    entries: Vec<ChapterTransportEntryInput>,
) -> Result<Vec<ChapterTransportChapter>, Vec<ChapterTransportBlocker>> {
    let mut grouped: BTreeMap<GroupKey, Vec<ChapterTransportMember>> = BTreeMap::new();
    for entry in entries {
        let key = group_for_path(&entry.relative_path);
        grouped.entry(key).or_default().push(member_from_input(entry));
    }
    let mut chapters = Vec::with_capacity(grouped.len());
    for (key, members) in grouped {
        let ordinal = u32::try_from(chapters.len()).map_err(|_| {
            vec![blocker(
                ChapterTransportBlockerKind::ChapterCountOverflow,
                None,
                "chapter transport chapter count overflowed u32",
            )]
        })?;
        let uncompressed_file_bytes = member_file_bytes(&members)?;
        chapters.push(ChapterTransportChapter {
            ordinal,
            role: key.role,
            group: key.group,
            uncompressed_file_bytes,
            members,
        });
    }
    validate_chapter_count(chapters.len())?;
    debug_assert!(chapters.windows(2).all(|pair| pair[0].ordinal < pair[1].ordinal));
    debug_assert!(chapters.first().is_some_and(|chapter| chapter.role == ChapterTransportRole::Control));
    Ok(chapters)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    role: ChapterTransportRole,
    group: String,
}

fn group_for_path(path: &str) -> GroupKey {
    if path == CHAPTER_TRANSPORT_MANIFEST_PATH {
        return GroupKey {
            role: ChapterTransportRole::Control,
            group: CONTROL_GROUP.to_string(),
        };
    }
    let mut components = path.split('/');
    let first = components.next().unwrap_or_default();
    if first == SOURCE_GROUP {
        return GroupKey {
            role: ChapterTransportRole::Source,
            group: SOURCE_GROUP.to_string(),
        };
    }
    if first == BINARIES_GROUP {
        let second = components.next();
        let group = second.map_or_else(|| BINARIES_GROUP.to_string(), |name| format!("{BINARIES_GROUP}/{name}"));
        return GroupKey {
            role: ChapterTransportRole::Binary,
            group,
        };
    }
    GroupKey {
        role: ChapterTransportRole::ArtifactGroup,
        group: first.to_string(),
    }
}

fn member_from_input(entry: ChapterTransportEntryInput) -> ChapterTransportMember {
    ChapterTransportMember {
        relative_path: entry.relative_path,
        kind: entry.kind,
        mode: entry.mode,
        size_bytes: entry.size_bytes,
        symlink_target: entry.symlink_target,
    }
}

fn member_file_bytes(members: &[ChapterTransportMember]) -> Result<u64, Vec<ChapterTransportBlocker>> {
    let mut bytes = 0_u64;
    for member in members {
        if member.kind != ChapterTransportEntryKind::File {
            continue;
        }
        bytes = bytes.checked_add(member.size_bytes).ok_or_else(|| {
            vec![blocker(
                ChapterTransportBlockerKind::UncompressedBytesOverflow,
                Some(member.relative_path.clone()),
                "chapter transport uncompressed byte count overflowed u64",
            )]
        })?;
    }
    Ok(bytes)
}

fn validate_chapter_count(count: usize) -> Result<(), Vec<ChapterTransportBlocker>> {
    let count_u32 = u32::try_from(count).map_err(|_| {
        vec![blocker(
            ChapterTransportBlockerKind::ChapterCountOverflow,
            None,
            "chapter transport chapter count overflowed u32",
        )]
    })?;
    if count_u32 > CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT {
        return Err(vec![blocker(
            ChapterTransportBlockerKind::ChapterLimitExceeded,
            None,
            format!("chapter transport chapter count {count_u32} exceeds {CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT}"),
        )]);
    }
    Ok(())
}

struct ChapterTotals {
    chapter_count: u32,
    member_count: u32,
    file_bytes: u64,
}

fn chapter_totals(chapters: &[ChapterTransportChapter]) -> Result<ChapterTotals, Vec<ChapterTransportBlocker>> {
    let chapter_count = u32::try_from(chapters.len()).map_err(|_| {
        vec![blocker(
            ChapterTransportBlockerKind::ChapterCountOverflow,
            None,
            "chapter transport chapter count overflowed u32",
        )]
    })?;
    let mut member_count = 0_u32;
    let mut file_bytes = 0_u64;
    for chapter in chapters {
        let count = u32::try_from(chapter.members.len()).map_err(|_| {
            vec![blocker(
                ChapterTransportBlockerKind::EntryCountOverflow,
                None,
                "chapter transport chapter member count overflowed u32",
            )]
        })?;
        member_count = member_count.checked_add(count).ok_or_else(|| {
            vec![blocker(
                ChapterTransportBlockerKind::EntryCountOverflow,
                None,
                "chapter transport total member count overflowed u32",
            )]
        })?;
        file_bytes = file_bytes.checked_add(chapter.uncompressed_file_bytes).ok_or_else(|| {
            vec![blocker(
                ChapterTransportBlockerKind::UncompressedBytesOverflow,
                None,
                "chapter transport total uncompressed bytes overflowed u64",
            )]
        })?;
    }
    if file_bytes > CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES {
        return Err(vec![blocker(
            ChapterTransportBlockerKind::UncompressedBytesLimitExceeded,
            None,
            format!(
                "chapter transport uncompressed bytes {file_bytes} exceed {CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES}"
            ),
        )]);
    }
    Ok(ChapterTotals {
        chapter_count,
        member_count,
        file_bytes,
    })
}

pub fn canonical_chapter_transport_index(index: &ChapterTransportIndex) -> Result<Vec<u8>, ChapterTransportError> {
    validate_chapter_transport_index(index)?;
    serde_json::to_vec(index).map_err(|error| transport_error(format!("serializing chapter transport index: {error}")))
}

pub fn parse_canonical_chapter_transport_index(bytes: &[u8]) -> Result<ChapterTransportIndex, ChapterTransportError> {
    validate_index_size(bytes)?;
    let index: ChapterTransportIndex = serde_json::from_slice(bytes)
        .map_err(|error| transport_error(format!("parsing chapter transport index: {error}")))?;
    let canonical = canonical_chapter_transport_index(&index)?;
    if canonical != bytes {
        return Err(transport_error("chapter transport index is not canonical compact JSON"));
    }
    Ok(index)
}

pub fn validate_chapter_transport_index(index: &ChapterTransportIndex) -> Result<(), ChapterTransportError> {
    if index.schema != CHAPTER_TRANSPORT_INDEX_SCHEMA {
        return Err(transport_error(format!("unsupported chapter transport index schema: {}", index.schema)));
    }
    if index.format != CHAPTER_TRANSPORT_FORMAT {
        return Err(transport_error(format!("unsupported chapter transport format: {}", index.format)));
    }
    if !is_blake3_hex(&index.source_manifest_digest_blake3) {
        return Err(transport_error("chapter transport index manifest digest is not lowercase BLAKE3 hex"));
    }
    let inputs = flatten_index_members(index)?;
    let expected = plan_chapter_transport(inputs, index.source_manifest_digest_blake3.clone())
        .map_err(|blockers| transport_error(render_blockers("chapter transport index", &blockers)))?;
    if &expected != index {
        return Err(transport_error("chapter transport index does not match the canonical deterministic plan"));
    }
    debug_assert_eq!(index.chapter_count, expected.chapter_count);
    debug_assert_eq!(index.member_count, expected.member_count);
    Ok(())
}

fn flatten_index_members(
    index: &ChapterTransportIndex,
) -> Result<Vec<ChapterTransportEntryInput>, ChapterTransportError> {
    let member_capacity = usize::try_from(index.member_count)
        .map_err(|_| transport_error("chapter transport index member count overflowed usize"))?;
    let mut inputs = Vec::with_capacity(member_capacity);
    for chapter in &index.chapters {
        for member in &chapter.members {
            inputs.push(ChapterTransportEntryInput {
                relative_path: member.relative_path.clone(),
                kind: member.kind,
                mode: member.mode,
                size_bytes: member.size_bytes,
                symlink_target: member.symlink_target.clone(),
            });
        }
    }
    if inputs.len() != member_capacity {
        return Err(transport_error(format!(
            "chapter transport index member count {} does not match {}",
            index.member_count,
            inputs.len()
        )));
    }
    Ok(inputs)
}

fn validate_index_size(bytes: &[u8]) -> Result<(), ChapterTransportError> {
    let byte_count =
        u64::try_from(bytes.len()).map_err(|_| transport_error("chapter transport index byte count overflowed u64"))?;
    if byte_count == 0 {
        return Err(transport_error("chapter transport index is empty"));
    }
    if byte_count > CHAPTER_TRANSPORT_MAX_INDEX_BYTES {
        return Err(transport_error(format!(
            "chapter transport index size {byte_count} exceeds {CHAPTER_TRANSPORT_MAX_INDEX_BYTES}"
        )));
    }
    Ok(())
}

pub fn build_chapter_transport_receipt(
    input: ChapterTransportReceiptInput,
) -> Result<ChapterTransportReceipt, ChapterTransportError> {
    let receipt = ChapterTransportReceipt {
        schema: CHAPTER_TRANSPORT_RECEIPT_SCHEMA.to_string(),
        format: CHAPTER_TRANSPORT_FORMAT.to_string(),
        chapter_tgz_version: CHAPTER_TRANSPORT_CHAPTER_TGZ_VERSION.to_string(),
        archive_size_bytes: input.archive_size_bytes,
        archive_digest_blake3: input.archive_digest_blake3,
        source_manifest_digest_blake3: input.source_manifest_digest_blake3,
        transport_index_digest_blake3: input.transport_index_digest_blake3,
        chapter_count: input.chapter_count,
        member_count: input.member_count,
        claim_scope: CHAPTER_TRANSPORT_CLAIM_SCOPE.to_string(),
        non_claims: chapter_transport_non_claims(),
    };
    validate_chapter_transport_receipt(&receipt)?;
    debug_assert_eq!(receipt.schema, CHAPTER_TRANSPORT_RECEIPT_SCHEMA);
    debug_assert_eq!(receipt.non_claims, chapter_transport_non_claims());
    Ok(receipt)
}

pub fn canonical_chapter_transport_receipt(
    receipt: &ChapterTransportReceipt,
) -> Result<Vec<u8>, ChapterTransportError> {
    validate_chapter_transport_receipt(receipt)?;
    serde_json::to_vec(receipt)
        .map_err(|error| transport_error(format!("serializing chapter transport receipt: {error}")))
}

pub fn parse_canonical_chapter_transport_receipt(
    bytes: &[u8],
) -> Result<ChapterTransportReceipt, ChapterTransportError> {
    let receipt: ChapterTransportReceipt = serde_json::from_slice(bytes)
        .map_err(|error| transport_error(format!("parsing chapter transport receipt: {error}")))?;
    let canonical = canonical_chapter_transport_receipt(&receipt)?;
    if canonical != bytes {
        return Err(transport_error("chapter transport receipt is not canonical compact JSON"));
    }
    Ok(receipt)
}

pub fn validate_chapter_transport_receipt(receipt: &ChapterTransportReceipt) -> Result<(), ChapterTransportError> {
    if receipt.schema != CHAPTER_TRANSPORT_RECEIPT_SCHEMA || receipt.format != CHAPTER_TRANSPORT_FORMAT {
        return Err(transport_error("unsupported chapter transport receipt schema or format"));
    }
    if receipt.chapter_tgz_version != CHAPTER_TRANSPORT_CHAPTER_TGZ_VERSION {
        return Err(transport_error(format!("unsupported chapter-tgz version: {}", receipt.chapter_tgz_version)));
    }
    if receipt.archive_size_bytes == 0 {
        return Err(transport_error("chapter transport archive size must be positive"));
    }
    if receipt.archive_size_bytes > CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES {
        return Err(transport_error(format!(
            "chapter transport archive size {} exceeds {}",
            receipt.archive_size_bytes, CHAPTER_TRANSPORT_MAX_ARCHIVE_BYTES
        )));
    }
    validate_receipt_digests(receipt)?;
    validate_receipt_counts(receipt)?;
    if receipt.claim_scope != CHAPTER_TRANSPORT_CLAIM_SCOPE {
        return Err(transport_error("chapter transport receipt claim scope is unsupported"));
    }
    if receipt.non_claims != chapter_transport_non_claims() {
        return Err(transport_error("chapter transport receipt non-claims are incomplete or out of order"));
    }
    Ok(())
}

fn validate_receipt_digests(receipt: &ChapterTransportReceipt) -> Result<(), ChapterTransportError> {
    let digests = [
        ("archive", receipt.archive_digest_blake3.as_str()),
        ("source manifest", receipt.source_manifest_digest_blake3.as_str()),
        ("transport index", receipt.transport_index_digest_blake3.as_str()),
    ];
    for (label, digest) in digests {
        if !is_blake3_hex(digest) {
            return Err(transport_error(format!(
                "chapter transport receipt {label} digest is not lowercase BLAKE3 hex"
            )));
        }
    }
    Ok(())
}

fn validate_receipt_counts(receipt: &ChapterTransportReceipt) -> Result<(), ChapterTransportError> {
    if receipt.chapter_count == 0 || receipt.chapter_count > CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT {
        return Err(transport_error(format!(
            "chapter transport receipt chapter count {} is outside the supported bound",
            receipt.chapter_count
        )));
    }
    if receipt.member_count == 0 || receipt.member_count > CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT {
        return Err(transport_error(format!(
            "chapter transport receipt member count {} is outside the supported bound",
            receipt.member_count
        )));
    }
    Ok(())
}

pub fn chapter_transport_non_claims() -> Vec<String> {
    vec![
        NON_CLAIM_BUILD_CORRECTNESS.to_string(),
        NON_CLAIM_SOURCE_CORRECTNESS.to_string(),
        NON_CLAIM_SEMANTICS.to_string(),
        NON_CLAIM_REPRODUCIBILITY.to_string(),
        NON_CLAIM_DEPLOYMENT.to_string(),
        NON_CLAIM_RELEASE_ELIGIBILITY.to_string(),
    ]
}

pub fn render_chapter_transport_blockers(blockers: &[ChapterTransportBlocker]) -> String {
    render_blockers("chapter transport plan", blockers)
}

fn render_blockers(label: &str, blockers: &[ChapterTransportBlocker]) -> String {
    let details = blockers.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
    format!("{label} rejected with {} blocker(s): {details}", blockers.len())
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn blocker(
    kind: ChapterTransportBlockerKind,
    relative_path: Option<String>,
    message: impl Into<String>,
) -> ChapterTransportBlocker {
    ChapterTransportBlocker {
        kind,
        relative_path,
        message: message.into(),
    }
}

fn sort_blockers(blockers: &mut [ChapterTransportBlocker]) {
    blockers.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.message.cmp(&right.message))
    });
}

fn transport_error(message: impl Into<String>) -> ChapterTransportError {
    ChapterTransportError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_MODE: u32 = 0o644;
    const DIRECTORY_MODE: u32 = 0o755;
    const SYMLINK_MODE: u32 = 0o777;
    const PRIVILEGED_FILE_MODE: u32 = 0o4_755;
    const FILE_BYTES: u64 = 8;
    const EXPECTED_CHAPTER_COUNT: u32 = 5;

    fn digest(character: char) -> String {
        character.to_string().repeat(BLAKE3_HEX_CHARS)
    }

    fn file(path: &str) -> ChapterTransportEntryInput {
        ChapterTransportEntryInput {
            relative_path: path.to_string(),
            kind: ChapterTransportEntryKind::File,
            mode: FILE_MODE,
            size_bytes: FILE_BYTES,
            symlink_target: None,
        }
    }

    fn directory(path: &str) -> ChapterTransportEntryInput {
        ChapterTransportEntryInput {
            relative_path: path.to_string(),
            kind: ChapterTransportEntryKind::Directory,
            mode: DIRECTORY_MODE,
            size_bytes: 0,
            symlink_target: None,
        }
    }

    fn symlink(path: &str, target: &str) -> ChapterTransportEntryInput {
        ChapterTransportEntryInput {
            relative_path: path.to_string(),
            kind: ChapterTransportEntryKind::Symlink,
            mode: SYMLINK_MODE,
            size_bytes: 0,
            symlink_target: Some(target.to_string()),
        }
    }

    fn valid_entries() -> Vec<ChapterTransportEntryInput> {
        vec![
            file(CHAPTER_TRANSPORT_MANIFEST_PATH),
            directory("source"),
            file("source/mantle-src.tar"),
            directory("binaries"),
            file("binaries/mantle"),
            directory("proof"),
            file("proof/report.json"),
            symlink("proof/latest", "report.json"),
        ]
    }

    // r[verify mantle.release_provenance.chapter_transport.plan]
    #[test]
    fn plan_is_input_order_independent_and_groups_roles() {
        let first = plan_chapter_transport(valid_entries(), digest('a')).unwrap();
        let mut reversed = valid_entries();
        reversed.reverse();
        let second = plan_chapter_transport(reversed, digest('a')).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.chapter_count, EXPECTED_CHAPTER_COUNT);
        assert_eq!(first.chapters[0].role, ChapterTransportRole::Control);
        assert_eq!(first.chapters[1].role, ChapterTransportRole::Source);
        assert_eq!(first.chapters[2].group, BINARIES_GROUP);
        assert_eq!(first.chapters[3].group, "binaries/mantle");
        assert_eq!(first.chapters[4].group, "proof");
    }

    #[test]
    fn canonical_index_and_receipt_round_trip() {
        let index = plan_chapter_transport(valid_entries(), digest('b')).unwrap();
        let index_bytes = canonical_chapter_transport_index(&index).unwrap();
        let parsed_index = parse_canonical_chapter_transport_index(&index_bytes).unwrap();
        let receipt = build_chapter_transport_receipt(ChapterTransportReceiptInput {
            archive_size_bytes: FILE_BYTES,
            archive_digest_blake3: digest('c'),
            source_manifest_digest_blake3: digest('b'),
            transport_index_digest_blake3: digest('d'),
            chapter_count: index.chapter_count,
            member_count: index.member_count,
        })
        .unwrap();
        let receipt_bytes = canonical_chapter_transport_receipt(&receipt).unwrap();
        let parsed_receipt = parse_canonical_chapter_transport_receipt(&receipt_bytes).unwrap();

        assert_eq!(parsed_index, index);
        assert_eq!(parsed_receipt, receipt);
        assert!(!index_bytes.contains(&b'\n'));
        assert!(!receipt_bytes.contains(&b'\n'));
    }

    #[test]
    fn invalid_plan_matrix_fails_closed() {
        let cases = [
            (Vec::new(), ChapterTransportBlockerKind::ManifestMissing),
            (
                vec![
                    file(CHAPTER_TRANSPORT_RESERVED_INDEX_PATH),
                    file(CHAPTER_TRANSPORT_MANIFEST_PATH),
                ],
                ChapterTransportBlockerKind::ReservedIndexCollision,
            ),
            (
                vec![
                    file(CHAPTER_TRANSPORT_MANIFEST_PATH),
                    file(CHAPTER_TRANSPORT_MANIFEST_PATH),
                ],
                ChapterTransportBlockerKind::TreePolicy,
            ),
            (
                vec![file(CHAPTER_TRANSPORT_MANIFEST_PATH), symlink("escape", "../outside")],
                ChapterTransportBlockerKind::TreePolicy,
            ),
        ];
        for (entries, expected) in cases {
            let blockers = plan_chapter_transport(entries, digest('e')).unwrap_err();
            assert!(blockers.iter().any(|blocker| blocker.kind == expected), "missing {expected:?}: {blockers:?}");
        }
    }

    #[test]
    fn invalid_entry_size_mode_and_shape_fail_closed() {
        let oversized = ChapterTransportEntryInput {
            size_bytes: CHAPTER_TRANSPORT_MAX_MEMBER_BYTES.saturating_add(1),
            ..file(CHAPTER_TRANSPORT_MANIFEST_PATH)
        };
        let privileged_mode = ChapterTransportEntryInput {
            mode: PRIVILEGED_FILE_MODE,
            ..file(CHAPTER_TRANSPORT_MANIFEST_PATH)
        };
        let malformed_directory = ChapterTransportEntryInput {
            size_bytes: FILE_BYTES,
            ..directory("proof")
        };

        let size_blockers = plan_chapter_transport(vec![oversized], digest('f')).unwrap_err();
        let mode_blockers = plan_chapter_transport(vec![privileged_mode], digest('f')).unwrap_err();
        let shape_blockers =
            plan_chapter_transport(vec![file(CHAPTER_TRANSPORT_MANIFEST_PATH), malformed_directory], digest('f'))
                .unwrap_err();

        assert!(size_blockers.iter().any(|blocker| blocker.kind == ChapterTransportBlockerKind::InvalidEntrySize));
        assert!(mode_blockers.iter().any(|blocker| blocker.kind == ChapterTransportBlockerKind::InvalidEntryMode));
        assert!(shape_blockers.iter().any(|blocker| blocker.kind == ChapterTransportBlockerKind::InvalidEntryShape));
    }

    #[test]
    fn count_and_total_byte_limits_fail_closed() {
        let mut excessive_members = vec![file(CHAPTER_TRANSPORT_MANIFEST_PATH)];
        for ordinal in 0..CHAPTER_TRANSPORT_MAX_MEMBERS_COUNT {
            excessive_members.push(file(&format!("member-{ordinal}")));
        }
        let member_blockers = plan_chapter_transport(excessive_members, digest('f')).unwrap_err();
        assert!(
            member_blockers
                .iter()
                .any(|blocker| blocker.kind == ChapterTransportBlockerKind::EntryLimitExceeded)
        );

        let mut excessive_chapters = vec![file(CHAPTER_TRANSPORT_MANIFEST_PATH), directory(BINARIES_GROUP)];
        let binary_count = CHAPTER_TRANSPORT_MAX_CHAPTERS_COUNT.saturating_sub(1);
        for ordinal in 0..binary_count {
            excessive_chapters.push(file(&format!("{BINARIES_GROUP}/binary-{ordinal}")));
        }
        let chapter_blockers = plan_chapter_transport(excessive_chapters, digest('f')).unwrap_err();
        assert!(
            chapter_blockers
                .iter()
                .any(|blocker| blocker.kind == ChapterTransportBlockerKind::ChapterLimitExceeded)
        );

        let oversized_total = vec![file(CHAPTER_TRANSPORT_MANIFEST_PATH), ChapterTransportEntryInput {
            relative_path: "payload".to_string(),
            kind: ChapterTransportEntryKind::File,
            mode: FILE_MODE,
            size_bytes: CHAPTER_TRANSPORT_MAX_UNCOMPRESSED_BYTES,
            symlink_target: None,
        }];
        let byte_blockers = plan_chapter_transport(oversized_total, digest('f')).unwrap_err();
        assert!(
            byte_blockers
                .iter()
                .any(|blocker| blocker.kind == ChapterTransportBlockerKind::UncompressedBytesLimitExceeded)
        );
    }

    #[test]
    fn receipt_rejects_stale_digest_counts_and_non_claims() {
        let mut receipt = build_chapter_transport_receipt(ChapterTransportReceiptInput {
            archive_size_bytes: FILE_BYTES,
            archive_digest_blake3: digest('a'),
            source_manifest_digest_blake3: digest('b'),
            transport_index_digest_blake3: digest('c'),
            chapter_count: 1,
            member_count: 1,
        })
        .unwrap();
        receipt.archive_digest_blake3 = digest('A');
        assert!(validate_chapter_transport_receipt(&receipt).is_err());
        receipt.archive_digest_blake3 = digest('a');
        receipt.chapter_count = 0;
        assert!(validate_chapter_transport_receipt(&receipt).is_err());
        receipt.chapter_count = 1;
        receipt.non_claims.pop();
        assert!(validate_chapter_transport_receipt(&receipt).is_err());
    }

    #[test]
    fn parse_rejects_noncanonical_and_unknown_fields() {
        let index = plan_chapter_transport(valid_entries(), digest('a')).unwrap();
        let mut pretty = serde_json::to_vec_pretty(&index).unwrap();
        pretty.push(b'\n');
        let unknown = br#"{"schema":"mantle-release-chapter-transport-receipt-v1","unknown":true}"#;

        assert!(parse_canonical_chapter_transport_index(&pretty).is_err());
        assert!(parse_canonical_chapter_transport_receipt(unknown).is_err());
    }
}
