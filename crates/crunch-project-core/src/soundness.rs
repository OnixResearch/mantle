use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use serde::Serialize;

use crate::drift::DriftStatus;
use crate::drift::check_drift;
use crate::generate::generate_inputs_ncl;
use crate::lock::LockEntry;
use crate::lock::LockedKind;
use crate::lock::LockedPatch;
use crate::lock::LockedPatchSource;
use crate::lock::Lockfile;
use crate::manifest::GitReference;
use crate::manifest::InputKind;
use crate::manifest::ManifestInput;
use crate::manifest::PatchDef;
use crate::manifest::PatchSource;
use crate::manifest::ProjectManifest;
use crate::version::SchemaVersion;
use crate::version::parse_version;

const SOUNDNESS_SCHEMA: &str = "mantle-project-soundness-v1";
const STATIC_MODE_NAME: &str = "static";
const STATIC_NETWORK_BEHAVIOR: &str = "no-network";
const STATIC_PROCESS_BEHAVIOR: &str = "no-probe-commands";
const PROBE_MODE_NAME: &str = "static-with-probes-requested";
const TRUST_MODE_NAME: &str = "static-with-trust-requested";
const PROBE_TRUST_MODE_NAME: &str = "static-with-probes-and-trust-requested";
const PROBE_NETWORK_BEHAVIOR: &str = "freshness-probes-may-contact-network";
const TRUST_NETWORK_BEHAVIOR: &str = "trust-verification-may-contact-network";
const PROBE_TRUST_NETWORK_BEHAVIOR: &str = "freshness-probes-or-trust-verification-may-contact-network";
const PROBE_PROCESS_BEHAVIOR: &str = "freshness-probes-may-run-commands";
const TRUST_PROCESS_BEHAVIOR: &str = "trust-verification-may-run-commands";
const PROBE_TRUST_PROCESS_BEHAVIOR: &str = "freshness-probes-or-trust-verification-may-run-commands";
const MAX_SOUNDNESS_ISSUES: u32 = 512;
const GENERATED_INPUTS_PATH: &str = ".mantle/inputs.ncl";
const NON_CLAIM_BUILD_SUCCESS: &str = "static project soundness does not prove build success";
const NON_CLAIM_SOURCE_AVAILABILITY: &str =
    "static project soundness does not prove remote source availability or upstream freshness";
