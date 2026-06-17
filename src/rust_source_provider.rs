//! Shell/orchestration for the source-built Rust compiler/sysroot provider.
//!
//! This module performs filesystem work around the pure provider metadata
//! validator in `source_toolchain_closure`. It fails closed unless every
//! source-built stage reaches a validated final provider candidate; only then
//! does it copy that candidate into the requested provider output directory.

use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_METADATA_PATH;
use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
use crate::source_toolchain_closure::RustProviderArtifact;
use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
use crate::source_toolchain_closure::RustProviderObservedArtifact;
use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
use crate::source_toolchain_closure::RustProviderReceiptArtifact;
use crate::source_toolchain_closure::RustProviderReceiptStep;
use crate::source_toolchain_closure::RustProviderRole;
use crate::source_toolchain_closure::RustProviderSourceIdentity;
use crate::source_toolchain_closure::RustSourceProviderBootstrapOutput;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlanValidation;
use crate::source_toolchain_closure::RustSourceProviderBootstrapSource;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStage;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStageKind;
use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
use crate::source_toolchain_closure::RustSourceProviderMetadata;
use crate::source_toolchain_closure::RustSourceProviderProvenance;
use crate::source_toolchain_closure::RustSourceProviderValidation;
use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
use crate::source_toolchain_closure::ToolchainSourceKind;

pub(crate) const RUST_SOURCE_PROVIDER_BLOCKED_REASON: &str = "source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust";

const DIRECTORY_DIGEST_MAX_ENTRIES: usize = 1_000_000;
const RUST_SOURCE_PROVIDER_PLAN_FILE: &str = "rust-source-plan.ncl";
const FIRST_STAGE_PLAN_SCHEMA: &str = "mantle-rust-source-provider-first-stage-plan-v1";
const FIRST_STAGE_PLAN_FILE: &str = "mrustc-first-stage-plan.json";
const FIRST_STAGE_SOURCES_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-first-stage-sources-v1";
const FIRST_STAGE_SOURCES_MANIFEST_FILE: &str = "mrustc-first-stage-sources.json";
const FIRST_STAGE_BUILD_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-first-stage-build-v1";
const FIRST_STAGE_BUILD_MANIFEST_FILE: &str = "mrustc-first-stage-build.json";
const FIRST_STAGE_BUILD_LOG_FILE: &str = "mrustc-first-stage-build.log";
const FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA: &str = "mantle-rust-source-provider-first-stage-candidate-v1";
const FIRST_STAGE_PROVIDER_CANDIDATE_DIR: &str = "mrustc-first-stage-provider-candidate";
const FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE: &str = "mrustc-first-stage-provider-candidate.json";
const FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR: &str = "mrustc-first-stage-provider-candidate-smoke";
const FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_WORK_DIR: &str = "work";
const FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/mantle-rust-provider/receipts/build.json";
const FIRST_STAGE_PROVIDER_RECEIPT_ID: &str = "mrustc-first-stage-build";
const FIRST_STAGE_PROVIDER_RECEIPT_NAME: &str = "mrustc-first-stage-build-receipt";
const FIRST_STAGE_PROVIDER_BUILD_RECIPE: &str = "mrustc-first-stage-source-route";
const RUSTC_STAGE1_PLAN_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-plan-v1";
const RUSTC_STAGE1_PLAN_FILE: &str = "rustc-stage1-plan.json";
const RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-sources-v1";
const RUSTC_STAGE1_SOURCES_MANIFEST_FILE: &str = "rustc-stage1-sources.json";
const RUSTC_STAGE1_ARCHIVE_DIR: &str = "rustc-stage1-archives";
const RUSTC_STAGE1_SOURCE_DIR: &str = "rustc-stage1-sources";
const RUSTC_STAGE1_BUILD_DIR: &str = "rustc-stage1-build";
const RUSTC_STAGE1_OUTPUT_DIR: &str = "rustc-stage1-output";
const RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-build-v1";
const RUSTC_STAGE1_BUILD_MANIFEST_FILE: &str = "rustc-stage1-build.json";
const RUSTC_STAGE1_BUILD_LOG_FILE: &str = "rustc-stage1-build.log";
const RUSTC_STAGE1_SCRIPT_FILE: &str = "run-rustc-stage1.sh";
const RUSTC_STAGE1_CHAIN_DIR: &str = "rustc-stage1-chain";
const RUSTC_STAGE1_CHAIN_MAX_STAGES: usize = 8;
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-candidate-v1";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR: &str = "rustc-stage1-provider-candidate";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE: &str = "rustc-stage1-provider-candidate.json";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR: &str = "rustc-stage1-provider-candidate-smoke";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_WORK_DIR: &str = "work";
const RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/mantle-rust-provider/receipts/rustc-stage1-build.json";
const RUSTC_STAGE1_PROVIDER_RECEIPT_ID: &str = "rustc-stage1-build";
const RUSTC_STAGE1_PROVIDER_RECEIPT_NAME: &str = "rustc-stage1-build-receipt";
const RUSTC_STAGE1_PROVIDER_BUILD_RECIPE: &str = "rustc-stage1-source-route";
const RUSTC_STAGE1_SOURCE_BUILD_SCRIPT: &str = "mantle-rustc-stage1-build.sh";
const RUSTC_STAGE1_GENERATED_BUILD_SCRIPT: &str = "mantle-generated-rustc-stage1-build.sh";
const RUSTC_STAGE1_XPY_GOALS: &str = "install rustc cargo library/std";
const RUSTC_STAGE1_SHELL_PROGRAM: &str = "sh";
const RUSTC_FINAL_DIR: &str = "rustc-final";
const RUSTC_FINAL_PLAN_SCHEMA: &str = "mantle-rust-source-provider-rustc-final-plan-v1";
const RUSTC_FINAL_PLAN_FILE: &str = "rustc-final-plan.json";
const RUSTC_FINAL_SOURCES_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-final-sources-v1";
const RUSTC_FINAL_SOURCES_MANIFEST_FILE: &str = "rustc-final-sources.json";
const RUSTC_FINAL_ARCHIVE_DIR: &str = "rustc-final-archives";
const RUSTC_FINAL_SOURCE_DIR: &str = "rustc-final-sources";
const RUSTC_FINAL_BUILD_DIR: &str = "rustc-final-build";
const RUSTC_FINAL_OUTPUT_DIR: &str = "rustc-final-output";
const RUSTC_FINAL_BUILD_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-final-build-v1";
const RUSTC_FINAL_BUILD_MANIFEST_FILE: &str = "rustc-final-build.json";
const RUSTC_FINAL_BUILD_LOG_FILE: &str = "rustc-final-build.log";
const RUSTC_FINAL_SCRIPT_FILE: &str = "run-rustc-final.sh";
const RUSTC_FINAL_PROVIDER_CANDIDATE_SCHEMA: &str = "mantle-rust-source-provider-rustc-final-candidate-v1";
const RUSTC_FINAL_PROVIDER_CANDIDATE_DIR: &str = "rustc-final-provider-candidate";
const RUSTC_FINAL_PROVIDER_CANDIDATE_MANIFEST_FILE: &str = "rustc-final-provider-candidate.json";
const RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_DIR: &str = "rustc-final-provider-candidate-smoke";
const RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_WORK_DIR: &str = "work";
const RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/mantle-rust-provider/receipts/build.json";
const RUSTC_FINAL_PROVIDER_RECEIPT_ID: &str = "rust-final-build";
const RUSTC_FINAL_PROVIDER_RECEIPT_NAME: &str = "rust-final-build-receipt";
const RUSTC_FINAL_PROVIDER_BUILD_RECIPE: &str = "rust-final-source-route";
const RUSTC_FINAL_SOURCE_BUILD_SCRIPT: &str = "mantle-rustc-final-build.sh";
const RUSTC_FINAL_GENERATED_BUILD_SCRIPT: &str = "mantle-generated-rustc-final-build.sh";
const RUSTC_FINAL_XPY_GOALS: &str = "install rustc cargo library/std";
const RUSTC_FINAL_SHELL_PROGRAM: &str = "sh";
const RUSTC_SOURCE_XPY_SCRIPT: &str = "x.py";
const RUSTC_SOURCE_BOOTSTRAP_MANIFEST: &str = "src/bootstrap/Cargo.toml";
const RUSTC_SOURCE_CRANELIFT_MANIFEST: &str = "compiler/rustc_codegen_cranelift/Cargo.toml";
const RUSTC_SOURCE_CODEGEN_GCC_MANIFEST: &str = "compiler/rustc_codegen_gcc/Cargo.toml";
const RUSTC_SOURCE_GENERATED_CONFIG_FILE: &str = "mantle-rust-build-config.toml";
const RUSTC_SOURCE_GENERATED_CARGO_HOME_DIR: &str = "cargo-home";
const RUSTC_SOURCE_TARGET_CC_VAR: &str = "MANTLE_TARGET_CC";
const RUSTC_SOURCE_TARGET_CXX_VAR: &str = "MANTLE_TARGET_CXX";
const RUSTC_SOURCE_TARGET_AR_VAR: &str = "MANTLE_TARGET_AR";
const RUSTC_SOURCE_TARGET_RANLIB_VAR: &str = "MANTLE_TARGET_RANLIB";
const RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR: &str = "MANTLE_TARGET_MUSL_ROOT";
const FIRST_STAGE_SCRIPT_FILE: &str = "run-mrustc-first-stage.sh";
const FIRST_STAGE_ARCHIVE_DIR: &str = "archives";
const FIRST_STAGE_SOURCE_DIR: &str = "sources";
const FIRST_STAGE_BUILD_DIR: &str = "build";
const FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION: &str = "tar.gz";
const FIRST_STAGE_TAR_GZ_EXTENSION: &str = ".tar.gz";
const FIRST_STAGE_TGZ_EXTENSION: &str = ".tgz";
const FIRST_STAGE_MISSING_SOURCE_EXIT_CODE: i32 = 2;
const FIRST_STAGE_BUILD_FAILED_EXIT_CODE: i32 = 3;
const RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE: i32 = 4;
const FIRST_STAGE_FETCH_TIMEOUT_SECS: u64 = 600;
const FIRST_STAGE_FETCH_MAX_RETRIES: u32 = 3;
const FIRST_STAGE_FETCH_RETRY_BASE_DELAY_MS: u64 = 2_000;
const FIRST_STAGE_FETCH_MAX_MIB: u64 = 768;
const BYTES_PER_KIB: u64 = 1_024;
const BYTES_PER_MIB: u64 = BYTES_PER_KIB * BYTES_PER_KIB;
const FIRST_STAGE_FETCH_MAX_BYTES: u64 = FIRST_STAGE_FETCH_MAX_MIB * BYTES_PER_MIB;
const FIRST_STAGE_ARCHIVE_MAX_ENTRIES: u32 = 500_000;
const FIRST_STAGE_MRUSTC_SOURCE_PREFIX: &str = "mrustc-";
const FIRST_STAGE_RUST_SOURCE_PREFIX: &str = "rust-";
const FIRST_STAGE_MRUSTC_BINARY: &str = "bin/mrustc";
const FIRST_STAGE_MINICARGO_BINARY: &str = "bin/minicargo";
const FIRST_STAGE_TRANSLATED_RUSTC_BINARY: &str = "output/rustc";
const FIRST_STAGE_TRANSLATED_CARGO_BINARY: &str = "output/cargo";
const FIRST_STAGE_RUN_RUSTC_DIR: &str = "run_rustc";
const FIRST_STAGE_PREFIX_DIR: &str = "run_rustc/output/prefix";
const FIRST_STAGE_PREFIX_RUSTC_BINARY: &str = "run_rustc/output/prefix/bin/rustc";
const FIRST_STAGE_PREFIX_CARGO_BINARY: &str = "run_rustc/output/prefix/bin/cargo";
const FIRST_STAGE_MINICARGO_MAKEFILE: &str = "minicargo.mk";
const FIRST_STAGE_MAKEFILE: &str = "Makefile";
const FIRST_STAGE_WORKSPACE_BOUNDARY_FILE: &str = "Cargo.toml";
const FIRST_STAGE_PROC_MACRO_WORKSPACE_MEMBER: &str = "lib/libproc_macro";
const FIRST_STAGE_MINICARGO_WORKSPACE_RESOLVER: &str = "2";
const FIRST_STAGE_MAKE_PROGRAM: &str = "make";
const FIRST_STAGE_COPY_PROGRAM: &str = "cp";
const FIRST_STAGE_PKG_CONFIG_PROGRAM: &str = "pkg-config";
const FIRST_STAGE_CMAKE_PROGRAM: &str = "cmake";
const FIRST_STAGE_CC_PROGRAM: &str = "cc";
const FIRST_STAGE_CXX_PROGRAM: &str = "c++";
const FIRST_STAGE_MUSL_TRIPLE: &str = "x86_64-unknown-linux-musl";
const FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX: &str = "x86_64-linux-musl";
const FIRST_STAGE_TARGET_CC_PROGRAM: &str = "x86_64-unknown-linux-musl-gcc";
const FIRST_STAGE_TARGET_CXX_PROGRAM: &str = "x86_64-unknown-linux-musl-g++";
const FIRST_STAGE_TARGET_AR_PROGRAM: &str = "x86_64-unknown-linux-musl-ar";
const FIRST_STAGE_TARGET_RANLIB_PROGRAM: &str = "x86_64-unknown-linux-musl-ranlib";
const FIRST_STAGE_TARGET_MUSL_TOOL_PREFIXES: &[&str] = &[FIRST_STAGE_MUSL_TRIPLE, FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX];
const FIRST_STAGE_TARGET_MUSL_MACHINE_ALIASES: &[&str] = FIRST_STAGE_TARGET_MUSL_TOOL_PREFIXES;
const FIRST_STAGE_TARGET_LINKER_ALIAS_DIR: &str = "target-linker-bin";
const FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR: &str = "target-linker-runtime";
const FIRST_STAGE_STATIC_MUSL_DYLIB_EXT: &str = "rlib";
const FIRST_STAGE_TARGET_OUTDIR_SUFFIX: &str = "-target";
const FIRST_STAGE_TARGET_PREFIX_S_DIR: &str = "run_rustc/output$target_outdir_suffix/prefix-s";
const FIRST_STAGE_TARGET_NIX_CC_WRAPPER_HOST_ROLE_VAR: &str = "NIX_CC_WRAPPER_TARGET_HOST_x86_64_unknown_linux_musl";
const FIRST_STAGE_TARGET_NIX_SUPPORT_DIR: &str = "nix-support";
const FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE: &str = "orig-libc";
const FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE: &str = "orig-cc";
const FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE: &str = "_LARGEFILE64_SOURCE";
const FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG: &str = "-fno-asynchronous-unwind-tables";
const FIRST_STAGE_TARGET_MRUSTC_CC_ENV_VAR: &str = "CC_x86_64_linux_musl";
const FIRST_STAGE_TARGET_CARGO_CC_ENV_VAR: &str = "CC_x86_64_unknown_linux_musl";
const FIRST_STAGE_TARGET_CARGO_CXX_ENV_VAR: &str = "CXX_x86_64_unknown_linux_musl";
const FIRST_STAGE_TARGET_CARGO_AR_ENV_VAR: &str = "AR_x86_64_unknown_linux_musl";
const FIRST_STAGE_TARGET_CARGO_RANLIB_ENV_VAR: &str = "RANLIB_x86_64_unknown_linux_musl";
const FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_SOURCE: &str = "musl-lfs-compat.c";
const FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT: &str = "musl-lfs-compat.o";
const FIRST_STAGE_HOST_GNU_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const FIRST_STAGE_CXXFLAGS: &str = "-g0 -O2 -D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_NONE";
const FIRST_STAGE_MAKE_FALLBACK_GLOB: &str = "/nix/store/*-gnumake-*/bin/make /nix/store/*-gnumake-static-*/bin/make";
const FIRST_STAGE_CMAKE_FALLBACK_GLOB: &str = "/nix/store/*-cmake-*/bin/cmake /nix/store/*-cmake-minimal-*/bin/cmake";
const FIRST_STAGE_GCC_FALLBACK_GLOB: &str =
    "/nix/store/*-bootstrap-stage*-gcc-wrapper-*/bin/g++ /nix/store/*-gcc-wrapper-*/bin/g++";
const FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB: &str =
    "/nix/store/*-x86_64-unknown-linux-musl-gcc-wrapper-*/bin/x86_64-unknown-linux-musl-gcc";
const FIRST_STAGE_ZLIB_PKG_CONFIG_FALLBACK_GLOB: &str =
    "/nix/store/*-zlib-*-dev/lib/pkgconfig /nix/store/*-libz-*-dev/lib/pkgconfig";
const FIRST_STAGE_ENV_SCRUB_VARS: &[&str] = &[
    "CARGO",
    "CARGO_BIN_NAME",
    "CARGO_CFG_LINUX",
    "CARGO_CFG_TARGET_ABI",
    "CARGO_CFG_TARGET_ARCH",
    "CARGO_CFG_TARGET_ENDIAN",
    "CARGO_CFG_TARGET_ENV",
    "CARGO_CFG_TARGET_FAMILY",
    "CARGO_CFG_TARGET_HAS_ATOMIC",
    "CARGO_CFG_TARGET_HAS_ATOMIC_EQUAL_ALIGNMENT",
    "CARGO_CFG_TARGET_HAS_ATOMIC_LOAD_STORE",
    "CARGO_CFG_TARGET_OS",
    "CARGO_CFG_TARGET_POINTER_WIDTH",
    "CARGO_CFG_TARGET_VENDOR",
    "CARGO_CFG_UNIX",
    "CARGO_CFG_WINDOWS",
    "CARGO_CRATE_NAME",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_HOME",
    "CARGO_MANIFEST_DIR",
    "CARGO_MANIFEST_PATH",
    "CARGO_PKG_AUTHORS",
    "CARGO_PKG_DESCRIPTION",
    "CARGO_PKG_HOMEPAGE",
    "CARGO_PKG_LICENSE",
    "CARGO_PKG_LICENSE_FILE",
    "CARGO_PKG_NAME",
    "CARGO_PKG_README",
    "CARGO_PKG_REPOSITORY",
    "CARGO_PKG_RUST_VERSION",
    "CARGO_PKG_VERSION",
    "CARGO_PKG_VERSION_MAJOR",
    "CARGO_PKG_VERSION_MINOR",
    "CARGO_PKG_VERSION_PATCH",
    "CARGO_PKG_VERSION_PRE",
    "CARGO_PRIMARY_PACKAGE",
    "CARGO_TARGET_DIR",
    "DEBUG",
    "HOST",
    "MRUSTC_LIBDIR",
    "NUM_JOBS",
    "OPT_LEVEL",
    "OUT_DIR",
    "PROFILE",
    "RUSTC",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTC_WRAPPER",
    "RUSTDOC",
    "RUSTFLAGS",
    "TARGET",
];
const PROVIDER_RUSTC_RELATIVE_PATH: &str = "bin/rustc";
const PROVIDER_CARGO_RELATIVE_PATH: &str = "bin/cargo";
const PROVIDER_RUSTDOC_RELATIVE_PATH: &str = "bin/rustdoc";
const PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH: &str = "bin/rustc_binary";
const PROVIDER_RUSTLIB_PREFIX: &str = "lib/rustlib";
const MRUSTC_RUSTC_WRAPPER_DIRNAME_MARKER: &str = "dirname $0";
const MRUSTC_RUSTC_WRAPPER_BINARY_MARKER: &str = "rustc_binary";
const REQUIRED_FIRST_STAGE_OUTPUT_ROLES: [RustProviderRole; 4] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
const REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES: [RustProviderRole; 4] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
const REQUIRED_RUSTC_FINAL_STAGE_OUTPUT_ROLES: [RustProviderRole; 6] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::Rustdoc,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
    RustProviderRole::ProviderReceipt,
];
const REQUIRED_RUSTC_FINAL_BUILD_OUTPUT_ROLES: [RustProviderRole; 5] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::Rustdoc,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
const FIRST_STAGE_PROVIDER_RECEIPT_ARTIFACT_COUNT: usize = 1;
const RUSTC_STAGE1_PROVIDER_RECEIPT_ARTIFACT_COUNT: usize = 1;
const RUSTC_FINAL_PROVIDER_RECEIPT_ARTIFACT_COUNT: usize = 1;
const SMOKE_OUTPUT_CAPTURE_BYTES: usize = 16_384;
const SMOKE_SOURCE_FILE: &str = "mantle-rust-provider-smoke.rs";
const SMOKE_OUTPUT_FILE: &str = "libmantle_rust_provider_smoke.rlib";
const SMOKE_CRATE_NAME: &str = "mantle_rust_provider_smoke";
const SMOKE_RETURN_CODE: u32 = 42;
const SMOKE_EVIDENCE_SCHEMA: &str = "mantle-rust-source-provider-smoke-evidence-v1";
const SMOKE_EVIDENCE_SUMMARY_FILE: &str = "smoke.json";
const SMOKE_EVIDENCE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_EVIDENCE_STDERR_FILE: &str = "smoke-stderr.txt";
const SMOKE_EVIDENCE_SOURCE_FILE: &str = "smoke-source.rs";
const SMOKE_EVIDENCE_OUTPUT_FILE: &str = "smoke-output.rlib";
const SMOKE_EVIDENCE_METADATA_FILE: &str = "provider-metadata.json";
const RUST_PROVIDER_ARTIFACT_TEXT_PROBE_BYTES: usize = 4_096;
const RUST_PROVIDER_ARTIFACT_MARKER_SCAN_BYTES: usize = 65_536;

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderMaterialization {
    pub(crate) output_path: PathBuf,
    pub(crate) recipe_digest_blake3: String,
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderDirectoryValidation {
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) metadata: RustSourceProviderMetadata,
    pub(crate) validation: RustSourceProviderValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderImport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) validation: RustSourceProviderDirectoryValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderSmoke {
    pub(crate) provider_path: PathBuf,
    pub(crate) scratch_dir: PathBuf,
    pub(crate) rustc_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) output_digest_blake3: String,
    pub(crate) target_triple: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderSmokeEvidence {
    pub(crate) evidence_dir: PathBuf,
    pub(crate) summary_path: PathBuf,
    pub(crate) stdout_path: PathBuf,
    pub(crate) stderr_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) metadata_path: PathBuf,
    pub(crate) output_digest_blake3: String,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) policy_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderMaterializationPlan {
    recipe_path: PathBuf,
    output_dir: PathBuf,
    scratch_dir: PathBuf,
    recipe_digest_blake3: String,
    route_plan_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedRustSourceProviderRoute {
    plan_path: PathBuf,
    plan_digest_blake3: String,
    plan: RustSourceProviderBootstrapPlan,
    validation: RustSourceProviderBootstrapPlanValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderFirstStageBoundary {
    route_plan_path: PathBuf,
    route_plan_digest_blake3: String,
    route_policy_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    rust_version: String,
    host_triple: String,
    target_triple: String,
    mrustc_target_version: String,
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    archive_dir: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    sources_manifest_path: PathBuf,
    build_manifest_path: PathBuf,
    build_log_path: PathBuf,
    provider_candidate_dir: PathBuf,
    provider_candidate_manifest_path: PathBuf,
    provider_candidate_smoke_dir: PathBuf,
    provider_candidate_smoke_work_dir: PathBuf,
    script_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderRustcStage1Boundary {
    route_plan_path: PathBuf,
    route_plan_digest_blake3: String,
    route_policy_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    bootstrap_stage_id: String,
    rust_version: String,
    host_triple: String,
    target_triple: String,
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    bootstrap_provider_candidate_dir: PathBuf,
    bootstrap_provider_metadata_digest_blake3: String,
    bootstrap_provider_policy_digest_blake3: String,
    archive_dir: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    stage_output_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    sources_manifest_path: PathBuf,
    build_manifest_path: PathBuf,
    build_log_path: PathBuf,
    script_path: PathBuf,
    provider_candidate_dir: PathBuf,
    provider_candidate_manifest_path: PathBuf,
    provider_candidate_smoke_dir: PathBuf,
    provider_candidate_smoke_work_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageSourceAcquisition {
    id: String,
    url: String,
    archive_path: PathBuf,
    archive_sha256_hex: String,
    extracted_path: PathBuf,
    extracted_digest_blake3: String,
    archive_entry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageBuild {
    stage_id: String,
    script_path: PathBuf,
    log_path: PathBuf,
    mrustc_path: PathBuf,
    mrustc_digest_blake3: String,
    minicargo_path: PathBuf,
    minicargo_digest_blake3: String,
    translated_rustc_path: PathBuf,
    translated_rustc_digest_blake3: String,
    translated_cargo_path: PathBuf,
    translated_cargo_digest_blake3: String,
    prefix_path: PathBuf,
    prefix_digest_blake3: String,
    prefix_rustlib_path: PathBuf,
    prefix_rustlib_digest_blake3: String,
    target_prefix_rustlib_path: PathBuf,
    target_prefix_rustlib_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
    artifact_count: usize,
    source_count: usize,
    receipt_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderBootstrapProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustcStage1ScratchLayout {
    TopLevel,
    Scoped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcStage1Build {
    stage_id: String,
    script_path: PathBuf,
    log_path: PathBuf,
    stage_output_dir: PathBuf,
    stage_output_digest_blake3: String,
    rustc_path: PathBuf,
    rustc_digest_blake3: String,
    cargo_path: PathBuf,
    cargo_digest_blake3: String,
    host_rustlib_path: PathBuf,
    host_rustlib_digest_blake3: String,
    target_rustlib_path: PathBuf,
    target_rustlib_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcStage1ProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
    artifact_count: usize,
    source_count: usize,
    receipt_count: usize,
}

#[derive(Debug, Clone)]
struct RustSourceProviderRustcStage1ProviderCandidateRun {
    boundary: RustSourceProviderRustcStage1Boundary,
    candidate: RustSourceProviderRustcStage1ProviderCandidate,
    smoke: RustSourceProviderSmokeEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderRustcFinalBoundary {
    route_plan_path: PathBuf,
    route_plan_digest_blake3: String,
    route_policy_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    bootstrap_stage_id: String,
    rust_version: String,
    host_triple: String,
    target_triple: String,
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    bootstrap_provider_candidate_dir: PathBuf,
    bootstrap_provider_metadata_digest_blake3: String,
    bootstrap_provider_policy_digest_blake3: String,
    archive_dir: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    stage_output_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    sources_manifest_path: PathBuf,
    build_manifest_path: PathBuf,
    build_log_path: PathBuf,
    script_path: PathBuf,
    provider_candidate_dir: PathBuf,
    provider_candidate_manifest_path: PathBuf,
    provider_candidate_smoke_dir: PathBuf,
    provider_candidate_smoke_work_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcFinalBuild {
    stage_id: String,
    script_path: PathBuf,
    log_path: PathBuf,
    stage_output_dir: PathBuf,
    stage_output_digest_blake3: String,
    rustc_path: PathBuf,
    rustc_digest_blake3: String,
    cargo_path: PathBuf,
    cargo_digest_blake3: String,
    rustdoc_path: PathBuf,
    rustdoc_digest_blake3: String,
    host_rustlib_path: PathBuf,
    host_rustlib_digest_blake3: String,
    target_rustlib_path: PathBuf,
    target_rustlib_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcFinalProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
    artifact_count: usize,
    source_count: usize,
    receipt_count: usize,
}

#[derive(Debug, Clone)]
struct RustSourceProviderRustcFinalProviderCandidateRun {
    boundary: RustSourceProviderRustcFinalBoundary,
    candidate: RustSourceProviderRustcFinalProviderCandidate,
    smoke: RustSourceProviderSmokeEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderFirstStageBuildSources {
    mrustc_id: String,
    rust_id: String,
    mrustc_path: PathBuf,
    rust_path: PathBuf,
    mrustc_archive_path: PathBuf,
    rust_archive_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderSmokePlan {
    provider_dir: PathBuf,
    scratch_dir: PathBuf,
    rustc_path: PathBuf,
    source_path: PathBuf,
    output_path: PathBuf,
    target_triple: String,
}

#[derive(Debug)]
pub(crate) enum RustSourceProviderError {
    Read(String),
    Parse(String),
    Validate(String),
    MissingArtifact(String),
    Digest(String),
    Fetch(String),
    Extract(String),
    Build(String),
    Copy(String),
    Smoke(String),
    Blocked {
        recipe_path: PathBuf,
        recipe_digest_blake3: String,
        route_plan_path: Option<PathBuf>,
        route_plan_digest_blake3: Option<String>,
        first_stage_id: Option<String>,
        first_stage_script_path: Option<PathBuf>,
        final_candidate_stage_id: Option<String>,
        final_candidate_dir: Option<PathBuf>,
        final_candidate_manifest_path: Option<PathBuf>,
        final_candidate_metadata_digest_blake3: Option<String>,
        final_candidate_smoke_summary_path: Option<PathBuf>,
        reason: &'static str,
    },
}

impl std::fmt::Display for RustSourceProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(message) => write!(formatter, "read: {message}"),
            Self::Parse(message) => write!(formatter, "parse: {message}"),
            Self::Validate(message) => write!(formatter, "validate: {message}"),
            Self::MissingArtifact(message) => write!(formatter, "missing artifact: {message}"),
            Self::Digest(message) => write!(formatter, "digest: {message}"),
            Self::Fetch(message) => write!(formatter, "fetch: {message}"),
            Self::Extract(message) => write!(formatter, "extract: {message}"),
            Self::Build(message) => write!(formatter, "build: {message}"),
            Self::Copy(message) => write!(formatter, "copy: {message}"),
            Self::Smoke(message) => write!(formatter, "smoke: {message}"),
            Self::Blocked {
                recipe_path,
                recipe_digest_blake3,
                route_plan_path,
                route_plan_digest_blake3,
                first_stage_id,
                first_stage_script_path,
                final_candidate_stage_id,
                final_candidate_dir,
                final_candidate_manifest_path,
                final_candidate_metadata_digest_blake3,
                final_candidate_smoke_summary_path,
                reason,
            } => {
                write!(
                    formatter,
                    "{reason}; recipe={} recipe_digest_blake3={recipe_digest_blake3}",
                    recipe_path.display()
                )?;
                if let Some(route_plan_path) = route_plan_path {
                    write!(formatter, " route_plan={}", route_plan_path.display())?;
                }
                if let Some(route_plan_digest_blake3) = route_plan_digest_blake3 {
                    write!(formatter, " route_plan_digest_blake3={route_plan_digest_blake3}")?;
                }
                if let Some(first_stage_id) = first_stage_id {
                    write!(formatter, " first_stage_id={first_stage_id}")?;
                }
                if let Some(first_stage_script_path) = first_stage_script_path {
                    write!(formatter, " first_stage_script={}", first_stage_script_path.display())?;
                }
                if let Some(final_candidate_stage_id) = final_candidate_stage_id {
                    write!(formatter, " final_candidate_stage_id={final_candidate_stage_id}")?;
                }
                if let Some(final_candidate_dir) = final_candidate_dir {
                    write!(formatter, " final_candidate_dir={}", final_candidate_dir.display())?;
                }
                if let Some(final_candidate_manifest_path) = final_candidate_manifest_path {
                    write!(formatter, " final_candidate_manifest={}", final_candidate_manifest_path.display())?;
                }
                if let Some(final_candidate_metadata_digest_blake3) = final_candidate_metadata_digest_blake3 {
                    write!(
                        formatter,
                        " final_candidate_metadata_digest_blake3={final_candidate_metadata_digest_blake3}"
                    )?;
                }
                if let Some(final_candidate_smoke_summary_path) = final_candidate_smoke_summary_path {
                    write!(
                        formatter,
                        " final_candidate_smoke_summary={}",
                        final_candidate_smoke_summary_path.display()
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RustSourceProviderError {}

pub(crate) fn materialize_rust_source_provider(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    materialize_rust_source_provider_with_route_plan(recipe_path, None, output_dir, scratch_dir, verbose)
}

pub(crate) fn materialize_rust_source_provider_with_route_plan(
    recipe_path: &Path,
    route_plan_path: Option<&Path>,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    let recipe_bytes = fs::read(recipe_path)
        .map_err(|err| RustSourceProviderError::Read(format!("recipe {}: {err}", recipe_path.display())))?;
    if recipe_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("recipe {} is empty", recipe_path.display())));
    }
    let recipe_digest_blake3 = blake3::hash(&recipe_bytes).to_hex().to_string();
    let plan = plan_materialization(recipe_path, route_plan_path, output_dir, scratch_dir, &recipe_digest_blake3)?;
    let route = load_rust_source_provider_route(&plan.route_plan_path)?;
    let boundary = prepare_first_stage_boundary(&plan, &route)?;
    write_first_stage_boundary(&boundary)?;
    let first_stage_sources = acquire_first_stage_sources(&boundary, verbose)?;
    write_first_stage_sources_manifest(&boundary, &first_stage_sources)?;
    let first_stage_build = run_first_stage_build(&boundary, verbose)?;
    write_first_stage_build_manifest(&boundary, &first_stage_build)?;
    let first_stage_candidate =
        assemble_first_stage_provider_candidate(&boundary, &route, &first_stage_sources, &first_stage_build)?;
    let first_stage_candidate_smoke = smoke_first_stage_provider_candidate(&boundary, &first_stage_candidate)?;
    write_first_stage_provider_candidate_manifest(&boundary, &first_stage_candidate)?;
    let first_stage_bootstrap_candidate = bootstrap_candidate_from_first_stage(&first_stage_candidate);
    let rustc_stage1_runs =
        run_rustc_stage1_provider_candidate_chain(&plan, &route, &first_stage_bootstrap_candidate, verbose)?;
    let rustc_final_bootstrap_candidate = bootstrap_candidate_from_rustc_stage1(
        &rustc_stage1_runs
            .last()
            .ok_or_else(|| RustSourceProviderError::Validate("rustc stage1 chain produced no candidates".to_string()))?
            .candidate,
    );
    let rustc_final_run = run_rustc_final_provider_candidate(&plan, &route, &rustc_final_bootstrap_candidate, verbose)?;
    let materialized = promote_rustc_final_provider_candidate(&plan, &rustc_final_run)?;
    if let Err(err) =
        write_rustc_final_provider_candidate_manifest(&rustc_final_run.boundary, &rustc_final_run.candidate, true)
    {
        let _ = remove_import_output(&materialized.output_path);
        return Err(err);
    }
    if verbose {
        emit_rust_source_materialization_progress(
            &plan,
            &boundary,
            &first_stage_candidate,
            &first_stage_candidate_smoke,
            &rustc_stage1_runs,
            &rustc_final_run,
        );
    }
    Ok(materialized)
}

fn promote_rustc_final_provider_candidate(
    materialization: &RustSourceProviderMaterializationPlan,
    run: &RustSourceProviderRustcFinalProviderCandidateRun,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    if run.candidate.stage_id != run.boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc final candidate stage expected {}, got {}",
            run.boundary.stage_id, run.candidate.stage_id
        )));
    }
    if run.candidate.candidate_dir != run.boundary.provider_candidate_dir {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc final candidate dir expected {}, got {}",
            run.boundary.provider_candidate_dir.display(),
            run.candidate.candidate_dir.display()
        )));
    }
    if materialization.output_dir != run.boundary.output_dir {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source provider output expected {}, got {}",
            materialization.output_dir.display(),
            run.boundary.output_dir.display()
        )));
    }
    let candidate_validation = validate_materialized_rust_source_provider(&run.candidate.candidate_dir)?;
    if candidate_validation.metadata_digest_blake3 != run.candidate.metadata_digest_blake3 {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc final candidate metadata digest expected {}, got {}",
            run.candidate.metadata_digest_blake3, candidate_validation.metadata_digest_blake3
        )));
    }
    copy_provider_tree(&run.candidate.candidate_dir, &materialization.output_dir)?;
    let output_validation = match validate_materialized_rust_source_provider(&materialization.output_dir) {
        Ok(validation) => validation,
        Err(err) => {
            let _ = remove_import_output(&materialization.output_dir);
            return Err(err);
        }
    };
    if output_validation.metadata_digest_blake3 != run.candidate.metadata_digest_blake3 {
        let _ = remove_import_output(&materialization.output_dir);
        return Err(RustSourceProviderError::Validate(format!(
            "materialized provider metadata digest expected {}, got {}",
            run.candidate.metadata_digest_blake3, output_validation.metadata_digest_blake3
        )));
    }
    Ok(RustSourceProviderMaterialization {
        output_path: materialization.output_dir.clone(),
        recipe_digest_blake3: materialization.recipe_digest_blake3.clone(),
        metadata_path: output_validation.metadata_path,
        metadata_digest_blake3: output_validation.metadata_digest_blake3,
    })
}

