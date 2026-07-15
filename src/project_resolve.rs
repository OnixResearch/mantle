use std::io::Read;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use crunch_project::CommandFreshnessProbe;
use crunch_project::DarcsSelector;
use crunch_project::FRESHNESS_PROBE_VERSION;
use crunch_project::FossilSelector;
use crunch_project::FreshnessObservation;
use crunch_project::FreshnessObservationRequest;
use crunch_project::FreshnessObservationStatus;
use crunch_project::FreshnessProbe;
use crunch_project::FreshnessProbeKind;
use crunch_project::GitReference;
use crunch_project::HashAlgo;
use crunch_project::HashResolutionMode;
use crunch_project::LockedHash;
use crunch_project::LockedPatch;
use crunch_project::ManifestInput;
use crunch_project::PatchDef;
use crunch_project::PijulSelector;
use crunch_project::RefreshResolver;
use crunch_project::ResolvedDarcsIdentity;
use crunch_project::ResolvedFossilIdentity;
use crunch_project::ResolvedPijulIdentity;
use crunch_project::TrustSignatureRef;
use crunch_project::TrustSubject;
use crunch_project::VerifiedTrustFact;
use crunch_project::normalize_freshness_observation;
use crunch_project::trust_signature_payload;
use digest::Digest;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobServiceConfig;
use snix_castore::composition::CompositionContext;
use snix_castore::composition::REG;
use snix_castore::composition::ServiceBuilder;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tempfile::tempdir;

const MIB_BYTES: u64 = 1_048_576;
const MAX_DOWNLOAD_BYTES: u64 = 4_294_967_296;
const MAX_TREE_BYTES: u64 = 4_294_967_296;
const MAX_TREE_ENTRIES: u32 = 500_000;
const FETCH_CONNECT_TIMEOUT_SECS: u64 = 60;
const FETCH_READ_TIMEOUT_SECS: u64 = 600;
const FRESHNESS_LOCAL_TREE_MAX_BYTES: u64 = 4_294_967_296;
const FRESHNESS_LOCAL_TREE_MAX_ENTRIES: u32 = 500_000;
const FRESHNESS_HTTP_MAX_BYTES: u64 = MIB_BYTES;
const FRESHNESS_STDERR_LIMIT_BYTES: u32 = 16_384;
const FRESHNESS_STDERR_LIMIT_BYTES_USIZE: usize = 16_384;
const COMMAND_POLL_INTERVAL_MS: u64 = 10;
const GIT_REV_HEX_BYTES: usize = 40;
const BLAKE3_HEX_BYTES: usize = 64;
const PROJECT_READ_BUFFER_BYTES: usize = 65_536;
const PROJECT_READ_CHUNK_COUNT_MAX: u32 = 65_537;
const FRESHNESS_DIGEST_PREFIX: &str = "blake3:";
const HTTP_JSON_POINTER_ROOT: &str = "";
const MAX_PROJECT_TRUST_SIGNATURE_BYTES: u64 = 16_384;

pub struct LiveResolver {
    project_root: PathBuf,
}

impl LiveResolver {
    pub fn new(project_root: &Path) -> Self {
        assert!(project_root.components().next().is_some(), "project root must not be empty");
        Self {
            project_root: project_root.to_path_buf(),
        }
    }
}

impl RefreshResolver for LiveResolver {
    fn resolve_git_rev(
        &self,
        repository: &str,
        reference: &GitReference,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(resolve_git_reference(repository, reference)?))
    }

    fn hash_url_content(
        &self,
        url: &str,
        algo: &HashAlgo,
        mode: HashResolutionMode,
    ) -> Result<Option<String>, crunch_project::Error> {
        let sri = match mode {
            HashResolutionMode::Flat => hash_flat_url(url, algo)?,
            HashResolutionMode::Recursive => hash_tarball_url(url, algo)?,
        };
        Ok(Some(sri))
    }

    fn hash_git_checkout(
        &self,
        repository: &str,
        rev: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(hash_git_tree(repository, rev, algo)?))
    }

    fn resolve_darcs_identity(
        &self,
        repository: &str,
        selector: &DarcsSelector,
    ) -> Result<Option<ResolvedDarcsIdentity>, crunch_project::Error> {
        Ok(Some(resolve_darcs_identity(repository, selector)?))
    }

    fn hash_darcs_checkout(
        &self,
        repository: &str,
        identity: &ResolvedDarcsIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(hash_darcs_tree(repository, identity, algo)?))
    }

    fn resolve_pijul_identity(
        &self,
        repository: &str,
        selector: &PijulSelector,
    ) -> Result<Option<ResolvedPijulIdentity>, crunch_project::Error> {
        Ok(Some(resolve_pijul_identity(repository, selector)?))
    }

    fn hash_pijul_checkout(
        &self,
        repository: &str,
        identity: &ResolvedPijulIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(hash_pijul_tree(repository, identity, algo)?))
    }

    fn resolve_fossil_identity(
        &self,
        repository: &str,
        selector: &FossilSelector,
    ) -> Result<Option<ResolvedFossilIdentity>, crunch_project::Error> {
        Ok(Some(resolve_fossil_identity(repository, selector)?))
    }

    fn hash_fossil_checkout(
        &self,
        repository: &str,
        identity: &ResolvedFossilIdentity,
        algo: &HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(hash_fossil_tree(repository, identity, algo)?))
    }

    fn hash_local_file(&self, path: &str, algo: &HashAlgo) -> Result<Option<String>, crunch_project::Error> {
        let resolved = resolve_local_path(&self.project_root, path);
        Ok(Some(hash_flat_file(&resolved, algo)?))
    }

    fn observe_freshness(
        &self,
        input: &ManifestInput,
        no_network: bool,
    ) -> Result<Option<FreshnessObservation>, crunch_project::Error> {
        let Some(probe) = input.freshness.as_ref() else {
            return Ok(None);
        };
        Ok(Some(observe_freshness_probe(&self.project_root, &input.name, probe, no_network)))
    }

    fn verify_input_trust(
        &self,
        input: &ManifestInput,
        entry: &crunch_project::LockEntry,
    ) -> Result<Vec<VerifiedTrustFact>, crunch_project::Error> {
        let subject = TrustSubject::input(input.name.clone());
        verify_project_trust_policy(&self.project_root, &subject, input.trust.as_ref(), &entry.hash)
    }

    fn verify_patch_trust(
        &self,
        patch: &PatchDef,
        locked: &LockedPatch,
    ) -> Result<Vec<VerifiedTrustFact>, crunch_project::Error> {
        let subject = TrustSubject::patch(patch.name.clone());
        verify_project_trust_policy(&self.project_root, &subject, patch.trust.as_ref(), &locked.hash)
    }
}

fn verify_project_trust_policy(
    project_root: &Path,
    subject: &TrustSubject,
    policy: Option<&crunch_project::InputTrustPolicy>,
    hash: &LockedHash,
) -> Result<Vec<VerifiedTrustFact>, crunch_project::Error> {
    assert!(project_root.components().next().is_some(), "project root must not be empty");
    assert!(!subject.name.is_empty(), "trust subject name must not be empty");
    let Some(policy) = policy else {
        return Ok(Vec::new());
    };
    let payload = trust_signature_payload(subject, policy.digest_binding, &hash.algo, &hash.value);
    let trusted_keys = parse_trusted_project_keys(&policy.trusted_public_keys)?;
    let fact_slot_count =
        policy.signatures.len().checked_mul(trusted_keys.len()).ok_or_else(|| {
            crunch_project::Error::Manifest("project trust fact capacity overflowed usize".to_string())
        })?;
    let mut facts = Vec::with_capacity(fact_slot_count);
    for signature_ref in &policy.signatures {
        let signature_text = read_project_trust_signature(project_root, signature_ref)?;
        let signature = nix_compat::narinfo::SignatureRef::parse(signature_text.trim()).map_err(|err| {
            crunch_project::Error::Manifest(format!("{} trust signature is invalid: {err}", subject.label()))
        })?;
        for key in &trusted_keys {
            if key.name() != *signature.name() {
                continue;
            }
            if !key.verify(&payload, &signature) {
                continue;
            }
            facts.push(VerifiedTrustFact {
                subject: subject.clone(),
                verifier: policy.verifier,
                digest_binding: policy.digest_binding,
                hash_algo: hash.algo.clone(),
                hash_value: hash.value.clone(),
                signer: key.name().to_string(),
                key_ref: key.to_string(),
                signature_ref: signature_ref.ref_text().to_string(),
            });
        }
    }
    Ok(facts)
}

