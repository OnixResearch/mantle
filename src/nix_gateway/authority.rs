//! Owner-private production authority loading. No ambient issuer/key trust,
//! replay bypass, credential provisioning or permissive revocation fallback.
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read as _;
use std::os::unix::fs::MetadataExt as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use basalt::authority_core::BasaltPolicyRef;
use basalt::mantle;
pub use basalt::mantle::Denied as Error;
pub use basalt::mantle::Invocation;
pub use basalt::mantle::Operation;
pub use basalt::mantle::Request as OperationRequest;
use basalt::ucan::DurableHolderReplayAdmission;
use basalt::ucan::DurableReplayContext;
use basalt::ucan::DurableReplayContextInput;
use basalt::ucan::DurableReplayDurability;
use basalt::ucan::{self};
use basalt::ucan_enforcement::UcanEnforcementBoundaries;
use crunch_nix_gateway::replay::PrivateReplayLedger;
use crunch_nix_gateway_core::Authority;
use crunch_nix_gateway_core::CredentialKind;
use serde::Deserialize;
use zeroize::Zeroizing;

const MAX_AUTHORITY_BYTES: u64 = 1024 * 1024;
/// Both paths must already exist and be owner-private. Updating the authority
/// file atomically updates policy, keys, proofs, revocation and generation.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayAuthorityConfig {
    pub authority_file: PathBuf,
    pub replay_directory: PathBuf,
}

/// Operator-maintained current authority snapshot, not a caller wire envelope.
/// `revision` is monotonic within a running verifier; `generation` is bound into
/// grants and must increase when admission policy/issuer trust is replaced.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    revision: u64,
    generation: u64,
    issued_unix_s: u64,
    expires_unix_s: u64,
    endpoint: String,
    audience: String,
    policy_json: String,
    policy_blake3: String,
    trusted_roots: Vec<String>,
    keys: Vec<[u8; 32]>,
    proofs: Vec<String>,
    revoked: Vec<[u8; 32]>,
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        for proof in &mut self.proofs {
            proof.zeroize();
        }
    }
}

pub struct GatewaySession {
    inner: mantle::Session,
    project_filter: Option<ProjectFilter>,
}
impl std::fmt::Debug for GatewaySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewaySession").finish_non_exhaustive()
    }
}
impl GatewaySession {
    pub fn policy_identity(&self) -> &str {
        self.inner.policy_identity()
    }
    pub fn decision_id(&self) -> &str {
        self.inner.decision_id()
    }
    pub fn subject(&self) -> &str {
        &self.inner.request().subject
    }
    pub fn account(&self) -> &str {
        &self.inner.request().account
    }
    pub fn endpoint(&self) -> &str {
        &self.inner.request().endpoint
    }
    pub fn project(&self) -> &str {
        &self.inner.request().project
    }
    pub fn operation(&self) -> Operation {
        self.inner.request().operation
    }
    pub fn target(&self) -> &str {
        &self.inner.request().target
    }
    pub fn request_digest(&self) -> [u8; 32] {
        self.inner.request().request_digest
    }
    pub fn require(&self, operation: Operation, target: &str, digest: [u8; 32]) -> Result<(), Error> {
        self.inner.require(operation, target, digest)
    }
    /// Pure admission projection of an already verified live session. This is
    /// not revalidation; effect owners must check current policy before use.
    pub(crate) fn admission_facts(&self) -> Authority<'_> {
        let request = self.inner.request();
        Authority {
            kind: CredentialKind::VerifiedUcan,
            subject: &request.subject,
            scope: &request.project,
            audience: &request.audience,
            expires_unix_s: self.inner.expires_unix_s(),
            permissions: permission(request.operation),
        }
    }
}

struct ProjectFilter {
    original: String,
    owner_prefix: String,
}

/// Canonical owner/selector segment vocabulary; no decoding or path escapes.
pub(crate) fn project_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_+.-=".contains(&byte))
}

