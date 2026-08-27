//! Shell/orchestration for the source-built Rust compiler/sysroot provider.
//!
//! This module performs filesystem work around the pure provider metadata
//! validator in `source_toolchain_closure`. It fails closed unless every
//! source-built stage reaches a validated final provider candidate; only then
//! does it copy that candidate into the requested provider output directory.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::time::Duration;

use crate::full_source_rust_binding::FULL_SOURCE_RUST_STAGE_PARALLEL_JOB_COUNT_MAX;
use crate::full_source_rust_binding::FullSourceRustHostToolRole;
use crate::full_source_rust_binding_shell::FullSourceRustExecutionContext;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchCapabilities;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchOperation;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchOperationKind;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchPhase;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchPlan;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchPlanInput;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchSourceIdentity;
use crate::rust_bootstrap_patch_plan::RustBootstrapPatchStage;
use crate::rust_bootstrap_patch_plan::derive_rust_bootstrap_patch_plan;
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

pub(crate) const RUST_SOURCE_PROVIDER_BLOCKED_REASON: &str = "source-built Rust provider completion remains pending until every planned stage passes and the validated final provider is published";

const DIRECTORY_DIGEST_MAX_ENTRIES: usize = 1_000_000;
const GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS: u32 = 16;
const GENERATED_SCRIPT_LAUNCH_RETRY_DELAY_MS: u64 = 20;
const SHELL_COMMAND_NOT_EXECUTABLE_EXIT_CODE: i32 = 126;
const TEXT_FILE_BUSY_MESSAGE: &str = "Text file busy";
pub(crate) const RUST_BOOTSTRAP_JOB_COUNT: u32 = 4;
#[cfg(test)]
#[cfg(unix)]
const UNIX_EXIT_STATUS_SHIFT_BITS: u32 = 8;
const _: () = assert!(GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS > 1);
const _: () = assert!(GENERATED_SCRIPT_LAUNCH_RETRY_DELAY_MS > 0);
const _: () = assert!(SHELL_COMMAND_NOT_EXECUTABLE_EXIT_CODE > 1);
const _: () = assert!(RUST_BOOTSTRAP_JOB_COUNT > 0);
const _: () = assert!(RUST_BOOTSTRAP_JOB_COUNT <= FULL_SOURCE_RUST_STAGE_PARALLEL_JOB_COUNT_MAX);
const ELF_MAGIC: [u8; ELF_MAGIC_LEN] = [0x7f, b'E', b'L', b'F'];
const ELF_MAGIC_LEN: usize = 4;
const BLAKE3_DIGEST_HEX_LEN: usize = 64;
const RUST_SOURCE_PROVIDER_PLAN_FILE: &str = "rust-source-plan.ncl";
const RUST_PROVIDER_ACTION_EVIDENCE_DIR: &str = "rust-provider-action-trust";
pub(crate) const RUST_PROVIDER_ACTION_EVIDENCE_RELATIVE_PATH: &str = "share/mantle-rust-provider/action-trust";
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
const RUSTC_STAGE1_BOOTSTRAP_TOOLS_JSON: &str = "[\"cargo\"]";
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
const STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR: &str = "share/mantle-rust-provider/construction-evidence";
const STAGE_CONSTRUCTION_EVIDENCE_FILE_COUNT: usize = 6;
const STAGE_EVIDENCE_PLAN_FILE: &str = "stage-plan.json";
const STAGE_EVIDENCE_SCRIPT_FILE: &str = "stage-script.sh";
const STAGE_EVIDENCE_BUILD_FILE: &str = "stage-build.json";
const STAGE_EVIDENCE_RECEIPT_FILE: &str = "provider-receipt.json";
const STAGE_EVIDENCE_METADATA_FILE: &str = "provider-metadata.json";
const STAGE_EVIDENCE_SMOKE_FILE: &str = "smoke-summary.json";
const RUSTC_FINAL_SOURCE_BUILD_SCRIPT: &str = "mantle-rustc-final-build.sh";
const RUSTC_FINAL_GENERATED_BUILD_SCRIPT: &str = "mantle-generated-rustc-final-build.sh";
const RUSTC_FINAL_XPY_GOALS: &str = "install rustc cargo library/std";
const RUSTC_FINAL_BOOTSTRAP_TOOLS_JSON: &str = "[\"cargo\", \"rustdoc\"]";
const RUSTC_FINAL_SHELL_PROGRAM: &str = "sh";
const RUSTC_SOURCE_XPY_SCRIPT: &str = "x.py";
const RUSTC_SOURCE_BOOTSTRAP_MANIFEST: &str = "src/bootstrap/Cargo.toml";
const RUSTC_SOURCE_CRANELIFT_MANIFEST: &str = "compiler/rustc_codegen_cranelift/Cargo.toml";
const RUSTC_SOURCE_CODEGEN_GCC_MANIFEST: &str = "compiler/rustc_codegen_gcc/Cargo.toml";
const RUSTC_SOURCE_RUSTC_DRIVER_MANIFEST: &str = "compiler/rustc_driver/Cargo.toml";
const RUSTC_SOURCE_FILESEARCH_SOURCE: &str = "compiler/rustc_session/src/filesearch.rs";
const RUSTC_SOURCE_TOOL_BUILD_SOURCE: &str = "src/bootstrap/src/core/build_steps/tool.rs";
const RUSTC_SOURCE_TOOL_BUILD_CARGO_ANCHOR_LINE: &str = "        let mut cargo = prepare_tool_cargo(";
const RUSTC_SOURCE_TOOL_BUILD_CARGO_END_LINE: &str = "        );";
const RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_MARKER: &str =
    "normalizing Rust bootstrap rustc-private tool rlib lookup for static musl compiler host";
const RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_ERROR: &str =
    "Rust bootstrap tool source lacks expected ToolBuild cargo construction for rustc-private rlib lookup";
const RUSTC_SOURCE_FILESEARCH_DEFAULT_SYSROOT_LINE: &str = "    from_env_args_next()";
const RUSTC_SOURCE_FILESEARCH_DYLIB_FALLBACK_LINE: &str =
    "        .unwrap_or_else(|| default_from_rustc_driver_dll().expect(\"Failed finding sysroot\"))";
const RUSTC_SOURCE_FILESEARCH_ENV_FALLBACK_LINE: &str =
    "        .or_else(|| env::var_os(\"RUSTC_SYSROOT\").map(PathBuf::from))";
const RUSTC_SOURCE_GENERATED_CONFIG_FILE: &str = "mantle-rust-build-config.toml";
const RUSTC_SOURCE_GENERATED_CARGO_HOME_DIR: &str = "cargo-home";
const RUSTC_SOURCE_TARGET_LINKER_ALIAS_DIR: &str = "rust-bootstrap-target-linker-bin";
const RUSTC_SOURCE_TARGET_LINKER_RUNTIME_DIR: &str = "rust-bootstrap-target-linker-runtime";
const RUSTC_SOURCE_TARGET_CC_VAR: &str = "MANTLE_TARGET_CC";
const RUSTC_SOURCE_TARGET_CXX_VAR: &str = "MANTLE_TARGET_CXX";
const RUSTC_SOURCE_TARGET_AR_VAR: &str = "MANTLE_TARGET_AR";
const RUSTC_SOURCE_TARGET_RANLIB_VAR: &str = "MANTLE_TARGET_RANLIB";
const RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR: &str = "MANTLE_TARGET_MUSL_ROOT";
const RUSTC_SOURCE_LLVM_BUILD_CONFIG: &str = concat!(
    "build-config = { ",
    "\"LLVM_ENABLE_ZLIB\" = \"OFF\", ",
    "\"LLVM_TOOL_BUGPOINT_PASSES_BUILD\" = \"OFF\", ",
    "\"LLVM_TOOL_GOLD_BUILD\" = \"OFF\", ",
    "\"LLVM_TOOL_LLVM_SHLIB_BUILD\" = \"OFF\", ",
    "\"LLVM_TOOL_LTO_BUILD\" = \"OFF\", ",
    "\"LLVM_TOOL_REMARKS_SHLIB_BUILD\" = \"OFF\" ",
    "}\n",
);
const FIRST_STAGE_SCRIPT_FILE: &str = "run-mrustc-first-stage.sh";
const FIRST_STAGE_ARCHIVE_DIR: &str = "archives";
const FIRST_STAGE_SOURCE_DIR: &str = "sources";
const FIRST_STAGE_BUILD_DIR: &str = "build";
const GENERATED_SCRIPT_TEMP_DIR: &str = "tmp";
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
const BYTES_PER_MIB: u64 = BYTES_PER_KIB.saturating_mul(BYTES_PER_KIB);
const FIRST_STAGE_FETCH_MAX_BYTES: u64 = FIRST_STAGE_FETCH_MAX_MIB.saturating_mul(BYTES_PER_MIB);
const FIRST_STAGE_ARCHIVE_MAX_ENTRIES: u32 = 500_000;
const FIRST_STAGE_BUSYBOX_PATCH_SKIPPED_SECTION_COUNT: u32 = 2;
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
const FIRST_STAGE_MAKE_PARALLEL_ARG: &str = "-j \"$PARLEVEL\"";
const FIRST_STAGE_TRANSLATED_CARGO_MINICARGO_JOB_COUNT: u32 = 1;
const FIRST_STAGE_COPY_PROGRAM: &str = "cp";
const FIRST_STAGE_PKG_CONFIG_PROGRAM: &str = "pkg-config";
const FIRST_STAGE_CMAKE_PROGRAM: &str = "cmake";
const FULL_SOURCE_PERL_LIBRARY_RELATIVE_PATH: &str = "lib/5.10.1";
const FULL_SOURCE_GCC_SUBPROGRAM_PREFIX_RELATIVE_PATH: &str = "libexec/gcc/x86_64-unknown-linux-musl/10.5.0";
const GCC_EXEC_PREFIX_ENV: &str = "GCC_EXEC_PREFIX";
const COMPILER_PATH_ENV: &str = "COMPILER_PATH";
const FIRST_STAGE_CC_PROGRAM: &str = "cc";
const FIRST_STAGE_CXX_PROGRAM: &str = "c++";
const FIRST_STAGE_MUSL_TRIPLE: &str = "x86_64-unknown-linux-musl";
const FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX: &str = "x86_64-linux-musl";
const FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR: &str = "MANTLE_TARGET_TOOLCHAIN_ROOT";
const FIRST_STAGE_SOURCE_ROOT_VAR: &str = "SOURCE_ROOT";
const FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING: &str = "source-root musl target gcc missing under root bin";
const FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE: &str = "source-root musl target toolchain root is incomplete";
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
const FIRST_STAGE_PROC_MACRO_RUNTIME_DIR: &str = "lib/mantle-runtime";
const FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH: &str = "lib/mantle-runtime/libc.so";
const FIRST_STAGE_RUSTC_DRIVER_MANIFEST_SUFFIX: &str = "compiler/rustc_driver/Cargo.toml";
const FIRST_STAGE_RUSTC_CONFIG_SOURCE: &str = "rustc-${RUSTC_VERSION}-src/compiler/rustc_session/src/config.rs";
const FIRST_STAGE_RUSTC_CONFIG_SYSROOT_ORIGINAL_LINE: &str =
    "        Sysroot { explicit, default: filesearch::default_sysroot() }";
const FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_1: &str = "        match explicit {";
const FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_2: &str =
    "            Some(explicit) => Sysroot { default: explicit.clone(), explicit: Some(explicit) },";
const FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_3: &str =
    "            None => Sysroot { explicit: None, default: filesearch::default_sysroot() },";
const FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_4: &str = "        }";
const FIRST_STAGE_RUSTC_DRIVER_MANIFEST: &str = "rustc-${RUSTC_VERSION}-src/compiler/rustc_driver/Cargo.toml";
const FIRST_STAGE_RUSTC_DRIVER_DYLIB_CRATE_TYPE_LINE: &str = "crate-type = [\"dylib\"]";
const FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE: &str = "crate-type = [\"rlib\"]";
const FIRST_STAGE_MINICARGO_BUILD_SOURCE: &str = "tools/minicargo/build.cpp";
const FIRST_STAGE_MINICARGO_OUT_DIR_ORIGINAL_LINE: &str =
    "    auto out_dir = parent.get_output_dir(m_is_for_host).to_absolute() / parent.get_build_script_out(m_manifest);";
const FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_MARKER: &str = "    if( m_manifest.build_script() != \"\" ) {";
const FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_LINE: &str =
    "        out_dir = parent.get_output_dir(true).to_absolute() / parent.get_build_script_out(m_manifest);";
const FIRST_STAGE_MINICARGO_RUSTC_FORCE_UNSTABLE_LINE: &str = "        args.push_back(\"force-unstable-if-unmarked\");";
const FIRST_STAGE_MINICARGO_RUSTC_THREADS_MARKER_LINE: &str =
    "        // Mantle: keep static musl first-stage rustc single-threaded.";
const FIRST_STAGE_MINICARGO_RUSTC_THREADS_FLAG_LINE: &str = "        args.push_back(\"-Z\");";
const FIRST_STAGE_MINICARGO_RUSTC_THREADS_VALUE_LINE: &str = "        args.push_back(\"threads=1\");";
const FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_ORIGINAL_LINE: &str = "MINICARGO ?= bin/minicargo$(EXESUF)";
const FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_PROTECTED_LINE: &str = "MINICARGO ?= $(CURDIR)/bin/minicargo$(EXESUF)";
const FIRST_STAGE_RUN_RUSTC_MINICARGO_ORIGINAL_LINE: &str = "MINICARGO ?= ../bin/minicargo$(EXESUF)";
const FIRST_STAGE_RUN_RUSTC_MINICARGO_PROTECTED_LINE: &str = "MINICARGO ?= $(abspath ../bin/minicargo$(EXESUF))";
const FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_ORIGINAL_TOKEN: &str = "$(BINDIR_S)rustc ";
const FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_PROTECTED_TOKEN: &str = "$(abspath $(BINDIR_S)rustc) ";
const FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_ORIGINAL_TOKEN: &str = "$(BINDIR_S)cargo ";
const FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_PROTECTED_TOKEN: &str = "$(abspath $(BINDIR_S)cargo) ";
const FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE: &str = "\t$V$(DBG) $(BINDIR)rustc -L $(LIBDIR) $< -o $@";
const FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_PROTECTED_LINE: &str =
    "\t$V$(DBG) $(abspath $(BINDIR)rustc) -C linker=$(MANTLE_TARGET_LINKER) -L $(LIBDIR) $< -o $@";
const FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE: &str = "\t./$@";
const FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_PROTECTED_LINE: &str = "\t$(abspath $@)";
const FIRST_STAGE_RUN_RUSTC_PROXY_PATH: &str = "run_rustc/rustc_proxy.sh";
const FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_ORIGINAL_LINE: &str = "    ${PROXY_RUSTC} \"$@\"";
const FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_PROTECTED_LINE: &str =
    "    ${PROXY_RUSTC} -C linker=\"$MANTLE_TARGET_LINKER\" \"$@\"";
const FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_ORIGINAL_LINE: &str = "    ${PROXY_MRUSTC} \"$@\"";
const FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_PROTECTED_LINE: &str =
    "    ${PROXY_MRUSTC} -C linker=\"$MANTLE_TARGET_LINKER\" \"$@\"";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_ORIGINAL_LINE: &str =
    "\t$Vcd $(RUSTCSRC)build && cmake $(addprefix -D , $(LLVM_CMAKE_OPTS)) ../$(LLVM_DIR)";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_PROTECTED_LINE: &str = "\t$Vcd $(RUSTCSRC)build && cmake $(addprefix -D , $(LLVM_CMAKE_OPTS)) ../$(LLVM_DIR) && $(MANTLE_LLVM_RULE_NORMALIZER) $(abspath $(RUSTCSRC)build)";
const FIRST_STAGE_MRUSTC_CODEGEN_C_SOURCE: &str = "src/trans/codegen_c.cpp";
const FIRST_STAGE_MRUSTC_SYSTEM_ORIGINAL_LINE: &str = "                int ec = system(cmd_ss.str().c_str());";
const FIRST_STAGE_MRUSTC_PROC_MACRO_SOURCE: &str = "src/expand/proc_macro.cpp";
const FIRST_STAGE_MRUSTC_PROC_MACRO_ARGV_ORIGINAL_LINE: &str = "    char*   argv[3] = { const_cast<char*>(executable), const_cast<char*>(proc_macro_desc.name.c_str()), nullptr };";
const FIRST_STAGE_MRUSTC_PROC_MACRO_SPAWN_ORIGINAL_LINE: &str =
    "    int rv = posix_spawn(&this->handles.child_pid, executable, &file_actions, nullptr, argv, environ);";
const FIRST_STAGE_LLVM_RULE_FILES_MAX: u32 = 4_096;
const FIRST_STAGE_LLVM_RULE_REPLACEMENTS_MAX: u32 = 4_096;
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_ORIGINAL_LINE: &str =
    "LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_PATCHED_LINE: &str = "LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_ZSTD=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF LLVM_ENABLE_BACKTRACES=OFF CMAKE_DISABLE_FIND_PACKAGE_Backtrace=ON CMAKE_DISABLE_FIND_PACKAGE_zstd=ON LLVM_TOOL_LTO_BUILD=OFF LLVM_BUILD_TOOLS=OFF";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_ORIGINAL_LINE: &str =
    "\t$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL)";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_PATCHED_PREFIX: &str =
    "\t$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL) llvm-headers vt_gen llvm-config";
const FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_STATIC_ARCHIVE_TARGETS: &str = "\
LLVMBinaryFormat LLVMMC LLVMAArch64Info LLVMBitstreamReader LLVMRemarks LLVMCore \
LLVMAArch64Utils LLVMCodeGenTypes LLVMAArch64Desc LLVMBitReader LLVMAsmParser \
LLVMIRReader LLVMMCParser LLVMTextAPI LLVMObject LLVMDebugInfoDWARF \
LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMDebugInfoPDB LLVMDebugInfoBTF \
LLVMSymbolize LLVMProfileData LLVMAnalysis LLVMBitWriter LLVMCGData \
LLVMTransformUtils LLVMObjCARCOpts LLVMAggressiveInstCombine LLVMInstCombine \
LLVMScalarOpts LLVMTarget LLVMCodeGen LLVMAsmPrinter LLVMCFGuard \
LLVMSelectionDAG LLVMGlobalISel LLVMSandboxIR LLVMVectorize LLVMAArch64CodeGen \
LLVMAArch64AsmParser LLVMMCDisassembler LLVMAArch64Disassembler LLVMARMInfo \
LLVMARMUtils LLVMARMDesc LLVMFrontendOffloading LLVMFrontendAtomic LLVMFrontendOpenMP \
LLVMLinker LLVMInstrumentation LLVMipo LLVMARMCodeGen LLVMARMAsmParser \
LLVMARMDisassembler LLVMCoverage LLVMExtensions LLVMCoroutines LLVMHipStdPar \
LLVMIRPrinter LLVMPasses LLVMLTO LLVMX86Info LLVMX86Desc LLVMX86CodeGen \
LLVMX86AsmParser LLVMX86Disassembler LLVMMCA LLVMX86TargetMCA";
const FIRST_STAGE_RUN_RUSTC_LD_LIBRARY_PATH_LINE: &str = "RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))";
const FIRST_STAGE_RUN_RUSTC_RUNTIME_LD_LIBRARY_PATH_TEMPLATE: &str =
    "RUSTC_ENV_VARS += LD_LIBRARY_PATH=$target_runtime_dir:\\$(abspath \\$(PREFIX_2)lib):\\$(abspath \\$(LIBDIR))";
const FIRST_STAGE_RUN_RUSTC_STAGE2_ENV_LINE: &str = "CARGO_ENV_STAGE2_STD := CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)";
const FIRST_STAGE_RUN_RUSTC_STAGE2_RUNTIME_ENV_LINE: &str = "CARGO_ENV_STAGE2_STD := $(RUSTC_ENV_VARS) CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)";
const FIRST_STAGE_RUN_RUSTC_FINAL_ENV_LINE: &str = "CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)";
const FIRST_STAGE_RUN_RUSTC_FINAL_PREFIX2_ENV_LINE: &str = "CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_2)rustc) $(CARGO_ENV)";
const FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE: &str = "\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml";
const FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_ALL_STATIC_LINE: &str = "\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml --features all-static";
const FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_LINE: &str = "\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(abspath $(BINDIR_S)cargo) build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml";
const FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_ALL_STATIC_LINE: &str = "\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(abspath $(BINDIR_S)cargo) build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml --features all-static";
const FIRST_STAGE_RUN_RUSTC_STAGE1_RUSTC_RULE_LINE: &str = "$(BINDIR_S)rustc: ../output$(OUTDIR_SUF)/rustc";
const FIRST_STAGE_RUN_RUSTC_STAGE2_RUSTC_RULE_LINE: &str = "$(BINDIR_2)rustc: ../output$(OUTDIR_SUF)/rustc";
const FIRST_STAGE_RUN_RUSTC_COPY_PREREQ_LINE: &str = "\tcp $< $@";
const FIRST_STAGE_RUN_RUSTC_STAGE_SYSROOT_SYMLINK_LINE: &str = "\tln -sf \"$(abspath $<)\" \"$@.bin\"";
const FIRST_STAGE_RUN_RUSTC_STAGE1_SYSROOT_WRAPPER_LINE: &str =
    "\tprintf '#!/bin/sh\\nexec \"$$0.bin\" --sysroot \"$(abspath $(PREFIX_S))\" \"$$@\"\\n' >$@";
const FIRST_STAGE_RUN_RUSTC_STAGE2_SYSROOT_WRAPPER_LINE: &str =
    "\tprintf '#!/bin/sh\\nexec \"$$0.bin\" --sysroot \"$(abspath $(PREFIX_2))\" \"$$@\"\\n' >$@";
const FIRST_STAGE_RUN_RUSTC_CHMOD_LINE: &str = "\tchmod +x $@";
const FIRST_STAGE_RUN_RUSTC_FINAL_WRAPPER_LINE: &str = "\t$Vprintf '#!/bin/sh\\nd=$$(dirname $$0)\\nLD_LIBRARY_PATH=\"$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))\" $$d/rustc_binary \"$$@\"' >$@";
const FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_SYMLINK_LINE: &str = "\t$Vln -sf rustc_binary $(BINDIR)rustc_binary.sysroot";
const FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_WRAPPER_LINE: &str = "\t$Vprintf '#!/bin/sh\\nd=$$(dirname $$0)\\nLD_LIBRARY_PATH=\"$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))\" $$d/rustc_binary.sysroot --sysroot \"$$d/..\" \"$$@\"' >$@";
const FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT: &str = "libc.so";
const FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT: &str = "libgcc_s.so";
const FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT: &str = "libgcc_s.so.1";
const FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE: &str = "libstdc++.a";
const REQUIRED_DYNAMIC_RUNTIME_SHARED_OBJECTS: &[&str] = &[FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT];
const OPTIONAL_DYNAMIC_RUNTIME_SHARED_OBJECTS: &[&str] = &[FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT];
const FIRST_STAGE_TARGET_MUSL_LIBATOMIC_STATIC_ARCHIVE: &str = "libatomic.a";
const FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SHIM_SOURCE: &str = "mantle-libatomic-shim.c";
const FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SHIM_OBJECT: &str = "mantle-libatomic-shim.o";
const FIRST_STAGE_TARGET_MUSL_TLS_KEY_CAPACITY: u32 = 4_096;
const FIRST_STAGE_TARGET_MUSL_PTHREAD_TLS_WRAP_FLAGS: &str = "-Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific";
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
const FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT: &str = "Scrt1.o";
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
    "CARGO_BUILD_RUSTC_WRAPPER",
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
    GCC_EXEC_PREFIX_ENV,
    COMPILER_PATH_ENV,
    "HOST",
    "MANTLE_RUST_CACHE_POLICY",
    "MANTLE_RUSTC_MANIFEST",
    "MANTLE_RUSTC_MANIFEST_DIR",
    "MANTLE_RUSTC_MANIFEST_REF",
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
const PROVIDER_DYNAMIC_TOOL_BINARY_SUFFIX: &str = ".dynamic";
const PROVIDER_DYNAMIC_TOOL_RUNTIME_MISSING_EXIT_CODE: i32 = 1;
const PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH: &str = "bin/rustc_binary";
const PROVIDER_MRUSTC_RUSTC_BINARY_BASENAME: &str = "rustc_binary";
const PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_RELATIVE_PATH: &str = "bin/rustc_binary.sysroot";
const PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_BASENAME: &str = "rustc_binary.sysroot";
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
    pub(crate) action_trust: Option<crate::source_built_rust_provider_action::RustProviderActionEvidence>,
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
    full_source_context: Option<FullSourceRustExecutionContext>,
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
    full_source_context: Option<FullSourceRustExecutionContext>,
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
    full_source_context: Option<FullSourceRustExecutionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageSourceAcquisition {
    id: String,
    url: String,
    source_policy: String,
    source_input_path: Option<PathBuf>,
    archive_path: PathBuf,
    archive_sha256_hex: String,
    extracted_path: PathBuf,
    extracted_digest_blake3: String,
    archive_entry_count: u32,
}

#[derive(Debug)]
struct RustSourceProviderSourceBytes {
    bytes: Vec<u8>,
    source_policy: &'static str,
    source_input_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageBuild {
    stage_id: String,
    plan_digest_blake3: String,
    script_path: PathBuf,
    script_digest_blake3: String,
    parallel_job_count: u32,
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
    plan_digest_blake3: String,
    script_path: PathBuf,
    script_digest_blake3: String,
    parallel_job_count: u32,
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
    full_source_context: Option<FullSourceRustExecutionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcFinalBuild {
    stage_id: String,
    plan_digest_blake3: String,
    script_path: PathBuf,
    script_digest_blake3: String,
    parallel_job_count: u32,
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
pub(crate) struct RustSourceProviderBlocked {
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
    Blocked(Box<RustSourceProviderBlocked>),
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
            Self::Blocked(details) => {
                let RustSourceProviderBlocked {
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
                } = details.as_ref();
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

fn verify_optional_full_source_binding(
    output_path: &Path,
    publication: Option<&crate::full_source_rust_binding_shell::FullSourceRustBindingPublication>,
) -> Result<(), RustSourceProviderError> {
    let Some(publication) = publication else {
        return Ok(());
    };
    if let Err(error) = verify_materialized_full_source_binding(output_path, publication) {
        cleanup_failed_output(output_path);
        return Err(error);
    }
    assert!(output_path.is_absolute());
    assert!(!publication.content_digest_blake3.is_empty());
    Ok(())
}

pub(crate) fn materialize_rust_source_provider(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    materialize_rust_source_provider_internal(
        recipe_path,
        None,
        None,
        None,
        None,
        None,
        output_dir,
        scratch_dir,
        verbose,
    )
}

pub(crate) fn materialize_rust_source_provider_with_route_plan(
    recipe_path: &Path,
    route_plan_path: Option<&Path>,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    materialize_rust_source_provider_internal(
        recipe_path,
        route_plan_path,
        None,
        None,
        None,
        None,
        output_dir,
        scratch_dir,
        verbose,
    )
}

pub(crate) fn materialize_full_source_bound_rust_provider_with_route_plan(
    recipe_path: &Path,
    route_plan_path: Option<&Path>,
    admission_report_path: &Path,
    host_tool_manifest_path: &Path,
    rust_source_archive_dir: &Path,
    action_limits: Option<crate::source_built_rust_provider_action::RustProviderActionLimits>,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    materialize_rust_source_provider_internal(
        recipe_path,
        route_plan_path,
        Some(admission_report_path),
        Some(host_tool_manifest_path),
        Some(rust_source_archive_dir),
        action_limits,
        output_dir,
        scratch_dir,
        verbose,
    )
}

fn materialize_rust_source_provider_internal(
    recipe_path: &Path,
    route_plan_path: Option<&Path>,
    admission_report_path: Option<&Path>,
    host_tool_manifest_path: Option<&Path>,
    rust_source_archive_dir: Option<&Path>,
    action_limits: Option<crate::source_built_rust_provider_action::RustProviderActionLimits>,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    let full_source_context = match (admission_report_path, host_tool_manifest_path, rust_source_archive_dir) {
        (Some(admission), Some(host_tools), Some(source_archives)) => {
            Some(crate::full_source_rust_binding_shell::prepare_full_source_rust_execution_context(
                admission,
                host_tools,
                source_archives,
            )?)
        }
        (None, None, None) => None,
        _ => {
            return Err(RustSourceProviderError::Validate(
                "full-source admission, host-tool manifest, and offline Rust source archives must be supplied together"
                    .to_string(),
            ));
        }
    };
    let recipe_bytes = fs::read(recipe_path)
        .map_err(|err| RustSourceProviderError::Read(format!("recipe {}: {err}", recipe_path.display())))?;
    if recipe_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("recipe {} is empty", recipe_path.display())));
    }
    let recipe_digest_blake3 = blake3::hash(&recipe_bytes).to_hex().to_string();
    debug_assert!(!recipe_bytes.is_empty());
    debug_assert_eq!(recipe_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    let mut plan = plan_materialization(recipe_path, route_plan_path, output_dir, scratch_dir, &recipe_digest_blake3)?;
    plan.full_source_context = full_source_context;
    let route = load_rust_source_provider_route(&plan.route_plan_path)?;
    let boundary = prepare_first_stage_boundary(&plan, &route)?;
    write_first_stage_boundary(&boundary)?;
    let first_stage_sources = acquire_first_stage_sources(&boundary, verbose)?;
    write_first_stage_sources_manifest(&boundary, &first_stage_sources)?;
    let mut action_runtime = start_rust_provider_action_runtime(&plan, &route, action_limits)?;
    let first_scope = begin_rust_provider_action_stage(action_runtime.as_mut(), first_stage_action_input(&boundary))?;
    let first_result = (|| {
        let first_stage_build = run_first_stage_build(&boundary, verbose)?;
        write_first_stage_build_manifest(&boundary, &first_stage_build)?;
        let first_stage_candidate =
            assemble_first_stage_provider_candidate(&boundary, &route, &first_stage_sources, &first_stage_build)?;
        let first_stage_candidate_smoke = smoke_first_stage_provider_candidate(&boundary, &first_stage_candidate)?;
        write_first_stage_provider_candidate_manifest(&boundary, &first_stage_candidate)?;
        Ok((first_stage_candidate, first_stage_candidate_smoke))
    })();
    let (first_stage_candidate, first_stage_candidate_smoke) =
        finish_rust_provider_action_stage(action_runtime.as_mut(), first_scope, first_result)?;
    let first_stage_bootstrap_candidate = bootstrap_candidate_from_first_stage(&first_stage_candidate);
    let rustc_stage1_runs = run_rustc_stage1_provider_candidate_chain(
        &plan,
        &route,
        &first_stage_bootstrap_candidate,
        action_runtime.as_mut(),
        verbose,
    )?;
    let rustc_final_bootstrap_candidate = bootstrap_candidate_from_rustc_stage1(
        &rustc_stage1_runs
            .last()
            .ok_or_else(|| RustSourceProviderError::Validate("rustc stage1 chain produced no candidates".to_string()))?
            .candidate,
    );
    let rustc_final_run = run_rustc_final_provider_candidate(
        &plan,
        &route,
        &rustc_final_bootstrap_candidate,
        action_runtime.as_mut(),
        verbose,
    )?;
    persist_stage_construction_evidence(&boundary, &first_stage_candidate, &rustc_stage1_runs, &rustc_final_run)?;
    let action_trust = action_runtime
        .take()
        .map(crate::source_built_rust_provider_action::RustProviderActionRuntime::finish)
        .transpose()?;
    if let Some(evidence) = &action_trust {
        embed_rust_provider_action_evidence(&rustc_final_run.candidate.candidate_dir, evidence)?;
    }
    let binding_publication = admission_report_path
        .map(|report_path| {
            let host_tools = host_tool_manifest_path.ok_or_else(|| {
                RustSourceProviderError::Validate(
                    "full-source admission reached publication without its host-tool manifest".to_string(),
                )
            })?;
            crate::full_source_rust_binding_shell::bind_full_source_rust_provider_candidate(
                &rustc_final_run.candidate.candidate_dir,
                report_path,
                host_tools,
            )
        })
        .transpose()?;
    let mut materialized = promote_rustc_final_provider_candidate(&plan, &rustc_final_run)?;
    verify_optional_full_source_binding(&materialized.output_path, binding_publication.as_ref())?;
    materialized.action_trust = action_trust
        .map(|_evidence| {
            crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(
                &materialized.output_path.join(RUST_PROVIDER_ACTION_EVIDENCE_RELATIVE_PATH),
            )
        })
        .transpose()?;
    if let Err(err) =
        write_rustc_final_provider_candidate_manifest(&rustc_final_run.boundary, &rustc_final_run.candidate, true)
    {
        cleanup_failed_output(&materialized.output_path);
        return Err(err);
    }
    if verbose {
        emit_rust_source_materialization_progress(RustSourceMaterializationProgress {
            plan: &plan,
            boundary: &boundary,
            first_stage_candidate: &first_stage_candidate,
            first_stage_candidate_smoke: &first_stage_candidate_smoke,
            rustc_stage1_runs: &rustc_stage1_runs,
            rustc_final_run: &rustc_final_run,
        });
    }
    Ok(materialized)
}

fn start_rust_provider_action_runtime(
    plan: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    action_limits: Option<crate::source_built_rust_provider_action::RustProviderActionLimits>,
) -> Result<Option<crate::source_built_rust_provider_action::RustProviderActionRuntime>, RustSourceProviderError> {
    match (&plan.full_source_context, action_limits) {
        (Some(context), Some(limits)) => crate::source_built_rust_provider_action::RustProviderActionRuntime::start(
            context,
            &route.plan,
            &route.plan_digest_blake3,
            &plan.scratch_dir.join(RUST_PROVIDER_ACTION_EVIDENCE_DIR),
            limits,
        )
        .map(Some),
        (Some(_), None) | (None, None) => Ok(None),
        (None, Some(_limits)) => Err(RustSourceProviderError::Validate(
            "Rust provider action limits require full-source execution authority".to_string(),
        )),
    }
}

fn first_stage_action_input(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> crate::source_built_rust_provider_action::RustProviderStageActionInput {
    crate::source_built_rust_provider_action::RustProviderStageActionInput {
        stage_id: boundary.stage_id.clone(),
        stage_kind: boundary.stage_kind,
        predecessor_stage_id: None,
        plan_path: boundary.plan_path.clone(),
        script_path: boundary.script_path.clone(),
        sources_manifest_path: boundary.sources_manifest_path.clone(),
        bootstrap_metadata_path: None,
        output_roots: vec![
            boundary.source_dir.clone(),
            boundary.build_dir.clone(),
            boundary.provider_candidate_dir.clone(),
            boundary.provider_candidate_smoke_work_dir.clone(),
        ],
    }
}

fn rustc_stage1_action_input(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> crate::source_built_rust_provider_action::RustProviderStageActionInput {
    crate::source_built_rust_provider_action::RustProviderStageActionInput {
        stage_id: boundary.stage_id.clone(),
        stage_kind: boundary.stage_kind,
        predecessor_stage_id: Some(boundary.bootstrap_stage_id.clone()),
        plan_path: boundary.plan_path.clone(),
        script_path: boundary.script_path.clone(),
        sources_manifest_path: boundary.sources_manifest_path.clone(),
        bootstrap_metadata_path: Some(
            boundary.bootstrap_provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH),
        ),
        output_roots: vec![
            boundary.source_dir.clone(),
            boundary.build_dir.clone(),
            boundary.stage_output_dir.clone(),
            boundary.provider_candidate_dir.clone(),
            boundary.provider_candidate_smoke_work_dir.clone(),
        ],
    }
}

fn rustc_final_action_input(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> crate::source_built_rust_provider_action::RustProviderStageActionInput {
    crate::source_built_rust_provider_action::RustProviderStageActionInput {
        stage_id: boundary.stage_id.clone(),
        stage_kind: boundary.stage_kind,
        predecessor_stage_id: Some(boundary.bootstrap_stage_id.clone()),
        plan_path: boundary.plan_path.clone(),
        script_path: boundary.script_path.clone(),
        sources_manifest_path: boundary.sources_manifest_path.clone(),
        bootstrap_metadata_path: Some(
            boundary.bootstrap_provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH),
        ),
        output_roots: vec![
            boundary.source_dir.clone(),
            boundary.build_dir.clone(),
            boundary.stage_output_dir.clone(),
            boundary.provider_candidate_dir.clone(),
            boundary.provider_candidate_smoke_work_dir.clone(),
        ],
    }
}

fn begin_rust_provider_action_stage(
    runtime: Option<&mut crate::source_built_rust_provider_action::RustProviderActionRuntime>,
    input: crate::source_built_rust_provider_action::RustProviderStageActionInput,
) -> Result<Option<crate::source_built_rust_provider_action::RustProviderStageScope>, RustSourceProviderError> {
    runtime.map(|runtime| runtime.begin_stage(input)).transpose()
}

fn finish_rust_provider_action_stage<T>(
    runtime: Option<&mut crate::source_built_rust_provider_action::RustProviderActionRuntime>,
    scope: Option<crate::source_built_rust_provider_action::RustProviderStageScope>,
    result: Result<T, RustSourceProviderError>,
) -> Result<T, RustSourceProviderError> {
    let reconciliation = match (runtime, scope) {
        (Some(runtime), Some(scope)) => runtime.end_stage(scope, result.is_ok()),
        (None, None) => Ok(()),
        _ => Err(RustSourceProviderError::Build("Rust provider action runtime and scope presence differ".to_string())),
    };
    match (result, reconciliation) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_value), Err(error)) => Err(error),
        (Err(error), Err(reconciliation_error)) => Err(RustSourceProviderError::Build(format!(
            "{error}; Rust provider action reconciliation also failed: {reconciliation_error}"
        ))),
    }
}

fn embed_rust_provider_action_evidence(
    candidate_dir: &Path,
    evidence: &crate::source_built_rust_provider_action::RustProviderActionEvidence,
) -> Result<(), RustSourceProviderError> {
    let source = evidence
        .plan_path
        .parent()
        .filter(|parent| evidence.audit_path.parent() == Some(*parent))
        .filter(|parent| evidence.reconciliation_path.parent() == Some(*parent))
        .ok_or_else(|| {
            RustSourceProviderError::Validate(
                "Rust provider action evidence paths do not share one directory".to_string(),
            )
        })?;
    let destination = candidate_dir.join(RUST_PROVIDER_ACTION_EVIDENCE_RELATIVE_PATH);
    if destination.exists() {
        return Err(RustSourceProviderError::Copy(format!(
            "Rust provider action evidence destination already exists: {}",
            destination.display()
        )));
    }
    fs::create_dir_all(&destination)
        .map_err(|error| RustSourceProviderError::Copy(format!("create {}: {error}", destination.display())))?;
    copy_provider_prefix(source, &destination)?;
    crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(&destination)?;
    assert!(destination.join(crate::source_built_rust_provider_action::RUST_PROVIDER_ACTION_PLAN_FILE).is_file());
    assert!(
        destination
            .join(crate::source_built_rust_provider_action::RUST_PROVIDER_ACTION_RECONCILIATION_FILE)
            .is_file()
    );
    Ok(())
}

struct RustStageConstructionEvidenceSource<'a> {
    stage_id: &'a str,
    plan_path: &'a Path,
    script_path: &'a Path,
    build_manifest_path: &'a Path,
    receipt_path: PathBuf,
    metadata_path: &'a Path,
    smoke_summary_path: PathBuf,
}

fn persist_stage_construction_evidence(
    first_boundary: &RustSourceProviderFirstStageBoundary,
    first_candidate: &RustSourceProviderFirstStageProviderCandidate,
    rustc_stage1_runs: &[RustSourceProviderRustcStage1ProviderCandidateRun],
    rustc_final_run: &RustSourceProviderRustcFinalProviderCandidateRun,
) -> Result<(), RustSourceProviderError> {
    let stage_count = rustc_stage1_runs.len().saturating_add(2);
    let stage_count_max = RUSTC_STAGE1_CHAIN_MAX_STAGES.saturating_add(2);
    if stage_count > stage_count_max {
        return Err(RustSourceProviderError::Validate(format!(
            "stage construction evidence count {stage_count} exceeds {stage_count_max}"
        )));
    }
    let evidence_root = rustc_final_run.candidate.candidate_dir.join(STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR);
    fs::create_dir(&evidence_root).map_err(|error| {
        RustSourceProviderError::Copy(format!(
            "create stage construction evidence {}: {error}",
            evidence_root.display()
        ))
    })?;
    persist_stage_construction_evidence_source(&evidence_root, RustStageConstructionEvidenceSource {
        stage_id: &first_boundary.stage_id,
        plan_path: &first_boundary.plan_path,
        script_path: &first_boundary.script_path,
        build_manifest_path: &first_boundary.build_manifest_path,
        receipt_path: first_candidate.candidate_dir.join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH),
        metadata_path: &first_candidate.metadata_path,
        smoke_summary_path: first_boundary.provider_candidate_smoke_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE),
    })?;
    for run in rustc_stage1_runs {
        persist_stage_construction_evidence_source(&evidence_root, RustStageConstructionEvidenceSource {
            stage_id: &run.boundary.stage_id,
            plan_path: &run.boundary.plan_path,
            script_path: &run.boundary.script_path,
            build_manifest_path: &run.boundary.build_manifest_path,
            receipt_path: run.candidate.candidate_dir.join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH),
            metadata_path: &run.candidate.metadata_path,
            smoke_summary_path: run.boundary.provider_candidate_smoke_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE),
        })?;
    }
    persist_stage_construction_evidence_source(&evidence_root, RustStageConstructionEvidenceSource {
        stage_id: &rustc_final_run.boundary.stage_id,
        plan_path: &rustc_final_run.boundary.plan_path,
        script_path: &rustc_final_run.boundary.script_path,
        build_manifest_path: &rustc_final_run.boundary.build_manifest_path,
        receipt_path: rustc_final_run.candidate.candidate_dir.join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH),
        metadata_path: &rustc_final_run.candidate.metadata_path,
        smoke_summary_path: rustc_final_run.boundary.provider_candidate_smoke_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE),
    })
}

fn persist_stage_construction_evidence_source(
    evidence_root: &Path,
    source: RustStageConstructionEvidenceSource<'_>,
) -> Result<(), RustSourceProviderError> {
    let stage_dir = evidence_root.join(safe_source_component(source.stage_id)?);
    fs::create_dir(&stage_dir).map_err(|error| {
        RustSourceProviderError::Copy(format!("create stage evidence {}: {error}", stage_dir.display()))
    })?;
    let files = [
        (source.plan_path, STAGE_EVIDENCE_PLAN_FILE),
        (source.script_path, STAGE_EVIDENCE_SCRIPT_FILE),
        (source.build_manifest_path, STAGE_EVIDENCE_BUILD_FILE),
        (source.receipt_path.as_path(), STAGE_EVIDENCE_RECEIPT_FILE),
        (source.metadata_path, STAGE_EVIDENCE_METADATA_FILE),
        (source.smoke_summary_path.as_path(), STAGE_EVIDENCE_SMOKE_FILE),
    ];
    assert_eq!(files.len(), STAGE_CONSTRUCTION_EVIDENCE_FILE_COUNT);
    assert!(!source.stage_id.is_empty());
    for (source_path, file_name) in files {
        copy_stage_construction_evidence_file(source_path, &stage_dir.join(file_name))?;
    }
    Ok(())
}

fn copy_stage_construction_evidence_file(
    source_path: &Path,
    destination: &Path,
) -> Result<(), RustSourceProviderError> {
    if !source_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "stage construction evidence source {} is missing",
            source_path.display()
        )));
    }
    let source_digest_blake3 = file_digest_blake3(source_path)?;
    fs::copy(source_path, destination).map_err(|error| {
        RustSourceProviderError::Copy(format!(
            "copy stage construction evidence {} -> {}: {error}",
            source_path.display(),
            destination.display()
        ))
    })?;
    let destination_digest_blake3 = file_digest_blake3(destination)?;
    if destination_digest_blake3 != source_digest_blake3 {
        let _ = fs::remove_file(destination);
        return Err(RustSourceProviderError::Digest(format!(
            "stage construction evidence {} expected {}, got {}",
            destination.display(),
            source_digest_blake3,
            destination_digest_blake3
        )));
    }
    debug_assert!(destination.is_file());
    debug_assert_eq!(destination_digest_blake3, source_digest_blake3);
    Ok(())
}

fn verify_materialized_full_source_binding(
    output_dir: &Path,
    publication: &crate::full_source_rust_binding_shell::FullSourceRustBindingPublication,
) -> Result<(), RustSourceProviderError> {
    let copied_path = output_dir.join(crate::full_source_rust_binding_shell::FULL_SOURCE_RUST_BINDING_RELATIVE_PATH);
    let bytes = fs::read(&copied_path).map_err(|error| {
        RustSourceProviderError::Read(format!("copied full-source binding {}: {error}", copied_path.display()))
    })?;
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!(
            "copied full-source binding {} is empty",
            copied_path.display()
        )));
    }
    let copied_digest = blake3::hash(&bytes).to_hex().to_string();
    if copied_digest != publication.content_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "copied full-source binding digest expected {}, got {}",
            publication.content_digest_blake3, copied_digest
        )));
    }
    debug_assert_ne!(publication.path, copied_path);
    debug_assert!(publication.native_artifact_count > 1);
    Ok(())
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
    debug_assert_eq!(run.candidate.stage_id, run.boundary.stage_id);
    debug_assert_eq!(materialization.output_dir, run.boundary.output_dir);
    let candidate_validation = validate_materialized_rust_source_provider(&run.candidate.candidate_dir)?;
    let candidate_stage_evidence_path = run.candidate.candidate_dir.join(STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR);
    validate_first_stage_build_product(&candidate_stage_evidence_path, STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR)?;
    let candidate_stage_evidence_digest_blake3 = content_digest_blake3(&candidate_stage_evidence_path)?;
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
            cleanup_failed_output(&materialization.output_dir);
            return Err(err);
        }
    };
    if output_validation.metadata_digest_blake3 != run.candidate.metadata_digest_blake3 {
        cleanup_failed_output(&materialization.output_dir);
        return Err(RustSourceProviderError::Validate(format!(
            "materialized provider metadata digest expected {}, got {}",
            run.candidate.metadata_digest_blake3, output_validation.metadata_digest_blake3
        )));
    }
    let output_stage_evidence_path = materialization.output_dir.join(STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR);
    let output_stage_evidence_digest_blake3 = match content_digest_blake3(&output_stage_evidence_path) {
        Ok(digest) => digest,
        Err(error) => {
            cleanup_failed_output(&materialization.output_dir);
            return Err(error);
        }
    };
    if output_stage_evidence_digest_blake3 != candidate_stage_evidence_digest_blake3 {
        cleanup_failed_output(&materialization.output_dir);
        return Err(RustSourceProviderError::Digest(format!(
            "materialized stage construction evidence expected {}, got {}",
            candidate_stage_evidence_digest_blake3, output_stage_evidence_digest_blake3
        )));
    }
    Ok(RustSourceProviderMaterialization {
        output_path: materialization.output_dir.clone(),
        recipe_digest_blake3: materialization.recipe_digest_blake3.clone(),
        metadata_path: output_validation.metadata_path,
        metadata_digest_blake3: output_validation.metadata_digest_blake3,
        action_trust: None,
    })
}

struct RustSourceMaterializationProgress<'a> {
    plan: &'a RustSourceProviderMaterializationPlan,
    boundary: &'a RustSourceProviderFirstStageBoundary,
    first_stage_candidate: &'a RustSourceProviderFirstStageProviderCandidate,
    first_stage_candidate_smoke: &'a RustSourceProviderSmokeEvidence,
    rustc_stage1_runs: &'a [RustSourceProviderRustcStage1ProviderCandidateRun],
    rustc_final_run: &'a RustSourceProviderRustcFinalProviderCandidateRun,
}

fn emit_rust_source_materialization_progress(progress: RustSourceMaterializationProgress<'_>) {
    eprintln!("Rust source provider recipe: {}", progress.plan.recipe_path.display());
    eprintln!("  recipe_digest_blake3: {}", progress.plan.recipe_digest_blake3);
    eprintln!("  route_plan: {}", progress.boundary.route_plan_path.display());
    eprintln!("  route_plan_digest_blake3: {}", progress.boundary.route_plan_digest_blake3);
    eprintln!("  route_policy_digest_blake3: {}", progress.boundary.route_policy_digest_blake3);
    eprintln!("  planned_output: {}", progress.plan.output_dir.display());
    eprintln!("  planned_scratch: {}", progress.plan.scratch_dir.display());
    emit_first_stage_progress(progress.boundary, progress.first_stage_candidate, progress.first_stage_candidate_smoke);
    for run in progress.rustc_stage1_runs {
        emit_rustc_stage1_progress(run);
    }
    emit_rustc_final_progress(progress.rustc_final_run);
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
            cleanup_failed_output(output_dir);
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
    debug_assert!(!source.is_empty());
    debug_assert_eq!(plan.provider_dir, provider_dir);
    fs::write(&plan.source_path, source).map_err(|err| {
        RustSourceProviderError::Smoke(format!("write smoke source {}: {err}", plan.source_path.display()))
    })?;
    let rustc_args = smoke_rustc_args(&plan);
    let output = run_smoke_rustc_with_bounded_launch_retry(&plan.rustc_path, &rustc_args)
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
    debug_assert_eq!(copied_output_digest.len(), BLAKE3_DIGEST_HEX_LEN);
    debug_assert_eq!(copied_metadata_digest.len(), BLAKE3_DIGEST_HEX_LEN);
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
    debug_assert!(!metadata_bytes.is_empty());
    debug_assert_eq!(metadata_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
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
    debug_assert!(!bytes.is_empty());
    debug_assert!(!evidence.output_digest_blake3.is_empty());
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
    debug_assert!(!recipe_path.as_os_str().is_empty());
    debug_assert!(!route_plan_path.as_os_str().is_empty());
    Ok(RustSourceProviderMaterializationPlan {
        recipe_path: recipe_path.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        recipe_digest_blake3: recipe_digest_blake3.to_string(),
        route_plan_path,
        full_source_context: None,
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
    debug_assert!(!plan_path.as_os_str().is_empty());
    let plan_bytes = fs::read(plan_path)
        .map_err(|err| RustSourceProviderError::Read(format!("route plan {}: {err}", plan_path.display())))?;
    if plan_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("route plan {} is empty", plan_path.display())));
    }
    let plan_digest_blake3 = blake3::hash(&plan_bytes).to_hex().to_string();
    debug_assert_eq!(plan_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    let nickel_search_dirs: Vec<OsString> = Vec::new();
    let plan: RustSourceProviderBootstrapPlan =
        crunch_eval::evaluate_and_deserialize(plan_path, &nickel_search_dirs)
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
    debug_assert!(!stage.id.is_empty());
    debug_assert!(!expected_outputs.is_empty());
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
        full_source_context: materialization.full_source_context.clone(),
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
    debug_assert!(!stage.id.is_empty());
    debug_assert!(!stage_root.as_os_str().is_empty());
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
        full_source_context: materialization.full_source_context.clone(),
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
    mut action_runtime: Option<&mut crate::source_built_rust_provider_action::RustProviderActionRuntime>,
    verbose: bool,
) -> Result<Vec<RustSourceProviderRustcStage1ProviderCandidateRun>, RustSourceProviderError> {
    debug_assert!(!bootstrap_candidate.stage_id.is_empty());
    let mut current_bootstrap = bootstrap_candidate.clone();
    let mut scratch_layout = RustcStage1ScratchLayout::TopLevel;
    let mut runs = Vec::with_capacity(RUSTC_STAGE1_CHAIN_MAX_STAGES);
    for _ in 0..RUSTC_STAGE1_CHAIN_MAX_STAGES {
        if !has_next_rustc_stage1_stage(&route.plan, &current_bootstrap) {
            break;
        }
        let run = run_rustc_stage1_provider_candidate(
            materialization,
            route,
            &current_bootstrap,
            scratch_layout,
            action_runtime.as_deref_mut(),
            verbose,
        )?;
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
    debug_assert!(runs.len() <= RUSTC_STAGE1_CHAIN_MAX_STAGES);
    Ok(runs)
}

fn run_rustc_stage1_provider_candidate(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    scratch_layout: RustcStage1ScratchLayout,
    mut action_runtime: Option<&mut crate::source_built_rust_provider_action::RustProviderActionRuntime>,
    verbose: bool,
) -> Result<RustSourceProviderRustcStage1ProviderCandidateRun, RustSourceProviderError> {
    let boundary = prepare_rustc_stage1_boundary(materialization, route, bootstrap_candidate, scratch_layout)?;
    write_rustc_stage1_boundary(&boundary)?;
    let sources = acquire_rustc_stage1_sources(&boundary, verbose)?;
    write_rustc_stage1_sources_manifest(&boundary, &sources)?;
    let scope = begin_rust_provider_action_stage(action_runtime.as_deref_mut(), rustc_stage1_action_input(&boundary))?;
    let result = (|| {
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
    })();
    finish_rust_provider_action_stage(action_runtime, scope, result)
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
    debug_assert!(!stage.id.is_empty());
    debug_assert!(!stage_root.as_os_str().is_empty());
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
        full_source_context: materialization.full_source_context.clone(),
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
    mut action_runtime: Option<&mut crate::source_built_rust_provider_action::RustProviderActionRuntime>,
    verbose: bool,
) -> Result<RustSourceProviderRustcFinalProviderCandidateRun, RustSourceProviderError> {
    let boundary = prepare_rustc_final_boundary(materialization, route, bootstrap_candidate)?;
    write_rustc_final_boundary(&boundary)?;
    let sources = acquire_rustc_final_sources(&boundary, verbose)?;
    write_rustc_final_sources_manifest(&boundary, &sources)?;
    let scope = begin_rust_provider_action_stage(action_runtime.as_deref_mut(), rustc_final_action_input(&boundary))?;
    let result = (|| {
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
    })();
    finish_rust_provider_action_stage(action_runtime, scope, result)
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
    let patch_plan = rustc_stage1_patch_plan(boundary)?;
    let manifest = rustc_stage1_boundary_manifest(boundary, &patch_plan);
    write_json_pretty(&boundary.plan_path, &manifest, "rustc stage1 plan")?;
    let script = rustc_stage1_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write rustc stage1 script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn rustc_stage1_boundary_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    patch_plan: &RustBootstrapPatchPlan,
) -> serde_json::Value {
    serde_json::json!({
        "schema": RUSTC_STAGE1_PLAN_SCHEMA,
        "full_source_execution": full_source_context_manifest(boundary.full_source_context.as_ref()),
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
        "patch_plan": patch_plan,
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
    let patch_plan = rustc_stage1_patch_plan(boundary)?;
    let mut script = String::new();
    push_generated_script_shebang(&mut script, boundary.full_source_context.as_ref())?;
    push_rustc_stage1_script_header(&mut script, boundary, &inputs);
    if let Some(context) = boundary.full_source_context.as_ref() {
        push_full_source_tool_bindings(&mut script, context)?;
    }
    push_generated_scratch_temp_environment(&mut script);
    push_rustc_stage1_script_input_checks(&mut script, &patch_plan, boundary.full_source_context.as_ref())?;
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
    let patch_plan = rustc_final_patch_plan(boundary)?;
    let manifest = rustc_final_boundary_manifest(boundary, &patch_plan);
    write_json_pretty(&boundary.plan_path, &manifest, "rustc final plan")?;
    let script = rustc_final_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write rustc final script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn rustc_final_boundary_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    patch_plan: &RustBootstrapPatchPlan,
) -> serde_json::Value {
    serde_json::json!({
        "schema": RUSTC_FINAL_PLAN_SCHEMA,
        "full_source_execution": full_source_context_manifest(boundary.full_source_context.as_ref()),
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
        "patch_plan": patch_plan,
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
    let patch_plan = rustc_final_patch_plan(boundary)?;
    let mut script = String::new();
    push_generated_script_shebang(&mut script, boundary.full_source_context.as_ref())?;
    push_rustc_final_script_header(&mut script, boundary, &inputs);
    if let Some(context) = boundary.full_source_context.as_ref() {
        push_full_source_tool_bindings(&mut script, context)?;
    }
    push_generated_scratch_temp_environment(&mut script);
    push_rustc_final_script_input_checks(&mut script, &patch_plan, boundary.full_source_context.as_ref())?;
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
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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

fn push_rustc_stage1_script_input_checks(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
        patch_plan,
        full_source_context,
    )?;
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
    if full_source_context.is_some() {
        push_required_shell_path_file_check(
            script,
            "\"$SHELL_PROGRAM\"",
            "receipt-bound shell is required for rustc stage1",
            RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
        );
    } else {
        push_required_shell_command_check(script, "SHELL_PROGRAM", "sh is required for rustc stage1");
    }
    Ok(())
}

struct MissingBuildScriptMessage<'a>(&'a str);

impl<'a> From<&'a str> for MissingBuildScriptMessage<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

struct GeneratedBuildScriptFile<'a>(&'a str);

impl<'a> From<&'a str> for GeneratedBuildScriptFile<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

fn push_rustc_source_build_script_resolution<'a>(
    script: &mut String,
    missing_message: impl Into<MissingBuildScriptMessage<'a>>,
    generated_script_file: impl Into<GeneratedBuildScriptFile<'a>>,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    let missing_message = missing_message.into().0;
    let generated_script_file = generated_script_file.into().0;
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
    push_generated_rustc_source_build_script(script, patch_plan, full_source_context)?;
    script.push_str("  chmod +x \"$RESOLVED_BUILD_SCRIPT\"\n");
    script.push_str("fi\n");
    Ok(())
}

fn push_generated_rustc_source_build_script(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("  cat > \"$RESOLVED_BUILD_SCRIPT\" <<'MANTLE_RUST_SOURCE_GENERATED_SCRIPT'\n");
    push_generated_script_shebang(script, full_source_context)?;
    script.push_str("set -eu\n");
    script.push_str("CONFIG=\"$MANTLE_BUILD_DIR/");
    script.push_str(RUSTC_SOURCE_GENERATED_CONFIG_FILE);
    script.push_str("\"\n");
    script.push_str("mkdir -p \"$MANTLE_BUILD_DIR\" \"$MANTLE_STAGE_OUTPUT\"\n");
    push_rustc_source_build_env_scrub(script);
    push_rustc_source_build_tool_discovery(script, full_source_context)?;
    script.push_str("MANTLE_RUST_TOOLS=${MANTLE_RUST_BOOTSTRAP_TOOLS:-'[\"cargo\"]'}\n");
    script.push_str(
        "case \" $MANTLE_RUST_BUILD_GOALS \" in *' rustdoc '*) MANTLE_RUST_TOOLS='[\"cargo\", \"rustdoc\"]' ;; esac\n",
    );
    script.push_str("cat > \"$CONFIG\" <<MANTLE_RUST_BUILD_CONFIG\n");
    script.push_str("profile = \"compiler\"\n");
    script.push_str("change-id = \"ignore\"\n\n");
    script.push_str("[build]\n");
    script.push_str("build = \"$MANTLE_HOST_TRIPLE\"\n");
    script.push_str("host = [\"$MANTLE_HOST_TRIPLE\"]\n");
    script.push_str("target = [\"$MANTLE_HOST_TRIPLE\", \"$MANTLE_TARGET_TRIPLE\"]\n");
    script.push_str(&format!("jobs = {RUST_BOOTSTRAP_JOB_COUNT}\n"));
    script.push_str("cargo = \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\n");
    script.push_str("rustc = \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\n");
    script.push_str("extended = true\n");
    script.push_str("tools = $MANTLE_RUST_TOOLS\n");
    script.push_str("vendor = true\n");
    script.push_str("cargo-native-static = true\n");
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
    push_rustc_source_llvm_config(script, full_source_context);
    script.push_str("MANTLE_RUST_BUILD_CONFIG\n");
    push_rustc_source_patch_plan_operations(script, patch_plan)?;
    script.push_str("printf '%s\\n' \"using generated x.py Rust build adapter: $MANTLE_RUST_SOURCE\"\n");
    if full_source_context.is_some() {
        script.push_str("if [ ! -x \"$MANTLE_PYTHON_PROGRAM\" ]; then printf '%s\\n' 'receipt-bound Python is unavailable' >&2; exit ");
        script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
        script.push_str("; fi\n");
        script.push_str("\"$MANTLE_PYTHON_PROGRAM\" \"$MANTLE_RUST_SOURCE/");
        script.push_str(RUSTC_SOURCE_XPY_SCRIPT);
        script.push_str("\" --config \"$CONFIG\" $MANTLE_RUST_BUILD_GOALS\n");
    } else {
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
        script.push('\n');
        script.push_str("fi\n");
    }
    script.push_str("MANTLE_RUST_SOURCE_GENERATED_SCRIPT\n");
    Ok(())
}

fn push_rustc_source_llvm_config(script: &mut String, full_source_context: Option<&FullSourceRustExecutionContext>) {
    debug_assert!(!script.contains('\0'));
    script.push_str("download-ci-llvm = false\n");
    script.push_str("ninja = false\n");
    if full_source_context.is_some() {
        script.push_str("cflags = \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"\n");
        script.push_str("cxxflags = \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"\n");
    }
    script.push_str(RUSTC_SOURCE_LLVM_BUILD_CONFIG);
    assert!(script.contains("download-ci-llvm = false"));
    assert!(script.contains(RUSTC_SOURCE_LLVM_BUILD_CONFIG.trim_end()));
}

fn push_rustc_source_patch_plan_operations(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
) -> Result<(), RustSourceProviderError> {
    for operation in patch_plan
        .operations
        .iter()
        .filter(|operation| operation.phase == RustBootstrapPatchPhase::RustBootstrapBeforeXpy)
    {
        push_rustc_source_patch_plan_operation(script, operation)?;
    }
    Ok(())
}

fn push_rustc_source_patch_plan_operation(
    script: &mut String,
    operation: &RustBootstrapPatchOperation,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    match operation.kind {
        RustBootstrapPatchOperationKind::RustBootstrapTargetToolConfig => {
            push_rustc_source_build_target_tool_config(script);
        }
        RustBootstrapPatchOperationKind::RustBootstrapWorkspaceIsolation => {
            push_rustc_source_bootstrap_workspace_isolation(script);
        }
        RustBootstrapPatchOperationKind::RustBootstrapRustcDriverRlib => {
            push_rustc_source_musl_rustc_driver_rlib_patch(script);
        }
        RustBootstrapPatchOperationKind::RustBootstrapSysrootFallback => {
            push_rustc_source_musl_sysroot_env_fallback_patch(script);
        }
        RustBootstrapPatchOperationKind::RustBootstrapRustcPrivateToolRlibLookup => {
            push_rustc_source_musl_rustc_private_tool_sysroot_patch(script);
        }
        RustBootstrapPatchOperationKind::ProviderContractAssertion => {}
        unsupported => {
            return Err(RustSourceProviderError::Validate(format!(
                "unsupported Rust bootstrap script patch operation {unsupported:?}"
            )));
        }
    }
    Ok(())
}

fn push_rustc_source_musl_rustc_driver_rlib_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$MANTLE_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str(
        "  printf '%s\\n' 'normalizing Rust bootstrap rustc_driver crate type for static musl compiler host'\n",
    );
    script.push_str("  rustc_driver_manifest=\"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_RUSTC_DRIVER_MANIFEST);
    script.push_str("\"\n");
    script.push_str(&format!(
        "  if [ ! -f \"$rustc_driver_manifest\" ]; then printf '%s\\n' 'Rust bootstrap rustc_driver manifest missing before musl host normalization' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  rustc_driver_manifest_tmp=\"$MANTLE_BUILD_DIR/rust-bootstrap-rustc-driver-Cargo.toml\"\n");
    script.push_str("  rustc_driver_replaced=false\n");
    script.push_str("  rustc_driver_seen_rlib=false\n");
    script.push_str("  : > \"$rustc_driver_manifest_tmp\"\n");
    script.push_str("  while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("    case \"$line\" in\n");
    script.push_str(&format!(
        "      {dylib}) printf '%s\\n' {rlib} >> \"$rustc_driver_manifest_tmp\"; rustc_driver_replaced=true ;;\n",
        dylib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_DYLIB_CRATE_TYPE_LINE),
        rlib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE),
    ));
    script.push_str(&format!(
        "      {rlib}) printf '%s\\n' \"$line\" >> \"$rustc_driver_manifest_tmp\"; rustc_driver_seen_rlib=true ;;\n",
        rlib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE),
    ));
    script.push_str("      *) printf '%s\\n' \"$line\" >> \"$rustc_driver_manifest_tmp\" ;;\n");
    script.push_str("    esac\n");
    script.push_str("  done < \"$rustc_driver_manifest\"\n");
    script.push_str(&format!(
        "  if [ \"$rustc_driver_replaced\" = false ] && [ \"$rustc_driver_seen_rlib\" = false ]; then printf '%s\\n' 'Rust bootstrap rustc_driver manifest lacks expected crate-type line for musl host normalization' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  mv \"$rustc_driver_manifest_tmp\" \"$rustc_driver_manifest\"\n");
    script.push_str("fi\n");
}

fn push_rustc_source_musl_sysroot_env_fallback_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$MANTLE_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("  printf '%s\\n' 'normalizing Rust bootstrap sysroot fallback for static musl compiler host'\n");
    script.push_str("  filesearch_source=\"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_FILESEARCH_SOURCE);
    script.push_str("\"\n");
    script.push_str(&format!(
        "  if [ ! -f \"$filesearch_source\" ]; then printf '%s\\n' 'Rust bootstrap filesearch source missing before musl sysroot normalization' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  filesearch_tmp=\"$MANTLE_BUILD_DIR/rust-bootstrap-filesearch.rs\"\n");
    script.push_str("  filesearch_replaced=false\n");
    script.push_str("  filesearch_seen_env_fallback=false\n");
    script.push_str("  filesearch_after_from_env=false\n");
    script.push_str("  : > \"$filesearch_tmp\"\n");
    script.push_str("  while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("    if [ \"$filesearch_after_from_env\" = true ]; then\n");
    script.push_str("      case \"$line\" in\n");
    script.push_str(&format!(
        "        {dylib_fallback}) printf '%s\\n' {env_fallback} {dylib_fallback} >> \"$filesearch_tmp\"; filesearch_replaced=true; filesearch_after_from_env=false; continue ;;\n",
        dylib_fallback = shell_quote(RUSTC_SOURCE_FILESEARCH_DYLIB_FALLBACK_LINE),
        env_fallback = shell_quote(RUSTC_SOURCE_FILESEARCH_ENV_FALLBACK_LINE),
    ));
    script.push_str(&format!(
        "        {env_fallback}) printf '%s\\n' \"$line\" >> \"$filesearch_tmp\"; filesearch_seen_env_fallback=true; filesearch_after_from_env=false; continue ;;\n",
        env_fallback = shell_quote(RUSTC_SOURCE_FILESEARCH_ENV_FALLBACK_LINE),
    ));
    script.push_str("      esac\n");
    script.push_str("      filesearch_after_from_env=false\n");
    script.push_str("    fi\n");
    script.push_str("    case \"$line\" in\n");
    script.push_str(&format!(
        "      {from_env}) printf '%s\\n' \"$line\" >> \"$filesearch_tmp\"; filesearch_after_from_env=true; continue ;;\n",
        from_env = shell_quote(RUSTC_SOURCE_FILESEARCH_DEFAULT_SYSROOT_LINE),
    ));
    script.push_str(&format!(
        "      {env_fallback}) filesearch_seen_env_fallback=true ;;\n",
        env_fallback = shell_quote(RUSTC_SOURCE_FILESEARCH_ENV_FALLBACK_LINE),
    ));
    script.push_str("    esac\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$filesearch_tmp\"\n");
    script.push_str("  done < \"$filesearch_source\"\n");
    script.push_str(&format!(
        "  if [ \"$filesearch_replaced\" = false ] && [ \"$filesearch_seen_env_fallback\" = false ]; then printf '%s\\n' 'Rust bootstrap filesearch source lacks expected sysroot fallback lines for musl host normalization' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  mv \"$filesearch_tmp\" \"$filesearch_source\"\n");
    script.push_str("fi\n");
}

fn push_rustc_source_musl_rustc_private_tool_sysroot_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$MANTLE_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("  printf '%s\\n' ");
    script.push_str(&shell_quote(RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_MARKER));
    script.push('\n');
    script.push_str("  tool_build_source=\"$MANTLE_RUST_SOURCE/");
    script.push_str(RUSTC_SOURCE_TOOL_BUILD_SOURCE);
    script.push_str("\"\n");
    script.push_str(&format!(
        "  if [ ! -f \"$tool_build_source\" ]; then printf '%s\\n' 'Rust bootstrap tool source missing before rustc-private rlib lookup normalization' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  tool_build_tmp=\"$MANTLE_BUILD_DIR/rust-bootstrap-tool.rs\"\n");
    script.push_str("  tool_build_after_cargo_anchor=false\n");
    script.push_str("  tool_build_inserted=false\n");
    script.push_str("  : > \"$tool_build_tmp\"\n");
    script.push_str("  while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$tool_build_tmp\"\n");
    script.push_str("    if [ \"$tool_build_after_cargo_anchor\" = true ]; then\n");
    script.push_str("      case \"$line\" in\n");
    script
        .push_str(&format!("        {cargo_end})\n", cargo_end = shell_quote(RUSTC_SOURCE_TOOL_BUILD_CARGO_END_LINE),));
    script.push_str("          printf '%s\\n' \\\n");
    script.push_str("'        if self.mode == Mode::ToolRustcPrivate {' \\\n");
    script.push_str("'            cargo.append_to_env(' \\\n");
    script.push_str("'                \"RUSTC_ADDITIONAL_SYSROOT_PATHS\",' \\\n");
    script.push_str("'                builder' \\\n");
    script.push_str("'                    .cargo_out(self.build_compiler, Mode::Rustc, target)' \\\n");
    script.push_str("'                    .join(\"deps\")' \\\n");
    script.push_str("'                    .to_str()' \\\n");
    script.push_str("'                    .expect(\"rustc private tool rlib path should be UTF-8\")' \\\n");
    script.push_str("'                    .to_owned(),' \\\n");
    script.push_str("'                \",\",' \\\n");
    script.push_str("'            );' \\\n");
    script.push_str("'        }' >> \"$tool_build_tmp\"\n");
    script.push_str("          tool_build_inserted=true\n");
    script.push_str("          tool_build_after_cargo_anchor=false\n");
    script.push_str("          continue\n");
    script.push_str("          ;;\n");
    script.push_str("      esac\n");
    script.push_str("    fi\n");
    script.push_str("    case \"$line\" in\n");
    script.push_str(&format!(
        "      {cargo_anchor}) tool_build_after_cargo_anchor=true ;;\n",
        cargo_anchor = shell_quote(RUSTC_SOURCE_TOOL_BUILD_CARGO_ANCHOR_LINE),
    ));
    script.push_str("    esac\n");
    script.push_str("  done < \"$tool_build_source\"\n");
    script.push_str(&format!(
        "  if [ \"$tool_build_inserted\" = false ]; then printf '%s\\n' {error} >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n",
        error = shell_quote(RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_ERROR),
    ));
    script.push_str("  mv \"$tool_build_tmp\" \"$tool_build_source\"\n");
    script.push_str("fi\n");
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

fn push_rustc_source_target_toolchain_root_preference(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("=${");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str(":-${");
    script.push_str(FIRST_STAGE_SOURCE_ROOT_VAR);
    script.push_str(":-}}\n");
    script.push_str("if [ -n \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\n");
    script.push_str("  for candidate_name in ");
    script.push_str(&musl_target_tool_program_candidates_shell_words("gcc"));
    script.push_str("; do\n");
    script.push_str("    if [ -x \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("/bin/$candidate_name\" ]; then ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("/bin/$candidate_name\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("  if [ -z \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then printf '%s\\n' ");
    script.push_str(&shell_quote(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING));
    script.push_str(" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("  printf '%s\\n' \"using explicit Rust bootstrap source-root musl target toolchain: $");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("\"\n");
    script.push_str("fi\n");
}

fn push_rustc_source_target_toolchain_root_validation(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("  if [ -n \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("    for target_required_tool in \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("\" \"$");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("\" \"$");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("\" \"$target_tool_dir/$target_tool_prefix-objcopy\"; do\n");
    script.push_str("      if [ ! -x \"$target_required_tool\" ]; then printf '%s\\n' \"");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE);
    script.push_str(": $target_required_tool\" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("    done\n");
    script.push_str("    for target_required_file in \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.so\" \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libgcc_s.so.1\"; do\n");
    script.push_str("      if [ ! -f \"$target_required_file\" ]; then printf '%s\\n' \"");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE);
    script.push_str(": $target_required_file\" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("    done\n");
    script.push_str("  fi\n");
}

fn push_first_stage_target_toolchain_root_preference(script: &mut String) {
    script.push_str("target_toolchain_root=${");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str(":-${");
    script.push_str(FIRST_STAGE_SOURCE_ROOT_VAR);
    script.push_str(":-}}\n");
    script.push_str("if [ -n \"$target_toolchain_root\" ]; then\n");
    script.push_str("  TARGET_CC_PROGRAM=\n");
    script.push_str("  for candidate_name in $TARGET_CC_PROGRAM_CANDIDATES; do\n");
    script.push_str("    if [ -x \"$target_toolchain_root/bin/$candidate_name\" ]; then TARGET_CC_PROGRAM=\"$target_toolchain_root/bin/$candidate_name\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("  if [ -z \"$TARGET_CC_PROGRAM\" ]; then printf '%s\\n' ");
    script.push_str(&shell_quote(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING));
    script.push_str(" >&2; exit ");
    script.push_str(&FIRST_STAGE_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("  printf '%s\\n' \"using explicit source-root musl target toolchain: $target_toolchain_root\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_target_toolchain_root_validation(script: &mut String) {
    script.push_str("if [ -n \"$target_toolchain_root\" ]; then\n");
    script.push_str("  for target_required_tool in \"$target_tool_dir/$target_cxx_program\" \"$target_tool_dir/$target_ar_program\" \"$target_tool_dir/$target_ranlib_program\" \"$target_tool_dir/$target_objcopy_program\"; do\n");
    script.push_str("    if [ ! -x \"$target_required_tool\" ]; then printf '%s\\n' \"");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE);
    script.push_str(": $target_required_tool\" >&2; exit ");
    script.push_str(&FIRST_STAGE_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("  done\n");
    script.push_str("  for target_required_file in \"$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libc.so\" \"$target_toolchain_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/libgcc_s.so.1\"; do\n");
    script.push_str("    if [ ! -f \"$target_required_file\" ]; then printf '%s\\n' \"");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE);
    script.push_str(": $target_required_file\" >&2; exit ");
    script.push_str(&FIRST_STAGE_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("  done\n");
    script.push_str("fi\n");
}

fn push_rustc_source_build_tool_discovery(
    script: &mut String,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    if let Some(context) = full_source_context {
        push_full_source_tool_bindings(script, context)?;
        script.push_str("export CFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"\n");
        script.push_str("export CPPFLAGS=\"$MANTLE_LINUX_HEADERS_CFLAGS\"\n");
        script.push_str("export CXXFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"\n");
        script.push_str("export LDFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\"\n");
        push_rustc_source_explicit_target_compiler(script);
    } else {
        push_rustc_source_make_and_cmake_discovery(script);
        push_rustc_source_target_compiler_discovery(script);
    }
    push_rustc_source_target_tool_prefix(script);
    push_rustc_source_target_runtime_config(script, full_source_context.is_some());
    Ok(())
}

fn push_rustc_source_explicit_target_compiler(script: &mut String) {
    script.push_str("MANTLE_TARGET_MUSL_MACHINE_ALIASES=");
    script.push_str(&shell_quote(&musl_target_machine_aliases_shell_words()));
    script.push('\n');
    script.push_str("MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT=");
    script.push_str(&shell_quote(FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX));
    script.push('\n');
    script.push_str("if [ ! -x \"$MANTLE_TARGET_CC\" ]; then printf '%s\\n' 'receipt-bound Rust target compiler is missing' >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; fi\n");
    script.push_str("target_tool_dir=${MANTLE_TARGET_CC%/*}\n");
    script.push_str("target_cc_basename=${MANTLE_TARGET_CC##*/}\n");
    script.push_str("target_cc_machine=$(\"$MANTLE_TARGET_CC\" -dumpmachine 2>/dev/null || true)\n");
    script.push_str("case \" $MANTLE_TARGET_MUSL_MACHINE_ALIASES \" in *\" $target_cc_machine \"*) ;; *) printf '%s\\n' \"receipt-bound Rust target compiler machine is invalid: $target_cc_machine\" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str(" ;; esac\n");
}

fn push_rustc_source_make_and_cmake_discovery(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
}

fn push_rustc_source_target_compiler_discovery(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=${");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str(":-}\n");
    script.push_str("MANTLE_TARGET_MUSL_MACHINE_ALIASES=");
    script.push_str(&shell_quote(&musl_target_machine_aliases_shell_words()));
    script.push('\n');
    script.push_str("MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT=");
    script.push_str(&shell_quote(FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX));
    script.push('\n');
    push_rustc_source_target_toolchain_root_preference(script);
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
    script.push_str("  case \" $MANTLE_TARGET_MUSL_MACHINE_ALIASES \" in *\" $target_cc_machine \"*) ;; *) printf '%s\\n' \"Rust bootstrap target gcc machine expected one of: $MANTLE_TARGET_MUSL_MACHINE_ALIASES; got $target_cc_machine\" >&2; if [ -n \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("\" ]; then exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; else ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=; fi ;; esac\n");
    script.push_str("fi\n");
}

fn push_rustc_source_target_tool_prefix(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("if [ -n \"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\" ]; then\n");
    script.push_str("  case \"$target_cc_basename\" in *-gcc) target_tool_prefix=${target_cc_basename%-gcc} ;; *) target_tool_prefix= ;; esac\n");
    script.push_str("  if [ -z \"$target_tool_prefix\" ]; then if [ -n \"$");
    script.push_str(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR);
    script.push_str("\" ]; then printf '%s\\n' ");
    script.push_str(&shell_quote(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE));
    script.push_str(" >&2; exit ");
    script.push_str(&RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE.to_string());
    script.push_str("; else ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=; fi; fi\n");
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
    push_rustc_source_target_toolchain_root_validation(script);
}

fn push_rustc_source_target_runtime_config(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("  target_wrapper_root=${target_tool_dir%/*}\n");
    script.push_str("  ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("=\n");
    script.push_str("  if [ -f \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push('/');
    script.push_str(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE);
    script.push_str("\" ]; then\n");
    script.push_str("    IFS= read -r ");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str(" < \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push('/');
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
    push_rustc_source_target_linker_wrapper(script, full_source_bound);
    script.push_str("    PATH=\"$target_alias_dir:$target_tool_dir:$PATH\"\n");
    script.push_str("    export ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push(' ');
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push(' ');
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push(' ');
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push(' ');
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str(" PATH\n");
    script.push_str("    printf '%s\\n' \"using Rust bootstrap target linker wrapper: $target_alias_dir/cc -> $target_cc_path; runtime CRT/unwind dir: $target_runtime_dir\"\n");
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

fn push_rustc_source_target_libgcc_shared_copy(script: &mut String) {
    script.push_str(&format!(
        "    for target_libgcc_shared_name in {FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT} {FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT}; do\n"
    ));
    script.push_str("      for target_libgcc_shared_dir in ");
    script.push_str("\"${");
    script.push_str(RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR);
    script.push_str("}/lib\" ");
    script.push_str("\"$target_orig_cc_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib\" ");
    script.push_str("\"$target_gcc_crt_dir\"; do\n");
    script.push_str("        if [ -f \"$target_libgcc_shared_dir/$target_libgcc_shared_name\" ]; then cp \"$target_libgcc_shared_dir/$target_libgcc_shared_name\" \"$target_runtime_dir/$target_libgcc_shared_name\"; break; fi\n");
    script.push_str("      done\n");
    script.push_str("    done\n");
}

fn push_rustc_source_target_linker_wrapper(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_rustc_source_target_linker_runtime(script);
    push_rustc_source_target_cc_wrapper(script);
    push_rustc_source_target_companion_wrappers(script, full_source_bound);
}

fn push_rustc_source_target_linker_runtime(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("    target_cc_path=\"$");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("\"\n");
    script.push_str("    target_cxx_path=\"$");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("\"\n");
    script.push_str("    target_ar_path=\"$");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("\"\n");
    script.push_str("    target_ranlib_path=\"$");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("\"\n");
    script.push_str("    target_objcopy_program=$target_tool_prefix-objcopy\n");
    script.push_str("    target_orig_cc_root=\"$target_wrapper_root\"\n");
    script.push_str("    if [ -f \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push('/');
    script.push_str(FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE);
    script.push_str("\" ]; then\n");
    script.push_str("      IFS= read -r target_orig_cc_root < \"$target_wrapper_root/");
    script.push_str(FIRST_STAGE_TARGET_NIX_SUPPORT_DIR);
    script.push('/');
    script.push_str(FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE);
    script.push_str("\" || true\n");
    script.push_str("    fi\n");
    script.push_str("    target_gcc_crt_machine=$target_cc_machine\n");
    script.push_str("    target_gcc_version=$(\"$target_cc_path\" -dumpfullversion 2>/dev/null || \"$target_cc_path\" -dumpversion 2>/dev/null || true)\n");
    script.push_str(
        "    target_gcc_crt_dir=\"$target_orig_cc_root/lib/gcc/$target_gcc_crt_machine/$target_gcc_version\"\n",
    );
    script.push_str("    if [ ! -f \"$target_gcc_crt_dir/crtbeginS.o\" ]; then for candidate_dir in \"$target_orig_cc_root\"/lib/gcc/\"$target_gcc_crt_machine\"/*; do if [ -f \"$candidate_dir/crtbeginS.o\" ]; then target_gcc_crt_dir=\"$candidate_dir\"; break; fi; done; fi\n");
    script.push_str(&format!(
        "    if [ ! -f \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/crt1.o\" ] || [ ! -f \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}\" ] || [ ! -f \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/rcrt1.o\" ] || [ ! -f \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/crti.o\" ] || [ ! -f \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/crtn.o\" ] || [ ! -f \"$target_gcc_crt_dir/crtbeginS.o\" ] || [ ! -f \"$target_gcc_crt_dir/crtendS.o\" ] || [ ! -f \"$target_gcc_crt_dir/libgcc.a\" ] || [ ! -x \"$target_tool_dir/$target_objcopy_program\" ]; then printf '%s\\n' 'Rust bootstrap target toolchain does not expose musl/gcc CRT, unwinder, and objcopy objects' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!("    target_alias_dir=\"$MANTLE_BUILD_DIR/{RUSTC_SOURCE_TARGET_LINKER_ALIAS_DIR}\"\n"));
    script
        .push_str(&format!("    target_runtime_dir=\"$MANTLE_BUILD_DIR/{RUSTC_SOURCE_TARGET_LINKER_RUNTIME_DIR}\"\n"));
    script.push_str("    mkdir -p \"$target_alias_dir\" \"$target_runtime_dir\"\n");
    script.push_str(&format!(
        "    for crt_name in crt1.o {FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT} rcrt1.o crti.o crtn.o; do rm -f \"$target_runtime_dir/$crt_name\"; cp \"${RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}/lib/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n"
    ));
    script.push_str("    for crt_name in crtbeginS.o crtendS.o; do rm -f \"$target_runtime_dir/$crt_name\"; cp \"$target_gcc_crt_dir/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n");
    script.push_str("    target_crtbegin_no_frame_init=\"$target_runtime_dir/crtbeginS.o.no-frame-init\"\n");
    script.push_str("    rm -f \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("    \"$target_tool_dir/$target_objcopy_program\" --remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array \"$target_runtime_dir/crtbeginS.o\" \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("    rm -f \"$target_runtime_dir/crtbeginS.o\"\n");
    script.push_str("    cp \"$target_crtbegin_no_frame_init\" \"$target_runtime_dir/crtbeginS.o\"\n");
    script.push_str("    rm -f \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("    target_unwind_archive=\"$target_gcc_crt_dir/libgcc_eh.a\"\n");
    script.push_str(
        "    if [ ! -f \"$target_unwind_archive\" ]; then target_unwind_archive=\"$target_gcc_crt_dir/libgcc.a\"; fi\n",
    );
    script.push_str("    rm -f \"$target_runtime_dir/libunwind.a\"; cp \"$target_unwind_archive\" \"$target_runtime_dir/libunwind.a\"\n");
    script.push_str(&format!(
        "    if [ ! -f \"${{{RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}}}/lib/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\" ]; then printf '%s\\n' 'Rust bootstrap target musl libc.so missing for dynamic compiler host links' >&2; exit {RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "    rm -f \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\"; cp \"${{{RUSTC_SOURCE_TARGET_MUSL_ROOT_VAR}}}/lib/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\"\n"
    ));
    script.push_str("    rm -f \"$target_runtime_dir/libgcc.a\"; cp \"$target_gcc_crt_dir/libgcc.a\" \"$target_runtime_dir/libgcc.a\"\n");
    script.push_str(&format!(
        "    rm -f \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT}\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT}\"\n"
    ));
    push_rustc_source_target_libgcc_shared_copy(script);
}

fn push_rustc_source_target_cc_wrapper(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("    printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'set -eu' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' \"target_cc_path=\\\"\\${MANTLE_TARGET_CC_PATH:-$target_cc_path}\\\"\" >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "    printf '%s\\n' \"target_runtime_dir=\\\"$target_runtime_dir\\\"\" >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("    printf '%s\\n' 'output_path=' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'previous_arg=' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  if [ \"$previous_arg\" = \"-o\" ]; then output_path=\"$arg\"; previous_arg=; continue; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  if [ \"$arg\" = \"-o\" ]; then previous_arg=-o; continue; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'dynamic_rustc_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'case \"$output_path\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  */stage1/bin/rustc|stage1/bin/rustc|*/stage2/bin/rustc|stage2/bin/rustc) dynamic_rustc_link=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'shared_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '    -shared|-dynamiclib) shared_link=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'mapped_args_set=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '    rcrt1.o|*/rcrt1.o) mapped_arg=\"$target_runtime_dir/crt1.o\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str(&format!(
        "    printf '%s\\n' '    crt1.o|*/crt1.o|{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}|*/{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg=\"$target_runtime_dir/${{arg##*/}}\" ;;' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str("    printf '%s\\n' '    -lgcc_s) continue ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "    printf '%s\\n' '    -static) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"$arg\"; fi ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str(
        "    printf '%s\\n' '    -static-pie) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"-static\"; fi ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("    printf '%s\\n' '    *) mapped_arg=\"$arg\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  if [ \"$mapped_args_set\" = false ]; then set -- \"$mapped_arg\"; mapped_args_set=true; else set -- \"$@\" \"$mapped_arg\"; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "    printf '%s\\n' 'if [ \"$mapped_args_set\" = false ]; then set --; fi' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("    printf '%s\\n' 'link_command=true' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'static_support_link=true' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'dynamic_executable_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'shared_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '    -c|-S|-E) link_command=false ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' '    -pie) static_support_link=false; dynamic_executable_link=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "    printf '%s\\n' '    -shared|-dynamiclib) static_support_link=false; shared_link=true ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("    printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("    printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(&format!(
        "    printf '%s\\n' 'if [ \"$link_command\" = true ] && [ \"$shared_link\" = true ]; then set -- \"$@\" -Wl,-Bstatic -lunwind -lgcc -Wl,-Bdynamic; elif [ \"$link_command\" = true ] && {{ [ \"$dynamic_rustc_link\" = true ] || [ \"$dynamic_executable_link\" = true ]; }}; then set -- \"$@\" -no-pie -Wl,-Bdynamic \"-Wl,-dynamic-linker,$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\" -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; elif [ \"$link_command\" = true ] && [ \"$static_support_link\" = true ]; then set -- \"$@\" -static -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; fi' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str(
        "    printf '%s\\n' 'if [ -n \"${MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG:-}\" ]; then set -- \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" \"$@\"; fi' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str(&format!(
        "    printf '%s\\n' 'exec \"$target_cc_path\" -D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -B\"$target_runtime_dir/\" -L\"$target_runtime_dir\" \"$@\"' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str("    chmod +x \"$target_alias_dir/cc\"\n");
}

fn push_rustc_source_target_companion_wrappers(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("    printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/c++\"\n");
    script.push_str("    printf '%s\\n' 'set -eu' >> \"$target_alias_dir/c++\"\n");
    script.push_str(
        "    printf '%s\\n' \"MANTLE_TARGET_CC_PATH=\\\"$target_cxx_path\\\"\" >> \"$target_alias_dir/c++\"\n",
    );
    script.push_str("    printf '%s\\n' 'export MANTLE_TARGET_CC_PATH' >> \"$target_alias_dir/c++\"\n");
    script.push_str(
        "    printf '%s\\n' \"exec \\\"$target_alias_dir/cc\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/c++\"\n",
    );
    script.push_str("    chmod +x \"$target_alias_dir/c++\"\n");
    script.push_str("    for target_tool in ar ranlib; do case \"$target_tool\" in ar) target_program=\"$target_ar_path\" ;; ranlib) target_program=\"$target_ranlib_path\" ;; esac; printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/$target_tool\"; printf '%s\\n' \"exec \\\"$target_program\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/$target_tool\"; chmod +x \"$target_alias_dir/$target_tool\"; done\n");
    push_first_stage_target_musl_libatomic_shim(script, full_source_bound);
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_CC_VAR);
    script.push_str("=\"$target_alias_dir/cc\"\n");
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_CXX_VAR);
    script.push_str("=\"$target_alias_dir/c++\"\n");
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_AR_VAR);
    script.push_str("=\"$target_alias_dir/ar\"\n");
    script.push_str("    ");
    script.push_str(RUSTC_SOURCE_TARGET_RANLIB_VAR);
    script.push_str("=\"$target_alias_dir/ranlib\"\n");
}

fn push_rustc_source_build_target_tool_config(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
    script.push_str("crt-static = false\n");
    script.push_str("MANTLE_RUST_TARGET_CONFIG\n");
    script.push_str("fi\n");
}

fn push_rustc_source_bootstrap_workspace_isolation(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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

struct RustBuildGoals<'a>(&'a str);

impl<'a> From<&'a str> for RustBuildGoals<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

struct RustBootstrapToolsJson<'a>(&'a str);

impl<'a> From<&'a str> for RustBootstrapToolsJson<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

fn push_rustc_source_build_script_launch<'a>(
    script: &mut String,
    goals: impl Into<RustBuildGoals<'a>>,
    bootstrap_tools_json: impl Into<RustBootstrapToolsJson<'a>>,
) {
    let goals = goals.into().0;
    let bootstrap_tools_json = bootstrap_tools_json.into().0;
    script.push_str(
        "MANTLE_BOOTSTRAP_PROVIDER=\"$BOOTSTRAP_PROVIDER\" MANTLE_STAGE_OUTPUT=\"$STAGE_OUTPUT\" MANTLE_BUILD_DIR=\"$BUILD_DIR\" MANTLE_RUST_SOURCE=\"$RUST_SOURCE\" MANTLE_RUST_BUILD_GOALS=",
    );
    script.push_str(&shell_quote(goals));
    script.push_str(" MANTLE_RUST_BOOTSTRAP_TOOLS=");
    script.push_str(&shell_quote(bootstrap_tools_json));
    script.push_str(
        " MANTLE_RUST_VERSION=\"$RUSTC_VERSION\" MANTLE_HOST_TRIPLE=\"$HOST_TRIPLE\" MANTLE_TARGET_TRIPLE=\"$TARGET_TRIPLE\" \"$SHELL_PROGRAM\" \"$RESOLVED_BUILD_SCRIPT\"\n",
    );
}

fn push_rustc_stage1_script_build_commands(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$STAGE_OUTPUT\"\n");
    push_rustc_source_build_script_launch(script, RUSTC_STAGE1_XPY_GOALS, RUSTC_STAGE1_BOOTSTRAP_TOOLS_JSON);
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
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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

fn push_rustc_final_script_input_checks(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
        patch_plan,
        full_source_context,
    )?;
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
    if full_source_context.is_some() {
        push_required_shell_path_file_check(
            script,
            "\"$SHELL_PROGRAM\"",
            "receipt-bound shell is required for rustc final",
            RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
        );
    } else {
        push_required_shell_command_check(script, "SHELL_PROGRAM", "sh is required for rustc final");
    }
    Ok(())
}

fn push_rustc_final_script_build_commands(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$STAGE_OUTPUT\"\n");
    push_rustc_source_build_script_launch(script, RUSTC_FINAL_XPY_GOALS, RUSTC_FINAL_BOOTSTRAP_TOOLS_JSON);
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

fn full_source_context_manifest(context: Option<&FullSourceRustExecutionContext>) -> serde_json::Value {
    match context {
        Some(context) => serde_json::json!({
            "ambient_tool_discovery": false,
            "source_policy": "authenticated-offline-only",
            "native_provider_path": context.native_provider_dir.display().to_string(),
            "rust_source_archive_dir": context.rust_source_archive_dir.display().to_string(),
            "host_tool_manifest_digest_blake3": context.host_tools.manifest_digest_blake3,
            "host_tools": context.host_tools.manifest.tools,
            "host_support_inputs": context.host_tools.manifest.support_inputs,
        }),
        None => serde_json::Value::Null,
    }
}

fn first_stage_boundary_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let patch_plan = first_stage_patch_plan(boundary)?;
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert!(!boundary.source_ids.is_empty());
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
        "patch_plan": patch_plan,
        "sources": &boundary.sources,
        "full_source_execution": full_source_context_manifest(boundary.full_source_context.as_ref()),
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
    acquire_stage_sources(
        &boundary.stage_id,
        &boundary.archive_dir,
        &boundary.source_dir,
        &boundary.sources,
        boundary.full_source_context.as_ref(),
        verbose,
    )
}

fn acquire_rustc_stage1_sources(
    boundary: &RustSourceProviderRustcStage1Boundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(
        &boundary.stage_id,
        &boundary.archive_dir,
        &boundary.source_dir,
        &boundary.sources,
        boundary.full_source_context.as_ref(),
        verbose,
    )
}

fn acquire_rustc_final_sources(
    boundary: &RustSourceProviderRustcFinalBoundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(
        &boundary.stage_id,
        &boundary.archive_dir,
        &boundary.source_dir,
        &boundary.sources,
        boundary.full_source_context.as_ref(),
        verbose,
    )
}

fn acquire_stage_sources(
    stage_id: &str,
    archive_dir: &Path,
    source_dir: &Path,
    sources: &[RustSourceProviderBootstrapSource],
    full_source_context: Option<&FullSourceRustExecutionContext>,
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
        acquired.push(acquire_stage_source(archive_dir, source_dir, source, full_source_context, verbose)?);
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
    full_source_context: Option<&FullSourceRustExecutionContext>,
    verbose: bool,
) -> Result<RustSourceProviderFirstStageSourceAcquisition, RustSourceProviderError> {
    let source_component = safe_source_component(&source.id)?;
    ensure_supported_first_stage_archive(&source.url)?;
    let archive_path = archive_dir.join(format!("{source_component}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}"));
    let extracted_path = source_dir.join(&source_component);
    let source_bytes = fetch_first_stage_source_bytes(source, full_source_context)?;
    let archive_sha256_hex = sha256_hex(&source_bytes.bytes);
    if archive_sha256_hex != source.sha256_hex {
        return Err(RustSourceProviderError::Digest(format!(
            "source '{}' digest mismatch: expected {}, got {archive_sha256_hex}",
            source.id, source.sha256_hex
        )));
    }
    fs::write(&archive_path, &source_bytes.bytes)
        .map_err(|err| RustSourceProviderError::Fetch(format!("write {}: {err}", archive_path.display())))?;
    prepare_empty_directory(&extracted_path)?;
    let archive_entry_count = extract_first_stage_tar_gz(&archive_path, &extracted_path)?;
    let extracted_digest_blake3 = content_digest_blake3(&extracted_path)?;
    debug_assert_eq!(archive_sha256_hex, source.sha256_hex);
    debug_assert_eq!(extracted_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
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
        source_policy: source_bytes.source_policy.to_string(),
        source_input_path: source_bytes.source_input_path,
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
    debug_assert_eq!(sources.len(), boundary.sources.len());
    debug_assert!(!boundary.stage_id.is_empty());
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
    debug_assert_eq!(sources.len(), boundary.sources.len());
    debug_assert!(!boundary.stage_id.is_empty());
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

fn should_retry_generated_script_launch(error: &std::io::Error, attempt: u32) -> bool {
    debug_assert!(attempt > 0);
    debug_assert!(attempt <= GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS);
    if attempt >= GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS {
        return false;
    }
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::ETXTBSY)
    }
    #[cfg(not(unix))]
    {
        let _ = error;
        false
    }
}

fn should_retry_smoke_process_output(output: &Output, attempt: u32) -> bool {
    debug_assert!(attempt > 0);
    debug_assert!(attempt <= GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS);
    if attempt >= GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS {
        return false;
    }
    let shell_rejected_exec = output.status.code() == Some(SHELL_COMMAND_NOT_EXECUTABLE_EXIT_CODE);
    let text_file_busy = String::from_utf8_lossy(&output.stderr).contains(TEXT_FILE_BUSY_MESSAGE);
    shell_rejected_exec && text_file_busy
}

fn run_smoke_rustc_with_bounded_launch_retry(rustc_path: &Path, rustc_args: &[OsString]) -> std::io::Result<Output> {
    debug_assert!(!rustc_path.as_os_str().is_empty());
    debug_assert!(!rustc_args.is_empty());
    for attempt in 1..=GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS {
        let result = Command::new(rustc_path).env_clear().args(rustc_args).output();
        match result {
            Ok(output) if should_retry_smoke_process_output(&output, attempt) => {
                std::thread::sleep(Duration::from_millis(GENERATED_SCRIPT_LAUNCH_RETRY_DELAY_MS));
            }
            Ok(output) => return Ok(output),
            Err(error) if should_retry_generated_script_launch(&error, attempt) => {
                std::thread::sleep(Duration::from_millis(GENERATED_SCRIPT_LAUNCH_RETRY_DELAY_MS));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("the final bounded smoke launch attempt must return an output or error")
}

fn generated_script_command(script_path: &Path, interpreter: Option<&Path>) -> Command {
    match interpreter {
        Some(interpreter) => {
            let mut command = Command::new(interpreter);
            command.arg(script_path);
            command
        }
        None => Command::new(script_path),
    }
}

fn run_generated_script_with_log(
    script_path: &Path,
    log_path: &Path,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<ExitStatus, RustSourceProviderError> {
    if script_path == log_path {
        return Err(RustSourceProviderError::Build("generated script path and build log path must differ".to_string()));
    }
    debug_assert_ne!(script_path, log_path);
    let interpreter = full_source_context.map(|context| full_source_busybox_applet_path(context, "sh")).transpose()?;
    let log = File::create(log_path)
        .map_err(|err| RustSourceProviderError::Build(format!("create {}: {err}", log_path.display())))?;
    for attempt in 1..=GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS {
        let stdout = log
            .try_clone()
            .map_err(|err| RustSourceProviderError::Build(format!("clone {}: {err}", log_path.display())))?;
        let stderr = log
            .try_clone()
            .map_err(|err| RustSourceProviderError::Build(format!("clone {}: {err}", log_path.display())))?;
        let mut command = generated_script_command(script_path, interpreter.as_deref());
        if full_source_context.is_some() {
            command.env_clear();
        }
        match command.stdout(stdout).stderr(stderr).status() {
            Ok(status) => return Ok(status),
            Err(error) if should_retry_generated_script_launch(&error, attempt) => {
                std::thread::sleep(Duration::from_millis(GENERATED_SCRIPT_LAUNCH_RETRY_DELAY_MS));
            }
            Err(error) => {
                return Err(RustSourceProviderError::Build(format!("launch {}: {error}", script_path.display())));
            }
        }
    }
    Err(RustSourceProviderError::Build(format!(
        "launch {} exhausted bounded retry attempts",
        script_path.display()
    )))
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
    let status = run_generated_script_with_log(
        &boundary.script_path,
        &boundary.build_log_path,
        boundary.full_source_context.as_ref(),
    )?;
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
    debug_assert!(boundary.script_path.is_file());
    debug_assert!(boundary.sources_manifest_path.is_file());
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
    let status = run_generated_script_with_log(
        &boundary.script_path,
        &boundary.build_log_path,
        boundary.full_source_context.as_ref(),
    )?;
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
    debug_assert!(boundary.script_path.is_file());
    debug_assert!(boundary.sources_manifest_path.is_file());
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
    debug_assert!(rustc_path.is_file());
    debug_assert!(target_rustlib_path.exists());
    Ok(RustSourceProviderRustcStage1Build {
        stage_id: boundary.stage_id.clone(),
        plan_digest_blake3: file_digest_blake3(&boundary.plan_path)?,
        script_path: boundary.script_path.clone(),
        script_digest_blake3: file_digest_blake3(&boundary.script_path)?,
        parallel_job_count: RUST_BOOTSTRAP_JOB_COUNT,
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
    debug_assert!(rustc_path.is_file());
    debug_assert!(target_rustlib_path.exists());
    Ok(RustSourceProviderRustcFinalBuild {
        stage_id: boundary.stage_id.clone(),
        plan_digest_blake3: file_digest_blake3(&boundary.plan_path)?,
        script_path: boundary.script_path.clone(),
        script_digest_blake3: file_digest_blake3(&boundary.script_path)?,
        parallel_job_count: RUST_BOOTSTRAP_JOB_COUNT,
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
    let patch_plan = rustc_stage1_patch_plan(boundary)?;
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
        "patch_plan": patch_plan,
        "build": build,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.build_manifest_path, &manifest, "rustc stage1 build manifest")
}

fn write_rustc_final_build_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    build: &RustSourceProviderRustcFinalBuild,
) -> Result<(), RustSourceProviderError> {
    let patch_plan = rustc_final_patch_plan(boundary)?;
    debug_assert_eq!(boundary.stage_id, build.stage_id);
    debug_assert!(!boundary.build_manifest_path.as_os_str().is_empty());
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
        "patch_plan": patch_plan,
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
    debug_assert_eq!(sources.len(), boundary.sources.len());
    debug_assert_ne!(boundary.provider_candidate_dir, boundary.output_dir);
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.stage_output_dir, &boundary.provider_candidate_dir)?;
    wrap_rustc_stage_provider_dynamic_tools(
        &boundary.provider_candidate_dir,
        &boundary
            .build_dir
            .join(RUSTC_SOURCE_TARGET_LINKER_RUNTIME_DIR)
            .join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT),
        &boundary.host_triple,
        RUSTC_STAGE1_DYNAMIC_TOOL_SPECS,
    )?;
    let source_identities = rustc_stage1_provider_source_identities(boundary, sources)?;
    let artifact_source_id = rustc_stage1_artifact_source_id(boundary)?;
    let receipt_artifacts = rustc_stage1_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = rustc_stage1_provider_receipt(boundary, route, build, &source_identities, &receipt_artifacts)?;
    let receipt_path = boundary.provider_candidate_dir.join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "rustc stage1 provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = rustc_stage1_provider_metadata(RustProviderMetadataInput {
        route,
        sources: &source_identities,
        artifacts: RustProviderArtifactsInput {
            receipt_artifacts: &receipt_artifacts,
            receipt_digest_blake3: &receipt_digest_blake3,
            artifact_source_id: &artifact_source_id,
        },
    });
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

#[derive(Clone, Copy)]
enum RustProviderDynamicToolSysrootMode {
    Inject,
    LoaderOnly,
}

#[derive(Clone, Copy)]
struct RustProviderDynamicToolSpec {
    relative_path: &'static str,
    sysroot_mode: RustProviderDynamicToolSysrootMode,
}

const RUSTC_STAGE1_DYNAMIC_TOOL_SPECS: &[RustProviderDynamicToolSpec] = &[
    RustProviderDynamicToolSpec {
        relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
        sysroot_mode: RustProviderDynamicToolSysrootMode::Inject,
    },
    RustProviderDynamicToolSpec {
        relative_path: PROVIDER_CARGO_RELATIVE_PATH,
        sysroot_mode: RustProviderDynamicToolSysrootMode::LoaderOnly,
    },
];

const RUSTC_FINAL_DYNAMIC_TOOL_SPECS: &[RustProviderDynamicToolSpec] = &[
    RustProviderDynamicToolSpec {
        relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
        sysroot_mode: RustProviderDynamicToolSysrootMode::Inject,
    },
    RustProviderDynamicToolSpec {
        relative_path: PROVIDER_CARGO_RELATIVE_PATH,
        sysroot_mode: RustProviderDynamicToolSysrootMode::LoaderOnly,
    },
    RustProviderDynamicToolSpec {
        relative_path: PROVIDER_RUSTDOC_RELATIVE_PATH,
        sysroot_mode: RustProviderDynamicToolSysrootMode::Inject,
    },
];

fn wrap_rustc_stage_provider_dynamic_tools(
    candidate_dir: &Path,
    runtime_loader_path: &Path,
    host_triple: &str,
    specs: &[RustProviderDynamicToolSpec],
) -> Result<(), RustSourceProviderError> {
    if !runtime_loader_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "Rust bootstrap dynamic runtime loader is missing at {}",
            runtime_loader_path.display()
        )));
    }
    debug_assert!(runtime_loader_path.is_file());
    debug_assert!(!specs.is_empty());
    let runtime_dir = candidate_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_DIR);
    let provider_loader_path = candidate_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH);
    fs::create_dir_all(&runtime_dir).map_err(|err| {
        RustSourceProviderError::Copy(format!("create dynamic runtime dir {}: {err}", runtime_dir.display()))
    })?;
    fs::copy(runtime_loader_path, &provider_loader_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "copy Rust bootstrap dynamic loader {} -> {}: {err}",
            runtime_loader_path.display(),
            provider_loader_path.display()
        ))
    })?;
    make_executable(&provider_loader_path)?;
    copy_dynamic_runtime_shared_objects(runtime_loader_path, &runtime_dir, "Rust bootstrap")?;
    for spec in specs {
        wrap_rustc_stage_provider_dynamic_tool(candidate_dir, *spec, host_triple)?;
    }
    Ok(())
}

fn copy_dynamic_runtime_shared_objects(
    runtime_loader_path: &Path,
    runtime_dir: &Path,
    context: &str,
) -> Result<(), RustSourceProviderError> {
    assert!(runtime_loader_path.is_file(), "runtime loader must exist before runtime copy");
    assert!(runtime_dir.is_dir(), "runtime dir must exist before runtime copy");
    let source_dir = runtime_loader_path.parent().ok_or_else(|| {
        RustSourceProviderError::Copy(format!(
            "{context} dynamic runtime loader has no parent directory: {}",
            runtime_loader_path.display()
        ))
    })?;
    for object_name in REQUIRED_DYNAMIC_RUNTIME_SHARED_OBJECTS {
        copy_dynamic_runtime_shared_object(source_dir, runtime_dir, *object_name, context, true)?;
    }
    for object_name in OPTIONAL_DYNAMIC_RUNTIME_SHARED_OBJECTS {
        copy_dynamic_runtime_shared_object(source_dir, runtime_dir, *object_name, context, false)?;
    }
    Ok(())
}

struct DynamicRuntimeObjectName<'a>(&'a str);

impl<'a> From<&'a str> for DynamicRuntimeObjectName<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

struct DynamicRuntimeContext<'a>(&'a str);

impl<'a> From<&'a str> for DynamicRuntimeContext<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

fn copy_dynamic_runtime_shared_object<'a>(
    source_dir: &Path,
    runtime_dir: &Path,
    object_name: impl Into<DynamicRuntimeObjectName<'a>>,
    context: impl Into<DynamicRuntimeContext<'a>>,
    required: bool,
) -> Result<(), RustSourceProviderError> {
    let object_name = object_name.into().0;
    let context = context.into().0;
    assert!(!object_name.trim().is_empty(), "runtime object name must not be empty");
    assert!(runtime_dir.is_dir(), "runtime dir must exist before object copy");
    let source_path = source_dir.join(object_name);
    if !source_path.is_file() {
        if required {
            return Err(RustSourceProviderError::Copy(format!(
                "{context} dynamic runtime shared object is missing at {}",
                source_path.display()
            )));
        }
        return Ok(());
    }
    let destination_path = runtime_dir.join(object_name);
    if fs::symlink_metadata(&destination_path).is_ok() {
        fs::remove_file(&destination_path).map_err(|err| {
            RustSourceProviderError::Copy(format!(
                "remove stale {context} dynamic runtime shared object {}: {err}",
                destination_path.display()
            ))
        })?;
    }
    fs::copy(&source_path, &destination_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "copy {context} dynamic runtime shared object {} -> {}: {err}",
            source_path.display(),
            destination_path.display()
        ))
    })?;
    Ok(())
}

fn wrap_rustc_stage_provider_dynamic_tool(
    candidate_dir: &Path,
    spec: RustProviderDynamicToolSpec,
    host_triple: &str,
) -> Result<(), RustSourceProviderError> {
    let wrapper_path = candidate_dir.join(spec.relative_path);
    debug_assert!(!spec.relative_path.is_empty());
    debug_assert!(!host_triple.is_empty());
    if !wrapper_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "Rust bootstrap dynamic tool {} is missing before wrapper installation",
            wrapper_path.display()
        )));
    }
    let binary_relative_path = format!("{}{}", spec.relative_path, PROVIDER_DYNAMIC_TOOL_BINARY_SUFFIX);
    let binary_path = candidate_dir.join(&binary_relative_path);
    if binary_path.exists() {
        fs::remove_file(&binary_path).map_err(|err| {
            RustSourceProviderError::Copy(format!("remove stale dynamic tool {}: {err}", binary_path.display()))
        })?;
    }
    fs::rename(&wrapper_path, &binary_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "rename dynamic Rust tool {} -> {}: {err}",
            wrapper_path.display(),
            binary_path.display()
        ))
    })?;
    make_executable(&binary_path)?;
    let binary_name = dynamic_tool_binary_name(&binary_relative_path)?;
    fs::write(
        &wrapper_path,
        provider_local_dynamic_rust_tool_wrapper(binary_name.as_str(), host_triple, spec.sysroot_mode),
    )
    .map_err(|err| {
        RustSourceProviderError::Copy(format!("write dynamic tool wrapper {}: {err}", wrapper_path.display()))
    })?;
    make_executable(&wrapper_path)
}

fn dynamic_tool_binary_name(binary_relative_path: &str) -> Result<String, RustSourceProviderError> {
    Path::new(binary_relative_path)
        .file_name()
        .and_then(OsStr::to_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            RustSourceProviderError::Copy(format!(
                "dynamic Rust tool relative path lacks a UTF-8 file name: {binary_relative_path}"
            ))
        })
}

struct DynamicToolBinaryName<'a>(&'a str);

impl<'a> From<&'a str> for DynamicToolBinaryName<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

struct DynamicToolHostTriple<'a>(&'a str);

impl<'a> From<&'a str> for DynamicToolHostTriple<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

fn provider_local_dynamic_rust_tool_wrapper<'a>(
    tool_binary_name: impl Into<DynamicToolBinaryName<'a>>,
    host_triple: impl Into<DynamicToolHostTriple<'a>>,
    sysroot_mode: RustProviderDynamicToolSysrootMode,
) -> String {
    let tool_binary_name = tool_binary_name.into().0;
    let host_triple = host_triple.into().0;
    debug_assert!(!tool_binary_name.is_empty());
    debug_assert!(!host_triple.is_empty());
    let sysroot_injection = match sysroot_mode {
        RustProviderDynamicToolSysrootMode::Inject => {
            "has_sysroot=false\n\
for arg in \"$@\"; do\n\
  if [ \"$arg\" = \"--sysroot\" ]; then has_sysroot=true; break; fi\n\
  case \"$arg\" in --sysroot=*) has_sysroot=true; break ;; esac\n\
done\n\
if [ \"$has_sysroot\" = false ]; then set -- --sysroot \"$root_dir\" \"$@\"; fi\n"
        }
        RustProviderDynamicToolSysrootMode::LoaderOnly => "",
    };
    format!(
        "#!/bin/sh\n\
set -eu\n\
self_dir=${{0%/*}}\n\
case \"$self_dir\" in\n\
  /*) ;;\n\
  *) self_dir=\"$(pwd -P)/$self_dir\" ;;\n\
esac\n\
root_dir=$(cd \"$self_dir/..\" && pwd -P)\n\
runtime_dir=\"$root_dir/{FIRST_STAGE_PROC_MACRO_RUNTIME_DIR}\"\n\
loader=\"$root_dir/{FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH}\"\n\
ld_path=\"$runtime_dir:$root_dir/lib:$root_dir/lib/rustlib/{host_triple}/lib\"\n\
if [ \"${{LD_LIBRARY_PATH+x}}\" = x ] && [ -n \"${{LD_LIBRARY_PATH}}\" ]; then\n\
  ld_path=\"$ld_path:$LD_LIBRARY_PATH\"\n\
fi\n\
if [ ! -x \"$loader\" ]; then\n\
  printf '%s\\n' \"provider dynamic runtime loader missing: $loader\" >&2\n\
  exit {PROVIDER_DYNAMIC_TOOL_RUNTIME_MISSING_EXIT_CODE}\n\
fi\n\
{sysroot_injection}\
LD_LIBRARY_PATH=\"$ld_path\" exec \"$loader\" \"$self_dir/{tool_binary_name}\" \"$@\"\n"
    )
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
    debug_assert!(boundary.sources.len() <= DIRECTORY_DIGEST_MAX_ENTRIES);
    debug_assert!(!boundary.stage_id.is_empty());
    let mut identities = Vec::with_capacity(boundary.sources.len().saturating_add(1));
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
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Rustc,
            name: "rustc",
            relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Cargo,
            name: "cargo",
            relative_path: PROVIDER_CARGO_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::HostRustlib,
            name: "host-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.host_triple),
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::TargetRustlib,
            name: "target-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.target_triple),
        })?,
    ])
}

fn rustc_stage1_provider_receipt(
    boundary: &RustSourceProviderRustcStage1Boundary,
    route: &LoadedRustSourceProviderRoute,
    build: &RustSourceProviderRustcStage1Build,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> Result<RustSourceProviderBuildReceipt, RustSourceProviderError> {
    let patch_plan = rustc_stage1_patch_plan(boundary)?;
    Ok(RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: rustc_stage1_provider_receipt_steps(boundary, build, &patch_plan),
    })
}

fn rustc_stage1_provider_receipt_steps(
    boundary: &RustSourceProviderRustcStage1Boundary,
    build: &RustSourceProviderRustcStage1Build,
    patch_plan: &RustBootstrapPatchPlan,
) -> Vec<RustProviderReceiptStep> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert_eq!(build.stage_id, boundary.stage_id);
    debug_assert!(!patch_plan.operations.is_empty());
    let mut steps = vec![
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
    ];
    steps.push(stage_construction_identity_receipt_step(
        &build.plan_digest_blake3,
        &build.script_digest_blake3,
        build.parallel_job_count,
        boundary.full_source_context.as_ref(),
    ));
    steps.push(rust_bootstrap_patch_plan_receipt_step(patch_plan));
    steps
}

#[derive(Clone, Copy)]
struct RustProviderArtifactsInput<'a> {
    receipt_artifacts: &'a [RustProviderReceiptArtifact],
    receipt_digest_blake3: &'a str,
    artifact_source_id: &'a str,
}

#[derive(Clone, Copy)]
struct RustProviderMetadataInput<'a> {
    route: &'a LoadedRustSourceProviderRoute,
    sources: &'a [RustProviderSourceIdentity],
    artifacts: RustProviderArtifactsInput<'a>,
}

fn rustc_stage1_provider_metadata(input: RustProviderMetadataInput<'_>) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: input.route.plan.host_triple.clone(),
        target_triple: input.route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: RUSTC_STAGE1_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: input.sources.to_vec(),
        build_receipts: vec![rustc_stage1_provider_receipt_identity(
            input.artifacts.receipt_digest_blake3,
        )],
        artifacts: rustc_stage1_provider_artifacts(input.artifacts),
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

fn rustc_stage1_provider_artifacts(input: RustProviderArtifactsInput<'_>) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = input
        .receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: input.artifact_source_id.to_string(),
            build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: RUSTC_STAGE1_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: input.receipt_digest_blake3.to_string(),
        source_id: input.artifact_source_id.to_string(),
        build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
    });
    debug_assert_eq!(artifacts.len(), input.receipt_artifacts.len().saturating_add(1));
    debug_assert!(!artifacts.is_empty());
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
    debug_assert_eq!(sources.len(), boundary.sources.len());
    debug_assert_ne!(boundary.provider_candidate_dir, boundary.output_dir);
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.stage_output_dir, &boundary.provider_candidate_dir)?;
    wrap_rustc_stage_provider_dynamic_tools(
        &boundary.provider_candidate_dir,
        &boundary
            .build_dir
            .join(RUSTC_SOURCE_TARGET_LINKER_RUNTIME_DIR)
            .join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT),
        &boundary.host_triple,
        RUSTC_FINAL_DYNAMIC_TOOL_SPECS,
    )?;
    let source_identities = rustc_final_provider_source_identities(boundary, sources)?;
    let artifact_source_id = rustc_final_artifact_source_id(boundary)?;
    let receipt_artifacts = rustc_final_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = rustc_final_provider_receipt(boundary, route, build, &source_identities, &receipt_artifacts)?;
    let receipt_path = boundary.provider_candidate_dir.join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "rustc final provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = rustc_final_provider_metadata(RustProviderMetadataInput {
        route,
        sources: &source_identities,
        artifacts: RustProviderArtifactsInput {
            receipt_artifacts: &receipt_artifacts,
            receipt_digest_blake3: &receipt_digest_blake3,
            artifact_source_id: &artifact_source_id,
        },
    });
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
    debug_assert!(boundary.sources.len() <= DIRECTORY_DIGEST_MAX_ENTRIES);
    debug_assert!(!boundary.stage_id.is_empty());
    let mut identities = Vec::with_capacity(boundary.sources.len().saturating_add(1));
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
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Rustc,
            name: "rustc",
            relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Cargo,
            name: "cargo",
            relative_path: PROVIDER_CARGO_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Rustdoc,
            name: "rustdoc",
            relative_path: PROVIDER_RUSTDOC_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::HostRustlib,
            name: "host-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.host_triple),
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::TargetRustlib,
            name: "target-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.target_triple),
        })?,
    ])
}

fn rustc_final_provider_receipt(
    boundary: &RustSourceProviderRustcFinalBoundary,
    route: &LoadedRustSourceProviderRoute,
    build: &RustSourceProviderRustcFinalBuild,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> Result<RustSourceProviderBuildReceipt, RustSourceProviderError> {
    let patch_plan = rustc_final_patch_plan(boundary)?;
    Ok(RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: rustc_final_provider_receipt_steps(boundary, build, &patch_plan),
    })
}

fn rustc_final_provider_receipt_steps(
    boundary: &RustSourceProviderRustcFinalBoundary,
    build: &RustSourceProviderRustcFinalBuild,
    patch_plan: &RustBootstrapPatchPlan,
) -> Vec<RustProviderReceiptStep> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert_eq!(build.stage_id, boundary.stage_id);
    debug_assert!(!patch_plan.operations.is_empty());
    let mut steps = vec![
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
    ];
    steps.push(stage_construction_identity_receipt_step(
        &build.plan_digest_blake3,
        &build.script_digest_blake3,
        build.parallel_job_count,
        boundary.full_source_context.as_ref(),
    ));
    steps.push(rust_bootstrap_patch_plan_receipt_step(patch_plan));
    steps
}

fn rustc_final_provider_metadata(input: RustProviderMetadataInput<'_>) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: input.route.plan.host_triple.clone(),
        target_triple: input.route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: RUSTC_FINAL_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: input.sources.to_vec(),
        build_receipts: vec![rustc_final_provider_receipt_identity(
            input.artifacts.receipt_digest_blake3,
        )],
        artifacts: rustc_final_provider_artifacts(input.artifacts),
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

fn rustc_final_provider_artifacts(input: RustProviderArtifactsInput<'_>) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = input
        .receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: input.artifact_source_id.to_string(),
            build_receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: RUSTC_FINAL_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: input.receipt_digest_blake3.to_string(),
        source_id: input.artifact_source_id.to_string(),
        build_receipt_id: RUSTC_FINAL_PROVIDER_RECEIPT_ID.to_string(),
    });
    debug_assert_eq!(artifacts.len(), input.receipt_artifacts.len().saturating_add(1));
    debug_assert!(!artifacts.is_empty());
    artifacts
}

fn write_rustc_final_provider_candidate_manifest(
    boundary: &RustSourceProviderRustcFinalBoundary,
    candidate: &RustSourceProviderRustcFinalProviderCandidate,
    final_output_written: bool,
) -> Result<(), RustSourceProviderError> {
    debug_assert_eq!(boundary.stage_id, candidate.stage_id);
    debug_assert!(!candidate.metadata_digest_blake3.is_empty());
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
    let status = run_generated_script_with_log(
        &boundary.script_path,
        &boundary.build_log_path,
        boundary.full_source_context.as_ref(),
    )?;
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
    debug_assert!(mrustc_path.is_file());
    debug_assert!(target_prefix_rustlib_path.exists());
    Ok(RustSourceProviderFirstStageBuild {
        stage_id: boundary.stage_id.clone(),
        plan_digest_blake3: file_digest_blake3(&boundary.plan_path)?,
        script_path: boundary.script_path.clone(),
        script_digest_blake3: file_digest_blake3(&boundary.script_path)?,
        parallel_job_count: RUST_BOOTSTRAP_JOB_COUNT,
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
    let patch_plan = first_stage_patch_plan(boundary)?;
    debug_assert_eq!(boundary.stage_id, build.stage_id);
    debug_assert!(!boundary.build_manifest_path.as_os_str().is_empty());
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "patch_plan": patch_plan,
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
    debug_assert_eq!(sources.len(), boundary.sources.len());
    debug_assert_ne!(boundary.provider_candidate_dir, boundary.output_dir);
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.prefix_path, &boundary.provider_candidate_dir)?;
    install_first_stage_proc_macro_capable_rustc(boundary, build)?;
    normalize_first_stage_provider_wrappers(&boundary.provider_candidate_dir, &boundary.host_triple)?;
    let source_identities = first_stage_provider_source_identities(boundary, sources)?;
    let build_sources = first_stage_build_sources(boundary)?;
    let artifact_source_id = build_sources.rust_id;
    let receipt_artifacts = first_stage_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = first_stage_provider_receipt(boundary, route, build, &source_identities, &receipt_artifacts)?;
    let receipt_path = boundary.provider_candidate_dir.join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "first-stage provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = first_stage_provider_metadata(RustProviderMetadataInput {
        route,
        sources: &source_identities,
        artifacts: RustProviderArtifactsInput {
            receipt_artifacts: &receipt_artifacts,
            receipt_digest_blake3: &receipt_digest_blake3,
            artifact_source_id: &artifact_source_id,
        },
    });
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

fn file_has_elf_magic(path: &Path) -> Result<bool, RustSourceProviderError> {
    let mut file = File::open(path)
        .map_err(|err| RustSourceProviderError::Read(format!("open {} for ELF detection: {err}", path.display())))?;
    let mut magic = [0_u8; ELF_MAGIC_LEN];
    let bytes_read = file
        .read(&mut magic)
        .map_err(|err| RustSourceProviderError::Read(format!("read {} ELF magic: {err}", path.display())))?;
    if bytes_read != ELF_MAGIC_LEN {
        return Ok(false);
    }
    Ok(magic == ELF_MAGIC)
}

fn install_first_stage_proc_macro_capable_rustc(
    boundary: &RustSourceProviderFirstStageBoundary,
    build: &RustSourceProviderFirstStageBuild,
) -> Result<(), RustSourceProviderError> {
    if boundary.host_triple != FIRST_STAGE_MUSL_TRIPLE {
        return Ok(());
    }
    assert_eq!(boundary.host_triple, FIRST_STAGE_MUSL_TRIPLE);
    assert!(!boundary.target_triple.trim().is_empty(), "target triple must not be empty");
    let candidate_dir = &boundary.provider_candidate_dir;
    let dynamic_rustc_path = &build.translated_rustc_path;
    let runtime_loader_path = boundary
        .build_dir
        .join(FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR)
        .join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT);
    install_first_stage_proc_macro_capable_rustc_from_paths(
        candidate_dir,
        dynamic_rustc_path,
        &runtime_loader_path,
        &boundary.host_triple,
    )
}

fn install_first_stage_proc_macro_capable_rustc_from_paths(
    candidate_dir: &Path,
    dynamic_rustc_path: &Path,
    runtime_loader_path: &Path,
    host_triple: &str,
) -> Result<(), RustSourceProviderError> {
    assert!(!candidate_dir.as_os_str().is_empty(), "candidate dir must not be empty");
    assert!(!host_triple.trim().is_empty(), "host triple must not be empty");
    if !dynamic_rustc_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "proc-macro-capable first-stage rustc is missing at {}",
            dynamic_rustc_path.display()
        )));
    }
    if !runtime_loader_path.is_file() {
        return Err(RustSourceProviderError::Copy(format!(
            "first-stage proc-macro runtime loader is missing at {}",
            runtime_loader_path.display()
        )));
    }
    if !file_has_elf_magic(dynamic_rustc_path)? {
        #[cfg(test)]
        {
            return Ok(());
        }
        #[cfg(not(test))]
        {
            return Err(RustSourceProviderError::Copy(format!(
                "proc-macro-capable first-stage rustc is not an ELF binary at {}",
                dynamic_rustc_path.display()
            )));
        }
    }
    let rustc_binary_path = candidate_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH);
    let runtime_dir = candidate_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_DIR);
    let runtime_loader_candidate_path = candidate_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH);
    fs::create_dir_all(&runtime_dir).map_err(|err| {
        RustSourceProviderError::Copy(format!("create proc-macro runtime dir {}: {err}", runtime_dir.display()))
    })?;
    fs::copy(runtime_loader_path, &runtime_loader_candidate_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "copy first-stage proc-macro loader {} -> {}: {err}",
            runtime_loader_path.display(),
            runtime_loader_candidate_path.display()
        ))
    })?;
    make_executable(&runtime_loader_candidate_path)?;
    copy_dynamic_runtime_shared_objects(runtime_loader_path, &runtime_dir, "first-stage proc-macro")?;
    fs::copy(dynamic_rustc_path, &rustc_binary_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "copy proc-macro-capable rustc {} -> {}: {err}",
            dynamic_rustc_path.display(),
            rustc_binary_path.display()
        ))
    })?;
    make_executable(&rustc_binary_path)?;
    let rustc_sysroot_binary_path = candidate_dir.join(PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_RELATIVE_PATH);
    prepare_provider_rustc_sysroot_binary_link(&rustc_binary_path, &rustc_sysroot_binary_path)?;
    let rustc_path = candidate_dir.join(PROVIDER_RUSTC_RELATIVE_PATH);
    fs::write(&rustc_path, provider_local_dynamic_mrustc_rustc_wrapper(host_triple)).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "write proc-macro-capable first-stage rustc wrapper {}: {err}",
            rustc_path.display()
        ))
    })?;
    make_executable(&rustc_path)
}

fn normalize_first_stage_provider_wrappers(
    candidate_dir: &Path,
    host_triple: &str,
) -> Result<(), RustSourceProviderError> {
    if host_triple.trim().is_empty() {
        return Err(RustSourceProviderError::Copy("first-stage provider wrapper host triple is empty".to_string()));
    }
    debug_assert!(!host_triple.trim().is_empty());
    debug_assert!(!candidate_dir.as_os_str().is_empty());
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
    let rustc_sysroot_binary_path = candidate_dir.join(PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_RELATIVE_PATH);
    prepare_provider_rustc_sysroot_binary_link(&rustc_binary_path, &rustc_sysroot_binary_path)?;
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

#[cfg(unix)]
fn prepare_provider_rustc_sysroot_binary_link(
    _binary_path: &Path,
    link_path: &Path,
) -> Result<(), RustSourceProviderError> {
    if link_path.exists() || fs::symlink_metadata(link_path).is_ok() {
        fs::remove_file(link_path).map_err(|err| {
            RustSourceProviderError::Copy(format!(
                "remove stale rustc sysroot binary link {}: {err}",
                link_path.display()
            ))
        })?;
    }
    std::os::unix::fs::symlink(PROVIDER_MRUSTC_RUSTC_BINARY_BASENAME, link_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "create rustc sysroot binary link {} -> {}: {err}",
            link_path.display(),
            PROVIDER_MRUSTC_RUSTC_BINARY_BASENAME
        ))
    })
}

#[cfg(not(unix))]
fn prepare_provider_rustc_sysroot_binary_link(
    binary_path: &Path,
    link_path: &Path,
) -> Result<(), RustSourceProviderError> {
    fs::copy(binary_path, link_path).map_err(|err| {
        RustSourceProviderError::Copy(format!(
            "copy rustc sysroot binary {} -> {}: {err}",
            binary_path.display(),
            link_path.display()
        ))
    })?;
    Ok(())
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
has_sysroot=false\n\
for arg in \"$@\"; do\n\
  case \"$arg\" in\n\
    --sysroot|--sysroot=*) has_sysroot=true ;;\n\
  esac\n\
done\n\
if [ \"$has_sysroot\" = false ]; then\n\
  set -- --sysroot \"$root_dir\" \"$@\"\n\
fi\n\
LD_LIBRARY_PATH=\"$ld_path\" exec \"$self_dir/rustc_binary.sysroot\" \"$@\"\n"
    )
}

fn provider_local_dynamic_mrustc_rustc_wrapper(host_triple: &str) -> String {
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
runtime_dir=\"$root_dir/{FIRST_STAGE_PROC_MACRO_RUNTIME_DIR}\"\n\
loader=\"$root_dir/{FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH}\"\n\
ld_path=\"$runtime_dir:$root_dir/lib:$root_dir/lib/rustlib/{host_triple}/lib\"\n\
if [ \"${{LD_LIBRARY_PATH+x}}\" = x ] && [ -n \"${{LD_LIBRARY_PATH}}\" ]; then\n\
  ld_path=\"$ld_path:$LD_LIBRARY_PATH\"\n\
fi\n\
has_sysroot=false\n\
for arg in \"$@\"; do\n\
  case \"$arg\" in\n\
    --sysroot|--sysroot=*) has_sysroot=true ;;\n\
  esac\n\
done\n\
if [ \"$has_sysroot\" = false ]; then\n\
  set -- --sysroot \"$root_dir\" \"$@\"\n\
fi\n\
LD_LIBRARY_PATH=\"$ld_path\" exec \"$loader\" \"$self_dir/{PROVIDER_MRUSTC_RUSTC_BINARY_BASENAME}\" \"$@\"\n"
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
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Rustc,
            name: "rustc",
            relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::Cargo,
            name: "cargo",
            relative_path: PROVIDER_CARGO_RELATIVE_PATH,
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::HostRustlib,
            name: "host-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.host_triple),
        })?,
        provider_receipt_artifact(ProviderReceiptArtifactRequest {
            provider_dir: candidate_dir,
            role: RustProviderRole::TargetRustlib,
            name: "target-rustlib",
            relative_path: &provider_rustlib_relative_path(&route.plan.target_triple),
        })?,
    ])
}

struct ProviderReceiptArtifactRequest<'a> {
    provider_dir: &'a Path,
    role: RustProviderRole,
    name: &'a str,
    relative_path: &'a str,
}

fn provider_receipt_artifact(
    request: ProviderReceiptArtifactRequest<'_>,
) -> Result<RustProviderReceiptArtifact, RustSourceProviderError> {
    Ok(RustProviderReceiptArtifact {
        role: request.role,
        name: request.name.to_string(),
        path: request.relative_path.to_string(),
        content_digest_blake3: content_digest_blake3(&request.provider_dir.join(request.relative_path))?,
    })
}

fn first_stage_provider_receipt(
    boundary: &RustSourceProviderFirstStageBoundary,
    route: &LoadedRustSourceProviderRoute,
    build: &RustSourceProviderFirstStageBuild,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> Result<RustSourceProviderBuildReceipt, RustSourceProviderError> {
    let patch_plan = first_stage_patch_plan(boundary)?;
    Ok(RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: first_stage_provider_receipt_steps(boundary, build, &patch_plan),
    })
}

fn first_stage_provider_receipt_steps(
    boundary: &RustSourceProviderFirstStageBoundary,
    build: &RustSourceProviderFirstStageBuild,
    patch_plan: &RustBootstrapPatchPlan,
) -> Vec<RustProviderReceiptStep> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert_eq!(build.stage_id, boundary.stage_id);
    debug_assert!(!patch_plan.operations.is_empty());
    let mut steps = vec![
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
    ];
    steps.push(stage_construction_identity_receipt_step(
        &build.plan_digest_blake3,
        &build.script_digest_blake3,
        build.parallel_job_count,
        boundary.full_source_context.as_ref(),
    ));
    steps.push(rust_bootstrap_patch_plan_receipt_step(patch_plan));
    steps
}

fn stage_construction_identity_receipt_step(
    plan_digest_blake3: &str,
    script_digest_blake3: &str,
    parallel_job_count: u32,
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> RustProviderReceiptStep {
    assert_eq!(plan_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    assert_eq!(script_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    assert!(parallel_job_count > 0);
    assert!(parallel_job_count <= FULL_SOURCE_RUST_STAGE_PARALLEL_JOB_COUNT_MAX);
    let mut arguments = vec![
        format!("stage-plan-blake3={plan_digest_blake3}"),
        format!("script-blake3={script_digest_blake3}"),
        format!("parallel-jobs={parallel_job_count}"),
    ];
    if let Some(context) = full_source_context {
        let linux_headers = context
            .host_tools
            .manifest
            .support_inputs
            .iter()
            .find(|input| input.id == "linux-headers")
            .expect("validated full-source context must bind Linux headers");
        assert_eq!(linux_headers.content_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
        arguments.extend([
            format!("native-provider-id={}", context.admission.provider_id),
            format!("native-provider-metadata-blake3={}", context.admission.metadata_digest_blake3),
            format!("native-provider-output-blake3={}", context.admission.output_digest_blake3),
            format!("source-closure-blake3={}", context.admission.source_closure_manifest_blake3),
            format!("admission-report-blake3={}", context.admission.admission_report_digest_blake3),
            format!("host-tool-manifest-blake3={}", context.host_tools.manifest_digest_blake3),
            format!("linux-headers-blake3={}", linux_headers.content_digest_blake3),
        ]);
    }
    RustProviderReceiptStep {
        name: "record-stage-construction-identity".to_string(),
        program: "mantle-provider-digest".to_string(),
        arguments,
    }
}

fn rust_bootstrap_patch_plan_receipt_step(patch_plan: &RustBootstrapPatchPlan) -> RustProviderReceiptStep {
    RustProviderReceiptStep {
        name: "record-rust-bootstrap-patch-plan".to_string(),
        program: "mantle-rust-bootstrap-patch-plan".to_string(),
        arguments: patch_plan.receipt_arguments(),
    }
}

fn first_stage_provider_metadata(input: RustProviderMetadataInput<'_>) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: input.route.plan.host_triple.clone(),
        target_triple: input.route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: FIRST_STAGE_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: input.sources.to_vec(),
        build_receipts: vec![first_stage_provider_receipt_identity(
            input.artifacts.receipt_digest_blake3,
        )],
        artifacts: first_stage_provider_artifacts(input.artifacts),
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

fn first_stage_provider_artifacts(input: RustProviderArtifactsInput<'_>) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = input
        .receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: input.artifact_source_id.to_string(),
            build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: FIRST_STAGE_PROVIDER_RECEIPT_NAME.to_string(),
        path: FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: input.receipt_digest_blake3.to_string(),
        source_id: input.artifact_source_id.to_string(),
        build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
    });
    debug_assert_eq!(artifacts.len(), input.receipt_artifacts.len().saturating_add(1));
    debug_assert!(!artifacts.is_empty());
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

fn first_stage_patch_plan(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<RustBootstrapPatchPlan, RustSourceProviderError> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert!(!boundary.route_plan_digest_blake3.is_empty());
    let build_sources = first_stage_build_sources(boundary)?;
    let input = RustBootstrapPatchPlanInput {
        stage: RustBootstrapPatchStage::FirstStage,
        route_id: route_id_for_patch_plan(&boundary.route_plan_path),
        route_plan_digest_blake3: boundary.route_plan_digest_blake3.clone(),
        route_policy_digest_blake3: boundary.route_policy_digest_blake3.clone(),
        stage_id: boundary.stage_id.clone(),
        rust_version: boundary.rust_version.clone(),
        mrustc_version: Some(source_version_from_id(SourceVersionRequest {
            source_id: &build_sources.mrustc_id,
            expected_prefix: FIRST_STAGE_MRUSTC_SOURCE_PREFIX,
        })?),
        host_triple: boundary.host_triple.clone(),
        target_triple: boundary.target_triple.clone(),
        source_identities: patch_plan_source_identities(&boundary.sources),
        capabilities: patch_plan_capabilities(false),
    };
    derive_rust_bootstrap_patch_plan(input).map_err(patch_plan_error)
}

fn rustc_stage1_patch_plan(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<RustBootstrapPatchPlan, RustSourceProviderError> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert!(!boundary.bootstrap_provider_metadata_digest_blake3.is_empty());
    let input = RustBootstrapPatchPlanInput {
        stage: RustBootstrapPatchStage::RustBootstrap,
        route_id: route_id_for_patch_plan(&boundary.route_plan_path),
        route_plan_digest_blake3: boundary.route_plan_digest_blake3.clone(),
        route_policy_digest_blake3: boundary.route_policy_digest_blake3.clone(),
        stage_id: boundary.stage_id.clone(),
        rust_version: boundary.rust_version.clone(),
        mrustc_version: None,
        host_triple: boundary.host_triple.clone(),
        target_triple: boundary.target_triple.clone(),
        source_identities: rustc_stage_patch_plan_source_identities(RustcPatchIdentityInput {
            sources: &boundary.sources,
            stage_id: &boundary.stage_id,
            bootstrap_provider_metadata_digest_blake3: &boundary.bootstrap_provider_metadata_digest_blake3,
        }),
        capabilities: patch_plan_capabilities(false),
    };
    derive_rust_bootstrap_patch_plan(input).map_err(patch_plan_error)
}

fn rustc_final_patch_plan(
    boundary: &RustSourceProviderRustcFinalBoundary,
) -> Result<RustBootstrapPatchPlan, RustSourceProviderError> {
    debug_assert!(!boundary.stage_id.is_empty());
    debug_assert!(!boundary.bootstrap_provider_metadata_digest_blake3.is_empty());
    let input = RustBootstrapPatchPlanInput {
        stage: RustBootstrapPatchStage::RustBootstrap,
        route_id: route_id_for_patch_plan(&boundary.route_plan_path),
        route_plan_digest_blake3: boundary.route_plan_digest_blake3.clone(),
        route_policy_digest_blake3: boundary.route_policy_digest_blake3.clone(),
        stage_id: boundary.stage_id.clone(),
        rust_version: boundary.rust_version.clone(),
        mrustc_version: None,
        host_triple: boundary.host_triple.clone(),
        target_triple: boundary.target_triple.clone(),
        source_identities: rustc_stage_patch_plan_source_identities(RustcPatchIdentityInput {
            sources: &boundary.sources,
            stage_id: &boundary.stage_id,
            bootstrap_provider_metadata_digest_blake3: &boundary.bootstrap_provider_metadata_digest_blake3,
        }),
        capabilities: patch_plan_capabilities(true),
    };
    derive_rust_bootstrap_patch_plan(input).map_err(patch_plan_error)
}

fn route_id_for_patch_plan(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or("rust-source-route")
        .to_string()
}

fn patch_plan_capabilities(rustdoc_tool: bool) -> RustBootstrapPatchCapabilities {
    RustBootstrapPatchCapabilities {
        source_built_provider: true,
        proc_macro_runtime: true,
        static_executable_linking: true,
        dynamic_tool_runtime: true,
        rustdoc_tool,
        provider_contract_version: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
    }
}

fn patch_plan_source_identities(
    sources: &[RustSourceProviderBootstrapSource],
) -> Vec<RustBootstrapPatchSourceIdentity> {
    sources
        .iter()
        .map(|source| RustBootstrapPatchSourceIdentity {
            id: source.id.clone(),
            name: source.name.clone(),
            digest_kind: "sha256".to_string(),
            digest: source.sha256_hex.clone(),
        })
        .collect()
}

struct RustcPatchIdentityInput<'a> {
    sources: &'a [RustSourceProviderBootstrapSource],
    stage_id: &'a str,
    bootstrap_provider_metadata_digest_blake3: &'a str,
}

fn rustc_stage_patch_plan_source_identities(
    input: RustcPatchIdentityInput<'_>,
) -> Vec<RustBootstrapPatchSourceIdentity> {
    let mut identities = patch_plan_source_identities(input.sources);
    identities.push(RustBootstrapPatchSourceIdentity {
        id: format!("{}-bootstrap-provider", input.stage_id),
        name: "bootstrap-provider-candidate".to_string(),
        digest_kind: "blake3".to_string(),
        digest: input.bootstrap_provider_metadata_digest_blake3.to_string(),
    });
    identities
}

struct SourceVersionRequest<'a> {
    source_id: &'a str,
    expected_prefix: &'a str,
}

fn source_version_from_id(request: SourceVersionRequest<'_>) -> Result<String, RustSourceProviderError> {
    request
        .source_id
        .strip_prefix(request.expected_prefix)
        .filter(|version| !version.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            RustSourceProviderError::Validate(format!(
                "source id '{}' lacks expected prefix '{}'",
                request.source_id, request.expected_prefix
            ))
        })
}

fn patch_plan_error(err: crate::rust_bootstrap_patch_plan::RustBootstrapPatchPlanError) -> RustSourceProviderError {
    RustSourceProviderError::Validate(format!("rust bootstrap patch plan: {}", err.message()))
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
    full_source_context: Option<&FullSourceRustExecutionContext>,
) -> Result<RustSourceProviderSourceBytes, RustSourceProviderError> {
    if let Some(context) = full_source_context {
        return read_full_source_archive_bytes(source, &context.rust_source_archive_dir);
    }
    let parsed = url::Url::parse(&source.url)
        .map_err(|err| RustSourceProviderError::Fetch(format!("source '{}' url parse: {err}", source.id)))?;
    let bytes = match parsed.scheme() {
        "file" => read_file_url_limited(&source.id, &parsed)?,
        "http" | "https" => fetch_http_source_bytes(HttpSourceRequest {
            source_id: &source.id,
            url: &source.url,
        })?,
        scheme => {
            return Err(RustSourceProviderError::Fetch(format!(
                "source '{}' uses unsupported url scheme '{scheme}'",
                source.id
            )));
        }
    };
    Ok(RustSourceProviderSourceBytes {
        bytes,
        source_policy: "authenticated-network-or-file",
        source_input_path: None,
    })
}

fn read_full_source_archive_bytes(
    source: &RustSourceProviderBootstrapSource,
    archive_dir: &Path,
) -> Result<RustSourceProviderSourceBytes, RustSourceProviderError> {
    let source_component = safe_source_component(&source.id)?;
    let path = archive_dir.join(format!("{source_component}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}"));
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        RustSourceProviderError::Fetch(format!("offline source '{}' {}: {error}", source.id, path.display()))
    })?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > FIRST_STAGE_FETCH_MAX_BYTES {
        return Err(RustSourceProviderError::Fetch(format!(
            "offline source '{}' must be a nonempty bounded regular file without symlink traversal: {}",
            source.id,
            path.display()
        )));
    }
    let mut file = File::open(&path).map_err(|error| {
        RustSourceProviderError::Fetch(format!("offline source '{}' open {}: {error}", source.id, path.display()))
    })?;
    let bytes = read_limited_bytes(&mut file, &format!("offline source '{}' {}", source.id, path.display()))?;
    Ok(RustSourceProviderSourceBytes {
        bytes,
        source_policy: "authenticated-offline-only",
        source_input_path: Some(path),
    })
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

#[derive(Clone, Copy)]
struct HttpSourceRequest<'a> {
    source_id: &'a str,
    url: &'a str,
}

fn fetch_http_source_bytes(request: HttpSourceRequest<'_>) -> Result<Vec<u8>, RustSourceProviderError> {
    debug_assert!(!request.source_id.is_empty());
    debug_assert!(!request.url.is_empty());
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FIRST_STAGE_FETCH_TIMEOUT_SECS)))
        .build()
        .into();
    let mut last_error = String::new();
    for attempt in 0..=FIRST_STAGE_FETCH_MAX_RETRIES {
        if attempt != 0 {
            sleep_before_first_stage_fetch_retry(request.url, attempt);
        }
        match fetch_http_source_once(&agent, request) {
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

fn fetch_http_source_once(
    agent: &ureq::Agent,
    request: HttpSourceRequest<'_>,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let response = agent.get(request.url).call().map_err(|err| {
        RustSourceProviderError::Fetch(format!("source '{}' {}: {err}", request.source_id, request.url))
    })?;
    let mut body = response.into_body();
    let mut reader = body.with_config().limit(FIRST_STAGE_FETCH_MAX_BYTES).reader();
    read_limited_bytes(&mut reader, &format!("source '{}' {}", request.source_id, request.url))
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
    debug_assert!(entry_count > 0);
    debug_assert!(entry_count <= FIRST_STAGE_ARCHIVE_MAX_ENTRIES);
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
    debug_assert!(matches!(first, Component::Normal(_)));
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
    debug_assert!(!stripped.is_absolute());
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

fn full_source_host_tool_path(
    context: &FullSourceRustExecutionContext,
    role: FullSourceRustHostToolRole,
) -> Result<&Path, RustSourceProviderError> {
    let tool =
        context.host_tools.manifest.tools.iter().find(|tool| tool.role == role).ok_or_else(|| {
            RustSourceProviderError::Validate(format!("full-source host-tool role {role:?} is missing"))
        })?;
    let path = Path::new(&tool.path);
    if !path.is_absolute() {
        return Err(RustSourceProviderError::Validate(format!(
            "full-source host-tool role {role:?} path is not absolute"
        )));
    }
    Ok(path)
}

fn full_source_busybox_applet_path(
    context: &FullSourceRustExecutionContext,
    applet: &str,
) -> Result<PathBuf, RustSourceProviderError> {
    let busybox = full_source_host_tool_path(context, FullSourceRustHostToolRole::Busybox)?;
    let parent = busybox.parent().ok_or_else(|| {
        RustSourceProviderError::Validate("full-source BusyBox path has no parent directory".to_string())
    })?;
    Ok(parent.join(applet))
}

fn push_generated_script_shebang(
    script: &mut String,
    context: Option<&FullSourceRustExecutionContext>,
) -> Result<(), RustSourceProviderError> {
    if let Some(context) = context {
        let shell = full_source_busybox_applet_path(context, "sh")?;
        script.push_str("#!");
        script.push_str(&shell.display().to_string());
        script.push('\n');
    } else {
        script.push_str("#!/bin/sh\n");
    }
    Ok(())
}

fn push_full_source_tool_bindings(
    script: &mut String,
    context: &FullSourceRustExecutionContext,
) -> Result<(), RustSourceProviderError> {
    let make = full_source_host_tool_path(context, FullSourceRustHostToolRole::Make)?;
    let cmake = full_source_host_tool_path(context, FullSourceRustHostToolRole::Cmake)?;
    let python = full_source_host_tool_path(context, FullSourceRustHostToolRole::Python)?;
    let perl = full_source_host_tool_path(context, FullSourceRustHostToolRole::Perl)?;
    let busybox = full_source_host_tool_path(context, FullSourceRustHostToolRole::Busybox)?;
    let busybox_dir = busybox
        .parent()
        .ok_or_else(|| RustSourceProviderError::Validate("full-source BusyBox path has no parent".to_string()))?;
    let make_dir = make
        .parent()
        .ok_or_else(|| RustSourceProviderError::Validate("full-source Make path has no parent".to_string()))?;
    let cmake_dir = cmake
        .parent()
        .ok_or_else(|| RustSourceProviderError::Validate("full-source CMake path has no parent".to_string()))?;
    let python_dir = python
        .parent()
        .ok_or_else(|| RustSourceProviderError::Validate("full-source Python path has no parent".to_string()))?;
    let perl_dir = perl
        .parent()
        .ok_or_else(|| RustSourceProviderError::Validate("full-source Perl path has no parent".to_string()))?;
    let python_home = python_dir.parent().ok_or_else(|| {
        RustSourceProviderError::Validate("full-source Python path has no installation root".to_string())
    })?;
    let perl_root = perl_dir.parent().ok_or_else(|| {
        RustSourceProviderError::Validate("full-source Perl path has no installation root".to_string())
    })?;
    let perl_library = perl_root.join(FULL_SOURCE_PERL_LIBRARY_RELATIVE_PATH);
    let cmake_root = cmake_dir.parent().ok_or_else(|| {
        RustSourceProviderError::Validate("full-source CMake path has no installation root".to_string())
    })?;
    let zlib_header = cmake_root.join("include/zlib.h");
    let zlib_library_dir = cmake_root.join("lib");
    let linux_headers_include = context.linux_headers_root.join("include");
    let zlib_archive = zlib_library_dir.join("libz.a");
    let native_bin = context.native_provider_dir.join("bin");
    let native_gcc = native_bin.join("x86_64-linux-musl-gcc");
    let native_gxx = native_bin.join("x86_64-linux-musl-g++");
    let native_ar = native_bin.join("x86_64-linux-musl-ar");
    let native_ranlib = native_bin.join("x86_64-linux-musl-ranlib");
    let gcc_subprogram_prefix = context.native_provider_dir.join(FULL_SOURCE_GCC_SUBPROGRAM_PREFIX_RELATIVE_PATH);
    let gcc_subprogram_prefix_flag = format!("-B{}/", gcc_subprogram_prefix.display());
    let controlled_path = [
        busybox_dir,
        make_dir,
        cmake_dir,
        python_dir,
        perl_dir,
        native_bin.as_path(),
    ]
    .iter()
    .map(|path| path.display().to_string())
    .collect::<Vec<_>>()
    .join(":");
    script.push_str("MANTLE_FULL_SOURCE_BOUND=true\n");
    script.push_str(&format!("MAKE_PROGRAM={}\n", shell_quote(&make.display().to_string())));
    script.push_str(&format!("CMAKE_PROGRAM={}\n", shell_quote(&cmake.display().to_string())));
    script.push_str(&format!("MANTLE_PYTHON_PROGRAM={}\n", shell_quote(&python.display().to_string())));
    script.push_str(&format!("PERL_PROGRAM={}\n", shell_quote(&perl.display().to_string())));
    script.push_str(&format!("PYTHONHOME={}\n", shell_quote(&python_home.display().to_string())));
    script.push_str(&format!("PERL5LIB={}\n", shell_quote(&perl_library.display().to_string())));
    script.push_str(&format!("COPY_PROGRAM={}\n", shell_quote(&busybox_dir.join("cp").display().to_string())));
    script.push_str(&format!("SHELL_PROGRAM={}\n", shell_quote(&busybox_dir.join("sh").display().to_string())));
    script.push_str(&format!("CC_PROGRAM={}\n", shell_quote(&native_gcc.display().to_string())));
    script.push_str(&format!("CXX_PROGRAM={}\n", shell_quote(&native_gxx.display().to_string())));
    script.push_str(&format!("TARGET_CC_PROGRAM={}\n", shell_quote(&native_gcc.display().to_string())));
    script.push_str(&format!("TARGET_CXX_PROGRAM={}\n", shell_quote(&native_gxx.display().to_string())));
    script.push_str(&format!("TARGET_AR_PROGRAM={}\n", shell_quote(&native_ar.display().to_string())));
    script.push_str(&format!("TARGET_RANLIB_PROGRAM={}\n", shell_quote(&native_ranlib.display().to_string())));
    script.push_str(&format!("MANTLE_TARGET_CC={}\n", shell_quote(&native_gcc.display().to_string())));
    script.push_str(&format!(
        "MANTLE_TARGET_TOOLCHAIN_ROOT={}\n",
        shell_quote(&context.native_provider_dir.display().to_string())
    ));
    script.push_str(&format!("SOURCE_ROOT={}\n", shell_quote(&context.native_provider_dir.display().to_string())));
    script.push_str(&format!("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG={}\n", shell_quote(&gcc_subprogram_prefix_flag)));
    script.push_str(&format!("unset {GCC_EXEC_PREFIX_ENV} {COMPILER_PATH_ENV}\n"));
    script.push_str(&format!("PATH={}\n", shell_quote(&controlled_path)));
    script
        .push_str("SHELL=\"$SHELL_PROGRAM\"\nCONFIG_SHELL=\"$SHELL_PROGRAM\"\nGNUMAKEFLAGS=\"SHELL=$SHELL_PROGRAM\"\n");
    script.push_str("PKG_CONFIG_PROGRAM=\n");
    script.push_str(&format!("MANTLE_ZLIB_HEADER={}\n", shell_quote(&zlib_header.display().to_string())));
    script.push_str(&format!("MANTLE_ZLIB_ARCHIVE={}\n", shell_quote(&zlib_archive.display().to_string())));
    script.push_str(&format!("LIBRARY_PATH={}\n", shell_quote(&zlib_library_dir.display().to_string())));
    script.push_str(&format!("ZLIB_CFLAGS={}\n", shell_quote(&format!("-I{}", cmake_root.join("include").display()))));
    script.push_str(&format!("ZLIB_LIBS={}\n", shell_quote(&zlib_archive.display().to_string())));
    script.push_str(&format!(
        "MANTLE_LINUX_HEADERS_ROOT={}\n",
        shell_quote(&context.linux_headers_root.display().to_string())
    ));
    script.push_str(&format!(
        "MANTLE_LINUX_HEADERS_CFLAGS={}\n",
        shell_quote(&format!("-I{}", linux_headers_include.display()))
    ));
    script.push_str("export PATH PYTHONHOME PERL5LIB LIBRARY_PATH MAKE_PROGRAM CMAKE_PROGRAM MANTLE_PYTHON_PROGRAM PERL_PROGRAM COPY_PROGRAM SHELL_PROGRAM SHELL CONFIG_SHELL GNUMAKEFLAGS CC_PROGRAM CXX_PROGRAM TARGET_CC_PROGRAM TARGET_CXX_PROGRAM TARGET_AR_PROGRAM TARGET_RANLIB_PROGRAM MANTLE_TARGET_CC MANTLE_TARGET_TOOLCHAIN_ROOT SOURCE_ROOT MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG MANTLE_ZLIB_HEADER MANTLE_ZLIB_ARCHIVE ZLIB_CFLAGS ZLIB_LIBS MANTLE_LINUX_HEADERS_ROOT MANTLE_LINUX_HEADERS_CFLAGS\n");
    script
        .push_str("printf '%s\\n' 'using receipt-bound full-source Rust host tools with ambient discovery disabled'\n");
    Ok(())
}

fn first_stage_boundary_script(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<String, RustSourceProviderError> {
    let build_sources = first_stage_build_sources(boundary)?;
    let patch_plan = first_stage_patch_plan(boundary)?;
    let mut script = String::new();
    push_generated_script_shebang(&mut script, boundary.full_source_context.as_ref())?;
    push_first_stage_script_header(&mut script, boundary, &build_sources, boundary.full_source_context.is_some());
    if let Some(context) = boundary.full_source_context.as_ref() {
        push_full_source_tool_bindings(&mut script, context)?;
    }
    push_first_stage_source_checks(&mut script, boundary);
    push_first_stage_env_scrub(&mut script);
    push_first_stage_tool_checks(&mut script, boundary.full_source_context.as_ref());
    push_first_stage_build_commands(&mut script, &patch_plan, boundary.full_source_context.is_some())?;
    Ok(script)
}

fn push_first_stage_script_header(
    script: &mut String,
    boundary: &RustSourceProviderFirstStageBoundary,
    build_sources: &RustSourceProviderFirstStageBuildSources,
    full_source_bound: bool,
) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
    script.push_str(&format!("PARLEVEL={RUST_BOOTSTRAP_JOB_COUNT}\n"));
    script.push_str("export PARLEVEL\n");
    if !full_source_bound {
        script.push_str(&format!("MAKE_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_MAKE_FALLBACK_GLOB)));
    }
    script.push_str(&format!("COPY_PROGRAM={}\n", shell_quote(FIRST_STAGE_COPY_PROGRAM)));
    script.push_str(&format!("PKG_CONFIG_PROGRAM={}\n", shell_quote(FIRST_STAGE_PKG_CONFIG_PROGRAM)));
    script.push_str(&format!("CMAKE_PROGRAM={}\n", shell_quote(FIRST_STAGE_CMAKE_PROGRAM)));
    script.push_str(&format!("CC_PROGRAM={}\n", shell_quote(FIRST_STAGE_CC_PROGRAM)));
    script.push_str(&format!("CXX_PROGRAM={}\n", shell_quote(FIRST_STAGE_CXX_PROGRAM)));
    script.push_str(&format!("HOST_GNU_TRIPLE={}\n", shell_quote(FIRST_STAGE_HOST_GNU_TRIPLE)));
    if !full_source_bound {
        script.push_str(&format!("CMAKE_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_CMAKE_FALLBACK_GLOB)));
        script.push_str(&format!("GCC_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_GCC_FALLBACK_GLOB)));
        script.push_str(&format!(
            "ZLIB_PKG_CONFIG_FALLBACK_GLOB={}\n",
            shell_quote(FIRST_STAGE_ZLIB_PKG_CONFIG_FALLBACK_GLOB)
        ));
    }
    script.push_str(&format!("MRUSTC_CXXFLAGS={}\n", shell_quote(FIRST_STAGE_CXXFLAGS)));
    script.push_str(&format!("TARGET_CC_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_CC_PROGRAM)));
    script.push_str(&format!("TARGET_CXX_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_CXX_PROGRAM)));
    script.push_str(&format!("TARGET_AR_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_AR_PROGRAM)));
    script.push_str(&format!("TARGET_RANLIB_PROGRAM={}\n", shell_quote(FIRST_STAGE_TARGET_RANLIB_PROGRAM)));
    if !full_source_bound {
        script.push_str(&format!(
            "TARGET_CC_PROGRAM_CANDIDATES={}\n",
            shell_quote(&musl_target_tool_program_candidates_shell_words("gcc"))
        ));
    }
    script.push_str(&format!("TARGET_MUSL_TOOL_PREFIXES={}\n", shell_quote(&musl_target_tool_prefixes_shell_words())));
    script.push_str(&format!(
        "TARGET_MUSL_MACHINE_ALIASES={}\n",
        shell_quote(&musl_target_machine_aliases_shell_words())
    ));
    script.push_str(&format!("TARGET_MUSL_SOURCE_ROOT_SYSROOT={}\n", shell_quote(FIRST_STAGE_SOURCE_ROOT_MUSL_PREFIX)));
    if !full_source_bound {
        script.push_str(&format!(
            "TARGET_MUSL_GCC_FALLBACK_GLOB={}\n",
            shell_quote(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB)
        ));
    }
}

fn push_first_stage_source_checks(script: &mut String, boundary: &RustSourceProviderFirstStageBoundary) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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

fn push_first_stage_tool_checks(script: &mut String, full_source_context: Option<&FullSourceRustExecutionContext>) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    if full_source_context.is_some() {
        for (shell_var, message) in [
            ("MAKE_PROGRAM", "receipt-bound make is required for mrustc first stage"),
            ("COPY_PROGRAM", "receipt-bound cp is required for mrustc first stage"),
            ("CMAKE_PROGRAM", "receipt-bound cmake is required for mrustc first stage"),
            ("CC_PROGRAM", "receipt-bound cc is required for mrustc first stage"),
            ("CXX_PROGRAM", "receipt-bound c++ is required for mrustc first stage"),
            ("SHELL_PROGRAM", "receipt-bound shell is required for mrustc first stage"),
            ("MANTLE_PYTHON_PROGRAM", "receipt-bound Python is required for Rust bootstrap"),
            ("PERL_PROGRAM", "receipt-bound Perl is required for Rust bootstrap"),
        ] {
            script.push_str(&format!(
                "if [ ! -x \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n",
                shell_quote(message)
            ));
        }
        for (shell_var, message) in [
            ("MANTLE_ZLIB_HEADER", "receipt-bound zlib header is required for mrustc first stage"),
            ("MANTLE_ZLIB_ARCHIVE", "receipt-bound zlib archive is required for mrustc first stage"),
        ] {
            script.push_str(&format!(
                "if [ ! -f \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n",
                shell_quote(message)
            ));
        }
        for (shell_var, message) in [
            ("PERL5LIB", "receipt-bound Perl library is required for Rust bootstrap"),
            ("LIBRARY_PATH", "receipt-bound zlib library directory is required for mrustc first stage"),
        ] {
            script.push_str(&format!(
                "if [ ! -d \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n",
                shell_quote(message)
            ));
        }
        return;
    }
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
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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

fn push_first_stage_target_musl_lfs_compat(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_target_musl_lfs_compat_preamble(script);
    script.push_str(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_HEADER_TEXT);
    script.push_str(&format!(
        "#define MANTLE_PTHREAD_TLS_KEY_CAPACITY {}u\n\n",
        FIRST_STAGE_TARGET_MUSL_TLS_KEY_CAPACITY
    ));
    script.push_str(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_BODY_TEXT);
    push_first_stage_target_musl_lfs_compat_compile(script, full_source_bound);
}

fn push_first_stage_target_musl_lfs_compat_preamble(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!(
        "target_lfs_compat_source=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_SOURCE}\"\n"
    ));
    script.push_str(&format!(
        "target_lfs_compat_object=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT}\"\n"
    ));
    script.push_str("cat > \"$target_lfs_compat_source\" <<'MANTLE_MUSL_LFS_COMPAT_C'\n");
}

const FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_HEADER_TEXT: &str = r#"#define _LARGEFILE64_SOURCE 1
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
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

"#;

const FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_BODY_TEXT: &str = r#"typedef void (*mantle_pthread_tls_destructor)(void *);

static int mantle_pthread_tls_used[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static mantle_pthread_tls_destructor mantle_pthread_tls_destructors[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static __thread void *mantle_pthread_tls_values[MANTLE_PTHREAD_TLS_KEY_CAPACITY];
static unsigned int mantle_pthread_tls_next_key;

static int mantle_pthread_tls_key_is_valid(pthread_key_t key) {
    unsigned int index = (unsigned int)key;
    if (index >= MANTLE_PTHREAD_TLS_KEY_CAPACITY) {
        return 0;
    }
    if (__sync_add_and_fetch(&mantle_pthread_tls_used[index], 0) == 0) {
        return 0;
    }
    return 1;
}

int __wrap_pthread_key_create(pthread_key_t *key, void (*destructor)(void *)) {
    if (key == NULL) {
        return EINVAL;
    }
    for (unsigned int offset = 0; offset < MANTLE_PTHREAD_TLS_KEY_CAPACITY; offset++) {
        unsigned int candidate = (mantle_pthread_tls_next_key + offset) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
        if (__sync_bool_compare_and_swap(&mantle_pthread_tls_used[candidate], 0, 1) != 0) {
            mantle_pthread_tls_destructors[candidate] = destructor;
            *key = (pthread_key_t)candidate;
            mantle_pthread_tls_next_key = (candidate + 1) % MANTLE_PTHREAD_TLS_KEY_CAPACITY;
            return 0;
        }
    }
    return EAGAIN;
}

int __wrap_pthread_key_delete(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    unsigned int index = (unsigned int)key;
    mantle_pthread_tls_values[index] = NULL;
    mantle_pthread_tls_destructors[index] = NULL;
    __sync_lock_release(&mantle_pthread_tls_used[index]);
    return 0;
}

void *__wrap_pthread_getspecific(pthread_key_t key) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return NULL;
    }
    return mantle_pthread_tls_values[(unsigned int)key];
}

int __wrap_pthread_setspecific(pthread_key_t key, const void *value) {
    if (mantle_pthread_tls_key_is_valid(key) == 0) {
        return EINVAL;
    }
    mantle_pthread_tls_values[(unsigned int)key] = (void *)value;
    return 0;
}

int backtrace(void **buffer, int size) {
    (void)buffer;
    (void)size;
    return 0;
}

void backtrace_symbols_fd(void *const *buffer, int size, int fd) {
    (void)buffer;
    (void)size;
    (void)fd;
}

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
"#;

fn push_first_stage_target_cc_compile_command(script: &mut String, full_source_bound: bool, arguments: &str) {
    let original_len = script.len();
    assert!(!arguments.is_empty());
    assert!(!arguments.contains('\n'));
    if full_source_bound {
        script.push_str("\"$target_cc_path\" \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" ");
    } else {
        script.push_str("\"$target_cc_path\" ");
    }
    script.push_str(arguments);
    script.push('\n');
    assert!(script.len() > original_len);
    assert_eq!(script[original_len..].contains("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"), full_source_bound);
}

fn push_first_stage_target_musl_lfs_compat_compile(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("MANTLE_MUSL_LFS_COMPAT_C\n");
    let arguments = format!(
        "-D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -fPIC -c \"$target_lfs_compat_source\" -o \"$target_lfs_compat_object\""
    );
    push_first_stage_target_cc_compile_command(script, full_source_bound, &arguments);
}

fn push_first_stage_target_musl_libatomic_shim(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_target_musl_libatomic_preamble(script);
    script.push_str(FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SOURCE_TEXT);
    push_first_stage_target_musl_libatomic_compile(script, full_source_bound);
}

fn push_first_stage_target_musl_libatomic_preamble(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!(
        "target_libatomic_source=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SHIM_SOURCE}\"\n"
    ));
    script.push_str(&format!(
        "target_libatomic_object=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SHIM_OBJECT}\"\n"
    ));
    script.push_str("cat > \"$target_libatomic_source\" <<'MANTLE_MUSL_LIBATOMIC_C'\n");
}

const FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SOURCE_TEXT: &str = r#"typedef unsigned char mantle_atomic_u8;

enum {
    ATOMIC_WIDTH_BYTES = sizeof(unsigned __int128),
    ATOMIC_LOCK_FREE = 0,
    ATOMIC_LOCK_HELD = 1,
};

static volatile int mantle_atomic_lock = ATOMIC_LOCK_FREE;

static void mantle_atomic_lock_acquire(void) {
    while (__sync_lock_test_and_set(&mantle_atomic_lock, ATOMIC_LOCK_HELD) != ATOMIC_LOCK_FREE) {
    }
}

static void mantle_atomic_lock_release(void) {
    __sync_lock_release(&mantle_atomic_lock);
}

_Bool __atomic_compare_exchange_16(
    volatile void *mem,
    void *expected,
    unsigned __int128 desired,
    _Bool weak,
    int success_memorder,
    int failure_memorder
) {
    (void)weak;
    (void)success_memorder;
    (void)failure_memorder;

    volatile mantle_atomic_u8 *actual_bytes = (volatile mantle_atomic_u8 *)mem;
    mantle_atomic_u8 *expected_bytes = (mantle_atomic_u8 *)expected;
    mantle_atomic_u8 *desired_bytes = (mantle_atomic_u8 *)&desired;
    mantle_atomic_u8 observed_bytes[ATOMIC_WIDTH_BYTES];
    _Bool matches = 1;

    mantle_atomic_lock_acquire();
    for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
        observed_bytes[byte_index] = actual_bytes[byte_index];
        if (observed_bytes[byte_index] != expected_bytes[byte_index]) {
            matches = 0;
        }
    }
    if (matches) {
        for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
            actual_bytes[byte_index] = desired_bytes[byte_index];
        }
    } else {
        for (unsigned int byte_index = 0; byte_index < ATOMIC_WIDTH_BYTES; byte_index += 1) {
            expected_bytes[byte_index] = observed_bytes[byte_index];
        }
    }
    mantle_atomic_lock_release();
    return matches;
}
"#;

fn push_first_stage_target_musl_libatomic_compile(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("MANTLE_MUSL_LIBATOMIC_C\n");
    let arguments = format!(
        "-fPIC {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -c \"$target_libatomic_source\" -o \"$target_libatomic_object\""
    );
    push_first_stage_target_cc_compile_command(script, full_source_bound, &arguments);
    script.push_str(&format!(
        "rm -f \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBATOMIC_STATIC_ARCHIVE}\"; \"$target_alias_dir/ar\" rcs \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBATOMIC_STATIC_ARCHIVE}\" \"$target_libatomic_object\"\n"
    ));
    script.push_str(&format!(
        "\"$target_alias_dir/ranlib\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBATOMIC_STATIC_ARCHIVE}\"\n"
    ));
}

fn push_first_stage_target_libgcc_shared_copy(script: &mut String) {
    script.push_str(&format!(
        "for target_libgcc_shared_name in {FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT} {FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT}; do\n"
    ));
    script.push_str("  for target_libgcc_shared_dir in \"$target_musl_crt_dir\" \"$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib\" \"$target_gcc_crt_dir\"; do\n");
    script.push_str("    if [ -f \"$target_libgcc_shared_dir/$target_libgcc_shared_name\" ]; then $COPY_PROGRAM \"$target_libgcc_shared_dir/$target_libgcc_shared_name\" \"$target_runtime_dir/$target_libgcc_shared_name\"; break; fi\n");
    script.push_str("  done\n");
    script.push_str("done\n");
}

fn push_first_stage_target_linker_wrapper(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_target_linker_discovery(script, full_source_bound);
    push_first_stage_target_linker_runtime(script, full_source_bound);
    push_first_stage_target_cc_wrapper(script, full_source_bound);
    push_first_stage_target_companion_wrappers(script, full_source_bound);
}

fn push_first_stage_target_linker_discovery(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!(
        "if [ \"$RUSTC_TARGET\" = \"$RUSTC_PROVIDER_TARGET_TRIPLE\" ] && [ \"$RUSTC_TARGET\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"
    ));
    if full_source_bound {
        script.push_str(&format!(
            "if [ ! -x \"$TARGET_CC_PROGRAM\" ]; then printf '%s\\n' 'receipt-bound x86_64 musl target gcc is missing' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
        ));
        script.push_str("target_cc_path=\"$TARGET_CC_PROGRAM\"\n");
        script.push_str("target_toolchain_root=\"$MANTLE_TARGET_TOOLCHAIN_ROOT\"\n");
    } else {
        push_first_stage_target_toolchain_root_preference(script);
        script.push_str("if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then for candidate_name in $TARGET_CC_PROGRAM_CANDIDATES; do if command -v \"$candidate_name\" >/dev/null 2>&1; then TARGET_CC_PROGRAM=\"$candidate_name\"; break; fi; done; fi\n");
        script.push_str("if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then for candidate in $TARGET_MUSL_GCC_FALLBACK_GLOB; do if [ -x \"$candidate\" ]; then TARGET_CC_PROGRAM=\"$candidate\"; break; fi; done; fi\n");
        script.push_str(&format!(
            "if ! command -v \"$TARGET_CC_PROGRAM\" >/dev/null 2>&1; then printf '%s\\n' 'x86_64 musl target gcc is required for run_rustc target linking' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
        ));
        script.push_str("target_cc_path=$(command -v \"$TARGET_CC_PROGRAM\")\n");
    }
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
    script.push_str("target_objcopy_program=$target_tool_prefix-objcopy\n");
    push_first_stage_target_toolchain_root_validation(script);
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
        "if [ ! -f \"$target_musl_crt_dir/crt1.o\" ] || [ ! -f \"$target_musl_crt_dir/{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}\" ] || [ ! -f \"$target_musl_crt_dir/rcrt1.o\" ] || [ ! -f \"$target_musl_crt_dir/crti.o\" ] || [ ! -f \"$target_musl_crt_dir/crtn.o\" ] || [ ! -f \"$target_gcc_crt_dir/crtbeginS.o\" ] || [ ! -f \"$target_gcc_crt_dir/libgcc.a\" ]; then printf '%s\\n' 'target gcc toolchain does not expose musl/gcc CRT and unwinder objects' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
}

fn push_first_stage_target_linker_runtime(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("export {FIRST_STAGE_TARGET_NIX_CC_WRAPPER_HOST_ROLE_VAR}=1\n"));
    script.push_str("export COPY_PROGRAM\n");
    script.push_str(&format!("target_alias_dir=\"$BUILD_DIR/{FIRST_STAGE_TARGET_LINKER_ALIAS_DIR}\"\n"));
    script.push_str(&format!("target_runtime_dir=\"$BUILD_DIR/{FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR}\"\n"));
    script.push_str("mkdir -p \"$target_alias_dir\" \"$target_runtime_dir\"\n");
    script.push_str(&format!(
        "for crt_name in crt1.o {FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT} rcrt1.o crti.o crtn.o; do rm -f \"$target_runtime_dir/$crt_name\"; $COPY_PROGRAM \"$target_musl_crt_dir/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n"
    ));
    script.push_str("for crt_name in crtbeginS.o crtendS.o; do rm -f \"$target_runtime_dir/$crt_name\"; $COPY_PROGRAM \"$target_gcc_crt_dir/$crt_name\" \"$target_runtime_dir/$crt_name\"; done\n");
    script.push_str(&format!(
        "if [ ! -x \"$target_tool_dir/$target_objcopy_program\" ]; then printf '%s\\n' 'target gcc toolchain does not expose objcopy for CRT normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("target_crtbegin_no_frame_init=\"$target_runtime_dir/crtbeginS.o.no-frame-init\"\n");
    script.push_str("rm -f \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("\"$target_tool_dir/$target_objcopy_program\" --remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array \"$target_runtime_dir/crtbeginS.o\" \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("rm -f \"$target_runtime_dir/crtbeginS.o\"\n");
    script.push_str("$COPY_PROGRAM \"$target_crtbegin_no_frame_init\" \"$target_runtime_dir/crtbeginS.o\"\n");
    script.push_str("rm -f \"$target_crtbegin_no_frame_init\"\n");
    script.push_str("target_unwind_archive=\"$target_gcc_crt_dir/libgcc_eh.a\"\n");
    script.push_str(
        "if [ ! -f \"$target_unwind_archive\" ]; then target_unwind_archive=\"$target_gcc_crt_dir/libgcc.a\"; fi\n",
    );
    script.push_str("rm -f \"$target_runtime_dir/libunwind.a\"; $COPY_PROGRAM \"$target_unwind_archive\" \"$target_runtime_dir/libunwind.a\"\n");
    script.push_str("rm -f \"$target_runtime_dir/libgcc.a\"; $COPY_PROGRAM \"$target_gcc_crt_dir/libgcc.a\" \"$target_runtime_dir/libgcc.a\"\n");
    script.push_str(&format!(
        "rm -f \"$target_runtime_dir/libgcc_s.a\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT}\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT}\"\n"
    ));
    script.push_str("$COPY_PROGRAM \"$target_gcc_crt_dir/libgcc.a\" \"$target_runtime_dir/libgcc_s.a\"\n");
    push_first_stage_target_libgcc_shared_copy(script);
    push_first_stage_target_musl_lfs_compat(script, full_source_bound);
}

fn push_first_stage_target_cc_wrapper(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    if full_source_bound {
        script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/sh\"\n");
        script.push_str("printf '%s\\n' \"exec \\\"$SHELL_PROGRAM\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/sh\"\n");
        script.push_str("chmod +x \"$target_alias_dir/sh\"\n");
        script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/emcc\"\n");
        script.push_str("printf '%s\\n' 'exit 127' >> \"$target_alias_dir/emcc\"\n");
        script.push_str("chmod +x \"$target_alias_dir/emcc\"\n");
        for unavailable_tool in ["pkg-config", "pkgconf", "git"] {
            script.push_str(&format!(
                "printf '%s\\n' \"#!$SHELL_PROGRAM\" 'exit 127' > \"$target_alias_dir/{unavailable_tool}\"\n"
            ));
            script.push_str(&format!("chmod +x \"$target_alias_dir/{unavailable_tool}\"\n"));
        }
        script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/perl\"\n");
        script.push_str("printf '%s\\n' \"exec \\\"$PERL_PROGRAM\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/perl\"\n");
        script.push_str("chmod +x \"$target_alias_dir/perl\"\n");
        script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/make\"\n");
        script.push_str("printf '%s\\n' \"exec \\\"$MAKE_PROGRAM\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/make\"\n");
        script.push_str("chmod +x \"$target_alias_dir/make\"\n");
        script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/cc\"\n");
    } else {
        script.push_str("printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/cc\"\n");
    }
    script.push_str("printf '%s\\n' 'set -eu' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' \"target_cc_path=\\\"\\${MANTLE_TARGET_CC_PATH:-$target_cc_path}\\\"\" >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' \"target_runtime_dir=\\\"$target_runtime_dir\\\"\" >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'static_pie_normalized=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'output_path=' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'previous_arg=' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'rustc_main_response_file=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'shared_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  if [ \"$previous_arg\" = \"-o\" ]; then output_path=\"$arg\"; previous_arg=; continue; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -static-pie) static_pie_normalized=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -shared|-dynamiclib) shared_link=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -o) previous_arg=-o ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    @*/output/rustc-build/rustc_main_cmd.txt|@output/rustc-build/rustc_main_cmd.txt) rustc_main_response_file=true ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'dynamic_rustc_link=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'if [ \"$rustc_main_response_file\" = true ]; then dynamic_rustc_link=true; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'case \"$output_path\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "printf '%s\\n' '  */output/rustc|output/rustc|*/output/rustc-build/rustc_main|output/rustc-build/rustc_main) dynamic_rustc_link=true ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("printf '%s\\n' 'esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'mapped_args_set=false' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "printf '%s\\n' '    rcrt1.o|*/rcrt1.o) mapped_arg=\"$target_runtime_dir/crt1.o\" ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str(&format!(
        "printf '%s\\n' '    crt1.o|*/crt1.o|{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}|*/{FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT}|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg=\"$target_runtime_dir/${{arg##*/}}\" ;;' >> \"$target_alias_dir/cc\"\n"
    ));
    script.push_str(
        "printf '%s\\n' '    -static) if [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"$arg\"; fi ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("printf '%s\\n' '    -static-pie) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"-static\"; fi ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    *) mapped_arg=\"$arg\" ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  if [ \"$mapped_args_set\" = false ]; then set -- \"$mapped_arg\"; mapped_args_set=true; else set -- \"$@\" \"$mapped_arg\"; fi' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "printf '%s\\n' 'if [ \"$mapped_args_set\" = false ]; then set --; fi' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("printf '%s\\n' 'link_command=true' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'static_support_link=true' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'for arg in \"$@\"; do' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '  case \"$arg\" in' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' '    -c|-S|-E) link_command=false ;;' >> \"$target_alias_dir/cc\"\n");
    script.push_str(
        "printf '%s\\n' '    -shared|-dynamiclib) static_support_link=false ;;' >> \"$target_alias_dir/cc\"\n",
    );
    script.push_str("printf '%s\\n' '  esac' >> \"$target_alias_dir/cc\"\n");
    script.push_str("printf '%s\\n' 'done' >> \"$target_alias_dir/cc\"\n");
    script.push_str(&format!(
        "printf '%s\\n' 'if [ \"$link_command\" = true ] && [ \"$dynamic_rustc_link\" = true ]; then set -- \"$@\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT}\" -no-pie -Wl,-Bdynamic \"-Wl,-dynamic-linker,$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\" {FIRST_STAGE_TARGET_MUSL_PTHREAD_TLS_WRAP_FLAGS} -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; elif [ \"$link_command\" = true ] && [ \"$static_support_link\" = true ]; then set -- \"$@\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT}\" -static {FIRST_STAGE_TARGET_MUSL_PTHREAD_TLS_WRAP_FLAGS} -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group; fi' >> \"$target_alias_dir/cc\"\n"
    ));
    if full_source_bound {
        script.push_str(&format!(
            "printf '%s\\n' 'exec \"$target_cc_path\" \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" -D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -B\"$target_runtime_dir/\" -L\"$target_runtime_dir\" \"$@\"' >> \"$target_alias_dir/cc\"\n"
        ));
    } else {
        script.push_str(&format!(
            "printf '%s\\n' 'exec \"$target_cc_path\" -D{FIRST_STAGE_TARGET_LARGEFILE64_FEATURE_DEFINE} {FIRST_STAGE_TARGET_NO_ASYNC_UNWIND_TABLES_FLAG} -B\"$target_runtime_dir/\" -L\"$target_runtime_dir\" \"$@\"' >> \"$target_alias_dir/cc\"\n"
        ));
    }
    script.push_str("chmod +x \"$target_alias_dir/cc\"\n");
}

fn push_first_stage_target_companion_wrappers(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    if full_source_bound {
        script.push_str("if [ -x \"$target_tool_dir/$target_cxx_program\" ]; then printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/c++\"; printf '%s\\n' 'set -eu' >> \"$target_alias_dir/c++\"; printf '%s\\n' \"MANTLE_TARGET_CC_PATH=\\\"$target_tool_dir/$target_cxx_program\\\"\" >> \"$target_alias_dir/c++\"; printf '%s\\n' 'export MANTLE_TARGET_CC_PATH' >> \"$target_alias_dir/c++\"; printf '%s\\n' \"exec \\\"$target_alias_dir/cc\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/c++\"; chmod +x \"$target_alias_dir/c++\"; fi\n");
    } else {
        script.push_str("if [ -x \"$target_tool_dir/$target_cxx_program\" ]; then printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/c++\"; printf '%s\\n' 'set -eu' >> \"$target_alias_dir/c++\"; printf '%s\\n' \"MANTLE_TARGET_CC_PATH=\\\"$target_tool_dir/$target_cxx_program\\\"\" >> \"$target_alias_dir/c++\"; printf '%s\\n' 'export MANTLE_TARGET_CC_PATH' >> \"$target_alias_dir/c++\"; printf '%s\\n' \"exec \\\"$target_alias_dir/cc\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/c++\"; chmod +x \"$target_alias_dir/c++\"; fi\n");
    }
    script.push_str("PATH=\"$target_alias_dir:$PATH\"\n");
    script.push_str("export PATH\n");
    if full_source_bound {
        script.push_str("for target_tool in ar ranlib; do case \"$target_tool\" in ar) target_program=\"$target_ar_program\" ;; ranlib) target_program=\"$target_ranlib_program\" ;; esac; if [ -x \"$target_tool_dir/$target_program\" ]; then printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$target_alias_dir/$target_tool\"; printf '%s\\n' \"exec \\\"$target_tool_dir/$target_program\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/$target_tool\"; chmod +x \"$target_alias_dir/$target_tool\"; fi; done\n");
    } else {
        script.push_str("for target_tool in ar ranlib; do case \"$target_tool\" in ar) target_program=\"$target_ar_program\" ;; ranlib) target_program=\"$target_ranlib_program\" ;; esac; if [ -x \"$target_tool_dir/$target_program\" ]; then printf '%s\\n' '#!/bin/sh' > \"$target_alias_dir/$target_tool\"; printf '%s\\n' \"exec \\\"$target_tool_dir/$target_program\\\" \\\"\\$@\\\"\" >> \"$target_alias_dir/$target_tool\"; chmod +x \"$target_alias_dir/$target_tool\"; fi; done\n");
    }
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

fn push_first_stage_musl_host_llvm_runtime(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!(
        "if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ] && [ \"$RUSTC_PROVIDER_TARGET_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"
    ));
    script.push_str("printf '%s\\n' 'preparing source-root musl LLVM host compiler runtime'\n");
    script.push_str("RUSTC_TARGET=\"$RUSTC_HOST_TRIPLE\"\n");
    push_first_stage_target_linker_wrapper(script, full_source_bound);
    script.push_str(&format!(
        "if [ ! -x \"$target_alias_dir/cc\" ] || [ ! -x \"$target_alias_dir/c++\" ] || [ ! -x \"$target_alias_dir/ar\" ] || [ ! -x \"$target_alias_dir/ranlib\" ]; then printf '%s\\n' 'source-root musl LLVM host wrapper tools are incomplete' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    push_first_stage_target_musl_libatomic_shim(script, full_source_bound);
    script.push_str("MINICARGO_FLAGS=\"${MINICARGO_FLAGS:-} --target $RUSTC_HOST_TRIPLE\"\n");
    script.push_str("export MINICARGO_FLAGS\n");
    script.push_str(&format!(
        "target_static_stdcxx_archive=\"$target_musl_crt_dir/{FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE}\"\n"
    ));
    script.push_str(&format!(
        "if [ ! -f \"$target_static_stdcxx_archive\" ]; then target_static_stdcxx_archive=\"$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib/{FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE}\"; fi\n"
    ));
    script.push_str("if [ -f \"$target_static_stdcxx_archive\" ]; then\n");
    script.push_str(&format!(
        "  rm -f \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE}\"; $COPY_PROGRAM \"$target_static_stdcxx_archive\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE}\"\n"
    ));
    script.push_str(&format!(
        "  LLVM_STATIC_STDCPP=\"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE}\"\n"
    ));
    script.push_str("else\n");
    script.push_str(&format!(
        "  if [ ! -f \"$target_orig_cc_file\" ]; then printf '%s\\n' 'source-root musl libstdc++.a missing for LLVM host build' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("  printf '%s\\n' 'source-root musl libstdc++.a unavailable; continuing only for wrapper-backed synthetic route without LLVM_STATIC_STDCPP'\n");
    script.push_str("fi\n");
    script.push_str("CC_PROGRAM=\"$target_alias_dir/cc\"\n");
    script.push_str("CXX_PROGRAM=\"$target_alias_dir/c++\"\n");
    script.push_str("AR=\"$target_alias_dir/ar\"\n");
    script.push_str("RANLIB=\"$target_alias_dir/ranlib\"\n");
    script.push_str("CMAKE_C_COMPILER=\"$target_alias_dir/cc\"\n");
    script.push_str("CMAKE_CXX_COMPILER=\"$target_alias_dir/c++\"\n");
    script.push_str("CMAKE_AR=\"$target_alias_dir/ar\"\n");
    script.push_str("CMAKE_RANLIB=\"$target_alias_dir/ranlib\"\n");
    script.push_str("CMAKE_C_COMPILER_AR=\"$target_alias_dir/ar\"\n");
    script.push_str("CMAKE_CXX_COMPILER_AR=\"$target_alias_dir/ar\"\n");
    script.push_str("CMAKE_C_COMPILER_RANLIB=\"$target_alias_dir/ranlib\"\n");
    script.push_str("CMAKE_CXX_COMPILER_RANLIB=\"$target_alias_dir/ranlib\"\n");
    script.push_str("MRUSTC_CXXFLAGS=\"$MRUSTC_CXXFLAGS -static-libstdc++ -static-libgcc\"\n");
    if !full_source_bound {
        script.push_str("ZLIB_CFLAGS=\n");
        script.push_str("ZLIB_LIBS=\n");
    }
    script.push_str("CC=\"$CC_PROGRAM\"\n");
    script.push_str("CXX=\"$CXX_PROGRAM\"\n");
    if full_source_bound {
        script.push_str("CFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_CFLAGS\"\n");
        script.push_str("CXXFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MRUSTC_CXXFLAGS $ZLIB_CFLAGS\"\n");
        script.push_str("LDFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_LIBS\"\n");
    } else {
        script.push_str("CFLAGS=\"$ZLIB_CFLAGS\"\n");
        script.push_str("CXXFLAGS=\"$MRUSTC_CXXFLAGS $ZLIB_CFLAGS\"\n");
        script.push_str("LDFLAGS=\"$ZLIB_LIBS\"\n");
    }
    script.push_str("CPPFLAGS=\"$ZLIB_CFLAGS\"\n");
    script.push_str("LIBS=\"$ZLIB_LIBS\"\n");
    script.push_str("if [ -n \"${LLVM_LINKER_FLAGS:-}\" ]; then LLVM_LINKER_FLAGS=\"-L$target_runtime_dir -lgcc -lunwind $LLVM_LINKER_FLAGS\"; else LLVM_LINKER_FLAGS=\"-L$target_runtime_dir -lgcc -lunwind\"; fi\n");
    script.push_str("export CC_PROGRAM CXX_PROGRAM CC CXX CFLAGS CPPFLAGS CXXFLAGS LDFLAGS LIBS AR RANLIB CMAKE_C_COMPILER CMAKE_CXX_COMPILER CMAKE_AR CMAKE_RANLIB CMAKE_C_COMPILER_AR CMAKE_CXX_COMPILER_AR CMAKE_C_COMPILER_RANLIB CMAKE_CXX_COMPILER_RANLIB LLVM_STATIC_STDCPP LLVM_LINKER_FLAGS\n");
    script.push_str(
        "printf '%s\\n' \"using source-root musl LLVM host wrappers: CC=$CC_PROGRAM CXX=$CXX_PROGRAM LLVM_STATIC_STDCPP=${LLVM_STATIC_STDCPP:-} LLVM_LINKER_FLAGS=$LLVM_LINKER_FLAGS\"\n",
    );
    script.push_str("fi\n");
}

fn push_first_stage_build_commands(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_bound: bool,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_build_pipeline(script, patch_plan, full_source_bound)?;
    push_first_stage_product_checks(script)
}

fn push_first_stage_authenticated_version_metadata(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    let replacement = r#"	$V$(CXX) -o $@ -c $< $(CXXFLAGS) $(CPPFLAGS) -MMD -MP -MF $@.dep -D VERSION_GIT_FULLHASH=\"authenticated-mrustc-0.12.0\" -D VERSION_GIT_BRANCH=\"v0.12.0\" -D VERSION_GIT_SHORTHASH=\"v0.12.0\" -D VERSION_BUILDTIME=\"1970-01-01T00:00:00Z\" -D VERSION_GIT_ISDIRTY=0"#;
    script.push_str("mrustc_makefile=Makefile\n");
    script.push_str("mrustc_version_tmp=\"$BUILD_DIR/mrustc-authenticated-version.mk\"\n");
    script.push_str("mrustc_version_matches=0\n");
    script.push_str(&format!("mrustc_version_replacement={}\n", shell_quote(replacement)));
    script.push_str(": > \"$mrustc_version_tmp\"\n");
    script.push_str("while IFS= read -r line; do\n");
    script.push_str("  if [ \"$line\" = \"$mrustc_version_replacement\" ]; then\n");
    script.push_str("    mrustc_version_matches=$((mrustc_version_matches + 1))\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$mrustc_version_tmp\"\n");
    script.push_str("  else\n");
    script.push_str("    case \"$line\" in\n");
    script.push_str("      *'-D VERSION_GIT_FULLHASH='*'-D VERSION_GIT_ISDIRTY='*) mrustc_version_matches=$((mrustc_version_matches + 1)); printf '%s\\n' \"$mrustc_version_replacement\" >> \"$mrustc_version_tmp\" ;;\n");
    script.push_str("      *) printf '%s\\n' \"$line\" >> \"$mrustc_version_tmp\" ;;\n");
    script.push_str("    esac\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$mrustc_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$mrustc_version_matches\" -ne 1 ]; then printf '%s\\n' 'mrustc Makefile must expose exactly one authenticated version metadata rule' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$mrustc_version_tmp\" \"$mrustc_makefile\"\n");
    script.push_str("rm -f \"$mrustc_version_tmp\"\n");
}

fn push_first_stage_mrustc_ivar_bounds_fix(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_exact_source_rewrite(
        script,
        "src/hir_typeck/expr_cs.cpp",
        "                                context.possible_ivar_vals[te->index].force_disable = true;",
        &[
            "                                if(te->index >= context.possible_ivar_vals.size()) {",
            "                                    context.possible_ivar_vals.resize(te->index + 1);",
            "                                }",
            "                                context.possible_ivar_vals[te->index].force_disable = true;",
        ],
        "mrustc-ivar-bounds",
    );
}

fn push_generated_scratch_temp_environment(script: &mut String) {
    assert!(!GENERATED_SCRIPT_TEMP_DIR.is_empty());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("TEMP_ROOT=\"$BUILD_DIR/{GENERATED_SCRIPT_TEMP_DIR}\"\n"));
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$TEMP_ROOT\"\n");
    script.push_str("export TMPDIR=\"$TEMP_ROOT\"\nexport TMP=\"$TEMP_ROOT\"\nexport TEMP=\"$TEMP_ROOT\"\nexport TEMPDIR=\"$TEMP_ROOT\"\n");
    assert!(script.contains("export TMPDIR=\"$TEMP_ROOT\""));
}

fn push_first_stage_mrustc_make_command(script: &mut String, full_source_bound: bool) {
    let original_len = script.len();
    debug_assert!(!script.contains('\0'));
    if full_source_bound {
        script.push_str(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} CC=\"$CC\" CXX=\"$CXX\" CXXFLAGS=\"$CXXFLAGS\" LDFLAGS=\"$LDFLAGS\" LINKFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" LIBS=\"$LIBS\"\n"
        ));
    } else {
        script.push_str(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} CC=\"$CC\" CXX=\"$CXX\" CXXFLAGS=\"$CXXFLAGS\" LDFLAGS=\"$LDFLAGS\" LIBS=\"$LIBS\"\n"
        ));
    }
    assert!(script.len() > original_len);
    assert_eq!(script[original_len..].contains("LINKFLAGS_EXTRA"), full_source_bound);
}

fn push_first_stage_minicargo_make_command(script: &mut String, full_source_bound: bool) {
    let original_len = script.len();
    debug_assert!(!script.contains('\0'));
    if full_source_bound {
        script.push_str(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} CXXFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" LINKFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" {FIRST_STAGE_MINICARGO_BINARY}\n"
        ));
    } else {
        script.push_str(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_MINICARGO_BINARY}\n"
        ));
    }
    assert!(script.len() > original_len);
    assert_eq!(script[original_len..].contains("CXXFLAGS_EXTRA"), full_source_bound);
    assert_eq!(script[original_len..].contains("LINKFLAGS_EXTRA"), full_source_bound);
}

fn push_first_stage_build_pipeline(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    full_source_bound: bool,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(
        "printf '%s\\n' \"verified sources manifest: $SOURCE_MANIFEST\"\nprintf '%s\\n' \"build dir: $BUILD_DIR\"\nprintf '%s\\n' \"output dir: $OUTPUT_DIR\"\n",
    );
    push_generated_scratch_temp_environment(script);
    script.push_str("$COPY_PROGRAM \"$RUST_ARCHIVE\" \"$MRUSTC_SOURCE/rustc-${RUSTC_VERSION}-src.tar.gz\"\n");
    script.push_str("cd \"$MRUSTC_SOURCE\"\n");
    if full_source_bound {
        push_first_stage_authenticated_version_metadata(script);
        push_first_stage_mrustc_ivar_bounds_fix(script);
    }
    push_first_stage_minicargo_workspace_boundary(script);
    script.push_str("export RUSTC_TARGET MRUSTC_TARGET_VER RUSTC_VERSION\n");
    script.push_str("export RUSTC_INSTALL_BINDIR=bin\n");
    script.push_str("export OUTDIR_SUF=\n");
    script.push_str("export CARGO_CFG_RUSTIX_NO_LINUX_RAW=1\n");
    script.push_str("export CC=\"$CC_PROGRAM\"\n");
    script.push_str("export CXX=\"$CXX_PROGRAM\"\n");
    if full_source_bound {
        script.push_str(
            "export CFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS\"\n",
        );
        script.push_str("export CPPFLAGS=\"$ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS\"\n");
        script.push_str(
            "export CXXFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MRUSTC_CXXFLAGS $ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS\"\n",
        );
        script.push_str("export LDFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_LIBS\"\n");
    } else {
        script.push_str("export CFLAGS=\"$ZLIB_CFLAGS\"\n");
        script.push_str("export CPPFLAGS=\"$ZLIB_CFLAGS\"\n");
        script.push_str("export CXXFLAGS=\"$MRUSTC_CXXFLAGS $ZLIB_CFLAGS\"\n");
        script.push_str("export LDFLAGS=\"$ZLIB_LIBS\"\n");
    }
    script.push_str("export LIBS=\"$ZLIB_LIBS\"\n");
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::BeforeFirstStageMainMake,
        full_source_bound,
    )?;
    push_first_stage_mrustc_make_command(script, full_source_bound);
    push_first_stage_minicargo_make_command(script, full_source_bound);
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::AfterFirstStageMinicargo,
        full_source_bound,
    )?;
    if full_source_bound {
        push_first_stage_busybox_patch_normalization(script);
    }
    script.push_str(&format!(
        "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} RUSTCSRC\n"
    ));
    if full_source_bound {
        push_first_stage_busybox_patch_replacements(script);
    }
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::AfterFirstStageRustSourceExtract,
        full_source_bound,
    )?;
    script.push_str(&format!(
        "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_RUSTC_BINARY}\n"
    ));
    push_first_stage_translated_cargo_build(script, full_source_bound);
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::AfterFirstStageTranslatedCargo,
        full_source_bound,
    )?;
    push_first_stage_run_rustc_stage_tool_absolute_patch(script, full_source_bound);
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::FirstStageRunRustcHost,
        full_source_bound,
    )?;
    push_first_stage_patch_plan_operations(
        script,
        patch_plan,
        RustBootstrapPatchPhase::FirstStageRunRustcTarget,
        full_source_bound,
    )?;
    Ok(())
}

fn push_first_stage_translated_cargo_build(script: &mut String, full_source_bound: bool) {
    let original_len = script.len();
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE}"));
    if full_source_bound {
        script.push_str(&format!(" PARLEVEL={FIRST_STAGE_TRANSLATED_CARGO_MINICARGO_JOB_COUNT}"));
    }
    script.push_str(&format!(" {FIRST_STAGE_TRANSLATED_CARGO_BINARY}\n"));
    assert!(script.len() > original_len);
    assert!(script[original_len..].contains(FIRST_STAGE_TRANSLATED_CARGO_BINARY));
}

fn push_first_stage_busybox_patch_normalization(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("mrustc_patch_file=rustc-${RUSTC_VERSION}-src.patch\n");
    script.push_str("mrustc_patch_tmp=\"$BUILD_DIR/rustc-source-busybox.patch\"\n");
    script.push_str("mrustc_patch_skip=false\nmrustc_patch_skipped=0\n");
    script.push_str(": > \"$mrustc_patch_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  case \"$line\" in\n");
    script.push_str("    '--- compiler/rustc_errors/src/lib.rs'|'--- compiler/rustc_middle/src/ty/sty.rs') mrustc_patch_skip=true; mrustc_patch_skipped=$((mrustc_patch_skipped + 1)) ;;\n");
    script.push_str("    '--- '*) mrustc_patch_skip=false; printf '%s\\n' \"$line\" >> \"$mrustc_patch_tmp\" ;;\n");
    script.push_str(
        "    *) if [ \"$mrustc_patch_skip\" = false ]; then printf '%s\\n' \"$line\" >> \"$mrustc_patch_tmp\"; fi ;;\n",
    );
    script.push_str("  esac\n");
    script.push_str("done < \"$mrustc_patch_file\"\n");
    script.push_str(&format!(
        "if [ \"$mrustc_patch_skipped\" -ne {FIRST_STAGE_BUSYBOX_PATCH_SKIPPED_SECTION_COUNT} ]; then printf '%s\\n' 'mrustc Rust patch lacks the exact BusyBox-incompatible section set' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$mrustc_patch_tmp\" \"$mrustc_patch_file\"\n");
    script.push_str("rm -f \"$mrustc_patch_tmp\"\n");
}

fn push_first_stage_busybox_patch_replacements(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_exact_source_rewrite(
        script,
        "rustc-${RUSTC_VERSION}-src/compiler/rustc_errors/src/lib.rs",
        "rustc_data_structures::static_assert_size!(PResult<'_, bool>, 24);",
        &[
            "#[cfg(not(rust_compiler=\"mrustc\"))]",
            "rustc_data_structures::static_assert_size!(PResult<'_, bool>, 24);",
        ],
        "rustc-errors-presult",
    );
    push_first_stage_exact_source_rewrite(
        script,
        "rustc-${RUSTC_VERSION}-src/compiler/rustc_middle/src/ty/sty.rs",
        "        self.split_last().unwrap().1",
        &["        (**self).split_last().unwrap().1"],
        "rustc-middle-tys-inputs",
    );
}

fn push_first_stage_exact_source_rewrite(
    script: &mut String,
    source_path: &str,
    original_line: &str,
    replacement_lines: &[&str],
    label: &str,
) {
    debug_assert!(!source_path.is_empty());
    debug_assert!(!replacement_lines.is_empty());
    let prefix = label.replace('-', "_");
    script.push_str(&format!("{prefix}_source={source_path}\n"));
    script.push_str(&format!("{prefix}_tmp=\"$BUILD_DIR/{label}.tmp\"\n{prefix}_matches=0\n"));
    script.push_str(&format!(": > \"${{{prefix}_tmp}}\"\nwhile IFS= read -r line || [ -n \"$line\" ]; do\n"));
    script.push_str(&format!("  if [ \"$line\" = {} ]; then\n", shell_quote(original_line)));
    for replacement_line in replacement_lines {
        script.push_str(&format!("    printf '%s\\n' {} >> \"${{{prefix}_tmp}}\"\n", shell_quote(replacement_line)));
    }
    script.push_str(&format!("    {prefix}_matches=$((\u{24}{{{prefix}_matches}} + 1))\n  else\n    printf '%s\\n' \"$line\" >> \"${{{prefix}_tmp}}\"\n  fi\ndone < \"${{{prefix}_source}}\"\n"));
    script.push_str(&format!("if [ \"${{{prefix}_matches}}\" -ne 1 ]; then printf '%s\\n' 'authenticated Rust source rewrite {label} did not match exactly once' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"));
    script.push_str(&format!(
        "$COPY_PROGRAM \"${{{prefix}_tmp}}\" \"${{{prefix}_source}}\"\nrm -f \"${{{prefix}_tmp}}\"\n"
    ));
}

fn push_first_stage_product_checks(script: &mut String) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
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
    Ok(())
}

fn push_first_stage_patch_plan_operations(
    script: &mut String,
    patch_plan: &RustBootstrapPatchPlan,
    phase: RustBootstrapPatchPhase,
    full_source_bound: bool,
) -> Result<(), RustSourceProviderError> {
    for operation in patch_plan.operations.iter().filter(|operation| operation.phase == phase) {
        push_first_stage_patch_plan_operation(script, operation, full_source_bound)?;
    }
    Ok(())
}

fn push_first_stage_patch_plan_operation(
    script: &mut String,
    operation: &RustBootstrapPatchOperation,
    full_source_bound: bool,
) -> Result<(), RustSourceProviderError> {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    match operation.kind {
        RustBootstrapPatchOperationKind::MinicargoBuildOutDir => push_first_stage_minicargo_build_out_dir_patch(script),
        RustBootstrapPatchOperationKind::MinicargoRustcThreads => {
            push_first_stage_minicargo_rustc_threads_patch(script)
        }
        RustBootstrapPatchOperationKind::MinicargoProtectedExecutionPaths => {
            push_first_stage_minicargo_protected_execution_paths(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::MinicargoLlvmStaticArchiveTargets => {
            push_first_stage_minicargo_llvm_backtrace_patch(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::SourceRootMuslLlvmRuntime => {
            push_first_stage_musl_host_llvm_runtime(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::OpenSslNoAsm => {
            push_first_stage_openssl_no_asm_patch(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::RustExplicitSysroot => push_first_stage_rustc_explicit_sysroot_patch(script),
        RustBootstrapPatchOperationKind::RustcDriverRlib => push_first_stage_musl_rustc_driver_rlib_patch(script),
        RustBootstrapPatchOperationKind::RunRustcHostRuntime => {
            push_first_stage_run_rustc_host(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::RunRustcTargetRustlib => {
            push_first_stage_run_rustc_target(script, full_source_bound)
        }
        RustBootstrapPatchOperationKind::ProviderContractAssertion => {}
        unsupported => {
            return Err(RustSourceProviderError::Validate(format!(
                "unsupported first-stage script patch operation {unsupported:?}"
            )));
        }
    }
    Ok(())
}

fn push_first_stage_minicargo_build_out_dir_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("printf '%s\\n' 'normalizing minicargo build-script OUT_DIR for static musl compiler host'\n");
    script.push_str(&format!("minicargo_build_source={FIRST_STAGE_MINICARGO_BUILD_SOURCE}\n"));
    script.push_str(&format!(
        "if [ ! -f \"$minicargo_build_source\" ]; then printf '%s\\n' 'minicargo build.cpp missing before musl host OUT_DIR normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("minicargo_out_dir_tmp=\"$BUILD_DIR/minicargo-build.cpp\"\n");
    script.push_str(&format!(
        "minicargo_out_dir_original={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_OUT_DIR_ORIGINAL_LINE)
    ));
    script.push_str(&format!("minicargo_out_dir_marker={}\n", shell_quote(FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_MARKER)));
    script
        .push_str(&format!("minicargo_out_dir_patch_line={}\n", shell_quote(FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_LINE)));
    script.push_str("minicargo_out_dir_replaced=false\n");
    script.push_str("minicargo_out_dir_seen=false\n");
    script.push_str("minicargo_out_dir_pending=false\n");
    script.push_str(": > \"$minicargo_out_dir_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$minicargo_out_dir_pending\" = true ]; then\n");
    script.push_str("    if [ \"$line\" = \"$minicargo_out_dir_marker\" ]; then\n");
    script.push_str("      minicargo_out_dir_seen=true\n");
    script.push_str("    else\n");
    script.push_str("      printf '%s\\n' \"$minicargo_out_dir_marker\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("      printf '%s\\n' \"$minicargo_out_dir_patch_line\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("      printf '%s\\n' '    }' >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("      minicargo_out_dir_replaced=true\n");
    script.push_str("    fi\n");
    script.push_str("    minicargo_out_dir_pending=false\n");
    script.push_str("  fi\n");
    script.push_str("  if [ \"$line\" = \"$minicargo_out_dir_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("    minicargo_out_dir_pending=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$minicargo_build_source\"\n");
    script.push_str("if [ \"$minicargo_out_dir_pending\" = true ]; then\n");
    script.push_str("  printf '%s\\n' \"$minicargo_out_dir_marker\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("  printf '%s\\n' \"$minicargo_out_dir_patch_line\" >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("  printf '%s\\n' '    }' >> \"$minicargo_out_dir_tmp\"\n");
    script.push_str("  minicargo_out_dir_replaced=true\n");
    script.push_str("fi\n");
    script.push_str(&format!(
        "if [ \"$minicargo_out_dir_replaced\" = false ] && [ \"$minicargo_out_dir_seen\" = false ]; then printf '%s\\n' 'minicargo build.cpp lacks expected OUT_DIR line for musl host normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$minicargo_out_dir_tmp\" \"$minicargo_build_source\"\n");
    script.push_str("rm -f \"$minicargo_out_dir_tmp\"\n");
    script.push_str("fi\n");
}

fn first_stage_minicargo_makefile_llvm_config_build_patched_line() -> String {
    format!(
        "{FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_PATCHED_PREFIX} {FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_STATIC_ARCHIVE_TARGETS}"
    )
}

fn push_first_stage_minicargo_protected_execution_paths(script: &mut String, full_source_bound: bool) {
    let original_len = script.len();
    debug_assert!(!script.contains('\0'));
    if !full_source_bound {
        return;
    }
    script.push_str("printf '%s\\n' 'binding minicargo and Make recipes to protected absolute executables'\n");
    script.push_str("minicargo_makefile=minicargo.mk\n");
    script.push_str("minicargo_exec_tmp=\"$BUILD_DIR/minicargo-protected-exec.mk\"\n");
    script.push_str(&format!(
        "minicargo_exec_original={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "minicargo_exec_patched={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_PROTECTED_LINE)
    ));
    script.push_str("minicargo_exec_replaced=false\n");
    script.push_str("minicargo_exec_seen=false\n");
    script.push_str(": > \"$minicargo_exec_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$minicargo_exec_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$minicargo_exec_patched\" >> \"$minicargo_exec_tmp\"\n");
    script.push_str("    minicargo_exec_replaced=true\n");
    script.push_str("  else\n");
    script.push_str("    if [ \"$line\" = \"$minicargo_exec_patched\" ]; then minicargo_exec_seen=true; fi\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$minicargo_exec_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$minicargo_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$minicargo_exec_replaced\" = false ] && [ \"$minicargo_exec_seen\" = false ]; then printf '%s\\n' 'minicargo Makefile lacks the expected protected executable variable' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$minicargo_exec_tmp\" \"$minicargo_makefile\"\n");
    script.push_str("rm -f \"$minicargo_exec_tmp\"\n");
    push_first_stage_run_rustc_minicargo_absolute_patch(script);
    push_first_stage_mrustc_bound_shell_patch(script);
    push_first_stage_mrustc_proc_macro_absolute_patch(script);
    push_first_stage_llvm_rule_normalizer(script);
    script.push_str("RECEIPT_MAKE_PROGRAM=\"$MAKE_PROGRAM\"\n");
    script.push_str("receipt_make_dir=\"$BUILD_DIR/receipt-make-bin\"\n");
    script.push_str("mkdir -p \"$receipt_make_dir\"\n");
    script.push_str("MAKE_PROGRAM=\"$receipt_make_dir/make\"\n");
    script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$MAKE_PROGRAM\"\n");
    script.push_str(
        "printf '%s\\n' \"exec \\\"$RECEIPT_MAKE_PROGRAM\\\" \\\"SHELL=$SHELL_PROGRAM\\\" \\\"\\$@\\\"\" >> \"$MAKE_PROGRAM\"\n",
    );
    script.push_str("chmod +x \"$MAKE_PROGRAM\"\n");
    script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$receipt_make_dir/sh\"\n");
    script.push_str("printf '%s\\n' \"exec \\\"$SHELL_PROGRAM\\\" \\\"\\$@\\\"\" >> \"$receipt_make_dir/sh\"\n");
    script.push_str("chmod +x \"$receipt_make_dir/sh\"\n");
    script.push_str("PATH=\"$receipt_make_dir:$PATH\"\n");
    script.push_str("MRUSTC_SHELL=\"$SHELL_PROGRAM\"\n");
    script.push_str("export PATH MAKE_PROGRAM RECEIPT_MAKE_PROGRAM MRUSTC_SHELL MANTLE_LLVM_RULE_NORMALIZER\n");
    assert!(script.len() > original_len);
    assert!(script[original_len..].contains(FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_PROTECTED_LINE));
    assert!(script[original_len..].contains("SHELL=$SHELL_PROGRAM"));
}

fn push_first_stage_run_rustc_minicargo_absolute_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("run_rustc_makefile=run_rustc/Makefile\n");
    script.push_str("run_rustc_minicargo_tmp=\"$BUILD_DIR/run-rustc-minicargo.mk\"\n");
    script.push_str(&format!(
        "run_rustc_minicargo_original={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_MINICARGO_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "run_rustc_minicargo_patched={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_MINICARGO_PROTECTED_LINE)
    ));
    script.push_str("run_rustc_minicargo_replaced=false\nrun_rustc_minicargo_seen=false\n");
    script.push_str(": > \"$run_rustc_minicargo_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_minicargo_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_minicargo_patched\" >> \"$run_rustc_minicargo_tmp\"\n");
    script.push_str("    run_rustc_minicargo_replaced=true\n  else\n");
    script
        .push_str("    if [ \"$line\" = \"$run_rustc_minicargo_patched\" ]; then run_rustc_minicargo_seen=true; fi\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_minicargo_tmp\"\n  fi\n");
    script.push_str("done < \"$run_rustc_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_minicargo_replaced\" = false ] && [ \"$run_rustc_minicargo_seen\" = false ]; then printf '%s\\n' 'run_rustc Makefile lacks the expected protected minicargo executable' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_minicargo_tmp\" \"$run_rustc_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_minicargo_tmp\"\n");
    assert!(script.contains(FIRST_STAGE_RUN_RUSTC_MINICARGO_PROTECTED_LINE));
    assert!(script.contains("run_rustc Makefile lacks the expected protected minicargo executable"));
}

fn push_first_stage_run_rustc_stage_tool_absolute_patch(script: &mut String, full_source_bound: bool) {
    let original_len = script.len();
    debug_assert!(!script.contains('\0'));
    if !full_source_bound {
        return;
    }
    push_first_stage_run_rustc_stage_tool_bindings(script);
    push_first_stage_run_rustc_stage_tool_rewrite(script);
    push_first_stage_run_rustc_stage_tool_validation(script);
    push_first_stage_run_rustc_final_smoke_patch(script);
    push_first_stage_run_rustc_final_smoke_exec_patch(script);
    assert!(script.len() > original_len);
    assert!(script[original_len..].contains(FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_PROTECTED_TOKEN));
    assert!(script[original_len..].contains(FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_PROTECTED_TOKEN));
    assert!(script[original_len..].contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_PROTECTED_LINE));
    assert!(script[original_len..].contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_PROTECTED_LINE));
}

fn push_first_stage_run_rustc_stage_tool_bindings(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("printf '%s\\n' 'binding run_rustc stage tools to protected absolute executables'\n");
    script.push_str("run_rustc_stage_tool_makefile=run_rustc/Makefile\n");
    script.push_str("run_rustc_stage_tool_tmp=\"$BUILD_DIR/run-rustc-stage-tools.mk\"\n");
    script.push_str("run_rustc_stage_tool_tab=$(printf '\\t')\n");
    for (name, token) in [
        ("rustc_original", FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_ORIGINAL_TOKEN),
        ("rustc_patched", FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_PROTECTED_TOKEN),
        ("cargo_original", FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_ORIGINAL_TOKEN),
        ("cargo_patched", FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_PROTECTED_TOKEN),
    ] {
        script.push_str(&format!("run_rustc_stage_{name}={}\n", shell_quote(token)));
    }
    script.push_str("run_rustc_stage_rustc_seen=false\nrun_rustc_stage_cargo_seen=false\n");
    assert!(script.contains("run_rustc_stage_rustc_original"));
    assert!(script.contains("run_rustc_stage_cargo_patched"));
}

fn push_first_stage_run_rustc_stage_tool_rewrite(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str(": > \"$run_rustc_stage_tool_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  case \"$line\" in\n");
    script.push_str("    \"$run_rustc_stage_tool_tab\"*)\n");
    script.push_str("      case \"$line\" in\n");
    push_first_stage_run_rustc_stage_tool_rewrite_arm(script, "rustc");
    push_first_stage_run_rustc_stage_tool_rewrite_arm(script, "cargo");
    script.push_str("        *) printf '%s\\n' \"$line\" >> \"$run_rustc_stage_tool_tmp\" ;;\n");
    script.push_str("      esac\n      ;;\n");
    script.push_str("    *) printf '%s\\n' \"$line\" >> \"$run_rustc_stage_tool_tmp\" ;;\n");
    script.push_str("  esac\n");
    script.push_str("done < \"$run_rustc_stage_tool_makefile\"\n");
    assert!(script.contains("run_rustc_stage_prefix"));
    assert!(script.contains("run_rustc_stage_suffix"));
}

fn push_first_stage_run_rustc_stage_tool_rewrite_arm(script: &mut String, tool: &str) {
    debug_assert!(tool == "rustc" || tool == "cargo");
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("        *\"$run_rustc_stage_{tool}_original\"*)\n"));
    script.push_str(&format!("          run_rustc_stage_prefix=${{line%%\"$run_rustc_stage_{tool}_original\"*}}\n"));
    script.push_str(&format!("          run_rustc_stage_suffix=${{line#*\"$run_rustc_stage_{tool}_original\"}}\n"));
    script.push_str(&format!(
        "          case \"$run_rustc_stage_suffix\" in *\"$run_rustc_stage_{tool}_original\"*) printf '%s\\n' 'run_rustc {tool} recipe contains repeated executable tokens' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE} ;; esac\n"
    ));
    script.push_str(&format!(
        "          printf '%s%s%s\\n' \"$run_rustc_stage_prefix\" \"$run_rustc_stage_{tool}_patched\" \"$run_rustc_stage_suffix\" >> \"$run_rustc_stage_tool_tmp\"\n"
    ));
    script.push_str(&format!("          run_rustc_stage_{tool}_seen=true\n          ;;\n"));
    assert!(script.contains(&format!("run_rustc_stage_{tool}_seen=true")));
}

fn push_first_stage_run_rustc_stage_tool_validation(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    for tool in ["rustc", "cargo"] {
        script.push_str(&format!(
            "if [ \"$run_rustc_stage_{tool}_seen\" = false ]; then printf '%s\\n' 'run_rustc Makefile lacks a protected stage {tool} recipe' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
        ));
    }
    script.push_str("$COPY_PROGRAM \"$run_rustc_stage_tool_tmp\" \"$run_rustc_stage_tool_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_stage_tool_tmp\"\n");
    assert!(script.contains("lacks a protected stage rustc recipe"));
    assert!(script.contains("lacks a protected stage cargo recipe"));
}

fn push_first_stage_run_rustc_final_smoke_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("run_rustc_final_smoke_tmp=\"$BUILD_DIR/run-rustc-final-smoke.mk\"\n");
    script.push_str(&format!(
        "run_rustc_final_smoke_original={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "run_rustc_final_smoke_protected={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_PROTECTED_LINE)
    ));
    script.push_str("run_rustc_final_smoke_replaced=false\nrun_rustc_final_smoke_seen=false\n");
    script.push_str(": > \"$run_rustc_final_smoke_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_final_smoke_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_final_smoke_protected\" >> \"$run_rustc_final_smoke_tmp\"\n");
    script.push_str("    run_rustc_final_smoke_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_smoke_protected\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_final_smoke_tmp\"\n");
    script.push_str("    run_rustc_final_smoke_seen=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_final_smoke_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_stage_tool_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_final_smoke_replaced\" = false ] && [ \"$run_rustc_final_smoke_seen\" = false ]; then printf '%s\\n' 'run_rustc Makefile lacks the protected final rustc smoke recipe' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_final_smoke_tmp\" \"$run_rustc_stage_tool_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_final_smoke_tmp\"\n");
    assert!(script.contains("run_rustc_final_smoke_replaced=true"));
    assert!(script.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_PROTECTED_LINE));
}

fn push_first_stage_run_rustc_final_smoke_exec_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("run_rustc_final_smoke_exec_tmp=\"$BUILD_DIR/run-rustc-final-smoke-exec.mk\"\n");
    script.push_str(&format!(
        "run_rustc_final_smoke_exec_original={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "run_rustc_final_smoke_exec_protected={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_PROTECTED_LINE)
    ));
    script.push_str("run_rustc_final_smoke_exec_replaced=false\nrun_rustc_final_smoke_exec_seen=false\n");
    script.push_str(": > \"$run_rustc_final_smoke_exec_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_final_smoke_exec_original\" ]; then\n");
    script.push_str(
        "    printf '%s\\n' \"$run_rustc_final_smoke_exec_protected\" >> \"$run_rustc_final_smoke_exec_tmp\"\n",
    );
    script.push_str("    run_rustc_final_smoke_exec_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_smoke_exec_protected\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_final_smoke_exec_tmp\"\n");
    script.push_str("    run_rustc_final_smoke_exec_seen=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_final_smoke_exec_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_stage_tool_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_final_smoke_exec_replaced\" = false ] && [ \"$run_rustc_final_smoke_exec_seen\" = false ]; then printf '%s\\n' 'run_rustc Makefile lacks the protected final smoke execution' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_final_smoke_exec_tmp\" \"$run_rustc_stage_tool_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_final_smoke_exec_tmp\"\n");
    assert!(script.contains("run_rustc_final_smoke_exec_replaced=true"));
    assert!(script.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_PROTECTED_LINE));
}

fn push_first_stage_mrustc_bound_shell_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    let replacement = [
        "                // Mantle: execute compiler commands through the receipt-bound shell.",
        "                int ec = 0;",
        "                const char* mantle_shell = getenv(\"MRUSTC_SHELL\");",
        "                if(mantle_shell != nullptr && mantle_shell[0] != '\\0')",
        "                {",
        "                    pid_t child_pid = ::fork();",
        "                    if(child_pid == -1)",
        "                    {",
        "                        ec = -1;",
        "                    }",
        "                    else if(child_pid == 0)",
        "                    {",
        "                        ::execl(mantle_shell, mantle_shell, \"-c\", cmd_ss.str().c_str(), static_cast<char*>(nullptr));",
        "                        ::_exit(127);",
        "                    }",
        "                    else",
        "                    {",
        "                        int child_status = 0;",
        "                        pid_t wait_result;",
        "                        do",
        "                        {",
        "                            wait_result = ::waitpid(child_pid, &child_status, 0);",
        "                        } while(wait_result == -1 && errno == EINTR);",
        "                        ec = wait_result == -1 ? -1 : child_status;",
        "                    }",
        "                }",
        "                else",
        "                {",
        "                    ec = system(cmd_ss.str().c_str());",
        "                }",
    ];
    script.push_str(&format!("mrustc_codegen_source={FIRST_STAGE_MRUSTC_CODEGEN_C_SOURCE}\n"));
    script.push_str("mrustc_codegen_tmp=\"$BUILD_DIR/codegen-c-bound-shell.cpp\"\n");
    script.push_str("mrustc_system_matches=0\n");
    script.push_str(": > \"$mrustc_codegen_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str(&format!("  if [ \"$line\" = {} ]; then\n", shell_quote(FIRST_STAGE_MRUSTC_SYSTEM_ORIGINAL_LINE)));
    for line in replacement {
        script.push_str(&format!("    printf '%s\\n' {} >> \"$mrustc_codegen_tmp\"\n", shell_quote(line)));
    }
    script.push_str("    mrustc_system_matches=$((mrustc_system_matches + 1))\n");
    script.push_str("  elif [ \"$line\" = '#include <cmath>' ]; then\n");
    for line in [
        "#include <cmath>",
        "#include <cerrno>",
        "#include <cstdlib>",
        "#include <sys/wait.h>",
        "#include <unistd.h>",
    ] {
        script.push_str(&format!("    printf '%s\\n' {} >> \"$mrustc_codegen_tmp\"\n", shell_quote(line)));
    }
    script.push_str("  else\n    printf '%s\\n' \"$line\" >> \"$mrustc_codegen_tmp\"\n  fi\n");
    script.push_str("done < \"$mrustc_codegen_source\"\n");
    script.push_str(&format!(
        "if [ \"$mrustc_system_matches\" -ne 1 ]; then printf '%s\\n' 'mrustc codegen C source lacks one expected system call' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$mrustc_codegen_tmp\" \"$mrustc_codegen_source\"\n");
    script.push_str("rm -f \"$mrustc_codegen_tmp\"\n");
}

fn push_first_stage_mrustc_proc_macro_absolute_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    let argv_replacement = [
        "    char resolved_executable[PATH_MAX];",
        "    const char* spawn_executable = executable;",
        "    if(executable[0] != '/')",
        "    {",
        "        if(realpath(executable, resolved_executable) == nullptr)",
        "        {",
        "            BUG(sp, \"Unable to resolve proc-macro executable `\" << executable << \"`, \" << strerror(errno));",
        "        }",
        "        spawn_executable = resolved_executable;",
        "    }",
        "    char*   argv[3] = { const_cast<char*>(spawn_executable), const_cast<char*>(proc_macro_desc.name.c_str()), nullptr };",
    ];
    script.push_str(&format!("mrustc_proc_macro_source={FIRST_STAGE_MRUSTC_PROC_MACRO_SOURCE}\n"));
    script.push_str("mrustc_proc_macro_tmp=\"$BUILD_DIR/proc-macro-absolute.cpp\"\n");
    script.push_str("mrustc_proc_macro_argv_matches=0\n");
    script.push_str("mrustc_proc_macro_spawn_matches=0\n");
    script.push_str(": > \"$mrustc_proc_macro_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = '#include <unordered_set>' ]; then\n");
    for line in ["#include <unordered_set>", "#include <limits.h>", "#include <stdlib.h>"] {
        script.push_str(&format!("    printf '%s\\n' {} >> \"$mrustc_proc_macro_tmp\"\n", shell_quote(line)));
    }
    script.push_str(&format!(
        "  elif [ \"$line\" = {} ]; then\n",
        shell_quote(FIRST_STAGE_MRUSTC_PROC_MACRO_ARGV_ORIGINAL_LINE)
    ));
    for line in argv_replacement {
        script.push_str(&format!("    printf '%s\\n' {} >> \"$mrustc_proc_macro_tmp\"\n", shell_quote(line)));
    }
    script.push_str("    mrustc_proc_macro_argv_matches=$((mrustc_proc_macro_argv_matches + 1))\n");
    script.push_str(&format!(
        "  elif [ \"$line\" = {} ]; then\n",
        shell_quote(FIRST_STAGE_MRUSTC_PROC_MACRO_SPAWN_ORIGINAL_LINE)
    ));
    script.push_str(
        "    printf '%s\\n' '    int rv = posix_spawn(&this->handles.child_pid, spawn_executable, &file_actions, nullptr, argv, environ);' >> \"$mrustc_proc_macro_tmp\"\n",
    );
    script.push_str("    mrustc_proc_macro_spawn_matches=$((mrustc_proc_macro_spawn_matches + 1))\n");
    script.push_str("  else\n    printf '%s\\n' \"$line\" >> \"$mrustc_proc_macro_tmp\"\n  fi\n");
    script.push_str("done < \"$mrustc_proc_macro_source\"\n");
    script.push_str(&format!(
        "if [ \"$mrustc_proc_macro_argv_matches\" -ne 1 ] || [ \"$mrustc_proc_macro_spawn_matches\" -ne 1 ]; then printf '%s\\n' 'mrustc proc-macro source lacks one expected spawn boundary' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$mrustc_proc_macro_tmp\" \"$mrustc_proc_macro_source\"\n");
    script.push_str("rm -f \"$mrustc_proc_macro_tmp\"\n");
}

fn push_first_stage_llvm_rule_normalizer(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("MANTLE_LLVM_RULE_NORMALIZER=\"$BUILD_DIR/normalize-llvm-rules\"\n");
    script.push_str("printf '%s\\n' \"#!$SHELL_PROGRAM\" > \"$MANTLE_LLVM_RULE_NORMALIZER\"\n");
    let normalizer_lines = vec![
        "set -eu".to_string(),
        "build_root=$1".to_string(),
        "busybox_dir=${SHELL_PROGRAM%/sh}".to_string(),
        "file_list=$build_root/.mantle-build-makefiles".to_string(),
        "files_seen=0".to_string(),
        "replacements=0".to_string(),
        format!("files_max={FIRST_STAGE_LLVM_RULE_FILES_MAX}"),
        format!("replacements_max={FIRST_STAGE_LLVM_RULE_REPLACEMENTS_MAX}"),
        "\"$busybox_dir/find\" \"$build_root\" -name build.make -type f > \"$file_list\"".to_string(),
        "while IFS= read -r file || [ -n \"$file\" ]; do".to_string(),
        "  files_seen=$((files_seen + 1))".to_string(),
        "  if [ \"$files_seen\" -gt \"$files_max\" ]; then echo 'LLVM Makefile count exceeds bound' >&2; exit 3; fi".to_string(),
        "  tmp=$file.mantle-absolute-exec".to_string(),
        "  : > \"$tmp\"".to_string(),
        "  while IFS= read -r line || [ -n \"$line\" ]; do".to_string(),
        "    for tool_name in llvm-min-tblgen llvm-tblgen; do".to_string(),
        "      for relative_prefix in ../../../../bin ../../../bin; do".to_string(),
        "        needle=$relative_prefix/$tool_name".to_string(),
        "        case \"$line\" in".to_string(),
        "        *\"$needle\"*)".to_string(),
        "          prefix=${line%%${needle}*}".to_string(),
        "          suffix=${line#*${needle}}".to_string(),
        "          line=${prefix}${build_root}/bin/${tool_name}${suffix}".to_string(),
        "          replacements=$((replacements + 1))".to_string(),
        "          if [ \"$replacements\" -gt \"$replacements_max\" ]; then echo 'LLVM rule replacement count exceeds bound' >&2; exit 3; fi".to_string(),
        "          ;;".to_string(),
        "        esac".to_string(),
        "      done".to_string(),
        "    done".to_string(),
        "    printf '%s\\n' \"$line\" >> \"$tmp\"".to_string(),
        "  done < \"$file\"".to_string(),
        "  \"$busybox_dir/mv\" \"$tmp\" \"$file\"".to_string(),
        "done < \"$file_list\"".to_string(),
        "\"$busybox_dir/rm\" -f \"$file_list\"".to_string(),
        "if [ \"$replacements\" -eq 0 ]; then echo 'LLVM generated rules contain no bounded tblgen paths' >&2; exit 3; fi".to_string(),
    ];
    for line in normalizer_lines {
        script.push_str(&format!("printf '%s\\n' {} >> \"$MANTLE_LLVM_RULE_NORMALIZER\"\n", shell_quote(&line)));
    }
    script.push_str("chmod +x \"$MANTLE_LLVM_RULE_NORMALIZER\"\n");
}

fn push_first_stage_minicargo_llvm_backtrace_patch(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("printf '%s\\n' 'disabling LLVM shared-tool and execinfo backtraces for source-root musl host'\n");
    script.push_str(&format!("minicargo_makefile={FIRST_STAGE_MINICARGO_MAKEFILE}\n"));
    script.push_str(&format!(
        "if [ ! -f \"$minicargo_makefile\" ]; then printf '%s\\n' 'minicargo Makefile missing before LLVM normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("minicargo_llvm_tmp=\"$BUILD_DIR/minicargo-llvm-backtrace.mk\"\n");
    script.push_str("minicargo_llvm_options_replaced=false\n");
    script.push_str("minicargo_llvm_options_seen_patched=false\n");
    script.push_str("minicargo_llvm_config_replaced=false\n");
    script.push_str("minicargo_llvm_config_seen_patched=false\n");
    if full_source_bound {
        script.push_str("minicargo_cmake_command_replaced=false\n");
        script.push_str("minicargo_cmake_command_seen_patched=false\n");
    }
    script.push_str(&format!(
        "minicargo_llvm_options_original={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "minicargo_llvm_options_patched={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_PATCHED_LINE)
    ));
    script.push_str(&format!(
        "minicargo_llvm_config_original={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_ORIGINAL_LINE)
    ));
    let minicargo_llvm_config_patched = first_stage_minicargo_makefile_llvm_config_build_patched_line();
    script.push_str(&format!("minicargo_llvm_config_patched={}\n", shell_quote(&minicargo_llvm_config_patched)));
    if full_source_bound {
        script.push_str(&format!(
            "minicargo_cmake_command_original={}\n",
            shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_ORIGINAL_LINE)
        ));
        script.push_str(&format!(
            "minicargo_cmake_command_patched={}\n",
            shell_quote(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_PROTECTED_LINE)
        ));
    }
    script.push_str("while IFS= read -r line; do\n");
    script.push_str("  if [ \"$line\" = \"$minicargo_llvm_options_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$minicargo_llvm_options_patched\" >> \"$minicargo_llvm_tmp\"\n");
    script.push_str("    minicargo_llvm_options_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$minicargo_llvm_config_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$minicargo_llvm_config_patched\" >> \"$minicargo_llvm_tmp\"\n");
    script.push_str("    minicargo_llvm_config_replaced=true\n");
    if full_source_bound {
        script.push_str("  elif [ \"$line\" = \"$minicargo_cmake_command_original\" ]; then\n");
        script.push_str("    printf '%s\\n' \"$minicargo_cmake_command_patched\" >> \"$minicargo_llvm_tmp\"\n");
        script.push_str("    minicargo_cmake_command_replaced=true\n");
    }
    script.push_str("  else\n");
    script.push_str("    if [ \"$line\" = \"$minicargo_llvm_options_patched\" ]; then minicargo_llvm_options_seen_patched=true; fi\n");
    script.push_str(
        "    if [ \"$line\" = \"$minicargo_llvm_config_patched\" ]; then minicargo_llvm_config_seen_patched=true; fi\n",
    );
    if full_source_bound {
        script.push_str(
            "    if [ \"$line\" = \"$minicargo_cmake_command_patched\" ]; then minicargo_cmake_command_seen_patched=true; fi\n",
        );
    }
    script.push_str("    printf '%s\\n' \"$line\" >> \"$minicargo_llvm_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$minicargo_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$minicargo_llvm_options_replaced\" = false ] && [ \"$minicargo_llvm_options_seen_patched\" = false ]; then printf '%s\\n' 'minicargo Makefile lacks expected LLVM CMake options line for musl normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$minicargo_llvm_config_replaced\" = false ] && [ \"$minicargo_llvm_config_seen_patched\" = false ]; then printf '%s\\n' 'minicargo Makefile lacks expected llvm-config build line for musl normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    if full_source_bound {
        script.push_str(&format!(
            "if [ \"$minicargo_cmake_command_replaced\" = false ] && [ \"$minicargo_cmake_command_seen_patched\" = false ]; then printf '%s\\n' 'minicargo Makefile lacks expected LLVM configure command for protected rule normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
        ));
    }
    script.push_str("$COPY_PROGRAM \"$minicargo_llvm_tmp\" \"$minicargo_makefile\"\n");
    script.push_str("rm -f \"$minicargo_llvm_tmp\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_minicargo_rustc_threads_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("printf '%s\\n' 'normalizing minicargo rustc worker threads for static musl compiler host'\n");
    script.push_str(&format!("minicargo_threads_source={FIRST_STAGE_MINICARGO_BUILD_SOURCE}\n"));
    script.push_str(&format!(
        "if [ ! -f \"$minicargo_threads_source\" ]; then printf '%s\\n' 'minicargo build.cpp missing before rustc thread normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("minicargo_threads_tmp=\"$BUILD_DIR/minicargo-rustc-threads-build.cpp\"\n");
    script.push_str(&format!(
        "minicargo_threads_anchor={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_RUSTC_FORCE_UNSTABLE_LINE)
    ));
    script.push_str(&format!(
        "minicargo_threads_marker_line={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_RUSTC_THREADS_MARKER_LINE)
    ));
    script.push_str(&format!(
        "minicargo_threads_flag_line={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_RUSTC_THREADS_FLAG_LINE)
    ));
    script.push_str(&format!(
        "minicargo_threads_value_line={}\n",
        shell_quote(FIRST_STAGE_MINICARGO_RUSTC_THREADS_VALUE_LINE)
    ));
    script.push_str("minicargo_threads_inserted=false\n");
    script.push_str("minicargo_threads_pending=false\n");
    script.push_str("minicargo_threads_seen=false\n");
    script.push_str(": > \"$minicargo_threads_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$minicargo_threads_pending\" = true ]; then\n");
    script.push_str("    if [ \"$line\" = \"$minicargo_threads_marker_line\" ]; then\n");
    script.push_str("      minicargo_threads_seen=true\n");
    script.push_str("    else\n");
    script.push_str("      printf '%s\\n' \"$minicargo_threads_marker_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("      printf '%s\\n' \"$minicargo_threads_flag_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("      printf '%s\\n' \"$minicargo_threads_value_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("      minicargo_threads_inserted=true\n");
    script.push_str("    fi\n");
    script.push_str("    minicargo_threads_pending=false\n");
    script.push_str("  fi\n");
    script.push_str("  printf '%s\\n' \"$line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("  if [ \"$line\" = \"$minicargo_threads_anchor\" ]; then minicargo_threads_pending=true; fi\n");
    script.push_str("done < \"$minicargo_threads_source\"\n");
    script.push_str("if [ \"$minicargo_threads_pending\" = true ]; then\n");
    script.push_str("  printf '%s\\n' \"$minicargo_threads_marker_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("  printf '%s\\n' \"$minicargo_threads_flag_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("  printf '%s\\n' \"$minicargo_threads_value_line\" >> \"$minicargo_threads_tmp\"\n");
    script.push_str("  minicargo_threads_inserted=true\n");
    script.push_str("fi\n");
    script.push_str(&format!(
        "if [ \"$minicargo_threads_inserted\" = false ] && [ \"$minicargo_threads_seen\" = false ]; then printf '%s\\n' 'minicargo build.cpp lacks expected rustc force-unstable line for thread normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$minicargo_threads_tmp\" \"$minicargo_threads_source\"\n");
    script.push_str("rm -f \"$minicargo_threads_tmp\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_openssl_no_asm_patch(script: &mut String, full_source_bound: bool) {
    debug_assert!(!script.contains('\0'));
    if !full_source_bound {
        return;
    }
    script.push_str("printf '%s\\n' 'disabling OpenSSL assembly generators for protected source bootstrap'\n");
    script.push_str("set -- rustc-${RUSTC_VERSION}-src/vendor/openssl-src-300.*/src/lib.rs\n");
    script.push_str(&format!(
        "if [ \"$#\" -ne 1 ] || [ ! -f \"$1\" ]; then printf '%s\\n' 'vendored OpenSSL source configuration is not unique' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("openssl_src_config=$1\n");
    script.push_str("openssl_src_tmp=\"$BUILD_DIR/openssl-src-no-asm.rs\"\n");
    script.push_str("openssl_no_asm_matches=0\n");
    script.push_str(": > \"$openssl_src_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str(
        "  if [ \"$line\" = '        // On Android it looks like not passing no-stdio may cause a build' ]; then\n",
    );
    script.push_str("    printf '%s\\n' '        // Mantle: avoid Perl assembly pipes that require ambient /bin/sh.' >> \"$openssl_src_tmp\"\n");
    script.push_str("    printf '%s\\n' '        configure.arg(\"no-asm\");' >> \"$openssl_src_tmp\"\n");
    script.push_str("    openssl_no_asm_matches=$((openssl_no_asm_matches + 1))\n");
    script.push_str("  fi\n");
    script.push_str("  printf '%s\\n' \"$line\" >> \"$openssl_src_tmp\"\n");
    script.push_str("done < \"$openssl_src_config\"\n");
    script.push_str(&format!(
        "if [ \"$openssl_no_asm_matches\" -ne 1 ]; then printf '%s\\n' 'vendored OpenSSL configuration lacks one expected insertion point' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$openssl_src_tmp\" \"$openssl_src_config\"\n");
    script.push_str("rm -f \"$openssl_src_tmp\"\n");
}

fn push_first_stage_rustc_explicit_sysroot_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("printf '%s\\n' 'normalizing Rust explicit sysroot handling for static musl rustc'\n");
    script.push_str(&format!("rustc_config_source={FIRST_STAGE_RUSTC_CONFIG_SOURCE}\n"));
    script.push_str(&format!(
        "if [ ! -f \"$rustc_config_source\" ]; then printf '%s\\n' 'rustc_session config.rs missing before sysroot normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("rustc_config_tmp=\"$BUILD_DIR/rustc-session-config-sysroot.rs\"\n");
    script.push_str(&format!(
        "rustc_sysroot_original_line={}\n",
        shell_quote(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_ORIGINAL_LINE)
    ));
    script.push_str(&format!(
        "rustc_sysroot_replacement_line_1={}\n",
        shell_quote(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_1)
    ));
    script.push_str(&format!(
        "rustc_sysroot_replacement_line_2={}\n",
        shell_quote(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_2)
    ));
    script.push_str(&format!(
        "rustc_sysroot_replacement_line_3={}\n",
        shell_quote(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_3)
    ));
    script.push_str(&format!(
        "rustc_sysroot_replacement_line_4={}\n",
        shell_quote(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_4)
    ));
    script.push_str("rustc_sysroot_replaced=false\n");
    script.push_str("rustc_sysroot_seen=false\n");
    script.push_str(": > \"$rustc_config_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$rustc_sysroot_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$rustc_sysroot_replacement_line_1\" >> \"$rustc_config_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$rustc_sysroot_replacement_line_2\" >> \"$rustc_config_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$rustc_sysroot_replacement_line_3\" >> \"$rustc_config_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$rustc_sysroot_replacement_line_4\" >> \"$rustc_config_tmp\"\n");
    script.push_str("    rustc_sysroot_replaced=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$rustc_config_tmp\"\n");
    script.push_str("    if [ \"$line\" = \"$rustc_sysroot_replacement_line_1\" ]; then rustc_sysroot_seen=true; fi\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$rustc_config_source\"\n");
    script.push_str(&format!(
        "if [ \"$rustc_sysroot_replaced\" = false ] && [ \"$rustc_sysroot_seen\" = false ]; then printf '%s\\n' 'rustc_session Sysroot::new no longer has expected default_sysroot shape' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$rustc_config_tmp\" \"$rustc_config_source\"\n");
    script.push_str("rm -f \"$rustc_config_tmp\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_musl_rustc_driver_rlib_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"));
    script.push_str("printf '%s\\n' 'normalizing rustc_driver crate type for static musl compiler host'\n");
    script.push_str(&format!("rustc_driver_manifest={FIRST_STAGE_RUSTC_DRIVER_MANIFEST}\n"));
    script.push_str(&format!(
        "if [ ! -f \"$rustc_driver_manifest\" ]; then printf '%s\\n' 'rustc_driver manifest missing before musl host normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("rustc_driver_manifest_tmp=\"$BUILD_DIR/rustc-driver-Cargo.toml\"\n");
    script.push_str("rustc_driver_replaced=false\n");
    script.push_str("rustc_driver_seen_rlib=false\n");
    script.push_str(": > \"$rustc_driver_manifest_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  case \"$line\" in\n");
    script.push_str(&format!(
        "    {dylib}) printf '%s\\n' {rlib} >> \"$rustc_driver_manifest_tmp\"; rustc_driver_replaced=true ;;\n",
        dylib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_DYLIB_CRATE_TYPE_LINE),
        rlib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE),
    ));
    script.push_str(&format!(
        "    {rlib}) printf '%s\\n' \"$line\" >> \"$rustc_driver_manifest_tmp\"; rustc_driver_seen_rlib=true ;;\n",
        rlib = shell_quote(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE),
    ));
    script.push_str("    *) printf '%s\\n' \"$line\" >> \"$rustc_driver_manifest_tmp\" ;;\n");
    script.push_str("  esac\n");
    script.push_str("done < \"$rustc_driver_manifest\"\n");
    script.push_str(&format!(
        "if [ \"$rustc_driver_replaced\" = false ] && [ \"$rustc_driver_seen_rlib\" = false ]; then printf '%s\\n' 'rustc_driver manifest lacks expected crate-type line for musl host normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$rustc_driver_manifest_tmp\" \"$rustc_driver_manifest\"\n");
    script.push_str("rm -f \"$rustc_driver_manifest_tmp\"\n");
    script.push_str("fi\n");
}

fn push_first_stage_run_rustc_host(script: &mut String, full_source_bound: bool) {
    script.push_str("RUSTC_TARGET=\"$RUSTC_HOST_TRIPLE\"\n");
    push_first_stage_target_linker_wrapper(script, full_source_bound);
    if full_source_bound {
        script.push_str("export MANTLE_TARGET_LINKER=\"$target_alias_dir/cc\"\n");
        push_first_stage_run_rustc_sysroot_linker_aliases(script);
        push_first_stage_run_rustc_proxy_linker_patch(script);
    }
    push_first_stage_musl_proc_macro_runtime(script);
    push_first_stage_run_rustc_dylib_ext(script);
    if full_source_bound {
        script.push_str(&format!(
            "if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -C {FIRST_STAGE_RUN_RUSTC_DIR} DYLIB_EXT=\"$RUN_RUSTC_DYLIB_EXT\" MANTLE_TARGET_LINKER=\"$target_alias_dir/cc\"; else $MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -C {FIRST_STAGE_RUN_RUSTC_DIR} MANTLE_TARGET_LINKER=\"$target_alias_dir/cc\"; fi\n"
        ));
    } else {
        script.push_str(&format!(
            "if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -C {FIRST_STAGE_RUN_RUSTC_DIR} DYLIB_EXT=\"$RUN_RUSTC_DYLIB_EXT\"; else $MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -C {FIRST_STAGE_RUN_RUSTC_DIR}; fi\n"
        ));
    }
}

fn push_first_stage_run_rustc_sysroot_linker_aliases(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str("printf '%s\\n' 'binding run_rustc sysroot linker aliases to the protected wrapper'\n");
    script
        .push_str("run_rustc_sysroot_linker_dir=\"$(pwd)/run_rustc/output/prefix-s/lib/rustlib/$RUSTC_TARGET/bin\"\n");
    script.push_str("run_rustc_self_contained_linker_dir=\"$run_rustc_sysroot_linker_dir/self-contained\"\n");
    script.push_str("mkdir -p \"$run_rustc_sysroot_linker_dir\" \"$run_rustc_self_contained_linker_dir\"\n");
    script.push_str(
        "for linker_alias in \"$run_rustc_sysroot_linker_dir/cc\" \"$run_rustc_self_contained_linker_dir/cc\"; do\n",
    );
    script.push_str("  rm -f \"$linker_alias\"\n");
    script.push_str("  $COPY_PROGRAM \"$MANTLE_TARGET_LINKER\" \"$linker_alias\"\n");
    script.push_str("  chmod +x \"$linker_alias\"\n");
    script.push_str("done\n");
    script.push_str("test -x \"$run_rustc_sysroot_linker_dir/cc\"\n");
    script.push_str("test -x \"$run_rustc_self_contained_linker_dir/cc\"\n");
    assert!(script.contains("run_rustc/output/prefix-s/lib/rustlib/$RUSTC_TARGET/bin"));
    assert!(script.contains("$run_rustc_self_contained_linker_dir/cc"));
}

fn push_first_stage_run_rustc_proxy_linker_patch(script: &mut String) {
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!("run_rustc_proxy={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_PROXY_PATH)));
    script.push_str("run_rustc_proxy_tmp=\"$BUILD_DIR/rustc-proxy.sh\"\n");
    for (name, original, protected) in [
        (
            "target",
            FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_ORIGINAL_LINE,
            FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_PROTECTED_LINE,
        ),
        (
            "bootstrap",
            FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_ORIGINAL_LINE,
            FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_PROTECTED_LINE,
        ),
    ] {
        script.push_str(&format!("run_rustc_proxy_{name}_original={}\n", shell_quote(original)));
        script.push_str(&format!("run_rustc_proxy_{name}_protected={}\n", shell_quote(protected)));
        script.push_str(&format!("run_rustc_proxy_{name}_replaced=false\nrun_rustc_proxy_{name}_seen=false\n"));
    }
    script.push_str(": > \"$run_rustc_proxy_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_proxy_target_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_proxy_target_protected\" >> \"$run_rustc_proxy_tmp\"\n");
    script.push_str("    run_rustc_proxy_target_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_proxy_target_protected\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_proxy_tmp\"\n");
    script.push_str("    run_rustc_proxy_target_seen=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_proxy_bootstrap_original\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_proxy_bootstrap_protected\" >> \"$run_rustc_proxy_tmp\"\n");
    script.push_str("    run_rustc_proxy_bootstrap_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_proxy_bootstrap_protected\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_proxy_tmp\"\n");
    script.push_str("    run_rustc_proxy_bootstrap_seen=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_proxy_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_proxy\"\n");
    for name in ["target", "bootstrap"] {
        script.push_str(&format!(
            "if [ \"$run_rustc_proxy_{name}_replaced\" = false ] && [ \"$run_rustc_proxy_{name}_seen\" = false ]; then printf '%s\\n' 'run_rustc proxy lacks the protected {name} linker invocation' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
        ));
    }
    script.push_str("$COPY_PROGRAM \"$run_rustc_proxy_tmp\" \"$run_rustc_proxy\"\n");
    script.push_str("chmod +x \"$run_rustc_proxy\"\n");
    script.push_str("rm -f \"$run_rustc_proxy_tmp\"\n");
    assert!(script.contains("run_rustc_proxy_target_replaced=true"));
    assert!(script.contains("run_rustc_proxy_bootstrap_replaced=true"));
    assert!(script.contains(FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_PROTECTED_LINE));
    assert!(script.contains(FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_PROTECTED_LINE));
}

fn push_first_stage_musl_proc_macro_runtime(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_musl_proc_macro_runtime_setup(script);
    push_first_stage_musl_proc_macro_runtime_patch(script);
}

fn push_first_stage_musl_proc_macro_runtime_setup(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(&format!(
        "if [ \"$RUSTC_TARGET\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ] && [ \"$RUSTC_PROVIDER_TARGET_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then\n"
    ));
    script.push_str("printf '%s\\n' 'preparing source-root musl proc-macro runtime search path'\n");
    script.push_str(&format!(
        "target_musl_shared_libc=\"$target_musl_crt_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\"\n"
    ));
    script.push_str(&format!(
        "if [ ! -f \"$target_musl_shared_libc\" ]; then printf '%s\\n' 'source-root musl libc.so missing for proc-macro runtime' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "rm -f \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\"; $COPY_PROGRAM \"$target_musl_shared_libc\" \"$target_runtime_dir/{FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT}\"\n"
    ));
    script.push_str(&format!("run_rustc_makefile={FIRST_STAGE_RUN_RUSTC_DIR}/Makefile\n"));
    script.push_str(&format!(
        "if [ ! -f \"$run_rustc_makefile\" ]; then printf '%s\\n' 'mrustc run_rustc Makefile missing before proc-macro runtime normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("run_rustc_ld_tmp=\"$BUILD_DIR/run-rustc-Makefile\"\n");
    script
        .push_str(&format!("run_rustc_ld_original_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_LD_LIBRARY_PATH_LINE)));
    script.push_str(&format!(
        "run_rustc_ld_runtime_line=\"{}\"\n",
        FIRST_STAGE_RUN_RUSTC_RUNTIME_LD_LIBRARY_PATH_TEMPLATE
    ));
    script
        .push_str(&format!("run_rustc_stage2_original_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE2_ENV_LINE)));
    script.push_str(&format!(
        "run_rustc_stage2_runtime_line={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE2_RUNTIME_ENV_LINE)
    ));
    script.push_str(&format!("run_rustc_final_original_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_ENV_LINE)));
    script.push_str(&format!(
        "run_rustc_final_prefix2_line={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_PREFIX2_ENV_LINE)
    ));
    script.push_str("run_rustc_ld_replaced=false\n");
    script.push_str("run_rustc_ld_seen_runtime=false\n");
    script.push_str("run_rustc_stage2_replaced=false\n");
    script.push_str("run_rustc_stage2_seen_runtime=false\n");
    script.push_str("run_rustc_final_replaced=false\n");
    script.push_str("run_rustc_final_seen_prefix2=false\n");
}

fn push_first_stage_musl_proc_macro_runtime_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str(": > \"$run_rustc_ld_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_ld_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_ld_runtime_line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_ld_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_ld_runtime_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_ld_seen_runtime=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_stage2_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_stage2_runtime_line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_stage2_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_stage2_runtime_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_stage2_seen_runtime=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_final_prefix2_line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_final_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_prefix2_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("    run_rustc_final_seen_prefix2=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_ld_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_ld_replaced\" = false ] && [ \"$run_rustc_ld_seen_runtime\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected LD_LIBRARY_PATH line for proc-macro runtime normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$run_rustc_stage2_replaced\" = false ] && [ \"$run_rustc_stage2_seen_runtime\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected stage2 Cargo env line for runtime normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$run_rustc_final_replaced\" = false ] && [ \"$run_rustc_final_seen_prefix2\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected final rustc Cargo env line for prefix-2 host normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_ld_tmp\" \"$run_rustc_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_ld_tmp\"\n");
    push_first_stage_run_rustc_static_sysroot_patch(script);
    push_first_stage_run_rustc_cargo_all_static_patch(script);
    script.push_str("fi\n");
}

fn push_first_stage_run_rustc_cargo_all_static_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("printf '%s\\n' 'normalizing run_rustc Cargo static feature set for source-root musl host'\n");
    script.push_str("run_rustc_cargo_tmp=\"$BUILD_DIR/run-rustc-cargo-all-static-Makefile\"\n");
    for (name, line) in [
        ("original", FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE),
        ("patched", FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_ALL_STATIC_LINE),
        ("protected_original", FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_LINE),
        ("protected_patched", FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_ALL_STATIC_LINE),
    ] {
        script.push_str(&format!("run_rustc_cargo_{name}_line={}\n", shell_quote(line)));
    }
    script.push_str("run_rustc_cargo_replaced=false\n");
    script.push_str("run_rustc_cargo_seen_patched=false\n");
    script.push_str(": > \"$run_rustc_cargo_tmp\"\n");
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_cargo_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_cargo_patched_line\" >> \"$run_rustc_cargo_tmp\"\n");
    script.push_str("    run_rustc_cargo_replaced=true\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_cargo_protected_original_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_cargo_protected_patched_line\" >> \"$run_rustc_cargo_tmp\"\n");
    script.push_str("    run_rustc_cargo_replaced=true\n");
    script.push_str(
        "  elif [ \"$line\" = \"$run_rustc_cargo_patched_line\" ] || [ \"$line\" = \"$run_rustc_cargo_protected_patched_line\" ]; then\n",
    );
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_cargo_tmp\"\n");
    script.push_str("    run_rustc_cargo_seen_patched=true\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_cargo_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_cargo_replaced\" = false ] && [ \"$run_rustc_cargo_seen_patched\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected Cargo build line for all-static normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_cargo_tmp\" \"$run_rustc_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_cargo_tmp\"\n");
}

fn push_first_stage_run_rustc_static_sysroot_patch(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    push_first_stage_run_rustc_static_sysroot_setup(script);
    push_first_stage_run_rustc_static_sysroot_rewrite(script);
}

fn push_first_stage_run_rustc_static_sysroot_setup(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("printf '%s\\n' 'normalizing run_rustc static rustc sysroot wrappers'\n");
    script.push_str("run_rustc_sysroot_tmp=\"$BUILD_DIR/run-rustc-sysroot-Makefile\"\n");
    script.push_str(&format!("run_rustc_stage1_rule={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE1_RUSTC_RULE_LINE)));
    script.push_str(&format!("run_rustc_stage2_rule={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE2_RUSTC_RULE_LINE)));
    script.push_str(&format!("run_rustc_copy_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_COPY_PREREQ_LINE)));
    script.push_str(&format!(
        "run_rustc_stage_symlink_line={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE_SYSROOT_SYMLINK_LINE)
    ));
    script.push_str("run_rustc_stage1_wrapper_line=\"\tprintf '#!/bin/sh\\nLD_LIBRARY_PATH=\\\"$target_runtime_dir:\\$(abspath \\$(PREFIX_2)lib):\\$(abspath \\$(LIBDIR))\\\" \\\"$target_runtime_dir/libc.so\\\" \\\"\\$\\$0.bin\\\" --sysroot \\\"\\$(abspath \\$(PREFIX_S))\\\" \\\"\\$\\$@\\\"\\n' >\\$@\"\n");
    script.push_str("run_rustc_stage2_wrapper_line=\"\tprintf '#!/bin/sh\\nLD_LIBRARY_PATH=\\\"$target_runtime_dir:\\$(abspath \\$(PREFIX_2)lib):\\$(abspath \\$(LIBDIR))\\\" \\\"$target_runtime_dir/libc.so\\\" \\\"\\$\\$0.bin\\\" --sysroot \\\"\\$(abspath \\$(PREFIX_2))\\\" \\\"\\$\\$@\\\"\\n' >\\$@\"\n");
    script.push_str(&format!("run_rustc_chmod_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_CHMOD_LINE)));
    script
        .push_str(&format!("run_rustc_final_wrapper_line={}\n", shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_WRAPPER_LINE)));
    script.push_str(&format!(
        "run_rustc_final_symlink_line={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_SYMLINK_LINE)
    ));
    script.push_str(&format!(
        "run_rustc_final_sysroot_wrapper_line={}\n",
        shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_WRAPPER_LINE)
    ));
    script.push_str("run_rustc_rule_context=\n");
    script.push_str("run_rustc_stage1_wrapper_replaced=false\n");
    script.push_str("run_rustc_stage1_wrapper_seen=false\n");
    script.push_str("run_rustc_stage2_wrapper_replaced=false\n");
    script.push_str("run_rustc_stage2_wrapper_seen=false\n");
    script.push_str("run_rustc_final_symlink_seen=false\n");
    script.push_str("run_rustc_final_wrapper_replaced=false\n");
    script.push_str("run_rustc_final_wrapper_seen=false\n");
    script.push_str(": > \"$run_rustc_sysroot_tmp\"\n");
}

fn push_first_stage_run_rustc_static_sysroot_rewrite(script: &mut String) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("while IFS= read -r line || [ -n \"$line\" ]; do\n");
    script.push_str("  if [ \"$line\" = \"$run_rustc_stage1_rule\" ]; then\n");
    script.push_str("    run_rustc_rule_context=stage1\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_stage2_rule\" ]; then\n");
    script.push_str("    run_rustc_rule_context=stage2\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str(
        "  elif [ \"$line\" = \"$run_rustc_copy_line\" ] && [ \"$run_rustc_rule_context\" = stage1 ]; then\n",
    );
    script.push_str("    printf '%s\\n' \"$run_rustc_stage_symlink_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_stage1_wrapper_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_chmod_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_stage1_wrapper_replaced=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str(
        "  elif [ \"$line\" = \"$run_rustc_copy_line\" ] && [ \"$run_rustc_rule_context\" = stage2 ]; then\n",
    );
    script.push_str("    printf '%s\\n' \"$run_rustc_stage_symlink_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_stage2_wrapper_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_chmod_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_stage2_wrapper_replaced=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_stage1_wrapper_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_stage1_wrapper_seen=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_stage2_wrapper_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_stage2_wrapper_seen=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_wrapper_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_final_symlink_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    printf '%s\\n' \"$run_rustc_final_sysroot_wrapper_line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_final_symlink_seen=true\n");
    script.push_str("    run_rustc_final_wrapper_replaced=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_symlink_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_final_symlink_seen=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  elif [ \"$line\" = \"$run_rustc_final_sysroot_wrapper_line\" ]; then\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("    run_rustc_final_wrapper_seen=true\n");
    script.push_str("    run_rustc_rule_context=\n");
    script.push_str("  else\n");
    script.push_str("    printf '%s\\n' \"$line\" >> \"$run_rustc_sysroot_tmp\"\n");
    script.push_str("  fi\n");
    script.push_str("done < \"$run_rustc_makefile\"\n");
    script.push_str(&format!(
        "if [ \"$run_rustc_stage1_wrapper_replaced\" = false ] && [ \"$run_rustc_stage1_wrapper_seen\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected stage1 rustc copy rule for sysroot normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$run_rustc_stage2_wrapper_replaced\" = false ] && [ \"$run_rustc_stage2_wrapper_seen\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected stage2 rustc copy rule for sysroot normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$run_rustc_final_symlink_seen\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected final rustc symlink rule for sysroot normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str(&format!(
        "if [ \"$run_rustc_final_wrapper_replaced\" = false ] && [ \"$run_rustc_final_wrapper_seen\" = false ]; then printf '%s\\n' 'mrustc run_rustc Makefile lacks expected final rustc wrapper for sysroot normalization' >&2; exit {FIRST_STAGE_BUILD_FAILED_EXIT_CODE}; fi\n"
    ));
    script.push_str("$COPY_PROGRAM \"$run_rustc_sysroot_tmp\" \"$run_rustc_makefile\"\n");
    script.push_str("rm -f \"$run_rustc_sysroot_tmp\"\n");
}

fn push_first_stage_run_rustc_target(script: &mut String, full_source_bound: bool) {
    debug_assert!(script.len() <= script.capacity());
    debug_assert!(!script.contains('\0'));
    script.push_str("if [ \"$RUSTC_PROVIDER_TARGET_TRIPLE\" != \"$RUSTC_HOST_TRIPLE\" ]; then\n");
    script.push_str("RUSTC_TARGET=\"$RUSTC_PROVIDER_TARGET_TRIPLE\"\n");
    push_first_stage_target_linker_wrapper(script, full_source_bound);
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
        "STD_ENV_ARCH=\"$target_std_env_arch\" MRUSTC_PATH=\"$(pwd)/$target_bin_dir/rustc\" \"$(pwd)/bin/minicargo\" --vendor-dir \"rustc-${RUSTC_VERSION}-src/vendor\" --script-overrides \"script-overrides/stable-${RUSTC_VERSION}-linux/\" --output-dir \"$target_libdir\" $target_minicargo_flags \"$target_sysroot_source\"\n",
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

struct ShellCheckSubject<'a>(&'a str);

impl<'a> From<&'a str> for ShellCheckSubject<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

struct ShellCheckMessage<'a>(&'a str);

impl<'a> From<&'a str> for ShellCheckMessage<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

fn push_mrustc_source_file_check<'a>(
    script: &mut String,
    file_name: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
) {
    let file_name = file_name.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -f \"$MRUSTC_SOURCE/{file_name}\" ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_var_file_check<'a>(
    script: &mut String,
    shell_var: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
    exit_code: i32,
) {
    let shell_var = shell_var.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -f \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_var_dir_check<'a>(
    script: &mut String,
    shell_var: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
    exit_code: i32,
) {
    let shell_var = shell_var.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -d \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_file_check<'a>(
    script: &mut String,
    shell_expr: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
    exit_code: i32,
) {
    let shell_expr = shell_expr.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -f {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_dir_check<'a>(
    script: &mut String,
    shell_expr: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
    exit_code: i32,
) {
    let shell_expr = shell_expr.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -d {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_relative_file_check<'a>(
    script: &mut String,
    relative_path: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
) {
    let relative_path = relative_path.into().0;
    let message = message.into().0;
    script.push_str(&format!(
        "if [ ! -f {relative_path} ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_command_check<'a>(
    script: &mut String,
    shell_var: impl Into<ShellCheckSubject<'a>>,
    message: impl Into<ShellCheckMessage<'a>>,
) {
    let shell_var = shell_var.into().0;
    let message = message.into().0;
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
    debug_assert!(!provider_dir.as_os_str().is_empty());
    debug_assert!(!metadata.host_triple.trim().is_empty());
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

fn smoke_rustc_args(plan: &RustSourceProviderSmokePlan) -> Vec<OsString> {
    let args = vec![
        OsString::from("--target"),
        OsString::from(&plan.target_triple),
        OsString::from("--crate-name"),
        OsString::from(SMOKE_CRATE_NAME),
        OsString::from("--crate-type"),
        OsString::from("lib"),
        OsString::from("--emit"),
        OsString::from("link"),
        plan.source_path.clone().into_os_string(),
        OsString::from("-o"),
        plan.output_path.clone().into_os_string(),
    ];
    debug_assert!(args.iter().any(|arg| arg == "--target"));
    debug_assert!(!args.iter().any(|arg| arg == "--sysroot"));
    args
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
        cleanup_failed_output(output_dir);
        return Err(err);
    }
    Ok(())
}

fn copy_directory_contents(src: &Path, dst: &Path, copied_entries: &mut usize) -> Result<(), RustSourceProviderError> {
    debug_assert!(*copied_entries <= DIRECTORY_DIGEST_MAX_ENTRIES);
    debug_assert_ne!(src, dst);
    let mut pending_directories = Vec::with_capacity(1);
    pending_directories.push((src.to_path_buf(), dst.to_path_buf()));
    while let Some((source_dir, dest_dir)) = pending_directories.pop() {
        let entries = fs::read_dir(&source_dir)
            .map_err(|err| RustSourceProviderError::Copy(format!("read dir {}: {err}", source_dir.display())))?;
        for entry in entries {
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
            let dest_path = dest_dir.join(entry.file_name());
            let file_type = entry
                .file_type()
                .map_err(|err| RustSourceProviderError::Copy(format!("file type {}: {err}", source_path.display())))?;
            if file_type.is_dir() {
                fs::create_dir(&dest_path)
                    .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", dest_path.display())))?;
                pending_directories.push((source_path, dest_path));
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
                return Err(RustSourceProviderError::Copy(format!(
                    "unsupported file type at {}",
                    source_path.display()
                )));
            }
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

fn cleanup_failed_output(output_dir: &Path) {
    if let Err(cleanup_error) = remove_import_output(output_dir) {
        eprintln!("warning: failed to clean Rust source provider output: {cleanup_error}");
    }
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

pub(crate) fn observed_provider_receipts(
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
    debug_assert!(dir.is_dir());
    let mut entries = Vec::with_capacity(1);
    collect_relative_paths(dir, dir, &mut entries)?;
    debug_assert!(entries.len() <= DIRECTORY_DIGEST_MAX_ENTRIES);
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
    debug_assert!(current.starts_with(root));
    debug_assert!(out.len() <= DIRECTORY_DIGEST_MAX_ENTRIES);
    let mut pending_directories = Vec::with_capacity(1);
    pending_directories.push(current.to_path_buf());
    while let Some(directory) = pending_directories.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|err| RustSourceProviderError::Digest(format!("read dir {}: {err}", directory.display())))?;
        for entry in entries {
            if out.len() >= DIRECTORY_DIGEST_MAX_ENTRIES {
                return Err(RustSourceProviderError::Digest(format!(
                    "directory {} has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries",
                    root.display()
                )));
            }
            let entry = entry.map_err(|err| RustSourceProviderError::Digest(format!("read dir entry: {err}")))?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|err| RustSourceProviderError::Digest(format!("strip prefix: {err}")))?
                .to_string_lossy()
                .to_string();
            out.push(relative);
            if path.is_dir() && !path.is_symlink() {
                pending_directories.push(path);
            }
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
    const SYNTHETIC_RUSTC_UNEXPECTED_SYSROOT_EXIT_CODE: u32 = 3;
    const FIRST_STAGE_TEST_SOURCE_COUNT: usize = 2;
    const STAGE_CONSTRUCTION_IDENTITY_ARGUMENT_COUNT: usize = 3;
    const FULL_SOURCE_STAGE_CONSTRUCTION_IDENTITY_ARGUMENT_COUNT: usize = 10;
    const FULL_SOURCE_TEST_CLOSURE_RECORD_COUNT: u32 = 7;
    const RUSTC_STAGE1_TEST_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT: usize = 1;
    const RUSTC_FINAL_TEST_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_NEXT_ID: &str = "rust-1.92.0-stage1";
    const RUSTC_STAGE1_FINAL_CHAIN_ID: &str = "rust-1.93.1-stage1";
    const RUSTC_FINAL_STAGE_ID: &str = "rust-1.94.0-final";
    const SOURCE_ARCHIVE_FILE_MODE: u32 = 0o644;
    const SOURCE_ARCHIVE_EXECUTABLE_FILE_MODE: u32 = 0o755;
    const OBSERVED_RUST_190_SOURCE_TREE_ENTRIES: usize = 279_266;
    const _: () = assert!(DIRECTORY_DIGEST_MAX_ENTRIES > OBSERVED_RUST_190_SOURCE_TREE_ENTRIES);
    const _: () = assert!(OBSERVED_RUST_190_SOURCE_TREE_ENTRIES > 0);
    const SHA256_HEX_CHAR_COUNT: usize = 64;
    const FIRST_STAGE_PATCH_PLAN_MIN_OPERATIONS: usize = 9;
    const RUST_BOOTSTRAP_PATCH_PLAN_MIN_OPERATIONS: usize = 6;
    #[cfg(unix)]
    const EXECUTABLE_MODE: u32 = 0o755;

    #[test]
    fn generated_script_command_uses_selected_interpreter_without_ambient_fallback() {
        let script = Path::new("/proof/attempt/with/a/long/path/generated-stage.sh");
        let interpreter = Path::new("/verified/host-tools/busybox/bin/sh");

        let direct = generated_script_command(script, None);
        let interpreted = generated_script_command(script, Some(interpreter));

        assert_eq!(direct.get_program(), script.as_os_str());
        assert_eq!(direct.get_args().count(), 0);
        assert_eq!(interpreted.get_program(), interpreter.as_os_str());
        assert_eq!(interpreted.get_args().collect::<Vec<_>>(), vec![script.as_os_str()]);
    }

    #[cfg(unix)]
    #[test]
    fn generated_script_launch_retry_is_bounded_and_etxtbsy_only() {
        let transient = std::io::Error::from_raw_os_error(libc::ETXTBSY);
        let permanent = std::io::Error::from_raw_os_error(libc::EACCES);

        assert!(should_retry_generated_script_launch(&transient, 1));
        assert!(!should_retry_generated_script_launch(&transient, GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS));
        assert!(!should_retry_generated_script_launch(&permanent, 1));
    }

    #[cfg(unix)]
    #[test]
    fn smoke_process_retry_is_bounded_and_text_file_busy_only() {
        use std::os::unix::process::ExitStatusExt;

        let transient = Output {
            status: ExitStatus::from_raw(SHELL_COMMAND_NOT_EXECUTABLE_EXIT_CODE << UNIX_EXIT_STATUS_SHIFT_BITS),
            stdout: Vec::new(),
            stderr: TEXT_FILE_BUSY_MESSAGE.as_bytes().to_vec(),
        };
        let wrong_status = Output {
            status: ExitStatus::from_raw(1 << UNIX_EXIT_STATUS_SHIFT_BITS),
            stdout: Vec::new(),
            stderr: TEXT_FILE_BUSY_MESSAGE.as_bytes().to_vec(),
        };
        let wrong_message = Output {
            status: ExitStatus::from_raw(SHELL_COMMAND_NOT_EXECUTABLE_EXIT_CODE << UNIX_EXIT_STATUS_SHIFT_BITS),
            stdout: Vec::new(),
            stderr: b"permission denied".to_vec(),
        };

        assert!(should_retry_smoke_process_output(&transient, 1));
        assert!(!should_retry_smoke_process_output(&transient, GENERATED_SCRIPT_LAUNCH_MAX_ATTEMPTS));
        assert!(!should_retry_smoke_process_output(&wrong_status, 1));
        assert!(!should_retry_smoke_process_output(&wrong_message, 1));
    }

    #[test]
    fn checked_in_source_recipe_selects_command_materializer_without_prebuilt_route() {
        let recipe = include_str!("../bootstrap/rust-source.ncl");

        assert!(recipe.contains("mantle-rust-source-provider-materialization-recipe-v1"));
        assert!(recipe.contains("mantle bootstrap rust-source-provider"));
        assert!(recipe.contains("bootstrap/rust-source-musl-host-plan.ncl"));
        assert!(!recipe.contains("rust-source-provider-blocked"));
        assert!(!recipe.contains("materialization is not implemented"));
        assert!(!recipe.contains("import \"rust.ncl\""));
    }

    #[test]
    fn downstream_native_consumers_do_not_branch_on_rust_bootstrap_internals() {
        let downstream_sources = [
            ("rust_plan", include_str!("rust_plan.rs")),
            ("native_toolchain_closure", include_str!("native_toolchain_closure.rs")),
            ("cargo_free_self_build", include_str!("cargo_free_self_build.rs")),
        ];
        let internal_markers = ["mrustc", "minicargo", "run_rustc"];

        for (source_name, source_text) in downstream_sources {
            for marker in internal_markers {
                assert!(
                    !source_text.contains(marker),
                    "{source_name} must consume provider metadata/capabilities, not compiler-bootstrap detail {marker}"
                );
            }
        }
    }

    fn assert_patch_plan_operation(
        manifest: &serde_json::Value,
        stage_id: &str,
        operation_id: &str,
        min_operation_count: usize,
    ) {
        assert_eq!(
            manifest["patch_plan"]["schema"],
            crate::rust_bootstrap_patch_plan::RUST_BOOTSTRAP_PATCH_PLAN_SCHEMA
        );
        assert_eq!(manifest["patch_plan"]["input"]["stage_id"], stage_id);
        assert_eq!(manifest["patch_plan"]["input_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(manifest["patch_plan"]["output_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        let operations = manifest["patch_plan"]["operations"].as_array().unwrap();
        assert!(operations.len() >= min_operation_count);
        assert!(operations.iter().any(|operation| operation["id"] == operation_id));
    }

    fn assert_receipt_has_patch_plan_step(receipt_path: &Path, operation_id: &str) {
        let receipt: serde_json::Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
        let steps = receipt["build_steps"].as_array().unwrap();
        let patch_step = steps.iter().find(|step| step["name"] == "record-rust-bootstrap-patch-plan").unwrap();
        let arguments = patch_step["arguments"].as_array().unwrap();
        assert!(arguments.iter().any(|argument| argument.as_str().unwrap().contains("output-digest=")));
        assert!(arguments.iter().any(|argument| argument.as_str().unwrap().contains(operation_id)));
        assert_eq!(patch_step["program"], "mantle-rust-bootstrap-patch-plan");
    }

    fn assert_stage_construction_evidence(provider_dir: &Path, stage_id: &str) {
        let stage_dir = provider_dir.join(STAGE_CONSTRUCTION_EVIDENCE_RELATIVE_DIR).join(stage_id);
        let expected_files = [
            STAGE_EVIDENCE_PLAN_FILE,
            STAGE_EVIDENCE_SCRIPT_FILE,
            STAGE_EVIDENCE_BUILD_FILE,
            STAGE_EVIDENCE_RECEIPT_FILE,
            STAGE_EVIDENCE_METADATA_FILE,
            STAGE_EVIDENCE_SMOKE_FILE,
        ];
        assert_eq!(expected_files.len(), STAGE_CONSTRUCTION_EVIDENCE_FILE_COUNT);
        assert!(stage_dir.is_dir());
        for file_name in expected_files {
            assert!(stage_dir.join(file_name).is_file(), "missing {file_name} for stage {stage_id}");
        }
    }

    fn assert_receipt_has_stage_construction_identity(receipt_path: &Path) {
        let receipt: serde_json::Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
        let steps = receipt["build_steps"].as_array().unwrap();
        let identity_steps: Vec<&serde_json::Value> =
            steps.iter().filter(|step| step["name"] == "record-stage-construction-identity").collect();
        assert_eq!(identity_steps.len(), 1);
        let identity_step = identity_steps[0];
        let arguments = identity_step["arguments"].as_array().unwrap();
        assert_eq!(arguments.len(), STAGE_CONSTRUCTION_IDENTITY_ARGUMENT_COUNT);
        assert_eq!(identity_step["program"], "mantle-provider-digest");
        assert!(arguments.iter().any(|argument| {
            argument.as_str().is_some_and(|value| {
                value.strip_prefix("stage-plan-blake3=").is_some_and(|digest| digest.len() == SHA256_HEX_CHAR_COUNT)
            })
        }));
        assert!(arguments.iter().any(|argument| {
            argument.as_str().is_some_and(|value| {
                value.strip_prefix("script-blake3=").is_some_and(|digest| digest.len() == SHA256_HEX_CHAR_COUNT)
            })
        }));
        assert!(arguments.iter().any(|argument| argument == &format!("parallel-jobs={RUST_BOOTSTRAP_JOB_COUNT}")));
    }

    fn unsupported_patch_operation(phase: RustBootstrapPatchPhase) -> RustBootstrapPatchOperation {
        RustBootstrapPatchOperation {
            id: "unsupported-test-operation".to_string(),
            kind: RustBootstrapPatchOperationKind::Unsupported,
            phase,
            summary: "exercise fail-closed shell operation dispatch".to_string(),
            source_id: None,
            expected_anchor: Some("synthetic anchor".to_string()),
        }
    }

    #[test]
    fn stage_construction_evidence_rejects_missing_source_file() {
        let temp = tempfile::tempdir().unwrap();
        let evidence_root = temp.path().join("evidence");
        let missing = temp.path().join("missing");
        fs::create_dir(&evidence_root).unwrap();

        let error = persist_stage_construction_evidence_source(&evidence_root, RustStageConstructionEvidenceSource {
            stage_id: "missing-stage",
            plan_path: &missing,
            script_path: &missing,
            build_manifest_path: &missing,
            receipt_path: missing.clone(),
            metadata_path: &missing,
            smoke_summary_path: missing.clone(),
        })
        .unwrap_err();

        assert!(matches!(error, RustSourceProviderError::Copy(_)));
        assert!(error.to_string().contains("stage construction evidence source"));
    }

    #[test]
    fn first_stage_patch_operation_dispatch_rejects_unsupported_operation() {
        let mut script = String::new();
        let operation = unsupported_patch_operation(RustBootstrapPatchPhase::BeforeFirstStageMainMake);

        let err = push_first_stage_patch_plan_operation(&mut script, &operation, false).unwrap_err();

        assert!(script.is_empty());
        assert!(err.to_string().contains("unsupported first-stage script patch operation"));
    }

    #[test]
    fn rust_bootstrap_patch_operation_dispatch_rejects_unsupported_operation() {
        let mut script = String::new();
        let operation = unsupported_patch_operation(RustBootstrapPatchPhase::RustBootstrapBeforeXpy);

        let err = push_rustc_source_patch_plan_operation(&mut script, &operation).unwrap_err();

        assert!(script.is_empty());
        assert!(err.to_string().contains("unsupported Rust bootstrap script patch operation"));
    }

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
    fn full_source_generated_tool_bindings_exclude_ambient_discovery() {
        let context = full_source_execution_context_fixture();
        let mut script = String::new();

        push_generated_script_shebang(&mut script, Some(&context)).unwrap();
        push_full_source_tool_bindings(&mut script, &context).unwrap();
        push_first_stage_target_linker_wrapper(&mut script, true);
        push_rustc_source_build_tool_discovery(&mut script, Some(&context)).unwrap();

        assert!(script.starts_with("#!/receipt-bound/sh\n"));
        assert!(script.contains("MAKE_PROGRAM='/receipt-bound/make'"));
        assert!(script.contains("PERL_PROGRAM='/receipt-bound/perl'"));
        assert!(script.contains("PERL5LIB='/lib/5.10.1'"));
        assert!(script.contains("LIBRARY_PATH='/lib'"));
        assert!(script.contains("GNUMAKEFLAGS=\"SHELL=$SHELL_PROGRAM\""));
        assert!(script.contains("MANTLE_ZLIB_HEADER='/include/zlib.h'"));
        assert!(script.contains("MANTLE_ZLIB_ARCHIVE='/lib/libz.a'"));
        assert!(script.contains("MANTLE_LINUX_HEADERS_ROOT='/receipt-bound/linux-headers'"));
        assert!(script.contains("MANTLE_LINUX_HEADERS_CFLAGS='-I/receipt-bound/linux-headers/include'"));
        assert!(script.contains(
            "MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG='-B/native-provider/libexec/gcc/x86_64-unknown-linux-musl/10.5.0/'"
        ));
        assert!(script.contains("export CFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\""));
        assert!(script.contains("unset GCC_EXEC_PREFIX COMPILER_PATH"));
        assert!(script.contains("MANTLE_TARGET_CC='/native-provider/bin/x86_64-linux-musl-gcc'"));
        assert!(!script.contains("ZLIB_CFLAGS=\nZLIB_LIBS=\n"));
        assert!(!script.contains("command -v"));
        assert!(!script.contains("/nix/store"));
    }

    #[test]
    fn full_source_llvm_config_binds_only_receipt_bound_linux_headers() {
        let context = full_source_execution_context_fixture();
        let mut full_source_config = String::new();
        let mut compatibility_config = String::new();

        push_rustc_source_llvm_config(&mut full_source_config, Some(&context));
        push_rustc_source_llvm_config(&mut compatibility_config, None);

        assert!(
            full_source_config.contains("cflags = \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"")
        );
        assert!(
            full_source_config
                .contains("cxxflags = \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MANTLE_LINUX_HEADERS_CFLAGS\"")
        );
        assert!(!full_source_config.contains("/usr/include"));
        assert!(!compatibility_config.contains("MANTLE_LINUX_HEADERS_CFLAGS"));
        assert!(!compatibility_config.contains("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"));
    }

    #[test]
    fn full_source_mrustc_link_uses_receipt_bound_gcc_subprogram_prefix() {
        let mut script = String::new();

        push_first_stage_mrustc_make_command(&mut script, true);

        assert!(script.contains("LINKFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(script.contains("LDFLAGS=\"$LDFLAGS\""));
    }

    #[test]
    fn compatibility_mrustc_link_does_not_invent_full_source_gcc_prefix() {
        let mut script = String::new();

        push_first_stage_mrustc_make_command(&mut script, false);

        assert!(!script.contains("LINKFLAGS_EXTRA"));
        assert!(!script.contains("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"));
    }

    #[test]
    fn full_source_minicargo_compile_and_link_use_receipt_bound_gcc_prefix() {
        let mut script = String::new();

        push_first_stage_minicargo_make_command(&mut script, true);

        assert!(script.contains("CXXFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(script.contains("LINKFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
    }

    #[test]
    fn compatibility_minicargo_does_not_invent_full_source_gcc_prefix() {
        let mut script = String::new();

        push_first_stage_minicargo_make_command(&mut script, false);

        assert!(!script.contains("CXXFLAGS_EXTRA"));
        assert!(!script.contains("LINKFLAGS_EXTRA"));
        assert!(!script.contains("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"));
    }

    #[test]
    fn full_source_runtime_shim_compile_uses_receipt_bound_gcc_prefix() {
        let mut script = String::new();

        push_first_stage_target_cc_compile_command(&mut script, true, "-c input.c -o output.o");

        assert!(script.starts_with("\"$target_cc_path\" \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(script.ends_with("-c input.c -o output.o\n"));
    }

    #[test]
    fn compatibility_runtime_shim_compile_does_not_invent_full_source_gcc_prefix() {
        let mut script = String::new();

        push_first_stage_target_cc_compile_command(&mut script, false, "-c input.c -o output.o");

        assert!(script.starts_with("\"$target_cc_path\" -c input.c"));
        assert!(!script.contains("MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"));
    }

    #[test]
    fn full_source_tool_bindings_reject_missing_perl_without_ambient_fallback() {
        let mut context = full_source_execution_context_fixture();
        context.host_tools.manifest.tools.retain(|tool| tool.role != FullSourceRustHostToolRole::Perl);
        let mut script = String::new();

        let error = push_full_source_tool_bindings(&mut script, &context).unwrap_err();

        assert!(script.is_empty());
        assert!(error.to_string().contains("host-tool role Perl is missing"));
    }

    #[test]
    fn protected_execution_paths_are_full_source_only() {
        let mut compatibility_script = String::new();
        let mut full_source_script = String::new();

        push_first_stage_minicargo_protected_execution_paths(&mut compatibility_script, false);
        push_first_stage_minicargo_protected_execution_paths(&mut full_source_script, true);

        assert!(compatibility_script.is_empty());
        assert!(full_source_script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_PROTECTED_LINE));
        assert!(full_source_script.contains(FIRST_STAGE_RUN_RUSTC_MINICARGO_PROTECTED_LINE));
        assert!(full_source_script.contains("run_rustc Makefile lacks the expected protected minicargo executable"));
        assert!(full_source_script.contains("receipt-make-bin"));
        assert!(full_source_script.contains("SHELL=$SHELL_PROGRAM"));
        assert!(full_source_script.contains("MRUSTC_SHELL=\"$SHELL_PROGRAM\""));
        assert!(full_source_script.contains("MANTLE_LLVM_RULE_NORMALIZER"));
        assert!(full_source_script.contains("llvm-min-tblgen"));
        assert!(full_source_script.contains("llvm-tblgen"));
        assert!(full_source_script.contains("::execl(mantle_shell"));
        assert!(full_source_script.contains("spawn_executable"));
        assert!(full_source_script.contains("realpath(executable"));
        assert!(!full_source_script.contains("SHELL=/bin/sh"));
        push_first_stage_run_rustc_stage_tool_absolute_patch(&mut compatibility_script, false);
        push_first_stage_run_rustc_stage_tool_absolute_patch(&mut full_source_script, true);
        assert!(compatibility_script.is_empty());
        assert!(full_source_script.contains(FIRST_STAGE_RUN_RUSTC_STAGE_RUSTC_EXEC_PROTECTED_TOKEN));
        assert!(full_source_script.contains(FIRST_STAGE_RUN_RUSTC_STAGE_CARGO_EXEC_PROTECTED_TOKEN));
    }

    #[test]
    fn full_source_translated_cargo_build_serializes_minicargo_jobs() {
        let mut script = String::new();
        push_first_stage_translated_cargo_build(&mut script, true);

        assert!(script.contains(&format!(
            "PARLEVEL={FIRST_STAGE_TRANSLATED_CARGO_MINICARGO_JOB_COUNT} {FIRST_STAGE_TRANSLATED_CARGO_BINARY}"
        )));
        assert!(script.contains(FIRST_STAGE_MAKE_PARALLEL_ARG));
    }

    #[test]
    fn compatibility_translated_cargo_build_keeps_parallel_minicargo_jobs() {
        let mut script = String::new();
        push_first_stage_translated_cargo_build(&mut script, false);

        assert!(script.contains(&format!("-f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_CARGO_BINARY}")));
        assert!(!script.contains(&format!("PARLEVEL={FIRST_STAGE_TRANSLATED_CARGO_MINICARGO_JOB_COUNT}")));
    }

    #[test]
    fn run_rustc_stage_tool_patch_rewrites_only_recipe_executables() {
        let makefile = format!(
            "target: $(BINDIR_S)rustc $(BINDIR_S)cargo\n\t$(BINDIR_S)rustc --version\n\t$(BINDIR_S)cargo --version\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE}\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE}\n"
        );
        let (output, rewritten) = run_test_run_rustc_stage_tool_patch(&makefile);

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(rewritten.contains("target: $(BINDIR_S)rustc $(BINDIR_S)cargo"));
        assert!(rewritten.contains("\t$(abspath $(BINDIR_S)rustc) --version"));
        assert!(rewritten.contains("\t$(abspath $(BINDIR_S)cargo) --version"));
        assert!(rewritten.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_PROTECTED_LINE));
        assert!(!rewritten.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE));
        assert!(rewritten.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_PROTECTED_LINE));
        assert!(!rewritten.contains(FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE));
    }

    #[test]
    fn run_rustc_stage_tool_patch_rejects_repeated_executable_token() {
        let makefile = "target:\n\t$(BINDIR_S)rustc $(BINDIR_S)rustc --version\n\t$(BINDIR_S)cargo --version\n";
        let (output, rewritten) = run_test_run_rustc_stage_tool_patch(makefile);

        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("run_rustc rustc recipe contains repeated executable tokens")
        );
        assert_eq!(rewritten, makefile);
    }

    #[test]
    fn run_rustc_stage_tool_and_all_static_patches_compose() {
        let makefile = format!(
            "target:\n\t$(BINDIR_S)rustc --version\n{FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE}\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE}\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE}\n"
        );
        let (output, rewritten) = run_test_run_rustc_composed_patches(&makefile);

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(rewritten.lines().any(|line| line == FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_ALL_STATIC_LINE));
        assert!(!rewritten.lines().any(|line| line == FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_PROTECTED_LINE));
    }

    #[test]
    fn run_rustc_all_static_patch_rejects_unknown_protected_cargo_recipe() {
        let makefile = format!(
            "target:\n\t$(BINDIR_S)rustc --version\n\t$(BINDIR_S)cargo build --unexpected\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_ORIGINAL_LINE}\n{FIRST_STAGE_RUN_RUSTC_FINAL_SMOKE_EXEC_ORIGINAL_LINE}\n"
        );
        let (output, rewritten) = run_test_run_rustc_composed_patches(&makefile);

        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("mrustc run_rustc Makefile lacks expected Cargo build line for all-static normalization")
        );
        assert!(rewritten.contains("\t$(abspath $(BINDIR_S)cargo) build --unexpected"));
    }

    #[test]
    fn run_rustc_stage_tool_patch_rejects_unknown_final_smoke_recipe() {
        let makefile = "target:\n\t$(BINDIR_S)rustc --version\n\t$(BINDIR_S)cargo --version\n\t$V$(DBG) $(BINDIR)rustc --unexpected\n";
        let (output, rewritten) = run_test_run_rustc_stage_tool_patch(makefile);

        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("run_rustc Makefile lacks the protected final rustc smoke recipe")
        );
        assert!(rewritten.contains("\t$V$(DBG) $(BINDIR)rustc --unexpected"));
    }

    #[test]
    fn run_rustc_sysroot_linker_aliases_copy_protected_wrapper() {
        let dir = tempfile::tempdir().unwrap();
        let linker = dir.path().join("protected-linker");
        fs::write(&linker, "protected linker\n").unwrap();
        let mut script = format!(
            "set -eu\nCOPY_PROGRAM=cp\nMANTLE_TARGET_LINKER={}\nRUSTC_TARGET={}\n",
            shell_quote(linker.to_str().unwrap()),
            shell_quote(FIRST_STAGE_MUSL_TRIPLE)
        );
        push_first_stage_run_rustc_sysroot_linker_aliases(&mut script);
        let output = Command::new("/bin/sh").arg("-c").arg(script).current_dir(dir.path()).output().unwrap();
        let linker_dir =
            dir.path().join("run_rustc/output/prefix-s/lib/rustlib").join(FIRST_STAGE_MUSL_TRIPLE).join("bin");

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(fs::read_to_string(linker_dir.join("cc")).unwrap(), "protected linker\n");
        assert_eq!(fs::read_to_string(linker_dir.join("self-contained/cc")).unwrap(), "protected linker\n");
    }

    #[test]
    fn run_rustc_sysroot_linker_aliases_reject_missing_wrapper() {
        let dir = tempfile::tempdir().unwrap();
        let mut script = format!(
            "set -eu\nCOPY_PROGRAM=cp\nMANTLE_TARGET_LINKER={}\nRUSTC_TARGET={}\n",
            shell_quote(dir.path().join("missing-linker").to_str().unwrap()),
            shell_quote(FIRST_STAGE_MUSL_TRIPLE)
        );
        push_first_stage_run_rustc_sysroot_linker_aliases(&mut script);
        let output = Command::new("/bin/sh").arg("-c").arg(script).current_dir(dir.path()).output().unwrap();
        let linker_dir =
            dir.path().join("run_rustc/output/prefix-s/lib/rustlib").join(FIRST_STAGE_MUSL_TRIPLE).join("bin");

        assert!(!output.status.success());
        assert!(!linker_dir.join("cc").exists());
        assert!(!linker_dir.join("self-contained/cc").exists());
    }

    #[test]
    fn run_rustc_proxy_patch_binds_bootstrap_linker() {
        let proxy = format!(
            "#!/bin/sh\n{FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_ORIGINAL_LINE}\n{FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_ORIGINAL_LINE}\n"
        );
        let (output, rewritten) = run_test_run_rustc_proxy_patch(&proxy);

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(rewritten.contains(FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_PROTECTED_LINE));
        assert!(!rewritten.contains(FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_ORIGINAL_LINE));
        assert!(rewritten.contains(FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_PROTECTED_LINE));
        assert!(!rewritten.contains(FIRST_STAGE_RUN_RUSTC_PROXY_BOOTSTRAP_ORIGINAL_LINE));
    }

    #[test]
    fn run_rustc_proxy_patch_rejects_unknown_bootstrap_invocation() {
        let proxy = format!(
            "#!/bin/sh\n{FIRST_STAGE_RUN_RUSTC_PROXY_TARGET_ORIGINAL_LINE}\n    ${{PROXY_MRUSTC}} --unexpected \"$@\"\n"
        );
        let (output, rewritten) = run_test_run_rustc_proxy_patch(&proxy);

        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("run_rustc proxy lacks the protected bootstrap linker invocation")
        );
        assert_eq!(rewritten, proxy);
    }

    fn run_test_run_rustc_proxy_patch(proxy: &str) -> (std::process::Output, String) {
        let dir = tempfile::tempdir().unwrap();
        let run_rustc_dir = dir.path().join(FIRST_STAGE_RUN_RUSTC_DIR);
        let build_dir = dir.path().join("build");
        fs::create_dir_all(&run_rustc_dir).unwrap();
        fs::create_dir_all(&build_dir).unwrap();
        let proxy_path = dir.path().join(FIRST_STAGE_RUN_RUSTC_PROXY_PATH);
        fs::write(&proxy_path, proxy).unwrap();
        let mut script = format!("set -eu\nBUILD_DIR={}\nCOPY_PROGRAM=cp\n", shell_quote(build_dir.to_str().unwrap()));
        push_first_stage_run_rustc_proxy_linker_patch(&mut script);
        let output = Command::new("/bin/sh").arg("-c").arg(script).current_dir(dir.path()).output().unwrap();
        let rewritten = fs::read_to_string(proxy_path).unwrap();
        (output, rewritten)
    }

    fn run_test_run_rustc_stage_tool_patch(makefile: &str) -> (std::process::Output, String) {
        let dir = tempfile::tempdir().unwrap();
        let run_rustc_dir = dir.path().join(FIRST_STAGE_RUN_RUSTC_DIR);
        let build_dir = dir.path().join("build");
        fs::create_dir_all(&run_rustc_dir).unwrap();
        fs::create_dir_all(&build_dir).unwrap();
        let makefile_path = run_rustc_dir.join(FIRST_STAGE_MAKEFILE);
        fs::write(&makefile_path, makefile).unwrap();
        let mut script = format!("set -eu\nBUILD_DIR={}\nCOPY_PROGRAM=cp\n", shell_quote(build_dir.to_str().unwrap()));
        push_first_stage_run_rustc_stage_tool_absolute_patch(&mut script, true);
        let output = Command::new("/bin/sh").arg("-c").arg(script).current_dir(dir.path()).output().unwrap();
        let rewritten = fs::read_to_string(makefile_path).unwrap();
        (output, rewritten)
    }

    fn run_test_run_rustc_composed_patches(makefile: &str) -> (std::process::Output, String) {
        let dir = tempfile::tempdir().unwrap();
        let run_rustc_dir = dir.path().join(FIRST_STAGE_RUN_RUSTC_DIR);
        let build_dir = dir.path().join("build");
        fs::create_dir_all(&run_rustc_dir).unwrap();
        fs::create_dir_all(&build_dir).unwrap();
        let makefile_path = run_rustc_dir.join(FIRST_STAGE_MAKEFILE);
        fs::write(&makefile_path, makefile).unwrap();
        let mut script = format!("set -eu\nBUILD_DIR={}\nCOPY_PROGRAM=cp\n", shell_quote(build_dir.to_str().unwrap()));
        push_first_stage_run_rustc_stage_tool_absolute_patch(&mut script, true);
        script.push_str(&format!("run_rustc_makefile={FIRST_STAGE_RUN_RUSTC_DIR}/{FIRST_STAGE_MAKEFILE}\n"));
        push_first_stage_run_rustc_cargo_all_static_patch(&mut script);
        let output = Command::new("/bin/sh").arg("-c").arg(script).current_dir(dir.path()).output().unwrap();
        let rewritten = fs::read_to_string(makefile_path).unwrap();
        (output, rewritten)
    }

    #[test]
    fn full_source_first_stage_script_omits_compatibility_fallbacks() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("provider-out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let recipe_digest = blake3::hash(b"source-built recipe\n").to_hex().to_string();
        let mut plan = plan_materialization(&recipe, None, &output, &scratch, &recipe_digest).unwrap();
        plan.full_source_context = Some(full_source_execution_context_fixture());
        let route = load_rust_source_provider_route(&plan.route_plan_path).unwrap();
        let boundary = prepare_first_stage_boundary(&plan, &route).unwrap();

        let script = first_stage_boundary_script(&boundary).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&first_stage_boundary_manifest(&boundary).unwrap()).unwrap();

        assert_eq!(manifest["full_source_execution"]["host_support_inputs"][0]["id"], "linux-headers");
        assert_eq!(
            manifest["full_source_execution"]["host_support_inputs"][0]["content_digest_blake3"],
            "1".repeat(SHA256_HEX_CHAR_COUNT)
        );
        assert!(script.starts_with("#!/receipt-bound/sh\n"));
        assert!(script.contains("ambient discovery disabled"));
        assert!(script.contains("MANTLE_ZLIB_ARCHIVE='/lib/libz.a'"));
        assert!(script.contains("MANTLE_LINUX_HEADERS_ROOT='/receipt-bound/linux-headers'"));
        assert!(script.contains(
            "MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG='-B/native-provider/libexec/gcc/x86_64-unknown-linux-musl/10.5.0/'"
        ));
        assert!(script.contains("unset GCC_EXEC_PREFIX COMPILER_PATH"));
        assert!(script.contains(
            "export CXXFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $MRUSTC_CXXFLAGS $ZLIB_CFLAGS $MANTLE_LINUX_HEADERS_CFLAGS\""
        ));
        assert!(script.contains("export LDFLAGS=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG $ZLIB_LIBS\""));
        assert!(script.contains("CXXFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(script.contains("LINKFLAGS_EXTRA=\"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_MINICARGO_PROTECTED_LINE));
        assert!(script.contains("receipt-make-bin"));
        assert!(script.contains("SHELL=$SHELL_PROGRAM"));
        assert!(script.contains("MRUSTC_SHELL=\"$SHELL_PROGRAM\""));
        assert!(script.contains("$target_alias_dir/sh"));
        assert!(script.contains("$target_alias_dir/emcc"));
        assert!(script.contains("$target_alias_dir/pkg-config"));
        assert!(script.contains("$target_alias_dir/pkgconf"));
        assert!(script.contains("$target_alias_dir/git"));
        assert!(script.contains("$target_alias_dir/perl"));
        assert!(script.contains("$target_alias_dir/make"));
        assert!(script.contains("disabling OpenSSL assembly generators for protected source bootstrap"));
        assert!(script.contains("configure.arg(\"no-asm\");"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_PROTECTED_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_MINICARGO_PROTECTED_LINE));
        assert!(script.contains("run_rustc Makefile lacks the expected protected minicargo executable"));
        let stage_tool_binding_position =
            script.find("binding run_rustc stage tools to protected absolute executables").unwrap();
        let run_rustc_build_position = script.find("if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM").unwrap();
        assert!(stage_tool_binding_position < run_rustc_build_position);
        assert!(script.contains("::waitpid(child_pid"));
        assert!(script.contains("\"$(pwd)/bin/minicargo\" --vendor-dir"));
        assert!(!script.contains(
            "STD_ENV_ARCH=\"$target_std_env_arch\" MRUSTC_PATH=\"$(pwd)/$target_bin_dir/rustc\" bin/minicargo"
        ));
        assert!(script.contains("\"$target_cc_path\" \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" -fPIC"));
        assert!(script.contains("exec \"$target_cc_path\" \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\""));
        assert!(!script.contains("GCC_EXEC_PREFIX='/native-provider"));
        assert!(script.contains("target_toolchain_root=\"$MANTLE_TARGET_TOOLCHAIN_ROOT\""));
        assert!(script.contains("authenticated-mrustc-0.12.0"));
        assert!(script.contains("authenticated Rust source rewrite mrustc-ivar-bounds"));
        assert!(script.contains("context.possible_ivar_vals.resize(te->index + 1);"));
        assert!(script.contains("mrustc_patch_skipped"));
        assert!(script.contains("authenticated Rust source rewrite rustc-errors-presult"));
        assert!(script.contains("authenticated Rust source rewrite rustc-middle-tys-inputs"));
        assert!(!script.contains("ZLIB_CFLAGS=\nZLIB_LIBS=\n"));
        assert!(!script.contains("git show"));
        assert!(!script.contains("git diff-index"));
        assert!(!script.contains("command -v"));
        assert!(!script.contains("/nix/store"));
    }

    fn full_source_execution_context_fixture() -> FullSourceRustExecutionContext {
        use crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_SCHEMA;
        use crate::full_source_rust_binding::FullSourceNativeProviderAdmissionIdentity;
        use crate::full_source_rust_binding::FullSourceRustHostSupportInputBinding;
        use crate::full_source_rust_binding::FullSourceRustHostToolBinding;
        use crate::full_source_rust_binding::FullSourceRustHostToolManifest;
        use crate::full_source_rust_binding::FullSourceRustHostToolRole;
        use crate::full_source_rust_binding_shell::FullSourceRustHostToolObservation;

        let tool = |role, name: &str| FullSourceRustHostToolBinding {
            role,
            path: format!("/receipt-bound/{name}"),
            content_digest_blake3: "a".repeat(SHA256_HEX_CHAR_COUNT),
            source_id: format!("{name}-source"),
            construction_receipt_path: format!("/receipt-bound/{name}.receipt.json"),
            construction_receipt_digest_blake3: "b".repeat(SHA256_HEX_CHAR_COUNT),
        };
        FullSourceRustExecutionContext {
            native_provider_dir: PathBuf::from("/native-provider"),
            rust_source_archive_dir: PathBuf::from("/receipt-bound/sources"),
            linux_headers_root: PathBuf::from("/receipt-bound/linux-headers"),
            admission: FullSourceNativeProviderAdmissionIdentity {
                schema: "mantle-full-source-provider-admission-v2".to_string(),
                status: "admitted".to_string(),
                provider_id: "full-source-v1".to_string(),
                provider_target: "x86_64-linux-musl".to_string(),
                compiler_target: "x86_64-unknown-linux-musl".to_string(),
                admission_report_digest_blake3: "d".repeat(SHA256_HEX_CHAR_COUNT),
                metadata_digest_blake3: "e".repeat(SHA256_HEX_CHAR_COUNT),
                output_digest_blake3: "f".repeat(SHA256_HEX_CHAR_COUNT),
                expected_output_digest_blake3: "f".repeat(SHA256_HEX_CHAR_COUNT),
                source_closure_manifest_blake3: "0".repeat(SHA256_HEX_CHAR_COUNT),
                expected_source_closure_manifest_blake3: "0".repeat(SHA256_HEX_CHAR_COUNT),
                source_closure_record_count: FULL_SOURCE_TEST_CLOSURE_RECORD_COUNT,
            },
            host_tools: FullSourceRustHostToolObservation {
                manifest: FullSourceRustHostToolManifest {
                    schema: FULL_SOURCE_RUST_HOST_TOOL_SCHEMA.to_string(),
                    source_policy: "authenticated-offline-only".to_string(),
                    ambient_tool_discovery: false,
                    tools: vec![
                        tool(FullSourceRustHostToolRole::Make, "make"),
                        tool(FullSourceRustHostToolRole::Cmake, "cmake"),
                        tool(FullSourceRustHostToolRole::Python, "python"),
                        tool(FullSourceRustHostToolRole::Perl, "perl"),
                        tool(FullSourceRustHostToolRole::Busybox, "busybox"),
                    ],
                    support_inputs: vec![FullSourceRustHostSupportInputBinding {
                        id: "linux-headers".to_string(),
                        path: "/receipt-bound/linux-headers".to_string(),
                        content_digest_blake3: "1".repeat(SHA256_HEX_CHAR_COUNT),
                        source_id: "linux-6.6".to_string(),
                        attestation_path: "/receipt-bound/linux-headers-attestation.json".to_string(),
                        attestation_digest_blake3: "2".repeat(SHA256_HEX_CHAR_COUNT),
                    }],
                },
                manifest_digest_blake3: "c".repeat(SHA256_HEX_CHAR_COUNT),
            },
        }
    }

    fn full_source_archive_fixture(id: &str) -> RustSourceProviderBootstrapSource {
        RustSourceProviderBootstrapSource {
            id: id.to_string(),
            kind: ToolchainSourceKind::Tarball,
            name: "offline-rust-source".to_string(),
            version: "1.0.0".to_string(),
            url: "https://127.0.0.1:9/network-must-not-be-used.tar.gz".to_string(),
            sha256_hex: "a".repeat(SHA256_HEX_CHAR_COUNT),
        }
    }

    #[test]
    fn full_source_stage_identity_binds_native_closure_and_host_tools() {
        let context = full_source_execution_context_fixture();
        let step = stage_construction_identity_receipt_step(
            &"a".repeat(SHA256_HEX_CHAR_COUNT),
            &"b".repeat(SHA256_HEX_CHAR_COUNT),
            RUST_BOOTSTRAP_JOB_COUNT,
            Some(&context),
        );

        assert_eq!(step.name, "record-stage-construction-identity");
        assert_eq!(step.arguments.len(), FULL_SOURCE_STAGE_CONSTRUCTION_IDENTITY_ARGUMENT_COUNT);
        assert!(step.arguments.contains(&format!("native-provider-id={}", context.admission.provider_id)));
        assert!(
            step.arguments
                .contains(&format!("native-provider-output-blake3={}", context.admission.output_digest_blake3))
        );
        assert!(
            step.arguments
                .contains(&format!("source-closure-blake3={}", context.admission.source_closure_manifest_blake3))
        );
        assert!(
            step.arguments
                .contains(&format!("host-tool-manifest-blake3={}", context.host_tools.manifest_digest_blake3))
        );
        assert!(step.arguments.contains(&format!(
            "linux-headers-blake3={}",
            context.host_tools.manifest.support_inputs[0].content_digest_blake3
        )));
    }

    #[test]
    fn full_source_archive_acquisition_reads_only_the_verified_local_path() {
        let dir = tempfile::tempdir().unwrap();
        let source = full_source_archive_fixture("rust-offline-positive");
        let archive_path = dir.path().join("rust-offline-positive.tar.gz");
        let expected = b"offline-rust-source";
        fs::write(&archive_path, expected).unwrap();
        let mut context = full_source_execution_context_fixture();
        context.rust_source_archive_dir = dir.path().to_path_buf();

        let observed = fetch_first_stage_source_bytes(&source, Some(&context)).unwrap();

        assert_eq!(observed.bytes, expected);
        assert_eq!(observed.source_policy, "authenticated-offline-only");
        assert_eq!(observed.source_input_path, Some(archive_path));
    }

    #[test]
    fn full_source_archive_acquisition_rejects_a_missing_local_source() {
        let dir = tempfile::tempdir().unwrap();
        let source = full_source_archive_fixture("rust-offline-missing");
        let mut context = full_source_execution_context_fixture();
        context.rust_source_archive_dir = dir.path().to_path_buf();

        let error = fetch_first_stage_source_bytes(&source, Some(&context)).unwrap_err().to_string();

        assert!(error.contains("offline source 'rust-offline-missing'"));
        assert!(error.contains("rust-offline-missing.tar.gz"));
        assert!(!error.contains("127.0.0.1"));
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
        assert_stage_construction_evidence(&output, "mrustc-to-rust-1.90.0");
        assert_stage_construction_evidence(&output, RUSTC_FINAL_STAGE_ID);
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
        assert_patch_plan_operation(
            &manifest,
            "mrustc-to-rust-1.90.0",
            "first-stage-minicargo-out-dir",
            FIRST_STAGE_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        assert_eq!(build_manifest["build"]["plan_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(build_manifest["build"]["script_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(build_manifest["build"]["parallel_job_count"], RUST_BOOTSTRAP_JOB_COUNT);
        assert_patch_plan_operation(
            &build_manifest,
            "mrustc-to-rust-1.90.0",
            "first-stage-run-rustc-host-runtime",
            FIRST_STAGE_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        let first_stage_receipt_path =
            scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR).join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH);
        assert_receipt_has_stage_construction_identity(&first_stage_receipt_path);
        assert_receipt_has_patch_plan_step(&first_stage_receipt_path, "first-stage-run-rustc-target");
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
        assert_patch_plan_operation(
            &rustc_stage1_plan,
            "rust-1.91.1-stage1",
            "rust-bootstrap-rustc-private-tool-rlibs",
            RUST_BOOTSTRAP_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        assert_patch_plan_operation(
            &rustc_stage1_build,
            "rust-1.91.1-stage1",
            "rust-bootstrap-target-tool-config",
            RUST_BOOTSTRAP_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        let rustc_stage1_receipt_path =
            scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR).join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH);
        assert_receipt_has_stage_construction_identity(&rustc_stage1_receipt_path);
        assert_receipt_has_patch_plan_step(&rustc_stage1_receipt_path, "rust-bootstrap-rustc-private-tool-rlibs");
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
        assert_patch_plan_operation(
            &rustc_final_plan,
            RUSTC_FINAL_STAGE_ID,
            "rust-bootstrap-rustc-private-tool-rlibs",
            RUST_BOOTSTRAP_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        assert_patch_plan_operation(
            &rustc_final_build,
            RUSTC_FINAL_STAGE_ID,
            "rust-bootstrap-rustc-private-tool-rlibs",
            RUST_BOOTSTRAP_PATCH_PLAN_MIN_OPERATIONS,
        );
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
        let rustc_final_receipt_path = rustc_final_root
            .join(RUSTC_FINAL_PROVIDER_CANDIDATE_DIR)
            .join(RUSTC_FINAL_PROVIDER_RECEIPT_RELATIVE_PATH);
        assert_receipt_has_stage_construction_identity(&rustc_final_receipt_path);
        assert_receipt_has_patch_plan_step(&rustc_final_receipt_path, "rust-bootstrap-rustc-private-tool-rlibs");
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
        let rustc_stage1_script = fs::read_to_string(scratch.join(RUSTC_STAGE1_SCRIPT_FILE)).unwrap();
        assert!(rustc_stage1_script.contains("using Rust bootstrap target linker wrapper"));
        assert!(rustc_stage1_script.contains("export TMPDIR=\"$TEMP_ROOT\""));
        assert!(!rustc_stage1_script.contains("TMPDIR=/tmp"));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_LLVM_BUILD_CONFIG.trim_end()));
        assert!(rustc_stage1_script.contains("\"LLVM_ENABLE_ZLIB\" = \"OFF\""));
        assert!(!rustc_stage1_script.contains("\"LLVM_ENABLE_ZLIB\" = \"ON\""));
        assert!(rustc_stage1_script.contains("LLVM_TOOL_LTO_BUILD"));
        assert!(rustc_stage1_script.contains("LLVM_TOOL_REMARKS_SHLIB_BUILD"));
        assert!(
            rustc_stage1_script
                .contains("normalizing Rust bootstrap rustc_driver crate type for static musl compiler host")
        );
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_RUSTC_DRIVER_MANIFEST));
        assert!(rustc_stage1_script.contains(
            "Rust bootstrap rustc_driver manifest lacks expected crate-type line for musl host normalization"
        ));
        assert!(
            rustc_stage1_script.contains("normalizing Rust bootstrap sysroot fallback for static musl compiler host")
        );
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_FILESEARCH_SOURCE));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_FILESEARCH_ENV_FALLBACK_LINE));
        assert!(rustc_stage1_script.contains(
            "Rust bootstrap filesearch source lacks expected sysroot fallback lines for musl host normalization"
        ));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_MARKER));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_TOOL_BUILD_SOURCE));
        assert!(rustc_stage1_script.contains("RUSTC_ADDITIONAL_SYSROOT_PATHS"));
        assert!(rustc_stage1_script.contains("Mode::ToolRustcPrivate"));
        assert!(rustc_stage1_script.contains("Mode::Rustc, target"));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_ERROR));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_TARGET_LINKER_ALIAS_DIR));
        assert!(rustc_stage1_script.contains(RUSTC_SOURCE_TARGET_LINKER_RUNTIME_DIR));
        assert!(
            rustc_stage1_script.contains(
                "Rust bootstrap target toolchain does not expose musl/gcc CRT, unwinder, and objcopy objects"
            )
        );
        assert!(rustc_stage1_script.contains("target_objcopy_program=$target_tool_prefix-objcopy"));
        assert!(rustc_stage1_script.contains("--remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array"));
        assert!(rustc_stage1_script.contains("rcrt1.o|*/rcrt1.o) mapped_arg=\"$target_runtime_dir/crt1.o\""));
        assert!(rustc_stage1_script.contains("-lgcc_s) continue"));
        assert!(rustc_stage1_script.contains("dynamic_rustc_link=false"));
        assert!(rustc_stage1_script.contains(
            "*/stage1/bin/rustc|stage1/bin/rustc|*/stage2/bin/rustc|stage2/bin/rustc) dynamic_rustc_link=true"
        ));
        assert!(rustc_stage1_script.contains("dynamic_executable_link=false"));
        assert!(rustc_stage1_script.contains("shared_link=false"));
        assert!(rustc_stage1_script.contains("-pie) static_support_link=false; dynamic_executable_link=true"));
        assert!(rustc_stage1_script.contains("-shared|-dynamiclib) shared_link=true"));
        assert!(rustc_stage1_script.contains("-shared|-dynamiclib) static_support_link=false; shared_link=true"));
        assert!(rustc_stage1_script.contains(
            "-static) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"$arg\"; fi"
        ));
        assert!(rustc_stage1_script.contains(
            "-static-pie) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"-static\"; fi"
        ));
        assert!(rustc_stage1_script.contains("-B\"$target_runtime_dir/\" -L\"$target_runtime_dir\""));
        assert!(rustc_stage1_script.contains("MANTLE_RUST_TOOLS=${MANTLE_RUST_BOOTSTRAP_TOOLS:-'[\"cargo\"]'}"));
        assert!(rustc_stage1_script.contains("*' rustdoc '*) MANTLE_RUST_TOOLS='[\"cargo\", \"rustdoc\"]'"));
        assert!(rustc_stage1_script.contains("tools = $MANTLE_RUST_TOOLS"));
        assert!(rustc_stage1_script.contains(&format!("jobs = {RUST_BOOTSTRAP_JOB_COUNT}")));
        assert!(!rustc_stage1_script.contains("jobs = $"));
        assert!(rustc_stage1_script.contains("cargo-native-static = true"));
        assert!(
            rustc_stage1_script.contains("Rust bootstrap target musl libc.so missing for dynamic compiler host links")
        );
        assert!(rustc_stage1_script.contains("$target_orig_cc_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib"));
        assert!(rustc_stage1_script.contains("set -- \"$@\" -Wl,-Bstatic -lunwind -lgcc -Wl,-Bdynamic"));
        assert!(rustc_stage1_script.contains(
            "{ [ \"$dynamic_rustc_link\" = true ] || [ \"$dynamic_executable_link\" = true ]; }; then set -- \"$@\" -no-pie -Wl,-Bdynamic \"-Wl,-dynamic-linker,$target_runtime_dir/libc.so\" -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group"
        ));
        assert!(
            rustc_stage1_script
                .contains("set -- \"$@\" -static -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group")
        );
        assert!(rustc_stage1_script.contains("crt-static = false"));
        assert!(rustc_stage1_script.contains("__atomic_compare_exchange_16"));
        assert!(rustc_stage1_script.contains("linker = \"$MANTLE_TARGET_CC\""));
        assert!(rustc_stage1_script.contains(
            "if [ -n \"${MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG:-}\" ]; then set -- \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" \"$@\"; fi"
        ));
        let script = fs::read_to_string(scratch.join(FIRST_STAGE_SCRIPT_FILE)).unwrap();
        assert!(script.contains("missing verified source"));
        assert!(script.contains("verified sources manifest"));
        assert!(script.contains("TEMP_ROOT=\"$BUILD_DIR/tmp\""));
        assert!(script.contains("export TMPDIR=\"$TEMP_ROOT\""));
        assert!(script.contains("export TMP=\"$TEMP_ROOT\""));
        assert!(script.contains("export TEMP=\"$TEMP_ROOT\""));
        assert!(script.contains("export TEMPDIR=\"$TEMP_ROOT\""));
        assert!(script.contains(&format!("PARLEVEL={RUST_BOOTSTRAP_JOB_COUNT}\nexport PARLEVEL")));
        assert!(script.contains(&format!("$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} CC=\"$CC\"")));
        assert!(!script.contains("$MAKE_PROGRAM CC=\"$CC\""));
        assert!(!script.contains("PARLEVEL=${"));
        assert!(!script.contains("TMPDIR=/tmp"));
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
        assert!(script.contains("target_toolchain_root=${MANTLE_TARGET_TOOLCHAIN_ROOT:-${SOURCE_ROOT:-}}"));
        assert!(script.contains("using explicit source-root musl target toolchain"));
        assert!(script.contains(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING));
        assert!(script.contains(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE));
        assert!(script.contains("TARGET_CC_PROGRAM_CANDIDATES='x86_64-unknown-linux-musl-gcc x86_64-linux-musl-gcc'"));
        assert!(script.contains("TARGET_MUSL_MACHINE_ALIASES='x86_64-unknown-linux-musl x86_64-linux-musl'"));
        assert!(script.contains("TARGET_MUSL_SOURCE_ROOT_SYSROOT='x86_64-linux-musl'"));
        assert!(script.contains("target_libc_root=\"$target_wrapper_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT\""));
        assert!(script.contains("$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib"));
        assert!(script.contains("target_gcc_crt_machine=$target_cc_machine"));
        assert!(script.contains("target_cxx_program=$target_tool_prefix-g++"));
        assert!(script.contains("target_objcopy_program=$target_tool_prefix-objcopy"));
        assert!(script.contains("target gcc toolchain does not expose objcopy for CRT normalization"));
        assert!(script.contains("--remove-section .init_array --remove-section .rela.init_array --remove-section .fini_array --remove-section .rela.fini_array"));
        assert!(script.contains("target_crtbegin_no_frame_init=\"$target_runtime_dir/crtbeginS.o.no-frame-init\""));
        assert!(script.contains("preparing source-root musl LLVM host compiler runtime"));
        assert!(script.contains("source-root musl LLVM host wrapper tools are incomplete"));
        assert!(script.contains("source-root musl libstdc++.a missing for LLVM host build"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LIBSTDCXX_STATIC_ARCHIVE));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LIBATOMIC_STATIC_ARCHIVE));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SHIM_SOURCE));
        assert!(script.contains("__atomic_compare_exchange_16"));
        assert!(script.contains("ATOMIC_WIDTH_BYTES"));
        assert!(script.contains("--target $RUSTC_HOST_TRIPLE"));
        assert!(script.contains("CC_PROGRAM=\"$target_alias_dir/cc\""));
        assert!(script.contains("CXX_PROGRAM=\"$target_alias_dir/c++\""));
        assert!(script.contains("CC=\"$CC_PROGRAM\""));
        assert!(script.contains("CXX=\"$CXX_PROGRAM\""));
        assert!(script.contains("CFLAGS=\"$ZLIB_CFLAGS\""));
        assert!(script.contains("CXXFLAGS=\"$MRUSTC_CXXFLAGS $ZLIB_CFLAGS\""));
        assert!(script.contains("LDFLAGS=\"$ZLIB_LIBS\""));
        assert!(script.contains("CMAKE_C_COMPILER=\"$target_alias_dir/cc\""));
        assert!(script.contains("CMAKE_CXX_COMPILER=\"$target_alias_dir/c++\""));
        assert!(script.contains("CMAKE_AR=\"$target_alias_dir/ar\""));
        assert!(script.contains("CMAKE_RANLIB=\"$target_alias_dir/ranlib\""));
        assert!(script.contains("MRUSTC_CXXFLAGS=\"$MRUSTC_CXXFLAGS -static-libstdc++ -static-libgcc\""));
        assert!(script.contains("ZLIB_CFLAGS=\nZLIB_LIBS="));
        assert!(script.contains("LLVM_STATIC_STDCPP=\"$target_runtime_dir/libstdc++.a\""));
        assert!(script.contains("LLVM_LINKER_FLAGS=\"-L$target_runtime_dir -lgcc -lunwind"));
        assert!(script.contains("PATH=\"$target_alias_dir:$PATH\""));
        assert!(script.contains("export PATH"));
        assert!(script.contains("static_pie_normalized=false"));
        assert!(script.contains("MANTLE_TARGET_CC_PATH"));
        assert!(script.contains("-shared|-dynamiclib) shared_link=true"));
        assert!(script.contains("-shared|-dynamiclib) static_support_link=false"));
        assert!(
            script.contains("-static) if [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"$arg\"; fi")
        );
        assert!(script.contains(
            "-static-pie) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"-static\"; fi"
        ));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_PTHREAD_TLS_WRAP_FLAGS));
        assert!(script.contains(
            "-static -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group"
        ));
        assert!(script.contains("-static-pie) static_pie_normalized=true"));
        assert!(script.contains("dynamic_rustc_link=false"));
        assert!(script.contains("@*/output/rustc-build/rustc_main_cmd.txt|@output/rustc-build/rustc_main_cmd.txt) rustc_main_response_file=true"));
        assert!(script.contains("if [ \"$rustc_main_response_file\" = true ]; then dynamic_rustc_link=true; fi"));
        assert!(script.contains("*/output/rustc|output/rustc|*/output/rustc-build/rustc_main|output/rustc-build/rustc_main) dynamic_rustc_link=true"));
        assert!(script.contains("if [ \"$link_command\" = true ] && [ \"$dynamic_rustc_link\" = true ]; then set -- \"$@\" \"$target_runtime_dir/musl-lfs-compat.o\" -no-pie -Wl,-Bdynamic \"-Wl,-dynamic-linker,$target_runtime_dir/libc.so\" -Wl,--wrap=pthread_key_create -Wl,--wrap=pthread_key_delete -Wl,--wrap=pthread_getspecific -Wl,--wrap=pthread_setspecific -Wl,--start-group -latomic -lunwind -lgcc -Wl,--end-group"));
        assert!(script.contains("rcrt1.o|*/rcrt1.o) mapped_arg=\"$target_runtime_dir/crt1.o\""));
        assert!(script.contains("crt1.o|*/crt1.o|Scrt1.o|*/Scrt1.o|crti.o|*/crti.o|crtn.o|*/crtn.o|crtbeginS.o|*/crtbeginS.o|crtendS.o|*/crtendS.o) mapped_arg=\"$target_runtime_dir/${arg##*/}\""));
        assert!(script.contains(
            "-static-pie) if [ \"$dynamic_rustc_link\" = true ] || [ \"$shared_link\" = true ]; then continue; else mapped_arg=\"-static\"; fi"
        ));
        assert!(script.contains("$target_runtime_dir/musl-lfs-compat.o\" -no-pie -Wl,-Bdynamic"));
        assert!(!script.contains("-Wl,-Bdynamic -pie"));
        assert!(script.contains(FIRST_STAGE_TARGET_LINKER_ALIAS_DIR));
        assert!(script.contains(FIRST_STAGE_TARGET_LINKER_RUNTIME_DIR));
        assert!(script.contains(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE));
        assert!(script.contains(FIRST_STAGE_TARGET_NIX_ORIG_CC_FILE));
        assert!(script.contains(&format!("export {FIRST_STAGE_TARGET_NIX_CC_WRAPPER_HOST_ROLE_VAR}=1")));
        assert!(script.contains("crt1.o"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_DYNAMIC_PIE_CRT_OBJECT));
        assert!(script.contains("rcrt1.o"));
        assert!(script.contains("crtbeginS.o"));
        assert!(script.contains("libgcc.a"));
        assert!(script.contains("libgcc_eh.a"));
        assert!(script.contains("libgcc_s.so"));
        assert!(script.contains("libgcc_s.so.1"));
        assert!(script.contains("libunwind.a"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT));
        assert!(script.contains("preparing source-root musl proc-macro runtime search path"));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_LD_LIBRARY_PATH_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_STAGE2_ENV_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_STAGE2_RUNTIME_ENV_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_FINAL_ENV_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_FINAL_PREFIX2_ENV_LINE));
        assert!(script.contains("run_rustc_ld_runtime_line"));
        assert!(script.contains("run_rustc_stage2_runtime_line"));
        assert!(script.contains("run_rustc_final_prefix2_line"));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_RUNTIME_LD_LIBRARY_PATH_TEMPLATE));
        assert!(script.contains("\\$(abspath \\$(PREFIX_2)lib)"));
        assert!(script.contains("source-root musl libc.so missing for proc-macro runtime"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected LD_LIBRARY_PATH line"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected stage2 Cargo env line"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected final rustc Cargo env line"));
        assert!(script.contains("normalizing run_rustc Cargo static feature set for source-root musl host"));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_ALL_STATIC_LINE));
        assert!(
            script.contains("mrustc run_rustc Makefile lacks expected Cargo build line for all-static normalization")
        );
        assert!(script.contains("normalizing minicargo rustc worker threads for static musl compiler host"));
        assert!(script.contains("disabling LLVM shared-tool and execinfo backtraces for source-root musl host"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_ORIGINAL_LINE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_PATCHED_LINE));
        let llvm_config_patched_line = first_stage_minicargo_makefile_llvm_config_build_patched_line();
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_CONFIG_BUILD_ORIGINAL_LINE));
        assert!(script.contains(&llvm_config_patched_line));
        assert!(script.contains("llvm-headers vt_gen llvm-config"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_STATIC_ARCHIVE_TARGETS));
        assert!(script.contains("LLVMLTO"));
        assert!(script.contains("LLVMX86CodeGen"));
        assert!(script.contains("minicargo Makefile lacks expected LLVM CMake options line for musl normalization"));
        assert!(script.contains("minicargo Makefile lacks expected llvm-config build line for musl normalization"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_RUSTC_FORCE_UNSTABLE_LINE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_RUSTC_THREADS_MARKER_LINE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_RUSTC_THREADS_FLAG_LINE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_RUSTC_THREADS_VALUE_LINE));
        assert!(
            script.contains("minicargo build.cpp lacks expected rustc force-unstable line for thread normalization")
        );
        assert!(script.contains("normalizing Rust explicit sysroot handling for static musl rustc"));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SOURCE));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_ORIGINAL_LINE));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_1));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_2));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SYSROOT_REPLACEMENT_LINE_3));
        assert!(script.contains("rustc_session Sysroot::new no longer has expected default_sysroot shape"));
        assert!(script.contains(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} RUSTCSRC"
        )));
        assert!(script.contains("normalizing run_rustc static rustc sysroot wrappers"));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_STAGE1_RUSTC_RULE_LINE));
        assert!(script.contains(FIRST_STAGE_RUN_RUSTC_STAGE2_RUSTC_RULE_LINE));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE_SYSROOT_SYMLINK_LINE)));
        assert!(script.contains("run_rustc_stage1_wrapper_line="));
        assert!(script.contains("run_rustc_stage2_wrapper_line="));
        assert!(script.contains("$target_runtime_dir/libc.so"));
        assert!(script.contains("--sysroot \\\"\\$(abspath \\$(PREFIX_S))"));
        assert!(script.contains("--sysroot \\\"\\$(abspath \\$(PREFIX_2))"));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_SYMLINK_LINE)));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_WRAPPER_LINE)));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected stage1 rustc copy rule"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected stage2 rustc copy rule"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected final rustc symlink rule"));
        assert!(script.contains("mrustc run_rustc Makefile lacks expected final rustc wrapper"));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_SOURCE));
        assert!(script.contains(FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_OBJECT));
        assert!(script.contains("-fPIC -c \"$target_lfs_compat_source\""));
        assert!(
            script.contains(&format!("MANTLE_PTHREAD_TLS_KEY_CAPACITY {}u", FIRST_STAGE_TARGET_MUSL_TLS_KEY_CAPACITY))
        );
        assert!(script.contains("__wrap_pthread_key_create"));
        assert!(script.contains("__wrap_pthread_getspecific"));
        assert!(script.contains("__wrap_pthread_setspecific"));
        assert!(script.contains("__wrap_pthread_key_delete"));
        assert!(script.contains("int backtrace(void **buffer, int size)"));
        assert!(script.contains("void backtrace_symbols_fd(void *const *buffer, int size, int fd)"));
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
        assert!(script.contains("normalizing minicargo build-script OUT_DIR for static musl compiler host"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_BUILD_SOURCE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_OUT_DIR_ORIGINAL_LINE));
        assert!(script.contains(FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_MARKER));
        assert!(script.contains(FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_LINE));
        assert!(script.contains("minicargo build.cpp lacks expected OUT_DIR line"));
        assert!(script.contains("normalizing rustc_driver crate type for static musl compiler host"));
        assert!(script.contains(FIRST_STAGE_RUSTC_DRIVER_MANIFEST));
        assert!(script.contains(FIRST_STAGE_RUSTC_DRIVER_DYLIB_CRATE_TYPE_LINE));
        assert!(script.contains(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE));
        assert!(script.contains("rustc_driver_replaced=false"));
        assert!(script.contains("rustc_driver_seen_rlib=false"));
        assert!(script.contains("rustc_driver manifest lacks expected crate-type line for musl host normalization"));
        assert!(script.contains("target_outdir_suffix"));
        assert!(script.contains("target_sysroot_source=\"rustc-${RUSTC_VERSION}-src/library/sysroot\""));
        assert!(script.contains("MRUSTC_PATH=\"$(pwd)/$target_bin_dir/rustc\""));
        assert!(script.contains("--target $RUSTC_TARGET"));
        assert!(script.contains(FIRST_STAGE_TARGET_PREFIX_S_DIR));
        assert!(script.contains("$COPY_PROGRAM output/rustc \"$target_bin_dir/rustc\""));
        assert!(script.contains("target rustlib build did not produce libstd.rlib"));
        assert!(script.contains("unset CARGO_PKG_VERSION"));
        assert!(script.contains("unset CARGO_BUILD_RUSTC_WRAPPER"));
        assert!(script.contains("unset MANTLE_RUST_CACHE_POLICY"));
        assert!(script.contains("unset MANTLE_RUSTC_MANIFEST"));
        assert!(script.contains("unset MANTLE_RUSTC_MANIFEST_DIR"));
        assert!(script.contains("unset MANTLE_RUSTC_MANIFEST_REF"));
        assert!(script.contains("unset CARGO_MANIFEST_DIR"));
        assert!(script.contains("unset OUT_DIR"));
        assert!(script.contains("unset TARGET"));
        assert!(script.contains("unset HOST"));
        assert!(!script.contains("unset RUSTC_TARGET"));
        let scrub_index = script.find("unset CARGO_PKG_VERSION").unwrap();
        let workspace_index = script.find("writing minicargo workspace boundary").unwrap();
        let minicargo_out_dir_patch_index =
            script.find("normalizing minicargo build-script OUT_DIR for static musl compiler host").unwrap();
        let minicargo_build_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_MINICARGO_BINARY}"
            ))
            .unwrap();
        let rust_source_extract_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} RUSTCSRC"
            ))
            .unwrap();
        let rust_explicit_sysroot_patch_index =
            script.find("normalizing Rust explicit sysroot handling for static musl rustc").unwrap();
        let translated_rustc_build_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_RUSTC_BINARY}"
            ))
            .unwrap();
        let translated_cargo_build_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_CARGO_BINARY}"
            ))
            .unwrap();
        let compiler_host_linker_index = translated_cargo_build_index
            + script[translated_cargo_build_index..].find("using compiler-host linker").unwrap();
        let proc_macro_runtime_index =
            script.find("preparing source-root musl proc-macro runtime search path").unwrap();
        let host_run_rustc_index = script
            .find(&format!(
                "if [ -n \"$RUN_RUSTC_DYLIB_EXT\" ]; then $MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -C {FIRST_STAGE_RUN_RUSTC_DIR}"
            ))
            .unwrap();
        let target_assignment_index = script.rfind("RUSTC_TARGET=\"$RUSTC_PROVIDER_TARGET_TRIPLE\"").unwrap();
        let target_root_preference_index =
            script.rfind("target_toolchain_root=${MANTLE_TARGET_TOOLCHAIN_ROOT:-${SOURCE_ROOT:-}}").unwrap();
        let target_nix_fallback_index = script.rfind("for candidate in $TARGET_MUSL_GCC_FALLBACK_GLOB").unwrap();
        let target_linker_index = script.rfind("using target linker wrapper").unwrap();
        let target_sysroot_index = script.rfind("target_sysroot_source").unwrap();
        let llvm_host_setup_index = script.find("preparing source-root musl LLVM host compiler runtime").unwrap();
        assert!(scrub_index < minicargo_build_index);
        assert!(workspace_index < minicargo_build_index);
        assert!(minicargo_out_dir_patch_index < minicargo_build_index);
        assert!(minicargo_build_index < llvm_host_setup_index);
        assert!(llvm_host_setup_index < rust_source_extract_index);
        assert!(rust_source_extract_index < rust_explicit_sysroot_patch_index);
        assert!(rust_explicit_sysroot_patch_index < translated_rustc_build_index);
        assert!(translated_rustc_build_index < translated_cargo_build_index);
        assert!(translated_cargo_build_index < compiler_host_linker_index);
        assert!(compiler_host_linker_index < proc_macro_runtime_index);
        assert!(proc_macro_runtime_index < host_run_rustc_index);
        assert!(host_run_rustc_index < target_assignment_index);
        assert!(target_assignment_index < target_root_preference_index);
        assert!(target_root_preference_index < target_nix_fallback_index);
        assert!(target_nix_fallback_index < target_linker_index);
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
        let script = fs::read_to_string(scratch.join(FIRST_STAGE_SCRIPT_FILE)).unwrap();
        assert!(script.contains(&format!("if [ \"$RUSTC_HOST_TRIPLE\" = \"{FIRST_STAGE_MUSL_TRIPLE}\" ]; then")));
        assert!(script.contains("preparing source-root musl LLVM host compiler runtime"));
        assert!(script.contains("using source-root musl LLVM host wrappers"));
        assert!(script.contains("CC_PROGRAM=\"$target_alias_dir/cc\""));
        assert!(script.contains("LLVM_STATIC_STDCPP=\"$target_runtime_dir/libstdc++.a\""));
        assert!(script.contains("LLVM_LINKER_FLAGS=\"-L$target_runtime_dir -lgcc -lunwind"));
        assert!(script.contains("normalizing minicargo build-script OUT_DIR for static musl compiler host"));
        assert!(script.contains("disabling LLVM shared-tool and execinfo backtraces for source-root musl host"));
        let llvm_config_patched_line = first_stage_minicargo_makefile_llvm_config_build_patched_line();
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_PATCHED_LINE));
        assert!(script.contains(&llvm_config_patched_line));
        assert!(script.contains(FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_STATIC_ARCHIVE_TARGETS));
        assert!(script.contains(FIRST_STAGE_MINICARGO_OUT_DIR_PATCH_MARKER));
        assert!(script.contains("normalizing Rust explicit sysroot handling for static musl rustc"));
        assert!(script.contains(FIRST_STAGE_RUSTC_CONFIG_SOURCE));
        assert!(script.contains(&format!(
            "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} RUSTCSRC"
        )));
        assert!(script.contains("normalizing run_rustc static rustc sysroot wrappers"));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_STAGE_SYSROOT_SYMLINK_LINE)));
        assert!(script.contains("run_rustc_stage1_wrapper_line="));
        assert!(script.contains("$target_runtime_dir/libc.so"));
        assert!(script.contains("--sysroot \\\"\\$(abspath \\$(PREFIX_S))"));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_SYMLINK_LINE)));
        assert!(script.contains(&shell_quote(FIRST_STAGE_RUN_RUSTC_FINAL_SYSROOT_WRAPPER_LINE)));
        let mrustc_source = scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0");
        let minicargo_makefile = fs::read_to_string(mrustc_source.join(FIRST_STAGE_MINICARGO_MAKEFILE)).unwrap();
        assert!(minicargo_makefile.contains("LLVM_ENABLE_ZSTD=OFF"));
        assert!(minicargo_makefile.contains("CMAKE_DISABLE_FIND_PACKAGE_zstd=ON"));
        assert!(
            !minicargo_makefile
                .lines()
                .any(|line| line == FIRST_STAGE_MINICARGO_MAKEFILE_LLVM_BACKTRACE_ORIGINAL_LINE)
        );
        let run_rustc_makefile =
            fs::read_to_string(mrustc_source.join(FIRST_STAGE_RUN_RUSTC_DIR).join(FIRST_STAGE_MAKEFILE)).unwrap();
        assert!(run_rustc_makefile.contains(FIRST_STAGE_RUN_RUSTC_FINAL_PREFIX2_ENV_LINE));
        assert!(!run_rustc_makefile.contains(FIRST_STAGE_RUN_RUSTC_FINAL_ENV_LINE));
        assert!(run_rustc_makefile.lines().any(|line| line == FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_ALL_STATIC_LINE));
        assert!(!run_rustc_makefile.lines().any(|line| line == FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE));
        let llvm_host_setup_index = script.find("preparing source-root musl LLVM host compiler runtime").unwrap();
        let minicargo_build_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_MINICARGO_BINARY}"
            ))
            .unwrap();
        let rust_source_extract_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} RUSTCSRC"
            ))
            .unwrap();
        let rust_explicit_sysroot_patch_index =
            script.find("normalizing Rust explicit sysroot handling for static musl rustc").unwrap();
        let first_rustc_build_index = script
            .find(&format!(
                "$MAKE_PROGRAM {FIRST_STAGE_MAKE_PARALLEL_ARG} -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_RUSTC_BINARY}"
            ))
            .unwrap();
        assert!(minicargo_build_index < llvm_host_setup_index);
        assert!(llvm_host_setup_index < rust_source_extract_index);
        assert!(rust_source_extract_index < rust_explicit_sysroot_patch_index);
        assert!(rust_explicit_sysroot_patch_index < first_rustc_build_index);
        assert!(script.contains(FIRST_STAGE_RUSTC_DRIVER_MANIFEST));
        assert!(script.contains(FIRST_STAGE_RUSTC_DRIVER_RLIB_CRATE_TYPE_LINE));
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
    fn first_stage_cargo_all_static_patch_rejects_missing_makefile_line() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        let broken_run_rustc_makefile = std::str::from_utf8(test_mrustc_run_rustc_makefile())
            .unwrap()
            .replace(FIRST_STAGE_RUN_RUSTC_CARGO_BUILD_LINE, "\tprintf 'missing cargo build line\\n'");
        let sources =
            write_test_source_archives_with_mrustc_run_rustc_makefile(dir.path(), broken_run_rustc_makefile.as_bytes());
        write_test_route_plan_with_sources(dir.path(), &sources, &sources.mrustc_sha256_hex);
        rewrite_test_route_plan_to_musl_host(dir.path());

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("first-stage mrustc/minicargo build failed"));
        let build_log = fs::read_to_string(scratch.join(FIRST_STAGE_BUILD_LOG_FILE)).unwrap();
        assert!(
            build_log
                .contains("mrustc run_rustc Makefile lacks expected Cargo build line for all-static normalization")
        );
        assert!(!output.join(RUST_SOURCE_PROVIDER_METADATA_PATH).exists());
    }

    #[test]
    fn materializer_rejects_invalid_explicit_source_root_env_in_subprocess() {
        let invalid_root_parent = tempfile::tempdir().unwrap();
        let invalid_root = invalid_root_parent.path().join("missing-source-root");
        let output = Command::new(std::env::current_exe().unwrap())
            .arg("rust_source_provider::tests::materializer_invalid_explicit_source_root_child")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_TEST_INVALID_SOURCE_ROOT_CHILD", "1")
            .env(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_VAR, &invalid_root)
            .env_remove(FIRST_STAGE_SOURCE_ROOT_VAR)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "child status: {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn materializer_invalid_explicit_source_root_child() {
        if std::env::var_os("MANTLE_TEST_INVALID_SOURCE_ROOT_CHILD").is_none() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "source-built recipe\n").unwrap();
        write_test_musl_host_route_plan(dir.path());

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("first-stage mrustc/minicargo build failed"));
        let build_log = fs::read_to_string(scratch.join(FIRST_STAGE_BUILD_LOG_FILE)).unwrap();
        assert!(build_log.contains(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING));
        assert!(!output.exists());
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
        assert!(rustc_stage1_log.contains("install rustc cargo library/std"));
        assert!(!rustc_stage1_log.contains("install rustc cargo rustdoc"));
        let generated_stage1_script =
            fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_DIR).join(RUSTC_STAGE1_GENERATED_BUILD_SCRIPT)).unwrap();
        assert!(generated_stage1_script.contains("change-id = \"ignore\""));
        assert!(generated_stage1_script.contains("cargo-native-static = true"));
        assert!(generated_stage1_script.contains("ninja = false"));
        assert!(generated_stage1_script.contains("sysconfdir = \"etc\""));
        assert!(generated_stage1_script.contains("target = [\"$MANTLE_HOST_TRIPLE\", \"$MANTLE_TARGET_TRIPLE\"]"));
        assert!(generated_stage1_script.contains(FIRST_STAGE_MAKE_FALLBACK_GLOB));
        assert!(generated_stage1_script.contains(FIRST_STAGE_CMAKE_FALLBACK_GLOB));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB));
        assert!(
            generated_stage1_script
                .contains("MANTLE_TARGET_TOOLCHAIN_ROOT=${MANTLE_TARGET_TOOLCHAIN_ROOT:-${SOURCE_ROOT:-}}")
        );
        assert!(generated_stage1_script.contains("using explicit Rust bootstrap source-root musl target toolchain"));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_GCC_MISSING));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_TOOLCHAIN_ROOT_INCOMPLETE));
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
        assert!(generated_stage1_script.contains(
            "if [ -n \"${MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG:-}\" ]; then set -- \"$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG\" \"$@\"; fi"
        ));
        assert!(generated_stage1_script.contains(FIRST_STAGE_TARGET_NIX_ORIG_LIBC_FILE));
        assert!(generated_stage1_script.contains("musl-root = \"$MANTLE_TARGET_MUSL_ROOT\""));
        let rustc_source_root_preference_index = generated_stage1_script
            .find("MANTLE_TARGET_TOOLCHAIN_ROOT=${MANTLE_TARGET_TOOLCHAIN_ROOT:-${SOURCE_ROOT:-}}")
            .unwrap();
        let rustc_nix_fallback_index = generated_stage1_script
            .find(&format!("for candidate in {FIRST_STAGE_TARGET_MUSL_GCC_FALLBACK_GLOB}"))
            .unwrap();
        assert!(rustc_source_root_preference_index < rustc_nix_fallback_index);
        assert!(generated_stage1_script.contains(RUSTC_SOURCE_CRANELIFT_MANIFEST));
        assert!(generated_stage1_script.contains(RUSTC_SOURCE_CODEGEN_GCC_MANIFEST));
        assert!(generated_stage1_script.contains(RUSTC_SOURCE_TOOL_BUILD_RLIB_SYSROOT_MARKER));
        assert!(generated_stage1_script.contains("RUSTC_ADDITIONAL_SYSROOT_PATHS"));
        let rustc_final_log = fs::read_to_string(rustc_final_root.join(RUSTC_FINAL_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_final_log.contains("scrubbing inherited Rust bootstrap Cargo environment"));
        assert!(rustc_final_log.contains("isolated Rust Cargo workspace"));
        assert!(rustc_final_log.contains("using generated x.py Rust build adapter"));
        assert!(rustc_final_log.contains("synthetic x.py for 1.94.0"));
        assert!(rustc_final_log.contains("install rustc cargo library/std"));
        assert!(!rustc_final_log.contains("install rustc cargo rustdoc"));
        let generated_final_script =
            fs::read_to_string(rustc_final_root.join(RUSTC_FINAL_BUILD_DIR).join(RUSTC_FINAL_GENERATED_BUILD_SCRIPT))
                .unwrap();
        assert!(generated_final_script.contains("tools = $MANTLE_RUST_TOOLS"));
        let final_script = fs::read_to_string(rustc_final_root.join(RUSTC_FINAL_SCRIPT_FILE)).unwrap();
        assert!(final_script.contains("MANTLE_RUST_BOOTSTRAP_TOOLS='[\"cargo\", \"rustdoc\"]'"));
        assert!(final_script.contains("export TMPDIR=\"$TEMP_ROOT\""));
        assert!(!final_script.contains("TMPDIR=/tmp"));
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

        assert!(
            err.to_string().contains("disallowed Rust provider marker")
                || err.to_string().contains("uses prebuilt Rust")
        );
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
        let rustc_args = [OsString::from("-o"), output_path.as_os_str().to_os_string()];
        let output =
            run_smoke_rustc_with_bounded_launch_retry(&provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH), &rustc_args)
                .unwrap();

        assert!(!wrapper.contains(MRUSTC_RUSTC_WRAPPER_DIRNAME_MARKER));
        assert!(wrapper.contains("self_dir=${self%/*}"));
        assert!(wrapper.contains("LD_LIBRARY_PATH=\"$ld_path\""));
        assert!(wrapper.contains(PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_BASENAME));
        assert!(wrapper.contains("--sysroot \"$root_dir\""));
        assert!(
            fs::symlink_metadata(provider_dir.join(PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_RELATIVE_PATH))
                .unwrap()
                .file_type()
                .is_symlink()
        );
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
    fn first_stage_proc_macro_rustc_install_writes_relocatable_loader_wrapper() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let dynamic_rustc = dir.path().join("dynamic-rustc");
        let runtime_loader = dir.path().join("libc.so");
        let mut dynamic_rustc_bytes = ELF_MAGIC.to_vec();
        dynamic_rustc_bytes.extend_from_slice(b"dynamic rustc");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        write_bytes(&provider_dir, PROVIDER_RUSTC_RELATIVE_PATH, b"old wrapper");
        fs::write(&dynamic_rustc, &dynamic_rustc_bytes).unwrap();
        fs::write(&runtime_loader, b"runtime loader").unwrap();
        fs::write(
            runtime_loader.with_file_name(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT),
            b"versioned shared libgcc runtime",
        )
        .unwrap();
        fs::write(
            runtime_loader.with_file_name(FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT),
            b"shared libgcc linker name",
        )
        .unwrap();

        install_first_stage_proc_macro_capable_rustc_from_paths(
            &provider_dir,
            &dynamic_rustc,
            &runtime_loader,
            FIRST_STAGE_MUSL_TRIPLE,
        )
        .unwrap();
        let wrapper = fs::read_to_string(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).unwrap();

        assert!(wrapper.contains(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH));
        assert!(wrapper.contains("exec \"$loader\" \"$self_dir/rustc_binary\""));
        assert!(wrapper.contains("--sysroot \"$root_dir\""));
        assert!(wrapper.contains("LD_LIBRARY_PATH=\"$ld_path\""));
        assert_eq!(
            fs::read(provider_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH)).unwrap(),
            dynamic_rustc_bytes
        );
        assert_eq!(
            fs::read(provider_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH)).unwrap(),
            b"runtime loader"
        );
        assert_eq!(
            fs::read(
                provider_dir
                    .join(FIRST_STAGE_PROC_MACRO_RUNTIME_DIR)
                    .join(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT)
            )
            .unwrap(),
            b"versioned shared libgcc runtime"
        );
        assert_eq!(
            fs::read(
                provider_dir
                    .join(FIRST_STAGE_PROC_MACRO_RUNTIME_DIR)
                    .join(FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT)
            )
            .unwrap(),
            b"shared libgcc linker name"
        );
        assert!(
            fs::symlink_metadata(provider_dir.join(PROVIDER_MRUSTC_RUSTC_SYSROOT_BINARY_RELATIVE_PATH))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn first_stage_proc_macro_rustc_install_rejects_missing_shared_libgcc_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let dynamic_rustc = dir.path().join("dynamic-rustc");
        let runtime_loader = dir.path().join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT);
        let mut dynamic_rustc_bytes = ELF_MAGIC.to_vec();
        dynamic_rustc_bytes.extend_from_slice(b"dynamic rustc");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        fs::write(&dynamic_rustc, &dynamic_rustc_bytes).unwrap();
        fs::write(&runtime_loader, b"runtime loader").unwrap();

        let err = install_first_stage_proc_macro_capable_rustc_from_paths(
            &provider_dir,
            &dynamic_rustc,
            &runtime_loader,
            FIRST_STAGE_MUSL_TRIPLE,
        )
        .unwrap_err();

        assert!(err.to_string().contains("dynamic runtime shared object is missing"));
        assert!(err.to_string().contains(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT));
        assert!(!provider_dir.join(PROVIDER_MRUSTC_RUSTC_BINARY_RELATIVE_PATH).exists());
    }

    #[test]
    fn rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let runtime_source_dir = dir.path().join("runtime-source");
        let runtime_loader = runtime_source_dir.join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT);
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        fs::create_dir_all(&runtime_source_dir).unwrap();
        fs::write(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH), b"rustc binary").unwrap();
        fs::write(provider_dir.join(PROVIDER_CARGO_RELATIVE_PATH), b"cargo binary").unwrap();
        fs::write(&runtime_loader, b"runtime loader").unwrap();
        fs::write(
            runtime_source_dir.join(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT),
            b"versioned shared libgcc runtime",
        )
        .unwrap();
        fs::write(runtime_source_dir.join(FIRST_STAGE_TARGET_MUSL_LIBGCC_SHARED_OBJECT), b"shared libgcc linker name")
            .unwrap();

        wrap_rustc_stage_provider_dynamic_tools(
            &provider_dir,
            &runtime_loader,
            FIRST_STAGE_MUSL_TRIPLE,
            RUSTC_STAGE1_DYNAMIC_TOOL_SPECS,
        )
        .unwrap();
        let rustc_wrapper = fs::read_to_string(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).unwrap();
        let cargo_wrapper = fs::read_to_string(provider_dir.join(PROVIDER_CARGO_RELATIVE_PATH)).unwrap();

        assert!(rustc_wrapper.contains("--sysroot \"$root_dir\""));
        assert!(!cargo_wrapper.contains("--sysroot \"$root_dir\""));
        assert_eq!(fs::read(provider_dir.join("bin/rustc.dynamic")).unwrap(), b"rustc binary");
        assert_eq!(fs::read(provider_dir.join("bin/cargo.dynamic")).unwrap(), b"cargo binary");
        assert_eq!(
            fs::read(provider_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH)).unwrap(),
            b"runtime loader"
        );
        assert_eq!(
            fs::read(
                provider_dir
                    .join(FIRST_STAGE_PROC_MACRO_RUNTIME_DIR)
                    .join(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT)
            )
            .unwrap(),
            b"versioned shared libgcc runtime"
        );
    }

    #[test]
    fn rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let runtime_source_dir = dir.path().join("runtime-source");
        let runtime_loader = runtime_source_dir.join(FIRST_STAGE_TARGET_MUSL_LIBC_SHARED_OBJECT);
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        fs::create_dir_all(&runtime_source_dir).unwrap();
        fs::write(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH), b"rustc binary").unwrap();
        fs::write(&runtime_loader, b"runtime loader").unwrap();
        let specs = [RustProviderDynamicToolSpec {
            relative_path: PROVIDER_RUSTC_RELATIVE_PATH,
            sysroot_mode: RustProviderDynamicToolSysrootMode::Inject,
        }];

        let err =
            wrap_rustc_stage_provider_dynamic_tools(&provider_dir, &runtime_loader, FIRST_STAGE_MUSL_TRIPLE, &specs)
                .unwrap_err();

        assert!(err.to_string().contains("dynamic runtime shared object is missing"));
        assert!(err.to_string().contains(FIRST_STAGE_TARGET_MUSL_LIBGCC_VERSIONED_SHARED_OBJECT));
        assert_eq!(fs::read(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).unwrap(), b"rustc binary");
        assert!(!provider_dir.join("bin/rustc.dynamic").exists());
    }

    #[cfg(unix)]
    #[test]
    fn first_stage_proc_macro_rustc_wrapper_skips_duplicate_sysroot() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let loader_path = provider_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH);
        let explicit_sysroot = dir.path().join("explicit-sysroot");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        fs::create_dir_all(loader_path.parent().unwrap()).unwrap();
        fs::write(&loader_path, b"#!/bin/sh\nfor arg in \"$@\"; do printf '<%s>\\n' \"$arg\"; done\n").unwrap();
        fs::write(
            provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH),
            provider_local_dynamic_mrustc_rustc_wrapper(FIRST_STAGE_MUSL_TRIPLE),
        )
        .unwrap();
        make_executable(&loader_path);
        make_executable(&provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH));

        let implicit_output = Command::new(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH)).output().unwrap();
        let explicit_output = Command::new(provider_dir.join(PROVIDER_RUSTC_RELATIVE_PATH))
            .arg("--sysroot")
            .arg(&explicit_sysroot)
            .output()
            .unwrap();
        let implicit_stdout = String::from_utf8(implicit_output.stdout).unwrap();
        let explicit_stdout = String::from_utf8(explicit_output.stdout).unwrap();
        let explicit_sysroot_line = format!("<{}>", explicit_sysroot.display());
        let provider_sysroot_line = format!("<{}>", provider_dir.display());

        assert!(implicit_output.status.success());
        assert!(explicit_output.status.success());
        assert!(implicit_stdout.contains("<--sysroot>"));
        assert!(implicit_stdout.contains(&provider_sysroot_line));
        assert!(explicit_stdout.contains("<--sysroot>"));
        assert!(explicit_stdout.contains(&explicit_sysroot_line));
        assert!(!explicit_stdout.contains(&provider_sysroot_line));
        assert_eq!(explicit_stdout.lines().filter(|line| *line == "<--sysroot>").count(), 1);
    }

    #[test]
    fn first_stage_proc_macro_rustc_install_rejects_missing_runtime_loader() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let dynamic_rustc = dir.path().join("dynamic-rustc");
        let missing_runtime_loader = dir.path().join("missing-libc.so");
        fs::create_dir_all(provider_dir.join("bin")).unwrap();
        fs::write(&dynamic_rustc, b"dynamic rustc").unwrap();

        let err = install_first_stage_proc_macro_capable_rustc_from_paths(
            &provider_dir,
            &dynamic_rustc,
            &missing_runtime_loader,
            FIRST_STAGE_MUSL_TRIPLE,
        )
        .unwrap_err();

        assert!(err.to_string().contains("proc-macro runtime loader is missing"));
        assert!(!provider_dir.join(FIRST_STAGE_PROC_MACRO_RUNTIME_LOADER_RELATIVE_PATH).exists());
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

    #[test]
    fn smoke_rustc_args_do_not_duplicate_provider_wrapper_sysroot() {
        let provider_dir = PathBuf::from("provider");
        let scratch_dir = PathBuf::from("scratch");
        let plan = RustSourceProviderSmokePlan {
            provider_dir: provider_dir.clone(),
            scratch_dir: scratch_dir.clone(),
            rustc_path: provider_dir.join("bin/rustc"),
            source_path: scratch_dir.join(SMOKE_SOURCE_FILE),
            output_path: scratch_dir.join(SMOKE_OUTPUT_FILE),
            target_triple: TARGET_TRIPLE.to_string(),
        };

        let args = smoke_rustc_args(&plan);

        assert!(!args.iter().any(|arg| arg == "--sysroot"));
        assert!(!args.iter().any(|arg| arg.to_string_lossy().starts_with("--sysroot=")));
        assert!(args.iter().any(|arg| arg == "--target"));
        assert!(args.iter().any(|arg| arg == TARGET_TRIPLE));
        assert!(args.iter().any(|arg| arg == SMOKE_CRATE_NAME));
        assert!(args.iter().any(|arg| arg.to_string_lossy().ends_with(SMOKE_OUTPUT_FILE)));
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

        assert!(
            err.to_string().contains("disallowed Rust provider marker")
                || err.to_string().contains("uses prebuilt Rust")
        );
        assert!(!sentinel_path.exists());
        assert!(!scratch_dir.join(SMOKE_OUTPUT_FILE).exists());
    }

    fn write_test_route_plan(root: &Path) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_sources(root, &sources, &sources.mrustc_sha256_hex);
    }

    fn write_test_musl_host_route_plan(root: &Path) {
        write_test_route_plan(root);
        rewrite_test_route_plan_to_musl_host(root);
    }

    fn rewrite_test_route_plan_to_musl_host(root: &Path) {
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

    fn write_test_source_archives_with_mrustc_run_rustc_makefile(
        root: &Path,
        run_rustc_makefile: &[u8],
    ) -> TestSourceArchives {
        let mut sources = write_test_source_archives(root);
        let mrustc_path = root.join("test-source-archives").join("mrustc-0.12.0.tar.gz");
        write_test_mrustc_source_archive(&mrustc_path, run_rustc_makefile);
        sources.mrustc_url = url::Url::from_file_path(&mrustc_path).unwrap().to_string();
        sources.mrustc_sha256_hex = sha256_file_hex(&mrustc_path);
        sources
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

    fn write_test_mrustc_source_archive(path: &Path, run_rustc_makefile: &[u8]) {
        let top_dir = "mrustc-0.12.0";
        let file = File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        append_test_tar_file(&mut builder, &format!("{top_dir}/README.txt"), b"mrustc seed source\n");
        append_test_tar_file(&mut builder, &format!("{top_dir}/{FIRST_STAGE_MAKEFILE}"), test_mrustc_makefile());
        append_test_tar_file(
            &mut builder,
            &format!("{top_dir}/{FIRST_STAGE_MINICARGO_MAKEFILE}"),
            test_mrustc_minicargo_makefile(),
        );
        append_test_tar_file(
            &mut builder,
            &format!("{top_dir}/{FIRST_STAGE_RUN_RUSTC_DIR}/{FIRST_STAGE_MAKEFILE}"),
            run_rustc_makefile,
        );
        append_test_tar_file(
            &mut builder,
            &format!("{top_dir}/{FIRST_STAGE_MINICARGO_BUILD_SOURCE}"),
            test_mrustc_minicargo_build_source(),
        );
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
        if top_dir == "rust-1.90.0" {
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_RUSTC_DRIVER_MANIFEST_SUFFIX}"),
                test_rustc_driver_manifest(),
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
            append_test_tar_file_with_mode(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_RUN_RUSTC_PROXY_PATH}"),
                test_mrustc_run_rustc_proxy(),
                SOURCE_ARCHIVE_EXECUTABLE_FILE_MODE,
            );
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_MINICARGO_BUILD_SOURCE}"),
                test_mrustc_minicargo_build_source(),
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

    fn test_mrustc_minicargo_build_source() -> &'static [u8] {
        b"void synthetic_minicargo_build_source() {\n    auto out_dir = parent.get_output_dir(m_is_for_host).to_absolute() / parent.get_build_script_out(m_manifest);\n    env.push_back(\"OUT_DIR\", out_dir.str());\n    if( parent.is_rustc() ) {\n        args.push_back(\"-Z\");\n        args.push_back(\"force-unstable-if-unmarked\");\n    }\n}\n"
    }

    fn test_mrustc_minicargo_makefile() -> &'static [u8] {
        concat!(
            "bin/minicargo:\n",
            "\tmkdir -p bin\n",
            "\tprintf '%s\\n' '#!/bin/sh' 'set -eu' 'out=' 'prev=' 'for arg in \"$$@\"; do' '  if [ \"$$prev\" = \"--output-dir\" ]; then out=\"$$arg\"; prev=\"\"; continue; fi' '  if [ \"$$arg\" = \"--output-dir\" ]; then prev=\"--output-dir\"; continue; fi' 'done' 'if [ -z \"$$out\" ]; then echo missing-output-dir >&2; exit 2; fi' 'mkdir -p \"$$out\"' 'printf \"synthetic target std\\\\n\" > \"$$out/libstd.rlib\"' > bin/minicargo\n",
            "\tchmod +x bin/minicargo\n",
            "LLVM_CMAKE_OPTS += LLVM_ENABLE_ZLIB=OFF LLVM_ENABLE_TERMINFO=OFF LLVM_ENABLE_LIBEDIT=OFF WITH_POLLY=OFF\n",
            "$(RUSTCSRC)build/bin/llvm-config: $(RUSTCSRC)build/Makefile\n",
            "\t$Vcd $(RUSTCSRC)build && $(MAKE) -j $(PARLEVEL)\n",
            "RUSTCSRC:\n",
            "\tmkdir -p rustc-$${RUSTC_VERSION}-src/compiler/rustc_driver rustc-$${RUSTC_VERSION}-src/compiler/rustc_session/src\n",
            "\tprintf '%s\\n' '[package]' 'name = \"rustc_driver\"' 'version = \"0.0.0\"' 'edition = \"2024\"' '' '[lib]' 'crate-type = [\"dylib\"]' > rustc-$${RUSTC_VERSION}-src/compiler/rustc_driver/Cargo.toml\n",
            "\tprintf '%s\\n' 'impl Sysroot {' '    pub fn new(explicit: Option<PathBuf>) -> Sysroot {' '        Sysroot { explicit, default: filesearch::default_sysroot() }' '    }' '}' > rustc-$${RUSTC_VERSION}-src/compiler/rustc_session/src/config.rs\n",
            "\ttouch rustc-$${RUSTC_VERSION}-src/dl-version\n",
            "output/rustc:\n",
            "\tmkdir -p $(@D)\n",
            "\tprintf 'synthetic translated rustc\\n' > $@\n",
            "\tchmod +x $@\n",
            "output-%/rustc:\n",
            "\tmkdir -p $(@D)\n",
            "\tprintf 'synthetic translated rustc\\n' > $@\n",
            "\tchmod +x $@\n",
            "output/cargo:\n",
            "\tmkdir -p $(@D)\n",
            "\tprintf 'synthetic translated cargo\\n' > $@\n",
            "\tchmod +x $@\n",
            "output-%/cargo:\n",
            "\tmkdir -p $(@D)\n",
            "\tprintf 'synthetic translated cargo\\n' > $@\n",
            "\tchmod +x $@\n",
        )
        .as_bytes()
    }

    fn test_mrustc_run_rustc_makefile() -> &'static [u8] {
        b"RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))\nCARGO_ENV_STAGE2_STD := CARGO_TARGET_DIR=$(OUTDIR)build-std2 RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)\nCARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_S)rustc) $(CARGO_ENV)\n\nall:\n\tmkdir -p output/prefix/bin output/prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib\n\tprintf '%s\\n' '#!/bin/sh' 'd=$$(dirname $$0)' 'LD_LIBRARY_PATH=/old/build/prefix/lib $$d/rustc_binary \"$$@\"' > output/prefix/bin/rustc\n\tprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$$@\"; do' '  if [ \"$$prev\" = \"-o\" ]; then out=\"$$arg\"; prev=\"\"; continue; fi' '  if [ \"$$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$$out\"' 'echo synthetic rustc smoke' > output/prefix/bin/rustc_binary\n\tprintf 'synthetic prefix cargo\\n' > output/prefix/bin/cargo\n\tprintf 'synthetic prefix host std\\n' > output/prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib\n\tprintf 'synthetic prefix musl host std\\n' > output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib\n\tchmod +x output/prefix/bin/rustc output/prefix/bin/rustc_binary output/prefix/bin/cargo\n\nfinal-smoke:\n\t$V$(DBG) $(BINDIR)rustc -L $(LIBDIR) $< -o $@\n\t./$@\n\n$(BINDIR_S)rustc: ../output$(OUTDIR_SUF)/rustc\n\t@mkdir -p $(dir $@)\n\tcp $< $@\n$(BINDIR_2)rustc: ../output$(OUTDIR_SUF)/rustc\n\t@mkdir -p $(dir $@)\n\tcp $< $@\n$(BINDIR)rustc: $(BINDIR_2)rustc\n\t@mkdir -p $(BINDIR)\n\t$Vprintf '#!/bin/sh\\nd=$$(dirname $$0)\\nLD_LIBRARY_PATH=\"$(abspath $(OUTDIR)prefix/lib):$(abspath $(LIBDIR))\" $$d/rustc_binary \"$$@\"' >$@\n\t$Vchmod +x $@\n\n$(BINDIR)cargo: $(BINDIR)rustc\n\t$VTMPDIR=$(abspath $(PREFIX)tmp) $(CARGO_ENV_RUSTC) $(BINDIR_S)cargo build $(CARGO_FLAGS) --manifest-path $(RUST_SRC_CARGO)Cargo.toml\n\t$Vcp $(CARGO_OUTDIR_RUSTC)cargo $(BINDIR)cargo\n\noutput%/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib:\n\tmkdir -p $(@D)\n\tprintf 'synthetic target std\\n' > $@\n"
    }

    fn test_mrustc_run_rustc_proxy() -> &'static [u8] {
        b"#!/bin/sh\nif [ -n \"${PROXY_RUSTC:-}\" ]; then\n    ${PROXY_RUSTC} \"$@\"\nelse\n    ${PROXY_MRUSTC} \"$@\"\nfi\n"
    }

    fn test_rustc_driver_manifest() -> &'static [u8] {
        b"[package]\nname = \"rustc_driver\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"dylib\"]\n"
    }

    fn test_rustc_stage1_build_script() -> &'static [u8] {
        b"set -eu\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\nmkdir -p \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime\" \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nrm -f \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf '%s\\n' '#!/bin/sh' 'exec \"$@\"' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nchmod +x \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf 'synthetic shared libgcc runtime\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so.1\"\nprintf 'synthetic shared libgcc linker name\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic stage1 rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic stage1 cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic stage1 host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic stage1 target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\n"
    }

    fn test_rustc_final_build_script() -> &'static [u8] {
        b"set -eu\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\nmkdir -p \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime\" \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nrm -f \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf '%s\\n' '#!/bin/sh' 'exec \"$@\"' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nchmod +x \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf 'synthetic shared libgcc runtime\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so.1\"\nprintf 'synthetic shared libgcc linker name\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic final rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic final cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic final rustdoc for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\nprintf 'synthetic final host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic final target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\" \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\n"
    }

    fn test_rustc_xpy_script() -> &'static [u8] {
        b"#!/bin/sh\nset -eu\nprintf '%s\\n' \"synthetic x.py for $MANTLE_RUST_VERSION $*\"\nmkdir -p \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime\" \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nrm -f \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf '%s\\n' '#!/bin/sh' 'exec \"$@\"' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nchmod +x \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libc.so\"\nprintf 'synthetic shared libgcc runtime\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so.1\"\nprintf 'synthetic shared libgcc linker name\\n' > \"$MANTLE_BUILD_DIR/rust-bootstrap-target-linker-runtime/libgcc_s.so\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic xpy rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic xpy cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic xpy rustdoc for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\nprintf 'synthetic xpy host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic xpy target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\" \"$MANTLE_STAGE_OUTPUT/bin/rustdoc\"\n"
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
        write_bytes(root, "bin/rustc", synthetic_smoke_rustc_script(sentinel_path).as_bytes());
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
    fn synthetic_smoke_rustc_script(sentinel_path: Option<&Path>) -> String {
        let sentinel_write =
            sentinel_path.map(|path| format!("printf launched > '{}'\n", path.display())).unwrap_or_default();
        format!(
            "#!/bin/sh\n{sentinel_write}out=''\nprev=''\nfor arg in \"$@\"; do\n  if [ \"$prev\" = '--sysroot' ]; then echo unexpected-sysroot-value >&2; exit {SYNTHETIC_RUSTC_UNEXPECTED_SYSROOT_EXIT_CODE}; fi\n  case \"$arg\" in --sysroot|--sysroot=*) echo unexpected-sysroot >&2; exit {SYNTHETIC_RUSTC_UNEXPECTED_SYSROOT_EXIT_CODE} ;; esac\n  if [ \"$prev\" = '-o' ]; then out=\"$arg\"; prev=''; continue; fi\n  if [ \"$arg\" = '-o' ]; then prev='-o'; continue; fi\ndone\nif [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi\nprintf 'synthetic rlib\\n' > \"$out\"\necho synthetic rustc smoke\n"
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
