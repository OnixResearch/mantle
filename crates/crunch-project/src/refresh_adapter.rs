use crunch_project_core::ApplyOutcomesRequest;
use crunch_project_core::FreshnessDecision;
use crunch_project_core::FreshnessObservation;
use crunch_project_core::FreshnessRefreshPlanRequest;
use crunch_project_core::FreshnessTemplateDestination;
use crunch_project_core::FreshnessTemplateRequest;
use crunch_project_core::LockedFreshnessValue;
use crunch_project_core::PatchResolution;
use crunch_project_core::PatchResolutionPlanRequest;
use crunch_project_core::RefreshFailure;
use crunch_project_core::RefreshInputsPlanRequest;
use crunch_project_core::RefreshInputsRequest;
use crunch_project_core::ResolvedInput;
use crunch_project_core::ResolvedInputState;
use crunch_project_core::VerifiedTrustFact;

use crate::DarcsSelector;
use crate::Error;
use crate::FossilSelector;
use crate::GitReference;
use crate::HashAlgo;
use crate::HashResolutionMode;
use crate::InputKind;
use crate::LockEntry;
use crate::LockedHash;
use crate::LockedKind;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::ManifestInput;
use crate::PatchDef;
use crate::PatchSource;
use crate::PijulSelector;
use crate::ProjectManifest;
use crate::refresh::ApplyResult;
use crate::refresh::RefreshOutcome;
use crate::refresh::StaleReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDarcsIdentity {
    pub context: Option<String>,
    pub weak_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPijulIdentity {
    pub channel: String,
    pub state: String,
    pub change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFossilIdentity {
    pub checkin: String,
}

pub trait RefreshResolver {
    fn resolve_git_rev(&self, repository: &str, reference: &GitReference) -> Result<Option<String>, Error>;

    fn hash_url_content(&self, url: &str, algo: &HashAlgo, mode: HashResolutionMode) -> Result<Option<String>, Error>;

    fn hash_git_checkout(&self, repository: &str, rev: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
        let _ = (repository, rev, algo);
        Ok(None)
    }

    fn resolve_darcs_identity(
        &self,
        repository: &str,
        selector: &DarcsSelector,
    ) -> Result<Option<ResolvedDarcsIdentity>, Error> {
        let _ = (repository, selector);
        Err(Error::Manifest("unsupported-VCS-tool: darcs resolver is not implemented".into()))
    }

    fn hash_darcs_checkout(
        &self,
        repository: &str,
        identity: &ResolvedDarcsIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error> {
        let _ = (repository, identity, algo);
        Err(Error::Manifest("unsupported-VCS-tool: darcs checkout hashing is not implemented".into()))
    }

    fn resolve_pijul_identity(
        &self,
        repository: &str,
        selector: &PijulSelector,
    ) -> Result<Option<ResolvedPijulIdentity>, Error> {
        let _ = (repository, selector);
        Err(Error::Manifest("unsupported-VCS-tool: pijul resolver is not implemented".into()))
    }

    fn hash_pijul_checkout(
        &self,
        repository: &str,
        identity: &ResolvedPijulIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error> {
        let _ = (repository, identity, algo);
        Err(Error::Manifest("unsupported-VCS-tool: pijul checkout hashing is not implemented".into()))
    }

    fn resolve_fossil_identity(
        &self,
        repository: &str,
        selector: &FossilSelector,
    ) -> Result<Option<ResolvedFossilIdentity>, Error> {
        let _ = (repository, selector);
        Err(Error::Manifest("unsupported-VCS-tool: fossil resolver is not implemented".into()))
    }

    fn hash_fossil_checkout(
        &self,
        repository: &str,
        identity: &ResolvedFossilIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, Error> {
        let _ = (repository, identity, algo);
        Err(Error::Manifest("unsupported-VCS-tool: fossil checkout hashing is not implemented".into()))
    }

    fn hash_local_file(&self, path: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
        let _ = (path, algo);
        Ok(None)
    }

    fn observe_freshness(
        &self,
        input: &ManifestInput,
        no_network: bool,
    ) -> Result<Option<FreshnessObservation>, Error> {
        let _ = (input, no_network);
        Ok(None)
    }

    fn verify_input_trust(&self, input: &ManifestInput, entry: &LockEntry) -> Result<Vec<VerifiedTrustFact>, Error> {
        let _ = (input, entry);
        Ok(Vec::new())
    }

    fn verify_patch_trust(&self, patch: &PatchDef, locked: &LockedPatch) -> Result<Vec<VerifiedTrustFact>, Error> {
        let _ = (patch, locked);
        Ok(Vec::new())
    }
}

pub fn refresh_inputs(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
) -> Vec<RefreshOutcome> {
    refresh_inputs_with_options(manifest, lock, selected, resolver, false)
}

pub fn refresh_inputs_with_options(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
    no_network: bool,
) -> Vec<RefreshOutcome> {
    let request = build_refresh_request(manifest, lock, selected, resolver, no_network);
    crunch_project_core::refresh_inputs(request)
}

pub fn apply_outcomes(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    outcomes: &[RefreshOutcome],
    resolver: &dyn RefreshResolver,
) -> ApplyResult {
    let (patch_resolutions, trust_facts) = resolve_needed_patches(manifest, lock, outcomes, resolver);
    crunch_project_core::apply_outcomes(ApplyOutcomesRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        outcomes: outcomes.to_vec(),
        patch_resolutions,
        trust_facts,
    })
}

pub fn list_stale(manifest: &ProjectManifest, lock: &Lockfile, resolver: &dyn RefreshResolver) -> StaleReport {
    list_stale_with_options(manifest, lock, resolver, false)
}

pub fn list_stale_with_options(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    resolver: &dyn RefreshResolver,
    no_network: bool,
) -> StaleReport {
    let request = build_refresh_request(manifest, lock, &[], resolver, no_network);
    crunch_project_core::list_stale(request)
}

fn build_refresh_request(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
    no_network: bool,
) -> RefreshInputsRequest {
    let plan = crunch_project_core::plan_refresh_inputs(RefreshInputsPlanRequest {
        manifest: manifest.clone(),
        selected: selected.to_vec(),
    });
    assert!(plan.len() <= manifest.inputs.len(), "refresh plan must be a manifest input subset");
    let observations = collect_freshness_observations(&plan, resolver, no_network);
    assert!(observations.len() <= plan.len(), "freshness collection must emit at most one observation per input");
    let decisions = plan_freshness_decisions(&plan, lock, selected, &observations, no_network);
    let mut trust_facts = Vec::new();
    let resolutions = plan
        .into_iter()
        .filter(|input| !input.frozen)
        .filter(|input| should_resolve_input(input, &decisions))
        .map(|input| {
            resolve_manifest_input_with_trust(
                &input,
                resolver,
                freshness_observation_for(&input.name, &observations),
                &mut trust_facts,
            )
        })
        .collect();
    RefreshInputsRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        selected: selected.to_vec(),
        resolutions,
        source_state: Vec::new(),
        freshness_decisions: decisions,
        trust_facts,
    }
}

fn collect_freshness_observations(
    plan: &[ManifestInput],
    resolver: &dyn RefreshResolver,
    no_network: bool,
) -> Vec<FreshnessObservation> {
    plan.iter()
        .filter(|input| input.freshness.is_some())
        .filter_map(|input| match resolver.observe_freshness(input, no_network) {
            Ok(Some(observation)) => Some(observation),
            Ok(None) => None,
            Err(err) => Some(failed_freshness_observation(input, err.to_string())),
        })
        .collect()
}

fn plan_freshness_decisions(
    plan: &[ManifestInput],
    lock: &Lockfile,
    selected: &[String],
    observations: &[FreshnessObservation],
    no_network: bool,
) -> Vec<FreshnessDecision> {
    let declared_inputs = plan
        .iter()
        .filter(|input| input.freshness.is_some())
        .map(|input| input.name.clone())
        .collect::<Vec<_>>();
    assert!(declared_inputs.len() <= plan.len(), "freshness declarations must be a refresh plan subset");
    if declared_inputs.is_empty() {
        return Vec::new();
    }
    let locked_values = declared_inputs
        .iter()
        .filter_map(|name| lock.inputs.get(name).and_then(|entry| entry.freshness.clone()))
        .collect::<Vec<_>>();
    assert!(locked_values.len() <= declared_inputs.len(), "locked freshness values must match declared inputs");
    let selected = selected
        .iter()
        .filter(|name| declared_inputs.iter().any(|declared| declared == *name))
        .cloned()
        .collect::<Vec<_>>();
    let requested_inputs = if selected.is_empty() {
        declared_inputs.clone()
    } else {
        selected.clone()
    };
    crunch_project_core::plan_freshness_refresh(FreshnessRefreshPlanRequest {
        declared_inputs,
        selected,
        locked_values,
        observations: observations.to_vec(),
        no_network,
    })
    .unwrap_or_else(|err| freshness_plan_failures(requested_inputs, err.to_string()))
}

fn freshness_plan_failures(input_names: Vec<String>, reason: String) -> Vec<FreshnessDecision> {
    input_names
        .into_iter()
        .map(|input_name| FreshnessDecision {
            input_name,
            kind: crunch_project_core::FreshnessDecisionKind::Failed,
            observed_value_digest: None,
            locked_value_digest: None,
            reason: reason.clone(),
        })
        .collect()
}

fn should_resolve_input(input: &ManifestInput, decisions: &[FreshnessDecision]) -> bool {
    match decisions.iter().find(|decision| decision.input_name == input.name) {
        None => true,
        Some(decision) => matches!(decision.kind, crunch_project_core::FreshnessDecisionKind::Stale),
    }
}

fn freshness_observation_for<'a>(
    input_name: &str,
    observations: &'a [FreshnessObservation],
) -> Option<&'a FreshnessObservation> {
    observations.iter().find(|observation| observation.input_name == input_name)
}

fn failed_freshness_observation(input: &ManifestInput, reason: String) -> FreshnessObservation {
    let probe_kind = input
        .freshness
        .as_ref()
        .map(freshness_kind)
        .unwrap_or(crunch_project_core::FreshnessProbeKind::Command);
    let is_network_required = input.freshness.as_ref().is_some_and(freshness_requires_network);
    let diagnostic = if reason.is_empty() {
        "freshness resolver failed without a diagnostic".to_string()
    } else {
        reason
    };
    let request = crunch_project_core::FreshnessObservationRequest {
        version: crunch_project_core::FRESHNESS_PROBE_VERSION,
        input_name: input.name.clone(),
        probe_kind,
        requires_network: is_network_required,
        status: crunch_project_core::FreshnessObservationStatus::Failed,
        value: None,
        diagnostic,
        probe_identity_digest: None,
    };
    assert_eq!(request.input_name, input.name, "failed observation must retain the manifest input name");
    assert!(!request.diagnostic.is_empty(), "failed observation must carry a diagnostic");
    match crunch_project_core::normalize_freshness_observation(request) {
        Ok(observation) => observation,
        Err(err) => FreshnessObservation {
            version: crunch_project_core::FRESHNESS_PROBE_VERSION,
            input_name: input.name.clone(),
            probe_kind,
            requires_network: is_network_required,
            status: crunch_project_core::FreshnessObservationStatus::Failed,
            value: None,
            value_digest: None,
            diagnostic: format!("invalid failed freshness observation: {err}"),
            probe_identity_digest: None,
        },
    }
}

fn freshness_kind(probe: &crunch_project_core::FreshnessProbe) -> crunch_project_core::FreshnessProbeKind {
    match probe {
        crunch_project_core::FreshnessProbe::GitRef { .. } => crunch_project_core::FreshnessProbeKind::GitRef,
        crunch_project_core::FreshnessProbe::HttpText { .. } => crunch_project_core::FreshnessProbeKind::HttpText,
        crunch_project_core::FreshnessProbe::HttpJson { .. } => crunch_project_core::FreshnessProbeKind::HttpJson,
        crunch_project_core::FreshnessProbe::LocalFile { .. } => crunch_project_core::FreshnessProbeKind::LocalFile,
        crunch_project_core::FreshnessProbe::LocalDirectory { .. } => {
            crunch_project_core::FreshnessProbeKind::LocalDirectory
        }
        crunch_project_core::FreshnessProbe::Command { .. } => crunch_project_core::FreshnessProbeKind::Command,
    }
}

fn freshness_requires_network(probe: &crunch_project_core::FreshnessProbe) -> bool {
    match probe {
        crunch_project_core::FreshnessProbe::Command { requires_network, .. } => *requires_network,
        _ => freshness_kind(probe).requires_network(),
    }
}

fn resolve_manifest_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    freshness_observation: Option<&FreshnessObservation>,
) -> ResolvedInputState {
    match resolve_input(input, resolver, freshness_observation) {
        Ok(entry) => ResolvedInputState::Resolved(ResolvedInput {
            name: input.name.clone(),
            entry,
        }),
        Err(err) => ResolvedInputState::Failed(RefreshFailure {
            name: input.name.clone(),
            reason: err.to_string(),
        }),
    }
}

