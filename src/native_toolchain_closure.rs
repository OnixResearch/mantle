use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::RunError;
use crate::source_toolchain_closure::NATIVE_HOST_CC_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_CRT1_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LIBC_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LIBGCC_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LIBUNWIND_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LINKER_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_SYSROOT_NAME;
use crate::source_toolchain_closure::NATIVE_RUSTC_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_AR_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_CRT1_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_GCC_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_GXX_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_LD_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_LIBC_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_LIBGCC_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_LIBUNWIND_NAME;
use crate::source_toolchain_closure::NATIVE_TARGET_RANLIB_NAME;
use crate::source_toolchain_closure::NativeClosureCandidateMember;
use crate::source_toolchain_closure::ToolchainBuildReceiptIdentity;
use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
use crate::source_toolchain_closure::ToolchainRole;
use crate::source_toolchain_closure::ToolchainSourceIdentity;
use crate::source_toolchain_closure::ToolchainSourceKind;

const SOURCE_ROOT_PROVIDER_METADATA_PATH: &str = "share/crunch-bootstrap/provider.json";
const NATIVE_PROVIDER_METADATA_PATH: &str = "share/mantle-native-toolchain/provider.json";
const NATIVE_PROVIDER_SCHEMA: &str = "mantle-native-toolchain-provider-v1";
const NATIVE_PROVIDER_ID: &str = "mantle-native-toolchain";
const SOURCE_ROOT_PROVIDER_ID: &str = "source-root-v1";
const TARGET_TRIPLE: &str = "x86_64-linux-musl";
const SOURCE_ROOT_GCC_LIB_DIR: &str = "lib/gcc";
const SOURCE_ROOT_LIBUNWIND_ARCHIVE: &str = "libgcc_eh.a";
const NATIVE_PROVIDER_LIBUNWIND_ARCHIVE: &str = "libunwind.a";
const HOST_ROOT_LABEL: &str = "host-root";
const TARGET_ROOT_LABEL: &str = "target-root";
const HOST_NATIVE_CAPABILITY: &str = "host-native";
const TARGET_NATIVE_CAPABILITY: &str = "target-native";
const UNKNOWN_VENDOR_TARGET_SEGMENT: &str = "-unknown-linux-";
const VENDORLESS_LINUX_TARGET_SEGMENT: &str = "-linux-";
const DIRECTORY_ENTRY_KIND_FILE: &str = "file";
const DIRECTORY_ENTRY_KIND_DIR: &str = "dir";
const DIRECTORY_ENTRY_KIND_SYMLINK: &str = "symlink";
const DIRECTORY_ENTRY_KIND_OTHER: &str = "other";
const DIRECTORY_DIGEST_CONTEXT: &str = "mantle-native-toolchain-directory-digest-v1";
const DIRECTORY_ENTRY_LIMIT: usize = 1_000_000;

pub(crate) struct NativeToolchainClosureOptions<'a> {
    pub(crate) rust_source_provider: &'a Path,
    pub(crate) host_root: &'a Path,
    pub(crate) target_root: &'a Path,
    pub(crate) output: &'a Path,
}

struct ProviderIdentity {
    source: ToolchainSourceIdentity,
    receipt: ToolchainBuildReceiptIdentity,
}

