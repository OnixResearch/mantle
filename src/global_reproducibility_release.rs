use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA;
use crunch_release_core::GlobalReproducibilityPolicy;
use crunch_release_core::GlobalReproducibilityUniverse;
use crunch_release_core::GlobalSurfaceEvidence;
use crunch_release_core::GlobalWitnessEvidence;
use crunch_release_core::GlobalWitnessTrustStatus;
use crunch_release_core::global_reproducibility_policy_digest_blake3;
use crunch_release_core::global_reproducibility_universe_digest_blake3;
use serde::Deserialize;

use crate::errors::RunError;

const RELEASE_SURFACE_EVIDENCE_OUTPUT_KIND: &str = "mantle-global-reproducibility-release-surface-evidence-v1";
const RELEASE_MANIFEST_RELATIVE_PATH: &str = "manifest.json";
const SELF_HOSTING_MANIFEST_RELATIVE_PATH: &str = "proof/self-hosting/manifest.json";
const SELF_HOSTING_SUMMARY_RELATIVE_PATH: &str = "proof/self-hosting/summary.txt";
const PROVIDER_FIXED_POINT_META_RELATIVE_PATH: &str = "proof/provider-fixed-point/meta.json";
const RELEASE_ATTESTATION_RELATIVE_PATH: &str = "release-attestation.json";
const WITNESSES_RELATIVE_DIR: &str = "witnesses";
const DIGEST_ALGORITHM_BLAKE3: &str = "blake3";
const STRICT_HERMETICITY_MODE: &str = "strict";
const EMPTY_FALLBACK_EVENTS: &str = "[]";
const SELF_HOSTING_STAGE2_SURFACE_KIND: &str = "self-hosting-stage2";
const PROVIDER_FIXED_POINT_SURFACE_KIND: &str = "provider-fixed-point-handoff";
const UNKNOWN_RELEASE_SURFACE_KIND: &str = "unknown-release-artifact";
const PROVIDER_FIXED_POINT_UNSUPPORTED_REASON: &str = "provider fixed-point handoff evidence is release-bounded but is not admitted as strict/fresh global reproducibility evidence";
const UNKNOWN_ARTIFACT_UNSUPPORTED_REASON: &str =
    "release artifact set is not present in the release evidence manifest";
