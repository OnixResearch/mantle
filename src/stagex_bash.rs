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
const BASH_RECORD_NAME: &str = "bash-2.05b-src";
const BASH_SOURCE_OUTPUT_NAME: &str = "bash-2.05b";
pub(crate) const BASH_SOURCE_ARTIFACT_ID: &str = "bash-2.05b-source";
pub(crate) const BASH_SOURCE_CONTENT_BLAKE3: &str = "130da251856869c8d04a52d35299181bc6c3be8e38624e731a9323382c3d755e";
pub(crate) const BASH_RECIPE_SOURCE_ARTIFACT_ID: &str = "bash-2.05b-recipe-source";
pub(crate) const BASH_RECIPE_SOURCE_BLAKE3: &str = "83fa7ce8da2012b7efab048046f52b6a1416e71058704b086478419bffe0994d";
const BASH_RECIPE_SOURCE: &[u8] = include_bytes!("../bootstrap/bash-2.05b-tcc.ncl");
const BASH_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-bash-source-materialization-v1";
const BASH_REPORT_FORMAT: &str = "mantle-stagex-bash-2.05b-inventory-v1";
const BASH_SOURCE_NON_CLAIM: &str =
    "bash source materialization proves authenticated offline archive identity and fixed-output parity only";
const BASH_NON_CLAIM: &str = "this inventory binds the bounded non-interactive bash 2.05b handoff and positive and negative command observations only; it does not prove a complete interactive shell or provider admission";
const BASH_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const BASH_HELPER_COUNT: usize = 10;
const BASH_CORE_COMPILE_COUNT: u32 = 42;
const BASH_GENERATED_COMPILE_COUNT: u32 = 2;
const BASH_GLOB_COMPILE_COUNT: u32 = 3;
const BASH_TILDE_COMPILE_COUNT: u32 = 1;
const BASH_SH_LIBRARY_COMPILE_COUNT: u32 = 39;
const BASH_BUILTIN_COMPILE_COUNT: u32 = 5;
const BASH_SOURCE_COMPILE_COUNT: u32 = BASH_CORE_COMPILE_COUNT
    + BASH_GENERATED_COMPILE_COUNT
    + BASH_GLOB_COMPILE_COUNT
    + BASH_TILDE_COMPILE_COUNT
    + BASH_SH_LIBRARY_COMPILE_COUNT
    + BASH_BUILTIN_COMPILE_COUNT;
const BASH_LINK_COMMAND_COUNT: u32 = 1;
const BASH_BUILD_COMMAND_COUNT: u32 = BASH_SOURCE_COMPILE_COUNT + BASH_LINK_COMMAND_COUNT;
const BASH_SMOKE_COMMAND_COUNT: u32 = 3;
const BASH_OUTPUT_COUNT: usize = 3;
const BASH_SOURCE_ARTIFACT_COUNT: usize = 2 + BASH_HELPER_COUNT;
pub(crate) const BASH_CONFIGURED_SOURCE_BLAKE3: &str =
    "acd33bededf76dc149f7c7fa14a2e1a1246c5dafe2ef65eee18cf0d82e3d6bcd";
pub(crate) const BASH_FINAL_BLAKE3: &str = "f7326a0fcc0878e7cd3616640cee376f71bab5583460b5459cac91168809c03f";
const BASH_SMOKE_OUTPUT_BLAKE3: &str = "f9bb7985b1ac20ccbfdd1195397be795edb7996ab02e6f173d166517c19eb579";
const BASH_SMOKE_OUTPUT: &[u8] = b"bash-ok\n";

#[derive(Debug, Clone, Copy)]
struct BashHelperSpec {
    artifact_id: &'static str,
    tag: &'static str,
    target: &'static str,
    digest_blake3: &'static str,
}

