//! Signed owner-bound project declaration receiver. The configured canonical
//! root is operator input, not a root or owner asserted by the remote caller.
#![allow(dead_code)] // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
use std::path::Path;
use std::path::PathBuf;

use super::BuildTarget;
use super::ResolvedProject;
use super::Selector;
use super::resolve_project_target;
use crate::errors::RunError;
use crate::nix_gateway::authority::Error;
use crate::nix_gateway::authority::GatewaySession;
use crate::nix_gateway::authority::GatewayVerifier;
use crate::nix_gateway::authority::Operation;
use crate::nix_gateway::authority::project_segment;

/// Explicit operator project identity. Construction grants no authority.
pub struct ProjectAuthorityConfig {
    owner: String,
    root_file: PathBuf,
}
impl ProjectAuthorityConfig {
    /// Admit one existing canonical project file, never ancestor discovery from
    /// a caller-supplied working directory. This does not read its Nickel source.
    pub fn open(owner: &str, root_file: &Path) -> Result<Self, Error> {
        if !project_segment(owner) {
            return Err(Error);
        }
        let root_file = root_file.canonicalize().map_err(|_| Error)?;
        let file_name = root_file.file_name().and_then(|name| name.to_str()).ok_or(Error)?;
        if ![super::CANONICAL_PROJECT_ROOT_FILE, super::LEGACY_PROJECT_ROOT_FILE].contains(&file_name)
            || !root_file.is_file()
        {
            return Err(Error);
        }
        // Existing expression generation embeds the import path as Nickel text.
        let root_text = root_file.to_str().ok_or(Error)?;
        if root_text.chars().any(|ch| ch.is_control() || matches!(ch, '"' | '\\' | '%')) {
            return Err(Error);
        }
        Ok(Self {
            owner: owner.into(),
            root_file,
        })
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn root_file(&self) -> &Path {
        &self.root_file
    }
}

fn suffix<'a>(owner: &str, value: &'a str, kind: &str) -> Result<&'a str, Error> {
    let suffix = value
        .strip_prefix("/project/")
        .and_then(|value| value.strip_prefix(owner))
        .and_then(|value| value.strip_prefix('/'))
        .and_then(|value| value.strip_prefix(kind))
        .and_then(|value| value.strip_prefix('/'))
        .ok_or(Error)?;
    if value.len() > crunch_remote_core::caveat::MAX_VALUE_BYTES || !suffix.split('/').all(project_segment) {
        return Err(Error);
    }
    Ok(suffix)
}

/// Canonical signing digest, not a grant. All four fields are length-framed;
/// root identity is the exact canonical configured project file path.
pub fn declaration_digest(
    config: &ProjectAuthorityConfig,
    original: &str,
    final_goal: &str,
) -> Result<[u8; 32], Error> {
    suffix(&config.owner, original, "declared")?;
    suffix(&config.owner, final_goal, "goal")?;
    let mut digest = blake3::Hasher::new();
    digest.update(b"mantle/project/declaration-effect/v1\0");
    for field in [
        config.owner.as_bytes(),
        original.as_bytes(),
        final_goal.as_bytes(),
        config.root_file.as_os_str().as_encoded_bytes(),
    ] {
        digest.update(&u64::try_from(field.len()).map_err(|_| Error)?.to_be_bytes());
        digest.update(field);
    }
    Ok(*digest.finalize().as_bytes())
}

fn denied(_: Error) -> RunError {
    RunError::Internal("gateway-authority-denied".into())
}

/// Revalidate the real non-cloneable signed session before project resolution.
/// The goal's slash-separated suffix becomes the actual selector segments. The
/// caller retains `session` through evaluation/build and revalidates at effects;
/// this function never spends the holder nonce a second time.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn resolve_project_target_with_authority(
    config: &ProjectAuthorityConfig,
    verifier: &GatewayVerifier,
    session: &GatewaySession,
    now: u64,
    user_import_paths: &[PathBuf],
) -> Result<ResolvedProject, RunError> {
    let original = session.project_declaration().ok_or_else(|| denied(Error))?;
    if session.project() != config.owner {
        return Err(denied(Error));
    }
    let digest = declaration_digest(config, original, session.target()).map_err(denied)?;
    session.require(Operation::ProjectDeclare, session.target(), digest).map_err(denied)?;
    verifier.revalidate(session, now).map_err(denied)?;
    let segments = suffix(&config.owner, session.target(), "goal")
        .map_err(denied)?
        .split('/')
        .map(String::from)
        .collect();
    let cwd = config.root_file.parent().ok_or_else(|| denied(Error))?;
    let resolved = resolve_project_target(&BuildTarget::Selector(Selector { segments }), cwd, user_import_paths)?;
    if resolved.root_file.canonicalize().map_err(|_| denied(Error))? != config.root_file {
        return Err(denied(Error));
    }
    Ok(resolved)
}