fn parse_trusted_project_keys(
    keys: &[String],
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, crunch_project::Error> {
    keys.iter()
        .map(|key| {
            nix_compat::narinfo::VerifyingKey::parse(key).map_err(|err| {
                crunch_project::Error::Manifest(format!("invalid project input trusted public key '{key}': {err}"))
            })
        })
        .collect()
}

fn read_project_trust_signature(
    project_root: &Path,
    signature: &TrustSignatureRef,
) -> Result<String, crunch_project::Error> {
    match signature {
        TrustSignatureRef::LocalFile { path } => {
            let resolved = resolve_local_path(project_root, path);
            let metadata = std::fs::metadata(&resolved).map_err(|err| {
                crunch_project::Error::Manifest(format!(
                    "reading project trust signature '{}': {err}",
                    resolved.display()
                ))
            })?;
            if metadata.len() > MAX_PROJECT_TRUST_SIGNATURE_BYTES {
                return Err(crunch_project::Error::Manifest(format!(
                    "project trust signature '{}' exceeds {MAX_PROJECT_TRUST_SIGNATURE_BYTES} bytes",
                    resolved.display()
                )));
            }
            std::fs::read_to_string(&resolved).map_err(|err| {
                crunch_project::Error::Manifest(format!(
                    "reading project trust signature '{}': {err}",
                    resolved.display()
                ))
            })
        }
    }
}

fn observe_freshness_probe(
    project_root: &Path,
    input_name: &str,
    probe: &FreshnessProbe,
    no_network: bool,
) -> FreshnessObservation {
    assert!(!input_name.is_empty(), "freshness input name must not be empty");
    assert!(project_root.components().next().is_some(), "project root must not be empty");
    let kind = freshness_probe_kind(probe);
    let is_network_required = freshness_probe_requires_network(probe);
    let identity_digest = match freshness_probe_identity_digest(probe) {
        Ok(digest) => Some(digest),
        Err(error) => {
            let diagnostic = match error {
                FreshnessProbeShellError::Failed(message) | FreshnessProbeShellError::Unavailable(message) => message,
            };
            return freshness_status(FreshnessStatusRequest {
                input_name,
                probe_kind: kind,
                is_network_required,
                status: FreshnessObservationStatus::Failed,
                value: None,
                diagnostic,
                probe_identity_digest: None,
            });
        }
    };
    if no_network && is_network_required {
        return freshness_status(FreshnessStatusRequest {
            input_name,
            probe_kind: kind,
            is_network_required,
            status: FreshnessObservationStatus::NetworkRequired,
            value: None,
            diagnostic: "network disabled for freshness probe".to_string(),
            probe_identity_digest: identity_digest,
        });
    }
    let (status, value, diagnostic) = match run_freshness_probe(project_root, probe) {
        Ok(value) if value.is_empty() => {
            (FreshnessObservationStatus::Failed, None, "freshness probe produced empty value".to_string())
        }
        Ok(value) => (FreshnessObservationStatus::Observed, Some(value), String::new()),
        Err(FreshnessProbeShellError::Unavailable(message)) => {
            (FreshnessObservationStatus::ProbeUnavailable, None, message)
        }
        Err(FreshnessProbeShellError::Failed(message)) => (FreshnessObservationStatus::Failed, None, message),
    };
    freshness_status(FreshnessStatusRequest {
        input_name,
        probe_kind: kind,
        is_network_required,
        status,
        value,
        diagnostic,
        probe_identity_digest: identity_digest,
    })
}

struct FreshnessStatusRequest<'a> {
    input_name: &'a str,
    probe_kind: FreshnessProbeKind,
    is_network_required: bool,
    status: FreshnessObservationStatus,
    value: Option<String>,
    diagnostic: String,
    probe_identity_digest: Option<String>,
}

fn freshness_status(request: FreshnessStatusRequest<'_>) -> FreshnessObservation {
    debug_assert!(!request.input_name.is_empty());
    debug_assert_eq!(request.status == FreshnessObservationStatus::Observed, request.value.is_some());
    let core_request = FreshnessObservationRequest {
        version: FRESHNESS_PROBE_VERSION,
        input_name: request.input_name.to_string(),
        probe_kind: request.probe_kind,
        requires_network: request.is_network_required,
        status: request.status,
        value: request.value.clone(),
        diagnostic: request.diagnostic.clone(),
        probe_identity_digest: request.probe_identity_digest.clone(),
    };
    match normalize_freshness_observation(core_request) {
        Ok(observation) => observation,
        Err(error) => failed_freshness_observation(request, error),
    }
}

fn failed_freshness_observation(
    request: FreshnessStatusRequest<'_>,
    error: crunch_project::FreshnessError,
) -> FreshnessObservation {
    debug_assert!(!request.input_name.is_empty());
    debug_assert!(!error.to_string().is_empty());
    let diagnostic = format!("freshness observation rejected: {error}");
    let fallback_request = FreshnessObservationRequest {
        version: FRESHNESS_PROBE_VERSION,
        input_name: request.input_name.to_string(),
        probe_kind: request.probe_kind,
        requires_network: request.is_network_required,
        status: FreshnessObservationStatus::Failed,
        value: None,
        diagnostic: diagnostic.clone(),
        probe_identity_digest: request.probe_identity_digest,
    };
    normalize_freshness_observation(fallback_request).unwrap_or(FreshnessObservation {
        version: FRESHNESS_PROBE_VERSION,
        input_name: request.input_name.to_string(),
        probe_kind: request.probe_kind,
        requires_network: request.is_network_required,
        status: FreshnessObservationStatus::Failed,
        value: None,
        value_digest: None,
        diagnostic,
        probe_identity_digest: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FreshnessProbeShellError {
    Failed(String),
    Unavailable(String),
}

fn run_freshness_probe(project_root: &Path, probe: &FreshnessProbe) -> Result<String, FreshnessProbeShellError> {
    match probe {
        FreshnessProbe::GitRef { repository, reference } => observe_git_ref(repository, reference),
        FreshnessProbe::HttpText { url } => observe_http_text(url),
        FreshnessProbe::HttpJson { url, pointer } => observe_http_json(url, pointer),
        FreshnessProbe::LocalFile { path } => observe_local_file(project_root, path),
        FreshnessProbe::LocalDirectory { path } => observe_local_directory(project_root, path),
        FreshnessProbe::Command { command, .. } => observe_command(project_root, command),
    }
}

fn freshness_probe_kind(probe: &FreshnessProbe) -> FreshnessProbeKind {
    match probe {
        FreshnessProbe::GitRef { .. } => FreshnessProbeKind::GitRef,
        FreshnessProbe::HttpText { .. } => FreshnessProbeKind::HttpText,
        FreshnessProbe::HttpJson { .. } => FreshnessProbeKind::HttpJson,
        FreshnessProbe::LocalFile { .. } => FreshnessProbeKind::LocalFile,
        FreshnessProbe::LocalDirectory { .. } => FreshnessProbeKind::LocalDirectory,
        FreshnessProbe::Command { .. } => FreshnessProbeKind::Command,
    }
}

fn freshness_probe_requires_network(probe: &FreshnessProbe) -> bool {
    match probe {
        FreshnessProbe::Command { requires_network, .. } => *requires_network,
        _ => freshness_probe_kind(probe).requires_network(),
    }
}

fn freshness_probe_identity_digest(probe: &FreshnessProbe) -> Result<String, FreshnessProbeShellError> {
    let bytes = serde_json::to_vec(probe)
        .map_err(|error| FreshnessProbeShellError::Failed(format!("serializing freshness probe identity: {error}")))?;
    Ok(blake3_hex_digest(&bytes))
}

fn observe_git_ref<R: AsRef<str>, F: AsRef<str>>(
    repository: R,
    reference: F,
) -> Result<String, FreshnessProbeShellError> {
    let repository = repository.as_ref();
    let reference = reference.as_ref();
    let git = find_git_binary().map_err(|err| FreshnessProbeShellError::Unavailable(err.to_string()))?;
    let output = Command::new(&git)
        .arg("ls-remote")
        .arg(repository)
        .arg(reference)
        .output()
        .map_err(|err| FreshnessProbeShellError::Unavailable(format!("running git ls-remote: {err}")))?;
    if !output.status.success() {
        return Err(FreshnessProbeShellError::Failed(format!(
            "git ls-remote failed: {}",
            bounded_stderr_utf8_lossy(&output.stderr)
        )));
    }
    parse_git_ref_probe_output(String::from_utf8_lossy(&output.stdout), reference)
}

fn parse_git_ref_probe_output<O: AsRef<str>, F: AsRef<str>>(
    output: O,
    reference: F,
) -> Result<String, FreshnessProbeShellError> {
    let output = output.as_ref();
    let reference = reference.as_ref();
    for line in output.lines() {
        let Some((rev, found_ref)) = line.split_once('\t') else {
            continue;
        };
        if found_ref == reference {
            if rev.len() == GIT_REV_HEX_BYTES && rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Ok(rev.to_string());
            }
            return Err(FreshnessProbeShellError::Failed("git freshness probe returned invalid rev".to_string()));
        }
    }
    Err(FreshnessProbeShellError::Failed(format!("git reference not found: {reference}")))
}

fn observe_http_text(url: &str) -> Result<String, FreshnessProbeShellError> {
    let bytes = read_url_limited(url, FRESHNESS_HTTP_MAX_BYTES)?;
    String::from_utf8(bytes)
        .map(|value| trim_freshness_text(value).to_string())
        .map_err(|err| FreshnessProbeShellError::Failed(format!("HTTP text freshness value is not UTF-8: {err}")))
}

fn observe_http_json<U: AsRef<str>, P: AsRef<str>>(url: U, pointer: P) -> Result<String, FreshnessProbeShellError> {
    let url = url.as_ref();
    let pointer = pointer.as_ref();
    let bytes = read_url_limited(url, FRESHNESS_HTTP_MAX_BYTES)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|err| FreshnessProbeShellError::Failed(format!("HTTP JSON freshness body did not parse: {err}")))?;
    let selected = select_json_pointer(&json, pointer)?;
    json_value_to_freshness_string(selected)
}

fn select_json_pointer<'a>(
    json: &'a serde_json::Value,
    pointer: &str,
) -> Result<&'a serde_json::Value, FreshnessProbeShellError> {
    if pointer == HTTP_JSON_POINTER_ROOT {
        return Ok(json);
    }
    json.pointer(pointer)
        .ok_or_else(|| FreshnessProbeShellError::Failed(format!("HTTP JSON pointer not found: {pointer}")))
}

