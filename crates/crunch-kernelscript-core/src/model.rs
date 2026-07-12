use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::Sha256Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub artifact_ref: String,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub relative_path: String,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerDependency {
    pub package: String,
    pub version: String,
    pub artifact: ArtifactIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerDependencyLock {
    pub format: String,
    pub dependency_count: u32,
    pub dependencies: Vec<CompilerDependency>,
    pub lock_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerCohort {
    pub version: String,
    pub source_revision: String,
    pub source_archive_url: String,
    pub source_archive_sha256: Sha256Digest,
    pub source_archive_blake3: Blake3Digest,
    pub dependency_lock: CompilerDependencyLock,
    pub compiler_executable: ArtifactIdentity,
    pub closure_blake3: Blake3Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolRole {
    Ocaml,
    Dune,
    Menhir,
    Clang,
    CCompiler,
    Bpftool,
    Libbpf,
    ElfLibrary,
    Zlib,
    KernelBuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolIdentity {
    pub role: ToolRole,
    pub version: String,
    pub executable_or_library: ArtifactIdentity,
    pub configuration_blake3: Blake3Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KernelArchitecture {
    X86_64,
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetInputRole {
    Btf,
    Headers,
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInputIdentity {
    pub role: TargetInputRole,
    pub artifact: ArtifactIdentity,
    pub architecture: KernelArchitecture,
    pub kernel_release: String,
    pub kernel_build_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelTarget {
    pub architecture: KernelArchitecture,
    pub kernel_release: String,
    pub kernel_build_identity: String,
    pub btf: TargetInputIdentity,
    pub headers: TargetInputIdentity,
    pub config: TargetInputIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactOutputClass {
    GeneratedSourceBundle,
    UserspaceLoader,
    EbpfObject,
    KernelModule,
    TestBinary,
    CandidatePackManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedFileClass {
    UserspaceC,
    EbpfC,
    ModuleC,
    TestC,
    MakefileEvidence,
    KbuildEvidence,
    DocumentationEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedGeneratedFile {
    pub relative_path: String,
    pub class: GeneratedFileClass,
    pub required: bool,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentBounds {
    pub max_generated_files: u32,
    pub max_generated_file_bytes: u64,
    pub max_generated_total_bytes: u64,
    pub max_compilation_steps: u32,
    pub max_output_files: u32,
    pub max_output_bytes: u64,
    pub max_elf_sections: u32,
    pub max_section_name_bytes: u32,
    pub max_receipt_blockers: u32,
    pub max_text_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentProfile {
    pub schema: String,
    pub experiment_id: String,
    pub beta: bool,
    pub enabled_by_default: bool,
    pub source: SourceIdentity,
    pub compiler: CompilerCohort,
    pub toolchain: Vec<ToolIdentity>,
    pub target: KernelTarget,
    pub output_classes: Vec<ArtifactOutputClass>,
    pub expected_generated_files: Vec<ExpectedGeneratedFile>,
    pub bpf_compiler_flags: Vec<String>,
    pub userspace_compiler_flags: Vec<String>,
    pub module_compiler_flags: Vec<String>,
    pub bounds: ExperimentBounds,
    pub network_policy: NetworkPolicy,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputClassStatus {
    Planned,
    BuildFailed,
    Built,
    InspectionFailed,
    InspectionPassed,
    NotSelected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputMember {
    pub relative_path: String,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
}
