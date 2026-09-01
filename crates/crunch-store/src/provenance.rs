//! Bounded castore-backed provenance observations for realized foreign outputs.
//!
//! The classifier is pure. The asynchronous shell only retrieves signed PathInfo,
//! directories, and blobs from the active store services.
// r[impl foreign_derivation_import.castore_provenance_audit]
// r[impl foreign_derivation_import.executable_payload_classification]
// r[impl foreign_derivation_import.provenance_audit_receipt]
// r[impl foreign_derivation_import.live_guixpkgs_export_realization]
// r[impl store_lifecycle.tiger_conformance]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::io::Read;

use flate2::read::GzDecoder;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use tokio::io::AsyncReadExt;
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::Error;
use crate::StoreHandle;

pub const FOREIGN_PROVENANCE_POLICY_SCHEMA: &str = "mantle-foreign-provenance-policy-v1";
const ELF_MAGIC: &[u8] = b"\x7fELF";
const SCRIPT_MAGIC: &[u8] = b"#!";
const GZIP_MAGIC: &[u8] = b"\x1f\x8b";
const ZIP_MAGIC: &[u8] = b"PK\x03\x04";
const XZ_MAGIC: &[u8] = b"\xfd7zXZ\x00";
const BZIP2_MAGIC: &[u8] = b"BZh";
const ZSTD_MAGIC: &[u8] = b"\x28\xb5\x2f\xfd";
const CPIO_NEWC_MAGIC: &[u8] = b"070701";
const CPIO_CRC_MAGIC: &[u8] = b"070702";
const TAR_USTAR_MAGIC: &[u8] = b"ustar";
const TAR_USTAR_OFFSET: usize = 257;
const TAR_USTAR_END: usize = TAR_USTAR_OFFSET.saturating_add(TAR_USTAR_MAGIC.len());
const ELF_IDENT_BYTES: usize = 16;
const ELF32_HEADER_BYTES: usize = 52;
const ELF64_HEADER_BYTES: usize = 64;
const ELF_CLASS_OFFSET: usize = 4;
const ELF_DATA_OFFSET: usize = 5;
const ELF_VERSION_OFFSET: usize = 6;
const ELF_CLASS_32: u8 = 1;
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_DATA_BIG_ENDIAN: u8 = 2;
const ELF_CURRENT_VERSION: u8 = 1;
const CPIO_HEADER_BYTES: usize = 110;
const CPIO_MODE_START: usize = 14;
const CPIO_MODE_END: usize = 22;
const CPIO_FILE_SIZE_START: usize = 54;
const CPIO_FILE_SIZE_END: usize = 62;
const CPIO_NAME_SIZE_START: usize = 94;
const CPIO_NAME_SIZE_END: usize = 102;
const CPIO_ALIGNMENT_BYTES: usize = 4;
const CPIO_TRAILER_NAME: &str = "TRAILER!!!";
const UNIX_FILE_TYPE_MASK: u32 = 0o170000;
const UNIX_DIRECTORY_FILE_TYPE: u32 = 0o040000;
const UNIX_REGULAR_FILE_TYPE: u32 = 0o100000;
const UNIX_SYMLINK_FILE_TYPE: u32 = 0o120000;
const UNIX_EXECUTABLE_BITS: u32 = 0o111;
#[cfg(test)]
const STORE_PATH_SEPARATOR: u8 = b'/';
const PATH_COMPONENT_SEPARATOR: char = '/';
const CURRENT_PATH_COMPONENT: &str = ".";
const PARENT_PATH_COMPONENT: &str = "..";
const EMPTY_PATH: &str = "";
const HEX_RADIX: u32 = 16;
#[cfg(test)]
const DIGEST_HEX_CHARS: usize = 64;
const INITIAL_WORKLIST_CAPACITY: usize = 128;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignProvenancePolicy {
    pub schema: String,
    pub max_path_infos: u32,
    pub max_nodes: u32,
    pub max_blobs: u32,
    pub max_blob_bytes: u64,
    pub max_total_bytes: u64,
    pub max_depth: u32,
    pub max_findings: u32,
    pub max_duplicates: u32,
    pub max_container_entries: u32,
    pub max_container_expanded_bytes: u64,
    pub max_container_depth: u32,
    pub max_path_bytes: u32,
    pub max_shebang_bytes: u32,
    pub allowed_profile_paths: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenancePayloadClass {
    Data,
    Elf,
    Script,
    Symlink,
    TarArchive,
    CpioInitrd,
    GzipCpioInitrd,
    GzipStream,
    ZstdStream,
    LibtoolArchive,
    Malformed,
    UnsupportedContainer,
    UnsupportedExecutable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceReferenceKind {
    ByteReference,
    ShebangInterpreter,
    SymlinkTarget,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceFinding {
    pub code: String,
    pub path: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceReferenceObservation {
    pub owner_store_path: String,
    pub view: String,
    pub reference: String,
    pub suffix: String,
    pub kind: ProvenanceReferenceKind,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenancePayloadObservation {
    pub owner_store_path: String,
    pub path: String,
    pub node_identity: String,
    pub class: ProvenancePayloadClass,
    pub executable: bool,
    pub byte_count: u64,
    pub content_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenancePathInfoObservation {
    pub logical_path: String,
    pub nar_sha256: String,
    pub nar_size: u64,
    pub node_identity: String,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CastoreProvenanceScan {
    pub preflight_complete: bool,
    pub traversal_complete: bool,
    pub path_infos: Vec<ProvenancePathInfoObservation>,
    pub payloads: Vec<ProvenancePayloadObservation>,
    pub references: Vec<ProvenanceReferenceObservation>,
    pub findings: Vec<ProvenanceFinding>,
    pub visited_node_count: u32,
    pub visited_blob_count: u32,
    pub read_byte_count: u64,
    pub duplicate_node_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceExpectedPathInfo {
    pub nar_sha256: String,
    pub nar_size: u64,
}

pub struct CastoreProvenanceRequest<'a> {
    pub selected_root_paths: &'a [String],
    pub admitted_closure_paths: &'a BTreeSet<String>,
    pub expected_path_infos: &'a BTreeMap<String, ProvenanceExpectedPathInfo>,
    pub declared_references_by_output: &'a BTreeMap<String, BTreeSet<String>>,
    pub foreign_to_target_paths: &'a BTreeMap<String, String>,
    pub trusted_keys: &'a [VerifyingKey],
    pub policy: &'a ForeignProvenancePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurePayloadInput<'a> {
    pub owner_store_path: &'a str,
    pub path: &'a str,
    pub node_identity: &'a str,
    pub executable: bool,
    pub bytes: &'a [u8],
}

struct ObservedReferenceInput<'a> {
    owner_store_path: &'a str,
    view: &'a str,
    reference: &'a str,
    suffix: &'a str,
    kind: ProvenanceReferenceKind,
}

struct SymlinkInput<'a> {
    owner_store_path: &'a str,
    path: &'a str,
    target: &'a str,
    link_root: &'a str,
}

struct RelativePathInput<'a> {
    path: &'a str,
    target: &'a str,
    root: &'a str,
}

struct CompressedStreamInput<'a, R> {
    payload: &'a PurePayloadInput<'a>,
    decoder: R,
    extension: &'a str,
    format_name: &'a str,
    container_depth: u32,
}

struct CpioInput<'a> {
    payload: &'a PurePayloadInput<'a>,
    bytes: &'a [u8],
    container_depth: u32,
}

struct ContainerInspection<'a> {
    payload: &'a PurePayloadInput<'a>,
    entries: Vec<ContainerEntry>,
    container_depth: u32,
}

struct ContainerSymlinkInput<'a> {
    payload: &'a PurePayloadInput<'a>,
    path: &'a str,
    target: &'a str,
}

struct ScanCompletion {
    is_preflight_complete: bool,
    is_traversal_complete: bool,
}

struct StoreReferenceInput<'a> {
    raw: &'a str,
    prefix: &'a str,
}

struct AlignmentInput {
    value_bytes: usize,
    alignment_bytes: usize,
}

struct ResolutionContext<'a> {
    target_store_prefix: &'a str,
    admitted_closure_paths: &'a BTreeSet<String>,
    declared_references_by_output: &'a BTreeMap<String, BTreeSet<String>>,
    foreign_to_target_paths: &'a BTreeMap<String, String>,
    foreign_store_prefixes: Vec<String>,
    allowed_profile_paths: BTreeSet<String>,
}

struct ScanAccumulator {
    payloads: Vec<ProvenancePayloadObservation>,
    references: Vec<ProvenanceReferenceObservation>,
    findings: Vec<ProvenanceFinding>,
    observed_paths: BTreeSet<String>,
    symlink_edges: BTreeMap<String, String>,
    halted: bool,
    visited_node_count: u32,
    visited_blob_count: u32,
    read_byte_count: u64,
    duplicate_node_count: u32,
    container_entry_count: u32,
    container_expanded_bytes: u64,
}

impl ScanAccumulator {
    fn new() -> Self {
        Self {
            payloads: Vec::new(),
            references: Vec::new(),
            findings: Vec::new(),
            observed_paths: BTreeSet::new(),
            symlink_edges: BTreeMap::new(),
            halted: false,
            visited_node_count: 0,
            visited_blob_count: 0,
            read_byte_count: 0,
            duplicate_node_count: 0,
            container_entry_count: 0,
            container_expanded_bytes: 0,
        }
    }

    fn finding(&mut self, policy: &ForeignProvenancePolicy, code: &str, path: &str, detail: impl Into<String>) {
        if self.halted {
            return;
        }
        let Ok(finding_index) = u32::try_from(self.findings.len()) else {
            self.halted = true;
            return;
        };
        if finding_index >= policy.max_findings {
            self.halted = true;
            if policy.max_findings > 0 {
                self.findings.pop();
                self.findings.push(ProvenanceFinding {
                    code: "limit-exhausted".to_string(),
                    path: path.to_string(),
                    detail: "findings".to_string(),
                });
            }
            return;
        }
        self.findings.push(ProvenanceFinding {
            code: code.to_string(),
            path: path.to_string(),
            detail: detail.into(),
        });
    }

    fn limit(&mut self, policy: &ForeignProvenancePolicy, path: &str, limit: &str) {
        self.finding(policy, "limit-exhausted", path, limit);
        self.halted = true;
    }
}

struct LoadedPathInfo {
    logical_path: String,
    path_info: snix_store::path_info::PathInfo,
}

struct VerifiedPreflightPathInfo {
    logical_path: String,
    path_info: snix_store::path_info::PathInfo,
    observed_nar_sha256: String,
}

struct WorkItem {
    owner_store_path: String,
    logical_path: String,
    node: Node,
    depth: u32,
}

struct DirectoryWorkInput {
    owner_store_path: String,
    logical_path: String,
    digest: snix_castore::B3Digest,
    size_bytes: u64,
    depth: u32,
}

struct FileWorkInput {
    owner_store_path: String,
    logical_path: String,
    node_identity: String,
    digest: snix_castore::B3Digest,
    size_bytes: u64,
    is_executable: bool,
}

struct SymlinkWorkInput<'a> {
    owner_store_path: String,
    logical_path: String,
    node_identity: String,
    target: &'a str,
}

struct EncodedSymlinkWorkInput {
    owner_store_path: String,
    logical_path: String,
    node_identity: String,
    target: snix_castore::SymlinkTarget,
}

struct PayloadWalkContext<'services, 'policy, 'context, 'facts> {
    directory_service: &'services dyn snix_castore::directoryservice::DirectoryService,
    blob_service: &'services dyn snix_castore::blobservice::BlobService,
    policy: &'policy ForeignProvenancePolicy,
    resolution: &'context ResolutionContext<'facts>,
}

#[derive(Clone, Debug)]
struct ContainerEntry {
    path: String,
    executable: bool,
    bytes: Vec<u8>,
    symlink_target: Option<String>,
}

