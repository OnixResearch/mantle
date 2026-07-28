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
const OYACC_RECORD_NAME: &str = "oyacc-6.6-src";
const OYACC_SOURCE_OUTPUT_NAME: &str = "oyacc-6.6";
pub(crate) const OYACC_SOURCE_ARTIFACT_ID: &str = "oyacc-6.6-source";
pub(crate) const OYACC_SOURCE_CONTENT_BLAKE3: &str = "b1ea74a498a000f5f27160860fd44042d738295c78c1db56a4e0fdd10ea5ee76";
pub(crate) const OYACC_SMOKE_SOURCE_ARTIFACT_ID: &str = "oyacc-smoke-grammar-source";
pub(crate) const OYACC_SMOKE_SOURCE_BLAKE3: &str = "5c61da581cf9442bc3da26780e417e3918a8a12c9c9be4d846607e7f5f994934";
const OYACC_SMOKE_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/oyacc-smoke.y");
pub(crate) const OYACC_MALFORMED_SOURCE_ARTIFACT_ID: &str = "oyacc-malformed-grammar-source";
pub(crate) const OYACC_MALFORMED_SOURCE_BLAKE3: &str =
    "dbab9cbaaae8309d582edff2ed8c25748316c0364c7049333eb616056033228a";
const OYACC_MALFORMED_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/oyacc-malformed.y");
pub(crate) const OYACC_EMPTY_CONFIG_ARTIFACT_ID: &str = "oyacc-empty-config-source";
pub(crate) const OYACC_EMPTY_CONFIG_BLAKE3: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
const OYACC_MESLIBC_PATCH_ARTIFACT_ID: &str = "oyacc-meslibc-patch-source";
const OYACC_MESLIBC_PATCH_BLAKE3: &str = "776fde281b132b5f38011466151a96f40fa1d25c0132c00f4e7c64ce44101c31";
const OYACC_MESLIBC_PATCH: &[u8] = include_bytes!("../bootstrap/seeds/oyacc-6.6-meslibc.patch");
const OYACC_TCC_PATCH_ARTIFACT_ID: &str = "oyacc-tcc-patch-source";
const OYACC_TCC_PATCH_BLAKE3: &str = "56c7df4bfa6adbf223a14fedfdc56b82d3c0fc702d6d88d5f54de158dcea0706";
const OYACC_TCC_PATCH: &[u8] = include_bytes!("../bootstrap/seeds/oyacc-6.6-tcc.patch");
const OYACC_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-oyacc-source-materialization-v1";
const OYACC_REPORT_FORMAT: &str = "mantle-stagex-oyacc-6.6-inventory-v1";
const OYACC_SOURCE_NON_CLAIM: &str =
    "oyacc source materialization proves authenticated offline archive identity and fixed-output parity only";
const OYACC_NON_CLAIM: &str = "this inventory binds oyacc 6.6 and positive and negative parser-generation observations only; it does not prove the later shell or provider admission";
const OYACC_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const OYACC_PATCH_COMMAND_COUNT: u32 = 2;
const OYACC_SOURCE_COMPILE_COUNT: u32 = 13;
const OYACC_LINK_COMMAND_COUNT: u32 = 1;
const OYACC_BUILD_COMMAND_COUNT: u32 = OYACC_SOURCE_COMPILE_COUNT + OYACC_LINK_COMMAND_COUNT;
const OYACC_SMOKE_COMMAND_COUNT: u32 = 2;
const OYACC_OUTPUT_COUNT: usize = 2;
const OYACC_SOURCE_ARTIFACT_COUNT: usize = 6;
pub(crate) const OYACC_CONFIGURED_SOURCE_BLAKE3: &str =
    "f4cde58315dcbfc49c47928f9258debd030140c54e249cb2a11beef1eba30296";
