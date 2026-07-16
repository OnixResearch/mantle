use std::error::Error;
use std::fs;
use std::path::Path;

use crunch_attestation::AttestationDigest;
use crunch_attestation::BinaryDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::PolicyFailureReason;
use crunch_attestation::RebuildEnvironmentSummary;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::ReleaseAttestationInit;
use crunch_attestation::ReleasePolicy;
use crunch_attestation::ReleaseRevocations;
use crunch_attestation::ValidatedWitness;
use crunch_attestation::VerificationDirectory;
use crunch_attestation::WitnessAttestation;
use crunch_attestation::Workflow;
use nix_compat::narinfo::Signature;

const RELEASE_KEYPAIR: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const WITNESS_IDENTITY: &str = "witness-a";
const INDEPENDENCE_FIELD: &str = "witness_identity";
const REQUIRED_WITNESSES: u32 = 1;

fn binary_digests() -> Vec<BinaryDigest> {
    vec![BinaryDigest {
        name: "release-demo".to_string(),
        algorithm: "blake3".to_string(),
        digest: blake3::hash(b"release-demo-binary").to_hex().to_string(),
    }]
}

fn release_attestation() -> ReleaseAttestation {
    ReleaseAttestation::new(ReleaseAttestationInit {
        release_id: "release-witness-demo-v1".to_string(),
        release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest".to_vec()),
        proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof".to_vec()),
        proof_mode: "bounded-example".to_string(),
        declared_effect_claims: None,
        observed_effect_facts: None,
        workflow: Workflow {
            command: "mantle example release-witness-handoff".to_string(),
            version: "1".to_string(),
        },
        binary_digests: binary_digests(),
    })
}

fn witness_attestation(release_digest: AttestationDigest) -> WitnessAttestation {
    WitnessAttestation::new(release_digest, WITNESS_IDENTITY.to_string(), binary_digests(), RebuildEnvironmentSummary {
        system: "x86_64-linux".to_string(),
        toolchain: "independent-example-toolchain".to_string(),
        host_class: "example-witness-host".to_string(),
    })
    .with_source_acquisition_mode("public-handoff".to_string())
}

fn sign_bytes(keypair: &crunch_build::KeyPair, bytes: &[u8]) -> String {
    keypair.signing_key.sign(bytes).to_string()
}

fn signature_is_valid(keypair: &crunch_build::KeyPair, bytes: &[u8], text: &str) -> Result<bool, Box<dyn Error>> {
    let canonical_text = std::str::from_utf8(bytes)?;
    let signature = Signature::<&str>::parse(text)?;
    Ok(keypair.verifying_key.verify(canonical_text, &signature.as_ref()))
}

fn attestation_result<T>(result: Result<T, crunch_attestation::Error>) -> Result<T, Box<dyn Error>> {
    result.map_err(|error| std::io::Error::other(error.to_string()).into())
}

fn write_signed_document(path: &Path, bytes: &[u8], keypair: &crunch_build::KeyPair) -> Result<(), Box<dyn Error>> {
    let signature_path = format!("{}.sig", path.display());
    fs::write(path, bytes)?;
    fs::write(signature_path, format!("{}\n", sign_bytes(keypair, bytes)))?;
    Ok(())
}

fn write_policy(directory: &Path, release_key_name: &str) -> Result<ReleasePolicy, Box<dyn Error>> {
    let policy = ReleasePolicy::new(
        REQUIRED_WITNESSES,
        INDEPENDENCE_FIELD.to_string(),
        vec![release_key_name.to_string()],
        vec![WITNESS_IDENTITY.to_string()],
    );
    fs::write(directory.join("policy.json"), serde_json::to_vec(&policy)?)?;
    fs::write(directory.join("revocations.json"), serde_json::to_vec(&ReleaseRevocations::empty())?)?;
    Ok(policy)
}

struct HandoffDirectories {
    _scratch: tempfile::TempDir,
    publisher: std::path::PathBuf,
    returned: std::path::PathBuf,
    policy: ReleasePolicy,
}

fn prepare_handoff(
    release_bytes: &[u8],
    witness_bytes: &[u8],
    release_keypair: &crunch_build::KeyPair,
    witness_keypair: &crunch_build::KeyPair,
) -> Result<HandoffDirectories, Box<dyn Error>> {
    let scratch = tempfile::tempdir()?;
    let publisher = scratch.path().join("publisher");
    let returned = scratch.path().join("returned");
    fs::create_dir_all(publisher.join("witnesses"))?;
    fs::create_dir_all(returned.join("witnesses"))?;
    write_signed_document(&publisher.join("release-attestation.json"), release_bytes, release_keypair)?;
    let policy = write_policy(&publisher, release_keypair.verifying_key.name())?;
    write_signed_document(
        &returned.join("witnesses").join(format!("{WITNESS_IDENTITY}.json")),
        witness_bytes,
        witness_keypair,
    )?;
    Ok(HandoffDirectories {
        _scratch: scratch,
        publisher,
        returned,
        policy,
    })
}

