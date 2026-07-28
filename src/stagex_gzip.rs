use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const GZIP_RECORD_NAME: &str = "gzip-1.2.4-src";
const GZIP_SOURCE_OUTPUT_NAME: &str = "gzip-1.2.4";
pub(crate) const GZIP_SOURCE_ARTIFACT_ID: &str = "gzip-1.2.4-source";
pub(crate) const GZIP_SOURCE_CONTENT_BLAKE3: &str = "de8516b8bfc78a91f5d37837ff70d6926f4b03cf68924ae7ef6ed602f7f316fd";
pub(crate) const GZIP_MAKECRC_SOURCE_ARTIFACT_ID: &str = "gzip-makecrc-source";
pub(crate) const GZIP_MAKECRC_SOURCE_BLAKE3: &str = "48240f8bb743e9fd202f7bc2d98367607f853d2ed7483d484f50892bc69d9ef1";
const GZIP_MAKECRC_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/gzip-makecrc.c");
const GZIP_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-gzip-source-materialization-v1";
const GZIP_REPORT_FORMAT: &str = "mantle-stagex-gzip-1.2.4-inventory-v1";
const GZIP_SOURCE_NON_CLAIM: &str =
    "gzip source materialization proves authenticated offline archive identity and fixed-output parity only";
const GZIP_NON_CLAIM: &str = "this inventory binds gzip 1.2.4, its source-built CRC generator, and positive and negative smoke observations only; it does not prove later bootstrap utilities or provider admission";
const GZIP_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const GZIP_SOURCE_COMPILE_COUNT: u32 = 14;
const GZIP_GENERATOR_BUILD_COMMAND_COUNT: u32 = 1;
const GZIP_GENERATOR_RUN_COMMAND_COUNT: u32 = 1;
const GZIP_BUILD_COMMAND_COUNT: u32 = GZIP_SOURCE_COMPILE_COUNT + 1;
const GZIP_SMOKE_COMMAND_COUNT: u32 = 4;
const GZIP_OUTPUT_COUNT: usize = 5;
const GZIP_SOURCE_ARTIFACT_COUNT: usize = 2;
const GZIP_CRC_TABLE_START: &str = "ulg crc_32_tab";
const GZIP_CRC_TABLE_END: &str = "};";
const GZIP_SMOKE_INPUT: &[u8] = b"gzip smoke\n";
const GZIP_MALFORMED_INPUT: &[u8] = b"not a gzip stream\n";
pub(crate) const GZIP_MAKECRC_BLAKE3: &str = "dd10648de513f76a1ecdbfa23cc88953f8c6e46848311a6372de7498aed43c6e";
const GZIP_CRC_TABLE_BLAKE3: &str = "19a3f07c4b071fe898927d1da2c3b64d37e02b0e8cf309d2a9fc4d70ac9d8fa1";
pub(crate) const GZIP_FINAL_BLAKE3: &str = "d2939461715a694222c1a612c0908754e599eb2d268fe012c525845367eff04a";
const GZIP_SMOKE_BLAKE3: &str = "78abaccbae9efa7039ff82d80215d2fbd3c4cd0221dadd85dd9df731f15d8751";
const GZIP_SOURCE_NAMES: [&str; GZIP_SOURCE_COMPILE_COUNT as usize] = [
    "gzip", "bits", "crypt", "deflate", "getopt", "inflate", "lzw", "trees", "unlzh", "unlzw", "unpack", "unzip",
    "util", "zip",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GzipExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const GZIP_EXPECTED_OUTPUTS: [GzipExpectedOutput; GZIP_OUTPUT_COUNT] = [
    GzipExpectedOutput {
        artifact_id: "gzip-makecrc",
        digest_blake3: GZIP_MAKECRC_BLAKE3,
    },
    GzipExpectedOutput {
        artifact_id: "gzip-crc-table",
        digest_blake3: GZIP_CRC_TABLE_BLAKE3,
    },
    GzipExpectedOutput {
        artifact_id: "gzip-1.2.4",
        digest_blake3: GZIP_FINAL_BLAKE3,
    },
    GzipExpectedOutput {
        artifact_id: "gunzip-1.2.4",
        digest_blake3: GZIP_FINAL_BLAKE3,
    },
    GzipExpectedOutput {
        artifact_id: "gzip-smoke",
        digest_blake3: GZIP_SMOKE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GzipSourceMaterializationReport {
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
pub(crate) struct GzipOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GzipInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub generator_build_command_count: u32,
    pub generator_run_command_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<GzipOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GzipInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexGzipError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexGzipError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "gzip source record was not found"),
            Self::Materialization(message) => write!(formatter, "gzip materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "gzip runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexGzipError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexGzipError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexGzipError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); GZIP_SOURCE_ARTIFACT_COUNT] {
    [
        (GZIP_SOURCE_ARTIFACT_ID, GZIP_SOURCE_CONTENT_BLAKE3),
        (GZIP_MAKECRC_SOURCE_ARTIFACT_ID, GZIP_MAKECRC_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_gzip_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<GzipSourceMaterializationReport, StagexGzipError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexGzipError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexGzipError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexGzipError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexGzipError::Materialization(format!("creating gzip source scratch: {error}")))?;
    let output_path = scratch_dir.join(GZIP_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexGzipError::Materialization(format!("materializing gzip source {}: {error}", record.identity))
    })?;
    if !output_path.join("gzip.c").is_file() || !output_path.join("util.c").is_file() {
        return Err(StagexGzipError::Materialization("materialized gzip tree lacks gzip.c or util.c".to_string()));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, GZIP_SOURCE_CONTENT_BLAKE3);
    Ok(GzipSourceMaterializationReport {
        format: GZIP_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: GZIP_SOURCE_ARTIFACT_ID,
        record_name: GZIP_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: GZIP_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_gzip_inventory(request: GzipInventoryRequest<'_>) -> Result<GzipInventoryReport, StagexGzipError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexGzipError::Materialization(format!("creating gzip scratch: {error}")))?;
    let source_root = request.scratch_dir.join("gzip-1.2.4");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    prepare_crc_source(&request, &source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3(&source_root)?;
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexGzipError::Materialization(format!("creating gzip output: {error}")))?;
    let gzip = build_gzip(&request, &source_root, &output_root)?;
    let gunzip = output_root.join("gunzip");
    fs::copy(&gzip, &gunzip)
        .map_err(|error| StagexGzipError::Materialization(format!("copying gunzip alias: {error}")))?;
    crate::stagex_tinycc::set_owner_executable(&gunzip)?;
    run_gzip_smokes(&request, &source_root, &gzip, &gunzip)?;
    let outputs = collect_outputs(&request, &source_root, &gzip, &gunzip)?;
    validate_expected_outputs(&outputs)?;
    let report = GzipInventoryReport {
        format: GZIP_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: GZIP_SOURCE_COMPILE_COUNT,
        generator_build_command_count: GZIP_GENERATOR_BUILD_COMMAND_COUNT,
        generator_run_command_count: GZIP_GENERATOR_RUN_COMMAND_COUNT,
        build_command_count: GZIP_BUILD_COMMAND_COUNT,
        smoke_command_count: GZIP_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: GZIP_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("gzip-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexGzipError::Materialization(format!("serializing gzip report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), GZIP_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &GzipInventoryRequest<'_>) -> Result<(), StagexGzipError> {
    if request.scratch_dir.exists() {
        return Err(StagexGzipError::Materialization(format!(
            "create-new gzip scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("gzip source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexGzipError::Materialization(format!(
                "{label} root is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.tinycc27_root.join("bin/tcc"),
        crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
        "TinyCC 0.9.27 compiler",
    )?;
    validate_bound_bytes(GZIP_MAKECRC_SOURCE, GZIP_MAKECRC_SOURCE_BLAKE3, "gzip makecrc source")?;
    assert!(request.source_root.join("gzip.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn prepare_crc_source(request: &GzipInventoryRequest<'_>, source_root: &Path) -> Result<(), StagexGzipError> {
    let util_path = source_root.join("util.c");
    let util_bytes = fs::read(&util_path)
        .map_err(|error| StagexGzipError::Materialization(format!("reading gzip util.c: {error}")))?;
    let stripped = strip_crc_table(&util_bytes)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("makecrc.c"), GZIP_MAKECRC_SOURCE)?;
    let makecrc = request.scratch_dir.join("generated/bin/makecrc");
    fs::create_dir_all(makecrc.parent().expect("makecrc has parent"))
        .map_err(|error| StagexGzipError::Materialization(format!("creating gzip generator output: {error}")))?;
    let args = static_link_args(request.tinycc27_root, &makecrc, &["makecrc.c"])?;
    crate::stagex_mes_lib::run_bounded_process(
        &request.tinycc27_root.join("bin/tcc"),
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("gzip-makecrc-build.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&makecrc)?;
    crate::stagex_mes_lib::run_bounded_process(
        &makecrc,
        &[] as &[&str],
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("gzip-makecrc-run.stderr.txt"),
    )?;
    let crc = fs::read(source_root.join("crc.c"))
        .map_err(|error| StagexGzipError::Materialization(format!("reading generated gzip CRC table: {error}")))?;
    let mut combined = stripped;
    combined.extend_from_slice(&crc);
    fs::write(&util_path, &combined)
        .map_err(|error| StagexGzipError::Materialization(format!("writing configured gzip util.c: {error}")))?;
    assert!(!crc.is_empty());
    assert!(makecrc.is_file());
    Ok(())
}

fn strip_crc_table(input: &[u8]) -> Result<Vec<u8>, StagexGzipError> {
    let text = std::str::from_utf8(input)
        .map_err(|error| StagexGzipError::Materialization(format!("gzip util.c is not UTF-8: {error}")))?;
    let mut output = String::new();
    let mut inside = false;
    let mut range_count = 0_u32;
    for line in text.split_inclusive('\n') {
        if !inside && line.trim_start().starts_with(GZIP_CRC_TABLE_START) {
            inside = true;
            range_count = range_count.saturating_add(1);
            continue;
        }
        if inside {
            if line.trim() == GZIP_CRC_TABLE_END {
                inside = false;
            }
            continue;
        }
        output.push_str(line);
    }
    if inside || range_count != 1 {
        return Err(StagexGzipError::Materialization(format!(
            "gzip util.c CRC table range count is {range_count}; expected one closed range"
        )));
    }
    assert!(!output.is_empty());
    assert!(output.len() < text.len());
    Ok(output.into_bytes())
}

fn build_gzip(
    request: &GzipInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexGzipError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    for (index, source_name) in GZIP_SOURCE_NAMES.iter().enumerate() {
        let args = compile_args(request.tinycc27_root, source_name)?;
        crate::stagex_mes_lib::run_bounded_process(
            &compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("gzip-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&source_root.join(format!("{source_name}.o")), source_name)?;
    }
    let output = output_root.join("gzip");
    let objects = GZIP_SOURCE_NAMES.iter().map(|name| format!("{name}.o")).collect::<Vec<_>>();
    let object_refs = objects.iter().map(String::as_str).collect::<Vec<_>>();
    let args = static_link_args(request.tinycc27_root, &output, &object_refs)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("gzip-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "gzip 1.2.4")?;
    assert_eq!(GZIP_SOURCE_NAMES.len(), usize::try_from(GZIP_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn compile_args(tinycc27_root: &Path, source_name: &str) -> Result<Vec<String>, StagexGzipError> {
    let include_path = tinycc27_root.join("include/mes");
    let include = crate::stagex_mes_lib::utf8_absolute(&include_path, "TinyCC include")?;
    let args = vec![
        "-I".to_string(),
        include.to_string(),
        "-I.".to_string(),
        "-DNO_UTIME".to_string(),
        "-Dstrlwr=unused".to_string(),
        "-c".to_string(),
        "-o".to_string(),
        format!("{source_name}.o"),
        format!("{source_name}.c"),
    ];
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(!source_name.contains('/'));
    Ok(args)
}

fn static_link_args(tinycc27_root: &Path, output: &Path, inputs: &[&str]) -> Result<Vec<String>, StagexGzipError> {
    let libdir = tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let mut args = vec![
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC runtime")?.to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "gzip static output")?.to_string(),
    ];
    args.extend(inputs.iter().map(|input| (*input).to_string()));
    args.extend([
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ]);
    assert!(!inputs.is_empty());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn run_gzip_smokes(
    request: &GzipInventoryRequest<'_>,
    source_root: &Path,
    gzip: &Path,
    gunzip: &Path,
) -> Result<(), StagexGzipError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexGzipError::Materialization(format!("creating gzip smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("smoke.txt"), GZIP_SMOKE_INPUT)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.gz"), GZIP_MALFORMED_INPUT)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        gzip,
        &["--help"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("gzip-help.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        gzip,
        &["-f", "smoke/smoke.txt"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("gzip-compress.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        gunzip,
        &["-f", "smoke/smoke.txt.gz"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("gzip-decompress.stderr.txt"),
    )?;
    require_expected_process_failure(
        gunzip,
        &["-f", "smoke/malformed.gz"],
        source_root,
        &request.scratch_dir.join("gzip-malformed.stderr.txt"),
    )?;
    let observed = fs::read(smoke_root.join("smoke.txt"))
        .map_err(|error| StagexGzipError::Materialization(format!("reading gzip smoke output: {error}")))?;
    if observed != GZIP_SMOKE_INPUT {
        return Err(StagexGzipError::Materialization("gzip roundtrip output mismatch".to_string()));
    }
    assert_eq!(observed, GZIP_SMOKE_INPUT);
    assert!(smoke_root.join("smoke.txt").is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexGzipError> {
    match crate::stagex_mes_lib::run_bounded_process(
        executable,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        stderr_path,
    ) {
        Err(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(exit_code),
            stderr,
            ..
        }) if exit_code != 0 && !stderr.trim().is_empty() => {
            assert_ne!(exit_code, 0);
            assert!(!stderr.trim().is_empty());
            Ok(())
        }
        Ok(()) => Err(StagexGzipError::Materialization("gunzip accepted malformed input".to_string())),
        Err(error) => Err(StagexGzipError::Runtime(error)),
    }
}

fn collect_outputs(
    request: &GzipInventoryRequest<'_>,
    source_root: &Path,
    gzip: &Path,
    gunzip: &Path,
) -> Result<Vec<GzipOutputReport>, StagexGzipError> {
    let paths = [
        ("gzip-makecrc", request.scratch_dir.join("generated/bin/makecrc")),
        ("gzip-crc-table", source_root.join("crc.c")),
        ("gzip-1.2.4", gzip.to_path_buf()),
        ("gunzip-1.2.4", gunzip.to_path_buf()),
        ("gzip-smoke", source_root.join("smoke/smoke.txt")),
    ];
    let mut outputs = Vec::with_capacity(GZIP_OUTPUT_COUNT);
    for (artifact_id, path) in paths {
        let metadata = fs::metadata(&path)
            .map_err(|error| StagexGzipError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(GzipOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.clone(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(&path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), GZIP_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[GzipOutputReport]) -> Result<(), StagexGzipError> {
    if outputs.len() != GZIP_EXPECTED_OUTPUTS.len() {
        return Err(StagexGzipError::Materialization(format!(
            "gzip output count mismatch: expected {}, observed {}",
            GZIP_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in GZIP_EXPECTED_OUTPUTS {
        let Some(output) = outputs.iter().find(|output| output.artifact_id == expected.artifact_id) else {
            return Err(StagexGzipError::Materialization(format!("gzip output {} is missing", expected.artifact_id)));
        };
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexGzipError::Materialization(format!(
                "gzip output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), GZIP_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexGzipError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(GZIP_RECORD_NAME))
        .ok_or(StagexGzipError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexGzipError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != GZIP_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexGzipError::Materialization(format!(
            "gzip source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexGzipError::Materialization(
            "gzip source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, GZIP_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3(source_root: &Path) -> Result<String, StagexGzipError> {
    let util = fs::read(source_root.join("util.c"))
        .map_err(|error| StagexGzipError::Materialization(format!("reading configured gzip util.c: {error}")))?;
    let crc = fs::read(source_root.join("crc.c"))
        .map_err(|error| StagexGzipError::Materialization(format!("reading generated gzip crc.c: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(GZIP_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(GZIP_MAKECRC_SOURCE_BLAKE3.as_bytes());
    hasher.update(&util);
    hasher.update(&crc);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!util.is_empty());
    Ok(digest)
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexGzipError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexGzipError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexGzipError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexGzipError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexGzipError> {
    let metadata =
        fs::metadata(path).map_err(|error| StagexGzipError::Materialization(format!("reading {label}: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > GZIP_FILE_BYTES_MAX {
        return Err(StagexGzipError::Materialization(format!(
            "{label} is absent, empty, or exceeds {GZIP_FILE_BYTES_MAX} bytes"
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= GZIP_FILE_BYTES_MAX);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexGzipError> {
    validate_nonempty_file(path, label)?;
    let bytes =
        fs::read(path).map_err(|error| StagexGzipError::Materialization(format!("reading {label} bytes: {error}")))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_artifact_digests_are_closed() {
        let digests = source_artifact_digests();
        assert_eq!(digests.len(), GZIP_SOURCE_ARTIFACT_COUNT);
        assert!(digests.iter().all(|(_, digest)| digest.len() == blake3::OUT_LEN * 2));
        assert!(digests.iter().all(|(_, digest)| digest.bytes().all(|byte| byte.is_ascii_hexdigit())));
        assert_ne!(digests[0].0, digests[1].0);
    }

    #[test]
    fn crc_table_rewrite_is_bounded_and_fail_closed() {
        let input = b"before\nulg crc_32_tab[] = {\n  0x0L,\n};\nafter\n";
        let output = strip_crc_table(input).unwrap();
        assert_eq!(output, b"before\nafter\n");
        assert!(!output.windows(GZIP_CRC_TABLE_START.len()).any(|window| window == GZIP_CRC_TABLE_START.as_bytes()));

        let missing = strip_crc_table(b"before\nafter\n").unwrap_err();
        assert!(missing.to_string().contains("expected one"));
        assert!(!missing.to_string().is_empty());
    }

    #[test]
    fn substituted_output_is_rejected() {
        let outputs = GZIP_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| GzipOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(format!("/stagex/{}", expected.artifact_id)),
                bytes_len: 1,
                digest_blake3: if expected.artifact_id == "gzip-1.2.4" {
                    "5".repeat(blake3::OUT_LEN * 2)
                } else {
                    expected.digest_blake3.to_string()
                },
            })
            .collect::<Vec<_>>();
        let error = validate_expected_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_gzip_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_GZIP_SOURCE_SCRATCH").unwrap());
        let report = materialize_authenticated_gzip_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, GZIP_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("gzip.c").is_file());
    }

    #[test]
    #[ignore = "requires retained gzip source and TinyCC 0.9.27"]
    fn derives_retained_gzip_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_GZIP_SOURCE_ROOT").unwrap());
        let tinycc27 = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_GZIP_BUILD_SCRATCH").unwrap());
        let report = derive_gzip_inventory(GzipInventoryRequest {
            source_root: &source,
            tinycc27_root: &tinycc27,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.outputs.len(), GZIP_OUTPUT_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