fn project_value(value: &str, prefix: &str, kind: &str) -> bool {
    value.len() <= crunch_remote_core::caveat::MAX_VALUE_BYTES
        && value
            .strip_prefix(prefix)
            .and_then(|value| value.strip_prefix(kind))
            .is_some_and(|suffix| suffix.split('/').all(project_segment))
}

impl GatewaySession {
    /// Original signed declaration retained through the effect and revalidation.
    pub fn project_declaration(&self) -> Option<&str> {
        self.project_filter.as_ref().map(|filter| filter.original.as_str())
    }
}

pub struct GatewayVerifier {
    config: GatewayAuthorityConfig,
    replay: PrivateReplayLedger,
    // Content identity prevents same-revision rollback or replacement. A source
    // unavailable/stale/malformed at any recheck grants no authority.
    observed: Mutex<(u64, [u8; 32])>,
}
impl GatewayVerifier {
    pub fn open(config: GatewayAuthorityConfig) -> Result<Self, Error> {
        let _ = private_read(&config.authority_file)?;
        let replay = PrivateReplayLedger::new(&config.replay_directory).map_err(|_| Error)?;
        Ok(Self {
            config,
            replay,
            observed: Mutex::new((0, [0; 32])),
        })
    }

    fn load(&self, now: u64) -> Result<Loaded, Error> {
        let bytes = private_read(&self.config.authority_file)?;
        let digest = *blake3::hash(&bytes).as_bytes();
        let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(|_| Error)?;
        if snapshot.version != 1
            || snapshot.revision == 0
            || snapshot.generation == 0
            || now < snapshot.issued_unix_s
            || now >= snapshot.expires_unix_s
            || snapshot.trusted_roots.is_empty()
            || snapshot.keys.is_empty()
            || snapshot.keys.len() > 256
            || snapshot.proofs.len() > 256
            || snapshot.revoked.len() > 8192
            || snapshot.policy_blake3 != blake3::hash(snapshot.policy_json.as_bytes()).to_hex().as_str()
        {
            return Err(Error);
        }
        let mut observed = self.observed.lock().map_err(|_| Error)?;
        if snapshot.revision < observed.0 || (snapshot.revision == observed.0 && digest != observed.1) {
            return Err(Error);
        }
        let policy = basalt::parse_policy_json(&snapshot.policy_json).map_err(|_| Error)?;
        let policy_ref = BasaltPolicyRef::new(
            format!("blake3:{}", snapshot.policy_blake3),
            basalt::SUPPORTED_SCHEMA_VERSION,
            "mantle-build-submit",
        );
        let keys = snapshot
            .keys
            .iter()
            .map(|key| {
                let key = ucan::Ed25519PublicKey::from_bytes(*key);
                ucan::Ed25519VerificationKey::new(key.issuer().map_err(|_| Error)?, key).map_err(|_| Error)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let proofs = snapshot
            .proofs
            .iter()
            .map(|token| ucan::CompactToken::new(token).map_err(|_| Error))
            .collect::<Result<Vec<_>, Error>>()?;
        *observed = (snapshot.revision, digest);
        Ok(Loaded {
            snapshot,
            policy,
            policy_ref,
            keys: ucan::KeyResolutionContext::new(keys),
            proofs: ucan::ProofCollection::from_tokens(proofs),
        })
    }

    /// Produces the exact request/resource to be signed, using current operator
    /// policy. This discloses no token/proof material and grants no authority.
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub fn enforcement(&self, request: &OperationRequest, now: u64) -> Result<basalt::EnforcementRequest, Error> {
        self.load(now)?.verifier().enforcement(request)
    }

    pub fn verify(
        &self,
        request: OperationRequest,
        invocation: Invocation<'_>,
        now: u64,
    ) -> Result<GatewaySession, Error> {
        self.verify_filtered(request, invocation, now, None)
    }

    /// Verify the original owner-bound declaration against its exact final goal.
    /// Capability, issuer, holder signature and replay checks remain canonical.
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub fn verify_project_declaration(
        &self,
        request: OperationRequest,
        original: &str,
        invocation: Invocation<'_>,
        now: u64,
    ) -> Result<GatewaySession, Error> {
        if request.operation != Operation::ProjectDeclare || !project_segment(&request.project) {
            return Err(Error);
        }
        let owner_prefix = format!("/project/{}/", request.project);
        if !project_value(original, &owner_prefix, "declared/")
            || !project_value(&request.target, &owner_prefix, "goal/")
        {
            return Err(Error);
        }
        self.verify_filtered(
            request,
            invocation,
            now,
            Some(ProjectFilter {
                original: original.into(),
                owner_prefix,
            }),
        )
    }

    fn verify_filtered(
        &self,
        request: OperationRequest,
        invocation: Invocation<'_>,
        now: u64,
        project_filter: Option<ProjectFilter>,
    ) -> Result<GatewaySession, Error> {
        let loaded = self.load(now)?;
        let revocations = loaded.revocations(now)?;
        let time = ucan::VerificationTime::from_unix_seconds(now);
        // This observation provides the exact immutable signed effective list to
        // the filter policy. It never bypasses the final holder/replay verifier.
        let verified = ucan::verify_compact_token_str_with_resolvers_and_revocations(
            invocation.compact_token,
            time,
            &loaded.keys,
            &loaded.proofs,
            &revocations,
            ucan::VerificationLimits::strict(),
        )
        .map_err(|_| Error)?;
        // The service uses a half-open lifetime, even if a token verifier's
        // boundary convention would still admit the exact expiration second.
        if now >= verified.effective_delegation().time_bounds().expires_at_seconds() {
            return Err(Error);
        }
        let enforcement = loaded.verifier().enforcement(&request)?;
        let selected = selected_capability(verified.effective_delegation().capabilities(), &enforcement)?;
        let exact = caveat_value(&request)?;
        let original = project_filter.as_ref().map_or(exact.as_str(), |filter| filter.original.as_str());
        let site_policy = |value: &str| {
            project_filter.as_ref().map_or(value == exact, |filter| {
                project_value(value, &filter.owner_prefix, "declared/")
                    || project_value(value, &filter.owner_prefix, "goal/")
            })
        };
        let policies = crate::remote_build::authority_caveats::PatternPolicies {
            original,
            expected: &exact,
            site_policy: &site_policy,
            selected,
        };
        // Empty caveat lists never call the registered policy, so cannot
        // establish a declaration-to-goal rewrite. Nonempty chains are checked
        // by the canonical verifier through PatternPolicies, not locally granted.
        if project_filter.is_some() && selected.caveats.is_empty() {
            return Err(Error);
        }
        let replay = self.replay_for(&request)?;
        let boundaries = loaded.boundaries(now, &revocations, &policies, &replay);
        let inner = loaded.verifier().verify(request, &invocation, &boundaries)?;
        Ok(GatewaySession { inner, project_filter })
    }

    pub fn revalidate<'a>(&self, session: &'a GatewaySession, now: u64) -> Result<Authority<'a>, Error> {
        if now >= session.inner.expires_unix_s() {
            return Err(Error);
        }
        let loaded = self.load(now)?;
        let request = session.inner.request();
        let revocations = loaded.revocations(now)?;
        let enforcement = loaded.verifier().enforcement(request)?;
        let selected = selected_capability(session.inner.effective_capabilities(), &enforcement)?;
        let exact = caveat_value(request)?;
        let original = session.project_filter.as_ref().map_or(exact.as_str(), |filter| filter.original.as_str());
        let site_policy = |value: &str| {
            session.project_filter.as_ref().map_or(value == exact, |filter| {
                project_value(value, &filter.owner_prefix, "declared/")
                    || project_value(value, &filter.owner_prefix, "goal/")
            })
        };
        let policies = crate::remote_build::authority_caveats::PatternPolicies {
            original,
            expected: &exact,
            site_policy: &site_policy,
            selected,
        };
        // Constructing this replay adapter is non-consuming. Basalt revalidation
        // does NOT call it; only initial verify can spend the holder nonce.
        let replay = self.replay_for(request)?;
        let boundaries = loaded.boundaries(now, &revocations, &policies, &replay);
        loaded.verifier().revalidate(&session.inner, &boundaries)?;
        Ok(session.admission_facts())
    }

