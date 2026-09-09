//! Thin filesystem shell for OCI projection export and import.
//!
//! r[impl kernel_bundle_oci.export]
//! r[impl kernel_bundle_oci.import]
//! r[related kernel_bundle_oci.reports]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use tempfile::NamedTempFile;
use tempfile::TempDir;

use crate::frontend_artifact_store::frontend_artifact_identity;
use crate::frontend_artifact_store::frontend_artifact_is_available;
use crate::frontend_artifact_store::import_frontend_artifact;
use crate::frontend_artifact_store::materialize_frontend_artifact;
use crate::oci_projection::LayoutFacts;
use crate::oci_projection::MaterializedObject;
use crate::oci_projection::OCI_BLOB_DIR;
use crate::oci_projection::OCI_BLOB_MAX_BYTES;
use crate::oci_projection::OCI_DOCUMENT_MAX_BYTES;
use crate::oci_projection::OCI_EXPORT_REPORT_FILENAME;
use crate::oci_projection::OCI_INDEX_FILENAME;
use crate::oci_projection::OCI_INPUT_MAX_BYTES;
use crate::oci_projection::OCI_LAYER_MAX_COUNT;
use crate::oci_projection::OCI_LAYOUT_FILENAME;
use crate::oci_projection::ObjectEntry;
use crate::oci_projection::ObjectEntryKind;
use crate::oci_projection::OciExportReport;
use crate::oci_projection::OciImportReport;
use crate::oci_projection::OciIndexDocument;
use crate::oci_projection::OciManifestDocument;
use crate::oci_projection::OciProjection;
use crate::oci_projection::PlannedBlob;
use crate::oci_projection::SourceAdmissionBundle;
use crate::oci_projection::attach_imported_refs;
use crate::oci_projection::build_export_plan;
use crate::oci_projection::canonical_mantle_ref;
use crate::oci_projection::export_report;
use crate::oci_projection::is_sha256_digest;
use crate::oci_projection::validate_import;
use crate::oci_projection::validate_projection;
use crate::oci_projection::validate_source_admissions;
use crate::oci_projection::verify_import_report;

const MAX_OBJECT_ENTRIES: usize = 1_000_000;
const INITIAL_WORKLIST_CAPACITY: usize = 64;
const BOUND_PROBE_BYTES: u64 = 1;
const OCI_CONTROL_DESCRIPTOR_COUNT: usize = 2;
const ROOT_ENTRY: &str = ".";
const SHA256_PREFIX: &str = "sha256:";
const EXPORT_REPORT_COPY_SCHEMA: &str = "mantle-oci-export-report-v1";
const ALLOWED_LAYOUT_ROOT_ENTRIES: &[&str] = &[
    OCI_LAYOUT_FILENAME,
    OCI_INDEX_FILENAME,
    "blobs",
    OCI_EXPORT_REPORT_FILENAME,
];

#[derive(Clone, Debug)]
pub struct ExportRequest<'a> {
    pub projection_path: &'a Path,
    pub spec_material_path: &'a Path,
    pub source_admissions_path: &'a Path,
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
}

#[derive(Clone, Debug)]
pub struct ImportRequest<'a> {
    pub layout_dir: &'a Path,
    pub report_path: &'a Path,
    pub state_dir: &'a Path,
}

#[cfg(unix)]
fn open_regular_no_follow(path: &Path) -> Result<(File, fs::Metadata), String> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| format!("opening non-symlink file {}: {error}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("reading opened metadata for {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("expected an opened regular file: {}", path.display()));
    }
    Ok((file, metadata))
}

#[cfg(not(unix))]
fn open_regular_no_follow(path: &Path) -> Result<(File, fs::Metadata), String> {
    Err(format!("secure no-follow file reads are unsupported on this platform: {}", path.display()))
}

fn bounded_read_limit(max_bytes: u64) -> Result<u64, String> {
    max_bytes.checked_add(BOUND_PROBE_BYTES).ok_or_else(|| "bounded read limit overflowed".to_string())
}

fn byte_len_u64(bytes: &[u8]) -> Result<u64, String> {
    u64::try_from(bytes.len()).map_err(|_| "byte length does not fit u64".to_string())
}

pub(crate) fn read_bounded_regular(path: &Path, max_bytes: u64) -> Result<Vec<u8>, String> {
    let (file, metadata) = open_regular_no_follow(path)?;
    if metadata.len() > max_bytes {
        return Err(format!("{} exceeds the {max_bytes}-byte bound", path.display()));
    }
    let file_size_bytes =
        usize::try_from(metadata.len()).map_err(|_| format!("{} is too large for this host", path.display()))?;
    let mut bytes = Vec::with_capacity(file_size_bytes);
    file.take(bounded_read_limit(max_bytes)?)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("reading {}: {error}", path.display()))?;
    if byte_len_u64(&bytes)? > max_bytes {
        return Err(format!("{} grew beyond the {max_bytes}-byte bound while reading", path.display()));
    }
    Ok(bytes)
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("output has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("creating {}: {error}", parent.display()))?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("creating {}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("writing {}: {error}", path.display()))?;
    file.sync_all().map_err(|error| format!("syncing {}: {error}", path.display()))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("creating {}: {error}", parent.display()))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| format!("creating temporary report in {}: {error}", parent.display()))?;
    temporary
        .write_all(bytes)
        .map_err(|error| format!("writing temporary report for {}: {error}", path.display()))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| format!("syncing temporary report for {}: {error}", path.display()))?;
    temporary.persist(path).map_err(|error| format!("persisting {}: {}", path.display(), error.error))?;
    Ok(())
}

fn relative_string(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("deriving path below {}: {error}", root.display()))?;
    if relative.as_os_str().is_empty() {
        return Ok(ROOT_ENTRY.to_string());
    }
    relative
        .to_str()
        .map(|value| value.replace(std::path::MAIN_SEPARATOR, "/"))
        .ok_or_else(|| format!("materialized path is not UTF-8: {}", path.display()))
}

