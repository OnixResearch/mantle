use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactOutputClass;
use crate::Blake3Digest;
use crate::CandidatePackProjection;
use crate::CodegenPlan;
use crate::CompilationPlan;
use crate::CompilerAdmission;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::GeneratedProjectManifest;
use crate::OutputClassStatus;
use crate::OutputInspection;
use crate::OutputMember;
use crate::TargetAdmission;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const EXPERIMENT_RECEIPT_SCHEMA: &str = "mantle-kernelscript-experiment-receipt-v1";
const RECEIPT_NON_CLAIMS: &[&str] = &[
    "not-language-soundness",
    "not-bpf-verifier-acceptance",
    "not-kernel-safety",
    "not-runtime-correctness",
    "not-deployability",
    "not-production-readiness",
    "onix-semantics-external",
    "chaoscontrol-runtime-evidence-required",
];
const FORBIDDEN_RECEIPT_FRAGMENTS: &[&str] = &[
    "/home/",
    "/tmp/",
    "BEGIN PRIVATE KEY",
    "credential=",
    "password=",
    "secret=",
    "token=",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExperimentStageStatus {
    Blocked,
    Planned,
    CodegenClassified,
    PartiallyInspected,
    Inspected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputClassReceipt {
    pub output_class: ArtifactOutputClass,
    pub status: OutputClassStatus,
    pub members: Vec<OutputMember>,
    pub inspection_identities_blake3: Vec<Blake3Digest>,
    pub blocker_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentReceiptInput {
    pub profile: ExperimentProfile,
    pub stage_status: ExperimentStageStatus,
    pub compiler_admission: CompilerAdmission,
    pub codegen_plan: Option<CodegenPlan>,
    pub generated_manifest: Option<GeneratedProjectManifest>,
    pub target_admission: Option<TargetAdmission>,
    pub compilation_plan: Option<CompilationPlan>,
    pub output_classes: Vec<OutputClassReceipt>,
    pub output_inspections: Vec<OutputInspection>,
    pub candidate_packs: Vec<CandidatePackProjection>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentReceipt {
    pub schema: String,
    pub stage_status: ExperimentStageStatus,
    pub experiment_id: String,
    pub profile_identity_blake3: Blake3Digest,
    pub source_blake3: Blake3Digest,
    pub compiler_source_revision: String,
    pub compiler_source_archive_sha256: crate::Sha256Digest,
    pub compiler_source_archive_blake3: Blake3Digest,
    pub compiler_dependency_lock_blake3: Blake3Digest,
    pub expected_compiler_closure_blake3: Blake3Digest,
    pub admitted_compiler_materialization_blake3: Option<Blake3Digest>,
    pub codegen_command_identity_blake3: Option<Blake3Digest>,
    pub generated_manifest_identity_blake3: Option<Blake3Digest>,
    pub target_identity_blake3: Option<Blake3Digest>,
    pub compilation_plan_identity_blake3: Option<Blake3Digest>,
    pub output_classes: Vec<OutputClassReceipt>,
    pub output_inspections: Vec<OutputInspection>,
    pub candidate_packs: Vec<CandidatePackProjection>,
    pub blockers: Vec<ExperimentBlocker>,
    pub non_claims: Vec<String>,
    pub receipt_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentReceiptResult {
    pub receipt: Option<ExperimentReceipt>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Serialize)]
struct ReceiptIdentityInput {
    schema: String,
    stage_status: ExperimentStageStatus,
    experiment_id: String,
    profile_identity_blake3: Blake3Digest,
    source_blake3: Blake3Digest,
    compiler_source_revision: String,
    compiler_source_archive_sha256: crate::Sha256Digest,
    compiler_source_archive_blake3: Blake3Digest,
    compiler_dependency_lock_blake3: Blake3Digest,
    expected_compiler_closure_blake3: Blake3Digest,
    admitted_compiler_materialization_blake3: Option<Blake3Digest>,
    codegen_command_identity_blake3: Option<Blake3Digest>,
    generated_manifest_identity_blake3: Option<Blake3Digest>,
    target_identity_blake3: Option<Blake3Digest>,
    compilation_plan_identity_blake3: Option<Blake3Digest>,
    output_classes: Vec<OutputClassReceipt>,
    output_inspections: Vec<OutputInspection>,
    candidate_packs: Vec<CandidatePackProjection>,
    blockers: Vec<ExperimentBlocker>,
    non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ReceiptCollectionBounds {
    max_output_files: u32,
    max_receipt_blockers: u32,
}

struct CandidateReceiptValidation<'a> {
    input: &'a ExperimentReceiptInput,
    profile_identity: &'a Blake3Digest,
    candidate: &'a CandidatePackProjection,
    expected_members: &'a [OutputMember],
    expected_inspections: &'a [Blake3Digest],
    is_kind_unique: bool,
}

pub fn build_experiment_receipt(mut input: ExperimentReceiptInput) -> ExperimentReceiptResult {
    let validation = crate::validate_profile(input.profile.clone());
    let mut receipt_blockers = validation.blockers;
    let Some(profile_identity) = validation.profile_identity_blake3 else {
        return ExperimentReceiptResult {
            receipt: None,
            blockers: receipt_blockers,
        };
    };
    validate_receipt_bounds(&input, &mut receipt_blockers);
    if !receipt_blockers.is_empty() {
        return ExperimentReceiptResult {
            receipt: None,
            blockers: receipt_blockers,
        };
    }
    validate_receipt_relations(&input, &profile_identity, &mut receipt_blockers);
    validate_no_leaks(&input, &mut receipt_blockers);
    if !receipt_blockers.is_empty() {
        return ExperimentReceiptResult {
            receipt: None,
            blockers: receipt_blockers,
        };
    }
    canonicalize_input(&mut input);
    let receipt = identify_receipt(input, profile_identity, &mut receipt_blockers);
    debug_assert!(receipt_blockers.is_empty() == receipt.is_some());
    debug_assert!(receipt_blockers.iter().all(|item| !item.code.is_empty()));
    ExperimentReceiptResult {
        receipt,
        blockers: receipt_blockers,
    }
}

fn validate_receipt_bounds(input: &ExperimentReceiptInput, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    let bounds = &input.profile.bounds;
    if !receipt_collections_are_bounded(input) {
        blockers.push(blocker("receipt-collection-limit", "receipt", "receipt collections exceed the profile bounds"));
    }
    for class in &input.output_classes {
        if !class_receipt_collections_are_bounded(class, ReceiptCollectionBounds {
            max_output_files: bounds.max_output_files,
            max_receipt_blockers: bounds.max_receipt_blockers,
        }) {
            blockers.push(blocker(
                "output-class-receipt-limit",
                "receipt.output_classes",
                "output class receipt exceeds fixed bounds",
            ));
        }
    }
    debug_assert!(bounds.max_output_files > 0);
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn receipt_collections_are_bounded(input: &ExperimentReceiptInput) -> bool {
    let bounds = &input.profile.bounds;
    let is_blocker_count_bounded = !count_above_bound(input.blockers.len(), bounds.max_receipt_blockers);
    let is_class_count_bounded = !count_above_bound(input.output_classes.len(), bounds.max_output_files);
    let is_inspection_count_bounded = !count_above_bound(input.output_inspections.len(), bounds.max_output_files);
    let is_candidate_count_bounded = !count_above_bound(input.candidate_packs.len(), bounds.max_output_files);
    is_blocker_count_bounded && is_class_count_bounded && is_inspection_count_bounded && is_candidate_count_bounded
}

fn class_receipt_collections_are_bounded(class: &OutputClassReceipt, bounds: ReceiptCollectionBounds) -> bool {
    let is_member_count_bounded = !count_above_bound(class.members.len(), bounds.max_output_files);
    let is_inspection_count_bounded =
        !count_above_bound(class.inspection_identities_blake3.len(), bounds.max_output_files);
    let is_blocker_count_bounded = !count_above_bound(class.blocker_codes.len(), bounds.max_receipt_blockers);
    is_member_count_bounded && is_inspection_count_bounded && is_blocker_count_bounded
}

fn validate_receipt_relations(
    input: &ExperimentReceiptInput,
    profile_identity: &Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    validate_optional_plan_relations(input, profile_identity, blockers);
    validate_output_relations(input, profile_identity, blockers);
    validate_stage_status(input, blockers);
    let is_compiler_clean = input.compiler_admission.admitted
        && input.compiler_admission.materialization_identity_blake3.is_some()
        && input.compiler_admission.blockers.is_empty();
    let is_compiler_blocked = !input.compiler_admission.admitted
        && input.compiler_admission.materialization_identity_blake3.is_none()
        && !input.compiler_admission.blockers.is_empty();
    if !is_compiler_clean && !is_compiler_blocked {
        blockers.push(blocker(
            "compiler-admission-inconsistent",
            "receipt.compiler",
            "compiler admission status, identity, and blockers disagree",
        ));
    }
    if !input.compiler_admission.admitted && input.stage_status != ExperimentStageStatus::Blocked {
        blockers.push(blocker(
            "compiler-admission-overclaim",
            "receipt.compiler",
            "non-admitted compiler materialization requires a blocked receipt",
        ));
    }
    debug_assert!(!profile_identity.as_str().is_empty());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn validate_optional_plan_relations(
    input: &ExperimentReceiptInput,
    profile_identity: &Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    if let Some(codegen) = &input.codegen_plan {
        let is_codegen_valid = &codegen.profile_identity_blake3 == profile_identity
            && crate::planner::expected_codegen_plan_identity(codegen).as_ref() == Some(&codegen.plan_identity_blake3);
        if !is_codegen_valid {
            blockers.push(blocker(
                "receipt-codegen-profile-mismatch",
                "receipt.codegen",
                "codegen plan identity or profile binding differs",
            ));
        }
    }
    if let Some(manifest) = &input.generated_manifest {
        let is_manifest_valid = &manifest.profile_identity_blake3 == profile_identity
            && crate::generated::expected_generated_manifest_identity(manifest).as_ref()
                == Some(&manifest.manifest_identity_blake3);
        if !is_manifest_valid {
            blockers.push(blocker(
                "receipt-generated-profile-mismatch",
                "receipt.generated",
                "generated manifest identity or profile binding differs",
            ));
        }
    }
    if let Some(plan) = &input.compilation_plan {
        let has_matching_manifest = input.generated_manifest.as_ref().is_some_and(|manifest| {
            plan.generated_manifest_identity_blake3 == manifest.manifest_identity_blake3
                && plan.generated_members == manifest.members
        });
        let is_plan_valid = &plan.profile_identity_blake3 == profile_identity
            && crate::planner::expected_compilation_plan_identity(plan).as_ref() == Some(&plan.plan_identity_blake3)
            && has_matching_manifest;
        if !is_plan_valid {
            blockers.push(blocker(
                "receipt-compilation-profile-mismatch",
                "receipt.compilation",
                "compilation plan identity, generated manifest, or profile binding differs",
            ));
        }
    }
    debug_assert!(!profile_identity.as_str().is_empty());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn validate_output_relations(
    input: &ExperimentReceiptInput,
    profile_identity: &Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let mut classes = BTreeSet::new();
    for class in &input.output_classes {
        if !classes.insert(class.output_class) {
            blockers.push(blocker(
                "duplicate-output-class-receipt",
                "receipt.output_classes",
                "output class receipt is duplicated",
            ));
        }
        validate_class_receipt(input, class, blockers);
    }
    validate_inspection_receipts(input, &classes, blockers);
    validate_candidate_receipts(input, profile_identity, blockers);
}

fn validate_class_receipt(
    input: &ExperimentReceiptInput,
    class: &OutputClassReceipt,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    let mut expected_members = input
        .output_inspections
        .iter()
        .filter(|inspection| inspection.output_class == class.output_class)
        .map(inspection_output_member)
        .collect::<Vec<_>>();
    let mut expected_identities = input
        .output_inspections
        .iter()
        .filter(|inspection| inspection.output_class == class.output_class)
        .map(|inspection| inspection.inspection_identity_blake3.clone())
        .collect::<Vec<_>>();
    let mut actual_members = class.members.clone();
    let mut actual_identities = class.inspection_identities_blake3.clone();
    sort_output_members(&mut expected_members);
    sort_output_members(&mut actual_members);
    expected_identities.sort();
    actual_identities.sort();
    let is_passed_shape = class.status == OutputClassStatus::InspectionPassed
        && !expected_members.is_empty()
        && class.blocker_codes.is_empty()
        && actual_members == expected_members
        && actual_identities == expected_identities;
    let is_non_passed_shape = class.status != OutputClassStatus::InspectionPassed && expected_members.is_empty();
    if !is_passed_shape && !is_non_passed_shape {
        blockers.push(blocker(
            "output-class-receipt-mismatch",
            "receipt.output_classes",
            "output class status, members, and inspection identities disagree",
        ));
    }
    debug_assert_eq!(expected_members.len(), expected_identities.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn validate_inspection_receipts(
    input: &ExperimentReceiptInput,
    receipt_classes: &BTreeSet<ArtifactOutputClass>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    for inspection in &input.output_inspections {
        if !inspection_receipt_is_valid(input, receipt_classes, inspection) {
            blockers.push(blocker(
                "receipt-inspection-mismatch",
                &inspection.relative_path,
                "inspection lacks admission, identity integrity, exact plan linkage, or a class receipt",
            ));
        }
    }
}

fn inspection_receipt_is_valid(
    input: &ExperimentReceiptInput,
    receipt_classes: &BTreeSet<ArtifactOutputClass>,
    inspection: &OutputInspection,
) -> bool {
    let has_matching_plan = input
        .compilation_plan
        .as_ref()
        .is_some_and(|plan| inspection.plan_identity_blake3 == plan.plan_identity_blake3);
    let has_matching_identity = crate::inspection::expected_inspection_identity(inspection).as_ref()
        == Some(&inspection.inspection_identity_blake3);
    let has_class_receipt = receipt_classes.contains(&inspection.output_class);
    inspection.accepted && has_matching_plan && has_matching_identity && has_class_receipt
}

fn validate_candidate_receipts(
    input: &ExperimentReceiptInput,
    profile_identity: &Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    let mut kinds = BTreeSet::new();
    for candidate in &input.candidate_packs {
        let class = match candidate.kind {
            crate::CandidatePackKind::ModulePack => ArtifactOutputClass::KernelModule,
            crate::CandidatePackKind::BpfPack => ArtifactOutputClass::EbpfObject,
        };
        let expected_members = candidate_members(input, class);
        let expected_inspections = candidate_inspection_identities(input, class);
        let is_kind_unique = kinds.insert(candidate.kind);
        let is_candidate_valid = candidate_receipt_is_valid(CandidateReceiptValidation {
            input,
            profile_identity,
            candidate,
            expected_members: &expected_members,
            expected_inspections: &expected_inspections,
            is_kind_unique,
        });
        if !is_candidate_valid {
            blockers.push(blocker(
                "receipt-candidate-mismatch",
                "receipt.candidates",
                "candidate projection differs from exact inspections, profile, target, plan, or identity",
            ));
        }
    }
    debug_assert!(kinds.len() <= input.candidate_packs.len());
    debug_assert!(blockers.len() >= initial_blocker_count);
}

fn candidate_receipt_is_valid(validation: CandidateReceiptValidation<'_>) -> bool {
    debug_assert!(!validation.profile_identity.as_str().is_empty());
    debug_assert!(validation.expected_members.len() <= validation.input.output_inspections.len());
    let has_matching_identity = crate::handoff::expected_candidate_identity(validation.candidate).as_ref()
        == Some(&validation.candidate.manifest_identity_blake3);
    let has_matching_plan = validation.input.compilation_plan.as_ref().is_some_and(|plan| {
        validation.candidate.compilation_plan_identity_blake3 == plan.plan_identity_blake3
            && validation.candidate.generated_manifest_identity_blake3 == plan.generated_manifest_identity_blake3
    });
    let has_matching_target = validation.candidate.profile_identity_blake3 == *validation.profile_identity
        && validation.candidate.target_kernel_build_identity == validation.input.profile.target.kernel_build_identity
        && validation.candidate.target_architecture == validation.input.profile.target.architecture
        && validation.candidate.target_kernel_release == validation.input.profile.target.kernel_release
        && validation.candidate.btf_blake3 == validation.input.profile.target.btf.artifact.digest_blake3;
    let has_expected_members =
        !validation.expected_members.is_empty() && validation.candidate.members == validation.expected_members;
    let has_expected_inspections = validation.candidate.inspection_identities_blake3 == validation.expected_inspections;
    validation.is_kind_unique
        && crate::handoff::candidate_projection_is_bounded(validation.candidate)
        && has_matching_identity
        && has_matching_plan
        && has_matching_target
        && has_expected_members
        && has_expected_inspections
}

fn candidate_members(input: &ExperimentReceiptInput, class: ArtifactOutputClass) -> Vec<OutputMember> {
    let mut members = input
        .output_inspections
        .iter()
        .filter(|inspection| inspection.output_class == class)
        .map(inspection_output_member)
        .collect::<Vec<_>>();
    sort_output_members(&mut members);
    members
}

fn candidate_inspection_identities(input: &ExperimentReceiptInput, class: ArtifactOutputClass) -> Vec<Blake3Digest> {
    let mut inspections = input
        .output_inspections
        .iter()
        .filter(|inspection| inspection.output_class == class)
        .collect::<Vec<_>>();
    inspections.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    inspections.into_iter().map(|inspection| inspection.inspection_identity_blake3.clone()).collect()
}

fn inspection_output_member(inspection: &OutputInspection) -> OutputMember {
    OutputMember {
        relative_path: inspection.relative_path.clone(),
        digest_blake3: inspection.digest_blake3.clone(),
        size_bytes: inspection.size_bytes,
    }
}

fn sort_output_members(members: &mut [OutputMember]) {
    members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
}

fn validate_stage_status(input: &ExperimentReceiptInput, blockers: &mut Vec<ExperimentBlocker>) {
    let initial_blocker_count: usize = blockers.len();
    let is_codegen_present = input.codegen_plan.is_some();
    let is_classified_present = input.generated_manifest.is_some() && input.compilation_plan.is_some();
    let is_target_admitted = input
        .target_admission
        .as_ref()
        .is_some_and(|target| target.admitted && target.target_identity_blake3.is_some() && target.blockers.is_empty());
    let is_inspection_set_clean = !input.output_inspections.is_empty()
        && input.output_inspections.iter().all(|inspection| inspection.accepted)
        && input.blockers.is_empty();
    let is_stage_invalid = match input.stage_status {
        ExperimentStageStatus::Blocked => input.blockers.is_empty(),
        ExperimentStageStatus::Planned => !is_codegen_present || input.generated_manifest.is_some(),
        ExperimentStageStatus::CodegenClassified => !is_codegen_present || !is_classified_present,
        ExperimentStageStatus::PartiallyInspected => {
            !is_codegen_present || !is_classified_present || !is_target_admitted || input.output_inspections.is_empty()
        }
        ExperimentStageStatus::Inspected => {
            !is_codegen_present || !is_classified_present || !is_target_admitted || !is_inspection_set_clean
        }
    };
    if is_stage_invalid {
        blockers.push(blocker(
            "receipt-stage-status-overclaim",
            "receipt.stage_status",
            "receipt stage status exceeds its attached evidence",
        ));
    }
    debug_assert!(blockers.len() >= initial_blocker_count);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn validate_no_leaks(input: &ExperimentReceiptInput, blockers: &mut Vec<ExperimentBlocker>) {
    for item in &input.blockers {
        let has_forbidden_fragment = FORBIDDEN_RECEIPT_FRAGMENTS
            .iter()
            .any(|fragment| item.subject.contains(fragment) || item.message.contains(fragment));
        if has_forbidden_fragment || item.subject.starts_with('/') {
            blockers.push(blocker(
                "receipt-sensitive-data",
                "receipt.blockers",
                "receipt blocker contains a host path or credential-like material",
            ));
            return;
        }
    }
}

fn canonicalize_input(input: &mut ExperimentReceiptInput) {
    input.output_classes.sort_by_key(|output| output.output_class);
    for class in &mut input.output_classes {
        class.members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        class.inspection_identities_blake3.sort();
        class.blocker_codes.sort();
    }
    input.output_inspections.sort_by(|left, right| {
        left.output_class.cmp(&right.output_class).then(left.relative_path.cmp(&right.relative_path))
    });
    input.candidate_packs.sort_by_key(|candidate| candidate.kind);
    input.blockers.sort();
}

fn identify_receipt(
    input: ExperimentReceiptInput,
    profile_identity: Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<ExperimentReceipt> {
    let non_claims = RECEIPT_NON_CLAIMS.iter().map(|value| String::from(*value)).collect::<Vec<_>>();
    let identity_input = receipt_identity_input(&input, profile_identity.clone(), non_claims.clone());
    let identity = match canonical_identity(&identity_input) {
        Ok(identity) => identity,
        Err(error) => {
            blockers.push(blocker("receipt-identity-failed", "receipt", &error.to_string()));
            return None;
        }
    };
    Some(receipt_from_input(input, identity_input, identity, non_claims))
}

fn receipt_identity_input(
    input: &ExperimentReceiptInput,
    profile_identity: Blake3Digest,
    non_claims: Vec<String>,
) -> ReceiptIdentityInput {
    ReceiptIdentityInput {
        schema: String::from(EXPERIMENT_RECEIPT_SCHEMA),
        stage_status: input.stage_status,
        experiment_id: input.profile.experiment_id.clone(),
        profile_identity_blake3: profile_identity,
        source_blake3: input.profile.source.digest_blake3.clone(),
        compiler_source_revision: input.profile.compiler.source_revision.clone(),
        compiler_source_archive_sha256: input.profile.compiler.source_archive_sha256.clone(),
        compiler_source_archive_blake3: input.profile.compiler.source_archive_blake3.clone(),
        compiler_dependency_lock_blake3: input.profile.compiler.dependency_lock.lock_blake3.clone(),
        expected_compiler_closure_blake3: input.profile.compiler.closure_blake3.clone(),
        admitted_compiler_materialization_blake3: input.compiler_admission.materialization_identity_blake3.clone(),
        codegen_command_identity_blake3: input.codegen_plan.as_ref().map(|plan| plan.plan_identity_blake3.clone()),
        generated_manifest_identity_blake3: input
            .generated_manifest
            .as_ref()
            .map(|manifest| manifest.manifest_identity_blake3.clone()),
        target_identity_blake3: input
            .target_admission
            .as_ref()
            .and_then(|admission| admission.target_identity_blake3.clone()),
        compilation_plan_identity_blake3: input.compilation_plan.as_ref().map(|plan| plan.plan_identity_blake3.clone()),
        output_classes: input.output_classes.clone(),
        output_inspections: input.output_inspections.clone(),
        candidate_packs: input.candidate_packs.clone(),
        blockers: input.blockers.clone(),
        non_claims,
    }
}

fn receipt_from_input(
    _input: ExperimentReceiptInput,
    identity_input: ReceiptIdentityInput,
    receipt_identity_blake3: Blake3Digest,
    non_claims: Vec<String>,
) -> ExperimentReceipt {
    ExperimentReceipt {
        schema: identity_input.schema,
        stage_status: identity_input.stage_status,
        experiment_id: identity_input.experiment_id,
        profile_identity_blake3: identity_input.profile_identity_blake3,
        source_blake3: identity_input.source_blake3,
        compiler_source_revision: identity_input.compiler_source_revision,
        compiler_source_archive_sha256: identity_input.compiler_source_archive_sha256,
        compiler_source_archive_blake3: identity_input.compiler_source_archive_blake3,
        compiler_dependency_lock_blake3: identity_input.compiler_dependency_lock_blake3,
        expected_compiler_closure_blake3: identity_input.expected_compiler_closure_blake3,
        admitted_compiler_materialization_blake3: identity_input.admitted_compiler_materialization_blake3,
        codegen_command_identity_blake3: identity_input.codegen_command_identity_blake3,
        generated_manifest_identity_blake3: identity_input.generated_manifest_identity_blake3,
        target_identity_blake3: identity_input.target_identity_blake3,
        compilation_plan_identity_blake3: identity_input.compilation_plan_identity_blake3,
        output_classes: identity_input.output_classes,
        output_inspections: identity_input.output_inspections,
        candidate_packs: identity_input.candidate_packs,
        blockers: identity_input.blockers,
        non_claims,
        receipt_identity_blake3,
    }
}
