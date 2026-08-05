use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const FLEX_RECORD_NAME: &str = "flex-2.6.4-src";
const FLEX_RECORD_IDENTITY: &str = "fixed-url-61814c646c0a35930a96b389ab3d211f942a0dfd503ee8f33247bdfbb41cb980";
const FLEX_SOURCE_OUTPUT_NAME: &str = "flex-2.6.4";
pub(crate) const FLEX_SOURCE_ARTIFACT_ID: &str = "flex-2.6.4-source";
pub(crate) const FLEX_SOURCE_CONTENT_BLAKE3: &str = "6a84744fa55e734f9640c58c694c509dba34ddff104ab83e0131b9bc2468f8c0";
pub(crate) const FLEX_RECIPE_ARTIFACT_ID: &str = "flex-2.6.4-recipe-source";
pub(crate) const FLEX_RECIPE_BLAKE3: &str = "b9e11c2ef244ef9fab988d13bda45d0f0bdeabea8913964bb58cb00c9ac9688b";
const FLEX_RECIPE: &[u8] = include_bytes!("../bootstrap/flex-2.6.4-musl.ncl");
const FLEX_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-flex-source-materialization-v1";
const FLEX_SOURCE_NON_CLAIM: &str =
    "Flex source materialization proves authenticated offline archive identity and fixed-output parity only";
const FLEX_RECORD_HASH: &str = "sha256-HBPAUDpqF4HYocMD3YDIk6OaIUrZw8Gk3K59OmHB6xU=";
const FLEX_RECORD_URL: &str = "https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz";
const FLEX_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const FLEX_RECORD_UNPACK: &str = "1";
const FLEX_SOURCE_ARTIFACT_COUNT: usize = 2;
const REQUIRED_SOURCE_FILES: &[&str] = &[
    "src/main.c",
    "src/filter.c",
    "src/misc.c",
    "src/parse.c",
    "src/scan.c",
    "src/FlexLexer.h",
];
const REQUIRED_SOURCE_FILE_COUNT: usize = 6;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const FLEX_SOURCE_FILE_MEBIBYTES_MAX: u64 = 8;
const FLEX_SOURCE_FILE_BYTES_MAX: u64 = FLEX_SOURCE_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const FLEX_ARTIFACT_MEBIBYTES_MAX: u64 = 64;
const FLEX_ARTIFACT_BYTES_MAX: u64 = FLEX_ARTIFACT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const FLEX_OBSERVATION_MEBIBYTES_MAX: u64 = 16;
const FLEX_OBSERVATION_BYTES_MAX: u64 = FLEX_OBSERVATION_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const FLEX_PROGRAM_SOURCE_COUNT: usize = 20;
const FLEX_OBJECT_COUNT: usize = FLEX_PROGRAM_SOURCE_COUNT + 1;
const FLEX_BUILD_COMMAND_COUNT: u32 = 25;
const FLEX_SMOKE_COMMAND_COUNT: u32 = 4;
const FLEX_OUTPUT_COUNT: usize = 7;
const FLEX_EXECUTABLE_MODE: u32 = 0o755;
const FLEX_REGULAR_MODE: u32 = 0o644;
const FLEX_PERMISSION_MODE_MASK: u32 = 0o777;
const FLEX_NO_PERMISSION_MODE: u32 = 0o000;
const FLEX_EXECUTE_MODE_MASK: u32 = 0o111;
const ELF_MAGIC_BYTES: usize = 4;
const ELF_MAGIC: &[u8; ELF_MAGIC_BYTES] = b"\x7fELF";
const ELF64_HEADER_BYTES: usize = 64;
const ELF64_SECTION_HEADER_BYTES: usize = 64;
const ELF_IDENT_CLASS_OFFSET: usize = 4;
const ELF_IDENT_DATA_OFFSET: usize = 5;
const ELF_TYPE_OFFSET: usize = 16;
const ELF_MACHINE_OFFSET: usize = 18;
const ELF_SECTION_TABLE_OFFSET: usize = 40;
const ELF_SECTION_HEADER_SIZE_OFFSET: usize = 58;
const ELF_SECTION_COUNT_OFFSET: usize = 60;
const ELF_SECTION_NAMES_INDEX_OFFSET: usize = 62;
const ELF_SECTION_NAME_OFFSET: usize = 0;
const ELF_SECTION_TYPE_OFFSET: usize = 4;
const ELF_SECTION_FLAGS_OFFSET: usize = 8;
const ELF_SECTION_DATA_OFFSET: usize = 24;
const ELF_SECTION_DATA_SIZE_OFFSET: usize = 32;
const ELF_SECTION_ALIGNMENT_OFFSET: usize = 48;
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_TYPE_EXECUTABLE: u16 = 2;
const ELF_MACHINE_X86_64: u16 = 62;
const ELF_SECTION_TYPE_PROGBITS: u32 = 1;
const ELF_SECTION_TYPE_STRING_TABLE: u32 = 3;
const ELF_SECTION_FLAG_ALLOC: u64 = 2;
const FLEX_RUNTIME_SECTION_ALIGNMENT: u64 = 8;
const FLEX_RUNTIME_SECTION_BYTES_LEN: usize = 49;
const FLEX_RUNTIME_SECTION_NAME_BYTES_LEN: usize = 3;
const FLEX_RUNTIME_SECTION_NAME_BYTES_LEN_MAX: usize = 4;
const FLEX_RUNTIME_SECTION_NAME: &[u8; FLEX_RUNTIME_SECTION_NAME_BYTES_LEN] = b"stx";
const FLEX_RUNTIME_SECTION_BYTES: [u8; FLEX_RUNTIME_SECTION_BYTES_LEN] = [
    0x48, 0xc7, 0xc0, 0x39, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc3, 0x48, 0x89, 0xfe, 0x48, 0xc7, 0xc7, 0xff, 0xff, 0xff,
    0xff, 0x48, 0x31, 0xd2, 0x4d, 0x31, 0xd2, 0x48, 0xc7, 0xc0, 0x3d, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc3, 0x48, 0x31,
    0xd2, 0x48, 0xc7, 0xc0, 0x3b, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc3,
];
const FLEX_RUNTIME_SECTION_MATCH_COUNT: usize = 1;
const FLEX_COMPAT_START: &str = "      $BB cat > /tmp/flex-compat.c <<'COMPAT'\n";
const FLEX_COMPAT_END: &str = "\nCOMPAT\n";
const FLEX_REPORT_FORMAT: &str = "mantle-stagex-flex-2.6.4-inventory-v1";
const FLEX_NON_CLAIM: &str = "this inventory binds GNU Flex 2.6.4, its exact bounded source normalization and compatibility runtime, its declared GNU M4 child, and positive and negative scanner observations only; it does not prove arbitrary scanner semantics, binutils, native TinyCC, compiler correctness, or provider admission";
pub(crate) const FLEX_CONFIGURED_SOURCE_BLAKE3: &str =
    "d2d7e2a0ed46329fc2e37d58d25dab8f17c01b5c9b7b97de09930ef3e25fe3b7";
pub(crate) const FLEX_BINARY_BLAKE3: &str = "502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf";
const FLEX_LIBFL_BLAKE3: &str = "45c3a2d2e338eae19d24fa17df35127f5c61c16dca0ce98dce45efb13b920bfa";
const FLEX_HEADER_BLAKE3: &str = "713ca824326279098cc1eb9b18ddd200e7506335c502cf2d79b4af7aaa330cdf";
pub(crate) const FLEX_LIBFL_CONSUMER_BLAKE3: &str = "9d13005da4640c131d85edff0a255a83471b136d31ce41d2b04534c8a7b533b4";
const FLEX_VERSION_BLAKE3: &str = "3697eccbbb98cf059d94cc199ff12f77bee88f5097ad8f0826c86bcf1fcbd5a7";
const FLEX_POSITIVE_SOURCE_BLAKE3: &str = "fb2c93883438dca7366f41ba363944da66b78787f76be32767cb697943b26732";
const FLEX_NEGATIVE_BLAKE3: &str = "f1180cfefce605bb56ee38ccfec3bbd99f448e2d5f391bec146f4a02cdff336f";
const FLEX_PROGRAM_SOURCES: &[&str] = &[
    "buf",
    "ccl",
    "dfa",
    "ecs",
    "filter",
    "gen",
    "main",
    "misc",
    "nfa",
    "options",
    "parse",
    "scan",
    "scanflags",
    "scanopt",
    "skel",
    "sym",
    "tables",
    "tables_shared",
    "tblcmp",
    "yylex",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FlexSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FlexExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const FLEX_EXPECTED_OUTPUTS: [FlexExpectedOutput; FLEX_OUTPUT_COUNT] = [
    FlexExpectedOutput {
        artifact_id: "flex-2.6.4",
        digest_blake3: FLEX_BINARY_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-libfl",
        digest_blake3: FLEX_LIBFL_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-cxx-header",
        digest_blake3: FLEX_HEADER_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-libfl-consumer",
        digest_blake3: FLEX_LIBFL_CONSUMER_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-version-observation",
        digest_blake3: FLEX_VERSION_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-positive-scanner-source",
        digest_blake3: FLEX_POSITIVE_SOURCE_BLAKE3,
    },
    FlexExpectedOutput {
        artifact_id: "flex-negative-observation",
        digest_blake3: FLEX_NEGATIVE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FlexOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FlexInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub negative_exit_code: i32,
    pub outputs: Vec<FlexOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlexInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub m4_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexFlexError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexFlexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU Flex source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU Flex materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU Flex runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexFlexError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexFlexError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); FLEX_SOURCE_ARTIFACT_COUNT] {
    [
        (FLEX_SOURCE_ARTIFACT_ID, FLEX_SOURCE_CONTENT_BLAKE3),
        (FLEX_RECIPE_ARTIFACT_ID, FLEX_RECIPE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_flex_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<FlexSourceMaterializationReport, StagexFlexError> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexFlexError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexFlexError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex source scratch: {error}")))?;
    let output_path = scratch_dir.join(FLEX_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path)
        .map_err(|error| StagexFlexError::Materialization(format!("materializing Flex source: {error}")))?;
    validate_materialized_source(&output_path)?;
    let materialization = FlexSourceMaterializationReport {
        format: FLEX_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: FLEX_SOURCE_ARTIFACT_ID,
        record_name: FLEX_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: FLEX_SOURCE_NON_CLAIM,
    };
    assert_eq!(materialization.record_identity, FLEX_RECORD_IDENTITY);
    assert!(materialization.output_path.join("src/main.c").is_file());
    Ok(materialization)
}

fn validate_bundle_path(path: &Path) -> Result<(), StagexFlexError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(StagexFlexError::Materialization(format!(
            "source bundle must be an absolute regular file: {}",
            path.display()
        )));
    }
    let bytes_len = path
        .metadata()
        .map_err(|error| StagexFlexError::Materialization(format!("reading source-bundle metadata: {error}")))?
        .len();
    if bytes_len == 0 {
        return Err(StagexFlexError::Materialization("source bundle is empty".to_string()));
    }
    assert!(path.is_absolute());
    assert!(bytes_len > 0);
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexFlexError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(FLEX_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(StagexFlexError::SourceRecordNotFound);
    }
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some(FLEX_RECORD_NAME));
    Ok(matches[0])
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexFlexError> {
    require_record_fact(record.identity == FLEX_RECORD_IDENTITY, "source record identity")?;
    require_record_fact(record.kind == SourceRecordKind::FixedUrl, "source record kind")?;
    require_metadata(record, "builder", "builtin:fetchurl")?;
    require_metadata(record, "hash", FLEX_RECORD_HASH)?;
    require_metadata(record, "hash_algo", "sha256")?;
    require_metadata(record, "hash_mode", "recursive")?;
    require_metadata(record, "payload_encoding", FLEX_RECORD_PAYLOAD_ENCODING)?;
    require_metadata(record, "unpack", FLEX_RECORD_UNPACK)?;
    require_metadata(record, "url", FLEX_RECORD_URL)?;
    require_record_fact(record.content_blake3 == FLEX_SOURCE_CONTENT_BLAKE3, "source content BLAKE3")?;
    require_record_fact(record.files.len() == 1, "source record file count")?;
    let archive = &record.files[0];
    require_record_fact(archive.path == "archive", "source archive path")?;
    require_record_fact(archive.file_type == SourceFileType::Regular, "source archive file type")?;
    require_record_fact(!archive.executable, "source archive mode")?;
    require_record_fact(archive.size > 0, "source archive size")?;
    require_record_fact(archive.content_hex.is_some(), "source archive bytes")?;
    assert_eq!(record.identity, FLEX_RECORD_IDENTITY);
    assert_eq!(record.files.len(), 1);
    Ok(())
}

fn require_record_fact(condition: bool, label: &str) -> Result<(), StagexFlexError> {
    if !condition {
        return Err(StagexFlexError::Materialization(format!("unexpected Flex {label}")));
    }
    Ok(())
}

fn require_metadata(record: &SourceRecord, key: &str, expected: &str) -> Result<(), StagexFlexError> {
    let observed = record.metadata.get(key).map(String::as_str);
    if observed != Some(expected) {
        return Err(StagexFlexError::Materialization(format!(
            "unexpected Flex source metadata {key}: expected {expected}, observed {observed:?}"
        )));
    }
    Ok(())
}

fn validate_materialized_source(path: &Path) -> Result<(), StagexFlexError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(StagexFlexError::Materialization(format!(
            "materialized Flex source is not an absolute directory: {}",
            path.display()
        )));
    }
    for relative in REQUIRED_SOURCE_FILES {
        if !path.join(relative).is_file() {
            return Err(StagexFlexError::Materialization(format!("materialized Flex source is missing {relative}")));
        }
    }
    assert_eq!(REQUIRED_SOURCE_FILES.len(), REQUIRED_SOURCE_FILE_COUNT);
    assert!(path.join("src/FlexLexer.h").is_file());
    Ok(())
}

