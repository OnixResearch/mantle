// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::too_many_parameters
)]

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use data_encoding::HEXLOWER;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::retention::GcRootClass;
use crate::retention::GcRootLease;
use crate::retention::RootRegistration;
use crate::retention::core_retention_policy;
use crate::retention::store_retention_policy_blake3;
use crate::retention::store_retention_runtime_policy;

const ROOTS_FILE_NAME: &str = "gc-roots.json";
const ROOT_RECORD_SCHEMA_VERSION: u32 = 3;
const PREVIOUS_ROOT_RECORD_SCHEMA_VERSION: u32 = 2;
const LEGACY_OWNER_SCOPE: &str = "legacy-unmanaged";
const OPERATOR_OWNER_SCOPE: &str = "operator";
const REMOTE_OWNER_SCOPE: &str = "remote";
const SYSTEM_OWNER_SCOPE: &str = "system";
const TRANSITION_DIGEST_HEX_BYTES: usize = 64;
const TRANSITION_DOMAIN: &[u8] = b"mantle.root.transition.v1";

#[derive(Clone, Copy)]
pub struct LogicalStorePathRef<'a> {
    pub logical_path: &'a str,
    pub store_dir: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GcRootSource {
    Build,
    Source,
    Remote,
    SelfBuild,
    /// `mantle bootstrap --fetch` retained root.
    Bootstrap,
    Pin,
}

impl GcRootSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Source => "source",
            Self::Remote => "remote",
            Self::SelfBuild => "self-build",
            Self::Bootstrap => "bootstrap",
            Self::Pin => "pin",
        }
    }

    #[must_use]
    pub const fn for_remote_result(self) -> Self {
        match self {
            Self::Build | Self::Source | Self::Remote => Self::Remote,
            Self::SelfBuild => Self::SelfBuild,
            Self::Bootstrap => Self::Bootstrap,
            Self::Pin => Self::Pin,
        }
    }
}

impl std::fmt::Display for GcRootSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

// r[impl store_lifecycle.root_provenance]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GcRootRecord {
    pub schema_version: u32,
    pub logical_path: String,
    pub source: GcRootSource,
    pub root_class: GcRootClass,
    pub owner_scope: String,
    pub project_identity: Option<String>,
    pub selector: Option<String>,
    pub generation: Option<u64>,
    pub generation_identity: Option<String>,
    pub lease: Option<GcRootLease>,
    pub policy_blake3: String,
    pub created_unix_s: i64,
    pub last_transition_id: String,
    pub last_transition_reason: String,
    pub removal_requested: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGcRootRecord {
    #[serde(default)]
    schema_version: u32,
    logical_path: String,
    source: GcRootSource,
    #[serde(default)]
    root_class: Option<GcRootClass>,
    #[serde(default)]
    owner_scope: Option<String>,
    #[serde(default)]
    project_identity: Option<String>,
    #[serde(default)]
    selector: Option<String>,
    #[serde(default)]
    generation: Option<u64>,
    #[serde(default)]
    generation_identity: Option<String>,
    #[serde(default)]
    lease: Option<GcRootLease>,
    #[serde(default)]
    policy_blake3: Option<String>,
    created_unix_s: i64,
    #[serde(default)]
    last_transition_id: Option<String>,
    #[serde(default)]
    last_transition_reason: Option<String>,
    #[serde(default)]
    removal_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcInterestFact {
    pub owner: String,
    pub reason: String,
    pub record_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcPathInterests {
    pub logical_path: String,
    pub record_count: u32,
    pub legacy_unmanaged: bool,
    pub interests: Vec<GcInterestFact>,
}

pub fn list_interest_reports(state_dir: &Path) -> Result<Vec<GcPathInterests>, Error> {
    let legacy = load_registry(state_dir)?;
    let mut by_path = BTreeMap::<String, GcPathInterests>::new();
    for (path, root) in legacy {
        by_path.insert(path.clone(), GcPathInterests {
            logical_path: path,
            record_count: 0,
            legacy_unmanaged: root.root_class == GcRootClass::LegacyUnmanaged,
            interests: vec![GcInterestFact {
                owner: root.owner_scope,
                reason: root.last_transition_reason,
                record_id: None,
            }],
        });
    }
    for (id, record) in crate::interest_store::load(state_dir)? {
        let entry = by_path.entry(record.logical_path.clone()).or_insert_with(|| GcPathInterests {
            logical_path: record.logical_path.clone(),
            record_count: 0,
            legacy_unmanaged: false,
            interests: Vec::new(),
        });
        entry.record_count = entry
            .record_count
            .checked_add(1)
            .ok_or_else(|| Error::RootRegistry("interest count overflow".to_string()))?;
        entry.interests.push(GcInterestFact {
            owner: record.owner,
            reason: record.reason,
            record_id: Some(id),
        });
    }
    for entry in by_path.values_mut() {
        entry
            .interests
            .sort_by(|a, b| (&a.owner, &a.reason, &a.record_id).cmp(&(&b.owner, &b.reason, &b.record_id)));
    }
    Ok(by_path.into_values().collect())
}

// r[impl mantle.store_lifecycle.retention_interest_records]
pub fn list_roots(state_dir: &Path) -> Result<Vec<GcRootRecord>, Error> {
    let mut registry = load_registry(state_dir)?;
    let loaded = crate::interest_store::load(state_dir)?;
    merge_loaded_roots(&mut registry, &loaded)?;
    Ok(registry.into_values().collect())
}

fn merge_loaded_roots(
    registry: &mut BTreeMap<String, GcRootRecord>,
    loaded: &[(String, crate::interest_store::InterestRecord)],
) -> Result<(), Error> {
    let mut groups = BTreeMap::<&str, Vec<(&str, &crate::interest_store::InterestRecord)>>::new();
    for (id, record) in loaded {
        groups.entry(record.logical_path.as_str()).or_default().push((id.as_str(), record));
    }
    let needs_policy_selection =
        groups.values().any(|interests| interests.len() > 1) || groups.keys().any(|path| registry.contains_key(*path));
    let retained = if needs_policy_selection {
        retained_interest_ids(registry, loaded)?
    } else {
        BTreeMap::new()
    };
    for (path, interests) in groups {
        let selected = interests
            .iter()
            .min_by_key(|(id, record)| {
                (
                    !retained.get(*id).copied().unwrap_or(true),
                    root_priority(record.declaration.root_class),
                    record.owner.as_str(),
                    record.reason.as_str(),
                    *id,
                )
            })
            .ok_or_else(|| Error::RootRegistry("empty retention interest group".to_string()))?;
        if let Some(legacy) = registry.get(path) {
            let legacy_kept = retained.get(path).copied().unwrap_or(true);
            let selected_kept = retained.get(selected.0).copied().unwrap_or(true);
            if (legacy_kept && !selected_kept)
                || (legacy_kept == selected_kept
                    && root_priority(legacy.root_class) <= root_priority(selected.1.declaration.root_class))
            {
                continue;
            }
        }
        registry.insert(path.to_owned(), selected.1.declaration.clone());
    }
    if registry.len() > store_retention_runtime_policy().limits.max_roots {
        return Err(Error::RootRegistry("merged root count exceeds policy limit".to_string()));
    }
    Ok(())
}

fn retained_interest_ids(
    legacy: &BTreeMap<String, GcRootRecord>,
    interests: &[(String, crate::interest_store::InterestRecord)],
) -> Result<BTreeMap<String, bool>, Error> {
    let declarations = legacy
        .values()
        .cloned()
        .chain(interests.iter().map(|(_, record)| record.declaration.clone()))
        .collect::<Vec<_>>();
    let mut facts = crate::retention::records_to_core(&declarations)?;
    for (fact, (id, _)) in facts.iter_mut().skip(legacy.len()).zip(interests) {
        fact.path_id = format!("/retention-interest/{id}");
    }
    let plan = crunch_gc_core::retention::plan_retention(&core_retention_policy(), current_unix_seconds()?, facts)
        .map_err(|error| Error::RootRegistry(format!("merging retention interest decisions: {error:?}")))?;
    Ok(plan
        .decisions
        .into_iter()
        .map(|decision| {
            let id = decision
                .path_id
                .strip_prefix("/retention-interest/")
                .unwrap_or(decision.path_id.as_str())
                .to_owned();
            (id, decision.disposition.retains_path())
        })
        .collect())
}

fn root_priority(class: GcRootClass) -> u8 {
    match class {
        GcRootClass::LegacyUnmanaged => 0,
        GcRootClass::ExplicitPin => 1,
        GcRootClass::Bootstrap => 2,
        GcRootClass::SelfBuild => 3,
        GcRootClass::RemoteResult => 4,
        GcRootClass::ProjectOutputGeneration => 5,
        GcRootClass::ProjectSourceGeneration => 6,
        GcRootClass::ActiveShellLease => 7,
    }
}

// A partially replaced owner can have multiple transition reasons. Resolve
// renewal against its newest declaration, with a stable identity tie-break.
fn root_revision_rank(record: &GcRootRecord) -> (i64, u32, u64, &str) {
    (
        record.created_unix_s,
        record.lease.as_ref().map_or(0, |lease| lease.renewal_count),
        record.generation.unwrap_or(0),
        &record.last_transition_id,
    )
}

pub fn migrate_legacy_registry(state_dir: &Path) -> Result<Vec<GcRootRecord>, Error> {
    ensure_root_mutation_allowed(state_dir)?;
    let registry = load_registry(state_dir)?;
    let path = roots_path(state_dir);
    let archive = state_dir.join("gc-roots.migrated.json");
    if path.exists() {
        match std::fs::symlink_metadata(&archive) {
            Ok(_) => {
                return Err(Error::RootRegistry(format!("legacy root archive already exists: {}", archive.display())));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Error::RootRegistry(format!("observing {}: {error}", archive.display()))),
        }
    }
    for record in registry.values() {
        crate::interest_store::publish(state_dir, &crate::interest_store::InterestRecord::new(record.clone()))?;
    }
    if path.exists() {
        std::fs::rename(&path, &archive)
            .map_err(|error| Error::RootRegistry(format!("archiving {}: {error}", path.display())))?;
        std::fs::File::open(state_dir)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| Error::RootRegistry(format!("syncing {}: {error}", state_dir.display())))?;
    }
    list_roots(state_dir)
}

pub async fn register_root(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    store_path: &StorePath<String>,
    source: GcRootSource,
) -> Result<GcRootRecord, Error> {
    let registration = registration_for_source(source);
    register_root_with_registration(state_dir, store_dir, pathinfo, store_path, source, registration).await
}

pub async fn register_root_with_registration(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    store_path: &StorePath<String>,
    source: GcRootSource,
    registration: RootRegistration,
) -> Result<GcRootRecord, Error> {
    let records = register_root_batch_with_registration(state_dir, store_dir, pathinfo, vec![(
        store_path.clone(),
        source,
        registration,
    )])
    .await?;
    records
        .into_iter()
        .next()
        .ok_or_else(|| Error::RootRegistry("single root registration produced no record".to_string()))
}

pub(crate) async fn register_root_batch_with_registration(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    registrations: Vec<(StorePath<String>, GcRootSource, RootRegistration)>,
) -> Result<Vec<GcRootRecord>, Error> {
    assert!(!store_dir.is_empty(), "store_dir must not be empty");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    if registrations.is_empty() {
        return Ok(Vec::new());
    }
    let max_roots = store_retention_runtime_policy().limits.max_roots;
    if registrations.len() > max_roots {
        return Err(Error::RootRegistry(format!("root registration batch exceeds policy limit {max_roots}")));
    }
    for (store_path, _, _) in &registrations {
        ensure_pathinfo_exists(pathinfo, store_path).await?;
    }

    let created_unix_s = current_unix_seconds()?;
    let legacy = load_registry(state_dir)?;
    let interests = crate::interest_store::load(state_dir)?;
    let mut by_owner = BTreeMap::<(&str, &str), &GcRootRecord>::new();
    for record in legacy.values().chain(interests.iter().map(|(_, interest)| &interest.declaration)) {
        by_owner
            .entry((record.logical_path.as_str(), record.owner_scope.as_str()))
            .and_modify(|prior| {
                if root_revision_rank(record) > root_revision_rank(prior) {
                    *prior = record;
                }
            })
            .or_insert(record);
    }
    // Keep the unmerged legacy facts available for owner-specific renewal.
    let mut registry = legacy.clone();
    merge_loaded_roots(&mut registry, &interests)?;
    let mut records = Vec::with_capacity(registrations.len());
    for (store_path, source, registration) in registrations {
        let logical_path = store_path.to_absolute_path_with_prefix(store_dir);
        if records.iter().any(|record: &GcRootRecord| record.logical_path == logical_path) {
            return Err(Error::RootRegistry(format!("duplicate path in root registration batch: {logical_path}")));
        }
        let prior = by_owner.get(&(logical_path.as_str(), registration.owner_scope.as_str())).copied();
        let registration = resolve_project_generation(by_owner.values().copied().chain(records.iter()), registration)?;
        let registration = resolve_shell_lease_renewal(prior, registration)?;
        let transition_reason = registration_transition_reason(prior, &registration);
        let record = new_record(logical_path.clone(), source, registration, created_unix_s, transition_reason);
        validate_record(&record)?;
        registry.insert(logical_path, record.clone());
        records.push(record);
    }
    if registry.len() > max_roots {
        return Err(Error::RootRegistry(format!("root registry exceeds policy limit {max_roots}")));
    }
    for record in &records {
        let interest = crate::interest_store::InterestRecord::new(record.clone());
        if record.root_class == GcRootClass::ExplicitPin {
            crate::interest_store::publish(state_dir, &interest)?;
        } else {
            crate::interest_store::publish_replacing_owner(state_dir, &interest)?;
        }
    }
    Ok(records)
}

/// Operator labels are local declarations, not authenticated credentials.
pub fn operator_owner(label: &str) -> Result<String, Error> {
    if label == OPERATOR_OWNER_SCOPE {
        return Ok(OPERATOR_OWNER_SCOPE.to_string());
    }
    if label.is_empty()
        || label.len() > 64
        || !label.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(Error::RootRegistry("operator owner label is invalid or exceeds 64 bytes".to_string()));
    }
    Ok(format!("operator:{label}"))
}

