use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crunch_kernelscript_core::ArtifactIdentity;
use crunch_kernelscript_core::ArtifactOutputClass;
use crunch_kernelscript_core::Blake3Digest;
use crunch_kernelscript_core::CodegenPlan;
use crunch_kernelscript_core::CompilationPlan;
use crunch_kernelscript_core::CompilerAdmission;
use crunch_kernelscript_core::CompilerCohort;
use crunch_kernelscript_core::CompilerDependency;
use crunch_kernelscript_core::CompilerDependencyLock;
use crunch_kernelscript_core::CompilerMaterializationFacts;
use crunch_kernelscript_core::ExpectedGeneratedFile;
use crunch_kernelscript_core::ExperimentBlocker;
use crunch_kernelscript_core::ExperimentBounds;
use crunch_kernelscript_core::ExperimentProfile;
use crunch_kernelscript_core::ExperimentReceipt;
use crunch_kernelscript_core::ExperimentReceiptInput;
use crunch_kernelscript_core::ExperimentStageStatus;
use crunch_kernelscript_core::GeneratedProjectFacts;
use crunch_kernelscript_core::GeneratedProjectManifest;
use crunch_kernelscript_core::KernelArchitecture;
use crunch_kernelscript_core::KernelTarget;
use crunch_kernelscript_core::NIX_CLOSURE_OBSERVATION_LOCK_FORMAT;
use crunch_kernelscript_core::NetworkPolicy;
use crunch_kernelscript_core::OBSERVATION_KERNEL_BUILD_IDENTITY_PREFIX;
use crunch_kernelscript_core::ObservedGeneratedFile;
use crunch_kernelscript_core::ResolvedKernelTargetFacts;
use crunch_kernelscript_core::Sha256Digest;
use crunch_kernelscript_core::SourceIdentity;
use crunch_kernelscript_core::TargetAdmission;
use crunch_kernelscript_core::TargetInputIdentity;
use crunch_kernelscript_core::TargetInputRole;
use crunch_kernelscript_core::ToolIdentity;
use crunch_kernelscript_core::ToolRole;
use crunch_kernelscript_core::admit_compiler_materialization;
use crunch_kernelscript_core::admit_kernel_target;
use crunch_kernelscript_core::build_experiment_receipt;
use crunch_kernelscript_core::classify_generated_project;
use crunch_kernelscript_core::plan_codegen;
use crunch_kernelscript_core::plan_compilation;
use crunch_kernelscript_core::validate_profile;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest as Sha2Digest;
use sha2::Sha256;

pub const ADAPTER_REQUEST_SCHEMA: &str = "mantle-kernelscript-core-adapter-request-v1";
pub const ADAPTER_REPORT_SCHEMA: &str = "mantle-kernelscript-core-adapter-report-v1";

