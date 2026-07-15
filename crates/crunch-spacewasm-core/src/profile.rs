use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

use crate::Blake3Digest;
use crate::CheckStatus;
use crate::CorpusRole;
use crate::Diagnostic;
use crate::FixtureClass;
use crate::ReferenceProfile;
use crate::SupportStatus;
use crate::TargetRole;
use crate::diagnostic::ErrorDiagnostic;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;
use crate::digest::canonical_identity;
use crate::digest::count_exceeds;
use crate::digest::count_is_below;

pub const PROFILE_SCHEMA: &str = "mantle-spacewasm-reference-profile-v1";
pub const GIT_REVISION_HEX_LENGTH: usize = 40;
pub const POINTER_WIDTH_32_BITS: u32 = 32;
pub const POINTER_WIDTH_64_BITS: u32 = 64;
pub const MINIMUM_REQUIRED_FIXTURES: u32 = 8;
pub const MINIMUM_REQUIRED_CORPORA: u32 = 2;
pub const MINIMUM_REQUIRED_TARGETS: u32 = 3;
pub const MINIMUM_REQUIRED_CHECKS: u32 = 5;

pub const REQUIRED_NON_CLAIMS: [&str; 8] = [
    "not-spacewasm-correctness",
    "not-memory-safety",
    "not-webassembly-conformance",
    "not-flight-qualification",
    "not-sandbox-effectiveness",
    "not-consumer-runtime-admission",
    "not-production-readiness",
    "not-release-eligibility",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileValidation {
    pub profile_identity_blake3: Option<Blake3Digest>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn cohort_identity(mut profile: ReferenceProfile) -> Result<Blake3Digest, crate::DigestError> {
    profile.targets.sort();
    profile.support_matrix.sort();
    profile.requested_features.sort();
    let input = CohortIdentityInput {
        profile_id: profile.profile_id,
        source: profile.source,
        toolchain: profile.toolchain,
        targets: profile.targets,
        support_matrix: profile.support_matrix,
        requested_features: profile.requested_features,
    };
    debug_assert!(!input.profile_id.is_empty());
    debug_assert!(!input.targets.is_empty());
    canonical_identity(input)
}

pub fn validate_profile(profile: ReferenceProfile) -> ProfileValidation {
    let mut diagnostics = Vec::new();
    validate_header(&profile, &mut diagnostics);
    validate_update(&profile, &mut diagnostics);
    validate_source(&profile, &mut diagnostics);
    validate_targets(&profile, &mut diagnostics);
    validate_support(&profile, &mut diagnostics);
    validate_fixtures_and_corpora(&profile, &mut diagnostics);
    validate_checks(&profile, &mut diagnostics);
    validate_retention_and_non_claims(&profile, &mut diagnostics);
    validate_bounds(&profile, &mut diagnostics);
    let diagnostics = ordered(diagnostics);
    let profile_identity_blake3 = if has_errors(&diagnostics) {
        None
    } else {
        canonical_identity(normalized_identity_input(profile.clone())).ok()
    };
    debug_assert_eq!(profile_identity_blake3.is_some(), !has_errors(&diagnostics));
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.code.is_empty()));
    ProfileValidation {
        profile_identity_blake3,
        diagnostics,
    }
}

fn validate_header(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    if profile.schema != PROFILE_SCHEMA {
        diagnostics.push(error(ErrorDiagnostic {
            code: "unsupported-profile-schema",
            subject: "profile.schema",
            message: "SpaceWasm profile schema is unsupported",
        }));
    }
    if profile.profile_id.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "empty-profile-id",
            subject: "profile.profile_id",
            message: "SpaceWasm profile id must not be empty",
        }));
    }
    debug_assert!(PROFILE_SCHEMA.contains("spacewasm"));
    debug_assert!(!PROFILE_SCHEMA.is_empty());
}

