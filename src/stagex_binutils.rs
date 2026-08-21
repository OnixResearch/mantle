use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const BINUTILS_RECORD_NAME: &str = "binutils-2.30-src";
const BINUTILS_RECORD_IDENTITY: &str = "fixed-url-69a9734ff8cf5b25adafed78fba527bac05601a9da4bebafd91c75abe45f512e";
const BINUTILS_SOURCE_OUTPUT_NAME: &str = "binutils-2.30";
pub(crate) const BINUTILS_SOURCE_ARTIFACT_ID: &str = "binutils-2.30-source";
pub(crate) const BINUTILS_SOURCE_CONTENT_BLAKE3: &str =
    "809e8c1d946b14650362cf2929520efe07623fe069ce8c16c5870dafe6d93603";
pub(crate) const BINUTILS_RECIPE_ARTIFACT_ID: &str = "binutils-2.30-recipe-source";
pub(crate) const BINUTILS_RECIPE_BLAKE3: &str = "406d4aaecf2cc8c9b357463fb8f48f185acaa67e77c4136e5908cca851efd177";
const BINUTILS_RECIPE: &[u8] = include_bytes!("../bootstrap/binutils-tcc.ncl");
const SED_BRIDGE_LAUNCHER_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-sed-bridge-launcher.c");
const SED_BRIDGE_SCRIPT_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-sed-regular-file-bridge.sh");
const SINGLE_THREAD_SEMAPHORE_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-single-thread-semaphore-compat.c");
const CONFIGURE_UTILITY_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-configure-utility.c");
const ARCHIVE_CANONICAL_TEMP_EXTENSION: &str = "mantle-canonical-archive.tmp";
const YLWRAP_SED_RUNNER_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-ylwrap-sed-runner.c");
const BINUTILS_AR_RUNNER_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-binutils-ar-runner.sh");
const ELF_SYMBOL_CANONICALIZER_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-elf-local-symbol-canonicalizer.c");
const SED_BRIDGE_LAUNCHER_SOURCE_ARTIFACT_ID: &str = "stagex-sed-bridge-launcher-source";
const SED_BRIDGE_LAUNCHER_SOURCE_NAME: &str = "stagex-sed-bridge-launcher.c";
const SED_BRIDGE_LAUNCHER_OUTPUT_NAME: &str = "stagex-sed-bridge-launcher";
const SINGLE_THREAD_SEMAPHORE_SOURCE_NAME: &str = "stagex-single-thread-semaphore-compat.c";
const SED_BRIDGE_SCRIPT_SOURCE_ARTIFACT_ID: &str = "stagex-sed-regular-file-bridge-source";
const SINGLE_THREAD_SEMAPHORE_SOURCE_ARTIFACT_ID: &str = "stagex-single-thread-semaphore-compat-source";
const CONFIGURE_UTILITY_SOURCE_ARTIFACT_ID: &str = "stagex-configure-utility-source";
const YLWRAP_SED_RUNNER_SOURCE_ARTIFACT_ID: &str = "stagex-ylwrap-sed-runner-source";
const BINUTILS_AR_RUNNER_SOURCE_ARTIFACT_ID: &str = "stagex-binutils-ar-runner-source";
const ELF_SYMBOL_CANONICALIZER_SOURCE_ARTIFACT_ID: &str = "stagex-elf-local-symbol-canonicalizer-source";
const ELF_SYMBOL_CANONICALIZER_SOURCE_NAME: &str = "stagex-elf-local-symbol-canonicalizer.c";
const ELF_SYMBOL_CANONICALIZER_OUTPUT_NAME: &str = "stagex-elf-local-symbol-canonicalizer";
const YLWRAP_SED_RUNNER_SOURCE_NAME: &str = "stagex-ylwrap-sed-runner.c";
const YLWRAP_SED_RUNNER_OUTPUT_NAME: &str = "stagex-ylwrap-sed-runner";
const BINUTILS_AR_RUNNER_SOURCE_NAME: &str = "stagex-binutils-ar-runner.sh";
const SED_BRIDGE_LAUNCHER_SOURCE_BLAKE3: &str = "606c52085de42d0221ba5490e81d8c539a50a65aaa89f7b4b478371bdc643dab";
pub(crate) const SED_BRIDGE_LAUNCHER_BLAKE3: &str = "9b4d6a5eca05f55c407a70f7e426f756f482c9ae8f76d0a22d1b1b46e7dba1a1";
const SED_BRIDGE_SCRIPT_SOURCE_BLAKE3: &str = "fcdaf54c41ea283af6d7d75b2e2dce24afcbdf9286981e11603b2361775f5d6a";
const YLWRAP_SED_RUNNER_SOURCE_BLAKE3: &str = "372a51aef1c1d06bef3ec1963bc61e89598b2912c8e709dce747637ffe49587d";
pub(crate) const YLWRAP_SED_RUNNER_BLAKE3: &str = "d675a75869cba2e3c7cdb932ee75106a4a6094161e4d95fc196c08aa7b9723f8";
const BINUTILS_AR_RUNNER_SOURCE_BLAKE3: &str = "27daef7796f882d478b0d7f26508b4b7c0a3d67faa3fd9dafe6e6fb67733c9f1";
const ELF_SYMBOL_CANONICALIZER_SOURCE_BLAKE3: &str = "eed4dcf5e348d6b77317243a394ad2effb76ac112186eaf71e7115a1c5dc4a20";
pub(crate) const ELF_SYMBOL_CANONICALIZER_BLAKE3: &str =
    "1a7a10d6ce97f3cea18ffc4f956fe28bdb10d94568146d89670016a80d1de06a";
const BINUTILS_AR_SMOKE_ARCHIVE_BLAKE3: &str = "b54d2b2a954c606f06e62177013cec573bb4ac598c50b2e4f9867879ea5b65a9";
const SINGLE_THREAD_SEMAPHORE_SOURCE_BLAKE3: &str = "52c3ec19c484b0b4c3c5de84fc7ae77f40601fc81993d16ef6e15eb7401ee084";
const CONFIGURE_UTILITY_SOURCE_BLAKE3: &str = "b150327f4ef9256764e8024466dd706ee87012f70554f1c7a01a0d8bc08975b1";
pub(crate) const CONFIGURE_UTILITY_BLAKE3: &str = "a0d4f306ed84086cb0cebff1dffb0f5fea0a93e9ee4085e6e5e0acc3e4df201f";
const BINUTILS_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-binutils-2.30-source-materialization-v1";
const BINUTILS_INVENTORY_FORMAT: &str = "mantle-stagex-binutils-2.30-inventory-v1";
const BINUTILS_SOURCE_NON_CLAIM: &str =
    "binutils source materialization proves authenticated offline archive identity and checked-recipe identity only";
const BINUTILS_INVENTORY_NON_CLAIM: &str = "this inventory binds the checked binutils configure, generated-source, component, install, link, and smoke observations only; it does not prove compiler correctness, complete protected child authorization, provider publication, or provider admission";
const BINUTILS_RECORD_HASH: &str = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=";
const BINUTILS_RECORD_URL: &str = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz";
const BINUTILS_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const BINUTILS_RECORD_UNPACK: &str = "1";
const BINUTILS_SOURCE_ARTIFACT_COUNT: usize = 9;
const REQUIRED_SOURCE_FILES: &[&str] = &[
    "configure",
    "config.sub",
    "intl/configure",
    "libiberty/configure",
    "zlib/configure",
    "bfd/configure",
    "opcodes/configure",
    "binutils/configure",
    "gas/configure",
    "gprof/configure",
    "ld/configure",
    "opcodes/i386-gen.c",
    "binutils/arparse.y",
    "ld/ldgram.y",
];
const REQUIRED_SOURCE_FILE_COUNT: usize = 14;
const OPCODES_DEPENDENCY_OLD: &str = "$(srcdir)/i386-init.h: @MAINT@ ";
const OPCODES_DEPENDENCY_NEW: &str = "$(srcdir)/i386-init.h: ";
const BFD_BOOTSTRAP_CONFIG_REFERENCE: &str = "bfd-in3.h:bfd-in2.h";
const BFD_BOOTSTRAP_CONFIG_REFERENCE_COUNT: usize = 2;
const GENERATED_SOURCE_REMOVALS: &[&str] = &[
    "binutils/arparse.c",
    "binutils/arparse.h",
    "binutils/defparse.c",
    "binutils/defparse.h",
    "binutils/mcparse.c",
    "binutils/mcparse.h",
    "binutils/rcparse.c",
    "binutils/rcparse.h",
    "binutils/sysinfo.c",
    "binutils/sysinfo.h",
    "intl/plural.c",
    "ld/deffilep.c",
    "ld/deffilep.h",
    "ld/ldgram.c",
    "ld/ldgram.h",
    "binutils/arlex.c",
    "binutils/deflex.c",
    "binutils/syslex.c",
    "ld/ldlex.c",
    "opcodes/i386-init.h",
    "opcodes/i386-tbl.h",
    "bfd/libbfd.h",
    "bfd/bfd-in2.h",
    "bfd/libcoff.h",
    "zlib/crc32.h",
    "zlib/inffixed.h",
    "bfd/doc/bfd.info",
    "binutils/doc/binutils.info",
    "gas/doc/as.info",
    "ld/ld.info",
    "libiberty/functions.texi",
];
const GENERATED_SOURCE_REMOVAL_COUNT: usize = 31;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BinutilsSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum StagexBinutilsError {
    SourceRecordNotFound,
    Materialization(String),
}

impl std::fmt::Display for StagexBinutilsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU binutils source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU binutils materialization failed: {message}"),
        }
    }
}

impl std::error::Error for StagexBinutilsError {}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); BINUTILS_SOURCE_ARTIFACT_COUNT] {
    [
        (BINUTILS_SOURCE_ARTIFACT_ID, BINUTILS_SOURCE_CONTENT_BLAKE3),
        (BINUTILS_RECIPE_ARTIFACT_ID, BINUTILS_RECIPE_BLAKE3),
        (SED_BRIDGE_LAUNCHER_SOURCE_ARTIFACT_ID, SED_BRIDGE_LAUNCHER_SOURCE_BLAKE3),
        (SED_BRIDGE_SCRIPT_SOURCE_ARTIFACT_ID, SED_BRIDGE_SCRIPT_SOURCE_BLAKE3),
        (SINGLE_THREAD_SEMAPHORE_SOURCE_ARTIFACT_ID, SINGLE_THREAD_SEMAPHORE_SOURCE_BLAKE3),
        (CONFIGURE_UTILITY_SOURCE_ARTIFACT_ID, CONFIGURE_UTILITY_SOURCE_BLAKE3),
        (YLWRAP_SED_RUNNER_SOURCE_ARTIFACT_ID, YLWRAP_SED_RUNNER_SOURCE_BLAKE3),
        (BINUTILS_AR_RUNNER_SOURCE_ARTIFACT_ID, BINUTILS_AR_RUNNER_SOURCE_BLAKE3),
        (ELF_SYMBOL_CANONICALIZER_SOURCE_ARTIFACT_ID, ELF_SYMBOL_CANONICALIZER_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_binutils_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<BinutilsSourceMaterializationReport, StagexBinutilsError> {
    validate_source_scratch(scratch_dir)?;
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading source bundle: {error}")))?;
    require_manifest_identity(&manifest.manifest_blake3, expected_manifest_blake3)?;
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating binutils source scratch: {error}")))?;
    let output_path = scratch_dir.join(BINUTILS_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("materializing binutils source: {error}")))?;
    validate_materialized_source(&output_path)?;
    let report = source_report(manifest.manifest_blake3, record, output_path);
    assert_eq!(report.record_identity, BINUTILS_RECORD_IDENTITY);
    assert!(report.output_path.join("bfd/configure").is_file());
    Ok(report)
}

fn source_report(
    manifest_blake3: String,
    record: &SourceRecord,
    output_path: PathBuf,
) -> BinutilsSourceMaterializationReport {
    let report = BinutilsSourceMaterializationReport {
        format: BINUTILS_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest_blake3,
        artifact_id: BINUTILS_SOURCE_ARTIFACT_ID,
        record_name: BINUTILS_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: BINUTILS_SOURCE_NON_CLAIM,
    };
    assert_eq!(report.artifact_id, BINUTILS_SOURCE_ARTIFACT_ID);
    assert_eq!(report.record_content_blake3, BINUTILS_SOURCE_CONTENT_BLAKE3);
    report
}

fn validate_source_scratch(path: &Path) -> Result<(), StagexBinutilsError> {
    if !path.is_absolute() || path.exists() {
        return Err(StagexBinutilsError::Materialization(format!(
            "source scratch must be an absent absolute path: {}",
            path.display()
        )));
    }
    assert!(path.is_absolute());
    assert!(!path.exists());
    Ok(())
}

fn validate_bundle_path(path: &Path) -> Result<(), StagexBinutilsError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(StagexBinutilsError::Materialization(format!(
            "source bundle must be an absolute regular file: {}",
            path.display()
        )));
    }
    let bytes_len = path
        .metadata()
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading source-bundle metadata: {error}")))?
        .len();
    if bytes_len == 0 {
        return Err(StagexBinutilsError::Materialization("source bundle is empty".to_string()));
    }
    assert!(path.is_absolute());
    assert!(bytes_len > 0);
    Ok(())
}

fn require_manifest_identity(observed: &str, expected: &str) -> Result<(), StagexBinutilsError> {
    if observed != expected {
        return Err(StagexBinutilsError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed, expected);
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexBinutilsError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(BINUTILS_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(StagexBinutilsError::SourceRecordNotFound);
    }
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some(BINUTILS_RECORD_NAME));
    Ok(matches[0])
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexBinutilsError> {
    require_record_fact(record.identity == BINUTILS_RECORD_IDENTITY, "source record identity")?;
    require_record_fact(record.kind == SourceRecordKind::FixedUrl, "source record kind")?;
    require_metadata(record, "builder", "builtin:fetchurl")?;
    require_metadata(record, "hash", BINUTILS_RECORD_HASH)?;
    require_metadata(record, "hash_algo", "sha256")?;
    require_metadata(record, "hash_mode", "recursive")?;
    require_metadata(record, "payload_encoding", BINUTILS_RECORD_PAYLOAD_ENCODING)?;
    require_metadata(record, "unpack", BINUTILS_RECORD_UNPACK)?;
    require_metadata(record, "url", BINUTILS_RECORD_URL)?;
    require_record_fact(record.content_blake3 == BINUTILS_SOURCE_CONTENT_BLAKE3, "source content BLAKE3")?;
    require_record_fact(record.files.len() == 1, "source record file count")?;
    validate_archive_record(&record.files[0])?;
    assert_eq!(record.identity, BINUTILS_RECORD_IDENTITY);
    assert_eq!(record.files.len(), 1);
    Ok(())
}

fn validate_archive_record(archive: &crate::source_bundle::SourceFileEntry) -> Result<(), StagexBinutilsError> {
    require_record_fact(archive.path == "archive", "source archive path")?;
    require_record_fact(archive.file_type == SourceFileType::Regular, "source archive file type")?;
    require_record_fact(!archive.executable, "source archive mode")?;
    require_record_fact(archive.size > 0, "source archive size")?;
    require_record_fact(archive.content_hex.is_some(), "source archive bytes")?;
    assert_eq!(archive.path, "archive");
    assert!(archive.size > 0);
    Ok(())
}

fn require_record_fact(condition: bool, label: &str) -> Result<(), StagexBinutilsError> {
    if !condition {
        return Err(StagexBinutilsError::Materialization(format!("unexpected binutils {label}")));
    }
    Ok(())
}

fn require_metadata(record: &SourceRecord, key: &str, expected: &str) -> Result<(), StagexBinutilsError> {
    let observed = record.metadata.get(key).map(String::as_str);
    if observed != Some(expected) {
        return Err(StagexBinutilsError::Materialization(format!(
            "unexpected binutils source metadata {key}: expected {expected}, observed {observed:?}"
        )));
    }
    Ok(())
}

fn validate_materialized_source(path: &Path) -> Result<(), StagexBinutilsError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(StagexBinutilsError::Materialization(format!(
            "materialized binutils source is not an absolute directory: {}",
            path.display()
        )));
    }
    for relative in REQUIRED_SOURCE_FILES {
        if !path.join(relative).is_file() {
            return Err(StagexBinutilsError::Materialization(format!(
                "materialized binutils source is missing {relative}"
            )));
        }
    }
    assert_eq!(REQUIRED_SOURCE_FILES.len(), REQUIRED_SOURCE_FILE_COUNT);
    assert!(path.join("opcodes/i386-gen.c").is_file());
    Ok(())
}