const HASH_BUFFER_BYTES: usize = 65_536;
const MAX_IDENTITY_FILE_BYTES: u64 = 536_870_912;
const MAX_REQUEST_BYTES: u64 = 1_048_576;
const HARD_MAX_GENERATED_FILES: u32 = 64;
const HARD_MAX_GENERATED_FILE_BYTES: u64 = 8_388_608;
const HARD_MAX_GENERATED_TOTAL_BYTES: u64 = 33_554_432;
const HARD_MAX_COMPILATION_STEPS: u32 = 32;
const HARD_MAX_OUTPUT_FILES: u32 = 16;
const HARD_MAX_OUTPUT_BYTES: u64 = 67_108_864;
const HARD_MAX_ELF_SECTIONS: u32 = 512;
const HARD_MAX_SECTION_NAME_BYTES: u32 = 256;
const HARD_MAX_RECEIPT_BLOCKERS: u32 = 128;
const HARD_MAX_TEXT_BYTES: u32 = 4_096;
const HARD_MAX_TOOLCHAIN_MEMBERS: u32 = 16;
const TARGET_OBSERVATION_BLOCKER_CODE: &str = "kernel-target-observation-only";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequest {
    pub relative_path: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerRequest {
    pub version: String,
    pub source_revision: String,
    pub source_archive_url: String,
    pub source_archive_path: String,
    pub source_archive_sha256: String,
    pub source_archive_blake3: String,
    pub executable_path: String,
    pub closure_path_set_path: String,
    pub closure_package: String,
    pub closure_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolRequest {
    pub role: ToolRole,
    pub version: String,
    pub path: String,
    pub configuration: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetRequest {
    pub architecture: KernelArchitecture,
    pub kernel_release: String,
    pub cohort_label: String,
    pub btf_path: String,
    pub headers_marker_path: String,
    pub config_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreAdapterRequest {
    pub schema: String,
    pub experiment_id: String,
    pub source: SourceRequest,
    pub compiler: CompilerRequest,
    pub toolchain: Vec<ToolRequest>,
    pub target: TargetRequest,
    pub output_classes: Vec<ArtifactOutputClass>,
    pub expected_generated_files: Vec<ExpectedGeneratedFile>,
    pub bpf_compiler_flags: Vec<String>,
    pub userspace_compiler_flags: Vec<String>,
    pub module_compiler_flags: Vec<String>,
    pub bounds: ExperimentBounds,
    pub receipt_blockers: Vec<ExperimentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreAdapterReport {
    pub schema: String,
    pub profile: ExperimentProfile,
    pub compiler_admission: CompilerAdmission,
    pub target_admission: TargetAdmission,
    pub codegen_plan: CodegenPlan,
    pub generated_manifest: GeneratedProjectManifest,
    pub compilation_plan: CompilationPlan,
    pub receipt: ExperimentReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    Io {
        operation: &'static str,
        path: String,
        message: String,
    },
    Invalid(String),
    CoreRejected(Vec<ExperimentBlocker>),
    Json(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                message,
            } => write!(formatter, "{operation} `{path}`: {message}"),
            Self::Invalid(message) => formatter.write_str(message),
            Self::CoreRejected(blockers) => {
                let codes = blockers.iter().map(|blocker| blocker.code.as_str()).collect::<Vec<_>>().join(",");
                write!(formatter, "KernelScript core rejected adapter facts: {codes}")
            }
            Self::Json(message) => write!(formatter, "KernelScript adapter JSON error: {message}"),
        }
    }
}

impl std::error::Error for AdapterError {}

pub fn read_request(path: &Path) -> Result<CoreAdapterRequest, AdapterError> {
    let bytes = read_regular_file_bounded(path, MAX_REQUEST_BYTES)?;
    let request = serde_json::from_slice(&bytes).map_err(|error| AdapterError::Json(error.to_string()))?;
    let maximum_request_bytes = usize::try_from(MAX_REQUEST_BYTES)
        .map_err(|_| AdapterError::Invalid(String::from("request byte bound does not fit memory bounds")))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(bytes.len() <= maximum_request_bytes);
    Ok(request)
}

pub fn write_report(path: &Path, report: &CoreAdapterReport) -> Result<(), AdapterError> {
    let bytes = serde_json::to_vec_pretty(report).map_err(|error| AdapterError::Json(error.to_string()))?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("creating report", path, error))?;
    output.write_all(&bytes).map_err(|error| io_error("writing report", path, error))?;
    debug_assert!(!bytes.is_empty());
    debug_assert_eq!(report.schema, ADAPTER_REPORT_SCHEMA);
    Ok(())
}

pub fn run_request(request: CoreAdapterRequest, generated_root: &Path) -> Result<CoreAdapterReport, AdapterError> {
    validate_request_header(&request)?;
    let profile = observe_profile(&request)?;
    let validation = validate_profile(profile.clone());
    require_clean_core_result(validation.blockers)?;
    let compiler_admission = observe_compiler_admission(&request, &profile)?;
    let codegen_plan = plan_codegen(profile.clone()).map_err(AdapterError::CoreRejected)?;
    let generated_manifest = observe_generated_manifest(&profile, generated_root)?;
    let compilation = plan_compilation(profile.clone(), generated_manifest.clone());
    let compilation_plan = compilation.plan.ok_or(AdapterError::CoreRejected(compilation.blockers))?;
    let target_admission = observe_target_admission(&profile)?;
    let receipt = build_blocked_receipt(BlockedReceiptInput {
        request: &request,
        profile: &profile,
        compiler_admission: &compiler_admission,
        target_admission: &target_admission,
        codegen_plan: &codegen_plan,
        generated_manifest: &generated_manifest,
        compilation_plan: &compilation_plan,
    })?;
    let adapter_output = CoreAdapterReport {
        schema: String::from(ADAPTER_REPORT_SCHEMA),
        profile,
        compiler_admission,
        target_admission,
        codegen_plan,
        generated_manifest,
        compilation_plan,
        receipt,
    };
    debug_assert!(adapter_output.compiler_admission.admitted);
    debug_assert!(!adapter_output.target_admission.admitted);
    Ok(adapter_output)
}

fn validate_request_header(request: &CoreAdapterRequest) -> Result<(), AdapterError> {
    if request.schema != ADAPTER_REQUEST_SCHEMA {
        return Err(AdapterError::Invalid(format!(
            "unsupported KernelScript adapter request schema `{}`",
            request.schema
        )));
    }
    validate_request_bounds(request)?;
    if request.receipt_blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE) {
        return Err(AdapterError::Invalid(String::from(
            "target observation blocker is adapter-owned and must not be supplied",
        )));
    }
    debug_assert!(!request.experiment_id.is_empty() || validate_profile_placeholder(request));
    debug_assert!(request.receipt_blockers.iter().all(|blocker| !blocker.code.is_empty()));
    Ok(())
}

fn validate_request_bounds(request: &CoreAdapterRequest) -> Result<(), AdapterError> {
    let bounds = &request.bounds;
    let is_valid = is_bounded_u32(BoundU32Validation {
        value: bounds.max_generated_files,
        maximum: HARD_MAX_GENERATED_FILES,
    }) && is_bounded_u64(BoundU64Validation {
        value: bounds.max_generated_file_bytes,
        maximum: HARD_MAX_GENERATED_FILE_BYTES,
    }) && is_bounded_u64(BoundU64Validation {
        value: bounds.max_generated_total_bytes,
        maximum: HARD_MAX_GENERATED_TOTAL_BYTES,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_compilation_steps,
        maximum: HARD_MAX_COMPILATION_STEPS,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_output_files,
        maximum: HARD_MAX_OUTPUT_FILES,
    }) && is_bounded_u64(BoundU64Validation {
        value: bounds.max_output_bytes,
        maximum: HARD_MAX_OUTPUT_BYTES,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_elf_sections,
        maximum: HARD_MAX_ELF_SECTIONS,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_section_name_bytes,
        maximum: HARD_MAX_SECTION_NAME_BYTES,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_receipt_blockers,
        maximum: HARD_MAX_RECEIPT_BLOCKERS,
    }) && is_bounded_u32(BoundU32Validation {
        value: bounds.max_text_bytes,
        maximum: HARD_MAX_TEXT_BYTES,
    });
    let expected_file_count = u32::try_from(request.expected_generated_files.len())
        .map_err(|_| AdapterError::Invalid(String::from("expected generated-file count exceeds u32 bounds")))?;
    let toolchain_member_count = u32::try_from(request.toolchain.len())
        .map_err(|_| AdapterError::Invalid(String::from("toolchain-member count exceeds u32 bounds")))?;
    if !is_valid || expected_file_count == 0 || expected_file_count > bounds.max_generated_files {
        return Err(AdapterError::Invalid(String::from(
            "KernelScript adapter request exceeds hard generated/output/text bounds",
        )));
    }
    if toolchain_member_count == 0 || toolchain_member_count > HARD_MAX_TOOLCHAIN_MEMBERS {
        return Err(AdapterError::Invalid(String::from(
            "KernelScript adapter request exceeds the hard toolchain-member bound",
        )));
    }
    debug_assert!(bounds.max_generated_file_bytes <= HARD_MAX_GENERATED_FILE_BYTES);
    debug_assert!(bounds.max_generated_total_bytes <= HARD_MAX_GENERATED_TOTAL_BYTES);
    Ok(())
}

#[derive(Clone, Copy)]
struct BoundU32Validation {
    value: u32,
    maximum: u32,
}

#[derive(Clone, Copy)]
struct BoundU64Validation {
    value: u64,
    maximum: u64,
}

fn is_bounded_u32(validation: BoundU32Validation) -> bool {
    validation.value > 0 && validation.value <= validation.maximum
}

fn is_bounded_u64(validation: BoundU64Validation) -> bool {
    validation.value > 0 && validation.value <= validation.maximum
}

fn observe_profile(request: &CoreAdapterRequest) -> Result<ExperimentProfile, AdapterError> {
    let source_bytes =
        read_regular_file_bounded(Path::new(&request.source.path), request.bounds.max_generated_file_bytes)?;
    let compiler_executable = artifact_identity(Path::new(&request.compiler.executable_path))?;
    let closure_identity = artifact_identity(Path::new(&request.compiler.closure_path_set_path))?;
    let dependency = CompilerDependency {
        package: request.compiler.closure_package.clone(),
        version: request.compiler.closure_version.clone(),
        artifact: closure_identity.clone(),
    };
    let toolchain = observe_toolchain(&request.toolchain)?;
    let target = observe_target(&request.target)?;
    let source_size_bytes = u64::try_from(source_bytes.len())
        .map_err(|_| AdapterError::Invalid(String::from("source byte count exceeds u64 bounds")))?;
    let profile = ExperimentProfile {
        schema: String::from(crunch_kernelscript_core::EXPERIMENT_PROFILE_SCHEMA),
        experiment_id: request.experiment_id.clone(),
        beta: true,
        enabled_by_default: false,
        source: SourceIdentity {
            relative_path: request.source.relative_path.clone(),
            digest_blake3: Blake3Digest::from_slice(&source_bytes),
            size_bytes: source_size_bytes,
        },
        compiler: compiler_cohort(request, compiler_executable, closure_identity, dependency)?,
        toolchain,
        target,
        output_classes: request.output_classes.clone(),
        expected_generated_files: request.expected_generated_files.clone(),
        bpf_compiler_flags: request.bpf_compiler_flags.clone(),
        userspace_compiler_flags: request.userspace_compiler_flags.clone(),
        module_compiler_flags: request.module_compiler_flags.clone(),
        bounds: request.bounds.clone(),
        network_policy: NetworkPolicy::Denied,
        non_claims: crunch_kernelscript_core::REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect(),
    };
    debug_assert!(profile.beta);
    debug_assert!(!profile.enabled_by_default);
    Ok(profile)
}

fn compiler_cohort(
    request: &CoreAdapterRequest,
    compiler_executable: ArtifactIdentity,
    closure_identity: ArtifactIdentity,
    dependency: CompilerDependency,
) -> Result<CompilerCohort, AdapterError> {
    let dependency_count = 1_u32;
    let cohort = CompilerCohort {
        version: request.compiler.version.clone(),
        source_revision: request.compiler.source_revision.clone(),
        source_archive_url: request.compiler.source_archive_url.clone(),
        source_archive_sha256: Sha256Digest::parse(request.compiler.source_archive_sha256.clone())
            .map_err(|error| AdapterError::Invalid(error.to_string()))?,
        source_archive_blake3: Blake3Digest::parse(request.compiler.source_archive_blake3.clone())
            .map_err(|error| AdapterError::Invalid(error.to_string()))?,
        dependency_lock: CompilerDependencyLock {
            format: String::from(NIX_CLOSURE_OBSERVATION_LOCK_FORMAT),
            dependency_count,
            dependencies: vec![dependency],
            lock_blake3: closure_identity.digest_blake3.clone(),
        },
        compiler_executable,
        closure_blake3: closure_identity.digest_blake3,
    };
    debug_assert_eq!(cohort.dependency_lock.dependency_count, dependency_count);
    debug_assert_eq!(cohort.dependency_lock.dependencies.len(), 1);
    Ok(cohort)
}

fn observe_toolchain(requests: &[ToolRequest]) -> Result<Vec<ToolIdentity>, AdapterError> {
    let mut tools = Vec::with_capacity(requests.len());
    for request in requests {
        tools.push(ToolIdentity {
            role: request.role,
            version: request.version.clone(),
            executable_or_library: artifact_identity(Path::new(&request.path))?,
            configuration_blake3: Blake3Digest::from_slice(request.configuration.as_bytes()),
        });
    }
    debug_assert_eq!(tools.len(), requests.len());
    debug_assert!(tools.iter().all(|tool| !tool.version.is_empty()));
    Ok(tools)
}

fn observe_target(request: &TargetRequest) -> Result<KernelTarget, AdapterError> {
    let btf = artifact_identity(Path::new(&request.btf_path))?;
    let headers = artifact_identity(Path::new(&request.headers_marker_path))?;
    let config = artifact_identity(Path::new(&request.config_path))?;
    let identity = observation_kernel_identity(request, &btf, &headers, &config)?;
    let input = |role, artifact| TargetInputIdentity {
        role,
        artifact,
        architecture: request.architecture,
        kernel_release: request.kernel_release.clone(),
        kernel_build_identity: identity.clone(),
    };
    let target = KernelTarget {
        architecture: request.architecture,
        kernel_release: request.kernel_release.clone(),
        kernel_build_identity: identity.clone(),
        btf: input(TargetInputRole::Btf, btf),
        headers: input(TargetInputRole::Headers, headers),
        config: input(TargetInputRole::Config, config),
    };
    debug_assert_eq!(target.architecture, request.architecture);
    debug_assert_eq!(target.kernel_build_identity, identity);
    Ok(target)
}

#[derive(Serialize)]
struct ObservationKernelIdentity<'a> {
    architecture: KernelArchitecture,
    kernel_release: &'a str,
    cohort_label: &'a str,
    btf: &'a ArtifactIdentity,
    headers: &'a ArtifactIdentity,
    config: &'a ArtifactIdentity,
}