pub async fn pin_root_for_owner(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    logical_path: &str,
    owner_label: &str,
    reason: &str,
) -> Result<GcRootRecord, Error> {
    if reason.is_empty() || reason.len() > 256 || reason.chars().any(char::is_control) {
        return Err(Error::RootRegistry("pin reason is empty, invalid, or exceeds 256 bytes".to_string()));
    }
    let owner = operator_owner(owner_label)?;
    let store_path = parse_logical_store_path(LogicalStorePathRef {
        logical_path,
        store_dir,
    })?;
    ensure_pathinfo_exists(pathinfo, &store_path).await?;
    let timestamp = current_unix_seconds()?;
    let registration = RootRegistration {
        owner_scope: owner,
        ..registration_for_source(GcRootSource::Pin)
    };
    let root = new_record(
        store_path.to_absolute_path_with_prefix(store_dir),
        GcRootSource::Pin,
        registration,
        timestamp,
        reason,
    );
    crate::interest_store::publish(state_dir, &crate::interest_store::InterestRecord::new(root.clone()))?;
    Ok(root)
}

pub fn unpin_root_for_owner_reason(
    state_dir: &Path,
    path: LogicalStorePathRef<'_>,
    owner: &str,
    reason: Option<&str>,
) -> Result<Option<GcRootRecord>, Error> {
    let normalized_path = normalize_logical_path(path)?;
    crate::interest_store::release(state_dir, owner, &normalized_path, reason)
}

pub(crate) fn load_registry(state_dir: &Path) -> Result<BTreeMap<String, GcRootRecord>, Error> {
    let path = roots_path(state_dir);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => {
            return Err(Error::RootRegistry(format!("reading {}: {error}", path.display())));
        }
    };
    let raw_registry: BTreeMap<String, RawGcRootRecord> = serde_json::from_slice(&bytes)
        .map_err(|error| Error::RootRegistry(format!("parsing {}: {error}", path.display())))?;
    let mut registry = BTreeMap::new();
    for (key, raw) in raw_registry {
        let record = normalize_raw_record(raw)?;
        if registry.insert(key, record).is_some() {
            return Err(Error::RootRegistry("duplicate normalized root path".to_string()));
        }
    }
    validate_registry(&registry)?;
    Ok(registry)
}

pub(crate) fn roots_path(state_dir: &Path) -> PathBuf {
    state_dir.join(ROOTS_FILE_NAME)
}

pub(crate) fn parse_logical_store_path(path: LogicalStorePathRef<'_>) -> Result<StorePath<String>, Error> {
    StorePath::from_absolute_path_with_prefix(path.logical_path.as_bytes(), path.store_dir).map_err(|_| {
        Error::RootRegistry(format!(
            "path is not under configured store prefix {}: {}",
            path.store_dir, path.logical_path
        ))
    })
}

fn normalize_logical_path(path: LogicalStorePathRef<'_>) -> Result<String, Error> {
    let store_path = parse_logical_store_path(path)?;
    Ok(store_path.to_absolute_path_with_prefix(path.store_dir))
}