fn read_file_with_budget(path: &Path, remaining: &mut u64) -> Result<Vec<u8>, String> {
    let remaining_before_bytes = *remaining;
    let (file, metadata) = open_regular_no_follow(path)?;
    if metadata.len() > *remaining {
        return Err(format!("materialized objects exceed the admitted byte bound at {}", path.display()));
    }
    let file_size_bytes =
        usize::try_from(metadata.len()).map_err(|_| format!("{} is too large for this host", path.display()))?;
    let mut bytes = Vec::with_capacity(file_size_bytes);
    file.take(bounded_read_limit(*remaining)?)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("reading {}: {error}", path.display()))?;
    let bytes_read = byte_len_u64(&bytes)?;
    if bytes_read > *remaining {
        return Err(format!("materialized objects exceed the admitted byte bound at {}", path.display()));
    }
    *remaining = remaining
        .checked_sub(bytes_read)
        .ok_or_else(|| "materialized object byte counter underflowed".to_string())?;
    assert!(*remaining <= remaining_before_bytes, "materialized byte budget must not increase");
    assert_eq!(
        remaining_before_bytes.checked_sub(*remaining),
        Some(bytes_read),
        "materialized byte budget must account for exact bytes read"
    );
    Ok(bytes)
}

fn sorted_children(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut children = fs::read_dir(path)
        .map_err(|error| format!("reading directory {}: {error}", path.display()))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| format!("reading directory entry below {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    Ok(children)
}

fn collect_object(root: &Path, artifact_ref: String, remaining_bytes: &mut u64) -> Result<MaterializedObject, String> {
    assert!(!root.as_os_str().is_empty(), "materialized object root must not be empty");
    assert!(!artifact_ref.is_empty(), "materialized object ref must not be empty");
    let mut entries = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY);
    let mut worklist = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY);
    worklist.push(root.to_path_buf());
    while let Some(path) = worklist.pop() {
        if entries.len() >= MAX_OBJECT_ENTRIES {
            return Err(format!("materialized object exceeds {MAX_OBJECT_ENTRIES} entries"));
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("reading metadata for {}: {error}", path.display()))?;
        let relative_path = relative_string(root, &path)?;
        if metadata.file_type().is_symlink() {
            let target =
                fs::read_link(&path).map_err(|error| format!("reading symlink {}: {error}", path.display()))?;
            let link_target = target
                .to_str()
                .ok_or_else(|| format!("symlink target is not UTF-8: {}", path.display()))?
                .to_string();
            entries.push(ObjectEntry {
                relative_path,
                kind: ObjectEntryKind::Symlink,
                data: Vec::new(),
                link_target: Some(link_target),
            });
        } else if metadata.is_file() {
            entries.push(ObjectEntry {
                relative_path,
                kind: ObjectEntryKind::File,
                data: read_file_with_budget(&path, remaining_bytes)?,
                link_target: None,
            });
        } else if metadata.is_dir() {
            entries.push(ObjectEntry {
                relative_path,
                kind: ObjectEntryKind::Directory,
                data: Vec::new(),
                link_target: None,
            });
            for child in sorted_children(&path)?.into_iter().rev() {
                worklist.push(child);
            }
        } else {
            return Err(format!("unsupported special file in materialized object: {}", path.display()));
        }
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    assert!(!entries.is_empty(), "materialized object must include its root entry");
    assert!(entries.len() <= MAX_OBJECT_ENTRIES, "materialized object entries must stay bounded");
    Ok(MaterializedObject { artifact_ref, entries })
}