fn resolve_manifest_input_with_trust(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    freshness_observation: Option<&FreshnessObservation>,
    trust_facts: &mut Vec<VerifiedTrustFact>,
) -> ResolvedInputState {
    let resolved = resolve_manifest_input(input, resolver, freshness_observation);
    let ResolvedInputState::Resolved(resolved_input) = resolved else {
        return resolved;
    };
    if input.trust.is_none() {
        return ResolvedInputState::Resolved(resolved_input);
    }
    match resolver.verify_input_trust(input, &resolved_input.entry) {
        Ok(mut facts) => {
            trust_facts.append(&mut facts);
            ResolvedInputState::Resolved(resolved_input)
        }
        Err(err) => ResolvedInputState::Failed(RefreshFailure {
            name: input.name.clone(),
            reason: err.to_string(),
        }),
    }
}

fn require_resolution<T>(value: Option<T>, what: &str) -> Result<T, Error> {
    value.ok_or_else(|| Error::Manifest(format!("unable to resolve {what}")))
}

fn require_string_resolution(value: Option<String>, what: &str) -> Result<String, Error> {
    let resolved = require_resolution(value, what)?;
    if resolved.is_empty() {
        return Err(Error::Manifest(format!("resolver returned empty {what}")));
    }
    Ok(resolved)
}