const BASH_HELPERS: [BashHelperSpec; BASH_HELPER_COUNT] = [
    BashHelperSpec {
        artifact_id: "bash-config-header-source",
        tag: "CONFIG",
        target: "config.h",
        digest_blake3: "c4c01b7f8f7129e8f6228d84d9792f05d79b345effcc715436cb5f3dd5f900ff",
    },
    BashHelperSpec {
        artifact_id: "bash-pathnames-header-source",
        tag: "PATHS",
        target: "pathnames.h",
        digest_blake3: "37f38802c7756f7fa6f316697b8c45ab40408fc37445b0acaf95a020f7506e64",
    },
    BashHelperSpec {
        artifact_id: "bash-signames-header-source",
        tag: "SIGS",
        target: "signames.h",
        digest_blake3: "9f778f7eb9163e51926466add39958654cc6f8ddf64bbc24b65d1ec9a331e0e4",
    },
    BashHelperSpec {
        artifact_id: "bash-version-header-source",
        tag: "VER",
        target: "version.h",
        digest_blake3: "2e23bab4e73551bf46dd410fc474e088c2586b0bd72483e751a8315c8790ddef",
    },
    BashHelperSpec {
        artifact_id: "bash-builtext-header-source",
        tag: "BUILTEXT",
        target: "builtins/builtext.h",
        digest_blake3: "3a39f97f535b38b4765ca9ab98866e44ef89e62bcde36de3502dbf572b27e0a3",
    },
    BashHelperSpec {
        artifact_id: "bash-termios-header-source",
        tag: "TERM",
        target: "termios.h",
        digest_blake3: "916c161448de17a494684e30eb1dfb4a474cb7c901f79dcc056067effdef2827",
    },
    BashHelperSpec {
        artifact_id: "bash-syntax-source",
        tag: "SYN",
        target: "syntax.c",
        digest_blake3: "17dcfcc315eec400baa889b9e109c93831e86018d275877ba9d19701ac2a4706",
    },
    BashHelperSpec {
        artifact_id: "bash-siglist-source",
        tag: "SIGL",
        target: "siglist.c",
        digest_blake3: "85d458eb329da9adb9c86e1333791a99e0253403aab2d2192f2586d512426ba9",
    },
    BashHelperSpec {
        artifact_id: "bash-builtin-stubs-source",
        tag: "BSTUB",
        target: "builtins/bootstrap_stubs.c",
        digest_blake3: "7b78354611deacef7f61db89f8890ba817e351045dbccb8f502b377ba34546b3",
    },
    BashHelperSpec {
        artifact_id: "bash-readline-stub-source",
        tag: "RLSTUB",
        target: "lib/readline/readline_stub.c",
        digest_blake3: "8178ce89f526ed0d738b7d0d4b7ae946e75949c4278713342e68ed969b6644ca",
    },
];

pub(crate) const CORE_SOURCES: [&str; BASH_CORE_COMPILE_COUNT as usize] = [
    "shell",
    "eval",
    "y.tab",
    "general",
    "make_cmd",
    "print_cmd",
    "dispose_cmd",
    "execute_cmd",
    "variables",
    "copy_cmd",
    "error",
    "expr",
    "flags",
    "nojobs",
    "subst",
    "hashcmd",
    "hashlib",
    "mailcheck",
    "trap",
    "input",
    "unwind_prot",
    "pathexp",
    "sig",
    "test",
    "version",
    "alias",
    "array",
    "arrayfunc",
    "braces",
    "bracecomp",
    "bashhist",
    "bashline",
    "list",
    "stringlib",
    "locale",
    "findcmd",
    "redir",
    "pcomplete",
    "pcomplib",
    "syntax",
    "xmalloc",
    "siglist",
];
pub(crate) const GLOB_SOURCES: [&str; BASH_GLOB_COMPILE_COUNT as usize] =
    ["lib/glob/glob", "lib/glob/strmatch", "lib/glob/smatch"];
