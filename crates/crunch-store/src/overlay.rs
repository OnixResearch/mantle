// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_collection_growth
)]

use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use crunch_overlay_core::BaseCapabilityClass;
use crunch_overlay_core::BaseObservation;
use crunch_overlay_core::GenerationMember;
use crunch_overlay_core::GenerationMemberKind;
use crunch_overlay_core::OverlayPlan;
use crunch_overlay_core::OverlayPolicy;
use data_encoding::HEXLOWER;
use futures::StreamExt;
use futures::stream::BoxStream;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint;
use nix_compat::narinfo::parse_keypair;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::StoreBackend;
use crate::StoreBackendCapabilityProfile;

pub const STORE_STATE_SCHEMA: &str = "mantle-store-state-v1";
pub const STORE_BACKEND_STATE_SCHEMA: &str = "mantle-store-state-v2";
pub const STORE_IDENTITY_FILE_NAME: &str = "store-identity.json";
pub const STORE_OVERLAY_REPORT_SCHEMA: &str = "mantle-store-overlay-report-v2";

const STORE_OVERLAY_POLICY_SCHEMA: &str = "mantle-store-overlay-policy-v1";
const STORE_OVERLAY_POLICY_JSON: &str =
    include_str!("../../../config/store-overlay/generated/store-overlay-policy.json");