fn json_value_to_freshness_string(value: &serde_json::Value) -> Result<String, FreshnessProbeShellError> {
    match value {
        serde_json::Value::String(value) => Ok(value.clone()),
        serde_json::Value::Number(_) | serde_json::Value::Bool(_) => Ok(value.to_string()),
        _ => serde_json::to_string(value)
            .map_err(|err| FreshnessProbeShellError::Failed(format!("rendering JSON freshness value: {err}"))),
    }
}

fn observe_local_file(project_root: &Path, path: &str) -> Result<String, FreshnessProbeShellError> {
    let path = resolve_local_path(project_root, path);
    let file = std::fs::File::open(&path).map_err(|err| {
        FreshnessProbeShellError::Failed(format!("opening local freshness file {}: {err}", path.display()))
    })?;
    let bounded = BoundedReader::new(file, FRESHNESS_HTTP_MAX_BYTES, path.display().to_string());
    hash_reader_blake3(bounded, &format!("local freshness file {}", path.display()))
        .map_err(|err| FreshnessProbeShellError::Failed(err.to_string()))
}

fn observe_local_directory(project_root: &Path, path: &str) -> Result<String, FreshnessProbeShellError> {
    let path = resolve_local_path(project_root, path);
    let digest = hash_local_directory_freshness(&path)?;
    Ok(digest)
}

fn read_url_limited(url: &str, limit_bytes: u64) -> Result<Vec<u8>, FreshnessProbeShellError> {
    let mut reader = open_url_reader(url).map_err(|err| FreshnessProbeShellError::Failed(err.to_string()))?;
    let mut bounded = BoundedReader::new(&mut reader, limit_bytes, url.to_string());
    let mut bytes = Vec::new();
    bounded
        .read_to_end(&mut bytes)
        .map_err(|err| FreshnessProbeShellError::Failed(format!("reading freshness URL {url}: {err}")))?;
    Ok(bytes)
}

fn trim_freshness_text(value: String) -> String {
    value.trim_end_matches(['\r', '\n']).to_string()
}

fn observe_command(project_root: &Path, command: &CommandFreshnessProbe) -> Result<String, FreshnessProbeShellError> {
    assert!(!command.argv.is_empty(), "validated command probe must have argv");
    assert!(project_root.components().next().is_some(), "project root must not be empty");
    let cwd = resolve_local_path(project_root, &command.cwd);
    let mut child = spawn_freshness_command(&cwd, command)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| FreshnessProbeShellError::Failed("command stdout pipe unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| FreshnessProbeShellError::Failed("command stderr pipe unavailable".to_string()))?;
    let stdout_limit_bytes = command.output_limit_bytes;
    let stdout_handle = thread::spawn(move || read_pipe_limited(stdout, stdout_limit_bytes, "stdout"));
    let stderr_handle = thread::spawn(move || read_pipe_limited(stderr, FRESHNESS_STDERR_LIMIT_BYTES, "stderr"));
    let status = wait_for_freshness_command(&mut child, command.timeout_ms)?;
    let stdout_bytes = join_reader_thread(stdout_handle, "stdout")?;
    let stderr_bytes = join_reader_thread(stderr_handle, "stderr")?;
    let Some(code) = status.code() else {
        return Err(FreshnessProbeShellError::Failed("command terminated by signal".to_string()));
    };
    if !command.success_statuses.contains(&code) {
        return Err(FreshnessProbeShellError::Failed(format!(
            "command exited {code}: {}",
            bounded_stderr_utf8_lossy(&stderr_bytes)
        )));
    }
    command_output_value(command, stdout_bytes)
}

fn spawn_freshness_command(
    cwd: &Path,
    command: &CommandFreshnessProbe,
) -> Result<std::process::Child, FreshnessProbeShellError> {
    let mut process = Command::new(&command.argv[0]);
    process.args(&command.argv[1..]);
    process.current_dir(cwd);
    process.env_clear();
    for entry in &command.env {
        process.env(&entry.name, &entry.value);
    }
    process.stdout(Stdio::piped());
    process.stderr(Stdio::piped());
    process.spawn().map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            FreshnessProbeShellError::Unavailable(format!("command executable not found: {}", command.argv[0]))
        } else {
            FreshnessProbeShellError::Failed(format!("spawning command probe {}: {err}", command.argv[0]))
        }
    })
}

fn wait_for_freshness_command(
    child: &mut std::process::Child,
    timeout_ms: u32,
) -> Result<std::process::ExitStatus, FreshnessProbeShellError> {
    let poll_interval_ms = u32::try_from(COMMAND_POLL_INTERVAL_MS)
        .map_err(|_| FreshnessProbeShellError::Failed("command poll interval exceeds u32".to_string()))?;
    let poll_count_max = timeout_ms.div_ceil(poll_interval_ms).max(1);
    for _ in 0..poll_count_max {
        if let Some(status) = child
            .try_wait()
            .map_err(|err| FreshnessProbeShellError::Failed(format!("waiting for command probe: {err}")))?
        {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(COMMAND_POLL_INTERVAL_MS));
    }
    terminate_timed_out_child(child, timeout_ms)
}

