#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Bounded Unix-socket shell for the Mantle Rust compiler cache protocol.
//!
//! `crunch-rust-cache-core::wrapper` owns canonical data and decisions. This
//! crate owns policy loading, peer admission, framing, sandbox execution,
//! cache orchestration, output publication, and receipt persistence.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::CString;
use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crunch_rust_cache::PublishRequest;
use crunch_rust_cache::RustCache;
use crunch_rust_cache::shared::DirectoryRustResultSource;
use crunch_rust_cache::shared::HttpRustResultSource;
use crunch_rust_cache::shared::RustResultSource;
use crunch_rust_cache::shared::SHARED_CACHE_HIT;
use crunch_rust_cache::shared::SharedPublishRequest;
use crunch_rust_cache::shared::SharedRestoreRequest;
use crunch_rust_cache::shared::SharedRustCachePolicy;
use crunch_rust_cache_core::LocalCachePolicy;
use crunch_rust_cache_core::shared::RustResultProducerIdentity;
use crunch_rust_cache_core::wrapper::CurrentManifestFacts;
use crunch_rust_cache_core::wrapper::DeclaredInput;
use crunch_rust_cache_core::wrapper::DeclaredInputKind;
use crunch_rust_cache_core::wrapper::WRAPPER_RESPONSE_SCHEMA;
use crunch_rust_cache_core::wrapper::WrapperArtifactKind;
use crunch_rust_cache_core::wrapper::WrapperBypassClass;
use crunch_rust_cache_core::wrapper::WrapperDaemonPolicy;
use crunch_rust_cache_core::wrapper::WrapperDecision;
use crunch_rust_cache_core::wrapper::WrapperDisposition;
use crunch_rust_cache_core::wrapper::WrapperFailureMode;
use crunch_rust_cache_core::wrapper::WrapperInvocationManifest;
use crunch_rust_cache_core::wrapper::WrapperInvocationManifestInput;
use crunch_rust_cache_core::wrapper::WrapperOutputContract;
use crunch_rust_cache_core::wrapper::WrapperReceipt;
use crunch_rust_cache_core::wrapper::WrapperReceiptInput;
use crunch_rust_cache_core::wrapper::WrapperRequest;
use crunch_rust_cache_core::wrapper::WrapperRequestInput;
use crunch_rust_cache_core::wrapper::WrapperResponse;
use crunch_rust_cache_core::wrapper::arguments_identity_blake3;
use crunch_rust_cache_core::wrapper::classify_wrapper_request;
use crunch_rust_cache_core::wrapper::environment_identity_blake3;
use crunch_rust_cache_core::wrapper::seal_wrapper_manifest;
use crunch_rust_cache_core::wrapper::seal_wrapper_receipt;
use crunch_rust_cache_core::wrapper::seal_wrapper_request;
use crunch_rust_cache_core::wrapper::validate_wrapper_manifest;
use crunch_rust_cache_core::wrapper::validate_wrapper_policy;
use crunch_rust_cache_core::wrapper::validate_wrapper_request;
use crunch_rust_cache_core::wrapper::validate_wrapper_response;
use crunch_store::StoreConfig;
use ed25519_dalek::SigningKey;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tempfile::Builder;

const FRAME_PREFIX_BYTES: usize = 8;
const READ_BUFFER_BYTES: usize = 65_536;
const EXTRA_EOF_READ_ITERATIONS: u64 = 2;
const MAX_STAGED_TREE_ENTRIES: usize = 1_024;
const EXIT_PROTOCOL_FAILURE: i32 = 70;
const EXIT_TIMEOUT: i32 = 124;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const SOCKET_MODE: u32 = 0o600;
const ACCEPT_POLL_MILLIS: u64 = 20;
const CHILD_POLL_MILLIS: u64 = 10;
const MAX_POLICY_BYTES: u64 = 1_048_576;
const MAX_MANIFEST_BYTES: u64 = crunch_rust_cache_core::wrapper::MAX_WRAPPER_REQUEST_BYTES;
const SIGNING_KEY_BYTES: usize = 32;
const SIGNING_KEY_HEX_BYTES: usize = 64;
const STORE_PREFIX: &str = "/mantle/store";
const WRAPPER_POLICY_ENV: &str = "MANTLE_RUST_CACHE_POLICY";
const WRAPPER_MANIFEST_ENV: &str = "MANTLE_RUSTC_MANIFEST";
const WRAPPER_MANIFEST_DIR_ENV: &str = "MANTLE_RUSTC_MANIFEST_DIR";
const WRAPPER_MANIFEST_REF_ENV: &str = "MANTLE_RUSTC_MANIFEST_REF";
const BWRAP_BASENAME: &str = "bwrap";
const RECEIPT_SUFFIX: &str = ".json";
const TEMP_SUFFIX: &str = ".tmp";
const DIRECTORY_IDENTITY_DOMAIN: &[u8] = b"mantle.rustc-wrapper.directory.v1";
const DIRECTORY_IDENTITY_SEPARATOR: u8 = 0;
const CASITA_ENVELOPE_INVALID: &str = "casita-envelope-invalid";
const CASITA_ROOT_MISSING: &str = "casita-root-missing";

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum Error {
    Bound(String),
    Core(String),
    Io { context: String, source: std::io::Error },
    Json { context: String, source: serde_json::Error },
    Policy(String),
    Process(String),
    Protocol(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bound(code) | Self::Core(code) | Self::Policy(code) | Self::Process(code) | Self::Protocol(code) => {
                formatter.write_str(code)
            }
            Self::Io { context, source } => write!(formatter, "{context}:{source}"),
            Self::Json { context, source } => write!(formatter, "{context}:{source}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct PublishedManifest {
    pub manifest: WrapperInvocationManifest,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DaemonOptions {
    pub policy_path: PathBuf,
    pub state_dir: PathBuf,
    pub backend: crunch_store::StoreBackend,
    pub store_output_dir: PathBuf,
    pub receipt_dir: PathBuf,
    pub run_once: bool,
}

struct DaemonContext {
    policy: WrapperDaemonPolicy,
    cache: RustCache,
    local_policy: LocalCachePolicy,
    shared_policy: SharedRustCachePolicy,
    sources: Vec<Arc<dyn RustResultSource>>,
    receipt_dir: PathBuf,
    response_flushed: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Debug)]
struct CompilerOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    did_timeout: bool,
}

#[derive(Debug)]
struct AdmittedManifest {
    manifest: WrapperInvocationManifest,
    stage_root: PathBuf,
}

#[derive(Debug)]
struct VerifiedArtifacts {
    digests: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirectoryEntryKind {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DirectoryEntryFact {
    relative_path: Vec<u8>,
    kind: DirectoryEntryKind,
    mode: u32,
    size_bytes: u64,
    content_identity: Vec<u8>,
}

#[derive(Debug, Clone)]
struct PendingDirectory {
    path: PathBuf,
    depth: u32,
}

#[derive(Debug)]
struct ResponseFacts {
    disposition: WrapperDisposition,
    bypass_class: Option<WrapperBypassClass>,
    reason_codes: Vec<String>,
    compiler_status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    artifact_commit_complete: bool,
    receipt_ref: Option<String>,
}

pub fn publish_manifest_to_registry(input_path: &Path, registry_dir: &Path) -> Result<PublishedManifest, Error> {
    assert!(!input_path.as_os_str().is_empty());
    assert!(!registry_dir.as_os_str().is_empty());
    let input = load_json_file::<WrapperInvocationManifestInput>(input_path, MAX_MANIFEST_BYTES, "manifest-input")?;
    let manifest = seal_wrapper_manifest(input).map_err(|error| Error::Core(String::from(error.code())))?;
    create_private_directory(registry_dir)?;
    let destination = registry_dir.join(format!("{}{RECEIPT_SUFFIX}", manifest.input.arguments_blake3));
    if destination.exists() {
        let existing =
            load_json_file::<WrapperInvocationManifest>(&destination, MAX_MANIFEST_BYTES, "existing-wrapper-manifest")?;
        if existing != manifest {
            return Err(Error::Protocol("rustc-wrapper-manifest-registry-collision".to_string()));
        }
        return Ok(PublishedManifest {
            manifest,
            path: destination,
        });
    }
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|source| Error::Json {
        context: "encode-wrapper-manifest".to_string(),
        source,
    })?;
    publish_immutable_bytes(registry_dir, &destination, &bytes, "wrapper-manifest")?;
    assert!(destination.is_file());
    Ok(PublishedManifest {
        manifest,
        path: destination,
    })
}

pub fn declared_input_digest_blake3(path: &Path, kind: DeclaredInputKind, limit_bytes: u64) -> Result<String, Error> {
    assert!(!path.as_os_str().is_empty());
    assert!(limit_bytes > 0);
    match kind {
        DeclaredInputKind::File => hash_file_bounded(path, limit_bytes),
        DeclaredInputKind::Directory => hash_directory_bounded(path, limit_bytes),
    }
}

pub fn run_wrapper_from_environment() -> Result<i32, Error> {
    let process_arguments = std::env::args_os().collect::<Vec<_>>();
    let (compiler, arguments) = split_wrapper_arguments(&process_arguments)?;
    let Some(policy_path) = std::env::var_os(WRAPPER_POLICY_ENV).map(PathBuf::from) else {
        return run_direct_compiler(&compiler, &arguments);
    };
    let policy = load_json_file::<WrapperDaemonPolicy>(&policy_path, MAX_POLICY_BYTES, "wrapper-policy")?;
    validate_wrapper_policy(&policy).map_err(|error| Error::Policy(String::from(error.code())))?;
    let arguments = os_strings_to_strings(&arguments)?;
    if crunch_rust_cache_core::wrapper::classify_argument_bypass(&arguments).is_some() {
        return run_direct_compiler_strings(&compiler, &arguments);
    }
    let request = build_client_request(&policy, &compiler, arguments)?;
    let response = request_daemon(&policy, &request);
    match response {
        Ok(response) => finish_client_response(&policy, &compiler, &request, response),
        Err(error) => finish_client_failure(&policy, &compiler, &request.input.arguments, error),
    }
}

pub fn run_daemon(options: DaemonOptions) -> Result<(), Error> {
    run_daemon_observed(options, None, None)
}

/// Observe listener admission and only a validated Rust response successfully
/// flushed to its client. Observers never affect the cache response frame.
pub fn run_daemon_with_response_observer(
    options: DaemonOptions,
    listener_started: Arc<dyn Fn() + Send + Sync>,
    response_flushed: Arc<dyn Fn() + Send + Sync>,
) -> Result<(), Error> {
    run_daemon_observed(options, Some(listener_started), Some(response_flushed))
}

fn run_daemon_observed(
    options: DaemonOptions,
    listener_started: Option<Arc<dyn Fn() + Send + Sync>>,
    response_flushed: Option<Arc<dyn Fn() + Send + Sync>>,
) -> Result<(), Error> {
    assert!(!options.policy_path.as_os_str().is_empty());
    assert!(!options.state_dir.as_os_str().is_empty());
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
    install_shutdown_handlers()?;
    let mut context = open_daemon_context(&options)?;
    context.response_flushed = response_flushed;
    let context = Arc::new(context);
    let socket_path = context.policy.socket_path.clone();
    prepare_socket(&socket_path)?;
    let listener = UnixListener::bind(&socket_path).map_err(|source| Error::Io {
        context: "bind-daemon-socket".to_string(),
        source,
    })?;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(SOCKET_MODE)).map_err(|source| Error::Io {
        context: "chmod-daemon-socket".to_string(),
        source,
    })?;
    listener.set_nonblocking(true).map_err(|source| Error::Io {
        context: "configure-daemon-socket".to_string(),
        source,
    })?;
    if let Some(listener_started) = listener_started {
        listener_started();
    }
    let result = serve_listener(&listener, context, options.run_once);
    drop(listener);
    remove_socket(&socket_path);
    result
}

fn open_daemon_context(options: &DaemonOptions) -> Result<DaemonContext, Error> {
    let policy = load_json_file::<WrapperDaemonPolicy>(&options.policy_path, MAX_POLICY_BYTES, "daemon-policy")?;
    validate_wrapper_policy(&policy).map_err(|error| Error::Policy(String::from(error.code())))?;
    assert!(!policy.policy_id.is_empty());
    assert!(policy.max_concurrency > 0);
    let store_config = StoreConfig::new(
        options.backend,
        options.state_dir.clone(),
        options.store_output_dir.clone(),
        STORE_PREFIX.to_string(),
    );
    store_config
        .preflight_backend_identity()
        .map_err(|error| Error::Process(format!("open-rust-cache:{error}")))?;
    create_private_directory(&options.state_dir)?;
    create_private_directory(&options.store_output_dir)?;
    create_private_directory(&options.receipt_dir)?;
    let cache = RustCache::open(store_config).map_err(|error| Error::Process(format!("open-rust-cache:{error}")))?;
    let local_policy = LocalCachePolicy {
        schema: crunch_rust_cache_core::LOCAL_CACHE_POLICY_SCHEMA.to_string(),
        policy_id: format!("{}-local", policy.policy_id),
        reads_enabled: policy.local_reads_enabled,
        writes_enabled: policy.local_writes_enabled,
        execute_after_rejection: policy.failure_mode == WrapperFailureMode::FailOpen,
        max_candidates: u32::try_from(crunch_rust_cache_core::MAX_RESULT_CANDIDATES)
            .map_err(|_| Error::Bound("rustc-wrapper-candidate-count-unrepresentable".to_string()))?,
        max_tree_entries: crunch_rust_cache_core::MAX_TREE_ENTRIES,
        max_tree_depth: crunch_rust_cache_core::MAX_TREE_DEPTH,
        max_tree_bytes: policy.limits.artifact_bytes,
    };
    let shared_policy = SharedRustCachePolicy {
        schema: crunch_rust_cache::shared::SHARED_CACHE_POLICY_SCHEMA.to_string(),
        policy_id: format!("{}-shared", policy.policy_id),
        reads_enabled: policy.shared_reads_enabled,
        publishes_enabled: policy.shared_writes_enabled,
        offline: false,
        execute_after_rejection: policy.failure_mode == WrapperFailureMode::FailOpen,
        block_on_conflict: true,
        max_sources: u32::try_from(policy.result_sources.len())
            .map_err(|_| Error::Bound("rustc-wrapper-source-count-unrepresentable".to_string()))?,
        max_candidates: u32::try_from(crunch_rust_cache::shared::MAX_SHARED_CANDIDATES)
            .map_err(|_| Error::Bound("rustc-wrapper-shared-candidate-count-unrepresentable".to_string()))?,
        max_metadata_bytes: crunch_rust_cache::shared::MAX_SHARED_METADATA_BYTES,
        max_transfer_bytes: policy.limits.artifact_bytes,
        max_redirects: 0,
        max_retries: 0,
    };
    let sources = open_result_sources(&policy)?;
    assert_eq!(sources.len(), policy.result_sources.len());
    Ok(DaemonContext {
        policy,
        cache,
        local_policy,
        shared_policy,
        sources,
        receipt_dir: options.receipt_dir.clone(),
        response_flushed: None,
    })
}

fn serve_listener(listener: &UnixListener, context: Arc<DaemonContext>, run_once: bool) -> Result<(), Error> {
    let worker_count = usize::try_from(context.policy.max_concurrency)
        .map_err(|_| Error::Bound("rustc-wrapper-concurrency-unrepresentable".to_string()))?;
    let (sender, receiver) = mpsc::sync_channel::<UnixStream>(worker_count);
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let worker_context = Arc::clone(&context);
        let worker_receiver = Arc::clone(&receiver);
        workers.push(thread::spawn(move || worker_loop(worker_receiver, worker_context)));
    }
    let mut accepted_count = 0_u64;
    while !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _address)) => {
                sender
                    .send(stream)
                    .map_err(|_| Error::Protocol("rustc-wrapper-worker-channel-closed".to_string()))?;
                accepted_count = accepted_count
                    .checked_add(1)
                    .ok_or_else(|| Error::Bound("rustc-wrapper-accepted-count-overflow".to_string()))?;
                if run_once {
                    break;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(ACCEPT_POLL_MILLIS));
            }
            Err(source) => {
                return Err(Error::Io {
                    context: "accept-daemon-client".to_string(),
                    source,
                });
            }
        }
    }
    drop(sender);
    for worker in workers {
        worker.join().map_err(|_| Error::Process("rustc-wrapper-worker-panicked".to_string()))??;
    }
    if run_once {
        assert!(accepted_count <= 1);
    }
    assert!(worker_count > 0);
    Ok(())
}

fn worker_loop(receiver: Arc<Mutex<mpsc::Receiver<UnixStream>>>, context: Arc<DaemonContext>) -> Result<(), Error> {
    while let Some(mut stream) = receive_stream(&receiver)? {
        if let Err(error) = handle_connection(&mut stream, &context) {
            if let Err(shutdown_error) = stream.shutdown(Shutdown::Both) {
                eprintln!("mantle-rust-cache-daemon:shutdown-client:{shutdown_error}");
            }
            eprintln!("mantle-rust-cache-daemon:{error}");
        }
    }
    assert!(!context.policy.policy_id.is_empty());
    assert!(context.policy.max_concurrency > 0);
    Ok(())
}

fn receive_stream(receiver: &Mutex<mpsc::Receiver<UnixStream>>) -> Result<Option<UnixStream>, Error> {
    let guard = receiver.lock().map_err(|_| Error::Process("rustc-wrapper-worker-lock-poisoned".to_string()))?;
    let stream = guard.recv().ok();
    assert!(!std::thread::panicking());
    Ok(stream)
}

fn handle_connection(stream: &mut UnixStream, context: &DaemonContext) -> Result<(), Error> {
    assert!(context.policy.limits.elapsed_millis > 0);
    let timeout_millis = Some(Duration::from_millis(context.policy.limits.elapsed_millis));
    stream.set_read_timeout(timeout_millis).map_err(|source| Error::Io {
        context: "set-client-read-timeout".to_string(),
        source,
    })?;
    stream.set_write_timeout(timeout_millis).map_err(|source| Error::Io {
        context: "set-client-write-timeout".to_string(),
        source,
    })?;
    let peer_uid = peer_uid(stream)?;
    if context.policy.allowed_peer_uids.binary_search(&peer_uid).is_err() {
        return Err(Error::Protocol("rustc-wrapper-peer-uid-rejected".to_string()));
    }
    assert!(context.policy.allowed_peer_uids.binary_search(&peer_uid).is_ok());
    let request = read_frame::<WrapperRequest>(stream, context.policy.max_request_bytes)?;
    validate_wrapper_request(&request).map_err(|error| Error::Core(String::from(error.code())))?;
    let response = handle_request(context, &request)?;
    validate_wrapper_response(&response, context.policy.max_response_bytes)
        .map_err(|error| Error::Core(String::from(error.code())))?;
    write_frame(stream, &response, context.policy.max_response_bytes)?;
    stream.flush().map_err(|source| Error::Io {
        context: "flush-daemon-response".to_string(),
        source,
    })?;
    if let Some(response_flushed) = &context.response_flushed {
        response_flushed();
    }
    assert_eq!(response.request_ref, request.request_ref);
    Ok(())
}

