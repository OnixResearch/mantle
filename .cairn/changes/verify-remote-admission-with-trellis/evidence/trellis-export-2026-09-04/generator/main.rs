use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use kamacite_core::IdentityDomain;
use kamacite_core::TrellisProofEvidenceInput;
use kamacite_core::TrellisProofEvidenceRole;
use kamacite_core::build_trellis_proof_evidence_envelope;
use kamacite_core::compute_identity;
use kamacite_core::trellis_proof_assumptions_root;
use kamacite_core::trellis_proof_dependency_root;
use kamacite_core::trellis_proof_evidence_json_projection;
use kamacite_core::trellis_proof_profile_canonical_bytes;
use kamacite_core::validate_trellis_proof_evidence_input;
use kamacite_core::validate_trellis_proof_evidence_json_projection;
use serde_json::Value;
use valence_core::evidence_chain::EvidenceChainDigest;
use valence_core::evidence_ir::EvidenceSpan;
use valence_core::evidence_ir::VerificationRole;
use valence_core::trellis_proof_evidence::TRELLIS_PROOF_NON_CLAIM;
use valence_core::trellis_proof_evidence::TrellisProofEvidence;
use valence_core::trellis_proof_evidence::TrellisProofOutcome;
use valence_core::trellis_proof_evidence::validate_trellis_proof_evidence;

const KAMACITE_REVISION: &str = "de710a092d351e829abfb288d46124e2db8e5b7f";
const VALENCE_REVISION: &str = "27b8b2124e12b80718ded124274fec98bed7a581";
const TRELLIS_REVISION: &str = "8de4b24aa2d66cc2e6ec966d686df023492265d3";
const TRELLIS_SOURCE_BLAKE3: &str =
    "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c";
const RECEIPT_DOMAIN: &[u8] = b"valence.trellis-proof-evidence.receipt.v1\0";
const PROOFS_SOURCE_BYTES: u64 = 5_923;
const PROOFS_SOURCE_LINES: u32 = 189;

fn main() {
    let paths = input_paths();
    fs::create_dir_all(&paths.output).expect("create output directory");
    let source_archive = read(&paths.source_archive);
    let proof_ir = read(&paths.proof_ir);
    let verifier_receipt = read(&paths.verifier_receipt);
    let oracle = read(&paths.oracle);
    let policy = read(&paths.policy);
    assert_eq!(blake3_hex(&source_archive), TRELLIS_SOURCE_BLAKE3);

    let assumptions = assumptions();
    let dependencies = vec![
        compute_identity(IdentityDomain::EvidenceChain, &oracle),
        compute_identity(IdentityDomain::EvidenceChain, &policy),
    ];
    let input = kamacite_input(
        &source_archive,
        &proof_ir,
        &verifier_receipt,
        assumptions.clone(),
        dependencies,
    );
    assert!(validate_trellis_proof_evidence_input(&input).is_empty());
    assert_kamacite_negative_paths(&input);

    let envelope = build_trellis_proof_evidence_envelope(
        &input,
        compute_identity(IdentityDomain::Receipt, &policy),
        compute_identity(IdentityDomain::Receipt, &oracle),
    )
    .expect("build Kamacite Trellis proof envelope");
    let projection = trellis_proof_evidence_json_projection(&envelope);
    assert!(validate_trellis_proof_evidence_json_projection(&projection).is_empty());
    let canonical = trellis_proof_profile_canonical_bytes(&envelope.profile)
        .expect("encode canonical Kamacite profile");

    write(&paths.output.join("kamacite-envelope.preserves"), &canonical);
    write_json(&paths.output.join("kamacite-envelope.json"), &envelope);
    write_json(&paths.output.join("kamacite-envelope.compat.json"), &projection);

    let valence_input = valence_input(&proof_ir, &verifier_receipt, &assumptions);
    let valence_report = validate_trellis_proof_evidence(&valence_input);
    assert!(valence_report.valid);
    assert!(valence_report.issues.is_empty());
    assert_eq!(valence_report.outcome, TrellisProofOutcome::AcceptedFormalProof);
    assert!(!valence_report.graph_rows.is_empty());
    assert_valence_negative_paths(&valence_input);
    let receipt_body = serde_json::json!({
        "schema": "trellis.proof-evidence",
        "valence_revision": VALENCE_REVISION,
        "input": valence_input,
        "report": valence_report
    });
    let receipt_hash_blake3 = domain_hash(RECEIPT_DOMAIN, &canonical_json(&receipt_body));
    let receipt = serde_json::json!({
        "schema": "valence-trellis-proof-acceptance-receipt-v1",
        "receipt_hash_blake3": receipt_hash_blake3,
        "body": receipt_body
    });
    write_json(&paths.output.join("valence-acceptance.json"), &receipt);

    let projection_bytes = pretty_json(&serde_json::to_value(&projection).expect("projection value"));
    let valence_bytes = pretty_json(&receipt);
    println!("kamacite_revision={KAMACITE_REVISION}");
    println!("valence_revision={VALENCE_REVISION}");
    println!("trellis_revision={TRELLIS_REVISION}");
    println!("kamacite_canonical_bytes={}", canonical.len());
    println!("kamacite_canonical_blake3={}", blake3_hex(&canonical));
    println!("kamacite_projection_blake3={}", blake3_hex(&projection_bytes));
    println!("valence_artifact_blake3={}", blake3_hex(&valence_bytes));
    println!("valence_receipt_hash_blake3={receipt_hash_blake3}");
}

