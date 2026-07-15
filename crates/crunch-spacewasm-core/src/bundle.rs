use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::BundleMember;
use crate::BundleRole;
use crate::Diagnostic;
use crate::ParentEdge;
use crate::ReferenceProfile;
use crate::diagnostic::ErrorDiagnostic;
use crate::diagnostic::MAX_DIAGNOSTIC_COUNT;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;
use crate::digest::canonical_identity;
use crate::digest::count_exceeds;
use crate::profile::REQUIRED_NON_CLAIMS;

pub const BUNDLE_MANIFEST_SCHEMA: &str = "mantle-spacewasm-reference-bundle-v1";

const REQUIRED_SINGLETON_ROLES: [BundleRole; 18] = [
    BundleRole::SourceArchive,
    BundleRole::CargoLock,
    BundleRole::DependencyManifest,
    BundleRole::DependencyClosure,
    BundleRole::RustcBinary,
    BundleRole::CargoBinary,
    BundleRole::ToolchainArchive,
    BundleRole::FixtureGenerator,
    BundleRole::HostLibrary,
    BundleRole::WasmLibrary,
    BundleRole::HostRunner,
    BundleRole::ProfileSource,
    BundleRole::ProfileExport,
    BundleRole::SupportProjection,
    BundleRole::ResultReport,
    BundleRole::ReplayEvidence,
    BundleRole::MaterializationReport,
    BundleRole::NonClaims,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleManifestInput {
    pub profile: ReferenceProfile,
    pub profile_identity_blake3: Blake3Digest,
    pub cohort_identity_blake3: Blake3Digest,
    pub members: Vec<BundleMember>,
    pub parent_edges: Vec<ParentEdge>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub schema: String,
    pub profile_id: String,
    pub profile_identity_blake3: Blake3Digest,
    pub cohort_identity_blake3: Blake3Digest,
    pub members: Vec<BundleMember>,
    pub parent_edges: Vec<ParentEdge>,
    pub non_claims: Vec<String>,
    pub bundle_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleVerification {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn plan_bundle_parent_edges(
    members: Vec<BundleMember>,
    root_path: String,
) -> Result<Vec<ParentEdge>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let member_paths: BTreeSet<String> = members.iter().map(|member| member.path.clone()).collect();
    if !member_paths.contains(&root_path) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "missing-parent-root",
            subject: &root_path,
            message: "bundle parent root is not a declared member",
        }));
    }
    if members.iter().filter(|member| member.path == root_path).count() != 1 {
        diagnostics.push(error(ErrorDiagnostic {
            code: "invalid-parent-root",
            subject: &root_path,
            message: "bundle parent root must be unique",
        }));
    }
    if has_errors(&diagnostics) {
        return Err(ordered(diagnostics));
    }
    let mut edges: Vec<_> = members
        .into_iter()
        .filter(|member| member.path != root_path)
        .map(|member| ParentEdge {
            parent_path: root_path.clone(),
            child_path: member.path,
            relation: String::from("profile-binds-member"),
        })
        .collect();
    edges.sort();
    debug_assert!(edges.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert_eq!(edges.len().saturating_add(1), member_paths.len());
    Ok(edges)
}

pub fn build_bundle_manifest(mut input: BundleManifestInput) -> Result<BundleManifest, Vec<Diagnostic>> {
    normalize_bundle_input(&mut input);
    let diagnostics = validate_bundle_input(&input);
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }
    let identity_input = BundleIdentityInput {
        schema: String::from(BUNDLE_MANIFEST_SCHEMA),
        profile_id: input.profile.profile_id.clone(),
        profile_identity_blake3: input.profile_identity_blake3.clone(),
        cohort_identity_blake3: input.cohort_identity_blake3.clone(),
        members: input.members.clone(),
        parent_edges: input.parent_edges.clone(),
        non_claims: input.non_claims.clone(),
    };
    let bundle_identity_blake3 = canonical_identity(identity_input).map_err(|_| {
        vec![error(ErrorDiagnostic {
            code: "bundle-identity-failed",
            subject: "bundle",
            message: "bundle manifest could not be canonically identified",
        })]
    })?;
    debug_assert!(!input.members.is_empty());
    debug_assert!(!input.parent_edges.is_empty());
    Ok(BundleManifest {
        schema: String::from(BUNDLE_MANIFEST_SCHEMA),
        profile_id: input.profile.profile_id,
        profile_identity_blake3: input.profile_identity_blake3,
        cohort_identity_blake3: input.cohort_identity_blake3,
        members: input.members,
        parent_edges: input.parent_edges,
        non_claims: input.non_claims,
        bundle_identity_blake3,
    })
}