fn materialize_projection_objects(
    projection: &OciProjection,
    state_dir: &Path,
) -> Result<(TempDir, BTreeMap<String, MaterializedObject>), String> {
    let temporary =
        tempfile::tempdir().map_err(|error| format!("creating OCI materialization staging directory: {error}"))?;
    let references = projection
        .layers
        .iter()
        .flat_map(|layer| layer.entries.iter())
        .map(|entry| {
            canonical_mantle_ref(&entry.object_ref)
                .ok_or_else(|| format!("invalid projection object ref: {}", entry.object_ref))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    for artifact_ref in &references {
        if !frontend_artifact_is_available(state_dir, artifact_ref)? {
            return Err(format!("projection object is absent from the Mantle CAS: {artifact_ref}"));
        }
    }
    let initial_budget_bytes = projection.bounds.max_total_bytes;
    let mut remaining_bytes = initial_budget_bytes;
    let object_count_max = references.len();
    let mut objects = BTreeMap::new();
    for (index, artifact_ref) in references.into_iter().enumerate() {
        let destination = temporary.path().join(format!("object-{index}"));
        let stored = materialize_frontend_artifact(state_dir, &artifact_ref, &destination)?
            .ok_or_else(|| format!("projection object is absent from the Mantle CAS: {artifact_ref}"))?;
        if stored.artifact_ref != artifact_ref {
            return Err(format!("materialized artifact ref mismatch for {artifact_ref}"));
        }
        let recomputed = frontend_artifact_identity(&destination)?;
        if recomputed != artifact_ref {
            return Err(format!("materialized artifact identity mismatch: expected {artifact_ref}, got {recomputed}"));
        }
        let object = collect_object(&destination, artifact_ref.clone(), &mut remaining_bytes)?;
        if objects.len() >= object_count_max {
            return Err("projection object count exceeded admitted references".to_string());
        }
        objects.insert(artifact_ref, object);
    }
    assert_eq!(objects.len(), object_count_max, "every admitted projection ref must materialize exactly once");
    assert!(remaining_bytes <= initial_budget_bytes, "projection materialization budget must not increase");
    Ok((temporary, objects))
}

fn write_export_plan(stage: &Path, plan: &crate::oci_projection::ExportPlan) -> Result<(), String> {
    write_new_synced(&stage.join(OCI_LAYOUT_FILENAME), &plan.oci_layout_bytes)?;
    write_new_synced(&stage.join(OCI_INDEX_FILENAME), &plan.index_bytes)?;
    for PlannedBlob { digest, bytes } in &plan.blobs {
        let hex = digest
            .strip_prefix(SHA256_PREFIX)
            .ok_or_else(|| format!("planned blob lacks SHA-256 prefix: {digest}"))?;
        write_new_synced(&stage.join(OCI_BLOB_DIR).join(hex), bytes)?;
    }
    sync_directory(&stage.join(OCI_BLOB_DIR))?;
    sync_directory(&stage.join("blobs"))?;
    Ok(())
}

fn write_pulled_layout(stage: &Path, plan: &crate::oci_registry::RegistryPullPlan) -> Result<(), String> {
    write_new_synced(&stage.join(OCI_LAYOUT_FILENAME), &plan.oci_layout_bytes)?;
    write_new_synced(&stage.join(OCI_INDEX_FILENAME), &plan.index_bytes)?;
    write_new_synced(&stage.join(OCI_EXPORT_REPORT_FILENAME), &plan.export_report_bytes)?;
    for (digest, bytes) in &plan.descriptor_blobs {
        let hex = digest
            .strip_prefix(SHA256_PREFIX)
            .ok_or_else(|| format!("pulled blob lacks SHA-256 prefix: {digest}"))?;
        write_new_synced(&stage.join(OCI_BLOB_DIR).join(hex), bytes)?;
    }
    sync_directory(&stage.join(OCI_BLOB_DIR))?;
    sync_directory(&stage.join("blobs"))?;
    Ok(())
}

fn parse_projection(path: &Path) -> Result<OciProjection, String> {
    let bytes = read_bounded_regular(path, OCI_INPUT_MAX_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parsing OCI projection {}: {error}", path.display()))
}

fn parse_source_admissions(path: &Path) -> Result<SourceAdmissionBundle, String> {
    let bytes = read_bounded_regular(path, OCI_INPUT_MAX_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parsing OCI source admissions {}: {error}", path.display()))
}

fn serialize_pretty<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("syncing directory {}: {error}", path.display()))
}

#[cfg(target_os = "linux")]
fn publish_directory_no_replace(staging: &Path, destination: &Path) -> Result<(), String> {
    let staging =
        CString::new(staging.as_os_str().as_bytes()).map_err(|_| "OCI staging path contains a NUL byte".to_string())?;
    let destination = CString::new(destination.as_os_str().as_bytes())
        .map_err(|_| "OCI destination path contains a NUL byte".to_string())?;
    crate::linux_rename::rename_no_replace(libc::AT_FDCWD, &staging, libc::AT_FDCWD, &destination)
        .map_err(|error| format!("atomically publishing OCI layout without replacement: {error}"))
}

#[cfg(not(target_os = "linux"))]
fn publish_directory_no_replace(_staging: &Path, _destination: &Path) -> Result<(), String> {
    Err("atomic no-replace OCI publication is unsupported on this platform".to_string())
}

pub fn export_oci_layout(request: &ExportRequest<'_>) -> Result<OciExportReport, String> {
    if request.output_dir.exists() {
        return Err(format!("OCI output already exists: {}", request.output_dir.display()));
    }
    assert!(!request.output_dir.exists(), "OCI export requires an absent destination");
    assert_ne!(request.output_dir, request.state_dir, "OCI export destination must not alias state storage");
    let projection = parse_projection(request.projection_path)?;
    let spec_material = read_bounded_regular(request.spec_material_path, OCI_INPUT_MAX_BYTES)?;
    let source_admissions = parse_source_admissions(request.source_admissions_path)?;
    let mut admission_issues = validate_projection(&projection, &spec_material);
    admission_issues.extend(validate_source_admissions(&projection, &source_admissions));
    if !admission_issues.is_empty() {
        return Err(format!(
            "OCI projection admission failed: {}",
            serde_json::to_string(&admission_issues).unwrap_or_default()
        ));
    }
    let (_materialized, objects) = materialize_projection_objects(&projection, request.state_dir)?;
    let plan = build_export_plan(&projection, &spec_material, &source_admissions, &objects).map_err(|issues| {
        format!("OCI projection planning failed: {}", serde_json::to_string(&issues).unwrap_or_default())
    })?;
    let outcome = export_report(&plan)?;
    if outcome.schema != EXPORT_REPORT_COPY_SCHEMA {
        return Err("internal OCI export report schema disagreement".to_string());
    }
    let report_bytes = serialize_pretty(&outcome)?;

    let parent = request.output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("creating OCI output parent {}: {error}", parent.display()))?;
    let stage = tempfile::Builder::new()
        .prefix(".mantle-oci-export-")
        .tempdir_in(parent)
        .map_err(|error| format!("creating OCI export staging directory: {error}"))?;
    write_export_plan(stage.path(), &plan)?;
    write_new_synced(&stage.path().join(OCI_EXPORT_REPORT_FILENAME), &report_bytes)?;
    let staged_facts = read_layout_facts(stage.path())?;
    let staged_admission = validate_import(&staged_facts).map_err(|issues| {
        format!("staged OCI self-verification failed: {}", serde_json::to_string(&issues).unwrap_or_default())
    })?;
    if staged_admission.state != "admitted"
        || staged_admission.projection_blake3.as_deref() != Some(&plan.projection_blake3)
    {
        return Err("staged OCI self-verification did not preserve admitted projection identity".to_string());
    }
    sync_directory(stage.path())?;
    let stage_path = stage.keep();
    if let Err(error) = publish_directory_no_replace(&stage_path, request.output_dir) {
        let _cleanup_result = fs::remove_dir_all(&stage_path);
        return Err(error);
    }
    sync_directory(parent)?;
    Ok(outcome)
}

fn descriptor_blob_path(layout_dir: &Path, digest: &str) -> Result<PathBuf, String> {
    if !is_sha256_digest(digest) {
        return Err(format!("invalid OCI descriptor digest: {digest}"));
    }
    let hex = digest.strip_prefix(SHA256_PREFIX).ok_or_else(|| format!("invalid SHA-256 descriptor: {digest}"))?;
    Ok(layout_dir.join(OCI_BLOB_DIR).join(hex))
}

fn read_descriptor_blob(layout_dir: &Path, digest: &str, total: &mut u64) -> Result<Vec<u8>, String> {
    let bytes = read_bounded_regular(&descriptor_blob_path(layout_dir, digest)?, OCI_BLOB_MAX_BYTES)?;
    *total = total
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| "OCI import byte count overflowed".to_string())?;
    if *total > OCI_BLOB_MAX_BYTES {
        return Err(format!("OCI referenced blobs exceed the {OCI_BLOB_MAX_BYTES}-byte aggregate bound"));
    }
    Ok(bytes)
}