struct Paths {
    output: PathBuf,
    source_archive: PathBuf,
    proof_ir: PathBuf,
    verifier_receipt: PathBuf,
    oracle: PathBuf,
    policy: PathBuf,
}

fn input_paths() -> Paths {
    let mut arguments = env::args_os().skip(1);
    let paths = Paths {
        output: PathBuf::from(arguments.next().expect("output directory")),
        source_archive: PathBuf::from(arguments.next().expect("source archive")),
        proof_ir: PathBuf::from(arguments.next().expect("proof IR")),
        verifier_receipt: PathBuf::from(arguments.next().expect("verifier receipt")),
        oracle: PathBuf::from(arguments.next().expect("oracle")),
        policy: PathBuf::from(arguments.next().expect("policy")),
    };
    assert!(arguments.next().is_none(), "generator accepts exactly six paths");
    paths
}

fn kamacite_input(
    source_archive: &[u8],
    proof_ir: &[u8],
    verifier_receipt: &[u8],
    assumptions: Vec<String>,
    dependencies: Vec<kamacite_core::Identity>,
) -> TrellisProofEvidenceInput {
    TrellisProofEvidenceInput {
        producer: "github.com/OnixResearch/trellis".to_string(),
        role: TrellisProofEvidenceRole::FormalProofCandidate,
        source_identity: compute_identity(IdentityDomain::Source, source_archive),
        proof_ir_identity: compute_identity(IdentityDomain::EvidenceChain, proof_ir),
        verifier_receipt_identity: compute_identity(IdentityDomain::Receipt, verifier_receipt),
        verifier_passed: true,
        declared_assumptions_root: Some(trellis_proof_assumptions_root(&assumptions)),
        declared_dependency_root: Some(trellis_proof_dependency_root(&dependencies)),
        assumptions,
        dependency_identities: dependencies,
        property_ids: property_ids(),
        requirement_ids: requirement_ids(),
        non_claims: kamacite_non_claims(),
        semantic_claims: vec![
            "named abstract fenced-attempt properties at the recorded Trellis revision".to_string(),
        ],
    }
}

