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
const EMPTY_ARCHIVE_MIN_BYTES: usize = TAR_BLOCK_BYTES * TAR_END_BLOCKS;
const GZIP_LEVEL: u32 = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LayerPayload {
    pub blob: Vec<u8>,
    pub uncompressed: Vec<u8>,
    pub canonical_archive: bool,
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

fn join_mount(mount: &str, internal: &str) -> String {
    if internal == ROOT_ENTRY {
        mount.to_string()
    } else {
        format!("{mount}/{internal}")
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
        let relative_path = join_mount(&entry.relative_path, &source.relative_path);
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
    Ok(mounted)
}

fn validate_link(path: &str, target: &str, max_depth: usize) -> bool {
    if target.is_empty() || target.starts_with('/') || target.contains('\0') || target.contains('\\') {
        return false;
    }
    let mut stack = path.rsplit_once('/').map_or_else(Vec::new, |(parent, _)| parent.split('/').collect::<Vec<_>>());
    for component in target.split('/') {
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

fn canonical_entries(
    layer: &ProjectionLayer,
    objects: &BTreeMap<String, MaterializedObject>,
    max_depth: usize,
    max_entries: usize,
) -> Result<Vec<ObjectEntry>, ProjectionIssue> {
    let mut result = Vec::new();
    for entry in &layer.entries {
        let object = objects
            .get(&entry.object_ref)
            .ok_or_else(|| archive_issue("missing-object", &entry.object_ref, "projection object is unavailable"))?;
        result.extend(mounted_entries(entry, object, max_depth)?);
    }
    result.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if result.len() > max_entries {
        return Err(archive_issue(
            "archive-entry-bound",
            &layer.role,
            "expanded archive entries exceed the admitted bound",
        ));
    }
    let mut paths = BTreeSet::new();
    for entry in &result {
        if !paths.insert(entry.relative_path.as_str()) {
            return Err(archive_issue(
                "archive-path-collision",
                &entry.relative_path,
                "multiple objects project to the same archive path",
            ));
        }
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
                if !validate_link(&entry.relative_path, target, max_depth) {
                    return Err(archive_issue(
                        "escaping-link",
                        &entry.relative_path,
                        "symlink target is absolute, escaping, or too deep",
                    ));
                }
            }
        }
    }
    let kinds = result.iter().map(|entry| (entry.relative_path.as_str(), entry.kind)).collect::<BTreeMap<_, _>>();
    for entry in &result {
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
    Ok(result)
}

fn append_entry(
    builder: &mut Builder<Vec<u8>>,
    entry: &ObjectEntry,
    policy: &ArchivePolicy,
) -> Result<(), ProjectionIssue> {
    let mut header = Header::new_gnu();
    let (entry_type, size, mode) = match entry.kind {
        ObjectEntryKind::File => (EntryType::Regular, entry.data.len() as u64, policy.file_mode),
        ObjectEntryKind::Directory => (EntryType::Directory, 0_u64, policy.directory_mode),
        ObjectEntryKind::Symlink => (EntryType::Symlink, 0_u64, policy.symlink_mode),
    };
    header.set_entry_type(entry_type);
    header.set_size(size);
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

pub(crate) fn layer_payload(
    layer: &ProjectionLayer,
    objects: &BTreeMap<String, MaterializedObject>,
    policy: &ArchivePolicy,
    max_depth: usize,
    max_entries: usize,
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
            if object.entries.len() != 1
                || object.entries[0].kind != ObjectEntryKind::File
                || object.entries[0].relative_path != ROOT_ENTRY
                || object.entries[0].link_target.is_some()
            {
                return Err(archive_issue(
                    "exact-object-shape",
                    &projection_entry.object_ref,
                    "exact blob requires one root regular file",
                ));
            }
            let bytes = object.entries[0].data.clone();
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
            let entries = canonical_entries(layer, objects, max_depth, max_entries)?;
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

fn parse_tar(bytes: &[u8], max_depth: usize, max_entries: usize) -> Result<Vec<ObjectEntry>, ProjectionIssue> {
    if bytes.len() < EMPTY_ARCHIVE_MIN_BYTES {
        return Err(archive_issue("truncated-archive", "layer", "tar archive is truncated"));
    }
    let mut entries = Vec::new();
    let mut archive = Archive::new(Cursor::new(bytes));
    let archive_entries =
        archive.entries().map_err(|error| archive_issue("tar-entries", "layer", &error.to_string()))?;
    for entry_result in archive_entries {
        if entries.len() >= max_entries {
            return Err(archive_issue("archive-entry-bound", "layer", "archive entry count exceeds the bound"));
        }
        let mut entry = entry_result.map_err(|error| archive_issue("tar-entry", "layer", &error.to_string()))?;
        let path = entry
            .path()
            .map_err(|error| archive_issue("tar-path", "layer", &error.to_string()))?
            .to_str()
            .ok_or_else(|| archive_issue("non-utf8-path", "layer", "archive path is not UTF-8"))?
            .trim_end_matches('/')
            .to_string();
        if !safe_relative_path(&path, max_depth) {
            return Err(archive_issue("unsafe-archive-path", path, "archive path is absolute, escaping, or too deep"));
        }
        let kind = match entry.header().entry_type() {
            value if value.is_file() => ObjectEntryKind::File,
            value if value.is_dir() => ObjectEntryKind::Directory,
            value if value.is_symlink() => ObjectEntryKind::Symlink,
            _ => {
                return Err(archive_issue(
                    "unsupported-archive-entry",
                    path,
                    "hard links and special files are forbidden",
                ));
            }
        };
        let link_target = if kind == ObjectEntryKind::Symlink {
            let target = entry
                .link_name()
                .map_err(|error| archive_issue("tar-link", &path, &error.to_string()))?
                .ok_or_else(|| archive_issue("missing-link-target", &path, "archive symlink target is missing"))?;
            let target = target
                .to_str()
                .ok_or_else(|| archive_issue("non-utf8-link", &path, "archive symlink target is not UTF-8"))?
                .to_string();
            if !validate_link(&path, &target, max_depth) {
                return Err(archive_issue("escaping-link", &path, "archive symlink escapes its root"));
            }
            Some(target)
        } else {
            None
        };
        let mut data = Vec::new();
        if kind == ObjectEntryKind::File {
            entry
                .read_to_end(&mut data)
                .map_err(|error| archive_issue("tar-content", &path, &error.to_string()))?;
        }
        entries.push(ObjectEntry {
            relative_path: path,
            kind,
            data,
            link_target,
        });
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

pub(crate) fn inspect_archive(
    media_type: &str,
    blob: &[u8],
    policy: &ArchivePolicy,
    max_depth: usize,
    max_entries: usize,
    max_uncompressed_bytes: u64,
    require_canonical: bool,
) -> Result<String, ProjectionIssue> {
    let Some(profile) = archive_profile(media_type) else {
        return Ok("exact-blob".to_string());
    };
    let uncompressed = decompress(blob, profile, max_uncompressed_bytes)?;
    let entries = parse_tar(&uncompressed, max_depth, max_entries)?;
    if require_canonical {
        let canonical = canonical_tar(&entries, policy)?;
        if canonical != uncompressed {
            return Err(archive_issue(
                "non-canonical-archive",
                "layer",
                "Mantle-profile archive does not reproduce canonical bytes",
            ));
        }
        if compress(&canonical, profile)? != blob {
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
