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
use std::sync::atomic::AtomicU32;
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
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::blobservice::BlobReader;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::DirectoryPutter;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::proto::stat_blob_response::ChunkMeta;
use snix_castore::service_provenance::LayeredRead;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice;
use snix_store::pathinfoservice::LruPathInfoService;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncReadExt;
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
    total_gets: Arc<AtomicU32>,
    puts: Arc<AtomicU32>,
    lists: Arc<AtomicU32>,
}

impl CountingPathInfoService {
    pub fn get_count(&self, path: &StorePath<String>) -> u32 {
        self.get_counts.lock().unwrap().get(path.digest()).copied().unwrap_or(0)
    }

    pub fn total_get_count(&self) -> u32 {
        self.total_gets.load(AtomicOrdering::SeqCst)
    }

    pub fn put_count(&self) -> u32 {
        self.puts.load(AtomicOrdering::SeqCst)
    }

    pub fn list_count(&self) -> u32 {
        self.lists.load(AtomicOrdering::SeqCst)
    }

    pub fn reset_counts(&self) {
        self.get_counts.lock().unwrap().clear();
        self.total_gets.store(0, AtomicOrdering::SeqCst);
        self.puts.store(0, AtomicOrdering::SeqCst);
        self.lists.store(0, AtomicOrdering::SeqCst);
    }

    pub fn insert_for_test(&self, path_info: PathInfo) {
        self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info);
    }

    pub fn get_cached_for_test(&self, path: &StorePath<String>) -> Option<PathInfo> {
        self.entries.lock().unwrap().get(path.digest()).cloned()
    }

    pub fn remove_for_test(&self, path: &StorePath<String>) -> Option<PathInfo> {
        self.entries.lock().unwrap().remove(path.digest())
    }
}

#[async_trait]
impl PathInfoService for CountingPathInfoService {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        self.total_gets.fetch_add(1, AtomicOrdering::SeqCst);
        let mut counts = self.get_counts.lock().unwrap();
        let next_count = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
        counts.insert(digest, next_count);
        drop(counts);
        Ok(self.entries.lock().unwrap().get(&digest).cloned())
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        self.puts.fetch_add(1, AtomicOrdering::SeqCst);
        self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info.clone());
        Ok(path_info)
    }

    fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        self.lists.fetch_add(1, AtomicOrdering::SeqCst);
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlobServiceCounts {
    pub has_calls: u32,
    pub open_read_calls: u32,
    pub open_write_calls: u32,
    pub chunks_calls: u32,
}

#[derive(Clone)]
pub struct CountingBlobService {
    inner: Arc<dyn BlobService>,
    has_calls: Arc<AtomicU32>,
    open_read_calls: Arc<AtomicU32>,
    open_write_calls: Arc<AtomicU32>,
    chunks_calls: Arc<AtomicU32>,
    hidden_digests: Arc<Mutex<BTreeSet<[u8; 32]>>>,
}

impl CountingBlobService {
    pub fn new(inner: Arc<dyn BlobService>) -> Self {
        Self {
            inner,
            has_calls: Arc::new(AtomicU32::new(0)),
            open_read_calls: Arc::new(AtomicU32::new(0)),
            open_write_calls: Arc::new(AtomicU32::new(0)),
            chunks_calls: Arc::new(AtomicU32::new(0)),
            hidden_digests: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub fn counts(&self) -> BlobServiceCounts {
        BlobServiceCounts {
            has_calls: self.has_calls.load(AtomicOrdering::SeqCst),
            open_read_calls: self.open_read_calls.load(AtomicOrdering::SeqCst),
            open_write_calls: self.open_write_calls.load(AtomicOrdering::SeqCst),
            chunks_calls: self.chunks_calls.load(AtomicOrdering::SeqCst),
        }
    }

    pub fn reset_counts(&self) {
        self.has_calls.store(0, AtomicOrdering::SeqCst);
        self.open_read_calls.store(0, AtomicOrdering::SeqCst);
        self.open_write_calls.store(0, AtomicOrdering::SeqCst);
        self.chunks_calls.store(0, AtomicOrdering::SeqCst);
    }

    pub fn hide_digest(&self, digest: &B3Digest) {
        let mut raw = [0u8; 32];
        raw.copy_from_slice(digest.as_slice());
        self.hidden_digests.lock().unwrap().insert(raw);
    }

    pub fn unhide_all(&self) {
        self.hidden_digests.lock().unwrap().clear();
    }
}

struct UnhidingBlobWriter {
    inner: Box<dyn snix_castore::blobservice::BlobWriter>,
    hidden_digests: Arc<Mutex<BTreeSet<[u8; 32]>>>,
}

impl tokio::io::AsyncWrite for UnhidingBlobWriter {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_shutdown(cx)
    }
}

#[async_trait]
impl snix_castore::blobservice::BlobWriter for UnhidingBlobWriter {
    async fn close(&mut self) -> std::io::Result<B3Digest> {
        let digest = self.inner.close().await?;
        let slice = digest.as_slice();
        if slice.len() == 32 {
            let mut raw = [0u8; 32];
            raw.copy_from_slice(slice);
            self.hidden_digests.lock().unwrap().remove(&raw);
        }
        Ok(digest)
    }
}

impl CountingBlobService {
    fn is_hidden(&self, digest: &B3Digest) -> bool {
        let guard = self.hidden_digests.lock().unwrap();
        let slice = digest.as_slice();
        if slice.len() != 32 {
            return false;
        }
        let mut raw = [0u8; 32];
        raw.copy_from_slice(slice);
        guard.contains(&raw)
    }