fn render_input_kind(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
) -> Result<InputKind, Error> {
    match &input.kind {
        InputKind::File { url } => Ok(InputKind::File {
            url: render_template_if_needed(input, freshness_observation, url, FreshnessTemplateDestination::Url)?,
        }),
        InputKind::Tarball { url } => Ok(InputKind::Tarball {
            url: render_template_if_needed(input, freshness_observation, url, FreshnessTemplateDestination::Url)?,
        }),
        InputKind::Git { repository, reference } => Ok(InputKind::Git {
            repository: repository.clone(),
            reference: render_git_reference(input, freshness_observation, reference)?,
        }),
        InputKind::Darcs { repository, selector } => Ok(InputKind::Darcs {
            repository: repository.clone(),
            selector: render_darcs_selector(input, freshness_observation, selector)?,
        }),
        InputKind::Pijul { repository, selector } => Ok(InputKind::Pijul {
            repository: repository.clone(),
            selector: render_pijul_selector(input, freshness_observation, selector)?,
        }),
        InputKind::Fossil { repository, selector } => Ok(InputKind::Fossil {
            repository: repository.clone(),
            selector: render_fossil_selector(input, freshness_observation, selector)?,
        }),
    }
}

fn render_git_reference(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
    reference: &GitReference,
) -> Result<GitReference, Error> {
    match reference {
        GitReference::Branch(branch) => Ok(GitReference::Branch(render_template_if_needed(
            input,
            freshness_observation,
            branch,
            FreshnessTemplateDestination::Reference,
        )?)),
        GitReference::Tag(tag) => Ok(GitReference::Tag(render_template_if_needed(
            input,
            freshness_observation,
            tag,
            FreshnessTemplateDestination::Reference,
        )?)),
        GitReference::Rev(rev) => Ok(GitReference::Rev(render_template_if_needed(
            input,
            freshness_observation,
            rev,
            FreshnessTemplateDestination::Reference,
        )?)),
    }
}