pub(crate) fn validate_recipe_digest() -> Result<(), StagexBinutilsError> {
    let observed = blake3::hash(BINUTILS_RECIPE).to_hex().to_string();
    if observed != BINUTILS_RECIPE_BLAKE3 {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils recipe BLAKE3 mismatch: expected {BINUTILS_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert_ne!(observed, BINUTILS_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BinutilsConfigureProbeRequest<'a> {
    pub source_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub bash: &'a Path,
    pub tcc: &'a Path,
    pub musl_root: &'a Path,
    pub coreutils_bin: &'a Path,
    pub sed: &'a Path,
    pub grep: &'a Path,
    pub diff: &'a Path,
    pub cmp: &'a Path,
    pub gawk: &'a Path,
    pub m4: &'a Path,
    pub bison: &'a Path,
    pub flex: &'a Path,
    pub make: &'a Path,
}

#[derive(Debug, Clone)]
struct SedBridgePaths {
    launcher: PathBuf,
    script: PathBuf,
    ylwrap_runner: PathBuf,
    spool_root: PathBuf,
    audit: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SedBridgeAuditEntry {
    invocation: u32,
    input_bytes: u64,
    output_bytes: u64,
    input_file_count: u32,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigureProbeOutcome {
    pub configure_class: String,
    pub exit_code: i32,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BinutilsOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BinutilsInventoryReport {
    pub format: &'static str,
    pub configure_class_count: u32,
    pub component_count: u32,
    pub installed_tool_count: u32,
    pub archive_count: u32,
    pub sed_invocation_count: u32,
    pub protected_exec_event_count_bounds: [u32; 2],
    pub component_outputs: Vec<BinutilsOutputReport>,
    pub installed_tools: Vec<BinutilsOutputReport>,
    pub runtime_root: PathBuf,
    pub install_root: PathBuf,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone)]
struct AuthenticatedBinutilsBuild {
    component_outputs: Vec<PathBuf>,
    install_root: PathBuf,
    sed_invocation_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConfigurePreprocessNegativeCase {
    label: &'static str,
    expected_reason: &'static str,
    configure_class: &'static str,
    source_name: &'static str,
    source_bytes_override: Option<usize>,
    count_override: Option<u32>,
    omit_authority: bool,
    mismatched_directory: bool,
}

#[derive(Debug, Clone)]
struct ArchiveRunnerPaths {
    script: PathBuf,
    scratch: PathBuf,
    audit: PathBuf,
    count: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchiveAuditEntry {
    invocation: u32,
    member_count: u32,
    output_bytes: u64,
    archive: String,
    status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentityValidationMode {
    Enforce,
    Observe,
}

#[derive(Debug, Clone)]
struct ConfigureProbeContext {
    source: PathBuf,
    bfd_configure_authenticated: PathBuf,
    tools: PathBuf,
    compiler_wrapper: PathBuf,
    configure_utility: PathBuf,
    sed_bridge: SedBridgePaths,
    archive_runner: ArchiveRunnerPaths,
    stdin_path: PathBuf,
    identity_validation_mode: IdentityValidationMode,
}

const COREUTILS_TOOL_NAMES: &[&str] = &[
    "basename", "cat", "chmod", "cp", "dirname", "echo", "expr", "false", "head", "install", "ln", "ls", "mkdir", "mv",
    "rm", "rmdir", "sort", "tail", "tee", "test", "touch", "tr", "true", "uniq", "wc",
];
const COREUTILS_TOOL_COUNT: usize = 25;
const PROBE_TOOL_ALIAS_NAMES: &[&str] = &[
    "sh", "bash", "cc", "sed", "sleep", "file", "emit", "grep", "egrep", "fgrep", "diff", "cmp", "awk", "gawk", "m4",
    "bison", "flex", "make",
];
const PROBE_TOOL_ALIAS_COUNT: usize = 18;
const BINUTILS_DIAGNOSTIC_EXEC_PATH_COUNT_MAX: usize = 128;
const BINUTILS_DIAGNOSTIC_UNIQUE_EXECUTABLE_COUNT_MAX: usize = 512;
const BINUTILS_DIAGNOSTIC_EXEC_EVENT_COUNT_MAX: usize = 131_072;
const BINUTILS_EXEC_OBSERVATION_SCHEMA: &str = "mantle-stagex-binutils-exec-observation-v1";
const CONFIGURE_OUTPUT_MEBIBYTES_MAX: u64 = 16;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const CONFIGURE_OUTPUT_BYTES_MAX: u64 = CONFIGURE_OUTPUT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const CONFIGURE_PROBE_SOURCE_KIBIBYTES_MAX: u64 = 64;
const CONFIGURE_PROBE_SOURCE_BYTES_MAX: u64 = CONFIGURE_PROBE_SOURCE_KIBIBYTES_MAX * KIBIBYTE_BYTES;
const CONFIGURE_PROBE_INVOCATION_COUNT_MAX: u32 = 4_096;
const CONFIGURE_PREPROCESS_NEGATIVE_CASE_COUNT: usize = 6;
const CONFIGURE_PREPROCESS_REJECTION_STATUS: i32 = 1;
const CONFIG_SUB_LEGACY_INPUT: &str = "sun4";
const CONFIG_SUB_EXPECTED_OUTPUT: &str = "sparc-sun-sunos4.1.1\n";
const MAKE_JOB_COUNT: u32 = 1;
const BINUTILS_CFLAGS_FEATURES: &str = "-static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1";
const BFD_HEADER_MARKER: &str = "void bfd_init (void);";
const BFD_LIBRARY_HEADER_MARKER: &str = "/* Extracted from libbfd.c.  */";
const BFD_COFF_HEADER_MARKER: &str = "generated from \"libcoff-in.h\" and \"coffcode.h\"";
const BISON_OUTPUT_MARKER: &str = "#define YYBISON_VERSION \"2.3\"";
const BISON_FILE_MARKER: &str = "A Bison parser, made by GNU Bison 2.3.";
const FLEX_OUTPUT_MARKER: &str = "#define YY_FLEX_MAJOR_VERSION 2";
const BINUTILS_GENERATED_TARGET_COUNT: usize = 8;
const BINUTILS_GENERATED_TARGETS: [&str; BINUTILS_GENERATED_TARGET_COUNT] = [
    "arparse.c",
    "defparse.c",
    "mcparse.c",
    "rcparse.c",
    "sysinfo.c",
    "arlex.c",
    "deflex.c",
    "syslex.c",
];
const LD_GENERATED_TARGET_COUNT: usize = 3;
const LD_GENERATED_TARGETS: [&str; LD_GENERATED_TARGET_COUNT] = ["deffilep.c", "ldgram.c", "ldlex.c"];
const BISON_GENERATED_FILE_COUNT: usize = 14;
const BISON_GENERATED_FILES: [&str; BISON_GENERATED_FILE_COUNT] = [
    "binutils/arparse.c",
    "binutils/arparse.h",
    "binutils/defparse.c",
    "binutils/defparse.h",
    "binutils/mcparse.c",
    "binutils/mcparse.h",
    "binutils/rcparse.c",
    "binutils/rcparse.h",
    "binutils/sysinfo.c",
    "binutils/sysinfo.h",
    "ld/deffilep.c",
    "ld/deffilep.h",
    "ld/ldgram.c",
    "ld/ldgram.h",
];
const FLEX_GENERATED_FILE_COUNT: usize = 4;
const FLEX_GENERATED_FILES: [&str; FLEX_GENERATED_FILE_COUNT] = [
    "binutils/arlex.c",
    "binutils/deflex.c",
    "binutils/syslex.c",
    "ld/ldlex.c",
];
pub(crate) const BINUTILS_GENERATED_SOURCE_OUTPUTS: &[&str] = &[
    "bfd/bfd-in2.h",
    "bfd/libbfd.h",
    "bfd/libcoff.h",
    "intl/plural.c",
    "binutils/arparse.c",
    "binutils/arparse.h",
    "binutils/defparse.c",
    "binutils/defparse.h",
    "binutils/mcparse.c",
    "binutils/mcparse.h",
    "binutils/rcparse.c",
    "binutils/rcparse.h",
    "binutils/sysinfo.c",
    "binutils/sysinfo.h",
    "ld/deffilep.c",
    "ld/deffilep.h",
    "ld/ldgram.c",
    "ld/ldgram.h",
    "binutils/arlex.c",
    "binutils/deflex.c",
    "binutils/syslex.c",
    "ld/ldlex.c",
];
const BINUTILS_REQUIRED_TOOL_COUNT: usize = 11;
pub(crate) const BINUTILS_REQUIRED_TOOLS: [(&str, &str); BINUTILS_REQUIRED_TOOL_COUNT] = [
    ("as", "36bb17408403b4fd8283bf80f78410ae76eedb4e1565f0fc6db0f7a8c0a1eac4"),
    ("ld", "e2939e05b0e115efa3530f66d60b63ef06627a1ba0c0c1a70ce71fabcf4ca158"),
    ("ar", "c5836470e484f7b9137abc3bbdffbec7155a7700fdf094662b4253a576e76600"),
    ("ranlib", "8c6d65ff0cc4b6936e6f93a016782890dd39e556ef2f99d1699d0071c6691533"),
    ("nm", "bd92671b478f6f88d3aea88d910335cb024079bebcf4432c8abba77a51d1b00b"),
    ("objcopy", "919f6ad3c798a023395ca4d1d4574f395c8316b5595b25fbdf3bdafc891552e3"),
    ("objdump", "f48859e9dbfb3594a92cc52ba2281879c7f02855a5679a03ed5349489a019ca1"),
    ("readelf", "9c13500d32b180628d9768d0c56c3eabdd242f35dee79119c60e7b52f4ad3e0d"),
    ("size", "812bd48e35d078b759192073cb2de2accaf9620c075c2ae24df9329c5623a7dd"),
    ("strings", "8ff50f5ca1c25ab91d259225eb8fc7ed85584e9071d732192a80d82f9c24866f"),
    ("strip", "d5505e1aa9e5b016a0b976b067aa0c7d096c84d0e875a41ff456be612d8ccc07"),
];
const BINUTILS_COMPONENT_COUNT: usize = 8;
pub(crate) const BINUTILS_COMPONENT_ARTIFACT_IDS: [&str; BINUTILS_COMPONENT_COUNT] = [
    "binutils-libiberty-archive",
    "binutils-zlib-archive",
    "binutils-bfd-archive",
    "binutils-opcodes-archive",
    "binutils-size-build-output",
    "binutils-as-build-output",
    "binutils-gprof-build-output",
    "binutils-ld-build-output",
];
const BINUTILS_COMPONENTS: [(&str, &str, bool, &str); BINUTILS_COMPONENT_COUNT] = [
    (
        "libiberty",
        "libiberty.a",
        false,
        "9e054fcfdd504399c978714e8013154c57eb51951ecbf6ae71b52ab550d3af01",
    ),
    ("zlib", "libz.a", false, "7cfe930119c77ef8bd7f34ec122de1a7f644851183cd7bc82dadc27b6bbb54b4"),
    ("bfd", ".libs/libbfd.a", false, "94a9d681cbacecb5edc26a100fd94b860332115f0adf65d89cffeb5a4038144e"),
    (
        "opcodes",
        ".libs/libopcodes.a",
        false,
        "d75ecd69a0f592d5b8fa4d9189d3af133c98c369ec5f89939e55371eb72a6dda",
    ),
    ("binutils", "size", true, "812bd48e35d078b759192073cb2de2accaf9620c075c2ae24df9329c5623a7dd"),
    ("gas", "as-new", true, "36bb17408403b4fd8283bf80f78410ae76eedb4e1565f0fc6db0f7a8c0a1eac4"),
    ("gprof", "gprof", true, "37321441797634fb44356c1cbfba5d4b59bb4fec97fd39f49df7124f258cbb9e"),
    ("ld", "ld-new", true, "e2939e05b0e115efa3530f66d60b63ef06627a1ba0c0c1a70ce71fabcf4ca158"),
];
const BFD_SECOND_CONFIG_UNRESOLVED_MARKER: &str = "@BFD_HOST_64_BIT@";
const SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX: u64 = KIBIBYTE_BYTES;
const YLWRAP_SED_FILE_MEBIBYTES_MAX: u64 = 8;
const YLWRAP_SED_FILE_BYTES_MAX: u64 = YLWRAP_SED_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const YLWRAP_SED_DYNAMIC_PROGRAM_COUNT: usize = 3;
const YLWRAP_SED_ARGUMENT_COUNT: usize = 9;
const YLWRAP_SED_EXPECTED_INVOCATION_COUNT: usize = 18;
const YLWRAP_SED_PATH_PROGRAM_INDEX: usize = 0;
const YLWRAP_SED_NAME_PROGRAM_INDEX: usize = 1;
const YLWRAP_SED_GUARD_PROGRAM_INDEX: usize = 2;
const SED_BRIDGE_AUTHORITY_FAILURE: i32 = 125;
const BINUTILS_AR_AUTHORITY_FAILURE: i32 = 125;
const BINUTILS_AR_SMOKE_SHORT_BYTES: &[u8] = b"abc";
const BINUTILS_AR_SMOKE_LONG_BYTES: &[u8] = b"long-payload";
const BINUTILS_AR_EXPECTED_INVOCATION_COUNT: usize = 4;
const BINUTILS_AR_EXPECTED_ARCHIVES: [&str; BINUTILS_AR_EXPECTED_INVOCATION_COUNT] =
    ["./libiberty.a", "libz.a", ".libs/libbfd.a", ".libs/libopcodes.a"];
const BINUTILS_AR_AUDIT_FIELD_COUNT: usize = 5;
const BINUTILS_AR_AUDIT_INVOCATION_INDEX: usize = 0;
const BINUTILS_AR_AUDIT_MEMBER_COUNT_INDEX: usize = 1;
const BINUTILS_AR_AUDIT_OUTPUT_BYTES_INDEX: usize = 2;
const BINUTILS_AR_AUDIT_ARCHIVE_INDEX: usize = 3;
const BINUTILS_AR_AUDIT_STATUS_INDEX: usize = 4;
const SED_BRIDGE_INVOCATION_COUNT_MAX: u32 = 8_192;
const BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS: [u32; 3] = [4_771, 4_772, 4_773];
const BINUTILS_INSTALL_SED_INVOCATION_COUNTS: [u32; 3] = [4_891, 4_892, 4_893];
const BINUTILS_SMOKE_OUTPUT_KIBIBYTES_MAX: u64 = 64;
const BINUTILS_SMOKE_OUTPUT_BYTES_MAX: u64 = BINUTILS_SMOKE_OUTPUT_KIBIBYTES_MAX * KIBIBYTE_BYTES;
const BINUTILS_SMOKE_EXIT_STATUS: i32 = 42;
pub(crate) const BINUTILS_BFD_CHEW_BLAKE3: &str = "8df489a85fdb18b2bcff0e78f6fd5462ac2bf24b0c0b425ce742049c8f16cecd";
pub(crate) const BINUTILS_POSITIVE_SMOKE_BLAKE3: &str =
    "4deb353a3e09f526f5c041a1614727deb0d8746fd72a23c521afbc16b6a5d974";
pub(crate) const BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS: [usize; 2] = [73_980, 74_066];
pub(crate) const BINUTILS_PROTECTED_EXEC_UNIQUE_IDENTITY_COUNT: usize = 68;
pub(crate) const BINUTILS_GENERATED_EXECUTABLE_COUNT: usize = 24;
pub(crate) const BINUTILS_FIXED_EXECUTABLE_EVENT_COUNT: usize = 12;
pub(crate) const BINUTILS_GENERATED_EXECUTABLES: [(&str, &str, &str); BINUTILS_GENERATED_EXECUTABLE_COUNT] = [
    ("bfd-a-out", "source/bfd/a.out", "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc"),
    (
        "bfd-conftest-daae",
        "source/bfd/conftest",
        "daae114d834a0f84cf389d094b713e73a35beb173bbe49fd5505721747d37317",
    ),
    (
        "bfd-conftest-e5e6",
        "source/bfd/conftest",
        "e5e602e06fba34c690ce4b1ddbeace13d33c269bc642b9c5b86a557669afa4e7",
    ),
    (
        "binutils-a-out",
        "source/binutils/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "binutils-conftest",
        "source/binutils/conftest",
        "e5e602e06fba34c690ce4b1ddbeace13d33c269bc642b9c5b86a557669afa4e7",
    ),
    (
        "binutils-sysinfo",
        "source/binutils/sysinfo",
        "33a57b54cf9ca0e7d3f52bf7ffa1c2ef274339514fbcedc0e825f1268357bac1",
    ),
    ("gas-a-out", "source/gas/a.out", "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc"),
    (
        "gprof-a-out",
        "source/gprof/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "intl-a-out",
        "source/intl/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "intl-conftest-5572",
        "source/intl/conftest",
        "5572ffef24deef821bcddc3af7c17c37d068b75c1f7df6274208afc0e6858622",
    ),
    (
        "intl-conftest-daae",
        "source/intl/conftest",
        "daae114d834a0f84cf389d094b713e73a35beb173bbe49fd5505721747d37317",
    ),
    ("ld-a-out", "source/ld/a.out", "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc"),
    (
        "ld-conftest",
        "source/ld/conftest",
        "daae114d834a0f84cf389d094b713e73a35beb173bbe49fd5505721747d37317",
    ),
    (
        "libiberty-a-out",
        "source/libiberty/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "libiberty-conftest-6800",
        "source/libiberty/conftest",
        "6800f0a5dd0357f810b9a89e1f80c58e156e089f00a14a8b370cdcd24934e6dc",
    ),
    (
        "libiberty-conftest-6dfb",
        "source/libiberty/conftest",
        "6dfbb9e993f93fc4229343aba8442f3db3cdc49fdc1f320d7c8c29c1f1c91dd9",
    ),
    (
        "libiberty-conftest-7dae",
        "source/libiberty/conftest",
        "7dae69a3a70594e98e4e972cf472603e4f5a397772deb02b0d5bebd60f2b61da",
    ),
    (
        "libiberty-conftest-b9cf",
        "source/libiberty/conftest",
        "b9cf04138573456b66e435463e4dcb848a67a07963ff9ce098ba587761b76d3b",
    ),
    (
        "libiberty-conftest-daae",
        "source/libiberty/conftest",
        "daae114d834a0f84cf389d094b713e73a35beb173bbe49fd5505721747d37317",
    ),
    (
        "libiberty-conftest-e5e6",
        "source/libiberty/conftest",
        "e5e602e06fba34c690ce4b1ddbeace13d33c269bc642b9c5b86a557669afa4e7",
    ),
    (
        "opcodes-a-out",
        "source/opcodes/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "opcodes-i386-gen",
        "source/opcodes/i386-gen",
        "350fb1765f3402bf34b3af71239b3942e67116e71f9d6621ba99aa8fa42497e7",
    ),
    (
        "zlib-a-out",
        "source/zlib/a.out",
        "8f928f6223ea972f54e8f14c7b14aba87af247f4f0d020058f82c04d57ae4fdc",
    ),
    (
        "zlib-conftest",
        "source/zlib/conftest",
        "daae114d834a0f84cf389d094b713e73a35beb173bbe49fd5505721747d37317",
    ),
];
pub(crate) const BINUTILS_FIXED_EXECUTABLE_EVENT_COUNTS: [(&str, &str, u32); BINUTILS_FIXED_EXECUTABLE_EVENT_COUNT] = [
    ("stagex-elf-local-symbol-canonicalizer", ELF_SYMBOL_CANONICALIZER_BLAKE3, 941),
    ("stagex-ylwrap-sed-runner", YLWRAP_SED_RUNNER_BLAKE3, 24),
    ("source/bfd/doc/chew", BINUTILS_BFD_CHEW_BLAKE3, 47),
    ("binutils-runtime-smoke/positive", BINUTILS_POSITIVE_SMOKE_BLAKE3, 1),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/ar",
        "c5836470e484f7b9137abc3bbdffbec7155a7700fdf094662b4253a576e76600",
        3,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/as",
        "36bb17408403b4fd8283bf80f78410ae76eedb4e1565f0fc6db0f7a8c0a1eac4",
        2,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/ld",
        "e2939e05b0e115efa3530f66d60b63ef06627a1ba0c0c1a70ce71fabcf4ca158",
        2,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/nm",
        "bd92671b478f6f88d3aea88d910335cb024079bebcf4432c8abba77a51d1b00b",
        1,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/objcopy",
        "919f6ad3c798a023395ca4d1d4574f395c8316b5595b25fbdf3bdafc891552e3",
        1,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/objdump",
        "f48859e9dbfb3594a92cc52ba2281879c7f02855a5679a03ed5349489a019ca1",
        1,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/ranlib",
        "8c6d65ff0cc4b6936e6f93a016782890dd39e556ef2f99d1699d0071c6691533",
        1,
    ),
    (
        "install-destdir/mantle/stagex/binutils-probe-output/bin/readelf",
        "9c13500d32b180628d9768d0c56c3eabdd242f35dee79119c60e7b52f4ad3e0d",
        1,
    ),
];
pub(crate) const BINUTILS_GENERATED_EXECUTABLE_EVENT_COUNTS: [(&str, u32); BINUTILS_GENERATED_EXECUTABLE_COUNT] = [
    ("bfd-a-out", 2),
    ("bfd-conftest-daae", 2),
    ("bfd-conftest-e5e6", 8),
    ("binutils-a-out", 1),
    ("binutils-conftest", 2),
    ("binutils-sysinfo", 1),
    ("gas-a-out", 1),
    ("gprof-a-out", 1),
    ("intl-a-out", 1),
    ("intl-conftest-5572", 1),
    ("intl-conftest-daae", 1),
    ("ld-a-out", 1),
    ("ld-conftest", 1),
    ("libiberty-a-out", 1),
    ("libiberty-conftest-6800", 1),
    ("libiberty-conftest-6dfb", 1),
    ("libiberty-conftest-7dae", 1),
    ("libiberty-conftest-b9cf", 1),
    ("libiberty-conftest-daae", 1),
    ("libiberty-conftest-e5e6", 3),
    ("opcodes-a-out", 1),
    ("opcodes-i386-gen", 1),
    ("zlib-a-out", 1),
    ("zlib-conftest", 1),
];
const BINUTILS_INSPECTION_SMOKE_COUNT: usize = 7;
const BINUTILS_POSITIVE_ASSEMBLY: &[u8] =
    b".global _start\n.text\n_start:\n  mov $60, %rax\n  mov $42, %rdi\n  syscall\n";
const BINUTILS_INVALID_ASSEMBLY: &[u8] = b".mantle-invalid-directive\n";
const BINUTILS_INVALID_OBJECT: &[u8] = b"not-an-object\n";
const BINUTILS_INVALID_ARCHIVE: &[u8] = b"not-an-archive\n";
const SED_BRIDGE_AUDIT_FIELD_COUNT: usize = 5;
const SED_BRIDGE_CANONICAL_FIELD_COUNT: usize = 4;
const SED_BRIDGE_CANONICAL_INPUT_INDEX: usize = 0;
const SED_BRIDGE_CANONICAL_OUTPUT_INDEX: usize = 1;
const SED_BRIDGE_CANONICAL_FILE_COUNT_INDEX: usize = 2;
const SED_BRIDGE_CANONICAL_STATUS_INDEX: usize = 3;
const SED_BRIDGE_SMOKE_EXPECTED: &[u8] = b"S[\"LTLIBOBJS\"]=\"\"\n";
const SED_BRIDGE_SMOKE_INPUT: &[u8] = b"LTLIBOBJS!%!_!# \n";
const SED_BRIDGE_FIRST_PROGRAM: &[u8] = b"h\ns/^/S[\"/; s/!.*/\"]=/\np\ng\ns/^[^!]*!//\n:repl\nt repl\ns/%!_!# $//\nt delim\n:nl\nh\ns/\\(.\\{148\\}\\).*/\\1/\nt more1\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\\\\n\"\\\\/\np\nn\nb repl\n:more1\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"\\\\/\np\ng\ns/.\\{148\\}//\nt nl\n:delim\nh\ns/\\(.\\{148\\}\\).*/\\1/\nt more2\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"/\np\nb\n:more2\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"\\\\/\np\ng\ns/.\\{148\\}//\nt delim\n";
const SED_BRIDGE_SECOND_PROGRAM: &[u8] = b"/^[^\"\"]/ {\n  N\n  s/\\n//\n}\n";
const TARGET: &str = "x86_64-unknown-linux-gnu";
const BINUTILS_RUNTIME_START: &str = "      $BB cat > \"$WORK/binutils-runtime.c\" <<'RUNTIME_EOF'\n";
const BINUTILS_RUNTIME_END: &str = "\nRUNTIME_EOF";
const ELF_MAGIC: &[u8] = b"\x7fELF";
const REGULAR_FILE_MODE: u32 = 0o644;
const EXECUTABLE_FILE_MODE: u32 = 0o755;
const EXECUTABLE_MODE_BITS: u32 = 0o111;
const NATIVE_HELPER_COMPILE_ARGUMENT_COUNT: usize = 6;
const ELF_CANONICALIZER_COMPILE_ARGUMENT_COUNT: usize = 5;
const ELF_CANONICALIZER_LINK_ARGUMENT_COUNT: usize = 10;
const ELF_CANONICALIZER_SMOKE_COMPILE_ARGUMENT_COUNT: usize = 4;
const BINUTILS_INSTALL_PREFIX: &str = "/mantle/stagex/binutils-probe-output";
const BINUTILS_INSTALL_PREFIX_RELATIVE: &str = "mantle/stagex/binutils-probe-output";
const BINUTILS_INSTALL_PREFIX_ARGUMENT: &str = "--prefix=/mantle/stagex/binutils-probe-output";
const BINUTILS_INSTALL_LIBDIR_ARGUMENT: &str = "--libdir=/mantle/stagex/binutils-probe-output/lib";
const CONFIGURE_CLASS_COUNT: usize = 9;
const CONFIGURE_CLASSES: [&str; CONFIGURE_CLASS_COUNT] = [
    "intl",
    "libiberty",
    "zlib",
    "bfd",
    "opcodes",
    "binutils",
    "gas",
    "gprof",
    "ld",
];
const LEGACY_ABSOLUTE_HOST_PROBE_BLOCK: &str = "/usr/bin/uname -p = `(/usr/bin/uname -p) 2>/dev/null || echo unknown`\n/bin/uname -X     = `(/bin/uname -X) 2>/dev/null     || echo unknown`\n\n/bin/arch              = `(/bin/arch) 2>/dev/null              || echo unknown`\n/usr/bin/arch -k       = `(/usr/bin/arch -k) 2>/dev/null       || echo unknown`\n/usr/convex/getsysinfo = `(/usr/convex/getsysinfo) 2>/dev/null || echo unknown`\n/usr/bin/hostinfo      = `(/usr/bin/hostinfo) 2>/dev/null      || echo unknown`\n/bin/machine           = `(/bin/machine) 2>/dev/null           || echo unknown`\n/usr/bin/oslevel       = `(/usr/bin/oslevel) 2>/dev/null       || echo unknown`\n/bin/universe          = `(/bin/universe) 2>/dev/null          || echo unknown`";
const DISABLED_ABSOLUTE_HOST_PROBE_BLOCK: &str = "/usr/bin/uname -p = unknown\n/bin/uname -X     = unknown\n\n/bin/arch              = unknown\n/usr/bin/arch -k       = unknown\n/usr/convex/getsysinfo = unknown\n/usr/bin/hostinfo      = unknown\n/bin/machine           = unknown\n/usr/bin/oslevel       = unknown\n/bin/universe          = unknown";
const CONFIGURE_DYNAMIC_EXEC_OLD: &str = "ac_try='./$ac_file'";
const CONFIGURE_DYNAMIC_EXEC_NEW: &str = "ac_try=\"$PWD/$ac_file\"";
const CONFIGURE_CONFTEST_EXEC_OLD: &str = "ac_try='./conftest$ac_exeext'";
const CONFIGURE_CONFTEST_EXEC_NEW: &str = "ac_try=\"$PWD/conftest$ac_exeext\"";
const CONFIGURE_LIBTOOL_EXEC_OLD: &str = "(./conftest; exit; )";
const CONFIGURE_LIBTOOL_EXEC_NEW: &str = "(\"$PWD/conftest\"; exit; )";
const CONFIGURE_LIBTOOL_EXEC_COUNT_PER_CLASS: usize = 2;
const BFD_CHEW_RELATIVE_EXEC: &str = "\t./$(MKDOC)";
const BFD_CHEW_ABSOLUTE_EXEC: &str = "\t$(CURDIR)/$(MKDOC)";
const BFD_CHEW_EXEC_OCCURRENCE_COUNT: usize = 25;
const OPCODES_I386_GEN_RELATIVE_EXEC: &str = "\t./i386-gen$(EXEEXT_FOR_BUILD) --srcdir $(srcdir)";
const OPCODES_I386_GEN_ABSOLUTE_EXEC: &str = "\t$(CURDIR)/i386-gen$(EXEEXT_FOR_BUILD) --srcdir $(srcdir)";
const OPCODES_I386_GEN_EXEC_OCCURRENCE_COUNT: usize = 1;
const BINUTILS_SYSINFO_RELATIVE_EXEC: &str = "\t./sysinfo$(EXEEXT_FOR_BUILD)";
const BINUTILS_SYSINFO_ABSOLUTE_EXEC: &str = "\t$(CURDIR)/sysinfo$(EXEEXT_FOR_BUILD)";
const BINUTILS_SYSINFO_EXEC_OCCURRENCE_COUNT: usize = 4;
const BFD_GEN_AOUT_RELATIVE_EXEC: &str = "\t./gen-aout host > aout-params.h";
const BFD_GEN_AOUT_ABSOLUTE_EXEC: &str = "\t$(CURDIR)/gen-aout host > aout-params.h";
const BFD_GEN_AOUT_EXEC_OCCURRENCE_COUNT: usize = 1;
const COMPONENT_GENERATOR_REWRITE_COUNT: usize = 4;
const CONFIGURE_UTILITY_SOURCE_NAME: &str = "stagex-configure-utility.c";
const CONFIGURE_UTILITY_OUTPUT_NAME: &str = "stagex-configure-utility";
const FILE_RELOCATION_CLASS_COUNT: usize = 7;
const FILE_RELOCATION_CLASSES: [&str; FILE_RELOCATION_CLASS_COUNT] =
    ["zlib", "bfd", "opcodes", "binutils", "gas", "gprof", "ld"];
const FILE_RELOCATION_OCCURRENCES_PER_CLASS: usize = 10;
const FILE_RELOCATION_OCCURRENCE_COUNT: usize = FILE_RELOCATION_CLASS_COUNT * FILE_RELOCATION_OCCURRENCES_PER_CLASS;
const AMBIENT_FILE_PATH: &str = "/usr/bin/file";
const DECLARED_FILE_PATH: &str = "$MANTLE_STAGE_X_FILE";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BinutilsObservedExecutable {
    resolved_path: PathBuf,
    digest_blake3: String,
    event_count: u32,
    tracee_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BinutilsExecObservationReport {
    schema_version: String,
    status: String,
    claim_boundary: String,
    execution_error: Option<String>,
    event_count: u32,
    observed_event_count: u32,
    denied_event_count: u32,
    unique_executables: Vec<BinutilsObservedExecutable>,
    denied_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
}

fn diagnostic_exec_observation_paths(
    request: &BinutilsConfigureProbeRequest<'_>,
) -> std::collections::BTreeSet<PathBuf> {
    let mut paths = std::collections::BTreeSet::new();
    insert_diagnostic_tool_paths(request, &mut paths);
    insert_diagnostic_generated_paths(request, &mut paths);
    assert!(!paths.is_empty());
    assert!(paths.len() <= BINUTILS_DIAGNOSTIC_EXEC_PATH_COUNT_MAX);
    assert!(paths.iter().all(|path| path.is_absolute()));
    paths
}

fn insert_diagnostic_tool_paths(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &mut std::collections::BTreeSet<PathBuf>,
) {
    for name in COREUTILS_TOOL_NAMES {
        paths.insert(request.coreutils_bin.join(name));
        paths.insert(request.scratch_dir.join("tools").join(name));
    }
    for name in PROBE_TOOL_ALIAS_NAMES {
        paths.insert(request.scratch_dir.join("tools").join(name));
    }
    for path in [
        request.bash,
        request.tcc,
        request.sed,
        request.grep,
        request.diff,
        request.cmp,
        request.gawk,
        request.m4,
        request.bison,
        request.flex,
        request.make,
    ] {
        paths.insert(path.to_path_buf());
    }
    assert_eq!(COREUTILS_TOOL_NAMES.len(), COREUTILS_TOOL_COUNT);
    assert_eq!(PROBE_TOOL_ALIAS_NAMES.len(), PROBE_TOOL_ALIAS_COUNT);
}

fn insert_diagnostic_generated_paths(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &mut std::collections::BTreeSet<PathBuf>,
) {
    let source = request.scratch_dir.join("source");
    for configure_class in CONFIGURE_CLASSES {
        paths.insert(source.join(configure_class).join("a.out"));
        paths.insert(source.join(configure_class).join("conftest"));
    }
    for name in [
        SED_BRIDGE_LAUNCHER_OUTPUT_NAME,
        YLWRAP_SED_RUNNER_OUTPUT_NAME,
        CONFIGURE_UTILITY_OUTPUT_NAME,
        ELF_SYMBOL_CANONICALIZER_OUTPUT_NAME,
    ] {
        paths.insert(request.scratch_dir.join(name));
    }
    paths.insert(source.join("bfd/doc/chew"));
    paths.insert(source.join("opcodes/i386-gen"));
    paths.insert(source.join("binutils/sysinfo"));
    paths.insert(source.join("bfd/gen-aout"));
    let install_bin = request.scratch_dir.join("install-destdir").join(BINUTILS_INSTALL_PREFIX_RELATIVE).join("bin");
    for (name, _) in BINUTILS_REQUIRED_TOOLS {
        paths.insert(install_bin.join(name));
    }
    paths.insert(request.scratch_dir.join("binutils-runtime-smoke/positive"));
    assert_eq!(CONFIGURE_CLASSES.len(), CONFIGURE_CLASS_COUNT);
    assert_eq!(BINUTILS_REQUIRED_TOOLS.len(), BINUTILS_REQUIRED_TOOL_COUNT);
}

fn binutils_exec_observation_report(
    events: &[crate::protected_exec::ProtectedSeccompAuditEvent],
    execution_error: Option<String>,
) -> Result<BinutilsExecObservationReport, StagexBinutilsError> {
    if events.len() > BINUTILS_DIAGNOSTIC_EXEC_EVENT_COUNT_MAX {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils diagnostic exec event count exceeds {BINUTILS_DIAGNOSTIC_EXEC_EVENT_COUNT_MAX}"
        )));
    }
    let mut grouped = std::collections::BTreeMap::new();
    let mut denied_events = Vec::new();
    let mut observed_event_count = 0_u32;
    for event in events {
        if event.policy_decision != "diagnostic-observed" {
            denied_events.push(event.clone());
            continue;
        }
        validate_observed_exec_event(event)?;
        let grouped_entry = grouped
            .entry((event.resolved_host_path.clone(), event.digest_hex.clone()))
            .or_insert_with(|| (0_u32, std::collections::BTreeSet::new()));
        grouped_entry.0 = grouped_entry.0.checked_add(1).ok_or_else(|| {
            StagexBinutilsError::Materialization("binutils grouped exec event count overflow".to_string())
        })?;
        grouped_entry.1.insert(event.tracee_path.clone());
        observed_event_count = observed_event_count.checked_add(1).ok_or_else(|| {
            StagexBinutilsError::Materialization("binutils observed exec event count overflow".to_string())
        })?;
    }
    let unique_executables = grouped
        .into_iter()
        .map(|((resolved_path, digest_blake3), (event_count, tracee_paths))| BinutilsObservedExecutable {
            resolved_path,
            digest_blake3,
            event_count,
            tracee_paths: tracee_paths.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    let event_count = u32::try_from(events.len())
        .map_err(|_| StagexBinutilsError::Materialization("binutils diagnostic event count exceeds u32".to_string()))?;
    let denied_event_count = u32::try_from(denied_events.len())
        .map_err(|_| StagexBinutilsError::Materialization("binutils denied event count exceeds u32".to_string()))?;
    let status = if execution_error.is_none() && denied_events.is_empty() {
        "complete-diagnostic-observation"
    } else {
        "blocked-diagnostic-observation"
    };
    assert_eq!(event_count, observed_event_count.saturating_add(denied_event_count));
    assert!(unique_executables.len() <= BINUTILS_DIAGNOSTIC_UNIQUE_EXECUTABLE_COUNT_MAX);
    Ok(BinutilsExecObservationReport {
        schema_version: BINUTILS_EXEC_OBSERVATION_SCHEMA.to_string(),
        status: status.to_string(),
        claim_boundary:
            "Diagnostic path-and-digest observation only; this report grants no protected execution authority."
                .to_string(),
        execution_error,
        event_count,
        observed_event_count,
        denied_event_count,
        unique_executables,
        denied_events,
    })
}

fn validate_observed_exec_event(
    event: &crate::protected_exec::ProtectedSeccompAuditEvent,
) -> Result<(), StagexBinutilsError> {
    let digest_valid = event.digest_hex.len() == BLAKE3_HEX_CHAR_COUNT
        && event.digest_hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if event.phase != "diagnostic" || event.inventory_entry_id.is_some() || !digest_valid {
        return Err(StagexBinutilsError::Materialization(format!(
            "invalid diagnostic exec event for {}",
            event.resolved_host_path.display()
        )));
    }
    if !event.tracee_path.is_absolute() || !event.resolved_host_path.is_absolute() {
        return Err(StagexBinutilsError::Materialization("diagnostic exec event contains a relative path".to_string()));
    }
    assert!(!event.digest_hex.is_empty());
    assert_eq!(event.policy_decision, "diagnostic-observed");
    Ok(())
}

pub(crate) fn probe_authenticated_intl_configure(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<i32, StagexBinutilsError> {
    let outcomes = probe_authenticated_configures(request, &["intl"], false)?;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].configure_class, "intl");
    Ok(outcomes[0].exit_code)
}

pub(crate) fn probe_authenticated_configure_matrix(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<Vec<ConfigureProbeOutcome>, StagexBinutilsError> {
    let outcomes = probe_authenticated_configures(request, &CONFIGURE_CLASSES, true)?;
    assert_eq!(outcomes.len(), CONFIGURE_CLASS_COUNT);
    assert!(outcomes.iter().all(|outcome| outcome.exit_code == 0));
    Ok(outcomes)
}

fn probe_authenticated_generated_sources(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<ConfigureProbeOutcome, StagexBinutilsError> {
    let (context, second_bfd) = prepare_generated_source_context(&request)?;
    finalize_sed_bridge_audit(&context.sed_bridge)?;
    validate_ylwrap_sed_audit(&context.sed_bridge)?;
    assert_eq!(second_bfd.configure_class, "bfd");
    assert!(context.source.join("ld/ldlex.c").is_file());
    Ok(second_bfd)
}

fn prepare_generated_source_context(
    request: &BinutilsConfigureProbeRequest<'_>,
) -> Result<(ConfigureProbeContext, ConfigureProbeOutcome), StagexBinutilsError> {
    prepare_generated_source_context_with_mode(request, IdentityValidationMode::Enforce)
}

fn prepare_generated_source_context_with_mode(
    request: &BinutilsConfigureProbeRequest<'_>,
    identity_validation_mode: IdentityValidationMode,
) -> Result<(ConfigureProbeContext, ConfigureProbeOutcome), StagexBinutilsError> {
    let context = prepare_configure_probe(request, identity_validation_mode)?;
    let smoke_environment = configure_environment(request, &context, CONFIGURE_CLASSES[0])?;
    run_sed_bridge_smokes(request, &context.sed_bridge, &smoke_environment)?;
    let outcomes = run_configure_classes(request, &context, &CONFIGURE_CLASSES, true)?;
    run_initial_generated_source_targets(request, &context)?;
    restore_authenticated_bfd_configure(&context)?;
    let second_bfd = run_configure_class_named(request, &context, "bfd", "bfd-second")?;
    if second_bfd.exit_code != 0 {
        return Err(StagexBinutilsError::Materialization(format!(
            "second authenticated BFD configure failed with status {}",
            second_bfd.exit_code
        )));
    }
    validate_second_bfd_header(&context.source.join("bfd/bfd-in3.h"))?;
    run_remaining_generated_source_targets(request, &context)?;
    assert_eq!(outcomes.len(), CONFIGURE_CLASS_COUNT);
    assert_eq!(second_bfd.configure_class, "bfd");
    Ok((context, second_bfd))
}

pub(crate) fn derive_binutils_inventory(
    request: BinutilsConfigureProbeRequest<'_>,
    protected_exec_enforced: bool,
) -> Result<BinutilsInventoryReport, StagexBinutilsError> {
    let build = probe_authenticated_component_builds_with_mode(request, IdentityValidationMode::Enforce)?;
    let component_outputs = component_output_reports(&build.component_outputs)?;
    let installed_tools = installed_tool_output_reports(&build.install_root)?;
    let report = BinutilsInventoryReport {
        format: BINUTILS_INVENTORY_FORMAT,
        configure_class_count: u32::try_from(CONFIGURE_CLASS_COUNT)
            .map_err(|_| StagexBinutilsError::Materialization("configure class count exceeds u32".to_string()))?,
        component_count: u32::try_from(component_outputs.len())
            .map_err(|_| StagexBinutilsError::Materialization("component count exceeds u32".to_string()))?,
        installed_tool_count: u32::try_from(installed_tools.len())
            .map_err(|_| StagexBinutilsError::Materialization("installed tool count exceeds u32".to_string()))?,
        archive_count: u32::try_from(BINUTILS_AR_EXPECTED_INVOCATION_COUNT)
            .map_err(|_| StagexBinutilsError::Materialization("archive count exceeds u32".to_string()))?,
        sed_invocation_count: build.sed_invocation_count,
        protected_exec_event_count_bounds: BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS
            .map(|count| u32::try_from(count).expect("bounded binutils protected exec event count fits u32")),
        component_outputs,
        installed_tools,
        runtime_root: request.scratch_dir.to_path_buf(),
        install_root: build.install_root,
        protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: BINUTILS_INVENTORY_NON_CLAIM,
    };
    write_binutils_inventory_report(request.scratch_dir, &report)?;
    assert_eq!(report.component_outputs.len(), BINUTILS_COMPONENT_COUNT);
    assert_eq!(report.installed_tools.len(), BINUTILS_REQUIRED_TOOL_COUNT);
    assert!(report.runtime_root.is_absolute());
    assert!(
        BINUTILS_GENERATED_EXECUTABLES
            .iter()
            .zip(BINUTILS_GENERATED_EXECUTABLE_EVENT_COUNTS.iter())
            .all(|((label, _, _), (count_label, count))| label == count_label && *count > 0)
    );
    assert_eq!(
        report.protected_exec_event_count_bounds,
        BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS.map(|count| u32::try_from(count).unwrap())
    );
    Ok(report)
}

fn probe_authenticated_component_builds(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<Vec<PathBuf>, StagexBinutilsError> {
    let build = probe_authenticated_component_builds_with_mode(request, IdentityValidationMode::Enforce)?;
    Ok(build.component_outputs)
}

fn probe_authenticated_component_builds_observing_identities(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<Vec<PathBuf>, StagexBinutilsError> {
    let build = probe_authenticated_component_builds_with_mode(request, IdentityValidationMode::Observe)?;
    Ok(build.component_outputs)
}

fn probe_authenticated_component_builds_with_mode(
    request: BinutilsConfigureProbeRequest<'_>,
    identity_validation_mode: IdentityValidationMode,
) -> Result<AuthenticatedBinutilsBuild, StagexBinutilsError> {
    let (context, second_bfd) = prepare_generated_source_context_with_mode(&request, identity_validation_mode)?;
    let component_outputs = run_component_builds(&request, &context)?;
    validate_sed_bridge_total_count(&context.sed_bridge, &BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS)?;
    let install_root = run_component_installs(&request, &context)?;
    run_installed_tool_smokes(&request, &install_root)?;
    validate_sed_bridge_total_count(&context.sed_bridge, &BINUTILS_INSTALL_SED_INVOCATION_COUNTS)?;
    let sed_invocation_count = read_sed_bridge_total_count(&context.sed_bridge)?;
    finalize_archive_runner_audit(&context.archive_runner)?;
    finalize_sed_bridge_audit(&context.sed_bridge)?;
    validate_ylwrap_sed_audit(&context.sed_bridge)?;
    assert_eq!(second_bfd.exit_code, 0);
    assert_eq!(component_outputs.len(), BINUTILS_COMPONENT_COUNT);
    assert!(install_root.join("bin/ld").is_file());
    Ok(AuthenticatedBinutilsBuild {
        component_outputs,
        install_root,
        sed_invocation_count,
    })
}

fn probe_authenticated_configures(
    request: BinutilsConfigureProbeRequest<'_>,
    configure_classes: &[&str],
    require_success: bool,
) -> Result<Vec<ConfigureProbeOutcome>, StagexBinutilsError> {
    validate_configure_classes(configure_classes)?;
    let context = prepare_configure_probe(&request, IdentityValidationMode::Enforce)?;
    let smoke_environment = configure_environment(&request, &context, configure_classes[0])?;
    run_sed_bridge_smokes(&request, &context.sed_bridge, &smoke_environment)?;
    let outcomes = run_configure_classes(&request, &context, configure_classes, require_success)?;
    finalize_sed_bridge_audit(&context.sed_bridge)?;
    assert_eq!(outcomes.len(), configure_classes.len());
    assert!(!outcomes.is_empty());
    Ok(outcomes)
}

fn run_configure_classes(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    configure_classes: &[&str],
    require_success: bool,
) -> Result<Vec<ConfigureProbeOutcome>, StagexBinutilsError> {
    let mut outcomes = Vec::with_capacity(configure_classes.len());
    for configure_class in configure_classes {
        let outcome = run_configure_class(request, context, configure_class)?;
        if require_success && outcome.exit_code != 0 {
            return Err(StagexBinutilsError::Materialization(format!(
                "authenticated configure failed for {configure_class} with status {}",
                outcome.exit_code
            )));
        }
        outcomes.push(outcome);
    }
    assert_eq!(outcomes.len(), configure_classes.len());
    assert!(!outcomes.is_empty());
    Ok(outcomes)
}

fn prepare_configure_probe(
    request: &BinutilsConfigureProbeRequest<'_>,
    identity_validation_mode: IdentityValidationMode,
) -> Result<ConfigureProbeContext, StagexBinutilsError> {
    validate_configure_probe_request(request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure probe root: {error}")))?;
    let source = request.scratch_dir.join("source");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source)
        .map_err(StagexBinutilsError::from_runtime)?;
    let bfd_configure_authenticated = prepare_recipe_source(request.scratch_dir, &source)?;
    relocate_configure_file_utility(&source)?;
    let tools = request.scratch_dir.join("tools");
    fs::create_dir(&tools)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure tool namespace: {error}")))?;
    let compiler_wrapper = prepare_tcc_wrapper(request)?;
    run_tcc_wrapper_negative_smokes(request, &compiler_wrapper)?;
    let archive_runner = prepare_archive_runner(request)?;
    let sed_bridge = prepare_sed_bridge(request, &compiler_wrapper)?;
    let configure_utility = prepare_configure_utility(request, &compiler_wrapper)?;
    populate_probe_tool_namespace(request, &tools, &sed_bridge.launcher, &configure_utility)?;
    run_configure_utility_smokes(request, &tools, &configure_utility)?;
    let stdin_path = request.scratch_dir.join("configure.stdin");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    let context = ConfigureProbeContext {
        source,
        bfd_configure_authenticated,
        tools,
        compiler_wrapper,
        configure_utility,
        sed_bridge,
        archive_runner,
        stdin_path,
        identity_validation_mode,
    };
    run_config_sub_preflight(request, &context)?;
    assert!(context.source.is_dir());
    assert!(context.bfd_configure_authenticated.is_file());
    assert!(context.tools.is_dir());
    Ok(context)
}

fn run_config_sub_preflight(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<(), StagexBinutilsError> {
    let mut environment = std::collections::BTreeMap::new();
    environment.insert("PATH".to_string(), shell_path(&context.tools, "config.sub tool namespace")?);
    environment.insert("CONFIG_SHELL".to_string(), shell_path(request.bash, "config.sub Bash")?);
    environment.insert("SHELL".to_string(), shell_path(request.bash, "config.sub Bash")?);
    append_sed_bridge_environment(&mut environment, request, context)?;
    let stdout = request.scratch_dir.join("config-sub-sun4.stdout.txt");
    let stderr = request.scratch_dir.join("config-sub-sun4.stderr.txt");
    let arguments = vec![
        shell_path(&context.source.join("config.sub"), "authenticated config.sub")?,
        CONFIG_SUB_LEGACY_INPUT.to_string(),
    ];
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &arguments,
        &context.source,
        &environment,
        &context.stdin_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let output = fs::read_to_string(&stdout).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading config.sub preflight output: {error}"))
    })?;
    let diagnostic = fs::read_to_string(&stderr).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading config.sub preflight diagnostic: {error}"))
    })?;
    if status != 0 || output != CONFIG_SUB_EXPECTED_OUTPUT || diagnostic.contains("command not found") {
        return Err(StagexBinutilsError::Materialization(format!(
            "authenticated config.sub preflight failed: status {status}, output {output:?}, diagnostic {diagnostic:?}"
        )));
    }
    assert!(!output.is_empty());
    assert!(!diagnostic.contains("No such file"));
    Ok(())
}

fn run_initial_generated_source_targets(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<(), StagexBinutilsError> {
    let environment = generated_source_environment(request, context)?;
    run_make_generation_targets(request, context, &environment, "bfd", &["headers"], "make-bfd-headers")?;
    validate_generated_file(&context.source.join("bfd/bfd-in2.h"), BFD_HEADER_MARKER, "first BFD header")?;
    validate_generated_file(&context.source.join("bfd/libbfd.h"), BFD_LIBRARY_HEADER_MARKER, "BFD library header")?;
    validate_generated_file(&context.source.join("bfd/libcoff.h"), BFD_COFF_HEADER_MARKER, "BFD COFF header")?;
    let chew = context.source.join("bfd/doc/chew");
    let chew_mode = fs::metadata(&chew)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading BFD chew metadata: {error}")))?
        .permissions()
        .mode();
    if chew_mode & EXECUTABLE_MODE_BITS == 0 {
        return Err(StagexBinutilsError::Materialization("BFD chew generator is not executable".to_string()));
    }
    validate_file_digest(&chew, BINUTILS_BFD_CHEW_BLAKE3, "BFD chew generator")?;
    run_make_generation_targets(request, context, &environment, "intl", &["plural.c"], "make-intl-plural")?;
    validate_generated_file(&context.source.join("intl/plural.c"), BISON_OUTPUT_MARKER, "intl plural parser")?;
    assert!(chew.is_file());
    assert!(context.source.join("intl/plural.c").is_file());
    Ok(())
}

fn run_remaining_generated_source_targets(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<(), StagexBinutilsError> {
    let environment = generated_source_environment(request, context)?;
    run_make_generation_targets(
        request,
        context,
        &environment,
        "binutils",
        &BINUTILS_GENERATED_TARGETS,
        "make-binutils-generated",
    )?;
    run_make_generation_targets(request, context, &environment, "ld", &LD_GENERATED_TARGETS, "make-ld-generated")?;
    for relative in BISON_GENERATED_FILES {
        validate_generated_file(&context.source.join(relative), BISON_FILE_MARKER, relative)?;
    }
    for relative in FLEX_GENERATED_FILES {
        validate_generated_file(&context.source.join(relative), FLEX_OUTPUT_MARKER, relative)?;
    }
    assert_eq!(BINUTILS_GENERATED_TARGETS.len(), BINUTILS_GENERATED_TARGET_COUNT);
    assert_eq!(LD_GENERATED_TARGETS.len(), LD_GENERATED_TARGET_COUNT);
    Ok(())
}

fn generated_source_environment(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<std::collections::BTreeMap<String, String>, StagexBinutilsError> {
    let bash = shell_path(request.bash, "generated-source Bash")?;
    let wrapper = shell_path(&context.compiler_wrapper, "generated-source compiler wrapper")?;
    let compiler = format!("{bash} {wrapper}");
    let musl_include = shell_path(&request.musl_root.join("include"), "native musl include")?;
    let musl_lib = shell_path(&request.musl_root.join("lib"), "native musl lib")?;
    let bison_root = request
        .bison
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| StagexBinutilsError::Materialization("Bison path lacks an installation root".to_string()))?;
    let mut environment = std::collections::BTreeMap::new();
    environment.insert("PATH".to_string(), shell_path(&context.tools, "generated-source tool namespace")?);
    environment.insert("CONFIG_SHELL".to_string(), bash.clone());
    environment.insert("SHELL".to_string(), bash.clone());
    environment.insert("CC".to_string(), compiler.clone());
    environment.insert("CC_FOR_BUILD".to_string(), compiler.clone());
    environment.insert("CPP".to_string(), format!("{compiler} -E"));
    environment.insert(
        "AR".to_string(),
        format!("{bash} {}", shell_path(&context.archive_runner.script, "binutils ar runner")?),
    );
    environment.insert("RANLIB".to_string(), "true".to_string());
    environment.insert("MAKEINFO".to_string(), "true".to_string());
    environment.insert("AWK".to_string(), shell_path(request.gawk, "generated-source Gawk")?);
    environment.insert("M4".to_string(), shell_path(request.m4, "generated-source M4")?);
    environment.insert("BISON".to_string(), shell_path(request.bison, "generated-source Bison")?);
    environment.insert("FLEX".to_string(), shell_path(request.flex, "generated-source Flex")?);
    environment
        .insert("BISON_PKGDATADIR".to_string(), shell_path(&bison_root.join("share/bison"), "Bison runtime data")?);
    let cflags = format!("-I{musl_include} {BINUTILS_CFLAGS_FEATURES}");
    environment.insert("CFLAGS".to_string(), cflags.clone());
    environment.insert("CFLAGS_FOR_BUILD".to_string(), cflags);
    environment.insert("LDFLAGS".to_string(), format!("-static -L{musl_lib}"));
    environment.insert("MANTLE_STAGE_X_FILE".to_string(), shell_path(&context.tools.join("file"), "configure file")?);
    append_sed_bridge_environment(&mut environment, request, context)?;
    append_archive_runner_environment(&mut environment, request, context)?;
    assert!(environment.contains_key("CFLAGS_FOR_BUILD"));
    assert!(environment.contains_key("BISON_PKGDATADIR"));
    Ok(environment)
}

fn run_make_generation_targets(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    environment: &std::collections::BTreeMap<String, String>,
    subdirectory: &str,
    targets: &[&str],
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let compiler = environment.get("CC").ok_or_else(|| {
        StagexBinutilsError::Materialization("generated-source compiler authority is missing".to_string())
    })?;
    let cflags_for_build = environment
        .get("CFLAGS_FOR_BUILD")
        .ok_or_else(|| StagexBinutilsError::Materialization("generated-source build flags are missing".to_string()))?;
    let archive_runner = environment.get("AR").ok_or_else(|| {
        StagexBinutilsError::Materialization("generated-source archive authority is missing".to_string())
    })?;
    let shell_assignment = make_shell_assignment(environment)?;
    if targets.is_empty() {
        return Err(StagexBinutilsError::Materialization(format!("{label} has no declared Make targets")));
    }
    let mut arguments = vec![
        format!("-j{MAKE_JOB_COUNT}"),
        "-C".to_string(),
        shell_path(&context.source.join(subdirectory), "generated-source directory")?,
    ];
    arguments.extend(targets.iter().map(|target| (*target).to_string()));
    arguments.extend([
        format!("CC={compiler}"),
        format!("CC_FOR_BUILD={compiler}"),
        format!("CFLAGS_FOR_BUILD={cflags_for_build}"),
        format!("AR={archive_runner}"),
        shell_assignment,
        "RANLIB=true".to_string(),
        "MAKEINFO=true".to_string(),
    ]);
    let stdout = request.scratch_dir.join(format!("{label}.stdout.txt"));
    let stderr = request.scratch_dir.join(format!("{label}.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.make,
        &arguments,
        &context.source,
        environment,
        &context.stdin_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stdout,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let diagnostic = fs::read_to_string(&stderr)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading {label} diagnostic: {error}")))?;
    if status != 0 || diagnostic.contains("No such file") || diagnostic.contains("include file 'assert.h' not found") {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} failed: status {status}, diagnostic {diagnostic:?}"
        )));
    }
    assert!(!targets.is_empty());
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(())
}

fn run_component_builds(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<Vec<PathBuf>, StagexBinutilsError> {
    let environment = generated_source_environment(request, context)?;
    let mut outputs = Vec::with_capacity(BINUTILS_COMPONENT_COUNT);
    for (subdirectory, output, executable, expected_digest) in BINUTILS_COMPONENTS {
        outputs.push(run_component_build(
            request,
            context,
            &environment,
            subdirectory,
            output,
            executable,
            expected_digest,
        )?);
    }
    assert_eq!(outputs.len(), BINUTILS_COMPONENT_COUNT);
    assert!(outputs.iter().all(|output| output.is_file()));
    Ok(outputs)
}

fn run_component_build(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    environment: &std::collections::BTreeMap<String, String>,
    subdirectory: &str,
    expected_output: &str,
    executable: bool,
    expected_digest: &str,
) -> Result<PathBuf, StagexBinutilsError> {
    let compiler = environment
        .get("CC")
        .ok_or_else(|| StagexBinutilsError::Materialization("component compiler authority is missing".to_string()))?;
    let archive_runner = environment
        .get("AR")
        .ok_or_else(|| StagexBinutilsError::Materialization("component archive authority is missing".to_string()))?;
    let cflags = environment
        .get("CFLAGS")
        .ok_or_else(|| StagexBinutilsError::Materialization("component C flags are missing".to_string()))?;
    let shell_assignment = make_shell_assignment(environment)?;
    let arguments = [
        format!("-j{MAKE_JOB_COUNT}"),
        "-C".to_string(),
        shell_path(&context.source.join(subdirectory), "component source directory")?,
        format!("tooldir={BINUTILS_INSTALL_PREFIX}"),
        format!("CC={compiler}"),
        format!("AR={archive_runner}"),
        shell_assignment,
        "RANLIB=true".to_string(),
        "MAKEINFO=true".to_string(),
        "CPPFLAGS=-DPLUGIN_LITTLE_ENDIAN".to_string(),
        format!("CFLAGS={cflags}"),
    ];
    let argument_refs = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    let stdout = request.scratch_dir.join(format!("make-{subdirectory}-build.stdout.txt"));
    let stderr = request.scratch_dir.join(format!("make-{subdirectory}-build.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.make,
        &argument_refs,
        &context.source,
        environment,
        &context.stdin_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stdout,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    if status != 0 {
        let diagnostic = fs::read_to_string(&stderr).unwrap_or_else(|error| format!("unreadable diagnostic: {error}"));
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {subdirectory} failed with status {status}: {diagnostic}"
        )));
    }
    let output = context.source.join(subdirectory).join(expected_output);
    if !executable {
        canonicalize_component_archive(&output, subdirectory)?;
    }
    validate_component_output_with_mode(
        &output,
        subdirectory,
        executable,
        expected_digest,
        context.identity_validation_mode,
    )?;
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(output)
}

fn canonicalize_component_archive(path: &Path, label: &str) -> Result<(), StagexBinutilsError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading {label} archive metadata: {error}")))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {label} archive is not a regular file"
        )));
    }
    let input = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, label)
        .map_err(StagexBinutilsError::from_runtime)?;
    let canonical = crate::stagex_archive_core::canonicalize_stagex_archive_elf_members(&input).map_err(|error| {
        StagexBinutilsError::Materialization(format!("canonicalizing binutils component {label} archive: {error}"))
    })?;
    let repeated =
        crate::stagex_archive_core::canonicalize_stagex_archive_elf_members(&canonical.bytes).map_err(|error| {
            StagexBinutilsError::Materialization(format!("rechecking binutils component {label} archive: {error}"))
        })?;
    if canonical.bytes != repeated.bytes {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {label} archive canonicalization is not idempotent"
        )));
    }
    if canonical.bytes != input {
        publish_component_archive(path, &metadata, &canonical.bytes, label)?;
    }
    assert!(canonical.member_count > 0);
    assert_eq!(canonical.bytes, repeated.bytes);
    Ok(())
}