pub(crate) const SH_LIBRARY_SOURCES: [&str; BASH_SH_LIBRARY_COMPILE_COUNT as usize] = [
    "lib/sh/clktck",
    "lib/sh/getcwd",
    "lib/sh/getenv",
    "lib/sh/oslib",
    "lib/sh/setlinebuf",
    "lib/sh/strcasecmp",
    "lib/sh/strerror",
    "lib/sh/strtod",
    "lib/sh/vprint",
    "lib/sh/itos",
    "lib/sh/rename",
    "lib/sh/zread",
    "lib/sh/zwrite",
    "lib/sh/shtty",
    "lib/sh/inet_aton",
    "lib/sh/netopen",
    "lib/sh/timeval",
    "lib/sh/clock",
    "lib/sh/makepath",
    "lib/sh/pathcanon",
    "lib/sh/pathphys",
    "lib/sh/stringlist",
    "lib/sh/stringvec",
    "lib/sh/tmpfile",
    "lib/sh/spell",
    "lib/sh/strtrans",
    "lib/sh/strindex",
    "lib/sh/shquote",
    "lib/sh/mailstat",
    "lib/sh/fmtulong",
    "lib/sh/fmtullong",
    "lib/sh/strtoll",
    "lib/sh/strtoull",
    "lib/sh/strtoimax",
    "lib/sh/strtoumax",
    "lib/sh/fmtumax",
    "lib/sh/netconn",
    "lib/sh/xstrchr",
    "lib/sh/zcatfd",
];
const BUILTIN_SOURCES: [&str; BASH_BUILTIN_COMPILE_COUNT as usize] = [
    "builtins/common",
    "builtins/evalstring",
    "builtins/evalfile",
    "builtins/bashgetopt",
    "builtins/getopt",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BashExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const BASH_EXPECTED_OUTPUTS: [BashExpectedOutput; BASH_OUTPUT_COUNT] = [
    BashExpectedOutput {
        artifact_id: "bash-2.05b",
        digest_blake3: BASH_FINAL_BLAKE3,
    },
    BashExpectedOutput {
        artifact_id: "sh-2.05b",
        digest_blake3: BASH_FINAL_BLAKE3,
    },
    BashExpectedOutput {
        artifact_id: "bash-smoke-output",
        digest_blake3: BASH_SMOKE_OUTPUT_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BashSourceMaterializationReport {
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
pub(crate) struct BashOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BashInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub helper_count: u32,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<BashOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BashInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexBashError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexBashError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "bash source record was not found"),
            Self::Materialization(message) => write!(formatter, "bash materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "bash runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexBashError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexBashError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexBashError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> Vec<(&'static str, &'static str)> {
    let mut artifacts = Vec::with_capacity(BASH_SOURCE_ARTIFACT_COUNT);
    artifacts.push((BASH_SOURCE_ARTIFACT_ID, BASH_SOURCE_CONTENT_BLAKE3));
    artifacts.push((BASH_RECIPE_SOURCE_ARTIFACT_ID, BASH_RECIPE_SOURCE_BLAKE3));
    artifacts.extend(BASH_HELPERS.iter().map(|helper| (helper.artifact_id, helper.digest_blake3)));
    assert_eq!(artifacts.len(), BASH_SOURCE_ARTIFACT_COUNT);
    assert!(artifacts.iter().all(|(_, digest)| !digest.is_empty()));
    artifacts
}

pub(crate) fn materialize_authenticated_bash_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<BashSourceMaterializationReport, StagexBashError> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexBashError::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexBashError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexBashError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexBashError::Materialization(format!("creating bash source scratch: {error}")))?;
    let output_path = scratch_dir.join(BASH_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexBashError::Materialization(format!("materializing bash source {}: {error}", record.identity))
    })?;
    if !output_path.join("shell.c").is_file() || !output_path.join("execute_cmd.c").is_file() {
        return Err(StagexBashError::Materialization("materialized bash tree lacks required source files".to_string()));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, BASH_SOURCE_CONTENT_BLAKE3);
    Ok(BashSourceMaterializationReport {
        format: BASH_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: BASH_SOURCE_ARTIFACT_ID,
        record_name: BASH_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: BASH_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_bash_inventory(request: BashInventoryRequest<'_>) -> Result<BashInventoryReport, StagexBashError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBashError::Materialization(format!("creating bash scratch: {error}")))?;
    let source_root = request.scratch_dir.join(BASH_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    materialize_helpers(&source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3()?;
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexBashError::Materialization(format!("creating bash output: {error}")))?;
    let bash = build_bash(&request, &source_root, &output_root)?;
    let smoke_output = run_bash_smokes(&request, &source_root, &bash)?;
    let sh = create_sh_alias(&bash, &output_root)?;
    let outputs = collect_outputs(&bash, &sh, &smoke_output)?;
    validate_expected_outputs(&outputs)?;
    let report = BashInventoryReport {
        format: BASH_REPORT_FORMAT,
        configured_source_digest_blake3,
        helper_count: u32::try_from(BASH_HELPER_COUNT).unwrap(),
        source_compile_count: BASH_SOURCE_COMPILE_COUNT,
        build_command_count: BASH_BUILD_COMMAND_COUNT,
        smoke_command_count: BASH_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: BASH_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("bash-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| StagexBashError::Materialization(format!("serializing bash report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), BASH_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &BashInventoryRequest<'_>) -> Result<(), StagexBashError> {
    if request.scratch_dir.exists() {
        return Err(StagexBashError::Materialization(format!(
            "create-new bash scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("bash source", request.source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexBashError::Materialization(format!(
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
    validate_bound_bytes(BASH_RECIPE_SOURCE, BASH_RECIPE_SOURCE_BLAKE3, "bash recipe source")?;
    validate_helper_digests()?;
    assert!(request.source_root.join("shell.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn materialize_helpers(source_root: &Path) -> Result<(), StagexBashError> {
    for helper in BASH_HELPERS {
        let bytes = extract_heredoc(BASH_RECIPE_SOURCE, helper.tag)?;
        let target = source_root.join(helper.target);
        if target.exists() {
            fs::remove_file(&target).map_err(|error| {
                StagexBashError::Materialization(format!(
                    "removing declared bash helper target {}: {error}",
                    target.display()
                ))
            })?;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                StagexBashError::Materialization(format!("creating bash helper parent {}: {error}", parent.display()))
            })?;
        }
        crate::stagex_mes_lib::write_create_new(&target, &bytes)?;
    }
    assert!(source_root.join("config.h").is_file());
    assert!(source_root.join("builtins/bootstrap_stubs.c").is_file());
    Ok(())
}

fn extract_heredoc(source: &[u8], tag: &str) -> Result<Vec<u8>, StagexBashError> {
    let text = std::str::from_utf8(source)
        .map_err(|error| StagexBashError::Materialization(format!("bash recipe is not UTF-8: {error}")))?;
    let marker = format!("<< '{tag}'\n");
    let starts = text.match_indices(&marker).map(|(index, _)| index).collect::<Vec<_>>();
    if starts.len() != 1 {
        return Err(StagexBashError::Materialization(format!("bash helper tag {tag} occurs {} times", starts.len())));
    }
    let start = starts[0]
        .checked_add(marker.len())
        .ok_or_else(|| StagexBashError::Materialization(format!("bash helper tag {tag} start overflow")))?;
    let terminator = format!("\n{tag}\n");
    let relative_end = text[start..]
        .find(&terminator)
        .ok_or_else(|| StagexBashError::Materialization(format!("bash helper tag {tag} lacks a terminator")))?;
    let end = start
        .checked_add(relative_end)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| StagexBashError::Materialization(format!("bash helper tag {tag} end overflow")))?;
    let bytes = source[start..end].to_vec();
    if bytes.is_empty() {
        return Err(StagexBashError::Materialization(format!("bash helper tag {tag} is empty")));
    }
    assert!(end <= source.len());
    assert!(bytes.ends_with(b"\n"));
    Ok(bytes)
}

fn validate_helper_digests() -> Result<(), StagexBashError> {
    let mut mismatches = Vec::new();
    for helper in BASH_HELPERS {
        let bytes = extract_heredoc(BASH_RECIPE_SOURCE, helper.tag)?;
        let observed = blake3::hash(&bytes).to_hex().to_string();
        if observed != helper.digest_blake3 {
            mismatches.push(format!("{}={observed}", helper.artifact_id));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexBashError::Materialization(format!(
            "bash helper BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(BASH_HELPERS.len(), BASH_HELPER_COUNT);
    assert!(BASH_HELPERS.iter().all(|helper| helper.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn build_bash(
    request: &BashInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexBashError> {
    let compiler = request.tinycc27_root.join("bin/tcc");
    let mut objects = Vec::with_capacity(usize::try_from(BASH_SOURCE_COMPILE_COUNT).unwrap());
    compile_source_group(request, &compiler, source_root, &mut objects, "core", &CORE_SOURCES)?;
    compile_source_group(request, &compiler, source_root, &mut objects, "generated", &[
        "builtins/bootstrap_stubs",
        "lib/readline/readline_stub",
    ])?;
    compile_source_group(request, &compiler, source_root, &mut objects, "glob", &GLOB_SOURCES)?;
    compile_source_group(request, &compiler, source_root, &mut objects, "tilde", &["lib/tilde/tilde"])?;
    compile_source_group(request, &compiler, source_root, &mut objects, "sh", &SH_LIBRARY_SOURCES)?;
    compile_source_group(request, &compiler, source_root, &mut objects, "builtin", &BUILTIN_SOURCES)?;
    let bash = output_root.join("bash");
    link_bash(request, &compiler, source_root, &bash, &objects)?;
    crate::stagex_tinycc::set_owner_executable(&bash)?;
    validate_nonempty_file(&bash, "bash 2.05b")?;
    assert_eq!(objects.len(), usize::try_from(BASH_SOURCE_COMPILE_COUNT).unwrap());
    assert!(bash.is_file());
    Ok(bash)
}

fn compile_source_group(
    request: &BashInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    objects: &mut Vec<String>,
    group: &str,
    sources: &[&str],
) -> Result<(), StagexBashError> {
    for (source_index, source_name) in sources.iter().enumerate() {
        let object = format!("bash-{group}-{source_index:02}.o");
        let mut args = compiler_prefix(request.tinycc27_root)?;
        args.extend([
            "-I.".to_string(),
            "-Iinclude".to_string(),
            "-Ilib".to_string(),
            "-Ibuiltins".to_string(),
            "-DHAVE_CONFIG_H".to_string(),
            "-c".to_string(),
            format!("{source_name}.c"),
            "-o".to_string(),
            object.clone(),
        ]);
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("bash-compile-{group}-{source_index:02}.stderr.txt")),
        )?;
        validate_nonempty_file(&source_root.join(&object), &object)?;
        objects.push(object);
    }
    assert!(!sources.is_empty());
    assert!(objects.len() <= usize::try_from(BASH_SOURCE_COMPILE_COUNT).unwrap());
    Ok(())
}

fn link_bash(
    request: &BashInventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
    output: &Path,
    objects: &[String],
) -> Result<(), StagexBashError> {
    let libdir = request.tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let mut args = vec![
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC runtime")?.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "bash output")?.to_string(),
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
        &request.scratch_dir.join("bash-link.stderr.txt"),
    )?;
    assert_eq!(objects.len(), usize::try_from(BASH_SOURCE_COMPILE_COUNT).unwrap());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(())
}

fn compiler_prefix(tinycc27_root: &Path) -> Result<Vec<String>, StagexBashError> {
    Ok(vec![
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tinycc27_root.join("include/mes"), "TinyCC include")?.to_string(),
    ])
}

fn run_bash_smokes(
    request: &BashInventoryRequest<'_>,
    source_root: &Path,
    bash: &Path,
) -> Result<PathBuf, StagexBashError> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexBashError::Materialization(format!("creating bash smoke root: {error}")))?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        bash,
        &["--version"],
        &smoke_root,
        &empty_env,
        &request.scratch_dir.join("bash-version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        bash,
        &["-c", "echo bash-ok > positive.txt"],
        &smoke_root,
        &empty_env,
        &request.scratch_dir.join("bash-positive.stderr.txt"),
    )?;
    require_expected_process_failure(
        bash,
        &["-c", "if"],
        &smoke_root,
        &request.scratch_dir.join("bash-negative.stderr.txt"),
    )?;
    let output = smoke_root.join("positive.txt");
    let bytes = fs::read(&output)
        .map_err(|error| StagexBashError::Materialization(format!("reading bash smoke output: {error}")))?;
    if bytes != BASH_SMOKE_OUTPUT {
        return Err(StagexBashError::Materialization("bash smoke output mismatch".to_string()));
    }
    assert_eq!(bytes, BASH_SMOKE_OUTPUT);
    assert!(output.is_file());
    Ok(output)
}

fn require_expected_process_failure(
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), StagexBashError> {
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
        Ok(()) => Err(StagexBashError::Materialization("bash accepted malformed command".to_string())),
        Err(error) => Err(StagexBashError::Runtime(error)),
    }
}

fn create_sh_alias(bash: &Path, output_root: &Path) -> Result<PathBuf, StagexBashError> {
    let sh = output_root.join("sh");
    fs::copy(bash, &sh).map_err(|error| StagexBashError::Materialization(format!("copying bash sh alias: {error}")))?;
    crate::stagex_tinycc::set_owner_executable(&sh)?;
    validate_nonempty_file(&sh, "bash sh alias")?;
    assert!(bash.is_file());
    assert!(sh.is_file());
    Ok(sh)
}

fn collect_outputs(bash: &Path, sh: &Path, smoke: &Path) -> Result<Vec<BashOutputReport>, StagexBashError> {
    let mut outputs = Vec::with_capacity(BASH_OUTPUT_COUNT);
    for (artifact_id, path) in [("bash-2.05b", bash), ("sh-2.05b", sh), ("bash-smoke-output", smoke)] {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexBashError::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(BashOutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), BASH_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[BashOutputReport]) -> Result<(), StagexBashError> {
    let mut mismatches = Vec::new();
    for expected in BASH_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexBashError::Materialization(format!("bash output {} is missing", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, observed.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(StagexBashError::Materialization(format!(
            "bash output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), BASH_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexBashError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(BASH_RECORD_NAME))
        .ok_or(StagexBashError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexBashError> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != BASH_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexBashError::Materialization(format!(
            "bash source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexBashError::Materialization(
            "bash source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, BASH_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

fn configured_source_digest_blake3() -> Result<String, StagexBashError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-bash-configured-source-v1\0");
    hasher.update(BASH_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(BASH_RECIPE_SOURCE_BLAKE3.as_bytes());
    for helper in BASH_HELPERS {
        hasher.update(&extract_heredoc(BASH_RECIPE_SOURCE, helper.tag)?);
        hasher.update(b"\0");
    }
    for source in CORE_SOURCES
        .iter()
        .chain(GLOB_SOURCES.iter())
        .chain(SH_LIBRARY_SOURCES.iter())
        .chain(BUILTIN_SOURCES.iter())
    {
        hasher.update(source.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(b"builtins/bootstrap_stubs\0lib/readline/readline_stub\0lib/tilde/tilde\0");
    hasher.update(b"-I.\0-Iinclude\0-Ilib\0-Ibuiltins\0-DHAVE_CONFIG_H\0-static\0-nostdlib\0");
    Ok(hasher.finalize().to_hex().to_string())
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), StagexBashError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(StagexBashError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexBashError> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexBashError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexBashError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexBashError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > BASH_FILE_BYTES_MAX {
        return Err(StagexBashError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    let bytes =
        fs::read(path).map_err(|error| StagexBashError::Materialization(format!("reading {label}: {error}")))?;
    assert_eq!(u64::try_from(bytes.len()).unwrap(), metadata.len());
    assert!(!bytes.is_empty());
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexBashError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexBashError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > BASH_FILE_BYTES_MAX {
        return Err(StagexBashError::Materialization(format!(
            "{label} is not a bounded non-empty file: {} bytes",
            metadata.len()
        )));
    }
    assert!(path.is_file());
    assert!(metadata.len() <= BASH_FILE_BYTES_MAX);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_BASH_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_BASH_BUILD_SCRATCH";

    #[test]
    fn helper_digests_are_bound() {
        validate_helper_digests().unwrap();
        assert_eq!(BASH_HELPERS.len(), BASH_HELPER_COUNT);
        assert!(BASH_HELPERS.iter().all(|helper| !helper.target.is_empty()));
    }

    #[test]
    fn configured_source_digest_is_stable() {
        let digest = configured_source_digest_blake3().unwrap();
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert_eq!(digest, BASH_CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn unknown_helper_tag_is_rejected() {
        let error = extract_heredoc(BASH_RECIPE_SOURCE, "MISSING").unwrap_err();
        assert!(error.to_string().contains("occurs 0 times"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn source_lists_are_bounded_and_complete() {
        assert_eq!(CORE_SOURCES.len(), usize::try_from(BASH_CORE_COMPILE_COUNT).unwrap());
        assert_eq!(SH_LIBRARY_SOURCES.len(), usize::try_from(BASH_SH_LIBRARY_COMPILE_COUNT).unwrap());
        assert_eq!(BASH_SOURCE_COMPILE_COUNT, 92);
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_bash_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let expected_manifest = read_source_bundle(&bundle).unwrap().manifest_blake3;
        let report = materialize_authenticated_bash_source(&bundle, &expected_manifest, &scratch).unwrap();
        assert_eq!(report.record_content_blake3, BASH_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("shell.c").is_file());
    }

    #[test]
    #[ignore = "requires retained bash source and TinyCC 0.9.27"]
    fn derives_retained_bash_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tinycc27_root = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_bash_inventory(BashInventoryRequest {
            source_root: &source_root,
            tinycc27_root: &tinycc27_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, BASH_BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
