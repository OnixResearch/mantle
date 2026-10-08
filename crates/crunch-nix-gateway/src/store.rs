//! Store-read projection for the bounded Nix worker protocol. No substitution,
//! execution, import or fallback to the host's `/nix/store` is permitted.

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{self, Write};

use crunch_build::HermeticityMode;
use crunch_build::action_result::{
    SignedDerivationSource, admit_signed_derivation_outputs, verify_signed_store_derivation,
};
use crunch_glue::{ConversionCache, CrunchDerivation, Input};
use crunch_nix_gateway_core::{self as core, Reject};
use crunch_store::{ActionResultPort, BuildServiceStore, BuildStore, OutputLookup, StoreHandle};
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::{VerifyingKey, fingerprint_with_store_dir};
use nix_compat::store_path::{StorePath, StorePathRef, build_ca_path_with_store_dir};
use snix_store::path_info::PathInfo;

use crate::{ParsedRequest, STDERR_LAST, write_bytes, write_number};

const PREFIX: &str = "/nix/store";

fn ca_identity_matches(info: &PathInfo) -> bool {
    let Some(ca) = info.ca.as_ref() else { return true };
    let marker: Result<StorePath<String>, _> =
        build_ca_path_with_store_dir(info.store_path.name(), ca, Vec::<String>::new(), false, PREFIX);
    if marker.is_ok_and(|candidate| candidate == info.store_path) {
        return true;
    }
    let self_reference = info.references.iter().any(|path| path == &info.store_path);
    let references = info.references.iter().filter(|path| *path != &info.store_path)
        .map(ToString::to_string).collect::<Vec<_>>();
    let standard: Result<StorePath<String>, _> =
        build_ca_path_with_store_dir(info.store_path.name(), ca, references, self_reference, PREFIX);
    standard.is_ok_and(|candidate| candidate == info.store_path)
}

/// The verified, lossless IA subset that the existing remote shell can
/// convert to its canonical concrete derivation request. This is not
/// permission to dispatch a worker or acknowledge BuildPaths.
pub struct ProvenRemoteDerivation {
    drv_path: StorePath<String>,
    crunch_derivation: CrunchDerivation,
    nix_derivation: Derivation,
}

impl ProvenRemoteDerivation {
    pub fn into_derivations(self) -> (StorePath<String>, CrunchDerivation, Derivation) {
        (self.drv_path, self.crunch_derivation, self.nix_derivation)
    }
}

/// Store capability owner and read projections. Action-result discovery does
/// not grant execution or publication authority to the Nix worker socket.
pub struct VerifiedStore {
    owner: BuildStore,
    lookup: OutputLookup,
    content: BuildServiceStore,
    action_results: ActionResultPort,
    trusted_keys: Vec<VerifyingKey>,
}

impl VerifiedStore {
    /// Trust must be configured explicitly; an absent trust file must not
    /// silently turn every PathInfo record into a valid Nix store path.
    pub fn new(handle: StoreHandle, trusted_keys: Vec<VerifyingKey>) -> Result<Self, Reject> {
        if handle.store_dir() != PREFIX || trusted_keys.is_empty() || trusted_keys.len() > 16 {
            return Err(Reject::Authority);
        }
        let parts = handle.into_pipeline_store_parts();
        Ok(Self {
            owner: parts.build_store,
            lookup: parts.output_lookup,
            content: parts.build_service_store,
            action_results: parts.action_results,
            trusted_keys,
        })
    }