const NON_CLAIM_INPUT_TRUST: &str =
    "static project soundness does not prove input trust, release reproducibility, or clean VCS state";

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSoundnessInput {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub generated_inputs: Option<String>,
    pub supplemental_facts: Vec<ProjectSoundnessFact>,
    pub mode: ProjectSoundnessMode,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectSoundnessReport {
    pub schema: String,
    pub mode: ProjectSoundnessMode,
    pub valid: bool,
    pub issue_count: u32,
    pub highest_severity: Option<ProjectSoundnessSeverity>,
    pub issues: Vec<ProjectSoundnessIssue>,
    pub non_claims: Vec<String>,
}

impl ProjectSoundnessReport {
    pub fn from_issues(issues: Vec<ProjectSoundnessIssue>) -> Self {
        Self::from_issues_with_mode(issues, ProjectSoundnessMode::static_no_network())
    }

    pub fn from_issues_with_mode(mut issues: Vec<ProjectSoundnessIssue>, mode: ProjectSoundnessMode) -> Self {
        sort_issues(&mut issues);
        let highest_severity = highest_severity(&issues);
        let valid = !issues.iter().any(ProjectSoundnessIssue::is_error);
        let issue_count = issue_count_u32(&issues);
        Self {
            schema: SOUNDNESS_SCHEMA.into(),
            mode,
            valid,
            issue_count,
            highest_severity,
            issues,
            non_claims: default_non_claims(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSoundnessMode {
    pub name: String,
    pub network_behavior: String,
    pub process_behavior: String,
}

impl ProjectSoundnessMode {
    pub fn static_no_network() -> Self {
        Self {
            name: STATIC_MODE_NAME.into(),
            network_behavior: STATIC_NETWORK_BEHAVIOR.into(),
            process_behavior: STATIC_PROCESS_BEHAVIOR.into(),
        }
    }

    pub fn from_dynamic_requests(probes: bool, trust: bool) -> Self {
        match (probes, trust) {
            (false, false) => Self::static_no_network(),
            (true, false) => Self {
                name: PROBE_MODE_NAME.into(),
                network_behavior: PROBE_NETWORK_BEHAVIOR.into(),
                process_behavior: PROBE_PROCESS_BEHAVIOR.into(),
            },
            (false, true) => Self {
                name: TRUST_MODE_NAME.into(),
                network_behavior: TRUST_NETWORK_BEHAVIOR.into(),
                process_behavior: TRUST_PROCESS_BEHAVIOR.into(),
            },
            (true, true) => Self {
                name: PROBE_TRUST_MODE_NAME.into(),
                network_behavior: PROBE_TRUST_NETWORK_BEHAVIOR.into(),
                process_behavior: PROBE_TRUST_PROCESS_BEHAVIOR.into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSoundnessFact {
    pub class: ProjectSoundnessClass,
    pub severity: ProjectSoundnessSeverity,
    pub subject: ProjectSoundnessSubject,
    pub message: String,
    pub evidence_refs: Vec<String>,
}

impl ProjectSoundnessFact {
    pub fn error(class: ProjectSoundnessClass, subject: ProjectSoundnessSubject, message: String) -> Self {
        Self::new(class, ProjectSoundnessSeverity::Error, subject, message)
    }

    pub fn warning(class: ProjectSoundnessClass, subject: ProjectSoundnessSubject, message: String) -> Self {
        Self::new(class, ProjectSoundnessSeverity::Warning, subject, message)
    }

    fn new(
        class: ProjectSoundnessClass,
        severity: ProjectSoundnessSeverity,
        subject: ProjectSoundnessSubject,
        message: String,
    ) -> Self {
        Self {
            class,
            severity,
            subject,
            message,
            evidence_refs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSoundnessIssue {
    pub class: ProjectSoundnessClass,
    pub severity: ProjectSoundnessSeverity,
    pub subject: ProjectSoundnessSubject,
    pub message: String,
    pub evidence_refs: Vec<String>,
    pub fix_guidance: Option<String>,
}

impl ProjectSoundnessIssue {
    pub fn error(class: ProjectSoundnessClass, subject: ProjectSoundnessSubject, message: String) -> Self {
        Self::new(class, ProjectSoundnessSeverity::Error, subject, message, None)
    }

    pub fn warning(class: ProjectSoundnessClass, subject: ProjectSoundnessSubject, message: String) -> Self {
        Self::new(class, ProjectSoundnessSeverity::Warning, subject, message, None)
    }

    pub fn with_fix(mut self, fix_guidance: String) -> Self {
        self.fix_guidance = Some(fix_guidance);
        self
    }

    fn new(
        class: ProjectSoundnessClass,
        severity: ProjectSoundnessSeverity,
        subject: ProjectSoundnessSubject,
        message: String,
        fix_guidance: Option<String>,
    ) -> Self {
        Self {
            class,
            severity,
            subject,
            message,
            evidence_refs: Vec::new(),
            fix_guidance,
        }
    }

    fn is_error(&self) -> bool {
        self.severity == ProjectSoundnessSeverity::Error
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectSoundnessSeverity {
    Error,
    Warning,
}

impl ProjectSoundnessSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }

    fn rank(self) -> u32 {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectSoundnessClass {
    ManifestParseError,
    LockfileParseError,
    ManifestVersionUnsupported,
    LockfileVersionUnsupported,
    ManifestValidation,
    LockfileValidation,
    LockEntryMissing,
    LockEntryOrphaned,
    InputKindMismatch,
    SourceIdentityMismatch,
    HashAlgorithmMismatch,
    HashValueMismatch,
    PatchSetMismatch,
    PatchDefinitionMismatch,
    MirrorSetMismatch,
    FetchPolicyConflict,
    FreshnessProbeFailure,
    TrustPolicyFailure,
    RetentionRootMismatch,
    GeneratedInputMissing,
    GeneratedInputStale,
}

impl ProjectSoundnessClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManifestParseError => "manifest-parse-error",
            Self::LockfileParseError => "lockfile-parse-error",
            Self::ManifestVersionUnsupported => "manifest-version-unsupported",
            Self::LockfileVersionUnsupported => "lockfile-version-unsupported",
            Self::ManifestValidation => "manifest-validation",
            Self::LockfileValidation => "lockfile-validation",
            Self::LockEntryMissing => "lock-entry-missing",
            Self::LockEntryOrphaned => "lock-entry-orphaned",
            Self::InputKindMismatch => "input-kind-mismatch",
            Self::SourceIdentityMismatch => "source-identity-mismatch",
            Self::HashAlgorithmMismatch => "hash-algorithm-mismatch",
            Self::HashValueMismatch => "hash-value-mismatch",
            Self::PatchSetMismatch => "patch-set-mismatch",
            Self::PatchDefinitionMismatch => "patch-definition-mismatch",
            Self::MirrorSetMismatch => "mirror-set-mismatch",
            Self::FetchPolicyConflict => "fetch-policy-conflict",
            Self::FreshnessProbeFailure => "freshness-probe-failure",
            Self::TrustPolicyFailure => "trust-policy-failure",
            Self::RetentionRootMismatch => "retention-root-mismatch",
            Self::GeneratedInputMissing => "generated-input-missing",
            Self::GeneratedInputStale => "generated-input-stale",
        }
    }

    fn rank(self) -> u32 {
        match self {
            Self::ManifestParseError => 10,
            Self::LockfileParseError => 20,
            Self::ManifestVersionUnsupported => 30,
            Self::LockfileVersionUnsupported => 40,
            Self::ManifestValidation => 50,
            Self::LockfileValidation => 60,
            Self::LockEntryMissing => 70,
            Self::LockEntryOrphaned => 80,
            Self::InputKindMismatch => 90,
            Self::SourceIdentityMismatch => 100,
            Self::HashAlgorithmMismatch => 110,
            Self::HashValueMismatch => 120,
            Self::PatchSetMismatch => 130,
            Self::PatchDefinitionMismatch => 140,
            Self::MirrorSetMismatch => 150,
            Self::FetchPolicyConflict => 160,
            Self::FreshnessProbeFailure => 170,
            Self::TrustPolicyFailure => 180,
            Self::RetentionRootMismatch => 190,
            Self::GeneratedInputMissing => 200,
            Self::GeneratedInputStale => 210,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSoundnessSubject {
    pub kind: String,
    pub name: String,
}

impl ProjectSoundnessSubject {
    pub fn file(name: impl Into<String>) -> Self {
        Self {
            kind: "file".into(),
            name: name.into(),
        }
    }

    pub fn input(name: impl Into<String>) -> Self {
        Self {
            kind: "input".into(),
            name: name.into(),
        }
    }

    pub fn patch(name: impl Into<String>) -> Self {
        Self {
            kind: "patch".into(),
            name: name.into(),
        }
    }

    pub fn project() -> Self {
        Self {
            kind: "project".into(),
            name: "project".into(),
        }
    }
}

pub fn check_project_soundness(input: ProjectSoundnessInput) -> ProjectSoundnessReport {
    let mut issues = Vec::new();
    push_manifest_version_issue(&input.manifest, &mut issues);
    push_lock_version_issue(&input.lock, &mut issues);
    push_manifest_validation_issues(input.manifest.clone(), &mut issues);
    push_lock_validation_issues(input.lock.clone(), &mut issues);
    push_manifest_lock_issues(&input.manifest, &input.lock, &mut issues);
    push_patch_definition_issues(&input.manifest, &input.lock, &mut issues);
    push_supplemental_fact_issues(input.supplemental_facts, &mut issues);
    push_generated_input_issue(input.lock, input.generated_inputs, &mut issues);
    ProjectSoundnessReport::from_issues_with_mode(issues, input.mode)
}

pub fn project_soundness_parse_error(
    class: ProjectSoundnessClass,
    subject: ProjectSoundnessSubject,
    message: String,
) -> ProjectSoundnessReport {
    debug_assert!(
        matches!(class, ProjectSoundnessClass::ManifestParseError | ProjectSoundnessClass::LockfileParseError),
        "parse helper must use a parse-error class"
    );
    ProjectSoundnessReport::from_issues(vec![ProjectSoundnessIssue::error(class, subject, message)])
}

fn push_manifest_version_issue(manifest: &ProjectManifest, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) {
        return;
    }
    match parse_version(manifest.version.clone()) {
        Some(version) if version.is_compatible_with(SchemaVersion::CURRENT) => {}
        Some(version) => issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::ManifestVersionUnsupported,
            ProjectSoundnessSubject::file("mantle-project.ncl"),
            format!("manifest version {version} is not compatible with current version {}", SchemaVersion::CURRENT),
        )),
        None => issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::ManifestVersionUnsupported,
            ProjectSoundnessSubject::file("mantle-project.ncl"),
            format!("manifest version '{}' is invalid", manifest.version),
        )),
    }
}

fn push_lock_version_issue(lock: &Lockfile, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) {
        return;
    }
    if lock.version.is_compatible_with(SchemaVersion::CURRENT) {
        return;
    }
    issues.push(ProjectSoundnessIssue::error(
        ProjectSoundnessClass::LockfileVersionUnsupported,
        ProjectSoundnessSubject::file("mantle.lock"),
        format!(
            "lockfile version {} is not compatible with current version {}",
            lock.version,
            SchemaVersion::CURRENT
        ),
    ));
}

fn push_manifest_validation_issues(manifest: ProjectManifest, issues: &mut Vec<ProjectSoundnessIssue>) {
    for problem in manifest.validate() {
        if issues_at_limit(issues) {
            break;
        }
        issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::ManifestValidation,
            ProjectSoundnessSubject::file("mantle-project.ncl"),
            problem,
        ));
    }
}

fn push_lock_validation_issues(lock: Lockfile, issues: &mut Vec<ProjectSoundnessIssue>) {
    for problem in lock.validate() {
        if issues_at_limit(issues) {
            break;
        }
        issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::LockfileValidation,
            ProjectSoundnessSubject::file("mantle.lock"),
            problem,
        ));
    }
}

fn push_manifest_lock_issues(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<ProjectSoundnessIssue>) {
    let manifest_names = manifest_input_names(manifest);
    push_missing_lock_entries(manifest, lock, issues);
    push_orphaned_lock_entries(lock, &manifest_names, issues);
    for input in &manifest.inputs {
        if issues_at_limit(issues) {
            break;
        }
        let Some(entry) = lock.inputs.get(&input.name) else {
            continue;
        };
        push_input_entry_issues(input, entry, issues);
    }
}

fn push_missing_lock_entries(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<ProjectSoundnessIssue>) {
    for input in &manifest.inputs {
        if issues_at_limit(issues) {
            break;
        }
        if lock.inputs.contains_key(&input.name) {
            continue;
        }
        issues.push(
            ProjectSoundnessIssue::error(
                ProjectSoundnessClass::LockEntryMissing,
                ProjectSoundnessSubject::input(input.name.clone()),
                format!("input '{}' is in the manifest but has no lock entry", input.name),
            )
            .with_fix("run `mantle refresh` for this input or remove the manifest entry".into()),
        );
    }
}

fn push_orphaned_lock_entries(
    lock: &Lockfile,
    manifest_names: &BTreeSet<&str>,
    issues: &mut Vec<ProjectSoundnessIssue>,
) {
    for name in lock.inputs.keys() {
        if issues_at_limit(issues) {
            break;
        }
        if manifest_names.contains(name.as_str()) {
            continue;
        }
        issues.push(ProjectSoundnessIssue::warning(
            ProjectSoundnessClass::LockEntryOrphaned,
            ProjectSoundnessSubject::input(name.clone()),
            format!("lock entry '{name}' has no corresponding manifest input"),
        ));
    }
}

fn push_input_entry_issues(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if !same_kind(&input.kind, &entry.kind) {
        push_kind_mismatch(input, entry, issues);
        return;
    }
    push_source_identity_mismatch(input, entry, issues);
    push_hash_mismatches(input, entry, issues);
    push_patch_set_mismatch(input, entry, issues);
    push_mirror_set_mismatch(input, entry, issues);
}

fn push_kind_mismatch(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) {
        return;
    }
    issues.push(ProjectSoundnessIssue::error(
        ProjectSoundnessClass::InputKindMismatch,
        ProjectSoundnessSubject::input(input.name.clone()),
        format!(
            "input '{}': manifest kind {} does not match lock kind {}",
            input.name,
            input_kind_label(&input.kind),
            locked_kind_label(&entry.kind)
        ),
    ));
}

fn push_source_identity_mismatch(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) {
        return;
    }
    if source_identity_matches(&input.kind, &entry.kind) {
        return;
    }
    issues.push(ProjectSoundnessIssue::error(
        ProjectSoundnessClass::SourceIdentityMismatch,
        ProjectSoundnessSubject::input(input.name.clone()),
        format!(
            "input '{}': manifest source {} does not match lock source {}",
            input.name,
            input_source_label(&input.kind),
            lock_source_label(&entry.kind)
        ),
    ));
}

fn push_hash_mismatches(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) {
        return;
    }
    if input.hash.algo != entry.hash.algo {
        issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::HashAlgorithmMismatch,
            ProjectSoundnessSubject::input(input.name.clone()),
            format!(
                "input '{}': manifest hash algorithm {} does not match lock hash algorithm {}",
                input.name, input.hash.algo, entry.hash.algo
            ),
        ));
    }
    if let Some(expected) = &input.hash.expected
        && expected != &entry.hash.value
        && !issues_at_limit(issues)
    {
        issues.push(ProjectSoundnessIssue::error(
            ProjectSoundnessClass::HashValueMismatch,
            ProjectSoundnessSubject::input(input.name.clone()),
            format!("input '{}': manifest expected hash does not match lock hash", input.name),
        ));
    }
}