fn handle_request(context: &DaemonContext, request: &WrapperRequest) -> Result<WrapperResponse, Error> {
    assert!(!request.request_ref.is_empty());
    assert!(!context.policy.policy_id.is_empty());
    let manifest = match load_request_manifest(request) {
        Ok(manifest) => manifest,
        Err(_error) => {
            let decision = crunch_rust_cache_core::wrapper::failure_decision(
                context.policy.failure_mode,
                WrapperBypassClass::ManifestInvalid,
            );
            return decision_response(context, request, None, decision);
        }
    };
    let admitted = admit_request(context, request, manifest.as_ref())?;
    let AdmittedManifest { manifest, stage_root } = match admitted {
        Ok(admitted) => admitted,
        Err(decision) => return decision_response(context, request, manifest.as_ref(), decision),
    };
    let result = execute_admitted_request(context, request, &manifest, &stage_root);
    let cleanup_result = remove_tree_if_exists(&stage_root);
    match (result, cleanup_result) {
        (Ok(response), Ok(())) => Ok(response),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(cleanup_error)) | (Err(_), Err(cleanup_error)) => Err(cleanup_error),
    }
}

fn admit_request(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: Option<&WrapperInvocationManifest>,
) -> Result<Result<AdmittedManifest, WrapperDecision>, Error> {
    assert!(!request.request_ref.is_empty());
    assert!(!context.policy.policy_id.is_empty());
    let Some(manifest) = manifest else {
        let decision = classify_wrapper_request(request, None, &context.policy, None);
        return Ok(Err(decision));
    };
    let failure_class = verify_manifest_runtime(context, request, manifest)?;
    let facts = CurrentManifestFacts {
        manifest_ref_matches: request.input.expected_manifest_ref.as_deref() == Some(manifest.manifest_ref.as_str()),
        compiler_matches: request.input.real_compiler_path == manifest.input.real_compiler_path,
        arguments_match: arguments_identity_blake3(&request.input.arguments).ok().as_ref()
            == Some(&manifest.input.arguments_blake3),
        environment_matches: environment_identity_blake3(&request.input.admitted_environment).ok().as_ref()
            == Some(&manifest.input.environment_blake3),
        working_directory_matches: request.input.working_directory == manifest.input.working_directory,
        inputs_verified: failure_class.is_none(),
        outputs_admissible: outputs_are_admissible(&context.policy, &manifest.input.output_contracts),
        enforcement_available: enforcement_is_available(&context.policy),
        failure_class,
    };
    let decision = classify_wrapper_request(request, Some(manifest), &context.policy, Some(&facts));
    if decision != WrapperDecision::StrongEligible {
        return Ok(Err(decision));
    }
    let stage_root = stage_root_for_request(request, &manifest.input.output_contracts)?;
    assert!(!path_entry_exists(&stage_root));
    Ok(Ok(AdmittedManifest {
        manifest: manifest.clone(),
        stage_root,
    }))
}

fn execute_admitted_request(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
) -> Result<WrapperResponse, Error> {
    assert!(!request.request_ref.is_empty());
    assert!(!manifest.manifest_ref.is_empty());
    if let Some(response) = try_local_cache(context, request, manifest, stage_root)? {
        return Ok(response);
    }
    if let Some(response) = try_shared_cache(context, request, manifest, stage_root)? {
        return Ok(response);
    }
    compile_and_publish(context, request, manifest, stage_root)
}

fn try_local_cache(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
) -> Result<Option<WrapperResponse>, Error> {
    if !context.local_policy.reads_enabled {
        return Ok(None);
    }
    let outcome = match context.cache.restore_blocking(&manifest.input.action, stage_root, &context.local_policy) {
        Ok(outcome) => outcome,
        Err(error) => {
            let reason = casita_payload_integrity_code(&error).unwrap_or("local-cache-restore-failed");
            let response = runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::ProtocolFailure,
                observed_compiler_status: 0,
                compiler_executed: false,
                reason,
            })?;
            return Ok(Some(response));
        }
    };
    if outcome.disposition != crunch_rust_cache::CACHE_DISPOSITION_HIT {
        return Ok(None);
    }
    let response = finish_cache_hit(context, request, manifest, stage_root, CacheHitInput {
        disposition: WrapperDisposition::LocalHit,
        result_ref: outcome.selected_result_ref,
        reason: "local-cache-hit",
    })?;
    assert!(context.local_policy.reads_enabled);
    assert_eq!(response.request_ref, request.request_ref);
    Ok(Some(response))
}

fn try_shared_cache(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
) -> Result<Option<WrapperResponse>, Error> {
    if !context.shared_policy.reads_enabled {
        return Ok(None);
    }
    let trust_policy = context
        .policy
        .shared_trust_policy
        .as_ref()
        .ok_or_else(|| Error::Policy("rustc-wrapper-shared-trust-policy-missing".to_string()))?;
    let outcome = context.cache.restore_shared_blocking(SharedRestoreRequest {
        action: &manifest.input.action,
        output_dir: stage_root,
        local_policy: &context.local_policy,
        shared_policy: &context.shared_policy,
        trust_policy,
        sources: &context.sources,
    });
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(_error) => {
            let response = runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::ProtocolFailure,
                observed_compiler_status: 0,
                compiler_executed: false,
                reason: "shared-cache-restore-failed",
            })?;
            return Ok(Some(response));
        }
    };
    if outcome.disposition != SHARED_CACHE_HIT {
        return Ok(None);
    }
    let response = finish_cache_hit(context, request, manifest, stage_root, CacheHitInput {
        disposition: WrapperDisposition::SharedHit,
        result_ref: outcome.selected_result_ref,
        reason: "shared-cache-hit",
    })?;
    assert!(context.shared_policy.reads_enabled);
    assert_eq!(response.request_ref, request.request_ref);
    Ok(Some(response))
}

struct CacheHitInput<'a> {
    disposition: WrapperDisposition,
    result_ref: Option<String>,
    reason: &'a str,
}

fn finish_cache_hit(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
    input: CacheHitInput<'_>,
) -> Result<WrapperResponse, Error> {
    assert!(matches!(input.disposition, WrapperDisposition::LocalHit | WrapperDisposition::SharedHit));
    assert!(input.result_ref.is_some());
    let verified = match verify_staged_outputs(stage_root, &manifest.input.output_contracts, &manifest.input.limits) {
        Ok(verified) => verified,
        Err(_error) => {
            return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::OutputContractUnsupported,
                observed_compiler_status: 0,
                compiler_executed: false,
                reason: "cache-output-validation-failed",
            });
        }
    };
    if commit_staged_outputs(stage_root, &manifest.input.output_contracts).is_err() {
        return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
            class: WrapperBypassClass::ProtocolFailure,
            observed_compiler_status: 0,
            compiler_executed: false,
            reason: "cache-output-publication-failed",
        });
    }
    completed_response(context, request, manifest, CompletionInput {
        disposition: input.disposition,
        result_ref: input.result_ref,
        compiler_status: 0,
        compiler_executed: false,
        stdout: Vec::new(),
        stderr: Vec::new(),
        verified,
        reason: input.reason,
    })
}

struct CompletionInput<'a> {
    disposition: WrapperDisposition,
    result_ref: Option<String>,
    compiler_status: i32,
    compiler_executed: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    verified: VerifiedArtifacts,
    reason: &'a str,
}

enum ShellStep<T> {
    Continue(T),
    Respond(WrapperResponse),
}

struct LocalPublication {
    result_ref: Option<String>,
    result: Option<crunch_rust_cache_core::RustUnitResult>,
}

struct VerifiedCompileInput {
    output: CompilerOutput,
    verified: VerifiedArtifacts,
    preliminary: WrapperReceipt,
}

struct LocalCompileInput<'a> {
    stage_root: &'a Path,
    output: &'a CompilerOutput,
    preliminary: &'a WrapperReceipt,
}

fn compile_and_publish(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
) -> Result<WrapperResponse, Error> {
    assert!(!stage_root.as_os_str().is_empty());
    assert!(!manifest.input.output_contracts.is_empty());
    let output = match start_compiler(context, request, manifest, stage_root)? {
        ShellStep::Continue(output) => output,
        ShellStep::Respond(response) => return Ok(response),
    };
    if output.status != 0 {
        return failed_compile_response(context, request, manifest, output);
    }
    finish_successful_compile(context, request, manifest, stage_root, output)
}

fn start_compiler(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
) -> Result<ShellStep<CompilerOutput>, Error> {
    create_private_directory(stage_root)?;
    let arguments = rewrite_output_arguments(&request.input.arguments, &manifest.input.output_contracts, stage_root);
    let arguments = match arguments {
        Ok(arguments) => arguments,
        Err(_error) => {
            let response = runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::OutputContractUnsupported,
                observed_compiler_status: 0,
                compiler_executed: false,
                reason: "output-arguments-unsupported",
            })?;
            return Ok(ShellStep::Respond(response));
        }
    };
    let output = match execute_compiler(context, request, &manifest.input.declared_inputs, stage_root, &arguments) {
        Ok(output) => output,
        Err(_error) => {
            let response = runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::EnforcementUnavailable,
                observed_compiler_status: 0,
                compiler_executed: false,
                reason: "compiler-execution-unavailable",
            })?;
            return Ok(ShellStep::Respond(response));
        }
    };
    assert!(!arguments.is_empty());
    assert!(stage_root.is_dir());
    Ok(ShellStep::Continue(output))
}

fn finish_successful_compile(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
    output: CompilerOutput,
) -> Result<WrapperResponse, Error> {
    assert_eq!(output.status, 0);
    assert!(stage_root.is_dir());
    match declared_inputs_unchanged(manifest) {
        Ok(true) => {}
        Ok(false) => return changed_input_after_execution_response(context, request, manifest, output),
        Err(_error) => {
            return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::InputChanged,
                observed_compiler_status: output.status,
                compiler_executed: true,
                reason: "input-revalidation-failed",
            });
        }
    }
    let verified = match verify_staged_outputs(stage_root, &manifest.input.output_contracts, &manifest.input.limits) {
        Ok(verified) => verified,
        Err(_error) => {
            return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::OutputContractUnsupported,
                observed_compiler_status: output.status,
                compiler_executed: true,
                reason: "output-validation-failed",
            });
        }
    };
    let preliminary = persist_preliminary_receipt(context, request, manifest, &output, &verified)?;
    publish_verified_compile(context, request, manifest, stage_root, VerifiedCompileInput {
        output,
        verified,
        preliminary,
    })
}

fn persist_preliminary_receipt(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    output: &CompilerOutput,
    verified: &VerifiedArtifacts,
) -> Result<WrapperReceipt, Error> {
    let receipt = seal_wrapper_receipt(WrapperReceiptInput {
        request_ref: request.request_ref.clone(),
        manifest_ref: Some(manifest.manifest_ref.clone()),
        policy_id: context.policy.policy_id.clone(),
        disposition: WrapperDisposition::Compiled,
        bypass_class: None,
        action_ref: Some(manifest.input.action.action_ref.clone()),
        result_ref: None,
        artifact_digests_blake3: verified.digests.clone(),
        compiler_status: output.status,
        compiler_executed: true,
        artifact_commit_complete: false,
        reason_codes: vec!["compiler-finished".to_string()],
    })
    .map_err(|error| Error::Core(String::from(error.code())))?;
    assert_eq!(output.status, 0);
    assert!(!verified.digests.is_empty());
    persist_receipt(&context.receipt_dir, receipt)
}

fn publish_verified_compile(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    stage_root: &Path,
    input: VerifiedCompileInput,
) -> Result<WrapperResponse, Error> {
    let publication = match publish_local_compile(context, request, manifest, LocalCompileInput {
        stage_root,
        output: &input.output,
        preliminary: &input.preliminary,
    })? {
        ShellStep::Continue(publication) => publication,
        ShellStep::Respond(response) => return Ok(response),
    };
    if context.shared_policy.publishes_enabled {
        let result = publication
            .result
            .as_ref()
            .ok_or_else(|| Error::Policy("rustc-wrapper-shared-publication-needs-local-result".to_string()))?;
        if publish_shared_result(context, result).is_err() {
            return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::ProtocolFailure,
                observed_compiler_status: input.output.status,
                compiler_executed: true,
                reason: "shared-cache-publication-failed",
            });
        }
    }
    if commit_staged_outputs(stage_root, &manifest.input.output_contracts).is_err() {
        return runtime_failure_response(context, request, manifest, RuntimeFailureInput {
            class: WrapperBypassClass::ProtocolFailure,
            observed_compiler_status: input.output.status,
            compiler_executed: true,
            reason: "output-publication-failed",
        });
    }
    assert_eq!(input.output.status, 0);
    assert!(!input.verified.digests.is_empty());
    completed_response(context, request, manifest, CompletionInput {
        disposition: WrapperDisposition::Compiled,
        result_ref: publication.result_ref,
        compiler_status: input.output.status,
        compiler_executed: true,
        stdout: input.output.stdout,
        stderr: input.output.stderr,
        verified: input.verified,
        reason: "compiled",
    })
}

fn publish_local_compile(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    input: LocalCompileInput<'_>,
) -> Result<ShellStep<LocalPublication>, Error> {
    if !context.local_policy.writes_enabled {
        return Ok(ShellStep::Continue(LocalPublication {
            result_ref: None,
            result: None,
        }));
    }
    let result = context.cache.publish_blocking(PublishRequest {
        action: &manifest.input.action,
        output_dir: input.stage_root,
        producer_receipt_ref: &input.preliminary.receipt_ref,
        policy: &context.local_policy,
    });
    let result = match result {
        Ok(result) => result,
        Err(_error) => {
            let response = runtime_failure_response(context, request, manifest, RuntimeFailureInput {
                class: WrapperBypassClass::ProtocolFailure,
                observed_compiler_status: input.output.status,
                compiler_executed: true,
                reason: "local-cache-publication-failed",
            })?;
            return Ok(ShellStep::Respond(response));
        }
    };
    assert!(!result.result_ref.is_empty());
    assert_eq!(result.input.action_ref, manifest.input.action.action_ref);
    Ok(ShellStep::Continue(LocalPublication {
        result_ref: Some(result.result_ref.clone()),
        result: Some(result),
    }))
}

fn completed_response(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    input: CompletionInput<'_>,
) -> Result<WrapperResponse, Error> {
    assert!(!input.reason.is_empty());
    assert!(!manifest.manifest_ref.is_empty());
    let receipt = seal_wrapper_receipt(WrapperReceiptInput {
        request_ref: request.request_ref.clone(),
        manifest_ref: Some(manifest.manifest_ref.clone()),
        policy_id: context.policy.policy_id.clone(),
        disposition: input.disposition,
        bypass_class: None,
        action_ref: Some(manifest.input.action.action_ref.clone()),
        result_ref: input.result_ref,
        artifact_digests_blake3: input.verified.digests,
        compiler_status: input.compiler_status,
        compiler_executed: input.compiler_executed,
        artifact_commit_complete: true,
        reason_codes: vec![input.reason.to_string()],
    })
    .map_err(|error| Error::Core(String::from(error.code())))?;
    let receipt = persist_receipt(&context.receipt_dir, receipt)?;
    response_from_facts(request, ResponseFacts {
        disposition: input.disposition,
        bypass_class: None,
        reason_codes: vec![input.reason.to_string()],
        compiler_status: input.compiler_status,
        stdout: input.stdout,
        stderr: input.stderr,
        artifact_commit_complete: true,
        receipt_ref: Some(receipt.receipt_ref),
    })
}

fn declared_inputs_unchanged(manifest: &WrapperInvocationManifest) -> Result<bool, Error> {
    for input in &manifest.input.declared_inputs {
        if !verify_declared_input(input, manifest.input.limits.artifact_bytes)? {
            return Ok(false);
        }
    }
    assert!(!manifest.input.declared_inputs.is_empty());
    Ok(true)
}

fn changed_input_after_execution_response(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    output: CompilerOutput,
) -> Result<WrapperResponse, Error> {
    assert_eq!(output.status, 0);
    assert!(!manifest.manifest_ref.is_empty());
    let decision = crunch_rust_cache_core::wrapper::failure_decision(
        context.policy.failure_mode,
        WrapperBypassClass::InputChanged,
    );
    let (disposition, status) = match decision {
        WrapperDecision::Bypass(_) => (WrapperDisposition::Bypass, 0),
        WrapperDecision::Reject(_) => (WrapperDisposition::Rejected, EXIT_PROTOCOL_FAILURE),
        WrapperDecision::StrongEligible => {
            return Err(Error::Protocol("rustc-wrapper-input-change-decision-invalid".to_string()));
        }
    };
    let receipt = persist_receipt(
        &context.receipt_dir,
        seal_wrapper_receipt(WrapperReceiptInput {
            request_ref: request.request_ref.clone(),
            manifest_ref: Some(manifest.manifest_ref.clone()),
            policy_id: context.policy.policy_id.clone(),
            disposition,
            bypass_class: Some(WrapperBypassClass::InputChanged),
            action_ref: Some(manifest.input.action.action_ref.clone()),
            result_ref: None,
            artifact_digests_blake3: Vec::new(),
            compiler_status: output.status,
            compiler_executed: true,
            artifact_commit_complete: false,
            reason_codes: vec!["input-changed-after-execution".to_string()],
        })
        .map_err(|error| Error::Core(String::from(error.code())))?,
    )?;
    response_from_facts(request, ResponseFacts {
        disposition,
        bypass_class: Some(WrapperBypassClass::InputChanged),
        reason_codes: vec!["input-changed-after-execution".to_string()],
        compiler_status: status,
        stdout: Vec::new(),
        stderr: Vec::new(),
        artifact_commit_complete: false,
        receipt_ref: Some(receipt.receipt_ref),
    })
}

struct RuntimeFailureInput<'a> {
    class: WrapperBypassClass,
    observed_compiler_status: i32,
    compiler_executed: bool,
    reason: &'a str,
}

