use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::AgreementWitnessClassification;
use crunch_attestation::AttestationDigest;
use crunch_attestation::BinaryDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::DetachedSignature;
use crunch_attestation::FinalClass;
use crunch_attestation::IndependentAgreementReport;
use crunch_attestation::IndependentAgreementReportInit;
use crunch_attestation::IndependentAgreementStatus;
use crunch_attestation::PolicyFailureReason;
use crunch_attestation::PolicyStatus;
use crunch_attestation::RebuildEnvironmentSummary;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::ReleaseAttestationInit;
use crunch_attestation::ReleasePolicy;
use crunch_attestation::ReleaseRevocations;
use crunch_attestation::TechnicalClass;
use crunch_attestation::ValidatedWitness;
use crunch_attestation::VerificationDirectory;
use crunch_attestation::VerificationMaterial;
use crunch_attestation::WITNESS_SOURCE_ACQUISITION_MODE_NOT_RECORDED;
use crunch_attestation::WitnessAttestation;
use crunch_attestation::WitnessClassificationReason;
use crunch_attestation::encode_detached_signature;
use crunch_build::KeyPair;
use nix_compat::narinfo::SignatureRef;
use nix_compat::narinfo::VerifyingKey;
use serde::Serialize;

use crate::build_cmd::load_or_generate_signing_keypair;
use crate::errors::RunError;
use crate::release_evidence::ReleaseEvidenceManifest;
use crate::release_evidence::compute_path_blake3_digest;