    pub async fn read_all_bytes_uncounted(&self, digest: &B3Digest) -> std::io::Result<Option<Vec<u8>>> {
        if self.is_hidden(digest) {
            return Ok(None);
        }
        let Some(mut reader) = self.inner.open_read(digest).await? else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(Some(bytes))
    }
}

#[async_trait]
impl BlobService for CountingBlobService {
    async fn has(&self, digest: &B3Digest) -> std::io::Result<bool> {
        self.has_calls.fetch_add(1, AtomicOrdering::SeqCst);
        if self.is_hidden(digest) {
            return Ok(false);
        }
        self.inner.has(digest).await
    }

    async fn open_read(&self, digest: &B3Digest) -> std::io::Result<Option<Box<dyn BlobReader>>> {
        self.open_read_calls.fetch_add(1, AtomicOrdering::SeqCst);
        if self.is_hidden(digest) {
            return Ok(None);
        }
        self.inner.open_read(digest).await
    }

    async fn open_read_with_layer(
        &self,
        digest: &B3Digest,
    ) -> std::io::Result<Option<LayeredRead<Box<dyn BlobReader>>>> {
        self.open_read_calls.fetch_add(1, AtomicOrdering::SeqCst);
        if self.is_hidden(digest) {
            return Ok(None);
        }
        self.inner.open_read_with_layer(digest).await
    }

    async fn open_write(&self) -> Box<dyn snix_castore::blobservice::BlobWriter> {
        self.open_write_calls.fetch_add(1, AtomicOrdering::SeqCst);
        Box::new(UnhidingBlobWriter {
            inner: self.inner.open_write().await,
            hidden_digests: self.hidden_digests.clone(),
        })
    }

    async fn chunks(&self, digest: &B3Digest) -> std::io::Result<Option<Vec<ChunkMeta>>> {
        self.chunks_calls.fetch_add(1, AtomicOrdering::SeqCst);
        if self.is_hidden(digest) {
            return Ok(None);
        }
        self.inner.chunks(digest).await
    }