fn check_blob_directory(layout_dir: &Path, expected: &BTreeSet<String>) -> Result<(), String> {
    assert!(!layout_dir.as_os_str().is_empty(), "OCI layout directory must not be empty");
    assert!(!expected.is_empty(), "OCI layout must reference at least one descriptor blob");
    let directory = layout_dir.join(OCI_BLOB_DIR);
    let metadata = fs::symlink_metadata(&directory)
        .map_err(|error| format!("reading blob directory {}: {error}", directory.display()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!("OCI blob path is not a regular directory: {}", directory.display()));
    }
    let actual = sorted_children(&directory)?
        .into_iter()
        .map(|path| {
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("reading blob metadata {}: {error}", path.display()))?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(format!("OCI blob entry is not a regular file: {}", path.display()));
            }
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("OCI blob filename is not UTF-8: {}", path.display()))?;
            Ok(format!("{SHA256_PREFIX}{name}"))
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    if &actual != expected {
        return Err("OCI blob directory contains missing or unreferenced descriptors".to_string());
    }
    assert_eq!(&actual, expected, "admitted blob directory must match descriptor closure");
    assert!(!actual.is_empty(), "admitted blob directory must not be empty");
    Ok(())
}

fn check_layout_root_entries(layout_dir: &Path) -> Result<(), String> {
    for path in sorted_children(layout_dir)? {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("OCI layout root entry is not UTF-8: {}", path.display()))?;
        if !ALLOWED_LAYOUT_ROOT_ENTRIES.contains(&name) {
            return Err(format!("OCI layout contains an unexpected root entry: {name}"));
        }
    }
    Ok(())
}

pub fn read_layout_facts(layout_dir: &Path) -> Result<LayoutFacts, String> {
    assert!(!layout_dir.as_os_str().is_empty(), "OCI layout path must not be empty");
    assert!(!ALLOWED_LAYOUT_ROOT_ENTRIES.is_empty(), "OCI root allowlist must not be empty");
    let root_metadata = fs::symlink_metadata(layout_dir)
        .map_err(|error| format!("reading OCI layout root {}: {error}", layout_dir.display()))?;
    if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
        return Err(format!("OCI layout root must be a non-symlink directory: {}", layout_dir.display()));
    }
    check_layout_root_entries(layout_dir)?;
    let oci_layout_bytes = read_bounded_regular(&layout_dir.join(OCI_LAYOUT_FILENAME), OCI_DOCUMENT_MAX_BYTES)?;
    let index_bytes = read_bounded_regular(&layout_dir.join(OCI_INDEX_FILENAME), OCI_DOCUMENT_MAX_BYTES)?;
    let index: OciIndexDocument =
        serde_json::from_slice(&index_bytes).map_err(|error| format!("parsing OCI index: {error}"))?;
    if index.manifests.len() != 1 {
        return Err("OCI index must reference exactly one manifest before blob reads".to_string());
    }
    let mut total = 0_u64;
    let manifest_descriptor = index.manifests[0].clone();
    let manifest_bytes = read_descriptor_blob(layout_dir, &manifest_descriptor.digest, &mut total)?;
    let manifest: OciManifestDocument =
        serde_json::from_slice(&manifest_bytes).map_err(|error| format!("parsing OCI manifest: {error}"))?;
    if manifest.layers.is_empty() || manifest.layers.len() > OCI_LAYER_MAX_COUNT {
        return Err(format!("OCI manifest layer count exceeds the {OCI_LAYER_MAX_COUNT}-layer bound"));
    }
    let descriptor_count = manifest
        .layers
        .len()
        .checked_add(OCI_CONTROL_DESCRIPTOR_COUNT)
        .ok_or_else(|| "OCI descriptor count overflowed".to_string())?;
    let mut descriptors = Vec::with_capacity(descriptor_count);
    descriptors.push(manifest_descriptor);
    descriptors.push(manifest.config);
    descriptors.extend(manifest.layers);
    let descriptor_count_max = descriptors.len();
    let mut expected = BTreeSet::new();
    let mut blobs = BTreeMap::<String, Vec<u8>>::new();
    for descriptor in descriptors {
        if !expected.insert(descriptor.digest.clone()) {
            return Err(format!("duplicate OCI descriptor before admission: {}", descriptor.digest));
        }
        let bytes = if let Some(existing) = blobs.get(&descriptor.digest) {
            existing.clone()
        } else if descriptor.digest == index.manifests[0].digest {
            manifest_bytes.clone()
        } else {
            read_descriptor_blob(layout_dir, &descriptor.digest, &mut total)?
        };
        if blobs.len() >= descriptor_count_max {
            return Err("OCI blob count exceeded descriptor closure".to_string());
        }
        blobs.insert(descriptor.digest, bytes);
    }
    check_blob_directory(layout_dir, &expected)?;
    let producer_summary = read_optional_export_report(layout_dir)?;
    assert_eq!(blobs.len(), expected.len(), "every admitted descriptor must have one blob");
    assert!(blobs.len() <= descriptor_count_max, "admitted descriptor blobs must stay bounded");
    Ok(LayoutFacts {
        oci_layout_bytes,
        index_bytes,
        blobs,
        export_report: producer_summary,
    })
}