fn observation_kernel_identity(
    request: &TargetRequest,
    btf: &ArtifactIdentity,
    headers: &ArtifactIdentity,
    config: &ArtifactIdentity,
) -> Result<String, AdapterError> {
    let input = ObservationKernelIdentity {
        architecture: request.architecture,
        kernel_release: &request.kernel_release,
        cohort_label: &request.cohort_label,
        btf,
        headers,
        config,
    };
    let bytes = serde_json::to_vec(&input).map_err(|error| AdapterError::Json(error.to_string()))?;
    let digest = Blake3Digest::from_slice(&bytes);
    debug_assert!(!request.cohort_label.is_empty());
    debug_assert!(!bytes.is_empty());
    Ok(format!("{OBSERVATION_KERNEL_BUILD_IDENTITY_PREFIX}{digest}"))
}

fn observe_compiler_admission(
    request: &CoreAdapterRequest,
    profile: &ExperimentProfile,
) -> Result<CompilerAdmission, AdapterError> {
    let archive = hash_source_archive(Path::new(&request.compiler.source_archive_path))?;
    let source_archive_sha256 = archive.sha256.ok_or_else(|| {
        AdapterError::Invalid(String::from("source archive measurement did not include required SHA-256"))
    })?;
    let dependency = profile.compiler.dependency_lock.dependencies[0].clone();
    let facts = CompilerMaterializationFacts {
        source_revision: request.compiler.source_revision.clone(),
        source_archive_sha256,
        source_archive_blake3: archive.blake3,
        dependency_lock_blake3: Some(profile.compiler.dependency_lock.lock_blake3.clone()),
        dependency_count: profile.compiler.dependency_lock.dependency_count,
        dependencies: vec![dependency],
        compiler_executable: Some(profile.compiler.compiler_executable.clone()),
        closure_blake3: Some(profile.compiler.closure_blake3.clone()),
        network_attempted: false,
        ambient_opam_used: false,
        ambient_compiler_used: false,
    };
    let admission = admit_compiler_materialization(profile.clone(), facts);
    if !admission.admitted {
        return Err(AdapterError::CoreRejected(admission.blockers));
    }
    debug_assert!(admission.materialization_identity_blake3.is_some());
    debug_assert!(admission.blockers.is_empty());
    Ok(admission)
}

