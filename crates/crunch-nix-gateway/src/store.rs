//! Verified store projections and an explicitly admitted signed input import.
//! No implicit execution, substitution or host `/nix/store` fallback authority.

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use std::io::{self};

use crunch_build::HermeticityMode;
use crunch_build::action_result::SignedDerivationSource;
use crunch_build::action_result::admit_signed_derivation_outputs;
use crunch_build::action_result::verify_signed_store_derivation;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use crunch_nix_gateway_core::Reject;
use crunch_nix_gateway_core::{self as core};
use crunch_store::ActionResultPort;
use crunch_store::BuildServiceStore;
use crunch_store::BuildStore;
use crunch_store::OutputLookup;
use crunch_store::StoreHandle;
use crunch_store::standard_nix_ca_path;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use nix_compat::store_path::build_ca_path_with_store_dir;
use snix_store::path_info::PathInfo;

use crate::ParsedRequest;
use crate::STDERR_LAST;
use crate::write_bytes;
use crate::write_number;

const PREFIX: &str = "/nix/store";

fn ca_identity_matches(info: &PathInfo) -> bool {
    let Some(ca) = info.ca.as_ref() else { return true };
    let marker: Result<StorePath<String>, _> =
        build_ca_path_with_store_dir(info.store_path.name(), ca, Vec::<String>::new(), false, PREFIX);
    if marker.is_ok_and(|candidate| candidate == info.store_path) {
        return true;
    }
    let self_reference = info.references.iter().any(|path| path == &info.store_path);
    let references = info.references.iter().filter(|path| *path != &info.store_path).map(ToString::to_string);
    let native: Result<StorePath<String>, _> =
        build_ca_path_with_store_dir(info.store_path.name(), ca, references, self_reference, PREFIX);
    if native.is_ok_and(|candidate| candidate == info.store_path) {
        return true;
    }
    // The vendored native path builder uses BLAKE3 even with /nix/store.
    // Signed Nix imports instead require the existing SHA-256 interop path.
    let mut references = info
        .references
        .iter()
        .filter(|path| *path != &info.store_path)
        .map(StorePath::to_absolute_path)
        .collect::<Vec<_>>();
    if self_reference {
        let nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(_)) = ca else {
            return false;
        };
        references.push("self".to_string());
    }
    standard_nix_ca_path(info.store_path.name(), ca, &references).is_ok_and(|candidate| candidate == info.store_path)
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

    pub fn declared_output_paths(&self) -> impl Iterator<Item = String> + '_ {
        self.nix_derivation
            .outputs
            .values()
            .filter_map(|output| output.path.as_ref())
            .map(StorePath::to_absolute_path)
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
        if info.store_path != parsed
            || info.nar_size == 0
            || info.nar_size > core::MAX_TRANSFER_BYTES
            || info.references.len() > core::MAX_STORE_PATHS as usize
            || info.signatures.len() > 16
            || info.signatures.iter().any(|signature| signature.name().len() > 256)
            || !ca_identity_matches(&info)
        {
            return Err(Reject::Authority);
        }
        let references: Vec<StorePathRef<'_>> = info.references.iter().map(StorePath::as_ref).collect();
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &info.nar_sha256,
            info.nar_size,
            references.iter(),
            PREFIX,
        );
        if !info
            .signatures
            .iter()
            .any(|signature| self.trusted_keys.iter().any(|key| key.verify(&fingerprint, &signature.as_ref())))
        {
            return Err(Reject::Authority);
        }
        if !self.content.has_complete_content(&info).await.map_err(|_| Reject::Authority)? {
            return Err(Reject::Authority);
        }
        let (measured_size, measured_hash) =
            self.owner.calculate_nar(&info.node).await.map_err(|_| Reject::Authority)?;
        if measured_size != info.nar_size || measured_hash != info.nar_sha256 {
            return Err(Reject::Authority);
        }
        Ok(Some(info))
    }

    /// Reuse the ordinary signature, overlay, complete-CAS and measured-NAR
    /// admission before comparing retained producer identities. Matching paths
    /// or another currently trusted result are not the same producer result.
    pub async fn verify_retained_path_infos(&self, expected: &[&PathInfo]) -> Result<(), Reject> {
        if expected.is_empty() || expected.len() > core::MAX_STORE_PATHS as usize {
            return Err(Reject::Bound);
        }
        let mut seen = HashSet::with_capacity(expected.len());
        for retained in expected {
            if !seen.insert(&retained.store_path) {
                return Err(Reject::Identity);
            }
            let absolute = retained.store_path.to_absolute_path();
            let current = self.verified_path_info(&absolute).await?.ok_or(Reject::Operation)?;
            if current.store_path != retained.store_path
                || current.node != retained.node
                || current.nar_sha256 != retained.nar_sha256
                || current.nar_size != retained.nar_size
                || current.references != retained.references
                || current.ca != retained.ca
            {
                return Err(Reject::Authority);
            }
        }
        Ok(())
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
        let verified = verify_signed_store_derivation(
            SignedDerivationSource {
                build_store: &self.owner,
                lookup: &self.lookup,
                content: &self.content,
                store_dir: PREFIX,
                trusted_keys: &self.trusted_keys,
            },
            &path,
        )
        .await
        .map_err(|_| Reject::Authority)?;
        let original = verified.derivation();
        if !original.input_derivations.is_empty()
            || original.input_sources.len() > core::MAX_STORE_PATHS as usize
            || original.outputs.is_empty()
            || original.outputs.len() > core::MAX_STORE_PATHS as usize
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
            inputs: original
                .input_sources
                .iter()
                .map(|source| Input::Source(source.to_absolute_path_with_prefix(PREFIX)))
                .collect(),
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        };
        let mut cache = ConversionCache::new(PREFIX);
        let (computed_path, nix_derivation) =
            crunch_glue::convert(&crunch_derivation, &mut cache).map_err(|_| Reject::Operation)?;
        if computed_path != path || &nix_derivation != original {
            return Err(Reject::Identity);
        }
        Ok(ProvenRemoteDerivation {
            drv_path: computed_path,
            crunch_derivation,
            nix_derivation,
        })
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

    /// Import only an input named by the host's already-admitted concrete
    /// request. `admitted_inputs` and `remaining_upload_bytes` come from the
    /// existing durable ticket/attempt boundary, never the Nix peer. This
    /// method does not grant upload authority or redeem a credential.
    // r[impl remote_builds.gateway_store_integrity]
    pub async fn import_admitted_nar<R: tokio::io::AsyncRead + Unpin + Send>(
        &mut self,
        request: ParsedRequest,
        reader: &mut R,
        admitted_inputs: &[String],
        remaining_upload_bytes: u64,
        mut reauthorize: impl FnMut() -> Result<(), Reject>,
    ) -> Result<u64, Reject> {
        use nix_compat::narinfo::Signature;
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::HashAlgo;
        use nix_compat::nixhash::NixHash;
        use tokio::io::AsyncReadExt;
        reauthorize()?;
        let ParsedRequest::AddToStoreNar {
            path,
            deriver,
            nar_hash,
            references,
            nar_size,
            signatures,
            content_address,
            ..
        } = request
        else {
            return Err(Reject::Operation);
        };
        if admitted_inputs.len() > core::MAX_STORE_PATHS as usize
            || !admitted_inputs.contains(&path)
            || nar_size == 0
            || nar_size > remaining_upload_bytes
            || nar_size > core::MAX_TRANSFER_BYTES
            || references.len() > core::MAX_STORE_PATHS as usize
            || signatures.is_empty()
            || signatures.len() > 16
        {
            return Err(Reject::Authority);
        }
        if !core::valid_store_path(&path, false)
            || (!deriver.is_empty() && !core::valid_store_path(&deriver, true))
            || nar_hash.len() != 64
            || content_address.len() > 128
            || !nar_hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || references.iter().any(|path| !core::valid_store_path(path, false))
            || signatures.iter().any(|signature| signature.len() > 1024)
        {
            return Err(Reject::Identity);
        }
        let store_path = StorePath::<String>::from_absolute_path(path.as_bytes()).map_err(|_| Reject::Identity)?;
        let references = references
            .iter()
            .map(|reference| {
                StorePath::<String>::from_absolute_path(reference.as_bytes()).map_err(|_| Reject::Identity)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if references.iter().collect::<HashSet<_>>().len() != references.len() {
            return Err(Reject::Identity);
        }
        let hash = NixHash::from_str(&nar_hash, Some(HashAlgo::Sha256)).map_err(|_| Reject::Identity)?;
        let nar_sha256: [u8; 32] = hash.digest_as_bytes().try_into().map_err(|_| Reject::Identity)?;
        let signatures = signatures
            .iter()
            .map(|text| Signature::<String>::parse(text).map_err(|_| Reject::Authority))
            .collect::<Result<Vec<_>, _>>()?;
        let refs: Vec<_> = references.iter().map(StorePath::as_ref).collect();
        let fingerprint = fingerprint_with_store_dir(&store_path.as_ref(), &nar_sha256, nar_size, refs.iter(), PREFIX);
        if !signatures
            .iter()
            .any(|signature| self.trusted_keys.iter().any(|key| key.verify(&fingerprint, &signature.as_ref())))
        {
            return Err(Reject::Authority);
        }
        for reference in &references {
            if reference == &store_path {
                continue;
            }
            self.verified_path_info(&reference.to_absolute_path()).await?.ok_or(Reject::Authority)?;
        }
        let ca = if content_address.is_empty() {
            None
        } else {
            Some(CAHash::from_nix_hex_str(&content_address).ok_or(Reject::Identity)?)
        };
        let mut bounded = reader.take(nar_size.checked_add(1).ok_or(Reject::Bound)?);
        let (node, measured_hash, measured_size) =
            self.owner.ingest_nar(&mut bounded, &ca).await.map_err(|_| Reject::Identity)?;
        let mut trailing = [0_u8; 1];
        if measured_hash != nar_sha256
            || measured_size != nar_size
            || bounded.read(&mut trailing).await.map_err(|_| Reject::Bound)? != 0
        {
            return Err(Reject::Identity);
        }
        let deriver = if deriver.is_empty() {
            None
        } else {
            let value = StorePath::<String>::from_absolute_path(deriver.as_bytes()).map_err(|_| Reject::Identity)?;
            let name = value.name().strip_suffix(".drv").ok_or(Reject::Identity)?;
            Some(StorePath::from_name_and_digest(name, value.digest()).map_err(|_| Reject::Identity)?)
        };
        let info = PathInfo {
            store_path,
            node,
            references,
            nar_size,
            nar_sha256,
            signatures,
            deriver,
            ca,
        };
        if !ca_identity_matches(&info) {
            return Err(Reject::Identity);
        }
        if let Some(existing) = self.lookup.find(&info.store_path).await.map_err(|_| Reject::Authority)?
            && (existing.node != info.node
                || existing.nar_sha256 != info.nar_sha256
                || existing.nar_size != info.nar_size
                || existing.references != info.references
                || existing.ca != info.ca)
        {
            return Err(Reject::Conflict);
        }
        reauthorize()?;
        self.owner
            .persist_and_export_signed_output(crunch_store::PersistOutputRequest {
                output_name: "input",
                output_path: &info.store_path,
                path_info: info.clone(),
                final_node: info.node.clone(),
                provenance: None,
                is_root: false,
                root_source: None,
            })
            .await
            .map_err(|_| Reject::Authority)?;
        // An admitted NAR import is published for the clients that asked for it, so
        // the store path must be readable at its path. It is not a retained
        // garbage-collection root, so it is materialized without root registration.
        self.owner.materialize_persisted_output(&info.store_path).await.map_err(|_| Reject::Authority)?;
        Ok(nar_size)
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
    /// The read-only `answer` path never acknowledges `BuildPaths`.
    async fn require_cached_derived_outputs(&self, drv: &str, output_spec: &str) -> Result<(), Reject> {
        if !core::valid_store_path(drv, true) || output_spec.is_empty() {
            return Err(Reject::Identity);
        }
        let path = StorePath::<String>::from_absolute_path(drv.as_bytes()).map_err(|_| Reject::Identity)?;
        let verified = verify_signed_store_derivation(
            SignedDerivationSource {
                build_store: &self.owner,
                lookup: &self.lookup,
                content: &self.content,
                store_dir: PREFIX,
                trusted_keys: &self.trusted_keys,
            },
            &path,
        )
        .await
        .map_err(|_| Reject::Authority)?;
        let admitted = admit_signed_derivation_outputs(&self.action_results, &verified, HermeticityMode::Strict)
            .await
            .map_err(|_| Reject::Authority)?
            .ok_or(Reject::Operation)?;
        let selected: Vec<&PathInfo> = if output_spec == "*" {
            admitted.outputs.values().collect()
        } else {
            output_spec
                .split(',')
                .map(|name| admitted.outputs.get(name).ok_or(Reject::Operation))
                .collect::<Result<_, _>>()?
        };
        if selected.is_empty() || selected.len() > core::MAX_STORE_PATHS as usize {
            return Err(Reject::Bound);
        }
        for expected in selected {
            let absolute = expected.store_path.to_absolute_path();
            let current = self.verified_path_info(&absolute).await?.ok_or(Reject::Operation)?;
            if current.store_path != expected.store_path
                || current.node != expected.node
                || current.nar_sha256 != expected.nar_sha256
                || current.nar_size != expected.nar_size
                || current.references != expected.references
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
            ParsedRequest::QueryValidPaths { paths, substitute } => {
                if substitute || paths.len() > core::MAX_STORE_PATHS as usize {
                    return Err(Reject::Operation);
                }
                let mut valid = Vec::with_capacity(paths.len());
                for path in paths {
                    if self.verified_path_info(&path).await?.is_some() {
                        valid.push(path);
                    }
                }
                valid.sort();
                valid.dedup();
                write_number(output, STDERR_LAST).map_err(|_| Reject::Operation)?;
                write_paths(output, &valid).map_err(|_| Reject::Operation)?;
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