pub fn validate_foreign_provenance_policy(policy: &ForeignProvenancePolicy) -> Result<(), String> {
    if policy.schema != FOREIGN_PROVENANCE_POLICY_SCHEMA {
        return Err(format!("foreign provenance policy schema must be {FOREIGN_PROVENANCE_POLICY_SCHEMA}"));
    }
    let bounded_fields = [
        ("max_path_infos", u64::from(policy.max_path_infos)),
        ("max_nodes", u64::from(policy.max_nodes)),
        ("max_blobs", u64::from(policy.max_blobs)),
        ("max_blob_bytes", policy.max_blob_bytes),
        ("max_total_bytes", policy.max_total_bytes),
        ("max_depth", u64::from(policy.max_depth)),
        ("max_findings", u64::from(policy.max_findings)),
        ("max_duplicates", u64::from(policy.max_duplicates)),
        ("max_container_entries", u64::from(policy.max_container_entries)),
        ("max_container_expanded_bytes", policy.max_container_expanded_bytes),
        ("max_container_depth", u64::from(policy.max_container_depth)),
        ("max_path_bytes", u64::from(policy.max_path_bytes)),
        ("max_shebang_bytes", u64::from(policy.max_shebang_bytes)),
    ];
    for (name, value) in bounded_fields {
        if value == 0 {
            return Err(format!("foreign provenance policy limit must be nonzero: {name}"));
        }
    }
    if policy.max_blob_bytes > policy.max_total_bytes {
        return Err("max_blob_bytes must not exceed max_total_bytes".to_string());
    }
    if policy.max_container_expanded_bytes > policy.max_total_bytes {
        return Err("max_container_expanded_bytes must not exceed max_total_bytes".to_string());
    }
    let mut previous = None;
    for path in &policy.allowed_profile_paths {
        if !path.starts_with(PATH_COMPONENT_SEPARATOR) || contains_parent_component(path) {
            return Err(format!("allowed profile path must be absolute and lexical: {path}"));
        }
        if previous.is_some_and(|value| value >= path) {
            return Err("allowed profile paths must be sorted and unique".to_string());
        }
        previous = Some(path);
    }
    assert_eq!(policy.schema, FOREIGN_PROVENANCE_POLICY_SCHEMA);
    assert!(policy.max_blob_bytes <= policy.max_total_bytes);
    Ok(())
}

pub async fn scan_castore_provenance(
    store: &StoreHandle,
    request: CastoreProvenanceRequest<'_>,
) -> Result<CastoreProvenanceScan, Error> {
    validate_foreign_provenance_policy(request.policy).map_err(Error::Store)?;
    if request.selected_root_paths.is_empty() {
        return Err(Error::Store("foreign provenance audit requires at least one selected root".to_string()));
    }
    if request.trusted_keys.is_empty() {
        return Err(Error::Store("foreign provenance audit requires at least one trusted key".to_string()));
    }
    assert!(!request.selected_root_paths.is_empty());
    assert!(!request.trusted_keys.is_empty());
    let target_store_prefix = store.store_dir();
    let context = ResolutionContext {
        target_store_prefix,
        admitted_closure_paths: request.admitted_closure_paths,
        declared_references_by_output: request.declared_references_by_output,
        foreign_to_target_paths: request.foreign_to_target_paths,
        foreign_store_prefixes: foreign_store_prefixes(request.foreign_to_target_paths),
        allowed_profile_paths: request.policy.allowed_profile_paths.iter().cloned().collect(),
    };
    let (loaded_path_infos, path_infos, mut preflight) = preflight_path_infos(store, &request, &context).await?;
    if !preflight.findings.is_empty() || preflight.halted {
        return Ok(finalize_scan(
            ScanCompletion {
                is_preflight_complete: false,
                is_traversal_complete: false,
            },
            path_infos,
            preflight,
        ));
    }
    walk_castore_payloads(store, &loaded_path_infos, request.policy, &context, &mut preflight).await?;
    finalize_symlink_graph(request.policy, &mut preflight);
    let is_traversal_complete =
        !preflight.halted && !preflight.findings.iter().any(|finding| finding.code == "incomplete-closure");
    Ok(finalize_scan(
        ScanCompletion {
            is_preflight_complete: true,
            is_traversal_complete,
        },
        path_infos,
        preflight,
    ))
}

struct PreflightTraversal {
    pending: BTreeSet<String>,
    seen: BTreeSet<String>,
    loaded_path_infos: Vec<LoadedPathInfo>,
    observations: Vec<ProvenancePathInfoObservation>,
    state: ScanAccumulator,
}

impl PreflightTraversal {
    fn new(request: &CastoreProvenanceRequest<'_>) -> Result<Self, Error> {
        Ok(Self {
            pending: request.selected_root_paths.iter().cloned().collect(),
            seen: BTreeSet::new(),
            loaded_path_infos: Vec::with_capacity(
                usize::try_from(request.policy.max_path_infos)
                    .map_err(|_| Error::Store("foreign provenance path-info limit exceeds usize".to_string()))?,
            ),
            observations: Vec::with_capacity(
                usize::try_from(request.policy.max_path_infos)
                    .map_err(|_| Error::Store("foreign provenance path-info limit exceeds usize".to_string()))?,
            ),
            state: ScanAccumulator::new(),
        })
    }

    fn admit_logical_path(&mut self, logical_path: &str, request: &CastoreProvenanceRequest<'_>) -> bool {
        if !self.seen.insert(logical_path.to_string()) {
            return false;
        }
        let should_stop = match u32::try_from(self.seen.len()) {
            Ok(value) => value > request.policy.max_path_infos,
            Err(_) => true,
        };
        if should_stop {
            self.state.limit(request.policy, logical_path, "path-infos");
            return false;
        }
        if !request.admitted_closure_paths.contains(logical_path) {
            self.state.finding(
                request.policy,
                "unadmitted-closure-path",
                logical_path,
                "path is outside the admitted plan closure",
            );
            return false;
        }
        assert!(self.seen.contains(logical_path));
        assert!(request.admitted_closure_paths.contains(logical_path));
        true
    }

    fn finish(mut self) -> (Vec<LoadedPathInfo>, Vec<ProvenancePathInfoObservation>, ScanAccumulator) {
        self.loaded_path_infos.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
        self.observations.sort();
        (self.loaded_path_infos, self.observations, self.state)
    }
}

async fn load_preflight_path_info(
    service: &dyn snix_store::pathinfoservice::PathInfoService,
    logical_path: &str,
    context: &ResolutionContext<'_>,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
) -> Result<Option<snix_store::path_info::PathInfo>, Error> {
    let parsed: StorePath<String> =
        match StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), context.target_store_prefix) {
            Ok(path) => path,
            Err(error) => {
                state.finding(policy, "invalid-store-path", logical_path, error.to_string());
                return Ok(None);
            }
        };
    assert!(!logical_path.is_empty());
    assert!(!context.target_store_prefix.is_empty());
    let path_info = service
        .get(*parsed.digest())
        .await
        .map_err(|error| Error::PathInfoService(format!("foreign provenance preflight for {logical_path}: {error}")))?;
    let Some(path_info) = path_info else {
        state.finding(policy, "incomplete-closure", logical_path, "PathInfo is missing");
        return Ok(None);
    };
    Ok(Some(path_info))
}

fn verify_preflight_path_info(
    path_info: &snix_store::path_info::PathInfo,
    logical_path: &str,
    request: &CastoreProvenanceRequest<'_>,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) -> Option<String> {
    let observed_path = path_info.store_path.to_absolute_path_with_prefix(context.target_store_prefix);
    if observed_path != logical_path {
        state.finding(
            request.policy,
            "inconsistent-pathinfo",
            logical_path,
            format!("stored path was {observed_path}"),
        );
        return None;
    }
    if !path_info_has_trusted_signature(path_info, request.trusted_keys, context.target_store_prefix) {
        state.finding(
            request.policy,
            "unsigned-or-untrusted-pathinfo",
            logical_path,
            "no trusted PathInfo signature verified",
        );
        return None;
    }
    let observed_nar_sha256 = data_encoding::HEXLOWER.encode(&path_info.nar_sha256);
    let Some(expected) = request.expected_path_infos.get(logical_path) else {
        state.finding(
            request.policy,
            "unbound-pathinfo",
            logical_path,
            "realization receipt has no NAR fact for this path",
        );
        return None;
    };
    if expected.nar_sha256 != observed_nar_sha256 || expected.nar_size != path_info.nar_size {
        state.finding(
            request.policy,
            "inconsistent-pathinfo",
            logical_path,
            format!(
                "receipt NAR was {}:{} but store NAR was {}:{}",
                expected.nar_sha256, expected.nar_size, observed_nar_sha256, path_info.nar_size
            ),
        );
        return None;
    }
    assert_eq!(observed_path, logical_path);
    assert_eq!(expected.nar_size, path_info.nar_size);
    Some(observed_nar_sha256)
}

fn record_preflight_path_info(
    verified: VerifiedPreflightPathInfo,
    request: &CastoreProvenanceRequest<'_>,
    context: &ResolutionContext<'_>,
    traversal: &mut PreflightTraversal,
) {
    let VerifiedPreflightPathInfo {
        logical_path,
        path_info,
        observed_nar_sha256,
    } = verified;
    let mut references = Vec::with_capacity(path_info.references.len());
    for reference in &path_info.references {
        let target = reference.to_absolute_path_with_prefix(context.target_store_prefix);
        references.push(target.clone());
        if request.admitted_closure_paths.contains(&target) {
            traversal.pending.insert(target);
        } else {
            traversal.state.finding(
                request.policy,
                "unadmitted-closure-reference",
                &logical_path,
                format!("PathInfo references {target}"),
            );
        }
    }
    references.sort();
    references.dedup();
    traversal.observations.push(ProvenancePathInfoObservation {
        logical_path: logical_path.clone(),
        nar_sha256: observed_nar_sha256,
        nar_size: path_info.nar_size,
        node_identity: node_identity(&path_info.node),
        references,
    });
    traversal.loaded_path_infos.push(LoadedPathInfo {
        logical_path,
        path_info,
    });
    assert_eq!(traversal.loaded_path_infos.len(), traversal.observations.len());
    assert!(u32::try_from(traversal.loaded_path_infos.len()).is_ok_and(|value| value <= request.policy.max_path_infos));
}

async fn preflight_path_infos(
    store: &StoreHandle,
    request: &CastoreProvenanceRequest<'_>,
    context: &ResolutionContext<'_>,
) -> Result<(Vec<LoadedPathInfo>, Vec<ProvenancePathInfoObservation>, ScanAccumulator), Error> {
    assert!(!request.selected_root_paths.is_empty());
    assert!(!context.target_store_prefix.is_empty());
    let service = store.pathinfo_service();
    let mut traversal = PreflightTraversal::new(request)?;
    while let Some(logical_path) = traversal.pending.pop_first() {
        if traversal.state.halted {
            break;
        }
        if !traversal.admit_logical_path(&logical_path, request) {
            continue;
        }
        let Some(path_info) =
            load_preflight_path_info(service.as_ref(), &logical_path, context, request.policy, &mut traversal.state)
                .await?
        else {
            continue;
        };
        let Some(observed_nar_sha256) =
            verify_preflight_path_info(&path_info, &logical_path, request, context, &mut traversal.state)
        else {
            continue;
        };
        record_preflight_path_info(
            VerifiedPreflightPathInfo {
                logical_path,
                path_info,
                observed_nar_sha256,
            },
            request,
            context,
            &mut traversal,
        );
    }
    Ok(traversal.finish())
}

async fn process_directory_node(
    directory_service: &dyn snix_castore::directoryservice::DirectoryService,
    input: DirectoryWorkInput,
    policy: &ForeignProvenancePolicy,
    worklist: &mut Vec<WorkItem>,
    state: &mut ScanAccumulator,
) -> Result<(), Error> {
    assert!(!input.logical_path.is_empty());
    assert!(input.depth <= policy.max_depth);
    let directory = directory_service
        .get(&input.digest)
        .await
        .map_err(|error| Error::DirectoryService(format!("foreign provenance directory {}: {error}", input.digest)))?;
    let Some(directory) = directory else {
        state.finding(
            policy,
            "incomplete-closure",
            &input.logical_path,
            format!("directory {} is missing", input.digest),
        );
        return Ok(());
    };
    if directory.digest() != input.digest || directory.size() != input.size_bytes {
        state.finding(
            policy,
            "incomplete-closure",
            &input.logical_path,
            format!("directory {} content or size is inconsistent", input.digest),
        );
        return Ok(());
    }
    let mut children = Vec::with_capacity(directory.nodes().count());
    for (name, child) in directory.nodes() {
        let name = match std::str::from_utf8(name.as_ref()) {
            Ok(name) => name,
            Err(error) => {
                state.finding(
                    policy,
                    "malformed-path",
                    &input.logical_path,
                    format!("non-UTF-8 directory name: {error}"),
                );
                continue;
            }
        };
        let child_path = format!("{}/{name}", input.logical_path);
        let is_path_too_long = u32::try_from(child_path.len()).map_or(true, |value| value > policy.max_path_bytes);
        if is_path_too_long {
            state.limit(policy, &child_path, "path-bytes");
            break;
        }
        children.push(WorkItem {
            owner_store_path: input.owner_store_path.clone(),
            logical_path: child_path,
            node: child.clone(),
            depth: input.depth.saturating_add(1),
        });
    }
    children.sort_by(|left, right| right.logical_path.cmp(&left.logical_path));
    worklist
        .try_reserve(children.len())
        .map_err(|error| Error::Store(format!("reserving provenance worklist: {error}")))?;
    worklist.extend(children);
    Ok(())
}