struct RustProviderIdentity {
    identity: ProviderIdentity,
    host_triple: String,
    target_triple: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeRootRole {
    Host,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeRootLayout {
    NativeProvider,
    SourceRootMusl,
}

struct NativeRootIdentity {
    identity: ProviderIdentity,
    layout: NativeRootLayout,
}

#[derive(Debug, Deserialize)]
struct ProviderMetadataSummary {
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    provider_id: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    host_triple: Option<String>,
    #[serde(default)]
    target_triple: Option<String>,
    #[serde(default)]
    source_built: Option<bool>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    provenance: Option<ProviderMetadataProvenance>,
}

#[derive(Debug, Deserialize)]
struct ProviderMetadataProvenance {
    #[serde(default)]
    source_built: Option<bool>,
}

pub(crate) fn cmd_materialize_native_toolchain_closure(
    options: NativeToolchainClosureOptions<'_>,
) -> Result<(), RunError> {
    if options.output.exists() {
        return Err(RunError::Build(format!(
            "source-built native toolchain closure blocked: output {} already exists",
            options.output.display()
        )));
    }

    let rust_identity = rust_provider_identity(options.rust_source_provider)?;
    let host_identity =
        native_root_identity(options.host_root, HOST_ROOT_LABEL, NativeRootRole::Host, &rust_identity.host_triple)?;
    let target_identity = native_root_identity(
        options.target_root,
        TARGET_ROOT_LABEL,
        NativeRootRole::Target,
        &rust_identity.target_triple,
    )?;
    let candidates = collect_native_closure_candidates(
        options.rust_source_provider,
        options.host_root,
        options.target_root,
        &rust_identity.identity,
        &host_identity,
        &target_identity,
    )?;
    let materialized = crate::source_toolchain_closure::materialize_source_built_native_closure(&candidates)
        .map_err(|err| RunError::Build(format!("source-built native toolchain closure blocked: {}", err.message())))?;
    if !materialized.manifest.seed_exceptions.is_empty() {
        return Err(RunError::Internal("native materializer produced seed exceptions".to_string()));
    }
    if materialized.validation.seed_exception_count != 0 {
        return Err(RunError::Internal("native materializer validation counted seed exceptions".to_string()));
    }
    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    let bytes = serde_json::to_vec_pretty(&materialized.manifest)
        .map_err(|err| RunError::Internal(format!("serializing native toolchain closure: {err}")))?;
    fs::write(options.output, bytes)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", options.output.display())))?;
    eprintln!("Materialized source-built native toolchain closure {}", options.output.display());
    eprintln!("  members: {}", materialized.validation.member_count);
    eprintln!("  policy_digest_blake3: {}", materialized.validation.policy_digest_blake3);
    Ok(())
}

fn rust_provider_identity(provider_root: &Path) -> Result<RustProviderIdentity, RunError> {
    let validation =
        crate::rust_source_provider::validate_materialized_rust_source_provider(provider_root).map_err(|err| {
            RunError::Build(format!(
                "source-built native toolchain closure blocked: invalid Rust provider {}: {err}",
                provider_root.display()
            ))
        })?;
    let digest = validation.validation.policy_digest_blake3;
    Ok(RustProviderIdentity {
        identity: ProviderIdentity {
            source: ToolchainSourceIdentity {
                kind: ToolchainSourceKind::LocalTree,
                name: "mantle-rust-source-provider".to_string(),
                digest_blake3: digest.clone(),
            },
            receipt: ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::MantleRustTopology,
                name: "mantle-rust-source-provider-receipt".to_string(),
                digest_blake3: digest,
            },
        },
        host_triple: validation.metadata.host_triple,
        target_triple: validation.metadata.target_triple,
    })
}

fn native_root_identity(
    root: &Path,
    root_label: &str,
    role: NativeRootRole,
    expected_triple: &str,
) -> Result<NativeRootIdentity, RunError> {
    let metadata_path = provider_metadata_path(root).ok_or_else(|| {
        RunError::Build(format!(
            "source-built native toolchain closure blocked: missing {root_label} provider metadata under {}",
            root.display()
        ))
    })?;
    let bytes = fs::read(&metadata_path)
        .map_err(|err| RunError::Build(format!("reading provider metadata {}: {err}", metadata_path.display())))?;
    let metadata: ProviderMetadataSummary = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Build(format!("parsing provider metadata {}: {err}", metadata_path.display())))?;
    let layout = classify_native_root_capability(&metadata, role, expected_triple).map_err(|err| {
        RunError::Build(format!(
            "source-built native toolchain closure blocked: {root_label} capability mismatch under {}: {err}",
            root.display()
        ))
    })?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok(NativeRootIdentity {
        identity: ProviderIdentity {
            source: ToolchainSourceIdentity {
                kind: ToolchainSourceKind::LocalTree,
                name: "mantle-source-built-native-root".to_string(),
                digest_blake3: digest.clone(),
            },
            receipt: ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
                name: "mantle-source-built-native-root-receipt".to_string(),
                digest_blake3: digest,
            },
        },
        layout,
    })
}

