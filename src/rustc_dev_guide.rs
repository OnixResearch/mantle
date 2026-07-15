use std::collections::BTreeSet;

const MAX_GUIDE_REFERENCES: usize = 64;
const MAX_EVIDENCE_ROWS: usize = 128;
const BLAKE3_HEX_LENGTH: usize = 64;
const COMPILER_NON_CLAIM_FRAGMENT: &str = "not compiler correctness";
const PROGRAM_NON_CLAIM_FRAGMENT: &str = "not program correctness";

const MOVING_REVISIONS: &[&str] = &["main", "master", "HEAD"];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves compiler correctness",
    "proves program correctness",
    "proves full cargo compatibility",
    "proves release reproducibility",
    "proves bootstrap correctness",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustcGuideReference {
    pub id: String,
    pub section: String,
    pub pinned_revision: String,
    pub content_digest_blake3: Option<String>,
    pub consuming_surface: String,
    pub bounded_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustcBackendInvocationEvidence {
    pub crate_type: String,
    pub target_triple: String,
    pub codegen_backend: String,
    pub linker_args: Vec<String>,
    pub metadata_hash: String,
    pub externs: Vec<String>,
    pub cfgs: Vec<String>,
    pub path_remaps: Vec<String>,
    pub toolchain_identity: String,
    pub output_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustPlanningEvidence {
    pub claim_id: String,
    pub required_reference_ids: Vec<String>,
    pub invocation: RustcBackendInvocationEvidence,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompilerPolicyAdapterReceipt {
    pub adapter_identity: String,
    pub selected_rustc_identity: String,
    pub policy_digest_blake3: String,
    pub guide_reference_ids: Vec<String>,
    pub invocation_status: String,
    pub waiver_summary: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustProviderPatchPlan {
    pub operation: String,
    pub source_anchor: String,
    pub stage: String,
    pub guide_reference_id: String,
    pub input_digest_blake3: String,
    pub output_digest_blake3: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustcDevGuideValidationInput {
    pub references: Vec<RustcGuideReference>,
    pub planning_evidence: Vec<RustPlanningEvidence>,
    pub compiler_policy_receipts: Vec<CompilerPolicyAdapterReceipt>,
    pub provider_patch_plans: Vec<RustProviderPatchPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustcDevGuideValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

// r[impl rust_package_planning.rustc_dev_guide.reference_map]
// r[impl rust_package_planning.rustc_dev_guide.backend_invocation]
// r[impl rust_package_planning.rustc_dev_guide.compiler_policy_adapter]
// r[impl rust_package_planning.rustc_dev_guide.source_provider_patch_plan]
pub(crate) fn validate_rustc_dev_guide_boundaries(
    input: &RustcDevGuideValidationInput,
) -> RustcDevGuideValidationReport {
    debug_assert!(MAX_GUIDE_REFERENCES > 0);
    debug_assert!(MAX_EVIDENCE_ROWS > 0);
    let mut diagnostics = Vec::new();
    validate_count(
        CountBound {
            field_name: "references",
            actual_count: input.references.len(),
            maximum_count: MAX_GUIDE_REFERENCES,
        },
        &mut diagnostics,
    );
    for (field_name, actual_count) in [
        ("planning_evidence", input.planning_evidence.len()),
        ("compiler_policy_receipts", input.compiler_policy_receipts.len()),
        ("provider_patch_plans", input.provider_patch_plans.len()),
    ] {
        validate_count(
            CountBound {
                field_name,
                actual_count,
                maximum_count: MAX_EVIDENCE_ROWS,
            },
            &mut diagnostics,
        );
    }
    let reference_ids = validate_references(&input.references, &mut diagnostics);
    validate_planning_evidence(&input.planning_evidence, &reference_ids, &mut diagnostics);
    validate_compiler_policy_receipts(&input.compiler_policy_receipts, &reference_ids, &mut diagnostics);
    validate_provider_patch_plans(&input.provider_patch_plans, &reference_ids, &mut diagnostics);
    RustcDevGuideValidationReport {
        valid: diagnostics.is_empty(),
        diagnostics,
    }
}

struct CountBound<'a> {
    field_name: &'a str,
    actual_count: usize,
    maximum_count: usize,
}

fn validate_count(bound: CountBound<'_>, diagnostics: &mut Vec<String>) {
    if bound.actual_count > bound.maximum_count {
        diagnostics.push(format!("rustc-dev-guide {} exceeds maximum count {}", bound.field_name, bound.maximum_count));
    }
}

fn validate_references(references: &[RustcGuideReference], diagnostics: &mut Vec<String>) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for reference in references {
        push_nonempty(&reference.id, "reference.id", diagnostics);
        push_nonempty(&reference.section, "reference.section", diagnostics);
        push_nonempty(&reference.consuming_surface, "reference.consuming_surface", diagnostics);
        push_nonempty(&reference.bounded_claim, "reference.bounded_claim", diagnostics);
        validate_pinned_revision(&reference.pinned_revision, diagnostics);
        validate_optional_digest(&reference.content_digest_blake3, "reference.content_digest_blake3", diagnostics);
        push_no_overclaim(&reference.bounded_claim, "reference.bounded_claim", diagnostics);
        if !ids.insert(reference.id.clone()) {
            diagnostics.push(format!("duplicate rustc-dev-guide reference id: {}", reference.id));
        }
    }
    ids
}

fn validate_pinned_revision(revision: &str, diagnostics: &mut Vec<String>) {
    push_nonempty(revision, "reference.pinned_revision", diagnostics);
    if MOVING_REVISIONS.contains(&revision) {
        diagnostics.push(format!("reference.pinned_revision uses moving revision: {revision}"));
    }
}

fn validate_optional_digest(value: &Option<String>, field_name: &str, diagnostics: &mut Vec<String>) {
    if let Some(digest) = value {
        validate_digest(digest, field_name, diagnostics);
    }
}

fn validate_planning_evidence(
    rows: &[RustPlanningEvidence],
    reference_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    for row in rows {
        push_nonempty(&row.claim_id, "planning.claim_id", diagnostics);
        validate_reference_links(
            &row.required_reference_ids,
            reference_ids,
            "planning.required_reference_ids",
            diagnostics,
        );
        validate_invocation(&row.invocation, diagnostics);
        validate_non_claims(&row.non_claims, "planning.non_claims", diagnostics);
    }
}

fn validate_invocation(invocation: &RustcBackendInvocationEvidence, diagnostics: &mut Vec<String>) {
    push_nonempty(&invocation.crate_type, "invocation.crate_type", diagnostics);
    push_nonempty(&invocation.target_triple, "invocation.target_triple", diagnostics);
    push_nonempty(&invocation.codegen_backend, "invocation.codegen_backend", diagnostics);
    push_nonempty(&invocation.metadata_hash, "invocation.metadata_hash", diagnostics);
    push_nonempty(&invocation.toolchain_identity, "invocation.toolchain_identity", diagnostics);
    validate_digest(&invocation.output_digest_blake3, "invocation.output_digest_blake3", diagnostics);
    if invocation.externs.is_empty() {
        diagnostics.push("invocation.externs must name dependency artifacts".to_string());
    }
    if invocation.cfgs.is_empty() {
        diagnostics.push("invocation.cfgs must name selected cfgs".to_string());
    }
    if invocation.path_remaps.is_empty() {
        diagnostics.push("invocation.path_remaps must name path-remap policy".to_string());
    }
}

fn validate_compiler_policy_receipts(
    receipts: &[CompilerPolicyAdapterReceipt],
    reference_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    for receipt in receipts {
        push_nonempty(&receipt.adapter_identity, "compiler_policy.adapter_identity", diagnostics);
        push_nonempty(&receipt.selected_rustc_identity, "compiler_policy.selected_rustc_identity", diagnostics);
        validate_digest(&receipt.policy_digest_blake3, "compiler_policy.policy_digest_blake3", diagnostics);
        validate_reference_links(
            &receipt.guide_reference_ids,
            reference_ids,
            "compiler_policy.guide_reference_ids",
            diagnostics,
        );
        push_nonempty(&receipt.invocation_status, "compiler_policy.invocation_status", diagnostics);
        push_no_overclaim(&receipt.waiver_summary, "compiler_policy.waiver_summary", diagnostics);
        validate_non_claims(&receipt.non_claims, "compiler_policy.non_claims", diagnostics);
    }
}

fn validate_provider_patch_plans(
    plans: &[RustProviderPatchPlan],
    reference_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    for plan in plans {
        push_nonempty(&plan.operation, "patch_plan.operation", diagnostics);
        push_nonempty(&plan.source_anchor, "patch_plan.source_anchor", diagnostics);
        push_nonempty(&plan.stage, "patch_plan.stage", diagnostics);
        validate_reference_links(
            core::slice::from_ref(&plan.guide_reference_id),
            reference_ids,
            "patch_plan.guide_reference_id",
            diagnostics,
        );
        validate_digest(&plan.input_digest_blake3, "patch_plan.input_digest_blake3", diagnostics);
        validate_digest(&plan.output_digest_blake3, "patch_plan.output_digest_blake3", diagnostics);
        validate_non_claims(&plan.non_claims, "patch_plan.non_claims", diagnostics);
    }
}

fn validate_reference_links(
    links: &[String],
    reference_ids: &BTreeSet<String>,
    field_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if links.is_empty() {
        diagnostics.push(format!("{field_name} must not be empty"));
    }
    for link in links {
        if !reference_ids.contains(link) {
            diagnostics.push(format!("{field_name} names missing guide reference: {link}"));
        }
    }
}

fn validate_non_claims(non_claims: &[String], field_name: &str, diagnostics: &mut Vec<String>) {
    let has_compiler = non_claims.iter().any(|value| value.contains(COMPILER_NON_CLAIM_FRAGMENT));
    let has_program = non_claims.iter().any(|value| value.contains(PROGRAM_NON_CLAIM_FRAGMENT));
    if !has_compiler || !has_program {
        diagnostics.push(format!("{field_name} missing compiler/program correctness non-claims"));
    }
    for non_claim in non_claims {
        push_no_overclaim(non_claim, field_name, diagnostics);
    }
}

fn validate_digest(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    if value.len() != BLAKE3_HEX_LENGTH || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        diagnostics.push(format!("{} is not BLAKE3 hex", field_name.as_ref()));
    }
}

fn push_nonempty(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{} must not be empty", field_name.as_ref()));
    }
}

fn push_no_overclaim(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    let lower = value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            diagnostics.push(format!("{} contains overclaim fragment {fragment:?}", field_name.as_ref()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_pinned_rustc_dev_guide_fixture() {
        let input = valid_input();

        let report = validate_rustc_dev_guide_boundaries(&input);

        assert!(report.valid);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn rejects_rustc_dev_guide_boundary_fixture_matrix() {
        let cases: [(&str, fn(&mut RustcDevGuideValidationInput), &str); 4] = [
            ("moving main", use_moving_main_reference, "moving revision"),
            ("missing guide reference", drop_planning_reference, "missing guide reference"),
            ("semantic overclaim", add_semantic_overclaim, "overclaim"),
            ("unstated backend", remove_backend_assumption, "invocation.codegen_backend"),
        ];

        for (name, mutate, expected) in cases {
            let mut input = valid_input();
            mutate(&mut input);

            let report = validate_rustc_dev_guide_boundaries(&input);

            assert!(!report.valid, "{name} should be rejected");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should mention {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    fn valid_input() -> RustcDevGuideValidationInput {
        let digest = "a".repeat(BLAKE3_HEX_LENGTH);
        RustcDevGuideValidationInput {
            references: vec![RustcGuideReference {
                id: "rustc-dev-guide.backend.codegen".to_string(),
                section: "backend".to_string(),
                pinned_revision: "9f4e3d2c1b0a".to_string(),
                content_digest_blake3: Some(digest.clone()),
                consuming_surface: "rust-plan backend invocation".to_string(),
                bounded_claim: "documents rustc backend invocation shape, not compiler correctness".to_string(),
            }],
            planning_evidence: vec![RustPlanningEvidence {
                claim_id: "rust-plan-backend-shape".to_string(),
                required_reference_ids: vec!["rustc-dev-guide.backend.codegen".to_string()],
                invocation: RustcBackendInvocationEvidence {
                    crate_type: "rlib".to_string(),
                    target_triple: "x86_64-unknown-linux-gnu".to_string(),
                    codegen_backend: "llvm".to_string(),
                    linker_args: vec!["-C link-self-contained=no".to_string()],
                    metadata_hash: "metadata-b3".to_string(),
                    externs: vec!["dep=/store/dep.rlib".to_string()],
                    cfgs: vec!["feature=std".to_string()],
                    path_remaps: vec!["/source=/mantle/source".to_string()],
                    toolchain_identity: "rustc-1.91.1".to_string(),
                    output_digest_blake3: digest.clone(),
                },
                non_claims: standard_non_claims(),
            }],
            compiler_policy_receipts: vec![CompilerPolicyAdapterReceipt {
                adapter_identity: "hir-policy-v1".to_string(),
                selected_rustc_identity: "rustc-1.91.1".to_string(),
                policy_digest_blake3: digest.clone(),
                guide_reference_ids: vec!["rustc-dev-guide.backend.codegen".to_string()],
                invocation_status: "passed".to_string(),
                waiver_summary: "no waivers".to_string(),
                non_claims: standard_non_claims(),
            }],
            provider_patch_plans: vec![RustProviderPatchPlan {
                operation: "sysroot rlib lookup patch".to_string(),
                source_anchor: "compiler/rustc_driver".to_string(),
                stage: "source-provider-stage1".to_string(),
                guide_reference_id: "rustc-dev-guide.backend.codegen".to_string(),
                input_digest_blake3: digest.clone(),
                output_digest_blake3: digest,
                non_claims: standard_non_claims(),
            }],
        }
    }

    fn standard_non_claims() -> Vec<String> {
        vec![
            "not compiler correctness".to_string(),
            "not program correctness".to_string(),
            "not full Cargo compatibility".to_string(),
        ]
    }

    fn use_moving_main_reference(input: &mut RustcDevGuideValidationInput) {
        input.references[0].pinned_revision = "main".to_string();
    }

    fn drop_planning_reference(input: &mut RustcDevGuideValidationInput) {
        input.planning_evidence[0].required_reference_ids = vec!["missing.reference".to_string()];
    }

    fn add_semantic_overclaim(input: &mut RustcDevGuideValidationInput) {
        input.compiler_policy_receipts[0].waiver_summary = "proves compiler correctness".to_string();
    }

    fn remove_backend_assumption(input: &mut RustcDevGuideValidationInput) {
        input.planning_evidence[0].invocation.codegen_backend.clear();
    }
}
