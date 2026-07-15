use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactIdentity;
use crate::Blake3Digest;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::KernelArchitecture;
use crate::blocker::blocker;
use crate::digest::canonical_identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedKernelTargetFacts {
    pub architecture: KernelArchitecture,
    pub kernel_release: String,
    pub kernel_build_identity: String,
    pub btf: Option<ArtifactIdentity>,
    pub headers: Option<ArtifactIdentity>,
    pub config: Option<ArtifactIdentity>,
    pub compiler_flags_blake3: Blake3Digest,
    pub toolchain_blake3: Blake3Digest,
    pub ambient_inputs_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetAdmission {
    pub admitted: bool,
    pub target_identity_blake3: Option<Blake3Digest>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Serialize)]
struct CompilerFlagIdentityInput<'a> {
    bpf: &'a [alloc::string::String],
    userspace: &'a [alloc::string::String],
    module: &'a [alloc::string::String],
}

pub fn admit_kernel_target(profile: ExperimentProfile, facts: ResolvedKernelTargetFacts) -> TargetAdmission {
    let mut blockers = crate::validate_profile(profile.clone()).blockers;
    validate_target_identity(&profile, &facts, &mut blockers);
    validate_target_artifacts(&profile, &facts, &mut blockers);
    validate_target_cohort(&profile, &facts, &mut blockers);
    if !crate::profile::kernel_build_identity_has_onix_authority(&profile.target.kernel_build_identity) {
        blockers.push(blocker(
            "kernel-target-observation-only",
            "target",
            "materialized probe facts are observation evidence, not accepted Onix target authority",
        ));
    }
    if facts.ambient_inputs_used {
        blockers.push(blocker(
            "ambient-kernel-input",
            "target",
            "kernel target admission observed running-host input state",
        ));
    }
    let target_identity_blake3 = identity_if_clean(&facts, &mut blockers);
    debug_assert!(blockers.is_empty() == target_identity_blake3.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    TargetAdmission {
        admitted: blockers.is_empty(),
        target_identity_blake3,
        blockers,
    }
}

pub(crate) fn expected_compiler_flags_identity(profile: &ExperimentProfile) -> Option<Blake3Digest> {
    canonical_identity(&CompilerFlagIdentityInput {
        bpf: &profile.bpf_compiler_flags,
        userspace: &profile.userspace_compiler_flags,
        module: &profile.module_compiler_flags,
    })
    .ok()
}

pub(crate) fn expected_toolchain_identity(profile: &ExperimentProfile) -> Option<Blake3Digest> {
    canonical_identity(&profile.toolchain).ok()
}

fn validate_target_identity(
    profile: &ExperimentProfile,
    facts: &ResolvedKernelTargetFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let expected = &profile.target;
    if facts.architecture != expected.architecture
        || facts.kernel_release != expected.kernel_release
        || facts.kernel_build_identity != expected.kernel_build_identity
    {
        blockers.push(blocker(
            "kernel-target-mismatch",
            "target",
            "resolved architecture, release, or Onix kernel-build identity differs from the profile",
        ));
    }
}

fn validate_target_artifacts(
    profile: &ExperimentProfile,
    facts: &ResolvedKernelTargetFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let initial_blocker_count: usize = blockers.len();
    if facts.btf.as_ref() != Some(&profile.target.btf.artifact) {
        blockers.push(blocker(
            "missing-or-mismatched-btf",
            "target.btf",
            "BTF is absent or differs from the target profile",
        ));
    }
    if facts.headers.as_ref() != Some(&profile.target.headers.artifact) {
        blockers.push(blocker(
            "missing-or-mismatched-headers",
            "target.headers",
            "kernel headers are absent or differ from the target profile",
        ));
    }
    if facts.config.as_ref() != Some(&profile.target.config.artifact) {
        blockers.push(blocker(
            "missing-or-mismatched-config",
            "target.config",
            "kernel config is absent or differs from the target profile",
        ));
    }
    debug_assert!(blockers.len() >= initial_blocker_count);
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
}

fn validate_target_cohort(
    profile: &ExperimentProfile,
    facts: &ResolvedKernelTargetFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if expected_compiler_flags_identity(profile).as_ref() != Some(&facts.compiler_flags_blake3) {
        blockers.push(blocker(
            "compiler-flag-cohort-mismatch",
            "target.compiler_flags",
            "compiler flags changed after planning",
        ));
    }
    if expected_toolchain_identity(profile).as_ref() != Some(&facts.toolchain_blake3) {
        blockers.push(blocker(
            "target-toolchain-mismatch",
            "target.toolchain",
            "toolchain identities changed after planning",
        ));
    }
}

fn identity_if_clean(facts: &ResolvedKernelTargetFacts, blockers: &mut Vec<ExperimentBlocker>) -> Option<Blake3Digest> {
    if !blockers.is_empty() {
        return None;
    }
    match canonical_identity(facts) {
        Ok(identity) => Some(identity),
        Err(error) => {
            blockers.push(blocker("target-identity-failed", "target", &error.to_string()));
            None
        }
    }
}