fn terminate_timed_out_child(
    child: &mut std::process::Child,
    timeout_ms: u32,
) -> Result<std::process::ExitStatus, FreshnessProbeShellError> {
    child
        .kill()
        .map_err(|error| FreshnessProbeShellError::Failed(format!("killing timed-out command probe: {error}")))?;
    child
        .wait()
        .map_err(|error| FreshnessProbeShellError::Failed(format!("reaping timed-out command probe: {error}")))?;
    Err(FreshnessProbeShellError::Failed(format!("command timed out after {timeout_ms} ms")))
}

fn command_output_value(
    command: &CommandFreshnessProbe,
    stdout_bytes: Vec<u8>,
) -> Result<String, FreshnessProbeShellError> {
    if command.utf8_required {
        let value = String::from_utf8(stdout_bytes)
            .map_err(|err| FreshnessProbeShellError::Failed(format!("command output is not UTF-8: {err}")))?;
        let trimmed = trim_freshness_text(value);
        if trimmed.is_empty() {
            return Err(FreshnessProbeShellError::Failed("command produced empty output".to_string()));
        }
        return Ok(trimmed);
    }
    if stdout_bytes.is_empty() {
        return Err(FreshnessProbeShellError::Failed("command produced empty output".to_string()));
    }
    Ok(blake3_hex_digest(&stdout_bytes))
}

fn read_pipe_limited<R: Read>(
    mut reader: R,
    limit_bytes: u32,
    label: &'static str,
) -> Result<Vec<u8>, FreshnessProbeShellError> {
    let mut bytes = Vec::new();
    let mut bounded = BoundedReader::new(&mut reader, limit_bytes as u64, label.to_string());
    bounded
        .read_to_end(&mut bytes)
        .map_err(|err| FreshnessProbeShellError::Failed(format!("command {label} exceeds limit: {err}")))?;
    Ok(bytes)
}

fn join_reader_thread(
    handle: thread::JoinHandle<Result<Vec<u8>, FreshnessProbeShellError>>,
    label: &'static str,
) -> Result<Vec<u8>, FreshnessProbeShellError> {
    handle
        .join()
        .map_err(|_| FreshnessProbeShellError::Failed(format!("command {label} reader thread panicked")))?
}

fn hash_local_directory_freshness(path: &Path) -> Result<String, FreshnessProbeShellError> {
    if !path.is_dir() {
        return Err(FreshnessProbeShellError::Failed(format!(
            "local freshness directory is not a directory: {}",
            path.display()
        )));
    }
    debug_assert!(path.exists());
    debug_assert!(path.is_dir());
    let mut files = Vec::new();
    collect_freshness_files(path, &mut files)?;
    files.sort();
    let mut hasher = blake3::Hasher::new();
    let mut bytes_total: u64 = 0;
    for relative in files {
        let file_bytes = update_directory_freshness_hash(path, &relative, &mut hasher)?;
        bytes_total = bytes_total.saturating_add(file_bytes);
        if bytes_total > FRESHNESS_LOCAL_TREE_MAX_BYTES {
            return Err(FreshnessProbeShellError::Failed(format!(
                "local freshness directory exceeds {FRESHNESS_LOCAL_TREE_MAX_BYTES} bytes: {}",
                path.display()
            )));
        }
    }
    Ok(format!("{FRESHNESS_DIGEST_PREFIX}{}", blake3::Hasher::finalize(&hasher).to_hex()))
}

fn collect_freshness_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), FreshnessProbeShellError> {
    debug_assert!(root.is_dir());
    debug_assert!(files.is_empty());
    let mut pending = vec![root.to_path_buf()];
    for _ in 0..FRESHNESS_LOCAL_TREE_MAX_ENTRIES {
        let Some(current) = pending.pop() else {
            return Ok(());
        };
        let children = std::fs::read_dir(&current)
            .map_err(|err| FreshnessProbeShellError::Failed(format!("read_dir {}: {err}", current.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| FreshnessProbeShellError::Failed(format!("walking {}: {err}", current.display())))?;
        let projected_entry_count = pending
            .len()
            .checked_add(files.len())
            .and_then(|count| count.checked_add(children.len()))
            .and_then(|count| u32::try_from(count).ok())
            .ok_or_else(|| {
                FreshnessProbeShellError::Failed("local freshness entry count overflowed u32".to_string())
            })?;
        if projected_entry_count > FRESHNESS_LOCAL_TREE_MAX_ENTRIES {
            return Err(FreshnessProbeShellError::Failed(format!(
                "local freshness directory exceeds {FRESHNESS_LOCAL_TREE_MAX_ENTRIES} entries: {}",
                root.display()
            )));
        }
        for child in children {
            let path = child.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|err| FreshnessProbeShellError::Failed(format!("stat {}: {err}", path.display())))?;
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|err| {
                    FreshnessProbeShellError::Failed(format!("relativizing {}: {err}", path.display()))
                })?;
                files.push(relative.to_path_buf());
            }
        }
    }
    Err(FreshnessProbeShellError::Failed(format!(
        "local freshness directory exceeds {FRESHNESS_LOCAL_TREE_MAX_ENTRIES} entries: {}",
        root.display()
    )))
}

fn update_directory_freshness_hash(
    root: &Path,
    relative: &Path,
    hasher: &mut blake3::Hasher,
) -> Result<u64, FreshnessProbeShellError> {
    let path = root.join(relative);
    let metadata = std::fs::metadata(&path)
        .map_err(|err| FreshnessProbeShellError::Failed(format!("stat {}: {err}", path.display())))?;
    if metadata.len() > FRESHNESS_LOCAL_TREE_MAX_BYTES {
        return Err(FreshnessProbeShellError::Failed(format!(
            "local freshness file exceeds {FRESHNESS_LOCAL_TREE_MAX_BYTES} bytes: {}",
            path.display()
        )));
    }
    hasher.update(relative.to_string_lossy().as_bytes());
    hasher.update(&[0]);
    let file = std::fs::File::open(&path)
        .map_err(|err| FreshnessProbeShellError::Failed(format!("opening {}: {err}", path.display())))?;
    hash_reader_into_blake3(file, hasher, &format!("local freshness path {}", path.display()))?;
    Ok(metadata.len())
}

fn hash_reader_blake3<R: Read>(reader: R, label: &str) -> Result<String, crunch_project::Error> {
    let mut hasher = blake3::Hasher::new();
    hash_reader_into_blake3(reader, &mut hasher, label).map_err(|err| match err {
        FreshnessProbeShellError::Failed(message) | FreshnessProbeShellError::Unavailable(message) => {
            crunch_project::Error::Manifest(message)
        }
    })?;
    Ok(format!("{FRESHNESS_DIGEST_PREFIX}{}", blake3::Hasher::finalize(&hasher).to_hex()))
}

fn hash_reader_into_blake3<R: Read>(
    mut reader: R,
    hasher: &mut blake3::Hasher,
    label: &str,
) -> Result<(), FreshnessProbeShellError> {
    let mut buffer = [0u8; PROJECT_READ_BUFFER_BYTES];
    for _ in 0..PROJECT_READ_CHUNK_COUNT_MAX {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| FreshnessProbeShellError::Failed(format!("reading {label}: {err}")))?;
        if read == 0 {
            return Ok(());
        }
        hasher.update(&buffer[..read]);
    }
    Err(FreshnessProbeShellError::Failed(format!(
        "reading {label} exceeded {PROJECT_READ_CHUNK_COUNT_MAX} chunks"
    )))
}

fn blake3_hex_digest(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes).to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_BYTES, "BLAKE3 hex digest length changed");
    format!("{FRESHNESS_DIGEST_PREFIX}{digest}")
}

fn bounded_stderr_utf8_lossy(bytes: &[u8]) -> String {
    let max_len = bytes.len().min(FRESHNESS_STDERR_LIMIT_BYTES_USIZE);
    String::from_utf8_lossy(&bytes[..max_len]).into_owned()
}

