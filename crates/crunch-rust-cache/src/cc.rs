//! Local C/C++ compile-object records served by the existing Rust cache daemon.
//!
//! Object payloads and depfile manifests share the Rust cache's private state
//! and mutation lock, but never enter a build derivation or proof receipt.

use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crunch_rust_cache_core::cc::CC_RECORD_SCHEMA;
use crunch_rust_cache_core::cc::CC_REQUEST_SCHEMA;
use crunch_rust_cache_core::cc::CC_RESPONSE_SCHEMA;
use crunch_rust_cache_core::cc::CcObjectRecord;
use crunch_rust_cache_core::cc::CcOperation;
use crunch_rust_cache_core::cc::CcWireRequest;
use crunch_rust_cache_core::cc::CcWireResponse;
use crunch_rust_cache_core::cc::MAX_CC_DEPENDENCIES;
use crunch_rust_cache_core::cc::MAX_CC_OBJECT_BYTES;
use crunch_rust_cache_core::cc::MAX_CC_ROOTS;
use crunch_rust_cache_core::cc::admit_cc_reuse;
use crunch_rust_cache_core::cc::validate_cc_record;
use data_encoding::BASE64;
use serde_json::Value;
use tempfile::Builder;

use crate::CacheMutationLock;
use crate::Error;
use crate::MUTATION_LOCK_FILE;
use crate::PRIVATE_FILE_MODE;
use crate::RustCache;
use crate::hash_file_bounded;
use crate::read_bounded_optional;
use crate::sync_directory;
use crate::write_atomic_json;

pub(super) const CC_OBJECT_DIRECTORY: &str = "cc-objects";
pub(super) const CC_MANIFEST_DIRECTORY: &str = "cc-manifests";
const CC_OBJECT_SUFFIX: &str = ".object";
const CC_MANIFEST_SUFFIX: &str = ".json";
const CC_DISPOSITION_MISS: &str = "miss";
const CC_DISPOSITION_HIT: &str = "hit";
const CC_DISPOSITION_PUBLISHED: &str = "published";

const CC_POLICY_SCHEMA: &str = "mantle-cc-cache-policy-v2";
const CC_MAX_POLICY_BYTES: usize = 65_536;
const CC_MAX_COMPILER_BYTES: u64 = 134_217_728;
const CC_DEFAULT_POLICY_JSON: &str = include_str!("../../../config/cc-compile-cache/generated/cc-cache-policy.json");

/// The daemon's effective C authority; absent policy always means Off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CcAdmissionMode {
    Off,
    LocalRead,
    LocalReadWrite,
}

impl CcAdmissionMode {
    pub fn permits(self, operation: &CcOperation) -> bool {
        match (self, operation) {
            (Self::LocalRead, CcOperation::Manifest | CcOperation::Read) => true,
            (Self::LocalReadWrite, _) => true,
            _ => false,
        }
    }
}

/// Load a bounded policy export and bind the declared tools to actual bytes.
/// A default/off export cannot activate C requests by itself.
pub fn load_cc_admission_policy(
    policy_path: &Path,
    daemon_socket: &Path,
    receipt_dir: &Path,
) -> Result<CcAdmissionMode, Error> {
    let bytes = read_bounded_optional(policy_path)?.ok_or_else(|| Error::State("cc-policy-missing".to_string()))?;
    if bytes.len() > CC_MAX_POLICY_BYTES {
        return Err(Error::Bound("cc-policy-bytes-exceeded".to_string()));
    }
    parse_cc_admission_policy(&bytes, daemon_socket, receipt_dir)
}