const FILE_HASH_BUFFER_BYTES: usize = 65_536;
const DIRECTORY_IDENTITY_DOMAIN: &[u8] = b"mantle.overlay.directory.v1";
pub(crate) const OVERLAY_TRUSTED_PUBLIC_KEYS_FILE_NAME: &str = "overlay-trusted-public-keys";
pub(crate) const LOCAL_SIGNING_KEY_FILE_NAME: &str = "signing-key";
const MAX_LAYER_TRUST_KEYS: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StoreOverlayRuntimePolicy {
    pub schema: String,
    pub policy_name: String,
    pub hash_algorithm: String,
    pub limits: StoreOverlayLimits,
    pub composition: StoreOverlayCompositionPolicy,
    pub trust: StoreOverlayTrustPolicy,
    pub generation: StoreOverlayGenerationPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StoreOverlayLimits {
    pub max_base_layers: usize,
    pub max_generation_members: usize,
    pub max_generation_bytes: u64,
    pub max_descriptor_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StoreOverlayCompositionPolicy {
    pub writable_overlay_count: usize,
    pub base_order: String,
    pub base_capability: String,
    pub require_same_prefix: bool,
    pub reject_duplicate_bases: bool,
    pub no_backfill: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StoreOverlayTrustPolicy {
    pub policy_id: String,
    pub verification: String,
    pub unknown_policy: String,
    pub shadow_failure: String,
    pub skip_failure_classes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct StoreOverlayGenerationPolicy {
    pub allowed_state_schemas: Vec<String>,
    pub bind_root_inventory: bool,
    pub bind_pathinfo: bool,
    pub bind_directories: bool,
    pub bind_blobs: bool,
    pub revalidate_before_execution: bool,
    pub revalidate_before_output_admission: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoreIdentityRecord {
    pub schema: String,
    pub logical_prefix: String,
    pub trust_policy_id: String,
    /// Absent only in legacy v1 records, which unambiguously select Snix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct StoreOverlayState {
    pub plan: OverlayPlan,
    pub base_state_dirs: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoreOverlayReport {
    pub schema: String,
    pub policy_blake3: String,
    pub plan_blake3: String,
    pub logical_prefix: String,
    pub writable_overlay_count: usize,
    pub base_order: String,
    pub no_backfill: bool,
    pub bases: Vec<StoreOverlayBaseReport>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StoreOverlayBaseReport {
    pub declaration_index: usize,
    pub declaration_identity_blake3: String,
    pub accepted_signer_names: Vec<String>,
    pub descriptor_blake3: String,
    pub generation_blake3: String,
    pub state_schema: String,
    pub trust_policy_id: String,
    pub capability_class: String,
    pub observed_member_count: usize,
    pub observed_bytes: u64,
}

pub fn store_overlay_runtime_policy() -> Result<StoreOverlayRuntimePolicy, Error> {
    let policy: StoreOverlayRuntimePolicy = serde_json::from_str(STORE_OVERLAY_POLICY_JSON)
        .map_err(|error| Error::Store(format!("parsing embedded store overlay policy: {error}")))?;
    validate_runtime_policy(&policy)?;
    Ok(policy)
}

#[must_use]
pub fn store_overlay_policy_blake3() -> String {
    format_digest(blake3::hash(STORE_OVERLAY_POLICY_JSON.as_bytes()).as_bytes())
}

#[derive(Clone, Debug)]
enum ObservedStoreIdentity {
    Recorded(StoreIdentityRecord),
    Identityless {
        casita_marker: bool,
        identity_filename: bool,
        other_content: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoreOpenDecision {
    BindNew,
    Reuse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoreOpenRejection<'a> {
    UnsupportedIdentity,
    BackendMismatch(StoreBackend),
    IdentityMismatch {
        observed_prefix: &'a str,
        observed_trust: &'a str,
    },
    IdentitylessContent,
    OverlayUnsupported,
}

#[derive(Clone, Copy)]
struct StoreOpenIdentity<'a> {
    logical_prefix: &'a str,
    trust_policy_id: &'a str,
}

fn observe_store_identity(state_dir: &Path, maximum_identity_bytes: u64) -> Result<ObservedStoreIdentity, Error> {
    let path = state_dir.join(STORE_IDENTITY_FILE_NAME);
    if path.exists() {
        return read_store_identity(&path, maximum_identity_bytes).map(ObservedStoreIdentity::Recorded);
    }
    let mut casita_marker = false;
    let mut identity_filename = false;
    let mut other_content = false;
    if state_dir.exists() {
        for entry in fs::read_dir(state_dir)
            .map_err(|error| Error::Store(format!("reading state directory {}: {error}", state_dir.display())))?
        {
            let entry = entry
                .map_err(|error| Error::Store(format!("reading state directory {}: {error}", state_dir.display())))?;
            let name = entry.file_name();
            if name == "casita" {
                casita_marker = true;
                continue;
            }
            if name == STORE_IDENTITY_FILE_NAME {
                identity_filename = true;
                continue;
            }
            if !matches!(
                name.to_str(),
                Some(
                    "store-mutation.lock"
                        | "signing-key"
                        | "casita-trusted-public-keys"
                        | "overlay-trusted-public-keys"
                )
            ) {
                other_content = true;
            }
        }
    }
    Ok(ObservedStoreIdentity::Identityless {
        casita_marker,
        identity_filename,
        other_content,
    })
}

fn decide_store_open<'a>(
    observed: &'a ObservedStoreIdentity,
    backend: StoreBackend,
    profile: StoreBackendCapabilityProfile,
    needs_overlay: bool,
    identity: StoreOpenIdentity<'_>,
) -> Result<StoreOpenDecision, StoreOpenRejection<'a>> {
    if needs_overlay && !profile.overlay_composition {
        return Err(StoreOpenRejection::OverlayUnsupported);
    }
    match observed {
        ObservedStoreIdentity::Recorded(actual) => {
            let actual_backend = match (actual.schema.as_str(), actual.backend.as_deref()) {
                (STORE_STATE_SCHEMA, None) => StoreBackend::Snix,
                (STORE_BACKEND_STATE_SCHEMA, Some("snix")) => StoreBackend::Snix,
                (STORE_BACKEND_STATE_SCHEMA, Some("casita")) => StoreBackend::Casita,
                _ => return Err(StoreOpenRejection::UnsupportedIdentity),
            };
            if actual_backend != backend {
                return Err(StoreOpenRejection::BackendMismatch(actual_backend));
            }
            if actual.logical_prefix != identity.logical_prefix || actual.trust_policy_id != identity.trust_policy_id {
                return Err(StoreOpenRejection::IdentityMismatch {
                    observed_prefix: &actual.logical_prefix,
                    observed_trust: &actual.trust_policy_id,
                });
            }
            Ok(StoreOpenDecision::Reuse)
        }
        ObservedStoreIdentity::Identityless {
            casita_marker,
            identity_filename,
            other_content,
        } => {
            if *casita_marker || *identity_filename || (backend == StoreBackend::Casita && *other_content) {
                return Err(StoreOpenRejection::IdentitylessContent);
            }
            Ok(StoreOpenDecision::BindNew)
        }
    }
}

fn store_open_rejection_error(
    rejection: StoreOpenRejection<'_>,
    state_dir: &Path,
    identity: StoreOpenIdentity<'_>,
    backend: StoreBackend,
) -> Error {
    match rejection {
        StoreOpenRejection::UnsupportedIdentity => {
            Error::Store("store-backend-mismatch: unsupported store identity schema or backend".to_string())
        }
        StoreOpenRejection::BackendMismatch(actual) => Error::Store(format!(
            "store-backend-mismatch: requested {}, state declares {}",
            backend.as_str(),
            actual.as_str(),
        )),
        StoreOpenRejection::IdentityMismatch {
            observed_prefix,
            observed_trust,
        } => Error::Store(format!(
            "store-identity-mismatch: requested prefix={} trust={}, observed prefix={observed_prefix} trust={observed_trust}",
            identity.logical_prefix, identity.trust_policy_id,
        )),
        StoreOpenRejection::IdentitylessContent => Error::Store(format!(
            "store-backend-mismatch: state directory {} has content but no backend identity",
            state_dir.display(),
        )),
        StoreOpenRejection::OverlayUnsupported => {
            Error::Store(format!("{}-overlay-unsupported: backend cannot compose base stores", backend.as_str(),))
        }
    }
}

fn inspect_store_open(
    state_dir: &Path,
    logical_prefix: &str,
    backend: StoreBackend,
    profile: StoreBackendCapabilityProfile,
    needs_overlay: bool,
) -> Result<(StoreOpenDecision, StoreOverlayRuntimePolicy), Error> {
    let policy = store_overlay_runtime_policy()?;
    let observed = observe_store_identity(state_dir, policy.limits.max_descriptor_bytes)?;
    let identity = StoreOpenIdentity {
        logical_prefix,
        trust_policy_id: &policy.trust.policy_id,
    };
    let decision = decide_store_open(&observed, backend, profile, needs_overlay, identity)
        .map_err(|rejection| store_open_rejection_error(rejection, state_dir, identity, backend))?;
    Ok((decision, policy))
}

/// Validate backend and logical identity without creating or mutating state.
pub(crate) fn preflight_store_identity(
    state_dir: &Path,
    logical_prefix: &str,
    backend: StoreBackend,
    profile: StoreBackendCapabilityProfile,
    needs_overlay: bool,
) -> Result<(), Error> {
    inspect_store_open(state_dir, logical_prefix, backend, profile, needs_overlay).map(|_| ())
}

pub(crate) fn ensure_store_identity(
    state_dir: &Path,
    logical_prefix: &str,
    backend: StoreBackend,
) -> Result<(), Error> {
    let (decision, policy) = inspect_store_open(state_dir, logical_prefix, backend, backend.profile(), false)?;
    if decision == StoreOpenDecision::Reuse {
        return Ok(());
    }
    let path = state_dir.join(STORE_IDENTITY_FILE_NAME);
    let expected = StoreIdentityRecord {
        schema: STORE_BACKEND_STATE_SCHEMA.to_string(),
        logical_prefix: logical_prefix.to_string(),
        trust_policy_id: policy.trust.policy_id,
        backend: Some(backend.as_str().to_string()),
    };
    let bytes = serde_json::to_vec_pretty(&expected)
        .map_err(|error| Error::Store(format!("serializing store identity: {error}")))?;
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| Error::Store("store identity byte count does not fit u64".to_string()))?;
    if byte_count > policy.limits.max_descriptor_bytes {
        return Err(Error::Store(format!("store identity exceeds {} bytes", policy.limits.max_descriptor_bytes)));
    }
    let mut temporary = tempfile::NamedTempFile::new_in(state_dir).map_err(|error| {
        Error::Store(format!("creating temporary store identity in {}: {error}", state_dir.display()))
    })?;
    temporary
        .write_all(&bytes)
        .map_err(|error| Error::Store(format!("writing temporary store identity: {error}")))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| Error::Store(format!("syncing temporary store identity: {error}")))?;
    match temporary.persist_noclobber(&path) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            preflight_store_identity(state_dir, logical_prefix, backend, backend.profile(), false)
        }
        Err(error) => Err(Error::Store(format!("publishing {}: {}", path.display(), error.error))),
    }?;
    Ok(())
}

pub(crate) fn prepare_overlay_state(
    base_state_dirs: &[PathBuf],
    logical_prefix: &str,
) -> Result<StoreOverlayState, Error> {
    let policy = store_overlay_runtime_policy()?;
    let core_policy = core_policy(&policy)?;
    let observations = observe_bases(base_state_dirs, &policy)?;
    let plan = crunch_overlay_core::plan_overlay(&core_policy, logical_prefix, observations)
        .map_err(|error| overlay_core_error("planning overlay composition", error))?;
    Ok(StoreOverlayState {
        plan,
        base_state_dirs: base_state_dirs.to_vec(),
    })
}

pub(crate) fn revalidate_overlay_state(state: &StoreOverlayState) -> Result<(), Error> {
    let observed = prepare_overlay_state(&state.base_state_dirs, &state.plan.logical_prefix)?;
    crunch_overlay_core::revalidate_overlay(&state.plan, &observed.plan)
        .map_err(|error| overlay_core_error("revalidating overlay composition", error))
}

pub(crate) fn overlay_report(state: &StoreOverlayState) -> Result<StoreOverlayReport, Error> {
    let policy = store_overlay_runtime_policy()?;
    if state.plan.descriptors.len() != state.base_state_dirs.len() {
        return Err(Error::Store("overlay report descriptor and base counts differ".to_string()));
    }
    let bases = state
        .plan
        .descriptors
        .iter()
        .zip(&state.base_state_dirs)
        .map(|(descriptor, base_state_dir)| {
            let accepted_signer_names =
                load_layer_trust_keys(base_state_dir)?.into_iter().map(|key| key.name().to_string()).collect();
            Ok(StoreOverlayBaseReport {
                declaration_index: descriptor.declaration_index,
                declaration_identity_blake3: descriptor.declaration_identity.clone(),
                accepted_signer_names,
                descriptor_blake3: format_digest(&descriptor.descriptor_identity.into_bytes()),
                generation_blake3: format_digest(&descriptor.generation_identity.into_bytes()),
                state_schema: descriptor.state_schema.clone(),
                trust_policy_id: descriptor.trust_policy_id.clone(),
                capability_class: descriptor.capability_class.as_str().to_string(),
                observed_member_count: descriptor.observed_member_count,
                observed_bytes: descriptor.observed_bytes,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(StoreOverlayReport {
        schema: STORE_OVERLAY_REPORT_SCHEMA.to_string(),
        policy_blake3: store_overlay_policy_blake3(),
        plan_blake3: format_digest(&state.plan.plan_identity.into_bytes()),
        logical_prefix: state.plan.logical_prefix.clone(),
        writable_overlay_count: policy.composition.writable_overlay_count,
        base_order: policy.composition.base_order,
        no_backfill: policy.composition.no_backfill,
        bases,
        non_claims: vec![
            "overlay-report-does-not-prove-base-immutability".to_string(),
            "base-generation-does-not-prove-whole-database-atomicity".to_string(),
            "composition-report-does-not-authorize-base-mutation".to_string(),
        ],
    })
}

pub(crate) fn trusted_base_pathinfo_service(
    inner: Arc<dyn PathInfoService>,
    base_state_dir: &Path,
    declaration_index: usize,
) -> Result<Arc<dyn PathInfoService>, Error> {
    let trusted_keys = load_layer_trust_keys(base_state_dir)?;
    Ok(Arc::new(TrustedBasePathInfoService {
        inner,
        trusted_keys,
        declaration_index,
    }))
}

pub(crate) fn verify_pathinfo_trust(path_info: &PathInfo, trusted_keys: &[VerifyingKey]) -> Result<(), String> {
    if trusted_keys.is_empty() {
        return Err("layer trust policy has no accepted keys".to_string());
    }
    let references: Vec<_> = path_info.references.iter().map(|reference| reference.as_ref()).collect();
    let signed_fingerprint =
        fingerprint(&path_info.store_path.as_ref(), &path_info.nar_sha256, path_info.nar_size, references.iter());
    let is_trusted = path_info
        .signatures
        .iter()
        .any(|signature| trusted_keys.iter().any(|key| key.verify(&signed_fingerprint, &signature.as_ref())));
    if !is_trusted {
        return Err(format!("{} has no valid signature from its layer trust policy", path_info.store_path));
    }
    Ok(())
}

pub(crate) fn load_layer_trust_keys(state_dir: &Path) -> Result<Vec<VerifyingKey>, Error> {
    let policy = store_overlay_runtime_policy()?;
    let public_keys_path = state_dir.join(OVERLAY_TRUSTED_PUBLIC_KEYS_FILE_NAME);
    let mut keys = if public_keys_path.exists() {
        parse_public_keys(&public_keys_path, policy.limits.max_descriptor_bytes)?
    } else {
        vec![load_local_verifying_key(state_dir)?.ok_or_else(|| {
            Error::Store("overlay-layer-trust-key-missing: no signing key or public trust policy".to_string())
        })?]
    };
    keys.sort_by_cached_key(ToString::to_string);
    keys.dedup();
    if keys.is_empty() || keys.len() > MAX_LAYER_TRUST_KEYS {
        return Err(Error::Store(format!(
            "overlay-layer-trust-key-limit: {} keys are outside 1..={MAX_LAYER_TRUST_KEYS}",
            keys.len()
        )));
    }
    Ok(keys)
}
/// Use the same bounded, comment-aware public-key syntax for backend trust.
pub(crate) fn parse_public_keys(path: &Path, maximum_bytes: u64) -> Result<Vec<VerifyingKey>, Error> {
    let bytes = read_file_bounded(path, maximum_bytes)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| Error::Store(format!("parsing {} as UTF-8: {error}", path.display())))?;
    let mut keys = Vec::new();
    for line in text.lines() {
        let value = line.split_once('#').map_or(line, |(prefix, _)| prefix);
        for encoded_key in value.split(|character: char| character.is_whitespace() || character == ',') {
            if encoded_key.is_empty() {
                continue;
            }
            if keys.len() >= MAX_LAYER_TRUST_KEYS {
                return Err(Error::Store(format!(
                    "overlay-layer-trust-key-limit: {} exceeds {MAX_LAYER_TRUST_KEYS}",
                    keys.len().saturating_add(1)
                )));
            }
            keys.push(
                VerifyingKey::parse(encoded_key)
                    .map_err(|error| Error::Store(format!("parsing layer trust key in {}: {error}", path.display())))?,
            );
        }
    }
    Ok(keys)
}

pub(crate) fn load_local_verifying_key(state_dir: &Path) -> Result<Option<VerifyingKey>, Error> {
    let signing_key_path = state_dir.join(LOCAL_SIGNING_KEY_FILE_NAME);
    if !signing_key_path.exists() {
        return Ok(None);
    }
    let policy = store_overlay_runtime_policy()?;
    let bytes = read_file_bounded(&signing_key_path, policy.limits.max_descriptor_bytes)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| Error::Store(format!("parsing {} as UTF-8: {error}", signing_key_path.display())))?;
    let (_, verifying_key) = parse_keypair(text.trim())
        .map_err(|error| Error::Store(format!("parsing layer signing key {}: {error}", signing_key_path.display())))?;
    Ok(Some(verifying_key))
}

struct TrustedBasePathInfoService {
    inner: Arc<dyn PathInfoService>,
    trusted_keys: Vec<VerifyingKey>,
    declaration_index: usize,
}

#[async_trait]
impl PathInfoService for TrustedBasePathInfoService {
    async fn get(
        &self,
        digest: [u8; nix_compat::store_path::DIGEST_SIZE],
    ) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
        let value = self.inner.get(digest).await?;
        if let Some(path_info) = value.as_ref() {
            verify_pathinfo_trust(path_info, &self.trusted_keys).map_err(|error| {
                std::io::Error::other(format!(
                    "overlay-layer-trust-failure in base {}: {error}",
                    self.declaration_index
                ))
            })?;
        }
        Ok(value)
    }

    async fn put(&self, _path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("base {} PathInfo service is read-only", self.declaration_index),
        )
        .into())
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
        let trusted_keys = self.trusted_keys.clone();
        let declaration_index = self.declaration_index;
        self.inner
            .list()
            .map(move |result| {
                let path_info = result?;
                verify_pathinfo_trust(&path_info, &trusted_keys).map_err(|error| {
                    std::io::Error::other(format!("overlay-layer-trust-failure in base {declaration_index}: {error}"))
                })?;
                Ok(path_info)
            })
            .boxed()
    }
}

fn validate_runtime_policy(policy: &StoreOverlayRuntimePolicy) -> Result<(), Error> {
    let valid = policy.schema == STORE_OVERLAY_POLICY_SCHEMA
        && policy.hash_algorithm == "BLAKE3"
        && !policy.policy_name.is_empty()
        && !policy.trust.policy_id.is_empty()
        && policy.composition.writable_overlay_count == 1
        && policy.composition.base_order == "declaration-order"
        && policy.composition.base_capability == "read-only"
        && policy.composition.require_same_prefix
        && policy.composition.reject_duplicate_bases
        && policy.composition.no_backfill
        && policy.trust.verification == "ed25519-pathinfo"
        && policy.trust.unknown_policy == "reject"
        && policy.trust.shadow_failure == "block"
        && policy.trust.skip_failure_classes.is_empty()
        && policy.generation.bind_root_inventory
        && policy.generation.bind_pathinfo
        && policy.generation.bind_directories
        && policy.generation.bind_blobs
        && policy.generation.revalidate_before_execution
        && policy.generation.revalidate_before_output_admission;
    if !valid {
        return Err(Error::Store("embedded store overlay policy violates required fail-closed invariants".to_string()));
    }
    Ok(())
}

fn core_policy(policy: &StoreOverlayRuntimePolicy) -> Result<OverlayPolicy, Error> {
    let max_descriptor_bytes = usize::try_from(policy.limits.max_descriptor_bytes)
        .map_err(|_| Error::Store("overlay descriptor byte limit does not fit usize".to_string()))?;
    if max_descriptor_bytes == 0 {
        return Err(Error::Store("overlay descriptor byte limit must be positive".to_string()));
    }
    Ok(OverlayPolicy {
        policy_id: store_overlay_policy_blake3(),
        trust_policy_id: policy.trust.policy_id.clone(),
        allowed_state_schemas: policy.generation.allowed_state_schemas.clone(),
        max_base_layers: policy.limits.max_base_layers,
        max_generation_members: policy.limits.max_generation_members,
        max_generation_bytes: policy.limits.max_generation_bytes,
        require_same_prefix: policy.composition.require_same_prefix,
        require_read_only_bases: policy.composition.base_capability == "read-only",
        reject_duplicate_bases: policy.composition.reject_duplicate_bases,
        no_backfill: policy.composition.no_backfill,
        revalidate_before_execution: policy.generation.revalidate_before_execution,
        revalidate_before_output_admission: policy.generation.revalidate_before_output_admission,
    })
}

fn observe_bases(
    base_state_dirs: &[PathBuf],
    policy: &StoreOverlayRuntimePolicy,
) -> Result<Vec<BaseObservation>, Error> {
    if base_state_dirs.len() > policy.limits.max_base_layers {
        return Err(Error::Store(format!(
            "overlay-layer-limit: {} bases exceed policy limit {}",
            base_state_dirs.len(),
            policy.limits.max_base_layers
        )));
    }
    let mut observations = Vec::with_capacity(base_state_dirs.len());
    for (declaration_index, base_state_dir) in base_state_dirs.iter().enumerate() {
        observations.push(observe_base(declaration_index, base_state_dir, policy)?);
    }
    Ok(observations)
}

fn observe_base(
    declaration_index: usize,
    base_state_dir: &Path,
    policy: &StoreOverlayRuntimePolicy,
) -> Result<BaseObservation, Error> {
    let metadata = fs::symlink_metadata(base_state_dir)
        .map_err(|error| Error::Store(format!("opening overlay base {}: {error}", base_state_dir.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::Store(format!(
            "overlay-base-not-directory: declaration {declaration_index} is not a direct directory"
        )));
    }
    ensure_read_only_member(declaration_index, base_state_dir, &metadata)?;
    let canonical = fs::canonicalize(base_state_dir)
        .map_err(|error| Error::Store(format!("resolving overlay base {}: {error}", base_state_dir.display())))?;
    let declaration_identity = declaration_identity(&canonical)?;
    let identity_path = canonical.join(STORE_IDENTITY_FILE_NAME);
    let identity = read_store_identity(&identity_path, policy.limits.max_descriptor_bytes)?;
    let members = observe_generation_members(declaration_index, &canonical, policy)?;
    load_layer_trust_keys(&canonical)?;
    Ok(BaseObservation {
        declaration_index,
        declaration_identity,
        logical_prefix: identity.logical_prefix,
        state_schema: identity.schema,
        trust_policy_id: identity.trust_policy_id,
        capability_class: BaseCapabilityClass::ReadOnly,
        members,
    })
}

fn read_store_identity(path: &Path, maximum_bytes: u64) -> Result<StoreIdentityRecord, Error> {
    let bytes = read_file_bounded(path, maximum_bytes)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| Error::Store(format!("parsing store identity {}: {error}", path.display())))
}

fn observe_generation_members(
    declaration_index: usize,
    root: &Path,
    policy: &StoreOverlayRuntimePolicy,
) -> Result<Vec<GenerationMember>, Error> {
    let mut members = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut observed_bytes = 0_u64;
    while let Some(directory) = stack.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| Error::Store(format!("reading overlay base directory {}: {error}", directory.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                Error::Store(format!("enumerating overlay base directory {}: {error}", directory.display()))
            })?;
        children.sort_by_cached_key(std::fs::DirEntry::file_name);
        for child in children.into_iter().rev() {
            if members.len() >= policy.limits.max_generation_members {
                return Err(Error::Store(format!(
                    "overlay-generation-member-limit: declaration exceeds {} members",
                    policy.limits.max_generation_members
                )));
            }
            let path = child.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|error| Error::Store(format!("relativizing overlay generation path: {error}")))?;
            let relative_path = relative
                .to_str()
                .ok_or_else(|| Error::Store("overlay generation path is not UTF-8".to_string()))?
                .to_string();
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                Error::Store(format!("observing overlay generation member {}: {error}", path.display()))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(Error::Store(format!(
                    "overlay-generation-symlink: declaration contains symlink {relative_path}"
                )));
            }
            ensure_read_only_member(declaration_index, &path, &metadata)?;
            if metadata.is_dir() {
                members.push(GenerationMember {
                    relative_path,
                    kind: GenerationMemberKind::Directory,
                    bytes: 0,
                    content_blake3: *blake3::hash(DIRECTORY_IDENTITY_DOMAIN).as_bytes(),
                });
                stack.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(Error::Store(format!(
                    "overlay-generation-special-file: declaration contains unsupported member {relative_path}"
                )));
            }
            observed_bytes = observed_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| Error::Store("overlay generation observed-byte count overflow".to_string()))?;
            if observed_bytes > policy.limits.max_generation_bytes {
                return Err(Error::Store(format!(
                    "overlay-generation-byte-limit: declaration exceeds {} bytes",
                    policy.limits.max_generation_bytes
                )));
            }
            let digest = hash_file(&path, metadata.len())?;
            members.push(GenerationMember {
                relative_path,
                kind: GenerationMemberKind::File,
                bytes: metadata.len(),
                content_blake3: digest,
            });
        }
    }
    members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(members)
}