fn resolve_git_reference(repository: &str, reference: &GitReference) -> Result<String, crunch_project::Error> {
    match reference {
        GitReference::Rev(rev) => {
            validate_git_rev(rev)?;
            Ok(rev.clone())
        }
        GitReference::Branch(branch) => {
            let output = run_git_ls_remote(repository, &[&format!("refs/heads/{branch}")])?;
            parse_branch_output(&output, branch)
        }
        GitReference::Tag(tag) => {
            let direct = format!("refs/tags/{tag}");
            let peeled = format!("refs/tags/{tag}^{{}}");
            let output = run_git_ls_remote(repository, &[&direct, &peeled])?;
            parse_tag_output(&output, tag)
        }
    }
}

fn validate_git_rev(rev: &str) -> Result<(), crunch_project::Error> {
    if rev.len() != 40 {
        return Err(crunch_project::Error::Validation(format!("git rev must be 40 hex characters, got {rev}")));
    }
    if !rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(crunch_project::Error::Validation(format!("git rev must be hex, got {rev}")));
    }
    Ok(())
}

fn run_git_ls_remote(repository: &str, refs: &[&str]) -> Result<String, crunch_project::Error> {
    let git = find_git_binary()?;
    let output = Command::new(&git)
        .arg("ls-remote")
        .arg(repository)
        .args(refs)
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git ls-remote for {repository}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git ls-remote failed for {repository}: {stderr}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_branch_output<O: AsRef<str>, B: AsRef<str>>(output: O, branch: B) -> Result<String, crunch_project::Error> {
    let output = output.as_ref();
    let branch = branch.as_ref();
    let wanted = format!("refs/heads/{branch}");
    for line in output.lines() {
        let Some((rev, reference)) = line.split_once('\t') else {
            continue;
        };
        if reference == wanted {
            validate_git_rev(rev)?;
            return Ok(rev.to_string());
        }
    }
    Err(crunch_project::Error::Manifest(format!("git branch not found: {branch}")))
}

fn parse_tag_output<O: AsRef<str>, T: AsRef<str>>(output: O, tag: T) -> Result<String, crunch_project::Error> {
    let output = output.as_ref();
    let tag = tag.as_ref();
    let direct = format!("refs/tags/{tag}");
    let peeled = format!("refs/tags/{tag}^{{}}");
    debug_assert!(direct.starts_with("refs/tags/"));
    debug_assert!(peeled.ends_with("^{}"));
    let mut direct_rev: Option<String> = None;
    for line in output.lines() {
        let Some((rev, reference)) = line.split_once('\t') else {
            continue;
        };
        if reference == peeled {
            validate_git_rev(rev)?;
            return Ok(rev.to_string());
        }
        if reference == direct {
            validate_git_rev(rev)?;
            direct_rev = Some(rev.to_string());
        }
    }
    direct_rev.ok_or_else(|| crunch_project::Error::Manifest(format!("git tag not found: {tag}")))
}

fn hash_flat_url(url: &str, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let reader = open_url_reader(url)?;
    let bounded = BoundedReader::new(reader, MAX_DOWNLOAD_BYTES, url.to_string());
    hash_reader_to_sri(bounded, algo, &format!("downloaded content for {url}"))
}

fn hash_tarball_url(url: &str, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let reader = open_url_reader(url)?;
    let bounded = BoundedReader::new(reader, MAX_DOWNLOAD_BYTES, url.to_string());
    let unpacked = crunch_build::fetcher::decompress_reader(url, bounded)
        .map_err(|err| crunch_project::Error::Manifest(format!("decompressing {url}: {err}")))?;
    let dir = tempdir()?;
    let out = path_to_str(dir.path(), "temporary unpack directory")?;
    crunch_build::fetcher::extract_tar(unpacked, out)
        .map_err(|err| crunch_project::Error::Manifest(format!("extracting tarball {url}: {err}")))?;
    ensure_tree_within_limits(dir.path())?;
    hash_recursive_path(dir.path(), algo)
}

fn hash_git_tree<R: AsRef<str>, V: AsRef<str>>(
    repository: R,
    rev: V,
    algo: &HashAlgo,
) -> Result<String, crunch_project::Error> {
    let repository = repository.as_ref();
    let rev = rev.as_ref();
    let git = find_git_binary()?;
    let dir = tempdir()?;
    let bare = dir.path().join("source.git");
    let checkout = dir.path().join("checkout");
    run_git_clone_bare(&git, repository, &bare)?;
    run_git_checkout_tree(&git, &bare, &checkout, rev)?;
    ensure_tree_within_limits(&checkout)?;
    hash_recursive_path(&checkout, algo)
}

fn resolve_darcs_identity(
    repository: &str,
    selector: &DarcsSelector,
) -> Result<ResolvedDarcsIdentity, crunch_project::Error> {
    match selector {
        DarcsSelector::Context(context) => Ok(ResolvedDarcsIdentity {
            context: Some(context.clone()),
            weak_hash: None,
        }),
        DarcsSelector::Tag(tag) => {
            let darcs =
                find_vcs_binary("darcs", &["/usr/bin/darcs", "/bin/darcs", "/run/current-system/sw/bin/darcs"])?;
            let dir = tempdir()?;
            let checkout = dir.path().join("checkout");
            run_darcs_clone_tag(&darcs, repository, tag, &checkout)?;
            let context = darcs_context_for_checkout(&darcs, &checkout)?;
            Ok(ResolvedDarcsIdentity {
                context: Some(context),
                weak_hash: None,
            })
        }
    }
}

fn hash_darcs_tree(
    repository: &str,
    identity: &ResolvedDarcsIdentity,
    algo: &HashAlgo,
) -> Result<String, crunch_project::Error> {
    let darcs = find_vcs_binary("darcs", &["/usr/bin/darcs", "/bin/darcs", "/run/current-system/sw/bin/darcs"])?;
    let context = identity.context.as_deref().ok_or_else(|| {
        crunch_project::Error::Manifest(
            "unsupported-VCS-tool: darcs checkout hashing requires context identity metadata".to_string(),
        )
    })?;
    let dir = tempdir()?;
    let context_file = dir.path().join("context");
    let checkout = dir.path().join("checkout");
    std::fs::write(&context_file, context).map_err(|err| {
        crunch_project::Error::Manifest(format!("writing temporary darcs context {}: {err}", context_file.display()))
    })?;
    run_darcs_clone_context(&darcs, repository, &context_file, &checkout)?;
    ensure_tree_within_limits(&checkout)?;
    hash_recursive_path(&checkout, algo)
}

fn resolve_pijul_identity(
    repository: &str,
    selector: &PijulSelector,
) -> Result<ResolvedPijulIdentity, crunch_project::Error> {
    match selector {
        PijulSelector::State { channel, state } => Ok(ResolvedPijulIdentity {
            channel: channel.clone(),
            state: state.clone(),
            change: None,
        }),
        PijulSelector::Change { .. } => Err(crunch_project::Error::Manifest(
            "unsupported-VCS-tool: pijul change selector cannot prove channel state with this adapter".to_string(),
        )),
        PijulSelector::Channel { channel } => {
            let pijul =
                find_vcs_binary("pijul", &["/usr/bin/pijul", "/bin/pijul", "/run/current-system/sw/bin/pijul"])?;
            let dir = tempdir()?;
            let checkout = dir.path().join("checkout");
            run_pijul_clone(&pijul, repository, channel, &checkout)?;
            let state = pijul_channel_state(&pijul, &checkout, channel)?;
            Ok(ResolvedPijulIdentity {
                channel: channel.clone(),
                state,
                change: None,
            })
        }
    }
}

fn hash_pijul_tree(
    repository: &str,
    identity: &ResolvedPijulIdentity,
    algo: &HashAlgo,
) -> Result<String, crunch_project::Error> {
    let pijul = find_vcs_binary("pijul", &["/usr/bin/pijul", "/bin/pijul", "/run/current-system/sw/bin/pijul"])?;
    let dir = tempdir()?;
    let checkout = dir.path().join("checkout");
    run_pijul_clone(&pijul, repository, &identity.channel, &checkout)?;
    run_pijul_reset(&pijul, &checkout, &identity.state)?;
    ensure_tree_within_limits(&checkout)?;
    hash_recursive_path(&checkout, algo)
}

fn resolve_fossil_identity(
    repository: &str,
    selector: &FossilSelector,
) -> Result<ResolvedFossilIdentity, crunch_project::Error> {
    match selector {
        FossilSelector::Checkin(checkin) => Ok(ResolvedFossilIdentity {
            checkin: checkin.clone(),
        }),
        FossilSelector::Branch(branch) => resolve_fossil_named_selector(repository, branch),
        FossilSelector::Tag(tag) => resolve_fossil_named_selector(repository, tag),
    }
}

fn hash_fossil_tree(
    repository: &str,
    identity: &ResolvedFossilIdentity,
    algo: &HashAlgo,
) -> Result<String, crunch_project::Error> {
    let fossil = find_vcs_binary("fossil", &["/usr/bin/fossil", "/bin/fossil", "/run/current-system/sw/bin/fossil"])?;
    let dir = tempdir()?;
    let checkout = dir.path().join("checkout");
    materialize_fossil_checkout(&fossil, repository, &identity.checkin, &checkout)?;
    ensure_tree_within_limits(&checkout)?;
    hash_recursive_path(&checkout, algo)
}

fn run_darcs_clone_tag<R: AsRef<str>, T: AsRef<str>>(
    darcs: &Path,
    repository: R,
    tag: T,
    checkout: &Path,
) -> Result<(), crunch_project::Error> {
    let repository = repository.as_ref();
    let tag = tag.as_ref();
    let mut command = Command::new(darcs);
    command.args(["clone", "--quiet", "--tag", tag, repository]).arg(checkout);
    run_status_command(command, &format!("darcs clone tag {tag} from {repository}"))
}

fn run_darcs_clone_context(
    darcs: &Path,
    repository: &str,
    context_file: &Path,
    checkout: &Path,
) -> Result<(), crunch_project::Error> {
    let mut command = Command::new(darcs);
    command.args(["clone", "--quiet", "--context"]).arg(context_file).arg(repository).arg(checkout);
    run_status_command(command, &format!("darcs clone context from {repository}"))
}

fn darcs_context_for_checkout(darcs: &Path, checkout: &Path) -> Result<String, crunch_project::Error> {
    let mut command = Command::new(darcs);
    command.args(["changes", "--context"]).current_dir(checkout);
    let context = run_stdout_command(command, &format!("darcs changes --context in {}", checkout.display()))?;
    require_nonempty_command_value(context, "darcs context")
}

fn run_pijul_clone<R: AsRef<str>, C: AsRef<str>>(
    pijul: &Path,
    repository: R,
    channel: C,
    checkout: &Path,
) -> Result<(), crunch_project::Error> {
    let repository = repository.as_ref();
    let channel = channel.as_ref();
    let mut command = Command::new(pijul);
    command.arg("clone");
    if !channel.is_empty() {
        command.args(["--channel", channel]);
    }
    command.arg(repository).arg(checkout);
    run_status_command(command, &format!("pijul clone from {repository}"))
}

fn run_pijul_reset(pijul: &Path, checkout: &Path, state: &str) -> Result<(), crunch_project::Error> {
    let mut command = Command::new(pijul);
    command.args(["reset", "--state", state]).current_dir(checkout);
    run_status_command(command, &format!("pijul reset --state {state}"))
}

fn pijul_channel_state(pijul: &Path, checkout: &Path, channel: &str) -> Result<String, crunch_project::Error> {
    let mut command = Command::new(pijul);
    command.arg("channel").current_dir(checkout);
    let output = run_stdout_command(command, &format!("pijul channel in {}", checkout.display()))?;
    parse_pijul_channel_state(&output, channel)
}

fn parse_pijul_channel_state<O: AsRef<str>, C: AsRef<str>>(
    output: O,
    channel: C,
) -> Result<String, crunch_project::Error> {
    let output = output.as_ref();
    let channel = channel.as_ref();
    for line in output.lines() {
        if !line.contains(channel) {
            continue;
        }
        if let Some((_, state)) = line.rsplit_once(' ') {
            return require_nonempty_command_value(state.trim().to_string(), "pijul channel state");
        }
    }
    Err(crunch_project::Error::Manifest(format!("pijul channel state not found for channel '{channel}'")))
}

fn resolve_fossil_named_selector<R: AsRef<str>, S: AsRef<str>>(
    repository: R,
    selector: S,
) -> Result<ResolvedFossilIdentity, crunch_project::Error> {
    let repository = repository.as_ref();
    let selector = selector.as_ref();
    let fossil = find_vcs_binary("fossil", &["/usr/bin/fossil", "/bin/fossil", "/run/current-system/sw/bin/fossil"])?;
    let dir = tempdir()?;
    let checkout = dir.path().join("checkout");
    materialize_fossil_checkout(&fossil, repository, selector, &checkout)?;
    let checkin = fossil_checkout_id(&fossil, &checkout)?;
    Ok(ResolvedFossilIdentity { checkin })
}

fn materialize_fossil_checkout<R: AsRef<str>, S: AsRef<str>>(
    fossil: &Path,
    repository: R,
    selector: S,
    checkout: &Path,
) -> Result<(), crunch_project::Error> {
    let repository = repository.as_ref();
    let selector = selector.as_ref();
    let repo_file = checkout.with_extension("fossil");
    let mut clone = Command::new(fossil);
    clone.args(["clone", repository]).arg(&repo_file);
    run_status_command(clone, &format!("fossil clone {repository}"))?;
    std::fs::create_dir_all(checkout)?;
    let mut open = Command::new(fossil);
    open.arg("open").arg(&repo_file).current_dir(checkout);
    run_status_command(open, &format!("fossil open {}", repo_file.display()))?;
    let mut update = Command::new(fossil);
    update.args(["update", selector]).current_dir(checkout);
    run_status_command(update, &format!("fossil update {selector}"))
}

fn fossil_checkout_id(fossil: &Path, checkout: &Path) -> Result<String, crunch_project::Error> {
    let mut command = Command::new(fossil);
    command.arg("info").current_dir(checkout);
    let output = run_stdout_command(command, &format!("fossil info in {}", checkout.display()))?;
    parse_fossil_checkout_id(&output)
}

fn parse_fossil_checkout_id(output: &str) -> Result<String, crunch_project::Error> {
    for line in output.lines() {
        let trimmed = line.trim_start();
        let Some(value) = trimmed.strip_prefix("checkout:") else {
            continue;
        };
        let id = value.split_whitespace().next().unwrap_or("").to_string();
        return require_nonempty_command_value(id, "fossil check-in");
    }
    Err(crunch_project::Error::Manifest("fossil checkout id not found in `fossil info`".to_string()))
}

fn run_status_command(mut command: Command, label: &str) -> Result<(), crunch_project::Error> {
    let output = command.output().map_err(|err| crunch_project::Error::Manifest(format!("running {label}: {err}")))?;
    if output.status.success() {
        return Ok(());
    }
    Err(crunch_project::Error::Manifest(format!(
        "{label} failed: {}",
        bounded_stderr_utf8_lossy(&output.stderr)
    )))
}

fn run_stdout_command(mut command: Command, label: &str) -> Result<String, crunch_project::Error> {
    let output = command.output().map_err(|err| crunch_project::Error::Manifest(format!("running {label}: {err}")))?;
    if !output.status.success() {
        return Err(crunch_project::Error::Manifest(format!(
            "{label} failed: {}",
            bounded_stderr_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| crunch_project::Error::Manifest(format!("{label} output is not UTF-8: {err}")))
}

fn require_nonempty_command_value(value: String, label: &str) -> Result<String, crunch_project::Error> {
    if value.is_empty() {
        return Err(crunch_project::Error::Manifest(format!("{label} was empty")));
    }
    Ok(value)
}

fn run_git_clone_bare(git: &Path, repository: &str, bare: &Path) -> Result<(), crunch_project::Error> {
    let output = Command::new(git)
        .args(["clone", "--bare", repository])
        .arg(bare)
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git clone for {repository}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git clone failed for {repository}: {stderr}")));
    }
    Ok(())
}

fn run_git_checkout_tree(git: &Path, bare: &Path, checkout: &Path, rev: &str) -> Result<(), crunch_project::Error> {
    std::fs::create_dir_all(checkout)?;
    let output = Command::new(git)
        .arg("--git-dir")
        .arg(bare)
        .arg("--work-tree")
        .arg(checkout)
        .args(["checkout", rev, "--", "."])
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git checkout for {rev}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git checkout failed for {rev}: {stderr}")));
    }
    Ok(())
}

fn hash_flat_file(path: &Path, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let file = std::fs::File::open(path)
        .map_err(|err| crunch_project::Error::Manifest(format!("opening {}: {err}", path.display())))?;
    let bounded = BoundedReader::new(io::BufReader::new(file), MAX_DOWNLOAD_BYTES, path.display().to_string());
    hash_reader_to_sri(bounded, algo, &format!("local file {}", path.display()))
}

fn hash_reader_to_sri<R: Read>(mut reader: R, algo: &HashAlgo, label: &str) -> Result<String, crunch_project::Error> {
    assert!(!label.is_empty(), "hash input label must not be empty");
    let mut buffer = [0u8; PROJECT_READ_BUFFER_BYTES];
    assert!(!buffer.is_empty(), "hash read buffer must not be empty");
    match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            read_into_hasher(&mut reader, &mut buffer, &mut hasher, label)?;
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash).to_sri_string())
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            read_into_hasher(&mut reader, &mut buffer, &mut hasher, label)?;
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)).to_sri_string())
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            for _ in 0..PROJECT_READ_CHUNK_COUNT_MAX {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|err| crunch_project::Error::Manifest(format!("reading {label}: {err}")))?;
                if read == 0 {
                    return Ok(NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes()).to_sri_string());
                }
                hasher.update(&buffer[..read]);
            }
            Err(crunch_project::Error::Manifest(format!(
                "reading {label} exceeded {PROJECT_READ_CHUNK_COUNT_MAX} chunks"
            )))
        }
    }
}

