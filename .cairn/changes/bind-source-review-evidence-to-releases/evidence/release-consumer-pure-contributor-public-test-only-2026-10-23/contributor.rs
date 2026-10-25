// PUBLIC TEST ONLY. Exercises the exported pure contributor, not a release CLI.
// The deliberately invalid ordinary manifest context cannot assert release success.
use crunch_release_core::{BundledArtifact, BundledArtifactKind, ReleaseEvidenceManifest, ReleaseProofLinkage, ReleaseWorkflowIdentity};
use crunch_release_core::source_review::{verify_release_source_review, SourceReviewStatus, SourceReviewVerificationRequest};
use std::{env,fs,path::PathBuf};
fn main() {
    let args: Vec<String> = env::args().collect();
    let input = PathBuf::from(&args[1]);
    let archive_digest = args[2].clone();
    let archive_size: u64 = args[3].parse().unwrap();
    let unavailable = BundledArtifact { kind: BundledArtifactKind::Directory, relative_path: "UNAVAILABLE-GENUINE-SELF-HOSTING-PROOF".into(), size_bytes: 0, digest_blake3: String::new() };
    let manifest = ReleaseEvidenceManifest {
        schema: "PUBLIC-TEST-ONLY-CONTRIBUTOR-CONTEXT-NOT-A-RELEASE".into(),
        release_id: "PUBLIC-TEST-ONLY-NOT-A-RELEASE".into(),
        claim_scope: "CONTRIBUTOR-ONLY-NO-RELEASE-CLAIM".into(),
        workflow: ReleaseWorkflowIdentity { command: "PUBLIC-TEST-ONLY-pure-verifier".into(), version: "contributor-only".into() },
        source_archive: BundledArtifact { kind: BundledArtifactKind::File, relative_path: "PUBLIC-TEST-ONLY-actual-source.tar".into(), size_bytes: archive_size, digest_blake3: archive_digest },
        source_acquisition: None,
        binaries: Vec::new(), proof_bundle: unavailable.clone(), prerequisite_inventory: unavailable,
        provider_fixed_point_proof: None, reproducibility_report: None,
        deterministic_build_proof: None, deterministic_sandbox_isolation_evidence: None,
        independent_agreement_report: None, external_evidence: Vec::new(), kani_toolchain_evidence: Vec::new(),
        stack_provenance: None, opaque_evidence_sidecar_bindings: Vec::new(), cairn_handoff_validation: None,
        function_address_evidence: None,
        proof_linkage: ReleaseProofLinkage { release_id: "PUBLIC-TEST-ONLY-NOT-A-RELEASE".into(), source_archive_digest_blake3: String::new(), proof_bundle_schema: "UNAVAILABLE".into(), proof_mode: "UNAVAILABLE".into(), selected_provider_kind: "UNAVAILABLE".into(), staged_source: "UNAVAILABLE".into(), stage2_binary_digest_blake3: String::new(), prerequisite_inventory_digest_blake3: String::new(), proof_manifest_digest_blake3: String::new() },
        provenance_coverage: None, content_bound_requirement_evidence: None,
    };
    assert!(crunch_release_core::canonical_release_evidence_manifest(manifest.clone()).is_err(), "Context must not be an admissible release");
    let attachment = fs::read(input.join("attachment.json")).unwrap();
    let policy = fs::read(input.join("policy.json")).unwrap();
    let valence = fs::read(input.join("acceptance.json")).unwrap();
    let tampered = fs::read(input.join("tampered-attachment.json")).unwrap();
    let mut changed_policy = policy.clone(); changed_policy.push(b'\n');
    let cases = [
        ("selected-required-policy", Some(attachment.as_slice()), Some(policy.as_slice()), Some(valence.as_slice()), Some(policy.as_slice()), true, SourceReviewStatus::Satisfied, true, 2),
        ("unselected-recorded-only", Some(attachment.as_slice()), Some(policy.as_slice()), Some(valence.as_slice()), None, false, SourceReviewStatus::Recorded, true, 2),
        ("optional-absence", None, None, None, None, false, SourceReviewStatus::NotRequired, true, 0),
        ("required-missing-policy", Some(attachment.as_slice()), Some(policy.as_slice()), Some(valence.as_slice()), None, true, SourceReviewStatus::Missing, false, 0),
        ("selected-policy-byte-mismatch", Some(attachment.as_slice()), Some(policy.as_slice()), Some(valence.as_slice()), Some(changed_policy.as_slice()), true, SourceReviewStatus::Stale, false, 0),
        ("tampered-signature", Some(tampered.as_slice()), Some(policy.as_slice()), Some(valence.as_slice()), Some(policy.as_slice()), true, SourceReviewStatus::Invalid, false, 0),
    ];
    for (label,a,p,v,t,required,status,valid,count) in cases {
        let result = verify_release_source_review(&SourceReviewVerificationRequest { manifest: &manifest, attachment_bytes:a,policy_bytes:p,valence_bytes:v,trusted_policy_bytes:t,require_review:required });
        println!("{label}: {result:?}");
        assert_eq!(result.status,status,"{label}"); assert_eq!(result.valid,valid,"{label}"); assert_eq!(result.approved_key_blake3.len(),count,"{label}");
    }
    println!("PUBLIC TEST ONLY: six pure contributor cases observed; zero successful release creations, release signers or build witnesses; no genuine human review or release eligibility");
}
