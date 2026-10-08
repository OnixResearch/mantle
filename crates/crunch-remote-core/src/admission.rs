use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_REMOTE_INPUT_REFS: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAdmissionError {
    ManifestRequestIdMismatch,
    ManifestStorePrefixMismatch,
    InputRefCountExceeded,
    ManifestDoesNotMatchRequest,
    SourceInputRefCountExceeded,
    SourceInputRefEmpty,
    SourceInputRefDuplicate,
    SourceInputRefNotDeclared,
}

impl InputAdmissionError {
    pub fn diagnostic(self) -> String {
        match self {
            Self::ManifestRequestIdMismatch => String::from("input-manifest-request-id-mismatch"),
            Self::ManifestStorePrefixMismatch => String::from("input-manifest-store-prefix-mismatch"),
            Self::InputRefCountExceeded => format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"),
            Self::ManifestDoesNotMatchRequest => String::from("input-manifest-does-not-match-build-request"),
            Self::SourceInputRefCountExceeded => format!("source-input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"),
            Self::SourceInputRefEmpty => String::from("source-input-ref-empty"),
            Self::SourceInputRefDuplicate => String::from("source-input-ref-duplicate"),
            Self::SourceInputRefNotDeclared => String::from("source-input-ref-not-declared"),
        }
    }
}

/// A view over already decoded wire fields; references are never copied just
/// to ask whether the manifest matches the declared request.
pub struct ManifestFacts<'a> {
    pub request_id: &'a str,
    pub store_prefix: &'a str,
    pub input_refs: &'a [String],
    pub closure_refs_count: usize,
}

/// Validate protocol binding without asserting that any uploaded bytes arrived.
// r[impl remote_builds.hexagonal_core]
pub fn admit_input_manifest(
    manifest: ManifestFacts<'_>,
    request_id: &str,
    store_prefix: &str,
    requested_refs: &[String],
) -> Result<(), InputAdmissionError> {
    if manifest.request_id != request_id {
        return Err(InputAdmissionError::ManifestRequestIdMismatch);
    }
    if manifest.store_prefix != store_prefix {
        return Err(InputAdmissionError::ManifestStorePrefixMismatch);
    }
    if manifest.input_refs.len() > MAX_REMOTE_INPUT_REFS || manifest.closure_refs_count > MAX_REMOTE_INPUT_REFS {
        return Err(InputAdmissionError::InputRefCountExceeded);
    }
    if manifest.input_refs != requested_refs {
        return Err(InputAdmissionError::ManifestDoesNotMatchRequest);
    }
    Ok(())
}

/// Source refs must be declared as build inputs and cannot be silently
/// deduplicated. The caller still verifies source content and store identity.
// r[impl remote_builds.hexagonal_core]
pub fn admit_source_input_refs(input_refs: &[String], source_input_refs: &[String]) -> Result<(), InputAdmissionError> {
    if source_input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(InputAdmissionError::SourceInputRefCountExceeded);
    }
    let inputs = input_refs.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for source_ref in source_input_refs {
        if source_ref.is_empty() {
            return Err(InputAdmissionError::SourceInputRefEmpty);
        }
        if !seen.insert(source_ref) {
            return Err(InputAdmissionError::SourceInputRefDuplicate);
        }
        if !inputs.contains(source_ref) {
            return Err(InputAdmissionError::SourceInputRefNotDeclared);
        }
    }
    Ok(())
}

