//! Supported explicit-target package profiles; no resolution or PATH mutation.
mod admission;
mod publication;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

pub use admission::package_from_derivation;
use crunch_composition_core::user_profile::GENERATION_SCHEMA;
use crunch_composition_core::user_profile::Generation;
use crunch_composition_core::user_profile::Manifest;
use crunch_composition_core::user_profile::Package;
use crunch_store::GcRootClass;
use crunch_store::GcRootSource;
use crunch_store::PipelineStoreParts;
use crunch_store::RootRegistration;
use crunch_store::RootRegistry;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use crunch_store::StoreMutationGuard;
use nix_compat::narinfo::VerifyingKey;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum ProfileOperation {
    Install { package: Package, replace: bool },
    Remove { name: String },
    List,
    Generations,
    Switch { generation: u64 },
    Rollback,
    RemoveGeneration { generation: u64 },
}

pub struct ProfileRequest {
    pub profile: PathBuf,
    pub store: StoreConfig,
    pub trusted_keys: Vec<VerifyingKey>,
    /// Mandatory compare-and-swap for mutations: None means "must be empty".
    pub expected_active: Option<u64>,
    /// False inspects the complete admitted plan without publishing roots,
    /// generations, or the current pointer. Lock directories may be created.
    pub apply: bool,
    pub operation: ProfileOperation,
}

#[derive(Debug, Serialize)]
pub struct ProfileReport {
    pub schema: &'static str,
    pub profile: PathBuf,
    pub operation: &'static str,
    pub applied: bool,
    pub previous_active: Option<u64>,
    pub active: Option<u64>,
    pub target_generation: Option<u64>,
    pub retained_generations: Vec<u64>,
    pub pending_generations: Vec<u64>,
    pub manifest: Option<Manifest>,
    pub links: BTreeMap<String, String>,
    pub root_owner_label: Option<String>,
    pub admission_checked: bool,
}

/// Read a bounded exact package request for the thin CLI adapter.
pub fn read_package_request(path: &Path) -> Result<Package, String> {
    publication::read_optional(path)?.ok_or_else(|| format!("package request missing: {}", path.display()))
}

/// Own the store mutation lock until after profile roots and pointer are durable.
/// A separate profile lock rejects writers using another store state directory.
pub async fn execute_profile(mut request: ProfileRequest) -> Result<ProfileReport, String> {
    let platform = host_platform()?;
    request.store.preflight_backend_identity().map_err(|error| error.to_string())?;
    publication::require_directory(&request.store.state_dir, true)?;
    publication::require_directory(&request.store.output_dir, false)?;
    let store_guard = StoreMutationGuard::try_acquire(&request.store.state_dir).map_err(|error| error.to_string())?;
    if request.profile.starts_with(&request.store.output_dir)
        || request.store.output_dir.starts_with(&request.profile)
        || request.profile == request.store.state_dir
    {
        return Err("profile target must be separate from the physical output store and state root".into());
    }
    let _profile_guard = publication::lock_profile(&request.profile)?;
    request.store.state_dir = std::fs::canonicalize(&request.store.state_dir).map_err(|error| error.to_string())?;
    request.store.output_dir = std::fs::canonicalize(&request.store.output_dir).map_err(|error| error.to_string())?;
    request.profile = std::fs::canonicalize(&request.profile).map_err(|error| error.to_string())?;
    let binding = publication::Binding {
        schema: "mantle-user-profile-binding-v1".into(),
        state_dir: request.store.state_dir.clone(),
        output_dir: request.store.output_dir.clone(),
        store_prefix: request.store.store_dir.clone(),
        platform,
    };
    publication::bind(&request.profile, &binding, false)?;
    let active = publication::active(&request.profile)?;
    if is_mutation(&request.operation) && active != request.expected_active {
        return Err(format!("stale profile update: expected {:?}, active {active:?}", request.expected_active));
    }
    let retained = publication::generations(&request.profile)?;
    let mut result_document = initial_report(&request, active, retained);
    result_document.pending_generations = publication::pending_generations(&request.profile)?;
    if matches!(request.operation, ProfileOperation::List | ProfileOperation::Generations) {
        populate_read_only_view(&request.profile, active, &binding, &mut result_document)?;
        return Ok(result_document);
    }
    let should_apply = request.apply;
    let is_removing_generation = matches!(&request.operation, ProfileOperation::RemoveGeneration { .. });
    if should_apply {
        publication::bind(&request.profile, &binding, true)?;
    }
    let parts = open_pipeline_parts(request.store, &store_guard).await?;
    let authority = admission::Admission {
        lookup: &parts.output_lookup,
        content: &parts.build_service_store,
        output_dir: &binding.output_dir,
        trusted_keys: &request.trusted_keys,
    };
    let context = MutationContext {
        profile: &request.profile,
        binding: &binding,
        roots: &parts.root_registry,
        authority,
    };
    mutate(request.operation, should_apply, &context, &mut result_document).await?;
    // Observe publication while both ownership locks are still held. A
    // successful mutator return is not evidence that its requested pointer or
    // generation retention actually became the durable profile state.
    verify_published_state(&request.profile, &result_document, is_removing_generation)?;
    debug_assert_eq!(result_document.previous_active, active);
    debug_assert_eq!(result_document.applied, should_apply);
    Ok(result_document)
}