fn push_patch_set_mismatch(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) || input.patches == entry.patches {
        return;
    }
    issues.push(ProjectSoundnessIssue::warning(
        ProjectSoundnessClass::PatchSetMismatch,
        ProjectSoundnessSubject::input(input.name.clone()),
        format!(
            "input '{}': manifest patches {:?} differ from lock patches {:?}",
            input.name, input.patches, entry.patches
        ),
    ));
}

fn push_mirror_set_mismatch(input: &ManifestInput, entry: &LockEntry, issues: &mut Vec<ProjectSoundnessIssue>) {
    if issues_at_limit(issues) || input.mirrors == entry.mirrors {
        return;
    }
    issues.push(ProjectSoundnessIssue::warning(
        ProjectSoundnessClass::MirrorSetMismatch,
        ProjectSoundnessSubject::input(input.name.clone()),
        format!(
            "input '{}': manifest mirrors {:?} differ from lock mirrors {:?}",
            input.name, input.mirrors, entry.mirrors
        ),
    ));
}

fn push_patch_definition_issues(manifest: &ProjectManifest, lock: &Lockfile, issues: &mut Vec<ProjectSoundnessIssue>) {
    for patch in &manifest.patches {
        if issues_at_limit(issues) {
            break;
        }
        let Some(locked) = lock.patches.get(&patch.name) else {
            continue;
        };
        if patch_source_matches(&patch.source, &locked.source) {
            continue;
        }
        issues.push(ProjectSoundnessIssue::warning(
            ProjectSoundnessClass::PatchDefinitionMismatch,
            ProjectSoundnessSubject::patch(patch.name.clone()),
            format!(
                "patch '{}': manifest source {} differs from lock source {}",
                patch.name,
                patch_source_label(&patch.source),
                locked_patch_source_label(&locked.source)
            ),
        ));
    }
}