fn emit_rust_source_materialization_progress(
    plan: &RustSourceProviderMaterializationPlan,
    boundary: &RustSourceProviderFirstStageBoundary,
    first_stage_candidate: &RustSourceProviderFirstStageProviderCandidate,
    first_stage_candidate_smoke: &RustSourceProviderSmokeEvidence,
    rustc_stage1_runs: &[RustSourceProviderRustcStage1ProviderCandidateRun],
    rustc_final_run: &RustSourceProviderRustcFinalProviderCandidateRun,
) {
    eprintln!("Rust source provider recipe: {}", plan.recipe_path.display());
    eprintln!("  recipe_digest_blake3: {}", plan.recipe_digest_blake3);
    eprintln!("  route_plan: {}", boundary.route_plan_path.display());
    eprintln!("  route_plan_digest_blake3: {}", boundary.route_plan_digest_blake3);
    eprintln!("  route_policy_digest_blake3: {}", boundary.route_policy_digest_blake3);
    eprintln!("  planned_output: {}", plan.output_dir.display());
    eprintln!("  planned_scratch: {}", plan.scratch_dir.display());
    emit_first_stage_progress(boundary, first_stage_candidate, first_stage_candidate_smoke);
    for run in rustc_stage1_runs {
        emit_rustc_stage1_progress(run);
    }
    emit_rustc_final_progress(rustc_final_run);
}

fn emit_first_stage_progress(
    boundary: &RustSourceProviderFirstStageBoundary,
    candidate: &RustSourceProviderFirstStageProviderCandidate,
    smoke: &RustSourceProviderSmokeEvidence,
) {
    eprintln!("  first_stage_id: {}", boundary.stage_id);
    eprintln!("  first_stage_script: {}", boundary.script_path.display());
    eprintln!("  first_stage_plan: {}", boundary.plan_path.display());
    eprintln!("  first_stage_sources: {}", boundary.sources_manifest_path.display());
    eprintln!("  first_stage_build: {}", boundary.build_manifest_path.display());
    eprintln!("  first_stage_build_log: {}", boundary.build_log_path.display());
    eprintln!("  first_stage_provider_candidate: {}", candidate.candidate_dir.display());
    eprintln!("  first_stage_provider_candidate_manifest: {}", boundary.provider_candidate_manifest_path.display());
    eprintln!("  first_stage_provider_candidate_smoke: {}", smoke.summary_path.display());
}

fn emit_rustc_stage1_progress(run: &RustSourceProviderRustcStage1ProviderCandidateRun) {
    eprintln!("  rustc_stage1_id: {}", run.boundary.stage_id);
    eprintln!("  rustc_stage1_plan: {}", run.boundary.plan_path.display());
    eprintln!("  rustc_stage1_sources: {}", run.boundary.sources_manifest_path.display());
    eprintln!("  rustc_stage1_build: {}", run.boundary.build_manifest_path.display());
    eprintln!("  rustc_stage1_build_log: {}", run.boundary.build_log_path.display());
    eprintln!("  rustc_stage1_provider_candidate: {}", run.candidate.candidate_dir.display());
    eprintln!(
        "  rustc_stage1_provider_candidate_manifest: {}",
        run.boundary.provider_candidate_manifest_path.display()
    );
    eprintln!("  rustc_stage1_provider_candidate_smoke: {}", run.smoke.summary_path.display());
}

fn emit_rustc_final_progress(run: &RustSourceProviderRustcFinalProviderCandidateRun) {
    eprintln!("  rustc_final_id: {}", run.boundary.stage_id);
    eprintln!("  rustc_final_plan: {}", run.boundary.plan_path.display());
    eprintln!("  rustc_final_sources: {}", run.boundary.sources_manifest_path.display());
    eprintln!("  rustc_final_build: {}", run.boundary.build_manifest_path.display());
    eprintln!("  rustc_final_build_log: {}", run.boundary.build_log_path.display());
    eprintln!("  rustc_final_provider_candidate: {}", run.candidate.candidate_dir.display());
    eprintln!(
        "  rustc_final_provider_candidate_manifest: {}",
        run.boundary.provider_candidate_manifest_path.display()
    );
    eprintln!("  rustc_final_provider_candidate_smoke: {}", run.smoke.summary_path.display());
}

pub(crate) fn import_rust_source_provider(
    import_dir: &Path,
    output_dir: &Path,
) -> Result<RustSourceProviderImport, RustSourceProviderError> {
    validate_import_request(import_dir, output_dir)?;
    validate_materialized_rust_source_provider(import_dir)?;
    copy_provider_tree(import_dir, output_dir)?;
    let validation = match validate_materialized_rust_source_provider(output_dir) {
        Ok(validation) => validation,
        Err(err) => {
            let _ = remove_import_output(output_dir);
            return Err(err);
        }
    };
    Ok(RustSourceProviderImport {
        input_path: import_dir.to_path_buf(),
        output_path: output_dir.to_path_buf(),
        validation,
    })
}

pub(crate) fn smoke_rust_source_provider(
    provider_dir: &Path,
    scratch_dir: &Path,
) -> Result<RustSourceProviderSmoke, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(provider_dir)?;
    let plan = plan_smoke(provider_dir, scratch_dir, &validation.metadata)?;
    fs::create_dir_all(&plan.scratch_dir).map_err(|err| {
        RustSourceProviderError::Smoke(format!("create smoke scratch {}: {err}", plan.scratch_dir.display()))
    })?;
    let source = smoke_source_text();
    fs::write(&plan.source_path, source).map_err(|err| {
        RustSourceProviderError::Smoke(format!("write smoke source {}: {err}", plan.source_path.display()))
    })?;
    let output = Command::new(&plan.rustc_path)
        .env_clear()
        .arg("--sysroot")
        .arg(&plan.provider_dir)
        .arg("--target")
        .arg(&plan.target_triple)
        .arg("--crate-name")
        .arg(SMOKE_CRATE_NAME)
        .arg("--crate-type")
        .arg("lib")
        .arg("--emit")
        .arg("link")
        .arg(&plan.source_path)
        .arg("-o")
        .arg(&plan.output_path)
        .output()
        .map_err(|err| RustSourceProviderError::Smoke(format!("launch {}: {err}", plan.rustc_path.display())))?;
    let stdout = bounded_output_text(&output.stdout);
    let stderr = bounded_output_text(&output.stderr);
    if !output.status.success() {
        return Err(RustSourceProviderError::Smoke(format!(
            "rustc smoke failed with status {}; stdout={stdout:?}; stderr={stderr:?}",
            output.status
        )));
    }
    if !plan.output_path.is_file() {
        return Err(RustSourceProviderError::Smoke(format!(
            "smoke output {} was not created",
            plan.output_path.display()
        )));
    }
    let output_digest_blake3 = file_digest_blake3(&plan.output_path)?;
    Ok(RustSourceProviderSmoke {
        provider_path: plan.provider_dir,
        scratch_dir: plan.scratch_dir,
        rustc_path: plan.rustc_path,
        source_path: plan.source_path,
        output_path: plan.output_path,
        output_digest_blake3,
        target_triple: plan.target_triple,
        stdout,
        stderr,
    })
}

pub(crate) fn persist_rust_source_provider_smoke_evidence(
    evidence_dir: &Path,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    if evidence_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("smoke evidence dir is empty".to_string()));
    }
    fs::create_dir_all(evidence_dir).map_err(|err| {
        RustSourceProviderError::Smoke(format!("create smoke evidence dir {}: {err}", evidence_dir.display()))
    })?;
    let evidence = smoke_evidence_paths(evidence_dir, smoke, validation);
    copy_evidence_file(&smoke.source_path, &evidence.source_path)?;
    copy_evidence_file(&smoke.output_path, &evidence.output_path)?;
    copy_evidence_file(&validation.metadata_path, &evidence.metadata_path)?;
    write_evidence_text(&evidence.stdout_path, &smoke.stdout)?;
    write_evidence_text(&evidence.stderr_path, &smoke.stderr)?;
    let copied_output_digest = file_digest_blake3(&evidence.output_path)?;
    if copied_output_digest != smoke.output_digest_blake3 {
        return Err(RustSourceProviderError::Smoke("copied smoke output digest mismatch".to_string()));
    }
    let copied_metadata_digest = file_digest_blake3(&evidence.metadata_path)?;
    if copied_metadata_digest != validation.metadata_digest_blake3 {
        return Err(RustSourceProviderError::Smoke("copied provider metadata digest mismatch".to_string()));
    }
    write_smoke_evidence_summary(&evidence, smoke, validation)?;
    Ok(evidence)
}

pub(crate) fn validate_materialized_rust_source_provider(
    provider_dir: &Path,
) -> Result<RustSourceProviderDirectoryValidation, RustSourceProviderError> {
    let metadata_path = provider_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    let metadata_bytes = fs::read(&metadata_path).map_err(|err| {
        RustSourceProviderError::Read(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    if metadata_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("provider metadata {} is empty", metadata_path.display())));
    }
    let metadata_digest_blake3 = blake3::hash(&metadata_bytes).to_hex().to_string();
    let metadata = serde_json::from_slice::<RustSourceProviderMetadata>(&metadata_bytes).map_err(|err| {
        RustSourceProviderError::Parse(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    let observed = observed_provider_artifacts(provider_dir, &metadata)?;
    let observed_receipts = observed_provider_receipts(provider_dir, &metadata)?;
    crate::source_toolchain_closure::enforce_observed_rust_source_provider_artifacts(&metadata, &observed)
        .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    let validation =
        crate::source_toolchain_closure::enforce_observed_rust_source_provider_receipts(&metadata, &observed_receipts)
            .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    Ok(RustSourceProviderDirectoryValidation {
        metadata_path,
        metadata_digest_blake3,
        metadata,
        validation,
    })
}

fn smoke_evidence_paths(
    evidence_dir: &Path,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> RustSourceProviderSmokeEvidence {
    RustSourceProviderSmokeEvidence {
        evidence_dir: evidence_dir.to_path_buf(),
        summary_path: evidence_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE),
        stdout_path: evidence_dir.join(SMOKE_EVIDENCE_STDOUT_FILE),
        stderr_path: evidence_dir.join(SMOKE_EVIDENCE_STDERR_FILE),
        source_path: evidence_dir.join(SMOKE_EVIDENCE_SOURCE_FILE),
        output_path: evidence_dir.join(SMOKE_EVIDENCE_OUTPUT_FILE),
        metadata_path: evidence_dir.join(SMOKE_EVIDENCE_METADATA_FILE),
        output_digest_blake3: smoke.output_digest_blake3.clone(),
        metadata_digest_blake3: validation.metadata_digest_blake3.clone(),
        policy_digest_blake3: validation.validation.policy_digest_blake3.clone(),
    }
}

fn write_smoke_evidence_summary(
    evidence: &RustSourceProviderSmokeEvidence,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> Result<(), RustSourceProviderError> {
    let summary = serde_json::json!({
        "schema": SMOKE_EVIDENCE_SCHEMA,
        "provider_path": smoke.provider_path,
        "rustc_path": smoke.rustc_path,
        "target_triple": smoke.target_triple,
        "stdout_path": evidence.stdout_path,
        "stderr_path": evidence.stderr_path,
        "source_path": evidence.source_path,
        "output_path": evidence.output_path,
        "output_digest_blake3": evidence.output_digest_blake3,
        "metadata": {
            "original_path": validation.metadata_path,
            "evidence_path": evidence.metadata_path,
            "metadata_digest_blake3": evidence.metadata_digest_blake3,
            "policy_digest_blake3": evidence.policy_digest_blake3,
            "host_triple": validation.metadata.host_triple,
            "target_triple": validation.metadata.target_triple,
            "artifact_count": validation.validation.artifact_count,
            "source_count": validation.validation.source_count,
            "receipt_count": validation.validation.receipt_count,
        }
    });
    let bytes = serde_json::to_vec_pretty(&summary)
        .map_err(|err| RustSourceProviderError::Smoke(format!("serialize smoke evidence summary: {err}")))?;
    fs::write(&evidence.summary_path, bytes).map_err(|err| {
        RustSourceProviderError::Smoke(format!(
            "write smoke evidence summary {}: {err}",
            evidence.summary_path.display()
        ))
    })
}

fn copy_evidence_file(source_path: &Path, dest_path: &Path) -> Result<(), RustSourceProviderError> {
    fs::copy(source_path, dest_path).map_err(|err| {
        RustSourceProviderError::Smoke(format!(
            "copy evidence {} -> {}: {err}",
            source_path.display(),
            dest_path.display()
        ))
    })?;
    Ok(())
}

fn write_evidence_text(path: &Path, text: &str) -> Result<(), RustSourceProviderError> {
    fs::write(path, text).map_err(|err| RustSourceProviderError::Smoke(format!("write {}: {err}", path.display())))
}

fn write_json_pretty<T: serde::Serialize>(path: &Path, value: &T, label: &str) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize {label}: {err}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", parent.display())))?;
    fs::write(path, bytes)
        .map_err(|err| RustSourceProviderError::Read(format!("write {label} {}: {err}", path.display())))
}

fn validate_import_request(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if import_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("import dir is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if !import_dir.is_dir() {
        return Err(RustSourceProviderError::Read(format!("import dir {} is not a directory", import_dir.display())));
    }
    if output_dir.exists() {
        return Err(RustSourceProviderError::Copy(format!("output dir {} already exists", output_dir.display())));
    }
    Ok(())
}

fn plan_materialization(
    recipe_path: &Path,
    route_plan_path: Option<&Path>,
    output_dir: &Path,
    scratch_dir: &Path,
    recipe_digest_blake3: &str,
) -> Result<RustSourceProviderMaterializationPlan, RustSourceProviderError> {
    if recipe_path.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("recipe path is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if output_dir.exists() {
        return Err(RustSourceProviderError::Copy(format!("output dir {} already exists", output_dir.display())));
    }
    if scratch_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("scratch dir is empty".to_string()));
    }
    if recipe_digest_blake3.is_empty() {
        return Err(RustSourceProviderError::Digest("recipe digest is empty".to_string()));
    }
    let route_plan_path = match route_plan_path {
        Some(path) => validate_explicit_route_plan_path(path)?,
        None => route_plan_path_for_recipe(recipe_path)?,
    };
    Ok(RustSourceProviderMaterializationPlan {
        recipe_path: recipe_path.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        recipe_digest_blake3: recipe_digest_blake3.to_string(),
        route_plan_path,
    })
}

fn validate_explicit_route_plan_path(path: &Path) -> Result<PathBuf, RustSourceProviderError> {
    if path.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("route plan path is empty".to_string()));
    }
    Ok(path.to_path_buf())
}

fn route_plan_path_for_recipe(recipe_path: &Path) -> Result<PathBuf, RustSourceProviderError> {
    let parent = recipe_path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("recipe {} has no parent", recipe_path.display())))?;
    Ok(parent.join(RUST_SOURCE_PROVIDER_PLAN_FILE))
}

fn load_rust_source_provider_route(plan_path: &Path) -> Result<LoadedRustSourceProviderRoute, RustSourceProviderError> {
    let plan_bytes = fs::read(plan_path)
        .map_err(|err| RustSourceProviderError::Read(format!("route plan {}: {err}", plan_path.display())))?;
    if plan_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("route plan {} is empty", plan_path.display())));
    }
    let plan_digest_blake3 = blake3::hash(&plan_bytes).to_hex().to_string();
    let import_paths: Vec<OsString> = Vec::new();
    let plan: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(plan_path, &import_paths)
        .map_err(|err| RustSourceProviderError::Parse(format!("route plan {}: {err}", plan_path.display())))?;
    let validation = crate::source_toolchain_closure::validate_rust_source_provider_bootstrap_plan(&plan)
        .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    Ok(LoadedRustSourceProviderRoute {
        plan_path: plan_path.to_path_buf(),
        plan_digest_blake3,
        plan,
        validation,
    })
}

fn prepare_first_stage_boundary(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
) -> Result<RustSourceProviderFirstStageBoundary, RustSourceProviderError> {
    let stage = select_first_mrustc_stage(&route.plan)?;
    validate_first_stage_outputs(stage)?;
    let sources = sources_for_stage(&route.plan, stage)?;
    let expected_outputs = expected_outputs_for_stage(&route.plan, stage)?;
    Ok(RustSourceProviderFirstStageBoundary {
        route_plan_path: route.plan_path.clone(),
        route_plan_digest_blake3: route.plan_digest_blake3.clone(),
        route_policy_digest_blake3: route.validation.policy_digest_blake3.clone(),
        stage_id: stage.id.clone(),
        stage_kind: stage.kind,
        rust_version: stage.rust_version.clone(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        mrustc_target_version: mrustc_target_version(&stage.rust_version),
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        archive_dir: materialization.scratch_dir.join(FIRST_STAGE_ARCHIVE_DIR),
        source_dir: materialization.scratch_dir.join(FIRST_STAGE_SOURCE_DIR),
        build_dir: materialization.scratch_dir.join(FIRST_STAGE_BUILD_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: materialization.scratch_dir.join(FIRST_STAGE_PLAN_FILE),
        sources_manifest_path: materialization.scratch_dir.join(FIRST_STAGE_SOURCES_MANIFEST_FILE),
        build_manifest_path: materialization.scratch_dir.join(FIRST_STAGE_BUILD_MANIFEST_FILE),
        build_log_path: materialization.scratch_dir.join(FIRST_STAGE_BUILD_LOG_FILE),
        provider_candidate_dir: materialization.scratch_dir.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR),
        provider_candidate_manifest_path: materialization
            .scratch_dir
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE),
        provider_candidate_smoke_dir: materialization.scratch_dir.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR),
        provider_candidate_smoke_work_dir: materialization
            .scratch_dir
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR)
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_WORK_DIR),
        script_path: materialization.scratch_dir.join(FIRST_STAGE_SCRIPT_FILE),
    })
}

fn select_first_mrustc_stage(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<&RustSourceProviderBootstrapStage, RustSourceProviderError> {
    let mut matches = plan.stages.iter().filter(|stage| stage.kind == RustSourceProviderBootstrapStageKind::MrustcSeed);
    let Some(stage) = matches.next() else {
        return Err(RustSourceProviderError::Validate("rust source route has no mrustc seed stage".to_string()));
    };
    if matches.next().is_none() {
        return Ok(stage);
    }
    Err(RustSourceProviderError::Validate("rust source route has multiple mrustc seed stages".to_string()))
}

fn validate_first_stage_outputs(stage: &RustSourceProviderBootstrapStage) -> Result<(), RustSourceProviderError> {
    for role in REQUIRED_FIRST_STAGE_OUTPUT_ROLES {
        if stage.outputs.contains(&role) {
            continue;
        }
        return Err(RustSourceProviderError::Validate(format!(
            "first mrustc stage '{}' lacks output role {role:?}",
            stage.id
        )));
    }
    Ok(())
}

fn mrustc_target_version(rust_version: &str) -> String {
    rust_version.strip_suffix(".0").unwrap_or(rust_version).to_string()
}

fn prepare_rustc_stage1_boundary(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    scratch_layout: RustcStage1ScratchLayout,
) -> Result<RustSourceProviderRustcStage1Boundary, RustSourceProviderError> {
    let stage = select_next_rustc_stage1_stage(&route.plan, bootstrap_candidate)?;
    validate_rustc_stage1_outputs(stage)?;
    let sources = sources_for_stage(&route.plan, stage)?;
    let expected_outputs = expected_outputs_for_stage(&route.plan, stage)?;
    let stage_root = rustc_stage1_stage_root(materialization, &stage.id, scratch_layout)?;
    Ok(RustSourceProviderRustcStage1Boundary {
        route_plan_path: route.plan_path.clone(),
        route_plan_digest_blake3: route.plan_digest_blake3.clone(),
        route_policy_digest_blake3: route.validation.policy_digest_blake3.clone(),
        stage_id: stage.id.clone(),
        stage_kind: stage.kind,
        bootstrap_stage_id: stage.bootstrap_stage_id.clone(),
        rust_version: stage.rust_version.clone(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        bootstrap_provider_candidate_dir: bootstrap_candidate.candidate_dir.clone(),
        bootstrap_provider_metadata_digest_blake3: bootstrap_candidate.metadata_digest_blake3.clone(),
        bootstrap_provider_policy_digest_blake3: bootstrap_candidate.policy_digest_blake3.clone(),
        archive_dir: stage_root.join(RUSTC_STAGE1_ARCHIVE_DIR),
        source_dir: stage_root.join(RUSTC_STAGE1_SOURCE_DIR),
        build_dir: stage_root.join(RUSTC_STAGE1_BUILD_DIR),
        stage_output_dir: stage_root.join(RUSTC_STAGE1_OUTPUT_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: stage_root.join(RUSTC_STAGE1_PLAN_FILE),
        sources_manifest_path: stage_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE),
        build_manifest_path: stage_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE),
        build_log_path: stage_root.join(RUSTC_STAGE1_BUILD_LOG_FILE),
        script_path: stage_root.join(RUSTC_STAGE1_SCRIPT_FILE),
        provider_candidate_dir: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR),
        provider_candidate_manifest_path: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE),
        provider_candidate_smoke_dir: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR),
        provider_candidate_smoke_work_dir: stage_root
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_WORK_DIR),
    })
}

fn bootstrap_candidate_from_first_stage(
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> RustSourceProviderBootstrapProviderCandidate {
    RustSourceProviderBootstrapProviderCandidate {
        stage_id: candidate.stage_id.clone(),
        candidate_dir: candidate.candidate_dir.clone(),
        metadata_digest_blake3: candidate.metadata_digest_blake3.clone(),
        policy_digest_blake3: candidate.policy_digest_blake3.clone(),
    }
}

fn bootstrap_candidate_from_rustc_stage1(
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> RustSourceProviderBootstrapProviderCandidate {
    RustSourceProviderBootstrapProviderCandidate {
        stage_id: candidate.stage_id.clone(),
        candidate_dir: candidate.candidate_dir.clone(),
        metadata_digest_blake3: candidate.metadata_digest_blake3.clone(),
        policy_digest_blake3: candidate.policy_digest_blake3.clone(),
    }
}

fn rustc_stage1_stage_root(
    materialization: &RustSourceProviderMaterializationPlan,
    stage_id: &str,
    scratch_layout: RustcStage1ScratchLayout,
) -> Result<PathBuf, RustSourceProviderError> {
    match scratch_layout {
        RustcStage1ScratchLayout::TopLevel => Ok(materialization.scratch_dir.clone()),
        RustcStage1ScratchLayout::Scoped => {
            Ok(materialization.scratch_dir.join(RUSTC_STAGE1_CHAIN_DIR).join(safe_source_component(stage_id)?))
        }
    }
}

fn select_next_rustc_stage1_stage<'a>(
    plan: &'a RustSourceProviderBootstrapPlan,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
) -> Result<&'a RustSourceProviderBootstrapStage, RustSourceProviderError> {
    let mut matches = plan.stages.iter().filter(|stage| {
        stage.kind == RustSourceProviderBootstrapStageKind::RustcStage1
            && stage.bootstrap_stage_id == bootstrap_candidate.stage_id
    });
    let Some(stage) = matches.next() else {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source route has no rustc stage1 after '{}'",
            bootstrap_candidate.stage_id
        )));
    };
    if matches.next().is_none() {
        return Ok(stage);
    }
    Err(RustSourceProviderError::Validate(format!(
        "rust source route has multiple rustc stage1 stages after '{}'",
        bootstrap_candidate.stage_id
    )))
}

fn has_next_rustc_stage1_stage(
    plan: &RustSourceProviderBootstrapPlan,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
) -> bool {
    plan.stages.iter().any(|stage| {
        stage.kind == RustSourceProviderBootstrapStageKind::RustcStage1
            && stage.bootstrap_stage_id == bootstrap_candidate.stage_id
    })
}

fn run_rustc_stage1_provider_candidate_chain(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    verbose: bool,
) -> Result<Vec<RustSourceProviderRustcStage1ProviderCandidateRun>, RustSourceProviderError> {
    let mut current_bootstrap = bootstrap_candidate.clone();
    let mut scratch_layout = RustcStage1ScratchLayout::TopLevel;
    let mut runs = Vec::new();
    for _ in 0..RUSTC_STAGE1_CHAIN_MAX_STAGES {
        if !has_next_rustc_stage1_stage(&route.plan, &current_bootstrap) {
            break;
        }
        let run =
            run_rustc_stage1_provider_candidate(materialization, route, &current_bootstrap, scratch_layout, verbose)?;
        current_bootstrap = bootstrap_candidate_from_rustc_stage1(&run.candidate);
        scratch_layout = RustcStage1ScratchLayout::Scoped;
        runs.push(run);
    }
    if has_next_rustc_stage1_stage(&route.plan, &current_bootstrap) {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source route has more than {RUSTC_STAGE1_CHAIN_MAX_STAGES} rustc stage1 stages"
        )));
    }
    if runs.is_empty() {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source route has no rustc stage1 after '{}'",
            bootstrap_candidate.stage_id
        )));
    }
    Ok(runs)
}

fn run_rustc_stage1_provider_candidate(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    scratch_layout: RustcStage1ScratchLayout,
    verbose: bool,
) -> Result<RustSourceProviderRustcStage1ProviderCandidateRun, RustSourceProviderError> {
    let boundary = prepare_rustc_stage1_boundary(materialization, route, bootstrap_candidate, scratch_layout)?;
    write_rustc_stage1_boundary(&boundary)?;
    let sources = acquire_rustc_stage1_sources(&boundary, verbose)?;
    write_rustc_stage1_sources_manifest(&boundary, &sources)?;
    let build = run_rustc_stage1_build(&boundary, verbose)?;
    write_rustc_stage1_build_manifest(&boundary, &build)?;
    let candidate = assemble_rustc_stage1_provider_candidate(&boundary, route, &sources, &build)?;
    let smoke = smoke_rustc_stage1_provider_candidate(&boundary, &candidate)?;
    write_rustc_stage1_provider_candidate_manifest(&boundary, &candidate)?;
    Ok(RustSourceProviderRustcStage1ProviderCandidateRun {
        boundary,
        candidate,
        smoke,
    })
}

fn prepare_rustc_final_boundary(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
) -> Result<RustSourceProviderRustcFinalBoundary, RustSourceProviderError> {
    let stage = select_rustc_final_stage(&route.plan, bootstrap_candidate)?;
    validate_rustc_final_outputs(stage)?;
    let sources = sources_for_stage(&route.plan, stage)?;
    let expected_outputs = expected_outputs_for_stage(&route.plan, stage)?;
    let stage_root = materialization.scratch_dir.join(RUSTC_FINAL_DIR);
    Ok(RustSourceProviderRustcFinalBoundary {
        route_plan_path: route.plan_path.clone(),
        route_plan_digest_blake3: route.plan_digest_blake3.clone(),
        route_policy_digest_blake3: route.validation.policy_digest_blake3.clone(),
        stage_id: stage.id.clone(),
        stage_kind: stage.kind,
        bootstrap_stage_id: stage.bootstrap_stage_id.clone(),
        rust_version: stage.rust_version.clone(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        bootstrap_provider_candidate_dir: bootstrap_candidate.candidate_dir.clone(),
        bootstrap_provider_metadata_digest_blake3: bootstrap_candidate.metadata_digest_blake3.clone(),
        bootstrap_provider_policy_digest_blake3: bootstrap_candidate.policy_digest_blake3.clone(),
        archive_dir: stage_root.join(RUSTC_FINAL_ARCHIVE_DIR),
        source_dir: stage_root.join(RUSTC_FINAL_SOURCE_DIR),
        build_dir: stage_root.join(RUSTC_FINAL_BUILD_DIR),
        stage_output_dir: stage_root.join(RUSTC_FINAL_OUTPUT_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: stage_root.join(RUSTC_FINAL_PLAN_FILE),
        sources_manifest_path: stage_root.join(RUSTC_FINAL_SOURCES_MANIFEST_FILE),
        build_manifest_path: stage_root.join(RUSTC_FINAL_BUILD_MANIFEST_FILE),
        build_log_path: stage_root.join(RUSTC_FINAL_BUILD_LOG_FILE),
        script_path: stage_root.join(RUSTC_FINAL_SCRIPT_FILE),
        provider_candidate_dir: stage_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR),
        provider_candidate_manifest_path: stage_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_MANIFEST_FILE),
        provider_candidate_smoke_dir: stage_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_DIR),
        provider_candidate_smoke_work_dir: stage_root
            .join(RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_DIR)
            .join(RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_WORK_DIR),
    })
}

