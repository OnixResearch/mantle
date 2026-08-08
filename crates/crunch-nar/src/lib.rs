#![cfg_attr(not(kani), feature(register_tool))]
#![register_tool(tigerstyle)]

#[cfg(not(unix))]
compile_error!("crunch-nar requires Unix byte-safe filesystem APIs");

use std::collections::BTreeSet;
use std::fs::Metadata;
use std::io;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

use digest::Digest;
use nix_archive::nar;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use serde::Serialize;
use thiserror::Error;

pub const ADAPTER_VERSION: &str = "mantle-filesystem-nar-v1";
pub const UPSTREAM_PACKAGE: &str = "nix-archive";
pub const UPSTREAM_VERSION: &str = "0.1.0";
pub const UPSTREAM_COMMIT: &str = "14362ab589daa4869bda744d4fbe26a1914b5491";
pub const UPSTREAM_CARGO_CHECKSUM: &str = "70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1";
pub const PARITY_CASE_COUNT_MAX: usize = 64;
const EXPECTED_COMPARISON_FIELD_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseHackPolicy {
    Disabled,
    Enabled,
}

impl CaseHackPolicy {
    #[must_use]
    pub const fn native() -> Self {
        if cfg!(target_os = "macos") {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    const fn upstream(self) -> nar::CaseHack {
        match self {
            Self::Disabled => nar::CaseHack::Disabled,
            Self::Enabled => nar::CaseHack::Enabled,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FilesystemNarRequest {
    pub algorithm: HashAlgo,
    pub case_hack: CaseHackPolicy,
    pub nar_bytes_max: u64,
}

impl FilesystemNarRequest {
    #[must_use]
    pub const fn new(algorithm: HashAlgo, case_hack: CaseHackPolicy, nar_bytes_max: u64) -> Self {
        Self {
            algorithm,
            case_hack,
            nar_bytes_max,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesystemNarObservation {
    pub algorithm: HashAlgo,
    pub case_hack: CaseHackPolicy,
    pub nar_size: u64,
    pub digest: NixHash,
}

impl FilesystemNarObservation {
    #[must_use]
    pub fn evidence(&self) -> FilesystemNarEvidence {
        assert_eq!(self.algorithm, self.digest.algo(), "observation algorithm must match digest");
        assert!(self.nar_size > 0, "a NAR observation must include the archive header");
        FilesystemNarEvidence {
            schema: ADAPTER_VERSION,
            upstream_package: UPSTREAM_PACKAGE,
            upstream_version: UPSTREAM_VERSION,
            upstream_commit: UPSTREAM_COMMIT,
            upstream_cargo_checksum: UPSTREAM_CARGO_CHECKSUM,
            case_hack: self.case_hack,
            algorithm: self.algorithm.to_string(),
            nar_size: self.nar_size,
            digest_hex: data_encoding::HEXLOWER.encode(self.digest.digest_as_bytes()),
            digest_sri: self.digest.to_sri_string(),
            disposition: "success",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FilesystemNarEvidence {
    pub schema: &'static str,
    pub upstream_package: &'static str,
    pub upstream_version: &'static str,
    pub upstream_commit: &'static str,
    pub upstream_cargo_checksum: &'static str,
    pub case_hack: CaseHackPolicy,
    pub algorithm: String,
    pub nar_size: u64,
    pub digest_hex: String,
    pub digest_sri: String,
    pub disposition: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedNarFacts {
    pub algorithm: HashAlgo,
    pub case_hack: CaseHackPolicy,
    pub nar_size: u64,
    pub digest: NixHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NarFactMismatch {
    Algorithm,
    CaseHack,
    NarSize,
    Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NarComparison {
    Match,
    Mismatch(Vec<NarFactMismatch>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OptionalOracleDisposition {
    Matched,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityComparison {
    pub case_id: String,
    pub bytes_match: bool,
    pub size_match: bool,
    pub digests_match: bool,
    pub errors_match: bool,
}

impl ParityComparison {
    #[must_use]
    pub fn is_match(&self) -> bool {
        !self.case_id.is_empty() && self.bytes_match && self.size_match && self.digests_match && self.errors_match
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutoverEvidence {
    pub upstream_version: String,
    pub upstream_commit: String,
    pub upstream_cargo_checksum: String,
    pub platform_supported: bool,
    pub required_cases: Vec<String>,
    pub comparisons: Vec<ParityComparison>,
    pub nix_oracle: OptionalOracleDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutoverDecision {
    pub accepted: bool,
    pub nix_oracle: OptionalOracleDisposition,
    pub rejections: Vec<String>,
}

#[derive(Debug, Error)]
pub enum FilesystemNarError {
    #[error("filesystem NAR path must not be empty")]
    EmptyPath,
    #[error("filesystem NAR byte limit must be positive")]
    InvalidByteLimit,
    #[error("unsupported NAR hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),
    #[error("filesystem NAR exceeded the {limit_bytes}-byte limit")]
    NarBytesLimitExceeded { limit_bytes: u64 },
    #[error("filesystem NAR root changed during observation: {0}")]
    RootChanged(PathBuf),
    #[error("filesystem NAR blocking worker failed: {0}")]
    Worker(String),
    #[error(transparent)]
    Upstream(#[from] nar::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RootIdentity {
    device: u64,
    inode: u64,
    mode: u32,
    size_bytes: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl RootIdentity {
    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            size_bytes: metadata.size(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

enum DigestState {
    Md5(md5::Md5),
    Sha1(sha1::Sha1),
    Sha256(sha2::Sha256),
    Sha512(sha2::Sha512),
    Blake3(Box<blake3::Hasher>),
}

impl DigestState {
    fn new(algorithm: HashAlgo) -> Self {
        match algorithm {
            HashAlgo::Md5 => Self::Md5(md5::Md5::new()),
            HashAlgo::Sha1 => Self::Sha1(sha1::Sha1::new()),
            HashAlgo::Sha256 => Self::Sha256(sha2::Sha256::new()),
            HashAlgo::Sha512 => Self::Sha512(sha2::Sha512::new()),
            HashAlgo::Blake3 => Self::Blake3(Box::new(blake3::Hasher::new())),
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        match self {
            Self::Md5(hasher) => hasher.update(bytes),
            Self::Sha1(hasher) => hasher.update(bytes),
            Self::Sha256(hasher) => hasher.update(bytes),
            Self::Sha512(hasher) => hasher.update(bytes),
            Self::Blake3(hasher) => {
                hasher.update(bytes);
            }
        }
    }

    fn finish(self) -> NixHash {
        match self {
            Self::Md5(hasher) => NixHash::Md5(hasher.finalize().into()),
            Self::Sha1(hasher) => NixHash::Sha1(hasher.finalize().into()),
            Self::Sha256(hasher) => NixHash::Sha256(hasher.finalize().into()),
            Self::Sha512(hasher) => NixHash::Sha512(Box::new(hasher.finalize().into())),
            Self::Blake3(hasher) => NixHash::Blake3(*blake3::Hasher::finalize(hasher.as_ref()).as_bytes()),
        }
    }
}

struct BoundedDigestWriter {
    state: DigestState,
    bytes_written: u64,
    bytes_max: u64,
    limit_exceeded: bool,
}

impl BoundedDigestWriter {
    fn new(algorithm: HashAlgo, bytes_max: u64) -> Self {
        assert!(bytes_max > 0, "digest writer limit must be positive");
        Self {
            state: DigestState::new(algorithm),
            bytes_written: 0,
            bytes_max,
            limit_exceeded: false,
        }
    }

    fn finish(self) -> (u64, NixHash) {
        assert!(!self.limit_exceeded, "a limited writer cannot finish after limit failure");
        assert!(self.bytes_written <= self.bytes_max, "writer byte count must stay within the limit");
        (self.bytes_written, self.state.finish())
    }
}

impl Write for BoundedDigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let bytes_len =
            u64::try_from(bytes.len()).map_err(|_| io::Error::other("NAR write length does not fit u64"))?;
        let next = self
            .bytes_written
            .checked_add(bytes_len)
            .ok_or_else(|| io::Error::other("NAR byte count overflowed u64"))?;
        if next > self.bytes_max {
            self.limit_exceeded = true;
            return Err(io::Error::new(io::ErrorKind::FileTooLarge, "filesystem NAR byte limit exceeded"));
        }
        self.state.update(bytes);
        self.bytes_written = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
struct BoundedVecWriter {
    bytes: Vec<u8>,
    bytes_max: u64,
    limit_exceeded: bool,
}

#[cfg(test)]
impl BoundedVecWriter {
    fn new(bytes_max: u64) -> Self {
        assert!(bytes_max > 0, "vector writer limit must be positive");
        Self {
            bytes: Vec::new(),
            bytes_max,
            limit_exceeded: false,
        }
    }
}

#[cfg(test)]
impl Write for BoundedVecWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let current =
            u64::try_from(self.bytes.len()).map_err(|_| io::Error::other("NAR buffer length does not fit u64"))?;
        let additional =
            u64::try_from(bytes.len()).map_err(|_| io::Error::other("NAR write length does not fit u64"))?;
        let next = current
            .checked_add(additional)
            .ok_or_else(|| io::Error::other("NAR buffer length overflowed u64"))?;
        if next > self.bytes_max {
            self.limit_exceeded = true;
            return Err(io::Error::new(io::ErrorKind::FileTooLarge, "filesystem NAR byte limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn observe_path(
    path: &Path,
    request: FilesystemNarRequest,
) -> Result<FilesystemNarObservation, FilesystemNarError> {
    observe_path_with_hook(path, request, || {})
}

pub async fn observe_path_blocking(
    path: PathBuf,
    request: FilesystemNarRequest,
) -> Result<FilesystemNarObservation, FilesystemNarError> {
    if path.as_os_str().is_empty() {
        return Err(FilesystemNarError::EmptyPath);
    }
    if request.nar_bytes_max == 0 {
        return Err(FilesystemNarError::InvalidByteLimit);
    }
    let worker = tokio::task::spawn_blocking(move || observe_path(&path, request)).await;
    map_worker_result(worker.map_err(|error| error.to_string()))
}

#[cfg(test)]
fn encode_path_bytes_bounded(
    path: &Path,
    case_hack: CaseHackPolicy,
    nar_bytes_max: u64,
) -> Result<Vec<u8>, FilesystemNarError> {
    validate_byte_request(path, nar_bytes_max)?;
    let before = root_identity(path)?;
    let mut writer = BoundedVecWriter::new(nar_bytes_max);
    let encoded = nar::encode_path_with_case_hack(&mut writer, path, case_hack.upstream());
    if writer.limit_exceeded {
        return Err(FilesystemNarError::NarBytesLimitExceeded {
            limit_bytes: nar_bytes_max,
        });
    }
    encoded?;
    let after = root_identity(path)?;
    if before != after {
        return Err(FilesystemNarError::RootChanged(path.to_path_buf()));
    }
    assert!(!writer.bytes.is_empty(), "a complete NAR must not be empty");
    Ok(writer.bytes)
}

pub fn compare_nar_facts(expected: &ExpectedNarFacts, observed: &FilesystemNarObservation) -> NarComparison {
    let mut mismatches = Vec::with_capacity(EXPECTED_COMPARISON_FIELD_COUNT);
    if expected.algorithm != observed.algorithm {
        mismatches.push(NarFactMismatch::Algorithm);
    }
    if expected.case_hack != observed.case_hack {
        mismatches.push(NarFactMismatch::CaseHack);
    }
    if expected.nar_size != observed.nar_size {
        mismatches.push(NarFactMismatch::NarSize);
    }
    if expected.digest != observed.digest {
        mismatches.push(NarFactMismatch::Digest);
    }
    if mismatches.is_empty() {
        NarComparison::Match
    } else {
        NarComparison::Mismatch(mismatches)
    }
}

pub fn evaluate_cutover(evidence: &CutoverEvidence) -> CutoverDecision {
    let mut rejections = Vec::new();
    validate_upstream_identity(evidence, &mut rejections);
    if !evidence.platform_supported {
        rejections.push("unsupported-platform".to_string());
    }
    validate_parity_cases(evidence, &mut rejections);
    CutoverDecision {
        accepted: rejections.is_empty(),
        nix_oracle: evidence.nix_oracle,
        rejections,
    }
}

pub fn parse_hash_algorithm(value: &str) -> Result<HashAlgo, FilesystemNarError> {
    match value {
        "md5" => Ok(HashAlgo::Md5),
        "sha1" => Ok(HashAlgo::Sha1),
        "sha256" => Ok(HashAlgo::Sha256),
        "sha512" => Ok(HashAlgo::Sha512),
        "blake3" => Ok(HashAlgo::Blake3),
        other => Err(FilesystemNarError::UnsupportedHashAlgorithm(other.to_string())),
    }
}

pub fn digest_bytes(bytes: &[u8], algorithm: HashAlgo) -> NixHash {
    let mut state = DigestState::new(algorithm);
    state.update(bytes);
    let digest = state.finish();
    assert_eq!(digest.algo(), algorithm, "byte digest must use the requested algorithm");
    digest
}

fn observe_path_with_hook(
    path: &Path,
    request: FilesystemNarRequest,
    before_encode: impl FnOnce(),
) -> Result<FilesystemNarObservation, FilesystemNarError> {
    validate_request(path, request)?;
    let before = root_identity(path)?;
    before_encode();
    let (nar_size, digest) = observe_path_inner(path, request)?;
    let after = root_identity(path)?;
    if before != after {
        return Err(FilesystemNarError::RootChanged(path.to_path_buf()));
    }
    assert!(nar_size <= request.nar_bytes_max, "successful NAR observation must obey its byte limit");
    assert_eq!(digest.algo(), request.algorithm, "successful NAR digest must use the requested algorithm");
    Ok(FilesystemNarObservation {
        algorithm: request.algorithm,
        case_hack: request.case_hack,
        nar_size,
        digest,
    })
}

fn observe_path_inner(path: &Path, request: FilesystemNarRequest) -> Result<(u64, NixHash), FilesystemNarError> {
    if request.algorithm == HashAlgo::Sha256 {
        let (nar_size, digest) = nar::hash_path_with_case_hack(path, request.case_hack.upstream())?;
        if nar_size > request.nar_bytes_max {
            return Err(FilesystemNarError::NarBytesLimitExceeded {
                limit_bytes: request.nar_bytes_max,
            });
        }
        return Ok((nar_size, NixHash::Sha256(digest)));
    }
    let mut writer = BoundedDigestWriter::new(request.algorithm, request.nar_bytes_max);
    let encoded = nar::encode_path_with_case_hack(&mut writer, path, request.case_hack.upstream());
    if writer.limit_exceeded {
        return Err(FilesystemNarError::NarBytesLimitExceeded {
            limit_bytes: request.nar_bytes_max,
        });
    }
    encoded?;
    Ok(writer.finish())
}

fn validate_request(path: &Path, request: FilesystemNarRequest) -> Result<(), FilesystemNarError> {
    validate_byte_request(path, request.nar_bytes_max)
}

fn validate_byte_request(path: &Path, nar_bytes_max: u64) -> Result<(), FilesystemNarError> {
    if path.as_os_str().is_empty() {
        return Err(FilesystemNarError::EmptyPath);
    }
    if nar_bytes_max == 0 {
        return Err(FilesystemNarError::InvalidByteLimit);
    }
    Ok(())
}

fn root_identity(path: &Path) -> Result<RootIdentity, FilesystemNarError> {
    let metadata = std::fs::symlink_metadata(path)?;
    let identity = RootIdentity::from_metadata(&metadata);
    assert!(identity.inode > 0, "observed filesystem roots must have a nonzero inode");
    assert!(identity.mode > 0, "observed filesystem roots must have a nonzero mode");
    Ok(identity)
}

fn map_worker_result<T>(worker: Result<Result<T, FilesystemNarError>, String>) -> Result<T, FilesystemNarError> {
    match worker {
        Ok(result) => result,
        Err(error) => Err(FilesystemNarError::Worker(error)),
    }
}

fn validate_upstream_identity(evidence: &CutoverEvidence, rejections: &mut Vec<String>) {
    assert!(rejections.len() <= PARITY_CASE_COUNT_MAX, "upstream validation rejection set must be bounded");
    assert!(!UPSTREAM_VERSION.is_empty(), "accepted upstream version must not be empty");
    if evidence.upstream_version != UPSTREAM_VERSION {
        rejections.push("stale-upstream-version".to_string());
    }
    if evidence.upstream_commit != UPSTREAM_COMMIT {
        rejections.push("stale-upstream-commit".to_string());
    }
    if evidence.upstream_cargo_checksum != UPSTREAM_CARGO_CHECKSUM {
        rejections.push("stale-upstream-cargo-checksum".to_string());
    }
}

fn validate_parity_cases(evidence: &CutoverEvidence, rejections: &mut Vec<String>) {
    if evidence.required_cases.len() > PARITY_CASE_COUNT_MAX || evidence.comparisons.len() > PARITY_CASE_COUNT_MAX {
        rejections.push("parity-case-limit-exceeded".to_string());
        return;
    }
    let mut required = BTreeSet::new();
    for case_id in &evidence.required_cases {
        if case_id.is_empty() || !required.insert(case_id.as_str()) {
            rejections.push(format!("invalid-required-case:{case_id}"));
        }
    }
    let mut observed = BTreeSet::new();
    for comparison in &evidence.comparisons {
        if !observed.insert(comparison.case_id.as_str()) {
            rejections.push(format!("duplicate-comparison:{}", comparison.case_id));
            continue;
        }
        if required.contains(comparison.case_id.as_str()) && !comparison.is_match() {
            rejections.push(format!("parity-mismatch:{}", comparison.case_id));
        }
    }
    for case_id in required.difference(&observed) {
        rejections.push(format!("missing-parity-case:{case_id}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_NAR_BYTES_MAX: u64 = 1_048_576;

    #[test]
    fn fact_comparison_accepts_equal_facts_and_rejects_case_hack_drift() {
        let temporary = tempfile::tempdir().unwrap();
        let file = temporary.path().join("file");
        std::fs::write(&file, b"facts").unwrap();
        let observed = observe_path(
            &file,
            FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Disabled, TEST_NAR_BYTES_MAX),
        )
        .unwrap();
        let observation_evidence = observed.evidence();
        assert_eq!(observation_evidence.disposition, "success");
        assert_eq!(observation_evidence.upstream_commit, UPSTREAM_COMMIT);
        let expected = ExpectedNarFacts {
            algorithm: observed.algorithm,
            case_hack: observed.case_hack,
            nar_size: observed.nar_size,
            digest: observed.digest.clone(),
        };
        assert_eq!(compare_nar_facts(&expected, &observed), NarComparison::Match);

        let mismatched = ExpectedNarFacts {
            case_hack: CaseHackPolicy::Enabled,
            ..expected
        };
        assert_eq!(compare_nar_facts(&mismatched, &observed), NarComparison::Mismatch(vec![NarFactMismatch::CaseHack]));
    }

    #[test]
    fn request_rejects_empty_paths_zero_limits_and_unknown_algorithms() {
        let empty = observe_path(
            Path::new(""),
            FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Disabled, TEST_NAR_BYTES_MAX),
        );
        let temporary = tempfile::tempdir().unwrap();
        let zero_limit =
            observe_path(temporary.path(), FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Disabled, 0));
        let unsupported = parse_hash_algorithm("sha3-256");

        assert!(matches!(empty, Err(FilesystemNarError::EmptyPath)));
        assert!(matches!(zero_limit, Err(FilesystemNarError::InvalidByteLimit)));
        assert!(matches!(unsupported, Err(FilesystemNarError::UnsupportedHashAlgorithm(_))));
    }

    #[test]
    fn bounded_encoding_rejects_a_complete_archive_over_the_limit() {
        const ONE_BYTE_LIMIT: u64 = 1;
        let temporary = tempfile::tempdir().unwrap();
        let file = temporary.path().join("file");
        std::fs::write(&file, b"payload larger than one byte").unwrap();
        let result = encode_path_bytes_bounded(&file, CaseHackPolicy::Disabled, ONE_BYTE_LIMIT);
        assert!(matches!(
            result,
            Err(FilesystemNarError::NarBytesLimitExceeded {
                limit_bytes: ONE_BYTE_LIMIT
            })
        ));
        assert!(file.is_file());
    }

    #[test]
    fn root_substitution_after_admission_fails_closed() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("root");
        let moved = temporary.path().join("moved");
        std::fs::write(&root, b"admitted bytes").unwrap();
        let request = FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::Disabled, TEST_NAR_BYTES_MAX);
        let result = observe_path_with_hook(&root, request, || {
            std::fs::rename(&root, &moved).unwrap();
            std::fs::write(&root, b"substituted bytes").unwrap();
        });
        assert!(matches!(result, Err(FilesystemNarError::RootChanged(path)) if path == root));
        assert_eq!(std::fs::read(&moved).unwrap(), b"admitted bytes");
    }

    #[test]
    fn worker_result_preserves_observation_errors_and_maps_worker_failures() {
        let observation_error = map_worker_result::<()>(Ok(Err(FilesystemNarError::InvalidByteLimit)));
        let worker_error = map_worker_result::<()>(Err("worker panicked".to_string()));
        assert!(matches!(observation_error, Err(FilesystemNarError::InvalidByteLimit)));
        assert!(matches!(worker_error, Err(FilesystemNarError::Worker(message)) if message == "worker panicked"));
    }

    #[test]
    fn cutover_gate_accepts_complete_parity_and_rejects_stale_or_missing_evidence() {
        let matching = ParityComparison {
            case_id: "regular".to_string(),
            bytes_match: true,
            size_match: true,
            digests_match: true,
            errors_match: true,
        };
        let evidence = CutoverEvidence {
            upstream_version: UPSTREAM_VERSION.to_string(),
            upstream_commit: UPSTREAM_COMMIT.to_string(),
            upstream_cargo_checksum: UPSTREAM_CARGO_CHECKSUM.to_string(),
            platform_supported: cfg!(unix),
            required_cases: vec!["regular".to_string()],
            comparisons: vec![matching],
            nix_oracle: OptionalOracleDisposition::Unavailable,
        };
        let accepted = evaluate_cutover(&evidence);
        assert!(accepted.accepted);
        assert_eq!(accepted.nix_oracle, OptionalOracleDisposition::Unavailable);

        let rejected = evaluate_cutover(&CutoverEvidence {
            upstream_commit: "stale".to_string(),
            required_cases: vec!["regular".to_string(), "symlink".to_string()],
            ..evidence
        });
        assert!(!rejected.accepted);
        assert!(rejected.rejections.contains(&"stale-upstream-commit".to_string()));
        assert!(rejected.rejections.contains(&"missing-parity-case:symlink".to_string()));
    }
}
