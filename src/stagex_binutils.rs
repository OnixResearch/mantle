use std::fs;
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
const SED_BRIDGE_LAUNCHER_SOURCE_ARTIFACT_ID: &str = "stagex-sed-bridge-launcher-source";
const SED_BRIDGE_LAUNCHER_SOURCE_NAME: &str = "stagex-sed-bridge-launcher.c";
const SED_BRIDGE_LAUNCHER_OUTPUT_NAME: &str = "stagex-sed-bridge-launcher";
const SINGLE_THREAD_SEMAPHORE_SOURCE_NAME: &str = "stagex-single-thread-semaphore-compat.c";
const SED_BRIDGE_SCRIPT_SOURCE_ARTIFACT_ID: &str = "stagex-sed-regular-file-bridge-source";
const SINGLE_THREAD_SEMAPHORE_SOURCE_ARTIFACT_ID: &str = "stagex-single-thread-semaphore-compat-source";
const CONFIGURE_UTILITY_SOURCE_ARTIFACT_ID: &str = "stagex-configure-utility-source";
const SED_BRIDGE_LAUNCHER_SOURCE_BLAKE3: &str = "606c52085de42d0221ba5490e81d8c539a50a65aaa89f7b4b478371bdc643dab";
const SED_BRIDGE_LAUNCHER_BLAKE3: &str = "9b4d6a5eca05f55c407a70f7e426f756f482c9ae8f76d0a22d1b1b46e7dba1a1";
const SED_BRIDGE_SCRIPT_SOURCE_BLAKE3: &str = "5e7f2c7575a6e9de292737c0dc3c25174d6284e967b2a43ba26472a050f8b0d0";
const SINGLE_THREAD_SEMAPHORE_SOURCE_BLAKE3: &str = "52c3ec19c484b0b4c3c5de84fc7ae77f40601fc81993d16ef6e15eb7401ee084";
const CONFIGURE_UTILITY_SOURCE_BLAKE3: &str = "b150327f4ef9256764e8024466dd706ee87012f70554f1c7a01a0d8bc08975b1";
const CONFIGURE_UTILITY_BLAKE3: &str = "a0d4f306ed84086cb0cebff1dffb0f5fea0a93e9ee4085e6e5e0acc3e4df201f";
const BINUTILS_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-binutils-2.30-source-materialization-v1";
const BINUTILS_SOURCE_NON_CLAIM: &str =
    "binutils source materialization proves authenticated offline archive identity and checked-recipe identity only";
const BINUTILS_RECORD_HASH: &str = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=";
const BINUTILS_RECORD_URL: &str = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz";
const BINUTILS_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const BINUTILS_RECORD_UNPACK: &str = "1";
const BINUTILS_SOURCE_ARTIFACT_COUNT: usize = 6;
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

#[derive(Debug, Clone)]
struct ConfigureProbeContext {
    source: PathBuf,
    tools: PathBuf,
    compiler_wrapper: PathBuf,
    configure_utility: PathBuf,
    sed_bridge: SedBridgePaths,
    stdin_path: PathBuf,
}

