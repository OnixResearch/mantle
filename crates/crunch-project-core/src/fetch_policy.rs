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
const MAX_DIAGNOSTICS_PER_INPUT: usize = 3;
const SOURCE_ID_FILE_PREFIX: &str = "file:";
const SOURCE_ID_TARBALL_PREFIX: &str = "tarball:";
const SOURCE_ID_GIT_PREFIX: &str = "git:";
const SOURCE_ID_DARCS_PREFIX: &str = "darcs:";
const SOURCE_ID_PIJUL_PREFIX: &str = "pijul:";
const SOURCE_ID_FOSSIL_PREFIX: &str = "fossil:";
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
    let mut diagnostics = Vec::with_capacity(request.manifest.inputs.len().saturating_mul(MAX_DIAGNOSTICS_PER_INPUT));

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

    let planned = PlannedInput {
        item: InputFetchPolicyItem {
            name: input.name.clone(),
            policy: input.fetch_policy,
            requirement,
            generated_input_mode: generated_input_mode(input, existing, requirement),
            source_state_blake3: source_fact.map(|fact| fact.source_state_blake3.clone()),
        },
        diagnostics,
    };
    debug_assert_eq!(planned.item.name, input.name);
    debug_assert!(planned.diagnostics.len() <= MAX_DIAGNOSTICS_PER_INPUT);
    planned
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
    assert!(!input.name.is_empty(), "input name must not be empty");
    let diagnostic_count_before = diagnostics.len();
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
    debug_assert!(diagnostics.len() >= diagnostic_count_before);
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
    assert!(!input.name.is_empty(), "input name must not be empty");
    if expected.is_empty() {
        return Err(diagnostic(
            &input.name,
            InputFetchDiagnosticKind::Conflicting,
            "expected hash for non-generation fetch policy must not be empty",
        ));
    }
    debug_assert!(!expected.is_empty());
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
        InputKind::Darcs { repository, selector } => locked_darcs_kind(repository, selector, existing, &input.name),
        InputKind::Pijul { repository, selector } => locked_pijul_kind(repository, selector, existing, &input.name),
        InputKind::Fossil { repository, selector } => locked_fossil_kind(repository, selector, existing, &input.name),
    }
}

