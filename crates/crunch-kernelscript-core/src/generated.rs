use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::ExperimentBlocker;
use crate::ExperimentProfile;
use crate::GeneratedFileClass;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::count_above_bound;

pub const GENERATED_PROJECT_MANIFEST_SCHEMA: &str = "mantle-kernelscript-generated-project-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedGeneratedFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedProjectFacts {
    pub profile: ExperimentProfile,
    pub files: Vec<ObservedGeneratedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedFileMember {
    pub relative_path: String,
    pub class: GeneratedFileClass,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedProjectManifest {
    pub schema: String,
    pub profile_identity_blake3: Blake3Digest,
    pub members: Vec<GeneratedFileMember>,
    pub total_bytes: u64,
    pub manifest_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedProjectResult {
    pub manifest: Option<GeneratedProjectManifest>,
    pub blockers: Vec<ExperimentBlocker>,
}

#[derive(Serialize)]
struct ManifestIdentityInput {
    schema: String,
    profile_identity_blake3: Blake3Digest,
    members: Vec<GeneratedFileMember>,
    total_bytes: u64,
}

pub fn classify_generated_project(facts: GeneratedProjectFacts) -> GeneratedProjectResult {
    let profile_validation = crate::validate_profile(facts.profile.clone());
    let mut blockers = profile_validation.blockers;
    let Some(profile_identity) = profile_validation.profile_identity_blake3 else {
        return GeneratedProjectResult {
            manifest: None,
            blockers,
        };
    };
    validate_file_count(&facts, &mut blockers);
    if count_above_bound(facts.files.len(), facts.profile.bounds.max_generated_files) {
        return GeneratedProjectResult {
            manifest: None,
            blockers,
        };
    }
    let (mut members, total_bytes) = classify_files(&facts, &mut blockers);
    validate_required_files(&facts, &members, &mut blockers);
    members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let manifest = build_manifest(profile_identity, members, total_bytes, &mut blockers);
    debug_assert!(blockers.is_empty() == manifest.is_some());
    debug_assert!(blockers.iter().all(|item| !item.code.is_empty()));
    GeneratedProjectResult { manifest, blockers }
}

fn validate_file_count(facts: &GeneratedProjectFacts, blockers: &mut Vec<ExperimentBlocker>) {
    if facts.files.is_empty() || count_above_bound(facts.files.len(), facts.profile.bounds.max_generated_files) {
        blockers.push(blocker(
            "generated-file-count",
            "generated-project",
            "generated project must contain a bounded non-empty exact file set",
        ));
    }
}

fn classify_files(
    facts: &GeneratedProjectFacts,
    blockers: &mut Vec<ExperimentBlocker>,
) -> (Vec<GeneratedFileMember>, u64) {
    let expected = expected_by_path(facts);
    let mut seen = BTreeSet::new();
    let mut members = Vec::with_capacity(facts.files.len());
    let mut total_bytes = 0_u64;
    for file in &facts.files {
        let Some(declaration) = expected.get(&file.relative_path) else {
            blockers.push(blocker(
                "unexpected-generated-file",
                &file.relative_path,
                "generated file is not in the exact profile allowlist",
            ));
            if !safe_relative_path(&file.relative_path) {
                blockers.push(blocker(
                    "generated-path-escape",
                    &file.relative_path,
                    "generated path is absolute, escaping, or non-canonical",
                ));
            }
            continue;
        };
        if !seen.insert(file.relative_path.clone()) {
            blockers.push(blocker(
                "duplicate-generated-file",
                &file.relative_path,
                "generated file path is duplicated",
            ));
            continue;
        }
        validate_observed_file(file, declaration.max_bytes, facts.profile.bounds.max_generated_file_bytes, blockers);
        let size_bytes = u64::try_from(file.bytes.len()).unwrap_or(u64::MAX);
        total_bytes = total_bytes.saturating_add(size_bytes);
        members.push(GeneratedFileMember {
            relative_path: file.relative_path.clone(),
            class: declaration.class,
            digest_blake3: Blake3Digest::from_slice(&file.bytes),
            size_bytes,
            executable: file.executable,
        });
    }
    if total_bytes > facts.profile.bounds.max_generated_total_bytes {
        blockers.push(blocker(
            "generated-total-byte-bound",
            "generated-project",
            "generated project exceeds the aggregate byte bound",
        ));
    }
    (members, total_bytes)
}

fn expected_by_path(facts: &GeneratedProjectFacts) -> BTreeMap<String, crate::ExpectedGeneratedFile> {
    facts
        .profile
        .expected_generated_files
        .iter()
        .cloned()
        .map(|expected| (expected.relative_path.clone(), expected))
        .collect()
}

fn validate_observed_file(
    file: &ObservedGeneratedFile,
    declared_max_bytes: u64,
    profile_max_bytes: u64,
    blockers: &mut Vec<ExperimentBlocker>,
) {
    if !safe_relative_path(&file.relative_path) {
        blockers.push(blocker(
            "generated-path-escape",
            &file.relative_path,
            "generated path is absolute, escaping, or non-canonical",
        ));
    }
    let size_bytes = u64::try_from(file.bytes.len()).unwrap_or(u64::MAX);
    if size_bytes == 0 || size_bytes > declared_max_bytes || size_bytes > profile_max_bytes {
        blockers.push(blocker(
            "generated-file-byte-bound",
            &file.relative_path,
            "generated file is empty or exceeds its byte bound",
        ));
    }
    if file.executable {
        blockers.push(blocker(
            "generated-file-executable",
            &file.relative_path,
            "generated project files must remain non-executable review inputs",
        ));
    }
}

fn validate_required_files(
    facts: &GeneratedProjectFacts,
    members: &[GeneratedFileMember],
    blockers: &mut Vec<ExperimentBlocker>,
) {
    let observed: BTreeSet<&str> = members.iter().map(|member| member.relative_path.as_str()).collect();
    for expected in &facts.profile.expected_generated_files {
        if expected.required && !observed.contains(expected.relative_path.as_str()) {
            blockers.push(blocker(
                "missing-generated-file",
                &expected.relative_path,
                "required generated file is absent",
            ));
        }
    }
}

fn build_manifest(
    profile_identity_blake3: Blake3Digest,
    members: Vec<GeneratedFileMember>,
    total_bytes: u64,
    blockers: &mut Vec<ExperimentBlocker>,
) -> Option<GeneratedProjectManifest> {
    if !blockers.is_empty() {
        return None;
    }
    let input = ManifestIdentityInput {
        schema: String::from(GENERATED_PROJECT_MANIFEST_SCHEMA),
        profile_identity_blake3: profile_identity_blake3.clone(),
        members: members.clone(),
        total_bytes,
    };
    let identity = match canonical_identity(&input) {
        Ok(identity) => identity,
        Err(error) => {
            blockers.push(blocker("generated-manifest-identity-failed", "generated-project", &error.to_string()));
            return None;
        }
    };
    debug_assert!(!members.is_empty());
    debug_assert!(total_bytes > 0);
    Some(GeneratedProjectManifest {
        schema: String::from(GENERATED_PROJECT_MANIFEST_SCHEMA),
        profile_identity_blake3,
        members,
        total_bytes,
        manifest_identity_blake3: identity,
    })
}

pub(crate) fn expected_generated_manifest_identity(manifest: &GeneratedProjectManifest) -> Option<Blake3Digest> {
    canonical_identity(&ManifestIdentityInput {
        schema: manifest.schema.clone(),
        profile_identity_blake3: manifest.profile_identity_blake3.clone(),
        members: manifest.members.clone(),
        total_bytes: manifest.total_bytes,
    })
    .ok()
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}