pub fn verify_bundle_manifest(manifest: BundleManifest, measured: Vec<BundleMember>) -> BundleVerification {
    let reserved_diagnostic_count = manifest.members.len().saturating_add(measured.len()).min(MAX_DIAGNOSTIC_COUNT);
    let mut diagnostics = Vec::with_capacity(reserved_diagnostic_count);
    if manifest.schema != BUNDLE_MANIFEST_SCHEMA {
        diagnostics.push(error(ErrorDiagnostic {
            code: "unsupported-bundle-schema",
            subject: "bundle.schema",
            message: "bundle manifest schema is unsupported",
        }));
    }
    let measured_by_path = member_map(&measured, "measured", &mut diagnostics);
    let declared_by_path = member_map(&manifest.members, "manifest", &mut diagnostics);
    for (path, declared) in &declared_by_path {
        match measured_by_path.get(path) {
            Some(actual) if actual == declared => {}
            Some(_) => diagnostics.push(error(ErrorDiagnostic {
                code: "bundle-member-tamper",
                subject: path,
                message: "measured member role, size, or BLAKE3 differs from the manifest",
            })),
            None => diagnostics.push(error(ErrorDiagnostic {
                code: "incomplete-bundle",
                subject: path,
                message: "required bundle member is missing",
            })),
        }
    }
    for path in measured_by_path.keys() {
        if !declared_by_path.contains_key(path) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "undeclared-bundle-member",
                subject: path,
                message: "bundle contains an undeclared portable member",
            }));
        }
    }
    validate_manifest_identity(&manifest, &mut diagnostics);
    let diagnostics = ordered(diagnostics);
    let is_valid = !has_errors(&diagnostics);
    debug_assert_eq!(is_valid, diagnostics.is_empty());
    debug_assert!(diagnostics.iter().all(|item| !item.code.is_empty()));
    BundleVerification {
        valid: is_valid,
        diagnostics,
    }
}

fn validate_bundle_input(input: &BundleManifestInput) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_bundle_bounds(input, &mut diagnostics);
    let members = member_map(&input.members, "manifest", &mut diagnostics);
    validate_required_roles(input, &members, &mut diagnostics);
    validate_profile_bound_digests(input, &members, &mut diagnostics);
    validate_fixture_and_corpus_members(input, &members, &mut diagnostics);
    validate_legal_members(input, &members, &mut diagnostics);
    validate_parent_edges(input, &members, &mut diagnostics);
    validate_non_claims(&input.non_claims, &mut diagnostics);
    ordered(diagnostics)
}

fn validate_bundle_bounds(input: &BundleManifestInput, diagnostics: &mut Vec<Diagnostic>) {
    if count_exceeds(input.members.len(), input.profile.bounds.max_bundle_members)
        || count_exceeds(input.parent_edges.len(), input.profile.bounds.max_parent_edges)
    {
        diagnostics.push(error(ErrorDiagnostic {
            code: "bundle-limit",
            subject: "bundle",
            message: "bundle member or parent-edge collection exceeds the profile bound",
        }));
    }
    let mut total_bytes = 0_u64;
    for member in &input.members {
        if member.size_bytes == 0 || member.size_bytes > input.profile.bounds.max_bundle_member_bytes {
            diagnostics.push(error(ErrorDiagnostic {
                code: "bundle-member-size",
                subject: &member.path,
                message: "bundle member size is zero or exceeds the profile bound",
            }));
        }
        total_bytes = total_bytes.saturating_add(member.size_bytes);
    }
    if total_bytes > input.profile.bounds.max_bundle_total_bytes {
        diagnostics.push(error(ErrorDiagnostic {
            code: "bundle-total-size",
            subject: "bundle",
            message: "bundle total byte count exceeds the profile bound",
        }));
    }
    debug_assert!(total_bytes > 0 || !diagnostics.is_empty());
    debug_assert!(input.members.is_empty() || total_bytes >= input.members[0].size_bytes);
}

fn validate_required_roles(
    input: &BundleManifestInput,
    members: &BTreeMap<String, BundleMember>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let role_counts = role_counts(members.values());
    for role in REQUIRED_SINGLETON_ROLES {
        if role_counts.get(&role) != Some(&1_u32) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-or-duplicate-bundle-role",
                subject: &role_label(&role),
                message: "bundle requires exactly one member for this role",
            }));
        }
    }
    for role in [
        BundleRole::FixtureArtifact,
        BundleRole::FixtureDescriptor,
        BundleRole::CorpusArtifact,
        BundleRole::CorpusDescriptor,
        BundleRole::CheckReceipt,
    ] {
        if role_counts.get(&role).copied().unwrap_or(0) == 0 {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-bundle-role",
                subject: &role_label(&role),
                message: "bundle omits a required repeated role",
            }));
        }
    }
    debug_assert!(role_counts.len() <= input.members.len());
    debug_assert!(members.len() <= input.members.len());
}

