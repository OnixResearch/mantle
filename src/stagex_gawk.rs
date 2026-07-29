use std::collections::BTreeMap;
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
const GAWK_RECORD_NAME: &str = "gawk-3.0.4-src";
const GAWK_RECORD_IDENTITY: &str = "fixed-url-26df0c332cbf1ba0f74000fa7384284180e583a84eeec6976e989dee9e869b77";
const GAWK_SOURCE_OUTPUT_NAME: &str = "gawk-3.0.4";
pub(crate) const GAWK_SOURCE_ARTIFACT_ID: &str = "gawk-3.0.4-source";
pub(crate) const GAWK_SOURCE_CONTENT_BLAKE3: &str = "db0f59d2c7905c1e16d8c45df3548e304a593bdd193444da7e550ff109deb9aa";
pub(crate) const GAWK_RECIPE_ARTIFACT_ID: &str = "gawk-3.0.4-recipe-source";
pub(crate) const GAWK_RECIPE_BLAKE3: &str = "1d35ef5716aea8d04a916ec05d3b4345256d4c4313205fd360b119aebb54b637";
const GAWK_RECIPE: &[u8] = include_bytes!("../bootstrap/gawk-3.0.4-musl.ncl");
const GAWK_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-gawk-source-materialization-v1";
const GAWK_SOURCE_NON_CLAIM: &str =
    "Gawk source materialization proves authenticated offline archive identity and fixed-output parity only";
const GAWK_RECORD_HASH: &str = "sha256-Vvj7dAxXMJhOHjWRd6ikD4eyjIERORHR9UcWfBDjAkY=";
const GAWK_RECORD_URL: &str = "https://mirrors.kernel.org/gnu/gawk/gawk-3.0.4.tar.gz";
const GAWK_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const GAWK_RECORD_UNPACK: &str = "1";
const GAWK_SOURCE_ARTIFACT_COUNT: usize = 2;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const GAWK_SOURCE_FILE_MEBIBYTES_MAX: u64 = 8;
const GAWK_SOURCE_FILE_BYTES_MAX: u64 = GAWK_SOURCE_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const GAWK_ARTIFACT_MEBIBYTES_MAX: u64 = 64;
const GAWK_ARTIFACT_BYTES_MAX: u64 = GAWK_ARTIFACT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const GAWK_OBSERVATION_KIBIBYTES_MAX: u64 = 64;
const GAWK_OBSERVATION_BYTES_MAX: u64 = GAWK_OBSERVATION_KIBIBYTES_MAX * KIBIBYTE_BYTES;
const GAWK_BUILD_COMMAND_COUNT: u32 = 17;
const GAWK_SMOKE_COMMAND_COUNT: u32 = 7;
const GAWK_OUTPUT_COUNT: usize = 9;
const GAWK_OBJECT_COUNT: usize = 16;
const REQUIRED_SOURCE_FILE_COUNT: usize = 14;
const FILE_MODE_MASK: u32 = 0o777;
const GAWK_OBJECT_OWNER_READ_WRITE_MASK: u32 = 0o600;
const GAWK_EXECUTABLE_MODE: u32 = 0o755;
const GAWK_EXECUTE_MODE_MASK: u32 = 0o111;
const GAWK_CONFIG_START: &str = "      $BB cat > config.h << 'CONFIG'\n";
const GAWK_CONFIG_END: &str = "\nCONFIG\n";
const GAWK_DECIMAL_START: &str = "      $BB cat > mantle_decimal.c <<'DECIMAL_EOF'\n";
const GAWK_DECIMAL_END: &str = "\nDECIMAL_EOF\n";
const GAWK_REPORT_FORMAT: &str = "mantle-stagex-gawk-3.0.4-inventory-v1";
const GAWK_NON_CLAIM: &str = "this inventory binds GNU Gawk 3.0.4, exact checked source normalizations, its bounded decimal runtime, and positive and negative generator observations only; it does not prove general AWK behavior, parser generators, binutils, native TinyCC, or provider admission";
pub(crate) const GAWK_CONFIGURED_SOURCE_BLAKE3: &str =
    "ccf0a5c408f8d60f4c8d72413c4c0ec0f8a49e29884c54c229ede61284dc7988";