async fn process_file_node(
    blob_service: &dyn snix_castore::blobservice::BlobService,
    input: FileWorkInput,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) -> Result<(), Error> {
    if state.visited_blob_count >= policy.max_blobs {
        state.limit(policy, &input.logical_path, "blobs");
        return Ok(());
    }
    if input.size_bytes > policy.max_blob_bytes {
        state.limit(policy, &input.logical_path, "blob-bytes");
        return Ok(());
    }
    let Some(next_total_bytes) = state.read_byte_count.checked_add(input.size_bytes) else {
        state.limit(policy, &input.logical_path, "total-bytes");
        return Ok(());
    };
    if next_total_bytes > policy.max_total_bytes {
        state.limit(policy, &input.logical_path, "total-bytes");
        return Ok(());
    }
    assert!(input.size_bytes <= policy.max_blob_bytes);
    assert!(next_total_bytes <= policy.max_total_bytes);
    state.visited_blob_count = state.visited_blob_count.saturating_add(1);
    let bytes = match read_blob_exact(blob_service, &input.digest, input.size_bytes).await {
        Ok(bytes) => bytes,
        Err(detail) => {
            state.finding(policy, "incomplete-closure", &input.logical_path, detail);
            return Ok(());
        }
    };
    state.read_byte_count = next_total_bytes;
    inspect_payload(
        PurePayloadInput {
            owner_store_path: &input.owner_store_path,
            path: &input.logical_path,
            node_identity: &input.node_identity,
            executable: input.is_executable,
            bytes: &bytes,
        },
        0,
        policy,
        context,
        state,
    );
    Ok(())
}

fn process_symlink_node(
    input: SymlinkWorkInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.owner_store_path.is_empty());
    assert!(!input.logical_path.is_empty());
    let target_size_bytes = match u64::try_from(input.target.len()) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, &input.logical_path, "symlink-bytes");
            return;
        }
    };
    state.payloads.push(ProvenancePayloadObservation {
        owner_store_path: input.owner_store_path.clone(),
        path: input.logical_path.clone(),
        node_identity: input.node_identity,
        class: ProvenancePayloadClass::Symlink,
        executable: false,
        byte_count: target_size_bytes,
        content_blake3: blake3::hash(input.target.as_bytes()).to_hex().to_string(),
    });
    inspect_symlink(
        SymlinkInput {
            owner_store_path: &input.owner_store_path,
            path: &input.logical_path,
            target: input.target,
            link_root: &input.owner_store_path,
        },
        policy,
        context,
        state,
    );
}

fn process_encoded_symlink(
    input: EncodedSymlinkWorkInput,
    walk: &PayloadWalkContext<'_, '_, '_, '_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.owner_store_path.is_empty());
    assert!(!input.logical_path.is_empty());
    let target = match std::str::from_utf8(input.target.as_ref()) {
        Ok(target) => target,
        Err(error) => {
            state.finding(walk.policy, "malformed-symlink", &input.logical_path, format!("non-UTF-8 target: {error}"));
            return;
        }
    };
    process_symlink_node(
        SymlinkWorkInput {
            owner_store_path: input.owner_store_path,
            logical_path: input.logical_path,
            node_identity: input.node_identity,
            target,
        },
        walk.policy,
        walk.resolution,
        state,
    );
}

async fn process_work_item(
    walk: &PayloadWalkContext<'_, '_, '_, '_>,
    item: WorkItem,
    worklist: &mut Vec<WorkItem>,
    state: &mut ScanAccumulator,
) -> Result<(), Error> {
    assert!(item.depth <= walk.policy.max_depth);
    assert!(!state.halted);
    let identity = node_identity(&item.node);
    match item.node {
        Node::Directory { digest, size } => {
            process_directory_node(
                walk.directory_service,
                DirectoryWorkInput {
                    owner_store_path: item.owner_store_path,
                    logical_path: item.logical_path,
                    digest,
                    size_bytes: size,
                    depth: item.depth,
                },
                walk.policy,
                worklist,
                state,
            )
            .await?;
        }
        Node::File {
            digest,
            size,
            executable,
        } => {
            process_file_node(
                walk.blob_service,
                FileWorkInput {
                    owner_store_path: item.owner_store_path,
                    logical_path: item.logical_path,
                    node_identity: identity,
                    digest,
                    size_bytes: size,
                    is_executable: executable,
                },
                walk.policy,
                walk.resolution,
                state,
            )
            .await?;
        }
        Node::Symlink { target } => process_encoded_symlink(
            EncodedSymlinkWorkInput {
                owner_store_path: item.owner_store_path,
                logical_path: item.logical_path,
                node_identity: identity,
                target,
            },
            walk,
            state,
        ),
    }
    Ok(())
}

async fn walk_castore_payloads(
    store: &StoreHandle,
    path_infos: &[LoadedPathInfo],
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) -> Result<(), Error> {
    assert!(u32::try_from(path_infos.len()).is_ok_and(|value| value <= policy.max_path_infos));
    assert!(!context.target_store_prefix.is_empty());
    let directory_service = store.directory_service();
    let blob_service = store.blob_service();
    let walk = PayloadWalkContext {
        directory_service: directory_service.as_ref(),
        blob_service: blob_service.as_ref(),
        policy,
        resolution: context,
    };
    let mut worklist = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY.max(path_infos.len()));
    for loaded in path_infos.iter().rev() {
        worklist.push(WorkItem {
            owner_store_path: loaded.logical_path.clone(),
            logical_path: loaded.logical_path.clone(),
            node: loaded.path_info.node.clone(),
            depth: 0,
        });
    }
    let mut node_identities = BTreeSet::new();

    while let Some(item) = worklist.pop() {
        if state.halted {
            break;
        }
        if item.depth > policy.max_depth {
            state.limit(policy, &item.logical_path, "depth");
            break;
        }
        if state.visited_node_count >= policy.max_nodes {
            state.limit(policy, &item.logical_path, "nodes");
            break;
        }
        state.visited_node_count = state.visited_node_count.saturating_add(1);
        state.observed_paths.insert(item.logical_path.clone());
        let identity = node_identity(&item.node);
        if !node_identities.insert(identity) {
            state.duplicate_node_count = state.duplicate_node_count.saturating_add(1);
            if state.duplicate_node_count > policy.max_duplicates {
                state.limit(policy, &item.logical_path, "duplicates");
                break;
            }
        }
        process_work_item(&walk, item, &mut worklist, state).await?;
    }
    Ok(())
}

async fn read_blob_exact(
    blob_service: &dyn snix_castore::blobservice::BlobService,
    digest: &snix_castore::B3Digest,
    expected_size_bytes: u64,
) -> Result<Vec<u8>, String> {
    let Some(reader) = blob_service
        .open_read(digest)
        .await
        .map_err(|error| format!("foreign provenance blob {digest}: {error}"))?
    else {
        return Err(format!("foreign provenance blob is missing: {digest}"));
    };
    let mut bytes = Vec::with_capacity(
        usize::try_from(expected_size_bytes).map_err(|_| "foreign provenance blob size exceeds usize".to_string())?,
    );
    reader
        .take(expected_size_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .await
        .map_err(|error| format!("reading foreign provenance blob {digest}: {error}"))?;
    if u64::try_from(bytes.len()).ok() != Some(expected_size_bytes)
        || blake3::hash(&bytes).as_bytes() != digest.as_slice()
    {
        return Err(format!("foreign provenance blob is corrupt: {digest}"));
    }
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(expected_size_bytes));
    assert_eq!(blake3::hash(&bytes).as_bytes(), digest.as_slice());
    Ok(bytes)
}

pub fn classify_payload(executable: bool, path: &str, bytes: &[u8]) -> ProvenancePayloadClass {
    assert!(!ELF_MAGIC.is_empty());
    assert!(!SCRIPT_MAGIC.is_empty());
    if bytes.starts_with(GZIP_MAGIC) {
        return ProvenancePayloadClass::GzipStream;
    }
    if bytes.starts_with(ZSTD_MAGIC) {
        return ProvenancePayloadClass::ZstdStream;
    }
    if bytes.starts_with(ZIP_MAGIC) || bytes.starts_with(XZ_MAGIC) || bytes.starts_with(BZIP2_MAGIC) {
        return ProvenancePayloadClass::UnsupportedContainer;
    }
    if bytes.starts_with(CPIO_NEWC_MAGIC) || bytes.starts_with(CPIO_CRC_MAGIC) {
        return ProvenancePayloadClass::CpioInitrd;
    }
    if looks_like_tar(bytes) || path.ends_with(".tar") {
        return ProvenancePayloadClass::TarArchive;
    }
    if executable && bytes.starts_with(ELF_MAGIC) {
        if valid_elf_header(bytes) {
            return ProvenancePayloadClass::Elf;
        }
        return ProvenancePayloadClass::Malformed;
    }
    if executable && bytes.starts_with(SCRIPT_MAGIC) {
        return ProvenancePayloadClass::Script;
    }
    if executable && is_libtool_archive(path, bytes) {
        return ProvenancePayloadClass::LibtoolArchive;
    }
    if executable {
        ProvenancePayloadClass::UnsupportedExecutable
    } else {
        ProvenancePayloadClass::Data
    }
}

fn inspect_payload(
    input: PurePayloadInput<'_>,
    container_depth: u32,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    if state.halted {
        return;
    }
    assert!(!state.halted);
    assert!(policy.max_findings > 0);
    let class = classify_payload(input.executable, input.path, input.bytes);
    let payload_size_bytes = match u64::try_from(input.bytes.len()) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, input.path, "payload-bytes");
            return;
        }
    };
    state.observed_paths.insert(input.path.to_string());
    state.payloads.push(ProvenancePayloadObservation {
        owner_store_path: input.owner_store_path.to_string(),
        path: input.path.to_string(),
        node_identity: input.node_identity.to_string(),
        class,
        executable: input.executable,
        byte_count: payload_size_bytes,
        content_blake3: blake3::hash(input.bytes).to_hex().to_string(),
    });
    scan_byte_references(&input, policy, context, state);
    match class {
        ProvenancePayloadClass::Elf | ProvenancePayloadClass::LibtoolArchive => {}
        ProvenancePayloadClass::Script => inspect_shebang(&input, policy, context, state),
        ProvenancePayloadClass::TarArchive => inspect_tar(&input, container_depth, policy, context, state),
        ProvenancePayloadClass::CpioInitrd => inspect_cpio(
            CpioInput {
                payload: &input,
                bytes: input.bytes,
                container_depth,
            },
            policy,
            context,
            state,
        ),
        ProvenancePayloadClass::GzipCpioInitrd | ProvenancePayloadClass::GzipStream => {
            inspect_gzip_stream(&input, container_depth, policy, context, state)
        }
        ProvenancePayloadClass::ZstdStream => inspect_zstd_stream(&input, container_depth, policy, context, state),
        ProvenancePayloadClass::Malformed => {
            state.finding(policy, "malformed-elf", input.path, "ELF header is incomplete or invalid")
        }
        ProvenancePayloadClass::UnsupportedContainer => state.finding(
            policy,
            "unsupported-container",
            input.path,
            "container format is not in the accepted bounded set",
        ),
        ProvenancePayloadClass::UnsupportedExecutable => state.finding(
            policy,
            "unclassified-executable",
            input.path,
            "executable bytes match no accepted payload class",
        ),
        ProvenancePayloadClass::Data | ProvenancePayloadClass::Symlink => {}
    }
}