    async fn verified_path_info(&self, path: &str) -> Result<Option<PathInfo>, Reject> {
        if !core::valid_store_path(path, false) {
            return Err(Reject::Identity);
        }
        let parsed = StorePath::<String>::from_absolute_path(path.as_bytes()).map_err(|_| Reject::Identity)?;
        let Some(info) = self.lookup.find(&parsed).await.map_err(|_| Reject::Authority)? else {
            return Ok(None);
        };
        // OutputLookup additionally revalidates overlay/layer selection. The
        // gateway imposes its own configured signature policy even on local
        // PathInfo and never trusts the mere existence of a metadata record.
        if info.store_path != parsed || info.nar_size == 0 || info.nar_size > core::MAX_TRANSFER_BYTES
            || info.references.len() > core::MAX_STORE_PATHS as usize
            || info.signatures.len() > 16
            || info.signatures.iter().any(|signature| signature.name().len() > 256)
            || !ca_identity_matches(&info)
        {
            return Err(Reject::Authority);
        }
        let references: Vec<StorePathRef<'_>> = info.references.iter().map(StorePath::as_ref).collect();
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(), &info.nar_sha256, info.nar_size, references.iter(), PREFIX,
        );
        if !info.signatures.iter().any(|signature| {
            self.trusted_keys.iter().any(|key| key.verify(&fingerprint, &signature.as_ref()))
        }) {
            return Err(Reject::Authority);
        }
        if !self.content.has_complete_content(&info).await.map_err(|_| Reject::Authority)? {
            return Err(Reject::Authority);
        }
        let (measured_size, measured_hash) = self.owner.calculate_nar(&info.node).await
            .map_err(|_| Reject::Authority)?;
        if measured_size != info.nar_size || measured_hash != info.nar_sha256 {
            return Err(Reject::Authority);
        }
        Ok(Some(info))
    }

    /// Reconstruct only a signed IA derivation that round-trips byte-for-byte
    /// through the already deployed Crunch-to-Nix converter. A native Nix
    /// `.drv` with a different derivation hash, non-UTF8 environment,
    /// derivation dependencies or altered declared output cannot run by
    /// changing its identity. Every declared source must already be signed,
    /// present and measured before any remote request is considered.
    pub async fn project_signed_ia_for_remote(&self, drv: &str) -> Result<ProvenRemoteDerivation, Reject> {
        if !core::valid_store_path(drv, true) {
            return Err(Reject::Identity);
        }
        let path = StorePath::<String>::from_absolute_path(drv.as_bytes()).map_err(|_| Reject::Identity)?;
        let verified = verify_signed_store_derivation(SignedDerivationSource {
            build_store: &self.owner,
            lookup: &self.lookup,
            content: &self.content,
            store_dir: PREFIX,
            trusted_keys: &self.trusted_keys,
        }, &path).await.map_err(|_| Reject::Authority)?;
        let original = verified.derivation();
        if !original.input_derivations.is_empty() || original.input_sources.len() > core::MAX_STORE_PATHS as usize
            || original.outputs.is_empty() || original.outputs.len() > core::MAX_STORE_PATHS as usize
            || original.outputs.values().any(|output| output.ca_hash.is_some() || output.path.is_none())
        {
            return Err(Reject::Operation);
        }
        for source in &original.input_sources {
            let absolute = source.to_absolute_path_with_prefix(PREFIX);
            self.verified_path_info(&absolute).await?.ok_or(Reject::Operation)?;
        }
        let name = path.name().strip_suffix(".drv").filter(|name| !name.is_empty()).ok_or(Reject::Identity)?;
        let mut env = HashMap::with_capacity(original.environment.len());
        for (key, value) in &original.environment {
            let text = std::str::from_utf8(value.as_ref()).map_err(|_| Reject::Operation)?;
            env.insert(key.clone(), text.to_string());
        }
        let crunch_derivation = CrunchDerivation {
            name: name.to_string(),
            builder: original.builder.clone(),
            system: original.system.clone(),
            args: original.arguments.clone(),
            outputs: original.outputs.keys().cloned().collect(),
            dynamic_plan_outputs: Vec::new(),
            env,
            inputs: original.input_sources.iter()
                .map(|source| Input::Source(source.to_absolute_path_with_prefix(PREFIX)))
                .collect(),
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        };
        let mut cache = ConversionCache::new(PREFIX);
        let (computed_path, nix_derivation) = crunch_glue::convert(&crunch_derivation, &mut cache)
            .map_err(|_| Reject::Operation)?;
        if computed_path != path || &nix_derivation != original {
            return Err(Reject::Identity);
        }
        Ok(ProvenRemoteDerivation { drv_path: computed_path, crunch_derivation, nix_derivation })
    }

    /// Read-only preparation of complete native `BuildPaths` targets. This
    /// does not redeem a ticket, select a worker, submit an attempt, or write
    /// any Nix success response. The eventual host must still bind an exact
    /// authorized concrete request to a current eligible worker and fence.
    pub async fn project_build_paths_for_remote(
        &self,
        targets: &[String],
    ) -> Result<Vec<ProvenRemoteDerivation>, Reject> {
        if targets.is_empty() || targets.len() > core::MAX_STORE_PATHS as usize {
            return Err(Reject::Bound);
        }
        let mut seen = HashSet::with_capacity(targets.len());
        let mut projected = Vec::with_capacity(targets.len());
        for target in targets {
            let (drv, selected) = target.split_once('!').ok_or(Reject::Identity)?;
            if !seen.insert(drv) {
                return Err(Reject::Conflict);
            }
            let verified = self.project_signed_ia_for_remote(drv).await?;
            if selected != "*" {
                let mut outputs = HashSet::with_capacity(verified.nix_derivation.outputs.len());
                for name in selected.split(',') {
                    if !verified.nix_derivation.outputs.contains_key(name) || !outputs.insert(name) {
                        return Err(Reject::Operation);
                    }
                }
                if outputs.len() != verified.nix_derivation.outputs.len() {
                    return Err(Reject::Operation);
                }
            }
            projected.push(verified);
        }
        Ok(projected)
    }

    /// QueryMissing's response is a set of *store paths*, not derived-path
    /// strings. An unsigned CA mapping never authorizes a derived target.
    /// A signed derivation+action-result may establish that all its selected
    /// outputs are already present; no build or substitution is advertised.
    pub async fn query_missing(&self, targets: &[String]) -> Result<Vec<String>, Reject> {
        if targets.len() > core::MAX_STORE_PATHS as usize {
            return Err(Reject::Bound);
        }
        let mut unknown = Vec::new();
        for target in targets {
            if let Some((drv, output_spec)) = target.split_once('!') {
                self.require_cached_derived_outputs(drv, output_spec).await?;
                continue;
            }
            let found = self.verified_path_info(target).await?;
            if found.is_none() {
                unknown.push(target.clone());
            }
        }
        unknown.sort();
        unknown.dedup();
        Ok(unknown)
    }

    /// Cached derived paths may be *read* through QueryMissing but their
    /// presence cannot bypass the separate authenticated Build permission.
    /// In particular, `BuildPaths` still has no successful wire response.
    async fn require_cached_derived_outputs(&self, drv: &str, output_spec: &str) -> Result<(), Reject> {
        if !core::valid_store_path(drv, true) || output_spec.is_empty() {
            return Err(Reject::Identity);
        }
        let path = StorePath::<String>::from_absolute_path(drv.as_bytes()).map_err(|_| Reject::Identity)?;
        let verified = verify_signed_store_derivation(SignedDerivationSource {
            build_store: &self.owner,
            lookup: &self.lookup,
            content: &self.content,
            store_dir: PREFIX,
            trusted_keys: &self.trusted_keys,
        }, &path).await.map_err(|_| Reject::Authority)?;
        let admitted = admit_signed_derivation_outputs(
            &self.action_results, &verified, HermeticityMode::Strict,
        ).await.map_err(|_| Reject::Authority)?.ok_or(Reject::Operation)?;
        let selected: Vec<&PathInfo> = if output_spec == "*" {
            admitted.outputs.values().collect()
        } else {
            output_spec.split(',').map(|name| {
                admitted.outputs.get(name).ok_or(Reject::Operation)
            }).collect::<Result<_, _>>()?
        };
        if selected.is_empty() || selected.len() > core::MAX_STORE_PATHS as usize {
            return Err(Reject::Bound);
        }
        for expected in selected {
            let absolute = expected.store_path.to_absolute_path();
            let current = self.verified_path_info(&absolute).await?
                .ok_or(Reject::Operation)?;
            if current.store_path != expected.store_path
                || current.node != expected.node || current.nar_sha256 != expected.nar_sha256
                || current.nar_size != expected.nar_size || current.references != expected.references
                || current.ca != expected.ca
            {
                return Err(Reject::Authority);
            }
        }
        Ok(())
    }

    /// This reply is emitted only after every requested path was verified or
    /// classified as unknown. An adapter error must instead send a redacted
    /// Nix STDERR_ERROR and never a successful empty set.
    pub async fn answer(&self, request: ParsedRequest, output: &mut impl Write) -> Result<(), Reject> {
        match request {
            ParsedRequest::QueryMissing { targets } => {
                let unknown = self.query_missing(&targets).await?;
                write_number(output, STDERR_LAST).map_err(|_| Reject::Operation)?;
                write_number(output, 0).map_err(|_| Reject::Operation)?; // willBuild
                write_number(output, 0).map_err(|_| Reject::Operation)?; // willSubstitute
                write_paths(output, &unknown).map_err(|_| Reject::Operation)?;
                write_number(output, 0).map_err(|_| Reject::Operation)?; // downloadSize
                write_number(output, 0).map_err(|_| Reject::Operation)?; // narSize
            }
            ParsedRequest::IsValidPath { path } => {
                let found = self.verified_path_info(&path).await?.is_some();
                write_number(output, STDERR_LAST).map_err(|_| Reject::Operation)?;
                write_number(output, u64::from(found)).map_err(|_| Reject::Operation)?;
            }
            ParsedRequest::QueryPathInfo { path } => {
                let info = self.verified_path_info(&path).await?;
                write_number(output, STDERR_LAST).map_err(|_| Reject::Operation)?;
                if let Some(info) = info {
                    write_number(output, 1).map_err(|_| Reject::Operation)?;
                    write_path_info(output, &info).map_err(|_| Reject::Operation)?;
                } else {
                    write_number(output, 0).map_err(|_| Reject::Operation)?;
                }
            }
            _ => return Err(Reject::Operation),
        }
        output.flush().map_err(|_| Reject::Operation)
    }
}