fn import_witness(directories: &HandoffDirectories) -> Result<(), Box<dyn Error>> {
    let witness_file = format!("{WITNESS_IDENTITY}.json");
    fs::copy(
        directories.returned.join("witnesses").join(&witness_file),
        directories.publisher.join("witnesses").join(&witness_file),
    )?;
    fs::copy(
        directories.returned.join("witnesses").join(format!("{witness_file}.sig")),
        directories.publisher.join("witnesses").join(format!("{witness_file}.sig")),
    )?;
    Ok(())
}

fn verify_handoff(
    directories: &HandoffDirectories,
    release_bytes: &[u8],
    witness_bytes: &[u8],
    release_keypair: &crunch_build::KeyPair,
    witness_keypair: &crunch_build::KeyPair,
) -> Result<(), Box<dyn Error>> {
    let material = VerificationDirectory::new(directories.publisher.clone()).discover()?;
    let release_signature = fs::read_to_string(directories.publisher.join("release-attestation.json.sig"))?;
    let witness_signature =
        fs::read_to_string(directories.publisher.join("witnesses").join(format!("{WITNESS_IDENTITY}.json.sig")))?;
    assert!(signature_is_valid(release_keypair, release_bytes, release_signature.trim())?);
    assert!(signature_is_valid(witness_keypair, witness_bytes, witness_signature.trim())?);
    let unrelated_keypair = crunch_build::generate_keypair().0;
    assert!(!signature_is_valid(&unrelated_keypair, witness_bytes, witness_signature.trim())?);
    assert_eq!(material.witnesses.len(), 1);
    Ok(())
}

fn evaluate_witness_policy(
    release: &ReleaseAttestation,
    witness: WitnessAttestation,
    witness_digest: AttestationDigest,
    witness_key_name: &str,
    policy: &ReleasePolicy,
) -> Result<(), Box<dyn Error>> {
    let no_witness =
        attestation_result(crunch_attestation::evaluate_policy(release, &[], policy, &ReleaseRevocations::empty()))?;
    assert!(matches!(no_witness.policy_failure_reason, Some(PolicyFailureReason::InsufficientQuorum { .. })));
    let mut wrong_release_witness = witness.clone();
    wrong_release_witness.release_attestation_digest_blake3 =
        AttestationDigest::from_canonical_bytes(b"wrong-release".to_vec());
    let wrong_release = ValidatedWitness {
        attestation: wrong_release_witness,
        attestation_digest: witness_digest,
        signer_key_name: witness_key_name.to_string(),
    };
    let wrong_release_result = attestation_result(crunch_attestation::evaluate_policy(
        release,
        &[wrong_release],
        policy,
        &ReleaseRevocations::empty(),
    ))?;
    assert_eq!(wrong_release_result.matching_witness_count, 0);
    assert!(wrong_release_result.policy_failure_reason.is_some());
    let validated = ValidatedWitness {
        attestation: witness,
        attestation_digest: witness_digest,
        signer_key_name: witness_key_name.to_string(),
    };
    let accepted = attestation_result(crunch_attestation::evaluate_policy(
        release,
        std::slice::from_ref(&validated),
        policy,
        &ReleaseRevocations::empty(),
    ))?;
    assert_eq!(accepted.matching_witness_count, REQUIRED_WITNESSES);
    assert!(accepted.policy_failure_reason.is_none());
    let revocations = ReleaseRevocations::new(vec![validated.signer_key_name.clone()], vec![]);
    let rejected =
        attestation_result(crunch_attestation::evaluate_policy(release, &[validated], policy, &revocations))?;
    assert_eq!(rejected.revoked_witness_count, REQUIRED_WITNESSES);
    assert!(rejected.policy_failure_reason.is_some());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let release_keypair = crunch_build::load_keypair(RELEASE_KEYPAIR)?;
    let witness_keypair = crunch_build::generate_keypair().0;
    let release = release_attestation();
    let release_bytes = attestation_result(release.canonical_bytes())?;
    let witness = witness_attestation(attestation_result(release.canonical_digest())?);
    let witness_bytes = attestation_result(witness.canonical_bytes())?;
    let witness_digest = attestation_result(witness.canonical_digest())?;
    let directories = prepare_handoff(&release_bytes, &witness_bytes, &release_keypair, &witness_keypair)?;
    import_witness(&directories)?;
    verify_handoff(&directories, &release_bytes, &witness_bytes, &release_keypair, &witness_keypair)?;
    evaluate_witness_policy(
        &release,
        witness,
        witness_digest,
        witness_keypair.verifying_key.name(),
        &directories.policy,
    )?;
    println!("handoff: release and witness signatures valid");
    println!("policy: quorum satisfied with {REQUIRED_WITNESSES} independent witness");
    println!("negative: unknown key, wrong release, missing quorum, and revoked witness rejected");
    Ok(())
}
