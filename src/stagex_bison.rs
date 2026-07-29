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
const BISON_RECORD_NAME: &str = "bison-2.3-src";
const BISON_RECORD_IDENTITY: &str = "fixed-url-eb9a2eda57a4defe597f3db08ec664c41d9db4dcbf20674e7d34f47fd5114cf8";
const BISON_SOURCE_OUTPUT_NAME: &str = "bison-2.3";
pub(crate) const BISON_SOURCE_ARTIFACT_ID: &str = "bison-2.3-source";
pub(crate) const BISON_SOURCE_CONTENT_BLAKE3: &str = "45e40c482750cf4aa35637a7f62ee4f865a8a6d5dcf06aa557be8797c7bc4c62";
pub(crate) const BISON_RECIPE_ARTIFACT_ID: &str = "bison-2.3-recipe-source";
pub(crate) const BISON_RECIPE_BLAKE3: &str = "a086f71d144ee17961fe8462fee3b1d9184af885e45ee0200aa133c9fd54bd63";
const BISON_RECIPE: &[u8] = include_bytes!("../bootstrap/bison-2.3-musl.ncl");
const BISON_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-bison-source-materialization-v1";
const BISON_SOURCE_NON_CLAIM: &str =
    "Bison source materialization proves authenticated offline archive identity and fixed-output parity only";
const BISON_RECORD_HASH: &str = "sha256-A7EYDEuY+Hw8DHostT2RBv+kjkMumqDlshfpCi2I77U=";
const BISON_RECORD_URL: &str = "https://mirrors.kernel.org/gnu/bison/bison-2.3.tar.bz2";
const BISON_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const BISON_RECORD_UNPACK: &str = "1";
const BISON_SOURCE_ARTIFACT_COUNT: usize = 2;
const REQUIRED_SOURCE_FILES: &[&str] = &[
    "lib/hash.c",
    "lib/abitset.c",
    "src/main.c",
    "src/parse-gram.c",
    "src/scan-gram.c",
    "data/yacc.c",
];
const REQUIRED_SOURCE_FILE_COUNT: usize = 6;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const BISON_SOURCE_FILE_MEBIBYTES_MAX: u64 = 8;
const BISON_SOURCE_FILE_BYTES_MAX: u64 = BISON_SOURCE_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const BISON_ARTIFACT_MEBIBYTES_MAX: u64 = 64;
const BISON_ARTIFACT_BYTES_MAX: u64 = BISON_ARTIFACT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const BISON_OBSERVATION_KIBIBYTES_MAX: u64 = 256;
const BISON_OBSERVATION_BYTES_MAX: u64 = BISON_OBSERVATION_KIBIBYTES_MAX * KIBIBYTE_BYTES;
const BISON_LIBRARY_SOURCE_COUNT: usize = 27;
const BISON_PROGRAM_SOURCE_COUNT: usize = 29;
const BISON_OBJECT_COUNT: usize = BISON_LIBRARY_SOURCE_COUNT + BISON_PROGRAM_SOURCE_COUNT;
const BISON_BUILD_COMMAND_COUNT: u32 = 57;
const BISON_SMOKE_COMMAND_COUNT: u32 = 3;
const BISON_OUTPUT_COUNT: usize = 6;
const BISON_EXECUTABLE_MODE: u32 = 0o755;
const BISON_EXECUTE_MODE_MASK: u32 = 0o111;
const BISON_RUNTIME_START: &str = "      $BB cat > mantle_runtime_compat.c <<'RUNTIME_EOF'\n";
const BISON_RUNTIME_END: &str = "\nRUNTIME_EOF\n";
const BISON_REPORT_FORMAT: &str = "mantle-stagex-bison-2.3-inventory-v1";
const BISON_NON_CLAIM: &str = "this inventory binds GNU Bison 2.3, its exact bounded hash patch and compatibility runtime, and positive and negative grammar observations only; it does not prove arbitrary grammar semantics, Flex, binutils, native TinyCC, or provider admission";
pub(crate) const BISON_CONFIGURED_SOURCE_BLAKE3: &str =
    "7b9699a83f7028fb22c8aa5bffaa5e314e2a9e1ad8625c0304c2a7bbbf37f121";