fn parse_cc_admission_policy(bytes: &[u8], daemon_socket: &Path, receipt_dir: &Path) -> Result<CcAdmissionMode, Error> {
    if bytes.is_empty() || bytes.len() > CC_MAX_POLICY_BYTES {
        return Err(Error::Bound("cc-policy-bytes-invalid".to_string()));
    }
    let mut static_fields: Value =
        serde_json::from_slice(bytes).map_err(|error| Error::Json(format!("decode-cc-policy:{error}")))?;
    let expected: Value = serde_json::from_str(CC_DEFAULT_POLICY_JSON)
        .map_err(|error| Error::Json(format!("decode-checked-cc-policy:{error}")))?;
    if static_fields.pointer("/schema").and_then(Value::as_str) != Some(CC_POLICY_SCHEMA) {
        return Err(Error::State("cc-policy-schema-invalid".to_string()));
    }
    let static_driver = static_fields
        .pointer_mut("/driver")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| Error::State("cc-policy-driver-invalid".to_string()))?;
    let mode = take_cc_policy_field(static_driver, "mode")?;
    let driver_path = take_cc_policy_field(static_driver, "driver_path")?;
    let compiler_path = take_cc_policy_field(static_driver, "compiler_path")?;
    let socket_path = take_cc_policy_field(static_driver, "socket_path")?;
    let receipt_path = take_cc_policy_field(static_driver, "receipt_path")?;
    let driver_digest = take_cc_policy_field(static_driver, "driver_digest_blake3")?;
    let compiler_digest = take_cc_policy_field(static_driver, "compiler_digest_blake3")?;
    let platform_digest = take_cc_policy_field(static_driver, "platform_digest")?;
    let admission = match mode.as_str() {
        "off" => CcAdmissionMode::Off,
        "local-read" => CcAdmissionMode::LocalRead,
        "local-read-write" => CcAdmissionMode::LocalReadWrite,
        _ => return Err(Error::State("cc-policy-mode-invalid".to_string())),
    };
    if static_fields != expected {
        return Err(Error::State("cc-policy-static-contract-invalid".to_string()));
    }
    if admission == CcAdmissionMode::Off {
        if [
            &driver_path,
            &compiler_path,
            &socket_path,
            &receipt_path,
            &driver_digest,
            &compiler_digest,
            &platform_digest,
        ]
        .iter()
        .any(|value| !value.is_empty())
        {
            return Err(Error::State("cc-policy-off-has-authority".to_string()));
        }
        return Ok(CcAdmissionMode::Off);
    }
    let driver = validated_runtime_path(&driver_path)?;
    let compiler = validated_runtime_path(&compiler_path)?;
    let socket = validated_runtime_path(&socket_path)?;
    let receipt = validated_runtime_path(&receipt_path)?;
    if socket != daemon_socket || !receipt.starts_with(receipt_dir) {
        return Err(Error::State("cc-policy-endpoint-mismatch".to_string()));
    }
    if !is_lowercase_digest(&driver_digest)
        || !is_lowercase_digest(&compiler_digest)
        || !is_lowercase_digest(&platform_digest)
    {
        return Err(Error::State("cc-policy-tool-digest-invalid".to_string()));
    }
    verify_cc_policy_executable(driver, &driver_digest)?;
    verify_cc_policy_executable(compiler, &compiler_digest)?;
    Ok(admission)
}

fn take_cc_policy_field(fields: &mut serde_json::Map<String, Value>, name: &str) -> Result<String, Error> {
    let value = fields.get_mut(name).ok_or_else(|| Error::State(format!("cc-policy-field-missing:{name}")))?;
    let replacement = if name == "mode" { "off" } else { "" };
    match std::mem::replace(value, Value::String(replacement.to_string())) {
        Value::String(value) => Ok(value),
        _ => Err(Error::State(format!("cc-policy-field-invalid:{name}"))),
    }
}

fn validated_runtime_path(path: &str) -> Result<&Path, Error> {
    let parsed = Path::new(path);
    if !parsed.is_absolute()
        || path.as_bytes().contains(&0)
        || parsed
            .components()
            .any(|component| matches!(component, std::path::Component::CurDir | std::path::Component::ParentDir))
    {
        return Err(Error::State("cc-policy-path-invalid".to_string()));
    }
    Ok(parsed)
}

fn verify_cc_policy_executable(path: &Path, digest: &str) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Error::Io {
        context: "stat-cc-policy-executable".to_string(),
        source,
    })?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > CC_MAX_COMPILER_BYTES
        || metadata.permissions().mode() & 0o111 == 0
    {
        return Err(Error::State("cc-policy-executable-invalid".to_string()));
    }
    let (observed, size_bytes) = hash_file_bounded(path)?;
    if observed != digest || size_bytes != metadata.len() {
        return Err(Error::State("cc-policy-executable-digest-mismatch".to_string()));
    }
    Ok(())
}

