#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure, deterministic planning and admission core for Mantle WebAssembly
//! component materialization. Registry access, credential resolution, byte I/O,
//! tool execution, sandboxing, store persistence, and report rendering belong
//! to std-facing imperative adapters.

extern crate alloc;

const ADJACENT_WINDOW_LENGTH: usize = 2;

#[cfg(test)]
extern crate std;

mod admission;
mod blocker;
mod composition;
mod digest;
mod generated;
mod manifest;
mod model;
mod report;
mod virtualization;

pub use admission::AOT_ADMISSION_SCHEMA;
pub use admission::AOT_TRUST_CLASS;
pub use admission::AotAdmission;
pub use admission::AotAdmissionRequest;
pub use admission::AotAdmissionResult;
pub use admission::AotReceipt;
pub use admission::BuildValidationBinding;
pub use admission::OctetValidationBinding;
pub use admission::PORTABLE_ADMISSION_SCHEMA;
pub use admission::PortableAdmission;
pub use admission::PortableAdmissionRequest;
pub use admission::PortableAdmissionResult;
pub use admission::TransformAdmission;
pub use admission::TransformAdmissionRequest;
pub use admission::TransformAdmissionResult;
pub use admission::TransformImportFact;
pub use admission::ValidationDecision;
pub use admission::WIZER_ADMISSION_SCHEMA;
pub use admission::admit_aot;
pub use admission::admit_transform;
pub use admission::bind_portable_admission;
pub use blocker::ComponentBlocker;
pub use composition::COMPOSITION_PLAN_SCHEMA;
pub use composition::CompositionPlan;
pub use composition::CompositionValidation;
pub use composition::validate_composition;
pub use digest::BLAKE3_HEX_LENGTH;
pub use digest::Blake3Identity;
pub use digest::DigestError;
pub use digest::DigestRole;
pub use digest::OCI_SHA256_PREFIX;
pub use digest::OciSha256Digest;
pub use digest::RoleDigest;
pub use digest::SHA256_HEX_LENGTH;
pub use digest::parse_role_digest;
pub use generated::GENERATED_INPUT_OWNER;
pub use generated::GENERATED_INPUT_PLAN_SCHEMA;
pub use generated::GENERATED_INPUT_RECEIPT_SCHEMA;
pub use generated::GeneratedFreshnessValidation;
pub use generated::GeneratedInputCandidate;
pub use generated::GeneratedInputOwner;
pub use generated::GeneratedInputPlan;
pub use generated::GeneratedInputPlanResult;
pub use generated::GeneratedInputReceipt;
pub use generated::ObservedGeneratedInput;
pub use generated::finalize_generated_inputs;
pub use generated::verify_generated_freshness;
pub use manifest::LockValidation;
pub use manifest::LockValidationRequest;
pub use manifest::ManifestValidation;
pub use manifest::PackageFetchPlan;
pub use manifest::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
pub use manifest::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;
pub use manifest::SOURCE_ACQUISITION_PLAN_SCHEMA;
pub use manifest::SourceAcquisitionPlan;
pub use manifest::SourcePlanResult;
pub use manifest::WKG_LOCK_VERSION;
pub use manifest::cohort_identity;
pub use manifest::plan_source_acquisition;
pub use manifest::registry_config_identity;
pub use manifest::validate_lock;
pub use manifest::validate_manifest;
pub use model::AotConfig;
pub use model::AotMode;
pub use model::COMPONENT_MANIFEST_SCHEMA;
pub use model::ComponentManifest;
pub use model::Composition;
pub use model::CompositionEdge;
pub use model::CompositionNode;
pub use model::LocalPackageOverride;
pub use model::LockFacts;
pub use model::OciProtocol;
pub use model::OutputClass;
pub use model::OutputDeclaration;
pub use model::PackageKind;
pub use model::PackageMaterialization;
pub use model::PackageRequirement;
pub use model::PackageResolution;
pub use model::RegistryMapping;
pub use model::RequiredImport;
pub use model::RustImplementation;
pub use model::RustProfile;
pub use model::StoreObject;
pub use model::ToolCohort;
pub use model::ToolIdentity;
pub use model::ValidationProfiles;
pub use model::VirtualizationConfig;
pub use model::VirtualizationMode;
pub use model::VirtualizationRule;
pub use model::WasiSubsystem;
pub use model::WitSelection;
pub use model::WizerConfig;
pub use model::WizerMode;
pub use model::WkgLock;
pub use model::WkgLockPackage;
pub use model::WkgLockedVersion;
pub use report::BoundedComponentClaim;
pub use report::COMPONENT_BUILD_REPORT_SCHEMA;
pub use report::ComponentBuildReport;
pub use report::ComponentBuildReportResult;
pub use report::ComponentStageKind;
pub use report::ComponentStageStatus;
pub use report::StageReportInput;
pub use report::StageReportNode;
pub use report::build_component_report;
pub use report::stage_report_identity;
pub use virtualization::RemainingImportValidation;
pub use virtualization::VIRTUALIZATION_PLAN_SCHEMA;
pub use virtualization::VirtualizationAction;
pub use virtualization::VirtualizationEntry;
pub use virtualization::VirtualizationPlan;
pub use virtualization::VirtualizationPlanResult;
pub use virtualization::WASI_SUBSYSTEM_COUNT;
pub use virtualization::plan_virtualization;
pub use virtualization::validate_remaining_imports;

#[cfg(test)]
mod tests;
