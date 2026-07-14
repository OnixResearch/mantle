use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::CorpusFact;
use crate::Diagnostic;
use crate::ReferenceKind;
use crate::ReferenceProfile;
use crate::TargetFact;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFacts {
    pub reference_kind: ReferenceKind,
    pub revision: String,
    pub archive_blake3: Blake3Digest,
    pub cargo_lock_blake3: Blake3Digest,
    pub dependency_manifest_blake3: Blake3Digest,
    pub dependency_package_count: u32,
    pub octet_support_projection_blake3: Blake3Digest,
    pub rust_version: String,
    pub targets: Vec<TargetFact>,
    pub present_licenses: Vec<String>,
    pub present_notices: Vec<String>,
    pub corpora: Vec<CorpusFact>,
    pub network_attempted_during_build: bool,
    pub fallback_acquisition_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAdmission {
    pub admitted: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn admit_source(profile: ReferenceProfile, facts: SourceFacts) -> SourceAdmission {
    let mut diagnostics = Vec::new();
    compare_source_identity(&profile, &facts, &mut diagnostics);
    compare_toolchain_and_targets(&profile, &facts, &mut diagnostics);
    compare_legal_members(&profile, &facts, &mut diagnostics);
    compare_corpora(&profile, &facts, &mut diagnostics);
    if facts.network_attempted_during_build || facts.fallback_acquisition_used {
        diagnostics.push(error(
            "mutable-build-input-attempted",
            "source-admission",
            "offline build admission forbids network access and fallback acquisition",
        ));
    }
    let diagnostics = ordered(diagnostics);
    let admitted = !has_errors(&diagnostics);
    debug_assert_eq!(admitted, diagnostics.iter().all(|item| item.severity != crate::DiagnosticSeverity::Error));
    debug_assert!(diagnostics.iter().all(|item| !item.code.is_empty()));
    SourceAdmission { admitted, diagnostics }
}

fn compare_source_identity(profile: &ReferenceProfile, facts: &SourceFacts, diagnostics: &mut Vec<Diagnostic>) {
    if facts.reference_kind != ReferenceKind::ExactCommit {
        diagnostics.push(error(
            "floating-source-ref",
            "source.reference",
            "source admission requires an exact commit reference",
        ));
    }
    if facts.revision != profile.source.revision || facts.archive_blake3 != profile.source.archive_blake3 {
        diagnostics.push(error(
            "source-identity-mismatch",
            "source.archive",
            "source revision or archive BLAKE3 differs from the profile",
        ));
    }
    if facts.cargo_lock_blake3 != profile.source.cargo_lock_blake3 {
        diagnostics.push(error(
            "stale-cargo-lock",
            "source.Cargo.lock",
            "Cargo.lock BLAKE3 differs from the reviewed profile",
        ));
    }
    if facts.dependency_manifest_blake3 != profile.source.dependency_manifest_blake3
        || facts.dependency_package_count != profile.source.dependency_package_count
    {
        diagnostics.push(error(
            "dependency-closure-mismatch",
            "source.dependencies",
            "dependency manifest identity or package count differs from the reviewed closure",
        ));
    }
    if facts.octet_support_projection_blake3 != profile.source.octet_support_projection_blake3 {
        diagnostics.push(error(
            "support-projection-drift",
            "source.octet-support-projection",
            "Octet support projection bytes differ from the selected cohort",
        ));
    }
    debug_assert!(!facts.revision.is_empty());
    debug_assert!(facts.dependency_package_count > 0 || !diagnostics.is_empty());
}

fn compare_toolchain_and_targets(profile: &ReferenceProfile, facts: &SourceFacts, diagnostics: &mut Vec<Diagnostic>) {
    if facts.rust_version != profile.toolchain.rust_version {
        diagnostics.push(error(
            "wrong-rust-toolchain",
            "toolchain.rust-version",
            "observed Rust version differs from the exact profile",
        ));
    }
    let expected: BTreeMap<_, _> = profile.targets.iter().map(|target| (target.role, target)).collect();
    let observed: BTreeMap<_, _> = facts.targets.iter().map(|target| (target.role, target)).collect();
    if expected.len() != profile.targets.len() || observed.len() != facts.targets.len() {
        diagnostics.push(error("duplicate-target-role", "toolchain.targets", "target roles must be unique"));
    }
    for (role, expected_target) in expected {
        let Some(observed_target) = observed.get(&role) else {
            diagnostics.push(error(
                "missing-target-fact",
                "toolchain.targets",
                "observed build facts omit a declared target",
            ));
            continue;
        };
        if observed_target.triple != expected_target.triple
            || observed_target.pointer_width_bits != expected_target.pointer_width_bits
            || normalized_features(&observed_target.features) != normalized_features(&expected_target.features)
        {
            diagnostics.push(error(
                "target-identity-mismatch",
                &expected_target.triple,
                "target triple, pointer width, or feature identity differs from the profile",
            ));
        }
    }
    debug_assert!(expected_target_count(profile) > 0);
    debug_assert!(observed.len() <= facts.targets.len());
}

fn compare_legal_members(profile: &ReferenceProfile, facts: &SourceFacts, diagnostics: &mut Vec<Diagnostic>) {
    let licenses: BTreeSet<_> = facts.present_licenses.iter().collect();
    let notices: BTreeSet<_> = facts.present_notices.iter().collect();
    for required in &profile.source.license_members {
        if !licenses.contains(required) {
            diagnostics.push(error("omitted-license", required, "required upstream or corpus license is absent"));
        }
    }
    for required in &profile.source.notice_members {
        if !notices.contains(required) {
            diagnostics.push(error("omitted-notice", required, "required upstream or corpus notice is absent"));
        }
    }
    debug_assert!(licenses.len() <= facts.present_licenses.len());
    debug_assert!(notices.len() <= facts.present_notices.len());
}

fn compare_corpora(profile: &ReferenceProfile, facts: &SourceFacts, diagnostics: &mut Vec<Diagnostic>) {
    let observed: BTreeMap<String, Blake3Digest> = facts
        .corpora
        .iter()
        .map(|corpus| (corpus.corpus_id.clone(), corpus.descriptor_blake3.clone()))
        .collect();
    if observed.len() != facts.corpora.len() {
        diagnostics.push(error("duplicate-corpus-fact", "corpora", "observed corpus ids must be unique"));
    }
    for expected in &profile.corpora {
        match observed.get(&expected.corpus_id) {
            Some(digest) if digest == &expected.descriptor_blake3 => {}
            Some(_) => diagnostics.push(error(
                "corpus-drift",
                &expected.corpus_id,
                "corpus descriptor BLAKE3 differs from the profile",
            )),
            None => {
                diagnostics.push(error("missing-corpus", &expected.corpus_id, "required corpus descriptor is absent"))
            }
        }
    }
    debug_assert!(observed.len() <= facts.corpora.len());
    debug_assert!(profile.corpora.is_empty() || !observed.is_empty() || !diagnostics.is_empty());
}

fn normalized_features(features: &[String]) -> BTreeSet<String> {
    features.iter().cloned().collect()
}

fn expected_target_count(profile: &ReferenceProfile) -> usize {
    profile.targets.len()
}