fn observe_generated_manifest(
    profile: &ExperimentProfile,
    generated_root: &Path,
) -> Result<GeneratedProjectManifest, AdapterError> {
    let files = read_generated_files(generated_root, GeneratedFileReadBounds {
        maximum_files_count: profile.bounds.max_generated_files,
        maximum_file_bytes: profile.bounds.max_generated_file_bytes,
        maximum_total_bytes: profile.bounds.max_generated_total_bytes,
    })?;
    let result = classify_generated_project(GeneratedProjectFacts {
        profile: profile.clone(),
        files,
    });
    let manifest = result.manifest.ok_or(AdapterError::CoreRejected(result.blockers))?;
    debug_assert!(!manifest.members.is_empty());
    debug_assert!(manifest.total_bytes > 0);
    Ok(manifest)
}

fn observe_target_admission(profile: &ExperimentProfile) -> Result<TargetAdmission, AdapterError> {
    let flags = compiler_flags_identity(profile)?;
    let tools = Blake3Digest::from_slice(
        &serde_json::to_vec(&profile.toolchain).map_err(|error| AdapterError::Json(error.to_string()))?,
    );
    let facts = ResolvedKernelTargetFacts {
        architecture: profile.target.architecture,
        kernel_release: profile.target.kernel_release.clone(),
        kernel_build_identity: profile.target.kernel_build_identity.clone(),
        btf: Some(profile.target.btf.artifact.clone()),
        headers: Some(profile.target.headers.artifact.clone()),
        config: Some(profile.target.config.artifact.clone()),
        compiler_flags_blake3: flags,
        toolchain_blake3: tools,
        ambient_inputs_used: false,
    };
    let admission = admit_kernel_target(profile.clone(), facts);
    let is_observation_blocked =
        admission.blockers.iter().any(|blocker| blocker.code == TARGET_OBSERVATION_BLOCKER_CODE);
    if admission.admitted || !is_observation_blocked {
        return Err(AdapterError::Invalid(String::from(
            "observation target unexpectedly crossed the Onix authority boundary",
        )));
    }
    debug_assert!(admission.target_identity_blake3.is_none());
    debug_assert!(!admission.blockers.is_empty());
    Ok(admission)
}