fn casita_payload_integrity_code(error: &crunch_rust_cache::Error) -> Option<&'static str> {
    match error {
        crunch_rust_cache::Error::Castore(message) if message.contains(CASITA_ENVELOPE_INVALID) => {
            Some(CASITA_ENVELOPE_INVALID)
        }
        crunch_rust_cache::Error::Castore(message) if message.contains(CASITA_ROOT_MISSING) => {
            Some(CASITA_ROOT_MISSING)
        }
        crunch_rust_cache::Error::Authority(crunch_store::Error::Store(message))
            if message.starts_with(CASITA_ENVELOPE_INVALID) =>
        {
            Some(CASITA_ENVELOPE_INVALID)
        }
        crunch_rust_cache::Error::Authority(crunch_store::Error::Store(message))
            if message.starts_with(CASITA_ROOT_MISSING) =>
        {
            Some(CASITA_ROOT_MISSING)
        }
        _ => None,
    }
}

fn is_casita_integrity_reason(reason: &str) -> bool {
    matches!(reason, CASITA_ENVELOPE_INVALID | CASITA_ROOT_MISSING)
}

fn runtime_failure_response(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    input: RuntimeFailureInput<'_>,
) -> Result<WrapperResponse, Error> {
    assert!(!input.reason.is_empty());
    assert!(!manifest.manifest_ref.is_empty());
    let decision = if is_casita_integrity_reason(input.reason) {
        WrapperDecision::Reject(input.class)
    } else {
        crunch_rust_cache_core::wrapper::failure_decision(context.policy.failure_mode, input.class)
    };
    let (disposition, response_status) = match decision {
        WrapperDecision::Bypass(_) => (WrapperDisposition::Bypass, 0),
        WrapperDecision::Reject(_) => (WrapperDisposition::Rejected, EXIT_PROTOCOL_FAILURE),
        WrapperDecision::StrongEligible => {
            return Err(Error::Protocol("rustc-wrapper-runtime-failure-decision-invalid".to_string()));
        }
    };
    let receipt = persist_receipt(
        &context.receipt_dir,
        seal_wrapper_receipt(WrapperReceiptInput {
            request_ref: request.request_ref.clone(),
            manifest_ref: Some(manifest.manifest_ref.clone()),
            policy_id: context.policy.policy_id.clone(),
            disposition,
            bypass_class: Some(input.class),
            action_ref: Some(manifest.input.action.action_ref.clone()),
            result_ref: None,
            artifact_digests_blake3: Vec::new(),
            compiler_status: input.observed_compiler_status,
            compiler_executed: input.compiler_executed,
            artifact_commit_complete: false,
            reason_codes: vec![input.reason.to_string()],
        })
        .map_err(|error| Error::Core(String::from(error.code())))?,
    )?;
    response_from_facts(request, ResponseFacts {
        disposition,
        bypass_class: Some(input.class),
        reason_codes: vec![input.reason.to_string()],
        compiler_status: response_status,
        stdout: Vec::new(),
        stderr: if is_casita_integrity_reason(input.reason) {
            format!("mantle-rustc-wrapper:rejected:{}\n", input.reason).into_bytes()
        } else {
            Vec::new()
        },
        artifact_commit_complete: false,
        receipt_ref: Some(receipt.receipt_ref),
    })
}

fn failed_compile_response(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
    output: CompilerOutput,
) -> Result<WrapperResponse, Error> {
    assert_ne!(output.status, 0);
    assert!(!manifest.manifest_ref.is_empty());
    let reason = if output.did_timeout {
        "compiler-timeout"
    } else {
        "compiler-failed"
    };
    let receipt = persist_receipt(
        &context.receipt_dir,
        seal_wrapper_receipt(WrapperReceiptInput {
            request_ref: request.request_ref.clone(),
            manifest_ref: Some(manifest.manifest_ref.clone()),
            policy_id: context.policy.policy_id.clone(),
            disposition: WrapperDisposition::Compiled,
            bypass_class: None,
            action_ref: Some(manifest.input.action.action_ref.clone()),
            result_ref: None,
            artifact_digests_blake3: Vec::new(),
            compiler_status: output.status,
            compiler_executed: true,
            artifact_commit_complete: false,
            reason_codes: vec![reason.to_string()],
        })
        .map_err(|error| Error::Core(String::from(error.code())))?,
    )?;
    response_from_facts(request, ResponseFacts {
        disposition: WrapperDisposition::Compiled,
        bypass_class: None,
        reason_codes: vec![reason.to_string()],
        compiler_status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
        artifact_commit_complete: false,
        receipt_ref: Some(receipt.receipt_ref),
    })
}

fn decision_response(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: Option<&WrapperInvocationManifest>,
    decision: WrapperDecision,
) -> Result<WrapperResponse, Error> {
    assert_ne!(decision, WrapperDecision::StrongEligible);
    assert!(!context.policy.policy_id.is_empty());
    let (disposition, class, status, reason) = match decision {
        WrapperDecision::StrongEligible => {
            return Err(Error::Protocol("rustc-wrapper-unexecuted-eligible-request".to_string()));
        }
        WrapperDecision::Bypass(class) => (WrapperDisposition::Bypass, class, 0, "bypass"),
        WrapperDecision::Reject(class) => (WrapperDisposition::Rejected, class, EXIT_PROTOCOL_FAILURE, "rejected"),
    };
    let receipt = persist_receipt(
        &context.receipt_dir,
        seal_wrapper_receipt(WrapperReceiptInput {
            request_ref: request.request_ref.clone(),
            manifest_ref: manifest.map(|value| value.manifest_ref.clone()),
            policy_id: context.policy.policy_id.clone(),
            disposition,
            bypass_class: Some(class),
            action_ref: manifest.map(|value| value.input.action.action_ref.clone()),
            result_ref: None,
            artifact_digests_blake3: Vec::new(),
            compiler_status: status,
            compiler_executed: false,
            artifact_commit_complete: false,
            reason_codes: vec![format!("{reason}-{class:?}").to_ascii_lowercase()],
        })
        .map_err(|error| Error::Core(String::from(error.code())))?,
    )?;
    response_from_facts(request, ResponseFacts {
        disposition,
        bypass_class: Some(class),
        reason_codes: vec![format!("{reason}-{class:?}").to_ascii_lowercase()],
        compiler_status: status,
        stdout: Vec::new(),
        stderr: Vec::new(),
        artifact_commit_complete: false,
        receipt_ref: Some(receipt.receipt_ref),
    })
}

fn response_from_facts(request: &WrapperRequest, facts: ResponseFacts) -> Result<WrapperResponse, Error> {
    let response = WrapperResponse {
        schema: WRAPPER_RESPONSE_SCHEMA.to_string(),
        request_ref: request.request_ref.clone(),
        disposition: facts.disposition,
        bypass_class: facts.bypass_class,
        reason_codes: facts.reason_codes,
        compiler_status: facts.compiler_status,
        stdout: facts.stdout,
        stderr: facts.stderr,
        artifact_commit_complete: facts.artifact_commit_complete,
        receipt_ref: facts.receipt_ref,
    };
    validate_wrapper_response(&response, crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESPONSE_BYTES)
        .map_err(|error| Error::Core(String::from(error.code())))?;
    assert_eq!(response.request_ref, request.request_ref);
    Ok(response)
}

fn execute_compiler(
    context: &DaemonContext,
    request: &WrapperRequest,
    declared_inputs: &[DeclaredInput],
    stage_root: &Path,
    rewritten_arguments: &[String],
) -> Result<CompilerOutput, Error> {
    if !enforcement_is_available(&context.policy) {
        return Err(Error::Policy("rustc-wrapper-sandbox-changed-before-execution".to_string()));
    }
    let mut command = sandbox_command(&context.policy, request, declared_inputs, stage_root, rewritten_arguments)?;
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| Error::Io {
        context: "spawn-sandboxed-compiler".to_string(),
        source,
    })?;
    let stdout = child.stdout.take().ok_or_else(|| Error::Process("rustc-wrapper-stdout-pipe-missing".to_string()))?;
    let stderr = child.stderr.take().ok_or_else(|| Error::Process("rustc-wrapper-stderr-pipe-missing".to_string()))?;
    let stdout_limit_bytes = context.policy.limits.stdout_bytes;
    let stderr_limit_bytes = context.policy.limits.stderr_bytes;
    let stdout_reader = thread::spawn(move || read_stream_bounded(stdout, stdout_limit_bytes, "compiler-stdout"));
    let stderr_reader = thread::spawn(move || read_stream_bounded(stderr, stderr_limit_bytes, "compiler-stderr"));
    let (status, is_timeout) = wait_for_compiler(&mut child, context.policy.limits.elapsed_millis)?;
    let stdout = stdout_reader
        .join()
        .map_err(|_| Error::Process("rustc-wrapper-stdout-reader-panicked".to_string()))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| Error::Process("rustc-wrapper-stderr-reader-panicked".to_string()))??;
    let status = if is_timeout { EXIT_TIMEOUT } else { exit_code(status) };
    if is_timeout {
        assert_eq!(status, EXIT_TIMEOUT);
    }
    assert!(context.policy.limits.elapsed_millis > 0);
    Ok(CompilerOutput {
        status,
        stdout,
        stderr,
        did_timeout: is_timeout,
    })
}

fn wait_for_compiler(child: &mut std::process::Child, timeout_millis: u64) -> Result<(ExitStatus, bool), Error> {
    let poll_count_max = timeout_millis
        .checked_div(CHILD_POLL_MILLIS)
        .and_then(|count| count.checked_add(2))
        .ok_or_else(|| Error::Bound("rustc-wrapper-compiler-poll-count-overflow".to_string()))?;
    assert!(timeout_millis > 0);
    assert!(poll_count_max > 0);
    for _ in 0..poll_count_max {
        if let Some(status) = child.try_wait().map_err(|source| Error::Io {
            context: "poll-sandboxed-compiler".to_string(),
            source,
        })? {
            return Ok((status, false));
        }
        thread::sleep(Duration::from_millis(CHILD_POLL_MILLIS));
    }
    child.kill().map_err(|source| Error::Io {
        context: "kill-sandboxed-compiler".to_string(),
        source,
    })?;
    let status = child.wait().map_err(|source| Error::Io {
        context: "wait-killed-compiler".to_string(),
        source,
    })?;
    assert!(poll_count_max > 1);
    Ok((status, true))
}

fn sandbox_command(
    policy: &WrapperDaemonPolicy,
    request: &WrapperRequest,
    declared_inputs: &[DeclaredInput],
    stage_root: &Path,
    rewritten_arguments: &[String],
) -> Result<Command, Error> {
    assert!(!policy.sandbox_program.is_empty());
    assert!(!declared_inputs.is_empty());
    assert!(stage_root.is_dir());
    assert!(!rewritten_arguments.is_empty());
    let sandbox_path = Path::new(&policy.sandbox_program);
    let is_bwrap = sandbox_path.file_name().and_then(|name| name.to_str()) == Some(BWRAP_BASENAME);
    let mut command = Command::new(&policy.sandbox_program);
    command.env_clear();
    for (name, value) in &request.input.admitted_environment {
        command.env(name, value);
    }
    if is_bwrap {
        command.args([
            "--die-with-parent",
            "--unshare-net",
            "--new-session",
            "--tmpfs",
            "/",
            "--dev",
            "/dev",
        ]);
        command.args(["--dir", &request.input.working_directory]);
        let mut bound_paths = BTreeSet::new();
        for input in declared_inputs {
            if bound_paths.insert(input.path.as_str()) {
                command.args(["--ro-bind", &input.path, &input.path]);
            }
        }
        let stage_root =
            stage_root.to_str().ok_or_else(|| Error::Policy("rustc-wrapper-stage-path-not-utf8".to_string()))?;
        command.args(["--bind", stage_root, stage_root]);
        command.args([
            "--chdir",
            &request.input.working_directory,
            "--",
            &request.input.real_compiler_path,
        ]);
    } else {
        #[cfg(not(test))]
        return Err(Error::Policy("rustc-wrapper-sandbox-provider-unsupported".to_string()));
        #[cfg(test)]
        {
            command.arg("--").arg(&request.input.real_compiler_path);
            command.current_dir(&request.input.working_directory);
        }
    }
    command.args(rewritten_arguments);
    assert!(!rewritten_arguments.is_empty());
    Ok(command)
}

fn rewrite_output_arguments(
    arguments: &[String],
    contracts: &[WrapperOutputContract],
    stage_root: &Path,
) -> Result<Vec<String>, Error> {
    assert!(!arguments.is_empty());
    assert!(!contracts.is_empty());
    let stage_text =
        stage_root.to_str().ok_or_else(|| Error::Policy("rustc-wrapper-stage-path-not-utf8".to_string()))?;
    let common_parent = common_destination_parent(contracts)?;
    let common_parent_text = common_parent
        .to_str()
        .ok_or_else(|| Error::Policy("rustc-wrapper-output-parent-not-utf8".to_string()))?;
    let mut rewritten = Vec::with_capacity(arguments.len());
    for argument in arguments {
        let out_dir_assignment = format!("--out-dir={common_parent_text}");
        let mut value = if argument == common_parent_text {
            stage_text.to_string()
        } else if argument == &out_dir_assignment {
            format!("--out-dir={stage_text}")
        } else {
            argument.clone()
        };
        for contract in contracts {
            let staged = stage_root.join(&contract.staged_relative_path);
            let staged =
                staged.to_str().ok_or_else(|| Error::Policy("rustc-wrapper-staged-output-not-utf8".to_string()))?;
            value = value.replace(&contract.destination_path, staged);
        }
        rewritten.push(value);
    }
    if rewritten == arguments {
        return Err(Error::Policy("rustc-wrapper-output-arguments-not-rewritten".to_string()));
    }
    assert_eq!(rewritten.len(), arguments.len());
    Ok(rewritten)
}

fn verify_manifest_runtime(
    context: &DaemonContext,
    request: &WrapperRequest,
    manifest: &WrapperInvocationManifest,
) -> Result<Option<WrapperBypassClass>, Error> {
    assert!(!request.request_ref.is_empty());
    assert!(!context.policy.policy_id.is_empty());
    validate_wrapper_manifest(manifest).map_err(|error| Error::Core(String::from(error.code())))?;
    if manifest.input.policy_id != context.policy.policy_id {
        return Ok(Some(WrapperBypassClass::PolicyRejected));
    }
    if manifest.input.readable_roots != context.policy.readable_roots {
        return Ok(Some(WrapperBypassClass::EffectPolicyUnsupported));
    }
    if manifest.input.writable_roots != context.policy.writable_roots {
        return Ok(Some(WrapperBypassClass::EffectPolicyUnsupported));
    }
    if manifest.input.effects != context.policy.effects || manifest.input.limits != context.policy.limits {
        return Ok(Some(WrapperBypassClass::EffectPolicyUnsupported));
    }
    if !path_is_within_roots(&manifest.input.working_directory, &context.policy.readable_roots) {
        return Ok(Some(WrapperBypassClass::InputOutOfRoot));
    }
    let is_manifest_path_admitted = request
        .input
        .manifest_path
        .as_ref()
        .is_some_and(|path| path_is_within_roots(path, &context.policy.readable_roots));
    if !is_manifest_path_admitted {
        return Ok(Some(WrapperBypassClass::InputOutOfRoot));
    }
    let compiler_input = manifest.input.declared_inputs.iter().find(|input| input.role == "compiler");
    let Some(compiler_input) = compiler_input else {
        return Ok(Some(WrapperBypassClass::InputMissing));
    };
    if compiler_input.path != manifest.input.real_compiler_path {
        return Ok(Some(WrapperBypassClass::ManifestIdentityMismatch));
    }
    if compiler_input.digest_blake3 != manifest.input.action.input.compiler_digest_blake3 {
        return Ok(Some(WrapperBypassClass::ManifestIdentityMismatch));
    }
    for input in &manifest.input.declared_inputs {
        if !verify_declared_input(input, manifest.input.limits.artifact_bytes)? {
            return Ok(Some(WrapperBypassClass::InputChanged));
        }
    }
    if request.input.output_contracts != manifest.input.output_contracts {
        return Ok(Some(WrapperBypassClass::OutputContractUnsupported));
    }
    assert_eq!(manifest.input.policy_id, context.policy.policy_id);
    Ok(None)
}

fn verify_declared_input(input: &DeclaredInput, limit: u64) -> Result<bool, Error> {
    assert!(!input.path.is_empty());
    assert!(limit > 0);
    let path = Path::new(&input.path);
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(Error::Io {
                context: "stat-declared-input".to_string(),
                source,
            });
        }
    };
    let has_expected_kind = match input.kind {
        DeclaredInputKind::File => metadata.file_type().is_file(),
        DeclaredInputKind::Directory => metadata.file_type().is_dir(),
    };
    if !has_expected_kind {
        return Ok(false);
    }
    let digest = declared_input_digest_blake3(path, input.kind, limit)?;
    assert!(!digest.is_empty());
    Ok(digest == input.digest_blake3)
}

fn outputs_are_admissible(policy: &WrapperDaemonPolicy, contracts: &[WrapperOutputContract]) -> bool {
    if contracts.len() != 1 {
        return false;
    }
    for contract in contracts {
        if !path_is_within_roots(&contract.destination_path, &policy.writable_roots) {
            return false;
        }
        if !output_parent_is_admitted(&contract.destination_path, &policy.writable_roots) {
            return false;
        }
        if path_entry_exists(Path::new(&contract.destination_path)) {
            return false;
        }
    }
    common_destination_parent(contracts).is_ok()
}

fn enforcement_is_available(policy: &WrapperDaemonPolicy) -> bool {
    let is_bwrap =
        Path::new(&policy.sandbox_program).file_name().and_then(|name| name.to_str()) == Some(BWRAP_BASENAME);
    if !is_bwrap && !cfg!(test) {
        return false;
    }
    let metadata = fs::symlink_metadata(&policy.sandbox_program);
    let Ok(metadata) = metadata else {
        return false;
    };
    if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return false;
    }
    let digest = hash_file_bounded(
        Path::new(&policy.sandbox_program),
        crunch_rust_cache_core::wrapper::MAX_WRAPPER_ARTIFACT_BYTES,
    );
    digest.is_ok_and(|value| value == policy.sandbox_program_blake3)
}

