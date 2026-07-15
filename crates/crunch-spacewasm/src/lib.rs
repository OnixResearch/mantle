use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_spacewasm_core::Blake3Digest;
use crunch_spacewasm_core::BundleManifest;
use crunch_spacewasm_core::BundleManifestInput;
use crunch_spacewasm_core::BundleMember;
use crunch_spacewasm_core::BundleRole;
use crunch_spacewasm_core::CheckEvaluation;
use crunch_spacewasm_core::Diagnostic;
use crunch_spacewasm_core::ObservedCheck;
use crunch_spacewasm_core::REFERENCE_CLAIM_CLASS;
use crunch_spacewasm_core::ReferenceProfile;
use crunch_spacewasm_core::ReportBuildInput;
use crunch_spacewasm_core::SourceAdmission;
use crunch_spacewasm_core::SourceFacts;
use crunch_spacewasm_core::SupportComparison;
use crunch_spacewasm_core::SupportEntry;
use crunch_spacewasm_core::admit_source;
use crunch_spacewasm_core::build_bundle_manifest;
use crunch_spacewasm_core::build_materialization_report;
use crunch_spacewasm_core::cohort_identity;
use crunch_spacewasm_core::compare_support_matrix;
use crunch_spacewasm_core::evaluate_checks;
use crunch_spacewasm_core::plan_bundle_parent_edges;
use crunch_spacewasm_core::validate_materialization_report;
use crunch_spacewasm_core::validate_profile;
use crunch_spacewasm_core::verify_bundle_manifest;
use serde::Deserialize;
use serde::Serialize;

pub const MATERIALIZATION_REQUEST_SCHEMA: &str = "mantle-spacewasm-materialization-request-v1";
pub const MATERIALIZATION_SUMMARY_SCHEMA: &str = "mantle-spacewasm-materialization-summary-v1";
pub const VERIFY_SUMMARY_SCHEMA: &str = "mantle-spacewasm-bundle-verify-summary-v1";