fn resolve_project_generation<'a>(
    records: impl Iterator<Item = &'a GcRootRecord> + Clone,
    mut registration: RootRegistration,
) -> Result<RootRegistration, Error> {
    if !matches!(registration.class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration)
        || registration.generation.is_some()
    {
        return Ok(registration);
    }
    let (Some(project_identity), Some(selector), Some(generation_identity)) = (
        registration.project_identity.as_deref(),
        registration.selector.as_deref(),
        registration.generation_identity.as_deref(),
    ) else {
        return Err(Error::RootRegistry(
            "project registration requires project, selector, and generation identities".to_string(),
        ));
    };
    if let Some(existing) = records.clone().find(|record| {
        record.owner_scope == registration.owner_scope
            && record.root_class == registration.class
            && record.project_identity.as_deref() == Some(project_identity)
            && record.selector.as_deref() == Some(selector)
            && record.generation_identity.as_deref() == Some(generation_identity)
    }) {
        registration.generation = existing.generation;
        return Ok(registration);
    }
    let latest_generation = records
        .filter(|record| {
            record.owner_scope == registration.owner_scope
                && record.root_class == registration.class
                && record.project_identity.as_deref() == Some(project_identity)
                && record.selector.as_deref() == Some(selector)
        })
        .filter_map(|record| record.generation)
        .max()
        .unwrap_or(0);
    registration.generation = Some(
        latest_generation
            .checked_add(1)
            .ok_or_else(|| Error::RootRegistry("project root generation overflowed u64".to_string()))?,
    );
    Ok(registration)
}

fn resolve_shell_lease_renewal(
    existing: Option<&GcRootRecord>,
    mut registration: RootRegistration,
) -> Result<RootRegistration, Error> {
    if registration.class != GcRootClass::ActiveShellLease {
        return Ok(registration);
    }
    let Some(lease) = registration.lease.as_mut() else {
        return Err(Error::RootRegistry("active shell registration requires lease facts".to_string()));
    };
    let Some(existing) = existing else {
        return Ok(registration);
    };
    if existing.root_class != GcRootClass::ActiveShellLease
        || existing.project_identity != registration.project_identity
        || existing.selector != registration.selector
        || existing.owner_scope != registration.owner_scope
    {
        return Ok(registration);
    }
    let Some(existing_lease) = existing.lease.as_ref() else {
        return Err(Error::RootRegistry("persisted shell root has no lease facts".to_string()));
    };
    lease.lease_id.clone_from(&existing_lease.lease_id);
    let renewal_count = existing_lease
        .renewal_count
        .checked_add(1)
        .ok_or_else(|| Error::RootRegistry("shell lease renewal count overflowed u32".to_string()))?;
    let max_renewals = store_retention_runtime_policy().limits.max_lease_renewals;
    if renewal_count > max_renewals {
        return Err(Error::RootRegistry(format!("shell lease renewal exceeds policy limit {max_renewals}")));
    }
    lease.renewal_count = renewal_count;
    Ok(registration)
}

fn registration_transition_reason(existing: Option<&GcRootRecord>, registration: &RootRegistration) -> &'static str {
    let Some(existing) = existing else {
        return match registration.class {
            GcRootClass::ProjectOutputGeneration => "project-output-generation-registered",
            GcRootClass::ProjectSourceGeneration => "project-source-generation-registered",
            GcRootClass::ActiveShellLease => "shell-lease-registered",
            GcRootClass::ExplicitPin => "explicit-pin-registered",
            GcRootClass::Bootstrap => "bootstrap-root-registered",
            GcRootClass::SelfBuild => "self-build-root-registered",
            GcRootClass::RemoteResult => "remote-result-registered",
            GcRootClass::LegacyUnmanaged => "legacy-root-registered",
        };
    };
    if registration.class == GcRootClass::ActiveShellLease
        && existing.root_class == GcRootClass::ActiveShellLease
        && existing.project_identity == registration.project_identity
        && existing.selector == registration.selector
    {
        return "shell-lease-renewed";
    }
    if matches!(registration.class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration)
        && existing.root_class == registration.class
        && existing.project_identity == registration.project_identity
        && existing.selector == registration.selector
    {
        if existing.generation_identity == registration.generation_identity {
            return "project-generation-reobserved";
        }
        return "project-generation-advanced";
    }
    "root-reclassified"
}

fn registration_for_source(source: GcRootSource) -> RootRegistration {
    match source {
        GcRootSource::Pin => RootRegistration {
            class: GcRootClass::ExplicitPin,
            owner_scope: OPERATOR_OWNER_SCOPE.to_string(),
            project_identity: None,
            selector: None,
            generation: None,
            generation_identity: None,
            lease: None,
            removal_requested: false,
        },
        GcRootSource::SelfBuild => durable_system_registration(GcRootClass::SelfBuild),
        GcRootSource::Bootstrap => durable_system_registration(GcRootClass::Bootstrap),
        GcRootSource::Remote => RootRegistration {
            class: GcRootClass::RemoteResult,
            owner_scope: REMOTE_OWNER_SCOPE.to_string(),
            project_identity: None,
            selector: None,
            generation: None,
            generation_identity: None,
            lease: None,
            removal_requested: false,
        },
        GcRootSource::Build | GcRootSource::Source => RootRegistration {
            class: GcRootClass::LegacyUnmanaged,
            owner_scope: LEGACY_OWNER_SCOPE.to_string(),
            project_identity: None,
            selector: None,
            generation: None,
            generation_identity: None,
            lease: None,
            removal_requested: false,
        },
    }
}

fn durable_system_registration(class: GcRootClass) -> RootRegistration {
    RootRegistration {
        class,
        owner_scope: SYSTEM_OWNER_SCOPE.to_string(),
        project_identity: None,
        selector: None,
        generation: None,
        generation_identity: None,
        lease: None,
        removal_requested: false,
    }
}

fn new_record(
    logical_path: String,
    source: GcRootSource,
    registration: RootRegistration,
    created_unix_s: i64,
    last_transition_reason: &str,
) -> GcRootRecord {
    let policy_blake3 = store_retention_policy_blake3();
    let last_transition_id = transition_id(
        &logical_path,
        registration.class,
        &registration.owner_scope,
        &policy_blake3,
        created_unix_s,
        last_transition_reason,
    );
    GcRootRecord {
        schema_version: ROOT_RECORD_SCHEMA_VERSION,
        logical_path,
        source,
        root_class: registration.class,
        owner_scope: registration.owner_scope,
        project_identity: registration.project_identity,
        selector: registration.selector,
        generation: registration.generation,
        generation_identity: registration.generation_identity,
        lease: registration.lease,
        policy_blake3,
        created_unix_s,
        last_transition_id,
        last_transition_reason: last_transition_reason.to_string(),
        removal_requested: registration.removal_requested,
    }
}

fn normalize_raw_record(raw: RawGcRootRecord) -> Result<GcRootRecord, Error> {
    if raw.schema_version == 0 || raw.schema_version == 1 || raw.root_class.is_none() {
        return Ok(migrate_legacy_record(raw));
    }
    if raw.schema_version != ROOT_RECORD_SCHEMA_VERSION && raw.schema_version != PREVIOUS_ROOT_RECORD_SCHEMA_VERSION {
        return Err(Error::RootRegistry(format!("unsupported root record schema version: {}", raw.schema_version)));
    }
    let Some(root_class) = raw.root_class else {
        return Err(Error::RootRegistry("versioned root class is missing".to_string()));
    };
    let owner_scope = raw.owner_scope.unwrap_or_default();
    let policy_blake3 = raw.policy_blake3.unwrap_or_default();
    let is_previous_schema = raw.schema_version == PREVIOUS_ROOT_RECORD_SCHEMA_VERSION;
    let last_transition_reason = if is_previous_schema {
        "migrated-versioned-v2".to_string()
    } else {
        raw.last_transition_reason.unwrap_or_default()
    };
    let last_transition_id = if is_previous_schema {
        transition_id(
            &raw.logical_path,
            root_class,
            &owner_scope,
            &policy_blake3,
            raw.created_unix_s,
            &last_transition_reason,
        )
    } else {
        raw.last_transition_id.unwrap_or_default()
    };
    let record = GcRootRecord {
        schema_version: ROOT_RECORD_SCHEMA_VERSION,
        logical_path: raw.logical_path,
        source: raw.source,
        root_class,
        owner_scope,
        project_identity: raw.project_identity,
        selector: raw.selector,
        generation: raw.generation,
        generation_identity: raw.generation_identity,
        lease: raw.lease,
        policy_blake3,
        created_unix_s: raw.created_unix_s,
        last_transition_id,
        last_transition_reason,
        removal_requested: raw.removal_requested,
    };
    validate_record(&record)?;
    Ok(record)
}

