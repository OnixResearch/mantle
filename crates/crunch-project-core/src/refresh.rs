use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::FreshnessDecision;
use crate::FreshnessDecisionKind;
use crate::InputFetchPolicy;
use crate::InputSourceStateFact;
use crate::LockEntry;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::MAX_INPUTS;
use crate::MAX_LOCKED_PATCHES;
use crate::ManifestInput;
use crate::PatchDef;
use crate::PatchSource;
use crate::ProjectManifest;
use crate::TrustSubject;
use crate::VerifiedTrustFact;
use crate::evaluate_trust_policy;
use crate::fetch_policy_compatibility_problems;
use crate::lock_entry_without_fetch;
use crate::trust_policy_matches_locked;
use crate::validate_locked_trust;

const MAX_REFRESH_BATCH: u32 = 256;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedInput {
    pub name: String,
    pub entry: LockEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashResolutionMode {
    Flat,
    Recursive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshFailure {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleReport {
    pub stale: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<RefreshFailure>,
    pub network_required: Vec<RefreshFailure>,
    pub failed: Vec<RefreshFailure>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefreshOutcome {
    Updated(ResolvedInput),
    Unchanged { name: String },
    Frozen { name: String },
    Skipped { name: String, reason: String },
    NetworkRequired { name: String, reason: String },
    Failed { name: String, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedInputState {
    Resolved(ResolvedInput),
    Failed(RefreshFailure),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefreshInputsPlanRequest {
    pub manifest: ProjectManifest,
    pub selected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefreshInputsRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub selected: Vec<String>,
    pub resolutions: Vec<ResolvedInputState>,
    pub source_state: Vec<InputSourceStateFact>,
    pub freshness_decisions: Vec<FreshnessDecision>,
    pub trust_facts: Vec<VerifiedTrustFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchResolutionPlanRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub outcomes: Vec<RefreshOutcome>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchResolution {
    Resolved { name: String, patch: LockedPatch },
    Failed { name: String, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyOutcomesRequest {
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub outcomes: Vec<RefreshOutcome>,
    pub patch_resolutions: Vec<PatchResolution>,
    pub trust_facts: Vec<VerifiedTrustFact>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyResult {
    pub lock: Lockfile,
    pub inputs_changed: u32,
    pub patches_changed: bool,
    pub has_changes: bool,
    pub failures: Vec<RefreshFailure>,
}

pub fn plan_refresh_inputs(request: RefreshInputsPlanRequest) -> Vec<ManifestInput> {
    select_inputs(&request.manifest, &request.selected)
        .into_iter()
        .filter(|input| input.fetch_policy.needs_generation_resolution())
        .cloned()
        .collect()
}

pub fn refresh_inputs(request: RefreshInputsRequest) -> Vec<RefreshOutcome> {
    let inputs_to_refresh = select_inputs(&request.manifest, &request.selected);
    assert!(
        inputs_to_refresh.len() as u64 <= MAX_REFRESH_BATCH as u64,
        "too many inputs to refresh: {}",
        inputs_to_refresh.len()
    );
    let resolution_map = input_resolution_map(request.resolutions);
    let source_state_map = source_state_map(&request.source_state);
    let freshness_decision_map = freshness_decision_map(request.freshness_decisions);

    inputs_to_refresh
        .into_iter()
        .map(|input| {
            refresh_one(
                input,
                &request.lock,
                &resolution_map,
                &source_state_map,
                &freshness_decision_map,
                &request.trust_facts,
            )
        })
        .collect()
}

pub fn list_stale(request: RefreshInputsRequest) -> StaleReport {
    let outcomes = refresh_inputs(request);
    let stale = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Updated(resolved) => Some(resolved.name.clone()),
            _ => None,
        })
        .collect();
    let unchanged = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Unchanged { name } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let skipped = skipped_from_outcomes(&outcomes);
    let network_required = network_required_from_outcomes(&outcomes);
    let failed = refresh_failures_from_outcomes(&outcomes);
    StaleReport {
        stale,
        unchanged,
        skipped,
        network_required,
        failed,
    }
}

pub fn plan_patch_resolutions(request: PatchResolutionPlanRequest) -> Vec<PatchDef> {
    let mut staged_lock = request.lock;
    apply_updated_outcomes(&mut staged_lock, &request.outcomes);
    compute_needed_patch_resolutions(&request.manifest, &staged_lock)
}

pub fn apply_outcomes(request: ApplyOutcomesRequest) -> ApplyResult {
    assert!(request.lock.inputs.len() as u64 <= MAX_INPUTS as u64, "lock inputs must stay within manifest limit");
    let mut new_lock = request.lock.clone();
    let inputs_changed = apply_updated_outcomes(&mut new_lock, &request.outcomes);

    let planned_patches = compute_needed_patch_resolutions(&request.manifest, &new_lock);
    let patch_resolution_map = patch_resolution_map(request.patch_resolutions);
    let mut patch_failures = Vec::new();
    let mut patch_changed = false;
    for patch_def in planned_patches {
        let patch_name = patch_def.name.clone();
        match patch_resolution_map.get(patch_name.as_str()) {
            Some(PatchResolution::Resolved { patch, .. }) => {
                match trusted_locked_patch(&patch_def, patch.clone(), &request.trust_facts) {
                    Ok(trusted_patch) => {
                        new_lock.patches.insert(patch_name, trusted_patch);
                        patch_changed = true;
                    }
                    Err(reason) => patch_failures.push(RefreshFailure {
                        name: patch_name,
                        reason,
                    }),
                }
            }
            Some(PatchResolution::Failed { reason, .. }) => patch_failures.push(RefreshFailure {
                name: patch_name,
                reason: reason.clone(),
            }),
            None => patch_failures.push(RefreshFailure {
                name: patch_name,
                reason: "missing patch resolution".to_string(),
            }),
        }
    }

    let reverted = revert_failed_patch_inputs(&request.lock, &mut new_lock, &patch_failures);
    let inputs_changed = inputs_changed.saturating_sub(reverted);
    let orphan_changed = remove_orphaned_patches(&mut new_lock);

    let has_changes = inputs_changed > 0 || patch_changed || orphan_changed;

    ApplyResult {
        lock: new_lock,
        inputs_changed,
        patches_changed: patch_changed || orphan_changed,
        has_changes,
        failures: patch_failures,
    }
}

fn select_inputs<'a>(manifest: &'a ProjectManifest, selected: &[String]) -> Vec<&'a ManifestInput> {
    if selected.is_empty() {
        return manifest.inputs.iter().collect();
    }
    let selected_set: BTreeSet<&str> = selected.iter().map(|value| value.as_str()).collect();
    manifest.inputs.iter().filter(|input| selected_set.contains(input.name.as_str())).collect()
}

fn input_resolution_map(resolutions: Vec<ResolvedInputState>) -> BTreeMap<String, ResolvedInputState> {
    let mut map = BTreeMap::new();
    for resolution in resolutions {
        let key = match &resolution {
            ResolvedInputState::Resolved(resolved) => resolved.name.clone(),
            ResolvedInputState::Failed(failure) => failure.name.clone(),
        };
        map.insert(key, resolution);
    }
    map
}

fn freshness_decision_map(decisions: Vec<FreshnessDecision>) -> BTreeMap<String, FreshnessDecision> {
    let mut map = BTreeMap::new();
    for decision in decisions {
        map.insert(decision.input_name.clone(), decision);
    }
    map
}

fn refresh_one(
    input: &ManifestInput,
    lock: &Lockfile,
    resolutions: &BTreeMap<String, ResolvedInputState>,
    source_state: &BTreeMap<&str, Vec<&InputSourceStateFact>>,
    freshness_decisions: &BTreeMap<String, FreshnessDecision>,
    trust_facts: &[VerifiedTrustFact],
) -> RefreshOutcome {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.mirrors.len() as u64 <= crate::MAX_MIRRORS_PER_INPUT as u64,
        "mirror count must stay within manifest limit"
    );
    if input.frozen {
        return RefreshOutcome::Frozen {
            name: input.name.clone(),
        };
    }
    if let Some(outcome) = freshness_gate(input, freshness_decisions) {
        return outcome;
    }
    let compatibility = fetch_policy_compatibility_problems(input);
    if let Some(problem) = compatibility.first() {
        return RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: problem.clone(),
        };
    }

    match input.fetch_policy {
        InputFetchPolicy::GenerationMaterial => refresh_generation_material(input, lock, resolutions, trust_facts),
        InputFetchPolicy::BuildFetchAction => refresh_without_generation_fetch(input, lock, trust_facts),
        InputFetchPolicy::ImportedSourceRequired => refresh_imported_source(input, lock, source_state, trust_facts),
    }
}

fn freshness_gate(
    input: &ManifestInput,
    freshness_decisions: &BTreeMap<String, FreshnessDecision>,
) -> Option<RefreshOutcome> {
    let decision = freshness_decisions.get(input.name.as_str())?;
    match decision.kind {
        FreshnessDecisionKind::Stale => None,
        FreshnessDecisionKind::Unchanged => Some(RefreshOutcome::Unchanged {
            name: input.name.clone(),
        }),
        FreshnessDecisionKind::Skipped => Some(RefreshOutcome::Skipped {
            name: input.name.clone(),
            reason: decision.reason.clone(),
        }),
        FreshnessDecisionKind::NetworkRequired => Some(RefreshOutcome::NetworkRequired {
            name: input.name.clone(),
            reason: decision.reason.clone(),
        }),
        FreshnessDecisionKind::Failed | FreshnessDecisionKind::MissingObservation => Some(RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: decision.reason.clone(),
        }),
    }
}

fn refresh_generation_material(
    input: &ManifestInput,
    lock: &Lockfile,
    resolutions: &BTreeMap<String, ResolvedInputState>,
    trust_facts: &[VerifiedTrustFact],
) -> RefreshOutcome {
    match resolutions.get(input.name.as_str()) {
        Some(ResolvedInputState::Resolved(resolved)) => {
            match trusted_resolved_input(input, resolved.clone(), trust_facts) {
                Ok(trusted) => unchanged_or_updated(input, lock, trusted),
                Err(reason) => RefreshOutcome::Failed {
                    name: input.name.clone(),
                    reason,
                },
            }
        }
        Some(ResolvedInputState::Failed(failure)) => RefreshOutcome::Failed {
            name: failure.name.clone(),
            reason: failure.reason.clone(),
        },
        None => RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: alloc::format!("missing resolution for {}", input.name),
        },
    }
}

fn refresh_without_generation_fetch(
    input: &ManifestInput,
    lock: &Lockfile,
    trust_facts: &[VerifiedTrustFact],
) -> RefreshOutcome {
    match lock_entry_without_fetch(input, lock.inputs.get(&input.name)) {
        Ok(entry) => {
            let resolved = ResolvedInput {
                name: input.name.clone(),
                entry,
            };
            match trusted_resolved_input(input, resolved, trust_facts) {
                Ok(trusted) => unchanged_or_updated(input, lock, trusted),
                Err(reason) => RefreshOutcome::Failed {
                    name: input.name.clone(),
                    reason,
                },
            }
        }
        Err(diagnostic) => RefreshOutcome::Failed {
            name: diagnostic.input_name,
            reason: diagnostic.message,
        },
    }
}

fn refresh_imported_source(
    input: &ManifestInput,
    lock: &Lockfile,
    source_state: &BTreeMap<&str, Vec<&InputSourceStateFact>>,
    trust_facts: &[VerifiedTrustFact],
) -> RefreshOutcome {
    let existing = lock.inputs.get(&input.name);
    if matching_source_fact(input, existing, source_state).is_none() {
        return RefreshOutcome::Failed {
            name: input.name.clone(),
            reason: "imported-source-required policy has no matching ready source-state record".to_string(),
        };
    }
    refresh_without_generation_fetch(input, lock, trust_facts)
}

fn unchanged_or_updated(input: &ManifestInput, lock: &Lockfile, resolved: ResolvedInput) -> RefreshOutcome {
    if let Some(existing) = lock.inputs.get(&input.name)
        && existing == &resolved.entry
    {
        return RefreshOutcome::Unchanged {
            name: input.name.clone(),
        };
    }
    RefreshOutcome::Updated(resolved)
}

fn trusted_resolved_input(
    input: &ManifestInput,
    mut resolved: ResolvedInput,
    trust_facts: &[VerifiedTrustFact],
) -> Result<ResolvedInput, String> {
    assert_eq!(resolved.name, input.name, "resolved input name must match manifest input");
    if let Some(policy) = &input.trust {
        let subject = TrustSubject::input(input.name.clone());
        let trust =
            evaluate_trust_policy(subject, policy, &resolved.entry.hash.algo, &resolved.entry.hash.value, trust_facts)?;
        resolved.entry.trust = Some(trust);
        return Ok(resolved);
    }
    resolved.entry.trust = None;
    Ok(resolved)
}

fn trusted_locked_patch(
    def: &PatchDef,
    mut patch: LockedPatch,
    trust_facts: &[VerifiedTrustFact],
) -> Result<LockedPatch, String> {
    assert!(!def.name.is_empty(), "patch name must not be empty");
    if let Some(policy) = &def.trust {
        let subject = TrustSubject::patch(def.name.clone());
        let trust = evaluate_trust_policy(subject, policy, &patch.hash.algo, &patch.hash.value, trust_facts)?;
        patch.trust = Some(trust);
        return Ok(patch);
    }
    patch.trust = None;
    Ok(patch)
}

fn source_state_map(facts: &[InputSourceStateFact]) -> BTreeMap<&str, Vec<&InputSourceStateFact>> {
    let mut map: BTreeMap<&str, Vec<&InputSourceStateFact>> = BTreeMap::new();
    for fact in facts {
        map.entry(fact.input_name.as_str()).or_default().push(fact);
    }
    map
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
    if fact.ready_class != crate::InputSourceStateClass::Ready {
        return false;
    }
    if fact.input_name != input.name {
        return false;
    }
    if let Some(existing) = existing
        && fact.hash_value == existing.hash.value
    {
        return true;
    }
    input.hash.expected.as_deref() == Some(fact.hash_value.as_str())
}

fn apply_updated_outcomes(lock: &mut Lockfile, outcomes: &[RefreshOutcome]) -> u32 {
    let mut inputs_changed: u32 = 0;
    for outcome in outcomes {
        if let RefreshOutcome::Updated(resolved) = outcome {
            lock.inputs.insert(resolved.name.clone(), resolved.entry.clone());
            inputs_changed = inputs_changed.saturating_add(1);
        }
    }
    inputs_changed
}

fn compute_needed_patch_resolutions(manifest: &ProjectManifest, lock: &Lockfile) -> Vec<PatchDef> {
    assert!(
        manifest.patches.len() as u64 <= MAX_LOCKED_PATCHES as u64,
        "manifest patch count must stay within lock limit"
    );
    let defs: BTreeMap<&str, &PatchDef> = manifest.patches.iter().map(|def| (def.name.as_str(), def)).collect();
    collect_needed_patches(lock)
        .into_iter()
        .filter_map(|name| {
            let def = defs.get(name.as_str())?;
            let needs_resolve = match lock.patches.get(name.as_str()) {
                None => true,
                Some(existing) => !patch_matches_def(existing, def),
            };
            if needs_resolve { Some((*def).clone()) } else { None }
        })
        .collect()
}

fn patch_resolution_map(resolutions: Vec<PatchResolution>) -> BTreeMap<String, PatchResolution> {
    let mut map = BTreeMap::new();
    for resolution in resolutions {
        let key = match &resolution {
            PatchResolution::Resolved { name, .. } => name.clone(),
            PatchResolution::Failed { name, .. } => name.clone(),
        };
        map.insert(key, resolution);
    }
    map
}

fn collect_needed_patches(lock: &Lockfile) -> BTreeSet<String> {
    let mut needed = BTreeSet::new();
    for entry in lock.inputs.values() {
        for patch_name in &entry.patches {
            needed.insert(patch_name.clone());
        }
    }
    needed
}

fn remove_orphaned_patches(lock: &mut Lockfile) -> bool {
    let needed = collect_needed_patches(lock);
    let before_len = lock.patches.len() as u32;
    lock.patches.retain(|name, _| needed.contains(name));
    lock.patches.len() as u32 != before_len
}

fn revert_failed_patch_inputs(old_lock: &Lockfile, new_lock: &mut Lockfile, failures: &[RefreshFailure]) -> u32 {
    if failures.is_empty() {
        return 0;
    }
    let failed_names: BTreeSet<&str> = failures.iter().map(|failure| failure.name.as_str()).collect();
    let impacted_inputs: Vec<String> = new_lock
        .inputs
        .iter()
        .filter(|(_, entry)| entry.patches.iter().any(|patch_name| failed_names.contains(patch_name.as_str())))
        .map(|(name, _)| name.clone())
        .collect();
    let mut reverted: u32 = 0;
    for name in impacted_inputs {
        match old_lock.inputs.get(&name) {
            Some(previous) => {
                let current = new_lock.inputs.get(&name);
                if current != Some(previous) {
                    new_lock.inputs.insert(name, previous.clone());
                    reverted = reverted.saturating_add(1);
                }
            }
            None => {
                if new_lock.inputs.remove(&name).is_some() {
                    reverted = reverted.saturating_add(1);
                }
            }
        }
    }
    reverted
}

fn patch_matches_def(locked: &LockedPatch, def: &PatchDef) -> bool {
    if !patch_trust_matches_def(locked, def) {
        return false;
    }
    match (&locked.source, &def.source) {
        (LockedPatchSource::Local { path: locked_path }, PatchSource::Local { path: def_path }) => {
            locked_path == def_path
        }
        (
            LockedPatchSource::Remote { url: locked_url },
            PatchSource::Remote {
                url: def_url,
                hash: def_hash,
            },
        ) => {
            if locked_url != def_url {
                return false;
            }
            if locked.hash.algo != def_hash.algo {
                return false;
            }
            if let Some(expected) = &def_hash.expected
                && locked.hash.value != *expected
            {
                return false;
            }
            true
        }
        _ => false,
    }
}

fn patch_trust_matches_def(locked: &LockedPatch, def: &PatchDef) -> bool {
    match (&def.trust, &locked.trust) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(_), None) => false,
        (Some(policy), Some(trust)) => {
            if !trust_policy_matches_locked(policy, trust) {
                return false;
            }
            let subject = TrustSubject::patch(def.name.clone());
            validate_locked_trust(trust, &subject, &locked.hash.algo, &locked.hash.value, "locked patch trust")
                .is_empty()
        }
    }
}

fn skipped_from_outcomes(outcomes: &[RefreshOutcome]) -> Vec<RefreshFailure> {
    outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Skipped { name, reason } => Some(RefreshFailure {
                name: name.clone(),
                reason: reason.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn network_required_from_outcomes(outcomes: &[RefreshOutcome]) -> Vec<RefreshFailure> {
    outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::NetworkRequired { name, reason } => Some(RefreshFailure {
                name: name.clone(),
                reason: reason.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn refresh_failures_from_outcomes(outcomes: &[RefreshOutcome]) -> Vec<RefreshFailure> {
    outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Failed { name, reason } => Some(RefreshFailure {
                name: name.clone(),
                reason: reason.clone(),
            }),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::format;
    use alloc::vec;

    use super::*;
    use crate::FreshnessDecision;
    use crate::FreshnessDecisionKind;
    use crate::HashAlgo;
    use crate::HashSpec;
    use crate::InputKind;
    use crate::InputSourceStateClass;
    use crate::InputTrustPolicy;
    use crate::LockedHash;
    use crate::LockedKind;
    use crate::ManifestInput;
    use crate::PROJECT_INPUT_TRUST_NON_CLAIM;
    use crate::PatchSource;
    use crate::SchemaVersion;
    use crate::TrustDigestBinding;
    use crate::TrustSignatureRef;
    use crate::TrustSubject;
    use crate::TrustVerifierKind;
    use crate::VerifiedTrustFact;

    fn file_input(name: &str, url: &str) -> ManifestInput {
        ManifestInput {
            name: name.into(),
            kind: InputKind::File { url: url.into() },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
            fetch_policy: InputFetchPolicy::GenerationMaterial,
            retention: None,
            freshness: None,
            trust: None,
        }
    }

    fn manifest_with(inputs: Vec<ManifestInput>) -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".into(),
            inputs,
            patches: vec![],
            retention: crate::InputRetentionPolicy::Untracked,
        }
    }

    fn empty_lock() -> Lockfile {
        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        }
    }

    fn resolved_file(name: &str, url: &str, hash: &str) -> ResolvedInputState {
        ResolvedInputState::Resolved(ResolvedInput {
            name: name.to_string(),
            entry: LockEntry {
                kind: LockedKind::File { url: url.to_string() },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash.to_string(),
                },
                patches: vec![],
                mirrors: vec![],
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                freshness: None,
                trust: None,
            },
        })
    }

    fn freshness_decision(name: &str, kind: FreshnessDecisionKind, reason: &str) -> FreshnessDecision {
        FreshnessDecision {
            input_name: name.to_string(),
            kind,
            observed_value_digest: None,
            locked_value_digest: None,
            reason: reason.to_string(),
        }
    }

    fn source_fact(name: &str, hash: &str) -> InputSourceStateFact {
        InputSourceStateFact {
            input_name: name.to_string(),
            hash_value: hash.to_string(),
            identity: format!("fixture-{name}"),
            source_state_blake3: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            ready_class: InputSourceStateClass::Ready,
        }
    }

    fn trust_policy() -> InputTrustPolicy {
        InputTrustPolicy {
            verifier: TrustVerifierKind::Ed25519Detached,
            signatures: vec![TrustSignatureRef::LocalFile {
                path: "pkg.sig".to_string(),
            }],
            trusted_public_keys: vec!["alice:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string()],
            required_signers: vec!["alice".to_string()],
            quorum: 1,
            digest_binding: TrustDigestBinding::ContentHash,
        }
    }

    fn trust_fact(subject: TrustSubject, hash: &str) -> VerifiedTrustFact {
        VerifiedTrustFact {
            subject,
            verifier: TrustVerifierKind::Ed25519Detached,
            digest_binding: TrustDigestBinding::ContentHash,
            hash_algo: HashAlgo::Sha256,
            hash_value: hash.to_string(),
            signer: "alice".to_string(),
            key_ref: "alice:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
            signature_ref: "pkg.sig".to_string(),
        }
    }

    #[test]
    fn refresh_inputs_marks_resolved_entry_updated() {
        let manifest = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest,
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: vec![resolved_file("data", "https://example.com/data.bin", "sha256-abc=")],
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });

        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            RefreshOutcome::Updated(resolved) => {
                assert_eq!(resolved.name, "data");
                assert_eq!(resolved.entry.hash.value, "sha256-abc=");
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn refresh_inputs_marks_matching_entry_unchanged() {
        let manifest = manifest_with(vec![file_input("data", "https://example.com/data.bin")]);
        let mut lock = empty_lock();
        lock.inputs.insert("data".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/data.bin".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-same=".into(),
            },
            patches: vec![],
            mirrors: vec![],
            fetch_policy: InputFetchPolicy::GenerationMaterial,
            freshness: None,
            trust: None,
        });

        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest,
            lock,
            selected: Vec::new(),
            resolutions: vec![resolved_file("data", "https://example.com/data.bin", "sha256-same=")],
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });
        assert!(matches!(&outcomes[0], RefreshOutcome::Unchanged { .. }));
    }

    #[test]
    fn refresh_inputs_marks_frozen_without_resolution() {
        let mut input = file_input("frozen", "https://example.com/f");
        input.frozen = true;
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });
        assert!(matches!(&outcomes[0], RefreshOutcome::Frozen { name } if name == "frozen"));
    }

    #[test]
    fn freshness_unchanged_decision_skips_resolution_work() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.freshness = Some(crate::FreshnessProbe::LocalFile {
            path: "VERSION".to_string(),
        });
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: Vec::new(),
            freshness_decisions: vec![freshness_decision(
                "pkg",
                FreshnessDecisionKind::Unchanged,
                "freshness value matches lock",
            )],
            trust_facts: Vec::new(),
        });

        assert_eq!(outcomes.len(), 1);
        assert!(matches!(&outcomes[0], RefreshOutcome::Unchanged { name } if name == "pkg"));
    }

    #[test]
    fn freshness_network_required_decision_fails_closed_for_refresh() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.freshness = Some(crate::FreshnessProbe::HttpText {
            url: "https://example.com/version".to_string(),
        });
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: Vec::new(),
            freshness_decisions: vec![freshness_decision(
                "pkg",
                FreshnessDecisionKind::NetworkRequired,
                "network disabled",
            )],
            trust_facts: Vec::new(),
        });

        assert_eq!(outcomes.len(), 1);
        assert!(
            matches!(&outcomes[0], RefreshOutcome::NetworkRequired { name, reason } if name == "pkg" && reason.contains("network"))
        );
    }

    #[test]
    fn plan_refresh_inputs_filters_selected_inputs() {
        let manifest = manifest_with(vec![
            file_input("a", "https://example.com/a"),
            file_input("b", "https://example.com/b"),
            file_input("c", "https://example.com/c"),
        ]);
        let plan = plan_refresh_inputs(RefreshInputsPlanRequest {
            manifest,
            selected: vec!["a".into(), "c".into()],
        });
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].name, "a");
        assert_eq!(plan[1].name, "c");
    }

    #[test]
    fn plan_refresh_inputs_excludes_build_fetch_policy_from_resolver_work() {
        let mut build_fetch = file_input("build", "https://example.com/build");
        build_fetch.fetch_policy = InputFetchPolicy::BuildFetchAction;
        let manifest = manifest_with(vec![file_input("gen", "https://example.com/gen"), build_fetch]);
        let plan = plan_refresh_inputs(RefreshInputsPlanRequest {
            manifest,
            selected: Vec::new(),
        });

        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "gen");
    }

    #[test]
    fn refresh_build_fetch_policy_locks_expected_hash_without_resolution() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.fetch_policy = InputFetchPolicy::BuildFetchAction;
        input.hash.expected = Some("sha256-expected=".to_string());
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });

        match &outcomes[0] {
            RefreshOutcome::Updated(resolved) => {
                assert_eq!(resolved.entry.hash.value, "sha256-expected=");
                assert_eq!(resolved.entry.fetch_policy, InputFetchPolicy::BuildFetchAction);
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn refresh_imported_source_policy_requires_ready_source_state() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.fetch_policy = InputFetchPolicy::ImportedSourceRequired;
        input.hash.expected = Some("sha256-expected=".to_string());
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: vec![source_fact("pkg", "sha256-expected=")],
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });

        assert!(
            matches!(&outcomes[0], RefreshOutcome::Updated(resolved) if resolved.entry.fetch_policy == InputFetchPolicy::ImportedSourceRequired)
        );
    }

    #[test]
    fn refresh_imported_source_policy_fails_without_source_state() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.fetch_policy = InputFetchPolicy::ImportedSourceRequired;
        input.hash.expected = Some("sha256-expected=".to_string());
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: Vec::new(),
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });

        assert!(
            matches!(&outcomes[0], RefreshOutcome::Failed { name, reason } if name == "pkg" && reason.contains("source-state"))
        );
    }

    #[test]
    fn apply_outcomes_updates_lock_and_resolves_patches() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["fix1".into()],
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                retention: None,
                freshness: None,
                trust: None,
            }],
            patches: vec![PatchDef {
                name: "fix1".into(),
                source: PatchSource::Local {
                    path: "patches/fix1.patch".into(),
                },
                trust: None,
            }],
            retention: crate::InputRetentionPolicy::Untracked,
        };
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "pkg".into(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-pkghash=".into(),
                },
                patches: vec!["fix1".into()],
                mirrors: vec![],
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                freshness: None,
                trust: None,
            },
        })];
        let patch_plan = plan_patch_resolutions(PatchResolutionPlanRequest {
            manifest: manifest.clone(),
            lock: empty_lock(),
            outcomes: outcomes.clone(),
        });
        assert_eq!(patch_plan.len(), 1);
        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock: empty_lock(),
            outcomes,
            patch_resolutions: vec![PatchResolution::Resolved {
                name: "fix1".into(),
                patch: LockedPatch {
                    source: LockedPatchSource::Local {
                        path: "patches/fix1.patch".into(),
                    },
                    hash: LockedHash {
                        algo: HashAlgo::Sha256,
                        value: "sha256-patchhash=".into(),
                    },
                    trust: None,
                },
            }],
            trust_facts: Vec::new(),
        });

        assert!(result.lock.inputs.contains_key("pkg"));
        assert!(result.lock.patches.contains_key("fix1"));
        assert!(result.patches_changed);
        assert_eq!(result.inputs_changed, 1);
    }

    #[test]
    fn apply_outcomes_reverts_input_when_patch_resolution_fails() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["bad".into()],
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                retention: None,
                freshness: None,
                trust: None,
            }],
            patches: vec![PatchDef {
                name: "bad".into(),
                source: PatchSource::Local {
                    path: "patches/bad.patch".into(),
                },
                trust: None,
            }],
            retention: crate::InputRetentionPolicy::Untracked,
        };
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "pkg".into(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-pkghash=".into(),
                },
                patches: vec!["bad".into()],
                mirrors: vec![],
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                freshness: None,
                trust: None,
            },
        })];

        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock: empty_lock(),
            outcomes,
            patch_resolutions: vec![PatchResolution::Failed {
                name: "bad".into(),
                reason: "hash failed".into(),
            }],
            trust_facts: Vec::new(),
        });

        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].name, "bad");
        assert!(!result.lock.inputs.contains_key("pkg"));
        assert!(!result.lock.patches.contains_key("bad"));
    }

    #[test]
    fn apply_outcomes_removes_orphaned_patches() {
        let manifest = manifest_with(vec![file_input("pkg", "https://example.com/pkg")]);
        let mut lock = empty_lock();
        lock.patches.insert("old-patch".into(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "old.patch".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-old=".into(),
            },
            trust: None,
        });
        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock,
            outcomes: vec![RefreshOutcome::Updated(ResolvedInput {
                name: "pkg".into(),
                entry: LockEntry {
                    kind: LockedKind::File {
                        url: "https://example.com/pkg".into(),
                    },
                    hash: LockedHash {
                        algo: HashAlgo::Sha256,
                        value: "sha256-h=".into(),
                    },
                    patches: vec![],
                    mirrors: vec![],
                    fetch_policy: InputFetchPolicy::GenerationMaterial,
                    freshness: None,
                    trust: None,
                },
            })],
            patch_resolutions: Vec::new(),
            trust_facts: Vec::new(),
        });

        assert!(!result.lock.patches.contains_key("old-patch"));
        assert!(result.patches_changed);
    }

    #[test]
    fn refresh_inputs_attaches_verified_trust_to_updated_lock() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.trust = Some(trust_policy());
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: vec![resolved_file("pkg", "https://example.com/pkg", "sha256-trusted=")],
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: vec![trust_fact(TrustSubject::input("pkg".to_string()), "sha256-trusted=")],
        });

        let RefreshOutcome::Updated(resolved) = &outcomes[0] else {
            panic!("expected trusted update, got {:?}", outcomes[0]);
        };
        let trust = resolved.entry.trust.as_ref().expect("trusted input lock should carry evidence");
        assert_eq!(trust.signers, vec!["alice".to_string()]);
        assert!(trust.claim.contains(PROJECT_INPUT_TRUST_NON_CLAIM));
    }

    #[test]
    fn refresh_inputs_rejects_trusted_input_without_evidence() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.trust = Some(trust_policy());
        let outcomes = refresh_inputs(RefreshInputsRequest {
            manifest: manifest_with(vec![input]),
            lock: empty_lock(),
            selected: Vec::new(),
            resolutions: vec![resolved_file("pkg", "https://example.com/pkg", "sha256-trusted=")],
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });

        assert!(
            matches!(&outcomes[0], RefreshOutcome::Failed { reason, .. } if reason.contains("missing required trust signer"))
        );
    }

    #[test]
    fn apply_outcomes_rejects_trusted_patch_without_evidence() {
        let mut input = file_input("pkg", "https://example.com/pkg");
        input.patches = vec!["fix".to_string()];
        let mut manifest = manifest_with(vec![input]);
        manifest.patches = vec![PatchDef {
            name: "fix".to_string(),
            source: PatchSource::Local {
                path: "patches/fix.patch".to_string(),
            },
            trust: Some(trust_policy()),
        }];
        let outcomes = vec![RefreshOutcome::Updated(ResolvedInput {
            name: "pkg".to_string(),
            entry: LockEntry {
                kind: LockedKind::File {
                    url: "https://example.com/pkg".to_string(),
                },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: "sha256-pkg=".to_string(),
                },
                patches: vec!["fix".to_string()],
                mirrors: Vec::new(),
                fetch_policy: InputFetchPolicy::GenerationMaterial,
                freshness: None,
                trust: None,
            },
        })];
        let result = apply_outcomes(ApplyOutcomesRequest {
            manifest,
            lock: empty_lock(),
            outcomes,
            patch_resolutions: vec![PatchResolution::Resolved {
                name: "fix".to_string(),
                patch: LockedPatch {
                    source: LockedPatchSource::Local {
                        path: "patches/fix.patch".to_string(),
                    },
                    hash: LockedHash {
                        algo: HashAlgo::Sha256,
                        value: "sha256-fix=".to_string(),
                    },
                    trust: None,
                },
            }],
            trust_facts: Vec::new(),
        });

        assert_eq!(result.failures.len(), 1);
        assert!(result.failures[0].reason.contains("missing required trust signer"));
        assert!(!result.lock.patches.contains_key("fix"));
    }

    #[test]
    fn list_stale_separates_changed_and_failed_inputs() {
        let manifest = manifest_with(vec![
            file_input("fresh", "https://example.com/fresh"),
            file_input("stale", "https://example.com/stale"),
            file_input("broken", "https://example.com/broken"),
        ]);
        let mut lock = empty_lock();
        lock.inputs.insert("fresh".into(), LockEntry {
            kind: LockedKind::File {
                url: "https://example.com/fresh".into(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-current=".into(),
            },
            patches: vec![],
            mirrors: vec![],
            fetch_policy: InputFetchPolicy::GenerationMaterial,
            freshness: None,
            trust: None,
        });
        let stale = list_stale(RefreshInputsRequest {
            manifest,
            lock,
            selected: Vec::new(),
            resolutions: vec![
                resolved_file("fresh", "https://example.com/fresh", "sha256-current="),
                resolved_file("stale", "https://example.com/stale", "sha256-current="),
                ResolvedInputState::Failed(RefreshFailure {
                    name: "broken".into(),
                    reason: "network down".into(),
                }),
            ],
            source_state: Vec::new(),
            freshness_decisions: Vec::new(),
            trust_facts: Vec::new(),
        });
        assert_eq!(stale.stale, vec!["stale"]);
        assert_eq!(stale.failed.len(), 1);
        assert_eq!(stale.failed[0].name, "broken");
    }
}