const BLAKE3_ALGORITHM_NAME: &str = "blake3";
pub(crate) const RELEASE_ATTESTATION_FILE_NAME: &str = "release-attestation.json";
pub(crate) const RELEASE_ATTESTATION_SIG_FILE_NAME: &str = "release-attestation.json.sig";
pub(crate) const POLICY_FILE_NAME: &str = "policy.json";
pub(crate) const REVOCATIONS_FILE_NAME: &str = "revocations.json";
pub(crate) const WITNESSES_DIR_NAME: &str = "witnesses";
const MAX_WITNESS_SHOW_FILES: u32 = 1_024;
const MAX_WITNESS_IDENTITY_BYTES: usize = 128;
const SINGLE_WITNESS_QUORUM: u32 = 1;
const SELF_PROOF_ONLY_QUORUM: u32 = 0;
const POLICY_INIT_REQUIRED_SIGNER_COUNT: u32 = 1;
const POLICY_INIT_REQUIRED_WITNESS_COUNT: u32 = 1;
const INDEPENDENCE_FIELD_WITNESS_IDENTITY: &str = "witness_identity";
const INDEPENDENCE_FIELD_SIGNER_KEY_NAME: &str = "signer_key_name";
const INDEPENDENCE_FIELD_REBUILD_HOST_CLASS: &str = "rebuild_environment_summary.host_class";
const INDEPENDENT_AGREEMENT_CLASS: &str = "independent-rebuild-agreement";
const AGREEMENT_REPORT_FILE_NAME: &str = "agreement-report.json";
const MAX_AGREEMENT_REPORT_CANDIDATES: usize = 32;
pub(crate) const WITNESS_SOURCE_ACQUISITION_MODE_MANUAL: &str = "manual-operator-supplied";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatedReleaseAttestation {
    pub attestation: ReleaseAttestation,
    pub digest_hex: String,
    pub signer_key_name: String,
    pub attestation_path: PathBuf,
    pub signature_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessDocument {
    pub attestation: WitnessAttestation,
    pub attestation_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatedWitnessAttestation {
    pub attestation: WitnessAttestation,
    pub digest_hex: String,
    pub signer_key_name: String,
    pub attestation_path: PathBuf,
    pub signature_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyInitProfile {
    SelfProofOnly,
    SingleWitness,
}

impl PolicyInitProfile {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SelfProofOnly => "self-proof-only",
            Self::SingleWitness => "single-witness",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatedPolicyFiles {
    pub profile: PolicyInitProfile,
    pub policy: ReleasePolicy,
    pub revocations: ReleaseRevocations,
    pub policy_path: PathBuf,
    pub revocations_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AgreementWitnessOutput {
    pub witness_identity: String,
    pub signer_key_name: String,
    pub release_attestation_digest_blake3: String,
    pub signature_valid: bool,
    pub digest_match: bool,
    pub independence_domain: String,
    pub source_acquisition_mode: String,
    pub policy_counted: bool,
    pub classification_reason: WitnessClassificationReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReleaseVerificationOutput {
    pub release_attestation_digest: String,
    pub release_signer_key_name: String,
    pub discovered_witness_count: u32,
    pub considered_witness_count: u32,
    pub technical_class: TechnicalClass,
    pub policy_status: PolicyStatus,
    pub final_class: FinalClass,
    pub matching_witness_count: u32,
    pub independent_witness_identities: u32,
    pub revoked_witness_count: u32,
    pub policy_independence_field: String,
    pub policy_required_witness_count: u32,
    pub independent_agreement_status: IndependentAgreementStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub independent_agreement_class: Option<&'static str>,
    pub independent_agreement_report_digest: String,
    pub independent_agreement_counted_witness_count: u32,
    pub independent_agreement_skipped_witness_count: u32,
    pub independent_agreement_failed_witness_count: u32,
    pub independent_agreement_witnesses: Vec<AgreementWitnessOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_failure_reason: Option<PolicyFailureReason>,
}

pub(crate) fn create_release_attestation(
    manifest: &ReleaseEvidenceManifest,
    verification_dir: &Path,
    signing_key_path: Option<&Path>,
    state_dir: &Path,
) -> Result<CreatedReleaseAttestation, RunError> {
    let attestation = build_release_attestation_from_bundle(manifest)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let canonical_bytes = canonical_release_bytes(&attestation)?;
    let digest_hex = AttestationDigest::from_canonical_bytes(canonical_bytes.clone()).to_hex();
    let signature = sign_detached_message(&canonical_bytes, &keypair);

    prepare_verification_dir(verification_dir)?;
    let attestation_path = verification_dir.join(RELEASE_ATTESTATION_FILE_NAME);
    let signature_path = verification_dir.join(RELEASE_ATTESTATION_SIG_FILE_NAME);
    std::fs::write(&attestation_path, &canonical_bytes)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", attestation_path.display())))?;
    std::fs::write(&signature_path, format!("{}\n", encode_detached_signature(&signature)))
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", signature_path.display())))?;

    Ok(CreatedReleaseAttestation {
        attestation,
        digest_hex,
        signer_key_name: keypair.verifying_key.name().to_string(),
        attestation_path,
        signature_path,
    })
}

pub(crate) fn create_witness_attestation(
    verification_dir: &Path,
    rebuilt_binary_paths: &[PathBuf],
    witness_identity: Option<&str>,
    system: &str,
    toolchain: &str,
    host_class: &str,
    source_acquisition_mode: &str,
    signing_key_path: Option<&Path>,
    state_dir: &Path,
) -> Result<CreatedWitnessAttestation, RunError> {
    let (release_attestation, _stored_path) = load_release_attestation_document(verification_dir)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let signer_key_name = keypair.verifying_key.name().to_string();
    let resolved_identity = resolve_witness_identity(witness_identity, &signer_key_name)?;
    let rebuilt_digests = compute_rebuilt_binary_digests(&release_attestation, rebuilt_binary_paths)?;
    let attestation = WitnessAttestation::new(
        release_attestation
            .canonical_digest()
            .map_err(|err| RunError::Build(format!("release attestation digest: {err}")))?,
        resolved_identity.clone(),
        rebuilt_digests,
        RebuildEnvironmentSummary {
            system: system.to_string(),
            toolchain: toolchain.to_string(),
            host_class: host_class.to_string(),
        },
    )
    .with_source_acquisition_mode(source_acquisition_mode.to_string());
    let canonical_bytes = canonical_witness_bytes(&attestation)?;
    let digest_hex = AttestationDigest::from_canonical_bytes(canonical_bytes.clone()).to_hex();
    let signature = sign_detached_message(&canonical_bytes, &keypair);
    let witness_dir = prepare_witness_dir(verification_dir)?;
    let attestation_path = witness_dir.join(format!("{resolved_identity}.json"));
    let signature_path = witness_dir.join(format!("{resolved_identity}.json.sig"));
    std::fs::write(&attestation_path, &canonical_bytes)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", attestation_path.display())))?;
    std::fs::write(&signature_path, format!("{}\n", encode_detached_signature(&signature)))
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", signature_path.display())))?;

    Ok(CreatedWitnessAttestation {
        attestation,
        digest_hex,
        signer_key_name,
        attestation_path,
        signature_path,
    })
}

pub(crate) fn create_policy_files(
    verification_dir: &Path,
    profile: PolicyInitProfile,
    trusted_release_signers: &[String],
    trusted_witness_identities: &[String],
    force: bool,
) -> Result<CreatedPolicyFiles, RunError> {
    let (_release_attestation, _stored_path) = load_release_attestation_document(verification_dir)?;
    let policy_file = build_policy_file_contents(profile, trusted_release_signers, trusted_witness_identities)?;
    let policy_path = verification_dir.join(POLICY_FILE_NAME);
    let revocations_path = verification_dir.join(REVOCATIONS_FILE_NAME);
    validate_policy_output_path(&policy_path, force)?;
    validate_policy_output_path(&revocations_path, force)?;
    write_json_document(&policy_path, &policy_file.policy)?;
    write_json_document(&revocations_path, &policy_file.revocations)?;

    Ok(CreatedPolicyFiles {
        profile,
        policy: policy_file.policy,
        revocations: policy_file.revocations,
        policy_path,
        revocations_path,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyFileContents {
    policy: ReleasePolicy,
    revocations: ReleaseRevocations,
}

fn build_policy_file_contents(
    profile: PolicyInitProfile,
    trusted_release_signers: &[String],
    trusted_witness_identities: &[String],
) -> Result<PolicyFileContents, RunError> {
    let release_signers = normalize_policy_name_set(
        trusted_release_signers,
        "trusted release signer",
        POLICY_INIT_REQUIRED_SIGNER_COUNT,
    )?;
    let witness_identities = normalize_policy_witness_identities(profile, trusted_witness_identities)?;
    let policy = ReleasePolicy::new(
        profile_min_matching_witnesses(profile),
        INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string(),
        release_signers,
        witness_identities,
    );
    Ok(PolicyFileContents {
        policy,
        revocations: ReleaseRevocations::empty(),
    })
}

fn normalize_policy_witness_identities(
    profile: PolicyInitProfile,
    trusted_witness_identities: &[String],
) -> Result<Vec<String>, RunError> {
    let normalized = normalize_policy_name_set(trusted_witness_identities, "trusted witness identity", 0)?;
    match profile {
        PolicyInitProfile::SelfProofOnly => {
            if normalized.is_empty() {
                return Ok(normalized);
            }
            Err(RunError::Internal("self-proof-only profile does not accept trusted witness identities".to_string()))
        }
        PolicyInitProfile::SingleWitness => {
            let witness_count = count_policy_entries(&normalized, "trusted witness identity")?;
            if witness_count < POLICY_INIT_REQUIRED_WITNESS_COUNT {
                return Err(RunError::Internal(
                    "single-witness profile requires at least one trusted witness identity".to_string(),
                ));
            }
            Ok(normalized)
        }
    }
}

fn normalize_policy_name_set(
    requested_values: &[String],
    field_label: &str,
    required_count: u32,
) -> Result<Vec<String>, RunError> {
    let mut normalized_values = BTreeSet::new();
    for requested_value in requested_values {
        let normalized_value = normalize_policy_name(requested_value, field_label)?;
        normalized_values.insert(normalized_value);
    }
    let normalized = normalized_values.into_iter().collect::<Vec<_>>();
    let normalized_count = count_policy_entries(&normalized, field_label)?;
    if normalized_count < required_count {
        return Err(RunError::Internal(format!("{field_label} requires at least {required_count} value(s)")));
    }
    Ok(normalized)
}

fn normalize_policy_name(requested_value: &str, field_label: &str) -> Result<String, RunError> {
    let normalized = requested_value.trim();
    if normalized.is_empty() {
        return Err(RunError::Internal(format!("{field_label} must not be empty")));
    }
    if normalized.chars().any(|character| character.is_control()) {
        return Err(RunError::Internal(format!(
            "{field_label} must not contain control characters: {:?}",
            requested_value
        )));
    }
    Ok(normalized.to_string())
}

fn count_policy_entries(values: &[String], field_label: &str) -> Result<u32, RunError> {
    u32::try_from(values.len()).map_err(|_| RunError::Internal(format!("{field_label} count overflowed u32")))
}

fn profile_min_matching_witnesses(profile: PolicyInitProfile) -> u32 {
    match profile {
        PolicyInitProfile::SelfProofOnly => SELF_PROOF_ONLY_QUORUM,
        PolicyInitProfile::SingleWitness => SINGLE_WITNESS_QUORUM,
    }
}

fn validate_policy_output_path(path: &Path, force: bool) -> Result<(), RunError> {
    if path.exists() && !force {
        return Err(RunError::Internal(format!("refusing to overwrite existing {} without --force", path.display())));
    }
    if path.is_dir() {
        return Err(RunError::Internal(format!("policy output path is a directory: {}", path.display())));
    }
    Ok(())
}

fn write_json_document<T: Serialize>(path: &Path, document: &T) -> Result<(), RunError> {
    let json_bytes = serde_json::to_vec_pretty(document)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    std::fs::write(path, json_bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

pub(crate) fn default_verification_dir(current_dir: &Path, release_id: &str) -> PathBuf {
    current_dir.join("target").join("release-verification").join(release_id)
}

pub(crate) fn load_release_attestation_document(dir: &Path) -> Result<(ReleaseAttestation, PathBuf), RunError> {
    let path = dir.join(RELEASE_ATTESTATION_FILE_NAME);
    let attestation = read_release_attestation_file(&path)?;
    Ok((attestation, path))
}

pub(crate) fn load_witness_documents(dir: &Path) -> Result<Vec<WitnessDocument>, RunError> {
    let mut documents = Vec::new();
    for path in witness_json_paths(dir)? {
        let attestation = read_witness_attestation_file(&path)?;
        documents.push(WitnessDocument {
            attestation,
            attestation_path: path,
        });
    }
    Ok(documents)
}

pub(crate) fn verify_release_attestation_directory(
    dir: &Path,
    trusted_public_keys: &[VerifyingKey],
) -> Result<ReleaseVerificationOutput, RunError> {
    if trusted_public_keys.is_empty() {
        return Err(RunError::Internal("release verification requires at least one trusted public key".to_string()));
    }

    let material = VerificationDirectory::new(dir.to_path_buf())
        .discover()
        .map_err(|err| RunError::Build(format!("release verification discovery: {err}")))?;
    evaluate_release_verification(dir, &material, trusted_public_keys)
}

fn evaluate_release_verification(
    verification_dir: &Path,
    material: &VerificationMaterial,
    trusted_public_keys: &[VerifyingKey],
) -> Result<ReleaseVerificationOutput, RunError> {
    let release_signer_key_name = verify_release_signature(material, trusted_public_keys)?;
    let validated_witnesses = collect_trusted_validated_witnesses(material, trusted_public_keys)?;
    let evaluation = crunch_attestation::evaluate_policy(
        &material.release_attestation,
        &validated_witnesses,
        &material.policy,
        &material.revocations,
    )
    .map_err(|err| RunError::Build(format!("release verification policy evaluation: {err}")))?;
    let release_attestation_digest = canonical_release_digest_hex(&material.release_attestation)?;
    let discovered_witness_count = u32::try_from(material.witnesses.len())
        .map_err(|_| RunError::Internal("release verification witness count overflowed u32".to_string()))?;
    let considered_witness_count = u32::try_from(validated_witnesses.len())
        .map_err(|_| RunError::Internal("release verification considered witness count overflowed u32".to_string()))?;
    let agreement_report = build_independent_agreement_report(material, trusted_public_keys)?;
    verify_optional_agreement_report_attachment(verification_dir, &agreement_report)?;
    let independent_agreement_report_digest =
        crunch_attestation::independent_agreement_report_canonical_digest(agreement_report.clone())
            .map_err(|err| RunError::Build(format!("independent agreement report digest: {err}")))?
            .to_hex();
    let independent_agreement_status = agreement_report.status();
    let independent_agreement_class = if independent_agreement_status == IndependentAgreementStatus::Satisfied {
        Some(INDEPENDENT_AGREEMENT_CLASS)
    } else {
        None
    };
    let independent_agreement_witnesses =
        agreement_report.witnesses.iter().map(agreement_witness_output).collect::<Vec<_>>();

    Ok(ReleaseVerificationOutput {
        release_attestation_digest,
        release_signer_key_name,
        discovered_witness_count,
        considered_witness_count,
        technical_class: evaluation.trust_tier.technical_class,
        policy_status: evaluation.trust_tier.policy_status,
        final_class: evaluation.trust_tier.final_class,
        matching_witness_count: evaluation.matching_witness_count,
        independent_witness_identities: evaluation.independent_witness_identities,
        revoked_witness_count: evaluation.revoked_witness_count,
        policy_independence_field: material.policy.independence_field.clone(),
        policy_required_witness_count: material.policy.min_matching_witnesses,
        independent_agreement_status,
        independent_agreement_class,
        independent_agreement_report_digest,
        independent_agreement_counted_witness_count: agreement_report.counted_witness_count,
        independent_agreement_skipped_witness_count: agreement_report.skipped_witness_count,
        independent_agreement_failed_witness_count: agreement_report.failed_witness_count,
        independent_agreement_witnesses,
        policy_failure_reason: evaluation.policy_failure_reason,
    })
}

fn verify_optional_agreement_report_attachment(
    verification_dir: &Path,
    derived_report: &IndependentAgreementReport,
) -> Result<(), RunError> {
    let candidates = agreement_report_candidates(verification_dir)?;
    if candidates.is_empty() {
        return Ok(());
    }
    if candidates.len() > 1 {
        return Err(RunError::Build(format!(
            "ambiguous independent agreement report attachments: {}",
            candidates.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
        )));
    }
    let report_path = &candidates[0];
    let expected_path = verification_dir.join(AGREEMENT_REPORT_FILE_NAME);
    if report_path != &expected_path {
        return Err(RunError::Build(format!(
            "independent agreement report must be stored at {}, got {}",
            expected_path.display(),
            report_path.display()
        )));
    }
    let attached_bytes = std::fs::read(report_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", report_path.display())))?;
    let attached_report: IndependentAgreementReport = serde_json::from_slice(&attached_bytes).map_err(|err| {
        RunError::Build(format!("parsing independent agreement report {}: {err}", report_path.display()))
    })?;
    let canonical_attached = crunch_attestation::independent_agreement_report_canonical_bytes(attached_report)
        .map_err(|err| RunError::Build(format!("canonicalizing independent agreement report: {err}")))?;
    if attached_bytes != canonical_attached {
        return Err(RunError::Build("independent agreement report is not canonical compact JSON".to_string()));
    }
    let canonical_derived = crunch_attestation::independent_agreement_report_canonical_bytes(derived_report.clone())
        .map_err(|err| RunError::Build(format!("canonicalizing derived independent agreement report: {err}")))?;
    if canonical_attached != canonical_derived {
        let attached_digest = AttestationDigest::from_canonical_bytes(canonical_attached).to_hex();
        let derived_digest = AttestationDigest::from_canonical_bytes(canonical_derived).to_hex();
        return Err(RunError::Build(format!(
            "independent agreement report digest mismatch: attached {attached_digest} derived {derived_digest}"
        )));
    }
    Ok(())
}

fn agreement_report_candidates(verification_dir: &Path) -> Result<Vec<PathBuf>, RunError> {
    let mut candidates = Vec::new();
    collect_agreement_report_candidates(verification_dir, &mut candidates)?;
    candidates.sort();
    candidates.dedup();
    if candidates.len() > MAX_AGREEMENT_REPORT_CANDIDATES {
        return Err(RunError::Build(format!(
            "too many independent agreement report candidates: {} > {}",
            candidates.len(),
            MAX_AGREEMENT_REPORT_CANDIDATES
        )));
    }
    Ok(candidates)
}

fn collect_agreement_report_candidates(dir: &Path, candidates: &mut Vec<PathBuf>) -> Result<(), RunError> {
    for entry in
        std::fs::read_dir(dir).map_err(|err| RunError::Internal(format!("reading {}: {err}", dir.display())))?
    {
        let entry = entry.map_err(|err| RunError::Internal(format!("reading {} entry: {err}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| RunError::Internal(format!("reading {} type: {err}", path.display())))?;
        if file_type.is_file() && path.file_name().and_then(|name| name.to_str()) == Some(AGREEMENT_REPORT_FILE_NAME) {
            candidates.push(path);
        } else if file_type.is_dir() {
            collect_agreement_report_candidates(&path, candidates)?;
        }
    }
    Ok(())
}

fn build_independent_agreement_report(
    material: &VerificationMaterial,
    trusted_public_keys: &[VerifyingKey],
) -> Result<IndependentAgreementReport, RunError> {
    let release_digest = material
        .release_attestation
        .canonical_digest()
        .map_err(|err| RunError::Build(format!("release attestation digest: {err}")))?;
    let policy_bytes = serde_json::to_vec(&material.policy)
        .map_err(|err| RunError::Internal(format!("serializing release policy for agreement digest: {err}")))?;
    let policy_digest = AttestationDigest::from_canonical_bytes(policy_bytes);
    let mut used_domains = BTreeSet::new();
    let mut witnesses = Vec::with_capacity(material.witnesses.len());
    for witness in &material.witnesses {
        witnesses.push(classify_agreement_witness(
            material,
            witness,
            trusted_public_keys,
            &release_digest,
            &mut used_domains,
        )?);
    }
    IndependentAgreementReport::new(IndependentAgreementReportInit {
        release_attestation_digest_blake3: release_digest,
        policy_digest_blake3: policy_digest,
        independence_selector: material.policy.independence_field.clone(),
        required_witness_count: material.policy.min_matching_witnesses,
        witnesses,
        artifact_digest_sets: material.release_attestation.binary_digests.clone(),
    })
    .map_err(|err| RunError::Build(format!("building independent agreement report: {err}")))
}

fn classify_agreement_witness(
    material: &VerificationMaterial,
    witness: &crunch_attestation::discovery::DiscoveredWitness,
    trusted_public_keys: &[VerifyingKey],
    release_digest: &AttestationDigest,
    used_domains: &mut BTreeSet<String>,
) -> Result<AgreementWitnessClassification, RunError> {
    let witness_digest = witness_attestation_digest_for_report(&witness.attestation)?;
    let (signature_valid, signer_key_name, mut reason) = classify_witness_signature(witness, trusted_public_keys)?;
    let is_identity_trusted = is_trusted_witness_identity(&material.policy, &witness.attestation);
    if signature_valid && !is_identity_trusted {
        reason = WitnessClassificationReason::MissingIndependenceEvidence;
    }
    let is_revoked = witness_is_revoked(&witness_digest, &signer_key_name, &material.revocations);
    if signature_valid && is_revoked {
        reason = WitnessClassificationReason::Revoked;
    }
    let digest_match =
        witness_digest_matches_release(&material.release_attestation, &witness.attestation, release_digest);
    if signature_valid && !is_revoked && is_identity_trusted && !digest_match {
        reason = WitnessClassificationReason::DigestMismatch;
    }
    let independence_domain = agreement_independence_domain(&material.policy, &witness.attestation, &signer_key_name);
    if signature_valid && !is_revoked && is_identity_trusted && digest_match && independence_domain.is_empty() {
        reason = WitnessClassificationReason::MissingIndependenceEvidence;
    }
    let policy_counted = signature_valid
        && is_identity_trusted
        && !is_revoked
        && digest_match
        && !independence_domain.is_empty()
        && mark_domain_if_new(used_domains, &independence_domain);
    if signature_valid
        && is_identity_trusted
        && !is_revoked
        && digest_match
        && !independence_domain.is_empty()
        && !policy_counted
    {
        reason = WitnessClassificationReason::DuplicateIndependenceDomain;
    }
    if policy_counted {
        reason = WitnessClassificationReason::Counted;
    }

    Ok(AgreementWitnessClassification {
        witness_identity: witness.attestation.witness_identity.clone(),
        signer_key_name,
        witness_digest_blake3: witness_digest,
        release_attestation_digest_blake3: witness.attestation.release_attestation_digest_blake3.clone(),
        signature_valid,
        digest_match,
        independence_domain,
        source_acquisition_mode: witness_source_acquisition_mode(&witness.attestation),
        policy_counted,
        classification_reason: reason,
        rebuilt_output_digests: witness.attestation.rebuilt_digests.clone(),
        environment_summary: witness.attestation.rebuild_environment_summary.clone(),
    })
}

fn witness_source_acquisition_mode(witness: &WitnessAttestation) -> String {
    witness
        .source_acquisition_mode
        .clone()
        .unwrap_or_else(|| WITNESS_SOURCE_ACQUISITION_MODE_NOT_RECORDED.to_string())
}

fn classify_witness_signature(
    witness: &crunch_attestation::discovery::DiscoveredWitness,
    trusted_public_keys: &[VerifyingKey],
) -> Result<(bool, String, WitnessClassificationReason), RunError> {
    let canonical_bytes = match canonical_witness_bytes(&witness.attestation) {
        Ok(bytes) => bytes,
        Err(RunError::Build(_)) => {
            return Ok((
                false,
                witness.signature.key_name.clone(),
                WitnessClassificationReason::MalformedEnvironmentEvidence,
            ));
        }
        Err(err) => return Err(err),
    };
    match verify_signature_bytes(&canonical_bytes, &witness.signature, trusted_public_keys, "witness attestation") {
        Ok(signer_key_name) => Ok((true, signer_key_name, WitnessClassificationReason::Counted)),
        Err(RunError::Build(message)) => {
            let reason = if message.contains("missing from the trusted public key set") {
                WitnessClassificationReason::UnknownKey
            } else {
                WitnessClassificationReason::InvalidSignature
            };
            Ok((false, witness.signature.key_name.clone(), reason))
        }
        Err(err) => Err(err),
    }
}

fn witness_attestation_digest_for_report(attestation: &WitnessAttestation) -> Result<AttestationDigest, RunError> {
    match attestation.canonical_digest() {
        Ok(digest) => Ok(digest),
        Err(_) => serde_json::to_vec(attestation)
            .map(AttestationDigest::from_canonical_bytes)
            .map_err(|err| RunError::Build(format!("serializing malformed witness attestation: {err}"))),
    }
}

fn witness_is_revoked(
    witness_digest: &AttestationDigest,
    signer_key_name: &str,
    revocations: &ReleaseRevocations,
) -> bool {
    let digest_hex = witness_digest.to_hex();
    revocations.revoked_witness_keys.iter().any(|key| key == signer_key_name)
        || revocations.revoked_witness_attestation_digests_blake3.iter().any(|digest| digest == &digest_hex)
}

fn witness_digest_matches_release(
    release: &ReleaseAttestation,
    witness: &WitnessAttestation,
    release_digest: &AttestationDigest,
) -> bool {
    witness.release_attestation_digest_blake3 == *release_digest
        && crunch_attestation::binary_digests_match(&release.binary_digests, &witness.rebuilt_digests)
}

fn agreement_independence_domain(
    policy: &ReleasePolicy,
    witness: &WitnessAttestation,
    signer_key_name: &str,
) -> String {
    match policy.independence_field.as_str() {
        INDEPENDENCE_FIELD_WITNESS_IDENTITY => witness.witness_identity.clone(),
        INDEPENDENCE_FIELD_SIGNER_KEY_NAME => signer_key_name.to_string(),
        INDEPENDENCE_FIELD_REBUILD_HOST_CLASS => witness.rebuild_environment_summary.host_class.clone(),
        _ => String::new(),
    }
}

fn mark_domain_if_new(used_domains: &mut BTreeSet<String>, domain: &str) -> bool {
    if used_domains.contains(domain) {
        false
    } else {
        used_domains.insert(domain.to_string());
        true
    }
}

fn agreement_witness_output(witness: &AgreementWitnessClassification) -> AgreementWitnessOutput {
    AgreementWitnessOutput {
        witness_identity: witness.witness_identity.clone(),
        signer_key_name: witness.signer_key_name.clone(),
        release_attestation_digest_blake3: witness.release_attestation_digest_blake3.to_hex(),
        signature_valid: witness.signature_valid,
        digest_match: witness.digest_match,
        independence_domain: witness.independence_domain.clone(),
        source_acquisition_mode: witness.source_acquisition_mode.clone(),
        policy_counted: witness.policy_counted,
        classification_reason: witness.classification_reason,
    }
}

fn collect_trusted_validated_witnesses(
    material: &VerificationMaterial,
    trusted_public_keys: &[VerifyingKey],
) -> Result<Vec<ValidatedWitness>, RunError> {
    let mut validated = Vec::with_capacity(material.witnesses.len());

    for witness in &material.witnesses {
        let maybe_validated = validate_witness_for_policy(material, witness, trusted_public_keys)?;
        if let Some(validated_witness) = maybe_validated {
            validated.push(validated_witness);
        }
    }

    assert!(validated.len() <= material.witnesses.len(), "validated witnesses must stay bounded");
    Ok(validated)
}

fn validate_witness_for_policy(
    material: &VerificationMaterial,
    witness: &crunch_attestation::discovery::DiscoveredWitness,
    trusted_public_keys: &[VerifyingKey],
) -> Result<Option<ValidatedWitness>, RunError> {
    let canonical_bytes = match canonical_witness_bytes(&witness.attestation) {
        Ok(bytes) => bytes,
        Err(RunError::Build(_)) => return Ok(None),
        Err(err) => return Err(err),
    };
    let signer_key_name = match verify_signature_bytes(
        &canonical_bytes,
        &witness.signature,
        trusted_public_keys,
        "witness attestation",
    ) {
        Ok(signer_key_name) => signer_key_name,
        Err(RunError::Build(_)) => return Ok(None),
        Err(err) => return Err(err),
    };
    if !is_trusted_witness_identity(&material.policy, &witness.attestation) {
        return Ok(None);
    }

    let attestation_digest = witness
        .attestation
        .canonical_digest()
        .map_err(|err| RunError::Build(format!("witness attestation digest: {err}")))?;
    Ok(Some(ValidatedWitness {
        attestation: witness.attestation.clone(),
        attestation_digest,
        signer_key_name,
    }))
}

fn is_trusted_witness_identity(policy: &ReleasePolicy, witness: &WitnessAttestation) -> bool {
    if policy.trusted_witness_signers.is_empty() {
        return true;
    }
    policy.trusted_witness_signers.iter().any(|identity| identity == &witness.witness_identity)
}

fn verify_release_signature(
    material: &VerificationMaterial,
    trusted_public_keys: &[VerifyingKey],
) -> Result<String, RunError> {
    let canonical_bytes = canonical_release_bytes(&material.release_attestation)?;
    verify_signature_with_policy_names(
        &canonical_bytes,
        &material.release_signature,
        trusted_public_keys,
        &material.policy.trusted_release_signers,
        "release attestation",
    )
}

fn verify_signature_with_policy_names(
    canonical_bytes: &[u8],
    signature: &DetachedSignature,
    trusted_public_keys: &[VerifyingKey],
    trusted_signer_names: &[String],
    context: &str,
) -> Result<String, RunError> {
    if trusted_signer_names.is_empty() {
        return Err(RunError::Build(format!("{context} has no trusted signer names configured in policy.json")));
    }
    if !trusted_signer_names.iter().any(|name| name == &signature.key_name) {
        return Err(RunError::Build(format!(
            "{context} signer '{}' is not trusted by policy.json",
            signature.key_name
        )));
    }
    verify_signature_bytes(canonical_bytes, signature, trusted_public_keys, context)
}

fn verify_signature_bytes(
    canonical_bytes: &[u8],
    signature: &DetachedSignature,
    trusted_public_keys: &[VerifyingKey],
    context: &str,
) -> Result<String, RunError> {
    let canonical_text = std::str::from_utf8(canonical_bytes)
        .map_err(|err| RunError::Internal(format!("{context} canonical bytes are not UTF-8: {err}")))?;
    let signature_text = encode_detached_signature(signature);
    let signature_ref = SignatureRef::parse(&signature_text)
        .map_err(|err| RunError::Internal(format!("encoding {context} signature: {err}")))?;

    let mut found_named_key = false;
    for trusted_key in trusted_public_keys {
        if trusted_key.name() != signature.key_name {
            continue;
        }
        found_named_key = true;
        if trusted_key.verify(canonical_text, &signature_ref) {
            return Ok(signature.key_name.clone());
        }
    }

    if !found_named_key {
        return Err(RunError::Build(format!(
            "{context} signer '{}' is missing from the trusted public key set",
            signature.key_name
        )));
    }

    Err(RunError::Build(format!(
        "{context} signature failed verification for signer '{}'",
        signature.key_name
    )))
}

fn build_release_attestation_from_bundle(manifest: &ReleaseEvidenceManifest) -> Result<ReleaseAttestation, RunError> {
    let release_evidence_manifest_digest_blake3 = release_manifest_digest(manifest)?;
    let binary_digests = published_binary_digests(manifest);

    Ok(ReleaseAttestation::new(ReleaseAttestationInit {
        release_id: manifest.release_id.clone(),
        release_evidence_manifest_digest_blake3,
        proof_bundle_digest_blake3: parse_attestation_digest(&manifest.proof_bundle.digest_blake3, "proof_bundle")?,
        proof_mode: manifest.proof_linkage.proof_mode.clone(),
        declared_effect_claims: None,
        observed_effect_facts: None,
        workflow: crunch_attestation::Workflow {
            command: manifest.workflow.command.clone(),
            version: manifest.workflow.version.clone(),
        },
        binary_digests,
    }))
}

fn release_manifest_digest(manifest: &ReleaseEvidenceManifest) -> Result<AttestationDigest, RunError> {
    let canonical_bytes = crunch_release_core::canonical_release_evidence_manifest(manifest.clone())
        .map_err(|err| RunError::Internal(format!("release evidence manifest canonicalization: {err}")))?;
    Ok(AttestationDigest::from_canonical_bytes(canonical_bytes))
}

fn parse_attestation_digest(value: &str, field_name: &str) -> Result<AttestationDigest, RunError> {
    AttestationDigest::parse_hex(value.to_string())
        .map_err(|err| RunError::Internal(format!("parsing {field_name} digest '{value}': {err}")))
}

fn published_binary_digests(manifest: &ReleaseEvidenceManifest) -> Vec<BinaryDigest> {
    manifest
        .binaries
        .iter()
        .map(|artifact| BinaryDigest {
            name: artifact.relative_path.clone(),
            algorithm: BLAKE3_ALGORITHM_NAME.to_string(),
            digest: artifact.digest_blake3.clone(),
        })
        .collect()
}

fn sign_detached_message(message: &[u8], keypair: &KeyPair) -> DetachedSignature {
    let signature = keypair.signing_key.sign(message);
    DetachedSignature {
        key_name: signature.name().to_string(),
        signature_bytes: *signature.bytes(),
    }
}

fn prepare_verification_dir(dir: &Path) -> Result<(), RunError> {
    if dir.exists() && !dir.is_dir() {
        return Err(RunError::Internal(format!("release verification path is not a directory: {}", dir.display())));
    }
    std::fs::create_dir_all(dir).map_err(|err| RunError::Internal(format!("creating {}: {err}", dir.display())))?;
    let witnesses_dir = dir.join(WITNESSES_DIR_NAME);
    std::fs::create_dir_all(&witnesses_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", witnesses_dir.display())))?;
    Ok(())
}

pub(crate) fn prepare_witness_dir(verification_dir: &Path) -> Result<PathBuf, RunError> {
    prepare_verification_dir(verification_dir)?;
    let witness_dir = verification_dir.join(WITNESSES_DIR_NAME);
    assert!(witness_dir.starts_with(verification_dir), "witness dir must stay under verification dir");
    Ok(witness_dir)
}

fn resolve_witness_identity(requested_identity: Option<&str>, signer_key_name: &str) -> Result<String, RunError> {
    let identity = match requested_identity {
        Some(identity) => identity.trim(),
        None => signer_key_name,
    };
    validate_witness_identity(identity)?;
    Ok(identity.to_string())
}

pub(crate) fn validate_witness_identity(identity: &str) -> Result<(), RunError> {
    let identity_len_bytes = identity.len();
    if identity.is_empty() {
        return Err(RunError::Internal("witness identity must not be empty".to_string()));
    }
    if identity == "." || identity == ".." {
        return Err(RunError::Internal(format!("witness identity must not be '{}'", identity)));
    }
    if identity.contains('/') || identity.contains('\\') {
        return Err(RunError::Internal(format!("witness identity must not contain path separators: {}", identity)));
    }
    if identity.chars().any(|character| character.is_control()) {
        return Err(RunError::Internal(format!(
            "witness identity must not contain control characters: {:?}",
            identity
        )));
    }
    if identity_len_bytes > MAX_WITNESS_IDENTITY_BYTES {
        return Err(RunError::Internal(format!(
            "witness identity exceeds {} bytes: {}",
            MAX_WITNESS_IDENTITY_BYTES, identity_len_bytes
        )));
    }
    Ok(())
}

fn compute_rebuilt_binary_digests(
    release_attestation: &ReleaseAttestation,
    rebuilt_binary_paths: &[PathBuf],
) -> Result<Vec<BinaryDigest>, RunError> {
    let expected_count_u32 = u32::try_from(release_attestation.binary_digests.len())
        .map_err(|_| RunError::Internal("published release binary count overflowed u32".to_string()))?;
    let actual_count_u32 = u32::try_from(rebuilt_binary_paths.len())
        .map_err(|_| RunError::Internal("rebuilt binary count overflowed u32".to_string()))?;
    if expected_count_u32 != actual_count_u32 {
        return Err(RunError::Internal(format!(
            "rebuilt binary count mismatch: release attestation expects {expected_count_u32}, got {actual_count_u32}"
        )));
    }

    let mut rebuilt_digests = Vec::with_capacity(rebuilt_binary_paths.len());
    for (published_digest, rebuilt_path) in release_attestation.binary_digests.iter().zip(rebuilt_binary_paths.iter()) {
        rebuilt_digests.push(build_rebuilt_binary_digest(published_digest, rebuilt_path)?);
    }
    assert_eq!(rebuilt_digests.len(), rebuilt_binary_paths.len(), "rebuilt digest list must align with inputs");
    Ok(rebuilt_digests)
}

fn build_rebuilt_binary_digest(published_digest: &BinaryDigest, rebuilt_path: &Path) -> Result<BinaryDigest, RunError> {
    let digest = compute_path_blake3_digest(rebuilt_path)?;
    Ok(BinaryDigest {
        name: published_digest.name.clone(),
        algorithm: BLAKE3_ALGORITHM_NAME.to_string(),
        digest,
    })
}

fn read_release_attestation_file(path: &Path) -> Result<ReleaseAttestation, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let attestation: ReleaseAttestation = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    canonical_release_bytes(&attestation)?;
    Ok(attestation)
}

fn read_witness_attestation_file(path: &Path) -> Result<WitnessAttestation, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let attestation: WitnessAttestation = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    canonical_witness_bytes(&attestation)?;
    Ok(attestation)
}

fn witness_json_paths(dir: &Path) -> Result<Vec<PathBuf>, RunError> {
    let witnesses_dir = dir.join(WITNESSES_DIR_NAME);
    if !witnesses_dir.exists() {
        return Ok(Vec::new());
    }
    if !witnesses_dir.is_dir() {
        return Err(RunError::Internal(format!(
            "release verification witnesses path is not a directory: {}",
            witnesses_dir.display()
        )));
    }

    let mut paths = Vec::new();
    for entry_result in std::fs::read_dir(&witnesses_dir)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", witnesses_dir.display())))?
    {
        let entry = entry_result
            .map_err(|err| RunError::Internal(format!("reading {} entry: {err}", witnesses_dir.display())))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let count_u32 = u32::try_from(paths.len())
                .map_err(|_| RunError::Internal("witness show path count overflowed u32".to_string()))?;
            if count_u32 >= MAX_WITNESS_SHOW_FILES {
                return Err(RunError::Internal(format!(
                    "release verification witness listing exceeds {MAX_WITNESS_SHOW_FILES} files"
                )));
            }
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn canonical_release_bytes(attestation: &ReleaseAttestation) -> Result<Vec<u8>, RunError> {
    attestation
        .canonical_bytes()
        .map_err(|err| RunError::Build(format!("release attestation canonicalization: {err}")))
}

fn canonical_witness_bytes(attestation: &WitnessAttestation) -> Result<Vec<u8>, RunError> {
    attestation
        .canonical_bytes()
        .map_err(|err| RunError::Build(format!("witness attestation canonicalization: {err}")))
}

fn canonical_release_digest_hex(attestation: &ReleaseAttestation) -> Result<String, RunError> {
    attestation
        .canonical_digest()
        .map(|digest| digest.to_hex())
        .map_err(|err| RunError::Build(format!("release attestation digest: {err}")))
}

#[cfg(test)]
mod tests {
    use crunch_attestation::RebuildEnvironmentSummary;
    use crunch_attestation::SignatureSuite;
    use crunch_release_core::BundledArtifact;
    use crunch_release_core::BundledArtifactKind;
    use crunch_release_core::CLAIM_SCOPE_PACKAGED_INTEGRITY;
    use crunch_release_core::DEFAULT_PROOF_WORKFLOW_COMMAND;
    use crunch_release_core::DEFAULT_PROOF_WORKFLOW_VERSION;
    use crunch_release_core::FULL_SELF_HOSTING_PROOF_SCHEMA;
    use crunch_release_core::RELEASE_EVIDENCE_SCHEMA;
    use crunch_release_core::ReleaseProofLinkage;
    use crunch_release_core::ReleaseWorkflowIdentity;

    use super::*;

    const TEST_KEYPAIR: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    fn renamed_keypair(key_name: &str) -> crunch_build::KeyPair {
        let (_generated_keypair, generated_line) = crunch_build::generate_keypair();
        let (_generated_name, encoded_keypair) = generated_line
            .split_once(':')
            .unwrap_or_else(|| panic!("generated signing key must contain ':' separator: {generated_line}"));
        let renamed_line = format!("{key_name}:{encoded_keypair}");
        crunch_build::load_keypair(&renamed_line)
            .unwrap_or_else(|err| panic!("renamed signing key must stay parseable: {err}"))
    }

    #[test]
    fn build_release_attestation_uses_manifest_digests_and_relative_paths() {
        let manifest = sample_manifest();
        let manifest_digest = release_manifest_digest(&manifest).unwrap();

        let attestation = build_release_attestation_from_bundle(&manifest).unwrap();

        assert_eq!(attestation.release_id, manifest.release_id);
        assert_eq!(attestation.release_evidence_manifest_digest_blake3, manifest_digest);
        assert_eq!(attestation.proof_bundle_digest_blake3.to_hex(), manifest.proof_bundle.digest_blake3);
        assert_eq!(attestation.binary_digests.len(), 1);
        assert_eq!(attestation.binary_digests[0].name, manifest.binaries[0].relative_path);
        assert_eq!(attestation.binary_digests[0].algorithm, BLAKE3_ALGORITHM_NAME);
    }

    #[test]
    fn verify_signature_bytes_accepts_matching_trusted_key() {
        let keypair = crunch_build::load_keypair(TEST_KEYPAIR).unwrap();
        let attestation = sample_release_attestation();
        let canonical_bytes = canonical_release_bytes(&attestation).unwrap();
        let signature = sign_detached_message(&canonical_bytes, &keypair);

        let signer =
            verify_signature_bytes(&canonical_bytes, &signature, &[keypair.verifying_key], "release attestation")
                .unwrap();

        assert_eq!(signer, "cache.example.com-1");
    }

    #[test]
    fn verify_signature_bytes_accepts_later_matching_key_when_names_collide() {
        let keypair = crunch_build::load_keypair(TEST_KEYPAIR).unwrap();
        let colliding_keypair = renamed_keypair("cache.example.com-1");
        let attestation = sample_release_attestation();
        let canonical_bytes = canonical_release_bytes(&attestation).unwrap();
        let signature = sign_detached_message(&canonical_bytes, &keypair);

        let signer = verify_signature_bytes(
            &canonical_bytes,
            &signature,
            &[colliding_keypair.verifying_key, keypair.verifying_key],
            "release attestation",
        )
        .unwrap();

        assert_eq!(signer, "cache.example.com-1");
    }

    #[test]
    fn verify_signature_bytes_rejects_colliding_names_when_no_key_matches() {
        let keypair = crunch_build::load_keypair(TEST_KEYPAIR).unwrap();
        let colliding_keypair = renamed_keypair("cache.example.com-1");
        let attestation = sample_release_attestation();
        let canonical_bytes = canonical_release_bytes(&attestation).unwrap();
        let signature = sign_detached_message(&canonical_bytes, &keypair);

        let err = verify_signature_bytes(
            &canonical_bytes,
            &signature,
            &[colliding_keypair.verifying_key],
            "release attestation",
        )
        .unwrap_err();

        assert!(
            err.message()
                .contains("release attestation signature failed verification for signer 'cache.example.com-1'")
        );
    }

    #[test]
    fn verify_signature_with_policy_names_rejects_untrusted_policy_signer() {
        let keypair = crunch_build::load_keypair(TEST_KEYPAIR).unwrap();
        let attestation = sample_release_attestation();
        let canonical_bytes = canonical_release_bytes(&attestation).unwrap();
        let signature = sign_detached_message(&canonical_bytes, &keypair);

        let err = verify_signature_with_policy_names(
            &canonical_bytes,
            &signature,
            &[keypair.verifying_key],
            &["other-signer".to_string()],
            "release attestation",
        )
        .unwrap_err();

        assert!(
            err.message()
                .contains("release attestation signer 'cache.example.com-1' is not trusted by policy.json")
        );
    }

    #[test]
    fn trusted_witness_identity_filter_allows_empty_allowlist() {
        let witness = WitnessAttestation::new(
            AttestationDigest::from_canonical_bytes(b"release".to_vec()),
            "witness-a".to_string(),
            sample_release_attestation().binary_digests,
            RebuildEnvironmentSummary {
                system: "x86_64-linux".to_string(),
                toolchain: "rust-1.91.1".to_string(),
                host_class: "nixos-25.05".to_string(),
            },
        );
        assert_eq!(witness.signature_suite, SignatureSuite::Ed25519DetachedV1);

        let allow_all =
            ReleasePolicy::new(1, "witness_identity".to_string(), vec!["release-signer".to_string()], Vec::new());
        let allow_only_b =
            ReleasePolicy::new(1, "witness_identity".to_string(), vec!["release-signer".to_string()], vec![
                "witness-b".to_string(),
            ]);

        assert!(is_trusted_witness_identity(&allow_all, &witness));
        assert!(!is_trusted_witness_identity(&allow_only_b, &witness));
    }

    #[test]
    fn validate_witness_identity_rejects_control_characters() {
        let err = validate_witness_identity("witness\nname").unwrap_err();

        assert!(err.message().contains("control characters"));
        assert!(err.message().contains("witness\\nname"));
    }

    #[test]
    fn validate_witness_identity_rejects_overlong_names() {
        let err = validate_witness_identity(&"w".repeat(MAX_WITNESS_IDENTITY_BYTES.saturating_add(1))).unwrap_err();

        assert!(err.message().contains("witness identity exceeds"));
        assert!(err.message().contains(&MAX_WITNESS_IDENTITY_BYTES.to_string()));
    }

    #[test]
    fn build_policy_file_contents_self_proof_only_drops_witnesses() {
        let created = build_policy_file_contents(
            PolicyInitProfile::SelfProofOnly,
            &[
                "release-b".to_string(),
                "release-a".to_string(),
                "release-a".to_string(),
            ],
            &[],
        )
        .unwrap();

        assert_eq!(created.policy.min_matching_witnesses, SELF_PROOF_ONLY_QUORUM);
        assert_eq!(created.policy.trusted_release_signers, vec!["release-a".to_string(), "release-b".to_string()]);
        assert!(created.policy.trusted_witness_signers.is_empty());
        assert_eq!(created.revocations, ReleaseRevocations::empty());
    }

    #[test]
    fn build_policy_file_contents_single_witness_requires_identity() {
        let err =
            build_policy_file_contents(PolicyInitProfile::SingleWitness, &["release-a".to_string()], &[]).unwrap_err();

        assert!(err.message().contains("single-witness profile requires at least one trusted witness identity"));
    }

    #[test]
    fn build_policy_file_contents_self_proof_only_rejects_identity() {
        let err = build_policy_file_contents(PolicyInitProfile::SelfProofOnly, &["release-a".to_string()], &[
            "witness-a".to_string(),
        ])
        .unwrap_err();

        assert!(err.message().contains("self-proof-only profile does not accept trusted witness identities"));
    }

    fn sample_manifest() -> ReleaseEvidenceManifest {
        ReleaseEvidenceManifest {
            schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
            release_id: "mantle-0.1.0-rc1".to_string(),
            claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
            workflow: ReleaseWorkflowIdentity {
                command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
                version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            },
            source_archive: sample_artifact(BundledArtifactKind::File, "source/mantle-src.tar", 1),
            source_acquisition: None,
            binaries: vec![sample_artifact(BundledArtifactKind::File, "binaries/01-mantle", 2)],
            proof_bundle: sample_artifact(BundledArtifactKind::Directory, "proof/self-hosting", 3),
            prerequisite_inventory: sample_artifact(BundledArtifactKind::File, "proof/inventory.md", 4),
            provider_fixed_point_proof: None,
            reproducibility_report: None,
            deterministic_build_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            independent_agreement_report: None,
            external_evidence: vec![],
            kani_toolchain_evidence: vec![],
            proof_linkage: ReleaseProofLinkage {
                release_id: "mantle-0.1.0-rc1".to_string(),
                source_archive_digest_blake3: sample_digest(1),
                proof_bundle_schema: FULL_SELF_HOSTING_PROOF_SCHEMA.to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "legacy-fetch".to_string(),
                staged_source: "src".to_string(),
                stage2_binary_digest_blake3: sample_digest(2),
                prerequisite_inventory_digest_blake3: sample_digest(4),
                proof_manifest_digest_blake3: sample_digest(9),
            },
            provenance_coverage: None,
        }
    }

    fn sample_release_attestation() -> ReleaseAttestation {
        build_release_attestation_from_bundle(&sample_manifest()).unwrap()
    }

    fn sample_artifact(kind: BundledArtifactKind, relative_path: &str, seed: u8) -> BundledArtifact {
        BundledArtifact {
            kind,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn sample_digest(seed: u8) -> String {
        let nibble = format!("{:x}", seed % 16);
        nibble.repeat(64)
    }
}