fn read_into_hasher<R: Read, H: Digest>(
    reader: &mut R,
    buffer: &mut [u8],
    hasher: &mut H,
    label: &str,
) -> Result<(), crunch_project::Error> {
    for _ in 0..PROJECT_READ_CHUNK_COUNT_MAX {
        let read = reader
            .read(buffer)
            .map_err(|err| crunch_project::Error::Manifest(format!("reading {label}: {err}")))?;
        if read == 0 {
            return Ok(());
        }
        hasher.update(&buffer[..read]);
    }
    Err(crunch_project::Error::Manifest(format!(
        "reading {label} exceeded {PROJECT_READ_CHUNK_COUNT_MAX} chunks"
    )))
}

fn hash_recursive_path(path: &Path, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let root = path.to_path_buf();
    debug_assert_eq!(root.as_path(), path);
    debug_assert_eq!(root.is_absolute(), path.is_absolute());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| crunch_project::Error::Manifest(format!("creating hash runtime: {err}")))?;
    rt.block_on(async move {
        let blob_service = MemoryBlobServiceConfig {}
            .build("project-refresh", &CompositionContext::blank(&REG))
            .await
            .map_err(|err| crunch_project::Error::Manifest(format!("creating temporary blob service: {err}")))?;
        let directory_service =
            RedbDirectoryService::new_temporary("project-refresh".to_string(), RedbDirectoryServiceConfig {
                path: None,
                cache_size: None,
                read_only: false,
            })
            .map_err(|err| crunch_project::Error::Manifest(format!("creating temporary directory service: {err}")))?;
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), &root, None)
            .await
            .map_err(|err| crunch_project::Error::Manifest(format!("ingesting {}: {err}", root.display())))?;
        nar_hash_to_sri(&node, algo, blob_service, directory_service).await
    })
}