const UNRECOGNIZED_ARTIFACT_UNSUPPORTED_REASON: &str =
    "release artifact does not match a recognized strict self-hosting stage2 proof surface";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseSurfaceEvidenceCommandOutput {
    pub universe_digest_blake3: String,
    pub policy_digest_blake3: String,
    pub evidence_path: PathBuf,
    pub evidence: Vec<GlobalSurfaceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseSurfaceEvidenceDerivationInput {
    universe: GlobalReproducibilityUniverse,
    policy: GlobalReproducibilityPolicy,
    universe_digest_blake3: String,
    policy_digest_blake3: String,
    release_verify_digest_blake3: String,
    manifest: ReleaseManifestSubset,
    release_attestation: ReleaseAttestationSubset,
    final_verify: FinalReleaseVerifySubset,
    witness_attestations: Vec<WitnessAttestationSubset>,
    self_hosting_manifest: Option<SelfHostingManifestSubset>,
    self_hosting_summary: SelfHostingSummaryFacts,
    provider_fixed_point_meta: Option<ProviderFixedPointMetaSubset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ReleaseManifestSubset {
    source_archive: ManifestArtifactSubset,
    binaries: Vec<ManifestArtifactSubset>,
    proof_bundle: ManifestArtifactSubset,
    #[serde(default)]
    provider_fixed_point_proof: Option<ManifestArtifactSubset>,
    #[serde(default)]
    proof_linkage: Option<ProofLinkageSubset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ManifestArtifactSubset {
    relative_path: String,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProofLinkageSubset {
    proof_manifest_digest_blake3: String,
    stage2_binary_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ReleaseAttestationSubset {
    binary_digests: Vec<NamedDigestSubset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct NamedDigestSubset {
    name: String,
    algorithm: String,
    digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FinalReleaseVerifySubset {
    independent_agreement_witnesses: Vec<FinalWitnessSubset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FinalWitnessSubset {
    witness_identity: String,
    signature_valid: bool,
    digest_match: bool,
    independence_domain: String,
    policy_counted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct WitnessAttestationSubset {
    witness_identity: String,
    rebuilt_digests: Vec<NamedDigestSubset>,
    rebuild_environment_summary: WitnessEnvironmentSubset,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct WitnessEnvironmentSubset {
    host_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SelfHostingManifestSubset {
    state_dirs: SelfHostingStateDirsSubset,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SelfHostingStateDirsSubset {
    stage0: String,
    stage2: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProviderFixedPointMetaSubset {
    fixed_point: bool,
    stage2: ProviderFixedPointStageSubset,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProviderFixedPointStageSubset {
    binary_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelfHostingSummaryFacts {
    stage2_strict: bool,
    stage2_no_fallbacks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SurfaceClassification {
    kind: &'static str,
    strict_hermeticity: bool,
    fresh_rebuild_store: bool,
    toolchain_provenance_digest_blake3: Option<String>,
    hermeticity_evidence_digest_blake3: Option<String>,
    unsupported_reason: Option<String>,
}

pub(crate) fn cmd_global_reproducibility_release_evidence(
    current_dir: &Path,
    json: bool,
    universe_path: PathBuf,
    policy_path: PathBuf,
    bundle_dir: PathBuf,
    verification_dir: PathBuf,
    release_verify_json: PathBuf,
    evidence_path: PathBuf,
) -> Result<(), RunError> {
    let output = derive_release_surface_evidence_from_paths(
        current_dir,
        universe_path,
        policy_path,
        bundle_dir,
        verification_dir,
        release_verify_json,
        evidence_path,
    )?;
    if json {
        print_release_surface_evidence_json(&output)?;
    } else {
        print_release_surface_evidence_human(&output);
    }
    Ok(())
}

fn derive_release_surface_evidence_from_paths(
    current_dir: &Path,
    universe_path: PathBuf,
    policy_path: PathBuf,
    bundle_dir: PathBuf,
    verification_dir: PathBuf,
    release_verify_json: PathBuf,
    evidence_path: PathBuf,
) -> Result<ReleaseSurfaceEvidenceCommandOutput, RunError> {
    let universe = read_json_file::<GlobalReproducibilityUniverse>(&resolve_input_path(current_dir, universe_path))?;
    let policy = read_json_file::<GlobalReproducibilityPolicy>(&resolve_input_path(current_dir, policy_path))?;
    let universe_digest_blake3 = global_reproducibility_universe_digest_blake3(universe.clone()).map_err(core_error)?;
    let policy_digest_blake3 = global_reproducibility_policy_digest_blake3(policy.clone()).map_err(core_error)?;
    let bundle_dir = resolve_input_path(current_dir, bundle_dir);
    let verification_dir = resolve_input_path(current_dir, verification_dir);
    let release_verify_json = resolve_input_path(current_dir, release_verify_json);
    let evidence_path = resolve_input_path(current_dir, evidence_path);
    let manifest = read_json_file::<ReleaseManifestSubset>(&bundle_dir.join(RELEASE_MANIFEST_RELATIVE_PATH))?;
    let release_attestation =
        read_json_file::<ReleaseAttestationSubset>(&verification_dir.join(RELEASE_ATTESTATION_RELATIVE_PATH))?;
    let final_verify = read_json_file::<FinalReleaseVerifySubset>(&release_verify_json)?;
    let release_verify_digest_blake3 = blake3_file_hex(&release_verify_json)?;
    let witness_attestations = read_witness_attestations(&verification_dir.join(WITNESSES_RELATIVE_DIR))?;
    let self_hosting_manifest =
        read_optional_json_file::<SelfHostingManifestSubset>(&bundle_dir.join(SELF_HOSTING_MANIFEST_RELATIVE_PATH))?;
    let self_hosting_summary = read_self_hosting_summary(&bundle_dir.join(SELF_HOSTING_SUMMARY_RELATIVE_PATH))?;
    let provider_fixed_point_meta = read_optional_json_file::<ProviderFixedPointMetaSubset>(
        &bundle_dir.join(PROVIDER_FIXED_POINT_META_RELATIVE_PATH),
    )?;
    let evidence = derive_release_surface_evidence(ReleaseSurfaceEvidenceDerivationInput {
        universe,
        policy,
        universe_digest_blake3: universe_digest_blake3.clone(),
        policy_digest_blake3: policy_digest_blake3.clone(),
        release_verify_digest_blake3,
        manifest,
        release_attestation,
        final_verify,
        witness_attestations,
        self_hosting_manifest,
        self_hosting_summary,
        provider_fixed_point_meta,
    })?;
    write_surface_evidence(&evidence_path, &evidence)?;
    Ok(ReleaseSurfaceEvidenceCommandOutput {
        universe_digest_blake3,
        policy_digest_blake3,
        evidence_path,
        evidence,
    })
}

fn derive_release_surface_evidence(
    input: ReleaseSurfaceEvidenceDerivationInput,
) -> Result<Vec<GlobalSurfaceEvidence>, RunError> {
    validate_policy_is_bound(&input.policy)?;
    let binary_digests = release_binary_digest_map(&input.manifest, &input.release_attestation)?;
    let counted_witnesses = counted_witness_map(&input.final_verify, &input.witness_attestations)?;
    let mut evidence = Vec::new();
    for surface in &input.universe.included_surfaces {
        let output_digest = binary_digests.get(&surface.release_artifact_set).cloned();
        let classification = classify_release_surface(&input, output_digest.as_deref());
        let witnesses = witnesses_for_surface(output_digest.as_deref(), &counted_witnesses);
        evidence.push(GlobalSurfaceEvidence {
            schema: GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA.to_string(),
            surface_id: surface.id.clone(),
            universe_digest_blake3: input.universe_digest_blake3.clone(),
            policy_digest_blake3: input.policy_digest_blake3.clone(),
            action_receipt_digest_blake3: Some(input.release_verify_digest_blake3.clone()),
            source_acquisition_digest_blake3: Some(input.manifest.source_archive.digest_blake3.clone()),
            toolchain_provenance_digest_blake3: classification.toolchain_provenance_digest_blake3,
            hermeticity_evidence_digest_blake3: classification.hermeticity_evidence_digest_blake3,
            output_digest_set_blake3: output_digest.into_iter().collect(),
            strict_hermeticity: classification.strict_hermeticity,
            fresh_rebuild_store: classification.fresh_rebuild_store,
            unsupported_reason: classification.unsupported_reason,
            witnesses,
        });
        assert!(!classification.kind.is_empty(), "surface classification kind must be named");
    }
    evidence.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));
    Ok(evidence)
}

fn validate_policy_is_bound(policy: &GlobalReproducibilityPolicy) -> Result<(), RunError> {
    if !policy.require_strict_hermeticity {
        return Err(internal("release-derived global evidence requires a strict-hermeticity policy"));
    }
    if !policy.require_fresh_rebuild_store {
        return Err(internal("release-derived global evidence requires a fresh-rebuild-store policy"));
    }
    Ok(())
}

fn release_binary_digest_map(
    manifest: &ReleaseManifestSubset,
    attestation: &ReleaseAttestationSubset,
) -> Result<BTreeMap<String, String>, RunError> {
    let mut attested = BTreeMap::new();
    for digest in &attestation.binary_digests {
        if digest.algorithm != DIGEST_ALGORITHM_BLAKE3 {
            return Err(internal(format!("release attestation digest for {} is not BLAKE3", digest.name)));
        }
        let previous = attested.insert(digest.name.clone(), digest.digest.clone());
        if previous.is_some() {
            return Err(internal(format!("duplicate release attestation digest for {}", digest.name)));
        }
    }
    let mut binaries = BTreeMap::new();
    for binary in &manifest.binaries {
        let Some(attested_digest) = attested.get(&binary.relative_path) else {
            return Err(internal(format!("release attestation is missing binary {}", binary.relative_path)));
        };
        if attested_digest != &binary.digest_blake3 {
            return Err(internal(format!("release attestation digest mismatch for {}", binary.relative_path)));
        }
        let previous = binaries.insert(binary.relative_path.clone(), binary.digest_blake3.clone());
        if previous.is_some() {
            return Err(internal(format!("duplicate release binary {}", binary.relative_path)));
        }
    }
    Ok(binaries)
}

fn counted_witness_map(
    final_verify: &FinalReleaseVerifySubset,
    witnesses: &[WitnessAttestationSubset],
) -> Result<BTreeMap<String, CountedWitness>, RunError> {
    let witness_by_identity = witnesses
        .iter()
        .map(|witness| (witness.witness_identity.clone(), witness))
        .collect::<BTreeMap<_, _>>();
    let mut counted = BTreeMap::new();
    for witness in &final_verify.independent_agreement_witnesses {
        if !witness.policy_counted {
            continue;
        }
        if !witness.signature_valid {
            continue;
        }
        if !witness.digest_match {
            continue;
        }
        let Some(attestation) = witness_by_identity.get(&witness.witness_identity) else {
            return Err(internal(format!("counted witness {} has no witness sidecar", witness.witness_identity)));
        };
        counted.insert(witness.witness_identity.clone(), CountedWitness {
            identity: witness.witness_identity.clone(),
            operator_domain: witness.independence_domain.clone(),
            host_class: attestation.rebuild_environment_summary.host_class.clone(),
            rebuilt_digests: witness_digest_map(attestation)?,
        });
    }
    Ok(counted)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CountedWitness {
    identity: String,
    operator_domain: String,
    host_class: String,
    rebuilt_digests: BTreeMap<String, String>,
}

fn witness_digest_map(witness: &WitnessAttestationSubset) -> Result<BTreeMap<String, String>, RunError> {
    let mut digests = BTreeMap::new();
    for digest in &witness.rebuilt_digests {
        if digest.algorithm != DIGEST_ALGORITHM_BLAKE3 {
            return Err(internal(format!("witness digest for {} is not BLAKE3", digest.name)));
        }
        let previous = digests.insert(digest.name.clone(), digest.digest.clone());
        if previous.is_some() {
            return Err(internal(format!("duplicate witness digest for {}", digest.name)));
        }
    }
    Ok(digests)
}

fn classify_release_surface(
    input: &ReleaseSurfaceEvidenceDerivationInput,
    output_digest: Option<&str>,
) -> SurfaceClassification {
    let Some(output_digest) = output_digest else {
        return unsupported_classification(UNKNOWN_RELEASE_SURFACE_KIND, UNKNOWN_ARTIFACT_UNSUPPORTED_REASON);
    };
    if is_self_hosting_stage2_surface(input, output_digest) {
        return self_hosting_stage2_classification(input);
    }
    if is_provider_fixed_point_surface(input, output_digest) {
        return provider_fixed_point_classification(input);
    }
    unsupported_classification(UNKNOWN_RELEASE_SURFACE_KIND, UNRECOGNIZED_ARTIFACT_UNSUPPORTED_REASON)
}

fn is_self_hosting_stage2_surface(input: &ReleaseSurfaceEvidenceDerivationInput, output_digest: &str) -> bool {
    input
        .manifest
        .proof_linkage
        .as_ref()
        .is_some_and(|linkage| linkage.stage2_binary_digest_blake3 == output_digest)
}

fn self_hosting_stage2_classification(input: &ReleaseSurfaceEvidenceDerivationInput) -> SurfaceClassification {
    let proof_linkage = input.manifest.proof_linkage.as_ref();
    let stage2_fresh = input.self_hosting_manifest.as_ref().is_some_and(|manifest| {
        !manifest.state_dirs.stage2.is_empty() && manifest.state_dirs.stage2 != manifest.state_dirs.stage0
    });
    let strict = input.self_hosting_summary.stage2_strict && input.self_hosting_summary.stage2_no_fallbacks;
    SurfaceClassification {
        kind: SELF_HOSTING_STAGE2_SURFACE_KIND,
        strict_hermeticity: strict,
        fresh_rebuild_store: stage2_fresh,
        toolchain_provenance_digest_blake3: Some(input.manifest.proof_bundle.digest_blake3.clone()),
        hermeticity_evidence_digest_blake3: proof_linkage.map(|linkage| linkage.proof_manifest_digest_blake3.clone()),
        unsupported_reason: None,
    }
}

fn is_provider_fixed_point_surface(input: &ReleaseSurfaceEvidenceDerivationInput, output_digest: &str) -> bool {
    input
        .provider_fixed_point_meta
        .as_ref()
        .is_some_and(|meta| meta.fixed_point && meta.stage2.binary_blake3 == output_digest)
}

fn provider_fixed_point_classification(input: &ReleaseSurfaceEvidenceDerivationInput) -> SurfaceClassification {
    let proof_digest =
        input.manifest.provider_fixed_point_proof.as_ref().map(|artifact| artifact.digest_blake3.clone());
    SurfaceClassification {
        kind: PROVIDER_FIXED_POINT_SURFACE_KIND,
        strict_hermeticity: false,
        fresh_rebuild_store: false,
        toolchain_provenance_digest_blake3: proof_digest.clone(),
        hermeticity_evidence_digest_blake3: proof_digest,
        unsupported_reason: Some(PROVIDER_FIXED_POINT_UNSUPPORTED_REASON.to_string()),
    }
}

fn unsupported_classification(kind: &'static str, reason: &str) -> SurfaceClassification {
    SurfaceClassification {
        kind,
        strict_hermeticity: false,
        fresh_rebuild_store: false,
        toolchain_provenance_digest_blake3: None,
        hermeticity_evidence_digest_blake3: None,
        unsupported_reason: Some(reason.to_string()),
    }
}

fn witnesses_for_surface(
    output_digest: Option<&str>,
    counted_witnesses: &BTreeMap<String, CountedWitness>,
) -> Vec<GlobalWitnessEvidence> {
    let mut evidence = Vec::new();
    let Some(output_digest) = output_digest else {
        return evidence;
    };
    for witness in counted_witnesses.values() {
        if witness.rebuilt_digests.values().any(|digest| digest == output_digest) {
            evidence.push(GlobalWitnessEvidence {
                identity: witness.identity.clone(),
                operator_domain: witness.operator_domain.clone(),
                host_class: witness.host_class.clone(),
                perturbation_axes: Vec::new(),
                trust_status: GlobalWitnessTrustStatus::Valid,
                output_digest_set_blake3: vec![output_digest.to_string()],
            });
        }
    }
    evidence
}

fn read_witness_attestations(witness_dir: &Path) -> Result<Vec<WitnessAttestationSubset>, RunError> {
    if !witness_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(witness_dir)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", witness_dir.display())))?
    {
        let entry =
            entry.map_err(|err| RunError::Internal(format!("reading {} entry: {err}", witness_dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut witnesses = Vec::new();
    for path in paths {
        witnesses.push(read_json_file::<WitnessAttestationSubset>(&path)?);
    }
    Ok(witnesses)
}

fn read_self_hosting_summary(path: &Path) -> Result<SelfHostingSummaryFacts, RunError> {
    let text = read_optional_string_file(path)?.unwrap_or_default();
    Ok(parse_self_hosting_summary(&text))
}

fn parse_self_hosting_summary(text: &str) -> SelfHostingSummaryFacts {
    let mut stage2_strict = false;
    let mut stage2_no_fallbacks = false;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if key == "stage2_hermeticity_mode" {
            stage2_strict = value == STRICT_HERMETICITY_MODE;
        }
        if key == "stage2_fallback_events" {
            stage2_no_fallbacks = value == EMPTY_FALLBACK_EVENTS;
        }
    }
    SelfHostingSummaryFacts {
        stage2_strict,
        stage2_no_fallbacks,
    }
}

fn read_optional_json_file<T>(path: &Path) -> Result<Option<T>, RunError>
where T: serde::de::DeserializeOwned {
    if !path.exists() {
        return Ok(None);
    }
    read_json_file(path).map(Some)
}

fn read_optional_string_file(path: &Path) -> Result<Option<String>, RunError> {
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))
}

fn read_json_file<T>(path: &Path) -> Result<T, RunError>
where T: serde::de::DeserializeOwned {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
}

fn write_surface_evidence(path: &Path, evidence: &[GlobalSurfaceEvidence]) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    let bytes =
        serde_json::to_vec(evidence).map_err(|err| RunError::Internal(format!("serializing evidence: {err}")))?;
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn print_release_surface_evidence_json(output: &ReleaseSurfaceEvidenceCommandOutput) -> Result<(), RunError> {
    let rendered = serde_json::json!({
        "kind": RELEASE_SURFACE_EVIDENCE_OUTPUT_KIND,
        "universe_digest_blake3": output.universe_digest_blake3,
        "policy_digest_blake3": output.policy_digest_blake3,
        "evidence_path": output.evidence_path.display().to_string(),
        "surface_count": output.evidence.len(),
        "evidence": output.evidence,
    });
    println!(
        "{}",
        serde_json::to_string(&rendered)
            .map_err(|err| RunError::Internal(format!("serializing release surface evidence output: {err}")))?
    );
    Ok(())
}

fn print_release_surface_evidence_human(output: &ReleaseSurfaceEvidenceCommandOutput) {
    println!("release-derived global surface evidence: {} surface(s)", output.evidence.len());
    println!("surface evidence: {}", output.evidence_path.display());
    println!("universe digest: {}", output.universe_digest_blake3);
    println!("policy digest: {}", output.policy_digest_blake3);
    for evidence in &output.evidence {
        println!("  surface: {}", evidence.surface_id);
        if let Some(reason) = &evidence.unsupported_reason {
            println!("    unsupported: {reason}");
        }
    }
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn blake3_file_hex(path: &Path) -> Result<String, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn core_error(error: crunch_release_core::ReleaseEvidenceError) -> RunError {
    RunError::Internal(error.to_string())
}

fn internal(message: impl Into<String>) -> RunError {
    RunError::Internal(message.into())
}

#[cfg(test)]
mod tests {
    use crunch_release_core::GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA;
    use crunch_release_core::GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA;
    use crunch_release_core::GlobalReproducibilityClaimClass;
    use crunch_release_core::GlobalReproducibilityEvaluationInput;
    use crunch_release_core::evaluate_global_reproducibility;
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use serde_json::json;

    use super::*;

    const SAMPLE_SIZE_BYTES: usize = 32;
    const STAGE2_SURFACE_ID: &str = "stage2-surface";
    const PROVIDER_SURFACE_ID: &str = "provider-surface";
    const STAGE2_ARTIFACT: &str = "binaries/02-stage2-mantle";
    const PROVIDER_ARTIFACT: &str = "binaries/01-mantle";

    fn digest(seed: u8) -> String {
        blake3::hash(&[seed; SAMPLE_SIZE_BYTES]).to_hex().to_string()
    }

    fn stage2_universe() -> GlobalReproducibilityUniverse {
        GlobalReproducibilityUniverse {
            schema: GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA.to_string(),
            name: "stage2 only".to_string(),
            included_surfaces: vec![surface(STAGE2_SURFACE_ID, STAGE2_ARTIFACT)],
            excluded_surfaces: Vec::new(),
        }
    }

    fn full_release_universe() -> GlobalReproducibilityUniverse {
        GlobalReproducibilityUniverse {
            schema: GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA.to_string(),
            name: "full release".to_string(),
            included_surfaces: vec![
                surface(PROVIDER_SURFACE_ID, PROVIDER_ARTIFACT),
                surface(STAGE2_SURFACE_ID, STAGE2_ARTIFACT),
            ],
            excluded_surfaces: Vec::new(),
        }
    }

    fn surface(id: &str, artifact: &str) -> crunch_release_core::GlobalBuildSurface {
        crunch_release_core::GlobalBuildSurface {
            id: id.to_string(),
            target_system: "x86_64-linux".to_string(),
            source_acquisition_mode: "release-source-archive".to_string(),
            toolchain_route: "release-proof".to_string(),
            cache_substitution_mode: "no-global-cache-claim".to_string(),
            release_artifact_set: artifact.to_string(),
        }
    }

    fn strict_policy() -> GlobalReproducibilityPolicy {
        GlobalReproducibilityPolicy {
            schema: GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA.to_string(),
            policy_id: "one-witness".to_string(),
            witness_policy: "one-domain".to_string(),
            required_operator_domains: 1,
            required_host_classes: 1,
            required_perturbation_axes: Vec::new(),
            require_strict_hermeticity: true,
            require_fresh_rebuild_store: true,
        }
    }

    fn derivation_input(universe: GlobalReproducibilityUniverse) -> ReleaseSurfaceEvidenceDerivationInput {
        let policy = strict_policy();
        let universe_digest = global_reproducibility_universe_digest_blake3(universe.clone()).unwrap();
        let policy_digest = global_reproducibility_policy_digest_blake3(policy.clone()).unwrap();
        ReleaseSurfaceEvidenceDerivationInput {
            universe,
            policy,
            universe_digest_blake3: universe_digest,
            policy_digest_blake3: policy_digest,
            release_verify_digest_blake3: digest(1),
            manifest: ReleaseManifestSubset {
                source_archive: artifact("source/archive.tar", digest(2)),
                binaries: vec![
                    artifact(PROVIDER_ARTIFACT, digest(10)),
                    artifact(STAGE2_ARTIFACT, digest(20)),
                ],
                proof_bundle: artifact("proof/self-hosting", digest(3)),
                provider_fixed_point_proof: Some(artifact("proof/provider-fixed-point", digest(4))),
                proof_linkage: Some(ProofLinkageSubset {
                    proof_manifest_digest_blake3: digest(5),
                    stage2_binary_digest_blake3: digest(20),
                }),
            },
            release_attestation: ReleaseAttestationSubset {
                binary_digests: vec![
                    named_digest(PROVIDER_ARTIFACT, digest(10)),
                    named_digest(STAGE2_ARTIFACT, digest(20)),
                ],
            },
            final_verify: FinalReleaseVerifySubset {
                independent_agreement_witnesses: vec![FinalWitnessSubset {
                    witness_identity: "aspen".to_string(),
                    signature_valid: true,
                    digest_match: true,
                    independence_domain: "aspen-domain".to_string(),
                    policy_counted: true,
                }],
            },
            witness_attestations: vec![WitnessAttestationSubset {
                witness_identity: "aspen".to_string(),
                rebuilt_digests: vec![
                    named_digest(PROVIDER_ARTIFACT, digest(10)),
                    named_digest(STAGE2_ARTIFACT, digest(20)),
                ],
                rebuild_environment_summary: WitnessEnvironmentSubset {
                    host_class: "nixos".to_string(),
                },
            }],
            self_hosting_manifest: Some(SelfHostingManifestSubset {
                state_dirs: SelfHostingStateDirsSubset {
                    stage0: "state0".to_string(),
                    stage2: "state2".to_string(),
                },
            }),
            self_hosting_summary: SelfHostingSummaryFacts {
                stage2_strict: true,
                stage2_no_fallbacks: true,
            },
            provider_fixed_point_meta: Some(ProviderFixedPointMetaSubset {
                fixed_point: true,
                stage2: ProviderFixedPointStageSubset {
                    binary_blake3: digest(10),
                },
            }),
        }
    }

    fn artifact(relative_path: &str, digest_blake3: String) -> ManifestArtifactSubset {
        ManifestArtifactSubset {
            relative_path: relative_path.to_string(),
            digest_blake3,
        }
    }

    fn named_digest(name: &str, digest_value: String) -> NamedDigestSubset {
        NamedDigestSubset {
            name: name.to_string(),
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            digest: digest_value,
        }
    }

    fn evaluate_generated(
        universe: GlobalReproducibilityUniverse,
        evidence: Vec<GlobalSurfaceEvidence>,
    ) -> crunch_release_core::GlobalReproducibilityReport {
        let policy = strict_policy();
        evaluate_global_reproducibility(GlobalReproducibilityEvaluationInput {
            universe: universe.clone(),
            policy: policy.clone(),
            universe_digest_blake3: global_reproducibility_universe_digest_blake3(universe).unwrap(),
            policy_digest_blake3: global_reproducibility_policy_digest_blake3(policy).unwrap(),
            surface_evidence: evidence,
        })
        .unwrap()
    }

    #[test]
    fn release_derived_stage2_surface_evaluates_eligible() {
        let universe = stage2_universe();
        let evidence = derive_release_surface_evidence(derivation_input(universe.clone())).unwrap();
        let report = evaluate_generated(universe, evidence.clone());

        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].surface_id, STAGE2_SURFACE_ID);
        assert_eq!(evidence[0].unsupported_reason, None);
        assert!(evidence[0].strict_hermeticity);
        assert!(evidence[0].fresh_rebuild_store);
        assert_eq!(report.claim_class, GlobalReproducibilityClaimClass::Eligible);
    }

    #[test]
    fn release_derived_full_release_blocks_provider_fixed_point_surface() {
        let universe = full_release_universe();
        let evidence = derive_release_surface_evidence(derivation_input(universe.clone())).unwrap();
        let report = evaluate_generated(universe, evidence.clone());
        let provider = evidence.iter().find(|item| item.surface_id == PROVIDER_SURFACE_ID).unwrap();
        let stage2 = evidence.iter().find(|item| item.surface_id == STAGE2_SURFACE_ID).unwrap();

        assert_eq!(evidence.len(), 2);
        assert!(provider.unsupported_reason.as_ref().unwrap().contains("provider fixed-point"));
        assert!(!provider.strict_hermeticity);
        assert!(!provider.fresh_rebuild_store);
        assert_eq!(stage2.unsupported_reason, None);
        assert_eq!(report.claim_class, GlobalReproducibilityClaimClass::Blocked);
        assert!(report.blockers.iter().any(|blocker| blocker.surface_id.as_deref() == Some(PROVIDER_SURFACE_ID)));
    }

    #[test]
    fn self_hosting_summary_parser_requires_strict_and_empty_fallbacks() {
        let facts = parse_self_hosting_summary("stage2_hermeticity_mode: strict\nstage2_fallback_events: []\n");
        let weak =
            parse_self_hosting_summary("stage2_hermeticity_mode: practical\nstage2_fallback_events: [\"fallback\"]\n");

        assert!(facts.stage2_strict);
        assert!(facts.stage2_no_fallbacks);
        assert!(!weak.stage2_strict);
        assert!(!weak.stage2_no_fallbacks);
    }

    #[test]
    fn shell_writes_release_surface_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let universe_path = temp.path().join("universe.json");
        let policy_path = temp.path().join("policy.json");
        let bundle_dir = temp.path().join("bundle");
        let verification_dir = temp.path().join("verification");
        let witness_dir = verification_dir.join(WITNESSES_RELATIVE_DIR);
        let release_verify_json = temp.path().join("release-verify.json");
        let evidence_path = temp.path().join("out/evidence.json");
        std::fs::create_dir_all(bundle_dir.join("proof/self-hosting")).unwrap();
        std::fs::create_dir_all(bundle_dir.join("proof/provider-fixed-point")).unwrap();
        std::fs::create_dir_all(&witness_dir).unwrap();
        write_json(&universe_path, &stage2_universe()).unwrap();
        write_json(&policy_path, &strict_policy()).unwrap();
        write_json(&bundle_dir.join(RELEASE_MANIFEST_RELATIVE_PATH), &manifest_json()).unwrap();
        write_json(&bundle_dir.join(SELF_HOSTING_MANIFEST_RELATIVE_PATH), &self_hosting_manifest_json()).unwrap();
        std::fs::write(
            bundle_dir.join(SELF_HOSTING_SUMMARY_RELATIVE_PATH),
            "stage2_hermeticity_mode: strict\nstage2_fallback_events: []\n",
        )
        .unwrap();
        write_json(&bundle_dir.join(PROVIDER_FIXED_POINT_META_RELATIVE_PATH), &provider_meta_json()).unwrap();
        write_json(&verification_dir.join(RELEASE_ATTESTATION_RELATIVE_PATH), &release_attestation_json()).unwrap();
        write_json(&release_verify_json, &final_verify_json()).unwrap();
        write_json(&witness_dir.join("aspen.json"), &witness_json()).unwrap();

        let output = derive_release_surface_evidence_from_paths(
            temp.path(),
            PathBuf::from("universe.json"),
            PathBuf::from("policy.json"),
            PathBuf::from("bundle"),
            PathBuf::from("verification"),
            PathBuf::from("release-verify.json"),
            PathBuf::from("out/evidence.json"),
        )
        .unwrap();

        assert_eq!(output.evidence.len(), 1);
        assert!(evidence_path.is_file());
    }

    fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
        let bytes = serde_json::to_vec(value).unwrap();
        std::fs::write(path, bytes)
    }

    fn manifest_json() -> serde_json::Value {
        json!({
            "source_archive": {"relative_path": "source/archive.tar", "digest_blake3": digest(2)},
            "binaries": [
                {"relative_path": PROVIDER_ARTIFACT, "digest_blake3": digest(10)},
                {"relative_path": STAGE2_ARTIFACT, "digest_blake3": digest(20)}
            ],
            "proof_bundle": {"relative_path": "proof/self-hosting", "digest_blake3": digest(3)},
            "provider_fixed_point_proof": {"relative_path": "proof/provider-fixed-point", "digest_blake3": digest(4)},
            "proof_linkage": {
                "proof_manifest_digest_blake3": digest(5),
                "stage2_binary_digest_blake3": digest(20)
            }
        })
    }

    fn self_hosting_manifest_json() -> serde_json::Value {
        json!({"state_dirs": {"stage0": "state0", "stage2": "state2"}})
    }

    fn provider_meta_json() -> serde_json::Value {
        json!({"fixed_point": true, "stage2": {"binary_blake3": digest(10)}})
    }

    fn release_attestation_json() -> serde_json::Value {
        json!({
            "binary_digests": [
                {"name": PROVIDER_ARTIFACT, "algorithm": "blake3", "digest": digest(10)},
                {"name": STAGE2_ARTIFACT, "algorithm": "blake3", "digest": digest(20)}
            ]
        })
    }

    fn final_verify_json() -> serde_json::Value {
        json!({
            "independent_agreement_witnesses": [{
                "witness_identity": "aspen",
                "signature_valid": true,
                "digest_match": true,
                "independence_domain": "aspen-domain",
                "policy_counted": true
            }]
        })
    }

    fn witness_json() -> serde_json::Value {
        json!({
            "witness_identity": "aspen",
            "rebuilt_digests": [
                {"name": PROVIDER_ARTIFACT, "algorithm": "blake3", "digest": digest(10)},
                {"name": STAGE2_ARTIFACT, "algorithm": "blake3", "digest": digest(20)}
            ],
            "rebuild_environment_summary": {"host_class": "nixos"}
        })
    }
}