fn validate_update(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let update = &profile.profile_update;
    if !is_git_revision(&update.previous_review_revision) || !is_git_revision(&update.selected_revision) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-profile-update-revision",
            subject: "profile.profile_update",
            message: "profile update revisions must be exact lowercase Git commit identifiers",
        }));
    }
    if update.selected_revision != profile.source.revision
        || update.selected_revision == update.previous_review_revision
    {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-profile-update-selection",
            subject: "profile.profile_update",
            message: "selected revision must match source and differ from the proposal review revision",
        }));
    }
    if !update.replay_required || update.replay_evidence_id.is_empty() || update.reason.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "missing-profile-update-replay",
            subject: "profile.profile_update",
            message: "cohort update requires a reason and named complete replay evidence",
        }));
    }
    debug_assert!(update.replay_required || !diagnostics.is_empty());
    debug_assert!(!update.selected_revision.is_empty() || !diagnostics.is_empty());
}

fn validate_source(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let source = &profile.source;
    if !is_git_revision(&source.revision) || source.repository != "https://github.com/nasa/spacewasm" {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-source-pin",
            subject: "profile.source",
            message: "source must bind the reviewed NASA SpaceWasm repository and exact commit",
        }));
    }
    if !source.archive_url.contains(&source.revision) || !source.archive_sha256_sri.starts_with("sha256-") {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-source-archive",
            subject: "profile.source.archive",
            message: "source archive must be content-addressed and revision-specific",
        }));
    }
    if source.dependency_package_count == 0 || source.license_members.is_empty() || source.notice_members.is_empty() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "incomplete-source-closure",
            subject: "profile.source",
            message: "source profile requires dependency, license, and notice members",
        }));
    }
    require_member(
        RequiredMember {
            members: &source.license_members,
            required: "LICENSE",
            code: "missing-license-member",
        },
        diagnostics,
    );
    require_member(
        RequiredMember {
            members: &source.notice_members,
            required: "NOTICE",
            code: "missing-notice-member",
        },
        diagnostics,
    );
    debug_assert!(!source.repository.is_empty());
    debug_assert!(source.dependency_package_count > 0 || !diagnostics.is_empty());
}

fn validate_targets(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let is_below_required = count_is_below(profile.targets.len(), MINIMUM_REQUIRED_TARGETS);
    if count_exceeds(profile.targets.len(), profile.bounds.max_collection_items) || is_below_required {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-target-count",
            subject: "profile.targets",
            message: "profile requires bounded host library, Wasm library, and host runner targets",
        }));
        return;
    }
    let mut roles = BTreeSet::new();
    for target in &profile.targets {
        if !roles.insert(target.role) || target.triple.is_empty() || target.features.is_empty() {
            diagnostics.push(error(ErrorDiagnostic {
                code: "invalid-target-profile",
                subject: &target.triple,
                message: "target roles and triples must be unique with explicit feature identity",
            }));
        }
        if target.pointer_width_bits != POINTER_WIDTH_32_BITS && target.pointer_width_bits != POINTER_WIDTH_64_BITS {
            diagnostics.push(error(ErrorDiagnostic {
                code: "invalid-pointer-width",
                subject: &target.triple,
                message: "target pointer width must be explicitly 32 or 64 bits",
            }));
        }
    }
    for required in [
        TargetRole::HostLibrary,
        TargetRole::WasmLibrary,
        TargetRole::HostDiagnosticRunner,
    ] {
        if !roles.contains(&required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-target-role",
                subject: "profile.targets",
                message: "profile omits a required target role",
            }));
        }
    }
    debug_assert!(roles.len() <= profile.targets.len());
    debug_assert!(profile.targets.iter().all(|target| !target.triple.is_empty()) || !diagnostics.is_empty());
}

fn validate_support(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let mut features = BTreeSet::new();
    for entry in &profile.support_matrix {
        if entry.feature.is_empty() || !features.insert(entry.feature.clone()) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "duplicate-support-entry",
                subject: &entry.feature,
                message: "support matrix feature names must be unique and non-empty",
            }));
        }
    }
    require_support(&features, "wasm1", diagnostics);
    require_support(&features, "mutable-globals", diagnostics);
    for requested in &profile.requested_features {
        let status = profile.support_matrix.iter().find(|entry| entry.feature == *requested).map(|entry| entry.status);
        if status != Some(SupportStatus::Supported) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "unsupported-feature-requested",
                subject: requested,
                message: "requested feature is not supported by the reviewed cohort",
            }));
        }
    }
    debug_assert!(features.len() <= profile.support_matrix.len());
    debug_assert!(!profile.requested_features.is_empty() || !diagnostics.is_empty());
}