fn publish_component_archive(
    path: &Path,
    metadata: &fs::Metadata,
    bytes: &[u8],
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let staged = path.with_extension(ARCHIVE_CANONICAL_TEMP_EXTENSION);
    let write_result = (|| -> Result<(), StagexBinutilsError> {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&staged).map_err(|error| {
            StagexBinutilsError::Materialization(format!("creating {label} canonical archive: {error}"))
        })?;
        file.write_all(bytes).map_err(|error| {
            StagexBinutilsError::Materialization(format!("writing {label} canonical archive: {error}"))
        })?;
        file.set_permissions(metadata.permissions()).map_err(|error| {
            StagexBinutilsError::Materialization(format!("setting {label} canonical archive mode: {error}"))
        })?;
        file.sync_all().map_err(|error| {
            StagexBinutilsError::Materialization(format!("syncing {label} canonical archive: {error}"))
        })?;
        fs::rename(&staged, path).map_err(|error| {
            StagexBinutilsError::Materialization(format!("publishing {label} canonical archive: {error}"))
        })?;
        Ok(())
    })();
    if write_result.is_err() && staged.exists() {
        let _ = fs::remove_file(&staged);
    }
    write_result?;
    assert!(!bytes.is_empty());
    debug_assert!(!staged.exists());
    Ok(())
}

fn make_shell_assignment(
    environment: &std::collections::BTreeMap<String, String>,
) -> Result<String, StagexBinutilsError> {
    let shell = environment
        .get("SHELL")
        .ok_or_else(|| StagexBinutilsError::Materialization("Make shell authority is missing".to_string()))?;
    let shell_path = Path::new(shell);
    if !shell_path.is_absolute() || shell.contains(['\n', '\r']) {
        return Err(StagexBinutilsError::Materialization(format!(
            "Make shell authority must be one absolute line: {shell:?}"
        )));
    }
    let assignment = format!("SHELL={shell}");
    assert!(assignment.starts_with("SHELL=/"));
    assert!(!assignment.contains('\n'));
    Ok(assignment)
}

fn validate_component_output(
    path: &Path,
    label: &str,
    executable: bool,
    expected_digest: &str,
) -> Result<(), StagexBinutilsError> {
    validate_component_output_with_mode(path, label, executable, expected_digest, IdentityValidationMode::Enforce)
}

fn validate_component_output_with_mode(
    path: &Path,
    label: &str,
    executable: bool,
    expected_digest: &str,
    identity_validation_mode: IdentityValidationMode,
) -> Result<(), StagexBinutilsError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, label)
        .map_err(StagexBinutilsError::from_runtime)?;
    let mode = fs::metadata(path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading {label} output mode: {error}")))?
        .permissions()
        .mode();
    validate_component_output_facts_with_mode(
        &bytes,
        label,
        executable,
        mode,
        expected_digest,
        identity_validation_mode,
    )
}

fn validate_component_output_facts(
    bytes: &[u8],
    label: &str,
    executable: bool,
    mode: u32,
    expected_digest: &str,
) -> Result<(), StagexBinutilsError> {
    validate_component_output_facts_with_mode(
        bytes,
        label,
        executable,
        mode,
        expected_digest,
        IdentityValidationMode::Enforce,
    )
}

fn validate_component_output_facts_with_mode(
    bytes: &[u8],
    label: &str,
    executable: bool,
    mode: u32,
    expected_digest: &str,
    identity_validation_mode: IdentityValidationMode,
) -> Result<(), StagexBinutilsError> {
    if executable {
        if bytes.len() < ELF_MAGIC.len() || &bytes[..ELF_MAGIC.len()] != ELF_MAGIC || mode & EXECUTABLE_MODE_BITS == 0 {
            return Err(StagexBinutilsError::Materialization(format!(
                "binutils component {label} lacks an executable ELF output"
            )));
        }
    } else if !bytes.starts_with(b"!<arch>\n") {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {label} lacks an archive output"
        )));
    }
    let observed_digest = blake3::hash(bytes).to_hex().to_string();
    if identity_validation_mode == IdentityValidationMode::Enforce && observed_digest != expected_digest {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {label} digest mismatch: expected {expected_digest}, observed {observed_digest}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed_digest.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(())
}

fn component_output_reports(outputs: &[PathBuf]) -> Result<Vec<BinutilsOutputReport>, StagexBinutilsError> {
    if outputs.len() != BINUTILS_COMPONENT_COUNT {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component report count mismatch: expected {BINUTILS_COMPONENT_COUNT}, observed {}",
            outputs.len()
        )));
    }
    let reports = outputs
        .iter()
        .zip(BINUTILS_COMPONENT_ARTIFACT_IDS)
        .zip(BINUTILS_COMPONENTS)
        .map(|((path, artifact_id), (_, _, _, expected_digest))| output_report(artifact_id, path, expected_digest))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(reports.len(), BINUTILS_COMPONENT_COUNT);
    assert!(reports.iter().all(|report| report.bytes_len > 0));
    Ok(reports)
}

fn installed_tool_output_reports(install_root: &Path) -> Result<Vec<BinutilsOutputReport>, StagexBinutilsError> {
    let reports = BINUTILS_REQUIRED_TOOLS
        .iter()
        .map(|(tool, expected_digest)| {
            output_report(&format!("binutils-installed-{tool}"), &install_root.join("bin").join(tool), expected_digest)
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(reports.len(), BINUTILS_REQUIRED_TOOL_COUNT);
    assert!(reports.iter().all(|report| report.path.starts_with(install_root)));
    Ok(reports)
}

fn output_report(
    artifact_id: &str,
    path: &Path,
    expected_digest: &str,
) -> Result<BinutilsOutputReport, StagexBinutilsError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, artifact_id)
        .map_err(StagexBinutilsError::from_runtime)?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    if digest_blake3 != expected_digest {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils report {artifact_id} digest mismatch: expected {expected_digest}, observed {digest_blake3}"
        )));
    }
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexBinutilsError::Materialization(format!("binutils report {artifact_id} size exceeds u64")))?;
    assert!(bytes_len > 0);
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(BinutilsOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn write_binutils_inventory_report(
    scratch_dir: &Path,
    report: &BinutilsInventoryReport,
) -> Result<(), StagexBinutilsError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| StagexBinutilsError::Materialization(format!("serializing binutils inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&scratch_dir.join("binutils-inventory.json"), &bytes)
        .map_err(StagexBinutilsError::from_runtime)?;
    assert!(!bytes.is_empty());
    assert_eq!(report.format, BINUTILS_INVENTORY_FORMAT);
    Ok(())
}

fn run_component_installs(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<PathBuf, StagexBinutilsError> {
    let destination = request.scratch_dir.join("install-destdir");
    fs::create_dir(&destination)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating install destination: {error}")))?;
    let environment = generated_source_environment(request, context)?;
    for (subdirectory, _, _, _) in BINUTILS_COMPONENTS {
        run_component_install(request, context, &environment, &destination, subdirectory)?;
    }
    let install_root = destination.join(BINUTILS_INSTALL_PREFIX_RELATIVE);
    validate_installed_tools_with_mode(request, &install_root, context.identity_validation_mode)?;
    materialize_triplet_tool_links(&install_root)?;
    assert!(destination.is_dir());
    assert!(install_root.join("bin").is_dir());
    Ok(install_root)
}

fn run_component_install(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    environment: &std::collections::BTreeMap<String, String>,
    destination: &Path,
    subdirectory: &str,
) -> Result<(), StagexBinutilsError> {
    let shell_assignment = make_shell_assignment(environment)?;
    let arguments = [
        format!("-j{MAKE_JOB_COUNT}"),
        "-C".to_string(),
        shell_path(&context.source.join(subdirectory), "install source directory")?,
        format!("tooldir={BINUTILS_INSTALL_PREFIX}"),
        format!("DESTDIR={}", shell_path(destination, "install destination")?),
        format!("prefix={BINUTILS_INSTALL_PREFIX}"),
        shell_assignment,
        "MAKEINFO=true".to_string(),
        "install".to_string(),
    ];
    let argument_refs = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    let stdout = request.scratch_dir.join(format!("make-{subdirectory}-install.stdout.txt"));
    let stderr = request.scratch_dir.join(format!("make-{subdirectory}-install.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.make,
        &argument_refs,
        &context.source,
        environment,
        &context.stdin_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stdout,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    if status != 0 {
        let diagnostic = fs::read_to_string(&stderr).unwrap_or_else(|error| format!("unreadable diagnostic: {error}"));
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils component {subdirectory} install failed with status {status}: {diagnostic}"
        )));
    }
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(())
}

fn validate_installed_tools(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
) -> Result<(), StagexBinutilsError> {
    validate_installed_tools_with_mode(request, install_root, IdentityValidationMode::Enforce)
}

fn validate_installed_tools_with_mode(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
    identity_validation_mode: IdentityValidationMode,
) -> Result<(), StagexBinutilsError> {
    let predecessor = request.tcc.as_os_str().as_encoded_bytes();
    if predecessor.is_empty() {
        return Err(StagexBinutilsError::Materialization("predecessor TinyCC path is empty".to_string()));
    }
    for (tool, expected_digest) in BINUTILS_REQUIRED_TOOLS {
        let path = install_root.join("bin").join(tool);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| StagexBinutilsError::Materialization(format!("reading installed {tool}: {error}")))?;
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, CONFIGURE_OUTPUT_BYTES_MAX, tool)
            .map_err(StagexBinutilsError::from_runtime)?;
        validate_installed_tool_facts_with_mode(
            tool,
            &bytes,
            metadata.file_type().is_file(),
            metadata.permissions().mode(),
            predecessor,
            expected_digest,
            identity_validation_mode,
        )?;
    }
    assert!(!predecessor.is_empty());
    assert_eq!(BINUTILS_REQUIRED_TOOLS.len(), BINUTILS_REQUIRED_TOOL_COUNT);
    Ok(())
}