fn verify_staged_outputs(
    stage_root: &Path,
    contracts: &[WrapperOutputContract],
    limits: &crunch_rust_cache_core::wrapper::WrapperResourceLimits,
) -> Result<VerifiedArtifacts, Error> {
    validate_staged_output_set(stage_root, contracts)?;
    let mut total_bytes = 0_u64;
    let mut digests = Vec::with_capacity(contracts.len());
    for contract in contracts {
        let path = stage_root.join(&contract.staged_relative_path);
        let metadata = fs::symlink_metadata(&path).map_err(|source| Error::Io {
            context: "stat-staged-output".to_string(),
            source,
        })?;
        if contract.kind != WrapperArtifactKind::File || !metadata.file_type().is_file() {
            return Err(Error::Policy("rustc-wrapper-staged-output-kind-invalid".to_string()));
        }
        if metadata.len() > contract.max_bytes {
            return Err(Error::Bound("rustc-wrapper-staged-output-too-large".to_string()));
        }
        let is_executable = metadata.permissions().mode() & 0o111 != 0;
        if is_executable != contract.executable {
            return Err(Error::Policy("rustc-wrapper-staged-output-mode-mismatch".to_string()));
        }
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| Error::Bound("rustc-wrapper-artifact-bytes-overflow".to_string()))?;
        if total_bytes > limits.artifact_bytes {
            return Err(Error::Bound("rustc-wrapper-artifact-limit-exceeded".to_string()));
        }
        digests.push(hash_file_bounded(&path, contract.max_bytes)?);
    }
    assert_eq!(digests.len(), contracts.len());
    assert!(total_bytes <= limits.artifact_bytes);
    Ok(VerifiedArtifacts { digests })
}

fn validate_staged_output_set(stage_root: &Path, contracts: &[WrapperOutputContract]) -> Result<(), Error> {
    assert!(!contracts.is_empty());
    assert!(!stage_root.as_os_str().is_empty());
    let expected = contracts
        .iter()
        .map(|contract| PathBuf::from(&contract.staged_relative_path))
        .collect::<BTreeSet<_>>();
    let mut files = BTreeSet::new();
    let mut pending = vec![stage_root.to_path_buf()];
    for _ in 0..MAX_STAGED_TREE_ENTRIES {
        let Some(directory) = pending.pop() else {
            break;
        };
        let read_dir = fs::read_dir(&directory).map_err(|source| Error::Io {
            context: "read-staged-output-directory".to_string(),
            source,
        })?;
        let mut entries = Vec::with_capacity(MAX_STAGED_TREE_ENTRIES);
        for entry in read_dir.take(MAX_STAGED_TREE_ENTRIES.checked_add(1).unwrap_or(MAX_STAGED_TREE_ENTRIES)) {
            entries.push(entry.map_err(|source| Error::Io {
                context: "read-staged-output-entry".to_string(),
                source,
            })?);
        }
        if entries.len() > MAX_STAGED_TREE_ENTRIES {
            return Err(Error::Bound("rustc-wrapper-staged-tree-entry-limit-exceeded".to_string()));
        }
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|source| Error::Io {
                context: "stat-staged-output-entry".to_string(),
                source,
            })?;
            if metadata.file_type().is_symlink() {
                return Err(Error::Policy("rustc-wrapper-staged-output-symlink-rejected".to_string()));
            }
            if metadata.file_type().is_dir() {
                pending.push(entry.path());
            } else if metadata.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(stage_root)
                    .map_err(|_| Error::Policy("rustc-wrapper-staged-output-outside-stage".to_string()))?
                    .to_path_buf();
                files.insert(relative);
            } else {
                return Err(Error::Policy("rustc-wrapper-staged-output-kind-invalid".to_string()));
            }
            if pending.len().checked_add(files.len()).is_none_or(|count| count > MAX_STAGED_TREE_ENTRIES) {
                return Err(Error::Bound("rustc-wrapper-staged-tree-entry-limit-exceeded".to_string()));
            }
        }
    }
    if !pending.is_empty() {
        return Err(Error::Bound("rustc-wrapper-staged-tree-entry-limit-exceeded".to_string()));
    }
    if files != expected {
        return Err(Error::Policy("rustc-wrapper-staged-output-set-mismatch".to_string()));
    }
    assert_eq!(files.len(), contracts.len());
    Ok(())
}

fn commit_staged_outputs(stage_root: &Path, contracts: &[WrapperOutputContract]) -> Result<(), Error> {
    assert!(!stage_root.as_os_str().is_empty());
    assert!(!contracts.is_empty());
    let mut committed = Vec::with_capacity(contracts.len());
    for contract in contracts {
        let source = stage_root.join(&contract.staged_relative_path);
        let destination = Path::new(&contract.destination_path);
        if path_entry_exists(destination) {
            rollback_committed(&committed)?;
            return Err(Error::Policy("rustc-wrapper-output-destination-exists".to_string()));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| Error::Policy("rustc-wrapper-output-parent-missing".to_string()))?;
        if !parent.exists() {
            rollback_committed(&committed)?;
            return Err(Error::Policy("rustc-wrapper-output-parent-absent".to_string()));
        }
        if let Err(source_error) = rename_noreplace(&source, destination) {
            rollback_committed(&committed)?;
            return Err(Error::Io {
                context: "commit-staged-output".to_string(),
                source: source_error,
            });
        }
        committed.push((destination.to_path_buf(), source));
    }
    assert_eq!(committed.len(), contracts.len());
    Ok(())
}

fn rename_noreplace(source: &Path, destination: &Path) -> std::io::Result<()> {
    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let destination = CString::new(destination.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    // SAFETY: Both C strings are NUL-terminated path buffers that remain live
    // for this call. `AT_FDCWD` selects process-relative absolute paths, and
    // `RENAME_NOREPLACE` gives the required atomic no-clobber publication.
    let result = unsafe {
        libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, destination.as_ptr(), libc::RENAME_NOREPLACE)
    };
    if result == 0 {
        return Ok(());
    }
    Err(std::io::Error::last_os_error())
}

fn rollback_committed(committed: &[(PathBuf, PathBuf)]) -> Result<(), Error> {
    for (destination, source) in committed.iter().rev() {
        fs::rename(destination, source).map_err(|source_error| Error::Io {
            context: "rollback-staged-output".to_string(),
            source: source_error,
        })?;
    }
    assert!(committed.iter().all(|(destination, _source)| !destination.exists()));
    Ok(())
}

fn stage_root_for_request(request: &WrapperRequest, contracts: &[WrapperOutputContract]) -> Result<PathBuf, Error> {
    let parent = common_destination_parent(contracts)?;
    let digest = request
        .request_ref
        .rsplit('/')
        .next()
        .ok_or_else(|| Error::Protocol("rustc-wrapper-request-digest-missing".to_string()))?;
    let path = parent.join(format!(".mantle-rustc-{digest}.stage"));
    if path_entry_exists(&path) {
        return Err(Error::Policy("rustc-wrapper-stage-already-exists".to_string()));
    }
    assert!(path.starts_with(&parent));
    Ok(path)
}

fn common_destination_parent(contracts: &[WrapperOutputContract]) -> Result<PathBuf, Error> {
    let first = contracts.first().ok_or_else(|| Error::Policy("rustc-wrapper-output-contracts-empty".to_string()))?;
    let parent = Path::new(&first.destination_path)
        .parent()
        .ok_or_else(|| Error::Policy("rustc-wrapper-output-parent-missing".to_string()))?
        .to_path_buf();
    for contract in contracts.iter().skip(1) {
        if Path::new(&contract.destination_path).parent() != Some(parent.as_path()) {
            return Err(Error::Policy("rustc-wrapper-output-parents-differ".to_string()));
        }
    }
    assert!(
        contracts
            .iter()
            .all(|contract| Path::new(&contract.destination_path).parent() == Some(parent.as_path()))
    );
    Ok(parent)
}

fn build_client_request(
    policy: &WrapperDaemonPolicy,
    compiler: &OsString,
    arguments: Vec<String>,
) -> Result<WrapperRequest, Error> {
    assert!(!arguments.is_empty());
    assert!(!policy.policy_id.is_empty());
    let real_compiler_path = compiler
        .to_str()
        .ok_or_else(|| Error::Policy("rustc-wrapper-compiler-path-not-utf8".to_string()))?
        .to_string();
    let manifest_path = resolve_manifest_path(&arguments)?;
    let manifest = manifest_path
        .as_ref()
        .map(|path| load_json_file::<WrapperInvocationManifest>(path, MAX_MANIFEST_BYTES, "wrapper-manifest"))
        .transpose()?;
    let expected_manifest_ref = std::env::var(WRAPPER_MANIFEST_REF_ENV)
        .ok()
        .or_else(|| manifest.as_ref().map(|value| value.manifest_ref.clone()));
    let output_contracts = manifest.as_ref().map_or_else(Vec::new, |value| value.input.output_contracts.clone());
    let admitted_environment = admitted_environment(policy)?;
    let working_directory = std::env::current_dir()
        .map_err(|source| Error::Io {
            context: "read-wrapper-working-directory".to_string(),
            source,
        })?
        .to_str()
        .ok_or_else(|| Error::Policy("rustc-wrapper-working-directory-not-utf8".to_string()))?
        .to_string();
    let manifest_path = manifest_path
        .as_ref()
        .map(|path| {
            path.to_str()
                .map(str::to_string)
                .ok_or_else(|| Error::Policy("rustc-wrapper-manifest-path-not-utf8".to_string()))
        })
        .transpose()?;
    seal_wrapper_request(WrapperRequestInput {
        manifest_path,
        expected_manifest_ref,
        real_compiler_path,
        arguments,
        admitted_environment,
        working_directory,
        output_contracts,
    })
    .map_err(|error| Error::Core(String::from(error.code())))
}

fn resolve_manifest_path(arguments: &[String]) -> Result<Option<PathBuf>, Error> {
    assert!(!arguments.is_empty());
    if let Some(path) = std::env::var_os(WRAPPER_MANIFEST_ENV) {
        return Ok(Some(PathBuf::from(path)));
    }
    let Some(directory) = std::env::var_os(WRAPPER_MANIFEST_DIR_ENV) else {
        return Ok(None);
    };
    let digest = arguments_identity_blake3(arguments).map_err(|error| Error::Core(String::from(error.code())))?;
    let path = PathBuf::from(directory).join(format!("{digest}{RECEIPT_SUFFIX}"));
    assert!(path.file_name().is_some());
    Ok(Some(path))
}

fn admitted_environment(policy: &WrapperDaemonPolicy) -> Result<BTreeMap<String, String>, Error> {
    let environment = policy
        .allowed_environment
        .iter()
        .filter_map(|name| std::env::var(name).ok().map(|value| (name.clone(), value)))
        .collect::<BTreeMap<_, _>>();
    if environment.len() > crunch_rust_cache_core::wrapper::MAX_WRAPPER_ENVIRONMENT {
        return Err(Error::Bound("rustc-wrapper-environment-limit-exceeded".to_string()));
    }
    assert!(environment.keys().all(|name| policy.allowed_environment.binary_search(name).is_ok()));
    Ok(environment)
}

fn request_daemon(policy: &WrapperDaemonPolicy, request: &WrapperRequest) -> Result<WrapperResponse, Error> {
    assert!(!policy.socket_path.is_empty());
    assert!(!request.request_ref.is_empty());
    let mut stream = UnixStream::connect(&policy.socket_path).map_err(|source| Error::Io {
        context: "connect-daemon-socket".to_string(),
        source,
    })?;
    let timeout_millis = Some(Duration::from_millis(policy.limits.elapsed_millis));
    stream.set_read_timeout(timeout_millis).map_err(|source| Error::Io {
        context: "set-daemon-read-timeout".to_string(),
        source,
    })?;
    stream.set_write_timeout(timeout_millis).map_err(|source| Error::Io {
        context: "set-daemon-write-timeout".to_string(),
        source,
    })?;
    write_frame(&mut stream, request, policy.max_request_bytes)?;
    stream.flush().map_err(|source| Error::Io {
        context: "flush-wrapper-request".to_string(),
        source,
    })?;
    stream.shutdown(Shutdown::Write).map_err(|source| Error::Io {
        context: "finish-wrapper-request".to_string(),
        source,
    })?;
    let response = read_frame::<WrapperResponse>(&mut stream, policy.max_response_bytes)?;
    validate_wrapper_response(&response, policy.max_response_bytes)
        .map_err(|error| Error::Core(String::from(error.code())))?;
    if response.request_ref != request.request_ref {
        return Err(Error::Protocol("rustc-wrapper-response-request-mismatch".to_string()));
    }
    assert_eq!(response.request_ref, request.request_ref);
    Ok(response)
}

fn finish_client_response(
    policy: &WrapperDaemonPolicy,
    compiler: &OsString,
    request: &WrapperRequest,
    response: WrapperResponse,
) -> Result<i32, Error> {
    if response.disposition == WrapperDisposition::Bypass {
        return run_direct_compiler_strings(compiler, &request.input.arguments);
    }
    std::io::stdout().write_all(&response.stdout).map_err(|source| Error::Io {
        context: "write-wrapper-stdout".to_string(),
        source,
    })?;
    std::io::stderr().write_all(&response.stderr).map_err(|source| Error::Io {
        context: "write-wrapper-stderr".to_string(),
        source,
    })?;
    if response.disposition == WrapperDisposition::Rejected
        && policy.failure_mode == WrapperFailureMode::FailOpen
        && !response.reason_codes.iter().any(|reason| is_casita_integrity_reason(reason))
    {
        return run_direct_compiler_strings(compiler, &request.input.arguments);
    }
    Ok(response.compiler_status)
}

fn finish_client_failure(
    policy: &WrapperDaemonPolicy,
    compiler: &OsString,
    arguments: &[String],
    error: Error,
) -> Result<i32, Error> {
    if policy.failure_mode == WrapperFailureMode::FailOpen {
        eprintln!("mantle-rustc-wrapper:bypass:{error}");
        return run_direct_compiler_strings(compiler, arguments);
    }
    eprintln!("mantle-rustc-wrapper:rejected:{error}");
    Ok(EXIT_PROTOCOL_FAILURE)
}

fn run_direct_compiler(compiler: &OsString, arguments: &[OsString]) -> Result<i32, Error> {
    let status = Command::new(compiler).args(arguments).status().map_err(|source| Error::Io {
        context: "execute-direct-compiler".to_string(),
        source,
    })?;
    Ok(exit_code(status))
}

fn run_direct_compiler_strings(compiler: &OsString, arguments: &[String]) -> Result<i32, Error> {
    let status = Command::new(compiler).args(arguments).status().map_err(|source| Error::Io {
        context: "execute-direct-compiler".to_string(),
        source,
    })?;
    Ok(exit_code(status))
}

fn split_wrapper_arguments(arguments: &[OsString]) -> Result<(OsString, Vec<OsString>), Error> {
    if arguments.len() < 3 {
        return Err(Error::Protocol("rustc-wrapper-arguments-missing".to_string()));
    }
    let compiler = arguments[1].clone();
    let compiler_arguments = arguments[2..].to_vec();
    assert!(!compiler_arguments.is_empty());
    Ok((compiler, compiler_arguments))
}

fn os_strings_to_strings(arguments: &[OsString]) -> Result<Vec<String>, Error> {
    arguments
        .iter()
        .map(|argument| {
            argument
                .to_str()
                .map(str::to_string)
                .ok_or_else(|| Error::Policy("rustc-wrapper-argument-not-utf8".to_string()))
        })
        .collect()
}

fn load_request_manifest(request: &WrapperRequest) -> Result<Option<WrapperInvocationManifest>, Error> {
    request
        .input
        .manifest_path
        .as_ref()
        .map(|path| load_json_file(Path::new(path), MAX_MANIFEST_BYTES, "daemon-manifest"))
        .transpose()
}

fn load_json_file<T: DeserializeOwned>(path: &Path, limit: u64, context: &str) -> Result<T, Error> {
    let bytes = read_file_bounded(path, limit, context)?;
    serde_json::from_slice(&bytes).map_err(|source| Error::Json {
        context: format!("decode-{context}"),
        source,
    })
}

fn read_file_bounded(path: &Path, limit: u64, context: &str) -> Result<Vec<u8>, Error> {
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path).map_err(
        |source| Error::Io {
            context: format!("open-{context}"),
            source,
        },
    )?;
    let metadata = file.metadata().map_err(|source| Error::Io {
        context: format!("stat-{context}"),
        source,
    })?;
    if !metadata.file_type().is_file() || metadata.len() > limit {
        return Err(Error::Bound(format!("{context}-invalid-or-too-large")));
    }
    let capacity_bytes =
        usize::try_from(metadata.len()).map_err(|_| Error::Bound(format!("{context}-size-unrepresentable")))?;
    let mut bytes = vec![0_u8; capacity_bytes];
    file.read_exact(&mut bytes).map_err(|source| Error::Io {
        context: format!("read-{context}"),
        source,
    })?;
    let mut extra = [0_u8; 1];
    let extra_count_bytes = file.read(&mut extra).map_err(|source| Error::Io {
        context: format!("check-eof-{context}"),
        source,
    })?;
    if extra_count_bytes != 0 {
        return Err(Error::Bound(format!("{context}-changed-during-read")));
    }
    assert_eq!(bytes.len(), capacity_bytes);
    assert_eq!(extra_count_bytes, 0);
    Ok(bytes)
}

fn read_stream_bounded(mut stream: impl Read, limit: u64, context: &str) -> Result<Vec<u8>, Error> {
    let output_capacity_bytes =
        usize::try_from(limit).map_err(|_| Error::Bound(format!("{context}-capacity-unrepresentable")))?;
    let mut output = Vec::with_capacity(output_capacity_bytes);
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let read_buffer_bytes =
        u64::try_from(READ_BUFFER_BYTES).map_err(|_| Error::Bound(format!("{context}-buffer-size-unrepresentable")))?;
    let iteration_count_max = limit
        .checked_div(read_buffer_bytes)
        .and_then(|count| count.checked_add(EXTRA_EOF_READ_ITERATIONS))
        .ok_or_else(|| Error::Bound(format!("{context}-iteration-count-overflow")))?;
    for _ in 0..iteration_count_max {
        let count = stream.read(&mut buffer).map_err(|source| Error::Io {
            context: format!("read-{context}"),
            source,
        })?;
        if count == 0 {
            break;
        }
        let next = u64::try_from(output.len())
            .ok()
            .and_then(|current| current.checked_add(u64::try_from(count).ok()?))
            .ok_or_else(|| Error::Bound(format!("{context}-size-overflow")))?;
        if next > limit {
            return Err(Error::Bound(format!("{context}-too-large")));
        }
        output.extend_from_slice(&buffer[..count]);
    }
    assert!(u64::try_from(output.len()).is_ok_and(|size| size <= limit));
    assert!(output.len() <= output_capacity_bytes);
    Ok(output)
}