fn push_supplemental_fact_issues(
    supplemental_facts: Vec<ProjectSoundnessFact>,
    issues: &mut Vec<ProjectSoundnessIssue>,
) {
    for fact in supplemental_facts {
        if issues_at_limit(issues) {
            break;
        }
        let mut issue = ProjectSoundnessIssue::new(fact.class, fact.severity, fact.subject, fact.message, None);
        issue.evidence_refs = fact.evidence_refs;
        issues.push(issue);
    }
}

fn push_generated_input_issue(
    lock: Lockfile,
    generated_inputs: Option<String>,
    issues: &mut Vec<ProjectSoundnessIssue>,
) {
    if issues_at_limit(issues) {
        return;
    }
    match check_drift(lock, generated_inputs) {
        DriftStatus::InSync => {}
        DriftStatus::Missing => issues.push(
            ProjectSoundnessIssue::error(
                ProjectSoundnessClass::GeneratedInputMissing,
                ProjectSoundnessSubject::file(GENERATED_INPUTS_PATH),
                format!("generated inputs drift: {GENERATED_INPUTS_PATH} does not exist"),
            )
            .with_fix("run `mantle refresh` or `mantle init` to regenerate inputs".into()),
        ),
        DriftStatus::Drifted {
            expected_fingerprint,
            actual_fingerprint,
        } => issues.push(
            ProjectSoundnessIssue::error(
                ProjectSoundnessClass::GeneratedInputStale,
                ProjectSoundnessSubject::file(GENERATED_INPUTS_PATH),
                format!(
                    "generated inputs drift: {GENERATED_INPUTS_PATH} is out of date (expected fingerprint {expected_fingerprint}, actual fingerprint {actual_fingerprint})"
                ),
            )
            .with_fix("run `mantle refresh` to rewrite generated inputs from the lockfile".into()),
        ),
    }
}