pub(crate) fn validate_recipe_digest() -> Result<(), StagexFlexError> {
    let observed = blake3::hash(FLEX_RECIPE).to_hex().to_string();
    if observed != FLEX_RECIPE_BLAKE3 {
        return Err(StagexFlexError::Materialization(format!(
            "Flex recipe BLAKE3 mismatch: expected {FLEX_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert_ne!(observed, FLEX_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

pub(crate) fn derive_flex_inventory(request: FlexInventoryRequest<'_>) -> Result<FlexInventoryReport, StagexFlexError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex runtime scratch: {error}")))?;
    let retained_source = request.scratch_dir.join(FLEX_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &retained_source)?;
    configure_retained_source(&retained_source)?;
    let output = request.scratch_dir.join("output");
    let output_bin = output.join("bin");
    let output_lib = output.join("lib");
    let output_include = output.join("include");
    fs::create_dir_all(&output_bin)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex bin output: {error}")))?;
    fs::create_dir_all(&output_lib)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex library output: {error}")))?;
    fs::create_dir_all(&output_include)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex include output: {error}")))?;
    let products = build_flex(&request, &retained_source, &output_bin, &output_lib, &output_include)?;
    let observations = run_checked_smokes(&request, &products)?;
    let outputs = collect_outputs(&products, &observations)?;
    validate_expected_outputs(&outputs)?;
    let inventory = FlexInventoryReport {
        format: FLEX_REPORT_FORMAT,
        configured_source_digest_blake3: configured_source_digest_blake3()?,
        source_compile_count: u32::try_from(FLEX_OBJECT_COUNT)
            .map_err(|_| StagexFlexError::Materialization("Flex object count does not fit u32".to_string()))?,
        build_command_count: FLEX_BUILD_COMMAND_COUNT,
        smoke_command_count: FLEX_SMOKE_COMMAND_COUNT,
        negative_exit_code: observations.negative_exit_code,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: FLEX_NON_CLAIM,
    };
    let inventory_path = request.scratch_dir.join("flex-inventory.json");
    let bytes = serde_json::to_vec_pretty(&inventory)
        .map_err(|error| StagexFlexError::Materialization(format!("serializing Flex inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&inventory_path, &bytes)?;
    assert_eq!(inventory.outputs.len(), FLEX_OUTPUT_COUNT);
    assert_ne!(inventory.negative_exit_code, 0);
    Ok(inventory)
}

fn validate_inventory_inputs(request: &FlexInventoryRequest<'_>) -> Result<(), StagexFlexError> {
    if request.scratch_dir.exists() {
        return Err(StagexFlexError::Materialization(format!(
            "Flex runtime scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    validate_materialized_source(request.source_root)?;
    validate_recipe_digest()?;
    let configured_source = configured_source_digest_blake3()?;
    if configured_source != FLEX_CONFIGURED_SOURCE_BLAKE3 {
        return Err(StagexFlexError::Materialization(format!(
            "Flex configured-source BLAKE3 mismatch: expected {FLEX_CONFIGURED_SOURCE_BLAKE3}, observed {configured_source}"
        )));
    }
    validate_file_digest(
        &request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2"),
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "TinyCC musl-v2",
    )?;
    validate_file_digest(&request.m4_root.join("bin/m4"), crate::stagex_m4::M4_FINAL_BLAKE3, "protected GNU M4")?;
    let libc_digest = native_musl_digest("musl-native-libc")?;
    let crt1_digest = native_musl_digest("musl-native-crt1")?;
    validate_file_digest(&request.musl_native_root.join("lib/libc.a"), libc_digest, "native musl libc")?;
    validate_file_digest(&request.musl_native_root.join("lib/crt1.o"), crt1_digest, "native musl crt1")?;
    if !request.musl_native_root.join("include/stdio.h").is_file() {
        return Err(StagexFlexError::Materialization("native musl headers are missing".to_string()));
    }
    assert!(!request.scratch_dir.exists());
    assert!(request.m4_root.join("bin/m4").is_file());
    Ok(())
}

fn flex_config_header() -> String {
    let header = r#"#define PACKAGE "flex"
#define PACKAGE_NAME "flex"
#define VERSION "2.6.4"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define STDC_HEADERS 1
#define FLEX_DECIMAL_RENDER_CAPACITY 32
#define FLEX_ASCII_SPACE 32
#define FLEX_ASCII_NEWLINE 10
#define FLEX_VERSION_ARGUMENT_COUNT 2
#define FLEX_ARRAY_ELEMENTS_MAX 1048576
#define FLEX_ARRAY_ELEMENT_BYTES_MAX 64
#define FLEX_ARRAY_ALLOCATION_BYTES_MAX 67108864
#define M4 "/mantle/stagex/required-m4"
int flex_parse_decimal(const char *input);
int flex_exec_declared(const char *path, char *const arguments[]);
"#;
    assert!(header.contains("/mantle/stagex/required-m4"));
    assert!(!header.contains("$NIX_STORE"));
    header.to_string()
}

fn configure_retained_source(source_root: &Path) -> Result<(), StagexFlexError> {
    let compatibility = extract_recipe_block(FLEX_COMPAT_START, FLEX_COMPAT_END, "compatibility runtime")?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), flex_config_header().as_bytes())?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("mantle_flex_compat.c"), compatibility.as_bytes())?;
    transform_source_file(&source_root.join("src/filter.c"), transform_filter_source, "filter")?;
    transform_source_file(&source_root.join("src/misc.c"), transform_misc_source, "misc")?;
    transform_source_file(&source_root.join("src/main.c"), transform_main_source, "main")?;
    transform_source_file(&source_root.join("src/buf.c"), transform_buf_source, "buffer")?;
    validate_configured_source(source_root)?;
    assert!(source_root.join("config.h").is_file());
    assert!(source_root.join("mantle_flex_compat.c").is_file());
    Ok(())
}

fn transform_source_file(
    path: &Path,
    transform: fn(&str) -> Result<String, StagexFlexError>,
    label: &str,
) -> Result<(), StagexFlexError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FLEX_SOURCE_FILE_BYTES_MAX, label)?;
    let source = String::from_utf8(bytes)
        .map_err(|error| StagexFlexError::Materialization(format!("reading UTF-8 Flex {label} source: {error}")))?;
    let transformed = transform(&source)?;
    fs::write(path, transformed.as_bytes()).map_err(|error| {
        StagexFlexError::Materialization(format!("writing configured Flex {label} source: {error}"))
    })?;
    assert_ne!(transformed, source);
    assert!(!transformed.is_empty());
    Ok(())
}

fn transform_filter_source(input: &str) -> Result<String, StagexFlexError> {
    let mut text = input.to_string();
    replace_c_function(
        &mut text,
        "struct filter *filter_create_ext (struct filter *chain, const char *cmd,",
        FILTER_CREATE_EXT_NEW,
        "external-filter adapter",
    )?;
    replace_line_region(
        &mut text,
        "int filter_fix_linedirs (struct filter *chain)\n",
        "/* vim:set expandtab cindent tabstop=4 softtabstop=4 shiftwidth=4 textwidth=0: */\n",
        FILTER_FIX_LINEDIRS_NEW,
        "line filter",
    )?;
    replace_exact(
        &mut text,
        "if (freopen ((char *) chain->extra, \"w\", stdout) == NULL)",
        "fclose(stdout); if ((stdout = fopen ((char *) chain->extra, \"w\")) == NULL)",
        "header stdout replacement",
    )?;
    replace_exact(
        &mut text,
        "flexfatal (_(\"freopen(headerfilename) failed\"));",
        "flexfatal (_(\"fopen(headerfilename) failed\"));",
        "header diagnostic",
    )?;
    replace_exact(
        &mut text,
        "execvp (chain->argv[0],",
        "flex_exec_declared (chain->argv[0],",
        "declared process call",
    )?;
    if text.contains("va_start")
        || text.contains("va_arg")
        || text.contains("va_end")
        || text.contains("freopen")
        || text.contains("execvp")
    {
        return Err(StagexFlexError::Materialization(
            "unsupported Flex filter source surface remained after normalization".to_string(),
        ));
    }
    assert!(text.contains("FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT"));
    assert!(text.contains("flex_exec_declared"));
    Ok(text)
}

const FILTER_CREATE_EXT_NEW: &str = r#"enum { FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT = 2, FLEX_EXTERNAL_FILTER_VECTOR_LENGTH = 3 };
struct filter *filter_create_ext (struct filter *chain, const char *cmd, ...)
{
    struct filter *f;
    if (cmd == NULL) flexerror (_("missing command in filter_create_ext"));
    f = malloc(sizeof(*f));
    if (f == NULL) flexerror (_("malloc failed in filter_create_ext"));
    memset(f, 0, sizeof(*f));
    if (chain != NULL && chain->next != NULL)
        flexerror (_("unexpected external-filter chain shape"));
    f->argc = FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT;
    f->argv = malloc(sizeof(char *) * FLEX_EXTERNAL_FILTER_VECTOR_LENGTH);
    if (f->argv == NULL) flexerror (_("malloc failed for external-filter argv"));
    f->argv[0] = cmd;
    f->argv[1] = "-P";
    f->argv[2] = NULL;
    if (chain != NULL) chain->next = f;
    return f;
}"#;

const FILTER_FIX_LINEDIRS_NEW: &str = r#"int filter_fix_linedirs (struct filter *chain)
{
    int byte;
    if (chain == NULL) return 0;
    while ((byte = fgetc(stdin)) != EOF) {
        if (fputc(byte, stdout) == EOF) flexerror ("error writing filtered output");
    }
    if (ferror(stdin)) flexerror ("error reading filtered input");
    if (fflush(stdout) != 0) flexerror ("error flushing filtered output");
    return 0;
}
/* vim:set expandtab cindent tabstop=4 softtabstop=4 shiftwidth=4 textwidth=0: */
"#;

fn transform_misc_source(input: &str) -> Result<String, StagexFlexError> {
    let mut text = input.to_string();
    replace_c_function(&mut text, "void lerr (const char *msg, ...)", LERR_NEW, "error reporter")?;
    replace_c_function(&mut text, "void lerr_fatal (const char *msg, ...)", LERR_FATAL_NEW, "fatal error reporter")?;
    replace_c_function(
        &mut text,
        "void   *allocate_array (int size, size_t element_size)",
        ALLOCATE_ARRAY_NEW,
        "allocator",
    )?;
    replace_c_function(
        &mut text,
        "void   *reallocate_array (void *array, int size, size_t element_size)",
        REALLOCATE_ARRAY_NEW,
        "reallocator",
    )?;
    replace_exact(
        &mut text,
        "(void) sscanf (array, \"%d\", &val);",
        "val = flex_parse_decimal(array);",
        "decimal parser",
    )?;
    if text.contains("va_start") || text.contains("va_arg") || text.contains("va_end") || text.contains("sscanf") {
        return Err(StagexFlexError::Materialization(
            "unsupported Flex misc source surface remained after normalization".to_string(),
        ));
    }
    assert!(text.contains("FLEX_ARRAY_ALLOCATION_BYTES_MAX"));
    assert!(text.contains("flex_parse_decimal"));
    Ok(text)
}

const LERR_NEW: &str = r#"void lerr (const char *msg, ...)
{
    if (msg == NULL) flexerror ("missing error message");
    flexerror (msg);
}"#;

const LERR_FATAL_NEW: &str = r#"void lerr_fatal (const char *msg, ...)
{
    if (msg == NULL) flexfatal ("missing fatal error message");
    flexfatal (msg);
}"#;

const ALLOCATE_ARRAY_NEW: &str = r#"void *allocate_array (int size, size_t element_size)
{
    size_t count;
    size_t byte_count;
    void *memory;
    if (size <= 0) flexfatal ("invalid array element count");
    if (element_size == 0) flexfatal ("invalid array element size");
    count = (size_t)size;
    if (count > FLEX_ARRAY_ELEMENTS_MAX) flexfatal ("array element count exceeds bound");
    if (element_size > FLEX_ARRAY_ELEMENT_BYTES_MAX) flexfatal ("array element size exceeds bound");
    byte_count = count * element_size;
    if (byte_count > FLEX_ARRAY_ALLOCATION_BYTES_MAX) flexfatal ("array allocation exceeds byte bound");
    memory = malloc(byte_count);
    if (memory == NULL) flexfatal ("memory allocation failed");
    return memory;
}"#;

const REALLOCATE_ARRAY_NEW: &str = r#"void *reallocate_array (void *array, int size, size_t element_size)
{
    size_t count;
    size_t byte_count;
    void *memory;
    if (size <= 0) flexfatal ("invalid array element count");
    if (element_size == 0) flexfatal ("invalid array element size");
    count = (size_t)size;
    if (count > FLEX_ARRAY_ELEMENTS_MAX) flexfatal ("array element count exceeds bound");
    if (element_size > FLEX_ARRAY_ELEMENT_BYTES_MAX) flexfatal ("array element size exceeds bound");
    byte_count = count * element_size;
    if (byte_count > FLEX_ARRAY_ALLOCATION_BYTES_MAX) flexfatal ("array reallocation exceeds byte bound");
    memory = realloc(array, byte_count);
    if (memory == NULL) flexfatal ("array reallocation failed");
    return memory;
}"#;

fn transform_main_source(input: &str) -> Result<String, StagexFlexError> {
    let mut text = input.to_string();
    replace_exact(&mut text, "int main (int argc, char *argv[])\n{", FLEX_MAIN_WRAPPER_NEW, "main wrapper")?;
    replace_exact(
        &mut text,
        "(size_t)(1 + ceil (log10(i))) + 2",
        "FLEX_DECIMAL_RENDER_CAPACITY",
        "main decimal width",
    )?;
    replace_exact(
        &mut text,
        "prev_stdout = freopen (outfilename, \"w+\", stdout);",
        "fclose(stdout); prev_stdout = stdout = fopen (outfilename, \"w+\");",
        "main stdout replacement",
    )?;
    replace_line_region(
        &mut text,
        "\tprintstats = syntaxerror = trace = spprdflt = false;\n",
        "\ttablesfilename = tablesname = NULL;\n",
        FLEX_GLOBAL_INITIALIZATION_NEW,
        "global initialization",
    )?;
    replace_line_region(
        &mut text,
        "\t\tcase OPT_VERSION:\n",
        "\t\t\tFLEX_EXIT (0);\n",
        FLEX_VERSION_CASE_NEW,
        "version option",
    )?;
    replace_exact(
        &mut text,
        "action_array = allocate_character_array (action_size);",
        "action_array = (char *)malloc((size_t)action_size); if (action_array == NULL) flexfatal(\"action allocation failed\");",
        "action allocation",
    )?;
    if text.contains("log10") || text.contains("freopen") {
        return Err(StagexFlexError::Materialization(
            "unsupported Flex main source surface remained after normalization".to_string(),
        ));
    }
    assert!(text.contains("flex_version_output"));
    assert!(text.contains("action allocation failed"));
    Ok(text)
}

const FLEX_MAIN_WRAPPER_NEW: &str = r#"int main (int argc, char *argv[])
{
    static const char flex_version_output[] = "flex 2.6.4\n";
    if (argc == FLEX_VERSION_ARGUMENT_COUNT && strcmp(argv[1], "--version") == 0) {
        write(STDOUT_FILENO, flex_version_output, sizeof(flex_version_output) - 1);
        return 0;
    }"#;

const FLEX_GLOBAL_INITIALIZATION_NEW: &str = r#"    printstats = false;
    syntaxerror = false;
    trace = false;
    spprdflt = false;
    lex_compat = false;
    posix_compat = false;
    C_plus_plus = false;
    backing_up_report = false;
    ddebug = false;
    fulltbl = false;
    fullspd = false;
    long_align = false;
    nowarn = false;
    yymore_used = false;
    continued_action = false;
    do_yylineno = false;
    yytext_is_array = false;
    in_rule = false;
    reject = false;
    do_stdinit = false;
    yymore_really_used = unspecified;
    reject_really_used = unspecified;
    interactive = unspecified;
    csize = unspecified;
    do_yywrap = true;
    gen_line_dirs = true;
    usemecs = true;
    useecs = true;
    reentrant = false;
    bison_bridge_lval = false;
    bison_bridge_lloc = false;
    performance_report = 0;
    did_outfilename = 0;
    prefix = "yy";
    yyclass = NULL;
    use_read = false;
    use_stdout = false;
    tablesext = false;
    tablesverify = false;
    gentables = true;
    tablesfilename = NULL;
    tablesname = NULL;
"#;

const FLEX_VERSION_CASE_NEW: &str = r#"        case OPT_VERSION:
            fputs(program_name, stdout);
            fputc(FLEX_ASCII_SPACE, stdout);
            fputs(flex_version, stdout);
            fputc(FLEX_ASCII_NEWLINE, stdout);
            FLEX_EXIT (0);
"#;

fn transform_buf_source(input: &str) -> Result<String, StagexFlexError> {
    let mut text = input.to_string();
    replace_exact(
        &mut text,
        "(size_t) (1 + ceil (log10 (abs (lineno))))",
        "FLEX_DECIMAL_RENDER_CAPACITY",
        "buffer decimal width",
    )?;
    if text.contains("log10") || text.contains("ceil") {
        return Err(StagexFlexError::Materialization(
            "unsupported Flex buffer decimal-width call remained".to_string(),
        ));
    }
    assert!(text.contains("FLEX_DECIMAL_RENDER_CAPACITY"));
    assert_ne!(text, input);
    Ok(text)
}

fn replace_exact(text: &mut String, old: &str, new: &str, label: &str) -> Result<(), StagexFlexError> {
    let observed_count = text.matches(old).count();
    if observed_count != 1 {
        return Err(StagexFlexError::Materialization(format!(
            "unexpected Flex {label} source shape: expected 1, observed {observed_count}"
        )));
    }
    *text = text.replacen(old, new, 1);
    if !new.contains(old) {
        assert_eq!(text.matches(old).count(), 0);
    }
    assert!(text.contains(new));
    Ok(())
}

fn replace_line_region(
    text: &mut String,
    start: &str,
    end: &str,
    replacement: &str,
    label: &str,
) -> Result<(), StagexFlexError> {
    if text.matches(start).count() != 1 {
        return Err(StagexFlexError::Materialization(format!("unexpected Flex {label} start marker count")));
    }
    let start_index = text
        .find(start)
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} start marker is missing")))?;
    let suffix = &text[start_index..];
    let relative_end = suffix
        .find(end)
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} end marker is missing")))?;
    let end_index = start_index + relative_end + end.len();
    if end_index <= start_index {
        return Err(StagexFlexError::Materialization(format!("Flex {label} marker order is invalid")));
    }
    text.replace_range(start_index..end_index, replacement);
    assert!(text.contains(replacement));
    assert!(!replacement.is_empty());
    Ok(())
}

fn replace_c_function(
    text: &mut String,
    signature_start: &str,
    replacement: &str,
    label: &str,
) -> Result<(), StagexFlexError> {
    if text.matches(signature_start).count() != 1 {
        return Err(StagexFlexError::Materialization(format!("unexpected Flex {label} function count")));
    }
    let start = text
        .find(signature_start)
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} function is missing")))?;
    let open_relative = text[start..]
        .find('{')
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} opening brace is missing")))?;
    let open = start + open_relative;
    let end = find_balanced_c_function_end(text, open, label)?;
    text.replace_range(start..end, replacement);
    assert!(text.contains(replacement));
    assert_ne!(end, start);
    Ok(())
}

fn find_balanced_c_function_end(text: &str, open: usize, label: &str) -> Result<usize, StagexFlexError> {
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut in_character = false;
    let mut escaped = false;
    for (relative, byte) in text.as_bytes()[open..].iter().copied().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && (in_string || in_character) {
            escaped = true;
            continue;
        }
        if byte == b'"' && !in_character {
            in_string = !in_string;
            continue;
        }
        if byte == b'\'' && !in_string {
            in_character = !in_character;
            continue;
        }
        if in_string || in_character {
            continue;
        }
        if byte == b'{' {
            depth = depth
                .checked_add(1)
                .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} brace depth overflow")))?;
        } else if byte == b'}' {
            depth = depth
                .checked_sub(1)
                .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} brace depth underflow")))?;
            if depth == 0 {
                let end = open + relative + 1;
                assert!(end > open);
                assert_eq!(&text.as_bytes()[end - 1], &b'}');
                return Ok(end);
            }
        }
    }
    Err(StagexFlexError::Materialization(format!("Flex {label} closing brace is missing")))
}

fn extract_recipe_block(start: &str, end: &str, label: &str) -> Result<String, StagexFlexError> {
    let recipe = std::str::from_utf8(FLEX_RECIPE)
        .map_err(|error| StagexFlexError::Materialization(format!("reading UTF-8 Flex recipe: {error}")))?;
    let (_, suffix) = recipe
        .split_once(start)
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} start marker is missing")))?;
    let (body, _) = suffix
        .split_once(end)
        .ok_or_else(|| StagexFlexError::Materialization(format!("Flex {label} end marker is missing")))?;
    if body.is_empty() {
        return Err(StagexFlexError::Materialization(format!("Flex {label} block is empty")));
    }
    assert!(!body.is_empty());
    assert!(!body.contains(end));
    Ok(format!("{body}\n"))
}