fn migrate_legacy_record(raw: RawGcRootRecord) -> GcRootRecord {
    let policy_blake3 = store_retention_policy_blake3();
    let owner_scope = LEGACY_OWNER_SCOPE.to_string();
    let last_transition_id = transition_id(
        &raw.logical_path,
        GcRootClass::LegacyUnmanaged,
        &owner_scope,
        &policy_blake3,
        raw.created_unix_s,
        "migrated-from-path-only-v1",
    );
    GcRootRecord {
        schema_version: ROOT_RECORD_SCHEMA_VERSION,
        logical_path: raw.logical_path,
        source: raw.source,
        root_class: GcRootClass::LegacyUnmanaged,
        owner_scope,
        project_identity: None,
        selector: None,
        generation: None,
        generation_identity: None,
        lease: None,
        policy_blake3,
        created_unix_s: raw.created_unix_s,
        last_transition_id,
        last_transition_reason: "migrated-from-path-only-v1".to_string(),
        removal_requested: false,
    }
}

async fn ensure_pathinfo_exists(pathinfo: &dyn PathInfoService, store_path: &StorePath<String>) -> Result<(), Error> {
    let digest = *store_path.digest();
    let stored = pathinfo
        .get(digest)
        .await
        .map_err(|error| Error::RootRegistry(format!("reading PathInfo for {store_path}: {error}")))?;
    let Some(stored) = stored else {
        return Err(Error::RootRegistry(format!("cannot pin nonexistent store path {store_path}")));
    };
    if stored.store_path != *store_path {
        return Err(Error::RootRegistry(format!(
            "PathInfo digest collision for {store_path}: stored path was {}",
            stored.store_path
        )));
    }
    Ok(())
}

pub(crate) fn ensure_root_mutation_allowed(state_dir: &Path) -> Result<(), Error> {
    if selected_backend_requires_casita_fence(state_dir)? && crate::gc::casita_gc_fence_pending(state_dir)? {
        return Err(Error::Gc(
            "gc-recovery-required: recover fenced Casita GC before changing retained roots".to_string(),
        ));
    }
    Ok(())
}

fn selected_backend_requires_casita_fence(state_dir: &Path) -> Result<bool, Error> {
    let identity_path = state_dir.join(crate::overlay::STORE_IDENTITY_FILE_NAME);
    let metadata = match std::fs::symlink_metadata(&identity_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(Error::Store(format!("observing {}: {error}", identity_path.display()))),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::Store(format!(
            "store-backend-mismatch: {} is not a regular identity file",
            identity_path.display()
        )));
    }
    let max_descriptor_bytes = crate::overlay::store_overlay_runtime_policy()?.limits.max_descriptor_bytes;
    let read_limit_bytes = max_descriptor_bytes
        .checked_add(1)
        .ok_or_else(|| Error::Store("bounded store identity read limit overflow".to_string()))?;
    let mut bytes = Vec::new();
    std::fs::File::open(&identity_path)
        .map_err(|error| Error::Store(format!("opening {}: {error}", identity_path.display())))?
        .take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Store(format!("reading {}: {error}", identity_path.display())))?;
    if u64::try_from(bytes.len()).map_or(true, |size_bytes| size_bytes > max_descriptor_bytes) {
        return Err(Error::Store(format!(
            "store-backend-mismatch: {} exceeds bounded identity size",
            identity_path.display()
        )));
    }
    let identity: crate::overlay::StoreIdentityRecord = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Store(format!("parsing store identity {}: {error}", identity_path.display())))?;
    match (identity.schema.as_str(), identity.backend.as_deref()) {
        (crate::overlay::STORE_STATE_SCHEMA, None) | (crate::overlay::STORE_BACKEND_STATE_SCHEMA, Some("snix")) => {
            Ok(false)
        }
        (crate::overlay::STORE_BACKEND_STATE_SCHEMA, Some("casita")) => Ok(true),
        _ => Err(Error::Store("store-backend-mismatch: unsupported store identity schema or backend".to_string())),
    }
}

fn validate_registry(registry: &BTreeMap<String, GcRootRecord>) -> Result<(), Error> {
    for (key, record) in registry {
        if key.is_empty() {
            return Err(Error::RootRegistry("root registry key must not be empty".to_string()));
        }
        if !key.starts_with('/') {
            return Err(Error::RootRegistry(format!("root registry key must be absolute: {key}")));
        }
        if record.logical_path != *key {
            return Err(Error::RootRegistry(format!(
                "root registry key/value mismatch: key={key}, value={}",
                record.logical_path
            )));
        }
        validate_record(record)?;
    }
    Ok(())
}

pub(crate) fn validate_record(record: &GcRootRecord) -> Result<(), Error> {
    if record.schema_version != ROOT_RECORD_SCHEMA_VERSION {
        return Err(Error::RootRegistry(format!(
            "root record {} has unsupported schema {}",
            record.logical_path, record.schema_version
        )));
    }
    if record.owner_scope.is_empty() {
        return Err(Error::RootRegistry(format!("root owner scope is missing: {}", record.logical_path)));
    }
    let owner_kind = record.owner_scope.split_once(':').map_or(record.owner_scope.as_str(), |(kind, _)| kind);
    let owner_is_eligible = core_retention_policy()
        .eligible_owner_scopes
        .get(&record.root_class.into_core())
        .is_some_and(|scopes| scopes.iter().any(|scope| scope == owner_kind));
    if !owner_is_eligible {
        return Err(Error::RootRegistry(format!(
            "root owner scope is not eligible for {}: {}",
            record.root_class, record.logical_path
        )));
    }
    if !is_blake3_identity(&record.policy_blake3) {
        return Err(Error::RootRegistry(format!("root policy identity is invalid: {}", record.logical_path)));
    }
    if !is_blake3_identity(&record.last_transition_id) {
        return Err(Error::RootRegistry(format!("root transition identity is invalid: {}", record.logical_path)));
    }
    if record.last_transition_reason.is_empty() {
        return Err(Error::RootRegistry(format!("root transition reason is missing: {}", record.logical_path)));
    }
    let project_facts_complete = record.project_identity.as_ref().is_some_and(|value| !value.is_empty())
        && record.selector.as_ref().is_some_and(|value| !value.is_empty())
        && record.generation.is_some()
        && record.generation_identity.as_ref().is_some_and(|value| !value.is_empty());
    if matches!(record.root_class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration)
        && !project_facts_complete
    {
        return Err(Error::RootRegistry(format!("project root facts are incomplete: {}", record.logical_path)));
    }
    if record.root_class == GcRootClass::ActiveShellLease && record.lease.is_none() {
        return Err(Error::RootRegistry(format!("shell lease facts are missing: {}", record.logical_path)));
    }
    Ok(())
}

fn is_blake3_identity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("b3:") else {
        return false;
    };
    hex.len() == TRANSITION_DIGEST_HEX_BYTES
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn transition_id(
    logical_path: &str,
    class: GcRootClass,
    owner_scope: &str,
    policy_blake3: &str,
    created_unix_s: i64,
    transition: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TRANSITION_DOMAIN);
    hash_transition_field(&mut hasher, logical_path);
    hash_transition_field(&mut hasher, class.as_str());
    hash_transition_field(&mut hasher, owner_scope);
    hash_transition_field(&mut hasher, policy_blake3);
    hasher.update(&created_unix_s.to_be_bytes());
    hash_transition_field(&mut hasher, transition);
    format!("b3:{}", HEXLOWER.encode(hasher.finalize().as_bytes()))
}

fn hash_transition_field(hasher: &mut blake3::Hasher, value: &str) {
    let byte_count = value.len() as u128;
    hasher.update(&byte_count.to_be_bytes());
    hasher.update(value.as_bytes());
}

