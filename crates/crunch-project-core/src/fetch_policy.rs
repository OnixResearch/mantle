use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::GitReference;
use crate::InputKind;
use crate::LockEntry;
use crate::LockedHash;
use crate::LockedKind;
use crate::Lockfile;
use crate::ManifestInput;
use crate::ProjectManifest;

pub const FETCH_POLICY_NON_CLAIM: &str =
    "input fetch policy classifies source acquisition timing; it does not prove source availability or build success";
const MAX_SOURCE_STATE_FACTS: u32 = 4096;
const SOURCE_ID_FILE_PREFIX: &str = "file:";
const SOURCE_ID_TARBALL_PREFIX: &str = "tarball:";
const SOURCE_ID_GIT_PREFIX: &str = "git:";
const SOURCE_ID_GIT_REF_SEPARATOR: &str = "#";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum InputFetchPolicy {
    #[default]
    GenerationMaterial,
    BuildFetchAction,
    ImportedSourceRequired,
}

impl InputFetchPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenerationMaterial => "generation-material",
            Self::BuildFetchAction => "build-fetch-action",
            Self::ImportedSourceRequired => "imported-source-required",
        }
    }

    pub fn needs_generation_resolution(self) -> bool {
        matches!(self, Self::GenerationMaterial)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputFetchRequirement {
    AlreadyPresent,
    GenerationMaterial,
    BuildFetchAction,
    ImportedSourceRequired,
    Unsupported,
    Conflicting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedInputMode {
    RenderLockedFetch,
    RenderBuildFetchAction,
    RequiresImportedSourceState,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputFetchDiagnosticKind {
    NetworkRequired,
    SourceStateRequired,
    BuildFetchRequired,
    Unsupported,
    Conflicting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputSourceStateClass {
    Ready,
    Missing,
    Stale,
    Unsupported,
    Untrusted,
    NetworkRequired,
    Unpinned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSourceStateFact {
    pub input_name: String,
    pub hash_value: String,
    pub identity: String,
    pub source_state_blake3: String,
    pub ready_class: InputSourceStateClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputFetchDiagnostic {
    pub input_name: String,
    pub kind: InputFetchDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputFetchPolicyItem {
    pub name: String,
    pub policy: InputFetchPolicy,
    pub requirement: InputFetchRequirement,
    pub generated_input_mode: GeneratedInputMode,
    pub source_state_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputFetchPolicyPlanRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub source_state: Vec<InputSourceStateFact>,
    pub offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputFetchPolicyReport {
    pub items: Vec<InputFetchPolicyItem>,
    pub diagnostics: Vec<InputFetchDiagnostic>,
    pub non_claim: &'static str,
}

// r[impl project_workflows.input_fetch_policy]
// r[impl project_workflows.input_fetch_policy_preflight]
pub fn plan_input_fetch_policies(request: InputFetchPolicyPlanRequest) -> InputFetchPolicyReport {
    assert!(request.manifest.inputs.len() as u64 <= crate::MAX_INPUTS as u64, "manifest input limit exceeded");
    assert!(
        request.source_state.len() as u64 <= MAX_SOURCE_STATE_FACTS as u64,
        "source-state fact limit exceeded"
    );
    let source_state = source_state_map(&request.source_state);
    let mut items = Vec::with_capacity(request.manifest.inputs.len());
    let mut diagnostics = Vec::new();

    for input in &request.manifest.inputs {
        let planned = plan_one_input(input, &request.lock, &source_state, request.offline);
        items.push(planned.item);
        diagnostics.extend(planned.diagnostics);
    }

    InputFetchPolicyReport {
        items,
        diagnostics,
        non_claim: FETCH_POLICY_NON_CLAIM,
    }
}

pub fn fetch_policy_compatibility_problems(input: &ManifestInput) -> Vec<String> {
    let mut problems = Vec::new();
    if input.name.is_empty() {
        return problems;
    }
    if input.fetch_policy == InputFetchPolicy::BuildFetchAction && !input.patches.is_empty() {
        problems.push(format!(
            "input '{}': fetch_policy build-fetch-action cannot apply project patches before consumption",
            input.name
        ));
    }
    problems
}

pub fn lock_entry_without_fetch(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
) -> Result<LockEntry, InputFetchDiagnostic> {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        !matches!(input.fetch_policy, InputFetchPolicy::GenerationMaterial),
        "generation-material inputs require resolver-backed refresh"
    );
    if let Some(expected) = input.hash.expected.as_deref() {
        return lock_entry_from_expected(input, existing, expected);
    }
    let Some(existing) = existing else {
        return Err(diagnostic(
            &input.name,
            InputFetchDiagnosticKind::BuildFetchRequired,
            "fetch policy requires an existing lock entry or explicit expected hash",
        ));
    };
    Ok(lock_entry_from_existing_policy(input, existing))
}

struct PlannedInput {
    item: InputFetchPolicyItem,
    diagnostics: Vec<InputFetchDiagnostic>,
}

fn plan_one_input(
    input: &ManifestInput,
    lock: &Lockfile,
    source_state: &BTreeMap<&str, Vec<&InputSourceStateFact>>,
    offline: bool,
) -> PlannedInput {
    assert!(!input.name.is_empty(), "input name must not be empty");
    let existing = lock.inputs.get(&input.name);
    let source_fact = matching_source_fact(input, existing, source_state);
    let mut diagnostics = compatibility_diagnostics(input);
    add_requirement_diagnostics(input, existing, source_fact, &mut diagnostics);
    let mut requirement = requirement_for_input(input, existing, source_fact);

    if offline {
        add_offline_diagnostics(input, source_fact, &mut diagnostics);
        if source_fact.is_none() && requirement != InputFetchRequirement::Conflicting {
            requirement = offline_requirement(input.fetch_policy);
        }
    }
    if !diagnostics.is_empty() && requirement != InputFetchRequirement::Unsupported {
        requirement = InputFetchRequirement::Conflicting;
    }

    PlannedInput {
        item: InputFetchPolicyItem {
            name: input.name.clone(),
            policy: input.fetch_policy,
            requirement,
            generated_input_mode: generated_input_mode(input, existing, requirement),
            source_state_blake3: source_fact.map(|fact| fact.source_state_blake3.clone()),
        },
        diagnostics,
    }
}

fn compatibility_diagnostics(input: &ManifestInput) -> Vec<InputFetchDiagnostic> {
    fetch_policy_compatibility_problems(input)
        .into_iter()
        .map(|message| diagnostic(&input.name, InputFetchDiagnosticKind::Conflicting, &message))
        .collect()
}

fn requirement_for_input(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
    source_fact: Option<&InputSourceStateFact>,
) -> InputFetchRequirement {
    if source_fact.is_some() {
        return InputFetchRequirement::AlreadyPresent;
    }
    if existing.is_some() && input.fetch_policy == InputFetchPolicy::GenerationMaterial {
        return InputFetchRequirement::AlreadyPresent;
    }
    match input.fetch_policy {
        InputFetchPolicy::GenerationMaterial => InputFetchRequirement::GenerationMaterial,
        InputFetchPolicy::BuildFetchAction => InputFetchRequirement::BuildFetchAction,
        InputFetchPolicy::ImportedSourceRequired => InputFetchRequirement::ImportedSourceRequired,
    }
}

fn add_requirement_diagnostics(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
    source_fact: Option<&InputSourceStateFact>,
    diagnostics: &mut Vec<InputFetchDiagnostic>,
) {
    match input.fetch_policy {
        InputFetchPolicy::BuildFetchAction if existing.is_none() && input.hash.expected.is_none() => {
            diagnostics.push(diagnostic(
                &input.name,
                InputFetchDiagnosticKind::BuildFetchRequired,
                "policy build-fetch-action requires an existing lock entry or explicit expected hash",
            ))
        }
        InputFetchPolicy::ImportedSourceRequired if source_fact.is_none() => diagnostics.push(diagnostic(
            &input.name,
            InputFetchDiagnosticKind::SourceStateRequired,
            "policy imported-source-required has no matching ready source-state record (required source-state class: ready)",
        )),
        _ => {}
    }
}

fn add_offline_diagnostics(
    input: &ManifestInput,
    source_fact: Option<&InputSourceStateFact>,
    diagnostics: &mut Vec<InputFetchDiagnostic>,
) {
    if source_fact.is_some() {
        return;
    }
    match input.fetch_policy {
        InputFetchPolicy::GenerationMaterial | InputFetchPolicy::BuildFetchAction => diagnostics.push(diagnostic(
            &input.name,
            InputFetchDiagnosticKind::NetworkRequired,
            &format!(
                "offline preflight for policy {} requires imported source-state class ready before network-backed acquisition",
                input.fetch_policy.as_str()
            ),
        )),
        InputFetchPolicy::ImportedSourceRequired => push_diagnostic_once(
            diagnostics,
            diagnostic(
                &input.name,
                InputFetchDiagnosticKind::SourceStateRequired,
                "policy imported-source-required has no matching ready source-state record (required source-state class: ready)",
            ),
        ),
    }
}

fn offline_requirement(policy: InputFetchPolicy) -> InputFetchRequirement {
    match policy {
        InputFetchPolicy::GenerationMaterial => InputFetchRequirement::GenerationMaterial,
        InputFetchPolicy::BuildFetchAction => InputFetchRequirement::BuildFetchAction,
        InputFetchPolicy::ImportedSourceRequired => InputFetchRequirement::ImportedSourceRequired,
    }
}

fn generated_input_mode(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
    requirement: InputFetchRequirement,
) -> GeneratedInputMode {
    if matches!(requirement, InputFetchRequirement::Conflicting | InputFetchRequirement::Unsupported) {
        return GeneratedInputMode::Blocked;
    }
    match input.fetch_policy {
        InputFetchPolicy::GenerationMaterial => GeneratedInputMode::RenderLockedFetch,
        InputFetchPolicy::BuildFetchAction if existing.is_some() || input.hash.expected.is_some() => {
            GeneratedInputMode::RenderBuildFetchAction
        }
        InputFetchPolicy::BuildFetchAction => GeneratedInputMode::Blocked,
        InputFetchPolicy::ImportedSourceRequired => GeneratedInputMode::RequiresImportedSourceState,
    }
}

fn lock_entry_from_expected(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
    expected: &str,
) -> Result<LockEntry, InputFetchDiagnostic> {
    if expected.is_empty() {
        return Err(diagnostic(
            &input.name,
            InputFetchDiagnosticKind::Conflicting,
            "expected hash for non-generation fetch policy must not be empty",
        ));
    }
    let kind = locked_kind_from_manifest(input, existing)?;
    Ok(LockEntry {
        kind,
        hash: LockedHash {
            algo: input.hash.algo.clone(),
            value: expected.to_string(),
        },
        patches: input.patches.clone(),
        mirrors: input.mirrors.clone(),
        fetch_policy: input.fetch_policy,
        freshness: None,
        trust: None,
    })
}

fn lock_entry_from_existing_policy(input: &ManifestInput, existing: &LockEntry) -> LockEntry {
    let mut entry = existing.clone();
    entry.patches = input.patches.clone();
    entry.mirrors = input.mirrors.clone();
    entry.fetch_policy = input.fetch_policy;
    entry
}

fn locked_kind_from_manifest(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
) -> Result<LockedKind, InputFetchDiagnostic> {
    match &input.kind {
        InputKind::File { url } => Ok(LockedKind::File { url: url.clone() }),
        InputKind::Tarball { url } => Ok(LockedKind::Tarball { url: url.clone() }),
        InputKind::Git { repository, reference } => locked_git_kind(repository, reference, existing, &input.name),
    }
}

fn locked_git_kind(
    repository: &str,
    reference: &GitReference,
    existing: Option<&LockEntry>,
    input_name: &str,
) -> Result<LockedKind, InputFetchDiagnostic> {
    if let Some(LockEntry {
        kind:
            LockedKind::Git {
                repository: locked_repository,
                rev,
                ref_name,
            },
        ..
    }) = existing
        && locked_repository == repository
    {
        return Ok(LockedKind::Git {
            repository: repository.to_string(),
            rev: rev.clone(),
            ref_name: ref_name.clone(),
        });
    }
    if let GitReference::Rev(rev) = reference {
        return Ok(LockedKind::Git {
            repository: repository.to_string(),
            rev: rev.clone(),
            ref_name: None,
        });
    }
    Err(diagnostic(
        input_name,
        InputFetchDiagnosticKind::BuildFetchRequired,
        "git build-time or imported fetch policy requires an existing lock rev or explicit rev reference",
    ))
}

fn matching_source_fact<'a>(
    input: &ManifestInput,
    existing: Option<&LockEntry>,
    source_state: &BTreeMap<&str, Vec<&'a InputSourceStateFact>>,
) -> Option<&'a InputSourceStateFact> {
    let candidates = source_state.get(input.name.as_str())?;
    candidates.iter().copied().find(|fact| source_fact_matches(input, existing, fact))
}

fn source_fact_matches(input: &ManifestInput, existing: Option<&LockEntry>, fact: &InputSourceStateFact) -> bool {
    if fact.ready_class != InputSourceStateClass::Ready {
        return false;
    }
    if fact.input_name != input.name {
        return false;
    }
    if fact.identity != input_source_identity(input, existing) {
        return false;
    }
    if let Some(existing) = existing
        && fact.hash_value == existing.hash.value
    {
        return true;
    }
    input.hash.expected.as_deref() == Some(fact.hash_value.as_str())
}

pub fn input_source_identity(input: &ManifestInput, existing: Option<&LockEntry>) -> String {
    if let Some(existing) = existing {
        return source_identity_from_lock_kind(&existing.kind);
    }
    source_identity_from_input_kind(&input.kind)
}

fn source_identity_from_lock_kind(kind: &LockedKind) -> String {
    match kind {
        LockedKind::File { url } => source_identity(SOURCE_ID_FILE_PREFIX, url),
        LockedKind::Tarball { url } => source_identity(SOURCE_ID_TARBALL_PREFIX, url),
        LockedKind::Git { repository, rev, .. } => {
            source_identity(SOURCE_ID_GIT_PREFIX, &git_identity(repository, rev))
        }
    }
}

fn source_identity_from_input_kind(kind: &InputKind) -> String {
    match kind {
        InputKind::File { url } => source_identity(SOURCE_ID_FILE_PREFIX, url),
        InputKind::Tarball { url } => source_identity(SOURCE_ID_TARBALL_PREFIX, url),
        InputKind::Git { repository, reference } => {
            source_identity(SOURCE_ID_GIT_PREFIX, &git_reference_identity(repository, reference))
        }
    }
}

fn git_reference_identity(repository: &str, reference: &GitReference) -> String {
    match reference {
        GitReference::Rev(rev) => git_identity(repository, rev),
        GitReference::Branch(branch) => git_identity(repository, branch),
        GitReference::Tag(tag) => git_identity(repository, tag),
    }
}

fn git_identity(repository: &str, revision_or_ref: &str) -> String {
    format!("{repository}{SOURCE_ID_GIT_REF_SEPARATOR}{revision_or_ref}")
}

fn source_identity(prefix: &str, value: &str) -> String {
    format!("{prefix}{value}")
}

fn source_state_map(facts: &[InputSourceStateFact]) -> BTreeMap<&str, Vec<&InputSourceStateFact>> {
    let mut map: BTreeMap<&str, Vec<&InputSourceStateFact>> = BTreeMap::new();
    for fact in facts {
        map.entry(fact.input_name.as_str()).or_default().push(fact);
    }
    map
}

fn push_diagnostic_once(diagnostics: &mut Vec<InputFetchDiagnostic>, diagnostic: InputFetchDiagnostic) {
    let duplicate = diagnostics
        .iter()
        .any(|existing| existing.input_name == diagnostic.input_name && existing.kind == diagnostic.kind);
    if !duplicate {
        diagnostics.push(diagnostic);
    }
}

fn diagnostic(input_name: &str, kind: InputFetchDiagnosticKind, message: &str) -> InputFetchDiagnostic {
    InputFetchDiagnostic {
        input_name: input_name.to_string(),
        kind,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::format;
    use alloc::vec;

    use super::*;
    use crate::HashAlgo;
    use crate::HashSpec;
    use crate::SchemaVersion;

    const HASH_A: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const HASH_B: &str = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
    const SOURCE_STATE_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn file_input(name: &str, policy: InputFetchPolicy) -> ManifestInput {
        ManifestInput {
            name: name.to_string(),
            kind: InputKind::File {
                url: format!("https://example.com/{name}.tar.gz"),
            },
            hash: HashSpec {
                algo: HashAlgo::Sha256,
                expected: Some(HASH_A.to_string()),
            },
            frozen: false,
            mirrors: Vec::new(),
            patches: Vec::new(),
            fetch_policy: policy,
            retention: None,
            freshness: None,
            trust: None,
        }
    }

    fn manifest(inputs: Vec<ManifestInput>) -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".to_string(),
            inputs,
            patches: Vec::new(),
            retention: crate::InputRetentionPolicy::Untracked,
        }
    }

    fn lock_with(name: &str, policy: InputFetchPolicy, hash: &str) -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert(name.to_string(), LockEntry {
            kind: LockedKind::File {
                url: format!("https://example.com/{name}.tar.gz"),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: hash.to_string(),
            },
            patches: Vec::new(),
            mirrors: Vec::new(),
            fetch_policy: policy,
            freshness: None,
            trust: None,
        });
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches: BTreeMap::new(),
        }
    }

    fn source_fact(name: &str, hash: &str, ready_class: InputSourceStateClass) -> InputSourceStateFact {
        InputSourceStateFact {
            input_name: name.to_string(),
            hash_value: hash.to_string(),
            identity: source_identity(SOURCE_ID_FILE_PREFIX, &format!("https://example.com/{name}.tar.gz")),
            source_state_blake3: SOURCE_STATE_DIGEST.to_string(),
            ready_class,
        }
    }

    // r[verify project_workflows.input_fetch_policy]
    #[test]
    fn policy_plan_classifies_defaults_over_lock_without_fetching() {
        let report = plan_input_fetch_policies(InputFetchPolicyPlanRequest {
            manifest: manifest(vec![file_input("pkg", InputFetchPolicy::GenerationMaterial)]),
            lock: lock_with("pkg", InputFetchPolicy::GenerationMaterial, HASH_A),
            source_state: Vec::new(),
            offline: false,
        });

        assert!(report.diagnostics.is_empty(), "diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].requirement, InputFetchRequirement::AlreadyPresent);
        assert_eq!(report.items[0].generated_input_mode, GeneratedInputMode::RenderLockedFetch);
        assert_eq!(report.non_claim, FETCH_POLICY_NON_CLAIM);
    }

    // r[verify project_workflows.input_fetch_policy]
    #[test]
    fn build_fetch_policy_lowers_expected_hash_without_resolution() {
        let input = file_input("pkg", InputFetchPolicy::BuildFetchAction);
        let entry = lock_entry_without_fetch(&input, None).expect("expected hash should lock build fetch");

        assert_eq!(entry.hash.value, HASH_A);
        assert_eq!(entry.fetch_policy, InputFetchPolicy::BuildFetchAction);
        assert!(entry.patches.is_empty());
    }

    // r[verify project_workflows.input_fetch_policy_preflight]
    #[test]
    fn imported_source_policy_binds_ready_source_state_digest() {
        let report = plan_input_fetch_policies(InputFetchPolicyPlanRequest {
            manifest: manifest(vec![file_input("pkg", InputFetchPolicy::ImportedSourceRequired)]),
            lock: lock_with("pkg", InputFetchPolicy::ImportedSourceRequired, HASH_A),
            source_state: vec![source_fact("pkg", HASH_A, InputSourceStateClass::Ready)],
            offline: true,
        });

        assert!(report.diagnostics.is_empty(), "diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.items[0].requirement, InputFetchRequirement::AlreadyPresent);
        assert_eq!(report.items[0].source_state_blake3.as_deref(), Some(SOURCE_STATE_DIGEST));
        assert_eq!(report.items[0].generated_input_mode, GeneratedInputMode::RequiresImportedSourceState);
    }

    // r[verify project_workflows.input_fetch_policy]
    #[test]
    fn build_fetch_policy_rejects_patched_inputs_until_patch_lowering_exists() {
        let mut input = file_input("pkg", InputFetchPolicy::BuildFetchAction);
        input.patches = vec!["fix".to_string()];

        let problems = fetch_policy_compatibility_problems(&input);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("cannot apply project patches"));
    }

    // r[verify project_workflows.input_fetch_policy]
    #[test]
    fn missing_expected_hash_is_conflicting_for_build_fetch_without_lock() {
        let mut input = file_input("pkg", InputFetchPolicy::BuildFetchAction);
        input.hash.expected = None;

        let err = lock_entry_without_fetch(&input, None).expect_err("missing hash should fail closed");

        assert_eq!(err.kind, InputFetchDiagnosticKind::BuildFetchRequired);
        assert_eq!(err.input_name, "pkg");
        assert!(err.message.contains("existing lock entry or explicit expected hash"));
    }

    // r[verify project_workflows.input_fetch_policy_preflight]
    #[test]
    fn offline_network_policy_reports_network_required_without_source_state() {
        let report = plan_input_fetch_policies(InputFetchPolicyPlanRequest {
            manifest: manifest(vec![file_input("pkg", InputFetchPolicy::BuildFetchAction)]),
            lock: Lockfile::new(),
            source_state: Vec::new(),
            offline: true,
        });
        let classes = report.diagnostics.iter().map(|diag| diag.kind).collect::<Vec<_>>();

        assert_eq!(report.items[0].requirement, InputFetchRequirement::Conflicting);
        assert!(classes.contains(&InputFetchDiagnosticKind::NetworkRequired));
        assert!(report.diagnostics.iter().any(|diag| diag.message.contains("offline preflight")));
    }

    // r[verify project_workflows.input_fetch_policy_preflight]
    #[test]
    fn stale_source_state_does_not_satisfy_imported_policy() {
        let report = plan_input_fetch_policies(InputFetchPolicyPlanRequest {
            manifest: manifest(vec![file_input("pkg", InputFetchPolicy::ImportedSourceRequired)]),
            lock: lock_with("pkg", InputFetchPolicy::ImportedSourceRequired, HASH_A),
            source_state: vec![source_fact("pkg", HASH_B, InputSourceStateClass::Stale)],
            offline: true,
        });
        let diagnostics = report.diagnostics.iter().map(|diag| diag.kind).collect::<Vec<_>>();

        assert_eq!(report.items[0].requirement, InputFetchRequirement::Conflicting);
        assert!(diagnostics.contains(&InputFetchDiagnosticKind::SourceStateRequired));
        assert!(report.items[0].source_state_blake3.is_none());
    }

    // r[verify project_workflows.input_fetch_policy_preflight]
    #[test]
    fn mismatched_source_identity_does_not_satisfy_imported_policy() {
        let mut fact = source_fact("pkg", HASH_A, InputSourceStateClass::Ready);
        fact.identity = source_identity(SOURCE_ID_FILE_PREFIX, "https://example.com/other.tar.gz");
        let report = plan_input_fetch_policies(InputFetchPolicyPlanRequest {
            manifest: manifest(vec![file_input("pkg", InputFetchPolicy::ImportedSourceRequired)]),
            lock: lock_with("pkg", InputFetchPolicy::ImportedSourceRequired, HASH_A),
            source_state: vec![fact],
            offline: true,
        });
        let diagnostics = report.diagnostics.iter().map(|diag| diag.kind).collect::<Vec<_>>();

        assert_eq!(report.items[0].requirement, InputFetchRequirement::Conflicting);
        assert!(diagnostics.contains(&InputFetchDiagnosticKind::SourceStateRequired));
        assert!(report.items[0].source_state_blake3.is_none());
    }
}