#[derive(Serialize)]
struct CompilerFlagsIdentity<'a> {
    bpf: &'a [String],
    userspace: &'a [String],
    module: &'a [String],
}

fn compiler_flags_identity(profile: &ExperimentProfile) -> Result<Blake3Digest, AdapterError> {
    let input = CompilerFlagsIdentity {
        bpf: &profile.bpf_compiler_flags,
        userspace: &profile.userspace_compiler_flags,
        module: &profile.module_compiler_flags,
    };
    let bytes = serde_json::to_vec(&input).map_err(|error| AdapterError::Json(error.to_string()))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(!profile.bpf_compiler_flags.is_empty());
    Ok(Blake3Digest::from_slice(&bytes))
}

struct BlockedReceiptInput<'a> {
    request: &'a CoreAdapterRequest,
    profile: &'a ExperimentProfile,
    compiler_admission: &'a CompilerAdmission,
    target_admission: &'a TargetAdmission,
    codegen_plan: &'a CodegenPlan,
    generated_manifest: &'a GeneratedProjectManifest,
    compilation_plan: &'a CompilationPlan,
}

fn build_blocked_receipt(input: BlockedReceiptInput<'_>) -> Result<ExperimentReceipt, AdapterError> {
    let mut blockers = input.target_admission.blockers.clone();
    blockers.extend(input.request.receipt_blockers.clone());
    let result = build_experiment_receipt(ExperimentReceiptInput {
        profile: input.profile.clone(),
        stage_status: ExperimentStageStatus::Blocked,
        compiler_admission: input.compiler_admission.clone(),
        codegen_plan: Some(input.codegen_plan.clone()),
        generated_manifest: Some(input.generated_manifest.clone()),
        target_admission: Some(input.target_admission.clone()),
        compilation_plan: Some(input.compilation_plan.clone()),
        output_classes: Vec::new(),
        output_inspections: Vec::new(),
        candidate_packs: Vec::new(),
        blockers,
    });
    let receipt = result.receipt.ok_or(AdapterError::CoreRejected(result.blockers))?;
    debug_assert_eq!(receipt.stage_status, ExperimentStageStatus::Blocked);
    debug_assert!(!receipt.blockers.is_empty());
    Ok(receipt)
}