fn validate_fixtures_and_corpora(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let is_fixture_count_below = count_is_below(profile.fixtures.len(), MINIMUM_REQUIRED_FIXTURES);
    let is_corpus_count_below = count_is_below(profile.corpora.len(), MINIMUM_REQUIRED_CORPORA);
    if is_fixture_count_below || is_corpus_count_below {
        diagnostics.push(error(ErrorDiagnostic {
            code: "incomplete-fixture-corpus-profile",
            subject: "profile.fixtures",
            message: "all required fixture and corpus classes must be declared",
        }));
    }
    let fixture_classes: BTreeSet<FixtureClass> = profile.fixtures.iter().map(|fixture| fixture.class).collect();
    for required in required_fixture_classes() {
        if !fixture_classes.contains(&required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-fixture-class",
                subject: "profile.fixtures",
                message: "profile omits a required SpaceWasm fixture class",
            }));
        }
    }
    unique_fixture_paths(profile, diagnostics);
    let corpus_roles: BTreeSet<CorpusRole> = profile.corpora.iter().map(|corpus| corpus.role).collect();
    for required in [CorpusRole::UpstreamSpectest, CorpusRole::UpstreamFuzz] {
        if !corpus_roles.contains(&required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-corpus-role",
                subject: "profile.corpora",
                message: "profile omits a required upstream corpus descriptor",
            }));
        }
    }
    debug_assert!(fixture_classes.len() <= profile.fixtures.len());
    debug_assert!(corpus_roles.len() <= profile.corpora.len());
}

fn validate_checks(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let is_check_count_below = count_is_below(profile.checks.len(), MINIMUM_REQUIRED_CHECKS);
    if is_check_count_below {
        diagnostics.push(error(ErrorDiagnostic {
            code: "incomplete-check-profile",
            subject: "profile.checks",
            message: "profile omits required build, test, unavailable, or unsupported checks",
        }));
    }
    let mut ids = BTreeSet::new();
    let mut statuses = BTreeSet::new();
    for check in &profile.checks {
        if check.check_id.is_empty() || !ids.insert(check.check_id.clone()) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "duplicate-check-id",
                subject: &check.check_id,
                message: "check ids must be unique and non-empty",
            }));
        }
        statuses.insert(check.expected_status);
    }
    for required in [
        CheckStatus::Passed,
        CheckStatus::Skipped,
        CheckStatus::Unavailable,
        CheckStatus::Unsupported,
    ] {
        if !statuses.contains(&required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-check-status-class",
                subject: "profile.checks",
                message: "profile must explicitly exercise bounded and unavailable/unsupported result classes",
            }));
        }
    }
    debug_assert!(ids.len() <= profile.checks.len());
    debug_assert!(statuses.len() <= profile.checks.len());
}

fn validate_retention_and_non_claims(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let retention = &profile.retention;
    let is_complete_retention = retention.source_archive
        && retention.dependency_closure
        && retention.toolchain
        && retention.binaries
        && retention.fixtures
        && retention.reports
        && retention.licenses_and_notices
        && retention.non_claims;
    if !is_complete_retention {
        diagnostics.push(error(ErrorDiagnostic {
            code: "incomplete-retention-policy",
            subject: "profile.retention",
            message: "reference bundle retention must preserve every required evidence class",
        }));
    }
    let non_claims: BTreeSet<String> = profile.non_claims.iter().cloned().collect();
    for required in REQUIRED_NON_CLAIMS {
        if !non_claims.contains(required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-required-non-claim",
                subject: required,
                message: "profile omits a required SpaceWasm claim boundary",
            }));
        }
    }
    if non_claims.len() != profile.non_claims.len() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "duplicate-non-claim",
            subject: "profile.non_claims",
            message: "profile non-claims must be unique",
        }));
    }
    debug_assert!(is_complete_retention || !diagnostics.is_empty());
    debug_assert!(non_claims.len() <= profile.non_claims.len());
}