const COREUTILS_TOOL_NAMES: &[&str] = &[
    "basename", "cat", "chmod", "cp", "dirname", "echo", "expr", "false", "head", "install", "ln", "ls", "mkdir", "mv",
    "rm", "rmdir", "sort", "tail", "tee", "test", "touch", "tr", "true", "uniq", "wc",
];
const COREUTILS_TOOL_COUNT: usize = 25;
const CONFIGURE_OUTPUT_MEBIBYTES_MAX: u64 = 16;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const CONFIGURE_OUTPUT_BYTES_MAX: u64 = CONFIGURE_OUTPUT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const SED_BRIDGE_SMOKE_OUTPUT_BYTES_MAX: u64 = KIBIBYTE_BYTES;
const SED_BRIDGE_AUTHORITY_FAILURE: i32 = 125;
const SED_BRIDGE_INVOCATION_COUNT_MAX: u32 = 4_096;
const SED_BRIDGE_AUDIT_FIELD_COUNT: usize = 5;
const SED_BRIDGE_SMOKE_EXPECTED: &[u8] = b"S[\"LTLIBOBJS\"]=\"\"\n";
const SED_BRIDGE_SMOKE_INPUT: &[u8] = b"LTLIBOBJS!%!_!# \n";
const SED_BRIDGE_FIRST_PROGRAM: &[u8] = b"h\ns/^/S[\"/; s/!.*/\"]=/\np\ng\ns/^[^!]*!//\n:repl\nt repl\ns/%!_!# $//\nt delim\n:nl\nh\ns/\\(.\\{148\\}\\).*/\\1/\nt more1\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\\\\n\"\\\\/\np\nn\nb repl\n:more1\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"\\\\/\np\ng\ns/.\\{148\\}//\nt nl\n:delim\nh\ns/\\(.\\{148\\}\\).*/\\1/\nt more2\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"/\np\nb\n:more2\ns/[\"\\\\]/\\\\&/g; s/^/\"/; s/$/\"\\\\/\np\ng\ns/.\\{148\\}//\nt delim\n";
const SED_BRIDGE_SECOND_PROGRAM: &[u8] = b"/^[^\"\"]/ {\n  N\n  s/\\n//\n}\n";
const TARGET: &str = "x86_64-unknown-linux-gnu";
const BINUTILS_RUNTIME_START: &str = "      $BB cat > \"$WORK/binutils-runtime.c\" <<'RUNTIME_EOF'\n";
const BINUTILS_RUNTIME_END: &str = "\nRUNTIME_EOF";
const REGULAR_FILE_MODE: u32 = 0o644;
const EXECUTABLE_FILE_MODE: u32 = 0o755;
const NATIVE_HELPER_COMPILE_ARGUMENT_COUNT: usize = 6;
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
const CONFIGURE_UTILITY_SOURCE_NAME: &str = "stagex-configure-utility.c";
const CONFIGURE_UTILITY_OUTPUT_NAME: &str = "stagex-configure-utility";
const FILE_RELOCATION_CLASS_COUNT: usize = 7;
const FILE_RELOCATION_CLASSES: [&str; FILE_RELOCATION_CLASS_COUNT] =
    ["zlib", "bfd", "opcodes", "binutils", "gas", "gprof", "ld"];
const FILE_RELOCATION_OCCURRENCES_PER_CLASS: usize = 10;
const FILE_RELOCATION_OCCURRENCE_COUNT: usize = FILE_RELOCATION_CLASS_COUNT * FILE_RELOCATION_OCCURRENCES_PER_CLASS;
const AMBIENT_FILE_PATH: &str = "/usr/bin/file";
const DECLARED_FILE_PATH: &str = "$MANTLE_STAGE_X_FILE";

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

fn probe_authenticated_configures(
    request: BinutilsConfigureProbeRequest<'_>,
    configure_classes: &[&str],
    require_success: bool,
) -> Result<Vec<ConfigureProbeOutcome>, StagexBinutilsError> {
    validate_configure_classes(configure_classes)?;
    let context = prepare_configure_probe(&request)?;
    let smoke_environment = configure_environment(&request, &context, configure_classes[0])?;
    run_sed_bridge_smokes(&request, &context.sed_bridge, &smoke_environment)?;
    let mut outcomes = Vec::with_capacity(configure_classes.len());
    for configure_class in configure_classes {
        let outcome = run_configure_class(&request, &context, configure_class)?;
        if require_success && outcome.exit_code != 0 {
            return Err(StagexBinutilsError::Materialization(format!(
                "authenticated configure failed for {configure_class} with status {}",
                outcome.exit_code
            )));
        }
        outcomes.push(outcome);
    }
    finalize_sed_bridge_audit(&context.sed_bridge)?;
    assert_eq!(outcomes.len(), configure_classes.len());
    assert!(!outcomes.is_empty());
    Ok(outcomes)
}