fn is_libtool_archive(path: &str, bytes: &[u8]) -> bool {
    if !path.ends_with(".la") {
        return false;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut lines = text.lines();
    let Some(first_line) = lines.next() else {
        return false;
    };
    let expected_suffix = ".la - a libtool library file";
    if !first_line.starts_with("# ") || !first_line.ends_with(expected_suffix) {
        return false;
    }
    let is_generated_by_libtool = text.lines().any(|line| line.starts_with("# Generated by libtool "));
    let has_dynamic_name = text.lines().any(|line| line.starts_with("dlname='"));
    let has_library_names = text.lines().any(|line| line.starts_with("library_names='"));
    let has_installation_state = text.lines().any(|line| line.starts_with("installed="));
    assert!(path.ends_with(".la"));
    assert!(first_line.ends_with(expected_suffix));
    is_generated_by_libtool && has_dynamic_name && has_library_names && has_installation_state
}

fn valid_elf_header(bytes: &[u8]) -> bool {
    if bytes.len() < ELF_IDENT_BYTES || bytes.get(ELF_VERSION_OFFSET).copied() != Some(ELF_CURRENT_VERSION) {
        return false;
    }
    if !matches!(bytes.get(ELF_DATA_OFFSET).copied(), Some(ELF_DATA_LITTLE_ENDIAN | ELF_DATA_BIG_ENDIAN)) {
        return false;
    }
    match bytes.get(ELF_CLASS_OFFSET).copied() {
        Some(ELF_CLASS_32) => bytes.len() >= ELF32_HEADER_BYTES,
        Some(ELF_CLASS_64) => bytes.len() >= ELF64_HEADER_BYTES,
        Some(_) | None => false,
    }
}

fn inspect_shebang(
    input: &PurePayloadInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    let maximum_shebang_bytes = match usize::try_from(policy.max_shebang_bytes) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, input.path, "shebang-bytes");
            return;
        }
    };
    let line_end = input.bytes.iter().position(|byte| *byte == b'\n').unwrap_or(input.bytes.len());
    if line_end > maximum_shebang_bytes {
        state.limit(policy, input.path, "shebang-bytes");
        return;
    }
    assert!(line_end <= maximum_shebang_bytes);
    assert!(input.bytes.starts_with(SCRIPT_MAGIC));
    let line = match std::str::from_utf8(&input.bytes[SCRIPT_MAGIC.len()..line_end]) {
        Ok(line) => line.trim(),
        Err(error) => {
            state.finding(policy, "malformed-shebang", input.path, format!("non-UTF-8 shebang: {error}"));
            return;
        }
    };
    let Some(interpreter) = line.split_ascii_whitespace().next() else {
        state.finding(policy, "malformed-shebang", input.path, "shebang has no interpreter");
        return;
    };
    if !interpreter.starts_with(PATH_COMPONENT_SEPARATOR) {
        state.finding(policy, "relative-shebang", input.path, interpreter);
        return;
    }
    if context.allowed_profile_paths.contains(interpreter) {
        state.references.push(ProvenanceReferenceObservation {
            owner_store_path: input.owner_store_path.to_string(),
            view: format!("{}:shebang", input.path),
            reference: interpreter.to_string(),
            suffix: String::new(),
            kind: ProvenanceReferenceKind::ShebangInterpreter,
        });
        return;
    }
    let prefix = std::iter::once(context.target_store_prefix)
        .chain(context.foreign_store_prefixes.iter().map(String::as_str))
        .find(|prefix| interpreter.starts_with(&format!("{prefix}/")));
    let (reference, suffix) = prefix.map_or((interpreter, EMPTY_PATH), |prefix| {
        split_store_reference(StoreReferenceInput {
            raw: interpreter,
            prefix,
        })
    });
    resolve_observed_reference(
        ObservedReferenceInput {
            owner_store_path: input.owner_store_path,
            view: input.path,
            reference,
            suffix,
            kind: ProvenanceReferenceKind::ShebangInterpreter,
        },
        policy,
        context,
        state,
    );
}

fn scan_byte_references(
    input: &PurePayloadInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!context.target_store_prefix.is_empty());
    assert!(policy.max_path_bytes > 0);
    let mut prefixes = context.foreign_store_prefixes.clone();
    prefixes.push(context.target_store_prefix.to_string());
    prefixes.sort();
    prefixes.dedup();
    for prefix in prefixes {
        let needle = format!("{prefix}/");
        let mut cursor = 0usize;
        while cursor < input.bytes.len() {
            let Some(relative) = find_bytes(&input.bytes[cursor..], needle.as_bytes()) else {
                break;
            };
            let start = cursor.saturating_add(relative);
            let end = reference_end(input.bytes, start);
            let raw = &input.bytes[start..end];
            if let Ok(raw) = std::str::from_utf8(raw) {
                let (reference, suffix) = split_store_reference(StoreReferenceInput { raw, prefix: &prefix });
                if !is_well_formed_store_reference(StoreReferenceInput {
                    raw: reference,
                    prefix: &prefix,
                }) {
                    cursor = end.max(start.saturating_add(1));
                    continue;
                }
                resolve_observed_reference(
                    ObservedReferenceInput {
                        owner_store_path: input.owner_store_path,
                        view: &format!("{}#byte-{start}", input.path),
                        reference,
                        suffix,
                        kind: ProvenanceReferenceKind::ByteReference,
                    },
                    policy,
                    context,
                    state,
                );
            }
            cursor = end.max(start.saturating_add(1));
        }
    }
}

fn resolve_observed_reference(
    input: ObservedReferenceInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.owner_store_path.is_empty());
    assert!(!input.view.is_empty());
    let normalized_suffix = match normalize_store_suffix(input.suffix) {
        Ok(normalized) => normalized,
        Err(()) => {
            state.finding(policy, "reference-path-escape", input.view, format!("{}{}", input.reference, input.suffix));
            return;
        }
    };
    if is_untranslated_foreign_reference(
        input.reference,
        context.foreign_to_target_paths,
        &context.foreign_store_prefixes,
    ) {
        state.finding(
            policy,
            "untranslated-foreign-path",
            input.view,
            format!("{}{normalized_suffix}", input.reference),
        );
        return;
    }
    if !input.reference.starts_with(&format!("{}/", context.target_store_prefix)) {
        state.finding(
            policy,
            "missing-executable-target",
            input.view,
            format!("{}{normalized_suffix}", input.reference),
        );
        return;
    }
    if !context.admitted_closure_paths.contains(input.reference) {
        state.finding(policy, "missing-target", input.view, format!("{}{normalized_suffix}", input.reference));
        return;
    }
    let declared = context.declared_references_by_output.get(input.owner_store_path);
    let is_self = input.reference == input.owner_store_path;
    if !is_self && !declared.is_some_and(|values| values.contains(input.reference)) {
        state.finding(
            policy,
            "undeclared-target-reference",
            input.view,
            format!("{}{normalized_suffix}", input.reference),
        );
        return;
    }
    state.references.push(ProvenanceReferenceObservation {
        owner_store_path: input.owner_store_path.to_string(),
        view: input.view.to_string(),
        reference: input.reference.to_string(),
        suffix: normalized_suffix,
        kind: input.kind,
    });
}

fn inspect_symlink(
    input: SymlinkInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.owner_store_path.is_empty());
    assert!(!input.path.is_empty());
    if input.target.starts_with(PATH_COMPONENT_SEPARATOR) {
        let prefix = if input.target.starts_with(&format!("{}/", context.target_store_prefix)) {
            Some(context.target_store_prefix)
        } else {
            context
                .foreign_store_prefixes
                .iter()
                .find(|prefix| input.target.starts_with(&format!("{prefix}/")))
                .map(String::as_str)
        };
        let Some(prefix) = prefix else {
            state.finding(policy, "symlink-outside-closure", input.path, input.target);
            return;
        };
        let (reference, suffix) = split_store_reference(StoreReferenceInput {
            raw: input.target,
            prefix,
        });
        resolve_observed_reference(
            ObservedReferenceInput {
                owner_store_path: input.owner_store_path,
                view: input.path,
                reference,
                suffix,
                kind: ProvenanceReferenceKind::SymlinkTarget,
            },
            policy,
            context,
            state,
        );
        if reference.starts_with(&format!("{}/", context.target_store_prefix)) {
            state.symlink_edges.insert(input.path.to_string(), format!("{reference}{suffix}"));
        }
        return;
    }
    match resolve_relative_path(RelativePathInput {
        path: input.path,
        target: input.target,
        root: input.link_root,
    }) {
        Ok(resolved) => {
            state.symlink_edges.insert(input.path.to_string(), resolved);
        }
        Err(detail) => state.finding(policy, "symlink-path-escape", input.path, detail),
    }
}

fn resolve_relative_path(input: RelativePathInput<'_>) -> Result<String, String> {
    let mut components = input
        .path
        .rsplit_once(PATH_COMPONENT_SEPARATOR)
        .map(|(parent, _)| parent)
        .unwrap_or(input.path)
        .split(PATH_COMPONENT_SEPARATOR)
        .filter(|component| !component.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let root_count = input.root.split(PATH_COMPONENT_SEPARATOR).filter(|component| !component.is_empty()).count();
    for component in input.target.split(PATH_COMPONENT_SEPARATOR) {
        match component {
            EMPTY_PATH | CURRENT_PATH_COMPONENT => {}
            PARENT_PATH_COMPONENT => {
                if components.len() <= root_count {
                    return Err(input.target.to_string());
                }
                components.pop();
            }
            value => components.push(value.to_string()),
        }
    }
    let resolved = format!("/{}", components.join("/"));
    assert!(resolved.starts_with(PATH_COMPONENT_SEPARATOR));
    assert!(components.len() >= root_count);
    Ok(resolved)
}

fn finalize_symlink_graph(policy: &ForeignProvenancePolicy, state: &mut ScanAccumulator) {
    let edges = state.symlink_edges.clone();
    for (source, target) in &edges {
        if !state.observed_paths.contains(target) {
            state.finding(policy, "missing-symlink-target", source, target);
        }
    }
    for source in edges.keys() {
        let mut seen = BTreeSet::new();
        let mut current = source.as_str();
        while let Some(next) = edges.get(current) {
            if !seen.insert(current.to_string()) {
                state.finding(policy, "symlink-loop", source, current);
                break;
            }
            current = next;
        }
    }
}

enum TarEntryOutcome {
    Parsed(ContainerEntry),
    Skip,
    Stop,
}

fn parse_tar_symlink<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    virtual_path: String,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
) -> TarEntryOutcome {
    assert!(!virtual_path.is_empty());
    assert!(policy.max_container_entries > 0);
    let target = match entry.link_name() {
        Ok(Some(target)) => match target.to_str() {
            Some(target) => Some(target.to_string()),
            None => {
                state.finding(policy, "malformed-container", &virtual_path, "tar link target is non-UTF-8");
                return TarEntryOutcome::Skip;
            }
        },
        Ok(None) => {
            state.finding(policy, "malformed-container", &virtual_path, "tar link target is missing");
            return TarEntryOutcome::Skip;
        }
        Err(error) => {
            state.finding(policy, "malformed-container", &virtual_path, format!("tar link target: {error}"));
            return TarEntryOutcome::Skip;
        }
    };
    TarEntryOutcome::Parsed(ContainerEntry {
        path: virtual_path,
        executable: false,
        bytes: Vec::new(),
        symlink_target: target,
    })
}

fn parse_tar_entry<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    input: &PurePayloadInput<'_>,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
) -> TarEntryOutcome {
    assert!(!input.path.is_empty());
    assert!(policy.max_container_entries > 0);
    let path = match entry.path() {
        Ok(path) => match path.to_str() {
            Some(path) => path.to_string(),
            None => {
                state.finding(policy, "malformed-container", input.path, "tar path is non-UTF-8");
                return TarEntryOutcome::Stop;
            }
        },
        Err(error) => {
            state.finding(policy, "malformed-container", input.path, format!("tar path: {error}"));
            return TarEntryOutcome::Stop;
        }
    };
    if container_path_escapes(&path) {
        state.finding(policy, "container-path-escape", input.path, path);
        return TarEntryOutcome::Skip;
    }
    let virtual_path = format!("{}!/{path}", input.path);
    let entry_type = entry.header().entry_type();
    let is_executable = entry.header().mode().is_ok_and(|mode| mode & UNIX_EXECUTABLE_BITS != 0);
    if entry_type.is_symlink() {
        return parse_tar_symlink(entry, virtual_path, policy, state);
    }
    if entry_type.is_file() {
        let Some(bytes) = read_container_entry(entry, policy, state, &virtual_path) else {
            return TarEntryOutcome::Stop;
        };
        return TarEntryOutcome::Parsed(ContainerEntry {
            path: virtual_path,
            executable: is_executable,
            bytes,
            symlink_target: None,
        });
    }
    if !entry_type.is_dir() {
        state.finding(
            policy,
            "unsupported-container-entry",
            &virtual_path,
            format!("tar entry type {}", entry_type.as_byte()),
        );
    }
    TarEntryOutcome::Skip
}

fn inspect_tar(
    input: &PurePayloadInput<'_>,
    container_depth: u32,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    if container_depth >= policy.max_container_depth {
        state.limit(policy, input.path, "container-depth");
        return;
    }
    assert!(container_depth < policy.max_container_depth);
    assert!(policy.max_container_entries > 0);
    let mut archive = tar::Archive::new(Cursor::new(input.bytes));
    let entries = match archive.entries() {
        Ok(entries) => entries,
        Err(error) => {
            state.finding(policy, "malformed-container", input.path, format!("tar: {error}"));
            return;
        }
    };
    let mut parsed = Vec::with_capacity(match usize::try_from(policy.max_container_entries) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, input.path, "container-entries");
            return;
        }
    });
    for entry in entries {
        if state.halted {
            return;
        }
        if state.container_entry_count >= policy.max_container_entries {
            state.limit(policy, input.path, "container-entries");
            return;
        }
        state.container_entry_count = state.container_entry_count.saturating_add(1);
        let mut entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                state.finding(policy, "malformed-container", input.path, format!("tar entry: {error}"));
                return;
            }
        };
        match parse_tar_entry(&mut entry, input, policy, state) {
            TarEntryOutcome::Parsed(entry) => {
                if parsed.try_reserve(1).is_err() {
                    state.limit(policy, input.path, "container-entries");
                    return;
                }
                parsed.push(entry);
            }
            TarEntryOutcome::Skip => {}
            TarEntryOutcome::Stop => return,
        }
    }
    inspect_container_entries(
        ContainerInspection {
            payload: input,
            entries: parsed,
            container_depth,
        },
        policy,
        context,
        state,
    );
}

