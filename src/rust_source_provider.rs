//! Shell/orchestration for the source-built Rust compiler/sysroot provider.
//!
//! This module performs filesystem work around the pure provider metadata
//! validator in `source_toolchain_closure`. It deliberately fails closed for
//! materialization until Mantle has a real Rust-from-source bootstrap path; the
//! validation entry points are ready for a future materializer output and refuse
//! prebuilt/wrapper metadata in the pure core.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_METADATA_PATH;
use crate::source_toolchain_closure::RustProviderObservedArtifact;
use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
use crate::source_toolchain_closure::RustSourceProviderMetadata;
use crate::source_toolchain_closure::RustSourceProviderValidation;

pub(crate) const RUST_SOURCE_PROVIDER_BLOCKED_REASON: &str = "source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust";

const DIRECTORY_DIGEST_MAX_ENTRIES: usize = 200_000;

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderMaterialization {
    pub(crate) output_path: PathBuf,
    pub(crate) recipe_digest_blake3: String,
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderDirectoryValidation {
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) validation: RustSourceProviderValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderImport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) validation: RustSourceProviderDirectoryValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderMaterializationPlan {
    recipe_path: PathBuf,
    output_dir: PathBuf,
    scratch_dir: PathBuf,
    recipe_digest_blake3: String,
}

#[derive(Debug)]
pub(crate) enum RustSourceProviderError {
    Read(String),
    Parse(String),
    Validate(String),
    MissingArtifact(String),
    Digest(String),
    Copy(String),
    Blocked {
        recipe_path: PathBuf,
        recipe_digest_blake3: String,
        reason: &'static str,
    },
}

impl std::fmt::Display for RustSourceProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(message) => write!(formatter, "read: {message}"),
            Self::Parse(message) => write!(formatter, "parse: {message}"),
            Self::Validate(message) => write!(formatter, "validate: {message}"),
            Self::MissingArtifact(message) => write!(formatter, "missing artifact: {message}"),
            Self::Digest(message) => write!(formatter, "digest: {message}"),
            Self::Copy(message) => write!(formatter, "copy: {message}"),
            Self::Blocked {
                recipe_path,
                recipe_digest_blake3,
                reason,
            } => write!(
                formatter,
                "{reason}; recipe={} recipe_digest_blake3={recipe_digest_blake3}",
                recipe_path.display()
            ),
        }
    }
}

impl std::error::Error for RustSourceProviderError {}

pub(crate) fn materialize_rust_source_provider(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    let recipe_bytes = fs::read(recipe_path)
        .map_err(|err| RustSourceProviderError::Read(format!("recipe {}: {err}", recipe_path.display())))?;
    if recipe_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("recipe {} is empty", recipe_path.display())));
    }
    let recipe_digest_blake3 = blake3::hash(&recipe_bytes).to_hex().to_string();
    let plan = plan_materialization(recipe_path, output_dir, scratch_dir, &recipe_digest_blake3)?;
    if verbose {
        eprintln!("Rust source provider recipe: {}", plan.recipe_path.display());
        eprintln!("  recipe_digest_blake3: {}", plan.recipe_digest_blake3);
        eprintln!("  planned_output: {}", plan.output_dir.display());
        eprintln!("  planned_scratch: {}", plan.scratch_dir.display());
    }
    Err(RustSourceProviderError::Blocked {
        recipe_path: plan.recipe_path,
        recipe_digest_blake3: plan.recipe_digest_blake3,
        reason: RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    })
}

pub(crate) fn import_rust_source_provider(
    import_dir: &Path,
    output_dir: &Path,
) -> Result<RustSourceProviderImport, RustSourceProviderError> {
    validate_import_request(import_dir, output_dir)?;
    validate_materialized_rust_source_provider(import_dir)?;
    copy_provider_tree(import_dir, output_dir)?;
    let validation = match validate_materialized_rust_source_provider(output_dir) {
        Ok(validation) => validation,
        Err(err) => {
            let _ = remove_import_output(output_dir);
            return Err(err);
        }
    };
    Ok(RustSourceProviderImport {
        input_path: import_dir.to_path_buf(),
        output_path: output_dir.to_path_buf(),
        validation,
    })
}