fn read_optional_export_report(layout_dir: &Path) -> Result<Option<OciExportReport>, String> {
    let producer_summary_path = layout_dir.join(OCI_EXPORT_REPORT_FILENAME);
    match fs::symlink_metadata(&producer_summary_path) {
        Ok(_) => {
            let bytes = read_bounded_regular(&producer_summary_path, OCI_DOCUMENT_MAX_BYTES)?;
            serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| format!("parsing Mantle OCI export report: {error}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("checking Mantle OCI export report: {error}")),
    }
}

fn admit_blobs(facts: &LayoutFacts, state_dir: &Path) -> Result<BTreeMap<String, String>, String> {
    assert!(!facts.blobs.is_empty(), "OCI admission requires descriptor blobs");
    assert!(!state_dir.as_os_str().is_empty(), "OCI admission state directory must not be empty");
    let temporary = tempfile::tempdir().map_err(|error| format!("creating OCI import staging directory: {error}"))?;
    let blob_count_max = facts.blobs.len();
    let mut refs = BTreeMap::new();
    for (index, (digest, bytes)) in facts.blobs.iter().enumerate() {
        let path = temporary.path().join(format!("blob-{index}"));
        write_new_synced(&path, bytes)?;
        let admission = import_frontend_artifact(&path, state_dir)?;
        let verification_path = temporary.path().join(format!("verified-blob-{index}"));
        let materialized = materialize_frontend_artifact(state_dir, &admission.artifact_ref, &verification_path)?
            .ok_or_else(|| format!("imported OCI blob disappeared before verification: {digest}"))?;
        if frontend_artifact_identity(&verification_path)? != admission.artifact_ref
            || read_bounded_regular(&verification_path, OCI_BLOB_MAX_BYTES)? != *bytes
            || materialized.artifact_ref != admission.artifact_ref
        {
            return Err(format!("imported OCI blob failed exact CAS verification: {digest}"));
        }
        if refs.len() >= blob_count_max {
            return Err("OCI admitted ref count exceeded descriptor blobs".to_string());
        }
        refs.insert(digest.clone(), admission.artifact_ref);
    }
    assert_eq!(refs.len(), blob_count_max, "every descriptor blob must receive one admitted ref");
    assert_eq!(refs.len(), facts.blobs.len(), "admitted refs must preserve descriptor closure cardinality");
    Ok(refs)
}

pub fn publish_pulled_layout(plan: &crate::oci_registry::RegistryPullPlan, output_dir: &Path) -> Result<(), String> {
    if output_dir.exists() {
        return Err(format!("OCI pull output already exists: {}", output_dir.display()));
    }
    if plan.descriptor_blobs.is_empty() || plan.export_report_bytes.is_empty() {
        return Err("OCI pull plan is missing descriptor or export-report bytes".to_string());
    }
    let parent = output_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("creating OCI pull output parent {}: {error}", parent.display()))?;
    let stage = tempfile::Builder::new()
        .prefix(".mantle-oci-pull-")
        .tempdir_in(parent)
        .map_err(|error| format!("creating OCI pull staging directory: {error}"))?;
    write_pulled_layout(stage.path(), plan)?;
    let facts = read_layout_facts(stage.path())?;
    let preview = validate_import(&facts).map_err(|issues| {
        format!("pulled OCI self-verification failed: {}", serde_json::to_string(&issues).unwrap_or_default())
    })?;
    if preview.state != "admitted" || preview.layout_blake3 != plan.layout_blake3 {
        return Err("pulled OCI self-verification did not preserve admitted layout identity".to_string());
    }
    assert_eq!(preview.projection_blake3.as_deref(), Some(plan.projection_blake3.as_str()));
    assert_eq!(facts.blobs.len(), plan.descriptor_blobs.len(), "pulled descriptor closure must stay exact");
    sync_directory(stage.path())?;
    let stage_path = stage.keep();
    if let Err(error) = publish_directory_no_replace(&stage_path, output_dir) {
        let _cleanup_result = fs::remove_dir_all(&stage_path);
        return Err(error);
    }
    sync_directory(parent)
}

pub fn import_oci_layout(request: &ImportRequest<'_>) -> Result<OciImportReport, String> {
    let facts = read_layout_facts(request.layout_dir)?;
    let mut outcome = validate_import(&facts).map_err(|issues| {
        format!("OCI import validation failed: {}", serde_json::to_string(&issues).unwrap_or_default())
    })?;
    let refs = admit_blobs(&facts, request.state_dir)?;
    attach_imported_refs(&mut outcome, &refs)?;
    verify_import_report(&outcome)?;
    let report_bytes = serialize_pretty(&outcome)?;
    write_atomic(request.report_path, &report_bytes)?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use super::ExportRequest;
    use super::ImportRequest;
    use super::export_oci_layout;
    use super::import_oci_layout;
    use crate::frontend_artifact_spec::FrontendArtifactAdmissionAttestation;
    use crate::frontend_artifact_store::import_frontend_artifact;
    use crate::oci_projection::ArchivePolicy;
    use crate::oci_projection::EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB;
    use crate::oci_projection::ExternalDigestExpectation;
    use crate::oci_projection::FrontendSpecBinding;
    use crate::oci_projection::LayerMode;
    use crate::oci_projection::OCI_BLOB_DIR;
    use crate::oci_projection::OCI_CONFIG_MEDIA_TYPE;
    use crate::oci_projection::OCI_EXPORT_REPORT_FILENAME;
    use crate::oci_projection::OCI_INDEX_FILENAME;
    use crate::oci_projection::OCI_INDEX_MEDIA_TYPE;
    use crate::oci_projection::OCI_LAYER_MAX_COUNT;
    use crate::oci_projection::OCI_LAYOUT_FILENAME;
    use crate::oci_projection::OCI_LAYOUT_VERSION;
    use crate::oci_projection::OCI_MANIFEST_MEDIA_TYPE;
    use crate::oci_projection::OCI_PROJECTION_SCHEMA;
    use crate::oci_projection::OciProjection;
    use crate::oci_projection::Platform;
    use crate::oci_projection::ProjectionAdmission;
    use crate::oci_projection::ProjectionBounds;
    use crate::oci_projection::ProjectionEntry;
    use crate::oci_projection::ProjectionLayer;
    use crate::oci_projection::ProjectionObjectAdmission;
    use crate::oci_projection::RoundTripExpectation;
    use crate::oci_projection::SOURCE_ADMISSION_BUNDLE_SCHEMA;
    use crate::oci_projection::SourceAdmissionBundle;
    use crate::oci_projection::SourceArtifactAdmission;
    use crate::oci_projection::blake3_hex;
    use crate::oci_projection::reduce_source_admission;
    use crate::oci_projection::seal_projection;
    use crate::oci_projection::sha256_digest;

    const KERNEL_BYTES: &[u8] = b"bounded-kernel-image";
    const MODULE_BYTES: &[u8] = b"bounded-module";
    const SPEC_BYTES: &[u8] = b"accepted kernel bundle specification";
    const HEX_CHARS_PER_BYTE: usize = 2;
    const HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
    const EXPECTED_LAYER_COUNT: usize = 2;
    const OCI_SCHEMA_VERSION: u16 = 2;

    fn identity(label: &str, byte: char) -> String {
        let digest = std::iter::repeat_n(byte, HEX_LENGTH).collect::<String>();
        format!("onix:{label}:blake3:{digest}")
    }

    fn source_admission(object_ref: String, target_identity: String, spec_hash: &str) -> SourceArtifactAdmission {
        let digest = object_ref
            .strip_prefix("mantle://blake3/")
            .expect("fixture ref should use the Mantle scheme")
            .to_string();
        SourceArtifactAdmission {
            spec_id: "kernel-bundles".to_string(),
            spec_version: "1".to_string(),
            spec_hash_algorithm: "blake3".to_string(),
            spec_hash: spec_hash.to_string(),
            validator_kind: "onix-kernel-bundle-v1".to_string(),
            validator_ref: "builtin:onix-kernel-bundle-v1".to_string(),
            artifact_kind: "kernel-bundle-component".to_string(),
            artifact_ref: object_ref,
            artifact_digest: Some(format!("blake3:{digest}")),
            target_identity: Some(target_identity),
            build_root: "fixture-build-root".to_string(),
            validation_result: "admitted".to_string(),
            no_hidden_fallback: true,
        }
    }

    fn admission(object_ref: String, target_identity: String, spec_hash: &str) -> ProjectionObjectAdmission {
        reduce_source_admission(&source_admission(object_ref, target_identity, spec_hash))
            .expect("source admission should reduce")
    }

    fn source_bundle(projection: &OciProjection) -> SourceAdmissionBundle {
        SourceAdmissionBundle {
            schema: SOURCE_ADMISSION_BUNDLE_SCHEMA.to_string(),
            admissions: projection
                .object_admissions
                .iter()
                .map(|admission| SourceArtifactAdmission {
                    spec_id: admission.spec_id.clone(),
                    spec_version: admission.spec_version.clone(),
                    spec_hash_algorithm: admission.spec_hash_algorithm.clone(),
                    spec_hash: admission.spec_hash.clone(),
                    validator_kind: admission.validator_kind.clone(),
                    validator_ref: admission.validator_ref.clone(),
                    artifact_kind: admission.artifact_kind.clone(),
                    artifact_ref: admission.artifact_ref.clone(),
                    artifact_digest: admission.artifact_digest.clone(),
                    target_identity: admission.target_identity.clone(),
                    build_root: "fixture-build-root".to_string(),
                    validation_result: admission.validation_result.clone(),
                    no_hidden_fallback: admission.no_hidden_fallback,
                })
                .collect(),
        }
    }

    fn projection(kernel_ref: String, module_ref: String) -> OciProjection {
        let spec_hash = blake3_hex(SPEC_BYTES);
        let kernel_identity = identity("component", '1');
        let module_identity = identity("component", '2');
        let mut value = OciProjection {
            schema: OCI_PROJECTION_SCHEMA.to_string(),
            frontend_spec: FrontendSpecBinding {
                id: "kernel-bundles".to_string(),
                version: "1".to_string(),
                hash_algorithm: "blake3".to_string(),
                hash: spec_hash.clone(),
            },
            admission: ProjectionAdmission {
                validation_result: "admitted".to_string(),
                projection_blake3: None,
                no_hidden_fallback: true,
            },
            platform: Platform {
                architecture: "x86_64".to_string(),
                os: "linux".to_string(),
            },
            bounds: ProjectionBounds::default(),
            archive_policy: ArchivePolicy::default(),
            object_admissions: vec![
                admission(kernel_ref.clone(), kernel_identity.clone(), &spec_hash),
                admission(module_ref.clone(), module_identity.clone(), &spec_hash),
            ],
            layers: vec![
                ProjectionLayer {
                    role: "base-kernel".to_string(),
                    media_type: "application/vnd.onix.kernel.v1".to_string(),
                    mode: LayerMode::ExactBlob,
                    entries: vec![ProjectionEntry {
                        relative_path: "boot/vmlinuz".to_string(),
                        object_ref: kernel_ref,
                        identity: kernel_identity.clone(),
                        size_bytes: KERNEL_BYTES.len() as u64,
                    }],
                    pack_identity: None,
                    annotations: BTreeMap::new(),
                },
                ProjectionLayer {
                    role: "module-pack".to_string(),
                    media_type: "application/vnd.onix.module-pack.v1.tar".to_string(),
                    mode: LayerMode::CanonicalArchive,
                    entries: vec![ProjectionEntry {
                        relative_path: "usr/lib/modules".to_string(),
                        object_ref: module_ref,
                        identity: module_identity.clone(),
                        size_bytes: MODULE_BYTES.len() as u64,
                    }],
                    pack_identity: Some(identity("module-pack", '3')),
                    annotations: BTreeMap::new(),
                },
            ],
            required_annotations: BTreeMap::from([
                (
                    "io.multikernel.kbi.id".to_string(),
                    format!("kbi:sha256:{}", std::iter::repeat_n('7', HEX_LENGTH).collect::<String>()),
                ),
                ("org.onix.bundle.schema".to_string(), "onix-kernel-bundle-v1".to_string()),
            ]),
            expected_external_digests: vec![ExternalDigestExpectation {
                role: EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB.to_string(),
                subject_identity: kernel_identity.clone(),
                digest: sha256_digest(KERNEL_BYTES),
            }],
            round_trip: RoundTripExpectation {
                kernel_build_identity: identity("kernel-build", '4'),
                bundle_identity: identity("kernel-bundle", '5'),
                manifest_identity: identity("manifest", '6'),
                component_identities: vec![kernel_identity, module_identity],
                pack_identities: vec![identity("module-pack", '3')],
            },
            non_claims: vec![
                "no registry publication".to_string(),
                "no kernel compatibility decision".to_string(),
                "no bootability claim".to_string(),
                "no deployability claim".to_string(),
                "no release eligibility claim".to_string(),
                "no signature policy claim".to_string(),
                "frontend semantics remain external".to_string(),
            ],
        };
        seal_projection(&mut value).expect("fixture projection should seal");
        value
    }

    struct Fixture {
        _root: tempfile::TempDir,
        state: std::path::PathBuf,
        spec: std::path::PathBuf,
        projection: std::path::PathBuf,
        source_admissions: std::path::PathBuf,
        layout: std::path::PathBuf,
        import_report: std::path::PathBuf,
    }

    fn fixture() -> Fixture {
        let root = tempfile::tempdir().expect("fixture root should exist");
        let state = root.path().join("state");
        let kernel = root.path().join("vmlinuz");
        fs::write(&kernel, KERNEL_BYTES).expect("kernel fixture should write");
        let modules = root.path().join("modules");
        fs::create_dir_all(modules.join("kernel")).expect("module tree should exist");
        fs::write(modules.join("kernel/module.ko"), MODULE_BYTES).expect("module fixture should write");
        let kernel_ref = import_frontend_artifact(&kernel, &state).expect("kernel should import").artifact_ref;
        let module_ref = import_frontend_artifact(&modules, &state).expect("modules should import").artifact_ref;
        let spec = root.path().join("spec.md");
        fs::write(&spec, SPEC_BYTES).expect("spec fixture should write");
        let projection_value = projection(kernel_ref, module_ref);
        let projection_path = root.path().join("projection.json");
        fs::write(&projection_path, serde_json::to_vec_pretty(&projection_value).expect("projection should serialize"))
            .expect("projection should write");
        let source_admissions = root.path().join("source-admissions.json");
        fs::write(
            &source_admissions,
            serde_json::to_vec_pretty(&source_bundle(&projection_value)).expect("source admissions should serialize"),
        )
        .expect("source admissions should write");
        Fixture {
            state,
            spec,
            projection: projection_path,
            source_admissions,
            layout: root.path().join("layout"),
            import_report: root.path().join("import-report.json"),
            _root: root,
        }
    }

    #[test]
    fn shell_exports_atomically_and_imports_only_after_validation() {
        let fixture = fixture();
        let exported = export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect("OCI export should succeed");
        assert!(fixture.layout.join("oci-layout").is_file());
        assert!(fixture.layout.join("index.json").is_file());
        assert!(fixture.layout.join(OCI_EXPORT_REPORT_FILENAME).is_file());
        assert_eq!(exported.layers.len(), EXPECTED_LAYER_COUNT);

        let imported = import_oci_layout(&ImportRequest {
            layout_dir: &fixture.layout,
            report_path: &fixture.import_report,
            state_dir: &fixture.state,
        })
        .expect("OCI import should succeed");
        assert!(imported.imported);
        assert_eq!(imported.state, "admitted");
        assert!(imported.objects.iter().all(|object| object.artifact_ref.is_some()));
        assert!(fixture.import_report.is_file());
    }

    #[test]
    fn failed_report_commit_leaves_cas_bytes_recoverable_but_unreferenced() {
        let fixture = fixture();
        export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect("OCI export should succeed");
        let blocked_parent = fixture._root.path().join("blocked-report-parent");
        fs::write(&blocked_parent, b"not a directory").expect("report parent blocker should write");
        let blocked_report = blocked_parent.join("report.json");
        let error = import_oci_layout(&ImportRequest {
            layout_dir: &fixture.layout,
            report_path: &blocked_report,
            state_dir: &fixture.state,
        })
        .expect_err("report commit failure must fail the import command");
        assert!(error.contains("creating"));
        assert!(!blocked_report.exists());

        let recovered = import_oci_layout(&ImportRequest {
            layout_dir: &fixture.layout,
            report_path: &fixture.import_report,
            state_dir: &fixture.state,
        })
        .expect("content-addressed bytes should admit idempotently on retry");
        assert!(recovered.imported);
        assert!(fixture.import_report.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_reader_rejects_a_symlink_at_open_time() {
        let temporary = tempfile::tempdir().expect("bounded-reader fixture should exist");
        let target = temporary.path().join("target.json");
        let link = temporary.path().join("link.json");
        fs::write(&target, b"{}").expect("target should write");
        std::os::unix::fs::symlink(&target, &link).expect("symlink should be created");
        let error = super::read_bounded_regular(&link, super::OCI_DOCUMENT_MAX_BYTES)
            .expect_err("no-follow reads must reject a final-component symlink");
        assert!(error.contains("opening non-symlink file"));
        assert!(target.is_file());
    }

    #[test]
    fn layout_reader_rejects_unexpected_root_entries() {
        let fixture = fixture();
        export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect("OCI export should succeed");
        let unexpected = fixture.layout.join("unreferenced.json");
        fs::write(&unexpected, b"{}").expect("unexpected fixture should write");
        let error = super::read_layout_facts(&fixture.layout).expect_err("an unreferenced root entry must fail closed");
        assert!(error.contains("unexpected root entry"));
        assert!(unexpected.is_file());
    }

    #[test]
    fn layout_reader_rejects_layer_count_before_missing_blob_reads() {
        let temporary = tempfile::tempdir().expect("layer-count fixture should exist");
        let layout = temporary.path().join("layout");
        let blob_dir = layout.join(OCI_BLOB_DIR);
        fs::create_dir_all(&blob_dir).expect("blob directory should exist");
        let layout_document = serde_json::json!({"imageLayoutVersion": OCI_LAYOUT_VERSION});
        fs::write(
            layout.join(OCI_LAYOUT_FILENAME),
            serde_json::to_vec(&layout_document).expect("layout document should serialize"),
        )
        .expect("layout document should write");

        let missing_digest = sha256_digest(b"intentionally missing descriptor");
        let layer_descriptor = serde_json::json!({
            "mediaType": "application/vnd.onix.kernel.v1",
            "digest": missing_digest.clone(),
            "size": 0,
        });
        let layer_count_over_bound = OCI_LAYER_MAX_COUNT.checked_add(1).expect("test layer count should fit usize");
        let layers = std::iter::repeat_n(layer_descriptor, layer_count_over_bound).collect::<Vec<_>>();
        let manifest = serde_json::json!({
            "schemaVersion": OCI_SCHEMA_VERSION,
            "mediaType": OCI_MANIFEST_MEDIA_TYPE,
            "config": {
                "mediaType": OCI_CONFIG_MEDIA_TYPE,
                "digest": sha256_digest(b"missing config"),
                "size": 0,
            },
            "layers": layers,
        });
        let manifest_bytes = serde_json::to_vec(&manifest).expect("manifest should serialize");
        let manifest_digest = sha256_digest(&manifest_bytes);
        let manifest_hex = manifest_digest.strip_prefix(super::SHA256_PREFIX).expect("digest should be SHA-256");
        fs::write(blob_dir.join(manifest_hex), &manifest_bytes).expect("manifest blob should write");
        let index = serde_json::json!({
            "schemaVersion": OCI_SCHEMA_VERSION,
            "mediaType": OCI_INDEX_MEDIA_TYPE,
            "manifests": [{
                "mediaType": OCI_MANIFEST_MEDIA_TYPE,
                "digest": manifest_digest,
                "size": u64::try_from(manifest_bytes.len()).expect("manifest length should fit u64"),
            }],
        });
        fs::write(layout.join(OCI_INDEX_FILENAME), serde_json::to_vec(&index).expect("index should serialize"))
            .expect("index should write");

        let error = super::read_layout_facts(&layout)
            .expect_err("the manifest layer bound must fail before missing descriptor reads");
        assert!(error.contains("manifest layer count"));
        assert!(!blob_dir.join(missing_digest.trim_start_matches(super::SHA256_PREFIX)).exists());
    }

    #[test]
    fn source_admission_shape_matches_the_frontend_owner() {
        let fixture = fixture();
        let bytes = fs::read(&fixture.source_admissions).expect("source admissions should be readable");
        let bundle: SourceAdmissionBundle = serde_json::from_slice(&bytes).expect("source admissions should parse");
        let source_value = serde_json::to_value(&bundle.admissions[0]).expect("source admission should serialize");
        let frontend_attestation: FrontendArtifactAdmissionAttestation =
            serde_json::from_value(source_value.clone()).expect("frontend owner should accept source admission shape");
        assert_eq!(
            serde_json::to_value(frontend_attestation).expect("frontend attestation should serialize"),
            source_value
        );
    }

    #[test]
    fn source_admission_mismatch_fails_before_layout_materialization() {
        let fixture = fixture();
        let bytes = fs::read(&fixture.source_admissions).expect("source admissions should be readable");
        let mut bundle: SourceAdmissionBundle = serde_json::from_slice(&bytes).expect("source admissions should parse");
        bundle.admissions[0].validator_ref = "stale-validator".to_string();
        fs::write(
            &fixture.source_admissions,
            serde_json::to_vec_pretty(&bundle).expect("stale admissions should serialize"),
        )
        .expect("stale admissions should write");
        fs::remove_dir_all(&fixture.state).expect("CAS state should be removable before preflight");
        let error = export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect_err("stale source admission must fail");
        assert!(error.contains("source-admission-mismatch"));
        assert!(!fixture.layout.exists());
        assert!(!fixture.state.exists());
    }

    #[cfg(unix)]
    #[test]
    fn frontend_cas_rejects_special_files_before_oci_projection() {
        let temporary = tempfile::tempdir().expect("special-file fixture should exist");
        let source = temporary.path().join("source");
        fs::create_dir(&source).expect("special-file source should exist");
        let socket_path = source.join("control.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("special-file socket should bind");
        let state = temporary.path().join("state");
        let error =
            import_frontend_artifact(&source, &state).expect_err("special files must not enter the frontend CAS");
        assert!(error.starts_with("bounded-tree observing frontend artifact failed with PlanRejected("), "{error}");
        assert!(error.contains("kind: UnsupportedKind"), "{error}");
        assert!(!state.exists(), "rejection must precede store mutation");

        let root_error = import_frontend_artifact(&socket_path, &state).expect_err("a socket root must also fail");
        assert!(root_error.starts_with("unsupported frontend artifact file type: "), "{root_error}");
        assert!(!state.exists(), "root rejection must precede store mutation");
    }

    #[test]
    fn shell_rejects_existing_outputs_and_tampered_blobs() {
        let fixture = fixture();
        fs::create_dir(&fixture.layout).expect("existing output should be created");
        let existing_error = export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect_err("existing output must be rejected");
        assert!(existing_error.contains("already exists"));
        fs::remove_dir(&fixture.layout).expect("existing output should be removable");

        let exported = export_oci_layout(&ExportRequest {
            projection_path: &fixture.projection,
            spec_material_path: &fixture.spec,
            source_admissions_path: &fixture.source_admissions,
            output_dir: &fixture.layout,
            state_dir: &fixture.state,
        })
        .expect("untampered export should succeed");
        let layer_hex =
            exported.layers[0].blob_sha256.strip_prefix("sha256:").expect("layer descriptor should be SHA-256");
        let layer_path = fixture.layout.join(OCI_BLOB_DIR).join(layer_hex);
        let mut bytes = fs::read(&layer_path).expect("layer should be readable");
        bytes.push(0);
        fs::write(&layer_path, bytes).expect("test should tamper layer");
        let error = import_oci_layout(&ImportRequest {
            layout_dir: &fixture.layout,
            report_path: &fixture.import_report,
            state_dir: &fixture.state,
        })
        .expect_err("tampered layer must fail before CAS admission");
        assert!(error.contains("descriptor-mismatch"));
        assert!(!fixture.import_report.exists());
    }
}
