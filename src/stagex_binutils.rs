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
const BINUTILS_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-binutils-2.30-source-materialization-v1";
const BINUTILS_SOURCE_NON_CLAIM: &str =
    "binutils source materialization proves authenticated offline archive identity and checked-recipe identity only";
const BINUTILS_RECORD_HASH: &str = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=";
const BINUTILS_RECORD_URL: &str = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz";
const BINUTILS_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const BINUTILS_RECORD_UNPACK: &str = "1";
const BINUTILS_SOURCE_ARTIFACT_COUNT: usize = 2;
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

const COREUTILS_TOOL_NAMES: &[&str] = &[
    "basename", "cat", "chmod", "cp", "dirname", "echo", "expr", "false", "head", "install", "ln", "ls", "mkdir", "mv",
    "rm", "rmdir", "sort", "tail", "tee", "test", "touch", "tr", "true", "uniq", "wc",
];
const COREUTILS_TOOL_COUNT: usize = 25;
const CONFIGURE_OUTPUT_MEBIBYTES_MAX: u64 = 16;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const CONFIGURE_OUTPUT_BYTES_MAX: u64 = CONFIGURE_OUTPUT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const TARGET: &str = "x86_64-unknown-linux-gnu";
const BINUTILS_RUNTIME_START: &str = "      $BB cat > \"$WORK/binutils-runtime.c\" <<'RUNTIME_EOF'\n";
const BINUTILS_RUNTIME_END: &str = "\nRUNTIME_EOF";
const REGULAR_FILE_MODE: u32 = 0o644;

pub(crate) fn probe_authenticated_intl_configure(
    request: BinutilsConfigureProbeRequest<'_>,
) -> Result<i32, StagexBinutilsError> {
    validate_configure_probe_request(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure probe root: {error}")))?;
    let source = request.scratch_dir.join("source");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source)
        .map_err(StagexBinutilsError::from_runtime)?;
    let tools = request.scratch_dir.join("tools");
    fs::create_dir(&tools)
        .map_err(|error| StagexBinutilsError::Materialization(format!("creating configure tool namespace: {error}")))?;
    populate_probe_tool_namespace(&request, &tools)?;
    let compiler_wrapper = prepare_tcc_wrapper(&request)?;
    let stdin_path = request.scratch_dir.join("configure.stdin");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"").map_err(StagexBinutilsError::from_runtime)?;
    let stdout_path = request.scratch_dir.join("configure.stdout.txt");
    let stderr_path = request.scratch_dir.join("configure.stderr.txt");
    let arguments = configure_arguments();
    let environment = configure_environment(&request, &tools, &compiler_wrapper)?;
    let configure_dir = source.join("intl");
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        request.bash,
        &arguments,
        &configure_dir,
        &environment,
        &stdin_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stdout_path,
        CONFIGURE_OUTPUT_BYTES_MAX,
        &stderr_path,
    )
    .map_err(StagexBinutilsError::from_runtime)?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(exit_code)
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
) -> Result<(), StagexBinutilsError> {
    for name in COREUTILS_TOOL_NAMES {
        add_tool_binding(tools, name, &request.coreutils_bin.join(name))?;
    }
    let bindings = [
        ("sh", request.bash),
        ("bash", request.bash),
        ("cc", request.tcc),
        ("sed", request.sed),
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
    tools: &Path,
    compiler_wrapper: &Path,
) -> Result<std::collections::BTreeMap<String, String>, StagexBinutilsError> {
    let utf8 = |path: &Path, label: &str| {
        path.to_str()
            .map(str::to_string)
            .ok_or_else(|| StagexBinutilsError::Materialization(format!("{label} is not UTF-8")))
    };
    let bash = utf8(request.bash, "full Bash")?;
    let wrapper = utf8(compiler_wrapper, "TinyCC wrapper")?;
    let compiler = format!("{bash} {wrapper}");
    let musl_include = utf8(&request.musl_root.join("include"), "native musl include")?;
    let musl_lib = utf8(&request.musl_root.join("lib"), "native musl lib")?;
    let mut environment = std::collections::BTreeMap::new();
    environment.insert("PATH".to_string(), utf8(tools, "tool namespace")?);
    environment.insert("CONFIG_SHELL".to_string(), bash.clone());
    environment.insert("SHELL".to_string(), bash);
    environment.insert("CC".to_string(), compiler.clone());
    environment.insert("CC_FOR_BUILD".to_string(), compiler.clone());
    environment.insert("CPP".to_string(), format!("{compiler} -E"));
    environment.insert("AR".to_string(), "true".to_string());
    environment.insert("RANLIB".to_string(), "true".to_string());
    environment.insert("LD".to_string(), "true".to_string());
    environment.insert("MAKEINFO".to_string(), "true".to_string());
    environment.insert("AWK".to_string(), utf8(request.gawk, "Gawk")?);
    environment.insert("M4".to_string(), utf8(request.m4, "M4")?);
    environment.insert("CFLAGS".to_string(), format!("-I{musl_include} -static -D_GNU_SOURCE"));
    environment.insert("LDFLAGS".to_string(), format!("-static -L{musl_lib}"));
    let probe_root = utf8(request.scratch_dir, "configure probe root")?;
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_DIR".to_string(), format!("{probe_root}/source/intl"));
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS".to_string(), "intl".to_string());
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT".to_string(), format!("{probe_root}/preprocess.audit"));
    environment.insert("MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT".to_string(), format!("{probe_root}/preprocess.count"));
    assert!(environment.contains_key("CONFIG_SHELL"));
    assert_eq!(environment.get("LD").map(String::as_str), Some("true"));
    Ok(environment)
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
    fn detects_protected_sed_config_status_blocker() {
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
        assert_eq!(exit_code, 1);
        assert!(stderr.contains("config.status: error: could not create Makefile"));
        assert!(scratch_dir.join("source/intl/config.status").is_file());
    }
}