impl RustCache {
    /// Serve one typed C request after the daemon has checked peer UID and frame bounds.
    pub fn serve_cc_request(&self, request: &CcWireRequest) -> Result<CcWireResponse, Error> {
        validate_request(request)?;
        let manifests = self.cache_dir.join(CC_MANIFEST_DIRECTORY);
        let objects = self.cache_dir.join(CC_OBJECT_DIRECTORY);
        let record_path = manifests.join(format!("{}{}", request.action_key, CC_MANIFEST_SUFFIX));
        match request.operation {
            CcOperation::Manifest => self.cc_manifest(request, &record_path),
            CcOperation::Read => self.cc_read(request, &record_path, &objects),
            CcOperation::Publish => self.cc_publish(request, &record_path, &objects),
        }
    }

    fn cc_manifest(&self, request: &CcWireRequest, path: &Path) -> Result<CcWireResponse, Error> {
        if !request.dependencies.is_empty() || request.object_base64.is_some() {
            return Err(Error::State("cc-manifest-request-has-payload".to_string()));
        }
        let Some(record) = load_record(path, &request.action_key)? else {
            return Ok(cc_response(request, CC_DISPOSITION_MISS, None, None));
        };
        Ok(cc_response(request, CC_DISPOSITION_HIT, Some(record), None))
    }

    fn cc_read(&self, request: &CcWireRequest, path: &Path, objects: &Path) -> Result<CcWireResponse, Error> {
        if request.object_base64.is_some() {
            return Err(Error::State("cc-read-request-has-object".to_string()));
        }
        let Some(record) = load_record(path, &request.action_key)? else {
            return Ok(cc_response(request, CC_DISPOSITION_MISS, None, None));
        };
        if !admit_cc_reuse(&record, &request.dependencies, MAX_CC_ROOTS)
            .map_err(|error| Error::Core(error.code().to_string()))?
        {
            return Ok(cc_response(request, CC_DISPOSITION_MISS, None, None));
        }
        let object_path = objects.join(format!("{}{}", record.object_digest_blake3, CC_OBJECT_SUFFIX));
        let Some(bytes) = read_verified_object(&object_path, &record)? else {
            return Ok(cc_response(request, CC_DISPOSITION_MISS, None, None));
        };
        Ok(cc_response(request, CC_DISPOSITION_HIT, Some(record), Some(BASE64.encode(&bytes))))
    }

    fn cc_publish(&self, request: &CcWireRequest, path: &Path, objects: &Path) -> Result<CcWireResponse, Error> {
        let encoded = request
            .object_base64
            .as_deref()
            .ok_or_else(|| Error::State("cc-publish-object-missing".to_string()))?;
        if encoded.len() > (MAX_CC_OBJECT_BYTES as usize).div_ceil(3) * 4 {
            return Err(Error::Bound("cc-object-encoded-too-large".to_string()));
        }
        let bytes = BASE64
            .decode(encoded.as_bytes())
            .map_err(|_| Error::State("cc-object-base64-invalid".to_string()))?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_CC_OBJECT_BYTES {
            return Err(Error::Bound("cc-object-size-invalid".to_string()));
        }
        let digest = blake3::hash(&bytes).to_hex().to_string();
        let record = CcObjectRecord {
            schema: CC_RECORD_SCHEMA.to_string(),
            action_key: request.action_key.clone(),
            dependencies: request.dependencies.clone(),
            object_digest_blake3: digest.clone(),
            object_bytes: u64::try_from(bytes.len())
                .map_err(|_| Error::Bound("cc-object-size-unrepresentable".to_string()))?,
        };
        validate_cc_record(&record, MAX_CC_ROOTS).map_err(|error| Error::Core(error.code().to_string()))?;
        let _lock = CacheMutationLock::acquire(&self.cache_dir.join(MUTATION_LOCK_FILE))?;
        let object_path = objects.join(format!("{digest}{CC_OBJECT_SUFFIX}"));
        publish_object(&object_path, &bytes, &record)?;
        write_atomic_json(path, &record)?;
        Ok(cc_response(request, CC_DISPOSITION_PUBLISHED, Some(record), None))
    }
}