fn select_rustc_final_stage<'a>(
    plan: &'a RustSourceProviderBootstrapPlan,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
) -> Result<&'a RustSourceProviderBootstrapStage, RustSourceProviderError> {
    let mut matches = plan.stages.iter().filter(|stage| {
        stage.kind == RustSourceProviderBootstrapStageKind::RustcFinal
            && stage.bootstrap_stage_id == bootstrap_candidate.stage_id
    });
    let Some(stage) = matches.next() else {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source route has no rustc final stage after '{}'",
            bootstrap_candidate.stage_id
        )));
    };
    if matches.next().is_none() {
        return Ok(stage);
    }
    Err(RustSourceProviderError::Validate(format!(
        "rust source route has multiple rustc final stages after '{}'",
        bootstrap_candidate.stage_id
    )))
}

fn run_rustc_final_provider_candidate(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    verbose: bool,
) -> Result<RustSourceProviderRustcFinalProviderCandidateRun, RustSourceProviderError> {
    let boundary = prepare_rustc_final_boundary(materialization, route, bootstrap_candidate)?;
    write_rustc_final_boundary(&boundary)?;
    let sources = acquire_rustc_final_sources(&boundary, verbose)?;
    write_rustc_final_sources_manifest(&boundary, &sources)?;
    let build = run_rustc_final_build(&boundary, verbose)?;
    write_rustc_final_build_manifest(&boundary, &build)?;
    let candidate = assemble_rustc_final_provider_candidate(&boundary, route, &sources, &build)?;
    let smoke = smoke_rustc_final_provider_candidate(&boundary, &candidate)?;
    write_rustc_final_provider_candidate_manifest(&boundary, &candidate, false)?;
    Ok(RustSourceProviderRustcFinalProviderCandidateRun {
        boundary,
        candidate,
        smoke,
    })
}

fn validate_rustc_stage1_outputs(stage: &RustSourceProviderBootstrapStage) -> Result<(), RustSourceProviderError> {
    for role in REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES {
        if stage.outputs.contains(&role) {
            continue;
        }
        return Err(RustSourceProviderError::Validate(format!(
            "rustc stage1 '{}' lacks output role {role:?}",
            stage.id
        )));
    }
    Ok(())
}

fn validate_rustc_final_outputs(stage: &RustSourceProviderBootstrapStage) -> Result<(), RustSourceProviderError> {
    for role in REQUIRED_RUSTC_FINAL_STAGE_OUTPUT_ROLES {
        if stage.outputs.contains(&role) {
            continue;
        }
        return Err(RustSourceProviderError::Validate(format!(
            "rustc final '{}' lacks output role {role:?}",
            stage.id
        )));
    }
    Ok(())
}

fn sources_for_stage(
    plan: &RustSourceProviderBootstrapPlan,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<Vec<RustSourceProviderBootstrapSource>, RustSourceProviderError> {
    let mut sources = Vec::with_capacity(stage.source_ids.len());
    for source_id in &stage.source_ids {
        let Some(source) = plan.sources.iter().find(|source| &source.id == source_id) else {
            return Err(RustSourceProviderError::Validate(format!(
                "stage '{}' references unknown source_id '{source_id}'",
                stage.id
            )));
        };
        sources.push(source.clone());
    }
    Ok(sources)
}

fn expected_outputs_for_stage(
    plan: &RustSourceProviderBootstrapPlan,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<Vec<RustSourceProviderBootstrapOutput>, RustSourceProviderError> {
    let mut outputs = Vec::with_capacity(stage.outputs.len());
    for role in &stage.outputs {
        let Some(output) = plan.final_outputs.iter().find(|output| output.role == *role) else {
            return Err(RustSourceProviderError::Validate(format!(
                "first mrustc stage '{}' has no output path for role {role:?}",
                stage.id
            )));
        };
        outputs.push(output.clone());
    }
    outputs.sort_by_key(|output| (output.role, output.path.clone()));
    Ok(outputs)
}

fn write_first_stage_boundary(boundary: &RustSourceProviderFirstStageBoundary) -> Result<(), RustSourceProviderError> {
    fs::create_dir_all(&boundary.archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.archive_dir.display())))?;
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let plan_bytes = first_stage_boundary_manifest(boundary)?;
    fs::write(&boundary.plan_path, plan_bytes).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage plan {}: {err}", boundary.plan_path.display()))
    })?;
    let script = first_stage_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn write_rustc_stage1_boundary(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<(), RustSourceProviderError> {
    fs::create_dir_all(&boundary.archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.archive_dir.display())))?;
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let manifest = rustc_stage1_boundary_manifest(boundary);
    write_json_pretty(&boundary.plan_path, &manifest, "rustc stage1 plan")?;
    let script = rustc_stage1_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write rustc stage1 script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn rustc_stage1_boundary_manifest(boundary: &RustSourceProviderRustcStage1Boundary) -> serde_json::Value {
    serde_json::json!({
        "schema": RUSTC_STAGE1_PLAN_SCHEMA,
        "route_plan": {
            "path": boundary.route_plan_path.display().to_string(),
            "digest_blake3": &boundary.route_plan_digest_blake3,
            "policy_digest_blake3": &boundary.route_policy_digest_blake3,
        },
        "stage": {
            "id": &boundary.stage_id,
            "kind": boundary.stage_kind,
            "bootstrap_stage_id": &boundary.bootstrap_stage_id,
            "rust_version": &boundary.rust_version,
            "host_triple": &boundary.host_triple,
            "target_triple": &boundary.target_triple,
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": &boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": &boundary.bootstrap_provider_policy_digest_blake3,
        },
        "sources": &boundary.sources,
        "paths": {
            "archive_dir": boundary.archive_dir.display().to_string(),
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "stage_output_dir": boundary.stage_output_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
            "source_manifest": boundary.sources_manifest_path.display().to_string(),
            "build_manifest": boundary.build_manifest_path.display().to_string(),
            "build_log": boundary.build_log_path.display().to_string(),
        },
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    })
}

fn rustc_stage1_boundary_script(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<String, RustSourceProviderError> {
    let inputs = rustc_stage1_script_inputs(boundary)?;
    let mut script = String::new();
    push_rustc_stage1_script_header(&mut script, boundary, &inputs);
    push_rustc_stage1_script_input_checks(&mut script);
    push_rustc_stage1_script_build_commands(&mut script);
    Ok(script)
}

fn write_rustc_final_boundary(boundary: &RustSourceProviderRustcFinalBoundary) -> Result<(), RustSourceProviderError> {
    fs::create_dir_all(&boundary.archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.archive_dir.display())))?;
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let manifest = rustc_final_boundary_manifest(boundary);
    write_json_pretty(&boundary.plan_path, &manifest, "rustc final plan")?;
    let script = rustc_final_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write rustc final script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn rustc_final_boundary_manifest(boundary: &RustSourceProviderRustcFinalBoundary) -> serde_json::Value {
    serde_json::json!({
        "schema": RUSTC_FINAL_PLAN_SCHEMA,
        "route_plan": {
            "path": boundary.route_plan_path.display().to_string(),
            "digest_blake3": &boundary.route_plan_digest_blake3,
            "policy_digest_blake3": &boundary.route_policy_digest_blake3,
        },
        "stage": {
            "id": &boundary.stage_id,
            "kind": boundary.stage_kind,
            "bootstrap_stage_id": &boundary.bootstrap_stage_id,
            "rust_version": &boundary.rust_version,
            "host_triple": &boundary.host_triple,
            "target_triple": &boundary.target_triple,
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": &boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": &boundary.bootstrap_provider_policy_digest_blake3,
        },
        "sources": &boundary.sources,
        "paths": {
            "archive_dir": boundary.archive_dir.display().to_string(),
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "stage_output_dir": boundary.stage_output_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
            "source_manifest": boundary.sources_manifest_path.display().to_string(),
            "build_manifest": boundary.build_manifest_path.display().to_string(),
            "build_log": boundary.build_log_path.display().to_string(),
        },
        "candidate_only": true,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    })
}

fn rustc_final_boundary_script(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<String, RustSourceProviderError> {
    let inputs = rustc_final_script_inputs(boundary)?;
    let mut script = String::new();
    push_rustc_final_script_header(&mut script, boundary, &inputs);
    push_rustc_final_script_input_checks(&mut script);
    push_rustc_final_script_build_commands(&mut script);
    Ok(script)
}

struct RustSourceProviderRustcStage1ScriptInputs {
    source_id: String,
    source_path: PathBuf,
    build_script_path: PathBuf,
}

fn rustc_stage1_script_inputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<RustSourceProviderRustcStage1ScriptInputs, RustSourceProviderError> {
    let source = rustc_stage1_build_source(boundary)?;
    let source_component = safe_source_component(&source.id)?;
    let source_path = boundary.source_dir.join(&source_component);
    let build_script_path = source_path.join(RUSTC_STAGE1_SOURCE_BUILD_SCRIPT);
    Ok(RustSourceProviderRustcStage1ScriptInputs {
        source_id: source.id.clone(),
        source_path,
        build_script_path,
    })
}

struct RustSourceProviderRustcFinalScriptInputs {
    source_id: String,
    source_path: PathBuf,
    build_script_path: PathBuf,
}

fn rustc_final_script_inputs(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<RustSourceProviderRustcFinalScriptInputs, RustSourceProviderError> {
    let source = rustc_final_build_source(boundary)?;
    let source_component = safe_source_component(&source.id)?;
    let source_path = boundary.source_dir.join(&source_component);
    let build_script_path = source_path.join(RUSTC_FINAL_SOURCE_BUILD_SCRIPT);
    Ok(RustSourceProviderRustcFinalScriptInputs {
        source_id: source.id.clone(),
        source_path,
        build_script_path,
    })
}

fn push_rustc_stage1_script_header(
    script: &mut String,
    boundary: &RustSourceProviderRustcStage1Boundary,
    inputs: &RustSourceProviderRustcStage1ScriptInputs,
) {
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("RUSTC_VERSION={}\n", shell_quote(&boundary.rust_version)));
    script.push_str(&format!("HOST_TRIPLE={}\n", shell_quote(&boundary.host_triple)));
    script.push_str(&format!("TARGET_TRIPLE={}\n", shell_quote(&boundary.target_triple)));
    script
        .push_str(&format!("SOURCE_MANIFEST={}\n", shell_quote(&boundary.sources_manifest_path.display().to_string())));
    script.push_str(&format!("RUST_SOURCE_ID={}\n", shell_quote(&inputs.source_id)));
    script.push_str(&format!("RUST_SOURCE={}\n", shell_quote(&inputs.source_path.display().to_string())));
    script.push_str(&format!("BUILD_SCRIPT={}\n", shell_quote(&inputs.build_script_path.display().to_string())));
    script.push_str(&format!(
        "BOOTSTRAP_PROVIDER={}\n",
        shell_quote(&boundary.bootstrap_provider_candidate_dir.display().to_string())
    ));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("STAGE_OUTPUT={}\n", shell_quote(&boundary.stage_output_dir.display().to_string())));
    script.push_str(&format!("SHELL_PROGRAM={}\n", shell_quote(RUSTC_STAGE1_SHELL_PROGRAM)));
}

fn push_rustc_stage1_script_input_checks(script: &mut String) {
    script.push_str("printf '%s\\n' \"mantle rustc stage1: $STAGE_ID\"\n");
    push_required_shell_var_file_check(
        script,
        "SOURCE_MANIFEST",
        "missing rustc stage1 source manifest",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_dir_check(
        script,
        "RUST_SOURCE",
        "missing verified rustc stage1 source",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_rustc_source_build_script_resolution(
        script,
        "rustc stage1 source has no build script or x.py",
        RUSTC_STAGE1_GENERATED_BUILD_SCRIPT,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/rustc\"",
        "bootstrap provider candidate has no rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/cargo\"",
        "bootstrap provider candidate has no cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "bootstrap provider candidate has no target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_command_check(script, "SHELL_PROGRAM", "sh is required for rustc stage1");
}

fn push_rustc_source_build_script_resolution(script: &mut String, missing_message: &str, generated_script_file: &str) {
    script.push_str("if [ -f \"$BUILD_SCRIPT\" ]; then\n");
    script.push_str("  RESOLVED_BUILD_SCRIPT=\"$BUILD_SCRIPT\"\n");
    script.push_str("else\n");
    script.push_str("  XPY_SCRIPT=\"$RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_XPY_SCRIPT);
    script.push_str("\"\n");
    script.push_str("  if [ ! -f \"$XPY_SCRIPT\" ]; then printf '%s\\n' ");
    script.push_str(&shell_quote(missing_message));
    script.push_str(" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("  mkdir -p \"$BUILD_DIR\"\n");
    script.push_str("  RESOLVED_BUILD_SCRIPT=\"$BUILD_DIR/");
    script.push_str(generated_script_file);
    script.push_str("\"\n");
    push_generated_rustc_source_build_script(script);
    script.push_str("  chmod +x \"$RESOLVED_BUILD_SCRIPT\"\n");
    script.push_str("fi\n");
}

fn push_generated_rustc_source_build_script(script: &mut String) {
    script.push_str("  cat > \"$RESOLVED_BUILD_SCRIPT\" <<'MANTLE_RUST_SOURCE_GENERATED_SCRIPT'\n");
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str("CONFIG=\"$MANTLE_BUILD_DIR/");
    script.push_str(RUSTC_SOURCE_GENERATED_CONFIG_FILE);
    script.push_str("\"\n");
    script.push_str("mkdir -p \"$MANTLE_BUILD_DIR\" \"$MANTLE_STAGE_OUTPUT\"\n");
    push_rustc_source_build_env_scrub(script);
    push_rustc_source_build_tool_discovery(script);
    script.push_str("cat > \"$CONFIG\" <<MANTLE_RUST_BUILD_CONFIG\n");
    script.push_str("profile = \"compiler\"\n");
    script.push_str("change-id = \"ignore\"\n\n");
    script.push_str("[build]\n");
    script.push_str("build = \"$MANTLE_HOST_TRIPLE\"\n");
    script.push_str("host = [\"$MANTLE_HOST_TRIPLE\"]\n");
    script.push_str("target = [\"$MANTLE_HOST_TRIPLE\", \"$MANTLE_TARGET_TRIPLE\"]\n");
    script.push_str("cargo = \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\n");
    script.push_str("rustc = \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\n");
    script.push_str("extended = true\n");
    script.push_str("tools = [\"cargo\", \"rustdoc\"]\n");
    script.push_str("vendor = true\n");
    script.push_str("build-dir = \"$MANTLE_BUILD_DIR/rust-build\"\n\n");
    script.push_str("[install]\n");
    script.push_str("prefix = \"$MANTLE_STAGE_OUTPUT\"\n");
    script.push_str("sysconfdir = \"etc\"\n\n");
    script.push_str("[rust]\n");
    script.push_str("download-rustc = false\n");
    script.push_str("channel = \"stable\"\n");
    script.push_str("codegen-tests = false\n");
    script.push_str("deny-warnings = false\n\n");
    script.push_str("[llvm]\n");
    script.push_str("download-ci-llvm = false\n");
    script.push_str("ninja = false\n");
    script.push_str("MANTLE_RUST_BUILD_CONFIG\n");
    push_rustc_source_build_target_tool_config(script);
    push_rustc_source_bootstrap_workspace_isolation(script);
    script.push_str("printf '%s\\n' \"using generated x.py Rust build adapter: $MANTLE_RUST_SOURCE\"\n");
    script.push_str("if [ -x \"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_XPY_SCRIPT);
    script.push_str("\" ]; then\n");
    script.push_str("  \"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_XPY_SCRIPT);
    script.push_str("\" --config \"$CONFIG\" $MANTLE_RUST_BUILD_GOALS\n");
    script.push_str("elif command -v python3 >/dev/null 2>&1; then\n");
    script.push_str("  python3 \"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_XPY_SCRIPT);
    script.push_str("\" --config \"$CONFIG\" $MANTLE_RUST_BUILD_GOALS\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' 'x.py is not executable and python3 is unavailable' >&2\n");
    script.push_str("  exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("\n");
    script.push_str("fi\n");
    script.push_str("MANTLE_RUST_SOURCE_GENERATED_SCRIPT\n");
}

fn push_rustc_source_build_env_scrub(script: &mut String) {
    script.push_str("printf '%s\\n' 'scrubbing inherited Rust bootstrap Cargo environment'\n");
    for var_name in FIRST_STAGE_ENV_SCRUB_VARS {
        script.push_str(&format!("unset {var_name}\n"));
    }
    script.push_str("CARGO_HOME=\"$MANTLE_BUILD_DIR/");
    script.push_str(RUSTC_SOURCE_GENERATED_CARGO_HOME_DIR);
    script.push_str("\"\n");
    script.push_str("mkdir -p \"$CARGO_HOME\"\n");
    script.push_str("export CARGO_HOME\n");
    script.push_str("RUSTC_WRAPPER=\n");
    script.push_str("RUSTC_WORKSPACE_WRAPPER=\n");
    script.push_str("export RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER\n");
}

fn push_rustc_source_build_tool_discovery(script: &mut String) {
    script.push_str("MAKE_PROGRAM=${MAKE_PROGRAM:-");
    script.push_str(FIRST_STAGE_MAKE_PROGRAM);
    script.push_str("}\n");
    script.push_str("make_path=$(command -v \"$MAKE_PROGRAM\" 2>/dev/null || true)\n");
    script.push_str("if [ -z \"$make_path\" ]; then\n");
    script.push_str("  for candidate in ");
    script.push_str(FIRST_STAGE_MAKE_FALLBACK_GLOB);
    script.push_str("; do\n");
    script.push_str("    if [ -x \"$candidate\" ]; then make_path=\"$candidate\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("fi\n");
    script.push_str("if [ -n \"$make_path\" ]; then\n");
    script.push_str("  MAKE_PROGRAM=\"$make_path\"\n");
    script.push_str("  make_dir=${make_path%/*}\n");
    script.push_str("  PATH=\"$make_dir:$PATH\"\n");
    script.push_str("  export MAKE_PROGRAM PATH\n");
    script.push_str("  printf '%s\\n' \"using Rust bootstrap make: $make_path\"\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' 'Rust bootstrap make was not found in PATH or fallback store paths' >&2\n");
    script.push_str("fi\n");
    script.push_str("CMAKE_PROGRAM=${CMAKE_PROGRAM:-");
    script.push_str(FIRST_STAGE_CMAKE_PROGRAM);
    script.push_str("}\n");
    script.push_str("cmake_path=$(command -v \"$CMAKE_PROGRAM\" 2>/dev/null || true)\n");
    script.push_str("if [ -z \"$cmake_path\" ]; then\n");
    script.push_str("  for candidate in ");
    script.push_str(FIRST_STAGE_CMAKE_FALLBACK_GLOB);
    script.push_str("; do\n");
    script.push_str("    if [ -x \"$candidate\" ]; then cmake_path=\"$candidate\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("fi\n");
    script.push_str("if [ -n \"$cmake_path\" ]; then\n");
    script.push_str("  CMAKE_PROGRAM=\"$cmake_path\"\n");
    script.push_str("  cmake_dir=${cmake_path%/*}\n");
    script.push_str("  PATH=\"$cmake_dir:$PATH\"\n");
    script.push_str("  export CMAKE_PROGRAM PATH\n");
    script.push_str("  printf '%s\\n' \"using Rust bootstrap cmake: $cmake_path\"\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' 'Rust bootstrap cmake was not found in PATH or fallback store paths' >&2\n");
    script.push_str("fi\n");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=${");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str(":-}\n");
    script.push_str("MANTLE_TARGET_MUSL_MACHINE_ALIASES=");
    script.push_str(&shell_quote(&musl_target_machine_aliases_shell_words()));
    script.push_str("\n");
    script.push_str("MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT=");
    script.push_str(&shell_quote(FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX));
    script.push_str("\n");
    script.push_str("if [ -z \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  for candidate_name in ");
    script.push_str(&musl_target_tool_program_candidates_shell_words("gcc"));
    script.push_str("; do\n");
    script.push_str("    if command -v \"$candidate_name\" >/dev/null 2>&1; then ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\"$(command -v \"$candidate_name\")\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("fi\n");
    script.push_str("if [ -z \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  for candidate in ");
    script.push_str(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB);
    script.push_str("; do\n");
    script.push_str("    if [ -x \"$candidate\" ]; then ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\"$candidate\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("fi\n");
    script.push_str("if [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  target_tool_dir=${");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("%/*}\n");
    script.push_str("  target_cc_basename=${");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("##*/}\n");
    script.push_str("  target_cc_machine=$(\"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" -dumpmachine 2>/dev/null || true)\n");
    script.push_str("  case \" $MANTLE_TARGET_MUSL_MACHINE_ALIASES \" in *\" $target_cc_machine \"*) ;; *) printf '%s\\n' \"Rust bootstrap target gcc machine expected one of: $MANTLE_TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine\" >&2; ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("= ;; esac\n");
    script.push_str("fi\n");
    script.push_str("if [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  case \"$target_cc_basename\" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac\n");
    script.push_str("  if [ -z \"$target_tool_prefix\" ]; then ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=; fi\n");
    script.push_str("fi\n");
    script.push_str("if [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("=\"$target_tool_dir/$target_tool_prefix-g++\"\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("=\"$target_tool_dir/$target_tool_prefix-ar\"\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("=\"$target_tool_dir/$target_tool_prefix-ranlib\"\n");
    script.push_str("  target_wrapper_root=${target_tool_dir%/*}\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("=\n");
    script.push_str("  if [ -f \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push_str("/");
    script.push_str(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE);
    script.push_str("\" ]; then\n");
    script.push_str("    IFS= read -r ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str(" < \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push_str("/");
    script.push_str(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE);
    script.push_str("\" || true\n");
    script.push_str("  fi\n");
    script.push_str("  if [ -z \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script
        .push_str("\" ] && [ -f \"$target_wrapper_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.a\" ]; then\n");
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("=\"$target_wrapper_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT\"\n");
    script.push_str("  fi\n");
    script.push_str("  if [ -x \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("\" ] && [ -x \"$");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("\" ] && [ -x \"$");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("\" ] && [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("\" ] && [ -f \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("/lib/libc.a\" ] && [ -f \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("/lib/crt1.o\" ] && [ -f \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("/lib/rcrt1.o\" ]; then\n");
    script.push_str("    PATH=\"$target_tool_dir:$PATH\"\n");
    script.push_str("    export ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str(" ");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str(" ");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str(" ");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str(" ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str(" PATH\n");
    script.push_str("    printf '%s\\n' \"using Rust bootstrap target tools from: $target_tool_dir\"\n");
    script.push_str("  else\n");
    script
        .push_str("    printf '%s\\n' \"Rust bootstrap target tool directory is incomplete: $target_tool_dir\" >&2\n");
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\n");
    script.push_str("  fi\n");
    script.push_str("else\n");
    script.push_str(
        "  printf '%s\\n' 'Rust bootstrap target musl compiler was not found in PATH or fallback store paths' >&2\n",
    );
    script.push_str("fi\n");
}

fn push_rustc_source_build_target_tool_config(script: &mut String) {
    script.push_str("if [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ] && [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("\" ] && [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("\" ] && [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("\" ] && [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  cat >> \"$CONFIG\" <<MANTLE_RUST_TARGET_CONFIG\n");
    script.push_str("[target.$MANTLE_TARGET_TRIPLE]\n");
    script.push_str("cc = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\"\n");
    script.push_str("cxx = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("\"\n");
    script.push_str("ar = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("\"\n");
    script.push_str("ranlib = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("\"\n");
    script.push_str("linker = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\"\n");
    script.push_str("musl-root = \"$");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("\"\n");
    script.push_str("MANTLE_RUST_TARGET_CONFIG\n");
    script.push_str("fi\n");
}

fn push_rustc_source_bootstrap_workspace_isolation(script: &mut String) {
    script.push_str("BOOTSTRAP_MANIFEST=\"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_BOOTSTRAP_MANIFEST);
    script.push_str("\"\n");
    script.push_str("if [ ! -f \"$BOOTSTRAP_MANIFEST\" ]; then\n");
    script.push_str("  printf '%s\\n' 'Rust bootstrap manifest missing for workspace isolation' >&2\n");
    script.push_str("  exit 4\n");
    script.push_str("fi\n");
    script.push_str("for workspace_boundary_manifest in \"$BOOTSTRAP_MANIFEST\" \"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_CRANELIFT_MANIFEST);
    script.push_str("\" \"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_CODEGEN_GCC_MANIFEST);
    script.push_str("\"; do\n");
    script.push_str("  if [ ! -f \"$workspace_boundary_manifest\" ]; then continue; fi\n");
    script.push_str("  workspace_boundary_seen=0\n");
    script.push_str("  while IFS= read -r workspace_boundary_line; do\n");
    script.push_str(
        "    if [ \"$workspace_boundary_line\" = '[workspace]' ]; then workspace_boundary_seen=1; break; fi\n",
    );
    script.push_str("  done < \"$workspace_boundary_manifest\"\n");
    script.push_str("  if [ \"$workspace_boundary_seen\" = 0 ]; then\n");
    script.push_str("    printf '\\n[workspace]\\n' >> \"$workspace_boundary_manifest\"\n");
    script.push_str("    printf '%s\\n' \"isolated Rust Cargo workspace: $workspace_boundary_manifest\"\n");
    script.push_str("  fi\n");
    script.push_str("done\n");
}

fn push_rustc_source_build_script_launch(script: &mut String, goals: &str) {
    script.push_str(
        "MANTLE_BOOTSTRAP_PROVIDER=\"$BOOTSTRAP_PROVIDER\" MANTLE_STAGE_OUTPUT=\"$STAGE_OUTPUT\" MANTLE_BUILD_DIR=\"$BUILD_DIR\" MANTLE_RUST_SOURCE=\"$RUST_SOURCE\" MANTLE_RUST_BUILD_GOALS=",
    );
    script.push_str(&shell_quote(goals));
    script.push_str(
        " MANTLE_RUST_VERSION=\"$RUSTC_VERSION\" MANTLE_HOST_TRIPLE=\"$HOST_TRIPLE\" MANTLE_TARGET_TRIPLE=\"$TARGET_TRIPLE\" \"$SHELL_PROGRAM\" \"$RESOLVED_BUILD_SCRIPT\"\n",
    );
}

fn push_rustc_stage1_script_build_commands(script: &mut String) {
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$STAGE_OUTPUT\"\n");
    push_rustc_source_build_script_launch(script, RUSTC_STAGE1_XPY_GOALS);
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/rustc\"",
        "rustc stage1 build did not produce bin/rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/cargo\"",
        "rustc stage1 build did not produce bin/cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$HOST_TRIPLE/lib\"",
        "rustc stage1 build did not produce host rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "rustc stage1 build did not produce target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    script.push_str("printf '%s\\n' \"rustc stage1 products ready\"\n");
}

fn push_rustc_final_script_header(
    script: &mut String,
    boundary: &RustSourceProviderRustcFinalBoundary,
    inputs: &RustSourceProviderRustcFinalScriptInputs,
) {
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("RUSTC_VERSION={}\n", shell_quote(&boundary.rust_version)));
    script.push_str(&format!("HOST_TRIPLE={}\n", shell_quote(&boundary.host_triple)));
    script.push_str(&format!("TARGET_TRIPLE={}\n", shell_quote(&boundary.target_triple)));
    script
        .push_str(&format!("SOURCE_MANIFEST={}\n", shell_quote(&boundary.sources_manifest_path.display().to_string())));
    script.push_str(&format!("RUST_SOURCE_ID={}\n", shell_quote(&inputs.source_id)));
    script.push_str(&format!("RUST_SOURCE={}\n", shell_quote(&inputs.source_path.display().to_string())));
    script.push_str(&format!("BUILD_SCRIPT={}\n", shell_quote(&inputs.build_script_path.display().to_string())));
    script.push_str(&format!(
        "BOOTSTRAP_PROVIDER={}\n",
        shell_quote(&boundary.bootstrap_provider_candidate_dir.display().to_string())
    ));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("STAGE_OUTPUT={}\n", shell_quote(&boundary.stage_output_dir.display().to_string())));
    script.push_str(&format!("SHELL_PROGRAM={}\n", shell_quote(RUSTC_FINAL_SHELL_PROGRAM)));
}

fn push_rustc_final_script_input_checks(script: &mut String) {
    script.push_str("printf '%s\\n' \"mantle rustc final: $STAGE_ID\"\n");
    push_required_shell_var_file_check(
        script,
        "SOURCE_MANIFEST",
        "missing rustc final source manifest",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_dir_check(
        script,
        "RUST_SOURCE",
        "missing verified rustc final source",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_rustc_source_build_script_resolution(
        script,
        "rustc final source has no build script or x.py",
        RUSTC_FINAL_GENERATED_BUILD_SCRIPT,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/rustc\"",
        "bootstrap provider candidate has no rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/cargo\"",
        "bootstrap provider candidate has no cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "bootstrap provider candidate has no target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_command_check(script, "SHELL_PROGRAM", "sh is required for rustc final");
}

fn push_rustc_final_script_build_commands(script: &mut String) {
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$STAGE_OUTPUT\"\n");
    push_rustc_source_build_script_launch(script, RUSTC_FINAL_XPY_GOALS);
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/rustc\"",
        "rustc final build did not produce bin/rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/cargo\"",
        "rustc final build did not produce bin/cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/rustdoc\"",
        "rustc final build did not produce bin/rustdoc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$HOST_TRIPLE/lib\"",
        "rustc final build did not produce host rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "rustc final build did not produce target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    script.push_str("printf '%s\\n' \"rustc final products ready\"\n");
}

fn rustc_stage1_build_source(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<&RustSourceProviderBootstrapSource, RustSourceProviderError> {
    let mut matches = boundary.sources.iter().filter(|source| source.id.starts_with(FIRST_STAGE_RUST_SOURCE_PREFIX));
    let Some(source) = matches.next() else {
        return Err(RustSourceProviderError::Build(format!("rustc stage1 '{}' has no Rust source", boundary.stage_id)));
    };
    if matches.next().is_none() {
        return Ok(source);
    }
    Err(RustSourceProviderError::Build(format!(
        "rustc stage1 '{}' has multiple Rust sources",
        boundary.stage_id
    )))
}

fn rustc_final_build_source(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<&RustSourceProviderBootstrapSource, RustSourceProviderError> {
    let mut matches = boundary.sources.iter().filter(|source| source.id.starts_with(FIRST_STAGE_RUST_SOURCE_PREFIX));
    let Some(source) = matches.next() else {
        return Err(RustSourceProviderError::Build(format!("rustc final '{}' has no Rust source", boundary.stage_id)));
    };
    if matches.next().is_none() {
        return Ok(source);
    }
    Err(RustSourceProviderError::Build(format!(
        "rustc final '{}' has multiple Rust sources",
        boundary.stage_id
    )))
}

fn first_stage_boundary_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_PLAN_SCHEMA,
        "route_plan": {
            "path": boundary.route_plan_path.display().to_string(),
            "digest_blake3": &boundary.route_plan_digest_blake3,
            "policy_digest_blake3": &boundary.route_policy_digest_blake3,
        },
        "stage": {
            "id": &boundary.stage_id,
            "kind": boundary.stage_kind,
            "rust_version": &boundary.rust_version,
            "host_triple": &boundary.host_triple,
            "target_triple": &boundary.target_triple,
            "mrustc_target_version": &boundary.mrustc_target_version,
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "sources": &boundary.sources,
        "paths": {
            "archive_dir": boundary.archive_dir.display().to_string(),
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
            "source_manifest": boundary.sources_manifest_path.display().to_string(),
            "build_manifest": boundary.build_manifest_path.display().to_string(),
            "build_log": boundary.build_log_path.display().to_string(),
            "provider_candidate_dir": boundary.provider_candidate_dir.display().to_string(),
            "provider_candidate_manifest": boundary.provider_candidate_manifest_path.display().to_string(),
            "provider_candidate_smoke_dir": boundary.provider_candidate_smoke_dir.display().to_string(),
            "provider_candidate_smoke_work_dir": boundary.provider_candidate_smoke_work_dir.display().to_string(),
        },
        "blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    serde_json::to_vec_pretty(&manifest)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize first stage plan: {err}")))
}

fn acquire_first_stage_sources(
    boundary: &RustSourceProviderFirstStageBoundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(&boundary.stage_id, &boundary.archive_dir, &boundary.source_dir, &boundary.sources, verbose)
}

fn acquire_rustc_stage1_sources(
    boundary: &RustSourceProviderRustcStage1Boundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(&boundary.stage_id, &boundary.archive_dir, &boundary.source_dir, &boundary.sources, verbose)
}

fn acquire_rustc_final_sources(
    boundary: &RustSourceProviderRustcFinalBoundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(&boundary.stage_id, &boundary.archive_dir, &boundary.source_dir, &boundary.sources, verbose)
}

fn acquire_stage_sources(
    stage_id: &str,
    archive_dir: &Path,
    source_dir: &Path,
    sources: &[RustSourceProviderBootstrapSource],
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    assert!(!stage_id.trim().is_empty(), "stage id must not be empty");
    assert!(!sources.is_empty(), "stage sources must not be empty");
    assert!(!archive_dir.as_os_str().is_empty(), "archive dir must not be empty");
    fs::create_dir_all(archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", archive_dir.display())))?;
    fs::create_dir_all(source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", source_dir.display())))?;
    let mut acquired = Vec::with_capacity(sources.len());
    for source in sources {
        acquired.push(acquire_stage_source(archive_dir, source_dir, source, verbose)?);
    }
    if acquired.len() != sources.len() {
        return Err(RustSourceProviderError::Fetch(format!("stage '{stage_id}' source acquisition count mismatch")));
    }
    Ok(acquired)
}

fn acquire_stage_source(
    archive_dir: &Path,
    source_dir: &Path,
    source: &RustSourceProviderBootstrapSource,
    verbose: bool,
) -> Result<RustSourceProviderFirstStageSourceAcquisition, RustSourceProviderError> {
    let source_component = safe_source_component(&source.id)?;
    ensure_supported_first_stage_archive(&source.url)?;
    let archive_path = archive_dir.join(format!("{source_component}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}"));
    let extracted_path = source_dir.join(&source_component);
    let bytes = fetch_first_stage_source_bytes(source)?;
    let archive_sha256_hex = sha256_hex(&bytes);
    if archive_sha256_hex != source.sha256_hex {
        return Err(RustSourceProviderError::Digest(format!(
            "source '{}' digest mismatch: expected {}, got {archive_sha256_hex}",
            source.id, source.sha256_hex
        )));
    }
    fs::write(&archive_path, &bytes)
        .map_err(|err| RustSourceProviderError::Fetch(format!("write {}: {err}", archive_path.display())))?;
    prepare_empty_directory(&extracted_path)?;
    let archive_entry_count = extract_first_stage_tar_gz(&archive_path, &extracted_path)?;
    let extracted_digest_blake3 = content_digest_blake3(&extracted_path)?;
    if verbose {
        eprintln!(
            "  acquired source {} sha256={} extracted={} entries={}",
            source.id,
            archive_sha256_hex,
            extracted_path.display(),
            archive_entry_count
        );
    }
    Ok(RustSourceProviderFirstStageSourceAcquisition {
        id: source.id.clone(),
        url: source.url.clone(),
        archive_path,
        archive_sha256_hex,
        extracted_path,
        extracted_digest_blake3,
        archive_entry_count,
    })
}

fn write_first_stage_sources_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Fetch("first stage sources manifest count mismatch".to_string()));
    }
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_SOURCES_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "source_count": sources.len(),
        "sources": sources,
        "blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.sources_manifest_path, &manifest, "first stage sources manifest")
}

fn write_rustc_stage1_sources_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Fetch("rustc stage1 sources manifest count mismatch".to_string()));
    }
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "source_count": sources.len(),
        "sources": sources,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.sources_manifest_path, &manifest, "rustc stage1 sources manifest")
}

fn write_rustc_final_sources_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Fetch("rustc final sources manifest count mismatch".to_string()));
    }
    let manifest = serde_json::json!({
        "schema": RUSTC_FINAL_SOURCES_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "source_count": sources.len(),
        "sources": sources,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "candidate_only": true,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.sources_manifest_path, &manifest, "rustc final sources manifest")
}

fn run_rustc_stage1_build(
    boundary: &RustSourceProviderRustcStage1Boundary,
    verbose: bool,
) -> Result<RustSourceProviderRustcStage1Build, RustSourceProviderError> {
    validate_rustc_stage1_build_inputs(boundary)?;
    prepare_empty_provider_candidate_dir(&boundary.stage_output_dir)?;
    if verbose {
        eprintln!("  rustc_stage1_build_script: {}", boundary.script_path.display());
        eprintln!("  rustc_stage1_build_log: {}", boundary.build_log_path.display());
    }
    let log = File::create(&boundary.build_log_path).map_err(|err| {
        RustSourceProviderError::Build(format!("create {}: {err}", boundary.build_log_path.display()))
    })?;
    let status = Command::new(&boundary.script_path)
        .stdout(log.try_clone().map_err(|err| {
            RustSourceProviderError::Build(format!("clone {}: {err}", boundary.build_log_path.display()))
        })?)
        .stderr(log)
        .status()
        .map_err(|err| RustSourceProviderError::Build(format!("launch {}: {err}", boundary.script_path.display())))?;
    if !status.success() {
        let log_tail = fs::read(&boundary.build_log_path)
            .map(|bytes| bounded_output_text(&bytes))
            .unwrap_or_else(|err| format!("<failed to read build log: {err}>"));
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 build failed with status {status}; log={}; tail={log_tail:?}",
            boundary.build_log_path.display()
        )));
    }
    rustc_stage1_build_from_outputs(boundary)
}

fn validate_rustc_stage1_build_inputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<(), RustSourceProviderError> {
    if !boundary.script_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 script {} is missing",
            boundary.script_path.display()
        )));
    }
    if !boundary.sources_manifest_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 source manifest {} is missing",
            boundary.sources_manifest_path.display()
        )));
    }
    if !boundary.bootstrap_provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "bootstrap provider candidate metadata is missing under {}",
            boundary.bootstrap_provider_candidate_dir.display()
        )));
    }
    Ok(())
}

