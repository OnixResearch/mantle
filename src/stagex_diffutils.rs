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
const DIFFUTILS_RECORD_NAME: &str = "diffutils-2.7-src";
const DIFFUTILS_SOURCE_OUTPUT_NAME: &str = "diffutils-2.7";
pub(crate) const DIFFUTILS_SOURCE_ARTIFACT_ID: &str = "diffutils-2.7-source";
pub(crate) const DIFFUTILS_SOURCE_CONTENT_BLAKE3: &str =
    "36714b7ee4e36f7f39ca3fb32010c5babe8a09532ee991f565771f9ca26daf90";
const DIFFUTILS_RECIPE: &[u8] = include_bytes!("../bootstrap/diffutils-2.7-musl.ncl");
pub(crate) const DIFFUTILS_RECIPE_ARTIFACT_ID: &str = "diffutils-2.7-recipe-source";
pub(crate) const DIFFUTILS_RECIPE_BLAKE3: &str = "51279c523c0443488a89650985fce68305233907ee2b915540978cb3271f1254";
const DIFFUTILS_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-diffutils-source-materialization-v1";
const DIFFUTILS_SOURCE_NON_CLAIM: &str =
    "diffutils source materialization proves authenticated offline archive identity and fixed-output parity only";
const DIFFUTILS_REPORT_FORMAT: &str = "mantle-stagex-diffutils-2.7-inventory-v1";
const DIFFUTILS_NON_CLAIM: &str = "this inventory binds GNU diffutils 2.7 and bounded equality, difference, version, and malformed-input observations only; it does not prove later parser generators, binutils, native TinyCC, or provider admission";
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const DIFFUTILS_FILE_MEBIBYTES_MAX: u64 = 64;
const DIFFUTILS_FILE_BYTES_MAX: u64 = DIFFUTILS_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const DIFFUTILS_CMP_SOURCE_COUNT: usize = 7;
const DIFFUTILS_DIFF_SOURCE_COUNT: usize = 17;
const DIFFUTILS_SOURCE_COMPILE_COUNT_USIZE: usize = 24;
const DIFFUTILS_SOURCE_COMPILE_COUNT: u32 = 24;
const DIFFUTILS_BUILD_COMMAND_COUNT: u32 = 26;
const DIFFUTILS_SMOKE_COMMAND_COUNT: u32 = 6;
const DIFFUTILS_OUTPUT_COUNT: usize = 6;
const DIFFUTILS_SOURCE_ARTIFACT_COUNT: usize = 2;
const DIFFUTILS_COMPILE_ARG_CAPACITY: usize = 16;
const DIFFUTILS_OBJECT_MODE: u32 = 0o644;
const DIFFUTILS_OUTPUT_MODE: u32 = 0o755;
const DIFFUTILS_EXECUTE_MODE_MASK: u32 = 0o111;
const DIFFUTILS_DIFFERENCE_EXIT_CODE: i32 = 1;
const DIFFUTILS_MALFORMED_EXIT_CODE: i32 = 2;
const DIFFUTILS_CONFIG_H: &[u8] = b"";
const DIFFUTILS_EQUAL_BYTES: &[u8] = b"same\n";
const DIFFUTILS_CHANGED_BYTES: &[u8] = b"changed\n";
const DIFFUTILS_REJECTION_RECEIPT: &[u8] =
    b"{\"format\":\"mantle-stagex-diffutils-rejection-v1\",\"malformed_input_rejected\":true}\n";
const DIFFUTILS_COMMON_FLAGS: [&str; 10] = [
    "-I.",
    "-DNULL_DEVICE=\"/dev/null\"",
    "-DHAVE_STRERROR=1",
    "-DREGEX_MALLOC=1",
    "-DHAVE_DIRENT_H=1",
    "-DHAVE_DUP2=1",
    "-DHAVE_FORK=1",
    "-DHAVE_UNISTD_H=1",
    "-DHAVE_STRING_H=1",
    "-DHAVE_STDLIB_H=1",
];
const DIFFUTILS_CMP_SOURCES: [&str; DIFFUTILS_CMP_SOURCE_COUNT] =
    ["cmp", "cmpbuf", "error", "getopt", "getopt1", "xmalloc", "version"];
