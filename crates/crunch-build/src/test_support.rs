//! Shared test helpers for crunch-build tests.
//!
//! Provides MockBuildService, build_and_register, and factory functions
//! for in-memory blob/directory/pathinfo services.
#![cfg(test)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Read as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::thread::JoinHandle;

use async_trait::async_trait;
use futures::StreamExt as _;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildOutput;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice;
use snix_store::pathinfoservice::LruPathInfoService;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncWriteExt;

use crate::registry::DerivationRegistry;
use crate::signing;

/// Create a temporary in-memory directory service.
pub fn tmp_ds() -> RedbDirectoryService {
    RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
}

/// Test source admission for store sources that the operator already selected.
///
/// The store refuses a substituter or overlay base without an admission
/// authority, so tests that open a store with a remote cache URL or a base
/// layer install this implementation instead of weakening that boundary.
#[derive(Debug)]
pub struct TestSourceAdmission;

impl crunch_store::StoreSourceLease for TestSourceAdmission {}

impl crunch_store::StoreSourceAdmission for TestSourceAdmission {
    fn admit(
        &self,
        _source: crunch_store::StoreSource<'_>,
        _store_prefix: &str,
    ) -> Result<Arc<dyn crunch_store::StoreSourceLease>, crunch_store::Error> {
        Ok(Arc::new(TestSourceAdmission))
    }
}

/// A source admission authority for one store configuration.
pub fn test_source_admission() -> Arc<dyn crunch_store::StoreSourceAdmission> {
    Arc::new(TestSourceAdmission)
}

/// A test keypair (same as nix-compat's DUMMY_KEYPAIR).
pub fn test_keypair() -> signing::KeyPair {
    signing::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap()
}

/// Default trusted keys for tests (local + cache.nixos.org-1).
pub fn test_trusted_keys() -> Vec<VerifyingKey> {
    signing::build_trusted_keys(&test_keypair(), None)
}

/// Create an in-memory LRU path info service.
pub fn test_pis() -> LruPathInfoService {
    const TEST_PATH_INFO_CAPACITY: usize = 128;
    let capacity = std::num::NonZeroUsize::new(TEST_PATH_INFO_CAPACITY).expect("test capacity must be positive");
    LruPathInfoService::with_capacity("test".to_string(), capacity)
}

pub fn store_handle<BS, DS>(blob_service: BS, directory_service: DS) -> crunch_store::StoreHandle
where
    BS: BlobService + 'static,
    DS: DirectoryService + 'static,
{
    const TEST_STORE_DIR: &str = "/nix/store";
    crunch_store::StoreHandle::from_services_with_store_dir(
        crunch_store::StoreBackend::Snix,
        crunch_store::StoreHandleServices {
            blob_service: Arc::new(blob_service),
            directory_service: Arc::new(directory_service),
            pathinfo_service: Arc::new(test_pis()) as Arc<dyn PathInfoService>,
            remote_pathinfo: None,
            state_dir: std::path::PathBuf::from("/tmp/mantle-store-authority-test"),
            output_dir_str: TEST_STORE_DIR.to_string(),
            publishers: Vec::new(),
        },
        TEST_STORE_DIR.to_string(),
    )
    .unwrap()
}

pub fn pipeline_store_parts<BS, DS>(blob_service: BS, directory_service: DS) -> crunch_store::PipelineStoreParts
where
    BS: BlobService + 'static,
    DS: DirectoryService + 'static,
{
    store_handle(blob_service, directory_service).into_pipeline_store_parts()
}

/// A mock BuildService that records command_args from each request and
/// returns a synthetic output node for each requested output.
pub struct MockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
}

impl MockBuildService {
    pub fn new(blob_service: MemoryBlobService) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for MockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(b"mock output").await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest,
                    size: 11,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult {
            outputs,
            log: None,
            timing: None,
        })
    }
}