fn hash_directory_bounded(root: &Path, limit_bytes: u64) -> Result<String, Error> {
    let max_entries = usize::try_from(crunch_rust_cache_core::MAX_TREE_ENTRIES)
        .map_err(|_| Error::Bound("rustc-wrapper-directory-entry-limit-unrepresentable".to_string()))?;
    let mut scan = DirectoryScan {
        root,
        limit_bytes,
        total_bytes: 0,
        max_entries,
        facts: Vec::with_capacity(max_entries),
        pending: Vec::with_capacity(max_entries),
    };
    scan.pending.push(PendingDirectory {
        path: root.to_path_buf(),
        depth: 0,
    });
    for _ in 0..max_entries {
        let Some(directory) = scan.pending.pop() else {
            break;
        };
        scan_one_directory(&mut scan, directory)?;
    }
    if !scan.pending.is_empty() {
        return Err(Error::Bound("rustc-wrapper-directory-entry-limit-exceeded".to_string()));
    }
    scan.facts.sort();
    if scan.facts.windows(2).any(|pair| pair[0].relative_path == pair[1].relative_path) {
        return Err(Error::Policy("rustc-wrapper-directory-entry-duplicate".to_string()));
    }
    let digest = directory_identity_from_facts(&scan.facts)?;
    assert!(scan.facts.len() <= max_entries);
    assert!(scan.total_bytes <= limit_bytes);
    Ok(digest)
}

struct DirectoryScan<'a> {
    root: &'a Path,
    limit_bytes: u64,
    total_bytes: u64,
    max_entries: usize,
    facts: Vec<DirectoryEntryFact>,
    pending: Vec<PendingDirectory>,
}

fn scan_one_directory(scan: &mut DirectoryScan<'_>, directory: PendingDirectory) -> Result<(), Error> {
    if directory.depth > crunch_rust_cache_core::MAX_TREE_DEPTH {
        return Err(Error::Bound("rustc-wrapper-directory-depth-limit-exceeded".to_string()));
    }
    let remaining_entries = scan
        .max_entries
        .checked_sub(scan.facts.len())
        .ok_or_else(|| Error::Bound("rustc-wrapper-directory-entry-count-underflow".to_string()))?;
    let entries = read_directory_entries(&directory.path, remaining_entries)?;
    for entry in entries {
        let fact = scan_directory_entry(scan, &directory, entry.path())?;
        scan.facts.push(fact);
    }
    assert!(scan.facts.len() <= scan.max_entries);
    assert!(scan.total_bytes <= scan.limit_bytes);
    Ok(())
}

fn read_directory_entries(path: &Path, remaining_entries: usize) -> Result<Vec<fs::DirEntry>, Error> {
    assert!(!path.as_os_str().is_empty());
    let scan_entries = remaining_entries
        .checked_add(1)
        .ok_or_else(|| Error::Bound("rustc-wrapper-directory-scan-count-overflow".to_string()))?;
    let read_dir = fs::read_dir(path).map_err(|source| Error::Io {
        context: "read-declared-directory".to_string(),
        source,
    })?;
    let mut entries = Vec::with_capacity(scan_entries);
    for entry in read_dir.take(scan_entries) {
        entries.push(entry.map_err(|source| Error::Io {
            context: "read-declared-directory-entry".to_string(),
            source,
        })?);
    }
    if entries.len() > remaining_entries {
        return Err(Error::Bound("rustc-wrapper-directory-entry-limit-exceeded".to_string()));
    }
    entries.sort_by(|left, right| left.file_name().as_bytes().cmp(right.file_name().as_bytes()));
    assert!(entries.len() <= remaining_entries);
    assert!(path.is_dir());
    Ok(entries)
}

fn scan_directory_entry(
    scan: &mut DirectoryScan<'_>,
    directory: &PendingDirectory,
    path: PathBuf,
) -> Result<DirectoryEntryFact, Error> {
    let metadata = fs::symlink_metadata(&path).map_err(|source| Error::Io {
        context: "stat-declared-directory-entry".to_string(),
        source,
    })?;
    let relative_path = path
        .strip_prefix(scan.root)
        .map_err(|_| Error::Policy("rustc-wrapper-directory-entry-outside-root".to_string()))?
        .as_os_str()
        .as_bytes()
        .to_vec();
    let mode = metadata.permissions().mode() & 0o7777;
    let (kind, size_bytes, content_identity) = classify_directory_entry(scan, directory, &path, &metadata)?;
    assert!(!relative_path.is_empty());
    assert!(scan.total_bytes <= scan.limit_bytes);
    Ok(DirectoryEntryFact {
        relative_path,
        kind,
        mode,
        size_bytes,
        content_identity,
    })
}

fn classify_directory_entry(
    scan: &mut DirectoryScan<'_>,
    directory: &PendingDirectory,
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<(DirectoryEntryKind, u64, Vec<u8>), Error> {
    assert!(!path.as_os_str().is_empty());
    assert!(scan.total_bytes <= scan.limit_bytes);
    if metadata.file_type().is_dir() {
        let depth = directory
            .depth
            .checked_add(1)
            .ok_or_else(|| Error::Bound("rustc-wrapper-directory-depth-overflow".to_string()))?;
        scan.pending.push(PendingDirectory {
            path: path.to_path_buf(),
            depth,
        });
        return Ok((DirectoryEntryKind::Directory, 0, Vec::new()));
    }
    if metadata.file_type().is_file() {
        add_directory_bytes(scan, metadata.len())?;
        let digest = hash_file_bounded(path, scan.limit_bytes)?;
        return Ok((DirectoryEntryKind::File, metadata.len(), digest.into_bytes()));
    }
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(path).map_err(|source| Error::Io {
            context: "read-declared-directory-symlink".to_string(),
            source,
        })?;
        if !symlink_target_stays_within_root(scan.root, path, &target) {
            return Err(Error::Policy("rustc-wrapper-directory-symlink-escape".to_string()));
        }
        let target = target.as_os_str().as_bytes().to_vec();
        let size_bytes = u64::try_from(target.len())
            .map_err(|_| Error::Bound("rustc-wrapper-symlink-size-unrepresentable".to_string()))?;
        add_directory_bytes(scan, size_bytes)?;
        return Ok((DirectoryEntryKind::Symlink, size_bytes, target));
    }
    Err(Error::Policy("rustc-wrapper-directory-entry-kind-unsupported".to_string()))
}

fn symlink_target_stays_within_root(root: &Path, link: &Path, target: &Path) -> bool {
    assert!(!root.as_os_str().is_empty());
    assert!(!link.as_os_str().is_empty());
    let Some(parent) = link.parent() else {
        return false;
    };
    let Ok(relative_parent) = parent.strip_prefix(root) else {
        return false;
    };
    let mut depth = relative_parent.components().filter(|component| matches!(component, Component::Normal(_))).count();
    for component in target.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(_) => {
                let Some(next_depth) = depth.checked_add(1) else {
                    return false;
                };
                depth = next_depth;
            }
            Component::ParentDir => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next_depth;
            }
            Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

fn add_directory_bytes(scan: &mut DirectoryScan<'_>, size_bytes: u64) -> Result<(), Error> {
    scan.total_bytes = scan
        .total_bytes
        .checked_add(size_bytes)
        .ok_or_else(|| Error::Bound("rustc-wrapper-directory-bytes-overflow".to_string()))?;
    if scan.total_bytes > scan.limit_bytes {
        return Err(Error::Bound("rustc-wrapper-directory-bytes-limit-exceeded".to_string()));
    }
    assert!(scan.total_bytes >= size_bytes);
    assert!(scan.total_bytes <= scan.limit_bytes);
    Ok(())
}

fn directory_identity_from_facts(facts: &[DirectoryEntryFact]) -> Result<String, Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DIRECTORY_IDENTITY_DOMAIN);
    hasher.update(&[DIRECTORY_IDENTITY_SEPARATOR]);
    for fact in facts {
        update_hash_bytes(&mut hasher, &fact.relative_path)?;
        let kind = match fact.kind {
            DirectoryEntryKind::Directory => 0_u8,
            DirectoryEntryKind::File => 1_u8,
            DirectoryEntryKind::Symlink => 2_u8,
        };
        hasher.update(&[kind]);
        hasher.update(&fact.mode.to_be_bytes());
        hasher.update(&fact.size_bytes.to_be_bytes());
        update_hash_bytes(&mut hasher, &fact.content_identity)?;
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    assert!(facts.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(digest)
}

fn update_hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), Error> {
    let length_bytes = u64::try_from(bytes.len())
        .map_err(|_| Error::Bound("rustc-wrapper-directory-field-size-unrepresentable".to_string()))?;
    hasher.update(&length_bytes.to_be_bytes());
    hasher.update(bytes);
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(length_bytes));
    Ok(())
}

fn hash_file_bounded(path: &Path, limit: u64) -> Result<String, Error> {
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path).map_err(
        |source| Error::Io {
            context: "open-hash-input".to_string(),
            source,
        },
    )?;
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0_u64;
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let read_buffer_bytes = u64::try_from(READ_BUFFER_BYTES)
        .map_err(|_| Error::Bound("rustc-wrapper-hash-buffer-size-unrepresentable".to_string()))?;
    let iteration_count_max = limit
        .checked_div(read_buffer_bytes)
        .and_then(|count| count.checked_add(EXTRA_EOF_READ_ITERATIONS))
        .ok_or_else(|| Error::Bound("rustc-wrapper-hash-iteration-count-overflow".to_string()))?;
    for _ in 0..iteration_count_max {
        let count = file.read(&mut buffer).map_err(|source| Error::Io {
            context: "read-hash-input".to_string(),
            source,
        })?;
        if count == 0 {
            break;
        }
        total_bytes = total_bytes
            .checked_add(
                u64::try_from(count).map_err(|_| Error::Bound("rustc-wrapper-hash-count-invalid".to_string()))?,
            )
            .ok_or_else(|| Error::Bound("rustc-wrapper-hash-bytes-overflow".to_string()))?;
        if total_bytes > limit {
            return Err(Error::Bound("rustc-wrapper-hash-input-too-large".to_string()));
        }
        hasher.update(&buffer[..count]);
    }
    assert!(total_bytes <= limit);
    assert!(iteration_count_max >= EXTRA_EOF_READ_ITERATIONS);
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn write_frame<T: Serialize>(writer: &mut impl Write, value: &T, limit: u64) -> Result<(), Error> {
    let bytes = serde_json::to_vec(value).map_err(|source| Error::Json {
        context: "encode-protocol-frame".to_string(),
        source,
    })?;
    let length_bytes =
        u64::try_from(bytes.len()).map_err(|_| Error::Bound("rustc-wrapper-frame-size-invalid".to_string()))?;
    if length_bytes == 0 || length_bytes > limit {
        return Err(Error::Bound("rustc-wrapper-frame-size-invalid".to_string()));
    }
    writer.write_all(&length_bytes.to_be_bytes()).map_err(|source| Error::Io {
        context: "write-protocol-frame-prefix".to_string(),
        source,
    })?;
    writer.write_all(&bytes).map_err(|source| Error::Io {
        context: "write-protocol-frame-body".to_string(),
        source,
    })?;
    assert!(length_bytes > 0);
    assert!(length_bytes <= limit);
    Ok(())
}

pub fn read_frame<T: DeserializeOwned>(reader: &mut impl Read, limit: u64) -> Result<T, Error> {
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    reader.read_exact(&mut prefix).map_err(|source| Error::Io {
        context: "read-protocol-frame-prefix".to_string(),
        source,
    })?;
    let length_bytes = u64::from_be_bytes(prefix);
    if length_bytes == 0 || length_bytes > limit {
        return Err(Error::Bound("rustc-wrapper-frame-size-invalid".to_string()));
    }
    let frame_size_bytes =
        usize::try_from(length_bytes).map_err(|_| Error::Bound("rustc-wrapper-frame-size-invalid".to_string()))?;
    let mut bytes = vec![0_u8; frame_size_bytes];
    reader.read_exact(&mut bytes).map_err(|source| Error::Io {
        context: "read-protocol-frame-body".to_string(),
        source,
    })?;
    let value = serde_json::from_slice(&bytes).map_err(|source| Error::Json {
        context: "decode-protocol-frame".to_string(),
        source,
    })?;
    assert!(length_bytes > 0);
    assert_eq!(bytes.len(), frame_size_bytes);
    Ok(value)
}

fn publish_immutable_bytes(directory: &Path, destination: &Path, bytes: &[u8], context: &str) -> Result<(), Error> {
    assert!(destination.starts_with(directory));
    assert!(!bytes.is_empty());
    let mut temporary =
        Builder::new()
            .prefix(".mantle-rustc-")
            .suffix(TEMP_SUFFIX)
            .tempfile_in(directory)
            .map_err(|source| Error::Io {
                context: format!("create-{context}-stage"),
                source,
            })?;
    temporary.as_file_mut().write_all(bytes).map_err(|source| Error::Io {
        context: format!("write-{context}-stage"),
        source,
    })?;
    temporary.as_file_mut().sync_all().map_err(|source| Error::Io {
        context: format!("sync-{context}-stage"),
        source,
    })?;
    temporary.persist_noclobber(destination).map_err(|error| Error::Io {
        context: format!("publish-{context}"),
        source: error.error,
    })?;
    assert!(destination.is_file());
    Ok(())
}

fn persist_receipt(directory: &Path, receipt: WrapperReceipt) -> Result<WrapperReceipt, Error> {
    assert!(!directory.as_os_str().is_empty());
    assert!(!receipt.receipt_ref.is_empty());
    create_private_directory(directory)?;
    let digest = receipt
        .receipt_ref
        .rsplit('/')
        .next()
        .ok_or_else(|| Error::Protocol("rustc-wrapper-receipt-digest-missing".to_string()))?;
    let destination = directory.join(format!("{digest}{RECEIPT_SUFFIX}"));
    let bytes = serde_json::to_vec_pretty(&receipt).map_err(|source| Error::Json {
        context: "encode-wrapper-receipt".to_string(),
        source,
    })?;
    if destination.exists() {
        let existing = read_file_bounded(
            &destination,
            crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESPONSE_BYTES,
            "existing-wrapper-receipt",
        )?;
        let existing_receipt = serde_json::from_slice::<WrapperReceipt>(&existing).map_err(|source| Error::Json {
            context: "decode-existing-wrapper-receipt".to_string(),
            source,
        })?;
        if existing_receipt != receipt {
            return Err(Error::Protocol("rustc-wrapper-receipt-collision".to_string()));
        }
        return Ok(receipt);
    }
    let mut temporary =
        Builder::new().prefix(".mantle-rustc-receipt-").suffix(TEMP_SUFFIX).tempfile_in(directory).map_err(
            |source| Error::Io {
                context: "create-wrapper-receipt-stage".to_string(),
                source,
            },
        )?;
    temporary.as_file_mut().write_all(&bytes).map_err(|source| Error::Io {
        context: "write-wrapper-receipt-stage".to_string(),
        source,
    })?;
    temporary.as_file_mut().sync_all().map_err(|source| Error::Io {
        context: "sync-wrapper-receipt-stage".to_string(),
        source,
    })?;
    temporary.persist_noclobber(&destination).map_err(|error| Error::Io {
        context: "publish-wrapper-receipt".to_string(),
        source: error.error,
    })?;
    assert!(destination.is_file());
    Ok(receipt)
}

fn publish_shared_result(
    context: &DaemonContext,
    result: &crunch_rust_cache_core::RustUnitResult,
) -> Result<(), Error> {
    let publication = &context.policy.publication;
    let source_id = publication
        .source_id
        .as_ref()
        .ok_or_else(|| Error::Policy("rustc-wrapper-publication-source-missing".to_string()))?;
    let source = context
        .sources
        .iter()
        .find(|source| source.source_id() == source_id)
        .ok_or_else(|| Error::Policy("rustc-wrapper-publication-source-unknown".to_string()))?;
    let signer_name = publication
        .signer_name
        .as_ref()
        .ok_or_else(|| Error::Policy("rustc-wrapper-publication-signer-missing".to_string()))?;
    let key_path = publication
        .signing_key_path
        .as_ref()
        .ok_or_else(|| Error::Policy("rustc-wrapper-publication-key-missing".to_string()))?;
    let signing_key = load_signing_key(Path::new(key_path))?;
    let publication_outcome = context
        .cache
        .publish_shared_blocking(source.as_ref(), SharedPublishRequest {
            result,
            producer: RustResultProducerIdentity {
                producer_id: "mantle-rust-cache-daemon".to_string(),
                producer_policy_id: context.policy.policy_id.clone(),
            },
            signer_name: signer_name.clone(),
            signing_key: &signing_key,
            policy: &context.shared_policy,
        })
        .map_err(|error| Error::Process(format!("shared-cache-publish:{error}")))?;
    assert!(context.shared_policy.publishes_enabled);
    assert_eq!(publication_outcome.result_ref, result.result_ref);
    Ok(())
}

fn load_signing_key(path: &Path) -> Result<SigningKey, Error> {
    let key_limit_bytes = u64::try_from(SIGNING_KEY_HEX_BYTES)
        .map_err(|_| Error::Bound("rustc-wrapper-signing-key-limit-unrepresentable".to_string()))?;
    let bytes = read_file_bounded(path, key_limit_bytes, "rust-cache-signing-key")?;
    let key_bytes = if bytes.len() == SIGNING_KEY_BYTES {
        bytes
    } else if bytes.len() == SIGNING_KEY_HEX_BYTES {
        decode_hex(&bytes)?
    } else {
        return Err(Error::Policy("rustc-wrapper-signing-key-size-invalid".to_string()));
    };
    let key: [u8; SIGNING_KEY_BYTES] = key_bytes
        .try_into()
        .map_err(|_| Error::Policy("rustc-wrapper-signing-key-size-invalid".to_string()))?;
    Ok(SigningKey::from_bytes(&key))
}

const HEX_PAIR_BYTES: usize = 2;

fn decode_hex(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    if !bytes.len().is_multiple_of(HEX_PAIR_BYTES) {
        return Err(Error::Policy("rustc-wrapper-signing-key-hex-invalid".to_string()));
    }
    let mut output = Vec::with_capacity(bytes.len() / HEX_PAIR_BYTES);
    let (pairs, remainder) = bytes.as_chunks::<HEX_PAIR_BYTES>();
    debug_assert!(remainder.is_empty());
    for [high_byte, low_byte] in pairs {
        let high = hex_value(*high_byte)?;
        let low = hex_value(*low_byte)?;
        output.push((high << 4) | low);
    }
    assert_eq!(output.len().checked_mul(HEX_PAIR_BYTES), Some(bytes.len()));
    Ok(output)
}

