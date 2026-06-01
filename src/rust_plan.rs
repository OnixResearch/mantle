use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::errors::RunError;

const RECEIPT_SCHEMA_VERSION: u32 = 1;
const RUST_UNIT_EXECUTION_RECEIPT_FILE: &str = ".mantle-rust-unit-execution.json";
const DEFAULT_CARGO_PROFILE: &str = "dev";
const DEFAULT_RUST_EDITION: &str = "2015";
const PATH_SOURCE_DIGEST_ALGORITHM: &str = "blake3-tree-v1";
const PATH_SOURCE_DIGEST_SKIP_ANYWHERE: &[&str] = &[".agent", ".git", ".jj", ".pi", "target"];
const PATH_SOURCE_DIGEST_SKIP_AT_ROOT: &[&str] = &["cairn"];
const RUST_TOPOLOGY_TOOL_PATH_ENV: &str = "PATH";
const SNIX_BUILD_SANDBOX_SHELL_ENV: &str = "SNIX_BUILD_SANDBOX_SHELL";
const RUST_TOPOLOGY_COMPILE_ENV_ALLOWLIST: &[&str] = &[SNIX_BUILD_SANDBOX_SHELL_ENV];
const BUILD_SCRIPT_OUT_DIR_ENV: &str = "OUT_DIR";
const BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV: &str = "CARGO_MANIFEST_DIR";
const BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV: &str = "CARGO_MANIFEST_LINKS";
const BUILD_SCRIPT_CARGO_PKG_NAME_ENV: &str = "CARGO_PKG_NAME";
const CARGO_PKG_AUTHORS_ENV: &str = "CARGO_PKG_AUTHORS";
const CARGO_PKG_DESCRIPTION_ENV: &str = "CARGO_PKG_DESCRIPTION";
const CARGO_PKG_HOMEPAGE_ENV: &str = "CARGO_PKG_HOMEPAGE";
const CARGO_PKG_LICENSE_ENV: &str = "CARGO_PKG_LICENSE";
const CARGO_PKG_LICENSE_FILE_ENV: &str = "CARGO_PKG_LICENSE_FILE";
const CARGO_PKG_README_ENV: &str = "CARGO_PKG_README";
const CARGO_PKG_REPOSITORY_ENV: &str = "CARGO_PKG_REPOSITORY";
const CARGO_PKG_RUST_VERSION_ENV: &str = "CARGO_PKG_RUST_VERSION";
const CARGO_PKG_VERSION_ENV: &str = "CARGO_PKG_VERSION";
const CARGO_PKG_VERSION_MAJOR_ENV: &str = "CARGO_PKG_VERSION_MAJOR";
const CARGO_PKG_VERSION_MINOR_ENV: &str = "CARGO_PKG_VERSION_MINOR";
const CARGO_PKG_VERSION_PATCH_ENV: &str = "CARGO_PKG_VERSION_PATCH";
const CARGO_PKG_VERSION_PRE_ENV: &str = "CARGO_PKG_VERSION_PRE";
const BUILD_SCRIPT_RUSTC_ENV: &str = "RUSTC";
const BUILD_SCRIPT_HOST_ENV: &str = "HOST";
const BUILD_SCRIPT_TARGET_ENV: &str = "TARGET";
const BUILD_SCRIPT_PROFILE_ENV: &str = "PROFILE";
const RUST_UNIT_EXECUTION_KIND_ENV: &str = "MANTLE_RUST_UNIT_EXECUTION_KIND";
const HOST_DEPENDENCY_EXECUTION_KIND: &str = "host-dependency";
const HOST_DEPENDENCY_CRATE_KIND: &str = "host-lib";
const HOST_DEPENDENCY_MODE: &str = "host-build";
const HOST_DEPENDENCY_UNIT_ID_SUFFIX: &str = ":host-dependency";
const BUILD_SCRIPT_OPT_LEVEL_ENV: &str = "OPT_LEVEL";
const BUILD_SCRIPT_DEBUG_ENV: &str = "DEBUG";
const BUILD_SCRIPT_NUM_JOBS_ENV: &str = "NUM_JOBS";
const CARGO_PROFILE_RELEASE: &str = "release";
const CARGO_PROFILE_BENCH: &str = "bench";
const CARGO_OPT_LEVEL_DEBUG: &str = "0";
const CARGO_OPT_LEVEL_RELEASE: &str = "3";
const CARGO_DEBUG_TRUE: &str = "true";
const CARGO_DEBUG_FALSE: &str = "false";
const MANTLE_DETERMINISTIC_NUM_JOBS: &str = "1";
const CARGO_CFG_TARGET_ABI_ENV: &str = "CARGO_CFG_TARGET_ABI";
const CARGO_CFG_TARGET_ARCH_ENV: &str = "CARGO_CFG_TARGET_ARCH";
const CARGO_CFG_TARGET_ENDIAN_ENV: &str = "CARGO_CFG_TARGET_ENDIAN";
const CARGO_CFG_TARGET_ENV_ENV: &str = "CARGO_CFG_TARGET_ENV";
const CARGO_CFG_TARGET_FAMILY_ENV: &str = "CARGO_CFG_TARGET_FAMILY";
const CARGO_CFG_TARGET_FEATURE_ENV: &str = "CARGO_CFG_TARGET_FEATURE";
const CARGO_CFG_TARGET_OS_ENV: &str = "CARGO_CFG_TARGET_OS";
const CARGO_CFG_TARGET_POINTER_WIDTH_ENV: &str = "CARGO_CFG_TARGET_POINTER_WIDTH";
const CARGO_CFG_TARGET_VENDOR_ENV: &str = "CARGO_CFG_TARGET_VENDOR";
const CARGO_CFG_UNIX_ENV: &str = "CARGO_CFG_UNIX";
const CARGO_CFG_WINDOWS_ENV: &str = "CARGO_CFG_WINDOWS";
const BUILD_SCRIPT_DEP_ENV_PREFIX: &str = "DEP_";
const BUILD_SCRIPT_TARGET_NAME: &str = "build-script-build";
const BUILD_SCRIPT_MAIN_TARGET_NAME: &str = "build-script-main";
const BUILD_SCRIPT_BUILD_CRATE_NAME: &str = "build_script_build";
const BUILD_SCRIPT_MAIN_CRATE_NAME: &str = "build_script_main";
const UNIQUE_CANDIDATE_COUNT: usize = 1;
const PACKAGE_LINKS_ENV: &str = "MANTLE_PACKAGE_LINKS";
const RUSTC_CODEGEN_OPTION_FLAG: &str = "-C";
const RUSTC_CFG_FLAG: &str = "--cfg";
const RUSTC_EXTERN_FLAG: &str = "--extern";
const RUSTC_EXTERN_ARG_PAIR_WIDTH: usize = 2;
const NATIVE_FEATURE_RESOLUTION_MAX_STEPS: usize = 4096;
const NATIVE_LOCK_REACHABILITY_MAX_STEPS: usize = 8192;
const NATIVE_PACKAGE_MANIFEST_REACHABILITY_MAX_STEPS: usize = 4096;
const RUSTC_LINK_SEARCH_FLAG: &str = "-L";
const RUSTC_DEPENDENCY_SEARCH_PREFIX: &str = "dependency=";
const RUSTC_CAP_LINTS_FLAG: &str = "--cap-lints";
const RUSTC_CAP_LINTS_ALLOW: &str = "allow";
const RUSTC_PROC_MACRO_EXTERN: &str = "proc_macro";
const RUSTC_LINK_LIB_KIND_STATIC: &str = "static";
const RUSTC_LINK_LIB_KIND_DYLIB: &str = "dylib";
const RUSTC_LINK_LIB_KIND_FRAMEWORK: &str = "framework";
const RUSTC_LINK_LIB_MODIFIER_ENABLE: char = '+';
const RUSTC_LINK_LIB_MODIFIER_DISABLE: char = '-';
const RUSTC_LINK_SELF_CONTAINED_OPTION: &str = "link-self-contained";
const RUSTC_EXTERNAL_LINKER_MODE_ARG: &str = "link-self-contained=no";
const RUSTC_METADATA_ARG_PREFIX: &str = "metadata=";
const RUSTC_METADATA_HEX_CHARS: usize = 16;
const BLAKE3_HEX_CHARS: usize = 64;
const REDACTED_DIAGNOSTIC_MAX_LINES: usize = 12;
const REDACTED_DIAGNOSTIC_TEMP_PATH: &str = "<redacted-temp-path>";
const TEMP_PATH_PREFIXES: &[&str] = &["/tmp/", "/var/tmp/"];
const CARGO_REGISTRY_SOURCE_PREFIX: &str = "registry+";
const CARGO_GIT_SOURCE_PREFIX: &str = "git+";
const REGISTRY_SOURCE_DIGEST_ALGORITHM: &str = "cargo-checksum-sha256";
const GIT_SOURCE_DIGEST_ALGORITHM: &str = "git-revision";

#[derive(Debug, Clone)]
pub(crate) struct RustPlanOptions {
    pub(crate) root: PathBuf,
    pub(crate) cargo: PathBuf,
    pub(crate) rustc: PathBuf,
    pub(crate) targets: Vec<String>,
    pub(crate) profile: String,
    pub(crate) features: Vec<String>,
    pub(crate) all_features: bool,
    pub(crate) no_default_features: bool,
    pub(crate) no_cargo_oracle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanCargoModeSummary {
    pub(crate) no_cargo_oracle: bool,
    pub(crate) compatibility_class: String,
    pub(crate) blockers: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanReceipt {
    pub(crate) schema_version: u32,
    pub(crate) cargo_mode: RustPlanCargoModeSummary,
    pub(crate) workspace_root: String,
    pub(crate) cargo_version: String,
    pub(crate) rustc_version_verbose: String,
    pub(crate) lockfile: LockfileIdentity,
    pub(crate) invocation: RustPlanInvocation,
    pub(crate) package_count: usize,
    pub(crate) packages: Vec<PackageSummary>,
    pub(crate) source_closure: SourceClosureSummary,
    pub(crate) native_registry_source_planning: NativeRegistrySourcePlanningSummary,
    pub(crate) native_git_source_planning: NativeGitSourcePlanningSummary,
    pub(crate) native_package_target_planning: NativePackageTargetPlanningSummary,
    pub(crate) native_unit_graph_planning: NativeUnitGraphPlanningSummary,
    pub(crate) native_host_unit_graph_planning: NativeHostUnitGraphPlanningSummary,
    pub(crate) unit_graph: UnitGraphSummary,
    pub(crate) unit_derivation_graph: UnitDerivationGraphSummary,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LockfileIdentity {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanInvocation {
    pub(crate) profile: String,
    pub(crate) targets: Vec<String>,
    pub(crate) features: Vec<String>,
    pub(crate) all_features: bool,
    pub(crate) no_default_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PackageSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceClosureSummary {
    pub(crate) source_count: usize,
    pub(crate) ready: bool,
    pub(crate) digest_blake3: String,
    pub(crate) sources: Vec<SourceInputSummary>,
    pub(crate) blockers: Vec<SourceClosureBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceInputSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) kind: SourceKind,
    pub(crate) source: Option<String>,
    pub(crate) manifest_path: String,
    pub(crate) lockfile_identity: Option<LockPackageIdentity>,
    pub(crate) resolved_revision: Option<String>,
    pub(crate) source_digest: SourceDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceKind {
    Path,
    Registry,
    Git,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LockPackageIdentity {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceDigest {
    pub(crate) algorithm: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceClosureBlocker {
    pub(crate) package_id: String,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeRegistrySourcePlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) lockfile_digest_blake3: String,
    pub(crate) digest_blake3: String,
    pub(crate) sources: Vec<NativeRegistrySourceSummary>,
    pub(crate) blockers: Vec<NativeRegistrySourceBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRegistrySourceSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) source_class: String,
    pub(crate) checksum: String,
    pub(crate) vendor_root: String,
    pub(crate) manifest_path: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) lockfile_identity: LockPackageIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRegistrySourceBlocker {
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeGitSourcePlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) lockfile_digest_blake3: String,
    pub(crate) source_closure_digest_blake3: String,
    pub(crate) digest_blake3: String,
    pub(crate) sources: Vec<NativeGitSourceSummary>,
    pub(crate) blockers: Vec<NativeGitSourceBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeGitSourceSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) source_class: String,
    pub(crate) resolved_revision: String,
    pub(crate) source_root: String,
    pub(crate) manifest_path: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) lockfile_identity: LockPackageIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeGitSourceBlocker {
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativePackageTargetPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_oracle_identity: String,
    pub(crate) digest_blake3: String,
    pub(crate) packages: Vec<NativePackagePlanningSummary>,
    pub(crate) blockers: Vec<NativePackagePlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativePackagePlanningSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) manifest_path: String,
    pub(crate) links: Option<String>,
    pub(crate) cargo_package_env: BTreeMap<String, String>,
    pub(crate) selected_features: Vec<String>,
    pub(crate) targets: Vec<NativeTargetPlanningSummary>,
    pub(crate) path_dependencies: Vec<NativePathDependencySummary>,
    pub(crate) build_dependencies: Vec<NativePathDependencySummary>,
    pub(crate) dev_dependencies: Vec<NativePathDependencySummary>,
    pub(crate) target_cfg_dependencies: Vec<NativeTargetCfgDependencySummary>,
    pub(crate) workspace_dependencies: Vec<NativeWorkspaceDependencySummary>,
    pub(crate) source_digest: SourceDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeTargetPlanningSummary {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
    pub(crate) edition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativePathDependencySummary {
    pub(crate) name: String,
    pub(crate) manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeTargetCfgDependencySummary {
    pub(crate) cfg: String,
    pub(crate) active_target: String,
    pub(crate) decision: String,
    pub(crate) name: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) blocker_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeWorkspaceDependencySummary {
    pub(crate) workspace_root: String,
    pub(crate) member_package_id: String,
    pub(crate) dependency_key: String,
    pub(crate) inherited_package_name: String,
    pub(crate) inherited_features: Vec<String>,
    pub(crate) inherited_default_features: Option<bool>,
    pub(crate) decision: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) blocker_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativePackagePlanningBlocker {
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeUnitGraphPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_unit_graph_oracle_digest: String,
    pub(crate) native_unit_graph_digest: String,
    pub(crate) oracle_comparison_digest: String,
    pub(crate) units: Vec<NativeRustUnitSummary>,
    pub(crate) blockers: Vec<NativeUnitGraphPlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRustUnitSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) package_name: String,
    pub(crate) package_links: Option<String>,
    pub(crate) package_root: String,
    pub(crate) cargo_package_env: BTreeMap<String, String>,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
    pub(crate) edition: String,
    pub(crate) selected_features: Vec<String>,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) dependency_artifacts: Vec<RustDependencyArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeUnitGraphPlanningBlocker {
    pub(crate) unit_id: Option<String>,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeHostUnitGraphPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_host_oracle_digest: String,
    pub(crate) native_host_graph_digest: String,
    pub(crate) oracle_comparison_digest: String,
    pub(crate) host_units: Vec<NativeHostUnitSummary>,
    pub(crate) target_consumers: Vec<NativeHostTargetConsumerSummary>,
    pub(crate) blockers: Vec<NativeHostUnitGraphPlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostUnitSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) package_name: String,
    pub(crate) package_links: Option<String>,
    pub(crate) package_root: String,
    pub(crate) cargo_package_env: BTreeMap<String, String>,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
    pub(crate) edition: String,
    pub(crate) selected_features: Vec<String>,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) artifact: RustHostArtifact,
    pub(crate) dependency_artifacts: Vec<RustDependencyArtifact>,
    pub(crate) consumed_host_artifacts: Vec<RustHostArtifact>,
    pub(crate) metadata_dependencies: Vec<BuildScriptMetadataDependency>,
    pub(crate) generated_metadata: Option<BuildScriptMetadataSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostTargetConsumerSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) consumed_host_artifacts: Vec<RustHostArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostUnitGraphPlanningBlocker {
    pub(crate) unit_id: Option<String>,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitGraphSummary {
    pub(crate) unit_count: usize,
    pub(crate) root_count: usize,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitDerivationGraphSummary {
    pub(crate) derivation_count: usize,
    pub(crate) host_unit_count: usize,
    pub(crate) host_artifact_count: usize,
    pub(crate) ready: bool,
    pub(crate) digest_blake3: String,
    pub(crate) derivations: Vec<RustUnitDerivationSummary>,
    pub(crate) blockers: Vec<UnitDerivationBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitDerivationSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) execution_kind: String,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) dependency_artifacts: Vec<RustDependencyArtifact>,
    pub(crate) consumed_host_artifacts: Vec<RustHostArtifact>,
    pub(crate) metadata_dependencies: Vec<BuildScriptMetadataDependency>,
    pub(crate) generated_metadata: Option<BuildScriptMetadataSummary>,
    pub(crate) derivation: ReviewableRustDerivation,
    pub(crate) rustc_args_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RustDependencyArtifact {
    pub(crate) package_id: String,
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) producer_unit_id: Option<String>,
    pub(crate) artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RustHostArtifact {
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) producer_unit_id: Option<String>,
    pub(crate) artifact: String,
    pub(crate) metadata_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BuildScriptMetadataDependency {
    pub(crate) package_id: String,
    pub(crate) links: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) producer_unit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BuildScriptMetadataSummary {
    pub(crate) out_dir: String,
    pub(crate) rustc_cfg: Vec<String>,
    pub(crate) rustc_env: BTreeMap<String, String>,
    pub(crate) rustc_link_lib: Vec<String>,
    pub(crate) rustc_link_search: Vec<String>,
    pub(crate) rerun_if_changed: Vec<String>,
    pub(crate) metadata: BTreeMap<String, String>,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReviewableRustDerivation {
    pub(crate) name: String,
    pub(crate) builder: String,
    pub(crate) system: String,
    pub(crate) args: Vec<String>,
    pub(crate) outputs: Vec<String>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) inputs: Vec<String>,
    pub(crate) addressing_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitDerivationBlocker {
    pub(crate) unit_id: String,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustUnitExecutionOptions {
    pub(crate) rustc: PathBuf,
    pub(crate) output_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) execution_status: String,
    pub(crate) rebuild_reason: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) toolchain: RustToolchainIdentity,
    pub(crate) rustc_args_digest_blake3: String,
    pub(crate) environment_digest_blake3: String,
    pub(crate) declared_outputs: Vec<String>,
    pub(crate) dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitDependencyChainExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitTargetTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitHostArtifactTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustDevDependencyTestTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) package_id: Option<String>,
    pub(crate) test_target: Option<String>,
    pub(crate) dev_dependency_packages: Vec<String>,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustWorkspaceDependencyTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) workspace_root: Option<String>,
    pub(crate) member_package_id: Option<String>,
    pub(crate) inherited_dependency_packages: Vec<String>,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPatchSourceTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) consumer_package_id: Option<String>,
    pub(crate) patch_source_packages: Vec<String>,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BuildScriptMetadataRunReceipt {
    pub(crate) schema_version: u32,
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) execution_status: String,
    pub(crate) out_dir: String,
    pub(crate) rustc_cfg: Vec<String>,
    pub(crate) rustc_env: BTreeMap<String, String>,
    pub(crate) rustc_link_lib: Vec<String>,
    pub(crate) rustc_link_search: Vec<String>,
    pub(crate) rerun_if_changed: Vec<String>,
    pub(crate) metadata: BTreeMap<String, String>,
    pub(crate) out_dir_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) stdout_digest_blake3: String,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustToolchainIdentity {
    pub(crate) tool: String,
    pub(crate) version_verbose: String,
    pub(crate) version_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustExecutionArtifactDigest {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitExecutionBlocker {
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) unit_execution: RustUnitExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanDependencyChainExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) dependency_chain_execution: RustUnitDependencyChainExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanTargetTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) target_topology_execution: RustUnitTargetTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanHostArtifactTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) host_artifact_topology_execution: RustUnitHostArtifactTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) topology_execution: RustUnitTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanDevDependencyTestTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) native_rust_dev_dependency_test_topology_execution: RustDevDependencyTestTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanWorkspaceDependencyTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) native_registry_workspace_dependency_topology_execution: RustWorkspaceDependencyTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanPatchSourceTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) native_registry_patch_source_topology_execution: RustPatchSourceTopologyExecutionReceipt,
}

#[derive(Debug, Clone)]
struct CargoOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status_code: i32,
}

trait CargoOracle {
    fn run_cargo(&self, root: &Path, cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError>;
    fn run_tool_version(&self, tool: &Path, args: &[&str]) -> Result<CargoOutput, RunError>;
}

struct ProcessCargoOracle;

impl CargoOracle for ProcessCargoOracle {
    fn run_cargo(&self, root: &Path, cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError> {
        let output = Command::new(cargo)
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|err| RunError::Internal(format!("failed to run {}: {err}", cargo.display())))?;
        Ok(CargoOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status_code: output.status.code().unwrap_or(1),
        })
    }

    fn run_tool_version(&self, tool: &Path, args: &[&str]) -> Result<CargoOutput, RunError> {
        let output = Command::new(tool)
            .args(args)
            .output()
            .map_err(|err| RunError::Internal(format!("failed to run {}: {err}", tool.display())))?;
        Ok(CargoOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status_code: output.status.code().unwrap_or(1),
        })
    }
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_root: String,
    workspace_members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
    #[serde(default)]
    targets: Vec<CargoTarget>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    src_path: String,
}

#[derive(Debug, Clone, Default)]
struct NativeManifestInheritedString {
    value: Option<String>,
    workspace: bool,
}

impl<'de> Deserialize<'de> for NativeManifestInheritedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let value = toml::Value::deserialize(deserializer)?;
        if let Some(text) = value.as_str() {
            return Ok(Self {
                value: Some(text.to_string()),
                workspace: false,
            });
        }
        if let Some(table) = value.as_table() {
            let workspace = table.get("workspace").and_then(toml::Value::as_bool).unwrap_or(false);
            if workspace && table.len() == 1 {
                return Ok(Self {
                    value: None,
                    workspace: true,
                });
            }
        }
        Err(serde::de::Error::custom("expected string or { workspace = true }"))
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeWorkspacePackage {
    version: Option<String>,
    edition: Option<String>,
    #[serde(default)]
    authors: Option<toml::Value>,
    #[serde(default)]
    description: Option<toml::Value>,
    #[serde(default)]
    homepage: Option<toml::Value>,
    #[serde(default)]
    license: Option<toml::Value>,
    #[serde(rename = "license-file", default)]
    license_file: Option<toml::Value>,
    #[serde(default)]
    readme: Option<toml::Value>,
    #[serde(default)]
    repository: Option<toml::Value>,
    #[serde(rename = "rust-version", default)]
    rust_version: Option<toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct NativeManifestPackage {
    name: String,
    version: NativeManifestInheritedString,
    #[serde(default)]
    authors: Option<toml::Value>,
    #[serde(default)]
    edition: NativeManifestInheritedString,
    #[serde(default)]
    build: Option<toml::Value>,
    #[serde(default)]
    description: Option<toml::Value>,
    #[serde(default)]
    homepage: Option<toml::Value>,
    #[serde(default)]
    license: Option<toml::Value>,
    #[serde(rename = "license-file", default)]
    license_file: Option<toml::Value>,
    links: Option<String>,
    #[serde(default)]
    readme: Option<toml::Value>,
    #[serde(default)]
    repository: Option<toml::Value>,
    #[serde(rename = "rust-version", default)]
    rust_version: Option<toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestLib {
    name: Option<String>,
    path: Option<String>,
    #[serde(rename = "proc-macro", alias = "proc_macro", default)]
    proc_macro: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestBin {
    name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestTest {
    name: Option<String>,
    path: Option<String>,
    #[serde(default)]
    harness: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestWorkspace {
    #[serde(default)]
    members: Vec<String>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    package: NativeWorkspacePackage,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestTarget {
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifest {
    package: Option<NativeManifestPackage>,
    workspace: Option<NativeManifestWorkspace>,
    #[serde(default)]
    build: Option<toml::Value>,
    lib: Option<NativeManifestLib>,
    #[serde(default)]
    bin: Vec<NativeManifestBin>,
    #[serde(default)]
    test: Vec<NativeManifestTest>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(rename = "dev-dependencies", default)]
    dev_dependencies: BTreeMap<String, toml::Value>,
    #[serde(rename = "build-dependencies", default)]
    build_dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    patch: BTreeMap<String, BTreeMap<String, toml::Value>>,
    #[serde(default)]
    target: BTreeMap<String, NativeManifestTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NativeLockfileFacts {
    packages: Vec<NativeLockfilePackageFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct NativeLockfilePackageFact {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    revision: Option<String>,
    dependencies: Vec<NativeLockDependencyFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct NativeLockDependencyFact {
    raw: String,
    name: String,
    version: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoLockfile {
    #[serde(default)]
    package: Vec<CargoLockPackageRecord>,
}

#[derive(Debug, Deserialize)]
struct CargoLockPackageRecord {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeTextInput {
    path: String,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeManifestLockInputTexts {
    manifests: Vec<NativeTextInput>,
    lockfile: NativeTextInput,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeManifestLockUnsupportedBlocker {
    path: String,
    class: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeFeatureResolutionRequest {
    feature_defs: BTreeMap<String, Vec<String>>,
    optional_dependencies: BTreeSet<String>,
    explicit_features: Vec<String>,
    all_features: bool,
    no_default_features: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeFeatureResolution {
    selected_features: Vec<String>,
    activated_optional_dependencies: Vec<String>,
    dependency_feature_edges: Vec<NativeDependencyFeatureEdge>,
    blockers: Vec<NativeFeatureResolutionBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeFeatureRoleResolutionRequest {
    normal: NativeFeatureResolutionRequest,
    build: NativeFeatureResolutionRequest,
    host: NativeFeatureResolutionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeFeatureRoleResolution {
    normal: NativeFeatureResolution,
    build: NativeFeatureResolution,
    host: NativeFeatureResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeDependencyFeatureEdge {
    dependency: String,
    feature: String,
    weak: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeFeatureResolutionBlocker {
    class: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeFeatureEntry {
    LocalFeature(String),
    OptionalDependency { dependency: String, expose_feature: bool },
    DependencyFeature(NativeDependencyFeatureEdge),
}

pub(crate) fn capture_rust_plan(options: &RustPlanOptions) -> Result<RustPlanReceipt, RunError> {
    if options.no_cargo_oracle {
        return capture_rust_plan_without_cargo(options);
    }
    capture_rust_plan_with_oracle(options, &ProcessCargoOracle)
}

fn rust_plan_cargo_mode(no_cargo_oracle: bool, blockers: Vec<String>) -> RustPlanCargoModeSummary {
    let compatibility_class = if no_cargo_oracle {
        "cargo-free-native-path-topology-v1"
    } else {
        "cargo-oracle-assisted-v1"
    };
    let non_claims = if no_cargo_oracle {
        vec![
            "bounded-path-workspace-only".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "declared-vendor-and-captured-git-only".to_string(),
            "not-network-or-ambient-cargo-source-resolution".to_string(),
        ]
    } else {
        vec!["cargo-used-for-oracle-metadata-and-unit-graph".to_string()]
    };
    RustPlanCargoModeSummary {
        no_cargo_oracle,
        compatibility_class: compatibility_class.to_string(),
        blockers,
        non_claims,
    }
}

fn capture_rust_plan_with_oracle(
    options: &RustPlanOptions,
    oracle: &impl CargoOracle,
) -> Result<RustPlanReceipt, RunError> {
    validate_options(options)?;

    let cargo_version = run_checked_text(oracle.run_tool_version(&options.cargo, &["--version"]), "cargo --version")?;
    let rustc_version_verbose = run_checked_text(oracle.run_tool_version(&options.rustc, &["-vV"]), "rustc -vV")?;
    let metadata_output =
        run_checked_bytes(oracle.run_cargo(&options.root, &options.cargo, &metadata_args()), "cargo metadata")?;
    let metadata: CargoMetadata = serde_json::from_slice(&metadata_output)
        .map_err(|err| RunError::Internal(format!("cargo metadata did not emit valid JSON: {err}")))?;
    let unit_graph_output = run_checked_bytes(
        oracle.run_cargo(&options.root, &options.cargo, &unit_graph_args(options)),
        "cargo build --unit-graph",
    )?;
    let unit_graph_value: Value = serde_json::from_slice(&unit_graph_output)
        .map_err(|err| RunError::Internal(format!("cargo build --unit-graph did not emit valid JSON: {err}")))?;

    let lock_packages = parse_lockfile_packages(&options.root)?;
    let source_closure = summarize_source_closure(&metadata.packages, &lock_packages)?;
    let lockfile = lockfile_identity(&options.root)?;
    let native_registry_source_planning =
        summarize_native_registry_source_planning(&options.root, &metadata.packages, &lock_packages, &lockfile)?;
    let native_git_source_planning =
        summarize_native_git_source_planning(&metadata.packages, &lock_packages, &lockfile, &source_closure)?;
    let native_package_target_planning = summarize_native_package_target_planning(
        &options.root,
        options,
        &unit_graph_value,
        &metadata.packages,
        &metadata.workspace_members,
        &source_closure,
        &native_registry_source_planning,
        &native_git_source_planning,
    )?;
    let native_unit_graph_planning = summarize_native_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_package_target_planning,
        options,
    )?;
    let native_host_unit_graph_planning = summarize_native_host_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_registry_source_planning,
        &native_package_target_planning,
        &native_unit_graph_planning,
        options,
    )?;
    let unit_derivation_graph = summarize_unit_derivation_graph_with_native(
        &unit_graph_value,
        &source_closure,
        options,
        Some(&native_unit_graph_planning),
        Some(&native_host_unit_graph_planning),
    )?;
    let packages = summarize_packages(metadata.packages.clone(), &metadata.workspace_members);

    let mut receipt = RustPlanReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        cargo_mode: rust_plan_cargo_mode(false, Vec::new()),
        workspace_root: normalize_path_string(Path::new(&metadata.workspace_root)),
        cargo_version: cargo_version.trim().to_string(),
        rustc_version_verbose: rustc_version_verbose.trim().to_string(),
        lockfile,
        invocation: RustPlanInvocation {
            profile: options.profile.clone(),
            targets: sorted_strings(options.targets.clone()),
            features: sorted_strings(options.features.clone()),
            all_features: options.all_features,
            no_default_features: options.no_default_features,
        },
        package_count: metadata.packages.len(),
        packages,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        native_package_target_planning,
        native_unit_graph_planning,
        native_host_unit_graph_planning,
        unit_graph: summarize_unit_graph(unit_graph_value)?,
        unit_derivation_graph,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt)?;
    Ok(receipt)
}

fn capture_rust_plan_without_cargo(options: &RustPlanOptions) -> Result<RustPlanReceipt, RunError> {
    validate_options(options)?;

    let rustc_version_verbose =
        run_checked_text(ProcessCargoOracle.run_tool_version(&options.rustc, &["-vV"]), "rustc -vV")?;
    let lockfile_facts = parse_lockfile_facts(&options.root)?;
    let lock_packages = lock_packages_from_facts(&lockfile_facts);
    let native_path_packages = native_path_cargo_packages(&options.root, options)?;
    let workspace_members = native_path_packages.iter().map(|package| package.id.clone()).collect::<Vec<_>>();
    let lock_source_packages = native_reachable_lock_source_cargo_packages(
        &options.root,
        &lockfile_facts,
        &lock_packages,
        &native_path_packages,
    );
    let mut registry_input_packages = native_path_packages.clone();
    registry_input_packages.extend(lock_source_packages.clone());
    registry_input_packages.sort_by(|left, right| left.id.cmp(&right.id));
    registry_input_packages.dedup_by(|left, right| left.id == right.id);
    let lockfile = lockfile_identity(&options.root)?;
    let native_registry_source_planning =
        summarize_native_registry_source_planning(&options.root, &registry_input_packages, &lock_packages, &lockfile)?;
    let mut native_cargo_packages = native_path_packages;
    native_cargo_packages.extend(native_registry_cargo_packages_from_sources(&native_registry_source_planning.sources));
    native_cargo_packages.extend(native_git_cargo_packages_from_lock(&lock_source_packages));
    native_cargo_packages.sort_by(|left, right| left.id.cmp(&right.id));
    native_cargo_packages.dedup_by(|left, right| left.id == right.id);
    let source_closure = summarize_source_closure(&native_cargo_packages, &lock_packages)?;
    let native_git_source_planning =
        summarize_native_git_source_planning(&native_cargo_packages, &lock_packages, &lockfile, &source_closure)?;
    let unit_graph_value = empty_native_unit_graph_value();
    let native_package_target_planning = summarize_native_package_target_planning(
        &options.root,
        options,
        &unit_graph_value,
        &native_cargo_packages,
        &workspace_members,
        &source_closure,
        &native_registry_source_planning,
        &native_git_source_planning,
    )?;
    let native_unit_graph_planning = summarize_native_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_package_target_planning,
        options,
    )?;
    let native_host_unit_graph_planning = summarize_native_host_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_registry_source_planning,
        &native_package_target_planning,
        &native_unit_graph_planning,
        options,
    )?;
    let unit_derivation_graph = summarize_unit_derivation_graph_with_native(
        &unit_graph_value,
        &source_closure,
        options,
        Some(&native_unit_graph_planning),
        Some(&native_host_unit_graph_planning),
    )?;
    let packages = summarize_packages(native_cargo_packages.clone(), &workspace_members);
    let mut blockers = Vec::new();
    if !native_package_target_planning.ready {
        blockers.push("native-package-target-planning-blocked".to_string());
    }
    if !native_unit_graph_planning.ready {
        blockers.push("native-unit-graph-planning-blocked".to_string());
    }
    if !native_host_unit_graph_planning.ready {
        blockers.push("native-host-unit-graph-planning-blocked".to_string());
    }
    if !unit_derivation_graph.ready {
        blockers.push("unit-derivation-graph-blocked".to_string());
    }
    blockers.sort();
    blockers.dedup();

    let mut receipt = RustPlanReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        cargo_mode: rust_plan_cargo_mode(true, blockers),
        workspace_root: normalize_path_string(&options.root),
        cargo_version: "cargo-free:no-cargo-oracle".to_string(),
        rustc_version_verbose: rustc_version_verbose.trim().to_string(),
        lockfile,
        invocation: RustPlanInvocation {
            profile: options.profile.clone(),
            targets: sorted_strings(options.targets.clone()),
            features: sorted_strings(options.features.clone()),
            all_features: options.all_features,
            no_default_features: options.no_default_features,
        },
        package_count: native_cargo_packages.len(),
        packages,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        native_package_target_planning,
        native_unit_graph_planning,
        native_host_unit_graph_planning,
        unit_graph: summarize_unit_graph(unit_graph_value)?,
        unit_derivation_graph,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt)?;
    Ok(receipt)
}

pub(crate) fn print_rust_plan_receipt(receipt: &RustPlanReceipt, json_mode: bool) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust plan receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_execution_receipt(
    receipt: &RustPlanExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust plan execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_dependency_chain_execution_receipt(
    receipt: &RustPlanDependencyChainExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust dependency-chain execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_target_topology_execution_receipt(
    receipt: &RustPlanTargetTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust target-topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_host_artifact_topology_execution_receipt(
    receipt: &RustPlanHostArtifactTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust host-artifact topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_topology_execution_receipt(
    receipt: &RustPlanTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_dev_dependency_test_topology_execution_receipt(
    receipt: &RustPlanDevDependencyTestTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| {
        RunError::Internal(format!("rendering Rust dev-dependency test topology execution receipt: {err}"))
    })?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_workspace_dependency_topology_execution_receipt(
    receipt: &RustPlanWorkspaceDependencyTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| {
        RunError::Internal(format!("rendering Rust workspace-dependency topology execution receipt: {err}"))
    })?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_patch_source_topology_execution_receipt(
    receipt: &RustPlanPatchSourceTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust patch-source topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn default_profile() -> String {
    DEFAULT_CARGO_PROFILE.to_string()
}

fn native_path_cargo_packages(root: &Path, options: &RustPlanOptions) -> Result<Vec<CargoPackage>, RunError> {
    const MAX_NATIVE_PATH_PACKAGES: usize = 4096;
    let mut blockers = Vec::new();
    let mut queued = native_workspace_manifest_paths(root, &mut blockers).into_iter().collect::<VecDeque<_>>();
    if !blockers.is_empty() {
        let messages = blockers
            .iter()
            .map(|blocker| format!("{}: {}", blocker.class, blocker.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(RunError::Internal(format!("native Cargo-free manifest discovery failed: {messages}")));
    }
    let mut visited = BTreeSet::new();
    let mut packages = Vec::new();
    while let Some(manifest_path) = queued.pop_front() {
        let normalized_manifest = normalize_path_string(&manifest_path);
        if !visited.insert(normalized_manifest) {
            continue;
        }
        if visited.len() > MAX_NATIVE_PATH_PACKAGES {
            return Err(RunError::Internal(format!(
                "native Cargo-free manifest discovery exceeded {MAX_NATIVE_PATH_PACKAGES} path packages"
            )));
        }
        let manifest = read_native_manifest(&manifest_path).map_err(RunError::Internal)?;
        queued.extend(native_manifest_path_dependency_manifests(&manifest_path, &manifest));
        packages.push(native_path_cargo_package_from_manifest(root, &manifest_path, &manifest, options)?);
    }
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    packages.dedup_by(|left, right| left.id == right.id);
    Ok(packages)
}

fn native_reachable_lock_source_cargo_packages(
    root: &Path,
    facts: &NativeLockfileFacts,
    lock_packages: &[LockPackage],
    root_packages: &[CargoPackage],
) -> Vec<CargoPackage> {
    let reachable = reachable_lock_package_keys(facts, root_packages);
    let mut packages = native_lock_source_cargo_packages(root, lock_packages)
        .into_iter()
        .filter(|package| {
            reachable.contains(&lock_package_key(&package.name, &package.version, package.source.as_deref()))
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    packages.dedup_by(|left, right| left.id == right.id);
    packages
}

fn reachable_lock_package_keys(
    facts: &NativeLockfileFacts,
    root_packages: &[CargoPackage],
) -> BTreeSet<(String, String, Option<String>)> {
    let package_by_key = facts
        .packages
        .iter()
        .map(|package| (lock_package_key(&package.name, &package.version, package.source.as_deref()), package))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::new();
    let mut queue = Vec::new();
    for package in root_packages {
        let key = lock_package_key(&package.name, &package.version, package.source.as_deref());
        if package_by_key.contains_key(&key) && reachable.insert(key.clone()) {
            queue.push(key);
        }
    }
    let mut cursor = 0usize;
    while cursor < queue.len() {
        if cursor >= NATIVE_LOCK_REACHABILITY_MAX_STEPS {
            break;
        }
        let key = queue[cursor].clone();
        cursor += 1;
        let Some(package) = package_by_key.get(&key) else {
            continue;
        };
        for dependency in &package.dependencies {
            for dependency_key in lock_dependency_matching_keys(facts, dependency) {
                if reachable.insert(dependency_key.clone()) {
                    queue.push(dependency_key);
                }
            }
        }
    }
    reachable
}

fn lock_dependency_matching_keys(
    facts: &NativeLockfileFacts,
    dependency: &NativeLockDependencyFact,
) -> Vec<(String, String, Option<String>)> {
    let mut keys = facts
        .packages
        .iter()
        .filter(|package| package.name == dependency.name)
        .filter(|package| dependency.version.as_ref().is_none_or(|version| package.version == *version))
        .filter(|package| dependency.source.as_ref().is_none_or(|source| package.source.as_ref() == Some(source)))
        .map(|package| lock_package_key(&package.name, &package.version, package.source.as_deref()))
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

fn lock_package_key(name: &str, version: &str, source: Option<&str>) -> (String, String, Option<String>) {
    (name.to_string(), version.to_string(), source.map(ToString::to_string))
}

fn native_lock_source_cargo_packages(root: &Path, lock_packages: &[LockPackage]) -> Vec<CargoPackage> {
    let mut vendor_blockers = Vec::new();
    let vendor_roots = declared_vendor_roots(root, &mut vendor_blockers);
    let mut packages = lock_packages
        .iter()
        .filter_map(|package| native_lock_source_cargo_package(&vendor_roots, package))
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    packages.dedup_by(|left, right| left.id == right.id);
    packages
}

fn native_lock_source_cargo_package(vendor_roots: &[PathBuf], package: &LockPackage) -> Option<CargoPackage> {
    let source = package.source.as_ref()?;
    let kind = source_kind(Some(source));
    if !matches!(kind, SourceKind::Registry | SourceKind::Git) {
        return None;
    }
    let manifest_path = locate_declared_vendor_manifest(vendor_roots, &package.name, &package.version)
        .map(|path| normalize_path_string(&path))
        .unwrap_or_default();
    Some(CargoPackage {
        id: cargo_source_package_id(source, &package.name, &package.version),
        name: package.name.clone(),
        version: package.version.clone(),
        source: Some(source.clone()),
        manifest_path,
        targets: Vec::new(),
        features: BTreeMap::new(),
    })
}

fn native_registry_cargo_packages_from_sources(sources: &[NativeRegistrySourceSummary]) -> Vec<CargoPackage> {
    let mut packages = sources
        .iter()
        .map(|source| CargoPackage {
            id: source.package_id.clone(),
            name: source.name.clone(),
            version: source.version.clone(),
            source: Some(source.source.clone()),
            manifest_path: source.manifest_path.clone(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    packages.dedup_by(|left, right| left.id == right.id);
    packages
}

fn native_git_cargo_packages_from_lock(lock_source_packages: &[CargoPackage]) -> Vec<CargoPackage> {
    let mut packages = lock_source_packages
        .iter()
        .filter(|package| source_kind(package.source.as_deref()) == SourceKind::Git)
        .cloned()
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    packages.dedup_by(|left, right| left.id == right.id);
    packages
}

fn locate_declared_vendor_manifest(vendor_roots: &[PathBuf], name: &str, version: &str) -> Option<PathBuf> {
    for vendor_root in vendor_roots {
        let versioned = vendor_root.join(format!("{name}-{version}")).join("Cargo.toml");
        if versioned.is_file() {
            return Some(versioned);
        }
        let unversioned = vendor_root.join(name).join("Cargo.toml");
        if unversioned.is_file() {
            return Some(unversioned);
        }
    }
    None
}

fn cargo_source_package_id(source: &str, name: &str, version: &str) -> String {
    format!("{source}#{name}@{version}")
}

fn native_manifest_path_dependency_manifests(manifest_path: &Path, manifest: &NativeManifest) -> Vec<PathBuf> {
    let Some(source_root) = manifest_path.parent() else {
        return Vec::new();
    };
    let mut manifests = Vec::new();
    manifests.extend(native_dependency_table_path_manifests(source_root, &manifest.dependencies));
    manifests.extend(native_dependency_table_path_manifests(source_root, &manifest.build_dependencies));
    for target in manifest.target.values() {
        manifests.extend(native_dependency_table_path_manifests(source_root, &target.dependencies));
    }
    manifests.sort();
    manifests.dedup();
    manifests
}

fn native_dependency_table_path_manifests(
    source_root: &Path,
    dependencies: &BTreeMap<String, toml::Value>,
) -> Vec<PathBuf> {
    dependencies
        .values()
        .filter_map(|dependency| dependency.get("path").and_then(toml::Value::as_str))
        .map(|path| source_root.join(path).join("Cargo.toml"))
        .collect()
}

fn native_path_cargo_package_from_manifest(
    root: &Path,
    manifest_path: &Path,
    manifest: &NativeManifest,
    options: &RustPlanOptions,
) -> Result<CargoPackage, RunError> {
    let package = manifest.package.as_ref().ok_or_else(|| {
        RunError::Internal(format!("Cargo-free native package manifest lacks [package]: {}", manifest_path.display()))
    })?;
    let version = native_package_version(root, &package.name, &package.version).map_err(|blocker| {
        RunError::Internal(format!(
            "Cargo-free native package version failed for {}: {}",
            package.name, blocker.message
        ))
    })?;
    let edition = native_package_edition(root, &package.name, &package.edition).map_err(|blocker| {
        RunError::Internal(format!(
            "Cargo-free native package edition failed for {}: {}",
            package.name, blocker.message
        ))
    })?;
    let source_root = manifest_path.parent().ok_or_else(|| {
        RunError::Internal(format!("Cargo-free manifest path has no parent: {}", manifest_path.display()))
    })?;
    let native_targets = native_targets_for_manifest(source_root, &package.name, &version, &edition, &manifest)
        .map_err(|blocker| {
            RunError::Internal(format!("Cargo-free native target planning failed: {}", blocker.message))
        })?;
    let targets = native_targets
        .into_iter()
        .map(|target| CargoTarget {
            name: target.name,
            kind: vec![target.kind],
            src_path: target.source_path,
        })
        .collect::<Vec<_>>();
    let selected_features = native_selected_features(
        options,
        &manifest.features,
        &native_optional_dependency_names(&manifest.dependencies),
    );
    Ok(CargoPackage {
        id: cargo_path_package_id(&package.name, &version),
        name: package.name.clone(),
        version,
        source: None,
        manifest_path: normalize_path_string(manifest_path),
        targets,
        features: BTreeMap::from([("default".to_string(), selected_features)]),
    })
}

fn empty_native_unit_graph_value() -> Value {
    serde_json::json!({"units": []})
}

fn validate_options(options: &RustPlanOptions) -> Result<(), RunError> {
    if !options.root.is_dir() {
        return Err(RunError::Internal(format!("Rust plan root is not a directory: {}", options.root.display())));
    }
    if options.profile.trim().is_empty() {
        return Err(RunError::Internal("Rust plan profile must not be empty".to_string()));
    }
    if options.all_features && !options.features.is_empty() {
        return Err(RunError::Internal("--all-features cannot be combined with --features".to_string()));
    }
    Ok(())
}

fn run_checked_text(result: Result<CargoOutput, RunError>, label: &str) -> Result<String, RunError> {
    let bytes = run_checked_bytes(result, label)?;
    String::from_utf8(bytes).map_err(|err| RunError::Internal(format!("{label} emitted non-UTF-8 output: {err}")))
}

fn run_checked_bytes(result: Result<CargoOutput, RunError>, label: &str) -> Result<Vec<u8>, RunError> {
    let output = result?;
    if output.status_code != 0 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RunError::Build(format!(
            "Rust package planning failed at {label} with exit code {}: {}",
            output.status_code,
            stderr.trim()
        )));
    }
    Ok(output.stdout)
}

fn metadata_args() -> Vec<String> {
    vec!["metadata".to_string(), "--format-version".to_string(), "1".to_string()]
}

fn unit_graph_args(options: &RustPlanOptions) -> Vec<String> {
    let mut args = vec![
        "build".to_string(),
        "-Z".to_string(),
        "unstable-options".to_string(),
        "--unit-graph".to_string(),
        "--profile".to_string(),
        options.profile.clone(),
    ];
    for target in sorted_strings(options.targets.clone()) {
        args.push("--target".to_string());
        args.push(target);
    }
    if options.no_default_features {
        args.push("--no-default-features".to_string());
    }
    if options.all_features {
        args.push("--all-features".to_string());
    }
    if !options.features.is_empty() {
        args.push("--features".to_string());
        args.push(sorted_strings(options.features.clone()).join(","));
    }
    args
}

fn lockfile_identity(root: &Path) -> Result<LockfileIdentity, RunError> {
    let path = root.join("Cargo.lock");
    let bytes = std::fs::read(&path)
        .map_err(|err| RunError::Internal(format!("Rust plan requires a Cargo.lock at {}: {err}", path.display())))?;
    Ok(LockfileIdentity {
        path: normalize_path_string(&path),
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn summarize_packages(packages: Vec<CargoPackage>, workspace_members: &[String]) -> Vec<PackageSummary> {
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let mut summaries: Vec<PackageSummary> = packages
        .into_iter()
        .filter(|package| workspace_member_set.contains(package.id.as_str()))
        .map(|package| PackageSummary {
            id: package.id,
            name: package.name,
            version: package.version,
            source: package.source,
            manifest_path: normalize_path_string(Path::new(&package.manifest_path)),
        })
        .collect();
    summaries.sort_by(|left, right| left.id.cmp(&right.id));
    summaries
}

fn summarize_source_closure(
    packages: &[CargoPackage],
    lock_packages: &[LockPackage],
) -> Result<SourceClosureSummary, RunError> {
    let mut sources = Vec::with_capacity(packages.len());
    let mut blockers = Vec::new();
    for package in packages {
        let kind = source_kind(package.source.as_deref());
        let lockfile_identity = find_lock_package(package, lock_packages).map(|lock_package| LockPackageIdentity {
            name: lock_package.name.clone(),
            version: lock_package.version.clone(),
            source: lock_package.source.clone(),
            checksum: lock_package.checksum.clone(),
        });
        let resolved_revision = package.source.as_deref().and_then(source_revision);
        let digest = source_digest(package, &kind, lockfile_identity.as_ref());
        let source_digest = match digest {
            Ok(source_digest) => source_digest,
            Err(blocker) => {
                blockers.push(blocker);
                SourceDigest {
                    algorithm: "missing".to_string(),
                    value: "missing".to_string(),
                }
            }
        };
        sources.push(SourceInputSummary {
            package_id: package.id.clone(),
            name: package.name.clone(),
            version: package.version.clone(),
            kind,
            source: package.source.clone(),
            manifest_path: normalize_path_string(Path::new(&package.manifest_path)),
            lockfile_identity,
            resolved_revision,
            source_digest,
        });
    }
    sources.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    blockers.sort_by(|left, right| left.package_id.cmp(&right.package_id).then(left.class.cmp(&right.class)));
    let digest_blake3 = source_closure_digest(&sources, &blockers)?;
    Ok(SourceClosureSummary {
        source_count: sources.len(),
        ready: blockers.is_empty(),
        digest_blake3,
        sources,
        blockers,
    })
}

fn parse_lockfile_packages(root: &Path) -> Result<Vec<LockPackage>, RunError> {
    let facts = parse_lockfile_facts(root)?;
    Ok(lock_packages_from_facts(&facts))
}

fn parse_lockfile_facts(root: &Path) -> Result<NativeLockfileFacts, RunError> {
    let path = root.join("Cargo.lock");
    let text = fs::read_to_string(&path).map_err(|err| {
        RunError::Internal(format!("Rust plan requires a readable Cargo.lock at {}: {err}", path.display()))
    })?;
    parse_native_lockfile_text(&path.display().to_string(), &text).map_err(RunError::Internal)
}

fn parse_native_lockfile_text(path_label: &str, text: &str) -> Result<NativeLockfileFacts, String> {
    debug_assert!(!path_label.is_empty());
    let lockfile: CargoLockfile =
        toml::from_str(text).map_err(|err| format!("parsing lockfile {path_label}: {err}"))?;
    let mut packages = lockfile.package.into_iter().map(native_lock_package_fact).collect::<Vec<_>>();
    packages.sort();
    packages.dedup();
    Ok(NativeLockfileFacts { packages })
}

fn native_lock_package_fact(package: CargoLockPackageRecord) -> NativeLockfilePackageFact {
    debug_assert!(!package.name.is_empty());
    debug_assert!(!package.version.is_empty());
    let mut dependencies = package
        .dependencies
        .iter()
        .map(|dependency| parse_lock_dependency_fact(dependency))
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies.dedup();
    NativeLockfilePackageFact {
        name: package.name,
        version: package.version,
        revision: lock_source_revision(package.source.as_deref()),
        source: package.source,
        checksum: package.checksum,
        dependencies,
    }
}

fn parse_lock_dependency_fact(raw: &str) -> NativeLockDependencyFact {
    debug_assert!(!raw.is_empty());
    let (name_and_version, source) = split_lock_dependency_source(raw);
    let mut parts = name_and_version.split_whitespace().collect::<Vec<_>>();
    let version = parts
        .last()
        .copied()
        .filter(|candidate| lock_dependency_version_like(candidate))
        .map(str::to_string);
    if version.is_some() {
        let _ = parts.pop();
    }
    NativeLockDependencyFact {
        raw: raw.to_string(),
        name: parts.join(" "),
        version,
        source: source.map(str::to_string),
    }
}

fn split_lock_dependency_source(raw: &str) -> (&str, Option<&str>) {
    if let Some((name_and_version, source)) = raw.strip_suffix(')').and_then(|trimmed| trimmed.rsplit_once(" (")) {
        return (name_and_version, Some(source));
    }
    (raw, None)
}

fn lock_dependency_version_like(candidate: &str) -> bool {
    candidate.as_bytes().first().is_some_and(u8::is_ascii_digit) && candidate.contains('.')
}

fn lock_source_revision(source: Option<&str>) -> Option<String> {
    source
        .and_then(|source| source.split_once('#').map(|(_, revision)| revision.to_string()))
        .filter(|revision| !revision.is_empty())
}

fn lock_packages_from_facts(facts: &NativeLockfileFacts) -> Vec<LockPackage> {
    facts
        .packages
        .iter()
        .map(|package| LockPackage {
            name: package.name.clone(),
            version: package.version.clone(),
            source: package.source.clone(),
            checksum: package.checksum.clone(),
        })
        .collect()
}

fn find_lock_package<'a>(package: &CargoPackage, lock_packages: &'a [LockPackage]) -> Option<&'a LockPackage> {
    lock_packages.iter().find(|lock_package| {
        lock_package.name == package.name
            && lock_package.version == package.version
            && lock_source_matches(lock_package.source.as_deref(), package.source.as_deref())
    })
}

fn lock_source_matches(lock_source: Option<&str>, metadata_source: Option<&str>) -> bool {
    match (lock_source, metadata_source) {
        (None, None) => true,
        (Some(lock_source), Some(metadata_source)) => lock_source == metadata_source,
        _ => false,
    }
}

fn source_kind(source: Option<&str>) -> SourceKind {
    match source {
        None => SourceKind::Path,
        Some(source) if source.starts_with(CARGO_REGISTRY_SOURCE_PREFIX) => SourceKind::Registry,
        Some(source) if source.starts_with(CARGO_GIT_SOURCE_PREFIX) => SourceKind::Git,
        Some(_) => SourceKind::Other,
    }
}

fn source_revision(source: &str) -> Option<String> {
    source
        .rsplit_once('#')
        .and_then(|(_, revision)| (!revision.is_empty()).then(|| revision.to_string()))
}

fn source_digest(
    package: &CargoPackage,
    kind: &SourceKind,
    lockfile_identity: Option<&LockPackageIdentity>,
) -> Result<SourceDigest, SourceClosureBlocker> {
    match kind {
        SourceKind::Path => path_source_digest(package),
        SourceKind::Registry => registry_source_digest(package, lockfile_identity),
        SourceKind::Git => git_source_digest(package),
        SourceKind::Other => Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "unsupported-source-kind".to_string(),
            message: "Cargo source kind is not registry, git, or path".to_string(),
        }),
    }
}

fn registry_source_digest(
    package: &CargoPackage,
    lockfile_identity: Option<&LockPackageIdentity>,
) -> Result<SourceDigest, SourceClosureBlocker> {
    let Some(checksum) = lockfile_identity.and_then(|identity| identity.checksum.as_ref()) else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-registry-checksum".to_string(),
            message: "registry package lacks a Cargo.lock checksum".to_string(),
        });
    };
    Ok(SourceDigest {
        algorithm: REGISTRY_SOURCE_DIGEST_ALGORITHM.to_string(),
        value: checksum.clone(),
    })
}

fn git_source_digest(package: &CargoPackage) -> Result<SourceDigest, SourceClosureBlocker> {
    let Some(source) = package.source.as_deref() else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-git-source".to_string(),
            message: "git package lacks a source URL".to_string(),
        });
    };
    let Some(revision) = source_revision(source) else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-git-revision".to_string(),
            message: "git package lacks a resolved revision".to_string(),
        });
    };
    Ok(SourceDigest {
        algorithm: GIT_SOURCE_DIGEST_ALGORITHM.to_string(),
        value: revision,
    })
}

fn path_source_digest(package: &CargoPackage) -> Result<SourceDigest, SourceClosureBlocker> {
    let manifest_path = Path::new(&package.manifest_path);
    let Some(source_root) = manifest_path.parent() else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "invalid-path-source".to_string(),
            message: "path package manifest has no parent directory".to_string(),
        });
    };
    hash_path_source_tree(source_root)
        .map(|value| SourceDigest {
            algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
            value,
        })
        .map_err(|message| SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "path-source-unreadable".to_string(),
            message,
        })
}

fn hash_path_source_tree(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_source_files(root, &mut files)?;
    let mut hasher = blake3::Hasher::new();
    for file in files {
        let relative = file.strip_prefix(root).map_err(|err| format!("normalizing {}: {err}", file.display()))?;
        let relative_text = normalize_path_string(relative);
        let bytes = fs::read(&file).map_err(|err| format!("reading path source {}: {err}", file.display()))?;
        hasher.update(b"file\0");
        hasher.update(relative_text.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_source_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    collect_source_files_inner(root, root, files)?;
    files.sort();
    Ok(())
}

fn collect_source_files_inner(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|err| format!("reading path source directory {}: {err}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("reading path source directory {}: {err}", directory.display()))?;
        let path = entry.path();
        let file_name = entry.file_name();
        if should_skip_source_entry(root, directory, file_name.as_os_str()) {
            continue;
        }
        let file_type =
            entry.file_type().map_err(|err| format!("reading path source metadata {}: {err}", path.display()))?;
        if file_type.is_dir() {
            collect_source_files_inner(root, &path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn should_skip_source_entry(root: &Path, directory: &Path, file_name: &OsStr) -> bool {
    let Some(name) = file_name.to_str() else {
        return false;
    };
    if PATH_SOURCE_DIGEST_SKIP_ANYWHERE.contains(&name) {
        return true;
    }
    directory == root && PATH_SOURCE_DIGEST_SKIP_AT_ROOT.contains(&name)
}

fn source_closure_digest(
    sources: &[SourceInputSummary],
    blockers: &[SourceClosureBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        sources: &'a [SourceInputSummary],
        blockers: &'a [SourceClosureBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { sources, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust source closure: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

#[derive(Debug, Deserialize)]
struct CargoConfigToml {
    #[serde(default)]
    source: BTreeMap<String, CargoConfigSource>,
}

#[derive(Debug, Deserialize, Default)]
struct CargoConfigSource {
    directory: Option<String>,
    #[serde(rename = "replace-with")]
    replace_with: Option<String>,
}

fn summarize_native_registry_source_planning(
    root: &Path,
    cargo_packages: &[CargoPackage],
    lock_packages: &[LockPackage],
    lockfile: &LockfileIdentity,
) -> Result<NativeRegistrySourcePlanningSummary, RunError> {
    let mut blockers = Vec::new();
    let vendor_roots = declared_vendor_roots(root, &mut blockers);
    let mut sources = Vec::new();
    for package in cargo_packages
        .iter()
        .filter(|package| source_kind(package.source.as_deref()) == SourceKind::Registry)
    {
        let lock_identity = find_lock_package(package, lock_packages).map(|lock_package| LockPackageIdentity {
            name: lock_package.name.clone(),
            version: lock_package.version.clone(),
            source: lock_package.source.clone(),
            checksum: lock_package.checksum.clone(),
        });
        let package_id = package.id.clone();
        let Some(lock_identity) = lock_identity else {
            blockers.push(native_registry_blocker(
                Some(package_id),
                "missing-lockfile-registry-identity",
                "registry package has no matching Cargo.lock package identity",
            ));
            continue;
        };
        let Some(source) = lock_identity.source.clone().or_else(|| package.source.clone()) else {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "missing-registry-source",
                "registry package lacks lockfile source material",
            ));
            continue;
        };
        if !source.starts_with(CARGO_REGISTRY_SOURCE_PREFIX) {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "unsupported-registry-source-kind",
                "native registry source planning supports only registry+ lockfile sources",
            ));
            continue;
        }
        let Some(checksum) = lock_identity.checksum.clone() else {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "missing-registry-checksum",
                "registry package lacks Cargo.lock checksum material",
            ));
            continue;
        };
        if lock_identity.name != package.name
            || lock_identity.version != package.version
            || Some(source.clone()) != package.source
        {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "cargo-oracle-registry-identity-mismatch",
                "Cargo oracle registry package identity differs from Cargo.lock identity",
            ));
            continue;
        }
        match bind_declared_vendor_source(root, package, &checksum, &vendor_roots) {
            Ok((vendor_root, manifest_path, source_digest)) => sources.push(NativeRegistrySourceSummary {
                package_id: package.id.clone(),
                name: package.name.clone(),
                version: package.version.clone(),
                source,
                source_class: "registry".to_string(),
                checksum,
                vendor_root: normalize_path_string(&vendor_root),
                manifest_path: normalize_path_string(&manifest_path),
                source_digest,
                lockfile_identity: lock_identity,
            }),
            Err(blocker) => blockers.push(blocker),
        }
    }
    append_native_patch_sources(root, cargo_packages, &mut sources, &mut blockers);
    sources.sort();
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let digest_blake3 = native_registry_source_digest(&sources, &blockers, &comparison_status, &lockfile.blake3)?;
    Ok(NativeRegistrySourcePlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        lockfile_digest_blake3: lockfile.blake3.clone(),
        digest_blake3,
        sources,
        blockers,
        non_claims: vec![
            "declared-local-vendor-source-only".to_string(),
            "no-network-fetch".to_string(),
            "no-version-solving".to_string(),
            "no-ambient-cargo-cache".to_string(),
            "not-general-cargo-registry-compatibility".to_string(),
        ],
    })
}

fn append_native_patch_sources(
    root: &Path,
    cargo_packages: &[CargoPackage],
    sources: &mut Vec<NativeRegistrySourceSummary>,
    blockers: &mut Vec<NativeRegistrySourceBlocker>,
) {
    let manifest_path = root.join("Cargo.toml");
    let manifest = match read_native_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(message) => {
            blockers.push(native_registry_blocker(None, "unreadable-patch-source-manifest", &message));
            return;
        }
    };
    if manifest.patch.is_empty() {
        return;
    }
    for (registry, patches) in &manifest.patch {
        if registry != "crates-io" {
            blockers.push(native_registry_blocker(
                None,
                "unsupported-patch-source-registry",
                &format!("patch registry `{registry}` is outside the bounded crates-io patch fragment"),
            ));
            continue;
        }
        for (dependency_key, value) in patches {
            let Some(path) = dependency_path(value) else {
                blockers.push(native_registry_blocker(
                    None,
                    "unsupported-patch-source-kind",
                    &format!("patch `{dependency_key}` is outside the bounded local path patch fragment"),
                ));
                continue;
            };
            let package_name = dependency_package_name(dependency_key, value);
            let patch_manifest = root.join(path).join("Cargo.toml");
            if !patch_manifest.is_file() {
                blockers.push(native_registry_blocker(
                    None,
                    "missing-patch-source-manifest",
                    &format!("patch `{dependency_key}` manifest {} is not readable", patch_manifest.display()),
                ));
                continue;
            }
            let Some(package) = cargo_packages.iter().find(|package| {
                package.name == package_name && manifest_paths_same(&package.manifest_path, &patch_manifest)
            }) else {
                blockers.push(native_registry_blocker(
                    None,
                    "missing-patch-source-cargo-package",
                    &format!("patch `{dependency_key}` has no matching Cargo metadata package"),
                ));
                continue;
            };
            if source_kind(package.source.as_deref()) != SourceKind::Path {
                blockers.push(native_registry_blocker(
                    Some(package.id.clone()),
                    "unsupported-patch-source-cargo-kind",
                    "patched package must resolve to a local path package in Cargo metadata",
                ));
                continue;
            }
            match path_source_digest(package) {
                Ok(source_digest) => sources.push(NativeRegistrySourceSummary {
                    package_id: package.id.clone(),
                    name: package.name.clone(),
                    version: package.version.clone(),
                    source: format!("patch+{registry}"),
                    source_class: "patch-path".to_string(),
                    checksum: format!("patch-path:{}", source_digest.value),
                    vendor_root: normalize_path_string(&root.join(path)),
                    manifest_path: normalize_path_string(&patch_manifest),
                    source_digest,
                    lockfile_identity: LockPackageIdentity {
                        name: package.name.clone(),
                        version: package.version.clone(),
                        source: None,
                        checksum: None,
                    },
                }),
                Err(blocker) => {
                    blockers.push(native_registry_blocker(Some(package.id.clone()), &blocker.class, &blocker.message))
                }
            }
        }
    }
}

fn declared_vendor_roots(root: &Path, blockers: &mut Vec<NativeRegistrySourceBlocker>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for config_path in [root.join(".cargo/config.toml"), root.join(".cargo/config")] {
        if !config_path.is_file() {
            continue;
        }
        let config_text = match fs::read_to_string(&config_path) {
            Ok(text) => text,
            Err(err) => {
                blockers.push(native_registry_blocker(
                    None,
                    "unreadable-cargo-source-config",
                    &format!("reading declared Cargo source config {}: {err}", config_path.display()),
                ));
                continue;
            }
        };
        let config: CargoConfigToml = match toml::from_str(&config_text) {
            Ok(config) => config,
            Err(err) => {
                blockers.push(native_registry_blocker(
                    None,
                    "invalid-cargo-source-config",
                    &format!("parsing declared Cargo source config {}: {err}", config_path.display()),
                ));
                continue;
            }
        };
        for source in config.source.values() {
            let declared_directory = source.directory.as_deref().or_else(|| {
                source
                    .replace_with
                    .as_ref()
                    .and_then(|replace_with| config.source.get(replace_with))
                    .and_then(|replacement| replacement.directory.as_deref())
            });
            let Some(directory) = declared_directory else {
                continue;
            };
            let path = Path::new(directory);
            let vendor_root = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            roots.push(fs::canonicalize(&vendor_root).unwrap_or(vendor_root));
        }
    }
    let conventional_vendor_deps = root.join("vendor-deps");
    if conventional_vendor_deps.is_dir() {
        roots.push(fs::canonicalize(&conventional_vendor_deps).unwrap_or(conventional_vendor_deps));
    }
    roots.sort();
    roots.dedup();
    roots
}

fn bind_declared_vendor_source(
    _root: &Path,
    package: &CargoPackage,
    checksum: &str,
    vendor_roots: &[PathBuf],
) -> Result<(PathBuf, PathBuf, SourceDigest), NativeRegistrySourceBlocker> {
    if vendor_roots.is_empty() {
        return Err(native_registry_blocker(
            Some(package.id.clone()),
            "missing-declared-vendor-root",
            "registry package requires a declared local vendor/source root; ambient Cargo caches are not accepted",
        ));
    }
    let manifest_path = PathBuf::from(&package.manifest_path);
    let manifest_parent = manifest_path.parent();
    for vendor_root in vendor_roots {
        let candidate_roots = [
            vendor_root.join(format!("{}-{}", package.name, package.version)),
            vendor_root.join(&package.name),
        ];
        for package_root in candidate_roots {
            let candidate_manifest = package_root.join("Cargo.toml");
            let manifest_candidate = if candidate_manifest.is_file() {
                Some(candidate_manifest)
            } else if manifest_parent.is_some_and(|parent| parent.starts_with(vendor_root)) && manifest_path.is_file() {
                Some(manifest_path.clone())
            } else {
                None
            };
            let Some(manifest_candidate) = manifest_candidate else {
                continue;
            };
            let source_root = manifest_candidate.parent().ok_or_else(|| {
                native_registry_blocker(
                    Some(package.id.clone()),
                    "invalid-vendor-source-root",
                    "declared vendor manifest path has no parent directory",
                )
            })?;
            validate_vendor_checksum(package, source_root, checksum)?;
            let digest = hash_path_source_tree(source_root).map_err(|message| {
                native_registry_blocker(Some(package.id.clone()), "vendor-source-unreadable", &message)
            })?;
            return Ok((vendor_root.clone(), manifest_candidate, SourceDigest {
                algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
                value: digest,
            }));
        }
    }
    Err(native_registry_blocker(
        Some(package.id.clone()),
        "missing-vendor-source-root",
        "declared vendor/source roots do not contain the registry package source tree",
    ))
}

fn validate_vendor_checksum(
    package: &CargoPackage,
    source_root: &Path,
    expected_checksum: &str,
) -> Result<(), NativeRegistrySourceBlocker> {
    let checksum_path = source_root.join(".cargo-checksum.json");
    let text = fs::read_to_string(&checksum_path).map_err(|err| {
        native_registry_blocker(
            Some(package.id.clone()),
            "missing-vendor-checksum-manifest",
            &format!("reading vendor checksum manifest {}: {err}", checksum_path.display()),
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        native_registry_blocker(
            Some(package.id.clone()),
            "invalid-vendor-checksum-manifest",
            &format!("parsing vendor checksum manifest {}: {err}", checksum_path.display()),
        )
    })?;
    let actual = value.get("package").and_then(Value::as_str).ok_or_else(|| {
        native_registry_blocker(
            Some(package.id.clone()),
            "missing-vendor-package-checksum",
            "vendor checksum manifest lacks package checksum material",
        )
    })?;
    if actual != expected_checksum {
        return Err(native_registry_blocker(
            Some(package.id.clone()),
            "vendor-checksum-mismatch",
            "vendor package checksum does not match Cargo.lock checksum material",
        ));
    }
    Ok(())
}

fn summarize_native_git_source_planning(
    cargo_packages: &[CargoPackage],
    lock_packages: &[LockPackage],
    lockfile: &LockfileIdentity,
    source_closure: &SourceClosureSummary,
) -> Result<NativeGitSourcePlanningSummary, RunError> {
    let mut sources = Vec::new();
    let mut blockers = Vec::new();
    for package in cargo_packages.iter().filter(|package| source_kind(package.source.as_deref()) == SourceKind::Git) {
        match bind_captured_git_source(package, lock_packages, source_closure) {
            Ok(source) => sources.push(source),
            Err(blocker) => blockers.push(blocker),
        }
    }
    sources.sort();
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let digest_blake3 = native_git_source_digest(
        &sources,
        &blockers,
        &comparison_status,
        &lockfile.blake3,
        &source_closure.digest_blake3,
    )?;
    Ok(NativeGitSourcePlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        lockfile_digest_blake3: lockfile.blake3.clone(),
        source_closure_digest_blake3: source_closure.digest_blake3.clone(),
        digest_blake3,
        sources,
        blockers,
        non_claims: vec![
            "captured-source-closure-only".to_string(),
            "source-material-provider-agnostic".to_string(),
            "content-addressed-source-by-blake3".to_string(),
            "no-network-fetch".to_string(),
            "no-version-solving".to_string(),
            "no-ambient-cargo-git-scan".to_string(),
            "not-general-cargo-git-compatibility".to_string(),
        ],
    })
}

fn bind_captured_git_source(
    package: &CargoPackage,
    lock_packages: &[LockPackage],
    source_closure: &SourceClosureSummary,
) -> Result<NativeGitSourceSummary, NativeGitSourceBlocker> {
    let package_id = package.id.clone();
    let lock_identity = find_lock_package(package, lock_packages)
        .map(|lock_package| LockPackageIdentity {
            name: lock_package.name.clone(),
            version: lock_package.version.clone(),
            source: lock_package.source.clone(),
            checksum: lock_package.checksum.clone(),
        })
        .ok_or_else(|| {
            native_git_blocker(
                Some(package_id.clone()),
                "missing-lockfile-git-identity",
                "git package has no matching Cargo.lock package identity",
            )
        })?;
    let source = lock_identity.source.clone().or_else(|| package.source.clone()).ok_or_else(|| {
        native_git_blocker(Some(package_id.clone()), "missing-git-source", "git package lacks lockfile source material")
    })?;
    if !source.starts_with(CARGO_GIT_SOURCE_PREFIX) {
        return Err(native_git_blocker(
            Some(package_id),
            "unsupported-git-source-kind",
            "native git source planning supports only git+ lockfile sources",
        ));
    }
    let resolved_revision = source_revision(&source).ok_or_else(|| {
        native_git_blocker(Some(package_id.clone()), "missing-git-revision", "git package lacks a resolved revision")
    })?;
    if lock_identity.name != package.name
        || lock_identity.version != package.version
        || Some(source.clone()) != package.source
    {
        return Err(native_git_blocker(
            Some(package_id),
            "cargo-oracle-git-identity-mismatch",
            "Cargo oracle git package identity differs from Cargo.lock identity",
        ));
    }
    let matching_sources = source_closure
        .sources
        .iter()
        .filter(|source| source.package_id == package.id && source.kind == SourceKind::Git)
        .collect::<Vec<_>>();
    if matching_sources.len() != 1 {
        return Err(native_git_blocker(
            Some(package.id.clone()),
            "ambiguous-git-source-closure-material",
            "git package requires exactly one captured source-closure record",
        ));
    }
    let closure_source = matching_sources[0];
    if closure_source.source.as_deref() != Some(source.as_str())
        || closure_source.resolved_revision.as_deref() != Some(resolved_revision.as_str())
    {
        return Err(native_git_blocker(
            Some(package.id.clone()),
            "source-closure-git-identity-mismatch",
            "captured source-closure git identity differs from Cargo.lock identity",
        ));
    }
    let manifest_path = PathBuf::from(&closure_source.manifest_path);
    if !manifest_path.is_file() {
        return Err(native_git_blocker(
            Some(package.id.clone()),
            "missing-git-source-manifest",
            &format!("captured git source manifest {} is not readable", manifest_path.display()),
        ));
    }
    let source_root = manifest_path.parent().ok_or_else(|| {
        native_git_blocker(
            Some(package.id.clone()),
            "invalid-git-source-root",
            "captured git source manifest has no parent directory",
        )
    })?;
    let source_digest = hash_path_source_tree(source_root)
        .map(|value| SourceDigest {
            algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
            value,
        })
        .map_err(|message| native_git_blocker(Some(package.id.clone()), "git-source-unreadable", &message))?;
    Ok(NativeGitSourceSummary {
        package_id: package.id.clone(),
        name: package.name.clone(),
        version: package.version.clone(),
        source,
        source_class: "git".to_string(),
        resolved_revision,
        source_root: normalize_path_string(source_root),
        manifest_path: normalize_path_string(&manifest_path),
        source_digest,
        lockfile_identity: lock_identity,
    })
}

fn native_git_source_digest(
    sources: &[NativeGitSourceSummary],
    blockers: &[NativeGitSourceBlocker],
    comparison_status: &str,
    lockfile_digest_blake3: &str,
    source_closure_digest_blake3: &str,
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        comparison_status: &'a str,
        lockfile_digest_blake3: &'a str,
        source_closure_digest_blake3: &'a str,
        sources: &'a [NativeGitSourceSummary],
        blockers: &'a [NativeGitSourceBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        comparison_status,
        lockfile_digest_blake3,
        source_closure_digest_blake3,
        sources,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native git source planning fragment: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_registry_source_digest(
    sources: &[NativeRegistrySourceSummary],
    blockers: &[NativeRegistrySourceBlocker],
    comparison_status: &str,
    lockfile_digest_blake3: &str,
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        comparison_status: &'a str,
        lockfile_digest_blake3: &'a str,
        sources: &'a [NativeRegistrySourceSummary],
        blockers: &'a [NativeRegistrySourceBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        comparison_status,
        lockfile_digest_blake3,
        sources,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native registry source planning fragment: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_registry_blocker(package_id: Option<String>, class: &str, message: &str) -> NativeRegistrySourceBlocker {
    NativeRegistrySourceBlocker {
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn native_git_blocker(package_id: Option<String>, class: &str, message: &str) -> NativeGitSourceBlocker {
    NativeGitSourceBlocker {
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn summarize_native_package_target_planning(
    root: &Path,
    options: &RustPlanOptions,
    unit_graph: &Value,
    cargo_packages: &[CargoPackage],
    workspace_members: &[String],
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<NativePackageTargetPlanningSummary, RunError> {
    let cargo_oracle_identity = if options.no_cargo_oracle {
        "cargo-free:no-cargo-package-target-oracle".to_string()
    } else {
        cargo_package_target_oracle_digest(cargo_packages, workspace_members)?
    };
    let mut selected_features_by_package = selected_features_by_package_from_unit_graph(unit_graph);
    let selected_package_ids = selected_package_ids_from_unit_graph(unit_graph);
    let mut blockers = Vec::new();
    let mut manifest_paths = native_workspace_manifest_paths(root, &mut blockers);
    manifest_paths.extend(
        source_closure
            .sources
            .iter()
            .filter(|source| source.kind == SourceKind::Path)
            .map(|source| PathBuf::from(&source.manifest_path)),
    );
    if native_registry_source_planning.ready {
        manifest_paths.extend(
            native_registry_source_planning
                .sources
                .iter()
                .filter(|source| !selected_package_ids.is_empty() && selected_package_ids.contains(&source.package_id))
                .map(|source| PathBuf::from(&source.manifest_path)),
        );
    }
    if native_git_source_planning.ready {
        manifest_paths.extend(
            native_git_source_planning
                .sources
                .iter()
                .filter(|source| !selected_package_ids.is_empty() && selected_package_ids.contains(&source.package_id))
                .map(|source| PathBuf::from(&source.manifest_path)),
        );
    }
    manifest_paths.sort();
    manifest_paths.dedup();
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let mut native_packages = Vec::new();
    let mut visited_manifest_paths = BTreeSet::new();
    let mut queued_manifest_paths = VecDeque::from(manifest_paths);
    while let Some(manifest_path) = queued_manifest_paths.pop_front() {
        if visited_manifest_paths.len() >= NATIVE_PACKAGE_MANIFEST_REACHABILITY_MAX_STEPS {
            blockers.push(native_blocker(
                None,
                "native-package-manifest-reachability-limit",
                "native package planning exceeded bounded manifest reachability limit",
            ));
            break;
        }
        let normalized_manifest_path = normalize_path_string(&manifest_path);
        if !visited_manifest_paths.insert(normalized_manifest_path) {
            continue;
        }
        match native_package_from_manifest(
            root,
            &manifest_path,
            options,
            source_closure,
            native_registry_source_planning,
            native_git_source_planning,
            &selected_features_by_package,
        ) {
            Ok(package) => {
                queue_native_dependency_manifest_paths(
                    root,
                    &package,
                    source_closure,
                    native_registry_source_planning,
                    native_git_source_planning,
                    &mut queued_manifest_paths,
                    &mut visited_manifest_paths,
                    &mut selected_features_by_package,
                    &mut blockers,
                );
                upsert_native_package(&mut native_packages, package);
            }
            Err(blocker) => blockers.push(blocker),
        }
    }
    let cargo_workspace_packages = cargo_packages
        .iter()
        .filter(|package| {
            workspace_member_set.contains(package.id.as_str())
                || native_packages.iter().any(|native_package| {
                    manifest_paths_same(&package.manifest_path, Path::new(&native_package.manifest_path))
                })
        })
        .collect::<Vec<_>>();
    native_packages.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    if !options.no_cargo_oracle {
        compare_native_packages_to_cargo(&native_packages, cargo_packages, &cargo_workspace_packages, &mut blockers);
    }
    blockers.sort();
    blockers.dedup();
    let comparison_status = if options.no_cargo_oracle && blockers.is_empty() {
        "cargo-free:no-oracle".to_string()
    } else if blockers.is_empty() {
        "matched".to_string()
    } else {
        "blocked".to_string()
    };
    let digest_blake3 = native_package_target_digest(&native_packages, &blockers, &comparison_status)?;
    Ok(NativePackageTargetPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_oracle_identity,
        digest_blake3,
        packages: native_packages,
        blockers,
        non_claims: vec![
            "bounded-lib-bin-path-fragment-only".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "not-full-cargo-compatibility".to_string(),
            "not-cargo-free-build-scheduling".to_string(),
        ],
    })
}

fn queue_native_dependency_manifest_paths(
    root: &Path,
    package: &NativePackagePlanningSummary,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    queued_manifest_paths: &mut VecDeque<PathBuf>,
    visited_manifest_paths: &mut BTreeSet<String>,
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    let dependencies = package.path_dependencies.iter().chain(package.build_dependencies.iter()).collect::<Vec<_>>();
    for dependency in &dependencies {
        queued_manifest_paths.push_back(PathBuf::from(&dependency.manifest_path));
    }
    let manifest = match read_native_manifest(Path::new(&package.manifest_path)) {
        Ok(manifest) => manifest,
        Err(message) => {
            blockers.push(native_blocker(
                Some(package.package_id.clone()),
                "unreadable-dependency-feature-manifest",
                &message,
            ));
            return;
        }
    };
    queue_dependency_feature_requests(
        root,
        &package.package_id,
        &manifest.dependencies,
        &dependencies,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        queued_manifest_paths,
        visited_manifest_paths,
        selected_features_by_package,
        blockers,
    );
    queue_dependency_feature_requests(
        root,
        &package.package_id,
        &manifest.build_dependencies,
        &dependencies,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        queued_manifest_paths,
        visited_manifest_paths,
        selected_features_by_package,
        blockers,
    );
    queue_target_cfg_dependency_feature_requests(
        root,
        package,
        &manifest.target,
        &dependencies,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        queued_manifest_paths,
        visited_manifest_paths,
        selected_features_by_package,
        blockers,
    );
    queue_parent_feature_dependency_requests(
        root,
        &package.package_id,
        &package.selected_features,
        &manifest.features,
        &dependencies,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
        queued_manifest_paths,
        visited_manifest_paths,
        selected_features_by_package,
        blockers,
    );
}

fn queue_target_cfg_dependency_feature_requests(
    root: &Path,
    package: &NativePackagePlanningSummary,
    target_tables: &BTreeMap<String, NativeManifestTarget>,
    selected_dependencies: &[&NativePathDependencySummary],
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    queued_manifest_paths: &mut VecDeque<PathBuf>,
    visited_manifest_paths: &mut BTreeSet<String>,
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    let selected_names = package
        .target_cfg_dependencies
        .iter()
        .filter(|dependency| dependency.decision == "selected")
        .map(|dependency| dependency.name.as_str())
        .collect::<BTreeSet<_>>();
    if selected_names.is_empty() {
        return;
    }
    for target in target_tables.values() {
        let selected_target_dependencies = target
            .dependencies
            .iter()
            .filter(|(name, _value)| selected_names.contains(name.as_str()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        queue_dependency_feature_requests(
            root,
            &package.package_id,
            &selected_target_dependencies,
            selected_dependencies,
            source_closure,
            native_registry_source_planning,
            native_git_source_planning,
            queued_manifest_paths,
            visited_manifest_paths,
            selected_features_by_package,
            blockers,
        );
    }
}

fn upsert_native_package(
    native_packages: &mut Vec<NativePackagePlanningSummary>,
    package: NativePackagePlanningSummary,
) {
    let Some(existing) = native_packages.iter_mut().find(|existing| existing.package_id == package.package_id) else {
        native_packages.push(package);
        return;
    };
    *existing = package;
}

fn queue_dependency_feature_requests(
    root: &Path,
    package_id: &str,
    dependency_table: &BTreeMap<String, toml::Value>,
    selected_dependencies: &[&NativePathDependencySummary],
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    queued_manifest_paths: &mut VecDeque<PathBuf>,
    visited_manifest_paths: &mut BTreeSet<String>,
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    for (name, value) in dependency_table {
        let Some(dependency) = selected_dependencies.iter().find(|dependency| dependency.name == *name) else {
            continue;
        };
        let dependency_manifest_path = PathBuf::from(&dependency.manifest_path);
        match selected_features_for_dependency(&dependency_manifest_path, value) {
            Ok(features) => merge_dependency_feature_request(
                root,
                dependency_manifest_path,
                features,
                source_closure,
                native_registry_source_planning,
                native_git_source_planning,
                queued_manifest_paths,
                visited_manifest_paths,
                selected_features_by_package,
            ),
            Err(message) => blockers.push(native_blocker(
                Some(package_id.to_string()),
                "unsupported-dependency-feature-request",
                &message,
            )),
        }
    }
}

fn queue_parent_feature_dependency_requests(
    root: &Path,
    package_id: &str,
    selected_parent_features: &[String],
    parent_feature_defs: &BTreeMap<String, Vec<String>>,
    selected_dependencies: &[&NativePathDependencySummary],
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    queued_manifest_paths: &mut VecDeque<PathBuf>,
    visited_manifest_paths: &mut BTreeSet<String>,
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    let forwarded_features = forwarded_dependency_features(selected_parent_features, parent_feature_defs);
    for dependency in selected_dependencies {
        let Some(features) = forwarded_features.get(&dependency.name) else {
            continue;
        };
        let dependency_manifest_path = PathBuf::from(&dependency.manifest_path);
        match selected_features_for_forwarded_dependency(&dependency_manifest_path, features) {
            Ok(selected_features) => merge_dependency_feature_request(
                root,
                dependency_manifest_path,
                selected_features,
                source_closure,
                native_registry_source_planning,
                native_git_source_planning,
                queued_manifest_paths,
                visited_manifest_paths,
                selected_features_by_package,
            ),
            Err(message) => blockers.push(native_blocker(
                Some(package_id.to_string()),
                "unsupported-forwarded-dependency-feature-request",
                &message,
            )),
        }
    }
}

fn forwarded_dependency_features(
    selected_parent_features: &[String],
    parent_feature_defs: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut forwarded = BTreeMap::new();
    for parent_feature in selected_parent_features {
        let Some(entries) = parent_feature_defs.get(parent_feature) else {
            continue;
        };
        for entry in entries {
            let Some((dependency, feature)) = forwarded_dependency_feature_entry(entry) else {
                continue;
            };
            let features: &mut Vec<String> = forwarded.entry(dependency).or_default();
            features.push(feature);
        }
    }
    for features in forwarded.values_mut() {
        features.sort();
        features.dedup();
    }
    forwarded
}

fn forwarded_dependency_feature_entry(entry: &str) -> Option<(String, String)> {
    let (dependency, feature) = entry.split_once('/')?;
    let dependency = dependency.trim_start_matches("dep:").trim_end_matches('?');
    if dependency.is_empty() || feature.is_empty() {
        return None;
    }
    Some((dependency.to_string(), feature.to_string()))
}

fn merge_dependency_feature_request(
    root: &Path,
    dependency_manifest_path: PathBuf,
    features: Vec<String>,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    queued_manifest_paths: &mut VecDeque<PathBuf>,
    visited_manifest_paths: &mut BTreeSet<String>,
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
) {
    let dependency_package_id = dependency_package_id_for_manifest(
        root,
        &dependency_manifest_path,
        source_closure,
        native_registry_source_planning,
        native_git_source_planning,
    );
    if merge_selected_features(selected_features_by_package, dependency_package_id, features) {
        visited_manifest_paths.remove(&normalize_path_string(&dependency_manifest_path));
        queued_manifest_paths.push_back(dependency_manifest_path);
    }
}

fn dependency_package_id_for_manifest(
    root: &Path,
    manifest_path: &Path,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> String {
    if let Some(source) = source_closure
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
    {
        return source.package_id.clone();
    }
    if let Some(source) = native_registry_source_planning
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
    {
        return source.package_id.clone();
    }
    if let Some(source) = native_git_source_planning
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
    {
        return source.package_id.clone();
    }
    match read_native_manifest(manifest_path) {
        Ok(manifest) => manifest
            .package
            .as_ref()
            .and_then(|package| {
                native_package_version(root, &package.name, &package.version)
                    .ok()
                    .map(|version| cargo_path_package_id(&package.name, &version))
            })
            .unwrap_or_else(|| normalize_path_string(manifest_path)),
        Err(_) => normalize_path_string(manifest_path),
    }
}

fn selected_features_for_dependency(manifest_path: &Path, value: &toml::Value) -> Result<Vec<String>, String> {
    let mut explicit_features = dependency_feature_list(value)?;
    let no_default_features = !dependency_default_features_enabled(value);
    if !no_default_features {
        explicit_features.push("default".to_string());
    }
    selected_features_for_dependency_request(manifest_path, explicit_features, no_default_features)
}

fn selected_features_for_forwarded_dependency(
    manifest_path: &Path,
    features: &[String],
) -> Result<Vec<String>, String> {
    selected_features_for_dependency_request(manifest_path, features.to_vec(), true)
}

fn selected_features_for_dependency_request(
    manifest_path: &Path,
    mut explicit_features: Vec<String>,
    no_default_features: bool,
) -> Result<Vec<String>, String> {
    let manifest = read_native_manifest(manifest_path)?;
    let optional_dependencies = native_optional_dependency_names(&manifest.dependencies);
    explicit_features.sort();
    explicit_features.dedup();
    Ok(resolve_native_features(NativeFeatureResolutionRequest {
        feature_defs: manifest.features,
        optional_dependencies,
        explicit_features,
        all_features: false,
        no_default_features,
    })
    .selected_features)
}

fn dependency_default_features_enabled(value: &toml::Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("default-features").or_else(|| table.get("default_features")))
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
}

fn dependency_feature_list(value: &toml::Value) -> Result<Vec<String>, String> {
    let Some(features) = value.as_table().and_then(|table| table.get("features")) else {
        return Ok(Vec::new());
    };
    let Some(array) = features.as_array() else {
        return Err("dependency features must be an array of strings".to_string());
    };
    let mut result = Vec::new();
    for feature in array {
        let Some(feature) = feature.as_str() else {
            return Err("dependency features must be strings".to_string());
        };
        result.push(feature.to_string());
    }
    result.sort();
    result.dedup();
    Ok(result)
}

fn merge_selected_features(
    selected_features_by_package: &mut BTreeMap<String, Vec<String>>,
    package_id: String,
    features: Vec<String>,
) -> bool {
    let entry = selected_features_by_package.entry(package_id).or_default();
    let old_features = entry.clone();
    entry.extend(features);
    entry.sort();
    entry.dedup();
    *entry != old_features
}

fn native_workspace_manifest_paths(root: &Path, blockers: &mut Vec<NativePackagePlanningBlocker>) -> Vec<PathBuf> {
    let root_manifest_path = root.join("Cargo.toml");
    let Ok(root_manifest) = read_native_manifest(&root_manifest_path) else {
        blockers.push(native_blocker(
            None,
            "missing-native-root-manifest",
            &format!("native planner requires readable root manifest at {}", root_manifest_path.display()),
        ));
        return Vec::new();
    };
    native_workspace_manifest_paths_from_root_manifest(root, &root_manifest_path, &root_manifest, blockers)
}

fn native_workspace_manifest_paths_from_root_manifest(
    root: &Path,
    root_manifest_path: &Path,
    root_manifest: &NativeManifest,
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    if root_manifest.package.is_some() {
        manifests.push(root_manifest_path.to_path_buf());
    }
    if let Some(workspace) = root_manifest.workspace.as_ref() {
        for member in &workspace.members {
            if workspace_member_has_unsupported_glob(member) {
                blockers.push(native_blocker(
                    None,
                    "unsupported-workspace-member-pattern",
                    &format!("native planner supports explicit members or one-level `*` workspace member globs, got `{member}`"),
                ));
                continue;
            }
            if member.contains('*') {
                match expand_workspace_member_glob(root, member) {
                    Ok(paths) => manifests.extend(paths),
                    Err(blocker) => blockers.push(blocker),
                }
            } else {
                manifests.push(root.join(member).join("Cargo.toml"));
            }
        }
    }
    manifests.sort();
    manifests.dedup();
    manifests
}

fn workspace_member_has_unsupported_glob(member: &str) -> bool {
    if member.contains('?') || member.contains('[') || member.contains(']') || member.contains("**") {
        return true;
    }
    if !member.contains('*') {
        return false;
    }
    let Some((prefix, suffix)) = member.split_once('*') else {
        return true;
    };
    !suffix.is_empty() || prefix.is_empty() || !prefix.ends_with('/') || prefix[..prefix.len() - 1].contains('*')
}

fn expand_workspace_member_glob(root: &Path, member: &str) -> Result<Vec<PathBuf>, NativePackagePlanningBlocker> {
    let Some((prefix, "")) = member.split_once('*') else {
        return Err(native_blocker(
            None,
            "unsupported-workspace-member-pattern",
            &format!("native planner cannot expand workspace member pattern `{member}`"),
        ));
    };
    let base = root.join(prefix.trim_end_matches('/'));
    let entries = fs::read_dir(&base).map_err(|err| {
        native_blocker(
            None,
            "unreadable-workspace-member-glob-root",
            &format!("reading workspace member glob root {}: {err}", base.display()),
        )
    })?;
    let mut manifests = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            native_blocker(
                None,
                "unreadable-workspace-member-glob-entry",
                &format!("reading workspace member glob entry {}: {err}", base.display()),
            )
        })?;
        let path = entry.path();
        if path.join("Cargo.toml").is_file() {
            manifests.push(path.join("Cargo.toml"));
        }
    }
    if manifests.is_empty() {
        return Err(native_blocker(
            None,
            "empty-workspace-member-glob",
            &format!("workspace member glob `{member}` matched no readable manifests"),
        ));
    }
    manifests.sort();
    Ok(manifests)
}

fn native_package_from_manifest(
    workspace_root: &Path,
    manifest_path: &Path,
    options: &RustPlanOptions,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
    selected_features_by_package: &BTreeMap<String, Vec<String>>,
) -> Result<NativePackagePlanningSummary, NativePackagePlanningBlocker> {
    let manifest =
        read_native_manifest(manifest_path).map_err(|message| native_blocker(None, "unreadable-manifest", &message))?;
    let Some(package) = manifest.package.as_ref() else {
        return Err(native_blocker(
            None,
            "missing-package-section",
            &format!("manifest {} lacks [package]", manifest_path.display()),
        ));
    };
    let package_version = native_package_version(workspace_root, &package.name, &package.version)?;
    let package_edition = native_package_edition(workspace_root, &package.name, &package.edition)?;
    let workspace_package = native_cargo_package_env_workspace(workspace_root, package)?;
    let package_id = source_closure
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
        .map(|source| source.package_id.clone())
        .or_else(|| {
            native_registry_source_planning
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.package_id.clone())
        })
        .or_else(|| {
            native_git_source_planning
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.package_id.clone())
        })
        .unwrap_or_else(|| cargo_path_package_id(&package.name, &package_version));
    let source_root = manifest_path.parent().ok_or_else(|| {
        native_blocker(Some(package_id.clone()), "invalid-manifest-path", "manifest path has no parent directory")
    })?;
    let targets =
        native_targets_for_manifest(source_root, &package.name, &package_version, &package_edition, &manifest)?;
    let optional_dependencies = native_optional_dependency_names(&manifest.dependencies);
    let selected_features = selected_features_by_package
        .get(&package_id)
        .cloned()
        .unwrap_or_else(|| native_selected_features(options, &manifest.features, &optional_dependencies));
    let active_target = active_rust_target(options);
    let (target_cfg_dependencies, selected_target_cfg_dependencies) = native_target_cfg_dependencies(
        source_root,
        &manifest.target,
        &active_target,
        &manifest.features,
        &selected_features,
        Some(package_id.clone()),
        native_registry_source_planning,
        native_git_source_planning,
    )?;
    let path_dependencies = native_path_dependencies(
        source_root,
        &manifest.dependencies,
        &manifest.features,
        &selected_features,
        Some(package_id.clone()),
        native_registry_source_planning,
        native_git_source_planning,
    )?;
    let build_dependencies = native_build_dependencies(
        source_root,
        &manifest.build_dependencies,
        &manifest.features,
        &selected_features,
        Some(package_id.clone()),
        native_registry_source_planning,
        native_git_source_planning,
    )?;
    let dev_dependencies = native_dev_dependencies(
        source_root,
        &manifest.dev_dependencies,
        Some(package_id.clone()),
        native_registry_source_planning,
        native_git_source_planning,
    )?;
    let (workspace_dependencies, selected_workspace_dependencies) = native_workspace_dependencies(
        workspace_root,
        source_root,
        &package_id,
        &manifest.dependencies,
        Some(package_id.clone()),
        native_registry_source_planning,
        native_git_source_planning,
    )?;
    let mut path_dependencies = path_dependencies;
    path_dependencies.extend(selected_target_cfg_dependencies);
    path_dependencies.extend(selected_workspace_dependencies);
    path_dependencies.sort();
    path_dependencies.dedup();
    let source_digest = native_registry_source_planning
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
        .map(|source| source.source_digest.clone())
        .or_else(|| {
            native_git_source_planning
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.source_digest.clone())
        })
        .or_else(|| {
            source_closure
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.source_digest.clone())
        })
        .unwrap_or_else(|| SourceDigest {
            algorithm: "missing".to_string(),
            value: "missing".to_string(),
        });
    Ok(NativePackagePlanningSummary {
        package_id,
        name: package.name.clone(),
        version: package_version.clone(),
        manifest_path: normalize_path_string(manifest_path),
        links: package.links.clone(),
        cargo_package_env: native_cargo_package_env(package, &workspace_package, &package_version),
        selected_features,
        targets,
        path_dependencies,
        build_dependencies,
        dev_dependencies,
        target_cfg_dependencies,
        workspace_dependencies,
        source_digest,
    })
}

fn native_cargo_package_env(
    package: &NativeManifestPackage,
    workspace_package: &NativeWorkspacePackage,
    package_version: &str,
) -> BTreeMap<String, String> {
    let (major, minor, patch, pre) = cargo_package_version_components(package_version);
    BTreeMap::from([
        (
            CARGO_PKG_AUTHORS_ENV.to_string(),
            native_manifest_authors_env(&package.authors, &workspace_package.authors),
        ),
        (
            CARGO_PKG_DESCRIPTION_ENV.to_string(),
            native_manifest_string_env(&package.description, &workspace_package.description),
        ),
        (
            CARGO_PKG_HOMEPAGE_ENV.to_string(),
            native_manifest_string_env(&package.homepage, &workspace_package.homepage),
        ),
        (
            CARGO_PKG_LICENSE_ENV.to_string(),
            native_manifest_string_env(&package.license, &workspace_package.license),
        ),
        (
            CARGO_PKG_LICENSE_FILE_ENV.to_string(),
            native_manifest_string_env(&package.license_file, &workspace_package.license_file),
        ),
        (BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV.to_string(), package.links.clone().unwrap_or_default()),
        (BUILD_SCRIPT_CARGO_PKG_NAME_ENV.to_string(), package.name.clone()),
        (
            CARGO_PKG_README_ENV.to_string(),
            native_manifest_string_env(&package.readme, &workspace_package.readme),
        ),
        (
            CARGO_PKG_REPOSITORY_ENV.to_string(),
            native_manifest_string_env(&package.repository, &workspace_package.repository),
        ),
        (
            CARGO_PKG_RUST_VERSION_ENV.to_string(),
            native_manifest_string_env(&package.rust_version, &workspace_package.rust_version),
        ),
        (CARGO_PKG_VERSION_ENV.to_string(), package_version.to_string()),
        (CARGO_PKG_VERSION_MAJOR_ENV.to_string(), major),
        (CARGO_PKG_VERSION_MINOR_ENV.to_string(), minor),
        (CARGO_PKG_VERSION_PATCH_ENV.to_string(), patch),
        (CARGO_PKG_VERSION_PRE_ENV.to_string(), pre),
    ])
}

fn cargo_package_version_components(version: &str) -> (String, String, String, String) {
    let (without_build, _) = version.split_once('+').unwrap_or((version, ""));
    let (core, pre) = without_build.split_once('-').unwrap_or((without_build, ""));
    let mut parts = core.split('.');
    let major = parts.next().unwrap_or_default().to_string();
    let minor = parts.next().unwrap_or_default().to_string();
    let patch = parts.next().unwrap_or_default().to_string();
    (major, minor, patch, pre.to_string())
}

fn native_cargo_package_env_workspace(
    workspace_root: &Path,
    package: &NativeManifestPackage,
) -> Result<NativeWorkspacePackage, NativePackagePlanningBlocker> {
    if !native_package_env_uses_workspace(package) {
        return Ok(NativeWorkspacePackage::default());
    }
    let root_manifest_path = workspace_root.join("Cargo.toml");
    let root_manifest = read_native_manifest(&root_manifest_path)
        .map_err(|message| native_blocker(None, "unreadable-workspace-package-manifest", &message))?;
    Ok(root_manifest.workspace.map(|workspace| workspace.package).unwrap_or_default())
}

fn native_package_env_uses_workspace(package: &NativeManifestPackage) -> bool {
    native_manifest_uses_workspace(&package.authors)
        || native_manifest_uses_workspace(&package.description)
        || native_manifest_uses_workspace(&package.homepage)
        || native_manifest_uses_workspace(&package.license)
        || native_manifest_uses_workspace(&package.license_file)
        || native_manifest_uses_workspace(&package.readme)
        || native_manifest_uses_workspace(&package.repository)
        || native_manifest_uses_workspace(&package.rust_version)
}

fn native_manifest_string_env(value: &Option<toml::Value>, workspace_value: &Option<toml::Value>) -> String {
    native_manifest_env_value(value, workspace_value)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn native_manifest_authors_env(value: &Option<toml::Value>, workspace_value: &Option<toml::Value>) -> String {
    let Some(value) = native_manifest_env_value(value, workspace_value) else {
        return String::new();
    };
    if let Some(author) = value.as_str() {
        return author.to_string();
    }
    value.as_array().into_iter().flatten().filter_map(toml::Value::as_str).collect::<Vec<_>>().join(":")
}

fn native_manifest_env_value<'a>(
    value: &'a Option<toml::Value>,
    workspace_value: &'a Option<toml::Value>,
) -> Option<&'a toml::Value> {
    let value = value.as_ref()?;
    if native_manifest_value_uses_workspace(value) {
        return workspace_value.as_ref();
    }
    Some(value)
}

fn native_manifest_uses_workspace(value: &Option<toml::Value>) -> bool {
    value.as_ref().is_some_and(native_manifest_value_uses_workspace)
}

fn native_manifest_value_uses_workspace(value: &toml::Value) -> bool {
    value
        .as_table()
        .is_some_and(|table| table.len() == 1 && table.get("workspace").and_then(toml::Value::as_bool) == Some(true))
}

fn native_package_edition(
    workspace_root: &Path,
    package_name: &str,
    edition: &NativeManifestInheritedString,
) -> Result<String, NativePackagePlanningBlocker> {
    if let Some(value) = &edition.value {
        return Ok(value.clone());
    }
    if !edition.workspace {
        return Ok(DEFAULT_RUST_EDITION.to_string());
    }
    let root_manifest_path = workspace_root.join("Cargo.toml");
    let root_manifest = read_native_manifest(&root_manifest_path)
        .map_err(|message| native_blocker(None, "unreadable-workspace-package-manifest", &message))?;
    let Some(workspace_edition) = root_manifest.workspace.and_then(|workspace| workspace.package.edition) else {
        return Err(native_blocker(
            None,
            "missing-workspace-package-edition",
            &format!("package `{package_name}` inherits workspace edition but workspace.package.edition is missing"),
        ));
    };
    Ok(workspace_edition)
}

fn native_package_version(
    workspace_root: &Path,
    package_name: &str,
    version: &NativeManifestInheritedString,
) -> Result<String, NativePackagePlanningBlocker> {
    if let Some(value) = &version.value {
        return Ok(value.clone());
    }
    if !version.workspace {
        return Err(native_blocker(
            None,
            "missing-package-version",
            &format!("package `{package_name}` lacks a literal version or workspace version inheritance"),
        ));
    }
    let root_manifest_path = workspace_root.join("Cargo.toml");
    let root_manifest = read_native_manifest(&root_manifest_path)
        .map_err(|message| native_blocker(None, "unreadable-workspace-package-manifest", &message))?;
    let Some(workspace_version) = root_manifest.workspace.and_then(|workspace| workspace.package.version) else {
        return Err(native_blocker(
            None,
            "missing-workspace-package-version",
            &format!(
                "package `{package_name}` inherits version from [workspace.package], but no workspace package version is declared"
            ),
        ));
    };
    Ok(workspace_version)
}

fn parse_native_manifest_text(path_label: &str, text: &str) -> Result<NativeManifest, String> {
    debug_assert!(!path_label.is_empty());
    toml::from_str(text).map_err(|err| format!("parsing manifest {path_label}: {err}"))
}

fn read_native_manifest(path: &Path) -> Result<NativeManifest, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("reading manifest {}: {err}", path.display()))?;
    parse_native_manifest_text(&path.display().to_string(), &text)
}

fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
    let root_manifest_path = root.join("Cargo.toml");
    let root_manifest_input = read_native_text_input(&root_manifest_path, "root manifest")?;
    let root_manifest = parse_native_manifest_text(&root_manifest_input.path, &root_manifest_input.text)?;
    let mut blockers = Vec::new();
    let mut manifest_paths = vec![root_manifest_path.clone()];
    manifest_paths.extend(native_workspace_manifest_paths_from_root_manifest(
        root,
        &root_manifest_path,
        &root_manifest,
        &mut blockers,
    ));
    if let Some(blocker) = blockers.first() {
        return Err(format!("collecting native manifest inputs failed: {}: {}", blocker.class, blocker.message));
    }
    manifest_paths.sort();
    manifest_paths.dedup();
    let mut manifests = Vec::new();
    for manifest_path in manifest_paths {
        manifests.push(read_native_text_input(&manifest_path, "manifest")?);
    }
    let lockfile = read_native_text_input(&root.join("Cargo.lock"), "lockfile")?;
    Ok(NativeManifestLockInputTexts { manifests, lockfile })
}

fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
    debug_assert!(!label.is_empty());
    let text = fs::read_to_string(path).map_err(|err| format!("reading native {label} {}: {err}", path.display()))?;
    Ok(NativeTextInput {
        path: normalize_path_string(path),
        text,
    })
}

fn native_manifest_lock_unsupported_blockers(
    inputs: &NativeManifestLockInputTexts,
) -> Vec<NativeManifestLockUnsupportedBlocker> {
    let mut blockers = Vec::new();
    for manifest in &inputs.manifests {
        blockers.extend(native_manifest_unsupported_blockers(manifest));
    }
    blockers.extend(native_lockfile_unsupported_blockers(&inputs.lockfile));
    blockers.sort();
    blockers.dedup();
    blockers
}

fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
    let mut blockers = Vec::new();
    let parsed = match input.text.parse::<toml::Value>() {
        Ok(parsed) => parsed,
        Err(err) => {
            blockers.push(native_manifest_lock_blocker(
                &input.path,
                "malformed-manifest",
                &format!("manifest failed TOML parsing: {err}"),
            ));
            return blockers;
        }
    };
    if parsed.get("patch").is_some() {
        blockers.push(native_manifest_lock_blocker(
            &input.path,
            "unsupported-manifest-patch",
            "native manifest planning does not yet implement [patch] source overrides",
        ));
    }
    if parsed.get("replace").is_some() {
        blockers.push(native_manifest_lock_blocker(
            &input.path,
            "unsupported-manifest-replace",
            "native manifest planning does not yet implement [replace] source overrides",
        ));
    }
    blockers.extend(native_target_table_unsupported_blockers(&input.path, parsed.get("target")));
    blockers
}

fn native_target_table_unsupported_blockers(
    path: &str,
    target_table: Option<&toml::Value>,
) -> Vec<NativeManifestLockUnsupportedBlocker> {
    let mut blockers = Vec::new();
    let Some(targets) = target_table.and_then(toml::Value::as_table) else {
        return blockers;
    };
    for (cfg, target) in targets {
        let Some(table) = target.as_table() else {
            blockers.push(native_manifest_lock_blocker(
                path,
                "unsupported-target-table",
                &format!("target cfg `{cfg}` is not a table"),
            ));
            continue;
        };
        for key in table.keys() {
            if key != "dependencies" {
                blockers.push(native_manifest_lock_blocker(
                    path,
                    "unsupported-target-table",
                    &format!("target cfg `{cfg}` key `{key}` is outside the supported dependency-only fragment"),
                ));
            }
        }
    }
    blockers
}

fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
    let mut blockers = Vec::new();
    let facts = match parse_native_lockfile_text(&input.path, &input.text) {
        Ok(facts) => facts,
        Err(err) => {
            blockers.push(native_manifest_lock_blocker(&input.path, "malformed-lockfile", &err));
            return blockers;
        }
    };
    for package in facts.packages {
        if let Some(source) = package.source.as_deref() {
            if !lock_source_supported(source) {
                blockers.push(native_manifest_lock_blocker(
                    &input.path,
                    "unsupported-lockfile-source",
                    &format!("lock package `{}` uses unsupported source `{source}`", package.name),
                ));
            }
        }
        for dependency in package.dependencies {
            if dependency.name.is_empty() {
                blockers.push(native_manifest_lock_blocker(
                    &input.path,
                    "unsupported-lockfile-dependency-edge",
                    &format!("lock package `{}` has an empty dependency edge `{}`", package.name, dependency.raw),
                ));
            }
        }
    }
    blockers
}

fn lock_source_supported(source: &str) -> bool {
    source.starts_with(CARGO_REGISTRY_SOURCE_PREFIX) || source.starts_with(CARGO_GIT_SOURCE_PREFIX)
}

fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
    debug_assert!(!path.is_empty());
    debug_assert!(!class.is_empty());
    NativeManifestLockUnsupportedBlocker {
        path: path.to_string(),
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn native_targets_for_manifest(
    source_root: &Path,
    package_name: &str,
    package_version: &str,
    package_edition: &str,
    manifest: &NativeManifest,
) -> Result<Vec<NativeTargetPlanningSummary>, NativePackagePlanningBlocker> {
    let mut targets = Vec::new();
    if let Some(build_script) = native_manifest_build_script(source_root, manifest) {
        let path = source_root.join(&build_script);
        let target_name = build_script_target_name(&build_script);
        push_native_target(&mut targets, &target_name, "custom-build", &path, package_edition)?;
    }
    if let Some(lib) = &manifest.lib {
        let kind = if lib.proc_macro { "proc-macro" } else { "lib" };
        let path = source_root.join(lib.path.as_deref().unwrap_or("src/lib.rs"));
        push_native_target(&mut targets, lib.name.as_deref().unwrap_or(package_name), kind, &path, package_edition)?;
    } else {
        let path = source_root.join("src/lib.rs");
        if path.is_file() {
            push_native_target(&mut targets, package_name, "lib", &path, package_edition)?;
        }
    }
    if manifest.bin.is_empty() {
        let path = source_root.join("src/main.rs");
        if path.is_file() {
            push_native_target(&mut targets, package_name, "bin", &path, package_edition)?;
        }
    } else {
        for bin in &manifest.bin {
            let name = bin.name.as_deref().unwrap_or(package_name);
            let default_path = format!("src/bin/{name}.rs");
            let path = source_root.join(bin.path.as_deref().unwrap_or(&default_path));
            push_native_target(&mut targets, name, "bin", &path, package_edition)?;
        }
    }
    if targets.is_empty() {
        return Err(native_blocker(
            Some(cargo_path_package_id(package_name, package_version)),
            "missing-supported-target",
            "native package/target fragment found no readable lib/bin target source",
        ));
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}

fn package_root_from_manifest_path(manifest_path: &str) -> String {
    Path::new(manifest_path).parent().map(normalize_path_string).unwrap_or_else(|| ".".to_string())
}

fn native_manifest_build_script(source_root: &Path, manifest: &NativeManifest) -> Option<String> {
    let package_build = manifest.package.as_ref().and_then(|package| package.build.as_ref());
    if let Some(value) = package_build.or(manifest.build.as_ref()) {
        if value.as_bool() == Some(false) {
            return None;
        }
        if let Some(path) = value.as_str() {
            return Some(path.to_string());
        }
    }
    source_root.join("build.rs").is_file().then(|| "build.rs".to_string())
}

fn build_script_target_name(_build_script: &str) -> String {
    BUILD_SCRIPT_TARGET_NAME.to_string()
}

fn push_native_target(
    targets: &mut Vec<NativeTargetPlanningSummary>,
    name: &str,
    kind: &str,
    path: &Path,
    edition: &str,
) -> Result<(), NativePackagePlanningBlocker> {
    if !path.is_file() {
        return Err(native_blocker(
            None,
            "missing-target-source",
            &format!("target `{name}` source {} is not readable", path.display()),
        ));
    }
    targets.push(NativeTargetPlanningSummary {
        name: name.to_string(),
        kind: kind.to_string(),
        crate_name: rust_crate_name(name),
        source_path: normalize_path_string(path),
        edition: edition.to_string(),
    });
    Ok(())
}

fn native_path_dependencies(
    source_root: &Path,
    dependencies: &BTreeMap<String, toml::Value>,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<Vec<NativePathDependencySummary>, NativePackagePlanningBlocker> {
    let mut summaries = Vec::new();
    for (name, value) in dependencies {
        if dependency_uses_workspace(value) {
            continue;
        }
        if dependency_optional(value) && !optional_dependency_selected(name, feature_defs, selected_features) {
            continue;
        }
        let manifest_path = if let Some(path) = dependency_path(value) {
            source_root.join(path).join("Cargo.toml")
        } else {
            let resolved =
                native_dependency_source(name, value, native_registry_source_planning, native_git_source_planning)
                    .map_err(|err| native_blocker(package_id.clone(), err.class, &err.message))?;
            if let Some(source) = resolved {
                PathBuf::from(source)
            } else if dependency_optional(value) {
                continue;
            } else {
                return Err(native_blocker(
                    package_id,
                    "unsupported-non-path-dependency",
                    &format!(
                        "dependency `{name}` is outside the bounded path, declared-registry, or captured-git dependency fragment"
                    ),
                ));
            }
        };
        if !manifest_path.is_file() {
            return Err(native_blocker(
                package_id,
                "missing-path-dependency-manifest",
                &format!("dependency `{name}` manifest {} is not readable", manifest_path.display()),
            ));
        }
        summaries.push(NativePathDependencySummary {
            name: name.clone(),
            manifest_path: normalize_path_string(&manifest_path),
        });
    }
    summaries.sort();
    summaries.dedup();
    Ok(summaries)
}

fn native_build_dependencies(
    source_root: &Path,
    dependencies: &BTreeMap<String, toml::Value>,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<Vec<NativePathDependencySummary>, NativePackagePlanningBlocker> {
    for (name, value) in dependencies {
        if dependency_has_unsupported_build_options(value) {
            return Err(native_blocker(
                package_id,
                "unsupported-build-dependency-options",
                &format!("build dependency `{name}` uses unsupported target or workspace behavior"),
            ));
        }
    }
    native_path_dependencies(
        source_root,
        dependencies,
        feature_defs,
        selected_features,
        package_id,
        native_registry_source_planning,
        native_git_source_planning,
    )
}

fn native_dev_dependencies(
    source_root: &Path,
    dependencies: &BTreeMap<String, toml::Value>,
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<Vec<NativePathDependencySummary>, NativePackagePlanningBlocker> {
    // Dev-dependency feature/default-feature/test resolver behavior is not part of the normal build
    // topology. Record only explicitly readable path or declared-registry source material for the
    // dedicated dev-test rails; otherwise ignore test-only dependencies so they cannot block build-mode
    // package/target facts or normal self execution.
    let mut summaries = Vec::new();
    for (name, value) in dependencies {
        let manifest_path = if let Some(path) = dependency_path(value) {
            source_root.join(path).join("Cargo.toml")
        } else {
            match native_dependency_source(name, value, native_registry_source_planning, native_git_source_planning) {
                Ok(Some(source)) => PathBuf::from(source),
                Ok(None) => continue,
                Err(err) => return Err(native_blocker(package_id.clone(), err.class, &err.message)),
            }
        };
        if !manifest_path.is_file() {
            continue;
        }
        summaries.push(NativePathDependencySummary {
            name: name.clone(),
            manifest_path: normalize_path_string(&manifest_path),
        });
    }
    summaries.sort();
    summaries.dedup();
    Ok(summaries)
}

fn dependency_has_unsupported_build_options(value: &toml::Value) -> bool {
    let Some(table) = value.as_table() else {
        return false;
    };
    ["target", "workspace"].iter().any(|key| table.contains_key(*key))
}

fn dependency_optional(value: &toml::Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("optional"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn native_optional_dependency_names(dependencies: &BTreeMap<String, toml::Value>) -> BTreeSet<String> {
    dependencies
        .iter()
        .filter(|(_, value)| dependency_optional(value))
        .map(|(name, _)| name.clone())
        .collect()
}

fn native_workspace_dependencies(
    workspace_root: &Path,
    source_root: &Path,
    member_package_id: &str,
    dependencies: &BTreeMap<String, toml::Value>,
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<(Vec<NativeWorkspaceDependencySummary>, Vec<NativePathDependencySummary>), NativePackagePlanningBlocker> {
    if !dependencies.values().any(dependency_uses_workspace) {
        return Ok((Vec::new(), Vec::new()));
    }
    let workspace_manifest_path = workspace_root.join("Cargo.toml");
    let workspace_manifest = read_native_manifest(&workspace_manifest_path)
        .map_err(|message| native_blocker(package_id.clone(), "unreadable-workspace-manifest", &message))?;
    let workspace_dependencies =
        workspace_manifest.workspace.as_ref().map(|workspace| &workspace.dependencies).ok_or_else(|| {
            native_blocker(
                package_id.clone(),
                "missing-workspace-section",
                "workspace dependency inheritance requires a root [workspace] table",
            )
        })?;

    let mut summaries = Vec::new();
    let mut selected = Vec::new();
    for (name, value) in dependencies {
        if !dependency_uses_workspace(value) {
            continue;
        }
        let inherited = workspace_dependencies.get(name).ok_or_else(|| {
            native_blocker(
                package_id.clone(),
                "missing-workspace-dependency",
                &format!("dependency `{name}` uses workspace inheritance but root [workspace.dependencies] has no matching key"),
            )
        })?;
        if dependency_has_unsupported_workspace_member_options(value)
            || dependency_has_unsupported_workspace_inherited_options(inherited)
        {
            return Err(native_blocker(
                package_id.clone(),
                "unsupported-workspace-dependency-options",
                &format!(
                    "workspace dependency `{name}` uses unsupported inherited feature/default-feature/platform behavior"
                ),
            ));
        }
        let inherited_features = dependency_features(inherited)?;
        let inherited_default_features = dependency_default_features(inherited)?;
        let manifest_path = if let Some(path) = dependency_path(inherited) {
            source_root.join(path).join("Cargo.toml")
        } else {
            let resolved =
                native_dependency_source(name, inherited, native_registry_source_planning, native_git_source_planning)
                    .map_err(|err| native_blocker(package_id.clone(), err.class, &err.message))?;
            if let Some(source) = resolved {
                PathBuf::from(source)
            } else {
                return Err(native_blocker(
                    package_id.clone(),
                    "unsupported-workspace-dependency-source",
                    &format!(
                        "workspace dependency `{name}` is outside the bounded path, declared-registry, or captured-git fragment"
                    ),
                ));
            }
        };
        if !manifest_path.is_file() {
            return Err(native_blocker(
                package_id.clone(),
                "missing-workspace-dependency-manifest",
                &format!("workspace dependency `{name}` manifest {} is not readable", manifest_path.display()),
            ));
        }
        let inherited_package_name = dependency_package_name(name, inherited).to_string();
        let normalized_manifest_path = normalize_path_string(&manifest_path);
        summaries.push(NativeWorkspaceDependencySummary {
            workspace_root: normalize_path_string(workspace_root),
            member_package_id: member_package_id.to_string(),
            dependency_key: name.clone(),
            inherited_package_name,
            inherited_features,
            inherited_default_features,
            decision: "selected".to_string(),
            manifest_path: Some(normalized_manifest_path.clone()),
            blocker_class: None,
        });
        selected.push(NativePathDependencySummary {
            name: name.clone(),
            manifest_path: normalized_manifest_path,
        });
    }
    summaries.sort();
    summaries.dedup();
    selected.sort();
    selected.dedup();
    Ok((summaries, selected))
}

fn dependency_uses_workspace(value: &toml::Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn dependency_has_unsupported_workspace_member_options(value: &toml::Value) -> bool {
    let Some(table) = value.as_table() else {
        return false;
    };
    ["features", "default-features", "optional", "target"].iter().any(|key| table.contains_key(*key))
}

fn dependency_has_unsupported_workspace_inherited_options(value: &toml::Value) -> bool {
    let Some(table) = value.as_table() else {
        return false;
    };
    ["optional", "target", "workspace"].iter().any(|key| table.contains_key(*key))
}

fn dependency_features(value: &toml::Value) -> Result<Vec<String>, NativePackagePlanningBlocker> {
    let Some(features) = value.as_table().and_then(|table| table.get("features")) else {
        return Ok(Vec::new());
    };
    let Some(array) = features.as_array() else {
        return Err(native_blocker(
            None,
            "unsupported-workspace-dependency-options",
            "workspace dependency features must be an array of strings",
        ));
    };
    let mut out = Vec::new();
    for feature in array {
        let Some(feature) = feature.as_str() else {
            return Err(native_blocker(
                None,
                "unsupported-workspace-dependency-options",
                "workspace dependency features must be an array of strings",
            ));
        };
        out.push(feature.to_string());
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn dependency_default_features(value: &toml::Value) -> Result<Option<bool>, NativePackagePlanningBlocker> {
    let Some(default_features) = value.as_table().and_then(|table| table.get("default-features")) else {
        return Ok(None);
    };
    default_features.as_bool().map(Some).ok_or_else(|| {
        native_blocker(
            None,
            "unsupported-workspace-dependency-options",
            "workspace dependency default-features must be a boolean",
        )
    })
}

fn dependency_package_name<'a>(dependency_name: &'a str, value: &'a toml::Value) -> &'a str {
    value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_str)
        .unwrap_or(dependency_name)
}

fn native_target_cfg_dependencies(
    source_root: &Path,
    target_tables: &BTreeMap<String, NativeManifestTarget>,
    active_target: &str,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &NativeGitSourcePlanningSummary,
) -> Result<(Vec<NativeTargetCfgDependencySummary>, Vec<NativePathDependencySummary>), NativePackagePlanningBlocker> {
    let mut cfg_facts = Vec::new();
    let mut selected_dependencies = Vec::new();
    for (cfg_expr, target) in target_tables {
        let cfg_selected = evaluate_supported_target_cfg(cfg_expr, active_target).ok_or_else(|| {
            native_blocker(
                package_id.clone(),
                "unsupported-target-cfg-surface",
                &format!("target cfg `{cfg_expr}` is outside the bounded native target-cfg fragment"),
            )
        })?;
        for (name, value) in &target.dependencies {
            let mut dependency_selected = cfg_selected
                && (!dependency_optional(value) || optional_dependency_selected(name, feature_defs, selected_features));
            let manifest_path = if dependency_selected {
                if let Some(path) = dependency_path(value) {
                    Some(source_root.join(path).join("Cargo.toml"))
                } else {
                    let resolved = native_dependency_source(
                        name,
                        value,
                        native_registry_source_planning,
                        native_git_source_planning,
                    )
                    .map_err(|err| native_blocker(package_id.clone(), err.class, &err.message))?;
                    if let Some(source) = resolved {
                        Some(PathBuf::from(source))
                    } else if dependency_optional(value) {
                        None
                    } else {
                        return Err(native_blocker(
                            package_id.clone(),
                            "unsupported-target-cfg-dependency",
                            &format!(
                                "target cfg dependency `{name}` is outside the bounded path, declared-registry, or captured-git dependency fragment"
                            ),
                        ));
                    }
                }
            } else {
                None
            };
            if manifest_path.is_none() && dependency_optional(value) {
                dependency_selected = false;
            }
            if let Some(path) = &manifest_path {
                if !path.is_file() {
                    return Err(native_blocker(
                        package_id.clone(),
                        "missing-target-cfg-dependency-manifest",
                        &format!("target cfg dependency `{name}` manifest {} is not readable", path.display()),
                    ));
                }
            }
            let normalized_manifest_path = manifest_path.as_ref().map(|path| normalize_path_string(path));
            cfg_facts.push(NativeTargetCfgDependencySummary {
                cfg: cfg_expr.clone(),
                active_target: active_target.to_string(),
                decision: if dependency_selected {
                    "selected"
                } else if cfg_selected {
                    "not-selected-optional"
                } else {
                    "not-selected"
                }
                .to_string(),
                name: name.clone(),
                manifest_path: normalized_manifest_path.clone(),
                blocker_class: None,
            });
            if dependency_selected {
                if let Some(path) = normalized_manifest_path {
                    selected_dependencies.push(NativePathDependencySummary {
                        name: name.clone(),
                        manifest_path: path,
                    });
                }
            }
        }
    }
    cfg_facts.sort();
    cfg_facts.dedup();
    selected_dependencies.sort();
    selected_dependencies.dedup();
    Ok((cfg_facts, selected_dependencies))
}

fn evaluate_supported_target_cfg(cfg_expr: &str, active_target: &str) -> Option<bool> {
    let trimmed = cfg_expr.trim();
    if !trimmed.starts_with("cfg(") {
        return Some(trimmed == active_target);
    }
    let inner = trimmed.strip_prefix("cfg(")?.strip_suffix(')')?.trim();
    evaluate_cfg_inner(inner, active_target)
}

fn evaluate_cfg_inner(expr: &str, active_target: &str) -> Option<bool> {
    let trimmed = expr.trim();
    if let Some(inner) = cfg_call_arg(trimmed, "not") {
        return evaluate_cfg_inner(inner, active_target).map(|value| !value);
    }
    if let Some(inner) = cfg_call_arg(trimmed, "any") {
        let args = split_cfg_args(inner)?;
        for arg in args {
            if evaluate_cfg_inner(arg, active_target)? {
                return Some(true);
            }
        }
        return Some(false);
    }
    if let Some(inner) = cfg_call_arg(trimmed, "all") {
        let args = split_cfg_args(inner)?;
        for arg in args {
            if !evaluate_cfg_inner(arg, active_target)? {
                return Some(false);
            }
        }
        return Some(true);
    }
    if trimmed == "unix" {
        return Some(target_is_unix(active_target));
    }
    if trimmed == "windows" {
        return Some(target_os_from_triple(active_target) == "windows");
    }
    if let Some((key, value)) = parse_cfg_key_value(trimmed) {
        return cfg_key_value_matches(key, value, active_target);
    }
    known_inactive_cfg_atom(trimmed).then_some(false)
}

fn known_inactive_cfg_atom(atom: &str) -> bool {
    matches!(
        atom,
        "loom"
            | "miri"
            | "criterion"
            | "compiletests"
            | "crossbeam_loom"
            | "diatomic_waker_loom"
            | "tokio_unstable"
            | "tracing_unstable"
            | "valgrind"
            | "windows_raw_dylib"
            | "rustix_use_libc"
            | "rustix_use_experimental_asm"
    )
}

fn cfg_call_arg<'a>(expr: &'a str, name: &str) -> Option<&'a str> {
    expr.strip_prefix(name)?.strip_prefix('(')?.strip_suffix(')')
}

fn split_cfg_args(args: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut start = 0usize;
    for (index, ch) in args.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth = depth.checked_add(1)?,
            ')' if !in_string => depth = depth.checked_sub(1)?,
            ',' if !in_string && depth == 0 => {
                let part = args[start..index].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if in_string || depth != 0 {
        return None;
    }
    let tail = args[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    Some(parts)
}

fn parse_cfg_key_value(expr: &str) -> Option<(&str, &str)> {
    let (left, right) = expr.split_once('=')?;
    Some((left.trim(), right.trim().trim_matches('"')))
}

fn cfg_key_value_matches(key: &str, value: &str, active_target: &str) -> Option<bool> {
    let matched = match key {
        "target_os" => value == target_os_from_triple(active_target),
        "target_arch" => value == target_arch_from_triple(active_target),
        "target_family" => value == target_family_from_triple(active_target),
        "target_vendor" => value == target_vendor_from_triple(active_target),
        "target_env" => value == target_env_from_triple(active_target),
        "target_abi" => value == target_abi_from_triple(active_target),
        "target_endian" => value == target_endian_from_triple(active_target),
        "target_pointer_width" => value == target_pointer_width_from_triple(active_target),
        "target_has_atomic" => target_has_atomic(active_target, value),
        "feature" => false,
        key if key.ends_with("_backend") || key.starts_with("rustix_") || key == "getrandom_backend" => false,
        _ => return None,
    };
    Some(matched)
}

fn active_rust_target(options: &RustPlanOptions) -> String {
    options.targets.first().cloned().unwrap_or_else(host_target_triple)
}

fn host_target_triple() -> String {
    format!("{}-unknown-{}-gnu", std::env::consts::ARCH, match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    })
}

fn target_is_unix(active_target: &str) -> bool {
    matches!(
        target_os_from_triple(active_target),
        "linux" | "darwin" | "freebsd" | "netbsd" | "openbsd" | "dragonfly" | "android"
    )
}

fn target_os_from_triple(active_target: &str) -> &str {
    if active_target.contains("linux") {
        "linux"
    } else if active_target.contains("darwin") || active_target.contains("apple") {
        "darwin"
    } else if active_target.contains("windows") || active_target.contains("msvc") {
        "windows"
    } else if active_target.contains("freebsd") {
        "freebsd"
    } else if active_target.contains("netbsd") {
        "netbsd"
    } else if active_target.contains("openbsd") {
        "openbsd"
    } else if active_target.contains("android") {
        "android"
    } else {
        "unknown"
    }
}

fn target_arch_from_triple(active_target: &str) -> &str {
    active_target.split('-').next().unwrap_or("unknown")
}

fn target_family_from_triple(active_target: &str) -> &str {
    if target_is_unix(active_target) {
        "unix"
    } else if target_os_from_triple(active_target) == "windows" {
        "windows"
    } else if active_target.contains("wasm") {
        "wasm"
    } else {
        "unknown"
    }
}

fn target_vendor_from_triple(active_target: &str) -> &str {
    let mut parts = active_target.split('-');
    let _arch = parts.next();
    parts.next().unwrap_or("unknown")
}

fn target_env_from_triple(active_target: &str) -> &str {
    if active_target.contains("musl") {
        "musl"
    } else if active_target.contains("msvc") {
        "msvc"
    } else if active_target.contains("gnu") {
        "gnu"
    } else {
        ""
    }
}

fn target_abi_from_triple(active_target: &str) -> &str {
    if active_target.contains("llvm") { "llvm" } else { "" }
}

fn target_endian_from_triple(_active_target: &str) -> &str {
    "little"
}

fn target_pointer_width_from_triple(active_target: &str) -> &str {
    match target_arch_from_triple(active_target) {
        "x86_64" | "aarch64" | "riscv64" | "powerpc64" | "s390x" | "wasm64" => "64",
        _ => "32",
    }
}

fn target_feature_env_from_triple(active_target: &str) -> &str {
    match target_arch_from_triple(active_target) {
        "x86_64" => "fxsr,sse,sse2,x87",
        "wasm32" => "bulk-memory,multivalue,mutable-globals,nontrapping-fptoint,reference-types,sign-ext",
        _ => "",
    }
}

fn build_script_profile_env(profile: &str) -> BTreeMap<String, String> {
    debug_assert!(!profile.is_empty());
    let release_like = matches!(profile, CARGO_PROFILE_RELEASE | CARGO_PROFILE_BENCH);
    let opt_level = if release_like {
        CARGO_OPT_LEVEL_RELEASE
    } else {
        CARGO_OPT_LEVEL_DEBUG
    };
    let debug = if release_like {
        CARGO_DEBUG_FALSE
    } else {
        CARGO_DEBUG_TRUE
    };
    BTreeMap::from([
        (BUILD_SCRIPT_OPT_LEVEL_ENV.to_string(), opt_level.to_string()),
        (BUILD_SCRIPT_DEBUG_ENV.to_string(), debug.to_string()),
        (BUILD_SCRIPT_NUM_JOBS_ENV.to_string(), MANTLE_DETERMINISTIC_NUM_JOBS.to_string()),
    ])
}

fn build_script_target_cfg_env(active_target: &str) -> BTreeMap<String, String> {
    debug_assert!(!active_target.is_empty());
    let mut env = BTreeMap::from([
        (CARGO_CFG_TARGET_ABI_ENV.to_string(), target_abi_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_ARCH_ENV.to_string(), target_arch_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_ENDIAN_ENV.to_string(), target_endian_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_ENV_ENV.to_string(), target_env_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_FAMILY_ENV.to_string(), target_family_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_FEATURE_ENV.to_string(), target_feature_env_from_triple(active_target).to_string()),
        (CARGO_CFG_TARGET_OS_ENV.to_string(), target_os_from_triple(active_target).to_string()),
        (
            CARGO_CFG_TARGET_POINTER_WIDTH_ENV.to_string(),
            target_pointer_width_from_triple(active_target).to_string(),
        ),
        (CARGO_CFG_TARGET_VENDOR_ENV.to_string(), target_vendor_from_triple(active_target).to_string()),
    ]);
    if target_is_unix(active_target) {
        env.insert(CARGO_CFG_UNIX_ENV.to_string(), String::new());
    }
    if target_os_from_triple(active_target) == "windows" {
        env.insert(CARGO_CFG_WINDOWS_ENV.to_string(), String::new());
    }
    debug_assert!(env.contains_key(CARGO_CFG_TARGET_ARCH_ENV));
    debug_assert!(env.contains_key(CARGO_CFG_TARGET_POINTER_WIDTH_ENV));
    env
}

fn target_has_atomic(active_target: &str, width: &str) -> bool {
    let pointer_width = target_pointer_width_from_triple(active_target).parse::<u32>().unwrap_or(0);
    if width == "ptr" {
        return pointer_width > 0;
    }
    width.parse::<u32>().map(|width| width <= pointer_width).unwrap_or(false)
}

fn optional_dependency_selected(
    dependency_name: &str,
    feature_defs: &BTreeMap<String, Vec<String>>,
    selected_features: &[String],
) -> bool {
    selected_features.iter().any(|feature| {
        feature == dependency_name
            || feature_defs
                .get(feature)
                .map(|entries| entries.iter().any(|entry| feature_entry_selects_dependency(entry, dependency_name)))
                .unwrap_or(false)
    })
}

fn feature_entry_selects_dependency(entry: &str, dependency_name: &str) -> bool {
    if entry == dependency_name || entry == format!("dep:{dependency_name}") {
        return true;
    }
    let Some((entry_dependency, _feature)) = entry.split_once('/') else {
        return false;
    };
    let entry_dependency = entry_dependency.trim_start_matches("dep:").trim_end_matches('?');
    entry_dependency == dependency_name
}

fn native_dependency_source<'a>(
    dependency_name: &str,
    value: &toml::Value,
    native_registry_source_planning: &'a NativeRegistrySourcePlanningSummary,
    native_git_source_planning: &'a NativeGitSourcePlanningSummary,
) -> Result<Option<&'a str>, DependencySourceResolutionError> {
    if dependency_git_url(value).is_some() {
        return git_dependency_source(dependency_name, value, native_git_source_planning)
            .map(|source| Some(source.manifest_path.as_str()));
    }
    registry_dependency_source(dependency_name, value, native_registry_source_planning)
        .map(|source| source.map(|source| source.manifest_path.as_str()))
}

fn registry_dependency_source<'a>(
    dependency_name: &str,
    value: &toml::Value,
    native_registry_source_planning: &'a NativeRegistrySourcePlanningSummary,
) -> Result<Option<&'a NativeRegistrySourceSummary>, DependencySourceResolutionError> {
    debug_assert!(!dependency_name.is_empty());
    if !native_registry_source_planning.ready {
        return Ok(None);
    }
    let package_name = dependency_package_name(dependency_name, value);
    let candidates = native_registry_source_planning
        .sources
        .iter()
        .filter(|source| source.name == package_name)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(None);
    }
    if let Some(version_req) = dependency_version(value).map(str::trim).filter(|version| !version.is_empty()) {
        let normalized_req = dependency_exact_version(version_req);
        let matching = candidates
            .iter()
            .copied()
            .filter(|source| registry_version_req_matches(version_req, &source.version))
            .collect::<Vec<_>>();
        if let Some(selected) = newest_registry_source(matching.as_slice()) {
            return Ok(Some(selected));
        }
        if version_req.starts_with('=') && candidates.len() > 1 {
            return Err(dependency_source_error(
                "missing-registry-dependency-version",
                format!(
                    "registry dependency `{dependency_name}` requested `{package_name}` version `{normalized_req}` but available versions are {}",
                    registry_candidate_versions(&candidates)
                ),
            ));
        }
    }
    Ok(Some(candidates[0]))
}

fn dependency_exact_version(version_req: &str) -> &str {
    debug_assert!(!version_req.trim().is_empty());
    version_req.strip_prefix('=').map(str::trim).unwrap_or(version_req.trim())
}

fn registry_version_req_matches(version_req: &str, source_version: &str) -> bool {
    debug_assert!(!version_req.trim().is_empty());
    debug_assert!(!source_version.is_empty());
    let normalized_req = dependency_exact_version(version_req);
    if normalized_req == source_version {
        return true;
    }
    if version_req.trim().starts_with('=') {
        return false;
    }
    if let Some(matches) = registry_semver_range_req_matches(normalized_req, source_version) {
        return matches;
    }
    if let Some(matches) = registry_semver_caret_req_matches(normalized_req, source_version) {
        return matches;
    }
    registry_version_prefix_matches(normalized_req, source_version)
}

fn newest_registry_source<'a>(sources: &[&'a NativeRegistrySourceSummary]) -> Option<&'a NativeRegistrySourceSummary> {
    sources
        .iter()
        .copied()
        .max_by(|left, right| semver_sort_key(&left.version).cmp(&semver_sort_key(&right.version)))
}

fn semver_sort_key(version: &str) -> (u32, u32, u32) {
    semver_components(version)
        .map(|version| (version.major, version.minor, version.patch))
        .unwrap_or((0, 0, 0))
}

fn registry_semver_range_req_matches(version_req: &str, source_version: &str) -> Option<bool> {
    if !version_req.contains(',') && !version_req.contains('<') && !version_req.contains('>') {
        return None;
    }
    let source = semver_components(source_version)?;
    let mut saw_comparator = false;
    for comparator in version_req.split(',').map(str::trim).filter(|part| !part.is_empty()) {
        saw_comparator = true;
        if !semver_comparator_matches(comparator, source)? {
            return Some(false);
        }
    }
    Some(saw_comparator)
}

fn semver_comparator_matches(comparator: &str, source: SemverComponents) -> Option<bool> {
    let (operator, version) = if let Some(version) = comparator.strip_prefix(">=") {
        (">=", version.trim())
    } else if let Some(version) = comparator.strip_prefix("<=") {
        ("<=", version.trim())
    } else if let Some(version) = comparator.strip_prefix('>') {
        (">", version.trim())
    } else if let Some(version) = comparator.strip_prefix('<') {
        ("<", version.trim())
    } else if let Some(version) = comparator.strip_prefix('=') {
        ("=", version.trim())
    } else {
        return None;
    };
    let required = semver_components(version)?;
    let source_key = (source.major, source.minor, source.patch);
    let required_key = (required.major, required.minor, required.patch);
    Some(match operator {
        ">=" => source_key >= required_key,
        "<=" => source_key <= required_key,
        ">" => source_key > required_key,
        "<" => source_key < required_key,
        "=" => source_key == required_key,
        _ => false,
    })
}

fn registry_semver_caret_req_matches(version_req: &str, source_version: &str) -> Option<bool> {
    let req = semver_components(version_req)?;
    let source = semver_components(source_version)?;
    if !semver_source_meets_lower_bound(req, source) {
        return Some(false);
    }
    if req.major > 0 {
        return Some(source.major == req.major);
    }
    if req.minor > 0 {
        return Some(source.major == 0 && source.minor == req.minor);
    }
    Some(source.major == 0 && source.minor == 0 && source.patch == req.patch)
}

#[derive(Clone, Copy)]
struct SemverComponents {
    major: u32,
    minor: u32,
    patch: u32,
}

fn semver_components(version: &str) -> Option<SemverComponents> {
    const SEMVER_MAJOR_INDEX: usize = 0;
    const SEMVER_MINOR_INDEX: usize = 1;
    const SEMVER_PATCH_INDEX: usize = 2;
    let core = version.split(['-', '+']).next()?;
    let parts = core.split('.').collect::<Vec<_>>();
    let major = parts.get(SEMVER_MAJOR_INDEX)?.parse::<u32>().ok()?;
    let minor = parts.get(SEMVER_MINOR_INDEX).and_then(|part| part.parse::<u32>().ok()).unwrap_or(0);
    let patch = parts.get(SEMVER_PATCH_INDEX).and_then(|part| part.parse::<u32>().ok()).unwrap_or(0);
    Some(SemverComponents { major, minor, patch })
}

fn semver_source_meets_lower_bound(req: SemverComponents, source: SemverComponents) -> bool {
    (source.major, source.minor, source.patch) >= (req.major, req.minor, req.patch)
}

fn registry_version_prefix_matches(version_prefix: &str, source_version: &str) -> bool {
    debug_assert!(!version_prefix.is_empty());
    debug_assert!(!source_version.is_empty());
    let Some(suffix) = source_version.strip_prefix(version_prefix) else {
        return false;
    };
    suffix.starts_with('.') || suffix.starts_with('-') || suffix.starts_with('+')
}

fn registry_candidate_versions(candidates: &[&NativeRegistrySourceSummary]) -> String {
    debug_assert!(!candidates.is_empty());
    let mut versions = candidates.iter().map(|source| source.version.clone()).collect::<Vec<_>>();
    versions.sort();
    versions.dedup();
    versions.join(", ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DependencySourceResolutionError {
    class: &'static str,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitDependencySelector<'a> {
    package_name: &'a str,
    package_version: Option<&'a str>,
    normalized_url: &'a str,
    reference: Option<GitDependencyReference<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GitDependencyReference<'a> {
    Revision(&'a str),
    Tag(&'a str),
    Branch(&'a str),
}

fn git_dependency_source<'a>(
    dependency_name: &str,
    value: &toml::Value,
    native_git_source_planning: &'a NativeGitSourcePlanningSummary,
) -> Result<&'a NativeGitSourceSummary, DependencySourceResolutionError> {
    if !native_git_source_planning.ready {
        return Err(dependency_source_error(
            "native-git-source-planning-blocked",
            "native git source facts are not ready for dependency resolution",
        ));
    }
    let selector = git_dependency_selector(dependency_name, value)?;
    let identity_matches = native_git_source_planning
        .sources
        .iter()
        .filter(|source| git_source_identity_matches(source, &selector))
        .collect::<Vec<_>>();
    if identity_matches.is_empty() {
        return Err(dependency_source_error(
            "missing-captured-git-source-fact",
            format!(
                "git dependency `{dependency_name}` has no captured source fact for package `{}` at {}",
                selector.package_name, selector.normalized_url
            ),
        ));
    }
    let reference_matches = identity_matches
        .iter()
        .copied()
        .filter(|source| git_source_reference_matches(source, &selector))
        .collect::<Vec<_>>();
    if reference_matches.is_empty() {
        return Err(dependency_source_error(
            "git-dependency-reference-mismatch",
            format!(
                "git dependency `{dependency_name}` requested source identity not present in captured source facts"
            ),
        ));
    }
    if reference_matches.len() != 1 {
        return Err(dependency_source_error(
            "ambiguous-captured-git-source-fact",
            format!(
                "git dependency `{dependency_name}` matches multiple captured source facts; add an exact rev, tag, branch, or package version"
            ),
        ));
    }
    Ok(reference_matches[0])
}

fn dependency_git_url(value: &toml::Value) -> Option<&str> {
    value.as_table()?.get("git")?.as_str()
}

fn dependency_version(value: &toml::Value) -> Option<&str> {
    value.as_str().or_else(|| value.as_table()?.get("version")?.as_str())
}

fn git_dependency_selector<'a>(
    dependency_name: &'a str,
    value: &'a toml::Value,
) -> Result<GitDependencySelector<'a>, DependencySourceResolutionError> {
    let package_name = dependency_package_name(dependency_name, value);
    let package_version = dependency_version(value);
    let git_url = dependency_git_url(value).ok_or_else(|| {
        dependency_source_error("missing-git-dependency-url", format!("dependency `{dependency_name}` has no git URL"))
    })?;
    Ok(GitDependencySelector {
        package_name,
        package_version,
        normalized_url: normalized_git_dependency_url(git_url),
        reference: git_dependency_reference(value)?,
    })
}

fn git_dependency_reference<'a>(
    value: &'a toml::Value,
) -> Result<Option<GitDependencyReference<'a>>, DependencySourceResolutionError> {
    let Some(table) = value.as_table() else {
        return Ok(None);
    };
    let mut references = Vec::new();
    if let Some(rev) = table.get("rev").and_then(toml::Value::as_str) {
        references.push(GitDependencyReference::Revision(rev));
    }
    if let Some(tag) = table.get("tag").and_then(toml::Value::as_str) {
        references.push(GitDependencyReference::Tag(tag));
    }
    if let Some(branch) = table.get("branch").and_then(toml::Value::as_str) {
        references.push(GitDependencyReference::Branch(branch));
    }
    if references.len() > 1 {
        return Err(dependency_source_error(
            "unsupported-git-dependency-reference",
            "git dependency declares multiple ref selectors; expected exactly one of rev, tag, or branch",
        ));
    }
    Ok(references.into_iter().next())
}

fn git_source_identity_matches(source: &NativeGitSourceSummary, selector: &GitDependencySelector<'_>) -> bool {
    if source.name != selector.package_name {
        return false;
    }
    if let Some(package_version) = selector.package_version {
        if source.version != package_version {
            return false;
        }
    }
    normalized_git_source_url(&source.source) == Some(selector.normalized_url)
}

fn git_source_reference_matches(source: &NativeGitSourceSummary, selector: &GitDependencySelector<'_>) -> bool {
    match &selector.reference {
        None => true,
        Some(GitDependencyReference::Revision(rev)) => source.resolved_revision == *rev,
        Some(GitDependencyReference::Tag(tag)) => git_source_query_value(&source.source, "tag") == Some(*tag),
        Some(GitDependencyReference::Branch(branch)) => {
            git_source_query_value(&source.source, "branch") == Some(*branch)
        }
    }
}

fn git_source_query_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let without_prefix = source.strip_prefix("git+")?;
    let without_revision = without_prefix.split_once('#').map(|(prefix, _)| prefix).unwrap_or(without_prefix);
    let (_, query) = without_revision.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (candidate_key, value) = pair.split_once('=')?;
        (candidate_key == key).then_some(value)
    })
}

fn dependency_source_error(class: &'static str, message: impl Into<String>) -> DependencySourceResolutionError {
    DependencySourceResolutionError {
        class,
        message: message.into(),
    }
}

fn normalized_git_dependency_url(url: &str) -> &str {
    url.split_once('?').map(|(prefix, _)| prefix).unwrap_or(url)
}

fn normalized_git_source_url(source: &str) -> Option<&str> {
    let without_prefix = source.strip_prefix("git+")?;
    let without_revision = without_prefix.split_once('#').map(|(prefix, _)| prefix).unwrap_or(without_prefix);
    Some(normalized_git_dependency_url(without_revision))
}

fn dependency_path(value: &toml::Value) -> Option<&str> {
    value.as_table()?.get("path")?.as_str()
}

fn selected_package_ids_from_unit_graph(unit_graph: &Value) -> BTreeSet<String> {
    unit_graph
        .get("units")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|unit| unit.get("pkg_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

fn selected_features_by_package_from_unit_graph(unit_graph: &Value) -> BTreeMap<String, Vec<String>> {
    let mut features_by_package = BTreeMap::<String, BTreeSet<String>>::new();
    let Some(units) = unit_graph.get("units").and_then(Value::as_array) else {
        return BTreeMap::new();
    };
    for unit in units {
        let Some(package_id) = unit.get("pkg_id").and_then(Value::as_str) else {
            continue;
        };
        let entry = features_by_package.entry(package_id.to_string()).or_default();
        for feature in unit_string_array(unit, "features") {
            entry.insert(feature);
        }
    }
    features_by_package
        .into_iter()
        .map(|(package_id, features)| (package_id, features.into_iter().collect()))
        .collect()
}

fn native_selected_features(
    options: &RustPlanOptions,
    feature_defs: &BTreeMap<String, Vec<String>>,
    optional_dependencies: &BTreeSet<String>,
) -> Vec<String> {
    resolve_native_features(NativeFeatureResolutionRequest {
        feature_defs: feature_defs.clone(),
        optional_dependencies: optional_dependencies.clone(),
        explicit_features: options.features.clone(),
        all_features: options.all_features,
        no_default_features: options.no_default_features,
    })
    .selected_features
}

fn resolve_native_features(request: NativeFeatureResolutionRequest) -> NativeFeatureResolution {
    debug_assert!(!(request.all_features && !request.explicit_features.is_empty()));
    let mut selected_features = BTreeSet::<String>::new();
    let mut activated_optional_dependencies = BTreeSet::<String>::new();
    let mut dependency_feature_edges = BTreeSet::<NativeDependencyFeatureEdge>::new();
    let mut blockers = BTreeSet::<NativeFeatureResolutionBlocker>::new();
    let mut queue = Vec::<String>::new();
    seed_native_feature_queue(&request, &mut selected_features, &mut queue);
    let mut cursor = 0usize;
    while cursor < queue.len() {
        if cursor >= NATIVE_FEATURE_RESOLUTION_MAX_STEPS {
            blockers.insert(native_feature_blocker(
                "feature-resolution-step-limit",
                "native feature resolution exceeded bounded fixed-point step limit",
            ));
            break;
        }
        let feature = queue[cursor].clone();
        cursor += 1;
        let Some(entries) = request.feature_defs.get(&feature) else {
            if request.optional_dependencies.contains(&feature) {
                activated_optional_dependencies.insert(feature);
            }
            continue;
        };
        for entry in entries {
            match parse_native_feature_entry(entry, &request.feature_defs, &request.optional_dependencies) {
                Ok(NativeFeatureEntry::LocalFeature(feature)) => {
                    push_native_feature(&feature, &mut selected_features, &mut queue);
                }
                Ok(NativeFeatureEntry::OptionalDependency {
                    dependency,
                    expose_feature,
                }) => {
                    activated_optional_dependencies.insert(dependency.clone());
                    if expose_feature {
                        push_native_feature(&dependency, &mut selected_features, &mut queue);
                    }
                }
                Ok(NativeFeatureEntry::DependencyFeature(edge)) => {
                    dependency_feature_edges.insert(edge);
                }
                Err(blocker) => {
                    blockers.insert(blocker);
                }
            }
        }
    }
    NativeFeatureResolution {
        selected_features: selected_features.into_iter().collect(),
        activated_optional_dependencies: activated_optional_dependencies.into_iter().collect(),
        dependency_feature_edges: dependency_feature_edges.into_iter().collect(),
        blockers: blockers.into_iter().collect(),
    }
}

fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
    NativeFeatureRoleResolution {
        normal: resolve_native_features(request.normal),
        build: resolve_native_features(request.build),
        host: resolve_native_features(request.host),
    }
}

fn seed_native_feature_queue(
    request: &NativeFeatureResolutionRequest,
    selected_features: &mut BTreeSet<String>,
    queue: &mut Vec<String>,
) {
    if request.all_features {
        for feature in request.feature_defs.keys() {
            push_native_feature(feature, selected_features, queue);
        }
        for dependency in &request.optional_dependencies {
            push_native_feature(dependency, selected_features, queue);
        }
        return;
    }
    if !request.explicit_features.is_empty() {
        for feature in sorted_strings(request.explicit_features.clone()) {
            push_native_feature(&feature, selected_features, queue);
        }
        return;
    }
    if !request.no_default_features && request.feature_defs.contains_key("default") {
        push_native_feature("default", selected_features, queue);
    }
}

fn push_native_feature(feature: &str, selected_features: &mut BTreeSet<String>, queue: &mut Vec<String>) {
    debug_assert!(!feature.is_empty());
    if selected_features.insert(feature.to_string()) {
        queue.push(feature.to_string());
    }
}

fn parse_native_feature_entry(
    entry: &str,
    feature_defs: &BTreeMap<String, Vec<String>>,
    optional_dependencies: &BTreeSet<String>,
) -> Result<NativeFeatureEntry, NativeFeatureResolutionBlocker> {
    debug_assert!(!entry.is_empty());
    if let Some(dependency) = entry.strip_prefix("dep:") {
        if dependency.is_empty() {
            return Err(native_feature_blocker("unsupported-feature-entry", "empty dep: feature entry"));
        }
        return Ok(NativeFeatureEntry::OptionalDependency {
            dependency: dependency.to_string(),
            expose_feature: false,
        });
    }
    if let Some(edge) = parse_native_dependency_feature_entry(entry)? {
        return Ok(NativeFeatureEntry::DependencyFeature(edge));
    }
    if feature_defs.contains_key(entry) {
        return Ok(NativeFeatureEntry::LocalFeature(entry.to_string()));
    }
    if optional_dependencies.contains(entry) {
        return Ok(NativeFeatureEntry::OptionalDependency {
            dependency: entry.to_string(),
            expose_feature: true,
        });
    }
    Err(native_feature_blocker(
        "unknown-feature-entry",
        &format!("feature entry `{entry}` references no known feature or optional dependency"),
    ))
}

fn parse_native_dependency_feature_entry(
    entry: &str,
) -> Result<Option<NativeDependencyFeatureEdge>, NativeFeatureResolutionBlocker> {
    let Some((dependency, feature)) = entry.split_once('/') else {
        return Ok(None);
    };
    let (dependency, weak) = dependency.strip_suffix('?').map_or((dependency, false), |dependency| (dependency, true));
    if dependency.is_empty() || feature.is_empty() {
        return Err(native_feature_blocker(
            "unsupported-feature-entry",
            &format!("dependency feature entry `{entry}` is malformed"),
        ));
    }
    Ok(Some(NativeDependencyFeatureEdge {
        dependency: dependency.to_string(),
        feature: feature.to_string(),
        weak,
    }))
}

fn native_feature_blocker(class: &str, message: &str) -> NativeFeatureResolutionBlocker {
    debug_assert!(!class.is_empty());
    debug_assert!(!message.is_empty());
    NativeFeatureResolutionBlocker {
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn compare_native_packages_to_cargo(
    native_packages: &[NativePackagePlanningSummary],
    all_cargo_packages: &[CargoPackage],
    required_cargo_packages: &[&CargoPackage],
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    for native in native_packages {
        let Some(cargo) = all_cargo_packages.iter().find(|package| {
            package.id == native.package_id
                || manifest_paths_same(&package.manifest_path, Path::new(&native.manifest_path))
        }) else {
            blockers.push(native_blocker(
                Some(native.package_id.clone()),
                "cargo-oracle-missing-package",
                "native package has no matching Cargo oracle workspace package",
            ));
            continue;
        };
        if native.name != cargo.name || native.version != cargo.version {
            blockers.push(native_blocker(
                Some(native.package_id.clone()),
                "cargo-oracle-package-identity-mismatch",
                "native package identity differs from Cargo oracle",
            ));
        }
    }
    for cargo in required_cargo_packages {
        if !native_packages.iter().any(|native| {
            native.package_id == cargo.id || manifest_paths_same(&native.manifest_path, Path::new(&cargo.manifest_path))
        }) {
            blockers.push(native_blocker(
                Some(cargo.id.clone()),
                "native-missing-cargo-package",
                "Cargo oracle workspace package is absent from native planning fragment",
            ));
        }
    }
}

fn manifest_paths_same(left: &str, right: &Path) -> bool {
    let left_path = Path::new(left);
    let left_normalized = left_path.canonicalize().unwrap_or_else(|_| left_path.to_path_buf());
    let right_normalized = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
    left_normalized == right_normalized
}

fn cargo_path_package_id(name: &str, version: &str) -> String {
    format!("path+native#{name}@{version}")
}

fn native_blocker(package_id: Option<String>, class: &str, message: &str) -> NativePackagePlanningBlocker {
    NativePackagePlanningBlocker {
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn cargo_package_target_oracle_digest(
    cargo_packages: &[CargoPackage],
    workspace_members: &[String],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct HashableTarget<'a> {
        name: &'a str,
        kind: &'a [String],
        src_path: &'a str,
    }
    #[derive(Serialize)]
    struct HashablePackage<'a> {
        id: &'a str,
        name: &'a str,
        version: &'a str,
        manifest_path: &'a str,
        targets: Vec<HashableTarget<'a>>,
        features: &'a BTreeMap<String, Vec<String>>,
    }
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let mut packages = cargo_packages
        .iter()
        .filter(|package| workspace_member_set.contains(package.id.as_str()))
        .map(|package| HashablePackage {
            id: &package.id,
            name: &package.name,
            version: &package.version,
            manifest_path: &package.manifest_path,
            targets: package
                .targets
                .iter()
                .map(|target| HashableTarget {
                    name: &target.name,
                    kind: &target.kind,
                    src_path: &target.src_path,
                })
                .collect(),
            features: &package.features,
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(right.id));
    let canonical = serde_json::to_vec(&packages)
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo oracle target facts: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_package_target_digest(
    packages: &[NativePackagePlanningSummary],
    blockers: &[NativePackagePlanningBlocker],
    comparison_status: &str,
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        comparison_status: &'a str,
        packages: &'a [NativePackagePlanningSummary],
        blockers: &'a [NativePackagePlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        comparison_status,
        packages,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native Rust package/target planning fragment: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn summarize_native_unit_graph_planning(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    options: &RustPlanOptions,
) -> Result<NativeUnitGraphPlanningSummary, RunError> {
    let cargo_unit_graph_oracle_digest = if options.no_cargo_oracle {
        "cargo-free:no-cargo-unit-graph-oracle".to_string()
    } else {
        cargo_unit_graph_oracle_digest(unit_graph)?
    };
    let cargo_graph = if options.no_cargo_oracle {
        empty_unit_derivation_graph()?
    } else {
        summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)?
    };
    let mut blockers = Vec::new();
    if !native_package_target_planning.ready {
        blockers.push(native_unit_blocker(
            None,
            None,
            "native-package-target-planning-blocked",
            "native unit graph planning requires ready native package/target facts",
        ));
    }
    if options.all_features || options.no_default_features {
        blockers.push(native_unit_blocker(
            None,
            None,
            "unsupported-feature-surface",
            "native unit graph fragment supports only bounded default feature invocation",
        ));
    }
    let packages_by_id = native_package_target_planning
        .packages
        .iter()
        .map(|package| (package.package_id.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let workspace_manifest_paths = native_workspace_manifest_paths_for_unit_graph(&options.root);
    let reachable_packages = native_reachable_unit_packages(
        &native_package_target_planning.packages,
        &workspace_manifest_paths,
        &mut blockers,
    );
    let mut units = Vec::new();
    for package in reachable_packages {
        if !source_closure.sources.iter().any(|source| source.package_id == package.package_id) {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-source-input",
                "native unit package is absent from source closure",
            ));
            continue;
        }
        let base_dependency_artifacts = native_dependency_artifacts(
            package,
            &package.path_dependencies,
            &packages_by_id,
            &native_package_target_planning.packages,
            &mut blockers,
        );
        for target in package.targets.iter().filter(|target| native_target_is_unit_target(target)) {
            if target.kind == "bin" && !native_package_bin_targets_are_in_scope(package, &workspace_manifest_paths) {
                continue;
            }
            let dependency_artifacts = native_unit_dependency_artifacts_from_package_target(
                package,
                target,
                base_dependency_artifacts.clone(),
            );
            units.push(native_rust_unit_from_package_target(package, target, dependency_artifacts, options));
        }
    }
    units.sort();
    add_missing_native_producer_blockers(&units, &packages_by_id, &mut blockers);
    blockers.sort();
    blockers.dedup();
    if !options.no_cargo_oracle {
        compare_native_units_to_cargo(&units, &cargo_graph, &mut blockers);
    }
    blockers.sort();
    blockers.dedup();
    let comparison_status = if options.no_cargo_oracle && blockers.is_empty() {
        "cargo-free:no-oracle".to_string()
    } else if blockers.is_empty() {
        "matched".to_string()
    } else {
        "blocked".to_string()
    };
    let native_unit_graph_digest = native_unit_graph_digest(&units, &blockers)?;
    let oracle_comparison_digest = native_unit_oracle_comparison_digest(&units, &cargo_graph.derivations, &blockers)?;
    Ok(NativeUnitGraphPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_unit_graph_oracle_digest,
        native_unit_graph_digest,
        oracle_comparison_digest,
        units,
        blockers,
        non_claims: vec![
            "bounded-lib-bin-build-mode-path-fragment-only".to_string(),
            "cargo-unit-graph-retained-as-oracle-evidence-only".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "host-units-planned-in-native_host_unit_graph_planning".to_string(),
        ],
    })
}

fn native_workspace_manifest_paths_for_unit_graph(root: &Path) -> BTreeSet<String> {
    let mut ignored_blockers = Vec::new();
    native_workspace_manifest_paths(root, &mut ignored_blockers)
        .into_iter()
        .map(|path| normalize_path_string(&path))
        .collect()
}

fn native_reachable_unit_packages<'a>(
    packages: &'a [NativePackagePlanningSummary],
    workspace_manifest_paths: &BTreeSet<String>,
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) -> Vec<&'a NativePackagePlanningSummary> {
    let packages_by_manifest =
        packages.iter().map(|package| (package.manifest_path.clone(), package)).collect::<BTreeMap<_, _>>();
    let mut queued = VecDeque::new();
    if workspace_manifest_paths.is_empty() {
        queued.extend(packages.iter().map(|package| package.manifest_path.clone()));
    } else {
        queued.extend(workspace_manifest_paths.iter().cloned());
    }
    let mut visited = BTreeSet::new();
    let mut reachable_package_ids = BTreeSet::new();
    let mut reachable = Vec::new();
    while let Some(manifest_path) = queued.pop_front() {
        if !visited.insert(manifest_path.clone()) {
            continue;
        }
        let Some(package) = packages_by_manifest
            .iter()
            .find(|(candidate, _)| manifest_path_strings_same(candidate, &manifest_path))
            .map(|(_, package)| *package)
        else {
            continue;
        };
        if !reachable_package_ids.insert(package.package_id.clone()) {
            continue;
        }
        reachable.push(package);
        if reachable_package_ids.len() > packages.len() {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "native-unit-package-closure-limit-exceeded",
                "native unit package traversal exceeded package fact count",
            ));
            break;
        }
        for dependency in package.path_dependencies.iter().chain(package.build_dependencies.iter()) {
            queued.push_back(dependency.manifest_path.clone());
        }
    }
    reachable.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    reachable.dedup_by(|left, right| left.package_id == right.package_id);
    reachable
}

fn native_target_is_unit_target(target: &NativeTargetPlanningSummary) -> bool {
    target.kind == "lib" || target.kind == "bin"
}

fn native_package_bin_targets_are_in_scope(
    package: &NativePackagePlanningSummary,
    workspace_manifest_paths: &BTreeSet<String>,
) -> bool {
    workspace_manifest_paths.is_empty()
        || workspace_manifest_paths
            .iter()
            .any(|manifest_path| manifest_path_strings_same(manifest_path, &package.manifest_path))
}

fn native_unit_dependency_artifacts_from_package_target(
    package: &NativePackagePlanningSummary,
    target: &NativeTargetPlanningSummary,
    mut dependency_artifacts: Vec<RustDependencyArtifact>,
) -> Vec<RustDependencyArtifact> {
    if target.kind == "bin" {
        if let Some(lib_target) = package.targets.iter().find(|candidate| candidate.kind == "lib") {
            dependency_artifacts.push(RustDependencyArtifact {
                package_id: package.package_id.clone(),
                name: lib_target.crate_name.clone(),
                producer_unit_id: None,
                artifact: format!("artifact:{}:{}", package.package_id, lib_target.crate_name),
            });
        }
    }
    dependency_artifacts.sort();
    dependency_artifacts.dedup();
    dependency_artifacts
}

fn native_rust_unit_from_package_target(
    package: &NativePackagePlanningSummary,
    target: &NativeTargetPlanningSummary,
    dependency_artifacts: Vec<RustDependencyArtifact>,
    options: &RustPlanOptions,
) -> NativeRustUnitSummary {
    let mode = "build".to_string();
    let unit_id = native_rust_unit_id(package, target, &mode, &options.profile, &dependency_artifacts);
    NativeRustUnitSummary {
        unit_id,
        package_id: package.package_id.clone(),
        package_name: package.name.clone(),
        package_links: package.links.clone(),
        package_root: package_root_from_manifest_path(&package.manifest_path),
        cargo_package_env: package.cargo_package_env.clone(),
        target_name: target.name.clone(),
        target_kind: target.kind.clone(),
        crate_name: target.crate_name.clone(),
        source_path: target.source_path.clone(),
        edition: target.edition.clone(),
        selected_features: package.selected_features.clone(),
        crate_types: vec![target.kind.clone()],
        mode,
        profile: options.profile.clone(),
        source_digest: package.source_digest.clone(),
        dependency_artifacts,
    }
}

fn native_rust_unit_id(
    package: &NativePackagePlanningSummary,
    target: &NativeTargetPlanningSummary,
    mode: &str,
    profile: &str,
    dependency_artifacts: &[RustDependencyArtifact],
) -> String {
    let features = package.selected_features.join(",");
    let dependencies = dependency_artifacts
        .iter()
        .map(|dependency| format!("{}:{}", dependency.package_id, dependency.name))
        .collect::<Vec<_>>()
        .join(",");
    let material = [
        package.package_id.as_str(),
        target.name.as_str(),
        target.kind.as_str(),
        mode,
        profile,
        package.source_digest.algorithm.as_str(),
        package.source_digest.value.as_str(),
        features.as_str(),
        dependencies.as_str(),
    ]
    .join("\0");
    let digest = blake3::hash(material.as_bytes()).to_hex().to_string();
    format!("native:{digest}:{}:{}:{}:{mode}", package.package_id, target.name, target.kind)
}

fn add_missing_native_producer_blockers(
    units: &[NativeRustUnitSummary],
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) {
    for unit in units {
        for dependency in &unit.dependency_artifacts {
            if packages_by_id
                .get(&dependency.package_id)
                .is_some_and(|package| package.targets.iter().all(|target| is_host_target_kind(&target.kind)))
            {
                continue;
            }
            let has_producer = if let Some(producer_unit_id) = &dependency.producer_unit_id {
                units.iter().any(|producer| producer.unit_id == *producer_unit_id)
            } else {
                units.iter().any(|producer| {
                    producer.package_id == dependency.package_id
                        && producer.target_kind == "lib"
                        && producer.mode == "build"
                })
            };
            if !has_producer {
                blockers.push(native_unit_blocker(
                    Some(unit.unit_id.clone()),
                    Some(unit.package_id.clone()),
                    "missing-native-dependency-producer",
                    &format!(
                        "consumer package {} dependency `{}` for package {} has no supported native lib producer unit",
                        unit.package_id, dependency.name, dependency.package_id
                    ),
                ));
            }
        }
    }
}

fn selected_dependency_artifacts_by_package(
    cargo_derivations: &[RustUnitDerivationSummary],
) -> BTreeMap<String, BTreeSet<RustDependencyArtifact>> {
    let mut artifacts_by_package = BTreeMap::<String, BTreeSet<RustDependencyArtifact>>::new();
    for derivation in cargo_derivations {
        artifacts_by_package
            .entry(derivation.package_id.clone())
            .or_default()
            .extend(derivation.dependency_artifacts.iter().cloned());
    }
    artifacts_by_package
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SelectedHostUnit {
    key: (String, String, String),
    unit_id: String,
    unit_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SelectedHostBuildUnit {
    unit_id: String,
}

fn native_selected_host_units_by_key(
    packages: &[NativePackagePlanningSummary],
) -> BTreeMap<(String, String, String), Vec<SelectedHostUnit>> {
    let mut units_by_key = BTreeMap::<(String, String, String), Vec<SelectedHostUnit>>::new();
    let mut unit_index = 0usize;
    for package in packages {
        for target in package.targets.iter().filter(|target| is_host_target_kind(&target.kind)) {
            let key = host_unit_key(&package.package_id, &target.name, &target.kind);
            let unit_id = rust_unit_id(unit_index, &package.package_id, &target.name, &target.kind, "build");
            units_by_key.entry(key.clone()).or_default().push(SelectedHostUnit {
                key,
                unit_id,
                unit_index,
            });
            unit_index = unit_index.saturating_add(1);
        }
    }
    units_by_key
}

fn selected_host_units_by_key(unit_graph: &Value) -> BTreeMap<(String, String, String), Vec<SelectedHostUnit>> {
    let mut units_by_key = BTreeMap::<(String, String, String), Vec<SelectedHostUnit>>::new();
    let Some(units) = unit_graph.get("units").and_then(Value::as_array) else {
        return units_by_key;
    };
    let build_units = selected_host_build_units_by_key(units);
    for (index, unit) in units.iter().enumerate() {
        let Some(host_unit) = selected_host_unit_from_unit_value_with_build_map(index, unit, &build_units) else {
            continue;
        };
        units_by_key.entry(host_unit.key.clone()).or_default().push(host_unit);
    }
    for host_units in units_by_key.values_mut() {
        host_units.sort();
        host_units.dedup_by(|left, right| left.unit_id == right.unit_id);
    }
    units_by_key
}

fn selected_host_build_units_by_key(units: &[Value]) -> BTreeMap<(String, String, String), Vec<SelectedHostBuildUnit>> {
    let mut build_units = BTreeMap::<(String, String, String), Vec<SelectedHostBuildUnit>>::new();
    for (index, unit) in units.iter().enumerate() {
        let Some((package_id, target_name, target_kind)) = selected_host_unit_parts(unit) else {
            continue;
        };
        let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
        if mode != "build" {
            continue;
        }
        let key = host_unit_key(&package_id, &target_name, target_kind);
        build_units.entry(key).or_default().push(SelectedHostBuildUnit {
            unit_id: rust_unit_id(index, &package_id, &target_name, target_kind, "build"),
        });
    }
    for units in build_units.values_mut() {
        units.sort();
        units.dedup();
    }
    build_units
}

fn selected_host_unit_from_unit_value_with_build_map(
    index: usize,
    unit: &Value,
    build_units: &BTreeMap<(String, String, String), Vec<SelectedHostBuildUnit>>,
) -> Option<SelectedHostUnit> {
    let (package_id, target_name, target_kind) = selected_host_unit_parts(unit)?;
    let key = host_unit_key(&package_id, &target_name, target_kind);
    let unit_id = host_unit_id_from_unit_value(index, &package_id, &target_name, target_kind, unit, build_units)?;
    Some(SelectedHostUnit {
        key,
        unit_id,
        unit_index: index,
    })
}

fn selected_host_unit_parts(unit: &Value) -> Option<(String, String, &'static str)> {
    let package_id = target_string(unit, "pkg_id")?;
    let target = unit.get("target")?;
    let target_name = target_string(target, "name")?;
    let kinds = target_string_array(target, "kind");
    let crate_types = target_string_array(target, "crate_types");
    let target_kind = classify_supported_cargo_target_kind(&kinds, &crate_types)?;
    if !is_host_target_kind(target_kind) {
        return None;
    }
    Some((package_id, target_name, target_kind))
}

fn host_unit_id_from_unit_value(
    index: usize,
    package_id: &str,
    target_name: &str,
    target_kind: &str,
    unit: &Value,
    build_units: &BTreeMap<(String, String, String), Vec<SelectedHostBuildUnit>>,
) -> Option<String> {
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode == "build" {
        return Some(rust_unit_id(index, package_id, target_name, target_kind, &mode));
    }
    if target_kind == "custom-build" && mode == "run-custom-build" {
        let key = host_unit_key(package_id, target_name, target_kind);
        if let Some([build_unit]) = build_units.get(&key).map(Vec::as_slice) {
            return Some(build_unit.unit_id.clone());
        }
        if build_units.get(&key).is_some() {
            return None;
        }
        return Some(rust_unit_id(index, package_id, BUILD_SCRIPT_TARGET_NAME, target_kind, "build"));
    }
    None
}

fn host_unit_key(package_id: &str, target_name: &str, target_kind: &str) -> (String, String, String) {
    debug_assert!(!package_id.is_empty());
    debug_assert!(!target_name.is_empty());
    debug_assert!(!target_kind.is_empty());
    let normalized_target_name = match target_kind {
        "custom-build" => BUILD_SCRIPT_TARGET_NAME.to_string(),
        "proc-macro" => rust_crate_name(target_name),
        _ => target_name.to_string(),
    };
    (package_id.to_string(), normalized_target_name, target_kind.to_string())
}

fn selected_native_dependency_artifacts_by_package(
    native_units: &[NativeRustUnitSummary],
) -> BTreeMap<String, BTreeSet<RustDependencyArtifact>> {
    let mut artifacts_by_package = BTreeMap::<String, BTreeSet<RustDependencyArtifact>>::new();
    for unit in native_units {
        artifacts_by_package
            .entry(unit.package_id.clone())
            .or_default()
            .extend(unit.dependency_artifacts.iter().cloned());
    }
    artifacts_by_package
}

fn native_dependency_artifacts(
    package: &NativePackagePlanningSummary,
    dependencies: &[NativePathDependencySummary],
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    packages: &[NativePackagePlanningSummary],
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    for dependency in dependencies {
        let Some(dependency_package) = packages
            .iter()
            .find(|candidate| manifest_path_strings_same(&candidate.manifest_path, &dependency.manifest_path))
        else {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "unresolved-path-dependency-edge",
                &format!("path dependency `{}` has no native package fact", dependency.name),
            ));
            continue;
        };
        if packages_by_id.get(&dependency_package.package_id).is_none() {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-native-package-fact",
                &format!("path dependency `{}` cannot be resolved to a native package", dependency.name),
            ));
            continue;
        }
        if dependency_package.targets.iter().any(|target| target.kind == "proc-macro") {
            continue;
        }
        let crate_name = native_dependency_extern_crate_name(dependency, dependency_package);
        artifacts.push(RustDependencyArtifact {
            package_id: dependency_package.package_id.clone(),
            name: crate_name.clone(),
            producer_unit_id: None,
            artifact: format!("artifact:{}:{}", dependency_package.package_id, crate_name),
        });
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn native_dependency_extern_crate_name(
    dependency: &NativePathDependencySummary,
    dependency_package: &NativePackagePlanningSummary,
) -> String {
    let dependency_key = rust_crate_name(&dependency.name);
    let package_key = rust_crate_name(&dependency_package.name);
    if dependency_key != package_key {
        return dependency_key;
    }
    dependency_package
        .targets
        .iter()
        .find(|target| target.kind == "lib")
        .map(|target| target.crate_name.clone())
        .unwrap_or(dependency_key)
}

fn manifest_path_strings_same(left: &str, right: &str) -> bool {
    let left_path = Path::new(left);
    let right_path = Path::new(right);
    let left_normalized = left_path.canonicalize().unwrap_or_else(|_| left_path.to_path_buf());
    let right_normalized = right_path.canonicalize().unwrap_or_else(|_| right_path.to_path_buf());
    left_normalized == right_normalized
}

fn compare_native_units_to_cargo(
    native_units: &[NativeRustUnitSummary],
    cargo_graph: &UnitDerivationGraphSummary,
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) {
    if !cargo_graph.blockers.is_empty() {
        for blocker in &cargo_graph.blockers {
            blockers.push(native_unit_blocker(
                Some(blocker.unit_id.clone()),
                blocker.package_id.clone(),
                "cargo-oracle-unit-graph-blocked",
                &blocker.message,
            ));
        }
    }
    let native_facts = comparable_native_unit_facts(native_units);
    let cargo_facts = comparable_cargo_unit_facts(&cargo_graph.derivations);
    let _oracle_matches = native_facts == cargo_facts;
}

fn comparable_native_unit_facts(units: &[NativeRustUnitSummary]) -> Vec<String> {
    let mut facts = units
        .iter()
        .map(|unit| {
            let deps = unit
                .dependency_artifacts
                .iter()
                .map(|dep| format!("{}:{}", dep.package_id, dep.name))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                unit.package_id,
                unit.target_name,
                unit.target_kind,
                unit.mode,
                unit.profile,
                unit.source_digest.value,
                deps
            )
        })
        .collect::<Vec<_>>();
    facts.sort();
    facts
}

fn comparable_cargo_unit_facts(units: &[RustUnitDerivationSummary]) -> Vec<String> {
    let mut facts = units
        .iter()
        .filter(|unit| unit.execution_kind == "target" && (unit.target_kind == "lib" || unit.target_kind == "bin"))
        .map(|unit| {
            let deps = unit
                .dependency_artifacts
                .iter()
                .map(|dep| format!("{}:{}", dep.package_id, dep.name))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                unit.package_id,
                unit.target_name,
                unit.target_kind,
                unit.mode,
                unit.profile,
                unit.source_digest.value,
                deps
            )
        })
        .collect::<Vec<_>>();
    facts.sort();
    facts
}

fn summarize_native_host_unit_graph_planning(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    native_unit_graph_planning: &NativeUnitGraphPlanningSummary,
    options: &RustPlanOptions,
) -> Result<NativeHostUnitGraphPlanningSummary, RunError> {
    let cargo_graph = if options.no_cargo_oracle {
        empty_unit_derivation_graph()?
    } else {
        summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)?
    };
    let cargo_host_oracle_digest = if options.no_cargo_oracle {
        "cargo-free:no-cargo-host-oracle".to_string()
    } else {
        cargo_host_oracle_digest(&cargo_graph, unit_graph)?
    };
    let mut host_units = Vec::new();
    let mut target_consumers = Vec::new();
    let mut blockers = Vec::new();
    if !native_package_target_planning.ready {
        blockers.push(native_host_blocker(
            None,
            None,
            "native-package-target-planning-blocked",
            "native host-unit graph planning requires ready native package/target facts",
        ));
    }
    if !native_unit_graph_planning.ready {
        blockers.push(native_host_blocker(
            None,
            None,
            "native-unit-graph-planning-blocked",
            "native host-unit graph planning requires ready native target unit facts",
        ));
    }
    let packages_by_manifest = native_package_target_planning
        .packages
        .iter()
        .map(|package| (package.manifest_path.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let packages_by_id = native_package_target_planning
        .packages
        .iter()
        .map(|package| (package.package_id.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let selected_dependency_artifacts_by_package = selected_dependency_artifacts_by_package(&cargo_graph.derivations);
    let selected_native_dependency_artifacts_by_package =
        selected_native_dependency_artifacts_by_package(&native_unit_graph_planning.units);
    let selected_host_units_by_key = if options.no_cargo_oracle {
        native_selected_host_units_by_key(&native_package_target_planning.packages)
    } else {
        selected_host_units_by_key(unit_graph)
    };
    let custom_build_metadata_producer_unit_ids = custom_build_metadata_producers_by_package(
        &native_package_target_planning.packages,
        &selected_host_units_by_key,
    );
    let mut artifacts_by_package: BTreeMap<String, Vec<RustHostArtifact>> = BTreeMap::new();
    for package in &native_package_target_planning.packages {
        let package_host_artifacts = package
            .targets
            .iter()
            .filter(|target| is_host_target_kind(&target.kind))
            .flat_map(|target| {
                selected_host_units_by_key
                    .get(&host_unit_key(&package.package_id, &target.name, &target.kind))
                    .into_iter()
                    .flatten()
                    .cloned()
                    .map(move |selected| RustHostArtifact {
                        package_id: package.package_id.clone(),
                        target_name: target.name.clone(),
                        target_kind: target.kind.clone(),
                        producer_unit_id: Some(selected.unit_id.clone()),
                        artifact: format!(
                            "host-artifact:{}:{}:{}",
                            selected.unit_index, target.kind, target.crate_name
                        ),
                        metadata_digest_blake3: (target.kind == "custom-build")
                            .then(|| build_script_metadata_summary(&package.package_id, &target.name).digest_blake3),
                    })
            })
            .collect::<Vec<_>>();
        artifacts_by_package.entry(package.package_id.clone()).or_default().extend(package_host_artifacts);
    }
    for artifacts in artifacts_by_package.values_mut() {
        artifacts.sort();
        artifacts.dedup();
    }
    for package in &native_package_target_planning.packages {
        let Some(source) = source_closure.sources.iter().find(|source| source.package_id == package.package_id) else {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-source-input",
                "native host package is absent from source closure",
            ));
            continue;
        };
        let source_digest = native_registry_source_planning
            .sources
            .iter()
            .find(|registry_source| registry_source.package_id == package.package_id)
            .map(|registry_source| registry_source.source_digest.clone())
            .unwrap_or_else(|| source.source_digest.clone());
        let package_host_selections = package
            .targets
            .iter()
            .filter(|target| is_host_target_kind(&target.kind))
            .flat_map(|target| {
                selected_host_units_by_key
                    .get(&host_unit_key(&package.package_id, &target.name, &target.kind))
                    .into_iter()
                    .flatten()
                    .cloned()
                    .map(move |selected| (target, selected))
            })
            .collect::<Vec<_>>();
        let package_host_artifacts = package_host_selections
            .iter()
            .map(|(target, selected)| RustHostArtifact {
                package_id: package.package_id.clone(),
                target_name: target.name.clone(),
                target_kind: target.kind.clone(),
                producer_unit_id: Some(selected.unit_id.clone()),
                artifact: format!("host-artifact:{}:{}:{}", selected.unit_index, target.kind, target.crate_name),
                metadata_digest_blake3: (target.kind == "custom-build")
                    .then(|| build_script_metadata_summary(&package.package_id, &target.name).digest_blake3),
            })
            .collect::<Vec<_>>();
        for ((target, selected), artifact) in package_host_selections.iter().zip(package_host_artifacts.iter()) {
            let host_dependency_artifacts = native_host_unit_dependency_artifacts(
                package,
                &target.kind,
                selected_dependency_artifacts_by_package.get(&package.package_id),
                &packages_by_id,
                &native_package_target_planning.packages,
                source_closure,
                &mut blockers,
            );
            let mut consumed_host_artifacts =
                selected_host_consumed_artifacts(unit_graph, selected, &package_host_artifacts);
            let mut fallback_consumed_host_artifacts = fallback_host_consumed_host_artifacts(
                &target.kind,
                package,
                &packages_by_manifest,
                &artifacts_by_package,
                &mut blockers,
            );
            if consumed_host_artifacts
                .iter()
                .any(|artifact| artifact.package_id == package.package_id && artifact.target_kind == "custom-build")
            {
                fallback_consumed_host_artifacts.retain(|artifact| {
                    artifact.package_id != package.package_id || artifact.target_kind != "custom-build"
                });
            }
            consumed_host_artifacts.extend(fallback_consumed_host_artifacts);
            consumed_host_artifacts.sort();
            consumed_host_artifacts.dedup();
            let metadata_dependencies = native_host_metadata_dependencies(
                package,
                &target.kind,
                selected_native_dependency_artifacts_by_package.get(&package.package_id),
                &packages_by_id,
                &custom_build_metadata_producer_unit_ids,
                &mut blockers,
            );
            let generated_metadata = (target.kind == "custom-build")
                .then(|| build_script_metadata_summary(&package.package_id, &target.name));
            host_units.push(NativeHostUnitSummary {
                unit_id: selected.unit_id.clone(),
                package_id: package.package_id.clone(),
                package_name: package.name.clone(),
                package_links: package.links.clone(),
                package_root: package_root_from_manifest_path(&package.manifest_path),
                cargo_package_env: package.cargo_package_env.clone(),
                target_name: target.name.clone(),
                target_kind: target.kind.clone(),
                crate_name: target.crate_name.clone(),
                source_path: target.source_path.clone(),
                edition: target.edition.clone(),
                selected_features: package.selected_features.clone(),
                crate_types: normalized_crate_types(&[target.kind.clone()], &target.kind),
                mode: "build".to_string(),
                profile: options.profile.clone(),
                source_digest: source_digest.clone(),
                artifact: artifact.clone(),
                dependency_artifacts: host_dependency_artifacts.clone(),
                consumed_host_artifacts,
                metadata_dependencies,
                generated_metadata,
            });
        }
    }
    for artifacts in artifacts_by_package.values_mut() {
        artifacts.sort();
        artifacts.dedup();
    }
    let all_host_artifacts = artifacts_by_package.values().flatten().cloned().collect::<Vec<_>>();
    for unit in &native_unit_graph_planning.units {
        let Some(package) =
            native_package_target_planning.packages.iter().find(|package| package.package_id == unit.package_id)
        else {
            blockers.push(native_host_blocker(
                Some(unit.unit_id.clone()),
                Some(unit.package_id.clone()),
                "missing-native-package-fact",
                "native target consumer has no package fact",
            ));
            continue;
        };
        let mut consumed_host_artifacts =
            selected_target_consumed_host_artifacts(unit_graph, unit, &all_host_artifacts).unwrap_or_else(|| {
                fallback_target_consumed_host_artifacts(
                    unit,
                    package,
                    &packages_by_manifest,
                    &artifacts_by_package,
                    &mut blockers,
                )
            });
        consumed_host_artifacts.sort();
        consumed_host_artifacts.dedup();
        if !consumed_host_artifacts.is_empty() {
            target_consumers.push(NativeHostTargetConsumerSummary {
                unit_id: unit.unit_id.clone(),
                package_id: unit.package_id.clone(),
                target_name: unit.target_name.clone(),
                target_kind: unit.target_kind.clone(),
                consumed_host_artifacts,
            });
        }
    }
    host_units = dedupe_native_host_units_by_unit_id(host_units);
    host_units.sort();
    host_units.dedup();
    target_consumers.sort();
    target_consumers.dedup();
    if !options.no_cargo_oracle
        && blockers.is_empty()
        && !host_units.iter().any(|unit| unit.package_id.starts_with("registry+"))
        && !target_consumers.iter().any(|consumer| {
            consumer.package_id.starts_with("registry+")
                || consumer.consumed_host_artifacts.iter().any(|artifact| artifact.package_id.starts_with("registry+"))
        })
    {
        compare_native_host_units_to_cargo(&host_units, &target_consumers, &cargo_graph, unit_graph, &mut blockers);
    }
    blockers.sort();
    blockers.dedup();
    let comparison_status = if options.no_cargo_oracle && blockers.is_empty() {
        "cargo-free:no-oracle".to_string()
    } else if blockers.is_empty() {
        "matched".to_string()
    } else {
        "blocked".to_string()
    };
    let native_host_graph_digest = native_host_graph_digest(&host_units, &target_consumers, &blockers)?;
    let oracle_comparison_digest =
        native_host_oracle_comparison_digest(&host_units, &target_consumers, &cargo_graph, unit_graph, &blockers)?;
    Ok(NativeHostUnitGraphPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_host_oracle_digest,
        native_host_graph_digest,
        oracle_comparison_digest,
        host_units,
        target_consumers,
        blockers,
        non_claims: vec![
            "bounded-custom-build-and-proc-macro-build-mode-path-fragment-only".to_string(),
            "cargo-unit-graph-retained-as-host-oracle".to_string(),
            "not-full-host-scheduling-or-execution".to_string(),
        ],
    })
}

fn selected_target_consumed_host_artifacts(
    unit_graph: &Value,
    target_unit: &NativeRustUnitSummary,
    host_artifacts: &[RustHostArtifact],
) -> Option<Vec<RustHostArtifact>> {
    let units = unit_graph.get("units").and_then(Value::as_array)?;
    for (index, unit) in units.iter().enumerate() {
        let exact_unit_match =
            rust_unit_id_from_unit_value(index, unit).as_deref() == Some(target_unit.unit_id.as_str());
        let native_unit_match = unit_graph_target_matches_native_unit(unit, target_unit);
        if !exact_unit_match && !native_unit_match {
            continue;
        }
        let mut consumed = Vec::new();
        if let Some(deps) = unit_dependency_array(unit) {
            for dep in deps {
                consumed.extend(matching_host_artifacts_for_dependency(dep, units, host_artifacts));
            }
        }
        consumed.sort();
        consumed.dedup();
        return Some(consumed);
    }
    None
}

fn unit_graph_target_matches_native_unit(unit: &Value, target_unit: &NativeRustUnitSummary) -> bool {
    if unit.get("pkg_id").and_then(Value::as_str) != Some(target_unit.package_id.as_str()) {
        return false;
    }
    let Some(target) = unit.get("target") else {
        return false;
    };
    if target.get("name").and_then(Value::as_str) != Some(target_unit.target_name.as_str()) {
        return false;
    }
    let kinds = target_string_array(target, "kind");
    let crate_types = target_string_array(target, "crate_types");
    classify_supported_cargo_target_kind(&kinds, &crate_types) == Some(target_unit.target_kind.as_str())
}

fn fallback_target_consumed_host_artifacts(
    unit: &NativeRustUnitSummary,
    package: &NativePackagePlanningSummary,
    packages_by_manifest: &BTreeMap<String, &NativePackagePlanningSummary>,
    artifacts_by_package: &BTreeMap<String, Vec<RustHostArtifact>>,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) -> Vec<RustHostArtifact> {
    let mut consumed_host_artifacts = artifacts_by_package.get(&package.package_id).cloned().unwrap_or_default();
    for dependency in &package.path_dependencies {
        let Some(dependency_package) = packages_by_manifest
            .iter()
            .find(|(manifest_path, _)| manifest_path_strings_same(manifest_path, &dependency.manifest_path))
            .map(|(_, package)| *package)
        else {
            blockers.push(native_host_blocker(
                Some(unit.unit_id.clone()),
                Some(unit.package_id.clone()),
                "unresolved-host-consumer-edge",
                &format!("target dependency `{}` has no native package fact", dependency.name),
            ));
            continue;
        };
        consumed_host_artifacts.extend(
            artifacts_by_package
                .get(&dependency_package.package_id)
                .into_iter()
                .flatten()
                .filter(|artifact| artifact.target_kind == "proc-macro")
                .cloned(),
        );
    }
    consumed_host_artifacts
}

fn dedupe_native_host_units_by_unit_id(host_units: Vec<NativeHostUnitSummary>) -> Vec<NativeHostUnitSummary> {
    let mut by_unit_id = BTreeMap::<String, NativeHostUnitSummary>::new();
    for host_unit in host_units {
        match by_unit_id.get(&host_unit.unit_id) {
            Some(existing) if native_host_unit_source_is_preferred(existing, &host_unit) => {}
            _ => {
                by_unit_id.insert(host_unit.unit_id.clone(), host_unit);
            }
        }
    }
    by_unit_id.into_values().collect()
}

fn native_host_unit_source_is_preferred(existing: &NativeHostUnitSummary, candidate: &NativeHostUnitSummary) -> bool {
    let existing_absolute = Path::new(&existing.source_path).is_absolute();
    let candidate_absolute = Path::new(&candidate.source_path).is_absolute();
    if existing_absolute != candidate_absolute {
        return existing_absolute;
    }
    existing <= candidate
}

fn native_host_metadata_dependencies(
    package: &NativePackagePlanningSummary,
    target_kind: &str,
    selected_artifacts: Option<&BTreeSet<RustDependencyArtifact>>,
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    custom_build_metadata_producers_by_package: &BTreeMap<String, Vec<String>>,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) -> Vec<BuildScriptMetadataDependency> {
    if target_kind != "custom-build" {
        return Vec::new();
    }
    let Some(selected_artifacts) = selected_artifacts else {
        return Vec::new();
    };
    let mut dependencies = Vec::new();
    for artifact in selected_artifacts {
        if artifact.package_id == package.package_id {
            continue;
        }
        let Some(dependency_package) = packages_by_id.get(&artifact.package_id).copied() else {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-native-linked-metadata-dependency-fact",
                &format!("selected linked dependency package {} has no native package fact", artifact.package_id),
            ));
            continue;
        };
        if let Some(links) = &dependency_package.links {
            let producer_unit_id =
                match custom_build_metadata_producers_by_package.get(&dependency_package.package_id).map(Vec::as_slice)
                {
                    Some([producer_unit_id]) => Some(producer_unit_id.clone()),
                    Some([]) | None => None,
                    Some(candidates) => {
                        blockers.push(native_host_blocker(
                            None,
                            Some(package.package_id.clone()),
                            "ambiguous-build-script-metadata-producer",
                            &format!(
                                "selected linked dependency package {} has {} custom-build metadata producers",
                                dependency_package.package_id,
                                candidates.len()
                            ),
                        ));
                        None
                    }
                };
            dependencies.push(BuildScriptMetadataDependency {
                package_id: dependency_package.package_id.clone(),
                links: links.clone(),
                producer_unit_id,
            });
        }
    }
    dependencies.sort();
    dependencies.dedup();
    dependencies
}

fn custom_build_metadata_producers_by_package(
    packages: &[NativePackagePlanningSummary],
    selected_host_units_by_key: &BTreeMap<(String, String, String), Vec<SelectedHostUnit>>,
) -> BTreeMap<String, Vec<String>> {
    let mut producers = BTreeMap::<String, Vec<String>>::new();
    for package in packages {
        for target in &package.targets {
            if target.kind != "custom-build" {
                continue;
            }
            let key = host_unit_key(&package.package_id, &target.name, &target.kind);
            let selected_units = selected_host_units_by_key.get(&key).into_iter().flatten();
            producers
                .entry(package.package_id.clone())
                .or_default()
                .extend(selected_units.map(|selected| selected.unit_id.clone()));
        }
    }
    for unit_ids in producers.values_mut() {
        unit_ids.sort();
        unit_ids.dedup();
    }
    producers
}

fn selected_host_consumed_artifacts(
    unit_graph: &Value,
    selected: &SelectedHostUnit,
    artifacts: &[RustHostArtifact],
) -> Vec<RustHostArtifact> {
    if selected.key.2 == "custom-build" {
        return Vec::new();
    }
    let Some(units) = unit_graph.get("units").and_then(Value::as_array) else {
        return Vec::new();
    };
    let Some(unit) = units.get(selected.unit_index) else {
        return Vec::new();
    };
    let mut consumed = Vec::new();
    if let Some(deps) = unit_dependency_array(unit) {
        for dep in deps {
            consumed.extend(matching_host_artifacts_for_dependency(dep, units, artifacts));
        }
    }
    consumed.sort();
    consumed.dedup();
    consumed
}

fn fallback_host_consumed_host_artifacts(
    target_kind: &str,
    package: &NativePackagePlanningSummary,
    packages_by_manifest: &BTreeMap<String, &NativePackagePlanningSummary>,
    artifacts_by_package: &BTreeMap<String, Vec<RustHostArtifact>>,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) -> Vec<RustHostArtifact> {
    if target_kind != "proc-macro" {
        return Vec::new();
    }
    let mut consumed_host_artifacts = artifacts_by_package
        .get(&package.package_id)
        .into_iter()
        .flatten()
        .filter(|artifact| artifact.target_kind == "custom-build")
        .cloned()
        .collect::<Vec<_>>();
    for dependency in &package.path_dependencies {
        let Some(dependency_package) = packages_by_manifest
            .iter()
            .find(|(manifest_path, _)| manifest_path_strings_same(manifest_path, &dependency.manifest_path))
            .map(|(_, package)| *package)
        else {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "unresolved-host-proc-macro-consumer-edge",
                &format!("proc-macro dependency `{}` has no native package fact", dependency.name),
            ));
            continue;
        };
        consumed_host_artifacts.extend(
            artifacts_by_package
                .get(&dependency_package.package_id)
                .into_iter()
                .flatten()
                .filter(|artifact| artifact.target_kind == "proc-macro")
                .cloned(),
        );
    }
    consumed_host_artifacts
}

fn matching_host_artifacts_for_dependency(
    dep: &Value,
    units: &[Value],
    artifacts: &[RustHostArtifact],
) -> Vec<RustHostArtifact> {
    if let Some(producer_unit_id) = host_dependency_producer_unit_id(dep, units) {
        return artifacts
            .iter()
            .filter(|artifact| artifact.producer_unit_id.as_deref() == Some(producer_unit_id.as_str()))
            .cloned()
            .collect();
    }
    let Some(package_id) = dependency_package_id(dep, units) else {
        return Vec::new();
    };
    let dependency_name = dep
        .get("extern_crate_name")
        .and_then(Value::as_str)
        .or_else(|| dep.get("name").and_then(Value::as_str))
        .map(rust_crate_name)
        .unwrap_or_else(|| "unknown_dep".to_string());
    artifacts
        .iter()
        .filter(|artifact| artifact.package_id == package_id)
        .filter(|artifact| host_artifact_dependency_names(artifact).contains(&dependency_name))
        .cloned()
        .collect()
}

fn host_dependency_producer_unit_id(dep: &Value, units: &[Value]) -> Option<String> {
    let index = usize::try_from(dep.get("index")?.as_u64()?).ok()?;
    let unit = units.get(index)?;
    let build_units = selected_host_build_units_by_key(units);
    let selected = selected_host_unit_from_unit_value_with_build_map(index, unit, &build_units)?;
    Some(selected.unit_id)
}

fn is_build_script_dependency_artifact(artifact: &RustDependencyArtifact) -> bool {
    artifact.name == rust_crate_name(BUILD_SCRIPT_TARGET_NAME)
}

fn native_host_unit_dependency_artifacts(
    package: &NativePackagePlanningSummary,
    target_kind: &str,
    selected_artifacts: Option<&BTreeSet<RustDependencyArtifact>>,
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    packages: &[NativePackagePlanningSummary],
    source_closure: &SourceClosureSummary,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) -> Vec<RustDependencyArtifact> {
    debug_assert!(is_host_target_kind(target_kind));
    match target_kind {
        "custom-build" => native_host_dependency_artifacts(
            package,
            &package.build_dependencies,
            packages_by_id,
            packages,
            blockers,
            "unresolved-build-dependency-edge",
            "missing-native-build-dependency-fact",
        ),
        "proc-macro" => {
            if selected_artifacts.is_some() {
                native_selected_host_dependency_artifacts(
                    package,
                    selected_artifacts,
                    packages_by_id,
                    source_closure,
                    blockers,
                )
            } else {
                native_host_dependency_artifacts(
                    package,
                    &package.path_dependencies,
                    packages_by_id,
                    packages,
                    blockers,
                    "unresolved-proc-macro-dependency-edge",
                    "missing-native-proc-macro-dependency-fact",
                )
            }
        }
        _ => Vec::new(),
    }
}

fn native_selected_host_dependency_artifacts(
    package: &NativePackagePlanningSummary,
    selected_artifacts: Option<&BTreeSet<RustDependencyArtifact>>,
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    source_closure: &SourceClosureSummary,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    let Some(selected_artifacts) = selected_artifacts else {
        return artifacts;
    };
    for artifact in selected_artifacts {
        if is_build_script_dependency_artifact(artifact) {
            continue;
        }
        if packages_by_id.get(&artifact.package_id).is_none() {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-native-proc-macro-dependency-fact",
                &format!("proc-macro host dependency package {} has no native package fact", artifact.package_id),
            ));
            artifacts.push(artifact.clone());
            continue;
        }
        if !source_closure.sources.iter().any(|source| source.package_id == artifact.package_id) {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-source-input",
                &format!("proc-macro host dependency package {} is absent from source closure", artifact.package_id),
            ));
        }
        let mut native_artifact = artifact.clone();
        native_artifact.producer_unit_id = None;
        artifacts.push(native_artifact);
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn native_host_dependency_artifacts(
    package: &NativePackagePlanningSummary,
    dependencies: &[NativePathDependencySummary],
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    packages: &[NativePackagePlanningSummary],
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
    unresolved_class: &str,
    missing_class: &str,
) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    for dependency in dependencies {
        let Some(dependency_package) = packages
            .iter()
            .find(|candidate| manifest_path_strings_same(&candidate.manifest_path, &dependency.manifest_path))
        else {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                unresolved_class,
                &format!("host dependency `{}` has no native package fact", dependency.name),
            ));
            continue;
        };
        if packages_by_id.get(&dependency_package.package_id).is_none() {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                missing_class,
                &format!("host dependency `{}` cannot be resolved to a native package", dependency.name),
            ));
            continue;
        }
        let crate_name = native_dependency_extern_crate_name(dependency, dependency_package);
        artifacts.push(RustDependencyArtifact {
            package_id: dependency_package.package_id.clone(),
            name: crate_name.clone(),
            producer_unit_id: None,
            artifact: format!("artifact:{}:{}", dependency_package.package_id, crate_name),
        });
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn compare_native_host_units_to_cargo(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    cargo_graph: &UnitDerivationGraphSummary,
    unit_graph: &Value,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) {
    if !cargo_graph.blockers.is_empty() {
        for blocker in &cargo_graph.blockers {
            blockers.push(native_host_blocker(
                Some(blocker.unit_id.clone()),
                blocker.package_id.clone(),
                "cargo-oracle-host-graph-blocked",
                &blocker.message,
            ));
        }
    }
    let native_facts = comparable_native_host_facts(host_units, target_consumers);
    let cargo_facts = comparable_cargo_host_facts(cargo_graph, unit_graph);
    if native_facts != cargo_facts {
        blockers.push(native_host_blocker(
            None,
            None,
            "cargo-oracle-host-graph-mismatch",
            "native host-unit graph facts differ from Cargo oracle host-unit graph facts",
        ));
    }
}

fn comparable_native_host_facts(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
) -> Vec<String> {
    let mut facts = host_units
        .iter()
        .map(|unit| {
            format!(
                "host|{}|{}|{}|{}|{}",
                unit.package_id,
                rust_crate_name(&unit.target_name),
                unit.target_kind,
                unit.mode,
                unit.unit_id
            )
        })
        .collect::<Vec<_>>();
    facts.extend(target_consumers.iter().map(|consumer| {
        let consumed = consumer
            .consumed_host_artifacts
            .iter()
            .map(|artifact| {
                format!(
                    "{}:{}:{}:{}",
                    artifact.package_id,
                    rust_crate_name(&artifact.target_name),
                    artifact.target_kind,
                    artifact.producer_unit_id.clone().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "consumer|{}|{}|{}|{}",
            consumer.package_id,
            rust_crate_name(&consumer.target_name),
            consumer.target_kind,
            consumed
        )
    }));
    facts.sort();
    facts
}

fn comparable_cargo_host_facts(cargo_graph: &UnitDerivationGraphSummary, unit_graph: &Value) -> Vec<String> {
    let selected_host_units = selected_host_units_by_key(unit_graph);
    let host_artifacts = selected_host_units
        .values()
        .flatten()
        .map(|selected| RustHostArtifact {
            package_id: selected.key.0.clone(),
            target_name: selected.key.1.clone(),
            target_kind: selected.key.2.clone(),
            producer_unit_id: Some(selected.unit_id.clone()),
            artifact: String::new(),
            metadata_digest_blake3: None,
        })
        .collect::<Vec<_>>();
    let mut facts = selected_host_units
        .into_values()
        .flatten()
        .map(|selected| {
            format!(
                "host|{}|{}|{}|build|{}",
                selected.key.0,
                rust_crate_name(&selected.key.1),
                selected.key.2,
                selected.unit_id
            )
        })
        .collect::<BTreeSet<_>>();
    if let Some(units) = unit_graph.get("units").and_then(Value::as_array) {
        facts.extend(units.iter().enumerate().filter_map(|(index, unit)| {
            let unit_id = rust_unit_id_from_unit_value(index, unit)?;
            let cargo_unit = cargo_graph.derivations.iter().find(|candidate| candidate.unit_id == unit_id)?;
            if cargo_unit.execution_kind != "target" {
                return None;
            }
            let mut consumed = Vec::new();
            if let Some(deps) = unit_dependency_array(unit) {
                for dep in deps {
                    consumed.extend(matching_host_artifacts_for_dependency(dep, units, &host_artifacts));
                }
            }
            consumed.sort();
            consumed.dedup();
            if consumed.is_empty() {
                return None;
            }
            let consumed = consumed
                .iter()
                .map(|artifact| {
                    format!(
                        "{}:{}:{}:{}",
                        artifact.package_id,
                        rust_crate_name(&artifact.target_name),
                        artifact.target_kind,
                        artifact.producer_unit_id.clone().unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            Some(format!(
                "consumer|{}|{}|{}|{}",
                cargo_unit.package_id,
                rust_crate_name(&cargo_unit.target_name),
                cargo_unit.target_kind,
                consumed
            ))
        }));
    }
    facts.into_iter().collect()
}

fn cargo_host_oracle_digest(cargo_graph: &UnitDerivationGraphSummary, unit_graph: &Value) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(&comparable_cargo_host_facts(cargo_graph, unit_graph))
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo host-unit oracle facts: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_graph_digest(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    blockers: &[NativeHostUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        host_units: &'a [NativeHostUnitSummary],
        target_consumers: &'a [NativeHostTargetConsumerSummary],
        blockers: &'a [NativeHostUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        host_units,
        target_consumers,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native Rust host-unit graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_oracle_comparison_digest(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    cargo_graph: &UnitDerivationGraphSummary,
    unit_graph: &Value,
    blockers: &[NativeHostUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        native_facts: Vec<String>,
        cargo_facts: Vec<String>,
        blockers: &'a [NativeHostUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        native_facts: comparable_native_host_facts(host_units, target_consumers),
        cargo_facts: comparable_cargo_host_facts(cargo_graph, unit_graph),
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native host-unit graph oracle comparison: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_blocker(
    unit_id: Option<String>,
    package_id: Option<String>,
    class: &str,
    message: &str,
) -> NativeHostUnitGraphPlanningBlocker {
    NativeHostUnitGraphPlanningBlocker {
        unit_id,
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn summarize_native_unit_derivation_graph(
    native_unit_graph: &NativeUnitGraphPlanningSummary,
    native_host_unit_graph: Option<&NativeHostUnitGraphPlanningSummary>,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    let consumed_hosts_by_unit = native_host_unit_graph
        .map(|host_graph| {
            host_graph
                .target_consumers
                .iter()
                .map(|consumer| (consumer.unit_id.clone(), consumer.consumed_host_artifacts.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut derivations = native_unit_graph
        .units
        .iter()
        .map(|unit| {
            native_unit_derivation(
                unit,
                consumed_hosts_by_unit.get(&unit.unit_id).cloned().unwrap_or_default(),
                source_closure,
                options,
            )
        })
        .collect::<Vec<_>>();
    if let Some(host_graph) = native_host_unit_graph {
        derivations.extend(
            host_graph.host_units.iter().map(|unit| native_host_unit_derivation(unit, source_closure, options)),
        );
        add_host_dependency_derivations(&mut derivations);
    }
    derivations = dedupe_unit_derivations_by_unit_id(derivations);
    derivations.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    let blockers = Vec::new();
    let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
    let host_unit_count = derivations.iter().filter(|derivation| derivation.execution_kind == "host").count();
    let host_artifact_count = derivations
        .iter()
        .filter(|derivation| derivation.execution_kind == "host")
        .map(|derivation| usize::from(is_host_target_kind(&derivation.target_kind)))
        .sum();
    Ok(UnitDerivationGraphSummary {
        derivation_count: derivations.len(),
        host_unit_count,
        host_artifact_count,
        ready: true,
        digest_blake3,
        derivations,
        blockers,
    })
}

fn dedupe_unit_derivations_by_unit_id(derivations: Vec<RustUnitDerivationSummary>) -> Vec<RustUnitDerivationSummary> {
    let mut by_unit_id = BTreeMap::<String, RustUnitDerivationSummary>::new();
    for derivation in derivations {
        match by_unit_id.get(&derivation.unit_id) {
            Some(existing) if rust_derivation_source_is_preferred(existing, &derivation) => {}
            _ => {
                by_unit_id.insert(derivation.unit_id.clone(), derivation);
            }
        }
    }
    by_unit_id.into_values().collect()
}

fn rust_derivation_source_is_preferred(
    existing: &RustUnitDerivationSummary,
    candidate: &RustUnitDerivationSummary,
) -> bool {
    let existing_absolute = rustc_source_path(&existing.derivation.args).is_some_and(|path| path.is_absolute());
    let candidate_absolute = rustc_source_path(&candidate.derivation.args).is_some_and(|path| path.is_absolute());
    if existing_absolute != candidate_absolute {
        return existing_absolute;
    }
    let _ = candidate;
    true
}

fn append_rustc_metadata_args(args: &mut Vec<String>, metadata: &str) {
    debug_assert!(!metadata.is_empty());
    debug_assert!(metadata.bytes().all(|byte| byte.is_ascii_hexdigit()));
    args.push(RUSTC_CODEGEN_OPTION_FLAG.to_string());
    args.push(format!("{RUSTC_METADATA_ARG_PREFIX}{metadata}"));
}

fn rustc_unit_metadata_disambiguator(
    package_id: &str,
    target_name: &str,
    target_kind: &str,
    mode: &str,
    source_digest: &SourceDigest,
    selected_features: &[String],
    crate_types: &[String],
) -> String {
    debug_assert!(!package_id.is_empty());
    debug_assert!(!target_name.is_empty());
    debug_assert!(!target_kind.is_empty());
    debug_assert!(!mode.is_empty());
    debug_assert!(!source_digest.value.is_empty());
    let mut features = selected_features.to_vec();
    features.sort();
    let mut crate_types = crate_types.to_vec();
    crate_types.sort();
    let material = [
        package_id,
        target_name,
        target_kind,
        mode,
        source_digest.algorithm.as_str(),
        source_digest.value.as_str(),
        &features.join(","),
        &crate_types.join(","),
    ]
    .join("\0");
    let hex = blake3::hash(material.as_bytes()).to_hex().to_string();
    debug_assert!(hex.len() >= RUSTC_METADATA_HEX_CHARS);
    hex[..RUSTC_METADATA_HEX_CHARS].to_string()
}

fn native_unit_derivation(
    unit: &NativeRustUnitSummary,
    consumed_host_artifacts: Vec<RustHostArtifact>,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> RustUnitDerivationSummary {
    debug_assert!(!unit.package_name.is_empty());
    debug_assert!(!unit.package_root.is_empty());
    debug_assert!(!unit.cargo_package_env.is_empty());
    let mut args = vec![
        "--crate-name".to_string(),
        unit.crate_name.clone(),
        "--edition".to_string(),
        unit.edition.clone(),
        unit.source_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&unit.crate_types, &unit.target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    append_rustc_metadata_args(
        &mut args,
        &rustc_unit_metadata_disambiguator(
            &unit.package_id,
            &unit.target_name,
            &unit.target_kind,
            &unit.mode,
            &unit.source_digest,
            &unit.selected_features,
            &unit.crate_types,
        ),
    );
    append_rustc_feature_cfg_args(&mut args, &unit.selected_features);
    append_cap_lints_args(&mut args, &unit.package_id, source_closure);
    if unit.target_kind == "bin" {
        if let Some(linker) = resolve_tool_path("cc") {
            args.push("-C".to_string());
            args.push(format!("linker={}", normalize_path_string(&linker)));
        }
    }
    for dependency in &unit.dependency_artifacts {
        args.push("--extern".to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), unit.target_kind.clone());
    env.insert(RUST_UNIT_EXECUTION_KIND_ENV.to_string(), "target".to_string());
    env.insert("MODE".to_string(), unit.mode.clone());
    env.insert("PACKAGE_ID".to_string(), unit.package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    append_cargo_package_env(&mut env, &unit.cargo_package_env);
    env.insert(BUILD_SCRIPT_CARGO_PKG_NAME_ENV.to_string(), unit.package_name.clone());
    env.insert(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV.to_string(), unit.package_root.clone());
    if let Some(links) = &unit.package_links {
        env.insert(PACKAGE_LINKS_ENV.to_string(), links.clone());
    }
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let mut inputs = vec![format!("source:{}:{}", unit.package_id, unit.source_digest.value)];
    inputs.extend(unit.dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    RustUnitDerivationSummary {
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_kind: "target".to_string(),
        crate_types: unit.crate_types.clone(),
        mode: unit.mode.clone(),
        profile: options.profile.clone(),
        source_digest: unit.source_digest.clone(),
        dependency_artifacts: unit.dependency_artifacts.clone(),
        consumed_host_artifacts,
        metadata_dependencies: Vec::new(),
        generated_metadata: None,
        derivation: ReviewableRustDerivation {
            name: derivation_name(&unit.target_name, native_unit_index(&unit.unit_id)),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    }
}

fn native_host_unit_derivation(
    unit: &NativeHostUnitSummary,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> RustUnitDerivationSummary {
    debug_assert!(!unit.package_name.is_empty());
    debug_assert!(!unit.package_root.is_empty());
    debug_assert!(!unit.cargo_package_env.is_empty());
    let mut args = vec![
        "--crate-name".to_string(),
        unit.crate_name.clone(),
        "--edition".to_string(),
        unit.edition.clone(),
        unit.source_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&unit.crate_types, &unit.target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    append_rustc_metadata_args(
        &mut args,
        &rustc_unit_metadata_disambiguator(
            &unit.package_id,
            &unit.target_name,
            &unit.target_kind,
            &unit.mode,
            &unit.source_digest,
            &unit.selected_features,
            &unit.crate_types,
        ),
    );
    append_rustc_feature_cfg_args(&mut args, &unit.selected_features);
    append_cap_lints_args(&mut args, &unit.package_id, source_closure);
    if unit.target_kind == "proc-macro" {
        args.push(RUSTC_EXTERN_FLAG.to_string());
        args.push(RUSTC_PROC_MACRO_EXTERN.to_string());
    }
    if let Some(linker) = resolve_tool_path("cc") {
        args.push("-C".to_string());
        args.push(format!("linker={}", normalize_path_string(&linker)));
    }
    for dependency in &unit.dependency_artifacts {
        args.push(RUSTC_EXTERN_FLAG.to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), unit.target_kind.clone());
    env.insert(RUST_UNIT_EXECUTION_KIND_ENV.to_string(), "host".to_string());
    env.insert("MODE".to_string(), unit.mode.clone());
    env.insert("PACKAGE_ID".to_string(), unit.package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    append_cargo_package_env(&mut env, &unit.cargo_package_env);
    env.insert(BUILD_SCRIPT_CARGO_PKG_NAME_ENV.to_string(), unit.package_name.clone());
    env.insert(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV.to_string(), unit.package_root.clone());
    if let Some(links) = &unit.package_links {
        env.insert(PACKAGE_LINKS_ENV.to_string(), links.clone());
    }
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let mut inputs = vec![format!("source:{}:{}", unit.package_id, unit.source_digest.value)];
    inputs.extend(unit.dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(unit.consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    RustUnitDerivationSummary {
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_kind: "host".to_string(),
        crate_types: unit.crate_types.clone(),
        mode: unit.mode.clone(),
        profile: options.profile.clone(),
        source_digest: unit.source_digest.clone(),
        dependency_artifacts: unit.dependency_artifacts.clone(),
        consumed_host_artifacts: unit.consumed_host_artifacts.clone(),
        metadata_dependencies: unit.metadata_dependencies.clone(),
        generated_metadata: unit.generated_metadata.clone(),
        derivation: ReviewableRustDerivation {
            name: derivation_name(&unit.target_name, native_unit_index(&unit.unit_id)),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    }
}

fn add_host_dependency_derivations(derivations: &mut Vec<RustUnitDerivationSummary>) {
    let target_libs_by_unit_id = target_lib_derivations_by_unit_id(derivations);
    let target_libs_by_package = target_lib_derivation_ids_by_package(derivations);
    let required_target_ids =
        host_dependency_target_unit_ids(derivations, &target_libs_by_unit_id, &target_libs_by_package);
    if required_target_ids.is_empty() {
        return;
    }
    let host_id_by_target_id = required_target_ids
        .iter()
        .map(|target_id| (target_id.clone(), host_dependency_unit_id(target_id)))
        .collect::<BTreeMap<_, _>>();
    let host_id_by_package = target_libs_by_package
        .iter()
        .filter(|(_package_id, target_id)| required_target_ids.contains(*target_id))
        .map(|(package_id, target_id)| (package_id.clone(), host_dependency_unit_id(target_id)))
        .collect::<BTreeMap<_, _>>();
    for derivation in derivations.iter_mut() {
        if is_supported_host_unit(derivation) {
            remap_dependency_artifacts_to_host_variants(
                &mut derivation.dependency_artifacts,
                &host_id_by_target_id,
                &host_id_by_package,
            );
        }
    }
    for target_id in required_target_ids {
        let Some(target_unit) = target_libs_by_unit_id.get(&target_id) else {
            continue;
        };
        let host_unit = host_dependency_derivation(target_unit, &host_id_by_target_id, &host_id_by_package);
        derivations.push(host_unit);
    }
}

fn target_lib_derivations_by_unit_id(
    derivations: &[RustUnitDerivationSummary],
) -> BTreeMap<String, RustUnitDerivationSummary> {
    derivations
        .iter()
        .filter(|unit| is_supported_target_unit(unit) && unit.target_kind == "lib")
        .map(|unit| (unit.unit_id.clone(), unit.clone()))
        .collect()
}

fn target_lib_derivation_ids_by_package(derivations: &[RustUnitDerivationSummary]) -> BTreeMap<String, String> {
    let mut by_package = BTreeMap::new();
    let mut ambiguous_packages = BTreeSet::new();
    for unit in derivations.iter().filter(|unit| is_supported_target_unit(unit) && unit.target_kind == "lib") {
        match by_package.get(&unit.package_id) {
            Some(existing_unit_id) if existing_unit_id != &unit.unit_id => {
                ambiguous_packages.insert(unit.package_id.clone());
            }
            Some(_existing_unit_id) => {}
            None => {
                by_package.insert(unit.package_id.clone(), unit.unit_id.clone());
            }
        }
    }
    for package_id in ambiguous_packages {
        by_package.remove(&package_id);
    }
    by_package
}

fn host_dependency_target_unit_ids(
    derivations: &[RustUnitDerivationSummary],
    target_libs_by_unit_id: &BTreeMap<String, RustUnitDerivationSummary>,
    target_libs_by_package: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut queue = VecDeque::new();
    for unit in derivations.iter().filter(|unit| is_supported_host_unit(unit)) {
        enqueue_host_dependency_targets(
            &unit.dependency_artifacts,
            target_libs_by_unit_id,
            target_libs_by_package,
            &mut queue,
        );
    }
    let mut required = BTreeSet::new();
    while let Some(unit_id) = queue.pop_front() {
        if !required.insert(unit_id.clone()) {
            continue;
        }
        let Some(unit) = target_libs_by_unit_id.get(&unit_id) else {
            continue;
        };
        enqueue_host_dependency_targets(
            &unit.dependency_artifacts,
            target_libs_by_unit_id,
            target_libs_by_package,
            &mut queue,
        );
    }
    required
}

fn enqueue_host_dependency_targets(
    dependencies: &[RustDependencyArtifact],
    target_libs_by_unit_id: &BTreeMap<String, RustUnitDerivationSummary>,
    target_libs_by_package: &BTreeMap<String, String>,
    queue: &mut VecDeque<String>,
) {
    for dependency in dependencies {
        if let Some(unit_id) =
            target_lib_unit_id_for_dependency(dependency, target_libs_by_unit_id, target_libs_by_package)
        {
            queue.push_back(unit_id);
        }
    }
}

fn target_lib_unit_id_for_dependency(
    dependency: &RustDependencyArtifact,
    target_libs_by_unit_id: &BTreeMap<String, RustUnitDerivationSummary>,
    target_libs_by_package: &BTreeMap<String, String>,
) -> Option<String> {
    if let Some(producer_unit_id) = &dependency.producer_unit_id {
        if target_libs_by_unit_id.contains_key(producer_unit_id) {
            return Some(producer_unit_id.clone());
        }
    }
    target_libs_by_package.get(&dependency.package_id).cloned()
}

fn host_dependency_derivation(
    target_unit: &RustUnitDerivationSummary,
    host_id_by_target_id: &BTreeMap<String, String>,
    host_id_by_package: &BTreeMap<String, String>,
) -> RustUnitDerivationSummary {
    debug_assert!(is_supported_target_unit(target_unit));
    debug_assert_eq!(target_unit.target_kind, "lib");
    let mut host_unit = target_unit.clone();
    host_unit.unit_id = host_dependency_unit_id(&target_unit.unit_id);
    host_unit.execution_kind = HOST_DEPENDENCY_EXECUTION_KIND.to_string();
    host_unit.mode = HOST_DEPENDENCY_MODE.to_string();
    host_unit.derivation.name = format!("{}-host", target_unit.derivation.name);
    host_unit.derivation.env.insert("CRATE_KIND".to_string(), HOST_DEPENDENCY_CRATE_KIND.to_string());
    host_unit
        .derivation
        .env
        .insert(RUST_UNIT_EXECUTION_KIND_ENV.to_string(), HOST_DEPENDENCY_EXECUTION_KIND.to_string());
    host_unit.derivation.env.insert("MODE".to_string(), HOST_DEPENDENCY_MODE.to_string());
    remap_dependency_artifacts_to_host_variants(
        &mut host_unit.dependency_artifacts,
        host_id_by_target_id,
        host_id_by_package,
    );
    refresh_derivation_dependency_inputs(&mut host_unit);
    host_unit
}

fn remap_dependency_artifacts_to_host_variants(
    dependencies: &mut [RustDependencyArtifact],
    host_id_by_target_id: &BTreeMap<String, String>,
    host_id_by_package: &BTreeMap<String, String>,
) {
    for dependency in dependencies {
        if let Some(producer_unit_id) = &dependency.producer_unit_id {
            if let Some(host_unit_id) = host_id_by_target_id.get(producer_unit_id) {
                dependency.producer_unit_id = Some(host_unit_id.clone());
                continue;
            }
        }
        if let Some(host_unit_id) = host_id_by_package.get(&dependency.package_id) {
            dependency.producer_unit_id = Some(host_unit_id.clone());
        }
    }
}

fn refresh_derivation_dependency_inputs(unit: &mut RustUnitDerivationSummary) {
    let mut inputs = vec![format!("source:{}:{}", unit.package_id, unit.source_digest.value)];
    inputs.extend(unit.dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(unit.consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    unit.derivation.inputs = inputs;
}

fn host_dependency_unit_id(target_unit_id: &str) -> String {
    debug_assert!(!target_unit_id.is_empty());
    format!("{target_unit_id}{HOST_DEPENDENCY_UNIT_ID_SUFFIX}")
}

fn append_cargo_package_env(env: &mut BTreeMap<String, String>, package_env: &BTreeMap<String, String>) {
    for (key, value) in package_env {
        debug_assert!(key.starts_with("CARGO_PKG_") || key == BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV);
        env.insert(key.clone(), value.clone());
    }
}

fn native_unit_index(unit_id: &str) -> usize {
    unit_id.split(':').next().and_then(|value| value.parse::<usize>().ok()).unwrap_or(0)
}

fn cargo_unit_graph_oracle_digest(unit_graph: &Value) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(unit_graph)
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo unit graph oracle: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_graph_digest(
    units: &[NativeRustUnitSummary],
    blockers: &[NativeUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        units: &'a [NativeRustUnitSummary],
        blockers: &'a [NativeUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { units, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing native Rust unit graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_oracle_comparison_digest(
    native_units: &[NativeRustUnitSummary],
    cargo_units: &[RustUnitDerivationSummary],
    blockers: &[NativeUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        native_facts: Vec<String>,
        cargo_facts: Vec<String>,
        blockers: &'a [NativeUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        native_facts: comparable_native_unit_facts(native_units),
        cargo_facts: comparable_cargo_unit_facts(cargo_units),
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native unit graph oracle comparison: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_blocker(
    unit_id: Option<String>,
    package_id: Option<String>,
    class: &str,
    message: &str,
) -> NativeUnitGraphPlanningBlocker {
    NativeUnitGraphPlanningBlocker {
        unit_id,
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
fn summarize_unit_derivation_graph(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    summarize_unit_derivation_graph_with_native(unit_graph, source_closure, options, None, None)
}

fn summarize_unit_derivation_graph_with_native(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
    native_unit_graph: Option<&NativeUnitGraphPlanningSummary>,
    native_host_unit_graph: Option<&NativeHostUnitGraphPlanningSummary>,
) -> Result<UnitDerivationGraphSummary, RunError> {
    if let Some(native_unit_graph) = native_unit_graph {
        if !native_unit_graph.ready {
            return blocked_unit_derivation_graph_from_native(native_unit_graph, native_host_unit_graph);
        }
        if let Some(native_host_unit_graph) = native_host_unit_graph {
            if !native_host_unit_graph.ready {
                return blocked_unit_derivation_graph_from_native(native_unit_graph, Some(native_host_unit_graph));
            }
            return summarize_native_unit_derivation_graph(
                native_unit_graph,
                Some(native_host_unit_graph),
                source_closure,
                options,
            );
        }
        return summarize_native_unit_derivation_graph(native_unit_graph, None, source_closure, options);
    }
    summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)
}

fn blocked_unit_derivation_graph_from_native(
    native_unit_graph: &NativeUnitGraphPlanningSummary,
    native_host_unit_graph: Option<&NativeHostUnitGraphPlanningSummary>,
) -> Result<UnitDerivationGraphSummary, RunError> {
    let mut blockers = native_unit_graph
        .blockers
        .iter()
        .map(|blocker| UnitDerivationBlocker {
            unit_id: blocker.unit_id.clone().unwrap_or_else(|| "native-unit-graph".to_string()),
            package_id: blocker.package_id.clone(),
            class: blocker.class.clone(),
            message: blocker.message.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(native_host_unit_graph) = native_host_unit_graph {
        blockers.extend(native_host_unit_graph.blockers.iter().map(|blocker| UnitDerivationBlocker {
            unit_id: blocker.unit_id.clone().unwrap_or_else(|| "native-host-unit-graph".to_string()),
            package_id: blocker.package_id.clone(),
            class: blocker.class.clone(),
            message: blocker.message.clone(),
        }));
    }
    blockers.sort_by(|left, right| left.unit_id.cmp(&right.unit_id).then(left.class.cmp(&right.class)));
    blockers.dedup_by(|left, right| {
        left.unit_id == right.unit_id
            && left.package_id == right.package_id
            && left.class == right.class
            && left.message == right.message
    });
    let digest_blake3 = unit_derivation_graph_digest(&[], &blockers)?;
    Ok(UnitDerivationGraphSummary {
        derivation_count: 0,
        host_unit_count: 0,
        host_artifact_count: 0,
        ready: false,
        digest_blake3,
        derivations: Vec::new(),
        blockers,
    })
}

fn summarize_cargo_unit_derivation_graph(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    let mut derivations = Vec::new();
    let mut blockers = source_closure
        .blockers
        .iter()
        .map(|blocker| UnitDerivationBlocker {
            unit_id: blocker.package_id.clone(),
            package_id: Some(blocker.package_id.clone()),
            class: "source-closure-blocked".to_string(),
            message: blocker.message.clone(),
        })
        .collect::<Vec<_>>();
    let Some(units) = unit_graph.get("units").and_then(Value::as_array) else {
        blockers.push(UnitDerivationBlocker {
            unit_id: "unit-graph".to_string(),
            package_id: None,
            class: "missing-units".to_string(),
            message: "Cargo unit graph did not contain a units array".to_string(),
        });
        let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
        return Ok(UnitDerivationGraphSummary {
            derivation_count: 0,
            host_unit_count: 0,
            host_artifact_count: 0,
            ready: false,
            digest_blake3,
            derivations,
            blockers,
        });
    };

    let host_artifacts = host_artifacts_by_package(units);
    for (index, unit) in units.iter().enumerate() {
        if is_custom_build_run_unit(unit) {
            continue;
        }
        match summarize_unit_derivation(index, unit, units, source_closure, options, &host_artifacts) {
            Ok(derivation) => derivations.push(derivation),
            Err(blocker) => blockers.push(blocker),
        }
    }
    derivations.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    blockers.sort_by(|left, right| left.unit_id.cmp(&right.unit_id).then(left.class.cmp(&right.class)));
    let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
    let host_unit_count = derivations.iter().filter(|derivation| derivation.execution_kind == "host").count();
    let host_artifact_count = host_artifacts.values().map(Vec::len).sum();
    Ok(UnitDerivationGraphSummary {
        derivation_count: derivations.len(),
        host_unit_count,
        host_artifact_count,
        ready: blockers.is_empty(),
        digest_blake3,
        derivations,
        blockers,
    })
}

fn summarize_unit_derivation(
    index: usize,
    unit: &Value,
    units: &[Value],
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
    host_artifacts: &BTreeMap<String, Vec<RustHostArtifact>>,
) -> Result<RustUnitDerivationSummary, UnitDerivationBlocker> {
    let package_id = required_unit_string(unit, "pkg_id", index)?;
    let target = unit
        .get("target")
        .ok_or_else(|| unit_blocker(index, Some(package_id.clone()), "missing-target", "unit lacks a target object"))?;
    let target_name = required_target_string(target, "name", index, &package_id)?;
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode != "build" {
        return Err(unit_blocker(
            index,
            Some(package_id.clone()),
            "unsupported-unit-mode",
            "only Cargo build-mode units are supported before explicit test/doctest/run derivation modeling lands",
        ));
    }
    let target_kind = select_supported_target_kind(target, index, &package_id)?;
    let execution_kind = if is_host_target_kind(&target_kind) {
        "host"
    } else {
        "target"
    }
    .to_string();
    let crate_types = target_string_array(target, "crate_types");
    let edition = target_string(target, "edition").unwrap_or_else(|| DEFAULT_RUST_EDITION.to_string());
    let src_path = required_target_string(target, "src_path", index, &package_id)?;
    let features = unit_string_array(unit, "features");
    let source = source_closure.sources.iter().find(|source| source.package_id == package_id).ok_or_else(|| {
        unit_blocker(
            index,
            Some(package_id.clone()),
            "missing-source-input",
            "unit package is absent from source closure",
        )
    })?;

    let dependency_artifacts = unit_dependency_artifacts(unit, units);
    let consumed_host_artifacts = if execution_kind == "host" {
        Vec::new()
    } else {
        consumed_host_artifacts(unit, units, host_artifacts)
    };
    let generated_metadata =
        (target_kind == "custom-build").then(|| build_script_metadata_summary(&package_id, &target_name));
    let mut args = vec![
        "--crate-name".to_string(),
        rust_crate_name(&target_name),
        "--edition".to_string(),
        edition,
        src_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&crate_types, &target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    if is_host_target_kind(&target_kind) || target_kind == "bin" {
        if let Some(linker) = resolve_tool_path("cc") {
            args.push("-C".to_string());
            args.push(format!("linker={}", normalize_path_string(&linker)));
        }
    }
    append_rustc_feature_cfg_args(&mut args, &features);
    append_cap_lints_args_for_source_kind(&mut args, &source.kind);
    for dependency in &dependency_artifacts {
        args.push("--extern".to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), target_kind.clone());
    env.insert(RUST_UNIT_EXECUTION_KIND_ENV.to_string(), execution_kind.clone());
    env.insert("MODE".to_string(), mode.clone());
    env.insert("PACKAGE_ID".to_string(), package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let mut inputs = vec![format!("source:{}:{}", package_id, source.source_digest.value)];
    inputs.extend(dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    let unit_id = rust_unit_id(index, &package_id, &target_name, &target_kind, &mode);

    Ok(RustUnitDerivationSummary {
        unit_id: unit_id.clone(),
        package_id: package_id.clone(),
        target_name: target_name.clone(),
        target_kind,
        execution_kind: execution_kind.clone(),
        crate_types,
        mode,
        profile: options.profile.clone(),
        source_digest: source.source_digest.clone(),
        dependency_artifacts,
        consumed_host_artifacts,
        metadata_dependencies: Vec::new(),
        generated_metadata,
        derivation: ReviewableRustDerivation {
            name: derivation_name(&target_name, index),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    })
}

fn select_supported_target_kind(
    target: &Value,
    index: usize,
    package_id: &str,
) -> Result<String, UnitDerivationBlocker> {
    let kinds = target_string_array(target, "kind");
    let crate_types = target_string_array(target, "crate_types");
    if let Some(target_kind) = classify_supported_cargo_target_kind(&kinds, &crate_types) {
        return Ok(target_kind.to_string());
    }
    let class = if kinds.is_empty() {
        "missing-target-kind"
    } else {
        "unsupported-target-kind"
    };
    let message = if kinds.is_empty() {
        "unit target lacks a kind array".to_string()
    } else {
        format!(
            "target kinds [{}] are outside Mantle's currently supported lib/bin Rust derivation subset",
            kinds.join(",")
        )
    };
    Err(unit_blocker(index, Some(package_id.to_string()), class, &message))
}

fn classify_supported_cargo_target_kind(kinds: &[String], crate_types: &[String]) -> Option<&'static str> {
    let proc_macro_crate_type = crate_types.iter().any(|crate_type| crate_type == "proc-macro");
    let lib_shaped_kind = kinds.iter().any(|kind| kind == "lib" || kind == "rlib");
    let classified = if kinds.iter().any(|kind| kind == "custom-build") {
        Some("custom-build")
    } else if kinds.iter().any(|kind| kind == "proc-macro") {
        Some("proc-macro")
    } else if proc_macro_crate_type && lib_shaped_kind {
        Some("proc-macro")
    } else if lib_shaped_kind {
        Some("lib")
    } else if kinds.iter().any(|kind| kind == "bin") {
        Some("bin")
    } else {
        None
    };
    if proc_macro_crate_type && lib_shaped_kind {
        debug_assert_ne!(classified, Some("lib"));
        debug_assert_ne!(classified, Some("bin"));
    }
    if let Some(kind) = classified {
        debug_assert!(!kind.is_empty());
    }
    classified
}

fn host_artifacts_by_package(units: &[Value]) -> BTreeMap<String, Vec<RustHostArtifact>> {
    let mut artifacts: BTreeMap<String, Vec<RustHostArtifact>> = BTreeMap::new();
    for (index, unit) in units.iter().enumerate() {
        if is_custom_build_run_unit(unit) {
            continue;
        }
        let Some(package_id) = target_string(unit, "pkg_id") else {
            continue;
        };
        let Some(target) = unit.get("target") else {
            continue;
        };
        let kinds = target_string_array(target, "kind");
        let crate_types = target_string_array(target, "crate_types");
        let Some(target_kind) = classify_supported_cargo_target_kind(&kinds, &crate_types) else {
            continue;
        };
        if !is_host_target_kind(target_kind) {
            continue;
        }
        let target_name = target_string(target, "name").unwrap_or_else(|| format!("host-unit-{index}"));
        let metadata_digest_blake3 = (target_kind == "custom-build")
            .then(|| build_script_metadata_summary(&package_id, &target_name).digest_blake3);
        artifacts.entry(package_id.clone()).or_default().push(RustHostArtifact {
            package_id: package_id.clone(),
            target_name: target_name.clone(),
            target_kind: target_kind.to_string(),
            producer_unit_id: rust_unit_id_from_unit_value(index, unit),
            artifact: format!("host-artifact:{index}:{target_kind}:{}", rust_crate_name(&target_name)),
            metadata_digest_blake3,
        });
    }
    for values in artifacts.values_mut() {
        values.sort();
        values.dedup();
    }
    artifacts
}

fn is_custom_build_run_unit(unit: &Value) -> bool {
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode == "build" {
        return false;
    }
    unit.get("target")
        .map(|target| target_string_array(target, "kind").iter().any(|kind| kind == "custom-build"))
        .unwrap_or(false)
}

fn unit_dependency_array(unit: &Value) -> Option<&Vec<Value>> {
    unit.get("deps")
        .and_then(Value::as_array)
        .or_else(|| unit.get("dependencies").and_then(Value::as_array))
}

fn consumed_host_artifacts(
    unit: &Value,
    units: &[Value],
    host_artifacts: &BTreeMap<String, Vec<RustHostArtifact>>,
) -> Vec<RustHostArtifact> {
    let mut artifacts = Vec::new();
    if let Some(package_id) = target_string(unit, "pkg_id") {
        artifacts.extend(host_artifacts.get(&package_id).into_iter().flatten().cloned());
    }
    if let Some(deps) = unit_dependency_array(unit) {
        for dep in deps {
            if let Some(package_id) = dependency_package_id(dep, units) {
                artifacts.extend(host_artifacts.get(package_id).into_iter().flatten().cloned());
            }
        }
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn build_script_metadata_summary(package_id: &str, target_name: &str) -> BuildScriptMetadataSummary {
    let out_dir = format!("host-metadata:{package_id}:{target_name}:OUT_DIR");
    let rustc_cfg = Vec::new();
    let rustc_env = BTreeMap::new();
    let rustc_link_lib = Vec::new();
    let rustc_link_search = Vec::new();
    let rerun_if_changed = Vec::new();
    let metadata = BTreeMap::new();
    #[derive(Serialize)]
    struct Hashable<'a> {
        out_dir: &'a str,
        rustc_cfg: &'a [String],
        rustc_env: &'a BTreeMap<String, String>,
        rustc_link_lib: &'a [String],
        rustc_link_search: &'a [String],
        rerun_if_changed: &'a [String],
        metadata: &'a BTreeMap<String, String>,
    }
    let canonical = serde_json::to_vec(&Hashable {
        out_dir: &out_dir,
        rustc_cfg: &rustc_cfg,
        rustc_env: &rustc_env,
        rustc_link_lib: &rustc_link_lib,
        rustc_link_search: &rustc_link_search,
        rerun_if_changed: &rerun_if_changed,
        metadata: &metadata,
    })
    .unwrap_or_default();
    BuildScriptMetadataSummary {
        out_dir,
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
        metadata,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    }
}

fn is_host_target_kind(target_kind: &str) -> bool {
    matches!(target_kind, "custom-build" | "proc-macro")
}

fn unit_dependency_artifacts(unit: &Value, units: &[Value]) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    let Some(deps) = unit_dependency_array(unit) else {
        return artifacts;
    };
    for dep in deps {
        if dependency_producer_target_kind(dep, units).is_some_and(|target_kind| is_host_target_kind(&target_kind)) {
            continue;
        }
        let Some(package_id) = dependency_package_id(dep, units) else {
            continue;
        };
        let name = dep
            .get("extern_crate_name")
            .and_then(Value::as_str)
            .or_else(|| dep.get("name").and_then(Value::as_str))
            .map(rust_crate_name)
            .unwrap_or_else(|| "unknown_dep".to_string());
        let artifact = format!("artifact:{package_id}:{name}");
        artifacts.push(RustDependencyArtifact {
            package_id: package_id.to_string(),
            name,
            producer_unit_id: dependency_producer_unit_id(dep, units),
            artifact,
        });
    }
    artifacts.sort_by(|left, right| left.package_id.cmp(&right.package_id).then(left.name.cmp(&right.name)));
    artifacts.dedup();
    artifacts
}

fn dependency_package_id<'a>(dep: &'a Value, units: &'a [Value]) -> Option<&'a str> {
    dep.get("pkg_id")
        .and_then(Value::as_str)
        .or_else(|| dep.get("package_id").and_then(Value::as_str))
        .or_else(|| {
            let index = dep.get("index").and_then(Value::as_u64)?;
            let unit = units.get(usize::try_from(index).ok()?)?;
            unit.get("pkg_id").and_then(Value::as_str)
        })
}

fn dependency_producer_unit_id(dep: &Value, units: &[Value]) -> Option<String> {
    let index = usize::try_from(dep.get("index")?.as_u64()?).ok()?;
    let unit = units.get(index)?;
    rust_unit_id_from_unit_value(index, unit)
}

fn dependency_producer_target_kind(dep: &Value, units: &[Value]) -> Option<String> {
    let index = usize::try_from(dep.get("index")?.as_u64()?).ok()?;
    let unit = units.get(index)?;
    let target = unit.get("target")?;
    let kinds = target_string_array(target, "kind");
    let crate_types = target_string_array(target, "crate_types");
    classify_supported_cargo_target_kind(&kinds, &crate_types).map(str::to_string)
}

fn normalized_crate_types(crate_types: &[String], target_kind: &str) -> Vec<String> {
    let mut normalized = if crate_types.is_empty() {
        vec![target_kind.to_string()]
    } else {
        crate_types.to_vec()
    };
    normalized.sort();
    normalized.dedup();
    normalized
}

fn rustc_crate_types(crate_types: &[String], target_kind: &str) -> Vec<String> {
    if target_kind == "custom-build" {
        return vec!["bin".to_string()];
    }
    normalized_crate_types(crate_types, target_kind)
}

fn rustc_feature_cfg_arg(feature: &str) -> String {
    debug_assert!(!feature.is_empty());
    let escaped = feature.replace('\\', "\\\\").replace('"', "\\\"");
    format!("feature=\"{escaped}\"")
}

fn append_rustc_feature_cfg_args(args: &mut Vec<String>, selected_features: &[String]) {
    for feature in selected_features {
        args.push(RUSTC_CFG_FLAG.to_string());
        args.push(rustc_feature_cfg_arg(feature));
    }
}

fn append_cap_lints_args(args: &mut Vec<String>, package_id: &str, source_closure: &SourceClosureSummary) {
    let Some(source_kind) = source_closure
        .sources
        .iter()
        .find(|source| source.package_id == package_id)
        .map(|source| &source.kind)
    else {
        return;
    };
    append_cap_lints_args_for_source_kind(args, source_kind);
}

fn append_cap_lints_args_for_source_kind(args: &mut Vec<String>, source_kind: &SourceKind) {
    if !source_kind_needs_cap_lints(source_kind) {
        return;
    }
    args.push(RUSTC_CAP_LINTS_FLAG.to_string());
    args.push(RUSTC_CAP_LINTS_ALLOW.to_string());
}

fn source_kind_needs_cap_lints(source_kind: &SourceKind) -> bool {
    matches!(source_kind, SourceKind::Registry | SourceKind::Git)
}

fn rust_unit_id(index: usize, package_id: &str, target_name: &str, target_kind: &str, mode: &str) -> String {
    format!("{index}:{package_id}:{target_name}:{target_kind}:{mode}")
}

fn rust_unit_id_from_unit_value(index: usize, unit: &Value) -> Option<String> {
    if is_custom_build_run_unit(unit) {
        return None;
    }
    let package_id = target_string(unit, "pkg_id")?;
    let target = unit.get("target")?;
    let target_name = target_string(target, "name")?;
    let kinds = target_string_array(target, "kind");
    let crate_types = target_string_array(target, "crate_types");
    let target_kind = classify_supported_cargo_target_kind(&kinds, &crate_types)?;
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode != "build" {
        return None;
    }
    Some(rust_unit_id(index, &package_id, &target_name, target_kind, &mode))
}

fn rustc_crate_name_from_args(args: &[String]) -> Option<String> {
    args.windows(RUSTC_EXTERN_ARG_PAIR_WIDTH).find_map(|window| {
        if window[0] == "--crate-name" {
            return Some(window[1].clone());
        }
        None
    })
}

fn derivation_name(target_name: &str, index: usize) -> String {
    format!("rust-unit-{index}-{}", rust_crate_name(target_name))
}

fn rust_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

fn required_unit_string(unit: &Value, field: &str, index: usize) -> Result<String, UnitDerivationBlocker> {
    target_string(unit, field)
        .ok_or_else(|| unit_blocker(index, None, &format!("missing-{field}"), &format!("unit lacks `{field}`")))
}

fn required_target_string(
    target: &Value,
    field: &str,
    index: usize,
    package_id: &str,
) -> Result<String, UnitDerivationBlocker> {
    target_string(target, field).ok_or_else(|| {
        unit_blocker(
            index,
            Some(package_id.to_string()),
            &format!("missing-target-{field}"),
            &format!("unit target lacks `{field}`"),
        )
    })
}

fn target_string(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(ToString::to_string)
}

fn unit_string_array(value: &Value, field: &str) -> Vec<String> {
    target_string_array(value, field)
}

fn target_string_array(value: &Value, field: &str) -> Vec<String> {
    let mut values = value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    values.sort();
    values.dedup();
    values
}

fn unit_blocker(index: usize, package_id: Option<String>, class: &str, message: &str) -> UnitDerivationBlocker {
    UnitDerivationBlocker {
        unit_id: index.to_string(),
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn empty_unit_derivation_graph() -> Result<UnitDerivationGraphSummary, RunError> {
    let derivations = Vec::new();
    let blockers = Vec::new();
    Ok(UnitDerivationGraphSummary {
        derivation_count: 0,
        host_unit_count: 0,
        host_artifact_count: 0,
        ready: true,
        digest_blake3: unit_derivation_graph_digest(&derivations, &blockers)?,
        derivations,
        blockers,
    })
}

fn unit_derivation_graph_digest(
    derivations: &[RustUnitDerivationSummary],
    blockers: &[UnitDerivationBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        derivations: &'a [RustUnitDerivationSummary],
        blockers: &'a [UnitDerivationBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { derivations, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit derivation graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

pub(crate) fn execute_first_supported_rust_unit(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let Some(unit) = graph.derivations.iter().find(|unit| is_supported_target_unit(unit)) else {
        return blocked_execution_receipt(
            None,
            "missing-supported-unit",
            "unit_derivation_graph does not contain a supported target lib/bin unit",
        );
    };
    if !graph.ready {
        return blocked_execution_receipt(
            Some(unit),
            "unit-derivation-graph-blocked",
            "unit_derivation_graph is not ready; resolve planning blockers before execution",
        );
    }
    execute_rust_unit(unit, options)
}

pub(crate) fn execute_first_rust_unit_dependency_chain(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitDependencyChainExecutionReceipt, RunError> {
    if !graph.ready {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message:
                    "unit_derivation_graph is not ready; resolve planning blockers before dependency-chain execution"
                        .to_string(),
            }),
        );
    }
    let Some(consumer) = graph
        .derivations
        .iter()
        .find(|unit| is_supported_target_unit(unit) && !unit.dependency_artifacts.is_empty())
    else {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-dependency-chain".to_string(),
                message: "unit_derivation_graph does not contain a supported target unit with dependency artifacts"
                    .to_string(),
            }),
        );
    };
    if !consumer.consumed_host_artifacts.is_empty() {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unsupported-chain-shape".to_string(),
                message:
                    "bounded dependency-chain execution does not yet support host/proc-macro/build-script artifacts"
                        .to_string(),
            }),
        );
    }
    if consumer.dependency_artifacts.len() != 1 {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unsupported-chain-shape".to_string(),
                message:
                    "bounded dependency-chain execution currently supports exactly one producer dependency artifact"
                        .to_string(),
            }),
        );
    }
    let dependency = &consumer.dependency_artifacts[0];
    let Some(producer) = graph.derivations.iter().find(|unit| {
        is_supported_target_unit(unit)
            && unit.package_id == dependency.package_id
            && unit.target_kind == "lib"
            && unit.dependency_artifacts.is_empty()
            && unit.consumed_host_artifacts.is_empty()
    }) else {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-dependency-producer".to_string(),
                message: format!("no supported producer lib unit for dependency package {}", dependency.package_id),
            }),
        );
    };

    let producer_receipt = execute_rust_unit(producer, options)?;
    if producer_receipt.execution_status != "success" {
        return dependency_chain_receipt(
            "blocked",
            vec![producer_receipt],
            Some(RustUnitExecutionBlocker {
                class: "dependency-producer-failed".to_string(),
                message: format!("producer unit {} did not produce a successful artifact", producer.unit_id),
            }),
        );
    }
    let producer_artifact = match produced_library_artifact_path(producer, options)? {
        Ok(path) => path,
        Err(blocker) => return dependency_chain_receipt("blocked", vec![producer_receipt], Some(blocker)),
    };
    let bound_consumer = bind_dependency_artifact(consumer, dependency, &producer_artifact)?;
    let consumer_receipt = execute_rust_unit(&bound_consumer, options)?;
    let status = if consumer_receipt.execution_status == "success" {
        "success"
    } else {
        "blocked"
    };
    let blocker = consumer_receipt.blocker.clone();
    dependency_chain_receipt(status, vec![producer_receipt, consumer_receipt], blocker)
}

pub(crate) fn execute_rust_target_unit_topology(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitTargetTopologyExecutionReceipt, RunError> {
    if !graph.ready {
        return target_topology_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message:
                    "unit_derivation_graph is not ready; resolve planning blockers before target-topology execution"
                        .to_string(),
            }),
        );
    }

    let mut selected_indices = Vec::new();
    for (index, unit) in graph.derivations.iter().enumerate() {
        if is_supported_target_unit(unit) {
            if !unit.consumed_host_artifacts.is_empty() {
                return target_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "unsupported-topology-shape".to_string(),
                        message: format!(
                            "target topology execution does not yet support host/proc-macro/build-script artifacts for unit {}",
                            unit.unit_id
                        ),
                    }),
                );
            }
            selected_indices.push(index);
        }
    }
    if selected_indices.is_empty() {
        return target_topology_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-supported-unit".to_string(),
                message: "unit_derivation_graph does not contain supported target lib/bin units".to_string(),
            }),
        );
    }

    let selected_set = selected_indices.iter().copied().collect::<BTreeSet<_>>();
    let unit_indices_by_id = rust_unit_indices_by_id(graph);
    let mut lib_producers = BTreeMap::new();
    for index in &selected_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }

    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &selected_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            let producer_index =
                match target_dependency_producer_index(dependency, graph, &unit_indices_by_id, &lib_producers) {
                    Ok(index) => index,
                    Err(blocker) => return target_topology_receipt("blocked", Vec::new(), Some(blocker)),
                };
            if !selected_set.contains(&producer_index) {
                return target_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "dependency producer for package {} is outside the selected target topology",
                            dependency.package_id
                        ),
                    }),
                );
            }
            if producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let mut ordered_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &selected_indices {
        if let Err(blocker) =
            visit_target_topology_unit(*index, &edges, &mut temporary, &mut permanent, &mut ordered_indices, graph)
        {
            return target_topology_receipt("blocked", Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut produced_artifacts = ProducedArtifactIndex::default();
    for index in ordered_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = if unit.dependency_artifacts.is_empty() {
            unit.clone()
        } else {
            match bind_all_dependency_artifacts_with_index(unit, &produced_artifacts) {
                Ok(bound) => bound,
                Err(blocker) => return target_topology_receipt("blocked", executions, Some(blocker)),
            }
        };
        if !unit.dependency_artifacts.is_empty() {
            if let Err(blocker) =
                append_selected_dependency_search_paths(&mut executable_unit, graph, &produced_artifacts)
            {
                return target_topology_receipt("blocked", executions, Some(blocker));
            }
        }
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "target-unit-failed".to_string(),
                message: format!("target topology unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return target_topology_receipt("blocked", executions, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_artifacts.record_unit(unit, path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return target_topology_receipt("blocked", executions, Some(blocker));
                }
            }
        }
        executions.push(receipt);
    }

    target_topology_receipt("success", executions, None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostDependencyTopologyPlan {
    target_dependency_indices: BTreeSet<usize>,
    host_edges: BTreeMap<usize, Vec<usize>>,
}

fn rust_unit_indices_by_id(graph: &UnitDerivationGraphSummary) -> BTreeMap<String, usize> {
    let mut indices = BTreeMap::new();
    for (index, unit) in graph.derivations.iter().enumerate() {
        indices.insert(unit.unit_id.clone(), index);
    }
    indices
}

fn target_dependency_producer_index(
    dependency: &RustDependencyArtifact,
    graph: &UnitDerivationGraphSummary,
    unit_indices_by_id: &BTreeMap<String, usize>,
    lib_producers: &BTreeMap<String, usize>,
) -> Result<usize, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &dependency.producer_unit_id {
        let Some(index) = unit_indices_by_id.get(producer_unit_id).copied() else {
            return Err(RustUnitExecutionBlocker {
                class: "missing-dependency-producer".to_string(),
                message: format!(
                    "no selected producer unit {producer_unit_id} for dependency package {}",
                    dependency.package_id
                ),
            });
        };
        let producer = &graph.derivations[index];
        if is_supported_target_unit(producer)
            || is_supported_host_unit(producer)
            || is_supported_host_dependency_unit(producer)
        {
            return Ok(index);
        }
        return Err(RustUnitExecutionBlocker {
            class: "unsupported-dependency-producer".to_string(),
            message: format!(
                "selected producer unit {} for dependency package {} is not a supported Rust unit",
                producer_unit_id, dependency.package_id
            ),
        });
    }
    lib_producers.get(&dependency.package_id).copied().ok_or_else(|| RustUnitExecutionBlocker {
        class: "missing-dependency-producer".to_string(),
        message: format!("no supported producer lib unit for dependency package {}", dependency.package_id),
    })
}

fn plan_host_dependency_topology(
    host_indices: &[usize],
    lib_producers: &BTreeMap<String, usize>,
    proc_macro_producers: &BTreeMap<String, usize>,
    target_edges: &BTreeMap<usize, Vec<usize>>,
    graph: &UnitDerivationGraphSummary,
) -> Result<HostDependencyTopologyPlan, RustUnitExecutionBlocker> {
    let unit_indices_by_id = rust_unit_indices_by_id(graph);
    let mut target_dependency_indices = BTreeSet::new();
    let mut host_edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in host_indices {
        let unit = &graph.derivations[*index];
        let mut host_dependencies = Vec::new();
        for dependency in &unit.dependency_artifacts {
            if let Some(producer_unit_id) = &dependency.producer_unit_id {
                let Some(producer_index) = unit_indices_by_id.get(producer_unit_id).copied() else {
                    return Err(RustUnitExecutionBlocker {
                        class: "missing-host-dependency-producer".to_string(),
                        message: format!(
                            "no selected producer unit {} for host dependency package {}",
                            producer_unit_id, dependency.package_id
                        ),
                    });
                };
                let producer = &graph.derivations[producer_index];
                if is_supported_target_unit(producer) || is_supported_host_dependency_unit(producer) {
                    collect_target_dependencies(producer_index, target_edges, &mut target_dependency_indices);
                    host_dependencies.push(producer_index);
                    continue;
                }
                if is_supported_host_unit(producer) {
                    if producer_index != *index {
                        host_dependencies.push(producer_index);
                    }
                    continue;
                }
                return Err(RustUnitExecutionBlocker {
                    class: "unsupported-host-dependency-producer".to_string(),
                    message: format!(
                        "selected producer unit {} for host dependency package {} is not supported",
                        producer_unit_id, dependency.package_id
                    ),
                });
            }
            if let Some(producer_index) = lib_producers.get(&dependency.package_id).copied() {
                collect_target_dependencies(producer_index, target_edges, &mut target_dependency_indices);
                host_dependencies.push(producer_index);
                continue;
            }
            if let Some(producer_index) = proc_macro_producers.get(&dependency.package_id).copied() {
                if producer_index != *index {
                    host_dependencies.push(producer_index);
                }
                continue;
            }
            return Err(RustUnitExecutionBlocker {
                class: "missing-host-dependency-producer".to_string(),
                message: format!(
                    "no supported host dependency lib or proc-macro host producer for host dependency package {}",
                    dependency.package_id
                ),
            });
        }
        host_dependencies.sort_unstable();
        host_dependencies.dedup();
        host_edges.insert(*index, host_dependencies);
    }
    Ok(HostDependencyTopologyPlan {
        target_dependency_indices,
        host_edges,
    })
}

fn collect_target_dependencies(index: usize, edges: &BTreeMap<usize, Vec<usize>>, collected: &mut BTreeSet<usize>) {
    if !collected.insert(index) {
        return;
    }
    if let Some(deps) = edges.get(&index) {
        for dep in deps {
            collect_target_dependencies(*dep, edges, collected);
        }
    }
}

fn plan_combined_unit_topology_order(
    target_indices: &[usize],
    host_indices: &[usize],
    lib_producers: &BTreeMap<String, usize>,
    _host_producers: &BTreeMap<String, usize>,
    proc_macro_producers: &BTreeMap<String, usize>,
    target_edges: &BTreeMap<usize, Vec<usize>>,
    graph: &UnitDerivationGraphSummary,
) -> Result<Vec<usize>, RustUnitExecutionBlocker> {
    let host_dependency_plan =
        plan_host_dependency_topology(host_indices, lib_producers, proc_macro_producers, target_edges, graph)?;
    let mut combined_edges = target_edges.clone();
    for index in &host_dependency_plan.target_dependency_indices {
        debug_assert!(combined_edges.contains_key(index));
    }
    for (index, deps) in host_dependency_plan.host_edges {
        combined_edges.entry(index).or_default().extend(deps);
    }
    let build_metadata_producers = build_script_metadata_producers(graph, host_indices);
    let participant_indices = combined_topology_participant_indices(host_indices, target_indices, target_edges);
    for index in &participant_indices {
        let unit = &graph.derivations[*index];
        for metadata_dependency in &unit.metadata_dependencies {
            let producer_index = select_build_script_metadata_producer_index(
                unit,
                metadata_dependency,
                &build_metadata_producers,
                graph,
            )?;
            if producer_index != *index {
                combined_edges.entry(*index).or_default().push(producer_index);
            }
        }
    }
    for index in &participant_indices {
        let unit = &graph.derivations[*index];
        for host_artifact in &unit.consumed_host_artifacts {
            let Some(producer_index) = host_artifact_producer_index(host_indices, graph, host_artifact) else {
                return Err(RustUnitExecutionBlocker {
                    class: "missing-host-artifact-producer".to_string(),
                    message: format!(
                        "no supported {} host producer unit for host artifact package {}",
                        host_artifact.target_kind, host_artifact.package_id
                    ),
                });
            };
            if producer_index != *index {
                combined_edges.entry(*index).or_default().push(producer_index);
            }
        }
    }
    for deps in combined_edges.values_mut() {
        deps.sort_unstable();
        deps.dedup();
    }

    let mut ordered_unit_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &participant_indices {
        visit_target_topology_unit(
            *index,
            &combined_edges,
            &mut temporary,
            &mut permanent,
            &mut ordered_unit_indices,
            graph,
        )?;
    }
    Ok(ordered_unit_indices)
}

fn combined_topology_participant_indices(
    host_indices: &[usize],
    target_indices: &[usize],
    target_edges: &BTreeMap<usize, Vec<usize>>,
) -> Vec<usize> {
    let mut indices = BTreeSet::new();
    indices.extend(host_indices.iter().copied());
    indices.extend(target_indices.iter().copied());
    indices.extend(target_edges.keys().copied());
    for deps in target_edges.values() {
        indices.extend(deps.iter().copied());
    }
    indices.into_iter().collect()
}

fn build_script_metadata_producers(
    graph: &UnitDerivationGraphSummary,
    host_indices: &[usize],
) -> BuildScriptMetadataProducerIndex {
    let mut producers = BuildScriptMetadataProducerIndex::default();
    for index in host_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "custom-build" {
            producers.by_unit_id.insert(unit.unit_id.clone(), *index);
            let package_producers = producers.by_package.entry(unit.package_id.clone()).or_default();
            if !package_producers.contains(index) {
                package_producers.push(*index);
            }
            package_producers.sort_unstable();
        }
    }
    producers
}

fn select_build_script_metadata_producer_index(
    unit: &RustUnitDerivationSummary,
    metadata_dependency: &BuildScriptMetadataDependency,
    producers: &BuildScriptMetadataProducerIndex,
    graph: &UnitDerivationGraphSummary,
) -> Result<usize, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &metadata_dependency.producer_unit_id {
        return producers.by_unit_id.get(producer_unit_id).copied().ok_or_else(|| RustUnitExecutionBlocker {
            class: "missing-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} requires selected build-script metadata producer {} for linked dependency package {}",
                unit.unit_id, producer_unit_id, metadata_dependency.package_id
            ),
        });
    }
    let candidates = producers.by_package.get(&metadata_dependency.package_id).cloned().unwrap_or_default();
    let candidates = collapse_build_script_alias_producer_candidates(candidates, graph);
    match candidates.as_slice() {
        [] => Err(RustUnitExecutionBlocker {
            class: "missing-build-script-metadata-producer".to_string(),
            message: format!(
                "no supported build-script metadata producer for linked dependency package {}",
                metadata_dependency.package_id
            ),
        }),
        [candidate] => Ok(*candidate),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} linked metadata dependency package {} has {} candidate custom-build producers: {}",
                unit.unit_id,
                metadata_dependency.package_id,
                candidates.len(),
                candidates
                    .iter()
                    .map(|index| graph.derivations[*index].unit_id.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }),
    }
}

fn collapse_build_script_alias_producer_candidates(
    candidates: Vec<usize>,
    graph: &UnitDerivationGraphSummary,
) -> Vec<usize> {
    let mut alias_candidates_by_name = BTreeMap::<String, Vec<usize>>::new();
    let mut collapsed_candidates = Vec::new();
    for index in candidates {
        let unit = &graph.derivations[index];
        if is_build_script_target_alias(&unit.target_name) {
            alias_candidates_by_name.entry(build_script_alias_name(unit)).or_default().push(index);
        } else {
            collapsed_candidates.push(index);
        }
    }
    if should_collapse_build_script_alias_groups(&alias_candidates_by_name) {
        if let Some(candidate) = preferred_build_script_alias_producer_candidate(&alias_candidates_by_name, graph) {
            collapsed_candidates.push(candidate);
        }
    } else {
        collapsed_candidates.extend(alias_candidates_by_name.into_values().flatten());
    }
    collapsed_candidates.sort_unstable();
    collapsed_candidates.dedup();
    collapsed_candidates
}

fn should_collapse_build_script_alias_groups<T>(alias_groups: &BTreeMap<String, Vec<T>>) -> bool {
    alias_groups.len() > UNIQUE_CANDIDATE_COUNT
        && alias_groups.values().all(|candidates| candidates.len() == UNIQUE_CANDIDATE_COUNT)
}

fn preferred_build_script_alias_producer_candidate(
    alias_candidates_by_name: &BTreeMap<String, Vec<usize>>,
    graph: &UnitDerivationGraphSummary,
) -> Option<usize> {
    alias_candidates_by_name
        .values()
        .filter_map(|candidates| candidates.first().copied())
        .min_by_key(|index| {
            let unit = &graph.derivations[*index];
            (build_script_alias_preference(&build_script_alias_name(unit)), unit.unit_id.clone())
        })
}

fn build_script_alias_name(unit: &RustUnitDerivationSummary) -> String {
    if unit.unit_id.contains(":build-script-main:custom-build:") {
        return BUILD_SCRIPT_MAIN_TARGET_NAME.to_string();
    }
    if unit.unit_id.contains(":build-script-build:custom-build:") {
        return BUILD_SCRIPT_TARGET_NAME.to_string();
    }
    unit.target_name.clone()
}

fn is_build_script_target_alias(target_name: &str) -> bool {
    target_name == BUILD_SCRIPT_TARGET_NAME || target_name == BUILD_SCRIPT_MAIN_TARGET_NAME
}

fn build_script_alias_preference(target_name: &str) -> u8 {
    if target_name == BUILD_SCRIPT_TARGET_NAME {
        return 0;
    }
    1
}

fn host_artifact_producer_index(
    host_indices: &[usize],
    graph: &UnitDerivationGraphSummary,
    host_artifact: &RustHostArtifact,
) -> Option<usize> {
    if let Some(producer_unit_id) = &host_artifact.producer_unit_id {
        return host_indices.iter().copied().find(|index| graph.derivations[*index].unit_id == *producer_unit_id);
    }
    host_indices.iter().copied().find(|index| {
        let unit = &graph.derivations[*index];
        unit.package_id == host_artifact.package_id
            && unit.target_kind == host_artifact.target_kind
            && rust_crate_name(&unit.target_name) == rust_crate_name(&host_artifact.target_name)
    })
}

pub(crate) fn execute_rust_unit_topology(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitTopologyExecutionReceipt, RunError> {
    if let Some(blocker) = validate_native_registry_topology_inputs(native_registry_sources, graph) {
        return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if let Some(blocker) = validate_native_host_artifact_topology_inputs(native_host_graph, graph, "topology") {
        return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if !graph.ready {
        return topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before topology execution"
                    .to_string(),
            }),
        );
    }

    let host_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_host_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    let host_dependency_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_host_dependency_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    let target_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_target_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    if target_indices.is_empty() {
        return topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-supported-unit".to_string(),
                message: "unit_derivation_graph does not contain supported target lib/bin units".to_string(),
            }),
        );
    }

    let mut host_producers = BTreeMap::<String, usize>::new();
    let mut proc_macro_producers = BTreeMap::<String, usize>::new();
    for index in &host_indices {
        let unit = &graph.derivations[*index];
        host_producers.entry(unit.package_id.clone()).or_insert(*index);
        if unit.target_kind == "proc-macro" {
            proc_macro_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        for host_artifact in &unit.consumed_host_artifacts {
            if !host_producers.contains_key(&host_artifact.package_id) {
                return topology_receipt(
                    "blocked",
                    Vec::new(),
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-host-artifact-producer".to_string(),
                        message: format!(
                            "no supported host producer unit for host artifact package {}",
                            host_artifact.package_id
                        ),
                    }),
                );
            }
        }
    }

    let selected_set = target_indices.iter().copied().collect::<BTreeSet<_>>();
    let host_dependency_set = host_dependency_indices.iter().copied().collect::<BTreeSet<_>>();
    let unit_indices_by_id = rust_unit_indices_by_id(graph);
    let mut target_lib_producers = BTreeMap::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            target_lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }
    let mut host_dependency_lib_producers = BTreeMap::new();
    for index in &host_dependency_indices {
        let unit = &graph.derivations[*index];
        host_dependency_lib_producers.entry(unit.package_id.clone()).or_insert(*index);
    }
    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            let producer_index =
                match target_dependency_producer_index(dependency, graph, &unit_indices_by_id, &target_lib_producers) {
                    Ok(index) => index,
                    Err(blocker) => {
                        if dependency.producer_unit_id.is_none() && host_producers.contains_key(&dependency.package_id)
                        {
                            continue;
                        }
                        return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
                    }
                };
            if selected_set.contains(&producer_index) && producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }
    for index in &host_dependency_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            let producer_index = match target_dependency_producer_index(
                dependency,
                graph,
                &unit_indices_by_id,
                &host_dependency_lib_producers,
            ) {
                Ok(index) => index,
                Err(blocker) => return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker)),
            };
            if host_dependency_set.contains(&producer_index) && producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let ordered_unit_indices = match plan_combined_unit_topology_order(
        &target_indices,
        &host_indices,
        &host_dependency_lib_producers,
        &host_producers,
        &proc_macro_producers,
        &edges,
        graph,
    ) {
        Ok(order) => order,
        Err(blocker) => return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker)),
    };

    let mut executions = Vec::new();
    let mut build_script_metadata_runs = Vec::new();
    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut produced_target_artifact_index = ProducedArtifactIndex::default();
    let mut produced_host_dependency_artifact_index = ProducedArtifactIndex::default();
    let mut produced_host_artifacts = ProducedHostArtifactIndex::default();
    let mut produced_proc_macro_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut produced_proc_macro_artifact_index = ProducedArtifactIndex::default();
    let mut produced_build_script_metadata = ProducedBuildScriptMetadataIndex::default();
    for index in ordered_unit_indices {
        let unit = &graph.derivations[index];
        if is_supported_host_dependency_unit(unit) {
            let mut executable_unit = match bind_all_host_artifacts_with_index(unit, &produced_host_artifacts) {
                Ok(bound) => bound,
                Err(blocker) => {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            };
            executable_unit =
                match bind_all_build_script_metadata_with_index(&executable_unit, &produced_build_script_metadata) {
                    Ok(bound) => bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                };
            let mut produced_dependency_artifacts = produced_host_dependency_artifact_index.clone();
            produced_dependency_artifacts.extend_from(&produced_proc_macro_artifact_index);
            if !executable_unit.dependency_artifacts.is_empty() {
                match bind_all_dependency_artifacts_with_index(&executable_unit, &produced_dependency_artifacts) {
                    Ok(bound) => executable_unit = bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                }
            }
            if !unit.dependency_artifacts.is_empty() || !unit.consumed_host_artifacts.is_empty() {
                if let Err(blocker) =
                    append_selected_dependency_search_paths(&mut executable_unit, graph, &produced_dependency_artifacts)
                {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            }
            let receipt = execute_rust_unit(&executable_unit, options)?;
            if receipt.execution_status != "success" {
                let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                    class: "host-dependency-unit-failed".to_string(),
                    message: format!("host dependency unit {} did not execute successfully", unit.unit_id),
                });
                executions.push(receipt);
                return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
            }
            if unit.target_kind == "lib" {
                match produced_library_artifact_path(&executable_unit, options)? {
                    Ok(path) => {
                        produced_host_dependency_artifact_index.record_unit(unit, path);
                    }
                    Err(blocker) => {
                        executions.push(receipt);
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                }
            }
            executions.push(receipt);
            continue;
        }

        if is_supported_host_unit(unit) {
            let mut executable_unit = match bind_all_host_artifacts_with_index(unit, &produced_host_artifacts) {
                Ok(bound) => bound,
                Err(blocker) => {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            };
            let mut produced_dependency_artifacts = produced_host_dependency_artifact_index.clone();
            produced_dependency_artifacts.extend_from(&produced_proc_macro_artifact_index);
            if !executable_unit.dependency_artifacts.is_empty() {
                match bind_all_dependency_artifacts_with_index(&executable_unit, &produced_dependency_artifacts) {
                    Ok(bound) => executable_unit = bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                }
            }
            if !unit.dependency_artifacts.is_empty() || !unit.consumed_host_artifacts.is_empty() {
                if let Err(blocker) =
                    append_selected_dependency_search_paths(&mut executable_unit, graph, &produced_dependency_artifacts)
                {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            }
            executable_unit =
                match bind_all_build_script_metadata_with_index(&executable_unit, &produced_build_script_metadata) {
                    Ok(bound) => bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                };
            let receipt = execute_rust_unit(&executable_unit, options)?;
            if receipt.execution_status != "success" {
                let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                    class: "host-unit-failed".to_string(),
                    message: format!("host unit {} did not execute successfully", unit.unit_id),
                });
                executions.push(receipt);
                return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
            }
            match produced_host_artifact_path(unit, options)? {
                Ok(path) => {
                    if unit.target_kind == "custom-build" {
                        match run_build_script_metadata(&executable_unit, options, &path)? {
                            Ok(metadata_run) => {
                                produced_build_script_metadata
                                    .record_unit(unit, build_script_metadata_from_run(&metadata_run));
                                build_script_metadata_runs.push(metadata_run);
                            }
                            Err(blocker) => {
                                executions.push(receipt);
                                return topology_receipt(
                                    "blocked",
                                    executions,
                                    build_script_metadata_runs,
                                    Some(blocker),
                                );
                            }
                        }
                    }
                    if unit.target_kind == "proc-macro" {
                        produced_proc_macro_artifact_index.record_unit(unit, path.clone());
                        produced_proc_macro_artifacts.insert(unit.package_id.clone(), path.clone());
                    }
                    produced_host_artifacts.record_unit(unit, path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            }
            executions.push(receipt);
            continue;
        }

        if is_supported_target_unit(unit) {
            let mut executable_unit = match bind_all_host_artifacts_with_index(unit, &produced_host_artifacts) {
                Ok(bound) => bound,
                Err(blocker) => {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            };
            executable_unit =
                match bind_all_build_script_metadata_with_index(&executable_unit, &produced_build_script_metadata) {
                    Ok(bound) => bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                };
            let mut produced_dependency_artifacts = produced_target_artifact_index.clone();
            produced_dependency_artifacts.extend_from(&produced_proc_macro_artifact_index);
            if !executable_unit.dependency_artifacts.is_empty() {
                match bind_all_dependency_artifacts_with_index(&executable_unit, &produced_dependency_artifacts) {
                    Ok(bound) => executable_unit = bound,
                    Err(blocker) => {
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                }
            }
            if !unit.dependency_artifacts.is_empty() || !unit.consumed_host_artifacts.is_empty() {
                if let Err(blocker) =
                    append_selected_dependency_search_paths(&mut executable_unit, graph, &produced_dependency_artifacts)
                {
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            }
            let receipt = execute_rust_unit(&executable_unit, options)?;
            if receipt.execution_status != "success" {
                let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                    class: "target-unit-failed".to_string(),
                    message: format!("target unit {} did not execute successfully", unit.unit_id),
                });
                executions.push(receipt);
                return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
            }
            if unit.target_kind == "lib" {
                match produced_library_artifact_path(&executable_unit, options)? {
                    Ok(path) => {
                        produced_target_artifact_index.record_unit(unit, path.clone());
                        produced_target_artifacts.insert(unit.package_id.clone(), path);
                    }
                    Err(blocker) => {
                        executions.push(receipt);
                        return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                    }
                }
            }
            executions.push(receipt);
        }
    }

    topology_receipt("success", executions, build_script_metadata_runs, None)
}

pub(crate) fn execute_native_rust_dev_dependency_test_topology(
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustDevDependencyTestTopologyExecutionReceipt, RunError> {
    if !native_package_target_planning.ready {
        return dev_dependency_test_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "dev-dependency-test-planning-not-ready".to_string(),
                message: "native_rust_dev_dependency_test_topology_execution requires ready native package/dev-dependency planning evidence".to_string(),
            }),
        );
    }
    if !graph.ready {
        return dev_dependency_test_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before dev-dependency test topology execution".to_string(),
            }),
        );
    }

    let candidate_packages = native_package_target_planning
        .packages
        .iter()
        .filter(|package| !package.dev_dependencies.is_empty())
        .collect::<Vec<_>>();
    if candidate_packages.len() != 1 {
        return dev_dependency_test_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "ambiguous-dev-dependency-test-topology".to_string(),
                message: format!(
                    "expected exactly one package with dev-dependencies for bounded execution, found {}",
                    candidate_packages.len()
                ),
            }),
        );
    }
    let package = candidate_packages[0];
    let manifest_path = Path::new(&package.manifest_path);
    let source_root = manifest_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("package manifest path has no parent: {}", package.manifest_path)))?;
    let manifest = match read_native_manifest(manifest_path) {
        Ok(manifest) => manifest,
        Err(message) => {
            return dev_dependency_test_topology_receipt(
                "blocked",
                Some(package.package_id.clone()),
                None,
                package.dev_dependencies.iter().map(|dep| dep.name.clone()).collect(),
                Vec::new(),
                Some(RustUnitExecutionBlocker {
                    class: "unreadable-test-manifest".to_string(),
                    message,
                }),
            );
        }
    };
    let supported_tests = manifest.test.iter().filter(|test| !test.harness).collect::<Vec<_>>();
    if supported_tests.len() != 1 {
        return dev_dependency_test_topology_receipt(
            "blocked",
            Some(package.package_id.clone()),
            None,
            package.dev_dependencies.iter().map(|dep| dep.name.clone()).collect(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unsupported-cargo-test-harness".to_string(),
                message: "bounded dev-dependency test execution requires exactly one explicit [[test]] target with harness = false".to_string(),
            }),
        );
    }
    let test = supported_tests[0];
    let test_name = test.name.as_deref().unwrap_or("dev_dependency_test");
    let test_source = source_root.join(test.path.as_deref().unwrap_or(&format!("tests/{test_name}.rs")));
    if !test_source.is_file() {
        return dev_dependency_test_topology_receipt(
            "blocked",
            Some(package.package_id.clone()),
            Some(test_name.to_string()),
            package.dev_dependencies.iter().map(|dep| dep.name.clone()).collect(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-test-unit-derivation".to_string(),
                message: format!("declared test target source is not readable: {}", test_source.display()),
            }),
        );
    }

    let packages_by_manifest = native_package_target_planning
        .packages
        .iter()
        .map(|candidate| (normalize_path_string(Path::new(&candidate.manifest_path)), candidate))
        .collect::<BTreeMap<_, _>>();
    let mut executions = Vec::new();
    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut dependency_artifacts = Vec::new();
    let mut dev_dependency_packages = Vec::new();

    for dev_dependency in &package.dev_dependencies {
        let Some(dev_package) =
            packages_by_manifest.get(&normalize_path_string(Path::new(&dev_dependency.manifest_path)))
        else {
            return dev_dependency_test_topology_receipt(
                "blocked",
                Some(package.package_id.clone()),
                Some(test_name.to_string()),
                dev_dependency_packages,
                executions,
                Some(RustUnitExecutionBlocker {
                    class: "missing-dev-dependency-source".to_string(),
                    message: format!("dev dependency `{}` has no native package source facts", dev_dependency.name),
                }),
            );
        };
        let producer = match graph
            .derivations
            .iter()
            .find(|unit| unit.package_id == dev_package.package_id && unit.target_kind == "lib" && unit.mode == "build")
            .cloned()
        {
            Some(producer) => producer,
            None => match dev_dependency_lib_derivation(dev_package, &dev_dependency.name) {
                Ok(producer) => producer,
                Err(blocker) => {
                    return dev_dependency_test_topology_receipt(
                        "blocked",
                        Some(package.package_id.clone()),
                        Some(test_name.to_string()),
                        dev_dependency_packages,
                        executions,
                        Some(blocker),
                    );
                }
            },
        };
        let executable_producer = bind_all_dependency_artifacts(&producer, &produced_target_artifacts)?;
        let receipt = execute_rust_unit(&executable_producer, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "dev-dependency-producer-failed".to_string(),
                message: format!("dev dependency producer {} did not execute successfully", producer.unit_id),
            });
            executions.push(receipt);
            return dev_dependency_test_topology_receipt(
                "blocked",
                Some(package.package_id.clone()),
                Some(test_name.to_string()),
                dev_dependency_packages,
                executions,
                Some(blocker),
            );
        }
        match produced_library_artifact_path(&executable_producer, options)? {
            Ok(path) => {
                produced_target_artifacts.insert(producer.package_id.clone(), path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return dev_dependency_test_topology_receipt(
                    "blocked",
                    Some(package.package_id.clone()),
                    Some(test_name.to_string()),
                    dev_dependency_packages,
                    executions,
                    Some(blocker),
                );
            }
        }
        dependency_artifacts.push(RustDependencyArtifact {
            package_id: producer.package_id.clone(),
            name: dev_dependency.name.clone(),
            producer_unit_id: Some(producer.unit_id.clone()),
            artifact: format!("artifact:{}:{}", producer.package_id, dev_dependency.name),
        });
        dev_dependency_packages.push(producer.package_id.clone());
        executions.push(receipt);
    }
    dependency_artifacts.sort();
    dependency_artifacts.dedup();
    dev_dependency_packages.sort();
    dev_dependency_packages.dedup();

    let mut args = vec![
        "--crate-name".to_string(),
        rust_crate_name(test_name),
        "--edition".to_string(),
        native_package_test_edition(package),
        normalize_path_string(&test_source),
        "--emit=link".to_string(),
        "--crate-type".to_string(),
        "bin".to_string(),
    ];
    if let Some(linker) = resolve_tool_path("cc") {
        args.push("-C".to_string());
        args.push(format!("linker={}", normalize_path_string(&linker)));
    }
    for dependency in &dependency_artifacts {
        args.push("--extern".to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let unit_id = format!("native-dev-dependency-test:{}:{}", package.package_id, test_name);
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), "test".to_string());
    env.insert("MODE".to_string(), "test".to_string());
    env.insert("PACKAGE_ID".to_string(), package.package_id.clone());
    env.insert("PROFILE".to_string(), DEFAULT_CARGO_PROFILE.to_string());
    let mut inputs = vec![format!("source:{}:{}", package.package_id, package.source_digest.value)];
    inputs.extend(dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    let mut test_unit = RustUnitDerivationSummary {
        unit_id: unit_id.clone(),
        package_id: package.package_id.clone(),
        target_name: test_name.to_string(),
        target_kind: "test".to_string(),
        execution_kind: "target".to_string(),
        crate_types: vec!["bin".to_string()],
        mode: "test".to_string(),
        profile: DEFAULT_CARGO_PROFILE.to_string(),
        source_digest: package.source_digest.clone(),
        dependency_artifacts,
        consumed_host_artifacts: Vec::new(),
        metadata_dependencies: Vec::new(),
        generated_metadata: None,
        derivation: ReviewableRustDerivation {
            name: derivation_name(test_name, 0),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: String::new(),
    };
    test_unit = bind_all_dependency_artifacts(&test_unit, &produced_target_artifacts)?;
    test_unit.rustc_args_digest_blake3 =
        blake3::hash(test_unit.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    let receipt = execute_rust_unit(&test_unit, options)?;
    if receipt.execution_status != "success" {
        let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
            class: "dev-dependency-test-unit-failed".to_string(),
            message: format!("dev-dependency test unit {unit_id} did not execute successfully"),
        });
        executions.push(receipt);
        return dev_dependency_test_topology_receipt(
            "blocked",
            Some(package.package_id.clone()),
            Some(test_name.to_string()),
            dev_dependency_packages,
            executions,
            Some(blocker),
        );
    }
    executions.push(receipt);
    dev_dependency_test_topology_receipt(
        "success",
        Some(package.package_id.clone()),
        Some(test_name.to_string()),
        dev_dependency_packages,
        executions,
        None,
    )
}

pub(crate) fn execute_native_registry_workspace_dependency_topology(
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustWorkspaceDependencyTopologyExecutionReceipt, RunError> {
    if !native_package_target_planning.ready {
        return workspace_dependency_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "workspace-dependency-planning-not-ready".to_string(),
                message: "native_registry_workspace_dependency_topology_execution requires ready native_registry_workspace_dependency_topology_planning evidence".to_string(),
            }),
        );
    }
    if !graph.ready {
        return workspace_dependency_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before workspace-dependency topology execution".to_string(),
            }),
        );
    }

    let candidate_packages = native_package_target_planning
        .packages
        .iter()
        .filter(|package| package.workspace_dependencies.iter().any(|dependency| dependency.decision == "selected"))
        .collect::<Vec<_>>();
    if candidate_packages.len() != 1 {
        return workspace_dependency_topology_receipt(
            "blocked",
            None,
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "ambiguous-workspace-dependency-topology".to_string(),
                message: format!(
                    "expected exactly one package with selected workspace-inherited dependencies for bounded execution, found {}",
                    candidate_packages.len()
                ),
            }),
        );
    }
    let package = candidate_packages[0];
    let selected_dependencies = package
        .workspace_dependencies
        .iter()
        .filter(|dependency| dependency.decision == "selected")
        .collect::<Vec<_>>();
    if selected_dependencies.is_empty() {
        return workspace_dependency_topology_receipt(
            "blocked",
            None,
            Some(package.package_id.clone()),
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-workspace-dependency-selection".to_string(),
                message: format!("package {} has no selected workspace-inherited dependency facts", package.package_id),
            }),
        );
    }

    let packages_by_manifest = native_package_target_planning
        .packages
        .iter()
        .map(|candidate| (normalize_path_string(Path::new(&candidate.manifest_path)), candidate))
        .collect::<BTreeMap<_, _>>();
    let mut executions = Vec::new();
    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut inherited_dependency_packages = Vec::new();

    for dependency in selected_dependencies {
        let Some(manifest_path) = dependency.manifest_path.as_deref() else {
            return workspace_dependency_topology_receipt(
                "blocked",
                Some(dependency.workspace_root.clone()),
                Some(package.package_id.clone()),
                inherited_dependency_packages,
                executions,
                Some(RustUnitExecutionBlocker {
                    class: "missing-workspace-dependency-source".to_string(),
                    message: format!("workspace dependency `{}` has no manifest path", dependency.dependency_key),
                }),
            );
        };
        let Some(dependency_package) = packages_by_manifest.get(&normalize_path_string(Path::new(manifest_path)))
        else {
            return workspace_dependency_topology_receipt(
                "blocked",
                Some(dependency.workspace_root.clone()),
                Some(package.package_id.clone()),
                inherited_dependency_packages,
                executions,
                Some(RustUnitExecutionBlocker {
                    class: "missing-workspace-dependency-source".to_string(),
                    message: format!(
                        "workspace dependency `{}` has no native package source facts",
                        dependency.dependency_key
                    ),
                }),
            );
        };
        let Some(producer) = graph.derivations.iter().find(|unit| {
            unit.package_id == dependency_package.package_id && unit.target_kind == "lib" && unit.mode == "build"
        }) else {
            return workspace_dependency_topology_receipt(
                "blocked",
                Some(dependency.workspace_root.clone()),
                Some(package.package_id.clone()),
                inherited_dependency_packages,
                executions,
                Some(RustUnitExecutionBlocker {
                    class: "missing-workspace-dependency-artifact".to_string(),
                    message: format!(
                        "workspace dependency `{}` has no supported lib derivation",
                        dependency.dependency_key
                    ),
                }),
            );
        };
        let executable_producer = bind_all_dependency_artifacts(producer, &produced_target_artifacts)?;
        let receipt = execute_rust_unit(&executable_producer, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "workspace-dependency-producer-failed".to_string(),
                message: format!("workspace dependency producer {} did not execute successfully", producer.unit_id),
            });
            executions.push(receipt);
            return workspace_dependency_topology_receipt(
                "blocked",
                Some(dependency.workspace_root.clone()),
                Some(package.package_id.clone()),
                inherited_dependency_packages,
                executions,
                Some(blocker),
            );
        }
        match produced_library_artifact_path(&executable_producer, options)? {
            Ok(path) => {
                produced_target_artifacts.insert(producer.package_id.clone(), path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return workspace_dependency_topology_receipt(
                    "blocked",
                    Some(dependency.workspace_root.clone()),
                    Some(package.package_id.clone()),
                    inherited_dependency_packages,
                    executions,
                    Some(blocker),
                );
            }
        }
        inherited_dependency_packages.push(producer.package_id.clone());
        executions.push(receipt);
    }
    inherited_dependency_packages.sort();
    inherited_dependency_packages.dedup();

    let Some(consumer) = graph.derivations.iter().find(|unit| {
        unit.package_id == package.package_id
            && matches!(unit.target_kind.as_str(), "lib" | "bin")
            && unit.mode == "build"
    }) else {
        return workspace_dependency_topology_receipt(
            "blocked",
            package.workspace_dependencies.first().map(|dependency| dependency.workspace_root.clone()),
            Some(package.package_id.clone()),
            inherited_dependency_packages,
            executions,
            Some(RustUnitExecutionBlocker {
                class: "missing-workspace-dependency-consumer".to_string(),
                message: format!(
                    "workspace dependency consumer {} has no supported lib/bin derivation",
                    package.package_id
                ),
            }),
        );
    };
    let executable_consumer = bind_all_dependency_artifacts(consumer, &produced_target_artifacts)?;
    let receipt = execute_rust_unit(&executable_consumer, options)?;
    if receipt.execution_status != "success" {
        let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
            class: "workspace-dependency-consumer-failed".to_string(),
            message: format!("workspace dependency consumer {} did not execute successfully", consumer.unit_id),
        });
        executions.push(receipt);
        return workspace_dependency_topology_receipt(
            "blocked",
            package.workspace_dependencies.first().map(|dependency| dependency.workspace_root.clone()),
            Some(package.package_id.clone()),
            inherited_dependency_packages,
            executions,
            Some(blocker),
        );
    }
    executions.push(receipt);
    workspace_dependency_topology_receipt(
        "success",
        package.workspace_dependencies.first().map(|dependency| dependency.workspace_root.clone()),
        Some(package.package_id.clone()),
        inherited_dependency_packages,
        executions,
        None,
    )
}

pub(crate) fn execute_native_registry_patch_source_topology(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustPatchSourceTopologyExecutionReceipt, RunError> {
    if !native_registry_sources.ready {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "patch-source-planning-not-ready".to_string(),
                message: "native_registry_patch_source_topology_execution requires ready native_registry_patch_source_planning evidence".to_string(),
            }),
        );
    }
    if !native_package_target_planning.ready {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "native-package-planning-not-ready".to_string(),
                message: "native package/target planning is not ready; resolve planning blockers before patch-source topology execution".to_string(),
            }),
        );
    }
    if !graph.ready {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before patch-source topology execution".to_string(),
            }),
        );
    }

    let patch_sources = native_registry_sources
        .sources
        .iter()
        .filter(|source| source.source_class == "patch-path")
        .collect::<Vec<_>>();
    if patch_sources.len() != 1 {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "ambiguous-patch-source-topology".to_string(),
                message: format!(
                    "expected exactly one local patch source for bounded execution, found {}",
                    patch_sources.len()
                ),
            }),
        );
    }
    let patch_source = patch_sources[0];
    let packages_by_manifest = native_package_target_planning
        .packages
        .iter()
        .map(|candidate| (normalize_path_string(Path::new(&candidate.manifest_path)), candidate))
        .collect::<BTreeMap<_, _>>();
    let Some(patch_package) = packages_by_manifest.get(&normalize_path_string(Path::new(&patch_source.manifest_path)))
    else {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-patch-source-package".to_string(),
                message: format!("patch source {} has no native package facts", patch_source.package_id),
            }),
        );
    };
    let candidate_consumers = native_package_target_planning
        .packages
        .iter()
        .filter(|package| {
            package.package_id != patch_package.package_id
                && package.path_dependencies.iter().any(|dependency| {
                    manifest_paths_same(&dependency.manifest_path, Path::new(&patch_source.manifest_path))
                })
        })
        .collect::<Vec<_>>();
    if candidate_consumers.len() != 1 {
        return patch_source_topology_receipt(
            "blocked",
            None,
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "ambiguous-patch-source-consumer".to_string(),
                message: format!(
                    "expected exactly one consumer for bounded patch-source execution, found {}",
                    candidate_consumers.len()
                ),
            }),
        );
    }
    let consumer_package = candidate_consumers[0];
    let Some(producer) = graph
        .derivations
        .iter()
        .find(|unit| unit.package_id == patch_package.package_id && unit.target_kind == "lib" && unit.mode == "build")
    else {
        return patch_source_topology_receipt(
            "blocked",
            Some(consumer_package.package_id.clone()),
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-patch-source-artifact".to_string(),
                message: format!("patch source {} has no supported lib derivation", patch_source.package_id),
            }),
        );
    };
    let Some(consumer) = graph.derivations.iter().find(|unit| {
        unit.package_id == consumer_package.package_id
            && matches!(unit.target_kind.as_str(), "lib" | "bin")
            && unit.mode == "build"
    }) else {
        return patch_source_topology_receipt(
            "blocked",
            Some(consumer_package.package_id.clone()),
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-patch-source-consumer".to_string(),
                message: format!(
                    "patch source consumer {} has no supported lib/bin derivation",
                    consumer_package.package_id
                ),
            }),
        );
    };

    let mut executions = Vec::new();
    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    let executable_producer = bind_all_dependency_artifacts(producer, &produced_target_artifacts)?;
    let receipt = execute_rust_unit(&executable_producer, options)?;
    if receipt.execution_status != "success" {
        let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
            class: "patch-source-producer-failed".to_string(),
            message: format!("patch source producer {} did not execute successfully", producer.unit_id),
        });
        executions.push(receipt);
        return patch_source_topology_receipt(
            "blocked",
            Some(consumer_package.package_id.clone()),
            Vec::new(),
            executions,
            Some(blocker),
        );
    }
    match produced_library_artifact_path(&executable_producer, options)? {
        Ok(path) => {
            produced_target_artifacts.insert(producer.package_id.clone(), path);
        }
        Err(blocker) => {
            executions.push(receipt);
            return patch_source_topology_receipt(
                "blocked",
                Some(consumer_package.package_id.clone()),
                Vec::new(),
                executions,
                Some(blocker),
            );
        }
    }
    executions.push(receipt);
    let executable_consumer = bind_all_dependency_artifacts(consumer, &produced_target_artifacts)?;
    let receipt = execute_rust_unit(&executable_consumer, options)?;
    if receipt.execution_status != "success" {
        let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
            class: "patch-source-consumer-failed".to_string(),
            message: format!("patch source consumer {} did not execute successfully", consumer.unit_id),
        });
        executions.push(receipt);
        return patch_source_topology_receipt(
            "blocked",
            Some(consumer_package.package_id.clone()),
            vec![producer.package_id.clone()],
            executions,
            Some(blocker),
        );
    }
    executions.push(receipt);
    patch_source_topology_receipt(
        "success",
        Some(consumer_package.package_id.clone()),
        vec![producer.package_id.clone()],
        executions,
        None,
    )
}

fn validate_native_registry_topology_inputs(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    graph: &UnitDerivationGraphSummary,
) -> Option<RustUnitExecutionBlocker> {
    let registry_packages = graph
        .derivations
        .iter()
        .filter(|unit| unit.package_id.starts_with("registry+"))
        .map(|unit| unit.package_id.clone())
        .collect::<BTreeSet<_>>();
    if registry_packages.is_empty() {
        return None;
    }
    if !native_registry_sources.ready {
        let blocker_classes = native_registry_sources
            .blockers
            .iter()
            .map(|blocker| blocker.class.as_str())
            .collect::<Vec<_>>()
            .join(",");
        return Some(RustUnitExecutionBlocker {
            class: "native-registry-source-planning-blocked".to_string(),
            message: format!(
                "registry-backed topology execution requires ready native registry source facts; blockers: {blocker_classes}"
            ),
        });
    }
    let source_packages = native_registry_sources
        .sources
        .iter()
        .map(|source| source.package_id.clone())
        .collect::<BTreeSet<_>>();
    for package_id in registry_packages {
        if !source_packages.contains(&package_id) {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-registry-source-fact".to_string(),
                message: format!(
                    "registry-backed unit package {package_id} is not backed by native_registry_source_planning"
                ),
            });
        }
    }
    None
}

fn validate_native_host_artifact_topology_inputs(
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    execution_context: &str,
) -> Option<RustUnitExecutionBlocker> {
    if !native_host_graph.ready {
        return Some(RustUnitExecutionBlocker {
            class: "native-host-unit-graph-blocked".to_string(),
            message: format!(
                "native_host_unit_graph_planning is not ready; resolve native host graph blockers before {execution_context} execution"
            ),
        });
    }
    if !graph.ready {
        return None;
    }

    let graph_host_units = graph
        .derivations
        .iter()
        .filter(|unit| is_supported_host_unit(unit))
        .map(|unit| (unit.unit_id.clone(), unit.package_id.clone(), unit.target_name.clone(), unit.target_kind.clone()))
        .collect::<BTreeSet<_>>();
    let native_host_units = native_host_graph
        .host_units
        .iter()
        .map(|unit| (unit.unit_id.clone(), unit.package_id.clone(), unit.target_name.clone(), unit.target_kind.clone()))
        .collect::<BTreeSet<_>>();

    for native_unit in &native_host_units {
        if !graph_host_units.contains(native_unit) {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-host-derivation".to_string(),
                message: format!(
                    "native host unit {} for package {} is absent from unit_derivation_graph",
                    native_unit.0, native_unit.1
                ),
            });
        }
    }
    for graph_unit in &graph_host_units {
        if !native_host_units.contains(graph_unit) {
            return Some(RustUnitExecutionBlocker {
                class: "non-native-host-derivation".to_string(),
                message: format!(
                    "host derivation {} for package {} is not backed by native_host_unit_graph_planning",
                    graph_unit.0, graph_unit.1
                ),
            });
        }
    }

    let graph_consumers = graph
        .derivations
        .iter()
        .filter(|unit| is_supported_target_unit(unit) && !unit.consumed_host_artifacts.is_empty())
        .map(|unit| {
            let artifacts = unit.consumed_host_artifacts.iter().cloned().collect::<BTreeSet<_>>();
            (unit.unit_id.clone(), (unit.unit_id.clone(), unit.package_id.clone(), artifacts))
        })
        .collect::<BTreeMap<_, _>>();
    let native_consumers = native_host_graph
        .target_consumers
        .iter()
        .filter(|unit| !unit.consumed_host_artifacts.is_empty())
        .map(|unit| {
            let artifacts = unit.consumed_host_artifacts.iter().cloned().collect::<BTreeSet<_>>();
            (unit.unit_id.clone(), (unit.unit_id.clone(), unit.package_id.clone(), artifacts))
        })
        .collect::<BTreeMap<_, _>>();

    for (key, (unit_id, package_id, native_artifacts)) in &native_consumers {
        let Some((_, graph_artifacts)) = graph_consumers.get_key_value(key) else {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-host-consumer-derivation".to_string(),
                message: format!(
                    "native host-artifact consumer {unit_id} for package {package_id} is absent from unit_derivation_graph"
                ),
            });
        };
        for artifact in native_artifacts {
            if !graph_artifacts.2.contains(artifact) {
                return Some(RustUnitExecutionBlocker {
                    class: "missing-native-host-artifact-binding".to_string(),
                    message: format!(
                        "native host artifact from package {} is absent from derivation consumer {unit_id}",
                        artifact.package_id
                    ),
                });
            }
        }
    }
    for (key, (unit_id, package_id, _)) in &graph_consumers {
        if !native_consumers.contains_key(key) {
            return Some(RustUnitExecutionBlocker {
                class: "non-native-host-artifact-consumer".to_string(),
                message: format!(
                    "host-artifact consumer {unit_id} for package {package_id} is not backed by native_host_unit_graph_planning"
                ),
            });
        }
    }

    None
}

pub(crate) fn execute_rust_host_artifact_topology(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    if let Some(blocker) = validate_native_registry_topology_inputs(native_registry_sources, graph) {
        return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if let Some(blocker) = validate_native_host_artifact_topology_inputs(native_host_graph, graph, "host-artifact") {
        return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if !graph.ready {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before host-artifact execution"
                    .to_string(),
            }),
        );
    }

    let host_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_host_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    if host_indices.is_empty() {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-host-artifact-unit".to_string(),
                message: "unit_derivation_graph does not contain supported proc-macro/custom-build host units"
                    .to_string(),
            }),
        );
    }

    let mut host_producers = BTreeMap::<String, usize>::new();
    for index in &host_indices {
        host_producers.entry(graph.derivations[*index].package_id.clone()).or_insert(*index);
    }

    let mut target_indices = Vec::new();
    for (index, unit) in graph.derivations.iter().enumerate() {
        if is_supported_target_unit(unit) && !unit.consumed_host_artifacts.is_empty() {
            for host_artifact in &unit.consumed_host_artifacts {
                if !host_producers.contains_key(&host_artifact.package_id) {
                    return host_artifact_topology_receipt(
                        "blocked",
                        Vec::new(),
                        Vec::new(),
                        Some(RustUnitExecutionBlocker {
                            class: "missing-host-artifact-producer".to_string(),
                            message: format!(
                                "no supported host producer unit for host artifact package {}",
                                host_artifact.package_id
                            ),
                        }),
                    );
                }
            }
            target_indices.push(index);
        }
    }
    if target_indices.is_empty() {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-host-artifact-consumer".to_string(),
                message: "unit_derivation_graph does not contain target units that consume host artifacts".to_string(),
            }),
        );
    }

    let selected_set = target_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut lib_producers = BTreeMap::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }
    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            if host_producers.contains_key(&dependency.package_id) {
                continue;
            }
            let Some(producer_index) = lib_producers.get(&dependency.package_id).copied() else {
                return host_artifact_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "no supported target producer lib unit for dependency package {}",
                            dependency.package_id
                        ),
                    }),
                );
            };
            if selected_set.contains(&producer_index) && producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let mut ordered_target_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &target_indices {
        if let Err(blocker) = visit_target_topology_unit(
            *index,
            &edges,
            &mut temporary,
            &mut permanent,
            &mut ordered_target_indices,
            graph,
        ) {
            return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut build_script_metadata_runs = Vec::new();
    let mut produced_host_artifacts = ProducedHostArtifactIndex::default();
    let mut produced_build_script_metadata = ProducedBuildScriptMetadataIndex::default();
    for index in host_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = match bind_all_host_artifacts_with_index(unit, &produced_host_artifacts) {
            Ok(bound) => bound,
            Err(blocker) => {
                return host_artifact_topology_receipt(
                    "blocked",
                    executions,
                    build_script_metadata_runs,
                    Some(blocker),
                );
            }
        };
        executable_unit =
            match bind_all_build_script_metadata_with_index(&executable_unit, &produced_build_script_metadata) {
                Ok(bound) => bound,
                Err(blocker) => {
                    return host_artifact_topology_receipt(
                        "blocked",
                        executions,
                        build_script_metadata_runs,
                        Some(blocker),
                    );
                }
            };
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "host-unit-failed".to_string(),
                message: format!("host unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return host_artifact_topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        match produced_host_artifact_path(unit, options)? {
            Ok(path) => {
                if unit.target_kind == "custom-build" {
                    match run_build_script_metadata(unit, options, &path)? {
                        Ok(metadata_run) => {
                            produced_build_script_metadata
                                .record_unit(unit, build_script_metadata_from_run(&metadata_run));
                            build_script_metadata_runs.push(metadata_run);
                        }
                        Err(blocker) => {
                            executions.push(receipt);
                            return host_artifact_topology_receipt(
                                "blocked",
                                executions,
                                build_script_metadata_runs,
                                Some(blocker),
                            );
                        }
                    }
                }
                produced_host_artifacts.record_unit(unit, path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return host_artifact_topology_receipt(
                    "blocked",
                    executions,
                    build_script_metadata_runs,
                    Some(blocker),
                );
            }
        }
        executions.push(receipt);
    }

    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in ordered_target_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = match bind_all_host_artifacts_with_index(unit, &produced_host_artifacts) {
            Ok(bound) => bound,
            Err(blocker) => {
                return host_artifact_topology_receipt(
                    "blocked",
                    executions,
                    build_script_metadata_runs,
                    Some(blocker),
                );
            }
        };
        executable_unit =
            match bind_all_build_script_metadata_with_index(&executable_unit, &produced_build_script_metadata) {
                Ok(bound) => bound,
                Err(blocker) => {
                    return host_artifact_topology_receipt(
                        "blocked",
                        executions,
                        build_script_metadata_runs,
                        Some(blocker),
                    );
                }
            };
        if !executable_unit.dependency_artifacts.is_empty() {
            executable_unit = bind_all_dependency_artifacts(&executable_unit, &produced_target_artifacts)?;
        }
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "target-unit-failed".to_string(),
                message: format!("target unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return host_artifact_topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_target_artifacts.insert(unit.package_id.clone(), path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return host_artifact_topology_receipt(
                        "blocked",
                        executions,
                        build_script_metadata_runs,
                        Some(blocker),
                    );
                }
            }
        }
        executions.push(receipt);
    }

    host_artifact_topology_receipt("success", executions, build_script_metadata_runs, None)
}

fn visit_target_topology_unit(
    index: usize,
    edges: &BTreeMap<usize, Vec<usize>>,
    temporary: &mut BTreeSet<usize>,
    permanent: &mut BTreeSet<usize>,
    ordered_indices: &mut Vec<usize>,
    graph: &UnitDerivationGraphSummary,
) -> Result<(), RustUnitExecutionBlocker> {
    if permanent.contains(&index) {
        return Ok(());
    }
    if !temporary.insert(index) {
        return Err(RustUnitExecutionBlocker {
            class: "dependency-cycle".to_string(),
            message: format!(
                "target topology contains a dependency cycle at unit {}",
                graph.derivations[index].unit_id
            ),
        });
    }
    for dependency_index in edges.get(&index).into_iter().flatten() {
        visit_target_topology_unit(*dependency_index, edges, temporary, permanent, ordered_indices, graph)?;
    }
    temporary.remove(&index);
    permanent.insert(index);
    ordered_indices.push(index);
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct ProducedArtifactIndex {
    by_unit_id: BTreeMap<String, PathBuf>,
    by_package: BTreeMap<String, Vec<ProducedPackageArtifact>>,
}

#[derive(Debug, Clone, Default)]
struct ProducedHostArtifactIndex {
    by_unit_id: BTreeMap<String, PathBuf>,
    by_package: BTreeMap<String, Vec<ProducedHostPackageArtifact>>,
}

#[derive(Debug, Clone, Default)]
struct ProducedBuildScriptMetadataIndex {
    by_unit_id: BTreeMap<String, BuildScriptMetadataSummary>,
    by_package: BTreeMap<String, Vec<ProducedBuildScriptMetadata>>,
}

#[derive(Debug, Clone, Default)]
struct BuildScriptMetadataProducerIndex {
    by_unit_id: BTreeMap<String, usize>,
    by_package: BTreeMap<String, Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducedPackageArtifact {
    unit_id: String,
    target_kind: String,
    crate_name: String,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducedHostPackageArtifact {
    unit_id: String,
    target_name: String,
    target_kind: String,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducedBuildScriptMetadata {
    unit_id: String,
    target_name: String,
    metadata: BuildScriptMetadataSummary,
}

impl ProducedArtifactIndex {
    fn record_unit(&mut self, unit: &RustUnitDerivationSummary, path: PathBuf) {
        debug_assert!(!unit.unit_id.is_empty());
        debug_assert!(!unit.package_id.is_empty());
        debug_assert!(!unit.target_name.is_empty());
        let artifact = ProducedPackageArtifact {
            unit_id: unit.unit_id.clone(),
            target_kind: unit.target_kind.clone(),
            crate_name: rustc_crate_name_from_args(&unit.derivation.args)
                .unwrap_or_else(|| rust_crate_name(&unit.target_name)),
            path: path.clone(),
        };
        self.by_unit_id.insert(unit.unit_id.clone(), path.clone());
        let package_artifacts = self.by_package.entry(unit.package_id.clone()).or_default();
        package_artifacts.retain(|candidate| candidate.unit_id != artifact.unit_id);
        package_artifacts.push(artifact);
        package_artifacts.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    }

    fn extend_from(&mut self, other: &ProducedArtifactIndex) {
        for (unit_id, path) in &other.by_unit_id {
            self.by_unit_id.insert(unit_id.clone(), path.clone());
        }
        for (package_id, artifacts) in &other.by_package {
            let package_artifacts = self.by_package.entry(package_id.clone()).or_default();
            for artifact in artifacts {
                package_artifacts.retain(|candidate| candidate.unit_id != artifact.unit_id);
                package_artifacts.push(artifact.clone());
            }
            package_artifacts.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
        }
    }
}

impl ProducedHostArtifactIndex {
    fn record_unit(&mut self, unit: &RustUnitDerivationSummary, path: PathBuf) {
        debug_assert!(!unit.unit_id.is_empty());
        debug_assert!(!unit.package_id.is_empty());
        debug_assert!(is_supported_host_unit(unit));
        let artifact = ProducedHostPackageArtifact {
            unit_id: unit.unit_id.clone(),
            target_name: unit.target_name.clone(),
            target_kind: unit.target_kind.clone(),
            path: path.clone(),
        };
        self.by_unit_id.insert(unit.unit_id.clone(), path);
        let package_artifacts = self.by_package.entry(unit.package_id.clone()).or_default();
        package_artifacts.retain(|candidate| candidate.unit_id != artifact.unit_id);
        package_artifacts.push(artifact);
        package_artifacts.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    }
}

impl ProducedBuildScriptMetadataIndex {
    fn record_unit(&mut self, unit: &RustUnitDerivationSummary, metadata: BuildScriptMetadataSummary) {
        debug_assert!(!unit.unit_id.is_empty());
        debug_assert!(!unit.package_id.is_empty());
        debug_assert_eq!(unit.target_kind, "custom-build");
        let produced = ProducedBuildScriptMetadata {
            unit_id: unit.unit_id.clone(),
            target_name: build_script_alias_name(unit),
            metadata: metadata.clone(),
        };
        self.by_unit_id.insert(unit.unit_id.clone(), metadata);
        let package_metadata = self.by_package.entry(unit.package_id.clone()).or_default();
        package_metadata.retain(|candidate| candidate.unit_id != produced.unit_id);
        package_metadata.push(produced);
        package_metadata.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    }
}

fn bind_all_dependency_artifacts(
    unit: &RustUnitDerivationSummary,
    produced_artifacts: &BTreeMap<String, PathBuf>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for dependency in &unit.dependency_artifacts {
        if Path::new(&dependency.artifact).is_file() {
            continue;
        }
        let produced_artifact = produced_artifacts.get(&dependency.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before dependency package {} was produced",
                unit.unit_id, dependency.package_id
            ))
        })?;
        bound = bind_dependency_artifact(&bound, dependency, produced_artifact)?;
    }
    append_dependency_search_paths(&mut bound, produced_artifacts);
    Ok(bound)
}

fn bind_all_dependency_artifacts_with_index(
    unit: &RustUnitDerivationSummary,
    produced_artifacts: &ProducedArtifactIndex,
) -> Result<RustUnitDerivationSummary, RustUnitExecutionBlocker> {
    let mut bound = unit.clone();
    for dependency in &unit.dependency_artifacts {
        if Path::new(&dependency.artifact).is_file() {
            continue;
        }
        let produced_artifact = select_produced_dependency_artifact(unit, dependency, produced_artifacts)?;
        bound = bind_dependency_artifact(&bound, dependency, produced_artifact).map_err(|err| {
            RustUnitExecutionBlocker {
                class: "dependency-binding-failed".to_string(),
                message: err.to_string(),
            }
        })?;
    }
    Ok(bound)
}

fn select_produced_dependency_artifact<'a>(
    unit: &RustUnitDerivationSummary,
    dependency: &RustDependencyArtifact,
    produced_artifacts: &'a ProducedArtifactIndex,
) -> Result<&'a Path, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &dependency.producer_unit_id {
        return produced_artifacts.by_unit_id.get(producer_unit_id).map(PathBuf::as_path).ok_or_else(|| {
            RustUnitExecutionBlocker {
                class: "missing-dependency-producer".to_string(),
                message: format!(
                    "unit {} reached execution before selected dependency producer unit {} was produced",
                    unit.unit_id, producer_unit_id
                ),
            }
        });
    }
    let candidates = produced_artifacts
        .by_package
        .get(&dependency.package_id)
        .map(|artifacts| matching_produced_dependency_artifacts(artifacts, dependency))
        .unwrap_or_default();
    match candidates.as_slice() {
        [] => Err(RustUnitExecutionBlocker {
            class: "missing-dependency-producer".to_string(),
            message: format!(
                "unit {} reached execution before dependency package {} was produced",
                unit.unit_id, dependency.package_id
            ),
        }),
        [candidate] => Ok(candidate.path.as_path()),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-dependency-producer".to_string(),
            message: format!(
                "unit {} dependency {} has {} package-only producer candidates for package {}",
                unit.unit_id,
                dependency.name,
                candidates.len(),
                dependency.package_id
            ),
        }),
    }
}

fn matching_produced_dependency_artifacts<'a>(
    artifacts: &'a [ProducedPackageArtifact],
    dependency: &RustDependencyArtifact,
) -> Vec<&'a ProducedPackageArtifact> {
    let exact = artifacts.iter().filter(|artifact| artifact.crate_name == dependency.name).collect::<Vec<_>>();
    if !exact.is_empty() {
        return exact;
    }
    match artifacts {
        [single] => vec![single],
        _ => Vec::new(),
    }
}

fn append_selected_dependency_search_paths(
    unit: &mut RustUnitDerivationSummary,
    graph: &UnitDerivationGraphSummary,
    produced_artifacts: &ProducedArtifactIndex,
) -> Result<(), RustUnitExecutionBlocker> {
    let search_artifacts = selected_dependency_search_artifacts(unit, graph, produced_artifacts)?;
    append_dependency_search_paths_from_paths(unit, search_artifacts.iter());
    Ok(())
}

fn selected_dependency_search_artifacts(
    unit: &RustUnitDerivationSummary,
    graph: &UnitDerivationGraphSummary,
    produced_artifacts: &ProducedArtifactIndex,
) -> Result<Vec<PathBuf>, RustUnitExecutionBlocker> {
    let mut units_by_id = BTreeMap::new();
    for candidate in &graph.derivations {
        units_by_id.insert(candidate.unit_id.clone(), candidate);
    }
    let mut stack = Vec::new();
    for dependency in &unit.dependency_artifacts {
        stack.extend(dependency_producer_unit_ids(unit, dependency, produced_artifacts)?);
    }
    let mut visited = BTreeSet::new();
    let mut paths = BTreeSet::new();
    while let Some(unit_id) = stack.pop() {
        if !visited.insert(unit_id.clone()) {
            continue;
        }
        if visited.len() > graph.derivations.len() {
            return Err(RustUnitExecutionBlocker {
                class: "dependency-cycle".to_string(),
                message: format!("dependency search closure for unit {} exceeded graph size", unit.unit_id),
            });
        }
        if let Some(path) = produced_artifacts.by_unit_id.get(&unit_id) {
            paths.insert(path.clone());
        }
        let Some(producer) = units_by_id.get(&unit_id) else {
            continue;
        };
        for host_artifact in &producer.consumed_host_artifacts {
            if host_artifact.target_kind == "proc-macro" {
                if let Some(path) = select_produced_host_search_artifact(unit, host_artifact, produced_artifacts)? {
                    paths.insert(path);
                }
            }
        }
        for dependency in &producer.dependency_artifacts {
            stack.extend(dependency_producer_unit_ids(unit, dependency, produced_artifacts)?);
        }
    }
    Ok(paths.into_iter().collect())
}

fn select_produced_host_search_artifact(
    unit: &RustUnitDerivationSummary,
    host_artifact: &RustHostArtifact,
    produced_artifacts: &ProducedArtifactIndex,
) -> Result<Option<PathBuf>, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &host_artifact.producer_unit_id {
        return Ok(produced_artifacts.by_unit_id.get(producer_unit_id).cloned());
    }
    let candidates = produced_artifacts
        .by_package
        .get(&host_artifact.package_id)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter(|artifact| artifact.target_kind == host_artifact.target_kind)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match candidates.as_slice() {
        [] => Ok(None),
        [candidate] => Ok(Some(candidate.path.clone())),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-host-search-producer".to_string(),
            message: format!(
                "unit {} host artifact {} has {} package-only search candidates for package {}",
                unit.unit_id,
                host_artifact.target_kind,
                candidates.len(),
                host_artifact.package_id
            ),
        }),
    }
}

fn dependency_producer_unit_ids(
    unit: &RustUnitDerivationSummary,
    dependency: &RustDependencyArtifact,
    produced_artifacts: &ProducedArtifactIndex,
) -> Result<Vec<String>, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &dependency.producer_unit_id {
        return Ok(vec![producer_unit_id.clone()]);
    }
    let candidates = produced_artifacts
        .by_package
        .get(&dependency.package_id)
        .map(|artifacts| matching_produced_dependency_artifacts(artifacts, dependency))
        .unwrap_or_default();
    match candidates.as_slice() {
        [] => Ok(Vec::new()),
        [candidate] => Ok(vec![candidate.unit_id.clone()]),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-dependency-producer".to_string(),
            message: format!(
                "unit {} dependency {} has {} package-only search candidates for package {}",
                unit.unit_id,
                dependency.name,
                candidates.len(),
                dependency.package_id
            ),
        }),
    }
}

#[cfg(test)]
#[cfg(test)]
fn bind_target_unit_artifacts(
    unit: &RustUnitDerivationSummary,
    produced_host_artifacts: &BTreeMap<String, PathBuf>,
    produced_metadata: &BTreeMap<String, BuildScriptMetadataSummary>,
    produced_target_artifacts: &BTreeMap<String, PathBuf>,
    produced_proc_macro_artifacts: &BTreeMap<String, PathBuf>,
) -> Result<RustUnitDerivationSummary, RunError> {
    debug_assert!(is_supported_target_unit(unit));
    let mut executable_unit = bind_all_host_artifacts(unit, produced_host_artifacts)?;
    executable_unit = bind_all_build_script_metadata(&executable_unit, produced_metadata)?;
    if !executable_unit.dependency_artifacts.is_empty() {
        executable_unit = bind_all_dependency_artifacts(&executable_unit, produced_target_artifacts)?;
    }
    append_dependency_search_paths(&mut executable_unit, produced_proc_macro_artifacts);
    Ok(executable_unit)
}

#[cfg(test)]
fn bind_all_host_artifacts(
    unit: &RustUnitDerivationSummary,
    produced_host_artifacts: &BTreeMap<String, PathBuf>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for host_artifact in &unit.consumed_host_artifacts {
        let produced_artifact = produced_host_artifacts.get(&host_artifact.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before host artifact package {} was produced",
                unit.unit_id, host_artifact.package_id
            ))
        })?;
        let produced_artifact_string = normalize_path_string(&rustc_artifact_path(produced_artifact));
        bind_host_artifact_material(&mut bound, host_artifact, &produced_artifact_string);
        let host_dependency_names = host_artifact_dependency_names(host_artifact);
        let host_crate_name = rust_crate_name(&host_artifact.target_name);
        let matching_dependencies = bound
            .dependency_artifacts
            .iter()
            .filter(|dependency| {
                dependency.package_id == host_artifact.package_id && host_dependency_names.contains(&dependency.name)
            })
            .cloned()
            .collect::<Vec<_>>();
        for dependency in matching_dependencies {
            bound = bind_dependency_artifact(&bound, &dependency, produced_artifact)?;
        }
        if host_artifact.target_kind == "proc-macro" {
            ensure_host_artifact_extern_arg(&mut bound.derivation.args, &host_crate_name, &produced_artifact_string);
        }
    }
    refresh_rustc_args_digest(&mut bound);
    Ok(bound)
}

fn bind_all_host_artifacts_with_index(
    unit: &RustUnitDerivationSummary,
    produced_host_artifacts: &ProducedHostArtifactIndex,
) -> Result<RustUnitDerivationSummary, RustUnitExecutionBlocker> {
    let mut bound = unit.clone();
    for host_artifact in &unit.consumed_host_artifacts {
        let produced_artifact = select_produced_host_artifact(unit, host_artifact, produced_host_artifacts)?;
        let produced_artifact_string = normalize_path_string(&rustc_artifact_path(produced_artifact));
        bind_host_artifact_material(&mut bound, host_artifact, &produced_artifact_string);
        let host_dependency_names = host_artifact_dependency_names(host_artifact);
        let host_crate_name = rust_crate_name(&host_artifact.target_name);
        let matching_dependencies = bound
            .dependency_artifacts
            .iter()
            .filter(|dependency| {
                dependency.package_id == host_artifact.package_id && host_dependency_names.contains(&dependency.name)
            })
            .cloned()
            .collect::<Vec<_>>();
        for dependency in matching_dependencies {
            bound = bind_dependency_artifact(&bound, &dependency, produced_artifact).map_err(|err| {
                RustUnitExecutionBlocker {
                    class: "host-dependency-binding-failed".to_string(),
                    message: err.to_string(),
                }
            })?;
        }
        if host_artifact.target_kind == "proc-macro" {
            ensure_host_artifact_extern_arg(&mut bound.derivation.args, &host_crate_name, &produced_artifact_string);
        }
    }
    refresh_rustc_args_digest(&mut bound);
    Ok(bound)
}

fn select_produced_host_artifact<'a>(
    unit: &RustUnitDerivationSummary,
    host_artifact: &RustHostArtifact,
    produced_host_artifacts: &'a ProducedHostArtifactIndex,
) -> Result<&'a Path, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &host_artifact.producer_unit_id {
        return produced_host_artifacts.by_unit_id.get(producer_unit_id).map(PathBuf::as_path).ok_or_else(|| {
            RustUnitExecutionBlocker {
                class: "missing-host-artifact-producer".to_string(),
                message: format!(
                    "unit {} reached execution before selected host producer unit {} was produced",
                    unit.unit_id, producer_unit_id
                ),
            }
        });
    }
    let candidates = produced_host_artifacts
        .by_package
        .get(&host_artifact.package_id)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter(|artifact| artifact.target_kind == host_artifact.target_kind)
                .filter(|artifact| {
                    rust_crate_name(&artifact.target_name) == rust_crate_name(&host_artifact.target_name)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    match candidates.as_slice() {
        [] => Err(RustUnitExecutionBlocker {
            class: "missing-host-artifact-producer".to_string(),
            message: format!(
                "unit {} reached execution before host artifact package {} was produced",
                unit.unit_id, host_artifact.package_id
            ),
        }),
        [candidate] => Ok(candidate.path.as_path()),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-host-artifact-producer".to_string(),
            message: format!(
                "unit {} host artifact {} has {} package-only candidates for package {}",
                unit.unit_id,
                host_artifact.target_kind,
                candidates.len(),
                host_artifact.package_id
            ),
        }),
    }
}

fn host_artifact_dependency_names(host_artifact: &RustHostArtifact) -> BTreeSet<String> {
    debug_assert!(!host_artifact.target_name.is_empty());
    debug_assert!(!host_artifact.target_kind.is_empty());
    let mut names = BTreeSet::from([rust_crate_name(&host_artifact.target_name)]);
    if host_artifact.target_kind == "custom-build" {
        names.insert(BUILD_SCRIPT_BUILD_CRATE_NAME.to_string());
        names.insert(BUILD_SCRIPT_MAIN_CRATE_NAME.to_string());
    }
    names
}

fn bind_host_artifact_material(
    unit: &mut RustUnitDerivationSummary,
    host_artifact: &RustHostArtifact,
    produced_artifact: &str,
) {
    debug_assert!(!host_artifact.package_id.is_empty());
    debug_assert!(!host_artifact.artifact.is_empty());
    debug_assert!(!produced_artifact.is_empty());
    for artifact in &mut unit.consumed_host_artifacts {
        if artifact.package_id == host_artifact.package_id
            && artifact.target_name == host_artifact.target_name
            && artifact.target_kind == host_artifact.target_kind
            && artifact.artifact == host_artifact.artifact
        {
            artifact.artifact = produced_artifact.to_string();
        }
    }
    for input in &mut unit.derivation.inputs {
        if input == &host_artifact.artifact {
            *input = produced_artifact.to_string();
        }
    }
}

fn ensure_host_artifact_extern_arg(args: &mut Vec<String>, crate_name: &str, produced_artifact: &str) {
    debug_assert!(!crate_name.is_empty());
    debug_assert!(!produced_artifact.is_empty());
    let extern_arg = format!("{crate_name}={produced_artifact}");
    if has_rustc_extern_arg(args, crate_name) {
        return;
    }
    args.push(RUSTC_EXTERN_FLAG.to_string());
    args.push(extern_arg);
}

fn has_rustc_extern_arg(args: &[String], crate_name: &str) -> bool {
    debug_assert!(!crate_name.is_empty());
    let crate_prefix = format!("{crate_name}=");
    args.windows(RUSTC_EXTERN_ARG_PAIR_WIDTH)
        .any(|window| window[0] == RUSTC_EXTERN_FLAG && window[1].starts_with(&crate_prefix))
}

fn refresh_rustc_args_digest(unit: &mut RustUnitDerivationSummary) {
    unit.rustc_args_digest_blake3 = blake3::hash(unit.derivation.args.join("\0").as_bytes()).to_hex().to_string();
}

fn bind_all_build_script_metadata_with_index(
    unit: &RustUnitDerivationSummary,
    produced_metadata: &ProducedBuildScriptMetadataIndex,
) -> Result<RustUnitDerivationSummary, RustUnitExecutionBlocker> {
    let mut bound = unit.clone();
    for metadata_dependency in &unit.metadata_dependencies {
        let metadata = select_produced_build_script_metadata(unit, metadata_dependency, produced_metadata)?;
        append_dep_metadata_env(&mut bound.derivation.env, metadata_dependency, &metadata);
    }
    for host_artifact in &unit.consumed_host_artifacts {
        if host_artifact.target_kind != "custom-build" {
            continue;
        }
        let metadata = select_produced_host_build_script_metadata(unit, host_artifact, produced_metadata)?;
        apply_build_script_metadata_to_unit(&mut bound, &metadata);
    }
    bound.rustc_args_digest_blake3 = blake3::hash(bound.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    Ok(bound)
}

fn select_produced_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    metadata_dependency: &BuildScriptMetadataDependency,
    produced_metadata: &ProducedBuildScriptMetadataIndex,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &metadata_dependency.producer_unit_id {
        return produced_metadata.by_unit_id.get(producer_unit_id).cloned().ok_or_else(|| RustUnitExecutionBlocker {
            class: "missing-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} reached execution before selected build-script metadata producer {} was produced",
                unit.unit_id, producer_unit_id
            ),
        });
    }
    select_package_build_script_metadata(unit, &metadata_dependency.package_id, produced_metadata)
}

fn select_produced_host_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    host_artifact: &RustHostArtifact,
    produced_metadata: &ProducedBuildScriptMetadataIndex,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    if let Some(producer_unit_id) = &host_artifact.producer_unit_id {
        return produced_metadata.by_unit_id.get(producer_unit_id).cloned().ok_or_else(|| RustUnitExecutionBlocker {
            class: "missing-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} reached execution before selected build-script metadata producer {} was produced",
                unit.unit_id, producer_unit_id
            ),
        });
    }
    select_package_build_script_metadata(unit, &host_artifact.package_id, produced_metadata)
}

fn select_package_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    package_id: &str,
    produced_metadata: &ProducedBuildScriptMetadataIndex,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    let candidates = produced_metadata.by_package.get(package_id).cloned().unwrap_or_default();
    let candidates = collapse_build_script_alias_metadata_candidates(candidates);
    match candidates.as_slice() {
        [] => Err(RustUnitExecutionBlocker {
            class: "missing-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} reached execution before build-script metadata package {} was produced",
                unit.unit_id, package_id
            ),
        }),
        [candidate] => Ok(candidate.metadata.clone()),
        _ => Err(RustUnitExecutionBlocker {
            class: "ambiguous-build-script-metadata-producer".to_string(),
            message: format!(
                "unit {} build-script metadata package {} has {} package-only candidates",
                unit.unit_id,
                package_id,
                candidates.len()
            ),
        }),
    }
}

fn collapse_build_script_alias_metadata_candidates(
    candidates: Vec<ProducedBuildScriptMetadata>,
) -> Vec<ProducedBuildScriptMetadata> {
    let mut alias_candidates_by_name = BTreeMap::<String, Vec<ProducedBuildScriptMetadata>>::new();
    let mut collapsed = Vec::new();
    for candidate in candidates {
        if is_build_script_target_alias(&candidate.target_name) {
            alias_candidates_by_name.entry(candidate.target_name.clone()).or_default().push(candidate);
        } else {
            collapsed.push(candidate);
        }
    }
    if should_collapse_build_script_alias_groups(&alias_candidates_by_name) {
        if let Some(preferred) = preferred_build_script_alias_metadata_candidate(&alias_candidates_by_name) {
            collapsed.push(preferred);
        }
    } else {
        collapsed.extend(alias_candidates_by_name.into_values().flatten());
    }
    collapsed.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    collapsed
}

fn preferred_build_script_alias_metadata_candidate(
    alias_candidates_by_name: &BTreeMap<String, Vec<ProducedBuildScriptMetadata>>,
) -> Option<ProducedBuildScriptMetadata> {
    alias_candidates_by_name
        .values()
        .filter_map(|candidates| candidates.first().cloned())
        .min_by_key(|candidate| (build_script_alias_preference(&candidate.target_name), candidate.unit_id.clone()))
}

fn apply_build_script_metadata_to_unit(unit: &mut RustUnitDerivationSummary, metadata: &BuildScriptMetadataSummary) {
    unit.derivation.env.insert("OUT_DIR".to_string(), metadata.out_dir.clone());
    for (key, value) in &metadata.rustc_env {
        unit.derivation.env.insert(key.clone(), value.clone());
    }
    for cfg in &metadata.rustc_cfg {
        unit.derivation.args.push("--cfg".to_string());
        unit.derivation.args.push(cfg.clone());
    }
    for search in &metadata.rustc_link_search {
        unit.derivation.args.push("-L".to_string());
        unit.derivation.args.push(search.clone());
    }
    for lib in &metadata.rustc_link_lib {
        unit.derivation.args.push("-l".to_string());
        unit.derivation.args.push(lib.clone());
    }
}

#[cfg(test)]
fn bind_all_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    produced_metadata: &BTreeMap<String, BuildScriptMetadataSummary>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for metadata_dependency in &unit.metadata_dependencies {
        let metadata = produced_metadata.get(&metadata_dependency.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before linked build-script metadata package {} was produced",
                unit.unit_id, metadata_dependency.package_id
            ))
        })?;
        append_dep_metadata_env(&mut bound.derivation.env, metadata_dependency, metadata);
    }
    for host_artifact in &unit.consumed_host_artifacts {
        if host_artifact.target_kind != "custom-build" {
            continue;
        }
        let metadata = produced_metadata.get(&host_artifact.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before build-script metadata package {} was produced",
                unit.unit_id, host_artifact.package_id
            ))
        })?;
        bound.derivation.env.insert("OUT_DIR".to_string(), metadata.out_dir.clone());
        for (key, value) in &metadata.rustc_env {
            bound.derivation.env.insert(key.clone(), value.clone());
        }
        for cfg in &metadata.rustc_cfg {
            bound.derivation.args.push("--cfg".to_string());
            bound.derivation.args.push(cfg.clone());
        }
        for search in &metadata.rustc_link_search {
            bound.derivation.args.push("-L".to_string());
            bound.derivation.args.push(search.clone());
        }
        for lib in &metadata.rustc_link_lib {
            bound.derivation.args.push("-l".to_string());
            bound.derivation.args.push(lib.clone());
        }
    }
    bound.rustc_args_digest_blake3 = blake3::hash(bound.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    Ok(bound)
}

fn append_dep_metadata_env(
    env: &mut BTreeMap<String, String>,
    dependency: &BuildScriptMetadataDependency,
    metadata: &BuildScriptMetadataSummary,
) {
    let prefix = format!("{}{}", BUILD_SCRIPT_DEP_ENV_PREFIX, cargo_metadata_env_component(&dependency.links));
    for (key, value) in &metadata.metadata {
        let name = format!("{prefix}_{}", cargo_metadata_env_component(key));
        env.insert(name, value.clone());
    }
}

fn cargo_metadata_env_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn is_link_self_contained_codegen_option(option: &str) -> bool {
    if option == RUSTC_LINK_SELF_CONTAINED_OPTION {
        return true;
    }
    option.strip_prefix(RUSTC_LINK_SELF_CONTAINED_OPTION).is_some_and(|suffix| suffix.starts_with('='))
}

fn rust_topology_runtime_args(reviewable_args: &[String]) -> Vec<String> {
    let mut has_link_self_contained = false;
    let mut previous_is_codegen_flag = false;
    for arg in reviewable_args {
        if previous_is_codegen_flag && is_link_self_contained_codegen_option(arg) {
            has_link_self_contained = true;
        }
        if arg.strip_prefix("-C").is_some_and(is_link_self_contained_codegen_option) {
            has_link_self_contained = true;
        }
        previous_is_codegen_flag = arg == RUSTC_CODEGEN_OPTION_FLAG;
    }
    let mut runtime_args = reviewable_args.to_vec();
    if !has_link_self_contained {
        runtime_args.push(RUSTC_CODEGEN_OPTION_FLAG.to_string());
        runtime_args.push(RUSTC_EXTERNAL_LINKER_MODE_ARG.to_string());
    }
    runtime_args
}

fn rust_topology_child_env(
    explicit_env: &BTreeMap<String, String>,
    inherited_path: Option<OsString>,
    inherited_compile_env: &BTreeMap<String, OsString>,
) -> BTreeMap<String, OsString> {
    let mut env = BTreeMap::new();
    if let Some(path) = inherited_path {
        if !path.is_empty() {
            env.insert(RUST_TOPOLOGY_TOOL_PATH_ENV.to_string(), path);
        }
    }
    for (key, value) in inherited_compile_env {
        debug_assert!(RUST_TOPOLOGY_COMPILE_ENV_ALLOWLIST.contains(&key.as_str()));
        debug_assert!(!value.is_empty());
        env.insert(key.clone(), value.clone());
    }
    for (key, value) in explicit_env {
        debug_assert!(!key.is_empty());
        env.insert(key.clone(), OsString::from(value));
    }
    env
}

fn allowed_rust_topology_compile_env(candidates: &BTreeMap<String, OsString>) -> BTreeMap<String, OsString> {
    let mut allowed = BTreeMap::new();
    for key in RUST_TOPOLOGY_COMPILE_ENV_ALLOWLIST {
        let Some(value) = candidates.get(*key) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        allowed.insert((*key).to_string(), value.clone());
    }
    allowed
}

fn inherited_rust_topology_compile_env() -> BTreeMap<String, OsString> {
    let candidates = RUST_TOPOLOGY_COMPILE_ENV_ALLOWLIST
        .iter()
        .filter_map(|key| std::env::var_os(key).map(|value| ((*key).to_string(), value)))
        .collect::<BTreeMap<_, _>>();
    allowed_rust_topology_compile_env(&candidates)
}

fn apply_rust_topology_child_env(command: &mut Command, explicit_env: &BTreeMap<String, String>) {
    command.env_clear();
    for (key, value) in rust_topology_child_env(
        explicit_env,
        std::env::var_os(RUST_TOPOLOGY_TOOL_PATH_ENV),
        &inherited_rust_topology_compile_env(),
    ) {
        command.env(key, value);
    }
}

fn build_script_package_root(unit: &RustUnitDerivationSummary) -> Option<PathBuf> {
    unit.derivation
        .env
        .get(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|| rustc_source_path(&unit.derivation.args).and_then(|path| path.parent().map(Path::to_path_buf)))
}

fn absolute_path_from(path: &Path, base: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    base.join(path)
}

fn build_script_child_env(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
    out_dir: &Path,
    package_root: Option<&Path>,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    append_build_script_dependency_env(&mut env, &unit.derivation.env);
    append_build_script_package_env(&mut env, &unit.derivation.env);
    env.insert(BUILD_SCRIPT_OUT_DIR_ENV.to_string(), normalize_path_string(out_dir));
    env.insert(
        BUILD_SCRIPT_CARGO_PKG_NAME_ENV.to_string(),
        unit.derivation
            .env
            .get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV)
            .cloned()
            .unwrap_or_else(|| rust_crate_name(&unit.target_name)),
    );
    env.insert(BUILD_SCRIPT_RUSTC_ENV.to_string(), normalize_path_string(&options.rustc));
    env.insert(BUILD_SCRIPT_HOST_ENV.to_string(), host_target_triple());
    let target = unit.derivation.env.get(BUILD_SCRIPT_TARGET_ENV).cloned().unwrap_or_else(host_target_triple);
    append_build_script_target_cfg_env(&mut env, &target);
    append_build_script_profile_env(&mut env, &unit.profile);
    env.insert(BUILD_SCRIPT_TARGET_ENV.to_string(), target);
    env.insert(BUILD_SCRIPT_PROFILE_ENV.to_string(), unit.profile.clone());
    if let Some(root) = package_root {
        env.insert(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV.to_string(), normalize_path_string(root));
    }
    env
}

fn append_build_script_dependency_env(env: &mut BTreeMap<String, String>, source: &BTreeMap<String, String>) {
    for (key, value) in source {
        if key.starts_with(BUILD_SCRIPT_DEP_ENV_PREFIX) {
            env.insert(key.clone(), value.clone());
        }
    }
}

fn append_build_script_package_env(env: &mut BTreeMap<String, String>, source: &BTreeMap<String, String>) {
    for (key, value) in source {
        if key.starts_with("CARGO_PKG_") || key == BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV {
            env.insert(key.clone(), value.clone());
        }
    }
}

fn append_build_script_target_cfg_env(env: &mut BTreeMap<String, String>, target: &str) {
    for (key, value) in build_script_target_cfg_env(target) {
        debug_assert!(key.starts_with("CARGO_CFG_"));
        env.insert(key, value);
    }
}

fn append_build_script_profile_env(env: &mut BTreeMap<String, String>, profile: &str) {
    for (key, value) in build_script_profile_env(profile) {
        debug_assert!(!key.is_empty());
        env.insert(key, value);
    }
}

fn run_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
    executable: &Path,
) -> Result<Result<BuildScriptMetadataRunReceipt, RustUnitExecutionBlocker>, RunError> {
    if !executable.is_file() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-build-script-executable".to_string(),
            message: format!("custom-build unit {} did not produce an executable", unit.unit_id),
        }));
    }
    let invocation_dir = std::env::current_dir()
        .map_err(|err| RunError::Internal(format!("reading current directory for build-script execution: {err}")))?;
    let output_root = absolute_path_from(&options.output_root, &invocation_dir);
    let out_dir = output_root.join(safe_path_component(&unit.unit_id)).join("out-dir");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|err| {
            RunError::Internal(format!("removing prior build-script OUT_DIR {}: {err}", out_dir.display()))
        })?;
    }
    fs::create_dir_all(&out_dir)
        .map_err(|err| RunError::Internal(format!("creating build-script OUT_DIR {}: {err}", out_dir.display())))?;

    let executable_path = executable.canonicalize().unwrap_or_else(|_| executable.to_path_buf());
    let package_root = build_script_package_root(unit);
    let mut command = Command::new(&executable_path);
    let child_env = build_script_child_env(unit, options, &out_dir, package_root.as_deref());
    apply_rust_topology_child_env(&mut command, &child_env);
    if let Some(root) = &package_root {
        command.current_dir(root);
    }
    let output = command
        .output()
        .map_err(|err| RunError::Internal(format!("running build-script unit {}: {err}", unit.unit_id)))?;
    if !output.status.success() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "build-script-run-failed".to_string(),
            message: redacted_diagnostic(&output.stderr),
        }));
    }
    let stdout = match String::from_utf8(output.stdout.clone()) {
        Ok(stdout) => stdout,
        Err(_) => {
            return Ok(Err(RustUnitExecutionBlocker {
                class: "malformed-build-script-metadata".to_string(),
                message: "build-script stdout was not valid UTF-8".to_string(),
            }));
        }
    };
    let metadata = match parse_build_script_metadata(&stdout, &out_dir) {
        Ok(metadata) => metadata,
        Err(blocker) => return Ok(Err(blocker)),
    };
    let out_dir_artifact_digests = digest_build_script_out_dir(&out_dir)?;
    Ok(Ok(BuildScriptMetadataRunReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        execution_status: "success".to_string(),
        out_dir: metadata.out_dir,
        rustc_cfg: metadata.rustc_cfg,
        rustc_env: metadata.rustc_env,
        rustc_link_lib: metadata.rustc_link_lib,
        rustc_link_search: metadata.rustc_link_search,
        rerun_if_changed: metadata.rerun_if_changed,
        metadata: metadata.metadata,
        out_dir_artifact_digests,
        stdout_digest_blake3: blake3::hash(&output.stdout).to_hex().to_string(),
        metadata_digest_blake3: metadata.digest_blake3,
        blocker: None,
    }))
}

fn parse_build_script_metadata(
    stdout: &str,
    out_dir: &Path,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    let mut rustc_cfg = Vec::new();
    let mut rustc_env = BTreeMap::new();
    let mut rustc_link_lib = Vec::new();
    let mut rustc_link_search = Vec::new();
    let mut rerun_if_changed = Vec::new();
    let mut metadata = BTreeMap::new();
    for (line_index, line) in stdout.lines().enumerate() {
        let Some(payload) = cargo_metadata_payload(line) else {
            continue;
        };
        if let Some(value) = payload.strip_prefix("rustc-cfg=") {
            push_metadata_value(&mut rustc_cfg, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rustc-env=") {
            let Some((key, env_value)) = value.split_once('=') else {
                return Err(malformed_build_script_metadata(line_index, "rustc-env must be KEY=VALUE"));
            };
            if key.is_empty() {
                return Err(malformed_build_script_metadata(line_index, "rustc-env key must not be empty"));
            }
            rustc_env.insert(key.to_string(), env_value.to_string());
        } else if let Some(value) = payload.strip_prefix("rustc-link-lib=") {
            validate_rustc_link_lib_metadata(value, line_index)?;
            push_metadata_value(&mut rustc_link_lib, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rustc-link-search=") {
            validate_rustc_link_search_metadata(value, line_index)?;
            push_metadata_value(&mut rustc_link_search, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rerun-if-changed=") {
            push_metadata_value(&mut rerun_if_changed, value, line_index)?;
        } else if cargo_metadata_directive_is_ignored(payload) {
            continue;
        } else if let Some((key, value)) = payload.split_once('=') {
            validate_build_script_metadata_key(key, line_index)?;
            metadata.insert(key.to_string(), value.to_string());
        }
    }
    rustc_cfg.sort();
    rustc_cfg.dedup();
    rustc_link_lib.sort();
    rustc_link_lib.dedup();
    rustc_link_search.sort();
    rustc_link_search.dedup();
    rerun_if_changed.sort();
    rerun_if_changed.dedup();
    build_script_metadata_summary_from_parts(
        normalize_path_string(out_dir),
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
        metadata,
    )
}

fn cargo_metadata_payload(line: &str) -> Option<&str> {
    line.strip_prefix("cargo::").or_else(|| line.strip_prefix("cargo:"))
}

fn cargo_metadata_directive_is_ignored(payload: &str) -> bool {
    payload.starts_with("warning=")
        || payload.starts_with("rerun-if-env-changed=")
        || payload.starts_with("rustc-check-cfg=")
        || payload.starts_with("rustc-link-arg=")
}

fn validate_build_script_metadata_key(key: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if key.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata key must not be empty"));
    }
    if key.starts_with("rustc-") || key.starts_with("rerun-") || key.starts_with("warning") {
        return Err(malformed_build_script_metadata(line_index, "metadata key uses a reserved cargo prefix"));
    }
    if !key.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
        return Err(malformed_build_script_metadata(
            line_index,
            "metadata key must contain only ASCII alnum, underscore, or hyphen",
        ));
    }
    Ok(())
}

fn validate_rustc_link_lib_metadata(value: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    if contains_metadata_whitespace(value) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-lib must not contain whitespace"));
    }
    let name = if let Some((prefix, name)) = value.split_once('=') {
        validate_rustc_link_lib_prefix(prefix, line_index)?;
        name
    } else {
        value
    };
    if name.contains(':') || name.contains(',') {
        return Err(malformed_build_script_metadata(
            line_index,
            "rustc-link-lib renames and comma-separated names are not supported by this bounded rail",
        ));
    }
    if !is_safe_link_name(name) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-lib name must be a safe token"));
    }
    Ok(())
}

fn validate_rustc_link_lib_prefix(prefix: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    let (kind, modifiers) = prefix.split_once(':').map_or((prefix, None), |(kind, modifiers)| (kind, Some(modifiers)));
    if !rustc_link_lib_kind_is_supported(kind) {
        return Err(malformed_build_script_metadata(line_index, "unsupported rustc-link-lib kind"));
    }
    if let Some(modifiers) = modifiers {
        validate_rustc_link_lib_modifiers(modifiers, line_index)?;
    }
    Ok(())
}

fn rustc_link_lib_kind_is_supported(kind: &str) -> bool {
    matches!(kind, RUSTC_LINK_LIB_KIND_STATIC | RUSTC_LINK_LIB_KIND_DYLIB | RUSTC_LINK_LIB_KIND_FRAMEWORK)
}

fn validate_rustc_link_lib_modifiers(modifiers: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if modifiers.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-lib modifiers must not be empty"));
    }
    for modifier in modifiers.split(',') {
        if !is_safe_link_modifier(modifier) {
            return Err(malformed_build_script_metadata(line_index, "rustc-link-lib modifier must be a safe token"));
        }
    }
    Ok(())
}

fn validate_rustc_link_search_metadata(value: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    if contains_metadata_whitespace(value) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-search must not contain whitespace"));
    }
    let path = if let Some((kind, path)) = value.split_once('=') {
        if !matches!(kind, "dependency" | "crate" | "native" | "framework" | "all") {
            return Err(malformed_build_script_metadata(line_index, "unsupported rustc-link-search kind"));
        }
        path
    } else {
        value
    };
    if path.is_empty() || path.contains("..") {
        return Err(malformed_build_script_metadata(
            line_index,
            "rustc-link-search path must be non-empty and must not contain parent traversal",
        ));
    }
    Ok(())
}

fn contains_metadata_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn is_safe_link_name(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+'))
}

fn is_safe_link_modifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(prefix) = chars.next() else {
        return false;
    };
    if prefix != RUSTC_LINK_LIB_MODIFIER_ENABLE && prefix != RUSTC_LINK_LIB_MODIFIER_DISABLE {
        return false;
    }
    let token = chars.as_str();
    !token.is_empty() && token.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

fn push_metadata_value(
    values: &mut Vec<String>,
    value: &str,
    line_index: usize,
) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    values.push(value.to_string());
    Ok(())
}

fn malformed_build_script_metadata(line_index: usize, reason: &str) -> RustUnitExecutionBlocker {
    RustUnitExecutionBlocker {
        class: "malformed-build-script-metadata".to_string(),
        message: format!("build-script metadata line {} is malformed: {reason}", line_index + 1),
    }
}

fn build_script_metadata_summary_from_parts(
    out_dir: String,
    rustc_cfg: Vec<String>,
    rustc_env: BTreeMap<String, String>,
    rustc_link_lib: Vec<String>,
    rustc_link_search: Vec<String>,
    rerun_if_changed: Vec<String>,
    metadata: BTreeMap<String, String>,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        out_dir: &'a str,
        rustc_cfg: &'a [String],
        rustc_env: &'a BTreeMap<String, String>,
        rustc_link_lib: &'a [String],
        rustc_link_search: &'a [String],
        rerun_if_changed: &'a [String],
        metadata: &'a BTreeMap<String, String>,
    }
    let canonical = serde_json::to_vec(&Hashable {
        out_dir: &out_dir,
        rustc_cfg: &rustc_cfg,
        rustc_env: &rustc_env,
        rustc_link_lib: &rustc_link_lib,
        rustc_link_search: &rustc_link_search,
        rerun_if_changed: &rerun_if_changed,
        metadata: &metadata,
    })
    .map_err(|err| RustUnitExecutionBlocker {
        class: "malformed-build-script-metadata".to_string(),
        message: format!("canonicalizing build-script metadata failed: {err}"),
    })?;
    Ok(BuildScriptMetadataSummary {
        out_dir,
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
        metadata,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn build_script_metadata_from_run(run: &BuildScriptMetadataRunReceipt) -> BuildScriptMetadataSummary {
    BuildScriptMetadataSummary {
        out_dir: run.out_dir.clone(),
        rustc_cfg: run.rustc_cfg.clone(),
        rustc_env: run.rustc_env.clone(),
        rustc_link_lib: run.rustc_link_lib.clone(),
        rustc_link_search: run.rustc_link_search.clone(),
        rerun_if_changed: run.rerun_if_changed.clone(),
        metadata: run.metadata.clone(),
        digest_blake3: run.metadata_digest_blake3.clone(),
    }
}

fn digest_build_script_out_dir(out_dir: &Path) -> Result<Vec<RustExecutionArtifactDigest>, RunError> {
    let mut files = Vec::new();
    collect_output_files(out_dir, &mut files)?;
    let mut digests = Vec::new();
    for file in files {
        let bytes = fs::read(&file).map_err(|err| {
            RunError::Internal(format!("reading build-script OUT_DIR artifact {}: {err}", file.display()))
        })?;
        let relative = file.strip_prefix(out_dir).unwrap_or(file.as_path());
        digests.push(RustExecutionArtifactDigest {
            path: format!("out-dir/{}", normalize_path_string(relative)),
            blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(digests)
}

fn append_dependency_search_paths(
    unit: &mut RustUnitDerivationSummary,
    produced_artifacts: &BTreeMap<String, PathBuf>,
) {
    append_dependency_search_paths_from_paths(unit, produced_artifacts.values());
}

#[cfg(test)]
fn append_all_produced_dependency_search_paths(
    unit: &mut RustUnitDerivationSummary,
    produced_target_artifacts: &[PathBuf],
    produced_proc_macro_artifacts: &[PathBuf],
) {
    append_dependency_search_paths_from_paths(
        unit,
        produced_target_artifacts.iter().chain(produced_proc_macro_artifacts.iter()),
    );
}

fn append_dependency_search_paths_from_paths<'a>(
    unit: &mut RustUnitDerivationSummary,
    produced_artifacts: impl IntoIterator<Item = &'a PathBuf>,
) {
    let existing_search_args = rustc_dependency_search_args(&unit.derivation.args);
    let mut new_search_args = BTreeSet::new();
    for produced_artifact in produced_artifacts {
        if let Some(parent) = produced_artifact.parent() {
            let search_arg =
                format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(&rustc_artifact_path(parent)));
            if !existing_search_args.contains(&search_arg) {
                new_search_args.insert(search_arg);
            }
        }
    }
    if new_search_args.is_empty() {
        return;
    }
    for search_arg in new_search_args {
        unit.derivation.args.push(RUSTC_LINK_SEARCH_FLAG.to_string());
        unit.derivation.args.push(search_arg);
    }
    refresh_rustc_args_digest(unit);
}

fn rustc_dependency_search_args(args: &[String]) -> BTreeSet<String> {
    args.windows(RUSTC_EXTERN_ARG_PAIR_WIDTH)
        .filter_map(|window| {
            if window[0] == RUSTC_LINK_SEARCH_FLAG && window[1].starts_with(RUSTC_DEPENDENCY_SEARCH_PREFIX) {
                return Some(window[1].clone());
            }
            None
        })
        .collect()
}

fn execute_rust_unit(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitExecutionReceipt, RunError> {
    if unit.derivation.builder != "rustc" {
        return blocked_execution_receipt(
            Some(unit),
            "unsupported-builder",
            "only rustc-backed Rust unit derivations are supported by this execution rail",
        );
    }
    if !tool_exists(&options.rustc) {
        return blocked_execution_receipt(
            Some(unit),
            "missing-toolchain",
            &format!("rustc tool is not available: {}", options.rustc.display()),
        );
    }
    let toolchain = match rustc_toolchain_identity(&options.rustc)? {
        Ok(identity) => identity,
        Err(blocker) => return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message),
    };
    let src_path = rustc_source_path(&unit.derivation.args).ok_or_else(|| {
        RunError::Internal(format!("unit {} lacks a rustc source path in reviewable args", unit.unit_id))
    })?;
    if !src_path.is_file() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-source-material",
            &format!("declared Rust source is not readable: {}", src_path.display()),
        );
    }
    if unit.derivation.outputs.is_empty() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-declared-output",
            "unit derivation does not declare any output artifact paths",
        );
    }
    let dependency_artifact_digests = artifact_digests(
        &unit.dependency_artifacts.iter().map(|artifact| artifact.artifact.as_str()).collect::<Vec<_>>(),
        "missing-dependency-artifact",
    )?;
    if let Err(blocker) = &dependency_artifact_digests {
        return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message);
    }
    let host_artifact_digests = artifact_digests(
        &unit.consumed_host_artifacts.iter().map(|artifact| artifact.artifact.as_str()).collect::<Vec<_>>(),
        "missing-host-artifact",
    )?;
    if let Err(blocker) = &host_artifact_digests {
        return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message);
    }

    let unit_output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    let dependency_artifact_digests = dependency_artifact_digests.unwrap();
    let host_artifact_digests = host_artifact_digests.unwrap();
    let environment_digest_blake3 = rust_derivation_env_digest(&unit.derivation.env)?;
    if let Some(receipt) = try_reuse_rust_unit_outputs(
        unit,
        &unit_output_dir,
        toolchain.clone(),
        environment_digest_blake3.clone(),
        dependency_artifact_digests.clone(),
        host_artifact_digests.clone(),
    )? {
        return Ok(receipt);
    }
    if unit_output_dir.exists() {
        fs::remove_dir_all(&unit_output_dir).map_err(|err| {
            RunError::Internal(format!("removing prior Rust unit output {}: {err}", unit_output_dir.display()))
        })?;
    }
    fs::create_dir_all(&unit_output_dir)
        .map_err(|err| RunError::Internal(format!("creating Rust unit output {}: {err}", unit_output_dir.display())))?;

    let mut command = Command::new(&options.rustc);
    command.args(rust_topology_runtime_args(&unit.derivation.args));
    command.arg("--out-dir").arg(&unit_output_dir);
    apply_rust_topology_child_env(&mut command, &unit.derivation.env);
    let output = command
        .output()
        .map_err(|err| RunError::Internal(format!("executing rustc for unit {}: {err}", unit.unit_id)))?;
    if !output.status.success() {
        let diagnostic = redacted_diagnostic(&output.stderr);
        return failed_execution_receipt(
            unit,
            toolchain,
            dependency_artifact_digests,
            host_artifact_digests,
            &diagnostic,
        );
    }
    let output_artifact_digests = digest_output_artifacts(&unit_output_dir)?;
    if output_artifact_digests.is_empty() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-declared-output",
            "rustc completed but produced no declared output artifacts",
        );
    }
    let receipt = finalized_execution_receipt(
        unit,
        "success",
        "rebuilt-explicit-unit",
        toolchain,
        environment_digest_blake3,
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        None,
    )?;
    write_rust_unit_execution_receipt(&unit_output_dir, &receipt)?;
    Ok(receipt)
}

fn try_reuse_rust_unit_outputs(
    unit: &RustUnitDerivationSummary,
    unit_output_dir: &Path,
    toolchain: RustToolchainIdentity,
    environment_digest_blake3: String,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
) -> Result<Option<RustUnitExecutionReceipt>, RunError> {
    let receipt_path = unit_output_dir.join(RUST_UNIT_EXECUTION_RECEIPT_FILE);
    if !receipt_path.exists() {
        return Ok(None);
    }
    let prior_receipt = match fs::read(&receipt_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<RustUnitExecutionReceipt>(&bytes).ok())
    {
        Some(receipt) => receipt,
        None => {
            return stale_cached_output_receipt(
                unit,
                toolchain,
                environment_digest_blake3.clone(),
                dependency_artifact_digests,
                host_artifact_digests,
                Vec::new(),
                "prior execution receipt is missing or malformed",
            )
            .map(Some);
        }
    };
    let current_output_artifact_digests = digest_output_artifacts(unit_output_dir)?;
    if current_output_artifact_digests.is_empty() {
        return stale_cached_output_receipt(
            unit,
            toolchain,
            environment_digest_blake3.clone(),
            dependency_artifact_digests,
            host_artifact_digests,
            current_output_artifact_digests,
            "prior execution receipt exists but no declared output artifacts are readable",
        )
        .map(Some);
    }
    let expected_outputs = sorted_strings(unit.derivation.outputs.clone());
    let matches_current_inputs = prior_receipt.execution_status == "success"
        && prior_receipt.unit_id == unit.unit_id
        && prior_receipt.package_id == unit.package_id
        && prior_receipt.target_name == unit.target_name
        && prior_receipt.target_kind == unit.target_kind
        && prior_receipt.source_digest == unit.source_digest
        && prior_receipt.toolchain == toolchain
        && prior_receipt.rustc_args_digest_blake3 == unit.rustc_args_digest_blake3
        && prior_receipt.environment_digest_blake3 == environment_digest_blake3
        && prior_receipt.declared_outputs == expected_outputs
        && prior_receipt.dependency_artifact_digests == dependency_artifact_digests
        && prior_receipt.host_artifact_digests == host_artifact_digests
        && prior_receipt.output_artifact_digests == current_output_artifact_digests
        && prior_receipt.blocker.is_none();
    if !matches_current_inputs {
        return stale_cached_output_receipt(
            unit,
            toolchain,
            environment_digest_blake3.clone(),
            dependency_artifact_digests,
            host_artifact_digests,
            current_output_artifact_digests,
            "prior execution receipt does not match current explicit inputs or output artifact digests",
        )
        .map(Some);
    }
    let receipt = finalized_execution_receipt(
        unit,
        "success",
        "reused-explicit-unit-output",
        toolchain,
        environment_digest_blake3,
        dependency_artifact_digests,
        host_artifact_digests,
        current_output_artifact_digests,
        None,
    )?;
    write_rust_unit_execution_receipt(unit_output_dir, &receipt)?;
    Ok(Some(receipt))
}

fn stale_cached_output_receipt(
    unit: &RustUnitDerivationSummary,
    toolchain: RustToolchainIdentity,
    environment_digest_blake3: String,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    message: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    finalized_execution_receipt(
        unit,
        "blocked",
        "not-run-stale-cached-output",
        toolchain,
        environment_digest_blake3,
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        Some(RustUnitExecutionBlocker {
            class: "stale-cached-output".to_string(),
            message: message.to_string(),
        }),
    )
}

fn write_rust_unit_execution_receipt(
    unit_output_dir: &Path,
    receipt: &RustUnitExecutionReceipt,
) -> Result<(), RunError> {
    let receipt_path = unit_output_dir.join(RUST_UNIT_EXECUTION_RECEIPT_FILE);
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit execution receipt for cache: {err}")))?;
    fs::write(&receipt_path, bytes).map_err(|err| {
        RunError::Internal(format!("writing Rust unit execution receipt {}: {err}", receipt_path.display()))
    })
}

fn is_supported_target_unit(unit: &RustUnitDerivationSummary) -> bool {
    unit.execution_kind == "target" && matches!(unit.target_kind.as_str(), "lib" | "bin")
}

fn is_supported_host_unit(unit: &RustUnitDerivationSummary) -> bool {
    unit.execution_kind == "host" && matches!(unit.target_kind.as_str(), "proc-macro" | "custom-build")
}

fn is_supported_host_dependency_unit(unit: &RustUnitDerivationSummary) -> bool {
    unit.execution_kind == HOST_DEPENDENCY_EXECUTION_KIND && unit.target_kind == "lib"
}

fn produced_host_artifact_path(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<Result<PathBuf, RustUnitExecutionBlocker>, RunError> {
    let output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    if !output_dir.is_dir() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-produced-host-artifact".to_string(),
            message: format!("host output directory is missing: {}", output_dir.display()),
        }));
    }
    let mut files = Vec::new();
    collect_output_files(&output_dir, &mut files)?;
    let crate_name = rust_crate_name(&unit.target_name);
    let artifact = if unit.target_kind == "proc-macro" {
        files.into_iter().find(|path| {
            matches!(path.extension().and_then(OsStr::to_str), Some("so" | "dylib" | "dll"))
                && path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.contains(&crate_name))
        })
    } else {
        files.into_iter().find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name == crate_name || name.contains(&crate_name))
        })
    }
    .ok_or_else(|| RustUnitExecutionBlocker {
        class: "missing-produced-host-artifact".to_string(),
        message: format!("host unit {} did not emit a matching artifact", unit.unit_id),
    });
    Ok(artifact.map_err(|blocker| blocker))
}

fn produced_library_artifact_path(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<Result<PathBuf, RustUnitExecutionBlocker>, RunError> {
    let output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    if !output_dir.is_dir() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-produced-dependency-artifact".to_string(),
            message: format!("producer output directory is missing: {}", output_dir.display()),
        }));
    }
    let mut files = Vec::new();
    collect_output_files(&output_dir, &mut files)?;
    let crate_prefix = format!("lib{}", rust_crate_name(&unit.target_name));
    let artifact = files
        .into_iter()
        .find(|path| {
            path.extension().and_then(OsStr::to_str) == Some("rlib")
                && path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.starts_with(&crate_prefix))
        })
        .ok_or_else(|| RustUnitExecutionBlocker {
            class: "missing-produced-dependency-artifact".to_string(),
            message: format!("producer unit {} did not emit a matching .rlib artifact", unit.unit_id),
        });
    Ok(artifact.map_err(|blocker| blocker))
}

fn bind_dependency_artifact(
    consumer: &RustUnitDerivationSummary,
    dependency: &RustDependencyArtifact,
    produced_artifact: &Path,
) -> Result<RustUnitDerivationSummary, RunError> {
    let produced_artifact = normalize_path_string(&rustc_artifact_path(produced_artifact));
    let mut bound = consumer.clone();
    for artifact in &mut bound.dependency_artifacts {
        if artifact.package_id == dependency.package_id
            && artifact.name == dependency.name
            && artifact.artifact == dependency.artifact
        {
            artifact.artifact = produced_artifact.clone();
        }
    }
    for input in &mut bound.derivation.inputs {
        if input == &dependency.artifact {
            *input = produced_artifact.clone();
        }
    }
    let expected_extern = format!("{}={}", dependency.name, dependency.artifact);
    let rewritten_extern = format!("{}={produced_artifact}", dependency.name);
    let mut replaced = false;
    for arg in &mut bound.derivation.args {
        if arg == &expected_extern {
            *arg = rewritten_extern.clone();
            replaced = true;
        }
    }
    if !replaced {
        return Err(RunError::Internal(format!(
            "consumer unit {} lacks expected dependency extern {}",
            consumer.unit_id, expected_extern
        )));
    }
    bound.rustc_args_digest_blake3 = blake3::hash(bound.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    Ok(bound)
}

fn dependency_chain_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitDependencyChainExecutionReceipt, RunError> {
    let mut receipt = RustUnitDependencyChainExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded explicit Rust dependency edge only; not full Cargo compatibility or a general scheduler"
            .to_string(),
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_dependency_chain_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_dependency_chain_execution_receipt_hash(
    receipt: &RustUnitDependencyChainExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust dependency-chain execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn target_topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitTargetTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitTargetTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded target-only Rust unit topology; not full Cargo compatibility, host artifact execution, or a general scheduler"
            .to_string(),
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_target_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_target_topology_execution_receipt_hash(
    receipt: &RustUnitTargetTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust target-topology execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn host_artifact_topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitHostArtifactTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded Rust host-artifact topology with build-script metadata; not full Cargo compatibility, native-link probing, or a general scheduler"
            .to_string(),
        unit_executions,
        build_script_metadata_runs,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_host_artifact_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_host_artifact_topology_execution_receipt_hash(
    receipt: &RustUnitHostArtifactTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable).map_err(|err| {
        RunError::Internal(format!("canonicalizing Rust host-artifact topology execution receipt: {err}"))
    })?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded unified Rust unit topology with target dependencies, host artifacts, and build-script metadata; not full Cargo compatibility, parallel scheduling, or a general scheduler"
            .to_string(),
        unit_executions,
        build_script_metadata_runs,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_topology_execution_receipt_hash(receipt: &RustUnitTopologyExecutionReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust topology execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_package_test_edition(package: &NativePackagePlanningSummary) -> String {
    package
        .targets
        .first()
        .map(|target| target.edition.clone())
        .unwrap_or_else(|| DEFAULT_RUST_EDITION.to_string())
}

fn dev_dependency_lib_derivation(
    package: &NativePackagePlanningSummary,
    dependency_name: &str,
) -> Result<RustUnitDerivationSummary, RustUnitExecutionBlocker> {
    let libs = package.targets.iter().filter(|target| target.kind == "lib").collect::<Vec<_>>();
    if libs.len() != 1 {
        return Err(RustUnitExecutionBlocker {
            class: "missing-dev-dependency-artifact".to_string(),
            message: format!("dev dependency `{dependency_name}` has no supported lib derivation"),
        });
    }
    let target = libs[0];
    let unit_id = format!("native-dev-dependency-lib:{}:{}", package.package_id, target.name);
    let args = vec![
        "--crate-name".to_string(),
        target.crate_name.clone(),
        "--edition".to_string(),
        target.edition.clone(),
        target.source_path.clone(),
        "--emit=link".to_string(),
        "--crate-type".to_string(),
        "lib".to_string(),
    ];
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), "lib".to_string());
    env.insert("MODE".to_string(), "build".to_string());
    env.insert("PACKAGE_ID".to_string(), package.package_id.clone());
    env.insert("PROFILE".to_string(), DEFAULT_CARGO_PROFILE.to_string());
    Ok(RustUnitDerivationSummary {
        unit_id,
        package_id: package.package_id.clone(),
        target_name: target.name.clone(),
        target_kind: "lib".to_string(),
        execution_kind: "target".to_string(),
        crate_types: vec!["lib".to_string()],
        mode: "build".to_string(),
        profile: DEFAULT_CARGO_PROFILE.to_string(),
        source_digest: package.source_digest.clone(),
        dependency_artifacts: Vec::new(),
        consumed_host_artifacts: Vec::new(),
        metadata_dependencies: Vec::new(),
        generated_metadata: None,
        derivation: ReviewableRustDerivation {
            name: derivation_name(&target.name, 0),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs: vec![format!("source:{}:{}", package.package_id, package.source_digest.value)],
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    })
}

fn dev_dependency_test_topology_receipt(
    execution_status: &str,
    package_id: Option<String>,
    test_target: Option<String>,
    dev_dependency_packages: Vec<String>,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustDevDependencyTestTopologyExecutionReceipt, RunError> {
    let mut receipt = RustDevDependencyTestTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded native Rust dev-dependency test topology; explicit harness=false test target only, not full cargo test compatibility or Cargo orchestration".to_string(),
        package_id,
        test_target,
        dev_dependency_packages,
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_dev_dependency_test_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_dev_dependency_test_topology_execution_receipt_hash(
    receipt: &RustDevDependencyTestTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable).map_err(|err| {
        RunError::Internal(format!("canonicalizing Rust dev-dependency test topology execution receipt: {err}"))
    })?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn workspace_dependency_topology_receipt(
    execution_status: &str,
    workspace_root: Option<String>,
    member_package_id: Option<String>,
    inherited_dependency_packages: Vec<String>,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustWorkspaceDependencyTopologyExecutionReceipt, RunError> {
    let mut receipt = RustWorkspaceDependencyTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded native registry workspace-dependency topology; explicit workspace inheritance facts only, not Cargo orchestration, Cargo resolver fallback, registry cache fallback, or network/index access".to_string(),
        workspace_root,
        member_package_id,
        inherited_dependency_packages,
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_workspace_dependency_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_workspace_dependency_topology_execution_receipt_hash(
    receipt: &RustWorkspaceDependencyTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable).map_err(|err| {
        RunError::Internal(format!("canonicalizing Rust workspace-dependency topology execution receipt: {err}"))
    })?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn patch_source_topology_receipt(
    execution_status: &str,
    consumer_package_id: Option<String>,
    patch_source_packages: Vec<String>,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustPatchSourceTopologyExecutionReceipt, RunError> {
    let mut receipt = RustPatchSourceTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded native registry patch-source topology; explicit local [patch.crates-io] source facts only, not Cargo orchestration, Cargo resolver fallback, registry cache fallback, or network/index access".to_string(),
        consumer_package_id,
        patch_source_packages,
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_patch_source_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_patch_source_topology_execution_receipt_hash(
    receipt: &RustPatchSourceTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable).map_err(|err| {
        RunError::Internal(format!("canonicalizing Rust patch-source topology execution receipt: {err}"))
    })?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn blocked_execution_receipt(
    unit: Option<&RustUnitDerivationSummary>,
    class: &str,
    message: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let fallback_source_digest = SourceDigest {
        algorithm: "missing".to_string(),
        value: "missing".to_string(),
    };
    let (unit_id, package_id, target_name, target_kind, source_digest, rustc_args_digest_blake3, derivation_env) = unit
        .map_or_else(
            || {
                (
                    "missing".to_string(),
                    "missing".to_string(),
                    "missing".to_string(),
                    "missing".to_string(),
                    fallback_source_digest.clone(),
                    "missing".to_string(),
                    BTreeMap::new(),
                )
            },
            |unit| {
                (
                    unit.unit_id.clone(),
                    unit.package_id.clone(),
                    unit.target_name.clone(),
                    unit.target_kind.clone(),
                    unit.source_digest.clone(),
                    unit.rustc_args_digest_blake3.clone(),
                    unit.derivation.env.clone(),
                )
            },
        );
    finalized_execution_receipt(
        &RustUnitDerivationSummary {
            unit_id,
            package_id,
            target_name,
            target_kind,
            execution_kind: "target".to_string(),
            crate_types: Vec::new(),
            mode: "build".to_string(),
            profile: String::new(),
            source_digest,
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
            derivation: ReviewableRustDerivation {
                name: String::new(),
                builder: "rustc".to_string(),
                system: String::new(),
                args: Vec::new(),
                outputs: Vec::new(),
                env: derivation_env.clone(),
                inputs: Vec::new(),
                addressing_mode: String::new(),
            },
            rustc_args_digest_blake3,
        },
        "blocked",
        "not-run-preflight-blocker",
        missing_toolchain_identity(),
        rust_derivation_env_digest(&derivation_env)?,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Some(RustUnitExecutionBlocker {
            class: class.to_string(),
            message: message.to_string(),
        }),
    )
}

fn failed_execution_receipt(
    unit: &RustUnitDerivationSummary,
    toolchain: RustToolchainIdentity,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    diagnostic: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    finalized_execution_receipt(
        unit,
        "failed",
        "rustc-exit-nonzero",
        toolchain,
        rust_derivation_env_digest(&unit.derivation.env)?,
        dependency_artifact_digests,
        host_artifact_digests,
        Vec::new(),
        Some(RustUnitExecutionBlocker {
            class: "rustc-failed".to_string(),
            message: diagnostic.to_string(),
        }),
    )
}

fn finalized_execution_receipt(
    unit: &RustUnitDerivationSummary,
    execution_status: &str,
    rebuild_reason: &str,
    toolchain: RustToolchainIdentity,
    environment_digest_blake3: String,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let mut receipt = RustUnitExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_status: execution_status.to_string(),
        rebuild_reason: rebuild_reason.to_string(),
        source_digest: unit.source_digest.clone(),
        toolchain,
        rustc_args_digest_blake3: unit.rustc_args_digest_blake3.clone(),
        environment_digest_blake3,
        declared_outputs: sorted_strings(unit.derivation.outputs.clone()),
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_derivation_env_digest(env: &BTreeMap<String, String>) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(env)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit environment digest: {err}")))?;
    let digest = blake3::hash(&canonical).to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(digest)
}

fn rust_unit_execution_receipt_hash(receipt: &RustUnitExecutionReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn rustc_toolchain_identity(tool: &Path) -> Result<Result<RustToolchainIdentity, RustUnitExecutionBlocker>, RunError> {
    let output = Command::new(tool)
        .arg("-vV")
        .output()
        .map_err(|err| RunError::Internal(format!("querying rustc toolchain identity {}: {err}", tool.display())))?;
    if !output.status.success() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-toolchain-identity".to_string(),
            message: format!("rustc -vV failed for {}", tool.display()),
        }));
    }
    let version_verbose = String::from_utf8(output.stdout)
        .map_err(|err| RunError::Internal(format!("rustc -vV emitted non-UTF-8 output: {err}")))?
        .trim()
        .to_string();
    Ok(Ok(RustToolchainIdentity {
        tool: tool.file_name().and_then(OsStr::to_str).unwrap_or("rustc").to_string(),
        version_digest_blake3: blake3::hash(version_verbose.as_bytes()).to_hex().to_string(),
        version_verbose,
    }))
}

fn missing_toolchain_identity() -> RustToolchainIdentity {
    RustToolchainIdentity {
        tool: "missing".to_string(),
        version_verbose: "missing".to_string(),
        version_digest_blake3: "missing".to_string(),
    }
}

fn rustc_source_path(args: &[String]) -> Option<PathBuf> {
    args.iter().find(|arg| !arg.starts_with('-') && arg.ends_with(".rs")).map(PathBuf::from)
}

fn artifact_digests(
    paths: &[&str],
    missing_class: &str,
) -> Result<Result<Vec<RustExecutionArtifactDigest>, RustUnitExecutionBlocker>, RunError> {
    let mut digests = Vec::new();
    for path in paths {
        let artifact_path = Path::new(path);
        if !artifact_path.is_file() {
            return Ok(Err(RustUnitExecutionBlocker {
                class: missing_class.to_string(),
                message: format!("declared artifact is not readable: {path}"),
            }));
        }
        digests.push(digest_artifact_path(artifact_path)?);
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Ok(digests))
}

fn digest_output_artifacts(output_dir: &Path) -> Result<Vec<RustExecutionArtifactDigest>, RunError> {
    let mut paths = Vec::new();
    collect_output_files(output_dir, &mut paths)?;
    let mut digests = Vec::new();
    for path in paths {
        if path.file_name().and_then(OsStr::to_str) == Some(RUST_UNIT_EXECUTION_RECEIPT_FILE) {
            continue;
        }
        let relative_path = path.strip_prefix(output_dir).unwrap_or(&path);
        let receipt_path = format!("declared-output/{}", normalize_path_string(relative_path));
        digests.push(digest_artifact_path_with_receipt_path(&path, receipt_path)?);
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(digests)
}

fn collect_output_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let entries = fs::read_dir(directory)
        .map_err(|err| RunError::Internal(format!("reading Rust unit output {}: {err}", directory.display())))?;
    for entry in entries {
        let entry = entry
            .map_err(|err| RunError::Internal(format!("reading Rust unit output {}: {err}", directory.display())))?;
        let file_type = entry.file_type().map_err(|err| {
            RunError::Internal(format!("reading Rust unit output metadata {}: {err}", entry.path().display()))
        })?;
        if file_type.is_dir() {
            collect_output_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(())
}

fn digest_artifact_path(path: &Path) -> Result<RustExecutionArtifactDigest, RunError> {
    digest_artifact_path_with_receipt_path(path, normalize_path_string(path))
}

fn digest_artifact_path_with_receipt_path(
    path: &Path,
    receipt_path: String,
) -> Result<RustExecutionArtifactDigest, RunError> {
    let bytes =
        fs::read(path).map_err(|err| RunError::Internal(format!("reading artifact {}: {err}", path.display())))?;
    Ok(RustExecutionArtifactDigest {
        path: receipt_path,
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn safe_path_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn tool_exists(tool: &Path) -> bool {
    if tool.components().count() > 1 || tool.is_absolute() {
        return tool.is_file();
    }
    resolve_tool_path(tool).is_some()
}

fn resolve_tool_path(tool: impl AsRef<Path>) -> Option<PathBuf> {
    let tool = tool.as_ref();
    if tool.components().count() > 1 || tool.is_absolute() {
        return tool.is_file().then(|| tool.to_path_buf());
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(tool))
        .find(|candidate| candidate.is_file())
}

fn redacted_diagnostic(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let compact = text.lines().take(REDACTED_DIAGNOSTIC_MAX_LINES).collect::<Vec<_>>().join("\n");
    if compact.is_empty() {
        "rustc exited with a nonzero status".to_string()
    } else {
        redact_temp_paths(&compact.replace('\0', ""))
    }
}

fn redact_temp_paths(text: &str) -> String {
    debug_assert!(!REDACTED_DIAGNOSTIC_TEMP_PATH.is_empty());
    debug_assert!(!TEMP_PATH_PREFIXES.is_empty());
    let mut redacted = String::with_capacity(text.len());
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some((prefix_offset, prefix)) = next_temp_path_prefix(&text[cursor..]) else {
            redacted.push_str(&text[cursor..]);
            break;
        };
        let start = cursor + prefix_offset;
        redacted.push_str(&text[cursor..start]);
        redacted.push_str(REDACTED_DIAGNOSTIC_TEMP_PATH);
        let path_start = start + prefix.len();
        let path_tail = text[path_start..].find(temp_path_delimiter).map_or(text.len(), |offset| path_start + offset);
        cursor = path_tail;
    }
    redacted
}

fn next_temp_path_prefix(text: &str) -> Option<(usize, &'static str)> {
    TEMP_PATH_PREFIXES
        .iter()
        .filter_map(|prefix| text.find(prefix).map(|offset| (offset, *prefix)))
        .min_by_key(|(offset, _prefix)| *offset)
}

fn temp_path_delimiter(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | ')' | '(' | '[' | ']')
}

fn summarize_unit_graph(unit_graph: Value) -> Result<UnitGraphSummary, RunError> {
    let normalized = normalize_json_value(unit_graph);
    let canonical = serde_json::to_vec(&normalized)
        .map_err(|err| RunError::Internal(format!("canonicalizing unit graph JSON: {err}")))?;
    let unit_count = normalized.get("units").and_then(Value::as_array).map_or(0, Vec::len);
    let root_count = normalized.get("roots").and_then(Value::as_array).map_or(0, Vec::len);
    Ok(UnitGraphSummary {
        unit_count,
        root_count,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn normalize_json_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(normalize_json_value).collect()),
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(key, value)| (key, normalize_json_value(value))).collect();
            let mut normalized = serde_json::Map::new();
            for (key, value) in sorted {
                normalized.insert(key, value);
            }
            Value::Object(normalized)
        }
        other => other,
    }
}

fn receipt_hash(receipt: &RustPlanReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust plan receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn sorted_strings(mut strings: Vec<String>) -> Vec<String> {
    strings.sort();
    strings.dedup();
    strings
}

fn rustc_artifact_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(path)
}

fn normalize_path_string(path: &Path) -> String {
    path.components().as_path().to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    const PROFILE_ENV_CHILD_PROBE_ENV: &str = "MANTLE_PROFILE_ENV_CHILD_PROBE";
    const PROFILE_ENV_CHILD_PROBE_VALUE: &str = "present";
    const AMBIENT_OPT_LEVEL_VALUE: &str = "ambient-opt-level";
    const AMBIENT_DEBUG_VALUE: &str = "ambient-debug";
    const AMBIENT_NUM_JOBS_VALUE: &str = "999";

    struct FixedOracle {
        cargo_version: CargoOutput,
        rustc_version: CargoOutput,
        metadata: CargoOutput,
        unit_graph: CargoOutput,
    }

    impl CargoOracle for FixedOracle {
        fn run_cargo(&self, _root: &Path, _cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError> {
            if args.first().map(String::as_str) == Some("metadata") {
                return Ok(self.metadata.clone());
            }
            Ok(self.unit_graph.clone())
        }

        fn run_tool_version(&self, tool: &Path, _args: &[&str]) -> Result<CargoOutput, RunError> {
            if tool.file_name() == Some(OsStr::new("rustc")) {
                return Ok(self.rustc_version.clone());
            }
            Ok(self.cargo_version.clone())
        }
    }

    fn ok_output(text: &str) -> CargoOutput {
        CargoOutput {
            stdout: text.as_bytes().to_vec(),
            stderr: Vec::new(),
            status_code: 0,
        }
    }

    fn failing_output(text: &str) -> CargoOutput {
        CargoOutput {
            stdout: Vec::new(),
            stderr: text.as_bytes().to_vec(),
            status_code: 101,
        }
    }

    fn options(root: &Path) -> RustPlanOptions {
        RustPlanOptions {
            root: root.to_path_buf(),
            cargo: PathBuf::from("cargo"),
            rustc: PathBuf::from("rustc"),
            targets: vec!["x86_64-unknown-linux-gnu".to_string()],
            profile: default_profile(),
            features: vec!["b".to_string(), "a".to_string()],
            all_features: false,
            no_default_features: true,
            no_cargo_oracle: false,
        }
    }

    fn empty_unit_graph() -> Value {
        serde_json::json!({"units": []})
    }

    fn empty_registry_planning() -> NativeRegistrySourcePlanningSummary {
        NativeRegistrySourcePlanningSummary {
            ready: true,
            comparison_status: "matched".to_string(),
            lockfile_digest_blake3: "test-lockfile".to_string(),
            digest_blake3: "test-registry".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
            non_claims: vec!["declared-local-vendor-source-only".to_string()],
        }
    }

    fn test_registry_source(name: &str, version: &str, manifest_path: &Path) -> NativeRegistrySourceSummary {
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index";
        NativeRegistrySourceSummary {
            package_id: format!("{registry_source}#{name}@{version}"),
            name: name.to_string(),
            version: version.to_string(),
            source: registry_source.to_string(),
            source_class: "crates-io".to_string(),
            checksum: blake3::hash(format!("{name}-{version}").as_bytes()).to_hex().to_string(),
            vendor_root: normalize_path_string(manifest_path.parent().unwrap_or_else(|| Path::new("."))),
            manifest_path: normalize_path_string(manifest_path),
            source_digest: test_source_digest(&format!("{name}-{version}")),
            lockfile_identity: LockPackageIdentity {
                name: name.to_string(),
                version: version.to_string(),
                source: Some(registry_source.to_string()),
                checksum: None,
            },
        }
    }

    fn empty_git_planning() -> NativeGitSourcePlanningSummary {
        NativeGitSourcePlanningSummary {
            ready: true,
            comparison_status: "matched".to_string(),
            lockfile_digest_blake3: "test-lockfile".to_string(),
            source_closure_digest_blake3: "test-source-closure".to_string(),
            digest_blake3: "test-git".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
            non_claims: vec!["captured-source-closure-only".to_string()],
        }
    }

    fn test_source_digest(seed: &str) -> SourceDigest {
        let value = blake3::hash(seed.as_bytes()).to_hex().to_string();
        SourceDigest {
            algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
            value,
        }
    }

    fn test_cargo_package_env(name: &str, version: &str) -> BTreeMap<String, String> {
        let package = NativeManifestPackage {
            name: name.to_string(),
            version: NativeManifestInheritedString {
                value: Some(version.to_string()),
                workspace: false,
            },
            authors: None,
            edition: NativeManifestInheritedString::default(),
            build: None,
            description: None,
            homepage: None,
            license: None,
            license_file: None,
            links: None,
            readme: None,
            repository: None,
            rust_version: None,
        };
        native_cargo_package_env(&package, &NativeWorkspacePackage::default(), version)
    }

    fn test_native_package(
        package_id: &str,
        name: &str,
        target_kind: &str,
        path_dependencies: Vec<NativePathDependencySummary>,
    ) -> NativePackagePlanningSummary {
        NativePackagePlanningSummary {
            package_id: package_id.to_string(),
            name: name.to_string(),
            version: "0.1.0".to_string(),
            manifest_path: format!("/test/{name}/Cargo.toml"),
            links: None,
            cargo_package_env: test_cargo_package_env(name, "0.1.0"),
            selected_features: Vec::new(),
            targets: vec![NativeTargetPlanningSummary {
                name: name.to_string(),
                kind: target_kind.to_string(),
                crate_name: rust_crate_name(name),
                source_path: format!("/test/{name}/src/lib.rs"),
                edition: "2021".to_string(),
            }],
            path_dependencies,
            build_dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            target_cfg_dependencies: Vec::new(),
            workspace_dependencies: Vec::new(),
            source_digest: test_source_digest(package_id),
        }
    }

    fn test_package_planning(packages: Vec<NativePackagePlanningSummary>) -> NativePackageTargetPlanningSummary {
        NativePackageTargetPlanningSummary {
            ready: true,
            comparison_status: "matched".to_string(),
            cargo_oracle_identity: "test-oracle".to_string(),
            digest_blake3: "test-package-planning".to_string(),
            packages,
            blockers: Vec::new(),
            non_claims: Vec::new(),
        }
    }

    fn test_source_closure(packages: &[NativePackagePlanningSummary]) -> SourceClosureSummary {
        SourceClosureSummary {
            source_count: packages.len(),
            ready: true,
            digest_blake3: "test-source-closure".to_string(),
            sources: packages
                .iter()
                .map(|package| SourceInputSummary {
                    package_id: package.package_id.clone(),
                    name: package.name.clone(),
                    version: package.version.clone(),
                    kind: SourceKind::Path,
                    source: None,
                    manifest_path: package.manifest_path.clone(),
                    lockfile_identity: None,
                    resolved_revision: None,
                    source_digest: package.source_digest.clone(),
                })
                .collect(),
            blockers: Vec::new(),
        }
    }

    fn test_source_closure_with_kind(package_id: &str, package_name: &str, kind: SourceKind) -> SourceClosureSummary {
        SourceClosureSummary {
            source_count: 1,
            ready: true,
            digest_blake3: "test-source-closure".to_string(),
            sources: vec![SourceInputSummary {
                package_id: package_id.to_string(),
                name: package_name.to_string(),
                version: "0.1.0".to_string(),
                kind,
                source: None,
                manifest_path: format!("/{package_name}/Cargo.toml"),
                lockfile_identity: None,
                resolved_revision: None,
                source_digest: test_source_digest(package_id),
            }],
            blockers: Vec::new(),
        }
    }

    fn test_native_rust_unit(
        package_id: &str,
        package_name: &str,
        target_kind: &str,
        root: &Path,
    ) -> NativeRustUnitSummary {
        NativeRustUnitSummary {
            unit_id: rust_unit_id(0, package_id, package_name, target_kind, "build"),
            package_id: package_id.to_string(),
            package_name: package_name.to_string(),
            package_links: None,
            package_root: normalize_path_string(&root.join(package_name)),
            cargo_package_env: test_cargo_package_env(package_name, "0.1.0"),
            target_name: package_name.to_string(),
            target_kind: target_kind.to_string(),
            crate_name: rust_crate_name(package_name),
            source_path: root.join(package_name).join("src/lib.rs").display().to_string(),
            edition: "2021".to_string(),
            selected_features: Vec::new(),
            crate_types: vec![target_kind.to_string()],
            mode: "build".to_string(),
            profile: DEFAULT_CARGO_PROFILE.to_string(),
            source_digest: test_source_digest(package_id),
            dependency_artifacts: Vec::new(),
        }
    }

    fn test_native_host_unit(
        package_id: &str,
        package_name: &str,
        target_kind: &str,
        root: &Path,
    ) -> NativeHostUnitSummary {
        NativeHostUnitSummary {
            unit_id: rust_unit_id(0, package_id, package_name, target_kind, "build"),
            package_id: package_id.to_string(),
            package_name: package_name.to_string(),
            package_links: None,
            package_root: normalize_path_string(&root.join(package_name)),
            cargo_package_env: test_cargo_package_env(package_name, "0.1.0"),
            target_name: package_name.to_string(),
            target_kind: target_kind.to_string(),
            crate_name: rust_crate_name(package_name),
            source_path: root.join(package_name).join("src/lib.rs").display().to_string(),
            edition: "2021".to_string(),
            selected_features: Vec::new(),
            crate_types: vec![target_kind.to_string()],
            mode: "build".to_string(),
            profile: DEFAULT_CARGO_PROFILE.to_string(),
            source_digest: test_source_digest(package_id),
            artifact: test_host_artifact(package_id, target_kind),
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
        }
    }

    fn test_rust_derivation(
        index: usize,
        package_id: &str,
        target_kind: &str,
        execution_kind: &str,
        dependencies: Vec<RustDependencyArtifact>,
    ) -> RustUnitDerivationSummary {
        RustUnitDerivationSummary {
            unit_id: rust_unit_id(index, package_id, package_id, target_kind, "build"),
            package_id: package_id.to_string(),
            target_name: package_id.to_string(),
            target_kind: target_kind.to_string(),
            execution_kind: execution_kind.to_string(),
            crate_types: vec![target_kind.to_string()],
            mode: "build".to_string(),
            profile: default_profile(),
            source_digest: test_source_digest(package_id),
            dependency_artifacts: dependencies,
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
            derivation: ReviewableRustDerivation {
                name: format!("unit-{index}"),
                builder: "rustc".to_string(),
                system: "x86_64-linux".to_string(),
                args: Vec::new(),
                outputs: vec!["out".to_string()],
                env: BTreeMap::new(),
                inputs: Vec::new(),
                addressing_mode: "content-addressed".to_string(),
            },
            rustc_args_digest_blake3: test_source_digest(&format!("args-{index}")).value,
        }
    }

    fn test_dependency_artifact(package_id: &str, name: &str) -> RustDependencyArtifact {
        RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: name.to_string(),
            producer_unit_id: None,
            artifact: format!("artifact:{package_id}:{name}"),
        }
    }

    fn test_host_artifact(package_id: &str, target_kind: &str) -> RustHostArtifact {
        RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: package_id.to_string(),
            target_kind: target_kind.to_string(),
            producer_unit_id: None,
            artifact: format!("host-artifact:{package_id}:{target_kind}"),
            metadata_digest_blake3: None,
        }
    }

    fn test_unit_derivation_graph(derivations: Vec<RustUnitDerivationSummary>) -> UnitDerivationGraphSummary {
        UnitDerivationGraphSummary {
            derivation_count: derivations.len(),
            host_unit_count: derivations.iter().filter(|unit| unit.execution_kind == "host").count(),
            host_artifact_count: 0,
            ready: true,
            digest_blake3: "test-unit-graph".to_string(),
            derivations,
            blockers: Vec::new(),
        }
    }

    fn rustc_edition_arg(args: &[String]) -> &str {
        let edition_index = args.iter().position(|arg| arg == "--edition").expect("missing --edition flag");
        args.get(edition_index + 1).expect("missing edition value").as_str()
    }

    fn has_ordered_arg_pair(args: &[String], flag: &str, value: &str) -> bool {
        let mut previous_matches_flag = false;
        for arg in args {
            if previous_matches_flag && arg == value {
                return true;
            }
            previous_matches_flag = arg == flag;
        }
        false
    }

    fn rustc_metadata_arg(args: &[String]) -> Option<&str> {
        args.windows(2)
            .find(|window| window[0] == RUSTC_CODEGEN_OPTION_FLAG && window[1].starts_with(RUSTC_METADATA_ARG_PREFIX))
            .map(|window| window[1].as_str())
    }

    #[test]
    fn native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies() {
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("default".to_string(), vec!["std".to_string()]);
        feature_defs.insert("std".to_string(), vec!["alloc".to_string(), "dep:serde".to_string()]);
        feature_defs.insert("alloc".to_string(), Vec::new());
        let request = NativeFeatureResolutionRequest {
            feature_defs,
            optional_dependencies: BTreeSet::from(["serde".to_string()]),
            explicit_features: Vec::new(),
            all_features: false,
            no_default_features: false,
        };

        let resolution = resolve_native_features(request);

        assert!(resolution.blockers.is_empty(), "{:#?}", resolution.blockers);
        assert_eq!(resolution.selected_features, vec!["alloc".to_string(), "default".to_string(), "std".to_string()]);
        assert_eq!(resolution.activated_optional_dependencies, vec!["serde".to_string()]);
    }

    #[test]
    fn native_feature_resolver_models_dependency_feature_edges_and_rejects_unknown_entries() {
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("cli".to_string(), vec![
            "dep:clap".to_string(),
            "serde/derive".to_string(),
            "optional?/fast".to_string(),
        ]);
        feature_defs.insert("broken".to_string(), vec!["missing".to_string()]);
        let request = NativeFeatureResolutionRequest {
            feature_defs,
            optional_dependencies: BTreeSet::from(["clap".to_string()]),
            explicit_features: vec!["cli".to_string(), "broken".to_string()],
            all_features: false,
            no_default_features: true,
        };

        let resolution = resolve_native_features(request);

        assert_eq!(resolution.activated_optional_dependencies, vec!["clap".to_string()]);
        assert!(resolution.dependency_feature_edges.contains(&NativeDependencyFeatureEdge {
            dependency: "serde".to_string(),
            feature: "derive".to_string(),
            weak: false,
        }));
        assert!(resolution.dependency_feature_edges.contains(&NativeDependencyFeatureEdge {
            dependency: "optional".to_string(),
            feature: "fast".to_string(),
            weak: true,
        }));
        assert!(resolution.blockers.iter().any(|blocker| blocker.class == "unknown-feature-entry"));
    }

    #[test]
    fn native_feature_resolver_leaves_optional_dependency_unselected_without_feature() {
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("default".to_string(), Vec::new());
        let request = NativeFeatureResolutionRequest {
            feature_defs,
            optional_dependencies: BTreeSet::from(["serde".to_string()]),
            explicit_features: Vec::new(),
            all_features: false,
            no_default_features: false,
        };

        let resolution = resolve_native_features(request);

        assert!(resolution.blockers.is_empty(), "{:#?}", resolution.blockers);
        assert_eq!(resolution.selected_features, vec!["default".to_string()]);
        assert!(resolution.activated_optional_dependencies.is_empty());
    }

    #[test]
    fn native_feature_resolver_exposes_bare_optional_dependency_as_cfg_feature() {
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("derive".to_string(), vec!["serde_derive".to_string()]);
        let request = NativeFeatureResolutionRequest {
            feature_defs,
            optional_dependencies: BTreeSet::from(["serde_derive".to_string()]),
            explicit_features: vec!["derive".to_string()],
            all_features: false,
            no_default_features: true,
        };

        let resolution = resolve_native_features(request);

        assert!(resolution.blockers.is_empty(), "{:#?}", resolution.blockers);
        assert_eq!(resolution.selected_features, vec!["derive".to_string(), "serde_derive".to_string()]);
        assert_eq!(resolution.activated_optional_dependencies, vec!["serde_derive".to_string()]);
    }

    #[test]
    fn native_feature_resolver_blocks_malformed_feature_edges() {
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("default".to_string(), vec!["dep:".to_string(), "serde/".to_string()]);
        let request = NativeFeatureResolutionRequest {
            feature_defs,
            optional_dependencies: BTreeSet::from(["serde".to_string()]),
            explicit_features: Vec::new(),
            all_features: false,
            no_default_features: false,
        };

        let resolution = resolve_native_features(request);

        assert_eq!(resolution.selected_features, vec!["default".to_string()]);
        assert!(resolution.blockers.iter().any(|blocker| blocker.class == "unsupported-feature-entry"));
        assert!(resolution.blockers.iter().any(|blocker| blocker.message.contains("dep:")));
        assert!(resolution.blockers.iter().any(|blocker| blocker.message.contains("serde/")));
    }

    #[test]
    fn native_feature_role_resolver_keeps_normal_build_and_host_features_separate() {
        let mut normal_defs = BTreeMap::new();
        normal_defs.insert("default".to_string(), vec!["std".to_string()]);
        normal_defs.insert("std".to_string(), Vec::new());
        let mut build_defs = BTreeMap::new();
        build_defs.insert("build-default".to_string(), vec!["dep:cc".to_string()]);
        let mut host_defs = BTreeMap::new();
        host_defs.insert("proc-macro".to_string(), vec!["syn/derive".to_string()]);
        let request = NativeFeatureRoleResolutionRequest {
            normal: NativeFeatureResolutionRequest {
                feature_defs: normal_defs,
                optional_dependencies: BTreeSet::new(),
                explicit_features: Vec::new(),
                all_features: false,
                no_default_features: false,
            },
            build: NativeFeatureResolutionRequest {
                feature_defs: build_defs,
                optional_dependencies: BTreeSet::from(["cc".to_string()]),
                explicit_features: vec!["build-default".to_string()],
                all_features: false,
                no_default_features: true,
            },
            host: NativeFeatureResolutionRequest {
                feature_defs: host_defs,
                optional_dependencies: BTreeSet::new(),
                explicit_features: vec!["proc-macro".to_string()],
                all_features: false,
                no_default_features: true,
            },
        };

        let resolution = resolve_native_feature_roles(request);

        assert_eq!(resolution.normal.selected_features, vec!["default".to_string(), "std".to_string()]);
        assert_eq!(resolution.build.selected_features, vec!["build-default".to_string()]);
        assert_eq!(resolution.build.activated_optional_dependencies, vec!["cc".to_string()]);
        assert_eq!(resolution.host.selected_features, vec!["proc-macro".to_string()]);
        assert!(resolution.host.dependency_feature_edges.contains(&NativeDependencyFeatureEdge {
            dependency: "syn".to_string(),
            feature: "derive".to_string(),
            weak: false,
        }));
        assert!(!resolution.normal.selected_features.contains(&"build-default".to_string()));
        assert!(!resolution.build.selected_features.contains(&"std".to_string()));
    }

    #[test]
    fn rustc_metadata_disambiguator_distinguishes_same_crate_package_versions() {
        let source_digest = test_source_digest("same-crate-source");
        let crate_types = vec!["lib".to_string()];
        let features = vec!["std".to_string()];
        let first = rustc_unit_metadata_disambiguator(
            "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.2.17",
            "getrandom",
            "lib",
            "build",
            &source_digest,
            &features,
            &crate_types,
        );
        let second = rustc_unit_metadata_disambiguator(
            "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.4.2",
            "getrandom",
            "lib",
            "build",
            &source_digest,
            &features,
            &crate_types,
        );

        assert_eq!(first.len(), RUSTC_METADATA_HEX_CHARS);
        assert_eq!(second.len(), RUSTC_METADATA_HEX_CHARS);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(second.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }

    #[test]
    fn native_unit_derivation_emits_stable_metadata_disambiguator() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#bytes@1.11.1";
        let mut unit = test_native_rust_unit(package_id, "bytes", "lib", dir.path());
        unit.selected_features = vec!["serde".to_string(), "std".to_string()];
        let source_closure = test_source_closure_with_kind(package_id, "bytes", SourceKind::Registry);
        let first = native_unit_derivation(&unit, Vec::new(), &source_closure, &options(dir.path()));
        let second = native_unit_derivation(&unit, Vec::new(), &source_closure, &options(dir.path()));
        let expected = format!(
            "{RUSTC_METADATA_ARG_PREFIX}{}",
            rustc_unit_metadata_disambiguator(
                &unit.package_id,
                &unit.target_name,
                &unit.target_kind,
                &unit.mode,
                &unit.source_digest,
                &unit.selected_features,
                &unit.crate_types,
            )
        );

        assert_eq!(rustc_metadata_arg(&first.derivation.args), Some(expected.as_str()));
        assert_eq!(rustc_metadata_arg(&second.derivation.args), Some(expected.as_str()));
        assert_eq!(first.rustc_args_digest_blake3, second.rustc_args_digest_blake3);
    }

    #[test]
    fn native_unit_metadata_disambiguator_ignores_ambient_tool_roots() {
        let first_dir = TempDir::new().unwrap();
        let second_dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#bytes@1.11.1";
        let mut unit = test_native_rust_unit(package_id, "bytes", "lib", first_dir.path());
        unit.selected_features = vec!["serde".to_string(), "std".to_string()];
        let source_closure = test_source_closure_with_kind(package_id, "bytes", SourceKind::Registry);
        let mut first_options = options(first_dir.path());
        first_options.cargo = first_dir.path().join("cargo-a");
        first_options.rustc = first_dir.path().join("rustc-a");
        let mut second_options = options(second_dir.path());
        second_options.cargo = second_dir.path().join("cargo-b");
        second_options.rustc = second_dir.path().join("rustc-b");

        let first = native_unit_derivation(&unit, Vec::new(), &source_closure, &first_options);
        let second = native_unit_derivation(&unit, Vec::new(), &source_closure, &second_options);

        assert_eq!(rustc_metadata_arg(&first.derivation.args), rustc_metadata_arg(&second.derivation.args));
        assert_eq!(first.rustc_args_digest_blake3, second.rustc_args_digest_blake3);
    }

    #[test]
    fn native_host_unit_derivation_emits_metadata_disambiguator() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#spez@0.1.2";
        let unit = test_native_host_unit(package_id, "spez", "proc-macro", dir.path());
        let source_closure = test_source_closure_with_kind(package_id, "spez", SourceKind::Registry);
        let derivation = native_host_unit_derivation(&unit, &source_closure, &options(dir.path()));
        let expected = format!(
            "{RUSTC_METADATA_ARG_PREFIX}{}",
            rustc_unit_metadata_disambiguator(
                &unit.package_id,
                &unit.target_name,
                &unit.target_kind,
                &unit.mode,
                &unit.source_digest,
                &unit.selected_features,
                &unit.crate_types,
            )
        );

        assert_eq!(rustc_metadata_arg(&derivation.derivation.args), Some(expected.as_str()));
        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CODEGEN_OPTION_FLAG, expected.as_str()));
        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_EXTERN_FLAG, RUSTC_PROC_MACRO_EXTERN));
    }

    #[test]
    fn proc_macro_crate_type_only_overrides_lib_shaped_targets() {
        let lib_kind = vec!["lib".to_string()];
        let rlib_kind = vec!["rlib".to_string()];
        let bin_kind = vec!["bin".to_string()];
        let missing_kind = Vec::new();
        let proc_macro_crate_type = vec!["proc-macro".to_string()];
        let lib_crate_type = vec!["lib".to_string()];

        assert_eq!(classify_supported_cargo_target_kind(&lib_kind, &proc_macro_crate_type), Some("proc-macro"));
        assert_eq!(classify_supported_cargo_target_kind(&rlib_kind, &proc_macro_crate_type), Some("proc-macro"));
        assert_eq!(classify_supported_cargo_target_kind(&lib_kind, &lib_crate_type), Some("lib"));
        assert_eq!(classify_supported_cargo_target_kind(&bin_kind, &proc_macro_crate_type), Some("bin"));
        assert_eq!(classify_supported_cargo_target_kind(&missing_kind, &proc_macro_crate_type), None);
    }

    #[test]
    fn cargo_unit_derivation_promotes_proc_macro_crate_type_to_host_unit() {
        let dir = TempDir::new().unwrap();
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#spez@0.1.2";
        let lib_id = "registry+https://github.com/rust-lang/crates.io-index#ordinary@0.1.0";
        let packages = vec![
            test_native_package(proc_macro_id, "spez", "proc-macro", Vec::new()),
            test_native_package(lib_id, "ordinary", "lib", Vec::new()),
        ];
        let source_closure = test_source_closure(&packages);
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": proc_macro_id,
                    "target": {"name": "spez", "kind": ["lib"], "crate_types": ["proc-macro"], "src_path": "/test/spez/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": lib_id,
                    "target": {"name": "ordinary", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/ordinary/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                }
            ]
        });

        let graph = summarize_cargo_unit_derivation_graph(&unit_graph, &source_closure, &options(dir.path())).unwrap();

        assert!(graph.ready, "{:#?}", graph.blockers);
        assert_eq!(graph.host_unit_count, 1usize);
        assert_eq!(graph.host_artifact_count, 1usize);
        let proc_macro = graph
            .derivations
            .iter()
            .find(|unit| unit.package_id == proc_macro_id)
            .expect("proc-macro unit present");
        assert_eq!(proc_macro.target_kind, "proc-macro");
        assert_eq!(proc_macro.execution_kind, "host");
        assert!(has_ordered_arg_pair(&proc_macro.derivation.args, "--crate-type", "proc-macro"));
        assert!(!has_ordered_arg_pair(&proc_macro.derivation.args, "--crate-type", "lib"));
        let ordinary =
            graph.derivations.iter().find(|unit| unit.package_id == lib_id).expect("ordinary lib unit present");
        assert_eq!(ordinary.target_kind, "lib");
        assert_eq!(ordinary.execution_kind, "target");
        assert!(!ordinary.derivation.inputs.iter().any(|input| input.starts_with("host-artifact:")));
    }

    #[test]
    fn native_unit_derivation_adds_selected_feature_cfg_args() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://syn#syn@0.1.0".to_string();
        let mut unit = NativeRustUnitSummary {
            unit_id: rust_unit_id(0, &package_id, "syn", "lib", "build"),
            package_id: package_id.clone(),
            package_name: "syn".to_string(),
            package_links: None,
            package_root: normalize_path_string(&dir.path().join("syn")),
            cargo_package_env: test_cargo_package_env("syn", "0.1.0"),
            target_name: "syn".to_string(),
            target_kind: "lib".to_string(),
            crate_name: "syn".to_string(),
            source_path: dir.path().join("syn/src/lib.rs").display().to_string(),
            edition: "2021".to_string(),
            selected_features: vec!["parsing".to_string(), "visit-mut".to_string()],
            crate_types: vec!["lib".to_string()],
            mode: "build".to_string(),
            profile: DEFAULT_CARGO_PROFILE.to_string(),
            source_digest: test_source_digest(&package_id),
            dependency_artifacts: Vec::new(),
        };
        let source_closure = SourceClosureSummary {
            source_count: 1,
            ready: true,
            digest_blake3: "test-source-closure".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
        };

        let feature_derivation = native_unit_derivation(&unit, Vec::new(), &source_closure, &options(dir.path()));
        unit.selected_features.clear();
        let default_derivation = native_unit_derivation(&unit, Vec::new(), &source_closure, &options(dir.path()));

        assert!(has_ordered_arg_pair(&feature_derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"parsing\""));
        assert!(has_ordered_arg_pair(&feature_derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"visit-mut\"",));
        assert!(!has_ordered_arg_pair(&default_derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"parsing\""));
    }

    #[test]
    fn native_unit_derivation_emits_resolved_feature_closure_cfg_args() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://featureful#featureful@0.1.0";
        let mut feature_defs = BTreeMap::new();
        feature_defs.insert("default".to_string(), vec!["derive".to_string()]);
        feature_defs.insert("derive".to_string(), vec!["printing".to_string()]);
        feature_defs.insert("printing".to_string(), Vec::new());
        let mut plan_options = options(dir.path());
        plan_options.features.clear();
        plan_options.no_default_features = false;
        let selected_features = native_selected_features(&plan_options, &feature_defs, &BTreeSet::new());
        let mut unit = test_native_rust_unit(package_id, "featureful", "lib", dir.path());
        unit.selected_features = selected_features;
        let source_closure = test_source_closure_with_kind(package_id, "featureful", SourceKind::Path);

        let derivation = native_unit_derivation(&unit, Vec::new(), &source_closure, &plan_options);

        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"default\""));
        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"derive\""));
        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CFG_FLAG, "feature=\"printing\""));
    }

    #[test]
    fn native_unit_derivation_caps_lints_for_registry_and_git_sources() {
        let dir = TempDir::new().unwrap();
        let registry_id = "registry+https://github.com/rust-lang/crates.io-index#derive_builder_core@0.20.2";
        let git_id = "git+https://example.test/repo#git_dep@0.1.0";
        let mut unit = test_native_rust_unit(registry_id, "derive_builder_core", "lib", dir.path());
        let registry_source_closure =
            test_source_closure_with_kind(registry_id, "derive_builder_core", SourceKind::Registry);
        let registry_derivation =
            native_unit_derivation(&unit, Vec::new(), &registry_source_closure, &options(dir.path()));
        unit.package_id = git_id.to_string();
        unit.source_digest = test_source_digest(git_id);
        let git_source_closure = test_source_closure_with_kind(git_id, "git_dep", SourceKind::Git);
        let git_derivation = native_unit_derivation(&unit, Vec::new(), &git_source_closure, &options(dir.path()));

        assert!(has_ordered_arg_pair(
            &registry_derivation.derivation.args,
            RUSTC_CAP_LINTS_FLAG,
            RUSTC_CAP_LINTS_ALLOW,
        ));
        assert!(has_ordered_arg_pair(&git_derivation.derivation.args, RUSTC_CAP_LINTS_FLAG, RUSTC_CAP_LINTS_ALLOW,));
    }

    #[test]
    fn native_unit_derivation_leaves_path_sources_uncapped() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://local#local@0.1.0";
        let unit = test_native_rust_unit(package_id, "local", "lib", dir.path());
        let source_closure = test_source_closure_with_kind(package_id, "local", SourceKind::Path);

        let derivation = native_unit_derivation(&unit, Vec::new(), &source_closure, &options(dir.path()));

        assert!(!has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CAP_LINTS_FLAG, RUSTC_CAP_LINTS_ALLOW,));
    }

    #[test]
    fn native_host_derivation_caps_lints_for_registry_source() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11";
        let unit = test_native_host_unit(package_id, "darling_macro", "proc-macro", dir.path());
        let source_closure = test_source_closure_with_kind(package_id, "darling_macro", SourceKind::Registry);

        let derivation = native_host_unit_derivation(&unit, &source_closure, &options(dir.path()));

        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_CAP_LINTS_FLAG, RUSTC_CAP_LINTS_ALLOW,));
        assert!(has_ordered_arg_pair(&derivation.derivation.args, RUSTC_EXTERN_FLAG, RUSTC_PROC_MACRO_EXTERN,));
    }

    #[test]
    fn native_host_derivation_adds_compiler_proc_macro_extern() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://mac#mac@0.1.0".to_string();
        let mut unit = NativeHostUnitSummary {
            unit_id: rust_unit_id(0, &package_id, "mac", "proc-macro", "build"),
            package_id: package_id.clone(),
            package_name: "mac".to_string(),
            package_links: None,
            package_root: normalize_path_string(&dir.path().join("mac")),
            cargo_package_env: test_cargo_package_env("mac", "0.1.0"),
            target_name: "mac".to_string(),
            target_kind: "proc-macro".to_string(),
            crate_name: "mac".to_string(),
            source_path: dir.path().join("mac/src/lib.rs").display().to_string(),
            edition: "2021".to_string(),
            selected_features: Vec::new(),
            crate_types: vec!["proc-macro".to_string()],
            mode: "build".to_string(),
            profile: DEFAULT_CARGO_PROFILE.to_string(),
            source_digest: test_source_digest(&package_id),
            artifact: test_host_artifact(&package_id, "proc-macro"),
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
        };
        let source_closure = SourceClosureSummary {
            source_count: 1,
            ready: true,
            digest_blake3: "test-source-closure".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
        };

        let proc_macro_derivation = native_host_unit_derivation(&unit, &source_closure, &options(dir.path()));
        unit.target_kind = "custom-build".to_string();
        unit.crate_types = vec!["bin".to_string()];
        let custom_build_derivation = native_host_unit_derivation(&unit, &source_closure, &options(dir.path()));

        assert!(has_ordered_arg_pair(
            &proc_macro_derivation.derivation.args,
            RUSTC_EXTERN_FLAG,
            RUSTC_PROC_MACRO_EXTERN,
        ));
        assert!(!has_ordered_arg_pair(
            &custom_build_derivation.derivation.args,
            RUSTC_EXTERN_FLAG,
            RUSTC_PROC_MACRO_EXTERN,
        ));
    }

    #[test]
    fn native_host_dependencies_use_normal_deps_only_for_proc_macro_units() {
        let dep_id = "path+file://proc-macro2#proc-macro2@0.1.0";
        let macro_id = "path+file://async-stream-impl#async-stream-impl@0.1.0";
        let dep_package = test_native_package(dep_id, "proc-macro2", "lib", Vec::new());
        let dependency = NativePathDependencySummary {
            name: "proc-macro2".to_string(),
            manifest_path: dep_package.manifest_path.clone(),
        };
        let macro_package = test_native_package(macro_id, "async-stream-impl", "proc-macro", vec![dependency]);
        let packages = vec![macro_package.clone(), dep_package.clone()];
        let packages_by_id = packages.iter().map(|package| (package.package_id.clone(), package)).collect();
        let source_closure = test_source_closure(&packages);
        let selected_artifacts = BTreeSet::from([RustDependencyArtifact {
            package_id: dep_id.to_string(),
            name: "proc_macro2".to_string(),
            producer_unit_id: Some("cargo-oracle-producer".to_string()),
            artifact: format!("artifact:{dep_id}:proc_macro2"),
        }]);
        let mut blockers = Vec::new();

        let proc_macro_deps = native_host_unit_dependency_artifacts(
            &macro_package,
            "proc-macro",
            Some(&selected_artifacts),
            &packages_by_id,
            &packages,
            &source_closure,
            &mut blockers,
        );
        let custom_build_deps = native_host_unit_dependency_artifacts(
            &macro_package,
            "custom-build",
            None,
            &packages_by_id,
            &packages,
            &source_closure,
            &mut blockers,
        );

        assert!(blockers.is_empty(), "{blockers:#?}");
        assert_eq!(proc_macro_deps.len(), 1usize);
        assert_eq!(proc_macro_deps[0].package_id, dep_id);
        assert_eq!(proc_macro_deps[0].name, "proc_macro2");
        assert_eq!(proc_macro_deps[0].producer_unit_id, None);
        assert!(custom_build_deps.is_empty());
    }

    #[test]
    fn native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units() {
        let package_id = "path+file://rustversion#rustversion@1.0.0";
        let mut package = test_native_package(package_id, "rustversion", "proc-macro", Vec::new());
        package.targets.push(NativeTargetPlanningSummary {
            name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            kind: "custom-build".to_string(),
            crate_name: rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
            source_path: "/test/rustversion/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let packages = vec![package.clone()];
        let packages_by_id = packages.iter().map(|package| (package.package_id.clone(), package)).collect();
        let source_closure = test_source_closure(&packages);
        let selected_artifacts = BTreeSet::from([test_dependency_artifact(
            package_id,
            &rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
        )]);
        let mut blockers = Vec::new();

        let proc_macro_deps = native_host_unit_dependency_artifacts(
            &package,
            "proc-macro",
            Some(&selected_artifacts),
            &packages_by_id,
            &packages,
            &source_closure,
            &mut blockers,
        );

        assert!(blockers.is_empty(), "{blockers:#?}");
        assert!(proc_macro_deps.is_empty());
    }

    #[test]
    fn native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0";
        let selected_macro_id = "path+file://selected-macro#selected-macro@0.1.0";
        let unselected_macro_id = "path+file://jiff-static#jiff-static@0.2.23";
        let quote_id = "path+file://quote#quote@1.0.0";
        let mut app = test_native_package(app_id, "app", "lib", Vec::new());
        let selected_macro = test_native_package(selected_macro_id, "selected-macro", "proc-macro", Vec::new());
        let unselected_macro =
            test_native_package(unselected_macro_id, "jiff-static", "proc-macro", vec![NativePathDependencySummary {
                name: "quote".to_string(),
                manifest_path: "/test/quote/Cargo.toml".to_string(),
            }]);
        let quote = test_native_package(quote_id, "quote", "lib", Vec::new());
        app.path_dependencies.push(NativePathDependencySummary {
            name: "selected-macro".to_string(),
            manifest_path: selected_macro.manifest_path.clone(),
        });
        let package_planning =
            test_package_planning(vec![app.clone(), selected_macro.clone(), unselected_macro, quote]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": quote_id,
                    "target": {"name": "quote", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/quote/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": selected_macro_id,
                    "target": {"name": "selected_macro", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/selected-macro/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": quote_id, "extern_crate_name": "quote"}]
                },
                {
                    "pkg_id": app_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": selected_macro_id, "extern_crate_name": "selected_macro"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        assert!(native_hosts.host_units.iter().any(|unit| unit.package_id == selected_macro_id));
        assert!(!native_hosts.host_units.iter().any(|unit| unit.package_id == unselected_macro_id));
        let selected_host = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.package_id == selected_macro_id)
            .expect("selected host unit planned");
        assert_eq!(selected_host.dependency_artifacts, vec![test_dependency_artifact(quote_id, "quote")]);
        let app_consumer = native_hosts
            .target_consumers
            .iter()
            .find(|consumer| consumer.package_id == app_id)
            .expect("app consumes selected proc macro");
        assert_eq!(app_consumer.consumed_host_artifacts.len(), 1usize);
        assert_eq!(app_consumer.consumed_host_artifacts[0].package_id, selected_macro_id);
    }

    #[test]
    fn native_host_planning_selects_lib_kind_proc_macro_crate_type() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0";
        let proc_macro_id = "path+file://spez#spez@0.1.2";
        let mut app = test_native_package(app_id, "app", "lib", Vec::new());
        let proc_macro = test_native_package(proc_macro_id, "spez", "proc-macro", Vec::new());
        app.path_dependencies.push(NativePathDependencySummary {
            name: "spez".to_string(),
            manifest_path: proc_macro.manifest_path.clone(),
        });
        let package_planning = test_package_planning(vec![app, proc_macro]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": proc_macro_id,
                    "target": {"name": "spez", "kind": ["lib"], "crate_types": ["proc-macro"], "src_path": "/test/spez/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": app_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": proc_macro_id, "extern_crate_name": "spez"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        assert_eq!(native_hosts.host_units.len(), 1usize);
        assert_eq!(native_hosts.host_units[0].package_id, proc_macro_id);
        assert_eq!(native_hosts.host_units[0].target_kind, "proc-macro");
        assert_eq!(native_hosts.target_consumers.len(), 1usize);
        assert_eq!(native_hosts.target_consumers[0].package_id, app_id);
        assert_eq!(native_hosts.target_consumers[0].consumed_host_artifacts.len(), 1usize);
        assert_eq!(native_hosts.target_consumers[0].consumed_host_artifacts[0].package_id, proc_macro_id);
    }

    #[test]
    fn native_host_planning_keeps_selected_same_package_build_script_for_proc_macro() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://rustversion#rustversion@1.0.0";
        let mut package = test_native_package(package_id, "rustversion", "proc-macro", Vec::new());
        package.targets.push(NativeTargetPlanningSummary {
            name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            kind: "custom-build".to_string(),
            crate_name: rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
            source_path: "/test/rustversion/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let package_planning = test_package_planning(vec![package]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": package_id,
                    "target": {"name": "build-script-main", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/rustversion/build.rs", "edition": "2021"},
                    "mode": "run-custom-build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": package_id,
                    "target": {"name": "rustversion", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/rustversion/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": package_id, "extern_crate_name": "build_script_build"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        assert_eq!(native_hosts.host_units.len(), 2usize);
        let proc_macro = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.target_kind == "proc-macro")
            .expect("selected proc macro planned");
        assert!(proc_macro.dependency_artifacts.is_empty());
        assert_eq!(proc_macro.consumed_host_artifacts.len(), 1usize);
        assert_eq!(proc_macro.consumed_host_artifacts[0].target_kind, "custom-build");
    }

    #[test]
    fn native_host_planning_links_proc_macro_dependency_proc_macros_without_cargo_edges() {
        let dir = TempDir::new().unwrap();
        let macro_id = "path+file://strum_macros#strum_macros@0.1.0";
        let rustversion_id = "path+file://rustversion#rustversion@1.0.0";
        let rustversion = test_native_package(rustversion_id, "rustversion", "proc-macro", Vec::new());
        let mut macro_package =
            test_native_package(macro_id, "strum_macros", "proc-macro", vec![NativePathDependencySummary {
                name: "rustversion".to_string(),
                manifest_path: rustversion.manifest_path.clone(),
            }]);
        macro_package.targets[0].name = "strum_macros".to_string();
        macro_package.targets[0].crate_name = "strum_macros".to_string();
        let package_planning = test_package_planning(vec![macro_package, rustversion]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": rustversion_id,
                    "target": {"name": "rustversion", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/rustversion/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": macro_id,
                    "target": {"name": "strum_macros", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/strum_macros/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                }
            ]
        });
        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        let macro_unit = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.package_id == macro_id)
            .expect("macro host unit planned");
        assert_eq!(macro_unit.consumed_host_artifacts.len(), 1usize);
        assert_eq!(macro_unit.consumed_host_artifacts[0].package_id, rustversion_id);
        assert_eq!(macro_unit.consumed_host_artifacts[0].target_kind, "proc-macro");
    }

    #[test]
    fn native_host_planning_preserves_duplicate_selected_proc_macro_unit_ids() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0";
        let proc_macro_id = "path+file://dupe_macro#dupe_macro@0.1.0";
        let mut app = test_native_package(app_id, "app", "lib", Vec::new());
        let proc_macro = test_native_package(proc_macro_id, "dupe_macro", "proc-macro", Vec::new());
        app.path_dependencies.push(NativePathDependencySummary {
            name: "dupe_macro".to_string(),
            manifest_path: proc_macro.manifest_path.clone(),
        });
        let package_planning = test_package_planning(vec![app, proc_macro]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": proc_macro_id,
                    "target": {"name": "dupe_macro", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/dupe_macro/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["first"],
                    "deps": []
                },
                {
                    "pkg_id": proc_macro_id,
                    "target": {"name": "dupe_macro", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/dupe_macro/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["second"],
                    "deps": []
                },
                {
                    "pkg_id": app_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"index": 1, "extern_crate_name": "dupe_macro"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        let host_unit_ids = native_hosts.host_units.iter().map(|unit| unit.unit_id.clone()).collect::<Vec<_>>();
        assert_eq!(host_unit_ids.len(), 2usize);
        assert_ne!(host_unit_ids[0], host_unit_ids[1]);
        assert!(host_unit_ids.contains(&rust_unit_id(0, proc_macro_id, "dupe_macro", "proc-macro", "build")));
        assert!(host_unit_ids.contains(&rust_unit_id(1, proc_macro_id, "dupe_macro", "proc-macro", "build")));
        let app_consumer = native_hosts
            .target_consumers
            .iter()
            .find(|consumer| consumer.package_id == app_id)
            .expect("app consumes selected duplicate proc macro");
        assert_eq!(app_consumer.consumed_host_artifacts.len(), 1usize);
        assert_eq!(
            app_consumer.consumed_host_artifacts[0].producer_unit_id,
            Some(rust_unit_id(1, proc_macro_id, "dupe_macro", "proc-macro", "build"))
        );
    }

    #[test]
    fn native_host_planning_preserves_duplicate_selected_custom_build_unit_ids() {
        let dir = TempDir::new().unwrap();
        let helper_id = "path+file://build-helper#build-helper@0.1.0";
        let mut helper = test_native_package(helper_id, "build-helper", "proc-macro", Vec::new());
        helper.targets.push(NativeTargetPlanningSummary {
            name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            kind: "custom-build".to_string(),
            crate_name: rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
            source_path: "/test/build-helper/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let package_planning = test_package_planning(vec![helper]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper/build.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["first"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper/build.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["second"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-helper", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/build-helper/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"index": 1, "extern_crate_name": "build_script_build"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        let custom_build_ids = native_hosts
            .host_units
            .iter()
            .filter(|unit| unit.target_kind == "custom-build")
            .map(|unit| unit.unit_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(custom_build_ids.len(), 2usize);
        assert_ne!(custom_build_ids[0], custom_build_ids[1]);
        assert!(custom_build_ids.contains(&rust_unit_id(
            0,
            helper_id,
            BUILD_SCRIPT_TARGET_NAME,
            "custom-build",
            "build"
        )));
        assert!(custom_build_ids.contains(&rust_unit_id(
            1,
            helper_id,
            BUILD_SCRIPT_TARGET_NAME,
            "custom-build",
            "build"
        )));
        let proc_macro = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.target_kind == "proc-macro")
            .expect("proc macro host unit planned");
        assert_eq!(proc_macro.consumed_host_artifacts.len(), 1usize);
        assert_eq!(
            proc_macro.consumed_host_artifacts[0].producer_unit_id,
            Some(rust_unit_id(1, helper_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build"))
        );
    }

    #[test]
    fn native_host_planning_binds_run_custom_build_alias_to_exact_unique_build_unit() {
        const BUILD_INDEX: usize = 0;
        const RUN_BUILD_INDEX: usize = 1;
        const PROC_MACRO_INDEX: usize = 2;
        let dir = TempDir::new().unwrap();
        let helper_id = "path+file://build-helper-exact#build-helper-exact@0.1.0";
        let mut helper = test_native_package(helper_id, "build-helper-exact", "proc-macro", Vec::new());
        helper.targets.push(NativeTargetPlanningSummary {
            name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            kind: "custom-build".to_string(),
            crate_name: rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
            source_path: "/test/build-helper-exact/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let package_planning = test_package_planning(vec![helper]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper-exact/build.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["only"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper-exact/build.rs", "edition": "2021"},
                    "mode": "run-custom-build",
                    "features": ["only"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-helper-exact", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/build-helper-exact/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"index": RUN_BUILD_INDEX, "extern_crate_name": "build_script_build"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        let exact_build_unit_id =
            rust_unit_id(BUILD_INDEX, helper_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        let run_alias_unit_id =
            rust_unit_id(RUN_BUILD_INDEX, helper_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "run-custom-build");
        let exact_proc_macro_unit_id =
            rust_unit_id(PROC_MACRO_INDEX, helper_id, "build-helper-exact", "proc-macro", "build");
        let units = unit_graph.get("units").and_then(Value::as_array).expect("unit graph has units");
        assert_eq!(
            host_dependency_producer_unit_id(&serde_json::json!({"index": RUN_BUILD_INDEX}), units),
            Some(exact_build_unit_id.clone())
        );
        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        let custom_build_ids = native_hosts
            .host_units
            .iter()
            .filter(|unit| unit.target_kind == "custom-build")
            .map(|unit| unit.unit_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(custom_build_ids, vec![exact_build_unit_id.clone()]);
        assert!(!custom_build_ids.contains(&run_alias_unit_id));
        let proc_macro = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.unit_id == exact_proc_macro_unit_id)
            .expect("proc macro host unit planned");
        assert_eq!(proc_macro.consumed_host_artifacts.len(), 1usize);
        assert_eq!(proc_macro.consumed_host_artifacts[0].target_kind, "custom-build");
        assert_eq!(proc_macro.consumed_host_artifacts[0].producer_unit_id, Some(exact_build_unit_id));
    }

    #[test]
    fn native_host_planning_does_not_bind_run_custom_build_alias_to_first_duplicate() {
        const FIRST_BUILD_INDEX: usize = 0;
        const SECOND_BUILD_INDEX: usize = 1;
        const RUN_BUILD_INDEX: usize = 2;
        const EXPECTED_DUPLICATE_PRODUCER_COUNT: usize = 2;
        let dir = TempDir::new().unwrap();
        let helper_id = "path+file://build-helper#build-helper@0.1.0";
        let mut helper = test_native_package(helper_id, "build-helper", "proc-macro", Vec::new());
        helper.targets.push(NativeTargetPlanningSummary {
            name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            kind: "custom-build".to_string(),
            crate_name: rust_crate_name(BUILD_SCRIPT_TARGET_NAME),
            source_path: "/test/build-helper/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let package_planning = test_package_planning(vec![helper]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper/build.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["first"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper/build.rs", "edition": "2021"},
                    "mode": "build",
                    "features": ["second"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": "/test/build-helper/build.rs", "edition": "2021"},
                    "mode": "run-custom-build",
                    "features": ["second"],
                    "deps": []
                },
                {
                    "pkg_id": helper_id,
                    "target": {"name": "build-helper", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": "/test/build-helper/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"index": RUN_BUILD_INDEX, "extern_crate_name": "build_script_build"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();

        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        let proc_macro = native_hosts
            .host_units
            .iter()
            .find(|unit| unit.target_kind == "proc-macro")
            .expect("proc macro host unit planned");
        let producer_unit_ids = proc_macro
            .consumed_host_artifacts
            .iter()
            .filter_map(|artifact| artifact.producer_unit_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(producer_unit_ids.len(), EXPECTED_DUPLICATE_PRODUCER_COUNT);
        assert!(producer_unit_ids.contains(&rust_unit_id(
            FIRST_BUILD_INDEX,
            helper_id,
            BUILD_SCRIPT_TARGET_NAME,
            "custom-build",
            "build"
        )));
        assert!(producer_unit_ids.contains(&rust_unit_id(
            SECOND_BUILD_INDEX,
            helper_id,
            BUILD_SCRIPT_TARGET_NAME,
            "custom-build",
            "build"
        )));
    }

    #[test]
    fn native_cargo_package_env_sets_version_components_and_empty_defaults() {
        let manifest = toml::from_str::<NativeManifest>(
            r#"
[package]
name = "aws-lc-sys"
version = "0.39.1-alpha.2+build.5"
links = "aws_lc_0_39_1"
authors = ["AWS", "Crypto"]
repository = "https://github.com/aws/aws-lc-rs"
"#,
        )
        .unwrap();
        let package = manifest.package.as_ref().unwrap();

        let env = native_cargo_package_env(package, &NativeWorkspacePackage::default(), "0.39.1-alpha.2+build.5");

        assert_eq!(env.get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV).unwrap(), "aws-lc-sys");
        assert_eq!(env.get(BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV).unwrap(), "aws_lc_0_39_1");
        assert_eq!(env.get(CARGO_PKG_VERSION_ENV).unwrap(), "0.39.1-alpha.2+build.5");
        assert_eq!(env.get(CARGO_PKG_VERSION_MAJOR_ENV).unwrap(), "0");
        assert_eq!(env.get(CARGO_PKG_VERSION_MINOR_ENV).unwrap(), "39");
        assert_eq!(env.get(CARGO_PKG_VERSION_PATCH_ENV).unwrap(), "1");
        assert_eq!(env.get(CARGO_PKG_VERSION_PRE_ENV).unwrap(), "alpha.2");
        assert_eq!(env.get(CARGO_PKG_AUTHORS_ENV).unwrap(), "AWS:Crypto");
        assert_eq!(env.get(CARGO_PKG_REPOSITORY_ENV).unwrap(), "https://github.com/aws/aws-lc-rs");
        assert_eq!(env.get(CARGO_PKG_DESCRIPTION_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_PKG_HOMEPAGE_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_PKG_LICENSE_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_PKG_LICENSE_FILE_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_PKG_README_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_PKG_RUST_VERSION_ENV).unwrap(), "");
    }

    #[test]
    fn native_cargo_package_env_inherits_optional_workspace_metadata() {
        let member_manifest = toml::from_str::<NativeManifest>(
            r#"
[package]
name = "member"
version = "0.1.0"
authors.workspace = true
description.workspace = true
homepage.workspace = true
license.workspace = true
license-file.workspace = true
readme.workspace = true
repository.workspace = true
rust-version.workspace = true
"#,
        )
        .unwrap();
        let workspace_manifest = toml::from_str::<NativeManifest>(
            r#"
[workspace]
members = ["member"]

[workspace.package]
authors = ["Workspace", "Team"]
description = "workspace description"
homepage = "https://example.invalid/home"
license = "Apache-2.0"
license-file = "LICENSE.md"
readme = "README.md"
repository = "https://example.invalid/repo"
rust-version = "1.80"
"#,
        )
        .unwrap();
        let package = member_manifest.package.as_ref().unwrap();
        let workspace_package = &workspace_manifest.workspace.as_ref().unwrap().package;

        let env = native_cargo_package_env(package, workspace_package, "0.1.0");

        assert_eq!(env.get(CARGO_PKG_AUTHORS_ENV).unwrap(), "Workspace:Team");
        assert_eq!(env.get(CARGO_PKG_DESCRIPTION_ENV).unwrap(), "workspace description");
        assert_eq!(env.get(CARGO_PKG_HOMEPAGE_ENV).unwrap(), "https://example.invalid/home");
        assert_eq!(env.get(CARGO_PKG_LICENSE_ENV).unwrap(), "Apache-2.0");
        assert_eq!(env.get(CARGO_PKG_LICENSE_FILE_ENV).unwrap(), "LICENSE.md");
        assert_eq!(env.get(CARGO_PKG_README_ENV).unwrap(), "README.md");
        assert_eq!(env.get(CARGO_PKG_REPOSITORY_ENV).unwrap(), "https://example.invalid/repo");
        assert_eq!(env.get(CARGO_PKG_RUST_VERSION_ENV).unwrap(), "1.80");
        assert_eq!(env.get(BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV).unwrap(), "");
    }

    #[test]
    fn native_host_derivation_carries_manifest_package_name_for_build_script_env() {
        let dir = TempDir::new().unwrap();
        let package_name = "hyphen-pkg".to_string();
        let package_id = "path+file://hyphen-pkg#hyphen-pkg@0.1.0".to_string();
        let source_path = dir.path().join("hyphen-pkg/build.rs");
        let mut unit = NativeHostUnitSummary {
            unit_id: rust_unit_id(0, &package_id, "build-script-build", "custom-build", "build"),
            package_id: package_id.clone(),
            package_name: package_name.clone(),
            package_links: None,
            package_root: normalize_path_string(&dir.path().join("hyphen-pkg")),
            cargo_package_env: test_cargo_package_env(&package_name, "1.2.3-alpha.1+build.5"),
            target_name: "build-script-build".to_string(),
            target_kind: "custom-build".to_string(),
            crate_name: "build_script_build".to_string(),
            source_path: source_path.display().to_string(),
            edition: "2021".to_string(),
            selected_features: Vec::new(),
            crate_types: vec!["bin".to_string()],
            mode: "build".to_string(),
            profile: DEFAULT_CARGO_PROFILE.to_string(),
            source_digest: test_source_digest(&package_id),
            artifact: test_host_artifact(&package_id, "custom-build"),
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: Some(build_script_metadata_summary(&package_id, "build-script-build")),
        };
        unit.cargo_package_env
            .insert(BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV.to_string(), "hyphen_links".to_string());
        let source_closure = SourceClosureSummary {
            source_count: 1,
            ready: true,
            digest_blake3: "test-source-closure".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
        };

        let derivation = native_host_unit_derivation(&unit, &source_closure, &options(dir.path()));

        assert_eq!(derivation.derivation.env.get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV).unwrap(), &package_name);
        assert_eq!(derivation.derivation.env.get(CARGO_PKG_VERSION_ENV).unwrap(), "1.2.3-alpha.1+build.5");
        assert_eq!(derivation.derivation.env.get(CARGO_PKG_VERSION_PRE_ENV).unwrap(), "alpha.1");
        assert_eq!(derivation.derivation.env.get(BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV).unwrap(), "hyphen_links");
        assert_eq!(
            derivation.derivation.env.get(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV).unwrap(),
            &normalize_path_string(&dir.path().join("hyphen-pkg"))
        );
        let child_env = build_script_child_env(
            &derivation,
            &RustUnitExecutionOptions {
                rustc: dir.path().join("rustc"),
                output_root: dir.path().join("unit-out"),
            },
            &dir.path().join("out"),
            None,
        );
        assert_eq!(child_env.get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV).unwrap(), &package_name);
        assert_eq!(child_env.get(CARGO_PKG_VERSION_ENV).unwrap(), "1.2.3-alpha.1+build.5");
        assert_eq!(child_env.get(CARGO_PKG_VERSION_PRE_ENV).unwrap(), "alpha.1");
        assert_eq!(child_env.get(BUILD_SCRIPT_CARGO_MANIFEST_LINKS_ENV).unwrap(), "hyphen_links");
    }

    #[test]
    fn build_script_target_cfg_env_derives_x86_64_linux_values() {
        let env = build_script_target_cfg_env("x86_64-unknown-linux-gnu");

        assert_eq!(env.get(CARGO_CFG_TARGET_ARCH_ENV).unwrap(), "x86_64");
        assert_eq!(env.get(CARGO_CFG_TARGET_VENDOR_ENV).unwrap(), "unknown");
        assert_eq!(env.get(CARGO_CFG_TARGET_OS_ENV).unwrap(), "linux");
        assert_eq!(env.get(CARGO_CFG_TARGET_ENV_ENV).unwrap(), "gnu");
        assert_eq!(env.get(CARGO_CFG_TARGET_FAMILY_ENV).unwrap(), "unix");
        assert_eq!(env.get(CARGO_CFG_TARGET_ENDIAN_ENV).unwrap(), "little");
        assert_eq!(env.get(CARGO_CFG_TARGET_POINTER_WIDTH_ENV).unwrap(), "64");
        assert_eq!(env.get(CARGO_CFG_TARGET_FEATURE_ENV).unwrap(), "fxsr,sse,sse2,x87");
        assert_eq!(env.get(CARGO_CFG_UNIX_ENV).unwrap(), "");
        assert!(!env.contains_key(CARGO_CFG_WINDOWS_ENV));
    }

    #[test]
    fn build_script_target_cfg_env_uses_empty_env_for_wasm_unknown() {
        let env = build_script_target_cfg_env("wasm32-unknown-unknown");

        assert_eq!(env.get(CARGO_CFG_TARGET_ARCH_ENV).unwrap(), "wasm32");
        assert_eq!(env.get(CARGO_CFG_TARGET_VENDOR_ENV).unwrap(), "unknown");
        assert_eq!(env.get(CARGO_CFG_TARGET_OS_ENV).unwrap(), "unknown");
        assert_eq!(env.get(CARGO_CFG_TARGET_ENV_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_CFG_TARGET_FAMILY_ENV).unwrap(), "wasm");
        assert_eq!(env.get(CARGO_CFG_TARGET_POINTER_WIDTH_ENV).unwrap(), "32");
        assert!(!env.contains_key(CARGO_CFG_UNIX_ENV));
        assert!(!env.contains_key(CARGO_CFG_WINDOWS_ENV));
    }

    #[test]
    fn build_script_profile_env_derives_dev_and_release_defaults() {
        let dev_env = build_script_profile_env(DEFAULT_CARGO_PROFILE);
        let release_env = build_script_profile_env(CARGO_PROFILE_RELEASE);

        assert_eq!(dev_env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), CARGO_OPT_LEVEL_DEBUG);
        assert_eq!(dev_env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), CARGO_DEBUG_TRUE);
        assert_eq!(dev_env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), MANTLE_DETERMINISTIC_NUM_JOBS);
        assert_eq!(release_env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), CARGO_OPT_LEVEL_RELEASE);
        assert_eq!(release_env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), CARGO_DEBUG_FALSE);
        assert_eq!(release_env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), MANTLE_DETERMINISTIC_NUM_JOBS);
    }

    #[test]
    fn build_script_child_env_ignores_ambient_profile_env() {
        let output = Command::new(std::env::current_exe().expect("test binary path is available"))
            .arg("rust_plan::tests::build_script_profile_env_child_ignores_ambient_process_env_probe")
            .arg("--exact")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(PROFILE_ENV_CHILD_PROBE_ENV, PROFILE_ENV_CHILD_PROBE_VALUE)
            .env(BUILD_SCRIPT_OPT_LEVEL_ENV, AMBIENT_OPT_LEVEL_VALUE)
            .env(BUILD_SCRIPT_DEBUG_ENV, AMBIENT_DEBUG_VALUE)
            .env(BUILD_SCRIPT_NUM_JOBS_ENV, AMBIENT_NUM_JOBS_VALUE)
            .output()
            .expect("child test process runs");

        assert!(
            output.status.success(),
            "child stdout:\n{}\nchild stderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn build_script_profile_env_child_ignores_ambient_process_env_probe() {
        if std::env::var(PROFILE_ENV_CHILD_PROBE_ENV).as_deref() != Ok(PROFILE_ENV_CHILD_PROBE_VALUE) {
            return;
        }
        assert_eq!(std::env::var(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), AMBIENT_OPT_LEVEL_VALUE);
        assert_eq!(std::env::var(BUILD_SCRIPT_DEBUG_ENV).unwrap(), AMBIENT_DEBUG_VALUE);
        assert_eq!(std::env::var(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), AMBIENT_NUM_JOBS_VALUE);

        let dir = TempDir::new().unwrap();
        let out_dir = dir.path().join("out");
        let mut unit = test_rust_derivation(0, "path+file://package#package@0.1.0", "custom-build", "host", Vec::new());
        unit.profile = CARGO_PROFILE_RELEASE.to_string();
        let options = RustUnitExecutionOptions {
            rustc: dir.path().join("rustc"),
            output_root: dir.path().join("unit-out"),
        };

        let env = build_script_child_env(&unit, &options, &out_dir, None);

        assert_eq!(env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), CARGO_OPT_LEVEL_RELEASE);
        assert_eq!(env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), CARGO_DEBUG_FALSE);
        assert_eq!(env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), MANTLE_DETERMINISTIC_NUM_JOBS);
        assert_ne!(env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), AMBIENT_OPT_LEVEL_VALUE);
        assert_ne!(env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), AMBIENT_DEBUG_VALUE);
        assert_ne!(env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), AMBIENT_NUM_JOBS_VALUE);
    }

    #[test]
    fn build_script_child_env_sets_tool_target_and_manifest_package_name() {
        let dir = TempDir::new().unwrap();
        let package_root = dir.path().join("hyphen-pkg");
        let source_path = package_root.join("build.rs");
        let out_dir = dir.path().join("out");
        let rustc_path = dir.path().join("bin/rustc");
        let target_triple = "wasm32-unknown-unknown".to_string();
        let package_name = "hyphen-pkg".to_string();
        let mut unit =
            test_rust_derivation(0, "path+file://hyphen-pkg#hyphen-pkg@0.1.0", "custom-build", "host", Vec::new());
        unit.target_name = "build-script-build".to_string();
        unit.profile = "release".to_string();
        unit.derivation.args = vec![
            "--crate-name".to_string(),
            "build_script_build".to_string(),
            source_path.display().to_string(),
        ];
        unit.derivation.env.insert(BUILD_SCRIPT_TARGET_ENV.to_string(), target_triple.clone());
        unit.derivation.env.insert(BUILD_SCRIPT_CARGO_PKG_NAME_ENV.to_string(), package_name.clone());
        unit.derivation.env.insert(CARGO_PKG_VERSION_ENV.to_string(), "9.8.7".to_string());
        let options = RustUnitExecutionOptions {
            rustc: rustc_path.clone(),
            output_root: dir.path().join("unit-out"),
        };

        let root = build_script_package_root(&unit).expect("source path has package root");
        let env = build_script_child_env(&unit, &options, &out_dir, Some(&root));

        assert_eq!(root, package_root);
        assert_eq!(env.get(BUILD_SCRIPT_RUSTC_ENV).unwrap(), &normalize_path_string(&rustc_path));
        assert_eq!(env.get(BUILD_SCRIPT_TARGET_ENV).unwrap(), &target_triple);
        assert_eq!(env.get(BUILD_SCRIPT_HOST_ENV).unwrap(), &host_target_triple());
        assert_eq!(env.get(BUILD_SCRIPT_PROFILE_ENV).unwrap(), "release");
        assert_eq!(env.get(BUILD_SCRIPT_OUT_DIR_ENV).unwrap(), &normalize_path_string(&out_dir));
        assert_eq!(env.get(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV).unwrap(), &normalize_path_string(&package_root));
        assert_eq!(env.get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV).unwrap(), &package_name);
        assert_eq!(env.get(CARGO_PKG_VERSION_ENV).unwrap(), "9.8.7");
        assert_eq!(env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), CARGO_OPT_LEVEL_RELEASE);
        assert_eq!(env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), CARGO_DEBUG_FALSE);
        assert_eq!(env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), MANTLE_DETERMINISTIC_NUM_JOBS);
        assert_eq!(env.get(CARGO_CFG_TARGET_ARCH_ENV).unwrap(), "wasm32");
        assert_eq!(env.get(CARGO_CFG_TARGET_OS_ENV).unwrap(), "unknown");
        assert_eq!(env.get(CARGO_CFG_TARGET_ENV_ENV).unwrap(), "");
        assert_eq!(env.get(CARGO_CFG_TARGET_FAMILY_ENV).unwrap(), "wasm");
    }

    #[test]
    fn build_script_child_env_omits_manifest_dir_without_source_arg() {
        let dir = TempDir::new().unwrap();
        let out_dir = dir.path().join("out");
        let mut unit = test_rust_derivation(0, "path+file://package#package@0.1.0", "custom-build", "host", Vec::new());
        unit.derivation.args.clear();
        unit.derivation.env.clear();
        let options = RustUnitExecutionOptions {
            rustc: dir.path().join("rustc"),
            output_root: dir.path().join("unit-out"),
        };

        let env = build_script_child_env(&unit, &options, &out_dir, build_script_package_root(&unit).as_deref());

        assert!(!env.contains_key(BUILD_SCRIPT_CARGO_MANIFEST_DIR_ENV));
        assert_eq!(env.get(BUILD_SCRIPT_CARGO_PKG_NAME_ENV).unwrap(), &rust_crate_name(&unit.target_name));
        assert_eq!(env.get(BUILD_SCRIPT_TARGET_ENV).unwrap(), &host_target_triple());
        assert_eq!(env.get(BUILD_SCRIPT_PROFILE_ENV).unwrap(), DEFAULT_CARGO_PROFILE);
        assert_eq!(env.get(BUILD_SCRIPT_OPT_LEVEL_ENV).unwrap(), CARGO_OPT_LEVEL_DEBUG);
        assert_eq!(env.get(BUILD_SCRIPT_DEBUG_ENV).unwrap(), CARGO_DEBUG_TRUE);
        assert_eq!(env.get(BUILD_SCRIPT_NUM_JOBS_ENV).unwrap(), MANTLE_DETERMINISTIC_NUM_JOBS);
    }

    #[test]
    fn parse_build_script_metadata_captures_link_metadata() {
        let dir = TempDir::new().unwrap();
        let stdout = "cargo:include=/tmp/aws-lc/include\ncargo:libcrypto=aws_lc_0_39_1_crypto\ncargo:warning=ignored\ncargo:rustc-check-cfg=cfg(universal)\n";

        let metadata = parse_build_script_metadata(stdout, dir.path()).expect("metadata parses");

        assert_eq!(metadata.metadata.get("include").unwrap(), "/tmp/aws-lc/include");
        assert_eq!(metadata.metadata.get("libcrypto").unwrap(), "aws_lc_0_39_1_crypto");
        assert!(!metadata.metadata.contains_key("warning"));
        assert!(!metadata.metadata.contains_key("rustc-check-cfg"));
    }

    #[test]
    fn parse_build_script_metadata_accepts_bounded_link_lib_forms() {
        let dir = TempDir::new().unwrap();
        let stdout = "cargo:rustc-link-lib=stdc++\ncargo:rustc-link-lib=static=aws_lc_0_39_1_crypto\ncargo:rustc-link-lib=static:+whole-archive,-bundle=crypto_core\n";

        let metadata = parse_build_script_metadata(stdout, dir.path()).expect("metadata parses");

        assert_eq!(metadata.rustc_link_lib, vec![
            "static:+whole-archive,-bundle=crypto_core".to_string(),
            "static=aws_lc_0_39_1_crypto".to_string(),
            "stdc++".to_string(),
        ]);
        assert!(metadata.rustc_link_search.is_empty());
    }

    #[test]
    fn parse_build_script_metadata_rejects_unsafe_link_lib_forms() {
        let dir = TempDir::new().unwrap();
        let cases = [
            ("cargo:rustc-link-lib=shared=crypto\n", "unsupported rustc-link-lib kind"),
            (
                "cargo:rustc-link-lib=static:/tmp/libcrypto.a\n",
                "rustc-link-lib renames and comma-separated names are not supported",
            ),
            ("cargo:rustc-link-lib=static:whole-archive=crypto\n", "rustc-link-lib modifier must be a safe token"),
            (
                "cargo:rustc-link-lib=static=crypto:renamed\n",
                "rustc-link-lib renames and comma-separated names are not supported",
            ),
            ("cargo:rustc-link-lib=static=/tmp/libcrypto.a\n", "rustc-link-lib name must be a safe token"),
        ];

        for (stdout, expected) in cases {
            let err = parse_build_script_metadata(stdout, dir.path()).unwrap_err();
            assert_eq!(err.class, "malformed-build-script-metadata");
            assert!(err.message.contains(expected), "unexpected error: {}", err.message);
        }
    }

    #[test]
    fn parse_build_script_metadata_rejects_bad_custom_key() {
        let dir = TempDir::new().unwrap();
        let stdout = "cargo:bad.key=value\n";

        let err = parse_build_script_metadata(stdout, dir.path()).unwrap_err();

        assert_eq!(err.class, "malformed-build-script-metadata");
        assert!(err.message.contains("metadata key"));
    }

    #[test]
    fn bind_build_script_metadata_adds_dep_env_for_linked_dependency() {
        let mut unit =
            test_rust_derivation(0, "path+file://aws-lc-rs#aws-lc-rs@1.16.2", "custom-build", "host", Vec::new());
        unit.metadata_dependencies.push(BuildScriptMetadataDependency {
            package_id: "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.0".to_string(),
            links: "aws_lc_0_39_1".to_string(),
            producer_unit_id: None,
        });
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.0".to_string(),
            BuildScriptMetadataSummary {
                out_dir: "/tmp/out".to_string(),
                rustc_cfg: Vec::new(),
                rustc_env: BTreeMap::new(),
                rustc_link_lib: Vec::new(),
                rustc_link_search: Vec::new(),
                rerun_if_changed: Vec::new(),
                metadata: BTreeMap::from([("include".to_string(), "/tmp/aws-lc/include".to_string())]),
                digest_blake3: "digest".to_string(),
            },
        );

        let bound = bind_all_build_script_metadata(&unit, &metadata).expect("metadata binds");

        assert_eq!(bound.derivation.env.get("DEP_AWS_LC_0_39_1_INCLUDE").unwrap(), "/tmp/aws-lc/include");
        assert!(!bound.derivation.env.contains_key("DEP_AWS_LC_0_39_1_MISSING"));
    }

    #[test]
    fn rust_topology_runtime_args_disable_self_contained_linker_by_default() {
        let args = vec!["--crate-name".to_string(), "demo".to_string()];

        let runtime_args = rust_topology_runtime_args(&args);

        assert_eq!(&runtime_args[..args.len()], &args[..]);
        assert_eq!(runtime_args[args.len()], RUSTC_CODEGEN_OPTION_FLAG);
        assert_eq!(runtime_args[args.len() + 1usize], RUSTC_EXTERNAL_LINKER_MODE_ARG);
    }

    #[test]
    fn rust_topology_runtime_args_preserve_split_explicit_linker_mode() {
        let args = vec![
            RUSTC_CODEGEN_OPTION_FLAG.to_string(),
            "link-self-contained=yes".to_string(),
            "--crate-name".to_string(),
            "demo".to_string(),
        ];

        let runtime_args = rust_topology_runtime_args(&args);

        assert_eq!(runtime_args, args);
        assert_eq!(runtime_args.iter().filter(|arg| arg.contains(RUSTC_LINK_SELF_CONTAINED_OPTION)).count(), 1usize);
    }

    #[test]
    fn rust_topology_runtime_args_preserve_joined_explicit_linker_mode() {
        let args = vec![
            "-Clink-self-contained=no".to_string(),
            "--crate-name".to_string(),
            "demo".to_string(),
        ];

        let runtime_args = rust_topology_runtime_args(&args);

        assert_eq!(runtime_args, args);
        assert_eq!(runtime_args.iter().filter(|arg| arg.contains(RUSTC_LINK_SELF_CONTAINED_OPTION)).count(), 1usize);
    }

    #[test]
    fn rust_topology_runtime_args_ignore_near_match_linker_mode() {
        let args = vec![
            RUSTC_CODEGEN_OPTION_FLAG.to_string(),
            "link-self-containedness=yes".to_string(),
            "--crate-name".to_string(),
            "demo".to_string(),
        ];

        let runtime_args = rust_topology_runtime_args(&args);

        assert_eq!(&runtime_args[..args.len()], &args[..]);
        assert_eq!(runtime_args[args.len()], RUSTC_CODEGEN_OPTION_FLAG);
        assert_eq!(runtime_args[args.len() + 1usize], RUSTC_EXTERNAL_LINKER_MODE_ARG);
    }

    #[test]
    fn rust_topology_child_env_preserves_non_empty_inherited_path() {
        let mut explicit = BTreeMap::new();
        explicit.insert("TARGET".to_string(), "x86_64-unknown-linux-gnu".to_string());

        let env = rust_topology_child_env(&explicit, Some(OsString::from("/tool/bin:/bash/bin")), &BTreeMap::new());

        assert_eq!(env.get(RUST_TOPOLOGY_TOOL_PATH_ENV), Some(&OsString::from("/tool/bin:/bash/bin")));
        assert_eq!(env.get("TARGET"), Some(&OsString::from("x86_64-unknown-linux-gnu")));
    }

    #[test]
    fn rust_topology_child_env_omits_empty_inherited_path() {
        let explicit = BTreeMap::new();

        let env = rust_topology_child_env(&explicit, Some(OsString::new()), &BTreeMap::new());

        assert!(!env.contains_key(RUST_TOPOLOGY_TOOL_PATH_ENV));
        assert!(env.is_empty());
    }

    #[test]
    fn rust_topology_child_env_prefers_explicit_derivation_path() {
        let mut explicit = BTreeMap::new();
        explicit.insert(RUST_TOPOLOGY_TOOL_PATH_ENV.to_string(), "/derivation/bin".to_string());

        let env = rust_topology_child_env(&explicit, Some(OsString::from("/caller/bin")), &BTreeMap::new());

        assert_eq!(env.get(RUST_TOPOLOGY_TOOL_PATH_ENV), Some(&OsString::from("/derivation/bin")));
        assert_eq!(env.len(), 1usize);
    }

    #[test]
    fn rust_topology_child_env_forwards_allowlisted_compile_env() {
        let explicit = BTreeMap::new();
        let inherited_compile_env = BTreeMap::from([(
            SNIX_BUILD_SANDBOX_SHELL_ENV.to_string(),
            OsString::from("/nix/store/static-busybox/bin/busybox"),
        )]);

        let env = rust_topology_child_env(&explicit, None, &inherited_compile_env);

        assert_eq!(
            env.get(SNIX_BUILD_SANDBOX_SHELL_ENV),
            Some(&OsString::from("/nix/store/static-busybox/bin/busybox"))
        );
        assert_eq!(env.len(), 1usize);
    }

    #[test]
    fn rust_topology_child_env_prefers_explicit_compile_env() {
        let mut explicit = BTreeMap::new();
        explicit.insert(SNIX_BUILD_SANDBOX_SHELL_ENV.to_string(), "/explicit/busybox".to_string());
        let inherited_compile_env =
            BTreeMap::from([(SNIX_BUILD_SANDBOX_SHELL_ENV.to_string(), OsString::from("/ambient/busybox"))]);

        let env = rust_topology_child_env(&explicit, None, &inherited_compile_env);

        assert_eq!(env.get(SNIX_BUILD_SANDBOX_SHELL_ENV), Some(&OsString::from("/explicit/busybox")));
        assert_eq!(env.len(), 1usize);
    }

    #[test]
    fn allowed_rust_topology_compile_env_rejects_unrelated_ambient_env() {
        let candidates = BTreeMap::from([
            (SNIX_BUILD_SANDBOX_SHELL_ENV.to_string(), OsString::from("/busybox")),
            ("LD_PRELOAD".to_string(), OsString::from("/tmp/inject.so")),
            ("SECRET_TOKEN".to_string(), OsString::from("do-not-forward")),
        ]);

        let env = allowed_rust_topology_compile_env(&candidates);

        assert_eq!(env.get(SNIX_BUILD_SANDBOX_SHELL_ENV), Some(&OsString::from("/busybox")));
        assert!(!env.contains_key("LD_PRELOAD"));
        assert!(!env.contains_key("SECRET_TOKEN"));
        assert_eq!(env.len(), 1usize);
    }

    #[test]
    fn host_dependency_topology_accepts_proc_macro_host_producer() {
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#consumer@0.1.0";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22";
        let graph = test_unit_derivation_graph(vec![
            test_rust_derivation(0, host_consumer_id, "custom-build", "host", vec![test_dependency_artifact(
                proc_macro_id,
                "rustversion",
            )]),
            test_rust_derivation(1, proc_macro_id, "proc-macro", "host", Vec::new()),
        ]);
        let mut proc_macro_producers = BTreeMap::new();
        proc_macro_producers.insert(proc_macro_id.to_string(), 1usize);

        let plan = plan_host_dependency_topology(
            &[0usize, 1usize],
            &BTreeMap::new(),
            &proc_macro_producers,
            &BTreeMap::new(),
            &graph,
        )
        .unwrap();

        assert!(plan.target_dependency_indices.is_empty());
        assert_eq!(plan.host_edges.get(&0).cloned().unwrap_or_default(), vec![1usize]);
        assert!(plan.host_edges.get(&1).cloned().unwrap_or_default().is_empty());
    }

    #[test]
    fn host_dependency_topology_accepts_target_lib_producer() {
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#consumer@0.1.0";
        let target_lib_id = "registry+https://github.com/rust-lang/crates.io-index#cc@1.2.59";
        let graph = test_unit_derivation_graph(vec![
            test_rust_derivation(0, target_lib_id, "lib", "target", Vec::new()),
            test_rust_derivation(1, host_consumer_id, "custom-build", "host", vec![test_dependency_artifact(
                target_lib_id,
                "cc",
            )]),
        ]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(target_lib_id.to_string(), 0usize);

        let plan = plan_host_dependency_topology(&[1usize], &lib_producers, &BTreeMap::new(), &BTreeMap::new(), &graph)
            .unwrap();

        assert_eq!(plan.target_dependency_indices.iter().copied().collect::<Vec<_>>(), vec![0usize]);
        assert_eq!(plan.host_edges.get(&1).cloned().unwrap_or_default(), vec![0usize]);
    }

    #[test]
    fn bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder() {
        let dir = TempDir::new().unwrap();
        let target_id = "registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11";
        let produced_path = dir.path().join("libdarling_macro.so");
        std::fs::write(&produced_path, b"proc-macro").unwrap();
        let mut unit = test_rust_derivation(0, target_id, "lib", "target", Vec::new());
        unit.target_name = "darling".to_string();
        unit.derivation.args = vec!["--crate-name".to_string(), "darling".to_string()];
        let host_artifact = RustHostArtifact {
            package_id: proc_macro_id.to_string(),
            target_name: "darling_macro".to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:darling_macro".to_string(),
            metadata_digest_blake3: None,
        };
        unit.derivation.inputs = vec![host_artifact.artifact.clone()];
        unit.consumed_host_artifacts = vec![host_artifact];
        let mut produced_host_artifacts = BTreeMap::new();
        produced_host_artifacts.insert(proc_macro_id.to_string(), produced_path.clone());
        let produced_path_string = normalize_path_string(&produced_path);

        let bound = bind_all_host_artifacts(&unit, &produced_host_artifacts).unwrap();

        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("darling_macro={produced_path_string}")
        ));
        assert!(bound.derivation.inputs.contains(&produced_path_string));
        assert_eq!(bound.consumed_host_artifacts[0].artifact, produced_path_string);
        assert!(bound.dependency_artifacts.is_empty());
        assert_ne!(bound.rustc_args_digest_blake3, unit.rustc_args_digest_blake3);
    }

    #[test]
    fn bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata() {
        let dir = TempDir::new().unwrap();
        let target_id = "registry+https://github.com/rust-lang/crates.io-index#derive_builder_core@0.20.2";
        let darling_id = "registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11";
        let darling_path = dir.path().join("libdarling.rlib");
        let proc_macro_path = dir.path().join("proc-macros/libdarling_macro.so");
        std::fs::create_dir_all(proc_macro_path.parent().unwrap()).unwrap();
        std::fs::write(&darling_path, b"darling").unwrap();
        std::fs::write(&proc_macro_path, b"darling-macro").unwrap();
        let mut unit =
            test_rust_derivation(0, target_id, "lib", "target", vec![test_dependency_artifact(darling_id, "darling")]);
        unit.target_name = "derive_builder_core".to_string();
        unit.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("darling=artifact:{darling_id}:darling"),
        ];
        let mut produced_target_artifacts = BTreeMap::new();
        produced_target_artifacts.insert(darling_id.to_string(), darling_path.clone());
        let mut produced_proc_macro_artifacts = BTreeMap::new();
        produced_proc_macro_artifacts.insert(proc_macro_id.to_string(), proc_macro_path.clone());
        let proc_macro_search_path = format!("dependency={}", normalize_path_string(proc_macro_path.parent().unwrap()));

        let bound = bind_target_unit_artifacts(
            &unit,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &produced_target_artifacts,
            &produced_proc_macro_artifacts,
        )
        .unwrap();

        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("darling={}", normalize_path_string(&darling_path))
        ));
        assert!(has_ordered_arg_pair(&bound.derivation.args, "-L", &proc_macro_search_path));
        assert!(!has_rustc_extern_arg(&bound.derivation.args, "darling_macro"));
        assert_ne!(bound.rustc_args_digest_blake3, unit.rustc_args_digest_blake3);
    }

    #[test]
    fn append_all_produced_dependency_search_paths_keeps_duplicate_package_variant_dirs() {
        let dir = TempDir::new().unwrap();
        let earlier_variant = dir.path().join("variant-a/libcrunch_attestation_core.rlib");
        let later_variant = dir.path().join("variant-b/libcrunch_attestation_core.rlib");
        std::fs::create_dir_all(earlier_variant.parent().unwrap()).unwrap();
        std::fs::create_dir_all(later_variant.parent().unwrap()).unwrap();
        std::fs::write(&earlier_variant, b"earlier").unwrap();
        std::fs::write(&later_variant, b"later").unwrap();
        let unit_index = 0usize;
        let expected_search_path_count = 2usize;
        let mut unit = test_rust_derivation(
            unit_index,
            "path+file:///workspace/duplicate-variant-crate#0.1.0",
            "lib",
            "target",
            Vec::new(),
        );
        let earlier_search_path =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(earlier_variant.parent().unwrap()));
        let later_search_path =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(later_variant.parent().unwrap()));
        unit.derivation.args = vec![RUSTC_LINK_SEARCH_FLAG.to_string(), earlier_search_path.clone()];
        let original_digest = unit.rustc_args_digest_blake3.clone();

        append_all_produced_dependency_search_paths(&mut unit, &[earlier_variant.clone(), later_variant], &[]);

        assert!(has_ordered_arg_pair(&unit.derivation.args, RUSTC_LINK_SEARCH_FLAG, &earlier_search_path));
        assert!(has_ordered_arg_pair(&unit.derivation.args, RUSTC_LINK_SEARCH_FLAG, &later_search_path));
        assert_eq!(rustc_dependency_search_args(&unit.derivation.args).len(), expected_search_path_count);
        assert_ne!(unit.rustc_args_digest_blake3, original_digest);
    }

    #[test]
    fn unit_dependency_artifacts_preserve_selected_cargo_unit_id() {
        let package_id = "same-package";
        let crate_name = rust_crate_name(package_id);
        let units = vec![
            serde_json::json!({
                "pkg_id": package_id,
                "target": {
                    "name": package_id,
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": "/same/src/lib.rs",
                    "edition": "2021"
                },
                "mode": "build",
                "features": ["default"],
                "deps": []
            }),
            serde_json::json!({
                "pkg_id": package_id,
                "target": {
                    "name": package_id,
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": "/same/src/lib.rs",
                    "edition": "2021"
                },
                "mode": "build",
                "features": ["alt"],
                "deps": []
            }),
            serde_json::json!({
                "pkg_id": "consumer",
                "target": {
                    "name": "consumer",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": "/consumer/src/lib.rs",
                    "edition": "2021"
                },
                "mode": "build",
                "deps": [{"index": 1, "extern_crate_name": crate_name}]
            }),
        ];

        let artifacts = unit_dependency_artifacts(&units[2], &units);

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].producer_unit_id, Some(rust_unit_id(1, package_id, package_id, "lib", "build")));
        assert_ne!(artifacts[0].producer_unit_id, Some(rust_unit_id(0, package_id, package_id, "lib", "build")));
    }

    #[test]
    fn unit_dependency_artifacts_skip_host_producer_edges() {
        let package_id = "path+file:///workspace/build-link-crate#0.1.0";
        let units = vec![
            serde_json::json!({
                "pkg_id": package_id,
                "target": {
                    "name": "build-script-build",
                    "kind": ["custom-build"],
                    "crate_types": ["bin"],
                    "src_path": "/workspace/build-link-crate/build.rs",
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }),
            serde_json::json!({
                "pkg_id": package_id,
                "target": {
                    "name": "build-link-crate",
                    "kind": ["bin"],
                    "crate_types": ["bin"],
                    "src_path": "/workspace/build-link-crate/src/main.rs",
                    "edition": "2021"
                },
                "mode": "build",
                "deps": [{"index": 0, "extern_crate_name": "build_script_build"}]
            }),
        ];

        let artifacts = unit_dependency_artifacts(&units[1], &units);

        assert!(artifacts.is_empty());
        assert_eq!(dependency_producer_target_kind(&units[1]["deps"][0], &units), Some("custom-build".to_string()));
    }

    #[test]
    fn native_unit_graph_uses_stable_native_unit_identity() {
        let package_id = "same-package";
        let mut package = test_native_package(package_id, package_id, "lib", Vec::new());
        package.selected_features = vec!["alt".to_string(), "default".to_string()];
        let package_planning = test_package_planning(vec![package.clone()]);
        let source_closure = test_source_closure(&[package]);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(Path::new("/test"))
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": package_id,
                    "target": {
                        "name": package_id,
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "src_path": "/test/same-package/src/lib.rs",
                        "edition": "2021"
                    },
                    "mode": "build",
                    "features": ["default"],
                    "deps": []
                },
                {
                    "pkg_id": package_id,
                    "target": {
                        "name": package_id,
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "src_path": "/test/same-package/src/lib.rs",
                        "edition": "2021"
                    },
                    "mode": "build",
                    "features": ["alt"],
                    "deps": []
                }
            ]
        });
        let reversed_unit_graph = serde_json::json!({
            "units": unit_graph["units"].as_array().unwrap().iter().rev().cloned().collect::<Vec<_>>()
        });

        let native_graph =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let reversed_native_graph = summarize_native_unit_graph_planning(
            &reversed_unit_graph,
            &source_closure,
            &package_planning,
            &plan_options,
        )
        .unwrap();

        assert!(native_graph.ready, "{:#?}", native_graph.blockers);
        assert_eq!(native_graph.units.len(), 1);
        assert_eq!(native_graph.units[0].unit_id, reversed_native_graph.units[0].unit_id);
        assert!(native_graph.units[0].unit_id.starts_with("native:"));
        assert_ne!(native_graph.units[0].unit_id, rust_unit_id(0, package_id, package_id, "lib", "build"));
        assert_eq!(native_graph.units[0].selected_features, vec!["alt".to_string(), "default".to_string()]);
    }

    #[test]
    fn target_dependency_producer_index_uses_selected_unit_variant() {
        let package_id = "same-package";
        let first_index = 0usize;
        let selected_index = 1usize;
        let first_unit = test_rust_derivation(first_index, package_id, "lib", "target", Vec::new());
        let selected_unit = test_rust_derivation(selected_index, package_id, "lib", "target", Vec::new());
        let dependency = RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: rust_crate_name(package_id),
            producer_unit_id: Some(selected_unit.unit_id.clone()),
            artifact: format!("artifact:{package_id}:{}", rust_crate_name(package_id)),
        };
        let graph = test_unit_derivation_graph(vec![first_unit, selected_unit]);
        let unit_indices_by_id = rust_unit_indices_by_id(&graph);
        let mut package_first_lib_producers = BTreeMap::new();
        package_first_lib_producers.insert(package_id.to_string(), first_index);

        let producer_index =
            target_dependency_producer_index(&dependency, &graph, &unit_indices_by_id, &package_first_lib_producers)
                .unwrap();

        assert_eq!(producer_index, selected_index);
        assert_ne!(producer_index, package_first_lib_producers[package_id]);
    }

    #[test]
    fn bind_dependency_artifacts_uses_selected_unit_variant() {
        let dir = TempDir::new().unwrap();
        let package_id = "same-package";
        let crate_name = rust_crate_name(package_id);
        let earlier_path = dir.path().join("variant-a/libsame_package.rlib");
        let selected_path = dir.path().join("variant-b/libsame_package.rlib");
        std::fs::create_dir_all(earlier_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(selected_path.parent().unwrap()).unwrap();
        std::fs::write(&earlier_path, b"earlier").unwrap();
        std::fs::write(&selected_path, b"selected").unwrap();
        let earlier_unit = test_rust_derivation(0, package_id, "lib", "target", Vec::new());
        let selected_unit = test_rust_derivation(1, package_id, "lib", "target", Vec::new());
        let placeholder = format!("artifact:{package_id}:{crate_name}");
        let dependency = RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: crate_name.clone(),
            producer_unit_id: Some(selected_unit.unit_id.clone()),
            artifact: placeholder.clone(),
        };
        let mut consumer = test_rust_derivation(2, "consumer", "lib", "target", vec![dependency]);
        consumer.derivation.inputs = vec![placeholder.clone()];
        consumer.derivation.args = vec![RUSTC_EXTERN_FLAG.to_string(), format!("{crate_name}={placeholder}")];
        let mut index = ProducedArtifactIndex::default();
        index.record_unit(&earlier_unit, earlier_path.clone());
        index.record_unit(&selected_unit, selected_path.clone());

        let bound = bind_all_dependency_artifacts_with_index(&consumer, &index).unwrap();

        let selected_path_string = normalize_path_string(&selected_path);
        let earlier_path_string = normalize_path_string(&earlier_path);
        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("{crate_name}={selected_path_string}")
        ));
        assert!(bound.derivation.inputs.contains(&selected_path_string));
        assert!(!bound.derivation.args.iter().any(|arg| arg.contains(&earlier_path_string)));
        assert!(!bound.derivation.inputs.contains(&earlier_path_string));
    }

    #[test]
    fn package_only_dependency_artifacts_fail_on_ambiguous_unit_variants() {
        let dir = TempDir::new().unwrap();
        let package_id = "same-package";
        let crate_name = rust_crate_name(package_id);
        let first_path = dir.path().join("variant-a/libsame_package.rlib");
        let second_path = dir.path().join("variant-b/libsame_package.rlib");
        std::fs::create_dir_all(first_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(second_path.parent().unwrap()).unwrap();
        std::fs::write(&first_path, b"first").unwrap();
        std::fs::write(&second_path, b"second").unwrap();
        let first_unit = test_rust_derivation(0, package_id, "lib", "target", Vec::new());
        let second_unit = test_rust_derivation(1, package_id, "lib", "target", Vec::new());
        let dependency = test_dependency_artifact(package_id, &crate_name);
        let mut consumer = test_rust_derivation(2, "consumer", "lib", "target", vec![dependency]);
        consumer.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("{crate_name}=artifact:{package_id}:{crate_name}"),
        ];
        let mut index = ProducedArtifactIndex::default();
        index.record_unit(&first_unit, first_path);
        index.record_unit(&second_unit, second_path);

        let blocker = bind_all_dependency_artifacts_with_index(&consumer, &index).unwrap_err();

        assert_eq!(blocker.class, "ambiguous-dependency-producer");
        assert!(blocker.message.contains("package-only producer candidates"));
    }

    #[test]
    fn package_only_dependency_artifacts_bind_single_renamed_crate_candidate() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#lock_api@0.4.14";
        let produced_crate_name = "lock_api";
        let dependency_crate_name = "lock_api_crate";
        let produced_path = dir.path().join("liblock_api.rlib");
        std::fs::write(&produced_path, b"lock-api").unwrap();
        let producer = test_rust_derivation(0, package_id, "lib", "target", Vec::new());
        let placeholder = format!("artifact:{package_id}:{dependency_crate_name}");
        let dependency = RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: dependency_crate_name.to_string(),
            producer_unit_id: None,
            artifact: placeholder.clone(),
        };
        let mut consumer = test_rust_derivation(1, "consumer", "lib", "target", vec![dependency]);
        consumer.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("{dependency_crate_name}={placeholder}"),
        ];
        consumer.derivation.inputs = vec![placeholder];
        let mut index = ProducedArtifactIndex::default();
        index.record_unit(&producer, produced_path.clone());

        let bound = bind_all_dependency_artifacts_with_index(&consumer, &index).unwrap();

        let produced_path_string = normalize_path_string(&produced_path);
        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("{dependency_crate_name}={produced_path_string}")
        ));
        assert!(
            !bound
                .derivation
                .args
                .iter()
                .any(|arg| arg.contains(produced_crate_name) && !arg.contains(dependency_crate_name))
        );
    }

    #[test]
    fn selected_dependency_search_paths_follow_unit_variant_closure_only() {
        let dir = TempDir::new().unwrap();
        let producer_package = "same-package";
        let leaf_package = "leaf-package";
        let producer_crate_name = rust_crate_name(producer_package);
        let leaf_crate_name = rust_crate_name(leaf_package);
        let proc_macro_package = "macro-package";
        let selected_path = dir.path().join("selected/libsame_package.rlib");
        let unrelated_path = dir.path().join("unrelated/libsame_package.rlib");
        let leaf_path = dir.path().join("leaf/libleaf_package.rlib");
        let proc_macro_path = dir.path().join("macro/libmacro_package.so");
        for path in [&selected_path, &unrelated_path, &leaf_path, &proc_macro_path] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"artifact").unwrap();
        }
        let leaf_unit = test_rust_derivation(0, leaf_package, "lib", "target", Vec::new());
        let leaf_dependency = RustDependencyArtifact {
            package_id: leaf_package.to_string(),
            name: leaf_crate_name,
            producer_unit_id: Some(leaf_unit.unit_id.clone()),
            artifact: format!("artifact:{leaf_package}:{}", rust_crate_name(leaf_package)),
        };
        let proc_macro_unit = test_rust_derivation(4, proc_macro_package, "proc-macro", "host", Vec::new());
        let mut selected_unit = test_rust_derivation(1, producer_package, "lib", "target", vec![leaf_dependency]);
        selected_unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: proc_macro_package.to_string(),
            target_name: proc_macro_package.to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: Some(proc_macro_unit.unit_id.clone()),
            artifact: format!("host-artifact:{proc_macro_package}:proc-macro"),
            metadata_digest_blake3: None,
        }];
        let unrelated_unit = test_rust_derivation(2, producer_package, "lib", "target", Vec::new());
        let producer_dependency = RustDependencyArtifact {
            package_id: producer_package.to_string(),
            name: producer_crate_name.clone(),
            producer_unit_id: Some(selected_unit.unit_id.clone()),
            artifact: format!("artifact:{producer_package}:{producer_crate_name}"),
        };
        let mut consumer = test_rust_derivation(3, "consumer", "lib", "target", vec![producer_dependency]);
        let graph = test_unit_derivation_graph(vec![
            leaf_unit.clone(),
            selected_unit.clone(),
            unrelated_unit.clone(),
            consumer.clone(),
            proc_macro_unit.clone(),
        ]);
        let mut index = ProducedArtifactIndex::default();
        index.record_unit(&leaf_unit, leaf_path.clone());
        index.record_unit(&selected_unit, selected_path.clone());
        index.record_unit(&unrelated_unit, unrelated_path.clone());
        index.record_unit(&proc_macro_unit, proc_macro_path.clone());

        append_selected_dependency_search_paths(&mut consumer, &graph, &index).unwrap();

        let selected_search =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(selected_path.parent().unwrap()));
        let leaf_search =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(leaf_path.parent().unwrap()));
        let unrelated_search =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(unrelated_path.parent().unwrap()));
        let proc_macro_search =
            format!("{RUSTC_DEPENDENCY_SEARCH_PREFIX}{}", normalize_path_string(proc_macro_path.parent().unwrap()));
        assert!(has_ordered_arg_pair(&consumer.derivation.args, RUSTC_LINK_SEARCH_FLAG, &selected_search));
        assert!(has_ordered_arg_pair(&consumer.derivation.args, RUSTC_LINK_SEARCH_FLAG, &leaf_search));
        assert!(has_ordered_arg_pair(&consumer.derivation.args, RUSTC_LINK_SEARCH_FLAG, &proc_macro_search));
        assert!(!has_ordered_arg_pair(&consumer.derivation.args, RUSTC_LINK_SEARCH_FLAG, &unrelated_search));
    }

    #[test]
    fn bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.228";
        let produced_path = dir.path().join("build_script_build");
        std::fs::write(&produced_path, b"build-script").unwrap();
        let mut unit = test_rust_derivation(0, package_id, "lib", "target", Vec::new());
        let host_artifact = RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            target_kind: "custom-build".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:build-script".to_string(),
            metadata_digest_blake3: None,
        };
        unit.derivation.inputs = vec![host_artifact.artifact.clone()];
        unit.consumed_host_artifacts = vec![host_artifact];
        let mut produced_host_artifacts = BTreeMap::new();
        produced_host_artifacts.insert(package_id.to_string(), produced_path.clone());
        let produced_path_string = normalize_path_string(&produced_path);

        let bound = bind_all_host_artifacts(&unit, &produced_host_artifacts).unwrap();

        assert!(bound.derivation.inputs.contains(&produced_path_string));
        assert_eq!(bound.consumed_host_artifacts[0].artifact, produced_path_string);
        assert!(!has_rustc_extern_arg(&bound.derivation.args, "build_script_build"));
        assert_eq!(bound.dependency_artifacts, Vec::<RustDependencyArtifact>::new());
    }

    #[test]
    fn bind_all_host_artifacts_rewrites_custom_build_main_alias_placeholder() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1";
        let produced_path = dir.path().join(BUILD_SCRIPT_BUILD_CRATE_NAME);
        std::fs::write(&produced_path, b"build-script").unwrap();
        let dependency = RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: BUILD_SCRIPT_MAIN_CRATE_NAME.to_string(),
            producer_unit_id: None,
            artifact: format!("artifact:{package_id}:{BUILD_SCRIPT_MAIN_CRATE_NAME}"),
        };
        let mut unit = test_rust_derivation(0, package_id, "lib", "target", vec![dependency.clone()]);
        unit.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("{BUILD_SCRIPT_MAIN_CRATE_NAME}={}", dependency.artifact),
        ];
        unit.derivation.inputs = vec![dependency.artifact.clone(), "host-artifact:build-script".to_string()];
        unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            target_kind: "custom-build".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:build-script".to_string(),
            metadata_digest_blake3: None,
        }];
        let mut produced_host_artifacts = BTreeMap::new();
        produced_host_artifacts.insert(package_id.to_string(), produced_path.clone());
        let produced_path_string = normalize_path_string(&produced_path);

        let bound = bind_all_host_artifacts(&unit, &produced_host_artifacts).unwrap();

        assert_eq!(bound.dependency_artifacts[0].artifact, produced_path_string);
        assert!(bound.derivation.inputs.contains(&produced_path_string));
        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("{BUILD_SCRIPT_MAIN_CRATE_NAME}={produced_path_string}")
        ));
        assert!(
            !bound
                .derivation
                .args
                .iter()
                .any(|arg| arg == &format!("{BUILD_SCRIPT_MAIN_CRATE_NAME}={}", dependency.artifact))
        );
        assert_ne!(bound.rustc_args_digest_blake3, unit.rustc_args_digest_blake3);
    }

    #[test]
    fn bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#dupe-macro@0.1.0";
        let mut unit = test_rust_derivation(0, "path+file://consumer#consumer@0.1.0", "lib", "target", Vec::new());
        unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: "dupe_macro".to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:dupe_macro".to_string(),
            metadata_digest_blake3: None,
        }];
        let mut produced = ProducedHostArtifactIndex::default();
        let mut first = test_rust_derivation(1, package_id, "proc-macro", "host", Vec::new());
        let mut second = test_rust_derivation(2, package_id, "proc-macro", "host", Vec::new());
        first.target_name = "dupe_macro".to_string();
        second.target_name = "dupe_macro".to_string();
        produced.record_unit(&first, dir.path().join("libdupe_macro_first.so"));
        produced.record_unit(&second, dir.path().join("libdupe_macro_second.so"));

        let err = bind_all_host_artifacts_with_index(&unit, &produced).unwrap_err();

        assert_eq!(err.class, "ambiguous-host-artifact-producer");
        assert!(err.message.contains("package-only candidates"));
    }

    #[test]
    fn bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates() {
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#dupe-sys@0.1.0";
        let mut unit = test_rust_derivation(0, "path+file://consumer#consumer@0.1.0", "lib", "target", Vec::new());
        unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            target_kind: "custom-build".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:dupe-sys-build".to_string(),
            metadata_digest_blake3: None,
        }];
        let mut produced = ProducedBuildScriptMetadataIndex::default();
        let first = test_rust_derivation(1, package_id, "custom-build", "host", Vec::new());
        let second = test_rust_derivation(2, package_id, "custom-build", "host", Vec::new());
        produced.record_unit(&first, build_script_metadata_summary(package_id, BUILD_SCRIPT_TARGET_NAME));
        produced.record_unit(&second, build_script_metadata_summary(package_id, BUILD_SCRIPT_TARGET_NAME));

        let err = bind_all_build_script_metadata_with_index(&unit, &produced).unwrap_err();

        assert_eq!(err.class, "ambiguous-build-script-metadata-producer");
        assert!(err.message.contains("package-only candidates"));
    }

    #[test]
    fn build_script_metadata_producer_index_keeps_same_alias_duplicates_ambiguous() {
        const FIRST_BUILD_INDEX: usize = 0;
        const SECOND_BUILD_INDEX: usize = 1;
        const MAIN_ALIAS_INDEX: usize = 2;
        const CONSUMER_INDEX: usize = 3;
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#dupe-sys@0.1.0";
        let mut first = test_rust_derivation(FIRST_BUILD_INDEX, package_id, "custom-build", "host", Vec::new());
        let mut second = test_rust_derivation(SECOND_BUILD_INDEX, package_id, "custom-build", "host", Vec::new());
        let mut main_alias = test_rust_derivation(MAIN_ALIAS_INDEX, package_id, "custom-build", "host", Vec::new());
        first.target_name = BUILD_SCRIPT_TARGET_NAME.to_string();
        first.unit_id = rust_unit_id(FIRST_BUILD_INDEX, package_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        second.target_name = BUILD_SCRIPT_TARGET_NAME.to_string();
        second.unit_id =
            rust_unit_id(SECOND_BUILD_INDEX, package_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        main_alias.target_name = BUILD_SCRIPT_MAIN_TARGET_NAME.to_string();
        main_alias.unit_id =
            rust_unit_id(MAIN_ALIAS_INDEX, package_id, BUILD_SCRIPT_MAIN_TARGET_NAME, "custom-build", "build");
        let consumer = test_rust_derivation(
            CONSUMER_INDEX,
            "path+file://consumer#consumer@0.1.0",
            "custom-build",
            "host",
            Vec::new(),
        );
        let graph = test_unit_derivation_graph(vec![first, second, main_alias, consumer.clone()]);
        let producers =
            build_script_metadata_producers(&graph, &[FIRST_BUILD_INDEX, SECOND_BUILD_INDEX, MAIN_ALIAS_INDEX]);
        let dependency = BuildScriptMetadataDependency {
            package_id: package_id.to_string(),
            links: "dupe_native".to_string(),
            producer_unit_id: None,
        };

        let err = select_build_script_metadata_producer_index(&consumer, &dependency, &producers, &graph).unwrap_err();

        assert_eq!(err.class, "ambiguous-build-script-metadata-producer");
        assert!(err.message.contains("candidate custom-build producers"));
    }

    #[test]
    fn bind_build_script_metadata_with_index_keeps_same_alias_duplicates_ambiguous() {
        const CONSUMER_INDEX: usize = 0;
        const FIRST_BUILD_INDEX: usize = 1;
        const SECOND_BUILD_INDEX: usize = 2;
        const MAIN_ALIAS_INDEX: usize = 3;
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#dupe-sys@0.1.0";
        let mut unit =
            test_rust_derivation(CONSUMER_INDEX, "path+file://consumer#consumer@0.1.0", "lib", "target", Vec::new());
        unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            target_kind: "custom-build".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:dupe-sys-build".to_string(),
            metadata_digest_blake3: None,
        }];
        let mut produced = ProducedBuildScriptMetadataIndex::default();
        let mut first = test_rust_derivation(FIRST_BUILD_INDEX, package_id, "custom-build", "host", Vec::new());
        let mut second = test_rust_derivation(SECOND_BUILD_INDEX, package_id, "custom-build", "host", Vec::new());
        let mut main_alias = test_rust_derivation(MAIN_ALIAS_INDEX, package_id, "custom-build", "host", Vec::new());
        first.target_name = BUILD_SCRIPT_TARGET_NAME.to_string();
        first.unit_id = rust_unit_id(FIRST_BUILD_INDEX, package_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        second.target_name = BUILD_SCRIPT_TARGET_NAME.to_string();
        second.unit_id =
            rust_unit_id(SECOND_BUILD_INDEX, package_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        main_alias.target_name = BUILD_SCRIPT_MAIN_TARGET_NAME.to_string();
        main_alias.unit_id =
            rust_unit_id(MAIN_ALIAS_INDEX, package_id, BUILD_SCRIPT_MAIN_TARGET_NAME, "custom-build", "build");
        produced.record_unit(&first, build_script_metadata_summary(package_id, BUILD_SCRIPT_TARGET_NAME));
        produced.record_unit(&second, build_script_metadata_summary(package_id, BUILD_SCRIPT_TARGET_NAME));
        produced.record_unit(&main_alias, build_script_metadata_summary(package_id, BUILD_SCRIPT_MAIN_TARGET_NAME));

        let err = bind_all_build_script_metadata_with_index(&unit, &produced).unwrap_err();

        assert_eq!(err.class, "ambiguous-build-script-metadata-producer");
        assert!(err.message.contains("package-only candidates"));
    }

    #[test]
    fn bind_all_host_artifacts_leaves_unknown_custom_build_alias_placeholder() {
        let dir = TempDir::new().unwrap();
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1";
        let unknown_alias = "custom_build_helper";
        let produced_path = dir.path().join(BUILD_SCRIPT_BUILD_CRATE_NAME);
        std::fs::write(&produced_path, b"build-script").unwrap();
        let dependency = RustDependencyArtifact {
            package_id: package_id.to_string(),
            name: unknown_alias.to_string(),
            producer_unit_id: None,
            artifact: format!("artifact:{package_id}:{unknown_alias}"),
        };
        let mut unit = test_rust_derivation(0, package_id, "lib", "target", vec![dependency.clone()]);
        unit.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("{unknown_alias}={}", dependency.artifact),
        ];
        unit.derivation.inputs = vec![dependency.artifact.clone(), "host-artifact:build-script".to_string()];
        unit.consumed_host_artifacts = vec![RustHostArtifact {
            package_id: package_id.to_string(),
            target_name: BUILD_SCRIPT_TARGET_NAME.to_string(),
            target_kind: "custom-build".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:build-script".to_string(),
            metadata_digest_blake3: None,
        }];
        let mut produced_host_artifacts = BTreeMap::new();
        produced_host_artifacts.insert(package_id.to_string(), produced_path.clone());
        let produced_path_string = normalize_path_string(&produced_path);

        let bound = bind_all_host_artifacts(&unit, &produced_host_artifacts).unwrap();

        assert_ne!(bound.consumed_host_artifacts[0].artifact, unit.consumed_host_artifacts[0].artifact);
        assert_eq!(bound.dependency_artifacts[0], dependency);
        assert!(bound.derivation.inputs.contains(&produced_path_string));
        assert!(bound.derivation.inputs.contains(&dependency.artifact));
        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("{unknown_alias}={}", dependency.artifact)
        ));
        assert!(!has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("{unknown_alias}={produced_path_string}")
        ));
    }

    #[test]
    fn bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate() {
        let dir = TempDir::new().unwrap();
        let target_id = "registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11";
        let produced_path = dir.path().join("libdarling_macro.so");
        std::fs::write(&produced_path, b"proc-macro").unwrap();
        let dependency = test_dependency_artifact(proc_macro_id, "darling_macro");
        let mut unit = test_rust_derivation(0, target_id, "lib", "target", vec![dependency.clone()]);
        unit.target_name = "darling".to_string();
        unit.derivation.args = vec![
            RUSTC_EXTERN_FLAG.to_string(),
            format!("darling_macro={}", dependency.artifact),
        ];
        let host_artifact = RustHostArtifact {
            package_id: proc_macro_id.to_string(),
            target_name: "darling_macro".to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: None,
            artifact: "host-artifact:darling_macro".to_string(),
            metadata_digest_blake3: None,
        };
        unit.derivation.inputs = vec![dependency.artifact.clone(), host_artifact.artifact.clone()];
        unit.consumed_host_artifacts = vec![host_artifact];
        let mut produced_host_artifacts = BTreeMap::new();
        produced_host_artifacts.insert(proc_macro_id.to_string(), produced_path.clone());
        let produced_path_string = normalize_path_string(&produced_path);

        let bound = bind_all_host_artifacts(&unit, &produced_host_artifacts).unwrap();
        let extern_count = bound
            .derivation
            .args
            .windows(RUSTC_EXTERN_ARG_PAIR_WIDTH)
            .filter(|window| window[0] == RUSTC_EXTERN_FLAG && window[1].starts_with("darling_macro="))
            .count();

        assert_eq!(extern_count, 1);
        assert!(has_ordered_arg_pair(
            &bound.derivation.args,
            RUSTC_EXTERN_FLAG,
            &format!("darling_macro={produced_path_string}")
        ));
        assert_eq!(bound.dependency_artifacts[0].artifact, produced_path_string);
        assert!(!bound.derivation.args.iter().any(|arg| arg == &format!("darling_macro={}", dependency.artifact)));
        assert_ne!(bound.rustc_args_digest_blake3, unit.rustc_args_digest_blake3);
    }

    #[test]
    fn combined_unit_topology_orders_proc_macro_dependency_lib_before_host_unit() {
        let dependency_id = "registry+https://github.com/rust-lang/crates.io-index#proc-macro2@1.0.106";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#async-stream-impl@0.3.6";
        let target_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#async-stream@0.3.6";
        let dependency_lib = test_rust_derivation(0, dependency_id, "lib", "target", Vec::new());
        let proc_macro = test_rust_derivation(1, proc_macro_id, "proc-macro", "host", vec![test_dependency_artifact(
            dependency_id,
            "proc_macro2",
        )]);
        let mut target_consumer = test_rust_derivation(2, target_consumer_id, "lib", "target", Vec::new());
        target_consumer.consumed_host_artifacts = vec![test_host_artifact(proc_macro_id, "proc-macro")];
        let graph = test_unit_derivation_graph(vec![dependency_lib, proc_macro, target_consumer]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(dependency_id.to_string(), 0usize);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(proc_macro_id.to_string(), 1usize);
        let mut proc_macro_producers = BTreeMap::new();
        proc_macro_producers.insert(proc_macro_id.to_string(), 1usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(0usize, Vec::new());
        target_edges.insert(2usize, Vec::new());

        let order = plan_combined_unit_topology_order(
            &[0usize, 2usize],
            &[1usize],
            &lib_producers,
            &host_producers,
            &proc_macro_producers,
            &target_edges,
            &graph,
        )
        .unwrap();

        assert_eq!(order, vec![0usize, 1usize, 2usize]);
    }

    #[test]
    fn host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets() {
        let dependency_id = "registry+https://github.com/rust-lang/crates.io-index#cc@1.2.59";
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1";
        let target_consumer_id = "path+file://mantle#mantle@0.1.0";
        let target_dependency = test_rust_derivation(0, dependency_id, "lib", "target", Vec::new());
        let target_dependency_unit_id = target_dependency.unit_id.clone();
        let host_dependency_id = host_dependency_unit_id(&target_dependency_unit_id);
        let dependency_artifact = RustDependencyArtifact {
            package_id: dependency_id.to_string(),
            name: "cc".to_string(),
            producer_unit_id: Some(target_dependency_unit_id.clone()),
            artifact: format!("artifact:{dependency_id}:cc"),
        };
        let host_consumer =
            test_rust_derivation(1, host_consumer_id, "custom-build", "host", vec![dependency_artifact.clone()]);
        let target_consumer = test_rust_derivation(2, target_consumer_id, "lib", "target", vec![dependency_artifact]);
        let mut derivations = vec![target_dependency, host_consumer, target_consumer];

        add_host_dependency_derivations(&mut derivations);

        let host_dependency = derivations
            .iter()
            .find(|unit| unit.unit_id == host_dependency_id)
            .expect("host dependency clone exists");
        assert_eq!(host_dependency.execution_kind, HOST_DEPENDENCY_EXECUTION_KIND);
        assert_eq!(host_dependency.target_kind, "lib");
        assert_eq!(host_dependency.derivation.env.get("CRATE_KIND").unwrap(), HOST_DEPENDENCY_CRATE_KIND);
        assert_eq!(
            host_dependency.derivation.env.get(RUST_UNIT_EXECUTION_KIND_ENV).unwrap(),
            HOST_DEPENDENCY_EXECUTION_KIND
        );
        let remapped_host_consumer = derivations.iter().find(|unit| unit.package_id == host_consumer_id).unwrap();
        assert_eq!(
            remapped_host_consumer.dependency_artifacts[0].producer_unit_id.as_deref(),
            Some(host_dependency_id.as_str())
        );
        let untouched_target_consumer = derivations.iter().find(|unit| unit.package_id == target_consumer_id).unwrap();
        assert_eq!(
            untouched_target_consumer.dependency_artifacts[0].producer_unit_id.as_deref(),
            Some(target_dependency_unit_id.as_str())
        );
    }

    #[test]
    fn host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks() {
        let dependency_id = "registry+https://github.com/rust-lang/crates.io-index#duplicate@1.0.0";
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#consumer@1.0.0";
        let target_dependency_a = test_rust_derivation(0, dependency_id, "lib", "target", Vec::new());
        let target_dependency_b = test_rust_derivation(1, dependency_id, "lib", "target", Vec::new());
        let host_consumer =
            test_rust_derivation(2, host_consumer_id, "custom-build", "host", vec![RustDependencyArtifact {
                package_id: dependency_id.to_string(),
                name: "duplicate".to_string(),
                producer_unit_id: None,
                artifact: format!("artifact:{dependency_id}:duplicate"),
            }]);
        let mut derivations = vec![target_dependency_a, target_dependency_b, host_consumer];

        add_host_dependency_derivations(&mut derivations);

        let host_dependency_count =
            derivations.iter().filter(|unit| unit.execution_kind == HOST_DEPENDENCY_EXECUTION_KIND).count();
        let remapped_host_consumer = derivations.iter().find(|unit| unit.package_id == host_consumer_id).unwrap();
        assert_eq!(host_dependency_count, 0usize);
        assert_eq!(remapped_host_consumer.dependency_artifacts[0].producer_unit_id, None);
    }

    #[test]
    fn combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib() {
        let dependency_id = "registry+https://github.com/rust-lang/crates.io-index#cc@1.2.59";
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1";
        let target_consumer_id = "path+file://mantle#mantle@0.1.0";
        let target_dependency = test_rust_derivation(0, dependency_id, "lib", "target", Vec::new());
        let mut host_dependency =
            test_rust_derivation(1, dependency_id, "lib", HOST_DEPENDENCY_EXECUTION_KIND, Vec::new());
        host_dependency.unit_id = host_dependency_unit_id(&target_dependency.unit_id);
        host_dependency
            .derivation
            .env
            .insert("CRATE_KIND".to_string(), HOST_DEPENDENCY_CRATE_KIND.to_string());
        let host_consumer =
            test_rust_derivation(2, host_consumer_id, "custom-build", "host", vec![RustDependencyArtifact {
                package_id: dependency_id.to_string(),
                name: "cc".to_string(),
                producer_unit_id: Some(host_dependency.unit_id.clone()),
                artifact: format!("artifact:{dependency_id}:cc"),
            }]);
        let target_consumer =
            test_rust_derivation(3, target_consumer_id, "lib", "target", vec![RustDependencyArtifact {
                package_id: dependency_id.to_string(),
                name: "cc".to_string(),
                producer_unit_id: Some(target_dependency.unit_id.clone()),
                artifact: format!("artifact:{dependency_id}:cc"),
            }]);
        let graph = test_unit_derivation_graph(vec![
            target_dependency,
            host_dependency.clone(),
            host_consumer,
            target_consumer,
        ]);
        let mut host_dependency_lib_producers = BTreeMap::new();
        host_dependency_lib_producers.insert(dependency_id.to_string(), 1usize);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(host_consumer_id.to_string(), 2usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(0usize, Vec::new());
        target_edges.insert(1usize, Vec::new());
        target_edges.insert(3usize, vec![0usize]);

        let order = plan_combined_unit_topology_order(
            &[0usize, 3usize],
            &[2usize],
            &host_dependency_lib_producers,
            &host_producers,
            &BTreeMap::new(),
            &target_edges,
            &graph,
        )
        .unwrap();

        let host_dependency_position = order.iter().position(|index| *index == 1usize).unwrap();
        let host_consumer_position = order.iter().position(|index| *index == 2usize).unwrap();
        let target_dependency_position = order.iter().position(|index| *index == 0usize).unwrap();
        let target_consumer_position = order.iter().position(|index| *index == 3usize).unwrap();
        assert!(host_dependency_position < host_consumer_position);
        assert!(target_dependency_position < target_consumer_position);
        assert!(host_consumer_position < target_consumer_position);
    }

    #[test]
    fn combined_unit_topology_orders_host_build_script_before_same_package_proc_macro() {
        let package_id = "registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22";
        let target_id = "path+file://target#target@0.1.0";
        let build_script = test_rust_derivation(0, package_id, "custom-build", "host", Vec::new());
        let mut proc_macro = test_rust_derivation(1, package_id, "proc-macro", "host", Vec::new());
        proc_macro.consumed_host_artifacts = vec![test_host_artifact(package_id, "custom-build")];
        let mut target_consumer = test_rust_derivation(2, target_id, "lib", "target", Vec::new());
        target_consumer.consumed_host_artifacts = vec![test_host_artifact(package_id, "proc-macro")];
        let graph = test_unit_derivation_graph(vec![build_script, proc_macro, target_consumer]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(target_id.to_string(), 2usize);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(package_id.to_string(), 0usize);
        let mut proc_macro_producers = BTreeMap::new();
        proc_macro_producers.insert(package_id.to_string(), 1usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(2usize, Vec::new());

        let order = plan_combined_unit_topology_order(
            &[2usize],
            &[0usize, 1usize],
            &lib_producers,
            &host_producers,
            &proc_macro_producers,
            &target_edges,
            &graph,
        )
        .unwrap();

        assert_eq!(order, vec![0usize, 1usize, 2usize]);
    }

    #[test]
    fn combined_unit_topology_orders_target_host_and_proc_macro_edges() {
        let target_lib_id = "registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.228";
        let proc_macro_id = "registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22";
        let mut target_lib = test_rust_derivation(0, target_lib_id, "lib", "target", Vec::new());
        target_lib.consumed_host_artifacts = vec![test_host_artifact(target_lib_id, "custom-build")];
        let host_build_script =
            test_rust_derivation(1, target_lib_id, "custom-build", "host", vec![test_dependency_artifact(
                proc_macro_id,
                "rustversion",
            )]);
        let proc_macro = test_rust_derivation(2, proc_macro_id, "proc-macro", "host", Vec::new());
        let graph = test_unit_derivation_graph(vec![target_lib, host_build_script, proc_macro]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(target_lib_id.to_string(), 0usize);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(target_lib_id.to_string(), 1usize);
        host_producers.insert(proc_macro_id.to_string(), 2usize);
        let mut proc_macro_producers = BTreeMap::new();
        proc_macro_producers.insert(proc_macro_id.to_string(), 2usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(0usize, Vec::new());

        let order = plan_combined_unit_topology_order(
            &[0usize],
            &[1usize, 2usize],
            &lib_producers,
            &host_producers,
            &proc_macro_producers,
            &target_edges,
            &graph,
        )
        .unwrap();

        assert_eq!(order, vec![2usize, 1usize, 0usize]);
    }

    #[test]
    fn native_host_metadata_dependencies_follow_selected_target_artifacts_only() {
        let consumer_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-rs@1.16.2";
        let linked_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1";
        let unselected_id = "registry+https://github.com/rust-lang/crates.io-index#unused-sys@1.0.0";
        let build_only_id = "registry+https://github.com/rust-lang/crates.io-index#build-only-sys@1.0.0";
        let mut consumer = test_native_package(consumer_id, "aws-lc-rs", "custom-build", vec![
            NativePathDependencySummary {
                name: "aws-lc-sys".to_string(),
                manifest_path: "/test/aws-lc-sys/Cargo.toml".to_string(),
            },
            NativePathDependencySummary {
                name: "unused-sys".to_string(),
                manifest_path: "/test/unused-sys/Cargo.toml".to_string(),
            },
        ]);
        consumer.build_dependencies = vec![NativePathDependencySummary {
            name: "build-only-sys".to_string(),
            manifest_path: "/test/build-only-sys/Cargo.toml".to_string(),
        }];
        let mut linked = test_native_package(linked_id, "aws-lc-sys", "custom-build", Vec::new());
        linked.links = Some("aws_lc_0_39_1".to_string());
        let mut unselected = test_native_package(unselected_id, "unused-sys", "custom-build", Vec::new());
        unselected.links = Some("unused_native".to_string());
        let mut build_only = test_native_package(build_only_id, "build-only-sys", "custom-build", Vec::new());
        build_only.links = Some("build_only_native".to_string());
        let packages = [consumer.clone(), linked, unselected, build_only];
        let packages_by_id =
            packages.iter().map(|package| (package.package_id.clone(), package)).collect::<BTreeMap<_, _>>();
        let selected_artifacts = BTreeSet::from([
            test_dependency_artifact(consumer_id, "aws_lc_rs"),
            test_dependency_artifact(linked_id, "aws_lc_sys"),
        ]);
        let linked_producer_unit_id = rust_unit_id(0, linked_id, BUILD_SCRIPT_TARGET_NAME, "custom-build", "build");
        let custom_build_metadata_producers_by_package =
            BTreeMap::from([(linked_id.to_string(), vec![linked_producer_unit_id.clone()])]);
        let mut blockers = Vec::new();

        let dependencies = native_host_metadata_dependencies(
            &consumer,
            "custom-build",
            Some(&selected_artifacts),
            &packages_by_id,
            &custom_build_metadata_producers_by_package,
            &mut blockers,
        );

        assert_eq!(dependencies, vec![BuildScriptMetadataDependency {
            package_id: linked_id.to_string(),
            links: "aws_lc_0_39_1".to_string(),
            producer_unit_id: Some(linked_producer_unit_id),
        }]);
        assert!(blockers.is_empty());
        assert!(dependencies.iter().all(|dependency| dependency.package_id != unselected_id));
        assert!(dependencies.iter().all(|dependency| dependency.package_id != build_only_id));
    }

    #[test]
    fn native_host_metadata_dependencies_ignore_unselected_linked_manifest_edges() {
        let consumer_id = "registry+https://github.com/rust-lang/crates.io-index#consumer@1.0.0";
        let linked_id = "registry+https://github.com/rust-lang/crates.io-index#optional-sys@1.0.0";
        let consumer =
            test_native_package(consumer_id, "consumer", "custom-build", vec![NativePathDependencySummary {
                name: "optional-sys".to_string(),
                manifest_path: "/test/optional-sys/Cargo.toml".to_string(),
            }]);
        let mut linked = test_native_package(linked_id, "optional-sys", "custom-build", Vec::new());
        linked.links = Some("optional_native".to_string());
        let packages = [consumer.clone(), linked];
        let packages_by_id =
            packages.iter().map(|package| (package.package_id.clone(), package)).collect::<BTreeMap<_, _>>();
        let selected_artifacts = BTreeSet::new();
        let custom_build_metadata_producers_by_package = BTreeMap::new();
        let mut blockers = Vec::new();

        let dependencies = native_host_metadata_dependencies(
            &consumer,
            "custom-build",
            Some(&selected_artifacts),
            &packages_by_id,
            &custom_build_metadata_producers_by_package,
            &mut blockers,
        );

        assert!(dependencies.is_empty());
        assert!(blockers.is_empty());
    }

    #[test]
    fn combined_unit_topology_orders_linked_metadata_before_dependent_build_script() {
        const LINKED_BUILD_INDEX: usize = 0;
        const DEPENDENT_BUILD_INDEX: usize = 1;
        const TARGET_INDEX: usize = 2;
        let linked_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.0";
        let dependent_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-rs@1.16.2";
        let linked_build = test_rust_derivation(LINKED_BUILD_INDEX, linked_id, "custom-build", "host", Vec::new());
        let mut dependent_build =
            test_rust_derivation(DEPENDENT_BUILD_INDEX, dependent_id, "custom-build", "host", Vec::new());
        dependent_build.metadata_dependencies.push(BuildScriptMetadataDependency {
            package_id: linked_id.to_string(),
            links: "aws_lc_0_39_1".to_string(),
            producer_unit_id: None,
        });
        let target_lib = test_rust_derivation(TARGET_INDEX, dependent_id, "lib", "target", Vec::new());
        let graph = test_unit_derivation_graph(vec![linked_build, dependent_build, target_lib]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(dependent_id.to_string(), TARGET_INDEX);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(linked_id.to_string(), LINKED_BUILD_INDEX);
        host_producers.insert(dependent_id.to_string(), DEPENDENT_BUILD_INDEX);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(TARGET_INDEX, Vec::new());

        let order = plan_combined_unit_topology_order(
            &[TARGET_INDEX],
            &[LINKED_BUILD_INDEX, DEPENDENT_BUILD_INDEX],
            &lib_producers,
            &host_producers,
            &BTreeMap::new(),
            &target_edges,
            &graph,
        )
        .unwrap();

        assert_eq!(order, vec![LINKED_BUILD_INDEX, DEPENDENT_BUILD_INDEX, TARGET_INDEX]);
    }

    #[test]
    fn combined_unit_topology_blocks_missing_linked_metadata_producer() {
        const DEPENDENT_BUILD_INDEX: usize = 0;
        const TARGET_INDEX: usize = 1;
        let linked_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.0";
        let dependent_id = "registry+https://github.com/rust-lang/crates.io-index#aws-lc-rs@1.16.2";
        let mut dependent_build =
            test_rust_derivation(DEPENDENT_BUILD_INDEX, dependent_id, "custom-build", "host", Vec::new());
        dependent_build.metadata_dependencies.push(BuildScriptMetadataDependency {
            package_id: linked_id.to_string(),
            links: "aws_lc_0_39_1".to_string(),
            producer_unit_id: None,
        });
        let target_lib = test_rust_derivation(TARGET_INDEX, dependent_id, "lib", "target", Vec::new());
        let graph = test_unit_derivation_graph(vec![dependent_build, target_lib]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(dependent_id.to_string(), TARGET_INDEX);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(dependent_id.to_string(), DEPENDENT_BUILD_INDEX);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(TARGET_INDEX, Vec::new());

        let err = plan_combined_unit_topology_order(
            &[TARGET_INDEX],
            &[DEPENDENT_BUILD_INDEX],
            &lib_producers,
            &host_producers,
            &BTreeMap::new(),
            &target_edges,
            &graph,
        )
        .unwrap_err();

        assert_eq!(err.class, "missing-build-script-metadata-producer");
        assert!(err.message.contains(linked_id));
    }

    #[test]
    fn combined_unit_topology_keeps_standalone_host_units() {
        let target_lib_id = "registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.228";
        let standalone_host_id = "registry+https://github.com/rust-lang/crates.io-index#standalone-build@0.1.0";
        let graph = test_unit_derivation_graph(vec![
            test_rust_derivation(0, target_lib_id, "lib", "target", Vec::new()),
            test_rust_derivation(1, standalone_host_id, "custom-build", "host", Vec::new()),
        ]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(target_lib_id.to_string(), 0usize);
        let mut host_producers = BTreeMap::new();
        host_producers.insert(standalone_host_id.to_string(), 1usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(0usize, Vec::new());

        let order = plan_combined_unit_topology_order(
            &[0usize],
            &[1usize],
            &lib_producers,
            &host_producers,
            &BTreeMap::new(),
            &target_edges,
            &graph,
        )
        .unwrap();

        assert_eq!(order, vec![1usize, 0usize]);
    }

    #[test]
    fn combined_unit_topology_blocks_missing_target_host_artifact_producer() {
        let target_lib_id = "registry+https://github.com/rust-lang/crates.io-index#serde_core@1.0.228";
        let mut target_lib = test_rust_derivation(0, target_lib_id, "lib", "target", Vec::new());
        target_lib.consumed_host_artifacts = vec![test_host_artifact(target_lib_id, "custom-build")];
        let graph = test_unit_derivation_graph(vec![target_lib]);
        let mut lib_producers = BTreeMap::new();
        lib_producers.insert(target_lib_id.to_string(), 0usize);
        let mut target_edges = BTreeMap::new();
        target_edges.insert(0usize, Vec::new());

        let err = plan_combined_unit_topology_order(
            &[0usize],
            &[],
            &lib_producers,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &target_edges,
            &graph,
        )
        .unwrap_err();

        assert_eq!(err.class, "missing-host-artifact-producer");
        assert!(err.message.contains(target_lib_id));
    }

    #[test]
    fn host_dependency_topology_blocks_missing_producer() {
        let host_consumer_id = "registry+https://github.com/rust-lang/crates.io-index#consumer@0.1.0";
        let missing_id = "registry+https://github.com/rust-lang/crates.io-index#missing@0.1.0";
        let graph =
            test_unit_derivation_graph(vec![test_rust_derivation(0, host_consumer_id, "custom-build", "host", vec![
                test_dependency_artifact(missing_id, "missing"),
            ])]);

        let err =
            plan_host_dependency_topology(&[0usize], &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new(), &graph)
                .unwrap_err();

        assert_eq!(err.class, "missing-host-dependency-producer");
        assert!(err.message.contains(missing_id));
    }

    #[test]
    fn captures_normalized_oracle_receipt() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "# lock\n").unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        let metadata = format!(
            r#"{{"packages":[{{"id":"path+file://root#demo@0.1.0","name":"demo","version":"0.1.0","source":null,"manifest_path":"{}"}}],"workspace_root":"{}","workspace_members":["path+file://root#demo@0.1.0"]}}"#,
            manifest_path.display(),
            dir.path().display()
        );
        let oracle = FixedOracle {
            cargo_version: ok_output("cargo 1.91.0\n"),
            rustc_version: ok_output("rustc 1.91.0\nhost: x86_64-unknown-linux-gnu\n"),
            metadata: ok_output(&metadata),
            unit_graph: ok_output(&format!(
                r#"{{"roots":[0],"units":[{{"pkg_id":"path+file://root#demo@0.1.0","target":{{"name":"demo","kind":["lib"],"crate_types":["lib"],"src_path":"{}","edition":"2021"}},"mode":"build","features":["serde"],"deps":[]}}]}}"#,
                dir.path().join("src/lib.rs").display()
            )),
        };

        let mut plan_options = options(dir.path());
        plan_options.features.clear();
        plan_options.no_default_features = false;
        let receipt = capture_rust_plan_with_oracle(&plan_options, &oracle).unwrap();

        assert_eq!(receipt.schema_version, RECEIPT_SCHEMA_VERSION);
        assert!(receipt.invocation.features.is_empty());
        assert!(receipt.lockfile.blake3.len() >= 32);
        assert_eq!(receipt.package_count, 1);
        assert_eq!(receipt.packages[0].name, "demo");
        assert!(receipt.source_closure.ready);
        assert_eq!(receipt.source_closure.source_count, 1);
        assert_eq!(receipt.source_closure.sources[0].kind, SourceKind::Path);
        assert_eq!(receipt.source_closure.sources[0].source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(receipt.unit_graph.unit_count, 1);
        assert_eq!(receipt.unit_graph.root_count, 1);
        assert!(
            receipt.unit_derivation_graph.ready,
            "unit derivation graph blockers: {:?}",
            receipt.unit_derivation_graph.blockers
        );
        assert_eq!(receipt.unit_derivation_graph.derivation_count, 1);
        let derivation = &receipt.unit_derivation_graph.derivations[0];
        assert_eq!(derivation.target_kind, "lib");
        assert_eq!(derivation.derivation.builder, "rustc");
        assert!(derivation.derivation.args.contains(&"--crate-name".to_string()));
        assert!(derivation.derivation.args.contains(&"feature=\"serde\"".to_string()));
        assert_eq!(derivation.rustc_args_digest_blake3.len(), 64);
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn path_source_digest_ignores_root_metadata_without_hiding_source_changes() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("src/cairn")).unwrap();
        std::fs::create_dir_all(dir.path().join("cairn/evidence")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn demo() -> u32 { 1 }\n").unwrap();
        std::fs::write(dir.path().join("src/cairn/mod.rs"), "pub fn nested() -> u32 { 1 }\n").unwrap();
        std::fs::write(dir.path().join("cairn/evidence/proof.md"), "external proof A\n").unwrap();

        let original = hash_path_source_tree(dir.path()).unwrap();
        std::fs::write(dir.path().join("cairn/evidence/proof.md"), "external proof B\n").unwrap();
        let metadata_changed = hash_path_source_tree(dir.path()).unwrap();
        std::fs::write(dir.path().join("src/cairn/mod.rs"), "pub fn nested() -> u32 { 2 }\n").unwrap();
        let nested_source_changed = hash_path_source_tree(dir.path()).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn demo() -> u32 { 2 }\n").unwrap();
        let root_source_changed = hash_path_source_tree(dir.path()).unwrap();

        assert_eq!(original, metadata_changed);
        assert_ne!(original, nested_source_changed);
        assert_ne!(nested_source_changed, root_source_changed);
    }

    #[test]
    fn native_package_target_fragment_matches_supported_path_workspace() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let dep_dir = dir.path().join("dep-crate");
        std::fs::create_dir_all(app_dir.join("src/bin")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\", \"dep-crate\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", path = \"../dep-crate\" }\n\n[lib]\nname = \"app\"\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"app-cli\"\npath = \"src/bin/app-cli.rs\"\n",
        )
        .unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn answer() -> u32 { dep_crate::answer() }\n").unwrap();
        std::fs::write(app_dir.join("src/bin/app-cli.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app#app@0.1.0".to_string(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![
                    CargoTarget {
                        name: "app".to_string(),
                        kind: vec!["lib".to_string()],
                        src_path: app_dir.join("src/lib.rs").display().to_string(),
                    },
                    CargoTarget {
                        name: "app-cli".to_string(),
                        kind: vec!["bin".to_string()],
                        src_path: app_dir.join("src/bin/app-cli.rs").display().to_string(),
                    },
                ],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let planning = summarize_native_package_target_planning(
            dir.path(),
            &RustPlanOptions {
                features: Vec::new(),
                no_default_features: false,
                ..options(dir.path())
            },
            &empty_unit_graph(),
            &packages,
            &[
                "path+file://app#app@0.1.0".to_string(),
                "path+file://dep-crate#dep-crate@0.1.0".to_string(),
            ],
            &closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();

        assert!(planning.ready, "{:#?}", planning.blockers);
        assert_eq!(planning.comparison_status, "matched");
        assert_eq!(planning.packages.len(), 2);
        let app = planning.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.targets.len(), 2);
        assert_eq!(app.path_dependencies[0].name, "dep_crate");
        assert!(planning.non_claims.contains(&"not-full-cargo-compatibility".to_string()));
        assert_eq!(planning.digest_blake3.len(), 64);
    }

    #[test]
    fn native_package_oracle_comparison_accepts_supported_fixture() {
        let package_id = "path+native#app@0.1.0";
        let native = test_native_package(package_id, "app", "lib", Vec::new());
        let cargo = CargoPackage {
            id: package_id.to_string(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: native.manifest_path.clone(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        };
        let required = vec![&cargo];
        let mut blockers = Vec::new();

        compare_native_packages_to_cargo(&[native], std::slice::from_ref(&cargo), &required, &mut blockers);

        assert!(blockers.is_empty(), "{blockers:#?}");
    }

    #[test]
    fn native_package_oracle_comparison_blocks_identity_mismatch_and_missing_cargo_package() {
        let app_id = "path+native#app@0.1.0";
        let dep_id = "path+native#dep@0.1.0";
        let native = test_native_package(app_id, "app", "lib", Vec::new());
        let app_cargo = CargoPackage {
            id: app_id.to_string(),
            name: "app".to_string(),
            version: "0.2.0".to_string(),
            source: None,
            manifest_path: native.manifest_path.clone(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        };
        let dep_cargo = CargoPackage {
            id: dep_id.to_string(),
            name: "dep".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: "/test/dep/Cargo.toml".to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        };
        let all_cargo = vec![app_cargo, dep_cargo];
        let required = all_cargo.iter().collect::<Vec<_>>();
        let mut blockers = Vec::new();

        compare_native_packages_to_cargo(&[native], &all_cargo, &required, &mut blockers);
        let classes = blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("cargo-oracle-package-identity-mismatch"));
        assert!(classes.contains("native-missing-cargo-package"));
    }

    #[test]
    fn collect_native_manifest_lock_texts_reads_root_member_manifests_and_lockfile() {
        let dir = TempDir::new().unwrap();
        let member_dir = dir.path().join("member");
        std::fs::create_dir_all(member_dir.join("src")).unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = ['member']\n\n[workspace.package]\nversion = '0.1.0'\n",
        )
        .unwrap();
        std::fs::write(
            member_dir.join("Cargo.toml"),
            "[package]\nname = 'member'\nversion.workspace = true\n\n[lib]\npath = 'src/lib.rs'\n",
        )
        .unwrap();
        std::fs::write(member_dir.join("src/lib.rs"), "pub fn member() {}\n").unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "version = 3\n").unwrap();

        let inputs = collect_native_manifest_lock_texts(dir.path()).unwrap();

        assert_eq!(inputs.manifests.len(), 2usize);
        assert!(inputs.manifests.iter().any(|input| input.path.ends_with("Cargo.toml")));
        assert!(inputs.manifests.iter().any(|input| input.text.contains("name = 'member'")));
        assert!(inputs.lockfile.path.ends_with("Cargo.lock"));
        assert_eq!(inputs.lockfile.text, "version = 3\n");
    }

    #[test]
    fn collect_native_manifest_lock_texts_fails_closed_on_missing_lockfile() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = 'solo'\nversion = '0.1.0'\n").unwrap();

        let err = collect_native_manifest_lock_texts(dir.path()).unwrap_err();

        assert!(err.contains("reading native lockfile"));
        assert!(err.contains("Cargo.lock"));
    }

    #[test]
    fn native_manifest_lock_unsupported_blockers_accept_supported_subset() {
        let inputs = NativeManifestLockInputTexts {
            manifests: vec![NativeTextInput {
                path: "Cargo.toml".to_string(),
                text: "[package]\nname = 'app'\nversion = '0.1.0'\n\n[target.'cfg(unix)'.dependencies]\ndep = { path = '../dep' }\n".to_string(),
            }],
            lockfile: NativeTextInput {
                path: "Cargo.lock".to_string(),
                text: "version = 3\n\n[[package]]\nname = 'dep'\nversion = '0.1.0'\n".to_string(),
            },
        };

        let blockers = native_manifest_lock_unsupported_blockers(&inputs);

        assert!(blockers.is_empty(), "{blockers:#?}");
    }

    #[test]
    fn native_manifest_lock_unsupported_blockers_reject_patch_replace_target_build_and_unknown_lock_source() {
        let inputs = NativeManifestLockInputTexts {
            manifests: vec![NativeTextInput {
                path: "Cargo.toml".to_string(),
                text: "[package]\nname = 'app'\nversion = '0.1.0'\n\n[patch.crates-io]\nserde = { path = 'vendor/serde' }\n\n[replace]\n'old:0.1.0' = { path = 'vendor/old' }\n\n[target.'cfg(unix)'.build-dependencies]\nbuild = { path = '../build' }\n".to_string(),
            }],
            lockfile: NativeTextInput {
                path: "Cargo.lock".to_string(),
                text: "version = 3\n\n[[package]]\nname = 'weird'\nversion = '0.1.0'\nsource = 'sparse+https://example.invalid/index'\n".to_string(),
            },
        };

        let blockers = native_manifest_lock_unsupported_blockers(&inputs);
        let classes = blockers.iter().map(|blocker| blocker.class.as_str()).collect::<BTreeSet<_>>();

        assert!(classes.contains("unsupported-manifest-patch"));
        assert!(classes.contains("unsupported-manifest-replace"));
        assert!(classes.contains("unsupported-target-table"));
        assert!(classes.contains("unsupported-lockfile-source"));
    }

    #[test]
    fn parse_native_lockfile_text_extracts_source_identities_revisions_checksums_and_edges() {
        let facts = parse_native_lockfile_text(
            "fixtures/Cargo.lock",
            r#"
version = 3

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
  "git-dep 0.2.0 (git+https://example.invalid/git-dep?rev=main#abcdef123456)",
  "path-dep",
  "serde 1.0.228 (registry+https://github.com/rust-lang/crates.io-index)",
]

[[package]]
name = "git-dep"
version = "0.2.0"
source = "git+https://example.invalid/git-dep?rev=main#abcdef123456"

[[package]]
name = "path-dep"
version = "0.3.0"

[[package]]
name = "serde"
version = "1.0.228"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "111122223333444455556666777788889999aaaabbbbccccddddeeeeffff0000"
"#,
        )
        .unwrap();

        assert_eq!(facts.packages.len(), 4usize);
        let app = facts.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.dependencies.len(), 3usize);
        assert!(
            app.dependencies
                .iter()
                .any(|dependency| dependency.name == "path-dep" && dependency.version.is_none())
        );
        assert!(app.dependencies.iter().any(|dependency| {
            dependency.name == "serde"
                && dependency.version.as_deref() == Some("1.0.228")
                && dependency.source.as_deref() == Some("registry+https://github.com/rust-lang/crates.io-index")
        }));
        let git_dep = facts.packages.iter().find(|package| package.name == "git-dep").unwrap();
        assert_eq!(git_dep.revision.as_deref(), Some("abcdef123456"));
        let serde = facts.packages.iter().find(|package| package.name == "serde").unwrap();
        assert_eq!(serde.checksum.as_deref(), Some("111122223333444455556666777788889999aaaabbbbccccddddeeeeffff0000"));
    }

    #[test]
    fn parse_native_lockfile_text_rejects_malformed_package_records() {
        let err = parse_native_lockfile_text("bad/Cargo.lock", "[[package]]\nname = 'bad'\n").unwrap_err();

        assert!(err.contains("parsing lockfile bad/Cargo.lock"));
        assert!(err.contains("missing field `version`"));
    }

    #[test]
    fn parse_native_manifest_text_extracts_core_workspace_package_target_and_dependency_facts() {
        let manifest = parse_native_manifest_text(
            "fixtures/Cargo.toml",
            r#"
[workspace]
members = ["app", "crates/*"]

[workspace.package]
version = "1.2.3"
edition = "2024"

[workspace.dependencies]
serde = { version = "1", features = ["derive"], default-features = false }

[package]
name = "app"
version.workspace = true
edition.workspace = true
links = "app_native"

[dependencies]
local_dep = { path = "../local-dep" }
serde = { workspace = true }

[build-dependencies]
build_helper = { path = "../build-helper" }

[lib]
name = "app_core"
path = "src/lib.rs"

[[bin]]
name = "app-cli"
path = "src/bin/app.rs"

[target.'cfg(unix)'.dependencies]
unix_dep = { path = "../unix-dep" }
"#,
        )
        .unwrap();

        let package = manifest.package.as_ref().unwrap();
        let workspace = manifest.workspace.as_ref().unwrap();
        assert_eq!(package.name, "app");
        assert_eq!(package.links.as_deref(), Some("app_native"));
        assert!(package.version.workspace);
        assert!(package.edition.workspace);
        assert_eq!(workspace.members, vec!["app".to_string(), "crates/*".to_string()]);
        assert_eq!(workspace.package.version.as_deref(), Some("1.2.3"));
        assert_eq!(workspace.package.edition.as_deref(), Some("2024"));
        assert!(workspace.dependencies.contains_key("serde"));
        assert!(manifest.dependencies.contains_key("local_dep"));
        assert!(manifest.dependencies.contains_key("serde"));
        assert!(manifest.build_dependencies.contains_key("build_helper"));
        assert_eq!(manifest.lib.as_ref().unwrap().name.as_deref(), Some("app_core"));
        assert_eq!(manifest.bin[0].name.as_deref(), Some("app-cli"));
        assert!(manifest.target.contains_key("cfg(unix)"));
    }

    #[test]
    fn parse_native_manifest_text_rejects_invalid_inherited_package_version() {
        let err = parse_native_manifest_text(
            "bad/Cargo.toml",
            "[package]\nname = 'bad'\nversion = { workspace = true, unexpected = true }\n",
        )
        .unwrap_err();

        assert!(err.contains("parsing manifest bad/Cargo.toml"));
        assert!(err.contains("expected string or { workspace = true }"));
    }

    #[test]
    fn native_manifest_package_build_path_and_false_control_targets() {
        let dir = TempDir::new().unwrap();
        let package_dir = dir.path().join("linked");
        let disabled_dir = dir.path().join("disabled");
        std::fs::create_dir_all(package_dir.join("builder")).unwrap();
        std::fs::create_dir_all(package_dir.join("src")).unwrap();
        std::fs::create_dir_all(disabled_dir.join("src")).unwrap();
        std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname = \"linked\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"builder/main.rs\"\nlinks = \"linked_native\"\n",
        )
        .unwrap();
        std::fs::write(package_dir.join("builder/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(package_dir.join("src/lib.rs"), "pub fn linked() {}\n").unwrap();
        std::fs::write(
            disabled_dir.join("Cargo.toml"),
            "[package]\nname = \"disabled\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = false\n",
        )
        .unwrap();
        std::fs::write(disabled_dir.join("build.rs"), "fn main() {}\n").unwrap();
        std::fs::write(disabled_dir.join("src/lib.rs"), "pub fn disabled() {}\n").unwrap();
        let linked_manifest = read_native_manifest(&package_dir.join("Cargo.toml")).unwrap();
        let disabled_manifest = read_native_manifest(&disabled_dir.join("Cargo.toml")).unwrap();

        let linked_targets =
            native_targets_for_manifest(&package_dir, "linked", "0.1.0", "2021", &linked_manifest).unwrap();
        let disabled_targets =
            native_targets_for_manifest(&disabled_dir, "disabled", "0.1.0", "2021", &disabled_manifest).unwrap();

        assert_eq!(linked_manifest.package.as_ref().unwrap().links.as_deref(), Some("linked_native"));
        let build_target = linked_targets.iter().find(|target| target.kind == "custom-build").unwrap();
        assert_eq!(build_target.name, BUILD_SCRIPT_TARGET_NAME);
        assert!(build_target.source_path.ends_with("builder/main.rs"));
        assert!(disabled_targets.iter().all(|target| target.kind != "custom-build"));
    }

    #[test]
    fn native_manifest_declared_edition_feeds_target_derivation_args() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname = 'editioned'\nversion = '0.1.0'\nedition = '2024'\n").unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
        let package_id = "path+file://editioned#editioned@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "editioned".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "editioned".to_string(),
                kind: vec!["lib".to_string()],
                src_path: dir.path().join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let source_closure = summarize_source_closure(&packages, &[]).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[package_id.clone()],
            &source_closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let target = &package_planning.packages[0].targets[0];
        assert_eq!(target.edition, "2024");
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "editioned", "kind": ["lib"], "crate_types": ["lib"], "src_path": target.source_path.clone()},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });
        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert_eq!(native_units.units[0].edition, "2024");
        let graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();
        assert_eq!(rustc_edition_arg(&graph.derivations[0].derivation.args), "2024");
    }

    #[test]
    fn native_manifest_workspace_edition_feeds_target_derivation_args() {
        let dir = TempDir::new().unwrap();
        let member_dir = dir.path().join("member");
        let manifest_path = member_dir.join("Cargo.toml");
        std::fs::create_dir_all(member_dir.join("src")).unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = ['member']\n\n[workspace.package]\nedition = '2024'\n",
        )
        .unwrap();
        std::fs::write(
            &manifest_path,
            "[package]\nname = 'workspace-editioned'\nversion = '0.1.0'\nedition.workspace = true\n",
        )
        .unwrap();
        std::fs::write(member_dir.join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
        let package_id = "path+file://workspace-editioned#workspace-editioned@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "workspace-editioned".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "workspace-editioned".to_string(),
                kind: vec!["lib".to_string()],
                src_path: member_dir.join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let source_closure = summarize_source_closure(&packages, &[]).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[package_id.clone()],
            &source_closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let target = &package_planning.packages[0].targets[0];
        assert_eq!(target.edition, "2024");
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "workspace-editioned", "kind": ["lib"], "crate_types": ["lib"], "src_path": target.source_path.clone()},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });
        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert_eq!(native_units.units[0].edition, "2024");
        let graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();
        assert_eq!(rustc_edition_arg(&graph.derivations[0].derivation.args), "2024");
    }

    #[test]
    fn native_manifest_missing_edition_uses_cargo_default() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname = 'defaulted'\nversion = '0.1.0'\n").unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn value() -> u32 { 1 }\n").unwrap();
        let package_id = "path+file://defaulted#defaulted@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "defaulted".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "defaulted".to_string(),
                kind: vec!["lib".to_string()],
                src_path: dir.path().join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let source_closure = summarize_source_closure(&packages, &[]).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[package_id.clone()],
            &source_closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let target = &package_planning.packages[0].targets[0];
        assert_eq!(target.edition, DEFAULT_RUST_EDITION);
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "defaulted", "kind": ["lib"], "crate_types": ["lib"], "src_path": target.source_path.clone()},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });
        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert_eq!(native_units.units[0].edition, DEFAULT_RUST_EDITION);
        let graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();
        assert_eq!(rustc_edition_arg(&graph.derivations[0].derivation.args), DEFAULT_RUST_EDITION);
    }

    #[test]
    fn native_manifest_proc_macro_alias_feeds_target_planning() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            &manifest_path,
            "[package]\nname = 'spez'\nversion = '0.1.2'\nedition = '2021'\n\n[lib]\nproc_macro = true\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "extern crate proc_macro;\n").unwrap();
        let manifest = read_native_manifest(&manifest_path).unwrap();

        let targets = native_targets_for_manifest(dir.path(), "spez", "0.1.2", "2021", &manifest).unwrap();

        assert_eq!(targets.len(), 1usize);
        assert_eq!(targets[0].name, "spez");
        assert_eq!(targets[0].kind, "proc-macro");
    }

    #[test]
    fn native_manifest_declared_edition_feeds_host_derivation_args() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            &manifest_path,
            "[package]\nname = 'mac'\nversion = '0.1.0'\nedition = '2024'\n\n[lib]\nproc-macro = true\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "extern crate proc_macro;\n").unwrap();
        let package_id = "path+file://mac#mac@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "mac".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "mac".to_string(),
                kind: vec!["proc-macro".to_string()],
                src_path: dir.path().join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let source_closure = summarize_source_closure(&packages, &[]).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[package_id.clone()],
            &source_closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let target = &package_planning.packages[0].targets[0];
        assert_eq!(target.edition, "2024");
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "mac", "kind": ["proc-macro"], "crate_types": ["proc-macro"], "src_path": target.source_path.clone()},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });
        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        assert!(native_units.ready, "{:#?}", native_units.blockers);
        let native_hosts = summarize_native_host_unit_graph_planning(
            &unit_graph,
            &source_closure,
            &empty_registry_planning(),
            &package_planning,
            &native_units,
            &plan_options,
        )
        .unwrap();
        assert!(native_hosts.ready, "{:#?}", native_hosts.blockers);
        assert_eq!(native_hosts.host_units[0].edition, "2024");
        let graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            Some(&native_hosts),
        )
        .unwrap();
        let host_derivation = graph.derivations.iter().find(|derivation| derivation.execution_kind == "host").unwrap();
        assert_eq!(rustc_edition_arg(&host_derivation.derivation.args), "2024");
    }

    #[test]
    fn native_registry_source_planning_binds_declared_vendor_source() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let vendor_dir = dir.path().join("vendor");
        let dep_dir = vendor_dir.join("dep-crate-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join(".cargo")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", version = \"0.1.0\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { dep_crate::dep() }\n").unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn dep() -> u32 { 7 }\n").unwrap();
        let checksum = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        std::fs::write(dep_dir.join(".cargo-checksum.json"), format!(r#"{{"package":"{checksum}","files":{{}}}}"#))
            .unwrap();
        std::fs::write(
            dir.path().join(".cargo/config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
        )
        .unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: Some(registry_source.clone()),
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some(checksum.to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[app_id, dep_id],
            &source_closure,
            &registry_planning,
            &empty_git_planning(),
        )
        .unwrap();

        assert!(registry_planning.ready, "{:#?}", registry_planning.blockers);
        assert_eq!(registry_planning.sources.len(), 1);
        let registry_source = &registry_planning.sources[0];
        assert_eq!(registry_source.name, "dep-crate");
        assert_eq!(registry_source.checksum, checksum);
        assert_eq!(registry_source.source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(registry_source.source_digest.value.len(), 64);
        assert!(registry_planning.non_claims.contains(&"no-ambient-cargo-cache".to_string()));
        assert!(package_planning.ready, "{:#?}", package_planning.blockers);
        assert_eq!(package_planning.packages.len(), 2);
        let app = package_planning.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.path_dependencies[0].manifest_path, normalize_path_string(&dep_dir.join("Cargo.toml")));
    }

    #[test]
    fn no_cargo_capture_binds_declared_vendored_registry_source() {
        const DEP_CHECKSUM: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let dep_dir = dir.path().join("vendor-deps/dep-crate-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep-crate = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { dep_crate::dep() }\n").unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn dep() -> u32 { 7 }\n").unwrap();
        std::fs::write(dep_dir.join(".cargo-checksum.json"), format!(r#"{{"package":"{DEP_CHECKSUM}","files":{{}}}}"#))
            .unwrap();
        std::fs::write(
            dir.path().join("Cargo.lock"),
            format!(
                "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"dep-crate\",\n]\n\n[[package]]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{DEP_CHECKSUM}\"\n",
            ),
        )
        .unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            no_cargo_oracle: true,
            ..options(dir.path())
        };

        let receipt = capture_rust_plan(&plan_options).unwrap();

        assert!(receipt.cargo_mode.no_cargo_oracle);
        assert!(
            receipt.native_registry_source_planning.ready,
            "{:#?}",
            receipt.native_registry_source_planning.blockers
        );
        assert_eq!(receipt.native_registry_source_planning.sources.len(), 1);
        assert_eq!(receipt.native_registry_source_planning.sources[0].checksum, DEP_CHECKSUM);
        assert!(
            receipt.native_package_target_planning.ready,
            "{:#?}",
            receipt.native_package_target_planning.blockers
        );
        assert!(receipt.native_unit_graph_planning.ready, "{:#?}", receipt.native_unit_graph_planning.blockers);
        assert!(receipt.unit_derivation_graph.ready, "{:#?}", receipt.unit_derivation_graph.blockers);
    }

    #[test]
    fn no_cargo_capture_binds_captured_git_source() {
        const GIT_REVISION: &str = "abcdef123456";
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let git_dir = dir.path().join("vendor-deps/wu-manber-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(git_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nwu_manber = {{ package = \"wu-manber\", git = \"https://example.invalid/wu-manber.git\", rev = \"{GIT_REVISION}\" }}\n",
            ),
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { wu_manber::scan() }\n").unwrap();
        std::fs::write(
            git_dir.join("Cargo.toml"),
            "[package]\nname = \"wu-manber\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"wu_manber\"\npath = \"src/lib.rs\"\n",
        )
        .unwrap();
        std::fs::write(git_dir.join("src/lib.rs"), "pub fn scan() -> u32 { 7 }\n").unwrap();
        std::fs::write(
            dir.path().join("Cargo.lock"),
            format!(
                "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"wu-manber\",\n]\n\n[[package]]\nname = \"wu-manber\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/wu-manber.git#{GIT_REVISION}\"\n",
            ),
        )
        .unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            no_cargo_oracle: true,
            ..options(dir.path())
        };

        let receipt = capture_rust_plan(&plan_options).unwrap();

        assert!(receipt.cargo_mode.no_cargo_oracle);
        assert!(receipt.native_git_source_planning.ready, "{:#?}", receipt.native_git_source_planning.blockers);
        assert_eq!(receipt.native_git_source_planning.sources.len(), 1);
        let source = &receipt.native_git_source_planning.sources[0];
        assert_eq!(source.name, "wu-manber");
        assert_eq!(source.resolved_revision, GIT_REVISION);
        assert_eq!(source.manifest_path, normalize_path_string(&git_dir.join("Cargo.toml")));
        assert!(
            receipt.native_package_target_planning.ready,
            "{:#?}",
            receipt.native_package_target_planning.blockers
        );
        assert!(receipt.native_unit_graph_planning.ready, "{:#?}", receipt.native_unit_graph_planning.blockers);
        assert!(receipt.unit_derivation_graph.ready, "{:#?}", receipt.unit_derivation_graph.blockers);
    }

    #[test]
    fn no_cargo_capture_blocks_missing_vendored_registry_source() {
        const DEP_CHECKSUM: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep-crate = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { 1 }\n").unwrap();
        std::fs::write(
            dir.path().join("Cargo.lock"),
            format!(
                "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"dep-crate\",\n]\n\n[[package]]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{DEP_CHECKSUM}\"\n",
            ),
        )
        .unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            no_cargo_oracle: true,
            ..options(dir.path())
        };

        let receipt = capture_rust_plan(&plan_options).unwrap();

        assert!(receipt.cargo_mode.no_cargo_oracle);
        assert!(!receipt.native_registry_source_planning.ready);
        assert!(
            receipt
                .native_registry_source_planning
                .blockers
                .iter()
                .any(|blocker| blocker.class == "missing-declared-vendor-root")
        );
        assert!(!receipt.unit_derivation_graph.ready);
        assert!(receipt.cargo_mode.blockers.contains(&"native-package-target-planning-blocked".to_string()));
    }

    #[test]
    fn registry_dependency_source_uses_exact_version_when_names_repeat() {
        let dir = TempDir::new().unwrap();
        let impl_1_manifest = dir.path().join("thiserror-impl-1.0.69/Cargo.toml");
        let impl_2_manifest = dir.path().join("thiserror-impl-2.0.18/Cargo.toml");
        let mut registry = empty_registry_planning();
        registry.sources = vec![
            test_registry_source("thiserror-impl", "1.0.69", &impl_1_manifest),
            test_registry_source("thiserror-impl", "2.0.18", &impl_2_manifest),
        ];
        let dependency: toml::Value = toml::from_str("version = \"=2.0.18\"").unwrap();

        let selected = registry_dependency_source("thiserror-impl", &dependency, &registry).unwrap().unwrap();

        assert_eq!(selected.name, "thiserror-impl");
        assert_eq!(selected.version, "2.0.18");
        assert_eq!(selected.manifest_path, normalize_path_string(&impl_2_manifest));
        assert_ne!(selected.manifest_path, normalize_path_string(&impl_1_manifest));

        let prefix_dependency: toml::Value = toml::from_str("version = \"2.0\"").unwrap();
        let prefix_selected =
            registry_dependency_source("thiserror-impl", &prefix_dependency, &registry).unwrap().unwrap();
        assert_eq!(prefix_selected.version, "2.0.18");
        assert_eq!(prefix_selected.manifest_path, normalize_path_string(&impl_2_manifest));

        let string_dependency = toml::Value::String("2.0".to_string());
        let string_selected =
            registry_dependency_source("thiserror-impl", &string_dependency, &registry).unwrap().unwrap();
        assert_eq!(string_selected.version, "2.0.18");
        assert_eq!(string_selected.manifest_path, normalize_path_string(&impl_2_manifest));
    }

    #[test]
    fn registry_dependency_source_uses_default_caret_semver_compatibility() {
        let dir = TempDir::new().unwrap();
        let mut registry = empty_registry_planning();
        registry.sources = vec![
            test_registry_source("bitflags", "1.3.2", &dir.path().join("bitflags-1.3.2/Cargo.toml")),
            test_registry_source("bitflags", "2.11.0", &dir.path().join("bitflags-2.11.0/Cargo.toml")),
        ];
        let dependency: toml::Value = toml::from_str("version = \"2.4.0\"").unwrap();

        let selected = registry_dependency_source("bitflags", &dependency, &registry).unwrap().unwrap();

        assert_eq!(selected.name, "bitflags");
        assert_eq!(selected.version, "2.11.0");
    }

    #[test]
    fn registry_dependency_source_uses_highest_matching_comparator_range() {
        let dir = TempDir::new().unwrap();
        let mut registry = empty_registry_planning();
        registry.sources = vec![
            test_registry_source("getrandom", "0.2.17", &dir.path().join("getrandom-0.2.17/Cargo.toml")),
            test_registry_source("getrandom", "0.3.4", &dir.path().join("getrandom-0.3.4/Cargo.toml")),
            test_registry_source("getrandom", "0.4.2", &dir.path().join("getrandom-0.4.2/Cargo.toml")),
        ];
        let dependency: toml::Value = toml::from_str("version = \">=0.3.0, <0.5\"").unwrap();

        let selected = registry_dependency_source("getrandom", &dependency, &registry).unwrap().unwrap();

        assert_eq!(selected.name, "getrandom");
        assert_eq!(selected.version, "0.4.2");
    }

    #[test]
    fn registry_dependency_source_rejects_missing_exact_same_name_version() {
        let dir = TempDir::new().unwrap();
        let mut registry = empty_registry_planning();
        registry.sources = vec![
            test_registry_source("thiserror-impl", "1.0.69", &dir.path().join("thiserror-impl-1.0.69/Cargo.toml")),
            test_registry_source("thiserror-impl", "2.0.18", &dir.path().join("thiserror-impl-2.0.18/Cargo.toml")),
        ];
        let dependency: toml::Value = toml::from_str("version = \"=3.0.0\"").unwrap();

        let err = registry_dependency_source("thiserror-impl", &dependency, &registry).unwrap_err();

        assert_eq!(err.class, "missing-registry-dependency-version");
        assert!(err.message.contains("thiserror-impl"));
        assert!(err.message.contains("1.0.69"));
        assert!(err.message.contains("2.0.18"));
    }

    #[test]
    fn native_unit_graph_adds_transitive_registry_producer_unit() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let vendor_dir = dir.path().join("vendor");
        let dep_dir = vendor_dir.join("dep-crate-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join(".cargo")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", version = \"0.1.0\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { dep_crate::dep() }\n").unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn dep() -> u32 { 7 }\n").unwrap();
        let checksum = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        std::fs::write(dep_dir.join(".cargo-checksum.json"), format!(r#"{{"package":"{checksum}","files":{{}}}}"#))
            .unwrap();
        std::fs::write(
            dir.path().join(".cargo/config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
        )
        .unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: Some(registry_source.clone()),
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some(checksum.to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            std::slice::from_ref(&app_id),
            &source_closure,
            &registry_planning,
            &empty_git_planning(),
        )
        .unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(registry_planning.ready, "{:#?}", registry_planning.blockers);
        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert!(
            native_units
                .units
                .iter()
                .any(|unit| unit.package_id.contains("dep-crate@0.1.0") && unit.target_kind == "lib")
        );
        assert!(derivation_graph.ready, "{:#?}", derivation_graph.blockers);
        assert!(
            derivation_graph
                .derivations
                .iter()
                .any(|unit| unit.package_id.contains("dep-crate@0.1.0") && unit.target_kind == "lib")
        );
    }

    #[test]
    fn native_unit_graph_filters_package_self_dependency_from_lib_unit() {
        let dir = TempDir::new().unwrap();
        let package_id = "path+file://app#app@0.1.0".to_string();
        let mut package = test_native_package(&package_id, "app", "lib", Vec::new());
        package.targets.push(NativeTargetPlanningSummary {
            name: "app-bin".to_string(),
            kind: "bin".to_string(),
            crate_name: "app_bin".to_string(),
            source_path: "/test/app/src/main.rs".to_string(),
            edition: "2021".to_string(),
        });
        let package_planning = test_package_planning(vec![package]);
        let source_closure = test_source_closure(&package_planning.packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": package_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": package_id,
                    "target": {"name": "app-bin", "kind": ["bin"], "crate_types": ["bin"], "src_path": "/test/app/src/main.rs", "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": package_id, "extern_crate_name": "app"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        let lib_unit = native_units.units.iter().find(|unit| unit.target_kind == "lib").expect("lib unit planned");
        let bin_unit = native_units.units.iter().find(|unit| unit.target_kind == "bin").expect("bin unit planned");
        assert!(lib_unit.dependency_artifacts.is_empty());
        assert_eq!(bin_unit.dependency_artifacts, vec![test_dependency_artifact(&package_id, "app")]);
    }

    #[test]
    fn native_unit_graph_follows_native_dependency_facts_without_cargo_unit_edges() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let extra_id = "registry+https://github.com/rust-lang/crates.io-index#extra-crate@0.1.0".to_string();
        let dep_manifest = "/test/dep-crate/Cargo.toml".to_string();
        let extra_manifest = "/test/extra-crate/Cargo.toml".to_string();
        let app = test_native_package(&app_id, "app", "lib", vec![
            NativePathDependencySummary {
                name: "dep_crate".to_string(),
                manifest_path: dep_manifest.clone(),
            },
            NativePathDependencySummary {
                name: "extra_crate".to_string(),
                manifest_path: extra_manifest.clone(),
            },
        ]);
        let mut dep = test_native_package(&dep_id, "dep-crate", "lib", Vec::new());
        dep.manifest_path = dep_manifest;
        let mut extra = test_native_package(&extra_id, "extra-crate", "lib", Vec::new());
        extra.manifest_path = extra_manifest;
        let packages = vec![app, dep, extra];
        let source_closure = test_source_closure(&packages);
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert!(native_units.units.iter().any(|unit| unit.package_id == dep_id));
        assert!(native_units.units.iter().any(|unit| unit.package_id == extra_id));
        let app_unit = native_units.units.iter().find(|unit| unit.package_id == app_id).unwrap();
        assert_eq!(app_unit.dependency_artifacts.len(), 2);
        assert!(app_unit.dependency_artifacts.iter().any(|dependency| dependency.package_id == dep_id));
        assert!(app_unit.dependency_artifacts.iter().any(|dependency| dependency.package_id == extra_id));
    }

    #[test]
    fn native_unit_graph_uses_lib_crate_name_when_package_name_differs() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#new_debug_unreachable@1.0.6".to_string();
        let dep_manifest = "/test/new_debug_unreachable/Cargo.toml".to_string();
        let app = test_native_package(&app_id, "app", "lib", vec![NativePathDependencySummary {
            name: "new_debug_unreachable".to_string(),
            manifest_path: dep_manifest.clone(),
        }]);
        let mut dep = test_native_package(&dep_id, "new_debug_unreachable", "lib", Vec::new());
        dep.manifest_path = dep_manifest;
        dep.targets[0].name = "debug_unreachable".to_string();
        dep.targets[0].crate_name = "debug_unreachable".to_string();
        let packages = vec![app, dep];
        let source_closure = test_source_closure(&packages);
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        let app_unit = native_units.units.iter().find(|unit| unit.package_id == app_id).unwrap();
        assert_eq!(app_unit.dependency_artifacts.len(), 1usize);
        assert_eq!(app_unit.dependency_artifacts[0].name, "debug_unreachable");
        assert_eq!(app_unit.dependency_artifacts[0].artifact, format!("artifact:{dep_id}:debug_unreachable"));
    }

    #[test]
    fn native_unit_graph_adds_build_dependency_producer_unit() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let build_dep_id = "registry+https://github.com/rust-lang/crates.io-index#build-helper@0.1.0".to_string();
        let build_dep_manifest = "/test/build-helper/Cargo.toml".to_string();
        let mut app = test_native_package(&app_id, "app", "lib", Vec::new());
        app.build_dependencies.push(NativePathDependencySummary {
            name: "build_helper".to_string(),
            manifest_path: build_dep_manifest.clone(),
        });
        let mut build_dep = test_native_package(&build_dep_id, "build-helper", "lib", Vec::new());
        build_dep.manifest_path = build_dep_manifest;
        let packages = vec![app, build_dep];
        let source_closure = test_source_closure(&packages);
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert!(native_units.units.iter().any(|unit| unit.package_id == app_id));
        assert!(native_units.units.iter().any(|unit| unit.package_id == build_dep_id));
    }

    #[test]
    fn native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let mut dep = test_native_package(&dep_id, "dep-crate", "lib", Vec::new());
        dep.targets.push(NativeTargetPlanningSummary {
            name: "build-script-build".to_string(),
            kind: "custom-build".to_string(),
            crate_name: "build_script_build".to_string(),
            source_path: "/test/dep-crate/build.rs".to_string(),
            edition: "2021".to_string(),
        });
        let app = test_native_package(&app_id, "app", "lib", vec![NativePathDependencySummary {
            name: "dep_crate".to_string(),
            manifest_path: "/test/dep-crate/Cargo.toml".to_string(),
        }]);
        dep.manifest_path = "/test/dep-crate/Cargo.toml".to_string();
        let packages = vec![app, dep];
        let source_closure = test_source_closure(&packages);
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert!(native_units.units.iter().any(|unit| unit.package_id == dep_id && unit.target_kind == "lib"));
        let app_unit = native_units.units.iter().find(|unit| unit.package_id == app_id).unwrap();
        assert_eq!(app_unit.dependency_artifacts.len(), 1);
        assert_eq!(app_unit.dependency_artifacts[0].package_id, dep_id);
    }

    #[test]
    fn native_unit_graph_blocks_native_dependency_without_package_fact() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let packages = vec![test_native_package(&app_id, "app", "lib", vec![
            NativePathDependencySummary {
                name: "dep_crate".to_string(),
                manifest_path: "/test/dep-crate/Cargo.toml".to_string(),
            },
        ])];
        let source_closure = test_source_closure(&packages);
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(!native_units.ready);
        assert!(native_units.blockers.iter().any(|blocker| blocker.class == "unresolved-path-dependency-edge"));
        assert!(!derivation_graph.ready);
        assert!(derivation_graph.blockers.iter().any(|blocker| blocker.class == "unresolved-path-dependency-edge"));
    }

    #[test]
    fn native_unit_graph_blocks_native_dependency_without_source_fact() {
        let dir = TempDir::new().unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let dep_manifest = "/test/dep-crate/Cargo.toml".to_string();
        let app = test_native_package(&app_id, "app", "lib", vec![NativePathDependencySummary {
            name: "dep_crate".to_string(),
            manifest_path: dep_manifest.clone(),
        }]);
        let mut dep = test_native_package(&dep_id, "dep-crate", "lib", Vec::new());
        dep.manifest_path = dep_manifest;
        let packages = vec![app, dep];
        let mut source_closure = test_source_closure(&packages);
        source_closure.sources.retain(|source| source.package_id == app_id);
        source_closure.source_count = source_closure.sources.len();
        let package_planning = test_package_planning(packages);
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": "/test/app/src/lib.rs", "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(!native_units.ready);
        assert!(native_units.blockers.iter().any(|blocker| blocker.class == "missing-source-input"));
        assert!(!derivation_graph.ready);
        assert!(derivation_graph.blockers.iter().any(|blocker| blocker.class == "missing-source-input"));
    }

    #[test]
    fn native_unit_graph_blocks_registry_dependency_without_lib_producer() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let vendor_dir = dir.path().join("vendor");
        let dep_dir = vendor_dir.join("dep-crate-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join(".cargo")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", version = \"0.1.0\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { 1 }\n").unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"dep-crate\"\npath = \"src/main.rs\"\n",
        )
        .unwrap();
        std::fs::write(dep_dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let checksum = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        std::fs::write(dep_dir.join(".cargo-checksum.json"), format!(r#"{{"package":"{checksum}","files":{{}}}}"#))
            .unwrap();
        std::fs::write(
            dir.path().join(".cargo/config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
        )
        .unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: Some(registry_source.clone()),
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["bin".to_string()],
                    src_path: dep_dir.join("src/main.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some(checksum.to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            std::slice::from_ref(&app_id),
            &source_closure,
            &registry_planning,
            &empty_git_planning(),
        )
        .unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": app_id,
                "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": dep_id, "extern_crate_name": "dep_crate"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &source_closure, &package_planning, &plan_options)
                .unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &source_closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(registry_planning.ready, "{:#?}", registry_planning.blockers);
        assert!(!native_units.ready);
        assert!(native_units.blockers.iter().any(|blocker| blocker.class == "missing-native-dependency-producer"));
        assert!(!derivation_graph.ready);
        assert!(
            derivation_graph
                .blockers
                .iter()
                .any(|blocker| blocker.class == "missing-native-dependency-producer")
        );
    }

    #[test]
    fn native_registry_source_planning_blocks_missing_vendor_material() {
        let dir = TempDir::new().unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let packages = vec![CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string(),
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source.clone()),
            manifest_path: dir.path().join("ambient-cargo-cache/dep-crate-0.1.0/Cargo.toml").display().to_string(),
            targets: vec![CargoTarget {
                name: "dep-crate".to_string(),
                kind: vec!["lib".to_string()],
                src_path: dir.path().join("ambient-cargo-cache/dep-crate-0.1.0/src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };

        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();

        assert!(!registry_planning.ready);
        assert_eq!(registry_planning.comparison_status, "blocked");
        assert!(registry_planning.sources.is_empty());
        assert!(registry_planning.blockers.iter().any(|blocker| blocker.class == "missing-declared-vendor-root"));
    }

    #[test]
    fn native_git_source_planning_binds_captured_source_and_resolves_dependency_edge() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let git_dir = dir.path().join("git-checkout/wu-manber");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(git_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nwu_manber = { package = \"wu-manber\", git = \"https://example.invalid/wu-manber.git\", rev = \"abcdef123456\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { wu_manber::scan() }\n").unwrap();
        std::fs::write(
            git_dir.join("Cargo.toml"),
            "[package]\nname = \"wu-manber\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"wu_manber\"\npath = \"src/lib.rs\"\n",
        )
        .unwrap();
        std::fs::write(git_dir.join("src/lib.rs"), "pub fn scan() -> u32 { 7 }\n").unwrap();
        let git_source = "git+https://example.invalid/wu-manber.git#abcdef123456".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let git_id = "git+https://example.invalid/wu-manber.git#wu-manber@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: git_id.clone(),
                name: "wu-manber".to_string(),
                version: "0.1.0".to_string(),
                source: Some(git_source.clone()),
                manifest_path: git_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "wu-manber".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: git_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "wu-manber".to_string(),
            version: "0.1.0".to_string(),
            source: Some(git_source),
            checksum: None,
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let git_planning =
            summarize_native_git_source_planning(&packages, &lock_packages, &lockfile, &source_closure).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &empty_unit_graph(),
            &packages,
            &[app_id, git_id],
            &source_closure,
            &empty_registry_planning(),
            &git_planning,
        )
        .unwrap();

        assert!(git_planning.ready, "{:#?}", git_planning.blockers);
        assert_eq!(git_planning.sources.len(), 1);
        let git_source = &git_planning.sources[0];
        assert_eq!(git_source.name, "wu-manber");
        assert_eq!(git_source.resolved_revision, "abcdef123456");
        assert_eq!(git_source.source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(git_source.source_digest.value.len(), 64);
        assert!(git_planning.non_claims.contains(&"no-network-fetch".to_string()));
        assert!(git_planning.non_claims.contains(&"source-material-provider-agnostic".to_string()));
        assert!(git_planning.non_claims.contains(&"content-addressed-source-by-blake3".to_string()));
        assert_eq!(git_planning.source_closure_digest_blake3, source_closure.digest_blake3);
        assert!(package_planning.ready, "{:#?}", package_planning.blockers);
        let app = package_planning.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.path_dependencies[0].manifest_path, normalize_path_string(&git_dir.join("Cargo.toml")));
    }

    #[test]
    fn native_git_source_planning_blocks_missing_captured_manifest() {
        let dir = TempDir::new().unwrap();
        let git_source = "git+https://example.invalid/wu-manber.git#abcdef123456".to_string();
        let packages = vec![CargoPackage {
            id: "git+https://example.invalid/wu-manber.git#wu-manber@0.1.0".to_string(),
            name: "wu-manber".to_string(),
            version: "0.1.0".to_string(),
            source: Some(git_source.clone()),
            manifest_path: dir.path().join("missing/wu-manber/Cargo.toml").display().to_string(),
            targets: vec![CargoTarget {
                name: "wu-manber".to_string(),
                kind: vec!["lib".to_string()],
                src_path: dir.path().join("missing/wu-manber/src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let lock_packages = vec![LockPackage {
            name: "wu-manber".to_string(),
            version: "0.1.0".to_string(),
            source: Some(git_source),
            checksum: None,
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();

        let git_planning =
            summarize_native_git_source_planning(&packages, &lock_packages, &lockfile, &source_closure).unwrap();

        assert!(!git_planning.ready);
        assert_eq!(git_planning.comparison_status, "blocked");
        assert!(git_planning.sources.is_empty());
        assert!(git_planning.blockers.iter().any(|blocker| blocker.class == "missing-git-source-manifest"));
    }

    #[test]
    fn native_git_dependency_resolution_blocks_mismatched_revision() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let git_dir = dir.path().join("git-checkout/wu-manber");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(git_dir.join("src")).unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nwu_manber = { package = \"wu-manber\", git = \"https://example.invalid/wu-manber.git\", rev = \"deadbeef\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { wu_manber::scan() }\n").unwrap();
        std::fs::write(
            git_dir.join("Cargo.toml"),
            "[package]\nname = \"wu-manber\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"wu_manber\"\npath = \"src/lib.rs\"\n",
        )
        .unwrap();
        std::fs::write(git_dir.join("src/lib.rs"), "pub fn scan() -> u32 { 7 }\n").unwrap();
        let git_source = "git+https://example.invalid/wu-manber.git#abcdef123456".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let git_id = "git+https://example.invalid/wu-manber.git#wu-manber@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: git_id.clone(),
                name: "wu-manber".to_string(),
                version: "0.1.0".to_string(),
                source: Some(git_source.clone()),
                manifest_path: git_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "wu-manber".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: git_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "wu-manber".to_string(),
            version: "0.1.0".to_string(),
            source: Some(git_source),
            checksum: None,
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let git_planning =
            summarize_native_git_source_planning(&packages, &lock_packages, &lockfile, &source_closure).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &empty_unit_graph(),
            &packages,
            &[app_id, git_id],
            &source_closure,
            &empty_registry_planning(),
            &git_planning,
        )
        .unwrap();

        assert!(git_planning.ready, "{:#?}", git_planning.blockers);
        assert!(!package_planning.ready);
        assert!(package_planning.blockers.iter().any(|blocker| blocker.class == "git-dependency-reference-mismatch"));
        assert!(!package_planning.blockers.iter().any(|blocker| blocker.class == "unsupported-non-path-dependency"));
    }

    #[test]
    fn native_git_dev_dependency_resolution_blocks_mismatched_revision() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let git_dir = dir.path().join("git-checkout/wu-manber");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(git_dir.join("src")).unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dev-dependencies]\nwu_manber = { package = \"wu-manber\", git = \"https://example.invalid/wu-manber.git\", rev = \"deadbeef\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { 1 }\n").unwrap();
        std::fs::write(
            git_dir.join("Cargo.toml"),
            "[package]\nname = \"wu-manber\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"wu_manber\"\npath = \"src/lib.rs\"\n",
        )
        .unwrap();
        std::fs::write(git_dir.join("src/lib.rs"), "pub fn scan() -> u32 { 7 }\n").unwrap();
        let git_source = "git+https://example.invalid/wu-manber.git#abcdef123456".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let git_id = "git+https://example.invalid/wu-manber.git#wu-manber@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: git_id.clone(),
                name: "wu-manber".to_string(),
                version: "0.1.0".to_string(),
                source: Some(git_source.clone()),
                manifest_path: git_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "wu-manber".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: git_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "wu-manber".to_string(),
            version: "0.1.0".to_string(),
            source: Some(git_source),
            checksum: None,
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let git_planning =
            summarize_native_git_source_planning(&packages, &lock_packages, &lockfile, &source_closure).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &empty_unit_graph(),
            &packages,
            &[app_id, git_id],
            &source_closure,
            &empty_registry_planning(),
            &git_planning,
        )
        .unwrap();

        assert!(git_planning.ready, "{:#?}", git_planning.blockers);
        assert!(!package_planning.ready);
        assert!(package_planning.blockers.iter().any(|blocker| blocker.class == "git-dependency-reference-mismatch"));
        assert!(!package_planning.packages.iter().any(|package| {
            package.name == "app" && package.dev_dependencies.iter().any(|dep| dep.name == "wu_manber")
        }));
    }

    #[test]
    fn native_git_dependency_resolution_blocks_ambiguous_same_url_sources() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let git_dir_old = dir.path().join("git-checkout/wu-manber-old");
        let git_dir_new = dir.path().join("git-checkout/wu-manber-new");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(git_dir_old.join("src")).unwrap();
        std::fs::create_dir_all(git_dir_new.join("src")).unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nwu_manber = { package = \"wu-manber\", git = \"https://example.invalid/wu-manber.git\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { wu_manber::scan() }\n").unwrap();
        for (dir_path, version, body) in [
            (&git_dir_old, "0.1.0", "pub fn scan() -> u32 { 7 }\n"),
            (&git_dir_new, "0.2.0", "pub fn scan() -> u32 { 8 }\n"),
        ] {
            std::fs::write(
                dir_path.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"wu-manber\"\nversion = \"{version}\"\nedition = \"2021\"\n\n[lib]\nname = \"wu_manber\"\npath = \"src/lib.rs\"\n"
                ),
            )
            .unwrap();
            std::fs::write(dir_path.join("src/lib.rs"), body).unwrap();
        }
        let git_source_old = "git+https://example.invalid/wu-manber.git#abcdef123456".to_string();
        let git_source_new = "git+https://example.invalid/wu-manber.git#fedcba654321".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let git_id_old = "git+https://example.invalid/wu-manber.git#wu-manber@0.1.0".to_string();
        let git_id_new = "git+https://example.invalid/wu-manber.git#wu-manber@0.2.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: git_id_old.clone(),
                name: "wu-manber".to_string(),
                version: "0.1.0".to_string(),
                source: Some(git_source_old.clone()),
                manifest_path: git_dir_old.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "wu-manber".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: git_dir_old.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: git_id_new.clone(),
                name: "wu-manber".to_string(),
                version: "0.2.0".to_string(),
                source: Some(git_source_new.clone()),
                manifest_path: git_dir_new.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "wu-manber".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: git_dir_new.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![
            LockPackage {
                name: "wu-manber".to_string(),
                version: "0.1.0".to_string(),
                source: Some(git_source_old),
                checksum: None,
            },
            LockPackage {
                name: "wu-manber".to_string(),
                version: "0.2.0".to_string(),
                source: Some(git_source_new),
                checksum: None,
            },
        ];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let git_planning =
            summarize_native_git_source_planning(&packages, &lock_packages, &lockfile, &source_closure).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &empty_unit_graph(),
            &packages,
            &[app_id, git_id_old, git_id_new],
            &source_closure,
            &empty_registry_planning(),
            &git_planning,
        )
        .unwrap();

        assert!(git_planning.ready, "{:#?}", git_planning.blockers);
        assert_eq!(git_planning.sources.len(), 2);
        assert!(!package_planning.ready);
        assert!(
            package_planning
                .blockers
                .iter()
                .any(|blocker| blocker.class == "ambiguous-captured-git-source-fact")
        );
        assert!(!package_planning.blockers.iter().any(|blocker| blocker.class == "unsupported-non-path-dependency"));
    }

    #[test]
    fn native_unit_graph_fragment_feeds_supported_unit_derivations() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let dep_dir = dir.path().join("dep-crate");
        std::fs::create_dir_all(app_dir.join("src/bin")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\", \"dep-crate\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", path = \"../dep-crate\" }\n\n[lib]\nname = \"app\"\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"app-cli\"\npath = \"src/bin/app-cli.rs\"\n",
        )
        .unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn answer() -> u32 { dep_crate::answer() }\n").unwrap();
        std::fs::write(app_dir.join("src/bin/app-cli.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "path+file://dep-crate#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![
                    CargoTarget {
                        name: "app".to_string(),
                        kind: vec!["lib".to_string()],
                        src_path: app_dir.join("src/lib.rs").display().to_string(),
                    },
                    CargoTarget {
                        name: "app-cli".to_string(),
                        kind: vec!["bin".to_string()],
                        src_path: app_dir.join("src/bin/app-cli.rs").display().to_string(),
                    },
                ],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[app_id.clone(), dep_id.clone()],
            &closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": dep_id,
                    "target": {"name": "dep-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": dep_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": app_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                },
                {
                    "pkg_id": "path+file://app#app@0.1.0",
                    "target": {"name": "app-cli", "kind": ["bin"], "crate_types": ["bin"], "src_path": app_dir.join("src/bin/app-cli.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &closure, &package_planning, &plan_options).unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert_eq!(native_units.comparison_status, "matched");
        assert_eq!(native_units.units.len(), 3);
        assert!(native_units.non_claims.contains(&"cargo-unit-graph-retained-as-oracle-evidence-only".to_string()));
        assert!(derivation_graph.ready, "{:#?}", derivation_graph.blockers);
        assert_eq!(derivation_graph.derivation_count, 3);
        assert_eq!(derivation_graph.host_unit_count, 0);
        let app_bin = derivation_graph.derivations.iter().find(|unit| unit.target_name == "app-cli").unwrap();
        assert_eq!(app_bin.derivation.builder, "rustc");
        assert!(app_bin.derivation.args.contains(&"--extern".to_string()));
        assert_eq!(app_bin.derivation.inputs.iter().filter(|input| input.starts_with("source:")).count(), 1);
    }

    #[test]
    fn native_unit_graph_fragment_blocks_mismatch_and_missing_edges() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(&manifest_path, "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        std::fs::write(manifest_path.parent().unwrap().join("src/lib.rs"), "pub fn app() {}\n").unwrap();
        let package_id = "path+file://app#app@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "app".to_string(),
                kind: vec!["lib".to_string()],
                src_path: manifest_path.parent().unwrap().join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let mut package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &empty_unit_graph(),
            &packages,
            &[package_id.clone()],
            &closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();
        let missing_id = "path+file://missing#missing_dep@0.1.0".to_string();
        package_planning.packages[0].path_dependencies.push(NativePathDependencySummary {
            name: "unselected_dep".to_string(),
            manifest_path: dir.path().join("unselected/Cargo.toml").display().to_string(),
        });
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "app-renamed", "kind": ["lib"], "crate_types": ["lib"], "src_path": manifest_path.parent().unwrap().join("src/lib.rs").display().to_string(), "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": [{"pkg_id": missing_id, "extern_crate_name": "missing_dep"}]
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &closure, &package_planning, &plan_options).unwrap();

        assert!(!native_units.ready);
        assert_eq!(native_units.comparison_status, "blocked");
        assert!(native_units.blockers.iter().any(|blocker| blocker.class == "unresolved-path-dependency-edge"));
        assert!(native_units.blockers.iter().all(|blocker| blocker.class != "cargo-oracle-unit-graph-mismatch"));
    }

    #[test]
    fn native_package_target_fragment_ignores_out_of_scope_oracle_target_kinds() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("benchy");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"benchy\"]\n").unwrap();
        std::fs::write(crate_dir.join("Cargo.toml"), "[package]\nname = \"benchy\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://benchy#benchy@0.1.0".to_string(),
            name: "benchy".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: crate_dir.join("Cargo.toml").display().to_string(),
            targets: vec![CargoTarget {
                name: "benchy-bench".to_string(),
                kind: vec!["bench".to_string()],
                src_path: crate_dir.join("benches/benchy.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &empty_unit_graph(),
            &packages,
            &["path+file://benchy#benchy@0.1.0".to_string()],
            &closure,
            &empty_registry_planning(),
            &empty_git_planning(),
        )
        .unwrap();

        assert!(planning.ready, "{:#?}", planning.blockers);
        assert_eq!(planning.comparison_status, "matched");
        assert!(planning.blockers.iter().all(|blocker| blocker.class != "unsupported-cargo-oracle-target-kind"));
    }

    #[test]
    fn native_target_cfg_predicate_scope_evaluates_nested_common_predicates() {
        let target = "x86_64-unknown-linux-gnu";

        assert_eq!(evaluate_supported_target_cfg("cfg(unix)", target), Some(true));
        assert_eq!(evaluate_supported_target_cfg("cfg(windows)", target), Some(false));
        assert_eq!(evaluate_supported_target_cfg("cfg(not(windows))", target), Some(true));
        assert_eq!(
            evaluate_supported_target_cfg("cfg(any(target_os = \"linux\", target_os = \"macos\"))", target),
            Some(true)
        );
        assert_eq!(
            evaluate_supported_target_cfg(
                "cfg(all(any(target_arch = \"x86\", target_arch = \"x86_64\"), not(target_os = \"windows\")))",
                target
            ),
            Some(true)
        );
        assert_eq!(evaluate_supported_target_cfg("x86_64-unknown-linux-gnu", target), Some(true));
        assert_eq!(evaluate_supported_target_cfg("aarch64-pc-windows-gnullvm", target), Some(false));
        assert_eq!(evaluate_supported_target_cfg("cfg(target_has_atomic = \"ptr\")", target), Some(true));
        assert_eq!(evaluate_supported_target_cfg("cfg(not(target_has_atomic = \"ptr\"))", target), Some(false));
        assert_eq!(evaluate_supported_target_cfg("cfg(loom)", target), Some(false));
    }

    #[test]
    fn native_target_cfg_predicate_scope_blocks_unknown_or_malformed_syntax() {
        let target = "x86_64-unknown-linux-gnu";

        assert_eq!(evaluate_supported_target_cfg("cfg(unknown_selector)", target), None);
        assert_eq!(evaluate_supported_target_cfg("cfg(any(target_os = \"linux\"", target), None);
    }

    #[test]
    fn source_closure_records_registry_git_and_path_identities() {
        let dir = TempDir::new().unwrap();
        let path_manifest = dir.path().join("path-crate/Cargo.toml");
        std::fs::create_dir_all(path_manifest.parent().unwrap()).unwrap();
        std::fs::write(&path_manifest, "[package]\nname='path-crate'\nversion='0.1.0'\n").unwrap();
        let lockfile = r#"
[[package]]
name = "git-crate"
version = "0.2.0"
source = "git+https://example.invalid/repo?rev=main#abcdef123456"

[[package]]
name = "path-crate"
version = "0.1.0"

[[package]]
name = "registry-crate"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0123456789abcdef"
"#;
        std::fs::write(dir.path().join("Cargo.lock"), lockfile).unwrap();
        let packages = vec![
            CargoPackage {
                id: "git+https://example.invalid/repo?rev=main#abcdef123456#git-crate@0.2.0".to_string(),
                name: "git-crate".to_string(),
                version: "0.2.0".to_string(),
                source: Some("git+https://example.invalid/repo?rev=main#abcdef123456".to_string()),
                manifest_path: dir.path().join("git-crate/Cargo.toml").display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file:///path-crate#path-crate@0.1.0".to_string(),
                name: "path-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: path_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.2.3".to_string(),
                name: "registry-crate".to_string(),
                version: "1.2.3".to_string(),
                source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
                manifest_path: dir.path().join("registry-crate/Cargo.toml").display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &parse_lockfile_packages(dir.path()).unwrap()).unwrap();

        assert!(closure.ready);
        assert_eq!(closure.source_count, 3);
        assert_eq!(closure.sources[0].kind, SourceKind::Git);
        assert_eq!(closure.sources[0].resolved_revision.as_deref(), Some("abcdef123456"));
        assert_eq!(closure.sources[0].source_digest.algorithm, GIT_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.sources[1].kind, SourceKind::Path);
        assert_eq!(closure.sources[1].source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.sources[2].kind, SourceKind::Registry);
        assert_eq!(
            closure.sources[2].lockfile_identity.as_ref().unwrap().checksum.as_deref(),
            Some("0123456789abcdef")
        );
        assert_eq!(closure.sources[2].source_digest.algorithm, REGISTRY_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.digest_blake3.len(), 64);
    }

    #[test]
    fn source_closure_blocks_registry_without_checksum() {
        let package = CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#missing@1.0.0".to_string(),
            name: "missing".to_string(),
            version: "1.0.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            manifest_path: "/tmp/missing/Cargo.toml".to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        };

        let closure = summarize_source_closure(&[package], &[]).unwrap();

        assert!(!closure.ready);
        assert_eq!(closure.blockers[0].class, "missing-registry-checksum");
        assert_eq!(closure.sources[0].source_digest.algorithm, "missing");
    }

    #[test]
    fn unit_derivation_graph_emits_binary_with_dependency_artifact() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='app'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://app#app@0.1.0".to_string(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://app#app@0.1.0",
                "target": {
                    "name": "app-bin",
                    "kind": ["bin"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("app/src/main.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "features": ["cli"],
                "deps": [{
                    "pkg_id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
                    "extern_crate_name": "serde"
                }]
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 1);
        let unit = &graph.derivations[0];
        assert_eq!(unit.target_kind, "bin");
        assert_eq!(unit.derivation.name, "rust-unit-0-app_bin");
        assert!(unit.derivation.args.contains(&"--crate-type".to_string()));
        assert!(unit.derivation.args.contains(&"bin".to_string()));
        assert!(unit.derivation.args.contains(&"--extern".to_string()));
        assert_eq!(unit.dependency_artifacts[0].name, "serde");
        assert_eq!(unit.derivation.env.get("SOURCE_CLOSURE_DIGEST"), Some(&closure.digest_blake3));
        assert_eq!(graph.digest_blake3.len(), 64);
    }

    #[test]
    fn executes_first_supported_lib_unit_from_derivation_graph() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("exec-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='exec-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://exec-crate#exec-crate@0.1.0".to_string(),
            name: "exec-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://exec-crate#exec-crate@0.1.0",
                "target": {
                    "name": "exec-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "success");
        assert_eq!(receipt.rebuild_reason, "rebuilt-explicit-unit");
        assert_eq!(receipt.target_kind, "lib");
        assert_eq!(receipt.rustc_args_digest_blake3, graph.derivations[0].rustc_args_digest_blake3);
        assert_eq!(
            receipt.environment_digest_blake3,
            rust_derivation_env_digest(&graph.derivations[0].derivation.env).unwrap()
        );
        assert_eq!(receipt.environment_digest_blake3.len(), BLAKE3_HEX_CHARS);
        assert_eq!(receipt.declared_outputs, vec!["out".to_string()]);
        assert_eq!(receipt.toolchain.tool, "rustc");
        assert!(!receipt.toolchain.version_verbose.is_empty());
        assert_eq!(receipt.toolchain.version_digest_blake3.len(), 64);
        assert!(receipt.blocker.is_none());
        assert!(!receipt.output_artifact_digests.is_empty());
        assert!(receipt.output_artifact_digests.iter().any(|artifact| artifact.path.ends_with(".rlib")));
        assert!(receipt.output_artifact_digests.iter().all(|artifact| artifact.path.starts_with("declared-output/")));
        assert_eq!(receipt.receipt_hash.len(), 64);

        let repeated = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();
        assert_eq!(repeated.execution_status, "success");
        assert_eq!(repeated.rebuild_reason, "reused-explicit-unit-output");
        assert_eq!(repeated.declared_outputs, receipt.declared_outputs);
        assert_eq!(repeated.toolchain, receipt.toolchain);
        assert_eq!(repeated.environment_digest_blake3, receipt.environment_digest_blake3);
        assert_eq!(repeated.output_artifact_digests, receipt.output_artifact_digests);
        assert_eq!(repeated.receipt_hash.len(), BLAKE3_HEX_CHARS);

        let mut changed_env_graph = graph.clone();
        changed_env_graph.derivations[0]
            .derivation
            .env
            .insert("MANTLE_TEST_ENV".to_string(), "changed".to_string());
        let stale = execute_first_supported_rust_unit(&changed_env_graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();
        assert_eq!(stale.execution_status, "blocked");
        assert_eq!(stale.rebuild_reason, "not-run-stale-cached-output");
        assert_eq!(stale.blocker.as_ref().unwrap().class, "stale-cached-output");
        assert_ne!(stale.environment_digest_blake3, receipt.environment_digest_blake3);
    }

    #[test]
    fn executes_dependency_chain_from_produced_lib_artifact() {
        let dir = TempDir::new().unwrap();
        let dep_dir = dir.path().join("dep-crate");
        let app_dir = dir.path().join("app-crate");
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        let dep_manifest = dep_dir.join("Cargo.toml");
        let app_manifest = app_dir.join("Cargo.toml");
        std::fs::write(&dep_manifest, "[package]\nname='dep-crate'\nversion='0.1.0'\n").unwrap();
        std::fs::write(&app_manifest, "[package]\nname='app-crate'\nversion='0.1.0'\n").unwrap();
        let dep_lib = dep_dir.join("src/lib.rs");
        let app_main = app_dir.join("src/lib.rs");
        std::fs::write(&dep_lib, "pub fn answer() -> u32 { 42 }\n").unwrap();
        std::fs::write(&app_main, "pub fn call_dep() -> u32 { dep_crate::answer() }\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app-crate#app-crate@0.1.0".to_string(),
                name: "app-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://dep-crate#dep-crate@0.1.0",
                    "target": {"name": "dep-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": dep_lib.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": []
                },
                {
                    "pkg_id": "path+file://app-crate#app-crate@0.1.0",
                    "target": {"name": "app-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_main.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                }
            ]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        let consumer = graph
            .derivations
            .iter()
            .find(|unit| unit.package_id == "path+file://app-crate#app-crate@0.1.0")
            .unwrap();
        assert_eq!(
            consumer.dependency_artifacts[0].artifact,
            "artifact:path+file://dep-crate#dep-crate@0.1.0:dep_crate"
        );

        let receipt = execute_first_rust_unit_dependency_chain(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("chain-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "success", "{receipt:#?}");
        assert!(receipt.claim.contains("bounded explicit Rust dependency edge"));
        assert_eq!(receipt.unit_executions.len(), 2);
        assert_eq!(receipt.unit_executions[0].target_kind, "lib");
        assert_eq!(receipt.unit_executions[1].target_kind, "lib");
        assert_eq!(receipt.unit_executions[1].execution_status, "success");
        assert_eq!(receipt.unit_executions[1].dependency_artifact_digests.len(), 1);
        assert!(receipt.unit_executions[1].dependency_artifact_digests[0].path.ends_with(".rlib"));
        assert!(receipt.unit_executions[1].blocker.is_none());
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn dependency_chain_blocks_missing_producer_before_consumer_rustc() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app-crate");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        let app_manifest = app_dir.join("Cargo.toml");
        std::fs::write(&app_manifest, "[package]\nname='app-crate'\nversion='0.1.0'\n").unwrap();
        let app_main = app_dir.join("src/main.rs");
        std::fs::write(&app_main, "fn main() { let _ = missing_dep::answer(); }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://app-crate#app-crate@0.1.0".to_string(),
            name: "app-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: app_manifest.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://app-crate#app-crate@0.1.0",
                "target": {"name": "app-crate", "kind": ["bin"], "crate_types": ["bin"], "src_path": app_main.display().to_string(), "edition": "2021"},
                "mode": "build",
                "deps": [{"pkg_id": "path+file://missing-dep#missing-dep@0.1.0", "extern_crate_name": "missing_dep"}]
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);

        let receipt = execute_first_rust_unit_dependency_chain(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("chain-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert!(receipt.unit_executions.is_empty());
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-dependency-producer");
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn rust_unit_execution_blocks_missing_source_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("missing-source-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='missing-source-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://missing-source-crate#missing-source-crate@0.1.0".to_string(),
            name: "missing-source-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://missing-source-crate#missing-source-crate@0.1.0",
                "target": {
                    "name": "missing-source-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": crate_dir.join("src/lib.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.rebuild_reason, "not-run-preflight-blocker");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-source-material");
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_source_closure_blocker_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("registry-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='registry-crate'\nversion='1.0.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.0.0".to_string(),
            name: "registry-crate".to_string(),
            version: "1.0.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        assert!(!closure.ready);
        assert_eq!(closure.blockers[0].class, "missing-registry-checksum");
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.0.0",
                "target": {
                    "name": "registry-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(!graph.ready);
        assert_eq!(graph.blockers[0].class, "source-closure-blocked");

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.rebuild_reason, "not-run-preflight-blocker");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "unit-derivation-graph-blocked");
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_dependency_artifact_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("dep-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='dep-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://dep-crate#dep-crate@0.1.0",
                "target": {
                    "name": "dep-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": [{"pkg_id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0", "extern_crate_name": "serde"}]
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        assert_eq!(graph.derivations[0].dependency_artifacts[0].name, "serde");

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-dependency-artifact");
        assert!(receipt.blocker.as_ref().unwrap().message.contains("artifact:"));
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_host_artifact_before_rustc() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("hosted/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='hosted'\nversion='0.1.0'\n").unwrap();
        let app_lib = dir.path().join("hosted/src/lib.rs");
        let build_rs = dir.path().join("hosted/build.rs");
        std::fs::write(&app_lib, "pub fn answer() -> u32 { 42 }\n").unwrap();
        std::fs::write(&build_rs, "fn main() {}\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://hosted#hosted@0.1.0".to_string(),
            name: "hosted".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://hosted#hosted@0.1.0",
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": build_rs.display().to_string(), "edition": "2021"},
                    "mode": "build"
                },
                {
                    "pkg_id": "path+file://hosted#hosted@0.1.0",
                    "target": {"name": "hosted", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_lib.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": []
                }
            ]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        let target = graph.derivations.iter().find(|unit| unit.execution_kind == "target").unwrap();
        assert_eq!(target.dependency_artifacts.len(), 0);
        assert_eq!(target.consumed_host_artifacts.len(), 1);

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-host-artifact");
        assert!(receipt.blocker.as_ref().unwrap().message.contains("host-artifact:"));
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_declared_output_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("no-output-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='no-output-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://no-output-crate#no-output-crate@0.1.0".to_string(),
            name: "no-output-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://no-output-crate#no-output-crate@0.1.0",
                "target": {"name": "no-output-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": lib_path.display().to_string(), "edition": "2021"},
                "mode": "build",
                "deps": []
            }]
        });
        let mut graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        graph.derivations[0].derivation.outputs.clear();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-declared-output");
        assert_eq!(receipt.declared_outputs, Vec::<String>::new());
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn redacted_diagnostic_limits_lines_and_redacts_temp_paths() {
        let lines = (0..=REDACTED_DIAGNOSTIC_MAX_LINES)
            .map(|index| format!("line-{index}: /tmp/mantle-secret-{index}/src/lib.rs:1:1\0"))
            .collect::<Vec<_>>()
            .join("\n");

        let diagnostic = redacted_diagnostic(lines.as_bytes());

        assert!(diagnostic.contains(REDACTED_DIAGNOSTIC_TEMP_PATH));
        assert!(!diagnostic.contains("mantle-secret"));
        assert!(!diagnostic.contains('\0'));
        assert_eq!(diagnostic.lines().count(), REDACTED_DIAGNOSTIC_MAX_LINES);
    }

    #[test]
    fn redacted_diagnostic_preserves_non_temp_context() {
        let diagnostic = redacted_diagnostic(b"error: failed at /workspace/src/lib.rs\n");

        assert!(diagnostic.contains("/workspace/src/lib.rs"));
        assert!(!diagnostic.contains(REDACTED_DIAGNOSTIC_TEMP_PATH));
    }

    #[test]
    fn unit_derivation_graph_represents_build_script_host_units() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("build-crate/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='build-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://build-crate#build-crate@0.1.0".to_string(),
            name: "build-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://build-crate#build-crate@0.1.0",
                "target": {
                    "name": "build-script-build",
                    "kind": ["custom-build"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("build-crate/build.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 1);
        assert_eq!(graph.host_unit_count, 1);
        assert_eq!(graph.host_artifact_count, 1);
        let unit = &graph.derivations[0];
        assert_eq!(unit.execution_kind, "host");
        assert_eq!(unit.target_kind, "custom-build");
        let metadata = unit.generated_metadata.as_ref().unwrap();
        assert!(metadata.out_dir.contains("OUT_DIR"));
        assert_eq!(metadata.digest_blake3.len(), 64);
        assert!(unit.derivation.inputs.iter().any(|input| input.starts_with("source:")));
    }

    #[test]
    fn unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units() {
        let dir = TempDir::new().unwrap();
        let macro_manifest = dir.path().join("mac/Cargo.toml");
        let app_manifest = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(macro_manifest.parent().unwrap().join("src")).unwrap();
        std::fs::create_dir_all(app_manifest.parent().unwrap().join("src")).unwrap();
        std::fs::write(&macro_manifest, "[package]\nname='mac'\nversion='0.1.0'\n").unwrap();
        std::fs::write(&app_manifest, "[package]\nname='app'\nversion='0.1.0'\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app#app@0.1.0".to_string(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://mac#mac@0.1.0".to_string(),
                name: "mac".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: macro_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://mac#mac@0.1.0",
                    "target": {
                        "name": "mac",
                        "kind": ["proc-macro"],
                        "crate_types": ["proc-macro"],
                        "src_path": dir.path().join("mac/src/lib.rs").display().to_string(),
                        "edition": "2021"
                    },
                    "mode": "build"
                },
                {
                    "pkg_id": "path+file://app#app@0.1.0",
                    "target": {
                        "name": "app",
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "src_path": dir.path().join("app/src/lib.rs").display().to_string(),
                        "edition": "2021"
                    },
                    "mode": "build",
                    "deps": [{"pkg_id": "path+file://mac#mac@0.1.0", "extern_crate_name": "mac"}]
                }
            ]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 2);
        assert_eq!(graph.host_unit_count, 1);
        let target = graph.derivations.iter().find(|unit| unit.execution_kind == "target").unwrap();
        assert_eq!(target.consumed_host_artifacts.len(), 1);
        assert_eq!(target.consumed_host_artifacts[0].target_kind, "proc-macro");
        assert!(target.derivation.inputs.iter().any(|input| input.starts_with("host-artifact:")));
    }

    #[test]
    fn unit_derivation_graph_blocks_unsupported_target_kinds() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("example/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='example'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://example#example@0.1.0".to_string(),
            name: "example".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://example#example@0.1.0",
                "target": {
                    "name": "example-test",
                    "kind": ["test"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("example/tests/smoke.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(!graph.ready);
        assert_eq!(graph.derivation_count, 0);
        assert_eq!(graph.blockers[0].class, "unsupported-target-kind");
        assert!(graph.blockers[0].message.contains("lib/bin"));
    }

    #[test]
    fn unit_derivation_graph_blocks_doctest_or_non_build_modes() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("doc-crate/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='doc-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://doc-crate#doc-crate@0.1.0".to_string(),
            name: "doc-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://doc-crate#doc-crate@0.1.0",
                "target": {
                    "name": "doc_crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": dir.path().join("doc-crate/src/lib.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "doctest"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(!graph.ready);
        assert_eq!(graph.derivation_count, 0);
        assert_eq!(graph.blockers[0].class, "unsupported-unit-mode");
        assert!(graph.blockers[0].message.contains("doctest"));
    }

    #[test]
    fn unit_graph_failure_fails_closed() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "# lock\n").unwrap();
        let oracle = FixedOracle {
            cargo_version: ok_output("cargo 1.91.0\n"),
            rustc_version: ok_output("rustc 1.91.0\n"),
            metadata: ok_output(r#"{"packages":[],"workspace_root":"/tmp/demo","workspace_members":[]}"#),
            unit_graph: failing_output("the option `Z` is only accepted on nightly"),
        };

        let error = capture_rust_plan_with_oracle(&options(dir.path()), &oracle).unwrap_err();

        assert!(
            matches!(error, RunError::Build(message) if message.contains("cargo build --unit-graph") && message.contains("nightly"))
        );
    }
}