async fn nar_hash_to_sri(
    node: &Node,
    algo: &HashAlgo,
    blob_service: std::sync::Arc<dyn BlobService>,
    directory_service: RedbDirectoryService,
) -> Result<String, crunch_project::Error> {
    let nix_hash = match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            let hash: [u8; 32] = hasher.finalize().into();
            NixHash::Sha256(hash)
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            let hash: [u8; 64] = hasher.finalize().into();
            NixHash::Sha512(Box::new(hash))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes())
        }
    };
    let sri = nix_hash.to_sri_string();
    debug_assert!(!sri.is_empty());
    debug_assert!(sri.contains('-'));
    Ok(sri)
}

fn ensure_tree_within_limits(path: &Path) -> Result<(), crunch_project::Error> {
    let mut pending = vec![path.to_path_buf()];
    let mut entries_seen: u32 = 0;
    let mut bytes_total: u64 = 0;
    while let Some(current) = pending.pop() {
        entries_seen = entries_seen.saturating_add(1);
        if entries_seen > MAX_TREE_ENTRIES {
            return Err(crunch_project::Error::Manifest(format!(
                "materialized tree exceeds {MAX_TREE_ENTRIES} entries: {}",
                path.display()
            )));
        }
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|err| crunch_project::Error::Manifest(format!("stat {}: {err}", current.display())))?;
        if metadata.is_file() {
            bytes_total = bytes_total.saturating_add(metadata.len());
            if bytes_total > MAX_TREE_BYTES {
                return Err(crunch_project::Error::Manifest(format!(
                    "materialized tree exceeds {MAX_TREE_BYTES} bytes: {}",
                    path.display()
                )));
            }
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        for child in std::fs::read_dir(&current)
            .map_err(|err| crunch_project::Error::Manifest(format!("read_dir {}: {err}", current.display())))?
        {
            let child = child
                .map_err(|err| crunch_project::Error::Manifest(format!("walking {}: {err}", current.display())))?;
            pending.push(child.path());
        }
    }
    debug_assert!(entries_seen <= MAX_TREE_ENTRIES);
    debug_assert!(bytes_total <= MAX_TREE_BYTES);
    Ok(())
}

fn resolve_local_path(project_root: &Path, path: &str) -> PathBuf {
    let source = Path::new(path);
    if source.is_absolute() {
        source.to_path_buf()
    } else {
        project_root.join(source)
    }
}

fn path_to_str<'a>(path: &'a Path, what: &str) -> Result<&'a str, crunch_project::Error> {
    path.to_str()
        .ok_or_else(|| crunch_project::Error::Manifest(format!("{what} is not valid UTF-8: {}", path.display())))
}

fn find_git_binary() -> Result<PathBuf, crunch_project::Error> {
    find_vcs_binary("git", &[
        "/usr/bin/git",
        "/bin/git",
        "/usr/local/bin/git",
        "/run/current-system/sw/bin/git",
    ])
}

fn find_vcs_binary(name: &str, absolute_candidates: &[&str]) -> Result<PathBuf, crunch_project::Error> {
    assert!(!name.is_empty(), "VCS tool name must not be empty");
    assert!(
        absolute_candidates.iter().all(|candidate| Path::new(candidate).is_absolute()),
        "VCS fixed candidates must be absolute"
    );
    for candidate in absolute_candidates {
        let path = Path::new(candidate);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }
    if let Ok(user) = std::env::var("USER") {
        let profile = PathBuf::from(format!("/etc/profiles/per-user/{user}/bin/{name}"));
        if profile.exists() {
            return Ok(profile);
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join(name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    Err(crunch_project::Error::Manifest(format!("unsupported-VCS-tool: {name} not found in PATH")))
}

fn open_url_reader(url: &str) -> Result<Box<dyn Read + Send>, crunch_project::Error> {
    if let Some(path) = url.strip_prefix("file://") {
        let file = std::fs::File::open(path)
            .map_err(|err| crunch_project::Error::Manifest(format!("opening local file URL {url}: {err}")))?;
        return Ok(Box::new(io::BufReader::new(file)));
    }
    let response = fetch_agent()
        .get(url)
        .call()
        .map_err(|err| crunch_project::Error::Manifest(format!("downloading {url}: {err}")))?;
    Ok(Box::new(response.into_body().into_reader()))
}

fn fetch_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FETCH_READ_TIMEOUT_SECS)))
        .timeout_connect(Some(std::time::Duration::from_secs(FETCH_CONNECT_TIMEOUT_SECS)))
        .build()
        .new_agent()
}

