use crunch_attestation_core::ArtifactAttestation;
use crunch_attestation_core::AttestationDigest;
use crunch_attestation_core::BinaryDigest;
use crunch_attestation_core::BinaryDigestMatchInput;
use crunch_attestation_core::ClosureAttestation;
use crunch_attestation_core::DetachedSignature;
use crunch_attestation_core::Error;
use crunch_attestation_core::PolicyEvaluation;
use crunch_attestation_core::PolicyEvaluationInput;
use crunch_attestation_core::ProjectAttestation;
use crunch_attestation_core::ReleaseAttestation;
use crunch_attestation_core::ReleasePolicy;
use crunch_attestation_core::ReleaseRevocations;
use crunch_attestation_core::ValidatedWitness;
use crunch_attestation_core::WitnessAttestation;

pub trait Canonicalize {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error>;

    fn canonical_digest(&self) -> Result<AttestationDigest, Error> {
        let bytes = self.canonical_bytes()?;
        Ok(AttestationDigest::from_canonical_bytes(bytes))
    }
}

impl Canonicalize for ArtifactAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        crunch_attestation_core::artifact_attestation_canonical_bytes(self.clone())
    }
}

impl Canonicalize for ClosureAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        crunch_attestation_core::closure_attestation_canonical_bytes(self.clone())
    }
}

impl Canonicalize for ProjectAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        crunch_attestation_core::project_attestation_canonical_bytes(self.clone())
    }
}

impl Canonicalize for ReleaseAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        crunch_attestation_core::release_attestation_canonical_bytes(self.clone())
    }
}

impl Canonicalize for WitnessAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        crunch_attestation_core::witness_attestation_canonical_bytes(self.clone())
    }
}

pub fn binary_digests_match(published: &[BinaryDigest], rebuilt: &[BinaryDigest]) -> bool {
    crunch_attestation_core::binary_digests_match(BinaryDigestMatchInput {
        published: published.to_vec(),
        rebuilt: rebuilt.to_vec(),
    })
}

pub fn evaluate_policy(
    release: &ReleaseAttestation,
    witnesses: &[ValidatedWitness],
    policy: &ReleasePolicy,
    revocations: &ReleaseRevocations,
) -> Result<PolicyEvaluation, Error> {
    crunch_attestation_core::evaluate_policy(PolicyEvaluationInput {
        release: release.clone(),
        witnesses: witnesses.to_vec(),
        policy: policy.clone(),
        revocations: revocations.clone(),
    })
}

pub fn parse_detached_signature(line: &str) -> Result<DetachedSignature, Error> {
    crunch_attestation_core::parse_detached_signature(line.to_string())
}

pub fn encode_detached_signature(signature: &DetachedSignature) -> String {
    crunch_attestation_core::encode_detached_signature(signature.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::RebuildEnvironmentSummary;
    use crate::release::Workflow;

    #[test]
    fn canonicalize_trait_routes_release_bytes_through_core() {
        let release = sample_release();
        let bytes_from_trait = release.canonical_bytes().unwrap();
        let bytes_from_core = crunch_attestation_core::release_attestation_canonical_bytes(release).unwrap();
        assert_eq!(bytes_from_trait, bytes_from_core);
    }

    #[test]
    fn evaluate_policy_adapter_preserves_old_signature() {
        let release = sample_release();
        let policy = ReleasePolicy::new(0, "witness_identity".to_string(), Vec::new(), Vec::new());
        let result = evaluate_policy(&release, &[], &policy, &ReleaseRevocations::empty()).unwrap();
        assert_eq!(result.trust_tier.final_class, crunch_attestation_core::FinalClass::QuorumSatisfied);
    }

    #[test]
    fn detached_signature_adapter_preserves_borrowed_text_api() {
        let signature = DetachedSignature {
            key_name: "witness-a".to_string(),
            signature_bytes: [0xab; 64],
        };

        let encoded = encode_detached_signature(&signature);
        let parsed = parse_detached_signature(&encoded).unwrap();

        assert_eq!(parsed, signature);
    }

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(crunch_attestation_core::ReleaseAttestationInit {
            release_id: "mantle-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest".to_vec()),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof".to_vec()),
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: vec![BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            }],
        })
    }

    #[allow(dead_code)]
    fn _sample_env() -> RebuildEnvironmentSummary {
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: "nixos-25.05".to_string(),
        }
    }
}
