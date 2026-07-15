use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

use flate2::Compression;
use flate2::GzBuilder;
use flate2::read::GzDecoder;
use tar::Archive;
use tar::Builder;
use tar::EntryType;
use tar::Header;
use tar::HeaderMode;

use super::ArchivePolicy;
use super::CompressionProfile;
use super::LayerMode;
use super::MaterializedObject;
use super::ObjectEntry;
use super::ObjectEntryKind;
use super::ProjectionEntry;
use super::ProjectionIssue;
use super::ProjectionLayer;
use super::issue;
use super::safe_relative_path;
use super::sha256_digest;

const ROOT_ENTRY: &str = ".";
const TAR_BLOCK_BYTES: usize = 512;
const TAR_END_BLOCKS: usize = 2;
const EMPTY_ARCHIVE_MIN_BYTES: usize = TAR_BLOCK_BYTES.saturating_mul(TAR_END_BLOCKS);
const GZIP_LEVEL: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveLimits {
    pub max_depth: usize,
    pub max_entries: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ArchiveInspection<'a> {
    pub media_type: &'a str,
    pub blob: &'a [u8],
    pub policy: &'a ArchivePolicy,
    pub limits: ArchiveLimits,
    pub max_uncompressed_bytes: u64,
    pub require_canonical: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LayerPayload {
    pub blob: Vec<u8>,
    pub uncompressed: Vec<u8>,
    pub canonical_archive: bool,
}

#[derive(Clone, Copy, Debug)]
struct MountPath<'a> {
    mount: &'a str,
    internal: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct LinkPath<'a> {
    path: &'a str,
    target: &'a str,
}

fn archive_issue(code: &str, path: impl Into<String>, message: &str) -> ProjectionIssue {
    issue(code, path, message)
}

fn source_size(entries: &[ObjectEntry]) -> u64 {
    entries
        .iter()
        .filter(|entry| entry.kind == ObjectEntryKind::File)
        .fold(0_u64, |total, entry| total.saturating_add(entry.data.len() as u64))
}

fn join_mount(parts: MountPath<'_>) -> String {
    if parts.internal == ROOT_ENTRY {
        parts.mount.to_string()
    } else {
        format!("{}/{}", parts.mount, parts.internal)
    }
}

fn mounted_entries(
    entry: &ProjectionEntry,
    object: &MaterializedObject,
    max_depth: usize,
) -> Result<Vec<ObjectEntry>, ProjectionIssue> {
    if object.entries.is_empty() {
        return Err(archive_issue("empty-object", &entry.object_ref, "materialized object has no entries"));
    }
    if source_size(&object.entries) != entry.size_bytes {
        return Err(archive_issue(
            "object-size-mismatch",
            &entry.object_ref,
            "projection size does not match materialized regular-file bytes",
        ));
    }
    let mut mounted = Vec::with_capacity(object.entries.len());
    for source in &object.entries {
        if source.relative_path != ROOT_ENTRY && !safe_relative_path(&source.relative_path, max_depth) {
            return Err(archive_issue(
                "unsafe-object-path",
                &source.relative_path,
                "materialized object path is unsafe",
            ));
        }
        let relative_path = join_mount(MountPath {
            mount: &entry.relative_path,
            internal: &source.relative_path,
        });
        if !safe_relative_path(&relative_path, max_depth) {
            return Err(archive_issue(
                "mounted-path-depth",
                relative_path,
                "mounted object path exceeds the projection depth bound",
            ));
        }
        mounted.push(ObjectEntry {
            relative_path,
            kind: source.kind,
            data: source.data.clone(),
            link_target: source.link_target.clone(),
        });
    }
    debug_assert_eq!(mounted.len(), object.entries.len());
    debug_assert!(mounted.iter().all(|value| safe_relative_path(&value.relative_path, max_depth)));
    Ok(mounted)
}

fn link_target_has_safe_shape(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }
    if target.starts_with('/') {
        return false;
    }
    if target.contains('\0') {
        return false;
    }
    if target.contains('\\') {
        return false;
    }
    true
}

