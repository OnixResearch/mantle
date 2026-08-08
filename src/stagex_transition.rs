use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_bootstrap_core::LineageManifest;
use crunch_bootstrap_core::STAGEX_PLAN_SCHEMA_V1;
use crunch_bootstrap_core::STAGEX_SEED_SOURCE_STAGE_ID;
use crunch_bootstrap_core::StagexExecutableAuthorization;
use crunch_bootstrap_core::StagexMaterializationPlan;
use crunch_bootstrap_core::StagexStagePlan;
use crunch_bootstrap_core::stagex_plan_digest_blake3;
use crunch_bootstrap_core::validate_stagex_plan;
use serde::Serialize;

use crate::protected_exec::OutputPromotionRecord;
use crate::protected_exec::PlannedExecutable;
use crate::protected_exec::PromotedExecutable;
use crate::protected_exec::ProtectedExecPolicy;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec::blake3_file_hex;
use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;
use crate::protected_exec_seccomp::install_current_thread_exec_supervisor;

pub(crate) const STAGEX_TRANSITION_REPORT_SCHEMA_V1: &str = "mantle-stagex-protected-transition-report-v1";
pub(crate) const HEX0_REPRODUCTION_STAGE_ID: &str = "hex0-reproduction";
pub(crate) const KAEM_MATERIALIZATION_STAGE_ID: &str = "kaem-materialization";
pub(crate) const KAEM_SMOKE_STAGE_ID: &str = "kaem-smoke";
pub(crate) const MES_M2_STAGE_ID: &str = "mes-m2-materialization";
pub(crate) const MES_M2_SMOKE_STAGE_ID: &str = "mes-m2-smoke";
pub(crate) const MES_NYACC_STAGE_ID: &str = "mes-nyacc-regeneration";
pub(crate) const MES_RUNTIME_STAGE_ID: &str = "mes-runtime-materialization";
pub(crate) const TINYCC_MES_STAGE_ID: &str = "tinycc-mes-materialization";
pub(crate) const TINYCC_RUNTIME_STAGE_ID: &str = "tinycc-runtime-refresh";
pub(crate) const TINYCC_BOOT0_STAGE_ID: &str = "tinycc-boot0-materialization";
pub(crate) const TINYCC_BOOT0_SMOKE_STAGE_ID: &str = "tinycc-boot0-smoke";
pub(crate) const TINYCC_FINAL_STAGE_ID: &str = "tinycc-final-materialization";
pub(crate) const TINYCC_FINAL_SMOKE_STAGE_ID: &str = "tinycc-final-smoke";
pub(crate) const TINYCC27_RUNTIME_STAGE_ID: &str = "tinycc27-runtime-refresh";
pub(crate) const TINYCC27_BUILD_STAGE_ID: &str = "tinycc27-materialization";
pub(crate) const TINYCC27_SMOKE_STAGE_ID: &str = "tinycc27-smoke";
pub(crate) const MAKE_RECIPE_RUNNER_STAGE_ID: &str = "make-recipe-runner-materialization";
pub(crate) const MAKE_BUILD_STAGE_ID: &str = "make-materialization";
pub(crate) const MAKE_SMOKE_STAGE_ID: &str = "make-smoke";
pub(crate) const GNU_PATCH_BUILD_STAGE_ID: &str = "gnu-patch-materialization";
pub(crate) const GNU_PATCH_SMOKE_STAGE_ID: &str = "gnu-patch-smoke";
pub(crate) const GZIP_GENERATOR_BUILD_STAGE_ID: &str = "gzip-generator-materialization";
pub(crate) const GZIP_GENERATOR_RUN_STAGE_ID: &str = "gzip-generator-execution";
pub(crate) const GZIP_BUILD_STAGE_ID: &str = "gzip-materialization";
pub(crate) const GZIP_SMOKE_STAGE_ID: &str = "gzip-smoke";
pub(crate) const TAR_BUILD_STAGE_ID: &str = "tar-materialization";
pub(crate) const TAR_SMOKE_STAGE_ID: &str = "tar-smoke";
pub(crate) const SED_BUILD_STAGE_ID: &str = "sed-materialization";
pub(crate) const SED_SMOKE_STAGE_ID: &str = "sed-smoke";
pub(crate) const BZIP2_REWRITE_STAGE_ID: &str = "bzip2-source-rewrite";
pub(crate) const BZIP2_BUILD_STAGE_ID: &str = "bzip2-materialization";
pub(crate) const BZIP2_SMOKE_STAGE_ID: &str = "bzip2-smoke";
pub(crate) const COREUTILS_PATCH_STAGE_ID: &str = "coreutils-source-patch";
pub(crate) const COREUTILS_BUILD_STAGE_ID: &str = "coreutils-materialization";
pub(crate) const COREUTILS_SMOKE_STAGE_ID: &str = "coreutils-smoke";
pub(crate) const OYACC_PATCH_STAGE_ID: &str = "oyacc-source-patch";
pub(crate) const OYACC_BUILD_STAGE_ID: &str = "oyacc-materialization";
pub(crate) const OYACC_SMOKE_STAGE_ID: &str = "oyacc-smoke";
pub(crate) const BASH_BUILD_STAGE_ID: &str = "bash-materialization";
pub(crate) const BASH_SMOKE_STAGE_ID: &str = "bash-smoke";
pub(crate) const TCC_MUSL_PREP_BUILD_STAGE_ID: &str = "tcc-musl-prep-materialization";
pub(crate) const TCC_MUSL_PREP_SMOKE_STAGE_ID: &str = "tcc-musl-prep-smoke";
pub(crate) const MUSL_BUILD_STAGE_ID: &str = "musl-materialization";
pub(crate) const MUSL_SMOKE_STAGE_ID: &str = "musl-smoke";
pub(crate) const TCC_MUSL_BUILD_STAGE_ID: &str = "tcc-musl-materialization";
pub(crate) const TCC_MUSL_SMOKE_STAGE_ID: &str = "tcc-musl-smoke";
pub(crate) const MUSL_PASS2_BUILD_STAGE_ID: &str = "musl-pass2-materialization";
pub(crate) const MUSL_PASS2_SMOKE_STAGE_ID: &str = "musl-pass2-smoke";
pub(crate) const TCC_MUSL_V2_BUILD_STAGE_ID: &str = "tcc-musl-v2-materialization";
pub(crate) const TCC_MUSL_V2_SMOKE_STAGE_ID: &str = "tcc-musl-v2-smoke";
pub(crate) const TCC_MUSL_V2_EXECUTION_STAGE_ID: &str = "tcc-musl-v2-execution";
pub(crate) const TCC_SELFHOST_BUILD_STAGE_ID: &str = "tcc-musl-selfhost-build";
pub(crate) const TCC_SELFHOST_SMOKE_STAGE_ID: &str = "tcc-musl-selfhost-smoke";
pub(crate) const MUSL_NATIVE_BUILD_STAGE_ID: &str = "musl-native-materialization";
pub(crate) const MUSL_NATIVE_SMOKE_STAGE_ID: &str = "musl-native-smoke";
pub(crate) const MUSL_NATIVE_EXECUTION_STAGE_ID: &str = "musl-native-execution";
pub(crate) const M4_BUILD_STAGE_ID: &str = "m4-materialization";
pub(crate) const M4_SMOKE_STAGE_ID: &str = "m4-smoke";
pub(crate) const DIFFUTILS_BUILD_STAGE_ID: &str = "diffutils-materialization";
pub(crate) const DIFFUTILS_SMOKE_STAGE_ID: &str = "diffutils-smoke";
pub(crate) const GREP_BUILD_STAGE_ID: &str = "grep-materialization";
pub(crate) const GREP_SMOKE_STAGE_ID: &str = "grep-smoke";
pub(crate) const GAWK_BUILD_STAGE_ID: &str = "gawk-materialization";
pub(crate) const GAWK_SMOKE_STAGE_ID: &str = "gawk-smoke";
pub(crate) const BISON_BUILD_STAGE_ID: &str = "bison-materialization";
pub(crate) const BISON_SMOKE_STAGE_ID: &str = "bison-smoke";
pub(crate) const FLEX_BUILD_STAGE_ID: &str = "flex-materialization";
pub(crate) const FLEX_SMOKE_STAGE_ID: &str = "flex-smoke";
pub(crate) const BASH_FULL_GENERATOR_BUILD_STAGE_ID: &str = "bash-full-generator-materialization";
pub(crate) const BASH_FULL_GENERATOR_RUN_STAGE_ID: &str = "bash-full-generator-execution";
pub(crate) const BASH_FULL_BUILD_STAGE_ID: &str = "bash-full-materialization";
pub(crate) const BASH_FULL_SMOKE_STAGE_ID: &str = "bash-full-smoke";
pub(crate) const BINUTILS_CANONICALIZER_BUILD_STAGE_ID: &str = "binutils-canonicalizer-materialization";
pub(crate) const BINUTILS_HELPER_BUILD_STAGE_ID: &str = "binutils-helper-materialization";
pub(crate) const BINUTILS_GENERATOR_BUILD_STAGE_ID: &str = "binutils-generator-materialization";
pub(crate) const BINUTILS_COMPONENT_BUILD_STAGE_ID: &str = "binutils-component-materialization";
pub(crate) const BINUTILS_INSTALL_STAGE_ID: &str = "binutils-install-materialization";
pub(crate) const BINUTILS_INSTALL_SMOKE_STAGE_ID: &str = "binutils-install-smoke";
const BINUTILS_EXECUTED_COREUTILS_TOOL_COUNT: usize = 20;
const BINUTILS_MKDIR_EXEC_COUNT_BOUNDS: [u32; 2] = [5_377, 5_425];
const BINUTILS_EXECUTED_COREUTILS_TOOL_NAMES: [&str; BINUTILS_EXECUTED_COREUTILS_TOOL_COUNT] = [
    "cat", "chmod", "cp", "echo", "install", "ln", "ls", "mkdir", "mv", "rm", "rmdir", "sort", "test", "head", "wc",
    "basename", "dirname", "tr", "expr", "touch",
];
const BINUTILS_INSTALL_SMOKE_EXECUTED_TOOL_COUNT: usize = 8;
const BINUTILS_INSTALL_SMOKE_EXECUTED_TOOL_NAMES: [&str; BINUTILS_INSTALL_SMOKE_EXECUTED_TOOL_COUNT] =
    ["as", "ld", "ar", "ranlib", "nm", "objcopy", "objdump", "readelf"];
const TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID: &str = "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc";
const M4_CANONICAL_AUTHORIZATION_ID: &str = "planned:m4-materialization:exec:m4-smoke:m4";
const GREP_RUNNER_CANONICAL_AUTHORIZATION_ID: &str = "planned:grep-materialization:exec:grep-smoke:grep";
const GAWK_CANONICAL_AUTHORIZATION_ID: &str = "planned:gawk-materialization:exec:gawk-smoke:gawk";
const BISON_CANONICAL_AUTHORIZATION_ID: &str = "planned:bison-materialization:exec:bison-smoke:bison";
const FLEX_CANONICAL_AUTHORIZATION_ID: &str = "planned:flex-materialization:exec:flex-smoke:flex";
const FLEX_LIBFL_CONSUMER_CANONICAL_AUTHORIZATION_ID: &str =
    "planned:flex-materialization:exec:flex-smoke:libfl-consumer";
const MAKE_CANONICAL_AUTHORIZATION_ID: &str = "planned:make-materialization:exec:make-smoke:make";
const BASH_FULL_GENERATOR_CANONICAL_AUTHORIZATION_ID: &str =
    "planned:bash-full-generator-materialization:exec:bash-full-generator-execution:mkbuiltins";
const BASH_FULL_CANONICAL_AUTHORIZATION_ID: &str = "planned:bash-full-materialization:exec:bash-full-smoke:bash-full";
pub(crate) const HEX0_SEED_BLAKE3: &str = "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80";
pub(crate) const HEX0_SOURCE_BLAKE3: &str = "0fb23576a10b41df29c165e39514f18c411da96ae71873a6a2e2f0b1d94de614";
pub(crate) const KAEM_SOURCE_BLAKE3: &str = "5a56b4164dca4d1e03ba35bc8ce4b418a0de2baf1cb9e6a0912607e26532d30d";
pub(crate) const EMPTY_KAEM_SMOKE_SOURCE_BLAKE3: &str =
    "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
pub(crate) const KAEM_OUTPUT_BLAKE3: &str = "edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a";

const HEX0_SEED_BYTES: u32 = 229;
const KIB_BYTES: u64 = 1_024;
const MIB_BYTES: u64 = KIB_BYTES * KIB_BYTES;
const AUDIT_SEED_MAX_KIB: u64 = 4;
const AUDIT_SEED_MAX_BYTES: u64 = AUDIT_SEED_MAX_KIB * KIB_BYTES;
const SOURCE_FILE_MAX_KIB: u64 = 256;
const SOURCE_FILE_MAX_BYTES: u64 = SOURCE_FILE_MAX_KIB * KIB_BYTES;
const GENERATED_EXECUTABLE_MAX_MIB: u64 = 4;
const GENERATED_EXECUTABLE_MAX_BYTES: u64 = GENERATED_EXECUTABLE_MAX_MIB * MIB_BYTES;
const LINEAGE_MANIFEST_MAX_MIB: u64 = 1;
const LINEAGE_MANIFEST_MAX_BYTES: u64 = LINEAGE_MANIFEST_MAX_MIB * MIB_BYTES;
const STAGE_TIMEOUT_MS: u64 = 30_000;
const STAGE0_STAGE_TIMEOUT_MS: u64 = 120_000;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const MES_STAGE_TIMEOUT_MINUTES: u64 = 10;
const MES_STAGE_TIMEOUT_MS: u64 = MES_STAGE_TIMEOUT_MINUTES * SECONDS_PER_MINUTE * MILLISECONDS_PER_SECOND;
const MES_OUTPUT_MAX_MIB: u64 = 64;
const MES_OUTPUT_MAX_BYTES: u64 = MES_OUTPUT_MAX_MIB * MIB_BYTES;
const STAGE_JOB_COUNT: u32 = 1;
const TRANSITION_STAGE_COUNT_MAX: u32 = 3;
const TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX: u32 = 97;
const TINYCC_LINK_CHILD_EVENT_COUNT: usize = 5;
const MAKE_RECIPE_CHILD_EVENT_COUNT: usize = 1;
const GZIP_GZIP_SMOKE_EVENT_COUNT: usize = 2;
const PROCESS_POLL_MS: u64 = 10;
const AUDIT_FLUSH_WAIT_MS: u64 = 50;
const GENERATED_LAUNCH_MAX_ATTEMPTS: u32 = 16;
const GENERATED_LAUNCH_RETRY_DELAY_MS: u64 = 20;
const PROCESS_STDERR_MAX_KIB: u64 = 4;
const PROCESS_STDERR_MAX_BYTES: u64 = PROCESS_STDERR_MAX_KIB * KIB_BYTES;
const EXECUTABLE_MODE_OWNER_ONLY: u32 = 0o700;
const ETXTBSY_EXIT_CODE: i32 = 126;
const ETXTBSY_STDERR_MARKER: &str = "Text file busy";
const KAEM_SMOKE_MARKER: &[u8] = b"kaem-smoke-success\n";
const EXPECTED_AUDIT_EVENT_COUNT: usize = 3;
const EXPECTED_PROMOTION_COUNT: usize = 2;
const SOURCE_STATE_DOMAIN: &[u8] = b"mantle-stagex-transition-source-state-v1\0";
const REPORT_FILE_NAME: &str = "transition-report.json";
const PLAN_FILE_NAME: &str = "transition-plan.json";
const AUDIT_FILE_NAME: &str = "protected-exec-audit.json";
const FAILURE_AUDIT_FILE_NAME: &str = "protected-exec-audit-failure.json";

#[derive(Debug, Clone)]
pub(crate) struct StagexTransitionRequest<'a> {
    pub seed_path: &'a Path,
    pub hex0_source_path: &'a Path,
    pub kaem_source_path: &'a Path,
    pub lineage_manifest_path: &'a Path,
    pub source_bundle_path: Option<&'a Path>,
    pub stage0_answers_path: Option<&'a Path>,
    pub scratch_dir: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StagexTransitionReport {
    pub schema_version: String,
    pub status: String,
    pub plan_digest_blake3: String,
    pub source_state_digest_blake3: String,
    pub seed_digest_blake3: String,
    pub reproduced_hex0_digest_blake3: String,
    pub kaem_digest_blake3: String,
    pub kaem_smoke_digest_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage0_full: Option<crate::stagex_stage0_full::Stage0FullInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mes_m2: Option<crate::stagex_mes::MesM2InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mes_runtime: Option<crate::stagex_mes_lib::MesRuntimeInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tinycc_sources: Option<crate::stagex_tinycc::TinyccSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tinycc_runtime: Option<crate::stagex_tinycc::TccMesInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tinycc27_sources: Option<crate::stagex_tinycc27::Tinycc27SourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tinycc27_runtime: Option<crate::stagex_tinycc27::Tinycc27InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make_sources: Option<crate::stagex_make::MakeSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make_runtime: Option<crate::stagex_make::MakeInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gnu_patch_sources: Option<crate::stagex_gnu_patch::GnuPatchSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gnu_patch_runtime: Option<crate::stagex_gnu_patch::GnuPatchInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gzip_sources: Option<crate::stagex_gzip::GzipSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gzip_runtime: Option<crate::stagex_gzip::GzipInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tar_sources: Option<crate::stagex_tar::TarSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tar_runtime: Option<crate::stagex_tar::TarInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sed_sources: Option<crate::stagex_sed::SedSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sed_runtime: Option<crate::stagex_sed::SedInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bzip2_sources: Option<crate::stagex_bzip2::Bzip2SourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bzip2_runtime: Option<crate::stagex_bzip2::Bzip2InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreutils_sources: Option<crate::stagex_coreutils::CoreutilsSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coreutils_runtime: Option<crate::stagex_coreutils::CoreutilsInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oyacc_sources: Option<crate::stagex_oyacc::OyaccSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oyacc_runtime: Option<crate::stagex_oyacc::OyaccInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash_sources: Option<crate::stagex_bash::BashSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash_runtime: Option<crate::stagex_bash::BashInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcc_musl_prep_runtime: Option<crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musl_sources: Option<crate::stagex_musl::MuslSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musl_runtime: Option<crate::stagex_musl::MuslInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcc_musl_runtime: Option<crate::stagex_tcc_musl::TccMuslInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musl_pass2_runtime: Option<crate::stagex_musl::MuslInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcc_musl_v2_runtime: Option<crate::stagex_tcc_musl_v2::InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcc_selfhost_runtime: Option<crate::stagex_tcc_selfhost::InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musl_native_runtime: Option<crate::stagex_musl_native::InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m4_sources: Option<crate::stagex_m4::M4SourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m4_runtime: Option<crate::stagex_m4::M4InventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffutils_sources: Option<crate::stagex_diffutils::DiffutilsSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffutils_runtime: Option<crate::stagex_diffutils::DiffutilsInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep_sources: Option<crate::stagex_grep::GrepSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep_runtime: Option<crate::stagex_grep::GrepInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gawk_sources: Option<crate::stagex_gawk::GawkSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gawk_runtime: Option<crate::stagex_gawk::GawkInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bison_sources: Option<crate::stagex_bison::BisonSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bison_runtime: Option<crate::stagex_bison::BisonInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_sources: Option<crate::stagex_flex::FlexSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_runtime: Option<crate::stagex_flex::FlexInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash_full_runtime: Option<crate::stagex_bash_full::BashFullInventoryReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binutils_sources: Option<crate::stagex_binutils::BinutilsSourceMaterializationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binutils_runtime: Option<crate::stagex_binutils::BinutilsInventoryReport>,
    pub fallback_events: Vec<String>,
    pub promotions: Vec<OutputPromotionRecord>,
    pub protected_exec_events: Vec<ProtectedSeccompAuditEvent>,
}

#[derive(Debug)]
pub(crate) enum StagexTransitionError {
    Io {
        action: String,
        source: io::Error,
    },
    InvalidInput(String),
    DigestMismatch {
        role: String,
        expected: String,
        actual: String,
    },
    SizeLimit {
        role: String,
        actual_bytes: u64,
        limit_bytes: u64,
    },
    Plan(Vec<String>),
    ProtectedExec(String),
    ProcessSpawn {
        executable: PathBuf,
        source: io::Error,
    },
    ProcessFailure {
        executable: PathBuf,
        exit_code: Option<i32>,
        stderr: String,
    },
    ProcessTimeout {
        executable: PathBuf,
        timeout_ms: u64,
    },
    Audit(String),
}

impl std::fmt::Display for StagexTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { action, source } => write!(f, "{action}: {source}"),
            Self::InvalidInput(message) => write!(f, "invalid StageX transition input: {message}"),
            Self::DigestMismatch { role, expected, actual } => {
                write!(f, "StageX {role} BLAKE3 mismatch: expected {expected}, got {actual}")
            }
            Self::SizeLimit {
                role,
                actual_bytes,
                limit_bytes,
            } => write!(f, "StageX {role} has {actual_bytes} bytes; limit is {limit_bytes}"),
            Self::Plan(errors) => write!(f, "invalid StageX transition plan: {}", errors.join("; ")),
            Self::ProtectedExec(message) => write!(f, "StageX protected execution failed: {message}"),
            Self::ProcessSpawn { executable, source } => {
                write!(f, "spawning StageX executable {}: {source}", executable.display())
            }
            Self::ProcessFailure {
                executable,
                exit_code,
                stderr,
            } => write!(f, "StageX executable {} failed with status {:?}: {}", executable.display(), exit_code, stderr),
            Self::ProcessTimeout { executable, timeout_ms } => {
                write!(f, "StageX executable {} exceeded {timeout_ms} ms", executable.display())
            }
            Self::Audit(message) => write!(f, "invalid StageX protected-exec audit: {message}"),
        }
    }
}

impl std::error::Error for StagexTransitionError {}

#[derive(Debug)]
struct TransitionManifestAuthority {
    digest_blake3: String,
    source_bundle_manifest_blake3: String,
}

struct StagedTransitionPaths {
    seed: PathBuf,
    hex0_source: PathBuf,
    kaem_source: PathBuf,
    reproduced_hex0: PathBuf,
    kaem: PathBuf,
    kaem_smoke_script: PathBuf,
    kaem_smoke: PathBuf,
}

struct BinutilsStagePaths {
    runtime: PathBuf,
    bash: PathBuf,
    tcc: PathBuf,
    musl_root: PathBuf,
    coreutils_bin: PathBuf,
    sed: PathBuf,
    grep: PathBuf,
    diff: PathBuf,
    cmp: PathBuf,
    gawk: PathBuf,
    m4: PathBuf,
    bison: PathBuf,
    flex: PathBuf,
    make: PathBuf,
}

impl BinutilsStagePaths {
    fn from_transition_root(root: &Path) -> Self {
        let path = |relative: &str| root.join(relative);
        let paths = Self {
            runtime: path("binutils-stage/runtime"),
            bash: path("bash-full-stage/runtime/output/bin/bash-full"),
            tcc: path("tcc-musl-v2-stage/runtime/output/bin/tcc-0.9.27-musl-v2"),
            musl_root: path("musl-native-stage/runtime/output"),
            coreutils_bin: path("coreutils-stage/runtime/output/bin"),
            sed: path("sed-stage/runtime/output/bin/sed"),
            grep: path("grep-stage/runtime/output/bin/grep"),
            diff: path("diffutils-stage/runtime/output/bin/diff"),
            cmp: path("diffutils-stage/runtime/output/bin/cmp"),
            gawk: path("gawk-stage/runtime/output/bin/gawk"),
            m4: path("m4-stage/runtime/output/bin/m4"),
            bison: path("bison-stage/runtime/output/bin/bison"),
            flex: path("flex-stage/runtime/output/bin/flex"),
            make: path("make-stage/runtime/output/bin/make"),
        };
        assert!(paths.runtime.is_absolute());
        assert!(paths.make.is_absolute());
        paths
    }

    fn probe_request<'a>(&'a self, source_root: &'a Path) -> crate::stagex_binutils::BinutilsConfigureProbeRequest<'a> {
        crate::stagex_binutils::BinutilsConfigureProbeRequest {
            source_root,
            scratch_dir: &self.runtime,
            bash: &self.bash,
            tcc: &self.tcc,
            musl_root: &self.musl_root,
            coreutils_bin: &self.coreutils_bin,
            sed: &self.sed,
            grep: &self.grep,
            diff: &self.diff,
            cmp: &self.cmp,
            gawk: &self.gawk,
            m4: &self.m4,
            bison: &self.bison,
            flex: &self.flex,
            make: &self.make,
        }
    }
}

struct BoundedCommandRequest<'a> {
    executable: &'a Path,
    args: &'a [PathBuf],
    current_dir: &'a Path,
    stderr_path: &'a Path,
    retry_generated_etxtbsy: bool,
}

struct ExpectedAuditEvent<'a> {
    executable_path: &'a Path,
    digest_blake3: &'a str,
    inventory_entry_id: String,
}

pub(crate) fn materialize_protected_transition(
    request: StagexTransitionRequest<'_>,
) -> Result<StagexTransitionReport, StagexTransitionError> {
    let manifest_authority = load_transition_manifest_authority(request.lineage_manifest_path)?;
    let staged = stage_transition_inputs(&request)?;
    validate_stage0_optional_inputs(request.source_bundle_path, request.stage0_answers_path)?;
    validate_optional_source_bundle(request.source_bundle_path, &manifest_authority)?;
    let source_state_digest_blake3 = source_state_digest(
        &staged,
        request.source_bundle_path.map(|_| manifest_authority.source_bundle_manifest_blake3.as_str()),
    )?;
    let plan = build_transition_plan(
        &manifest_authority,
        &staged,
        &source_state_digest_blake3,
        request.source_bundle_path.is_some(),
    )?;
    write_json_create_new(&request.scratch_dir.join(PLAN_FILE_NAME), &plan)?;

    let promotion_stage_ids = promotion_stage_ids(&plan);
    let stage0_root = request.scratch_dir.join("stage0-mini");
    let planned_executables = if request.source_bundle_path.is_some() {
        let mut planned = crate::stagex_stage0::planned_stage0_executables(&stage0_root)
            .into_iter()
            .map(|entry| PlannedExecutable {
                authorization_id: entry.authorization_id,
                source_stage_id: entry.source_stage_id,
                path: entry.path,
                digest_hex: entry.digest_blake3,
            })
            .collect::<Vec<_>>();
        planned.extend(crate::stagex_stage0_full::planned_stage0_full_executables(&stage0_root).into_iter().map(
            |entry| PlannedExecutable {
                authorization_id: entry.authorization_id,
                source_stage_id: entry.source_stage_id,
                path: entry.path,
                digest_hex: entry.digest_blake3,
            },
        ));
        planned.extend(additional_mes_stage0_planned_executables(&stage0_root));
        planned.push(PlannedExecutable {
            authorization_id: "exec:mes-m2-smoke:mes-m2".to_string(),
            source_stage_id: MES_M2_STAGE_ID.to_string(),
            path: request.scratch_dir.join("mes-stage/mes-0.27.1/bin/mes-m2"),
            digest_hex: crate::stagex_mes::MES_M2_BLAKE3.to_string(),
        });
        let tinycc_root = request.scratch_dir.join("tinycc-stage/runtime");
        planned.extend([
            PlannedExecutable {
                authorization_id: "exec:tinycc-runtime-refresh:tcc-mes".to_string(),
                source_stage_id: TINYCC_MES_STAGE_ID.to_string(),
                path: tinycc_root.join("tinycc-0.9.26/tcc-mes"),
                digest_hex: crate::stagex_tinycc::TCC_MES_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:tinycc-boot0-smoke:tcc-boot0".to_string(),
                source_stage_id: TINYCC_BOOT0_STAGE_ID.to_string(),
                path: tinycc_root.join("tinycc-0.9.26/tcc-boot0"),
                digest_hex: crate::stagex_tinycc::TCC_BOOT0_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:tinycc-final-smoke:tinycc".to_string(),
                source_stage_id: TINYCC_FINAL_STAGE_ID.to_string(),
                path: tinycc_root.join("output/bin/tcc"),
                digest_hex: crate::stagex_tinycc::TINYCC_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:tinycc27-smoke:tinycc27".to_string(),
                source_stage_id: TINYCC27_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("tinycc27-stage/runtime/output/bin/tcc"),
                digest_hex: crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:make-smoke:recipe-runner".to_string(),
                source_stage_id: MAKE_RECIPE_RUNNER_STAGE_ID.to_string(),
                path: request.scratch_dir.join("make-stage/runtime/output/bin/make-recipe-runner"),
                digest_hex: crate::stagex_make::MAKE_RECIPE_RUNNER_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:make-smoke:make".to_string(),
                source_stage_id: MAKE_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("make-stage/runtime/output/bin/make"),
                digest_hex: crate::stagex_make::MAKE_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:gnu-patch-smoke:patch".to_string(),
                source_stage_id: GNU_PATCH_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("gnu-patch-stage/runtime/output/bin/patch"),
                digest_hex: crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:gzip-generator-execution:makecrc".to_string(),
                source_stage_id: GZIP_GENERATOR_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("gzip-stage/runtime/generated/bin/makecrc"),
                digest_hex: crate::stagex_gzip::GZIP_MAKECRC_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:gzip-smoke:gzip".to_string(),
                source_stage_id: GZIP_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("gzip-stage/runtime/output/bin/gzip"),
                digest_hex: crate::stagex_gzip::GZIP_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:gzip-smoke:gunzip".to_string(),
                source_stage_id: GZIP_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("gzip-stage/runtime/output/bin/gunzip"),
                digest_hex: crate::stagex_gzip::GZIP_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:tar-smoke:tar".to_string(),
                source_stage_id: TAR_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("tar-stage/runtime/output/bin/tar"),
                digest_hex: crate::stagex_tar::TAR_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:sed-smoke:sed".to_string(),
                source_stage_id: SED_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("sed-stage/runtime/output/bin/sed"),
                digest_hex: crate::stagex_sed::SED_FINAL_BLAKE3.to_string(),
            },
            PlannedExecutable {
                authorization_id: "exec:bzip2-smoke:bzip2".to_string(),
                source_stage_id: BZIP2_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join("bzip2-stage/runtime/output/bin/bzip2"),
                digest_hex: crate::stagex_bzip2::BZIP2_FINAL_BLAKE3.to_string(),
            },
        ]);
        for (name, digest) in coreutils_smoke_executables() {
            planned.push(PlannedExecutable {
                authorization_id: format!("exec:coreutils-smoke:{name}"),
                source_stage_id: COREUTILS_BUILD_STAGE_ID.to_string(),
                path: request.scratch_dir.join(format!("coreutils-stage/runtime/output/bin/{name}")),
                digest_hex: digest.to_string(),
            });
        }
        planned.push(PlannedExecutable {
            authorization_id: "exec:oyacc-smoke:yacc".to_string(),
            source_stage_id: OYACC_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("oyacc-stage/runtime/output/bin/yacc"),
            digest_hex: crate::stagex_oyacc::OYACC_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:bash-smoke:bash".to_string(),
            source_stage_id: BASH_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("bash-stage/runtime/output/bin/bash"),
            digest_hex: crate::stagex_bash::BASH_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-prep-smoke:tcc".to_string(),
            source_stage_id: TCC_MUSL_PREP_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tcc-musl-prep-stage/runtime/output/bin/tcc-musl-prep"),
            digest_hex: crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-smoke:tcc".to_string(),
            source_stage_id: TCC_MUSL_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tcc-musl-stage/runtime/output/bin/tcc-0.9.27-musl"),
            digest_hex: crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-v2-materialization:tinycc26".to_string(),
            source_stage_id: TINYCC_FINAL_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tinycc-stage/runtime/output/bin/tcc-0.9.26"),
            digest_hex: crate::stagex_tinycc::TINYCC_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-v2-smoke:tcc".to_string(),
            source_stage_id: TCC_MUSL_V2_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tcc-musl-v2-stage/runtime/output/bin/tcc-0.9.27-musl-v2"),
            digest_hex: crate::stagex_tcc_musl_v2::COMPILER_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-v2-execution:positive".to_string(),
            source_stage_id: TCC_MUSL_V2_SMOKE_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tcc-musl-v2-stage/runtime/smoke/variadic-positive"),
            digest_hex: crate::stagex_tcc_musl_v2::POSITIVE_BINARY_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:tcc-musl-selfhost-smoke:tcc".to_string(),
            source_stage_id: TCC_SELFHOST_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("tcc-musl-selfhost-stage/runtime/output/bin/tcc-0.9.27-musl-selfhost"),
            digest_hex: crate::stagex_tcc_selfhost::COMPILER_BLAKE3.to_string(),
        });
        let native_smoke_digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
            .iter()
            .find(|output| output.artifact_id == "musl-native-smoke-binary")
            .expect("native musl smoke output has a fixed identity")
            .digest_blake3;
        planned.push(PlannedExecutable {
            authorization_id: "exec:musl-native-execution:binary".to_string(),
            source_stage_id: MUSL_NATIVE_SMOKE_STAGE_ID.to_string(),
            path: request.scratch_dir.join("musl-native-stage/runtime/smoke/native-musl-smoke"),
            digest_hex: native_smoke_digest.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:m4-smoke:m4".to_string(),
            source_stage_id: M4_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("m4-stage/runtime/output/bin/m4"),
            digest_hex: crate::stagex_m4::M4_FINAL_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:diffutils-smoke:diff".to_string(),
            source_stage_id: DIFFUTILS_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("diffutils-stage/runtime/output/bin/diff"),
            digest_hex: crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:diffutils-smoke:cmp".to_string(),
            source_stage_id: DIFFUTILS_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("diffutils-stage/runtime/output/bin/cmp"),
            digest_hex: crate::stagex_diffutils::DIFFUTILS_CMP_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:grep-smoke:grep".to_string(),
            source_stage_id: GREP_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("grep-stage/runtime/output/bin/grep"),
            digest_hex: crate::stagex_grep::GREP_RUNNER_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:gawk-smoke:gawk".to_string(),
            source_stage_id: GAWK_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("gawk-stage/runtime/output/bin/gawk"),
            digest_hex: crate::stagex_gawk::GAWK_BINARY_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:bison-smoke:bison".to_string(),
            source_stage_id: BISON_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("bison-stage/runtime/output/bin/bison"),
            digest_hex: crate::stagex_bison::BISON_BINARY_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:flex-smoke:flex".to_string(),
            source_stage_id: FLEX_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("flex-stage/runtime/output/bin/flex"),
            digest_hex: crate::stagex_flex::FLEX_BINARY_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:flex-smoke:libfl-consumer".to_string(),
            source_stage_id: FLEX_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("flex-stage/runtime/libfl-consumer"),
            digest_hex: crate::stagex_flex::FLEX_LIBFL_CONSUMER_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:bash-full-generator-execution:mkbuiltins".to_string(),
            source_stage_id: BASH_FULL_GENERATOR_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("bash-full-stage/runtime/bash-2.05b-full/builtins/stagex-full-mkbuiltins"),
            digest_hex: crate::stagex_bash_full::BASH_FULL_GENERATOR_BLAKE3.to_string(),
        });
        planned.push(PlannedExecutable {
            authorization_id: "exec:bash-full-smoke:bash-full".to_string(),
            source_stage_id: BASH_FULL_BUILD_STAGE_ID.to_string(),
            path: request.scratch_dir.join("bash-full-stage/runtime/output/bin/bash-full"),
            digest_hex: crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3.to_string(),
        });
        planned.extend(planned_binutils_executables(&request.scratch_dir.join("binutils-stage/runtime")));
        let binutils_predecessors = additional_planned_binutils_predecessor_executables(request.scratch_dir, &planned);
        planned.extend(binutils_predecessors);
        planned
    } else {
        Vec::new()
    };
    let policy = ProtectedExecPolicy::from_stagex_plan(
        staged.seed.clone(),
        HEX0_SEED_BLAKE3.to_string(),
        &promotion_stage_ids,
        &planned_executables,
    )
    .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let supervisor = install_current_thread_exec_supervisor(policy)
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;

    run_hex0_reproduction(&staged)?;
    let hex0_promotion =
        promote_executable(&supervisor, HEX0_REPRODUCTION_STAGE_ID, &staged.reproduced_hex0, HEX0_SEED_BLAKE3)?;
    run_kaem_materialization(&staged)?;
    let kaem_promotion =
        promote_executable(&supervisor, KAEM_MATERIALIZATION_STAGE_ID, &staged.kaem, KAEM_OUTPUT_BLAKE3)?;
    run_kaem_smoke(&staged)?;
    let stage0_full = match (request.source_bundle_path, request.stage0_answers_path) {
        (Some(source_bundle_path), Some(answers_path)) => {
            let mini = crate::stagex_stage0::derive_protected_stage0_mini_inventory(
                crate::stagex_stage0::Stage0MiniInventoryRequest {
                    source_bundle_path,
                    expected_source_bundle_blake3: &manifest_authority.source_bundle_manifest_blake3,
                    reproduced_hex0_path: &staged.reproduced_hex0,
                    kaem_path: &staged.kaem,
                    scratch_dir: &stage0_root,
                },
                &supervisor,
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            Some(
                crate::stagex_stage0_full::derive_protected_stage0_full_inventory(mini, answers_path, &supervisor)
                    .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?,
            )
        }
        (None, None) => None,
        _ => unreachable!("Stage0 optional inputs validated before execution"),
    };
    let mes_m2 = match (request.source_bundle_path, stage0_full.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let result = crate::stagex_mes::derive_mes_m2_inventory(crate::stagex_mes::MesM2InventoryRequest {
                source_bundle_path,
                expected_source_bundle_blake3: &manifest_authority.source_bundle_manifest_blake3,
                full_stage0_root: &stage0_root,
                scratch_dir: &request.scratch_dir.join("mes-stage"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("full Stage0 and Mes execution must advance together"),
    };

    let mes_runtime = match (stage0_full.as_ref(), mes_m2.as_ref()) {
        (Some(_), Some(_)) => {
            let mes_stage_root = request.scratch_dir.join("mes-stage");
            let result = crate::stagex_mes_lib::derive_mes_runtime_inventory(
                crate::stagex_mes_lib::MesRuntimeInventoryRequest {
                    mes_source_root: &mes_stage_root.join("mes-0.27.1"),
                    nyacc_source_root: &mes_stage_root.join("nyacc"),
                    stage0_root: &stage0_root,
                    output_root: &mes_stage_root.join("mes-runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("Mes runtime requires full Stage0 and mes-m2 reports together"),
    };

    let (tinycc_sources, tinycc_runtime) = match (request.source_bundle_path, mes_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let tinycc_stage = request.scratch_dir.join("tinycc-stage");
            fs::create_dir(&tinycc_stage)
                .map_err(|source| io_error("creating create-new protected TinyCC stage root", source))?;
            let sources = crate::stagex_tinycc::materialize_authenticated_tinycc_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &tinycc_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_tinycc::derive_tcc_mes_inventory(crate::stagex_tinycc::TccMesInventoryRequest {
                tinycc_source_root: &sources.output_path,
                mes_source_root: &sources.mes_output_path,
                mes_runtime_root: &request.scratch_dir.join("mes-stage/mes-runtime"),
                mes_m2_path: &request.scratch_dir.join("mes-stage/mes-0.27.1/bin/mes-m2"),
                stage0_root: &stage0_root,
                scratch_dir: &tinycc_stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("TinyCC runtime requires source bundle and Mes runtime together"),
    };

    let (tinycc27_sources, tinycc27_runtime) =
        match (request.source_bundle_path, tinycc_sources.as_ref(), tinycc_runtime.as_ref()) {
            (Some(source_bundle_path), Some(tinycc_sources), Some(_)) => {
                let tinycc27_stage = request.scratch_dir.join("tinycc27-stage");
                fs::create_dir(&tinycc27_stage)
                    .map_err(|source| io_error("creating create-new protected TinyCC 0.9.27 stage root", source))?;
                let sources = crate::stagex_tinycc27::materialize_authenticated_tinycc27_source(
                    source_bundle_path,
                    &manifest_authority.source_bundle_manifest_blake3,
                    &tinycc27_stage.join("sources"),
                )
                .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
                let result = crate::stagex_tinycc27::derive_tinycc27_inventory(
                    crate::stagex_tinycc27::Tinycc27InventoryRequest {
                        source_root: &sources.output_path,
                        mes_source_root: &tinycc_sources.mes_output_path,
                        tinycc26_root: &request.scratch_dir.join("tinycc-stage/runtime/output"),
                        scratch_dir: &tinycc27_stage.join("runtime"),
                        protected_exec_enforced: true,
                    },
                );
                match result {
                    Ok(report) => (Some(sources), Some(report)),
                    Err(error) => {
                        thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                        let events = supervisor.audit_events();
                        write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                        return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                    }
                }
            }
            (None, None, None) => (None, None),
            _ => unreachable!("TinyCC 0.9.27 requires source bundle and TinyCC 0.9.26 together"),
        };

    let (make_sources, make_runtime) = match (request.source_bundle_path, tinycc27_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let make_stage = request.scratch_dir.join("make-stage");
            fs::create_dir(&make_stage)
                .map_err(|source| io_error("creating create-new protected GNU Make stage root", source))?;
            let sources = crate::stagex_make::materialize_authenticated_make_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &make_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_make::derive_make_inventory(crate::stagex_make::MakeInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                scratch_dir: &make_stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU Make requires source bundle and TinyCC 0.9.27 together"),
    };

    let (gnu_patch_sources, gnu_patch_runtime) = match (request.source_bundle_path, make_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let patch_stage = request.scratch_dir.join("gnu-patch-stage");
            fs::create_dir(&patch_stage)
                .map_err(|source| io_error("creating create-new protected GNU patch stage root", source))?;
            let sources = crate::stagex_gnu_patch::materialize_authenticated_gnu_patch_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &patch_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_gnu_patch::derive_gnu_patch_inventory(
                crate::stagex_gnu_patch::GnuPatchInventoryRequest {
                    source_root: &sources.output_path,
                    tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                    scratch_dir: &patch_stage.join("runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU patch requires source bundle and GNU Make together"),
    };

    let (gzip_sources, gzip_runtime) = match (request.source_bundle_path, gnu_patch_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let gzip_stage = request.scratch_dir.join("gzip-stage");
            fs::create_dir(&gzip_stage)
                .map_err(|source| io_error("creating create-new protected gzip stage root", source))?;
            let sources = crate::stagex_gzip::materialize_authenticated_gzip_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &gzip_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_gzip::derive_gzip_inventory(crate::stagex_gzip::GzipInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                scratch_dir: &gzip_stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("gzip requires source bundle and GNU patch together"),
    };

    let (tar_sources, tar_runtime) = match (request.source_bundle_path, gzip_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let tar_stage = request.scratch_dir.join("tar-stage");
            fs::create_dir(&tar_stage)
                .map_err(|source| io_error("creating create-new protected GNU tar stage root", source))?;
            let sources = crate::stagex_tar::materialize_authenticated_tar_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &tar_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_tar::derive_tar_inventory(crate::stagex_tar::TarInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                scratch_dir: &tar_stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU tar requires source bundle and gzip together"),
    };

    let (sed_sources, sed_runtime) = match (request.source_bundle_path, tar_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let sed_stage = request.scratch_dir.join("sed-stage");
            fs::create_dir(&sed_stage)
                .map_err(|source| io_error("creating create-new protected GNU sed stage root", source))?;
            let sources = crate::stagex_sed::materialize_authenticated_sed_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &sed_stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_sed::derive_sed_inventory(crate::stagex_sed::SedInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                scratch_dir: &sed_stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU sed requires source bundle and GNU tar together"),
    };

    let (bzip2_sources, bzip2_runtime) = match (request.source_bundle_path, sed_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("bzip2-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected bzip2 stage root", source))?;
            let sources = crate::stagex_bzip2::materialize_authenticated_bzip2_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_bzip2::derive_bzip2_inventory(crate::stagex_bzip2::Bzip2InventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                sed_root: &request.scratch_dir.join("sed-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("bzip2 requires source bundle and GNU sed together"),
    };

    let (coreutils_sources, coreutils_runtime) = match (request.source_bundle_path, bzip2_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("coreutils-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected coreutils stage root", source))?;
            let sources = crate::stagex_coreutils::materialize_authenticated_coreutils_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_coreutils::derive_coreutils_inventory(
                crate::stagex_coreutils::CoreutilsInventoryRequest {
                    source_root: &sources.output_path,
                    tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                    patch_root: &request.scratch_dir.join("gnu-patch-stage/runtime/output"),
                    scratch_dir: &stage.join("runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("coreutils requires source bundle and bzip2 together"),
    };

    let (oyacc_sources, oyacc_runtime) = match (request.source_bundle_path, coreutils_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("oyacc-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected oyacc stage root", source))?;
            let sources = crate::stagex_oyacc::materialize_authenticated_oyacc_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_oyacc::derive_oyacc_inventory(crate::stagex_oyacc::OyaccInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                patch_root: &request.scratch_dir.join("gnu-patch-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("oyacc requires source bundle and coreutils together"),
    };

    let (bash_sources, bash_runtime) = match (request.source_bundle_path, oyacc_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("bash-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected bash stage root", source))?;
            let sources = crate::stagex_bash::materialize_authenticated_bash_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_bash::derive_bash_inventory(crate::stagex_bash::BashInventoryRequest {
                source_root: &sources.output_path,
                tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("bash requires source bundle and oyacc together"),
    };

    let tcc_musl_prep_runtime = match (tinycc27_sources.as_ref(), bash_runtime.as_ref()) {
        (Some(tinycc27_sources), Some(_)) => {
            let stage = request.scratch_dir.join("tcc-musl-prep-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected TinyCC musl-prep stage root", source))?;
            let result = crate::stagex_tcc_musl_prep::derive_tcc_musl_prep_inventory(
                crate::stagex_tcc_musl_prep::TccMuslPrepInventoryRequest {
                    tinycc27_source_root: &tinycc27_sources.output_path,
                    tinycc27_root: &request.scratch_dir.join("tinycc27-stage/runtime/output"),
                    scratch_dir: &stage.join("runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("TinyCC musl-prep requires TinyCC 0.9.27 sources and bash together"),
    };

    let (musl_sources, musl_runtime) = match (request.source_bundle_path, tcc_musl_prep_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("musl-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected musl stage root", source))?;
            let sources = crate::stagex_musl::materialize_authenticated_musl_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_musl::derive_musl_inventory(crate::stagex_musl::MuslInventoryRequest {
                source_root: &sources.output_path,
                compiler_root: &request.scratch_dir.join("tcc-musl-prep-stage/runtime/output"),
                profile: crate::stagex_musl::MuslProfile::First,
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("musl requires source bundle and TinyCC musl-prep together"),
    };

    let tcc_musl_runtime = match (tinycc27_sources.as_ref(), musl_runtime.as_ref()) {
        (Some(tinycc27_sources), Some(_)) => {
            let stage = request.scratch_dir.join("tcc-musl-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected musl-linked TinyCC stage root", source))?;
            let result =
                crate::stagex_tcc_musl::derive_tcc_musl_inventory(crate::stagex_tcc_musl::TccMuslInventoryRequest {
                    tinycc27_source_root: &tinycc27_sources.output_path,
                    tcc_musl_prep_root: &request.scratch_dir.join("tcc-musl-prep-stage/runtime/output"),
                    musl_root: &request.scratch_dir.join("musl-stage/runtime/output"),
                    scratch_dir: &stage.join("runtime"),
                    protected_exec_enforced: true,
                });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("musl-linked TinyCC requires TinyCC sources and musl together"),
    };

    let musl_pass2_runtime = match (musl_sources.as_ref(), tcc_musl_runtime.as_ref()) {
        (Some(sources), Some(_)) => {
            let stage = request.scratch_dir.join("musl-pass2-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected second-musl stage root", source))?;
            let result = crate::stagex_musl::derive_musl_inventory(crate::stagex_musl::MuslInventoryRequest {
                source_root: &sources.output_path,
                compiler_root: &request.scratch_dir.join("tcc-musl-stage/runtime/output"),
                profile: crate::stagex_musl::MuslProfile::Second,
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("second-musl requires authenticated musl source and musl-linked TinyCC"),
    };

    let tcc_musl_v2_runtime = match (tinycc27_sources.as_ref(), tinycc_runtime.as_ref(), musl_pass2_runtime.as_ref()) {
        (Some(tinycc27_sources), Some(_), Some(_)) => {
            let stage = request.scratch_dir.join("tcc-musl-v2-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected TinyCC musl-v2 stage root", source))?;
            let result = crate::stagex_tcc_musl_v2::derive_inventory(crate::stagex_tcc_musl_v2::InventoryRequest {
                tinycc27_source_root: &tinycc27_sources.output_path,
                tinycc26_root: &request.scratch_dir.join("tinycc-stage/runtime/output"),
                tcc_musl_prep_root: &request.scratch_dir.join("tcc-musl-prep-stage/runtime/output"),
                musl_pass2_root: &request.scratch_dir.join("musl-pass2-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None, None) => None,
        _ => unreachable!("TinyCC musl-v2 requires source, TinyCC 0.9.26, and second-musl"),
    };

    let tcc_selfhost_runtime = match tcc_musl_v2_runtime.as_ref() {
        Some(_) => {
            let stage = request.scratch_dir.join("tcc-musl-selfhost-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected TinyCC self-host stage root", source))?;
            let result = crate::stagex_tcc_selfhost::derive_inventory(crate::stagex_tcc_selfhost::InventoryRequest {
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                tinycc26_root: &request.scratch_dir.join("tinycc-stage/runtime/output"),
                musl_pass2_root: &request.scratch_dir.join("musl-pass2-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        None => None,
    };

    let musl_native_runtime = match (musl_sources.as_ref(), tcc_selfhost_runtime.as_ref()) {
        (Some(sources), Some(_)) => {
            let stage = request.scratch_dir.join("musl-native-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected native musl stage root", source))?;
            let result = crate::stagex_musl_native::derive_inventory(crate::stagex_musl_native::InventoryRequest {
                source_root: &sources.output_path,
                selfhost_root: &request.scratch_dir.join("tcc-musl-selfhost-stage/runtime/output"),
                predecessor_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("native musl requires authenticated source and self-hosted TinyCC"),
    };

    let (m4_sources, m4_runtime) = match (request.source_bundle_path, musl_native_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("m4-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU M4 stage root", source))?;
            let sources = crate::stagex_m4::materialize_authenticated_m4_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_m4::derive_m4_inventory(crate::stagex_m4::M4InventoryRequest {
                source_root: &sources.output_path,
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU M4 requires authenticated source and native musl"),
    };

    let (diffutils_sources, diffutils_runtime) = match (request.source_bundle_path, m4_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("diffutils-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU diffutils stage root", source))?;
            let sources = crate::stagex_diffutils::materialize_authenticated_diffutils_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_diffutils::derive_diffutils_inventory(
                crate::stagex_diffutils::DiffutilsInventoryRequest {
                    source_root: &sources.output_path,
                    tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                    musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                    scratch_dir: &stage.join("runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU diffutils requires authenticated source and protected GNU M4 completion"),
    };

    let (grep_sources, grep_runtime) = match (request.source_bundle_path, diffutils_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("grep-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU grep stage root", source))?;
            let sources = crate::stagex_grep::materialize_authenticated_grep_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_grep::derive_grep_inventory(crate::stagex_grep::GrepInventoryRequest {
                source_root: &sources.output_path,
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU grep requires authenticated source and protected GNU diffutils completion"),
    };

    let (gawk_sources, gawk_runtime) = match (request.source_bundle_path, grep_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("gawk-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU Gawk stage root", source))?;
            let sources = crate::stagex_gawk::materialize_authenticated_gawk_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_gawk::derive_gawk_inventory(crate::stagex_gawk::GawkInventoryRequest {
                source_root: &sources.output_path,
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU Gawk requires authenticated source and protected GNU grep completion"),
    };

    let (bison_sources, bison_runtime) = match (request.source_bundle_path, gawk_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("bison-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU Bison stage root", source))?;
            let sources = crate::stagex_bison::materialize_authenticated_bison_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_bison::derive_bison_inventory(crate::stagex_bison::BisonInventoryRequest {
                source_root: &sources.output_path,
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                m4_root: &request.scratch_dir.join("m4-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU Bison requires authenticated source and protected GNU Gawk completion"),
    };

    let (flex_sources, flex_runtime) = match (request.source_bundle_path, bison_runtime.as_ref()) {
        (Some(source_bundle_path), Some(_)) => {
            let stage = request.scratch_dir.join("flex-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected GNU Flex stage root", source))?;
            let sources = crate::stagex_flex::materialize_authenticated_flex_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let result = crate::stagex_flex::derive_flex_inventory(crate::stagex_flex::FlexInventoryRequest {
                source_root: &sources.output_path,
                tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                m4_root: &request.scratch_dir.join("m4-stage/runtime/output"),
                scratch_dir: &stage.join("runtime"),
                protected_exec_enforced: true,
            });
            match result {
                Ok(report) => (Some(sources), Some(report)),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => (None, None),
        _ => unreachable!("GNU Flex requires authenticated source and protected GNU Bison completion"),
    };

    let bash_full_runtime = match (request.source_bundle_path, flex_runtime.as_ref()) {
        (Some(_), Some(_)) => {
            let stage = request.scratch_dir.join("bash-full-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected full Bash stage root", source))?;
            let result = crate::stagex_bash_full::derive_bash_full_inventory(
                crate::stagex_bash_full::BashFullInventoryRequest {
                    configured_source_root: &request.scratch_dir.join("bash-stage/runtime/bash-2.05b"),
                    tcc_musl_v2_root: &request.scratch_dir.join("tcc-musl-v2-stage/runtime/output"),
                    musl_native_root: &request.scratch_dir.join("musl-native-stage/runtime/output"),
                    make: &request.scratch_dir.join("make-stage/runtime/output/bin/make"),
                    scratch_dir: &stage.join("runtime"),
                    protected_exec_enforced: true,
                },
            );
            match result {
                Ok(report) => Some(report),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    return Err(StagexTransitionError::ProtectedExec(error.to_string()));
                }
            }
        }
        (None, None) => None,
        _ => unreachable!("full Bash requires authenticated source closure and protected GNU Flex completion"),
    };

    crate::protected_exec_seccomp::reap_adopted_exec_descendants()
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let binutils_event_start = supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let (binutils_sources, binutils_runtime) =
        materialize_binutils_stage(&request, &manifest_authority, &supervisor, bash_full_runtime.is_some())?;
    crate::protected_exec_seccomp::reap_adopted_exec_descendants()
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let quiescent_event_count = supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let protected_exec_events = supervisor.audit_events();
    let binutils_event_count = binutils_runtime.as_ref().map(|_| {
        quiescent_event_count
            .checked_sub(binutils_event_start)
            .expect("binutils audit end follows its start")
    });
    if protected_exec_events.len() != quiescent_event_count {
        return Err(StagexTransitionError::Audit("protected exec audit changed after quiescence".to_string()));
    }
    write_json_create_new(&request.scratch_dir.join(AUDIT_FILE_NAME), &protected_exec_events)?;
    validate_transition_audit(
        &staged,
        TransitionAuditReports {
            stage0_full: stage0_full.as_ref(),
            mes_m2: mes_m2.as_ref(),
            mes_runtime: mes_runtime.as_ref(),
            tinycc_runtime: tinycc_runtime.as_ref(),
            tinycc27_runtime: tinycc27_runtime.as_ref(),
            make_runtime: make_runtime.as_ref(),
            gnu_patch_runtime: gnu_patch_runtime.as_ref(),
            gzip_runtime: gzip_runtime.as_ref(),
            tar_runtime: tar_runtime.as_ref(),
            sed_runtime: sed_runtime.as_ref(),
            bzip2_runtime: bzip2_runtime.as_ref(),
            coreutils_runtime: coreutils_runtime.as_ref(),
            oyacc_runtime: oyacc_runtime.as_ref(),
            bash_runtime: bash_runtime.as_ref(),
            tcc_musl_prep_runtime: tcc_musl_prep_runtime.as_ref(),
            musl_runtime: musl_runtime.as_ref(),
            tcc_musl_runtime: tcc_musl_runtime.as_ref(),
            musl_pass2_runtime: musl_pass2_runtime.as_ref(),
            tcc_musl_v2_runtime: tcc_musl_v2_runtime.as_ref(),
            tcc_selfhost_runtime: tcc_selfhost_runtime.as_ref(),
            musl_native_runtime: musl_native_runtime.as_ref(),
            m4_runtime: m4_runtime.as_ref(),
            diffutils_runtime: diffutils_runtime.as_ref(),
            grep_runtime: grep_runtime.as_ref(),
            gawk_runtime: gawk_runtime.as_ref(),
            bison_runtime: bison_runtime.as_ref(),
            flex_runtime: flex_runtime.as_ref(),
            bash_full_runtime: bash_full_runtime.as_ref(),
            binutils_runtime: binutils_runtime.as_ref(),
            binutils_event_count,
        },
        &protected_exec_events,
    )?;
    let kaem_smoke_digest_blake3 = blake3_file_hex(&staged.kaem_smoke).map_err(protected_digest_error)?;
    let report = StagexTransitionReport {
        schema_version: STAGEX_TRANSITION_REPORT_SCHEMA_V1.to_string(),
        status: "complete".to_string(),
        plan_digest_blake3: stagex_plan_digest_blake3(&plan)
            .map_err(|error| StagexTransitionError::Plan(vec![error.to_string()]))?,
        source_state_digest_blake3,
        seed_digest_blake3: HEX0_SEED_BLAKE3.to_string(),
        reproduced_hex0_digest_blake3: HEX0_SEED_BLAKE3.to_string(),
        kaem_digest_blake3: KAEM_OUTPUT_BLAKE3.to_string(),
        kaem_smoke_digest_blake3,
        stage0_full,
        mes_m2,
        mes_runtime,
        tinycc_sources,
        tinycc_runtime,
        tinycc27_sources,
        tinycc27_runtime,
        make_sources,
        make_runtime,
        gnu_patch_sources,
        gnu_patch_runtime,
        gzip_sources,
        gzip_runtime,
        tar_sources,
        tar_runtime,
        sed_sources,
        sed_runtime,
        bzip2_sources,
        bzip2_runtime,
        coreutils_sources,
        coreutils_runtime,
        oyacc_sources,
        oyacc_runtime,
        bash_sources,
        bash_runtime,
        tcc_musl_prep_runtime,
        musl_sources,
        musl_runtime,
        tcc_musl_runtime,
        musl_pass2_runtime,
        tcc_musl_v2_runtime,
        tcc_selfhost_runtime,
        musl_native_runtime,
        m4_sources,
        m4_runtime,
        diffutils_sources,
        diffutils_runtime,
        grep_sources,
        grep_runtime,
        gawk_sources,
        gawk_runtime,
        bison_sources,
        bison_runtime,
        flex_sources,
        flex_runtime,
        bash_full_runtime,
        binutils_sources,
        binutils_runtime,
        fallback_events: Vec::new(),
        promotions: vec![hex0_promotion, kaem_promotion],
        protected_exec_events,
    };
    write_json_create_new(&request.scratch_dir.join(REPORT_FILE_NAME), &report)?;
    Ok(report)
}

fn materialize_binutils_stage(
    request: &StagexTransitionRequest<'_>,
    manifest_authority: &TransitionManifestAuthority,
    supervisor: &ProtectedSeccompSupervisor,
    bash_full_complete: bool,
) -> Result<
    (
        Option<crate::stagex_binutils::BinutilsSourceMaterializationReport>,
        Option<crate::stagex_binutils::BinutilsInventoryReport>,
    ),
    StagexTransitionError,
> {
    match (request.source_bundle_path, bash_full_complete) {
        (Some(source_bundle_path), true) => {
            let stage = request.scratch_dir.join("binutils-stage");
            fs::create_dir(&stage)
                .map_err(|source| io_error("creating create-new protected binutils stage root", source))?;
            let sources = crate::stagex_binutils::materialize_authenticated_binutils_source(
                source_bundle_path,
                &manifest_authority.source_bundle_manifest_blake3,
                &stage.join("sources"),
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            let paths = BinutilsStagePaths::from_transition_root(request.scratch_dir);
            let result =
                crate::stagex_binutils::derive_binutils_inventory(paths.probe_request(&sources.output_path), true);
            match result {
                Ok(report) => Ok((Some(sources), Some(report))),
                Err(error) => {
                    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
                    let events = supervisor.audit_events();
                    write_json_create_new(&request.scratch_dir.join(FAILURE_AUDIT_FILE_NAME), &events)?;
                    Err(StagexTransitionError::ProtectedExec(error.to_string()))
                }
            }
        }
        (None, false) => Ok((None, None)),
        _ => Err(StagexTransitionError::InvalidInput(
            "binutils requires authenticated source closure and protected full Bash completion".to_string(),
        )),
    }
}

fn validate_stage0_optional_inputs(
    source_bundle_path: Option<&Path>,
    answers_path: Option<&Path>,
) -> Result<(), StagexTransitionError> {
    if source_bundle_path.is_some() == answers_path.is_some() {
        return Ok(());
    }
    Err(StagexTransitionError::InvalidInput(
        "Stage0 source bundle and answers path must be supplied together".to_string(),
    ))
}

fn validate_optional_source_bundle(
    source_bundle_path: Option<&Path>,
    manifest_authority: &TransitionManifestAuthority,
) -> Result<(), StagexTransitionError> {
    let Some(source_bundle_path) = source_bundle_path else {
        return Ok(());
    };
    let manifest = crate::source_bundle::read_source_bundle(source_bundle_path)
        .map_err(|error| StagexTransitionError::InvalidInput(error.to_string()))?;
    if manifest.manifest_blake3 != manifest_authority.source_bundle_manifest_blake3 {
        return Err(StagexTransitionError::InvalidInput(format!(
            "Stage0 source bundle BLAKE3 mismatch: expected {}, observed {}",
            manifest_authority.source_bundle_manifest_blake3, manifest.manifest_blake3
        )));
    }
    assert!(!manifest.records.is_empty());
    assert_eq!(manifest.manifest_blake3, manifest_authority.source_bundle_manifest_blake3);
    Ok(())
}

fn load_transition_manifest_authority(path: &Path) -> Result<TransitionManifestAuthority, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error("reading StageX transition manifest", source))?;
    validate_size("transition manifest", bytes.len(), LINEAGE_MANIFEST_MAX_BYTES)?;
    let manifest = crate::bootstrap_source_root::validate_stagex_lineage_manifest(&bytes).map_err(|diagnostics| {
        StagexTransitionError::InvalidInput(crate::bootstrap_source_root::format_diagnostics(&diagnostics))
    })?;
    let source_bundle_manifest_blake3 = validate_transition_manifest_fields(&manifest)?;
    Ok(TransitionManifestAuthority {
        digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        source_bundle_manifest_blake3,
    })
}

fn validate_transition_manifest_fields(manifest: &LineageManifest) -> Result<String, StagexTransitionError> {
    if manifest.seed.seed_bytes_len != HEX0_SEED_BYTES || manifest.seed.seed_digest.hex_value != HEX0_SEED_BLAKE3 {
        return Err(StagexTransitionError::InvalidInput(
            "transition manifest does not bind the audited 229-byte hex0 seed".to_string(),
        ));
    }
    let sources: BTreeMap<&str, &str> = manifest
        .source_artifacts
        .iter()
        .map(|source| (source.id.as_str(), source.digest.hex_value.as_str()))
        .collect();
    require_manifest_digest(&sources, "hex0-source", HEX0_SOURCE_BLAKE3)?;
    require_manifest_digest(&sources, "kaem-minimal-source", KAEM_SOURCE_BLAKE3)?;
    require_manifest_digest(&sources, "kaem-smoke-script", EMPTY_KAEM_SMOKE_SOURCE_BLAKE3)?;
    require_manifest_digest(
        &sources,
        "stage0-source-bundle",
        crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
    )?;
    for (artifact_id, digest_blake3) in crate::stagex_sources::expected_stagex_source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    require_manifest_digest(&sources, "stage0-amd64-answers", crate::stagex_stage0_full::STAGE0_ANSWERS_BLAKE3)?;
    require_manifest_digest(&sources, "stage0-after-script", EMPTY_KAEM_SMOKE_SOURCE_BLAKE3)?;
    for (artifact_id, digest_blake3) in crate::stagex_mes_sources::expected_mes_source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tinycc::tinycc_source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    let (tinycc27_source_id, tinycc27_source_blake3) = crate::stagex_tinycc27::source_artifact_digest();
    require_manifest_digest(&sources, tinycc27_source_id, tinycc27_source_blake3)?;
    for (artifact_id, digest_blake3) in crate::stagex_make::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    let (gnu_patch_source_id, gnu_patch_source_blake3) = crate::stagex_gnu_patch::source_artifact_digest();
    require_manifest_digest(&sources, gnu_patch_source_id, gnu_patch_source_blake3)?;
    for (artifact_id, digest_blake3) in crate::stagex_gzip::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tar::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_sed::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_bzip2::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_coreutils::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_oyacc::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_bash::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tcc_musl_prep::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_musl::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tcc_musl::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_musl::pass2_source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tcc_musl_v2::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_tcc_selfhost::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_musl_native::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_m4::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_diffutils::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_grep::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_gawk::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    for (artifact_id, digest_blake3) in crate::stagex_bison::source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    let generated: BTreeMap<&str, &str> = manifest
        .generated_artifacts
        .iter()
        .map(|artifact| (artifact.id.as_str(), artifact.digest.hex_value.as_str()))
        .collect();
    require_manifest_digest(&generated, "hex0-reproduced", HEX0_SEED_BLAKE3)?;
    require_manifest_digest(&generated, "kaem-0", KAEM_OUTPUT_BLAKE3)?;
    for expected in crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES {
        if expected.artifact_id == "stage0-hex0-copy" {
            continue;
        }
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "mes-m2", crate::stagex_mes::MES_M2_BLAKE3)?;
    require_manifest_digest(&generated, "mes", crate::stagex_mes::MES_M2_BLAKE3)?;
    for expected in crate::stagex_mes_lib::MES_RUNTIME_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_tinycc::TINYCC_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_tinycc27::TINYCC27_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_make::MAKE_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_gnu_patch::GNU_PATCH_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_gzip::GZIP_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_tar::TAR_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_sed::SED_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "bzip2-configured-source",
        crate::stagex_bzip2::BZIP2_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_bzip2::BZIP2_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "coreutils-configured-source",
        crate::stagex_coreutils::COREUTILS_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_coreutils::COREUTILS_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "oyacc-configured-source",
        crate::stagex_oyacc::OYACC_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_oyacc::OYACC_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "bash-configured-source", crate::stagex_bash::BASH_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_bash::BASH_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "tcc-musl-prep-configured-source",
        crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "musl-configured-source", crate::stagex_musl::MUSL_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_musl::MUSL_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "tcc-musl-configured-source",
        crate::stagex_tcc_musl::TCC_MUSL_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_tcc_musl::TCC_MUSL_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "musl-pass2-configured-source",
        crate::stagex_musl::MUSL_PASS2_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_musl::MUSL_PASS2_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "tcc-musl-v2-configured-source",
        crate::stagex_tcc_musl_v2::CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_tcc_musl_v2::EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_tcc_selfhost::EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "musl-native-configured-source",
        crate::stagex_musl_native::CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_musl_native::EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "m4-configured-source", crate::stagex_m4::M4_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_m4::M4_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "diffutils-configured-source",
        crate::stagex_diffutils::DIFFUTILS_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_diffutils::DIFFUTILS_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "grep-configured-source", crate::stagex_grep::GREP_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_grep::GREP_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "gawk-configured-source", crate::stagex_gawk::GAWK_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_gawk::GAWK_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        "bison-configured-source",
        crate::stagex_bison::BISON_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_bison::BISON_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(&generated, "flex-configured-source", crate::stagex_flex::FLEX_CONFIGURED_SOURCE_BLAKE3)?;
    for expected in crate::stagex_flex::FLEX_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    require_manifest_digest(
        &generated,
        crate::stagex_bash_full::BASH_FULL_GENERATOR_ARTIFACT_ID,
        crate::stagex_bash_full::BASH_FULL_GENERATOR_BLAKE3,
    )?;
    require_manifest_digest(
        &generated,
        crate::stagex_bash_full::BASH_FULL_GENERATED_TREE_ARTIFACT_ID,
        crate::stagex_bash_full::BASH_FULL_GENERATED_TREE_BLAKE3,
    )?;
    require_manifest_digest(
        &generated,
        crate::stagex_bash_full::BASH_FULL_CONFIGURED_SOURCE_ARTIFACT_ID,
        crate::stagex_bash_full::BASH_FULL_CONFIGURED_SOURCE_BLAKE3,
    )?;
    for expected in crate::stagex_bash_full::BASH_FULL_EXPECTED_OUTPUTS {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    let patches: BTreeMap<&str, &str> =
        manifest.patches.iter().map(|patch| (patch.id.as_str(), patch.digest.hex_value.as_str())).collect();
    require_manifest_digest(&patches, "tinycc-0.9.27-stagex-patch", crate::stagex_tinycc27::TINYCC27_PATCH_BLAKE3)?;
    require_manifest_digest(&patches, "make-3.82-stagex-patch", crate::stagex_make::MAKE_PATCH_BLAKE3)?;
    let assumption_ids: BTreeSet<&str> = manifest.environment_assumptions.iter().map(|item| item.id.as_str()).collect();
    if !assumption_ids.contains("linux-kernel") || !assumption_ids.contains("mantle-orchestrator") {
        return Err(StagexTransitionError::InvalidInput(
            "transition manifest omits kernel or orchestrator assumptions".to_string(),
        ));
    }
    Ok(crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3.to_string())
}

fn require_manifest_digest(
    entries: &BTreeMap<&str, &str>,
    id: &str,
    expected_digest: &str,
) -> Result<(), StagexTransitionError> {
    if entries.get(id).copied() == Some(expected_digest) {
        return Ok(());
    }
    Err(StagexTransitionError::InvalidInput(format!(
        "transition manifest has missing or substituted digest for {id}"
    )))
}

fn stage_transition_inputs(
    request: &StagexTransitionRequest<'_>,
) -> Result<StagedTransitionPaths, StagexTransitionError> {
    if request.scratch_dir.exists() {
        return Err(StagexTransitionError::InvalidInput(format!(
            "scratch path already exists: {}",
            request.scratch_dir.display()
        )));
    }
    fs::create_dir(request.scratch_dir).map_err(|source| io_error("creating StageX scratch", source))?;
    let seed_bytes = read_checked_input(request.seed_path, "hex0 seed", HEX0_SEED_BLAKE3, AUDIT_SEED_MAX_BYTES)?;
    let seed_len = u32::try_from(seed_bytes.len())
        .map_err(|_| StagexTransitionError::InvalidInput("hex0 seed length exceeds u32".to_string()))?;
    if seed_len != HEX0_SEED_BYTES {
        return Err(StagexTransitionError::InvalidInput(format!(
            "hex0 seed has {seed_len} bytes; expected {HEX0_SEED_BYTES}"
        )));
    }
    let hex0_source_bytes =
        read_checked_input(request.hex0_source_path, "hex0 source", HEX0_SOURCE_BLAKE3, SOURCE_FILE_MAX_BYTES)?;
    let kaem_source_bytes =
        read_checked_input(request.kaem_source_path, "kaem source", KAEM_SOURCE_BLAKE3, SOURCE_FILE_MAX_BYTES)?;

    let staged = StagedTransitionPaths {
        seed: request.scratch_dir.join("hex0-seed"),
        hex0_source: request.scratch_dir.join("hex0_AMD64.hex0"),
        kaem_source: request.scratch_dir.join("kaem-minimal.hex0"),
        reproduced_hex0: request.scratch_dir.join("hex0-reproduced"),
        kaem: request.scratch_dir.join("kaem-0"),
        kaem_smoke_script: request.scratch_dir.join("kaem-smoke.kaem"),
        kaem_smoke: request.scratch_dir.join("kaem-smoke.txt"),
    };
    write_file_create_new(&staged.seed, &seed_bytes)?;
    write_file_create_new(&staged.hex0_source, &hex0_source_bytes)?;
    write_file_create_new(&staged.kaem_source, &kaem_source_bytes)?;
    write_file_create_new(&staged.kaem_smoke_script, &[])?;
    set_executable_owner_only(&staged.seed)?;
    Ok(staged)
}

fn source_state_digest(
    staged: &StagedTransitionPaths,
    source_bundle_manifest_blake3: Option<&str>,
) -> Result<String, StagexTransitionError> {
    let seed = fs::read(&staged.seed).map_err(|source| io_error("reading staged seed", source))?;
    let hex0_source = fs::read(&staged.hex0_source).map_err(|source| io_error("reading staged hex0 source", source))?;
    let kaem_source = fs::read(&staged.kaem_source).map_err(|source| io_error("reading staged kaem source", source))?;
    let kaem_smoke_script =
        fs::read(&staged.kaem_smoke_script).map_err(|source| io_error("reading staged kaem smoke script", source))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_STATE_DOMAIN);
    update_length_bound_bytes(&mut hasher, &seed)?;
    update_length_bound_bytes(&mut hasher, &hex0_source)?;
    update_length_bound_bytes(&mut hasher, &kaem_source)?;
    update_length_bound_bytes(&mut hasher, &kaem_smoke_script)?;
    if let Some(source_bundle_manifest_blake3) = source_bundle_manifest_blake3 {
        update_length_bound_bytes(&mut hasher, source_bundle_manifest_blake3.as_bytes())?;
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn update_length_bound_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), StagexTransitionError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| StagexTransitionError::InvalidInput("source length exceeds u64".to_string()))?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn build_transition_plan(
    manifest_authority: &TransitionManifestAuthority,
    staged: &StagedTransitionPaths,
    source_state_digest_blake3: &str,
    include_stage0_mini: bool,
) -> Result<StagexMaterializationPlan, StagexTransitionError> {
    let mut stages = vec![
        transition_stage_plan(
            HEX0_REPRODUCTION_STAGE_ID,
            &[],
            &["hex0-source"],
            &[],
            &["hex0-reproduced"],
            authorization("exec:hex0-seed", &staged.seed, STAGEX_SEED_SOURCE_STAGE_ID, HEX0_SEED_BLAKE3),
        ),
        transition_stage_plan(
            KAEM_MATERIALIZATION_STAGE_ID,
            &[HEX0_REPRODUCTION_STAGE_ID],
            &["kaem-minimal-source"],
            &["hex0-reproduced"],
            &["kaem-0"],
            authorization(
                "exec:hex0-reproduced",
                &staged.reproduced_hex0,
                HEX0_REPRODUCTION_STAGE_ID,
                HEX0_SEED_BLAKE3,
            ),
        ),
        transition_stage_plan(
            KAEM_SMOKE_STAGE_ID,
            &[KAEM_MATERIALIZATION_STAGE_ID],
            &["kaem-smoke-script"],
            &["kaem-0"],
            &["kaem-smoke-observation"],
            authorization("exec:kaem-0", &staged.kaem, KAEM_MATERIALIZATION_STAGE_ID, KAEM_OUTPUT_BLAKE3),
        ),
    ];
    if include_stage0_mini {
        let stage0_root = staged.seed.parent().expect("staged seed has parent").join("stage0-mini");
        stages.extend(stage0_mini_stage_plans(&stage0_root));
        stages.extend(stage0_full_stage_plans(&stage0_root));
        let mes_root = staged.seed.parent().expect("staged seed has parent").join("mes-stage/mes-0.27.1");
        stages.extend(mes_m2_stage_plans(&stage0_root, &mes_root));
        stages.extend(mes_runtime_stage_plans(&stage0_root, &mes_root));
        let tinycc_root = staged.seed.parent().expect("staged seed has parent").join("tinycc-stage/runtime");
        stages.extend(tinycc_stage_plans(&stage0_root, &mes_root, &tinycc_root));
        let tinycc27_root = staged.seed.parent().expect("staged seed has parent").join("tinycc27-stage/runtime");
        stages.extend(tinycc27_stage_plans(&tinycc_root, &tinycc27_root));
        let make_root = staged.seed.parent().expect("staged seed has parent").join("make-stage/runtime");
        stages.extend(make_stage_plans(&tinycc27_root, &make_root));
        let patch_root = staged.seed.parent().expect("staged seed has parent").join("gnu-patch-stage/runtime");
        stages.extend(gnu_patch_stage_plans(&tinycc27_root, &patch_root));
        let gzip_root = staged.seed.parent().expect("staged seed has parent").join("gzip-stage/runtime");
        stages.extend(gzip_stage_plans(&tinycc27_root, &gzip_root));
        let tar_root = staged.seed.parent().expect("staged seed has parent").join("tar-stage/runtime");
        stages.extend(tar_stage_plans(&tinycc27_root, &tar_root));
        let sed_root = staged.seed.parent().expect("staged seed has parent").join("sed-stage/runtime");
        stages.extend(sed_stage_plans(&tinycc27_root, &sed_root));
        let bzip2_root = staged.seed.parent().expect("staged seed has parent").join("bzip2-stage/runtime");
        stages.extend(bzip2_stage_plans(&tinycc27_root, &sed_root, &bzip2_root));
        let coreutils_root = staged.seed.parent().expect("staged seed has parent").join("coreutils-stage/runtime");
        stages.extend(coreutils_stage_plans(&tinycc27_root, &patch_root, &coreutils_root));
        let oyacc_root = staged.seed.parent().expect("staged seed has parent").join("oyacc-stage/runtime");
        stages.extend(oyacc_stage_plans(&tinycc27_root, &patch_root, &oyacc_root));
        let bash_root = staged.seed.parent().expect("staged seed has parent").join("bash-stage/runtime");
        stages.extend(bash_stage_plans(&tinycc27_root, &bash_root));
        let tcc_musl_prep_root =
            staged.seed.parent().expect("staged seed has parent").join("tcc-musl-prep-stage/runtime");
        stages.extend(tcc_musl_prep_stage_plans(&tinycc27_root, &tcc_musl_prep_root));
        stages.extend(musl_stage_plans(&tcc_musl_prep_root));
        let tcc_musl_root = staged.seed.parent().expect("staged seed has parent").join("tcc-musl-stage/runtime");
        stages.extend(tcc_musl_stage_plans(&tcc_musl_prep_root, &tcc_musl_root));
        stages.extend(musl_pass2_stage_plans(&tcc_musl_root));
        let tinycc_root = staged.seed.parent().expect("staged seed has parent").join("tinycc-stage/runtime");
        let tcc_musl_v2_root = staged.seed.parent().expect("staged seed has parent").join("tcc-musl-v2-stage/runtime");
        stages.extend(tcc_musl_v2_stage_plans(&tinycc_root, &tcc_musl_v2_root));
        let tcc_selfhost_root =
            staged.seed.parent().expect("staged seed has parent").join("tcc-musl-selfhost-stage/runtime");
        stages.extend(tcc_selfhost_stage_plans(&tinycc_root, &tcc_musl_v2_root, &tcc_selfhost_root));
        let musl_native_root = staged.seed.parent().expect("staged seed has parent").join("musl-native-stage/runtime");
        stages.extend(musl_native_stage_plans(&tcc_musl_v2_root, &tcc_selfhost_root, &musl_native_root));
        let m4_root = staged.seed.parent().expect("staged seed has parent").join("m4-stage/runtime");
        stages.extend(m4_stage_plans(&tcc_musl_v2_root, &musl_native_root, &m4_root));
        let diffutils_root = staged.seed.parent().expect("staged seed has parent").join("diffutils-stage/runtime");
        stages.extend(diffutils_stage_plans(&tcc_musl_v2_root, &musl_native_root, &diffutils_root));
        let grep_root = staged.seed.parent().expect("staged seed has parent").join("grep-stage/runtime");
        stages.extend(grep_stage_plans(&tcc_musl_v2_root, &musl_native_root, &grep_root));
        let gawk_root = staged.seed.parent().expect("staged seed has parent").join("gawk-stage/runtime");
        stages.extend(gawk_stage_plans(&tcc_musl_v2_root, &musl_native_root, &gawk_root));
        let bison_root = staged.seed.parent().expect("staged seed has parent").join("bison-stage/runtime");
        let m4_root = staged.seed.parent().expect("staged seed has parent").join("m4-stage/runtime");
        stages.extend(bison_stage_plans(&tcc_musl_v2_root, &musl_native_root, &m4_root, &bison_root));
        let flex_root = staged.seed.parent().expect("staged seed has parent").join("flex-stage/runtime");
        stages.extend(flex_stage_plans(&tcc_musl_v2_root, &musl_native_root, &m4_root, &flex_root));
        let bash_full_root = staged.seed.parent().expect("staged seed has parent").join("bash-full-stage/runtime");
        stages.extend(bash_full_stage_plans(
            &tcc_musl_v2_root,
            &musl_native_root,
            &make_root,
            &bash_root,
            &bash_full_root,
        ));
        let transition_root = staged.seed.parent().expect("staged seed has parent");
        stages.extend(binutils_stage_plans(transition_root));
    }
    let stage_count_max = if include_stage0_mini {
        TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX
    } else {
        TRANSITION_STAGE_COUNT_MAX
    };
    let plan = StagexMaterializationPlan {
        schema_version: STAGEX_PLAN_SCHEMA_V1.to_string(),
        lineage_manifest_digest_blake3: manifest_authority.digest_blake3.clone(),
        source_state_digest_blake3: source_state_digest_blake3.to_string(),
        protected_transition_stage_id: HEX0_REPRODUCTION_STAGE_ID.to_string(),
        environment_assumption_ids: vec!["linux-kernel".to_string(), "mantle-orchestrator".to_string()],
        stage_count_max,
        stages,
    };
    let validation = validate_stagex_plan(&plan);
    if validation.is_valid() {
        return Ok(plan);
    }
    Err(StagexTransitionError::Plan(validation.errors.into_iter().map(|error| error.to_string()).collect()))
}

fn tinycc_stage_plans(stage0_root: &Path, mes_root: &Path, tinycc_root: &Path) -> Vec<StagexStagePlan> {
    let tcc_mes = tinycc_root.join("tinycc-0.9.26/tcc-mes");
    let boot0 = tinycc_root.join("tinycc-0.9.26/tcc-boot0");
    let final_tcc = tinycc_root.join("output/bin/tcc");
    vec![
        mes_stage_plan(
            TINYCC_MES_STAGE_ID,
            &[
                MES_RUNTIME_STAGE_ID,
                MES_M2_STAGE_ID,
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-blood-elf",
            ],
            &["tinycc-0.9.26-source", "tinycc-mes-0.27.1-source"],
            &[
                "mes-m2",
                "mescc-entrypoint",
                "mes-libc-tcc",
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-blood-elf",
            ],
            &["tcc-mes-assembly".to_string(), "tcc-mes".to_string()],
            vec![
                mes_m2_authorization(TINYCC_MES_STAGE_ID, mes_root),
                stage0_authorization(TINYCC_MES_STAGE_ID, stage0_root, "stage0-m1"),
                stage0_authorization(TINYCC_MES_STAGE_ID, stage0_root, "stage0-hex2"),
                stage0_full_authorization(TINYCC_MES_STAGE_ID, stage0_root, "stage0-full-blood-elf"),
            ],
        ),
        mes_stage_plan(
            TINYCC_RUNTIME_STAGE_ID,
            &[TINYCC_MES_STAGE_ID, MES_RUNTIME_STAGE_ID],
            &[],
            &["tcc-mes", "mes-crt1", "mes-libc-tcc"],
            &[
                "tinycc-crt1".to_string(),
                "tinycc-libc".to_string(),
                "tinycc-libtcc1".to_string(),
                "tinycc-libgetopt".to_string(),
            ],
            vec![authorization(
                "exec:tinycc-runtime-refresh:tcc-mes",
                &tcc_mes,
                TINYCC_MES_STAGE_ID,
                crate::stagex_tinycc::TCC_MES_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TINYCC_BOOT0_STAGE_ID,
            &[TINYCC_RUNTIME_STAGE_ID, TINYCC_MES_STAGE_ID],
            &[],
            &["tcc-mes", "tinycc-crt1", "tinycc-libc", "tinycc-libtcc1"],
            &["tcc-boot0".to_string()],
            vec![authorization(
                "exec:tinycc-boot0-materialization:tcc-mes",
                &tcc_mes,
                TINYCC_MES_STAGE_ID,
                crate::stagex_tinycc::TCC_MES_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TINYCC_BOOT0_SMOKE_STAGE_ID,
            &[TINYCC_BOOT0_STAGE_ID],
            &[],
            &["tcc-boot0"],
            &["tcc-boot0-smoke-observation".to_string()],
            vec![authorization(
                "exec:tinycc-boot0-smoke:tcc-boot0",
                &boot0,
                TINYCC_BOOT0_STAGE_ID,
                crate::stagex_tinycc::TCC_BOOT0_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TINYCC_FINAL_STAGE_ID,
            &[
                TINYCC_BOOT0_SMOKE_STAGE_ID,
                TINYCC_MES_STAGE_ID,
                MES_M2_STAGE_ID,
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-blood-elf",
            ],
            &[],
            &[
                "tcc-boot0-smoke-observation",
                "mes-m2",
                "tcc-mes",
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-blood-elf",
            ],
            &["tcc-final-assembly".to_string(), "tinycc-0.9.26".to_string()],
            vec![
                mes_m2_authorization(TINYCC_FINAL_STAGE_ID, mes_root),
                stage0_authorization(TINYCC_FINAL_STAGE_ID, stage0_root, "stage0-m1"),
                stage0_authorization(TINYCC_FINAL_STAGE_ID, stage0_root, "stage0-hex2"),
                stage0_full_authorization(TINYCC_FINAL_STAGE_ID, stage0_root, "stage0-full-blood-elf"),
            ],
        ),
        mes_stage_plan(
            TINYCC_FINAL_SMOKE_STAGE_ID,
            &[TINYCC_FINAL_STAGE_ID],
            &[],
            &["tinycc-0.9.26"],
            &["tinycc-final-smoke-observation".to_string()],
            vec![authorization(
                "exec:tinycc-final-smoke:tinycc",
                &final_tcc,
                TINYCC_FINAL_STAGE_ID,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn tinycc27_stage_plans(tinycc_root: &Path, tinycc27_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc26 = tinycc_root.join("output/bin/tcc");
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    vec![
        mes_stage_plan(
            TINYCC27_RUNTIME_STAGE_ID,
            &[
                TINYCC_FINAL_SMOKE_STAGE_ID,
                TINYCC_FINAL_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[
                crate::stagex_tinycc27::TINYCC27_SOURCE_ARTIFACT_ID,
                "tinycc-mes-0.27.1-source",
            ],
            &[
                "tinycc-0.9.26",
                "tinycc-crt1",
                "tinycc-libc",
                "tinycc-libtcc1",
                "tinycc-libgetopt",
            ],
            &["tinycc27-va-list".to_string(), "tinycc27-libc".to_string()],
            vec![authorization(
                "exec:tinycc27-runtime-refresh:tinycc26",
                &tinycc26,
                TINYCC_FINAL_STAGE_ID,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TINYCC27_BUILD_STAGE_ID,
            &[
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_FINAL_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_tinycc27::TINYCC27_SOURCE_ARTIFACT_ID],
            &["tinycc-0.9.26", "tinycc27-va-list", "tinycc27-libc", "tinycc-libtcc1"],
            &["tinycc-0.9.27".to_string(), "tinycc27-alias".to_string()],
            vec![authorization(
                "exec:tinycc27-materialization:tinycc26",
                &tinycc26,
                TINYCC_FINAL_STAGE_ID,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TINYCC27_SMOKE_STAGE_ID,
            &[TINYCC27_BUILD_STAGE_ID],
            &[],
            &["tinycc-0.9.27"],
            &[
                "tinycc27-positive-object".to_string(),
                "tinycc27-version-observation".to_string(),
                "tinycc27-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:tinycc27-smoke:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn make_stage_plans(tinycc27_root: &Path, make_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let recipe_runner = make_root.join("output/bin/make-recipe-runner");
    let make = make_root.join("output/bin/make");
    vec![
        mes_stage_plan(
            MAKE_RECIPE_RUNNER_STAGE_ID,
            &[
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_make::MAKE_RECIPE_RUNNER_SOURCE_ARTIFACT_ID],
            &["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
            &["make-recipe-runner".to_string()],
            vec![authorization(
                "exec:make-recipe-runner-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            MAKE_BUILD_STAGE_ID,
            &[
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_make::MAKE_SOURCE_ARTIFACT_ID],
            &["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
            &["make-3.82".to_string()],
            vec![authorization(
                "exec:make-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            MAKE_SMOKE_STAGE_ID,
            &[MAKE_BUILD_STAGE_ID, MAKE_RECIPE_RUNNER_STAGE_ID],
            &[],
            &["make-3.82", "make-recipe-runner"],
            &[
                "make-recipe-smoke".to_string(),
                "make-version-observation".to_string(),
                "make-malformed-observation".to_string(),
            ],
            vec![
                authorization(
                    "exec:make-smoke:make",
                    &make,
                    MAKE_BUILD_STAGE_ID,
                    crate::stagex_make::MAKE_FINAL_BLAKE3,
                ),
                authorization(
                    "exec:make-smoke:recipe-runner",
                    &recipe_runner,
                    MAKE_RECIPE_RUNNER_STAGE_ID,
                    crate::stagex_make::MAKE_RECIPE_RUNNER_BLAKE3,
                ),
            ],
        ),
    ]
}

fn gnu_patch_stage_plans(tinycc27_root: &Path, patch_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let patch = patch_root.join("output/bin/patch");
    vec![
        mes_stage_plan(
            GNU_PATCH_BUILD_STAGE_ID,
            &[
                MAKE_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_gnu_patch::GNU_PATCH_SOURCE_ARTIFACT_ID],
            &[
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
                "make-recipe-smoke",
            ],
            &["gnu-patch-2.5.9".to_string()],
            vec![authorization(
                "exec:gnu-patch-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            GNU_PATCH_SMOKE_STAGE_ID,
            &[GNU_PATCH_BUILD_STAGE_ID],
            &[],
            &["gnu-patch-2.5.9"],
            &[
                "gnu-patch-smoke".to_string(),
                "gnu-patch-version-observation".to_string(),
                "gnu-patch-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:gnu-patch-smoke:patch",
                &patch,
                GNU_PATCH_BUILD_STAGE_ID,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn gzip_stage_plans(tinycc27_root: &Path, gzip_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let makecrc = gzip_root.join("generated/bin/makecrc");
    let gzip = gzip_root.join("output/bin/gzip");
    let gunzip = gzip_root.join("output/bin/gunzip");
    vec![
        mes_stage_plan(
            GZIP_GENERATOR_BUILD_STAGE_ID,
            &[
                GNU_PATCH_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_gzip::GZIP_MAKECRC_SOURCE_ARTIFACT_ID],
            &[
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
                "gnu-patch-smoke",
            ],
            &["gzip-makecrc".to_string()],
            vec![authorization(
                "exec:gzip-generator-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            GZIP_GENERATOR_RUN_STAGE_ID,
            &[GZIP_GENERATOR_BUILD_STAGE_ID],
            &[],
            &["gzip-makecrc"],
            &["gzip-crc-table".to_string()],
            vec![authorization(
                "exec:gzip-generator-execution:makecrc",
                &makecrc,
                GZIP_GENERATOR_BUILD_STAGE_ID,
                crate::stagex_gzip::GZIP_MAKECRC_BLAKE3,
            )],
        ),
        mes_stage_plan(
            GZIP_BUILD_STAGE_ID,
            &[
                GZIP_GENERATOR_RUN_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_gzip::GZIP_SOURCE_ARTIFACT_ID],
            &[
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
                "gzip-crc-table",
            ],
            &["gzip-1.2.4".to_string(), "gunzip-1.2.4".to_string()],
            vec![authorization(
                "exec:gzip-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            GZIP_SMOKE_STAGE_ID,
            &[GZIP_BUILD_STAGE_ID],
            &[],
            &["gzip-1.2.4", "gunzip-1.2.4"],
            &[
                "gzip-smoke".to_string(),
                "gzip-help-observation".to_string(),
                "gzip-negative-observation".to_string(),
            ],
            vec![
                authorization(
                    "exec:gzip-smoke:gzip",
                    &gzip,
                    GZIP_BUILD_STAGE_ID,
                    crate::stagex_gzip::GZIP_FINAL_BLAKE3,
                ),
                authorization(
                    "exec:gzip-smoke:gunzip",
                    &gunzip,
                    GZIP_BUILD_STAGE_ID,
                    crate::stagex_gzip::GZIP_FINAL_BLAKE3,
                ),
            ],
        ),
    ]
}

fn bzip2_stage_plans(tinycc27_root: &Path, sed_root: &Path, bzip2_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let sed = sed_root.join("output/bin/sed");
    let bzip2 = bzip2_root.join("output/bin/bzip2");
    vec![
        mes_stage_plan(
            BZIP2_REWRITE_STAGE_ID,
            &[SED_BUILD_STAGE_ID, SED_SMOKE_STAGE_ID],
            &[
                crate::stagex_bzip2::BZIP2_SOURCE_ARTIFACT_ID,
                crate::stagex_bzip2::BZIP2_UTIME_SOURCE_ARTIFACT_ID,
            ],
            &["sed-4.0.9", "sed-smoke"],
            &["bzip2-configured-source".to_string()],
            vec![authorization(
                "exec:bzip2-source-rewrite:sed",
                &sed,
                SED_BUILD_STAGE_ID,
                crate::stagex_sed::SED_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            BZIP2_BUILD_STAGE_ID,
            &[
                BZIP2_REWRITE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[],
            &[
                "bzip2-configured-source",
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
            ],
            &["bzip2-1.0.8".to_string(), "bzip2recover-1.0.8".to_string()],
            vec![authorization(
                "exec:bzip2-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            BZIP2_SMOKE_STAGE_ID,
            &[BZIP2_BUILD_STAGE_ID],
            &[],
            &["bzip2-1.0.8"],
            &[
                "bzip2-smoke".to_string(),
                "bzip2-help-observation".to_string(),
                "bzip2-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:bzip2-smoke:bzip2",
                &bzip2,
                BZIP2_BUILD_STAGE_ID,
                crate::stagex_bzip2::BZIP2_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn oyacc_stage_plans(tinycc27_root: &Path, patch_root: &Path, oyacc_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let patch = patch_root.join("output/bin/patch");
    let yacc = oyacc_root.join("output/bin/yacc");
    vec![
        mes_stage_plan(
            OYACC_PATCH_STAGE_ID,
            &[
                COREUTILS_BUILD_STAGE_ID,
                COREUTILS_SMOKE_STAGE_ID,
                GNU_PATCH_BUILD_STAGE_ID,
            ],
            &[
                "oyacc-6.6-source",
                "oyacc-smoke-grammar-source",
                "oyacc-malformed-grammar-source",
                "oyacc-empty-config-source",
                "oyacc-meslibc-patch-source",
                "oyacc-tcc-patch-source",
            ],
            &["gnu-patch-2.5.9", "coreutils-smoke"],
            &["oyacc-configured-source".to_string()],
            vec![authorization(
                "exec:oyacc-source-patch:patch",
                &patch,
                GNU_PATCH_BUILD_STAGE_ID,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            OYACC_BUILD_STAGE_ID,
            &[
                OYACC_PATCH_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[],
            &[
                "oyacc-configured-source",
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
            ],
            &["oyacc-6.6".to_string()],
            vec![authorization(
                "exec:oyacc-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            OYACC_SMOKE_STAGE_ID,
            &[OYACC_BUILD_STAGE_ID],
            &[],
            &["oyacc-6.6"],
            &[
                "oyacc-smoke-output".to_string(),
                "oyacc-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:oyacc-smoke:yacc",
                &yacc,
                OYACC_BUILD_STAGE_ID,
                crate::stagex_oyacc::OYACC_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn tcc_musl_v2_stage_plans(tinycc_root: &Path, v2_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc = tinycc_root.join("output/bin/tcc-0.9.26");
    let compiler = v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let positive = v2_root.join("smoke/variadic-positive");
    vec![
        mes_stage_plan(
            TCC_MUSL_V2_BUILD_STAGE_ID,
            &[
                TINYCC_FINAL_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
                TCC_MUSL_PREP_BUILD_STAGE_ID,
                MUSL_PASS2_BUILD_STAGE_ID,
            ],
            &[
                "tinycc-0.9.27-source",
                "tinycc-0.9.27-patch-source",
                "tcc-musl-v2-recipe-source",
                "tcc-musl-v2-stdarg-runtime-source",
            ],
            &[
                "tinycc-0.9.26",
                "tinycc-libtcc1",
                "tcc-musl-prep-libtcc1",
                "musl-pass2-libc",
                "musl-pass2-crt1",
                "musl-pass2-headers",
            ],
            &[
                "tcc-musl-v2-configured-source".to_string(),
                "tcc-musl-v2".to_string(),
                "tcc-musl-v2-alias".to_string(),
                "tcc-musl-v2-libtcc1".to_string(),
                "tcc-musl-v2-source-tree".to_string(),
                "tcc-musl-v2-header-tree".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-v2-materialization:tinycc26",
                &tinycc,
                TINYCC_FINAL_STAGE_ID,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TCC_MUSL_V2_SMOKE_STAGE_ID,
            &[TCC_MUSL_V2_BUILD_STAGE_ID, MUSL_PASS2_BUILD_STAGE_ID],
            &[],
            &[
                "tcc-musl-v2",
                "tcc-musl-v2-libtcc1",
                "musl-pass2-libc",
                "musl-pass2-crt1",
                "musl-pass2-headers",
            ],
            &[
                "tcc-musl-v2-positive-object".to_string(),
                "tcc-musl-v2-positive-binary".to_string(),
                "tcc-musl-v2-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-v2-smoke:tcc",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TCC_MUSL_V2_EXECUTION_STAGE_ID,
            &[TCC_MUSL_V2_SMOKE_STAGE_ID],
            &[],
            &["tcc-musl-v2-positive-binary"],
            &["tcc-musl-v2-positive-execution".to_string()],
            vec![authorization(
                "exec:tcc-musl-v2-execution:positive",
                &positive,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
                crate::stagex_tcc_musl_v2::POSITIVE_BINARY_BLAKE3,
            )],
        ),
    ]
}

fn tcc_selfhost_stage_plans(tinycc_root: &Path, v2_root: &Path, selfhost_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc = tinycc_root.join("output/bin/tcc-0.9.26");
    let v2 = v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let selfhost = selfhost_root.join("output/bin/tcc-0.9.27-musl-selfhost");
    vec![
        mes_stage_plan(
            TCC_SELFHOST_BUILD_STAGE_ID,
            &[
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
                TCC_MUSL_V2_EXECUTION_STAGE_ID,
                TINYCC_FINAL_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
                MUSL_PASS2_BUILD_STAGE_ID,
                MUSL_PASS2_SMOKE_STAGE_ID,
            ],
            &["tcc-musl-selfhost-recipe-source"],
            &[
                "tcc-musl-v2",
                "tcc-musl-v2-source-tree",
                "tinycc-0.9.26",
                "tinycc-libc",
                "tinycc-libtcc1",
                "musl-pass2-libc",
            ],
            &[
                "tcc-musl-selfhost".to_string(),
                "tcc-musl-selfhost-alias".to_string(),
                "tcc-musl-selfhost-libtcc".to_string(),
                "tcc-musl-selfhost-main-object".to_string(),
                "tcc-musl-selfhost-patched-source".to_string(),
                "tcc-musl-selfhost-object-tree".to_string(),
            ],
            vec![
                authorization(
                    "exec:tcc-musl-selfhost-build:tcc-v2",
                    &v2,
                    TCC_MUSL_V2_BUILD_STAGE_ID,
                    crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
                ),
                authorization(
                    "exec:tcc-musl-selfhost-build:tinycc26",
                    &tinycc,
                    TINYCC_FINAL_STAGE_ID,
                    crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
                ),
            ],
        ),
        mes_stage_plan(
            TCC_SELFHOST_SMOKE_STAGE_ID,
            &[TCC_SELFHOST_BUILD_STAGE_ID],
            &[],
            &["tcc-musl-selfhost"],
            &["tcc-musl-selfhost-smoke-object".to_string()],
            vec![authorization(
                "exec:tcc-musl-selfhost-smoke:tcc",
                &selfhost,
                TCC_SELFHOST_BUILD_STAGE_ID,
                crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
            )],
        ),
    ]
}

fn musl_native_stage_plans(tcc_musl_v2_root: &Path, selfhost_root: &Path, native_root: &Path) -> Vec<StagexStagePlan> {
    let predecessor = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let selfhost = selfhost_root.join("output/bin/tcc-0.9.27-musl-selfhost");
    let smoke_binary = native_root.join("smoke/native-musl-smoke");
    let build_outputs = std::iter::once("musl-native-configured-source".to_string())
        .chain(
            crate::stagex_musl_native::EXPECTED_OUTPUTS
                .iter()
                .filter(|output| {
                    output.artifact_id != "musl-native-smoke-binary"
                        && output.artifact_id != "musl-native-validation-receipt"
                })
                .map(|output| output.artifact_id.to_string()),
        )
        .collect::<Vec<_>>();
    let smoke_digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == "musl-native-smoke-binary")
        .expect("native musl smoke output has a fixed identity")
        .digest_blake3;
    vec![
        mes_stage_plan(
            MUSL_NATIVE_BUILD_STAGE_ID,
            &[
                TCC_SELFHOST_BUILD_STAGE_ID,
                TCC_SELFHOST_SMOKE_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &["musl-1.1.24-source", "musl-native-recipe-source"],
            &["tcc-musl-selfhost", "tcc-musl-v2"],
            &build_outputs,
            vec![
                authorization(
                    "exec:musl-native-build:selfhost",
                    &selfhost,
                    TCC_SELFHOST_BUILD_STAGE_ID,
                    crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
                ),
                authorization(
                    "exec:musl-native-build:predecessor",
                    &predecessor,
                    TCC_MUSL_V2_BUILD_STAGE_ID,
                    crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
                ),
            ],
        ),
        mes_stage_plan(
            MUSL_NATIVE_SMOKE_STAGE_ID,
            &[MUSL_NATIVE_BUILD_STAGE_ID, TCC_SELFHOST_BUILD_STAGE_ID],
            &[],
            &[
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
                "tcc-musl-selfhost",
            ],
            &["musl-native-smoke-binary".to_string()],
            vec![authorization(
                "exec:musl-native-smoke:selfhost",
                &selfhost,
                TCC_SELFHOST_BUILD_STAGE_ID,
                crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(
            MUSL_NATIVE_EXECUTION_STAGE_ID,
            &[MUSL_NATIVE_SMOKE_STAGE_ID, TCC_MUSL_V2_BUILD_STAGE_ID],
            &[],
            &["musl-native-smoke-binary", "tcc-musl-v2"],
            &["musl-native-validation-receipt".to_string()],
            vec![
                authorization(
                    "exec:musl-native-execution:binary",
                    &smoke_binary,
                    MUSL_NATIVE_SMOKE_STAGE_ID,
                    smoke_digest,
                ),
                authorization(
                    "exec:musl-native-execution:predecessor",
                    &predecessor,
                    TCC_MUSL_V2_BUILD_STAGE_ID,
                    crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
                ),
            ],
        ),
    ]
}

fn m4_stage_plans(tcc_musl_v2_root: &Path, musl_native_root: &Path, m4_root: &Path) -> Vec<StagexStagePlan> {
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let m4 = m4_root.join("output/bin/m4");
    let smoke_outputs = crate::stagex_m4::M4_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id != "m4-1.4.7")
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), crate::stagex_m4::M4_EXPECTED_OUTPUTS.len() - 1);
    vec![
        mes_stage_plan(
            M4_BUILD_STAGE_ID,
            &[
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                MUSL_NATIVE_EXECUTION_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_m4::M4_SOURCE_ARTIFACT_ID,
                crate::stagex_m4::M4_RECIPE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
            ],
            &["m4-configured-source".to_string(), "m4-1.4.7".to_string()],
            vec![authorization(
                "exec:m4-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(M4_SMOKE_STAGE_ID, &[M4_BUILD_STAGE_ID], &[], &["m4-1.4.7"], &smoke_outputs, vec![
            authorization("exec:m4-smoke:m4", &m4, M4_BUILD_STAGE_ID, crate::stagex_m4::M4_FINAL_BLAKE3),
        ]),
    ]
}

fn diffutils_stage_plans(
    tcc_musl_v2_root: &Path,
    musl_native_root: &Path,
    diffutils_root: &Path,
) -> Vec<StagexStagePlan> {
    const DIFFUTILS_BINARY_OUTPUT_COUNT: usize = 2;
    const DIFFUTILS_SMOKE_OUTPUT_COUNT: usize =
        crate::stagex_diffutils::DIFFUTILS_EXPECTED_OUTPUTS.len() - DIFFUTILS_BINARY_OUTPUT_COUNT;
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let diff = diffutils_root.join("output/bin/diff");
    let cmp = diffutils_root.join("output/bin/cmp");
    let smoke_outputs = crate::stagex_diffutils::DIFFUTILS_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id != "diffutils-diff-2.7" && output.artifact_id != "diffutils-cmp-2.7")
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), DIFFUTILS_SMOKE_OUTPUT_COUNT);
    vec![
        mes_stage_plan(
            DIFFUTILS_BUILD_STAGE_ID,
            &[
                M4_SMOKE_STAGE_ID,
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                MUSL_NATIVE_EXECUTION_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_diffutils::DIFFUTILS_SOURCE_ARTIFACT_ID,
                crate::stagex_diffutils::DIFFUTILS_RECIPE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
            ],
            &[
                "diffutils-configured-source".to_string(),
                "diffutils-diff-2.7".to_string(),
                "diffutils-cmp-2.7".to_string(),
            ],
            vec![authorization(
                "exec:diffutils-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(
            DIFFUTILS_SMOKE_STAGE_ID,
            &[DIFFUTILS_BUILD_STAGE_ID],
            &[],
            &["diffutils-diff-2.7", "diffutils-cmp-2.7"],
            &smoke_outputs,
            vec![
                authorization(
                    "exec:diffutils-smoke:diff",
                    &diff,
                    DIFFUTILS_BUILD_STAGE_ID,
                    crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
                ),
                authorization(
                    "exec:diffutils-smoke:cmp",
                    &cmp,
                    DIFFUTILS_BUILD_STAGE_ID,
                    crate::stagex_diffutils::DIFFUTILS_CMP_BLAKE3,
                ),
            ],
        ),
    ]
}

fn grep_stage_plans(tcc_musl_v2_root: &Path, musl_native_root: &Path, grep_root: &Path) -> Vec<StagexStagePlan> {
    const GREP_BUILD_OUTPUT_COUNT: usize = 4;
    const GREP_SMOKE_OUTPUT_COUNT: usize = crate::stagex_grep::GREP_EXPECTED_OUTPUTS.len() - GREP_BUILD_OUTPUT_COUNT;
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let grep = grep_root.join("output/bin/grep");
    let smoke_outputs = crate::stagex_grep::GREP_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id.ends_with("-observation"))
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), GREP_SMOKE_OUTPUT_COUNT);
    vec![
        mes_stage_plan(
            GREP_BUILD_STAGE_ID,
            &[
                DIFFUTILS_SMOKE_STAGE_ID,
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                MUSL_NATIVE_EXECUTION_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_grep::GREP_SOURCE_ARTIFACT_ID,
                crate::stagex_grep::GREP_RECIPE_ARTIFACT_ID,
                crate::stagex_grep::GREP_RUNNER_SOURCE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
            ],
            &[
                "grep-configured-source".to_string(),
                "grep-2.4-runner".to_string(),
                "egrep-2.4-runner".to_string(),
                "fgrep-2.4-runner".to_string(),
                "grep-2.4-bridge-script".to_string(),
            ],
            vec![authorization(
                "exec:grep-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(GREP_SMOKE_STAGE_ID, &[GREP_BUILD_STAGE_ID], &[], &["grep-2.4-runner"], &smoke_outputs, vec![
            authorization("exec:grep-smoke:grep", &grep, GREP_BUILD_STAGE_ID, crate::stagex_grep::GREP_RUNNER_BLAKE3),
        ]),
    ]
}

fn gawk_stage_plans(tcc_musl_v2_root: &Path, musl_native_root: &Path, gawk_root: &Path) -> Vec<StagexStagePlan> {
    const GAWK_BUILD_OUTPUT_COUNT: usize = 2;
    const GAWK_SMOKE_OUTPUT_COUNT: usize = crate::stagex_gawk::GAWK_EXPECTED_OUTPUTS.len() - GAWK_BUILD_OUTPUT_COUNT;
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let gawk = gawk_root.join("output/bin/gawk");
    let smoke_outputs = crate::stagex_gawk::GAWK_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id.ends_with("-observation"))
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), GAWK_SMOKE_OUTPUT_COUNT);
    vec![
        mes_stage_plan(
            GAWK_BUILD_STAGE_ID,
            &[
                GREP_SMOKE_STAGE_ID,
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                MUSL_NATIVE_EXECUTION_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_gawk::GAWK_SOURCE_ARTIFACT_ID,
                crate::stagex_gawk::GAWK_RECIPE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
            ],
            &[
                "gawk-configured-source".to_string(),
                "gawk-3.0.4".to_string(),
                "awk-3.0.4-alias".to_string(),
            ],
            vec![authorization(
                "exec:gawk-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(GAWK_SMOKE_STAGE_ID, &[GAWK_BUILD_STAGE_ID], &[], &["gawk-3.0.4"], &smoke_outputs, vec![
            authorization("exec:gawk-smoke:gawk", &gawk, GAWK_BUILD_STAGE_ID, crate::stagex_gawk::GAWK_BINARY_BLAKE3),
        ]),
    ]
}

fn bison_stage_plans(
    tcc_musl_v2_root: &Path,
    musl_native_root: &Path,
    m4_root: &Path,
    bison_root: &Path,
) -> Vec<StagexStagePlan> {
    const BISON_BUILD_OUTPUT_COUNT: usize = 2;
    const BISON_SMOKE_OUTPUT_COUNT: usize =
        crate::stagex_bison::BISON_EXPECTED_OUTPUTS.len() - BISON_BUILD_OUTPUT_COUNT;
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let m4 = m4_root.join("output/bin/m4");
    let bison = bison_root.join("output/bin/bison");
    let smoke_outputs = crate::stagex_bison::BISON_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| !matches!(output.artifact_id, "bison-2.3" | "bison-2.3-runtime-data"))
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), BISON_SMOKE_OUTPUT_COUNT);
    vec![
        mes_stage_plan(
            BISON_BUILD_STAGE_ID,
            &[
                GAWK_SMOKE_STAGE_ID,
                M4_BUILD_STAGE_ID,
                M4_SMOKE_STAGE_ID,
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_bison::BISON_SOURCE_ARTIFACT_ID,
                crate::stagex_bison::BISON_RECIPE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
                "m4-1.4.7",
            ],
            &[
                "bison-configured-source".to_string(),
                "bison-2.3".to_string(),
                "bison-2.3-runtime-data".to_string(),
            ],
            vec![authorization(
                "exec:bison-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(
            BISON_SMOKE_STAGE_ID,
            &[BISON_BUILD_STAGE_ID, M4_BUILD_STAGE_ID],
            &[],
            &["bison-2.3", "bison-2.3-runtime-data", "m4-1.4.7"],
            &smoke_outputs,
            vec![
                authorization(
                    "exec:bison-smoke:bison",
                    &bison,
                    BISON_BUILD_STAGE_ID,
                    crate::stagex_bison::BISON_BINARY_BLAKE3,
                ),
                authorization("exec:bison-smoke:m4", &m4, M4_BUILD_STAGE_ID, crate::stagex_m4::M4_FINAL_BLAKE3),
            ],
        ),
    ]
}

fn flex_stage_plans(
    tcc_musl_v2_root: &Path,
    musl_native_root: &Path,
    m4_root: &Path,
    flex_root: &Path,
) -> Vec<StagexStagePlan> {
    const FLEX_BUILD_OUTPUT_COUNT: usize = 4;
    const FLEX_SMOKE_OUTPUT_COUNT: usize = crate::stagex_flex::FLEX_EXPECTED_OUTPUTS.len() - FLEX_BUILD_OUTPUT_COUNT;
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let m4 = m4_root.join("output/bin/m4");
    let flex = flex_root.join("output/bin/flex");
    let consumer = flex_root.join("libfl-consumer");
    let smoke_outputs = crate::stagex_flex::FLEX_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| {
            !matches!(output.artifact_id, "flex-2.6.4" | "flex-libfl" | "flex-cxx-header" | "flex-libfl-consumer")
        })
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert_eq!(smoke_outputs.len(), FLEX_SMOKE_OUTPUT_COUNT);
    vec![
        mes_stage_plan(
            FLEX_BUILD_STAGE_ID,
            &[
                BISON_SMOKE_STAGE_ID,
                M4_BUILD_STAGE_ID,
                M4_SMOKE_STAGE_ID,
                MUSL_NATIVE_BUILD_STAGE_ID,
                MUSL_NATIVE_SMOKE_STAGE_ID,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                TCC_MUSL_V2_SMOKE_STAGE_ID,
            ],
            &[
                crate::stagex_flex::FLEX_SOURCE_ARTIFACT_ID,
                crate::stagex_flex::FLEX_RECIPE_ARTIFACT_ID,
            ],
            &[
                "tcc-musl-v2",
                "musl-native-libc",
                "musl-native-crt1",
                "musl-native-headers",
                "m4-1.4.7",
            ],
            &[
                "flex-configured-source".to_string(),
                "flex-2.6.4".to_string(),
                "flex-libfl".to_string(),
                "flex-cxx-header".to_string(),
                "flex-libfl-consumer".to_string(),
            ],
            vec![authorization(
                "exec:flex-materialization:tcc-musl-v2",
                &compiler,
                TCC_MUSL_V2_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            )],
        ),
        mes_stage_plan(
            FLEX_SMOKE_STAGE_ID,
            &[FLEX_BUILD_STAGE_ID, M4_BUILD_STAGE_ID],
            &[],
            &["flex-2.6.4", "flex-libfl", "flex-libfl-consumer", "m4-1.4.7"],
            &smoke_outputs,
            vec![
                authorization(
                    "exec:flex-smoke:flex",
                    &flex,
                    FLEX_BUILD_STAGE_ID,
                    crate::stagex_flex::FLEX_BINARY_BLAKE3,
                ),
                authorization(
                    "exec:flex-smoke:libfl-consumer",
                    &consumer,
                    FLEX_BUILD_STAGE_ID,
                    crate::stagex_flex::FLEX_LIBFL_CONSUMER_BLAKE3,
                ),
                authorization("exec:flex-smoke:m4", &m4, M4_BUILD_STAGE_ID, crate::stagex_m4::M4_FINAL_BLAKE3),
            ],
        ),
    ]
}

fn musl_pass2_stage_plans(tcc_musl_root: &Path) -> Vec<StagexStagePlan> {
    let compiler = tcc_musl_root.join("output/bin/tcc-0.9.27-musl");
    let source_artifact_ids = crate::stagex_musl::pass2_source_artifact_digests()
        .into_iter()
        .map(|(artifact_id, _)| artifact_id)
        .chain(std::iter::once("musl-1.1.24-source"))
        .collect::<Vec<_>>();
    vec![
        mes_stage_plan(
            MUSL_PASS2_BUILD_STAGE_ID,
            &[TCC_MUSL_BUILD_STAGE_ID, TCC_MUSL_SMOKE_STAGE_ID, MUSL_BUILD_STAGE_ID],
            &source_artifact_ids,
            &["tcc-musl", "tcc-musl-libtcc1", "musl-libc", "musl-crt1", "musl-headers"],
            &[
                "musl-pass2-configured-source".to_string(),
                "musl-pass2-libc".to_string(),
                "musl-pass2-crt1".to_string(),
                "musl-pass2-crti".to_string(),
                "musl-pass2-crtn".to_string(),
                "musl-pass2-libm".to_string(),
                "musl-pass2-librt".to_string(),
                "musl-pass2-libpthread".to_string(),
                "musl-pass2-libcrypt".to_string(),
                "musl-pass2-libutil".to_string(),
                "musl-pass2-libxnet".to_string(),
                "musl-pass2-libresolv".to_string(),
                "musl-pass2-libdl".to_string(),
                "musl-pass2-headers".to_string(),
            ],
            vec![authorization(
                "exec:musl-pass2-materialization:tcc-musl",
                &compiler,
                TCC_MUSL_BUILD_STAGE_ID,
                crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            MUSL_PASS2_SMOKE_STAGE_ID,
            &[MUSL_PASS2_BUILD_STAGE_ID, TCC_MUSL_BUILD_STAGE_ID],
            &[],
            &[
                "musl-pass2-libc",
                "musl-pass2-crt1",
                "musl-pass2-headers",
                "tcc-musl-libtcc1",
            ],
            &[
                "musl-pass2-smoke-binary".to_string(),
                "musl-pass2-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:musl-pass2-smoke:tcc-musl",
                &compiler,
                TCC_MUSL_BUILD_STAGE_ID,
                crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn tcc_musl_stage_plans(prep_root: &Path, tcc_musl_root: &Path) -> Vec<StagexStagePlan> {
    let prep = prep_root.join("output/bin/tcc-musl-prep");
    let compiler = tcc_musl_root.join("output/bin/tcc-0.9.27-musl");
    vec![
        mes_stage_plan(
            TCC_MUSL_BUILD_STAGE_ID,
            &[TCC_MUSL_PREP_BUILD_STAGE_ID, MUSL_BUILD_STAGE_ID, MUSL_SMOKE_STAGE_ID],
            &[
                "tinycc-0.9.27-source",
                "tinycc-0.9.27-patch-source",
                "tcc-musl-recipe-source",
            ],
            &[
                "tcc-musl-prep",
                "tcc-musl-prep-libc",
                "tcc-musl-prep-crt1",
                "tcc-musl-prep-libtcc1",
                "musl-libc",
                "musl-crt1",
                "musl-headers",
            ],
            &[
                "tcc-musl-configured-source".to_string(),
                "tcc-musl".to_string(),
                "tcc-musl-alias".to_string(),
                "tcc-musl-libtcc1".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-materialization:tcc-musl-prep",
                &prep,
                TCC_MUSL_PREP_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TCC_MUSL_SMOKE_STAGE_ID,
            &[TCC_MUSL_BUILD_STAGE_ID, MUSL_BUILD_STAGE_ID],
            &[],
            &["tcc-musl", "tcc-musl-libtcc1", "musl-libc", "musl-crt1"],
            &[
                "tcc-musl-positive-object".to_string(),
                "tcc-musl-positive-binary".to_string(),
                "tcc-musl-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-smoke:tcc",
                &compiler,
                TCC_MUSL_BUILD_STAGE_ID,
                crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn musl_stage_plans(prep_root: &Path) -> Vec<StagexStagePlan> {
    let compiler = prep_root.join("output/bin/tcc-musl-prep");
    let source_artifacts = crate::stagex_musl::source_artifact_digests()
        .into_iter()
        .map(|(artifact_id, _)| artifact_id)
        .collect::<Vec<_>>();
    let build_outputs = std::iter::once("musl-configured-source".to_string())
        .chain(crate::stagex_musl::MUSL_EXPECTED_OUTPUTS.iter().filter_map(|output| {
            if output.artifact_id == "musl-smoke-binary" {
                None
            } else {
                Some(output.artifact_id.to_string())
            }
        }))
        .collect::<Vec<_>>();
    vec![
        mes_stage_plan(
            MUSL_BUILD_STAGE_ID,
            &[TCC_MUSL_PREP_BUILD_STAGE_ID, TCC_MUSL_PREP_SMOKE_STAGE_ID],
            &source_artifacts,
            &[
                "tcc-musl-prep",
                "tcc-musl-prep-libc",
                "tcc-musl-prep-crt1",
                "tcc-musl-prep-libtcc1",
                "tcc-musl-prep-headers",
            ],
            &build_outputs,
            vec![authorization(
                "exec:musl-materialization:tcc-musl-prep",
                &compiler,
                TCC_MUSL_PREP_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            MUSL_SMOKE_STAGE_ID,
            &[MUSL_BUILD_STAGE_ID, TCC_MUSL_PREP_BUILD_STAGE_ID],
            &[],
            &["musl-libc", "musl-crt1", "musl-headers", "tcc-musl-prep-libtcc1"],
            &["musl-smoke-binary".to_string(), "musl-negative-observation".to_string()],
            vec![authorization(
                "exec:musl-smoke:tcc-musl-prep",
                &compiler,
                TCC_MUSL_PREP_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn tcc_musl_prep_stage_plans(tinycc27_root: &Path, prep_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let prep = prep_root.join("output/bin/tcc-musl-prep");
    vec![
        mes_stage_plan(
            TCC_MUSL_PREP_BUILD_STAGE_ID,
            &[
                BASH_BUILD_STAGE_ID,
                BASH_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[
                "tinycc-0.9.27-source",
                "tinycc-0.9.27-patch-source",
                "tcc-musl-prep-recipe-source",
            ],
            &[
                "bash-smoke-output",
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
            ],
            &[
                "tcc-musl-prep-configured-source".to_string(),
                "tcc-musl-prep".to_string(),
                "tcc-musl-prep-alias".to_string(),
                "tcc-musl-prep-libc".to_string(),
                "tcc-musl-prep-crt1".to_string(),
                "tcc-musl-prep-libtcc1".to_string(),
                "tcc-musl-prep-headers".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-prep-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TCC_MUSL_PREP_SMOKE_STAGE_ID,
            &[TCC_MUSL_PREP_BUILD_STAGE_ID],
            &[],
            &["tcc-musl-prep", "tcc-musl-prep-headers"],
            &[
                "tcc-musl-prep-positive-object".to_string(),
                "tcc-musl-prep-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:tcc-musl-prep-smoke:tcc",
                &prep,
                TCC_MUSL_PREP_BUILD_STAGE_ID,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn bash_stage_plans(tinycc27_root: &Path, bash_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let bash = bash_root.join("output/bin/bash");
    let source_artifacts = crate::stagex_bash::source_artifact_digests()
        .into_iter()
        .map(|(artifact_id, _)| artifact_id)
        .collect::<Vec<_>>();
    vec![
        mes_stage_plan(
            BASH_BUILD_STAGE_ID,
            &[
                OYACC_BUILD_STAGE_ID,
                OYACC_SMOKE_STAGE_ID,
                COREUTILS_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &source_artifacts,
            &[
                "oyacc-smoke-output",
                "coreutils-smoke",
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
            ],
            &[
                "bash-configured-source".to_string(),
                "bash-2.05b".to_string(),
                "sh-2.05b".to_string(),
            ],
            vec![authorization(
                "exec:bash-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            BASH_SMOKE_STAGE_ID,
            &[BASH_BUILD_STAGE_ID],
            &[],
            &["bash-2.05b"],
            &["bash-smoke-output".to_string(), "bash-negative-observation".to_string()],
            vec![authorization(
                "exec:bash-smoke:bash",
                &bash,
                BASH_BUILD_STAGE_ID,
                crate::stagex_bash::BASH_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn coreutils_stage_plans(tinycc27_root: &Path, patch_root: &Path, coreutils_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let patch = patch_root.join("output/bin/patch");
    let binary_root = coreutils_root.join("output/bin");
    let build_outputs = crate::stagex_coreutils::COREUTILS_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id != "coreutils-smoke")
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    let smoke_authorizations = coreutils_smoke_executables()
        .into_iter()
        .map(|(name, digest)| {
            authorization(
                &format!("exec:coreutils-smoke:{name}"),
                &binary_root.join(name),
                COREUTILS_BUILD_STAGE_ID,
                digest,
            )
        })
        .collect::<Vec<_>>();
    vec![
        mes_stage_plan(
            COREUTILS_PATCH_STAGE_ID,
            &[BZIP2_BUILD_STAGE_ID, BZIP2_SMOKE_STAGE_ID, GNU_PATCH_BUILD_STAGE_ID],
            &[
                "coreutils-5.0-source",
                "coreutils-5.0-config-header-source",
                "coreutils-modechange-patch-source",
                "coreutils-mbstate-patch-source",
                "coreutils-ls-strcmp-patch-source",
                "coreutils-touch-getdate-patch-source",
                "coreutils-tac-uint64-patch-source",
                "coreutils-expr-strcmp-patch-source",
                "coreutils-sort-locale-patch-source",
                "coreutils-hash-tcc-patch-source",
            ],
            &["gnu-patch-2.5.9", "bzip2-smoke"],
            &["coreutils-configured-source".to_string()],
            vec![authorization(
                "exec:coreutils-source-patch:patch",
                &patch,
                GNU_PATCH_BUILD_STAGE_ID,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            COREUTILS_BUILD_STAGE_ID,
            &[
                COREUTILS_PATCH_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[],
            &[
                "coreutils-configured-source",
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
            ],
            &build_outputs,
            vec![authorization(
                "exec:coreutils-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            COREUTILS_SMOKE_STAGE_ID,
            &[COREUTILS_BUILD_STAGE_ID],
            &[],
            &[
                "coreutils-mkdir",
                "coreutils-echo",
                "coreutils-cp",
                "coreutils-cat",
                "coreutils-test",
            ],
            &[
                "coreutils-smoke".to_string(),
                "coreutils-negative-observation".to_string(),
            ],
            smoke_authorizations,
        ),
    ]
}

fn coreutils_smoke_executables() -> [(&'static str, &'static str); 5] {
    [
        ("mkdir", "b94fc26390ef29d55b71e608b800fa0451ff477b33c15ff3d2be3c8a2a06621b"),
        ("echo", "3ce3cfec07fda35f97485b30fddd8a14f4cb04bd8427d2aa3acc4e94fd650a3f"),
        ("cp", "8c11ead9af84f2371178ce892afbac6daa416c445f2575a2300b2b5ffadf17f3"),
        ("cat", "3d9de9e7f81612a74e75312f22d038b665e69c5d9590662c293843e9711cf020"),
        ("test", "87baf4f61820a2861bbb1a08b3bd9b4c7a5b8baed66ddb0e152d7a981ae27669"),
    ]
}

fn sed_stage_plans(tinycc27_root: &Path, sed_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let sed = sed_root.join("output/bin/sed");
    vec![
        mes_stage_plan(
            SED_BUILD_STAGE_ID,
            &[
                TAR_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[crate::stagex_sed::SED_SOURCE_ARTIFACT_ID],
            &[
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
                "tar-smoke",
            ],
            &["sed-4.0.9".to_string()],
            vec![authorization(
                "exec:sed-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            SED_SMOKE_STAGE_ID,
            &[SED_BUILD_STAGE_ID],
            &[],
            &["sed-4.0.9"],
            &[
                "sed-smoke".to_string(),
                "sed-version-observation".to_string(),
                "sed-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:sed-smoke:sed",
                &sed,
                SED_BUILD_STAGE_ID,
                crate::stagex_sed::SED_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn tar_stage_plans(tinycc27_root: &Path, tar_root: &Path) -> Vec<StagexStagePlan> {
    let tinycc27 = tinycc27_root.join("output/bin/tcc");
    let tar = tar_root.join("output/bin/tar");
    vec![
        mes_stage_plan(
            TAR_BUILD_STAGE_ID,
            &[
                GZIP_SMOKE_STAGE_ID,
                TINYCC27_BUILD_STAGE_ID,
                TINYCC27_RUNTIME_STAGE_ID,
                TINYCC_RUNTIME_STAGE_ID,
            ],
            &[
                crate::stagex_tar::TAR_SOURCE_ARTIFACT_ID,
                crate::stagex_tar::TAR_GETDATE_SOURCE_ARTIFACT_ID,
            ],
            &[
                "tinycc-0.9.27",
                "tinycc27-libc",
                "tinycc-crt1",
                "tinycc-libtcc1",
                "gzip-smoke",
            ],
            &["tar-1.12".to_string()],
            vec![authorization(
                "exec:tar-materialization:tinycc27",
                &tinycc27,
                TINYCC27_BUILD_STAGE_ID,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
            )],
        ),
        mes_stage_plan(
            TAR_SMOKE_STAGE_ID,
            &[TAR_BUILD_STAGE_ID],
            &[],
            &["tar-1.12"],
            &[
                "tar-smoke".to_string(),
                "tar-version-observation".to_string(),
                "tar-negative-observation".to_string(),
            ],
            vec![authorization(
                "exec:tar-smoke:tar",
                &tar,
                TAR_BUILD_STAGE_ID,
                crate::stagex_tar::TAR_FINAL_BLAKE3,
            )],
        ),
    ]
}

fn mes_runtime_stage_plans(stage0_root: &Path, mes_root: &Path) -> Vec<StagexStagePlan> {
    let runtime_outputs = crate::stagex_mes_lib::MES_RUNTIME_EXPECTED_OUTPUTS
        .iter()
        .filter(|expected| expected.artifact_id != "mes-m2")
        .map(|expected| expected.artifact_id.to_string())
        .collect::<Vec<_>>();
    vec![
        mes_stage_plan(
            MES_NYACC_STAGE_ID,
            &[MES_M2_STAGE_ID],
            &["nyacc-1.00.2-source"],
            &["mes-m2"],
            &["nyacc-generated-tables".to_string()],
            vec![mes_m2_authorization(MES_NYACC_STAGE_ID, mes_root)],
        ),
        mes_stage_plan(
            MES_RUNTIME_STAGE_ID,
            &[
                MES_NYACC_STAGE_ID,
                MES_M2_STAGE_ID,
                "stage0-m1",
                "stage0-full-blood-elf",
            ],
            &["mes-0.27.1-source", "nyacc-1.00.2-source"],
            &["nyacc-generated-tables", "mes-m2", "stage0-m1", "stage0-full-blood-elf"],
            &runtime_outputs,
            vec![
                mes_m2_authorization(MES_RUNTIME_STAGE_ID, mes_root),
                stage0_authorization(MES_RUNTIME_STAGE_ID, stage0_root, "stage0-m1"),
                stage0_full_authorization(MES_RUNTIME_STAGE_ID, stage0_root, "stage0-full-blood-elf"),
            ],
        ),
    ]
}

fn mes_stage_plan(
    stage_id: &str,
    predecessors: &[&str],
    sources: &[&str],
    inputs: &[&str],
    outputs: &[String],
    executable_authorizations: Vec<StagexExecutableAuthorization>,
) -> StagexStagePlan {
    StagexStagePlan {
        id: stage_id.to_string(),
        immediate_predecessor_stage_ids: predecessors.iter().map(|value| (*value).to_string()).collect(),
        source_artifact_ids: sources.iter().map(|value| (*value).to_string()).collect(),
        input_artifact_ids: inputs.iter().map(|value| (*value).to_string()).collect(),
        output_artifact_ids: outputs.to_vec(),
        executable_authorizations,
        timeout_ms_max: MES_STAGE_TIMEOUT_MS,
        output_bytes_max: MES_OUTPUT_MAX_BYTES,
        parallel_job_count_max: STAGE_JOB_COUNT,
    }
}

fn mes_m2_authorization(stage_id: &str, mes_root: &Path) -> StagexExecutableAuthorization {
    authorization(
        &format!("exec:{stage_id}:mes-m2"),
        &mes_root.join("bin/mes-m2"),
        MES_M2_STAGE_ID,
        crate::stagex_mes::MES_M2_BLAKE3,
    )
}

fn planned_binutils_executables(binutils_root: &Path) -> Vec<PlannedExecutable> {
    const FIXED_PLANNED_EXECUTABLE_COUNT: usize = 6;
    let mut planned = vec![
        PlannedExecutable {
            authorization_id: "exec:binutils-helper:sed-launcher".to_string(),
            source_stage_id: BINUTILS_HELPER_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join("stagex-sed-bridge-launcher"),
            digest_hex: crate::stagex_binutils::SED_BRIDGE_LAUNCHER_BLAKE3.to_string(),
        },
        PlannedExecutable {
            authorization_id: "exec:binutils-helper:configure-utility".to_string(),
            source_stage_id: BINUTILS_HELPER_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join("stagex-configure-utility"),
            digest_hex: crate::stagex_binutils::CONFIGURE_UTILITY_BLAKE3.to_string(),
        },
        PlannedExecutable {
            authorization_id: "exec:binutils-helper:ylwrap-runner".to_string(),
            source_stage_id: BINUTILS_HELPER_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join("stagex-ylwrap-sed-runner"),
            digest_hex: crate::stagex_binutils::YLWRAP_SED_RUNNER_BLAKE3.to_string(),
        },
        PlannedExecutable {
            authorization_id: "exec:binutils-helper:elf-canonicalizer".to_string(),
            source_stage_id: BINUTILS_CANONICALIZER_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join("stagex-elf-local-symbol-canonicalizer"),
            digest_hex: crate::stagex_binutils::ELF_SYMBOL_CANONICALIZER_BLAKE3.to_string(),
        },
        PlannedExecutable {
            authorization_id: "exec:binutils-component:bfd-chew".to_string(),
            source_stage_id: BINUTILS_GENERATOR_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join("source/bfd/doc/chew"),
            digest_hex: crate::stagex_binutils::BINUTILS_BFD_CHEW_BLAKE3.to_string(),
        },
        PlannedExecutable {
            authorization_id: "exec:binutils-smoke:positive".to_string(),
            source_stage_id: BINUTILS_INSTALL_STAGE_ID.to_string(),
            path: binutils_root.join("binutils-runtime-smoke/positive"),
            digest_hex: crate::stagex_binutils::BINUTILS_POSITIVE_SMOKE_BLAKE3.to_string(),
        },
    ];
    planned.extend(crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLES.iter().map(|(label, relative, digest)| {
        PlannedExecutable {
            authorization_id: format!("exec:binutils-generated:{label}"),
            source_stage_id: BINUTILS_GENERATOR_BUILD_STAGE_ID.to_string(),
            path: binutils_root.join(relative),
            digest_hex: (*digest).to_string(),
        }
    }));
    let install_bin = binutils_root.join("install-destdir/mantle/stagex/binutils-probe-output/bin");
    planned.extend(crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS.iter().map(|(tool, digest)| PlannedExecutable {
        authorization_id: format!("exec:binutils-smoke:{tool}"),
        source_stage_id: BINUTILS_INSTALL_STAGE_ID.to_string(),
        path: install_bin.join(tool),
        digest_hex: (*digest).to_string(),
    }));
    assert_eq!(
        planned.len(),
        crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS.len()
            + crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLE_COUNT
            + FIXED_PLANNED_EXECUTABLE_COUNT
    );
    assert!(planned.iter().all(|entry| entry.path.is_absolute()));
    planned
}

fn additional_planned_binutils_predecessor_executables(
    transition_root: &Path,
    existing: &[PlannedExecutable],
) -> Vec<PlannedExecutable> {
    let paths = BinutilsStagePaths::from_transition_root(transition_root);
    let additional = binutils_component_authorizations(&paths, "binutils-policy")
        .into_iter()
        .filter(|authorization| {
            !existing.iter().any(|planned| {
                planned.path == Path::new(&authorization.absolute_path)
                    && planned.digest_hex == authorization.digest_blake3
            })
        })
        .map(|authorization| PlannedExecutable {
            authorization_id: authorization.id,
            source_stage_id: authorization.source_stage_id,
            path: PathBuf::from(authorization.absolute_path),
            digest_hex: authorization.digest_blake3,
        })
        .collect::<Vec<_>>();
    assert!(!additional.is_empty());
    assert!(additional.iter().all(|entry| entry.path.is_absolute()));
    additional
}

fn additional_mes_stage0_planned_executables(stage0_root: &Path) -> Vec<PlannedExecutable> {
    ["stage0-full-mkdir", "stage0-full-cp"]
        .into_iter()
        .map(|artifact_id| {
            let expected = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
                .iter()
                .find(|expected| expected.artifact_id == artifact_id)
                .expect("Mes Stage0 tool has expected identity");
            PlannedExecutable {
                authorization_id: format!("exec:{artifact_id}"),
                source_stage_id: "stage0-full-extras".to_string(),
                path: stage0_root.join(expected.relative_path),
                digest_hex: expected.digest_blake3.to_string(),
            }
        })
        .collect()
}

fn mes_m2_stage_plans(stage0_root: &Path, mes_root: &Path) -> Vec<StagexStagePlan> {
    vec![
        stage0_stage_plan(
            MES_M2_STAGE_ID,
            &[
                "stage0-full-after-smoke",
                "stage0-kaem",
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-full-extras",
            ],
            &["stage0-source-bundle", "mes-0.27.1-source", "nyacc-1.00.2-source"],
            &[
                "stage0-kaem",
                "stage0-m1",
                "stage0-hex2",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-full-mkdir",
                "stage0-full-cp",
            ],
            &["mes-m2", "mes"],
            vec![
                stage0_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-kaem"),
                stage0_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-m1"),
                stage0_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-hex2"),
                stage0_full_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-full-m2-planet"),
                stage0_full_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-full-blood-elf"),
                stage0_full_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-full-mkdir"),
                stage0_full_authorization(MES_M2_STAGE_ID, stage0_root, "stage0-full-cp"),
            ],
        ),
        stage0_stage_plan(
            MES_M2_SMOKE_STAGE_ID,
            &[MES_M2_STAGE_ID],
            &[],
            &["mes-m2"],
            &["mes-m2-smoke-observation"],
            vec![authorization(
                "exec:mes-m2-smoke:mes-m2",
                &mes_root.join("bin/mes-m2"),
                MES_M2_STAGE_ID,
                crate::stagex_mes::MES_M2_BLAKE3,
            )],
        ),
    ]
}

fn stage0_full_stage_plans(root: &Path) -> Vec<StagexStagePlan> {
    vec![
        stage0_stage_plan(
            "stage0-full-m2-mesoplanet",
            &[
                "stage0-kaem",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &["stage0-amd64-answers", "stage0-after-script"],
            &[
                "stage0-kaem",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &["stage0-full-m2-mesoplanet"],
            vec![
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-kaem"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-m2"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-blood-elf-bootstrap"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-m1"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-blood-elf",
            &[
                "stage0-full-m2-mesoplanet",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &[],
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-full-blood-elf"],
            vec![
                stage0_authorization("stage0-full-blood-elf", root, "stage0-m2"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-blood-elf-bootstrap"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-m1"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-get-machine",
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &[],
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &["stage0-full-get-machine"],
            vec![
                stage0_authorization("stage0-full-get-machine", root, "stage0-m2"),
                stage0_full_authorization("stage0-full-get-machine", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-get-machine", root, "stage0-m1"),
                stage0_authorization("stage0-full-get-machine", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-m2-planet",
            &[
                "stage0-full-get-machine",
                "stage0-full-blood-elf",
                "stage0-m2",
                "stage0-m1",
                "stage0-hex2",
            ],
            &[],
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &["stage0-full-m2-planet"],
            vec![
                stage0_authorization("stage0-full-m2-planet", root, "stage0-m2"),
                stage0_full_authorization("stage0-full-m2-planet", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-m2-planet", root, "stage0-m1"),
                stage0_authorization("stage0-full-m2-planet", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-extras",
            &[
                "stage0-full-m2-planet",
                "stage0-full-m2-mesoplanet",
                "stage0-full-blood-elf",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[],
            &[
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[
                "stage0-full-sha256sum",
                "stage0-full-match",
                "stage0-full-mkdir",
                "stage0-full-untar",
                "stage0-full-ungz",
                "stage0-full-unbz2",
                "stage0-full-unxz",
                "stage0-full-catm",
                "stage0-full-cp",
                "stage0-full-chmod",
                "stage0-full-rm",
                "stage0-full-replace",
                "stage0-full-wrap",
            ],
            vec![
                stage0_authorization("stage0-full-extras", root, "stage0-kaem"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-m2-mesoplanet"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-m2-planet"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-extras", root, "stage0-m1"),
                stage0_authorization("stage0-full-extras", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-sha256-verify",
            &[
                "stage0-full-extras",
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-full-get-machine",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[],
            &[
                "stage0-full-sha256sum",
                "stage0-full-blood-elf",
                "stage0-full-catm",
                "stage0-full-chmod",
                "stage0-full-cp",
                "stage0-full-get-machine",
                "stage0-hex2",
                "stage0-kaem",
                "stage0-m1",
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-match",
                "stage0-full-mkdir",
                "stage0-full-replace",
                "stage0-full-rm",
                "stage0-full-ungz",
                "stage0-full-unbz2",
                "stage0-full-unxz",
                "stage0-full-untar",
            ],
            &["stage0-full-sha256-verification"],
            vec![stage0_full_authorization(
                "stage0-full-sha256-verify",
                root,
                "stage0-full-sha256sum",
            )],
        ),
        stage0_stage_plan(
            "stage0-full-after-smoke",
            &["stage0-full-sha256-verify", "stage0-kaem"],
            &[],
            &["stage0-full-sha256-verification", "stage0-kaem"],
            &["stage0-full-after-observation"],
            vec![stage0_authorization("stage0-full-after-smoke", root, "stage0-kaem")],
        ),
    ]
}

fn stage0_full_authorization(stage_id: &str, root: &Path, artifact_id: &str) -> StagexExecutableAuthorization {
    let expected = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
        .expect("full Stage0 plan artifact has expected executable identity");
    let source_stage_id = match artifact_id {
        "stage0-full-m2-mesoplanet" => "stage0-full-m2-mesoplanet",
        "stage0-full-m2-planet" => "stage0-full-m2-planet",
        "stage0-full-blood-elf" => "stage0-full-blood-elf",
        _ => "stage0-full-extras",
    };
    authorization(
        &format!("exec:{stage_id}:{artifact_id}"),
        &root.join(expected.relative_path),
        source_stage_id,
        expected.digest_blake3,
    )
}

fn stage0_mini_stage_plans(root: &Path) -> Vec<StagexStagePlan> {
    let source_artifacts = [
        "stage0-source-bundle",
        "stage0-posix-amd64-source",
        "bootstrap-seeds-source",
        "m2-planet-source",
        "m2libc-source",
        "mescc-tools-source",
        "mescc-tools-extra-source",
        "m2-mesoplanet-source",
    ];
    vec![
        stage0_stage_plan(
            "stage0-hex1",
            &[KAEM_MATERIALIZATION_STAGE_ID, HEX0_REPRODUCTION_STAGE_ID],
            &source_artifacts,
            &["kaem-0", "hex0-reproduced"],
            &["stage0-hex0-copy", "stage0-hex1"],
            vec![
                authorization(
                    "exec:stage0-hex1:kaem-0",
                    &root.parent().expect("Stage0 root has parent").join("kaem-0"),
                    KAEM_MATERIALIZATION_STAGE_ID,
                    KAEM_OUTPUT_BLAKE3,
                ),
                stage0_authorization("stage0-hex1", root, "stage0-hex0-copy"),
            ],
        ),
        stage0_stage("stage0-hex2-bootstrap", &["stage0-hex1"], &["stage0-hex1"], &["stage0-hex2-bootstrap"], root, &[
            "stage0-hex1",
        ]),
        stage0_stage("stage0-catm", &["stage0-hex2-bootstrap"], &["stage0-hex2-bootstrap"], &["stage0-catm"], root, &[
            "stage0-hex2-bootstrap",
        ]),
        stage0_stage(
            "stage0-m0",
            &["stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m0"],
            root,
            &["stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-cc-amd64",
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-cc-amd64"],
            root,
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-m2",
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m2"],
            root,
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-blood-elf-bootstrap",
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
            &["stage0-blood-elf-bootstrap"],
            root,
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-m1-bootstrap",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &["stage0-m1-bootstrap"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
        ),
        stage0_stage(
            "stage0-hex2-intermediate",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &["stage0-hex2-intermediate"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
        ),
        stage0_stage(
            "stage0-m1",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
            &["stage0-m1"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
        ),
        stage0_stage(
            "stage0-hex2",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
            &["stage0-hex2"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
        ),
        stage0_stage(
            "stage0-kaem",
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-kaem"],
            root,
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
        ),
    ]
}

fn stage0_stage(
    stage_id: &str,
    predecessors: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    root: &Path,
    executable_artifact_ids: &[&str],
) -> StagexStagePlan {
    let authorizations = executable_artifact_ids
        .iter()
        .map(|artifact_id| stage0_authorization(stage_id, root, artifact_id))
        .collect();
    stage0_stage_plan(stage_id, predecessors, &[], inputs, outputs, authorizations)
}

fn stage0_stage_plan(
    stage_id: &str,
    predecessors: &[&str],
    sources: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    executable_authorizations: Vec<StagexExecutableAuthorization>,
) -> StagexStagePlan {
    StagexStagePlan {
        id: stage_id.to_string(),
        immediate_predecessor_stage_ids: predecessors.iter().map(|value| (*value).to_string()).collect(),
        source_artifact_ids: sources.iter().map(|value| (*value).to_string()).collect(),
        input_artifact_ids: inputs.iter().map(|value| (*value).to_string()).collect(),
        output_artifact_ids: outputs.iter().map(|value| (*value).to_string()).collect(),
        executable_authorizations,
        timeout_ms_max: STAGE0_STAGE_TIMEOUT_MS,
        output_bytes_max: GENERATED_EXECUTABLE_MAX_BYTES,
        parallel_job_count_max: STAGE_JOB_COUNT,
    }
}

fn stage0_authorization(stage_id: &str, root: &Path, artifact_id: &str) -> StagexExecutableAuthorization {
    let expected = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
        .expect("Stage0 plan artifact has expected executable identity");
    authorization(
        &format!("exec:{stage_id}:{artifact_id}"),
        &root.join(expected.relative_path),
        expected.source_stage_id,
        expected.digest_blake3,
    )
}

fn promotion_stage_ids(plan: &StagexMaterializationPlan) -> Vec<String> {
    let mut source_stage_ids = BTreeSet::new();
    for stage in &plan.stages {
        for authorization in &stage.executable_authorizations {
            if authorization.source_stage_id != STAGEX_SEED_SOURCE_STAGE_ID {
                source_stage_ids.insert(authorization.source_stage_id.clone());
            }
        }
    }
    source_stage_ids.into_iter().collect()
}

fn binutils_stage_plans(transition_root: &Path) -> Vec<StagexStagePlan> {
    let paths = BinutilsStagePaths::from_transition_root(transition_root);
    let source_artifacts = crate::stagex_binutils::source_artifact_digests()
        .into_iter()
        .map(|(artifact_id, _)| artifact_id)
        .collect::<Vec<_>>();
    let canonicalizer_outputs = vec!["binutils-elf-symbol-canonicalizer".to_string()];
    let helper_outputs = vec![
        "binutils-sed-bridge-launcher".to_string(),
        "binutils-configure-utility".to_string(),
        "binutils-ylwrap-runner".to_string(),
    ];
    let generator_outputs = vec![
        "binutils-bfd-chew".to_string(),
        "binutils-configure-probe-executables".to_string(),
    ];
    let mut component_outputs = crate::stagex_binutils::BINUTILS_COMPONENT_ARTIFACT_IDS
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    component_outputs.push("binutils-generated-sources".to_string());
    let mut install_outputs = crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS
        .iter()
        .map(|(tool, _)| format!("binutils-installed-{tool}"))
        .collect::<Vec<_>>();
    install_outputs.push("binutils-positive-smoke-executable".to_string());
    let smoke_outputs = vec!["binutils-runtime-smoke".to_string()];
    assert!(transition_root.is_absolute());
    assert_eq!(install_outputs.len(), crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS.len() + 1);
    vec![
        binutils_canonicalizer_stage(&paths, &source_artifacts, &canonicalizer_outputs),
        binutils_helper_stage(&paths, &source_artifacts, &canonicalizer_outputs, &helper_outputs),
        binutils_generator_stage(&paths, &source_artifacts, &helper_outputs, &generator_outputs),
        binutils_component_stage(&paths, &source_artifacts, &generator_outputs, &component_outputs),
        binutils_install_stage(&paths, &component_outputs, &install_outputs),
        binutils_install_smoke_stage(&paths, &install_outputs, &smoke_outputs),
    ]
}

fn binutils_canonicalizer_stage(
    paths: &BinutilsStagePaths,
    source_artifacts: &[&str],
    canonicalizer_outputs: &[String],
) -> StagexStagePlan {
    let stage = mes_stage_plan(
        BINUTILS_CANONICALIZER_BUILD_STAGE_ID,
        &[
            TCC_MUSL_V2_BUILD_STAGE_ID,
            TCC_MUSL_V2_SMOKE_STAGE_ID,
            MUSL_NATIVE_BUILD_STAGE_ID,
        ],
        source_artifacts,
        &[
            "tcc-musl-v2",
            "musl-native-libc",
            "musl-native-crt1",
            "musl-native-headers",
        ],
        canonicalizer_outputs,
        vec![authorization(
            "exec:binutils-canonicalizer:tcc-musl-v2",
            &paths.tcc,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        )],
    );
    assert_eq!(stage.id, BINUTILS_CANONICALIZER_BUILD_STAGE_ID);
    assert_eq!(stage.executable_authorizations.len(), 1);
    stage
}

fn binutils_helper_stage(
    paths: &BinutilsStagePaths,
    source_artifacts: &[&str],
    canonicalizer_outputs: &[String],
    helper_outputs: &[String],
) -> StagexStagePlan {
    let mut authorizations = vec![
        authorization(
            "exec:binutils-helper:tcc-musl-v2",
            &paths.tcc,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        ),
        authorization(
            "exec:binutils-helper:bash-full",
            &paths.bash,
            BASH_FULL_BUILD_STAGE_ID,
            crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3,
        ),
    ];
    for tool in ["cat", "chmod", "mkdir", "rm", "wc"] {
        authorizations.push(binutils_coreutils_authorization(paths, BINUTILS_HELPER_BUILD_STAGE_ID, tool));
    }
    authorizations.push(binutils_generated_authorization(
        paths,
        BINUTILS_HELPER_BUILD_STAGE_ID,
        BINUTILS_CANONICALIZER_BUILD_STAGE_ID,
        "elf-canonicalizer",
        "stagex-elf-local-symbol-canonicalizer",
        crate::stagex_binutils::ELF_SYMBOL_CANONICALIZER_BLAKE3,
    ));
    let stage = mes_stage_plan(
        BINUTILS_HELPER_BUILD_STAGE_ID,
        &[
            BINUTILS_CANONICALIZER_BUILD_STAGE_ID,
            BASH_FULL_BUILD_STAGE_ID,
            BASH_FULL_SMOKE_STAGE_ID,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            TCC_MUSL_V2_SMOKE_STAGE_ID,
            COREUTILS_BUILD_STAGE_ID,
            COREUTILS_SMOKE_STAGE_ID,
        ],
        source_artifacts,
        &binutils_helper_inputs(canonicalizer_outputs),
        helper_outputs,
        authorizations,
    );
    assert_eq!(stage.id, BINUTILS_HELPER_BUILD_STAGE_ID);
    assert!(!stage.executable_authorizations.is_empty());
    stage
}

fn binutils_helper_inputs(canonicalizer_outputs: &[String]) -> Vec<&str> {
    let mut inputs = canonicalizer_outputs.iter().map(String::as_str).collect::<Vec<_>>();
    inputs.extend([
        "bash-2.05b-full",
        "tcc-musl-v2",
        "coreutils-cat",
        "coreutils-chmod",
        "coreutils-mkdir",
        "coreutils-rm",
        "coreutils-wc",
    ]);
    assert!(!inputs.is_empty());
    assert!(inputs.iter().all(|input| !input.is_empty()));
    inputs
}

fn binutils_helper_executable_authorizations(
    paths: &BinutilsStagePaths,
    consumer_stage_id: &str,
) -> Vec<StagexExecutableAuthorization> {
    let authorizations = vec![
        binutils_generated_authorization(
            paths,
            consumer_stage_id,
            BINUTILS_HELPER_BUILD_STAGE_ID,
            "sed-launcher",
            "stagex-sed-bridge-launcher",
            crate::stagex_binutils::SED_BRIDGE_LAUNCHER_BLAKE3,
        ),
        binutils_generated_authorization(
            paths,
            consumer_stage_id,
            BINUTILS_HELPER_BUILD_STAGE_ID,
            "configure-utility",
            "stagex-configure-utility",
            crate::stagex_binutils::CONFIGURE_UTILITY_BLAKE3,
        ),
        binutils_generated_authorization(
            paths,
            consumer_stage_id,
            BINUTILS_HELPER_BUILD_STAGE_ID,
            "ylwrap-runner",
            "stagex-ylwrap-sed-runner",
            crate::stagex_binutils::YLWRAP_SED_RUNNER_BLAKE3,
        ),
        binutils_generated_authorization(
            paths,
            consumer_stage_id,
            BINUTILS_CANONICALIZER_BUILD_STAGE_ID,
            "elf-canonicalizer",
            "stagex-elf-local-symbol-canonicalizer",
            crate::stagex_binutils::ELF_SYMBOL_CANONICALIZER_BLAKE3,
        ),
    ];
    assert!(!authorizations.is_empty());
    assert!(authorizations.iter().all(|authorization| authorization.id.contains(consumer_stage_id)));
    authorizations
}

fn binutils_component_inputs(helper_outputs: &[String]) -> Vec<&str> {
    let mut inputs = helper_outputs.iter().map(String::as_str).collect::<Vec<_>>();
    inputs.extend([
        "binutils-elf-symbol-canonicalizer",
        "bash-2.05b-full",
        "tcc-musl-v2",
        "sed-4.0.9",
        "grep-2.4-runner",
        "diffutils-diff-2.7",
        "diffutils-cmp-2.7",
        "gawk-3.0.4",
        "m4-1.4.7",
        "bison-2.3",
        "flex-2.6.4",
        "make-3.82",
    ]);
    inputs.extend(crate::stagex_coreutils::COREUTILS_EXPECTED_OUTPUTS.iter().map(|expected| expected.artifact_id));
    assert!(!inputs.is_empty());
    assert!(inputs.iter().all(|input| !input.is_empty()));
    inputs
}

fn binutils_generator_stage(
    paths: &BinutilsStagePaths,
    source_artifacts: &[&str],
    helper_outputs: &[String],
    generator_outputs: &[String],
) -> StagexStagePlan {
    let mut authorizations = binutils_component_authorizations(paths, BINUTILS_GENERATOR_BUILD_STAGE_ID);
    authorizations.extend(binutils_helper_executable_authorizations(paths, BINUTILS_GENERATOR_BUILD_STAGE_ID));
    let stage = mes_stage_plan(
        BINUTILS_GENERATOR_BUILD_STAGE_ID,
        &[
            BINUTILS_CANONICALIZER_BUILD_STAGE_ID,
            BINUTILS_HELPER_BUILD_STAGE_ID,
            BASH_FULL_BUILD_STAGE_ID,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            SED_BUILD_STAGE_ID,
            GREP_BUILD_STAGE_ID,
            DIFFUTILS_BUILD_STAGE_ID,
            GAWK_BUILD_STAGE_ID,
            M4_BUILD_STAGE_ID,
            BISON_BUILD_STAGE_ID,
            FLEX_BUILD_STAGE_ID,
            MAKE_BUILD_STAGE_ID,
            COREUTILS_BUILD_STAGE_ID,
            COREUTILS_SMOKE_STAGE_ID,
        ],
        source_artifacts,
        &binutils_component_inputs(helper_outputs),
        generator_outputs,
        authorizations,
    );
    assert_eq!(stage.id, BINUTILS_GENERATOR_BUILD_STAGE_ID);
    assert!(!stage.executable_authorizations.is_empty());
    stage
}

fn binutils_component_stage(
    paths: &BinutilsStagePaths,
    source_artifacts: &[&str],
    generator_outputs: &[String],
    component_outputs: &[String],
) -> StagexStagePlan {
    let mut authorizations = binutils_component_authorizations(paths, BINUTILS_COMPONENT_BUILD_STAGE_ID);
    authorizations.push(binutils_generated_authorization(
        paths,
        BINUTILS_COMPONENT_BUILD_STAGE_ID,
        BINUTILS_GENERATOR_BUILD_STAGE_ID,
        "bfd-chew",
        "source/bfd/doc/chew",
        crate::stagex_binutils::BINUTILS_BFD_CHEW_BLAKE3,
    ));
    authorizations.extend(binutils_helper_executable_authorizations(paths, BINUTILS_COMPONENT_BUILD_STAGE_ID));
    authorizations.extend(binutils_generated_executable_authorizations(paths, BINUTILS_COMPONENT_BUILD_STAGE_ID));
    let generator_inputs = generator_outputs.iter().map(String::as_str).collect::<Vec<_>>();
    let stage = mes_stage_plan(
        BINUTILS_COMPONENT_BUILD_STAGE_ID,
        &[BINUTILS_GENERATOR_BUILD_STAGE_ID],
        source_artifacts,
        &generator_inputs,
        component_outputs,
        authorizations,
    );
    assert_eq!(stage.id, BINUTILS_COMPONENT_BUILD_STAGE_ID);
    assert!(!stage.executable_authorizations.is_empty());
    stage
}

fn binutils_install_stage(
    paths: &BinutilsStagePaths,
    component_outputs: &[String],
    install_outputs: &[String],
) -> StagexStagePlan {
    let mut authorizations = binutils_component_authorizations(paths, BINUTILS_INSTALL_STAGE_ID);
    authorizations.extend(binutils_helper_executable_authorizations(paths, BINUTILS_INSTALL_STAGE_ID));
    let component_inputs = component_outputs.iter().map(String::as_str).collect::<Vec<_>>();
    let stage = mes_stage_plan(
        BINUTILS_INSTALL_STAGE_ID,
        &[BINUTILS_COMPONENT_BUILD_STAGE_ID],
        &[],
        &component_inputs,
        install_outputs,
        authorizations,
    );
    assert_eq!(stage.id, BINUTILS_INSTALL_STAGE_ID);
    assert!(!stage.executable_authorizations.is_empty());
    stage
}

fn binutils_install_smoke_stage(
    paths: &BinutilsStagePaths,
    install_outputs: &[String],
    smoke_outputs: &[String],
) -> StagexStagePlan {
    let install_bin = paths.runtime.join("install-destdir/mantle/stagex/binutils-probe-output/bin");
    let mut authorizations = BINUTILS_INSTALL_SMOKE_EXECUTED_TOOL_NAMES
        .iter()
        .map(|tool| {
            let digest = crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS
                .iter()
                .find(|(required, _)| required == tool)
                .map(|(_, digest)| *digest)
                .expect("executed binutils smoke tool has a required identity");
            authorization(
                &format!("exec:{BINUTILS_INSTALL_SMOKE_STAGE_ID}:{tool}"),
                &install_bin.join(tool),
                BINUTILS_INSTALL_STAGE_ID,
                digest,
            )
        })
        .collect::<Vec<_>>();
    authorizations.push(binutils_generated_authorization(
        paths,
        BINUTILS_INSTALL_SMOKE_STAGE_ID,
        BINUTILS_INSTALL_STAGE_ID,
        "positive",
        "binutils-runtime-smoke/positive",
        crate::stagex_binutils::BINUTILS_POSITIVE_SMOKE_BLAKE3,
    ));
    let install_inputs = install_outputs.iter().map(String::as_str).collect::<Vec<_>>();
    let stage = mes_stage_plan(
        BINUTILS_INSTALL_SMOKE_STAGE_ID,
        &[BINUTILS_INSTALL_STAGE_ID],
        &[],
        &install_inputs,
        smoke_outputs,
        authorizations,
    );
    assert_eq!(stage.id, BINUTILS_INSTALL_SMOKE_STAGE_ID);
    assert!(!stage.executable_authorizations.is_empty());
    stage
}

fn binutils_generated_executable_authorizations(
    paths: &BinutilsStagePaths,
    consumer_stage_id: &str,
) -> Vec<StagexExecutableAuthorization> {
    let authorizations = crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLES
        .iter()
        .map(|(label, relative, digest)| {
            binutils_generated_authorization(
                paths,
                consumer_stage_id,
                BINUTILS_GENERATOR_BUILD_STAGE_ID,
                label,
                relative,
                digest,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(authorizations.len(), crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLE_COUNT);
    assert!(
        authorizations
            .iter()
            .all(|authorization| authorization.source_stage_id == BINUTILS_GENERATOR_BUILD_STAGE_ID)
    );
    authorizations
}

fn binutils_component_authorizations(
    paths: &BinutilsStagePaths,
    consumer_stage_id: &str,
) -> Vec<StagexExecutableAuthorization> {
    let mut authorizations = vec![
        authorization(
            &format!("exec:{consumer_stage_id}:tcc-musl-v2"),
            &paths.tcc,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:bash-full"),
            &paths.bash,
            BASH_FULL_BUILD_STAGE_ID,
            crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:sed"),
            &paths.sed,
            SED_BUILD_STAGE_ID,
            crate::stagex_sed::SED_FINAL_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:grep"),
            &paths.grep,
            GREP_BUILD_STAGE_ID,
            crate::stagex_grep::GREP_RUNNER_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:diff"),
            &paths.diff,
            DIFFUTILS_BUILD_STAGE_ID,
            crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:cmp"),
            &paths.cmp,
            DIFFUTILS_BUILD_STAGE_ID,
            crate::stagex_diffutils::DIFFUTILS_CMP_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:gawk"),
            &paths.gawk,
            GAWK_BUILD_STAGE_ID,
            crate::stagex_gawk::GAWK_BINARY_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:m4"),
            &paths.m4,
            M4_BUILD_STAGE_ID,
            crate::stagex_m4::M4_FINAL_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:bison"),
            &paths.bison,
            BISON_BUILD_STAGE_ID,
            crate::stagex_bison::BISON_BINARY_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:flex"),
            &paths.flex,
            FLEX_BUILD_STAGE_ID,
            crate::stagex_flex::FLEX_BINARY_BLAKE3,
        ),
        authorization(
            &format!("exec:{consumer_stage_id}:make"),
            &paths.make,
            MAKE_BUILD_STAGE_ID,
            crate::stagex_make::MAKE_FINAL_BLAKE3,
        ),
    ];
    authorizations.extend(
        binutils_coreutils_tool_digests()
            .into_iter()
            .map(|(tool, _)| binutils_coreutils_authorization(paths, consumer_stage_id, tool)),
    );
    assert!(!authorizations.is_empty());
    assert!(authorizations.iter().all(|entry| Path::new(&entry.absolute_path).is_absolute()));
    authorizations
}

fn binutils_coreutils_tool_digests() -> Vec<(&'static str, &'static str)> {
    let tools = BINUTILS_EXECUTED_COREUTILS_TOOL_NAMES
        .iter()
        .map(|tool| {
            let expected = crate::stagex_coreutils::COREUTILS_EXPECTED_OUTPUTS
                .iter()
                .find(|expected| expected.artifact_id.strip_prefix("coreutils-") == Some(*tool))
                .expect("executed coreutils tool has a protected identity");
            (*tool, expected.digest_blake3)
        })
        .collect::<Vec<_>>();
    assert_eq!(tools.len(), BINUTILS_EXECUTED_COREUTILS_TOOL_NAMES.len());
    assert!(tools.iter().all(|(tool, digest)| !tool.is_empty() && !digest.is_empty()));
    tools
}

fn binutils_coreutils_authorization(
    paths: &BinutilsStagePaths,
    consumer_stage_id: &str,
    tool: &str,
) -> StagexExecutableAuthorization {
    let digest = binutils_coreutils_tool_digests()
        .into_iter()
        .find(|(name, _)| *name == tool)
        .map(|(_, digest)| digest)
        .expect("binutils coreutils tool has a protected identity");
    authorization(
        &format!("exec:{consumer_stage_id}:coreutils-{tool}"),
        &paths.coreutils_bin.join(tool),
        COREUTILS_BUILD_STAGE_ID,
        digest,
    )
}

fn binutils_generated_authorization(
    paths: &BinutilsStagePaths,
    consumer_stage_id: &str,
    source_stage_id: &str,
    label: &str,
    relative_path: &str,
    digest: &str,
) -> StagexExecutableAuthorization {
    authorization(
        &format!("exec:{consumer_stage_id}:{label}"),
        &paths.runtime.join(relative_path),
        source_stage_id,
        digest,
    )
}

fn bash_full_stage_plans(
    tcc_musl_v2_root: &Path,
    musl_native_root: &Path,
    make_root: &Path,
    bash_root: &Path,
    bash_full_root: &Path,
) -> Vec<StagexStagePlan> {
    let compiler = tcc_musl_v2_root.join("output/bin/tcc-0.9.27-musl-v2");
    let make = make_root.join("output/bin/make");
    let generator = bash_full_root.join("bash-2.05b-full/builtins/stagex-full-mkbuiltins");
    let bash = bash_full_root.join("output/bin/bash-full");
    let smoke_outputs = crate::stagex_bash_full::BASH_FULL_EXPECTED_OUTPUTS
        .iter()
        .filter(|output| output.artifact_id != "bash-2.05b-full")
        .map(|output| output.artifact_id.to_string())
        .collect::<Vec<_>>();
    assert!(musl_native_root.join("output/lib/libc.a").is_absolute());
    assert!(bash_root.join("bash-2.05b").is_absolute());
    vec![
        bash_full_generator_build_stage(&compiler),
        bash_full_generator_run_stage(&generator),
        bash_full_build_stage(&compiler),
        bash_full_smoke_stage(&bash, &make, &smoke_outputs),
    ]
}

fn bash_full_generator_build_stage(compiler: &Path) -> StagexStagePlan {
    let stage = mes_stage_plan(
        BASH_FULL_GENERATOR_BUILD_STAGE_ID,
        &[
            FLEX_SMOKE_STAGE_ID,
            BASH_BUILD_STAGE_ID,
            BASH_SMOKE_STAGE_ID,
            MUSL_NATIVE_BUILD_STAGE_ID,
            MUSL_NATIVE_SMOKE_STAGE_ID,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            TCC_MUSL_V2_SMOKE_STAGE_ID,
        ],
        &[],
        &[
            "bash-configured-source",
            "tcc-musl-v2",
            "musl-native-libc",
            "musl-native-headers",
        ],
        &[crate::stagex_bash_full::BASH_FULL_GENERATOR_ARTIFACT_ID.to_string()],
        vec![authorization(
            "exec:bash-full-generator-materialization:tcc-musl-v2",
            compiler,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        )],
    );
    assert_eq!(stage.id, BASH_FULL_GENERATOR_BUILD_STAGE_ID);
    assert_eq!(stage.executable_authorizations.len(), 1);
    stage
}

fn bash_full_generator_run_stage(generator: &Path) -> StagexStagePlan {
    let stage = mes_stage_plan(
        BASH_FULL_GENERATOR_RUN_STAGE_ID,
        &[BASH_FULL_GENERATOR_BUILD_STAGE_ID, BASH_BUILD_STAGE_ID],
        &[],
        &[
            crate::stagex_bash_full::BASH_FULL_GENERATOR_ARTIFACT_ID,
            "bash-configured-source",
        ],
        &[crate::stagex_bash_full::BASH_FULL_GENERATED_TREE_ARTIFACT_ID.to_string()],
        vec![authorization(
            "exec:bash-full-generator-execution:mkbuiltins",
            generator,
            BASH_FULL_GENERATOR_BUILD_STAGE_ID,
            crate::stagex_bash_full::BASH_FULL_GENERATOR_BLAKE3,
        )],
    );
    assert_eq!(stage.id, BASH_FULL_GENERATOR_RUN_STAGE_ID);
    assert_eq!(stage.executable_authorizations.len(), 1);
    stage
}

fn bash_full_build_stage(compiler: &Path) -> StagexStagePlan {
    let stage = mes_stage_plan(
        BASH_FULL_BUILD_STAGE_ID,
        &[
            BASH_FULL_GENERATOR_RUN_STAGE_ID,
            BASH_BUILD_STAGE_ID,
            MUSL_NATIVE_BUILD_STAGE_ID,
            MUSL_NATIVE_SMOKE_STAGE_ID,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            TCC_MUSL_V2_SMOKE_STAGE_ID,
        ],
        &[],
        &[
            crate::stagex_bash_full::BASH_FULL_GENERATED_TREE_ARTIFACT_ID,
            "bash-configured-source",
            "tcc-musl-v2",
            "musl-native-libc",
            "musl-native-headers",
            "musl-native-crt1",
        ],
        &[
            crate::stagex_bash_full::BASH_FULL_CONFIGURED_SOURCE_ARTIFACT_ID.to_string(),
            "bash-2.05b-full".to_string(),
        ],
        vec![authorization(
            "exec:bash-full-materialization:tcc-musl-v2",
            compiler,
            TCC_MUSL_V2_BUILD_STAGE_ID,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        )],
    );
    assert_eq!(stage.id, BASH_FULL_BUILD_STAGE_ID);
    assert_eq!(stage.executable_authorizations.len(), 1);
    stage
}

fn bash_full_smoke_stage(bash: &Path, make: &Path, smoke_outputs: &[String]) -> StagexStagePlan {
    const EXPECTED_AUTHORIZATION_COUNT: usize = 2;
    let stage = mes_stage_plan(
        BASH_FULL_SMOKE_STAGE_ID,
        &[BASH_FULL_BUILD_STAGE_ID, MAKE_BUILD_STAGE_ID, MAKE_SMOKE_STAGE_ID],
        &[],
        &["bash-2.05b-full", "make-3.82"],
        smoke_outputs,
        vec![
            authorization(
                "exec:bash-full-smoke:bash-full",
                bash,
                BASH_FULL_BUILD_STAGE_ID,
                crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3,
            ),
            authorization(
                "exec:bash-full-smoke:make",
                make,
                MAKE_BUILD_STAGE_ID,
                crate::stagex_make::MAKE_FINAL_BLAKE3,
            ),
        ],
    );
    assert_eq!(stage.id, BASH_FULL_SMOKE_STAGE_ID);
    assert_eq!(stage.executable_authorizations.len(), EXPECTED_AUTHORIZATION_COUNT);
    stage
}

fn transition_stage_plan(
    id: &str,
    predecessors: &[&str],
    sources: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    authorization: StagexExecutableAuthorization,
) -> StagexStagePlan {
    StagexStagePlan {
        id: id.to_string(),
        immediate_predecessor_stage_ids: predecessors.iter().map(|value| (*value).to_string()).collect(),
        source_artifact_ids: sources.iter().map(|value| (*value).to_string()).collect(),
        input_artifact_ids: inputs.iter().map(|value| (*value).to_string()).collect(),
        output_artifact_ids: outputs.iter().map(|value| (*value).to_string()).collect(),
        executable_authorizations: vec![authorization],
        timeout_ms_max: STAGE_TIMEOUT_MS,
        output_bytes_max: GENERATED_EXECUTABLE_MAX_BYTES,
        parallel_job_count_max: STAGE_JOB_COUNT,
    }
}

fn authorization(
    id: &str,
    absolute_path: &Path,
    source_stage_id: &str,
    digest_blake3: &str,
) -> StagexExecutableAuthorization {
    StagexExecutableAuthorization {
        id: id.to_string(),
        absolute_path: absolute_path.display().to_string(),
        role: "stagex-transition-runner".to_string(),
        source_stage_id: source_stage_id.to_string(),
        digest_blake3: digest_blake3.to_string(),
    }
}

fn run_hex0_reproduction(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.seed,
        args: &[staged.hex0_source.clone(), staged.reproduced_hex0.clone()],
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.seed.with_extension("stderr"),
        retry_generated_etxtbsy: false,
    })?;
    let reproduced = read_checked_output(&staged.reproduced_hex0, "reproduced hex0", GENERATED_EXECUTABLE_MAX_BYTES)?;
    validate_bytes_digest("reproduced hex0", &reproduced, HEX0_SEED_BLAKE3)?;
    set_executable_owner_only(&staged.reproduced_hex0)?;
    Ok(())
}

fn run_kaem_materialization(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.reproduced_hex0,
        args: &[staged.kaem_source.clone(), staged.kaem.clone()],
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.reproduced_hex0.with_extension("stderr"),
        retry_generated_etxtbsy: true,
    })?;
    let kaem = read_checked_output(&staged.kaem, "kaem-0", GENERATED_EXECUTABLE_MAX_BYTES)?;
    validate_bytes_digest("kaem-0", &kaem, KAEM_OUTPUT_BLAKE3)?;
    set_executable_owner_only(&staged.kaem)?;
    Ok(())
}

fn run_kaem_smoke(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.kaem,
        args: std::slice::from_ref(&staged.kaem_smoke_script),
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.kaem.with_extension("stderr"),
        retry_generated_etxtbsy: true,
    })?;
    write_file_create_new(&staged.kaem_smoke, KAEM_SMOKE_MARKER)
}

fn run_bounded_command(request: BoundedCommandRequest<'_>) -> Result<ExitStatus, StagexTransitionError> {
    let mut attempt: u32 = 0;
    while attempt < GENERATED_LAUNCH_MAX_ATTEMPTS {
        attempt = attempt.saturating_add(1);
        let result = run_bounded_command_once(&request);
        match result {
            Err(StagexTransitionError::ProcessSpawn { source, .. })
                if request.retry_generated_etxtbsy && source.raw_os_error() == Some(libc::ETXTBSY) =>
            {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            Err(StagexTransitionError::ProcessFailure { exit_code, stderr, .. })
                if request.retry_generated_etxtbsy
                    && exit_code == Some(ETXTBSY_EXIT_CODE)
                    && stderr.contains(ETXTBSY_STDERR_MARKER) =>
            {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            result => return result,
        }
    }
    Err(StagexTransitionError::ProcessSpawn {
        executable: request.executable.to_path_buf(),
        source: io::Error::from_raw_os_error(libc::ETXTBSY),
    })
}

fn run_bounded_command_once(request: &BoundedCommandRequest<'_>) -> Result<ExitStatus, StagexTransitionError> {
    let stderr_file =
        File::create(request.stderr_path).map_err(|source| io_error("creating StageX stderr file", source))?;
    let mut command = Command::new(request.executable);
    command
        .args(request.args)
        .current_dir(request.current_dir)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));
    let mut child = command.spawn().map_err(|source| StagexTransitionError::ProcessSpawn {
        executable: request.executable.to_path_buf(),
        source,
    })?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for StageX process", source))? {
            return classify_process_status(request, status);
        }
        if started.elapsed() >= Duration::from_millis(STAGE_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(StagexTransitionError::ProcessTimeout {
                executable: request.executable.to_path_buf(),
                timeout_ms: STAGE_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(PROCESS_POLL_MS));
    }
}

fn classify_process_status(
    request: &BoundedCommandRequest<'_>,
    status: ExitStatus,
) -> Result<ExitStatus, StagexTransitionError> {
    let stderr = read_bounded_stderr(request.stderr_path)?;
    if status.success() {
        return Ok(status);
    }
    Err(StagexTransitionError::ProcessFailure {
        executable: request.executable.to_path_buf(),
        exit_code: status.code(),
        stderr,
    })
}

fn read_bounded_stderr(path: &Path) -> Result<String, StagexTransitionError> {
    let metadata = fs::metadata(path).map_err(|source| io_error("reading StageX stderr metadata", source))?;
    if metadata.len() > PROCESS_STDERR_MAX_BYTES {
        return Err(StagexTransitionError::SizeLimit {
            role: "process stderr".to_string(),
            actual_bytes: metadata.len(),
            limit_bytes: PROCESS_STDERR_MAX_BYTES,
        });
    }
    let bytes = fs::read(path).map_err(|source| io_error("reading StageX stderr", source))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn promote_executable(
    supervisor: &ProtectedSeccompSupervisor,
    stage_id: &str,
    path: &Path,
    expected_digest: &str,
) -> Result<OutputPromotionRecord, StagexTransitionError> {
    let actual_digest = blake3_file_hex(path).map_err(protected_digest_error)?;
    if actual_digest != expected_digest {
        return Err(StagexTransitionError::DigestMismatch {
            role: stage_id.to_string(),
            expected: expected_digest.to_string(),
            actual: actual_digest,
        });
    }
    supervisor
        .promote_verified_output(stage_id, &["format=raw".to_string()], &[PromotedExecutable {
            path: path.to_path_buf(),
            digest_hex: expected_digest.to_string(),
        }])
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))
}

#[derive(Debug, Clone, Copy)]
struct TransitionAuditReports<'a> {
    stage0_full: Option<&'a crate::stagex_stage0_full::Stage0FullInventoryReport>,
    mes_m2: Option<&'a crate::stagex_mes::MesM2InventoryReport>,
    mes_runtime: Option<&'a crate::stagex_mes_lib::MesRuntimeInventoryReport>,
    tinycc_runtime: Option<&'a crate::stagex_tinycc::TccMesInventoryReport>,
    tinycc27_runtime: Option<&'a crate::stagex_tinycc27::Tinycc27InventoryReport>,
    make_runtime: Option<&'a crate::stagex_make::MakeInventoryReport>,
    gnu_patch_runtime: Option<&'a crate::stagex_gnu_patch::GnuPatchInventoryReport>,
    gzip_runtime: Option<&'a crate::stagex_gzip::GzipInventoryReport>,
    tar_runtime: Option<&'a crate::stagex_tar::TarInventoryReport>,
    sed_runtime: Option<&'a crate::stagex_sed::SedInventoryReport>,
    bzip2_runtime: Option<&'a crate::stagex_bzip2::Bzip2InventoryReport>,
    coreutils_runtime: Option<&'a crate::stagex_coreutils::CoreutilsInventoryReport>,
    oyacc_runtime: Option<&'a crate::stagex_oyacc::OyaccInventoryReport>,
    bash_runtime: Option<&'a crate::stagex_bash::BashInventoryReport>,
    tcc_musl_prep_runtime: Option<&'a crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport>,
    musl_runtime: Option<&'a crate::stagex_musl::MuslInventoryReport>,
    tcc_musl_runtime: Option<&'a crate::stagex_tcc_musl::TccMuslInventoryReport>,
    musl_pass2_runtime: Option<&'a crate::stagex_musl::MuslInventoryReport>,
    tcc_musl_v2_runtime: Option<&'a crate::stagex_tcc_musl_v2::InventoryReport>,
    tcc_selfhost_runtime: Option<&'a crate::stagex_tcc_selfhost::InventoryReport>,
    musl_native_runtime: Option<&'a crate::stagex_musl_native::InventoryReport>,
    m4_runtime: Option<&'a crate::stagex_m4::M4InventoryReport>,
    diffutils_runtime: Option<&'a crate::stagex_diffutils::DiffutilsInventoryReport>,
    grep_runtime: Option<&'a crate::stagex_grep::GrepInventoryReport>,
    gawk_runtime: Option<&'a crate::stagex_gawk::GawkInventoryReport>,
    bison_runtime: Option<&'a crate::stagex_bison::BisonInventoryReport>,
    flex_runtime: Option<&'a crate::stagex_flex::FlexInventoryReport>,
    bash_full_runtime: Option<&'a crate::stagex_bash_full::BashFullInventoryReport>,
    binutils_runtime: Option<&'a crate::stagex_binutils::BinutilsInventoryReport>,
    binutils_event_count: Option<usize>,
}

fn validate_transition_audit(
    staged: &StagedTransitionPaths,
    reports: TransitionAuditReports<'_>,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let TransitionAuditReports {
        stage0_full,
        mes_m2,
        mes_runtime,
        tinycc_runtime,
        tinycc27_runtime,
        make_runtime,
        gnu_patch_runtime,
        gzip_runtime,
        tar_runtime,
        sed_runtime,
        bzip2_runtime,
        coreutils_runtime,
        oyacc_runtime,
        bash_runtime,
        tcc_musl_prep_runtime,
        musl_runtime,
        tcc_musl_runtime,
        musl_pass2_runtime,
        tcc_musl_v2_runtime,
        tcc_selfhost_runtime,
        musl_native_runtime,
        m4_runtime,
        diffutils_runtime,
        grep_runtime,
        gawk_runtime,
        bison_runtime,
        flex_runtime,
        bash_full_runtime,
        binutils_runtime,
        binutils_event_count,
    } = reports;
    let expected = transition_expected_audit_events(staged);
    if events.len() < EXPECTED_AUDIT_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "expected at least {EXPECTED_AUDIT_EVENT_COUNT} events, observed {}",
            events.len()
        )));
    }
    for (event, expected_event) in events.iter().take(EXPECTED_AUDIT_EVENT_COUNT).zip(expected.iter()) {
        validate_audit_event(event, expected_event)?;
    }
    let Some(stage0) = stage0_full else {
        if mes_m2.is_none()
            && mes_runtime.is_none()
            && tinycc_runtime.is_none()
            && tinycc27_runtime.is_none()
            && make_runtime.is_none()
            && gnu_patch_runtime.is_none()
            && gzip_runtime.is_none()
            && tar_runtime.is_none()
            && sed_runtime.is_none()
            && bzip2_runtime.is_none()
            && coreutils_runtime.is_none()
            && oyacc_runtime.is_none()
            && bash_runtime.is_none()
            && tcc_musl_prep_runtime.is_none()
            && musl_runtime.is_none()
            && tcc_musl_runtime.is_none()
            && musl_pass2_runtime.is_none()
            && tcc_musl_v2_runtime.is_none()
            && tcc_selfhost_runtime.is_none()
            && musl_native_runtime.is_none()
            && m4_runtime.is_none()
            && diffutils_runtime.is_none()
            && grep_runtime.is_none()
            && gawk_runtime.is_none()
            && bison_runtime.is_none()
            && flex_runtime.is_none()
            && bash_full_runtime.is_none()
            && binutils_runtime.is_none()
            && binutils_event_count.is_none()
            && events.len() == EXPECTED_AUDIT_EVENT_COUNT
        {
            return Ok(());
        }
        return Err(StagexTransitionError::Audit(format!(
            "expected {EXPECTED_AUDIT_EVENT_COUNT} transition events without Stage0, Mes, or TinyCC; observed {}",
            events.len()
        )));
    };
    const STAGE0_PARENT_EVENT_COUNT: usize = 1;
    let mini_command_count = usize::try_from(stage0.mini.command_count)
        .map_err(|_| StagexTransitionError::Audit("Stage0 command count does not fit usize".to_string()))?;
    let mini_event_count = mini_command_count
        .checked_add(STAGE0_PARENT_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("Stage0 mini event count overflow".to_string()))?;
    let stage0_events = &events[EXPECTED_AUDIT_EVENT_COUNT..];
    if stage0_events.len() < mini_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected at least {mini_event_count} mini Stage0 events, observed {}",
            stage0_events.len()
        )));
    }
    validate_stage0_audit(staged, &stage0.mini, &stage0_events[..mini_event_count])?;
    let after_mini = &stage0_events[mini_event_count..];
    let (before_binutils, binutils_events) =
        split_binutils_audit_suffix(binutils_runtime, binutils_event_count, after_mini)?;
    let (before_bash_full, bash_full_events) = split_bash_full_audit_suffix(bash_full_runtime, before_binutils)?;
    let (before_flex, flex_events) = split_flex_audit_suffix(flex_runtime, before_bash_full)?;
    let (before_bison, bison_events) = split_bison_audit_suffix(bison_runtime, before_flex)?;
    let (before_gawk, gawk_events) = split_gawk_audit_suffix(gawk_runtime, before_bison)?;
    let (before_grep, grep_events) = split_grep_audit_suffix(grep_runtime, before_gawk)?;
    let (before_diffutils, diffutils_events) = split_diffutils_audit_suffix(diffutils_runtime, before_grep)?;
    let (before_m4, m4_events) = split_m4_audit_suffix(m4_runtime, before_diffutils)?;
    let (before_musl_native, musl_native_events) = split_musl_native_audit_suffix(musl_native_runtime, before_m4)?;
    let (before_tcc_selfhost, tcc_selfhost_events) =
        split_tcc_selfhost_audit_suffix(tcc_selfhost_runtime, before_musl_native)?;
    let (before_tcc_musl_v2, tcc_musl_v2_events) =
        split_tcc_musl_v2_audit_suffix(tcc_musl_v2_runtime, before_tcc_selfhost)?;
    let (before_musl_pass2, musl_pass2_events) = split_musl_audit_suffix(musl_pass2_runtime, before_tcc_musl_v2)?;
    let (before_tcc_musl, tcc_musl_events) = split_tcc_musl_audit_suffix(tcc_musl_runtime, before_musl_pass2)?;
    let (before_musl, musl_events) = split_musl_audit_suffix(musl_runtime, before_tcc_musl)?;
    let (before_tcc_musl_prep, tcc_musl_prep_events) =
        split_tcc_musl_prep_audit_suffix(tcc_musl_prep_runtime, before_musl)?;
    let (before_bash, bash_events) = split_bash_audit_suffix(bash_runtime, before_tcc_musl_prep)?;
    let (before_oyacc, oyacc_events) = split_oyacc_audit_suffix(oyacc_runtime, before_bash)?;
    let (before_coreutils, coreutils_events) = split_coreutils_audit_suffix(coreutils_runtime, before_oyacc)?;
    let (before_bzip2, bzip2_events) = split_bzip2_audit_suffix(bzip2_runtime, before_coreutils)?;
    let (before_sed, sed_events) = split_sed_audit_suffix(sed_runtime, before_bzip2)?;
    let (before_tar, tar_events) = split_tar_audit_suffix(tar_runtime, before_sed)?;
    let (before_gzip, gzip_events) = split_gzip_audit_suffix(gzip_runtime, before_tar)?;
    let (before_gnu_patch, gnu_patch_events) = split_gnu_patch_audit_suffix(gnu_patch_runtime, before_gzip)?;
    let (before_make, make_events) = split_make_audit_suffix(make_runtime, before_gnu_patch)?;
    let predecessor_result = match (mes_m2, mes_runtime, tinycc_runtime, tinycc27_runtime) {
        (Some(mes_report), Some(runtime_report), Some(tinycc_report), tinycc27_report) => {
            validate_mes_and_tinycc_suffix(
                stage0,
                mes_report,
                runtime_report,
                tinycc_report,
                tinycc27_report,
                before_make,
            )
        }
        (Some(mes_report), Some(runtime_report), None, None) => {
            let runtime_event_count = mes_runtime_event_count(runtime_report)?;
            let runtime_start = before_make.len().checked_sub(runtime_event_count).ok_or_else(|| {
                StagexTransitionError::Audit(format!(
                    "expected {runtime_event_count} trailing Mes runtime events, observed {}",
                    before_make.len()
                ))
            })?;
            let before_runtime = &before_make[..runtime_start];
            let mes_event_count = mes_m2_event_count(mes_report)?;
            let mes_start = before_runtime
                .len()
                .checked_sub(mes_event_count)
                .ok_or_else(|| StagexTransitionError::Audit("Mes event suffix exceeds audit".to_string()))?;
            validate_stage0_full_audit(stage0, &before_runtime[..mes_start])?;
            validate_mes_m2_audit(stage0, mes_report, &before_runtime[mes_start..])?;
            validate_mes_runtime_audit(stage0, mes_report, runtime_report, &before_make[runtime_start..])
        }
        (Some(mes_report), None, None, None) => {
            let mes_event_count = mes_m2_event_count(mes_report)?;
            let mes_start = before_make
                .len()
                .checked_sub(mes_event_count)
                .ok_or_else(|| StagexTransitionError::Audit("Mes event suffix exceeds audit".to_string()))?;
            validate_stage0_full_audit(stage0, &before_make[..mes_start])?;
            validate_mes_m2_audit(stage0, mes_report, &before_make[mes_start..])
        }
        (None, None, None, None) => validate_stage0_full_audit(stage0, before_make),
        _ => Err(StagexTransitionError::Audit(
            "Mes, TinyCC, or TinyCC 0.9.27 report exists without its predecessor report".to_string(),
        )),
    };
    predecessor_result?;
    match (make_runtime, tinycc27_runtime) {
        (Some(make), Some(tinycc27)) => validate_make_audit(tinycc27, make, make_events)?,
        (None, _) if make_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU Make report or events exist without TinyCC 0.9.27 authority".to_string(),
            ));
        }
    }
    match (gnu_patch_runtime, tinycc27_runtime, make_runtime) {
        (Some(patch), Some(tinycc27), Some(_)) => validate_gnu_patch_audit(tinycc27, patch, gnu_patch_events)?,
        (None, _, _) if gnu_patch_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU patch report or events exist without TinyCC 0.9.27 and GNU Make authority".to_string(),
            ));
        }
    }
    match (gzip_runtime, tinycc27_runtime, gnu_patch_runtime) {
        (Some(gzip), Some(tinycc27), Some(_)) => validate_gzip_audit(tinycc27, gzip, gzip_events)?,
        (None, _, _) if gzip_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "gzip report or events exist without TinyCC 0.9.27 and GNU patch authority".to_string(),
            ));
        }
    }
    match (tar_runtime, tinycc27_runtime, gzip_runtime) {
        (Some(tar), Some(tinycc27), Some(_)) => validate_tar_audit(tinycc27, tar, tar_events)?,
        (None, _, _) if tar_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU tar report or events exist without TinyCC 0.9.27 and gzip authority".to_string(),
            ));
        }
    }
    match (sed_runtime, tinycc27_runtime, tar_runtime) {
        (Some(sed), Some(tinycc27), Some(_)) => validate_sed_audit(tinycc27, sed, sed_events)?,
        (None, _, _) if sed_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU sed report or events exist without TinyCC 0.9.27 and GNU tar authority".to_string(),
            ));
        }
    }
    match (bzip2_runtime, tinycc27_runtime, sed_runtime) {
        (Some(bzip2), Some(tinycc27), Some(sed)) => validate_bzip2_audit(tinycc27, sed, bzip2, bzip2_events)?,
        (None, _, _) if bzip2_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "bzip2 report or events exist without TinyCC 0.9.27 and GNU sed authority".to_string(),
            ));
        }
    }
    match (coreutils_runtime, tinycc27_runtime, gnu_patch_runtime, bzip2_runtime) {
        (Some(coreutils), Some(tinycc27), Some(patch), Some(_)) => {
            validate_coreutils_audit(tinycc27, patch, coreutils, coreutils_events)?;
        }
        (None, _, _, _) if coreutils_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "coreutils report or events exist without TinyCC 0.9.27, GNU patch, and bzip2 authority".to_string(),
            ));
        }
    }
    match (oyacc_runtime, tinycc27_runtime, gnu_patch_runtime, coreutils_runtime) {
        (Some(oyacc), Some(tinycc27), Some(patch), Some(_)) => {
            validate_oyacc_audit(tinycc27, patch, oyacc, oyacc_events)?;
        }
        (None, _, _, _) if oyacc_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "oyacc report or events exist without TinyCC 0.9.27, GNU patch, and coreutils authority".to_string(),
            ));
        }
    }
    match (bash_runtime, tinycc27_runtime, oyacc_runtime) {
        (Some(bash), Some(tinycc27), Some(_)) => validate_bash_audit(tinycc27, bash, bash_events)?,
        (None, _, _) if bash_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "bash report or events exist without TinyCC 0.9.27 and oyacc authority".to_string(),
            ));
        }
    }
    match (tcc_musl_prep_runtime, tinycc27_runtime, bash_runtime) {
        (Some(prep), Some(tinycc27), Some(_)) => {
            validate_tcc_musl_prep_audit(tinycc27, prep, tcc_musl_prep_events)?;
        }
        (None, _, _) if tcc_musl_prep_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "TinyCC musl-prep report or events exist without TinyCC 0.9.27 and bash authority".to_string(),
            ));
        }
    }
    match (musl_runtime, tcc_musl_prep_runtime) {
        (Some(musl), Some(prep)) => validate_musl_audit(prep, musl, musl_events)?,
        (None, _) if musl_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "musl report or events exist without TinyCC musl-prep authority".to_string(),
            ));
        }
    }
    match (tcc_musl_runtime, tcc_musl_prep_runtime, musl_runtime) {
        (Some(tcc_musl), Some(prep), Some(_)) => validate_tcc_musl_audit(prep, tcc_musl, tcc_musl_events)?,
        (None, _, _) if tcc_musl_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "musl-linked TinyCC report or events exist without musl-prep and musl authority".to_string(),
            ));
        }
    }
    match (musl_pass2_runtime, tcc_musl_runtime) {
        (Some(musl_pass2), Some(tcc_musl)) => validate_musl_pass2_audit(tcc_musl, musl_pass2, musl_pass2_events)?,
        (None, _) if musl_pass2_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "second-musl report or events exist without musl-linked TinyCC authority".to_string(),
            ));
        }
    }
    match (tcc_musl_v2_runtime, tinycc_runtime) {
        (Some(v2), Some(tinycc)) => validate_tcc_musl_v2_audit(tinycc, v2, tcc_musl_v2_events)?,
        (None, _) if tcc_musl_v2_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "TinyCC musl-v2 report or events exist without TinyCC 0.9.26 authority".to_string(),
            ));
        }
    }
    match (tcc_selfhost_runtime, tcc_musl_v2_runtime, tinycc_runtime) {
        (Some(selfhost), Some(v2), Some(tinycc)) => {
            validate_tcc_selfhost_audit(v2, tinycc, selfhost, tcc_selfhost_events)?;
        }
        (None, _, _) if tcc_selfhost_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "TinyCC self-host report or events exist without TinyCC musl-v2 authority".to_string(),
            ));
        }
    }
    match (musl_native_runtime, tcc_selfhost_runtime, tcc_musl_v2_runtime) {
        (Some(native), Some(selfhost), Some(v2)) => {
            validate_musl_native_audit(selfhost, v2, native, musl_native_events)?;
        }
        (None, _, _) if musl_native_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "native musl report or events exist without self-hosted TinyCC and TinyCC musl-v2 authority"
                    .to_string(),
            ));
        }
    }
    match (m4_runtime, tcc_musl_v2_runtime, musl_native_runtime) {
        (Some(m4), Some(v2), Some(_)) => validate_m4_audit(v2, m4, m4_events)?,
        (None, _, _) if m4_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU M4 report or events exist without TinyCC musl-v2 and native musl authority".to_string(),
            ));
        }
    }
    match (diffutils_runtime, tcc_musl_v2_runtime, m4_runtime) {
        (Some(diffutils), Some(v2), Some(_)) => validate_diffutils_audit(v2, diffutils, diffutils_events)?,
        (None, _, _) if diffutils_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU diffutils report or events exist without TinyCC musl-v2 and protected GNU M4 completion"
                    .to_string(),
            ));
        }
    }
    match (grep_runtime, tcc_musl_v2_runtime, musl_native_runtime, diffutils_runtime) {
        (Some(grep), Some(v2), Some(_), Some(_)) => validate_grep_audit(v2, grep, grep_events)?,
        (None, _, _, _) if grep_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU grep report or events exist without TinyCC musl-v2, native musl, and protected GNU diffutils completion"
                    .to_string(),
            ));
        }
    }
    match (gawk_runtime, tcc_musl_v2_runtime, musl_native_runtime, grep_runtime) {
        (Some(gawk), Some(v2), Some(_), Some(_)) => validate_gawk_audit(v2, gawk, gawk_events)?,
        (None, _, _, _) if gawk_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU Gawk report or events exist without TinyCC musl-v2, native musl, and protected GNU grep completion"
                    .to_string(),
            ));
        }
    }
    match (bison_runtime, tcc_musl_v2_runtime, musl_native_runtime, m4_runtime, gawk_runtime) {
        (Some(bison), Some(v2), Some(_), Some(m4), Some(_)) => {
            validate_bison_audit(v2, m4, bison, bison_events)?;
        }
        (None, _, _, _, _) if bison_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU Bison report or events exist without TinyCC musl-v2, native musl, protected M4, and GNU Gawk completion"
                    .to_string(),
            ));
        }
    }
    match (flex_runtime, tcc_musl_v2_runtime, musl_native_runtime, m4_runtime, bison_runtime) {
        (Some(flex), Some(v2), Some(_), Some(m4), Some(_)) => validate_flex_audit(v2, m4, flex, flex_events)?,
        (None, _, _, _, _) if flex_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "GNU Flex report or events exist without TinyCC musl-v2, native musl, protected M4, and GNU Bison completion"
                    .to_string(),
            ));
        }
    }
    match (bash_full_runtime, tcc_musl_v2_runtime, musl_native_runtime, make_runtime, flex_runtime) {
        (Some(bash_full), Some(v2), Some(_), Some(make), Some(_)) => {
            validate_bash_full_audit(v2, make, bash_full, bash_full_events)?;
        }
        (None, _, _, _, _) if bash_full_events.is_empty() => {}
        _ => {
            return Err(StagexTransitionError::Audit(
                "full Bash report or events exist without TinyCC musl-v2, native musl, protected GNU Make, and GNU Flex completion"
                    .to_string(),
            ));
        }
    }
    match (binutils_runtime, bash_full_runtime) {
        (Some(binutils), Some(_)) => validate_binutils_audit(binutils, binutils_events),
        (None, _) if binutils_events.is_empty() => Ok(()),
        _ => Err(StagexTransitionError::Audit(
            "GNU binutils report or events exist without protected full Bash completion".to_string(),
        )),
    }
}

fn split_binutils_audit_suffix<'a>(
    report: Option<&crate::stagex_binutils::BinutilsInventoryReport>,
    observed_event_count: Option<usize>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        if observed_event_count.is_some() {
            return Err(StagexTransitionError::Audit(
                "GNU binutils event count exists without a runtime report".to_string(),
            ));
        }
        return Ok((events, &[]));
    };
    let event_count = observed_event_count
        .ok_or_else(|| StagexTransitionError::Audit("GNU binutils runtime report lacks an event count".to_string()))?;
    if !binutils_event_count_within_bounds(report.protected_exec_event_count_bounds, event_count) {
        return Err(StagexTransitionError::Audit(format!(
            "GNU binutils event count is outside the accepted closure: observed {event_count}"
        )));
    }
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing GNU binutils events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn binutils_event_count_within_bounds(count_bounds: [u32; 2], observed_count: usize) -> bool {
    let bounds = count_bounds.map(|count| usize::try_from(count).expect("bounded binutils event count fits usize"));
    let result = observed_count >= bounds[0] && observed_count <= bounds[1];
    assert_eq!(bounds.len(), count_bounds.len());
    assert!(bounds[0] > 0 && bounds[0] <= bounds[1]);
    result
}

fn validate_binutils_audit(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const EXPECTED_CONFIGURE_CLASS_COUNT: u32 = 9;
    const EXPECTED_COMPONENT_COUNT: u32 = 8;
    const EXPECTED_INSTALLED_TOOL_COUNT: u32 = 11;
    const EXPECTED_ARCHIVE_COUNT: u32 = 4;
    const EXPECTED_SED_INVOCATION_COUNTS: [u32; 3] = [4_891, 4_892, 4_893];
    let reported_event_count_bounds = report
        .protected_exec_event_count_bounds
        .map(|count| usize::try_from(count).expect("bounded binutils event count fits usize"));
    let sed_derived_event_count_bounds = binutils_event_count_bounds_from_sed_invocations(report.sed_invocation_count)?;
    if reported_event_count_bounds != crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS
        || !binutils_event_count_within_bounds(report.protected_exec_event_count_bounds, events.len())
        || events.len() < sed_derived_event_count_bounds[0]
        || events.len() > sed_derived_event_count_bounds[1]
    {
        return Err(StagexTransitionError::Audit(format!(
            "GNU binutils event count is not closed: bounds {:?}, sed bounds {:?}, observed {}",
            crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS,
            sed_derived_event_count_bounds,
            events.len()
        )));
    }
    if report.configure_class_count != EXPECTED_CONFIGURE_CLASS_COUNT
        || report.component_count != EXPECTED_COMPONENT_COUNT
        || report.installed_tool_count != EXPECTED_INSTALLED_TOOL_COUNT
        || report.archive_count != EXPECTED_ARCHIVE_COUNT
        || !EXPECTED_SED_INVOCATION_COUNTS.contains(&report.sed_invocation_count)
    {
        return Err(StagexTransitionError::Audit("GNU binutils report counts were substituted".to_string()));
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU binutils report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    validate_binutils_event_authorities(report, events)?;
    validate_binutils_critical_event_counts(report, events)?;
    validate_binutils_event_order(report, events)?;
    assert!(binutils_event_count_within_bounds(report.protected_exec_event_count_bounds, events.len()));
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn validate_binutils_event_authorities(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected = binutils_expected_exec_identities(report)?;
    for event in events {
        let identity = (event.resolved_host_path.clone(), event.digest_hex.clone());
        let valid_metadata = event.policy_decision == "allowed"
            && event.phase == "protected"
            && event.inventory_entry_id.as_deref().is_some_and(|id| id.starts_with("planned:"));
        if !valid_metadata || !expected.contains(&identity) {
            return Err(StagexTransitionError::Audit(format!(
                "unexpected GNU binutils protected exec identity: {} {}",
                event.resolved_host_path.display(),
                event.digest_hex
            )));
        }
    }
    assert!(!expected.is_empty());
    assert!(events.iter().all(|event| event.resolved_host_path.is_absolute()));
    Ok(())
}

fn binutils_expected_exec_identities(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
) -> Result<BTreeSet<(PathBuf, String)>, StagexTransitionError> {
    let binutils_stage = report
        .runtime_root
        .parent()
        .ok_or_else(|| StagexTransitionError::Audit("GNU binutils runtime root has no stage parent".to_string()))?;
    let transition_root = binutils_stage
        .parent()
        .ok_or_else(|| StagexTransitionError::Audit("GNU binutils stage has no transition parent".to_string()))?;
    if binutils_stage.file_name().and_then(|name| name.to_str()) != Some("binutils-stage") {
        return Err(StagexTransitionError::Audit("GNU binutils runtime root is outside binutils-stage".to_string()));
    }
    let paths = BinutilsStagePaths::from_transition_root(transition_root);
    let mut expected = planned_binutils_executables(&report.runtime_root)
        .into_iter()
        .map(|planned| (planned.path, planned.digest_hex))
        .collect::<BTreeSet<_>>();
    expected.extend(
        binutils_component_authorizations(&paths, "binutils-audit")
            .into_iter()
            .map(|authorization| (PathBuf::from(authorization.absolute_path), authorization.digest_blake3)),
    );
    assert!(report.runtime_root.is_absolute());
    assert!(expected.len() >= crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_UNIQUE_IDENTITY_COUNT);
    Ok(expected)
}

fn binutils_event_count_bounds_from_sed_invocations(
    sed_invocation_count: u32,
) -> Result<[usize; 2], StagexTransitionError> {
    const LOWER_SED_INVOCATION_COUNT: u32 = 4_891;
    const MIDDLE_SED_INVOCATION_COUNT: u32 = 4_892;
    const UPPER_SED_INVOCATION_COUNT: u32 = 4_893;
    const LOWER_SED_EVENT_COUNT_BOUNDS: [usize; 2] = [73_991, 74_045];
    const MIDDLE_SED_EVENT_COUNT_BOUNDS: [usize; 2] = [74_008, 74_066];
    const UPPER_SED_EVENT_COUNT_BOUNDS: [usize; 2] = [74_010, 74_057];
    match sed_invocation_count {
        LOWER_SED_INVOCATION_COUNT => Ok(LOWER_SED_EVENT_COUNT_BOUNDS),
        MIDDLE_SED_INVOCATION_COUNT => Ok(MIDDLE_SED_EVENT_COUNT_BOUNDS),
        UPPER_SED_INVOCATION_COUNT => Ok(UPPER_SED_EVENT_COUNT_BOUNDS),
        other => Err(StagexTransitionError::Audit(format!(
            "GNU binutils sed invocation count is outside the closed set: {other}"
        ))),
    }
}

fn validate_binutils_critical_event_counts(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let mut counts = BTreeMap::new();
    for event in events {
        let identity = (event.resolved_host_path.clone(), event.digest_hex.clone());
        let count = counts.entry(identity).or_insert(0_u32);
        *count = count
            .checked_add(1)
            .ok_or_else(|| StagexTransitionError::Audit("GNU binutils identity count overflow".to_string()))?;
    }
    for ((label, relative, digest), (count_label, expected_count)) in
        crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLES
            .iter()
            .zip(crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLE_EVENT_COUNTS.iter())
    {
        if label != count_label {
            return Err(StagexTransitionError::Audit("GNU binutils generated count labels drifted".to_string()));
        }
        require_binutils_identity_count(&counts, &report.runtime_root.join(relative), digest, *expected_count)?;
    }
    for (relative, digest, expected_count) in crate::stagex_binutils::BINUTILS_FIXED_EXECUTABLE_EVENT_COUNTS {
        require_binutils_identity_count(&counts, &report.runtime_root.join(relative), digest, expected_count)?;
    }
    validate_binutils_sed_derived_identity_counts(report, &counts)?;
    require_binutils_predecessor_identities(report, &counts)?;
    if counts.len() != crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_UNIQUE_IDENTITY_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU binutils unique identity count is not closed: expected {}, observed {}",
            crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_UNIQUE_IDENTITY_COUNT,
            counts.len()
        )));
    }
    assert!(counts.values().all(|count| *count > 0));
    assert_eq!(
        events.len(),
        counts.values().map(|count| usize::try_from(*count).unwrap_or_default()).sum::<usize>()
    );
    Ok(())
}

fn validate_binutils_sed_derived_identity_counts(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    counts: &BTreeMap<(PathBuf, String), u32>,
) -> Result<(), StagexTransitionError> {
    const CONFIGURE_UTILITY_EVENT_OFFSET: u32 = 21;
    const SED_LAUNCHER_EVENT_OFFSET: u32 = 10;
    let configure_utility_count = report
        .sed_invocation_count
        .checked_add(CONFIGURE_UTILITY_EVENT_OFFSET)
        .ok_or_else(|| StagexTransitionError::Audit("configure utility event count overflow".to_string()))?;
    let sed_launcher_count = report
        .sed_invocation_count
        .checked_add(SED_LAUNCHER_EVENT_OFFSET)
        .ok_or_else(|| StagexTransitionError::Audit("sed launcher event count overflow".to_string()))?;
    require_binutils_identity_count(
        counts,
        &report.runtime_root.join("stagex-configure-utility"),
        crate::stagex_binutils::CONFIGURE_UTILITY_BLAKE3,
        configure_utility_count,
    )?;
    require_binutils_identity_count(
        counts,
        &report.runtime_root.join("stagex-sed-bridge-launcher"),
        crate::stagex_binutils::SED_BRIDGE_LAUNCHER_BLAKE3,
        sed_launcher_count,
    )?;
    assert!(configure_utility_count > sed_launcher_count);
    assert!(sed_launcher_count > report.sed_invocation_count);
    Ok(())
}

fn require_binutils_identity_count(
    counts: &BTreeMap<(PathBuf, String), u32>,
    path: &Path,
    digest: &str,
    expected_count: u32,
) -> Result<(), StagexTransitionError> {
    let observed = counts.get(&(path.to_path_buf(), digest.to_string())).copied().unwrap_or_default();
    if observed != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "GNU binutils exec count drifted for {}: expected {expected_count}, observed {observed}",
            path.display()
        )));
    }
    assert!(path.is_absolute());
    assert!(expected_count > 0);
    Ok(())
}

fn require_binutils_predecessor_identities(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    counts: &BTreeMap<(PathBuf, String), u32>,
) -> Result<(), StagexTransitionError> {
    let transition_root =
        report.runtime_root.parent().and_then(Path::parent).ok_or_else(|| {
            StagexTransitionError::Audit("GNU binutils runtime root has no transition root".to_string())
        })?;
    let paths = BinutilsStagePaths::from_transition_root(transition_root);
    require_binutils_non_coreutils_counts(report, counts, &paths)?;
    require_binutils_coreutils_counts(report, counts, &paths)?;
    assert!(paths.coreutils_bin.is_absolute());
    assert!(report.sed_invocation_count > 0);
    Ok(())
}

fn require_binutils_non_coreutils_counts(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    counts: &BTreeMap<(PathBuf, String), u32>,
    paths: &BinutilsStagePaths,
) -> Result<(), StagexTransitionError> {
    const TCC_COUNT: u32 = 1_423;
    const BASH_COUNT_BOUNDS: [u32; 2] = [7_619, 7_627];
    const SED_COUNT_OFFSET: u32 = 125;
    let sed_count = report
        .sed_invocation_count
        .checked_add(SED_COUNT_OFFSET)
        .ok_or_else(|| StagexTransitionError::Audit("GNU sed child event count overflow".to_string()))?;
    for (path, expected_count) in [
        (&paths.tcc, TCC_COUNT),
        (&paths.sed, sed_count),
        (&paths.grep, 560),
        (&paths.diff, 107),
        (&paths.cmp, 36),
        (&paths.gawk, 52),
        (&paths.m4, 15),
        (&paths.bison, 9),
        (&paths.flex, 7),
        (&paths.make, 104),
    ] {
        require_binutils_path_count(counts, path, expected_count)?;
    }
    require_binutils_path_count_bounds(counts, &paths.bash, BASH_COUNT_BOUNDS)?;
    assert_eq!(sed_count, report.sed_invocation_count + SED_COUNT_OFFSET);
    assert!(BASH_COUNT_BOUNDS[0] <= BASH_COUNT_BOUNDS[1]);
    Ok(())
}

fn require_binutils_coreutils_counts(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    counts: &BTreeMap<(PathBuf, String), u32>,
    paths: &BinutilsStagePaths,
) -> Result<(), StagexTransitionError> {
    const CHMOD_COUNT_BOUNDS: [u32; 2] = [993, 994];
    const CP_COUNT_BOUNDS: [u32; 2] = [104, 105];
    let sed = report.sed_invocation_count;
    let cat = sed
        .checked_add(4_738)
        .ok_or_else(|| StagexTransitionError::Audit("cat count overflow".to_string()))?;
    let mv = sed
        .checked_sub(4_463)
        .ok_or_else(|| StagexTransitionError::Audit("mv count underflow".to_string()))?;
    let rm = sed
        .checked_mul(2)
        .and_then(|count| count.checked_sub(1_504))
        .ok_or_else(|| StagexTransitionError::Audit("rm count overflow".to_string()))?;
    let rmdir = sed
        .checked_add(54)
        .ok_or_else(|| StagexTransitionError::Audit("rmdir count overflow".to_string()))?;
    let touch = sed
        .checked_sub(4_214)
        .ok_or_else(|| StagexTransitionError::Audit("touch count underflow".to_string()))?;
    let wc = sed
        .checked_mul(2)
        .and_then(|count| count.checked_add(748))
        .ok_or_else(|| StagexTransitionError::Audit("wc count overflow".to_string()))?;
    for (tool, expected_count) in [
        ("basename", 359),
        ("cat", cat),
        ("dirname", 1_267),
        ("echo", 17),
        ("expr", 364),
        ("head", sed),
        ("install", 175),
        ("ln", 45),
        ("ls", 28),
        ("mv", mv),
        ("rm", rm),
        ("rmdir", rmdir),
        ("sort", 36),
        ("touch", touch),
        ("tr", 23),
        ("wc", wc),
    ] {
        require_binutils_path_count(counts, &paths.coreutils_bin.join(tool), expected_count)?;
    }
    require_binutils_path_count_bounds(counts, &paths.coreutils_bin.join("chmod"), CHMOD_COUNT_BOUNDS)?;
    require_binutils_path_count_bounds(counts, &paths.coreutils_bin.join("cp"), CP_COUNT_BOUNDS)?;
    require_binutils_path_count_bounds(counts, &paths.coreutils_bin.join("mkdir"), BINUTILS_MKDIR_EXEC_COUNT_BOUNDS)?;
    assert!(cat > sed);
    assert!(wc > cat);
    Ok(())
}

fn require_binutils_path_count(
    counts: &BTreeMap<(PathBuf, String), u32>,
    path: &Path,
    expected_count: u32,
) -> Result<(), StagexTransitionError> {
    require_binutils_path_count_bounds(counts, path, [expected_count, expected_count])
}

fn require_binutils_path_count_bounds(
    counts: &BTreeMap<(PathBuf, String), u32>,
    path: &Path,
    count_bounds: [u32; 2],
) -> Result<(), StagexTransitionError> {
    let matching = counts
        .iter()
        .filter(|((observed_path, _), _)| observed_path == path)
        .map(|(_, count)| *count)
        .collect::<Vec<_>>();
    if matching.len() != 1 || matching[0] < count_bounds[0] || matching[0] > count_bounds[1] {
        return Err(StagexTransitionError::Audit(format!(
            "GNU binutils exec count drifted for {}: bounds {:?}, observed {:?}",
            path.display(),
            count_bounds,
            matching
        )));
    }
    assert!(path.is_absolute());
    assert!(count_bounds[0] > 0 && count_bounds[0] <= count_bounds[1]);
    Ok(())
}

fn validate_binutils_event_order(
    report: &crate::stagex_binutils::BinutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let runtime = &report.runtime_root;
    let canonicalizer = runtime.join("stagex-elf-local-symbol-canonicalizer");
    let chew = runtime.join("source/bfd/doc/chew");
    let i386_gen = runtime.join("source/opcodes/i386-gen");
    let sysinfo = runtime.join("source/binutils/sysinfo");
    let install_bin = runtime.join("install-destdir/mantle/stagex/binutils-probe-output/bin");
    let positive = runtime.join("binutils-runtime-smoke/positive");
    let first_canonicalizer = first_binutils_event_index(events, &canonicalizer)?;
    let first_chew = first_binutils_event_index(events, &chew)?;
    let first_i386_gen = first_binutils_event_index(events, &i386_gen)?;
    let first_sysinfo = first_binutils_event_index(events, &sysinfo)?;
    let first_installed_as = first_binutils_event_index(events, &install_bin.join("as"))?;
    let first_installed_ld = first_binutils_event_index(events, &install_bin.join("ld"))?;
    let first_positive = first_binutils_event_index(events, &positive)?;
    if first_chew <= first_canonicalizer
        || first_i386_gen <= first_canonicalizer
        || first_sysinfo <= first_canonicalizer
        || first_positive <= first_installed_as
        || first_positive <= first_installed_ld
    {
        return Err(StagexTransitionError::Audit(
            "GNU binutils generated-child producer order was substituted".to_string(),
        ));
    }
    assert!(first_chew > first_canonicalizer);
    assert!(first_positive > first_installed_as);
    Ok(())
}

fn first_binutils_event_index(
    events: &[ProtectedSeccompAuditEvent],
    path: &Path,
) -> Result<usize, StagexTransitionError> {
    let index = events.iter().position(|event| event.resolved_host_path == path).ok_or_else(|| {
        StagexTransitionError::Audit(format!("missing GNU binutils exec event for {}", path.display()))
    })?;
    assert!(index < events.len());
    assert!(events[index].resolved_host_path.is_absolute());
    Ok(index)
}

fn diffutils_expected_event_count(
    report: &crate::stagex_diffutils::DiffutilsInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 24;
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 26;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 6;
    if report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
    {
        return Err(StagexTransitionError::Audit("GNU diffutils command counts were substituted".to_string()));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU diffutils event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU diffutils event count does not fit usize".to_string()))?;
    let expected_count = usize::try_from(EXPECTED_BUILD_COMMAND_COUNT + EXPECTED_SMOKE_COMMAND_COUNT).unwrap();
    assert_eq!(count, expected_count);
    assert_eq!(report.source_compile_count, EXPECTED_SOURCE_COMPILE_COUNT);
    Ok(count)
}

fn split_diffutils_audit_suffix<'a>(
    report: Option<&crate::stagex_diffutils::DiffutilsInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = diffutils_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {count} trailing GNU diffutils events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_diffutils_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    report: &crate::stagex_diffutils::DiffutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 26;
    const SMOKE_EXECUTION_COUNT: usize = 6;
    let expected = diffutils_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU diffutils event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let diff = diffutils_output_path(report, "diffutils-diff-2.7")?;
    let cmp = diffutils_output_path(report, "diffutils-cmp-2.7")?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    let smoke_authorities = [
        (
            &diff,
            crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:diff",
        ),
        (
            &cmp,
            crate::stagex_diffutils::DIFFUTILS_CMP_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:cmp",
        ),
        (
            &diff,
            crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:diff",
        ),
        (
            &cmp,
            crate::stagex_diffutils::DIFFUTILS_CMP_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:cmp",
        ),
        (
            &diff,
            crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:diff",
        ),
        (
            &diff,
            crate::stagex_diffutils::DIFFUTILS_DIFF_BLAKE3,
            "planned:diffutils-materialization:exec:diffutils-smoke:diff",
        ),
    ];
    for (event, (path, digest, authorization_id)) in events[BUILD_EXECUTION_COUNT..].iter().zip(smoke_authorities) {
        validate_coreutils_event(event, path, digest, authorization_id)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU diffutils report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[BUILD_EXECUTION_COUNT..].len(), SMOKE_EXECUTION_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn grep_expected_event_count(report: &crate::stagex_grep::GrepInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 2;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
    const EXPECTED_EVENT_COUNT: usize = 5;
    if report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
    {
        return Err(StagexTransitionError::Audit("GNU grep command counts were substituted".to_string()));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU grep event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU grep event count does not fit usize".to_string()))?;
    if count != EXPECTED_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU grep event count is not closed: expected {EXPECTED_EVENT_COUNT}, observed {count}"
        )));
    }
    assert_eq!(count, EXPECTED_EVENT_COUNT);
    assert!(report.protected_exec_enforced || report.fallback_events.is_empty());
    Ok(count)
}

fn split_grep_audit_suffix<'a>(
    report: Option<&crate::stagex_grep::GrepInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = grep_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing GNU grep events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_grep_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    report: &crate::stagex_grep::GrepInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 2;
    const SMOKE_EXECUTION_COUNT: usize = 3;
    let expected = grep_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU grep event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let grep = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "grep-2.4-runner")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU grep report lacks its native runner".to_string()))?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    for event in &events[BUILD_EXECUTION_COUNT..] {
        validate_coreutils_event(
            event,
            grep,
            crate::stagex_grep::GREP_RUNNER_BLAKE3,
            GREP_RUNNER_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU grep report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[BUILD_EXECUTION_COUNT..].len(), SMOKE_EXECUTION_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn gawk_expected_event_count(report: &crate::stagex_gawk::GawkInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 17;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 7;
    const EXPECTED_EVENT_COUNT: usize = 24;
    if report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.negative_exit_code == 0
    {
        return Err(StagexTransitionError::Audit(
            "GNU Gawk command counts or negative result were substituted".to_string(),
        ));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU Gawk event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU Gawk event count does not fit usize".to_string()))?;
    if count != EXPECTED_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Gawk event count is not closed: expected {EXPECTED_EVENT_COUNT}, observed {count}"
        )));
    }
    assert_eq!(count, EXPECTED_EVENT_COUNT);
    assert_ne!(report.negative_exit_code, 0);
    Ok(count)
}

fn split_gawk_audit_suffix<'a>(
    report: Option<&crate::stagex_gawk::GawkInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = gawk_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing GNU Gawk events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_gawk_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    report: &crate::stagex_gawk::GawkInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 17;
    const SMOKE_EXECUTION_COUNT: usize = 7;
    let expected = gawk_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Gawk event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let gawk = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "gawk-3.0.4")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU Gawk report lacks its executable".to_string()))?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    for event in &events[BUILD_EXECUTION_COUNT..] {
        validate_coreutils_event(event, gawk, crate::stagex_gawk::GAWK_BINARY_BLAKE3, GAWK_CANONICAL_AUTHORIZATION_ID)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU Gawk report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[BUILD_EXECUTION_COUNT..].len(), SMOKE_EXECUTION_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn bison_expected_event_count(
    report: &crate::stagex_bison::BisonInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 56;
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 57;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
    const BISON_M4_CHILD_EVENT_COUNT: u32 = 1;
    const EXPECTED_EVENT_COUNT: usize = 61;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.negative_exit_code == 0
    {
        return Err(StagexTransitionError::Audit(
            "GNU Bison command counts or negative result were substituted".to_string(),
        ));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .and_then(|count| count.checked_add(BISON_M4_CHILD_EVENT_COUNT))
        .ok_or_else(|| StagexTransitionError::Audit("GNU Bison event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU Bison event count does not fit usize".to_string()))?;
    if count != EXPECTED_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Bison event count is not closed: expected {EXPECTED_EVENT_COUNT}, observed {count}"
        )));
    }
    assert_eq!(count, EXPECTED_EVENT_COUNT);
    assert_ne!(report.negative_exit_code, 0);
    Ok(count)
}

fn split_bison_audit_suffix<'a>(
    report: Option<&crate::stagex_bison::BisonInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = bison_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing GNU Bison events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_bison_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    m4_report: &crate::stagex_m4::M4InventoryReport,
    report: &crate::stagex_bison::BisonInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 57;
    const SMOKE_WITH_CHILD_EVENT_COUNT: usize = 4;
    let expected = bison_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Bison event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let bison = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "bison-2.3")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU Bison report lacks its executable".to_string()))?;
    let m4 = m4_report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "m4-1.4.7")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU M4 report lacks its executable".to_string()))?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    let smoke = &events[BUILD_EXECUTION_COUNT..];
    let authorities = [
        (bison, crate::stagex_bison::BISON_BINARY_BLAKE3, BISON_CANONICAL_AUTHORIZATION_ID),
        (bison, crate::stagex_bison::BISON_BINARY_BLAKE3, BISON_CANONICAL_AUTHORIZATION_ID),
        (m4, crate::stagex_m4::M4_FINAL_BLAKE3, M4_CANONICAL_AUTHORIZATION_ID),
        (bison, crate::stagex_bison::BISON_BINARY_BLAKE3, BISON_CANONICAL_AUTHORIZATION_ID),
    ];
    for (event, (path, digest, authorization_id)) in smoke.iter().zip(authorities) {
        validate_coreutils_event(event, path, digest, authorization_id)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU Bison report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(smoke.len(), SMOKE_WITH_CHILD_EVENT_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn flex_expected_event_count(report: &crate::stagex_flex::FlexInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 21;
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 25;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 4;
    const FLEX_M4_CHILD_EVENT_COUNT: u32 = 2;
    const EXPECTED_EVENT_COUNT: usize = 31;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.negative_exit_code == 0
    {
        return Err(StagexTransitionError::Audit(
            "GNU Flex command counts or negative result were substituted".to_string(),
        ));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .and_then(|count| count.checked_add(FLEX_M4_CHILD_EVENT_COUNT))
        .ok_or_else(|| StagexTransitionError::Audit("GNU Flex event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU Flex event count does not fit usize".to_string()))?;
    if count != EXPECTED_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Flex event count is not closed: expected {EXPECTED_EVENT_COUNT}, observed {count}"
        )));
    }
    assert_eq!(count, EXPECTED_EVENT_COUNT);
    assert_ne!(report.negative_exit_code, 0);
    Ok(count)
}

fn split_flex_audit_suffix<'a>(
    report: Option<&crate::stagex_flex::FlexInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = flex_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing GNU Flex events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_flex_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    m4_report: &crate::stagex_m4::M4InventoryReport,
    report: &crate::stagex_flex::FlexInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 25;
    const SMOKE_WITH_CHILD_EVENT_COUNT: usize = 6;
    let expected = flex_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Flex event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let flex = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "flex-2.6.4")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU Flex report lacks its executable".to_string()))?;
    let consumer = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "flex-libfl-consumer")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU Flex report lacks its libfl consumer".to_string()))?;
    let m4 = m4_report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "m4-1.4.7")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU M4 report lacks its executable".to_string()))?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    let smoke = &events[BUILD_EXECUTION_COUNT..];
    let authorities = [
        (
            consumer,
            crate::stagex_flex::FLEX_LIBFL_CONSUMER_BLAKE3,
            FLEX_LIBFL_CONSUMER_CANONICAL_AUTHORIZATION_ID,
        ),
        (flex, crate::stagex_flex::FLEX_BINARY_BLAKE3, FLEX_CANONICAL_AUTHORIZATION_ID),
        (flex, crate::stagex_flex::FLEX_BINARY_BLAKE3, FLEX_CANONICAL_AUTHORIZATION_ID),
        (m4, crate::stagex_m4::M4_FINAL_BLAKE3, M4_CANONICAL_AUTHORIZATION_ID),
        (flex, crate::stagex_flex::FLEX_BINARY_BLAKE3, FLEX_CANONICAL_AUTHORIZATION_ID),
        (m4, crate::stagex_m4::M4_FINAL_BLAKE3, M4_CANONICAL_AUTHORIZATION_ID),
    ];
    for (event, (path, digest, authorization_id)) in smoke.iter().zip(authorities) {
        validate_coreutils_event(event, path, digest, authorization_id)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU Flex report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(smoke.len(), SMOKE_WITH_CHILD_EVENT_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn bash_full_expected_event_count(
    report: &crate::stagex_bash_full::BashFullInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 130;
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 133;
    const EXPECTED_GENERATOR_COMMAND_COUNT: u32 = 40;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 5;
    const CHILD_EVENT_COUNT: u32 = 2;
    const EXPECTED_EVENT_COUNT: usize = 180;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.generator_command_count != EXPECTED_GENERATOR_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.negative_exit_code == 0
    {
        return Err(StagexTransitionError::Audit(
            "full Bash command counts or negative result were substituted".to_string(),
        ));
    }
    let count = report
        .build_command_count
        .checked_add(report.generator_command_count)
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .and_then(|count| count.checked_add(CHILD_EVENT_COUNT))
        .ok_or_else(|| StagexTransitionError::Audit("full Bash event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("full Bash event count does not fit usize".to_string()))?;
    if count != EXPECTED_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "full Bash event count is not closed: expected {EXPECTED_EVENT_COUNT}, observed {count}"
        )));
    }
    assert_eq!(count, EXPECTED_EVENT_COUNT);
    assert_ne!(report.negative_exit_code, 0);
    Ok(count)
}

fn split_bash_full_audit_suffix<'a>(
    report: Option<&crate::stagex_bash_full::BashFullInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = bash_full_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing full Bash events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn bash_full_event_boundaries() -> Result<(usize, usize, usize), StagexTransitionError> {
    const GENERATOR_BUILD_EXECUTION_COUNT: usize = 2;
    const GENERATOR_RUN_EXECUTION_COUNT: usize = 40;
    const SHELL_BUILD_EXECUTION_COUNT: usize = 131;
    let generator_start = GENERATOR_BUILD_EXECUTION_COUNT;
    let shell_start = generator_start
        .checked_add(GENERATOR_RUN_EXECUTION_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("full Bash generator range overflow".to_string()))?;
    let smoke_start = shell_start
        .checked_add(SHELL_BUILD_EXECUTION_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("full Bash shell range overflow".to_string()))?;
    assert!(generator_start < shell_start);
    assert!(shell_start < smoke_start);
    Ok((generator_start, shell_start, smoke_start))
}

fn validate_bash_full_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    make_report: &crate::stagex_make::MakeInventoryReport,
    report: &crate::stagex_bash_full::BashFullInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const SMOKE_WITH_CHILD_EVENT_COUNT: usize = 7;
    let expected = bash_full_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "full Bash event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let bash = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "bash-2.05b-full")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("full Bash report lacks its executable".to_string()))?;
    let make = make_report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "make-3.82")
        .map(|output| output.path.as_path())
        .ok_or_else(|| StagexTransitionError::Audit("GNU Make report lacks its executable".to_string()))?;
    let (generator_start, shell_start, smoke_start) = bash_full_event_boundaries()?;
    validate_bash_full_build_events(
        events,
        &compiler,
        &report.generator_path,
        generator_start,
        shell_start,
        smoke_start,
    )?;
    let authorities = [
        (bash, crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3, BASH_FULL_CANONICAL_AUTHORIZATION_ID),
        (bash, crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3, BASH_FULL_CANONICAL_AUTHORIZATION_ID),
        (bash, crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3, BASH_FULL_CANONICAL_AUTHORIZATION_ID),
        (bash, crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3, BASH_FULL_CANONICAL_AUTHORIZATION_ID),
        (make, crate::stagex_make::MAKE_FINAL_BLAKE3, MAKE_CANONICAL_AUTHORIZATION_ID),
        (make, crate::stagex_make::MAKE_FINAL_BLAKE3, MAKE_CANONICAL_AUTHORIZATION_ID),
        (bash, crate::stagex_bash_full::BASH_FULL_BINARY_BLAKE3, BASH_FULL_CANONICAL_AUTHORIZATION_ID),
    ];
    for (event, (path, digest, authorization_id)) in events[smoke_start..].iter().zip(authorities) {
        validate_coreutils_event(event, path, digest, authorization_id)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "full Bash report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[smoke_start..].len(), SMOKE_WITH_CHILD_EVENT_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn validate_bash_full_build_events(
    events: &[ProtectedSeccompAuditEvent],
    compiler: &Path,
    generator: &Path,
    generator_start: usize,
    shell_start: usize,
    smoke_start: usize,
) -> Result<(), StagexTransitionError> {
    for event in events[..generator_start].iter().chain(events[shell_start..smoke_start].iter()) {
        validate_coreutils_event(
            event,
            compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    for event in &events[generator_start..shell_start] {
        validate_coreutils_event(
            event,
            generator,
            crate::stagex_bash_full::BASH_FULL_GENERATOR_BLAKE3,
            BASH_FULL_GENERATOR_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    assert!(generator_start < shell_start);
    assert!(shell_start < smoke_start);
    Ok(())
}

fn m4_expected_event_count(report: &crate::stagex_m4::M4InventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 29;
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 30;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 5;
    if report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
    {
        return Err(StagexTransitionError::Audit("GNU M4 command counts were substituted".to_string()));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU M4 event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU M4 event count does not fit usize".to_string()))?;
    let expected_count = usize::try_from(EXPECTED_BUILD_COMMAND_COUNT + EXPECTED_SMOKE_COMMAND_COUNT).unwrap();
    assert_eq!(count, expected_count);
    assert_eq!(report.source_compile_count, EXPECTED_SOURCE_COMPILE_COUNT);
    Ok(count)
}

fn split_m4_audit_suffix<'a>(
    report: Option<&crate::stagex_m4::M4InventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = m4_expected_event_count(report)?;
    let start = events.len().checked_sub(count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {count} trailing GNU M4 events, observed {}", events.len()))
    })?;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_m4_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    report: &crate::stagex_m4::M4InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const BUILD_EXECUTION_COUNT: usize = 30;
    const SMOKE_EXECUTION_COUNT: usize = 5;
    let expected = m4_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "GNU M4 event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let m4 = m4_output_path(report, "m4-1.4.7")?;
    for event in &events[..BUILD_EXECUTION_COUNT] {
        validate_coreutils_event(
            event,
            &compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )?;
    }
    for event in &events[BUILD_EXECUTION_COUNT..] {
        validate_coreutils_event(event, &m4, crate::stagex_m4::M4_FINAL_BLAKE3, M4_CANONICAL_AUTHORIZATION_ID)?;
    }
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "GNU M4 report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[BUILD_EXECUTION_COUNT..].len(), SMOKE_EXECUTION_COUNT);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn musl_native_expected_event_count(
    report: &crate::stagex_musl_native::InventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_ARCHIVE_COMMAND_COUNT: u32 = 1;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 2;
    const EXPECTED_EXECUTION_COMMAND_COUNT: u32 = 2;
    if report.archive_command_count != EXPECTED_ARCHIVE_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
        || report.execution_command_count != EXPECTED_EXECUTION_COMMAND_COUNT
        || report.selfhost_compile_count != crate::stagex_musl_native::SELFHOST_COMPILE_COUNT
        || report.predecessor_compile_count != crate::stagex_musl_native::PREDECESSOR_COMPILE_COUNT
    {
        return Err(StagexTransitionError::Audit("native musl command counts were substituted".to_string()));
    }
    let count = report
        .selfhost_compile_count
        .checked_add(report.predecessor_compile_count)
        .and_then(|value| value.checked_add(report.archive_command_count))
        .and_then(|value| value.checked_add(report.smoke_command_count))
        .and_then(|value| value.checked_add(report.execution_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("native musl event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("native musl event count does not fit usize".to_string()))?;
    assert!(count > usize::try_from(EXPECTED_SMOKE_COMMAND_COUNT).unwrap());
    assert!(report.selfhost_compile_count > 0);
    Ok(count)
}

fn split_musl_native_audit_suffix<'a>(
    report: Option<&crate::stagex_musl_native::InventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = musl_native_expected_event_count(report)?;
    if events.len() < count {
        return Err(StagexTransitionError::Audit(format!(
            "expected {count} trailing native musl events, observed {}",
            events.len()
        )));
    }
    let start = events.len() - count;
    assert_eq!(events[start..].len(), count);
    assert!(start <= events.len());
    Ok((&events[..start], &events[start..]))
}

fn validate_musl_native_audit(
    selfhost: &crate::stagex_tcc_selfhost::InventoryReport,
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    report: &crate::stagex_musl_native::InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const ARCHIVE_EXECUTIONS: usize = 1;
    const SMOKE_EXECUTIONS: usize = 4;
    const SMOKE_COMPILE_OFFSET: usize = 0;
    const SMOKE_LINK_OFFSET: usize = 1;
    const SMOKE_EXECUTION_OFFSET: usize = 2;
    const SMOKE_NEGATIVE_OFFSET: usize = 3;
    let expected = musl_native_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "expected {expected} native musl events, observed {}",
            events.len()
        )));
    }
    let selfhost_path = tcc_selfhost_output_path(selfhost, "tcc-musl-selfhost")?;
    let predecessor_path = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let selfhost_count = usize::try_from(report.selfhost_compile_count)
        .map_err(|_| StagexTransitionError::Audit("native musl self-host count does not fit usize".to_string()))?;
    let predecessor_count = usize::try_from(report.predecessor_compile_count)
        .map_err(|_| StagexTransitionError::Audit("native musl predecessor count does not fit usize".to_string()))?;
    let build_count = selfhost_count
        .checked_add(predecessor_count)
        .ok_or_else(|| StagexTransitionError::Audit("native musl build count overflow".to_string()))?;
    let mut observed_selfhost = 0usize;
    let mut observed_predecessor = 0usize;
    for event in &events[..build_count] {
        if event.executable_path == selfhost_path {
            validate_coreutils_event(
                event,
                &selfhost_path,
                crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
                "planned:tcc-musl-selfhost-build:exec:tcc-musl-selfhost-smoke:tcc",
            )?;
            observed_selfhost += 1;
        } else if event.executable_path == predecessor_path {
            validate_coreutils_event(
                event,
                &predecessor_path,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
                "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc",
            )?;
            observed_predecessor += 1;
        } else {
            return Err(StagexTransitionError::Audit(format!(
                "native musl build used undeclared executable {}",
                event.executable_path.display()
            )));
        }
    }
    if observed_selfhost != usize::try_from(report.selfhost_compile_count).unwrap()
        || observed_predecessor != usize::try_from(report.predecessor_compile_count).unwrap()
    {
        return Err(StagexTransitionError::Audit(
            "native musl compiler route counts do not match the report".to_string(),
        ));
    }
    let archive_index = build_count;
    validate_coreutils_event(
        &events[archive_index],
        &selfhost_path,
        crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
        "planned:tcc-musl-selfhost-build:exec:tcc-musl-selfhost-smoke:tcc",
    )?;
    let smoke_start = archive_index + ARCHIVE_EXECUTIONS;
    let smoke_binary = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "musl-native-smoke-binary")
        .ok_or_else(|| StagexTransitionError::Audit("native musl report lacks smoke binary".to_string()))?;
    for event in [
        &events[smoke_start + SMOKE_COMPILE_OFFSET],
        &events[smoke_start + SMOKE_LINK_OFFSET],
    ] {
        validate_coreutils_event(
            event,
            &selfhost_path,
            crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
            "planned:tcc-musl-selfhost-build:exec:tcc-musl-selfhost-smoke:tcc",
        )?;
    }
    validate_coreutils_event(
        &events[smoke_start + SMOKE_EXECUTION_OFFSET],
        &smoke_binary.path,
        &smoke_binary.digest_blake3,
        "planned:musl-native-smoke:exec:musl-native-execution:binary",
    )?;
    validate_coreutils_event(
        &events[smoke_start + SMOKE_NEGATIVE_OFFSET],
        &predecessor_path,
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc",
    )?;
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "native musl report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events[smoke_start..].len(), SMOKE_EXECUTIONS);
    assert_eq!(observed_selfhost, usize::try_from(report.selfhost_compile_count).unwrap());
    Ok(())
}

fn tcc_selfhost_expected_event_count(
    report: &crate::stagex_tcc_selfhost::InventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COMMAND_COUNT: u32 = 13;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 1;
    if report.build_command_count != EXPECTED_BUILD_COMMAND_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
    {
        return Err(StagexTransitionError::Audit("TinyCC self-host command count was substituted".to_string()));
    }
    let count = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC self-host event count overflow".to_string()))?;
    let count = usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("TinyCC self-host event count does not fit usize".to_string()))?;
    assert_eq!(count, 14);
    assert!(count > 0);
    Ok(count)
}

fn split_tcc_selfhost_audit_suffix<'a>(
    report: Option<&crate::stagex_tcc_selfhost::InventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &[]));
    };
    let count = tcc_selfhost_expected_event_count(report)?;
    if events.len() < count {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC self-host expected {count} events but only {} remain",
            events.len()
        )));
    }
    Ok(events.split_at(events.len() - count))
}

fn validate_tcc_selfhost_audit(
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    tinycc: &crate::stagex_tinycc::TccMesInventoryReport,
    report: &crate::stagex_tcc_selfhost::InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const V2_INITIAL_BUILD_EXECUTIONS: usize = 11;
    const TINYCC26_LINK_EXECUTIONS: usize = 1;
    const V2_CANONICAL_ARCHIVE_EXECUTIONS: usize = 1;
    let expected = tcc_selfhost_expected_event_count(report)?;
    if events.len() != expected {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC self-host event count is not closed: expected {expected}, observed {}",
            events.len()
        )));
    }
    let v2_compiler = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let tinycc26 = tinycc_output_path(tinycc, "tinycc-0.9.26")?;
    let selfhost = tcc_selfhost_output_path(report, "tcc-musl-selfhost")?;
    for event in &events[..V2_INITIAL_BUILD_EXECUTIONS] {
        validate_coreutils_event(
            event,
            &v2_compiler,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc",
        )?;
    }
    let tinycc26_index = V2_INITIAL_BUILD_EXECUTIONS;
    validate_coreutils_event(
        &events[tinycc26_index],
        &tinycc26,
        crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
        "planned:tinycc-final-materialization:exec:tcc-musl-v2-materialization:tinycc26",
    )?;
    let canonical_archive_index = tinycc26_index + TINYCC26_LINK_EXECUTIONS;
    validate_coreutils_event(
        &events[canonical_archive_index],
        &v2_compiler,
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc",
    )?;
    let smoke_index = canonical_archive_index + V2_CANONICAL_ARCHIVE_EXECUTIONS;
    validate_coreutils_event(
        &events[smoke_index],
        &selfhost,
        crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
        "planned:tcc-musl-selfhost-build:exec:tcc-musl-selfhost-smoke:tcc",
    )?;
    if !report.protected_exec_enforced || !report.fallback_events.is_empty() {
        return Err(StagexTransitionError::Audit(
            "TinyCC self-host report lacks protected execution or contains fallback events".to_string(),
        ));
    }
    assert_eq!(events.len(), expected);
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn split_tcc_musl_v2_audit_suffix<'a>(
    report: Option<&crate::stagex_tcc_musl_v2::InventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = tcc_musl_v2_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing TinyCC musl-v2 events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_tcc_musl_audit_suffix<'a>(
    report: Option<&crate::stagex_tcc_musl::TccMuslInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = tcc_musl_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing musl-linked TinyCC events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_musl_audit_suffix<'a>(
    report: Option<&crate::stagex_musl::MuslInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = musl_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {event_count} trailing musl events, observed {}", events.len()))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_tcc_musl_prep_audit_suffix<'a>(
    report: Option<&crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = tcc_musl_prep_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing TinyCC musl-prep events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_bash_audit_suffix<'a>(
    report: Option<&crate::stagex_bash::BashInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = bash_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {event_count} trailing bash events, observed {}", events.len()))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_oyacc_audit_suffix<'a>(
    report: Option<&crate::stagex_oyacc::OyaccInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = oyacc_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {event_count} trailing oyacc events, observed {}", events.len()))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_coreutils_audit_suffix<'a>(
    report: Option<&crate::stagex_coreutils::CoreutilsInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = coreutils_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing coreutils events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_bzip2_audit_suffix<'a>(
    report: Option<&crate::stagex_bzip2::Bzip2InventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = bzip2_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {event_count} trailing bzip2 events, observed {}", events.len()))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_sed_audit_suffix<'a>(
    report: Option<&crate::stagex_sed::SedInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = sed_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing GNU sed events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_tar_audit_suffix<'a>(
    report: Option<&crate::stagex_tar::TarInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = tar_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing GNU tar events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_gzip_audit_suffix<'a>(
    report: Option<&crate::stagex_gzip::GzipInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = gzip_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!("expected {event_count} trailing gzip events, observed {}", events.len()))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_gnu_patch_audit_suffix<'a>(
    report: Option<&crate::stagex_gnu_patch::GnuPatchInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let event_count = gnu_patch_expected_event_count(report)?;
    let start = events.len().checked_sub(event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {event_count} trailing GNU patch events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(start), event_count);
    assert!(event_count > 0);
    Ok((&events[..start], &events[start..]))
}

fn split_make_audit_suffix<'a>(
    report: Option<&crate::stagex_make::MakeInventoryReport>,
    events: &'a [ProtectedSeccompAuditEvent],
) -> Result<(&'a [ProtectedSeccompAuditEvent], &'a [ProtectedSeccompAuditEvent]), StagexTransitionError> {
    let Some(report) = report else {
        return Ok((events, &events[events.len()..]));
    };
    let make_event_count = make_expected_event_count(report)?;
    let make_start = events.len().checked_sub(make_event_count).ok_or_else(|| {
        StagexTransitionError::Audit(format!(
            "expected {make_event_count} trailing GNU Make events, observed {}",
            events.len()
        ))
    })?;
    assert_eq!(events.len().saturating_sub(make_start), make_event_count);
    assert!(make_event_count > 0);
    Ok((&events[..make_start], &events[make_start..]))
}

fn mes_m2_event_count(report: &crate::stagex_mes::MesM2InventoryReport) -> Result<usize, StagexTransitionError> {
    usize::try_from(report.command_count)
        .map_err(|_| StagexTransitionError::Audit("Mes command count does not fit usize".to_string()))?
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("Mes event count overflow".to_string()))
}

fn validate_mes_and_tinycc_suffix(
    stage0: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    mes_m2: &crate::stagex_mes::MesM2InventoryReport,
    mes_runtime: &crate::stagex_mes_lib::MesRuntimeInventoryReport,
    tinycc: &crate::stagex_tinycc::TccMesInventoryReport,
    tinycc27: Option<&crate::stagex_tinycc27::Tinycc27InventoryReport>,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let mes_path = mes_m2
        .outputs
        .iter()
        .find(|output| output.artifact_id == "mes-m2")
        .ok_or_else(|| StagexTransitionError::Audit("mes-m2 report lacks output path".to_string()))?
        .path
        .as_path();
    let first_mes_index = events
        .iter()
        .position(|event| event.executable_path == mes_path)
        .ok_or_else(|| StagexTransitionError::Audit("protected audit never executed mes-m2".to_string()))?;
    let mes_event_count = mes_m2_event_count(mes_m2)?;
    let mes_end = first_mes_index
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("Mes audit end overflow".to_string()))?;
    let mes_start = mes_end
        .checked_sub(mes_event_count)
        .ok_or_else(|| StagexTransitionError::Audit("Mes audit prefix is too short".to_string()))?;
    let runtime_event_count = mes_runtime_event_count(mes_runtime)?;
    let runtime_end = mes_end
        .checked_add(runtime_event_count)
        .ok_or_else(|| StagexTransitionError::Audit("Mes runtime audit end overflow".to_string()))?;
    if runtime_end > events.len() {
        return Err(StagexTransitionError::Audit(format!(
            "Mes runtime audit ends at {runtime_end}, but only {} events are available",
            events.len()
        )));
    }
    validate_stage0_full_audit(stage0, &events[..mes_start])?;
    validate_mes_m2_audit(stage0, mes_m2, &events[mes_start..mes_end])?;
    validate_mes_runtime_audit(stage0, mes_m2, mes_runtime, &events[mes_end..runtime_end])?;
    let compiler_events = &events[runtime_end..];
    if let Some(tinycc27) = tinycc27 {
        let tinycc27_event_count = tinycc27_expected_event_count(tinycc27)?;
        let tinycc27_start = compiler_events.len().checked_sub(tinycc27_event_count).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "expected {tinycc27_event_count} trailing TinyCC 0.9.27 events, observed {}",
                compiler_events.len()
            ))
        })?;
        validate_tinycc_audit(stage0, mes_m2, tinycc, &compiler_events[..tinycc27_start])?;
        return validate_tinycc27_audit(tinycc, tinycc27, &compiler_events[tinycc27_start..]);
    }
    validate_tinycc_audit(stage0, mes_m2, tinycc, compiler_events)
}

fn transition_expected_audit_events(
    staged: &StagedTransitionPaths,
) -> [ExpectedAuditEvent<'_>; EXPECTED_AUDIT_EVENT_COUNT] {
    [
        ExpectedAuditEvent {
            executable_path: &staged.seed,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: "stagex:seed:hex0".to_string(),
        },
        ExpectedAuditEvent {
            executable_path: &staged.reproduced_hex0,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: format!("promoted:{HEX0_REPRODUCTION_STAGE_ID}:{}", staged.reproduced_hex0.display()),
        },
        ExpectedAuditEvent {
            executable_path: &staged.kaem,
            digest_blake3: KAEM_OUTPUT_BLAKE3,
            inventory_entry_id: format!("promoted:{KAEM_MATERIALIZATION_STAGE_ID}:{}", staged.kaem.display()),
        },
    ]
}

fn validate_stage0_audit(
    staged: &StagedTransitionPaths,
    report: &crate::stagex_stage0::Stage0MiniInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const STAGE0_PARENT_EVENT_COUNT: usize = 1;
    let command_count = usize::try_from(report.command_count)
        .map_err(|_| StagexTransitionError::Audit("Stage0 command count does not fit usize".to_string()))?;
    let expected_event_count = command_count
        .checked_add(STAGE0_PARENT_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("Stage0 audit event count overflow".to_string()))?;
    if events.len() != expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected {expected_event_count} Stage0 events, observed {}",
            events.len()
        )));
    }
    let parent = ExpectedAuditEvent {
        executable_path: &staged.kaem,
        digest_blake3: KAEM_OUTPUT_BLAKE3,
        inventory_entry_id: format!("promoted:{KAEM_MATERIALIZATION_STAGE_ID}:{}", staged.kaem.display()),
    };
    validate_audit_event(&events[0], &parent)?;
    validate_stage0_child_events(report.sources_root(), &events[STAGE0_PARENT_EVENT_COUNT..])
}

fn validate_stage0_child_events(
    stage0_root: &Path,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let planned = crate::stagex_stage0::planned_stage0_executables(stage0_root);
    let by_path = planned.iter().map(|entry| (entry.path.as_path(), entry)).collect::<BTreeMap<_, _>>();
    let mut observed_paths = BTreeSet::new();
    for event in events {
        let expected = by_path.get(event.executable_path.as_path()).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "Stage0 event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let expected_id = format!("planned:{}:{}", expected.source_stage_id, expected.authorization_id);
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == expected.path
            && event.tracee_path == expected.path
            && event.digest_hex == expected.digest_blake3
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "Stage0 event for {} did not match exact planned authority",
                expected.path.display()
            )));
        }
        observed_paths.insert(event.executable_path.clone());
    }
    for expected in &planned {
        if !observed_paths.contains(&expected.path) {
            return Err(StagexTransitionError::Audit(format!(
                "planned Stage0 executable was not observed: {}",
                expected.path.display()
            )));
        }
    }
    assert!(!events.is_empty());
    assert_eq!(observed_paths.len(), by_path.len());
    Ok(())
}

fn validate_stage0_full_audit(
    report: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    if events.is_empty() {
        return Err(StagexTransitionError::Audit("full Stage0 audit has no events".to_string()));
    }
    let stage0_root = report.mini.sources_root();
    let mut authorities = BTreeMap::new();
    for planned in crate::stagex_stage0::planned_stage0_executables(stage0_root) {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path, (planned.digest_blake3, inventory_id));
    }
    let full_planned = crate::stagex_stage0_full::planned_stage0_full_executables(stage0_root);
    for planned in &full_planned {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path.clone(), (planned.digest_blake3.clone(), inventory_id));
    }
    let mut observed_paths = BTreeSet::new();
    for event in events {
        let (expected_digest, expected_id) = authorities.get(&event.executable_path).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "full Stage0 event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == event.executable_path
            && event.tracee_path == event.executable_path
            && event.digest_hex == *expected_digest
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "full Stage0 event for {} did not match exact planned authority",
                event.executable_path.display()
            )));
        }
        observed_paths.insert(event.executable_path.clone());
    }
    for planned in full_planned {
        if !observed_paths.contains(&planned.path) {
            return Err(StagexTransitionError::Audit(format!(
                "planned full Stage0 executable was not observed: {}",
                planned.path.display()
            )));
        }
    }
    assert!(!events.is_empty());
    assert!(observed_paths.len() <= authorities.len());
    Ok(())
}

fn validate_mes_m2_audit(
    stage0: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    report: &crate::stagex_mes::MesM2InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const MES_PARENT_EVENT_COUNT: usize = 1;
    let command_count = usize::try_from(report.command_count)
        .map_err(|_| StagexTransitionError::Audit("Mes command count does not fit usize".to_string()))?;
    let expected_event_count = command_count
        .checked_add(MES_PARENT_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("Mes event count overflow".to_string()))?;
    if events.len() != expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected exactly {expected_event_count} mes-m2 events, observed {}",
            events.len()
        )));
    }
    let stage0_root = stage0.mini.sources_root();
    let mut authorities = BTreeMap::new();
    for planned in crate::stagex_stage0::planned_stage0_executables(stage0_root) {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path, (planned.digest_blake3, inventory_id));
    }
    for planned in crate::stagex_stage0_full::planned_stage0_full_executables(stage0_root) {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path, (planned.digest_blake3, inventory_id));
    }
    for planned in additional_mes_stage0_planned_executables(stage0_root) {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path, (planned.digest_hex, inventory_id));
    }
    let mes_m2_path = report
        .outputs
        .iter()
        .find(|output| output.artifact_id == "mes-m2")
        .ok_or_else(|| StagexTransitionError::Audit("Mes report lacks mes-m2 output".to_string()))?
        .path
        .clone();
    let mes_inventory_id = format!("planned:{MES_M2_STAGE_ID}:exec:mes-m2-smoke:mes-m2");
    authorities.insert(mes_m2_path.clone(), (crate::stagex_mes::MES_M2_BLAKE3.to_string(), mes_inventory_id));
    let mut observed_paths = BTreeSet::new();
    for event in events {
        let (expected_digest, expected_id) = authorities.get(&event.executable_path).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "mes-m2 event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == event.executable_path
            && event.tracee_path == event.executable_path
            && event.digest_hex == *expected_digest
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "mes-m2 event for {} did not match exact planned authority",
                event.executable_path.display()
            )));
        }
        observed_paths.insert(event.executable_path.clone());
    }
    let required_artifacts = [
        "stage0-kaem",
        "stage0-m1",
        "stage0-hex2",
        "stage0-full-m2-planet",
        "stage0-full-blood-elf",
        "stage0-full-mkdir",
        "stage0-full-cp",
    ];
    for artifact_id in required_artifacts {
        let path = stage0_artifact_path(stage0_root, artifact_id)?;
        if !observed_paths.contains(&path) {
            return Err(StagexTransitionError::Audit(format!(
                "mes-m2 stage did not observe required executable {artifact_id} at {}",
                path.display()
            )));
        }
    }
    if !observed_paths.contains(&mes_m2_path) {
        return Err(StagexTransitionError::Audit("mes-m2 smoke executable was not observed".to_string()));
    }
    assert_eq!(events.len(), expected_event_count);
    assert!(observed_paths.len() <= authorities.len());
    Ok(())
}

fn mes_runtime_event_count(
    report: &crate::stagex_mes_lib::MesRuntimeInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXEC_EVENTS_PER_COMPILE: u32 = 2;
    let compile_events = report
        .source_compile_count
        .checked_mul(EXEC_EVENTS_PER_COMPILE)
        .ok_or_else(|| StagexTransitionError::Audit("Mes runtime compile event count overflow".to_string()))?;
    let total = report
        .nyacc_command_count
        .checked_add(compile_events)
        .ok_or_else(|| StagexTransitionError::Audit("Mes runtime event count overflow".to_string()))?;
    let count = usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("Mes runtime event count does not fit usize".to_string()))?;
    assert!(count >= usize::try_from(report.nyacc_command_count).unwrap_or(usize::MAX));
    assert!(count > 0);
    Ok(count)
}

fn validate_mes_runtime_audit(
    stage0: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    mes_m2: &crate::stagex_mes::MesM2InventoryReport,
    runtime: &crate::stagex_mes_lib::MesRuntimeInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_event_count = mes_runtime_event_count(runtime)?;
    if events.len() != expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected exactly {expected_event_count} Mes runtime events, observed {}",
            events.len()
        )));
    }
    let mes_path = mes_m2
        .outputs
        .iter()
        .find(|output| output.artifact_id == "mes-m2")
        .ok_or_else(|| StagexTransitionError::Audit("mes-m2 report lacks executable output".to_string()))?
        .path
        .clone();
    let stage0_root = stage0.mini.sources_root();
    let m1_path = stage0_artifact_path(stage0_root, "stage0-m1")?;
    const EXEC_EVENTS_PER_COMPILE: usize = 2;
    let mes_expected = ExpectedAuditEvent {
        executable_path: &mes_path,
        digest_blake3: crate::stagex_mes::MES_M2_BLAKE3,
        inventory_entry_id: "planned:mes-m2-materialization:exec:mes-m2-smoke:mes-m2".to_string(),
    };
    let m1_expected = ExpectedAuditEvent {
        executable_path: &m1_path,
        digest_blake3: crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
            .iter()
            .find(|expected| expected.artifact_id == "stage0-m1")
            .expect("Stage0 M1 identity exists")
            .digest_blake3,
        inventory_entry_id: "planned:stage0-m1:exec:stage0-m1".to_string(),
    };
    let nyacc_count = usize::try_from(runtime.nyacc_command_count)
        .map_err(|_| StagexTransitionError::Audit("NYACC command count does not fit usize".to_string()))?;
    for event in &events[..nyacc_count] {
        validate_audit_event(event, &mes_expected)?;
    }
    let compile_events = &events[nyacc_count..];
    let (compile_event_pairs, remainder) = compile_events.as_chunks::<EXEC_EVENTS_PER_COMPILE>();
    for pair in compile_event_pairs {
        validate_audit_event(&pair[0], &mes_expected)?;
        validate_audit_event(&pair[1], &m1_expected)?;
    }
    if !remainder.is_empty() {
        return Err(StagexTransitionError::Audit("Mes runtime compile events do not form mes-m2/M1 pairs".to_string()));
    }
    let observed_compile_count = compile_events.len() / EXEC_EVENTS_PER_COMPILE;
    if observed_compile_count != usize::try_from(runtime.source_compile_count).unwrap_or(usize::MAX) {
        return Err(StagexTransitionError::Audit(format!(
            "Mes runtime compile event pairs {observed_compile_count} do not match report count {}",
            runtime.source_compile_count
        )));
    }
    assert_eq!(events.len(), expected_event_count);
    assert_eq!(observed_compile_count, usize::try_from(runtime.source_compile_count).unwrap());
    Ok(())
}

fn validate_tinycc_audit(
    stage0: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    mes_m2: &crate::stagex_mes::MesM2InventoryReport,
    tinycc: &crate::stagex_tinycc::TccMesInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_event_count = tinycc_expected_event_count(
        tinycc.compile_command_count,
        tinycc.runtime_refresh_command_count,
        tinycc.boot0_command_count,
        tinycc.final_command_count,
    )?;
    validate_tinycc_observed_event_count(expected_event_count, events.len())?;
    let stage0_root = stage0.mini.sources_root();
    let mes_path = report_output_path(&mes_m2.outputs, "mes-m2")?;
    let tcc_mes_path = tinycc_output_path(tinycc, "tcc-mes")?;
    let boot0_path = tinycc_output_path(tinycc, "tcc-boot0")?;
    let final_binary_path = tinycc_output_path(tinycc, "tinycc-0.9.26")?;
    let final_alias_path = final_binary_path
        .parent()
        .ok_or_else(|| StagexTransitionError::Audit("final TinyCC path has no parent".to_string()))?
        .join("tcc");
    let mut authorities = BTreeMap::<PathBuf, (String, String)>::new();
    authorities.insert(
        mes_path.clone(),
        (
            crate::stagex_mes::MES_M2_BLAKE3.to_string(),
            "planned:mes-m2-materialization:exec:mes-m2-smoke:mes-m2".to_string(),
        ),
    );
    for artifact_id in ["stage0-m1", "stage0-hex2"] {
        let path = stage0_artifact_path(stage0_root, artifact_id)?;
        let expected = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
            .iter()
            .find(|expected| expected.artifact_id == artifact_id)
            .expect("TinyCC Stage0 executable identity exists");
        authorities.insert(
            path,
            (
                expected.digest_blake3.to_string(),
                format!("planned:{}:exec:{}", expected.source_stage_id, expected.artifact_id),
            ),
        );
    }
    let blood_path = stage0_artifact_path(stage0_root, "stage0-full-blood-elf")?;
    authorities.insert(
        blood_path,
        (
            crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
                .iter()
                .find(|expected| expected.artifact_id == "stage0-full-blood-elf")
                .expect("full Stage0 blood-elf identity exists")
                .digest_blake3
                .to_string(),
            "planned:stage0-full-blood-elf:exec:stage0-full-blood-elf".to_string(),
        ),
    );
    authorities.insert(
        tcc_mes_path.clone(),
        (
            crate::stagex_tinycc::TCC_MES_BLAKE3.to_string(),
            "planned:tinycc-mes-materialization:exec:tinycc-runtime-refresh:tcc-mes".to_string(),
        ),
    );
    authorities.insert(
        boot0_path.clone(),
        (
            crate::stagex_tinycc::TCC_BOOT0_BLAKE3.to_string(),
            "planned:tinycc-boot0-materialization:exec:tinycc-boot0-smoke:tcc-boot0".to_string(),
        ),
    );
    authorities.insert(
        final_alias_path.clone(),
        (
            crate::stagex_tinycc::TINYCC_FINAL_BLAKE3.to_string(),
            "planned:tinycc-final-materialization:exec:tinycc-final-smoke:tinycc".to_string(),
        ),
    );
    let mut observed = BTreeSet::new();
    for event in events {
        let (expected_digest, expected_id) = authorities.get(&event.executable_path).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "TinyCC event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == event.executable_path
            && event.tracee_path == event.executable_path
            && event.digest_hex == *expected_digest
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "TinyCC event for {} did not match exact planned authority",
                event.executable_path.display()
            )));
        }
        observed.insert(event.executable_path.clone());
    }
    for required in [&mes_path, &tcc_mes_path, &boot0_path, &final_alias_path] {
        if !observed.contains(required) {
            return Err(StagexTransitionError::Audit(format!(
                "TinyCC audit did not observe required executable {}",
                required.display()
            )));
        }
    }
    assert!(!events.is_empty());
    assert!(observed.len() <= authorities.len());
    Ok(())
}

fn validate_tinycc27_audit(
    tinycc: &crate::stagex_tinycc::TccMesInventoryReport,
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = tinycc27_expected_event_count(tinycc27)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC 0.9.27 protected audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let tinycc26_binary = tinycc_output_path(tinycc, "tinycc-0.9.26")?;
    let tinycc26_alias = tinycc26_binary
        .parent()
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC 0.9.26 path has no parent".to_string()))?
        .join("tcc");
    let tinycc27_alias = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let predecessor_count = tinycc27_predecessor_event_count(tinycc27)?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < predecessor_count {
            (
                &tinycc26_alias,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
                "planned:tinycc-final-materialization:exec:tinycc-final-smoke:tinycc",
            )
        } else {
            (
                &tinycc27_alias,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )
        };
        validate_tinycc27_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(predecessor_count < expected_count);
    Ok(())
}

fn validate_tinycc27_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC 0.9.27 event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn tinycc27_predecessor_event_count(
    report: &crate::stagex_tinycc27::Tinycc27InventoryReport,
) -> Result<usize, StagexTransitionError> {
    let count = report
        .runtime_command_count
        .checked_add(report.build_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC 0.9.27 predecessor count overflow".to_string()))?;
    usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("TinyCC 0.9.27 predecessor count does not fit usize".to_string()))
}

fn tinycc27_expected_event_count(
    report: &crate::stagex_tinycc27::Tinycc27InventoryReport,
) -> Result<usize, StagexTransitionError> {
    let command_count = report
        .runtime_command_count
        .checked_add(report.build_command_count)
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC 0.9.27 command count overflow".to_string()))?;
    usize::try_from(command_count)
        .map_err(|_| StagexTransitionError::Audit("TinyCC 0.9.27 command count does not fit usize".to_string()))
}

fn validate_tcc_musl_v2_audit(
    tinycc: &crate::stagex_tinycc::TccMesInventoryReport,
    v2: &crate::stagex_tcc_musl_v2::InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = tcc_musl_v2_expected_event_count(v2)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC musl-v2 audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let build_end = usize::try_from(v2.build_command_count).unwrap_or(usize::MAX);
    let positive_index = build_end
        .checked_add(3)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC musl-v2 positive index overflow".to_string()))?;
    let tinycc_path = tinycc_output_path(tinycc, "tinycc-0.9.26")?;
    let compiler_path = tcc_musl_v2_output_path(v2, "tcc-musl-v2")?;
    let positive_path = tcc_musl_v2_output_path(v2, "tcc-musl-v2-positive-binary")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < build_end {
            (
                &tinycc_path,
                crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
                "planned:tinycc-final-materialization:exec:tcc-musl-v2-materialization:tinycc26",
            )
        } else if index == positive_index {
            (
                &positive_path,
                crate::stagex_tcc_musl_v2::POSITIVE_BINARY_BLAKE3,
                "planned:tcc-musl-v2-smoke:exec:tcc-musl-v2-execution:positive",
            )
        } else {
            (
                &compiler_path,
                crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
                "planned:tcc-musl-v2-materialization:exec:tcc-musl-v2-smoke:tcc",
            )
        };
        validate_coreutils_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert_eq!(positive_index, 7);
    Ok(())
}

fn tcc_musl_v2_expected_event_count(
    report: &crate::stagex_tcc_musl_v2::InventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COUNT: u32 = 4;
    const EXPECTED_SMOKE_COUNT: u32 = 5;
    if report.build_command_count != EXPECTED_BUILD_COUNT || report.smoke_command_count != EXPECTED_SMOKE_COUNT {
        return Err(StagexTransitionError::Audit(
            "TinyCC musl-v2 report has substituted build or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC musl-v2 event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("TinyCC musl-v2 event count does not fit usize".to_string()))
}

fn validate_musl_pass2_audit(
    tcc_musl: &crate::stagex_tcc_musl::TccMuslInventoryReport,
    musl_pass2: &crate::stagex_musl::MuslInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = musl_expected_event_count(musl_pass2)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "second-musl audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let compiler_path = tcc_musl_output_path(tcc_musl, "tcc-musl")?;
    for event in events {
        validate_coreutils_event(
            event,
            &compiler_path,
            crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
            "planned:tcc-musl-materialization:exec:tcc-musl-smoke:tcc",
        )?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(expected_count > 1);
    Ok(())
}

fn validate_tcc_musl_audit(
    prep: &crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport,
    tcc_musl: &crate::stagex_tcc_musl::TccMuslInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = tcc_musl_expected_event_count(tcc_musl)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "musl-linked TinyCC audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let build_end = usize::try_from(tcc_musl.build_command_count).unwrap_or(usize::MAX);
    let prep_path = tcc_musl_prep_output_path(prep, "tcc-musl-prep")?;
    let compiler_path = tcc_musl_output_path(tcc_musl, "tcc-musl")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < build_end {
            (
                &prep_path,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
                "planned:tcc-musl-prep-materialization:exec:tcc-musl-prep-smoke:tcc",
            )
        } else {
            (
                &compiler_path,
                crate::stagex_tcc_musl::TCC_MUSL_FINAL_BLAKE3,
                "planned:tcc-musl-materialization:exec:tcc-musl-smoke:tcc",
            )
        };
        validate_coreutils_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert_eq!(build_end, 2);
    Ok(())
}

fn tcc_musl_expected_event_count(
    report: &crate::stagex_tcc_musl::TccMuslInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COUNT: u32 = 2;
    const EXPECTED_SMOKE_COUNT: u32 = 4;
    if report.build_command_count != EXPECTED_BUILD_COUNT || report.smoke_command_count != EXPECTED_SMOKE_COUNT {
        return Err(StagexTransitionError::Audit(
            "musl-linked TinyCC report has substituted build or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("musl-linked TinyCC event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("musl-linked TinyCC event count does not fit usize".to_string()))
}

fn validate_musl_audit(
    prep: &crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport,
    musl: &crate::stagex_musl::MuslInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = musl_expected_event_count(musl)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "musl protected audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let prep_path = tcc_musl_prep_output_path(prep, "tcc-musl-prep")?;
    for event in events {
        validate_coreutils_event(
            event,
            &prep_path,
            crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
            "planned:tcc-musl-prep-materialization:exec:tcc-musl-prep-smoke:tcc",
        )?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(expected_count > 0);
    Ok(())
}

fn musl_expected_event_count(report: &crate::stagex_musl::MuslInventoryReport) -> Result<usize, StagexTransitionError> {
    const FIRST_FORMAT: &str = "mantle-stagex-musl-1.1.24-inventory-v1";
    const SECOND_FORMAT: &str = "mantle-stagex-musl-1.1.24-pass2-inventory-v1";
    const FIRST_HELPER_COUNT: u32 = 24;
    const FIRST_SOURCE_COMPILE_COUNT: u32 = 215;
    const SECOND_HELPER_COUNT: u32 = 28;
    const SECOND_SOURCE_COMPILE_COUNT: u32 = 219;
    const EXPECTED_CRT_COMPILE_COUNT: u32 = 3;
    const EXPECTED_ARCHIVE_COUNT: u32 = 1;
    const EXPECTED_SMOKE_COUNT: u32 = 3;
    let (expected_helper_count, expected_source_compile_count) = match report.format {
        FIRST_FORMAT => (FIRST_HELPER_COUNT, FIRST_SOURCE_COMPILE_COUNT),
        SECOND_FORMAT => (SECOND_HELPER_COUNT, SECOND_SOURCE_COMPILE_COUNT),
        other => {
            return Err(StagexTransitionError::Audit(format!("musl report has unsupported format {other}")));
        }
    };
    if report.helper_count != expected_helper_count
        || report.source_compile_count != expected_source_compile_count
        || report.crt_compile_count != EXPECTED_CRT_COMPILE_COUNT
        || report.archive_command_count != EXPECTED_ARCHIVE_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "musl report has substituted helper, compile, archive, or smoke counts".to_string(),
        ));
    }
    let total = report
        .source_compile_count
        .checked_add(report.crt_compile_count)
        .and_then(|count| count.checked_add(report.archive_command_count))
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("musl event count overflow".to_string()))?;
    usize::try_from(total).map_err(|_| StagexTransitionError::Audit("musl event count does not fit usize".to_string()))
}

fn validate_tcc_musl_prep_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    prep: &crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = tcc_musl_prep_expected_event_count(prep)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC musl-prep audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let build_end = usize::try_from(prep.build_command_count).unwrap_or(usize::MAX);
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let prep_path = tcc_musl_prep_output_path(prep, "tcc-musl-prep")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < build_end {
            (
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )
        } else {
            (
                &prep_path,
                crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
                "planned:tcc-musl-prep-materialization:exec:tcc-musl-prep-smoke:tcc",
            )
        };
        validate_coreutils_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert_eq!(build_end, 2);
    Ok(())
}

fn tcc_musl_prep_expected_event_count(
    report: &crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_BUILD_COUNT: u32 = 2;
    const EXPECTED_SMOKE_COUNT: u32 = 3;
    if report.build_command_count != EXPECTED_BUILD_COUNT || report.smoke_command_count != EXPECTED_SMOKE_COUNT {
        return Err(StagexTransitionError::Audit(
            "TinyCC musl-prep report has substituted build or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC musl-prep event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("TinyCC musl-prep event count does not fit usize".to_string()))
}

fn validate_bash_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    bash: &crate::stagex_bash::BashInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = bash_expected_event_count(bash)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "bash protected audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let build_end = usize::try_from(bash.build_command_count).unwrap_or(usize::MAX);
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let bash_path = bash_output_path(bash, "bash-2.05b")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < build_end {
            (
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )
        } else {
            (
                &bash_path,
                crate::stagex_bash::BASH_FINAL_BLAKE3,
                "planned:bash-materialization:exec:bash-smoke:bash",
            )
        };
        validate_coreutils_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert_eq!(build_end, 93);
    Ok(())
}

fn bash_expected_event_count(report: &crate::stagex_bash::BashInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_HELPER_COUNT: u32 = 10;
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 92;
    const EXPECTED_BUILD_COUNT: u32 = 93;
    const EXPECTED_SMOKE_COUNT: u32 = 3;
    if report.helper_count != EXPECTED_HELPER_COUNT
        || report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "bash report has substituted helper, compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("bash event count overflow".to_string()))?;
    usize::try_from(total).map_err(|_| StagexTransitionError::Audit("bash event count does not fit usize".to_string()))
}

fn validate_oyacc_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    patch: &crate::stagex_gnu_patch::GnuPatchInventoryReport,
    oyacc: &crate::stagex_oyacc::OyaccInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = oyacc_expected_event_count(oyacc)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "oyacc protected audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let patch_end = usize::try_from(oyacc.patch_command_count).unwrap_or(usize::MAX);
    let build_end = patch_end
        .checked_add(usize::try_from(oyacc.build_command_count).unwrap_or(usize::MAX))
        .ok_or_else(|| StagexTransitionError::Audit("oyacc build boundary overflow".to_string()))?;
    let patch_path = gnu_patch_output_path(patch, "gnu-patch-2.5.9")?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let yacc_path = oyacc_output_path(oyacc, "oyacc-6.6")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < patch_end {
            (
                &patch_path,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
                "planned:gnu-patch-materialization:exec:gnu-patch-smoke:patch",
            )
        } else if index < build_end {
            (
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )
        } else {
            (
                &yacc_path,
                crate::stagex_oyacc::OYACC_FINAL_BLAKE3,
                "planned:oyacc-materialization:exec:oyacc-smoke:yacc",
            )
        };
        validate_coreutils_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(patch_end < build_end);
    Ok(())
}

fn oyacc_expected_event_count(
    report: &crate::stagex_oyacc::OyaccInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_PATCH_COUNT: u32 = 2;
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 13;
    const EXPECTED_BUILD_COUNT: u32 = 14;
    const EXPECTED_SMOKE_COUNT: u32 = 2;
    if report.patch_command_count != EXPECTED_PATCH_COUNT
        || report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "oyacc report has substituted patch, compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .patch_command_count
        .checked_add(report.build_command_count)
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("oyacc event count overflow".to_string()))?;
    usize::try_from(total).map_err(|_| StagexTransitionError::Audit("oyacc event count does not fit usize".to_string()))
}

fn validate_coreutils_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    patch: &crate::stagex_gnu_patch::GnuPatchInventoryReport,
    coreutils: &crate::stagex_coreutils::CoreutilsInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = coreutils_expected_event_count(coreutils)?;
    if events.len() != expected_count {
        return Err(StagexTransitionError::Audit(format!(
            "coreutils protected audit expected {expected_count} events, observed {}",
            events.len()
        )));
    }
    let patch_end = usize::try_from(coreutils.patch_command_count).unwrap_or(usize::MAX);
    let build_end = patch_end
        .checked_add(usize::try_from(coreutils.build_command_count).unwrap_or(usize::MAX))
        .ok_or_else(|| StagexTransitionError::Audit("coreutils build boundary overflow".to_string()))?;
    let patch_path = gnu_patch_output_path(patch, "gnu-patch-2.5.9")?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let smoke = coreutils_smoke_audit_authority(coreutils)?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < patch_end {
            (
                &patch_path,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
                "planned:gnu-patch-materialization:exec:gnu-patch-smoke:patch".to_string(),
            )
        } else if index < build_end {
            (
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27".to_string(),
            )
        } else {
            let smoke_index = index.saturating_sub(build_end);
            let (name, path, digest) = smoke.get(smoke_index).ok_or_else(|| {
                StagexTransitionError::Audit(format!("undeclared coreutils smoke event index {smoke_index}"))
            })?;
            (path, digest.as_str(), format!("planned:coreutils-materialization:exec:coreutils-smoke:{name}"))
        };
        validate_coreutils_event(event, path, digest, &inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(patch_end < build_end);
    Ok(())
}

fn coreutils_expected_event_count(
    report: &crate::stagex_coreutils::CoreutilsInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_PATCH_COUNT: u32 = 8;
    const EXPECTED_LIBRARY_COMPILE_COUNT: u32 = 96;
    const EXPECTED_ARCHIVE_COUNT: u32 = 1;
    const EXPECTED_UTILITY_COMPILE_COUNT: u32 = 34;
    const EXPECTED_UTILITY_LINK_COUNT: u32 = 25;
    const EXPECTED_BUILD_COUNT: u32 = 156;
    const EXPECTED_SMOKE_COUNT: u32 = 6;
    let counts_match = report.patch_command_count == EXPECTED_PATCH_COUNT
        && report.library_compile_count == EXPECTED_LIBRARY_COMPILE_COUNT
        && report.archive_command_count == EXPECTED_ARCHIVE_COUNT
        && report.utility_source_compile_count == EXPECTED_UTILITY_COMPILE_COUNT
        && report.utility_link_count == EXPECTED_UTILITY_LINK_COUNT
        && report.build_command_count == EXPECTED_BUILD_COUNT
        && report.smoke_command_count == EXPECTED_SMOKE_COUNT;
    if !counts_match {
        return Err(StagexTransitionError::Audit(
            "coreutils report has substituted patch, compile, archive, link, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .patch_command_count
        .checked_add(report.build_command_count)
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("coreutils event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("coreutils event count does not fit usize".to_string()))
}

fn coreutils_smoke_audit_authority(
    report: &crate::stagex_coreutils::CoreutilsInventoryReport,
) -> Result<Vec<(&'static str, PathBuf, String)>, StagexTransitionError> {
    let sequence = ["mkdir", "echo", "cp", "cat", "test", "cp"];
    let mut authority = Vec::with_capacity(sequence.len());
    for name in sequence {
        let artifact_id = format!("coreutils-{name}");
        let output = report.outputs.iter().find(|output| output.artifact_id == artifact_id).ok_or_else(|| {
            StagexTransitionError::Audit(format!("coreutils report lacks smoke executable {artifact_id}"))
        })?;
        authority.push((name, output.path.clone(), output.digest_blake3.clone()));
    }
    assert_eq!(authority.len(), sequence.len());
    assert!(authority.iter().all(|(_, path, _)| path.is_absolute()));
    Ok(authority)
}

fn validate_coreutils_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "coreutils event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_bzip2_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    sed: &crate::stagex_sed::SedInventoryReport,
    bzip2: &crate::stagex_bzip2::Bzip2InventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = bzip2_expected_event_count(bzip2)?;
    validate_bzip2_observed_event_count(expected_count, events.len())?;
    let rewrite_end = usize::try_from(bzip2.rewrite_command_count)
        .map_err(|_| StagexTransitionError::Audit("bzip2 rewrite count does not fit usize".to_string()))?;
    let build_end = rewrite_end
        .checked_add(usize::try_from(bzip2.build_command_count).unwrap_or(usize::MAX))
        .ok_or_else(|| StagexTransitionError::Audit("bzip2 build boundary overflow".to_string()))?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let sed_path = sed_output_path(sed, "sed-4.0.9")?;
    let bzip2_path = bzip2_output_path(bzip2, "bzip2-1.0.8")?;
    for (index, event) in events.iter().enumerate() {
        let (path, digest, inventory_id) = if index < rewrite_end {
            (&sed_path, crate::stagex_sed::SED_FINAL_BLAKE3, "planned:sed-materialization:exec:sed-smoke:sed")
        } else if index < build_end {
            (
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )
        } else {
            (
                &bzip2_path,
                crate::stagex_bzip2::BZIP2_FINAL_BLAKE3,
                "planned:bzip2-materialization:exec:bzip2-smoke:bzip2",
            )
        };
        validate_bzip2_event(event, path, digest, inventory_id)?;
    }
    assert_eq!(events.len(), expected_count);
    assert!(build_end < expected_count);
    Ok(())
}

fn bzip2_expected_event_count(
    report: &crate::stagex_bzip2::Bzip2InventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_REWRITE_COUNT: u32 = 3;
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 9;
    const EXPECTED_BUILD_COUNT: u32 = 11;
    const EXPECTED_SMOKE_COUNT: u32 = 4;
    if report.rewrite_command_count != EXPECTED_REWRITE_COUNT
        || report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != EXPECTED_BUILD_COUNT
        || report.smoke_command_count != EXPECTED_SMOKE_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "bzip2 report has substituted rewrite, compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .rewrite_command_count
        .checked_add(report.build_command_count)
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("bzip2 event count overflow".to_string()))?;
    usize::try_from(total).map_err(|_| StagexTransitionError::Audit("bzip2 event count does not fit usize".to_string()))
}

fn validate_bzip2_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if expected_count == observed_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "bzip2 protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_bzip2_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "bzip2 event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_sed_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    sed: &crate::stagex_sed::SedInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = sed_expected_event_count(sed)?;
    validate_sed_observed_event_count(expected_count, events.len())?;
    let build_count = usize::try_from(sed.build_command_count)
        .map_err(|_| StagexTransitionError::Audit("GNU sed build count does not fit usize".to_string()))?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let sed_path = sed_output_path(sed, "sed-4.0.9")?;
    for (index, event) in events.iter().enumerate() {
        if index < build_count {
            validate_sed_event(
                event,
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )?;
        } else {
            validate_sed_event(
                event,
                &sed_path,
                crate::stagex_sed::SED_FINAL_BLAKE3,
                "planned:sed-materialization:exec:sed-smoke:sed",
            )?;
        }
    }
    assert_eq!(events.len(), expected_count);
    assert!(build_count < expected_count);
    Ok(())
}

fn sed_expected_event_count(report: &crate::stagex_sed::SedInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 13;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
    let expected_build_count = report
        .source_compile_count
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("GNU sed build count overflow".to_string()))?;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != expected_build_count
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "GNU sed report has substituted compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU sed event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("GNU sed event count does not fit usize".to_string()))
}

fn validate_sed_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if expected_count == observed_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "GNU sed protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_sed_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "GNU sed event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_tar_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    tar: &crate::stagex_tar::TarInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = tar_expected_event_count(tar)?;
    validate_tar_observed_event_count(expected_count, events.len())?;
    let build_count = usize::try_from(tar.build_command_count)
        .map_err(|_| StagexTransitionError::Audit("GNU tar build count does not fit usize".to_string()))?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let tar_path = tar_output_path(tar, "tar-1.12")?;
    for (index, event) in events.iter().enumerate() {
        if index < build_count {
            validate_tar_event(
                event,
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )?;
        } else {
            validate_tar_event(
                event,
                &tar_path,
                crate::stagex_tar::TAR_FINAL_BLAKE3,
                "planned:tar-materialization:exec:tar-smoke:tar",
            )?;
        }
    }
    assert_eq!(events.len(), expected_count);
    assert!(build_count < expected_count);
    Ok(())
}

fn tar_expected_event_count(report: &crate::stagex_tar::TarInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 29;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 4;
    let expected_build_count = report
        .source_compile_count
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("GNU tar build count overflow".to_string()))?;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != expected_build_count
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "GNU tar report has substituted compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU tar event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("GNU tar event count does not fit usize".to_string()))
}

fn validate_tar_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if expected_count == observed_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "GNU tar protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_tar_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "GNU tar event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_gzip_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    gzip: &crate::stagex_gzip::GzipInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = gzip_expected_event_count(gzip)?;
    validate_gzip_observed_event_count(expected_count, events.len())?;
    let generator_build_count = usize::try_from(gzip.generator_build_command_count)
        .map_err(|_| StagexTransitionError::Audit("gzip generator build count does not fit usize".to_string()))?;
    let generator_run_end = generator_build_count
        .checked_add(usize::try_from(gzip.generator_run_command_count).unwrap_or(usize::MAX))
        .ok_or_else(|| StagexTransitionError::Audit("gzip generator boundary overflow".to_string()))?;
    let build_end = generator_run_end
        .checked_add(usize::try_from(gzip.build_command_count).unwrap_or(usize::MAX))
        .ok_or_else(|| StagexTransitionError::Audit("gzip build boundary overflow".to_string()))?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let makecrc_path = gzip_output_path(gzip, "gzip-makecrc")?;
    let gzip_path = gzip_output_path(gzip, "gzip-1.2.4")?;
    let gunzip_path = gzip_output_path(gzip, "gunzip-1.2.4")?;
    for (index, event) in events.iter().enumerate() {
        if index < generator_build_count || (index >= generator_run_end && index < build_end) {
            validate_gzip_event(
                event,
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )?;
        } else if index < generator_run_end {
            validate_gzip_event(
                event,
                &makecrc_path,
                crate::stagex_gzip::GZIP_MAKECRC_BLAKE3,
                "planned:gzip-generator-materialization:exec:gzip-generator-execution:makecrc",
            )?;
        } else if index < build_end.saturating_add(GZIP_GZIP_SMOKE_EVENT_COUNT) {
            validate_gzip_event(
                event,
                &gzip_path,
                crate::stagex_gzip::GZIP_FINAL_BLAKE3,
                "planned:gzip-materialization:exec:gzip-smoke:gzip",
            )?;
        } else {
            validate_gzip_event(
                event,
                &gunzip_path,
                crate::stagex_gzip::GZIP_FINAL_BLAKE3,
                "planned:gzip-materialization:exec:gzip-smoke:gunzip",
            )?;
        }
    }
    assert_eq!(events.len(), expected_count);
    assert!(build_end < expected_count);
    Ok(())
}

fn gzip_expected_event_count(report: &crate::stagex_gzip::GzipInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 14;
    const EXPECTED_GENERATOR_COMMAND_COUNT: u32 = 1;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 4;
    let expected_build_count = report
        .source_compile_count
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("gzip build count overflow".to_string()))?;
    let counts_match = report.source_compile_count == EXPECTED_SOURCE_COMPILE_COUNT
        && report.generator_build_command_count == EXPECTED_GENERATOR_COMMAND_COUNT
        && report.generator_run_command_count == EXPECTED_GENERATOR_COMMAND_COUNT
        && report.build_command_count == expected_build_count
        && report.smoke_command_count == EXPECTED_SMOKE_COMMAND_COUNT;
    if !counts_match {
        return Err(StagexTransitionError::Audit(
            "gzip report has substituted generator, compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .generator_build_command_count
        .checked_add(report.generator_run_command_count)
        .and_then(|count| count.checked_add(report.build_command_count))
        .and_then(|count| count.checked_add(report.smoke_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("gzip event count overflow".to_string()))?;
    usize::try_from(total).map_err(|_| StagexTransitionError::Audit("gzip event count does not fit usize".to_string()))
}

fn validate_gzip_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if expected_count == observed_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "gzip protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_gzip_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "gzip event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_gnu_patch_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    patch: &crate::stagex_gnu_patch::GnuPatchInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected_count = gnu_patch_expected_event_count(patch)?;
    validate_gnu_patch_observed_event_count(expected_count, events.len())?;
    let build_count = usize::try_from(patch.build_command_count)
        .map_err(|_| StagexTransitionError::Audit("GNU patch build count does not fit usize".to_string()))?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let patch_path = gnu_patch_output_path(patch, "gnu-patch-2.5.9")?;
    for (index, event) in events.iter().enumerate() {
        if index < build_count {
            validate_gnu_patch_event(
                event,
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )?;
        } else {
            validate_gnu_patch_event(
                event,
                &patch_path,
                crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
                "planned:gnu-patch-materialization:exec:gnu-patch-smoke:patch",
            )?;
        }
    }
    assert_eq!(events.len(), expected_count);
    assert!(build_count < expected_count);
    Ok(())
}

fn gnu_patch_expected_event_count(
    report: &crate::stagex_gnu_patch::GnuPatchInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 19;
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
    let expected_build_count = report
        .source_compile_count
        .checked_add(1)
        .ok_or_else(|| StagexTransitionError::Audit("GNU patch build count overflow".to_string()))?;
    if report.source_compile_count != EXPECTED_SOURCE_COMPILE_COUNT
        || report.build_command_count != expected_build_count
        || report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT
    {
        return Err(StagexTransitionError::Audit(
            "GNU patch report has substituted compile, build, or smoke counts".to_string(),
        ));
    }
    let total = report
        .build_command_count
        .checked_add(report.smoke_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU patch event count overflow".to_string()))?;
    usize::try_from(total)
        .map_err(|_| StagexTransitionError::Audit("GNU patch event count does not fit usize".to_string()))
}

fn validate_gnu_patch_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if expected_count == observed_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "GNU patch protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_gnu_patch_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "GNU patch event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn validate_make_audit(
    tinycc27: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    make: &crate::stagex_make::MakeInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const MAKE_VERSION_OFFSET: usize = 0;
    const MAKE_RECIPE_OFFSET: usize = 1;
    const RECIPE_RUNNER_OFFSET: usize = 2;
    const MAKE_MALFORMED_OFFSET: usize = 3;
    let expected_count = make_expected_event_count(make)?;
    validate_make_observed_event_count(expected_count, events.len())?;
    let predecessor_count = make_predecessor_event_count(make)?;
    let tinycc27_path = tinycc27_output_path(tinycc27, "tinycc27-alias")?;
    let make_path = make_output_path(make, "make-3.82")?;
    let runner_path = make_output_path(make, "make-recipe-runner")?;
    for (index, event) in events.iter().enumerate() {
        if index < predecessor_count {
            validate_make_event(
                event,
                &tinycc27_path,
                crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
                "planned:tinycc27-materialization:exec:tinycc27-smoke:tinycc27",
            )?;
            continue;
        }
        let smoke_offset = index.saturating_sub(predecessor_count);
        match smoke_offset {
            MAKE_VERSION_OFFSET | MAKE_RECIPE_OFFSET | MAKE_MALFORMED_OFFSET => validate_make_event(
                event,
                &make_path,
                crate::stagex_make::MAKE_FINAL_BLAKE3,
                "planned:make-materialization:exec:make-smoke:make",
            )?,
            RECIPE_RUNNER_OFFSET => validate_make_event(
                event,
                &runner_path,
                crate::stagex_make::MAKE_RECIPE_RUNNER_BLAKE3,
                "planned:make-recipe-runner-materialization:exec:make-smoke:recipe-runner",
            )?,
            _ => {
                return Err(StagexTransitionError::Audit(format!(
                    "GNU Make audit has undeclared smoke offset {smoke_offset}"
                )));
            }
        }
    }
    assert_eq!(events.len(), expected_count);
    assert!(predecessor_count < expected_count);
    Ok(())
}

fn validate_make_observed_event_count(
    expected_count: usize,
    observed_count: usize,
) -> Result<(), StagexTransitionError> {
    if observed_count == expected_count {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "GNU Make protected audit expected {expected_count} events, observed {observed_count}"
    )))
}

fn validate_make_event(
    event: &ProtectedSeccompAuditEvent,
    expected_path: &Path,
    expected_digest: &str,
    expected_inventory_id: &str,
) -> Result<(), StagexTransitionError> {
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected_path
        && event.tracee_path == expected_path
        && event.resolved_host_path == expected_path
        && event.digest_hex == expected_digest
        && event.inventory_entry_id.as_deref() == Some(expected_inventory_id);
    if !valid {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Make event for {} did not match exact planned authority",
            event.executable_path.display()
        )));
    }
    assert_eq!(event.executable_path, expected_path);
    assert_eq!(event.digest_hex, expected_digest);
    Ok(())
}

fn make_predecessor_event_count(
    report: &crate::stagex_make::MakeInventoryReport,
) -> Result<usize, StagexTransitionError> {
    const EXPECTED_RECIPE_RUNNER_COMMAND_COUNT: u32 = 1;
    if report.recipe_runner_command_count != EXPECTED_RECIPE_RUNNER_COMMAND_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Make recipe-runner command count is {}, expected {EXPECTED_RECIPE_RUNNER_COMMAND_COUNT}",
            report.recipe_runner_command_count
        )));
    }
    let count = report
        .recipe_runner_command_count
        .checked_add(report.build_command_count)
        .ok_or_else(|| StagexTransitionError::Audit("GNU Make predecessor count overflow".to_string()))?;
    usize::try_from(count)
        .map_err(|_| StagexTransitionError::Audit("GNU Make predecessor count does not fit usize".to_string()))
}

fn make_expected_event_count(report: &crate::stagex_make::MakeInventoryReport) -> Result<usize, StagexTransitionError> {
    const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
    if report.smoke_command_count != EXPECTED_SMOKE_COMMAND_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "GNU Make smoke command count is {}, expected {EXPECTED_SMOKE_COMMAND_COUNT}",
            report.smoke_command_count
        )));
    }
    let predecessor_count = make_predecessor_event_count(report)?;
    predecessor_count
        .checked_add(usize::try_from(report.smoke_command_count).unwrap_or(usize::MAX))
        .and_then(|count| count.checked_add(MAKE_RECIPE_CHILD_EVENT_COUNT))
        .ok_or_else(|| StagexTransitionError::Audit("GNU Make expected event count overflow".to_string()))
}

fn tinycc_expected_event_count(
    compile_command_count: u32,
    runtime_refresh_command_count: u32,
    boot0_command_count: u32,
    final_command_count: u32,
) -> Result<usize, StagexTransitionError> {
    let command_count = compile_command_count
        .checked_add(runtime_refresh_command_count)
        .and_then(|count| count.checked_add(boot0_command_count))
        .and_then(|count| count.checked_add(final_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC reported command count overflow".to_string()))?;
    usize::try_from(command_count)
        .map_err(|_| StagexTransitionError::Audit("TinyCC command count does not fit usize".to_string()))?
        .checked_add(TINYCC_LINK_CHILD_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("TinyCC expected event count overflow".to_string()))
}

fn validate_tinycc_observed_event_count(
    expected_event_count: usize,
    observed_event_count: usize,
) -> Result<(), StagexTransitionError> {
    if observed_event_count != expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "TinyCC protected audit expected {expected_event_count} events, observed {observed_event_count}"
        )));
    }
    assert!(expected_event_count > 0);
    assert_eq!(observed_event_count, expected_event_count);
    Ok(())
}

fn report_output_path(
    outputs: &[crate::stagex_mes::MesM2Output],
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("Mes report lacks output {artifact_id}")))
}

fn tinycc_output_path(
    report: &crate::stagex_tinycc::TccMesInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("TinyCC report lacks output {artifact_id}")))
}

fn tinycc27_output_path(
    report: &crate::stagex_tinycc27::Tinycc27InventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("TinyCC 0.9.27 report lacks output {artifact_id}")))
}

fn tcc_musl_v2_output_path(
    report: &crate::stagex_tcc_musl_v2::InventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("TinyCC musl-v2 report lacks output {artifact_id}")))
}

fn tcc_selfhost_output_path(
    report: &crate::stagex_tcc_selfhost::InventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("TinyCC self-host report lacks output {artifact_id}")))
}

fn m4_output_path(
    report: &crate::stagex_m4::M4InventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU M4 report lacks output {artifact_id}")))
}

fn diffutils_output_path(
    report: &crate::stagex_diffutils::DiffutilsInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU diffutils report lacks output {artifact_id}")))
}

fn tcc_musl_output_path(
    report: &crate::stagex_tcc_musl::TccMuslInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("musl-linked TinyCC report lacks output {artifact_id}")))
}

fn tcc_musl_prep_output_path(
    report: &crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("TinyCC musl-prep report lacks output {artifact_id}")))
}

fn bash_output_path(
    report: &crate::stagex_bash::BashInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("bash report lacks output {artifact_id}")))
}

fn oyacc_output_path(
    report: &crate::stagex_oyacc::OyaccInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("oyacc report lacks output {artifact_id}")))
}

fn bzip2_output_path(
    report: &crate::stagex_bzip2::Bzip2InventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("bzip2 report lacks output {artifact_id}")))
}

fn sed_output_path(
    report: &crate::stagex_sed::SedInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU sed report lacks output {artifact_id}")))
}

fn tar_output_path(
    report: &crate::stagex_tar::TarInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU tar report lacks output {artifact_id}")))
}

fn gzip_output_path(
    report: &crate::stagex_gzip::GzipInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("gzip report lacks output {artifact_id}")))
}

fn gnu_patch_output_path(
    report: &crate::stagex_gnu_patch::GnuPatchInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU patch report lacks output {artifact_id}")))
}

fn make_output_path(
    report: &crate::stagex_make::MakeInventoryReport,
    artifact_id: &str,
) -> Result<PathBuf, StagexTransitionError> {
    report
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.path.clone())
        .ok_or_else(|| StagexTransitionError::Audit(format!("GNU Make report lacks output {artifact_id}")))
}

fn stage0_artifact_path(root: &Path, artifact_id: &str) -> Result<PathBuf, StagexTransitionError> {
    if let Some(expected) = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok(root.join(expected.relative_path));
    }
    if let Some(expected) = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok(root.join(expected.relative_path));
    }
    Err(StagexTransitionError::Audit(format!("unknown Stage0 artifact {artifact_id}")))
}

fn validate_audit_event(
    event: &ProtectedSeccompAuditEvent,
    expected: &ExpectedAuditEvent<'_>,
) -> Result<(), StagexTransitionError> {
    let expected_id = Some(expected.inventory_entry_id.as_str());
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected.executable_path
        && event.resolved_host_path == expected.executable_path
        && event.digest_hex == expected.digest_blake3
        && event.inventory_entry_id.as_deref() == expected_id;
    if valid {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "event for {} did not match its path, digest, source stage, or allow decision",
        expected.executable_path.display()
    )))
}

fn read_checked_input(
    path: &Path,
    role: &str,
    expected_digest: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error(&format!("reading StageX {role}"), source))?;
    validate_size(role, bytes.len(), max_bytes)?;
    validate_bytes_digest(role, &bytes, expected_digest)?;
    Ok(bytes)
}

fn read_checked_output(path: &Path, role: &str, max_bytes: u64) -> Result<Vec<u8>, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error(&format!("reading StageX {role}"), source))?;
    validate_size(role, bytes.len(), max_bytes)?;
    Ok(bytes)
}

fn validate_size(role: &str, len: usize, max_bytes: u64) -> Result<(), StagexTransitionError> {
    let actual_bytes = u64::try_from(len)
        .map_err(|_| StagexTransitionError::InvalidInput(format!("{role} byte length exceeds u64")))?;
    if actual_bytes > max_bytes {
        return Err(StagexTransitionError::SizeLimit {
            role: role.to_string(),
            actual_bytes,
            limit_bytes: max_bytes,
        });
    }
    Ok(())
}

fn validate_bytes_digest(role: &str, bytes: &[u8], expected: &str) -> Result<(), StagexTransitionError> {
    let actual = blake3::hash(bytes).to_hex().to_string();
    if actual == expected {
        return Ok(());
    }
    Err(StagexTransitionError::DigestMismatch {
        role: role.to_string(),
        expected: expected.to_string(),
        actual,
    })
}

fn write_file_create_new(path: &Path, bytes: &[u8]) -> Result<(), StagexTransitionError> {
    use std::io::Write;
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error(&format!("creating {}", path.display()), source))?;
    file.write_all(bytes).map_err(|source| io_error(&format!("writing {}", path.display()), source))?;
    file.sync_all().map_err(|source| io_error(&format!("syncing {}", path.display()), source))?;
    Ok(())
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), StagexTransitionError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| StagexTransitionError::InvalidInput(format!("serializing {}: {error}", path.display())))?;
    bytes.push(b'\n');
    write_file_create_new(path, &bytes)
}

fn set_executable_owner_only(path: &Path) -> Result<(), StagexTransitionError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|source| io_error(&format!("reading permissions for {}", path.display()), source))?
        .permissions();
    permissions.set_mode(EXECUTABLE_MODE_OWNER_ONLY);
    fs::set_permissions(path, permissions)
        .map_err(|source| io_error(&format!("setting executable mode for {}", path.display()), source))
}

fn io_error(action: &str, source: io::Error) -> StagexTransitionError {
    StagexTransitionError::Io {
        action: action.to_string(),
        source,
    }
}

fn protected_digest_error(error: crate::protected_exec::Stage0InventoryGenerationError) -> StagexTransitionError {
    StagexTransitionError::ProtectedExec(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHILD_ENV: &str = "MANTLE_STAGE_X_TRANSITION_CHILD";
    const SCRATCH_ENV: &str = "MANTLE_STAGE_X_TRANSITION_SCRATCH";
    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const CHILD_TEST_NAME: &str = "stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem";
    const TEST_LINEAGE_MANIFEST_DIGEST: &str = "e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d";

    #[test]
    fn protected_transition_reproduces_seed_and_builds_kaem() {
        if std::env::var_os(CHILD_ENV).is_some() {
            run_protected_transition_child();
            return;
        }
        let current_exe = std::env::current_exe().unwrap();
        let output = Command::new(current_exe)
            .arg(CHILD_TEST_NAME)
            .arg("--exact")
            .arg("--nocapture")
            .env(CHILD_ENV, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
        assert!(stdout.contains("stagex-protected-transition-ok"), "stdout={stdout}");
        assert!(!stderr.contains("/bin/sh"), "stderr={stderr}");
    }

    #[test]
    fn transition_input_digest_mismatch_fails_before_scratch_execution() {
        let temp = tempfile::tempdir().unwrap();
        let seed = temp.path().join("bad-seed");
        fs::write(&seed, b"not the audited seed").unwrap();
        let error = read_checked_input(&seed, "hex0 seed", HEX0_SEED_BLAKE3, AUDIT_SEED_MAX_BYTES).unwrap_err();
        assert!(matches!(error, StagexTransitionError::DigestMismatch { .. }));
        assert!(!temp.path().join("hex0-reproduced").exists());
    }

    #[test]
    fn substituted_manifest_digest_fails_before_execution() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let bytes = fs::read(repo.join("bootstrap/stagex-transition-lineage.json")).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["generated_artifacts"][1]["digest"]["hex_value"] =
            serde_json::Value::String(HEX0_SEED_BLAKE3.to_string());
        let temp = tempfile::tempdir().unwrap();
        let manifest_path = temp.path().join("substituted.json");
        fs::write(&manifest_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let error = load_transition_manifest_authority(&manifest_path).unwrap_err();
        assert!(matches!(error, StagexTransitionError::InvalidInput(_)));
        assert!(error.to_string().contains("kaem-0"));
    }

    #[test]
    fn audit_rejects_substituted_path_and_denied_decision() {
        let expected_path = PathBuf::from("/stagex/hex0-seed");
        let expected = ExpectedAuditEvent {
            executable_path: &expected_path,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: "stagex:seed:hex0".to_string(),
        };
        let event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/bin/sh"),
            tracee_path: PathBuf::from("/bin/sh"),
            resolved_host_path: PathBuf::from("/bin/sh"),
            digest_hex: HEX0_SEED_BLAKE3.to_string(),
            reason: "test denial".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: None,
            policy_decision: "denied".to_string(),
        };
        let error = validate_audit_event(&event, &expected).unwrap_err();
        assert!(matches!(error, StagexTransitionError::Audit(_)));
        assert!(error.to_string().contains("path, digest, source stage, or allow decision"));
    }

    #[test]
    fn tinycc_event_count_accepts_complete_and_rejects_missing_event() {
        const COMPILE_COMMAND_COUNT: u32 = 2;
        const RUNTIME_COMMAND_COUNT: u32 = 7;
        const BOOT_COMMAND_COUNT: u32 = 2;
        const FINAL_COMMAND_COUNT: u32 = 3;
        const EXPECTED_EVENT_COUNT: usize = 19;
        let expected = tinycc_expected_event_count(
            COMPILE_COMMAND_COUNT,
            RUNTIME_COMMAND_COUNT,
            BOOT_COMMAND_COUNT,
            FINAL_COMMAND_COUNT,
        )
        .unwrap();
        assert_eq!(expected, EXPECTED_EVENT_COUNT);
        validate_tinycc_observed_event_count(expected, expected).unwrap();

        let missing_event_count = expected.checked_sub(1).unwrap();
        let error = validate_tinycc_observed_event_count(expected, missing_event_count).unwrap_err();
        assert!(error.to_string().contains("TinyCC protected audit expected"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn tinycc27_event_count_is_closed_over_runtime_build_and_smokes() {
        const EXPECTED_EVENT_COUNT: usize = 7;
        const EXPECTED_PREDECESSOR_COUNT: usize = 4;
        let report = crate::stagex_tinycc27::Tinycc27InventoryReport {
            format: "test",
            source_patch_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            patched_files: vec!["tcc.c".to_string()],
            unified_libc_source_count: 1,
            runtime_command_count: 3,
            build_command_count: 1,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tinycc27_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        assert_eq!(tinycc27_predecessor_event_count(&report).unwrap(), EXPECTED_PREDECESSOR_COUNT);

        let mut overflow = report;
        overflow.runtime_command_count = u32::MAX;
        let error = tinycc27_expected_event_count(&overflow).unwrap_err();
        assert!(error.to_string().contains("command count overflow"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn tcc_selfhost_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 14;
        let report = crate::stagex_tcc_selfhost::InventoryReport {
            format: "test",
            build_command_count: 13,
            smoke_command_count: 1,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tcc_selfhost_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = 0;
        assert!(tcc_selfhost_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn musl_native_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 770;
        let report = crate::stagex_musl_native::InventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            selfhost_compile_count: crate::stagex_musl_native::SELFHOST_COMPILE_COUNT,
            predecessor_compile_count: crate::stagex_musl_native::PREDECESSOR_COMPILE_COUNT,
            archive_command_count: 1,
            smoke_command_count: 2,
            execution_command_count: 2,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(musl_native_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.selfhost_compile_count = substituted.selfhost_compile_count.checked_sub(1).unwrap();
        assert!(musl_native_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn m4_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 29;
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 30;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 5;
        const EXPECTED_EVENT_COUNT: usize = 35;
        let report = crate::stagex_m4::M4InventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: EXPECTED_SOURCE_COMPILE_COUNT,
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(m4_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.source_compile_count = substituted.source_compile_count.checked_sub(1).unwrap();
        assert!(m4_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn diffutils_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 24;
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 26;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 6;
        const EXPECTED_EVENT_COUNT: usize = 32;
        let report = crate::stagex_diffutils::DiffutilsInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: EXPECTED_SOURCE_COMPILE_COUNT,
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(diffutils_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = substituted.smoke_command_count.checked_sub(1).unwrap();
        assert!(diffutils_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn grep_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 2;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
        const EXPECTED_EVENT_COUNT: usize = 5;
        let report = crate::stagex_grep::GrepInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(grep_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.build_command_count = substituted.build_command_count.checked_add(1).unwrap();
        assert!(grep_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn gawk_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 17;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 7;
        const EXPECTED_EVENT_COUNT: usize = 24;
        let report = crate::stagex_gawk::GawkInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            negative_exit_code: 1,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(gawk_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.negative_exit_code = 0;
        assert!(gawk_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn bison_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 56;
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 57;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 3;
        const EXPECTED_EVENT_COUNT: usize = 61;
        let report = crate::stagex_bison::BisonInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: EXPECTED_SOURCE_COMPILE_COUNT,
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            negative_exit_code: 1,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(bison_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.source_compile_count = substituted.source_compile_count.checked_sub(1).unwrap();
        assert!(bison_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn flex_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 21;
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 25;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 4;
        const EXPECTED_EVENT_COUNT: usize = 31;
        let report = crate::stagex_flex::FlexInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: EXPECTED_SOURCE_COMPILE_COUNT,
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            negative_exit_code: 1,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(flex_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = substituted.smoke_command_count.checked_sub(1).unwrap();
        assert!(flex_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn transition_plan_closes_binutils_producer_and_consumer_stages() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let authority =
            load_transition_manifest_authority(&repo.join("bootstrap/stagex-transition-lineage.json")).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("transition");
        let staged = StagedTransitionPaths {
            seed: root.join("inputs/hex0-seed"),
            hex0_source: root.join("inputs/hex0-source"),
            kaem_source: root.join("inputs/kaem-source"),
            reproduced_hex0: root.join("outputs/hex0"),
            kaem: root.join("outputs/kaem"),
            kaem_smoke_script: root.join("inputs/kaem-smoke"),
            kaem_smoke: root.join("outputs/kaem-smoke"),
        };
        let source_state = "a".repeat(blake3::OUT_LEN * 2);
        let plan = build_transition_plan(&authority, &staged, &source_state, true).unwrap();
        let stage_ids = plan.stages.iter().map(|stage| stage.id.as_str()).collect::<BTreeSet<_>>();
        assert_eq!(plan.stage_count_max, TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX);
        assert_eq!(plan.stages.len(), usize::try_from(TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX).unwrap());
        assert!(stage_ids.contains(BASH_FULL_GENERATOR_BUILD_STAGE_ID));
        assert!(stage_ids.contains(BASH_FULL_SMOKE_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_CANONICALIZER_BUILD_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_HELPER_BUILD_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_GENERATOR_BUILD_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_COMPONENT_BUILD_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_INSTALL_STAGE_ID));
        assert!(stage_ids.contains(BINUTILS_INSTALL_SMOKE_STAGE_ID));
        let authorization_ids = plan
            .stages
            .iter()
            .flat_map(|stage| stage.executable_authorizations.iter().map(|authorization| authorization.id.as_str()))
            .collect::<Vec<_>>();
        let unique_authorization_ids = authorization_ids.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(authorization_ids.len(), unique_authorization_ids.len());
        let expected_coreutils = BINUTILS_EXECUTED_COREUTILS_TOOL_NAMES.iter().copied().collect::<BTreeSet<_>>();
        for stage_id in [
            BINUTILS_GENERATOR_BUILD_STAGE_ID,
            BINUTILS_COMPONENT_BUILD_STAGE_ID,
            BINUTILS_INSTALL_STAGE_ID,
        ] {
            let stage = plan.stages.iter().find(|stage| stage.id == stage_id).unwrap();
            let observed = stage
                .executable_authorizations
                .iter()
                .filter_map(|authorization| authorization.id.rsplit_once(":coreutils-").map(|(_, tool)| tool))
                .collect::<BTreeSet<_>>();
            assert_eq!(observed, expected_coreutils);
        }
        let smoke = plan.stages.iter().find(|stage| stage.id == BINUTILS_INSTALL_SMOKE_STAGE_ID).unwrap();
        let smoke_ids = smoke
            .executable_authorizations
            .iter()
            .map(|authorization| authorization.id.as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            BINUTILS_INSTALL_SMOKE_EXECUTED_TOOL_NAMES
                .iter()
                .all(|tool| { smoke_ids.contains(format!("exec:{BINUTILS_INSTALL_SMOKE_STAGE_ID}:{tool}").as_str()) })
        );
        assert!(
            !["size", "strings", "strip"]
                .iter()
                .any(|tool| { smoke_ids.contains(format!("exec:{BINUTILS_INSTALL_SMOKE_STAGE_ID}:{tool}").as_str()) })
        );
    }

    #[test]
    fn binutils_runtime_policy_adds_missing_predecessors_without_duplicates() {
        let transition_root = PathBuf::from("/stagex/transition");
        let paths = BinutilsStagePaths::from_transition_root(&transition_root);
        let existing = vec![PlannedExecutable {
            authorization_id: "existing-tcc".to_string(),
            source_stage_id: TCC_MUSL_V2_BUILD_STAGE_ID.to_string(),
            path: paths.tcc.clone(),
            digest_hex: crate::stagex_tcc_musl_v2::COMPILER_BLAKE3.to_string(),
        }];
        let additional = additional_planned_binutils_predecessor_executables(&transition_root, &existing);
        let wc = paths.coreutils_bin.join("wc");
        assert!(!additional.iter().any(|entry| entry.path == paths.tcc));
        assert!(additional.iter().any(|entry| entry.path == wc));
        assert!(
            additional
                .iter()
                .all(|entry| !existing.iter().any(|old| old.path == entry.path && old.digest_hex == entry.digest_hex))
        );
    }

    #[test]
    fn binutils_mkdir_count_accepts_closed_observed_bounds() {
        let path = PathBuf::from("/stagex/coreutils/bin/mkdir");
        let digest = "a".repeat(blake3::OUT_LEN * 2);
        for count in BINUTILS_MKDIR_EXEC_COUNT_BOUNDS {
            let counts = BTreeMap::from([((path.clone(), digest.clone()), count)]);
            assert!(require_binutils_path_count_bounds(&counts, &path, BINUTILS_MKDIR_EXEC_COUNT_BOUNDS).is_ok());
        }
        assert!(path.is_absolute());
        assert!(BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[0] < BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[1]);
    }

    #[test]
    fn binutils_mkdir_count_rejects_outside_or_ambiguous_identity() {
        let path = PathBuf::from("/stagex/coreutils/bin/mkdir");
        let digest = "a".repeat(blake3::OUT_LEN * 2);
        let below = BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[0].checked_sub(1).unwrap();
        let above = BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[1].checked_add(1).unwrap();
        for count in [below, above] {
            let counts = BTreeMap::from([((path.clone(), digest.clone()), count)]);
            assert!(require_binutils_path_count_bounds(&counts, &path, BINUTILS_MKDIR_EXEC_COUNT_BOUNDS).is_err());
        }
        let ambiguous = BTreeMap::from([
            ((path.clone(), digest), BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[0]),
            ((path.clone(), "b".repeat(blake3::OUT_LEN * 2)), BINUTILS_MKDIR_EXEC_COUNT_BOUNDS[0]),
        ]);
        assert!(require_binutils_path_count_bounds(&ambiguous, &path, BINUTILS_MKDIR_EXEC_COUNT_BOUNDS).is_err());
        assert_eq!(ambiguous.len(), 2);
    }

    #[test]
    fn binutils_event_count_accepts_only_the_closed_bounds() {
        const LOWER_SED_INVOCATION_COUNT: u32 = 4_891;
        const MIDDLE_SED_INVOCATION_COUNT: u32 = 4_892;
        const UPPER_SED_INVOCATION_COUNT: u32 = 4_893;
        const OUTSIDE_SED_INVOCATION_COUNTS: [u32; 2] = [4_890, 4_894];
        const LOWER_SED_EVENT_BOUNDS: [usize; 2] = [73_991, 74_045];
        const MIDDLE_SED_EVENT_BOUNDS: [usize; 2] = [74_008, 74_066];
        const UPPER_SED_EVENT_BOUNDS: [usize; 2] = [74_010, 74_057];
        let bounds = crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS
            .map(|count| u32::try_from(count).unwrap());
        let below = crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[0].checked_sub(1).unwrap();
        let above = crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[1].checked_add(1).unwrap();
        assert!(binutils_event_count_within_bounds(
            bounds,
            crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[0]
        ));
        assert!(binutils_event_count_within_bounds(
            bounds,
            crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[1]
        ));
        assert!(!binutils_event_count_within_bounds(bounds, below));
        assert!(!binutils_event_count_within_bounds(bounds, above));
        assert_eq!(
            binutils_event_count_bounds_from_sed_invocations(LOWER_SED_INVOCATION_COUNT).unwrap(),
            LOWER_SED_EVENT_BOUNDS
        );
        assert_eq!(
            binutils_event_count_bounds_from_sed_invocations(MIDDLE_SED_INVOCATION_COUNT).unwrap(),
            MIDDLE_SED_EVENT_BOUNDS
        );
        assert_eq!(
            binutils_event_count_bounds_from_sed_invocations(UPPER_SED_INVOCATION_COUNT).unwrap(),
            UPPER_SED_EVENT_BOUNDS
        );
        for count in OUTSIDE_SED_INVOCATION_COUNTS {
            assert!(binutils_event_count_bounds_from_sed_invocations(count).is_err());
        }
    }

    #[test]
    fn binutils_plan_rejects_same_stage_generated_executable_authority() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let authority =
            load_transition_manifest_authority(&repo.join("bootstrap/stagex-transition-lineage.json")).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("transition");
        let staged = StagedTransitionPaths {
            seed: root.join("inputs/hex0-seed"),
            hex0_source: root.join("inputs/hex0-source"),
            kaem_source: root.join("inputs/kaem-source"),
            reproduced_hex0: root.join("outputs/hex0"),
            kaem: root.join("outputs/kaem"),
            kaem_smoke_script: root.join("inputs/kaem-smoke"),
            kaem_smoke: root.join("outputs/kaem-smoke"),
        };
        let source_state = "a".repeat(blake3::OUT_LEN * 2);
        let mut plan = build_transition_plan(&authority, &staged, &source_state, true).unwrap();
        let component = plan.stages.iter_mut().find(|stage| stage.id == BINUTILS_COMPONENT_BUILD_STAGE_ID).unwrap();
        let chew = component
            .executable_authorizations
            .iter_mut()
            .find(|authorization| authorization.id.ends_with(":bfd-chew"))
            .unwrap();
        chew.source_stage_id = BINUTILS_COMPONENT_BUILD_STAGE_ID.to_string();
        let validation = validate_stagex_plan(&plan);
        assert!(!validation.is_valid());
        assert!(
            validation
                .errors
                .iter()
                .any(|error| error.to_string().contains("invalid source stage binutils-component-materialization"))
        );
    }

    #[test]
    fn bash_full_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_SOURCE_COMPILE_COUNT: u32 = 130;
        const EXPECTED_BUILD_COMMAND_COUNT: u32 = 133;
        const EXPECTED_GENERATOR_COMMAND_COUNT: u32 = 40;
        const EXPECTED_SMOKE_COMMAND_COUNT: u32 = 5;
        const EXPECTED_EVENT_COUNT: usize = 180;
        let report = crate::stagex_bash_full::BashFullInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            generator_path: PathBuf::from("/stagex/mkbuiltins"),
            generator_digest_blake3: "b".repeat(blake3::OUT_LEN * 2),
            generated_tree_digest_blake3: "c".repeat(blake3::OUT_LEN * 2),
            source_compile_count: EXPECTED_SOURCE_COMPILE_COUNT,
            build_command_count: EXPECTED_BUILD_COMMAND_COUNT,
            generator_command_count: EXPECTED_GENERATOR_COMMAND_COUNT,
            smoke_command_count: EXPECTED_SMOKE_COMMAND_COUNT,
            negative_exit_code: 2,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(bash_full_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.generator_command_count = substituted.generator_command_count.checked_sub(1).unwrap();
        assert!(bash_full_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn m4_compiler_audit_uses_canonical_predecessor_authority() {
        let expected_path = PathBuf::from("/stagex/tcc-musl-v2");
        let mut event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: expected_path.clone(),
            tracee_path: expected_path.clone(),
            resolved_host_path: expected_path.clone(),
            digest_hex: crate::stagex_tcc_musl_v2::COMPILER_BLAKE3.to_string(),
            reason: "test allow".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some(TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID.to_string()),
            policy_decision: "allowed".to_string(),
        };
        validate_coreutils_event(
            &event,
            &expected_path,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )
        .unwrap();
        event.inventory_entry_id = Some("planned:substituted-authority".to_string());
        let error = validate_coreutils_event(
            &event,
            &expected_path,
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
            TCC_MUSL_V2_CANONICAL_AUTHORIZATION_ID,
        )
        .unwrap_err();
        assert!(error.to_string().contains("exact planned authority"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn tcc_musl_v2_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 9;
        let report = crate::stagex_tcc_musl_v2::InventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            build_command_count: 4,
            smoke_command_count: 5,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tcc_musl_v2_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = 4;
        assert!(tcc_musl_v2_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn musl_pass2_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 226;
        let report = crate::stagex_musl::MuslInventoryReport {
            format: "mantle-stagex-musl-1.1.24-pass2-inventory-v1",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            helper_count: 28,
            source_compile_count: 219,
            crt_compile_count: 3,
            archive_command_count: 1,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(musl_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = 2;
        assert!(musl_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn tcc_musl_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 6;
        let report = crate::stagex_tcc_musl::TccMuslInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            build_command_count: 2,
            smoke_command_count: 4,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tcc_musl_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = 3;
        assert!(tcc_musl_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn musl_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 222;
        let report = crate::stagex_musl::MuslInventoryReport {
            format: "mantle-stagex-musl-1.1.24-inventory-v1",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            helper_count: 24,
            source_compile_count: 215,
            crt_compile_count: 3,
            archive_command_count: 1,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(musl_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.source_compile_count = 214;
        assert!(musl_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn tcc_musl_prep_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 5;
        let report = crate::stagex_tcc_musl_prep::TccMuslPrepInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            build_command_count: 2,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tcc_musl_prep_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.smoke_command_count = 2;
        assert!(tcc_musl_prep_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn bash_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 96;
        let report = crate::stagex_bash::BashInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            helper_count: 10,
            source_compile_count: 92,
            build_command_count: 93,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(bash_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.build_command_count = 92;
        assert!(bash_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn oyacc_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 18;
        let report = crate::stagex_oyacc::OyaccInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            patch_command_count: 2,
            source_compile_count: 13,
            build_command_count: 14,
            smoke_command_count: 2,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(oyacc_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.source_compile_count = 12;
        assert!(oyacc_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn coreutils_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 170;
        let report = crate::stagex_coreutils::CoreutilsInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            patch_command_count: 8,
            library_compile_count: 96,
            archive_command_count: 1,
            utility_source_compile_count: 34,
            utility_link_count: 25,
            build_command_count: 156,
            smoke_command_count: 6,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(coreutils_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        let mut substituted = report;
        substituted.utility_link_count = 24;
        assert!(coreutils_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn coreutils_audit_event_rejects_denied_or_substituted_authority() {
        let expected_path = PathBuf::from("/stagex/coreutils/cp");
        let inventory_id = "planned:coreutils-materialization:exec:coreutils-smoke:cp";
        let mut event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: expected_path.clone(),
            tracee_path: expected_path.clone(),
            resolved_host_path: expected_path.clone(),
            digest_hex: "a".repeat(blake3::OUT_LEN * 2),
            reason: "test allow".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some(inventory_id.to_string()),
            policy_decision: "allowed".to_string(),
        };
        let expected_digest = event.digest_hex.clone();
        validate_coreutils_event(&event, &expected_path, &expected_digest, inventory_id).unwrap();
        event.policy_decision = "denied".to_string();
        assert!(validate_coreutils_event(&event, &expected_path, &expected_digest, inventory_id).is_err());
        event.policy_decision = "allowed".to_string();
        event.digest_hex = "b".repeat(blake3::OUT_LEN * 2);
        assert!(validate_coreutils_event(&event, &expected_path, &expected_digest, inventory_id).is_err());
    }

    #[test]
    fn bzip2_event_count_is_closed_and_rejects_substitution() {
        const EXPECTED_EVENT_COUNT: usize = 18;
        let report = crate::stagex_bzip2::Bzip2InventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            rewrite_command_count: 3,
            source_compile_count: 9,
            build_command_count: 11,
            smoke_command_count: 4,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(bzip2_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_bzip2_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        assert!(validate_bzip2_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT - 1).is_err());
        assert!(validate_bzip2_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT + 1).is_err());
        let mut substituted = report;
        substituted.rewrite_command_count = 2;
        assert!(bzip2_expected_event_count(&substituted).unwrap_err().to_string().contains("substituted"));
    }

    #[test]
    fn bzip2_audit_event_rejects_denied_or_substituted_authority() {
        let expected_path = PathBuf::from("/stagex/bzip2");
        let inventory_id = "planned:bzip2-materialization:exec:bzip2-smoke:bzip2";
        let mut event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: expected_path.clone(),
            tracee_path: expected_path.clone(),
            resolved_host_path: expected_path.clone(),
            digest_hex: crate::stagex_bzip2::BZIP2_FINAL_BLAKE3.to_string(),
            reason: "test allow".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some(inventory_id.to_string()),
            policy_decision: "allowed".to_string(),
        };
        validate_bzip2_event(&event, &expected_path, crate::stagex_bzip2::BZIP2_FINAL_BLAKE3, inventory_id).unwrap();
        event.policy_decision = "denied".to_string();
        assert!(
            validate_bzip2_event(&event, &expected_path, crate::stagex_bzip2::BZIP2_FINAL_BLAKE3, inventory_id)
                .is_err()
        );
        event.policy_decision = "allowed".to_string();
        event.digest_hex = "a".repeat(blake3::OUT_LEN * 2);
        assert!(
            validate_bzip2_event(&event, &expected_path, crate::stagex_bzip2::BZIP2_FINAL_BLAKE3, inventory_id)
                .is_err()
        );
    }

    #[test]
    fn sed_event_count_is_closed_and_rejects_missing_or_extra_events() {
        const EXPECTED_EVENT_COUNT: usize = 17;
        let report = crate::stagex_sed::SedInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: 13,
            build_command_count: 14,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(sed_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_sed_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        let missing = EXPECTED_EVENT_COUNT.checked_sub(1).unwrap();
        let extra = EXPECTED_EVENT_COUNT.checked_add(1).unwrap();
        assert!(validate_sed_observed_event_count(EXPECTED_EVENT_COUNT, missing).is_err());
        assert!(validate_sed_observed_event_count(EXPECTED_EVENT_COUNT, extra).is_err());

        let mut substituted = report;
        substituted.smoke_command_count = 2;
        let error = sed_expected_event_count(&substituted).unwrap_err();
        assert!(error.to_string().contains("substituted"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn sed_audit_event_rejects_denied_or_substituted_authority() {
        let expected_path = PathBuf::from("/stagex/sed");
        let inventory_id = "planned:sed-materialization:exec:sed-smoke:sed";
        let mut event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: expected_path.clone(),
            tracee_path: expected_path.clone(),
            resolved_host_path: expected_path.clone(),
            digest_hex: crate::stagex_sed::SED_FINAL_BLAKE3.to_string(),
            reason: "test allow".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some(inventory_id.to_string()),
            policy_decision: "allowed".to_string(),
        };
        validate_sed_event(&event, &expected_path, crate::stagex_sed::SED_FINAL_BLAKE3, inventory_id).unwrap();

        event.policy_decision = "denied".to_string();
        assert!(validate_sed_event(&event, &expected_path, crate::stagex_sed::SED_FINAL_BLAKE3, inventory_id).is_err());
        event.policy_decision = "allowed".to_string();
        event.inventory_entry_id = Some("planned:substituted".to_string());
        assert!(validate_sed_event(&event, &expected_path, crate::stagex_sed::SED_FINAL_BLAKE3, inventory_id).is_err());
    }

    #[test]
    fn tar_event_count_is_closed_and_rejects_missing_or_extra_events() {
        const EXPECTED_EVENT_COUNT: usize = 34;
        let report = crate::stagex_tar::TarInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: 29,
            build_command_count: 30,
            smoke_command_count: 4,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(tar_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_tar_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        let missing = EXPECTED_EVENT_COUNT.checked_sub(1).unwrap();
        let extra = EXPECTED_EVENT_COUNT.checked_add(1).unwrap();
        assert!(validate_tar_observed_event_count(EXPECTED_EVENT_COUNT, missing).is_err());
        assert!(validate_tar_observed_event_count(EXPECTED_EVENT_COUNT, extra).is_err());

        let mut substituted = report;
        substituted.build_command_count = 29;
        let error = tar_expected_event_count(&substituted).unwrap_err();
        assert!(error.to_string().contains("substituted"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn gzip_event_count_is_closed_and_rejects_missing_or_extra_events() {
        const EXPECTED_EVENT_COUNT: usize = 21;
        let report = crate::stagex_gzip::GzipInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: 14,
            generator_build_command_count: 1,
            generator_run_command_count: 1,
            build_command_count: 15,
            smoke_command_count: 4,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(gzip_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_gzip_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        let missing = EXPECTED_EVENT_COUNT.checked_sub(1).unwrap();
        let extra = EXPECTED_EVENT_COUNT.checked_add(1).unwrap();
        assert!(validate_gzip_observed_event_count(EXPECTED_EVENT_COUNT, missing).is_err());
        assert!(validate_gzip_observed_event_count(EXPECTED_EVENT_COUNT, extra).is_err());

        let mut substituted = report;
        substituted.generator_run_command_count = 2;
        let error = gzip_expected_event_count(&substituted).unwrap_err();
        assert!(error.to_string().contains("substituted"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn gnu_patch_event_count_is_closed_and_rejects_missing_or_extra_events() {
        const EXPECTED_EVENT_COUNT: usize = 23;
        let report = crate::stagex_gnu_patch::GnuPatchInventoryReport {
            format: "test",
            configured_source_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            source_compile_count: 19,
            build_command_count: 20,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(gnu_patch_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_gnu_patch_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        let missing = EXPECTED_EVENT_COUNT.checked_sub(1).unwrap();
        let extra = EXPECTED_EVENT_COUNT.checked_add(1).unwrap();
        assert!(validate_gnu_patch_observed_event_count(EXPECTED_EVENT_COUNT, missing).is_err());
        assert!(validate_gnu_patch_observed_event_count(EXPECTED_EVENT_COUNT, extra).is_err());

        let mut substituted = report;
        substituted.build_command_count = 19;
        let error = gnu_patch_expected_event_count(&substituted).unwrap_err();
        assert!(error.to_string().contains("substituted"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn make_event_count_is_closed_over_build_recipe_dispatch_and_smokes() {
        const EXPECTED_EVENT_COUNT: usize = 33;
        const EXPECTED_PREDECESSOR_COUNT: usize = 29;
        let report = crate::stagex_make::MakeInventoryReport {
            format: "test",
            source_patch_digest_blake3: "a".repeat(blake3::OUT_LEN * 2),
            patched_files: vec!["main.c".to_string()],
            source_compile_count: 27,
            recipe_runner_command_count: 1,
            build_command_count: 28,
            smoke_command_count: 3,
            outputs: Vec::new(),
            protected_exec_enforced: true,
            fallback_events: Vec::new(),
            non_claim: "test",
        };
        assert_eq!(make_predecessor_event_count(&report).unwrap(), EXPECTED_PREDECESSOR_COUNT);
        assert_eq!(make_expected_event_count(&report).unwrap(), EXPECTED_EVENT_COUNT);
        validate_make_observed_event_count(EXPECTED_EVENT_COUNT, EXPECTED_EVENT_COUNT).unwrap();
        let missing = EXPECTED_EVENT_COUNT.checked_sub(1).unwrap();
        let extra = EXPECTED_EVENT_COUNT.checked_add(1).unwrap();
        assert!(validate_make_observed_event_count(EXPECTED_EVENT_COUNT, missing).is_err());
        assert!(validate_make_observed_event_count(EXPECTED_EVENT_COUNT, extra).is_err());

        let mut substituted = report;
        substituted.smoke_command_count = 2;
        let error = make_expected_event_count(&substituted).unwrap_err();
        assert!(error.to_string().contains("smoke command count"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn make_audit_event_rejects_denied_or_substituted_authority() {
        let expected_path = PathBuf::from("/stagex/make");
        let mut event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: expected_path.clone(),
            tracee_path: expected_path.clone(),
            resolved_host_path: expected_path.clone(),
            digest_hex: crate::stagex_make::MAKE_FINAL_BLAKE3.to_string(),
            reason: "test allow".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some("planned:make-materialization:exec:make-smoke:make".to_string()),
            policy_decision: "allowed".to_string(),
        };
        validate_make_event(
            &event,
            &expected_path,
            crate::stagex_make::MAKE_FINAL_BLAKE3,
            "planned:make-materialization:exec:make-smoke:make",
        )
        .unwrap();

        event.policy_decision = "denied".to_string();
        assert!(
            validate_make_event(
                &event,
                &expected_path,
                crate::stagex_make::MAKE_FINAL_BLAKE3,
                "planned:make-materialization:exec:make-smoke:make",
            )
            .is_err()
        );
        event.policy_decision = "allowed".to_string();
        event.executable_path = PathBuf::from("/stagex/substituted-make");
        assert!(
            validate_make_event(
                &event,
                &expected_path,
                crate::stagex_make::MAKE_FINAL_BLAKE3,
                "planned:make-materialization:exec:make-smoke:make",
            )
            .is_err()
        );
    }

    #[test]
    fn bounded_retry_classification_is_exact() {
        let exact = io::Error::from_raw_os_error(libc::ETXTBSY);
        let other = io::Error::from_raw_os_error(libc::EACCES);
        let exit_marker = StagexTransitionError::ProcessFailure {
            executable: PathBuf::from("/stagex/generated"),
            exit_code: Some(ETXTBSY_EXIT_CODE),
            stderr: ETXTBSY_STDERR_MARKER.to_string(),
        };
        assert_eq!(exact.raw_os_error(), Some(libc::ETXTBSY));
        assert_ne!(other.raw_os_error(), Some(libc::ETXTBSY));
        let StagexTransitionError::ProcessFailure { exit_code, .. } = exit_marker else {
            panic!("expected process failure fixture");
        };
        assert_eq!(exit_code, Some(ETXTBSY_EXIT_CODE));
    }

    fn run_protected_transition_child() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let manifest_path = repo.join("bootstrap/stagex-transition-lineage.json");
        let manifest_bytes = fs::read(&manifest_path).unwrap();
        let manifest = crate::bootstrap_source_root::validate_stagex_lineage_manifest(&manifest_bytes).unwrap();
        let manifest_digest = blake3_file_hex(&manifest_path).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let scratch = std::env::var_os(SCRATCH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| temp.path().join("protected-transition"));
        let source_bundle = std::env::var_os(SOURCE_BUNDLE_ENV).map(PathBuf::from);
        let stage0_answers = source_bundle.as_ref().map(|_| repo.join("bootstrap/stage0-amd64.answers"));
        assert_eq!(manifest_digest, TEST_LINEAGE_MANIFEST_DIGEST);
        assert_eq!(manifest.seed.seed_digest.hex_value, HEX0_SEED_BLAKE3);
        assert_eq!(manifest.seed.seed_bytes_len, HEX0_SEED_BYTES);
        let report = materialize_protected_transition(StagexTransitionRequest {
            seed_path: &repo.join("bootstrap/seeds/AMD64/hex0-seed"),
            hex0_source_path: &repo.join("bootstrap/seeds/AMD64/hex0_AMD64.hex0"),
            kaem_source_path: &repo.join("bootstrap/seeds/AMD64/kaem-minimal.hex0"),
            lineage_manifest_path: &manifest_path,
            source_bundle_path: source_bundle.as_deref(),
            stage0_answers_path: stage0_answers.as_deref(),
            scratch_dir: &scratch,
        })
        .unwrap();
        assert_eq!(report.status, "complete");
        let minimum_event_count = report.stage0_full.as_ref().map_or(EXPECTED_AUDIT_EVENT_COUNT, |stage0| {
            let mini_count = usize::try_from(stage0.mini.command_count).unwrap().checked_add(1).unwrap();
            let mes_count = report
                .mes_m2
                .as_ref()
                .map(|mes| usize::try_from(mes.command_count).unwrap().checked_add(1).unwrap())
                .unwrap();
            let runtime_count = report.mes_runtime.as_ref().map(mes_runtime_event_count).unwrap().unwrap();
            EXPECTED_AUDIT_EVENT_COUNT
                .checked_add(mini_count)
                .and_then(|count| count.checked_add(mes_count))
                .and_then(|count| count.checked_add(runtime_count))
                .unwrap()
        });
        if report.stage0_full.is_some() {
            assert!(report.protected_exec_events.len() > minimum_event_count);
        } else {
            assert_eq!(report.protected_exec_events.len(), minimum_event_count);
        }
        assert_eq!(report.stage0_full.is_some(), source_bundle.is_some());
        assert_eq!(report.mes_m2.is_some(), source_bundle.is_some());
        assert_eq!(report.mes_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.tinycc_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.tinycc_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.tinycc27_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.tinycc27_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.musl_native_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.m4_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.m4_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.diffutils_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.diffutils_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.grep_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.grep_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.gawk_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.gawk_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.bison_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.bison_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.flex_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.flex_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.bash_full_runtime.is_some(), source_bundle.is_some());
        assert_eq!(report.binutils_sources.is_some(), source_bundle.is_some());
        assert_eq!(report.binutils_runtime.is_some(), source_bundle.is_some());
        assert!(
            report
                .stage0_full
                .as_ref()
                .is_none_or(|stage0| { stage0.protected_exec_enforced && stage0.mini.protected_exec_enforced })
        );
        assert!(report.mes_m2.as_ref().is_none_or(|mes| mes.protected_exec_enforced));
        assert!(report.mes_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.tinycc_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.tinycc27_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.musl_native_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.m4_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.diffutils_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.grep_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.gawk_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.bison_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.flex_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.bash_full_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert!(report.binutils_runtime.as_ref().is_none_or(|runtime| runtime.protected_exec_enforced));
        assert_eq!(report.promotions.len(), EXPECTED_PROMOTION_COUNT);
        assert!(report.fallback_events.is_empty());
        assert!(scratch.join(REPORT_FILE_NAME).is_file());
        assert!(scratch.join(PLAN_FILE_NAME).is_file());
        assert!(scratch.join(AUDIT_FILE_NAME).is_file());
        println!("stagex-protected-transition-ok");
    }
}