#[derive(Clone, Copy)]
struct GeneratedFileReadBounds {
    maximum_files_count: u32,
    maximum_file_bytes: u64,
    maximum_total_bytes: u64,
}

fn read_generated_files(
    root: &Path,
    bounds: GeneratedFileReadBounds,
) -> Result<Vec<ObservedGeneratedFile>, AdapterError> {
    require_real_directory(root)?;
    let mut files = Vec::with_capacity(
        usize::try_from(bounds.maximum_files_count)
            .map_err(|_| AdapterError::Invalid(String::from("generated-file count does not fit memory bounds")))?,
    );
    let mut total_bytes = 0_u64;
    let entries = fs::read_dir(root).map_err(|error| io_error("reading generated directory", root, error))?;
    for entry in entries {
        let observed_file_count = u32::try_from(files.len())
            .map_err(|_| AdapterError::Invalid(String::from("generated-file count exceeds u32 bounds")))?
            .checked_add(1)
            .ok_or_else(|| AdapterError::Invalid(String::from("generated-file count overflow")))?;
        if observed_file_count > bounds.maximum_files_count || observed_file_count > HARD_MAX_GENERATED_FILES {
            return Err(AdapterError::Invalid(String::from("generated file count exceeds the admitted bound")));
        }
        let entry = entry.map_err(|error| io_error("reading generated directory entry", root, error))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| io_error("inspecting generated file", &path, error))?;
        if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > bounds.maximum_file_bytes {
            return Err(AdapterError::Invalid(format!(
                "generated project member is not bounded regular data: {}",
                path.display()
            )));
        }
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| AdapterError::Invalid(String::from("generated total byte count overflow")))?;
        if total_bytes > bounds.maximum_total_bytes || total_bytes > HARD_MAX_GENERATED_TOTAL_BYTES {
            return Err(AdapterError::Invalid(String::from("generated total bytes exceed the admitted bound")));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| AdapterError::Invalid(String::from("generated project member name is not valid UTF-8")))?;
        files.push(ObservedGeneratedFile {
            relative_path: name,
            bytes: read_regular_file_bounded(&path, bounds.maximum_file_bytes)?,
            executable: generated_file_is_executable(&metadata),
        });
    }
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    debug_assert!(files.windows(2).all(|pair| pair[0].relative_path <= pair[1].relative_path));
    debug_assert!(total_bytes <= bounds.maximum_total_bytes);
    Ok(files)
}