fn manifest_input_names(manifest: &ProjectManifest) -> BTreeSet<&str> {
    manifest.inputs.iter().map(|input| input.name.as_str()).collect()
}

fn same_kind(manifest: &InputKind, locked: &LockedKind) -> bool {
    matches!(
        (manifest, locked),
        (InputKind::File { .. }, LockedKind::File { .. })
            | (InputKind::Tarball { .. }, LockedKind::Tarball { .. })
            | (InputKind::Git { .. }, LockedKind::Git { .. })
    )
}

fn source_identity_matches(manifest: &InputKind, locked: &LockedKind) -> bool {
    match (manifest, locked) {
        (InputKind::File { url: left }, LockedKind::File { url: right }) => left == right,
        (InputKind::Tarball { url: left }, LockedKind::Tarball { url: right }) => left == right,
        (
            InputKind::Git {
                repository: manifest_repo,
                reference,
            },
            LockedKind::Git {
                repository: lock_repo,
                rev,
                ref_name,
            },
        ) => manifest_repo == lock_repo && git_reference_matches(reference, rev, ref_name),
        _ => false,
    }
}

fn git_reference_matches(reference: &GitReference, rev: &str, ref_name: &Option<String>) -> bool {
    match reference {
        GitReference::Rev(expected_rev) => expected_rev == rev,
        GitReference::Branch(branch) | GitReference::Tag(branch) => ref_name.as_deref() == Some(branch.as_str()),
    }
}