fn validate_link(link: LinkPath<'_>, max_depth: usize) -> bool {
    if !link_target_has_safe_shape(link.target) {
        return false;
    }
    debug_assert!(safe_relative_path(link.path, max_depth));
    debug_assert!(!link.target.is_empty());
    let mut stack = link
        .path
        .rsplit_once('/')
        .map_or_else(Vec::new, |(parent, _)| parent.split('/').collect::<Vec<_>>());
    for component in link.target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if stack.pop().is_none() {
                    return false;
                }
            }
            value => stack.push(value),
        }
        if stack.len() > max_depth {
            return false;
        }
    }
    true
}

fn validate_entry_shape(entry: &ObjectEntry, max_depth: usize) -> Result<(), ProjectionIssue> {
    match entry.kind {
        ObjectEntryKind::File | ObjectEntryKind::Directory => {
            if entry.link_target.is_some() {
                return Err(archive_issue(
                    "unexpected-link-target",
                    &entry.relative_path,
                    "non-link archive entry carries a link target",
                ));
            }
        }
        ObjectEntryKind::Symlink => {
            let target = entry.link_target.as_deref().ok_or_else(|| {
                archive_issue("missing-link-target", &entry.relative_path, "symlink target is missing")
            })?;
            let link = LinkPath {
                path: &entry.relative_path,
                target,
            };
            if !validate_link(link, max_depth) {
                return Err(archive_issue(
                    "escaping-link",
                    &entry.relative_path,
                    "symlink target is absolute, escaping, or too deep",
                ));
            }
        }
    }
    debug_assert!(safe_relative_path(&entry.relative_path, max_depth));
    debug_assert_eq!(entry.link_target.is_some(), entry.kind == ObjectEntryKind::Symlink);
    Ok(())
}

fn validate_parent_kinds(entries: &[ObjectEntry]) -> Result<(), ProjectionIssue> {
    let kinds = entries.iter().map(|entry| (entry.relative_path.as_str(), entry.kind)).collect::<BTreeMap<_, _>>();
    for entry in entries {
        let components = entry.relative_path.split('/').collect::<Vec<_>>();
        for parent_depth in 1..components.len() {
            let parent = components[..parent_depth].join("/");
            if kinds.get(parent.as_str()).is_some_and(|kind| *kind != ObjectEntryKind::Directory) {
                return Err(archive_issue(
                    "non-directory-parent",
                    &entry.relative_path,
                    "archive path descends through a file or symlink",
                ));
            }
        }
    }
    Ok(())
}

fn canonical_entries(
    layer: &ProjectionLayer,
    objects: &BTreeMap<String, MaterializedObject>,
    limits: ArchiveLimits,
) -> Result<Vec<ObjectEntry>, ProjectionIssue> {
    let mut result = Vec::with_capacity(limits.max_entries);
    for entry in &layer.entries {
        let object = objects
            .get(&entry.object_ref)
            .ok_or_else(|| archive_issue("missing-object", &entry.object_ref, "projection object is unavailable"))?;
        let mounted = mounted_entries(entry, object, limits.max_depth)?;
        let expanded_count = result.len().saturating_add(mounted.len());
        if expanded_count > limits.max_entries {
            return Err(archive_issue(
                "archive-entry-bound",
                &layer.role,
                "expanded archive entries exceed the admitted bound",
            ));
        }
        result.extend(mounted);
    }
    result.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut paths = BTreeSet::new();
    for entry in &result {
        if !paths.insert(entry.relative_path.as_str()) {
            return Err(archive_issue(
                "archive-path-collision",
                &entry.relative_path,
                "multiple objects project to the same archive path",
            ));
        }
        validate_entry_shape(entry, limits.max_depth)?;
    }
    validate_parent_kinds(&result)?;
    debug_assert!(result.len() <= limits.max_entries);
    debug_assert_eq!(paths.len(), result.len());
    Ok(result)
}