struct BoundedReader<R> {
    inner: R,
    limit_bytes: u64,
    seen_bytes: u64,
    label: String,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, limit_bytes: u64, label: String) -> Self {
        assert!(limit_bytes > 0, "limit must be positive");
        assert!(!label.is_empty(), "label must not be empty");
        Self {
            inner,
            limit_bytes,
            seen_bytes: 0,
            label,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.seen_bytes == self.limit_bytes {
            let mut extra = [0u8; 1];
            let read = self.inner.read(&mut extra)?;
            if read == 0 {
                return Ok(0);
            }
            return Err(io::Error::other(format!("download exceeds {} bytes: {}", self.limit_bytes, self.label)));
        }
        let remaining = self.limit_bytes.saturating_sub(self.seen_bytes);
        let buffer_len_u64 = u64::try_from(buf.len())
            .map_err(|_| io::Error::other(format!("read buffer size does not fit u64: {}", self.label)))?;
        let allowed_u64 = remaining.min(buffer_len_u64);
        let allowed = usize::try_from(allowed_u64)
            .map_err(|_| io::Error::other(format!("read bound does not fit usize: {}", self.label)))?;
        let read = self.inner.read(&mut buf[..allowed])?;
        let read_u64 =
            u64::try_from(read).map_err(|_| io::Error::other(format!("read size does not fit u64: {}", self.label)))?;
        self.seen_bytes = self.seen_bytes.saturating_add(read_u64);
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRUST_TEST_HASH: &str = "sha256-trusted=";
    const TRUST_TEST_KEYPAIR: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    fn signed_trust_policy(root: &Path, subject: &TrustSubject, hash: &LockedHash) -> crunch_project::InputTrustPolicy {
        let (signing_key, verifying_key) =
            nix_compat::narinfo::parse_keypair(TRUST_TEST_KEYPAIR).expect("dummy keypair should parse");
        let payload =
            trust_signature_payload(subject, crunch_project::TrustDigestBinding::ContentHash, &hash.algo, &hash.value);
        let signature = signing_key.sign(payload.as_bytes()).to_owned().to_string();
        std::fs::write(root.join("pkg.sig"), signature).expect("signature fixture should be written");
        crunch_project::InputTrustPolicy {
            verifier: crunch_project::TrustVerifierKind::Ed25519Detached,
            signatures: vec![TrustSignatureRef::LocalFile {
                path: "pkg.sig".to_string(),
            }],
            trusted_public_keys: vec![verifying_key.to_string()],
            required_signers: vec![verifying_key.name().to_string()],
            quorum: 1,
            digest_binding: crunch_project::TrustDigestBinding::ContentHash,
        }
    }

    #[test]
    fn oversized_shell_observation_becomes_failed_observation() {
        let too_large_value = "x".repeat(crunch_project::MAX_FRESHNESS_VALUE_BYTES as usize + 1);
        let observation = freshness_status(FreshnessStatusRequest {
            input_name: "pkg",
            probe_kind: FreshnessProbeKind::HttpText,
            is_network_required: true,
            status: FreshnessObservationStatus::Observed,
            value: Some(too_large_value),
            diagnostic: String::new(),
            probe_identity_digest: Some("blake3:probe".to_string()),
        });

        assert_eq!(observation.input_name, "pkg");
        assert_eq!(observation.status, FreshnessObservationStatus::Failed);
        assert_eq!(observation.value, None);
        assert!(observation.value_digest.is_none());
        assert!(observation.diagnostic.contains("rejected"));
    }

    #[test]
    fn bounded_stderr_preserves_valid_text_within_limit() {
        let rendered = bounded_stderr_utf8_lossy(b"freshness-ok");

        assert_eq!(rendered, "freshness-ok");
        assert_eq!(rendered.len(), "freshness-ok".len());
    }

    #[test]
    fn bounded_stderr_replaces_malformed_utf8_without_exceeding_limit() {
        let malformed = vec![0xff; FRESHNESS_STDERR_LIMIT_BYTES_USIZE + 1];
        let rendered = bounded_stderr_utf8_lossy(&malformed);

        assert!(rendered.contains('\u{fffd}'));
        assert!(rendered.len() <= FRESHNESS_STDERR_LIMIT_BYTES_USIZE.saturating_mul(3));
    }

    #[test]
    fn local_project_trust_signature_produces_verified_fact() {
        let temp = tempfile::tempdir().expect("temp project should be created");
        let subject = TrustSubject::input("pkg".to_string());
        let hash = LockedHash {
            algo: HashAlgo::Sha256,
            value: TRUST_TEST_HASH.to_string(),
        };
        let policy = signed_trust_policy(temp.path(), &subject, &hash);

        let facts = verify_project_trust_policy(temp.path(), &subject, Some(&policy), &hash)
            .expect("valid local signature should verify");

        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].subject, subject);
        assert_eq!(facts[0].hash_value, TRUST_TEST_HASH);
    }

    #[test]
    fn local_project_trust_signature_with_wrong_digest_yields_no_fact() {
        let temp = tempfile::tempdir().expect("temp project should be created");
        let subject = TrustSubject::input("pkg".to_string());
        let signed_hash = LockedHash {
            algo: HashAlgo::Sha256,
            value: TRUST_TEST_HASH.to_string(),
        };
        let policy = signed_trust_policy(temp.path(), &subject, &signed_hash);
        let other_hash = LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-other=".to_string(),
        };

        let facts = verify_project_trust_policy(temp.path(), &subject, Some(&policy), &other_hash)
            .expect("wrong digest should not require a key-server lookup or panic");

        assert!(facts.is_empty());
    }

    #[test]
    fn parse_branch_output_returns_matching_rev() {
        let output = "0123456789abcdef0123456789abcdef01234567\trefs/heads/main\n";
        let rev = parse_branch_output(output, "main").unwrap();
        assert_eq!(rev, "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn parse_tag_output_prefers_peeled_commit() {
        let output = concat!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\trefs/tags/v1.0\n",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\trefs/tags/v1.0^{}\n",
        );
        let rev = parse_tag_output(output, "v1.0").unwrap();
        assert_eq!(rev, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    }

    #[test]
    fn parse_tag_output_falls_back_to_direct_tag() {
        let output = "cccccccccccccccccccccccccccccccccccccccc\trefs/tags/v1.0\n";
        let rev = parse_tag_output(output, "v1.0").unwrap();
        assert_eq!(rev, "cccccccccccccccccccccccccccccccccccccccc");
    }

    #[test]
    fn validate_git_rev_rejects_short_rev() {
        let err = validate_git_rev("deadbeef").unwrap_err();
        assert!(err.to_string().contains("40 hex"));
    }

    #[test]
    fn missing_vcs_tool_reports_unsupported_blocker() {
        let err = find_vcs_binary("mantle-definitely-missing-vcs-tool", &[]).unwrap_err();
        assert!(err.to_string().contains("unsupported-VCS-tool"));
        assert!(err.to_string().contains("mantle-definitely-missing-vcs-tool"));
    }

    #[test]
    fn parse_pijul_channel_state_uses_named_channel_line() {
        let output = "  dev abc\n* main state-main\n";
        let state = parse_pijul_channel_state(output, "main").unwrap();
        assert_eq!(state, "state-main");
    }

    #[test]
    fn pijul_change_selector_fails_closed_without_state_proof() {
        let err = resolve_pijul_identity("https://example.invalid/repo", &PijulSelector::Change {
            channel: "main".into(),
            change: "change".into(),
        })
        .unwrap_err();

        assert!(err.to_string().contains("unsupported-VCS-tool"));
        assert!(err.to_string().contains("cannot prove channel state"));
    }

    #[test]
    fn parse_fossil_checkout_id_reads_checkout_line() {
        let output = "repository: /tmp/repo.fossil\ncheckout: 0123456789abcdef tags: trunk\n";
        let checkin = parse_fossil_checkout_id(output).unwrap();
        assert_eq!(checkin, "0123456789abcdef");
    }
}
