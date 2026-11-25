use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
use crunch_action_result_core::ACTION_REF_PREFIX;
use crunch_action_result_core::ACTION_RESULT_REF_PREFIX;
use crunch_action_result_core::ActionResultOutput;
use crunch_action_result_core::ActionResultRecordInput;
use crunch_action_result_core::DetachedRecordSignature;
use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
use crunch_action_result_core::OBJECT_REF_PREFIX;
use crunch_action_result_core::PATH_INFO_REF_PREFIX;
use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
use crunch_action_result_core::SIGNATURE_REF_PREFIX;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::canonical_action_result;
use crunch_store::ActionResultStore;
use crunch_store::GcRootSource;
use crunch_store::HttpActionResultStore;
use crunch_store::LocalActionResultStore;
use crunch_store::PersistOutputRequest;
use crunch_store::PullOptions;
use crunch_store::PushOptions;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use crunch_store::StoreMutationGuard;
use crunch_store::StoreSource;
use crunch_store::StoreSourceAdmission;
use crunch_store::StoreSourceLease;
use crunch_store::export_paths_to_cache_dir;
use crunch_store::import_paths_from_cache_dir;
use crunch_store::store_verify;
use futures::StreamExt;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use snix_store::nar::NarCalculationService as _;
use snix_store::nar::SimpleRenderer;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncWriteExt;
use url::Url;

use super::artifacts::CounterexampleArtifact;
use super::artifacts::ExecutedScheduleStep;
use super::artifacts::RequestedVsAppliedFault;
use super::artifacts::build_artifact;
use super::artifacts::persist_counterexample_artifact;
use super::artifacts::shrink_history_preserving_failure;
use super::language::FaultInjectionMode;
use super::language::GcExecutionKind;
use super::language::HistoryProgramV1;
use super::language::ReopenKind;
use super::language::StoreOp;
use super::language::StoreProfile;
use super::language::TrustAdmissionMode;
use super::model::ModelPlanSnapshot;
use super::model::StepClassification;
use super::model::StoreRefModel;
use super::model::ViolationKind;

pub const TEST_STORE_DIR: &str = "/mantle/store";
pub const CHILD_REQ_ENV: &str = "MANTLE_STORE_HISTORY_CHILD_REQ";
pub const CHILD_ENTRY_TEST_NAME: &str = "store_crash_reopen_child_entry";

#[derive(Debug)]
struct TestHttpAdmission;

impl StoreSourceLease for TestHttpAdmission {}