fn validate_configured_source(source_root: &Path) -> Result<(), StagexFlexError> {
    let filter = fs::read_to_string(source_root.join("src/filter.c"))
        .map_err(|error| StagexFlexError::Materialization(format!("reading configured Flex filter source: {error}")))?;
    let misc = fs::read_to_string(source_root.join("src/misc.c"))
        .map_err(|error| StagexFlexError::Materialization(format!("reading configured Flex misc source: {error}")))?;
    let main = fs::read_to_string(source_root.join("src/main.c"))
        .map_err(|error| StagexFlexError::Materialization(format!("reading configured Flex main source: {error}")))?;
    if !filter.contains("FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT")
        || filter.contains("execvp")
        || filter.contains("freopen")
    {
        return Err(StagexFlexError::Materialization("bounded Flex filter patch is incomplete".to_string()));
    }
    if !misc.contains("FLEX_ARRAY_ALLOCATION_BYTES_MAX") || misc.contains("sscanf") || misc.contains("va_start") {
        return Err(StagexFlexError::Materialization("bounded Flex misc patch is incomplete".to_string()));
    }
    if !main.contains("flex_version_output") || main.contains("log10") || main.contains("freopen") {
        return Err(StagexFlexError::Materialization("bounded Flex main patch is incomplete".to_string()));
    }
    assert!(source_root.join("mantle_flex_compat.c").is_file());
    assert!(source_root.join("config.h").is_file());
    Ok(())
}

