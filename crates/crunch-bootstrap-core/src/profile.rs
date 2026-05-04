use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StagexProfileStatus {
    Satisfied,
    Unsatisfied,
}

impl StagexProfileStatus {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }
}

impl core::fmt::Display for StagexProfileStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Satisfied => f.write_str("satisfied"),
            Self::Unsatisfied => f.write_str("unsatisfied"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StagexQuorumStatus {
    NotEvaluated,
}

impl core::fmt::Display for StagexQuorumStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotEvaluated => f.write_str("not_evaluated"),
        }
    }
}

pub const STAGEX_VERIFIED_NO_QUORUM_CLASS: &str = "stagex-verified-no-quorum";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexNoQuorumResult {
    pub status: StagexProfileStatus,
    pub class: String,
    pub quorum_status: StagexQuorumStatus,
    pub provider_kind: String,
    pub proof_bundle_digest: String,
    pub reproducibility_report_digest: String,
    pub release_id: String,
    pub artifact_set_digest: String,
    pub failure_reasons: Vec<String>,
}

impl StagexNoQuorumResult {
    pub fn satisfied(
        provider_kind: String,
        proof_bundle_digest: String,
        reproducibility_report_digest: String,
        release_id: String,
        artifact_set_digest: String,
    ) -> Self {
        debug_assert!(!provider_kind.is_empty());
        debug_assert!(!proof_bundle_digest.is_empty());
        debug_assert!(!reproducibility_report_digest.is_empty());
        debug_assert!(!release_id.is_empty());
        debug_assert!(!artifact_set_digest.is_empty());
        Self {
            status: StagexProfileStatus::Satisfied,
            class: String::from(STAGEX_VERIFIED_NO_QUORUM_CLASS),
            quorum_status: StagexQuorumStatus::NotEvaluated,
            provider_kind,
            proof_bundle_digest,
            reproducibility_report_digest,
            release_id,
            artifact_set_digest,
            failure_reasons: Vec::new(),
        }
    }