/// Build a nix_compat::Derivation, compute paths, register in DerivationRegistry.
///
/// `input_drvs`: `&[(parent_store_path, output_name)]` — parents must
/// already be registered in `kp`.
pub fn build_and_register(
    name: &str,
    input_drvs: &[(StorePath<String>, &str)],
    kp: &mut DerivationRegistry,
) -> (StorePath<String>, Derivation) {
    let mut outputs = BTreeMap::new();
    outputs.insert("out".to_string(), Output {
        path: None,
        ca_hash: None,
    });
    let mut environment = BTreeMap::new();
    environment.insert("name".to_string(), name.into());
    environment.insert("system".to_string(), "x86_64-linux".into());
    environment.insert("builder".to_string(), "/bin/sh".into());
    environment.insert("out".to_string(), "".into());

    let mut input_derivations = BTreeMap::new();
    for (dp, on) in input_drvs {
        input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
    }

    let mut drv = Derivation {
        arguments: vec!["-c".into(), format!("echo {name} > $out")],
        builder: "/bin/sh".to_string(),
        environment,
        input_derivations,
        input_sources: BTreeSet::new(),
        outputs,
        system: "x86_64-linux".to_string(),
    };

    let hdm = drv.hash_derivation_modulo(|parent_path| {
        kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
    });
    drv.calculate_output_paths(name, &hdm).unwrap();
    let drv_path = drv.calculate_derivation_path(name).unwrap();

    let mut fake_hash = [0u8; 32];
    for (i, b) in name.bytes().enumerate().take(32) {
        fake_hash[i] = b;
    }
    kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

    (drv_path, drv)
}

/// Like `build_and_register` but with multiple outputs.
///
/// Creates a derivation with the given output names (e.g., `["out", "dev", "lib"]`).
/// Each output gets a distinct store path.
pub fn build_and_register_multi(
    name: &str,
    output_names: &[&str],
    input_drvs: &[(StorePath<String>, &str)],
    kp: &mut DerivationRegistry,
) -> (StorePath<String>, Derivation) {
    assert!(!output_names.is_empty(), "must have at least one output");

    let mut outputs = BTreeMap::new();
    let mut environment = BTreeMap::new();
    environment.insert("name".to_string(), name.into());
    environment.insert("system".to_string(), "x86_64-linux".into());
    environment.insert("builder".to_string(), "/bin/sh".into());
    environment.insert("outputs".to_string(), output_names.join(" ").into());

    for on in output_names {
        outputs.insert(on.to_string(), Output {
            path: None,
            ca_hash: None,
        });
        environment.insert(on.to_string(), "".into());
    }

    let mut input_derivations = BTreeMap::new();
    for (dp, on) in input_drvs {
        input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
    }

    let mut drv = Derivation {
        arguments: vec!["-c".into(), format!("echo {name} > $out")],
        builder: "/bin/sh".to_string(),
        environment,
        input_derivations,
        input_sources: BTreeSet::new(),
        outputs,
        system: "x86_64-linux".to_string(),
    };

    let hdm = drv.hash_derivation_modulo(|parent_path| {
        kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
    });
    drv.calculate_output_paths(name, &hdm).unwrap();
    let drv_path = drv.calculate_derivation_path(name).unwrap();

    let mut fake_hash = [0u8; 32];
    for (i, b) in name.bytes().enumerate().take(32) {
        fake_hash[i] = b;
    }
    kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

    (drv_path, drv)
}

/// Like `build_and_register` but names the output with `.drv` suffix
/// so the dynamic derivation detector picks it up.
#[allow(dead_code)]
pub fn build_and_register_producer(name: &str, kp: &mut DerivationRegistry) -> (StorePath<String>, Derivation) {
    // Producer name ends with .drv so output path triggers detection.
    let producer_name = format!("{name}.drv");
    build_and_register(&producer_name, &[], kp)
}

/// A mock BuildService that produces `.drv` ATerm content for builds
/// whose name contains a specific marker. All other builds get normal
/// mock output.
///
/// The "inner" derivation ATerm bytes are provided at construction.
pub struct DrvProducingMockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
    /// Map from name substring → ATerm bytes to produce as output.
    drv_outputs: HashMap<String, Vec<u8>>,
}