fn inspect_gzip_stream(
    input: &PurePayloadInput<'_>,
    container_depth: u32,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    let decoder = GzDecoder::new(Cursor::new(input.bytes));
    inspect_compressed_stream(
        CompressedStreamInput {
            payload: input,
            decoder,
            extension: ".gz",
            format_name: "gzip",
            container_depth,
        },
        policy,
        context,
        state,
    );
}

fn inspect_zstd_stream(
    input: &PurePayloadInput<'_>,
    container_depth: u32,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(input.bytes.starts_with(ZSTD_MAGIC));
    assert!(policy.max_container_expanded_bytes > 0);
    let decoder = match ZstdDecoder::new(Cursor::new(input.bytes)) {
        Ok(decoder) => decoder,
        Err(error) => {
            state.finding(policy, "malformed-container", input.path, format!("zstd stream: {error}"));
            return;
        }
    };
    inspect_compressed_stream(
        CompressedStreamInput {
            payload: input,
            decoder,
            extension: ".zst",
            format_name: "zstd",
            container_depth,
        },
        policy,
        context,
        state,
    );
}

fn inspect_compressed_stream<R: Read>(
    input: CompressedStreamInput<'_, R>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.extension.is_empty());
    assert!(!input.format_name.is_empty());
    let mut expanded = Vec::new();
    let mut bounded = input.decoder.take(policy.max_container_expanded_bytes.saturating_add(1));
    if let Err(error) = bounded.read_to_end(&mut expanded) {
        state.finding(
            policy,
            "malformed-container",
            input.payload.path,
            format!("{} stream: {error}", input.format_name),
        );
        return;
    }
    let expanded_size_bytes = match u64::try_from(expanded.len()) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, input.payload.path, "container-expanded-bytes");
            return;
        }
    };
    if expanded_size_bytes > policy.max_container_expanded_bytes {
        state.limit(policy, input.payload.path, "container-expanded-bytes");
        return;
    }
    if expanded.starts_with(CPIO_NEWC_MAGIC) || expanded.starts_with(CPIO_CRC_MAGIC) {
        inspect_cpio(
            CpioInput {
                payload: input.payload,
                bytes: &expanded,
                container_depth: input.container_depth,
            },
            policy,
            context,
            state,
        );
        return;
    }
    let inner_path = input.payload.path.strip_suffix(input.extension).unwrap_or(input.payload.path).to_string();
    inspect_container_entries(
        ContainerInspection {
            payload: input.payload,
            entries: vec![ContainerEntry {
                path: inner_path,
                executable: false,
                bytes: expanded,
                symlink_target: None,
            }],
            container_depth: input.container_depth,
        },
        policy,
        context,
        state,
    );
}

fn inspect_cpio(
    input: CpioInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    if input.container_depth >= policy.max_container_depth {
        state.limit(policy, input.payload.path, "container-depth");
        return;
    }
    assert!(input.container_depth < policy.max_container_depth);
    assert!(policy.max_container_entries > 0);
    let entries = match parse_newc_entries(input.bytes, policy, state, input.payload.path) {
        Some(entries) => entries,
        None => return,
    };
    inspect_container_entries(
        ContainerInspection {
            payload: input.payload,
            entries,
            container_depth: input.container_depth,
        },
        policy,
        context,
        state,
    );
}

struct NewcNameInput<'a> {
    bytes: &'a [u8],
    header_end: usize,
    header: &'a [u8],
    outer_path: &'a str,
}

struct NewcHeader {
    mode: u32,
    file_size_bytes: usize,
    name: String,
    data_start_bytes: usize,
}

struct NewcEntryInput {
    mode: u32,
    virtual_path: String,
    data: Vec<u8>,
    outer_path: String,
}

fn parse_newc_name(
    input: NewcNameInput<'_>,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
) -> Option<(String, usize)> {
    assert!(input.header_end <= input.bytes.len());
    assert!(!input.outer_path.is_empty());
    let name_size_bytes = parse_hex_u32(&input.header[CPIO_NAME_SIZE_START..CPIO_NAME_SIZE_END])
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| {
            state.finding(policy, "malformed-container", input.outer_path, "cpio name size is invalid");
            None
        })?;
    if name_size_bytes == 0 {
        state.finding(policy, "malformed-container", input.outer_path, "cpio entry name is empty");
        return None;
    }
    let name_end = input.header_end.checked_add(name_size_bytes).or_else(|| {
        state.finding(policy, "malformed-container", input.outer_path, "cpio name offset overflowed");
        None
    })?;
    let name_bytes = input.bytes.get(input.header_end..name_end).or_else(|| {
        state.finding(policy, "malformed-container", input.outer_path, "cpio entry name is truncated");
        None
    })?;
    let name_bytes = name_bytes.strip_suffix(&[0]).unwrap_or(name_bytes);
    let name = std::str::from_utf8(name_bytes)
        .map(str::to_string)
        .map_err(|error| {
            state.finding(policy, "malformed-container", input.outer_path, format!("cpio name is non-UTF-8: {error}"));
        })
        .ok()?;
    let data_start_bytes = align_up(AlignmentInput {
        value_bytes: name_end,
        alignment_bytes: CPIO_ALIGNMENT_BYTES,
    })
    .or_else(|| {
        state.finding(policy, "malformed-container", input.outer_path, "cpio data offset overflowed");
        None
    })?;
    Some((name, data_start_bytes))
}

fn parse_newc_header(
    bytes: &[u8],
    offset_bytes: usize,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
    outer_path: &str,
) -> Option<NewcHeader> {
    assert!(policy.max_container_entries > 0);
    assert!(!outer_path.is_empty());
    let header_end = offset_bytes.checked_add(CPIO_HEADER_BYTES).or_else(|| {
        state.finding(policy, "malformed-container", outer_path, "cpio header offset overflowed");
        None
    })?;
    let header = bytes.get(offset_bytes..header_end).or_else(|| {
        state.finding(policy, "malformed-container", outer_path, "cpio header is truncated");
        None
    })?;
    if header.starts_with(CPIO_CRC_MAGIC) {
        state.finding(policy, "unsupported-container", outer_path, "cpio CRC archives are not accepted");
        return None;
    }
    if !header.starts_with(CPIO_NEWC_MAGIC) {
        state.finding(policy, "malformed-container", outer_path, "cpio newc header magic is invalid");
        return None;
    }
    let mode = parse_hex_u32(&header[CPIO_MODE_START..CPIO_MODE_END]).or_else(|| {
        state.finding(policy, "malformed-container", outer_path, "cpio mode is invalid");
        None
    })?;
    let file_size_bytes = parse_hex_u32(&header[CPIO_FILE_SIZE_START..CPIO_FILE_SIZE_END])
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| {
            state.finding(policy, "malformed-container", outer_path, "cpio file size is invalid");
            None
        })?;
    let (name, data_start_bytes) = parse_newc_name(
        NewcNameInput {
            bytes,
            header_end,
            header,
            outer_path,
        },
        policy,
        state,
    )?;
    Some(NewcHeader {
        mode,
        file_size_bytes,
        name,
        data_start_bytes,
    })
}

fn newc_container_entry(
    input: NewcEntryInput,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
) -> Option<Option<ContainerEntry>> {
    assert!(!input.virtual_path.is_empty());
    assert!(!input.outer_path.is_empty());
    let file_type = input.mode & UNIX_FILE_TYPE_MASK;
    if file_type == UNIX_REGULAR_FILE_TYPE {
        return Some(Some(ContainerEntry {
            path: input.virtual_path,
            executable: input.mode & UNIX_EXECUTABLE_BITS != 0,
            bytes: input.data,
            symlink_target: None,
        }));
    }
    if file_type == UNIX_SYMLINK_FILE_TYPE {
        let target = String::from_utf8(input.data).map_err(|error| {
            state.finding(
                policy,
                "malformed-container",
                &input.outer_path,
                format!("cpio symlink is non-UTF-8: {error}"),
            );
        });
        return match target {
            Ok(target) => Some(Some(ContainerEntry {
                path: input.virtual_path,
                executable: false,
                bytes: Vec::new(),
                symlink_target: Some(target),
            })),
            Err(()) => None,
        };
    }
    if file_type != UNIX_DIRECTORY_FILE_TYPE {
        state.finding(
            policy,
            "unsupported-container-entry",
            &input.virtual_path,
            format!("cpio mode {:o}", input.mode),
        );
    }
    Some(None)
}

fn parse_newc_entries(
    bytes: &[u8],
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
    outer_path: &str,
) -> Option<Vec<ContainerEntry>> {
    assert!(!outer_path.is_empty());
    assert!(policy.max_container_entries > 0);
    let mut offset_bytes = 0usize;
    let mut entries = Vec::with_capacity(match usize::try_from(policy.max_container_entries) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, outer_path, "container-entries");
            return None;
        }
    });
    while offset_bytes < bytes.len() {
        if state.container_entry_count >= policy.max_container_entries {
            state.limit(policy, outer_path, "container-entries");
            return None;
        }
        let header = parse_newc_header(bytes, offset_bytes, policy, state, outer_path)?;
        if header.name == CPIO_TRAILER_NAME {
            return Some(entries);
        }
        if container_path_escapes(&header.name) {
            state.finding(policy, "container-path-escape", outer_path, &header.name);
            return None;
        }
        let data_end_bytes = header.data_start_bytes.checked_add(header.file_size_bytes).or_else(|| {
            state.finding(policy, "malformed-container", outer_path, "cpio payload offset overflowed");
            None
        })?;
        let data = bytes.get(header.data_start_bytes..data_end_bytes).map(<[u8]>::to_vec).or_else(|| {
            state.finding(policy, "malformed-container", outer_path, "cpio payload is truncated");
            None
        })?;
        state.container_entry_count = state.container_entry_count.saturating_add(1);
        if let Some(entry) = newc_container_entry(
            NewcEntryInput {
                mode: header.mode,
                virtual_path: format!("{outer_path}!/{}", header.name),
                data,
                outer_path: outer_path.to_string(),
            },
            policy,
            state,
        )? {
            if entries.try_reserve(1).is_err() {
                state.limit(policy, outer_path, "container-entries");
                return None;
            }
            entries.push(entry);
        }
        offset_bytes = align_up(AlignmentInput {
            value_bytes: data_end_bytes,
            alignment_bytes: CPIO_ALIGNMENT_BYTES,
        })
        .or_else(|| {
            state.finding(policy, "malformed-container", outer_path, "cpio next-entry offset overflowed");
            None
        })?;
    }
    state.finding(policy, "malformed-container", outer_path, "cpio trailer is missing");
    None
}

fn inspect_container_symlink(
    input: ContainerSymlinkInput<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.path.is_empty());
    assert!(!input.payload.owner_store_path.is_empty());
    let target_size_bytes = match u64::try_from(input.target.len()) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, input.path, "symlink-bytes");
            return;
        }
    };
    state.payloads.push(ProvenancePayloadObservation {
        owner_store_path: input.payload.owner_store_path.to_string(),
        path: input.path.to_string(),
        node_identity: format!("container-symlink:{}", blake3::hash(input.target.as_bytes()).to_hex()),
        class: ProvenancePayloadClass::Symlink,
        executable: false,
        byte_count: target_size_bytes,
        content_blake3: blake3::hash(input.target.as_bytes()).to_hex().to_string(),
    });
    inspect_symlink(
        SymlinkInput {
            owner_store_path: input.payload.owner_store_path,
            path: input.path,
            target: input.target,
            link_root: input.payload.path,
        },
        policy,
        context,
        state,
    );
}