/// Observe locally present references and plan only the missing input names.
/// Transport, content verification, and upload stay in host adapters.
// r[impl remote_builds.hexagonal_core]
pub fn plan_missing_inputs(
    declared_refs: &[String],
    present_refs: &[String],
) -> Result<Vec<String>, InputAdmissionError> {
    if declared_refs.len() > MAX_REMOTE_INPUT_REFS || present_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(InputAdmissionError::InputRefCountExceeded);
    }
    let present = present_refs.iter().collect::<BTreeSet<_>>();
    Ok(declared_refs.iter().filter(|reference| !present.contains(reference)).cloned().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorRequestError {
    RequiredSystemEmpty,
    RequiredSandboxEmpty,
    RequiredNetworkEmpty,
    TrustedOutputKeysEmpty,
    RequiredFeatureCountExceeded(usize),
    RequiredFeatureEmpty,
    RequiredFeatureDuplicate,
    LiveOutputClaimCountExceeded(usize),
    LiveOutputClaimEmpty,
    LiveOutputClaimDuplicate,
    LocalityScopeInvalid,
    UploadByteLimitExceeded,
    BuildTimeLimitExceeded,
}

impl CoordinatorRequestError {
    pub fn diagnostic(self) -> String {
        match self {
            Self::RequiredFeatureCountExceeded(max) => {
                format!("remote-coordinator-required-feature-count-exceeds-{max}")
            }
            Self::LiveOutputClaimCountExceeded(max) => {
                format!("remote-coordinator-live-output-claim-count-exceeds-{max}")
            }
            _ => String::from(self.as_str()),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::RequiredSystemEmpty => "remote-coordinator-required-system-empty",
            Self::RequiredSandboxEmpty => "remote-coordinator-required-sandbox-empty",
            Self::RequiredNetworkEmpty => "remote-coordinator-required-network-empty",
            Self::TrustedOutputKeysEmpty => "remote-coordinator-trusted-output-keys-empty",
            Self::RequiredFeatureCountExceeded(_) | Self::LiveOutputClaimCountExceeded(_) => "",
            Self::RequiredFeatureEmpty => "remote-coordinator-required-feature-empty",
            Self::RequiredFeatureDuplicate => "remote-coordinator-required-feature-duplicate",
            Self::LiveOutputClaimEmpty => "remote-coordinator-live-output-claim-empty",
            Self::LiveOutputClaimDuplicate => "remote-coordinator-live-output-claim-duplicate",
            Self::LocalityScopeInvalid => "remote-coordinator-locality-scope-invalid",
            Self::UploadByteLimitExceeded => "remote-coordinator-upload-byte-limit-exceeded",
            Self::BuildTimeLimitExceeded => "remote-coordinator-build-time-limit-exceeded",
        }
    }
}

pub struct CoordinatorDeclarationFacts<'a, F, C> {
    pub required_system: &'a str,
    pub required_sandbox_mode: &'a str,
    pub required_network_mode: &'a str,
    pub trusted_output_key_count: usize,
    pub required_features: F,
    pub required_features_max: usize,
    pub live_output_claims: C,
    pub live_output_claims_max: usize,
}

pub fn validate_coordinator_declaration<'a, F, C>(
    facts: CoordinatorDeclarationFacts<'a, F, C>,
) -> Result<(), CoordinatorRequestError>
where
    F: ExactSizeIterator<Item = &'a str>,
    C: ExactSizeIterator<Item = &'a str>,
{
    if facts.required_system.is_empty() {
        return Err(CoordinatorRequestError::RequiredSystemEmpty);
    }
    if facts.required_sandbox_mode.is_empty() {
        return Err(CoordinatorRequestError::RequiredSandboxEmpty);
    }
    if facts.required_network_mode.is_empty() {
        return Err(CoordinatorRequestError::RequiredNetworkEmpty);
    }
    if facts.trusted_output_key_count == 0 {
        return Err(CoordinatorRequestError::TrustedOutputKeysEmpty);
    }
    validate_bounded_unique_strings(facts.required_features, facts.required_features_max, false).map_err(|reason| {
        match reason {
            BoundedUniqueStringsError::CountExceeded(max) => CoordinatorRequestError::RequiredFeatureCountExceeded(max),
            BoundedUniqueStringsError::Missing | BoundedUniqueStringsError::Empty => {
                CoordinatorRequestError::RequiredFeatureEmpty
            }
            BoundedUniqueStringsError::Duplicate => CoordinatorRequestError::RequiredFeatureDuplicate,
        }
    })?;
    validate_bounded_unique_strings(facts.live_output_claims, facts.live_output_claims_max, false).map_err(
        |reason| match reason {
            BoundedUniqueStringsError::CountExceeded(max) => CoordinatorRequestError::LiveOutputClaimCountExceeded(max),
            BoundedUniqueStringsError::Missing | BoundedUniqueStringsError::Empty => {
                CoordinatorRequestError::LiveOutputClaimEmpty
            }
            BoundedUniqueStringsError::Duplicate => CoordinatorRequestError::LiveOutputClaimDuplicate,
        },
    )?;
    Ok(())
}