    fn replay_for(
        &self,
        request: &OperationRequest,
    ) -> Result<DurableHolderReplayAdmission<'_, PrivateReplayLedger>, Error> {
        let profile = request.operation.action().operation_profile_id();
        let context = DurableReplayContext::new(DurableReplayContextInput {
            tenant: *blake3::hash(request.account.as_bytes()).as_bytes(),
            operation_profile: *profile.as_bytes(),
            authority_generation: request.generation,
            ledger_namespace: *blake3::hash(b"mantle/gateway/holder-replay/v1").as_bytes(),
            attempt: *blake3::hash(&request.request_digest).as_bytes(),
            durability: DurableReplayDurability::Synchronized,
        })
        .map_err(|_| Error)?;
        Ok(DurableHolderReplayAdmission::new(&self.replay, context))
    }
}

/// Mirror UCAN's first-match selection using its own containment primitive;
/// capability matching here only selects the pure policy input, never admits.
fn selected_capability<'a>(
    effective: &'a ucan::CapabilitySet,
    enforcement: &basalt::EnforcementRequest,
) -> Result<&'a ucan::CapabilityDocument, Error> {
    let requested = ucan::VerifiedCapability::new(enforcement.resource().into(), enforcement.ability().into())
        .map_err(|_| Error)?;
    for capability in effective.as_slice() {
        let parent = ucan::VerifiedCapability::from_document(capability.clone()).map_err(|_| Error)?;
        if parent.contains(&requested) {
            return Ok(capability);
        }
    }
    Err(Error)
}