fn validate_native_root_capability(
    metadata: &ProviderMetadataSummary,
    role: NativeRootRole,
    expected_triple: &str,
) -> Result<(), String> {
    classify_native_root_capability(metadata, role, expected_triple).map(|_layout| ())
}

fn classify_native_root_capability(
    metadata: &ProviderMetadataSummary,
    role: NativeRootRole,
    expected_triple: &str,
) -> Result<NativeRootLayout, String> {
    if expected_triple.trim().is_empty() {
        return Err("expected triple is empty".to_string());
    }
    if is_source_root_provider(metadata) {
        return validate_source_root_capability(metadata, role, expected_triple);
    }
    if is_native_provider(metadata) {
        return validate_native_provider_capability(metadata, role, expected_triple);
    }
    Err(format!(
        "unsupported provider metadata provider_id={:?} schema={:?}",
        metadata.provider_id, metadata.schema
    ))
}

fn validate_source_root_capability(
    metadata: &ProviderMetadataSummary,
    role: NativeRootRole,
    expected_triple: &str,
) -> Result<NativeRootLayout, String> {
    let target = metadata.target.as_deref().ok_or_else(|| "source-root metadata missing target".to_string())?;
    if target_matches_expected(target, expected_triple) {
        return Ok(NativeRootLayout::SourceRootMusl);
    }
    if role == NativeRootRole::Host {
        return Err(format!("source-root target `{target}` cannot satisfy host-root for `{expected_triple}`"));
    }
    Err(format!("source-root target `{target}` does not match requested target `{expected_triple}`"))
}

fn validate_native_provider_capability(
    metadata: &ProviderMetadataSummary,
    role: NativeRootRole,
    expected_triple: &str,
) -> Result<NativeRootLayout, String> {
    if !metadata_is_source_built(metadata) {
        return Err("native provider metadata does not claim source_built=true".to_string());
    }
    let required_capability = required_native_capability(role);
    if !metadata.capabilities.iter().any(|capability| capability == required_capability) {
        return Err(format!("native provider metadata missing capability `{required_capability}`"));
    }
    if metadata_triple_matches(metadata, role, expected_triple) {
        return Ok(NativeRootLayout::NativeProvider);
    }
    Err(format!(
        "native provider metadata triples {:?} do not match requested {} triple `{expected_triple}`",
        advertised_triples(metadata),
        native_root_role_label(role)
    ))
}

fn is_source_root_provider(metadata: &ProviderMetadataSummary) -> bool {
    metadata.provider_id.as_deref() == Some(SOURCE_ROOT_PROVIDER_ID)
}

fn is_native_provider(metadata: &ProviderMetadataSummary) -> bool {
    metadata.provider_id.as_deref() == Some(NATIVE_PROVIDER_ID)
        || metadata.schema.as_deref() == Some(NATIVE_PROVIDER_SCHEMA)
}

fn metadata_is_source_built(metadata: &ProviderMetadataSummary) -> bool {
    metadata.source_built == Some(true)
        || metadata.provenance.as_ref().and_then(|provenance| provenance.source_built) == Some(true)
}

fn required_native_capability(role: NativeRootRole) -> &'static str {
    match role {
        NativeRootRole::Host => HOST_NATIVE_CAPABILITY,
        NativeRootRole::Target => TARGET_NATIVE_CAPABILITY,
    }
}

fn native_root_role_label(role: NativeRootRole) -> &'static str {
    match role {
        NativeRootRole::Host => "host",
        NativeRootRole::Target => "target",
    }
}

fn metadata_triple_matches(metadata: &ProviderMetadataSummary, role: NativeRootRole, expected_triple: &str) -> bool {
    advertised_triples(metadata)
        .into_iter()
        .any(|triple| triple_matches_role(&triple, role, expected_triple))
}

fn advertised_triples(metadata: &ProviderMetadataSummary) -> Vec<String> {
    let mut triples = Vec::new();
    for candidate in [&metadata.host_triple, &metadata.target_triple, &metadata.target] {
        if let Some(triple) = candidate.as_deref() {
            if !triple.trim().is_empty() && !triples.iter().any(|existing| existing == triple) {
                triples.push(triple.to_string());
            }
        }
    }
    triples
}