    async fn chunks_with_layer(&self, digest: &B3Digest) -> std::io::Result<Option<LayeredRead<Vec<ChunkMeta>>>> {
        self.chunks_calls.fetch_add(1, AtomicOrdering::SeqCst);
        if self.is_hidden(digest) {
            return Ok(None);
        }
        self.inner.chunks_with_layer(digest).await
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectoryServiceCounts {
    pub get_calls: u32,
    pub put_calls: u32,
    pub get_recursive_calls: u32,
}

#[derive(Clone)]
pub struct CountingDirectoryService {
    inner: Arc<dyn DirectoryService>,
    get_calls: Arc<AtomicU32>,
    put_calls: Arc<AtomicU32>,
    get_recursive_calls: Arc<AtomicU32>,
}

impl CountingDirectoryService {
    pub fn new(inner: Arc<dyn DirectoryService>) -> Self {
        Self {
            inner,
            get_calls: Arc::new(AtomicU32::new(0)),
            put_calls: Arc::new(AtomicU32::new(0)),
            get_recursive_calls: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn counts(&self) -> DirectoryServiceCounts {
        DirectoryServiceCounts {
            get_calls: self.get_calls.load(AtomicOrdering::SeqCst),
            put_calls: self.put_calls.load(AtomicOrdering::SeqCst),
            get_recursive_calls: self.get_recursive_calls.load(AtomicOrdering::SeqCst),
        }
    }

    pub fn reset_counts(&self) {
        self.get_calls.store(0, AtomicOrdering::SeqCst);
        self.put_calls.store(0, AtomicOrdering::SeqCst);
        self.get_recursive_calls.store(0, AtomicOrdering::SeqCst);
    }
}

#[async_trait]
impl DirectoryService for CountingDirectoryService {
    async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, snix_castore::directoryservice::Error> {
        self.get_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.get(digest).await
    }

    async fn get_with_layer(
        &self,
        digest: &B3Digest,
    ) -> Result<Option<LayeredRead<Directory>>, snix_castore::directoryservice::Error> {
        self.get_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.get_with_layer(digest).await
    }

    async fn put(&self, directory: Directory) -> Result<B3Digest, snix_castore::directoryservice::Error> {
        self.put_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.put(directory).await
    }

    fn get_recursive(
        &self,
        root_directory_digest: &B3Digest,
    ) -> futures::stream::BoxStream<'_, Result<Directory, snix_castore::directoryservice::Error>> {
        self.get_recursive_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.get_recursive(root_directory_digest)
    }

    fn get_recursive_with_layer(
        &self,
        root_directory_digest: &B3Digest,
    ) -> futures::stream::BoxStream<'_, Result<LayeredRead<Directory>, snix_castore::directoryservice::Error>> {
        self.get_recursive_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.get_recursive_with_layer(root_directory_digest)
    }

    fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
        self.inner.put_multiple_start()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionResultStoreCounts {
    pub lookup_calls: u32,
    pub publish_calls: u32,
}

#[derive(Debug, Clone)]
pub struct CountingActionResultStore {
    inner: Arc<dyn crunch_store::ActionResultStore>,
    lookup_calls: Arc<AtomicU32>,
    publish_calls: Arc<AtomicU32>,
    injected_records: Arc<Mutex<BTreeMap<String, Vec<crunch_action_result_core::SignedActionResultRecord>>>>,
    reverse_records: Arc<AtomicBool>,
}

impl CountingActionResultStore {
    pub fn new(inner: Arc<dyn crunch_store::ActionResultStore>) -> Self {
        Self {
            inner,
            lookup_calls: Arc::new(AtomicU32::new(0)),
            publish_calls: Arc::new(AtomicU32::new(0)),
            injected_records: Arc::new(Mutex::new(BTreeMap::new())),
            reverse_records: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn counts(&self) -> ActionResultStoreCounts {
        ActionResultStoreCounts {
            lookup_calls: self.lookup_calls.load(AtomicOrdering::SeqCst),
            publish_calls: self.publish_calls.load(AtomicOrdering::SeqCst),
        }
    }

    pub fn reset_counts(&self) {
        self.lookup_calls.store(0, AtomicOrdering::SeqCst);
        self.publish_calls.store(0, AtomicOrdering::SeqCst);
    }

    pub fn inject_record(&self, action_ref: &str, signed: crunch_action_result_core::SignedActionResultRecord) {
        self.injected_records.lock().unwrap().entry(action_ref.to_string()).or_default().push(signed);
    }

    pub fn set_reverse_records(&self, reverse: bool) {
        self.reverse_records.store(reverse, AtomicOrdering::SeqCst);
    }
}

#[async_trait]
impl crunch_store::ActionResultStore for CountingActionResultStore {
    fn source_id(&self) -> &str {
        self.inner.source_id()
    }

    fn source_class(&self) -> &str {
        self.inner.source_class()
    }

    async fn lookup(&self, action_ref: &str) -> Result<crunch_store::ActionResultLookup, String> {
        self.lookup_calls.fetch_add(1, AtomicOrdering::SeqCst);
        let mut result = self.inner.lookup(action_ref).await?;
        if let Some(extra) = self.injected_records.lock().unwrap().get(action_ref) {
            result.records.extend(extra.iter().cloned());
            result.index = crunch_action_result_core::canonical_action_result_index(
                action_ref.to_string(),
                result.records.iter().map(|r| r.record.result_ref.clone()).collect(),
            )
            .map_err(|err| err.code().to_string())?;
        }
        if self.reverse_records.load(AtomicOrdering::SeqCst) {
            result.records.reverse();
        }
        Ok(result)
    }

    async fn publish(
        &self,
        record: &crunch_action_result_core::SignedActionResultRecord,
    ) -> Result<crunch_store::ActionResultPublicationReport, String> {
        self.publish_calls.fetch_add(1, AtomicOrdering::SeqCst);
        self.inner.publish(record).await
    }
}

pub const MANTLE_REF_PROG_ARG_PREFIX: &str = "--mantle-ref-prog=";
pub const REFSCAN_SUFFIX_MARKER: &[u8] = b"\n--mantle-ref:";

pub fn strip_refscan_suffix(bytes: &[u8]) -> &[u8] {
    if let Some(pos) = bytes.windows(REFSCAN_SUFFIX_MARKER.len()).position(|w| w == REFSCAN_SUFFIX_MARKER) {
        &bytes[..pos]
    } else {
        bytes
    }
}

fn logical_store_name(component: &str) -> &str {
    // Store path components look like "<32-char-nixbase32-hash>-<name>".
    if component.len() > 33 && component.as_bytes()[32] == b'-' {
        &component[33..]
    } else {
        component
    }
}

fn parse_ref_tags(suffix: &[u8]) -> Vec<String> {
    let mut tags = Vec::new();
    let mut rest = suffix;
    while let Some(after_prefix) = rest.strip_prefix(REFSCAN_SUFFIX_MARKER) {
        if let Some(next_pos) =
            after_prefix.windows(REFSCAN_SUFFIX_MARKER.len()).position(|w| w == REFSCAN_SUFFIX_MARKER)
        {
            tags.push(String::from_utf8_lossy(&after_prefix[..next_pos]).into_owned());
            rest = &after_prefix[next_pos..];
        } else {
            tags.push(String::from_utf8_lossy(after_prefix).into_owned());
            break;
        }
    }
    tags
}

pub fn compute_fake_exec_output_prefix(
    task_name: &str,
    output_name: &str,
    salt: u64,
    sorted_inputs: &[(&str, &[u8])],
) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle.ref.fake-exec.v1");
    hasher.update(&salt.to_le_bytes());
    hasher.update(output_name.as_bytes());
    for (logical_name, bytes) in sorted_inputs {
        hasher.update(&(logical_name.len() as u64).to_le_bytes());
        hasher.update(logical_name.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    let digest = hasher.finalize();
    format!("out:{task_name}:{output_name}:{}", digest.to_hex()).into_bytes()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FakeRefTagMatcher {
    SelfOutput,
    SiblingOutput,
    InputComponent(String),
    InputContent {
        logical_name: String,
        content_digest: [u8; 32],
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FakeInputMatcher {
    NameOrComponent(String),
    ExactComponent(String),
    ContentDigest {
        logical_name: String,
        content_digest: [u8; 32],
        exclude_components: Vec<String>,
        expected_refs: Vec<FakeRefTagMatcher>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FakeOutputProgram {
    pub salt: u64,
    pub ca_content_override: Option<Vec<u8>>,
    pub embed_self_output_ref: bool,
    pub ref_sibling_outputs: Vec<String>,
    pub ref_input_names: Vec<String>,
    #[serde(default)]
    pub ref_inputs: Vec<FakeInputMatcher>,
    pub dynamic_plan_branches: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FakeBuildProgram {
    pub task_name: String,
    pub expected_input_names: Vec<String>,
    #[serde(default)]
    pub expected_inputs: Vec<FakeInputMatcher>,
    pub ignore_plan_inputs: bool,
    pub output_programs: BTreeMap<String, FakeOutputProgram>,
    pub fail_message: Option<String>,
    pub omit_outputs: Vec<String>,
}

struct MountedInputEntry {
    component: String,
    logical_name: String,
    content_prefix: Vec<u8>,
    content_digest: [u8; 32],
    ref_tags: Vec<String>,
}

fn matches_ref_tags(ref_tags: &[String], expected_refs: &[FakeRefTagMatcher], mounted: &[MountedInputEntry]) -> bool {
    if ref_tags.len() != expected_refs.len() {
        return false;
    }
    ref_tags.iter().zip(expected_refs).all(|(tag, expected)| match expected {
        FakeRefTagMatcher::SelfOutput => tag.starts_with("self:"),
        FakeRefTagMatcher::SiblingOutput => tag.starts_with("sibling:"),
        FakeRefTagMatcher::InputComponent(comp) => tag.strip_prefix("input:") == Some(comp.as_str()),
        FakeRefTagMatcher::InputContent {
            logical_name,
            content_digest,
        } => tag.strip_prefix("input:").is_some_and(|comp| {
            if let Some(entry) = mounted.iter().find(|m| m.component == comp) {
                entry.logical_name == *logical_name && entry.content_digest == *content_digest
            } else {
                logical_store_name(comp) == logical_name
            }
        }),
    })
}

fn find_matching_mounted_input<'a>(
    matcher: &FakeInputMatcher,
    mounted: &'a [MountedInputEntry],
) -> Option<&'a MountedInputEntry> {
    match matcher {
        FakeInputMatcher::NameOrComponent(target) => {
            mounted.iter().find(|m| m.logical_name == *target || m.component == *target)
        }
        FakeInputMatcher::ExactComponent(comp) => mounted.iter().find(|m| m.component == *comp),
        FakeInputMatcher::ContentDigest {
            logical_name,
            content_digest,
            exclude_components,
            expected_refs,
        } => mounted.iter().find(|m| {
            m.logical_name == *logical_name
                && m.content_digest == *content_digest
                && !exclude_components.iter().any(|ex| ex == &m.component)
                && matches_ref_tags(&m.ref_tags, expected_refs, mounted)
        }),
    }
}

#[derive(Debug, Clone)]
pub struct RecordedBuildCall {
    pub task_name: Option<String>,
    pub command_args: Vec<String>,
    pub outputs: Vec<std::path::PathBuf>,
    pub input_components: Vec<String>,
    pub was_cancellable: bool,
    pub cancelled_during_run: bool,
}

#[derive(Clone)]
pub struct WatchBarrierGate {
    pub match_substring: String,
    pub started: tokio::sync::mpsc::UnboundedSender<String>,
    pub cancellation_observed: tokio::sync::mpsc::UnboundedSender<String>,
    pub release_teardown: tokio::sync::watch::Receiver<bool>,
}

#[derive(Clone)]
pub struct DeterministicFakeBuildService {
    blob_service: Arc<dyn BlobService>,
    calls: Arc<Mutex<Vec<RecordedBuildCall>>>,
    watch_gate: Option<WatchBarrierGate>,
    fail_on_missing_expected_input: bool,
}

impl DeterministicFakeBuildService {
    pub fn new(blob_service: Arc<dyn BlobService>) -> Self {
        Self {
            blob_service,
            calls: Arc::new(Mutex::new(Vec::new())),
            watch_gate: None,
            fail_on_missing_expected_input: true,
        }
    }

    pub fn with_watch_gate(mut self, gate: WatchBarrierGate) -> Self {
        self.watch_gate = Some(gate);
        self
    }

    pub fn recorded_calls(&self) -> Vec<RecordedBuildCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn call_count(&self) -> u32 {
        u32::try_from(self.calls.lock().unwrap().len()).unwrap_or(u32::MAX)
    }

    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }

    async fn read_mounted_input_bytes(&self, node: &Node) -> std::io::Result<Vec<u8>> {
        match node {
            Node::File { digest, .. } => {
                let Some(mut reader) = self.blob_service.open_read(digest).await? else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("mounted input blob {digest} missing from BlobService"),
                    ));
                };
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf).await?;
                Ok(buf)
            }
            Node::Symlink { target } => Ok(target.as_ref().to_vec()),
            Node::Directory { digest, size } => Ok(format!("dir:{digest}:{size}").into_bytes()),
        }
    }

    async fn execute_request(
        &self,
        request: BuildRequest,
        was_cancellable: bool,
        cancelled_during_run: bool,
    ) -> std::io::Result<BuildResult> {
        let program = request.command_args.iter().find_map(|arg| {
            arg.strip_prefix(MANTLE_REF_PROG_ARG_PREFIX)
                .and_then(|json| serde_json::from_str::<FakeBuildProgram>(json).ok())
        });
        let input_components: Vec<String> = request
            .inputs
            .keys()
            .map(|component| String::from_utf8_lossy(component.as_ref()).into_owned())
            .collect();
        self.calls.lock().unwrap().push(RecordedBuildCall {
            task_name: program.as_ref().map(|p| p.task_name.clone()),
            command_args: request.command_args.clone(),
            outputs: request.outputs.clone(),
            input_components: input_components.clone(),
            was_cancellable,
            cancelled_during_run,
        });

        let Some(program) = program else {
            let mut writer = BlobService::open_write(&self.blob_service).await;
            let payload = format!("fallback:{}", request.command_args.join(" ")).into_bytes();
            let size = u64::try_from(payload.len()).map_err(std::io::Error::other)?;
            writer.write_all(&payload).await?;
            let digest = writer.close().await?;
            let outputs = request
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
            return Ok(BuildResult {
                outputs,
                log: None,
                timing: None,
            });
        };

        if let Some(reason) = &program.fail_message {
            return Err(std::io::Error::other(reason.clone()));
        }

        // Read all mounted input bytes in deterministic order, verifying declared inputs exist.
        let mut mounted_by_component: Vec<MountedInputEntry> = Vec::with_capacity(request.inputs.len());
        for (component, node) in &request.inputs {
            let comp_name = String::from_utf8_lossy(component.as_ref()).into_owned();
            let logical_name = logical_store_name(&comp_name).to_string();
            let bytes = self.read_mounted_input_bytes(node).await?;
            let content_prefix = strip_refscan_suffix(&bytes).to_vec();
            let content_digest = *blake3::hash(&content_prefix).as_bytes();
            let ref_tags = parse_ref_tags(&bytes[content_prefix.len()..]);
            mounted_by_component.push(MountedInputEntry {
                component: comp_name,
                logical_name,
                content_prefix,
                content_digest,
                ref_tags,
            });
        }
        mounted_by_component.sort_by(|left, right| {
            left.logical_name
                .cmp(&right.logical_name)
                .then(left.content_digest.cmp(&right.content_digest))
                .then(left.component.cmp(&right.component))
        });

        let hash_inputs: Vec<(&str, &[u8])> = mounted_by_component
            .iter()
            .filter(|entry| !(program.ignore_plan_inputs && entry.logical_name.ends_with("-plan")))
            .map(|entry| (entry.logical_name.as_str(), entry.content_prefix.as_slice()))
            .collect();

        if self.fail_on_missing_expected_input {
            for expected in &program.expected_input_names {
                let found = mounted_by_component
                    .iter()
                    .any(|entry| entry.logical_name == *expected || entry.component == *expected);
                if !found {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "task '{}' expected mounted input '{}' in BuildRequest.inputs, found {:?}",
                            program.task_name, expected, input_components
                        ),
                    ));
                }
            }
            for matcher in &program.expected_inputs {
                if find_matching_mounted_input(matcher, &mounted_by_component).is_none() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "task '{}' expected mounted input matching {:?} in BuildRequest.inputs, found {:?}",
                            program.task_name, matcher, input_components
                        ),
                    ));
                }
            }
            for arg in &request.command_args {
                if let Some((_, bound_path)) = arg.split_once('=').filter(|(k, _)| k.starts_with("--plan-binding-")) {
                    if bound_path.contains("{{mantle-plan-output:") {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("unbound plan output placeholder in command_args: {arg}"),
                        ));
                    }
                    let comp = bound_path.rsplit('/').next().unwrap_or(bound_path);
                    if !mounted_by_component.iter().any(|entry| entry.component == comp) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("bound plan root '{comp}' missing from BuildRequest.inputs"),
                        ));
                    }
                }
            }
        }

        let mut env_map: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for ev in &request.environment_vars {
            env_map.insert(ev.key.clone(), ev.value.to_vec());
        }

        // Map each request.outputs[i] back to its output name via env_map values.
        let mut outputs = Vec::with_capacity(request.outputs.len());
        for (idx, out_rel_path) in request.outputs.iter().enumerate() {
            let out_rel_str = out_rel_path.to_string_lossy();
            let out_abs_suffix = format!("/{out_rel_str}");
            let output_name = env_map
                .iter()
                .find(|(k, v)| {
                    program.output_programs.contains_key(k.as_str())
                        && (String::from_utf8_lossy(v) == out_abs_suffix || String::from_utf8_lossy(v) == out_rel_str)
                })
                .map(|(k, _)| k.clone())
                .or_else(|| program.output_programs.keys().nth(idx).cloned())
                .unwrap_or_else(|| "out".to_string());

            if program.omit_outputs.contains(&output_name) {
                continue;
            }

            let rule = program.output_programs.get(&output_name).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("missing fake output rule for output '{output_name}'"),
                )
            })?;

            let mut payload = if let Some(branches) = &rule.dynamic_plan_branches {
                let mut hasher = blake3::Hasher::new();
                for (_, bytes) in &hash_inputs {
                    hasher.update(&(bytes.len() as u64).to_le_bytes());
                    hasher.update(bytes);
                }
                let digest = hasher.finalize();
                let selector = usize::from(digest.as_bytes()[0]);
                let branch_idx = selector % branches.len().max(1);
                branches
                    .get(branch_idx)
                    .cloned()
                    .ok_or_else(|| std::io::Error::other("empty dynamic plan branches"))?
            } else if let Some(override_bytes) = &rule.ca_content_override {
                override_bytes.clone()
            } else {
                compute_fake_exec_output_prefix(&program.task_name, &output_name, rule.salt, &hash_inputs)
            };

            if rule.embed_self_output_ref
                && let Some(self_path) = env_map.get(&output_name)
            {
                payload.extend_from_slice(REFSCAN_SUFFIX_MARKER);
                payload.extend_from_slice(b"self:");
                payload.extend_from_slice(self_path);
            }
            for sibling in &rule.ref_sibling_outputs {
                if let Some(sib_path) = env_map.get(sibling) {
                    payload.extend_from_slice(REFSCAN_SUFFIX_MARKER);
                    payload.extend_from_slice(b"sibling:");
                    payload.extend_from_slice(sib_path);
                }
            }
            for ref_input in &rule.ref_input_names {
                if let Some(entry) = mounted_by_component
                    .iter()
                    .find(|entry| entry.logical_name == *ref_input || entry.component == *ref_input)
                {
                    payload.extend_from_slice(REFSCAN_SUFFIX_MARKER);
                    payload.extend_from_slice(b"input:");
                    payload.extend_from_slice(entry.component.as_bytes());
                }
            }
            for matcher in &rule.ref_inputs {
                if let Some(entry) = find_matching_mounted_input(matcher, &mounted_by_component) {
                    payload.extend_from_slice(REFSCAN_SUFFIX_MARKER);
                    payload.extend_from_slice(b"input:");
                    payload.extend_from_slice(entry.component.as_bytes());
                }
            }

            let mut output_needles = BTreeSet::new();
            for (needle_idx, needle) in request.refscan_needles.iter().enumerate() {
                let needle_bytes = needle.as_bytes();
                if !needle_bytes.is_empty() && payload.windows(needle_bytes.len()).any(|w| w == needle_bytes) {
                    output_needles.insert(u64::try_from(needle_idx).map_err(std::io::Error::other)?);
                }
            }

            let size = u64::try_from(payload.len()).map_err(std::io::Error::other)?;
            let mut writer = BlobService::open_write(&self.blob_service).await;
            writer.write_all(&payload).await?;
            let digest = writer.close().await?;

            outputs.push(BuildOutput {
                node: Node::File {
                    digest,
                    size,
                    executable: false,
                },
                output_needles,
            });
        }

        Ok(BuildResult {
            outputs,
            log: None,
            timing: None,
        })
    }
}

#[async_trait]
impl BuildService for DeterministicFakeBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.execute_request(request, false, false).await
    }

    async fn do_build_cancellable(
        &self,
        request: BuildRequest,
        mut cancellation: tokio::sync::watch::Receiver<bool>,
    ) -> std::io::Result<BuildResult> {
        let mut cancelled_during_run = *cancellation.borrow();
        if let Some(gate) = &self.watch_gate {
            let matches = request.command_args.iter().any(|arg| arg.contains(&gate.match_substring));
            if matches {
                let _ = gate.started.send(gate.match_substring.clone());
                while !*cancellation.borrow_and_update() {
                    cancellation.changed().await.map_err(std::io::Error::other)?;
                }
                cancelled_during_run = true;
                let _ = gate.cancellation_observed.send(gate.match_substring.clone());
                let mut release = gate.release_teardown.clone();
                while !*release.borrow_and_update() {
                    release.changed().await.map_err(std::io::Error::other)?;
                }
                // Intentionally return Ok(BuildResult) after cancellation to test that
                // Worker::run_watch / run_streaming_cancellable discards stale results.
            }
        }
        self.execute_request(request, true, cancelled_during_run).await
    }
}