pub(crate) fn configured_source_digest_blake3() -> Result<String, StagexFlexError> {
    let compatibility = extract_recipe_block(FLEX_COMPAT_START, FLEX_COMPAT_END, "compatibility runtime")?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-flex-configured-source-v1\0");
    hasher.update(FLEX_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(FLEX_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0exact-bounded-flex-normalization-v1\0");
    hasher.update(flex_config_header().as_bytes());
    hasher.update(b"\0");
    hasher.update(compatibility.as_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

struct FlexProducts {
    flex: PathBuf,
    libfl: PathBuf,
    header: PathBuf,
    consumer: PathBuf,
}

fn read_elf_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(std::mem::size_of::<u16>())?;
    let field = bytes.get(offset..end)?;
    Some(u16::from_le_bytes([field[0], field[1]]))
}

fn read_elf_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(std::mem::size_of::<u32>())?;
    let field = bytes.get(offset..end)?;
    Some(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}

fn read_elf_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(std::mem::size_of::<u64>())?;
    let field = bytes.get(offset..end)?;
    Some(u64::from_le_bytes([
        field[0], field[1], field[2], field[3], field[4], field[5], field[6], field[7],
    ]))
}

fn write_elf_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    let encoded = value.to_le_bytes();
    let end = offset.checked_add(encoded.len()).ok_or_else(|| "Flex ELF u32 write offset overflow".to_string())?;
    let field = bytes.get_mut(offset..end).ok_or_else(|| "Flex ELF u32 write exceeds the file".to_string())?;
    field.copy_from_slice(&encoded);
    assert_eq!(field.len(), std::mem::size_of::<u32>());
    assert!(end <= bytes.len());
    Ok(())
}

fn write_elf_u64(bytes: &mut [u8], offset: usize, value: u64) -> Result<(), String> {
    let encoded = value.to_le_bytes();
    let end = offset.checked_add(encoded.len()).ok_or_else(|| "Flex ELF u64 write offset overflow".to_string())?;
    let field = bytes.get_mut(offset..end).ok_or_else(|| "Flex ELF u64 write exceeds the file".to_string())?;
    field.copy_from_slice(&encoded);
    assert_eq!(field.len(), std::mem::size_of::<u64>());
    assert!(end <= bytes.len());
    Ok(())
}

fn elf_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("Flex ELF {label} exceeds this platform"))
}

