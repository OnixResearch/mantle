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
const SED_RECORD_NAME: &str = "sed-4.0.9-src";
const SED_SOURCE_OUTPUT_NAME: &str = "sed-4.0.9";
pub(crate) const SED_SOURCE_ARTIFACT_ID: &str = "sed-4.0.9-source";
pub(crate) const SED_SOURCE_CONTENT_BLAKE3: &str = "65aa1a57a2966a247b4992de6525e319adb99f8ad2fe4cc2eb543a28740994ad";
const SED_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-sed-source-materialization-v1";
const SED_REPORT_FORMAT: &str = "mantle-stagex-sed-4.0.9-inventory-v1";
const SED_SOURCE_NON_CLAIM: &str =
    "GNU sed source materialization proves authenticated offline archive identity and fixed-output parity only";
const SED_NON_CLAIM: &str = "this inventory binds GNU sed 4.0.9 and positive and negative substitution observations only; it does not prove later bootstrap utilities or provider admission";
const SED_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const SED_LIB_COMPILE_COUNT: u32 = 8;
const SED_APP_COMPILE_COUNT: u32 = 5;
const SED_SOURCE_COMPILE_COUNT: u32 = SED_LIB_COMPILE_COUNT + SED_APP_COMPILE_COUNT;
const SED_BUILD_COMMAND_COUNT: u32 = SED_SOURCE_COMPILE_COUNT + 1;
const SED_SMOKE_COMMAND_COUNT: u32 = 3;
const SED_OUTPUT_COUNT: usize = 2;
const SED_SOURCE_ARTIFACT_COUNT: usize = 1;
const SED_SMOKE_INPUT: &[u8] = b"sed409-input\n";
const SED_SMOKE_EXPECTED: &[u8] = b"sed409-ok\n";
pub(crate) const SED_FINAL_BLAKE3: &str = "cc4bc898fbad65d9c0a07eab85c61c99abdb9d7d7d747948067bea7d9954618a";
const SED_SMOKE_BLAKE3: &str = "91f02abc6d98e0f00da4f3b1249ae0acd29544f7b19304f190d29f523885c077";
const SED_CONFIG_H: &[u8] = b"";
const SED_COMMON_FLAGS: [&str; 9] = [
    "-DENABLE_NLS=0",
    "-DHAVE_FCNTL_H",
    "-DHAVE_ALLOCA_H",
    "-DSED_FEATURE_VERSION=\"4.0\"",
    "-DVERSION=\"4.0.9\"",
    "-DPACKAGE=\"sed\"",
    "-I.",
    "-Ilib",
    "-c",
];
const SED_LIB_SOURCES: [&str; SED_LIB_COMPILE_COUNT as usize] = [
    "getline",
    "getopt1",
    "getopt",
    "utils",
    "obstack",
    "strverscmp",
    "mkstemp",
    "regex",
];
const SED_APP_SOURCES: [&str; SED_APP_COMPILE_COUNT as usize] = ["compile", "execute", "regexp", "fmt", "sed"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SedExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const SED_EXPECTED_OUTPUTS: [SedExpectedOutput; SED_OUTPUT_COUNT] = [
    SedExpectedOutput {
        artifact_id: "sed-4.0.9",
        digest_blake3: SED_FINAL_BLAKE3,
    },
    SedExpectedOutput {
        artifact_id: "sed-smoke",
        digest_blake3: SED_SMOKE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SedSourceMaterializationReport {
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
pub(crate) struct SedOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SedInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<SedOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SedInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexSedError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexSedError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU sed source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU sed materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU sed runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexSedError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexSedError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexSedError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); SED_SOURCE_ARTIFACT_COUNT] {
    [(SED_SOURCE_ARTIFACT_ID, SED_SOURCE_CONTENT_BLAKE3)]
}

pub(crate) fn materialize_authenticated_sed_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<SedSourceMaterializationReport, StagexSedError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexSedError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexSedError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexSedError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexSedError::Materialization(format!("creating GNU sed source scratch: {error}")))?;
    let output_path = scratch_dir.join(SED_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexSedError::Materialization(format!("materializing GNU sed source {}: {error}", record.identity))
    })?;
    if !output_path.join("sed/sed.c").is_file() || !output_path.join("lib/regex.c").is_file() {
        return Err(StagexSedError::Materialization(
            "materialized GNU sed tree lacks required source files".to_string(),
        ));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, SED_SOURCE_CONTENT_BLAKE3);
    Ok(SedSourceMaterializationReport {
        format: SED_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: SED_SOURCE_ARTIFACT_ID,
        record_name: SED_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: SED_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_sed_inventory(request: SedInventoryRequest<'_>) -> Result<SedInventoryReport, StagexSedError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexSedError::Materialization(format!("creating GNU sed scratch: {error}")))?;
    let source_root = request.scratch_dir.join(SED_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), SED_CONFIG_H)?;
    fs::copy(source_root.join("lib/regex_.h"), source_root.join("lib/regex.h"))
        .map_err(|error| StagexSedError::Materialization(format!("copying GNU sed regex header: {error}")))?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexSedError::Materialization(format!("creating GNU sed output: {error}")))?;
    let sed = build_sed(&request, &source_root, &output_root)?;
    run_sed_smokes(&request, &source_root, &sed)?;
    let outputs = collect_outputs(&source_root, &sed)?;
    validate_expected_outputs(&outputs)?;
    let report = SedInventoryReport {
        format: SED_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: SED_SOURCE_COMPILE_COUNT,
        build_command_count: SED_BUILD_COMMAND_COUNT,
        smoke_command_count: SED_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: SED_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("sed-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexSedError::Materialization(format!("serializing GNU sed report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), SED_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &SedInventoryRequest<'_>) -> Result<(), StagexSedError> {
    if request.scratch_dir.exists() {
        return Err(StagexSedError::Materialization(format!(
            "create-new GNU sed scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU sed source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexSedError::Materialization(format!(
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
    assert!(request.source_root.join("sed/sed.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn build_sed(
    request: &SedInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexSedError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    compile_source_set(request, &compiler, source_root, "lib", &SED_LIB_SOURCES)?;
    compile_source_set(request, &compiler, source_root, "sed", &SED_APP_SOURCES)?;
    let link_root = source_root.join("link");
    fs::create_dir(&link_root)
        .map_err(|error| StagexSedError::Materialization(format!("creating GNU sed link root: {error}")))?;
    let object_names = copy_link_objects(source_root, &link_root)?;
    let object_refs = object_names.iter().map(String::as_str).collect::<Vec<_>>();
    let output = output_root.join("sed");
    let args = static_link_args(request.tinycc27_root, &output, &object_refs)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        &link_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("sed-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU sed 4.0.9")?;
    assert_eq!(object_names.len(), usize::try_from(SED_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn compile_source_set(
    request: &SedInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    directory: &str,
    sources: &[&str],
) -> Result<(), StagexSedError> {
    for (index, source_name) in sources.iter().enumerate() {
        let output = source_root.join(directory).join(format!("{source_name}.o"));
        let source = source_root.join(directory).join(format!("{source_name}.c"));
        let args = compile_args(request.tinycc27_root, &output, &source)?;
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("sed-{directory}-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&output, source_name)?;
    }
    assert!(!sources.is_empty());
    assert!(source_root.join(directory).is_dir());
    Ok(())
}

fn compile_args(tinycc27_root: &Path, output: &Path, source: &Path) -> Result<Vec<String>, StagexSedError> {
    let include_path = tinycc27_root.join("include/mes");
    let mut args = vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&include_path, "TinyCC include")?.to_string(),
    ];
    args.extend(SED_COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.extend([
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "GNU sed object output")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(source, "GNU sed source")?.to_string(),
    ]);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(output.is_absolute());
    Ok(args)
}

fn copy_link_objects(source_root: &Path, link_root: &Path) -> Result<Vec<String>, StagexSedError> {
    let mut names = Vec::with_capacity(usize::try_from(SED_SOURCE_COMPILE_COUNT).unwrap());
    for (directory, sources) in [("lib", SED_LIB_SOURCES.as_slice()), ("sed", SED_APP_SOURCES.as_slice())] {
        for source_name in sources {
            let target_name = format!("{directory}-{source_name}.o");
            fs::copy(source_root.join(directory).join(format!("{source_name}.o")), link_root.join(&target_name))
                .map_err(|error| {
                    StagexSedError::Materialization(format!(
                        "copying GNU sed link object {directory}/{source_name}: {error}"
                    ))
                })?;
            names.push(target_name);
        }
    }
    names.sort();
    names.dedup();
    if names.len() != usize::try_from(SED_SOURCE_COMPILE_COUNT).unwrap() {
        return Err(StagexSedError::Materialization("GNU sed object names collide or are incomplete".to_string()));
    }
    assert!(names.iter().all(|name| link_root.join(name).is_file()));
    assert!(!names.is_empty());
    Ok(names)
}

fn static_link_args(tinycc27_root: &Path, output: &Path, inputs: &[&str]) -> Result<Vec<String>, StagexSedError> {
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
        crate::stagex_mes_lib::utf8_absolute(output, "GNU sed output")?.to_string(),
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

fn run_sed_smokes(request: &SedInventoryRequest<'_>, source_root: &Path, sed: &Path) -> Result<(), StagexSedError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexSedError::Materialization(format!("creating GNU sed smoke root: {error}")))?;
    let smoke_file = smoke_root.join("smoke.txt");
    crate::stagex_mes_lib::write_create_new(&smoke_file, SED_SMOKE_INPUT)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        sed,
        &["--version"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("sed-version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        sed,
        &["-i", "s/input/ok/", "smoke/smoke.txt"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("sed-smoke.stderr.txt"),
    )?;
    require_expected_process_failure(
        sed,
        &["-e", "s/[//", "smoke/smoke.txt"],
        source_root,
        &request.scratch_dir.join("sed-malformed.stderr.txt"),
    )?;
    let observed = fs::read(&smoke_file)
        .map_err(|error| StagexSedError::Materialization(format!("reading GNU sed smoke output: {error}")))?;
    if observed != SED_SMOKE_EXPECTED {
        return Err(StagexSedError::Materialization(format!(
            "GNU sed smoke output mismatch: observed {:?}",
            String::from_utf8_lossy(&observed)
        )));
    }
    assert_eq!(observed, SED_SMOKE_EXPECTED);
    assert!(smoke_file.is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexSedError> {
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
        Ok(()) => Err(StagexSedError::Materialization("GNU sed accepted malformed substitution input".to_string())),
        Err(error) => Err(StagexSedError::Runtime(error)),
    }
}

fn collect_outputs(source_root: &Path, sed: &Path) -> Result<Vec<SedOutputReport>, StagexSedError> {
    let smoke = source_root.join("smoke/smoke.txt");
    let mut outputs = Vec::with_capacity(SED_OUTPUT_COUNT);
    for (artifact_id, path) in [("sed-4.0.9", sed), ("sed-smoke", smoke.as_path())] {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexSedError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(SedOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), SED_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[SedOutputReport]) -> Result<(), StagexSedError> {
    if outputs.len() != SED_EXPECTED_OUTPUTS.len() {
        return Err(StagexSedError::Materialization(format!(
            "GNU sed output count mismatch: expected {}, observed {}",
            SED_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in SED_EXPECTED_OUTPUTS {
        let Some(output) = outputs.iter().find(|output| output.artifact_id == expected.artifact_id) else {
            return Err(StagexSedError::Materialization(format!("GNU sed output {} is missing", expected.artifact_id)));
        };
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexSedError::Materialization(format!(
                "GNU sed output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), SED_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexSedError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(SED_RECORD_NAME))
        .ok_or(StagexSedError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexSedError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != SED_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexSedError::Materialization(format!(
            "GNU sed source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexSedError::Materialization(
            "GNU sed source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, SED_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-sed-configured-source-v1\0");
    hasher.update(SED_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(SED_CONFIG_H);
    for value in SED_COMMON_FLAGS.iter().chain(SED_LIB_SOURCES.iter()).chain(SED_APP_SOURCES.iter()) {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexSedError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexSedError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert!(path.is_file());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexSedError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexSedError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > SED_FILE_BYTES_MAX {
        return Err(StagexSedError::Materialization(format!(
            "{label} is empty, oversized, or not a file: {}",
            path.display()
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= SED_FILE_BYTES_MAX);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexSedError> {
    validate_nonempty_file(path, label)?;
    let bytes =
        fs::read(path).map_err(|error| StagexSedError::Materialization(format!("reading {label} bytes: {error}")))?;
    let bytes_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if bytes_len > SED_FILE_BYTES_MAX {
        return Err(StagexSedError::Materialization(format!("{label} exceeds {SED_FILE_BYTES_MAX} bytes")));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::source_bundle::SourceFileEntry;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_SED_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_SED_BUILD_SCRATCH";

    fn source_record(kind: SourceRecordKind, digest: &str) -> SourceRecord {
        SourceRecord {
            kind,
            identity: "fixed-url-sed-test".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([("name".to_string(), SED_RECORD_NAME.to_string())]),
            payload_bytes: 1,
            content_blake3: digest.to_string(),
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
    fn validates_exact_sed_source_record() {
        let record = source_record(SourceRecordKind::FixedUrl, SED_SOURCE_CONTENT_BLAKE3);
        validate_source_record(&record).unwrap();
        assert_eq!(record.kind, SourceRecordKind::FixedUrl);
        assert_eq!(record.content_blake3, SED_SOURCE_CONTENT_BLAKE3);
    }

    #[test]
    fn rejects_substituted_sed_source_record() {
        let wrong_kind = source_record(SourceRecordKind::VcsSnapshot, SED_SOURCE_CONTENT_BLAKE3);
        let wrong_digest = source_record(SourceRecordKind::FixedUrl, &"a".repeat(blake3::OUT_LEN * 2));
        assert!(validate_source_record(&wrong_kind).is_err());
        assert!(validate_source_record(&wrong_digest).is_err());
    }

    #[test]
    fn configured_source_digest_is_stable_and_nonempty() {
        let first = configured_source_digest_blake3();
        let second = configured_source_digest_blake3();
        assert_eq!(first, second);
        assert_eq!(first.len(), blake3::OUT_LEN * 2);
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_sed_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let report = materialize_authenticated_sed_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, SED_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("sed/sed.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU sed source and TinyCC 0.9.27 runtime"]
    fn derives_retained_sed_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tinycc27_root = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_sed_inventory(SedInventoryRequest {
            source_root: &source_root,
            tinycc27_root: &tinycc27_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.source_compile_count, SED_SOURCE_COMPILE_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