fn require_real_directory(path: &Path) -> Result<(), AdapterError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| io_error("inspecting generated directory", path, error))?;
    if !metadata.file_type().is_dir() {
        return Err(AdapterError::Invalid(format!("generated root is not a real directory: {}", path.display())));
    }
    debug_assert!(!metadata.file_type().is_symlink());
    debug_assert!(metadata.is_dir());
    Ok(())
}

#[cfg(unix)]
fn generated_file_is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    const EXECUTABLE_MODE_BITS: u32 = 0o111;
    metadata.permissions().mode() & EXECUTABLE_MODE_BITS != 0
}

#[cfg(not(unix))]
fn generated_file_is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

fn artifact_identity(path: &Path) -> Result<ArtifactIdentity, AdapterError> {
    let measurement = hash_open_file(open_regular_file_no_follow(path)?, path, MAX_IDENTITY_FILE_BYTES, false)?;
    let artifact_ref = format!("mantle://blake3/{}", measurement.blake3);
    debug_assert!(measurement.size_bytes > 0);
    debug_assert!(artifact_ref.ends_with(measurement.blake3.as_str()));
    Ok(ArtifactIdentity {
        artifact_ref,
        digest_blake3: measurement.blake3,
        size_bytes: measurement.size_bytes,
    })
}

struct FileMeasurement {
    blake3: Blake3Digest,
    sha256: Option<Sha256Digest>,
    size_bytes: u64,
}

fn hash_source_archive(path: &Path) -> Result<FileMeasurement, AdapterError> {
    hash_open_file(open_regular_file_no_follow(path)?, path, MAX_IDENTITY_FILE_BYTES, true)
}

fn hash_open_file(
    opened: (File, fs::Metadata),
    path: &Path,
    maximum_bytes: u64,
    require_sha256: bool,
) -> Result<FileMeasurement, AdapterError> {
    let (mut input, metadata) = opened;
    validate_open_metadata(&metadata, path, maximum_bytes)?;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha256_hasher = Sha256::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let mut total_bytes = 0_u64;
    for _read_attempt_count in 0..=maximum_bytes {
        let count = input.read(&mut buffer).map_err(|error| io_error("reading identity file", path, error))?;
        if count == 0 {
            break;
        }
        total_bytes = checked_read_total(total_bytes, count, maximum_bytes, path)?;
        blake3_hasher.update(&buffer[..count]);
        if require_sha256 {
            sha256_hasher.update(&buffer[..count]);
        }
    }
    require_stable_descriptor_size(
        DescriptorSize {
            observed_bytes: total_bytes,
            expected_bytes: metadata.len(),
        },
        path,
    )?;
    let sha256 = if require_sha256 {
        let hex = format!("{:x}", sha256_hasher.finalize());
        Some(Sha256Digest::parse(hex).map_err(|error| AdapterError::Invalid(error.to_string()))?)
    } else {
        None
    };
    let blake3_hex = blake3::Hasher::finalize(&blake3_hasher).to_hex().to_string();
    let blake3 = Blake3Digest::parse(blake3_hex).map_err(|error| AdapterError::Invalid(error.to_string()))?;
    debug_assert!(total_bytes > 0);
    debug_assert!(total_bytes <= maximum_bytes);
    Ok(FileMeasurement {
        blake3,
        sha256,
        size_bytes: total_bytes,
    })
}

