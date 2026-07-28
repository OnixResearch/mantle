use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileEntry;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const MAKE_RECORD_NAME: &str = "make-3.82-src";
const MAKE_SOURCE_OUTPUT_NAME: &str = "make-3.82";
pub(crate) const MAKE_SOURCE_ARTIFACT_ID: &str = "make-3.82-source";
pub(crate) const MAKE_SOURCE_CONTENT_BLAKE3: &str = "b768ff74b8f16f6c41fb869833582c7cf614e98a8728a1c74616a6b6157ece77";
const MAKE_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-make-source-materialization-v1";
const MAKE_SOURCE_ARTIFACT_COUNT: usize = 2;
const MAKE_SOURCE_NON_CLAIM: &str =
    "GNU Make source materialization proves authenticated offline archive identity and fixed-output parity only";
const MAKE_PATCH_BYTES: &[u8] = include_bytes!("../bootstrap/patches/make-3.82-stagex.patch");
pub(crate) const MAKE_PATCH_BLAKE3: &str = "51c890dd1f9a2ddf816c68ebfbd4506241e45e3993ca4f14151c0d8665f87e59";
const MAKE_PATCH_FILE_COUNT: usize = 1;
const MAKE_REPORT_FORMAT: &str = "mantle-stagex-make-3.82-inventory-v1";
const MAKE_NON_CLAIM: &str = "this inventory binds GNU Make 3.82, a bounded source-built recipe runner, and positive and negative smoke observations only; it does not prove a general shell, later toolchain stages, or provider admission";
const MAKE_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const MAKE_SOURCE_COMPILE_COUNT: u32 = 27;
const MAKE_BUILD_COMMAND_COUNT: u32 = MAKE_SOURCE_COMPILE_COUNT + 1;
const MAKE_RECIPE_RUNNER_COMMAND_COUNT: u32 = 1;
const MAKE_SMOKE_COMMAND_COUNT: u32 = 3;
const MAKE_OUTPUT_COUNT: usize = 3;
pub(crate) const MAKE_FINAL_BLAKE3: &str = "8a5aa115eab6069a8f69cf1b2c63c69c478d64b271bc13fda7ea4e3531b841d1";
pub(crate) const MAKE_RECIPE_RUNNER_BLAKE3: &str = "028ececbb67a3394595027c9273a44ed2ba5b465c209767589b5a989032b93d1";
const MAKE_PATCHED_SOURCE_BLAKE3: &str = "32f6a43a61aeb01f8f5c95739f1d67eae26c90ebf1fdf658ce7ab52bf1d147e9";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MakeExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const MAKE_EXPECTED_OUTPUTS: [MakeExpectedOutput; MAKE_OUTPUT_COUNT] = [
    MakeExpectedOutput {
        artifact_id: "make-3.82",
        digest_blake3: MAKE_FINAL_BLAKE3,
    },
    MakeExpectedOutput {
        artifact_id: "make-recipe-runner",
        digest_blake3: MAKE_RECIPE_RUNNER_BLAKE3,
    },
    MakeExpectedOutput {
        artifact_id: "make-recipe-smoke",
        digest_blake3: "8dd632d8df367d6751cc2e2497e8d9c38e0c5a1e58b100d59e1c0fdac235e3a2",
    },
];
const MAKE_RECIPE_RUNNER_SOURCE: &[u8] = include_bytes!("../bootstrap/seeds/make-recipe-runner.c");
pub(crate) const MAKE_RECIPE_RUNNER_SOURCE_ARTIFACT_ID: &str = "make-recipe-runner-source";
pub(crate) const MAKE_RECIPE_RUNNER_SOURCE_BLAKE3: &str =
    "2d6a9757ecb8ba18de695edadd2473f14cc8c51427c8a922310c13f97221f6e8";