fn validate_request(request: &CcWireRequest) -> Result<(), Error> {
    if request.schema != CC_REQUEST_SCHEMA {
        return Err(Error::State("cc-request-schema-invalid".to_string()));
    }
    if !is_lowercase_digest(&request.action_key) {
        return Err(Error::State("cc-action-key-invalid".to_string()));
    }
    if request.dependencies.len() > MAX_CC_DEPENDENCIES {
        return Err(Error::Bound("cc-request-dependencies-exceeded".to_string()));
    }
    Ok(())
}

fn is_lowercase_digest(digest: &str) -> bool {
    digest.len() == crate::BLAKE3_HEX_CHARS
        && digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn cc_response(
    request: &CcWireRequest,
    disposition: &str,
    record: Option<CcObjectRecord>,
    object_base64: Option<String>,
) -> CcWireResponse {
    CcWireResponse {
        schema: CC_RESPONSE_SCHEMA.to_string(),
        action_key: request.action_key.clone(),
        disposition: disposition.to_string(),
        record,
        object_base64,
    }
}

fn load_record(path: &Path, action_key: &str) -> Result<Option<CcObjectRecord>, Error> {
    let Some(bytes) = read_bounded_optional(path)? else {
        return Ok(None);
    };
    let record = serde_json::from_slice::<CcObjectRecord>(&bytes)
        .map_err(|error| Error::Json(format!("decode-cc-record:{error}")))?;
    validate_cc_record(&record, MAX_CC_ROOTS).map_err(|error| Error::Core(error.code().to_string()))?;
    if record.action_key != action_key {
        return Err(Error::State("cc-record-action-key-mismatch".to_string()));
    }
    Ok(Some(record))
}

fn read_verified_object(path: &Path, record: &CcObjectRecord) -> Result<Option<Vec<u8>>, Error> {
    let file = match OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::Io {
                context: "open-cc-object".to_string(),
                source,
            });
        }
    };
    let metadata = file.metadata().map_err(|source| Error::Io {
        context: "stat-cc-object".to_string(),
        source,
    })?;
    if !metadata.is_file() || metadata.len() != record.object_bytes || metadata.len() > MAX_CC_OBJECT_BYTES {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| Error::Bound("cc-object-size-unrepresentable".to_string()))?,
    );
    file.take(MAX_CC_OBJECT_BYTES + 1).read_to_end(&mut bytes).map_err(|source| Error::Io {
        context: "read-cc-object".to_string(),
        source,
    })?;
    if bytes.len() as u64 != record.object_bytes
        || blake3::hash(&bytes).to_hex().as_str() != record.object_digest_blake3
    {
        return Ok(None);
    }
    Ok(Some(bytes))
}

