use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::source_toolchain_closure::NATIVE_HOST_CC_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_CRT1_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LIBC_NAME;
use crate::source_toolchain_closure::NATIVE_HOST_LIBGCC_NAME;
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
use crate::source_toolchain_closure::NATIVE_TARGET_RANLIB_NAME;
use crate::source_toolchain_closure::NativeClosureCandidateMember;
use crate::source_toolchain_closure::ToolchainBuildReceiptIdentity;
use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
use crate::source_toolchain_closure::ToolchainRole;
use crate::source_toolchain_closure::ToolchainSourceIdentity;
use crate::source_toolchain_closure::ToolchainSourceKind;

const SOURCE_ROOT_PROVIDER_METADATA_PATH: &str = "share/crunch-bootstrap/provider.json";
const NATIVE_PROVIDER_METADATA_PATH: &str = "share/mantle-native-toolchain/provider.json";
const TARGET_TRIPLE: &str = "x86_64-linux-musl";
const HOST_PROVIDER_METADATA_NAME: &str = "host-provider-metadata";
const TARGET_PROVIDER_METADATA_NAME: &str = "target-provider-metadata";
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
    let host_identity = native_root_identity(options.host_root, HOST_PROVIDER_METADATA_NAME)?;
    let target_identity = native_root_identity(options.target_root, TARGET_PROVIDER_METADATA_NAME)?;
    let candidates = collect_native_closure_candidates(
        options.rust_source_provider,
        options.host_root,
        options.target_root,
        &rust_identity,
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

fn rust_provider_identity(provider_root: &Path) -> Result<ProviderIdentity, RunError> {
    let validation =
        crate::rust_source_provider::validate_materialized_rust_source_provider(provider_root).map_err(|err| {
            RunError::Build(format!(
                "source-built native toolchain closure blocked: invalid Rust provider {}: {err}",
                provider_root.display()
            ))
        })?;
    let digest = validation.validation.policy_digest_blake3;
    Ok(ProviderIdentity {
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
    })
}

fn native_root_identity(root: &Path, missing_label: &str) -> Result<ProviderIdentity, RunError> {
    let metadata_path = provider_metadata_path(root).ok_or_else(|| {
        RunError::Build(format!(
            "source-built native toolchain closure blocked: missing {missing_label} under {}",
            root.display()
        ))
    })?;
    let digest = file_blake3(&metadata_path)?;
    Ok(ProviderIdentity {
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
    })
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

fn collect_native_closure_candidates(
    rust_provider: &Path,
    host_root: &Path,
    target_root: &Path,
    rust_identity: &ProviderIdentity,
    host_identity: &ProviderIdentity,
    target_identity: &ProviderIdentity,
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
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CCompiler,
        NATIVE_HOST_CC_NAME,
        &host_root.join("bin").join("cc"),
        host_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::Linker,
        NATIVE_HOST_LINKER_NAME,
        &host_root.join("bin").join("ld"),
        host_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CrtObject,
        NATIVE_HOST_CRT1_NAME,
        &host_root.join("lib").join("crt1.o"),
        host_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_HOST_LIBGCC_NAME,
        &host_root.join("lib").join("libgcc_s.so.1"),
        host_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_HOST_LIBC_NAME,
        &host_root.join("lib").join("libc.so"),
        host_identity,
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
            target_identity,
        )?;
    }
    let target_lib = target_root.join(TARGET_TRIPLE).join("lib");
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::CrtObject,
        NATIVE_TARGET_CRT1_NAME,
        &target_lib.join("crt1.o"),
        target_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_TARGET_LIBGCC_NAME,
        &target_lib.join("libgcc_s.so.1"),
        target_identity,
    )?;
    push_candidate(
        &mut candidates,
        &mut missing,
        ToolchainRole::RuntimeLibrary,
        NATIVE_TARGET_LIBC_NAME,
        &target_lib.join("libc.so"),
        target_identity,
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
    fn directory_digest_changes_when_file_content_changes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tool");
        fs::write(&file, b"old").unwrap();
        let old_digest = directory_blake3(dir.path()).unwrap();
        fs::write(&file, b"new").unwrap();

        let new_digest = directory_blake3(dir.path()).unwrap();

        assert_ne!(old_digest, new_digest);
    }
}