const MAKE_POSITIVE_EXPECTED: &[u8] = b"smoke-ok\n";
const MAKE_MALFORMED_SOURCE: &[u8] = b"all:\nnot-a-recipe\n";
const MAKE_RECIPE_TEXT: &str = "write smoke/out smoke-ok";
const MAKE_COMMON_FLAGS: [&str; 8] = [
    "-DSTDC_HEADERS",
    "-DHAVE_STDLIB_H",
    "-DHAVE_STRING_H",
    "-DHAVE_UNISTD_H",
    "-DHAVE_FCNTL_H",
    "-DHAVE_SYS_WAIT_H",
    "-DHAVE_WAITPID",
    "-DPOSIX",
];

#[derive(Debug, Clone, Copy)]
struct MakeCompileSpec {
    source: &'static str,
    output: &'static str,
    flags: &'static [&'static str],
}

const MAKE_COMPILE_SPECS: [MakeCompileSpec; MAKE_SOURCE_COMPILE_COUNT as usize] = [
    MakeCompileSpec {
        source: "getopt.c",
        output: "getopt.o",
        flags: &[],
    },
    MakeCompileSpec {
        source: "getopt1.c",
        output: "getopt1.o",
        flags: &[],
    },
    MakeCompileSpec {
        source: "ar.c",
        output: "ar.o",
        flags: &[
            "-I.",
            "-Iglob",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DHAVE_STDINT_H",
        ],
    },
    MakeCompileSpec {
        source: "arscan.c",
        output: "arscan.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART", "-DHAVE_FCNTL_H"],
    },
    MakeCompileSpec {
        source: "commands.c",
        output: "commands.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DFILE_TIMESTAMP_HI_RES=0",
        ],
    },
    MakeCompileSpec {
        source: "default.c",
        output: "default.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DSCCS_GET=\"/nullop\"",
        ],
    },
    MakeCompileSpec {
        source: "dir.c",
        output: "dir.o",
        flags: &[
            "-I.",
            "-Iglob",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DHAVE_DIRENT_H",
        ],
    },
    MakeCompileSpec {
        source: "expand.c",
        output: "expand.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "file.c",
        output: "file.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DFILE_TIMESTAMP_HI_RES=0",
        ],
    },
    MakeCompileSpec {
        source: "function.c",
        output: "function.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART", "-Dvfork=fork"],
    },
    MakeCompileSpec {
        source: "implicit.c",
        output: "implicit.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "job.c",
        output: "job.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DHAVE_DUP2",
            "-DHAVE_STRCHR",
            "-Dvfork=fork",
        ],
    },
    MakeCompileSpec {
        source: "main.c",
        output: "main.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DNO_FLOAT",
            "-DLOCALEDIR=\"/fake\"",
            "-DPACKAGE=\"make\"",
            "-DHAVE_MKTEMP",
            "-DHAVE_GETCWD",
        ],
    },
    MakeCompileSpec {
        source: "misc.c",
        output: "misc.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DHAVE_STRERROR",
            "-DHAVE_VPRINTF",
            "-DHAVE_ANSI_COMPILER",
            "-DHAVE_STDARG_H",
        ],
    },
    MakeCompileSpec {
        source: "read.c",
        output: "read.o",
        flags: &[
            "-I.",
            "-Iglob",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DINCLUDEDIR=\"/include\"",
        ],
    },
    MakeCompileSpec {
        source: "remake.c",
        output: "remake.o",
        flags: &[
            "-I.",
            "-DHAVE_INTTYPES_H",
            "-DHAVE_SA_RESTART",
            "-DFILE_TIMESTAMP_HI_RES=0",
            "-DHAVE_FCNTL_H",
            "-DLIBDIR=\"/lib\"",
        ],
    },
    MakeCompileSpec {
        source: "rule.c",
        output: "rule.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "signame.c",
        output: "signame.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "strcache.c",
        output: "strcache.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "variable.c",
        output: "variable.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "version.c",
        output: "version.o",
        flags: &["-I.", "-DVERSION=\"3.82\""],
    },
    MakeCompileSpec {
        source: "vpath.c",
        output: "vpath.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "hash.c",
        output: "hash.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "remote-stub.c",
        output: "remote-stub.o",
        flags: &["-I.", "-DHAVE_INTTYPES_H", "-DHAVE_SA_RESTART"],
    },
    MakeCompileSpec {
        source: "getloadavg.c",
        output: "getloadavg.o",
        flags: &["-DHAVE_FCNTL_H"],
    },
    MakeCompileSpec {
        source: "glob/fnmatch.c",
        output: "fnmatch.o",
        flags: &["-Iglob"],
    },
    MakeCompileSpec {
        source: "glob/glob.c",
        output: "glob.o",
        flags: &["-Iglob", "-DHAVE_STRDUP", "-DHAVE_DIRENT_H"],
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MakeSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct MakeInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MakeOutput {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MakeInventoryReport {
    pub format: &'static str,
    pub source_patch_digest_blake3: String,
    pub patched_files: Vec<String>,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub recipe_runner_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<MakeOutput>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum StagexMakeError {
    Bundle(crate::RunError),
    InvalidAuthority(String),
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexMakeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "reading GNU Make source bundle: {error}"),
            Self::InvalidAuthority(message) => write!(formatter, "invalid GNU Make source authority: {message}"),
            Self::Materialization(message) => write!(formatter, "materializing GNU Make source: {message}"),
            Self::Runtime(error) => write!(formatter, "building GNU Make 3.82: {error}"),
        }
    }
}

