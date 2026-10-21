//! Operator-local build and store-source name authority, not payload authority.
//!
//! The client and receiver each load their own configured file; a client's path
//! is never interpreted as a receiver path. Relative state paths retain process
//! CWD semantics. Source identities are exact canonical URLs or base paths.

use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crunch_remote_core::endpoint::EndpointBinding;
use crunch_remote_core::endpoint::EndpointKind;
use crunch_remote_core::endpoint::EndpointReceiverRequirements;
use crunch_remote_core::endpoint::MAX_ENDPOINT_BINDINGS;
use crunch_remote_core::endpoint::MAX_ENDPOINT_SCOPE_BYTES;
use crunch_remote_core::endpoint::MAX_ENDPOINT_TRUST_BYTES;
use serde::Deserialize;

use super::endpoint::EndpointAdmissionGuard;
use super::endpoint::EndpointBindingStore;
use crate::errors::RunError;

pub const REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA: &str = "mantle-remote-endpoint-binding-config-v1";
pub const MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES: u64 = 65_536;

/// Local configuration is authoritative only for name admission. Existing
/// ticket, replay, signature, content and output checks remain mandatory.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteEndpointBindingConfig {
    schema: String,
    pub endpoint_state_directory: PathBuf,
    pub build: Option<BuildEndpointBindingConfig>,
    #[serde(deserialize_with = "deserialize_source_bindings")]
    pub source_bindings: Vec<SourceEndpointBindingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildEndpointBindingConfig {
    pub endpoint_binding: EndpointBinding,
    pub scope: String,
    pub trust_requirement: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEndpointBindingConfig {
    pub source: String,
    pub endpoint_binding: EndpointBinding,
    pub scope: String,
    pub trust_requirement: String,
}

impl RemoteEndpointBindingConfig {
    pub fn load(path: &Path) -> Result<Self, RunError> {
        let operation_plan = super::operation::plan(super::operation::RemoteOperation::EndpointConfiguration)?;
        let bytes = private_read(path)?;
        let config: Self = serde_json::from_slice(&bytes).map_err(|_| config_error("malformed"))?;
        config.validate()?;
        super::operation::classify(&operation_plan, super::operation::RemoteOperationFacts::EndpointConfiguration {
            bytes: bytes.len(),
            bytes_max: MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES,
            schema: &config.schema,
            expected_schema: REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA,
            bindings: config.source_bindings.len() + usize::from(config.build.is_some()),
            bindings_max: MAX_ENDPOINT_BINDINGS,
        })?;
        Ok(config)
    }

    /// Assemble independently configured operator facts without resolving or
    /// creating a durable binding. Admission remains per operation.
    pub fn from_build(endpoint_state_directory: PathBuf, build: BuildEndpointBindingConfig) -> Result<Self, RunError> {
        let config = Self {
            schema: REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA.into(),
            endpoint_state_directory,
            build: Some(build),
            source_bindings: Vec::new(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn build_binding(&self) -> Result<&EndpointBinding, RunError> {
        self.validate()?;
        Ok(&self.build_config()?.endpoint_binding)
    }

    /// Short pre-prepare check only. The receiver must independently load its
    /// own config and call `admit_build` before any manifest or payload work.
    // r[impl remote_builds.single_binding_resolution]
    pub fn verify_build(&self, now_unix_s: u64, store_prefix: &str) -> Result<EndpointBinding, RunError> {
        self.validate()?;
        let build = self.build_config()?;
        EndpointBindingStore::open(&self.endpoint_state_directory)?.verify_received(
            &build.endpoint_binding,
            requirements(EndpointKind::BuilderTicket, &build.scope, &build.trust_requirement, store_prefix),
            now_unix_s,
        )
    }

    /// Match the receiver's exact descriptor, then re-read durable authority.
    /// Retain the shared binding lease through independent ticket checks and
    /// effects; this guard never holds or replaces a ticket transaction lock.
    // r[impl remote_builds.single_binding_resolution]
    // r[impl remote_builds.binding_revocation]
    pub fn admit_build(
        &self,
        presented: &EndpointBinding,
        now_unix_s: u64,
        store_prefix: &str,
    ) -> Result<EndpointAdmissionGuard, RunError> {
        self.validate()?;
        let build = self.build_config()?;
        if presented != &build.endpoint_binding {
            return Err(config_error("descriptor-mismatch"));
        }
        EndpointBindingStore::open(&self.endpoint_state_directory)?.admit_received(
            presented,
            requirements(EndpointKind::BuilderTicket, &build.scope, &build.trust_requirement, store_prefix),
            now_unix_s,
        )
    }

    fn build_config(&self) -> Result<&BuildEndpointBindingConfig, RunError> {
        self.build.as_ref().ok_or_else(|| config_error("build-missing"))
    }

    fn validate(&self) -> Result<(), RunError> {
        if self.schema != REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA {
            return Err(config_error("schema-unsupported"));
        }
        if self.endpoint_state_directory.as_os_str().is_empty() {
            return Err(config_error("state-directory-missing"));
        }
        if self.source_bindings.len() > MAX_ENDPOINT_BINDINGS - usize::from(self.build.is_some()) {
            return Err(config_error("binding-limit-exceeded"));
        }
        if let Some(build) = &self.build {
            validate_descriptor(&build.endpoint_binding, &build.scope, &build.trust_requirement)?;
            if build.endpoint_binding.spec.kind != EndpointKind::BuilderTicket {
                return Err(config_error("kind-mismatch"));
            }
        }
        let mut sources = BTreeSet::new();
        for row in &self.source_bindings {
            validate_descriptor(&row.endpoint_binding, &row.scope, &row.trust_requirement)?;
            validate_source(row)?;
            if !sources.insert(row.source.as_str()) {
                return Err(config_error("source-duplicate"));
            }
        }
        Ok(())
    }

    fn admit_source_at(
        &self,
        source: crunch_store::StoreSource<'_>,
        store_prefix: &str,
        now_unix_s: u64,
    ) -> Result<EndpointAdmissionGuard, RunError> {
        self.validate()?;
        let canonical_path;
        let (identity, kind) = match source {
            crunch_store::StoreSource::Substituter(url) => (url, EndpointKind::Substituter),
            crunch_store::StoreSource::BaseLayer(path) => {
                canonical_path = std::fs::canonicalize(path).map_err(|_| config_error("source-invalid"))?;
                (canonical_path.to_str().ok_or_else(|| config_error("source-invalid"))?, EndpointKind::BaseLayer)
            }
        };
        let row = self
            .source_bindings
            .iter()
            .find(|row| row.source == identity)
            .ok_or_else(|| config_error("source-missing"))?;
        // The adapter chooses the expected kind, never the configured row.
        if row.endpoint_binding.spec.kind != kind {
            return Err(config_error("kind-mismatch"));
        }
        EndpointBindingStore::open(&self.endpoint_state_directory)?.admit_received(
            &row.endpoint_binding,
            requirements(kind, &row.scope, &row.trust_requirement, store_prefix),
            now_unix_s,
        )
    }
}

impl crunch_store::StoreSourceLease for EndpointAdmissionGuard {}

impl crunch_store::StoreSourceAdmission for RemoteEndpointBindingConfig {
    // r[impl remote_builds.single_binding_resolution]
    // r[impl remote_builds.binding_revocation]
    fn admit(
        &self,
        source: crunch_store::StoreSource<'_>,
        store_prefix: &str,
    ) -> Result<Arc<dyn crunch_store::StoreSourceLease>, crunch_store::Error> {
        let denial = || crunch_store::Error::Store("source endpoint admission denied".into());
        let now_unix_s = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| denial())?.as_secs();
        let guard = self.admit_source_at(source, store_prefix, now_unix_s).map_err(|_| denial())?;
        Ok(Arc::new(guard))
    }
}

fn requirements<'a>(
    kind: EndpointKind,
    scope: &'a str,
    trust_requirement: &'a str,
    store_prefix: &'a str,
) -> EndpointReceiverRequirements<'a> {
    EndpointReceiverRequirements {
        kind,
        scope,
        trust_requirement,
        store_prefix,
    }
}

fn validate_descriptor(binding: &EndpointBinding, scope: &str, trust_requirement: &str) -> Result<(), RunError> {
    binding.validate().map_err(|_| config_error("descriptor-invalid"))?;
    if !valid_identity(scope, MAX_ENDPOINT_SCOPE_BYTES) || !valid_identity(trust_requirement, MAX_ENDPOINT_TRUST_BYTES)
    {
        return Err(config_error("requirements-invalid"));
    }
    Ok(())
}

fn validate_source(row: &SourceEndpointBindingConfig) -> Result<(), RunError> {
    let canonical = match row.endpoint_binding.spec.kind {
        EndpointKind::Substituter => {
            url::Url::parse(&row.source).map(|url| url.as_str() == row.source).unwrap_or(false)
        }
        EndpointKind::BaseLayer => {
            let path = Path::new(&row.source);
            path.is_absolute()
                && std::fs::canonicalize(path)
                    .is_ok_and(|canonical| canonical.as_os_str() == path.as_os_str() && canonical.is_dir())
        }
        EndpointKind::BuilderTicket | EndpointKind::Transfer => return Err(config_error("kind-mismatch")),
    };
    if !canonical {
        return Err(config_error("source-invalid"));
    }
    Ok(())
}

fn deserialize_source_bindings<'de, D>(deserializer: D) -> Result<Vec<SourceEndpointBindingConfig>, D::Error>
where D: serde::Deserializer<'de> {
    struct BoundedSources;
    impl<'de> serde::de::Visitor<'de> for BoundedSources {
        type Value = Vec<SourceEndpointBindingConfig>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded source binding array")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where A: serde::de::SeqAccess<'de> {
            let mut rows = Vec::new();
            while let Some(row) = sequence.next_element()? {
                if rows.len() >= MAX_ENDPOINT_BINDINGS {
                    return Err(serde::de::Error::custom("source-binding-limit-exceeded"));
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
    deserializer.deserialize_seq(BoundedSources)
}

fn valid_identity(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && value.bytes().all(|byte| byte.is_ascii_graphic())
}

#[cfg(unix)]
fn private_read(path: &Path) -> Result<Vec<u8>, RunError> {
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    let file: File = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| config_error("open-failed"))?;
    let metadata = file.metadata().map_err(|_| config_error("metadata-failed"))?;
    // Match authority.rs: kernel-owned observation of the process's effective
    // owner, without accepting ownership supplied by the configuration itself.
    let owner = std::fs::metadata("/proc/self").map_err(|_| config_error("owner-unavailable"))?.uid();
    if !metadata.is_file() || metadata.uid() != owner || metadata.mode() & 0o077 != 0 {
        return Err(config_error("target-unsafe"));
    }
    if metadata.len() > MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES {
        return Err(config_error("size-limit-exceeded"));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| config_error("read-failed"))?;
    if bytes.len() as u64 > MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES {
        return Err(config_error("size-limit-exceeded"));
    }
    Ok(bytes)
}

#[cfg(not(unix))]
fn private_read(_path: &Path) -> Result<Vec<u8>, RunError> {
    Err(config_error("private-config-requires-unix"))
}

fn config_error(category: &str) -> RunError {
    RunError::Internal(format!("remote-endpoint-binding-config-{category}"))
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::os::unix::fs::symlink;

    use crunch_remote_core::endpoint::EndpointBindingSpec;
    use serde_json::Value;
    use serde_json::json;

    use super::*;

    fn spec(name: &str) -> EndpointBindingSpec {
        EndpointBindingSpec {
            kind: EndpointKind::BuilderTicket,
            name: name.into(),
            scope: "farm-a".into(),
            trust_requirement: "existing-ticket-policy".into(),
            store_prefix: "/mantle/store".into(),
            expires_unix_s: 100,
        }
    }

    fn document(state_dir: &Path, binding: &EndpointBinding) -> Value {
        json!({
            "schema": REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA,
            "endpoint_state_directory": state_dir,
            "build": {
                "endpoint_binding": binding,
                "scope": "farm-a",
                "trust_requirement": "existing-ticket-policy",
            },
            "source_bindings": [],
        })
    }

    fn write_private(path: &Path, bytes: &[u8]) {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt as _;
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).unwrap();
        file.write_all(bytes).unwrap();
    }

    fn load_document(path: &Path, value: &Value) -> Result<RemoteEndpointBindingConfig, RunError> {
        write_private(path, &serde_json::to_vec(value).unwrap());
        RemoteEndpointBindingConfig::load(path)
    }

    #[test]
    fn private_loader_rejects_unsafe_files_and_accepts_read_only_private_configuration() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let path = temporary.path().join("config.json");
        let value = document(temporary.path(), &binding);
        assert_eq!(load_document(&path, &value).unwrap().build_binding().unwrap(), &binding);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert_eq!(
            RemoteEndpointBindingConfig::load(&path).unwrap().verify_build(2, "/mantle/store").unwrap(),
            binding
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(RemoteEndpointBindingConfig::load(&path).is_err());
        let link = temporary.path().join("config-link.json");
        symlink(&path, &link).unwrap();
        assert!(RemoteEndpointBindingConfig::load(&link).is_err());
        assert!(RemoteEndpointBindingConfig::load(temporary.path()).is_err());
        assert!(RemoteEndpointBindingConfig::load(&temporary.path().join("absent")).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES + 1).unwrap();
        assert!(RemoteEndpointBindingConfig::load(&path).is_err());
    }

    #[test]
    fn strict_json_rejects_duplicate_unknown_absent_and_invalid_authority_fields() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let path = temporary.path().join("config.json");
        let valid = document(temporary.path(), &binding);
        let serialized = serde_json::to_string(&valid).unwrap();
        let duplicate = format!("{{\"source_bindings\":[],{}", &serialized[1..]);
        write_private(&path, duplicate.as_bytes());
        assert!(RemoteEndpointBindingConfig::load(&path).is_err());
        write_private(&path, b"{invalid-json");
        assert!(RemoteEndpointBindingConfig::load(&path).is_err());
        for (field, replacement) in [
            ("schema", json!("unsupported-v1")),
            ("endpoint_state_directory", json!("")),
            ("unknown", json!(true)),
        ] {
            let mut invalid = valid.clone();
            invalid[field] = replacement;
            assert!(load_document(&path, &invalid).is_err(), "accepted field {field}");
        }
        for field in ["endpoint_binding", "scope", "trust_requirement"] {
            let mut missing = valid.clone();
            missing["build"].as_object_mut().unwrap().remove(field);
            assert!(load_document(&path, &missing).is_err());
        }
        for (field, replacement) in [
            ("scope", json!("contains whitespace")),
            ("trust_requirement", json!("")),
            ("unknown", json!(true)),
        ] {
            let mut invalid = valid.clone();
            invalid["build"][field] = replacement;
            assert!(load_document(&path, &invalid).is_err());
        }
        for (field, replacement) in [("generation", json!(0)), ("unknown", json!(true))] {
            let mut invalid = valid.clone();
            invalid["build"]["endpoint_binding"][field] = replacement;
            assert!(load_document(&path, &invalid).is_err());
        }
        for (field, replacement) in [
            ("name", json!("../forged")),
            ("kind", json!("transfer")),
            ("scope", json!("")),
            ("trust_requirement", json!("")),
            ("store_prefix", json!("relative/store")),
            ("expires_unix_s", json!(0)),
            ("unknown", json!(true)),
        ] {
            let mut invalid = valid.clone();
            invalid["build"]["endpoint_binding"]["spec"][field] = replacement;
            assert!(load_document(&path, &invalid).is_err());
        }
        let mut relative = valid;
        relative["endpoint_state_directory"] = json!("relative-state");
        assert_eq!(load_document(&path, &relative).unwrap().endpoint_state_directory, PathBuf::from("relative-state"));
    }

    #[test]
    fn receiver_uses_independent_requirements_and_exact_descriptor_before_effects() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let other = store.create(spec("builder-b"), None, 1).unwrap();
        let path = temporary.path().join("config.json");
        let valid = document(temporary.path(), &binding);
        let config = load_document(&path, &valid).unwrap();
        assert_eq!(config.verify_build(2, "/mantle/store").unwrap(), binding);
        assert!(config.admit_build(&other, 2, "/mantle/store").is_err());
        let mut forged = binding.clone();
        forged.spec.expires_unix_s = 200;
        assert!(config.admit_build(&forged, 2, "/mantle/store").is_err());
        assert!(config.admit_build(&binding, 2, "/nix/store").is_err());
        assert!(config.admit_build(&binding, 100, "/mantle/store").is_err());
        for field in ["scope", "trust_requirement"] {
            let mut mismatched = valid.clone();
            mismatched["build"][field] = json!("independent-other-policy");
            let receiver = load_document(&path, &mismatched).unwrap();
            assert!(receiver.verify_build(2, "/mantle/store").is_err());
            assert!(receiver.admit_build(&binding, 2, "/mantle/store").is_err());
        }
        // Even a descriptor matching this file must exist in the real table.
        let mut forged = valid;
        forged["build"]["endpoint_binding"]["spec"]["name"] = json!("unregistered");
        let receiver = load_document(&path, &forged).unwrap();
        assert!(receiver.verify_build(2, "/mantle/store").is_err());
        assert!(receiver.admit_build(receiver.build_binding().unwrap(), 2, "/mantle/store").is_err());
        let guard = config.admit_build(&binding, 2, "/mantle/store").unwrap();
        assert_eq!(guard.binding(), &binding);
        let payload = temporary.path().join("admitted-content");
        fs::write(&payload, b"ticket-and-content-checks-remain-required").unwrap();
        drop(guard);
        store.revoke(&binding).unwrap();
        assert!(config.verify_build(2, "/mantle/store").is_err());
        if let Ok(_guard) = config.admit_build(&binding, 2, "/mantle/store") {
            fs::write(&payload, b"incorrect-overwrite").unwrap();
            panic!("revoked receiver reached payload work");
        }
        assert_eq!(fs::read(&payload).unwrap(), b"ticket-and-content-checks-remain-required");
        let recreated = store.create(binding.spec.clone(), Some(binding.generation), 2).unwrap();
        assert!(config.verify_build(2, "/mantle/store").is_err());
        assert!(config.admit_build(&binding, 2, "/mantle/store").is_err());
        assert!(config.admit_build(&recreated, 2, "/mantle/store").is_err());
        let refreshed = load_document(&path, &document(temporary.path(), &recreated)).unwrap();
        assert_eq!(refreshed.verify_build(2, "/mantle/store").unwrap(), recreated);
        assert_eq!(refreshed.admit_build(&recreated, 2, "/mantle/store").unwrap().binding(), &recreated);
    }

    fn source_document(state_dir: &Path, binding: &EndpointBinding, source: &str) -> Value {
        json!({
            "schema": REMOTE_ENDPOINT_BINDING_CONFIG_SCHEMA,
            "endpoint_state_directory": state_dir,
            "source_bindings": [{
                "source": source,
                "endpoint_binding": binding,
                "scope": "farm-a",
                "trust_requirement": "existing-ticket-policy",
            }],
        })
    }

    #[test]
    fn cache_only_configuration_has_no_implicit_build_authority() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let binding = store.create(spec("builder-a"), None, 1).unwrap();
        let mut value = document(temporary.path(), &binding);
        value.as_object_mut().unwrap().remove("build");
        let path = temporary.path().join("config.json");
        let config = load_document(&path, &value).unwrap();
        assert!(config.build_binding().is_err());
        assert!(config.verify_build(2, "/mantle/store").is_err());
        assert!(config.admit_build(&binding, 2, "/mantle/store").is_err());
        for field in ["endpoint_binding", "build_scope", "build_trust_requirement"] {
            let mut legacy = value.clone();
            legacy[field] = json!("forbidden-top-level-alias");
            assert!(load_document(&path, &legacy).is_err());
        }
    }

    #[test]
    fn sources_require_exact_canonical_identity_and_allowed_explicit_kind() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let mut source_spec = spec("cache-a");
        source_spec.kind = EndpointKind::Substituter;
        let binding = store.create(source_spec, None, 1).unwrap();
        let path = temporary.path().join("config.json");
        let valid = source_document(temporary.path(), &binding, "https://cache.example/");
        let config = load_document(&path, &valid).unwrap();
        assert_eq!(
            config
                .admit_source_at(crunch_store::StoreSource::Substituter("https://cache.example/"), "/mantle/store", 2)
                .unwrap()
                .binding(),
            &binding
        );
        for identity in [
            "https://cache.example",
            "https://CACHE.example/",
            "https://cache.example:443/",
            "not a URL",
            "",
        ] {
            let mut invalid = valid.clone();
            invalid["source_bindings"][0]["source"] = json!(identity);
            assert!(load_document(&path, &invalid).is_err(), "accepted {identity}");
        }
        let mut duplicate = valid.clone();
        duplicate["source_bindings"].as_array_mut().unwrap().push(valid["source_bindings"][0].clone());
        assert!(load_document(&path, &duplicate).is_err());
        for kind in ["builder-ticket", "transfer", "unknown"] {
            let mut invalid = valid.clone();
            invalid["source_bindings"][0]["endpoint_binding"]["spec"]["kind"] = json!(kind);
            assert!(load_document(&path, &invalid).is_err());
        }
        for field in ["scope", "trust_requirement", "unknown"] {
            let mut invalid = valid.clone();
            invalid["source_bindings"][0][field] = json!("");
            assert!(load_document(&path, &invalid).is_err());
        }
        for (field, maximum) in [
            ("scope", MAX_ENDPOINT_SCOPE_BYTES),
            ("trust_requirement", MAX_ENDPOINT_TRUST_BYTES),
        ] {
            let mut oversized = valid.clone();
            oversized["source_bindings"][0][field] = json!("x".repeat(maximum + 1));
            assert!(load_document(&path, &oversized).is_err());
        }
        for field in ["source", "endpoint_binding", "scope", "trust_requirement"] {
            let mut missing = valid.clone();
            missing["source_bindings"][0].as_object_mut().unwrap().remove(field);
            assert!(load_document(&path, &missing).is_err());
        }
        let mut oversized_rows = valid.clone();
        oversized_rows["source_bindings"] =
            Value::Array(vec![valid["source_bindings"][0].clone(); MAX_ENDPOINT_BINDINGS + 1]);
        assert!(serde_json::from_value::<RemoteEndpointBindingConfig>(oversized_rows).is_err());
        let sensitive_source = "https://private.example/?token=operator-secret";
        let denial = crunch_store::StoreSourceAdmission::admit(
            &config,
            crunch_store::StoreSource::Substituter(sensitive_source),
            "/mantle/store",
        )
        .unwrap_err();
        match denial {
            crunch_store::Error::Store(message) => {
                assert!(!message.contains("private.example"));
                assert!(!message.contains("operator-secret"));
            }
            other => panic!("unexpected source denial variant: {other:?}"),
        }
        let mut too_many = config;
        too_many.source_bindings = vec![too_many.source_bindings[0].clone(); MAX_ENDPOINT_BINDINGS + 1];
        assert!(too_many.validate().is_err());
        let mut oversized = valid;
        oversized["source_bindings"][0]["source"] =
            json!("x".repeat(MAX_REMOTE_ENDPOINT_BINDING_CONFIG_BYTES as usize));
        assert!(load_document(&path, &oversized).is_err());
    }

    #[test]
    fn base_source_identity_is_exact_canonical_directory_not_a_manifest() {
        let temporary = tempfile::tempdir().unwrap();
        let base = temporary.path().join("base");
        fs::create_dir(&base).unwrap();
        let canonical = fs::canonicalize(&base).unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let mut source_spec = spec("base-a");
        source_spec.kind = EndpointKind::BaseLayer;
        let binding = store.create(source_spec, None, 1).unwrap();
        let valid = source_document(temporary.path(), &binding, canonical.to_str().unwrap());
        let path = temporary.path().join("config.json");
        let config = load_document(&path, &valid).unwrap();
        assert_eq!(
            config
                .admit_source_at(crunch_store::StoreSource::BaseLayer(&base), "/mantle/store", 2)
                .unwrap()
                .binding(),
            &binding
        );
        assert!(
            config
                .admit_source_at(
                    crunch_store::StoreSource::Substituter(canonical.to_str().unwrap()),
                    "/mantle/store",
                    2
                )
                .is_err()
        );
        let alias = temporary.path().join("base-alias");
        symlink(&base, &alias).unwrap();
        for identity in [
            alias.to_str().unwrap().to_owned(),
            format!("{}/.", canonical.display()),
            format!("{}/", canonical.display()),
            "relative/base".into(),
        ] {
            let mut invalid = valid.clone();
            invalid["source_bindings"][0]["source"] = json!(identity);
            assert!(load_document(&path, &invalid).is_err());
        }
        fs::write(base.join("manifest.json"), b"not a manifest").unwrap();
        assert!(config.admit_source_at(crunch_store::StoreSource::BaseLayer(&alias), "/mantle/store", 2).is_ok());
        let mut file_source = valid;
        file_source["source_bindings"][0]["source"] = json!(base.join("manifest.json"));
        assert!(load_document(&path, &file_source).is_err());
    }

    #[test]
    fn source_receiver_rejects_independent_policy_mismatch_forgery_stale_and_retracted_authority() {
        let temporary = tempfile::tempdir().unwrap();
        let store = EndpointBindingStore::open(temporary.path()).unwrap();
        let mut source_spec = spec("cache-a");
        source_spec.kind = EndpointKind::Substituter;
        let binding = store.create(source_spec, None, 1).unwrap();
        let path = temporary.path().join("config.json");
        let url = "https://cache.example/";
        let valid = source_document(temporary.path(), &binding, url);
        let config = load_document(&path, &valid).unwrap();
        assert!(config.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2).is_ok());
        assert!(
            config
                .admit_source_at(crunch_store::StoreSource::Substituter("https://other.example/"), "/mantle/store", 2)
                .is_err()
        );
        assert!(config.admit_source_at(crunch_store::StoreSource::Substituter(url), "/nix/store", 2).is_err());
        assert!(config.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 100).is_err());
        for field in ["scope", "trust_requirement"] {
            let mut changed = valid.clone();
            changed["source_bindings"][0][field] = json!("independent-other-policy");
            let receiver = load_document(&path, &changed).unwrap();
            assert!(receiver.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2).is_err());
        }
        for (field, value) in [("name", json!("unregistered")), ("expires_unix_s", json!(200))] {
            let mut forged = valid.clone();
            forged["source_bindings"][0]["endpoint_binding"]["spec"][field] = value;
            let receiver = load_document(&path, &forged).unwrap();
            assert!(receiver.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2).is_err());
        }
        store.revoke(&binding).unwrap();
        assert!(config.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2).is_err());
        let recreated = store.create(binding.spec.clone(), Some(binding.generation), 2).unwrap();
        assert!(config.admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2).is_err());
        let refreshed = load_document(&path, &source_document(temporary.path(), &recreated, url)).unwrap();
        assert_eq!(
            refreshed
                .admit_source_at(crunch_store::StoreSource::Substituter(url), "/mantle/store", 2)
                .unwrap()
                .binding(),
            &recreated
        );
    }
}