fn elf_section_header_offset(table_offset: usize, entry_size: usize, index: usize) -> Result<usize, String> {
    let relative =
        index.checked_mul(entry_size).ok_or_else(|| "Flex ELF section-header offset overflow".to_string())?;
    table_offset
        .checked_add(relative)
        .ok_or_else(|| "Flex ELF section-table offset overflow".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlexElfSections {
    table_offset: usize,
    header_size: usize,
    count: usize,
    names_header: usize,
    names_offset: usize,
    names_end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlexRuntimeSectionName {
    start: usize,
    len: usize,
}

fn validate_flex_elf_header(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < ELF64_HEADER_BYTES || bytes.get(..ELF_MAGIC_BYTES) != Some(ELF_MAGIC) {
        return Err("Flex runtime section canonicalizer requires an ELF file".to_string());
    }
    if bytes[ELF_IDENT_CLASS_OFFSET] != ELF_CLASS_64 || bytes[ELF_IDENT_DATA_OFFSET] != ELF_DATA_LITTLE_ENDIAN {
        return Err("Flex runtime section canonicalizer requires little-endian ELF64".to_string());
    }
    if read_elf_u16(bytes, ELF_TYPE_OFFSET) != Some(ELF_TYPE_EXECUTABLE) {
        return Err("Flex runtime section canonicalizer requires an executable".to_string());
    }
    if read_elf_u16(bytes, ELF_MACHINE_OFFSET) != Some(ELF_MACHINE_X86_64) {
        return Err("Flex runtime section canonicalizer requires x86_64".to_string());
    }
    assert!(bytes.len() >= ELF64_HEADER_BYTES);
    assert_eq!(bytes[ELF_IDENT_CLASS_OFFSET], ELF_CLASS_64);
    Ok(())
}

fn parse_flex_elf_sections(bytes: &[u8]) -> Result<FlexElfSections, String> {
    let table_offset = elf_usize(
        read_elf_u64(bytes, ELF_SECTION_TABLE_OFFSET)
            .ok_or_else(|| "Flex ELF lacks a section-table offset".to_string())?,
        "section-table offset",
    )?;
    let header_size = usize::from(
        read_elf_u16(bytes, ELF_SECTION_HEADER_SIZE_OFFSET)
            .ok_or_else(|| "Flex ELF lacks a section-header size".to_string())?,
    );
    let count = usize::from(
        read_elf_u16(bytes, ELF_SECTION_COUNT_OFFSET).ok_or_else(|| "Flex ELF lacks a section count".to_string())?,
    );
    let names_index = usize::from(
        read_elf_u16(bytes, ELF_SECTION_NAMES_INDEX_OFFSET)
            .ok_or_else(|| "Flex ELF lacks a section-name table index".to_string())?,
    );
    if header_size != ELF64_SECTION_HEADER_BYTES || count == 0 || names_index >= count {
        return Err("Flex ELF has an invalid bounded section table".to_string());
    }
    let table_bytes =
        count.checked_mul(header_size).ok_or_else(|| "Flex ELF section-table size overflow".to_string())?;
    let table_end = table_offset
        .checked_add(table_bytes)
        .ok_or_else(|| "Flex ELF section-table range overflow".to_string())?;
    if table_end > bytes.len() {
        return Err("Flex ELF section table exceeds the file".to_string());
    }
    let names_header = elf_section_header_offset(table_offset, header_size, names_index)?;
    if read_elf_u32(bytes, names_header + ELF_SECTION_TYPE_OFFSET) != Some(ELF_SECTION_TYPE_STRING_TABLE) {
        return Err("Flex ELF section-name table has the wrong type".to_string());
    }
    let names_offset = elf_usize(
        read_elf_u64(bytes, names_header + ELF_SECTION_DATA_OFFSET)
            .ok_or_else(|| "Flex ELF lacks a section-name offset".to_string())?,
        "section-name offset",
    )?;
    let names_size = elf_usize(
        read_elf_u64(bytes, names_header + ELF_SECTION_DATA_SIZE_OFFSET)
            .ok_or_else(|| "Flex ELF lacks a section-name size".to_string())?,
        "section-name size",
    )?;
    let names_end = names_offset
        .checked_add(names_size)
        .ok_or_else(|| "Flex ELF section-name range overflow".to_string())?;
    if names_offset >= names_end {
        return Err("Flex ELF section-name table is empty or reversed".to_string());
    }
    if names_end > table_offset {
        return Err("Flex ELF section-name table must precede the section headers".to_string());
    }
    assert!(table_end <= bytes.len());
    assert!(names_end <= table_offset);
    Ok(FlexElfSections {
        table_offset,
        header_size,
        count,
        names_header,
        names_offset,
        names_end,
    })
}

fn validate_flex_section_name_offsets(bytes: &[u8], sections: FlexElfSections) -> Result<(), String> {
    let names_size = sections
        .names_end
        .checked_sub(sections.names_offset)
        .ok_or_else(|| "Flex ELF section-name range is reversed".to_string())?;
    for section_index in 0..sections.count {
        let header = elf_section_header_offset(sections.table_offset, sections.header_size, section_index)?;
        let offset = usize::try_from(
            read_elf_u32(bytes, header + ELF_SECTION_NAME_OFFSET)
                .ok_or_else(|| "Flex ELF section lacks a name offset".to_string())?,
        )
        .map_err(|_| "Flex ELF section-name offset exceeds this platform".to_string())?;
        if offset >= names_size {
            return Err(format!("Flex ELF section {section_index} name offset exceeds the string table"));
        }
        let start = sections
            .names_offset
            .checked_add(offset)
            .ok_or_else(|| "Flex ELF section-name offset overflow".to_string())?;
        if offset > 0 && bytes[start - 1] != 0 {
            return Err(format!("Flex ELF section {section_index} name does not start at a string boundary"));
        }
        if !bytes[start..sections.names_end].contains(&0) {
            return Err(format!("Flex ELF section {section_index} name is not terminated"));
        }
    }
    assert!(names_size > 0);
    assert_eq!(bytes[sections.names_offset], 0);
    Ok(())
}

fn flex_runtime_section_name(
    bytes: &[u8],
    sections: FlexElfSections,
    header: usize,
) -> Result<Option<FlexRuntimeSectionName>, String> {
    if read_elf_u32(bytes, header + ELF_SECTION_TYPE_OFFSET) != Some(ELF_SECTION_TYPE_PROGBITS) {
        return Ok(None);
    }
    if read_elf_u64(bytes, header + ELF_SECTION_FLAGS_OFFSET) != Some(ELF_SECTION_FLAG_ALLOC) {
        return Ok(None);
    }
    if read_elf_u64(bytes, header + ELF_SECTION_ALIGNMENT_OFFSET) != Some(FLEX_RUNTIME_SECTION_ALIGNMENT) {
        return Ok(None);
    }
    let data_offset = elf_usize(
        read_elf_u64(bytes, header + ELF_SECTION_DATA_OFFSET)
            .ok_or_else(|| "Flex ELF section lacks a data offset".to_string())?,
        "section data offset",
    )?;
    let data_size = elf_usize(
        read_elf_u64(bytes, header + ELF_SECTION_DATA_SIZE_OFFSET)
            .ok_or_else(|| "Flex ELF section lacks a data size".to_string())?,
        "section data size",
    )?;
    let data_end = data_offset
        .checked_add(data_size)
        .ok_or_else(|| "Flex ELF section data range overflow".to_string())?;
    if data_size != FLEX_RUNTIME_SECTION_BYTES_LEN {
        return Ok(None);
    }
    if data_end > bytes.len() {
        return Ok(None);
    }
    if bytes[data_offset..data_end] != FLEX_RUNTIME_SECTION_BYTES {
        return Ok(None);
    }
    let name_offset = usize::try_from(
        read_elf_u32(bytes, header + ELF_SECTION_NAME_OFFSET)
            .ok_or_else(|| "Flex ELF section lacks a name offset".to_string())?,
    )
    .map_err(|_| "Flex ELF section-name offset exceeds this platform".to_string())?;
    let start = sections
        .names_offset
        .checked_add(name_offset)
        .ok_or_else(|| "Flex ELF runtime section-name offset overflow".to_string())?;
    let len = bytes[start..sections.names_end]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| "Flex ELF runtime section name is not terminated".to_string())?;
    if !(FLEX_RUNTIME_SECTION_NAME_BYTES_LEN..=FLEX_RUNTIME_SECTION_NAME_BYTES_LEN_MAX).contains(&len) {
        return Err(format!(
            "Flex ELF runtime section name has {len} bytes; expected {}..={}",
            FLEX_RUNTIME_SECTION_NAME_BYTES_LEN, FLEX_RUNTIME_SECTION_NAME_BYTES_LEN_MAX
        ));
    }
    assert_eq!(data_size, FLEX_RUNTIME_SECTION_BYTES_LEN);
    assert!(start < sections.names_end);
    Ok(Some(FlexRuntimeSectionName { start, len }))
}

fn find_flex_runtime_section_name(bytes: &[u8], sections: FlexElfSections) -> Result<FlexRuntimeSectionName, String> {
    let mut found = None;
    let mut match_count = 0usize;
    for section_index in 0..sections.count {
        let header = elf_section_header_offset(sections.table_offset, sections.header_size, section_index)?;
        let Some(name) = flex_runtime_section_name(bytes, sections, header)? else {
            continue;
        };
        found = Some(name);
        match_count = match_count
            .checked_add(1)
            .ok_or_else(|| "Flex ELF runtime section match count overflow".to_string())?;
    }
    if match_count != FLEX_RUNTIME_SECTION_MATCH_COUNT {
        return Err(format!(
            "Flex ELF has {match_count} matching runtime sections; expected {FLEX_RUNTIME_SECTION_MATCH_COUNT}"
        ));
    }
    assert_eq!(match_count, FLEX_RUNTIME_SECTION_MATCH_COUNT);
    found.ok_or_else(|| "Flex ELF runtime section match disappeared".to_string())
}

fn shift_flex_section_name_offsets(
    bytes: &mut [u8],
    sections: FlexElfSections,
    removed_offset: usize,
    removed_bytes: usize,
) -> Result<(), String> {
    for section_index in 0..sections.count {
        let header = elf_section_header_offset(sections.table_offset, sections.header_size, section_index)?;
        let offset = usize::try_from(
            read_elf_u32(bytes, header + ELF_SECTION_NAME_OFFSET)
                .ok_or_else(|| "Flex ELF section lacks a name offset".to_string())?,
        )
        .map_err(|_| "Flex ELF section-name offset exceeds this platform".to_string())?;
        if offset > removed_offset {
            let shifted = offset
                .checked_sub(removed_bytes)
                .ok_or_else(|| "Flex ELF section-name shift underflow".to_string())?;
            let encoded =
                u32::try_from(shifted).map_err(|_| "Flex ELF shifted section-name offset exceeds u32".to_string())?;
            write_elf_u32(bytes, header + ELF_SECTION_NAME_OFFSET, encoded)?;
        }
    }
    assert!(removed_bytes > 0);
    assert!(removed_offset < sections.names_end - sections.names_offset);
    Ok(())
}

fn canonicalize_flex_section_name_table(
    bytes: &[u8],
    sections: FlexElfSections,
    runtime_name: FlexRuntimeSectionName,
) -> Result<Vec<u8>, String> {
    let removed_bytes = runtime_name
        .len
        .checked_sub(FLEX_RUNTIME_SECTION_NAME_BYTES_LEN)
        .ok_or_else(|| "Flex ELF runtime section-name shrink underflow".to_string())?;
    let canonical_name_end = runtime_name
        .start
        .checked_add(FLEX_RUNTIME_SECTION_NAME_BYTES_LEN)
        .ok_or_else(|| "Flex ELF canonical runtime section-name range overflow".to_string())?;
    let source_name_bytes = runtime_name
        .len
        .checked_add(1)
        .ok_or_else(|| "Flex ELF runtime section-name length overflow".to_string())?;
    let source_suffix_start = runtime_name
        .start
        .checked_add(source_name_bytes)
        .ok_or_else(|| "Flex ELF runtime section-name suffix overflow".to_string())?;
    let target_suffix_start = canonical_name_end
        .checked_add(1)
        .ok_or_else(|| "Flex ELF canonical runtime section-name suffix overflow".to_string())?;
    let new_names_end = sections
        .names_end
        .checked_sub(removed_bytes)
        .ok_or_else(|| "Flex ELF canonical section-name table underflow".to_string())?;
    let mut canonical = bytes.to_vec();
    canonical[runtime_name.start..canonical_name_end].copy_from_slice(FLEX_RUNTIME_SECTION_NAME);
    canonical[canonical_name_end] = 0;
    if removed_bytes > 0 {
        canonical.copy_within(source_suffix_start..sections.names_end, target_suffix_start);
        canonical[new_names_end..sections.names_end].fill(0);
        let removed_offset = canonical_name_end
            .checked_sub(sections.names_offset)
            .ok_or_else(|| "Flex ELF removed section-name offset underflow".to_string())?;
        shift_flex_section_name_offsets(&mut canonical, sections, removed_offset, removed_bytes)?;
        let new_names_size = new_names_end
            .checked_sub(sections.names_offset)
            .ok_or_else(|| "Flex ELF canonical section-name table range is reversed".to_string())?;
        let new_names_size = u64::try_from(new_names_size)
            .map_err(|_| "Flex ELF canonical section-name table size exceeds u64".to_string())?;
        write_elf_u64(&mut canonical, sections.names_header + ELF_SECTION_DATA_SIZE_OFFSET, new_names_size)?;
    }
    assert_eq!(canonical.len(), bytes.len());
    assert!(new_names_end <= sections.names_end);
    Ok(canonical)
}

fn validate_canonical_flex_runtime_section(bytes: &[u8]) -> Result<(), String> {
    let sections = parse_flex_elf_sections(bytes)?;
    validate_flex_section_name_offsets(bytes, sections)?;
    let runtime_name = find_flex_runtime_section_name(bytes, sections)?;
    let name_end = runtime_name
        .start
        .checked_add(FLEX_RUNTIME_SECTION_NAME_BYTES_LEN)
        .ok_or_else(|| "Flex ELF canonical runtime section-name range overflow".to_string())?;
    if runtime_name.len != FLEX_RUNTIME_SECTION_NAME_BYTES_LEN {
        return Err("Flex ELF canonical runtime section name did not converge".to_string());
    }
    if bytes[runtime_name.start..name_end] != *FLEX_RUNTIME_SECTION_NAME {
        return Err("Flex ELF canonical runtime section bytes did not converge".to_string());
    }
    assert_eq!(runtime_name.len, FLEX_RUNTIME_SECTION_NAME_BYTES_LEN);
    assert_eq!(&bytes[runtime_name.start..name_end], FLEX_RUNTIME_SECTION_NAME);
    Ok(())
}

fn canonicalize_flex_runtime_section_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    validate_flex_elf_header(bytes)?;
    let sections = parse_flex_elf_sections(bytes)?;
    validate_flex_section_name_offsets(bytes, sections)?;
    let runtime_name = find_flex_runtime_section_name(bytes, sections)?;
    let canonical = canonicalize_flex_section_name_table(bytes, sections, runtime_name)?;
    validate_canonical_flex_runtime_section(&canonical)?;
    let canonical_sections = parse_flex_elf_sections(&canonical)?;
    let canonical_name = find_flex_runtime_section_name(&canonical, canonical_sections)?;
    let repeated = canonicalize_flex_section_name_table(&canonical, canonical_sections, canonical_name)?;
    assert_eq!(canonical.len(), bytes.len());
    assert_eq!(repeated, canonical);
    Ok(canonical)
}

fn canonicalize_flex_runtime_section_file(path: &Path) -> Result<(), StagexFlexError> {
    fs::set_permissions(path, fs::Permissions::from_mode(FLEX_REGULAR_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting Flex canonicalization mode: {error}")))?;
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FLEX_ARTIFACT_BYTES_MAX, "Flex executable")?;
    let canonical = canonicalize_flex_runtime_section_bytes(&bytes)
        .map_err(|error| StagexFlexError::Materialization(format!("canonicalizing Flex runtime section: {error}")))?;
    fs::write(path, &canonical)
        .map_err(|error| StagexFlexError::Materialization(format!("writing canonical Flex executable: {error}")))?;
    if fs::read(path).unwrap_or_default() != canonical {
        return Err(StagexFlexError::Materialization(
            "canonical Flex executable did not persist byte-for-byte".to_string(),
        ));
    }
    assert_eq!(canonical.len(), bytes.len());
    assert!(path.is_file());
    Ok(())
}

fn build_flex(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
    output_lib: &Path,
    output_include: &Path,
) -> Result<FlexProducts, StagexFlexError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let include_path = request.musl_native_root.join("include");
    let include = utf8_absolute(&include_path, "native musl include")?;
    let common_args = [
        "-I.".to_string(),
        "-Isrc".to_string(),
        format!("-I{include}"),
        "-DHAVE_CONFIG_H".to_string(),
    ];
    let mut objects = Vec::with_capacity(FLEX_OBJECT_COUNT);
    for source_name in FLEX_PROGRAM_SOURCES {
        objects.push(compile_flex_source(request, source_root, &compiler, &common_args, source_name)?);
    }
    objects.push(compile_flex_compatibility(request, source_root, &compiler, &common_args)?);
    let flex = link_flex(request, source_root, output_bin, &compiler, &objects)?;
    install_flex_aliases(output_bin)?;
    let header = install_flex_header(source_root, output_include)?;
    let (libfl, consumer) = build_libfl(request, source_root, output_lib, &compiler)?;
    assert_eq!(objects.len(), FLEX_OBJECT_COUNT);
    assert!(consumer.is_file());
    Ok(FlexProducts {
        flex,
        libfl,
        header,
        consumer,
    })
}

fn compile_flex_source(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    common_args: &[String],
    source_name: &str,
) -> Result<String, StagexFlexError> {
    let source = format!("src/{source_name}.c");
    let object = format!("src/{source_name}.o");
    let mut args = vec!["-c".to_string()];
    args.extend_from_slice(common_args);
    args.push(source.clone());
    args.push("-o".to_string());
    args.push(object.clone());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("flex-{source_name}-compile.stderr.txt")),
    )?;
    validate_nonempty_artifact(&source_root.join(&object), "Flex object")?;
    assert!(source_root.join(source).is_file());
    assert!(source_root.join(&object).is_file());
    Ok(object)
}

