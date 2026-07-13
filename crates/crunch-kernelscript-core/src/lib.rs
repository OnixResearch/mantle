#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure bounded admission, planning, inspection, handoff, and receipt logic for
//! Mantle's explicitly beta KernelScript experiment. Filesystem access, source
//! acquisition, tool execution, sandboxing, and output persistence belong to a
//! future std-facing imperative shell after an authoritative compiler lock and
//! target kernel cohort are available.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod blocker;
mod compiler;
mod digest;
mod generated;
mod handoff;
mod inspection;
mod model;
mod planner;
mod profile;
mod receipt;
mod target;

pub use blocker::ExperimentBlocker;
pub use compiler::CompilerAdmission;
pub use compiler::CompilerMaterializationFacts;
pub use compiler::admit_compiler_materialization;
pub use digest::BLAKE3_HEX_LENGTH;
pub use digest::Blake3Digest;
pub use digest::DigestError;
pub use digest::SHA256_HEX_LENGTH;
pub use digest::Sha256Digest;
pub use generated::GENERATED_PROJECT_MANIFEST_SCHEMA;
pub use generated::GeneratedFileMember;
pub use generated::GeneratedProjectFacts;
pub use generated::GeneratedProjectManifest;
pub use generated::GeneratedProjectResult;
pub use generated::ObservedGeneratedFile;
pub use generated::classify_generated_project;
pub use handoff::CANDIDATE_PACK_SCHEMA;
pub use handoff::CandidatePackKind;
pub use handoff::CandidatePackProjection;
pub use handoff::CandidateProjectionResult;
pub use handoff::CandidateReadiness;
pub use handoff::project_candidate_packs;
pub use inspection::ELF_CLASS_64;
pub use inspection::ELF_MACHINE_AARCH64;
pub use inspection::ELF_MACHINE_BPF;
pub use inspection::ELF_MACHINE_X86_64;
pub use inspection::ELF_TYPE_DYNAMIC;
pub use inspection::ELF_TYPE_EXECUTABLE;
pub use inspection::ELF_TYPE_RELOCATABLE;
pub use inspection::InspectionResult;
pub use inspection::ObservedOutput;
pub use inspection::OutputInspection;
pub use inspection::inspect_output;
pub use model::ArtifactIdentity;
pub use model::ArtifactOutputClass;
pub use model::CompilerCohort;
pub use model::CompilerDependency;
pub use model::CompilerDependencyLock;
pub use model::ExpectedGeneratedFile;
pub use model::ExperimentBounds;
pub use model::ExperimentProfile;
pub use model::GeneratedFileClass;
pub use model::KernelArchitecture;
pub use model::KernelTarget;
pub use model::NetworkPolicy;
pub use model::OutputClassStatus;
pub use model::OutputMember;
pub use model::SourceIdentity;
pub use model::TargetInputIdentity;
pub use model::TargetInputRole;
pub use model::ToolIdentity;
pub use model::ToolRole;
pub use planner::CODEGEN_PLAN_SCHEMA;
pub use planner::COMPILATION_PLAN_SCHEMA;
pub use planner::CodegenPlan;
pub use planner::CompilationPlan;
pub use planner::CompilationPlanResult;
pub use planner::ExecutionAdmission;
pub use planner::ExecutionRequest;
pub use planner::PlanAction;
pub use planner::PlanStep;
pub use planner::admit_execution_request;
pub use planner::plan_codegen;
pub use planner::plan_compilation;
pub use profile::EXPERIMENT_PROFILE_SCHEMA;
pub use profile::NIX_CLOSURE_OBSERVATION_LOCK_FORMAT;
pub use profile::OBSERVATION_KERNEL_BUILD_IDENTITY_PREFIX;
pub use profile::ONIX_KERNEL_BUILD_IDENTITY_PREFIX;
pub use profile::OPAM_LOCK_FORMAT;
pub use profile::ProfileValidation;
pub use profile::REQUIRED_NON_CLAIMS;
pub use profile::validate_profile;
pub use receipt::EXPERIMENT_RECEIPT_SCHEMA;
pub use receipt::ExperimentReceipt;
pub use receipt::ExperimentReceiptInput;
pub use receipt::ExperimentReceiptResult;
pub use receipt::ExperimentStageStatus;
pub use receipt::OutputClassReceipt;
pub use receipt::build_experiment_receipt;
pub use target::ResolvedKernelTargetFacts;
pub use target::TargetAdmission;
pub use target::admit_kernel_target;

#[cfg(test)]
mod tests;