struct Loaded {
    snapshot: Snapshot,
    policy: basalt::Policy,
    policy_ref: BasaltPolicyRef,
    keys: ucan::KeyResolutionContext,
    proofs: ucan::ProofCollection,
}
impl Loaded {
    fn verifier(&self) -> mantle::Verifier<'_> {
        mantle::Verifier {
            policy: &self.policy,
            policy_ref: &self.policy_ref,
            generation: self.snapshot.generation,
            endpoint: &self.snapshot.endpoint,
            audience: &self.snapshot.audience,
            trusted_roots: &self.snapshot.trusted_roots,
        }
    }
    fn revocations(&self, now: u64) -> Result<CurrentRevocations, Error> {
        use basalt::revocation_view_core as core;
        let source =
            core::SourceDigest::new(*blake3::hash(self.snapshot.endpoint.as_bytes()).as_bytes()).map_err(|_| Error)?;
        let generation = core::SourceGeneration::new(self.snapshot.revision).map_err(|_| Error)?;
        let entries = self
            .snapshot
            .revoked
            .iter()
            .map(|reference| {
                core::RevocationEntryDigest::new(ucan::durable_revocation_entry_digest(&ucan::ProofReference::new(
                    reference.to_vec(),
                )))
                .map_err(|_| Error)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let snapshot = core::RevocationSnapshot::admit(core::SnapshotInput {
            source: Some(source),
            generation: Some(generation),
            observed_at: core::UnixTime::from_seconds(self.snapshot.issued_unix_s),
            effective_from: core::UnixTime::from_seconds(self.snapshot.issued_unix_s),
            expires_at: core::UnixTime::from_seconds(self.snapshot.expires_unix_s),
            // These are receiver observations of the successfully owner-checked
            // complete local inventory, NEVER fields in a caller request.
            authenticity: core::AuthenticityObservation::Verified,
            completeness: core::CompletenessObservation::Complete,
            entries,
            entry_limit: core::EntryLimit::new(8192).map_err(|_| Error)?,
        })
        .map_err(|_| Error)?;
        Ok(CurrentRevocations {
            snapshot,
            requirement: core::CurrentnessRequirement {
                source,
                minimum_generation: generation,
                current_time: core::UnixTime::from_seconds(now),
            },
        })
    }
    fn boundaries<'a>(
        &'a self,
        now: u64,
        revocations: &'a dyn ucan::RevocationChecker,
        policies: &'a dyn ucan::CaveatPolicySet,
        replay: &'a dyn ucan::HolderInvocationReplayAdmission,
    ) -> UcanEnforcementBoundaries<'a> {
        UcanEnforcementBoundaries {
            verification_time: ucan::VerificationTime::from_unix_seconds(now),
            key_resolver: &self.keys,
            proof_store: &self.proofs,
            revocations,
            caveat_policies: policies,
            replay,
            verification_limits: ucan::VerificationLimits::strict(),
            holder_limits: ucan::HolderInvocationLimits::default(),
        }
    }
}
struct CurrentRevocations {
    snapshot: basalt::revocation_view_core::RevocationSnapshot,
    requirement: basalt::revocation_view_core::CurrentnessRequirement,
}
impl ucan::RevocationChecker for CurrentRevocations {
    fn is_revoked(&self, reference: &ucan::ProofReference) -> Result<bool, ucan::RevocationError> {
        use basalt::revocation_view_core as core;
        let entry =
            core::RevocationEntryDigest::new(ucan::durable_revocation_entry_digest(reference)).map_err(|_| {
                ucan::RevocationError::Malformed {
                    reference: reference.clone(),
                    message: "gateway-authority-denied".into(),
                }
            })?;
        match core::lookup(&self.snapshot, self.requirement, entry) {
            core::LookupDecision::Revoked => Ok(true),
            core::LookupDecision::NotRevoked => Ok(false),
            core::LookupDecision::Unknown(_) => Err(ucan::RevocationError::Stale {
                reference: reference.clone(),
                message: "gateway-authority-denied".into(),
            }),
        }
    }
}