fn read_regular_file_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, AdapterError> {
    read_open_regular_file_bounded(open_regular_file_no_follow(path)?, path, maximum_bytes)
}

fn read_open_regular_file_bounded(
    opened: (File, fs::Metadata),
    path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, AdapterError> {
    let (mut input, metadata) = opened;
    validate_open_metadata(&metadata, path, maximum_bytes)?;
    let input_capacity_bytes = usize::try_from(metadata.len())
        .map_err(|_| AdapterError::Invalid(String::from("input byte count does not fit memory bounds")))?;
    let mut bytes = Vec::with_capacity(input_capacity_bytes);
    input.read_to_end(&mut bytes).map_err(|error| io_error("reading file", path, error))?;
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| AdapterError::Invalid(String::from("input byte count exceeds u64 bounds")))?;
    require_stable_descriptor_size(
        DescriptorSize {
            observed_bytes: byte_count,
            expected_bytes: metadata.len(),
        },
        path,
    )?;
    if byte_count > maximum_bytes {
        return Err(AdapterError::Invalid(format!("input exceeds byte bound: {}", path.display())));
    }
    debug_assert!(!bytes.is_empty());
    debug_assert!(byte_count <= maximum_bytes);
    Ok(bytes)
}

fn open_regular_file_no_follow(path: &Path) -> Result<(File, fs::Metadata), AdapterError> {
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
        return Err(AdapterError::Invalid(format!("opened input is not a regular file: {}", path.display())));
    }
    debug_assert!(metadata.file_type().is_file());
    debug_assert!(!metadata.file_type().is_dir());
    Ok((file, metadata))
}

fn validate_open_metadata(metadata: &fs::Metadata, path: &Path, maximum_bytes: u64) -> Result<(), AdapterError> {
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum_bytes {
        return Err(AdapterError::Invalid(format!(
            "input is not a bounded non-empty regular file: {}",
            path.display()
        )));
    }
    debug_assert!(metadata.len() > 0);
    debug_assert!(metadata.len() <= maximum_bytes);
    Ok(())
}

fn checked_read_total(total: u64, count: usize, maximum: u64, path: &Path) -> Result<u64, AdapterError> {
    let read_bytes = u64::try_from(count)
        .map_err(|_| AdapterError::Invalid(String::from("identity read byte count exceeds u64 bounds")))?;
    let next = total
        .checked_add(read_bytes)
        .ok_or_else(|| AdapterError::Invalid(String::from("identity input byte count overflow")))?;
    if next > maximum {
        return Err(AdapterError::Invalid(format!("identity input exceeds byte bound: {}", path.display())));
    }
    Ok(next)
}

#[derive(Clone, Copy)]
struct DescriptorSize {
    observed_bytes: u64,
    expected_bytes: u64,
}

fn require_stable_descriptor_size(size: DescriptorSize, path: &Path) -> Result<(), AdapterError> {
    if size.observed_bytes != size.expected_bytes {
        return Err(AdapterError::Invalid(format!("opened input changed while reading: {}", path.display())));
    }
    Ok(())
}

fn require_clean_core_result(blockers: Vec<ExperimentBlocker>) -> Result<(), AdapterError> {
    if blockers.is_empty() {
        return Ok(());
    }
    Err(AdapterError::CoreRejected(blockers))
}

fn io_error(operation: &'static str, path: &Path, error: std::io::Error) -> AdapterError {
    AdapterError::Io {
        operation,
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

fn validate_profile_placeholder(request: &CoreAdapterRequest) -> bool {
    request.schema == ADAPTER_REQUEST_SCHEMA && !request.source.relative_path.is_empty()
}

#[cfg(test)]
mod tests;