pub(crate) const BISON_BINARY_BLAKE3: &str = "3db09397aa2752eea9c438a4cb1c5ebd27605f58b56ec43781baf411c3f2b983";
const BISON_RUNTIME_DATA_BLAKE3: &str = "f3d4a63ba0b8049557c690ad7db8d48d954c3dd6e45cfdd9488fe6d75d01bc7c";
const BISON_VERSION_BLAKE3: &str = "a83fcaecff55230ffd6fb2ff09b2018392e8c403a3e2a8087d2cb68115f383c3";
const BISON_POSITIVE_SOURCE_BLAKE3: &str = "bf73d7f6db6f0dd4a96706a1efb9e23dd287cbf847badd17f64d4c7ed235c923";
const BISON_POSITIVE_HEADER_BLAKE3: &str = "afa05cc327a73b17e33baf53fb44030b1a111aa993db60f01565a2a80dc8f913";
const BISON_NEGATIVE_BLAKE3: &str = "d0c674aaeab59c3efab1090e74cb598ae4df44bbcd42bc451a0a3f1a1257dcb4";
const BISON_LIBRARY_SOURCES: &[&str] = &[
    "abitset",
    "argmatch",
    "basename",
    "bitset",
    "bitset_stats",
    "bitsetv",
    "bitsetv-print",
    "dup-safer",
    "ebitset",
    "error",
    "exitfail",
    "fd-safer",
    "fopen-safer",
    "get-errno",
    "getopt",
    "getopt1",
    "hash",
    "lbitset",
    "obstack",
    "quote",
    "quotearg",
    "subpipe",
    "timevar",
    "vbitset",
    "xalloc-die",
    "xmalloc",
    "xstrndup",
];
const BISON_PROGRAM_SOURCES: &[&str] = &[
    "LR0",
    "assoc",
    "closure",
    "complain",
    "conflicts",
    "derives",
    "files",
    "getargs",
    "gram",
    "lalr",
    "location",
    "main",
    "muscle_tab",
    "nullable",
    "output",
    "parse-gram",
    "print",
    "print_graph",
    "reader",
    "reduce",
    "relation",
    "scan-gram",
    "scan-skel",
    "state",
    "symlist",
    "symtab",
    "tables",
    "uniqstr",
    "vcg",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BisonExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const BISON_EXPECTED_OUTPUTS: [BisonExpectedOutput; BISON_OUTPUT_COUNT] = [
    BisonExpectedOutput {
        artifact_id: "bison-2.3",
        digest_blake3: BISON_BINARY_BLAKE3,
    },
    BisonExpectedOutput {
        artifact_id: "bison-2.3-runtime-data",
        digest_blake3: BISON_RUNTIME_DATA_BLAKE3,
    },
    BisonExpectedOutput {
        artifact_id: "bison-version-observation",
        digest_blake3: BISON_VERSION_BLAKE3,
    },
    BisonExpectedOutput {
        artifact_id: "bison-positive-parser-source",
        digest_blake3: BISON_POSITIVE_SOURCE_BLAKE3,
    },
    BisonExpectedOutput {
        artifact_id: "bison-positive-parser-header",
        digest_blake3: BISON_POSITIVE_HEADER_BLAKE3,
    },
    BisonExpectedOutput {
        artifact_id: "bison-negative-observation",
        digest_blake3: BISON_NEGATIVE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BisonSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BisonOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BisonInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub negative_exit_code: i32,
    pub outputs: Vec<BisonOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BisonInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub m4_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexBisonError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexBisonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU Bison source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU Bison materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU Bison runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexBisonError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexBisonError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); BISON_SOURCE_ARTIFACT_COUNT] {
    [
        (BISON_SOURCE_ARTIFACT_ID, BISON_SOURCE_CONTENT_BLAKE3),
        (BISON_RECIPE_ARTIFACT_ID, BISON_RECIPE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_bison_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<BisonSourceMaterializationReport, StagexBisonError> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexBisonError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexBisonError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexBisonError::Materialization(format!("creating Bison source scratch: {error}")))?;
    let output_path = scratch_dir.join(BISON_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path)
        .map_err(|error| StagexBisonError::Materialization(format!("materializing Bison source: {error}")))?;
    validate_materialized_source(&output_path)?;
    let materialization = BisonSourceMaterializationReport {
        format: BISON_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: BISON_SOURCE_ARTIFACT_ID,
        record_name: BISON_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: BISON_SOURCE_NON_CLAIM,
    };
    assert_eq!(materialization.record_identity, BISON_RECORD_IDENTITY);
    assert!(materialization.output_path.join("lib/hash.c").is_file());
    Ok(materialization)
}

fn validate_bundle_path(path: &Path) -> Result<(), StagexBisonError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(StagexBisonError::Materialization(format!(
            "source bundle must be an absolute regular file: {}",
            path.display()
        )));
    }
    let bytes_len = path
        .metadata()
        .map_err(|error| StagexBisonError::Materialization(format!("reading source-bundle metadata: {error}")))?
        .len();
    if bytes_len == 0 {
        return Err(StagexBisonError::Materialization("source bundle is empty".to_string()));
    }
    assert!(path.is_absolute());
    assert!(bytes_len > 0);
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexBisonError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(BISON_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(StagexBisonError::SourceRecordNotFound);
    }
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some(BISON_RECORD_NAME));
    Ok(matches[0])
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexBisonError> {
    require_record_fact(record.identity == BISON_RECORD_IDENTITY, "source record identity")?;
    require_record_fact(record.kind == SourceRecordKind::FixedUrl, "source record kind")?;
    require_metadata(record, "builder", "builtin:fetchurl")?;
    require_metadata(record, "hash", BISON_RECORD_HASH)?;
    require_metadata(record, "hash_algo", "sha256")?;
    require_metadata(record, "hash_mode", "recursive")?;
    require_metadata(record, "payload_encoding", BISON_RECORD_PAYLOAD_ENCODING)?;
    require_metadata(record, "unpack", BISON_RECORD_UNPACK)?;
    require_metadata(record, "url", BISON_RECORD_URL)?;
    require_record_fact(record.content_blake3 == BISON_SOURCE_CONTENT_BLAKE3, "source content BLAKE3")?;
    require_record_fact(record.files.len() == 1, "source record file count")?;
    let archive = &record.files[0];
    require_record_fact(archive.path == "archive", "source archive path")?;
    require_record_fact(archive.file_type == SourceFileType::Regular, "source archive file type")?;
    require_record_fact(!archive.executable, "source archive mode")?;
    require_record_fact(archive.size > 0, "source archive size")?;
    require_record_fact(archive.content_hex.is_some(), "source archive bytes")?;
    assert_eq!(record.identity, BISON_RECORD_IDENTITY);
    assert_eq!(record.files.len(), 1);
    Ok(())
}