/// Open the store under the held mutation guard and recover its GC state.
async fn open_pipeline_parts(
    store_config: StoreConfig,
    guard: &StoreMutationGuard,
) -> Result<PipelineStoreParts, String> {
    let mut store = StoreHandle::open(store_config).await.map_err(|error| error.to_string())?;
    store.recover_casita_gc_under_guard(guard).await.map_err(|error| error.to_string())?;
    let parts = store.into_pipeline_store_parts();
    debug_assert!(!parts.output_lookup.store_dir().is_empty());
    Ok(parts)
}

/// Report the admitted active generation without publishing anything.
fn populate_read_only_view(
    profile: &Path,
    active: Option<u64>,
    binding: &publication::Binding,
    result_document: &mut ProfileReport,
) -> Result<(), String> {
    if let Some(number) = active {
        let generation = publication::read_generation(profile, number)?;
        require_binding(&generation.manifest, binding)?;
        result_document.manifest = Some(generation.manifest);
        result_document.links = generation.links;
    }
    debug_assert_eq!(result_document.active, active);
    debug_assert!(result_document.manifest.is_some() == active.is_some());
    Ok(())
}

/// Confirm the durable profile state matches the admitted operation.
fn verify_published_state(
    profile: &Path,
    result_document: &ProfileReport,
    is_removing_generation: bool,
) -> Result<(), String> {
    let observed_active = publication::active(profile)?;
    let observed_retained = publication::generations(profile)?;
    let observed_pending = publication::pending_generations(profile)?;
    if result_document.active != observed_active
        || result_document.retained_generations != observed_retained
        || result_document.pending_generations != observed_pending
    {
        return Err("profile publication observation differs from the admitted operation".into());
    }
    if result_document.applied && !is_removing_generation {
        let number = observed_active.ok_or("profile publication omitted its active generation")?;
        let generation = publication::read_generation(profile, number)?;
        publication::verify_forest(profile, &generation)?;
        if result_document.manifest.as_ref() != Some(&generation.manifest) || result_document.links != generation.links
        {
            return Err("published profile composition differs from the admitted operation".into());
        }
    }
    debug_assert!(observed_retained.len() <= publication::GENERATIONS_MAX);
    debug_assert!(observed_pending.iter().all(|number| *number > 0));
    Ok(())
}

struct MutationContext<'a> {
    profile: &'a Path,
    binding: &'a publication::Binding,
    roots: &'a RootRegistry,
    authority: admission::Admission<'a>,
}