fn triple_matches_role(candidate: &str, role: NativeRootRole, expected_triple: &str) -> bool {
    match role {
        NativeRootRole::Host => candidate == expected_triple,
        NativeRootRole::Target => target_matches_expected(candidate, expected_triple),
    }
}

fn target_matches_expected(candidate: &str, expected_triple: &str) -> bool {
    accepted_target_triples(expected_triple).into_iter().any(|accepted| candidate == accepted)
}

fn accepted_target_triples(expected_triple: &str) -> Vec<String> {
    let trimmed = expected_triple.trim();
    let mut triples = vec![trimmed.to_string()];
    if trimmed.contains(UNKNOWN_VENDOR_TARGET_SEGMENT) {
        let vendorless = trimmed.replace(UNKNOWN_VENDOR_TARGET_SEGMENT, VENDORLESS_LINUX_TARGET_SEGMENT);
        if !triples.iter().any(|existing| existing == &vendorless) {
            triples.push(vendorless);
        }
    }
    triples
}

fn provider_metadata_path(root: &Path) -> Option<PathBuf> {
    let native = root.join(NATIVE_PROVIDER_METADATA_PATH);
    if native.is_file() {
        return Some(native);
    }
    let source_root = root.join(SOURCE_ROOT_PROVIDER_METADATA_PATH);
    if source_root.is_file() {
        return Some(source_root);
    }
    None
}

struct HostMemberPaths {
    c_compiler: PathBuf,
    linker: PathBuf,
    crt1: PathBuf,
    libgcc_s: PathBuf,
    libunwind: PathBuf,
    libc: PathBuf,
}

fn host_member_paths(root: &Path, layout: NativeRootLayout) -> Result<HostMemberPaths, RunError> {
    match layout {
        NativeRootLayout::NativeProvider => Ok(HostMemberPaths {
            c_compiler: root.join("bin").join("cc"),
            linker: root.join("bin").join("ld"),
            crt1: root.join("lib").join("crt1.o"),
            libgcc_s: root.join("lib").join("libgcc_s.so.1"),
            libunwind: root.join("lib").join(NATIVE_PROVIDER_LIBUNWIND_ARCHIVE),
            libc: root.join("lib").join("libc.so"),
        }),
        NativeRootLayout::SourceRootMusl => {
            let target_lib = root.join(TARGET_TRIPLE).join("lib");
            Ok(HostMemberPaths {
                c_compiler: root.join("bin").join(NATIVE_TARGET_GCC_NAME),
                linker: root.join("bin").join(NATIVE_TARGET_LD_NAME),
                crt1: target_lib.join("crt1.o"),
                libgcc_s: target_lib.join("libgcc_s.so.1"),
                libunwind: source_root_gcc_runtime_archive(root, SOURCE_ROOT_LIBUNWIND_ARCHIVE)?,
                libc: target_lib.join("libc.so"),
            })
        }
    }
}

fn source_root_gcc_runtime_archive(root: &Path, archive_name: &str) -> Result<PathBuf, RunError> {
    if archive_name.trim().is_empty() {
        return Err(RunError::Internal("source-root GCC archive name is empty".to_string()));
    }
    let gcc_root = root.join(SOURCE_ROOT_GCC_LIB_DIR).join(TARGET_TRIPLE);
    let entries = fs::read_dir(&gcc_root)
        .map_err(|err| RunError::Build(format!("read source-root GCC runtime dir {}: {err}", gcc_root.display())))?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            RunError::Build(format!("read source-root GCC runtime entry under {}: {err}", gcc_root.display()))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            RunError::Build(format!("stat source-root GCC runtime entry {}: {err}", entry.path().display()))
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let candidate = entry.path().join(archive_name);
        if candidate.is_file() {
            candidates.push(candidate);
        }
    }
    candidates.sort();
    match candidates.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => Err(RunError::Build(format!(
            "source-built native toolchain closure blocked: missing source-root GCC runtime archive {archive_name} under {}",
            gcc_root.display()
        ))),
        _many => Err(RunError::Build(format!(
            "source-built native toolchain closure blocked: multiple source-root GCC runtime archives named {archive_name} under {}",
            gcc_root.display()
        ))),
    }
}

