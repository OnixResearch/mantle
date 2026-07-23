use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

const PROVIDER_METADATA_RELATIVE_PATH: &str = "share/crunch-bootstrap/provider.json";
const PROVIDER_SCHEMA: &str = "mantle-full-source-seed-provider-v1";
const PROVIDER_ID: &str = "full-source-v1";
const PROVIDER_NAME: &str = "full-source-seed-toolchain";
const PROVIDER_TARGET: &str = "x86_64-linux-musl";
const COMPILER_TARGET: &str = "x86_64-unknown-linux-musl";
const COMPILER_VERSION: &str = "10.5.0";
const DYNAMIC_LINKER: &str = "ld-musl-x86_64.so.1";
const SOURCE_AUTHORITY: &str = "declared-mantle-derivation-closure";
const RUNTIME_ADMISSION_STATUS: &str = "passed-in-builder";
const CLOSURE_ADMISSION_STATUS: &str = "derivational-closure-passed";
const OUTPUT_DIGEST_SENTINEL: &str = "external-admission-report";
const STORE_PREFIX: &str = "/mantle/store/";
const TREE_ENTRY_COUNT_MAX: usize = 65_536;
const TEXT_SCAN_BYTES_MAX: u64 = 1_048_576;
const DIAGNOSTIC_COUNT_MAX: usize = 64;
const BLAKE3_HEX_LENGTH: usize = 64;
const SOURCE_OUTPUT_COUNT: usize = 3;
const SOURCE_CLOSURE_RECORD_COUNT_MIN: usize = 1;
const SOURCE_CLOSURE_STORE_PATH_KEY: &str = "store_path";
const SOURCE_CLOSURE_LEGACY_MARKERS: &[&str] = &["musl.cc", "seed-legacy.ncl"];
const RUNTIME_SMOKE_TIMEOUT_SECS: u64 = 30;
const RUNTIME_SMOKE_POLL_INTERVAL_MS: u64 = 10;
const COMPILER_RUNTIME_SMOKE_STEP_COUNT: usize = 8;
const BINARY_TOOL_RUNTIME_SMOKE_STEP_COUNT: usize = 12;
const REJECTION_RUNTIME_SMOKE_STEP_COUNT: usize = 5;
const RUNTIME_SMOKE_STEP_COUNT: usize =
    COMPILER_RUNTIME_SMOKE_STEP_COUNT + BINARY_TOOL_RUNTIME_SMOKE_STEP_COUNT + REJECTION_RUNTIME_SMOKE_STEP_COUNT;
#[cfg(unix)]
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;

