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
const GNU_PATCH_RECORD_NAME: &str = "patch-2.5.9-src";
const GNU_PATCH_SOURCE_OUTPUT_NAME: &str = "patch-2.5.9";
pub(crate) const GNU_PATCH_SOURCE_ARTIFACT_ID: &str = "gnu-patch-2.5.9-source";
pub(crate) const GNU_PATCH_SOURCE_CONTENT_BLAKE3: &str =
    "d7dd692b74dea83bc6b6aed6efbd9f874aa991a1ba5722e8f4b06bf0c20a21e0";
const GNU_PATCH_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-gnu-patch-source-materialization-v1";
const GNU_PATCH_SOURCE_NON_CLAIM: &str =
    "GNU patch source materialization proves authenticated offline archive identity and fixed-output parity only";
const GNU_PATCH_REPORT_FORMAT: &str = "mantle-stagex-gnu-patch-2.5.9-inventory-v1";
const GNU_PATCH_NON_CLAIM: &str = "this inventory binds GNU patch 2.5.9 and positive and negative smoke observations only; it does not prove a general shell, later toolchain stages, or provider admission";
const GNU_PATCH_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const GNU_PATCH_SOURCE_COMPILE_COUNT: u32 = 19;
const GNU_PATCH_BUILD_COMMAND_COUNT: u32 = GNU_PATCH_SOURCE_COMPILE_COUNT + 1;
const GNU_PATCH_SMOKE_COMMAND_COUNT: u32 = 3;
const GNU_PATCH_OUTPUT_COUNT: usize = 2;
pub(crate) const GNU_PATCH_FINAL_BLAKE3: &str = "2193ea653bfbc16acc2609a5c7ef6c0b03a7a65dc50e6525d23868598e25fbcb";
const GNU_PATCH_SMOKE_BLAKE3: &str = "79d1d8da0b625035cdbfc9d51841030861b9f4cf7c5abbe442a8d13efc352170";
const GNU_PATCH_CONFIG_H: &[u8] = br#"#define PACKAGE_BUGREPORT ""
#define PACKAGE_NAME "patch"
#define PACKAGE_VERSION "2.5.9"
#define PACKAGE "patch"
#define VERSION "2.5.9"
#define ed_PROGRAM "/nullop"
#define HAVE_DECL_GETENV 1
#define HAVE_DECL_MALLOC 1
#define HAVE_DECL_FREE 1
#define HAVE_DECL_MKTEMP 1
#define HAVE_DECL_STRERROR 1
#define HAVE_DECL_STRERROR_R 0
#define HAVE_DIRENT_H 1
#define HAVE_LIMITS_H 1
#define HAVE_GETEUID 1
#define HAVE_MKTEMP 1
#define HAVE_MKDIR 1
#define HAVE_RMDIR 1
#define HAVE_FCNTL_H 1
#define HAVE_MALLOC 1
#define HAVE_REALLOC 1
#define HAVE_STRING_H 1
#define HAVE_STRINGS_H 1
#define HAVE_STDLIB_H 1
#define HAVE_UNISTD_H 1
#define HAVE_UTIME_H 0
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_STDINT_H 1
#define HAVE_STDDEF_H 1
#define HAVE_VPRINTF 1
#define HAVE_GETUID 1
#define HAVE_STRERROR 1
#define HAVE_MEMCHR 1
#define HAVE_MEMCMP 1
#define HAVE_LONG_FILE_NAMES 1
#define HAVE_STRUCT_UTIMBUF 0
#define PROTOTYPES 1
#define STDC_HEADERS 1
#define mbstate_t void*
#define RETSIGTYPE int
"#;
const GNU_PATCH_SOURCE_NAMES: [&str; GNU_PATCH_SOURCE_COMPILE_COUNT as usize] = [
    "error",
    "getopt",
    "getopt1",
    "addext",
    "argmatch",
    "backupfile",
    "basename",
    "dirname",
    "inp",
    "maketime",
    "partime",
    "patch",
    "pch",
    "quote",
    "quotearg",
    "quotesys",
    "util",
    "version",
    "xmalloc",
];
const GNU_PATCH_POSITIVE_INPUT: &[u8] = b"old\n";
const GNU_PATCH_POSITIVE_EXPECTED: &[u8] = b"new\n";
const GNU_PATCH_POSITIVE_DIFF: &[u8] = b"--- smoke.txt\n+++ smoke.txt\n@@ -1 +1 @@\n-old\n+new\n";
const GNU_PATCH_MALFORMED_DIFF: &[u8] = b"this is not a patch\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GnuPatchExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const GNU_PATCH_EXPECTED_OUTPUTS: [GnuPatchExpectedOutput; GNU_PATCH_OUTPUT_COUNT] = [
    GnuPatchExpectedOutput {
        artifact_id: "gnu-patch-2.5.9",
        digest_blake3: GNU_PATCH_FINAL_BLAKE3,
    },
    GnuPatchExpectedOutput {
        artifact_id: "gnu-patch-smoke",
        digest_blake3: GNU_PATCH_SMOKE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GnuPatchSourceMaterializationReport {
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
pub(crate) struct GnuPatchOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GnuPatchInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<GnuPatchOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GnuPatchInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexGnuPatchError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexGnuPatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU patch source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU patch source materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU patch runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexGnuPatchError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexGnuPatchError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexGnuPatchError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digest() -> (&'static str, &'static str) {
    (GNU_PATCH_SOURCE_ARTIFACT_ID, GNU_PATCH_SOURCE_CONTENT_BLAKE3)
}

pub(crate) fn materialize_authenticated_gnu_patch_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<GnuPatchSourceMaterializationReport, StagexGnuPatchError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexGnuPatchError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexGnuPatchError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("creating GNU patch source scratch: {error}")))?;
    let output_path = scratch_dir.join(GNU_PATCH_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexGnuPatchError::Materialization(format!("materializing GNU patch source {}: {error}", record.identity))
    })?;
    validate_materialized_source(record, &output_path)?;
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, GNU_PATCH_SOURCE_CONTENT_BLAKE3);
    Ok(GnuPatchSourceMaterializationReport {
        format: GNU_PATCH_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: GNU_PATCH_SOURCE_ARTIFACT_ID,
        record_name: GNU_PATCH_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: GNU_PATCH_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_gnu_patch_inventory(
    request: GnuPatchInventoryRequest<'_>,
) -> Result<GnuPatchInventoryReport, StagexGnuPatchError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("creating GNU patch scratch: {error}")))?;
    let source_root = request.scratch_dir.join("patch-2.5.9");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("config.h"), GNU_PATCH_CONFIG_H)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("patchlevel.h"), &[])?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("creating GNU patch output: {error}")))?;
    let patch = build_gnu_patch(&request, &source_root, &output_root)?;
    run_gnu_patch_smokes(&request, &source_root, &patch)?;
    let outputs = collect_outputs(&request, &patch)?;
    validate_expected_outputs(&outputs)?;
    let report = GnuPatchInventoryReport {
        format: GNU_PATCH_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: GNU_PATCH_SOURCE_COMPILE_COUNT,
        build_command_count: GNU_PATCH_BUILD_COMMAND_COUNT,
        smoke_command_count: GNU_PATCH_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: GNU_PATCH_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("gnu-patch-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexGnuPatchError::Materialization(format!("serializing GNU patch report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), GNU_PATCH_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &GnuPatchInventoryRequest<'_>) -> Result<(), StagexGnuPatchError> {
    if request.scratch_dir.exists() {
        return Err(StagexGnuPatchError::Materialization(format!(
            "create-new GNU patch scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU patch source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexGnuPatchError::Materialization(format!(
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
    if !request.source_root.join("patch.c").is_file() || !request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file()
    {
        return Err(StagexGnuPatchError::Materialization(
            "GNU patch source or TinyCC runtime lacks a required file".to_string(),
        ));
    }
    assert!(request.source_root.join("pch.c").is_file());
    assert!(request.tinycc27_root.join("include/mes").is_dir());
    Ok(())
}

fn build_gnu_patch(
    request: &GnuPatchInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexGnuPatchError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    for (index, source_name) in GNU_PATCH_SOURCE_NAMES.iter().enumerate() {
        let args = compile_args(request.tinycc27_root, source_name)?;
        crate::stagex_mes_lib::run_bounded_process(
            &compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("gnu-patch-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&source_root.join(format!("{source_name}.o")), source_name)?;
    }
    let output = output_root.join("patch");
    let link_args = link_args(request.tinycc27_root, &output)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &link_args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("gnu-patch-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU patch 2.5.9")?;
    assert_eq!(GNU_PATCH_SOURCE_NAMES.len(), usize::try_from(GNU_PATCH_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn compile_args(tinycc27_root: &Path, source_name: &str) -> Result<Vec<String>, StagexGnuPatchError> {
    let include_path = tinycc27_root.join("include/mes");
    let include = crate::stagex_mes_lib::utf8_absolute(&include_path, "TinyCC include")?;
    let args = vec![
        "-I".to_string(),
        include.to_string(),
        "-I.".to_string(),
        "-DHAVE_CONFIG_H".to_string(),
        "-c".to_string(),
        "-o".to_string(),
        format!("{source_name}.o"),
        format!("{source_name}.c"),
    ];
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(!source_name.contains('/'));
    Ok(args)
}

fn link_args(tinycc27_root: &Path, output: &Path) -> Result<Vec<String>, StagexGnuPatchError> {
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
        crate::stagex_mes_lib::utf8_absolute(output, "GNU patch output")?.to_string(),
    ];
    args.extend(GNU_PATCH_SOURCE_NAMES.iter().map(|name| format!("{name}.o")));
    args.extend([
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ]);
    assert!(args.len() > GNU_PATCH_SOURCE_NAMES.len());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn run_gnu_patch_smokes(
    request: &GnuPatchInventoryRequest<'_>,
    source_root: &Path,
    patch: &Path,
) -> Result<(), StagexGnuPatchError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("creating GNU patch smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("smoke.txt"), GNU_PATCH_POSITIVE_INPUT)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("smoke.diff"), GNU_PATCH_POSITIVE_DIFF)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.diff"), GNU_PATCH_MALFORMED_DIFF)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        patch,
        &["--version"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("gnu-patch-version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        patch,
        &["--input=smoke/smoke.diff", "smoke/smoke.txt"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("gnu-patch-smoke.stderr.txt"),
    )?;
    require_expected_process_failure(
        patch,
        &["--input=smoke/malformed.diff", "smoke/smoke.txt"],
        source_root,
        &request.scratch_dir.join("gnu-patch-malformed.stderr.txt"),
    )?;
    let observed = fs::read(smoke_root.join("smoke.txt"))
        .map_err(|error| StagexGnuPatchError::Materialization(format!("reading GNU patch smoke output: {error}")))?;
    if observed != GNU_PATCH_POSITIVE_EXPECTED {
        return Err(StagexGnuPatchError::Materialization(format!(
            "GNU patch smoke output mismatch: observed {:?}",
            String::from_utf8_lossy(&observed)
        )));
    }
    assert_eq!(observed, GNU_PATCH_POSITIVE_EXPECTED);
    assert!(smoke_root.join("smoke.txt").is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexGnuPatchError> {
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
        Ok(()) => Err(StagexGnuPatchError::Materialization("GNU patch accepted malformed patch input".to_string())),
        Err(error) => Err(StagexGnuPatchError::Runtime(error)),
    }
}

fn collect_outputs(
    request: &GnuPatchInventoryRequest<'_>,
    patch: &Path,
) -> Result<Vec<GnuPatchOutputReport>, StagexGnuPatchError> {
    let smoke = request.scratch_dir.join("patch-2.5.9/smoke/smoke.txt");
    let mut outputs = Vec::with_capacity(GNU_PATCH_OUTPUT_COUNT);
    for (artifact_id, path) in [("gnu-patch-2.5.9", patch), ("gnu-patch-smoke", smoke.as_path())] {
        let metadata = fs::metadata(path).map_err(|error| {
            StagexGnuPatchError::Materialization(format!("reading {artifact_id} metadata: {error}"))
        })?;
        outputs.push(GnuPatchOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), GNU_PATCH_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[GnuPatchOutputReport]) -> Result<(), StagexGnuPatchError> {
    if outputs.len() != GNU_PATCH_EXPECTED_OUTPUTS.len() {
        return Err(StagexGnuPatchError::Materialization(format!(
            "GNU patch output count mismatch: expected {}, observed {}",
            GNU_PATCH_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in GNU_PATCH_EXPECTED_OUTPUTS {
        let Some(output) = outputs.iter().find(|output| output.artifact_id == expected.artifact_id) else {
            return Err(StagexGnuPatchError::Materialization(format!(
                "GNU patch output {} is missing",
                expected.artifact_id
            )));
        };
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexGnuPatchError::Materialization(format!(
                "GNU patch output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), GNU_PATCH_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexGnuPatchError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(GNU_PATCH_RECORD_NAME))
        .ok_or(StagexGnuPatchError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexGnuPatchError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != GNU_PATCH_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexGnuPatchError::Materialization(format!(
            "GNU patch source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexGnuPatchError::Materialization(
            "GNU patch source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, GNU_PATCH_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn validate_materialized_source(record: &SourceRecord, output_path: &Path) -> Result<(), StagexGnuPatchError> {
    if !output_path.join("patch.c").is_file() || !output_path.join("pch.c").is_file() {
        return Err(StagexGnuPatchError::Materialization(format!(
            "GNU patch record {} did not materialize expected source files",
            record.identity
        )));
    }
    assert_eq!(record.content_blake3, GNU_PATCH_SOURCE_CONTENT_BLAKE3);
    assert!(output_path.is_dir());
    assert!(output_path.join("patch.c").is_file());
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(GNU_PATCH_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"config.h\0");
    hasher.update(GNU_PATCH_CONFIG_H);
    hasher.update(b"patchlevel.h\0");
    hasher.update(&[]);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!GNU_PATCH_CONFIG_H.is_empty());
    digest
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexGnuPatchError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexGnuPatchError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexGnuPatchError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("reading {label}: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > GNU_PATCH_FILE_BYTES_MAX {
        return Err(StagexGnuPatchError::Materialization(format!(
            "{label} is absent, empty, or exceeds {GNU_PATCH_FILE_BYTES_MAX} bytes"
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= GNU_PATCH_FILE_BYTES_MAX);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexGnuPatchError> {
    validate_nonempty_file(path, label)?;
    let bytes = fs::read(path)
        .map_err(|error| StagexGnuPatchError::Materialization(format!("reading {label} bytes: {error}")))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_artifact_digest_is_closed() {
        let (artifact_id, digest) = source_artifact_digest();
        assert_eq!(artifact_id, GNU_PATCH_SOURCE_ARTIFACT_ID);
        assert_eq!(digest, GNU_PATCH_SOURCE_CONTENT_BLAKE3);
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn compile_recipe_is_relative_and_bounded() {
        let root = Path::new("/stagex/tinycc27");
        let args = compile_args(root, "patch").unwrap();
        assert_eq!(args.last().map(String::as_str), Some("patch.c"));
        assert!(args.iter().all(|argument| !argument.contains("/tmp/")));
        assert!(!args.iter().any(|argument| argument == "../patch.c"));
        assert!(args.len() < 16);
    }

    #[test]
    fn substituted_output_is_rejected() {
        let outputs = vec![
            GnuPatchOutputReport {
                artifact_id: "gnu-patch-2.5.9".to_string(),
                path: PathBuf::from("/stagex/patch"),
                bytes_len: 1,
                digest_blake3: "3".repeat(blake3::OUT_LEN * 2),
            },
            GnuPatchOutputReport {
                artifact_id: "gnu-patch-smoke".to_string(),
                path: PathBuf::from("/stagex/smoke"),
                bytes_len: 1,
                digest_blake3: GNU_PATCH_SMOKE_BLAKE3.to_string(),
            },
        ];
        let error = validate_expected_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_gnu_patch_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_GNU_PATCH_SOURCE_SCRATCH").unwrap());
        let report = materialize_authenticated_gnu_patch_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, GNU_PATCH_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("patch.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU patch source and TinyCC 0.9.27"]
    fn derives_retained_gnu_patch_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_GNU_PATCH_SOURCE_ROOT").unwrap());
        let tinycc27 = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_GNU_PATCH_BUILD_SCRATCH").unwrap());
        let report = derive_gnu_patch_inventory(GnuPatchInventoryRequest {
            source_root: &source,
            tinycc27_root: &tinycc27,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.outputs.len(), GNU_PATCH_OUTPUT_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