fn materialize_triplet_tool_links(install_root: &Path) -> Result<(), StagexBinutilsError> {
    let bin = install_root.join("bin");
    for (tool, link, target) in triplet_tool_link_plan(install_root) {
        symlink(&target, &link).map_err(|error| {
            StagexBinutilsError::Materialization(format!("creating installed triplet link for {tool}: {error}"))
        })?;
        let observed = fs::read_link(&link).map_err(|error| {
            StagexBinutilsError::Materialization(format!("reading installed triplet link for {tool}: {error}"))
        })?;
        if observed != target {
            return Err(StagexBinutilsError::Materialization(format!(
                "installed triplet link for {tool} targets {}, expected {}",
                observed.display(),
                target.display()
            )));
        }
    }
    assert!(bin.is_dir());
    assert!(bin.join(format!("{TARGET}-ld")).is_symlink());
    Ok(())
}

fn triplet_tool_link_plan(install_root: &Path) -> Vec<(&'static str, PathBuf, PathBuf)> {
    let bin = install_root.join("bin");
    let plan = BINUTILS_REQUIRED_TOOLS
        .iter()
        .map(|(tool, _)| {
            (
                *tool,
                bin.join(format!("{TARGET}-{tool}")),
                PathBuf::from(BINUTILS_INSTALL_PREFIX).join("bin").join(tool),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(plan.len(), BINUTILS_REQUIRED_TOOL_COUNT);
    assert!(plan.iter().all(|(_, link, target)| link.is_absolute() && target.is_absolute()));
    plan
}

fn validate_installed_tool_facts(
    tool: &str,
    bytes: &[u8],
    regular: bool,
    mode: u32,
    predecessor: &[u8],
    expected_digest: &str,
) -> Result<(), StagexBinutilsError> {
    validate_installed_tool_facts_with_mode(
        tool,
        bytes,
        regular,
        mode,
        predecessor,
        expected_digest,
        IdentityValidationMode::Enforce,
    )
}

fn validate_installed_tool_facts_with_mode(
    tool: &str,
    bytes: &[u8],
    regular: bool,
    mode: u32,
    predecessor: &[u8],
    expected_digest: &str,
    identity_validation_mode: IdentityValidationMode,
) -> Result<(), StagexBinutilsError> {
    if predecessor.is_empty() {
        return Err(StagexBinutilsError::Materialization("installed tool predecessor identity is empty".to_string()));
    }
    let is_elf = bytes.len() >= ELF_MAGIC.len() && &bytes[..ELF_MAGIC.len()] == ELF_MAGIC;
    if !regular || !is_elf {
        return Err(StagexBinutilsError::Materialization(format!("installed {tool} is not a regular ELF file")));
    }
    let delegates = bytes.windows(predecessor.len()).any(|window| window == predecessor);
    if mode & EXECUTABLE_MODE_BITS == 0 || delegates {
        return Err(StagexBinutilsError::Materialization(format!(
            "installed {tool} is not executable or delegates to predecessor TinyCC"
        )));
    }
    let observed_digest = blake3::hash(bytes).to_hex().to_string();
    if identity_validation_mode == IdentityValidationMode::Enforce && observed_digest != expected_digest {
        return Err(StagexBinutilsError::Materialization(format!(
            "installed {tool} digest mismatch: expected {expected_digest}, observed {observed_digest}"
        )));
    }
    assert!(regular);
    assert_eq!(observed_digest.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(())
}

fn run_installed_tool_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
) -> Result<(), StagexBinutilsError> {
    let smoke_root = request.scratch_dir.join("binutils-runtime-smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating binutils smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("empty.stdin"), b"")
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("positive.s"), BINUTILS_POSITIVE_ASSEMBLY)
        .map_err(StagexBinutilsError::from_runtime)?;
    run_link_and_execution_smoke(request, install_root, &smoke_root)?;
    run_archive_and_inspection_smokes(request, install_root, &smoke_root)?;
    run_negative_tool_smokes(request, install_root, &smoke_root)?;
    assert!(smoke_root.join("positive").is_file());
    assert!(smoke_root.join("libpositive.a").is_file());
    Ok(())
}

fn run_link_and_execution_smoke(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
    smoke_root: &Path,
) -> Result<(), StagexBinutilsError> {
    let assembler = install_root.join("bin/as");
    let linker = install_root.join("bin/ld");
    let assemble_status = run_binutils_smoke_process(
        request,
        smoke_root,
        &assembler,
        &["-o", "positive.o", "positive.s"],
        "assemble-positive",
    )?;
    require_binutils_smoke_status("positive assembly", assemble_status, 0)?;
    let link_status =
        run_binutils_smoke_process(request, smoke_root, &linker, &["-o", "positive", "positive.o"], "link-positive")?;
    require_binutils_smoke_status("positive link", link_status, 0)?;
    fs::set_permissions(smoke_root.join("positive"), fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting smoke executable mode: {error}")))?;
    validate_file_digest(
        &smoke_root.join("positive"),
        BINUTILS_POSITIVE_SMOKE_BLAKE3,
        "binutils positive smoke executable",
    )?;
    let execute_status =
        run_binutils_smoke_process(request, smoke_root, &smoke_root.join("positive"), &[], "execute-positive")?;
    require_binutils_smoke_status("positive execution", execute_status, BINUTILS_SMOKE_EXIT_STATUS)?;
    assert!(smoke_root.join("positive.o").is_file());
    assert!(smoke_root.join("positive").is_file());
    Ok(())
}

fn run_archive_and_inspection_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
    smoke_root: &Path,
) -> Result<(), StagexBinutilsError> {
    let cases: [(&str, &[&str], &str, Option<&str>); BINUTILS_INSPECTION_SMOKE_COUNT] = [
        ("ar", &["rc", "libpositive.a", "positive.o"], "archive-positive", None),
        ("ranlib", &["libpositive.a"], "ranlib-positive", None),
        ("ar", &["t", "libpositive.a"], "archive-list", Some("positive.o")),
        ("nm", &["positive.o"], "nm-positive", Some("_start")),
        ("objcopy", &["positive.o", "positive-copy.o"], "objcopy-positive", None),
        ("objdump", &["-d", "positive-copy.o"], "objdump-positive", Some("syscall")),
        ("readelf", &["-h", "positive.o"], "readelf-positive", Some("ELF64")),
    ];
    for (tool, arguments, label, marker) in cases {
        let status =
            run_binutils_smoke_process(request, smoke_root, &install_root.join("bin").join(tool), arguments, label)?;
        require_binutils_smoke_status(label, status, 0)?;
        if let Some(marker) = marker {
            let stdout = fs::read_to_string(request.scratch_dir.join(format!("{label}.stdout.txt")))
                .map_err(|error| StagexBinutilsError::Materialization(format!("reading {label} stdout: {error}")))?;
            if !stdout.contains(marker) {
                return Err(StagexBinutilsError::Materialization(format!("{label} lacks marker {marker:?}")));
            }
        }
    }
    assert!(smoke_root.join("libpositive.a").is_file());
    assert!(smoke_root.join("positive-copy.o").is_file());
    Ok(())
}

fn run_negative_tool_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    install_root: &Path,
    smoke_root: &Path,
) -> Result<(), StagexBinutilsError> {
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("rejected.s"), BINUTILS_INVALID_ASSEMBLY)
        .map_err(StagexBinutilsError::from_runtime)?;
    let assembler = install_root.join("bin/as");
    let assemble_status = run_binutils_smoke_process(
        request,
        smoke_root,
        &assembler,
        &["-o", "rejected.o", "rejected.s"],
        "assemble-negative",
    )?;
    require_binutils_smoke_rejection("malformed assembly", assemble_status, &smoke_root.join("rejected.o"))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("rejected.o"), BINUTILS_INVALID_OBJECT)
        .map_err(StagexBinutilsError::from_runtime)?;
    let link_status = run_binutils_smoke_process(
        request,
        smoke_root,
        &install_root.join("bin/ld"),
        &["-o", "rejected-bin", "rejected.o"],
        "link-negative",
    )?;
    require_binutils_smoke_rejection("malformed object", link_status, &smoke_root.join("rejected-bin"))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("rejected.a"), BINUTILS_INVALID_ARCHIVE)
        .map_err(StagexBinutilsError::from_runtime)?;
    let archive_status = run_binutils_smoke_process(
        request,
        smoke_root,
        &install_root.join("bin/ar"),
        &["t", "rejected.a"],
        "archive-negative",
    )?;
    if archive_status == 0 {
        return Err(StagexBinutilsError::Materialization("archive tool accepted malformed input".to_string()));
    }
    assert_ne!(assemble_status, 0);
    assert_ne!(archive_status, 0);
    Ok(())
}

fn run_binutils_smoke_process(
    request: &BinutilsConfigureProbeRequest<'_>,
    smoke_root: &Path,
    executable: &Path,
    arguments: &[&str],
    label: &str,
) -> Result<i32, StagexBinutilsError> {
    let stdout = request.scratch_dir.join(format!("{label}.stdout.txt"));
    let stderr = request.scratch_dir.join(format!("{label}.stderr.txt"));
    let environment = std::collections::BTreeMap::new();
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        executable,
        arguments,
        smoke_root,
        &environment,
        &smoke_root.join("empty.stdin"),
        BINUTILS_SMOKE_OUTPUT_BYTES_MAX,
        &stdout,
        BINUTILS_SMOKE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(status)
}

fn require_binutils_smoke_status(label: &str, observed: i32, expected: i32) -> Result<(), StagexBinutilsError> {
    if observed != expected {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} status mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed, expected);
    assert!(!label.is_empty());
    Ok(())
}

fn require_binutils_smoke_rejection(
    label: &str,
    status: i32,
    forbidden_output: &Path,
) -> Result<(), StagexBinutilsError> {
    if status == 0 || forbidden_output.exists() {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} did not fail closed: status {status}, output {}",
            forbidden_output.display()
        )));
    }
    assert_ne!(status, 0);
    assert!(!forbidden_output.exists());
    Ok(())
}

fn validate_generated_file(path: &Path, marker: &str, label: &str) -> Result<(), StagexBinutilsError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, label)
        .map_err(StagexBinutilsError::from_runtime)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| StagexBinutilsError::Materialization(format!("{label} is not UTF-8: {error}")))?;
    if !text.contains(marker) {
        return Err(StagexBinutilsError::Materialization(format!("{label} lacks its generated marker")));
    }
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn restore_authenticated_bfd_configure(context: &ConfigureProbeContext) -> Result<(), StagexBinutilsError> {
    let original = fs::read_to_string(&context.bfd_configure_authenticated)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading saved BFD configure: {error}")))?;
    let relocated = relocate_configure_file_text("bfd", &original)?;
    let configure = context.source.join("bfd/configure");
    fs::write(&configure, relocated.as_bytes())
        .map_err(|error| StagexBinutilsError::Materialization(format!("restoring BFD configure: {error}")))?;
    fs::set_permissions(&configure, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("restoring BFD configure mode: {error}")))?;
    assert_eq!(relocated.matches(BFD_BOOTSTRAP_CONFIG_REFERENCE).count(), BFD_BOOTSTRAP_CONFIG_REFERENCE_COUNT);
    assert!(!relocated.contains(AMBIENT_FILE_PATH));
    Ok(())
}

fn validate_second_bfd_header(path: &Path) -> Result<(), StagexBinutilsError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, "second BFD header")
        .map_err(StagexBinutilsError::from_runtime)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| StagexBinutilsError::Materialization(format!("second BFD header is not UTF-8: {error}")))?;
    if text.contains(BFD_SECOND_CONFIG_UNRESOLVED_MARKER) || !text.contains(BFD_HEADER_MARKER) {
        return Err(StagexBinutilsError::Materialization(
            "second BFD header did not resolve configured types".to_string(),
        ));
    }
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn run_configure_class(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    configure_class: &str,
) -> Result<ConfigureProbeOutcome, StagexBinutilsError> {
    let output_stem = if configure_class == "intl" {
        "configure"
    } else {
        configure_class
    };
    run_configure_class_named(request, context, configure_class, output_stem)
}

fn run_configure_class_named(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    configure_class: &str,
    output_stem: &str,
) -> Result<ConfigureProbeOutcome, StagexBinutilsError> {
    let stdout_path = request.scratch_dir.join(format!("{output_stem}.stdout.txt"));
    let stderr_path = request.scratch_dir.join(format!("{output_stem}.stderr.txt"));
    let environment = configure_environment(request, context, configure_class)?;
    let configure_dir = context.source.join(configure_class);
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &configure_arguments(),
        &configure_dir,
        &environment,
        &context.stdin_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stdout_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(ConfigureProbeOutcome {
        configure_class: configure_class.to_string(),
        exit_code,
        stdout_path,
        stderr_path,
    })
}

fn prepare_recipe_source(scratch_dir: &Path, source: &Path) -> Result<PathBuf, StagexBinutilsError> {
    repair_opcodes_dependencies(source)?;
    make_opcodes_generator_exec_paths_absolute(source)?;
    make_component_generator_exec_paths_absolute(source)?;
    disable_legacy_absolute_host_probes(source)?;
    make_generated_exec_paths_absolute(source)?;
    let authenticated_bfd_configure = scratch_dir.join("bfd-configure.authenticated");
    let bfd_configure = source.join("bfd/configure");
    let bfd_bytes = fs::read(&bfd_configure).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading authenticated BFD configure: {error}"))
    })?;
    crate::stagex_mes_lib::write_create_new(&authenticated_bfd_configure, &bfd_bytes)
        .map_err(StagexBinutilsError::from_runtime)?;
    repair_bfd_bootstrap_config(&bfd_configure, &bfd_bytes)?;
    make_bfd_chew_exec_paths_absolute(source)?;
    remove_release_generated_sources(source)?;
    assert!(authenticated_bfd_configure.is_file());
    assert!(GENERATED_SOURCE_REMOVALS.iter().all(|path| !source.join(path).exists()));
    Ok(authenticated_bfd_configure)
}

fn disable_legacy_absolute_host_probes(source: &Path) -> Result<(), StagexBinutilsError> {
    for configure_class in CONFIGURE_CLASSES {
        let configure = source.join(configure_class).join("configure");
        let original = fs::read_to_string(&configure).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "reading {configure_class} configure for legacy host probe removal: {error}"
            ))
        })?;
        let rewritten = disable_legacy_absolute_host_probe_text(configure_class, &original)?;
        fs::write(&configure, rewritten.as_bytes()).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "writing {configure_class} configure without legacy host probes: {error}"
            ))
        })?;
        assert!(!rewritten.contains(LEGACY_ABSOLUTE_HOST_PROBE_BLOCK));
        assert!(rewritten.contains(DISABLED_ABSOLUTE_HOST_PROBE_BLOCK));
    }
    assert_eq!(CONFIGURE_CLASSES.len(), CONFIGURE_CLASS_COUNT);
    assert!(CONFIGURE_CLASSES.iter().all(|class| source.join(class).join("configure").is_file()));
    Ok(())
}

fn disable_legacy_absolute_host_probe_text(
    configure_class: &str,
    original: &str,
) -> Result<String, StagexBinutilsError> {
    let label = format!("{configure_class} legacy absolute host probes");
    let rewritten =
        replace_exact_text(&label, original, LEGACY_ABSOLUTE_HOST_PROBE_BLOCK, DISABLED_ABSOLUTE_HOST_PROBE_BLOCK, 1)?;
    assert!(!rewritten.contains(LEGACY_ABSOLUTE_HOST_PROBE_BLOCK));
    assert_eq!(rewritten.matches(DISABLED_ABSOLUTE_HOST_PROBE_BLOCK).count(), 1);
    Ok(rewritten)
}

fn make_generated_exec_paths_absolute(source: &Path) -> Result<(), StagexBinutilsError> {
    for configure_class in CONFIGURE_CLASSES {
        let configure = source.join(configure_class).join("configure");
        let original = fs::read_to_string(&configure).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "reading {configure_class} configure for generated exec path repair: {error}"
            ))
        })?;
        let rewritten = make_configure_exec_paths_absolute(configure_class, &original)?;
        fs::write(&configure, rewritten.as_bytes()).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "writing {configure_class} configure with absolute generated exec paths: {error}"
            ))
        })?;
        assert!(!rewritten.contains(CONFIGURE_DYNAMIC_EXEC_OLD));
        assert!(!rewritten.contains(CONFIGURE_CONFTEST_EXEC_OLD));
    }
    assert_eq!(CONFIGURE_CLASSES.len(), CONFIGURE_CLASS_COUNT);
    assert!(CONFIGURE_CLASSES.iter().all(|class| source.join(class).join("configure").is_file()));
    Ok(())
}

fn make_configure_exec_paths_absolute(configure_class: &str, original: &str) -> Result<String, StagexBinutilsError> {
    let dynamic_label = format!("{configure_class} dynamic generated executable");
    let dynamic =
        replace_exact_text(&dynamic_label, original, CONFIGURE_DYNAMIC_EXEC_OLD, CONFIGURE_DYNAMIC_EXEC_NEW, 1)?;
    let conftest_label = format!("{configure_class} fixed conftest executable");
    let conftest =
        replace_exact_text(&conftest_label, &dynamic, CONFIGURE_CONFTEST_EXEC_OLD, CONFIGURE_CONFTEST_EXEC_NEW, 1)?;
    if !FILE_RELOCATION_CLASSES.contains(&configure_class) {
        assert!(!conftest.contains(CONFIGURE_DYNAMIC_EXEC_OLD));
        assert!(!conftest.contains(CONFIGURE_CONFTEST_EXEC_OLD));
        return Ok(conftest);
    }
    let libtool_label = format!("{configure_class} libtool conftest executable");
    replace_exact_text(
        &libtool_label,
        &conftest,
        CONFIGURE_LIBTOOL_EXEC_OLD,
        CONFIGURE_LIBTOOL_EXEC_NEW,
        CONFIGURE_LIBTOOL_EXEC_COUNT_PER_CLASS,
    )
}

fn make_bfd_chew_exec_paths_absolute(source: &Path) -> Result<(), StagexBinutilsError> {
    let makefile = source.join("bfd/doc/Makefile.in");
    let original = fs::read_to_string(&makefile)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading BFD doc Makefile: {error}")))?;
    let rewritten = replace_exact_text(
        "BFD chew executable paths",
        &original,
        BFD_CHEW_RELATIVE_EXEC,
        BFD_CHEW_ABSOLUTE_EXEC,
        BFD_CHEW_EXEC_OCCURRENCE_COUNT,
    )?;
    fs::write(&makefile, rewritten.as_bytes())
        .map_err(|error| StagexBinutilsError::Materialization(format!("writing BFD doc Makefile: {error}")))?;
    assert!(!rewritten.contains(BFD_CHEW_RELATIVE_EXEC));
    assert_eq!(rewritten.matches(BFD_CHEW_ABSOLUTE_EXEC).count(), BFD_CHEW_EXEC_OCCURRENCE_COUNT);
    Ok(())
}

fn make_opcodes_generator_exec_paths_absolute(source: &Path) -> Result<(), StagexBinutilsError> {
    for relative in ["opcodes/Makefile.am", "opcodes/Makefile.in"] {
        let path = source.join(relative);
        let original = fs::read_to_string(&path)
            .map_err(|error| StagexBinutilsError::Materialization(format!("reading {relative}: {error}")))?;
        let rewritten = make_opcodes_generator_exec_path_absolute(relative, &original)?;
        fs::write(&path, rewritten.as_bytes())
            .map_err(|error| StagexBinutilsError::Materialization(format!("writing {relative}: {error}")))?;
        assert!(!rewritten.contains(OPCODES_I386_GEN_RELATIVE_EXEC));
        assert_eq!(rewritten.matches(OPCODES_I386_GEN_ABSOLUTE_EXEC).count(), OPCODES_I386_GEN_EXEC_OCCURRENCE_COUNT);
    }
    assert!(source.join("opcodes/Makefile.in").is_file());
    assert!(source.join("opcodes/Makefile.am").is_file());
    Ok(())
}

fn make_opcodes_generator_exec_path_absolute(label: &str, original: &str) -> Result<String, StagexBinutilsError> {
    let rewritten = replace_exact_text(
        label,
        original,
        OPCODES_I386_GEN_RELATIVE_EXEC,
        OPCODES_I386_GEN_ABSOLUTE_EXEC,
        OPCODES_I386_GEN_EXEC_OCCURRENCE_COUNT,
    )?;
    assert!(!rewritten.contains(OPCODES_I386_GEN_RELATIVE_EXEC));
    assert_eq!(rewritten.matches(OPCODES_I386_GEN_ABSOLUTE_EXEC).count(), OPCODES_I386_GEN_EXEC_OCCURRENCE_COUNT);
    Ok(rewritten)
}

fn make_component_generator_exec_paths_absolute(source: &Path) -> Result<(), StagexBinutilsError> {
    let rewrites = [
        (
            "binutils/Makefile.am",
            BINUTILS_SYSINFO_RELATIVE_EXEC,
            BINUTILS_SYSINFO_ABSOLUTE_EXEC,
            BINUTILS_SYSINFO_EXEC_OCCURRENCE_COUNT,
        ),
        (
            "binutils/Makefile.in",
            BINUTILS_SYSINFO_RELATIVE_EXEC,
            BINUTILS_SYSINFO_ABSOLUTE_EXEC,
            BINUTILS_SYSINFO_EXEC_OCCURRENCE_COUNT,
        ),
        (
            "bfd/Makefile.am",
            BFD_GEN_AOUT_RELATIVE_EXEC,
            BFD_GEN_AOUT_ABSOLUTE_EXEC,
            BFD_GEN_AOUT_EXEC_OCCURRENCE_COUNT,
        ),
        (
            "bfd/Makefile.in",
            BFD_GEN_AOUT_RELATIVE_EXEC,
            BFD_GEN_AOUT_ABSOLUTE_EXEC,
            BFD_GEN_AOUT_EXEC_OCCURRENCE_COUNT,
        ),
    ];
    let mut prepared = Vec::with_capacity(COMPONENT_GENERATOR_REWRITE_COUNT);
    for (relative, old, new, count) in rewrites {
        let path = source.join(relative);
        let original = fs::read_to_string(&path)
            .map_err(|error| StagexBinutilsError::Materialization(format!("reading {relative}: {error}")))?;
        let rewritten = replace_exact_text(relative, &original, old, new, count)?;
        prepared.push((path, relative, old, new, count, rewritten));
    }
    assert_eq!(prepared.len(), COMPONENT_GENERATOR_REWRITE_COUNT);
    assert!(prepared.iter().all(|(_, _, old, _, _, rewritten)| !rewritten.contains(old)));
    for (path, relative, old, new, count, rewritten) in prepared {
        fs::write(&path, rewritten.as_bytes())
            .map_err(|error| StagexBinutilsError::Materialization(format!("writing {relative}: {error}")))?;
        assert!(!rewritten.contains(old));
        assert_eq!(rewritten.matches(new).count(), count);
    }
    Ok(())
}

fn repair_opcodes_dependencies(source: &Path) -> Result<(), StagexBinutilsError> {
    for relative in ["opcodes/Makefile.am", "opcodes/Makefile.in"] {
        let path = source.join(relative);
        let original = fs::read_to_string(&path)
            .map_err(|error| StagexBinutilsError::Materialization(format!("reading {relative}: {error}")))?;
        let repaired = replace_exact_text(relative, &original, OPCODES_DEPENDENCY_OLD, OPCODES_DEPENDENCY_NEW, 1)?;
        fs::write(&path, repaired.as_bytes())
            .map_err(|error| StagexBinutilsError::Materialization(format!("writing {relative}: {error}")))?;
        assert!(!repaired.contains(OPCODES_DEPENDENCY_OLD));
    }
    assert!(source.join("opcodes/Makefile.in").is_file());
    assert!(source.join("opcodes/Makefile.am").is_file());
    Ok(())
}

fn repair_bfd_bootstrap_config(path: &Path, original: &[u8]) -> Result<(), StagexBinutilsError> {
    let text = std::str::from_utf8(original)
        .map_err(|error| StagexBinutilsError::Materialization(format!("BFD configure is not UTF-8: {error}")))?;
    let repaired = replace_exact_text(
        "BFD bootstrap configure",
        text,
        BFD_BOOTSTRAP_CONFIG_REFERENCE,
        "",
        BFD_BOOTSTRAP_CONFIG_REFERENCE_COUNT,
    )?;
    fs::write(path, repaired.as_bytes())
        .map_err(|error| StagexBinutilsError::Materialization(format!("writing BFD bootstrap configure: {error}")))?;
    assert!(!repaired.contains(BFD_BOOTSTRAP_CONFIG_REFERENCE));
    assert!(path.is_file());
    Ok(())
}

fn replace_exact_text(
    label: &str,
    original: &str,
    old: &str,
    new: &str,
    expected_count: usize,
) -> Result<String, StagexBinutilsError> {
    let occurrence_count = original.matches(old).count();
    if occurrence_count != expected_count {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} replacement occurrence mismatch: expected {expected_count}, observed {occurrence_count}"
        )));
    }
    let replaced = original.replace(old, new);
    assert_eq!(replaced.matches(old).count(), 0);
    assert_ne!(original, replaced);
    Ok(replaced)
}

fn remove_release_generated_sources(source: &Path) -> Result<(), StagexBinutilsError> {
    if GENERATED_SOURCE_REMOVALS.len() != GENERATED_SOURCE_REMOVAL_COUNT {
        return Err(StagexBinutilsError::Materialization("generated source removal inventory drifted".to_string()));
    }
    for relative in GENERATED_SOURCE_REMOVALS {
        let path = source.join(relative);
        fs::remove_file(&path).map_err(|error| {
            StagexBinutilsError::Materialization(format!("removing authenticated generated source {relative}: {error}"))
        })?;
        if path.exists() {
            return Err(StagexBinutilsError::Materialization(format!(
                "generated source remained after removal: {relative}"
            )));
        }
    }
    assert_eq!(GENERATED_SOURCE_REMOVALS.len(), GENERATED_SOURCE_REMOVAL_COUNT);
    assert!(GENERATED_SOURCE_REMOVALS.iter().all(|path| !source.join(path).exists()));
    Ok(())
}

