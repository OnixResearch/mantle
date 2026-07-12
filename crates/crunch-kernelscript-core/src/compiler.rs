use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ArtifactIdentity;
use crate::Blake3Digest;
use crate::CompilerDependency;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::Sha256Digest;
use crate::blocker::blocker;
use crate::digest::canonical_identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerMaterializationFacts {
    pub source_revision: String,
    pub source_archive_sha256: Sha256Digest,
    pub source_archive_blake3: Blake3Digest,
    pub dependency_lock_blake3: Option<Blake3Digest>,
    pub dependency_count: u32,
    pub dependencies: Vec<CompilerDependency>,
    pub compiler_executable: Option<ArtifactIdentity>,
    pub closure_blake3: Option<Blake3Digest>,
    pub network_attempted: bool,
    pub ambient_opam_used: bool,
    pub ambient_compiler_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerAdmission {
    pub admitted: bool,
    pub materialization_identity_blake3: Option<Blake3Digest>,
    pub blockers: Vec<ExperimentBlocker>,
}

pub fn admit_compiler_materialization(
    profile: ExperimentProfile,
    facts: CompilerMaterializationFacts,
) -> CompilerAdmission {
    let mut blockers = crate::validate_profile(profile.clone()).blockers;
    validate_source_facts(&profile, &facts, &mut blockers);
    validate_dependency_facts(&profile, &facts, &mut blockers);
    validate_output_facts(&profile, &facts, &mut blockers);
    validate_offline_facts(&facts, &mut blockers);
    let materialization_identity_blake3 = identity_if_clean(&facts, &mut blockers);
    debug_assert!(blockers.is_empty() == materialization_identity_blake3.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    CompilerAdmission {
        admitted: blockers.is_empty(),
        materialization_identity_blake3,
        blockers,
    }
}

fn validate_source_facts(
    profile: &ExperimentProfile,
    facts: &CompilerMaterializationFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let compiler = &profile.compiler;
    if facts.source_revision != compiler.source_revision
        || facts.source_archive_sha256 != compiler.source_archive_sha256
        || facts.source_archive_blake3 != compiler.source_archive_blake3
    {
        blockers.push(blocker(
            "compiler-source-drift",
            "compiler.source",
            "observed compiler source revision or archive identity differs from the profile",
        ));
    }
}

fn validate_dependency_facts(
    profile: &ExperimentProfile,
    facts: &CompilerMaterializationFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let expected = &profile.compiler.dependency_lock;
    if facts.dependency_lock_blake3.as_ref() != Some(&expected.lock_blake3)
        || facts.dependency_count != expected.dependency_count
        || facts.dependencies != expected.dependencies
    {
        blockers.push(blocker(
            "compiler-dependency-closure-mismatch",
            "compiler.dependency_lock",
            "compiler dependency lock is missing or differs from the declared closure",
        ));
    }
}

fn validate_output_facts(
    profile: &ExperimentProfile,
    facts: &CompilerMaterializationFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if facts.compiler_executable.as_ref() != Some(&profile.compiler.compiler_executable) {
        blockers.push(blocker(
            "compiler-executable-mismatch",
            "compiler.executable",
            "compiler executable bytes differ from the pinned profile identity",
        ));
    }
    if facts.closure_blake3.as_ref() != Some(&profile.compiler.closure_blake3) {
        blockers.push(blocker(
            "compiler-closure-mismatch",
            "compiler.closure",
            "compiler closure bytes differ from the pinned profile identity",
        ));
    }
}

fn validate_offline_facts(facts: &CompilerMaterializationFacts, blockers: &mut Vec<ExperimentBlocker>) {
    if facts.network_attempted {
        blockers.push(blocker(
            "compiler-network-attempt",
            "compiler",
            "compiler materialization attempted network access",
        ));
    }
    if facts.ambient_opam_used || facts.ambient_compiler_used {
        blockers.push(blocker(
            "ambient-compiler-state",
            "compiler",
            "compiler materialization used ambient opam or host compiler state",
        ));
    }
}

fn identity_if_clean(
    facts: &CompilerMaterializationFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<Blake3Digest> {
    if !blockers.is_empty() {
        return None;
    }
    match canonical_identity(facts) {
        Ok(identity) => Some(identity),
        Err(error) => {
            blockers.push(blocker("compiler-admission-identity-failed", "compiler", &error.to_string()));
            None
        }
    }
}
