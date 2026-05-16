use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA: &str = "mantle-nix-cross-builder-witness-v1";
pub const NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS: &str = "nix-cross-builder-witness";
const MAX_WITNESS_ARTIFACT_DIGESTS_COUNT: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NixCrossBuilderWitnessVerdict {
    NixWitnessMatch,
    CrossBuilderMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixCrossBuilderArtifactDigest {
    pub name: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixCrossBuilderBuildPolicy {
    pub rust_toolchain_identity: String,
    pub target_triple: String,
    pub build_flags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linker_identity: Option<String>,
    pub strip_debug_policy: String,
    pub source_date_epoch_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixCrossBuilderWitnessReceipt {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_class: Option<String>,
    pub release_id: String,
    pub selected_artifact_identity: String,
    pub mantle_deterministic_proof_receipt_digest_blake3: String,
    pub mantle_artifact_digests: Vec<NixCrossBuilderArtifactDigest>,
    pub nix_artifact_digests: Vec<NixCrossBuilderArtifactDigest>,
    pub source_tree_digest_blake3: String,
    pub vendor_input_digest_blake3: String,
    pub build_policy: NixCrossBuilderBuildPolicy,
    pub nix_derivation_identity: String,
    pub nix_output_identity: String,
    pub comparison_verdict: NixCrossBuilderWitnessVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixCrossBuilderWitnessReceiptInit {
    pub release_id: String,
    pub selected_artifact_identity: String,
    pub mantle_deterministic_proof_receipt_digest_blake3: String,
    pub mantle_artifact_digests: Vec<NixCrossBuilderArtifactDigest>,
    pub nix_artifact_digests: Vec<NixCrossBuilderArtifactDigest>,
    pub source_tree_digest_blake3: String,
    pub vendor_input_digest_blake3: String,
    pub build_policy: NixCrossBuilderBuildPolicy,
    pub nix_derivation_identity: String,
    pub nix_output_identity: String,
}

impl NixCrossBuilderWitnessReceipt {
    pub fn new(init: NixCrossBuilderWitnessReceiptInit) -> Self {
        let mut receipt = Self {
            schema: NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA.to_string(),
            proof_class: None,
            release_id: init.release_id,
            selected_artifact_identity: init.selected_artifact_identity,
            mantle_deterministic_proof_receipt_digest_blake3: init.mantle_deterministic_proof_receipt_digest_blake3,
            mantle_artifact_digests: init.mantle_artifact_digests,
            nix_artifact_digests: init.nix_artifact_digests,
            source_tree_digest_blake3: init.source_tree_digest_blake3,
            vendor_input_digest_blake3: init.vendor_input_digest_blake3,
            build_policy: init.build_policy,
            nix_derivation_identity: init.nix_derivation_identity,
            nix_output_identity: init.nix_output_identity,
            comparison_verdict: NixCrossBuilderWitnessVerdict::CrossBuilderMismatch,
            receipt_blake3: None,
        };
        receipt.comparison_verdict = classify_cross_builder_witness(&receipt);
        receipt.proof_class = proof_class_for_verdict(receipt.comparison_verdict);
        receipt
    }
}

pub fn canonical_nix_cross_builder_witness_receipt(
    mut receipt: NixCrossBuilderWitnessReceipt,
) -> Result<NixCrossBuilderWitnessReceipt, ReleaseEvidenceError> {
    let provided_receipt_blake3 = receipt.receipt_blake3.take();
    validate_receipt_header(&receipt)?;
    receipt.schema = NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA.to_string();
    receipt.mantle_artifact_digests.sort_by(|left, right| left.name.cmp(&right.name));
    receipt.nix_artifact_digests.sort_by(|left, right| left.name.cmp(&right.name));
    validate_receipt_evidence(&receipt)?;
    let expected_verdict = classify_cross_builder_witness(&receipt);
    if receipt.comparison_verdict != expected_verdict {
        return Err(validation_error(format!(
            "nix cross-builder witness comparison_verdict {:?} does not match classified verdict {:?}",
            receipt.comparison_verdict, expected_verdict
        )));
    }
    let expected_proof_class = proof_class_for_verdict(expected_verdict);
    if receipt.proof_class != expected_proof_class {
        return Err(validation_error(format!(
            "nix cross-builder witness proof_class {:?} does not match classified verdict {:?}",
            receipt.proof_class, expected_verdict
        )));
    }
    if let Some(provided) = provided_receipt_blake3 {
        let digest = nix_cross_builder_witness_receipt_digest_blake3(receipt.clone())?;
        if provided != digest {
            return Err(validation_error(
                "nix cross-builder witness receipt_blake3 does not match canonical receipt bytes".to_string(),
            ));
        }
        receipt.receipt_blake3 = Some(provided);
    }
    Ok(receipt)
}

pub fn nix_cross_builder_witness_receipt_canonical_bytes(
    receipt: NixCrossBuilderWitnessReceipt,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let mut canonical = canonical_nix_cross_builder_witness_receipt(receipt)?;
    canonical.receipt_blake3 = None;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing nix cross-builder witness receipt: {err}")))
}

pub fn nix_cross_builder_witness_receipt_digest_blake3(
    receipt: NixCrossBuilderWitnessReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = nix_cross_builder_witness_receipt_canonical_bytes_without_digest(receipt)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn nix_cross_builder_witness_receipt_canonical_bytes_without_digest(
    mut receipt: NixCrossBuilderWitnessReceipt,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let mut canonical = canonical_nix_cross_builder_witness_receipt({
        receipt.receipt_blake3 = None;
        receipt
    })?;
    canonical.receipt_blake3 = None;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing nix cross-builder witness receipt: {err}")))
}

fn classify_cross_builder_witness(receipt: &NixCrossBuilderWitnessReceipt) -> NixCrossBuilderWitnessVerdict {
    let mut mantle = receipt.mantle_artifact_digests.clone();
    let mut nix = receipt.nix_artifact_digests.clone();
    mantle.sort_by(|left, right| left.name.cmp(&right.name));
    nix.sort_by(|left, right| left.name.cmp(&right.name));
    if mantle == nix {
        NixCrossBuilderWitnessVerdict::NixWitnessMatch
    } else {
        NixCrossBuilderWitnessVerdict::CrossBuilderMismatch
    }
}

fn proof_class_for_verdict(verdict: NixCrossBuilderWitnessVerdict) -> Option<String> {
    match verdict {
        NixCrossBuilderWitnessVerdict::NixWitnessMatch => Some(NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS.to_string()),
        NixCrossBuilderWitnessVerdict::CrossBuilderMismatch => None,
    }
}

fn validate_receipt_header(receipt: &NixCrossBuilderWitnessReceipt) -> Result<(), ReleaseEvidenceError> {
    if receipt.schema != NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA {
        return Err(validation_error(format!(
            "nix cross-builder witness schema must be {NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA}, got {}",
            receipt.schema
        )));
    }
    if let Some(proof_class) = &receipt.proof_class
        && proof_class != NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS
    {
        return Err(validation_error(format!(
            "nix cross-builder witness proof_class must be {NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS}, got {}",
            proof_class
        )));
    }
    validate_non_empty(&receipt.release_id, "release_id")?;
    validate_non_empty(&receipt.selected_artifact_identity, "selected_artifact_identity")?;
    validate_blake3_hex(
        &receipt.mantle_deterministic_proof_receipt_digest_blake3,
        "mantle_deterministic_proof_receipt_digest_blake3",
    )?;
    validate_blake3_hex(&receipt.source_tree_digest_blake3, "source_tree_digest_blake3")?;
    validate_blake3_hex(&receipt.vendor_input_digest_blake3, "vendor_input_digest_blake3")?;
    Ok(())
}

fn validate_receipt_evidence(receipt: &NixCrossBuilderWitnessReceipt) -> Result<(), ReleaseEvidenceError> {
    validate_artifact_digest_set(&receipt.mantle_artifact_digests, "mantle_artifact_digests")?;
    validate_artifact_digest_set(&receipt.nix_artifact_digests, "nix_artifact_digests")?;
    validate_build_policy(&receipt.build_policy)?;
    validate_non_empty(&receipt.nix_derivation_identity, "nix_derivation_identity")?;
    validate_non_empty(&receipt.nix_output_identity, "nix_output_identity")?;
    Ok(())
}

fn validate_artifact_digest_set(
    artifacts: &[NixCrossBuilderArtifactDigest],
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    let count = u32_count(artifacts.len(), &format!("nix cross-builder witness {field_name} count overflowed u32"))?;
    if count == 0 {
        return Err(validation_error(format!("nix cross-builder witness {field_name} must not be empty")));
    }
    if count > MAX_WITNESS_ARTIFACT_DIGESTS_COUNT {
        return Err(validation_error(format!(
            "nix cross-builder witness {field_name} records {count} artifacts, limit is {MAX_WITNESS_ARTIFACT_DIGESTS_COUNT}"
        )));
    }
    let mut seen_names = alloc::collections::BTreeSet::new();
    for artifact in artifacts {
        validate_non_empty(&artifact.name, &format!("{field_name}.name"))?;
        if !seen_names.insert(artifact.name.clone()) {
            return Err(validation_error(format!(
                "nix cross-builder witness {field_name} contains duplicate artifact {}",
                artifact.name
            )));
        }
        if artifact.size_bytes == 0 {
            return Err(validation_error(format!(
                "nix cross-builder witness {field_name}.{}.size_bytes must be non-zero",
                artifact.name
            )));
        }
        validate_blake3_hex(&artifact.digest_blake3, &format!("{field_name}.{}.digest_blake3", artifact.name))?;
    }
    Ok(())
}

fn validate_build_policy(policy: &NixCrossBuilderBuildPolicy) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty(&policy.rust_toolchain_identity, "build_policy.rust_toolchain_identity")?;
    validate_non_empty(&policy.target_triple, "build_policy.target_triple")?;
    validate_non_empty(&policy.strip_debug_policy, "build_policy.strip_debug_policy")?;
    validate_non_empty(&policy.source_date_epoch_policy, "build_policy.source_date_epoch_policy")?;
    for flag in &policy.build_flags {
        validate_non_empty(flag, "build_policy.build_flags[]")?;
    }
    if let Some(linker) = &policy.linker_identity {
        validate_non_empty(linker, "build_policy.linker_identity")?;
    }
    Ok(())
}

fn validate_non_empty(value: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if value.trim().is_empty() {
        return Err(validation_error(format!("nix cross-builder witness {field_name} must not be empty")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(crate::BLAKE3_HEX_LENGTH_CHARS)
    }

    fn artifact(name: &str, seed: u8) -> NixCrossBuilderArtifactDigest {
        NixCrossBuilderArtifactDigest {
            name: name.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn build_policy() -> NixCrossBuilderBuildPolicy {
        NixCrossBuilderBuildPolicy {
            rust_toolchain_identity: "rustc-1.91.1-x86_64-unknown-linux-musl".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            build_flags: vec![
                "RUSTFLAGS=-Ctarget-feature=+crt-static".to_string(),
                "--locked".to_string(),
            ],
            linker_identity: Some("clang-wrapper-21.1.8+mold-2.40.4".to_string()),
            strip_debug_policy: "strip-symbols-keep-build-id".to_string(),
            source_date_epoch_policy: "recorded-source-date-epoch=1".to_string(),
        }
    }

    fn sample_receipt() -> NixCrossBuilderWitnessReceipt {
        NixCrossBuilderWitnessReceipt::new(NixCrossBuilderWitnessReceiptInit {
            release_id: "mantle-0.1.0-rc1".to_string(),
            selected_artifact_identity: "mantle-linux-x86_64-musl".to_string(),
            mantle_deterministic_proof_receipt_digest_blake3: sample_digest(1),
            mantle_artifact_digests: vec![artifact("bin/mantle", 2)],
            nix_artifact_digests: vec![artifact("bin/mantle", 2)],
            source_tree_digest_blake3: sample_digest(3),
            vendor_input_digest_blake3: sample_digest(4),
            build_policy: build_policy(),
            nix_derivation_identity: "/nix/store/abc-mantle.drv".to_string(),
            nix_output_identity: "/nix/store/def-mantle-0.1.0".to_string(),
        })
    }

    #[test]
    fn canonical_receipt_is_compact_stable_and_digest_bound() {
        let receipt = sample_receipt();
        assert_eq!(receipt.comparison_verdict, NixCrossBuilderWitnessVerdict::NixWitnessMatch);
        let first = nix_cross_builder_witness_receipt_canonical_bytes(receipt.clone()).unwrap();
        let second = nix_cross_builder_witness_receipt_canonical_bytes(receipt.clone()).unwrap();
        assert_eq!(first, second);
        assert!(!first.contains(&b'\n'));

        let digest = nix_cross_builder_witness_receipt_digest_blake3(receipt.clone()).unwrap();
        let mut with_digest = receipt;
        with_digest.receipt_blake3 = Some(digest.clone());
        let canonical = canonical_nix_cross_builder_witness_receipt(with_digest).unwrap();
        assert_eq!(canonical.receipt_blake3.as_deref(), Some(digest.as_str()));
    }

    #[test]
    fn mismatch_is_fail_closed_to_cross_builder_mismatch_verdict() {
        let mut receipt = sample_receipt();
        receipt.nix_artifact_digests[0].digest_blake3 = sample_digest(9);
        receipt.comparison_verdict = NixCrossBuilderWitnessVerdict::CrossBuilderMismatch;
        receipt.proof_class = None;
        let canonical = canonical_nix_cross_builder_witness_receipt(receipt).unwrap();
        assert_eq!(canonical.comparison_verdict, NixCrossBuilderWitnessVerdict::CrossBuilderMismatch);
    }

    #[test]
    fn verdict_must_match_digest_set_comparison() {
        let mut receipt = sample_receipt();
        receipt.nix_artifact_digests[0].digest_blake3 = sample_digest(9);
        receipt.comparison_verdict = NixCrossBuilderWitnessVerdict::NixWitnessMatch;
        let err = canonical_nix_cross_builder_witness_receipt(receipt).unwrap_err();
        assert!(err.to_string().contains("comparison_verdict"));
    }

    #[test]
    fn missing_mantle_proof_digest_is_rejected() {
        let mut receipt = sample_receipt();
        receipt.mantle_deterministic_proof_receipt_digest_blake3 = String::new();
        let err = canonical_nix_cross_builder_witness_receipt(receipt).unwrap_err();
        assert!(err.to_string().contains("mantle_deterministic_proof_receipt_digest_blake3"));
    }

    #[test]
    fn receipt_digest_must_match_canonical_bytes() {
        let mut receipt = sample_receipt();
        receipt.receipt_blake3 = Some(sample_digest(8));
        let err = canonical_nix_cross_builder_witness_receipt(receipt).unwrap_err();
        assert!(err.to_string().contains("receipt_blake3"));
    }

    #[test]
    fn canonicalization_preserves_flag_order_and_sorts_artifact_sets() {
        let mut receipt = sample_receipt();
        receipt.build_policy.build_flags = vec!["z".to_string(), "a".to_string()];
        receipt.mantle_artifact_digests = vec![artifact("z", 6), artifact("a", 7)];
        receipt.nix_artifact_digests = vec![artifact("a", 7), artifact("z", 6)];
        receipt.comparison_verdict = NixCrossBuilderWitnessVerdict::NixWitnessMatch;
        let canonical = canonical_nix_cross_builder_witness_receipt(receipt).unwrap();
        assert_eq!(canonical.build_policy.build_flags, vec!["z", "a"]);
        assert_eq!(canonical.mantle_artifact_digests[0].name, "a");
        assert_eq!(canonical.nix_artifact_digests[0].name, "a");
    }

    #[test]
    fn build_flag_order_changes_canonical_digest() {
        let mut first = sample_receipt();
        first.build_policy.build_flags = vec![
            "-Clink-arg=-Wl,--as-needed".to_string(),
            "-Clink-arg=-Wl,--no-as-needed".to_string(),
        ];
        let mut second = first.clone();
        second.build_policy.build_flags.reverse();
        let first_digest = nix_cross_builder_witness_receipt_digest_blake3(first).unwrap();
        let second_digest = nix_cross_builder_witness_receipt_digest_blake3(second).unwrap();
        assert_ne!(first_digest, second_digest);
    }

    #[test]
    fn mismatch_receipt_must_not_report_nix_witness_proof_class() {
        let mut receipt = sample_receipt();
        receipt.nix_artifact_digests[0].digest_blake3 = sample_digest(9);
        receipt.comparison_verdict = NixCrossBuilderWitnessVerdict::CrossBuilderMismatch;
        receipt.proof_class = Some(NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS.to_string());
        let err = canonical_nix_cross_builder_witness_receipt(receipt).unwrap_err();
        assert!(err.to_string().contains("proof_class"));
    }
}