fn hex_value(byte: u8) -> Result<u8, Error> {
    match byte {
        b'0'..=b'9' => byte
            .checked_sub(b'0')
            .ok_or_else(|| Error::Policy("rustc-wrapper-signing-key-hex-invalid".to_string())),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(10))
            .ok_or_else(|| Error::Policy("rustc-wrapper-signing-key-hex-invalid".to_string())),
        _ => Err(Error::Policy("rustc-wrapper-signing-key-hex-invalid".to_string())),
    }
}

fn open_result_sources(policy: &WrapperDaemonPolicy) -> Result<Vec<Arc<dyn RustResultSource>>, Error> {
    assert!(policy.result_sources.len() <= crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESULT_SOURCES);
    let mut sources = Vec::<Arc<dyn RustResultSource>>::with_capacity(policy.result_sources.len());
    for configured in &policy.result_sources {
        let source: Arc<dyn RustResultSource> = match configured.kind {
            crunch_rust_cache_core::wrapper::WrapperResultSourceKind::Directory => Arc::new(
                DirectoryRustResultSource::open(PathBuf::from(&configured.endpoint), false)
                    .map_err(|error| Error::Process(format!("open-directory-result-source:{error}")))?,
            ),
            crunch_rust_cache_core::wrapper::WrapperResultSourceKind::Http => Arc::new(
                HttpRustResultSource::new(&configured.endpoint, Duration::from_millis(policy.limits.elapsed_millis))
                    .map_err(|error| Error::Process(format!("open-http-result-source:{error}")))?,
            ),
        };
        if source.source_id() != configured.source_id {
            return Err(Error::Policy("rustc-wrapper-result-source-id-mismatch".to_string()));
        }
        sources.push(source);
    }
    assert_eq!(sources.len(), policy.result_sources.len());
    assert!(sources.len() <= crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESULT_SOURCES);
    Ok(sources)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingSocketProbe {
    Active,
    Stale,
    Gone,
    Indeterminate,
}

fn classify_existing_socket_probe(result: &std::io::Result<UnixStream>) -> ExistingSocketProbe {
    match result {
        Ok(_) => ExistingSocketProbe::Active,
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => ExistingSocketProbe::Stale,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ExistingSocketProbe::Gone,
        Err(_) => ExistingSocketProbe::Indeterminate,
    }
}

fn prepare_socket(socket: &str) -> Result<(), Error> {
    assert!(!socket.is_empty());
    let path = Path::new(socket);
    let parent = path.parent().ok_or_else(|| Error::Policy("rustc-wrapper-socket-parent-missing".to_string()))?;
    create_private_directory(parent)?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_socket() {
            return Err(Error::Policy("rustc-wrapper-socket-path-occupied".to_string()));
        }
        let probe = UnixStream::connect(path);
        match classify_existing_socket_probe(&probe) {
            ExistingSocketProbe::Active => {
                return Err(Error::Policy("rustc-wrapper-socket-already-active".to_string()));
            }
            ExistingSocketProbe::Stale => {
                fs::remove_file(path).map_err(|source| Error::Io {
                    context: "remove-stale-daemon-socket".to_string(),
                    source,
                })?;
            }
            ExistingSocketProbe::Gone => {}
            ExistingSocketProbe::Indeterminate => {
                return Err(Error::Policy("rustc-wrapper-socket-probe-failed".to_string()));
            }
        }
    }
    assert!(!path.exists());
    Ok(())
}

fn remove_socket(socket: &str) {
    let path = Path::new(socket);
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => {
            if let Err(error) = fs::remove_file(path) {
                eprintln!("mantle-rust-cache-daemon:remove-socket:{error}");
            }
        }
        Ok(_) | Err(_) => {}
    }
}

fn create_private_directory(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|source| Error::Io {
        context: "create-private-directory".to_string(),
        source,
    })?;
    let metadata = fs::symlink_metadata(path).map_err(|source| Error::Io {
        context: "stat-private-directory".to_string(),
        source,
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(Error::Policy("rustc-wrapper-private-directory-invalid".to_string()));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(|source| Error::Io {
        context: "chmod-private-directory".to_string(),
        source,
    })?;
    assert!(path.is_dir());
    Ok(())
}

fn remove_tree_if_exists(path: &Path) -> Result<(), Error> {
    assert!(!path.as_os_str().is_empty());
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                return Err(Error::Policy("rustc-wrapper-stage-cleanup-path-invalid".to_string()));
            }
            fs::remove_dir_all(path).map_err(|source| Error::Io {
                context: "remove-wrapper-stage".to_string(),
                source,
            })?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(Error::Io {
                context: "stat-wrapper-stage".to_string(),
                source,
            });
        }
    }
    assert!(!path.exists());
    Ok(())
}

fn output_parent_is_admitted(destination: &str, roots: &[String]) -> bool {
    assert!(!destination.is_empty());
    assert!(!roots.is_empty());
    let Some(parent) = Path::new(destination).parent() else {
        return false;
    };
    let Ok(canonical_parent) = fs::canonicalize(parent) else {
        return false;
    };
    for root in roots {
        let root_path = Path::new(root);
        let Ok(canonical_root) = fs::canonicalize(root_path) else {
            continue;
        };
        if canonical_root != root_path {
            continue;
        }
        if canonical_parent.starts_with(&canonical_root) {
            return true;
        }
    }
    false
}

fn path_entry_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn path_is_within_roots(path: &str, roots: &[String]) -> bool {
    roots
        .iter()
        .any(|root| path == root || path.strip_prefix(root).is_some_and(|remainder| remainder.starts_with('/')))
}

fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(EXIT_PROTOCOL_FAILURE)
}

fn peer_uid(stream: &UnixStream) -> Result<u32, Error> {
    let mut credentials = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut credential_length_bytes = libc::socklen_t::try_from(std::mem::size_of::<libc::ucred>())
        .map_err(|_| Error::Bound("rustc-wrapper-peer-credential-size-unrepresentable".to_string()))?;
    // SAFETY: `credentials` and `length` are valid writable buffers for
    // SO_PEERCRED, and `stream` owns a live Unix-domain socket descriptor.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            std::ptr::addr_of_mut!(credentials).cast(),
            std::ptr::addr_of_mut!(credential_length_bytes),
        )
    };
    if result != 0 || usize::try_from(credential_length_bytes).ok() != Some(std::mem::size_of::<libc::ucred>()) {
        return Err(Error::Protocol("rustc-wrapper-peer-credentials-unavailable".to_string()));
    }
    assert!(credentials.pid > 0);
    Ok(credentials.uid)
}

