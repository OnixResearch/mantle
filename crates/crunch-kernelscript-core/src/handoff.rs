use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactOutputClass;
use crate::Blake3Digest;
use crate::CompilationPlan;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::KernelArchitecture;
use crate::OutputInspection;
use crate::OutputMember;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const CANDIDATE_PACK_SCHEMA: &str = "mantle-kernelscript-candidate-pack-v1";
const CANDIDATE_PACK_NON_CLAIMS: &[&str] = &[
    "experimental-unverified",
    "not-bpf-verifier-acceptance",
    "not-kernel-load-or-attach-evidence",
    "not-deployable",
    "not-production-supported",
    "onix-semantics-external",
    "chaoscontrol-runtime-evidence-required",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CandidatePackKind {
    ModulePack,
    BpfPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CandidateReadiness {
    ExperimentalUnverified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidatePackProjection {
    pub schema: String,
    pub name: String,
    pub kind: CandidatePackKind,
    pub readiness: CandidateReadiness,
    pub target_kernel_build_identity: String,
    pub target_architecture: KernelArchitecture,
    pub target_kernel_release: String,
    pub btf_blake3: Blake3Digest,
    pub members: Vec<OutputMember>,
    pub inspection_identities_blake3: Vec<Blake3Digest>,
    pub profile_identity_blake3: Blake3Digest,
    pub generated_manifest_identity_blake3: Blake3Digest,
    pub compilation_plan_identity_blake3: Blake3Digest,
    pub non_claims: Vec<String>,
    pub manifest_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateProjectionResult {
    pub projections: Vec<CandidatePackProjection>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Serialize)]
struct CandidateIdentityInput {
    schema: String,
    name: String,
    kind: CandidatePackKind,
    readiness: CandidateReadiness,
    target_kernel_build_identity: String,
    target_architecture: KernelArchitecture,
    target_kernel_release: String,
    btf_blake3: Blake3Digest,
    members: Vec<OutputMember>,
    inspection_identities_blake3: Vec<Blake3Digest>,
    profile_identity_blake3: Blake3Digest,
    generated_manifest_identity_blake3: Blake3Digest,
    compilation_plan_identity_blake3: Blake3Digest,
    non_claims: Vec<String>,
}

pub fn project_candidate_packs(
    profile: ExperimentProfile,
    plan: CompilationPlan,
    inspections: Vec<OutputInspection>,
) -> CandidateProjectionResult {
    let validation = crate::validate_profile(profile.clone());
    let mut blockers = validation.blockers;
    let Some(profile_identity) = validation.profile_identity_blake3 else {
        return CandidateProjectionResult {
            projections: Vec::new(),
            blockers,
        };
    };
    validate_plan_binding(&profile, &plan, &profile_identity, &mut blockers);
    validate_inspection_set(&profile, &plan, &inspections, &mut blockers);
    let mut projections = Vec::new();
    project_selected_kind(
        &profile,
        &plan,
        &inspections,
        &profile_identity,
        CandidatePackKind::BpfPack,
        ArtifactOutputClass::EbpfObject,
        &mut projections,
        &mut blockers,
    );
    project_selected_kind(
        &profile,
        &plan,
        &inspections,
        &profile_identity,
        CandidatePackKind::ModulePack,
        ArtifactOutputClass::KernelModule,
        &mut projections,
        &mut blockers,
    );
    projections.sort_by_key(|projection| projection.kind);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    debug_assert!(
        projections
            .iter()
            .all(|projection| projection.readiness == CandidateReadiness::ExperimentalUnverified)
    );
    CandidateProjectionResult { projections, blockers }
}

fn validate_plan_binding(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    profile_identity: &Blake3Digest,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if &plan.profile_identity_blake3 != profile_identity
        || crate::planner::expected_compilation_plan_identity(plan).as_ref() != Some(&plan.plan_identity_blake3)
        || plan.target_kernel_build_identity != profile.target.kernel_build_identity
        || plan.target_architecture != profile.target.architecture
        || plan.target_kernel_release != profile.target.kernel_release
        || !crate::profile::kernel_build_identity_has_onix_authority(&profile.target.kernel_build_identity)
    {
        blockers.push(blocker(
            "candidate-plan-mismatch",
            "candidate-pack",
            "candidate plan does not bind the exact profile and target",
        ));
    }
}

fn validate_inspection_set(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    inspections: &[OutputInspection],
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if count_above_bound(inspections.len(), profile.bounds.max_output_files) {
        blockers.push(blocker(
            "candidate-inspection-limit",
            "candidate-pack",
            "candidate inspection set exceeds the output bound",
        ));
        return;
    }
    let mut keys = BTreeSet::new();
    for inspection in inspections {
        if !inspection.accepted || inspection.plan_identity_blake3 != plan.plan_identity_blake3 {
            blockers.push(blocker(
                "unadmitted-candidate-member",
                &inspection.relative_path,
                "candidate member lacks exact static inspection admission",
            ));
        }
        if crate::inspection::expected_inspection_identity(inspection).as_ref()
            != Some(&inspection.inspection_identity_blake3)
        {
            blockers.push(blocker(
                "candidate-inspection-identity-mismatch",
                &inspection.relative_path,
                "candidate inspection fields differ from their BLAKE3 evidence identity",
            ));
        }
        if !keys.insert((inspection.output_class, inspection.relative_path.clone())) {
            blockers.push(blocker(
                "duplicate-candidate-member",
                &inspection.relative_path,
                "candidate member is duplicated",
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn project_selected_kind(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    inspections: &[OutputInspection],
    profile_identity: &Blake3Digest,
    kind: CandidatePackKind,
    class: ArtifactOutputClass,
    projections: &mut Vec<CandidatePackProjection>,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if !profile.output_classes.contains(&class) {
        return;
    }
    let mut admitted = inspections
        .iter()
        .filter(|inspection| inspection.output_class == class && inspection.accepted)
        .collect::<Vec<_>>();
    admitted.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let members = admitted.iter().map(|inspection| output_member(inspection)).collect::<Vec<_>>();
    let inspection_identities =
        admitted.iter().map(|inspection| inspection.inspection_identity_blake3.clone()).collect::<Vec<_>>();
    if members.is_empty() {
        blockers.push(blocker(
            "missing-candidate-members",
            &candidate_kind_name(kind),
            "selected candidate pack has no admitted members",
        ));
        return;
    }
    if let Some(projection) =
        identify_candidate(profile, plan, profile_identity, kind, members, inspection_identities, blockers)
    {
        projections.push(projection);
    }
}

fn identify_candidate(
    profile: &ExperimentProfile,
    plan: &CompilationPlan,
    profile_identity: &Blake3Digest,
    kind: CandidatePackKind,
    members: Vec<OutputMember>,
    inspection_identities_blake3: Vec<Blake3Digest>,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<CandidatePackProjection> {
    let non_claims = CANDIDATE_PACK_NON_CLAIMS.iter().map(|value| String::from(*value)).collect::<Vec<_>>();
    let input = CandidateIdentityInput {
        schema: String::from(CANDIDATE_PACK_SCHEMA),
        name: format!("{}-{}", profile.experiment_id, candidate_kind_name(kind)),
        kind,
        readiness: CandidateReadiness::ExperimentalUnverified,
        target_kernel_build_identity: profile.target.kernel_build_identity.clone(),
        target_architecture: profile.target.architecture,
        target_kernel_release: profile.target.kernel_release.clone(),
        btf_blake3: profile.target.btf.artifact.digest_blake3.clone(),
        members: members.clone(),
        inspection_identities_blake3: inspection_identities_blake3.clone(),
        profile_identity_blake3: profile_identity.clone(),
        generated_manifest_identity_blake3: plan.generated_manifest_identity_blake3.clone(),
        compilation_plan_identity_blake3: plan.plan_identity_blake3.clone(),
        non_claims: non_claims.clone(),
    };
    let identity = match canonical_identity(&input) {
        Ok(identity) => identity,
        Err(error) => {
            blockers.push(blocker("candidate-manifest-identity-failed", "candidate-pack", &error.to_string()));
            return None;
        }
    };
    Some(CandidatePackProjection {
        schema: input.schema,
        name: input.name,
        kind,
        readiness: input.readiness,
        target_kernel_build_identity: input.target_kernel_build_identity,
        target_architecture: input.target_architecture,
        target_kernel_release: input.target_kernel_release,
        btf_blake3: input.btf_blake3,
        members,
        inspection_identities_blake3,
        profile_identity_blake3: input.profile_identity_blake3,
        generated_manifest_identity_blake3: input.generated_manifest_identity_blake3,
        compilation_plan_identity_blake3: input.compilation_plan_identity_blake3,
        non_claims,
        manifest_identity_blake3: identity,
    })
}

pub(crate) fn candidate_projection_is_bounded(candidate: &CandidatePackProjection) -> bool {
    candidate.schema == CANDIDATE_PACK_SCHEMA
        && candidate.readiness == CandidateReadiness::ExperimentalUnverified
        && CANDIDATE_PACK_NON_CLAIMS
            .iter()
            .all(|required| candidate.non_claims.iter().any(|actual| actual == required))
}

pub(crate) fn expected_candidate_identity(candidate: &CandidatePackProjection) -> Option<Blake3Digest> {
    canonical_identity(&CandidateIdentityInput {
        schema: candidate.schema.clone(),
        name: candidate.name.clone(),
        kind: candidate.kind,
        readiness: candidate.readiness,
        target_kernel_build_identity: candidate.target_kernel_build_identity.clone(),
        target_architecture: candidate.target_architecture,
        target_kernel_release: candidate.target_kernel_release.clone(),
        btf_blake3: candidate.btf_blake3.clone(),
        members: candidate.members.clone(),
        inspection_identities_blake3: candidate.inspection_identities_blake3.clone(),
        profile_identity_blake3: candidate.profile_identity_blake3.clone(),
        generated_manifest_identity_blake3: candidate.generated_manifest_identity_blake3.clone(),
        compilation_plan_identity_blake3: candidate.compilation_plan_identity_blake3.clone(),
        non_claims: candidate.non_claims.clone(),
    })
    .ok()
}

fn output_member(inspection: &OutputInspection) -> OutputMember {
    OutputMember {
        relative_path: inspection.relative_path.clone(),
        digest_blake3: inspection.digest_blake3.clone(),
        size_bytes: inspection.size_bytes,
    }
}

fn candidate_kind_name(kind: CandidatePackKind) -> String {
    match kind {
        CandidatePackKind::ModulePack => String::from("module-pack"),
        CandidatePackKind::BpfPack => String::from("bpf-pack"),
    }
}