fn publish_object(path: &Path, bytes: &[u8], record: &CcObjectRecord) -> Result<(), Error> {
    if read_verified_object(path, record)?.is_some() {
        return Ok(());
    }
    let parent: &Path = path.parent().ok_or_else(|| Error::State("cc-object-parent-missing".to_string()))?;
    let mut staging = Builder::new().prefix("cc-object-").tempfile_in(parent).map_err(|source| Error::Io {
        context: "stage-cc-object".to_string(),
        source,
    })?;
    staging.as_file_mut().write_all(bytes).map_err(|source| Error::Io {
        context: "write-cc-object".to_string(),
        source,
    })?;
    staging.as_file_mut().sync_all().map_err(|source| Error::Io {
        context: "sync-cc-object".to_string(),
        source,
    })?;
    staging
        .as_file_mut()
        .set_permissions(fs::Permissions::from_mode(PRIVATE_FILE_MODE))
        .map_err(|source| Error::Io {
            context: "chmod-cc-object".to_string(),
            source,
        })?;
    match staging.persist_noclobber(path) {
        Ok(_) => sync_directory(parent)?,
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_verified_object(path, record)?.is_none() {
                return Err(Error::State("cc-object-content-conflict".to_string()));
            }
        }
        Err(error) => {
            return Err(Error::Io {
                context: "publish-cc-object".to_string(),
                source: error.error,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crunch_rust_cache_core::cc::CcDependency;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    use super::*;

    const ACTION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    async fn cache(root: &Path) -> RustCache {
        let directories = RedbDirectoryService::new("cc-cache-test".to_string(), RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        })
        .await
        .unwrap();
        RustCache::new(root.to_path_buf(), Arc::new(MemoryBlobService::default()), Arc::new(directories)).unwrap()
    }

    fn dependency(digest: &str) -> CcDependency {
        CcDependency {
            root_index: 0,
            relative_path: "include/header.h".to_string(),
            digest_blake3: digest.to_string(),
        }
    }

    fn request(operation: CcOperation, dependencies: Vec<CcDependency>, payload: Option<&[u8]>) -> CcWireRequest {
        CcWireRequest {
            schema: CC_REQUEST_SCHEMA.to_string(),
            operation,
            action_key: ACTION.to_string(),
            dependencies,
            object_base64: payload.map(|bytes| BASE64.encode(bytes)),
        }
    }

    #[tokio::test]
    async fn publication_and_manifest_read_restore_only_matching_dependency() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path()).await;
        let miss = cache.serve_cc_request(&request(CcOperation::Manifest, vec![], None)).unwrap();
        assert_eq!(miss.disposition, CC_DISPOSITION_MISS);
        let published = cache
            .serve_cc_request(&request(CcOperation::Publish, vec![dependency(DIGEST)], Some(b"real-object")))
            .unwrap();
        assert_eq!(published.disposition, CC_DISPOSITION_PUBLISHED);
        let candidate = cache.serve_cc_request(&request(CcOperation::Manifest, vec![], None)).unwrap();
        assert_eq!(candidate.record.unwrap().dependencies, vec![dependency(DIGEST)]);
        let hit = cache.serve_cc_request(&request(CcOperation::Read, vec![dependency(DIGEST)], None)).unwrap();
        assert_eq!(hit.disposition, CC_DISPOSITION_HIT);
        assert_eq!(BASE64.decode(hit.object_base64.unwrap().as_bytes()).unwrap(), b"real-object");
        let stale = cache.serve_cc_request(&request(CcOperation::Read, vec![dependency(ACTION)], None)).unwrap();
        assert_eq!(stale.disposition, CC_DISPOSITION_MISS);
        let unknown = cache.serve_cc_request(&request(CcOperation::Read, vec![], None)).unwrap();
        assert_eq!(unknown.disposition, CC_DISPOSITION_MISS);
    }

    #[tokio::test]
    async fn tampered_payload_never_returns_a_hit() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path()).await;
        let published = cache
            .serve_cc_request(&request(CcOperation::Publish, vec![dependency(DIGEST)], Some(b"verified")))
            .unwrap();
        let digest = published.record.unwrap().object_digest_blake3;
        fs::write(cache.cache_dir.join(CC_OBJECT_DIRECTORY).join(format!("{digest}{CC_OBJECT_SUFFIX}")), b"tampered")
            .unwrap();
        let response = cache.serve_cc_request(&request(CcOperation::Read, vec![dependency(DIGEST)], None)).unwrap();
        assert_eq!(response.disposition, CC_DISPOSITION_MISS);
        assert!(response.object_base64.is_none());
    }

    #[tokio::test]
    async fn invalid_manifest_path_and_object_are_rejected_without_publication() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path()).await;
        let traversal = CcDependency {
            relative_path: "../outside.h".to_string(),
            ..dependency(DIGEST)
        };
        assert!(cache.serve_cc_request(&request(CcOperation::Publish, vec![traversal], Some(b"object"))).is_err());
        assert!(cache.serve_cc_request(&request(CcOperation::Publish, vec![dependency(DIGEST)], None)).is_err());
        let miss = cache.serve_cc_request(&request(CcOperation::Manifest, vec![], None)).unwrap();
        assert_eq!(miss.disposition, CC_DISPOSITION_MISS);
    }

    fn activated_policy(root: &Path, mode: &str) -> (Value, std::path::PathBuf, std::path::PathBuf) {
        let driver_path = root.join("mantle-cc-cache-driver");
        let compiler_path = root.join("x86_64-linux-musl-cc");
        fs::write(&driver_path, b"driver executable bytes").unwrap();
        fs::write(&compiler_path, b"compiler executable bytes").unwrap();
        fs::set_permissions(&driver_path, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(&compiler_path, fs::Permissions::from_mode(0o700)).unwrap();
        let socket_path = root.join("cache.sock");
        let receipt_dir = root.join("receipts");
        fs::create_dir(&receipt_dir).unwrap();
        let mut policy: Value = serde_json::from_str(CC_DEFAULT_POLICY_JSON).unwrap();
        let driver = policy.get_mut("driver").unwrap().as_object_mut().unwrap();
        for (name, value) in [
            ("mode", mode.to_string()),
            ("driver_path", driver_path.to_str().unwrap().to_string()),
            ("compiler_path", compiler_path.to_str().unwrap().to_string()),
            ("socket_path", socket_path.to_str().unwrap().to_string()),
            ("receipt_path", receipt_dir.join("cc.json").to_str().unwrap().to_string()),
            ("driver_digest_blake3", blake3::hash(b"driver executable bytes").to_hex().to_string()),
            ("compiler_digest_blake3", blake3::hash(b"compiler executable bytes").to_hex().to_string()),
            ("platform_digest", ACTION.to_string()),
        ] {
            driver.insert(name.to_string(), Value::String(value));
        }
        (policy, socket_path, receipt_dir)
    }

    #[test]
    fn policy_activates_only_bound_executables_and_permissioned_operations() {
        let root = tempfile::tempdir().unwrap();
        let policy_path = root.path().join("cc-policy.json");
        fs::write(&policy_path, CC_DEFAULT_POLICY_JSON).unwrap();
        let socket = root.path().join("cache.sock");
        let receipts = root.path().join("receipts");
        let mode = load_cc_admission_policy(&policy_path, &socket, &receipts).unwrap();
        assert!(!mode.permits(&CcOperation::Manifest));
        assert!(!mode.permits(&CcOperation::Publish));

        let (mut selected, socket, receipts) = activated_policy(root.path(), "local-read");
        fs::write(&policy_path, serde_json::to_vec(&selected).unwrap()).unwrap();
        let mode = load_cc_admission_policy(&policy_path, &socket, &receipts).unwrap();
        assert!(mode.permits(&CcOperation::Read));
        assert!(!mode.permits(&CcOperation::Publish));
        selected["driver"]["mode"] = Value::String("local-read-write".to_string());
        fs::write(&policy_path, serde_json::to_vec(&selected).unwrap()).unwrap();
        let mode = load_cc_admission_policy(&policy_path, &socket, &receipts).unwrap();
        assert!(mode.permits(&CcOperation::Publish));
        fs::write(root.path().join("x86_64-linux-musl-cc"), b"changed compiler bytes").unwrap();
        assert!(load_cc_admission_policy(&policy_path, &socket, &receipts).is_err());
    }

    #[test]
    fn policy_rejects_wrong_endpoint_digest_and_static_contract_drift() {
        let root = tempfile::tempdir().unwrap();
        let (selected, socket, receipts) = activated_policy(root.path(), "local-read-write");
        let bytes = serde_json::to_vec(&selected).unwrap();
        assert!(parse_cc_admission_policy(&bytes, &root.path().join("other.sock"), &receipts).is_err());
        let mut wrong_digest = selected.clone();
        wrong_digest["driver"]["driver_digest_blake3"] = Value::String(ACTION.to_string());
        assert!(parse_cc_admission_policy(&serde_json::to_vec(&wrong_digest).unwrap(), &socket, &receipts).is_err());
        let mut downgraded = selected.clone();
        downgraded["schema"] = Value::String("mantle-cc-cache-policy-v1".to_string());
        assert!(parse_cc_admission_policy(&serde_json::to_vec(&downgraded).unwrap(), &socket, &receipts).is_err());
        let mut unbounded_probe = selected.clone();
        unbounded_probe["probe"]["max_diagnostic_bytes"] = Value::Number(262145.into());
        assert!(parse_cc_admission_policy(&serde_json::to_vec(&unbounded_probe).unwrap(), &socket, &receipts).is_err());
        let mut weak_proof = selected;
        weak_proof["receipts"]["proof_evidence"] = Value::Bool(true);
        assert!(parse_cc_admission_policy(&serde_json::to_vec(&weak_proof).unwrap(), &socket, &receipts).is_err());
    }
}