impl DrvProducingMockBuildService {
    pub fn new(
        blob_service: MemoryBlobService,
        drv_outputs: HashMap<String, Vec<u8>>,
    ) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
                drv_outputs,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for DrvProducingMockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        // Check if any name substring matches this request's args.
        let content: Vec<u8> = self
            .drv_outputs
            .iter()
            .find(|(marker, _)| request.command_args.iter().any(|a| a.contains(marker.as_str())))
            .map(|(_, aterm)| aterm.clone())
            .unwrap_or_else(|| b"mock output".to_vec());

        let size = content.len() as u64;
        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(&content).await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest,
                    size,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult {
            outputs,
            log: None,
            timing: None,
        })
    }
}

/// A mock BuildService that fails for builds whose command_args
/// contain a specific substring. All other builds succeed normally.
pub struct FailingMockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
    /// Substrings that trigger a build failure.
    fail_markers: Vec<String>,
}

impl FailingMockBuildService {
    pub fn new(blob_service: MemoryBlobService, fail_markers: Vec<String>) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                blob_service,
                fail_markers,
            },
            calls,
        )
    }
}

#[async_trait]
#[async_trait]
impl BuildService for FailingMockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls.lock().unwrap().push(request.command_args.clone());

        // Check if this build should fail.
        let should_fail = self
            .fail_markers
            .iter()
            .any(|marker| request.command_args.iter().any(|a| a.contains(marker.as_str())));

        if should_fail {
            return Err(std::io::Error::other("simulated build failure"));
        }

        let mut writer = BlobService::open_write(&self.blob_service).await;
        writer.write_all(b"mock output").await.unwrap();
        let digest = writer.close().await.unwrap();

        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: Node::File {
                    digest,
                    size: 11,
                    executable: false,
                },
                output_needles: BTreeSet::new(),
            })
            .collect();

        Ok(BuildResult {
            outputs,
            log: None,
            timing: None,
        })
    }
}

pub struct PanicBuildService;

#[async_trait]
impl BuildService for PanicBuildService {
    async fn do_build(&self, _request: BuildRequest) -> std::io::Result<BuildResult> {
        panic!("BuildService must NOT be called when preflight hits");
    }
}

#[derive(Default, Clone)]
pub struct CountingPathInfoService {
    entries: Arc<Mutex<HashMap<[u8; 20], PathInfo>>>,
    get_counts: Arc<Mutex<HashMap<[u8; 20], u32>>>,
}

impl CountingPathInfoService {
    pub fn get_count(&self, path: &StorePath<String>) -> u32 {
        self.get_counts.lock().unwrap().get(path.digest()).copied().unwrap_or(0)
    }
}

#[async_trait]
impl PathInfoService for CountingPathInfoService {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        let mut counts = self.get_counts.lock().unwrap();
        let next_count = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
        counts.insert(digest, next_count);
        drop(counts);
        Ok(self.entries.lock().unwrap().get(&digest).cloned())
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info.clone());
        Ok(path_info)
    }

    fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        let entries: Vec<_> = self.entries.lock().unwrap().values().cloned().collect();
        futures::stream::iter(entries.into_iter().map(Ok)).boxed()
    }
}

pub struct SequencePathInfoService {
    queues: Arc<Mutex<HashMap<[u8; 20], VecDeque<PathInfo>>>>,
}

impl SequencePathInfoService {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn push(&self, path_info: PathInfo) {
        self.queues.lock().unwrap().entry(*path_info.store_path.digest()).or_default().push_back(path_info);
    }
}