#[allow(
    tigerstyle::ambient_clock,
    reason = "shell boundary clock read for persisted GC root metadata"
)]
pub(crate) fn current_unix_seconds() -> Result<i64, Error> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::RootRegistry(format!("system clock before unix epoch: {error}")))?;
    i64::try_from(since_epoch.as_secs())
        .map_err(|_| Error::RootRegistry("unix timestamp exceeds i64 range".to_string()))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use async_trait::async_trait;
    use futures::StreamExt;
    use futures::stream::BoxStream;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;

    use super::*;

    const PATHINFO_CAPACITY: usize = 32;
    const CREATED_UNIX_S: i64 = 123;

    fn store_path(name: &str, seed: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [seed; 20]).expect("valid test store path")
    }

    fn pathinfo_service() -> Arc<dyn PathInfoService> {
        let capacity = NonZeroUsize::new(PATHINFO_CAPACITY).expect("positive PathInfo capacity");
        Arc::new(LruPathInfoService::with_capacity("roots-test".to_string(), capacity))
    }

    fn unsigned_pathinfo(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").expect("valid symlink target"),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [1_u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn register_root_survives_reload_with_versioned_provenance() {
        let state_dir = tempfile::tempdir().expect("temporary state");
        let pathinfo = pathinfo_service();
        let store_path = store_path("kept", 7);
        pathinfo.put(unsigned_pathinfo(store_path.clone())).await.expect("persist test PathInfo");

        let record = register_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &store_path, GcRootSource::Pin)
            .await
            .expect("register root");
        let listed = list_roots(state_dir.path()).expect("list roots");

        assert_eq!(listed, vec![record]);
        assert_eq!(listed[0].root_class, GcRootClass::ExplicitPin);
        assert_eq!(listed[0].owner_scope, OPERATOR_OWNER_SCOPE);
        assert!(is_blake3_identity(&listed[0].policy_blake3));
    }

    #[tokio::test]
    async fn managed_generation_batch_is_atomic_and_shares_generation_number() {
        const FIRST_PATH_DIGEST_BYTE: u8 = 23;
        const SECOND_PATH_DIGEST_BYTE: u8 = 24;
        const MISSING_PATH_DIGEST_BYTE: u8 = 25;
        const FIRST_GENERATION: u64 = 1;
        const EXPECTED_BATCH_SIZE: usize = 2;
        let state_dir = tempfile::tempdir().expect("temporary state");
        let pathinfo = pathinfo_service();
        let first_path = store_path("first-output", FIRST_PATH_DIGEST_BYTE);
        let second_path = store_path("second-output", SECOND_PATH_DIGEST_BYTE);
        let missing_path = store_path("missing-output", MISSING_PATH_DIGEST_BYTE);
        pathinfo.put(unsigned_pathinfo(first_path.clone())).await.expect("persist first PathInfo");
        pathinfo.put(unsigned_pathinfo(second_path.clone())).await.expect("persist second PathInfo");
        let registration = RootRegistration {
            class: GcRootClass::ProjectOutputGeneration,
            owner_scope: "project:b3:demo".to_string(),
            project_identity: Some("b3:demo".to_string()),
            selector: Some("default".to_string()),
            generation: None,
            generation_identity: Some("b3:lock".to_string()),
            lease: None,
            removal_requested: false,
        };

        let failed = register_root_batch_with_registration(state_dir.path(), "/nix/store", pathinfo.as_ref(), vec![
            (first_path.clone(), GcRootSource::Build, registration.clone()),
            (missing_path, GcRootSource::Build, registration.clone()),
        ])
        .await
        .expect_err("missing batch member must reject the full transition");
        assert!(matches!(failed, Error::RootRegistry(message) if message.contains("nonexistent")));
        assert!(list_roots(state_dir.path()).expect("unchanged root registry").is_empty());

        let records = register_root_batch_with_registration(state_dir.path(), "/nix/store", pathinfo.as_ref(), vec![
            (first_path, GcRootSource::Build, registration.clone()),
            (second_path, GcRootSource::Build, registration),
        ])
        .await
        .expect("complete generation batch");
        assert_eq!(records.len(), EXPECTED_BATCH_SIZE);
        assert!(records.iter().all(|record| record.generation == Some(FIRST_GENERATION)));
        assert_eq!(list_roots(state_dir.path()).expect("persisted batch").len(), records.len());
    }

    #[derive(Default)]
    struct FailingPathInfoService;

    #[async_trait]
    impl PathInfoService for FailingPathInfoService {
        async fn get(&self, _digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            Err(std::io::Error::other("unreadable metadata").into())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            Ok(path_info)
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            futures::stream::empty().boxed()
        }
    }

    #[tokio::test]
    async fn pin_rejects_nonexistent_and_unreadable_paths() {
        let state_dir = tempfile::tempdir().expect("temporary state");
        let pathinfo = pathinfo_service();
        let missing = store_path("missing", 9).to_absolute_path();
        let missing_error = pin_root_for_owner(
            state_dir.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &missing,
            "operator",
            "explicit-pin-registered",
        )
        .await
        .expect_err("missing path must fail");
        assert!(matches!(missing_error, Error::RootRegistry(message) if message.contains("nonexistent")));

        let unreadable = store_path("broken", 10).to_absolute_path();
        let unreadable_error = pin_root_for_owner(
            state_dir.path(),
            "/nix/store",
            &FailingPathInfoService,
            &unreadable,
            "operator",
            "explicit-pin-registered",
        )
        .await
        .expect_err("unreadable PathInfo must fail");
        assert!(matches!(unreadable_error, Error::RootRegistry(message) if message.contains("unreadable metadata")));
    }

    // r[verify store_lifecycle.root_provenance]
    #[test]
    fn legacy_record_migrates_to_protected_unmanaged_state() {
        let state_dir = tempfile::tempdir().expect("temporary state");
        let path = store_path("legacy", 11).to_absolute_path();
        let legacy = serde_json::json!({
            path.clone(): {
                "logical_path": path.clone(),
                "source": "build",
                "created_unix_s": CREATED_UNIX_S,
            }
        });
        std::fs::write(roots_path(state_dir.path()), serde_json::to_vec_pretty(&legacy).expect("legacy JSON"))
            .expect("write legacy registry");

        let loaded = list_roots(state_dir.path()).expect("load legacy roots");
        assert_eq!(loaded[0].root_class, GcRootClass::LegacyUnmanaged);
        assert_eq!(loaded[0].owner_scope, LEGACY_OWNER_SCOPE);
        assert!(!loaded[0].removal_requested);

        migrate_legacy_registry(state_dir.path()).expect("persist migrated registry");
        assert!(!roots_path(state_dir.path()).exists());
        let archived: serde_json::Value = serde_json::from_slice(
            &std::fs::read(state_dir.path().join("gc-roots.migrated.json")).expect("read archived registry"),
        )
        .expect("parse archived registry");
        assert_eq!(archived[&path]["source"], "build");
        let interests = crate::interest_store::per_path(state_dir.path()).expect("read migrated interests");
        assert_eq!(interests[&path][0].owner, LEGACY_OWNER_SCOPE);
        assert_eq!(
            list_roots(state_dir.path()).expect("read migrated roots")[0].root_class,
            GcRootClass::LegacyUnmanaged
        );
    }

    #[test]
    fn corrupt_registry_is_rejected_without_replacement() {
        let state_dir = tempfile::tempdir().expect("temporary state");
        let path = store_path("bad", 12).to_absolute_path();
        let corrupt = serde_json::json!({
            path.clone(): {
                "schema_version": ROOT_RECORD_SCHEMA_VERSION,
                "logical_path": path,
                "source": "pin",
                "root_class": "explicit-pin",
                "owner_scope": "",
                "policy_blake3": store_retention_policy_blake3(),
                "created_unix_s": CREATED_UNIX_S,
                "last_transition_id": transition_id("/nix/store/bad", GcRootClass::ExplicitPin, "", &store_retention_policy_blake3(), CREATED_UNIX_S, "created"),
                "removal_requested": false,
            }
        });
        let bytes = serde_json::to_vec_pretty(&corrupt).expect("corrupt fixture JSON");
        std::fs::write(roots_path(state_dir.path()), &bytes).expect("write corrupt fixture");

        let error = migrate_legacy_registry(state_dir.path()).expect_err("corrupt registry must fail");
        assert!(matches!(error, Error::RootRegistry(message) if message.contains("owner scope")));
        assert_eq!(std::fs::read(roots_path(state_dir.path())).expect("read unchanged fixture"), bytes);
    }

    #[test]
    fn unmanaged_remote_registration_uses_remote_owner_scope() {
        let registration = registration_for_source(GcRootSource::Remote);

        assert_eq!(registration.class, GcRootClass::RemoteResult);
        assert_eq!(registration.owner_scope, REMOTE_OWNER_SCOPE);
    }

    #[test]
    fn project_generation_reuses_identity_and_advances_new_identity() {
        const FIRST_GENERATION: u64 = 1;
        const SECOND_GENERATION: u64 = 2;
        const GENERATION_PATH_DIGEST_BYTE: u8 = 21;
        let logical_path = store_path("generation", GENERATION_PATH_DIGEST_BYTE).to_absolute_path();
        let registration = RootRegistration {
            class: GcRootClass::ProjectOutputGeneration,
            owner_scope: "project:b3:demo".to_string(),
            project_identity: Some("b3:demo".to_string()),
            selector: Some("default".to_string()),
            generation: None,
            generation_identity: Some("b3:lock-a".to_string()),
            lease: None,
            removal_requested: false,
        };
        let first = resolve_project_generation(std::iter::empty(), registration.clone()).expect("first generation");
        assert_eq!(first.generation, Some(FIRST_GENERATION));

        let first_record = new_record(
            logical_path.clone(),
            GcRootSource::Build,
            first,
            CREATED_UNIX_S,
            "project-generation-registered",
        );
        let registry = BTreeMap::from([(logical_path.clone(), first_record)]);
        let reused = resolve_project_generation(registry.values(), registration.clone())
            .expect("same identity reuses generation");
        assert_eq!(reused.generation, Some(FIRST_GENERATION));

        let mut next_registration = registration;
        next_registration.generation_identity = Some("b3:lock-b".to_string());
        let next =
            resolve_project_generation(registry.values(), next_registration).expect("new identity advances generation");
        assert_eq!(next.generation, Some(SECOND_GENERATION));
    }

    #[test]
    fn shell_lease_renewal_is_bounded_and_requires_lease_facts() {
        const INITIAL_RENEWAL_COUNT: u32 = 2;
        const NEXT_RENEWAL_COUNT: u32 = 3;
        const SHELL_PATH_DIGEST_BYTE: u8 = 22;
        const NEW_LEASE_EXTENSION_SECONDS: i64 = 100;
        const EXISTING_LEASE_EXTENSION_SECONDS: i64 = 50;
        const OBSERVATION_OFFSET_SECONDS: i64 = 1;
        let logical_path = store_path("shell", SHELL_PATH_DIGEST_BYTE).to_absolute_path();
        let registration = RootRegistration {
            class: GcRootClass::ActiveShellLease,
            owner_scope: "project:b3:demo".to_string(),
            project_identity: Some("b3:demo".to_string()),
            selector: Some("shell:default".to_string()),
            generation: None,
            generation_identity: None,
            lease: Some(GcRootLease {
                lease_id: "b3:lease-new".to_string(),
                expires_unix_s: CREATED_UNIX_S + NEW_LEASE_EXTENSION_SECONDS,
                last_observed_unix_s: CREATED_UNIX_S,
                renewal_count: 0,
            }),
            removal_requested: false,
        };
        let mut existing_registration = registration.clone();
        existing_registration.lease = Some(GcRootLease {
            lease_id: "b3:lease-existing".to_string(),
            expires_unix_s: CREATED_UNIX_S + EXISTING_LEASE_EXTENSION_SECONDS,
            last_observed_unix_s: CREATED_UNIX_S - OBSERVATION_OFFSET_SECONDS,
            renewal_count: INITIAL_RENEWAL_COUNT,
        });
        let existing = new_record(
            logical_path.clone(),
            GcRootSource::Build,
            existing_registration,
            CREATED_UNIX_S,
            "shell-lease-registered",
        );
        let registry = BTreeMap::from([(logical_path.clone(), existing)]);
        let renewed = resolve_shell_lease_renewal(registry.get(&logical_path), registration.clone())
            .expect("matching shell lease renews");
        let renewed_lease = renewed.lease.expect("renewed lease facts");
        assert_eq!(renewed_lease.lease_id, "b3:lease-existing");
        assert_eq!(renewed_lease.renewal_count, NEXT_RENEWAL_COUNT);

        let mut exhausted_registry = registry.clone();
        let exhausted_record = exhausted_registry.get_mut(&logical_path).expect("existing lease record");
        exhausted_record.lease.as_mut().expect("existing lease facts").renewal_count =
            store_retention_runtime_policy().limits.max_lease_renewals;
        let exhausted_error = resolve_shell_lease_renewal(exhausted_registry.get(&logical_path), registration)
            .expect_err("lease renewal beyond the policy limit must fail");
        assert!(matches!(exhausted_error, Error::RootRegistry(message) if message.contains("exceeds policy limit")));

        let missing_lease = RootRegistration {
            class: GcRootClass::ActiveShellLease,
            owner_scope: "project:b3:demo".to_string(),
            project_identity: Some("b3:demo".to_string()),
            selector: Some("shell:default".to_string()),
            generation: None,
            generation_identity: None,
            lease: None,
            removal_requested: false,
        };
        let error = resolve_shell_lease_renewal(registry.get(&logical_path), missing_lease)
            .expect_err("shell registration without lease must fail");
        assert!(matches!(error, Error::RootRegistry(message) if message.contains("requires lease facts")));
    }

    #[tokio::test]
    async fn renewed_shell_interest_replaces_its_owner_without_erasing_another_or_pending_file() {
        let state = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("renewed-shell", 62);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let logical_path = path.to_absolute_path();
        let now = current_unix_seconds().unwrap();
        let registration = |owner: &str| RootRegistration {
            class: GcRootClass::ActiveShellLease,
            owner_scope: format!("project:b3:{owner}"),
            project_identity: Some(format!("b3:{owner}")),
            selector: Some("shell:default".to_string()),
            generation: None,
            generation_identity: None,
            lease: Some(GcRootLease {
                lease_id: format!("b3:lease-{owner}"),
                expires_unix_s: now + 3_600,
                last_observed_unix_s: now,
                renewal_count: 0,
            }),
            removal_requested: false,
        };
        for owner in ["zeta", "alpha"] {
            register_root_with_registration(
                state.path(),
                "/nix/store",
                pathinfo.as_ref(),
                &path,
                GcRootSource::Build,
                registration(owner),
            )
            .await
            .unwrap();
        }
        let pending = state.path().join("retention-interests").join(".pending-interrupted");
        std::fs::write(&pending, b"incomplete publication").unwrap();
        for count in 1..=2 {
            let renewed = register_root_with_registration(
                state.path(),
                "/nix/store",
                pathinfo.as_ref(),
                &path,
                GcRootSource::Build,
                registration("zeta"),
            )
            .await
            .unwrap();
            assert_eq!(renewed.last_transition_reason, "shell-lease-renewed");
            assert_eq!(renewed.lease.as_ref().unwrap().renewal_count, count);
        }
        let records = crate::interest_store::per_path(state.path()).unwrap();
        assert_eq!(records[&logical_path].len(), 2);
        assert!(records[&logical_path].iter().any(|record| record.owner == "project:b3:alpha"
            && record.declaration.lease.as_ref().unwrap().renewal_count == 0));
        assert!(
            records[&logical_path].iter().any(|record| record.owner == "project:b3:zeta"
                && record.declaration.lease.as_ref().unwrap().renewal_count == 2)
        );
        assert_eq!(std::fs::read(pending).unwrap(), b"incomplete publication");
    }

    #[tokio::test]
    async fn renewed_shell_interest_rejects_limit_without_mutating_records() {
        let state = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("bounded-shell", 63);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let now = current_unix_seconds().unwrap();
        let max = store_retention_runtime_policy().limits.max_lease_renewals;
        let registration = || RootRegistration {
            class: GcRootClass::ActiveShellLease,
            owner_scope: "project:b3:bounded".to_string(),
            project_identity: Some("b3:bounded".to_string()),
            selector: Some("shell:default".to_string()),
            generation: None,
            generation_identity: None,
            lease: Some(GcRootLease {
                lease_id: "b3:bounded-lease".to_string(),
                expires_unix_s: now + 3_600,
                last_observed_unix_s: now,
                renewal_count: max - 1,
            }),
            removal_requested: false,
        };
        let prior =
            new_record(path.to_absolute_path(), GcRootSource::Build, registration(), now, "shell-lease-registered");
        crate::interest_store::publish(state.path(), &crate::interest_store::InterestRecord::new(prior)).unwrap();
        let last_allowed = register_root_with_registration(
            state.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &path,
            GcRootSource::Build,
            registration(),
        )
        .await
        .unwrap();
        assert_eq!(last_allowed.lease.unwrap().renewal_count, max);
        let before = crate::interest_store::load(state.path()).unwrap();
        let before_files = std::fs::read_dir(state.path().join("retention-interests"))
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), std::fs::read(entry.path()).unwrap())
            })
            .collect::<BTreeMap<_, _>>();
        let denied = register_root_with_registration(
            state.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &path,
            GcRootSource::Build,
            registration(),
        )
        .await
        .unwrap_err();
        assert!(denied.to_string().contains("renewal exceeds policy limit"));
        assert_eq!(crate::interest_store::load(state.path()).unwrap(), before);
        let after_files = std::fs::read_dir(state.path().join("retention-interests"))
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), std::fs::read(entry.path()).unwrap())
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(after_files, before_files);
    }

    #[tokio::test]
    async fn operator_pin_reasons_survive_automatic_explicit_pin_registration() {
        let state = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("distinct-pin-reasons", 64);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let logical_path = path.to_absolute_path();
        for reason in ["ci", "interactive"] {
            pin_root_for_owner(state.path(), "/nix/store", pathinfo.as_ref(), &logical_path, "operator", reason)
                .await
                .unwrap();
        }
        assert_eq!(crate::interest_store::per_path(state.path()).unwrap()[&logical_path].len(), 2);
        register_root(state.path(), "/nix/store", pathinfo.as_ref(), &path, GcRootSource::Pin)
            .await
            .unwrap();
        let records = crate::interest_store::per_path(state.path()).unwrap();
        assert_eq!(records[&logical_path].len(), 3);
        assert!(records[&logical_path].iter().any(|record| record.reason == "ci"));
        assert!(records[&logical_path].iter().any(|record| record.reason == "interactive"));
        let ambiguous = unpin_root_for_owner_reason(
            state.path(),
            LogicalStorePathRef {
                logical_path: &logical_path,
                store_dir: "/nix/store",
            },
            OPERATOR_OWNER_SCOPE,
            None,
        )
        .unwrap_err();
        assert!(ambiguous.to_string().contains("ambiguous release"));
        unpin_root_for_owner_reason(
            state.path(),
            LogicalStorePathRef {
                logical_path: &logical_path,
                store_dir: "/nix/store",
            },
            OPERATOR_OWNER_SCOPE,
            Some("ci"),
        )
        .unwrap()
        .unwrap();
        let after = crate::interest_store::per_path(state.path()).unwrap();
        assert_eq!(after[&logical_path].len(), 2);
        assert!(after[&logical_path].iter().any(|record| record.reason == "interactive"));
    }

    #[test]
    fn unpin_removes_existing_versioned_record() {
        let state_dir = tempfile::tempdir().expect("temporary state");
        let path = store_path("hello", 13).to_absolute_path();
        let record = new_record(
            path.clone(),
            GcRootSource::Pin,
            registration_for_source(GcRootSource::Pin),
            CREATED_UNIX_S,
            "explicit-pin-registered",
        );
        crate::interest_store::publish(state_dir.path(), &crate::interest_store::InterestRecord::new(record))
            .expect("publish root interest");

        let removed = unpin_root_for_owner_reason(
            state_dir.path(),
            LogicalStorePathRef {
                logical_path: &path,
                store_dir: "/nix/store",
            },
            OPERATOR_OWNER_SCOPE,
            None,
        )
        .expect("unpin root");

        assert!(removed.is_some());
        assert!(list_roots(state_dir.path()).expect("list roots").is_empty());
    }
    #[tokio::test]
    async fn snix_pin_and_unpin_ignore_foreign_casita_gc_fence() {
        let state_dir = tempfile::tempdir().unwrap();
        crate::overlay::ensure_store_identity(state_dir.path(), "/nix/store", crate::StoreBackend::Snix).unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("snix-foreign-fence", 44);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let logical_path = path.to_absolute_path();
        let fence = state_dir.path().join("casita-gc-fence.json");
        std::fs::write(&fence, b"foreign Casita fence").unwrap();

        let pinned = pin_root_for_owner(
            state_dir.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &logical_path,
            "operator",
            "explicit-pin-registered",
        )
        .await
        .unwrap();
        assert_eq!(pinned.logical_path, logical_path);
        let removed = unpin_root_for_owner_reason(
            state_dir.path(),
            LogicalStorePathRef {
                logical_path: &logical_path,
                store_dir: "/nix/store",
            },
            OPERATOR_OWNER_SCOPE,
            None,
        )
        .unwrap();
        assert_eq!(removed, Some(pinned));
        assert!(list_roots(state_dir.path()).unwrap().is_empty());
        assert_eq!(std::fs::read(fence).unwrap(), b"foreign Casita fence");
    }

    #[tokio::test]
    async fn casita_fenced_root_mutations_preserve_registry_bytes() {
        let state_dir = tempfile::tempdir().unwrap();
        crate::overlay::ensure_store_identity(state_dir.path(), "/nix/store", crate::StoreBackend::Casita).unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("casita-fenced", 45);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let logical_path = path.to_absolute_path();
        pin_root_for_owner(
            state_dir.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &logical_path,
            "operator",
            "explicit-pin-registered",
        )
        .await
        .unwrap();
        let registry_before = crate::interest_store::load(state_dir.path()).unwrap();
        let fence = state_dir.path().join("casita-gc-fence.json");
        std::fs::write(&fence, b"pending Casita fence").unwrap();

        let denied_pin = pin_root_for_owner(
            state_dir.path(),
            "/nix/store",
            pathinfo.as_ref(),
            &logical_path,
            "operator",
            "explicit-pin-registered",
        )
        .await
        .unwrap_err();
        assert!(denied_pin.to_string().contains("gc-recovery-required"));
        let denied_unpin = unpin_root_for_owner_reason(
            state_dir.path(),
            LogicalStorePathRef {
                logical_path: &logical_path,
                store_dir: "/nix/store",
            },
            OPERATOR_OWNER_SCOPE,
            None,
        )
        .unwrap_err();
        assert!(denied_unpin.to_string().contains("gc-recovery-required"));
        assert!(migrate_legacy_registry(state_dir.path()).unwrap_err().to_string().contains("gc-recovery-required"));
        assert_eq!(crate::interest_store::load(state_dir.path()).unwrap(), registry_before);
        assert_eq!(std::fs::read(fence).unwrap(), b"pending Casita fence");
    }

    // r[verify mantle.store_lifecycle.retention_interest_records]
    #[tokio::test]
    async fn concurrent_owners_keep_both_records_and_release_one() {
        let state_dir = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let path = store_path("shared", 51);
        pathinfo.put(unsigned_pathinfo(path.clone())).await.unwrap();
        let registration = |owner: &str| RootRegistration {
            owner_scope: format!("operator:{owner}"),
            ..registration_for_source(GcRootSource::Pin)
        };
        let (first, second) = tokio::join!(
            register_root_with_registration(
                state_dir.path(),
                "/nix/store",
                pathinfo.as_ref(),
                &path,
                GcRootSource::Pin,
                registration("a")
            ),
            register_root_with_registration(
                state_dir.path(),
                "/nix/store",
                pathinfo.as_ref(),
                &path,
                GcRootSource::Pin,
                registration("b")
            ),
        );
        first.unwrap();
        second.unwrap();
        let logical_path = path.to_absolute_path();
        let owners = crate::interest_store::per_path(state_dir.path()).unwrap();
        assert_eq!(owners[&logical_path].len(), 2);
        assert_eq!(list_roots(state_dir.path()).unwrap().len(), 1);
        let foreign = unpin_root_for_owner_reason(
            state_dir.path(),
            LogicalStorePathRef {
                logical_path: &logical_path,
                store_dir: "/nix/store",
            },
            "operator:c",
            None,
        )
        .unwrap_err();
        assert!(foreign.to_string().contains("foreign-owner"));
        assert!(
            unpin_root_for_owner_reason(
                state_dir.path(),
                LogicalStorePathRef {
                    logical_path: &logical_path,
                    store_dir: "/nix/store"
                },
                "operator:a",
                None,
            )
            .unwrap()
            .is_some()
        );
        let remaining = crate::interest_store::per_path(state_dir.path()).unwrap();
        assert_eq!(remaining[&logical_path].len(), 1);
        assert_eq!(remaining[&logical_path][0].owner, "operator:b");
        assert_eq!(list_roots(state_dir.path()).unwrap().len(), 1);
    }

    // r[verify mantle.store_lifecycle.retention_interest_records]
    #[test]
    fn invalid_interest_bytes_version_identity_and_duplicates_fail_closed() {
        let state = tempfile::tempdir().unwrap();
        let path = store_path("interest", 52).to_absolute_path();
        let declaration = new_record(
            path.clone(),
            GcRootSource::Pin,
            registration_for_source(GcRootSource::Pin),
            CREATED_UNIX_S,
            "explicit-pin-registered",
        );
        let record = crate::interest_store::InterestRecord::new(declaration.clone());
        crate::interest_store::publish(state.path(), &record).unwrap();
        let dir = state.path().join("retention-interests");
        let original = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let bytes = std::fs::read(&original).unwrap();
        std::fs::write(&original, b"not json").unwrap();
        assert!(list_roots(state.path()).unwrap_err().to_string().contains("parsing"));
        std::fs::write(&original, &bytes).unwrap();
        let mut wrong_version = serde_json::to_value(&record).unwrap();
        wrong_version["schema_version"] = serde_json::json!(99);
        let wrong_bytes = serde_json::to_vec(&wrong_version).unwrap();
        let wrong_id = HEXLOWER.encode(&crunch_gc_core::interest::identity(&wrong_bytes));
        let wrong_path = dir.join(format!("{wrong_id}.json"));
        std::fs::write(&wrong_path, &wrong_bytes).unwrap();
        assert!(list_roots(state.path()).unwrap_err().to_string().contains("version"));
        std::fs::remove_file(&wrong_path).unwrap();
        let invalid_name = dir.join("0000000000000000000000000000000000000000000000000000000000000000.json");
        std::fs::write(&invalid_name, &bytes).unwrap();
        assert!(list_roots(state.path()).unwrap_err().to_string().contains("identity mismatch"));
        std::fs::remove_file(&invalid_name).unwrap();
        let duplicate_declaration = new_record(
            path,
            GcRootSource::Pin,
            registration_for_source(GcRootSource::Pin),
            CREATED_UNIX_S + 1,
            "explicit-pin-registered",
        );
        let duplicate_bytes =
            serde_json::to_vec(&crate::interest_store::InterestRecord::new(duplicate_declaration)).unwrap();
        let duplicate_id = HEXLOWER.encode(&crunch_gc_core::interest::identity(&duplicate_bytes));
        let duplicate_path = dir.join(format!("{duplicate_id}.json"));
        std::fs::write(&duplicate_path, &duplicate_bytes).unwrap();
        assert!(list_roots(state.path()).unwrap_err().to_string().contains("duplicate retention interest"));
        std::fs::remove_file(&duplicate_path).unwrap();
        std::fs::write(&original, vec![b'x'; crunch_gc_core::interest::MAX_INTEREST_BYTES + 1]).unwrap();
        assert!(list_roots(state.path()).unwrap_err().to_string().contains("oversized"));
        let oversized_state = tempfile::tempdir().unwrap();
        let mut oversized_declaration = declaration;
        oversized_declaration.project_identity = Some("x".repeat(crunch_gc_core::interest::MAX_INTEREST_BYTES));
        assert!(
            crate::interest_store::publish(
                oversized_state.path(),
                &crate::interest_store::InterestRecord::new(oversized_declaration)
            )
            .unwrap_err()
            .to_string()
            .contains("size limit")
        );
        assert!(crate::interest_store::load(oversized_state.path()).unwrap().is_empty());
    }

    #[test]
    fn migration_refuses_to_overwrite_an_existing_legacy_archive() {
        let state = tempfile::tempdir().unwrap();
        let path = store_path("archived-root", 59).to_absolute_path();
        let legacy_bytes = serde_json::to_vec(&serde_json::json!({
            path.clone(): {"logical_path": path, "source": "build", "created_unix_s": CREATED_UNIX_S}
        }))
        .unwrap();
        let legacy_path = roots_path(state.path());
        let archive_path = state.path().join("gc-roots.migrated.json");
        std::fs::write(&legacy_path, &legacy_bytes).unwrap();
        std::fs::write(&archive_path, b"prior archive").unwrap();

        let error = migrate_legacy_registry(state.path()).unwrap_err();
        assert!(error.to_string().contains("archive already exists"));
        assert_eq!(std::fs::read(&legacy_path).unwrap(), legacy_bytes);
        assert_eq!(std::fs::read(&archive_path).unwrap(), b"prior archive");
        assert!(crate::interest_store::load(state.path()).unwrap().is_empty());
    }

    #[test]
    fn migrated_interest_set_preserves_gc_candidate_order() {
        use crunch_gc_core::GcEntry;
        use crunch_gc_core::GcExecutionMode;
        use crunch_gc_core::GcOwnership;
        use crunch_gc_core::GcPlanRequest;

        let state = tempfile::tempdir().unwrap();
        let keep = store_path("keep", 53).to_absolute_path();
        let drop_a = store_path("drop-a", 54).to_absolute_path();
        let drop_b = store_path("drop-b", 55).to_absolute_path();
        let legacy = serde_json::json!({
            keep.clone(): {"logical_path": keep.clone(), "source": "build", "created_unix_s": CREATED_UNIX_S}
        });
        std::fs::write(roots_path(state.path()), serde_json::to_vec(&legacy).unwrap()).unwrap();
        let plan_for_roots = |roots: Vec<GcRootRecord>| {
            crunch_gc_core::plan_gc(GcPlanRequest {
                roots: roots.into_iter().map(|root| root.logical_path).collect(),
                entries: [&keep, &drop_a, &drop_b]
                    .into_iter()
                    .map(|path| GcEntry {
                        path_id: path.clone(),
                        references: Vec::new(),
                        declared_nar_bytes: 1,
                        ownership: GcOwnership::Overlay,
                    })
                    .collect(),
                execution_mode: GcExecutionMode::DryRun,
            })
            .unwrap()
        };
        let before = plan_for_roots(list_roots(state.path()).unwrap());
        migrate_legacy_registry(state.path()).unwrap();
        let after = plan_for_roots(list_roots(state.path()).unwrap());
        assert_eq!(before.candidate_path_ids, after.candidate_path_ids);
        assert_eq!(before.plan_id, after.plan_id);
        assert_eq!(after.candidate_path_ids.len(), 2);
    }

    #[test]
    fn active_second_owner_lease_keeps_shared_path() {
        let state = tempfile::tempdir().unwrap();
        let logical_path = store_path("lease-shared", 58).to_absolute_path();
        let current_unix_s = current_unix_seconds().unwrap();
        let lease_record = |owner: &str, expires_unix_s: i64| {
            new_record(
                logical_path.clone(),
                GcRootSource::Build,
                RootRegistration {
                    class: GcRootClass::ActiveShellLease,
                    owner_scope: format!("operator:{owner}"),
                    project_identity: None,
                    selector: None,
                    generation: None,
                    generation_identity: None,
                    lease: Some(GcRootLease {
                        lease_id: format!("{owner}-lease"),
                        expires_unix_s,
                        last_observed_unix_s: current_unix_s - 60,
                        renewal_count: 0,
                    }),
                    removal_requested: false,
                },
                current_unix_s - 60,
                "shell-lease-registered",
            )
        };
        crate::interest_store::publish(
            state.path(),
            &crate::interest_store::InterestRecord::new(lease_record("a", current_unix_s - 1)),
        )
        .unwrap();
        crate::interest_store::publish(
            state.path(),
            &crate::interest_store::InterestRecord::new(lease_record("b", current_unix_s + 3600)),
        )
        .unwrap();
        let merged = list_roots(state.path()).unwrap();
        assert_eq!(merged.len(), 1);
        let plan = crunch_gc_core::retention::plan_retention(
            &core_retention_policy(),
            current_unix_s,
            crate::retention::records_to_core(&merged).unwrap(),
        )
        .unwrap();
        assert!(plan.decisions[0].disposition.retains_path(), "owner b has an active lease");
    }

    #[test]
    fn unmigrated_legacy_pin_survives_expired_new_owner_lease() {
        let state = tempfile::tempdir().unwrap();
        let logical_path = store_path("legacy-shared", 60).to_absolute_path();
        let current_unix_s = current_unix_seconds().unwrap();
        let legacy = new_record(
            logical_path.clone(),
            GcRootSource::Pin,
            registration_for_source(GcRootSource::Pin),
            current_unix_s - 60,
            "explicit-pin-registered",
        );
        std::fs::write(
            roots_path(state.path()),
            serde_json::to_vec(&BTreeMap::from([(logical_path.clone(), legacy)])).unwrap(),
        )
        .unwrap();
        let new_lease = new_record(
            logical_path,
            GcRootSource::Build,
            RootRegistration {
                class: GcRootClass::ActiveShellLease,
                owner_scope: "operator:new".to_string(),
                project_identity: None,
                selector: None,
                generation: None,
                generation_identity: None,
                lease: Some(GcRootLease {
                    lease_id: "new-lease".to_string(),
                    expires_unix_s: current_unix_s - 1,
                    last_observed_unix_s: current_unix_s - 60,
                    renewal_count: 0,
                }),
                removal_requested: false,
            },
            current_unix_s - 60,
            "shell-lease-registered",
        );
        crate::interest_store::publish(state.path(), &crate::interest_store::InterestRecord::new(new_lease)).unwrap();
        let merged = list_roots(state.path()).unwrap();
        assert_eq!(merged.len(), 1);
        let plan = crunch_gc_core::retention::plan_retention(
            &core_retention_policy(),
            current_unix_s,
            crate::retention::records_to_core(&merged).unwrap(),
        )
        .unwrap();
        assert!(plan.decisions[0].disposition.retains_path(), "unmigrated legacy pin must remain protective");
    }
}
