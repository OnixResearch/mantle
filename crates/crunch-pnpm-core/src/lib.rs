#![no_std]
//! Bounded pnpm input contract. pnpm, not Mantle, owns dependency resolution.
extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crunch_source_core::Blake3Digest;
use crunch_source_core::LocatorBoundaryPolicy;
use crunch_source_core::LocatorClass;
use crunch_source_core::ProjectionPath;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservation;
use crunch_source_core::SourceObservationRequest;
use crunch_source_core::admit_source_observation;
use crunch_source_core::domain_digest;
use serde::Deserialize;
use serde::Serialize;

pub const NODE_VERSION: &str = "22.14.0";
pub const PNPM_VERSION: &str = "9.15.9";
pub const MAX_PACKAGES: usize = 4096;
pub const MAX_INPUT_BYTES: u64 = 536_870_912;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub lock_key: String,
    pub archive: String,
    pub integrity: String,
    pub blake3: Blake3Digest,
    /// Non-secret observational mirror locator; never a download instruction.
    pub mirror: String,
    /// Optional immutable Git archive provenance. Raw Git resolution is rejected.
    pub git_revision: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub workspace: String,
    /// Explicit ordered script permissions; no automatic pre/post lifecycle hooks.
    pub scripts: Vec<String>,
    pub outputs: Vec<String>,
    /// Optional relative destination populated by pnpm pack (all hooks disabled).
    pub package_output: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub node_version: String,
    pub pnpm_version: String,
    pub platform: String,
    pub source_blake3: Blake3Digest,
    pub tools_blake3: Blake3Digest,
    pub packages: Vec<Package>,
    pub targets: Vec<Target>,
    /// Exact absolute runtime input roots (e.g. dynamic loader closures).
    pub runtime: Vec<Runtime>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub path: String,
    pub blake3: Blake3Digest,
}

/// Checked plan, preserving the wire request and its ordered source observations.
#[derive(Debug)]
pub struct AdmittedRequest {
    pub request: Request,
    pub identity: Blake3Digest,
    pub sources: Vec<SourceObservation>,
}

pub fn member(value: String) -> Result<String, String> {
    check_member(&value)?;
    Ok(value)
}

fn check_member(value: &str) -> Result<(), String> {
    ProjectionPath::parse(value).map_err(|e| e.message)?;
    if value.len() > 1024 || value.bytes().any(|b| b < 32) {
        return Err("unsafe or oversized member".to_string());
    }
    Ok(())
}

pub fn validate(request: Request) -> Result<AdmittedRequest, String> {
    check_toolchain(&request)?;
    check_bounds(&request)?;
    let sources = collect_packages(&request)?;
    check_targets(&request)?;
    check_runtime(&request)?;
    let identity = domain_digest(b"mantle-pnpm-workspace-v1\0", &request).map_err(|e| e.to_string())?;
    debug_assert_eq!(sources.len(), request.packages.len());
    Ok(AdmittedRequest {
        request,
        identity,
        sources,
    })
}

/// Exact schema, Node, pnpm and platform cohort this contract admits.
fn check_toolchain(request: &Request) -> Result<(), String> {
    let is_schema_admitted = request.schema == "mantle-pnpm-workspace-v1";
    let is_node_admitted = request.node_version == NODE_VERSION;
    let is_pnpm_admitted = request.pnpm_version == PNPM_VERSION;
    let is_platform_admitted = request.platform == "linux-x64";
    let is_toolchain_admitted = is_schema_admitted && is_node_admitted && is_pnpm_admitted && is_platform_admitted;
    if !is_toolchain_admitted {
        return Err("requires Node 22.14.0, pnpm 9.15.9, Linux x64".to_string());
    }
    Ok(())
}