impl StoreSourceAdmission for TestHttpAdmission {
    fn admit(
        &self,
        _source: StoreSource<'_>,
        _store_prefix: &str,
    ) -> Result<Arc<dyn StoreSourceLease>, crunch_store::Error> {
        Ok(Arc::new(Self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegativeMutant {
    None,
    ExposeIncompleteIndex,
    LoseProtectedReference,
}

#[derive(Debug, Clone)]
pub struct HistoryRunOutcome {
    pub violation: Option<(ViolationKind, String)>,
    pub requested_vs_applied_faults: Vec<RequestedVsAppliedFault>,
    pub executed_schedule: Vec<ExecutedScheduleStep>,
    pub is_mutant_reached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildRequest {
    pub profile: StoreProfile,
    pub state_dir: PathBuf,
    pub output_dir: PathBuf,
    pub store_dir: String,
    pub response_path: PathBuf,
    pub action: ChildAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildAction {
    VerifyReopen {
        expected_live_paths: Vec<String>,
        expected_unsigned_paths: Vec<String>,
        expected_action_refs: Vec<(String, String)>,
    },
    WriteActionRecordAndAbort {
        action_ref: String,
        record_json: String,
        partial_record_path: PathBuf,
        partial_bytes: Vec<u8>,
    },
    RaceMutationGuard {
        role: String,
        is_expect_busy: bool,
        is_wait_for_lock: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildReport {
    pub is_ok: bool,
    pub detail: String,
    pub observed_paths: Vec<String>,
}

#[derive(Default)]
struct HttpMockState {
    resources: BTreeMap<String, Vec<u8>>,
    is_fail_index_puts: bool,
}

struct HttpActionResultFixture {
    url: Url,
    state: Arc<Mutex<HttpMockState>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl HttpActionResultFixture {
    fn spawn() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind http mock: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| format!("nonblocking http mock: {e}"))?;
        let addr = listener.local_addr().map_err(|e| format!("addr http mock: {e}"))?;
        let state = Arc::new(Mutex::new(HttpMockState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let state_bg = Arc::clone(&state);
        let stop_bg = Arc::clone(&stop);
        let thread = std::thread::spawn(move || {
            while !stop_bg.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = handle_http_connection(stream, &state_bg);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        let url = Url::parse(&format!("http://{addr}/")).map_err(|e| format!("parse url: {e}"))?;
        Ok(Self {
            url,
            state,
            stop,
            thread: Some(thread),
        })
    }

    fn store(&self) -> Result<HttpActionResultStore, String> {
        HttpActionResultStore::with_default_timeout(self.url.clone())
    }
}

impl Drop for HttpActionResultFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn handle_http_connection(mut stream: TcpStream, state: &Mutex<HttpMockState>) -> Result<(), String> {
    stream.set_nonblocking(false).map_err(|e| e.to_string())?;
    let mut buf = vec![0_u8; 65_536];
    let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }
    let raw = &buf[..n];
    let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Ok(());
    };
    let header_str = String::from_utf8_lossy(&raw[..header_end]);
    let mut lines = header_str.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let content_len = lines
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.eq_ignore_ascii_case("content-length") {
                v.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    let mut body = raw[header_end + 4..].to_vec();
    while body.len() < content_len {
        let mut extra = vec![0_u8; content_len - body.len()];
        let got = stream.read(&mut extra).map_err(|e| e.to_string())?;
        if got == 0 {
            break;
        }
        body.extend_from_slice(&extra[..got]);
    }
    body.truncate(content_len);

    let (status_line, resp_body) = {
        let mut guard = state.lock().map_err(|_| "http mock lock poisoned".to_string())?;
        if method == "PUT" {
            if guard.is_fail_index_puts && path.contains("/indexes/") {
                ("HTTP/1.1 500 Internal Server Error", b"injected-http-500".to_vec())
            } else {
                guard.resources.insert(path.to_string(), body);
                ("HTTP/1.1 200 OK", Vec::new())
            }
        } else if method == "GET" {
            match guard.resources.get(path) {
                Some(data) => ("HTTP/1.1 200 OK", data.clone()),
                None => ("HTTP/1.1 404 Not Found", Vec::new()),
            }
        } else {
            ("HTTP/1.1 405 Method Not Allowed", Vec::new())
        }
    };
    let header = format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", resp_body.len());
    stream.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&resp_body).map_err(|e| e.to_string())?;
    let _ = stream.flush();
    Ok(())
}

fn backend_for_profile(profile: StoreProfile) -> StoreBackend {
    match profile {
        StoreProfile::SnixSingle => StoreBackend::Snix,
        StoreProfile::CasitaSingle => StoreBackend::Casita,
    }
}

fn deterministic_keypair(seed_byte: u8, name: &str) -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
    let raw = ed25519_dalek::SigningKey::from_bytes(&[seed_byte; 32]);
    let verifying = VerifyingKey::new(name.to_string(), raw.verifying_key());
    let signing = SigningKey::new(name.to_string(), raw);
    (signing, verifying)
}

fn canonical_object_deps(object_idx: u8) -> &'static [u8] {
    match object_idx {
        0 => &[],
        1 => &[0],
        2 => &[],
        3 => &[0, 2],
        4 => &[1, 3],
        _ => &[],
    }
}

fn object_store_path(object_idx: u8, seed: u64) -> StorePath<String> {
    let hash = blake3::hash(format!("obj:{seed}:{object_idx}").as_bytes());
    let mut digest = [0_u8; 20];
    digest.copy_from_slice(&hash.as_bytes()[..20]);
    StorePath::from_name_and_digest_fixed(&format!("obj-{object_idx}"), digest).expect("valid store path")
}

fn object_payload(object_idx: u8, seed: u64) -> Vec<u8> {
    let base = format!("payload:{seed}:{object_idx}:");
    let mut bytes = base.into_bytes();
    bytes.resize(64 + usize::from(object_idx) * 17, b'x');
    bytes
}

fn typed_ref(prefix: &str, label: &str) -> String {
    format!("{prefix}{}", blake3::hash(label.as_bytes()).to_hex())
}

fn build_action_record_with_label(label: &str, path_info: &PathInfo) -> SignedActionResultRecord {
    let logical_path = path_info.store_path.to_absolute_path_with_prefix(TEST_STORE_DIR);
    let record = canonical_action_result(ActionResultRecordInput {
        action_ref: typed_ref(ACTION_REF_PREFIX, label),
        outputs: vec![ActionResultOutput {
            name: "out".to_string(),
            object_ref: typed_ref(OBJECT_REF_PREFIX, label),
            store_path: logical_path,
            path_info_ref: typed_ref(PATH_INFO_REF_PREFIX, label),
        }],
        action_receipt_ref: typed_ref(ACTION_RECEIPT_REF_PREFIX, label),
        reference_scan_refs: vec![typed_ref(REFERENCE_SCAN_REF_PREFIX, label)],
        sandbox_policy_ref: typed_ref(SANDBOX_POLICY_REF_PREFIX, "sandbox"),
        network_policy_ref: typed_ref(NETWORK_POLICY_REF_PREFIX, "network"),
        producer_identity: "hist-trusted-1".to_string(),
        producer_policy_ref: typed_ref(PRODUCER_POLICY_REF_PREFIX, "producer"),
        signature_refs: vec![typed_ref(SIGNATURE_REF_PREFIX, label)],
        publication_policy_ref: typed_ref(PUBLICATION_POLICY_REF_PREFIX, "publication"),
        non_claims: vec![
            "ca-mapping-presence-is-not-output-trust".to_string(),
            "executor-correctness".to_string(),
            "index-presence-is-not-output-trust".to_string(),
        ],
    })
    .expect("canonical action result");
    SignedActionResultRecord {
        record,
        record_signatures: vec![DetachedRecordSignature {
            key_name: "hist-trusted-1".to_string(),
            signature: "sig".to_string(),
        }],
    }
}

fn build_action_record(object_idx: u8, seed: u64, path_info: &PathInfo) -> SignedActionResultRecord {
    let label = format!("hist-{seed}-{object_idx}");
    build_action_record_with_label(&label, path_info)
}

struct PublishStepParams {
    step_index: usize,
    object_idx: u8,
    is_publish_action_result: bool,
    is_register_root: bool,
    fault: FaultInjectionMode,
    mutant: NegativeMutant,
}

struct HarnessEnv {
    _root: tempfile::TempDir,
    state_dir: PathBuf,
    output_dir: PathBuf,
    profile: StoreProfile,
    seed: u64,
    trusted_signing: SigningKey<ed25519_dalek::SigningKey>,
    trusted_verifying: VerifyingKey,
    untrusted_signing: SigningKey<ed25519_dalek::SigningKey>,
    http_fixture: HttpActionResultFixture,
    store: Option<StoreHandle>,
    model: StoreRefModel,
    last_plan_id: Option<String>,
    last_casita_plan_id: Option<String>,
    is_mutant_reached: bool,
}

impl HarnessEnv {
    async fn new(profile: StoreProfile, seed: u64) -> Result<Self, String> {
        let root = tempfile::Builder::new().prefix("h-").tempdir().map_err(|e| format!("tempdir: {e}"))?;
        let state_dir = root.path().join("s");
        let output_dir = root.path().join("o");
        std::fs::create_dir_all(&state_dir).map_err(|e| format!("mkdir state: {e}"))?;
        std::fs::create_dir_all(&output_dir).map_err(|e| format!("mkdir output: {e}"))?;

        let (trusted_signing, trusted_verifying) = deterministic_keypair(17, "hist-trusted-1");
        let (untrusted_signing, _) = deterministic_keypair(99, "hist-untrusted-1");

        if profile == StoreProfile::CasitaSingle {
            std::fs::write(state_dir.join("casita-trusted-public-keys"), format!("{trusted_verifying}\n"))
                .map_err(|e| format!("write casita keys: {e}"))?;
        }

        let http_fixture = HttpActionResultFixture::spawn()?;
        let mut env = Self {
            _root: root,
            state_dir,
            output_dir,
            profile,
            seed,
            trusted_signing,
            trusted_verifying,
            untrusted_signing,
            http_fixture,
            store: None,
            model: StoreRefModel::new(),
            last_plan_id: None,
            last_casita_plan_id: None,
            is_mutant_reached: false,
        };
        env.open_store().await?;
        Ok(env)
    }

    fn store_config(&self) -> StoreConfig {
        let mut cfg = StoreConfig::new(
            backend_for_profile(self.profile),
            self.state_dir.clone(),
            self.output_dir.clone(),
            TEST_STORE_DIR.to_string(),
        );
        cfg.source_admission = Some(Arc::new(TestHttpAdmission));
        cfg
    }

    async fn open_store(&mut self) -> Result<(), String> {
        self.store = None;
        let handle = StoreHandle::open(self.store_config()).await.map_err(|e| format!("open store: {e}"))?;
        self.store = Some(handle);
        Ok(())
    }

    fn drop_store(&mut self) {
        self.store = None;
    }

    fn path_for(&self, object_idx: u8) -> StorePath<String> {
        object_store_path(object_idx, self.seed)
    }

    async fn create_signed_node_and_pi(
        store: &StoreHandle,
        object_idx: u8,
        seed: u64,
        deps: &[StorePath<String>],
        signing_key: Option<&SigningKey<ed25519_dalek::SigningKey>>,
    ) -> Result<(Node, PathInfo), String> {
        let payload = object_payload(object_idx, seed);
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(&payload).await.map_err(|e| format!("write blob: {e}"))?;
        let digest = writer.close().await.map_err(|e| format!("close blob: {e}"))?;
        let node = Node::File {
            digest,
            size: u64::try_from(payload.len()).map_err(|_| "payload len".to_string())?,
            executable: false,
        };
        let (nar_size, nar_sha256) = SimpleRenderer::new(store.blob_service(), store.directory_service())
            .calculate_nar(&node)
            .await
            .map_err(|e| format!("calculate nar: {e}"))?;
        let sp = object_store_path(object_idx, seed);
        let mut signatures = Vec::new();
        if let Some(key) = signing_key {
            let ref_refs: Vec<StorePathRef<'_>> = deps.iter().map(StorePath::as_ref).collect();
            let fp = fingerprint_with_store_dir(&sp.as_ref(), &nar_sha256, nar_size, ref_refs.iter(), TEST_STORE_DIR);
            signatures.push(key.sign(fp.as_bytes()).to_owned());
        }
        let pi = PathInfo {
            store_path: sp,
            node: node.clone(),
            references: deps.to_vec(),
            nar_size,
            nar_sha256,
            signatures,
            deriver: None,
            ca: None,
        };
        Ok((node, pi))
    }

    async fn verify_invariants(&mut self, mutant: NegativeMutant) -> Result<(), (ViolationKind, String)> {
        if self.store.is_none()
            && let Err(err) = self.open_store().await
        {
            return Err((ViolationKind::AcknowledgedDurabilityLost, format!("reopen failed: {err}")));
        }
        if self.profile == StoreProfile::CasitaSingle
            && StoreHandle::casita_gc_fence_pending(&self.state_dir)
                .map_err(|e| (ViolationKind::AcknowledgedDurabilityLost, format!("fence check: {e}")))?
        {
            return Err((
                ViolationKind::FencedMutationNotBlocked,
                "unexpected pending Casita GC fence in model".to_string(),
            ));
        }

        let store = self.store.as_ref().expect("store open");
        let live = self.model.reachable_live_objects();
        for &obj_idx in &self.model.acknowledged_durable {
            if !live.contains(&obj_idx) && self.model.objects_present.contains(&obj_idx) {
                // Present unrooted object before GC is still expected to exist.
            }
        }
        let must_exist: BTreeSet<u8> = live.union(&self.model.acknowledged_durable).copied().collect();
        for obj_idx in must_exist {
            let sp = self.path_for(obj_idx);
            let got = store
                .pathinfo_service()
                .get(*sp.digest())
                .await
                .map_err(|e| (ViolationKind::AcknowledgedDurabilityLost, format!("pathinfo get: {e}")))?;
            let Some(pi) = got else {
                return Err((
                    ViolationKind::ProtectedRootLost,
                    format!("expected object {} ({}) missing from pathinfo service", obj_idx, sp),
                ));
            };
            let is_complete = store
                .castore_has_complete_content(&pi.node)
                .await
                .map_err(|e| (ViolationKind::ProtectedRootLost, format!("completeness check: {e}")))?;
            if !is_complete {
                return Err((
                    ViolationKind::ProtectedRootLost,
                    format!("object {} ({}) castore content incomplete", obj_idx, sp),
                ));
            }
            let expected_unsigned = self.model.unsigned_objects.contains(&obj_idx);
            if expected_unsigned && !pi.signatures.is_empty() {
                return Err((
                    ViolationKind::UnsignedTrustMutated,
                    format!("unsigned object {} gained unexpected signature", obj_idx),
                ));
            }
            if !expected_unsigned && self.profile == StoreProfile::CasitaSingle && pi.signatures.is_empty() {
                return Err((ViolationKind::UnsignedTrustMutated, format!("Casita object {} lost signature", obj_idx)));
            }
        }

        // Verify LocalActionResultStore: every discoverable hit must point to complete PathInfo.
        let local_action_store = LocalActionResultStore::new(&self.state_dir);
        for (&obj_idx, action_ref) in &self.model.action_records {
            let lookup = local_action_store
                .lookup(action_ref)
                .await
                .map_err(|e| (ViolationKind::IncompleteIndexExposed, format!("lookup failed: {e}")))?;
            for rec in &lookup.records {
                let sp = self.path_for(obj_idx);
                let pi = store
                    .pathinfo_service()
                    .get(*sp.digest())
                    .await
                    .map_err(|e| (ViolationKind::InvalidCacheHitDiscovered, format!("get: {e}")))?;
                let Some(pi) = pi else {
                    return Err((
                        ViolationKind::InvalidCacheHitDiscovered,
                        format!("discoverable action result {} has no backing PathInfo", rec.record.result_ref),
                    ));
                };
                let ok = store.castore_has_complete_content(&pi.node).await.unwrap_or(false);
                if !ok {
                    return Err((
                        ViolationKind::InvalidCacheHitDiscovered,
                        format!("discoverable action result {} has incomplete castore content", rec.record.result_ref),
                    ));
                }
            }
        }

        let _ = mutant;
        Ok(())
    }

    async fn step(
        &mut self,
        step_index: usize,
        op: &StoreOp,
        mutant: NegativeMutant,
        faults: &mut Vec<RequestedVsAppliedFault>,
    ) -> (StepClassification, Vec<String>) {
        match op {
            StoreOp::PublishSignedOutput {
                object_idx,
                is_publish_action_result,
                is_register_root,
                fault,
            } => {
                self.exec_publish(
                    PublishStepParams {
                        step_index,
                        object_idx: *object_idx,
                        is_publish_action_result: *is_publish_action_result,
                        is_register_root: *is_register_root,
                        fault: *fault,
                        mutant,
                    },
                    faults,
                )
                .await
            }
            StoreOp::LookupAndProbeActionResult {
                object_idx,
                is_verify_export,
            } => self.exec_lookup_and_probe(*object_idx, *is_verify_export).await,
            StoreOp::ImportCacheDir {
                object_idx,
                dep_indices,
                trust_mode,
            } => self.exec_import_cache_dir(*object_idx, dep_indices, *trust_mode).await,
            StoreOp::PinRoot {
                object_idx,
                owner_idx,
                reason_idx,
            } => self.exec_pin(*object_idx, *owner_idx, *reason_idx).await,
            StoreOp::UnpinRoot {
                object_idx,
                owner_idx,
                reason_idx,
            } => self.exec_unpin(*object_idx, *owner_idx, *reason_idx),
            StoreOp::RunGc { kind } => self.exec_gc(*kind, mutant).await,
            StoreOp::RecoverCasitaGc => self.exec_recover_casita().await,
            StoreOp::Reopen { kind } => self.exec_reopen(*kind).await,
            StoreOp::CompetingProcessMutation { is_wait_for_lock } => {
                self.exec_competing_mutation(*is_wait_for_lock).await
            }
        }
    }

    async fn exec_publish(
        &mut self,
        params: PublishStepParams,
        faults: &mut Vec<RequestedVsAppliedFault>,
    ) -> (StepClassification, Vec<String>) {
        let PublishStepParams {
            step_index,
            object_idx,
            is_publish_action_result,
            is_register_root,
            fault,
            mutant,
        } = params;

        let sp = self.path_for(object_idx);
        let sp_str = sp.to_absolute_path_with_prefix(TEST_STORE_DIR);
        let signing = self.trusted_signing.clone();
        let dep_indices = canonical_object_deps(object_idx);
        let dep_paths: Vec<StorePath<String>> = dep_indices.iter().map(|&d| self.path_for(d)).collect();
        // Collect transitive canonical dependencies in topological order
        // (leaves first) so references and GC reachability stay consistent.
        let mut all_deps = Vec::new();
        let mut seen_deps = BTreeSet::new();
        let mut stack: Vec<u8> = dep_indices.to_vec();
        while let Some(idx) = stack.pop() {
            if seen_deps.insert(idx) {
                all_deps.push(idx);
                stack.extend_from_slice(canonical_object_deps(idx));
            }
        }
        all_deps.sort_unstable();
        for &dep_idx in &all_deps {
            if !self.model.objects_present.contains(&dep_idx) {
                let dep_sp = self.path_for(dep_idx);
                let sub_deps: Vec<StorePath<String>> =
                    canonical_object_deps(dep_idx).iter().map(|&d| self.path_for(d)).collect();
                let (dep_node, dep_pi) = {
                    let store = self.store.as_ref().expect("store open");
                    match Self::create_signed_node_and_pi(store, dep_idx, self.seed, &sub_deps, Some(&signing)).await {
                        Ok(pair) => pair,
                        Err(e) => return (StepClassification::HarnessError { detail: e }, Vec::new()),
                    }
                };
                let store = self.store.as_mut().expect("store open");
                if let Err(err) = store
                    .persist_and_export_signed_output(PersistOutputRequest {
                        output_name: "out",
                        output_path: &dep_sp,
                        path_info: dep_pi,
                        final_node: dep_node,
                        provenance: None,
                        is_root: is_register_root,
                        root_source: if is_register_root {
                            Some(GcRootSource::Build)
                        } else {
                            None
                        },
                    })
                    .await
                {
                    let msg = err.to_string();
                    if msg.contains("gc-recovery-required") {
                        return (
                            StepClassification::ExpectedRejection {
                                code: "gc-recovery-required".to_string(),
                            },
                            Vec::new(),
                        );
                    }
                    return (StepClassification::HarnessError { detail: msg }, Vec::new());
                }
                self.model.objects_present.insert(dep_idx);
                self.model.unsigned_objects.remove(&dep_idx);
                self.model.acknowledged_durable.insert(dep_idx);
                self.model
                    .dag_refs
                    .entry(dep_idx)
                    .or_default()
                    .extend(canonical_object_deps(dep_idx).iter().copied());
                if is_register_root {
                    self.model.registered_roots.insert(dep_idx);
                }
            }
        }
        let (node, pi) = {
            let store = self.store.as_ref().expect("store open");
            match Self::create_signed_node_and_pi(store, object_idx, self.seed, &dep_paths, Some(&signing)).await {
                Ok(pair) => pair,
                Err(e) => return (StepClassification::HarnessError { detail: e }, Vec::new()),
            }
        };

        let store = self.store.as_mut().expect("store open");
        let persist_res = store
            .persist_and_export_signed_output(PersistOutputRequest {
                output_name: "out",
                output_path: &sp,
                path_info: pi.clone(),
                final_node: node,
                provenance: None,
                is_root: is_register_root,
                root_source: if is_register_root {
                    Some(GcRootSource::Build)
                } else {
                    None
                },
            })
            .await;

        if let Err(err) = persist_res {
            let msg = err.to_string();
            if msg.contains("gc-recovery-required") {
                return (
                    StepClassification::ExpectedRejection {
                        code: "gc-recovery-required".to_string(),
                    },
                    vec![sp_str],
                );
            }
            return (StepClassification::HarnessError { detail: msg }, vec![sp_str]);
        }

        self.model.objects_present.insert(object_idx);
        self.model.unsigned_objects.remove(&object_idx);
        self.model.acknowledged_durable.insert(object_idx);
        self.model.dag_refs.entry(object_idx).or_default().extend(dep_indices.iter().copied());
        if is_register_root {
            self.model.registered_roots.insert(object_idx);
        }

        let mut observed = vec![sp_str];
        if !is_publish_action_result {
            faults.push(RequestedVsAppliedFault {
                step_index,
                requested: fault,
                applied: "none".to_string(),
            });
            return (StepClassification::Passed, observed);
        }

        let record = build_action_record(object_idx, self.seed, &pi);
        let action_ref = record.record.action_ref.clone();
        let result_ref = record.record.result_ref.clone();
        observed.push(result_ref.clone());

        match fault {
            FaultInjectionMode::None => {
                let local_action = LocalActionResultStore::new(&self.state_dir);
                if let Err(e) = local_action.publish(&record).await {
                    return (StepClassification::HarnessError { detail: e }, observed);
                }
                if let Ok(http) = self.http_fixture.store() {
                    let _ = http.publish(&record).await;
                }
                self.model.action_records.insert(object_idx, action_ref);
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: "published-clean".to_string(),
                });
                (StepClassification::Passed, observed)
            }
            FaultInjectionMode::RecordOnlyInterruptedPublish => {
                // Simulate crash after writing record JSON without publishing the index marker.
                let records_dir = self.state_dir.join("action-results/v1/records");
                let _ = std::fs::create_dir_all(&records_dir);
                let digest = result_ref.strip_prefix(ACTION_RESULT_REF_PREFIX).expect("action result ref prefix");
                let record_path = records_dir.join(format!("{digest}.json"));
                if !record_path.exists() {
                    let _ = std::fs::write(&record_path, serde_json::to_vec(&record).unwrap_or_default());
                }

                if mutant == NegativeMutant::ExposeIncompleteIndex {
                    // Intentional semantic mutant: publish a canonical action-result
                    // record + index pointing to a missing PathInfo, testing that
                    // the independent oracle catches invalid discoverable hits.
                    let missing_idx = object_idx.saturating_add(20);
                    let missing_sp = self.path_for(missing_idx);
                    let mut bad_pi = pi.clone();
                    bad_pi.store_path = missing_sp;
                    let bad_rec = build_action_record(missing_idx, self.seed, &bad_pi);
                    let _ = LocalActionResultStore::new(&self.state_dir).publish(&bad_rec).await;
                    self.model.action_records.insert(missing_idx, bad_rec.record.action_ref);
                    self.is_mutant_reached = true;
                }

                let lookup = LocalActionResultStore::new(&self.state_dir).lookup(&action_ref).await;
                if !self.model.action_records.contains_key(&object_idx)
                    && let Ok(found) = lookup
                    && found.records.iter().any(|r| r.record.result_ref == result_ref)
                {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::IncompleteIndexExposed,
                            detail: format!("interrupted record-only publish was discoverable for {result_ref}"),
                        },
                        observed,
                    );
                }
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: "record-only-no-index".to_string(),
                });
                (StepClassification::Passed, observed)
            }
            FaultInjectionMode::DanglingIndexWithoutRecord => {
                let indexes_dir = self.state_dir.join("action-results/v1/indexes");
                let act_digest = action_ref.strip_prefix(ACTION_REF_PREFIX).expect("action ref prefix");
                let res_digest = result_ref.strip_prefix(ACTION_RESULT_REF_PREFIX).expect("action result ref prefix");
                let dangling_res_digest =
                    blake3::hash(format!("dangling-{act_digest}-{res_digest}-{step_index}").as_bytes())
                        .to_hex()
                        .to_string();
                let dangling_result_ref = format!("{ACTION_RESULT_REF_PREFIX}{dangling_res_digest}");
                let idx_dir = indexes_dir.join(act_digest);
                let _ = std::fs::create_dir_all(&idx_dir);
                let marker_path = idx_dir.join(format!("{dangling_res_digest}.ref"));
                let _ = std::fs::write(&marker_path, format!("{dangling_result_ref}\n"));
                let local_action = LocalActionResultStore::new(&self.state_dir);
                let lookup = local_action.lookup(&action_ref).await;
                let _ = std::fs::remove_file(&marker_path);
                let _ = std::fs::remove_dir(&idx_dir);
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: "dangling-index-checked".to_string(),
                });
                match lookup {
                    Err(_) => (
                        StepClassification::ExpectedRejection {
                            code: "dangling-index-rejected".to_string(),
                        },
                        observed,
                    ),
                    Ok(found) if !found.records.iter().any(|r| r.record.result_ref == dangling_result_ref) => {
                        (StepClassification::Passed, observed)
                    }
                    Ok(_) => (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::IncompleteIndexExposed,
                            detail: "dangling index returned a hit without record".to_string(),
                        },
                        observed,
                    ),
                }
            }
            FaultInjectionMode::ConflictingRecordBytes => {
                let local_action = LocalActionResultStore::new(&self.state_dir);
                let _ = local_action.publish(&record).await;
                let records_dir = self.state_dir.join("action-results/v1/records");
                let digest = result_ref.strip_prefix(ACTION_RESULT_REF_PREFIX).expect("action result ref prefix");
                let record_path = records_dir.join(format!("{digest}.json"));
                let mut mutated = record.clone();
                mutated.record_signatures[0].signature = "different-sig".to_string();
                let conflict_res = local_action.publish(&mutated).await;
                let on_disk = std::fs::read(&record_path).unwrap_or_default();
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: "no-clobber-conflict-checked".to_string(),
                });
                if conflict_res.is_ok() || on_disk == serde_json::to_vec(&mutated).unwrap_or_default() {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::NoClobberBroken,
                            detail: "conflicting record overwrote existing record".to_string(),
                        },
                        observed,
                    );
                }
                self.model.action_records.insert(object_idx, action_ref);
                (
                    StepClassification::ExpectedRejection {
                        code: "no-clobber-conflict".to_string(),
                    },
                    observed,
                )
            }
            FaultInjectionMode::HttpIndexPut500 => {
                let fault_label = format!("hist-{}-{object_idx}-http500-{step_index}", self.seed);
                let fault_record = build_action_record_with_label(&fault_label, &pi);
                if let Ok(mut guard) = self.http_fixture.state.lock() {
                    guard.is_fail_index_puts = true;
                }
                let http_store = self.http_fixture.store();
                let res = match &http_store {
                    Ok(http) => http.publish(&fault_record).await,
                    Err(e) => Err(e.clone()),
                };
                if let Ok(mut guard) = self.http_fixture.state.lock() {
                    guard.is_fail_index_puts = false;
                }
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: "http-500-index-put".to_string(),
                });
                match res {
                    Err(msg) if msg.contains("500") => {
                        if let Ok(http) = http_store
                            && let Ok(lookup) = http.lookup(&fault_record.record.action_ref).await
                            && !lookup.records.is_empty()
                        {
                            return (
                                StepClassification::SafetyViolation {
                                    kind: ViolationKind::IncompleteIndexExposed,
                                    detail: "HTTP 500 index PUT left action result discoverable".to_string(),
                                },
                                observed,
                            );
                        }
                        (
                            StepClassification::ExpectedRejection {
                                code: "http-500".to_string(),
                            },
                            observed,
                        )
                    }
                    Err(msg) => (StepClassification::HarnessError { detail: msg }, observed),
                    Ok(_) => (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::IncompleteIndexExposed,
                            detail: "HTTP 500 index PUT falsely reported success".to_string(),
                        },
                        observed,
                    ),
                }
            }
            FaultInjectionMode::AbruptChildAbort => {
                let abort_label = format!("hist-{}-{object_idx}-abort-{step_index}", self.seed);
                let abort_record = build_action_record_with_label(&abort_label, &pi);
                let abort_action_ref = abort_record.record.action_ref.clone();
                self.drop_store();
                let partial_path =
                    self.state_dir.join(format!("action-results/v1/records/aborted-{step_index}.json.tmp-child"));
                let resp_path = self._root.path().join(format!("child-abort-{step_index}.json"));
                let req = ChildRequest {
                    profile: self.profile,
                    state_dir: self.state_dir.clone(),
                    output_dir: self.output_dir.clone(),
                    store_dir: TEST_STORE_DIR.to_string(),
                    response_path: resp_path,
                    action: ChildAction::WriteActionRecordAndAbort {
                        action_ref: abort_action_ref.clone(),
                        record_json: serde_json::to_string(&abort_record).unwrap_or_default(),
                        partial_record_path: partial_path.clone(),
                        partial_bytes: b"{aborted".to_vec(),
                    },
                };
                let status = run_child_expect_status(&req, false);
                if !status {
                    return (
                        StepClassification::HarnessError {
                            detail: "aborted child unexpectedly exited successfully".to_string(),
                        },
                        observed,
                    );
                }
                match StoreMutationGuard::try_acquire_for(&self.state_dir, "post-abort-verify") {
                    Ok(guard) => drop(guard),
                    Err(err) => {
                        return (
                            StepClassification::SafetyViolation {
                                kind: ViolationKind::FencedMutationNotBlocked,
                                detail: format!("lock not released after child SIGABRT: {err}"),
                            },
                            observed,
                        );
                    }
                }
                if let Err(e) = self.open_store().await {
                    return (StepClassification::HarnessError { detail: e }, observed);
                }
                faults.push(RequestedVsAppliedFault {
                    step_index,
                    requested: fault,
                    applied: format!("child-sigabrt:{status}"),
                });
                if !partial_path.exists() {
                    return (
                        StepClassification::HarnessError {
                            detail: "aborted child did not write partial marker before SIGABRT".to_string(),
                        },
                        observed,
                    );
                }
                if let Ok(lookup) = LocalActionResultStore::new(&self.state_dir).lookup(&abort_action_ref).await
                    && !lookup.records.is_empty()
                {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::IncompleteIndexExposed,
                            detail: "aborted child unindexed record was discoverable".to_string(),
                        },
                        observed,
                    );
                }
                (StepClassification::Passed, observed)
            }
        }
    }

    async fn exec_lookup_and_probe(
        &mut self,
        object_idx: u8,
        is_verify_export: bool,
    ) -> (StepClassification, Vec<String>) {
        let Some(action_ref) = self.model.action_records.get(&object_idx).cloned() else {
            return (
                StepClassification::IncompleteOrBlocked {
                    reason: "no-published-action-record".to_string(),
                },
                Vec::new(),
            );
        };
        let store = self.store.as_ref().expect("store open");
        let report = store.discover_action_results(&action_ref).await;
        let sp = self.path_for(object_idx);
        let sp_str = sp.to_absolute_path_with_prefix(TEST_STORE_DIR);
        if report.lookups.iter().all(|l| l.records.is_empty()) {
            if self.model.objects_present.contains(&object_idx) {
                return (
                    StepClassification::SafetyViolation {
                        kind: ViolationKind::AcknowledgedDurabilityLost,
                        detail: format!("action result for {sp_str} disappeared"),
                    },
                    vec![sp_str],
                );
            }
            return (StepClassification::Passed, vec![sp_str]);
        }
        if is_verify_export {
            let filter = sp.to_string();
            let verified = store_verify(store.pathinfo_service().as_ref(), Some(&filter), &self.output_dir).await;
            match verified {
                Ok(results) => {
                    for item in results {
                        if let crunch_store::VerifyResult::Mismatch { path, .. } = item {
                            return (
                                StepClassification::SafetyViolation {
                                    kind: ViolationKind::InvalidCacheHitDiscovered,
                                    detail: format!("NAR verify mismatch for {path}"),
                                },
                                vec![sp_str],
                            );
                        }
                    }
                }
                Err(e) => return (StepClassification::HarnessError { detail: e.to_string() }, vec![sp_str]),
            }
        }
        (StepClassification::Passed, vec![sp_str])
    }

    async fn exec_import_cache_dir(
        &mut self,
        object_idx: u8,
        dep_indices: &[u8],
        trust_mode: TrustAdmissionMode,
    ) -> (StepClassification, Vec<String>) {
        let sp = self.path_for(object_idx);
        let sp_str = sp.to_absolute_path_with_prefix(TEST_STORE_DIR);

        // Build a staging Snix store to export valid nar + narinfo into a cache dir.
        let stage_dir = self._root.path().join(format!("st-{object_idx}-{}", dep_indices.len()));
        let stage_state = stage_dir.join("s");
        let stage_out = stage_dir.join("o");
        let cache_dir = stage_dir.join("cache");
        let _ = std::fs::remove_dir_all(&stage_dir);
        let stage_cfg = StoreConfig::new(StoreBackend::Snix, stage_state, stage_out, TEST_STORE_DIR.to_string());
        let stage_store = match StoreHandle::open(stage_cfg).await {
            Ok(s) => s,
            Err(e) => return (StepClassification::HarnessError { detail: e.to_string() }, vec![sp_str]),
        };

        let signing_opt = match trust_mode {
            TrustAdmissionMode::SignedTrusted => Some(&self.trusted_signing),
            TrustAdmissionMode::SignedUntrusted => Some(&self.untrusted_signing),
            TrustAdmissionMode::Unsigned => None,
        };
        let mut to_export = Vec::new();
        let mut exported_set = BTreeSet::new();
        let mut stack: Vec<u8> = dep_indices.to_vec();
        let mut all_deps = Vec::new();
        while let Some(idx) = stack.pop() {
            if exported_set.insert(idx) {
                all_deps.push(idx);
                stack.extend_from_slice(canonical_object_deps(idx));
            }
        }
        all_deps.sort_unstable();
        for &dep_idx in &all_deps {
            let sub_deps: Vec<StorePath<String>> =
                canonical_object_deps(dep_idx).iter().map(|&d| self.path_for(d)).collect();
            let Ok((_, dep_pi)) =
                Self::create_signed_node_and_pi(&stage_store, dep_idx, self.seed, &sub_deps, signing_opt).await
            else {
                return (
                    StepClassification::HarnessError {
                        detail: "build dep pi".to_string(),
                    },
                    vec![sp_str],
                );
            };
            let _ = stage_store.pathinfo_service().put(dep_pi.clone()).await;
            to_export.push(dep_pi);
        }
        let dep_paths: Vec<StorePath<String>> = dep_indices.iter().map(|&d| self.path_for(d)).collect();
        let Ok((_, root_pi)) =
            Self::create_signed_node_and_pi(&stage_store, object_idx, self.seed, &dep_paths, signing_opt).await
        else {
            return (
                StepClassification::HarnessError {
                    detail: "build root pi".to_string(),
                },
                vec![sp_str],
            );
        };
        let _ = stage_store.pathinfo_service().put(root_pi.clone()).await;
        to_export.push(root_pi);

        if let Err(e) =
            export_paths_to_cache_dir(&stage_store, &to_export, &cache_dir, &PushOptions { trust_unsigned: true }).await
        {
            return (StepClassification::HarnessError { detail: e.to_string() }, vec![sp_str]);
        }
        drop(stage_store);

        let pull_opts = PullOptions {
            trust_unsigned: matches!(trust_mode, TrustAdmissionMode::Unsigned),
            trusted_public_keys: vec![self.trusted_verifying.clone()],
        };
        let store = self.store.as_ref().expect("store open");
        let pull_res = import_paths_from_cache_dir(store, &cache_dir, None, &pull_opts).await;

        let is_expected_allowed = matches!(
            (self.profile, trust_mode),
            (StoreProfile::SnixSingle, TrustAdmissionMode::SignedTrusted | TrustAdmissionMode::Unsigned)
                | (StoreProfile::CasitaSingle, TrustAdmissionMode::SignedTrusted)
        );
        match pull_res {
            Ok(report) => {
                if report.imported_count == 0 && report.skipped_untrusted_count > 0 {
                    return (
                        StepClassification::ExpectedRejection {
                            code: "untrusted-import-rejected".to_string(),
                        },
                        vec![sp_str],
                    );
                }
                if report.imported_count == 0 && report.skipped_already_present_count > 0 {
                    return (StepClassification::Passed, vec![sp_str]);
                }
                if !is_expected_allowed {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::UnsignedTrustMutated,
                            detail: format!("profile {:?} unexpectedly admitted {:?}", self.profile, trust_mode),
                        },
                        vec![sp_str],
                    );
                }
                for &dep_idx in &all_deps {
                    let is_new = self.model.objects_present.insert(dep_idx);
                    if is_new && trust_mode == TrustAdmissionMode::Unsigned {
                        self.model.unsigned_objects.insert(dep_idx);
                    }
                    self.model
                        .dag_refs
                        .entry(dep_idx)
                        .or_default()
                        .extend(canonical_object_deps(dep_idx).iter().copied());
                }
                let is_new_root = self.model.objects_present.insert(object_idx);
                if is_new_root && trust_mode == TrustAdmissionMode::Unsigned {
                    self.model.unsigned_objects.insert(object_idx);
                }
                self.model.dag_refs.entry(object_idx).or_default().extend(dep_indices.iter().copied());
                self.model.acknowledged_durable.insert(object_idx);
                (StepClassification::Passed, vec![sp_str])
            }
            Err(err) => {
                if is_expected_allowed {
                    return (
                        StepClassification::HarnessError {
                            detail: format!("unexpected import failure: {err}"),
                        },
                        vec![sp_str],
                    );
                }
                (
                    StepClassification::ExpectedRejection {
                        code: "untrusted-import-rejected".to_string(),
                    },
                    vec![sp_str],
                )
            }
        }
    }

    async fn exec_pin(&mut self, object_idx: u8, owner_idx: u8, reason_idx: u8) -> (StepClassification, Vec<String>) {
        let sp = self.path_for(object_idx);
        let logical = sp.to_absolute_path_with_prefix(TEST_STORE_DIR);
        let owner = format!("owner-{owner_idx}");
        let reason = format!("reason-{reason_idx}");
        let store = self.store.as_ref().expect("store open");
        let res = store.pin_retained_root(&logical, &owner, &reason).await;
        if !self.model.objects_present.contains(&object_idx) {
            return match res {
                Err(_) => (
                    StepClassification::ExpectedRejection {
                        code: "pin-missing-path".to_string(),
                    },
                    vec![logical],
                ),
                Ok(_) => (
                    StepClassification::SafetyViolation {
                        kind: ViolationKind::ProtectedRootLost,
                        detail: "pin succeeded for absent path".to_string(),
                    },
                    vec![logical],
                ),
            };
        }
        match res {
            Ok(_) => {
                self.model.active_pins.insert((object_idx, owner_idx, reason_idx), ());
                (StepClassification::Passed, vec![logical])
            }
            Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, vec![logical]),
        }
    }

    fn exec_unpin(&mut self, object_idx: u8, owner_idx: u8, reason_idx: u8) -> (StepClassification, Vec<String>) {
        let sp = self.path_for(object_idx);
        let logical = sp.to_absolute_path_with_prefix(TEST_STORE_DIR);
        let owner = format!("owner-{owner_idx}");
        let reason = format!("reason-{reason_idx}");
        let store = self.store.as_ref().expect("store open");
        let res = store.unpin_retained_root(&logical, &owner, Some(&reason));
        match res {
            Ok(_) => {
                self.model.active_pins.remove(&(object_idx, owner_idx, reason_idx));
                (StepClassification::Passed, vec![logical])
            }
            Err(e) if e.to_string().contains("foreign-owner") => (
                StepClassification::ExpectedRejection {
                    code: "foreign-owner-release-denied".to_string(),
                },
                vec![logical],
            ),
            Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, vec![logical]),
        }
    }

    async fn exec_gc(&mut self, kind: GcExecutionKind, mutant: NegativeMutant) -> (StepClassification, Vec<String>) {
        let mut observed = Vec::new();
        let guard = match StoreMutationGuard::try_acquire_for(&self.state_dir, "history-gc") {
            Ok(g) => g,
            Err(e) => return (StepClassification::HarnessError { detail: e.to_string() }, observed),
        };
        match kind {
            GcExecutionKind::DryRunPlanOnly => {
                let store = self.store.as_mut().expect("store open");
                let res = match self.profile {
                    StoreProfile::SnixSingle => store.garbage_collect(None).await,
                    StoreProfile::CasitaSingle => store.garbage_collect_under_guard(&guard, None).await,
                };
                match res {
                    Ok(report) => {
                        observed.push(report.plan_id.clone());
                        self.last_plan_id = Some(report.plan_id.clone());
                        if self.profile == StoreProfile::CasitaSingle {
                            self.last_casita_plan_id = Some(report.plan_id.clone());
                        }
                        self.model.last_plan = Some(ModelPlanSnapshot {
                            plan_id: report.plan_id,
                            root_snapshot: self.model.reachable_live_objects(),
                        });
                        (StepClassification::Passed, observed)
                    }
                    Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, observed),
                }
            }
            GcExecutionKind::ExecuteUnguarded => {
                let store = self.store.as_mut().expect("store open");
                if self.profile == StoreProfile::CasitaSingle {
                    let res = store.garbage_collect(None).await;
                    return match res {
                        Err(e) if e.to_string().contains("casita-gc-guard-required") => (
                            StepClassification::ExpectedRejection {
                                code: "casita-gc-guard-required".to_string(),
                            },
                            observed,
                        ),
                        _ => (
                            StepClassification::SafetyViolation {
                                kind: ViolationKind::FencedMutationNotBlocked,
                                detail: "unguarded Casita GC was not rejected".to_string(),
                            },
                            observed,
                        ),
                    };
                }
                // On Snix, unguarded dry-run plan still works cleanly.
                match store.garbage_collect(None).await {
                    Ok(report) => {
                        self.last_plan_id = Some(report.plan_id.clone());
                        observed.push(report.plan_id);
                        (StepClassification::Passed, observed)
                    }
                    Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, observed),
                }
            }
            GcExecutionKind::ExecuteLatestPlan => {
                let plan_id = match self.last_plan_id.clone() {
                    Some(id) => id,
                    None => {
                        let store = self.store.as_mut().expect("store open");
                        let plan = match self.profile {
                            StoreProfile::SnixSingle => store.garbage_collect(None).await,
                            StoreProfile::CasitaSingle => store.garbage_collect_under_guard(&guard, None).await,
                        };
                        match plan {
                            Ok(r) => r.plan_id,
                            Err(e) => return (StepClassification::HarnessError { detail: e.to_string() }, observed),
                        }
                    }
                };
                observed.push(plan_id.clone());
                let store = self.store.as_mut().expect("store open");
                let res = match self.profile {
                    StoreProfile::SnixSingle => store.garbage_collect(Some(&plan_id)).await,
                    StoreProfile::CasitaSingle => store.garbage_collect_under_guard(&guard, Some(&plan_id)).await,
                };
                match res {
                    Ok(_report) => {
                        self.model.apply_gc_collection();
                        self.last_plan_id = None;
                        if self.profile == StoreProfile::SnixSingle {
                            // Snix GC atomically renames pathinfo.redb / directories.redb;
                            // reopen the handle so subsequent operations target the new DB inodes.
                            let _ = store;
                            self.drop_store();
                            if let Err(e) = self.open_store().await {
                                return (StepClassification::HarnessError { detail: e }, observed);
                            }
                        }
                        if mutant == NegativeMutant::LoseProtectedReference {
                            let live = self.model.reachable_live_objects();
                            if let Some(&first_live) = live.iter().next() {
                                let _sp = self.path_for(first_live);
                                // Intentional mutant: drop a rooted object via Casita root removal or Snix rewrite!
                                let _ = std::fs::remove_dir_all(self.state_dir.join("casita"));
                                if self.profile == StoreProfile::SnixSingle {
                                    // Corrupt Snix by removing pathinfo DB after dropping store.
                                    self.drop_store();
                                    let _ = std::fs::remove_file(self.state_dir.join("pathinfo.redb"));
                                    let _ = self.open_store().await;
                                } else {
                                    self.drop_store();
                                    let _ = self.open_store().await;
                                }
                                self.is_mutant_reached = true;
                            }
                        }
                        (StepClassification::Passed, observed)
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("stale") || msg.contains("gc-plan-stale") {
                            return (
                                StepClassification::ExpectedRejection {
                                    code: "stale-gc-plan".to_string(),
                                },
                                observed,
                            );
                        }
                        (StepClassification::HarnessError { detail: msg }, observed)
                    }
                }
            }
            GcExecutionKind::ExecuteStalePlanAfterRootChange => {
                let Some(stale_id) = self.last_plan_id.clone() else {
                    return (
                        StepClassification::IncompleteOrBlocked {
                            reason: "no-prior-gc-plan".to_string(),
                        },
                        observed,
                    );
                };
                // Mutate a root by pinning/publishing a fresh object if possible so the plan becomes stale.
                let fresh_idx = 5_u8;
                let sp = self.path_for(fresh_idx);
                let signing = self.trusted_signing.clone();
                if let Ok((node, pi)) = Self::create_signed_node_and_pi(
                    self.store.as_ref().expect("open"),
                    fresh_idx,
                    self.seed,
                    &[],
                    Some(&signing),
                )
                .await
                {
                    let store = self.store.as_mut().expect("open");
                    if store
                        .persist_and_export_signed_output(PersistOutputRequest {
                            output_name: "out",
                            output_path: &sp,
                            path_info: pi,
                            final_node: node,
                            provenance: None,
                            is_root: true,
                            root_source: Some(GcRootSource::Pin),
                        })
                        .await
                        .is_ok()
                    {
                        self.model.objects_present.insert(fresh_idx);
                        self.model.registered_roots.insert(fresh_idx);
                        self.model.acknowledged_durable.insert(fresh_idx);
                    }
                }
                let store = self.store.as_mut().expect("open");
                let res = match self.profile {
                    StoreProfile::SnixSingle => store.garbage_collect(Some(&stale_id)).await,
                    StoreProfile::CasitaSingle => store.garbage_collect_under_guard(&guard, Some(&stale_id)).await,
                };
                observed.push(stale_id);
                match res {
                    Err(e) if e.to_string().contains("stale") => (
                        StepClassification::ExpectedRejection {
                            code: "stale-gc-plan".to_string(),
                        },
                        observed,
                    ),
                    Ok(_) => {
                        // Plan might still match if object 5 was already in the exact same root state.
                        self.model.apply_gc_collection();
                        (StepClassification::Passed, observed)
                    }
                    Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, observed),
                }
            }
        }
    }

    async fn exec_recover_casita(&mut self) -> (StepClassification, Vec<String>) {
        if self.profile != StoreProfile::CasitaSingle {
            return (
                StepClassification::IncompleteOrBlocked {
                    reason: "snix-has-no-casita-fence".to_string(),
                },
                Vec::new(),
            );
        }
        let Ok(guard) = StoreMutationGuard::try_acquire_for(&self.state_dir, "recover-casita") else {
            return (
                StepClassification::HarnessError {
                    detail: "lock busy during casita recovery".to_string(),
                },
                Vec::new(),
            );
        };
        let store = self.store.as_mut().expect("store open");
        match store.recover_casita_gc_under_guard(&guard).await {
            Ok(()) => match StoreHandle::casita_gc_fence_pending(&self.state_dir) {
                Ok(false) => (StepClassification::Passed, Vec::new()),
                Ok(true) => (
                    StepClassification::SafetyViolation {
                        kind: ViolationKind::FencedMutationNotBlocked,
                        detail: "Casita GC fence still pending after recovery".to_string(),
                    },
                    Vec::new(),
                ),
                Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, Vec::new()),
            },
            Err(e) => (StepClassification::HarnessError { detail: e.to_string() }, Vec::new()),
        }
    }

    async fn exec_reopen(&mut self, kind: ReopenKind) -> (StepClassification, Vec<String>) {
        match kind {
            ReopenKind::InProcessReopen => {
                self.drop_store();
                match self.open_store().await {
                    Ok(()) => (StepClassification::Passed, Vec::new()),
                    Err(e) => (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::AcknowledgedDurabilityLost,
                            detail: format!("in-process reopen failed: {e}"),
                        },
                        Vec::new(),
                    ),
                }
            }
            ReopenKind::FreshChildProcessReopen => {
                if self.profile == StoreProfile::SnixSingle {
                    self.drop_store();
                }
                let live: Vec<String> = self
                    .model
                    .reachable_live_objects()
                    .union(&self.model.acknowledged_durable)
                    .map(|&idx| self.path_for(idx).to_absolute_path_with_prefix(TEST_STORE_DIR))
                    .collect();
                let unsigned: Vec<String> = self
                    .model
                    .unsigned_objects
                    .iter()
                    .map(|&idx| self.path_for(idx).to_absolute_path_with_prefix(TEST_STORE_DIR))
                    .collect();
                let action_refs: Vec<(String, String)> = self
                    .model
                    .action_records
                    .iter()
                    .map(|(&idx, aref)| (self.path_for(idx).to_absolute_path_with_prefix(TEST_STORE_DIR), aref.clone()))
                    .collect();
                let resp_path = self._root.path().join("child-verify.json");
                let req = ChildRequest {
                    profile: self.profile,
                    state_dir: self.state_dir.clone(),
                    output_dir: self.output_dir.clone(),
                    store_dir: TEST_STORE_DIR.to_string(),
                    response_path: resp_path.clone(),
                    action: ChildAction::VerifyReopen {
                        expected_live_paths: live,
                        expected_unsigned_paths: unsigned,
                        expected_action_refs: action_refs,
                    },
                };
                let status = run_child_expect_status(&req, true);
                if self.profile == StoreProfile::SnixSingle
                    && let Err(e) = self.open_store().await
                {
                    return (StepClassification::HarnessError { detail: e }, Vec::new());
                }
                if !status {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::AcknowledgedDurabilityLost,
                            detail: "fresh child reopen verification process failed".to_string(),
                        },
                        Vec::new(),
                    );
                }
                let report: ChildReport =
                    match std::fs::read_to_string(&resp_path).ok().and_then(|s| serde_json::from_str(&s).ok()) {
                        Some(r) => r,
                        None => {
                            return (
                                StepClassification::HarnessError {
                                    detail: "missing child verify response".to_string(),
                                },
                                Vec::new(),
                            );
                        }
                    };
                if !report.is_ok {
                    return (
                        StepClassification::SafetyViolation {
                            kind: ViolationKind::AcknowledgedDurabilityLost,
                            detail: report.detail,
                        },
                        report.observed_paths,
                    );
                }
                (StepClassification::Passed, report.observed_paths)
            }
        }
    }

    async fn exec_competing_mutation(&mut self, is_wait_for_lock: bool) -> (StepClassification, Vec<String>) {
        let guard = match StoreMutationGuard::try_acquire_for(&self.state_dir, "parent-history") {
            Ok(g) => g,
            Err(e) => return (StepClassification::HarnessError { detail: e.to_string() }, Vec::new()),
        };
        let resp_path = self._root.path().join("child-race.json");
        let req = ChildRequest {
            profile: self.profile,
            state_dir: self.state_dir.clone(),
            output_dir: self.output_dir.clone(),
            store_dir: TEST_STORE_DIR.to_string(),
            response_path: resp_path.clone(),
            action: ChildAction::RaceMutationGuard {
                role: "competing-child".to_string(),
                is_expect_busy: !is_wait_for_lock,
                is_wait_for_lock,
            },
        };
        if is_wait_for_lock {
            let mut child = spawn_child(&req);
            tokio::time::sleep(Duration::from_millis(60)).await;
            drop(guard);
            let ok = child.wait().map(|s| s.success()).unwrap_or(false);
            if !ok {
                return (
                    StepClassification::SafetyViolation {
                        kind: ViolationKind::FencedMutationNotBlocked,
                        detail: "competing child failed lock handoff".to_string(),
                    },
                    Vec::new(),
                );
            }
            return (StepClassification::Passed, Vec::new());
        }
        let ok = run_child_expect_status(&req, true);
        drop(guard);
        if !ok {
            return (
                StepClassification::SafetyViolation {
                    kind: ViolationKind::FencedMutationNotBlocked,
                    detail: "competing child did not observe busy StoreMutationGuard".to_string(),
                },
                Vec::new(),
            );
        }
        (StepClassification::Passed, Vec::new())
    }
}

