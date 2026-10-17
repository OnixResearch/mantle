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
const BZIP2_RECORD_NAME: &str = "bzip2-1.0.8-src";
const BZIP2_SOURCE_OUTPUT_NAME: &str = "bzip2-1.0.8";
pub(crate) const BZIP2_SOURCE_ARTIFACT_ID: &str = "bzip2-1.0.8-source";
pub(crate) const BZIP2_SOURCE_CONTENT_BLAKE3: &str = "969e26e40644aaed6137a7403c29397ada624f6dc20d38dfae5fec9d2079292d";
pub(crate) const BZIP2_UTIME_SOURCE_ARTIFACT_ID: &str = "bzip2-utime-header-source";
pub(crate) const BZIP2_UTIME_SOURCE_BLAKE3: &str = "3bf37f6c622b15c4f6acada147c5ca5921da1cfefe9b0cb6dc53c5d10a10de13";
const BZIP2_UTIME_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/bzip2-utime.h");
const BZIP2_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-bzip2-source-materialization-v1";
const BZIP2_REPORT_FORMAT: &str = "mantle-stagex-bzip2-1.0.8-inventory-v1";
const BZIP2_SOURCE_NON_CLAIM: &str =
    "bzip2 source materialization proves authenticated offline archive identity and fixed-output parity only";
const BZIP2_NON_CLAIM: &str = "this inventory binds bzip2 1.0.8, protected source rewrites, and positive and negative compression observations only; it does not prove later bootstrap utilities or provider admission";
const BZIP2_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const BZIP2_REWRITE_COMMAND_COUNT: u32 = 3;
const BZIP2_LIBRARY_COMPILE_COUNT: u32 = 7;
const BZIP2_APP_COMPILE_COUNT: u32 = 2;
const BZIP2_SOURCE_COMPILE_COUNT: u32 = BZIP2_LIBRARY_COMPILE_COUNT + BZIP2_APP_COMPILE_COUNT;
const BZIP2_LINK_COMMAND_COUNT: u32 = 2;
const BZIP2_BUILD_COMMAND_COUNT: u32 = BZIP2_SOURCE_COMPILE_COUNT + BZIP2_LINK_COMMAND_COUNT;
const BZIP2_SMOKE_COMMAND_COUNT: u32 = 4;
const BZIP2_OUTPUT_COUNT: usize = 3;
const BZIP2_SOURCE_ARTIFACT_COUNT: usize = 2;
const BZIP2_SMOKE_INPUT: &[u8] = b"bzip2-108-ok\n";
const BZIP2_MALFORMED_INPUT: &[u8] = b"not bzip2 data\n";
pub(crate) const BZIP2_FINAL_BLAKE3: &str = "b6911b1a367e8b159264078e7976d363e3cb9d268a1dd074ddca51a76916b8a7";
pub(crate) const BZIP2_RECOVER_BLAKE3: &str = "f4bf3e19bd09bcd64503def65eb7bd7eaa5ea43af94536b8d6e638a48501e0a0";
const BZIP2_SMOKE_BLAKE3: &str = "b37fa30b2c16ae67d4e023b19fbecf3b10dab989e5fdedec510c29780cc5793b";
pub(crate) const BZIP2_CONFIGURED_SOURCE_BLAKE3: &str =
    "a3e8d977c0652889d2714f7eb9d1cfbe48581c05c7b7e3b79405334504fedd73";
