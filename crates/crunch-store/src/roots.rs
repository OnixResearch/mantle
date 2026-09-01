use std::collections::BTreeMap;
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

pub(crate) struct RootRegistrationRequest<'a> {
    pub state_dir: &'a Path,
    pub store_dir: &'a str,
    pub pathinfo: &'a dyn PathInfoService,
    pub store_path: &'a StorePath<String>,
    pub source: GcRootSource,
    pub registration: RootRegistration,
}

struct TransitionIdentityInput<'a> {
    logical_path: &'a str,
    class: GcRootClass,
    owner_scope: &'a str,
    policy_blake3: &'a str,
    created_unix_s: i64,
    transition: &'a str,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

fn missing<T>() -> Option<T> {
    None
}

const fn missing_schema_version() -> u32 {
    0
}

const fn removal_not_requested() -> bool {
    false
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGcRootRecord {
    #[serde(default = "missing_schema_version")]
    schema_version: u32,
    logical_path: String,
    source: GcRootSource,
    #[serde(default = "missing")]
    root_class: Option<GcRootClass>,
    #[serde(default = "missing")]
    owner_scope: Option<String>,
    #[serde(default = "missing")]
    project_identity: Option<String>,
    #[serde(default = "missing")]
    selector: Option<String>,
    #[serde(default = "missing")]
    generation: Option<u64>,
    #[serde(default = "missing")]
    generation_identity: Option<String>,
    #[serde(default = "missing")]
    lease: Option<GcRootLease>,
    #[serde(default = "missing")]
    policy_blake3: Option<String>,
    created_unix_s: i64,
    #[serde(default = "missing")]
    last_transition_id: Option<String>,
    #[serde(default = "missing")]
    last_transition_reason: Option<String>,
    #[serde(default = "removal_not_requested")]
    removal_requested: bool,
}

pub fn list_roots(state_dir: &Path) -> Result<Vec<GcRootRecord>, Error> {
    let registry = load_registry(state_dir)?;
    Ok(registry.into_values().collect())
}

pub fn migrate_legacy_registry(state_dir: &Path) -> Result<Vec<GcRootRecord>, Error> {
    let registry = load_registry(state_dir)?;
    save_registry(state_dir, &registry)?;
    Ok(registry.into_values().collect())
}

pub async fn register_root(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    store_path: &StorePath<String>,
    source: GcRootSource,
) -> Result<GcRootRecord, Error> {
    let registration = registration_for_source(source);
    register_root_with_registration(RootRegistrationRequest {
        state_dir,
        store_dir,
        pathinfo,
        store_path,
        source,
        registration,
    })
    .await
}

pub(crate) async fn register_root_with_registration(
    request: RootRegistrationRequest<'_>,
) -> Result<GcRootRecord, Error> {
    let records =
        register_root_batch_with_registration(request.state_dir, request.store_dir, request.pathinfo, vec![(
            request.store_path.clone(),
            request.source,
            request.registration,
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
    let mut registry = load_registry(state_dir)?;
    let mut records = Vec::with_capacity(registrations.len());
    for (store_path, source, registration) in registrations {
        let logical_path = store_path.to_absolute_path_with_prefix(store_dir);
        if records.iter().any(|record: &GcRootRecord| record.logical_path == logical_path) {
            return Err(Error::RootRegistry(format!("duplicate path in root registration batch: {logical_path}")));
        }
        let registration = resolve_project_generation(&registry, registration)?;
        let registration = resolve_shell_lease_renewal(&registry, &logical_path, registration)?;
        let transition_reason = registration_transition_reason(&registry, &logical_path, &registration);
        let record = new_record(logical_path.clone(), source, registration, created_unix_s, transition_reason);
        validate_record(&record)?;
        registry.insert(logical_path, record.clone());
        records.push(record);
    }
    if registry.len() > max_roots {
        return Err(Error::RootRegistry(format!("root registry exceeds policy limit {max_roots}")));
    }
    save_registry(state_dir, &registry)?;
    Ok(records)
}

pub async fn pin_root(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    logical_path: &str,
) -> Result<GcRootRecord, Error> {
    let store_path = parse_logical_store_path(LogicalStorePathRef {
        logical_path,
        store_dir,
    })?;
    register_root(state_dir, store_dir, pathinfo, &store_path, GcRootSource::Pin).await
}

pub fn unpin_root(state_dir: &Path, path: LogicalStorePathRef<'_>) -> Result<Option<GcRootRecord>, Error> {
    let normalized_path = normalize_logical_path(path)?;
    let mut registry = load_registry(state_dir)?;
    let removed = registry.remove(&normalized_path);
    save_registry(state_dir, &registry)?;
    Ok(removed)
}

pub(crate) fn load_registry(state_dir: &Path) -> Result<BTreeMap<String, GcRootRecord>, Error> {
    assert!(!state_dir.as_os_str().is_empty());
    let path = roots_path(state_dir);
    assert_eq!(path.file_name().and_then(|name| name.to_str()), Some(ROOTS_FILE_NAME));
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

fn resolve_project_generation(
    registry: &BTreeMap<String, GcRootRecord>,
    mut registration: RootRegistration,
) -> Result<RootRegistration, Error> {
    if !matches!(registration.class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration) {
        return Ok(registration);
    }
    if registration.generation.is_some() {
        return Ok(registration);
    }
    assert!(matches!(
        registration.class,
        GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration
    ));
    assert!(registration.generation.is_none());
    let (Some(project_identity), Some(selector), Some(generation_identity)) = (
        registration.project_identity.as_deref(),
        registration.selector.as_deref(),
        registration.generation_identity.as_deref(),
    ) else {
        return Err(Error::RootRegistry(
            "project registration requires project, selector, and generation identities".to_string(),
        ));
    };
    if let Some(existing) = registry.values().find(|record| {
        record.root_class == registration.class
            && record.project_identity.as_deref() == Some(project_identity)
            && record.selector.as_deref() == Some(selector)
            && record.generation_identity.as_deref() == Some(generation_identity)
    }) {
        registration.generation = existing.generation;
        return Ok(registration);
    }
    let latest_generation = registry
        .values()
        .filter(|record| {
            record.root_class == registration.class
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
    registry: &BTreeMap<String, GcRootRecord>,
    logical_path: &str,
    mut registration: RootRegistration,
) -> Result<RootRegistration, Error> {
    if registration.class != GcRootClass::ActiveShellLease {
        return Ok(registration);
    }
    assert_eq!(registration.class, GcRootClass::ActiveShellLease);
    let Some(lease) = registration.lease.as_mut() else {
        return Err(Error::RootRegistry("active shell registration requires lease facts".to_string()));
    };
    let Some(existing) = registry.get(logical_path) else {
        return Ok(registration);
    };
    if existing.root_class != GcRootClass::ActiveShellLease {
        return Ok(registration);
    }
    if existing.project_identity != registration.project_identity {
        return Ok(registration);
    }
    if existing.selector != registration.selector {
        return Ok(registration);
    }
    if existing.owner_scope != registration.owner_scope {
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
    assert_eq!(lease.renewal_count, renewal_count);
    Ok(registration)
}

fn is_shell_lease_renewal(existing: &GcRootRecord, registration: &RootRegistration) -> bool {
    if registration.class != GcRootClass::ActiveShellLease {
        return false;
    }
    if existing.root_class != GcRootClass::ActiveShellLease {
        return false;
    }
    if existing.project_identity != registration.project_identity {
        return false;
    }
    existing.selector == registration.selector
}

fn project_generation_transition(existing: &GcRootRecord, registration: &RootRegistration) -> Option<&'static str> {
    if !matches!(registration.class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration) {
        return None;
    }
    if existing.root_class != registration.class {
        return None;
    }
    if existing.project_identity != registration.project_identity {
        return None;
    }
    if existing.selector != registration.selector {
        return None;
    }
    assert_eq!(existing.root_class, registration.class);
    assert_eq!(existing.selector, registration.selector);
    if existing.generation_identity == registration.generation_identity {
        return Some("project-generation-reobserved");
    }
    Some("project-generation-advanced")
}

fn registration_transition_reason(
    registry: &BTreeMap<String, GcRootRecord>,
    logical_path: &str,
    registration: &RootRegistration,
) -> &'static str {
    let Some(existing) = registry.get(logical_path) else {
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
    if is_shell_lease_renewal(existing, registration) {
        assert_eq!(existing.root_class, registration.class);
        assert_eq!(existing.selector, registration.selector);
        return "shell-lease-renewed";
    }
    if let Some(reason) = project_generation_transition(existing, registration) {
        assert_eq!(existing.root_class, registration.class);
        assert_eq!(existing.selector, registration.selector);
        return reason;
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
    assert!(!logical_path.is_empty());
    assert!(!last_transition_reason.is_empty());
    let policy_blake3 = store_retention_policy_blake3();
    let last_transition_id = transition_id(TransitionIdentityInput {
        logical_path: &logical_path,
        class: registration.class,
        owner_scope: &registration.owner_scope,
        policy_blake3: &policy_blake3,
        created_unix_s,
        transition: last_transition_reason,
    });
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
    assert!(matches!(raw.schema_version, ROOT_RECORD_SCHEMA_VERSION | PREVIOUS_ROOT_RECORD_SCHEMA_VERSION));
    assert!(raw.root_class.is_some());
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
        transition_id(TransitionIdentityInput {
            logical_path: &raw.logical_path,
            class: root_class,
            owner_scope: &owner_scope,
            policy_blake3: &policy_blake3,
            created_unix_s: raw.created_unix_s,
            transition: &last_transition_reason,
        })
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
    let is_legacy_schema = raw.schema_version <= 1;
    let is_missing_root_class = raw.root_class.is_none();
    assert_ne!((is_legacy_schema, is_missing_root_class), (false, false));
    let policy_blake3 = store_retention_policy_blake3();
    let owner_scope = LEGACY_OWNER_SCOPE.to_string();
    assert_eq!(owner_scope, LEGACY_OWNER_SCOPE);
    let last_transition_id = transition_id(TransitionIdentityInput {
        logical_path: &raw.logical_path,
        class: GcRootClass::LegacyUnmanaged,
        owner_scope: &owner_scope,
        policy_blake3: &policy_blake3,
        created_unix_s: raw.created_unix_s,
        transition: "migrated-from-path-only-v1",
    });
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

fn save_registry(state_dir: &Path, registry: &BTreeMap<String, GcRootRecord>) -> Result<(), Error> {
    validate_registry(registry)?;
    std::fs::create_dir_all(state_dir)
        .map_err(|error| Error::RootRegistry(format!("creating {}: {error}", state_dir.display())))?;
    let path = roots_path(state_dir);
    let tmp_path = state_dir.join(format!("{ROOTS_FILE_NAME}.tmp"));
    let bytes = serde_json::to_vec_pretty(registry)
        .map_err(|error| Error::RootRegistry(format!("serializing {}: {error}", path.display())))?;
    std::fs::write(&tmp_path, bytes)
        .map_err(|error| Error::RootRegistry(format!("writing {}: {error}", tmp_path.display())))?;
    std::fs::rename(&tmp_path, &path).map_err(|error| {
        Error::RootRegistry(format!("renaming {} -> {}: {error}", tmp_path.display(), path.display()))
    })?;
    Ok(())
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

fn validate_record(record: &GcRootRecord) -> Result<(), Error> {
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
    let is_owner_eligible = core_retention_policy()
        .eligible_owner_scopes
        .get(&record.root_class.into_core())
        .is_some_and(|scopes| scopes.iter().any(|scope| scope == owner_kind));
    if !is_owner_eligible {
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
    let is_project_fact_set_complete = record.project_identity.as_ref().is_some_and(|value| !value.is_empty())
        && record.selector.as_ref().is_some_and(|value| !value.is_empty())
        && record.generation.is_some()
        && record.generation_identity.as_ref().is_some_and(|value| !value.is_empty());
    if matches!(record.root_class, GcRootClass::ProjectOutputGeneration | GcRootClass::ProjectSourceGeneration)
        && !is_project_fact_set_complete
    {
        return Err(Error::RootRegistry(format!("project root facts are incomplete: {}", record.logical_path)));
    }
    if record.root_class == GcRootClass::ActiveShellLease && record.lease.is_none() {
        return Err(Error::RootRegistry(format!("shell lease facts are missing: {}", record.logical_path)));
    }
    assert_eq!(record.schema_version, ROOT_RECORD_SCHEMA_VERSION);
    assert!(!record.owner_scope.is_empty());
    Ok(())
}

fn is_blake3_identity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("b3:") else {
        return false;
    };
    hex.len() == TRANSITION_DIGEST_HEX_BYTES
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn transition_id(input: TransitionIdentityInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TRANSITION_DOMAIN);
    hash_transition_field(&mut hasher, input.logical_path);
    hash_transition_field(&mut hasher, input.class.as_str());
    hash_transition_field(&mut hasher, input.owner_scope);
    hash_transition_field(&mut hasher, input.policy_blake3);
    hasher.update(&input.created_unix_s.to_be_bytes());
    hash_transition_field(&mut hasher, input.transition);
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
        let missing_error = pin_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &missing)
            .await
            .expect_err("missing path must fail");
        assert!(matches!(missing_error, Error::RootRegistry(message) if message.contains("nonexistent")));

        let unreadable = store_path("broken", 10).to_absolute_path();
        let unreadable_error = pin_root(state_dir.path(), "/nix/store", &FailingPathInfoService, &unreadable)
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
        let persisted: serde_json::Value =
            serde_json::from_slice(&std::fs::read(roots_path(state_dir.path())).expect("read registry"))
                .expect("parse registry");
        assert_eq!(persisted[&path]["schema_version"], ROOT_RECORD_SCHEMA_VERSION);
        assert_eq!(persisted[&path]["root_class"], "legacy-unmanaged");
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
                "last_transition_id": transition_id(TransitionIdentityInput {
                    logical_path: "/nix/store/bad",
                    class: GcRootClass::ExplicitPin,
                    owner_scope: "",
                    policy_blake3: &store_retention_policy_blake3(),
                    created_unix_s: CREATED_UNIX_S,
                    transition: "created",
                }),
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
        let first = resolve_project_generation(&BTreeMap::new(), registration.clone()).expect("first generation");
        assert_eq!(first.generation, Some(FIRST_GENERATION));

        let first_record = new_record(
            logical_path.clone(),
            GcRootSource::Build,
            first,
            CREATED_UNIX_S,
            "project-generation-registered",
        );
        let registry = BTreeMap::from([(logical_path.clone(), first_record)]);
        let reused =
            resolve_project_generation(&registry, registration.clone()).expect("same identity reuses generation");
        assert_eq!(reused.generation, Some(FIRST_GENERATION));

        let mut next_registration = registration;
        next_registration.generation_identity = Some("b3:lock-b".to_string());
        let next = resolve_project_generation(&registry, next_registration).expect("new identity advances generation");
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
        let renewed = resolve_shell_lease_renewal(&registry, &logical_path, registration.clone())
            .expect("matching shell lease renews");
        let renewed_lease = renewed.lease.expect("renewed lease facts");
        assert_eq!(renewed_lease.lease_id, "b3:lease-existing");
        assert_eq!(renewed_lease.renewal_count, NEXT_RENEWAL_COUNT);

        let mut exhausted_registry = registry.clone();
        let exhausted_record = exhausted_registry.get_mut(&logical_path).expect("existing lease record");
        exhausted_record.lease.as_mut().expect("existing lease facts").renewal_count =
            store_retention_runtime_policy().limits.max_lease_renewals;
        let exhausted_error = resolve_shell_lease_renewal(&exhausted_registry, &logical_path, registration)
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
        let error = resolve_shell_lease_renewal(&registry, &logical_path, missing_lease)
            .expect_err("shell registration without lease must fail");
        assert!(matches!(error, Error::RootRegistry(message) if message.contains("requires lease facts")));
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
        let mut registry = BTreeMap::new();
        registry.insert(path.clone(), record);
        save_registry(state_dir.path(), &registry).expect("save root registry");

        let removed = unpin_root(state_dir.path(), LogicalStorePathRef {
            logical_path: &path,
            store_dir: "/nix/store",
        })
        .expect("unpin root");

        assert!(removed.is_some());
        assert!(list_roots(state_dir.path()).expect("list roots").is_empty());
    }
}