const HARD_MAX_REQUEST_BYTES: u64 = 4_194_304;
const HARD_MAX_PROFILE_BYTES: u64 = 1_048_576;
const HARD_MAX_REPORT_BYTES: u64 = 16_777_216;
const HARD_MAX_BUNDLE_FILES: u32 = 1_024;
const READ_BUFFER_CAPACITY_BYTES: usize = 65_536;
const MANIFEST_PATH: &str = "manifest.json";
const REPORT_PATH: &str = "reports/materialization.json";
const NON_CLAIMS_PATH: &str = "non-claims.json";
const PROFILE_ROOT_PATH: &str = "profile/profile.ncl";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputMember {
    pub source_path: String,
    pub bundle_path: String,
    pub role: BundleRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationRequest {
    pub schema: String,
    pub profile_path: String,
    pub source_facts_path: String,
    pub support_matrix_path: String,
    pub checks_path: String,
    pub requested_claim_class: String,
    pub members: Vec<InputMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationSummary {
    pub schema: String,
    pub profile_identity_blake3: Blake3Digest,
    pub cohort_identity_blake3: Blake3Digest,
    pub report_identity_blake3: Blake3Digest,
    pub bundle_identity_blake3: Blake3Digest,
    pub member_count: u32,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifySummary {
    pub schema: String,
    pub valid: bool,
    pub bundle_identity_blake3: Option<Blake3Digest>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub enum ShellError {
    Io {
        operation: &'static str,
        path: String,
        message: String,
    },
    Invalid(String),
    Json(String),
    Core(Vec<Diagnostic>),
    Cleanup {
        path: String,
        materialization_error: Box<ShellError>,
        cleanup_message: String,
    },
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                message,
            } => write!(formatter, "{operation} `{path}`: {message}"),
            Self::Invalid(message) => formatter.write_str(message),
            Self::Json(message) => write!(formatter, "SpaceWasm JSON error: {message}"),
            Self::Core(diagnostics) => {
                let codes = diagnostics.iter().map(|item| item.code.as_str()).collect::<Vec<_>>().join(",");
                write!(formatter, "SpaceWasm core rejected materialization: {codes}")
            }
            Self::Cleanup {
                path,
                materialization_error,
                cleanup_message,
            } => write!(formatter, "{materialization_error}; removing incomplete bundle `{path}`: {cleanup_message}"),
        }
    }
}

impl std::error::Error for ShellError {}

pub fn read_materialization_request(path: &Path) -> Result<MaterializationRequest, ShellError> {
    read_json(path, HARD_MAX_REQUEST_BYTES)
}

pub fn materialize(request: MaterializationRequest, output: &Path) -> Result<MaterializationSummary, ShellError> {
    if output.exists() {
        return Err(ShellError::Invalid(format!("bundle output already exists: {}", output.display())));
    }
    fs::create_dir(output).map_err(|error| io_error("creating bundle root", output, error))?;
    match materialize_inner(request, output) {
        Ok(summary) => Ok(summary),
        Err(error) => Err(cleanup_failed_materialization(output, error)),
    }
}

fn materialize_inner(request: MaterializationRequest, output: &Path) -> Result<MaterializationSummary, ShellError> {
    validate_request_header(&request)?;
    let profile: ReferenceProfile = read_json(Path::new(&request.profile_path), HARD_MAX_PROFILE_BYTES)?;
    let validation = validate_profile(profile.clone());
    let profile_identity = validation.profile_identity_blake3.ok_or(ShellError::Core(validation.diagnostics))?;
    let cohort_identity = cohort_identity(profile.clone()).map_err(|error| ShellError::Invalid(error.to_string()))?;
    let source_facts: SourceFacts = read_json(Path::new(&request.source_facts_path), HARD_MAX_REPORT_BYTES)?;
    let matrix_entries: Vec<SupportEntry> = read_json(Path::new(&request.support_matrix_path), HARD_MAX_REPORT_BYTES)?;
    let observed_checks: Vec<ObservedCheck> = read_json(Path::new(&request.checks_path), HARD_MAX_REPORT_BYTES)?;
    let source_admission = admit_source(profile.clone(), source_facts);
    let matrix_evaluation = compare_support_matrix(profile.clone(), matrix_entries);
    let check_evaluation = evaluate_checks(profile.clone(), observed_checks);
    let mut members = copy_input_members(&profile, &request.members, output)?;
    let materialization_artifact = assemble_materialization_artifact(ReportAssemblyInput {
        profile: profile.clone(),
        profile_identity_blake3: profile_identity.clone(),
        cohort_identity_blake3: cohort_identity.clone(),
        source_admission,
        matrix_evaluation,
        check_evaluation,
        requested_claim_class: request.requested_claim_class,
    })?;
    let materialization_bytes =
        serde_json::to_vec_pretty(&materialization_artifact).map_err(|error| ShellError::Json(error.to_string()))?;
    write_bundle_file(output, REPORT_PATH, &materialization_bytes)?;
    members.push(measured_member(output, REPORT_PATH, BundleRole::MaterializationReport, &profile)?);
    let mut non_claims = profile.non_claims.clone();
    non_claims.sort();
    let non_claim_bytes = serde_json::to_vec_pretty(&NonClaimArtifact {
        schema: String::from("mantle-spacewasm-non-claims-v1"),
        non_claims,
    })
    .map_err(|error| ShellError::Json(error.to_string()))?;
    write_bundle_file(output, NON_CLAIMS_PATH, &non_claim_bytes)?;
    members.push(measured_member(output, NON_CLAIMS_PATH, BundleRole::NonClaims, &profile)?);
    let parent_edges =
        plan_bundle_parent_edges(members.clone(), String::from(PROFILE_ROOT_PATH)).map_err(ShellError::Core)?;
    let manifest = build_bundle_manifest(BundleManifestInput {
        profile: profile.clone(),
        profile_identity_blake3: profile_identity.clone(),
        cohort_identity_blake3: cohort_identity.clone(),
        members: members.clone(),
        parent_edges,
        non_claims: profile.non_claims.clone(),
    })
    .map_err(ShellError::Core)?;
    write_manifest(output, &manifest)?;
    let verification = verify_bundle(output)?;
    if !verification.valid {
        return Err(ShellError::Core(verification.diagnostics));
    }
    let member_count =
        u32::try_from(members.len()).map_err(|_| ShellError::Invalid(String::from("bundle member count overflow")))?;
    debug_assert!(member_count > 0);
    debug_assert!(verification.bundle_identity_blake3.is_some());
    Ok(MaterializationSummary {
        schema: String::from(MATERIALIZATION_SUMMARY_SCHEMA),
        profile_identity_blake3: profile_identity,
        cohort_identity_blake3: cohort_identity,
        report_identity_blake3: materialization_artifact.report_identity_blake3,
        bundle_identity_blake3: manifest.bundle_identity_blake3,
        member_count,
        valid: true,
    })
}

pub fn verify_bundle(root: &Path) -> Result<VerifySummary, ShellError> {
    require_real_directory(root)?;
    let manifest_path = root.join(MANIFEST_PATH);
    let manifest: BundleManifest = read_json(&manifest_path, HARD_MAX_REPORT_BYTES)?;
    let declared: BTreeMap<_, _> =
        manifest.members.iter().map(|member| (member.path.clone(), member.role.clone())).collect();
    let paths = collect_bundle_files(root, HARD_MAX_BUNDLE_FILES)?;
    let mut measured = Vec::with_capacity(paths.len());
    for relative in paths {
        if relative == MANIFEST_PATH {
            continue;
        }
        let role = declared.get(&relative).cloned().unwrap_or(BundleRole::NonClaims);
        measured.push(measured_member_unbounded(root, &relative, role, &manifest)?);
    }
    let mut diagnostics = verify_bundle_manifest(manifest.clone(), measured).diagnostics;
    let materialization_artifact_path = root.join(REPORT_PATH);
    match read_json::<crunch_spacewasm_core::MaterializationReport>(
        &materialization_artifact_path,
        HARD_MAX_REPORT_BYTES,
    ) {
        Ok(materialization_artifact) => {
            diagnostics.extend(validate_materialization_report(materialization_artifact).diagnostics);
        }
        Err(error) => {
            let message = error.to_string();
            diagnostics.push(shell_diagnostic(ShellDiagnosticInput {
                code: "report-read-failed",
                subject: REPORT_PATH,
                message: &message,
            }));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    let is_valid = diagnostics.is_empty();
    debug_assert_eq!(is_valid, diagnostics.is_empty());
    debug_assert!(diagnostics.iter().all(|item| !item.code.is_empty()));
    Ok(VerifySummary {
        schema: String::from(VERIFY_SUMMARY_SCHEMA),
        valid: is_valid,
        bundle_identity_blake3: Some(manifest.bundle_identity_blake3),
        diagnostics,
    })
}

pub fn check_profile(path: &Path) -> Result<VerifySummary, ShellError> {
    let profile: ReferenceProfile = read_json(path, HARD_MAX_PROFILE_BYTES)?;
    let validation = validate_profile(profile);
    let is_valid = validation.profile_identity_blake3.is_some();
    debug_assert_eq!(is_valid, validation.diagnostics.is_empty());
    debug_assert!(validation.diagnostics.iter().all(|item| !item.code.is_empty()));
    Ok(VerifySummary {
        schema: String::from(VERIFY_SUMMARY_SCHEMA),
        valid: is_valid,
        bundle_identity_blake3: validation.profile_identity_blake3,
        diagnostics: validation.diagnostics,
    })
}

struct ReportAssemblyInput {
    profile: ReferenceProfile,
    profile_identity_blake3: Blake3Digest,
    cohort_identity_blake3: Blake3Digest,
    source_admission: SourceAdmission,
    matrix_evaluation: SupportComparison,
    check_evaluation: CheckEvaluation,
    requested_claim_class: String,
}

fn assemble_materialization_artifact(
    input: ReportAssemblyInput,
) -> Result<crunch_spacewasm_core::MaterializationReport, ShellError> {
    let core_input = ReportBuildInput {
        profile: input.profile,
        profile_identity_blake3: input.profile_identity_blake3,
        cohort_identity_blake3: input.cohort_identity_blake3,
        source_admission: input.source_admission,
        support_comparison: input.matrix_evaluation,
        checks: input.check_evaluation.decisions,
        check_evaluation_complete: input.check_evaluation.complete,
        diagnostics: input.check_evaluation.diagnostics,
        requested_claim_class: input.requested_claim_class,
    };
    build_materialization_report(core_input).map_err(ShellError::Core)
}

fn validate_request_header(request: &MaterializationRequest) -> Result<(), ShellError> {
    if request.schema != MATERIALIZATION_REQUEST_SCHEMA {
        return Err(ShellError::Invalid(format!("unsupported request schema `{}`", request.schema)));
    }
    if request.requested_claim_class != REFERENCE_CLAIM_CLASS {
        return Err(ShellError::Invalid(String::from("request attempts unsupported SpaceWasm claim promotion")));
    }
    let member_count = u32::try_from(request.members.len())
        .map_err(|_| ShellError::Invalid(String::from("request member count exceeds the hard bound")))?;
    if member_count == 0 || member_count > HARD_MAX_BUNDLE_FILES {
        return Err(ShellError::Invalid(String::from("request member count exceeds the hard bound")));
    }
    debug_assert!(!request.profile_path.is_empty());
    debug_assert!(!request.members.is_empty());
    Ok(())
}

fn copy_input_members(
    profile: &ReferenceProfile,
    inputs: &[InputMember],
    output: &Path,
) -> Result<Vec<BundleMember>, ShellError> {
    let mut members = Vec::with_capacity(inputs.len());
    for input in inputs {
        require_safe_relative_path(&input.bundle_path)?;
        let source = Path::new(&input.source_path);
        let bytes = read_regular_file_bounded(source, profile.bounds.max_bundle_member_bytes)?;
        let size_bytes = u64::try_from(bytes.len())
            .map_err(|_| ShellError::Invalid(String::from("bundle member byte count does not fit u64")))?;
        write_bundle_file(output, &input.bundle_path, &bytes)?;
        members.push(BundleMember {
            path: input.bundle_path.clone(),
            role: input.role.clone(),
            digest_blake3: Blake3Digest::from_slice(&bytes),
            size_bytes,
        });
    }
    members.sort();
    debug_assert_eq!(members.len(), inputs.len());
    debug_assert!(members.windows(crunch_spacewasm_core::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    Ok(members)
}

fn measured_member(
    root: &Path,
    relative: &str,
    role: BundleRole,
    profile: &ReferenceProfile,
) -> Result<BundleMember, ShellError> {
    let bytes = read_regular_file_bounded(&root.join(relative), profile.bounds.max_bundle_member_bytes)?;
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| ShellError::Invalid(String::from("bundle member byte count does not fit u64")))?;
    Ok(BundleMember {
        path: String::from(relative),
        role,
        digest_blake3: Blake3Digest::from_slice(&bytes),
        size_bytes,
    })
}

fn measured_member_unbounded(
    root: &Path,
    relative: &str,
    role: BundleRole,
    manifest: &BundleManifest,
) -> Result<BundleMember, ShellError> {
    let maximum = manifest
        .members
        .iter()
        .find(|member| member.path == relative)
        .map_or(HARD_MAX_REPORT_BYTES, |member| member.size_bytes.max(1));
    let bytes = read_regular_file_bounded(&root.join(relative), maximum)?;
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| ShellError::Invalid(String::from("bundle member byte count does not fit u64")))?;
    Ok(BundleMember {
        path: String::from(relative),
        role,
        digest_blake3: Blake3Digest::from_slice(&bytes),
        size_bytes,
    })
}

fn write_manifest(root: &Path, manifest: &BundleManifest) -> Result<(), ShellError> {
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| ShellError::Json(error.to_string()))?;
    write_bundle_file(root, MANIFEST_PATH, &bytes)
}

fn write_bundle_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), ShellError> {
    require_safe_relative_path(relative)?;
    let path = root.join(relative);
    let parent = path.parent().ok_or_else(|| ShellError::Invalid(String::from("bundle member has no parent")))?;
    fs::create_dir_all(parent).map_err(|error| io_error("creating bundle member parent", parent, error))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| io_error("creating bundle member", &path, error))?;
    output.write_all(bytes).map_err(|error| io_error("writing bundle member", &path, error))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(path.starts_with(root));
    Ok(())
}

