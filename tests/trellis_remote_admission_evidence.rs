use crunch_release_core::TrellisRemoteAdmissionEvidenceReport;
use crunch_release_core::trellis_remote_admission_evidence_diagnostics;

const REPORT_JSON: &str = include_str!("../evidence/trellis/remote-admission-v1/report.json");
const MAX_DIAGNOSTICS: usize = 64;

#[test]
// r[verify remote_builds.trellis_admission_evidence_boundary]
fn checked_in_kamacite_and_valence_evidence_passes_the_typed_core() {
    let report: TrellisRemoteAdmissionEvidenceReport = serde_json::from_str(REPORT_JSON).unwrap();
    let diagnostics = trellis_remote_admission_evidence_diagnostics(&report);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(report.valence.valid);
    assert_eq!(report.valence.validation_role, "property");
}

#[test]
// r[verify remote_builds.trellis_admission_evidence_boundary]
// r[verify remote_builds.trellis_admission_claim_boundary]
fn proof_identity_assumption_role_and_claim_mutations_fail_closed() {
    let report: TrellisRemoteAdmissionEvidenceReport = serde_json::from_str(REPORT_JSON).unwrap();
    let mutations = [
        mutate_proof_identity(report.clone()),
        mutate_assumptions(report.clone()),
        mutate_valence_role(report.clone()),
        mutate_claim(report),
    ];

    for mutated in mutations {
        let diagnostics = trellis_remote_admission_evidence_diagnostics(&mutated);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.len() <= MAX_DIAGNOSTICS);
    }
}

fn mutate_proof_identity(mut report: TrellisRemoteAdmissionEvidenceReport) -> TrellisRemoteAdmissionEvidenceReport {
    report.proof.proof_ir_blake3 = "stale".to_string();
    report
}

fn mutate_assumptions(mut report: TrellisRemoteAdmissionEvidenceReport) -> TrellisRemoteAdmissionEvidenceReport {
    report.assumptions.clear();
    report
}

fn mutate_valence_role(mut report: TrellisRemoteAdmissionEvidenceReport) -> TrellisRemoteAdmissionEvidenceReport {
    report.valence.validation_role = "recorded_only".to_string();
    report
}

fn mutate_claim(mut report: TrellisRemoteAdmissionEvidenceReport) -> TrellisRemoteAdmissionEvidenceReport {
    report.claims = vec!["proves release eligibility".to_string()];
    report
}