fn run_rustc_final_build(
    boundary: &RustSourceProviderRustcFinalBoundary,
    verbose: bool,
) -> Result<RustSourceProviderRustcFinalBuild, RustSourceProviderError> {
    validate_rustc_final_build_inputs(boundary)?;
    prepare_empty_provider_candidate_dir(&boundary.stage_output_dir)?;
    if verbose {
        eprintln!("  rustc_final_build_script: {}", boundary.script_path.display());
        eprintln!("  rustc_final_build_log: {}", boundary.build_log_path.display());
    }
    let log = File::create(&boundary.build_log_path).map_err(|err| {
        RustSourceProviderError::Build(format!("create {}: {err}", boundary.build_log_path.display()))
    })?;
    let status = Command::new(&boundary.script_path)
        .stdout(log.try_clone().map_err(|err| {
            RustSourceProviderError::Build(format!("clone {}: {err}", boundary.build_log_path.display()))
        })?)
        .stderr(log)
        .status()
        .map_err(|err| RustSourceProviderError::Build(format!("launch {}: {err}", boundary.script_path.display())))?;
    if !status.success() {
        let log_tail = fs::read(&boundary.build_log_path)
            .map(|bytes| bounded_output_text(&bytes))
            .unwrap_or_else(|err| format!("<failed to read build log: {err}>"));
        return Err(RustSourceProviderError::Build(format!(
            "rustc final build failed with status {status}; log={}; tail={log_tail:?}",
            boundary.build_log_path.display()
        )));
    }
    rustc_final_build_from_outputs(boundary)
}

fn validate_rustc_final_build_inputs(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<(), RustSourceProviderError> {
    if !boundary.script_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc final script {} is missing",
            boundary.script_path.display()
        )));
    }
    if !boundary.sources_manifest_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc final source manifest {} is missing",
            boundary.sources_manifest_path.display()
        )));
    }
    if !boundary.bootstrap_provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "bootstrap provider candidate metadata is missing under {}",
            boundary.bootstrap_provider_candidate_dir.display()
        )));
    }
    Ok(())
}

fn rustc_stage1_build_from_outputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<RustSourceProviderRustcStage1Build, RustSourceProviderError> {
    let rustc_path = boundary.stage_output_dir.join(PROVIDER_RUSTC_RELATIVE_PATH);
    let cargo_path = boundary.stage_output_dir.join(PROVIDER_CARGO_RELATIVE_PATH);
    let host_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.host_triple));
    let target_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.target_triple));
    validate_first_stage_build_product(&rustc_path, PROVIDER_RUSTC_RELATIVE_PATH)?;
    validate_first_stage_build_product(&cargo_path, PROVIDER_CARGO_RELATIVE_PATH)?;
    validate_first_stage_build_product(&host_rustlib_path, "rustc stage1 host rustlib")?;
    validate_first_stage_build_product(&target_rustlib_path, "rustc stage1 target rustlib")?;
    Ok(RustSourceProviderRustcStage1Build {
        stage_id: boundary.stage_id.clone(),
        script_path: boundary.script_path.clone(),
        log_path: boundary.build_log_path.clone(),
        stage_output_digest_blake3: content_digest_blake3(&boundary.stage_output_dir)?,
        stage_output_dir: boundary.stage_output_dir.clone(),
        rustc_digest_blake3: file_digest_blake3(&rustc_path)?,
        cargo_digest_blake3: file_digest_blake3(&cargo_path)?,
        host_rustlib_digest_blake3: content_digest_blake3(&host_rustlib_path)?,
        target_rustlib_digest_blake3: content_digest_blake3(&target_rustlib_path)?,
        rustc_path,
        cargo_path,
        host_rustlib_path,
        target_rustlib_path,
    })
}

fn rustc_final_build_from_outputs(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<RustSourceProviderRustcFinalBuild, RustSourceProviderError> {
    let rustc_path = boundary.stage_output_dir.join(PROVIDER_RUSTC_RELATIVE_PATH);
    let cargo_path = boundary.stage_output_dir.join(PROVIDER_CARGO_RELATIVE_PATH);
    let rustdoc_path = boundary.stage_output_dir.join(PROVIDER_RUSTDOC_RELATIVE_PATH);
    let host_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.host_triple));
    let target_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.target_triple));
    validate_first_stage_build_product(&rustc_path, PROVIDER_RUSTC_RELATIVE_PATH)?;
    validate_first_stage_build_product(&cargo_path, PROVIDER_CARGO_RELATIVE_PATH)?;
    validate_first_stage_build_product(&rustdoc_path, PROVIDER_RUSTDOC_RELATIVE_PATH)?;
    validate_first_stage_build_product(&host_rustlib_path, "rustc final host rustlib")?;
    validate_first_stage_build_product(&target_rustlib_path, "rustc final target rustlib")?;
    Ok(RustSourceProviderRustcFinalBuild {
        stage_id: boundary.stage_id.clone(),
        script_path: boundary.script_path.clone(),
        log_path: boundary.build_log_path.clone(),
        stage_output_dir: boundary.stage_output_dir.clone(),
        stage_output_digest_blake3: content_digest_blake3(&boundary.stage_output_dir)?,
        rustc_digest_blake3: file_digest_blake3(&rustc_path)?,
        cargo_digest_blake3: file_digest_blake3(&cargo_path)?,
        rustdoc_digest_blake3: file_digest_blake3(&rustdoc_path)?,
        host_rustlib_digest_blake3: content_digest_blake3(&host_rustlib_path)?,
        target_rustlib_digest_blake3: content_digest_blake3(&target_rustlib_path)?,
        rustc_path,
        cargo_path,
        rustdoc_path,
        host_rustlib_path,
        target_rustlib_path,
    })
}

fn write_rustc_stage1_build_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    build: &RustSourceProviderRustcStage1Build,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "build": build,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.build_manifest_path, &manifest, "rustc stage1 build manifest")
}

fn write_rustc_final_build_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    build: &RustSourceProviderRustcFinalBuild,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": RUSTC_FINAL_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "build": build,
        "candidate_only": true,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.build_manifest_path, &manifest, "rustc final build manifest")
}

fn assemble_rustc_stage1_provider_candidate(
    boundary: &RustSourceProviderRustcStage1Boundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcStage1Build,
) -> Result<RustSourceProviderRustcStage1ProviderCandidate, RustSourceProviderError> {
    validate_rustc_stage1_candidate_request(boundary, sources, build)?;
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.stage_output_dir, &boundary.provider_candidate_dir)?;
    let source_identities = rustc_stage1_provider_source_identities(boundary, sources)?;
    let artifact_source_id = rustc_stage1_artifact_source_id(boundary)?;
    let receipt_artifacts = rustc_stage1_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = rustc_stage1_provider_receipt(boundary, route, &source_identities, &receipt_artifacts);
    let receipt_path = boundary.provider_candidate_dir.join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "rustc stage1 provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = rustc_stage1_provider_metadata(
        route,
        &source_identities,
        &receipt_artifacts,
        &receipt_digest_blake3,
        &artifact_source_id,
    );
    let metadata_path = boundary.provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    write_json_pretty(&metadata_path, &metadata, "rustc stage1 provider metadata")?;
    let validation = validate_materialized_rust_source_provider(&boundary.provider_candidate_dir)?;
    Ok(RustSourceProviderRustcStage1ProviderCandidate {
        stage_id: boundary.stage_id.clone(),
        candidate_dir: boundary.provider_candidate_dir.clone(),
        metadata_path,
        metadata_digest_blake3: validation.metadata_digest_blake3,
        policy_digest_blake3: validation.validation.policy_digest_blake3,
        artifact_count: validation.validation.artifact_count,
        source_count: validation.validation.source_count,
        receipt_count: validation.validation.receipt_count,
    })
}

fn smoke_rustc_stage1_provider_candidate(
    boundary: &RustSourceProviderRustcStage1Boundary,
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(&candidate.candidate_dir)?;
    let smoke = smoke_rust_source_provider(&candidate.candidate_dir, &boundary.provider_candidate_smoke_work_dir)?;
    persist_rust_source_provider_smoke_evidence(&boundary.provider_candidate_smoke_dir, &smoke, &validation)
}

fn validate_rustc_stage1_candidate_request(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcStage1Build,
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Validate("rustc stage1 candidate source count mismatch".to_string()));
    }
    if build.stage_id != boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc stage1 candidate build stage expected {}, got {}",
            boundary.stage_id, build.stage_id
        )));
    }
    if boundary.provider_candidate_dir == boundary.output_dir {
        return Err(RustSourceProviderError::Validate(
            "rustc stage1 provider candidate must not be the final output dir".to_string(),
        ));
    }
    validate_first_stage_build_product(&build.stage_output_dir, RUSTC_STAGE1_OUTPUT_DIR)
}

fn rustc_stage1_provider_source_identities(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<Vec<RustProviderSourceIdentity>, RustSourceProviderError> {
    let mut identities = Vec::with_capacity(boundary.sources.len() + 1);
    for source in &boundary.sources {
        let acquired = sources.iter().find(|acquired| acquired.id == source.id).ok_or_else(|| {
            RustSourceProviderError::Validate(format!(
                "rustc stage1 candidate lacks source acquisition '{}'",
                source.id
            ))
        })?;
        identities.push(RustProviderSourceIdentity {
            id: source.id.clone(),
            kind: source.kind,
            name: source.name.clone(),
            digest_blake3: acquired.extracted_digest_blake3.clone(),
        });
    }
    identities.push(RustProviderSourceIdentity {
        id: format!("{}-bootstrap-provider", boundary.stage_id),
        kind: ToolchainSourceKind::Generated,
        name: "bootstrap-provider-candidate".to_string(),
        digest_blake3: boundary.bootstrap_provider_metadata_digest_blake3.clone(),
    });
    Ok(identities)
}

fn rustc_stage1_artifact_source_id(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<String, RustSourceProviderError> {
    Ok(rustc_stage1_build_source(boundary)?.id.clone())
}

fn rustc_stage1_provider_receipt_artifacts(
    candidate_dir: &Path,
    route: &LoadedRustSourceProviderRoute,
) -> Result<Vec<RustProviderReceiptArtifact>, RustSourceProviderError> {
    Ok(vec![
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustc, "rustc", PROVIDER_RUSTC_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Cargo, "cargo", PROVIDER_CARGO_RELATIVE_PATH)?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::HostRustlib,
            "host-rustlib",
            &provider_rustlib_relative_path(&route.plan.host_triple),
        )?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::TargetRustlib,
            "target-rustlib",
            &provider_rustlib_relative_path(&route.plan.target_triple),
        )?,
    ])
}

fn rustc_stage1_provider_receipt(
    boundary: &RustSourceProviderRustcStage1Boundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> RustSourceProviderBuildReceipt {
    RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: rustc_stage1_provider_receipt_steps(boundary),
    }
}

fn rustc_stage1_provider_receipt_steps(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Vec<RustProviderReceiptStep> {
    vec![
        RustProviderReceiptStep {
            name: "acquire-rustc-stage1-source".to_string(),
            program: "mantle-rust-source-fetch".to_string(),
            arguments: boundary.source_ids.clone(),
        },
        RustProviderReceiptStep {
            name: "validate-bootstrap-provider".to_string(),
            program: "mantle-provider-validator".to_string(),
            arguments: vec![
                format!("metadata-digest={}", boundary.bootstrap_provider_metadata_digest_blake3),
                format!("policy-digest={}", boundary.bootstrap_provider_policy_digest_blake3),
            ],
        },
        RustProviderReceiptStep {
            name: "run-rustc-stage1".to_string(),
            program: RUSTC_STAGE1_SOURCE_BUILD_SCRIPT.to_string(),
            arguments: vec![
                format!("version={}", boundary.rust_version),
                format!("host={}", boundary.host_triple),
                format!("target={}", boundary.target_triple),
            ],
        },
        RustProviderReceiptStep {
            name: "record-rustc-stage1-products".to_string(),
            program: "mantle-provider-digest".to_string(),
            arguments: vec![
                "rustc".to_string(),
                "cargo".to_string(),
                "host-rustlib".to_string(),
                "target-rustlib".to_string(),
            ],
        },
    ]
}

fn rustc_stage1_provider_metadata(
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: RUSTC_STAGE1_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: sources.to_vec(),
        build_receipts: vec![rustc_stage1_provider_receipt_identity(receipt_digest_blake3)],
        artifacts: rustc_stage1_provider_artifacts(receipt_artifacts, receipt_digest_blake3, artifact_source_id),
    }
}

fn rustc_stage1_provider_receipt_identity(receipt_digest_blake3: &str) -> RustProviderBuildReceiptIdentity {
    RustProviderBuildReceiptIdentity {
        id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
        name: RUSTC_STAGE1_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        digest_blake3: receipt_digest_blake3.to_string(),
    }
}

fn rustc_stage1_provider_artifacts(
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: artifact_source_id.to_string(),
            build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: RUSTC_STAGE1_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: receipt_digest_blake3.to_string(),
        source_id: artifact_source_id.to_string(),
        build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
    });
    artifacts
}

fn write_rustc_stage1_provider_candidate_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "candidate": candidate,
        "candidate_only": true,
        "final_output_written": false,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.provider_candidate_manifest_path, &manifest, "rustc stage1 provider candidate manifest")
}

fn assemble_rustc_final_provider_candidate(
    boundary: &RustSourceProviderRustcFinalBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcFinalBuild,
) -> Result<RustSourceProviderRustcFinalProviderCandidate, RustSourceProviderError> {
    validate_rustc_final_candidate_request(boundary, sources, build)?;
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.stage_output_dir, &boundary.provider_candidate_dir)?;
    let source_identities = rustc_final_provider_source_identities(boundary, sources)?;
    let artifact_source_id = rustc_final_artifact_source_id(boundary)?;
    let receipt_artifacts = rustc_final_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = rustc_final_provider_receipt(boundary, route, &source_identities, &receipt_artifacts);
    let receipt_path = boundary.provider_candidate_dir.join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "rustc final provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = rustc_final_provider_metadata(
        route,
        &source_identities,
        &receipt_artifacts,
        &receipt_digest_blake3,
        &artifact_source_id,
    );
    let metadata_path = boundary.provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    write_json_pretty(&metadata_path, &metadata, "rustc final provider metadata")?;
    let validation = validate_materialized_rust_source_provider(&boundary.provider_candidate_dir)?;
    Ok(RustSourceProviderRustcFinalProviderCandidate {
        stage_id: boundary.stage_id.clone(),
        candidate_dir: boundary.provider_candidate_dir.clone(),
        metadata_path,
        metadata_digest_blake3: validation.metadata_digest_blake3,
        policy_digest_blake3: validation.validation.policy_digest_blake3,
        artifact_count: validation.validation.artifact_count,
        source_count: validation.validation.source_count,
        receipt_count: validation.validation.receipt_count,
    })
}

fn smoke_rustc_final_provider_candidate(
    boundary: &RustSourceProviderRustcFinalBoundary,
    candidate: &RustSourceProviderRustcFinalProviderCandidate,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(&candidate.candidate_dir)?;
    let smoke = smoke_rust_source_provider(&candidate.candidate_dir, &boundary.provider_candidate_smoke_work_dir)?;
    persist_rust_source_provider_smoke_evidence(&boundary.provider_candidate_smoke_dir, &smoke, &validation)
}

fn validate_rustc_final_candidate_request(
    boundary: &RustSourceProviderRustcFinalBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcFinalBuild,
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Validate("rustc final candidate source count mismatch".to_string()));
    }
    if build.stage_id != boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc final candidate build stage expected {}, got {}",
            boundary.stage_id, build.stage_id
        )));
    }
    if boundary.provider_candidate_dir == boundary.output_dir {
        return Err(RustSourceProviderError::Validate(
            "rustc final provider candidate must not be the final output dir".to_string(),
        ));
    }
    validate_first_stage_build_product(&build.stage_output_dir, RUSTC_FINAL_OUTPUT_DIR)
}

fn rustc_final_provider_source_identities(
    boundary: &RustSourceProviderRustcFinalBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<Vec<RustProviderSourceIdentity>, RustSourceProviderError> {
    let mut identities = Vec::with_capacity(boundary.sources.len() + 1);
    for source in &boundary.sources {
        let acquired = sources.iter().find(|acquired| acquired.id == source.id).ok_or_else(|| {
            RustSourceProviderError::Validate(format!("rustc final candidate lacks source acquisition '{}'", source.id))
        })?;
        identities.push(RustProviderSourceIdentity {
            id: source.id.clone(),
            kind: source.kind,
            name: source.name.clone(),
            digest_blake3: acquired.extracted_digest_blake3.clone(),
        });
    }
    identities.push(RustProviderSourceIdentity {
        id: format!("{}-bootstrap-provider", boundary.stage_id),
        kind: ToolchainSourceKind::Generated,
        name: "bootstrap-provider-candidate".to_string(),
        digest_blake3: boundary.bootstrap_provider_metadata_digest_blake3.clone(),
    });
    Ok(identities)
}

fn rustc_final_artifact_source_id(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<String, RustSourceProviderError> {
    Ok(rustc_final_build_source(boundary)?.id.clone())
}

fn rustc_final_provider_receipt_artifacts(
    candidate_dir: &Path,
    route: &LoadedRustSourceProviderRoute,
) -> Result<Vec<RustProviderReceiptArtifact>, RustSourceProviderError> {
    Ok(vec![
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustc, "rustc", PROVIDER_RUSTC_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Cargo, "cargo", PROVIDER_CARGO_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustdoc, "rustdoc", PROVIDER_RUSTDOC_RELATIVE_PATH)?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::HostRustlib,
            "host-rustlib",
            &provider_rustlib_relative_path(&route.plan.host_triple),
        )?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::TargetRustlib,
            "target-rustlib",
            &provider_rustlib_relative_path(&route.plan.target_triple),
        )?,
    ])
}

fn rustc_final_provider_receipt(
    boundary: &RustSourceProviderRustcFinalBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> RustSourceProviderBuildReceipt {
    RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: rustc_final_provider_receipt_steps(boundary),
    }
}

fn rustc_final_provider_receipt_steps(boundary: &RustSourceProviderRustcFinalBoundary) -> Vec<RustProviderReceiptStep> {
    vec![
        RustProviderReceiptStep {
            name: "acquire-rust-final-source".to_string(),
            program: "mantle-rust-source-fetch".to_string(),
            arguments: boundary.source_ids.clone(),
        },
        RustProviderReceiptStep {
            name: "validate-bootstrap-provider".to_string(),
            program: "mantle-provider-validator".to_string(),
            arguments: vec![
                format!("metadata-digest={}", boundary.bootstrap_provider_metadata_digest_blake3),
                format!("policy-digest={}", boundary.bootstrap_provider_policy_digest_blake3),
            ],
        },
        RustProviderReceiptStep {
            name: "run-rust-final".to_string(),
            program: RUSTC_FINAL_SOURCE_BUILD_SCRIPT.to_string(),
            arguments: vec![
                format!("version={}", boundary.rust_version),
                format!("host={}", boundary.host_triple),
                format!("target={}", boundary.target_triple),
            ],
        },
        RustProviderReceiptStep {
            name: "record-rust-final-products".to_string(),
            program: "mantle-provider-digest".to_string(),
            arguments: vec![
                "rustc".to_string(),
                "cargo".to_string(),
                "rustdoc".to_string(),
                "host-rustlib".to_string(),
                "target-rustlib".to_string(),
            ],
        },
    ]
}

fn rustc_final_provider_metadata(
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: RUSTC_FINAL_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: sources.to_vec(),
        build_receipts: vec![rustc_final_provider_receipt_identity(receipt_digest_blake3)],
        artifacts: rustc_final_provider_artifacts(receipt_artifacts, receipt_digest_blake3, artifact_source_id),
    }
}

fn rustc_final_provider_receipt_identity(receipt_digest_blake3: &str) -> RustProviderBuildReceiptIdentity {
    RustProviderBuildReceiptIdentity {
        id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
        kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
        name: RUSTC_FINAL_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        digest_blake3: receipt_digest_blake3.to_string(),
    }
}

fn rustc_final_provider_artifacts(
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: artifact_source_id.to_string(),
            build_receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: RUSTC_FINAL_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: receipt_digest_blake3.to_string(),
        source_id: artifact_source_id.to_string(),
        build_receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
    });
    artifacts
}

fn write_rustc_final_provider_candidate_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    candidate: &RustSourceProviderRustcFinalProviderCandidate,
    final_output_written: bool,
) -> Result<(), RustSourceProviderError> {
    let mut manifest = serde_json::json!({
        "schema": RUSTC_FINAL_PROVIDER_CANDIDATE_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "candidate": candidate,
        "candidate_only": !final_output_written,
        "final_output_path": boundary.output_dir.display().to_string(),
        "final_output_written": final_output_written,
    });
    if !final_output_written {
        manifest["next_blocked_reason"] = serde_json::Value::String(RUST_SOURCE_PROVIDER_BLOCKED_REASON.to_string());
    }
    write_json_pretty(&boundary.provider_candidate_manifest_path, &manifest, "rustc final provider candidate manifest")
}

fn run_first_stage_build(
    boundary: &RustSourceProviderFirstStageBoundary,
    verbose: bool,
) -> Result<RustSourceProviderFirstStageBuild, RustSourceProviderError> {
    let build_sources = first_stage_build_sources(boundary)?;
    validate_first_stage_build_inputs(&build_sources)?;
    if verbose {
        eprintln!("  first_stage_build_script: {}", boundary.script_path.display());
        eprintln!("  first_stage_build_log: {}", boundary.build_log_path.display());
    }
    let log = File::create(&boundary.build_log_path).map_err(|err| {
        RustSourceProviderError::Build(format!("create {}: {err}", boundary.build_log_path.display()))
    })?;
    let status = Command::new(&boundary.script_path)
        .stdout(log.try_clone().map_err(|err| {
            RustSourceProviderError::Build(format!("clone {}: {err}", boundary.build_log_path.display()))
        })?)
        .stderr(log)
        .status()
        .map_err(|err| RustSourceProviderError::Build(format!("launch {}: {err}", boundary.script_path.display())))?;
    if !status.success() {
        let log_tail = fs::read(&boundary.build_log_path)
            .map(|bytes| bounded_output_text(&bytes))
            .unwrap_or_else(|err| format!("<failed to read build log: {err}>"));
        return Err(RustSourceProviderError::Build(format!(
            "first-stage mrustc/minicargo build failed with status {status}; log={}; tail={log_tail:?}",
            boundary.build_log_path.display()
        )));
    }
    let mrustc_path = build_sources.mrustc_path.join(FIRST_STAGE_MRUSTC_BINARY);
    let minicargo_path = build_sources.mrustc_path.join(FIRST_STAGE_MINICARGO_BINARY);
    let translated_rustc_path = build_sources.mrustc_path.join(FIRST_STAGE_TRANSLATED_RUSTC_BINARY);
    let translated_cargo_path = build_sources.mrustc_path.join(FIRST_STAGE_TRANSLATED_CARGO_BINARY);
    let prefix_path = build_sources.mrustc_path.join(FIRST_STAGE_PREFIX_DIR);
    let prefix_rustlib_path = prefix_path.join("lib/rustlib").join(&boundary.host_triple).join("lib");
    let target_prefix_rustlib_path = prefix_path.join("lib/rustlib").join(&boundary.target_triple).join("lib");
    validate_first_stage_build_product(&mrustc_path, FIRST_STAGE_MRUSTC_BINARY)?;
    validate_first_stage_build_product(&minicargo_path, FIRST_STAGE_MINICARGO_BINARY)?;
    validate_first_stage_build_product(&translated_rustc_path, FIRST_STAGE_TRANSLATED_RUSTC_BINARY)?;
    validate_first_stage_build_product(&translated_cargo_path, FIRST_STAGE_TRANSLATED_CARGO_BINARY)?;
    validate_first_stage_build_product(&prefix_path, FIRST_STAGE_PREFIX_DIR)?;
    validate_first_stage_build_product(&prefix_rustlib_path, "prefix host rustlib")?;
    validate_first_stage_build_product(&target_prefix_rustlib_path, "prefix target rustlib")?;
    Ok(RustSourceProviderFirstStageBuild {
        stage_id: boundary.stage_id.clone(),
        script_path: boundary.script_path.clone(),
        log_path: boundary.build_log_path.clone(),
        mrustc_digest_blake3: file_digest_blake3(&mrustc_path)?,
        minicargo_digest_blake3: file_digest_blake3(&minicargo_path)?,
        translated_rustc_digest_blake3: file_digest_blake3(&translated_rustc_path)?,
        translated_cargo_digest_blake3: file_digest_blake3(&translated_cargo_path)?,
        prefix_digest_blake3: content_digest_blake3(&prefix_path)?,
        prefix_rustlib_digest_blake3: content_digest_blake3(&prefix_rustlib_path)?,
        target_prefix_rustlib_digest_blake3: content_digest_blake3(&target_prefix_rustlib_path)?,
        mrustc_path,
        minicargo_path,
        translated_rustc_path,
        translated_cargo_path,
        prefix_path,
        prefix_rustlib_path,
        target_prefix_rustlib_path,
    })
}

fn write_first_stage_build_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    build: &RustSourceProviderFirstStageBuild,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "build": build,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize first stage build manifest: {err}")))?;
    fs::write(&boundary.build_manifest_path, bytes).map_err(|err| {
        RustSourceProviderError::Read(format!(
            "write first stage build manifest {}: {err}",
            boundary.build_manifest_path.display()
        ))
    })
}