const DIFFUTILS_DIFF_SOURCES: [&str; DIFFUTILS_DIFF_SOURCE_COUNT] = [
    "diff", "alloca", "analyze", "cmpbuf", "dir", "io", "util", "context", "ed", "ifdef", "normal", "side", "fnmatch",
    "getopt", "getopt1", "regex", "version",
];
pub(crate) const DIFFUTILS_CONFIGURED_SOURCE_BLAKE3: &str =
    "00d79a6f09d23d7a3430ccfd407a8cac0351aa5ecc6efc122f8cf093989cbbe5";
pub(crate) const DIFFUTILS_DIFF_BLAKE3: &str = "054ba2636aef71a1cbda93de3358ff8b8a7c92127ce951dcb7cc5814ac0e2d66";
pub(crate) const DIFFUTILS_CMP_BLAKE3: &str = "145c05e3557ba74a8ce85912c08feee01b7616fcc273e92279a96dbe18a8793b";
const DIFFUTILS_VERSION_BLAKE3: &str = "33510988f98a66cf0bc0e53661c6af77e8ed062faa45eac70734aef40a65ce6c";
const DIFFUTILS_DIFF_OBSERVATION_BLAKE3: &str = "db3c58f16a007600d256c0be6306d55101919037d37ed4ece231bab4c3bbe2ca";
const DIFFUTILS_CMP_OBSERVATION_BLAKE3: &str = "5d5856604bb2fa260c00b5ed8bcaa9a6b0bb89a644c7b23a2b678bf0e9127f6e";
const DIFFUTILS_REJECTION_BLAKE3: &str = "202fa55131223d7bc743f2ff8d6896e001df815d001f698d58be1bc451ea5725";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffutilsExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const DIFFUTILS_EXPECTED_OUTPUTS: [DiffutilsExpectedOutput; DIFFUTILS_OUTPUT_COUNT] = [
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-diff-2.7",
        digest_blake3: DIFFUTILS_DIFF_BLAKE3,
    },
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-cmp-2.7",
        digest_blake3: DIFFUTILS_CMP_BLAKE3,
    },
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-version-observation",
        digest_blake3: DIFFUTILS_VERSION_BLAKE3,
    },
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-diff-observation",
        digest_blake3: DIFFUTILS_DIFF_OBSERVATION_BLAKE3,
    },
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-cmp-observation",
        digest_blake3: DIFFUTILS_CMP_OBSERVATION_BLAKE3,
    },
    DiffutilsExpectedOutput {
        artifact_id: "diffutils-malformed-rejection",
        digest_blake3: DIFFUTILS_REJECTION_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DiffutilsSourceMaterializationReport {
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
pub(crate) struct DiffutilsOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DiffutilsInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<DiffutilsOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DiffutilsInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexDiffutilsError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexDiffutilsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU diffutils source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU diffutils materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU diffutils runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexDiffutilsError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexDiffutilsError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexDiffutilsError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); DIFFUTILS_SOURCE_ARTIFACT_COUNT] {
    [
        (DIFFUTILS_SOURCE_ARTIFACT_ID, DIFFUTILS_SOURCE_CONTENT_BLAKE3),
        (DIFFUTILS_RECIPE_ARTIFACT_ID, DIFFUTILS_RECIPE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_diffutils_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<DiffutilsSourceMaterializationReport, StagexDiffutilsError> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexDiffutilsError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir).map_err(|error| {
        StagexDiffutilsError::Materialization(format!("creating diffutils source scratch: {error}"))
    })?;
    let output_path = scratch_dir.join(DIFFUTILS_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexDiffutilsError::Materialization(format!("materializing diffutils source {}: {error}", record.identity))
    })?;
    validate_materialized_source(&output_path)?;
    let source_materialization = DiffutilsSourceMaterializationReport {
        format: DIFFUTILS_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: DIFFUTILS_SOURCE_ARTIFACT_ID,
        record_name: DIFFUTILS_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: DIFFUTILS_SOURCE_NON_CLAIM,
    };
    assert_eq!(source_materialization.record_content_blake3, DIFFUTILS_SOURCE_CONTENT_BLAKE3);
    assert!(source_materialization.output_path.join("diff.c").is_file());
    Ok(source_materialization)
}