fn compile_flex_compatibility(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    common_args: &[String],
) -> Result<String, StagexFlexError> {
    let object = "mantle_flex_compat.o".to_string();
    let mut args = vec!["-c".to_string()];
    args.extend_from_slice(common_args);
    args.push("mantle_flex_compat.c".to_string());
    args.push("-o".to_string());
    args.push(object.clone());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("flex-compat-compile.stderr.txt"),
    )?;
    validate_nonempty_artifact(&source_root.join(&object), "Flex compatibility object")?;
    assert!(source_root.join("mantle_flex_compat.c").is_file());
    assert!(source_root.join(&object).is_file());
    Ok(object)
}

fn link_flex(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
    compiler: &Path,
    objects: &[String],
) -> Result<PathBuf, StagexFlexError> {
    let flex = output_bin.join("flex");
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_absolute(&flex, "Flex output")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("flex-link.stderr.txt"),
    )?;
    canonicalize_flex_runtime_section_file(&flex)?;
    fs::set_permissions(&flex, fs::Permissions::from_mode(FLEX_EXECUTABLE_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting Flex executable mode: {error}")))?;
    validate_nonempty_artifact(&flex, "Flex executable")?;
    assert_eq!(objects.len(), FLEX_OBJECT_COUNT);
    assert!(
        flex.metadata()
            .map(|value| value.permissions().mode() & FLEX_EXECUTE_MODE_MASK != 0)
            .unwrap_or(false)
    );
    Ok(flex)
}

fn install_flex_aliases(output_bin: &Path) -> Result<(), StagexFlexError> {
    std::os::unix::fs::symlink("flex", output_bin.join("lex"))
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex lex alias: {error}")))?;
    std::os::unix::fs::symlink("flex", output_bin.join("flex++"))
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex C++ alias: {error}")))?;
    if !output_bin.join("lex").is_symlink() || !output_bin.join("flex++").is_symlink() {
        return Err(StagexFlexError::Materialization("Flex aliases are missing".to_string()));
    }
    assert!(output_bin.join("lex").is_symlink());
    assert!(output_bin.join("flex++").is_symlink());
    Ok(())
}

fn install_flex_header(source_root: &Path, output_include: &Path) -> Result<PathBuf, StagexFlexError> {
    let source = source_root.join("src/FlexLexer.h");
    let output = output_include.join("FlexLexer.h");
    let bytes = crate::stagex_mes_lib::read_bounded_file(&source, FLEX_ARTIFACT_BYTES_MAX, "Flex C++ header")?;
    crate::stagex_mes_lib::write_create_new(&output, &bytes)?;
    fs::set_permissions(&output, fs::Permissions::from_mode(FLEX_REGULAR_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting Flex C++ header mode: {error}")))?;
    validate_nonempty_artifact(&output, "Flex C++ header")?;
    assert_eq!(fs::read(&output).unwrap_or_default(), bytes);
    assert!(output.is_file());
    Ok(output)
}

fn build_libfl(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    output_lib: &Path,
    compiler: &Path,
) -> Result<(PathBuf, PathBuf), StagexFlexError> {
    let main_source = source_root.join("stagex-libfl-main.c");
    let main_object = source_root.join("stagex-libfl-main.o");
    crate::stagex_mes_lib::write_create_new(&main_source, b"int yywrap(void) { return 1; }\n")?;
    compile_absolute_source(request, source_root, compiler, &main_source, &main_object, "libfl-main")?;
    fs::set_permissions(&main_object, fs::Permissions::from_mode(FLEX_REGULAR_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting libfl object mode: {error}")))?;
    let libfl = output_lib.join("libfl.a");
    write_single_member_archive(&main_object, &libfl, "libfl_main.o/")?;
    let consumer_source = source_root.join("stagex-libfl-consumer.c");
    let consumer_object = source_root.join("stagex-libfl-consumer.o");
    let consumer = request.scratch_dir.join("libfl-consumer");
    crate::stagex_mes_lib::write_create_new(
        &consumer_source,
        b"int yywrap(void);\nint main(void) { return yywrap() == 1 ? 0 : 1; }\n",
    )?;
    compile_absolute_source(request, source_root, compiler, &consumer_source, &consumer_object, "libfl-consumer")?;
    fs::set_permissions(&consumer_object, fs::Permissions::from_mode(FLEX_REGULAR_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting libfl consumer object mode: {error}")))?;
    link_libfl_consumer(request, source_root, compiler, &consumer_object, &libfl, &consumer)?;
    assert!(libfl.is_file());
    assert!(consumer.is_file());
    Ok((libfl, consumer))
}

fn compile_absolute_source(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    source: &Path,
    object: &Path,
    label: &str,
) -> Result<(), StagexFlexError> {
    let args = vec![
        "-c".to_string(),
        format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?),
        source
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| StagexFlexError::Materialization("Flex auxiliary source name is invalid".to_string()))?
            .to_string(),
        "-o".to_string(),
        object
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| StagexFlexError::Materialization("Flex auxiliary object name is invalid".to_string()))?
            .to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("{label}-compile.stderr.txt")),
    )?;
    validate_nonempty_artifact(object, "Flex auxiliary object")?;
    assert!(source.is_file());
    assert!(object.is_file());
    Ok(())
}

fn write_single_member_archive(object: &Path, archive: &Path, member_name: &str) -> Result<(), StagexFlexError> {
    const NAME_WIDTH: usize = 16;
    const TIMESTAMP_WIDTH: usize = 12;
    const IDENTIFIER_WIDTH: usize = 6;
    const MODE_WIDTH: usize = 8;
    const SIZE_WIDTH: usize = 10;
    const ALIGNMENT_BYTES: usize = 2;
    let object_bytes = crate::stagex_mes_lib::read_bounded_file(object, FLEX_ARTIFACT_BYTES_MAX, "libfl object")?;
    if member_name.len() > NAME_WIDTH || object_bytes.is_empty() {
        return Err(StagexFlexError::Materialization("invalid libfl archive member".to_string()));
    }
    const ARCHIVE_HEADER_CAPACITY_BYTES: usize = 80;
    let mut bytes = Vec::with_capacity(object_bytes.len().saturating_add(ARCHIVE_HEADER_CAPACITY_BYTES));
    bytes.extend_from_slice(b"!<arch>\n");
    append_fixed_field(&mut bytes, member_name, NAME_WIDTH)?;
    append_fixed_field(&mut bytes, "0", TIMESTAMP_WIDTH)?;
    append_fixed_field(&mut bytes, "0", IDENTIFIER_WIDTH)?;
    append_fixed_field(&mut bytes, "0", IDENTIFIER_WIDTH)?;
    append_fixed_field(&mut bytes, "100644", MODE_WIDTH)?;
    append_fixed_field(&mut bytes, &object_bytes.len().to_string(), SIZE_WIDTH)?;
    bytes.extend_from_slice(b"`\n");
    bytes.extend_from_slice(&object_bytes);
    if object_bytes.len() % ALIGNMENT_BYTES != 0 {
        bytes.push(b'\n');
    }
    crate::stagex_mes_lib::write_create_new(archive, &bytes)?;
    fs::set_permissions(archive, fs::Permissions::from_mode(FLEX_REGULAR_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting libfl archive mode: {error}")))?;
    assert!(bytes.starts_with(b"!<arch>\n"));
    assert!(bytes.len() > object_bytes.len());
    Ok(())
}

fn append_fixed_field(bytes: &mut Vec<u8>, value: &str, width: usize) -> Result<(), StagexFlexError> {
    if value.len() > width {
        return Err(StagexFlexError::Materialization(format!("libfl archive field exceeds width {width}: {value}")));
    }
    bytes.extend_from_slice(value.as_bytes());
    bytes.resize(bytes.len().saturating_add(width - value.len()), b' ');
    assert!(value.len() <= width);
    assert!(bytes.len() >= width);
    Ok(())
}

fn link_libfl_consumer(
    request: &FlexInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    object: &Path,
    archive: &Path,
    output: &Path,
) -> Result<(), StagexFlexError> {
    let args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_absolute(output, "libfl consumer")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
        object
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| StagexFlexError::Materialization("libfl consumer object name is invalid".to_string()))?
            .to_string(),
        utf8_absolute(archive, "libfl archive")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("libfl-consumer-link.stderr.txt"),
    )?;
    fs::set_permissions(output, fs::Permissions::from_mode(FLEX_EXECUTABLE_MODE))
        .map_err(|error| StagexFlexError::Materialization(format!("setting libfl consumer mode: {error}")))?;
    validate_nonempty_artifact(output, "libfl consumer")?;
    assert!(object.is_file());
    assert!(archive.is_file());
    Ok(())
}

struct FlexObservations {
    version: PathBuf,
    positive_source: PathBuf,
    negative: PathBuf,
    negative_exit_code: i32,
}

fn run_checked_smokes(
    request: &FlexInventoryRequest<'_>,
    products: &FlexProducts,
) -> Result<FlexObservations, StagexFlexError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexFlexError::Materialization(format!("creating Flex smoke root: {error}")))?;
    let environment = flex_environment(request)?;
    let consumer = run_flex_success(&products.consumer, &smoke, "libfl-consumer", &[], b"", &BTreeMap::new())?;
    require_exact_bytes(&consumer, b"", "libfl consumer stdout")?;
    let version = run_flex_success(&products.flex, &smoke, "version", &["--version"], b"", &environment)?;
    require_contains(&version, b"flex 2.6.4", "Flex version observation")?;
    let positive_l = smoke.join("positive.l");
    crate::stagex_mes_lib::write_create_new(&positive_l, b"%%\nhello  ECHO;\n%%\nint yywrap(void) { return 1; }\n")?;
    let positive_stdout =
        run_flex_success(&products.flex, &smoke, "positive", &["-o", "positive.c", "positive.l"], b"", &environment)?;
    require_exact_bytes(&positive_stdout, b"", "Flex positive stdout")?;
    let positive_source = smoke.join("positive.c");
    validate_nonempty_artifact(&positive_source, "Flex positive scanner source")?;
    let (negative, negative_exit_code) = run_negative_smoke(&products.flex, &smoke, &environment)?;
    assert_ne!(negative_exit_code, 0);
    assert!(negative.is_file());
    Ok(FlexObservations {
        version,
        positive_source,
        negative,
        negative_exit_code,
    })
}

fn run_flex_success(
    executable: &Path,
    smoke: &Path,
    label: &str,
    arguments: &[&str],
    stdin_bytes: &[u8],
    environment: &BTreeMap<String, String>,
) -> Result<PathBuf, StagexFlexError> {
    let stdin_path = smoke.join(format!("{label}.in"));
    let stdout_path = smoke.join(format!("{label}.out"));
    let stderr_path = smoke.join(format!("{label}.stderr.txt"));
    crate::stagex_mes_lib::write_create_new(&stdin_path, stdin_bytes)?;
    let args = arguments.iter().map(|argument| (*argument).to_string()).collect::<Vec<_>>();
    crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout(
        executable,
        &args,
        smoke,
        environment,
        &stdin_path,
        FLEX_OBSERVATION_BYTES_MAX,
        &stdout_path,
        FLEX_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(stdout_path)
}

fn run_negative_smoke(
    flex: &Path,
    smoke: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<(PathBuf, i32), StagexFlexError> {
    let scanner = smoke.join("negative.l");
    let stdin_path = smoke.join("negative.in");
    let stdout_path = smoke.join("negative.stdout.txt");
    let stderr_path = smoke.join("negative.stderr.txt");
    crate::stagex_mes_lib::write_create_new(&scanner, b"%%\n[unterminated  ECHO;\n%%\n")?;
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"")?;
    let args = ["-o".to_string(), "negative.c".to_string(), "negative.l".to_string()];
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        flex,
        &args,
        smoke,
        environment,
        &stdin_path,
        FLEX_OBSERVATION_BYTES_MAX,
        &stdout_path,
        FLEX_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    let retained_bytes = smoke.join("negative.c").metadata().map(|value| value.len()).unwrap_or(0);
    if exit_code == 0 || retained_bytes != 0 {
        return Err(StagexFlexError::Materialization("invalid Flex scanner succeeded or retained output".to_string()));
    }
    require_exact_bytes(&stdout_path, b"", "Flex negative stdout")?;
    validate_nonempty_artifact(&stderr_path, "Flex negative diagnostic")?;
    assert_ne!(exit_code, 0);
    assert_eq!(retained_bytes, 0);
    Ok((stderr_path, exit_code))
}

fn flex_environment(request: &FlexInventoryRequest<'_>) -> Result<BTreeMap<String, String>, StagexFlexError> {
    let mut environment = BTreeMap::new();
    environment
        .insert("M4".to_string(), utf8_absolute(&request.m4_root.join("bin/m4"), "protected GNU M4")?.to_string());
    environment.insert("LC_ALL".to_string(), "C".to_string());
    assert_eq!(environment.len(), 2);
    assert_eq!(environment.get("LC_ALL").map(String::as_str), Some("C"));
    Ok(environment)
}

fn collect_outputs(
    products: &FlexProducts,
    observations: &FlexObservations,
) -> Result<Vec<FlexOutputReport>, StagexFlexError> {
    let specs = [
        ("flex-2.6.4", products.flex.as_path()),
        ("flex-libfl", products.libfl.as_path()),
        ("flex-cxx-header", products.header.as_path()),
        ("flex-libfl-consumer", products.consumer.as_path()),
        ("flex-version-observation", observations.version.as_path()),
        ("flex-positive-scanner-source", observations.positive_source.as_path()),
        ("flex-negative-observation", observations.negative.as_path()),
    ];
    let outputs = specs
        .iter()
        .map(|(artifact_id, path)| output_report(artifact_id, path))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), FLEX_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[FlexOutputReport]) -> Result<(), StagexFlexError> {
    if outputs.len() != FLEX_EXPECTED_OUTPUTS.len() {
        return Err(StagexFlexError::Materialization(format!(
            "expected {} Flex outputs, observed {}",
            FLEX_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in FLEX_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexFlexError::Materialization(format!("Flex output is missing: {}", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            return Err(StagexFlexError::Materialization(format!(
                "Flex output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, observed.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), FLEX_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(())
}

fn output_report(artifact_id: &str, path: &Path) -> Result<FlexOutputReport, StagexFlexError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FLEX_ARTIFACT_BYTES_MAX, artifact_id)?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexFlexError::Materialization(format!("Flex artifact is too large: {artifact_id}")))?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert!(bytes_len <= FLEX_ARTIFACT_BYTES_MAX);
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(FlexOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexFlexError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexFlexError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() > FLEX_OBSERVATION_BYTES_MAX {
        return Err(StagexFlexError::Materialization(format!(
            "{label} is not regular or exceeds {FLEX_OBSERVATION_BYTES_MAX} bytes"
        )));
    }
    let observed =
        fs::read(path).map_err(|error| StagexFlexError::Materialization(format!("reading {label}: {error}")))?;
    if observed != expected {
        return Err(StagexFlexError::Materialization(format!("{label} bytes differ")));
    }
    assert_eq!(u64::try_from(observed.len()).unwrap_or(u64::MAX), metadata.len());
    assert_eq!(observed, expected);
    Ok(())
}

fn require_contains(path: &Path, needle: &[u8], label: &str) -> Result<(), StagexFlexError> {
    let observed = crate::stagex_mes_lib::read_bounded_file(path, FLEX_OBSERVATION_BYTES_MAX, label)?;
    if !observed.windows(needle.len()).any(|window| window == needle) {
        return Err(StagexFlexError::Materialization(format!("{label} lacks required bytes")));
    }
    assert!(!needle.is_empty());
    assert!(!observed.is_empty());
    Ok(())
}

fn validate_nonempty_artifact(path: &Path, label: &str) -> Result<(), StagexFlexError> {
    let metadata = path
        .metadata()
        .map_err(|error| StagexFlexError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > FLEX_ARTIFACT_BYTES_MAX {
        return Err(StagexFlexError::Materialization(format!("invalid {label}: {}", path.display())));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() <= FLEX_ARTIFACT_BYTES_MAX);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexFlexError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FLEX_ARTIFACT_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexFlexError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(())
}

fn native_musl_digest(artifact_id: &str) -> Result<&'static str, StagexFlexError> {
    let digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.digest_blake3)
        .ok_or_else(|| StagexFlexError::Materialization(format!("native musl inventory is missing {artifact_id}")))?;
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexFlexError> {
    if !path.is_absolute() {
        return Err(StagexFlexError::Materialization(format!("{label} path is not absolute: {}", path.display())));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexFlexError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const SOURCE_MANIFEST_BLAKE3: &str = "541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab";
    const SYNTHETIC_SECTION_COUNT: usize = 3;
    const SYNTHETIC_TARGET_SECTION_INDEX: usize = 1;
    const SYNTHETIC_NAMES_SECTION_INDEX: usize = 2;
    const SYNTHETIC_NAME_OFFSET: u32 = 1;
    const SYNTHETIC_RANDOM_NAME: &[u8; FLEX_RUNTIME_SECTION_NAME_BYTES_LEN] = b"rnd";
    const SYNTHETIC_RANDOM_NAME_LONG: &[u8; FLEX_RUNTIME_SECTION_NAME_BYTES_LEN_MAX] = b"rand";
    const SYNTHETIC_SECTION_NAMES_SUFFIX: &[u8] = b".shstrtab\0";
    const SYNTHETIC_SECTION_NAMES_PADDING_BYTES: usize = 8;
    const SYNTHETIC_INVALID_NAME_EMPTY: &[u8] = b"";
    const SYNTHETIC_INVALID_NAME_ONE_BYTE: &[u8] = b"x";
    const SYNTHETIC_INVALID_NAME_TWO_BYTES: &[u8] = b"xy";
    const SYNTHETIC_INVALID_NAME_FIVE_BYTES: &[u8] = b"abcde";

    fn write_fixture_u16(bytes: &mut [u8], offset: usize, value: u16) {
        let encoded = value.to_le_bytes();
        bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
    }

    fn write_fixture_u32(bytes: &mut [u8], offset: usize, value: u32) {
        let encoded = value.to_le_bytes();
        bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
    }

    fn write_fixture_u64(bytes: &mut [u8], offset: usize, value: u64) {
        let encoded = value.to_le_bytes();
        bytes[offset..offset + encoded.len()].copy_from_slice(&encoded);
    }

    fn synthetic_flex_section_names(random_name: &[u8]) -> Vec<u8> {
        let mut names = Vec::with_capacity(
            usize::try_from(SYNTHETIC_NAME_OFFSET).unwrap()
                + random_name.len()
                + 1
                + SYNTHETIC_SECTION_NAMES_SUFFIX.len(),
        );
        names.push(0);
        names.extend_from_slice(random_name);
        names.push(0);
        names.extend_from_slice(SYNTHETIC_SECTION_NAMES_SUFFIX);
        assert_eq!(names[0], 0);
        assert!(names.ends_with(SYNTHETIC_SECTION_NAMES_SUFFIX));
        names
    }

    fn synthetic_flex_elf(random_name: &[u8]) -> Vec<u8> {
        let content_offset = ELF64_HEADER_BYTES;
        let names_offset = content_offset + FLEX_RUNTIME_SECTION_BYTES_LEN;
        let section_names = synthetic_flex_section_names(random_name);
        let names_capacity = usize::try_from(SYNTHETIC_NAME_OFFSET).unwrap()
            + FLEX_RUNTIME_SECTION_NAME_BYTES_LEN_MAX
            + 1
            + SYNTHETIC_SECTION_NAMES_SUFFIX.len();
        let table_offset = names_offset + names_capacity + SYNTHETIC_SECTION_NAMES_PADDING_BYTES;
        let table_bytes = SYNTHETIC_SECTION_COUNT * ELF64_SECTION_HEADER_BYTES;
        let mut bytes = vec![0u8; table_offset + table_bytes];
        bytes[..ELF_MAGIC_BYTES].copy_from_slice(ELF_MAGIC);
        bytes[ELF_IDENT_CLASS_OFFSET] = ELF_CLASS_64;
        bytes[ELF_IDENT_DATA_OFFSET] = ELF_DATA_LITTLE_ENDIAN;
        write_fixture_u16(&mut bytes, ELF_TYPE_OFFSET, ELF_TYPE_EXECUTABLE);
        write_fixture_u16(&mut bytes, ELF_MACHINE_OFFSET, ELF_MACHINE_X86_64);
        write_fixture_u64(&mut bytes, ELF_SECTION_TABLE_OFFSET, u64::try_from(table_offset).unwrap());
        write_fixture_u16(
            &mut bytes,
            ELF_SECTION_HEADER_SIZE_OFFSET,
            u16::try_from(ELF64_SECTION_HEADER_BYTES).unwrap(),
        );
        write_fixture_u16(&mut bytes, ELF_SECTION_COUNT_OFFSET, u16::try_from(SYNTHETIC_SECTION_COUNT).unwrap());
        write_fixture_u16(
            &mut bytes,
            ELF_SECTION_NAMES_INDEX_OFFSET,
            u16::try_from(SYNTHETIC_NAMES_SECTION_INDEX).unwrap(),
        );
        bytes[content_offset..names_offset].copy_from_slice(&FLEX_RUNTIME_SECTION_BYTES);
        bytes[names_offset..names_offset + section_names.len()].copy_from_slice(&section_names);
        let target = table_offset + SYNTHETIC_TARGET_SECTION_INDEX * ELF64_SECTION_HEADER_BYTES;
        write_fixture_u32(&mut bytes, target + ELF_SECTION_NAME_OFFSET, SYNTHETIC_NAME_OFFSET);
        write_fixture_u32(&mut bytes, target + ELF_SECTION_TYPE_OFFSET, ELF_SECTION_TYPE_PROGBITS);
        write_fixture_u64(&mut bytes, target + ELF_SECTION_FLAGS_OFFSET, ELF_SECTION_FLAG_ALLOC);
        write_fixture_u64(&mut bytes, target + ELF_SECTION_DATA_OFFSET, u64::try_from(content_offset).unwrap());
        write_fixture_u64(
            &mut bytes,
            target + ELF_SECTION_DATA_SIZE_OFFSET,
            u64::try_from(FLEX_RUNTIME_SECTION_BYTES_LEN).unwrap(),
        );
        write_fixture_u64(&mut bytes, target + ELF_SECTION_ALIGNMENT_OFFSET, FLEX_RUNTIME_SECTION_ALIGNMENT);
        let names = table_offset + SYNTHETIC_NAMES_SECTION_INDEX * ELF64_SECTION_HEADER_BYTES;
        let suffix_offset = usize::try_from(SYNTHETIC_NAME_OFFSET).unwrap() + random_name.len() + 1;
        write_fixture_u32(&mut bytes, names + ELF_SECTION_NAME_OFFSET, u32::try_from(suffix_offset).unwrap());
        write_fixture_u32(&mut bytes, names + ELF_SECTION_TYPE_OFFSET, ELF_SECTION_TYPE_STRING_TABLE);
        write_fixture_u64(&mut bytes, names + ELF_SECTION_DATA_OFFSET, u64::try_from(names_offset).unwrap());
        write_fixture_u64(
            &mut bytes,
            names + ELF_SECTION_DATA_SIZE_OFFSET,
            u64::try_from(section_names.len()).unwrap(),
        );
        assert_eq!(&bytes[names_offset + 1..][..random_name.len()], random_name);
        assert_eq!(bytes.len(), table_offset + table_bytes);
        bytes
    }

    #[test]
    fn canonicalizes_bounded_flex_runtime_names_to_one_identity() {
        let short = synthetic_flex_elf(SYNTHETIC_RANDOM_NAME);
        let long = synthetic_flex_elf(SYNTHETIC_RANDOM_NAME_LONG);
        let short_canonical = canonicalize_flex_runtime_section_bytes(&short).unwrap();
        let long_canonical = canonicalize_flex_runtime_section_bytes(&long).unwrap();
        let names_offset = ELF64_HEADER_BYTES + FLEX_RUNTIME_SECTION_BYTES_LEN;
        let name_start = names_offset + usize::try_from(SYNTHETIC_NAME_OFFSET).unwrap();
        let repeated = canonicalize_flex_runtime_section_bytes(&long_canonical).unwrap();

        assert_eq!(short_canonical, long_canonical);
        assert_eq!(
            &long_canonical[name_start..name_start + FLEX_RUNTIME_SECTION_NAME_BYTES_LEN],
            FLEX_RUNTIME_SECTION_NAME
        );
        assert_eq!(repeated, long_canonical);
    }

    #[test]
    fn rejects_unbounded_flex_runtime_section_name_lengths() {
        for invalid_name in [
            SYNTHETIC_INVALID_NAME_EMPTY,
            SYNTHETIC_INVALID_NAME_ONE_BYTE,
            SYNTHETIC_INVALID_NAME_TWO_BYTES,
            SYNTHETIC_INVALID_NAME_FIVE_BYTES,
        ] {
            let error = canonicalize_flex_runtime_section_bytes(&synthetic_flex_elf(invalid_name)).unwrap_err();
            assert!(error.contains("runtime section name has"));
            assert!(error.contains("expected 3..=4"));
        }
    }

    #[test]
    fn rejects_flex_section_name_offsets_outside_string_boundaries() {
        let mut input = synthetic_flex_elf(SYNTHETIC_RANDOM_NAME_LONG);
        let sections = parse_flex_elf_sections(&input).unwrap();
        let target =
            elf_section_header_offset(sections.table_offset, sections.header_size, SYNTHETIC_TARGET_SECTION_INDEX)
                .unwrap();
        let interior_offset = SYNTHETIC_NAME_OFFSET + 1;
        write_fixture_u32(&mut input, target + ELF_SECTION_NAME_OFFSET, interior_offset);

        let error = canonicalize_flex_runtime_section_bytes(&input).unwrap_err();

        assert!(error.contains("does not start at a string boundary"));
        assert!(error.contains("section 1"));
    }

    #[test]
    fn rejects_flex_section_name_offsets_outside_the_table() {
        let mut input = synthetic_flex_elf(SYNTHETIC_RANDOM_NAME);
        let sections = parse_flex_elf_sections(&input).unwrap();
        let target =
            elf_section_header_offset(sections.table_offset, sections.header_size, SYNTHETIC_TARGET_SECTION_INDEX)
                .unwrap();
        let names_size = sections.names_end - sections.names_offset;
        write_fixture_u32(&mut input, target + ELF_SECTION_NAME_OFFSET, u32::try_from(names_size).unwrap());

        let error = canonicalize_flex_runtime_section_bytes(&input).unwrap_err();

        assert!(error.contains("name offset exceeds the string table"));
        assert!(error.contains("section 1"));
    }

    #[test]
    fn rejects_substituted_flex_runtime_section_bytes() {
        let mut input = synthetic_flex_elf(SYNTHETIC_RANDOM_NAME);
        input[ELF64_HEADER_BYTES] ^= 1;

        let error = canonicalize_flex_runtime_section_bytes(&input).unwrap_err();

        assert!(error.contains("0 matching runtime sections"));
        assert!(error.contains("expected 1"));
    }

    #[test]
    fn canonicalization_shell_restores_read_permission() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("flex");
        fs::write(&path, synthetic_flex_elf(SYNTHETIC_RANDOM_NAME)).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(FLEX_NO_PERMISSION_MODE)).unwrap();

        canonicalize_flex_runtime_section_file(&path).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & FLEX_PERMISSION_MODE_MASK;
        let after = fs::read(&path).unwrap();
        assert_eq!(mode, FLEX_REGULAR_MODE);
        assert_eq!(canonicalize_flex_runtime_section_bytes(&after).unwrap(), after);
    }

    #[test]
    fn canonicalizes_retained_flex_runtime_identity() {
        let path = match std::env::var("MANTLE_STAGE_X_FLEX_RETAINED_BINARY") {
            Ok(value) => PathBuf::from(value),
            Err(_) => return,
        };
        let input = fs::read(path).unwrap();
        let canonical = canonicalize_flex_runtime_section_bytes(&input).unwrap();
        let digest = blake3::hash(&canonical).to_hex().to_string();

        assert_eq!(digest, FLEX_BINARY_BLAKE3);
        assert_ne!(canonical, input);
    }

    #[test]
    fn validates_exact_flex_source_record_and_recipe() {
        let bundle = match std::env::var(SOURCE_BUNDLE_ENV) {
            Ok(value) => value,
            Err(_) => return,
        };
        let manifest = read_source_bundle(Path::new(&bundle)).unwrap();
        let record = find_source_record(&manifest.records).unwrap();
        validate_source_record(record).unwrap();
        validate_recipe_digest().unwrap();
        assert_eq!(record.content_blake3, FLEX_SOURCE_CONTENT_BLAKE3);
        assert_eq!(source_artifact_digests().len(), FLEX_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_flex_source_authority() {
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
    fn applies_exact_flex_source_normalization() {
        let source_root = match std::env::var("MANTLE_STAGE_X_FLEX_RETAINED_SOURCE") {
            Ok(value) => PathBuf::from(value),
            Err(_) => return,
        };
        let filter = fs::read_to_string(source_root.join("src/filter.c")).unwrap();
        let misc = fs::read_to_string(source_root.join("src/misc.c")).unwrap();
        let main = fs::read_to_string(source_root.join("src/main.c")).unwrap();
        let buf = fs::read_to_string(source_root.join("src/buf.c")).unwrap();
        assert!(transform_filter_source(&filter).unwrap().contains("flex_exec_declared"));
        assert!(transform_misc_source(&misc).unwrap().contains("FLEX_ARRAY_ALLOCATION_BYTES_MAX"));
        assert!(transform_main_source(&main).unwrap().contains("flex_version_output"));
        assert!(transform_buf_source(&buf).unwrap().contains("FLEX_DECIMAL_RENDER_CAPACITY"));
    }

    #[test]
    fn pins_deterministic_flex_configured_source_identity() {
        let observed = configured_source_digest_blake3().unwrap();
        assert_eq!(observed, FLEX_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(FLEX_EXPECTED_OUTPUTS.len(), FLEX_OUTPUT_COUNT);
        assert_ne!(FLEX_RECIPE_BLAKE3, FLEX_SOURCE_CONTENT_BLAKE3);
    }

    #[test]
    fn rejects_substituted_flex_output_identity() {
        let mut outputs = FLEX_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| FlexOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        outputs[0].digest_blake3 = "f".repeat(BLAKE3_HEX_CHAR_COUNT);
        let error = validate_expected_outputs(&outputs).unwrap_err().to_string();
        assert!(error.contains("BLAKE3 mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    #[ignore = "requires retained GNU Flex source, TinyCC musl-v2, native musl, protected M4, and create-new scratch"]
    fn derives_retained_flex_inventory() {
        let source_root = std::env::var("MANTLE_STAGE_X_FLEX_RETAINED_SOURCE").unwrap();
        let tcc_root = std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap();
        let musl_root = std::env::var("MANTLE_STAGE_X_MUSL_NATIVE_ROOT").unwrap();
        let m4_root = std::env::var("MANTLE_STAGE_X_M4_ROOT").unwrap();
        let scratch = std::env::var("MANTLE_STAGE_X_FLEX_RUNTIME_SCRATCH").unwrap();
        let inventory = derive_flex_inventory(FlexInventoryRequest {
            source_root: Path::new(&source_root),
            tcc_musl_v2_root: Path::new(&tcc_root),
            musl_native_root: Path::new(&musl_root),
            m4_root: Path::new(&m4_root),
            scratch_dir: Path::new(&scratch),
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(inventory.source_compile_count, u32::try_from(FLEX_OBJECT_COUNT).unwrap());
        assert_eq!(inventory.build_command_count, FLEX_BUILD_COMMAND_COUNT);
        assert_eq!(inventory.smoke_command_count, FLEX_SMOKE_COMMAND_COUNT);
        assert!(inventory.fallback_events.is_empty());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_flex_source() {
        let bundle = std::env::var(SOURCE_BUNDLE_ENV).unwrap();
        let scratch = std::env::var("MANTLE_STAGE_X_FLEX_SOURCE_SCRATCH").unwrap();
        let report =
            materialize_authenticated_flex_source(Path::new(&bundle), SOURCE_MANIFEST_BLAKE3, Path::new(&scratch))
                .unwrap();
        assert_eq!(report.record_content_blake3, FLEX_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("src/filter.c").is_file());
    }
}
