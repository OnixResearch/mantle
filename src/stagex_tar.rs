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
const TAR_RECORD_NAME: &str = "tar-1.12-src";
const TAR_SOURCE_OUTPUT_NAME: &str = "tar-1.12";
pub(crate) const TAR_SOURCE_ARTIFACT_ID: &str = "tar-1.12-source";
pub(crate) const TAR_SOURCE_CONTENT_BLAKE3: &str = "0cbc269e6eccf79f32b9f64204a0277a0e6df877dc1a6d0fd0d3e240365abdff";
pub(crate) const TAR_GETDATE_SOURCE_ARTIFACT_ID: &str = "tar-getdate-stub-source";
pub(crate) const TAR_GETDATE_SOURCE_BLAKE3: &str = "a479349008d81a7766d3e56dd5bd1d83e8645c0690228d4bb2d89d6f04d4f0a9";
const TAR_GETDATE_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/tar-getdate-stub.c");
const TAR_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-tar-source-materialization-v1";
const TAR_REPORT_FORMAT: &str = "mantle-stagex-tar-1.12-inventory-v1";
const TAR_SOURCE_NON_CLAIM: &str =
    "GNU tar source materialization proves authenticated offline archive identity and fixed-output parity only";
const TAR_NON_CLAIM: &str = "this inventory binds GNU tar 1.12 and positive and negative archive smoke observations only; it does not prove later bootstrap utilities or provider admission";
const TAR_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TAR_LIB_COMPILE_COUNT: u32 = 14;
const TAR_SRC_COMPILE_COUNT: u32 = 15;
const TAR_SOURCE_COMPILE_COUNT: u32 = TAR_LIB_COMPILE_COUNT + TAR_SRC_COMPILE_COUNT;
const TAR_BUILD_COMMAND_COUNT: u32 = TAR_SOURCE_COMPILE_COUNT + 1;
const TAR_SMOKE_COMMAND_COUNT: u32 = 4;
const TAR_OUTPUT_COUNT: usize = 2;
const TAR_SOURCE_ARTIFACT_COUNT: usize = 2;
const TAR_SMOKE_INPUT: &[u8] = b"tar112-ok\n";
const TAR_MALFORMED_INPUT: &[u8] = b"not a tar archive\n";
pub(crate) const TAR_FINAL_BLAKE3: &str = "57c861b961bfc2c1047ab7f175c0f16024ea91787ba5a5e43600c721b5eff6ca";
const TAR_SMOKE_BLAKE3: &str = "59392bf4bef63f5688ff8604868a22adcecfe94f75c1234fe7c9df1107038be2";
const TAR_CONFIG_H: &[u8] = b"#define VERSION \"1.12\"\n#define PACKAGE \"tar\"\n";
const TAR_COMMON_FLAGS: [&str; 10] = [
    "-DHAVE_CONFIG_H",
    "-DSTDC_HEADERS",
    "-DHAVE_STRING_H",
    "-DHAVE_STDLIB_H",
    "-DHAVE_UNISTD_H",
    "-DHAVE_FCNTL_H",
    "-DHAVE_DIRENT_H",
    "-DHAVE_GETCWD",
    "-DSIZEOF_UNSIGNED_LONG=4",
    "-I.",
];
const TAR_LIB_SOURCES: [&str; TAR_LIB_COMPILE_COUNT as usize] = [
    "argmatch",
    "backupfile",
    "error",
    "fnmatch",
    "ftruncate",
    "getdate_stub",
    "getopt",
    "getopt1",
    "getversion",
    "modechange",
    "msleep",
    "xgetcwd",
    "xmalloc",
    "xstrdup",
];
const TAR_SRC_SOURCES: [&str; TAR_SRC_COMPILE_COUNT as usize] = [
    "arith", "buffer", "compare", "create", "delete", "extract", "incremen", "list", "mangle", "misc", "names",
    "open3", "rtapelib", "tar", "update",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TarExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const TAR_EXPECTED_OUTPUTS: [TarExpectedOutput; TAR_OUTPUT_COUNT] = [
    TarExpectedOutput {
        artifact_id: "tar-1.12",
        digest_blake3: TAR_FINAL_BLAKE3,
    },
    TarExpectedOutput {
        artifact_id: "tar-smoke",
        digest_blake3: TAR_SMOKE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TarSourceMaterializationReport {
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
pub(crate) struct TarOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TarInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<TarOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TarInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexTarError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexTarError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU tar source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU tar materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU tar runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexTarError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexTarError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexTarError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); TAR_SOURCE_ARTIFACT_COUNT] {
    [
        (TAR_SOURCE_ARTIFACT_ID, TAR_SOURCE_CONTENT_BLAKE3),
        (TAR_GETDATE_SOURCE_ARTIFACT_ID, TAR_GETDATE_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_tar_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<TarSourceMaterializationReport, StagexTarError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexTarError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexTarError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexTarError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexTarError::Materialization(format!("creating GNU tar source scratch: {error}")))?;
    let output_path = scratch_dir.join(TAR_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexTarError::Materialization(format!("materializing GNU tar source {}: {error}", record.identity))
    })?;
    if !output_path.join("src/tar.c").is_file() || !output_path.join("lib/xmalloc.c").is_file() {
        return Err(StagexTarError::Materialization(
            "materialized GNU tar tree lacks required source files".to_string(),
        ));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, TAR_SOURCE_CONTENT_BLAKE3);
    Ok(TarSourceMaterializationReport {
        format: TAR_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: TAR_SOURCE_ARTIFACT_ID,
        record_name: TAR_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: TAR_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_tar_inventory(request: TarInventoryRequest<'_>) -> Result<TarInventoryReport, StagexTarError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexTarError::Materialization(format!("creating GNU tar scratch: {error}")))?;
    let source_root = request.scratch_dir.join("tar-1.12");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), TAR_CONFIG_H)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("lib/getdate_stub.c"), TAR_GETDATE_SOURCE)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexTarError::Materialization(format!("creating GNU tar output: {error}")))?;
    let tar = build_tar(&request, &source_root, &output_root)?;
    run_tar_smokes(&request, &source_root, &tar)?;
    let outputs = collect_outputs(&source_root, &tar)?;
    validate_expected_outputs(&outputs)?;
    let report = TarInventoryReport {
        format: TAR_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: TAR_SOURCE_COMPILE_COUNT,
        build_command_count: TAR_BUILD_COMMAND_COUNT,
        smoke_command_count: TAR_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: TAR_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tar-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexTarError::Materialization(format!("serializing GNU tar report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), TAR_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &TarInventoryRequest<'_>) -> Result<(), StagexTarError> {
    if request.scratch_dir.exists() {
        return Err(StagexTarError::Materialization(format!(
            "create-new GNU tar scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU tar source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexTarError::Materialization(format!(
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
    validate_bound_bytes(TAR_GETDATE_SOURCE, TAR_GETDATE_SOURCE_BLAKE3, "GNU tar getdate stub")?;
    assert!(request.source_root.join("src/tar.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn build_tar(
    request: &TarInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexTarError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    compile_source_set(request, &compiler, source_root, "lib", &TAR_LIB_SOURCES, &["-I..", "-I."])?;
    compile_source_set(request, &compiler, source_root, "src", &TAR_SRC_SOURCES, &["-I..", "-I../lib"])?;
    let link_root = source_root.join("link");
    fs::create_dir(&link_root)
        .map_err(|error| StagexTarError::Materialization(format!("creating GNU tar link root: {error}")))?;
    let object_names = copy_link_objects(source_root, &link_root)?;
    let object_refs = object_names.iter().map(String::as_str).collect::<Vec<_>>();
    let output = output_root.join("tar");
    let args = static_link_args(request.tinycc27_root, &output, &object_refs)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        &link_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tar-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU tar 1.12")?;
    assert_eq!(object_names.len(), usize::try_from(TAR_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn compile_source_set(
    request: &TarInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    directory: &str,
    sources: &[&str],
    include_flags: &[&str],
) -> Result<(), StagexTarError> {
    let current_dir = source_root.join(directory);
    for (index, source_name) in sources.iter().enumerate() {
        let args = compile_args(request.tinycc27_root, source_name, include_flags)?;
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            &current_dir,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("tar-{directory}-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&current_dir.join(format!("{source_name}.o")), source_name)?;
    }
    assert!(!sources.is_empty());
    assert!(current_dir.is_dir());
    Ok(())
}

fn compile_args(
    tinycc27_root: &Path,
    source_name: &str,
    include_flags: &[&str],
) -> Result<Vec<String>, StagexTarError> {
    let include_path = tinycc27_root.join("include/mes");
    let mut args = vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&include_path, "TinyCC include")?.to_string(),
    ];
    args.extend(TAR_COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.extend(include_flags.iter().map(|flag| (*flag).to_string()));
    args.extend([
        "-c".to_string(),
        "-o".to_string(),
        format!("{source_name}.o"),
        format!("{source_name}.c"),
    ]);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(!source_name.contains('/'));
    Ok(args)
}

fn copy_link_objects(source_root: &Path, link_root: &Path) -> Result<Vec<String>, StagexTarError> {
    let mut names = Vec::with_capacity(usize::try_from(TAR_SOURCE_COMPILE_COUNT).unwrap());
    for (directory, sources) in [("lib", TAR_LIB_SOURCES.as_slice()), ("src", TAR_SRC_SOURCES.as_slice())] {
        for source_name in sources {
            let name = format!("{source_name}.o");
            fs::copy(source_root.join(directory).join(&name), link_root.join(&name)).map_err(|error| {
                StagexTarError::Materialization(format!("copying GNU tar link object {directory}/{name}: {error}"))
            })?;
            names.push(name);
        }
    }
    names.sort();
    names.dedup();
    if names.len() != usize::try_from(TAR_SOURCE_COMPILE_COUNT).unwrap() {
        return Err(StagexTarError::Materialization("GNU tar object names collide or are incomplete".to_string()));
    }
    assert!(names.iter().all(|name| link_root.join(name).is_file()));
    assert!(!names.is_empty());
    Ok(names)
}

fn static_link_args(tinycc27_root: &Path, output: &Path, inputs: &[&str]) -> Result<Vec<String>, StagexTarError> {
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
        crate::stagex_mes_lib::utf8_absolute(output, "GNU tar output")?.to_string(),
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

fn run_tar_smokes(request: &TarInventoryRequest<'_>, source_root: &Path, tar: &Path) -> Result<(), StagexTarError> {
    let smoke_root = source_root.join("smoke");
    let input_root = smoke_root.join("in");
    let output_root = smoke_root.join("out");
    fs::create_dir_all(&input_root)
        .and_then(|()| fs::create_dir(&output_root))
        .map_err(|error| StagexTarError::Materialization(format!("creating GNU tar smoke roots: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&input_root.join("file.txt"), TAR_SMOKE_INPUT)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.tar"), TAR_MALFORMED_INPUT)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        tar,
        &["--version"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("tar-version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        tar,
        &["-cf", "../archive.tar", "file.txt"],
        &input_root,
        &empty_env,
        &request.scratch_dir.join("tar-create.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        tar,
        &["-xf", "../archive.tar"],
        &output_root,
        &empty_env,
        &request.scratch_dir.join("tar-extract.stderr.txt"),
    )?;
    require_expected_process_failure(
        tar,
        &["-xf", "smoke/malformed.tar"],
        source_root,
        &request.scratch_dir.join("tar-malformed.stderr.txt"),
    )?;
    let observed = fs::read(output_root.join("file.txt"))
        .map_err(|error| StagexTarError::Materialization(format!("reading GNU tar smoke output: {error}")))?;
    if observed != TAR_SMOKE_INPUT {
        return Err(StagexTarError::Materialization("GNU tar roundtrip output mismatch".to_string()));
    }
    assert_eq!(observed, TAR_SMOKE_INPUT);
    assert!(smoke_root.join("archive.tar").is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexTarError> {
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
        Ok(()) => Err(StagexTarError::Materialization("GNU tar accepted malformed archive input".to_string())),
        Err(error) => Err(StagexTarError::Runtime(error)),
    }
}

fn collect_outputs(source_root: &Path, tar: &Path) -> Result<Vec<TarOutputReport>, StagexTarError> {
    let smoke = source_root.join("smoke/out/file.txt");
    let mut outputs = Vec::with_capacity(TAR_OUTPUT_COUNT);
    for (artifact_id, path) in [("tar-1.12", tar), ("tar-smoke", smoke.as_path())] {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexTarError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(TarOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), TAR_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[TarOutputReport]) -> Result<(), StagexTarError> {
    if outputs.len() != TAR_EXPECTED_OUTPUTS.len() {
        return Err(StagexTarError::Materialization(format!(
            "GNU tar output count mismatch: expected {}, observed {}",
            TAR_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in TAR_EXPECTED_OUTPUTS {
        let Some(output) = outputs.iter().find(|output| output.artifact_id == expected.artifact_id) else {
            return Err(StagexTarError::Materialization(format!("GNU tar output {} is missing", expected.artifact_id)));
        };
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexTarError::Materialization(format!(
                "GNU tar output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), TAR_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexTarError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(TAR_RECORD_NAME))
        .ok_or(StagexTarError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexTarError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != TAR_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexTarError::Materialization(format!(
            "GNU tar source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexTarError::Materialization(
            "GNU tar source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, TAR_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TAR_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(TAR_CONFIG_H);
    hasher.update(TAR_GETDATE_SOURCE_BLAKE3.as_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!TAR_CONFIG_H.is_empty());
    digest
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexTarError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexTarError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexTarError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexTarError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexTarError> {
    let metadata =
        fs::metadata(path).map_err(|error| StagexTarError::Materialization(format!("reading {label}: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > TAR_FILE_BYTES_MAX {
        return Err(StagexTarError::Materialization(format!(
            "{label} is absent, empty, or exceeds {TAR_FILE_BYTES_MAX} bytes"
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= TAR_FILE_BYTES_MAX);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexTarError> {
    validate_nonempty_file(path, label)?;
    let bytes =
        fs::read(path).map_err(|error| StagexTarError::Materialization(format!("reading {label} bytes: {error}")))?;
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
        assert_eq!(digests.len(), TAR_SOURCE_ARTIFACT_COUNT);
        assert!(digests.iter().all(|(_, digest)| digest.len() == blake3::OUT_LEN * 2));
        assert!(digests.iter().all(|(_, digest)| digest.bytes().all(|byte| byte.is_ascii_hexdigit())));
        assert_ne!(digests[0].0, digests[1].0);
    }

    #[test]
    fn compile_recipe_is_relative_and_bounded() {
        let args = compile_args(Path::new("/stagex/tinycc27"), "tar", &["-I..", "-I../lib"]).unwrap();
        assert_eq!(args.last().map(String::as_str), Some("tar.c"));
        assert!(args.iter().all(|argument| !argument.contains("/tmp/")));
        assert!(!args.iter().any(|argument| argument == "../tar.c"));
        assert!(args.len() < 24);
    }

    #[test]
    fn substituted_output_is_rejected() {
        let outputs = vec![
            TarOutputReport {
                artifact_id: "tar-1.12".to_string(),
                path: PathBuf::from("/stagex/tar"),
                bytes_len: 1,
                digest_blake3: "3".repeat(blake3::OUT_LEN * 2),
            },
            TarOutputReport {
                artifact_id: "tar-smoke".to_string(),
                path: PathBuf::from("/stagex/smoke"),
                bytes_len: 1,
                digest_blake3: TAR_SMOKE_BLAKE3.to_string(),
            },
        ];
        let error = validate_expected_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_tar_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TAR_SOURCE_SCRATCH").unwrap());
        let report = materialize_authenticated_tar_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, TAR_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("src/tar.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU tar source and TinyCC 0.9.27"]
    fn derives_retained_tar_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_TAR_SOURCE_ROOT").unwrap());
        let tinycc27 = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TAR_BUILD_SCRATCH").unwrap());
        let report = derive_tar_inventory(TarInventoryRequest {
            source_root: &source,
            tinycc27_root: &tinycc27,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.outputs.len(), TAR_OUTPUT_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