fn assemble_first_stage_provider_candidate(
    boundary: &RustSourceProviderFirstStageBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderFirstStageBuild,
) -> Result<RustSourceProviderFirstStageProviderCandidate, RustSourceProviderError> {
    validate_first_stage_candidate_request(boundary, sources, build)?;
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.prefix_path, &boundary.provider_candidate_dir)?;
    normalize_first_stage_provider_wrappers(&boundary.provider_candidate_dir, &boundary.host_triple)?;
    let source_identities = first_stage_provider_source_identities(boundary, sources)?;
    let build_sources = first_stage_build_sources(boundary)?;
    let artifact_source_id = build_sources.rust_id;
    let receipt_artifacts = first_stage_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = first_stage_provider_receipt(boundary, route, &source_identities, &receipt_artifacts);
    let receipt_path = boundary.provider_candidate_dir.join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "first-stage provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = first_stage_provider_metadata(
        route,
        &source_identities,
        &receipt_artifacts,
        &receipt_digest_blake3,
        &artifact_source_id,
    );
    let metadata_path = boundary.provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    write_json_pretty(&metadata_path, &metadata, "first-stage provider metadata")?;
    let validation = validate_materialized_rust_source_provider(&boundary.provider_candidate_dir)?;
    Ok(RustSourceProviderFirstStageProviderCandidate {
        stage_id: boundary.stage_id.clone(),
        candidate_dir: boundary.provider_candidate_dir.clone(),
        metadata_path,
        metadata_digest_blake3: validation.metadata_digest_blake3,
        policy_digest_blake3: validation.validation.policy_digest_blake3,
        artifact_count: validation.validation.artifact_count,
        source_count: validation.validation.source_count,
        receipt_count: validation.validation.receipt_count,
    })
}

fn smoke_first_stage_provider_candidate(
    boundary: &RustSourceProviderFirstStageBoundary,
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(&candidate.candidate_dir)?;
    let smoke = smoke_rust_source_provider(&candidate.candidate_dir, &boundary.provider_candidate_smoke_work_dir)?;
    persist_rust_source_provider_smoke_evidence(&boundary.provider_candidate_smoke_dir, &smoke, &validation)
}

fn validate_first_stage_candidate_request(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderFirstStageBuild,
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Validate("first-stage candidate source count mismatch".to_string()));
    }
    if build.stage_id != boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "first-stage candidate build stage expected {}, got {}",
            boundary.stage_id, build.stage_id
        )));
    }
    if boundary.provider_candidate_dir == boundary.output_dir {
        return Err(RustSourceProviderError::Validate(
            "provider candidate must not be the final output dir".to_string(),
        ));
    }
    validate_first_stage_build_product(&build.prefix_path, FIRST_STAGE_PREFIX_DIR)
}

fn prepare_empty_provider_candidate_dir(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", path.display())))?;
    } else if path.exists() {
        fs::remove_file(path)
            .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", path.display())))?;
    }
    fs::create_dir_all(path).map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", path.display())))
}

fn copy_provider_prefix(src: &Path, dst: &Path) -> Result<(), RustSourceProviderError> {
    let mut copied_entries = 0usize;
    copy_directory_contents(src, dst, &mut copied_entries)?;
    if copied_entries == 0 {
        return Err(RustSourceProviderError::Copy(format!("provider prefix {} was empty", src.display())));
    }
    Ok(())
}

fn normalize_first_stage_provider_wrappers(
    candidate_dir: &Path,
    host_triple: &str,
) -> Result<(), RustSourceProviderError> {
    if host_triple.trim().is_empty() {
        return Err(RustSourceProviderError::Copy("first-stage provider wrapper host triple is empty".to_string()));
    }
    let rustc_path = candidate_dir.join(PROVIDER_RUSTC_RELATIVE_PATH);
    let rustc_text = fs::read_to_string(&rustc_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "read first-stage provider rustc wrapper {}: {err}",
            rustc_path.display()
        ))
    })?;
    if !is_mrustc_dirname_rustc_wrapper(&rustc_text) {
        return Ok(());
    }
    let rustc_binary_path = candidate_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH);
    if !rustc_binary_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "mrustc rustc wrapper {} references missing {}",
            rustc_path.display(),
            rustc_binary_path.display()
        )));
    }
    let wrapper = provider_local_mrustc_rustc_wrapper(host_triple);
    fs::write(&rustc_path, wrapper).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "write normalized first-stage rustc wrapper {}: {err}",
            rustc_path.display()
        ))
    })?;
    make_executable(&rustc_path)
}

fn is_mrustc_dirname_rustc_wrapper(text: &str) -> bool {
    text.contains(MRUSTC_RUSTC_WRAPPER_DIRNAME_MARKER) && text.contains(MRUSTC_RUSTC_WRAPPER_BINARY_MARKER)
}

fn provider_local_mrustc_rustc_wrapper(host_triple: &str) -> String {
    format!(
        "#!/bin/sh\n\
set -eu\n\
self=$0\n\
case \"$self\" in\n\
  */*) self_dir=${{self%/*}} ;;\n\
  *) self_dir=. ;;\n\
esac\n\
case \"$self_dir\" in\n\
  */bin) root_dir=${{self_dir%/bin}} ;;\n\
  *) root_dir=$self_dir/.. ;;\n\
esac\n\
ld_path=\"$root_dir/lib:$root_dir/lib/rustlib/{host_triple}/lib\"\n\
if [ \"${{LD_LIBRARY_PATH+x}}\" = x ] && [ -n \"${{LD_LIBRARY_PATH}}\" ]; then\n\
  ld_path=\"$ld_path:$LD_LIBRARY_PATH\"\n\
fi\n\
LD_LIBRARY_PATH=\"$ld_path\" exec \"$self_dir/rustc_binary\" \"$@\"\n"
    )
}

fn first_stage_provider_source_identities(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<Vec<RustProviderSourceIdentity>, RustSourceProviderError> {
    let mut identities = Vec::with_capacity(boundary.sources.len());
    for source in &boundary.sources {
        let acquired = sources.iter().find(|acquired| acquired.id == source.id).ok_or_else(|| {
            RustSourceProviderError::Validate(format!("first-stage candidate lacks source acquisition '{}'", source.id))
        })?;
        identities.push(RustProviderSourceIdentity {
            id: source.id.clone(),
            kind: source.kind,
            name: source.name.clone(),
            digest_blake3: acquired.extracted_digest_blake3.clone(),
        });
    }
    Ok(identities)
}

fn first_stage_provider_receipt_artifacts(
    candidate_dir: &Path,
    route: &LoadedRustSourceProviderRoute,
) -> Result<Vec<RustProviderReceiptArtifact>, RustSourceProviderError> {
    Ok(vec![
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustc, "rustc", PROVIDER_RUSTC_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Cargo, "cargo", PROVIDER_CARGO_RELATIVE_PATH)?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::HostRustlib,
            "host-rustlib",
            &provider_rustlib_relative_path(&route.plan.host_triple),
        )?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::TargetRustlib,
            "target-rustlib",
            &provider_rustlib_relative_path(&route.plan.target_triple),
        )?,
    ])
}

fn provider_receipt_artifact(
    provider_dir: &Path,
    role: RustProviderRole,
    name: &str,
    relative_path: &str,
) -> Result<RustProviderReceiptArtifact, RustSourceProviderError> {
    Ok(RustProviderReceiptArtifact {
        role,
        name: name.to_string(),
        path: relative_path.to_string(),
        content_digest_blake3: content_digest_blake3(&provider_dir.join(relative_path))?,
    })
}

fn first_stage_provider_receipt(
    boundary: &RustSourceProviderFirstStageBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> RustSourceProviderBuildReceipt {
    RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: first_stage_provider_receipt_steps(boundary),
    }
}

fn first_stage_provider_receipt_steps(boundary: &RustSourceProviderFirstStageBoundary) -> Vec<RustProviderReceiptStep> {
    vec![
        RustProviderReceiptStep {
            name: "acquire-first-stage-sources".to_string(),
            program: "mantle-rust-source-fetch".to_string(),
            arguments: boundary.source_ids.clone(),
        },
        RustProviderReceiptStep {
            name: "build-mrustc".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![format!("CXXFLAGS={FIRST_STAGE_CXXFLAGS}")],
        },
        RustProviderReceiptStep {
            name: "build-minicargo".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_MINICARGO_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "translate-rustc".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_TRANSLATED_RUSTC_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "translate-cargo".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_TRANSLATED_CARGO_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "build-rust-host-prefix".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-C".to_string(),
                FIRST_STAGE_RUN_RUSTC_DIR.to_string(),
                format!("host={}", boundary.host_triple),
            ],
        },
        RustProviderReceiptStep {
            name: "build-rust-target-rustlib".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-C".to_string(),
                FIRST_STAGE_RUN_RUSTC_DIR.to_string(),
                format!("target={}", boundary.target_triple),
            ],
        },
    ]
}

fn first_stage_provider_metadata(
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: FIRST_STAGE_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: sources.to_vec(),
        build_receipts: vec![first_stage_provider_receipt_identity(receipt_digest_blake3)],
        artifacts: first_stage_provider_artifacts(receipt_artifacts, receipt_digest_blake3, artifact_source_id),
    }
}

fn first_stage_provider_receipt_identity(receipt_digest_blake3: &str) -> RustProviderBuildReceiptIdentity {
    RustProviderBuildReceiptIdentity {
        id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
        name: FIRST_STAGE_PROVIDER_RECEIPT_NAME.to_string(),
        path: FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        digest_blake3: receipt_digest_blake3.to_string(),
    }
}

fn first_stage_provider_artifacts(
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: artifact_source_id.to_string(),
            build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: FIRST_STAGE_PROVIDER_RECEIPT_NAME.to_string(),
        path: FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: receipt_digest_blake3.to_string(),
        source_id: artifact_source_id.to_string(),
        build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
    });
    artifacts
}

fn provider_rustlib_relative_path(triple: &str) -> String {
    format!("{PROVIDER_RUSTLIB_PREFIX}/{triple}/lib")
}

fn write_first_stage_provider_candidate_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "candidate": candidate,
        "candidate_only": true,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.provider_candidate_manifest_path, &manifest, "first-stage provider candidate manifest")
}

fn first_stage_build_sources(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<RustSourceProviderFirstStageBuildSources, RustSourceProviderError> {
    let mrustc = select_first_stage_source(boundary, FIRST_STAGE_MRUSTC_SOURCE_PREFIX)?;
    let rust = select_first_stage_source(boundary, FIRST_STAGE_RUST_SOURCE_PREFIX)?;
    let mrustc_id = safe_source_component(&mrustc.id)?;
    let rust_id = safe_source_component(&rust.id)?;
    Ok(RustSourceProviderFirstStageBuildSources {
        mrustc_path: boundary.source_dir.join(&mrustc_id),
        rust_path: boundary.source_dir.join(&rust_id),
        mrustc_archive_path: boundary.archive_dir.join(format!("{mrustc_id}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}")),
        rust_archive_path: boundary.archive_dir.join(format!("{rust_id}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}")),
        mrustc_id,
        rust_id,
    })
}

fn select_first_stage_source<'a>(
    boundary: &'a RustSourceProviderFirstStageBoundary,
    id_prefix: &str,
) -> Result<&'a RustSourceProviderBootstrapSource, RustSourceProviderError> {
    let mut matches = boundary.sources.iter().filter(|source| source.id.starts_with(id_prefix));
    let Some(source) = matches.next() else {
        return Err(RustSourceProviderError::Build(format!(
            "first stage '{}' has no source with id prefix '{id_prefix}'",
            boundary.stage_id
        )));
    };
    if matches.next().is_none() {
        return Ok(source);
    }
    Err(RustSourceProviderError::Build(format!(
        "first stage '{}' has multiple sources with id prefix '{id_prefix}'",
        boundary.stage_id
    )))
}

fn validate_first_stage_build_inputs(
    sources: &RustSourceProviderFirstStageBuildSources,
) -> Result<(), RustSourceProviderError> {
    require_first_stage_path(&sources.mrustc_path, "mrustc source dir", true)?;
    require_first_stage_path(&sources.rust_path, "Rust source dir", true)?;
    require_first_stage_path(&sources.mrustc_archive_path, "mrustc source archive", false)?;
    require_first_stage_path(&sources.rust_archive_path, "Rust source archive", false)?;
    require_first_stage_path(&sources.mrustc_path.join(FIRST_STAGE_MAKEFILE), "mrustc Makefile", false)?;
    require_first_stage_path(
        &sources.mrustc_path.join(FIRST_STAGE_MINICARGO_MAKEFILE),
        "mrustc minicargo makefile",
        false,
    )
}

fn require_first_stage_path(path: &Path, label: &str, directory: bool) -> Result<(), RustSourceProviderError> {
    if directory && path.is_dir() {
        return Ok(());
    }
    if !directory && path.is_file() {
        return Ok(());
    }
    Err(RustSourceProviderError::Build(format!("missing {label}: {}", path.display())))
}

fn validate_first_stage_build_product(path: &Path, label: &str) -> Result<(), RustSourceProviderError> {
    if path.is_file() || path.is_dir() {
        return Ok(());
    }
    Err(RustSourceProviderError::Build(format!("missing first-stage product {label}: {}", path.display())))
}

fn fetch_first_stage_source_bytes(
    source: &RustSourceProviderBootstrapSource,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let parsed = url::Url::parse(&source.url)
        .map_err(|err| RustSourceProviderError::Fetch(format!("source '{}' url parse: {err}", source.id)))?;
    match parsed.scheme() {
        "file" => read_file_url_limited(&source.id, &parsed),
        "http" | "https" => fetch_http_source_bytes(&source.id, &source.url),
        scheme => Err(RustSourceProviderError::Fetch(format!(
            "source '{}' uses unsupported url scheme '{scheme}'",
            source.id
        ))),
    }
}

fn read_file_url_limited(source_id: &str, url: &url::Url) -> Result<Vec<u8>, RustSourceProviderError> {
    let path = url
        .to_file_path()
        .map_err(|_| RustSourceProviderError::Fetch(format!("source '{source_id}' file url is not a local path")))?;
    let mut file = File::open(&path).map_err(|err| {
        RustSourceProviderError::Fetch(format!("source '{source_id}' open {}: {err}", path.display()))
    })?;
    read_limited_bytes(&mut file, &format!("source '{source_id}' {}", path.display()))
}

fn fetch_http_source_bytes(source_id: &str, url: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FIRST_STAGE_FETCH_TIMEOUT_SECS)))
        .build()
        .into();
    let mut last_error = String::new();
    for attempt in 0..=FIRST_STAGE_FETCH_MAX_RETRIES {
        if attempt != 0 {
            sleep_before_first_stage_fetch_retry(url, attempt);
        }
        match fetch_http_source_once(&agent, source_id, url) {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_error = err.to_string();
                if !is_transient_first_stage_fetch_error(&last_error) {
                    return Err(err);
                }
            }
        }
    }
    Err(RustSourceProviderError::Fetch(last_error))
}

fn fetch_http_source_once(agent: &ureq::Agent, source_id: &str, url: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let response = agent
        .get(url)
        .call()
        .map_err(|err| RustSourceProviderError::Fetch(format!("source '{source_id}' {url}: {err}")))?;
    let mut body = response.into_body();
    let mut reader = body.with_config().limit(FIRST_STAGE_FETCH_MAX_BYTES).reader();
    read_limited_bytes(&mut reader, &format!("source '{source_id}' {url}"))
}

fn sleep_before_first_stage_fetch_retry(url: &str, attempt: u32) {
    let exponent = attempt.saturating_sub(1).min(FIRST_STAGE_FETCH_MAX_RETRIES);
    let delay_ms = FIRST_STAGE_FETCH_RETRY_BASE_DELAY_MS.saturating_mul(1_u64 << exponent);
    eprintln!("  [rust-source-provider] retry {attempt}/{FIRST_STAGE_FETCH_MAX_RETRIES} for {url} after {delay_ms}ms");
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
}

fn read_limited_bytes(reader: &mut dyn Read, label: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let mut bytes = Vec::new();
    reader
        .take(FIRST_STAGE_FETCH_MAX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|err| RustSourceProviderError::Fetch(format!("{label}: reading body: {err}")))?;
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Fetch(format!("{label}: empty source body")));
    }
    Ok(bytes)
}

fn is_transient_first_stage_fetch_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("network is unreachable")
        || lower.contains("connection reset")
        || lower.contains("timed out")
        || lower.contains("try again")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
}

fn ensure_supported_first_stage_archive(url: &str) -> Result<(), RustSourceProviderError> {
    let lower = url.to_ascii_lowercase();
    if lower.ends_with(FIRST_STAGE_TAR_GZ_EXTENSION) || lower.ends_with(FIRST_STAGE_TGZ_EXTENSION) {
        return Ok(());
    }
    Err(RustSourceProviderError::Extract(format!("first stage source archive must be tar.gz/tgz: {url}")))
}

fn prepare_empty_directory(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| RustSourceProviderError::Extract(format!("remove {}: {err}", path.display())))?;
    } else if path.exists() {
        fs::remove_file(path)
            .map_err(|err| RustSourceProviderError::Extract(format!("remove {}: {err}", path.display())))?;
    }
    fs::create_dir_all(path)
        .map_err(|err| RustSourceProviderError::Extract(format!("create {}: {err}", path.display())))
}