fn valence_input(
    proof_ir: &[u8],
    verifier_receipt: &[u8],
    assumptions: &[String],
) -> TrellisProofEvidence {
    TrellisProofEvidence {
        source: digest("source-envelope", TRELLIS_SOURCE_BLAKE3),
        proof_ir: digest("proof-ir", &blake3_hex(proof_ir)),
        verifier_receipt: digest("verifier-receipt", &blake3_hex(verifier_receipt)),
        assumptions: assumptions
            .iter()
            .map(|value| digest("assumption", &blake3_hex(value.as_bytes())))
            .collect(),
        property_ids: property_ids(),
        requirement_ids: requirement_ids(),
        spans: vec![EvidenceSpan {
            source_id: format!("trellis:{TRELLIS_REVISION}:{TRELLIS_SOURCE_BLAKE3}"),
            source_path: "src/fenced_attempt/proofs.rs".to_string(),
            start_byte: 0,
            end_byte: PROOFS_SOURCE_BYTES,
            start_line: 1,
            start_column: 1,
            end_line: PROOFS_SOURCE_LINES,
            end_column: 12,
            label: Some("fenced-attempt direct proof functions".to_string()),
        }],
        verifier_status: "passed".to_string(),
        policy_accepted: true,
        outcome: TrellisProofOutcome::AcceptedFormalProof,
        verification_role: VerificationRole::Property,
        non_claims: vec![TRELLIS_PROOF_NON_CLAIM.to_string()],
    }
}

fn assert_kamacite_negative_paths(input: &TrellisProofEvidenceInput) {
    let mut missing_assumption = input.clone();
    missing_assumption.assumptions.clear();
    assert!(!validate_trellis_proof_evidence_input(&missing_assumption).is_empty());

    let mut stale_source = input.clone();
    stale_source.source_identity = compute_identity(IdentityDomain::Receipt, b"wrong-domain");
    assert!(!validate_trellis_proof_evidence_input(&stale_source).is_empty());
}

fn assert_valence_negative_paths(input: &TrellisProofEvidence) {
    let mut wrong_role = input.clone();
    wrong_role.verification_role = VerificationRole::RecordedOnly;
    assert!(!validate_trellis_proof_evidence(&wrong_role).valid);

    let mut weakened = input.clone();
    weakened.non_claims.clear();
    assert!(!validate_trellis_proof_evidence(&weakened).valid);
}

fn assumptions() -> Vec<String> {
    vec![
        "Consumers admit raw identities and digests before they project equality and ordering facts.".to_string(),
        "The Verus verifier, Z3 solver, Rust compiler, vstd library, and selected Nix derivations are trusted verification dependencies.".to_string(),
        "The fenced-attempt module contains no assume_specification, assume_call, runtime_exempt, no_ensures, or external_body marker.".to_string(),
    ]
}

fn property_ids() -> Vec<String> {
    [
        "current_epoch_required",
        "reassignment_advances_epoch",
        "duplicate_event_is_noop",
        "event_conflict_rejected",
        "terminal_phase_closed",
        "completion_requires_linkage",
        "rejection_preserves_state",
        "exec_refines_spec",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn requirement_ids() -> Vec<String> {
    [
        "r[fenced-attempt-admission.boundary]",
        "r[fenced-attempt-admission.refinement]",
        "r[fenced-attempt-admission.fence-safety]",
        "r[fenced-attempt-admission.event-idempotence]",
        "r[fenced-attempt-admission.terminal-closure]",
        "r[fenced-attempt-admission.completion-linkage]",
        "r[fenced-attempt-admission.rejection-preservation]",
        "r[fenced-attempt-admission.evidence]",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn kamacite_non_claims() -> Vec<String> {
    [
        "not release eligibility",
        "not verifier soundness",
        "not downstream correctness",
        "not full implementation equivalence",
        "not persistence atomicity",
        "not transport reliability",
        "not cryptographic correctness",
        "not remote worker correctness",
        "not liveness or availability",
        "not whole-build correctness",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn digest(domain: &str, value: &str) -> EvidenceChainDigest {
    EvidenceChainDigest {
        domain: domain.to_string(),
        digest: value.to_string(),
    }
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) {
    write(path, &pretty_json(&serde_json::to_value(value).expect("JSON value")));
}

fn pretty_json(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize JSON");
    bytes.push(b'\n');
    bytes
}

fn canonical_json(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("serialize canonical JSON")
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}