fn append_entry(
    builder: &mut Builder<Vec<u8>>,
    entry: &ObjectEntry,
    policy: &ArchivePolicy,
) -> Result<(), ProjectionIssue> {
    let mut header = Header::new_gnu();
    let (entry_type, size_bytes, mode) = match entry.kind {
        ObjectEntryKind::File => (EntryType::Regular, entry.data.len() as u64, policy.file_mode),
        ObjectEntryKind::Directory => (EntryType::Directory, 0_u64, policy.directory_mode),
        ObjectEntryKind::Symlink => (EntryType::Symlink, 0_u64, policy.symlink_mode),
    };
    header.set_entry_type(entry_type);
    header.set_size(size_bytes);
    header.set_mode(mode);
    header.set_uid(policy.uid);
    header.set_gid(policy.gid);
    header.set_mtime(policy.mtime);
    if let Some(target) = &entry.link_target {
        header
            .set_link_name(target)
            .map_err(|error| archive_issue("link-header", &entry.relative_path, &error.to_string()))?;
    }
    header.set_cksum();
    debug_assert_eq!(header.entry_type(), entry_type);
    debug_assert_eq!(header.size().ok(), Some(size_bytes));
    builder
        .append_data(&mut header, &entry.relative_path, Cursor::new(&entry.data))
        .map_err(|error| archive_issue("tar-append", &entry.relative_path, &error.to_string()))
}

fn canonical_tar(entries: &[ObjectEntry], policy: &ArchivePolicy) -> Result<Vec<u8>, ProjectionIssue> {
    let mut builder = Builder::new(Vec::new());
    builder.mode(HeaderMode::Deterministic);
    for entry in entries {
        append_entry(&mut builder, entry, policy)?;
    }
    builder.finish().map_err(|error| archive_issue("tar-finish", "layer", &error.to_string()))?;
    builder.into_inner().map_err(|error| archive_issue("tar-inner", "layer", &error.to_string()))
}

fn compress(bytes: &[u8], profile: CompressionProfile) -> Result<Vec<u8>, ProjectionIssue> {
    match profile {
        CompressionProfile::None => Ok(bytes.to_vec()),
        CompressionProfile::GzipDeterministicV1 => {
            let mut encoder = GzBuilder::new().mtime(0).write(Vec::new(), Compression::new(GZIP_LEVEL));
            encoder.write_all(bytes).map_err(|error| archive_issue("gzip-write", "layer", &error.to_string()))?;
            encoder.finish().map_err(|error| archive_issue("gzip-finish", "layer", &error.to_string()))
        }
    }
}

fn exact_blob_entry(object: &MaterializedObject) -> Option<&ObjectEntry> {
    if object.entries.len() != 1 {
        return None;
    }
    let entry = &object.entries[0];
    if entry.kind != ObjectEntryKind::File {
        return None;
    }
    if entry.relative_path != ROOT_ENTRY {
        return None;
    }
    if entry.link_target.is_some() {
        return None;
    }
    Some(entry)
}

pub(crate) fn layer_payload(
    layer: &ProjectionLayer,
    objects: &BTreeMap<String, MaterializedObject>,
    policy: &ArchivePolicy,
    limits: ArchiveLimits,
) -> Result<LayerPayload, ProjectionIssue> {
    match layer.mode {
        LayerMode::ExactBlob => {
            let projection_entry = layer
                .entries
                .first()
                .ok_or_else(|| archive_issue("missing-exact-entry", &layer.role, "exact layer entry is absent"))?;
            let object = objects.get(&projection_entry.object_ref).ok_or_else(|| {
                archive_issue("missing-object", &projection_entry.object_ref, "exact object is unavailable")
            })?;
            let source_entry = exact_blob_entry(object).ok_or_else(|| {
                archive_issue(
                    "exact-object-shape",
                    &projection_entry.object_ref,
                    "exact blob requires one root regular file",
                )
            })?;
            let bytes = source_entry.data.clone();
            if bytes.len() as u64 != projection_entry.size_bytes {
                return Err(archive_issue(
                    "exact-object-size",
                    &projection_entry.object_ref,
                    "exact object size disagrees with the projection",
                ));
            }
            Ok(LayerPayload {
                blob: bytes.clone(),
                uncompressed: bytes,
                canonical_archive: false,
            })
        }
        LayerMode::CanonicalArchive => {
            let entries = canonical_entries(layer, objects, limits)?;
            let uncompressed = canonical_tar(&entries, policy)?;
            let blob = compress(&uncompressed, policy.compression)?;
            Ok(LayerPayload {
                blob,
                uncompressed,
                canonical_archive: true,
            })
        }
    }
}