extern "C" fn shutdown_signal_handler(_signal: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

fn install_shutdown_handlers() -> Result<(), Error> {
    let handler = shutdown_signal_handler as extern "C" fn(libc::c_int);
    let handler = handler as *const ();
    let handler = handler as libc::sighandler_t;
    // SAFETY: The handler only performs an atomic store, which is
    // async-signal-safe for this lock-free static atomic on supported targets.
    let term_previous = unsafe { libc::signal(libc::SIGTERM, handler) };
    // SAFETY: This installs the same bounded handler for SIGINT.
    let int_previous = unsafe { libc::signal(libc::SIGINT, handler) };
    if term_previous == libc::SIG_ERR || int_previous == libc::SIG_ERR {
        return Err(Error::Process("rustc-wrapper-signal-handler-install-failed".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_round_trip_is_exact() {
        let value = vec!["alpha".to_string(), "beta".to_string()];
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &value, 1_024).unwrap();
        let decoded = read_frame::<Vec<String>>(&mut bytes.as_slice(), 1_024).unwrap();
        assert_eq!(decoded, value);
        assert!(bytes.len() > FRAME_PREFIX_BYTES);
    }

    #[test]
    fn framing_rejects_oversized_prefix_before_allocation() {
        let mut bytes = (2_048_u64).to_be_bytes().to_vec();
        bytes.extend_from_slice(b"{}");
        let error = read_frame::<serde_json::Value>(&mut bytes.as_slice(), 1_024).unwrap_err();
        assert!(matches!(error, Error::Bound(_)));
        assert!(error.to_string().contains("frame-size-invalid"));
    }

    #[test]
    fn framing_rejects_truncated_prefix_and_payload() {
        const FRAME_LIMIT_BYTES: u64 = 1_024;
        const DECLARED_PAYLOAD_BYTES: u64 = 4;
        let truncated_prefix = [0_u8; FRAME_PREFIX_BYTES - 1];
        let prefix_error = read_frame::<serde_json::Value>(&mut truncated_prefix.as_slice(), FRAME_LIMIT_BYTES)
            .expect_err("a truncated prefix must fail");
        assert!(prefix_error.to_string().contains("read-protocol-frame-prefix"));

        let mut truncated_payload = DECLARED_PAYLOAD_BYTES.to_be_bytes().to_vec();
        truncated_payload.extend_from_slice(b"{}");
        let payload_error = read_frame::<serde_json::Value>(&mut truncated_payload.as_slice(), FRAME_LIMIT_BYTES)
            .expect_err("a truncated payload must fail");
        assert!(payload_error.to_string().contains("read-protocol-frame-body"));
    }

    #[test]
    fn directory_identity_is_canonical_and_content_bound() {
        const DIRECTORY_LIMIT_BYTES: u64 = 4_096;
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("nested")).unwrap();
        fs::write(root.path().join("alpha"), b"alpha").unwrap();
        fs::write(root.path().join("nested/beta"), b"beta").unwrap();
        let first =
            declared_input_digest_blake3(root.path(), DeclaredInputKind::Directory, DIRECTORY_LIMIT_BYTES).unwrap();
        let second =
            declared_input_digest_blake3(root.path(), DeclaredInputKind::Directory, DIRECTORY_LIMIT_BYTES).unwrap();
        assert_eq!(first, second);
        fs::write(root.path().join("nested/beta"), b"changed").unwrap();
        let changed =
            declared_input_digest_blake3(root.path(), DeclaredInputKind::Directory, DIRECTORY_LIMIT_BYTES).unwrap();
        assert_ne!(first, changed);
    }

    #[test]
    fn directory_identity_rejects_byte_limit_excess() {
        const DIRECTORY_LIMIT_BYTES: u64 = 1;
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("large"), b"too-large").unwrap();
        let error =
            declared_input_digest_blake3(root.path(), DeclaredInputKind::Directory, DIRECTORY_LIMIT_BYTES).unwrap_err();
        assert!(matches!(error, Error::Bound(_)));
        assert!(error.to_string().contains("bytes-limit-exceeded"));
    }

    #[test]
    fn directory_identity_rejects_escaping_symlink() {
        const DIRECTORY_LIMIT_BYTES: u64 = 4_096;
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("../outside", root.path().join("escape")).unwrap();
        let error =
            declared_input_digest_blake3(root.path(), DeclaredInputKind::Directory, DIRECTORY_LIMIT_BYTES).unwrap_err();
        assert!(matches!(error, Error::Policy(_)));
        assert!(error.to_string().contains("symlink-escape"));
    }

    #[test]
    fn peer_credentials_reject_a_different_uid() {
        let (client, _server) = UnixStream::pair().unwrap();
        let observed_uid = peer_uid(&client).unwrap();
        let rejected_uid = observed_uid.checked_add(1).unwrap_or_else(|| observed_uid.saturating_sub(1));
        let allowed_peer_uids = [rejected_uid];
        assert_eq!(observed_uid, unsafe { libc::geteuid() });
        assert!(allowed_peer_uids.binary_search(&observed_uid).is_err());
    }

    #[test]
    fn compiler_timeout_kills_and_reaps_the_child() {
        const TIMEOUT_MILLIS: u64 = 10;
        const SLEEP_SECONDS: &str = "1";
        let mut child = Command::new(find_test_executable("sleep")).arg(SLEEP_SECONDS).spawn().unwrap();
        let (status, is_timeout) = wait_for_compiler(&mut child, TIMEOUT_MILLIS).unwrap();
        assert!(is_timeout);
        assert!(!status.success());
    }

    #[test]
    fn socket_preparation_removes_stale_socket_but_preserves_active_daemon() {
        let root = tempfile::tempdir().unwrap();
        let socket_path = root.path().join("daemon.sock");
        let stale_listener = UnixListener::bind(&socket_path).unwrap();
        drop(stale_listener);
        assert!(socket_path.exists());
        prepare_socket(socket_path.to_str().unwrap()).unwrap();
        assert!(!socket_path.exists());

        let active_listener = UnixListener::bind(&socket_path).unwrap();
        let error = prepare_socket(socket_path.to_str().unwrap()).expect_err("an active daemon must not be unlinked");
        assert!(error.to_string().contains("socket-already-active"));
        assert!(socket_path.exists());
        drop(active_listener);
    }

    #[test]
    fn socket_preparation_rejects_a_non_socket_entry() {
        let root = tempfile::tempdir().unwrap();
        let socket_path = root.path().join("daemon.sock");
        fs::write(&socket_path, b"occupied").unwrap();
        let error = prepare_socket(socket_path.to_str().unwrap()).expect_err("a file must not be replaced");
        assert!(error.to_string().contains("socket-path-occupied"));
        assert_eq!(fs::read(&socket_path).unwrap(), b"occupied");
    }

    #[test]
    fn output_rewrite_changes_only_declared_destination_space() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("out.o");
        let stage = root.path().join("stage");
        let arguments = vec![
            "-o".to_string(),
            destination.display().to_string(),
            "input.rs".to_string(),
        ];
        let contracts = vec![WrapperOutputContract {
            staged_relative_path: "out.o".to_string(),
            destination_path: destination.display().to_string(),
            kind: WrapperArtifactKind::File,
            max_bytes: 1_024,
            executable: false,
        }];
        let rewritten = rewrite_output_arguments(&arguments, &contracts, &stage).unwrap();
        assert_eq!(rewritten[0], "-o");
        assert_eq!(rewritten[1], stage.join("out.o").display().to_string());
        assert_eq!(rewritten[2], "input.rs");
    }

    #[test]
    fn output_rewrite_rejects_unbound_arguments() {
        let root = tempfile::tempdir().unwrap();
        let contracts = vec![WrapperOutputContract {
            staged_relative_path: "out.o".to_string(),
            destination_path: root.path().join("out.o").display().to_string(),
            kind: WrapperArtifactKind::File,
            max_bytes: 1_024,
            executable: false,
        }];
        let error =
            rewrite_output_arguments(&["input.rs".to_string()], &contracts, &root.path().join("stage")).unwrap_err();
        assert!(matches!(error, Error::Policy(_)));
        assert!(error.to_string().contains("not-rewritten"));
    }

    #[test]
    fn failed_multi_output_commit_rolls_back_prior_moves() {
        const OUTPUT_LIMIT_BYTES: u64 = 1_024;
        let root = tempfile::tempdir().unwrap();
        let stage = root.path().join("stage");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("first"), b"first").unwrap();
        let contracts = [
            WrapperOutputContract {
                staged_relative_path: "first".to_string(),
                destination_path: root.path().join("first.out").display().to_string(),
                kind: WrapperArtifactKind::File,
                max_bytes: OUTPUT_LIMIT_BYTES,
                executable: false,
            },
            WrapperOutputContract {
                staged_relative_path: "missing".to_string(),
                destination_path: root.path().join("second.out").display().to_string(),
                kind: WrapperArtifactKind::File,
                max_bytes: OUTPUT_LIMIT_BYTES,
                executable: false,
            },
        ];
        let error = commit_staged_outputs(&stage, &contracts).unwrap_err();
        assert!(error.to_string().contains("commit-staged-output"));
        assert!(stage.join("first").is_file());
        assert!(!root.path().join("first.out").exists());
    }

    #[test]
    fn stale_destination_is_not_clobbered() {
        const OUTPUT_LIMIT_BYTES: u64 = 1_024;
        let root = tempfile::tempdir().unwrap();
        let stage = root.path().join("stage");
        let destination = root.path().join("output.bin");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("output.bin"), b"new-output").unwrap();
        fs::write(&destination, b"cargo-owned-output").unwrap();
        let contract = WrapperOutputContract {
            staged_relative_path: "output.bin".to_string(),
            destination_path: destination.display().to_string(),
            kind: WrapperArtifactKind::File,
            max_bytes: OUTPUT_LIMIT_BYTES,
            executable: false,
        };
        let error = commit_staged_outputs(&stage, &[contract]).expect_err("a stale destination must block commit");
        assert!(error.to_string().contains("destination-exists"));
        assert_eq!(fs::read(&destination).unwrap(), b"cargo-owned-output");
        assert_eq!(fs::read(stage.join("output.bin")).unwrap(), b"new-output");
    }

    #[test]
    fn concurrent_distinct_single_output_commits_do_not_collide() {
        const OUTPUT_LIMIT_BYTES: u64 = 1_024;
        const COMMIT_COUNT: usize = 2;
        let root = tempfile::tempdir().unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(COMMIT_COUNT));
        let mut workers = Vec::with_capacity(COMMIT_COUNT);
        for output_index in 0..COMMIT_COUNT {
            let stage = root.path().join(format!("stage-{output_index}"));
            let destination = root.path().join(format!("output-{output_index}.bin"));
            fs::create_dir(&stage).unwrap();
            fs::write(stage.join("output.bin"), output_index.to_string()).unwrap();
            let worker_barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                let contract = WrapperOutputContract {
                    staged_relative_path: "output.bin".to_string(),
                    destination_path: destination.display().to_string(),
                    kind: WrapperArtifactKind::File,
                    max_bytes: OUTPUT_LIMIT_BYTES,
                    executable: false,
                };
                worker_barrier.wait();
                commit_staged_outputs(&stage, &[contract])
            }));
        }
        for worker in workers {
            worker.join().unwrap().unwrap();
        }
        for output_index in 0..COMMIT_COUNT {
            assert_eq!(
                fs::read_to_string(root.path().join(format!("output-{output_index}.bin"))).unwrap(),
                output_index.to_string()
            );
        }
    }

    #[test]
    fn concurrent_same_destination_commit_has_one_winner_without_clobber() {
        const OUTPUT_LIMIT_BYTES: u64 = 1_024;
        const COMMIT_COUNT: usize = 2;
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("output.bin");
        let barrier = Arc::new(std::sync::Barrier::new(COMMIT_COUNT));
        let mut workers = Vec::with_capacity(COMMIT_COUNT);
        for output_index in 0..COMMIT_COUNT {
            let stage = root.path().join(format!("stage-race-{output_index}"));
            fs::create_dir(&stage).unwrap();
            fs::write(stage.join("output.bin"), output_index.to_string()).unwrap();
            let worker_barrier = Arc::clone(&barrier);
            let worker_destination = destination.clone();
            workers.push(thread::spawn(move || {
                let contract = WrapperOutputContract {
                    staged_relative_path: "output.bin".to_string(),
                    destination_path: worker_destination.display().to_string(),
                    kind: WrapperArtifactKind::File,
                    max_bytes: OUTPUT_LIMIT_BYTES,
                    executable: false,
                };
                worker_barrier.wait();
                commit_staged_outputs(&stage, &[contract])
            }));
        }
        let results = workers.into_iter().map(|worker| worker.join().unwrap()).collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
        let published = fs::read_to_string(destination).unwrap();
        assert!(published == "0" || published == "1");
    }

    #[test]
    fn signing_key_decoder_accepts_lower_hex_and_rejects_upper_hex() {
        let lower = vec![b'a'; SIGNING_KEY_HEX_BYTES];
        let upper = vec![b'A'; SIGNING_KEY_HEX_BYTES];
        assert_eq!(decode_hex(&lower).unwrap().len(), SIGNING_KEY_BYTES);
        assert!(decode_hex(&upper).is_err());
    }

    #[test]
    fn production_bubblewrap_subprocess_clears_ambient_environment_and_writes_only_admitted_root() {
        const ARTIFACT_LIMIT_BYTES: u64 = 1_048_576;
        const EXECUTABLE_LIMIT_BYTES: u64 = crunch_rust_cache_core::wrapper::MAX_WRAPPER_ARTIFACT_BYTES;
        const ELAPSED_LIMIT_MILLIS: u64 = 5_000;
        const STREAM_LIMIT_BYTES: u64 = 4_096;
        const COMPILER_FAILURE_STATUS: i32 = 23;
        const NETWORK_LEAK_STATUS: i32 = 66;
        const EXPECTED_OUTPUT: &[u8] = b"production-bwrap";
        const EXPECTED_STDERR: &[u8] = b"compiler-stderr";
        const EXPECTED_STDOUT: &[u8] = b"compiler-stdout";
        let root = tempfile::tempdir().unwrap();
        let root_text = root.path().to_str().unwrap().to_string();
        let work_root = root.path().join("work");
        let stage_root = root.path().join("stage");
        fs::create_dir(&work_root).unwrap();
        fs::create_dir(&stage_root).unwrap();
        let output_path = stage_root.join("output.bin");
        let output_text = output_path.to_str().unwrap().to_string();
        let undeclared_path = root.path().join("undeclared-secret");
        fs::write(&undeclared_path, b"must-not-be-visible").unwrap();
        let network_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let network_port = network_listener.local_addr().unwrap().port();
        let bwrap_path = find_test_executable(BWRAP_BASENAME);
        let bwrap_text = bwrap_path.to_str().unwrap().to_string();
        let bwrap_digest = hash_file_bounded(&bwrap_path, ARTIFACT_LIMIT_BYTES).unwrap();
        let shell_path = fs::canonicalize(find_test_executable("sh")).unwrap();
        let shell_text = shell_path.to_str().unwrap().to_string();
        let shell_digest = hash_file_bounded(&shell_path, EXECUTABLE_LIMIT_BYTES).unwrap();
        let mut readable_roots = vec!["/nix/store".to_string(), root_text.clone()];
        readable_roots.sort();
        let effects = crunch_rust_cache_core::wrapper::WrapperEffectPolicy {
            filesystem: crunch_rust_cache_core::wrapper::FilesystemEffect::DeclaredRoots,
            network: crunch_rust_cache_core::wrapper::NetworkEffect::Deny,
            clock: crunch_rust_cache_core::wrapper::AmbientEffect::Allow,
            randomness: crunch_rust_cache_core::wrapper::AmbientEffect::Allow,
        };
        let limits = crunch_rust_cache_core::wrapper::WrapperResourceLimits {
            elapsed_millis: ELAPSED_LIMIT_MILLIS,
            stdout_bytes: STREAM_LIMIT_BYTES,
            stderr_bytes: STREAM_LIMIT_BYTES,
            artifact_bytes: ARTIFACT_LIMIT_BYTES,
        };
        let policy = WrapperDaemonPolicy {
            schema: crunch_rust_cache_core::wrapper::WRAPPER_POLICY_SCHEMA.to_string(),
            policy_id: "production-bwrap-test-v1".to_string(),
            cache_mode: crunch_rust_cache_core::wrapper::WrapperCacheMode::Off,
            failure_mode: WrapperFailureMode::FailClosed,
            socket_path: root.path().join("daemon.sock").display().to_string(),
            allowed_peer_uids: vec![unsafe { libc::geteuid() }],
            readable_roots,
            writable_roots: vec![root_text.clone()],
            allowed_environment: vec!["SAFE".to_string()],
            effects,
            limits,
            max_request_bytes: crunch_rust_cache_core::wrapper::MAX_WRAPPER_REQUEST_BYTES,
            max_response_bytes: crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESPONSE_BYTES,
            max_concurrency: 1,
            sandbox_program: bwrap_text,
            sandbox_program_blake3: bwrap_digest.clone(),
            result_sources: Vec::new(),
            shared_trust_policy: None,
            publication: crunch_rust_cache_core::wrapper::WrapperPublicationPolicy {
                enabled: false,
                source_id: None,
                signer_name: None,
                signing_key_path: None,
            },
            local_reads_enabled: false,
            local_writes_enabled: false,
            shared_reads_enabled: false,
            shared_writes_enabled: false,
            redact_environment_values: true,
        };
        validate_wrapper_policy(&policy).unwrap();
        let compiler_script = format!(
            "test \"$SAFE\" = admitted; test -z \"${{FORBIDDEN+x}}\"; test ! -e \"$2\"; if (: >\"/dev/tcp/127.0.0.1/$3\") 2>/dev/null; then exit {NETWORK_LEAK_STATUS}; fi; printf production-bwrap > \"$1\"; printf compiler-stdout; printf compiler-stderr >&2; exit {COMPILER_FAILURE_STATUS}"
        );
        let arguments = vec![
            "-c".to_string(),
            compiler_script,
            "mantle-bwrap-test".to_string(),
            output_text.clone(),
            undeclared_path.display().to_string(),
            network_port.to_string(),
        ];
        let request = seal_wrapper_request(WrapperRequestInput {
            manifest_path: None,
            expected_manifest_ref: None,
            real_compiler_path: shell_text,
            arguments: arguments.clone(),
            admitted_environment: BTreeMap::from([("SAFE".to_string(), "admitted".to_string())]),
            working_directory: work_root.display().to_string(),
            output_contracts: vec![WrapperOutputContract {
                staged_relative_path: "output.bin".to_string(),
                destination_path: output_text,
                kind: WrapperArtifactKind::File,
                max_bytes: ARTIFACT_LIMIT_BYTES,
                executable: false,
            }],
        })
        .unwrap();
        let declared_inputs = vec![
            DeclaredInput {
                role: "compiler".to_string(),
                path: shell_path.display().to_string(),
                kind: DeclaredInputKind::File,
                digest_blake3: shell_digest,
            },
            DeclaredInput {
                role: "sysroot".to_string(),
                path: "/nix/store".to_string(),
                kind: DeclaredInputKind::Directory,
                digest_blake3: bwrap_digest,
            },
        ];
        assert!(enforcement_is_available(&policy));
        let command = sandbox_command(&policy, &request, &declared_inputs, &stage_root, &arguments).unwrap();
        let command_arguments =
            command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert!(command_arguments.iter().any(|argument| argument == "--unshare-net"));
        assert!(
            !command_arguments
                .windows(3)
                .any(|arguments| { arguments[0] == "--ro-bind" && arguments[1] == root.path().to_string_lossy() })
        );
        assert!(
            command_arguments
                .windows(3)
                .any(|arguments| { arguments[0] == "--bind" && arguments[1] == stage_root.to_string_lossy() })
        );
        let options = DaemonOptions {
            policy_path: write_policy_fixture(root.path(), &policy),
            state_dir: root.path().join("state"),
            backend: crunch_store::StoreBackend::Snix,
            store_output_dir: root.path().join("store"),
            receipt_dir: root.path().join("receipts"),
            run_once: true,
        };
        let context = open_daemon_context(&options).unwrap();
        let output = execute_compiler(&context, &request, &declared_inputs, &stage_root, &arguments).unwrap();
        assert_eq!(output.status, COMPILER_FAILURE_STATUS);
        assert!(!output.did_timeout);
        assert_eq!(output.stdout, EXPECTED_STDOUT);
        assert_eq!(output.stderr, EXPECTED_STDERR);
        assert_eq!(fs::read(&output_path).unwrap(), EXPECTED_OUTPUT);
    }

    #[test]
    fn crunch_rust_cache_core_adoption_compiles_then_reuses_local_castore_result() {
        const INPUT_BYTES: &[u8] = include_bytes!("../../crunch-rust-cache-core/Cargo.toml");
        const ARTIFACT_LIMIT_BYTES: u64 = 16_777_216;
        const STREAM_LIMIT_BYTES: u64 = 4_096;
        const ELAPSED_LIMIT_MILLIS: u64 = 5_000;
        const STARTUP_BASELINE_MILLIS: u128 = 5_000;
        const LOCAL_HIT_BASELINE_MILLIS: u128 = 2_000;
        const BENCHMARK_SAMPLE_COUNT: usize = 5;
        const BENCHMARK_CONCURRENCY: u32 = 1;
        const SIGNING_KEY_BYTE: u8 = 7;
        let root = tempfile::tempdir().unwrap();
        let input_path = root.path().join("input.bin");
        let output_path = root.path().join("output.bin");
        let sandbox_path = root.path().join("sandbox-adapter");
        let manifest_path = root.path().join("manifest.json");
        fs::write(&input_path, INPUT_BYTES).unwrap();
        fs::write(&sandbox_path, b"#!/bin/sh\ntest \"$1\" = \"--\" || exit 64\nshift\nexec \"$@\"\n").unwrap();
        fs::set_permissions(&sandbox_path, fs::Permissions::from_mode(0o700)).unwrap();
        let compiler_path = root.path().join("cp");
        fs::copy(find_test_executable("cp"), &compiler_path).unwrap();
        fs::set_permissions(&compiler_path, fs::Permissions::from_mode(0o700)).unwrap();
        let compiler_digest =
            hash_file_bounded(&compiler_path, crunch_rust_cache_core::wrapper::MAX_WRAPPER_ARTIFACT_BYTES).unwrap();
        let source_digest = hash_file_bounded(&input_path, ARTIFACT_LIMIT_BYTES).unwrap();
        let sandbox_digest = hash_file_bounded(&sandbox_path, ARTIFACT_LIMIT_BYTES).unwrap();
        let root_text = root.path().to_str().unwrap().to_string();
        let compiler_text = compiler_path.to_str().unwrap().to_string();
        let input_text = input_path.to_str().unwrap().to_string();
        let output_text = output_path.to_str().unwrap().to_string();
        let sandbox_text = sandbox_path.to_str().unwrap().to_string();
        let readable_roots = vec![root_text.clone()];
        let action = crunch_rust_cache_core::canonical_rust_action(crunch_rust_cache_core::RustUnitActionInput {
            unit_id: "unit-daemon-test".to_string(),
            package_id: "crunch-rust-cache-core-0.1.0".to_string(),
            crate_name: "crunch_rust_cache_core".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "test".to_string(),
            mode: "build".to_string(),
            features: Vec::new(),
            source_digest_blake3: source_digest.clone(),
            compiler_digest_blake3: compiler_digest.clone(),
            compiler_version_digest_blake3: compiler_digest.clone(),
            toolchain_closure_digest_blake3: compiler_digest.clone(),
            execution_platform_digest_blake3: compiler_digest.clone(),
            semantic_arguments: vec![crunch_rust_cache_core::RustSemanticArgument {
                value: "copy-declared-input".to_string(),
                contains_absolute_path: false,
                absolute_paths_classified: false,
            }],
            admitted_environment: BTreeMap::new(),
            dependency_artifacts: Vec::new(),
            host_artifacts: Vec::new(),
            build_script_facts: Vec::new(),
            native_link_facts: Vec::new(),
            compiler_policy_digest_blake3: sandbox_digest.clone(),
        })
        .unwrap();
        let limits = crunch_rust_cache_core::wrapper::WrapperResourceLimits {
            elapsed_millis: ELAPSED_LIMIT_MILLIS,
            stdout_bytes: STREAM_LIMIT_BYTES,
            stderr_bytes: STREAM_LIMIT_BYTES,
            artifact_bytes: ARTIFACT_LIMIT_BYTES,
        };
        let effects = crunch_rust_cache_core::wrapper::WrapperEffectPolicy {
            filesystem: crunch_rust_cache_core::wrapper::FilesystemEffect::DeclaredRoots,
            network: crunch_rust_cache_core::wrapper::NetworkEffect::Deny,
            clock: crunch_rust_cache_core::wrapper::AmbientEffect::Allow,
            randomness: crunch_rust_cache_core::wrapper::AmbientEffect::Allow,
        };
        let policy = WrapperDaemonPolicy {
            schema: crunch_rust_cache_core::wrapper::WRAPPER_POLICY_SCHEMA.to_string(),
            policy_id: "daemon-test-policy-v1".to_string(),
            cache_mode: crunch_rust_cache_core::wrapper::WrapperCacheMode::LocalReadWrite,
            failure_mode: WrapperFailureMode::FailClosed,
            socket_path: root.path().join("daemon.sock").display().to_string(),
            allowed_peer_uids: vec![unsafe { libc::geteuid() }],
            readable_roots,
            writable_roots: vec![root_text.clone()],
            allowed_environment: Vec::new(),
            effects: effects.clone(),
            limits: limits.clone(),
            max_request_bytes: crunch_rust_cache_core::wrapper::MAX_WRAPPER_REQUEST_BYTES,
            max_response_bytes: crunch_rust_cache_core::wrapper::MAX_WRAPPER_RESPONSE_BYTES,
            max_concurrency: 1,
            sandbox_program: sandbox_text,
            sandbox_program_blake3: sandbox_digest,
            result_sources: Vec::new(),
            shared_trust_policy: None,
            publication: crunch_rust_cache_core::wrapper::WrapperPublicationPolicy {
                enabled: false,
                source_id: None,
                signer_name: None,
                signing_key_path: None,
            },
            local_reads_enabled: true,
            local_writes_enabled: true,
            shared_reads_enabled: false,
            shared_writes_enabled: false,
            redact_environment_values: true,
        };
        validate_wrapper_policy(&policy).unwrap();
        let arguments = vec![input_text.clone(), output_text.clone()];
        let output_contracts = vec![WrapperOutputContract {
            staged_relative_path: "output.bin".to_string(),
            destination_path: output_text,
            kind: WrapperArtifactKind::File,
            max_bytes: ARTIFACT_LIMIT_BYTES,
            executable: false,
        }];
        let manifest = seal_wrapper_manifest(WrapperInvocationManifestInput {
            policy_id: policy.policy_id.clone(),
            action,
            real_compiler_path: compiler_text.clone(),
            arguments_blake3: arguments_identity_blake3(&arguments).unwrap(),
            environment_blake3: environment_identity_blake3(&BTreeMap::new()).unwrap(),
            working_directory: root_text.clone(),
            readable_roots: policy.readable_roots.clone(),
            writable_roots: policy.writable_roots.clone(),
            declared_inputs: vec![
                DeclaredInput {
                    role: "compiler".to_string(),
                    path: compiler_text.clone(),
                    kind: DeclaredInputKind::File,
                    digest_blake3: compiler_digest.clone(),
                },
                DeclaredInput {
                    role: "source".to_string(),
                    path: input_text.clone(),
                    kind: DeclaredInputKind::File,
                    digest_blake3: source_digest.clone(),
                },
                DeclaredInput {
                    role: "sysroot".to_string(),
                    path: compiler_text.clone(),
                    kind: DeclaredInputKind::File,
                    digest_blake3: compiler_digest,
                },
            ],
            output_contracts: output_contracts.clone(),
            effects,
            limits,
        })
        .unwrap();
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let request = seal_wrapper_request(WrapperRequestInput {
            manifest_path: Some(manifest_path.display().to_string()),
            expected_manifest_ref: Some(manifest.manifest_ref.clone()),
            real_compiler_path: compiler_text,
            arguments,
            admitted_environment: BTreeMap::new(),
            working_directory: root_text,
            output_contracts,
        })
        .unwrap();
        let options = DaemonOptions {
            policy_path: write_policy_fixture(root.path(), &policy),
            state_dir: root.path().join("state"),
            backend: crunch_store::StoreBackend::Snix,
            store_output_dir: root.path().join("store"),
            receipt_dir: root.path().join("receipts"),
            run_once: true,
        };
        run_test_daemon_shutdown(options.clone(), &policy);
        run_test_daemon_rejected_connection(options.clone(), &policy, &[0_u8; 4]);
        let mut invalid_json = (4_u64).to_be_bytes().to_vec();
        invalid_json.extend_from_slice(b"bad!");
        run_test_daemon_rejected_connection(options.clone(), &policy, &invalid_json);
        let startup_started = std::time::Instant::now();
        let first = run_test_daemon_request_observed(options.clone(), policy.clone(), request.clone());
        let startup_elapsed = startup_started.elapsed();
        assert_eq!(first.disposition, WrapperDisposition::Compiled);
        assert_eq!(first.compiler_status, 0, "{}", String::from_utf8_lossy(&first.stderr));
        assert_eq!(fs::read(&output_path).unwrap(), INPUT_BYTES);
        assert!(first.receipt_ref.is_some());
        assert!(startup_elapsed.as_millis() <= STARTUP_BASELINE_MILLIS);
        let cold_samples = vec![startup_elapsed.as_micros()];

        let mut local_hit_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for _ in 0..BENCHMARK_SAMPLE_COUNT {
            fs::remove_file(&output_path).unwrap();
            // The local-hit baseline covers the request round trip to a ready daemon; daemon
            // startup is bounded separately by STARTUP_BASELINE_MILLIS above.
            let (response, hit_elapsed) =
                run_test_daemon_request_timed(options.clone(), policy.clone(), request.clone());
            assert_eq!(response.disposition, WrapperDisposition::LocalHit);
            assert!(response.receipt_ref.is_some());
            assert_eq!(fs::read(&output_path).unwrap(), INPUT_BYTES);
            assert!(hit_elapsed.as_millis() <= LOCAL_HIT_BASELINE_MILLIS);
            local_hit_samples.push(hit_elapsed.as_micros());
        }

        let mut bypass_input = request.input.clone();
        bypass_input.manifest_path = None;
        bypass_input.expected_manifest_ref = None;
        bypass_input.arguments = vec!["--version".to_string()];
        let bypass_request = seal_wrapper_request(bypass_input).unwrap();
        let mut round_trip_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for _ in 0..BENCHMARK_SAMPLE_COUNT {
            let (response, round_trip_elapsed) =
                run_test_daemon_request_timed(options.clone(), policy.clone(), bypass_request.clone());
            round_trip_samples.push(round_trip_elapsed.as_micros());
            assert_eq!(response.disposition, WrapperDisposition::Bypass);
            assert_eq!(response.bypass_class, Some(WrapperBypassClass::CompilerQuery));
        }

        let pass_through_compiler = find_test_executable("true").into_os_string();
        let mut pass_through_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for _ in 0..BENCHMARK_SAMPLE_COUNT {
            let pass_through_started = std::time::Instant::now();
            let status = run_direct_compiler_strings(&pass_through_compiler, &[]).unwrap();
            pass_through_samples.push(pass_through_started.elapsed().as_micros());
            assert_eq!(status, 0);
        }

        let shared_root = root.path().join("shared");
        let shared_source = Arc::new(DirectoryRustResultSource::open(shared_root.clone(), true).unwrap());
        let producer_context = open_daemon_context(&options).unwrap();
        let seed_output = root.path().join("shared-seed");
        fs::create_dir(&seed_output).unwrap();
        fs::write(seed_output.join("output.bin"), INPUT_BYTES).unwrap();
        let producer_receipt_ref = first.receipt_ref.as_deref().unwrap();
        let result = producer_context
            .cache
            .publish_blocking(PublishRequest {
                action: &manifest.input.action,
                output_dir: &seed_output,
                producer_receipt_ref,
                policy: &producer_context.local_policy,
            })
            .unwrap();
        let signing_key = SigningKey::from_bytes(&[SIGNING_KEY_BYTE; SIGNING_KEY_BYTES]);
        let producer_policy_id = "daemon-shared-producer-policy-v1";
        let signer_name = "daemon-shared-test-key";
        let shared_policy = SharedRustCachePolicy {
            reads_enabled: true,
            publishes_enabled: true,
            max_sources: 1,
            ..SharedRustCachePolicy::default()
        };
        producer_context
            .cache
            .publish_shared_blocking(shared_source.as_ref(), SharedPublishRequest {
                result: &result,
                producer: RustResultProducerIdentity {
                    producer_id: "daemon-shared-test-producer".to_string(),
                    producer_policy_id: producer_policy_id.to_string(),
                },
                signer_name: signer_name.to_string(),
                signing_key: &signing_key,
                policy: &shared_policy,
            })
            .unwrap();

        let mut remote_policy = policy.clone();
        remote_policy.policy_id = "daemon-remote-read-test-v1".to_string();
        remote_policy.cache_mode = crunch_rust_cache_core::wrapper::WrapperCacheMode::SharedRead;
        remote_policy.local_writes_enabled = false;
        remote_policy.shared_reads_enabled = true;
        remote_policy.result_sources = vec![crunch_rust_cache_core::wrapper::WrapperResultSourcePolicy {
            priority: 0,
            source_id: shared_source.source_id().to_string(),
            kind: crunch_rust_cache_core::wrapper::WrapperResultSourceKind::Directory,
            endpoint: shared_root.display().to_string(),
        }];
        remote_policy.shared_trust_policy = Some(crunch_rust_cache_core::shared::RustResultTrustPolicy {
            schema: crunch_rust_cache_core::shared::SHARED_RUST_TRUST_POLICY_SCHEMA.to_string(),
            policy_id: "daemon-shared-test-trust-v1".to_string(),
            accepted_producer_policy_ids: vec![producer_policy_id.to_string()],
            trusted_keys: vec![crunch_rust_cache_core::shared::TrustedRustResultKey {
                signer_name: signer_name.to_string(),
                verifier_key_hex: data_encoding::HEXLOWER.encode(signing_key.verifying_key().as_bytes()),
            }],
        });
        validate_wrapper_policy(&remote_policy).unwrap();
        let mut remote_manifest_input = manifest.input.clone();
        remote_manifest_input.policy_id = remote_policy.policy_id.clone();
        let remote_manifest = seal_wrapper_manifest(remote_manifest_input).unwrap();
        fs::write(&manifest_path, serde_json::to_vec(&remote_manifest).unwrap()).unwrap();
        let mut remote_request_input = request.input.clone();
        remote_request_input.expected_manifest_ref = Some(remote_manifest.manifest_ref.clone());
        let remote_request = seal_wrapper_request(remote_request_input).unwrap();
        let remote_policy_path = write_policy_fixture(root.path(), &remote_policy);
        let mut remote_hit_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            fs::remove_file(&output_path).unwrap();
            let remote_options = DaemonOptions {
                policy_path: remote_policy_path.clone(),
                state_dir: root.path().join(format!("remote-state-{sample_index}")),
                backend: crunch_store::StoreBackend::Snix,
                store_output_dir: root.path().join(format!("remote-store-{sample_index}")),
                receipt_dir: root.path().join(format!("remote-receipts-{sample_index}")),
                run_once: true,
            };
            let remote_started = std::time::Instant::now();
            let remote = run_test_daemon_request(remote_options, remote_policy.clone(), remote_request.clone());
            let remote_elapsed = remote_started.elapsed();
            assert_eq!(remote.disposition, WrapperDisposition::SharedHit);
            assert_eq!(fs::read(&output_path).unwrap(), INPUT_BYTES);
            assert!(remote.receipt_ref.is_some());
            assert!(remote_elapsed.as_millis() <= STARTUP_BASELINE_MILLIS);
            remote_hit_samples.push(remote_elapsed.as_micros());
        }

        let artifact_bytes = u64::try_from(INPUT_BYTES.len()).unwrap();
        let cold = latency_summary(&cold_samples).unwrap();
        let local = latency_summary(&local_hit_samples).unwrap();
        let remote = latency_summary(&remote_hit_samples).unwrap();
        let round_trip = latency_summary(&round_trip_samples).unwrap();
        let pass_through = latency_summary(&pass_through_samples).unwrap();
        emit_benchmark_metric(
            "cold-miss",
            &cold_samples,
            cold,
            artifact_bytes,
            BENCHMARK_CONCURRENCY,
            ELAPSED_LIMIT_MILLIS,
        );
        emit_benchmark_metric(
            "local-hit-restoration",
            &local_hit_samples,
            local,
            artifact_bytes,
            BENCHMARK_CONCURRENCY,
            ELAPSED_LIMIT_MILLIS,
        );
        emit_benchmark_metric(
            "shared-remote-hit-restoration",
            &remote_hit_samples,
            remote,
            artifact_bytes,
            BENCHMARK_CONCURRENCY,
            ELAPSED_LIMIT_MILLIS,
        );
        emit_benchmark_metric(
            "daemon-round-trip",
            &round_trip_samples,
            round_trip,
            artifact_bytes,
            BENCHMARK_CONCURRENCY,
            ELAPSED_LIMIT_MILLIS,
        );
        emit_benchmark_metric(
            "pass-through",
            &pass_through_samples,
            pass_through,
            0,
            BENCHMARK_CONCURRENCY,
            ELAPSED_LIMIT_MILLIS,
        );
        eprintln!(
            "daemon-wrapper-benchmark-v1:workload=cold-miss-overhead:median_us={}:p95_us={}",
            cold.median_micros.saturating_sub(pass_through.median_micros),
            cold.p95_micros.saturating_sub(pass_through.p95_micros),
        );
        assert!(crunch_store::StoreBackend::Casita.profile().rust_unit_cache);
        if crunch_store::StoreBackend::Casita.profile().rust_unit_cache {
            let mut casita_policy = policy.clone();
            casita_policy.failure_mode = WrapperFailureMode::FailOpen;
            validate_wrapper_policy(&casita_policy).unwrap();
            fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
            let casita_options = DaemonOptions {
                policy_path: write_policy_fixture(root.path(), &casita_policy),
                state_dir: root.path().join("casita-state"),
                backend: crunch_store::StoreBackend::Casita,
                store_output_dir: root.path().join("casita-store"),
                receipt_dir: root.path().join("casita-receipts"),
                run_once: true,
            };
            fs::remove_file(&output_path).unwrap();
            let compiled = run_test_daemon_request(casita_options.clone(), casita_policy.clone(), request.clone());
            assert_eq!(compiled.disposition, WrapperDisposition::Compiled);
            assert_eq!(fs::read(&output_path).unwrap(), INPUT_BYTES);
            fs::remove_file(&output_path).unwrap();
            let reused = run_test_daemon_request(casita_options.clone(), casita_policy.clone(), request.clone());
            assert_eq!(reused.disposition, WrapperDisposition::LocalHit);
            assert_eq!(fs::read(&output_path).unwrap(), INPUT_BYTES);
            fs::remove_file(&output_path).unwrap();

            // Deliberate test-only misuse: raw guarded GC omits the Rust cache's retained payload.
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async {
                let guard = crunch_store::StoreMutationGuard::acquire_wait(&casita_options.state_dir).unwrap();
                let mut store = crunch_store::StoreHandle::open(StoreConfig::new(
                    crunch_store::StoreBackend::Casita,
                    casita_options.state_dir.clone(),
                    casita_options.store_output_dir.clone(),
                    STORE_PREFIX.to_string(),
                ))
                .await
                .unwrap();
                store.recover_casita_gc_under_guard(&guard).await.unwrap();
                let plan = store.garbage_collect_under_guard(&guard, None).await.unwrap();
                assert!(
                    store.garbage_collect_under_guard(&guard, Some(&plan.plan_id)).await.unwrap().execution_complete
                );
            });
            let refused = run_test_daemon_request(casita_options.clone(), casita_policy.clone(), request.clone());
            assert_eq!(refused.disposition, WrapperDisposition::Rejected, "{refused:?}");
            assert_eq!(refused.compiler_status, EXIT_PROTOCOL_FAILURE);
            assert_eq!(refused.reason_codes, vec![CASITA_ROOT_MISSING.to_string()]);
            assert_eq!(refused.stderr, format!("mantle-rustc-wrapper:rejected:{CASITA_ROOT_MISSING}\n").into_bytes());
            assert!(refused.receipt_ref.is_some());
            let receipt_digest = refused.receipt_ref.as_deref().unwrap().rsplit('/').next().unwrap();
            let receipt: WrapperReceipt = serde_json::from_slice(
                &fs::read(casita_options.receipt_dir.join(format!("{receipt_digest}{RECEIPT_SUFFIX}"))).unwrap(),
            )
            .unwrap();
            assert_eq!(receipt.input.disposition, WrapperDisposition::Rejected);
            assert_eq!(receipt.input.reason_codes, vec![CASITA_ROOT_MISSING.to_string()]);
            assert!(!receipt.input.compiler_executed);
            assert!(!receipt.input.artifact_commit_complete);
            let compiler = compiler_path.into_os_string();
            assert_eq!(
                finish_client_response(&casita_policy, &compiler, &request, refused).unwrap(),
                EXIT_PROTOCOL_FAILURE,
            );
            assert!(!output_path.exists(), "FailOpen must never execute cp after Casita payload integrity failure");
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct LatencySummary {
        median_micros: u128,
        p95_micros: u128,
    }

    fn latency_summary(samples: &[u128]) -> Option<LatencySummary> {
        const P95_PERCENT: usize = 95;
        const PERCENT_SCALE: usize = 100;
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let median_micros = ordered.get(ordered.len().checked_div(2)?)?;
        let rank = ordered
            .len()
            .checked_mul(P95_PERCENT)?
            .checked_add(PERCENT_SCALE.checked_sub(1)?)?
            .checked_div(PERCENT_SCALE)?;
        let p95_index = rank.checked_sub(1)?;
        let p95_micros = ordered.get(p95_index)?;
        Some(LatencySummary {
            median_micros: *median_micros,
            p95_micros: *p95_micros,
        })
    }

    fn emit_benchmark_metric(
        workload: &str,
        samples: &[u128],
        summary: LatencySummary,
        artifact_bytes: u64,
        concurrency: u32,
        timeout_millis: u64,
    ) {
        assert!(!workload.is_empty());
        assert!(!samples.is_empty());
        eprintln!(
            "daemon-wrapper-benchmark-v1:workload={workload}:samples={}:concurrency={concurrency}:artifact_bytes={artifact_bytes}:timeout_ms={timeout_millis}:median_us={}:p95_us={}",
            samples.len(),
            summary.median_micros,
            summary.p95_micros,
        );
    }

    #[test]
    fn latency_summary_handles_bounded_samples_and_empty_input() {
        const SAMPLES: &[u128] = &[10, 40, 20, 50, 30];
        const EXPECTED_MEDIAN_MICROS: u128 = 30;
        const EXPECTED_P95_MICROS: u128 = 50;
        let summary = latency_summary(SAMPLES).unwrap();
        assert_eq!(summary.median_micros, EXPECTED_MEDIAN_MICROS);
        assert_eq!(summary.p95_micros, EXPECTED_P95_MICROS);
        assert_eq!(latency_summary(&[]), None);
    }

    fn run_test_daemon_request_observed(
        options: DaemonOptions,
        policy: WrapperDaemonPolicy,
        request: WrapperRequest,
    ) -> WrapperResponse {
        let started = Arc::new(AtomicBool::new(false));
        let flushed = Arc::new(AtomicBool::new(false));
        let started_observer = Arc::clone(&started);
        let flushed_observer = Arc::clone(&flushed);
        let daemon = thread::spawn(move || {
            run_daemon_with_response_observer(
                options,
                Arc::new(move || started_observer.store(true, Ordering::SeqCst)),
                Arc::new(move || flushed_observer.store(true, Ordering::SeqCst)),
            )
        });
        wait_for_test_socket(&policy.socket_path, &daemon);
        for _ in 0..100 {
            if started.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(started.load(Ordering::SeqCst), "socket listener must actually start");
        assert!(!flushed.load(Ordering::SeqCst), "socket startup is not a valid request/response");
        let response = request_daemon(&policy, &request).unwrap();
        daemon.join().unwrap().unwrap();
        assert!(flushed.load(Ordering::SeqCst), "only a successful Rust response flush acknowledges readiness");
        assert_eq!(response.request_ref, request.request_ref);
        response
    }

    fn run_test_daemon_request(
        options: DaemonOptions,
        policy: WrapperDaemonPolicy,
        request: WrapperRequest,
    ) -> WrapperResponse {
        run_test_daemon_request_timed(options, policy, request).0
    }

    /// Returns the response and the elapsed time of the request round trip, measured only
    /// after the daemon has bound its socket.
    fn run_test_daemon_request_timed(
        options: DaemonOptions,
        policy: WrapperDaemonPolicy,
        request: WrapperRequest,
    ) -> (WrapperResponse, Duration) {
        let daemon = thread::spawn(move || run_daemon(options));
        wait_for_test_socket(&policy.socket_path, &daemon);
        let request_started = std::time::Instant::now();
        let response = request_daemon(&policy, &request).unwrap();
        let request_elapsed = request_started.elapsed();
        daemon.join().unwrap().unwrap();
        assert_eq!(response.request_ref, request.request_ref);
        (response, request_elapsed)
    }

    fn run_test_daemon_shutdown(mut options: DaemonOptions, policy: &WrapperDaemonPolicy) {
        options.run_once = false;
        let daemon = thread::spawn(move || run_daemon(options));
        wait_for_test_socket(&policy.socket_path, &daemon);
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        daemon.join().unwrap().unwrap();
        assert!(!path_entry_exists(Path::new(&policy.socket_path)));
    }

    fn run_test_daemon_rejected_connection(options: DaemonOptions, policy: &WrapperDaemonPolicy, payload: &[u8]) {
        let flushed = Arc::new(AtomicBool::new(false));
        let flushed_observer = Arc::clone(&flushed);
        let daemon = thread::spawn(move || {
            run_daemon_with_response_observer(
                options,
                Arc::new(|| {}),
                Arc::new(move || flushed_observer.store(true, Ordering::SeqCst)),
            )
        });
        wait_for_test_socket(&policy.socket_path, &daemon);
        let mut stream = UnixStream::connect(&policy.socket_path).unwrap();
        stream.write_all(payload).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        daemon.join().unwrap().unwrap();
        assert!(!flushed.load(Ordering::SeqCst), "incomplete or invalid frames cannot acknowledge readiness");
        assert!(!path_entry_exists(Path::new(&policy.socket_path)));
    }

    /// Waits until the daemon has bound its socket. Daemon startup opens the store and the
    /// Rust cache, which takes well under a second on an idle host but several seconds on a
    /// saturated shared builder, so the bound is generous; a daemon that exits before binding
    /// fails immediately with its own error instead of a timeout.
    fn wait_for_test_socket(socket_path: &str, daemon: &thread::JoinHandle<Result<(), Error>>) {
        const SOCKET_READY_DEADLINE: Duration = Duration::from_secs(60);
        const SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(5);
        let started = std::time::Instant::now();
        loop {
            if path_entry_exists(Path::new(socket_path)) {
                return;
            }
            assert!(!daemon.is_finished(), "daemon exited before binding {socket_path}");
            let waited = started.elapsed();
            assert!(
                waited < SOCKET_READY_DEADLINE,
                "daemon socket {socket_path} did not become ready within {} ms",
                waited.as_millis()
            );
            thread::sleep(SOCKET_POLL_INTERVAL);
        }
    }

    fn find_test_executable(name: &str) -> PathBuf {
        let path = std::env::var_os("PATH").unwrap();
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join(name);
            if candidate.is_file() {
                assert!(!candidate.as_os_str().is_empty());
                return candidate;
            }
        }
        panic!("test executable is absent: {name}");
    }

    fn write_policy_fixture(root: &Path, policy: &WrapperDaemonPolicy) -> PathBuf {
        let path = root.join("policy.json");
        fs::write(&path, serde_json::to_vec(policy).unwrap()).unwrap();
        assert!(path.is_file());
        path
    }
}