fn write_paths(output: &mut impl Write, paths: &[String]) -> io::Result<()> {
    write_number(output, paths.len() as u64)?;
    for path in paths {
        write_bytes(output, path.as_bytes())?;
    }
    Ok(())
}

fn write_path_info(output: &mut impl Write, info: &PathInfo) -> io::Result<()> {
    let deriver = info.deriver.as_ref().map(|path| path.to_absolute_path()).unwrap_or_default();
    write_bytes(output, deriver.as_bytes())?;
    let mut hash = String::with_capacity(7 + 64);
    hash.push_str("sha256:");
    for byte in info.nar_sha256 {
        use std::fmt::Write as _;
        write!(hash, "{byte:02x}").expect("writing to a String cannot fail");
    }
    write_bytes(output, hash.as_bytes())?;
    let mut references: Vec<String> = info.references.iter().map(StorePath::to_absolute_path).collect();
    if references.len() > core::MAX_STORE_PATHS as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "gateway-references-bound"));
    }
    references.sort();
    references.dedup();
    write_paths(output, &references)?;
    write_number(output, 0)?; // local PathInfo has no registration timestamp
    write_number(output, info.nar_size)?;
    write_number(output, 0)?; // gateway never claims ultimate trust
    if info.signatures.len() > 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "gateway-signatures-bound"));
    }
    write_number(output, info.signatures.len() as u64)?;
    for signature in &info.signatures {
        write_bytes(output, signature.to_string().as_bytes())?;
    }
    let address = info.ca.as_ref().map(|ca| ca.to_nix_nixbase32_string()).unwrap_or_default();
    write_bytes(output, address.as_bytes())
}