fn relocate_configure_file_utility(source: &Path) -> Result<(), StagexBinutilsError> {
    let mut replacement_count = 0usize;
    for configure_class in FILE_RELOCATION_CLASSES {
        let configure = source.join(configure_class).join("configure");
        let original = fs::read_to_string(&configure).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "reading {configure_class} configure for file relocation: {error}"
            ))
        })?;
        let class_count = original.matches(AMBIENT_FILE_PATH).count();
        let relocated = relocate_configure_file_text(configure_class, &original)?;
        fs::write(&configure, relocated.as_bytes()).map_err(|error| {
            StagexBinutilsError::Materialization(format!(
                "writing {configure_class} configure file relocation: {error}"
            ))
        })?;
        replacement_count = replacement_count.checked_add(class_count).ok_or_else(|| {
            StagexBinutilsError::Materialization("configure file relocation count overflow".to_string())
        })?;
        assert!(!relocated.contains(AMBIENT_FILE_PATH));
    }
    if replacement_count != FILE_RELOCATION_OCCURRENCE_COUNT {
        return Err(StagexBinutilsError::Materialization(format!(
            "ambient file relocation count mismatch: expected {FILE_RELOCATION_OCCURRENCE_COUNT}, observed {replacement_count}"
        )));
    }
    assert_eq!(replacement_count, FILE_RELOCATION_OCCURRENCE_COUNT);
    assert!(FILE_RELOCATION_CLASSES.iter().all(|class| source.join(class).join("configure").is_file()));
    Ok(())
}

fn relocate_configure_file_text(configure_class: &str, original: &str) -> Result<String, StagexBinutilsError> {
    let class_count = original.matches(AMBIENT_FILE_PATH).count();
    if class_count != FILE_RELOCATION_OCCURRENCES_PER_CLASS {
        return Err(StagexBinutilsError::Materialization(format!(
            "{configure_class} ambient file occurrence mismatch: expected {FILE_RELOCATION_OCCURRENCES_PER_CLASS}, observed {class_count}"
        )));
    }
    let relocated = original.replace(AMBIENT_FILE_PATH, DECLARED_FILE_PATH);
    assert_eq!(relocated.matches(DECLARED_FILE_PATH).count(), FILE_RELOCATION_OCCURRENCES_PER_CLASS);
    assert!(!relocated.contains(AMBIENT_FILE_PATH));
    Ok(relocated)
}

fn validate_configure_classes(configure_classes: &[&str]) -> Result<(), StagexBinutilsError> {
    if configure_classes.is_empty() || configure_classes.len() > CONFIGURE_CLASS_COUNT {
        return Err(StagexBinutilsError::Materialization("configure class count is outside bounds".to_string()));
    }
    let mut unique = std::collections::BTreeSet::new();
    for configure_class in configure_classes {
        if !CONFIGURE_CLASSES.contains(configure_class) || !unique.insert(*configure_class) {
            return Err(StagexBinutilsError::Materialization(format!(
                "unknown or duplicate configure class: {configure_class}"
            )));
        }
    }
    assert_eq!(unique.len(), configure_classes.len());
    assert!(!configure_classes.is_empty());
    Ok(())
}

impl StagexBinutilsError {
    fn from_runtime(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Materialization(format!("bounded process: {error}"))
    }
}

fn validate_configure_probe_request(request: &BinutilsConfigureProbeRequest<'_>) -> Result<(), StagexBinutilsError> {
    if request.scratch_dir.exists() || !request.scratch_dir.is_absolute() {
        return Err(StagexBinutilsError::Materialization(format!(
            "configure scratch must be an absent absolute path: {}",
            request.scratch_dir.display()
        )));
    }
    validate_materialized_source(request.source_root)?;
    for (label, path) in probe_required_executables(request) {
        if !path.is_absolute() || !path.is_file() {
            return Err(StagexBinutilsError::Materialization(format!("missing absolute {label}: {}", path.display())));
        }
    }
    if !request.musl_root.join("include/stdio.h").is_file() || !request.musl_root.join("lib/libc.a").is_file() {
        return Err(StagexBinutilsError::Materialization("native musl input is incomplete".to_string()));
    }
    assert!(!request.scratch_dir.exists());
    assert!(request.bash.is_file());
    Ok(())
}

fn probe_required_executables<'a>(request: &'a BinutilsConfigureProbeRequest<'a>) -> [(&'static str, &'a Path); 11] {
    [
        ("full Bash", request.bash),
        ("TinyCC", request.tcc),
        ("sed", request.sed),
        ("grep", request.grep),
        ("diff", request.diff),
        ("cmp", request.cmp),
        ("Gawk", request.gawk),
        ("M4", request.m4),
        ("Bison", request.bison),
        ("Flex", request.flex),
        ("GNU Make", request.make),
    ]
}

fn populate_probe_tool_namespace(
    request: &BinutilsConfigureProbeRequest<'_>,
    tools: &Path,
    sed_bridge_launcher: &Path,
    configure_utility: &Path,
) -> Result<(), StagexBinutilsError> {
    for name in COREUTILS_TOOL_NAMES {
        add_tool_binding(tools, name, &request.coreutils_bin.join(name))?;
    }
    let bindings = [
        ("sh", request.bash),
        ("bash", request.bash),
        ("cc", request.tcc),
        ("sed", sed_bridge_launcher),
        ("sleep", configure_utility),
        ("file", configure_utility),
        ("emit", configure_utility),
        ("grep", request.grep),
        ("egrep", request.grep),
        ("fgrep", request.grep),
        ("diff", request.diff),
        ("cmp", request.cmp),
        ("awk", request.gawk),
        ("gawk", request.gawk),
        ("m4", request.m4),
        ("bison", request.bison),
        ("flex", request.flex),
        ("make", request.make),
    ];
    for (name, path) in bindings {
        add_tool_binding(tools, name, path)?;
    }
    assert_eq!(COREUTILS_TOOL_NAMES.len(), COREUTILS_TOOL_COUNT);
    assert!(tools.join("sh").is_symlink());
    assert!(tools.join("file").is_symlink());
    Ok(())
}

fn add_tool_binding(tools: &Path, name: &str, target: &Path) -> Result<(), StagexBinutilsError> {
    if !target.is_absolute() || !target.is_file() {
        return Err(StagexBinutilsError::Materialization(format!(
            "tool binding target is not an absolute file: {name}: {}",
            target.display()
        )));
    }
    let link = tools.join(name);
    symlink(target, &link)
        .map_err(|error| StagexBinutilsError::Materialization(format!("binding configure tool {name}: {error}")))?;
    assert!(link.is_symlink());
    assert!(target.is_file());
    Ok(())
}

fn configure_arguments() -> Vec<String> {
    let arguments = [
        "./configure",
        "--disable-nls",
        "--enable-deterministic-archives",
        "--enable-64-bit-bfd",
        "--disable-shared",
        "--disable-plugins",
        "--disable-werror",
        "--build=x86_64-unknown-linux-gnu",
        "--host=x86_64-unknown-linux-gnu",
        "--target=x86_64-unknown-linux-gnu",
        "--program-prefix=",
        BINUTILS_INSTALL_PREFIX_ARGUMENT,
        BINUTILS_INSTALL_LIBDIR_ARGUMENT,
        "--with-sysroot=",
        "--srcdir=.",
        "--enable-compressed-debug-sections=all",
        "lt_cv_sys_max_cmd_len=32768",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();
    assert!(arguments.len() > 1);
    assert_eq!(arguments[0], "./configure");
    arguments
}

fn prepare_tcc_wrapper(request: &BinutilsConfigureProbeRequest<'_>) -> Result<PathBuf, StagexBinutilsError> {
    let runtime_source = request.scratch_dir.join("binutils-runtime.c");
    let runtime_object = request.scratch_dir.join("binutils-runtime.o");
    let runtime_assembly = request.scratch_dir.join("binutils-runtime.s");
    let runtime_assembly_object = request.scratch_dir.join("binutils-runtime-asm.o");
    let runtime = extract_recipe_block(BINUTILS_RUNTIME_START, BINUTILS_RUNTIME_END, "binutils runtime")?;
    crate::stagex_mes_lib::write_create_new(&runtime_source, runtime.as_bytes())
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(
        &runtime_assembly,
        b".global sigsetjmp\n.type sigsetjmp,@function\nsigsetjmp:\n  jmp setjmp\n",
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    compile_runtime_object(request, &runtime_source, &runtime_object, "runtime")?;
    compile_runtime_object(request, &runtime_assembly, &runtime_assembly_object, "runtime-assembly")?;
    let canonicalizer = prepare_elf_symbol_canonicalizer(request, &runtime_object, &runtime_assembly_object)?;
    let wrapper = request.scratch_dir.join("tcc-bounded.sh");
    let script = tcc_wrapper_script(request, &runtime_object, &runtime_assembly_object, &canonicalizer)?;
    crate::stagex_mes_lib::write_create_new(&wrapper, script.as_bytes()).map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting compiler wrapper mode: {error}")))?;
    assert!(runtime_object.is_file());
    assert!(canonicalizer.is_file());
    assert!(wrapper.is_file());
    Ok(wrapper)
}

fn prepare_elf_symbol_canonicalizer(
    request: &BinutilsConfigureProbeRequest<'_>,
    runtime_object: &Path,
    runtime_assembly_object: &Path,
) -> Result<PathBuf, StagexBinutilsError> {
    require_sed_bridge_source_digest(
        "ELF symbol canonicalizer",
        ELF_SYMBOL_CANONICALIZER_SOURCE,
        ELF_SYMBOL_CANONICALIZER_SOURCE_BLAKE3,
    )?;
    let source = request.scratch_dir.join(ELF_SYMBOL_CANONICALIZER_SOURCE_NAME);
    let object = request.scratch_dir.join("stagex-elf-local-symbol-canonicalizer.o");
    let output = request.scratch_dir.join(ELF_SYMBOL_CANONICALIZER_OUTPUT_NAME);
    crate::stagex_mes_lib::write_create_new(&source, ELF_SYMBOL_CANONICALIZER_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    compile_elf_symbol_canonicalizer(request, &object, &output, runtime_object, runtime_assembly_object)?;
    validate_file_digest(&output, ELF_SYMBOL_CANONICALIZER_BLAKE3, "ELF symbol canonicalizer")?;
    run_elf_symbol_canonicalizer_smokes(request, &output)?;
    assert!(source.is_file());
    assert!(output.is_file());
    Ok(output)
}

fn compile_elf_symbol_canonicalizer(
    request: &BinutilsConfigureProbeRequest<'_>,
    object: &Path,
    output: &Path,
    runtime_object: &Path,
    runtime_assembly_object: &Path,
) -> Result<(), StagexBinutilsError> {
    let include = format!("-I{}", shell_path(&request.musl_root.join("include"), "native musl include")?);
    let compile_arguments = [
        "-c".to_string(),
        include,
        ELF_SYMBOL_CANONICALIZER_SOURCE_NAME.to_string(),
        "-o".to_string(),
        shell_path(object, "ELF symbol canonicalizer object")?,
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.tcc,
        &compile_arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("elf-symbol-canonicalizer-compile.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(object, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting canonicalizer object mode: {error}")))?;
    link_elf_symbol_canonicalizer(request, object, output, runtime_object, runtime_assembly_object)?;
    assert_eq!(compile_arguments.len(), ELF_CANONICALIZER_COMPILE_ARGUMENT_COUNT);
    assert!(object.is_file());
    Ok(())
}

fn link_elf_symbol_canonicalizer(
    request: &BinutilsConfigureProbeRequest<'_>,
    object: &Path,
    output: &Path,
    runtime_object: &Path,
    runtime_assembly_object: &Path,
) -> Result<(), StagexBinutilsError> {
    let musl_lib = request.musl_root.join("lib");
    let link_arguments = [
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-s".to_string(),
        shell_path(&musl_lib.join("crt1.o"), "native musl crt1")?,
        shell_path(object, "ELF symbol canonicalizer object")?,
        shell_path(runtime_object, "runtime object")?,
        shell_path(runtime_assembly_object, "runtime assembly object")?,
        shell_path(&musl_lib.join("libc.a"), "native musl libc")?,
        "-o".to_string(),
        shell_path(output, "ELF symbol canonicalizer output")?,
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.tcc,
        &link_arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("elf-symbol-canonicalizer-link.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting canonicalizer mode: {error}")))?;
    assert_eq!(link_arguments.len(), ELF_CANONICALIZER_LINK_ARGUMENT_COUNT);
    assert!(output.is_file());
    Ok(())
}

fn run_elf_symbol_canonicalizer_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    canonicalizer: &Path,
) -> Result<(), StagexBinutilsError> {
    let source = request.scratch_dir.join("elf-canonicalizer-smoke.c");
    let first = request.scratch_dir.join("elf-canonicalizer-smoke-a.o");
    let second = request.scratch_dir.join("elf-canonicalizer-smoke-b.o");
    let malformed = request.scratch_dir.join("elf-canonicalizer-malformed.o");
    crate::stagex_mes_lib::write_create_new(
        &source,
        b"static const char value[] = \"anonymous-symbol\"; int value_at(unsigned i) { return value[i]; }\n",
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    compile_canonicalizer_smoke_object(request, &source, &first, "first")?;
    compile_canonicalizer_smoke_object(request, &source, &second, "second")?;
    run_canonicalizer_on_path(request, canonicalizer, &first, "first")?;
    run_canonicalizer_on_path(request, canonicalizer, &second, "second")?;
    let first_bytes = fs::read(&first)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading first canonical ELF: {error}")))?;
    let second_bytes = fs::read(&second)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading second canonical ELF: {error}")))?;
    if first_bytes != second_bytes {
        return Err(StagexBinutilsError::Materialization(
            "ELF symbol canonicalizer did not stabilize repeated compiler output".to_string(),
        ));
    }
    crate::stagex_mes_lib::write_create_new(&malformed, b"not-an-elf\n").map_err(StagexBinutilsError::from_runtime)?;
    require_canonicalizer_rejection(request, canonicalizer, &malformed, "malformed")?;
    assert!(!first_bytes.is_empty());
    assert_eq!(first_bytes, second_bytes);
    Ok(())
}

fn compile_canonicalizer_smoke_object(
    request: &BinutilsConfigureProbeRequest<'_>,
    source: &Path,
    output: &Path,
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let arguments = [
        "-c".to_string(),
        shell_path(source, "canonicalizer smoke source")?,
        "-o".to_string(),
        shell_path(output, "canonicalizer smoke object")?,
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.tcc,
        &arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join(format!("elf-canonicalizer-{label}-compile.stderr.txt")),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(REGULAR_FILE_MODE)).map_err(|error| {
        StagexBinutilsError::Materialization(format!("setting {label} canonicalizer smoke mode: {error}"))
    })?;
    assert_eq!(arguments.len(), ELF_CANONICALIZER_SMOKE_COMPILE_ARGUMENT_COUNT);
    assert!(source.is_file());
    assert!(output.is_file());
    Ok(())
}

fn run_canonicalizer_on_path(
    request: &BinutilsConfigureProbeRequest<'_>,
    canonicalizer: &Path,
    output: &Path,
    label: &str,
) -> Result<(), StagexBinutilsError> {
    crate::stagex_mes_lib::run_bounded_process(
        canonicalizer,
        &[shell_path(output, "canonicalizer smoke object")?],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join(format!("elf-canonicalizer-{label}.stderr.txt")),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(canonicalizer.is_file());
    assert!(output.is_file());
    Ok(())
}

fn require_canonicalizer_rejection(
    request: &BinutilsConfigureProbeRequest<'_>,
    canonicalizer: &Path,
    input: &Path,
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let result = crate::stagex_mes_lib::run_bounded_process(
        canonicalizer,
        &[shell_path(input, "rejected canonicalizer input")?],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join(format!("elf-canonicalizer-{label}-negative.stderr.txt")),
    );
    if result.is_ok() {
        return Err(StagexBinutilsError::Materialization(format!("ELF symbol canonicalizer accepted {label} input")));
    }
    assert!(canonicalizer.is_file());
    assert!(input.is_file());
    Ok(())
}

fn configure_preprocess_negative_cases() -> [ConfigurePreprocessNegativeCase; CONFIGURE_PREPROCESS_NEGATIVE_CASE_COUNT]
{
    let oversized_source_bytes =
        usize::try_from(CONFIGURE_PROBE_SOURCE_BYTES_MAX.checked_add(1).expect("probe bound overflow"))
            .expect("probe bound fits usize");
    let mut missing_authority = configure_preprocess_negative_case("missing-authority", "missing-authority-directory");
    missing_authority.omit_authority = true;
    let mut wrong_directory = configure_preprocess_negative_case("wrong-directory", "current-directory-mismatch");
    wrong_directory.mismatched_directory = true;
    let mut non_conftest = configure_preprocess_negative_case("non-conftest", "non-conftest-source");
    non_conftest.source_name = "probe.c";
    let mut unknown_class = configure_preprocess_negative_case("unknown-class", "unknown-configure-class");
    unknown_class.configure_class = "unknown";
    let mut oversized_source = configure_preprocess_negative_case("oversized-source", "source-too-large");
    oversized_source.source_bytes_override = Some(oversized_source_bytes);
    let mut exhausted_budget = configure_preprocess_negative_case("exhausted-budget", "invocation-budget-exhausted");
    exhausted_budget.count_override = Some(CONFIGURE_PROBE_INVOCATION_COUNT_MAX);
    [
        missing_authority,
        wrong_directory,
        non_conftest,
        unknown_class,
        oversized_source,
        exhausted_budget,
    ]
}

const fn configure_preprocess_negative_case(
    label: &'static str,
    expected_reason: &'static str,
) -> ConfigurePreprocessNegativeCase {
    ConfigurePreprocessNegativeCase {
        label,
        expected_reason,
        configure_class: "intl",
        source_name: "conftest.c",
        source_bytes_override: None,
        count_override: None,
        omit_authority: false,
        mismatched_directory: false,
    }
}

fn run_tcc_wrapper_negative_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    wrapper: &Path,
) -> Result<(), StagexBinutilsError> {
    for case in configure_preprocess_negative_cases() {
        run_tcc_wrapper_negative_smoke(request, wrapper, case)?;
    }
    assert!(wrapper.is_file());
    assert_eq!(configure_preprocess_negative_cases().len(), CONFIGURE_PREPROCESS_NEGATIVE_CASE_COUNT);
    Ok(())
}

fn run_tcc_wrapper_negative_smoke(
    request: &BinutilsConfigureProbeRequest<'_>,
    wrapper: &Path,
    case: ConfigurePreprocessNegativeCase,
) -> Result<(), StagexBinutilsError> {
    let root = request.scratch_dir.join(format!("preprocess-negative-{}", case.label));
    fs::create_dir(&root).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating {} preprocess smoke: {error}", case.label))
    })?;
    let source = case
        .source_bytes_override
        .map_or_else(|| b"int mantle_probe;\n".to_vec(), |source_bytes| vec![b'x'; source_bytes]);
    crate::stagex_mes_lib::write_create_new(&root.join(case.source_name), &source)
        .map_err(StagexBinutilsError::from_runtime)?;
    let audit = root.join("audit.tsv");
    let count = root.join("count.txt");
    if let Some(value) = case.count_override {
        crate::stagex_mes_lib::write_create_new(&count, format!("{value}\n").as_bytes())
            .map_err(StagexBinutilsError::from_runtime)?;
    }
    let environment = preprocess_negative_environment(&root, &audit, &count, case)?;
    let empty_stdin = root.join("empty.stdin");
    crate::stagex_mes_lib::write_create_new(&empty_stdin, b"").map_err(StagexBinutilsError::from_runtime)?;
    let stdout = root.join("stdout.txt");
    let stderr = root.join("stderr.txt");
    let arguments = vec![
        shell_path(wrapper, "compiler wrapper")?,
        "-E".to_string(),
        case.source_name.to_string(),
    ];
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &arguments,
        &root,
        &environment,
        &empty_stdin,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let diagnostic = fs::read_to_string(&stderr).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading {} preprocess smoke: {error}", case.label))
    })?;
    if status != CONFIGURE_PREPROCESS_REJECTION_STATUS || !diagnostic.contains(case.expected_reason) {
        return Err(StagexBinutilsError::Materialization(format!(
            "{} preprocess smoke mismatch: status {status}, diagnostic {diagnostic:?}",
            case.label
        )));
    }
    if audit.exists() {
        return Err(StagexBinutilsError::Materialization(format!("{} preprocess smoke wrote an audit", case.label)));
    }
    assert_eq!(status, CONFIGURE_PREPROCESS_REJECTION_STATUS);
    assert!(!diagnostic.is_empty());
    Ok(())
}

fn preprocess_negative_environment(
    root: &Path,
    audit: &Path,
    count: &Path,
    case: ConfigurePreprocessNegativeCase,
) -> Result<std::collections::BTreeMap<String, String>, StagexBinutilsError> {
    let mut environment = std::collections::BTreeMap::new();
    if case.omit_authority {
        assert!(environment.is_empty());
        return Ok(environment);
    }
    let declared_root = if case.mismatched_directory {
        root.join("declared-directory")
    } else {
        root.to_path_buf()
    };
    environment.insert(
        "MANTLE_BINUTILS_CONFIGURE_PROBE_DIR".to_string(),
        shell_path(&declared_root, "negative probe directory")?,
    );
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS".to_string(), case.configure_class.to_string());
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT".to_string(), shell_path(audit, "negative probe audit")?);
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT".to_string(), shell_path(count, "negative probe count")?);
    assert_eq!(environment.len(), 4);
    assert!(environment.contains_key("MANTLE_BINUTILS_CONFIGURE_PROBE_DIR"));
    Ok(environment)
}

fn prepare_archive_runner(
    request: &BinutilsConfigureProbeRequest<'_>,
) -> Result<ArchiveRunnerPaths, StagexBinutilsError> {
    require_sed_bridge_source_digest(
        "binutils ar runner",
        BINUTILS_AR_RUNNER_SOURCE,
        BINUTILS_AR_RUNNER_SOURCE_BLAKE3,
    )?;
    let script = request.scratch_dir.join(BINUTILS_AR_RUNNER_SOURCE_NAME);
    let scratch = request.scratch_dir.join("ar-scratch");
    fs::create_dir(&scratch)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating binutils ar scratch: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&script, BINUTILS_AR_RUNNER_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting binutils ar runner mode: {error}")))?;
    let paths = ArchiveRunnerPaths {
        script,
        scratch,
        audit: request.scratch_dir.join("ar-audit.tsv"),
        count: request.scratch_dir.join("ar-count.txt"),
    };
    run_archive_runner_smokes(request, &paths)?;
    assert!(paths.script.is_file());
    assert!(paths.scratch.is_dir());
    Ok(paths)
}

fn archive_runner_environment(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &ArchiveRunnerPaths,
) -> Result<std::collections::BTreeMap<String, String>, StagexBinutilsError> {
    let mut environment = std::collections::BTreeMap::new();
    for (name, tool) in [
        ("BASENAME", "basename"),
        ("CAT", "cat"),
        ("MV", "mv"),
        ("RM", "rm"),
        ("WC", "wc"),
    ] {
        environment.insert(
            format!("MANTLE_STAGE_X_AR_{name}"),
            shell_path(&request.coreutils_bin.join(tool), "binutils ar utility")?,
        );
    }
    environment.insert("MANTLE_STAGE_X_AR_AUDIT".to_string(), shell_path(&paths.audit, "binutils ar audit")?);
    environment.insert("MANTLE_STAGE_X_AR_COUNT".to_string(), shell_path(&paths.count, "binutils ar count")?);
    environment.insert("MANTLE_STAGE_X_AR_SCRATCH".to_string(), shell_path(&paths.scratch, "binutils ar scratch")?);
    assert_eq!(environment.len(), 8);
    assert!(environment.contains_key("MANTLE_STAGE_X_AR_AUDIT"));
    Ok(environment)
}

fn finalize_archive_runner_audit(paths: &ArchiveRunnerPaths) -> Result<(), StagexBinutilsError> {
    let count_text = fs::read_to_string(&paths.count)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading binutils ar count: {error}")))?;
    let count = count_text
        .trim()
        .parse::<usize>()
        .map_err(|error| StagexBinutilsError::Materialization(format!("parsing binutils ar count: {error}")))?;
    let audit_text = fs::read_to_string(&paths.audit)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading binutils ar audit: {error}")))?;
    let entries = parse_archive_runner_audit(&audit_text)?;
    if count != entries.len() || count != BINUTILS_AR_EXPECTED_INVOCATION_COUNT {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils ar audit count mismatch: expected {BINUTILS_AR_EXPECTED_INVOCATION_COUNT}, count {count}, entries {}",
            entries.len()
        )));
    }
    let mut canonical = String::new();
    for entry in &entries {
        canonical.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            entry.member_count, entry.output_bytes, entry.archive, entry.status
        ));
    }
    crate::stagex_mes_lib::write_create_new(&paths.scratch.join("audit.canonical.tsv"), canonical.as_bytes())
        .map_err(StagexBinutilsError::from_runtime)?;
    assert_eq!(entries.len(), BINUTILS_AR_EXPECTED_INVOCATION_COUNT);
    assert!(canonical.ends_with('\n'));
    Ok(())
}

fn parse_archive_runner_audit(text: &str) -> Result<Vec<ArchiveAuditEntry>, StagexBinutilsError> {
    let mut entries = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != BINUTILS_AR_AUDIT_FIELD_COUNT {
            return Err(StagexBinutilsError::Materialization(format!("malformed binutils ar audit: {line:?}")));
        }
        let invocation = fields[BINUTILS_AR_AUDIT_INVOCATION_INDEX].parse::<u32>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid binutils ar invocation: {error}"))
        })?;
        let expected_invocation = u32::try_from(index)
            .map_err(|error| StagexBinutilsError::Materialization(format!("binutils ar index overflow: {error}")))?
            .checked_add(1)
            .ok_or_else(|| StagexBinutilsError::Materialization("binutils ar invocation overflow".to_string()))?;
        let member_count = fields[BINUTILS_AR_AUDIT_MEMBER_COUNT_INDEX].parse::<u32>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid binutils ar member count: {error}"))
        })?;
        let output_bytes = fields[BINUTILS_AR_AUDIT_OUTPUT_BYTES_INDEX].parse::<u64>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid binutils ar output size: {error}"))
        })?;
        let expected_archive = BINUTILS_AR_EXPECTED_ARCHIVES.get(index).copied().unwrap_or("");
        let archive = fields[BINUTILS_AR_AUDIT_ARCHIVE_INDEX];
        let status = fields[BINUTILS_AR_AUDIT_STATUS_INDEX];
        if invocation != expected_invocation || member_count == 0 || output_bytes == 0 {
            return Err(StagexBinutilsError::Materialization(format!("invalid binutils ar audit facts: {line:?}")));
        }
        if archive != expected_archive || status != "ok" {
            return Err(StagexBinutilsError::Materialization(format!("substituted binutils ar audit facts: {line:?}")));
        }
        entries.push(ArchiveAuditEntry {
            invocation,
            member_count,
            output_bytes,
            archive: archive.to_string(),
            status: status.to_string(),
        });
    }
    if entries.len() != BINUTILS_AR_EXPECTED_INVOCATION_COUNT {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils ar audit entry mismatch: expected {BINUTILS_AR_EXPECTED_INVOCATION_COUNT}, observed {}",
            entries.len()
        )));
    }
    assert!(entries.iter().all(|entry| entry.status == "ok"));
    assert_eq!(entries.len(), BINUTILS_AR_EXPECTED_INVOCATION_COUNT);
    Ok(entries)
}