fn render_darcs_selector(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
    selector: &DarcsSelector,
) -> Result<DarcsSelector, Error> {
    match selector {
        DarcsSelector::Tag(tag) => Ok(DarcsSelector::Tag(render_template_if_needed(
            input,
            freshness_observation,
            tag,
            FreshnessTemplateDestination::Reference,
        )?)),
        DarcsSelector::Context(context) => Ok(DarcsSelector::Context(render_template_if_needed(
            input,
            freshness_observation,
            context,
            FreshnessTemplateDestination::Reference,
        )?)),
    }
}

fn render_pijul_selector(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
    selector: &PijulSelector,
) -> Result<PijulSelector, Error> {
    match selector {
        PijulSelector::Channel { channel } => Ok(PijulSelector::Channel {
            channel: render_template_if_needed(
                input,
                freshness_observation,
                channel,
                FreshnessTemplateDestination::Reference,
            )?,
        }),
        PijulSelector::State { channel, state } => Ok(PijulSelector::State {
            channel: render_template_if_needed(
                input,
                freshness_observation,
                channel,
                FreshnessTemplateDestination::Reference,
            )?,
            state: render_template_if_needed(
                input,
                freshness_observation,
                state,
                FreshnessTemplateDestination::Reference,
            )?,
        }),
        PijulSelector::Change { channel, change } => Ok(PijulSelector::Change {
            channel: render_template_if_needed(
                input,
                freshness_observation,
                channel,
                FreshnessTemplateDestination::Reference,
            )?,
            change: render_template_if_needed(
                input,
                freshness_observation,
                change,
                FreshnessTemplateDestination::Reference,
            )?,
        }),
    }
}

fn render_fossil_selector(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
    selector: &FossilSelector,
) -> Result<FossilSelector, Error> {
    match selector {
        FossilSelector::Branch(branch) => Ok(FossilSelector::Branch(render_template_if_needed(
            input,
            freshness_observation,
            branch,
            FreshnessTemplateDestination::Reference,
        )?)),
        FossilSelector::Tag(tag) => Ok(FossilSelector::Tag(render_template_if_needed(
            input,
            freshness_observation,
            tag,
            FreshnessTemplateDestination::Reference,
        )?)),
        FossilSelector::Checkin(checkin) => Ok(FossilSelector::Checkin(render_template_if_needed(
            input,
            freshness_observation,
            checkin,
            FreshnessTemplateDestination::Reference,
        )?)),
    }
}

fn render_template_if_needed(
    input: &ManifestInput,
    freshness_observation: Option<&FreshnessObservation>,
    template: &str,
    destination: FreshnessTemplateDestination,
) -> Result<String, Error> {
    if !template.contains("{{") {
        return Ok(template.to_string());
    }
    let observation = freshness_observation.ok_or_else(|| {
        Error::Manifest(format!("input '{}' uses a freshness template but has no observed freshness value", input.name))
    })?;
    crunch_project_core::render_freshness_template(FreshnessTemplateRequest {
        input_name: input.name.clone(),
        template: template.to_string(),
        observation: observation.clone(),
        destination,
        max_output_bytes: crunch_project_core::MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES,
    })
    .map_err(|err| Error::Manifest(format!("rendering freshness template for {}: {err}", input.name)))
}

fn locked_freshness_from_observation(observation: &FreshnessObservation) -> Option<LockedFreshnessValue> {
    observation.value_digest.as_ref().map(|value_digest| LockedFreshnessValue {
        input_name: observation.input_name.clone(),
        value_digest: value_digest.clone(),
    })
}

type ResolvedLockKind = (LockedKind, LockedHash);

fn resolve_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    freshness_observation: Option<&FreshnessObservation>,
) -> Result<LockEntry, Error> {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.patches.len() as u64 <= crate::manifest::MAX_PATCHES_PER_INPUT as u64,
        "patch count must stay within manifest limit"
    );
    let effective_kind = render_input_kind(input, freshness_observation)?;
    let (kind, hash) = resolve_effective_input(input, resolver, &effective_kind)?;
    Ok(LockEntry {
        kind,
        hash,
        patches: input.patches.clone(),
        mirrors: input.mirrors.clone(),
        fetch_policy: input.fetch_policy,
        freshness: freshness_observation.and_then(locked_freshness_from_observation),
        trust: None,
    })
}