fn locked_git_kind(
    repository: &str,
    reference: &GitReference,
    existing: Option<&LockEntry>,
    input_name: &str,
) -> Result<LockedKind, InputFetchDiagnostic> {
    let result = if let Some(LockEntry {
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
        Ok(LockedKind::Git {
            repository: repository.to_string(),
            rev: rev.clone(),
            ref_name: ref_name.clone(),
        })
    } else if let GitReference::Rev(rev) = reference {
        Ok(LockedKind::Git {
            repository: repository.to_string(),
            rev: rev.clone(),
            ref_name: None,
        })
    } else {
        Err(diagnostic(
            input_name,
            InputFetchDiagnosticKind::BuildFetchRequired,
            "git build-time or imported fetch policy requires an existing lock rev or explicit rev reference",
        ))
    };
    if result.is_ok() {
        debug_assert!(matches!(&result, Ok(LockedKind::Git { .. })));
    }
    if let Ok(LockedKind::Git { repository: locked, .. }) = &result {
        debug_assert_eq!(locked, repository);
    }
    result
}

fn locked_darcs_kind(
    repository: &str,
    selector: &crate::DarcsSelector,
    existing: Option<&LockEntry>,
    input_name: &str,
) -> Result<LockedKind, InputFetchDiagnostic> {
    let result = if let Some(LockEntry {
        kind:
            LockedKind::Darcs {
                repository: locked_repository,
                selector: locked_selector,
                context,
                weak_hash,
            },
        ..
    }) = existing
        && locked_repository == repository
        && locked_selector == selector
    {
        Ok(LockedKind::Darcs {
            repository: repository.to_string(),
            selector: selector.clone(),
            context: context.clone(),
            weak_hash: weak_hash.clone(),
        })
    } else if let crate::DarcsSelector::Context(context) = selector {
        Ok(LockedKind::Darcs {
            repository: repository.to_string(),
            selector: selector.clone(),
            context: Some(context.clone()),
            weak_hash: None,
        })
    } else {
        Err(diagnostic(
            input_name,
            InputFetchDiagnosticKind::BuildFetchRequired,
            "darcs build-time or imported fetch policy requires an existing lock context/weak-hash or an explicit context selector",
        ))
    };
    if result.is_ok() {
        debug_assert!(matches!(&result, Ok(LockedKind::Darcs { .. })));
    }
    if let Ok(LockedKind::Darcs { repository: locked, .. }) = &result {
        debug_assert_eq!(locked, repository);
    }
    result
}

fn locked_pijul_kind(
    repository: &str,
    selector: &crate::PijulSelector,
    existing: Option<&LockEntry>,
    input_name: &str,
) -> Result<LockedKind, InputFetchDiagnostic> {
    let result = if let Some(LockEntry {
        kind:
            LockedKind::Pijul {
                repository: locked_repository,
                selector: locked_selector,
                state,
                change,
            },
        ..
    }) = existing
        && locked_repository == repository
        && locked_selector == selector
    {
        Ok(LockedKind::Pijul {
            repository: repository.to_string(),
            selector: selector.clone(),
            state: state.clone(),
            change: change.clone(),
        })
    } else if let crate::PijulSelector::State { state, .. } = selector {
        Ok(LockedKind::Pijul {
            repository: repository.to_string(),
            selector: selector.clone(),
            state: state.clone(),
            change: None,
        })
    } else {
        Err(diagnostic(
            input_name,
            InputFetchDiagnosticKind::BuildFetchRequired,
            "pijul build-time or imported fetch policy requires an existing lock state or an explicit state selector",
        ))
    };
    if result.is_ok() {
        debug_assert!(matches!(&result, Ok(LockedKind::Pijul { .. })));
    }
    if let Ok(LockedKind::Pijul { repository: locked, .. }) = &result {
        debug_assert_eq!(locked, repository);
    }
    result
}

fn locked_fossil_kind(
    repository: &str,
    selector: &crate::FossilSelector,
    existing: Option<&LockEntry>,
    input_name: &str,
) -> Result<LockedKind, InputFetchDiagnostic> {
    let result = if let Some(LockEntry {
        kind:
            LockedKind::Fossil {
                repository: locked_repository,
                selector: locked_selector,
                checkin,
            },
        ..
    }) = existing
        && locked_repository == repository
        && locked_selector == selector
    {
        Ok(LockedKind::Fossil {
            repository: repository.to_string(),
            selector: selector.clone(),
            checkin: checkin.clone(),
        })
    } else if let crate::FossilSelector::Checkin(checkin) = selector {
        Ok(LockedKind::Fossil {
            repository: repository.to_string(),
            selector: selector.clone(),
            checkin: checkin.clone(),
        })
    } else {
        Err(diagnostic(
            input_name,
            InputFetchDiagnosticKind::BuildFetchRequired,
            "fossil build-time or imported fetch policy requires an existing lock check-in or an explicit check-in selector",
        ))
    };
    if result.is_ok() {
        debug_assert!(matches!(&result, Ok(LockedKind::Fossil { .. })));
    }
    if let Ok(LockedKind::Fossil { repository: locked, .. }) = &result {
        debug_assert_eq!(locked, repository);
    }
    result
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
        LockedKind::Darcs {
            repository,
            context,
            weak_hash,
            ..
        } => source_identity(SOURCE_ID_DARCS_PREFIX, &darcs_locked_identity(repository, context, weak_hash)),
        LockedKind::Pijul {
            repository,
            state,
            change,
            ..
        } => source_identity(SOURCE_ID_PIJUL_PREFIX, &pijul_locked_identity(repository, state, change)),
        LockedKind::Fossil {
            repository, checkin, ..
        } => source_identity(SOURCE_ID_FOSSIL_PREFIX, &fossil_identity(repository, checkin)),
    }
}