fn run_archive_runner_case(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &ArchiveRunnerPaths,
    current_dir: &Path,
    arguments: &[&str],
    environment: &std::collections::BTreeMap<String, String>,
    label: &str,
    stdin_path: &Path,
) -> Result<i32, StagexBinutilsError> {
    let mut command_arguments = vec![shell_path(&paths.script, "binutils ar runner")?];
    command_arguments.extend(arguments.iter().map(|argument| (*argument).to_string()));
    let argument_refs = command_arguments.iter().map(String::as_str).collect::<Vec<_>>();
    let stdout = request.scratch_dir.join(format!("ar-{label}.stdout.txt"));
    let stderr = request.scratch_dir.join(format!("ar-{label}.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &argument_refs,
        current_dir,
        environment,
        stdin_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(status)
}

fn require_archive_runner_rejection(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &ArchiveRunnerPaths,
    current_dir: &Path,
    arguments: &[&str],
    environment: &std::collections::BTreeMap<String, String>,
    label: &str,
    stdin_path: &Path,
) -> Result<(), StagexBinutilsError> {
    let status = run_archive_runner_case(request, paths, current_dir, arguments, environment, label, stdin_path)?;
    if status != BINUTILS_AR_AUTHORITY_FAILURE {
        return Err(StagexBinutilsError::Materialization(format!(
            "binutils ar {label} negative smoke mismatch: expected {BINUTILS_AR_AUTHORITY_FAILURE}, observed {status}"
        )));
    }
    assert_eq!(status, BINUTILS_AR_AUTHORITY_FAILURE);
    assert!(!arguments.is_empty());
    Ok(())
}

fn run_archive_runner_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &ArchiveRunnerPaths,
) -> Result<(), StagexBinutilsError> {
    let root = request.scratch_dir.join("ar-runner-smokes");
    let scratch = root.join("scratch");
    fs::create_dir(&root)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating binutils ar smoke root: {error}")))?;
    fs::create_dir(&scratch).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating binutils ar smoke scratch: {error}"))
    })?;
    let smoke_paths = ArchiveRunnerPaths {
        script: paths.script.clone(),
        scratch,
        audit: root.join("audit.tsv"),
        count: root.join("count.txt"),
    };
    let stdin_path = root.join("empty.stdin");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&root.join("a.o"), BINUTILS_AR_SMOKE_SHORT_BYTES)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&root.join("a-very-long-member-name.o"), BINUTILS_AR_SMOKE_LONG_BYTES)
        .map_err(StagexBinutilsError::from_runtime)?;
    let environment = archive_runner_environment(request, &smoke_paths)?;
    let status = run_archive_runner_case(
        request,
        &smoke_paths,
        &root,
        &["rc", "libtest.a", "a.o", "a-very-long-member-name.o"],
        &environment,
        "positive",
        &stdin_path,
    )?;
    if status != 0 {
        return Err(StagexBinutilsError::Materialization(format!("binutils ar positive smoke failed: {status}")));
    }
    validate_file_digest(&root.join("libtest.a"), BINUTILS_AR_SMOKE_ARCHIVE_BLAKE3, "binutils ar smoke archive")?;
    run_archive_runner_negative_smokes(request, &smoke_paths, &root, &environment, &stdin_path)?;
    assert!(root.join("libtest.a").is_file());
    assert!(smoke_paths.audit.is_file());
    Ok(())
}

fn run_archive_runner_negative_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    paths: &ArchiveRunnerPaths,
    root: &Path,
    environment: &std::collections::BTreeMap<String, String>,
    stdin_path: &Path,
) -> Result<(), StagexBinutilsError> {
    require_archive_runner_rejection(
        request,
        paths,
        root,
        &["rc", "empty.a"],
        environment,
        "empty-members",
        stdin_path,
    )?;
    crate::stagex_mes_lib::write_create_new(&root.join("existing.a"), b"occupied")
        .map_err(StagexBinutilsError::from_runtime)?;
    require_archive_runner_rejection(
        request,
        paths,
        root,
        &["rc", "existing.a", "a.o"],
        environment,
        "existing-output",
        stdin_path,
    )?;
    require_archive_runner_rejection(
        request,
        paths,
        root,
        &["t", "rejected.a", "a.o"],
        environment,
        "unsupported-mode",
        stdin_path,
    )?;
    require_archive_runner_rejection(
        request,
        paths,
        root,
        &["rc", "../escape.a", "a.o"],
        environment,
        "path-escape",
        stdin_path,
    )?;
    symlink("a.o", root.join("link.o")).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating binutils ar symlink smoke: {error}"))
    })?;
    require_archive_runner_rejection(
        request,
        paths,
        root,
        &["rc", "symlink.a", "link.o"],
        environment,
        "symlink",
        stdin_path,
    )?;
    assert!(root.join("link.o").is_symlink());
    assert!(stdin_path.is_file());
    Ok(())
}

fn prepare_sed_bridge(
    request: &BinutilsConfigureProbeRequest<'_>,
    compiler_wrapper: &Path,
) -> Result<SedBridgePaths, StagexBinutilsError> {
    validate_sed_bridge_sources()?;
    let launcher_source = request.scratch_dir.join(SED_BRIDGE_LAUNCHER_SOURCE_NAME);
    let semaphore_source = request.scratch_dir.join(SINGLE_THREAD_SEMAPHORE_SOURCE_NAME);
    let launcher = request.scratch_dir.join(SED_BRIDGE_LAUNCHER_OUTPUT_NAME);
    let script = request.scratch_dir.join("stagex-sed-regular-file-bridge.sh");
    let ylwrap_source = request.scratch_dir.join(YLWRAP_SED_RUNNER_SOURCE_NAME);
    let ylwrap_runner = request.scratch_dir.join(YLWRAP_SED_RUNNER_OUTPUT_NAME);
    let spool_root = request.scratch_dir.join("sed-spool");
    crate::stagex_mes_lib::write_create_new(&launcher_source, SED_BRIDGE_LAUNCHER_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&script, SED_BRIDGE_SCRIPT_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&semaphore_source, SINGLE_THREAD_SEMAPHORE_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&ylwrap_source, YLWRAP_SED_RUNNER_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting sed bridge script mode: {error}")))?;
    fs::create_dir(&spool_root)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating sed spool root: {error}")))?;
    compile_sed_bridge_launcher(request, compiler_wrapper, &launcher_source, &semaphore_source, &launcher)?;
    compile_ylwrap_sed_runner(request, compiler_wrapper, &ylwrap_runner)?;
    run_sed_bridge_launcher_self_test(request, &launcher)?;
    run_sed_bridge_launcher_missing_authority_test(request, &launcher)?;
    assert!(launcher.is_file());
    assert!(spool_root.is_dir());
    Ok(SedBridgePaths {
        launcher,
        script,
        ylwrap_runner,
        audit: spool_root.join("audit.tsv"),
        spool_root,
    })
}

fn prepare_configure_utility(
    request: &BinutilsConfigureProbeRequest<'_>,
    compiler_wrapper: &Path,
) -> Result<PathBuf, StagexBinutilsError> {
    require_sed_bridge_source_digest("configure utility", CONFIGURE_UTILITY_SOURCE, CONFIGURE_UTILITY_SOURCE_BLAKE3)?;
    let source = request.scratch_dir.join(CONFIGURE_UTILITY_SOURCE_NAME);
    let semaphore_source = request.scratch_dir.join(SINGLE_THREAD_SEMAPHORE_SOURCE_NAME);
    let output = request.scratch_dir.join(CONFIGURE_UTILITY_OUTPUT_NAME);
    crate::stagex_mes_lib::write_create_new(&source, CONFIGURE_UTILITY_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    if !semaphore_source.is_file() {
        return Err(StagexBinutilsError::Materialization(
            "configure utility lacks the declared semaphore source".to_string(),
        ));
    }
    compile_configure_utility(request, compiler_wrapper, &output)?;
    crate::stagex_mes_lib::run_bounded_process(
        &output,
        &["--mantle-self-test"],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("configure-utility-self-test.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(source.is_file());
    assert!(output.is_file());
    Ok(output)
}

fn compile_configure_utility(
    request: &BinutilsConfigureProbeRequest<'_>,
    compiler_wrapper: &Path,
    output: &Path,
) -> Result<(), StagexBinutilsError> {
    let arguments = vec![
        shell_path(compiler_wrapper, "TinyCC wrapper")?,
        format!("-I{}", shell_path(&request.musl_root.join("include"), "native musl include")?),
        CONFIGURE_UTILITY_SOURCE_NAME.to_string(),
        SINGLE_THREAD_SEMAPHORE_SOURCE_NAME.to_string(),
        "-o".to_string(),
        CONFIGURE_UTILITY_OUTPUT_NAME.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.bash,
        &arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("configure-utility-compile.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting configure utility mode: {error}")))?;
    validate_file_digest(output, CONFIGURE_UTILITY_BLAKE3, "configure utility")?;
    assert_eq!(arguments.len(), NATIVE_HELPER_COMPILE_ARGUMENT_COUNT);
    assert!(output.is_file());
    Ok(())
}

fn validate_sed_bridge_sources() -> Result<(), StagexBinutilsError> {
    require_sed_bridge_source_digest("launcher", SED_BRIDGE_LAUNCHER_SOURCE, SED_BRIDGE_LAUNCHER_SOURCE_BLAKE3)?;
    require_sed_bridge_source_digest("script", SED_BRIDGE_SCRIPT_SOURCE, SED_BRIDGE_SCRIPT_SOURCE_BLAKE3)?;
    require_sed_bridge_source_digest(
        "single-thread semaphore compatibility",
        SINGLE_THREAD_SEMAPHORE_SOURCE,
        SINGLE_THREAD_SEMAPHORE_SOURCE_BLAKE3,
    )?;
    require_sed_bridge_source_digest("configure utility", CONFIGURE_UTILITY_SOURCE, CONFIGURE_UTILITY_SOURCE_BLAKE3)?;
    require_sed_bridge_source_digest("ylwrap sed runner", YLWRAP_SED_RUNNER_SOURCE, YLWRAP_SED_RUNNER_SOURCE_BLAKE3)?;
    require_sed_bridge_source_digest(
        "binutils ar runner",
        BINUTILS_AR_RUNNER_SOURCE,
        BINUTILS_AR_RUNNER_SOURCE_BLAKE3,
    )?;
    require_sed_bridge_source_digest(
        "ELF symbol canonicalizer",
        ELF_SYMBOL_CANONICALIZER_SOURCE,
        ELF_SYMBOL_CANONICALIZER_SOURCE_BLAKE3,
    )?;
    assert_ne!(SED_BRIDGE_LAUNCHER_SOURCE_BLAKE3, SED_BRIDGE_SCRIPT_SOURCE_BLAKE3);
    assert!(!SED_BRIDGE_SCRIPT_SOURCE.is_empty());
    Ok(())
}

fn require_sed_bridge_source_digest(label: &str, bytes: &[u8], expected: &str) -> Result<(), StagexBinutilsError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge {label} source BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexBinutilsError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, CONFIGURE_OUTPUT_BYTES_MAX, label)
        .map_err(StagexBinutilsError::from_runtime)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(())
}

fn compile_sed_bridge_launcher(
    request: &BinutilsConfigureProbeRequest<'_>,
    compiler_wrapper: &Path,
    source: &Path,
    semaphore_source: &Path,
    output: &Path,
) -> Result<(), StagexBinutilsError> {
    if source != request.scratch_dir.join(SED_BRIDGE_LAUNCHER_SOURCE_NAME)
        || semaphore_source != request.scratch_dir.join(SINGLE_THREAD_SEMAPHORE_SOURCE_NAME)
        || output != request.scratch_dir.join(SED_BRIDGE_LAUNCHER_OUTPUT_NAME)
    {
        return Err(StagexBinutilsError::Materialization(
            "sed bridge compile paths differ from the fixed relative names".to_string(),
        ));
    }
    let arguments = vec![
        shell_path(compiler_wrapper, "TinyCC wrapper")?,
        format!("-I{}", shell_path(&request.musl_root.join("include"), "native musl include")?),
        SED_BRIDGE_LAUNCHER_SOURCE_NAME.to_string(),
        SINGLE_THREAD_SEMAPHORE_SOURCE_NAME.to_string(),
        "-o".to_string(),
        SED_BRIDGE_LAUNCHER_OUTPUT_NAME.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.bash,
        &arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("sed-bridge-launcher-compile.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting sed bridge launcher mode: {error}")))?;
    validate_file_digest(output, SED_BRIDGE_LAUNCHER_BLAKE3, "sed bridge launcher")?;
    assert_eq!(arguments.len(), NATIVE_HELPER_COMPILE_ARGUMENT_COUNT);
    assert!(source.is_file());
    assert!(semaphore_source.is_file());
    assert!(output.is_file());
    Ok(())
}

fn compile_ylwrap_sed_runner(
    request: &BinutilsConfigureProbeRequest<'_>,
    compiler_wrapper: &Path,
    output: &Path,
) -> Result<(), StagexBinutilsError> {
    if output != request.scratch_dir.join(YLWRAP_SED_RUNNER_OUTPUT_NAME) {
        return Err(StagexBinutilsError::Materialization(
            "ylwrap sed runner output differs from the fixed relative name".to_string(),
        ));
    }
    let arguments = vec![
        shell_path(compiler_wrapper, "TinyCC wrapper")?,
        format!("-I{}", shell_path(&request.musl_root.join("include"), "native musl include")?),
        YLWRAP_SED_RUNNER_SOURCE_NAME.to_string(),
        SINGLE_THREAD_SEMAPHORE_SOURCE_NAME.to_string(),
        "-o".to_string(),
        YLWRAP_SED_RUNNER_OUTPUT_NAME.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.bash,
        &arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("ylwrap-sed-runner-compile.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(EXECUTABLE_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting ylwrap sed runner mode: {error}")))?;
    validate_file_digest(output, YLWRAP_SED_RUNNER_BLAKE3, "ylwrap sed runner")?;
    crate::stagex_mes_lib::run_bounded_process(
        output,
        &["--mantle-self-test"],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("ylwrap-sed-runner-self-test.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    run_ylwrap_sed_runner_smokes(request, output)?;
    assert_eq!(arguments.len(), NATIVE_HELPER_COMPILE_ARGUMENT_COUNT);
    assert!(output.is_file());
    Ok(())
}

fn quote_ylwrap_sed_pattern(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(character, '[' | ']' | '\\' | '.' | '*') {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    assert!(quoted.len() >= text.len());
    assert!(!quoted.contains('\n'));
    quoted
}

fn ylwrap_sed_programs(parent: &Path) -> Result<[String; YLWRAP_SED_DYNAMIC_PROGRAM_COUNT], StagexBinutilsError> {
    let parent = shell_path(parent, "ylwrap sed smoke parent")?;
    let path_program = format!("s|{}/||", quote_ylwrap_sed_pattern(&parent));
    let names = "s|y\\.tab\\.c|arparse.c|g;s|y\\.tab\\.h|arparse.h|g;s|y\\.output|arparse.output|g;";
    let guards = "s|Y_TAB_C|ARPARSE_C|g;s|Y_TAB_H|ARPARSE_H|g;s|Y_OUTPUT|ARPARSE_OUTPUT|g;";
    assert!(path_program.starts_with("s|/"));
    assert!(names.contains("arparse.c"));
    Ok([path_program, names.to_string(), guards.to_string()])
}

fn run_ylwrap_sed_runner_case(
    runner: &Path,
    current_dir: &Path,
    programs: &[String; YLWRAP_SED_DYNAMIC_PROGRAM_COUNT],
    input_name: &str,
    label: &str,
    stdin_path: &Path,
    scratch_dir: &Path,
) -> Result<(i32, Vec<u8>), StagexBinutilsError> {
    let arguments = [
        "-e".to_string(),
        "/^#/!b".to_string(),
        "-e".to_string(),
        programs[YLWRAP_SED_PATH_PROGRAM_INDEX].clone(),
        "-e".to_string(),
        programs[YLWRAP_SED_NAME_PROGRAM_INDEX].clone(),
        "-e".to_string(),
        programs[YLWRAP_SED_GUARD_PROGRAM_INDEX].clone(),
        input_name.to_string(),
    ];
    let argument_refs = arguments.iter().map(String::as_str).collect::<Vec<_>>();
    let stdout = scratch_dir.join(format!("ylwrap-sed-{label}.stdout.txt"));
    let stderr = scratch_dir.join(format!("ylwrap-sed-{label}.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        runner,
        &argument_refs,
        current_dir,
        &std::collections::BTreeMap::new(),
        stdin_path,
        YLWRAP_SED_FILE_BYTES_MAX,
        &stdout,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let output = fs::read(&stdout)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading ylwrap sed {label} output: {error}")))?;
    assert_eq!(arguments.len(), YLWRAP_SED_ARGUMENT_COUNT);
    assert!(stderr.is_file());
    Ok((status, output))
}

fn run_ylwrap_sed_runner_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    runner: &Path,
) -> Result<(), StagexBinutilsError> {
    let parent = request.scratch_dir.join("ylwrap-sed-smokes");
    let current_dir = parent.join("ylwrap1");
    fs::create_dir(&parent)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating ylwrap sed smoke parent: {error}")))?;
    fs::create_dir(&current_dir).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating ylwrap sed smoke directory: {error}"))
    })?;
    let stdin_path = parent.join("empty.stdin");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    let programs = ylwrap_sed_programs(&parent)?;
    let input = format!(
        "#line 1 \"{}/arparse.y\"\n#include \"y.tab.h\"\n#ifndef Y_TAB_C\nbody y.tab.c\n",
        shell_path(&parent, "ylwrap sed fixture parent")?
    );
    crate::stagex_mes_lib::write_create_new(&current_dir.join("y.tab.c"), input.as_bytes())
        .map_err(StagexBinutilsError::from_runtime)?;
    let (status, output) = run_ylwrap_sed_runner_case(
        runner,
        &current_dir,
        &programs,
        "y.tab.c",
        "positive",
        &stdin_path,
        request.scratch_dir,
    )?;
    let expected = b"#line 1 \"arparse.y\"\n#include \"arparse.h\"\n#ifndef ARPARSE_C\nbody y.tab.c\n";
    if status != 0 || output != expected {
        return Err(StagexBinutilsError::Materialization(format!(
            "ylwrap sed positive smoke mismatch: status {status}, output {:?}",
            String::from_utf8_lossy(&output)
        )));
    }
    run_ylwrap_sed_runner_negative_smokes(request, runner, &parent, &current_dir, &programs, &stdin_path)?;
    assert_eq!(output, expected);
    assert!(runner.is_file());
    Ok(())
}

fn require_ylwrap_sed_rejection(
    request: &BinutilsConfigureProbeRequest<'_>,
    runner: &Path,
    current_dir: &Path,
    programs: &[String; YLWRAP_SED_DYNAMIC_PROGRAM_COUNT],
    input_name: &str,
    label: &str,
    stdin_path: &Path,
) -> Result<(), StagexBinutilsError> {
    let (status, output) =
        run_ylwrap_sed_runner_case(runner, current_dir, programs, input_name, label, stdin_path, request.scratch_dir)?;
    if status != SED_BRIDGE_AUTHORITY_FAILURE || !output.is_empty() {
        return Err(StagexBinutilsError::Materialization(format!(
            "ylwrap sed {label} negative smoke mismatch: status {status}, output bytes {}",
            output.len()
        )));
    }
    assert_eq!(status, SED_BRIDGE_AUTHORITY_FAILURE);
    assert!(output.is_empty());
    Ok(())
}

fn run_ylwrap_sed_runner_negative_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    runner: &Path,
    parent: &Path,
    current_dir: &Path,
    programs: &[String; YLWRAP_SED_DYNAMIC_PROGRAM_COUNT],
    stdin_path: &Path,
) -> Result<(), StagexBinutilsError> {
    let mut wrong_path = programs.clone();
    wrong_path[YLWRAP_SED_PATH_PROGRAM_INDEX] = "s|/substituted/source/||".to_string();
    require_ylwrap_sed_rejection(request, runner, current_dir, &wrong_path, "y.tab.c", "wrong-path", stdin_path)?;
    require_ylwrap_sed_rejection(request, runner, parent, programs, "y.tab.c", "wrong-directory", stdin_path)?;
    let scanner_programs = [
        programs[YLWRAP_SED_PATH_PROGRAM_INDEX].clone(),
        "s|lex\\.yy\\.c|arlex.c|g;".to_string(),
        "s|LEX_YY_C|ARLEX_C|g;".to_string(),
    ];
    symlink("y.tab.c", current_dir.join("lex.yy.c"))
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating ylwrap sed symlink smoke: {error}")))?;
    require_ylwrap_sed_rejection(request, runner, current_dir, &scanner_programs, "lex.yy.c", "symlink", stdin_path)?;
    let oversized_dir = parent.join("ylwrap2");
    fs::create_dir(&oversized_dir).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating ylwrap sed oversized directory: {error}"))
    })?;
    let oversized_len = usize::try_from(YLWRAP_SED_FILE_BYTES_MAX)
        .map_err(|error| StagexBinutilsError::Materialization(format!("converting ylwrap sed limit: {error}")))?
        .checked_add(1)
        .ok_or_else(|| StagexBinutilsError::Materialization("ylwrap sed oversized fixture overflow".to_string()))?;
    crate::stagex_mes_lib::write_create_new(&oversized_dir.join("y.tab.c"), &vec![b'x'; oversized_len])
        .map_err(StagexBinutilsError::from_runtime)?;
    require_ylwrap_sed_rejection(request, runner, &oversized_dir, programs, "y.tab.c", "oversized", stdin_path)?;
    assert!(current_dir.join("lex.yy.c").is_symlink());
    assert!(oversized_len > usize::try_from(YLWRAP_SED_FILE_BYTES_MAX).unwrap());
    Ok(())
}

fn run_sed_bridge_launcher_self_test(
    request: &BinutilsConfigureProbeRequest<'_>,
    launcher: &Path,
) -> Result<(), StagexBinutilsError> {
    crate::stagex_mes_lib::run_bounded_process(
        launcher,
        &["--mantle-self-test"],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("sed-bridge-launcher-self-test.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(launcher.is_file());
    assert!(request.scratch_dir.is_dir());
    Ok(())
}

fn run_configure_utility_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    tools: &Path,
    configure_utility: &Path,
) -> Result<(), StagexBinutilsError> {
    let empty_stdin = request.scratch_dir.join("configure-utility-empty.stdin");
    crate::stagex_mes_lib::write_create_new(&empty_stdin, b"").map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::run_bounded_process(
        &tools.join("sleep"),
        &["0"],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join("configure-sleep-positive.stderr.txt"),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    run_configure_utility_negative_smoke(request, &tools.join("sleep"), &["11"], &empty_stdin, "sleep")?;
    run_configure_utility_file_smokes(request, tools, configure_utility, &empty_stdin)?;
    assert!(tools.join("sleep").is_symlink());
    assert!(tools.join("file").is_symlink());
    Ok(())
}

fn run_configure_utility_file_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    tools: &Path,
    configure_utility: &Path,
    empty_stdin: &Path,
) -> Result<(), StagexBinutilsError> {
    let stdout_path = request.scratch_dir.join("configure-file-positive.stdout.txt");
    let stderr_path = request.scratch_dir.join("configure-file-positive.stderr.txt");
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        &tools.join("file"),
        &[CONFIGURE_UTILITY_OUTPUT_NAME],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        empty_stdin,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let stdout = fs::read_to_string(&stdout_path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading configure file smoke: {error}")))?;
    if status != 0 || !stdout.contains("ELF 64-bit LSB") {
        return Err(StagexBinutilsError::Materialization(format!(
            "configure file positive smoke mismatch: status {status}, stdout {stdout:?}"
        )));
    }
    run_configure_utility_negative_smoke(
        request,
        &tools.join("file"),
        &["/usr/bin/file"],
        empty_stdin,
        "file-absolute",
    )?;
    run_configure_utility_negative_smoke(
        request,
        &tools.join("file"),
        &["nested/conftest"],
        empty_stdin,
        "file-nested",
    )?;
    let symlink_name = "configure-file-symlink";
    let symlink_path = request.scratch_dir.join(symlink_name);
    std::os::unix::fs::symlink(configure_utility, &symlink_path).map_err(|error| {
        StagexBinutilsError::Materialization(format!("creating configure file smoke symlink: {error}"))
    })?;
    run_configure_utility_negative_smoke(request, &tools.join("file"), &[symlink_name], empty_stdin, "file-symlink")?;
    assert!(configure_utility.is_file());
    assert!(symlink_path.is_symlink());
    assert!(!stdout.is_empty());
    Ok(())
}

fn run_configure_utility_negative_smoke(
    request: &BinutilsConfigureProbeRequest<'_>,
    executable: &Path,
    arguments: &[&str],
    stdin_path: &Path,
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let stdout_path = request.scratch_dir.join(format!("configure-{label}-negative.stdout.txt"));
    let stderr_path = request.scratch_dir.join(format!("configure-{label}-negative.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        executable,
        arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        stdin_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    if status != SED_BRIDGE_AUTHORITY_FAILURE {
        return Err(StagexBinutilsError::Materialization(format!(
            "configure {label} negative smoke mismatch: expected {SED_BRIDGE_AUTHORITY_FAILURE}, observed {status}"
        )));
    }
    assert_eq!(status, SED_BRIDGE_AUTHORITY_FAILURE);
    assert!(stderr_path.is_file());
    Ok(())
}

fn run_sed_bridge_launcher_missing_authority_test(
    request: &BinutilsConfigureProbeRequest<'_>,
    launcher: &Path,
) -> Result<(), StagexBinutilsError> {
    let stdin_path = request.scratch_dir.join("sed-bridge-missing-authority.stdin");
    let stdout_path = request.scratch_dir.join("sed-bridge-missing-authority.stdout.txt");
    let stderr_path = request.scratch_dir.join("sed-bridge-missing-authority.stderr.txt");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        launcher,
        &["s/x/y/"],
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &stdin_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let stderr = fs::read_to_string(&stderr_path).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading sed bridge missing-authority stderr: {error}"))
    })?;
    if status != SED_BRIDGE_AUTHORITY_FAILURE || !stderr.contains("missing absolute launcher authority") {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge missing-authority test mismatch: status {status}, stderr {stderr:?}"
        )));
    }
    assert_eq!(status, SED_BRIDGE_AUTHORITY_FAILURE);
    assert!(!stderr.is_empty());
    Ok(())
}

fn extract_recipe_block(start: &str, end: &str, label: &str) -> Result<String, StagexBinutilsError> {
    let recipe = std::str::from_utf8(BINUTILS_RECIPE)
        .map_err(|error| StagexBinutilsError::Materialization(format!("{label} recipe is not UTF-8: {error}")))?;
    let parts = recipe.split(start).collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(StagexBinutilsError::Materialization(format!("{label} start marker count differs")));
    }
    let endings = parts[1].split(end).collect::<Vec<_>>();
    if endings.len() != 2 {
        return Err(StagexBinutilsError::Materialization(format!("{label} end marker count differs")));
    }
    assert!(!endings[0].is_empty());
    assert_eq!(parts.len(), 2);
    Ok(endings[0].to_string())
}

fn compile_runtime_object(
    request: &BinutilsConfigureProbeRequest<'_>,
    source: &Path,
    output: &Path,
    label: &str,
) -> Result<(), StagexBinutilsError> {
    let arguments = vec![
        "-c".to_string(),
        format!("-I{}", shell_path(&request.musl_root.join("include"), "native musl include")?),
        shell_path(source, "runtime source")?,
        "-o".to_string(),
        shell_path(output, "runtime object")?,
    ];
    crate::stagex_mes_lib::run_bounded_process(
        request.tcc,
        &arguments,
        request.scratch_dir,
        &std::collections::BTreeMap::new(),
        &request.scratch_dir.join(format!("{label}-compile.stderr.txt")),
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(output, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting {label} object mode: {error}")))?;
    assert!(source.is_file());
    assert!(output.is_file());
    Ok(())
}

const TCC_WRAPPER_TEMPLATE: &str = r#"set -eu
REAL_TCC='@TCC@'
MUSL_INCLUDE='@MUSL_INCLUDE@'
MUSL_LIB='@MUSL_LIB@'
RUNTIME_OBJECT='@RUNTIME@'
RUNTIME_ASM_OBJECT='@RUNTIME_ASM@'
CANONICALIZER='@CANONICALIZER@'
CHMOD='@CHMOD@'
CAT='@CAT@'
RM='@RM@'
WC='@WC@'
GREP='@GREP@'
SED='@SED@'
SOURCE_BYTES_MAX=@SOURCE_BYTES_MAX@
INVOCATION_COUNT_MAX=@INVOCATION_COUNT_MAX@
mode=link
source_file=
output_file=
previous=
for argument in "$@"; do
  if test "$previous" = -o; then output_file="$argument"; fi
  case "$argument" in
    -c) mode=compile ;;
    -E) mode=preprocess ;;
    --version|-V|-qversion) exec "$REAL_TCC" -v ;;
    *.c) source_file="$argument" ;;
  esac
  previous="$argument"