fn collect_bundle_files(root: &Path, maximum_files: u32) -> Result<Vec<String>, ShellError> {
    let file_slots = usize::try_from(maximum_files)
        .map_err(|_| ShellError::Invalid(String::from("bundle file bound does not fit memory bounds")))?;
    let mut pending = vec![PathBuf::from(root)];
    let mut files = Vec::with_capacity(file_slots);
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .map_err(|error| io_error("reading bundle directory", &directory, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| io_error("reading bundle entry", &directory, error))?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries.into_iter().rev() {
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| io_error("inspecting bundle entry", &path, error))?;
            if metadata.file_type().is_symlink() {
                return Err(ShellError::Invalid(format!("bundle contains a symlink: {}", path.display())));
            }
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(ShellError::Invalid(format!("bundle contains a special file: {}", path.display())));
            }
            if files.len() >= file_slots {
                return Err(ShellError::Invalid(String::from("bundle file count exceeds the hard bound")));
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| ShellError::Invalid(String::from("bundle path escaped root")))?
                .to_str()
                .ok_or_else(|| ShellError::Invalid(String::from("bundle path is not UTF-8")))?;
            require_safe_relative_path(relative)?;
            files.push(String::from(relative));
        }
    }
    files.sort();
    debug_assert!(files.windows(crunch_spacewasm_core::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(files.len() <= file_slots);
    Ok(files)
}

fn require_real_directory(path: &Path) -> Result<(), ShellError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("inspecting bundle root", path, error))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(ShellError::Invalid(format!("bundle root is not a real directory: {}", path.display())));
    }
    debug_assert!(metadata.is_dir());
    debug_assert!(!metadata.file_type().is_symlink());
    Ok(())
}