pub(crate) fn validate_materialized_rust_source_provider(
    provider_dir: &Path,
) -> Result<RustSourceProviderDirectoryValidation, RustSourceProviderError> {
    let metadata_path = provider_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    let metadata_bytes = fs::read(&metadata_path).map_err(|err| {
        RustSourceProviderError::Read(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    if metadata_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("provider metadata {} is empty", metadata_path.display())));
    }
    let metadata_digest_blake3 = blake3::hash(&metadata_bytes).to_hex().to_string();
    let metadata = serde_json::from_slice::<RustSourceProviderMetadata>(&metadata_bytes).map_err(|err| {
        RustSourceProviderError::Parse(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    let observed = observed_provider_artifacts(provider_dir, &metadata)?;
    let observed_receipts = observed_provider_receipts(provider_dir, &metadata)?;
    crate::source_toolchain_closure::enforce_observed_rust_source_provider_artifacts(&metadata, &observed)
        .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    let validation =
        crate::source_toolchain_closure::enforce_observed_rust_source_provider_receipts(&metadata, &observed_receipts)
            .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    Ok(RustSourceProviderDirectoryValidation {
        metadata_path,
        metadata_digest_blake3,
        validation,
    })
}

fn validate_import_request(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if import_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("import dir is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if !import_dir.is_dir() {
        return Err(RustSourceProviderError::Read(format!("import dir {} is not a directory", import_dir.display())));
    }
    if output_dir.exists() {
        return Err(RustSourceProviderError::Copy(format!("output dir {} already exists", output_dir.display())));
    }
    Ok(())
}

fn plan_materialization(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    recipe_digest_blake3: &str,
) -> Result<RustSourceProviderMaterializationPlan, RustSourceProviderError> {
    if recipe_path.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("recipe path is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if scratch_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("scratch dir is empty".to_string()));
    }
    if recipe_digest_blake3.is_empty() {
        return Err(RustSourceProviderError::Digest("recipe digest is empty".to_string()));
    }
    Ok(RustSourceProviderMaterializationPlan {
        recipe_path: recipe_path.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        recipe_digest_blake3: recipe_digest_blake3.to_string(),
    })
}

fn copy_provider_tree(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    let parent = output_dir
        .parent()
        .ok_or_else(|| RustSourceProviderError::Copy(format!("{} has no parent", output_dir.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", parent.display())))?;
    fs::create_dir(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", output_dir.display())))?;
    let mut copied_entries = 0usize;
    if let Err(err) = copy_directory_contents(import_dir, output_dir, &mut copied_entries) {
        let _ = remove_import_output(output_dir);
        return Err(err);
    }
    Ok(())
}

fn copy_directory_contents(src: &Path, dst: &Path, copied_entries: &mut usize) -> Result<(), RustSourceProviderError> {
    for entry in
        fs::read_dir(src).map_err(|err| RustSourceProviderError::Copy(format!("read dir {}: {err}", src.display())))?
    {
        *copied_entries = copied_entries
            .checked_add(1)
            .ok_or_else(|| RustSourceProviderError::Copy("provider import entry count overflowed".to_string()))?;
        if *copied_entries > DIRECTORY_DIGEST_MAX_ENTRIES {
            return Err(RustSourceProviderError::Copy(format!(
                "provider import has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(|err| RustSourceProviderError::Copy(format!("read dir entry: {err}")))?;
        let source_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|err| RustSourceProviderError::Copy(format!("file type {}: {err}", source_path.display())))?;
        if file_type.is_dir() {
            fs::create_dir(&dest_path)
                .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", dest_path.display())))?;
            copy_directory_contents(&source_path, &dest_path, copied_entries)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &dest_path).map_err(|err| {
                RustSourceProviderError::Copy(format!(
                    "copy {} -> {}: {err}",
                    source_path.display(),
                    dest_path.display()
                ))
            })?;
        } else if file_type.is_symlink() {
            copy_symlink(&source_path, &dest_path)?;
        } else {
            return Err(RustSourceProviderError::Copy(format!("unsupported file type at {}", source_path.display())));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source_path: &Path, dest_path: &Path) -> Result<(), RustSourceProviderError> {
    let target = fs::read_link(source_path)
        .map_err(|err| RustSourceProviderError::Copy(format!("readlink {}: {err}", source_path.display())))?;
    std::os::unix::fs::symlink(&target, dest_path).map_err(|err| {
        RustSourceProviderError::Copy(format!("symlink {} -> {}: {err}", dest_path.display(), target.display()))
    })
}

#[cfg(not(unix))]
fn copy_symlink(source_path: &Path, _dest_path: &Path) -> Result<(), RustSourceProviderError> {
    Err(RustSourceProviderError::Copy(format!(
        "symlink import is unsupported on this platform at {}",
        source_path.display()
    )))
}

fn remove_import_output(output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if !output_dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", output_dir.display())))
}

fn observed_provider_artifacts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedArtifact>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.artifacts.len());
    for artifact in &metadata.artifacts {
        let path = provider_dir.join(&artifact.path);
        if !path.exists() {
            return Err(RustSourceProviderError::MissingArtifact(format!("{} at {}", artifact.path, path.display())));
        }
        observed.push(RustProviderObservedArtifact {
            role: artifact.role,
            path: artifact.path.clone(),
            content_digest_blake3: content_digest_blake3(&path)?,
        });
    }
    Ok(observed)
}

fn observed_provider_receipts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedBuildReceipt>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.build_receipts.len());
    for receipt in &metadata.build_receipts {
        let path = provider_dir.join(&receipt.path);
        let bytes = fs::read(&path)
            .map_err(|err| RustSourceProviderError::Read(format!("provider receipt {}: {err}", path.display())))?;
        if bytes.is_empty() {
            return Err(RustSourceProviderError::Read(format!("provider receipt {} is empty", path.display())));
        }
        let parsed = serde_json::from_slice::<RustSourceProviderBuildReceipt>(&bytes)
            .map_err(|err| RustSourceProviderError::Parse(format!("provider receipt {}: {err}", path.display())))?;
        observed.push(RustProviderObservedBuildReceipt {
            path: receipt.path.clone(),
            content_digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
            receipt: parsed,
        });
    }
    Ok(observed)
}

fn content_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    if path.is_file() {
        return file_digest_blake3(path);
    }
    if path.is_dir() {
        return directory_digest_blake3(path);
    }
    Err(RustSourceProviderError::MissingArtifact(format!(
        "{} is neither file nor directory",
        path.display()
    )))
}

fn file_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    let bytes =
        fs::read(path).map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn directory_digest_blake3(dir: &Path) -> Result<String, RustSourceProviderError> {
    let mut entries = Vec::new();
    collect_relative_paths(dir, dir, &mut entries)?;
    if entries.len() > DIRECTORY_DIGEST_MAX_ENTRIES {
        return Err(RustSourceProviderError::Digest(format!(
            "directory {} has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries",
            dir.display()
        )));
    }
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    for relative in &entries {
        let full = dir.join(relative);
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        if full.is_dir() {
            hasher.update(b"dir");
        } else if full.is_file() {
            hasher.update(b"file");
            let bytes = fs::read(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", full.display())))?;
            hasher.update(&bytes);
        } else if full.is_symlink() {
            hasher.update(b"symlink");
            let target = fs::read_link(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("readlink {}: {err}", full.display())))?;
            hasher.update(target.to_string_lossy().as_bytes());
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_relative_paths(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<(), RustSourceProviderError> {
    for entry in fs::read_dir(current)
        .map_err(|err| RustSourceProviderError::Digest(format!("read dir {}: {err}", current.display())))?
    {
        let entry = entry.map_err(|err| RustSourceProviderError::Digest(format!("read dir entry: {err}")))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|err| RustSourceProviderError::Digest(format!("strip prefix: {err}")))?
            .to_string_lossy()
            .to_string();
        out.push(relative);
        if path.is_dir() && !path.is_symlink() {
            collect_relative_paths(root, &path, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize test metadata: {err}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", parent.display())))?;
    fs::write(path, bytes).map_err(|err| RustSourceProviderError::Read(format!("write {}: {err}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA;
    use crate::source_toolchain_closure::RustProviderArtifact;
    use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
    use crate::source_toolchain_closure::RustProviderReceiptArtifact;
    use crate::source_toolchain_closure::RustProviderReceiptStep;
    use crate::source_toolchain_closure::RustProviderRole;
    use crate::source_toolchain_closure::RustProviderSourceIdentity;
    use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
    use crate::source_toolchain_closure::RustSourceProviderProvenance;
    use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
    use crate::source_toolchain_closure::ToolchainSourceKind;

    const HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
    const TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";

    #[test]
    fn materializer_fails_closed_without_claiming_prebuilt_rust() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let text = err.to_string();
        assert!(text.contains(RUST_SOURCE_PROVIDER_BLOCKED_REASON));
        assert!(text.contains("recipe_digest_blake3="));
        assert!(!text.contains("source-built claim"));
        assert!(!output.exists());
    }

    #[test]
    fn directory_validator_accepts_complete_fake_provider() {
        let dir = tempfile::tempdir().unwrap();
        let metadata = write_fake_provider(dir.path());

        let validation = validate_materialized_rust_source_provider(dir.path()).unwrap();

        assert_eq!(validation.metadata_path, dir.path().join(RUST_SOURCE_PROVIDER_METADATA_PATH));
        assert_eq!(validation.validation.artifact_count, metadata.artifacts.len());
        assert_eq!(validation.validation.source_count, metadata.sources.len());
        assert_eq!(validation.validation.receipt_count, metadata.build_receipts.len());
        assert!(!validation.metadata_digest_blake3.is_empty());
    }

    #[test]
    fn directory_validator_rejects_artifact_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        write_fake_provider(dir.path());
        fs::write(dir.path().join("bin/rustc"), b"changed").unwrap();

        let err = validate_materialized_rust_source_provider(dir.path()).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
    }

    #[test]
    fn import_provider_copies_validated_provider() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let metadata = write_fake_provider(&import_dir);

        let imported = import_rust_source_provider(&import_dir, &output_dir).unwrap();

        assert_eq!(imported.input_path, import_dir);
        assert_eq!(imported.output_path, output_dir);
        assert_eq!(imported.validation.validation.artifact_count, metadata.artifacts.len());
        assert!(imported.output_path.join("bin/rustc").is_file());
        assert!(imported.output_path.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
    }

    #[test]
    fn import_provider_rejects_prebuilt_metadata_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let mut metadata = write_fake_provider(&import_dir);
        metadata.provenance.uses_prebuilt_rust = true;
        write_json(&import_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("uses prebuilt Rust"));
        assert!(!output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_existing_output_without_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();
        write_fake_provider(&import_dir);

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("already exists"));
        assert!(output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_malformed_receipt_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        write_fake_provider(&import_dir);
        fs::write(import_dir.join("share/mantle-rust-provider/receipts/build.json"), b"not-json").unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("provider receipt"));
        assert!(!output_dir.exists());
    }

    fn write_fake_provider(root: &Path) -> RustSourceProviderMetadata {
        write_bytes(root, "bin/rustc", b"rustc");
        write_bytes(root, "bin/cargo", b"cargo");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib", b"host-std");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib", b"target-std");
        let receipt = fake_receipt(root);
        write_json(&root.join("share/mantle-rust-provider/receipts/build.json"), &receipt).unwrap();
        let metadata = fake_metadata(root);
        write_json(&root.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();
        metadata
    }

    fn fake_receipt(root: &Path) -> RustSourceProviderBuildReceipt {
        RustSourceProviderBuildReceipt {
            schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
            receipt_id: "build-receipt".to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            source_ids: vec!["rust-src".to_string()],
            output_artifacts: vec![
                receipt_artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                receipt_artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                receipt_artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                receipt_artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
            ],
            build_steps: vec![RustProviderReceiptStep {
                name: "compile-rust-from-source".to_string(),
                program: "mantle-rust-source-stage".to_string(),
                arguments: vec!["bootstrap/rust-source.ncl".to_string()],
            }],
        }
    }

    fn fake_metadata(root: &Path) -> RustSourceProviderMetadata {
        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: true,
                uses_prebuilt_rust: false,
                build_recipe: "bootstrap/rust-source.ncl".to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: "rust-src".to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: blake3::hash(b"source").to_hex().to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: "build-receipt".to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-source-build-receipt".to_string(),
                path: "share/mantle-rust-provider/receipts/build.json".to_string(),
                digest_blake3: digest_path(root, "share/mantle-rust-provider/receipts/build.json"),
            }],
            artifacts: vec![
                artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::ProviderReceipt,
                    "build-receipt",
                    "share/mantle-rust-provider/receipts/build.json",
                ),
            ],
        }
    }

    fn artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderArtifact {
        RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
            source_id: "rust-src".to_string(),
            build_receipt_id: "build-receipt".to_string(),
        }
    }

    fn receipt_artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderReceiptArtifact {
        RustProviderReceiptArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
        }
    }

    fn write_bytes(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn digest_path(root: &Path, relative: &str) -> String {
        content_digest_blake3(&root.join(relative)).unwrap()
    }
}