/// Bounded package, target and runtime cardinalities.
fn check_bounds(request: &Request) -> Result<(), String> {
    let has_too_many_packages = request.packages.len() > MAX_PACKAGES;
    let has_zero_targets = request.targets.is_empty();
    let has_too_many_targets = request.targets.len() > 128;
    let has_too_many_runtime_inputs = request.runtime.len() > 128;
    let is_within_bounds =
        !has_too_many_packages && !has_zero_targets && !has_too_many_targets && !has_too_many_runtime_inputs;
    if !is_within_bounds {
        return Err("pnpm plan bound exceeded".to_string());
    }
    Ok(())
}

/// Admit every declared package and collect its ordered source observation.
fn collect_packages(request: &Request) -> Result<Vec<SourceObservation>, String> {
    let mut keys = BTreeSet::new();
    let mut sources = Vec::with_capacity(request.packages.len());
    for package in &request.packages {
        check_member(&package.archive)?;
        if !keys.insert(&package.lock_key) || !package.integrity.starts_with("sha512-") || package.integrity.len() != 95
        {
            return Err("duplicate package or unsupported integrity (SHA-512 SRI only)".to_string());
        }
        if let Some(revision) = &package.git_revision
            && (revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("Git archive requires immutable SHA-1 revision".to_string());
        }
        sources.push(observation(package)?);
    }
    debug_assert_eq!(sources.len(), request.packages.len());
    debug_assert_eq!(keys.len(), sources.len());
    Ok(sources)
}

/// Admit every declared workspace target and its explicit script permissions.
fn check_targets(request: &Request) -> Result<(), String> {
    let mut workspaces = BTreeSet::new();
    for target in &request.targets {
        check_member(&target.workspace)?;
        let is_new_workspace = workspaces.insert(&target.workspace);
        let has_zero_outputs = target.outputs.is_empty();
        let has_too_many_outputs = target.outputs.len() > 32;
        let has_too_many_scripts = target.scripts.len() > 16;
        let is_target_admissible =
            is_new_workspace && !has_zero_outputs && !has_too_many_outputs && !has_too_many_scripts;
        if !is_target_admissible {
            return Err("duplicate workspace or target bound exceeded".to_string());
        }
        for output in &target.outputs {
            check_member(output)?;
        }
        if let Some(output) = &target.package_output {
            check_member(output)?;
        }
        for script in &target.scripts {
            let is_script_admissible = !script.is_empty()
                && script.len() <= 128
                && script.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b));
            if !is_script_admissible {
                return Err("unsafe script permission".to_string());
            }
        }
    }
    debug_assert_eq!(workspaces.len(), request.targets.len());
    debug_assert!(request.targets.iter().all(|target| !target.outputs.is_empty()));
    Ok(())
}

/// Runtime inputs must be exact `/nix/store` entries, never nested paths.
fn check_runtime(request: &Request) -> Result<(), String> {
    for input in &request.runtime {
        let is_exact_store_entry = input.path.starts_with("/nix/store/") && input.path.len() > 11;
        let has_nested_path = is_exact_store_entry && input.path[11..].contains('/');
        if !is_exact_store_entry || has_nested_path {
            return Err("runtime roots must be exact /nix/store entries".to_string());
        }
    }
    Ok(())
}

fn observation(package: &Package) -> Result<SourceObservation, String> {
    let result = admit_source_observation(
        &SourceObservationRequest {
            kind: SourceKind::PackageMirror,
            locator: LocatorClass::PackageMirror {
                mirror: package.mirror.clone(),
            },
            mutable_ref_hint: None,
            git_object_format: None,
            git_revision: None,
            projection: ProjectionPath::parse(&package.archive).map_err(|e| e.message)?,
            snapshot_profile: SnapshotProfile {
                name: "pnpm-tarball-v1".to_string(),
                version: 1,
            },
            payload_blake3: package.blake3.clone(),
        },
        &LocatorBoundaryPolicy {
            allowed_query_fields: Vec::new(),
        },
    );
    let observation = result.observation.ok_or_else(|| alloc::format!("{:?}", result.diagnostics))?;
    debug_assert!(observation.mutable_ref_hint.is_none());
    debug_assert_eq!(observation.payload_blake3, package.blake3);
    Ok(observation)
}