fn extract_first_stage_tar_gz(archive_path: &Path, dest: &Path) -> Result<u32, RustSourceProviderError> {
    let file = File::open(archive_path)
        .map_err(|err| RustSourceProviderError::Extract(format!("open {}: {err}", archive_path.display())))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut entry_count = 0_u32;
    for entry in archive
        .entries()
        .map_err(|err| RustSourceProviderError::Extract(format!("read {}: {err}", archive_path.display())))?
    {
        entry_count = entry_count.checked_add(1).ok_or_else(|| {
            RustSourceProviderError::Extract("first stage archive entry count overflowed".to_string())
        })?;
        if entry_count > FIRST_STAGE_ARCHIVE_MAX_ENTRIES {
            return Err(RustSourceProviderError::Extract(format!(
                "first stage archive exceeds {FIRST_STAGE_ARCHIVE_MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(|err| RustSourceProviderError::Extract(format!("read archive entry: {err}")))?;
        unpack_first_stage_entry(entry, dest)?;
    }
    if entry_count == 0 {
        return Err(RustSourceProviderError::Extract(format!("{} is empty", archive_path.display())));
    }
    Ok(entry_count)
}

fn unpack_first_stage_entry<R: Read>(mut entry: tar::Entry<'_, R>, dest: &Path) -> Result<(), RustSourceProviderError> {
    let raw_path = entry
        .path()
        .map_err(|err| RustSourceProviderError::Extract(format!("read archive path: {err}")))?
        .into_owned();
    let Some(relative_path) = strip_first_archive_component(&raw_path)? else {
        return Ok(());
    };
    let output_path = dest.join(relative_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RustSourceProviderError::Extract(format!("create {}: {err}", parent.display())))?;
    }
    entry
        .unpack(&output_path)
        .map_err(|err| RustSourceProviderError::Extract(format!("unpack {}: {err}", output_path.display())))?;
    Ok(())
}

fn strip_first_archive_component(path: &Path) -> Result<Option<PathBuf>, RustSourceProviderError> {
    let mut components = path.components();
    let Some(first) = components.next() else {
        return Ok(None);
    };
    if !matches!(first, Component::Normal(_)) {
        return Err(RustSourceProviderError::Extract(format!("unsafe archive path {}", path.display())));
    }
    let mut stripped = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(part) => stripped.push(part),
            Component::CurDir => {}
            _ => return Err(RustSourceProviderError::Extract(format!("unsafe archive path {}", path.display()))),
        }
    }
    if stripped.as_os_str().is_empty() {
        return Ok(None);
    }
    Ok(Some(stripped))
}

fn safe_source_component(source_id: &str) -> Result<String, RustSourceProviderError> {
    let mut components = Path::new(source_id).components();
    let Some(Component::Normal(first)) = components.next() else {
        return Err(RustSourceProviderError::Validate(format!("source id '{source_id}' is not a path component")));
    };
    if components.next().is_some() {
        return Err(RustSourceProviderError::Validate(format!(
            "source id '{source_id}' is not a single path component"
        )));
    }
    Ok(first.to_string_lossy().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    <sha2::Sha256 as sha2::Digest>::update(&mut hasher, bytes);
    data_encoding::HEXLOWER.encode(&<sha2::Sha256 as sha2::Digest>::finalize(hasher))
}

fn first_stage_boundary_script(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<String, RustSourceProviderError> {
    let build_sources = first_stage_build_sources(boundary)?;
    let mut script = String::new();
    push_first_stage_script_header(&mut script, boundary, &build_sources);
    push_first_stage_source_checks(&mut script, boundary);
    push_first_stage_env_scrub(&mut script);
    push_first_stage_tool_checks(&mut script);
    push_first_stage_build_commands(&mut script);
    Ok(script)
}

fn push_first_stage_script_header(
    script: &mut String,
    boundary: &RustSourceProviderFirstStageBoundary,
    build_sources: &RustSourceProviderFirstStageBuildSources,
) {
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("RUSTC_VERSION={}\n", shell_quote(&boundary.rust_version)));
    script.push_str(&format!("RUSTC_HOST_TRIPLE={}\n", shell_quote(&boundary.host_triple)));
    script.push_str(&format!("RUSTC_TARGET={}\n", shell_quote(&boundary.host_triple)));
    script.push_str(&format!("RUSTC_PROVIDER_TARGET_TRIPLE={}\n", shell_quote(&boundary.target_triple)));
    script.push_str(&format!("MRUSTC_TARGET_VER={}\n", shell_quote(&boundary.mrustc_target_version)));
    script.push_str(&format!("SOURCE_DIR={}\n", shell_quote(&boundary.source_dir.display().to_string())));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("OUTPUT_DIR={}\n", shell_quote(&boundary.output_dir.display().to_string())));
    script.push_str(&format!("ARCHIVE_DIR={}\n", shell_quote(&boundary.archive_dir.display().to_string())));
    script.push_str(&format!("MRUSTC_SOURCE_ID={}\n", shell_quote(&build_sources.mrustc_id)));
    script.push_str(&format!("RUST_SOURCE_ID={}\n", shell_quote(&build_sources.rust_id)));
    script.push_str(&format!("MRUSTC_SOURCE={}\n", shell_quote(&build_sources.mrustc_path.display().to_string())));
    script.push_str(&format!("RUST_SOURCE={}\n", shell_quote(&build_sources.rust_path.display().to_string())));
    script.push_str(&format!(
        "MRUSTC_ARCHIVE={}\n",
        shell_quote(&build_sources.mrustc_archive_path.display().to_string())
    ));
    script.push_str(&format!("RUST_ARCHIVE={}\n", shell_quote(&build_sources.rust_archive_path.display().to_string())));
    script
        .push_str(&format!("SOURCE_MANIFEST={}\n", shell_quote(&boundary.sources_manifest_path.display().to_string())));
    script.push_str(&format!("MAKE_PROGRAM={}\n", shell_quote(FIRST_STAGE_MAKE_PROGRAM)));
    script.push_str(&format!("MAKE_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_MAKE_FALLBACK_GLOB)));
    script.push_str(&format!("COPY_PROGRAM={}\n", shell_quote(FIRST_STAGE_COPY_PROGRAM)));
    script.push_str(&format!("PKG_CONFIG_PROGRAM={}\n", shell_quote(FIRST_STAGE_PKG_CONFIG_PROGRAM)));
    script.push_str(&format!("CMAKE_PROGRAM={}\n", shell_quote(FIRST_STAGE_CMAKE_PROGRAM)));
    script.push_str(&format!("CC_PROGRAM={}\n", shell_quote(FIRST_STAGE_CC_PROGRAM)));
    script.push_str(&format!("CXX_PROGRAM={}\n", shell_quote(FIRST_STAGE_CXX_PROGRAM)));
    script.push_str(&format!("HOST_GNU_TRIPLE={}\n", shell_quote(FIRST_STAGE_HOST_GNU_TRIPLE)));
    script.push_str(&format!("CMAKE_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_CMAKE_FALLBACK_GLOB)));
    script.push_str(&format!("GCC_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_GCC_FALLBACK_GLOB)));
    script.push_str(&format!(
        "ZLIB_PKG_CONFIG_FALLBACK_GLOB={}\n",
        shell_quote(FIRST_STAGE_ZLIB_PKG_CONFIG_FALLBACK_GLOB)
    ));
    script.push_str(&format!("MRUSTC_CXXFLAGS={}\n", shell_quote(FIRST_STAGE_CXXFLAGS)));
    script.push_str(&format!("TARGET_CC_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_CC_PROGRAM)));
    script.push_str(&format!("TARGET_CXX_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_CXX_PROGRAM)));
    script.push_str(&format!("TARGET_AR_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_AR_PROGRAM)));
    script.push_str(&format!("TARGET_RANLIB_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_RANLIB_PROGRAM)));
    script.push_str(&format!(
        "TARGET_CC_PROGRAM_CANDIDATES={}\n",
        shell_quote(&musl_target_tool_program_candidates_shell_words("gcc"))
    ));
    script.push_str(&format!("TARGET_MUSL_TOOL_PREFIXES={}\n", shell_quote(&musl_target_tool_prefixes_shell_words())));
    script.push_str(&format!(
        "TARGET_MUSL_MACHINE_ALIASES={}\n",
        shell_quote(&musl_target_machine_aliases_shell_words())
    ));
    script.push_str(&format!("TARGET_MUSL_SOURCE_ROOT_SYSROOT={}\n", shell_quote(FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX)));
    script.push_str(&format!(
        "TARGET_MUSL_GCC_FALLBACK_GLOB={}\n",
        shell_quote(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB)
    ));
}

fn push_first_stage_source_checks(script: &mut String, boundary: &RustSourceProviderFirstStageBoundary) {
    script.push_str("printf '%s\\n' \"mantle rust source first stage: $STAGE_ID\"\n");
    script.push_str(&format!(
        "if [ ! -f \"$SOURCE_MANIFEST\" ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote("missing verified source manifest"),
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE
    ));
    for source in &boundary.sources {
        script.push_str(&format!(
            "if [ ! -d \"$SOURCE_DIR\"/{id} ]; then printf '%s\\n' {message} >&2; exit {missing}; fi\n",
            id = shell_quote(&source.id),
            message = shell_quote(&format!(
                "missing verified source '{}' from {} with sha256 {}",
                source.id, source.url, source.sha256_hex
            )),
            missing = FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
        ));
    }
    push_required_shell_var_file_check(
        script,
        "MRUSTC_ARCHIVE",
        "missing verified mrustc archive",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_file_check(
        script,
        "RUST_ARCHIVE",
        "missing verified Rust archive",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_mrustc_source_file_check(script, FIRST_STAGE_MAKEFILE, "mrustc source has no Makefile");
    push_mrustc_source_file_check(script, FIRST_STAGE_MINICARGO_MAKEFILE, "mrustc source has no minicargo.mk");
}

fn push_first_stage_env_scrub(script: &mut String) {
    script.push_str("printf '%s\\n' 'scrubbing inherited Cargo/build-script environment'\n");
    for var_name in FIRST_STAGE_ENV_SCRUB_VARS {
        script.push_str(&format!("unset {var_name}\n"));
    }
}

fn push_first_stage_tool_checks(script: &mut String) {
    script.push_str(
        "if ! command -v \"$MAKE_PROGRAM\" >/dev/null 2>&1; then for candidate in $MAKE_FALLBACK_GLOB; do if [ -x \"$candidate\" ]; then MAKE_PROGRAM=\"$candidate\"; break; fi; done; fi\n",
    );
    script.push_str(
        "if command -v \"$MAKE_PROGRAM\" >/dev/null 2>&1; then make_dir=$(dirname \"$(command -v \"$MAKE_PROGRAM\")\"); case \":$PATH:\" in *\":$make_dir:\"*) ;; *) PATH=\"$make_dir:$PATH\"; export PATH;; esac; fi\n",
    );
    script.push_str(
        "if ! command -v \"$CMAKE_PROGRAM\" >/dev/null 2>&1; then for candidate in $CMAKE_FALLBACK_GLOB; do if [ -x \"$candidate\" ]; then CMAKE_PROGRAM=\"$candidate\"; break; fi; done; fi\n",
    );
    script.push_str(
        "if command -v \"$CMAKE_PROGRAM\" >/dev/null 2>&1; then cmake_dir=$(dirname \"$(command -v \"$CMAKE_PROGRAM\")\"); case \":$PATH:\" in *\":$cmake_dir:\"*) ;; *) PATH=\"$cmake_dir:$PATH\"; export PATH;; esac; fi\n",
    );
    push_first_stage_gcc_toolchain(script);
    push_required_shell_command_check(script, "MAKE_PROGRAM", "make is required for mrustc first stage");
    push_required_shell_command_check(script, "COPY_PROGRAM", "cp is required for mrustc first stage");
    push_required_shell_command_check(script, "CMAKE_PROGRAM", "cmake is required for mrustc first stage");
    push_required_shell_command_check(script, "CC_PROGRAM", "cc is required for mrustc first stage");
    push_required_shell_command_check(script, "CXX_PROGRAM", "c++ is required for mrustc first stage");
    push_first_stage_zlib_flags(script);
}

fn push_first_stage_gcc_toolchain(script: &mut String) {
    script.push_str("for candidate in $GCC_FALLBACK_GLOB; do\n");
    script.push_str("  candidate_dir=$(dirname \"$candidate\")\n");
    script.push_str("  candidate_gcc=\"$candidate_dir/gcc\"\n");
    script.push_str("  if [ ! -x \"$candidate\" ] || [ ! -x \"$candidate_gcc\" ]; then continue; fi\n");
    script.push_str("  candidate_machine=$(\"$candidate_gcc\" -dumpmachine 2>/dev/null || true)\n");
    script.push_str("  if [ \"$candidate_machine\" != \"$HOST_GNU_TRIPLE\" ]; then continue; fi\n");
    script.push_str(
        "  case \":$PATH:\" in *\":$candidate_dir:\"*) ;; *) PATH=\"$candidate_dir:$PATH\"; export PATH;; esac\n",
    );
    script.push_str("  CC_PROGRAM=\"$candidate_gcc\"\n");
    script.push_str("  CXX_PROGRAM=\"$candidate\"\n");
    script.push_str("  printf '%s\\n' \"using GCC toolchain: $candidate_dir ($candidate_machine)\"\n");
    script.push_str("  break\n");
    script.push_str("done\n");
}

fn push_first_stage_zlib_flags(script: &mut String) {
    script.push_str("ZLIB_CFLAGS=\n");
    script.push_str("ZLIB_LIBS=\n");
    script.push_str("if command -v \"$PKG_CONFIG_PROGRAM\" >/dev/null 2>&1; then\n");
    script.push_str("  if ! \"$PKG_CONFIG_PROGRAM\" --exists zlib >/dev/null 2>&1; then\n");
    script.push_str(
        "    for pc_dir in $ZLIB_PKG_CONFIG_FALLBACK_GLOB; do if [ -f \"$pc_dir/zlib.pc\" ]; then PKG_CONFIG_PATH=\"${PKG_CONFIG_PATH:+$PKG_CONFIG_PATH:}$pc_dir\"; export PKG_CONFIG_PATH; break; fi; done\n",
    );
    script.push_str("  fi\n");
    script.push_str("  if \"$PKG_CONFIG_PROGRAM\" --exists zlib >/dev/null 2>&1; then\n");
    script.push_str("    ZLIB_CFLAGS=$(\"$PKG_CONFIG_PROGRAM\" --cflags zlib)\n");
    script.push_str("    ZLIB_LIBS=$(\"$PKG_CONFIG_PROGRAM\" --libs zlib)\n");
    script.push_str("    printf '%s\\n' \"using zlib pkg-config flags: $ZLIB_CFLAGS $ZLIB_LIBS\"\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' 'zlib pkg-config metadata not found; continuing without zlib flags'\n");
    script.push_str("  fi\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' 'pkg-config not found; continuing without zlib flags'\n");
    script.push_str("fi\n");
}

fn push_first_stage_minicargo_workspace_boundary(script: &mut String) {
    let workspace_members_line = format!("members = [\"{FIRST_STAGE_PROC_MACRO_WORKSPACE_MEMBER}\"]");
    let workspace_resolver_line = format!("resolver = \"{FIRST_STAGE_MINICARGO_WORKSPACE_RESOLVER}\"");
    script.push_str("printf '%s\\n' 'writing minicargo workspace boundary'\n");
    script.push_str(&format!("printf '%s\\n' '[workspace]' > {FIRST_STAGE_WORKSPACE_BOUNDARY_FILE}\n"));
    script.push_str(&format!(
        "printf '%s\\n' {} >> {FIRST_STAGE_WORKSPACE_BOUNDARY_FILE}\n",
        shell_quote(&workspace_members_line)
    ));
    script.push_str(&format!(
        "printf '%s\\n' {} >> {FIRST_STAGE_WORKSPACE_BOUNDARY_FILE}\n",
        shell_quote(&workspace_resolver_line)
    ));
}

fn push_first_stage_target_musl_lfs_compat(script: &mut String) {
    script.push_str(&format!(
        "target_lfs_compat_source=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_SOURCE}\"\n"
    ));
    script.push_str(&format!(
        "target_lfs_compat_object=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT}\"\n"
    ));
    script.push_str("cat > \"$target_lfs_compat_source\" <<'MANTLE_MUSL_LFS_COMPAT_C'\n");
    script.push_str(
        r#"#define _LARGEFILE64_SOURCE 1
#include <dirent.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stddef.h>
#include <sys/mman.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#ifdef fstat64
#undef fstat64
#endif
#ifdef fstatat64
#undef fstatat64
#endif
#ifdef ftruncate64
#undef ftruncate64
#endif
#ifdef sendfile64
#undef sendfile64
#endif
#ifdef lseek64
#undef lseek64
#endif
#ifdef lstat64
#undef lstat64
#endif
#ifdef mmap64
#undef mmap64
#endif
#ifdef open64
#undef open64
#endif
#ifdef openat64
#undef openat64
#endif
#ifdef pread64
#undef pread64
#endif
#ifdef pwrite64
#undef pwrite64
#endif
#ifdef readdir64
#undef readdir64
#endif
#ifdef stat64
#undef stat64
#endif

static int mantle_open_flags_need_mode(int flags) {
    if ((flags & O_CREAT) != 0) {
        return 1;
    }
#ifdef O_TMPFILE
    if ((flags & O_TMPFILE) == O_TMPFILE) {
        return 1;
    }
#endif
    return 0;
}

int fstat64(int fd, struct stat *buf) {
    return fstat(fd, buf);
}

int fstatat64(int dirfd, const char *pathname, struct stat *buf, int flags) {
    return fstatat(dirfd, pathname, buf, flags);
}

int ftruncate64(int fd, off_t length) {
    return ftruncate(fd, length);
}

ssize_t sendfile64(int out_fd, int in_fd, off_t *offset, size_t count) {
    return sendfile(out_fd, in_fd, offset, count);
}

off_t lseek64(int fd, off_t offset, int whence) {
    return lseek(fd, offset, whence);
}

int lstat64(const char *pathname, struct stat *buf) {
    return lstat(pathname, buf);
}

void *mmap64(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return mmap(addr, length, prot, flags, fd, offset);
}

int open64(const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return open(pathname, flags, mode);
    }
    return open(pathname, flags);
}

int openat64(int dirfd, const char *pathname, int flags, ...) {
    mode_t mode = 0;
    if (mantle_open_flags_need_mode(flags) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
        return openat(dirfd, pathname, flags, mode);
    }
    return openat(dirfd, pathname, flags);
}

ssize_t pread64(int fd, void *buf, size_t count, off_t offset) {
    return pread(fd, buf, count, offset);
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off_t offset) {
    return pwrite(fd, buf, count, offset);
}

struct dirent *readdir64(DIR *dirp) {
    return readdir(dirp);
}

int stat64(const char *pathname, struct stat *buf) {
    return stat(pathname, buf);
}
"#,
    );
    script.push_str("MANTLE_MUSL_LFS_COMPAT_C\n");
    script.push_str(&format!(
        "\"$target_cc_path\" -D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -c \"$target_lfs_compat_source\" -o \"$target_lfs_compat_object\"\n"
    ));
}

fn push_first_stage_target_linker_wrapper(script: &mut String) {
    script.push_str(&format!(
        "if [ \"$RUSTC_TARGET\" = \"$RUSTC_PROVIDER_TARGET_TRIPLE\" ] && [ \"$RUSTC_TARGET\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"
    ));
    script.push_str("if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then for candidate_name in $TARGET_CC_PROGRAM_CANDIDATES; do if command -v \"$candidate_name\" >/dev/null 2>&1; then TARGET_CC_PROGRAM=\"$candidate_name\"; break; fi; done; fi\n");
    script.push_str("if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then for candidate in $TARGET_MUSL_GCC_FALLBACK_GLOB; do if [ -x \"$candidate\" ]; then TARGET_CC_PROGRAM=\"$candidate\"; break; fi; done; fi\n");
    script.push_str(&format!(
        "if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then printf '%s\\n' 'x86_64 musl target gcc is required for run_rustc target linking' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("target_cc_path=$(command -v \"$TARGET_CC_PROGRAM\")\n");
    script.push_str("target_cc_machine=$(\"$target_cc_path\" -dumpmachine 2>/dev/null || true)\n");
    script.push_str("case \" $TARGET_MUSL_MACHINE_ALIASES \" in *\" $target_cc_machine \"*) ;; *) printf '%s\\n' \"target gcc machine expected one of: $TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine\" >&2; exit ");
    script.push_str(&FIRST_STAGE_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str(" ;; esac\n");
    script.push_str("target_tool_dir=$(dirname \"$target_cc_path\")\n");
    script.push_str("target_wrapper_root=$(dirname \"$target_tool_dir\")\n");
    script.push_str("target_cc_basename=${target_cc_path##*/}\n");
    script.push_str("case \"$target_cc_basename\" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac\n");
    script.push_str("if [ -z \"$target_tool_prefix\" ]; then printf '%s\\n' \"target gcc name does not end in -gcc: $target_cc_basename\" >&2; exit ");
    script.push_str(&FIRST_STAGE_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("target_cxx_program=$target_tool_prefix-g++\n");
    script.push_str("target_ar_program=$target_tool_prefix-ar\n");
    script.push_str("target_ranlib_program=$target_tool_prefix-ranlib\n");
    script.push_str(&format!("target_nix_support=\"$target_wrapper_root/{FIRST_STAGE_TARGET_NIX_SUPPORT_DIR}\"\n"));
    script
        .push_str(&format!("target_orig_libc_file=\"$target_nix_support/{FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE}\"\n"));
    script.push_str(&format!("target_orig_cc_file=\"$target_nix_support/{FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE}\"\n"));
    script.push_str("if [ -f \"$target_orig_libc_file\" ] && [ -f \"$target_orig_cc_file\" ]; then\n");
    script.push_str("  target_libc_root=$(cat \"$target_orig_libc_file\")\n");
    script.push_str("  target_orig_cc_root=$(cat \"$target_orig_cc_file\")\n");
    script.push_str("  target_gcc_crt_machine=$RUSTC_TARGET\n");
    script.push_str("else\n");
    script.push_str("  target_libc_root=\"$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT\"\n");
    script.push_str("  target_orig_cc_root=\"$target_wrapper_root\"\n");
    script.push_str("  target_gcc_crt_machine=$target_cc_machine\n");
    script.push_str("fi\n");
    script.push_str("target_musl_crt_dir=\"$target_libc_root/lib\"\n");
    script.push_str("target_gcc_version=$(\"$target_cc_path\" -dumpfullversion 2>/dev/null || \"$target_cc_path\" -dumpversion 2>/dev/null || true)\n");
    script
        .push_str("target_gcc_crt_dir=\"$target_orig_cc_root/lib/gcc/$target_gcc_crt_machine/$target_gcc_version\"\n");
    script.push_str("if [ ! -f \"$target_gcc_crt_dir/crtbeginS.o\" ]; then for candidate_dir in \"$target_orig_cc_root\"/lib/gcc/\"$target_gcc_crt_machine\"/*; do if [ -f \"$candidate_dir/crtbeginS.o\" ]; then target_gcc_crt_dir=\"$candidate_dir\"; break; fi; done; fi\n");
    script.push_str(&format!(
        "if [ ! -f \"$target_musl_crt_dir/rcrt1.o\" ] || [ ! -f \"$target_musl_crt_dir/crti.o\" ] || [ ! -f \"$target_musl_crt_dir/crtn.o\" ] || [ ! -f \"$target_gcc_crt_dir/crtbeginS.o\" ] || [ ! -f \"$target_gcc_crt_dir/libgcc.a\" ]; then printf '%s\\n' 'target gcc toolchain does not expose musl/gcc CRT and unwinder objects' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!("export {FIRST_STAGE_TARGET_NIX_CC_WRAPPER_HOST_ROLE_VAR}=1\n"));
    script.push_str("export COPY_PROGRAM\n");
    script.push_str(&format!("target_alias_dir=\"$BUILD_DIR/{FIRST_STAGE_TARGET_LINKER_ALIAS_DIR}\"\n"));
    script.push_str(&format!("target_runtime_dir=\"$BUILD_DIR/{FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR}\"\n"));
    script.push_str("mkdir -p \"$target_alias_dir\" \"$target_runtime_dir\"\n");
    script.push_str("for crt_name in rcrt1.o crti.o crtn.o; do rm -f \"$target_runtime_dir/$crt_name\"; $COPY_PROGRAM \"$target_musl_crt_dir/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n");
    script.push_str("for crt_name in crtbeginS.o crtendS.o; do rm -f \"$target_runtime_dir/$crt_name\"; $COPY_PROGRAM \"$target_gcc_crt_dir/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n");
    script.push_str("target_unwind_archive=\"$target_gcc_crt_dir/libgcc_eh.a\"\n");
    script.push_str(
        "if [ ! -f \"$target_unwind_archive\" ]; then target_unwind_archive=\"$target_gcc_crt_dir/libgcc.a\"; fi\n",
    );
    script.push_str("rm -f \"$target_runtime_dir/libunwind.a\"; $COPY_PROGRAM \"$target_unwind_archive\" \"$target_runtime_dir/libunwind.a\"\n");
    script.push_str("rm -f \"$target_runtime_dir/libgcc.a\"; $COPY_PROGRAM \"$target_gcc_crt_dir/libgcc.a\" \"$target_runtime_dir/libgcc.a\"\n");
    script.push_str("rm -f \"$target_runtime_dir/libgcc_s.a\"; $COPY_PROGRAM \"$target_gcc_crt_dir/libgcc.a\" \"$target_runtime_dir/libgcc_s.a\"\n");
    push_first_stage_target_musl_lfs_compat(script);
    script.push_str("printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'set -eu' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' \"target_cc_path=\\\"$target_cc_path\\\"\" >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' \"target_runtime_dir=\\\"$target_runtime_dir\\\"\" >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'mapped_args_set=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    rcrt1.o|crti.o|crtn.o|crtbeginS.o|crtendS.o) mapped_arg=\"$target_runtime_dir/$arg\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -static-pie) mapped_arg=\"-static\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    *) mapped_arg=\"$arg\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  if [ \"$mapped_args_set\" = false ]; then set -- \"$mapped_arg\"; mapped_args_set=true; else set -- \"$@\" \"$mapped_arg\"; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "printf '%s\\n' 'if [ \"$mapped_args_set\" = false ]; then set --; fi' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("printf '%s\\n' 'link_command=true' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -c|-S|-E) link_command=false ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(&format!(
        "printf '%s\\n' 'if [ \"$link_command\" = true ]; then set -- \"$@\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT}\"; fi' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str(&format!(
        "printf '%s\\n' 'exec \"$target_cc_path\" -D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -B\"$target_runtime_dir/\" -L\"$target_runtime_dir\" \"$@\"' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str("chmod +x \"$target_alias_dir/cc\"\n");
    script.push_str("PATH=\"$target_alias_dir:$PATH\"\n");
    script.push_str("export PATH\n");
    script.push_str("for target_tool in c++ ar ranlib; do case \"$target_tool\" in c++) target_program=\"$target_cxx_program\" ;; ar) target_program=\"$target_ar_program\" ;; ranlib) target_program=\"$target_ranlib_program\" ;; esac; if [ -x \"$target_tool_dir/$target_program\" ]; then printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/$target_tool\"; printf '%s\\n' \"exec \\\"$target_tool_dir/$target_program\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/$target_tool\"; chmod +x \"$target_alias_dir/$target_tool\"; fi; done\n");
    script.push_str(&format!("export {FIRST_STAGE_TARGET_MRUSTC_CC_ENV_VAR}=\"$target_alias_dir/cc\"\n"));
    script.push_str(&format!("export {FIRST_STAGE_TARGET_CARGO_CC_ENV_VAR}=\"$target_alias_dir/cc\"\n"));
    script.push_str(&format!(
        "if [ -x \"$target_alias_dir/c++\" ]; then export {FIRST_STAGE_TARGET_CARGO_CXX_ENV_VAR}=\"$target_alias_dir/c++\"; fi\n"
    ));
    script.push_str(&format!(
        "if [ -x \"$target_alias_dir/ar\" ]; then export {FIRST_STAGE_TARGET_CARGO_AR_ENV_VAR}=\"$target_alias_dir/ar\"; fi\n"
    ));
    script.push_str(&format!(
        "if [ -x \"$target_alias_dir/ranlib\" ]; then export {FIRST_STAGE_TARGET_CARGO_RANLIB_ENV_VAR}=\"$target_alias_dir/ranlib\"; fi\n"
    ));
    script.push_str("export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=\"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' \"using target linker wrapper: $target_alias_dir/cc -> $target_cc_path ($target_cc_machine); runtime CRT/unwind dir: $target_runtime_dir\"\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' \"using compiler-host linker for $RUSTC_TARGET; target sysroot remains $RUSTC_PROVIDER_TARGET_TRIPLE\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_build_commands(script: &mut String) {
    script.push_str(
        "printf '%s\\n' \"verified sources manifest: $SOURCE_MANIFEST\"\nprintf '%s\\n' \"build dir: $BUILD_DIR\"\nprintf '%s\\n' \"output dir: $OUTPUT_DIR\"\n",
    );
    script.push_str("mkdir -p \"$BUILD_DIR\"\n");
    script.push_str("$COPY_PROGRAM \"$RUST_ARCHIVE\" \"$MRUSTC_SOURCE/rustc-${RUSTC_VERSION}-src.tar.gz\"\n");
    script.push_str("cd \"$MRUSTC_SOURCE\"\n");
    push_first_stage_minicargo_workspace_boundary(script);
    script.push_str("export RUSTC_TARGET MRUSTC_TARGET_VER RUSTC_VERSION\n");
    script.push_str("export RUSTC_INSTALL_BINDIR=bin\n");
    script.push_str("export OUTDIR_SUF=\n");
    script.push_str("export CARGO_CFG_RUSTIX_NO_LINUX_RAW=1\n");
    script.push_str("export CC=\"$CC_PROGRAM\"\n");
    script.push_str("export CXX=\"$CXX_PROGRAM\"\n");
    script.push_str("export CFLAGS=\"$ZLIB_CFLAGS\"\n");
    script.push_str("export CPPFLAGS=\"$ZLIB_CFLAGS\"\n");
    script.push_str("export CXXFLAGS=\"$MRUSTC_CXXFLAGS $ZLIB_CFLAGS\"\n");
    script.push_str("export LDFLAGS=\"$ZLIB_LIBS\"\n");
    script.push_str("export LIBS=\"$ZLIB_LIBS\"\n");
    script
        .push_str("$MAKE_PROGRAM CC=\"$CC\" CXX=\"$CXX\" CXXFLAGS=\"$CXXFLAGS\" LDFLAGS=\"$LDFLAGS\" LIBS=\"$LIBS\"\n");
    script.push_str(&format!("$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_MINICARGO_BINARY}\n"));
    script.push_str(&format!(
        "$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_RUSTC_BINARY}\n"
    ));
    script.push_str(&format!(
        "$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_CARGO_BINARY}\n"
    ));
    push_first_stage_run_rustc_host(script);
    push_first_stage_run_rustc_target(script);
    push_required_relative_file_check(script, FIRST_STAGE_MRUSTC_BINARY, "mrustc build did not produce bin/mrustc");
    push_required_relative_file_check(
        script,
        FIRST_STAGE_MINICARGO_BINARY,
        "mrustc build did not produce bin/minicargo",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_TRANSLATED_RUSTC_BINARY,
        "mrustc build did not produce output/rustc",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_TRANSLATED_CARGO_BINARY,
        "mrustc build did not produce output/cargo",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_PREFIX_RUSTC_BINARY,
        "mrustc run_rustc did not produce prefix rustc",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_PREFIX_CARGO_BINARY,
        "mrustc run_rustc did not produce prefix cargo",
    );
    script.push_str("printf '%s\\n' \"Rust 1.90 first-stage products ready\"\n");
}

fn push_first_stage_run_rustc_host(script: &mut String) {
    script.push_str("RUSTC_TARGET=\"$RUSTC_HOST_TRIPLE\"\n");
    push_first_stage_target_linker_wrapper(script);
    push_first_stage_run_rustc_dylib_ext(script);
    script.push_str(&format!(
        "if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM -C {FIRST_STAGE_RUN_RUSTC_DIR} DYLIB_EXT=\"$RUN_RUSTC_DYLIB_EXT\"; else $MAKE_PROGRAM -C {FIRST_STAGE_RUN_RUSTC_DIR}; fi\n"
    ));
}

fn push_first_stage_run_rustc_target(script: &mut String) {
    script.push_str("if [ \"$RUSTC_PROVIDER_TARGET_TRIPLE\" != \"$RUSTC_HOST_TRIPLE\" ]; then\n");
    script.push_str("RUSTC_TARGET=\"$RUSTC_PROVIDER_TARGET_TRIPLE\"\n");
    push_first_stage_target_linker_wrapper(script);
    push_first_stage_run_rustc_dylib_ext(script);
    script.push_str(&format!("target_outdir_suffix={FIRST_STAGE_TARGET_OUTDIR_SUFFIX}-\"$RUSTC_TARGET\"\n"));
    script.push_str(&format!("target_prefix_s={FIRST_STAGE_TARGET_PREFIX_S_DIR}\n"));
    script.push_str("target_libdir=\"$target_prefix_s/lib/rustlib/$RUSTC_TARGET/lib\"\n");
    script.push_str("target_libstd=\"$target_libdir/libstd.rlib\"\n");
    script.push_str("target_minicargo_flags=\"--target $RUSTC_TARGET\"\n");
    script.push_str("target_bin_dir=\"$target_prefix_s/bin\"\n");
    script.push_str("target_std_env_arch=${RUSTC_TARGET%%-*}\n");
    script.push_str("target_sysroot_source=\"rustc-${RUSTC_VERSION}-src/library/sysroot\"\n");
    script.push_str("mkdir -p \"$target_bin_dir\" \"$target_libdir\"\n");
    script.push_str("$COPY_PROGRAM output/rustc \"$target_bin_dir/rustc\"\n");
    script.push_str("$COPY_PROGRAM output/cargo \"$target_bin_dir/cargo\"\n");
    script.push_str("chmod +x \"$target_bin_dir/rustc\" \"$target_bin_dir/cargo\"\n");
    script.push_str(
        "STD_ENV_ARCH=\"$target_std_env_arch\" MRUSTC_PATH=\"$(pwd)/$target_bin_dir/rustc\" bin/minicargo --vendor-dir \"rustc-${RUSTC_VERSION}-src/vendor\" --script-overrides \"script-overrides/stable-${RUSTC_VERSION}-linux/\" --output-dir \"$target_libdir\" $target_minicargo_flags \"$target_sysroot_source\"\n",
    );
    script.push_str(&format!(
        "if [ ! -f \"$target_libstd\" ]; then printf '%s\\n' 'target rustlib build did not produce libstd.rlib' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!("prefix_target_libdir={FIRST_STAGE_PREFIX_DIR}/lib/rustlib/\"$RUSTC_TARGET\"/lib\n"));
    script.push_str("mkdir -p \"$prefix_target_libdir\"\n");
    script.push_str("$COPY_PROGRAM \"$target_libdir\"/* \"$prefix_target_libdir/\"\n");
    script.push_str("RUSTC_TARGET=\"$RUSTC_HOST_TRIPLE\"\n");
    script.push_str("else\n");
    script.push_str("  printf '%s\\n' 'compiler host and provider target triples are identical; using host rustlib for target role'\n");
    script.push_str("fi\n");
}

fn push_first_stage_run_rustc_dylib_ext(script: &mut String) {
    script.push_str(&format!(
        "if [ \"$RUSTC_TARGET\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then RUN_RUSTC_DYLIB_EXT={FIRST_STAGE_STATIC_MUSL_DYLIB_EXT}; else RUN_RUSTC_DYLIB_EXT=; fi\n"
    ));
}

fn push_mrustc_source_file_check(script: &mut String, file_name: &str, message: &str) {
    script.push_str(&format!(
        "if [ ! -f \"$MRUSTC_SOURCE/{file_name}\" ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_var_file_check(script: &mut String, shell_var: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -f \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_var_dir_check(script: &mut String, shell_var: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -d \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_file_check(script: &mut String, shell_expr: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -f {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_dir_check(script: &mut String, shell_expr: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -d {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_relative_file_check(script: &mut String, relative_path: &str, message: &str) {
    script.push_str(&format!(
        "if [ ! -f {relative_path} ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_command_check(script: &mut String, shell_var: &str, message: &str) {
    script.push_str(&format!(
        "if ! command -v \"${shell_var}\" >/dev/null 2>&1; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn musl_target_tool_program_candidates(tool_suffix: &str) -> Vec<String> {
    assert!(!tool_suffix.trim().is_empty());
    assert!(!FIRST_STAGE_TARGET_MUSL_TOOL_PREFIXES.is_empty());
    FIRST_STAGE_TARGET_MUSL_TOOL_PREFIXES
        .iter()
        .map(|prefix| format!("{prefix}-{tool_suffix}"))
        .collect()
}

fn musl_target_tool_prefixes_shell_words() -> String {
    shell_words(FIRST_STAGE_TARGET_MUSL_TOOL_PREFIXES.iter().copied())
}

fn musl_target_machine_aliases_shell_words() -> String {
    shell_words(FIRST_STAGE_TARGET_MUSL_MACHINE_ALIASES.iter().copied())
}

fn musl_target_tool_program_candidates_shell_words(tool_suffix: &str) -> String {
    let candidates = musl_target_tool_program_candidates(tool_suffix);
    assert!(!candidates.is_empty());
    shell_words(candidates.iter().map(String::as_str))
}

fn shell_words<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let words: Vec<&str> = values.into_iter().collect();
    assert!(!words.is_empty());
    assert!(words.iter().all(|word| !word.trim().is_empty()));
    assert!(words.iter().all(|word| !word.chars().any(char::is_whitespace)));
    words.join(" ")
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\\''");
    format!("'{escaped}'")
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), RustSourceProviderError> {
    use std::os::unix::fs::PermissionsExt;

    const EXECUTABLE_FILE_MODE: u32 = 0o755;
    let mut permissions = fs::metadata(path)
        .map_err(|err| RustSourceProviderError::Read(format!("metadata {}: {err}", path.display())))?
        .permissions();
    permissions.set_mode(EXECUTABLE_FILE_MODE);
    fs::set_permissions(path, permissions)
        .map_err(|err| RustSourceProviderError::Read(format!("chmod {}: {err}", path.display())))
}

#[cfg(not(unix))]
fn make_executable(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.is_file() {
        return Ok(());
    }
    Err(RustSourceProviderError::Read(format!("first stage script {} is missing", path.display())))
}

fn plan_smoke(
    provider_dir: &Path,
    scratch_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderSmokePlan, RustSourceProviderError> {
    if provider_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("provider dir is empty".to_string()));
    }
    if scratch_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("smoke scratch dir is empty".to_string()));
    }
    if metadata.host_triple.trim().is_empty() {
        return Err(RustSourceProviderError::Smoke("provider host triple is empty".to_string()));
    }
    let rustc_path = provider_dir.join("bin/rustc");
    Ok(RustSourceProviderSmokePlan {
        provider_dir: provider_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        rustc_path,
        source_path: scratch_dir.join(SMOKE_SOURCE_FILE),
        output_path: scratch_dir.join(SMOKE_OUTPUT_FILE),
        target_triple: metadata.host_triple.clone(),
    })
}

fn smoke_source_text() -> String {
    format!(
        "#![no_std]\n#[no_mangle]\npub extern \"C\" fn mantle_rust_provider_smoke() -> u32 {{ {SMOKE_RETURN_CODE} }}\n"
    )
}

fn bounded_output_text(bytes: &[u8]) -> String {
    if bytes.len() <= SMOKE_OUTPUT_CAPTURE_BYTES {
        return String::from_utf8_lossy(bytes).to_string();
    }
    let mut text = String::from_utf8_lossy(&bytes[..SMOKE_OUTPUT_CAPTURE_BYTES]).to_string();
    text.push_str("\n<truncated>");
    text
}

fn copy_provider_tree(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    let parent = output_dir
        .parent()
        .ok_or_else(|| RustSourceProviderError::Copy(format!("{} has no parent", output_dir.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", parent.display())))?;
    fs::create_dir(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", output_dir.display())))?;
    let mut copied_entries = 0usize;
    if let Err(err) = copy_directory_contents(import_dir, output_dir, &mut copied_entries) {
        let _ = remove_import_output(output_dir);
        return Err(err);
    }
    Ok(())
}

fn copy_directory_contents(src: &Path, dst: &Path, copied_entries: &mut usize) -> Result<(), RustSourceProviderError> {
    for entry in
        fs::read_dir(src).map_err(|err| RustSourceProviderError::Copy(format!("read dir {}: {err}", src.display())))?
    {
        *copied_entries = copied_entries
            .checked_add(1)
            .ok_or_else(|| RustSourceProviderError::Copy("provider import entry count overflowed".to_string()))?;
        if *copied_entries > DIRECTORY_DIGEST_MAX_ENTRIES {
            return Err(RustSourceProviderError::Copy(format!(
                "provider import has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(|err| RustSourceProviderError::Copy(format!("read dir entry: {err}")))?;
        let source_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|err| RustSourceProviderError::Copy(format!("file type {}: {err}", source_path.display())))?;
        if file_type.is_dir() {
            fs::create_dir(&dest_path)
                .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", dest_path.display())))?;
            copy_directory_contents(&source_path, &dest_path, copied_entries)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &dest_path).map_err(|err| {
                RustSourceProviderError::Copy(format!(
                    "copy {} -> {}: {err}",
                    source_path.display(),
                    dest_path.display()
                ))
            })?;
        } else if file_type.is_symlink() {
            copy_symlink(&source_path, &dest_path)?;
        } else {
            return Err(RustSourceProviderError::Copy(format!("unsupported file type at {}", source_path.display())));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source_path: &Path, dest_path: &Path) -> Result<(), RustSourceProviderError> {
    let target = fs::read_link(source_path)
        .map_err(|err| RustSourceProviderError::Copy(format!("readlink {}: {err}", source_path.display())))?;
    std::os::unix::fs::symlink(&target, dest_path).map_err(|err| {
        RustSourceProviderError::Copy(format!("symlink {} -> {}: {err}", dest_path.display(), target.display()))
    })
}

#[cfg(not(unix))]
fn copy_symlink(source_path: &Path, _dest_path: &Path) -> Result<(), RustSourceProviderError> {
    Err(RustSourceProviderError::Copy(format!(
        "symlink import is unsupported on this platform at {}",
        source_path.display()
    )))
}

fn remove_import_output(output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if !output_dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", output_dir.display())))
}

fn observed_provider_artifacts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedArtifact>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.artifacts.len());
    for artifact in &metadata.artifacts {
        let path = provider_dir.join(&artifact.path);
        if !path.exists() {
            return Err(RustSourceProviderError::MissingArtifact(format!("{} at {}", artifact.path, path.display())));
        }
        observed.push(RustProviderObservedArtifact {
            role: artifact.role,
            path: artifact.path.clone(),
            content_digest_blake3: observed_provider_artifact_digest(artifact.role, &path)?,
        });
    }
    Ok(observed)
}

fn observed_provider_receipts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedBuildReceipt>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.build_receipts.len());
    for receipt in &metadata.build_receipts {
        let path = provider_dir.join(&receipt.path);
        let bytes = fs::read(&path)
            .map_err(|err| RustSourceProviderError::Read(format!("provider receipt {}: {err}", path.display())))?;
        if bytes.is_empty() {
            return Err(RustSourceProviderError::Read(format!("provider receipt {} is empty", path.display())));
        }
        let parsed = serde_json::from_slice::<RustSourceProviderBuildReceipt>(&bytes)
            .map_err(|err| RustSourceProviderError::Parse(format!("provider receipt {}: {err}", path.display())))?;
        observed.push(RustProviderObservedBuildReceipt {
            path: receipt.path.clone(),
            content_digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
            receipt: parsed,
        });
    }
    Ok(observed)
}

fn observed_provider_artifact_digest(role: RustProviderRole, path: &Path) -> Result<String, RustSourceProviderError> {
    if path.is_file() {
        let bytes =
            fs::read(path).map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", path.display())))?;
        validate_observed_provider_artifact_bytes(role, path, &bytes)?;
        return Ok(blake3::hash(&bytes).to_hex().to_string());
    }
    if path.is_dir() {
        return directory_digest_blake3(path);
    }
    Err(RustSourceProviderError::MissingArtifact(format!(
        "{} is neither file nor directory",
        path.display()
    )))
}

fn validate_observed_provider_artifact_bytes(
    role: RustProviderRole,
    path: &Path,
    bytes: &[u8],
) -> Result<(), RustSourceProviderError> {
    if !matches!(role, RustProviderRole::Rustc | RustProviderRole::Cargo) {
        return Ok(());
    }
    if !looks_like_text_artifact(bytes) {
        return Ok(());
    }
    let scan_len = bytes.len().min(RUST_PROVIDER_ARTIFACT_MARKER_SCAN_BYTES);
    let text = String::from_utf8_lossy(&bytes[..scan_len]);
    if let Some(marker) = crate::source_toolchain_closure::disallowed_rust_provider_marker(&text) {
        return Err(RustSourceProviderError::Validate(format!(
            "provider artifact {} contains disallowed Rust provider marker '{marker}'",
            path.display()
        )));
    }
    Ok(())
}

fn looks_like_text_artifact(bytes: &[u8]) -> bool {
    if bytes.starts_with(b"#!") {
        return true;
    }
    let probe_len = bytes.len().min(RUST_PROVIDER_ARTIFACT_TEXT_PROBE_BYTES);
    let probe = &bytes[..probe_len];
    !probe.contains(&0) && std::str::from_utf8(probe).is_ok()
}

fn content_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    if path.is_file() {
        return file_digest_blake3(path);
    }
    if path.is_dir() {
        return directory_digest_blake3(path);
    }
    Err(RustSourceProviderError::MissingArtifact(format!(
        "{} is neither file nor directory",
        path.display()
    )))
}

fn file_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    let bytes =
        fs::read(path).map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn directory_digest_blake3(dir: &Path) -> Result<String, RustSourceProviderError> {
    let mut entries = Vec::new();
    collect_relative_paths(dir, dir, &mut entries)?;
    if entries.len() > DIRECTORY_DIGEST_MAX_ENTRIES {
        return Err(RustSourceProviderError::Digest(format!(
            "directory {} has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries",
            dir.display()
        )));
    }
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    for relative in &entries {
        let full = dir.join(relative);
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        if full.is_dir() {
            hasher.update(b"dir");
        } else if full.is_file() {
            hasher.update(b"file");
            let bytes = fs::read(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", full.display())))?;
            hasher.update(&bytes);
        } else if full.is_symlink() {
            hasher.update(b"symlink");
            let target = fs::read_link(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("readlink {}: {err}", full.display())))?;
            hasher.update(target.to_string_lossy().as_bytes());
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_relative_paths(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<(), RustSourceProviderError> {
    for entry in fs::read_dir(current)
        .map_err(|err| RustSourceProviderError::Digest(format!("read dir {}: {err}", current.display())))?
    {
        let entry = entry.map_err(|err| RustSourceProviderError::Digest(format!("read dir entry: {err}")))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|err| RustSourceProviderError::Digest(format!("strip prefix: {err}")))?
            .to_string_lossy()
            .to_string();
        out.push(relative);
        if path.is_dir() && !path.is_symlink() {
            collect_relative_paths(root, &path, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize test metadata: {err}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", parent.display())))?;
    fs::write(path, bytes).map_err(|err| RustSourceProviderError::Read(format!("write {}: {err}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA;
    use crate::source_toolchain_closure::RustProviderArtifact;
    use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
    use crate::source_toolchain_closure::RustProviderReceiptArtifact;
    use crate::source_toolchain_closure::RustProviderReceiptStep;
    use crate::source_toolchain_closure::RustProviderRole;
    use crate::source_toolchain_closure::RustProviderSourceIdentity;
    use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
    use crate::source_toolchain_closure::RustSourceProviderProvenance;
    use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
    use crate::source_toolchain_closure::ToolchainSourceKind;

    const HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
    const TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
    const SYNTHETIC_RLIB_BYTES: &[u8] = b"synthetic rlib\n";
    const FIRST_STAGE_TEST_SOURCE_COUNT: usize = 2;
    const RUSTC_STAGE1_TEST_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT: usize = 1;
    const RUSTC_FINAL_TEST_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_NEXT_ID: &str = "rust-1.92.0-stage1";
    const RUSTC_STAGE1_FINAL_CHAIN_ID: &str = "rust-1.93.1-stage1";
    const RUSTC_FINAL_STAGE_ID: &str = "rust-1.94.0-final";
    const SOURCE_ARCHIVE_FILE_MODE: u32 = 0o644;
    const SOURCE_ARCHIVE_EXECUTABLE_FILE_MODE: u32 = 0o755;
    const OBSERVED_RUST_190_SOURCE_TREE_ENTRIES: usize = 279_266;
    const SHA256_HEX_CHAR_COUNT: usize = 64;
    #[cfg(unix)]
    const EXECUTABLE_MODE: u32 = 0o755;

    #[test]
    fn musl_target_tool_candidates_include_source_root_aliases_only() {
        let candidates = musl_target_tool_program_candidates("gcc");
        let words = musl_target_tool_program_candidates_shell_words("gcc");
        let machine_aliases = musl_target_machine_aliases_shell_words();

        assert_eq!(candidates[0], "x86_64-unknown-linux-musl-gcc");
        assert!(candidates.iter().any(|candidate| candidate == "x86_64-linux-musl-gcc"));
        assert!(words.contains("x86_64-unknown-linux-musl-gcc"));
        assert!(words.contains("x86_64-linux-musl-gcc"));
        assert!(machine_aliases.contains("x86_64-unknown-linux-musl"));
        assert!(machine_aliases.contains("x86_64-linux-musl"));
        assert!(!candidates.iter().any(|candidate| candidate == "cc"));
        assert!(!words.contains(" ld "));
    }

    #[test]
    fn directory_digest_entry_limit_covers_observed_rust_source_tree() {
        assert!(DIRECTORY_DIGEST_MAX_ENTRIES > OBSERVED_RUST_190_SOURCE_TREE_ENTRIES);
        assert!(OBSERVED_RUST_190_SOURCE_TREE_ENTRIES > 0);
    }

    #[test]
    fn materializer_writes_final_provider_output_from_validated_candidate() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan(dir.path());

        let materialized = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();

        assert_eq!(materialized.output_path, output);
        assert_eq!(materialized.recipe_digest_blake3.len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(materialized.metadata_digest_blake3.len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(materialized.metadata_path, output.join(RUST_SOURCE_PROVIDER_METADATA_PATH));
        assert!(output.join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(output.join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(output.join(PROVIDER_RUSTDOC_RELATIVE_PATH).is_file());
        assert!(output.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
        assert!(output.join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH).is_file());
        assert!(scratch.join(FIRST_STAGE_SCRIPT_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PLAN_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_BUILD_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_BUILD_LOG_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR).join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
        assert!(
            scratch
                .join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR)
                .join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_OUTPUT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SCRIPT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_ARCHIVE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_SOURCE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(scratch.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR).join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
        assert!(
            scratch
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_OUTPUT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SOURCE_DIR).join("rust-1.91.1/README.txt").is_file());
        assert!(scratch.join(FIRST_STAGE_ARCHIVE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_BUILD_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/README.txt").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/bin/mrustc").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/bin/minicargo").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/output/rustc").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/output/cargo").is_file());
        assert!(
            scratch
                .join(FIRST_STAGE_SOURCE_DIR)
                .join("mrustc-0.12.0/run_rustc/output/prefix/bin/rustc")
                .is_file()
        );
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("rust-1.90.0/README.txt").is_file());
        #[cfg(unix)]
        assert_eq!(script_mode(&scratch.join(FIRST_STAGE_SCRIPT_FILE)), EXECUTABLE_MODE);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(manifest["schema"], FIRST_STAGE_PLAN_SCHEMA);
        assert_eq!(manifest["stage"]["id"], "mrustc-to-rust-1.90.0");
        assert_eq!(
            manifest["stage"]["expected_outputs"].as_array().unwrap().len(),
            REQUIRED_FIRST_STAGE_OUTPUT_ROLES.len()
        );
        let sources_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(sources_manifest["schema"], FIRST_STAGE_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(sources_manifest["source_count"], FIRST_STAGE_TEST_SOURCE_COUNT);
        let build_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_BUILD_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(build_manifest["schema"], FIRST_STAGE_BUILD_MANIFEST_SCHEMA);
        assert_eq!(build_manifest["build"]["mrustc_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(
            build_manifest["build"]["translated_rustc_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        assert_eq!(
            build_manifest["build"]["prefix_rustlib_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        assert_eq!(
            build_manifest["build"]["target_prefix_rustlib_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let candidate_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(candidate_manifest["schema"], FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(candidate_manifest["candidate_only"], true);
        let expected_candidate_artifacts =
            REQUIRED_FIRST_STAGE_OUTPUT_ROLES.len() + FIRST_STAGE_PROVIDER_RECEIPT_ARTIFACT_COUNT;
        assert_eq!(candidate_manifest["candidate"]["artifact_count"], expected_candidate_artifacts);
        assert_eq!(candidate_manifest["candidate"]["source_count"], FIRST_STAGE_TEST_SOURCE_COUNT);
        let candidate_validation =
            validate_materialized_rust_source_provider(&scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR)).unwrap();
        assert_eq!(candidate_validation.validation.artifact_count, expected_candidate_artifacts);
        assert_eq!(candidate_validation.validation.receipt_count, 1);
        assert_eq!(candidate_validation.validation.source_count, FIRST_STAGE_TEST_SOURCE_COUNT);
        let smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(smoke_summary["output_digest_blake3"], blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string());
        let rustc_stage1_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_plan["schema"], RUSTC_STAGE1_PLAN_SCHEMA);
        assert_eq!(rustc_stage1_plan["stage"]["id"], "rust-1.91.1-stage1");
        assert_eq!(rustc_stage1_plan["stage"]["bootstrap_stage_id"], "mrustc-to-rust-1.90.0");
        assert_eq!(
            rustc_stage1_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            candidate_validation.metadata_digest_blake3
        );
        let rustc_stage1_sources: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_sources["schema"], RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_sources["source_count"], RUSTC_STAGE1_TEST_SOURCE_COUNT);
        assert_eq!(rustc_stage1_sources["sources"][0]["id"], "rust-1.91.1");
        let rustc_stage1_build: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_build["schema"], RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_build["build"]["stage_id"], "rust-1.91.1-stage1");
        assert_eq!(rustc_stage1_build["build"]["rustc_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(
            rustc_stage1_build["build"]["target_rustlib_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let rustc_stage1_candidate_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_candidate_manifest["schema"], RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_stage1_candidate_manifest["candidate_only"], true);
        assert_eq!(rustc_stage1_candidate_manifest["final_output_written"], false);
        assert_eq!(rustc_stage1_candidate_manifest["candidate"]["stage_id"], "rust-1.91.1-stage1");
        let expected_rustc_stage1_candidate_artifacts =
            REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES.len() + RUSTC_STAGE1_PROVIDER_RECEIPT_ARTIFACT_COUNT;
        let expected_rustc_stage1_candidate_sources =
            RUSTC_STAGE1_TEST_SOURCE_COUNT + RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT;
        assert_eq!(
            rustc_stage1_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(
            rustc_stage1_candidate_manifest["candidate"]["source_count"],
            expected_rustc_stage1_candidate_sources
        );
        let rustc_stage1_candidate_validation =
            validate_materialized_rust_source_provider(&scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)).unwrap();
        assert_eq!(
            rustc_stage1_candidate_validation.validation.artifact_count,
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(rustc_stage1_candidate_validation.validation.receipt_count, 1);
        assert_eq!(rustc_stage1_candidate_validation.validation.source_count, expected_rustc_stage1_candidate_sources);
        assert!(
            rustc_stage1_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.91.1-stage1-bootstrap-provider")
        );
        let rustc_stage1_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_stage1_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_stage1_next_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_NEXT_ID);
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SCRIPT_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCE_DIR).join("rust-1.92.0/README.txt").is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUST_SOURCE_PROVIDER_METADATA_PATH)
                .is_file()
        );
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
                .join(SMOKE_EVIDENCE_SUMMARY_FILE)
                .is_file()
        );
        let rustc_stage1_next_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_next_plan["schema"], RUSTC_STAGE1_PLAN_SCHEMA);
        assert_eq!(rustc_stage1_next_plan["stage"]["id"], RUSTC_STAGE1_NEXT_ID);
        assert_eq!(rustc_stage1_next_plan["stage"]["bootstrap_stage_id"], "rust-1.91.1-stage1");
        assert_eq!(
            rustc_stage1_next_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            rustc_stage1_candidate_validation.metadata_digest_blake3
        );
        let rustc_stage1_next_sources: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_next_sources["schema"], RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_next_sources["source_count"], RUSTC_STAGE1_TEST_SOURCE_COUNT);
        assert_eq!(rustc_stage1_next_sources["sources"][0]["id"], "rust-1.92.0");
        let rustc_stage1_next_build: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_next_build["schema"], RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_next_build["build"]["stage_id"], RUSTC_STAGE1_NEXT_ID);
        assert_eq!(
            rustc_stage1_next_build["build"]["rustc_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let rustc_stage1_next_candidate_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_next_candidate_manifest["schema"], RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_stage1_next_candidate_manifest["candidate_only"], true);
        assert_eq!(rustc_stage1_next_candidate_manifest["final_output_written"], false);
        assert_eq!(
            rustc_stage1_next_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(
            rustc_stage1_next_candidate_manifest["candidate"]["source_count"],
            expected_rustc_stage1_candidate_sources
        );
        let rustc_stage1_next_candidate_validation = validate_materialized_rust_source_provider(
            &rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR),
        )
        .unwrap();
        assert_eq!(
            rustc_stage1_next_candidate_validation.validation.artifact_count,
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(rustc_stage1_next_candidate_validation.validation.receipt_count, 1);
        assert_eq!(
            rustc_stage1_next_candidate_validation.validation.source_count,
            expected_rustc_stage1_candidate_sources
        );
        assert!(
            rustc_stage1_next_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.92.0-stage1-bootstrap-provider")
        );
        let rustc_stage1_next_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(
                rustc_stage1_next_root
                    .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
                    .join(SMOKE_EVIDENCE_SUMMARY_FILE),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_next_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_stage1_next_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_stage1_final_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_SCRIPT_FILE).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_SOURCE_DIR).join("rust-1.93.1/README.txt").is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(
            rustc_stage1_final_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUST_SOURCE_PROVIDER_METADATA_PATH)
                .is_file()
        );
        assert!(
            rustc_stage1_final_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        let rustc_stage1_final_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_final_root.join(RUSTC_STAGE1_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_final_plan["schema"], RUSTC_STAGE1_PLAN_SCHEMA);
        assert_eq!(rustc_stage1_final_plan["stage"]["id"], RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert_eq!(rustc_stage1_final_plan["stage"]["bootstrap_stage_id"], RUSTC_STAGE1_NEXT_ID);
        assert_eq!(
            rustc_stage1_final_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            rustc_stage1_next_candidate_validation.metadata_digest_blake3
        );
        let rustc_stage1_final_sources: serde_json::Value = serde_json::from_slice(
            &fs::read(rustc_stage1_final_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_final_sources["schema"], RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_final_sources["source_count"], RUSTC_STAGE1_TEST_SOURCE_COUNT);
        assert_eq!(rustc_stage1_final_sources["sources"][0]["id"], "rust-1.93.1");
        let rustc_stage1_final_build: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_final_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_final_build["schema"], RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_final_build["build"]["stage_id"], RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert_eq!(
            rustc_stage1_final_build["build"]["rustc_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let rustc_stage1_final_candidate_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(rustc_stage1_final_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_final_candidate_manifest["schema"], RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_stage1_final_candidate_manifest["candidate_only"], true);
        assert_eq!(rustc_stage1_final_candidate_manifest["final_output_written"], false);
        assert_eq!(
            rustc_stage1_final_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(
            rustc_stage1_final_candidate_manifest["candidate"]["source_count"],
            expected_rustc_stage1_candidate_sources
        );
        let rustc_stage1_final_candidate_validation = validate_materialized_rust_source_provider(
            &rustc_stage1_final_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR),
        )
        .unwrap();
        assert_eq!(
            rustc_stage1_final_candidate_validation.validation.artifact_count,
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(rustc_stage1_final_candidate_validation.validation.receipt_count, 1);
        assert_eq!(
            rustc_stage1_final_candidate_validation.validation.source_count,
            expected_rustc_stage1_candidate_sources
        );
        assert!(
            rustc_stage1_final_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.93.1-stage1-bootstrap-provider")
        );
        let rustc_stage1_final_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(
                rustc_stage1_final_root
                    .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
                    .join(SMOKE_EVIDENCE_SUMMARY_FILE),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_final_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_stage1_final_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_final_root = scratch.join(RUSTC_FINAL_DIR);
        assert!(rustc_final_root.join(RUSTC_FINAL_PLAN_FILE).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_SOURCES_MANIFEST_FILE).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_BUILD_MANIFEST_FILE).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_BUILD_LOG_FILE).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_SCRIPT_FILE).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_SOURCE_DIR).join("rust-1.94.0/README.txt").is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_OUTPUT_DIR).join(PROVIDER_RUSTDOC_RELATIVE_PATH).is_file());
        assert!(rustc_final_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(
            rustc_final_root
                .join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR)
                .join(RUST_SOURCE_PROVIDER_METADATA_PATH)
                .is_file()
        );
        assert!(
            rustc_final_root
                .join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(
            rustc_final_root
                .join(RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_DIR)
                .join(SMOKE_EVIDENCE_SUMMARY_FILE)
                .is_file()
        );
        let rustc_final_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_final_root.join(RUSTC_FINAL_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_final_plan["schema"], RUSTC_FINAL_PLAN_SCHEMA);
        assert_eq!(rustc_final_plan["stage"]["id"], RUSTC_FINAL_STAGE_ID);
        assert_eq!(rustc_final_plan["stage"]["bootstrap_stage_id"], RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert_eq!(
            rustc_final_plan["stage"]["expected_outputs"].as_array().unwrap().len(),
            REQUIRED_RUSTC_FINAL_STAGE_OUTPUT_ROLES.len()
        );
        assert_eq!(
            rustc_final_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            rustc_stage1_final_candidate_validation.metadata_digest_blake3
        );
        let rustc_final_sources: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_final_root.join(RUSTC_FINAL_SOURCES_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_final_sources["schema"], RUSTC_FINAL_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_final_sources["source_count"], RUSTC_FINAL_TEST_SOURCE_COUNT);
        assert_eq!(rustc_final_sources["sources"][0]["id"], "rust-1.94.0");
        let rustc_final_build: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_final_root.join(RUSTC_FINAL_BUILD_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_final_build["schema"], RUSTC_FINAL_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_final_build["build"]["stage_id"], RUSTC_FINAL_STAGE_ID);
        assert_eq!(rustc_final_build["build"]["rustdoc_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        let rustc_final_candidate_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(rustc_final_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_final_candidate_manifest["schema"], RUSTC_FINAL_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_final_candidate_manifest["candidate_only"], false);
        assert_eq!(rustc_final_candidate_manifest["final_output_written"], true);
        assert_eq!(rustc_final_candidate_manifest["final_output_path"], output.display().to_string());
        assert!(rustc_final_candidate_manifest.get("next_blocked_reason").is_none());
        assert_eq!(rustc_final_candidate_manifest["candidate"]["stage_id"], RUSTC_FINAL_STAGE_ID);
        let expected_rustc_final_candidate_artifacts =
            REQUIRED_RUSTC_FINAL_BUILD_OUTPUT_ROLES.len() + RUSTC_FINAL_PROVIDER_RECEIPT_ARTIFACT_COUNT;
        let expected_rustc_final_candidate_sources =
            RUSTC_FINAL_TEST_SOURCE_COUNT + RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT;
        assert_eq!(
            rustc_final_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_final_candidate_artifacts
        );
        assert_eq!(rustc_final_candidate_manifest["candidate"]["source_count"], expected_rustc_final_candidate_sources);
        let rustc_final_candidate_validation =
            validate_materialized_rust_source_provider(&rustc_final_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR))
                .unwrap();
        let materialized_validation = validate_materialized_rust_source_provider(&output).unwrap();
        assert_eq!(materialized_validation.metadata_digest_blake3, materialized.metadata_digest_blake3);
        assert_eq!(
            materialized_validation.validation.policy_digest_blake3,
            rustc_final_candidate_validation.validation.policy_digest_blake3
        );
        assert_eq!(
            rustc_final_candidate_validation.validation.artifact_count,
            expected_rustc_final_candidate_artifacts
        );
        assert_eq!(rustc_final_candidate_validation.validation.receipt_count, 1);
        assert_eq!(rustc_final_candidate_validation.validation.source_count, expected_rustc_final_candidate_sources);
        assert!(
            rustc_final_candidate_validation
                .metadata
                .artifacts
                .iter()
                .any(|artifact| artifact.role == RustProviderRole::Rustdoc)
        );
        assert!(
            rustc_final_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.94.0-final-bootstrap-provider")
        );
        let rustc_final_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(
                rustc_final_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_final_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_final_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_final_log = fs::read_to_string(rustc_final_root.join(RUSTC_FINAL_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_final_log.contains(RUSTC_FINAL_STAGE_ID));
        assert!(rustc_final_log.contains("rustc final products ready"));
        let rustc_stage1_next_log =
            fs::read_to_string(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_next_log.contains(RUSTC_STAGE1_NEXT_ID));
        assert!(rustc_stage1_next_log.contains("rustc stage1 products ready"));
        let rustc_stage1_final_log =
            fs::read_to_string(rustc_stage1_final_root.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_final_log.contains(RUSTC_STAGE1_FINAL_CHAIN_ID));
        assert!(rustc_stage1_final_log.contains("rustc stage1 products ready"));
        let rustc_stage1_log = fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_log.contains("mantle rustc stage1"));
        assert!(rustc_stage1_log.contains("rustc stage1 products ready"));
        let script = fs::read_to_string(scratch.join(FIRST_STAGE_SCRIPT_FILE)).unwrap();
        assert!(script.contains("missing verified source"));
        assert!(script.contains("verified sources manifest"));
        assert!(script.contains("mrustc-0.12.0"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_BINARY));
        assert!(script.contains(FIRST_STAGE_PKG_CONFIG_PROGRAM));
        assert!(script.contains(FIRST_STAGE_CMAKE_PROGRAM));
        assert!(script.contains("GCC_FALLBACK_GLOB"));
        assert!(script.contains("CC_PROGRAM=\"$candidate_gcc\""));
        assert!(script.contains("CXX_PROGRAM=\"$candidate\""));
        assert!(script.contains("using GCC toolchain"));
        assert!(script.contains("ZLIB_PKG_CONFIG_FALLBACK_GLOB"));
        assert!(script.contains("using zlib pkg-config flags"));
        assert!(script.contains("export CFLAGS=\"$ZLIB_CFLAGS\""));
        assert!(script.contains("export CPPFLAGS=\"$ZLIB_CFLAGS\""));
        assert!(script.contains("scrubbing inherited Cargo/build-script environment"));
        assert!(script.contains("writing minicargo workspace boundary"));
        assert!(script.contains(&format!("members = [\"{FIRST_STAGE_PROC_MACRO_WORKSPACE_MEMBER}\"]")));
        assert!(script.contains(&format!("resolver = \"{FIRST_STAGE_MINICARGO_WORKSPACE_RESOLVER}\"")));
        assert!(script.contains(&format!("RUSTC_HOST_TRIPLE={}", shell_quote(HOST_TRIPLE))));
        assert!(script.contains(&format!("RUSTC_TARGET={}", shell_quote(HOST_TRIPLE))));
        assert!(script.contains(&format!("RUSTC_PROVIDER_TARGET_TRIPLE={}", shell_quote(TARGET_TRIPLE))));
        assert!(script.contains("using target linker wrapper"));
        assert!(script.contains("using compiler-host linker"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB));
        assert!(script.contains("TARGET_CC_PROGRAM_CANDIDATES='x86_64-unknown-linux-musl-gcc x86_64-linux-musl-gcc'"));
        assert!(script.contains("TARGET_MUSL_MACHINE_ALIASES='x86_64-unknown-linux-musl x86_64-linux-musl'"));
        assert!(script.contains("TARGET_MUSL_SOURCE_ROOT_SYSROOT='x86_64-linux-musl'"));
        assert!(script.contains("target_libc_root=\"$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT\""));
        assert!(script.contains("target_gcc_crt_machine=$target_cc_machine"));
        assert!(script.contains("target_cxx_program=$target_tool_prefix-g++"));
        assert!(script.contains("PATH=\"$target_alias_dir:$PATH\""));
        assert!(script.contains("export PATH"));
        assert!(script.contains("-static-pie) mapped_arg=\"-static\""));
        assert!(script.contains(FIRST_STAGE_TARGET_LINKER_ALIAS_DIR));
        assert!(script.contains(FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR));
        assert!(script.contains(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE));
        assert!(script.contains(FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE));
        assert!(script.contains(&format!("export {FIRST_STAGE_TARGET_NIX_CC_WRAPPER_HOST_ROLE_VAR}=1")));
        assert!(script.contains("rcrt1.o"));
        assert!(script.contains("crtbeginS.o"));
        assert!(script.contains("libgcc.a"));
        assert!(script.contains("libgcc_eh.a"));
        assert!(script.contains("libgcc_s.a"));
        assert!(script.contains("libunwind.a"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_SOURCE));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT));
        assert!(script.contains(FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE));
        assert!(script.contains(FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG));
        assert!(script.contains(FIRST_STAGE_TARGET_MRUSTC_CC_ENV_VAR));
        assert!(script.contains(FIRST_STAGE_TARGET_CARGO_CC_ENV_VAR));
        assert!(script.contains(FIRST_STAGE_TARGET_CARGO_CXX_ENV_VAR));
        assert!(script.contains(FIRST_STAGE_TARGET_CARGO_AR_ENV_VAR));
        assert!(script.contains(FIRST_STAGE_TARGET_CARGO_RANLIB_ENV_VAR));
        assert!(script.contains("CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=\"$target_alias_dir/cc\""));
        assert!(!script.contains("export CC=cc"));
        assert!(script.contains(&format!("RUN_RUSTC_DYLIB_EXT={FIRST_STAGE_STATIC_MUSL_DYLIB_EXT}")));
        assert!(script.contains("target_outdir_suffix"));
        assert!(script.contains("target_sysroot_source=\"rustc-${RUSTC_VERSION}-src/library/sysroot\""));
        assert!(script.contains("MRUSTC_PATH=\"$(pwd)/$target_bin_dir/rustc\""));
        assert!(script.contains("--target $RUSTC_TARGET"));
        assert!(script.contains(FIRST_STAGE_TARGET_PREFIX_S_DIR));
        assert!(script.contains("$COPY_PROGRAM output/rustc \"$target_bin_dir/rustc\""));
        assert!(script.contains("target rustlib build did not produce libstd.rlib"));
        assert!(script.contains("unset CARGO_PKG_VERSION"));
        assert!(script.contains("unset CARGO_MANIFEST_DIR"));
        assert!(script.contains("unset OUT_DIR"));
        assert!(script.contains("unset TARGET"));
        assert!(script.contains("unset HOST"));
        assert!(!script.contains("unset RUSTC_TARGET"));
        let scrub_index = script.find("unset CARGO_PKG_VERSION").unwrap();
        let workspace_index = script.find("writing minicargo workspace boundary").unwrap();
        let compiler_host_linker_index = script.find("using compiler-host linker").unwrap();
        let build_index = script.find(&format!("$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE}")).unwrap();
        let host_run_rustc_index = script
            .find(&format!("if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM -C {FIRST_STAGE_RUN_RUSTC_DIR}"))
            .unwrap();
        let target_assignment_index = script.rfind("RUSTC_TARGET=\"$RUSTC_PROVIDER_TARGET_TRIPLE\"").unwrap();
        let target_linker_index = script.rfind("using target linker wrapper").unwrap();
        let target_sysroot_index = script.rfind("target_sysroot_source").unwrap();
        assert!(scrub_index < build_index);
        assert!(workspace_index < build_index);
        assert!(build_index < compiler_host_linker_index);
        assert!(compiler_host_linker_index < host_run_rustc_index);
        assert!(host_run_rustc_index < target_assignment_index);
        assert!(target_assignment_index < target_linker_index);
        assert!(target_linker_index < target_sysroot_index);
    }

    #[test]
    fn materializer_writes_musl_host_provider_metadata_from_route_plan() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_musl_host_route_plan(dir.path());

        let materialized = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let validation = validate_materialized_rust_source_provider(&materialized.output_path).unwrap();

        assert_eq!(validation.metadata.host_triple, TARGET_TRIPLE);
        assert_eq!(validation.metadata.target_triple, TARGET_TRIPLE);
        assert!(validation.metadata.artifacts.iter().any(|artifact| {
            artifact.role == RustProviderRole::HostRustlib
                && artifact.path == provider_rustlib_relative_path(TARGET_TRIPLE)
        }));
        assert!(validation.metadata.artifacts.iter().any(|artifact| {
            artifact.role == RustProviderRole::TargetRustlib
                && artifact.path == provider_rustlib_relative_path(TARGET_TRIPLE)
        }));
        assert!(!validation.metadata.artifacts.iter().any(|artifact| {
            artifact.role == RustProviderRole::HostRustlib
                && artifact.path == provider_rustlib_relative_path(HOST_TRIPLE)
        }));
    }

    #[test]
    fn materializer_uses_explicit_route_plan_when_default_plan_is_absent() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let explicit_plan = dir.path().join("custom-rust-source-plan.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan(dir.path());
        fs::rename(dir.path().join(RUST_SOURCE_PROVIDER_PLAN_FILE), &explicit_plan).unwrap();

        let materialized =
            materialize_rust_source_provider_with_route_plan(&recipe, Some(&explicit_plan), &output, &scratch, false)
                .unwrap();

        assert_eq!(materialized.output_path, output);
        assert!(!dir.path().join(RUST_SOURCE_PROVIDER_PLAN_FILE).exists());
        assert!(materialized.metadata_path.is_file());
    }

    #[test]
    fn materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan_with_xpy_adapters(dir.path());

        let materialized = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();

        assert_eq!(materialized.output_path, output);
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
        assert!(
            !scratch
                .join(RUSTC_STAGE1_SOURCE_DIR)
                .join("rust-1.91.1")
                .join(RUSTC_STAGE1_SOURCE_BUILD_SCRIPT)
                .exists()
        );
        assert!(scratch.join(RUSTC_STAGE1_BUILD_DIR).join(RUSTC_STAGE1_GENERATED_BUILD_SCRIPT).is_file());
        let rustc_final_root = scratch.join(RUSTC_FINAL_DIR);
        assert!(
            !rustc_final_root
                .join(RUSTC_FINAL_SOURCE_DIR)
                .join("rust-1.94.0")
                .join(RUSTC_FINAL_SOURCE_BUILD_SCRIPT)
                .exists()
        );
        assert!(rustc_final_root.join(RUSTC_FINAL_BUILD_DIR).join(RUSTC_FINAL_GENERATED_BUILD_SCRIPT).is_file());
        let rustc_stage1_log = fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_log.contains("scrubbing inherited Rust bootstrap Cargo environment"));
        assert!(rustc_stage1_log.contains("isolated Rust Cargo workspace"));
        assert!(rustc_stage1_log.contains("using generated x.py Rust build adapter"));
        assert!(rustc_stage1_log.contains("synthetic x.py for 1.91.1"));
        let generated_stage1_script =
            fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_DIR).join(RUSTC_STAGE1_GENERATED_BUILD_SCRIPT)).unwrap();
        assert!(generated_stage1_script.contains("change-id = \"ignore\""));
        assert!(generated_stage1_script.contains("ninja = false"));
        assert!(generated_stage1_script.contains("sysconfdir = \"etc\""));
        assert!(generated_stage1_script.contains("target = [\"$MANTLE_HOST_TRIPLE\", \"$MANTLE_TARGET_TRIPLE\"]"));
        assert!(generated_stage1_script.contains(FIRST_STAGE_MAKE_FALLBACK_GLOB));
        assert!(generated_stage1_script.contains(FIRST_STAGE_CMAKE_FALLBACK_GLOB));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB));
        assert!(
            generated_stage1_script
                .contains("for candidate_name in x86_64-unknown-linux-musl-gcc x86_64-linux-musl-gcc")
        );
        assert!(
            generated_stage1_script
                .contains("MANTLE_TARGET_MUSL_MACHINE_ALIASES='x86_64-unknown-linux-musl x86_64-linux-musl'")
        );
        assert!(generated_stage1_script.contains("MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT='x86_64-linux-musl'"));
        assert!(
            generated_stage1_script.contains("$target_wrapper_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.a")
        );
        assert!(generated_stage1_script.contains("$MANTLE_TARGET_MUSL_ROOT/lib/crt1.o"));
        assert!(generated_stage1_script.contains("[target.$MANTLE_TARGET_TRIPLE]"));
        assert!(generated_stage1_script.contains("cc = \"$MANTLE_TARGET_CC\""));
        assert!(generated_stage1_script.contains("linker = \"$MANTLE_TARGET_CC\""));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE));
        assert!(generated_stage1_script.contains("musl-root = \"$MANTLE_TARGET_MUSL_ROOT\""));
        assert!(generated_stage1_script.contains(RUSTC_SOURCE_CRANELIFT_MANIFEST));
        assert!(generated_stage1_script.contains(RUSTC_SOURCE_CODEGEN_GCC_MANIFEST));
        let rustc_final_log = fs::read_to_string(rustc_final_root.join(RUSTC_FINAL_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_final_log.contains("scrubbing inherited Rust bootstrap Cargo environment"));
        assert!(rustc_final_log.contains("isolated Rust Cargo workspace"));
        assert!(rustc_final_log.contains("using generated x.py Rust build adapter"));
        assert!(rustc_final_log.contains("synthetic x.py for 1.94.0"));
        assert!(rustc_final_log.contains("install rustc cargo library/std"));
        assert!(!rustc_final_log.contains("install rustc cargo rustdoc"));
    }

    #[test]
    fn materializer_rejects_rust_source_without_stage_script_or_xpy_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan_missing_rust191_adapter(dir.path());

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("rustc stage1 source has no build script or x.py"));
        assert!(!output.exists());
        assert!(scratch.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        let rustc_stage1_log = fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_log.contains("rustc stage1 source has no build script or x.py"));
        assert!(!scratch.join(RUSTC_STAGE1_BUILD_DIR).join(RUSTC_STAGE1_GENERATED_BUILD_SCRIPT).exists());
    }

    #[test]
    fn materializer_rejects_existing_output_before_scratch_work() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        fs::create_dir(&output).unwrap();
        write_test_route_plan(dir.path());

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        assert!(err.to_string().contains("already exists"));
        assert!(!scratch.join(FIRST_STAGE_SCRIPT_FILE).exists());
        assert!(output.is_dir());
    }

    #[test]
    fn first_stage_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let candidate = scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_RUSTC_RELATIVE_PATH), b"changed-rustc").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn rustc_stage1_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let candidate = scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_CARGO_RELATIVE_PATH), b"changed-cargo").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn chained_rustc_stage1_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let candidate = scratch
            .join(RUSTC_STAGE1_CHAIN_DIR)
            .join(RUSTC_STAGE1_NEXT_ID)
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_RUSTC_RELATIVE_PATH), b"changed-chained-rustc").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn final_chained_rustc_stage1_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let candidate = scratch
            .join(RUSTC_STAGE1_CHAIN_DIR)
            .join(RUSTC_STAGE1_FINAL_CHAIN_ID)
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_CARGO_RELATIVE_PATH), b"changed-final-chain-cargo").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn rustc_final_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        let candidate = scratch.join(RUSTC_FINAL_DIR).join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_RUSTDOC_RELATIVE_PATH), b"changed-final-rustdoc").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn materializer_rejects_missing_route_plan_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        assert!(err.to_string().contains(RUST_SOURCE_PROVIDER_PLAN_FILE));
        assert!(!output.exists());
        assert!(!scratch.join(FIRST_STAGE_SCRIPT_FILE).exists());
    }

    #[test]
    fn materializer_rejects_first_stage_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_mrustc_sha(dir.path(), &sample_sha256_hex('d'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("mrustc-0.12.0"));
        assert!(!output.exists());
        assert!(!scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_rustc_stage1_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust191_sha(dir.path(), &sample_sha256_hex('e'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.91.1"));
        assert!(!output.exists());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(!scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).exists());
        assert!(!scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust192_sha(dir.path(), &sample_sha256_hex('f'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.92.0"));
        assert!(!output.exists());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        let rustc_stage1_next_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_NEXT_ID);
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(!rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).exists());
        assert!(!rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust193_sha(dir.path(), &sample_sha256_hex('a'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.93.1"));
        assert!(!output.exists());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        let rustc_stage1_next_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_NEXT_ID);
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        let rustc_stage1_final_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(!rustc_stage1_final_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).exists());
        assert!(!rustc_stage1_final_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_rustc_final_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust194_sha(dir.path(), &sample_sha256_hex('b'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.94.0"));
        assert!(!output.exists());
        let rustc_stage1_final_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_FINAL_CHAIN_ID);
        assert!(rustc_stage1_final_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        let rustc_final_root = scratch.join(RUSTC_FINAL_DIR);
        assert!(rustc_final_root.join(RUSTC_FINAL_PLAN_FILE).is_file());
        assert!(!rustc_final_root.join(RUSTC_FINAL_SOURCES_MANIFEST_FILE).exists());
        assert!(!rustc_final_root.join(RUSTC_FINAL_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn first_stage_script_fails_when_sources_removed_after_materialization() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap();
        fs::remove_dir_all(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0")).unwrap();

        let status = Command::new(scratch.join(FIRST_STAGE_SCRIPT_FILE)).status().unwrap();

        assert_eq!(status.code(), Some(FIRST_STAGE_MISSING_SOURCE_EXIT_CODE));
        assert!(validate_materialized_rust_source_provider(&output).is_ok());
    }

    #[test]
    fn directory_validator_accepts_complete_fake_provider() {
        let dir = tempfile::tempdir().unwrap();
        let metadata = write_fake_provider(dir.path());

        let validation = validate_materialized_rust_source_provider(dir.path()).unwrap();

        assert_eq!(validation.metadata_path, dir.path().join(RUST_SOURCE_PROVIDER_METADATA_PATH));
        assert_eq!(validation.validation.artifact_count, metadata.artifacts.len());
        assert_eq!(validation.validation.source_count, metadata.sources.len());
        assert_eq!(validation.validation.receipt_count, metadata.build_receipts.len());
        assert!(!validation.metadata_digest_blake3.is_empty());
    }

    #[test]
    fn directory_validator_rejects_artifact_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        write_fake_provider(dir.path());
        fs::write(dir.path().join("bin/rustc"), b"changed").unwrap();

        let err = validate_materialized_rust_source_provider(dir.path()).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
    }

    #[test]
    fn directory_validator_rejects_nix_rustc_wrapper_even_with_matching_digest() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper = b"#!/bin/sh\nexec /nix/store/example-rust-default/bin/rustc \"$@\"\n";
        write_fake_provider(dir.path());
        write_bytes(dir.path(), "bin/rustc", wrapper);
        refresh_fake_receipt_and_metadata(dir.path());

        let err = validate_materialized_rust_source_provider(dir.path()).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("disallowed Rust provider marker"));
        assert!(message.contains("'nix'"));
    }

    #[test]
    fn import_provider_copies_validated_provider() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let metadata = write_fake_provider(&import_dir);

        let imported = import_rust_source_provider(&import_dir, &output_dir).unwrap();

        assert_eq!(imported.input_path, import_dir);
        assert_eq!(imported.output_path, output_dir);
        assert_eq!(imported.validation.validation.artifact_count, metadata.artifacts.len());
        assert!(imported.output_path.join("bin/rustc").is_file());
        assert!(imported.output_path.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
    }

    #[test]
    fn import_provider_rejects_prebuilt_metadata_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let mut metadata = write_fake_provider(&import_dir);
        metadata.provenance.uses_prebuilt_rust = true;
        write_json(&import_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("uses prebuilt Rust"));
        assert!(!output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_existing_output_without_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();
        write_fake_provider(&import_dir);

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("already exists"));
        assert!(output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_malformed_receipt_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        write_fake_provider(&import_dir);
        fs::write(import_dir.join("share/mantle-rust-provider/receipts/build.json"), b"not-json").unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("provider receipt"));
        assert!(!output_dir.exists());
    }

    #[cfg(unix)]
    #[test]
    fn first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let output_path = dir.path().join("smoke.rlib");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        write_bytes(
            &provider_dir,
            PROVIDER_RUSTC_RELATIVE_PATH,
            b"#!/bin/sh\nd=$(dirname $0)\nLD_LIBRARY_PATH=/old/build/prefix/lib $d/rustc_binary \"$@\"\n",
        );
        write_bytes(&provider_dir, PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH, synthetic_rustc_script(None).as_bytes());
        make_executable(&provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH));
        make_executable(&provider_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH));

        normalize_first_stage_provider_wrappers(&provider_dir, HOST_TRIPLE).unwrap();
        let wrapper = fs::read_to_string(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).unwrap();
        let output = Command::new(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH))
            .env_clear()
            .arg("-o")
            .arg(&output_path)
            .output()
            .unwrap();

        assert!(!wrapper.contains(MRUSTC_RUSTC_WRAPPER_DIRNAME_MARKER));
        assert!(wrapper.contains("self_dir=${self%/*}"));
        assert!(wrapper.contains("LD_LIBRARY_PATH=\"$ld_path\""));
        assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(fs::read(&output_path).unwrap(), SYNTHETIC_RLIB_BYTES);
    }

    #[cfg(unix)]
    #[test]
    fn first_stage_wrapper_normalization_rejects_missing_rustc_binary() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        write_bytes(
            &provider_dir,
            PROVIDER_RUSTC_RELATIVE_PATH,
            b"#!/bin/sh\nd=$(dirname $0)\nLD_LIBRARY_PATH=/old/build/prefix/lib $d/rustc_binary \"$@\"\n",
        );

        let err = normalize_first_stage_provider_wrappers(&provider_dir, HOST_TRIPLE).unwrap_err();
        let wrapper = fs::read_to_string(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).unwrap();

        assert!(err.to_string().contains("references missing"));
        assert!(wrapper.contains(MRUSTC_RUSTC_WRAPPER_DIRNAME_MARKER));
        assert!(!provider_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH).exists());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_provider_runs_synthetic_rustc_after_validation() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);

        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();

        assert_eq!(smoke.provider_path, provider_dir);
        assert_eq!(smoke.scratch_dir, scratch_dir);
        assert_eq!(smoke.target_triple, HOST_TRIPLE);
        assert!(smoke.rustc_path.ends_with("bin/rustc"));
        assert!(smoke.source_path.ends_with(SMOKE_SOURCE_FILE));
        assert!(smoke.output_path.ends_with(SMOKE_OUTPUT_FILE));
        assert_eq!(smoke.output_digest_blake3, blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string());
        assert!(smoke.stdout.contains("synthetic rustc smoke"));
        assert!(smoke.stderr.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_evidence_persists_summary_output_and_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let evidence_dir = dir.path().join("evidence");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);
        let validation = validate_materialized_rust_source_provider(&provider_dir).unwrap();
        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();

        let evidence = persist_rust_source_provider_smoke_evidence(&evidence_dir, &smoke, &validation).unwrap();
        let summary: serde_json::Value = serde_json::from_slice(&fs::read(&evidence.summary_path).unwrap()).unwrap();

        assert_eq!(evidence.evidence_dir, evidence_dir);
        assert_eq!(evidence.output_digest_blake3, smoke.output_digest_blake3);
        assert_eq!(evidence.metadata_digest_blake3, validation.metadata_digest_blake3);
        assert_eq!(summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(summary["output_digest_blake3"], smoke.output_digest_blake3);
        assert_eq!(fs::read_to_string(&evidence.stdout_path).unwrap(), smoke.stdout);
        assert_eq!(fs::read_to_string(&evidence.stderr_path).unwrap(), smoke.stderr);
        assert_eq!(fs::read(&evidence.output_path).unwrap(), SYNTHETIC_RLIB_BYTES);
        assert_eq!(content_digest_blake3(&evidence.metadata_path).unwrap(), validation.metadata_digest_blake3);
    }

    #[cfg(unix)]
    #[test]
    fn smoke_evidence_rejects_tampered_smoke_output() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let evidence_dir = dir.path().join("evidence");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);
        let validation = validate_materialized_rust_source_provider(&provider_dir).unwrap();
        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();
        fs::write(&smoke.output_path, b"tampered").unwrap();

        let err = persist_rust_source_provider_smoke_evidence(&evidence_dir, &smoke, &validation).unwrap_err();

        assert!(err.to_string().contains("copied smoke output digest mismatch"));
        assert!(!evidence_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_provider_rejects_invalid_metadata_before_launch() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let sentinel_path = dir.path().join("launched");
        fs::create_dir(&provider_dir).unwrap();
        let mut metadata = write_synthetic_smoke_provider(&provider_dir, Some(&sentinel_path));
        metadata.provenance.uses_prebuilt_rust = true;
        write_json(&provider_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();

        let err = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap_err();

        assert!(err.to_string().contains("uses prebuilt Rust"));
        assert!(!sentinel_path.exists());
        assert!(!scratch_dir.join(SMOKE_OUTPUT_FILE).exists());
    }

    fn write_test_route_plan(root: &Path) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_sources(root, &sources, &sources.mrustc_sha256_hex);
    }

    fn write_test_musl_host_route_plan(root: &Path) {
        write_test_route_plan(root);
        let route_path = root.join(RUST_SOURCE_PROVIDER_PLAN_FILE);
        let gnu_host_rustlib = provider_rustlib_relative_path(HOST_TRIPLE);
        let musl_host_rustlib = provider_rustlib_relative_path(TARGET_TRIPLE);
        let text = fs::read_to_string(&route_path)
            .unwrap()
            .replace("host_triple = \"x86_64-unknown-linux-gnu\"", "host_triple = \"x86_64-unknown-linux-musl\"")
            .replace(
                &format!("{{ role = \"host-rustlib\", path = \"{gnu_host_rustlib}\" }}"),
                &format!("{{ role = \"host-rustlib\", path = \"{musl_host_rustlib}\" }}"),
            );
        fs::write(route_path, text).unwrap();
    }

    fn write_test_route_plan_with_xpy_adapters(root: &Path) {
        let sources = write_test_source_archives_with_xpy_adapters(root);
        write_test_route_plan_with_sources(root, &sources, &sources.mrustc_sha256_hex);
    }

    fn write_test_route_plan_missing_rust191_adapter(root: &Path) {
        let mut sources = write_test_source_archives(root);
        let rust191_path = root.join("test-source-archives").join("rust-1.91.1.tar.gz");
        write_minimal_test_source_archive(&rust191_path, "rust-1.91.1", b"rust 1.91.1 source without adapter\n");
        sources.rust191_sha256_hex = sha256_file_hex(&rust191_path);
        write_test_route_plan_with_sources(root, &sources, &sources.mrustc_sha256_hex);
    }

    fn write_test_route_plan_with_mrustc_sha(root: &Path, mrustc_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
            &sources.rust193_sha256_hex,
            &sources.rust194_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust191_sha(root: &Path, rust191_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            rust191_sha256_hex,
            &sources.rust192_sha256_hex,
            &sources.rust193_sha256_hex,
            &sources.rust194_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust192_sha(root: &Path, rust192_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            rust192_sha256_hex,
            &sources.rust193_sha256_hex,
            &sources.rust194_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust193_sha(root: &Path, rust193_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
            rust193_sha256_hex,
            &sources.rust194_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust194_sha(root: &Path, rust194_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
            &sources.rust193_sha256_hex,
            rust194_sha256_hex,
        );
    }

    fn write_test_route_plan_with_sources(root: &Path, sources: &TestSourceArchives, mrustc_sha256_hex: &str) {
        write_test_route_plan_with_source_shas(
            root,
            sources,
            mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
            &sources.rust193_sha256_hex,
            &sources.rust194_sha256_hex,
        )
    }

    fn write_test_route_plan_with_source_shas(
        root: &Path,
        sources: &TestSourceArchives,
        mrustc_sha256_hex: &str,
        rust191_sha256_hex: &str,
        rust192_sha256_hex: &str,
        rust193_sha256_hex: &str,
        rust194_sha256_hex: &str,
    ) {
        let text = r#"
let Source = {
  id | String,
  kind | String,
  name | String,
  version | String,
  url | String,
  sha256_hex | String,
} in
let Stage = {
  id | String,
  kind | String,
  source_ids | Array String,
  bootstrap_stage_id | String,
  rust_version | String,
  outputs | Array String,
  notes | Array String,
} in
let Output = {
  role | String,
  path | String,
} in
let Policy = {
  source_built | Bool,
  uses_prebuilt_rust | Bool,
  forbids_prebuilt_rust | Bool,
  reference | String,
} in
let Plan = {
  schema | String,
  provider_id | String,
  route | String,
  host_triple | String,
  target_triple | String,
  final_version | String,
  policy | Policy,
  sources | Array Source,
  stages | Array Stage,
  final_outputs | Array Output,
} in
{
  schema = "mantle-rust-source-provider-bootstrap-plan-v1",
  provider_id = "mantle-rust-source-provider",
  route = "mrustc-source-route",
  host_triple = "x86_64-unknown-linux-gnu",
  target_triple = "x86_64-unknown-linux-musl",
  final_version = "1.94.0",
  policy = {
    source_built = true,
    uses_prebuilt_rust = false,
    forbids_prebuilt_rust = true,
    reference = "stagex route",
  },
  sources = [
    {
      id = "mrustc-0.12.0",
      kind = "tarball",
      name = "mrustc-source",
      version = "0.12.0",
      url = "__MRUSTC_URL__",
      sha256_hex = "__MRUSTC_SHA256__",
    },
    {
      id = "rust-1.90.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.90.0",
      url = "__RUST190_URL__",
      sha256_hex = "__RUST190_SHA256__",
    },
    {
      id = "rust-1.91.1",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.91.1",
      url = "__RUST191_URL__",
      sha256_hex = "__RUST191_SHA256__",
    },
    {
      id = "rust-1.92.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.92.0",
      url = "__RUST192_URL__",
      sha256_hex = "__RUST192_SHA256__",
    },
    {
      id = "rust-1.93.1",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.93.1",
      url = "__RUST193_URL__",
      sha256_hex = "__RUST193_SHA256__",
    },
    {
      id = "rust-1.94.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.94.0",
      url = "__RUST194_URL__",
      sha256_hex = "__RUST194_SHA256__",
    },
  ],
  stages = [
    {
      id = "mrustc-to-rust-1.90.0",
      kind = "mrustc-seed",
      source_ids = ["mrustc-0.12.0", "rust-1.90.0"],
      bootstrap_stage_id = "",
      rust_version = "1.90.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["compile source stage"],
    },
    {
      id = "rust-1.91.1-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.91.1"],
      bootstrap_stage_id = "mrustc-to-rust-1.90.0",
      rust_version = "1.91.1",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["advance one Rust source release after mrustc"],
    },
    {
      id = "rust-1.92.0-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.92.0"],
      bootstrap_stage_id = "rust-1.91.1-stage1",
      rust_version = "1.92.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["advance one chained Rust source release"],
    },
    {
      id = "rust-1.93.1-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.93.1"],
      bootstrap_stage_id = "rust-1.92.0-stage1",
      rust_version = "1.93.1",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["advance final chained Rust source release before final provider"],
    },
    {
      id = "rust-1.94.0-final",
      kind = "rustc-final",
      source_ids = ["rust-1.94.0"],
      bootstrap_stage_id = "rust-1.93.1-stage1",
      rust_version = "1.94.0",
      outputs = ["rustc", "cargo", "rustdoc", "host-rustlib", "target-rustlib", "provider-receipt"],
      notes = ["install provider metadata and receipts"],
    },
  ],
  final_outputs = [
    { role = "rustc", path = "bin/rustc" },
    { role = "cargo", path = "bin/cargo" },
    { role = "rustdoc", path = "bin/rustdoc" },
    { role = "host-rustlib", path = "lib/rustlib/x86_64-unknown-linux-gnu/lib" },
    { role = "target-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "provider-receipt", path = "share/mantle-rust-provider/receipts/build.json" },
  ],
} | Plan
"#;
        let text = text
            .replace("__MRUSTC_URL__", &sources.mrustc_url)
            .replace("__MRUSTC_SHA256__", mrustc_sha256_hex)
            .replace("__RUST190_URL__", &sources.rust190_url)
            .replace("__RUST190_SHA256__", &sources.rust190_sha256_hex)
            .replace("__RUST191_URL__", &sources.rust191_url)
            .replace("__RUST191_SHA256__", rust191_sha256_hex)
            .replace("__RUST192_URL__", &sources.rust192_url)
            .replace("__RUST192_SHA256__", rust192_sha256_hex)
            .replace("__RUST193_URL__", &sources.rust193_url)
            .replace("__RUST193_SHA256__", rust193_sha256_hex)
            .replace("__RUST194_URL__", &sources.rust194_url)
            .replace("__RUST194_SHA256__", rust194_sha256_hex);
        fs::write(root.join(RUST_SOURCE_PROVIDER_PLAN_FILE), text).unwrap();
    }

    struct TestSourceArchives {
        mrustc_url: String,
        mrustc_sha256_hex: String,
        rust190_url: String,
        rust190_sha256_hex: String,
        rust191_url: String,
        rust191_sha256_hex: String,
        rust192_url: String,
        rust192_sha256_hex: String,
        rust193_url: String,
        rust193_sha256_hex: String,
        rust194_url: String,
        rust194_sha256_hex: String,
    }

    fn write_test_source_archives(root: &Path) -> TestSourceArchives {
        write_test_source_archives_with_adapter(root, false)
    }

    fn write_test_source_archives_with_xpy_adapters(root: &Path) -> TestSourceArchives {
        write_test_source_archives_with_adapter(root, true)
    }

    fn write_test_source_archives_with_adapter(root: &Path, use_xpy_adapters: bool) -> TestSourceArchives {
        let archive_dir = root.join("test-source-archives");
        fs::create_dir_all(&archive_dir).unwrap();
        let mrustc_path = archive_dir.join("mrustc-0.12.0.tar.gz");
        let rust190_path = archive_dir.join("rust-1.90.0.tar.gz");
        let rust191_path = archive_dir.join("rust-1.91.1.tar.gz");
        let rust192_path = archive_dir.join("rust-1.92.0.tar.gz");
        let rust193_path = archive_dir.join("rust-1.93.1.tar.gz");
        let rust194_path = archive_dir.join("rust-1.94.0.tar.gz");
        write_test_source_archive(&mrustc_path, "mrustc-0.12.0", b"mrustc seed source\n", use_xpy_adapters);
        write_test_source_archive(&rust190_path, "rust-1.90.0", b"rust 1.90 source\n", use_xpy_adapters);
        write_test_source_archive(&rust191_path, "rust-1.91.1", b"rust 1.91.1 source\n", use_xpy_adapters);
        write_test_source_archive(&rust192_path, "rust-1.92.0", b"rust 1.92.0 source\n", use_xpy_adapters);
        write_test_source_archive(&rust193_path, "rust-1.93.1", b"rust 1.93.1 source\n", use_xpy_adapters);
        write_test_source_archive(&rust194_path, "rust-1.94.0", b"rust 1.94 source\n", use_xpy_adapters);
        TestSourceArchives {
            mrustc_url: url::Url::from_file_path(&mrustc_path).unwrap().to_string(),
            mrustc_sha256_hex: sha256_file_hex(&mrustc_path),
            rust190_url: url::Url::from_file_path(&rust190_path).unwrap().to_string(),
            rust190_sha256_hex: sha256_file_hex(&rust190_path),
            rust191_url: url::Url::from_file_path(&rust191_path).unwrap().to_string(),
            rust191_sha256_hex: sha256_file_hex(&rust191_path),
            rust192_url: url::Url::from_file_path(&rust192_path).unwrap().to_string(),
            rust192_sha256_hex: sha256_file_hex(&rust192_path),
            rust193_url: url::Url::from_file_path(&rust193_path).unwrap().to_string(),
            rust193_sha256_hex: sha256_file_hex(&rust193_path),
            rust194_url: url::Url::from_file_path(&rust194_path).unwrap().to_string(),
            rust194_sha256_hex: sha256_file_hex(&rust194_path),
        }
    }

    fn write_minimal_test_source_archive(path: &Path, top_dir: &str, readme: &[u8]) {
        let file = File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        append_test_tar_file(&mut builder, &format!("{top_dir}/README.txt"), readme);
        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
    }

    fn write_test_source_archive(path: &Path, top_dir: &str, readme: &[u8], use_xpy_adapters: bool) {
        let file = File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        append_test_tar_file(&mut builder, &format!("{top_dir}/README.txt"), readme);
        if use_xpy_adapters && matches!(top_dir, "rust-1.91.1" | "rust-1.92.0" | "rust-1.93.1" | "rust-1.94.0") {
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{RUSTC_SOURCE_BOOTSTRAP_MANIFEST}"),
                b"[package]\nname = \"bootstrap\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            );
        }
        if top_dir.starts_with(FIRST_STAGE_MRUSTC_SOURCE_PREFIX) {
            append_test_tar_file(&mut builder, &format!("{top_dir}/{FIRST_STAGE_MAKEFILE}"), test_mrustc_makefile());
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_MINICARGO_MAKEFILE}"),
                test_mrustc_minicargo_makefile(),
            );
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_RUN_RUSTC_DIR}/{FIRST_STAGE_MAKEFILE}"),
                test_mrustc_run_rustc_makefile(),
            );
        }
        if matches!(top_dir, "rust-1.91.1" | "rust-1.92.0" | "rust-1.93.1") {
            if use_xpy_adapters {
                append_test_tar_file_with_mode(
                    &mut builder,
                    &format!("{top_dir}/{RUSTC_SOURCE_XPY_SCRIPT}"),
                    test_rustc_xpy_script(),
                    SOURCE_ARCHIVE_EXECUTABLE_FILE_MODE,
                );
            } else {
                append_test_tar_file(
                    &mut builder,
                    &format!("{top_dir}/{RUSTC_STAGE1_SOURCE_BUILD_SCRIPT}"),
                    test_rustc_stage1_build_script(),
                );
            }
        }
        if top_dir == "rust-1.94.0" {
            if use_xpy_adapters {
                append_test_tar_file_with_mode(
                    &mut builder,
                    &format!("{top_dir}/{RUSTC_SOURCE_XPY_SCRIPT}"),
                    test_rustc_xpy_script(),
                    SOURCE_ARCHIVE_EXECUTABLE_FILE_MODE,
                );
            } else {
                append_test_tar_file(
                    &mut builder,
                    &format!("{top_dir}/{RUSTC_FINAL_SOURCE_BUILD_SCRIPT}"),
                    test_rustc_final_build_script(),
                );
            }
        }
        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
    }

    fn append_test_tar_file(builder: &mut tar::Builder<flate2::write::GzEncoder<File>>, path: &str, bytes: &[u8]) {
        append_test_tar_file_with_mode(builder, path, bytes, SOURCE_ARCHIVE_FILE_MODE);
    }

    fn append_test_tar_file_with_mode(
        builder: &mut tar::Builder<flate2::write::GzEncoder<File>>,
        path: &str,
        bytes: &[u8],
        mode: u32,
    ) {
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(bytes.len()).unwrap());
        header.set_mode(mode);
        header.set_cksum();
        builder.append_data(&mut header, path, bytes).unwrap();
    }

    fn test_mrustc_makefile() -> &'static [u8] {
        b"all:\n\tmkdir -p bin\n\tprintf 'synthetic mrustc\\n' > bin/mrustc\n\tchmod +x bin/mrustc\n"
    }

    fn test_mrustc_minicargo_makefile() -> &'static [u8] {
        b"bin/minicargo:\n\tmkdir -p bin\n\tprintf '%s\\n' '#!/bin/sh' 'set -eu' 'out=' 'prev=' 'for arg in \"$$@\"; do' '  if [ \"$$prev\" = \"--output-dir\" ]; then out=\"$$arg\"; prev=\"\"; continue; fi' '  if [ \"$$arg\" = \"--output-dir\" ]; then prev=\"--output-dir\"; continue; fi' 'done' 'if [ -z \"$$out\" ]; then echo missing-output-dir >&2; exit 2; fi' 'mkdir -p \"$$out\"' 'printf \"synthetic target std\\\\n\" > \"$$out/libstd.rlib\"' > bin/minicargo\n\tchmod +x bin/minicargo\noutput/rustc:\n\tmkdir -p $(@D)\n\tprintf 'synthetic translated rustc\\n' > $@\n\tchmod +x $@\noutput-%/rustc:\n\tmkdir -p $(@D)\n\tprintf 'synthetic translated rustc\\n' > $@\n\tchmod +x $@\noutput/cargo:\n\tmkdir -p $(@D)\n\tprintf 'synthetic translated cargo\\n' > $@\n\tchmod +x $@\noutput-%/cargo:\n\tmkdir -p $(@D)\n\tprintf 'synthetic translated cargo\\n' > $@\n\tchmod +x $@\n"
    }

    fn test_mrustc_run_rustc_makefile() -> &'static [u8] {
        b"all:\n\tmkdir -p output/prefix/bin output/prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib\n\tprintf '%s\\n' '#!/bin/sh' 'd=$$(dirname $$0)' 'LD_LIBRARY_PATH=/old/build/prefix/lib $$d/rustc_binary \"$$@\"' > output/prefix/bin/rustc\n\tprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$$@\"; do' '  if [ \"$$prev\" = \"-o\" ]; then out=\"$$arg\"; prev=\"\"; continue; fi' '  if [ \"$$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$$out\"' 'echo synthetic rustc smoke' > output/prefix/bin/rustc_binary\n\tprintf 'synthetic prefix cargo\\n' > output/prefix/bin/cargo\n\tprintf 'synthetic prefix host std\\n' > output/prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib\n\tprintf 'synthetic prefix musl host std\\n' > output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib\n\tchmod +x output/prefix/bin/rustc output/prefix/bin/rustc_binary output/prefix/bin/cargo\n\noutput%/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib:\n\tmkdir -p $(@D)\n\tprintf 'synthetic target std\\n' > $@\n"
    }

    fn test_rustc_stage1_build_script() -> &'static [u8] {
        b"set -eu\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\nmkdir -p \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic stage1 rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic stage1 cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic stage1 host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic stage1 target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\n"
    }

    fn test_rustc_final_build_script() -> &'static [u8] {
        b"set -eu\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\nmkdir -p \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic final rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic final cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic final rustdoc for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\nprintf 'synthetic final host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic final target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\" \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\n"
    }

    fn test_rustc_xpy_script() -> &'static [u8] {
        b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"synthetic x.py for $MANTLE_RUST_VERSION $*\"\nmkdir -p \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic xpy rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic xpy cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic xpy rustdoc for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\nprintf 'synthetic xpy host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic xpy target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\" \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\n"
    }

    fn sha256_file_hex(path: &Path) -> String {
        sha256_hex(&fs::read(path).unwrap())
    }

    fn sample_sha256_hex(ch: char) -> String {
        assert!(ch.is_ascii_hexdigit(), "sample digest char must be hex");
        assert!(!ch.is_ascii_uppercase(), "sample digest char must be lowercase");
        std::iter::repeat_n(ch, SHA256_HEX_CHAR_COUNT).collect()
    }

    #[cfg(unix)]
    fn script_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;

        fs::metadata(path).unwrap().permissions().mode() & EXECUTABLE_MODE
    }

    fn write_fake_provider(root: &Path) -> RustSourceProviderMetadata {
        write_bytes(root, "bin/rustc", b"rustc");
        write_bytes(root, "bin/cargo", b"cargo");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib", b"host-std");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib", b"target-std");
        refresh_fake_receipt_and_metadata(root)
    }

    #[cfg(unix)]
    fn write_synthetic_smoke_provider(root: &Path, sentinel_path: Option<&Path>) -> RustSourceProviderMetadata {
        write_bytes(root, "bin/rustc", synthetic_rustc_script(sentinel_path).as_bytes());
        make_executable(&root.join("bin/rustc"));
        write_bytes(root, "bin/cargo", b"cargo");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib", b"host-std");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib", b"target-std");
        refresh_fake_receipt_and_metadata(root)
    }

    fn refresh_fake_receipt_and_metadata(root: &Path) -> RustSourceProviderMetadata {
        let receipt = fake_receipt(root);
        write_json(&root.join("share/mantle-rust-provider/receipts/build.json"), &receipt).unwrap();
        let metadata = fake_metadata(root);
        write_json(&root.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();
        metadata
    }

    #[cfg(unix)]
    fn synthetic_rustc_script(sentinel_path: Option<&Path>) -> String {
        let sentinel_write =
            sentinel_path.map(|path| format!("printf launched > '{}'\n", path.display())).unwrap_or_default();
        format!(
            "#!/bin/sh\n{sentinel_write}out=''\nprev=''\nfor arg in \"$@\"; do\n  if [ \"$prev\" = '-o' ]; then out=\"$arg\"; prev=''; continue; fi\n  if [ \"$arg\" = '-o' ]; then prev='-o'; continue; fi\ndone\nif [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi\nprintf 'synthetic rlib\\n' > \"$out\"\necho synthetic rustc smoke\n"
        )
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(EXECUTABLE_MODE);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn fake_receipt(root: &Path) -> RustSourceProviderBuildReceipt {
        RustSourceProviderBuildReceipt {
            schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
            receipt_id: "build-receipt".to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            source_ids: vec!["rust-src".to_string()],
            output_artifacts: vec![
                receipt_artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                receipt_artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                receipt_artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                receipt_artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
            ],
            build_steps: vec![RustProviderReceiptStep {
                name: "compile-rust-from-source".to_string(),
                program: "mantle-rust-source-stage".to_string(),
                arguments: vec!["bootstrap/rust-source.ncl".to_string()],
            }],
        }
    }

    fn fake_metadata(root: &Path) -> RustSourceProviderMetadata {
        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: true,
                uses_prebuilt_rust: false,
                build_recipe: "bootstrap/rust-source.ncl".to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: "rust-src".to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: blake3::hash(b"source").to_hex().to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: "build-receipt".to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-source-build-receipt".to_string(),
                path: "share/mantle-rust-provider/receipts/build.json".to_string(),
                digest_blake3: digest_path(root, "share/mantle-rust-provider/receipts/build.json"),
            }],
            artifacts: vec![
                artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::ProviderReceipt,
                    "build-receipt",
                    "share/mantle-rust-provider/receipts/build.json",
                ),
            ],
        }
    }

    fn artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderArtifact {
        RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
            source_id: "rust-src".to_string(),
            build_receipt_id: "build-receipt".to_string(),
        }
    }

    fn receipt_artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderReceiptArtifact {
        RustProviderReceiptArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
        }
    }

    fn write_bytes(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn digest_path(root: &Path, relative: &str) -> String {
        content_digest_blake3(&root.join(relative)).unwrap()
    }
}