const BZIP2_COMMON_FLAGS: [&str; 3] = ["-D_FILE_OFFSET_BITS=64", "-I.", "-c"];
const BZIP2_LIBRARY_SOURCES: [&str; BZIP2_LIBRARY_COMPILE_COUNT as usize] = [
    "blocksort",
    "huffman",
    "crctable",
    "randtable",
    "compress",
    "decompress",
    "bzlib",
];
const BZIP2_REWRITES: [(&str, &str); BZIP2_REWRITE_COMMAND_COUNT as usize] = [
    ("s/retVal = utime ( dstName, \\&uTimBuf );/retVal = 0;/", "disable unavailable utime preservation"),
    (
        "s/retVal = fchmod ( fd, fileMetaInfo.st_mode );/retVal = 0;/",
        "disable unavailable fchmod preservation",
    ),
    (
        "s/(void) fchown ( fd, fileMetaInfo.st_uid, fileMetaInfo.st_gid );/(void)0;/",
        "disable unavailable fchown preservation",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bzip2ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const BZIP2_EXPECTED_OUTPUTS: [Bzip2ExpectedOutput; BZIP2_OUTPUT_COUNT] = [
    Bzip2ExpectedOutput {
        artifact_id: "bzip2-1.0.8",
        digest_blake3: BZIP2_FINAL_BLAKE3,
    },
    Bzip2ExpectedOutput {
        artifact_id: "bzip2recover-1.0.8",
        digest_blake3: BZIP2_RECOVER_BLAKE3,
    },
    Bzip2ExpectedOutput {
        artifact_id: "bzip2-smoke",
        digest_blake3: BZIP2_SMOKE_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Bzip2SourceMaterializationReport {
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
pub(crate) struct Bzip2OutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Bzip2InventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub rewrite_command_count: u32,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<Bzip2OutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Bzip2InventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub sed_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexBzip2Error {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexBzip2Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "bzip2 source record was not found"),
            Self::Materialization(message) => write!(formatter, "bzip2 materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "bzip2 runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexBzip2Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexBzip2Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexBzip2Error {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); BZIP2_SOURCE_ARTIFACT_COUNT] {
    [
        (BZIP2_SOURCE_ARTIFACT_ID, BZIP2_SOURCE_CONTENT_BLAKE3),
        (BZIP2_UTIME_SOURCE_ARTIFACT_ID, BZIP2_UTIME_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_bzip2_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<Bzip2SourceMaterializationReport, StagexBzip2Error> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexBzip2Error::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexBzip2Error::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexBzip2Error::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexBzip2Error::Materialization(format!("creating bzip2 source scratch: {error}")))?;
    let output_path = scratch_dir.join(BZIP2_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexBzip2Error::Materialization(format!("materializing bzip2 source {}: {error}", record.identity))
    })?;
    if !output_path.join("bzip2.c").is_file() || !output_path.join("bzlib.c").is_file() {
        return Err(StagexBzip2Error::Materialization(
            "materialized bzip2 tree lacks required source files".to_string(),
        ));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, BZIP2_SOURCE_CONTENT_BLAKE3);
    Ok(Bzip2SourceMaterializationReport {
        format: BZIP2_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: BZIP2_SOURCE_ARTIFACT_ID,
        record_name: BZIP2_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: BZIP2_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_bzip2_inventory(
    request: Bzip2InventoryRequest<'_>,
) -> Result<Bzip2InventoryReport, StagexBzip2Error> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBzip2Error::Materialization(format!("creating bzip2 scratch: {error}")))?;
    let source_root = request.scratch_dir.join(BZIP2_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    crate::stagex_mes_lib::write_create_new(&source_root.join("utime.h"), BZIP2_UTIME_SOURCE)?;
    apply_source_rewrites(&request, &source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexBzip2Error::Materialization(format!("creating bzip2 output: {error}")))?;
    let (bzip2, recover) = build_bzip2(&request, &source_root, &output_root)?;
    run_bzip2_smokes(&request, &source_root, &bzip2)?;
    let outputs = collect_outputs(&source_root, &bzip2, &recover)?;
    validate_expected_outputs(&outputs)?;
    let report = Bzip2InventoryReport {
        format: BZIP2_REPORT_FORMAT,
        configured_source_digest_blake3,
        rewrite_command_count: BZIP2_REWRITE_COMMAND_COUNT,
        source_compile_count: BZIP2_SOURCE_COMPILE_COUNT,
        build_command_count: BZIP2_BUILD_COMMAND_COUNT,
        smoke_command_count: BZIP2_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: BZIP2_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("bzip2-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexBzip2Error::Materialization(format!("serializing bzip2 report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), BZIP2_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &Bzip2InventoryRequest<'_>) -> Result<(), StagexBzip2Error> {
    if request.scratch_dir.exists() {
        return Err(StagexBzip2Error::Materialization(format!(
            "create-new bzip2 scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("bzip2 source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
        ("GNU sed runtime", request.sed_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexBzip2Error::Materialization(format!(
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
    validate_file_digest(&request.sed_root.join("bin/sed"), crate::stagex_sed::SED_FINAL_BLAKE3, "GNU sed 4.0.9")?;
    validate_bound_bytes(BZIP2_UTIME_SOURCE, BZIP2_UTIME_SOURCE_BLAKE3, "bzip2 utime header")?;
    assert!(request.source_root.join("bzip2.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn apply_source_rewrites(request: &Bzip2InventoryRequest<'_>, source_root: &Path) -> Result<(), StagexBzip2Error> {
    let sed = request.sed_root.join("bin/sed");
    for (index, (expression, description)) in BZIP2_REWRITES.iter().enumerate() {
        crate::stagex_mes_lib::run_bounded_process(
            &sed,
            &["-i", *expression, "bzip2.c"],
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("bzip2-rewrite-{index:02}.stderr.txt")),
        )?;
        if description.is_empty() {
            return Err(StagexBzip2Error::Materialization("bzip2 source rewrite lacks a description".to_string()));
        }
    }
    let text = fs::read_to_string(source_root.join("bzip2.c"))
        .map_err(|error| StagexBzip2Error::Materialization(format!("reading rewritten bzip2 source: {error}")))?;
    if text.contains("retVal = fchmod") || text.contains("(void) fchown") || text.contains("retVal = utime") {
        return Err(StagexBzip2Error::Materialization("bzip2 protected source rewrites were incomplete".to_string()));
    }
    assert_eq!(BZIP2_REWRITES.len(), usize::try_from(BZIP2_REWRITE_COMMAND_COUNT).unwrap());
    assert!(text.contains("retVal = 0;"));
    Ok(())
}

fn build_bzip2(
    request: &Bzip2InventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<(PathBuf, PathBuf), StagexBzip2Error> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    for (index, source_name) in BZIP2_LIBRARY_SOURCES.iter().enumerate() {
        compile_source(request, &compiler, source_root, source_name, index)?;
    }
    let bzip2_index = usize::try_from(BZIP2_LIBRARY_COMPILE_COUNT).unwrap();
    compile_source(request, &compiler, source_root, "bzip2", bzip2_index)?;
    let recover_index = bzip2_index.checked_add(1).unwrap();
    compile_source(request, &compiler, source_root, "bzip2recover", recover_index)?;
    let bzip2 = output_root.join("bzip2");
    let mut bzip2_objects = vec!["bzip2.o"];
    bzip2_objects.extend(BZIP2_LIBRARY_SOURCES.iter().copied());
    let bzip2_object_names = bzip2_objects
        .iter()
        .map(|name| {
            if name.ends_with(".o") {
                (*name).to_string()
            } else {
                format!("{name}.o")
            }
        })
        .collect::<Vec<_>>();
    link_executable(request, &compiler, source_root, &bzip2, &bzip2_object_names, "bzip2-link")?;
    let recover = output_root.join("bzip2recover");
    link_executable(request, &compiler, source_root, &recover, &["bzip2recover.o".to_string()], "bzip2recover-link")?;
    crate::stagex_tinycc::set_owner_executable(&bzip2)?;
    crate::stagex_tinycc::set_owner_executable(&recover)?;
    validate_nonempty_file(&bzip2, "bzip2 1.0.8")?;
    validate_nonempty_file(&recover, "bzip2recover 1.0.8")?;
    assert!(bzip2.is_file());
    assert!(recover.is_file());
    Ok((bzip2, recover))
}

fn compile_source(
    request: &Bzip2InventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    source_name: &str,
    index: usize,
) -> Result<(), StagexBzip2Error> {
    let mut args = compiler_prefix(request.tinycc27_root)?;
    args.extend(BZIP2_COMMON_FLAGS.iter().map(|flag| (*flag).to_string()));
    args.extend([format!("{source_name}.c"), "-o".to_string(), format!("{source_name}.o")]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("bzip2-compile-{index:02}.stderr.txt")),
    )?;
    validate_nonempty_file(&source_root.join(format!("{source_name}.o")), source_name)?;
    assert!(!source_name.contains('/'));
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(())
}

fn link_executable(
    request: &Bzip2InventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output: &Path,
    objects: &[String],
    log_name: &str,
) -> Result<(), StagexBzip2Error> {
    let libdir = request.tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let mut args = vec![
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC runtime")?.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "bzip2 output")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.extend([
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("{log_name}.stderr.txt")),
    )?;
    assert!(!objects.is_empty());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(())
}

fn compiler_prefix(tinycc27_root: &Path) -> Result<Vec<String>, StagexBzip2Error> {
    Ok(vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
    ])
}

fn run_bzip2_smokes(
    request: &Bzip2InventoryRequest<'_>,
    source_root: &Path,
    bzip2: &Path,
) -> Result<(), StagexBzip2Error> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexBzip2Error::Materialization(format!("creating bzip2 smoke root: {error}")))?;
    let smoke_file = smoke_root.join("smoke.txt");
    crate::stagex_mes_lib::write_create_new(&smoke_file, BZIP2_SMOKE_INPUT)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.bz2"), BZIP2_MALFORMED_INPUT)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        bzip2,
        &["--help"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("bzip2-help.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        bzip2,
        &["smoke/smoke.txt"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("bzip2-compress.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        bzip2,
        &["-d", "smoke/smoke.txt.bz2"],
        source_root,
        &empty_env,
        &request.scratch_dir.join("bzip2-decompress.stderr.txt"),
    )?;
    require_expected_process_failure(
        bzip2,
        &["-d", "smoke/malformed.bz2"],
        source_root,
        &request.scratch_dir.join("bzip2-malformed.stderr.txt"),
    )?;
    let observed = fs::read(&smoke_file)
        .map_err(|error| StagexBzip2Error::Materialization(format!("reading bzip2 smoke output: {error}")))?;
    if observed != BZIP2_SMOKE_INPUT {
        return Err(StagexBzip2Error::Materialization("bzip2 compression roundtrip output mismatch".to_string()));
    }
    assert_eq!(observed, BZIP2_SMOKE_INPUT);
    assert!(smoke_file.is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexBzip2Error> {
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
        Ok(()) => Err(StagexBzip2Error::Materialization("bzip2 accepted malformed compressed input".to_string())),
        Err(error) => Err(StagexBzip2Error::Runtime(error)),
    }
}

fn collect_outputs(
    source_root: &Path,
    bzip2: &Path,
    recover: &Path,
) -> Result<Vec<Bzip2OutputReport>, StagexBzip2Error> {
    let smoke = source_root.join("smoke/smoke.txt");
    let mut outputs = Vec::with_capacity(BZIP2_OUTPUT_COUNT);
    for (artifact_id, path) in [
        ("bzip2-1.0.8", bzip2),
        ("bzip2recover-1.0.8", recover),
        ("bzip2-smoke", smoke.as_path()),
    ] {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexBzip2Error::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(Bzip2OutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), BZIP2_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[Bzip2OutputReport]) -> Result<(), StagexBzip2Error> {
    if outputs.len() != BZIP2_EXPECTED_OUTPUTS.len() {
        return Err(StagexBzip2Error::Materialization(format!(
            "bzip2 output count mismatch: expected {}, observed {}",
            BZIP2_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    for expected in BZIP2_EXPECTED_OUTPUTS {
        let Some(output) = outputs.iter().find(|output| output.artifact_id == expected.artifact_id) else {
            return Err(StagexBzip2Error::Materialization(format!("bzip2 output {} is missing", expected.artifact_id)));
        };
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexBzip2Error::Materialization(format!(
                "bzip2 output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), BZIP2_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexBzip2Error> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(BZIP2_RECORD_NAME))
        .ok_or(StagexBzip2Error::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexBzip2Error> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != BZIP2_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexBzip2Error::Materialization(format!(
            "bzip2 source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexBzip2Error::Materialization(
            "bzip2 source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, BZIP2_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-bzip2-configured-source-v1\0");
    hasher.update(BZIP2_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(BZIP2_UTIME_SOURCE);
    for value in BZIP2_COMMON_FLAGS
        .iter()
        .chain(BZIP2_LIBRARY_SOURCES.iter())
        .chain(BZIP2_REWRITES.iter().flat_map(|(expression, description)| [expression, description]))
    {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexBzip2Error> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if bytes.is_empty() || observed != expected {
        return Err(StagexBzip2Error::Materialization(format!(
            "{label} source BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexBzip2Error> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexBzip2Error::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert!(path.is_file());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexBzip2Error> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexBzip2Error::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > BZIP2_FILE_BYTES_MAX {
        return Err(StagexBzip2Error::Materialization(format!(
            "{label} is empty, oversized, or not a file: {}",
            path.display()
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= BZIP2_FILE_BYTES_MAX);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexBzip2Error> {
    validate_nonempty_file(path, label)?;
    let bytes =
        fs::read(path).map_err(|error| StagexBzip2Error::Materialization(format!("reading {label} bytes: {error}")))?;
    let bytes_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if bytes_len > BZIP2_FILE_BYTES_MAX {
        return Err(StagexBzip2Error::Materialization(format!("{label} exceeds {BZIP2_FILE_BYTES_MAX} bytes")));
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
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_BZIP2_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_SED_ROOT_ENV: &str = "MANTLE_STAGE_X_SED_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_BZIP2_BUILD_SCRATCH";

    fn source_record(kind: SourceRecordKind, digest: &str) -> SourceRecord {
        SourceRecord {
            kind,
            identity: "fixed-url-bzip2-test".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([("name".to_string(), BZIP2_RECORD_NAME.to_string())]),
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
            store_path_attestation: None,
        }
    }

    #[test]
    fn validates_exact_bzip2_source_record() {
        let record = source_record(SourceRecordKind::FixedUrl, BZIP2_SOURCE_CONTENT_BLAKE3);
        validate_source_record(&record).unwrap();
        assert_eq!(record.kind, SourceRecordKind::FixedUrl);
        assert_eq!(record.content_blake3, BZIP2_SOURCE_CONTENT_BLAKE3);
    }

    #[test]
    fn rejects_substituted_bzip2_source_record() {
        let wrong_kind = source_record(SourceRecordKind::VcsSnapshot, BZIP2_SOURCE_CONTENT_BLAKE3);
        let wrong_digest = source_record(SourceRecordKind::FixedUrl, &"a".repeat(blake3::OUT_LEN * 2));
        assert!(validate_source_record(&wrong_kind).is_err());
        assert!(validate_source_record(&wrong_digest).is_err());
    }

    #[test]
    fn configured_source_digest_is_stable_and_binds_rewrites() {
        let first = configured_source_digest_blake3();
        let second = configured_source_digest_blake3();
        assert_eq!(first, second);
        assert_eq!(first, BZIP2_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(BZIP2_REWRITES.len(), usize::try_from(BZIP2_REWRITE_COMMAND_COUNT).unwrap());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_bzip2_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let report = materialize_authenticated_bzip2_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, BZIP2_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("bzip2.c").is_file());
    }

    #[test]
    #[ignore = "requires retained bzip2 source, GNU sed, and TinyCC 0.9.27"]
    fn derives_retained_bzip2_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tinycc27_root = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let sed_root = PathBuf::from(std::env::var(RETAINED_SED_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_bzip2_inventory(Bzip2InventoryRequest {
            source_root: &source_root,
            tinycc27_root: &tinycc27_root,
            sed_root: &sed_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.rewrite_command_count, BZIP2_REWRITE_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