#[async_trait]
impl PathInfoService for SequencePathInfoService {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        let mut guard = self.queues.lock().unwrap();
        let Some(queue) = guard.get_mut(&digest) else {
            return Ok(None);
        };
        if queue.len() > 1 {
            Ok(queue.pop_front())
        } else {
            Ok(queue.front().cloned())
        }
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        self.push(path_info.clone());
        Ok(path_info)
    }

    fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        let entries: Vec<_> = self.queues.lock().unwrap().values().flat_map(|queue| queue.iter().cloned()).collect();
        futures::stream::iter(entries.into_iter().map(Ok)).boxed()
    }
}

pub fn corrupt_signature_bytes(path_info: &mut PathInfo) {
    assert!(!path_info.signatures.is_empty(), "signature must exist before corruption");
    let bad_bytes = [0u8; 64];
    let bad_sig_str = format!("{}:{}", path_info.signatures[0].name(), data_encoding::BASE64.encode(&bad_bytes));
    path_info.signatures[0] = Signature::<String>::parse(&bad_sig_str).unwrap();
}

const STATIC_HTTP_READ_CAPACITY_BYTES: usize = 16_384;
const STATIC_HTTP_MAX_FILES: usize = 64;
const STATIC_HTTP_POLL_MS: u64 = 5;

pub struct StaticHttpServer {
    pub base_url: String,
    stop: Arc<AtomicBool>,
    pub requests: Arc<Mutex<Vec<String>>>,
    thread: Option<JoinHandle<()>>,
}

impl StaticHttpServer {
    pub fn start(files: BTreeMap<String, Vec<u8>>) -> Self {
        assert!(!files.is_empty());
        assert!(files.len() <= STATIC_HTTP_MAX_FILES);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let thread_stop = stop.clone();
        let thread_requests = requests.clone();
        let thread = std::thread::spawn(move || {
            while !thread_stop.load(AtomicOrdering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => serve_static_http_request(stream, &files, &thread_requests),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(STATIC_HTTP_POLL_MS));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            base_url: format!("http://{address}"),
            stop,
            requests,
            thread: Some(thread),
        }
    }
}

impl Drop for StaticHttpServer {
    fn drop(&mut self) {
        self.stop.store(true, AtomicOrdering::SeqCst);
        let _ = TcpStream::connect(self.base_url.trim_start_matches("http://"));
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn serve_static_http_request(
    mut stream: TcpStream,
    files: &BTreeMap<String, Vec<u8>>,
    requests: &Arc<Mutex<Vec<String>>>,
) {
    let mut request = [0u8; STATIC_HTTP_READ_CAPACITY_BYTES];
    let bytes_read = stream.read(&mut request).unwrap_or(0);
    let request_text = String::from_utf8_lossy(&request[..bytes_read]);
    let path = request_text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    requests.lock().unwrap().push(path.clone());
    let (status, body) =
        files.get(&path).map(|body| ("200 OK", body.as_slice())).unwrap_or(("404 Not Found", b"missing"));
    let response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
}

pub fn collect_static_http_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
                continue;
            }
            let relative = entry.path().strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            files.insert(format!("/{relative}"), std::fs::read(entry.path()).unwrap());
            assert!(files.len() <= STATIC_HTTP_MAX_FILES);
        }
    }
    files
}

pub fn write_http_action_result_sidecars(
    cache_dir: &Path,
    index: &crunch_action_result_core::ActionResultIndex,
    signed: &crunch_action_result_core::SignedActionResultRecord,
) {
    let action_digest = index.action_ref.strip_prefix(crunch_action_result_core::ACTION_REF_PREFIX).unwrap();
    let result_digest =
        signed.record.result_ref.strip_prefix(crunch_action_result_core::ACTION_RESULT_REF_PREFIX).unwrap();
    let index_path = cache_dir.join("action-results/v1/indexes").join(format!("{action_digest}.json"));
    let record_path = cache_dir.join("action-results/v1/records").join(format!("{result_digest}.json"));
    std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(record_path.parent().unwrap()).unwrap();
    std::fs::write(index_path, crunch_action_result_core::canonical_index_bytes(index).unwrap()).unwrap();
    std::fs::write(record_path, crunch_action_result_core::canonical_signed_record_bytes(signed).unwrap()).unwrap();
}