pub(crate) const GAWK_BINARY_BLAKE3: &str = "1608d2ac4b0f9faaecd91494502942fccfcec3fd6298aa610f2dfd4308aaf5d1";
const GAWK_BEHAVIOR_BLAKE3: &str = "e8d40a8383b8b65278313f2589e148f294e19cba489e3517f3153c38d8603f7f";
const GAWK_NUMERIC_BLAKE3: &str = "c95e800034a073abe91702b1faca0c3bf102d53fdebcd4d0cf46b39c524a98ba";
const GAWK_DECIMAL_BLAKE3: &str = "49124bf4f7f37328738ac34216a60dcd5f58bb198c5c3f6719b6becafb7e7882";
const GAWK_LENGTH_BLAKE3: &str = "b9a1a3183dd350f0e896d0f4b59c87e7bda8b1ed3a1af76afc86c1cb8f7cbbde";
const GAWK_HEX_BLAKE3: &str = "320b59884ca4a418f85cef0916f3761d8a7225abda1b1069cb2182a9a15bdde9";
const GAWK_NEGATIVE_BLAKE3: &str = "9633428410d0cea43777d103fa8f7aa73048977e6b3421ee6d8ea663f7722e65";
const GAWK_INSTALLED_BLAKE3: &str = "b59a7a3ea06ce3059bdb45f57197d14ea5d2ecae9fb8b840cf7f486fc9685f33";
const GAWK_COMPILE_SOURCES: &[&str] = &[
    "mantle_decimal.c",
    "array.c",
    "builtin.c",
    "dfa.c",
    "eval.c",
    "field.c",
    "io.c",
    "main.c",
    "msg.c",
    "node.c",
    "random.c",
    "re.c",
    "regex.c",
    "version.c",
    "gawkmisc.c",
];
const REQUIRED_SOURCE_FILES: &[&str] = &[
    "array.c",
    "builtin.c",
    "dfa.c",
    "eval.c",
    "field.c",
    "gawkmisc.c",
    "io.c",
    "main.c",
    "msg.c",
    "node.c",
    "random.c",
    "re.c",
    "regex.c",
    "version.c",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GawkExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const GAWK_EXPECTED_OUTPUTS: [GawkExpectedOutput; GAWK_OUTPUT_COUNT] = [
    GawkExpectedOutput {
        artifact_id: "gawk-3.0.4",
        digest_blake3: GAWK_BINARY_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "awk-3.0.4-alias",
        digest_blake3: GAWK_BINARY_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-behavior-observation",
        digest_blake3: GAWK_BEHAVIOR_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-numeric-observation",
        digest_blake3: GAWK_NUMERIC_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-decimal-observation",
        digest_blake3: GAWK_DECIMAL_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-length-observation",
        digest_blake3: GAWK_LENGTH_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-hex-observation",
        digest_blake3: GAWK_HEX_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-negative-observation",
        digest_blake3: GAWK_NEGATIVE_BLAKE3,
    },
    GawkExpectedOutput {
        artifact_id: "gawk-installed-observation",
        digest_blake3: GAWK_INSTALLED_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GawkSourceMaterializationReport {
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
pub(crate) struct GawkOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GawkInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub negative_exit_code: i32,
    pub outputs: Vec<GawkOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GawkInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexGawkError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexGawkError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU Gawk source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU Gawk materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU Gawk runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexGawkError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexGawkError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); GAWK_SOURCE_ARTIFACT_COUNT] {
    [
        (GAWK_SOURCE_ARTIFACT_ID, GAWK_SOURCE_CONTENT_BLAKE3),
        (GAWK_RECIPE_ARTIFACT_ID, GAWK_RECIPE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_gawk_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<GawkSourceMaterializationReport, StagexGawkError> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexGawkError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexGawkError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexGawkError::Materialization(format!("creating Gawk source scratch: {error}")))?;
    let output_path = scratch_dir.join(GAWK_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path)
        .map_err(|error| StagexGawkError::Materialization(format!("materializing Gawk source: {error}")))?;
    validate_materialized_source(&output_path)?;
    let materialization = GawkSourceMaterializationReport {
        format: GAWK_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: GAWK_SOURCE_ARTIFACT_ID,
        record_name: GAWK_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: GAWK_SOURCE_NON_CLAIM,
    };
    assert_eq!(materialization.record_identity, GAWK_RECORD_IDENTITY);
    assert!(materialization.output_path.join("builtin.c").is_file());
    Ok(materialization)
}

fn validate_bundle_path(path: &Path) -> Result<(), StagexGawkError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(StagexGawkError::Materialization(format!(
            "source bundle must be an absolute regular file: {}",
            path.display()
        )));
    }
    let bytes_len = path
        .metadata()
        .map_err(|error| StagexGawkError::Materialization(format!("reading source-bundle metadata: {error}")))?
        .len();
    if bytes_len == 0 {
        return Err(StagexGawkError::Materialization("source bundle is empty".to_string()));
    }
    assert!(path.is_absolute());
    assert!(bytes_len > 0);
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexGawkError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(GAWK_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(StagexGawkError::SourceRecordNotFound);
    }
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str), Some(GAWK_RECORD_NAME));
    Ok(matches[0])
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexGawkError> {
    require_record_fact(record.identity == GAWK_RECORD_IDENTITY, "source record identity")?;
    require_record_fact(record.kind == SourceRecordKind::FixedUrl, "source record kind")?;
    require_metadata(record, "builder", "builtin:fetchurl")?;
    require_metadata(record, "hash", GAWK_RECORD_HASH)?;
    require_metadata(record, "hash_algo", "sha256")?;
    require_metadata(record, "hash_mode", "recursive")?;
    require_metadata(record, "payload_encoding", GAWK_RECORD_PAYLOAD_ENCODING)?;
    require_metadata(record, "unpack", GAWK_RECORD_UNPACK)?;
    require_metadata(record, "url", GAWK_RECORD_URL)?;
    require_record_fact(record.content_blake3 == GAWK_SOURCE_CONTENT_BLAKE3, "source content BLAKE3")?;
    require_record_fact(record.files.len() == 1, "source record file count")?;
    let archive = &record.files[0];
    require_record_fact(archive.path == "archive", "source archive path")?;
    require_record_fact(archive.file_type == SourceFileType::Regular, "source archive file type")?;
    require_record_fact(!archive.executable, "source archive mode")?;
    require_record_fact(archive.size > 0, "source archive size")?;
    require_record_fact(archive.content_hex.is_some(), "source archive bytes")?;
    assert_eq!(record.identity, GAWK_RECORD_IDENTITY);
    assert_eq!(record.files.len(), 1);
    Ok(())
}