    pub fn unsatisfied(failure_reasons: Vec<String>) -> Self {
        debug_assert!(!failure_reasons.is_empty());
        Self {
            status: StagexProfileStatus::Unsatisfied,
            class: String::from(STAGEX_VERIFIED_NO_QUORUM_CLASS),
            quorum_status: StagexQuorumStatus::NotEvaluated,
            provider_kind: String::new(),
            proof_bundle_digest: String::new(),
            reproducibility_report_digest: String::new(),
            release_id: String::new(),
            artifact_set_digest: String::new(),
            failure_reasons,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexLineageProofBlock {
    pub seed_class: String,
    pub audit_seed_max_bytes: u32,
    pub seed_digest: String,
    pub lineage_manifest_digest: String,
    pub stage_graph_digest: String,
    pub normalized_provider_digest: String,
    pub staged_source_digest: String,
    pub stage1_crunch_digest: String,
    pub stage2_crunch_digest: String,
    pub bootstrap_tool_digests: Vec<BootstrapToolDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_exec_audit_digest: Option<String>,
    pub proof_bundle_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapToolDigest {
    pub name: String,
    pub digest: String,
}

pub const STAGEX_LINEAGE_PROVIDER_KIND: &str = "stagex-lineage";

pub fn evaluate_stagex_no_quorum(
    bundle_verified: bool,
    proof_block: Option<&StagexLineageProofBlock>,
    reproducibility_verified: bool,
    reproducibility_report_digest: &str,
    release_id: &str,
    artifact_set_digest: &str,
) -> StagexNoQuorumResult {
    let mut failures = Vec::new();

    if !bundle_verified {
        failures.push(String::from("release-evidence bundle integrity not verified"));
    }

    match proof_block {
        None => {
            failures.push(String::from("no StageX-class lineage proof present"));
        }
        Some(block) => {
            if block.seed_class.is_empty() {
                failures.push(String::from("proof block missing seed_class"));
            }
            if block.lineage_manifest_digest.is_empty() {
                failures.push(String::from("proof block missing lineage_manifest_digest"));
            }
            if block.stage1_crunch_digest.is_empty() || block.stage2_crunch_digest.is_empty() {
                failures.push(String::from("proof block missing stage1 or stage2 digest"));
            }
            if block.proof_bundle_digest.is_empty() {
                failures.push(String::from("proof block missing proof_bundle_digest"));
            }
        }
    }

    if !reproducibility_verified {
        failures.push(String::from("reproducibility report not verified or missing"));
    }

    if release_id.is_empty() {
        failures.push(String::from("release_id is empty"));
    }

    if artifact_set_digest.is_empty() {
        failures.push(String::from("artifact_set_digest is empty"));
    }

    if !failures.is_empty() {
        return StagexNoQuorumResult::unsatisfied(failures);
    }

    let block = proof_block.expect("proof block verified above");
    StagexNoQuorumResult::satisfied(
        String::from(STAGEX_LINEAGE_PROVIDER_KIND),
        block.proof_bundle_digest.clone(),
        reproducibility_report_digest.to_string(),
        release_id.to_string(),
        artifact_set_digest.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    fn sample_proof_block() -> StagexLineageProofBlock {
        StagexLineageProofBlock {
            seed_class: "hex0-seed".to_string(),
            audit_seed_max_bytes: 4096,
            seed_digest: "a".repeat(64),
            lineage_manifest_digest: "b".repeat(64),
            stage_graph_digest: "c".repeat(64),
            normalized_provider_digest: "d".repeat(64),
            staged_source_digest: "e".repeat(64),
            stage1_crunch_digest: "f".repeat(64),
            stage2_crunch_digest: "1".repeat(64),
            bootstrap_tool_digests: vec![BootstrapToolDigest {
                name: "bwrap".to_string(),
                digest: "2".repeat(64),
            }],
            protected_exec_audit_digest: Some("3".repeat(64)),
            proof_bundle_digest: "4".repeat(64),
        }
    }

    #[test]
    fn satisfied_result_has_correct_fields() {
        let block = sample_proof_block();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "release-1", &"6".repeat(64));

        assert!(result.status.is_satisfied());
        assert_eq!(result.class, STAGEX_VERIFIED_NO_QUORUM_CLASS);
        assert_eq!(result.quorum_status, StagexQuorumStatus::NotEvaluated);
        assert_eq!(result.provider_kind, STAGEX_LINEAGE_PROVIDER_KIND);
        assert!(!result.proof_bundle_digest.is_empty());
        assert!(!result.reproducibility_report_digest.is_empty());
        assert_eq!(result.release_id, "release-1");
        assert!(result.failure_reasons.is_empty());
    }

    #[test]
    fn satisfied_result_never_says_quorum_satisfied() {
        let block = sample_proof_block();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "release-1", &"6".repeat(64));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("quorum-satisfied"), "must not emit quorum-satisfied: {json}");
        assert!(!json.contains("quorum_satisfied"), "must not emit quorum_satisfied: {json}");
    }

    #[test]
    fn missing_bundle_verification_rejected() {
        let block = sample_proof_block();
        let result =
            evaluate_stagex_no_quorum(false, Some(&block), true, &"5".repeat(64), "release-1", &"6".repeat(64));
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("bundle integrity")));
    }

    #[test]
    fn missing_proof_block_rejected() {
        let result = evaluate_stagex_no_quorum(true, None, true, &"5".repeat(64), "release-1", &"6".repeat(64));
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")));
    }

    #[test]
    fn missing_reproducibility_rejected() {
        let block = sample_proof_block();
        let result =
            evaluate_stagex_no_quorum(true, Some(&block), false, &"5".repeat(64), "release-1", &"6".repeat(64));
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("reproducibility")));
    }

    #[test]
    fn empty_release_id_rejected() {
        let block = sample_proof_block();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "", &"6".repeat(64));
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("release_id")));
    }

    #[test]
    fn empty_artifact_set_digest_rejected() {
        let block = sample_proof_block();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "release-1", "");
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("artifact_set_digest")));
    }

    #[test]
    fn incomplete_proof_block_rejected() {
        let mut block = sample_proof_block();
        block.seed_class.clear();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "release-1", &"6".repeat(64));
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("seed_class")));
    }

    #[test]
    fn multiple_failures_all_reported() {
        let result = evaluate_stagex_no_quorum(false, None, false, "", "", "");
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.len() >= 4, "got {} failures", result.failure_reasons.len());
    }

    #[test]
    fn serde_roundtrip() {
        let block = sample_proof_block();
        let result = evaluate_stagex_no_quorum(true, Some(&block), true, &"5".repeat(64), "release-1", &"6".repeat(64));
        let json = serde_json::to_string(&result).unwrap();
        let parsed: StagexNoQuorumResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, parsed);
    }

    #[test]
    fn unsatisfied_result_binds_class() {
        let result = StagexNoQuorumResult::unsatisfied(vec!["test failure".to_string()]);
        assert_eq!(result.class, STAGEX_VERIFIED_NO_QUORUM_CLASS);
        assert_eq!(result.quorum_status, StagexQuorumStatus::NotEvaluated);
    }
}