const REQUIRED_TOOLS: &[&str] = &[
    "gcc",
    "g++",
    "c++",
    "cpp",
    "gcc-ar",
    "gcc-nm",
    "gcc-ranlib",
    "ar",
    "as",
    "ld",
    "nm",
    "objcopy",
    "objdump",
    "ranlib",
    "readelf",
    "size",
    "strings",
    "strip",
];
const REQUIRED_RUNTIME_FILES: &[&str] = &[
    "crt1.o",
    "crti.o",
    "crtn.o",
    "libc.a",
    "libc.so",
    "libgcc.a",
    "libgcc_eh.a",
    "libgcc_s.so.1",
    "libstdc++.a",
    "libstdc++.so.6.0.28",
];
const REQUIRED_RUNTIME_SURFACES: &[&str] = &[
    "c-static",
    "c-dynamic",
    "cxx-static",
    "cxx-dynamic",
    "assembly-link",
    "archive-index",
    "object-inspection",
    "malformed-c",
    "malformed-cxx",
    "malformed-assembly",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "compiler-correctness",
    "bootstrap-seed-correctness",
    "independent-rebuild-agreement",
    "release-reproducibility",
    "deployment-success",
    "full-cargo-compatibility",
];
const FORBIDDEN_TREE_MARKERS: &[(&str, &str)] = &[
    ("musl.cc", "legacy-musl-provider"),
    ("seed-legacy.ncl", "legacy-seed-selection"),
    ("tinycc", "tinycc-delegation"),
    ("tcc-musl", "tinycc-delegation"),
    ("generated-stub", "generated-stub"),
    ("compatibility-stub", "generated-stub"),
    ("NON_ADMISSION", "non-admitted-component"),
];
const FORBIDDEN_SCRIPT_MARKERS: &[(&str, &str)] = &[
    ("/usr/bin/", "host-tool-fallback"),
    ("/usr/local/", "host-tool-fallback"),
    ("command -v", "host-tool-fallback"),
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FullSourceProviderMetadata {
    schema: String,
    provider_id: String,
    name: String,
    target: String,
    compiler_target: String,
    dynamic_linker: String,
    source_authority: String,
    source_outputs: SourceOutputs,
    output_identity: OutputIdentity,
    runtime_admission: RuntimeAdmission,
    closure_admission: ClosureAdmission,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SourceOutputs {
    gcc: String,
    musl: String,
    binutils: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct OutputIdentity {
    logical_store_path: String,
    content_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RuntimeAdmission {
    status: String,
    surfaces: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ClosureAdmission {
    status: String,
    state_pinned_inputs: bool,
    legacy_members: Vec<String>,
    release_generated_sources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileObservation {
    relative_path: String,
    file_size_bytes: u64,
    executable: bool,
    symlink_target: Option<String>,
    text: Option<String>,
}

#[derive(Clone, Debug)]
struct AdmissionInput {
    provider_basename: String,
    metadata: FullSourceProviderMetadata,
    files: BTreeMap<String, FileObservation>,
    output_digest_blake3: String,
    expected_output_digest_blake3: String,
    source_closure: SourceClosureObservation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceClosureObservation {
    manifest_blake3: String,
    expected_manifest_blake3: String,
    record_count: usize,
    materialized_record_count: usize,
    planned_records: Vec<String>,
    state_pinned_records: Vec<String>,
    legacy_records: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RuntimeSmokeExpectation {
    Success,
    Rejection { absent_output: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeSmokeStep {
    label: &'static str,
    program: PathBuf,
    arguments: Vec<OsString>,
    expectation: RuntimeSmokeExpectation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FullSourceProviderAdmissionReport {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) provider_id: String,
    pub(crate) provider_path: PathBuf,
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) expected_output_digest_blake3: String,
    pub(crate) source_closure_path: PathBuf,
    pub(crate) source_closure_manifest_blake3: String,
    pub(crate) expected_source_closure_manifest_blake3: String,
    pub(crate) source_closure_record_count: usize,
    pub(crate) observed_entry_count: usize,
    pub(crate) required_tool_count: usize,
    pub(crate) required_runtime_count: usize,
    pub(crate) runtime_surfaces: Vec<String>,
    pub(crate) runtime_smoke_steps: Vec<String>,
    pub(crate) source_outputs: BTreeMap<String, String>,
    pub(crate) non_claims: Vec<String>,
}

pub(crate) fn cmd_admit_full_source_provider(
    provider_dir: &Path,
    expected_output_digest_blake3: &str,
    source_closure_path: &Path,
    expected_source_closure_manifest_blake3: &str,
    report_path: &Path,
    json: bool,
) -> Result<(), RunError> {
    let report = admit_full_source_provider(
        provider_dir,
        expected_output_digest_blake3,
        source_closure_path,
        expected_source_closure_manifest_blake3,
    )?;
    let mut report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| admission_error(format!("serializing admission report: {error}")))?;
    report_bytes.push(b'\n');
    let mut report_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(report_path)
        .map_err(|error| admission_error(format!("creating report {}: {error}", report_path.display())))?;
    report_file
        .write_all(&report_bytes)
        .map_err(|error| admission_error(format!("writing report {}: {error}", report_path.display())))?;
    report_file
        .sync_all()
        .map_err(|error| admission_error(format!("syncing report {}: {error}", report_path.display())))?;
    if json {
        print!("{}", String::from_utf8_lossy(&report_bytes));
    } else {
        eprintln!("Admitted full-source provider {}", provider_dir.display());
        eprintln!("  output_digest_blake3: {}", report.output_digest_blake3);
        eprintln!("  source_closure_blake3: {}", report.source_closure_manifest_blake3);
        eprintln!("  metadata_digest_blake3: {}", report.metadata_digest_blake3);
        eprintln!("  report: {}", report_path.display());
    }
    assert_eq!(report.output_digest_blake3, expected_output_digest_blake3);
    assert_eq!(report.source_closure_manifest_blake3, expected_source_closure_manifest_blake3);
    debug_assert!(!report_bytes.is_empty());
    Ok(())
}

pub(crate) fn admit_full_source_provider(
    provider_dir: &Path,
    expected_output_digest_blake3: &str,
    source_closure_path: &Path,
    expected_source_closure_manifest_blake3: &str,
) -> Result<FullSourceProviderAdmissionReport, RunError> {
    validate_expected_digest(expected_output_digest_blake3)?;
    validate_expected_digest(expected_source_closure_manifest_blake3)?;
    let metadata_path = provider_dir.join(PROVIDER_METADATA_RELATIVE_PATH);
    let metadata_bytes = fs::read(&metadata_path)
        .map_err(|error| admission_error(format!("reading metadata {}: {error}", metadata_path.display())))?;
    let metadata: FullSourceProviderMetadata = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| admission_error(format!("parsing metadata {}: {error}", metadata_path.display())))?;
    let files = observe_provider_tree(provider_dir)?;
    let source_closure = observe_source_closure(source_closure_path, expected_source_closure_manifest_blake3)?;
    let (_, output_digest_blake3) = crate::release_tree_copy::hash_directory_tree(provider_dir)
        .map_err(|error| admission_error(format!("hashing provider tree {}: {error}", provider_dir.display())))?;
    let provider_basename = provider_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| admission_error(format!("provider path has no UTF-8 basename: {}", provider_dir.display())))?
        .to_string();
    let input = AdmissionInput {
        provider_basename,
        metadata: metadata.clone(),
        files,
        output_digest_blake3: output_digest_blake3.clone(),
        expected_output_digest_blake3: expected_output_digest_blake3.to_string(),
        source_closure,
    };
    validate_admission_input(&input).map_err(|blockers| admission_error(blockers.join("; ")))?;
    let runtime_smoke_steps = execute_provider_runtime_smoke(provider_dir)?;
    let metadata_digest_blake3 = blake3::hash(&metadata_bytes).to_hex().to_string();
    Ok(admission_report(
        provider_dir,
        metadata_path,
        metadata_digest_blake3,
        output_digest_blake3,
        expected_output_digest_blake3,
        source_closure_path,
        &input,
        runtime_smoke_steps,
    ))
}

fn observe_source_closure(
    source_closure_path: &Path,
    expected_manifest_blake3: &str,
) -> Result<SourceClosureObservation, RunError> {
    let manifest = crate::source_bundle::read_source_bundle(source_closure_path).map_err(|error| {
        admission_error(format!("reading source closure {}: {error}", source_closure_path.display()))
    })?;
    let observation = source_closure_observation(&manifest, expected_manifest_blake3);
    assert_eq!(observation.record_count, manifest.records.len());
    debug_assert!(observation.materialized_record_count <= observation.record_count);
    Ok(observation)
}

fn source_closure_observation(
    manifest: &crate::source_bundle::SourceBundleManifest,
    expected_manifest_blake3: &str,
) -> SourceClosureObservation {
    let mut planned_records = Vec::new();
    let mut state_pinned_records = Vec::new();
    let mut legacy_records = Vec::new();
    for record in &manifest.records {
        if record.files.is_empty() {
            planned_records.push(record.identity.clone());
        }
        if record
            .metadata
            .get(SOURCE_CLOSURE_STORE_PATH_KEY)
            .is_some_and(|path| path.starts_with(STORE_PREFIX))
        {
            state_pinned_records.push(record.identity.clone());
        }
        let has_legacy_marker = record.metadata.values().any(|value| {
            let lowercase = value.to_ascii_lowercase();
            SOURCE_CLOSURE_LEGACY_MARKERS.iter().any(|marker| lowercase.contains(marker))
        });
        if has_legacy_marker {
            legacy_records.push(record.identity.clone());
        }
    }
    let materialized_record_count = manifest.records.len().saturating_sub(planned_records.len());
    assert!(materialized_record_count <= manifest.records.len());
    debug_assert!(planned_records.len() <= manifest.records.len());
    SourceClosureObservation {
        manifest_blake3: manifest.manifest_blake3.clone(),
        expected_manifest_blake3: expected_manifest_blake3.to_string(),
        record_count: manifest.records.len(),
        materialized_record_count,
        planned_records,
        state_pinned_records,
        legacy_records,
    }
}

fn validate_expected_digest(digest: &str) -> Result<(), RunError> {
    let valid_length = digest.len() == BLAKE3_HEX_LENGTH;
    let lowercase_hex = digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid_length || !lowercase_hex {
        return Err(admission_error(format!(
            "expected output BLAKE3 must be {BLAKE3_HEX_LENGTH} lowercase hexadecimal characters"
        )));
    }
    assert!(BLAKE3_HEX_LENGTH > 1);
    debug_assert!(valid_length);
    Ok(())
}

fn observe_provider_tree(provider_dir: &Path) -> Result<BTreeMap<String, FileObservation>, RunError> {
    if !provider_dir.is_dir() {
        return Err(admission_error(format!("provider path is not a directory: {}", provider_dir.display())));
    }
    let mut observations = BTreeMap::new();
    let mut pending = vec![provider_dir.to_path_buf()];
    while let Some(directory) = pending.pop() {
        observe_directory(provider_dir, &directory, &mut pending, &mut observations)?;
    }
    assert!(observations.len() <= TREE_ENTRY_COUNT_MAX);
    debug_assert!(pending.is_empty());
    Ok(observations)
}

fn observe_directory(
    provider_dir: &Path,
    directory: &Path,
    pending: &mut Vec<PathBuf>,
    observations: &mut BTreeMap<String, FileObservation>,
) -> Result<(), RunError> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| admission_error(format!("reading provider directory {}: {error}", directory.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| admission_error(format!("reading provider entry under {}: {error}", directory.display())))?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        ensure_observation_capacity(observations.len())?;
        observe_child(provider_dir, &child.path(), pending, observations)?;
    }
    assert!(directory.starts_with(provider_dir));
    debug_assert!(observations.len() <= TREE_ENTRY_COUNT_MAX);
    Ok(())
}

fn observe_child(
    provider_dir: &Path,
    child: &Path,
    pending: &mut Vec<PathBuf>,
    observations: &mut BTreeMap<String, FileObservation>,
) -> Result<(), RunError> {
    let relative_path = normalized_relative_path(provider_dir, child)?;
    let symlink_metadata = fs::symlink_metadata(child)
        .map_err(|error| admission_error(format!("reading provider metadata {}: {error}", child.display())))?;
    if symlink_metadata.is_dir() {
        pending.push(child.to_path_buf());
        return Ok(());
    }
    let symlink_target = if symlink_metadata.file_type().is_symlink() {
        Some(
            fs::read_link(child)
                .map_err(|error| admission_error(format!("reading provider symlink {}: {error}", child.display())))?
                .to_string_lossy()
                .replace('\\', "/"),
        )
    } else {
        None
    };
    let followed = fs::metadata(child)
        .map_err(|error| admission_error(format!("following provider entry {}: {error}", child.display())))?;
    let text = read_bounded_text(child, &followed)?;
    let observation = FileObservation {
        relative_path: relative_path.clone(),
        file_size_bytes: followed.len(),
        executable: metadata_is_executable(&followed),
        symlink_target,
        text,
    };
    let previous = observations.insert(relative_path.clone(), observation);
    if previous.is_some() {
        return Err(admission_error(format!("duplicate provider path observation: {relative_path}")));
    }
    debug_assert!(observations.contains_key(&relative_path));
    Ok(())
}

fn ensure_observation_capacity(observed_count: usize) -> Result<(), RunError> {
    if observed_count >= TREE_ENTRY_COUNT_MAX {
        return Err(admission_error(format!("provider entry count exceeds bounded maximum {TREE_ENTRY_COUNT_MAX}")));
    }
    assert!(TREE_ENTRY_COUNT_MAX > 1);
    debug_assert!(observed_count < TREE_ENTRY_COUNT_MAX);
    Ok(())
}

fn normalized_relative_path(provider_dir: &Path, child: &Path) -> Result<String, RunError> {
    let relative = child.strip_prefix(provider_dir).map_err(|error| {
        admission_error(format!("provider entry {} escapes {}: {error}", child.display(), provider_dir.display()))
    })?;
    let text = relative
        .to_str()
        .ok_or_else(|| admission_error(format!("provider entry path is not UTF-8: {}", child.display())))?
        .replace('\\', "/");
    if text.is_empty() || text.starts_with('/') || text.split('/').any(|component| component == "..") {
        return Err(admission_error(format!("provider entry path is not normalized: {text}")));
    }
    assert!(!text.is_empty());
    debug_assert!(!text.starts_with('/'));
    Ok(text)
}

fn read_bounded_text(path: &Path, metadata: &fs::Metadata) -> Result<Option<String>, RunError> {
    if !metadata.is_file() || metadata.len() > TEXT_SCAN_BYTES_MAX {
        return Ok(None);
    }
    let bytes = fs::read(path)
        .map_err(|error| admission_error(format!("reading bounded provider file {}: {error}", path.display())))?;
    let text = String::from_utf8(bytes).ok();
    assert!(metadata.len() <= TEXT_SCAN_BYTES_MAX);
    debug_assert!(text.as_ref().is_none_or(|value| value.len() as u64 <= TEXT_SCAN_BYTES_MAX));
    Ok(text)
}

#[cfg(unix)]
fn metadata_is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK != 0
}

#[cfg(not(unix))]
fn metadata_is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

fn validate_admission_input(input: &AdmissionInput) -> Result<(), Vec<String>> {
    let mut blockers = Vec::new();
    validate_metadata_identity(input, &mut blockers);
    validate_source_outputs(&input.metadata.source_outputs, &mut blockers);
    validate_required_paths(input, &mut blockers);
    validate_runtime_claims(&input.metadata, &mut blockers);
    validate_closure_claims(&input.metadata.closure_admission, &mut blockers);
    validate_source_closure(&input.source_closure, &mut blockers);
    validate_symlinks(&input.files, &mut blockers);
    validate_forbidden_markers(&input.files, &mut blockers);
    validate_output_digest(input, &mut blockers);
    blockers.sort();
    blockers.dedup();
    blockers.truncate(DIAGNOSTIC_COUNT_MAX);
    if blockers.is_empty() {
        return Ok(());
    }
    debug_assert!(blockers.len() <= DIAGNOSTIC_COUNT_MAX);
    Err(blockers)
}

fn validate_metadata_identity(input: &AdmissionInput, blockers: &mut Vec<String>) {
    let metadata = &input.metadata;
    require_equal("schema", &metadata.schema, PROVIDER_SCHEMA, blockers);
    require_equal("provider_id", &metadata.provider_id, PROVIDER_ID, blockers);
    require_equal("name", &metadata.name, PROVIDER_NAME, blockers);
    require_equal("target", &metadata.target, PROVIDER_TARGET, blockers);
    require_equal("compiler_target", &metadata.compiler_target, COMPILER_TARGET, blockers);
    require_equal("dynamic_linker", &metadata.dynamic_linker, DYNAMIC_LINKER, blockers);
    require_equal("source_authority", &metadata.source_authority, SOURCE_AUTHORITY, blockers);
    require_equal(
        "output_identity.content_blake3",
        &metadata.output_identity.content_blake3,
        OUTPUT_DIGEST_SENTINEL,
        blockers,
    );
    let logical_basename =
        Path::new(&metadata.output_identity.logical_store_path).file_name().and_then(|name| name.to_str());
    if logical_basename != Some(input.provider_basename.as_str()) {
        blockers.push("output-identity-logical-path-mismatch".to_string());
    }
    assert!(!PROVIDER_SCHEMA.is_empty());
    debug_assert!(!input.provider_basename.is_empty());
}

fn require_equal(label: &str, actual: &str, expected: &str, blockers: &mut Vec<String>) {
    if actual != expected {
        blockers.push(format!("metadata-{label}-mismatch"));
    }
    assert!(!label.is_empty());
    debug_assert!(!expected.is_empty());
}

fn validate_source_outputs(outputs: &SourceOutputs, blockers: &mut Vec<String>) {
    for (label, value) in [
        ("gcc", &outputs.gcc),
        ("musl", &outputs.musl),
        ("binutils", &outputs.binutils),
    ] {
        if !value.starts_with(STORE_PREFIX) || Path::new(value).file_name().is_none() {
            blockers.push(format!("source-output-{label}-invalid"));
        }
        let lowercase = value.to_ascii_lowercase();
        for (marker, diagnostic) in FORBIDDEN_TREE_MARKERS {
            if lowercase.contains(&marker.to_ascii_lowercase()) {
                blockers.push(format!("source-output-{label}-{diagnostic}"));
            }
        }
    }
    assert_eq!(STORE_PREFIX.as_bytes().first(), Some(&b'/'));
    debug_assert!(!outputs.gcc.is_empty());
}

fn validate_required_paths(input: &AdmissionInput, blockers: &mut Vec<String>) {
    for tool in REQUIRED_TOOLS {
        let path = format!("bin/{PROVIDER_TARGET}-{tool}");
        require_nonempty_executable(&input.files, &path, blockers);
    }
    for internal in ["cc1", "cc1plus"] {
        let path = format!("libexec/gcc/{COMPILER_TARGET}/{COMPILER_VERSION}/{internal}");
        require_nonempty_executable(&input.files, &path, blockers);
    }
    for runtime in REQUIRED_RUNTIME_FILES {
        let path = format!("{PROVIDER_TARGET}/lib/{runtime}");
        require_nonempty_file(&input.files, &path, blockers);
    }
    let dynamic_linker_path = format!("{PROVIDER_TARGET}/lib/{DYNAMIC_LINKER}");
    require_nonempty_file(&input.files, &dynamic_linker_path, blockers);
    require_nonempty_file(&input.files, PROVIDER_METADATA_RELATIVE_PATH, blockers);
    assert!(REQUIRED_TOOLS.len() > 1);
    debug_assert!(REQUIRED_RUNTIME_FILES.len() > 1);
}

fn require_nonempty_executable(files: &BTreeMap<String, FileObservation>, path: &str, blockers: &mut Vec<String>) {
    match files.get(path) {
        Some(file) if file.file_size_bytes > 0 && file.executable => {}
        Some(file) if file.file_size_bytes == 0 => blockers.push(format!("required-path-empty:{path}")),
        Some(_) => blockers.push(format!("required-path-not-executable:{path}")),
        None => blockers.push(format!("required-path-missing:{path}")),
    }
    assert!(!path.is_empty());
    debug_assert!(!path.starts_with('/'));
}

fn require_nonempty_file(files: &BTreeMap<String, FileObservation>, path: &str, blockers: &mut Vec<String>) {
    match files.get(path) {
        Some(file) if file.file_size_bytes > 0 => {}
        Some(_) => blockers.push(format!("required-path-empty:{path}")),
        None => blockers.push(format!("required-path-missing:{path}")),
    }
    assert!(!path.is_empty());
    debug_assert!(!path.starts_with('/'));
}

fn validate_runtime_claims(metadata: &FullSourceProviderMetadata, blockers: &mut Vec<String>) {
    require_equal("runtime_admission.status", &metadata.runtime_admission.status, RUNTIME_ADMISSION_STATUS, blockers);
    for surface in REQUIRED_RUNTIME_SURFACES {
        if !metadata.runtime_admission.surfaces.iter().any(|actual| actual == surface) {
            blockers.push(format!("runtime-surface-missing:{surface}"));
        }
    }
    for non_claim in REQUIRED_NON_CLAIMS {
        if !metadata.non_claims.iter().any(|actual| actual == non_claim) {
            blockers.push(format!("non-claim-missing:{non_claim}"));
        }
    }
    assert!(REQUIRED_RUNTIME_SURFACES.len() > 1);
    debug_assert!(REQUIRED_NON_CLAIMS.len() > 1);
}

fn validate_closure_claims(closure: &ClosureAdmission, blockers: &mut Vec<String>) {
    require_equal("closure_admission.status", &closure.status, CLOSURE_ADMISSION_STATUS, blockers);
    if closure.state_pinned_inputs {
        blockers.push("closure-state-pinned-inputs".to_string());
    }
    for legacy_member in &closure.legacy_members {
        blockers.push(format!("closure-legacy-member:{legacy_member}"));
    }
    for generated_source in &closure.release_generated_sources {
        blockers.push(format!("closure-release-generated-source:{generated_source}"));
    }
    assert!(!CLOSURE_ADMISSION_STATUS.is_empty());
    debug_assert!(blockers.len() <= TREE_ENTRY_COUNT_MAX);
}

fn validate_source_closure(source_closure: &SourceClosureObservation, blockers: &mut Vec<String>) {
    if source_closure.manifest_blake3 != source_closure.expected_manifest_blake3 {
        blockers.push(format!(
            "source-closure-blake3-mismatch:expected={} observed={}",
            source_closure.expected_manifest_blake3, source_closure.manifest_blake3
        ));
    }
    if source_closure.record_count < SOURCE_CLOSURE_RECORD_COUNT_MIN {
        blockers.push("source-closure-empty".to_string());
    }
    for record in &source_closure.planned_records {
        blockers.push(format!("source-closure-record-not-materialized:{record}"));
    }
    for record in &source_closure.state_pinned_records {
        blockers.push(format!("source-closure-state-pinned-record:{record}"));
    }
    for record in &source_closure.legacy_records {
        blockers.push(format!("source-closure-legacy-record:{record}"));
    }
    if source_closure.materialized_record_count != source_closure.record_count {
        blockers.push(format!(
            "source-closure-materialized-count-mismatch:expected={} observed={}",
            source_closure.record_count, source_closure.materialized_record_count
        ));
    }
    assert!(SOURCE_CLOSURE_RECORD_COUNT_MIN > 0);
    debug_assert!(source_closure.materialized_record_count <= source_closure.record_count);
}

fn validate_symlinks(files: &BTreeMap<String, FileObservation>, blockers: &mut Vec<String>) {
    for file in files.values() {
        let Some(target) = file.symlink_target.as_deref() else {
            continue;
        };
        let absolute = target.starts_with('/');
        let parent_escape = target.split('/').any(|component| component == "..");
        if absolute || parent_escape {
            blockers.push(format!("unsafe-symlink:{}->{target}", file.relative_path));
        }
    }
    assert!(files.len() <= TREE_ENTRY_COUNT_MAX);
    debug_assert!(blockers.len() <= TREE_ENTRY_COUNT_MAX);
}

fn validate_forbidden_markers(files: &BTreeMap<String, FileObservation>, blockers: &mut Vec<String>) {
    for file in files.values() {
        let path_lowercase = file.relative_path.to_ascii_lowercase();
        validate_marker_text(&file.relative_path, &path_lowercase, FORBIDDEN_TREE_MARKERS, blockers);
        if let Some(text) = file.text.as_deref() {
            let text_lowercase = text.to_ascii_lowercase();
            validate_marker_text(&file.relative_path, &text_lowercase, FORBIDDEN_TREE_MARKERS, blockers);
            if file.relative_path.starts_with("bin/") && text.starts_with("#!") {
                validate_marker_text(&file.relative_path, &text_lowercase, FORBIDDEN_SCRIPT_MARKERS, blockers);
            }
        }
    }
    assert!(FORBIDDEN_TREE_MARKERS.len() > 1);
    debug_assert!(files.len() <= TREE_ENTRY_COUNT_MAX);
}

fn validate_marker_text(path: &str, lowercase_text: &str, markers: &[(&str, &str)], blockers: &mut Vec<String>) {
    for (marker, diagnostic) in markers {
        if lowercase_text.contains(&marker.to_ascii_lowercase()) {
            blockers.push(format!("forbidden-{diagnostic}:{path}"));
        }
    }
    assert!(!path.is_empty());
    debug_assert_eq!(lowercase_text, lowercase_text.to_ascii_lowercase());
}

fn validate_output_digest(input: &AdmissionInput, blockers: &mut Vec<String>) {
    if input.output_digest_blake3 != input.expected_output_digest_blake3 {
        blockers.push(format!(
            "output-blake3-mismatch:expected={} observed={}",
            input.expected_output_digest_blake3, input.output_digest_blake3
        ));
    }
    assert_eq!(input.output_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert_eq!(input.expected_output_digest_blake3.len(), BLAKE3_HEX_LENGTH);
}

fn execute_provider_runtime_smoke(provider_dir: &Path) -> Result<Vec<String>, RunError> {
    let scratch = tempfile::Builder::new()
        .prefix("mantle-full-source-provider-admission-")
        .tempdir()
        .map_err(|error| admission_error(format!("creating runtime smoke directory: {error}")))?;
    write_runtime_smoke_sources(scratch.path())?;
    let steps = provider_runtime_smoke_plan(provider_dir, scratch.path());
    if steps.len() != RUNTIME_SMOKE_STEP_COUNT {
        return Err(admission_error(format!(
            "runtime smoke plan expected {RUNTIME_SMOKE_STEP_COUNT} steps, observed {}",
            steps.len()
        )));
    }
    let mut completed = Vec::with_capacity(steps.len());
    for step in &steps {
        execute_runtime_smoke_step(step)?;
        completed.push(step.label.to_string());
    }
    assert_eq!(completed.len(), RUNTIME_SMOKE_STEP_COUNT);
    debug_assert!(completed.iter().all(|label| !label.is_empty()));
    Ok(completed)
}

fn write_runtime_smoke_sources(scratch: &Path) -> Result<(), RunError> {
    let sources = [
        ("c-smoke.c", "#include <stdio.h>\nint main(void) { return puts(\"mantle-provider-c\") < 0; }\n"),
        (
            "cxx-smoke.cc",
            "#include <stdexcept>\nstruct Base { virtual ~Base() {} };\nstruct Derived : Base {};\nint main() { Base *value = new Derived; bool cast_ok = dynamic_cast<Derived *>(value) != 0; delete value; try { throw std::runtime_error(\"mantle\"); } catch (std::runtime_error const &) { return cast_ok ? 0 : 1; } }\n",
        ),
        ("start.s", ".global _start\n_start:\n  mov $60, %rax\n  xor %rdi, %rdi\n  syscall\n"),
        ("malformed.c", "int main( { return 0; }\n"),
        ("malformed.cc", "template <typename T int broken;\n"),
        ("malformed.s", ".global _start\n_start:\n  not-an-opcode %rax\n"),
        ("undefined.s", ".global _start\n_start:\n  call missing_symbol\n"),
    ];
    for (name, content) in sources {
        fs::write(scratch.join(name), content)
            .map_err(|error| admission_error(format!("writing runtime smoke source {name}: {error}")))?;
    }
    assert!(sources.len() > 1);
    debug_assert!(sources.iter().all(|(name, content)| !name.is_empty() && !content.is_empty()));
    Ok(())
}

fn provider_runtime_smoke_plan(provider_dir: &Path, scratch: &Path) -> Vec<RuntimeSmokeStep> {
    let mut steps = compiler_runtime_smoke_steps(provider_dir, scratch);
    steps.extend(binary_tool_runtime_smoke_steps(provider_dir, scratch));
    steps.extend(rejection_runtime_smoke_steps(provider_dir, scratch));
    assert_eq!(steps.len(), RUNTIME_SMOKE_STEP_COUNT);
    debug_assert!(steps.iter().all(|step| !step.program.as_os_str().is_empty()));
    steps
}

fn compiler_runtime_smoke_steps(provider_dir: &Path, scratch: &Path) -> Vec<RuntimeSmokeStep> {
    let target_bin = |name: &str| provider_dir.join("bin").join(format!("{PROVIDER_TARGET}-{name}"));
    let gcc = target_bin("gcc");
    let gxx = target_bin("g++");
    let c_source = scratch.join("c-smoke.c");
    let cxx_source = scratch.join("cxx-smoke.cc");
    let c_static = scratch.join("c-static");
    let c_dynamic = scratch.join("c-dynamic");
    let cxx_static = scratch.join("cxx-static");
    let cxx_dynamic = scratch.join("cxx-dynamic");
    let steps = vec![
        success_step("c-static-compile", &gcc, vec![
            "-static".into(),
            c_source.clone().into(),
            "-o".into(),
            c_static.clone().into(),
        ]),
        success_step("c-static-run", &c_static, Vec::new()),
        success_step("c-dynamic-compile", &gcc, vec![c_source.into(), "-o".into(), c_dynamic.clone().into()]),
        success_step("c-dynamic-run", &c_dynamic, Vec::new()),
        success_step("cxx-static-compile", &gxx, vec![
            "-static".into(),
            cxx_source.clone().into(),
            "-o".into(),
            cxx_static.clone().into(),
        ]),
        success_step("cxx-static-run", &cxx_static, Vec::new()),
        success_step("cxx-dynamic-compile", &gxx, vec![cxx_source.into(), "-o".into(), cxx_dynamic.clone().into()]),
        success_step("cxx-dynamic-run", &cxx_dynamic, Vec::new()),
    ];
    assert_eq!(steps.len(), COMPILER_RUNTIME_SMOKE_STEP_COUNT);
    debug_assert!(steps.iter().all(|step| matches!(step.expectation, RuntimeSmokeExpectation::Success)));
    steps
}

fn binary_tool_runtime_smoke_steps(provider_dir: &Path, scratch: &Path) -> Vec<RuntimeSmokeStep> {
    let target_bin = |name: &str| provider_dir.join("bin").join(format!("{PROVIDER_TARGET}-{name}"));
    let object = scratch.join("start.o");
    let executable = scratch.join("assembly-smoke");
    let archive = scratch.join("libsmoke.a");
    let copied = scratch.join("start-copy.o");
    let stripped = scratch.join("start-stripped.o");
    let start_source = scratch.join("start.s");
    let steps = vec![
        success_step("assembly", &target_bin("as"), vec![start_source.into(), "-o".into(), object.clone().into()]),
        success_step("link", &target_bin("ld"), vec!["-o".into(), executable.clone().into(), object.clone().into()]),
        success_step("assembly-run", &executable, Vec::new()),
        success_step("archive", &target_bin("ar"), vec!["cr".into(), archive.clone().into(), object.clone().into()]),
        success_step("archive-index", &target_bin("ranlib"), vec![archive.into()]),
        success_step("readelf", &target_bin("readelf"), vec!["-h".into(), object.clone().into()]),
        success_step("nm", &target_bin("nm"), vec![object.clone().into()]),
        success_step("objdump", &target_bin("objdump"), vec!["-f".into(), object.clone().into()]),
        success_step("size", &target_bin("size"), vec![object.clone().into()]),
        success_step("strings", &target_bin("strings"), vec![object.clone().into()]),
        success_step("objcopy", &target_bin("objcopy"), vec![object.clone().into(), copied.into()]),
        success_step("strip", &target_bin("strip"), vec!["-o".into(), stripped.into(), object.into()]),
    ];
    assert_eq!(steps.len(), BINARY_TOOL_RUNTIME_SMOKE_STEP_COUNT);
    debug_assert!(steps.iter().all(|step| matches!(step.expectation, RuntimeSmokeExpectation::Success)));
    steps
}

fn rejection_runtime_smoke_steps(provider_dir: &Path, scratch: &Path) -> Vec<RuntimeSmokeStep> {
    let target_bin = |name: &str| provider_dir.join("bin").join(format!("{PROVIDER_TARGET}-{name}"));
    let malformed_c_output = scratch.join("malformed-c.o");
    let malformed_cxx_output = scratch.join("malformed-cxx.o");
    let malformed_asm_output = scratch.join("malformed-assembly.o");
    let undefined_object = scratch.join("undefined.o");
    let undefined_output = scratch.join("undefined-link");
    let steps = vec![
        rejection_step(
            "malformed-c",
            &target_bin("gcc"),
            vec![
                "-c".into(),
                scratch.join("malformed.c").into(),
                "-o".into(),
                malformed_c_output.clone().into(),
            ],
            malformed_c_output,
        ),
        rejection_step(
            "malformed-cxx",
            &target_bin("g++"),
            vec![
                "-c".into(),
                scratch.join("malformed.cc").into(),
                "-o".into(),
                malformed_cxx_output.clone().into(),
            ],
            malformed_cxx_output,
        ),
        rejection_step(
            "malformed-assembly",
            &target_bin("as"),
            vec![
                scratch.join("malformed.s").into(),
                "-o".into(),
                malformed_asm_output.clone().into(),
            ],
            malformed_asm_output,
        ),
        success_step("undefined-assembly", &target_bin("as"), vec![
            scratch.join("undefined.s").into(),
            "-o".into(),
            undefined_object.clone().into(),
        ]),
        rejection_step(
            "undefined-link",
            &target_bin("ld"),
            vec!["-o".into(), undefined_output.clone().into(), undefined_object.into()],
            undefined_output,
        ),
    ];
    assert_eq!(steps.len(), REJECTION_RUNTIME_SMOKE_STEP_COUNT);
    debug_assert!(steps.iter().any(|step| matches!(step.expectation, RuntimeSmokeExpectation::Rejection { .. })));
    steps
}

fn success_step(label: &'static str, program: &Path, arguments: Vec<OsString>) -> RuntimeSmokeStep {
    assert!(!label.is_empty());
    assert!(!program.as_os_str().is_empty());
    RuntimeSmokeStep {
        label,
        program: program.to_path_buf(),
        arguments,
        expectation: RuntimeSmokeExpectation::Success,
    }
}

fn rejection_step(
    label: &'static str,
    program: &Path,
    arguments: Vec<OsString>,
    absent_output: PathBuf,
) -> RuntimeSmokeStep {
    assert!(!label.is_empty());
    assert!(!absent_output.as_os_str().is_empty());
    RuntimeSmokeStep {
        label,
        program: program.to_path_buf(),
        arguments,
        expectation: RuntimeSmokeExpectation::Rejection { absent_output },
    }
}

fn execute_runtime_smoke_step(step: &RuntimeSmokeStep) -> Result<(), RunError> {
    if let RuntimeSmokeExpectation::Rejection { absent_output } = &step.expectation {
        let _ = fs::remove_file(absent_output);
    }
    let mut child = Command::new(&step.program)
        .args(&step.arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| admission_error(format!("launching runtime smoke {}: {error}", step.label)))?;
    let deadline = Instant::now() + Duration::from_secs(RUNTIME_SMOKE_TIMEOUT_SECS);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| admission_error(format!("waiting for runtime smoke {}: {error}", step.label)))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(admission_error(format!(
                "runtime smoke {} exceeded {} seconds",
                step.label, RUNTIME_SMOKE_TIMEOUT_SECS
            )));
        }
        std::thread::sleep(Duration::from_millis(RUNTIME_SMOKE_POLL_INTERVAL_MS));
    };
    validate_runtime_smoke_status(step, status.success())
}

fn validate_runtime_smoke_status(step: &RuntimeSmokeStep, success: bool) -> Result<(), RunError> {
    match &step.expectation {
        RuntimeSmokeExpectation::Success if success => Ok(()),
        RuntimeSmokeExpectation::Success => Err(admission_error(format!("runtime smoke {} failed", step.label))),
        RuntimeSmokeExpectation::Rejection { absent_output } if !success && !absent_output.exists() => Ok(()),
        RuntimeSmokeExpectation::Rejection { absent_output } => Err(admission_error(format!(
            "runtime rejection smoke {} did not reject cleanly; output_exists={}",
            step.label,
            absent_output.exists()
        ))),
    }
}

fn admission_report(
    provider_dir: &Path,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    output_digest_blake3: String,
    expected_output_digest_blake3: &str,
    source_closure_path: &Path,
    input: &AdmissionInput,
    runtime_smoke_steps: Vec<String>,
) -> FullSourceProviderAdmissionReport {
    let source_outputs = BTreeMap::from([
        ("binutils".to_string(), input.metadata.source_outputs.binutils.clone()),
        ("gcc".to_string(), input.metadata.source_outputs.gcc.clone()),
        ("musl".to_string(), input.metadata.source_outputs.musl.clone()),
    ]);
    assert_eq!(output_digest_blake3, expected_output_digest_blake3);
    assert_eq!(input.source_closure.manifest_blake3, input.source_closure.expected_manifest_blake3);
    assert_eq!(runtime_smoke_steps.len(), RUNTIME_SMOKE_STEP_COUNT);
    debug_assert_eq!(source_outputs.len(), SOURCE_OUTPUT_COUNT);
    FullSourceProviderAdmissionReport {
        schema: "mantle-full-source-provider-admission-v2",
        status: "admitted",
        provider_id: input.metadata.provider_id.clone(),
        provider_path: provider_dir.to_path_buf(),
        metadata_path,
        metadata_digest_blake3,
        output_digest_blake3,
        expected_output_digest_blake3: expected_output_digest_blake3.to_string(),
        source_closure_path: source_closure_path.to_path_buf(),
        source_closure_manifest_blake3: input.source_closure.manifest_blake3.clone(),
        expected_source_closure_manifest_blake3: input.source_closure.expected_manifest_blake3.clone(),
        source_closure_record_count: input.source_closure.record_count,
        observed_entry_count: input.files.len(),
        required_tool_count: REQUIRED_TOOLS.len(),
        required_runtime_count: REQUIRED_RUNTIME_FILES.len(),
        runtime_surfaces: input.metadata.runtime_admission.surfaces.clone(),
        runtime_smoke_steps,
        source_outputs,
        non_claims: input.metadata.non_claims.clone(),
    }
}

fn admission_error(message: String) -> RunError {
    RunError::Build(format!("full-source provider admission failed closed: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const EXECUTABLE_SIZE_BYTES: u64 = 8;
    const RUNTIME_SIZE_BYTES: u64 = 16;
    const SOURCE_CLOSURE_RECORD_COUNT: usize = 1;
    const SOURCE_BUNDLE_VERSION: u32 = 1;
    const SOURCE_FILE_SIZE_BYTES: u64 = 1;

    fn valid_metadata(provider_basename: &str) -> FullSourceProviderMetadata {
        FullSourceProviderMetadata {
            schema: PROVIDER_SCHEMA.to_string(),
            provider_id: PROVIDER_ID.to_string(),
            name: PROVIDER_NAME.to_string(),
            target: PROVIDER_TARGET.to_string(),
            compiler_target: COMPILER_TARGET.to_string(),
            dynamic_linker: DYNAMIC_LINKER.to_string(),
            source_authority: SOURCE_AUTHORITY.to_string(),
            source_outputs: SourceOutputs {
                gcc: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-gcc-final".to_string(),
                musl: "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-musl-final".to_string(),
                binutils: "/mantle/store/cccccccccccccccccccccccccccccccc-binutils-final".to_string(),
            },
            output_identity: OutputIdentity {
                logical_store_path: format!("/mantle/store/{provider_basename}"),
                content_blake3: OUTPUT_DIGEST_SENTINEL.to_string(),
            },
            runtime_admission: RuntimeAdmission {
                status: RUNTIME_ADMISSION_STATUS.to_string(),
                surfaces: REQUIRED_RUNTIME_SURFACES.iter().map(|value| (*value).to_string()).collect(),
            },
            closure_admission: ClosureAdmission {
                status: CLOSURE_ADMISSION_STATUS.to_string(),
                state_pinned_inputs: false,
                legacy_members: Vec::new(),
                release_generated_sources: Vec::new(),
            },
            non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
        }
    }

    fn insert_file(files: &mut BTreeMap<String, FileObservation>, path: String, executable: bool, size: u64) {
        let observation = FileObservation {
            relative_path: path.clone(),
            file_size_bytes: size,
            executable,
            symlink_target: None,
            text: None,
        };
        assert!(files.insert(path.clone(), observation).is_none());
        assert!(files.contains_key(&path));
    }

    fn valid_files() -> BTreeMap<String, FileObservation> {
        let mut files = BTreeMap::new();
        for tool in REQUIRED_TOOLS {
            insert_file(&mut files, format!("bin/{PROVIDER_TARGET}-{tool}"), true, EXECUTABLE_SIZE_BYTES);
        }
        for internal in ["cc1", "cc1plus"] {
            insert_file(
                &mut files,
                format!("libexec/gcc/{COMPILER_TARGET}/{COMPILER_VERSION}/{internal}"),
                true,
                EXECUTABLE_SIZE_BYTES,
            );
        }
        for runtime in REQUIRED_RUNTIME_FILES {
            insert_file(&mut files, format!("{PROVIDER_TARGET}/lib/{runtime}"), false, RUNTIME_SIZE_BYTES);
        }
        insert_file(&mut files, format!("{PROVIDER_TARGET}/lib/{DYNAMIC_LINKER}"), true, RUNTIME_SIZE_BYTES);
        insert_file(&mut files, PROVIDER_METADATA_RELATIVE_PATH.to_string(), false, RUNTIME_SIZE_BYTES);
        files
    }

    fn valid_input() -> AdmissionInput {
        let provider_basename = "dddddddddddddddddddddddddddddddd-full-source-seed-toolchain".to_string();
        AdmissionInput {
            metadata: valid_metadata(&provider_basename),
            provider_basename,
            files: valid_files(),
            output_digest_blake3: DIGEST_A.to_string(),
            expected_output_digest_blake3: DIGEST_A.to_string(),
            source_closure: SourceClosureObservation {
                manifest_blake3: DIGEST_B.to_string(),
                expected_manifest_blake3: DIGEST_B.to_string(),
                record_count: SOURCE_CLOSURE_RECORD_COUNT,
                materialized_record_count: SOURCE_CLOSURE_RECORD_COUNT,
                planned_records: Vec::new(),
                state_pinned_records: Vec::new(),
                legacy_records: Vec::new(),
            },
        }
    }

    fn blocker_text(input: &AdmissionInput) -> String {
        validate_admission_input(input).unwrap_err().join("\n")
    }

    fn source_record(
        identity: &str,
        files: Vec<crate::source_bundle::SourceFileEntry>,
    ) -> crate::source_bundle::SourceRecord {
        crate::source_bundle::SourceRecord {
            kind: crate::source_bundle::SourceRecordKind::FixedUrl,
            identity: identity.to_string(),
            store_prefix: None,
            adapter: None,
            metadata: BTreeMap::new(),
            payload_bytes: SOURCE_FILE_SIZE_BYTES,
            content_blake3: DIGEST_A.to_string(),
            files,
        }
    }

    fn source_file() -> crate::source_bundle::SourceFileEntry {
        crate::source_bundle::SourceFileEntry {
            path: "source.c".to_string(),
            file_type: crate::source_bundle::SourceFileType::Regular,
            executable: false,
            size: SOURCE_FILE_SIZE_BYTES,
            content_hex: Some("00".to_string()),
            symlink_target: None,
            chunk_index: None,
            chunk_count: None,
            blake3: DIGEST_A.to_string(),
        }
    }

    #[test]
    fn admission_core_accepts_complete_provider_observation() {
        let input = valid_input();
        assert_eq!(validate_admission_input(&input), Ok(()));
        assert_eq!(input.output_digest_blake3, input.expected_output_digest_blake3);
        assert_eq!(input.source_closure.manifest_blake3, input.source_closure.expected_manifest_blake3);
    }

    #[test]
    fn source_closure_observation_detects_planned_state_pinned_and_legacy_records() {
        let materialized = source_record("materialized", vec![source_file()]);
        let planned = source_record("planned", Vec::new());
        let mut state_pinned = source_record("state-pinned", vec![source_file()]);
        state_pinned.metadata.insert(
            SOURCE_CLOSURE_STORE_PATH_KEY.to_string(),
            "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-state-output".to_string(),
        );
        let mut legacy = source_record("legacy", vec![source_file()]);
        legacy.metadata.insert("url".to_string(), "https://musl.cc/legacy-provider.tgz".to_string());
        let manifest = crate::source_bundle::SourceBundleManifest {
            format: "mantle-source-bundle-v1".to_string(),
            version: SOURCE_BUNDLE_VERSION,
            store_prefix: "/mantle/store".to_string(),
            roots: vec![
                "legacy".to_string(),
                "materialized".to_string(),
                "planned".to_string(),
                "state-pinned".to_string(),
            ],
            records: vec![legacy, materialized, planned, state_pinned],
            manifest_blake3: DIGEST_A.to_string(),
            non_claim: "fixture".to_string(),
        };

        let observation = source_closure_observation(&manifest, DIGEST_A);
        assert_eq!(observation.record_count, manifest.records.len());
        assert_eq!(observation.materialized_record_count, manifest.records.len().saturating_sub(1));
        assert_eq!(observation.planned_records, vec!["planned".to_string()]);
        assert_eq!(observation.state_pinned_records, vec!["state-pinned".to_string()]);
        assert_eq!(observation.legacy_records, vec!["legacy".to_string()]);
    }

    #[test]
    fn admission_core_rejects_unbound_or_incomplete_source_closure() {
        let mut input = valid_input();
        input.source_closure.manifest_blake3 = DIGEST_A.to_string();
        input.source_closure.planned_records = vec!["planned-source".to_string()];
        input.source_closure.materialized_record_count = 0;
        input.source_closure.state_pinned_records = vec!["state-pinned-source".to_string()];
        input.source_closure.legacy_records = vec!["legacy-provider-source".to_string()];

        let blockers = blocker_text(&input);
        assert!(blockers.contains("source-closure-blake3-mismatch"));
        assert!(blockers.contains("source-closure-record-not-materialized:planned-source"));
        assert!(blockers.contains("source-closure-state-pinned-record:state-pinned-source"));
        assert!(blockers.contains("source-closure-legacy-record:legacy-provider-source"));
        assert!(blockers.contains("source-closure-materialized-count-mismatch"));
    }

    #[test]
    fn admission_core_rejects_missing_tool_and_compiler_internal() {
        let mut input = valid_input();
        input.files.remove(&format!("bin/{PROVIDER_TARGET}-gcc"));
        input.files.remove(&format!("libexec/gcc/{COMPILER_TARGET}/{COMPILER_VERSION}/cc1plus"));
        let blockers = blocker_text(&input);
        assert!(blockers.contains("required-path-missing:bin/x86_64-linux-musl-gcc"));
        assert!(blockers.contains("required-path-missing:libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1plus"));
    }

    #[test]
    fn admission_core_rejects_missing_runtime_and_digest_mismatch() {
        let mut input = valid_input();
        input.files.remove(&format!("{PROVIDER_TARGET}/lib/libgcc_s.so.1"));
        input.expected_output_digest_blake3 = DIGEST_B.to_string();
        let blockers = blocker_text(&input);
        assert!(blockers.contains("required-path-missing:x86_64-linux-musl/lib/libgcc_s.so.1"));
        assert!(blockers.contains("output-blake3-mismatch"));
    }

    #[test]
    fn admission_core_rejects_legacy_tinycc_stub_and_host_fallback_markers() {
        for (text, expected) in [
            ("seed-legacy.ncl", "legacy-seed-selection"),
            ("exec /provider/bin/tinycc", "tinycc-delegation"),
            ("generated-stub", "generated-stub"),
            ("#!/bin/sh\nexec /usr/bin/cc", "host-tool-fallback"),
        ] {
            let mut input = valid_input();
            let path = format!("bin/{PROVIDER_TARGET}-gcc");
            input.files.get_mut(&path).unwrap().text = Some(text.to_string());
            assert!(blocker_text(&input).contains(expected), "marker {text} must be rejected");
        }
    }

    #[test]
    fn admission_core_rejects_state_pinned_and_release_generated_closure() {
        let mut input = valid_input();
        input.metadata.closure_admission.status = "blocked-state-pinned-diagnostic".to_string();
        input.metadata.closure_admission.state_pinned_inputs = true;
        input.metadata.closure_admission.release_generated_sources = vec!["gcc-10.5.0".to_string()];
        let blockers = blocker_text(&input);
        assert!(blockers.contains("metadata-closure_admission.status-mismatch"));
        assert!(blockers.contains("closure-state-pinned-inputs"));
        assert!(blockers.contains("closure-release-generated-source:gcc-10.5.0"));
    }

    #[test]
    fn admission_core_rejects_malformed_metadata_and_unsafe_symlink() {
        let mut input = valid_input();
        input.metadata.schema = "wrong-schema".to_string();
        input.metadata.runtime_admission.surfaces.pop();
        input.metadata.non_claims.pop();
        let path = format!("bin/{PROVIDER_TARGET}-c++");
        input.files.get_mut(&path).unwrap().symlink_target = Some("../../host-g++".to_string());
        let blockers = blocker_text(&input);
        assert!(blockers.contains("metadata-schema-mismatch"));
        assert!(blockers.contains("runtime-surface-missing"));
        assert!(blockers.contains("non-claim-missing"));
        assert!(blockers.contains("unsafe-symlink"));
    }

    #[test]
    fn runtime_smoke_plan_covers_positive_and_rejection_surfaces() {
        let provider = Path::new("/mantle/store/provider");
        let scratch = Path::new("/tmp/mantle-provider-smoke-test");
        let steps = provider_runtime_smoke_plan(provider, scratch);

        assert_eq!(steps.len(), RUNTIME_SMOKE_STEP_COUNT);
        assert!(steps.iter().any(|step| step.label == "cxx-dynamic-run"));
        assert!(steps.iter().any(|step| step.label == "undefined-link"));
        assert!(steps.iter().any(|step| matches!(step.expectation, RuntimeSmokeExpectation::Rejection { .. })));
    }

    #[test]
    fn runtime_smoke_status_rejects_leftover_negative_output() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("rejected-output");
        fs::write(&output, b"unexpected").unwrap();
        let step = rejection_step("negative", Path::new("/bin/false"), Vec::new(), output.clone());

        let error = validate_runtime_smoke_status(&step, false).unwrap_err();
        assert!(error.to_string().contains("did not reject cleanly"));
        fs::remove_file(&output).unwrap();
        assert!(validate_runtime_smoke_status(&step, false).is_ok());
    }

    #[test]
    fn expected_digest_validation_rejects_non_blake3_text() {
        let error = validate_expected_digest("not-a-digest").unwrap_err();
        assert!(error.to_string().contains("lowercase hexadecimal"));
        assert!(!error.to_string().contains(DIGEST_A));
    }
}