fn target_libunwind_path(root: &Path, layout: NativeRootLayout) -> Result<PathBuf, RunError> {
    match layout {
        NativeRootLayout::NativeProvider => {
            Ok(root.join(TARGET_TRIPLE).join("lib").join(NATIVE_PROVIDER_LIBUNWIND_ARCHIVE))
        }
        NativeRootLayout::SourceRootMusl => source_root_gcc_runtime_archive(root, SOURCE_ROOT_LIBUNWIND_ARCHIVE),
    }
}

fn collect_native_closure_candidates(
    rust_provider: &Path,
    host_root: &Path,
    target_root: &Path,
    rust_identity: &ProviderIdentity,
    host_identity: &NativeRootIdentity,
    target_identity: &NativeRootIdentity,
) -> Result<Vec<NativeClosureCandidateMember>, RunError> {
    let mut missing = Vec::new();
    let mut candidates = Vec::new();
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::Rustc,
        NATIVE_RUSTC_NAME,
        &rust_provider.join("bin").join("rustc"),
        rust_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::Sysroot,
        NATIVE_HOST_SYSROOT_NAME,
        rust_provider,
        rust_identity,
    )?;
    let host_paths = host_member_paths(host_root, host_identity.layout)?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CCompiler,
        NATIVE_HOST_CC_NAME,
        &host_paths.c_compiler,
        &host_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::Linker,
        NATIVE_HOST_LINKER_NAME,
        &host_paths.linker,
        &host_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CrtObject,
        NATIVE_HOST_CRT1_NAME,
        &host_paths.crt1,
        &host_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_HOST_LIBGCC_NAME,
        &host_paths.libgcc_s,
        &host_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_HOST_LIBUNWIND_NAME,
        &host_paths.libunwind,
        &host_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_HOST_LIBC_NAME,
        &host_paths.libc,
        &host_identity.identity,
    )?;
    for (name, file_name) in [
        (NATIVE_TARGET_GCC_NAME, NATIVE_TARGET_GCC_NAME),
        (NATIVE_TARGET_GXX_NAME, NATIVE_TARGET_GXX_NAME),
        (NATIVE_TARGET_LD_NAME, NATIVE_TARGET_LD_NAME),
        (NATIVE_TARGET_AR_NAME, NATIVE_TARGET_AR_NAME),
        (NATIVE_TARGET_RANLIB_NAME, NATIVE_TARGET_RANLIB_NAME),
    ] {
        push_candidate(
            &mut candidates,
            &mut missing,
            ToolchainRole::NativeHelper,
            name,
            &target_root.join("bin").join(file_name),
            &target_identity.identity,
        )?;
    }
    let target_lib = target_root.join(TARGET_TRIPLE).join("lib");
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CrtObject,
        NATIVE_TARGET_CRT1_NAME,
        &target_lib.join("crt1.o"),
        &target_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_TARGET_LIBGCC_NAME,
        &target_lib.join("libgcc_s.so.1"),
        &target_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_TARGET_LIBUNWIND_NAME,
        &target_libunwind_path(target_root, target_identity.layout)?,
        &target_identity.identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_TARGET_LIBC_NAME,
        &target_lib.join("libc.so"),
        &target_identity.identity,
    )?;
    if !missing.is_empty() {
        return Err(RunError::Build(format!(
            "source-built native toolchain closure blocked: missing native closure members: {}",
            missing.join(", ")
        )));
    }
    Ok(candidates)
}

fn push_candidate(
    candidates: &mut Vec<NativeClosureCandidateMember>,
    missing: &mut Vec<String>,
    role: ToolchainRole,
    name: &str,
    path: &Path,
    identity: &ProviderIdentity,
) -> Result<(), RunError> {
    if !path.exists() {
        missing.push(name.to_string());
        return Ok(());
    }
    let execution_path = canonical_path(path)?;
    let digest = path_blake3(&execution_path)?;
    candidates.push(NativeClosureCandidateMember {
        role,
        name: name.to_string(),
        execution_path: execution_path.display().to_string(),
        content_digest_blake3: digest,
        source: identity.source.clone(),
        build_receipt: identity.receipt.clone(),
    });
    Ok(())
}