fn decompress(
    bytes: &[u8],
    profile: CompressionProfile,
    max_uncompressed_bytes: u64,
) -> Result<Vec<u8>, ProjectionIssue> {
    let mut decoded = Vec::new();
    match profile {
        CompressionProfile::None => decoded.extend_from_slice(bytes),
        CompressionProfile::GzipDeterministicV1 => {
            GzDecoder::new(bytes)
                .take(max_uncompressed_bytes.saturating_add(1))
                .read_to_end(&mut decoded)
                .map_err(|error| archive_issue("gzip-read", "layer", &error.to_string()))?;
        }
    }
    if decoded.len() as u64 > max_uncompressed_bytes {
        return Err(archive_issue("archive-byte-bound", "layer", "uncompressed archive exceeds the import bound"));
    }
    Ok(decoded)
}

fn archive_profile(media_type: &str) -> Option<CompressionProfile> {
    if media_type.ends_with("+gzip") {
        Some(CompressionProfile::GzipDeterministicV1)
    } else if media_type.ends_with(".tar") || media_type.contains(".tar+") {
        Some(CompressionProfile::None)
    } else {
        None
    }
}

fn archive_entry_kind(entry_type: EntryType, path: &str) -> Result<ObjectEntryKind, ProjectionIssue> {
    if entry_type.is_file() {
        return Ok(ObjectEntryKind::File);
    }
    if entry_type.is_dir() {
        return Ok(ObjectEntryKind::Directory);
    }
    if entry_type.is_symlink() {
        return Ok(ObjectEntryKind::Symlink);
    }
    Err(archive_issue("unsupported-archive-entry", path, "hard links and special files are forbidden"))
}

fn parse_link_target<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    path: &str,
    limits: ArchiveLimits,
) -> Result<String, ProjectionIssue> {
    let target = entry
        .link_name()
        .map_err(|error| archive_issue("tar-link", path, &error.to_string()))?
        .ok_or_else(|| archive_issue("missing-link-target", path, "archive symlink target is missing"))?;
    let target = target
        .to_str()
        .ok_or_else(|| archive_issue("non-utf8-link", path, "archive symlink target is not UTF-8"))?
        .to_string();
    let link = LinkPath { path, target: &target };
    if !validate_link(link, limits.max_depth) {
        return Err(archive_issue("escaping-link", path, "archive symlink escapes its root"));
    }
    Ok(target)
}

fn parse_tar_entry<R: Read>(
    mut entry: tar::Entry<'_, R>,
    limits: ArchiveLimits,
) -> Result<ObjectEntry, ProjectionIssue> {
    let path = entry
        .path()
        .map_err(|error| archive_issue("tar-path", "layer", &error.to_string()))?
        .to_str()
        .ok_or_else(|| archive_issue("non-utf8-path", "layer", "archive path is not UTF-8"))?
        .trim_end_matches('/')
        .to_string();
    if !safe_relative_path(&path, limits.max_depth) {
        return Err(archive_issue("unsafe-archive-path", path, "archive path is absolute, escaping, or too deep"));
    }
    let kind = archive_entry_kind(entry.header().entry_type(), &path)?;
    let link_target = if kind == ObjectEntryKind::Symlink {
        Some(parse_link_target(&mut entry, &path, limits)?)
    } else {
        None
    };
    let mut data = Vec::new();
    if kind == ObjectEntryKind::File {
        entry
            .read_to_end(&mut data)
            .map_err(|error| archive_issue("tar-content", &path, &error.to_string()))?;
    } else {
        debug_assert!(data.is_empty());
    }
    debug_assert!(safe_relative_path(&path, limits.max_depth));
    debug_assert_eq!(link_target.is_some(), kind == ObjectEntryKind::Symlink);
    Ok(ObjectEntry {
        relative_path: path,
        kind,
        data,
        link_target,
    })
}