fn validate_profile_bound_digests(
    input: &BundleManifestInput,
    members: &BTreeMap<String, BundleMember>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    require_role_digest(
        members,
        BundleRole::SourceArchive,
        &input.profile.source.archive_blake3,
        "source-archive-drift",
        diagnostics,
    );
    require_role_digest(
        members,
        BundleRole::CargoLock,
        &input.profile.source.cargo_lock_blake3,
        "cargo-lock-drift",
        diagnostics,
    );
    require_role_digest(
        members,
        BundleRole::DependencyManifest,
        &input.profile.source.dependency_manifest_blake3,
        "dependency-manifest-drift",
        diagnostics,
    );
    require_role_digest(
        members,
        BundleRole::SupportProjection,
        &input.profile.source.octet_support_projection_blake3,
        "support-projection-drift",
        diagnostics,
    );
    debug_assert!(!input.profile.source.archive_blake3.as_str().is_empty());
    debug_assert!(!input.profile.source.cargo_lock_blake3.as_str().is_empty());
}

fn require_role_digest(
    members: &BTreeMap<String, BundleMember>,
    role: BundleRole,
    expected: &Blake3Digest,
    code: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let matching: Vec<_> = members.values().filter(|member| member.role == role).collect();
    if matching.len() != 1 || matching.first().map(|member| &member.digest_blake3) != Some(expected) {
        diagnostics.push(error(ErrorDiagnostic {
            code,
            subject: &role_label(&role),
            message: "bundle member digest differs from the selected profile",
        }));
    }
    debug_assert!(matching.len() <= members.len());
    debug_assert!(!code.is_empty());
}

fn validate_fixture_and_corpus_members(
    input: &BundleManifestInput,
    members: &BTreeMap<String, BundleMember>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for fixture in &input.profile.fixtures {
        require_path_role(members, &fixture.artifact_path, BundleRole::FixtureArtifact, diagnostics);
        require_path_role(members, &fixture.descriptor_path, BundleRole::FixtureDescriptor, diagnostics);
        if members.get(&fixture.artifact_path).map(|member| &member.digest_blake3) != Some(&fixture.artifact_blake3) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "fixture-digest-mismatch",
                subject: &fixture.fixture_id,
                message: "fixture bytes differ from the profile",
            }));
        }
        if members.get(&fixture.descriptor_path).map(|member| &member.digest_blake3) != Some(&fixture.descriptor_blake3)
        {
            diagnostics.push(error(ErrorDiagnostic {
                code: "fixture-descriptor-drift",
                subject: &fixture.fixture_id,
                message: "fixture descriptor bytes differ from the profile",
            }));
        }
    }
    for corpus in &input.profile.corpora {
        require_path_role(members, &corpus.descriptor_path, BundleRole::CorpusDescriptor, diagnostics);
        if members.get(&corpus.descriptor_path).map(|member| &member.digest_blake3) != Some(&corpus.descriptor_blake3) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "corpus-drift",
                subject: &corpus.corpus_id,
                message: "corpus descriptor bytes differ from the profile",
            }));
        }
    }
    debug_assert!(input.profile.fixtures.is_empty() || !members.is_empty());
    debug_assert!(input.profile.corpora.is_empty() || !members.is_empty());
}

fn validate_legal_members(
    input: &BundleManifestInput,
    members: &BTreeMap<String, BundleMember>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for path in &input.profile.source.license_members {
        require_path_role(members, path, BundleRole::License, diagnostics);
    }
    for path in &input.profile.source.notice_members {
        require_path_role(members, path, BundleRole::Notice, diagnostics);
    }
    debug_assert!(!input.profile.source.license_members.is_empty());
    debug_assert!(!input.profile.source.notice_members.is_empty());
}