fn resolve_effective_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    effective_kind: &InputKind,
) -> Result<ResolvedLockKind, Error> {
    match effective_kind {
        InputKind::File { url } => resolve_file_input(input, resolver, url),
        InputKind::Tarball { url } => resolve_tarball_input(input, resolver, url),
        InputKind::Git { repository, reference } => resolve_git_input(input, resolver, repository, reference),
        InputKind::Darcs { repository, selector } => resolve_darcs_input(input, resolver, repository, selector),
        InputKind::Pijul { repository, selector } => resolve_pijul_input(input, resolver, repository, selector),
        InputKind::Fossil { repository, selector } => resolve_fossil_input(input, resolver, repository, selector),
    }
}

fn resolve_file_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    url: &str,
) -> Result<ResolvedLockKind, Error> {
    let hash_value = require_string_resolution(
        resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Flat)?,
        &format!("flat hash for {}", input.name),
    )?;
    let hash = locked_hash(input, hash_value);
    assert!(!hash.value.is_empty(), "resolved file hash must not be empty");
    assert_eq!(hash.algo, input.hash.algo, "resolved file hash must retain the declared algorithm");
    Ok((LockedKind::File { url: url.to_string() }, hash))
}

fn resolve_tarball_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    url: &str,
) -> Result<ResolvedLockKind, Error> {
    let hash_value = require_string_resolution(
        resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Recursive)?,
        &format!("tarball tree hash for {}", input.name),
    )?;
    let hash = locked_hash(input, hash_value);
    assert!(!hash.value.is_empty(), "resolved tarball hash must not be empty");
    assert_eq!(hash.algo, input.hash.algo, "resolved tarball hash must retain the declared algorithm");
    Ok((LockedKind::Tarball { url: url.to_string() }, hash))
}

fn resolve_git_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    repository: &str,
    reference: &GitReference,
) -> Result<ResolvedLockKind, Error> {
    let rev = require_string_resolution(
        resolver.resolve_git_rev(repository, reference)?,
        &format!("git revision for {}", input.name),
    )?;
    let hash_value = require_string_resolution(
        resolver.hash_git_checkout(repository, &rev, &input.hash.algo)?,
        &format!("git tree hash for {}", input.name),
    )?;
    assert!(!rev.is_empty(), "resolved git revision must not be empty");
    assert!(!hash_value.is_empty(), "resolved git tree hash must not be empty");
    Ok((
        LockedKind::Git {
            repository: repository.to_string(),
            rev,
            ref_name: git_reference_name(reference),
        },
        locked_hash(input, hash_value),
    ))
}

fn git_reference_name(reference: &GitReference) -> Option<String> {
    match reference {
        GitReference::Branch(branch) => Some(branch.clone()),
        GitReference::Tag(tag) => Some(tag.clone()),
        GitReference::Rev(_) => None,
    }
}

fn resolve_darcs_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    repository: &str,
    selector: &DarcsSelector,
) -> Result<ResolvedLockKind, Error> {
    let identity = require_resolution(
        resolver.resolve_darcs_identity(repository, selector)?,
        &format!("darcs identity for {}", input.name),
    )?;
    require_darcs_identity(&input.name, &identity)?;
    let hash_value = require_string_resolution(
        resolver.hash_darcs_checkout(repository, &identity, &input.hash.algo)?,
        &format!("darcs tree hash for {}", input.name),
    )?;
    assert_ne!(
        (identity.context.as_deref(), identity.weak_hash.as_deref()),
        (None, None),
        "darcs identity must have a stable selector"
    );
    assert!(!hash_value.is_empty(), "resolved darcs tree hash must not be empty");
    Ok((
        LockedKind::Darcs {
            repository: repository.to_string(),
            selector: selector.clone(),
            context: identity.context,
            weak_hash: identity.weak_hash,
        },
        locked_hash(input, hash_value),
    ))
}

fn resolve_pijul_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    repository: &str,
    selector: &PijulSelector,
) -> Result<ResolvedLockKind, Error> {
    let identity = require_resolution(
        resolver.resolve_pijul_identity(repository, selector)?,
        &format!("pijul identity for {}", input.name),
    )?;
    require_nonempty_identity(&input.name, IdentityFieldLabel::PijulState, &identity.state)?;
    let hash_value = require_string_resolution(
        resolver.hash_pijul_checkout(repository, &identity, &input.hash.algo)?,
        &format!("pijul tree hash for {}", input.name),
    )?;
    assert!(!identity.state.is_empty(), "resolved pijul state must not be empty");
    assert!(!hash_value.is_empty(), "resolved pijul tree hash must not be empty");
    Ok((
        LockedKind::Pijul {
            repository: repository.to_string(),
            selector: selector.clone(),
            state: identity.state,
            change: identity.change,
        },
        locked_hash(input, hash_value),
    ))
}