impl std::error::Error for StagexMakeError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexMakeError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexMakeError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::RunError> for StagexMakeError {
    fn from(error: crate::RunError) -> Self {
        Self::Bundle(error)
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); MAKE_SOURCE_ARTIFACT_COUNT] {
    [
        (MAKE_SOURCE_ARTIFACT_ID, MAKE_SOURCE_CONTENT_BLAKE3),
        (MAKE_RECIPE_RUNNER_SOURCE_ARTIFACT_ID, MAKE_RECIPE_RUNNER_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_make_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    source_root: &Path,
) -> Result<MakeSourceMaterializationReport, StagexMakeError> {
    let manifest = read_source_bundle(bundle_path)?;
    validate_manifest_identity(&manifest.manifest_blake3, expected_manifest_blake3)?;
    let record = select_make_source_record(&manifest.records)?;
    fs::create_dir(source_root).map_err(|error| {
        StagexMakeError::Materialization(format!(
            "creating create-new GNU Make source root {}: {error}",
            source_root.display()
        ))
    })?;
    let output_path = source_root.join(MAKE_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexMakeError::Materialization(format!(
            "materializing {MAKE_RECORD_NAME} at {}: {error}",
            output_path.display()
        ))
    })?;
    if !output_path.is_dir() || !output_path.join("main.c").is_file() {
        return Err(StagexMakeError::Materialization(format!(
            "materialized GNU Make tree lacks main.c: {}",
            output_path.display()
        )));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, MAKE_SOURCE_CONTENT_BLAKE3);
    Ok(MakeSourceMaterializationReport {
        format: MAKE_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: MAKE_SOURCE_ARTIFACT_ID,
        record_name: MAKE_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: MAKE_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_make_inventory(request: MakeInventoryRequest<'_>) -> Result<MakeInventoryReport, StagexMakeError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexMakeError::Materialization(format!("creating create-new GNU Make scratch: {error}")))?;
    let make_root = request.scratch_dir.join("make-3.82");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &make_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&make_root)?;
    fs::write(make_root.join("config.h"), [])
        .map_err(|error| StagexMakeError::Materialization(format!("writing GNU Make config.h: {error}")))?;
    let patched_files = apply_make_source_patch(&make_root)?;
    let source_patch_digest_blake3 = patched_make_source_digest(&make_root, &patched_files)?;
    if source_patch_digest_blake3 != MAKE_PATCHED_SOURCE_BLAKE3 {
        return Err(StagexMakeError::Materialization(format!(
            "patched GNU Make source BLAKE3 mismatch: expected {MAKE_PATCHED_SOURCE_BLAKE3}, observed {source_patch_digest_blake3}"
        )));
    }
    let output_root = request.scratch_dir.join("output");
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| StagexMakeError::Materialization(format!("creating GNU Make output bin: {error}")))?;
    let recipe_runner = build_recipe_runner(&request, &output_root)?;
    let make = build_make(&request, &make_root, &output_root)?;
    run_make_smokes(&request, &make_root, &make, &recipe_runner)?;
    let outputs = collect_make_outputs(&request, &make, &recipe_runner)?;
    validate_expected_make_outputs(&outputs)?;
    let report = MakeInventoryReport {
        format: MAKE_REPORT_FORMAT,
        source_patch_digest_blake3,
        patched_files,
        source_compile_count: MAKE_SOURCE_COMPILE_COUNT,
        build_command_count: MAKE_BUILD_COMMAND_COUNT,
        recipe_runner_command_count: MAKE_RECIPE_RUNNER_COMMAND_COUNT,
        smoke_command_count: MAKE_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: MAKE_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("make-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexMakeError::Materialization(format!("serializing GNU Make report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), MAKE_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &MakeInventoryRequest<'_>) -> Result<(), StagexMakeError> {
    if request.scratch_dir.exists() {
        return Err(StagexMakeError::Materialization(format!(
            "create-new GNU Make scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU Make source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexMakeError::Materialization(format!(
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
    if !request.source_root.join("main.c").is_file() || !request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file() {
        return Err(StagexMakeError::Materialization(
            "GNU Make source or TinyCC runtime lacks a required file".to_string(),
        ));
    }
    assert!(request.tinycc27_root.join("include/mes").is_dir());
    assert!(request.source_root.join("glob/glob.c").is_file());
    Ok(())
}

fn build_recipe_runner(request: &MakeInventoryRequest<'_>, output_root: &Path) -> Result<PathBuf, StagexMakeError> {
    let build_root = request.scratch_dir.join("recipe-runner-build");
    fs::create_dir(&build_root).map_err(|error| {
        StagexMakeError::Materialization(format!("creating Make recipe-runner build root: {error}"))
    })?;
    validate_bound_bytes(MAKE_RECIPE_RUNNER_SOURCE, MAKE_RECIPE_RUNNER_SOURCE_BLAKE3, "GNU Make recipe-runner source")?;
    crate::stagex_mes_lib::write_create_new(&build_root.join("recipe-runner.c"), MAKE_RECIPE_RUNNER_SOURCE)?;
    let output = output_root.join("bin/make-recipe-runner");
    let args = static_tinycc_link_args(request.tinycc27_root, &output, &["recipe-runner.c"])?;
    crate::stagex_mes_lib::run_bounded_process(
        &request.tinycc27_root.join("bin/tcc"),
        &args,
        &build_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("recipe-runner-build.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU Make recipe runner")?;
    assert!(output.is_absolute());
    assert!(output.is_file());
    Ok(output)
}

fn build_make(
    request: &MakeInventoryRequest<'_>,
    make_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexMakeError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    for (index, spec) in MAKE_COMPILE_SPECS.iter().enumerate() {
        let args = make_compile_args(request.tinycc27_root, spec)?;
        crate::stagex_mes_lib::run_bounded_process(
            &compiler,
            &args,
            make_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("make-compile-{index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&make_root.join(spec.output), spec.output)?;
    }
    let output = output_root.join("bin/make");
    let objects = MAKE_COMPILE_SPECS.iter().map(|spec| spec.output).collect::<Vec<_>>();
    let link_args = static_tinycc_link_args(request.tinycc27_root, &output, &objects)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &link_args,
        make_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("make-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU Make 3.82")?;
    assert_eq!(MAKE_COMPILE_SPECS.len(), usize::try_from(MAKE_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn make_compile_args(tinycc27_root: &Path, spec: &MakeCompileSpec) -> Result<Vec<String>, StagexMakeError> {
    let mut args = MAKE_COMMON_FLAGS.iter().map(|flag| (*flag).to_string()).collect::<Vec<_>>();
    args.extend([
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
        "-c".to_string(),
    ]);
    args.extend(spec.flags.iter().map(|flag| (*flag).to_string()));
    args.extend(["-o".to_string(), spec.output.to_string(), spec.source.to_string()]);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(spec.source.ends_with(".c"));
    Ok(args)
}

fn static_tinycc_link_args(
    tinycc27_root: &Path,
    output: &Path,
    inputs: &[&str],
) -> Result<Vec<String>, StagexMakeError> {
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
        crate::stagex_mes_lib::utf8_absolute(output, "TinyCC static output")?.to_string(),
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

fn run_make_smokes(
    request: &MakeInventoryRequest<'_>,
    make_root: &Path,
    make: &Path,
    recipe_runner: &Path,
) -> Result<(), StagexMakeError> {
    let smoke_root = make_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexMakeError::Materialization(format!("creating GNU Make smoke root: {error}")))?;
    let recipe_runner = crate::stagex_mes_lib::utf8_absolute(recipe_runner, "GNU Make recipe runner")?;
    let makefile = format!("SHELL := {recipe_runner}\n.SHELLFLAGS := -c\nall:\n\t{MAKE_RECIPE_TEXT}\n");
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("Makefile"), makefile.as_bytes())?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("Malformed.mk"), MAKE_MALFORMED_SOURCE)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        make,
        &["--version"],
        make_root,
        &empty_env,
        &request.scratch_dir.join("make-version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        make,
        &["-f", "smoke/Makefile", "all"],
        make_root,
        &empty_env,
        &request.scratch_dir.join("make-recipe.stderr.txt"),
    )?;
    require_expected_process_failure(
        make,
        &["-f", "smoke/Malformed.mk", "all"],
        make_root,
        &request.scratch_dir.join("make-malformed.stderr.txt"),
    )?;
    let observed = fs::read(smoke_root.join("out"))
        .map_err(|error| StagexMakeError::Materialization(format!("reading GNU Make smoke output: {error}")))?;
    if observed != MAKE_POSITIVE_EXPECTED {
        return Err(StagexMakeError::Materialization(format!(
            "GNU Make recipe smoke output mismatch: observed {:?}",
            String::from_utf8_lossy(&observed)
        )));
    }
    assert_eq!(observed, MAKE_POSITIVE_EXPECTED);
    assert!(smoke_root.join("out").is_file());
    Ok(())
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexMakeError> {
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
        Ok(()) => Err(StagexMakeError::Materialization("GNU Make accepted malformed Makefile".to_string())),
        Err(error) => Err(StagexMakeError::Runtime(error)),
    }
}

fn collect_make_outputs(
    request: &MakeInventoryRequest<'_>,
    make: &Path,
    recipe_runner: &Path,
) -> Result<Vec<MakeOutput>, StagexMakeError> {
    let specs = [
        ("make-3.82", make.to_path_buf()),
        ("make-recipe-runner", recipe_runner.to_path_buf()),
        ("make-recipe-smoke", request.scratch_dir.join("make-3.82/smoke/out")),
    ];
    let mut outputs = Vec::with_capacity(specs.len());
    for (artifact_id, path) in specs {
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, MAKE_FILE_BYTES_MAX, artifact_id)?;
        outputs.push(MakeOutput {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| StagexMakeError::Materialization("GNU Make output size does not fit u64".to_string()))?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), MAKE_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_make_outputs(outputs: &[MakeOutput]) -> Result<(), StagexMakeError> {
    if outputs.len() != MAKE_OUTPUT_COUNT {
        return Err(StagexMakeError::Materialization(format!(
            "GNU Make report has {} outputs; expected {MAKE_OUTPUT_COUNT}",
            outputs.len()
        )));
    }
    for expected in MAKE_EXPECTED_OUTPUTS {
        let output = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexMakeError::Materialization(format!("GNU Make report lacks {}", expected.artifact_id))
        })?;
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(StagexMakeError::Materialization(format!(
                "GNU Make output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), MAKE_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| !output.digest_blake3.is_empty()));
    Ok(())
}

pub(crate) fn apply_make_source_patch(root: &Path) -> Result<Vec<String>, StagexMakeError> {
    validate_bound_bytes(MAKE_PATCH_BYTES, MAKE_PATCH_BLAKE3, "GNU Make source patch")?;
    let patched = crate::stagex_patch::apply_unified_patch_tree(root, MAKE_PATCH_BYTES)
        .map_err(|error| StagexMakeError::Materialization(error.to_string()))?;
    if patched.len() != MAKE_PATCH_FILE_COUNT {
        return Err(StagexMakeError::Materialization(format!(
            "GNU Make patch changed {} files; expected {MAKE_PATCH_FILE_COUNT}",
            patched.len()
        )));
    }
    assert_eq!(patched.len(), MAKE_PATCH_FILE_COUNT);
    assert!(patched.iter().all(|path| root.join(path).is_file()));
    Ok(patched)
}

fn patched_make_source_digest(root: &Path, files: &[String]) -> Result<String, StagexMakeError> {
    let mut sorted = files.to_vec();
    sorted.sort();
    let mut hasher = blake3::Hasher::new();
    for relative in &sorted {
        let bytes = crate::stagex_mes_lib::read_bounded_file(
            &root.join(relative),
            MAKE_FILE_BYTES_MAX,
            "patched GNU Make source",
        )?;
        let path_len = u64::try_from(relative.len())
            .map_err(|_| StagexMakeError::Materialization("GNU Make patch path length overflow".to_string()))?;
        let byte_len = u64::try_from(bytes.len())
            .map_err(|_| StagexMakeError::Materialization("GNU Make patch byte length overflow".to_string()))?;
        hasher.update(&path_len.to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&byte_len.to_le_bytes());
        hasher.update(&bytes);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert_eq!(sorted.len(), MAKE_PATCH_FILE_COUNT);
    Ok(digest)
}

fn validate_bound_bytes(bytes: &[u8], expected_blake3: &str, label: &str) -> Result<(), StagexMakeError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if expected_blake3.len() != blake3::OUT_LEN * 2 || observed != expected_blake3 {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "{label} BLAKE3 mismatch: expected {expected_blake3}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed, expected_blake3);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexMakeError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, MAKE_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed, expected);
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexMakeError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, MAKE_FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(StagexMakeError::Materialization(format!("{label} is empty: {}", path.display())));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_manifest_identity(observed: &str, expected: &str) -> Result<(), StagexMakeError> {
    if expected.len() != blake3::OUT_LEN * 2 || observed != expected {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "source bundle BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert_eq!(observed, expected);
    Ok(())
}

fn select_make_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexMakeError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(MAKE_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "source bundle has {} records named {MAKE_RECORD_NAME}; expected exactly one",
            matches.len()
        )));
    }
    validate_make_source_record(matches[0])?;
    assert!(matches[0].payload_bytes > 0);
    assert!(!matches[0].identity.is_empty());
    Ok(matches[0])
}

fn validate_make_source_record(record: &SourceRecord) -> Result<(), StagexMakeError> {
    if record.kind != SourceRecordKind::FixedUrl
        || record.content_blake3 != MAKE_SOURCE_CONTENT_BLAKE3
        || record.files.is_empty()
    {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "source record {MAKE_RECORD_NAME} has substituted kind, digest, or empty payload: {}",
            record.identity
        )));
    }
    if !record.files.iter().all(valid_source_file_entry) {
        return Err(StagexMakeError::InvalidAuthority(format!(
            "source record {MAKE_RECORD_NAME} has a malformed file entry: {}",
            record.identity
        )));
    }
    assert!(!record.files.is_empty());
    assert_eq!(record.content_blake3, MAKE_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn valid_source_file_entry(file: &SourceFileEntry) -> bool {
    let content_shape_valid = file.content_hex.is_some() || file.chunk_count.is_some();
    let path_valid = !file.path.is_empty() && !file.path.starts_with('/');
    content_shape_valid && path_valid && !file.blake3.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_bundle::SourceFileType;

    const TEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn record() -> SourceRecord {
        SourceRecord {
            kind: SourceRecordKind::FixedUrl,
            identity: "fixed-url-make".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), MAKE_RECORD_NAME.to_string())]),
            payload_bytes: 1,
            content_blake3: MAKE_SOURCE_CONTENT_BLAKE3.to_string(),
            files: vec![SourceFileEntry {
                path: "archive".to_string(),
                file_type: SourceFileType::Regular,
                executable: false,
                size: 1,
                content_hex: Some("78".to_string()),
                symlink_target: None,
                chunk_index: None,
                chunk_count: None,
                blake3: TEST_BLAKE3.to_string(),
            }],
        }
    }

    #[test]
    fn selects_exact_make_source_and_rejects_substitution() {
        let valid = record();
        let selected = select_make_source_record(std::slice::from_ref(&valid)).unwrap();
        assert_eq!(selected.content_blake3, MAKE_SOURCE_CONTENT_BLAKE3);
        assert_eq!(selected.kind, SourceRecordKind::FixedUrl);

        let mut substituted = record();
        substituted.content_blake3 = TEST_BLAKE3.to_string();
        let error = select_make_source_record(&[substituted]).unwrap_err();
        assert!(error.to_string().contains("substituted kind, digest, or empty payload"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn rejects_substituted_make_manifest_identity() {
        let error = validate_manifest_identity(TEST_BLAKE3, MAKE_SOURCE_CONTENT_BLAKE3).unwrap_err();
        assert!(error.to_string().contains("source bundle BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn rejects_substituted_make_patch() {
        let error = validate_bound_bytes(b"substituted", MAKE_PATCH_BLAKE3, "test patch").unwrap_err();
        assert!(error.to_string().contains("test patch BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn validates_exact_make_outputs_and_rejects_substitution() {
        let mut outputs = MAKE_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| MakeOutput {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_expected_make_outputs(&outputs).unwrap();
        assert_eq!(outputs.len(), MAKE_OUTPUT_COUNT);

        outputs[0].digest_blake3 = TEST_BLAKE3.to_string();
        let error = validate_expected_make_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("output make-3.82 BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires explicit materialized GNU Make source and create-new scratch"]
    fn applies_retained_make_source_patch() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_MAKE_SOURCE_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_MAKE_PATCH_SCRATCH").unwrap());
        crate::stagex_mes_lib::copy_tree_bounded(&source, &scratch).unwrap();
        crate::stagex_tinycc::make_tree_owner_writable(&scratch).unwrap();

        let patched = apply_make_source_patch(&scratch).unwrap();

        assert_eq!(patched.len(), MAKE_PATCH_FILE_COUNT);
        assert!(scratch.join("main.c").is_file());
    }

    #[test]
    #[ignore = "requires explicit GNU Make source, TinyCC 0.9.27 runtime, and create-new scratch"]
    fn derives_retained_make_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_MAKE_SOURCE_ROOT").unwrap());
        let tinycc27 = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_MAKE_BUILD_SCRATCH").unwrap());

        let report = derive_make_inventory(MakeInventoryRequest {
            source_root: &source,
            tinycc27_root: &tinycc27,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();

        assert_eq!(report.outputs.len(), MAKE_OUTPUT_COUNT);
        assert_eq!(report.source_compile_count, MAKE_SOURCE_COMPILE_COUNT);
        assert!(scratch.join("make-inventory.json").is_file());
        assert!(report.fallback_events.is_empty());
    }

    #[test]
    #[ignore = "requires explicit authenticated source bundle and create-new scratch"]
    fn materializes_retained_make_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_MAKE_SOURCE_SCRATCH").unwrap());

        let report = materialize_authenticated_make_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        fs::write(scratch.join("make-source-materialization.json"), serde_json::to_vec_pretty(&report).unwrap())
            .unwrap();

        assert!(report.output_path.is_dir());
        assert_eq!(report.record_content_blake3, MAKE_SOURCE_CONTENT_BLAKE3);
        assert!(scratch.join("make-source-materialization.json").is_file());
    }
}