fn inspect_container_entries(
    input: ContainerInspection<'_>,
    policy: &ForeignProvenancePolicy,
    context: &ResolutionContext<'_>,
    state: &mut ScanAccumulator,
) {
    assert!(!input.payload.path.is_empty());
    assert!(u32::try_from(input.entries.len()).is_ok_and(|value| value <= policy.max_container_entries));
    let entry_paths = input.entries.iter().map(|entry| entry.path.clone()).collect::<BTreeSet<_>>();
    state.observed_paths.extend(entry_paths);
    for entry in input.entries {
        if state.halted {
            return;
        }
        if let Some(target) = entry.symlink_target {
            inspect_container_symlink(
                ContainerSymlinkInput {
                    payload: input.payload,
                    path: &entry.path,
                    target: &target,
                },
                policy,
                context,
                state,
            );
            continue;
        }
        let expanded_size_bytes = match u64::try_from(entry.bytes.len()) {
            Ok(value) => value,
            Err(_) => {
                state.limit(policy, &entry.path, "container-expanded-bytes");
                return;
            }
        };
        state.container_expanded_bytes = state.container_expanded_bytes.saturating_add(expanded_size_bytes);
        if state.container_expanded_bytes > policy.max_container_expanded_bytes {
            state.limit(policy, &entry.path, "container-expanded-bytes");
            return;
        }
        let identity = format!("container-entry:{}", blake3::hash(&entry.bytes).to_hex());
        inspect_payload(
            PurePayloadInput {
                owner_store_path: input.payload.owner_store_path,
                path: &entry.path,
                node_identity: &identity,
                executable: entry.executable,
                bytes: &entry.bytes,
            },
            input.container_depth.saturating_add(1),
            policy,
            context,
            state,
        );
    }
}

fn read_container_entry(
    entry: &mut impl Read,
    policy: &ForeignProvenancePolicy,
    state: &mut ScanAccumulator,
    path: &str,
) -> Option<Vec<u8>> {
    assert!(!path.is_empty());
    assert!(policy.max_container_expanded_bytes > 0);
    let mut bytes = Vec::new();
    let mut bounded = entry.take(policy.max_container_expanded_bytes.saturating_add(1));
    if let Err(error) = bounded.read_to_end(&mut bytes) {
        state.finding(policy, "malformed-container", path, error.to_string());
        return None;
    }
    let expanded_size_bytes = match u64::try_from(bytes.len()) {
        Ok(value) => value,
        Err(_) => {
            state.limit(policy, path, "container-expanded-bytes");
            return None;
        }
    };
    if expanded_size_bytes > policy.max_container_expanded_bytes {
        state.limit(policy, path, "container-expanded-bytes");
        return None;
    }
    Some(bytes)
}

fn finalize_scan(
    completion: ScanCompletion,
    mut path_infos: Vec<ProvenancePathInfoObservation>,
    mut state: ScanAccumulator,
) -> CastoreProvenanceScan {
    path_infos.sort();
    state.payloads.sort();
    state.references.sort();
    state.findings.sort();
    state.findings.dedup();
    CastoreProvenanceScan {
        preflight_complete: completion.is_preflight_complete,
        traversal_complete: completion.is_traversal_complete,
        path_infos,
        payloads: state.payloads,
        references: state.references,
        findings: state.findings,
        visited_node_count: state.visited_node_count,
        visited_blob_count: state.visited_blob_count,
        read_byte_count: state.read_byte_count,
        duplicate_node_count: state.duplicate_node_count,
    }
}

fn path_info_has_trusted_signature(
    path_info: &snix_store::path_info::PathInfo,
    trusted_keys: &[VerifyingKey],
    store_dir: &str,
) -> bool {
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let references = path_info.references.iter().map(|reference| reference.as_ref()).collect::<Vec<_>>();
    let material = fingerprint_with_store_dir(
        &store_path_ref,
        &path_info.nar_sha256,
        path_info.nar_size,
        references.iter(),
        store_dir,
    );
    path_info.signatures.iter().any(|signature| {
        let signature = signature.as_ref();
        trusted_keys.iter().any(|key| key.verify(&material, &signature))
    })
}

fn node_identity(node: &Node) -> String {
    match node {
        Node::Directory { digest, size } => format!("directory:{digest}:{size}"),
        Node::File {
            digest,
            size,
            executable,
        } => format!("blob:{digest}:{size}:{executable}"),
        Node::Symlink { target } => format!("symlink:{}", blake3::hash(target.as_ref()).to_hex()),
    }
}

fn is_untranslated_foreign_reference(
    reference: &str,
    paths: &BTreeMap<String, String>,
    foreign_prefixes: &[String],
) -> bool {
    paths.get(reference).is_some_and(|target| target != reference)
        || foreign_prefixes.iter().any(|prefix| reference.starts_with(&format!("{prefix}/")))
}

fn foreign_store_prefixes(paths: &BTreeMap<String, String>) -> Vec<String> {
    let mut prefixes = BTreeSet::new();
    for (foreign, target) in paths {
        if foreign == target {
            continue;
        }
        if let Some((prefix, _)) = foreign.rsplit_once(PATH_COMPONENT_SEPARATOR) {
            prefixes.insert(prefix.to_string());
        }
    }
    prefixes.into_iter().collect()
}

fn looks_like_tar(bytes: &[u8]) -> bool {
    bytes.get(TAR_USTAR_OFFSET..TAR_USTAR_END) == Some(TAR_USTAR_MAGIC)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn reference_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while let Some(byte) = bytes.get(end) {
        if byte.is_ascii_whitespace() || matches!(*byte, 0 | b'\'' | b'"' | b':' | b';' | b',' | b')' | b'(') {
            break;
        }
        end = end.saturating_add(1);
    }
    end
}

fn split_store_reference<'a>(input: StoreReferenceInput<'a>) -> (&'a str, &'a str) {
    let root_end = input
        .raw
        .get(input.prefix.len().saturating_add(1)..)
        .and_then(|rest| {
            rest.find(PATH_COMPONENT_SEPARATOR)
                .map(|index| input.prefix.len().saturating_add(1).saturating_add(index))
        })
        .unwrap_or(input.raw.len());
    input.raw.split_at(root_end)
}

fn is_well_formed_store_reference(input: StoreReferenceInput<'_>) -> bool {
    StorePath::<String>::from_absolute_path_with_prefix(input.raw.as_bytes(), input.prefix).is_ok()
}

fn contains_parent_component(path: &str) -> bool {
    path.split(PATH_COMPONENT_SEPARATOR).any(|component| component == PARENT_PATH_COMPONENT)
}

fn normalize_store_suffix(suffix: &str) -> Result<String, ()> {
    assert!(!PARENT_PATH_COMPONENT.is_empty());
    assert_eq!(PATH_COMPONENT_SEPARATOR, '/');
    if suffix.is_empty() {
        return Ok(String::new());
    }
    if !suffix.starts_with(PATH_COMPONENT_SEPARATOR) {
        return Err(());
    }
    let mut components = Vec::with_capacity(suffix.split(PATH_COMPONENT_SEPARATOR).count());
    for component in suffix.split(PATH_COMPONENT_SEPARATOR) {
        match component {
            EMPTY_PATH | CURRENT_PATH_COMPONENT => {}
            PARENT_PATH_COMPONENT => {
                components.pop().ok_or(())?;
            }
            _ => components.push(component),
        }
    }
    if components.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("/{path}", path = components.join("/")))
    }
}

fn container_path_escapes(path: &str) -> bool {
    path.starts_with(PATH_COMPONENT_SEPARATOR) || contains_parent_component(path)
}

fn parse_hex_u32(bytes: &[u8]) -> Option<u32> {
    std::str::from_utf8(bytes).ok().and_then(|value| u32::from_str_radix(value, HEX_RADIX).ok())
}

fn align_up(input: AlignmentInput) -> Option<usize> {
    let alignment_bytes = input.alignment_bytes;
    if alignment_bytes == 0 {
        return None;
    }
    assert!(alignment_bytes > 0);
    let remainder = input.value_bytes % alignment_bytes;
    assert!(remainder < alignment_bytes);
    if remainder == 0 {
        Some(input.value_bytes)
    } else {
        input.value_bytes.checked_add(alignment_bytes.saturating_sub(remainder))
    }
}

#[cfg(test)]
// r[verify store_lifecycle.tiger_conformance]
mod tests {
    use std::collections::HashMap;
    use std::io::Write;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use nix_compat::narinfo::SigningKey;
    use nix_compat::narinfo::parse_keypair;
    use nix_compat::store_path::StorePath;
    use proptest::prelude::*;
    use snix_castore::B3Digest;
    use snix_castore::Directory;
    use snix_castore::PathComponent;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryPutter;
    use snix_castore::directoryservice::DirectoryService;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::StoreHandleServices;

    const TEST_OWNER: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-owner";
    const TEST_DEP: &str = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-dep";
    const TEST_FOREIGN: &str = "/gnu/store/cccccccccccccccccccccccccccccccc-dep";
    const TEST_NODE: &str = "blob:test";
    const TEST_SIGNING_KEY: &str =
        "audit-test-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
    const TEST_STORE_PREFIX: &str = "/mantle/store";
    const TEST_PATHINFO_CAPACITY: usize = 32;
    const CPIO_HEX_FIELD_WIDTH: usize = 8;
    const TEST_CPIO_INODE: u32 = 1;
    const TEST_CPIO_LINK_COUNT: u32 = 1;
    const TEST_EXECUTABLE_MODE: u32 = UNIX_REGULAR_FILE_TYPE | 0o755;
    const TEST_SYMLINK_MODE: u32 = 0o777;
    const EXPECTED_SYMLINK_TREE_NODES: u32 = 3;
    const PROPERTY_MAX_BYTES: usize = 4096;
    const PROPERTY_MAX_COMPONENTS: usize = 16;
    const PROPERTY_COMPONENT_REGEX: &str = "[a-z]{1,8}";

    struct StubDirectoryService {
        directories: Mutex<HashMap<B3Digest, Directory>>,
    }

    impl StubDirectoryService {
        fn new() -> Self {
            Self {
                directories: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl DirectoryService for StubDirectoryService {
        async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.directories.lock().unwrap().get(digest).cloned())
        }

        async fn put(&self, directory: Directory) -> Result<B3Digest, Box<dyn std::error::Error + Send + Sync>> {
            let digest = directory.digest();
            self.directories.lock().unwrap().insert(digest, directory);
            Ok(digest)
        }

        fn get_recursive(
            &self,
            _root_directory_digest: &B3Digest,
        ) -> futures::stream::BoxStream<'_, Result<Directory, Box<dyn std::error::Error + Send + Sync>>> {
            unimplemented!("recursive directory reads are not used by provenance tests")
        }

        fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
            unimplemented!("multi-directory writes are not used by provenance tests")
        }
    }

    struct ScanFixture {
        _temp: TempDir,
        handle: StoreHandle,
        pathinfo: Arc<dyn PathInfoService>,
        blob: Arc<MemoryBlobService>,
        directory: Arc<StubDirectoryService>,
        signing_key: SigningKey<ed25519_dalek::SigningKey>,
        verifying_key: VerifyingKey,
    }

    fn scan_fixture() -> ScanFixture {
        let temp = TempDir::new().unwrap();
        let blob = Arc::new(MemoryBlobService::default());
        let directory = Arc::new(StubDirectoryService::new());
        let pathinfo = Arc::new(LruPathInfoService::with_capacity(
            "provenance-test".to_string(),
            NonZeroUsize::new(TEST_PATHINFO_CAPACITY).unwrap(),
        )) as Arc<dyn PathInfoService>;
        let (signing_key, verifying_key) = parse_keypair(TEST_SIGNING_KEY).unwrap();
        let handle = StoreHandle::from_services_with_store_dir(
            StoreHandleServices {
                blob_service: blob.clone(),
                directory_service: directory.clone(),
                pathinfo_service: pathinfo.clone(),
                remote_pathinfo: None,
                state_dir: temp.path().join("state"),
                output_dir_str: temp.path().join("out").to_string_lossy().into_owned(),
                publishers: Vec::new(),
            },
            TEST_STORE_PREFIX.to_string(),
        );
        ScanFixture {
            _temp: temp,
            handle,
            pathinfo,
            blob,
            directory,
            signing_key,
            verifying_key,
        }
    }

    fn signed_path_info(
        logical_path: &str,
        node: Node,
        signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> PathInfo {
        let store_path: StorePath<String> =
            StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), TEST_STORE_PREFIX).unwrap();
        let mut path_info = PathInfo {
            store_path,
            node,
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [7u8; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        let store_path_ref: StorePathRef = path_info.store_path.as_ref();
        let references = path_info.references.iter().map(StorePath::as_ref).collect::<Vec<_>>();
        let material = fingerprint_with_store_dir(
            &store_path_ref,
            &path_info.nar_sha256,
            path_info.nar_size,
            references.iter(),
            TEST_STORE_PREFIX,
        );
        path_info.signatures.push(signing_key.sign(material.as_bytes()).to_owned());
        path_info
    }

    fn expected_fact() -> ProvenanceExpectedPathInfo {
        ProvenanceExpectedPathInfo {
            nar_sha256: data_encoding::HEXLOWER.encode(&[7u8; 32]),
            nar_size: 1,
        }
    }

    async fn write_blob(blob: &MemoryBlobService, bytes: &[u8]) -> B3Digest {
        let mut writer = blob.open_write().await;
        writer.write_all(bytes).await.unwrap();
        writer.close().await.unwrap()
    }

    fn append_newc_entry(archive: &mut Vec<u8>, name: &str, mode: u32, bytes: &[u8]) {
        let name_size = u32::try_from(name.len().saturating_add(1)).unwrap();
        let file_size = u32::try_from(bytes.len()).unwrap();
        let fields = [
            TEST_CPIO_INODE,
            mode,
            0,
            0,
            TEST_CPIO_LINK_COUNT,
            0,
            file_size,
            0,
            0,
            0,
            0,
            name_size,
            0,
        ];
        let header_start = archive.len();
        archive.extend_from_slice(CPIO_NEWC_MAGIC);
        for field in fields {
            archive.extend_from_slice(format!("{field:0width$x}", width = CPIO_HEX_FIELD_WIDTH).as_bytes());
        }
        assert_eq!(archive.len().saturating_sub(header_start), CPIO_HEADER_BYTES);
        archive.extend_from_slice(name.as_bytes());
        archive.push(0);
        while !archive.len().is_multiple_of(CPIO_ALIGNMENT_BYTES) {
            archive.push(0);
        }
        archive.extend_from_slice(bytes);
        while !archive.len().is_multiple_of(CPIO_ALIGNMENT_BYTES) {
            archive.push(0);
        }
    }

    fn newc_archive(name: &str, mode: u32, bytes: &[u8]) -> Vec<u8> {
        let mut archive = Vec::new();
        append_newc_entry(&mut archive, name, mode, bytes);
        append_newc_entry(&mut archive, CPIO_TRAILER_NAME, 0, &[]);
        archive
    }

    fn policy() -> ForeignProvenancePolicy {
        ForeignProvenancePolicy {
            schema: FOREIGN_PROVENANCE_POLICY_SCHEMA.to_string(),
            max_path_infos: 16,
            max_nodes: 64,
            max_blobs: 32,
            max_blob_bytes: 1_048_576,
            max_total_bytes: 4_194_304,
            max_depth: 16,
            max_findings: 32,
            max_duplicates: 16,
            max_container_entries: 32,
            max_container_expanded_bytes: 1_048_576,
            max_container_depth: 4,
            max_path_bytes: 4096,
            max_shebang_bytes: 256,
            allowed_profile_paths: vec!["/bin/sh".to_string()],
        }
    }

    fn context() -> ResolutionContext<'static> {
        let admitted = Box::leak(Box::new([TEST_OWNER.to_string(), TEST_DEP.to_string()].into_iter().collect()));
        let declared = Box::leak(Box::new(BTreeMap::from([(
            TEST_OWNER.to_string(),
            [TEST_DEP.to_string()].into_iter().collect(),
        )])));
        let foreign = Box::leak(Box::new(BTreeMap::from([(TEST_FOREIGN.to_string(), TEST_DEP.to_string())])));
        ResolutionContext {
            target_store_prefix: "/mantle/store",
            admitted_closure_paths: admitted,
            declared_references_by_output: declared,
            foreign_to_target_paths: foreign,
            foreign_store_prefixes: vec!["/gnu/store".to_string()],
            allowed_profile_paths: ["/bin/sh".to_string()].into_iter().collect(),
        }
    }

    fn inspect(bytes: &[u8], executable: bool, path: &str) -> ScanAccumulator {
        let mut state = ScanAccumulator::new();
        inspect_payload(
            PurePayloadInput {
                owner_store_path: TEST_OWNER,
                path,
                node_identity: TEST_NODE,
                executable,
                bytes,
            },
            0,
            &policy(),
            &context(),
            &mut state,
        );
        state
    }

    proptest! {
        #[test]
        fn payload_classification_is_deterministic_for_arbitrary_bytes(
            bytes in proptest::collection::vec(any::<u8>(), 0..PROPERTY_MAX_BYTES),
            executable in any::<bool>(),
        ) {
            let first = classify_payload(executable, "arbitrary", &bytes);
            let second = classify_payload(executable, "arbitrary", &bytes);
            prop_assert_eq!(first, second);
        }

        #[test]
        fn lexical_relative_resolution_never_returns_a_path_outside_root(
            components in proptest::collection::vec(PROPERTY_COMPONENT_REGEX, 0..PROPERTY_MAX_COMPONENTS),
        ) {
            let target = components.join("/");
            let resolved = resolve_relative_path(RelativePathInput {
                path: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-owner/link",
                target: &target,
                root: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-owner",
            }).expect("generated components remain within the root");
            prop_assert!(resolved.starts_with("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-owner"));
        }
    }

    #[test]
    fn policy_accepts_bounded_sorted_profile_paths_and_rejects_bad_limits() {
        let accepted = policy();
        assert!(validate_foreign_provenance_policy(&accepted).is_ok());
        let mut zero = accepted.clone();
        zero.max_nodes = 0;
        assert_eq!(
            validate_foreign_provenance_policy(&zero),
            Err("foreign provenance policy limit must be nonzero: max_nodes".to_string())
        );
        let mut unsorted = accepted;
        unsorted.allowed_profile_paths = vec!["/z".to_string(), "/a".to_string()];
        assert_eq!(
            validate_foreign_provenance_policy(&unsorted),
            Err("allowed profile paths must be sorted and unique".to_string())
        );
    }

    #[test]
    fn classifier_covers_data_elf_script_and_unknown_executable() {
        let mut elf = vec![0u8; ELF64_HEADER_BYTES];
        elf[..ELF_MAGIC.len()].copy_from_slice(ELF_MAGIC);
        elf[ELF_CLASS_OFFSET] = ELF_CLASS_64;
        elf[ELF_DATA_OFFSET] = ELF_DATA_LITTLE_ENDIAN;
        elf[ELF_VERSION_OFFSET] = ELF_CURRENT_VERSION;
        assert_eq!(classify_payload(false, "data", b"plain"), ProvenancePayloadClass::Data);
        assert_eq!(classify_payload(true, "elf", &elf), ProvenancePayloadClass::Elf);
        assert_eq!(classify_payload(true, "script", b"#!/bin/sh\n"), ProvenancePayloadClass::Script);
        let libtool = b"# libdemo.la - a libtool library file\n# Generated by libtool 2.5\ndlname='libdemo.so'\nlibrary_names='libdemo.so'\ninstalled=yes\n";
        assert_eq!(classify_payload(true, "libdemo.la", libtool), ProvenancePayloadClass::LibtoolArchive);
        assert_eq!(
            classify_payload(true, "libdemo.la", b"not libtool metadata"),
            ProvenancePayloadClass::UnsupportedExecutable
        );
        assert_eq!(
            classify_payload(true, "unknown", b"not executable syntax"),
            ProvenancePayloadClass::UnsupportedExecutable
        );
        let ambient_perl_polyglot = b"# -*- perl -*-\neval \"q () {\n  :\n}\";\nq { exec perl -e 'exit 0' }\n";
        assert_eq!(
            classify_payload(true, "mtrace", ambient_perl_polyglot),
            ProvenancePayloadClass::UnsupportedExecutable
        );
        assert_eq!(classify_payload(true, "bad-elf", ELF_MAGIC), ProvenancePayloadClass::Malformed);
        assert_eq!(classify_payload(false, "payload.zst", ZSTD_MAGIC), ProvenancePayloadClass::ZstdStream);
        assert_eq!(classify_payload(false, "payload.zip", ZIP_MAGIC), ProvenancePayloadClass::UnsupportedContainer);
    }

    #[test]
    fn shebang_accepts_profile_and_rejects_relative_missing_and_non_utf8_targets() {
        let accepted = inspect(b"#!/bin/sh\nexit 0\n", true, "script");
        assert!(accepted.findings.is_empty());
        assert_eq!(accepted.references.len(), 1);
        assert_eq!(accepted.references[0].kind, ProvenanceReferenceKind::ShebangInterpreter);

        let store_shebang = inspect(format!("#!{TEST_DEP}/bin/tool\n").as_bytes(), true, "store-script");
        assert!(store_shebang.findings.is_empty());
        assert_eq!(store_shebang.references[0].reference, TEST_DEP);
        assert_eq!(store_shebang.references[0].suffix, "/bin/tool");

        let relative = inspect(b"#!bin/sh\n", true, "relative");
        assert_eq!(relative.findings[0].code, "relative-shebang");
        let missing = inspect(b"#!/unknown/interpreter\n", true, "missing");
        assert_eq!(missing.findings[0].code, "missing-executable-target");
        let non_utf8 = inspect(b"#!\xff\n", true, "non-utf8");
        assert_eq!(non_utf8.findings[0].code, "malformed-shebang");
    }

    #[test]
    fn exact_reference_resolution_accepts_declared_target_and_rejects_foreign_unknown_and_escape() {
        let accepted = inspect(format!("run {TEST_DEP}/bin/tool").as_bytes(), false, "accepted");
        assert!(accepted.findings.is_empty());
        assert_eq!(accepted.references[0].reference, TEST_DEP);
        assert_eq!(accepted.references[0].suffix, "/bin/tool");

        let normalized = inspect(format!("run {TEST_DEP}/lib/../lib/tool").as_bytes(), false, "normalized");
        assert!(normalized.findings.is_empty());
        assert_eq!(normalized.references[0].suffix, "/lib/tool");

        let foreign = inspect(format!("run {TEST_FOREIGN}/bin/tool").as_bytes(), false, "foreign");
        assert_eq!(foreign.findings[0].code, "untranslated-foreign-path");
        let unknown = inspect(b"/mantle/store/dddddddddddddddddddddddddddddddd-unknown/bin", false, "unknown");
        assert_eq!(unknown.findings[0].code, "missing-target");
        let escape = inspect(format!("{TEST_DEP}/../escape").as_bytes(), false, "escape");
        assert_eq!(escape.findings[0].code, "reference-path-escape");
    }

    #[test]
    fn preserved_identity_paths_are_targets_not_untranslated_foreign_references() {
        const PRESERVED: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-preserved";
        let identity = BTreeMap::from([(PRESERVED.to_string(), PRESERVED.to_string())]);
        let identity_prefixes = foreign_store_prefixes(&identity);
        assert!(identity_prefixes.is_empty());
        assert!(!is_untranslated_foreign_reference(PRESERVED, &identity, &identity_prefixes));

        let translated = BTreeMap::from([(PRESERVED.to_string(), TEST_DEP.to_string())]);
        let translated_prefixes = foreign_store_prefixes(&translated);
        assert_eq!(translated_prefixes, vec!["/nix/store".to_string()]);
        assert!(is_untranslated_foreign_reference(PRESERVED, &translated, &translated_prefixes));
    }

    #[test]
    fn symlink_resolution_rejects_escape_missing_and_loop() {
        let mut state = ScanAccumulator::new();
        let policy = policy();
        let context = context();
        state.observed_paths.insert(format!("{TEST_OWNER}/a"));
        state.observed_paths.insert(format!("{TEST_OWNER}/b"));
        inspect_symlink(
            SymlinkInput {
                owner_store_path: TEST_OWNER,
                path: &format!("{TEST_OWNER}/a"),
                target: "b",
                link_root: TEST_OWNER,
            },
            &policy,
            &context,
            &mut state,
        );
        inspect_symlink(
            SymlinkInput {
                owner_store_path: TEST_OWNER,
                path: &format!("{TEST_OWNER}/b"),
                target: "a",
                link_root: TEST_OWNER,
            },
            &policy,
            &context,
            &mut state,
        );
        finalize_symlink_graph(&policy, &mut state);
        assert!(state.findings.iter().any(|finding| finding.code == "symlink-loop"));

        let mut escape = ScanAccumulator::new();
        inspect_symlink(
            SymlinkInput {
                owner_store_path: TEST_OWNER,
                path: &format!("{TEST_OWNER}/a"),
                target: "../..",
                link_root: TEST_OWNER,
            },
            &policy,
            &context,
            &mut escape,
        );
        assert_eq!(escape.findings[0].code, "symlink-path-escape");
    }

    #[test]
    fn tar_reader_finds_hidden_unclassified_executable_and_path_escape() {
        let mut archive_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut archive_bytes);
            let payload = b"unknown";
            let mut header = tar::Header::new_gnu();
            header.set_size(u64::try_from(payload.len()).unwrap());
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, "bin/hidden", Cursor::new(payload)).unwrap();
            let mut link_header = tar::Header::new_gnu();
            link_header.set_entry_type(tar::EntryType::Symlink);
            link_header.set_size(0);
            link_header.set_mode(TEST_SYMLINK_MODE);
            link_header.set_link_name("../../outside").unwrap();
            link_header.set_cksum();
            builder.append_data(&mut link_header, "dir/link", Cursor::new([])).unwrap();
            builder.finish().unwrap();
        }
        let state = inspect(&archive_bytes, false, "payload.tar");
        assert!(state.findings.iter().any(|finding| finding.code == "unclassified-executable"));
        assert!(state.findings.iter().any(|finding| finding.code == "symlink-path-escape"));
    }

    #[test]
    fn cpio_and_gzip_initrd_readers_classify_nested_executable_scripts() {
        let archive = newc_archive("bin/tool", TEST_EXECUTABLE_MODE, b"#!/bin/sh\nexit 0\n");
        let plain = inspect(&archive, false, "initrd.cpio");
        assert!(plain.findings.is_empty());
        assert!(plain.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::CpioInitrd));
        assert!(plain.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::Script));

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&archive).unwrap();
        let compressed = encoder.finish().unwrap();
        let gzip = inspect(&compressed, false, "initrd.cpio.gz");
        assert!(gzip.findings.is_empty());
        assert!(gzip.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::GzipStream));
        assert!(gzip.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::Script));
    }

    #[test]
    fn generic_compressed_streams_are_bounded_and_scanned_as_single_payloads() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"plain data").unwrap();
        let compressed = encoder.finish().unwrap();
        let gzip = inspect(&compressed, false, "manual.txt.gz");
        assert!(gzip.findings.is_empty());
        assert!(gzip.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::GzipStream));
        assert!(
            gzip.payloads
                .iter()
                .any(|payload| { payload.path == "manual.txt" && payload.class == ProvenancePayloadClass::Data })
        );

        let compressed = zstd::stream::encode_all(Cursor::new(b"plain data"), 0).unwrap();
        let zstd = inspect(&compressed, false, "manual.txt.zst");
        assert!(zstd.findings.is_empty());
        assert!(zstd.payloads.iter().any(|payload| payload.class == ProvenancePayloadClass::ZstdStream));
        assert!(
            zstd.payloads
                .iter()
                .any(|payload| { payload.path == "manual.txt" && payload.class == ProvenancePayloadClass::Data })
        );
    }

    #[test]
    fn malformed_and_bounded_containers_fail_closed() {
        let malformed = inspect(b"not-a-tar", false, "payload.tar");
        assert_eq!(malformed.findings[0].code, "malformed-container");
        let truncated_cpio = inspect(CPIO_NEWC_MAGIC, false, "initrd.cpio");
        assert_eq!(truncated_cpio.findings[0].code, "malformed-container");
        let truncated_zstd = inspect(ZSTD_MAGIC, false, "manual.txt.zst");
        assert_eq!(truncated_zstd.findings[0].code, "malformed-container");
        let mut bounded_policy = policy();
        bounded_policy.max_container_entries = 1;
        let mut state = ScanAccumulator::new();
        state.container_entry_count = bounded_policy.max_container_entries;
        state.limit(&bounded_policy, "archive", "container-entries");
        assert!(state.halted);
        assert_eq!(state.findings[0].detail, "container-entries");
    }

    #[test]
    fn equivalent_observation_order_canonicalizes_deterministically() {
        let mut left = ScanAccumulator::new();
        left.findings.push(ProvenanceFinding {
            code: "b".to_string(),
            path: "two".to_string(),
            detail: String::new(),
        });
        left.findings.push(ProvenanceFinding {
            code: "a".to_string(),
            path: "one".to_string(),
            detail: String::new(),
        });
        let mut right = ScanAccumulator::new();
        right.findings = left.findings.iter().cloned().rev().collect();
        let completion = ScanCompletion {
            is_preflight_complete: true,
            is_traversal_complete: true,
        };
        assert_eq!(
            finalize_scan(completion, Vec::new(), left),
            finalize_scan(
                ScanCompletion {
                    is_preflight_complete: true,
                    is_traversal_complete: true,
                },
                Vec::new(),
                right,
            ),
        );
    }

    #[test]
    fn named_limits_all_fail_closed() {
        let names = [
            "path-infos",
            "nodes",
            "blobs",
            "blob-bytes",
            "total-bytes",
            "depth",
            "findings",
            "duplicates",
            "container-entries",
            "container-expanded-bytes",
            "container-depth",
            "path-bytes",
            "shebang-bytes",
        ];
        for name in names {
            let mut state = ScanAccumulator::new();
            state.limit(&policy(), "fixture", name);
            assert!(state.halted, "limit {name} must halt");
            assert_eq!(state.findings[0].code, "limit-exhausted");
            assert_eq!(state.findings[0].detail, name);
        }
    }

    #[tokio::test]
    async fn castore_scan_accepts_complete_signed_blob_without_host_fallback() {
        let fixture = scan_fixture();
        let bytes = b"payload";
        let digest = write_blob(fixture.blob.as_ref(), bytes).await;
        let node = Node::File {
            digest,
            size: u64::try_from(bytes.len()).unwrap(),
            executable: false,
        };
        fixture.pathinfo.put(signed_path_info(TEST_OWNER, node, &fixture.signing_key)).await.unwrap();
        let roots = vec![TEST_OWNER.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = BTreeMap::from([(TEST_OWNER.to_string(), expected_fact())]);
        let declared = BTreeMap::from([(TEST_OWNER.to_string(), BTreeSet::new())]);
        let scan = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &declared,
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &policy(),
        })
        .await
        .unwrap();
        assert!(scan.preflight_complete);
        assert!(scan.traversal_complete);
        assert!(scan.findings.is_empty());
        assert_eq!(scan.payloads.len(), 1);
        assert_eq!(scan.payloads[0].content_blake3, blake3::hash(bytes).to_hex().to_string());
    }

    #[tokio::test]
    async fn castore_scan_rejects_untrusted_and_receipt_inconsistent_pathinfo_before_blob_reads() {
        let fixture = scan_fixture();
        let bytes = b"payload";
        let digest = write_blob(fixture.blob.as_ref(), bytes).await;
        let node = Node::File {
            digest,
            size: u64::try_from(bytes.len()).unwrap(),
            executable: false,
        };
        fixture.pathinfo.put(signed_path_info(TEST_OWNER, node, &fixture.signing_key)).await.unwrap();
        let roots = vec![TEST_OWNER.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = BTreeMap::from([(TEST_OWNER.to_string(), ProvenanceExpectedPathInfo {
            nar_sha256: "0".repeat(DIGEST_HEX_CHARS),
            nar_size: 1,
        })]);
        let other_verifying =
            VerifyingKey::parse("other-audit-test-1:tLAEn+EeaBUJYqEpTd2yeerr7Ic6+0vWe+aXL/vYUpE=").unwrap();
        let untrusted = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&other_verifying),
            policy: &policy(),
        })
        .await
        .unwrap();
        assert!(!untrusted.preflight_complete);
        assert_eq!(untrusted.findings[0].code, "unsigned-or-untrusted-pathinfo");
        assert_eq!(untrusted.read_byte_count, 0);

        let inconsistent = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &policy(),
        })
        .await
        .unwrap();
        assert!(!inconsistent.preflight_complete);
        assert_eq!(inconsistent.findings[0].code, "inconsistent-pathinfo");
        assert_eq!(inconsistent.read_byte_count, 0);
    }

    #[tokio::test]
    async fn castore_scan_reports_missing_blob_before_claiming_complete_traversal() {
        let fixture = scan_fixture();
        let missing_bytes = b"missing";
        let missing_digest = B3Digest::from(blake3::hash(missing_bytes).as_bytes());
        let node = Node::File {
            digest: missing_digest,
            size: u64::try_from(missing_bytes.len()).unwrap(),
            executable: false,
        };
        fixture.pathinfo.put(signed_path_info(TEST_OWNER, node, &fixture.signing_key)).await.unwrap();
        let roots = vec![TEST_OWNER.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = BTreeMap::from([(TEST_OWNER.to_string(), expected_fact())]);
        let scan = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &policy(),
        })
        .await
        .unwrap();
        assert!(scan.preflight_complete);
        assert!(scan.findings.iter().any(|finding| finding.code == "incomplete-closure"));
        assert!(!scan.traversal_complete);
        assert!(scan.payloads.is_empty());
    }

    #[tokio::test]
    async fn castore_directory_scan_detects_symlink_loop_without_following_links() {
        let fixture = scan_fixture();
        let mut directory = Directory::new();
        directory
            .add(PathComponent::try_from("a").unwrap(), Node::Symlink {
                target: SymlinkTarget::try_from("b").unwrap(),
            })
            .unwrap();
        directory
            .add(PathComponent::try_from("b").unwrap(), Node::Symlink {
                target: SymlinkTarget::try_from("a").unwrap(),
            })
            .unwrap();
        let digest = fixture.directory.put(directory.clone()).await.unwrap();
        let node = Node::Directory {
            digest,
            size: directory.size(),
        };
        fixture.pathinfo.put(signed_path_info(TEST_OWNER, node, &fixture.signing_key)).await.unwrap();
        let roots = vec![TEST_OWNER.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = BTreeMap::from([(TEST_OWNER.to_string(), expected_fact())]);
        let scan = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &policy(),
        })
        .await
        .unwrap();
        assert!(scan.preflight_complete);
        assert!(scan.findings.iter().any(|finding| finding.code == "symlink-loop"));
        assert_eq!(scan.visited_node_count, EXPECTED_SYMLINK_TREE_NODES);
    }

    #[tokio::test]
    async fn castore_scan_counts_duplicate_nodes_and_enforces_duplicate_limit() {
        let fixture = scan_fixture();
        let bytes = b"same";
        let digest = write_blob(fixture.blob.as_ref(), bytes).await;
        let node = Node::File {
            digest,
            size: u64::try_from(bytes.len()).unwrap(),
            executable: false,
        };
        let second = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-second";
        fixture
            .pathinfo
            .put(signed_path_info(TEST_OWNER, node.clone(), &fixture.signing_key))
            .await
            .unwrap();
        fixture.pathinfo.put(signed_path_info(second, node, &fixture.signing_key)).await.unwrap();
        let roots = vec![TEST_OWNER.to_string(), second.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = BTreeMap::from([
            (TEST_OWNER.to_string(), expected_fact()),
            (second.to_string(), expected_fact()),
        ]);
        let mut bounded = policy();
        bounded.max_duplicates = 1;
        let scan = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &bounded,
        })
        .await
        .unwrap();
        assert_eq!(scan.duplicate_node_count, 1);
        assert!(scan.findings.is_empty());

        bounded.max_duplicates = 1;
        let third = "/mantle/store/cccccccccccccccccccccccccccccccc-third";
        let parsed = fixture
            .pathinfo
            .get(
                *StorePath::<String>::from_absolute_path_with_prefix(TEST_OWNER.as_bytes(), TEST_STORE_PREFIX)
                    .unwrap()
                    .digest(),
            )
            .await
            .unwrap()
            .unwrap();
        fixture
            .pathinfo
            .put(PathInfo {
                store_path: StorePath::from_absolute_path_with_prefix(third.as_bytes(), TEST_STORE_PREFIX).unwrap(),
                ..signed_path_info(third, parsed.node, &fixture.signing_key)
            })
            .await
            .unwrap();
        let roots = vec![TEST_OWNER.to_string(), second.to_string(), third.to_string()];
        let admitted = roots.iter().cloned().collect();
        let expected = roots.iter().cloned().map(|path| (path, expected_fact())).collect();
        let scan = scan_castore_provenance(&fixture.handle, CastoreProvenanceRequest {
            selected_root_paths: &roots,
            admitted_closure_paths: &admitted,
            expected_path_infos: &expected,
            declared_references_by_output: &BTreeMap::new(),
            foreign_to_target_paths: &BTreeMap::new(),
            trusted_keys: std::slice::from_ref(&fixture.verifying_key),
            policy: &bounded,
        })
        .await
        .unwrap();
        assert!(scan.findings.iter().any(|finding| finding.detail == "duplicates"));
        assert!(!scan.traversal_complete);
    }

    #[test]
    fn byte_reference_admission_requires_a_valid_store_path_digest() {
        assert!(is_well_formed_store_reference(StoreReferenceInput {
            raw: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-valid",
            prefix: "/nix/store",
        }));
        assert!(!is_well_formed_store_reference(StoreReferenceInput {
            raw: "/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-placeholder",
            prefix: "/nix/store",
        }));
    }

    #[test]
    fn identity_shape_is_bounded_and_stable() {
        let identity = format!("container-entry:{}", blake3::hash(b"payload").to_hex());
        assert_eq!(identity.len(), "container-entry:".len() + DIGEST_HEX_CHARS);
        const EXPECTED_ALIGNED_CPIO_HEADER_BYTES: usize = 112;
        assert_eq!(
            align_up(AlignmentInput {
                value_bytes: CPIO_HEADER_BYTES,
                alignment_bytes: CPIO_ALIGNMENT_BYTES,
            }),
            Some(EXPECTED_ALIGNED_CPIO_HEADER_BYTES),
        );
        assert_eq!(STORE_PATH_SEPARATOR, b'/');
    }
}