fn require_record_fact(condition: bool, label: &str) -> Result<(), StagexBisonError> {
    if !condition {
        return Err(StagexBisonError::Materialization(format!("unexpected Bison {label}")));
    }
    Ok(())
}

fn require_metadata(record: &SourceRecord, key: &str, expected: &str) -> Result<(), StagexBisonError> {
    let observed = record.metadata.get(key).map(String::as_str);
    if observed != Some(expected) {
        return Err(StagexBisonError::Materialization(format!(
            "unexpected Bison source metadata {key}: expected {expected}, observed {observed:?}"
        )));
    }
    Ok(())
}

fn validate_materialized_source(path: &Path) -> Result<(), StagexBisonError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(StagexBisonError::Materialization(format!(
            "materialized Bison source is not an absolute directory: {}",
            path.display()
        )));
    }
    for relative in REQUIRED_SOURCE_FILES {
        if !path.join(relative).is_file() {
            return Err(StagexBisonError::Materialization(format!("materialized Bison source is missing {relative}")));
        }
    }
    assert_eq!(REQUIRED_SOURCE_FILES.len(), REQUIRED_SOURCE_FILE_COUNT);
    assert!(path.join("data/yacc.c").is_file());
    Ok(())
}

pub(crate) fn validate_recipe_digest() -> Result<(), StagexBisonError> {
    let observed = blake3::hash(BISON_RECIPE).to_hex().to_string();
    if observed != BISON_RECIPE_BLAKE3 {
        return Err(StagexBisonError::Materialization(format!(
            "Bison recipe BLAKE3 mismatch: expected {BISON_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert_ne!(observed, BISON_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

pub(crate) fn derive_bison_inventory(
    request: BisonInventoryRequest<'_>,
) -> Result<BisonInventoryReport, StagexBisonError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBisonError::Materialization(format!("creating Bison runtime scratch: {error}")))?;
    let retained_source = request.scratch_dir.join(BISON_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &retained_source)?;
    configure_retained_source(&retained_source)?;
    let output = request.scratch_dir.join("output");
    let output_bin = output.join("bin");
    let output_share = output.join("share");
    fs::create_dir_all(&output_bin)
        .map_err(|error| StagexBisonError::Materialization(format!("creating Bison output: {error}")))?;
    fs::create_dir_all(&output_share)
        .map_err(|error| StagexBisonError::Materialization(format!("creating Bison share output: {error}")))?;
    let bison = build_bison(&request, &retained_source, &output_bin)?;
    let runtime_data = output_share.join("bison");
    crate::stagex_mes_lib::copy_tree_bounded(&retained_source.join("data"), &runtime_data)?;
    let observations = run_checked_smokes(&request, &bison, &runtime_data)?;
    let outputs = collect_outputs(&bison, &runtime_data, &observations)?;
    validate_expected_outputs(&outputs)?;
    let inventory = BisonInventoryReport {
        format: BISON_REPORT_FORMAT,
        configured_source_digest_blake3: configured_source_digest_blake3()?,
        source_compile_count: u32::try_from(BISON_OBJECT_COUNT)
            .map_err(|_| StagexBisonError::Materialization("Bison object count does not fit u32".to_string()))?,
        build_command_count: BISON_BUILD_COMMAND_COUNT,
        smoke_command_count: BISON_SMOKE_COMMAND_COUNT,
        negative_exit_code: observations.negative_exit_code,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: BISON_NON_CLAIM,
    };
    let inventory_path = request.scratch_dir.join("bison-inventory.json");
    let bytes = serde_json::to_vec_pretty(&inventory)
        .map_err(|error| StagexBisonError::Materialization(format!("serializing Bison inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&inventory_path, &bytes)?;
    assert_eq!(inventory.outputs.len(), BISON_OUTPUT_COUNT);
    assert_ne!(inventory.negative_exit_code, 0);
    Ok(inventory)
}

fn validate_inventory_inputs(request: &BisonInventoryRequest<'_>) -> Result<(), StagexBisonError> {
    if request.scratch_dir.exists() {
        return Err(StagexBisonError::Materialization(format!(
            "Bison runtime scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    validate_materialized_source(request.source_root)?;
    validate_recipe_digest()?;
    let configured_source = configured_source_digest_blake3()?;
    if configured_source != BISON_CONFIGURED_SOURCE_BLAKE3 {
        return Err(StagexBisonError::Materialization(format!(
            "Bison configured-source BLAKE3 mismatch: expected {BISON_CONFIGURED_SOURCE_BLAKE3}, observed {configured_source}"
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
        return Err(StagexBisonError::Materialization("native musl headers are missing".to_string()));
    }
    assert!(!request.scratch_dir.exists());
    assert!(request.m4_root.join("bin/m4").is_file());
    Ok(())
}

fn configure_retained_source(source_root: &Path) -> Result<(), StagexBisonError> {
    let runtime = extract_recipe_block(BISON_RUNTIME_START, BISON_RUNTIME_END, "compatibility runtime")?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), bison_config_header().as_bytes())?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("mantle_runtime_compat.c"), runtime.as_bytes())?;
    let hash_path = source_root.join("lib/hash.c");
    let bytes = crate::stagex_mes_lib::read_bounded_file(&hash_path, BISON_SOURCE_FILE_BYTES_MAX, "Bison hash source")?;
    let input = String::from_utf8(bytes)
        .map_err(|error| StagexBisonError::Materialization(format!("reading UTF-8 Bison hash source: {error}")))?;
    let transformed = transform_hash_source(&input)?;
    fs::write(&hash_path, transformed.as_bytes())
        .map_err(|error| StagexBisonError::Materialization(format!("writing configured Bison hash source: {error}")))?;
    validate_configured_source(source_root)?;
    assert!(source_root.join("config.h").is_file());
    assert!(source_root.join("mantle_runtime_compat.c").is_file());
    Ok(())
}

fn bison_config_header() -> String {
    let header = r#"#define PACKAGE "bison"
#define PACKAGE_NAME "bison"
#define PACKAGE_VERSION "2.3"
#define VERSION "2.3"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_STDINT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_WCHAR_H 1
#define HAVE_WCTYPE_H 1
#define HAVE_CLOCK_T 1
#define STDC_HEADERS 1
#define M4 "/mantle/stagex/required-m4"
#define M4_GNU_OPTION ""
#define LOCALEDIR "/mantle/stagex/required-bison-locale"
#define PKGDATADIR "/mantle/stagex/required-bison-data"
#define EXEEXT ""
#define _GL_UNUSED
#define __getopt_argv_const const
"#;
    assert!(header.contains("/mantle/stagex/required-m4"));
    assert!(!header.contains("$NIX_STORE"));
    header.to_string()
}

fn transform_hash_source(input: &str) -> Result<String, StagexBisonError> {
    let mut text = input.to_string();
    replace_exact(
        &mut text,
        "#define DEFAULT_GROWTH_THRESHOLD 0.8\n#define DEFAULT_GROWTH_FACTOR 1.414",
        "#define MANTLE_HASH_RATIO_DENOMINATOR 1000\n#define MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR 800\n#define MANTLE_HASH_GROWTH_FACTOR_NUMERATOR 1414\n#define MANTLE_HASH_COMBINED_GROWTH_NUMERATOR 11312\n#define MANTLE_HASH_COMBINED_GROWTH_DENOMINATOR 10000\n#define DEFAULT_GROWTH_THRESHOLD ((float) 8 / (float) 10)\n#define DEFAULT_GROWTH_FACTOR ((float) 1414 / (float) 1000)",
        "growth constants",
    )?;
    replace_exact(
        &mut text,
        "#define DEFAULT_SHRINK_THRESHOLD 0.0\n#define DEFAULT_SHRINK_FACTOR 1.0",
        "#define DEFAULT_SHRINK_THRESHOLD ((float) 0)\n#define DEFAULT_SHRINK_FACTOR ((float) 1)",
        "shrink constants",
    )?;
    replace_exact(&mut text, CHECK_TUNING_OLD, CHECK_TUNING_NEW, "tuning core")?;
    replace_exact(&mut text, INITIAL_CANDIDATE_OLD, INITIAL_CANDIDATE_NEW, "initial candidate")?;
    replace_exact(&mut text, GROWTH_REHASH_OLD, GROWTH_REHASH_NEW, "growth rehash")?;
    assert_ne!(text, input);
    assert!(text.contains("mantle_hash_growth_reached"));
    Ok(text)
}

const CHECK_TUNING_OLD: &str = r#"static bool
check_tuning (Hash_table *table)
{
  const Hash_tuning *tuning = table->tuning;

  /* Be a bit stricter than mathematics would require, so that
     rounding errors in size calculations do not cause allocations to
     fail to grow or shrink as they should.  The smallest allocation
     is 11 (due to next_prime's algorithm), so an epsilon of 0.1
     should be good enough.  */
  float epsilon = 0.1f;

  if (epsilon < tuning->growth_threshold
      && tuning->growth_threshold < 1 - epsilon
      && 1 + epsilon < tuning->growth_factor
      && 0 <= tuning->shrink_threshold
      && tuning->shrink_threshold + epsilon < tuning->shrink_factor
      && tuning->shrink_factor <= 1
      && tuning->shrink_threshold + epsilon < tuning->growth_threshold)
    return true;

  table->tuning = &default_tuning;
  return false;
}"#;

const CHECK_TUNING_NEW: &str = r#"static bool
check_tuning (Hash_table *table)
{
  const Hash_tuning *tuning = table->tuning;

  if (tuning == &default_tuning)
    return true;

  table->tuning = &default_tuning;
  return false;
}

static bool
mantle_hash_ratio_ceil (size_t value, size_t numerator,
                        size_t denominator, size_t *result)
{
  size_t adjustment;

  if (denominator == 0 || numerator == 0 || result == NULL)
    return false;
  adjustment = denominator - 1;
  if (value > (SIZE_MAX - adjustment) / numerator)
    return false;
  *result = (value * numerator + adjustment) / denominator;
  return true;
}

static bool
mantle_hash_growth_reached (size_t used, size_t buckets)
{
  if (used > SIZE_MAX / MANTLE_HASH_RATIO_DENOMINATOR)
    return true;
  if (buckets > SIZE_MAX / MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR)
    return false;
  return used * MANTLE_HASH_RATIO_DENOMINATOR
         > buckets * MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR;
}"#;

const INITIAL_CANDIDATE_OLD: &str = r#"  if (!tuning->is_n_buckets)
    {
      float new_candidate = candidate / tuning->growth_threshold;
      if (SIZE_MAX <= new_candidate)
	goto fail;
      candidate = new_candidate;
    }"#;

const INITIAL_CANDIDATE_NEW: &str = r#"  if (!tuning->is_n_buckets)
    {
      if (! mantle_hash_ratio_ceil
          (candidate, MANTLE_HASH_RATIO_DENOMINATOR,
           MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR, &candidate))
        goto fail;
    }"#;

const GROWTH_REHASH_OLD: &str = r#"  if (table->n_buckets_used
      > table->tuning->growth_threshold * table->n_buckets)
    {
      /* Check more fully, before starting real work.  If tuning arguments
	 became invalid, the second check will rely on proper defaults.  */
      check_tuning (table);
      if (table->n_buckets_used
	  > table->tuning->growth_threshold * table->n_buckets)
	{
	  const Hash_tuning *tuning = table->tuning;
	  float candidate =
	    (tuning->is_n_buckets
	     ? (table->n_buckets * tuning->growth_factor)
	     : (table->n_buckets * tuning->growth_factor
		* tuning->growth_threshold));

	  if (SIZE_MAX <= candidate)
	    return NULL;"#;

const GROWTH_REHASH_NEW: &str = r#"  if (mantle_hash_growth_reached (table->n_buckets_used,
                                  table->n_buckets))
    {
      /* Check more fully, before starting real work.  If tuning arguments
	 became invalid, the second check will rely on proper defaults.  */
      check_tuning (table);
      if (mantle_hash_growth_reached (table->n_buckets_used,
	                              table->n_buckets))
	{
	  const Hash_tuning *tuning = table->tuning;
	  size_t candidate;
	  size_t numerator = tuning->is_n_buckets
	                     ? MANTLE_HASH_GROWTH_FACTOR_NUMERATOR
	                     : MANTLE_HASH_COMBINED_GROWTH_NUMERATOR;
	  size_t denominator = tuning->is_n_buckets
	                       ? MANTLE_HASH_RATIO_DENOMINATOR
	                       : MANTLE_HASH_COMBINED_GROWTH_DENOMINATOR;

	  if (! mantle_hash_ratio_ceil (table->n_buckets, numerator,
	                                denominator, &candidate))
	    return NULL;"#;

fn replace_exact(text: &mut String, old: &str, new: &str, label: &str) -> Result<(), StagexBisonError> {
    let observed_count = text.matches(old).count();
    if observed_count != 1 {
        return Err(StagexBisonError::Materialization(format!(
            "unexpected Bison {label} source shape: expected 1, observed {observed_count}"
        )));
    }
    *text = text.replace(old, new);
    if !new.contains(old) {
        assert_eq!(text.matches(old).count(), 0);
    }
    assert!(text.contains(new));
    Ok(())
}

fn extract_recipe_block(start: &str, end: &str, label: &str) -> Result<String, StagexBisonError> {
    let recipe = std::str::from_utf8(BISON_RECIPE)
        .map_err(|error| StagexBisonError::Materialization(format!("reading UTF-8 Bison recipe: {error}")))?;
    let (_, suffix) = recipe
        .split_once(start)
        .ok_or_else(|| StagexBisonError::Materialization(format!("Bison {label} start marker is missing")))?;
    let (body, _) = suffix
        .split_once(end)
        .ok_or_else(|| StagexBisonError::Materialization(format!("Bison {label} end marker is missing")))?;
    if body.is_empty() {
        return Err(StagexBisonError::Materialization(format!("Bison {label} block is empty")));
    }
    assert!(!body.is_empty());
    assert!(!body.contains(end));
    Ok(format!("{body}\n"))
}

fn validate_configured_source(source_root: &Path) -> Result<(), StagexBisonError> {
    let hash = fs::read_to_string(source_root.join("lib/hash.c"))
        .map_err(|error| StagexBisonError::Materialization(format!("reading configured Bison hash source: {error}")))?;
    let runtime = fs::read_to_string(source_root.join("mantle_runtime_compat.c"))
        .map_err(|error| StagexBisonError::Materialization(format!("reading Bison compatibility runtime: {error}")))?;
    if !hash.contains("mantle_hash_growth_reached") || hash.contains("#define DEFAULT_GROWTH_THRESHOLD 0.8") {
        return Err(StagexBisonError::Materialization("bounded Bison hash patch is incomplete".to_string()));
    }
    if !runtime.contains("MANTLE_BISON_TEXT_BYTES_MAX") || !runtime.contains("__fixunssfdi") {
        return Err(StagexBisonError::Materialization("Bison compatibility runtime is incomplete".to_string()));
    }
    assert!(hash.contains("MANTLE_HASH_RATIO_DENOMINATOR"));
    assert!(runtime.contains("mbsnwidth"));
    Ok(())
}

pub(crate) fn configured_source_digest_blake3() -> Result<String, StagexBisonError> {
    let runtime = extract_recipe_block(BISON_RUNTIME_START, BISON_RUNTIME_END, "compatibility runtime")?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-bison-configured-source-v1\0");
    hasher.update(BISON_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(BISON_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0exact-bounded-hash-patch-v1\0");
    hasher.update(bison_config_header().as_bytes());
    hasher.update(b"\0");
    hasher.update(runtime.as_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn build_bison(
    request: &BisonInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
) -> Result<PathBuf, StagexBisonError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let include_path = request.musl_native_root.join("include");
    let include = utf8_absolute(&include_path, "native musl include")?;
    let common_args = [
        "-I.".to_string(),
        "-Ilib".to_string(),
        format!("-I{include}"),
        "-DHAVE_CONFIG_H".to_string(),
    ];
    let mut objects = Vec::with_capacity(BISON_OBJECT_COUNT);
    for source_name in BISON_LIBRARY_SOURCES {
        objects.push(compile_bison_source(request, source_root, &compiler, &common_args, "lib", source_name)?);
    }
    for source_name in BISON_PROGRAM_SOURCES {
        objects.push(compile_bison_source(request, source_root, &compiler, &common_args, "src", source_name)?);
    }
    let bison = link_bison(request, source_root, output_bin, &compiler, &objects)?;
    assert_eq!(objects.len(), BISON_OBJECT_COUNT);
    assert!(bison.is_file());
    Ok(bison)
}

fn compile_bison_source(
    request: &BisonInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    common_args: &[String],
    directory: &str,
    source_name: &str,
) -> Result<String, StagexBisonError> {
    let source = format!("{directory}/{source_name}.c");
    let object = format!("{directory}/{source_name}.o");
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
        &request.scratch_dir.join(format!("bison-{directory}-{source_name}-compile.stderr.txt")),
    )?;
    validate_nonempty_artifact(&source_root.join(&object), "Bison object")?;
    assert!(source_root.join(source).is_file());
    assert!(source_root.join(&object).is_file());
    Ok(object)
}

fn link_bison(
    request: &BisonInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
    compiler: &Path,
    objects: &[String],
) -> Result<PathBuf, StagexBisonError> {
    let bison = output_bin.join("bison");
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?),
        "-o".to_string(),
        utf8_absolute(&bison, "Bison output")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
        "mantle_runtime_compat.c".to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("bison-link.stderr.txt"),
    )?;
    fs::set_permissions(&bison, fs::Permissions::from_mode(BISON_EXECUTABLE_MODE))
        .map_err(|error| StagexBisonError::Materialization(format!("setting Bison executable mode: {error}")))?;
    validate_nonempty_artifact(&bison, "Bison executable")?;
    assert_eq!(objects.len(), BISON_OBJECT_COUNT);
    assert!(
        bison
            .metadata()
            .map(|value| value.permissions().mode() & BISON_EXECUTE_MODE_MASK != 0)
            .unwrap_or(false)
    );
    Ok(bison)
}

struct BisonObservations {
    version: PathBuf,
    positive_c: PathBuf,
    positive_h: PathBuf,
    negative: PathBuf,
    negative_exit_code: i32,
}

fn run_checked_smokes(
    request: &BisonInventoryRequest<'_>,
    bison: &Path,
    runtime_data: &Path,
) -> Result<BisonObservations, StagexBisonError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexBisonError::Materialization(format!("creating Bison smoke root: {error}")))?;
    let environment = bison_environment(request, runtime_data)?;
    let version = run_bison_success(bison, &smoke, "version", &["--version"], b"", &environment)?;
    require_contains(&version, b"bison", "Bison version observation")?;
    let positive_y = smoke.join("positive.y");
    crate::stagex_mes_lib::write_create_new(&positive_y, b"%token WORD\n%%\ninput: WORD ;\n%%\n")?;
    let positive_stdout =
        run_bison_success(bison, &smoke, "positive", &["-d", "-o", "positive.c", "positive.y"], b"", &environment)?;
    require_exact_bytes(&positive_stdout, b"", "Bison positive stdout")?;
    let positive_c = smoke.join("positive.c");
    let positive_h = smoke.join("positive.h");
    validate_nonempty_artifact(&positive_c, "Bison positive parser source")?;
    validate_nonempty_artifact(&positive_h, "Bison positive parser header")?;
    let (negative, negative_exit_code) = run_negative_smoke(bison, &smoke, &environment)?;
    assert_ne!(negative_exit_code, 0);
    assert!(negative.is_file());
    Ok(BisonObservations {
        version,
        positive_c,
        positive_h,
        negative,
        negative_exit_code,
    })
}

fn run_bison_success(
    bison: &Path,
    smoke: &Path,
    label: &str,
    arguments: &[&str],
    stdin_bytes: &[u8],
    environment: &BTreeMap<String, String>,
) -> Result<PathBuf, StagexBisonError> {
    let stdin_path = smoke.join(format!("{label}.in"));
    let stdout_path = smoke.join(format!("{label}.out"));
    let stderr_path = smoke.join(format!("{label}.stderr.txt"));
    crate::stagex_mes_lib::write_create_new(&stdin_path, stdin_bytes)?;
    let args = arguments.iter().map(|argument| (*argument).to_string()).collect::<Vec<_>>();
    crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout(
        bison,
        &args,
        smoke,
        environment,
        &stdin_path,
        BISON_OBSERVATION_BYTES_MAX,
        &stdout_path,
        BISON_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(stdout_path)
}

fn run_negative_smoke(
    bison: &Path,
    smoke: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<(PathBuf, i32), StagexBisonError> {
    let grammar = smoke.join("negative.y");
    let stdin_path = smoke.join("negative.in");
    let stdout_path = smoke.join("negative.stdout.txt");
    let stderr_path = smoke.join("negative.stderr.txt");
    crate::stagex_mes_lib::write_create_new(&grammar, b"%token WORD\n%%\ninput WORD ;\n%%\n")?;
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"")?;
    let args = ["-o".to_string(), "negative.c".to_string(), "negative.y".to_string()];
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        bison,
        &args,
        smoke,
        environment,
        &stdin_path,
        BISON_OBSERVATION_BYTES_MAX,
        &stdout_path,
        BISON_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    if exit_code == 0 || smoke.join("negative.c").exists() {
        return Err(StagexBisonError::Materialization(
            "invalid Bison grammar succeeded or retained parser output".to_string(),
        ));
    }
    require_exact_bytes(&stdout_path, b"", "Bison negative stdout")?;
    validate_nonempty_artifact(&stderr_path, "Bison negative diagnostic")?;
    assert_ne!(exit_code, 0);
    assert!(!smoke.join("negative.c").exists());
    Ok((stderr_path, exit_code))
}

fn bison_environment(
    request: &BisonInventoryRequest<'_>,
    runtime_data: &Path,
) -> Result<BTreeMap<String, String>, StagexBisonError> {
    let mut environment = BTreeMap::new();
    environment
        .insert("M4".to_string(), utf8_absolute(&request.m4_root.join("bin/m4"), "protected GNU M4")?.to_string());
    environment.insert("BISON_PKGDATADIR".to_string(), utf8_absolute(runtime_data, "Bison runtime data")?.to_string());
    environment.insert("LC_ALL".to_string(), "C".to_string());
    assert_eq!(environment.len(), 3);
    assert_eq!(environment.get("LC_ALL").map(String::as_str), Some("C"));
    Ok(environment)
}

fn collect_outputs(
    bison: &Path,
    runtime_data: &Path,
    observations: &BisonObservations,
) -> Result<Vec<BisonOutputReport>, StagexBisonError> {
    let specs = [
        ("bison-2.3", bison, false),
        ("bison-2.3-runtime-data", runtime_data, true),
        ("bison-version-observation", observations.version.as_path(), false),
        ("bison-positive-parser-source", observations.positive_c.as_path(), false),
        ("bison-positive-parser-header", observations.positive_h.as_path(), false),
        ("bison-negative-observation", observations.negative.as_path(), false),
    ];
    let outputs = specs
        .iter()
        .map(|(artifact_id, path, is_tree)| output_report(artifact_id, path, *is_tree))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), BISON_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[BisonOutputReport]) -> Result<(), StagexBisonError> {
    if outputs.len() != BISON_EXPECTED_OUTPUTS.len() {
        return Err(StagexBisonError::Materialization(format!(
            "expected {} Bison outputs, observed {}",
            BISON_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in BISON_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexBisonError::Materialization(format!("Bison output is missing: {}", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            return Err(StagexBisonError::Materialization(format!(
                "Bison output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, observed.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), BISON_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(())
}

fn output_report(artifact_id: &str, path: &Path, is_tree: bool) -> Result<BisonOutputReport, StagexBisonError> {
    let (bytes_len, digest_blake3) = if is_tree {
        (
            crate::stagex_musl::tree_bytes_len(path)
                .map_err(|error| StagexBisonError::Materialization(error.to_string()))?,
            crate::stagex_musl::tree_digest_blake3(path)
                .map_err(|error| StagexBisonError::Materialization(error.to_string()))?,
        )
    } else {
        let bytes = crate::stagex_mes_lib::read_bounded_file(path, BISON_ARTIFACT_BYTES_MAX, artifact_id)?;
        let bytes_len = u64::try_from(bytes.len())
            .map_err(|_| StagexBisonError::Materialization(format!("Bison artifact is too large: {artifact_id}")))?;
        (bytes_len, blake3::hash(&bytes).to_hex().to_string())
    };
    assert!(bytes_len <= BISON_ARTIFACT_BYTES_MAX);
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(BisonOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexBisonError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexBisonError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() > BISON_OBSERVATION_BYTES_MAX {
        return Err(StagexBisonError::Materialization(format!(
            "{label} is not regular or exceeds {BISON_OBSERVATION_BYTES_MAX} bytes"
        )));
    }
    let observed =
        fs::read(path).map_err(|error| StagexBisonError::Materialization(format!("reading {label}: {error}")))?;
    if observed != expected {
        return Err(StagexBisonError::Materialization(format!("{label} bytes differ")));
    }
    assert_eq!(u64::try_from(observed.len()).unwrap_or(u64::MAX), metadata.len());
    assert_eq!(observed, expected);
    Ok(())
}

fn require_contains(path: &Path, needle: &[u8], label: &str) -> Result<(), StagexBisonError> {
    let observed = crate::stagex_mes_lib::read_bounded_file(path, BISON_OBSERVATION_BYTES_MAX, label)?;
    if !observed.windows(needle.len()).any(|window| window == needle) {
        return Err(StagexBisonError::Materialization(format!("{label} lacks required bytes")));
    }
    assert!(!needle.is_empty());
    assert!(!observed.is_empty());
    Ok(())
}

fn validate_nonempty_artifact(path: &Path, label: &str) -> Result<(), StagexBisonError> {
    let metadata = path
        .metadata()
        .map_err(|error| StagexBisonError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > BISON_ARTIFACT_BYTES_MAX {
        return Err(StagexBisonError::Materialization(format!("invalid {label}: {}", path.display())));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() <= BISON_ARTIFACT_BYTES_MAX);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexBisonError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, BISON_ARTIFACT_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexBisonError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(())
}

fn native_musl_digest(artifact_id: &str) -> Result<&'static str, StagexBisonError> {
    let digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.digest_blake3)
        .ok_or_else(|| StagexBisonError::Materialization(format!("native musl inventory is missing {artifact_id}")))?;
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexBisonError> {
    if !path.is_absolute() {
        return Err(StagexBisonError::Materialization(format!("{label} path is not absolute: {}", path.display())));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexBisonError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const SOURCE_MANIFEST_BLAKE3: &str = "541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab";

    #[test]
    fn validates_exact_bison_source_record_and_recipe() {
        let bundle = match std::env::var(SOURCE_BUNDLE_ENV) {
            Ok(value) => value,
            Err(_) => return,
        };
        let manifest = read_source_bundle(Path::new(&bundle)).unwrap();
        let record = find_source_record(&manifest.records).unwrap();
        validate_source_record(record).unwrap();
        validate_recipe_digest().unwrap();
        assert_eq!(record.content_blake3, BISON_SOURCE_CONTENT_BLAKE3);
        assert_eq!(source_artifact_digests().len(), BISON_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_bison_source_authority() {
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
    fn applies_exact_bison_hash_patch() {
        let source_root = match std::env::var("MANTLE_STAGE_X_BISON_RETAINED_SOURCE") {
            Ok(value) => PathBuf::from(value),
            Err(_) => return,
        };
        let input = fs::read_to_string(source_root.join("lib/hash.c")).unwrap();
        let output = transform_hash_source(&input).unwrap();
        assert_ne!(output, input);
        assert!(output.contains("mantle_hash_growth_reached"));
        assert_eq!(configured_source_digest_blake3().unwrap().len(), BLAKE3_HEX_CHAR_COUNT);
    }

    #[test]
    fn pins_deterministic_bison_identities() {
        assert_eq!(configured_source_digest_blake3().unwrap(), BISON_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(BISON_EXPECTED_OUTPUTS.len(), BISON_OUTPUT_COUNT);
        assert_eq!(BISON_EXPECTED_OUTPUTS[0].digest_blake3, BISON_BINARY_BLAKE3);
        assert_ne!(BISON_BINARY_BLAKE3, BISON_SOURCE_CONTENT_BLAKE3);
    }

    #[test]
    fn rejects_substituted_bison_output_identity() {
        let mut outputs = BISON_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| BisonOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        outputs[0].digest_blake3 = BISON_SOURCE_CONTENT_BLAKE3.to_string();
        let error = validate_expected_outputs(&outputs).unwrap_err().to_string();
        assert!(error.contains("BLAKE3 mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_bison_hash_patch_substitution() {
        let error = replace_exact(&mut "safe".to_string(), "missing", "new", "fixture").unwrap_err().to_string();
        assert!(error.contains("expected 1, observed 0"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_bison_source() {
        let bundle = PathBuf::from(std::env::var(SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(
            std::env::var("MANTLE_STAGE_X_BISON_SOURCE_SCRATCH")
                .unwrap_or_else(|_| "/tmp/mantle-stagex-bison-source".to_string()),
        );
        let report = materialize_authenticated_bison_source(&bundle, SOURCE_MANIFEST_BLAKE3, &scratch).unwrap();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        assert_eq!(report.record_content_blake3, BISON_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("lib/hash.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU Bison source, TinyCC musl-v2, native musl, protected M4, and create-new scratch"]
    fn derives_retained_bison_inventory() {
        let source_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_BISON_RETAINED_SOURCE").unwrap());
        let tcc_musl_v2_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap());
        let musl_native_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_NATIVE_MUSL_ROOT").unwrap());
        let m4_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_M4_ROOT").unwrap());
        let scratch_dir = PathBuf::from(std::env::var("MANTLE_STAGE_X_BISON_RUNTIME_SCRATCH").unwrap());
        let report = derive_bison_inventory(BisonInventoryRequest {
            source_root: &source_root,
            tcc_musl_v2_root: &tcc_musl_v2_root,
            musl_native_root: &musl_native_root,
            m4_root: &m4_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        println!("configured-source={}", configured_source_digest_blake3().unwrap());
        assert_eq!(report.source_compile_count, u32::try_from(BISON_OBJECT_COUNT).unwrap());
        assert_eq!(report.outputs.len(), BISON_OUTPUT_COUNT);
    }
}