fn validate_bounds(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let bounds = &profile.bounds;
    let is_all_positive = bounds.max_profile_text_bytes > 0
        && bounds.max_collection_items > 0
        && bounds.max_bundle_members > 0
        && bounds.max_parent_edges > 0
        && bounds.max_bundle_member_bytes > 0
        && bounds.max_bundle_total_bytes > 0;
    let is_collections_within_bound = !count_exceeds(profile.targets.len(), bounds.max_collection_items)
        && !count_exceeds(profile.support_matrix.len(), bounds.max_collection_items)
        && !count_exceeds(profile.fixtures.len(), bounds.max_collection_items)
        && !count_exceeds(profile.corpora.len(), bounds.max_collection_items)
        && !count_exceeds(profile.checks.len(), bounds.max_collection_items)
        && !count_exceeds(profile.non_claims.len(), bounds.max_collection_items);
    if !is_all_positive || !is_collections_within_bound {
        diagnostics.push(error(ErrorDiagnostic {
            code: "profile-bound-violation",
            subject: "profile.bounds",
            message: "profile limits must be positive and contain every declared collection",
        }));
    }
    debug_assert!(is_all_positive || !diagnostics.is_empty());
    debug_assert!(is_collections_within_bound || !diagnostics.is_empty());
}

struct FixturePathRegistry {
    ids: BTreeSet<String>,
    artifacts: BTreeSet<String>,
    descriptors: BTreeSet<String>,
}

fn unique_fixture_paths(profile: &ReferenceProfile, diagnostics: &mut Vec<Diagnostic>) {
    let mut registry = FixturePathRegistry {
        ids: BTreeSet::new(),
        artifacts: BTreeSet::new(),
        descriptors: BTreeSet::new(),
    };
    for fixture in &profile.fixtures {
        let is_fixture_valid = register_fixture_paths(fixture, &mut registry);
        if !is_fixture_valid {
            diagnostics.push(error(ErrorDiagnostic {
                code: "invalid-fixture-profile",
                subject: &fixture.fixture_id,
                message: "fixture ids, codes, artifact paths, and descriptor paths must be unique and non-empty",
            }));
        }
    }
    debug_assert!(registry.ids.len() <= profile.fixtures.len());
    debug_assert!(registry.artifacts.len() <= profile.fixtures.len());
}

fn register_fixture_paths(fixture: &crate::FixtureProfile, registry: &mut FixturePathRegistry) -> bool {
    if fixture.fixture_id.is_empty() {
        return false;
    }
    if fixture.expected_code.is_empty() {
        return false;
    }
    if !registry.ids.insert(fixture.fixture_id.clone()) {
        return false;
    }
    if !registry.artifacts.insert(fixture.artifact_path.clone()) {
        return false;
    }
    registry.descriptors.insert(fixture.descriptor_path.clone())
}

fn required_fixture_classes() -> [FixtureClass; 8] {
    [
        FixtureClass::MvpPositive,
        FixtureClass::MvpNegative,
        FixtureClass::StreamingPositive,
        FixtureClass::StreamingNegative,
        FixtureClass::AllocationFailure,
        FixtureClass::UnsupportedFeature,
        FixtureClass::Trap,
        FixtureClass::OutOfFuel,
    ]
}

fn require_support(features: &BTreeSet<String>, required: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !features.contains(required) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "missing-support-entry",
            subject: required,
            message: "support matrix omits a required reviewed feature",
        }));
    }
}

struct RequiredMember<'a> {
    members: &'a [String],
    required: &'a str,
    code: &'a str,
}

fn require_member(requirement: RequiredMember<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if !requirement.members.iter().any(|member| member == requirement.required) {
        diagnostics.push(error(ErrorDiagnostic {
            code: requirement.code,
            subject: requirement.required,
            message: "source profile omits a required legal member",
        }));
    }
}

fn is_git_revision(value: &str) -> bool {
    value.len() == GIT_REVISION_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Serialize)]
struct CohortIdentityInput {
    profile_id: String,
    source: crate::SourceProfile,
    toolchain: crate::ToolchainProfile,
    targets: Vec<crate::TargetProfile>,
    support_matrix: Vec<crate::SupportEntry>,
    requested_features: Vec<String>,
}

#[derive(Serialize)]
struct ProfileIdentityInput {
    profile: ReferenceProfile,
}

fn normalized_identity_input(mut profile: ReferenceProfile) -> ProfileIdentityInput {
    profile.targets.sort();
    profile.support_matrix.sort();
    profile.requested_features.sort();
    profile.fixtures.sort();
    profile.corpora.sort();
    profile.checks.sort();
    profile.non_claims.sort();
    debug_assert!(profile.targets.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(profile.fixtures.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    ProfileIdentityInput { profile }
}