pub(crate) fn derive_diffutils_inventory(
    request: DiffutilsInventoryRequest<'_>,
) -> Result<DiffutilsInventoryReport, StagexDiffutilsError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("creating diffutils scratch: {error}")))?;
    let source_root = request.scratch_dir.join(DIFFUTILS_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    fs::write(source_root.join("config.h"), DIFFUTILS_CONFIG_H)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("writing diffutils config: {error}")))?;
    validate_compile_source_set(&source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("creating diffutils output: {error}")))?;
    let binaries = build_diffutils(&request, &source_root, &output_root)?;
    let observations = run_diffutils_smokes(&request, &source_root, &binaries)?;
    let outputs = collect_outputs(&binaries, &observations)?;
    validate_expected_outputs(&outputs)?;
    let inventory = DiffutilsInventoryReport {
        format: DIFFUTILS_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: DIFFUTILS_SOURCE_COMPILE_COUNT,
        build_command_count: DIFFUTILS_BUILD_COMMAND_COUNT,
        smoke_command_count: DIFFUTILS_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: DIFFUTILS_NON_CLAIM,
    };
    write_inventory(request.scratch_dir, &inventory)?;
    assert_eq!(inventory.outputs.len(), DIFFUTILS_OUTPUT_COUNT);
    assert!(inventory.fallback_events.is_empty());
    Ok(inventory)
}

fn write_inventory(scratch_dir: &Path, report: &DiffutilsInventoryReport) -> Result<(), StagexDiffutilsError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("serializing diffutils report: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&scratch_dir.join("diffutils-inventory.json"), &bytes)?;
    assert!(!bytes.is_empty());
    assert!(scratch_dir.join("diffutils-inventory.json").is_file());
    Ok(())
}

fn validate_bundle_path(bundle_path: &Path) -> Result<(), StagexDiffutilsError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexDiffutilsError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    assert!(bundle_path.is_absolute());
    assert!(bundle_path.is_file());
    Ok(())
}

fn validate_materialized_source(output_path: &Path) -> Result<(), StagexDiffutilsError> {
    for relative in ["cmp.c", "diff.c", "regex.c"] {
        if !output_path.join(relative).is_file() {
            return Err(StagexDiffutilsError::Materialization(format!(
                "materialized diffutils tree lacks required source {relative}"
            )));
        }
    }
    assert!(output_path.is_absolute());
    assert!(output_path.join("diff.c").is_file());
    Ok(())
}