fn require_record_fact(condition: bool, label: &str) -> Result<(), StagexGawkError> {
    if !condition {
        return Err(StagexGawkError::Materialization(format!("unexpected Gawk {label}")));
    }
    Ok(())
}

fn require_metadata(record: &SourceRecord, key: &str, expected: &str) -> Result<(), StagexGawkError> {
    let observed = record.metadata.get(key).map(String::as_str);
    if observed != Some(expected) {
        return Err(StagexGawkError::Materialization(format!(
            "unexpected Gawk source metadata {key}: expected {expected}, observed {observed:?}"
        )));
    }
    Ok(())
}

fn validate_materialized_source(path: &Path) -> Result<(), StagexGawkError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(StagexGawkError::Materialization(format!(
            "materialized Gawk source is not an absolute directory: {}",
            path.display()
        )));
    }
    for relative in REQUIRED_SOURCE_FILES {
        let source = path.join(relative);
        if !source.is_file() {
            return Err(StagexGawkError::Materialization(format!("materialized Gawk source is missing {relative}")));
        }
    }
    let has_parser = path.join("awkgram.c").is_file() || path.join("awktab.c").is_file();
    if !has_parser {
        return Err(StagexGawkError::Materialization(
            "materialized Gawk source has no pregenerated parser".to_string(),
        ));
    }
    assert_eq!(REQUIRED_SOURCE_FILES.len(), REQUIRED_SOURCE_FILE_COUNT);
    assert!(has_parser);
    Ok(())
}