fn source_identity_from_input_kind(kind: &InputKind) -> String {
    match kind {
        InputKind::File { url } => source_identity(SOURCE_ID_FILE_PREFIX, url),
        InputKind::Tarball { url } => source_identity(SOURCE_ID_TARBALL_PREFIX, url),
        InputKind::Git { repository, reference } => {
            source_identity(SOURCE_ID_GIT_PREFIX, &git_reference_identity(repository, reference))
        }
        InputKind::Darcs { repository, selector } => {
            source_identity(SOURCE_ID_DARCS_PREFIX, &vcs_selector_identity(repository, &selector.identity_fragment()))
        }
        InputKind::Pijul { repository, selector } => {
            source_identity(SOURCE_ID_PIJUL_PREFIX, &vcs_selector_identity(repository, &selector.identity_fragment()))
        }
        InputKind::Fossil { repository, selector } => {
            source_identity(SOURCE_ID_FOSSIL_PREFIX, &vcs_selector_identity(repository, &selector.identity_fragment()))
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

fn git_identity(repository: &str, revision_or_ref: impl AsRef<str>) -> String {
    let revision_or_ref = revision_or_ref.as_ref();
    format!("{repository}{SOURCE_ID_GIT_REF_SEPARATOR}{revision_or_ref}")
}

fn darcs_locked_identity(repository: &str, context: &Option<String>, weak_hash: &Option<String>) -> String {
    if let Some(context) = context {
        return vcs_selector_identity(repository, &format!("context:{context}"));
    }
    if let Some(weak_hash) = weak_hash {
        return vcs_selector_identity(repository, &format!("weak-hash:{weak_hash}"));
    }
    vcs_selector_identity(repository, "unresolved")
}

fn pijul_locked_identity(repository: &str, state: impl AsRef<str>, change: &Option<String>) -> String {
    let state = state.as_ref();
    if let Some(change) = change {
        return vcs_selector_identity(repository, format!("state:{state}:change:{change}"));
    }
    vcs_selector_identity(repository, format!("state:{state}"))
}

fn fossil_identity(repository: &str, checkin: impl AsRef<str>) -> String {
    vcs_selector_identity(repository, format!("checkin:{}", checkin.as_ref()))
}

fn vcs_selector_identity(repository: &str, selector: impl AsRef<str>) -> String {
    let selector = selector.as_ref();
    format!("{repository}{SOURCE_ID_GIT_REF_SEPARATOR}{selector}")
}

fn source_identity(prefix: impl AsRef<str>, value: &str) -> String {
    let prefix = prefix.as_ref();
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
    let is_duplicate = diagnostics
        .iter()
        .any(|existing| existing.input_name == diagnostic.input_name && existing.kind == diagnostic.kind);
    if !is_duplicate {
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

    #[test]
    fn vcs_source_identity_uses_locked_native_identity() {
        let input = ManifestInput {
            name: "src".into(),
            kind: InputKind::Darcs {
                repository: "https://example.invalid/repo".into(),
                selector: crate::DarcsSelector::Tag("v1".into()),
            },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: Vec::new(),
            patches: Vec::new(),
            fetch_policy: InputFetchPolicy::ImportedSourceRequired,
            retention: None,
            freshness: None,
            trust: None,
        };
        let entry = LockEntry {
            kind: LockedKind::Darcs {
                repository: "https://example.invalid/repo".into(),
                selector: crate::DarcsSelector::Tag("v1".into()),
                context: Some("ctx".into()),
                weak_hash: None,
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: HASH_A.into(),
            },
            patches: Vec::new(),
            mirrors: Vec::new(),
            fetch_policy: InputFetchPolicy::ImportedSourceRequired,
            freshness: None,
            trust: None,
        };

        let identity = input_source_identity(&input, Some(&entry));

        assert_eq!(identity, "darcs:https://example.invalid/repo#context:ctx");
    }

    #[test]
    fn non_generation_vcs_policy_requires_proven_identity_for_ambiguous_selector() {
        let input = ManifestInput {
            name: "src".into(),
            kind: InputKind::Fossil {
                repository: "https://example.invalid/repo".into(),
                selector: crate::FossilSelector::Branch("trunk".into()),
            },
            hash: HashSpec {
                algo: HashAlgo::Sha256,
                expected: Some(HASH_A.into()),
            },
            frozen: false,
            mirrors: Vec::new(),
            patches: Vec::new(),
            fetch_policy: InputFetchPolicy::BuildFetchAction,
            retention: None,
            freshness: None,
            trust: None,
        };

        let err = lock_entry_without_fetch(&input, None).expect_err("branch selector needs resolved check-in");

        assert_eq!(err.kind, InputFetchDiagnosticKind::BuildFetchRequired);
        assert!(err.message.contains("existing lock check-in"));
    }
}