fn validate_inventory_inputs(request: &DiffutilsInventoryRequest<'_>) -> Result<(), StagexDiffutilsError> {
    if request.scratch_dir.exists() {
        return Err(StagexDiffutilsError::Materialization(format!(
            "create-new diffutils scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU diffutils source", request.source_root),
        ("TinyCC musl-v2", request.tcc_musl_v2_root),
        ("native musl", request.musl_native_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexDiffutilsError::Materialization(format!(
                "{label} root is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_recipe_digest()?;
    validate_file_digest(
        &request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2"),
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "TinyCC musl-v2 compiler",
    )?;
    validate_native_musl_inputs(request.musl_native_root)?;
    assert!(!request.scratch_dir.exists());
    assert!(request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2").is_file());
    Ok(())
}

fn validate_native_musl_inputs(root: &Path) -> Result<(), StagexDiffutilsError> {
    for (artifact_id, relative) in [("musl-native-crt1", "lib/crt1.o"), ("musl-native-libc", "lib/libc.a")] {
        let expected = crate::stagex_musl_native::EXPECTED_OUTPUTS
            .iter()
            .find(|output| output.artifact_id == artifact_id)
            .ok_or_else(|| {
                StagexDiffutilsError::Materialization(format!("native musl inventory lacks required {artifact_id}"))
            })?
            .digest_blake3;
        validate_file_digest(&root.join(relative), expected, artifact_id)?;
    }
    assert!(root.join("include").is_dir());
    assert!(root.join("lib/libc.a").is_file());
    Ok(())
}

fn validate_recipe_digest() -> Result<(), StagexDiffutilsError> {
    let observed = blake3::hash(DIFFUTILS_RECIPE).to_hex().to_string();
    if observed != DIFFUTILS_RECIPE_BLAKE3 {
        return Err(StagexDiffutilsError::Materialization(format!(
            "diffutils recipe BLAKE3 mismatch: expected {DIFFUTILS_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!DIFFUTILS_RECIPE.is_empty());
    Ok(())
}

fn validate_compile_source_set(source_root: &Path) -> Result<(), StagexDiffutilsError> {
    let paths = compile_source_paths();
    if paths.len() != DIFFUTILS_SOURCE_COMPILE_COUNT_USIZE {
        return Err(StagexDiffutilsError::Materialization(
            "diffutils compile source set has a substituted count".to_string(),
        ));
    }
    for (_, relative) in &paths {
        if !source_root.join(relative).is_file() {
            return Err(StagexDiffutilsError::Materialization(format!(
                "diffutils compile source is missing: {relative}"
            )));
        }
    }
    assert_eq!(paths.len(), DIFFUTILS_SOURCE_COMPILE_COUNT_USIZE);
    assert!(paths.iter().all(|(_, path)| path.ends_with(".c")));
    Ok(())
}

fn compile_source_paths() -> Vec<(&'static str, String)> {
    let mut paths = Vec::with_capacity(DIFFUTILS_SOURCE_COMPILE_COUNT_USIZE);
    paths.extend(DIFFUTILS_CMP_SOURCES.iter().map(|name| ("cmp", format!("{name}.c"))));
    paths.extend(DIFFUTILS_DIFF_SOURCES.iter().map(|name| ("diff", format!("{name}.c"))));
    assert_eq!(paths.len(), DIFFUTILS_SOURCE_COMPILE_COUNT_USIZE);
    assert!(!paths.is_empty());
    paths
}

#[derive(Debug, Clone)]
struct DiffutilsBinaries {
    diff: PathBuf,
    cmp: PathBuf,
}

fn build_diffutils(
    request: &DiffutilsInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<DiffutilsBinaries, StagexDiffutilsError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object_root = source_root.join("stagex-objects");
    fs::create_dir(&object_root)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("creating diffutils object root: {error}")))?;
    let cmp_objects =
        compile_target_sources(request, source_root, &object_root, &compiler, "cmp", &DIFFUTILS_CMP_SOURCES)?;
    let diff_objects =
        compile_target_sources(request, source_root, &object_root, &compiler, "diff", &DIFFUTILS_DIFF_SOURCES)?;
    let cmp = link_target(request, source_root, output_root, &compiler, "cmp", &cmp_objects)?;
    let diff = link_target(request, source_root, output_root, &compiler, "diff", &diff_objects)?;
    assert!(cmp.is_file());
    assert!(diff.is_file());
    Ok(DiffutilsBinaries { diff, cmp })
}

fn compile_target_sources<const SOURCE_COUNT: usize>(
    request: &DiffutilsInventoryRequest<'_>,
    source_root: &Path,
    object_root: &Path,
    compiler: &Path,
    target: &str,
    sources: &[&str; SOURCE_COUNT],
) -> Result<Vec<String>, StagexDiffutilsError> {
    let mut objects = Vec::with_capacity(SOURCE_COUNT);
    for (index, source_name) in sources.iter().enumerate() {
        let source = format!("{source_name}.c");
        let object_relative = format!("stagex-objects/{target}-{source_name}.o");
        let object = object_root.join(format!("{target}-{source_name}.o"));
        let args = compile_args(request.musl_native_root, &source, &object_relative)?;
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("diffutils-{target}-compile-{index:02}.stderr.txt")),
        )?;
        fs::set_permissions(&object, fs::Permissions::from_mode(DIFFUTILS_OBJECT_MODE)).map_err(|error| {
            StagexDiffutilsError::Materialization(format!(
                "setting diffutils {target} object {source_name} permissions: {error}"
            ))
        })?;
        validate_nonempty_file(&object, &format!("diffutils {target} object {source_name}"))?;
        objects.push(object_relative);
    }
    assert_eq!(objects.len(), SOURCE_COUNT);
    assert!(objects.iter().all(|path| path.ends_with(".o")));
    Ok(objects)
}

fn compile_args(musl_native_root: &Path, source: &str, object: &str) -> Result<Vec<String>, StagexDiffutilsError> {
    let mut args = Vec::with_capacity(DIFFUTILS_COMPILE_ARG_CAPACITY);
    args.push("-c".to_string());
    args.extend(DIFFUTILS_COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.push(format!("-I{}", utf8_path(&musl_native_root.join("include"), "native musl include")?));
    args.push(source.to_string());
    args.push("-o".to_string());
    args.push(object.to_string());
    assert!(args.iter().any(|arg| arg == "-c"));
    assert!(args.iter().any(|arg| arg == "-o"));
    Ok(args)
}

fn link_target(
    request: &DiffutilsInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
    compiler: &Path,
    target: &str,
    objects: &[String],
) -> Result<PathBuf, StagexDiffutilsError> {
    let output = output_root.join(target);
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_path(&output, "diffutils output")?.to_string(),
        utf8_path(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(utf8_path(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("diffutils-{target}-link.stderr.txt")),
    )?;
    validate_nonempty_file(&output, &format!("GNU {target} 2.7"))?;
    fs::set_permissions(&output, fs::Permissions::from_mode(DIFFUTILS_OUTPUT_MODE))
        .map_err(|error| StagexDiffutilsError::Materialization(format!("setting {target} permissions: {error}")))?;
    assert!(output.is_file());
    assert!(
        output
            .metadata()
            .map(|metadata| metadata.permissions().mode() & DIFFUTILS_EXECUTE_MODE_MASK != 0)
            .unwrap_or(false)
    );
    Ok(output)
}

#[derive(Debug, Clone)]
struct DiffutilsObservations {
    version: PathBuf,
    diff_changed: PathBuf,
    cmp_changed: PathBuf,
    rejection: PathBuf,
}

fn run_diffutils_smokes(
    request: &DiffutilsInventoryRequest<'_>,
    source_root: &Path,
    binaries: &DiffutilsBinaries,
) -> Result<DiffutilsObservations, StagexDiffutilsError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("creating diffutils smoke root: {error}")))?;
    let first = smoke_root.join("first.txt");
    let second = smoke_root.join("second.txt");
    crate::stagex_mes_lib::write_create_new(&first, DIFFUTILS_EQUAL_BYTES)?;
    crate::stagex_mes_lib::write_create_new(&second, DIFFUTILS_EQUAL_BYTES)?;
    run_success(request, &binaries.diff, &["first.txt", "second.txt"], &smoke_root, "diff-equal")?;
    run_success(request, &binaries.cmp, &["first.txt", "second.txt"], &smoke_root, "cmp-equal")?;
    fs::write(&second, DIFFUTILS_CHANGED_BYTES)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("writing changed diffutils input: {error}")))?;
    let diff_changed = smoke_root.join("diff-changed.out");
    let diff_result =
        run_capture(request, &binaries.diff, &["first.txt", "second.txt"], &smoke_root, &diff_changed, "diff-changed");
    require_difference_process(diff_result)?;
    let cmp_changed = smoke_root.join("cmp-changed.out");
    let cmp_result =
        run_capture(request, &binaries.cmp, &["first.txt", "second.txt"], &smoke_root, &cmp_changed, "cmp-changed");
    require_difference_process(cmp_result)?;
    let version = smoke_root.join("version.txt");
    run_capture(request, &binaries.diff, &["--version"], &smoke_root, &version, "version")?;
    validate_version(&version)?;
    let malformed_stdout = smoke_root.join("malformed.out");
    let malformed_result =
        run_capture(request, &binaries.diff, &["--mantle-invalid-option"], &smoke_root, &malformed_stdout, "malformed");
    require_rejected_process(malformed_result)?;
    require_exact_bytes(&malformed_stdout, b"", "diffutils malformed stdout")?;
    let rejection = smoke_root.join("malformed-rejection.json");
    crate::stagex_mes_lib::write_create_new(&rejection, DIFFUTILS_REJECTION_RECEIPT)?;
    assert!(diff_changed.is_file());
    assert!(cmp_changed.is_file());
    Ok(DiffutilsObservations {
        version,
        diff_changed,
        cmp_changed,
        rejection,
    })
}

fn run_success<S: AsRef<std::ffi::OsStr>>(
    request: &DiffutilsInventoryRequest<'_>,
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    label: &str,
) -> Result<(), StagexDiffutilsError> {
    crate::stagex_mes_lib::run_bounded_process(
        executable,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("diffutils-{label}.stderr.txt")),
    )?;
    assert!(executable.is_file());
    assert!(current_dir.is_dir());
    Ok(())
}

fn run_capture<S: AsRef<std::ffi::OsStr>>(
    request: &DiffutilsInventoryRequest<'_>,
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    stdout: &Path,
    label: &str,
) -> Result<(), StagexDiffutilsError> {
    crate::stagex_mes_lib::run_bounded_process_capturing_stdout(
        executable,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        stdout,
        DIFFUTILS_FILE_BYTES_MAX,
        &request.scratch_dir.join(format!("diffutils-{label}.stderr.txt")),
    )?;
    assert!(executable.is_file());
    assert!(stdout.is_file());
    Ok(())
}

fn require_difference_process(result: Result<(), StagexDiffutilsError>) -> Result<(), StagexDiffutilsError> {
    match result {
        Err(StagexDiffutilsError::Runtime(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(DIFFUTILS_DIFFERENCE_EXIT_CODE),
            ..
        })) => Ok(()),
        Err(error) => Err(StagexDiffutilsError::Materialization(format!(
            "diffutils difference check failed with unexpected result: {error}"
        ))),
        Ok(()) => {
            Err(StagexDiffutilsError::Materialization("diffutils accepted different inputs as equal".to_string()))
        }
    }
}

fn require_rejected_process(result: Result<(), StagexDiffutilsError>) -> Result<(), StagexDiffutilsError> {
    match result {
        Err(StagexDiffutilsError::Runtime(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(DIFFUTILS_MALFORMED_EXIT_CODE),
            stderr,
            ..
        })) if !stderr.is_empty() => Ok(()),
        Err(error) => Err(StagexDiffutilsError::Materialization(format!(
            "diffutils malformed-input check failed with unexpected result: {error}"
        ))),
        Ok(()) => Err(StagexDiffutilsError::Materialization("diffutils accepted malformed input".to_string())),
    }
}

fn validate_version(path: &Path) -> Result<(), StagexDiffutilsError> {
    let bytes = read_bounded_file(path, "diffutils version")?;
    let has_name = bytes.windows(b"GNU diffutils".len()).any(|window| window == b"GNU diffutils");
    let has_version = bytes.windows(b"2.7".len()).any(|window| window == b"2.7");
    if !has_name || !has_version {
        return Err(StagexDiffutilsError::Materialization(
            "diffutils version observation lacks GNU diffutils 2.7".to_string(),
        ));
    }
    assert!(has_name);
    assert!(has_version);
    Ok(())
}

fn collect_outputs(
    binaries: &DiffutilsBinaries,
    observations: &DiffutilsObservations,
) -> Result<Vec<DiffutilsOutputReport>, StagexDiffutilsError> {
    let paths = [
        ("diffutils-diff-2.7", binaries.diff.as_path()),
        ("diffutils-cmp-2.7", binaries.cmp.as_path()),
        ("diffutils-version-observation", observations.version.as_path()),
        ("diffutils-diff-observation", observations.diff_changed.as_path()),
        ("diffutils-cmp-observation", observations.cmp_changed.as_path()),
        ("diffutils-malformed-rejection", observations.rejection.as_path()),
    ];
    let outputs = paths
        .iter()
        .map(|(artifact_id, path)| output_report(artifact_id, path))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), DIFFUTILS_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[DiffutilsOutputReport]) -> Result<(), StagexDiffutilsError> {
    if outputs.len() != DIFFUTILS_EXPECTED_OUTPUTS.len() {
        return Err(StagexDiffutilsError::Materialization(format!(
            "expected {} diffutils outputs, observed {}",
            DIFFUTILS_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    let observed = outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), output.digest_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mismatches = DIFFUTILS_EXPECTED_OUTPUTS
        .iter()
        .filter_map(|expected| {
            let actual = observed.get(expected.artifact_id).copied().unwrap_or("missing");
            (actual != expected.digest_blake3).then(|| format!("{}={actual}", expected.artifact_id))
        })
        .collect::<Vec<_>>();
    if !mismatches.is_empty() {
        return Err(StagexDiffutilsError::Materialization(format!(
            "diffutils output BLAKE3 mismatches: {}",
            mismatches.join(", ")
        )));
    }
    assert_eq!(outputs.len(), DIFFUTILS_EXPECTED_OUTPUTS.len());
    assert!(mismatches.is_empty());
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexDiffutilsError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(DIFFUTILS_RECORD_NAME))
        .ok_or(StagexDiffutilsError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexDiffutilsError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != DIFFUTILS_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexDiffutilsError::Materialization(format!(
            "diffutils source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexDiffutilsError::Materialization(
            "diffutils source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, DIFFUTILS_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

pub(crate) fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-diffutils-configured-source-v1\0");
    hasher.update(DIFFUTILS_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(DIFFUTILS_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(DIFFUTILS_CONFIG_H);
    for value in DIFFUTILS_COMMON_FLAGS
        .iter()
        .chain(DIFFUTILS_CMP_SOURCES.iter())
        .chain(DIFFUTILS_DIFF_SOURCES.iter())
    {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    digest
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexDiffutilsError> {
    let observed = read_bounded_file(path, label)?;
    if observed != expected {
        return Err(StagexDiffutilsError::Materialization(format!("{label} bytes differ")));
    }
    assert_eq!(observed, expected);
    assert!(path.is_file());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexDiffutilsError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexDiffutilsError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > DIFFUTILS_FILE_BYTES_MAX {
        return Err(StagexDiffutilsError::Materialization(format!(
            "{label} is empty, not regular, or exceeds {DIFFUTILS_FILE_BYTES_MAX} bytes"
        )));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexDiffutilsError> {
    let bytes = read_bounded_file(path, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexDiffutilsError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(path.is_file());
    Ok(())
}

fn output_report(artifact_id: &str, path: &Path) -> Result<DiffutilsOutputReport, StagexDiffutilsError> {
    let bytes = read_bounded_file(path, artifact_id)?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexDiffutilsError::Materialization(format!("{artifact_id} byte count exceeds u64")))?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert!(bytes_len > 0);
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    Ok(DiffutilsOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn read_bounded_file(path: &Path, label: &str) -> Result<Vec<u8>, StagexDiffutilsError> {
    let bytes =
        fs::read(path).map_err(|error| StagexDiffutilsError::Materialization(format!("reading {label}: {error}")))?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexDiffutilsError::Materialization(format!("{label} byte count exceeds u64")))?;
    if bytes_len > DIFFUTILS_FILE_BYTES_MAX {
        return Err(StagexDiffutilsError::Materialization(format!("{label} exceeds {DIFFUTILS_FILE_BYTES_MAX} bytes")));
    }
    assert!(bytes_len <= DIFFUTILS_FILE_BYTES_MAX);
    assert!(path.is_file());
    Ok(bytes)
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexDiffutilsError> {
    if !path.is_absolute() {
        return Err(StagexDiffutilsError::Materialization(format!("{label} path is not absolute")));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexDiffutilsError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::source_bundle::SourceFileEntry;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_DIFFUTILS_SOURCE_ROOT";
    const RETAINED_TCC_V2_ROOT_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_V2_ROOT";
    const RETAINED_MUSL_NATIVE_ROOT_ENV: &str = "MANTLE_STAGE_X_MUSL_NATIVE_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_DIFFUTILS_BUILD_SCRATCH";

    fn source_record(kind: SourceRecordKind, content_blake3: &str) -> SourceRecord {
        SourceRecord {
            kind,
            identity: "test-diffutils".to_string(),
            store_prefix: None,
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), DIFFUTILS_RECORD_NAME.to_string())]),
            content_blake3: content_blake3.to_string(),
            payload_bytes: 1,
            files: vec![SourceFileEntry {
                path: "archive".to_string(),
                file_type: SourceFileType::Regular,
                executable: false,
                size: 1,
                content_hex: Some("78".to_string()),
                symlink_target: None,
                chunk_index: None,
                chunk_count: None,
                blake3: blake3::hash(b"x").to_hex().to_string(),
            }],
        }
    }

    #[test]
    fn validates_exact_diffutils_source_record_and_recipe() {
        let record = source_record(SourceRecordKind::FixedUrl, DIFFUTILS_SOURCE_CONTENT_BLAKE3);
        validate_source_record(&record).unwrap();
        validate_recipe_digest().unwrap();
        assert_eq!(record.kind, SourceRecordKind::FixedUrl);
        assert_eq!(source_artifact_digests().len(), DIFFUTILS_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_diffutils_source_record() {
        let wrong_kind = source_record(SourceRecordKind::VcsSnapshot, DIFFUTILS_SOURCE_CONTENT_BLAKE3);
        let wrong_digest = source_record(SourceRecordKind::FixedUrl, &"a".repeat(BLAKE3_HEX_CHAR_COUNT));
        assert!(validate_source_record(&wrong_kind).is_err());
        assert!(validate_source_record(&wrong_digest).is_err());
    }

    #[test]
    fn compile_source_multiset_is_complete() {
        let paths = compile_source_paths();
        let cmp_count = paths.iter().filter(|(target, _)| *target == "cmp").count();
        let diff_count = paths.iter().filter(|(target, _)| *target == "diff").count();
        let unique = paths.iter().collect::<BTreeSet<_>>();
        assert_eq!(cmp_count, DIFFUTILS_CMP_SOURCE_COUNT);
        assert_eq!(diff_count, DIFFUTILS_DIFF_SOURCE_COUNT);
        assert_eq!(unique.len(), paths.len());
    }

    #[test]
    fn compile_source_presence_is_required() {
        let source_root = tempfile::tempdir().unwrap();
        let unique_sources = compile_source_paths().into_iter().map(|(_, relative)| relative).collect::<BTreeSet<_>>();
        for relative in &unique_sources {
            fs::write(source_root.path().join(relative), []).unwrap();
        }
        validate_compile_source_set(source_root.path()).unwrap();
        let missing_relative = unique_sources.first().unwrap();
        fs::remove_file(source_root.path().join(missing_relative)).unwrap();
        let error = validate_compile_source_set(source_root.path()).unwrap_err();
        assert!(error.to_string().contains("compile source is missing"));
        assert!(error.to_string().contains(missing_relative));
    }

    #[test]
    fn configured_source_digest_is_bound() {
        let observed = configured_source_digest_blake3();
        assert_eq!(observed, DIFFUTILS_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    }

    #[test]
    fn rejects_substituted_output_digest() {
        let mut outputs = DIFFUTILS_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| DiffutilsOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_expected_outputs(&outputs).unwrap();
        outputs[0].digest_blake3 = "f".repeat(BLAKE3_HEX_CHAR_COUNT);
        let error = validate_expected_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("diffutils-diff-2.7"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn difference_and_rejection_classification_is_exact() {
        let difference = StagexDiffutilsError::Runtime(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            executable: PathBuf::from("/stagex/diff"),
            exit_code: Some(DIFFUTILS_DIFFERENCE_EXIT_CODE),
            stderr: String::new(),
        });
        let rejection = StagexDiffutilsError::Runtime(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            executable: PathBuf::from("/stagex/diff"),
            exit_code: Some(DIFFUTILS_MALFORMED_EXIT_CODE),
            stderr: "rejected".to_string(),
        });
        assert!(require_difference_process(Err(difference)).is_ok());
        assert!(require_rejected_process(Err(rejection)).is_ok());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_diffutils_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let report = materialize_authenticated_diffutils_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, DIFFUTILS_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("diff.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU diffutils source, TinyCC musl-v2, and native musl"]
    fn derives_retained_diffutils_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tcc_musl_v2_root = PathBuf::from(std::env::var(RETAINED_TCC_V2_ROOT_ENV).unwrap());
        let musl_native_root = PathBuf::from(std::env::var(RETAINED_MUSL_NATIVE_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_diffutils_inventory(DiffutilsInventoryRequest {
            source_root: &source_root,
            tcc_musl_v2_root: &tcc_musl_v2_root,
            musl_native_root: &musl_native_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.source_compile_count, DIFFUTILS_SOURCE_COMPILE_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