pub(crate) fn validate_recipe_digest() -> Result<(), StagexGawkError> {
    let observed = blake3::hash(GAWK_RECIPE).to_hex().to_string();
    if observed != GAWK_RECIPE_BLAKE3 {
        return Err(StagexGawkError::Materialization(format!(
            "Gawk recipe BLAKE3 mismatch: expected {GAWK_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert_ne!(observed, GAWK_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

pub(crate) fn derive_gawk_inventory(request: GawkInventoryRequest<'_>) -> Result<GawkInventoryReport, StagexGawkError> {
    derive_gawk_inventory_inner(request)
}

fn derive_gawk_inventory_inner(request: GawkInventoryRequest<'_>) -> Result<GawkInventoryReport, StagexGawkError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexGawkError::Materialization(format!("creating Gawk runtime scratch: {error}")))?;
    let retained_source = request.scratch_dir.join(GAWK_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &retained_source)?;
    configure_retained_source(&retained_source)?;
    let output_bin = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_bin)
        .map_err(|error| StagexGawkError::Materialization(format!("creating Gawk output: {error}")))?;
    let gawk = build_gawk(&request, &retained_source, &output_bin)?;
    let awk = output_bin.join("awk");
    symlink("gawk", &awk).map_err(|error| StagexGawkError::Materialization(format!("creating AWK alias: {error}")))?;
    let observations = run_checked_smokes(&request, &gawk)?;
    let outputs = collect_outputs(&gawk, &awk, &observations)?;
    validate_expected_outputs(&outputs)?;
    let inventory = GawkInventoryReport {
        format: GAWK_REPORT_FORMAT,
        configured_source_digest_blake3: configured_source_digest_blake3()?,
        build_command_count: GAWK_BUILD_COMMAND_COUNT,
        smoke_command_count: GAWK_SMOKE_COMMAND_COUNT,
        negative_exit_code: observations.negative_exit_code,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: GAWK_NON_CLAIM,
    };
    let inventory_path = request.scratch_dir.join("gawk-inventory.json");
    let bytes = serde_json::to_vec_pretty(&inventory)
        .map_err(|error| StagexGawkError::Materialization(format!("serializing Gawk inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&inventory_path, &bytes)?;
    assert_eq!(inventory.outputs.len(), GAWK_OUTPUT_COUNT);
    assert!(inventory.negative_exit_code != 0);
    Ok(inventory)
}

fn validate_inventory_inputs(request: &GawkInventoryRequest<'_>) -> Result<(), StagexGawkError> {
    if request.scratch_dir.exists() {
        return Err(StagexGawkError::Materialization(format!(
            "Gawk runtime scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    validate_materialized_source(request.source_root)?;
    validate_recipe_digest()?;
    let configured_source = configured_source_digest_blake3()?;
    if configured_source != GAWK_CONFIGURED_SOURCE_BLAKE3 {
        return Err(StagexGawkError::Materialization(format!(
            "Gawk configured-source BLAKE3 mismatch: expected {GAWK_CONFIGURED_SOURCE_BLAKE3}, observed {configured_source}"
        )));
    }
    validate_file_digest(
        &request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2"),
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "TinyCC musl-v2",
    )?;
    let libc_digest = native_musl_digest("musl-native-libc")?;
    let crt1_digest = native_musl_digest("musl-native-crt1")?;
    validate_file_digest(&request.musl_native_root.join("lib/libc.a"), libc_digest, "native musl libc")?;
    validate_file_digest(&request.musl_native_root.join("lib/crt1.o"), crt1_digest, "native musl crt1")?;
    if !request.musl_native_root.join("include/stdio.h").is_file() {
        return Err(StagexGawkError::Materialization("native musl headers are missing".to_string()));
    }
    assert!(!request.scratch_dir.exists());
    assert!(request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2").is_file());
    Ok(())
}

fn configure_retained_source(source_root: &Path) -> Result<(), StagexGawkError> {
    let config = extract_recipe_block(GAWK_CONFIG_START, GAWK_CONFIG_END, "config.h")?;
    let decimal = extract_recipe_block(GAWK_DECIMAL_START, GAWK_DECIMAL_END, "mantle_decimal.c")?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), config.as_bytes())?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("mantle_decimal.c"), decimal.as_bytes())?;
    for relative in ["builtin.c", "eval.c", "main.c", "node.c", "io.c", "awktab.c"] {
        transform_source_file(source_root, relative)?;
    }
    validate_configured_source(source_root)?;
    assert!(source_root.join("config.h").is_file());
    assert!(source_root.join("mantle_decimal.c").is_file());
    Ok(())
}

fn transform_source_file(source_root: &Path, relative: &str) -> Result<(), StagexGawkError> {
    let path = source_root.join(relative);
    let bytes = crate::stagex_mes_lib::read_bounded_file(&path, GAWK_SOURCE_FILE_BYTES_MAX, relative)?;
    let input = String::from_utf8(bytes)
        .map_err(|error| StagexGawkError::Materialization(format!("reading UTF-8 Gawk source {relative}: {error}")))?;
    let transformed = transform_source_text(relative, &input)?;
    fs::write(&path, transformed.as_bytes()).map_err(|error| {
        StagexGawkError::Materialization(format!("writing configured Gawk source {relative}: {error}"))
    })?;
    assert!(path.is_file());
    assert!(!transformed.is_empty());
    Ok(())
}

fn transform_source_text(relative: &str, input: &str) -> Result<String, StagexGawkError> {
    match relative {
        "builtin.c" => transform_builtin(input),
        "eval.c" => transform_eval(input),
        "main.c" => transform_main(input),
        "node.c" => transform_node(input),
        "io.c" => transform_io(input),
        "awktab.c" => transform_awktab(input),
        _ => Err(StagexGawkError::Materialization(format!("unsupported configured Gawk source {relative}"))),
    }
}

fn transform_builtin(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    replace_exact(
        &mut text,
        "#define Floor(n) floor((n) * (1.0 + DBL_EPSILON))",
        "#define Floor(n) floor(n)",
        1,
        "builtin Floor",
    )?;
    replace_exact(
        &mut text,
        "#define Ceil(n) ceil((n) * (1.0 + DBL_EPSILON))",
        "#define Ceil(n) ceil(n)",
        1,
        "builtin Ceil",
    )?;
    replace_exact(&mut text, "arg < 0.0", "arg < (AWKNUM)0", 2, "builtin negative comparison")?;
    replace_exact(&mut text, "d_index < 1.0", "d_index < (AWKNUM)1", 1, "builtin index literal")?;
    replace_exact(&mut text, "d_length <= 0.0", "d_length <= (AWKNUM)0", 1, "builtin length literal")?;
    replace_exact(&mut text, "rlength = -1.0", "rlength = (AWKNUM)-1", 1, "builtin failed-match literal")?;
    replace_exact(&mut text, "tmp_number((AWKNUM) 0.0)", "tmp_number((AWKNUM)0)", 1, "builtin zero result")?;
    replace_exact(&mut text, "g == 0.0", "g == (AWKNUM)0", 1, "builtin format zero")?;
    replace_exact(&mut text, "AWKNUM retval = 0.0", "AWKNUM retval = (AWKNUM)0", 1, "builtin accumulator zero")?;
    replace_exact(&mut text, "int sgn;", "int sgn; int mantle_conversion_ok;", 1, "builtin conversion local")?;
    replace_exact(&mut text, "if (! (tmpval <= (unsigned long) ULONG_MAX))", "if (0)", 2, "builtin unsigned guard")?;
    replace_exact(
        &mut text,
        "uval = - (unsigned long) (long) tmpval;",
        "uval = mantle_double_to_ulong_bounded(-tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
        1,
        "builtin negative conversion",
    )?;
    replace_exact(
        &mut text,
        "uval = (unsigned long) (long) tmpval;",
        "uval = mantle_double_to_ulong_bounded(tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
        1,
        "builtin signed conversion",
    )?;
    replace_exact(
        &mut text,
        "uval = (unsigned long) tmpval;",
        "uval = mantle_double_to_ulong_bounded(tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
        2,
        "builtin positive conversion",
    )?;
    assert_ne!(text, input);
    assert!(text.contains("mantle_double_to_ulong_bounded"));
    Ok(text)
}

fn transform_eval(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    replace_exact(&mut text, "? 1.0 : -1.0", "? (AWKNUM)1 : (AWKNUM)-1", 2, "eval increment literal")?;
    replace_exact(&mut text, " != 0.0", " != (AWKNUM)0", 3, "eval zero literal")?;
    assert_ne!(text, input);
    assert!(!text.contains("? 1.0 : -1.0"));
    Ok(text)
}

fn transform_main(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    replace_exact(&mut text, "Nnull_string->numbr = 0.0;", "Nnull_string->numbr = (AWKNUM)0;", 1, "main null literal")?;
    replace_exact(&mut text, "tmp_number(0.0)", "tmp_number((AWKNUM)0)", 1, "main ARGV literal")?;
    assert_ne!(text, input);
    assert!(!text.contains("Nnull_string->numbr = 0.0;"));
    Ok(text)
}

fn transform_node(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    let old_guard = "if ((val = double_to_int(s->numbr)) != s->numbr\n\t    || val < LONG_MIN || val > LONG_MAX) {";
    replace_exact(&mut text, old_guard, "if ((val = double_to_int(s->numbr)) != s->numbr) {", 1, "node integer guard")?;
    let old_table = "\t\tif (num < NVAL && num >= 0) {\n\t\t\tsp = (char *) values[num];\n\t\t\ts->stlen = 1;\n\t\t} else {\n\t\t\t(void) sprintf(sp, \"%ld\", num);\n\t\t\ts->stlen = strlen(sp);\n\t\t}";
    let new_table = "\t\t(void) sprintf(sp, \"%ld\", num);\n\t\ts->stlen = strlen(sp);";
    replace_exact(&mut text, old_table, new_table, 1, "node digit table")?;
    replace_exact(&mut text, "n->numbr = 0.0;", "n->numbr = (AWKNUM)0;", 1, "node forced-number assignment")?;
    replace_exact(&mut text, "return 0.0;", "return (AWKNUM)0;", 3, "node zero return")?;
    assert_ne!(text, input);
    assert!(!text.contains("val < LONG_MIN || val > LONG_MAX"));
    Ok(text)
}

fn transform_io(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    replace_exact(&mut text, "tmp_number((AWKNUM) 0.0)", "tmp_number((AWKNUM)0)", 4, "I/O zero result")?;
    replace_exact(&mut text, "tmp_number((AWKNUM) -1.0)", "tmp_number((AWKNUM)-1)", 2, "I/O negative result")?;
    replace_exact(&mut text, "tmp_number((AWKNUM) 1.0)", "tmp_number((AWKNUM)1)", 1, "I/O positive result")?;
    assert_ne!(text, input);
    assert!(!text.contains("tmp_number((AWKNUM) 0.0)"));
    Ok(text)
}

fn transform_awktab(input: &str) -> Result<String, StagexGawkError> {
    let mut text = input.to_string();
    replace_exact(&mut text, "make_number(0.0)", "make_number((AWKNUM)0)", 7, "parser zero node")?;
    assert_ne!(text, input);
    assert!(!text.contains("make_number(0.0)"));
    Ok(text)
}

fn replace_exact(
    text: &mut String,
    old: &str,
    new: &str,
    expected_count: usize,
    label: &str,
) -> Result<(), StagexGawkError> {
    let observed_count = text.matches(old).count();
    if observed_count != expected_count {
        return Err(StagexGawkError::Materialization(format!(
            "unexpected {label} source shape: expected {expected_count}, observed {observed_count}"
        )));
    }
    *text = text.replace(old, new);
    if !new.contains(old) {
        assert_eq!(text.matches(old).count(), 0);
    }
    assert!(text.matches(new).count() >= expected_count);
    Ok(())
}

fn extract_recipe_block(start: &str, end: &str, label: &str) -> Result<String, StagexGawkError> {
    let recipe = std::str::from_utf8(GAWK_RECIPE)
        .map_err(|error| StagexGawkError::Materialization(format!("reading UTF-8 Gawk recipe: {error}")))?;
    let (_, suffix) = recipe
        .split_once(start)
        .ok_or_else(|| StagexGawkError::Materialization(format!("Gawk {label} start marker is missing")))?;
    let (body, _) = suffix
        .split_once(end)
        .ok_or_else(|| StagexGawkError::Materialization(format!("Gawk {label} end marker is missing")))?;
    if body.is_empty() {
        return Err(StagexGawkError::Materialization(format!("Gawk {label} block is empty")));
    }
    assert!(!body.is_empty());
    assert!(!body.contains(end));
    Ok(format!("{body}\n"))
}

fn validate_configured_source(source_root: &Path) -> Result<(), StagexGawkError> {
    let builtin = read_source_text(source_root, "builtin.c")?;
    let eval = read_source_text(source_root, "eval.c")?;
    let node = read_source_text(source_root, "node.c")?;
    let decimal = read_source_text(source_root, "mantle_decimal.c")?;
    if builtin.contains("arg < 0.0") || eval.contains("? 1.0 : -1.0") || node.contains("return 0.0;") {
        return Err(StagexGawkError::Materialization(
            "unsupported integral floating literal remained in Gawk numeric-node paths".to_string(),
        ));
    }
    if !decimal.contains("#define MANTLE_DECIMAL_INPUT_BYTES_MAX 4096") {
        return Err(StagexGawkError::Materialization("bounded Gawk decimal parser is missing".to_string()));
    }
    assert!(builtin.contains("mantle_double_to_ulong_bounded"));
    assert!(decimal.contains("mantle_decimal_parse"));
    Ok(())
}

fn read_source_text(source_root: &Path, relative: &str) -> Result<String, StagexGawkError> {
    let bytes =
        crate::stagex_mes_lib::read_bounded_file(&source_root.join(relative), GAWK_SOURCE_FILE_BYTES_MAX, relative)?;
    String::from_utf8(bytes)
        .map_err(|error| StagexGawkError::Materialization(format!("reading UTF-8 Gawk source {relative}: {error}")))
}

pub(crate) fn configured_source_digest_blake3() -> Result<String, StagexGawkError> {
    let config = extract_recipe_block(GAWK_CONFIG_START, GAWK_CONFIG_END, "config.h")?;
    let decimal = extract_recipe_block(GAWK_DECIMAL_START, GAWK_DECIMAL_END, "mantle_decimal.c")?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-gawk-configured-source-v1\0");
    hasher.update(GAWK_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(GAWK_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0exact-counted-source-normalization-v1\0");
    hasher.update(config.as_bytes());
    hasher.update(b"\0");
    hasher.update(decimal.as_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn build_gawk(
    request: &GawkInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
) -> Result<PathBuf, StagexGawkError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let include_arg = format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?);
    let common_args = [
        "-I.".to_string(),
        include_arg,
        "-DHAVE_CONFIG_H".to_string(),
        "-Dstrtod=mantle_strtod".to_string(),
        "-Datof=mantle_atof".to_string(),
    ];
    let mut objects = Vec::with_capacity(GAWK_COMPILE_SOURCES.len() + 1);
    for source in GAWK_COMPILE_SOURCES {
        objects.push(compile_gawk_source(request, source_root, &compiler, &common_args, source, false)?);
    }
    let parser = if source_root.join("awkgram.c").is_file() {
        "awkgram.c"
    } else if source_root.join("awktab.c").is_file() {
        "awktab.c"
    } else {
        return Err(StagexGawkError::Materialization("no pregenerated Gawk parser source was found".to_string()));
    };
    objects.push(compile_gawk_source(request, source_root, &compiler, &common_args, parser, parser == "awktab.c")?);
    let gawk = link_gawk(request, source_root, output_bin, &compiler, &objects)?;
    assert_eq!(objects.len(), GAWK_OBJECT_COUNT);
    assert!(gawk.is_file());
    Ok(gawk)
}

fn compile_gawk_source(
    request: &GawkInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    common_args: &[String],
    source: &str,
    define_alloca: bool,
) -> Result<String, StagexGawkError> {
    let object = format!("{}.o", source.trim_end_matches(".c"));
    let mut args = vec!["-c".to_string()];
    args.extend_from_slice(common_args);
    if define_alloca {
        args.push("-Dalloca=malloc".to_string());
    }
    args.push(source.to_string());
    args.push("-o".to_string());
    args.push(object.clone());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("gawk-{}-compile.stderr.txt", source.trim_end_matches(".c"))),
    )?;
    let object_path = source_root.join(&object);
    add_owner_read_write_mode(&object_path)?;
    validate_nonempty_artifact(&object_path, "Gawk object")?;
    assert_eq!(
        object_path
            .metadata()
            .map(|value| value.permissions().mode() & GAWK_OBJECT_OWNER_READ_WRITE_MASK)
            .unwrap_or(0),
        GAWK_OBJECT_OWNER_READ_WRITE_MASK
    );
    assert!(source_root.join(source).is_file());
    Ok(object)
}

fn add_owner_read_write_mode(path: &Path) -> Result<(), StagexGawkError> {
    let metadata = path
        .metadata()
        .map_err(|error| StagexGawkError::Materialization(format!("reading Gawk object mode: {error}")))?;
    let initial_mode = metadata.permissions().mode() & FILE_MODE_MASK;
    let configured_mode = initial_mode | GAWK_OBJECT_OWNER_READ_WRITE_MASK;
    fs::set_permissions(path, fs::Permissions::from_mode(configured_mode))
        .map_err(|error| StagexGawkError::Materialization(format!("setting Gawk object mode: {error}")))?;
    assert_eq!(configured_mode & GAWK_OBJECT_OWNER_READ_WRITE_MASK, GAWK_OBJECT_OWNER_READ_WRITE_MASK);
    assert!(configured_mode >= initial_mode);
    Ok(())
}

fn link_gawk(
    request: &GawkInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
    compiler: &Path,
    objects: &[String],
) -> Result<PathBuf, StagexGawkError> {
    let gawk = output_bin.join("gawk");
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_absolute(&gawk, "Gawk output")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("gawk-link.stderr.txt"),
    )?;
    fs::set_permissions(&gawk, fs::Permissions::from_mode(GAWK_EXECUTABLE_MODE))
        .map_err(|error| StagexGawkError::Materialization(format!("setting Gawk executable mode: {error}")))?;
    validate_nonempty_artifact(&gawk, "Gawk executable")?;
    assert_eq!(objects.len(), GAWK_OBJECT_COUNT);
    assert!(
        gawk.metadata()
            .map(|value| value.permissions().mode() & GAWK_EXECUTE_MODE_MASK != 0)
            .unwrap_or(false)
    );
    Ok(gawk)
}

struct GawkObservations {
    behavior: PathBuf,
    numeric: PathBuf,
    decimal: PathBuf,
    length: PathBuf,
    hex: PathBuf,
    negative: PathBuf,
    installed: PathBuf,
    negative_exit_code: i32,
}

fn run_checked_smokes(request: &GawkInventoryRequest<'_>, gawk: &Path) -> Result<GawkObservations, StagexGawkError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexGawkError::Materialization(format!("creating Gawk smoke root: {error}")))?;
    let behavior = run_gawk_smoke(
        gawk,
        &smoke,
        "behavior",
        &[
            "BEGIN { label = toupper(\"awk\"); total = 2 + 5 } /a/ { matches++ } END { print total \":\" matches \":\" label }",
        ],
        b"alpha\nbeta\n",
    )?;
    require_exact_bytes(&behavior, b"7:2:AWK\n", "Gawk behavior observation")?;
    let numeric = run_gawk_smoke(
        gawk,
        &smoke,
        "numeric",
        &["BEGIN { n = 0; values[++n] = \"x\"; print n \":\" values[1] }"],
        b"",
    )?;
    require_exact_bytes(&numeric, b"1:x\n", "Gawk numeric observation")?;
    let decimal = run_gawk_smoke(gawk, &smoke, "decimal", &["BEGIN { product = 1.5 * 2.0; print product }"], b"")?;
    require_exact_bytes(&decimal, b"3\n", "Gawk decimal observation")?;
    let length = run_gawk_smoke(gawk, &smoke, "length", &["BEGIN { print length(\"S?\") }"], b"")?;
    require_exact_bytes(&length, b"2\n", "Gawk length observation")?;
    let hex = run_gawk_smoke(
        gawk,
        &smoke,
        "hex",
        &[
            "-v",
            "value=68",
            "-v",
            "format=%08x",
            "BEGIN { printf format \"\\n\", value }",
        ],
        b"",
    )?;
    require_exact_bytes(&hex, b"00000044\n", "Gawk hexadecimal observation")?;
    let (negative, negative_exit_code) = run_negative_smoke(gawk, &smoke)?;
    let installed = run_gawk_smoke(gawk, &smoke, "installed", &["{print \"awk-ok\"}"], b"test\n")?;
    require_exact_bytes(&installed, b"awk-ok\n", "installed Gawk observation")?;
    assert!(negative_exit_code != 0);
    assert!(negative.is_file());
    Ok(GawkObservations {
        behavior,
        numeric,
        decimal,
        length,
        hex,
        negative,
        installed,
        negative_exit_code,
    })
}

fn run_gawk_smoke(
    gawk: &Path,
    smoke: &Path,
    label: &str,
    arguments: &[&str],
    stdin_bytes: &[u8],
) -> Result<PathBuf, StagexGawkError> {
    let stdin_path = smoke.join(format!("{label}.in"));
    let stdout_path = smoke.join(format!("{label}.out"));
    let stderr_path = smoke.join(format!("{label}.stderr.txt"));
    crate::stagex_mes_lib::write_create_new(&stdin_path, stdin_bytes)?;
    let args = arguments.iter().map(|argument| (*argument).to_string()).collect::<Vec<_>>();
    crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout(
        gawk,
        &args,
        smoke,
        &gawk_environment(),
        &stdin_path,
        GAWK_OBSERVATION_BYTES_MAX,
        &stdout_path,
        GAWK_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(stdout_path)
}

fn run_negative_smoke(gawk: &Path, smoke: &Path) -> Result<(PathBuf, i32), StagexGawkError> {
    let stdin_path = smoke.join("negative.in");
    let stdout_path = smoke.join("negative.stdout.txt");
    let stderr_path = smoke.join("negative.stderr.txt");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"")?;
    let args = ["BEGIN {".to_string()];
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        gawk,
        &args,
        smoke,
        &gawk_environment(),
        &stdin_path,
        GAWK_OBSERVATION_BYTES_MAX,
        &stdout_path,
        GAWK_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    if exit_code == 0 {
        return Err(StagexGawkError::Materialization("invalid Gawk program unexpectedly succeeded".to_string()));
    }
    require_exact_bytes(&stdout_path, b"", "Gawk negative stdout")?;
    validate_nonempty_artifact(&stderr_path, "Gawk negative diagnostic")?;
    assert_ne!(exit_code, 0);
    assert!(stderr_path.is_file());
    Ok((stderr_path, exit_code))
}

fn gawk_environment() -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    environment.insert("LC_ALL".to_string(), "C".to_string());
    assert_eq!(environment.len(), 1);
    assert_eq!(environment.get("LC_ALL").map(String::as_str), Some("C"));
    environment
}

fn collect_outputs(
    gawk: &Path,
    awk: &Path,
    observations: &GawkObservations,
) -> Result<Vec<GawkOutputReport>, StagexGawkError> {
    let paths = [
        ("gawk-3.0.4", gawk),
        ("awk-3.0.4-alias", awk),
        ("gawk-behavior-observation", observations.behavior.as_path()),
        ("gawk-numeric-observation", observations.numeric.as_path()),
        ("gawk-decimal-observation", observations.decimal.as_path()),
        ("gawk-length-observation", observations.length.as_path()),
        ("gawk-hex-observation", observations.hex.as_path()),
        ("gawk-negative-observation", observations.negative.as_path()),
        ("gawk-installed-observation", observations.installed.as_path()),
    ];
    let outputs = paths
        .iter()
        .map(|(artifact_id, path)| output_report(artifact_id, path))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), GAWK_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[GawkOutputReport]) -> Result<(), StagexGawkError> {
    if outputs.len() != GAWK_EXPECTED_OUTPUTS.len() {
        return Err(StagexGawkError::Materialization(format!(
            "expected {} Gawk outputs, observed {}",
            GAWK_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in GAWK_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexGawkError::Materialization(format!("Gawk output is missing: {}", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            return Err(StagexGawkError::Materialization(format!(
                "Gawk output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, observed.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), GAWK_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(())
}

fn output_report(artifact_id: &str, path: &Path) -> Result<GawkOutputReport, StagexGawkError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, GAWK_ARTIFACT_BYTES_MAX, artifact_id)?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexGawkError::Materialization(format!("Gawk artifact is too large: {artifact_id}")))?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert!(bytes_len <= GAWK_ARTIFACT_BYTES_MAX);
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(GawkOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexGawkError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexGawkError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() > GAWK_OBSERVATION_BYTES_MAX {
        return Err(StagexGawkError::Materialization(format!(
            "{label} is not regular or exceeds {GAWK_OBSERVATION_BYTES_MAX} bytes"
        )));
    }
    let observed =
        fs::read(path).map_err(|error| StagexGawkError::Materialization(format!("reading {label}: {error}")))?;
    if observed != expected {
        return Err(StagexGawkError::Materialization(format!(
            "{label} mismatch: expected {expected:?}, observed {observed:?}"
        )));
    }
    assert_eq!(u64::try_from(observed.len()).unwrap_or(u64::MAX), metadata.len());
    assert_eq!(observed, expected);
    Ok(())
}

fn validate_nonempty_artifact(path: &Path, label: &str) -> Result<(), StagexGawkError> {
    let metadata = path
        .metadata()
        .map_err(|error| StagexGawkError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > GAWK_ARTIFACT_BYTES_MAX {
        return Err(StagexGawkError::Materialization(format!("invalid {label}: {}", path.display())));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() <= GAWK_ARTIFACT_BYTES_MAX);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexGawkError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, GAWK_ARTIFACT_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexGawkError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(())
}

fn native_musl_digest(artifact_id: &str) -> Result<&'static str, StagexGawkError> {
    let digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.digest_blake3)
        .ok_or_else(|| StagexGawkError::Materialization(format!("native musl inventory is missing {artifact_id}")))?;
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexGawkError> {
    if !path.is_absolute() {
        return Err(StagexGawkError::Materialization(format!("{label} path is not absolute: {}", path.display())));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexGawkError::Materialization(format!("{label} path is not UTF-8")))?;
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
    fn validates_exact_gawk_source_record_and_recipe() {
        let bundle = match std::env::var(SOURCE_BUNDLE_ENV) {
            Ok(value) => value,
            Err(_) => return,
        };
        let manifest = read_source_bundle(Path::new(&bundle)).unwrap();
        let record = find_source_record(&manifest.records).unwrap();
        validate_source_record(record).unwrap();
        validate_recipe_digest().unwrap();
        assert_eq!(record.content_blake3, GAWK_SOURCE_CONTENT_BLAKE3);
        assert_eq!(source_artifact_digests().len(), GAWK_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_gawk_source_authority() {
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
    fn applies_exact_gawk_source_transformations() {
        let source_root = match std::env::var("MANTLE_STAGE_X_GAWK_RETAINED_SOURCE") {
            Ok(value) => PathBuf::from(value),
            Err(_) => return,
        };
        for relative in ["builtin.c", "eval.c", "main.c", "node.c", "io.c", "awktab.c"] {
            let input = fs::read_to_string(source_root.join(relative)).unwrap();
            let output = transform_source_text(relative, &input).unwrap();
            assert_ne!(output, input);
            assert!(!output.is_empty());
        }
        assert_eq!(configured_source_digest_blake3().unwrap().len(), BLAKE3_HEX_CHAR_COUNT);
        assert!(
            extract_recipe_block(GAWK_DECIMAL_START, GAWK_DECIMAL_END, "decimal")
                .unwrap()
                .contains("mantle_decimal_parse")
        );
    }

    #[test]
    fn pins_deterministic_gawk_identities() {
        assert_eq!(configured_source_digest_blake3().unwrap(), GAWK_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(GAWK_EXPECTED_OUTPUTS.len(), GAWK_OUTPUT_COUNT);
        assert_eq!(GAWK_EXPECTED_OUTPUTS[0].digest_blake3, GAWK_BINARY_BLAKE3);
        assert_ne!(GAWK_BINARY_BLAKE3, GAWK_SOURCE_CONTENT_BLAKE3);
    }

    #[test]
    fn rejects_substituted_gawk_output_identity() {
        let outputs = GAWK_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| GawkOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        let mut substituted = outputs;
        substituted[0].digest_blake3 = GAWK_SOURCE_CONTENT_BLAKE3.to_string();
        let error = validate_expected_outputs(&substituted).unwrap_err().to_string();
        assert!(error.contains("BLAKE3 mismatch"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    fn rejects_gawk_source_shape_substitution() {
        let error = replace_exact(&mut "safe".to_string(), "missing", "new", 1, "fixture").unwrap_err().to_string();
        assert!(error.contains("expected 1, observed 0"));
        assert!(!error.contains("panicked"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_gawk_source() {
        let bundle = PathBuf::from(std::env::var(SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(
            std::env::var("MANTLE_STAGE_X_GAWK_SOURCE_SCRATCH")
                .unwrap_or_else(|_| "/tmp/mantle-stagex-gawk-source".to_string()),
        );
        let report = materialize_authenticated_gawk_source(&bundle, SOURCE_MANIFEST_BLAKE3, &scratch).unwrap();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        assert_eq!(report.record_content_blake3, GAWK_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("builtin.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU Gawk source, TinyCC musl-v2, native musl, and create-new scratch"]
    fn derives_retained_gawk_inventory() {
        let source_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_GAWK_RETAINED_SOURCE").unwrap());
        let tcc_musl_v2_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap());
        let musl_native_root = PathBuf::from(std::env::var("MANTLE_STAGE_X_NATIVE_MUSL_ROOT").unwrap());
        let scratch_dir = PathBuf::from(std::env::var("MANTLE_STAGE_X_GAWK_RUNTIME_SCRATCH").unwrap());
        let request = GawkInventoryRequest {
            source_root: &source_root,
            tcc_musl_v2_root: &tcc_musl_v2_root,
            musl_native_root: &musl_native_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        };
        let report = derive_gawk_inventory_inner(request).unwrap();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        println!("configured-source={}", configured_source_digest_blake3().unwrap());
        assert_eq!(report.build_command_count, GAWK_BUILD_COMMAND_COUNT);
        assert_eq!(report.outputs.len(), GAWK_OUTPUT_COUNT);
    }
}