fn patch_source_matches(manifest: &PatchSource, locked: &LockedPatchSource) -> bool {
    match (manifest, locked) {
        (PatchSource::Local { path: left }, LockedPatchSource::Local { path: right }) => left == right,
        (PatchSource::Remote { url: left, .. }, LockedPatchSource::Remote { url: right }) => left == right,
        _ => false,
    }
}

fn input_kind_label(kind: &InputKind) -> &'static str {
    match kind {
        InputKind::File { .. } => "file",
        InputKind::Tarball { .. } => "tarball",
        InputKind::Git { .. } => "git",
    }
}

fn locked_kind_label(kind: &LockedKind) -> &'static str {
    match kind {
        LockedKind::File { .. } => "file",
        LockedKind::Tarball { .. } => "tarball",
        LockedKind::Git { .. } => "git",
    }
}

fn input_source_label(kind: &InputKind) -> String {
    match kind {
        InputKind::File { url } => format!("file:{url}"),
        InputKind::Tarball { url } => format!("tarball:{url}"),
        InputKind::Git { repository, reference } => format!("git:{repository}@{}", git_reference_label(reference)),
    }
}

fn lock_source_label(kind: &LockedKind) -> String {
    match kind {
        LockedKind::File { url } => format!("file:{url}"),
        LockedKind::Tarball { url } => format!("tarball:{url}"),
        LockedKind::Git {
            repository,
            rev,
            ref_name,
        } => format!("git:{repository}@{}:{rev}", ref_name.clone().unwrap_or_else(|| "<detached>".into())),
    }
}

fn git_reference_label(reference: &GitReference) -> String {
    match reference {
        GitReference::Branch(branch) => format!("branch:{branch}"),
        GitReference::Tag(tag) => format!("tag:{tag}"),
        GitReference::Rev(rev) => format!("rev:{rev}"),
    }
}

fn patch_source_label(source: &PatchSource) -> String {
    match source {
        PatchSource::Local { path } => format!("local:{path}"),
        PatchSource::Remote { url, .. } => format!("remote:{url}"),
    }
}

fn locked_patch_source_label(source: &LockedPatchSource) -> String {
    match source {
        LockedPatchSource::Local { path } => format!("local:{path}"),
        LockedPatchSource::Remote { url } => format!("remote:{url}"),
    }
}

fn issues_at_limit(issues: &[ProjectSoundnessIssue]) -> bool {
    issues.len() as u64 >= MAX_SOUNDNESS_ISSUES as u64
}

fn issue_count_u32(issues: &[ProjectSoundnessIssue]) -> u32 {
    debug_assert!(issues.len() as u64 <= MAX_SOUNDNESS_ISSUES as u64, "issue count exceeds limit");
    issues.len() as u32
}

fn highest_severity(issues: &[ProjectSoundnessIssue]) -> Option<ProjectSoundnessSeverity> {
    if issues.iter().any(ProjectSoundnessIssue::is_error) {
        Some(ProjectSoundnessSeverity::Error)
    } else if issues.is_empty() {
        None
    } else {
        Some(ProjectSoundnessSeverity::Warning)
    }
}

fn sort_issues(issues: &mut [ProjectSoundnessIssue]) {
    issues.sort_by(compare_issues);
}

fn compare_issues(left: &ProjectSoundnessIssue, right: &ProjectSoundnessIssue) -> Ordering {
    left.severity
        .rank()
        .cmp(&right.severity.rank())
        .then_with(|| left.class.rank().cmp(&right.class.rank()))
        .then_with(|| left.subject.kind.cmp(&right.subject.kind))
        .then_with(|| left.subject.name.cmp(&right.subject.name))
        .then_with(|| left.message.cmp(&right.message))
}