fn prepare_configure_probe(
    request: &BinutilsConfigureProbeRequest<'_>,
) -> Result<ConfigureProbeContext, StagexBinutilsError> {
    validate_configure_probe_request(request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure probe root: {error}")))?;
    let source = request.scratch_dir.join("source");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source)
        .map_err(StagexBinutilsError::from_runtime)?;
    relocate_configure_file_utility(&source)?;
    let tools = request.scratch_dir.join("tools");
    fs::create_dir(&tools)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure tool namespace: {error}")))?;
    let compiler_wrapper = prepare_tcc_wrapper(request)?;
    let sed_bridge = prepare_sed_bridge(request, &compiler_wrapper)?;
    let configure_utility = prepare_configure_utility(request, &compiler_wrapper)?;
    populate_probe_tool_namespace(request, &tools, &sed_bridge.launcher, &configure_utility)?;
    run_configure_utility_smokes(request, &tools, &configure_utility)?;
    let stdin_path = request.scratch_dir.join("configure.stdin");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    assert!(source.is_dir());
    assert!(tools.is_dir());
    Ok(ConfigureProbeContext {
        source,
        tools,
        compiler_wrapper,
        configure_utility,
        sed_bridge,
        stdin_path,
    })
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
        "--prefix=/mantle/stagex/binutils-probe-output",
        "--libdir=/mantle/stagex/binutils-probe-output/lib",
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
    let wrapper = request.scratch_dir.join("tcc-bounded.sh");
    let script = tcc_wrapper_script(request, &runtime_object, &runtime_assembly_object)?;
    crate::stagex_mes_lib::write_create_new(&wrapper, script.as_bytes()).map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting compiler wrapper mode: {error}")))?;
    assert!(runtime_object.is_file());
    assert!(wrapper.is_file());
    Ok(wrapper)
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
    let spool_root = request.scratch_dir.join("sed-spool");
    crate::stagex_mes_lib::write_create_new(&launcher_source, SED_BRIDGE_LAUNCHER_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&script, SED_BRIDGE_SCRIPT_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    crate::stagex_mes_lib::write_create_new(&semaphore_source, SINGLE_THREAD_SEMAPHORE_SOURCE)
        .map_err(StagexBinutilsError::from_runtime)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(REGULAR_FILE_MODE))
        .map_err(|error| StagexBinutilsError::Materialization(format!("setting sed bridge script mode: {error}")))?;
    fs::create_dir(&spool_root)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating sed spool root: {error}")))?;
    compile_sed_bridge_launcher(request, compiler_wrapper, &launcher_source, &semaphore_source, &launcher)?;
    run_sed_bridge_launcher_self_test(request, &launcher)?;
    run_sed_bridge_launcher_missing_authority_test(request, &launcher)?;
    assert!(launcher.is_file());
    assert!(spool_root.is_dir());
    Ok(SedBridgePaths {
        launcher,
        script,
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
CHMOD='@CHMOD@'
CAT='@CAT@'
RM='@RM@'
WC='@WC@'
GREP='@GREP@'
SED='@SED@'
SOURCE_BYTES_MAX=65536
INVOCATION_COUNT_MAX=4096
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
"#;

fn tcc_wrapper_script(
    request: &BinutilsConfigureProbeRequest<'_>,
    runtime_object: &Path,
    runtime_assembly_object: &Path,
) -> Result<String, StagexBinutilsError> {
    let bindings = [
        ("@TCC@", shell_path(request.tcc, "TinyCC")?),
        ("@MUSL_INCLUDE@", shell_path(&request.musl_root.join("include"), "native musl include")?),
        ("@MUSL_LIB@", shell_path(&request.musl_root.join("lib"), "native musl lib")?),
        ("@RUNTIME@", shell_path(runtime_object, "runtime object")?),
        ("@RUNTIME_ASM@", shell_path(runtime_assembly_object, "runtime assembly object")?),
        ("@CHMOD@", shell_path(&request.coreutils_bin.join("chmod"), "chmod")?),
        ("@CAT@", shell_path(&request.coreutils_bin.join("cat"), "cat")?),
        ("@RM@", shell_path(&request.coreutils_bin.join("rm"), "rm")?),
        ("@WC@", shell_path(&request.coreutils_bin.join("wc"), "wc")?),
        ("@GREP@", shell_path(request.grep, "grep")?),
        ("@SED@", shell_path(request.sed, "sed")?),
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
    if status != "ok" && !status.starts_with("rejected:") {
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
    environment.insert("SHELL".to_string(), bash);
    environment.insert("CC".to_string(), compiler.clone());
    environment.insert("CC_FOR_BUILD".to_string(), compiler.clone());
    environment.insert("CPP".to_string(), format!("{compiler} -E"));
    environment.insert("AR".to_string(), "true".to_string());
    environment.insert("RANLIB".to_string(), "true".to_string());
    environment.insert("LD".to_string(), "true".to_string());
    environment.insert("MAKEINFO".to_string(), "true".to_string());
    environment.insert("AWK".to_string(), shell_path(request.gawk, "Gawk")?);
    environment.insert("M4".to_string(), shell_path(request.m4, "M4")?);
    environment.insert("CFLAGS".to_string(), format!("-I{musl_include} -static -D_GNU_SOURCE"));
    environment.insert("LDFLAGS".to_string(), format!("-static -L{musl_lib}"));
    append_configure_probe_environment(&mut environment, request, configure_class)?;
    environment.insert("MANTLE_STAGE_X_FILE".to_string(), shell_path(&context.tools.join("file"), "configure file")?);
    append_sed_bridge_environment(&mut environment, request, context)?;
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
    assert!(environment.contains_key("MANTLE_STAGE_X_SED_BRIDGE_EMIT"));
    assert!(environment.contains_key("MANTLE_STAGE_X_SED_BRIDGE_AUDIT"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const SOURCE_MANIFEST_BLAKE3: &str = "541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab";

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
        let unordered = "2\t17\t19\t0\tok\n1\t4\t0\t1\trejected:mixed-input-authority\n";
        let canonical = canonical_sed_bridge_audit(unordered, 2).unwrap();
        assert!(canonical.starts_with("17\t19\t0\tok\n"));
        assert!(canonical.ends_with("4\t0\t1\trejected:mixed-input-authority\n"));
    }

    #[test]
    fn rejects_incomplete_sed_bridge_audit() {
        let error = canonical_sed_bridge_audit("2\t17\t19\t0\tok\n", 2).unwrap_err().to_string();
        assert!(error.contains("audit count mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_unbounded_ambient_file_relocation() {
        let error = relocate_configure_file_text("bfd", "case `/usr/bin/file conftest.o`").unwrap_err().to_string();
        assert!(error.contains("occurrence mismatch"));
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
        let path = |name: &str| PathBuf::from(std::env::var(name).unwrap());
        let source_root = path("MANTLE_STAGE_X_BINUTILS_RETAINED_SOURCE");
        let scratch_dir = path("MANTLE_STAGE_X_BINUTILS_CONFIGURE_SCRATCH");
        let bash = path("MANTLE_STAGE_X_BASH_FULL");
        let tcc = path("MANTLE_STAGE_X_TCC_MUSL_V2");
        let musl_root = path("MANTLE_STAGE_X_MUSL_NATIVE_ROOT");
        let coreutils_bin = path("MANTLE_STAGE_X_COREUTILS_BIN");
        let sed = path("MANTLE_STAGE_X_SED");
        let grep = path("MANTLE_STAGE_X_GREP");
        let diff = path("MANTLE_STAGE_X_DIFF");
        let cmp = path("MANTLE_STAGE_X_CMP");
        let gawk = path("MANTLE_STAGE_X_GAWK");
        let m4 = path("MANTLE_STAGE_X_M4");
        let bison = path("MANTLE_STAGE_X_BISON");
        let flex = path("MANTLE_STAGE_X_FLEX");
        let make = path("MANTLE_STAGE_X_MAKE");
        let exit_code = probe_authenticated_intl_configure(BinutilsConfigureProbeRequest {
            source_root: &source_root,
            scratch_dir: &scratch_dir,
            bash: &bash,
            tcc: &tcc,
            musl_root: &musl_root,
            coreutils_bin: &coreutils_bin,
            sed: &sed,
            grep: &grep,
            diff: &diff,
            cmp: &cmp,
            gawk: &gawk,
            m4: &m4,
            bison: &bison,
            flex: &flex,
            make: &make,
        })
        .unwrap();
        let stderr = fs::read_to_string(scratch_dir.join("configure.stderr.txt")).unwrap();
        let invocation_count = fs::read_to_string(scratch_dir.join("sed-spool/invocation.count")).unwrap();
        assert_eq!(exit_code, 0);
        assert!(!stderr.contains("config.status: error"));
        assert!(invocation_count.trim().parse::<u32>().unwrap() > 1);
        assert!(scratch_dir.join("source/intl/config.status").is_file());
        assert!(scratch_dir.join("source/intl/Makefile").is_file());
    }

    #[test]
    #[ignore = "requires retained binutils source and protected tool roots"]
    fn configures_declared_matrix_with_regular_file_sed_bridge() {
        let path = |name: &str| PathBuf::from(std::env::var(name).unwrap());
        let source_root = path("MANTLE_STAGE_X_BINUTILS_RETAINED_SOURCE");
        let scratch_dir = path("MANTLE_STAGE_X_BINUTILS_CONFIGURE_SCRATCH");
        let bash = path("MANTLE_STAGE_X_BASH_FULL");
        let tcc = path("MANTLE_STAGE_X_TCC_MUSL_V2");
        let musl_root = path("MANTLE_STAGE_X_MUSL_NATIVE_ROOT");
        let coreutils_bin = path("MANTLE_STAGE_X_COREUTILS_BIN");
        let sed = path("MANTLE_STAGE_X_SED");
        let grep = path("MANTLE_STAGE_X_GREP");
        let diff = path("MANTLE_STAGE_X_DIFF");
        let cmp = path("MANTLE_STAGE_X_CMP");
        let gawk = path("MANTLE_STAGE_X_GAWK");
        let m4 = path("MANTLE_STAGE_X_M4");
        let bison = path("MANTLE_STAGE_X_BISON");
        let flex = path("MANTLE_STAGE_X_FLEX");
        let make = path("MANTLE_STAGE_X_MAKE");
        let request = BinutilsConfigureProbeRequest {
            source_root: &source_root,
            scratch_dir: &scratch_dir,
            bash: &bash,
            tcc: &tcc,
            musl_root: &musl_root,
            coreutils_bin: &coreutils_bin,
            sed: &sed,
            grep: &grep,
            diff: &diff,
            cmp: &cmp,
            gawk: &gawk,
            m4: &m4,
            bison: &bison,
            flex: &flex,
            make: &make,
        };
        let outcomes = probe_authenticated_configure_matrix(request).unwrap();
        assert_eq!(outcomes.len(), CONFIGURE_CLASS_COUNT);
        assert!(outcomes.iter().all(|outcome| outcome.exit_code == 0));
        assert!(
            CONFIGURE_CLASSES.iter().all(|class| scratch_dir
                .join("source")
                .join(class)
                .join("config.status")
                .is_file())
        );
        assert!(
            CONFIGURE_CLASSES
                .iter()
                .all(|class| scratch_dir.join("source").join(class).join("Makefile").is_file())
        );
        assert!(FILE_RELOCATION_CLASSES.iter().all(|class| {
            !fs::read_to_string(scratch_dir.join("source").join(class).join("configure"))
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
}