/// Filter-site canonical input. Absolute logical paths retain their exact
/// spelling; other targets become one encoded segment under the operation.
pub fn caveat_value(request: &OperationRequest) -> Result<String, Error> {
    if request.target.starts_with('/') {
        return Ok(request.target.clone());
    }
    use std::fmt::Write as _;
    let mut value = format!("/gateway/{}/", request.operation.action().as_str());
    for byte in request.target.bytes() {
        write!(&mut value, "{byte:02x}").map_err(|_| Error)?;
    }
    if value.len() > crunch_remote_core::caveat::MAX_VALUE_BYTES {
        return Err(Error);
    }
    Ok(value)
}
fn permission(operation: Operation) -> u16 {
    match operation {
        Operation::BuildSubmit => 1,
        Operation::InputUpload => 2,
        Operation::StoreRead => 4,
        Operation::StatusRead => 8,
        Operation::LogRead => 16,
        Operation::OwnedAttemptCancel => 32,
        Operation::ProjectDeclare => 0,
        Operation::Publish => 64,
        Operation::UsageRead => 128,
        Operation::Admin => 256,
        Operation::EventRead => 512,
        Operation::ResultRead => 1024,
    }
}
fn private_read(path: &Path) -> Result<Zeroizing<Vec<u8>>, Error> {
    let file: File = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| Error)?;
    let metadata = file.metadata().map_err(|_| Error)?;
    // /proc/self is kernel-owned observation of this process's effective owner;
    // no unsafe libc call is needed to validate owner-private configuration.
    let owner = std::fs::metadata("/proc/self").map_err(|_| Error)?.uid();
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.mode() & 0o077 != 0
        || metadata.len() > MAX_AUTHORITY_BYTES
    {
        return Err(Error);
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(metadata.len() as usize));
    file.take(MAX_AUTHORITY_BYTES + 1).read_to_end(&mut bytes).map_err(|_| Error)?;
    if bytes.len() as u64 > MAX_AUTHORITY_BYTES {
        return Err(Error);
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "authority/tests.rs"]
mod tests;