fn default_non_claims() -> Vec<String> {
    vec![
        NON_CLAIM_BUILD_SUCCESS.into(),
        NON_CLAIM_SOURCE_AVAILABILITY.into(),
        NON_CLAIM_INPUT_TRUST.into(),
    ]
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::HashAlgo;
    use crate::HashSpec;
    use crate::LockedHash;

    const SAMPLE_HASH: &str = "sha256-abc123=";
    const OTHER_HASH: &str = "sha256-other=";

    fn clean_manifest() -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg.txt".into(),
                },
                hash: HashSpec {
                    algo: HashAlgo::Sha256,
                    expected: Some(SAMPLE_HASH.into()),
                },
                frozen: false,
                mirrors: vec!["https://mirror.example.com/pkg.txt".into()],
                patches: vec!["fix".into()],
            }],
            patches: vec![PatchDef {
                name: "fix".into(),
                source: PatchSource::Local {
                    path: "patches/fix.patch".into(),
                },
            }],
        }
    }

    fn clean_lock() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert("pkg".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/pkg.txt".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: SAMPLE_HASH.into(),
            },
            patches: vec!["fix".into()],
            mirrors: vec!["https://mirror.example.com/pkg.txt".into()],
        });
        let mut patches = BTreeMap::new();
        patches.insert("fix".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "patches/fix.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-patch=".into(),
            },
        });
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches,
        }
    }

    fn soundness_input(
        manifest: ProjectManifest,
        lock: Lockfile,
        generated_inputs: Option<String>,
    ) -> ProjectSoundnessInput {
        ProjectSoundnessInput {
            manifest,
            lock,
            generated_inputs,
            supplemental_facts: Vec::new(),
            mode: ProjectSoundnessMode::static_no_network(),
        }
    }

    fn report_for(manifest: ProjectManifest, lock: Lockfile) -> ProjectSoundnessReport {
        let generated_inputs = generate_inputs_ncl(lock.clone());
        check_project_soundness(soundness_input(manifest, lock, Some(generated_inputs)))
    }

    fn classes(report: &ProjectSoundnessReport) -> Vec<ProjectSoundnessClass> {
        report.issues.iter().map(|issue| issue.class).collect()
    }

    #[test]
    fn clean_project_state_is_valid_static_no_network() {
        let report = report_for(clean_manifest(), clean_lock());
        assert!(report.valid);
        assert_eq!(report.issue_count, 0);
        assert_eq!(report.mode.network_behavior, STATIC_NETWORK_BEHAVIOR);
        assert!(report.non_claims.iter().any(|claim| claim.contains("does not prove build success")));
    }

    #[test]
    fn warning_only_orphaned_lock_entry_keeps_report_valid() {
        let manifest = clean_manifest();
        let mut lock = clean_lock();
        lock.inputs.insert("stale".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/stale".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: SAMPLE_HASH.into(),
            },
            patches: vec![],
            mirrors: vec![],
        });
        let generated_inputs = generate_inputs_ncl(lock.clone());

        let report = check_project_soundness(soundness_input(manifest, lock, Some(generated_inputs)));

        assert!(report.valid);
        assert_eq!(report.highest_severity, Some(ProjectSoundnessSeverity::Warning));
        assert_eq!(classes(&report), vec![ProjectSoundnessClass::LockEntryOrphaned]);
    }

    #[test]
    fn deterministic_issue_order_does_not_follow_manifest_order() {
        let mut manifest = clean_manifest();
        manifest.inputs.push(ManifestInput {
            name: "aaa".into(),
            kind: InputKind::Tarball {
                url: "https://example.com/archive.tar.gz".into(),
            },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
        });
        let report = report_for(manifest, clean_lock());

        assert!(!report.valid);
        assert_eq!(report.issues[0].subject.name, "aaa");
        assert_eq!(report.issues[0].class, ProjectSoundnessClass::LockEntryMissing);
    }

    #[test]
    fn detects_missing_lock_entry() {
        let manifest = clean_manifest();
        let lock = Lockfile::new();
        let generated_inputs = generate_inputs_ncl(lock.clone());

        let report = check_project_soundness(soundness_input(manifest, lock, Some(generated_inputs)));

        assert!(!report.valid);
        assert!(classes(&report).contains(&ProjectSoundnessClass::LockEntryMissing));
        assert_eq!(report.highest_severity, Some(ProjectSoundnessSeverity::Error));
    }

    #[test]
    fn detects_kind_and_source_mismatches() {
        let manifest = clean_manifest();
        let mut lock = clean_lock();
        lock.inputs.get_mut("pkg").unwrap().kind = LockedKind::Git {
            repository: "https://example.com/other.git".into(),
            rev: "abc".into(),
            ref_name: Some("main".into()),
        };
        let report = report_for(manifest, lock);

        assert!(!report.valid);
        assert_eq!(classes(&report), vec![ProjectSoundnessClass::InputKindMismatch]);
    }

    #[test]
    fn detects_same_kind_source_identity_mismatch() {
        let manifest = clean_manifest();
        let mut lock = clean_lock();
        lock.inputs.get_mut("pkg").unwrap().kind = LockedKind::File {
            url: "https://example.com/other.txt".into(),
        };
        let report = report_for(manifest, lock);

        assert!(!report.valid);
        assert!(classes(&report).contains(&ProjectSoundnessClass::SourceIdentityMismatch));
    }

    #[test]
    fn detects_hash_algorithm_and_expected_hash_mismatches() {
        let manifest = clean_manifest();
        let mut lock = clean_lock();
        let entry = lock.inputs.get_mut("pkg").unwrap();
        entry.hash.algo = HashAlgo::Blake3;
        entry.hash.value = OTHER_HASH.into();
        let report = report_for(manifest, lock);

        assert!(!report.valid);
        assert!(classes(&report).contains(&ProjectSoundnessClass::HashAlgorithmMismatch));
        assert!(classes(&report).contains(&ProjectSoundnessClass::HashValueMismatch));
    }

    #[test]
    fn detects_patch_mirror_and_patch_definition_mismatches_as_warnings() {
        let manifest = clean_manifest();
        let mut lock = clean_lock();
        lock.inputs.get_mut("pkg").unwrap().patches = vec![];
        lock.inputs.get_mut("pkg").unwrap().mirrors = vec![];
        lock.patches.get_mut("fix").unwrap().source = LockedPatchSource::Local {
            path: "patches/old.patch".into(),
        };
        let report = report_for(manifest, lock);

        assert!(report.valid);
        assert_eq!(report.highest_severity, Some(ProjectSoundnessSeverity::Warning));
        assert!(classes(&report).contains(&ProjectSoundnessClass::PatchSetMismatch));
        assert!(classes(&report).contains(&ProjectSoundnessClass::MirrorSetMismatch));
        assert!(classes(&report).contains(&ProjectSoundnessClass::PatchDefinitionMismatch));
    }

    #[test]
    fn detects_generated_input_missing_and_stale() {
        let manifest = clean_manifest();
        let lock = clean_lock();
        let missing = check_project_soundness(soundness_input(manifest.clone(), lock.clone(), None));
        let stale = check_project_soundness(soundness_input(manifest, lock, Some("stale".to_string())));

        assert!(!missing.valid);
        assert!(classes(&missing).contains(&ProjectSoundnessClass::GeneratedInputMissing));
        assert!(!stale.valid);
        assert!(classes(&stale).contains(&ProjectSoundnessClass::GeneratedInputStale));
    }

    #[test]
    fn supplemental_policy_trust_retention_and_freshness_facts_are_classified() {
        let manifest = clean_manifest();
        let lock = clean_lock();
        let generated_inputs = generate_inputs_ncl(lock.clone());
        let mut input = soundness_input(manifest, lock, Some(generated_inputs));
        input.supplemental_facts = vec![
            ProjectSoundnessFact::error(
                ProjectSoundnessClass::FetchPolicyConflict,
                ProjectSoundnessSubject::input("pkg"),
                "offline mode requires imported source state".into(),
            ),
            ProjectSoundnessFact::error(
                ProjectSoundnessClass::TrustPolicyFailure,
                ProjectSoundnessSubject::input("pkg"),
                "signature quorum was not met".into(),
            ),
            ProjectSoundnessFact::warning(
                ProjectSoundnessClass::RetentionRootMismatch,
                ProjectSoundnessSubject::input("pkg"),
                "retention root points at an older lock generation".into(),
            ),
            ProjectSoundnessFact::warning(
                ProjectSoundnessClass::FreshnessProbeFailure,
                ProjectSoundnessSubject::input("pkg"),
                "freshness probe failed in explicit probe mode".into(),
            ),
        ];

        let report = check_project_soundness(input);

        assert!(!report.valid);
        assert!(classes(&report).contains(&ProjectSoundnessClass::FetchPolicyConflict));
        assert!(classes(&report).contains(&ProjectSoundnessClass::TrustPolicyFailure));
        assert!(classes(&report).contains(&ProjectSoundnessClass::RetentionRootMismatch));
        assert!(classes(&report).contains(&ProjectSoundnessClass::FreshnessProbeFailure));
    }

    #[test]
    fn explicit_probe_and_trust_modes_label_possible_side_effects() {
        let probe = ProjectSoundnessMode::from_dynamic_requests(true, false);
        let trust = ProjectSoundnessMode::from_dynamic_requests(false, true);
        let both = ProjectSoundnessMode::from_dynamic_requests(true, true);

        assert_eq!(probe.name, PROBE_MODE_NAME);
        assert_eq!(probe.network_behavior, PROBE_NETWORK_BEHAVIOR);
        assert_eq!(probe.process_behavior, PROBE_PROCESS_BEHAVIOR);
        assert_eq!(trust.name, TRUST_MODE_NAME);
        assert_eq!(trust.network_behavior, TRUST_NETWORK_BEHAVIOR);
        assert_eq!(both.name, PROBE_TRUST_MODE_NAME);
        assert_eq!(both.process_behavior, PROBE_TRUST_PROCESS_BEHAVIOR);
    }

    #[test]
    fn parse_error_report_is_invalid_and_stable() {
        let report = project_soundness_parse_error(
            ProjectSoundnessClass::ManifestParseError,
            ProjectSoundnessSubject::file("mantle-project.ncl"),
            "loading failed".into(),
        );

        assert!(!report.valid);
        assert_eq!(report.issue_count, 1);
        assert_eq!(report.issues[0].class.as_str(), "manifest-parse-error");
    }
}
