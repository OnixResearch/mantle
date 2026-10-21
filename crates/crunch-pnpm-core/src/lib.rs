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
pub const MAX_INPUT_BYTES: u64 = 512 * 1024 * 1024;

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
    if request.schema != "mantle-pnpm-workspace-v1"
        || request.node_version != NODE_VERSION
        || request.pnpm_version != PNPM_VERSION
        || request.platform != "linux-x64"
    {
        return Err("requires Node 22.14.0, pnpm 9.15.9, Linux x64".to_string());
    }
    if request.packages.len() > MAX_PACKAGES
        || request.targets.is_empty()
        || request.targets.len() > 128
        || request.runtime.len() > 128
    {
        return Err("pnpm plan bound exceeded".to_string());
    }
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
    let mut workspaces = BTreeSet::new();
    for target in &request.targets {
        check_member(&target.workspace)?;
        if !workspaces.insert(&target.workspace)
            || target.outputs.is_empty()
            || target.outputs.len() > 32
            || target.scripts.len() > 16
        {
            return Err("duplicate workspace or target bound exceeded".to_string());
        }
        for output in &target.outputs {
            check_member(output)?;
        }
        if let Some(output) = &target.package_output {
            check_member(output)?;
        }
        for script in &target.scripts {
            if script.is_empty()
                || script.len() > 128
                || !script.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
            {
                return Err("unsafe script permission".to_string());
            }
        }
    }
    for input in &request.runtime {
        if !input.path.starts_with("/nix/store/") || input.path.len() <= 11 || input.path[11..].contains('/') {
            return Err("runtime roots must be exact /nix/store entries".to_string());
        }
    }
    let identity = domain_digest(b"mantle-pnpm-workspace-v1\0", &request).map_err(|e| e.to_string())?;
    Ok(AdmittedRequest {
        request,
        identity,
        sources,
    })
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
        &LocatorBoundaryPolicy::default(),
    );
    result.observation.ok_or_else(|| alloc::format!("{:?}", result.diagnostics))
}