fn spawn_child(req: &ChildRequest) -> std::process::Child {
    let req_path = req.response_path.parent().unwrap_or_else(|| Path::new("/tmp")).join("child-req.json");
    std::fs::write(&req_path, serde_json::to_vec(req).expect("serialize req")).expect("write child req");
    let exe = std::env::current_exe().expect("current_exe");
    Command::new(exe)
        .arg(CHILD_ENTRY_TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .env(CHILD_REQ_ENV, req_path)
        .spawn()
        .expect("spawn child")
}

fn run_child_expect_status(req: &ChildRequest, expect_success: bool) -> bool {
    let output = {
        let req_path = req.response_path.parent().unwrap_or_else(|| Path::new("/tmp")).join("child-req.json");
        std::fs::write(&req_path, serde_json::to_vec(req).expect("serialize req")).expect("write child req");
        let exe = std::env::current_exe().expect("current_exe");
        Command::new(exe)
            .arg(CHILD_ENTRY_TEST_NAME)
            .arg("--exact")
            .arg("--nocapture")
            .env(CHILD_REQ_ENV, req_path)
            .output()
            .expect("run child")
    };
    if expect_success {
        output.status.success()
    } else {
        !output.status.success()
    }
}

pub async fn run_child_entry_if_configured() {
    let Ok(req_path) = std::env::var(CHILD_REQ_ENV) else {
        return;
    };
    let raw = std::fs::read(&req_path).expect("read child req");
    let req: ChildRequest = serde_json::from_slice(&raw).expect("parse child req");
    match &req.action {
        ChildAction::VerifyReopen {
            expected_live_paths,
            expected_unsigned_paths,
            expected_action_refs,
        } => {
            let mut cfg = StoreConfig::new(
                backend_for_profile(req.profile),
                req.state_dir.clone(),
                req.output_dir.clone(),
                req.store_dir.clone(),
            );
            cfg.source_admission = Some(Arc::new(TestHttpAdmission));
            let report = match StoreHandle::open(cfg).await {
                Err(e) => ChildReport {
                    is_ok: false,
                    detail: format!("child open failed: {e}"),
                    observed_paths: Vec::new(),
                },
                Ok(store) => {
                    verify_child_store_state(
                        &store,
                        &req,
                        expected_live_paths,
                        expected_unsigned_paths,
                        expected_action_refs,
                    )
                    .await
                }
            };
            std::fs::write(&req.response_path, serde_json::to_vec(&report).expect("encode child report"))
                .expect("write child report");
            assert!(report.is_ok, "child verification failed: {}", report.detail);
        }
        ChildAction::WriteActionRecordAndAbort {
            action_ref,
            record_json,
            partial_record_path,
            partial_bytes,
        } => {
            let _guard =
                StoreMutationGuard::try_acquire_for(&req.state_dir, "aborted-child-publish").expect("abort child lock");
            let mut cfg = StoreConfig::new(
                backend_for_profile(req.profile),
                req.state_dir.clone(),
                req.output_dir.clone(),
                req.store_dir.clone(),
            );
            cfg.source_admission = Some(Arc::new(TestHttpAdmission));
            let _store = StoreHandle::open(cfg).await.expect("abort child open store");
            let parsed: SignedActionResultRecord = serde_json::from_str(record_json).expect("parse abort record json");
            let result_digest =
                parsed.record.result_ref.strip_prefix(ACTION_RESULT_REF_PREFIX).expect("result ref prefix");
            let records_dir = req.state_dir.join("action-results/v1/records");
            std::fs::create_dir_all(&records_dir).expect("create records dir");
            let canonical_record_path = records_dir.join(format!("{result_digest}.json"));
            std::fs::write(&canonical_record_path, record_json).expect("write unindexed record");
            if let Some(parent) = partial_record_path.parent() {
                std::fs::create_dir_all(parent).expect("create partial parent dir");
            }
            std::fs::write(partial_record_path, partial_bytes).expect("write partial tmp marker");
            let lookup =
                LocalActionResultStore::new(&req.state_dir).lookup(action_ref).await.expect("lookup in abort child");
            assert!(lookup.records.is_empty(), "unindexed record must not be discoverable before abort");
            std::process::abort();
        }
        ChildAction::RaceMutationGuard {
            role,
            is_expect_busy,
            is_wait_for_lock,
        } => {
            if *is_expect_busy {
                let err = StoreMutationGuard::try_acquire_for(&req.state_dir, role)
                    .expect_err("expected lock to be held by parent");
                assert!(err.to_string().contains("active"), "{err}");
            } else if *is_wait_for_lock {
                let guard = StoreMutationGuard::acquire_wait_bounded(&req.state_dir, role, 5_000)
                    .expect("expected lock handoff after parent drop");
                drop(guard);
            }
        }
    }
}

async fn verify_child_store_state(
    store: &StoreHandle,
    req: &ChildRequest,
    expected_live_paths: &[String],
    expected_unsigned_paths: &[String],
    expected_action_refs: &[(String, String)],
) -> ChildReport {
    let mut observed = Vec::new();
    let unsigned_set: BTreeSet<&str> = expected_unsigned_paths.iter().map(String::as_str).collect();
    for logical in expected_live_paths {
        let Ok(sp) = StorePath::<String>::from_absolute_path_with_prefix(logical.as_bytes(), &req.store_dir) else {
            return ChildReport {
                is_ok: false,
                detail: format!("invalid path {logical}"),
                observed_paths: observed,
            };
        };
        match store.pathinfo_service().get(*sp.digest()).await {
            Ok(Some(pi)) => {
                let is_complete = store.castore_has_complete_content(&pi.node).await.unwrap_or(false);
                if !is_complete {
                    return ChildReport {
                        is_ok: false,
                        detail: format!("incomplete castore content for {logical}"),
                        observed_paths: observed,
                    };
                }
                if unsigned_set.contains(logical.as_str()) && !pi.signatures.is_empty() {
                    return ChildReport {
                        is_ok: false,
                        detail: format!("unsigned object {logical} gained signature"),
                        observed_paths: observed,
                    };
                }
                observed.push(logical.clone());
            }
            Ok(None) => {
                return ChildReport {
                    is_ok: false,
                    detail: format!("missing live path {logical} in fresh child"),
                    observed_paths: observed,
                };
            }
            Err(e) => {
                return ChildReport {
                    is_ok: false,
                    detail: format!("pathinfo error for {logical}: {e}"),
                    observed_paths: observed,
                };
            }
        }
    }
    let local_action = LocalActionResultStore::new(&req.state_dir);
    for (logical, action_ref) in expected_action_refs {
        let Ok(lookup) = local_action.lookup(action_ref).await else {
            return ChildReport {
                is_ok: false,
                detail: format!("action result lookup failed for {logical}"),
                observed_paths: observed,
            };
        };
        if lookup.records.is_empty() {
            return ChildReport {
                is_ok: false,
                detail: format!("missing action result record for {logical}"),
                observed_paths: observed,
            };
        }
    }
    let _ = store.pathinfo_service().list().take(1).collect::<Vec<_>>().await;
    ChildReport {
        is_ok: true,
        detail: "ok".to_string(),
        observed_paths: observed,
    }
}

pub async fn execute_history(program: &HistoryProgramV1, mutant: NegativeMutant) -> HistoryRunOutcome {
    let mut env = match HarnessEnv::new(program.profile, program.seed).await {
        Ok(env) => env,
        Err(detail) => {
            return HistoryRunOutcome {
                violation: Some((ViolationKind::AcknowledgedDurabilityLost, detail)),
                requested_vs_applied_faults: Vec::new(),
                executed_schedule: Vec::new(),
                is_mutant_reached: false,
            };
        }
    };

    let mut faults = Vec::new();
    let mut schedule = Vec::new();
    let mut first_violation: Option<(ViolationKind, String)> = None;

    for (idx, op) in program.steps.iter().enumerate() {
        let (classification, observed_identities) = env.step(idx, op, mutant, &mut faults).await;
        if let StepClassification::SafetyViolation { kind, ref detail } = classification {
            first_violation = Some((kind, detail.clone()));
            schedule.push(ExecutedScheduleStep {
                step_index: idx,
                op: op.clone(),
                classification,
                observed_identities,
            });
            break;
        }
        if let StepClassification::HarnessError { ref detail } = classification {
            first_violation = Some((ViolationKind::AcknowledgedDurabilityLost, detail.clone()));
            schedule.push(ExecutedScheduleStep {
                step_index: idx,
                op: op.clone(),
                classification,
                observed_identities,
            });
            break;
        }
        schedule.push(ExecutedScheduleStep {
            step_index: idx,
            op: op.clone(),
            classification,
            observed_identities,
        });
        if let Err((kind, detail)) = env.verify_invariants(mutant).await {
            first_violation = Some((kind, detail));
            break;
        }
    }

    if first_violation.is_none() {
        env.drop_store();
        if let Err(detail) = env.open_store().await {
            first_violation = Some((ViolationKind::AcknowledgedDurabilityLost, detail));
        } else if let Err((kind, detail)) = env.verify_invariants(mutant).await {
            first_violation = Some((kind, detail));
        }
    }

    HistoryRunOutcome {
        violation: first_violation,
        requested_vs_applied_faults: faults,
        executed_schedule: schedule,
        is_mutant_reached: env.is_mutant_reached,
    }
}

pub async fn assert_history_passes_or_retain_artifact(program: &HistoryProgramV1) {
    let outcome = execute_history(program, NegativeMutant::None).await;
    if let Some((kind, detail)) = outcome.violation {
        let minimized = shrink_history_preserving_failure(program, kind, |cand| {
            let rt = tokio::runtime::Handle::current();
            tokio::task::block_in_place(|| rt.block_on(execute_history(cand, NegativeMutant::None)))
                .violation
                .map(|(k, _)| k)
        });
        let artifact = build_artifact(
            program.clone(),
            minimized,
            outcome.requested_vs_applied_faults,
            outcome.executed_schedule,
            kind,
            detail.clone(),
        );
        let saved_path = persist_counterexample_artifact(&artifact);
        panic!("history safety violation ({kind:?}): {detail}; artifact={saved_path:?}");
    }
}

pub async fn replay_artifact_from_json(json: &str) -> Result<(), String> {
    let artifact: CounterexampleArtifact = serde_json::from_str(json).map_err(|e| format!("parse artifact: {e}"))?;
    let orig = execute_history(&artifact.original_program, NegativeMutant::None).await;
    let mini = execute_history(&artifact.minimized_program, NegativeMutant::None).await;
    if orig.violation.is_some() || mini.violation.is_some() {
        return Err(format!(
            "replayed counterexample still reproduces failure: orig={:?} mini={:?}",
            orig.violation, mini.violation
        ));
    }
    Ok(())
}