fn require_safe_relative_path(path: &str) -> Result<(), ShellError> {
    let is_safe = !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|component| !component.is_empty() && component != "." && component != "..");
    if !is_safe {
        return Err(ShellError::Invalid(format!("unsafe bundle member path `{path}`")));
    }
    Ok(())
}

fn read_json<T>(path: &Path, maximum_bytes: u64) -> Result<T, ShellError>
where T: for<'de> Deserialize<'de> {
    let bytes = read_regular_file_bounded(path, maximum_bytes)?;
    serde_json::from_slice(&bytes).map_err(|error| ShellError::Json(error.to_string()))
}

fn read_regular_file_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, ShellError> {
    let (mut file, initial_metadata) = open_regular_file_no_follow(path)?;
    let initial_size_bytes = initial_metadata.len();
    if initial_size_bytes == 0 || initial_size_bytes > maximum_bytes {
        return Err(ShellError::Invalid(format!("input is empty or exceeds its byte bound: {}", path.display())));
    }
    let capacity_bytes = usize::try_from(initial_size_bytes)
        .map_err(|_| ShellError::Invalid(String::from("input byte count does not fit memory bounds")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    let mut buffer = [0_u8; READ_BUFFER_CAPACITY_BYTES];
    let mut bytes_read = 0_u64;
    while bytes_read < initial_size_bytes {
        let chunk_bytes = file.read(&mut buffer).map_err(|error| io_error("reading input", path, error))?;
        if chunk_bytes == 0 {
            break;
        }
        let chunk_size_bytes = u64::try_from(chunk_bytes)
            .map_err(|_| ShellError::Invalid(String::from("input read byte count does not fit u64")))?;
        bytes_read = bytes_read
            .checked_add(chunk_size_bytes)
            .ok_or_else(|| ShellError::Invalid(String::from("input byte count overflow")))?;
        if bytes_read > maximum_bytes {
            return Err(ShellError::Invalid(format!("input exceeds its byte bound: {}", path.display())));
        }
        bytes.extend_from_slice(&buffer[..chunk_bytes]);
    }
    let final_size_bytes = file.metadata().map_err(|error| io_error("remeasuring input", path, error))?.len();
    if bytes_read != initial_size_bytes || final_size_bytes != initial_size_bytes {
        return Err(ShellError::Invalid(format!("input changed while reading: {}", path.display())));
    }
    let buffered_size_bytes = u64::try_from(bytes.len())
        .map_err(|_| ShellError::Invalid(String::from("buffered input byte count does not fit u64")))?;
    debug_assert_eq!(bytes_read, buffered_size_bytes);
    debug_assert!(bytes_read <= maximum_bytes);
    Ok(bytes)
}

fn open_regular_file_no_follow(path: &Path) -> Result<(File, fs::Metadata), ShellError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path).map_err(|error| io_error("opening no-follow input", path, error))?;
    let metadata = file.metadata().map_err(|error| io_error("inspecting opened input", path, error))?;
    if !metadata.is_file() {
        return Err(ShellError::Invalid(format!("input is not a regular file: {}", path.display())));
    }
    debug_assert!(metadata.file_type().is_file());
    debug_assert!(!metadata.file_type().is_dir());
    Ok((file, metadata))
}

struct ShellDiagnosticInput<'a> {
    code: &'a str,
    subject: &'a str,
    message: &'a str,
}

fn shell_diagnostic(input: ShellDiagnosticInput<'_>) -> Diagnostic {
    Diagnostic {
        severity: crunch_spacewasm_core::DiagnosticSeverity::Error,
        code: String::from(input.code),
        subject: String::from(input.subject),
        message: String::from(input.message),
    }
}

fn cleanup_failed_materialization(output: &Path, materialization_error: ShellError) -> ShellError {
    match fs::remove_dir_all(output) {
        Ok(()) => materialization_error,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => materialization_error,
        Err(error) => ShellError::Cleanup {
            path: output.display().to_string(),
            materialization_error: Box::new(materialization_error),
            cleanup_message: error.to_string(),
        },
    }
}

fn io_error(operation: &'static str, path: &Path, error: std::io::Error) -> ShellError {
    ShellError::Io {
        operation,
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

#[derive(Serialize)]
struct NonClaimArtifact {
    schema: String,
    non_claims: Vec<String>,
}

#[cfg(test)]
mod tests;