done
if test "$mode" = preprocess; then
  reject_probe() {
    printf '%s\n' "binutils-tcc-wrapper: configure preprocess rejected: $1" >&2
    exit 1
  }
  probe_dir=${MANTLE_BINUTILS_CONFIGURE_PROBE_DIR:-}
  probe_class=${MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS:-}
  probe_audit=${MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT:-}
  probe_count_file=${MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT:-}
  test -n "$probe_dir" || reject_probe missing-authority-directory
  test -n "$probe_audit" || reject_probe missing-audit-path
  test -n "$probe_count_file" || reject_probe missing-count-path
  case "$probe_class" in
    intl|libiberty|zlib|bfd|opcodes|binutils|gas|gprof|ld) ;;
    *) reject_probe unknown-configure-class ;;
  esac
  current_dir=$PWD
  test "$current_dir" = "$probe_dir" || reject_probe current-directory-mismatch
  case "$source_file" in conftest.c|./conftest.c) ;; *) reject_probe non-conftest-source ;; esac
  test -z "$output_file" || reject_probe explicit-output-forbidden
  source_canonical="$probe_dir/conftest.c"
  test -f "$source_canonical" || reject_probe source-unavailable
  source_bytes=$("$WC" -c < "$source_canonical") || reject_probe source-size-unavailable
  source_bytes=${source_bytes//[[:space:]]/}
  case "$source_bytes" in ''|*[!0-9]*) reject_probe invalid-source-size ;; esac
  test "$source_bytes" -le "$SOURCE_BYTES_MAX" || reject_probe source-too-large
  probe_count=0
  if test -f "$probe_count_file"; then probe_count=$("$CAT" "$probe_count_file") || reject_probe count-read-failed; fi
  case "$probe_count" in ''|*[!0-9]*) reject_probe invalid-count ;; esac
  test "$probe_count" -lt "$INVOCATION_COUNT_MAX" || reject_probe invocation-budget-exhausted
  next_probe_count=$((probe_count + 1))
  printf '%s\n' "$next_probe_count" > "$probe_count_file" || reject_probe count-write-failed
  printf '%s\t%s\t%s\n' "$next_probe_count" "$probe_class" "$source_canonical" >> "$probe_audit" || reject_probe audit-write-failed
  if "$GREP" -q '^#ifdef CHAR_BIT' "$source_canonical"; then
    "$GREP" -Eq '^#define[[:space:]]+CHAR_BIT[[:space:]]+8([[:space:]]|$)' "$MUSL_INCLUDE/limits.h" || reject_probe unexpected-char-bit
    printf 'found\n'
    exit 0
  fi
  scratch_source="$probe_dir/.mantle-cpp-source.c"
  scratch_object="$probe_dir/.mantle-cpp-object.o"
  "$RM" -f "$scratch_source" "$scratch_object"
  "$SED" '/^[[:space:]]*Syntax error[[:space:]]*$/d' "$source_canonical" > "$scratch_source"
  printf '\nint mantle_configure_cpp_probe(void) { return 0; }\n' >> "$scratch_source"
  "$REAL_TCC" -c -I"$MUSL_INCLUDE" "$scratch_source" -o "$scratch_object" || reject_probe compile-validation-failed
  test -s "$scratch_object" || reject_probe compile-validation-output-missing
  "$CAT" "$source_canonical"
  "$RM" -f "$scratch_source" "$scratch_object"
  exit 0