fn resolve_fossil_input(
    input: &ManifestInput,
    resolver: &dyn RefreshResolver,
    repository: &str,
    selector: &FossilSelector,
) -> Result<ResolvedLockKind, Error> {
    let identity = require_resolution(
        resolver.resolve_fossil_identity(repository, selector)?,
        &format!("fossil identity for {}", input.name),
    )?;
    require_nonempty_identity(&input.name, IdentityFieldLabel::FossilCheckin, &identity.checkin)?;
    let hash_value = require_string_resolution(
        resolver.hash_fossil_checkout(repository, &identity, &input.hash.algo)?,
        &format!("fossil tree hash for {}", input.name),
    )?;
    assert!(!identity.checkin.is_empty(), "resolved fossil check-in must not be empty");
    assert!(!hash_value.is_empty(), "resolved fossil tree hash must not be empty");
    Ok((
        LockedKind::Fossil {
            repository: repository.to_string(),
            selector: selector.clone(),
            checkin: identity.checkin,
        },
        locked_hash(input, hash_value),
    ))
}

fn locked_hash(input: &ManifestInput, value: String) -> LockedHash {
    LockedHash {
        algo: input.hash.algo.clone(),
        value,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityFieldLabel {
    DarcsContext,
    DarcsWeakHash,
    PijulState,
    FossilCheckin,
}

impl IdentityFieldLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::DarcsContext => "darcs context",
            Self::DarcsWeakHash => "darcs weak hash",
            Self::PijulState => "pijul state",
            Self::FossilCheckin => "fossil check-in",
        }
    }
}

fn require_darcs_identity(input_name: &str, identity: &ResolvedDarcsIdentity) -> Result<(), Error> {
    if identity.context.as_deref().unwrap_or("").is_empty() && identity.weak_hash.as_deref().unwrap_or("").is_empty() {
        return Err(Error::Manifest(format!(
            "input '{input_name}' darcs resolver did not prove context or weak-hash identity"
        )));
    }
    require_optional_nonempty_identity(input_name, IdentityFieldLabel::DarcsContext, &identity.context)?;
    require_optional_nonempty_identity(input_name, IdentityFieldLabel::DarcsWeakHash, &identity.weak_hash)
}

fn require_optional_nonempty_identity(
    input_name: &str,
    label: IdentityFieldLabel,
    value: &Option<String>,
) -> Result<(), Error> {
    if matches!(value, Some(text) if text.is_empty()) {
        return Err(Error::Manifest(format!("input '{input_name}' resolver returned empty {}", label.as_str())));
    }
    Ok(())
}

fn require_nonempty_identity(input_name: &str, label: IdentityFieldLabel, value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::Manifest(format!("input '{input_name}' resolver returned empty {}", label.as_str())));
    }
    Ok(())
}

fn resolve_needed_patches(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    outcomes: &[RefreshOutcome],
    resolver: &dyn RefreshResolver,
) -> (Vec<PatchResolution>, Vec<VerifiedTrustFact>) {
    let plan = crunch_project_core::plan_patch_resolutions(PatchResolutionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        outcomes: outcomes.to_vec(),
    });
    let mut trust_facts = Vec::new();
    let resolutions = plan
        .into_iter()
        .map(|patch_def| resolve_patch_def_with_trust(&patch_def, resolver, &mut trust_facts))
        .collect();
    (resolutions, trust_facts)
}

fn resolve_patch_def(def: &PatchDef, resolver: &dyn RefreshResolver) -> PatchResolution {
    match resolve_patch(def, resolver) {
        Ok(patch) => PatchResolution::Resolved {
            name: def.name.clone(),
            patch,
        },
        Err(err) => PatchResolution::Failed {
            name: def.name.clone(),
            reason: err.to_string(),
        },
    }
}

fn resolve_patch_def_with_trust(
    def: &PatchDef,
    resolver: &dyn RefreshResolver,
    trust_facts: &mut Vec<VerifiedTrustFact>,
) -> PatchResolution {
    let resolved = resolve_patch_def(def, resolver);
    let PatchResolution::Resolved { name, patch } = resolved else {
        return resolved;
    };
    if def.trust.is_none() {
        return PatchResolution::Resolved { name, patch };
    }
    match resolver.verify_patch_trust(def, &patch) {
        Ok(mut facts) => {
            trust_facts.append(&mut facts);
            PatchResolution::Resolved { name, patch }
        }
        Err(err) => PatchResolution::Failed {
            name,
            reason: err.to_string(),
        },
    }
}