fn parse_tar(bytes: &[u8], limits: ArchiveLimits) -> Result<Vec<ObjectEntry>, ProjectionIssue> {
    if bytes.len() < EMPTY_ARCHIVE_MIN_BYTES {
        return Err(archive_issue("truncated-archive", "layer", "tar archive is truncated"));
    }
    let mut entries = Vec::with_capacity(limits.max_entries);
    let mut archive = Archive::new(Cursor::new(bytes));
    let archive_entries =
        archive.entries().map_err(|error| archive_issue("tar-entries", "layer", &error.to_string()))?;
    for entry_result in archive_entries {
        if entries.len() >= limits.max_entries {
            return Err(archive_issue("archive-entry-bound", "layer", "archive entry count exceeds the bound"));
        }
        let entry = entry_result.map_err(|error| archive_issue("tar-entry", "layer", &error.to_string()))?;
        entries.push(parse_tar_entry(entry, limits)?);
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    for pair in entries.windows(2) {
        if pair[0].relative_path == pair[1].relative_path {
            return Err(archive_issue(
                "duplicate-archive-path",
                &pair[0].relative_path,
                "archive contains duplicate paths",
            ));
        }
    }
    debug_assert!(entries.len() <= limits.max_entries);
    debug_assert!(entries.windows(2).all(|pair| pair[0].relative_path != pair[1].relative_path));
    Ok(entries)
}

pub(crate) fn uncompressed_sha256(
    media_type: &str,
    blob: &[u8],
    max_uncompressed_bytes: u64,
) -> Result<String, ProjectionIssue> {
    let bytes = match archive_profile(media_type) {
        Some(profile) => decompress(blob, profile, max_uncompressed_bytes)?,
        None => blob.to_vec(),
    };
    Ok(sha256_digest(&bytes))
}

pub(crate) fn inspect_archive_with_options(request: ArchiveInspection<'_>) -> Result<String, ProjectionIssue> {
    let Some(profile) = archive_profile(request.media_type) else {
        return Ok("exact-blob".to_string());
    };
    let uncompressed = decompress(request.blob, profile, request.max_uncompressed_bytes)?;
    let entries = parse_tar(&uncompressed, request.limits)?;
    debug_assert!(uncompressed.len() as u64 <= request.max_uncompressed_bytes);
    debug_assert!(entries.len() <= request.limits.max_entries);
    if request.require_canonical {
        let canonical = canonical_tar(&entries, request.policy)?;
        if canonical != uncompressed {
            return Err(archive_issue(
                "non-canonical-archive",
                "layer",
                "Mantle-profile archive does not reproduce canonical bytes",
            ));
        }
        if compress(&canonical, profile)? != request.blob {
            return Err(archive_issue(
                "non-canonical-compression",
                "layer",
                "Mantle-profile compressed bytes do not reproduce exactly",
            ));
        }
        Ok("mantle-canonical-exact".to_string())
    } else {
        Ok("external-safe-inspection".to_string())
    }
}

#[cfg(test)]
pub(crate) fn inspect_archive(
    media_type: &str,
    blob: &[u8],
    policy: &ArchivePolicy,
    max_depth: usize,
    max_entries: usize,
    max_uncompressed_bytes: u64,
    require_canonical: bool,
) -> Result<String, ProjectionIssue> {
    inspect_archive_with_options(ArchiveInspection {
        media_type,
        blob,
        policy,
        limits: ArchiveLimits { max_depth, max_entries },
        max_uncompressed_bytes,
        require_canonical,
    })
}