fn validate_parent_edges(
    input: &BundleManifestInput,
    members: &BTreeMap<String, BundleMember>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut edges = BTreeSet::new();
    for edge in &input.parent_edges {
        let has_declared_paths = members.contains_key(&edge.parent_path) && members.contains_key(&edge.child_path);
        let is_well_formed = has_declared_paths && edge.parent_path != edge.child_path && !edge.relation.is_empty();
        let is_unique = is_well_formed && edges.insert(edge.clone());
        if !is_unique {
            diagnostics.push(error(ErrorDiagnostic {
                code: "invalid-parent-edge",
                subject: &edge.child_path,
                message: "parent edges must be unique, non-circular, and refer to declared members",
            }));
        }
    }
    let children: BTreeSet<_> = input.parent_edges.iter().map(|edge| edge.child_path.as_str()).collect();
    for (path, member) in members {
        if !children.contains(path.as_str()) && !is_root_member(member) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-parent-edge",
                subject: path,
                message: "non-root bundle member has no declared parent edge",
            }));
        }
    }
    debug_assert!(edges.len() <= input.parent_edges.len());
    debug_assert!(children.len() <= input.parent_edges.len());
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<Diagnostic>) {
    let values: BTreeSet<_> = non_claims.iter().map(String::as_str).collect();
    for required in REQUIRED_NON_CLAIMS {
        if !values.contains(required) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-required-non-claim",
                subject: required,
                message: "bundle omits a required SpaceWasm claim boundary",
            }));
        }
    }
    if values.len() != non_claims.len() {
        diagnostics.push(error(ErrorDiagnostic {
            code: "duplicate-non-claim",
            subject: "bundle.non-claims",
            message: "bundle non-claims must be unique",
        }));
    }
    debug_assert!(values.len() <= non_claims.len());
    debug_assert!(non_claims.is_empty() || !values.is_empty());
}

fn validate_manifest_identity(manifest: &BundleManifest, diagnostics: &mut Vec<Diagnostic>) {
    let expected = manifest.bundle_identity_blake3.clone();
    let input = BundleIdentityInput {
        schema: manifest.schema.clone(),
        profile_id: manifest.profile_id.clone(),
        profile_identity_blake3: manifest.profile_identity_blake3.clone(),
        cohort_identity_blake3: manifest.cohort_identity_blake3.clone(),
        members: manifest.members.clone(),
        parent_edges: manifest.parent_edges.clone(),
        non_claims: manifest.non_claims.clone(),
    };
    if canonical_identity(input).as_ref() != Ok(&expected) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "bundle-manifest-tamper",
            subject: "bundle.manifest",
            message: "bundle manifest identity does not match canonical fields",
        }));
    }
    debug_assert!(!manifest.schema.is_empty());
    debug_assert!(!manifest.profile_id.is_empty());
}

fn normalize_bundle_input(input: &mut BundleManifestInput) {
    input.members.sort();
    input.parent_edges.sort();
    input.non_claims.sort();
    debug_assert!(input.members.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(input.parent_edges.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
}

fn member_map(
    members: &[BundleMember],
    subject: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, BundleMember> {
    let mut map = BTreeMap::new();
    for member in members {
        if !is_safe_relative_path(&member.path) || map.insert(member.path.clone(), member.clone()).is_some() {
            diagnostics.push(error(ErrorDiagnostic {
                code: "invalid-bundle-member-path",
                subject,
                message: "bundle member paths must be unique safe relative paths",
            }));
        }
    }
    debug_assert!(map.len() <= members.len());
    debug_assert!(members.is_empty() || !map.is_empty() || !diagnostics.is_empty());
    map
}

fn role_counts<'a>(members: impl Iterator<Item = &'a BundleMember>) -> BTreeMap<BundleRole, u32> {
    let mut counts = BTreeMap::new();
    for member in members {
        let count = counts.entry(member.role.clone()).or_insert(0_u32);
        *count = count.saturating_add(1);
    }
    debug_assert!(counts.values().all(|count| *count > 0));
    debug_assert!(counts.values().all(|count| *count < u32::MAX));
    counts
}

fn require_path_role(
    members: &BTreeMap<String, BundleMember>,
    path: &str,
    role: BundleRole,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if members.get(path).map(|member| &member.role) != Some(&role) {
        diagnostics.push(error(ErrorDiagnostic {
            code: "missing-or-wrong-role-member",
            subject: path,
            message: "bundle member is missing or has the wrong role",
        }));
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}

fn is_root_member(member: &BundleMember) -> bool {
    matches!(member.role, BundleRole::ProfileSource)
}

fn role_label(role: &BundleRole) -> String {
    serde_json::to_string(role).unwrap_or_else(|_| String::from("unknown-role"))
}

#[derive(Serialize)]
struct BundleIdentityInput {
    schema: String,
    profile_id: String,
    profile_identity_blake3: Blake3Digest,
    cohort_identity_blake3: Blake3Digest,
    members: Vec<BundleMember>,
    parent_edges: Vec<ParentEdge>,
    non_claims: Vec<String>,
}