pub fn validate_coordinator_runtime_limits(
    locality_scope: Option<(&str, &str)>,
    uploaded_bytes: u64,
    max_uploaded_bytes: u64,
    build_time_limit_secs: u64,
    max_build_time_limit_secs: u64,
) -> Result<(), CoordinatorRequestError> {
    if let Some((manifest_digest, policy_digest)) = locality_scope
        && (!crate::output::is_blake3_hex_digest(manifest_digest)
            || !crate::output::is_blake3_hex_digest(policy_digest))
    {
        return Err(CoordinatorRequestError::LocalityScopeInvalid);
    }
    if uploaded_bytes > max_uploaded_bytes {
        return Err(CoordinatorRequestError::UploadByteLimitExceeded);
    }
    if build_time_limit_secs == 0 || build_time_limit_secs > max_build_time_limit_secs {
        return Err(CoordinatorRequestError::BuildTimeLimitExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedUniqueStringsError {
    Missing,
    CountExceeded(usize),
    Empty,
    Duplicate,
}

impl BoundedUniqueStringsError {
    pub fn diagnostic(self, label: &str) -> String {
        match self {
            Self::Missing => format!("{label}-missing"),
            Self::CountExceeded(max) => format!("{label}-count-exceeds-{max}"),
            Self::Empty => format!("{label}-empty"),
            Self::Duplicate => format!("{label}-duplicate"),
        }
    }
}

pub fn validate_bounded_unique_strings<'a>(
    values: impl ExactSizeIterator<Item = &'a str>,
    max_count: usize,
    nonempty: bool,
) -> Result<(), BoundedUniqueStringsError> {
    if nonempty && values.len() == 0 {
        return Err(BoundedUniqueStringsError::Missing);
    }
    if values.len() > max_count {
        return Err(BoundedUniqueStringsError::CountExceeded(max_count));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty() {
            return Err(BoundedUniqueStringsError::Empty);
        }
        if !seen.insert(value) {
            return Err(BoundedUniqueStringsError::Duplicate);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerRegistrationError {
    ProtocolVersionMismatch,
    GenerationZero,
    ConcurrencyExceeded(u32),
    ResumeSummaryCountExceeded(usize),
    ResumeIdentityEmpty,
    ResumeFenceInvalid,
    ResumeKeyInvalid,
}

impl WorkerRegistrationError {
    pub fn diagnostic(self) -> String {
        match self {
            Self::ConcurrencyExceeded(max) => format!("remote-worker-concurrency-exceeds-{max}"),
            Self::ResumeSummaryCountExceeded(max) => format!("remote-worker-resume-summary-count-exceeds-{max}"),
            _ => String::from(self.as_str()),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::ProtocolVersionMismatch => "remote-worker-protocol-version-mismatch",
            Self::GenerationZero => "remote-worker-generation-zero",
            Self::ConcurrencyExceeded(_) | Self::ResumeSummaryCountExceeded(_) => "",
            Self::ResumeIdentityEmpty => "remote-worker-resume-identity-empty",
            Self::ResumeFenceInvalid => "fence-invalid",
            Self::ResumeKeyInvalid => "remote-worker-resume-key-invalid",
        }
    }
}

pub fn validate_worker_registration_header(
    protocol_version_matches: bool,
    worker_generation: u64,
    concurrency: u32,
    max_concurrency: u32,
) -> Result<(), WorkerRegistrationError> {
    if !protocol_version_matches {
        return Err(WorkerRegistrationError::ProtocolVersionMismatch);
    }
    if worker_generation == 0 {
        return Err(WorkerRegistrationError::GenerationZero);
    }
    if concurrency == 0 || concurrency > max_concurrency {
        return Err(WorkerRegistrationError::ConcurrencyExceeded(max_concurrency));
    }
    Ok(())
}

pub fn validate_worker_resume_count(count: usize, max_count: usize) -> Result<(), WorkerRegistrationError> {
    if count > max_count {
        Err(WorkerRegistrationError::ResumeSummaryCountExceeded(max_count))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorkerResumeFacts<'a> {
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub legacy_missing_attempt_id: &'a str,
    pub normalized_build_key: &'a str,
    pub fence_generation: u64,
}

pub fn validate_worker_resume_summary(facts: WorkerResumeFacts<'_>) -> Result<(), WorkerRegistrationError> {
    if facts.job_id.is_empty()
        || facts.attempt_id.is_empty()
        || facts.attempt_id == facts.legacy_missing_attempt_id
        || facts.normalized_build_key.is_empty()
    {
        return Err(WorkerRegistrationError::ResumeIdentityEmpty);
    }
    if facts.fence_generation == 0 {
        return Err(WorkerRegistrationError::ResumeFenceInvalid);
    }
    if !crate::output::is_blake3_hex_digest(facts.normalized_build_key) {
        return Err(WorkerRegistrationError::ResumeKeyInvalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn worker_registration_checks_version_list_uniqueness_and_resume_fence_before_key() {
        assert_eq!(validate_worker_registration_header(true, 2, 4, 4), Ok(()));
        assert_eq!(
            validate_worker_registration_header(false, 0, 0, 4),
            Err(WorkerRegistrationError::ProtocolVersionMismatch)
        );
        assert_eq!(validate_bounded_unique_strings(["x"].into_iter(), 2, true), Ok(()));
        assert_eq!(validate_bounded_unique_strings([].into_iter(), 2, true), Err(BoundedUniqueStringsError::Missing));
        assert_eq!(
            validate_bounded_unique_strings(["x", "x"].into_iter(), 2, true),
            Err(BoundedUniqueStringsError::Duplicate)
        );
        let digest = "a".repeat(64);
        let valid = WorkerResumeFacts {
            job_id: "job",
            attempt_id: "attempt",
            legacy_missing_attempt_id: "legacy-missing-attempt-id",
            normalized_build_key: &digest,
            fence_generation: 1,
        };
        assert_eq!(validate_worker_resume_summary(valid), Ok(()));
        assert_eq!(
            validate_worker_resume_summary(WorkerResumeFacts {
                attempt_id: "legacy-missing-attempt-id",
                fence_generation: 0,
                ..valid
            }),
            Err(WorkerRegistrationError::ResumeIdentityEmpty)
        );
        assert_eq!(
            validate_worker_resume_summary(WorkerResumeFacts {
                fence_generation: 0,
                ..valid
            }),
            Err(WorkerRegistrationError::ResumeFenceInvalid)
        );
        assert_eq!(
            validate_worker_resume_summary(WorkerResumeFacts {
                normalized_build_key: "forged",
                ..valid
            }),
            Err(WorkerRegistrationError::ResumeKeyInvalid)
        );
    }

    #[test]
    fn coordinator_declaration_rejects_duplicate_features_before_live_claims_and_invalid_locality() {
        let facts = |features: [&'static str; 2], claims: [&'static str; 2]| CoordinatorDeclarationFacts {
            required_system: "x86_64-linux",
            required_sandbox_mode: "strict",
            required_network_mode: "off",
            trusted_output_key_count: 1,
            required_features: features.into_iter(),
            required_features_max: 2,
            live_output_claims: claims.into_iter(),
            live_output_claims_max: 2,
        };
        assert_eq!(
            validate_coordinator_declaration(facts(["feature-a", "feature-b"], ["output-a", "output-b"])),
            Ok(())
        );
        assert_eq!(
            validate_coordinator_declaration(facts(["feature-a", "feature-a"], ["", ""])),
            Err(CoordinatorRequestError::RequiredFeatureDuplicate)
        );
        assert_eq!(
            validate_coordinator_declaration(facts(["feature-a", "feature-b"], ["output-a", "output-a"])),
            Err(CoordinatorRequestError::LiveOutputClaimDuplicate)
        );
        let digest = "a".repeat(64);
        assert_eq!(validate_coordinator_runtime_limits(Some((&digest, &digest)), 10, 10, 30, 30), Ok(()));
        assert_eq!(
            validate_coordinator_runtime_limits(Some(("forged", &digest)), 11, 10, 0, 30),
            Err(CoordinatorRequestError::LocalityScopeInvalid)
        );
        assert_eq!(
            validate_coordinator_runtime_limits(None, 11, 10, 0, 30),
            Err(CoordinatorRequestError::UploadByteLimitExceeded)
        );
        assert_eq!(
            validate_coordinator_runtime_limits(None, 10, 10, 0, 30),
            Err(CoordinatorRequestError::BuildTimeLimitExceeded)
        );
    }

    #[test]
    fn a_matching_manifest_and_declared_sources_are_admitted() {
        let refs = vec![String::from("a"), String::from("b")];
        let manifest = ManifestFacts {
            request_id: "request-a",
            store_prefix: "/mantle/store",
            input_refs: &refs,
            closure_refs_count: 0,
        };
        assert_eq!(admit_input_manifest(manifest, "request-a", "/mantle/store", &refs), Ok(()));
        assert_eq!(admit_source_input_refs(&refs, &[String::from("b")]), Ok(()));
    }

    #[test]
    fn wrong_manifest_identity_precedes_ref_bounds_and_mismatch() {
        let refs = vec![String::from("a")];
        let manifest = ManifestFacts {
            request_id: "wrong",
            store_prefix: "/wrong",
            input_refs: &refs,
            closure_refs_count: MAX_REMOTE_INPUT_REFS + 1,
        };
        assert_eq!(
            admit_input_manifest(manifest, "request-a", "/mantle/store", &[]),
            Err(InputAdmissionError::ManifestRequestIdMismatch)
        );
        let manifest = ManifestFacts {
            request_id: "request-a",
            store_prefix: "/mantle/store",
            input_refs: &refs,
            closure_refs_count: MAX_REMOTE_INPUT_REFS + 1,
        };
        assert_eq!(
            admit_input_manifest(manifest, "request-a", "/mantle/store", &[]),
            Err(InputAdmissionError::InputRefCountExceeded)
        );
    }

    #[test]
    fn duplicate_empty_and_undeclared_sources_retain_distinct_rejections() {
        let refs = vec![String::from("a")];
        for (sources, expected) in [
            (vec![String::new()], InputAdmissionError::SourceInputRefEmpty),
            (vec![String::from("a"), String::from("a")], InputAdmissionError::SourceInputRefDuplicate),
            (vec![String::from("missing")], InputAdmissionError::SourceInputRefNotDeclared),
        ] {
            assert_eq!(admit_source_input_refs(&refs, &sources), Err(expected));
        }
    }

    #[test]
    fn present_observation_order_does_not_change_missing_inputs() {
        let declared = vec![String::from("b"), String::from("a"), String::from("b")];
        let left = plan_missing_inputs(&declared, &[String::from("a"), String::from("c")]).unwrap();
        let right = plan_missing_inputs(&declared, &[String::from("c"), String::from("a")]).unwrap();
        assert_eq!(left, vec![String::from("b"), String::from("b")]);
        assert_eq!(left, right);
    }
}