async fn mutate(
    operation: ProfileOperation,
    should_apply: bool,
    context: &MutationContext<'_>,
    result_document: &mut ProfileReport,
) -> Result<(), String> {
    if let ProfileOperation::RemoveGeneration { generation } = operation {
        return remove_generation(context, generation, should_apply, result_document);
    }
    let (generation, is_new) = select_generation(operation, context, result_document)?;
    require_binding(&generation.manifest, context.binding)?;
    let links = admission::admit(&generation.manifest, &context.authority).await?;
    let mut generation = generation;
    if is_new {
        generation.links = links;
    } else {
        if generation.links != links {
            return Err("retained profile composition differs from admitted outputs".into());
        }
        publication::verify_forest(context.profile, &generation)?;
    }
    if serde_json::to_vec(&generation).map_err(|error| error.to_string())?.len() as u64 > publication::JSON_BYTES_MAX {
        return Err("profile generation manifest exceeds publication byte bound".into());
    }
    result_document.target_generation = Some(generation.number);
    result_document.root_owner_label = Some(owner_label(context.profile, generation.number));
    result_document.manifest = Some(generation.manifest.clone());
    result_document.links = generation.links.clone();
    result_document.admission_checked = true;
    if !should_apply {
        return Ok(());
    }
    if is_new {
        publication::allocate(context.profile, generation.number)?;
        let stage = publication::stage(context.profile, &generation)?;
        publication::record_pending(context.profile, &generation)?;
        retain(context, &generation).await?;
        publication::publish(context.profile, stage, generation.number)?;
        publication::finish_pending(context.profile, generation.number)?;
    } else {
        retain(context, &generation).await?;
    }
    publication::switch(context.profile, generation.number)?;
    result_document.applied = true;
    result_document.active = Some(generation.number);
    if is_new {
        result_document.retained_generations.push(generation.number);
        result_document.retained_generations.sort_unstable();
    }
    assert_eq!(result_document.active, result_document.target_generation);
    assert!(result_document.admission_checked);
    Ok(())
}

fn select_generation(
    operation: ProfileOperation,
    context: &MutationContext<'_>,
    result_document: &ProfileReport,
) -> Result<(Generation, bool), String> {
    let current = result_document
        .active
        .map(|number| publication::read_generation(context.profile, number))
        .transpose()?;
    let manifest = current
        .as_ref()
        .map(|generation| generation.manifest.clone())
        .unwrap_or_else(|| Manifest::empty(context.binding.platform.clone(), context.binding.store_prefix.clone()));
    require_binding(&manifest, context.binding)?;
    let candidate = match operation {
        ProfileOperation::Install { package, replace } => {
            manifest.install(package, replace).map_err(|error| error.to_string())?
        }
        ProfileOperation::Remove { name } => manifest.remove(&name).map_err(|error| error.to_string())?,
        ProfileOperation::Switch { generation } => {
            return Ok((publication::read_generation(context.profile, generation)?, false));
        }
        ProfileOperation::Rollback => {
            let active = result_document.active.ok_or("cannot roll back an empty profile")?;
            let previous = result_document
                .retained_generations
                .iter()
                .rev()
                .find(|number| **number < active)
                .copied()
                .ok_or("no previous retained generation")?;
            return Ok((publication::read_generation(context.profile, previous)?, false));
        }
        _ => return Err("invalid profile mutation operation".into()),
    };
    if result_document.retained_generations.len() >= publication::GENERATIONS_MAX {
        return Err("profile retained generation limit reached; remove an inactive generation separately".into());
    }
    let number = publication::next_number(context.profile, &result_document.retained_generations)?;
    assert!(number > result_document.retained_generations.last().copied().unwrap_or(0));
    assert_eq!(candidate.platform, manifest.platform);
    Ok((
        Generation {
            schema: GENERATION_SCHEMA.into(),
            number,
            parent: result_document.active,
            manifest: candidate,
            links: BTreeMap::new(),
        },
        true,
    ))
}

async fn retain(context: &MutationContext<'_>, generation: &Generation) -> Result<(), String> {
    let label = owner_label(context.profile, generation.number);
    let bytes = serde_json::to_vec(generation).map_err(|error| error.to_string())?;
    let identity = format!("b3:{}", blake3::hash(&bytes).to_hex());
    let mut unique = BTreeSet::new();
    let mut registrations = Vec::with_capacity(generation.manifest.packages.len());
    for package in &generation.manifest.packages {
        if !unique.insert(&package.output_path) {
            continue;
        }
        let path = admission::parse_path(&package.output_path, admission::StorePrefix(&context.binding.store_prefix))?;
        registrations.push((path, GcRootSource::Pin, RootRegistration {
            class: GcRootClass::ExplicitPin,
            owner_scope: format!("operator:{label}"),
            project_identity: None,
            selector: None,
            generation: Some(generation.number),
            generation_identity: Some(identity.clone()),
            lease: None,
            removal_requested: false,
        }));
    }
    let records = context.roots.register_managed_batch(registrations).await.map_err(|error| error.to_string())?;
    assert_eq!(records.len(), unique.len());
    assert!(records.iter().all(|record| record.owner_scope == format!("operator:{label}")));
    Ok(())
}