pub(crate) const OYACC_FINAL_BLAKE3: &str = "0b073649ea023167a293495f25302cc3debc4dd6727d9214ce6be9361503aa50";
const OYACC_SMOKE_OUTPUT_BLAKE3: &str = "6e719a5fc57aa5e8a06b7f7f37c6fdf672cd2ef2f91c27a7ca1daff128ac65a2";
const OYACC_SOURCES: [&str; OYACC_SOURCE_COMPILE_COUNT as usize] = [
    "closure", "error", "lalr", "lr0", "main", "mkpar", "output", "reader", "skeleton", "symtab", "verbose",
    "warshall", "portable",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OyaccExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const OYACC_EXPECTED_OUTPUTS: [OyaccExpectedOutput; OYACC_OUTPUT_COUNT] = [
    OyaccExpectedOutput {
        artifact_id: "oyacc-6.6",
        digest_blake3: OYACC_FINAL_BLAKE3,
    },
    OyaccExpectedOutput {
        artifact_id: "oyacc-smoke-output",
        digest_blake3: OYACC_SMOKE_OUTPUT_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OyaccSourceMaterializationReport {
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
pub(crate) struct OyaccOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OyaccInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub patch_command_count: u32,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<OyaccOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OyaccInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub patch_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexOyaccError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexOyaccError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "oyacc source record was not found"),
            Self::Materialization(message) => write!(formatter, "oyacc materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "oyacc runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexOyaccError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexOyaccError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexOyaccError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); OYACC_SOURCE_ARTIFACT_COUNT] {
    [
        (OYACC_SOURCE_ARTIFACT_ID, OYACC_SOURCE_CONTENT_BLAKE3),
        (OYACC_SMOKE_SOURCE_ARTIFACT_ID, OYACC_SMOKE_SOURCE_BLAKE3),
        (OYACC_MALFORMED_SOURCE_ARTIFACT_ID, OYACC_MALFORMED_SOURCE_BLAKE3),
        (OYACC_EMPTY_CONFIG_ARTIFACT_ID, OYACC_EMPTY_CONFIG_BLAKE3),
        (OYACC_MESLIBC_PATCH_ARTIFACT_ID, OYACC_MESLIBC_PATCH_BLAKE3),
        (OYACC_TCC_PATCH_ARTIFACT_ID, OYACC_TCC_PATCH_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_oyacc_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<OyaccSourceMaterializationReport, StagexOyaccError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexOyaccError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexOyaccError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexOyaccError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexOyaccError::Materialization(format!("creating oyacc source scratch: {error}")))?;
    let output_path = scratch_dir.join(OYACC_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexOyaccError::Materialization(format!("materializing oyacc source {}: {error}", record.identity))
    })?;
    if !output_path.join("main.c").is_file() || !output_path.join("reader.c").is_file() {
        return Err(StagexOyaccError::Materialization(
            "materialized oyacc tree lacks required source files".to_string(),
        ));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, OYACC_SOURCE_CONTENT_BLAKE3);
    Ok(OyaccSourceMaterializationReport {
        format: OYACC_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: OYACC_SOURCE_ARTIFACT_ID,
        record_name: OYACC_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: OYACC_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_oyacc_inventory(
    request: OyaccInventoryRequest<'_>,
) -> Result<OyaccInventoryReport, StagexOyaccError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexOyaccError::Materialization(format!("creating oyacc scratch: {error}")))?;
    let source_root = request.scratch_dir.join(OYACC_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), b"")?;
    apply_source_patches(&request, &source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexOyaccError::Materialization(format!("creating oyacc output: {error}")))?;
    let yacc = build_oyacc(&request, &source_root, &output_root)?;
    let smoke_output = run_oyacc_smokes(&request, &source_root, &yacc)?;
    let outputs = collect_outputs(&yacc, &smoke_output)?;
    validate_expected_outputs(&outputs)?;
    let report = OyaccInventoryReport {
        format: OYACC_REPORT_FORMAT,
        configured_source_digest_blake3,
        patch_command_count: OYACC_PATCH_COMMAND_COUNT,
        source_compile_count: OYACC_SOURCE_COMPILE_COUNT,
        build_command_count: OYACC_BUILD_COMMAND_COUNT,
        smoke_command_count: OYACC_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: OYACC_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("oyacc-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexOyaccError::Materialization(format!("serializing oyacc report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), OYACC_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &OyaccInventoryRequest<'_>) -> Result<(), StagexOyaccError> {
    if request.scratch_dir.exists() {
        return Err(StagexOyaccError::Materialization(format!(
            "create-new oyacc scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("oyacc source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
        ("GNU patch runtime", request.patch_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexOyaccError::Materialization(format!(
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
    validate_bound_bytes(OYACC_SMOKE_SOURCE, OYACC_SMOKE_SOURCE_BLAKE3, "oyacc smoke grammar")?;
    validate_bound_bytes(OYACC_MALFORMED_SOURCE, OYACC_MALFORMED_SOURCE_BLAKE3, "oyacc malformed grammar")?;
    validate_file_digest(
        &request.patch_root.join("bin/patch"),
        crate::stagex_gnu_patch::GNU_PATCH_FINAL_BLAKE3,
        "GNU patch 2.5.9",
    )?;
    validate_bound_bytes(OYACC_MESLIBC_PATCH, OYACC_MESLIBC_PATCH_BLAKE3, "oyacc Mes libc patch")?;
    validate_bound_bytes(OYACC_TCC_PATCH, OYACC_TCC_PATCH_BLAKE3, "oyacc TinyCC patch")?;
    assert!(request.source_root.join("main.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn apply_source_patches(request: &OyaccInventoryRequest<'_>, source_root: &Path) -> Result<(), StagexOyaccError> {
    let patch_dir = request.scratch_dir.join("patches");
    fs::create_dir(&patch_dir)
        .map_err(|error| StagexOyaccError::Materialization(format!("creating oyacc patch root: {error}")))?;
    let patch_executable = request.patch_root.join("bin/patch");
    for (index, (file_name, bytes)) in [
        ("oyacc-6.6-meslibc.patch", OYACC_MESLIBC_PATCH),
        ("oyacc-6.6-tcc.patch", OYACC_TCC_PATCH),
    ]
    .iter()
    .enumerate()
    {
        let patch_path = patch_dir.join(file_name);
        crate::stagex_mes_lib::write_create_new(&patch_path, bytes)?;
        crate::stagex_mes_lib::run_bounded_process(
            &patch_executable,
            &[
                "-Np1",
                "-i",
                crate::stagex_mes_lib::utf8_absolute(&patch_path, "oyacc patch")?,
            ],
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("oyacc-patch-{index:02}.stderr.txt")),
        )?;
    }
    assert_eq!(OYACC_PATCH_COMMAND_COUNT, 2);
    assert!(source_root.join("config.h").is_file());
    Ok(())
}

fn build_oyacc(
    request: &OyaccInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexOyaccError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    let mut objects = Vec::with_capacity(OYACC_SOURCES.len());
    for (index, source_name) in OYACC_SOURCES.iter().enumerate() {
        let object = format!("{source_name}.o");
        let mut args = compiler_prefix(request.tinycc27_root)?;
        args.extend([
            "-D__dead=".to_string(),
            "-D__unused=".to_string(),
            "-c".to_string(),
            format!("{source_name}.c"),
            "-o".to_string(),
            object.clone(),
        ]);
        crate::stagex_mes_lib::run_bounded_process(
            &compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("oyacc-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&source_root.join(&object), &object)?;
        objects.push(object);
    }
    let yacc = output_root.join("yacc");
    link_oyacc(request, &compiler, source_root, &yacc, &objects)?;
    crate::stagex_tinycc::set_owner_executable(&yacc)?;
    validate_nonempty_file(&yacc, "oyacc 6.6")?;
    assert_eq!(objects.len(), OYACC_SOURCES.len());
    assert!(yacc.is_file());
    Ok(yacc)
}

fn link_oyacc(
    request: &OyaccInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output: &Path,
    objects: &[String],
) -> Result<(), StagexOyaccError> {
    let libdir = request.tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let mut args = vec![
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC runtime")?.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "oyacc output")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.extend([
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libgetopt.a"), "TinyCC libgetopt")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("oyacc-link.stderr.txt"),
    )?;
    assert_eq!(objects.len(), OYACC_SOURCES.len());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(())
}

fn compiler_prefix(tinycc27_root: &Path) -> Result<Vec<String>, StagexOyaccError> {
    Ok(vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
    ])
}

fn run_oyacc_smokes(
    request: &OyaccInventoryRequest<'_>,
    source_root: &Path,
    yacc: &Path,
) -> Result<PathBuf, StagexOyaccError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexOyaccError::Materialization(format!("creating oyacc smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("positive.y"), OYACC_SMOKE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.y"), OYACC_MALFORMED_SOURCE)?;
    crate::stagex_mes_lib::run_bounded_process(
        yacc,
        &["positive.y"],
        &smoke_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("oyacc-positive.stderr.txt"),
    )?;
    let output = smoke_root.join("y.tab.c");
    validate_nonempty_file(&output, "oyacc generated parser")?;
    require_expected_process_failure(
        yacc,
        &["malformed.y"],
        &smoke_root,
        &request.scratch_dir.join("oyacc-negative.stderr.txt"),
    )?;
    assert!(output.is_file());
    assert_eq!(OYACC_SMOKE_COMMAND_COUNT, 2);
    Ok(output)
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexOyaccError> {
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
        Ok(()) => Err(StagexOyaccError::Materialization("oyacc accepted malformed grammar".to_string())),
        Err(error) => Err(StagexOyaccError::Runtime(error)),
    }
}

fn collect_outputs(yacc: &Path, smoke_output: &Path) -> Result<Vec<OyaccOutputReport>, StagexOyaccError> {
    let mut outputs = Vec::with_capacity(OYACC_OUTPUT_COUNT);
    for (artifact_id, path) in [("oyacc-6.6", yacc), ("oyacc-smoke-output", smoke_output)] {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexOyaccError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(OyaccOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), OYACC_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[OyaccOutputReport]) -> Result<(), StagexOyaccError> {
    let mut mismatches = Vec::new();
    for expected in OYACC_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexOyaccError::Materialization(format!("oyacc output {} is missing", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, observed.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexOyaccError::Materialization(format!(
            "oyacc output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), OYACC_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexOyaccError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(OYACC_RECORD_NAME))
        .ok_or(StagexOyaccError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexOyaccError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != OYACC_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexOyaccError::Materialization(format!(
            "oyacc source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexOyaccError::Materialization(
            "oyacc source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, OYACC_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-oyacc-configured-source-v1\0");
    hasher.update(OYACC_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(OYACC_SMOKE_SOURCE);
    hasher.update(OYACC_MALFORMED_SOURCE);
    hasher.update(OYACC_EMPTY_CONFIG_BLAKE3.as_bytes());
    hasher.update(OYACC_MESLIBC_PATCH);
    hasher.update(OYACC_TCC_PATCH);
    for source in OYACC_SOURCES {
        hasher.update(source.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(b"-D__dead=\0-D__unused=\0-static\0-nostdlib\0libgetopt.a\0");
    hasher.finalize().to_hex().to_string()
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexOyaccError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexOyaccError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexOyaccError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexOyaccError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexOyaccError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexOyaccError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > OYACC_FILE_BYTES_MAX {
        return Err(StagexOyaccError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    let bytes =
        fs::read(path).map_err(|error| StagexOyaccError::Materialization(format!("reading {label}: {error}")))?;
    assert_eq!(u64::try_from(bytes.len()).unwrap(), metadata.len());
    assert!(!bytes.is_empty());
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexOyaccError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexOyaccError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > OYACC_FILE_BYTES_MAX {
        return Err(StagexOyaccError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    assert!(path.is_file());
    assert!(metadata.len() <= OYACC_FILE_BYTES_MAX);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_OYACC_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_PATCH_ROOT_ENV: &str = "MANTLE_STAGE_X_PATCH_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_OYACC_BUILD_SCRATCH";

    #[test]
    fn configured_source_digest_is_stable() {
        let digest = configured_source_digest_blake3();
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert_eq!(digest, OYACC_CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn source_list_is_bounded_and_complete() {
        assert_eq!(OYACC_SOURCES.len(), usize::try_from(OYACC_SOURCE_COMPILE_COUNT).unwrap());
        assert!(OYACC_SOURCES.iter().all(|source| !source.is_empty()));
        assert!(OYACC_SOURCES.contains(&"reader"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_oyacc_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let expected_manifest = read_source_bundle(&bundle).unwrap().manifest_blake3;
        let report = materialize_authenticated_oyacc_source(&bundle, &expected_manifest, &scratch).unwrap();
        assert_eq!(report.record_content_blake3, OYACC_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("main.c").is_file());
    }

    #[test]
    #[ignore = "requires retained oyacc source and TinyCC 0.9.27"]
    fn derives_retained_oyacc_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tinycc27_root = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let patch_root = PathBuf::from(std::env::var(RETAINED_PATCH_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_oyacc_inventory(OyaccInventoryRequest {
            source_root: &source_root,
            tinycc27_root: &tinycc27_root,
            patch_root: &patch_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, OYACC_BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