fn resolve_patch(def: &PatchDef, resolver: &dyn RefreshResolver) -> Result<LockedPatch, Error> {
    match &def.source {
        PatchSource::Local { path } => {
            let hash_value = require_string_resolution(
                resolver.hash_local_file(path, &HashAlgo::Sha256)?,
                &format!("local patch hash for {path}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Local { path: path.clone() },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash_value,
                },
                trust: None,
            })
        }
        PatchSource::Remote { url, hash } => {
            let hash_value = require_string_resolution(
                resolver.hash_url_content(url, &hash.algo, HashResolutionMode::Flat)?,
                &format!("remote patch hash for {url}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Remote { url: url.clone() },
                hash: LockedHash {
                    algo: hash.algo.clone(),
                    value: hash_value,
                },
                trust: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::HashSpec;
    use crate::SchemaVersion;

    struct MockResolver {
        git_rev: Option<String>,
        git_hash: Option<String>,
        url_hash: Option<String>,
        local_hash: Option<String>,
        darcs_identity: Option<ResolvedDarcsIdentity>,
        darcs_hash: Option<String>,
        pijul_identity: Option<ResolvedPijulIdentity>,
        pijul_hash: Option<String>,
        fossil_identity: Option<ResolvedFossilIdentity>,
        fossil_hash: Option<String>,
    }

    impl RefreshResolver for MockResolver {
        fn resolve_git_rev(&self, _repository: &str, _reference: &GitReference) -> Result<Option<String>, Error> {
            Ok(self.git_rev.clone())
        }

        fn hash_url_content(
            &self,
            _url: &str,
            _algo: &HashAlgo,
            _mode: HashResolutionMode,
        ) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }

        fn hash_git_checkout(&self, _repository: &str, _rev: &str, _algo: &HashAlgo) -> Result<Option<String>, Error> {
            Ok(self.git_hash.clone())
        }

        fn hash_local_file(&self, _path: &str, _algo: &HashAlgo) -> Result<Option<String>, Error> {
            Ok(self.local_hash.clone())
        }

        fn resolve_darcs_identity(
            &self,
            _repository: &str,
            _selector: &DarcsSelector,
        ) -> Result<Option<ResolvedDarcsIdentity>, Error> {
            Ok(self.darcs_identity.clone())
        }

        fn hash_darcs_checkout(
            &self,
            _repository: &str,
            _identity: &ResolvedDarcsIdentity,
            _algo: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.darcs_hash.clone())
        }

        fn resolve_pijul_identity(
            &self,
            _repository: &str,
            _selector: &PijulSelector,
        ) -> Result<Option<ResolvedPijulIdentity>, Error> {
            Ok(self.pijul_identity.clone())
        }

        fn hash_pijul_checkout(
            &self,
            _repository: &str,
            _identity: &ResolvedPijulIdentity,
            _algo: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.pijul_hash.clone())
        }

        fn resolve_fossil_identity(
            &self,
            _repository: &str,
            _selector: &FossilSelector,
        ) -> Result<Option<ResolvedFossilIdentity>, Error> {
            Ok(self.fossil_identity.clone())
        }

        fn hash_fossil_checkout(
            &self,
            _repository: &str,
            _identity: &ResolvedFossilIdentity,
            _algo: &HashAlgo,
        ) -> Result<Option<String>, Error> {
            Ok(self.fossil_hash.clone())
        }
    }

    #[test]
    fn failed_freshness_observation_maps_empty_resolver_diagnostic() {
        let input = ManifestInput {
            name: "pkg".into(),
            kind: InputKind::File {
                url: "https://example.com/pkg".into(),
            },
            hash: HashSpec::default(),
            frozen: false,
            mirrors: vec![],
            patches: vec![],
            fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
            retention: None,
            freshness: Some(crunch_project_core::FreshnessProbe::LocalFile { path: "VERSION".into() }),
            trust: None,
        };

        let observation = failed_freshness_observation(&input, String::new());

        assert_eq!(observation.input_name, "pkg");
        assert_eq!(observation.status, crunch_project_core::FreshnessObservationStatus::Failed);
        assert!(observation.diagnostic.contains("without a diagnostic"));
    }

    #[test]
    fn identity_validation_preserves_missing_and_empty_behavior() {
        let missing = None;
        let empty = Some(String::new());

        assert!(require_optional_nonempty_identity("pkg", IdentityFieldLabel::DarcsContext, &missing).is_ok());
        assert!(matches!(
            require_optional_nonempty_identity("pkg", IdentityFieldLabel::DarcsWeakHash, &empty),
            Err(Error::Manifest(message)) if message == "input 'pkg' resolver returned empty darcs weak hash"
        ));
        assert!(matches!(
            require_nonempty_identity("pkg", IdentityFieldLabel::FossilCheckin, ""),
            Err(Error::Manifest(message)) if message == "input 'pkg' resolver returned empty fossil check-in"
        ));
    }

    #[test]
    fn shell_adapter_keeps_refresh_io_outside_core() {
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
                fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
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
            retention: crunch_project_core::InputRetentionPolicy::Untracked,
        };
        let resolver = MockResolver {
            git_rev: None,
            git_hash: None,
            url_hash: Some("sha256-data=".into()),
            local_hash: Some("sha256-patch=".into()),
            darcs_identity: None,
            darcs_hash: None,
            pijul_identity: None,
            pijul_hash: None,
            fossil_identity: None,
            fossil_hash: None,
        };
        let lock = Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        };

        let outcomes = refresh_inputs(&manifest, &lock, &[], &resolver);
        let result = apply_outcomes(&manifest, &lock, &outcomes, &resolver);

        assert!(matches!(&outcomes[0], RefreshOutcome::Updated(resolved) if resolved.name == "pkg"));
        assert_eq!(result.lock.inputs["pkg"].hash.value, "sha256-data=");
        assert_eq!(result.lock.patches["fix1"].hash.value, "sha256-patch=");
    }

    #[test]
    fn adapter_preserves_failed_resolution_behavior() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "broken".into(),
                kind: InputKind::File {
                    url: "https://example.com/broken".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
                fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
                retention: None,
                freshness: None,
                trust: None,
            }],
            patches: vec![],
            retention: crunch_project_core::InputRetentionPolicy::Untracked,
        };
        struct FailingResolver;
        impl RefreshResolver for FailingResolver {
            fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
            fn hash_url_content(&self, _: &str, _: &HashAlgo, _: HashResolutionMode) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
        }

        let outcomes = refresh_inputs(
            &manifest,
            &Lockfile {
                version: SchemaVersion::CURRENT,
                inputs: BTreeMap::new(),
                patches: BTreeMap::new(),
            },
            &[],
            &FailingResolver,
        );
        assert!(
            matches!(&outcomes[0], RefreshOutcome::Failed { name, reason } if name == "broken" && reason.contains("network down"))
        );
    }

    #[test]
    fn adapter_resolves_vcs_identities_and_tree_hashes() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![
                ManifestInput {
                    name: "darcs".into(),
                    kind: InputKind::Darcs {
                        repository: "https://example.invalid/darcs".into(),
                        selector: DarcsSelector::Tag("v1".into()),
                    },
                    hash: HashSpec::default(),
                    frozen: false,
                    mirrors: vec![],
                    patches: vec![],
                    fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
                    retention: None,
                    freshness: None,
                    trust: None,
                },
                ManifestInput {
                    name: "pijul".into(),
                    kind: InputKind::Pijul {
                        repository: "https://example.invalid/pijul".into(),
                        selector: PijulSelector::Channel { channel: "main".into() },
                    },
                    hash: HashSpec::default(),
                    frozen: false,
                    mirrors: vec![],
                    patches: vec![],
                    fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
                    retention: None,
                    freshness: None,
                    trust: None,
                },
                ManifestInput {
                    name: "fossil".into(),
                    kind: InputKind::Fossil {
                        repository: "https://example.invalid/fossil".into(),
                        selector: FossilSelector::Branch("trunk".into()),
                    },
                    hash: HashSpec::default(),
                    frozen: false,
                    mirrors: vec![],
                    patches: vec![],
                    fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
                    retention: None,
                    freshness: None,
                    trust: None,
                },
            ],
            patches: vec![],
            retention: crunch_project_core::InputRetentionPolicy::Untracked,
        };
        let resolver = MockResolver {
            git_rev: None,
            git_hash: None,
            url_hash: None,
            local_hash: None,
            darcs_identity: Some(ResolvedDarcsIdentity {
                context: Some("ctx".into()),
                weak_hash: None,
            }),
            darcs_hash: Some("blake3-darcs=".into()),
            pijul_identity: Some(ResolvedPijulIdentity {
                channel: "main".into(),
                state: "state".into(),
                change: None,
            }),
            pijul_hash: Some("blake3-pijul=".into()),
            fossil_identity: Some(ResolvedFossilIdentity {
                checkin: "checkin".into(),
            }),
            fossil_hash: Some("blake3-fossil=".into()),
        };

        let outcomes = refresh_inputs(&manifest, &Lockfile::new(), &[], &resolver);

        assert_eq!(outcomes.len(), 3);
        assert!(
            matches!(&outcomes[0], RefreshOutcome::Updated(resolved) if matches!(resolved.entry.kind, LockedKind::Darcs { .. }))
        );
        assert!(
            matches!(&outcomes[1], RefreshOutcome::Updated(resolved) if matches!(resolved.entry.kind, LockedKind::Pijul { .. }))
        );
        assert!(
            matches!(&outcomes[2], RefreshOutcome::Updated(resolved) if matches!(resolved.entry.kind, LockedKind::Fossil { .. }))
        );
    }

    #[test]
    fn default_vcs_hooks_fail_closed_with_unsupported_tool_diagnostic() {
        struct UnsupportedResolver;
        impl RefreshResolver for UnsupportedResolver {
            fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
                Ok(None)
            }
            fn hash_url_content(&self, _: &str, _: &HashAlgo, _: HashResolutionMode) -> Result<Option<String>, Error> {
                Ok(None)
            }
        }
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "darcs".into(),
                kind: InputKind::Darcs {
                    repository: "https://example.invalid/darcs".into(),
                    selector: DarcsSelector::Tag("v1".into()),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
                fetch_policy: crunch_project_core::InputFetchPolicy::GenerationMaterial,
                retention: None,
                freshness: None,
                trust: None,
            }],
            patches: vec![],
            retention: crunch_project_core::InputRetentionPolicy::Untracked,
        };

        let outcomes = refresh_inputs(&manifest, &Lockfile::new(), &[], &UnsupportedResolver);

        assert!(
            matches!(&outcomes[0], RefreshOutcome::Failed { reason, .. } if reason.contains("unsupported-VCS-tool"))
        );
    }
}