fn remove_generation(
    context: &MutationContext<'_>,
    number: u64,
    should_apply: bool,
    result_document: &mut ProfileReport,
) -> Result<(), String> {
    if number == 0 {
        return Err("profile generation numbers must be positive".into());
    }
    if result_document.active == Some(number) {
        return Err("cannot remove the active generation; switch separately first".into());
    }
    let generation = if result_document.retained_generations.contains(&number) {
        publication::read_generation(context.profile, number)?
    } else {
        let generation: Generation =
            match publication::read_optional(&context.profile.join(format!(".removed-{number}/manifest.json")))? {
                Some(generation) => generation,
                None => publication::read_optional(&context.profile.join(format!(".pending-{number}.json")))?
                    .ok_or("retained or pending generation not found")?,
            };
        if generation.number != number || generation.schema != GENERATION_SCHEMA {
            return Err("invalid removal tombstone".into());
        }
        generation
    };
    require_binding(&generation.manifest, context.binding)?;
    let label = owner_label(context.profile, number);
    result_document.target_generation = Some(number);
    result_document.root_owner_label = Some(label.clone());
    result_document.manifest = Some(generation.manifest.clone());
    result_document.links = generation.links.clone();
    if !should_apply {
        return Ok(());
    }
    let removed = publication::tombstone(context.profile, number)?;
    for package in &removed.manifest.packages {
        context.roots.release_owned_pin(&package.output_path, &label).map_err(|error| error.to_string())?;
    }
    publication::finish_pending(context.profile, number)?;
    result_document.pending_generations.retain(|pending| *pending != number);
    result_document.applied = true;
    result_document.retained_generations = publication::generations(context.profile)?;
    assert_eq!(result_document.active, result_document.previous_active);
    assert!(!result_document.retained_generations.contains(&number));
    Ok(())
}

fn require_binding(manifest: &Manifest, binding: &publication::Binding) -> Result<(), String> {
    manifest.validate().map_err(|error| error.to_string())?;
    if manifest.platform != binding.platform || manifest.store_prefix != binding.store_prefix {
        return Err("profile generation belongs to a different platform/store prefix".into());
    }
    assert_eq!(manifest.platform, binding.platform);
    assert_eq!(manifest.store_prefix, binding.store_prefix);
    Ok(())
}

fn owner_label(profile: &Path, generation: u64) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle.user-profile.owner.v1\0");
    hash.update(profile.as_os_str().as_encoded_bytes());
    hash.update(&generation.to_le_bytes());
    let label = format!("profile-{}", &hash.finalize().to_hex()[..48]);
    assert!(label.len() <= 64);
    assert!(generation > 0);
    label
}

fn host_platform() -> Result<String, String> {
    if !cfg!(target_os = "linux") {
        return Err("user profile publication is supported on Linux only".into());
    }
    match std::env::consts::ARCH {
        "x86_64" => Ok("x86_64-linux".into()),
        "aarch64" => Ok("aarch64-linux".into()),
        _ => Err("user profiles require x86_64-linux or aarch64-linux".into()),
    }
}

fn is_mutation(operation: &ProfileOperation) -> bool {
    !matches!(operation, ProfileOperation::List | ProfileOperation::Generations)
}

fn initial_report(request: &ProfileRequest, active: Option<u64>, retained: Vec<u64>) -> ProfileReport {
    ProfileReport {
        schema: "mantle-user-profile-report-v1",
        profile: request.profile.clone(),
        operation: match request.operation {
            ProfileOperation::Install { .. } => "install",
            ProfileOperation::Remove { .. } => "remove",
            ProfileOperation::List => "list",
            ProfileOperation::Generations => "generations",
            ProfileOperation::Switch { .. } => "switch",
            ProfileOperation::Rollback => "rollback",
            ProfileOperation::RemoveGeneration { .. } => "remove-generation",
        },
        applied: false,
        previous_active: active,
        active,
        target_generation: None,
        retained_generations: retained,
        manifest: None,
        links: BTreeMap::new(),
        root_owner_label: None,
        admission_checked: false,
        pending_generations: Vec::new(),
    }
}