fi
if test "$mode" = compile; then
  "$REAL_TCC" "$@"
  compile_status=$?
  test "$compile_status" -eq 0 || exit "$compile_status"
  if test -z "$output_file"; then
    test -n "$source_file" || exit 1
    source_name=${source_file##*/}
    output_file=${source_name%.c}.o
  fi
  test -s "$output_file" || exit 1
  "$CHMOD" 644 "$output_file"
  case "$output_file" in /*) canonical_output=$output_file ;; *) canonical_output=$PWD/$output_file ;; esac
  "$CANONICALIZER" "$canonical_output"
  exit 0
fi
if test -z "$output_file"; then output_file=a.out; fi
test -s "$RUNTIME_OBJECT"
test -s "$RUNTIME_ASM_OBJECT"
"$REAL_TCC" -nostdlib -static "$MUSL_LIB/crt1.o" "$@" "$RUNTIME_OBJECT" "$RUNTIME_ASM_OBJECT" "$MUSL_LIB/libc.a"
link_status=$?
test "$link_status" -eq 0 || exit "$link_status"
test -s "$output_file"
"$CHMOD" 755 "$output_file"
case "$output_file" in /*) canonical_output=$output_file ;; *) canonical_output=$PWD/$output_file ;; esac
"$CANONICALIZER" "$canonical_output"
"#;

fn tcc_wrapper_script(
    request: &BinutilsConfigureProbeRequest<'_>,
    runtime_object: &Path,
    runtime_assembly_object: &Path,
    canonicalizer: &Path,
) -> Result<String, StagexBinutilsError> {
    let bindings = [
        ("@TCC@", shell_path(request.tcc, "TinyCC")?),
        ("@MUSL_INCLUDE@", shell_path(&request.musl_root.join("include"), "native musl include")?),
        ("@MUSL_LIB@", shell_path(&request.musl_root.join("lib"), "native musl lib")?),
        ("@RUNTIME@", shell_path(runtime_object, "runtime object")?),
        ("@RUNTIME_ASM@", shell_path(runtime_assembly_object, "runtime assembly object")?),
        ("@CANONICALIZER@", shell_path(canonicalizer, "ELF symbol canonicalizer")?),
        ("@CHMOD@", shell_path(&request.coreutils_bin.join("chmod"), "chmod")?),
        ("@CAT@", shell_path(&request.coreutils_bin.join("cat"), "cat")?),
        ("@RM@", shell_path(&request.coreutils_bin.join("rm"), "rm")?),
        ("@WC@", shell_path(&request.coreutils_bin.join("wc"), "wc")?),
        ("@GREP@", shell_path(request.grep, "grep")?),
        ("@SED@", shell_path(request.sed, "sed")?),
        ("@SOURCE_BYTES_MAX@", CONFIGURE_PROBE_SOURCE_BYTES_MAX.to_string()),
        ("@INVOCATION_COUNT_MAX@", CONFIGURE_PROBE_INVOCATION_COUNT_MAX.to_string()),
    ];
    let mut script = TCC_WRAPPER_TEMPLATE.to_string();
    for (marker, value) in &bindings {
        script = script.replace(marker, value);
    }
    if bindings.iter().any(|(marker, _)| script.contains(marker)) {
        return Err(StagexBinutilsError::Materialization("compiler wrapper has unresolved bindings".to_string()));
    }
    assert!(script.contains("-nostdlib -static"));
    assert!(!script.contains("/bin/sh"));
    Ok(script)
}

fn run_sed_bridge_smokes(
    request: &BinutilsConfigureProbeRequest<'_>,
    bridge: &SedBridgePaths,
    environment: &std::collections::BTreeMap<String, String>,
) -> Result<(), StagexBinutilsError> {
    let smoke_root = request.scratch_dir.join("sed-bridge-smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating sed bridge smoke root: {error}")))?;
    let input = smoke_root.join("input.txt");
    let first = smoke_root.join("first.sed");
    let second = smoke_root.join("second.sed");
    let driver = smoke_root.join("positive.sh");
    let empty_stdin = smoke_root.join("empty.stdin");
    crate::stagex_mes_lib::write_create_new(&input, SED_BRIDGE_SMOKE_INPUT)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&first, SED_BRIDGE_FIRST_PROGRAM)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&second, SED_BRIDGE_SECOND_PROGRAM)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&empty_stdin, b"").map_err(StagexBinutilsError::from_runtime)?;
    let driver_bytes = sed_bridge_smoke_driver(bridge, &input, &first, &second)?;
    crate::stagex_mes_lib::write_create_new(&driver, driver_bytes.as_bytes())
        .map_err(StagexBinutilsError::from_runtime)?;
    run_positive_sed_bridge_smoke(request, environment, &driver, &empty_stdin, &smoke_root)?;
    run_negative_sed_bridge_smoke(bridge, environment, &input, &smoke_root)?;
    assert!(bridge.audit.is_file());
    assert!(smoke_root.is_dir());
    Ok(())
}

fn sed_bridge_smoke_driver(
    bridge: &SedBridgePaths,
    input: &Path,
    first: &Path,
    second: &Path,
) -> Result<String, StagexBinutilsError> {
    let driver = format!(
        "set -eu\nSED='{}'\n\"$SED\" -n -f '{}' < '{}' | \"$SED\" -f '{}'\n",
        shell_path(&bridge.launcher, "sed bridge launcher")?,
        shell_path(first, "first sed smoke program")?,
        shell_path(input, "sed smoke input")?,
        shell_path(second, "second sed smoke program")?,
    );
    assert!(driver.contains("\"$SED\""));
    assert!(!driver.contains("/bin/sh"));
    Ok(driver)
}

fn run_positive_sed_bridge_smoke(
    request: &BinutilsConfigureProbeRequest<'_>,
    environment: &std::collections::BTreeMap<String, String>,
    driver: &Path,
    stdin_path: &Path,
    smoke_root: &Path,
) -> Result<(), StagexBinutilsError> {
    let stdout_path = smoke_root.join("positive.stdout.txt");
    let stderr_path = smoke_root.join("positive.stderr.txt");
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &[shell_path(driver, "sed bridge smoke driver")?],
        smoke_root,
        environment,
        stdin_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let observed = fs::read(&stdout_path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading sed bridge smoke output: {error}")))?;
    if status != 0 || observed != SED_BRIDGE_SMOKE_EXPECTED {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge positive smoke mismatch: status {status}, observed {:?}",
            String::from_utf8_lossy(&observed)
        )));
    }
    assert_eq!(observed, SED_BRIDGE_SMOKE_EXPECTED);
    assert_eq!(status, 0);
    Ok(())
}

fn run_negative_sed_bridge_smoke(
    bridge: &SedBridgePaths,
    environment: &std::collections::BTreeMap<String, String>,
    input: &Path,
    smoke_root: &Path,
) -> Result<(), StagexBinutilsError> {
    let stdout_path = smoke_root.join("negative.stdout.txt");
    let stderr_path = smoke_root.join("negative.stderr.txt");
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        &bridge.launcher,
        &[
            "s/LTLIBOBJS/rejected/",
            shell_path(input, "negative sed smoke input")?.as_str(),
        ],
        smoke_root,
        environment,
        input,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stdout_path,
        SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    let stderr = fs::read_to_string(&stderr_path).map_err(|error| {
        StagexBinutilsError::Materialization(format!("reading sed bridge negative stderr: {error}"))
    })?;
    if status != SED_BRIDGE_AUTHORITY_FAILURE || !stderr.contains("mixed-input-authority") {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge negative smoke mismatch: status {status}, stderr {stderr:?}"
        )));
    }
    assert_eq!(status, SED_BRIDGE_AUTHORITY_FAILURE);
    assert!(!stderr.is_empty());
    Ok(())
}

fn validate_sed_bridge_total_count(bridge: &SedBridgePaths, expected: &[u32]) -> Result<(), StagexBinutilsError> {
    let observed = read_sed_bridge_total_count(bridge)?;
    validate_sed_bridge_count_value(observed, expected)
}

fn read_sed_bridge_total_count(bridge: &SedBridgePaths) -> Result<u32, StagexBinutilsError> {
    let count_text = fs::read_to_string(bridge.spool_root.join("invocation.count"))
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading final sed bridge count: {error}")))?;
    let observed = count_text
        .trim()
        .parse::<u32>()
        .map_err(|error| StagexBinutilsError::Materialization(format!("parsing final sed bridge count: {error}")))?;
    assert!(observed > 0);
    assert!(observed <= SED_BRIDGE_INVOCATION_COUNT_MAX);
    Ok(observed)
}

fn validate_sed_bridge_count_value(observed: u32, expected: &[u32]) -> Result<(), StagexBinutilsError> {
    if expected.is_empty() || !expected.contains(&observed) {
        return Err(StagexBinutilsError::Materialization(format!(
            "final sed bridge count mismatch: expected one of {expected:?}, observed {observed}"
        )));
    }
    assert!(observed > 0);
    assert!(expected.contains(&observed));
    Ok(())
}

fn finalize_sed_bridge_audit(bridge: &SedBridgePaths) -> Result<(), StagexBinutilsError> {
    let count_text = fs::read_to_string(bridge.spool_root.join("invocation.count"))
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading sed bridge count: {error}")))?;
    let expected_count = count_text
        .trim()
        .parse::<u32>()
        .map_err(|error| StagexBinutilsError::Materialization(format!("parsing sed bridge count: {error}")))?;
    let audit_text = fs::read_to_string(&bridge.audit)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading sed bridge audit: {error}")))?;
    let canonical = canonical_sed_bridge_audit(&audit_text, expected_count)?;
    crate::stagex_mes_lib::write_create_new(&bridge.spool_root.join("audit.canonical.tsv"), canonical.as_bytes())
        .map_err(StagexBinutilsError::from_runtime)?;
    assert!(expected_count > 0);
    assert!(!canonical.is_empty());
    Ok(())
}

fn validate_ylwrap_sed_audit(bridge: &SedBridgePaths) -> Result<(), StagexBinutilsError> {
    let canonical_path = bridge.spool_root.join("audit.canonical.tsv");
    let canonical = fs::read_to_string(&canonical_path)
        .map_err(|error| StagexBinutilsError::Materialization(format!("reading ylwrap sed audit: {error}")))?;
    let observed_count = validate_ylwrap_sed_canonical(&canonical)?;
    assert_eq!(observed_count, YLWRAP_SED_EXPECTED_INVOCATION_COUNT);
    assert!(canonical_path.is_file());
    Ok(())
}

fn validate_ylwrap_sed_canonical(canonical: &str) -> Result<usize, StagexBinutilsError> {
    let mut observed_count = 0usize;
    for line in canonical.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != SED_BRIDGE_CANONICAL_FIELD_COUNT {
            return Err(StagexBinutilsError::Materialization(format!("malformed canonical sed bridge line: {line:?}")));
        }
        if fields[SED_BRIDGE_CANONICAL_STATUS_INDEX] != "ok:ylwrap-sed" {
            continue;
        }
        let output_bytes = fields[SED_BRIDGE_CANONICAL_OUTPUT_INDEX].parse::<u64>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid ylwrap sed output size: {error}"))
        })?;
        if fields[SED_BRIDGE_CANONICAL_INPUT_INDEX] != "0"
            || fields[SED_BRIDGE_CANONICAL_FILE_COUNT_INDEX] != "1"
            || output_bytes == 0
        {
            return Err(StagexBinutilsError::Materialization(format!(
                "ylwrap sed audit has an invalid authority shape: {line:?}"
            )));
        }
        observed_count = observed_count
            .checked_add(1)
            .ok_or_else(|| StagexBinutilsError::Materialization("ylwrap sed audit count overflow".to_string()))?;
    }
    if observed_count != YLWRAP_SED_EXPECTED_INVOCATION_COUNT {
        return Err(StagexBinutilsError::Materialization(format!(
            "ylwrap sed invocation count mismatch: expected {YLWRAP_SED_EXPECTED_INVOCATION_COUNT}, observed {observed_count}"
        )));
    }
    assert_eq!(observed_count, YLWRAP_SED_EXPECTED_INVOCATION_COUNT);
    assert!(!canonical.is_empty());
    Ok(observed_count)
}

fn canonical_sed_bridge_audit(text: &str, expected_count: u32) -> Result<String, StagexBinutilsError> {
    if expected_count == 0 || expected_count > SED_BRIDGE_INVOCATION_COUNT_MAX {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge invocation count is outside bounds: {expected_count}"
        )));
    }
    let mut entries = text.lines().map(parse_sed_bridge_audit_line).collect::<Result<Vec<_>, _>>()?;
    if entries.len() != usize::try_from(expected_count).unwrap() {
        return Err(StagexBinutilsError::Materialization(format!(
            "sed bridge audit count mismatch: expected {expected_count}, observed {}",
            entries.len()
        )));
    }
    entries.sort_by_key(|entry| entry.invocation);
    validate_sed_bridge_audit_entries(&entries)?;
    entries.sort_by(|left, right| {
        (&left.status, left.input_bytes, left.output_bytes, left.input_file_count).cmp(&(
            &right.status,
            right.input_bytes,
            right.output_bytes,
            right.input_file_count,
        ))
    });
    let mut canonical = String::new();
    for entry in &entries {
        canonical.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            entry.input_bytes, entry.output_bytes, entry.input_file_count, entry.status
        ));
    }
    assert_eq!(entries.len(), usize::try_from(expected_count).unwrap());
    assert!(canonical.ends_with('\n'));
    Ok(canonical)
}

fn parse_sed_bridge_audit_line(line: &str) -> Result<SedBridgeAuditEntry, StagexBinutilsError> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != SED_BRIDGE_AUDIT_FIELD_COUNT {
        return Err(StagexBinutilsError::Materialization(format!("malformed sed bridge audit line: {line:?}")));
    }
    let parse_u32 = |value: &str, label: &str| {
        value.parse::<u32>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid sed bridge {label} {value:?}: {error}"))
        })
    };
    let parse_u64 = |value: &str, label: &str| {
        value.parse::<u64>().map_err(|error| {
            StagexBinutilsError::Materialization(format!("invalid sed bridge {label} {value:?}: {error}"))
        })
    };
    let status = fields[4];
    let accepted = status == "ok:protected-sed" || status == "ok:ylwrap-sed";
    if !accepted && !status.starts_with("rejected:") {
        return Err(StagexBinutilsError::Materialization(format!("invalid sed bridge status: {status:?}")));
    }
    Ok(SedBridgeAuditEntry {
        invocation: parse_u32(fields[0], "invocation")?,
        input_bytes: parse_u64(fields[1], "input size")?,
        output_bytes: parse_u64(fields[2], "output size")?,
        input_file_count: parse_u32(fields[3], "input file count")?,
        status: status.to_string(),
    })
}

fn validate_sed_bridge_audit_entries(entries: &[SedBridgeAuditEntry]) -> Result<(), StagexBinutilsError> {
    let mut mixed_input_rejection_seen = false;
    for (index, entry) in entries.iter().enumerate() {
        let expected_invocation = u32::try_from(index)
            .map_err(|error| StagexBinutilsError::Materialization(format!("sed audit index overflow: {error}")))?
            .checked_add(1)
            .ok_or_else(|| StagexBinutilsError::Materialization("sed audit invocation overflow".to_string()))?;
        if entry.invocation != expected_invocation {
            return Err(StagexBinutilsError::Materialization(format!(
                "sed bridge audit sequence mismatch: expected {expected_invocation}, observed {}",
                entry.invocation
            )));
        }
        mixed_input_rejection_seen |= entry.status == "rejected:mixed-input-authority";
    }
    if !mixed_input_rejection_seen {
        return Err(StagexBinutilsError::Materialization(
            "sed bridge audit lacks the mixed-input rejection smoke".to_string(),
        ));
    }
    assert!(!entries.is_empty());
    assert!(mixed_input_rejection_seen);
    Ok(())
}

fn shell_path(path: &Path, label: &str) -> Result<String, StagexBinutilsError> {
    if !path.is_absolute() || path.as_os_str().as_encoded_bytes().contains(&b'\'') {
        return Err(StagexBinutilsError::Materialization(format!(
            "{label} path is not an absolute shell-safe path: {}",
            path.display()
        )));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexBinutilsError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value.to_string())
}

fn configure_environment(
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
    configure_class: &str,
) -> Result<std::collections::BTreeMap<String, String>, StagexBinutilsError> {
    let bash = shell_path(request.bash, "full Bash")?;
    let wrapper = shell_path(&context.compiler_wrapper, "TinyCC wrapper")?;
    let compiler = format!("{bash} {wrapper}");
    let musl_include = shell_path(&request.musl_root.join("include"), "native musl include")?;
    let musl_lib = shell_path(&request.musl_root.join("lib"), "native musl lib")?;
    let mut environment = std::collections::BTreeMap::new();
    environment.insert("PATH".to_string(), shell_path(&context.tools, "tool namespace")?);
    environment.insert("CONFIG_SHELL".to_string(), bash.clone());
    environment.insert("SHELL".to_string(), bash.clone());
    environment.insert("CC".to_string(), compiler.clone());
    environment.insert("CC_FOR_BUILD".to_string(), compiler.clone());
    environment.insert("CPP".to_string(), format!("{compiler} -E"));
    environment.insert(
        "AR".to_string(),
        format!("{bash} {}", shell_path(&context.archive_runner.script, "binutils ar runner")?),
    );
    environment.insert("RANLIB".to_string(), "true".to_string());
    environment.insert("LD".to_string(), "true".to_string());
    environment.insert("MAKEINFO".to_string(), "true".to_string());
    environment.insert("AWK".to_string(), shell_path(request.gawk, "Gawk")?);
    environment.insert("M4".to_string(), shell_path(request.m4, "M4")?);
    let cflags = format!("-I{musl_include} {BINUTILS_CFLAGS_FEATURES}");
    environment.insert("CFLAGS".to_string(), cflags.clone());
    environment.insert("CFLAGS_FOR_BUILD".to_string(), cflags);
    environment.insert("LDFLAGS".to_string(), format!("-static -L{musl_lib}"));
    append_configure_probe_environment(&mut environment, request, configure_class)?;
    environment.insert("MANTLE_STAGE_X_FILE".to_string(), shell_path(&context.tools.join("file"), "configure file")?);
    append_sed_bridge_environment(&mut environment, request, context)?;
    append_archive_runner_environment(&mut environment, request, context)?;
    assert!(environment.contains_key("CONFIG_SHELL"));
    assert!(context.configure_utility.is_file());
    assert_eq!(environment.get("MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS").map(String::as_str), Some(configure_class));
    Ok(environment)
}

fn append_configure_probe_environment(
    environment: &mut std::collections::BTreeMap<String, String>,
    request: &BinutilsConfigureProbeRequest<'_>,
    configure_class: &str,
) -> Result<(), StagexBinutilsError> {
    let probe_root = shell_path(request.scratch_dir, "configure probe root")?;
    environment
        .insert("MANTLE_BINUTILS_CONFIGURE_PROBE_DIR".to_string(), format!("{probe_root}/source/{configure_class}"));
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS".to_string(), configure_class.to_string());
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT".to_string(), format!("{probe_root}/preprocess.audit"));
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT".to_string(), format!("{probe_root}/preprocess.count"));
    assert!(CONFIGURE_CLASSES.contains(&configure_class));
    assert!(environment.contains_key("MANTLE_BINUTILS_CONFIGURE_PROBE_DIR"));
    Ok(())
}

fn append_archive_runner_environment(
    environment: &mut std::collections::BTreeMap<String, String>,
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<(), StagexBinutilsError> {
    let additions = archive_runner_environment(request, &context.archive_runner)?;
    for (name, value) in additions {
        if environment.insert(name.clone(), value).is_some() {
            return Err(StagexBinutilsError::Materialization(format!("binutils ar environment duplicates {name}")));
        }
    }
    assert!(environment.contains_key("MANTLE_STAGE_X_AR_AUDIT"));
    assert!(environment.contains_key("MANTLE_STAGE_X_AR_COUNT"));
    Ok(())
}

fn append_sed_bridge_environment(
    environment: &mut std::collections::BTreeMap<String, String>,
    request: &BinutilsConfigureProbeRequest<'_>,
    context: &ConfigureProbeContext,
) -> Result<(), StagexBinutilsError> {
    environment.insert("MANTLE_STAGE_X_SED_BRIDGE_BASH".to_string(), shell_path(request.bash, "sed bridge Bash")?);
    environment.insert(
        "MANTLE_STAGE_X_SED_BRIDGE_SCRIPT".to_string(),
        shell_path(&context.sed_bridge.script, "sed bridge script")?,
    );
    environment.insert("MANTLE_STAGE_X_SED_BRIDGE_TARGET".to_string(), shell_path(request.sed, "protected sed")?);
    environment.insert(
        "MANTLE_STAGE_X_YLWRAP_SED".to_string(),
        shell_path(&context.sed_bridge.ylwrap_runner, "ylwrap sed runner")?,
    );
    for (name, tool) in [
        ("CAT", "cat"),
        ("HEAD", "head"),
        ("MKDIR", "mkdir"),
        ("RM", "rm"),
        ("RMDIR", "rmdir"),
        ("WC", "wc"),
    ] {
        environment.insert(
            format!("MANTLE_STAGE_X_SED_BRIDGE_{name}"),
            shell_path(&request.coreutils_bin.join(tool), "sed bridge coreutils tool")?,
        );
    }
    environment.insert(
        "MANTLE_STAGE_X_SED_BRIDGE_EMIT".to_string(),
        shell_path(&context.tools.join("emit"), "sed bridge emitter")?,
    );
    environment.insert(
        "MANTLE_STAGE_X_SED_SPOOL_ROOT".to_string(),
        shell_path(&context.sed_bridge.spool_root, "sed bridge spool root")?,
    );
    environment.insert(
        "MANTLE_STAGE_X_SED_BRIDGE_AUDIT".to_string(),
        shell_path(&context.sed_bridge.audit, "sed bridge audit")?,
    );
    assert!(environment.contains_key("MANTLE_STAGE_X_SED_BRIDGE_TARGET"));
    assert!(environment.contains_key("MANTLE_STAGE_X_YLWRAP_SED"));
    assert!(environment.contains_key("MANTLE_STAGE_X_SED_BRIDGE_EMIT"));
    assert!(environment.contains_key("MANTLE_STAGE_X_SED_BRIDGE_AUDIT"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const SOURCE_MANIFEST_BLAKE3: &str = "541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab";
    const OBSERVATION_FIXTURE_EVENT_COUNT: usize = 2;

    struct ProbeEnvironment {
        source_root: PathBuf,
        scratch_dir: PathBuf,
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

    impl ProbeEnvironment {
        fn from_environment() -> Self {
            let path = |name: &str| PathBuf::from(std::env::var(name).unwrap());
            Self {
                source_root: path("MANTLE_STAGE_X_BINUTILS_RETAINED_SOURCE"),
                scratch_dir: path("MANTLE_STAGE_X_BINUTILS_CONFIGURE_SCRATCH"),
                bash: path("MANTLE_STAGE_X_BASH_FULL"),
                tcc: path("MANTLE_STAGE_X_TCC_MUSL_V2"),
                musl_root: path("MANTLE_STAGE_X_MUSL_NATIVE_ROOT"),
                coreutils_bin: path("MANTLE_STAGE_X_COREUTILS_BIN"),
                sed: path("MANTLE_STAGE_X_SED"),
                grep: path("MANTLE_STAGE_X_GREP"),
                diff: path("MANTLE_STAGE_X_DIFF"),
                cmp: path("MANTLE_STAGE_X_CMP"),
                gawk: path("MANTLE_STAGE_X_GAWK"),
                m4: path("MANTLE_STAGE_X_M4"),
                bison: path("MANTLE_STAGE_X_BISON"),
                flex: path("MANTLE_STAGE_X_FLEX"),
                make: path("MANTLE_STAGE_X_MAKE"),
            }
        }

        fn request(&self) -> BinutilsConfigureProbeRequest<'_> {
            BinutilsConfigureProbeRequest {
                source_root: &self.source_root,
                scratch_dir: &self.scratch_dir,
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

    fn observed_event(tracee_path: &str, resolved_path: &str) -> crate::protected_exec::ProtectedSeccompAuditEvent {
        crate::protected_exec::ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from(resolved_path),
            tracee_path: PathBuf::from(tracee_path),
            resolved_host_path: PathBuf::from(resolved_path),
            digest_hex: "a".repeat(BLAKE3_HEX_CHAR_COUNT),
            reason: "diagnostic observation".to_string(),
            phase: "diagnostic".to_string(),
            inventory_entry_id: None,
            policy_decision: "diagnostic-observed".to_string(),
        }
    }

    #[test]
    fn make_shell_assignment_binds_one_absolute_shell() {
        let environment = std::collections::BTreeMap::from([("SHELL".to_string(), "/stagex/bash-full".to_string())]);
        let assignment = make_shell_assignment(&environment).unwrap();
        assert_eq!(assignment, "SHELL=/stagex/bash-full");
        assert!(!assignment.contains('\n'));
    }

    #[test]
    fn make_shell_assignment_rejects_missing_relative_and_multiline_authority() {
        let missing = std::collections::BTreeMap::new();
        let relative = std::collections::BTreeMap::from([("SHELL".to_string(), "bash".to_string())]);
        let multiline = std::collections::BTreeMap::from([("SHELL".to_string(), "/stagex/bash\nambient".to_string())]);
        assert!(make_shell_assignment(&missing).unwrap_err().to_string().contains("missing"));
        assert!(make_shell_assignment(&relative).unwrap_err().to_string().contains("absolute"));
        assert!(make_shell_assignment(&multiline).unwrap_err().to_string().contains("absolute"));
    }

    #[test]
    fn canonicalizes_diagnostic_exec_observations_without_authority() {
        let events = vec![
            observed_event("/tools/cc", "/protected/tcc"),
            observed_event("/tools/tcc", "/protected/tcc"),
        ];
        let report = binutils_exec_observation_report(&events, None).unwrap();
        let expected_event_count = u32::try_from(OBSERVATION_FIXTURE_EVENT_COUNT).unwrap();
        assert_eq!(report.event_count, expected_event_count);
        assert_eq!(report.observed_event_count, expected_event_count);
        assert_eq!(report.unique_executables.len(), 1);
        assert_eq!(report.unique_executables[0].tracee_paths.len(), OBSERVATION_FIXTURE_EVENT_COUNT);
        assert_eq!(report.unique_executables[0].event_count, u32::try_from(OBSERVATION_FIXTURE_EVENT_COUNT).unwrap());
        assert!(report.denied_events.is_empty());
        assert!(report.claim_boundary.contains("grants no protected execution authority"));
    }

    #[test]
    fn rejects_invalid_diagnostic_exec_observation_identity() {
        let mut wrong_phase = observed_event("/tools/cc", "/protected/tcc");
        wrong_phase.phase = "protected".to_string();
        let mut bad_digest = observed_event("/tools/cc", "/protected/tcc");
        bad_digest.digest_hex = "Z".repeat(BLAKE3_HEX_CHAR_COUNT);
        let phase_error = binutils_exec_observation_report(&[wrong_phase], None).unwrap_err();
        let digest_error = binutils_exec_observation_report(&[bad_digest], None).unwrap_err();
        assert!(phase_error.to_string().contains("invalid diagnostic exec event"));
        assert!(digest_error.to_string().contains("invalid diagnostic exec event"));
        assert!(!phase_error.to_string().contains("panicked"));
        assert!(!digest_error.to_string().contains("panicked"));
    }

    #[test]
    fn validates_exact_binutils_source_record_and_recipe() {
        let bundle = match std::env::var(SOURCE_BUNDLE_ENV) {
            Ok(value) => value,
            Err(_) => return,
        };
        let manifest = read_source_bundle(Path::new(&bundle)).unwrap();
        let record = find_source_record(&manifest.records).unwrap();
        validate_source_record(record).unwrap();
        validate_recipe_digest().unwrap();
        validate_sed_bridge_sources().unwrap();
        assert_eq!(record.content_blake3, BINUTILS_SOURCE_CONTENT_BLAKE3);
        assert_eq!(source_artifact_digests().len(), BINUTILS_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_binutils_source_authority() {
        let bundle = match std::env::var(SOURCE_BUNDLE_ENV) {
            Ok(value) => value,
            Err(_) => return,
        };
        let manifest = read_source_bundle(Path::new(&bundle)).unwrap();
        let mut record = find_source_record(&manifest.records).unwrap().clone();
        record.identity = "fixed-url-substituted".to_string();
        let error = validate_source_record(&record).unwrap_err().to_string();
        assert!(error.contains("source record identity"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn validates_bounded_elf_symbol_canonicalizer_source() {
        let source = std::str::from_utf8(ELF_SYMBOL_CANONICALIZER_SOURCE).unwrap();
        require_sed_bridge_source_digest(
            "ELF symbol canonicalizer",
            ELF_SYMBOL_CANONICALIZER_SOURCE,
            ELF_SYMBOL_CANONICALIZER_SOURCE_BLAKE3,
        )
        .unwrap();
        assert!(source.contains("SHT_SYMTAB"));
        assert!(source.contains("STB_LOCAL"));
        assert!(source.contains("O_EXCL"));
        assert!(source.contains("rename(staged_path, path)"));
        assert!(!source.contains("system("));
        assert!(!source.contains("popen("));
    }

    #[test]
    fn rejects_substituted_elf_symbol_canonicalizer_source() {
        let wrong_digest = "0".repeat(BLAKE3_HEX_CHAR_COUNT);
        let error = require_sed_bridge_source_digest(
            "ELF symbol canonicalizer",
            ELF_SYMBOL_CANONICALIZER_SOURCE,
            &wrong_digest,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("source BLAKE3 mismatch"));
        assert!(!error.contains("accepted"));
    }

    #[test]
    fn rejects_substituted_sed_bridge_source() {
        let wrong_digest = "0".repeat(BLAKE3_HEX_CHAR_COUNT);
        let error = require_sed_bridge_source_digest("launcher", SED_BRIDGE_LAUNCHER_SOURCE, &wrong_digest)
            .unwrap_err()
            .to_string();
        assert!(error.contains("source BLAKE3 mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn canonicalizes_complete_sed_bridge_audit() {
        let unordered = "2\t17\t19\t0\tok:protected-sed\n1\t4\t0\t1\trejected:mixed-input-authority\n";
        let canonical = canonical_sed_bridge_audit(unordered, 2).unwrap();
        assert!(canonical.starts_with("17\t19\t0\tok:protected-sed\n"));
        assert!(canonical.ends_with("4\t0\t1\trejected:mixed-input-authority\n"));
    }

    #[test]
    fn rejects_incomplete_sed_bridge_audit() {
        let error = canonical_sed_bridge_audit("2\t17\t19\t0\tok:protected-sed\n", 2).unwrap_err().to_string();
        assert!(error.contains("audit count mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn plans_exact_triplet_tool_links_without_io() {
        let root = Path::new("/retained/install");
        let plan = triplet_tool_link_plan(root);
        assert_eq!(plan.len(), BINUTILS_REQUIRED_TOOL_COUNT);
        assert_eq!(plan[0].1, root.join("bin/x86_64-unknown-linux-gnu-as"));
        assert_eq!(plan[0].2, Path::new(BINUTILS_INSTALL_PREFIX).join("bin/as"));
        assert!(plan.iter().all(|(_, link, target)| link != target));
    }

    #[test]
    fn validates_installed_tool_facts_without_io() {
        const PREDECESSOR: &[u8] = b"/protected/tcc";
        const ELF_FIXTURE: &[u8] = b"\x7fELFfixture";
        const SUBSTITUTED_EXPECTED_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
        let digest = blake3::hash(ELF_FIXTURE).to_hex().to_string();
        validate_installed_tool_facts("fixture", ELF_FIXTURE, true, EXECUTABLE_FILE_MODE, PREDECESSOR, &digest)
            .unwrap();
        let digest_error = validate_installed_tool_facts(
            "fixture",
            ELF_FIXTURE,
            true,
            EXECUTABLE_FILE_MODE,
            PREDECESSOR,
            SUBSTITUTED_EXPECTED_DIGEST,
        )
        .unwrap_err()
        .to_string();
        assert!(digest_error.contains("digest mismatch"));
        let delegating = [ELF_MAGIC, PREDECESSOR].concat();
        let delegating_digest = blake3::hash(&delegating).to_hex().to_string();
        let delegate_error = validate_installed_tool_facts(
            "fixture",
            &delegating,
            true,
            EXECUTABLE_FILE_MODE,
            PREDECESSOR,
            &delegating_digest,
        )
        .unwrap_err()
        .to_string();
        assert!(delegate_error.contains("delegates to predecessor TinyCC"));
        assert!(
            validate_installed_tool_facts("fixture", ELF_FIXTURE, true, EXECUTABLE_FILE_MODE, b"", &digest).is_err()
        );
    }

    #[test]
    fn rejects_substituted_component_output() {
        const SUBSTITUTED_EXPECTED_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("component");
        crate::stagex_mes_lib::write_create_new(&output, b"not-an-archive").unwrap();
        let type_error = validate_component_output(&output, "fixture", false, SUBSTITUTED_EXPECTED_DIGEST)
            .unwrap_err()
            .to_string();
        assert!(type_error.contains("lacks an archive output"));
        crate::stagex_mes_lib::write_create_new(&temp.path().join("archive"), b"!<arch>\n").unwrap();
        let digest_error =
            validate_component_output(&temp.path().join("archive"), "fixture", false, SUBSTITUTED_EXPECTED_DIGEST)
                .unwrap_err()
                .to_string();
        assert!(digest_error.contains("digest mismatch"));
    }

    #[test]
    fn accepts_bounded_full_build_and_install_sed_counts() {
        for count in BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS {
            validate_sed_bridge_count_value(count, &BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS).unwrap();
        }
        for count in BINUTILS_INSTALL_SED_INVOCATION_COUNTS {
            validate_sed_bridge_count_value(count, &BINUTILS_INSTALL_SED_INVOCATION_COUNTS).unwrap();
        }
        assert!(BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS.iter().all(|count| *count > 0));
        assert!(BINUTILS_INSTALL_SED_INVOCATION_COUNTS.iter().all(|count| *count > 0));
    }

    #[test]
    fn rejects_unobserved_full_build_and_install_sed_counts() {
        const UNOBSERVED_FULL_SED_COUNTS: [u32; 2] = [4_770, 4_774];
        const UNOBSERVED_INSTALL_SED_COUNTS: [u32; 2] = [4_890, 4_894];
        for count in UNOBSERVED_FULL_SED_COUNTS {
            let error = validate_sed_bridge_count_value(count, &BINUTILS_FULL_BUILD_SED_INVOCATION_COUNTS)
                .unwrap_err()
                .to_string();
            assert!(error.contains("expected one of [4771, 4772, 4773]"));
        }
        for count in UNOBSERVED_INSTALL_SED_COUNTS {
            assert!(validate_sed_bridge_count_value(count, &BINUTILS_INSTALL_SED_INVOCATION_COUNTS).is_err());
        }
        assert!(validate_sed_bridge_count_value(1, &[]).is_err());
    }

    #[test]
    fn accepts_exact_archive_runner_audit() {
        let audit = concat!(
            "1\t66\t704408\t./libiberty.a\tok\n",
            "2\t15\t164042\tlibz.a\tok\n",
            "3\t59\t2383682\t.libs/libbfd.a\tok\n",
            "4\t5\t2160260\t.libs/libopcodes.a\tok\n",
        );
        let entries = parse_archive_runner_audit(audit).unwrap();
        assert_eq!(entries.len(), BINUTILS_AR_EXPECTED_INVOCATION_COUNT);
        assert!(entries.iter().all(|entry| entry.output_bytes > 0));
    }

    #[test]
    fn rejects_substituted_archive_runner_audit() {
        let audit = concat!(
            "1\t66\t704408\t./libiberty.a\tok\n",
            "2\t15\t164042\tlibz.a\tok\n",
            "3\t59\t2383682\t.libs/libbfd.a\tok\n",
            "4\t5\t2160260\t../escaped.a\tok\n",
        );
        let error = parse_archive_runner_audit(audit).unwrap_err().to_string();
        assert!(error.contains("substituted binutils ar audit facts"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn accepts_exact_ylwrap_sed_audit_suffix() {
        let canonical = (0..YLWRAP_SED_EXPECTED_INVOCATION_COUNT)
            .map(|index| format!("0\t{}\t1\tok:ylwrap-sed\n", index + 1))
            .collect::<String>();
        let count = validate_ylwrap_sed_canonical(&canonical).unwrap();
        assert_eq!(count, YLWRAP_SED_EXPECTED_INVOCATION_COUNT);
        assert!(canonical.ends_with("ok:ylwrap-sed\n"));
    }

    #[test]
    fn rejects_incomplete_ylwrap_sed_audit_suffix() {
        let incomplete_count = YLWRAP_SED_EXPECTED_INVOCATION_COUNT - 1;
        let canonical = (0..incomplete_count)
            .map(|index| format!("0\t{}\t1\tok:ylwrap-sed\n", index + 1))
            .collect::<String>();
        let error = validate_ylwrap_sed_canonical(&canonical).unwrap_err().to_string();
        assert!(error.contains("invocation count mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_unbounded_ambient_file_relocation() {
        let error = relocate_configure_file_text("bfd", "case `/usr/bin/file conftest.o`").unwrap_err().to_string();
        assert!(error.contains("occurrence mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn disables_exact_legacy_absolute_host_probe_block() {
        let original = format!("before\n{LEGACY_ABSOLUTE_HOST_PROBE_BLOCK}\nafter\n");
        let rewritten = disable_legacy_absolute_host_probe_text("bfd", &original).unwrap();
        assert!(!rewritten.contains("`(/usr/bin/uname -p)"));
        assert!(!rewritten.contains("`(/bin/universe)"));
        assert!(rewritten.contains(DISABLED_ABSOLUTE_HOST_PROBE_BLOCK));
        assert!(rewritten.starts_with("before\n"));
        assert!(rewritten.ends_with("after\n"));
    }

    #[test]
    fn rejects_missing_or_duplicate_legacy_absolute_host_probe_blocks() {
        let missing = disable_legacy_absolute_host_probe_text("bfd", "no legacy probe block").unwrap_err();
        let duplicate_text = format!("{LEGACY_ABSOLUTE_HOST_PROBE_BLOCK}\n{LEGACY_ABSOLUTE_HOST_PROBE_BLOCK}");
        let duplicate = disable_legacy_absolute_host_probe_text("bfd", &duplicate_text).unwrap_err();
        assert!(missing.to_string().contains("expected 1, observed 0"));
        assert!(duplicate.to_string().contains("expected 1, observed 2"));
        assert!(!missing.to_string().contains("panicked"));
        assert!(!duplicate.to_string().contains("panicked"));
    }

    #[test]
    fn makes_generated_configure_exec_paths_absolute() {
        let bfd_original = format!(
            "{CONFIGURE_DYNAMIC_EXEC_OLD}\n{CONFIGURE_CONFTEST_EXEC_OLD}\n{}",
            CONFIGURE_LIBTOOL_EXEC_OLD.repeat(CONFIGURE_LIBTOOL_EXEC_COUNT_PER_CLASS)
        );
        let bfd = make_configure_exec_paths_absolute("bfd", &bfd_original).unwrap();
        let intl_original = format!("{CONFIGURE_DYNAMIC_EXEC_OLD}\n{CONFIGURE_CONFTEST_EXEC_OLD}\n");
        let intl = make_configure_exec_paths_absolute("intl", &intl_original).unwrap();
        assert!(bfd.contains(CONFIGURE_DYNAMIC_EXEC_NEW));
        assert!(bfd.contains(CONFIGURE_CONFTEST_EXEC_NEW));
        assert_eq!(bfd.matches(CONFIGURE_LIBTOOL_EXEC_NEW).count(), CONFIGURE_LIBTOOL_EXEC_COUNT_PER_CLASS);
        assert!(!bfd.contains("(./conftest"));
        assert!(!intl.contains("ac_try='./"));
    }

    #[test]
    fn makes_opcodes_generator_exec_path_absolute() {
        let original = format!("target:\n{OPCODES_I386_GEN_RELATIVE_EXEC}\n");
        let rewritten = make_opcodes_generator_exec_path_absolute("opcodes fixture", &original).unwrap();
        assert!(rewritten.contains(OPCODES_I386_GEN_ABSOLUTE_EXEC));
        assert!(!rewritten.contains(OPCODES_I386_GEN_RELATIVE_EXEC));
    }

    #[test]
    fn makes_component_generator_exec_paths_absolute() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path();
        fs::create_dir(source.join("binutils")).unwrap();
        fs::create_dir(source.join("bfd")).unwrap();
        let sysinfo = format!("{BINUTILS_SYSINFO_RELATIVE_EXEC}\n").repeat(BINUTILS_SYSINFO_EXEC_OCCURRENCE_COUNT);
        for relative in ["binutils/Makefile.am", "binutils/Makefile.in"] {
            fs::write(source.join(relative), &sysinfo).unwrap();
        }
        for relative in ["bfd/Makefile.am", "bfd/Makefile.in"] {
            fs::write(source.join(relative), format!("{BFD_GEN_AOUT_RELATIVE_EXEC}\n")).unwrap();
        }
        make_component_generator_exec_paths_absolute(source).unwrap();
        assert_eq!(
            fs::read_to_string(source.join("binutils/Makefile.in"))
                .unwrap()
                .matches(BINUTILS_SYSINFO_ABSOLUTE_EXEC)
                .count(),
            BINUTILS_SYSINFO_EXEC_OCCURRENCE_COUNT
        );
        assert!(fs::read_to_string(source.join("bfd/Makefile.in")).unwrap().contains(BFD_GEN_AOUT_ABSOLUTE_EXEC));
    }

    #[test]
    fn rejects_component_generator_exec_path_drift_before_writing() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path();
        fs::create_dir(source.join("binutils")).unwrap();
        fs::create_dir(source.join("bfd")).unwrap();
        let wrong_sysinfo = format!("{BINUTILS_SYSINFO_RELATIVE_EXEC}\n");
        for relative in ["binutils/Makefile.am", "binutils/Makefile.in"] {
            fs::write(source.join(relative), &wrong_sysinfo).unwrap();
        }
        for relative in ["bfd/Makefile.am", "bfd/Makefile.in"] {
            fs::write(source.join(relative), format!("{BFD_GEN_AOUT_RELATIVE_EXEC}\n")).unwrap();
        }
        let error = make_component_generator_exec_paths_absolute(source).unwrap_err();
        assert!(error.to_string().contains("expected 4, observed 1"));
        assert_eq!(fs::read_to_string(source.join("binutils/Makefile.am")).unwrap(), wrong_sysinfo);
    }

    #[test]
    fn rejects_opcodes_generator_exec_path_drift() {
        let missing = "target:\n\t./different-generator --srcdir $(srcdir)\n";
        let duplicate = format!("{OPCODES_I386_GEN_RELATIVE_EXEC}\n{OPCODES_I386_GEN_RELATIVE_EXEC}\n");
        let missing_error = make_opcodes_generator_exec_path_absolute("opcodes missing", missing).unwrap_err();
        let duplicate_error = make_opcodes_generator_exec_path_absolute("opcodes duplicate", &duplicate).unwrap_err();
        assert!(missing_error.to_string().contains("expected 1, observed 0"));
        assert!(duplicate_error.to_string().contains("expected 1, observed 2"));
    }

    #[test]
    fn rejects_generated_configure_exec_path_drift() {
        let missing_dynamic = format!("{CONFIGURE_CONFTEST_EXEC_OLD}\n");
        let wrong_libtool_count =
            format!("{CONFIGURE_DYNAMIC_EXEC_OLD}\n{CONFIGURE_CONFTEST_EXEC_OLD}\n{CONFIGURE_LIBTOOL_EXEC_OLD}");
        let dynamic_error = make_configure_exec_paths_absolute("intl", &missing_dynamic).unwrap_err();
        let libtool_error = make_configure_exec_paths_absolute("bfd", &wrong_libtool_count).unwrap_err();
        assert!(dynamic_error.to_string().contains("expected 1, observed 0"));
        assert!(libtool_error.to_string().contains("expected 2, observed 1"));
        assert!(!dynamic_error.to_string().contains("panicked"));
        assert!(!libtool_error.to_string().contains("panicked"));
    }

    #[test]
    fn repairs_exact_recipe_source_text() {
        let original = format!("before{BFD_BOOTSTRAP_CONFIG_REFERENCE}middle{BFD_BOOTSTRAP_CONFIG_REFERENCE}after");
        let repaired = replace_exact_text(
            "BFD test",
            &original,
            BFD_BOOTSTRAP_CONFIG_REFERENCE,
            "",
            BFD_BOOTSTRAP_CONFIG_REFERENCE_COUNT,
        )
        .unwrap();
        assert_eq!(repaired, "beforemiddleafter");
        assert!(!repaired.contains(BFD_BOOTSTRAP_CONFIG_REFERENCE));
    }

    #[test]
    fn rejects_recipe_source_transform_drift() {
        let error = replace_exact_text(
            "BFD test",
            BFD_BOOTSTRAP_CONFIG_REFERENCE,
            BFD_BOOTSTRAP_CONFIG_REFERENCE,
            "",
            BFD_BOOTSTRAP_CONFIG_REFERENCE_COUNT,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("replacement occurrence mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_duplicate_configure_class() {
        let error = validate_configure_classes(&["intl", "intl"]).unwrap_err().to_string();
        assert!(error.contains("duplicate configure class"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_existing_source_scratch_before_bundle_access() {
        let temp = tempfile::tempdir().unwrap();
        let missing_bundle = temp.path().join("missing-bundle.json");
        let error = materialize_authenticated_binutils_source(&missing_bundle, SOURCE_MANIFEST_BLAKE3, temp.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("source scratch"));
        assert!(!error.contains("source bundle must"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_binutils_source() {
        let bundle = std::env::var(SOURCE_BUNDLE_ENV).unwrap();
        let scratch = std::env::var("MANTLE_STAGE_X_BINUTILS_SOURCE_SCRATCH").unwrap();
        let report =
            materialize_authenticated_binutils_source(Path::new(&bundle), SOURCE_MANIFEST_BLAKE3, Path::new(&scratch))
                .unwrap();
        assert_eq!(report.record_content_blake3, BINUTILS_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("bfd/configure").is_file());
    }

    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn configures_intl_with_regular_file_sed_bridge() {
        let paths = ProbeEnvironment::from_environment();
        let exit_code = probe_authenticated_intl_configure(paths.request()).unwrap();
        let stderr = fs::read_to_string(paths.scratch_dir.join("configure.stderr.txt")).unwrap();
        let invocation_count = fs::read_to_string(paths.scratch_dir.join("sed-spool/invocation.count")).unwrap();
        assert_eq!(exit_code, 0);
        assert!(!stderr.contains("config.status: error"));
        assert!(invocation_count.trim().parse::<u32>().unwrap() > 1);
        assert!(paths.scratch_dir.join("source/intl/config.status").is_file());
        assert!(paths.scratch_dir.join("source/intl/Makefile").is_file());
    }

    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn configures_declared_matrix_with_regular_file_sed_bridge() {
        let paths = ProbeEnvironment::from_environment();
        let outcomes = probe_authenticated_configure_matrix(paths.request()).unwrap();
        assert_eq!(outcomes.len(), CONFIGURE_CLASS_COUNT);
        assert!(outcomes.iter().all(|outcome| outcome.exit_code == 0));
        assert!(
            CONFIGURE_CLASSES.iter().all(|class| paths
                .scratch_dir
                .join("source")
                .join(class)
                .join("config.status")
                .is_file())
        );
        assert!(
            CONFIGURE_CLASSES.iter().all(|class| paths
                .scratch_dir
                .join("source")
                .join(class)
                .join("Makefile")
                .is_file())
        );
        assert!(FILE_RELOCATION_CLASSES.iter().all(|class| {
            !fs::read_to_string(paths.scratch_dir.join("source").join(class).join("configure"))
                .unwrap()
                .contains(AMBIENT_FILE_PATH)
        }));
        for outcome in &outcomes {
            let stderr = fs::read_to_string(&outcome.stderr_path).unwrap();
            assert!(!stderr.contains("command not found"));
            assert!(!stderr.contains("No such file"));
            assert!(!stderr.contains(AMBIENT_FILE_PATH));
            assert!(!stderr.contains("Broken pipe"));
        }
    }

    #[test]
    #[ignore = "requires a retained authenticated binutils install root"]
    fn smokes_authenticated_installed_tools() {
        let paths = ProbeEnvironment::from_environment();
        let install_root = PathBuf::from(
            std::env::var("MANTLE_STAGE_X_BINUTILS_INSTALL_ROOT")
                .expect("MANTLE_STAGE_X_BINUTILS_INSTALL_ROOT must name the retained install root"),
        );
        fs::create_dir(&paths.scratch_dir).unwrap();
        validate_installed_tools(&paths.request(), &install_root).unwrap();
        for (_, link, target) in triplet_tool_link_plan(&install_root) {
            assert_eq!(fs::read_link(link).unwrap(), target);
        }
        run_installed_tool_smokes(&paths.request(), &install_root).unwrap();
        assert!(paths.scratch_dir.join("binutils-runtime-smoke/positive").is_file());
        assert!(!paths.scratch_dir.join("binutils-runtime-smoke/rejected-bin").exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn observes_authenticated_binutils_exec_inventory() {
        let paths = ProbeEnvironment::from_environment();
        let request = paths.request();
        let allowed_paths = diagnostic_exec_observation_paths(&request);
        let observer =
            crate::protected_exec_seccomp::install_current_thread_diagnostic_exec_observer(allowed_paths).unwrap();
        let outcome = probe_authenticated_component_builds_observing_identities(request);
        let reaped_count = crate::protected_exec_seccomp::reap_adopted_exec_descendants().unwrap();
        let quiescent_event_count = observer.wait_for_audit_quiescence().unwrap();
        let events = observer.audit_events();
        assert_eq!(quiescent_event_count, events.len());
        let execution_error = outcome.as_ref().err().map(ToString::to_string);
        let report = binutils_exec_observation_report(&events, execution_error).unwrap();
        let report_path = PathBuf::from(
            std::env::var("MANTLE_STAGE_X_BINUTILS_EXEC_OBSERVATION_REPORT")
                .expect("MANTLE_STAGE_X_BINUTILS_EXEC_OBSERVATION_REPORT must name an absent report path"),
        );
        let bytes = serde_json::to_vec_pretty(&report).unwrap();
        crate::stagex_mes_lib::write_create_new(&report_path, &bytes).unwrap();
        eprintln!(
            "binutils diagnostic exec observation: events={} unique={} denied={} reaped={} report={}",
            report.event_count,
            report.unique_executables.len(),
            report.denied_event_count,
            reaped_count,
            report_path.display()
        );
        assert!(outcome.is_ok(), "binutils execution failed; inspect {}", report_path.display());
        assert_eq!(report.denied_event_count, 0);
        let event_count_bounds = BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS.map(|count| u32::try_from(count).unwrap());
        assert!(report.event_count >= event_count_bounds[0]);
        assert!(report.event_count <= event_count_bounds[1]);
        assert_eq!(report.unique_executables.len(), BINUTILS_PROTECTED_EXEC_UNIQUE_IDENTITY_COUNT);
    }

    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn builds_authenticated_binutils_components() {
        let paths = ProbeEnvironment::from_environment();
        let outputs = probe_authenticated_component_builds(paths.request()).unwrap();
        assert_eq!(outputs.len(), BINUTILS_COMPONENT_COUNT);
        assert!(outputs.iter().all(|output| output.is_file()));
        assert!(paths.scratch_dir.join("source/bfd/.libs/libbfd.a").is_file());
        assert!(paths.scratch_dir.join("source/ld/ld-new").is_file());
    }

    #[test]
    #[ignore = "derives canonical identities from retained binutils source and protected tool roots"]
    fn builds_authenticated_binutils_components_observing_identities() {
        let paths = ProbeEnvironment::from_environment();
        let outputs = probe_authenticated_component_builds_observing_identities(paths.request()).unwrap();
        assert_eq!(outputs.len(), BINUTILS_COMPONENT_COUNT);
        assert!(outputs.iter().all(|output| output.is_file()));
        assert!(paths.scratch_dir.join("source/bfd/.libs/libbfd.a").is_file());
        assert!(paths.scratch_dir.join("install-destdir").is_dir());
    }

    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn generates_initial_sources_and_runs_second_bfd_configure() {
        let paths = ProbeEnvironment::from_environment();
        let second_bfd = probe_authenticated_generated_sources(paths.request()).unwrap();
        let source = paths.scratch_dir.join("source");
        assert_eq!(second_bfd.exit_code, 0);
        assert!(source.join("bfd/doc/chew").is_file());
        assert!(source.join("bfd/bfd-in3.h").is_file());
        assert!(source.join("intl/plural.c").is_file());
        assert!(source.join("binutils/arparse.c").is_file());
        assert!(source.join("binutils/arlex.c").is_file());
        assert!(source.join("ld/ldgram.c").is_file());
        assert!(source.join("ld/ldlex.c").is_file());
        assert!(paths.scratch_dir.join("make-bfd-headers.stdout.txt").is_file());
        assert!(paths.scratch_dir.join("make-intl-plural.stdout.txt").is_file());
        assert!(paths.scratch_dir.join("make-binutils-generated.stdout.txt").is_file());
        assert!(paths.scratch_dir.join("make-ld-generated.stdout.txt").is_file());
        let audit = fs::read_to_string(paths.scratch_dir.join("sed-spool/audit.canonical.tsv")).unwrap();
        assert!(audit.contains("ok:ylwrap-sed"));
    }
}