fn canonical_path(path: &Path) -> Result<PathBuf, RunError> {
    fs::canonicalize(path).map_err(|err| RunError::Build(format!("canonicalize {}: {err}", path.display())))
}

fn path_blake3(path: &Path) -> Result<String, RunError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|err| RunError::Build(format!("stat native closure path {}: {err}", path.display())))?;
    if metadata.is_dir() {
        return directory_blake3(path);
    }
    file_blake3(path)
}

fn file_blake3(path: &Path) -> Result<String, RunError> {
    let bytes = fs::read(path).map_err(|err| RunError::Build(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn directory_blake3(root: &Path) -> Result<String, RunError> {
    let mut entries = Vec::new();
    collect_directory_digest_entries(root, root, &mut entries)?;
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(DIRECTORY_DIGEST_CONTEXT.as_bytes());
    for entry in entries {
        hasher.update(entry.as_bytes());
        hasher.update(b"\n");
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_directory_digest_entries(root: &Path, path: &Path, entries: &mut Vec<String>) -> Result<(), RunError> {
    if entries.len() > DIRECTORY_ENTRY_LIMIT {
        return Err(RunError::Build(format!(
            "native closure directory digest exceeded {DIRECTORY_ENTRY_LIMIT} entries under {}",
            root.display()
        )));
    }
    let mut children = fs::read_dir(path)
        .map_err(|err| RunError::Build(format!("read directory {}: {err}", path.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| RunError::Build(format!("read directory entry under {}: {err}", path.display())))?;
    children.sort_by_key(|entry| entry.path());
    for child in children {
        let child_path = child.path();
        let relative = child_path.strip_prefix(root).map_err(|err| {
            RunError::Internal(format!("strip {} from {}: {err}", root.display(), child_path.display()))
        })?;
        let relative = relative.display().to_string();
        let metadata = fs::symlink_metadata(&child_path)
            .map_err(|err| RunError::Build(format!("stat {}: {err}", child_path.display())))?;
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            entries.push(format!("{DIRECTORY_ENTRY_KIND_DIR}:{relative}"));
            collect_directory_digest_entries(root, &child_path, entries)?;
        } else if file_type.is_file() {
            entries.push(format!("{DIRECTORY_ENTRY_KIND_FILE}:{relative}:{}", file_blake3(&child_path)?));
        } else if file_type.is_symlink() {
            let target = fs::read_link(&child_path)
                .map_err(|err| RunError::Build(format!("read symlink {}: {err}", child_path.display())))?;
            entries.push(format!("{DIRECTORY_ENTRY_KIND_SYMLINK}:{relative}:{}", target.display()));
        } else {
            entries.push(format!("{DIRECTORY_ENTRY_KIND_OTHER}:{relative}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata_path_prefers_native_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let native = dir.path().join(NATIVE_PROVIDER_METADATA_PATH);
        let source_root = dir.path().join(SOURCE_ROOT_PROVIDER_METADATA_PATH);
        fs::create_dir_all(native.parent().unwrap()).unwrap();
        fs::create_dir_all(source_root.parent().unwrap()).unwrap();
        fs::write(&native, b"native").unwrap();
        fs::write(&source_root, b"source-root").unwrap();

        let selected = provider_metadata_path(dir.path()).unwrap();

        assert_eq!(selected, native);
    }

    #[test]
    fn source_root_metadata_is_rejected_for_host_root() {
        let metadata = source_root_metadata(TARGET_TRIPLE);

        let err =
            validate_native_root_capability(&metadata, NativeRootRole::Host, "x86_64-unknown-linux-gnu").unwrap_err();

        assert!(err.contains("cannot satisfy host-root"));
        assert!(err.contains(TARGET_TRIPLE));
    }

    #[test]
    fn source_root_metadata_is_accepted_for_vendorless_target_root() {
        let metadata = source_root_metadata(TARGET_TRIPLE);

        validate_native_root_capability(&metadata, NativeRootRole::Target, "x86_64-unknown-linux-musl").unwrap();
    }

    #[test]
    fn source_root_metadata_is_accepted_for_matching_musl_host_root() {
        let metadata = source_root_metadata(TARGET_TRIPLE);

        let layout =
            classify_native_root_capability(&metadata, NativeRootRole::Host, "x86_64-unknown-linux-musl").unwrap();

        assert_eq!(layout, NativeRootLayout::SourceRootMusl);
    }

    #[test]
    fn source_root_host_layout_uses_target_prefixed_tools_and_target_libs() {
        let dir = tempfile::tempdir().unwrap();

        let gcc_runtime = dir.path().join(SOURCE_ROOT_GCC_LIB_DIR).join(TARGET_TRIPLE).join("10.5.0");
        fs::create_dir_all(&gcc_runtime).unwrap();
        fs::write(gcc_runtime.join(SOURCE_ROOT_LIBUNWIND_ARCHIVE), b"unwind").unwrap();
        let paths = host_member_paths(dir.path(), NativeRootLayout::SourceRootMusl).unwrap();

        assert!(paths.c_compiler.ends_with(format!("bin/{NATIVE_TARGET_GCC_NAME}")));
        assert!(paths.linker.ends_with(format!("bin/{NATIVE_TARGET_LD_NAME}")));
        assert!(paths.crt1.ends_with(format!("{TARGET_TRIPLE}/lib/crt1.o")));
        assert!(paths.libgcc_s.ends_with(format!("{TARGET_TRIPLE}/lib/libgcc_s.so.1")));
        assert!(
            paths
                .libunwind
                .ends_with(format!("{SOURCE_ROOT_GCC_LIB_DIR}/{TARGET_TRIPLE}/10.5.0/{SOURCE_ROOT_LIBUNWIND_ARCHIVE}"))
        );
        assert!(paths.libc.ends_with(format!("{TARGET_TRIPLE}/lib/libc.so")));
    }

    #[test]
    fn native_host_metadata_requires_host_capability() {
        let metadata = native_metadata("x86_64-unknown-linux-gnu", &[TARGET_NATIVE_CAPABILITY], true);

        let err =
            validate_native_root_capability(&metadata, NativeRootRole::Host, "x86_64-unknown-linux-gnu").unwrap_err();

        assert!(err.contains("missing capability"));
        assert!(err.contains(HOST_NATIVE_CAPABILITY));
    }

    #[test]
    fn native_host_metadata_accepts_matching_source_built_capability() {
        let metadata = native_metadata("x86_64-unknown-linux-gnu", &[HOST_NATIVE_CAPABILITY], true);

        validate_native_root_capability(&metadata, NativeRootRole::Host, "x86_64-unknown-linux-gnu").unwrap();
    }

    #[test]
    fn native_metadata_rejects_non_source_built_root() {
        let metadata = native_metadata("x86_64-unknown-linux-gnu", &[HOST_NATIVE_CAPABILITY], false);

        let err =
            validate_native_root_capability(&metadata, NativeRootRole::Host, "x86_64-unknown-linux-gnu").unwrap_err();

        assert!(err.contains("source_built=true"));
    }

    #[test]
    fn musl_host_source_root_fixture_materializes_zero_seed_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let rust_provider = dir.path().join("rust-provider");
        let source_root = dir.path().join("source-root-musl");
        write_fixture_file(&rust_provider.join("bin").join("rustc"), b"rustc");
        for tool in [
            NATIVE_TARGET_GCC_NAME,
            NATIVE_TARGET_GXX_NAME,
            NATIVE_TARGET_LD_NAME,
            NATIVE_TARGET_AR_NAME,
            NATIVE_TARGET_RANLIB_NAME,
        ] {
            write_fixture_file(&source_root.join("bin").join(tool), tool.as_bytes());
        }
        let target_lib = source_root.join(TARGET_TRIPLE).join("lib");
        write_fixture_file(&target_lib.join("crt1.o"), b"crt1");
        write_fixture_file(&target_lib.join("libgcc_s.so.1"), b"libgcc");
        write_fixture_file(&target_lib.join("libc.so"), b"libc");
        write_fixture_file(
            &source_root
                .join(SOURCE_ROOT_GCC_LIB_DIR)
                .join(TARGET_TRIPLE)
                .join("10.5.0")
                .join(SOURCE_ROOT_LIBUNWIND_ARCHIVE),
            b"unwind",
        );
        let rust_identity = fake_provider_identity("rust-provider");
        let source_root_identity = NativeRootIdentity {
            identity: fake_provider_identity("source-root-musl"),
            layout: NativeRootLayout::SourceRootMusl,
        };

        let candidates = collect_native_closure_candidates(
            &rust_provider,
            &source_root,
            &source_root,
            &rust_identity,
            &source_root_identity,
            &source_root_identity,
        )
        .unwrap();
        let materialized =
            crate::source_toolchain_closure::materialize_source_built_native_closure(&candidates).unwrap();

        assert!(materialized.manifest.seed_exceptions.is_empty());
        assert_eq!(materialized.validation.seed_exception_count, 0);
        assert_eq!(materialized.validation.source_built_member_count, materialized.validation.member_count);
        let cc = manifest_member(&materialized.manifest, NATIVE_HOST_CC_NAME);
        let ld = manifest_member(&materialized.manifest, NATIVE_HOST_LINKER_NAME);
        let libunwind = manifest_member(&materialized.manifest, NATIVE_HOST_LIBUNWIND_NAME);
        let libc = manifest_member(&materialized.manifest, NATIVE_HOST_LIBC_NAME);
        assert!(cc.execution_path.ends_with(&format!("bin/{NATIVE_TARGET_GCC_NAME}")));
        assert!(ld.execution_path.ends_with(&format!("bin/{NATIVE_TARGET_LD_NAME}")));
        assert!(
            libunwind.execution_path.ends_with(&format!(
                "{SOURCE_ROOT_GCC_LIB_DIR}/{TARGET_TRIPLE}/10.5.0/{SOURCE_ROOT_LIBUNWIND_ARCHIVE}"
            ))
        );
        assert!(libc.execution_path.ends_with(&format!("{TARGET_TRIPLE}/lib/libc.so")));
        assert_eq!(cc.trust, crate::source_toolchain_closure::ToolchainTrust::SourceBuilt);
        assert!(cc.source.is_some());
        assert!(cc.build_receipt.is_some());
    }

    #[test]
    fn directory_digest_changes_when_file_content_changes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tool");
        fs::write(&file, b"old").unwrap();
        let old_digest = directory_blake3(dir.path()).unwrap();
        fs::write(&file, b"new").unwrap();

        let new_digest = directory_blake3(dir.path()).unwrap();

        assert_ne!(old_digest, new_digest);
    }

    fn source_root_metadata(target: &str) -> ProviderMetadataSummary {
        ProviderMetadataSummary {
            schema: None,
            provider_id: Some(SOURCE_ROOT_PROVIDER_ID.to_string()),
            target: Some(target.to_string()),
            host_triple: None,
            target_triple: None,
            source_built: None,
            capabilities: Vec::new(),
            provenance: None,
        }
    }

    fn native_metadata(triple: &str, capabilities: &[&str], source_built: bool) -> ProviderMetadataSummary {
        ProviderMetadataSummary {
            schema: Some(NATIVE_PROVIDER_SCHEMA.to_string()),
            provider_id: Some(NATIVE_PROVIDER_ID.to_string()),
            target: None,
            host_triple: Some(triple.to_string()),
            target_triple: None,
            source_built: Some(source_built),
            capabilities: capabilities.iter().map(|capability| (*capability).to_string()).collect(),
            provenance: None,
        }
    }

    fn fake_provider_identity(name: &str) -> ProviderIdentity {
        ProviderIdentity {
            source: ToolchainSourceIdentity {
                kind: ToolchainSourceKind::Generated,
                name: format!("{name}-source"),
                digest_blake3: fake_digest(),
            },
            receipt: ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
                name: format!("{name}-receipt"),
                digest_blake3: fake_digest(),
            },
        }
    }

    fn manifest_member<'a>(
        manifest: &'a crate::source_toolchain_closure::ToolchainClosureManifest,
        name: &str,
    ) -> &'a crate::source_toolchain_closure::ToolchainClosureMember {
        manifest
            .members
            .iter()
            .find(|member| member.name == name)
            .unwrap_or_else(|| panic!("manifest member {name} missing"))
    }

    fn write_fixture_file(path: &Path, content: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn fake_digest() -> String {
        "abababababababababababababababababababababababababababababababab".to_string()
    }
}