fn ensure_read_only_member(declaration_index: usize, path: &Path, metadata: &fs::Metadata) -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        const WRITE_PERMISSION_BITS: u32 = 0o222;
        if metadata.permissions().mode() & WRITE_PERMISSION_BITS != 0 {
            return Err(Error::Store(format!(
                "overlay-base-writable: declaration {declaration_index} member {} has write permission",
                path.display()
            )));
        }
    }
    #[cfg(not(unix))]
    if !metadata.permissions().readonly() {
        return Err(Error::Store(format!(
            "overlay-base-writable: declaration {declaration_index} member {} is not read-only",
            path.display()
        )));
    }
    Ok(())
}

fn hash_file(path: &Path, expected_bytes: u64) -> Result<[u8; blake3::OUT_LEN], Error> {
    let mut file = File::open(path)
        .map_err(|error| Error::Store(format!("opening overlay generation member {}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_BYTES];
    let mut observed_bytes = 0_u64;
    let file_len = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let mut remaining_bytes = file_len;
    loop {
        let read_count = file
            .read(&mut buffer)
            .map_err(|error| Error::Store(format!("reading overlay generation member {}: {error}", path.display())))?;
        remaining_bytes = remaining_bytes.saturating_sub(u64::try_from(read_count).unwrap_or(0));
        if read_count == 0 {
            break;
        }
        observed_bytes = observed_bytes
            .checked_add(
                u64::try_from(read_count)
                    .map_err(|_| Error::Store("overlay generation read count does not fit u64".to_string()))?,
            )
            .ok_or_else(|| Error::Store("overlay generation read count overflow".to_string()))?;
        hasher.update(&buffer[..read_count]);
    }
    if observed_bytes != expected_bytes {
        return Err(Error::Store(format!("overlay-generation-race: {} changed size while observed", path.display())));
    }
    Ok(*hasher.finalize().as_bytes())
}

fn read_file_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, Error> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| Error::Store(format!("observing {}: {error}", path.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::Store(format!("overlay descriptor {} must be a direct regular file", path.display())));
    }
    let read_limit = maximum_bytes
        .checked_add(1)
        .ok_or_else(|| Error::Store("bounded file read limit overflow".to_string()))?;
    let file = File::open(path).map_err(|error| Error::Store(format!("opening {}: {error}", path.display())))?;
    let mut bytes = Vec::new();
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Store(format!("reading {}: {error}", path.display())))?;
    let observed = u64::try_from(bytes.len())
        .map_err(|_| Error::Store(format!("{} byte count does not fit u64", path.display())))?;
    if observed > maximum_bytes {
        return Err(Error::Store(format!("{} exceeds {maximum_bytes} bytes", path.display())));
    }
    Ok(bytes)
}

fn declaration_identity(path: &Path) -> Result<String, Error> {
    let value = path.to_str().ok_or_else(|| Error::Store("overlay base canonical path is not UTF-8".to_string()))?;
    Ok(format_digest(blake3::hash(value.as_bytes()).as_bytes()))
}

fn overlay_core_error(context: &str, error: crunch_overlay_core::OverlayError) -> Error {
    Error::Store(format!("{context}: {error:?}"))
}

fn format_digest(bytes: &[u8; blake3::OUT_LEN]) -> String {
    format!("b3:{}", HEXLOWER.encode(bytes))
}

#[cfg(test)]
mod tests {
    use snix_store::nar::NarCalculationService;

    use super::*;

    fn write_test_trust_policy(root: &Path) {
        let raw = ed25519_dalek::SigningKey::from_bytes(&[7_u8; ed25519_dalek::SECRET_KEY_LENGTH]);
        let key = VerifyingKey::new("overlay-policy-test-1".to_string(), raw.verifying_key());
        std::fs::write(root.join(OVERLAY_TRUSTED_PUBLIC_KEYS_FILE_NAME), format!("{key}\n")).unwrap();
    }

    fn make_test_tree_read_only(root: &Path) {
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            if metadata.file_type().is_symlink() {
                continue;
            }
            let mut permissions = metadata.permissions();
            permissions.set_readonly(true);
            std::fs::set_permissions(path, permissions).unwrap();
        }
        let mut permissions = std::fs::metadata(root).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(root, permissions).unwrap();
    }

    fn snapshot_state_tree(root: &Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
        let mut entries = std::collections::BTreeMap::new();
        let mut directories = vec![root.to_path_buf()];
        while let Some(directory) = directories.pop() {
            for child in std::fs::read_dir(directory).unwrap() {
                let child = child.unwrap();
                let path = child.path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                let kind = child.file_type().unwrap();
                let contents = if kind.is_dir() {
                    directories.push(path);
                    None
                } else {
                    assert!(kind.is_file(), "unexpected state member {}", relative.display());
                    Some(std::fs::read(path).unwrap())
                };
                assert!(entries.insert(relative, contents).is_none());
            }
        }
        entries
    }

    #[test]
    fn embedded_overlay_policy_is_typed_and_bounded() {
        let policy = store_overlay_runtime_policy().expect("embedded overlay policy must parse");
        assert_eq!(policy.schema, STORE_OVERLAY_POLICY_SCHEMA);
        assert_eq!(policy.hash_algorithm, "BLAKE3");
        assert!(policy.limits.max_base_layers <= crunch_overlay_core::MAX_BASE_LAYERS);
        assert!(policy.limits.max_generation_members <= crunch_overlay_core::MAX_GENERATION_MEMBERS);
        assert!(policy.limits.max_generation_bytes <= crunch_overlay_core::MAX_GENERATION_BYTES);
        assert!(store_overlay_policy_blake3().starts_with("b3:"));
    }

    #[test]
    fn pure_open_decision_covers_recorded_identity_and_identityless_markers() {
        const PREFIX: &str = "/nix/store";
        const TRUST: &str = "test-policy";
        let identityless = |casita_marker, identity_filename, other_content| ObservedStoreIdentity::Identityless {
            casita_marker,
            identity_filename,
            other_content,
        };
        let recorded = |schema: &str, backend: Option<&str>, prefix: &str, trust: &str| {
            ObservedStoreIdentity::Recorded(StoreIdentityRecord {
                schema: schema.to_string(),
                logical_prefix: prefix.to_string(),
                trust_policy_id: trust.to_string(),
                backend: backend.map(str::to_string),
            })
        };
        let no_overlay = StoreBackendCapabilityProfile {
            overlay_composition: false,
            ..StoreBackend::Snix.profile()
        };
        let cases = [
            (
                "empty snix",
                identityless(false, false, false),
                StoreBackend::Snix,
                false,
                None,
                Ok(StoreOpenDecision::BindNew),
            ),
            (
                "empty casita",
                identityless(false, false, false),
                StoreBackend::Casita,
                false,
                None,
                Ok(StoreOpenDecision::BindNew),
            ),
            (
                "unclaimed snix bytes",
                identityless(false, false, true),
                StoreBackend::Snix,
                false,
                None,
                Ok(StoreOpenDecision::BindNew),
            ),
            (
                "unclaimed casita bytes",
                identityless(false, false, true),
                StoreBackend::Casita,
                false,
                None,
                Err(StoreOpenRejection::IdentitylessContent),
            ),
            (
                "casita marker under snix",
                identityless(true, false, true),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::IdentitylessContent),
            ),
            (
                "casita marker under casita",
                identityless(true, false, true),
                StoreBackend::Casita,
                false,
                None,
                Err(StoreOpenRejection::IdentitylessContent),
            ),
            (
                "identity name without readable record",
                identityless(false, true, false),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::IdentitylessContent),
            ),
            (
                "legacy snix",
                recorded(STORE_STATE_SCHEMA, None, PREFIX, TRUST),
                StoreBackend::Snix,
                false,
                None,
                Ok(StoreOpenDecision::Reuse),
            ),
            (
                "legacy is not casita",
                recorded(STORE_STATE_SCHEMA, None, PREFIX, TRUST),
                StoreBackend::Casita,
                false,
                None,
                Err(StoreOpenRejection::BackendMismatch(StoreBackend::Snix)),
            ),
            (
                "recorded snix",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("snix"), PREFIX, TRUST),
                StoreBackend::Snix,
                false,
                None,
                Ok(StoreOpenDecision::Reuse),
            ),
            (
                "recorded casita",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("casita"), PREFIX, TRUST),
                StoreBackend::Casita,
                false,
                None,
                Ok(StoreOpenDecision::Reuse),
            ),
            (
                "wrong backend",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("casita"), PREFIX, TRUST),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::BackendMismatch(StoreBackend::Casita)),
            ),
            (
                "missing v2 backend",
                recorded(STORE_BACKEND_STATE_SCHEMA, None, PREFIX, TRUST),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::UnsupportedIdentity),
            ),
            (
                "legacy with backend",
                recorded(STORE_STATE_SCHEMA, Some("snix"), PREFIX, TRUST),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::UnsupportedIdentity),
            ),
            (
                "prefix drift",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("snix"), "/other/store", TRUST),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::IdentityMismatch {
                    observed_prefix: "/other/store",
                    observed_trust: TRUST,
                }),
            ),
            (
                "trust drift",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("snix"), PREFIX, "other-policy"),
                StoreBackend::Snix,
                false,
                None,
                Err(StoreOpenRejection::IdentityMismatch {
                    observed_prefix: PREFIX,
                    observed_trust: "other-policy",
                }),
            ),
            (
                "overlay disallowed",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("snix"), PREFIX, TRUST),
                StoreBackend::Snix,
                true,
                Some(no_overlay),
                Err(StoreOpenRejection::OverlayUnsupported),
            ),
            (
                "overlay allowed",
                recorded(STORE_BACKEND_STATE_SCHEMA, Some("snix"), PREFIX, TRUST),
                StoreBackend::Snix,
                true,
                None,
                Ok(StoreOpenDecision::Reuse),
            ),
        ];
        for (name, observation, backend, needs_overlay, profile, expected) in cases {
            let actual = decide_store_open(
                &observation,
                backend,
                profile.unwrap_or_else(|| backend.profile()),
                needs_overlay,
                StoreOpenIdentity {
                    logical_prefix: PREFIX,
                    trust_policy_id: TRUST,
                },
            );
            assert_eq!(actual, expected, "{name}");
        }
    }

    #[test]
    fn identity_record_rejects_prefix_drift() {
        let state = tempfile::tempdir().unwrap();
        ensure_store_identity(state.path(), "/nix/store", StoreBackend::Snix).unwrap();
        let error = ensure_store_identity(state.path(), "/mantle/store", StoreBackend::Snix).unwrap_err();
        assert!(error.to_string().contains("store-identity-mismatch"));
    }

    #[test]
    fn legacy_v1_identity_remains_snix_only_without_backend_field() {
        let state = tempfile::tempdir().unwrap();
        let policy = store_overlay_runtime_policy().unwrap();
        let legacy = serde_json::json!({
            "schema": STORE_STATE_SCHEMA,
            "logical_prefix": "/nix/store",
            "trust_policy_id": policy.trust.policy_id,
        });
        std::fs::write(state.path().join(STORE_IDENTITY_FILE_NAME), serde_json::to_vec(&legacy).unwrap()).unwrap();
        preflight_store_identity(state.path(), "/nix/store", StoreBackend::Snix, StoreBackend::Snix.profile(), false)
            .unwrap();
        assert!(
            preflight_store_identity(
                state.path(),
                "/nix/store",
                StoreBackend::Casita,
                StoreBackend::Casita.profile(),
                false
            )
            .unwrap_err()
            .to_string()
            .contains("store-backend-mismatch")
        );
    }

    #[test]
    fn backend_identity_cannot_be_reopened_under_another_backend() {
        for (recorded, requested) in [
            (StoreBackend::Snix, StoreBackend::Casita),
            (StoreBackend::Casita, StoreBackend::Snix),
        ] {
            let state = tempfile::tempdir().unwrap();
            ensure_store_identity(state.path(), "/nix/store", recorded).unwrap();
            let identity_path = state.path().join(STORE_IDENTITY_FILE_NAME);
            let original = std::fs::read(&identity_path).unwrap();
            let error = preflight_store_identity(state.path(), "/nix/store", requested, requested.profile(), false)
                .unwrap_err();
            assert!(error.to_string().contains("store-backend-mismatch"));
            assert!(
                ensure_store_identity(state.path(), "/nix/store", requested)
                    .unwrap_err()
                    .to_string()
                    .contains("store-backend-mismatch")
            );
            assert_eq!(std::fs::read(identity_path).unwrap(), original);
        }
    }

    #[tokio::test]
    async fn legacy_snix_identityless_populated_store_reopens_without_rewriting_existing_content() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("store");
        let config =
            || crate::StoreConfig::new(StoreBackend::Snix, state.clone(), output.clone(), "/nix/store".to_string());
        let first = crate::StoreHandle::open(config()).await.unwrap();
        assert_eq!(first.backend(), StoreBackend::Snix);
        assert!(state.join("pathinfo.redb").is_file());
        assert!(state.join("directories.redb").is_file());
        assert!(state.join("blobs").is_dir());
        let content = b"legacy Snix output remains readable";
        let mut blob = first.blob_service().open_write().await;
        tokio::io::AsyncWriteExt::write_all(&mut blob, content).await.unwrap();
        let digest = blob.close().await.unwrap();
        let node = snix_castore::Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        };
        let (nar_size, nar_sha256) =
            snix_store::nar::SimpleRenderer::new(first.blob_service(), first.directory_service())
                .calculate_nar(&node)
                .await
                .unwrap();
        let path_info = PathInfo {
            store_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("legacy-output", [23_u8; 20])
                .unwrap(),
            node,
            references: Vec::new(),
            nar_size,
            nar_sha256,
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        first.pathinfo_service().put(path_info.clone()).await.unwrap();
        drop(first);
        let legacy_file = state.join("legacy-observation.log");
        let legacy_bytes = b"legacy Snix state must remain byte-identical\n";
        std::fs::write(&legacy_file, legacy_bytes).unwrap();
        std::fs::remove_file(state.join(STORE_IDENTITY_FILE_NAME)).unwrap();
        let before = snapshot_state_tree(&state);

        preflight_store_identity(&state, "/nix/store", StoreBackend::Snix, StoreBackend::Snix.profile(), false)
            .unwrap();
        assert_eq!(snapshot_state_tree(&state), before, "preflight modified legacy state");
        ensure_store_identity(&state, "/nix/store", StoreBackend::Snix).unwrap();
        let mut claimed = snapshot_state_tree(&state);
        let identity_bytes = claimed.remove(Path::new(STORE_IDENTITY_FILE_NAME)).unwrap().unwrap();
        let identity: StoreIdentityRecord = serde_json::from_slice(&identity_bytes).unwrap();
        assert_eq!(identity.schema, STORE_BACKEND_STATE_SCHEMA);
        assert_eq!(identity.backend.as_deref(), Some("snix"));
        assert_eq!(claimed, before, "identity claim rewrote an existing Snix state member");
        // Exercise the public open path on the original unclaimed state too.
        std::fs::remove_file(state.join(STORE_IDENTITY_FILE_NAME)).unwrap();

        let reopened = crate::StoreHandle::open(config()).await.unwrap();
        assert_eq!(reopened.backend(), StoreBackend::Snix);
        let reopened_identity: StoreIdentityRecord =
            serde_json::from_slice(&std::fs::read(state.join(STORE_IDENTITY_FILE_NAME)).unwrap()).unwrap();
        assert_eq!(reopened_identity.schema, STORE_BACKEND_STATE_SCHEMA);
        assert_eq!(reopened_identity.backend.as_deref(), Some("snix"));
        assert_eq!(std::fs::read(legacy_file).unwrap(), legacy_bytes);
        assert_eq!(
            reopened.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap(),
            Some(path_info),
            "Snix must retain the previously admitted PathInfo",
        );
    }

    #[tokio::test]
    async fn identityless_casita_marker_blocks_any_backend_without_mutation() {
        for mixed_with_snix in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let state = root.path().join("state");
            std::fs::create_dir_all(state.join("casita")).unwrap();
            std::fs::write(state.join("casita").join("root"), b"preexisting Casita repository bytes").unwrap();
            if mixed_with_snix {
                std::fs::write(state.join("pathinfo.redb"), b"preexisting Snix database bytes").unwrap();
            }
            let before = snapshot_state_tree(&state);

            for backend in [StoreBackend::Snix, StoreBackend::Casita] {
                let output = root.path().join(format!("store-{}", backend.as_str()));
                let config = crate::StoreConfig::new(backend, state.clone(), output.clone(), "/nix/store".to_string());
                let error =
                    crate::StoreHandle::open(config).await.err().expect("unclaimed Casita repository must fail");
                assert!(error.to_string().contains("store-backend-mismatch"), "{backend:?}: {error}");
                assert_eq!(snapshot_state_tree(&state), before, "{backend:?} changed existing state");
                assert!(!output.exists(), "{backend:?} created output state");
            }
        }
    }

    #[tokio::test]
    async fn identityless_unknown_content_blocks_casita_without_mutation() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        std::fs::create_dir(&state).unwrap();
        std::fs::write(state.join("legacy-observation.log"), b"unclaimed data").unwrap();
        let before = snapshot_state_tree(&state);
        let output = root.path().join("store");
        let config =
            crate::StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), "/nix/store".to_string());

        let error = crate::StoreHandle::open(config).await.err().expect("Casita must reject unclaimed state");
        assert!(error.to_string().contains("store-backend-mismatch"), "{error}");
        assert_eq!(snapshot_state_tree(&state), before, "Casita changed unclaimed state");
        assert!(!output.exists(), "Casita created output state");
    }

    #[test]
    fn generation_observation_rejects_symlink_members() {
        let base = tempfile::tempdir().unwrap();
        ensure_store_identity(base.path(), "/nix/store", StoreBackend::Snix).unwrap();
        write_test_trust_policy(base.path());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(STORE_IDENTITY_FILE_NAME, base.path().join("alias")).unwrap();
            make_test_tree_read_only(base.path());
            let policy = store_overlay_runtime_policy().unwrap();
            let error = observe_base(0, base.path(), &policy).unwrap_err();
            assert!(error.to_string().contains("overlay-generation-symlink"));
        }
    }

    #[test]
    fn writable_base_is_rejected_before_generation_admission() {
        let base = tempfile::tempdir().unwrap();
        ensure_store_identity(base.path(), "/nix/store", StoreBackend::Snix).unwrap();
        write_test_trust_policy(base.path());
        let policy = store_overlay_runtime_policy().unwrap();

        let error = observe_base(0, base.path(), &policy).unwrap_err();

        assert!(error.to_string().contains("overlay-base-writable"));
    }
}
