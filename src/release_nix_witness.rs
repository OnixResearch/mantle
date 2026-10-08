use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofVerdict;
use crunch_release_core::NixCrossBuilderArtifactDigest;
use crunch_release_core::NixCrossBuilderBuildPolicy;
use crunch_release_core::NixCrossBuilderWitnessReceipt;
use crunch_release_core::NixCrossBuilderWitnessReceiptInit;
use crunch_release_core::NixCrossBuilderWitnessVerdict;
use crunch_release_core::canonical_nix_cross_builder_witness_receipt;
use crunch_release_core::deterministic_build_proof_has_genuine_rebuild_authority;
use crunch_release_core::deterministic_build_proof_receipt_canonical_bytes;
use crunch_release_core::nix_cross_builder_witness_receipt_digest_blake3;

use crate::errors::RunError;
use crate::release_evidence::verify_release_evidence_bundle;

const DEFAULT_WITNESS_RELATIVE_PATH: &str = "witness/nix-cross-builder-witness.json";
const HASH_BUFFER_BYTES: usize = 8192;

const _: () = assert!(HASH_BUFFER_BYTES > 0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseNixWitnessRequest {
    pub bundle_dir: PathBuf,
    pub nix_output_dir: PathBuf,
    pub deterministic_proof_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub rust_toolchain_identity: String,
    pub target_triple: String,
    pub build_flags: Vec<String>,
    pub linker_identity: Option<String>,
    pub strip_debug_policy: String,
    pub source_date_epoch_policy: String,
    pub nix_derivation_identity: String,
    pub nix_output_identity: String,
    pub require_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseNixWitnessSummary {
    pub release_id: String,
    pub receipt_path: PathBuf,
    pub receipt_digest_blake3: String,
    pub comparison_verdict: NixCrossBuilderWitnessVerdict,
    pub proof_class: Option<String>,
    pub mantle_artifact_count: usize,
    pub nix_artifact_count: usize,
}

impl ReleaseNixWitnessSummary {
    pub(crate) fn verdict_label(&self) -> &'static str {
        match self.comparison_verdict {
            NixCrossBuilderWitnessVerdict::NixWitnessMatch => "nix-witness-match",
            NixCrossBuilderWitnessVerdict::CrossBuilderMismatch => "cross-builder-mismatch",
        }
    }

    pub(crate) fn matched(&self) -> bool {
        self.comparison_verdict == NixCrossBuilderWitnessVerdict::NixWitnessMatch
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedDeterministicProof {
    receipt: DeterministicBuildProofReceipt,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedArtifact {
    size_bytes: u64,
    digest_blake3: String,
}

/// Pure pre-effect receipt destination selection shared with the release plan.
pub(crate) fn resolve_witness_receipt_path(bundle_dir: &Path, requested: Option<&Path>) -> PathBuf {
    requested.map(Path::to_path_buf).unwrap_or_else(|| bundle_dir.join(DEFAULT_WITNESS_RELATIVE_PATH))
}

pub(crate) fn write_release_nix_cross_builder_witness(
    request: &ReleaseNixWitnessRequest,
) -> Result<ReleaseNixWitnessSummary, RunError> {
    debug_assert!(!DEFAULT_WITNESS_RELATIVE_PATH.is_empty());
    validate_witness_request(request)?;
    let manifest = verify_release_evidence_bundle(&request.bundle_dir)?;
    let deterministic_proof = load_canonical_deterministic_proof(&request.deterministic_proof_path)?;
    ensure_deterministic_proof_promotes(&deterministic_proof, &manifest)?;

    let (mantle_artifact_digests, nix_artifact_digests) =
        cross_builder_artifact_digests(&manifest, &request.nix_output_dir)?;

    let mut receipt = NixCrossBuilderWitnessReceipt::new(NixCrossBuilderWitnessReceiptInit {
        release_id: manifest.release_id.clone(),
        selected_artifact_identity: deterministic_proof.receipt.proof_unit.target_artifact_identity.clone(),
        mantle_deterministic_proof_receipt_digest_blake3: deterministic_proof.digest_blake3,
        mantle_artifact_digests,
        nix_artifact_digests,
        source_tree_digest_blake3: deterministic_proof.receipt.source_blake3.clone(),
        vendor_input_digest_blake3: deterministic_proof.receipt.vendor_blake3.clone(),
        build_policy: NixCrossBuilderBuildPolicy {
            rust_toolchain_identity: request.rust_toolchain_identity.clone(),
            target_triple: request.target_triple.clone(),
            build_flags: request.build_flags.clone(),
            linker_identity: request.linker_identity.clone(),
            strip_debug_policy: request.strip_debug_policy.clone(),
            source_date_epoch_policy: request.source_date_epoch_policy.clone(),
        },
        nix_derivation_identity: request.nix_derivation_identity.clone(),
        nix_output_identity: request.nix_output_identity.clone(),
    });
    let receipt_digest_blake3 = nix_cross_builder_witness_receipt_digest_blake3(receipt.clone()).map_err(core_error)?;
    receipt.receipt_blake3 = Some(receipt_digest_blake3.clone());
    let receipt = canonical_nix_cross_builder_witness_receipt(receipt).map_err(core_error)?;
    let receipt_bytes = serde_json::to_vec(&receipt)
        .map_err(|err| RunError::Internal(format!("serializing nix witness receipt: {err}")))?;
    let receipt_path = resolve_witness_receipt_path(&request.bundle_dir, request.output_path.as_deref());
    write_receipt(&receipt_path, &receipt_bytes)?;

    let summary = ReleaseNixWitnessSummary {
        release_id: manifest.release_id,
        receipt_path,
        receipt_digest_blake3,
        comparison_verdict: receipt.comparison_verdict,
        proof_class: receipt.proof_class,
        mantle_artifact_count: receipt.mantle_artifact_digests.len(),
        nix_artifact_count: receipt.nix_artifact_digests.len(),
    };
    if request.require_match && !summary.matched() {
        return Err(RunError::Internal(format!(
            "nix cross-builder witness required but verdict is {}",
            summary.verdict_label()
        )));
    }
    Ok(summary)
}

fn cross_builder_artifact_digests(
    manifest: &crunch_release_core::ReleaseEvidenceManifest,
    nix_output_dir: &Path,
) -> Result<(Vec<NixCrossBuilderArtifactDigest>, Vec<NixCrossBuilderArtifactDigest>), RunError> {
    let mantle_artifact_digests = manifest
        .binaries
        .iter()
        .map(|artifact| NixCrossBuilderArtifactDigest {
            name: artifact.relative_path.clone(),
            size_bytes: artifact.size_bytes,
            digest_blake3: artifact.digest_blake3.clone(),
        })
        .collect::<Vec<_>>();
    let nix_artifact_digests = manifest
        .binaries
        .iter()
        .map(|artifact| {
            let path = nix_output_dir.join(&artifact.relative_path);
            let observed = observed_artifact(&path)?;
            Ok(NixCrossBuilderArtifactDigest {
                name: artifact.relative_path.clone(),
                size_bytes: observed.size_bytes,
                digest_blake3: observed.digest_blake3,
            })
        })
        .collect::<Result<Vec<_>, RunError>>()?;
    debug_assert_eq!(mantle_artifact_digests.len(), manifest.binaries.len());
    debug_assert_eq!(nix_artifact_digests.len(), manifest.binaries.len());
    Ok((mantle_artifact_digests, nix_artifact_digests))
}

fn validate_witness_request(request: &ReleaseNixWitnessRequest) -> Result<(), RunError> {
    debug_assert!(!DEFAULT_WITNESS_RELATIVE_PATH.is_empty());
    if !request.bundle_dir.is_dir() {
        return Err(RunError::Internal(format!(
            "release bundle directory does not exist: {}",
            request.bundle_dir.display()
        )));
    }
    if !request.nix_output_dir.is_dir() {
        return Err(RunError::Internal(format!(
            "nix output directory does not exist: {}",
            request.nix_output_dir.display()
        )));
    }
    if !request.deterministic_proof_path.is_file() {
        return Err(RunError::Internal(format!(
            "deterministic proof artifact does not exist: {}",
            request.deterministic_proof_path.display()
        )));
    }
    validate_non_empty(&request.rust_toolchain_identity, "rust toolchain identity")?;
    validate_non_empty(&request.target_triple, "target triple")?;
    validate_non_empty(&request.strip_debug_policy, "strip/debug policy")?;
    validate_non_empty(&request.source_date_epoch_policy, "SOURCE_DATE_EPOCH policy")?;
    validate_non_empty(&request.nix_derivation_identity, "nix derivation identity")?;
    validate_non_empty(&request.nix_output_identity, "nix output identity")?;
    Ok(())
}

fn validate_non_empty(value: &str, field_name: impl AsRef<str>) -> Result<(), RunError> {
    let field_name = field_name.as_ref();
    if value.trim().is_empty() {
        return Err(RunError::Internal(format!("nix cross-builder witness {field_name} must not be empty")));
    }
    Ok(())
}

fn load_canonical_deterministic_proof(path: &Path) -> Result<VerifiedDeterministicProof, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let receipt: DeterministicBuildProofReceipt = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    let canonical = deterministic_build_proof_receipt_canonical_bytes(receipt.clone()).map_err(|err| {
        RunError::Internal(format!("validating deterministic proof artifact {}: {err}", path.display()))
    })?;
    if bytes != canonical {
        return Err(RunError::Internal(format!(
            "deterministic proof artifact is not canonical compact JSON: {}",
            path.display()
        )));
    }
    Ok(VerifiedDeterministicProof {
        receipt,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn ensure_deterministic_proof_promotes(
    proof: &VerifiedDeterministicProof,
    manifest: &crunch_release_core::ReleaseEvidenceManifest,
) -> Result<(), RunError> {
    debug_assert!(!DEFAULT_WITNESS_RELATIVE_PATH.is_empty());
    let is_genuine_rebuild = deterministic_build_proof_has_genuine_rebuild_authority(proof.receipt.clone())
        .map_err(|err| RunError::Internal(format!("validating deterministic proof rebuild authority: {err}")))?;
    if !is_genuine_rebuild {
        return Err(RunError::Internal(
            "nix cross-builder witness requires content-bound genuine rebuild authority; legacy path-bound evidence is non-promoting"
                .to_string(),
        ));
    }
    if proof.receipt.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch {
        return Err(RunError::Internal(format!(
            "nix cross-builder witness requires a deterministic proof with self-rebuild-match verdict, got {:?}",
            proof.receipt.verdict
        )));
    }
    if proof.receipt.source_blake3 != manifest.source_archive.digest_blake3 {
        return Err(RunError::Internal(
            "nix cross-builder witness deterministic proof source digest does not match release bundle source digest"
                .to_string(),
        ));
    }
    if proof.receipt.vendor_blake3 != manifest.proof_bundle.digest_blake3 {
        return Err(RunError::Internal(
            "nix cross-builder witness deterministic proof vendor digest does not match release bundle proof bundle digest"
                .to_string(),
        ));
    }
    let expected_digests = manifest.binaries.iter().map(|artifact| artifact.digest_blake3.clone()).collect::<Vec<_>>();
    let proof_digests = proof
        .receipt
        .runs
        .iter()
        .flat_map(|run| run.output_digests.iter().map(|digest| digest.digest_blake3.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    for digest in expected_digests {
        if !proof_digests.contains(&digest) {
            return Err(RunError::Internal(
                "nix cross-builder witness deterministic proof does not cover release artifact digest set".to_string(),
            ));
        }
    }
    Ok(())
}

fn observed_artifact(path: &Path) -> Result<ObservedArtifact, RunError> {
    if !path.is_file() {
        return Err(RunError::Internal(format!(
            "nix cross-builder witness expected Nix artifact file is missing: {}",
            path.display()
        )));
    }
    let (size_bytes, digest_blake3) = hash_file(path)?;
    Ok(ObservedArtifact {
        size_bytes,
        digest_blake3,
    })
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    let mut file = File::open(path).map_err(|err| RunError::Internal(format!("open {}: {err}", path.display())))?;
    let metadata = file.metadata().map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!(
            "nix cross-builder witness artifact is not a regular file: {}",
            path.display()
        )));
    }
    let buffer_size_bytes = u64::try_from(HASH_BUFFER_BYTES)
        .map_err(|_| RunError::Internal("hash buffer size does not fit u64".to_string()))?;
    let maximum_read_count = metadata
        .len()
        .div_ceil(buffer_size_bytes)
        .checked_add(1)
        .ok_or_else(|| RunError::Internal(format!("hash read count overflowed for {}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    debug_assert!(metadata.is_file());
    debug_assert!(maximum_read_count > 0);
    for _ in 0..maximum_read_count {
        let read_size_bytes = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if read_size_bytes == 0 {
            return Ok((metadata.len(), hasher.finalize().to_hex().to_string()));
        }
        hasher.update(&buffer[..read_size_bytes]);
    }
    Err(RunError::Internal(format!("file grew while hashing: {}", path.display())))
}

fn write_receipt(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn core_error(error: crunch_release_core::ReleaseEvidenceError) -> RunError {
    RunError::Internal(error.to_string())
}
