use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
const M4_RECORD_NAME: &str = "m4-1.4.7-src";
const M4_SOURCE_OUTPUT_NAME: &str = "m4-1.4.7";
pub(crate) const M4_SOURCE_ARTIFACT_ID: &str = "m4-1.4.7-source";
pub(crate) const M4_SOURCE_CONTENT_BLAKE3: &str = "1acf045348ef14f9fb38dc1e23fec5146d32b4fb6b7bb5c4dfb9742da12ce0d1";
const M4_RECIPE: &[u8] = include_bytes!("../bootstrap/m4-1.4.7-musl.ncl");
pub(crate) const M4_RECIPE_ARTIFACT_ID: &str = "m4-1.4.7-recipe-source";
pub(crate) const M4_RECIPE_BLAKE3: &str = "39671a43fce58c3cae66c38810b1912b27a29f7a5c134c84acf03b678af22e74";
const M4_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-m4-source-materialization-v1";
const M4_REPORT_FORMAT: &str = "mantle-stagex-m4-1.4.7-inventory-v1";
const M4_SOURCE_NON_CLAIM: &str =
    "GNU M4 source materialization proves authenticated offline archive identity and fixed-output parity only";
const M4_NON_CLAIM: &str = "this inventory binds GNU M4 1.4.7 and bounded macro observations only; it does not prove the later parser generators, binutils, native TinyCC, or provider admission";
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const M4_FILE_MEBIBYTES_MAX: u64 = 64;
const M4_FILE_BYTES_MAX: u64 = M4_FILE_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const M4_INVOCATION_ARG_COUNT_MAX: usize = 2;
const M4_DISABLED_BUILTIN_COUNT: usize = 2;
const M4_DISABLED_BUILTINS: [&str; M4_DISABLED_BUILTIN_COUNT] = ["esyscmd", "syscmd"];
const M4_LIB_COMPILE_COUNT: u32 = 18;
const M4_APP_COMPILE_COUNT: u32 = 11;
const M4_SOURCE_COMPILE_COUNT: u32 = M4_LIB_COMPILE_COUNT + M4_APP_COMPILE_COUNT;
const M4_BUILD_COMMAND_COUNT: u32 = M4_SOURCE_COMPILE_COUNT + 1;
const M4_SMOKE_COMMAND_COUNT: u32 = 5;
const M4_OUTPUT_COUNT: usize = 5;
const M4_SOURCE_ARTIFACT_COUNT: usize = 2;
const M4_OBJECT_MODE: u32 = 0o644;
const M4_PERMISSION_MODE_MASK: u32 = 0o7777;
const M4_CONFIG_H: &[u8] = br#"#define VERSION "1.4.7"
#define PACKAGE_BUGREPORT "bug-m4@gnu.org"
#define PACKAGE_STRING "GNU M4 1.4.7"
#define PACKAGE "m4"
#define PACKAGE_NAME "GNU M4"
#define HAVE_STDINT_H 1
#define HAVE___FPENDING 1
#define HAVE_DECL___FPENDING 1
#ifndef SIZE_MAX
#define SIZE_MAX ((size_t)-1)
#endif
#define _GNU_SOURCE 1
#define _GL_UNUSED
#define __getopt_argv_const const
#define SYSCMD_SHELL "/__mantle_stagex_ambient_shell_disabled__"
"#;
const M4_COMMON_FLAGS: [&str; 6] = [
    "-Ilib",
    "-Isrc",
    "-DHAVE_CONFIG_H=1",
    "-DSIZE_MAX=~(size_t)0",
    "-c",
    "-o",
];
const M4_LIB_SOURCES: [&str; M4_LIB_COMPILE_COUNT as usize] = [
    "cloexec",
    "close-stream",
    "dup-safer",
    "error",
    "exitfail",
    "fd-safer",
    "fopen-safer",
    "getopt",
    "getopt1",
    "mkstemp-safer",
    "regex",
    "obstack",
    "tmpfile-safer",
    "verror",
    "xalloc-die",
    "xasprintf",
    "xmalloc",
    "xvasprintf",
];
const M4_APP_SOURCES: [&str; M4_APP_COMPILE_COUNT as usize] = [
    "m4", "builtin", "debug", "eval", "format", "freeze", "input", "macro", "output", "path", "symtab",
];
const M4_MACRO_INPUT: &[u8] =
    b"define(`mantle_value', `42')mantle_value\nifdef(`syscmd',`ambient-enabled',`ambient-disabled')\n";
const M4_MACRO_EXPECTED: &[u8] = b"42\nambient-disabled\n";
const M4_PREFIX_INPUT: &[u8] = b"m4_define(`mantle_prefixed_value', `prefixed-ok')mantle_prefixed_value\n";
const M4_PREFIX_EXPECTED: &[u8] = b"prefixed-ok\n";
const M4_PREFIX_NEGATIVE_INPUT: &[u8] =
    b"define(`mantle_unprefixed_value', `must-not-expand')mantle_unprefixed_value\n";
const M4_PREFIX_NEGATIVE_FORBIDDEN: &[u8] = b"must-not-expand\n";
const M4_MALFORMED_INPUT: &[u8] = b"define(`unterminated, `value')\n";
const M4_REJECTION_RECEIPT: &[u8] =
    b"{\"format\":\"mantle-stagex-m4-rejection-v1\",\"malformed_input_rejected\":true}\n";
pub(crate) const M4_CONFIGURED_SOURCE_BLAKE3: &str = "dbd75c146b108dfe62f23aeb29675b68d0d41bc8ca5b5dfd137d346a284376ac";
pub(crate) const M4_FINAL_BLAKE3: &str = "3dfd2a1223ff6e0c2c540bc2507a097948ab24e2465f232e2363944e5b704741";
const M4_VERSION_BLAKE3: &str = "c9b69698f54279bd5ce4b9cba7442ba625e7ba92717ee139d3152093331c0e75";
const M4_MACRO_BLAKE3: &str = "f737ffa0d1bffb14295a97110f43d0f9cf059d5cbffc7ff6187b2182250d34ca";
const M4_PREFIX_BLAKE3: &str = "6277dd4b0cbd4b080f3a4c4e561d05f128ffee445dadefa2116cc8e3b079e26c";
const M4_REJECTION_BLAKE3: &str = "675bd6ec5c382c78909a394f1f5a59e8b51e7e617f9ad9537e466fd799553420";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct M4ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const M4_EXPECTED_OUTPUTS: [M4ExpectedOutput; M4_OUTPUT_COUNT] = [
    M4ExpectedOutput {
        artifact_id: "m4-1.4.7",
        digest_blake3: M4_FINAL_BLAKE3,
    },
    M4ExpectedOutput {
        artifact_id: "m4-version-observation",
        digest_blake3: M4_VERSION_BLAKE3,
    },
    M4ExpectedOutput {
        artifact_id: "m4-macro-observation",
        digest_blake3: M4_MACRO_BLAKE3,
    },
    M4ExpectedOutput {
        artifact_id: "m4-prefix-observation",
        digest_blake3: M4_PREFIX_BLAKE3,
    },
    M4ExpectedOutput {
        artifact_id: "m4-malformed-rejection",
        digest_blake3: M4_REJECTION_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct M4SourceMaterializationReport {
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
pub(crate) struct M4OutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct M4InventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<M4OutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct M4InventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexM4Error {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexM4Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU M4 source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU M4 materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU M4 runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexM4Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexM4Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for StagexM4Error {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); M4_SOURCE_ARTIFACT_COUNT] {
    [
        (M4_SOURCE_ARTIFACT_ID, M4_SOURCE_CONTENT_BLAKE3),
        (M4_RECIPE_ARTIFACT_ID, M4_RECIPE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_m4_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<M4SourceMaterializationReport, StagexM4Error> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexM4Error::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexM4Error::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexM4Error::Materialization(format!("creating GNU M4 source scratch: {error}")))?;
    let output_path = scratch_dir.join(M4_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        StagexM4Error::Materialization(format!("materializing GNU M4 source {}: {error}", record.identity))
    })?;
    validate_materialized_source(&output_path)?;
    Ok(M4SourceMaterializationReport {
        format: M4_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: M4_SOURCE_ARTIFACT_ID,
        record_name: M4_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: M4_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_m4_inventory(request: M4InventoryRequest<'_>) -> Result<M4InventoryReport, StagexM4Error> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexM4Error::Materialization(format!("creating GNU M4 scratch: {error}")))?;
    let source_root = request.scratch_dir.join(M4_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    fs::write(source_root.join("lib/config.h"), M4_CONFIG_H)
        .map_err(|error| StagexM4Error::Materialization(format!("writing GNU M4 config: {error}")))?;
    disable_ambient_shell_builtins(&source_root)?;
    validate_compile_source_set(&source_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_root)
        .map_err(|error| StagexM4Error::Materialization(format!("creating GNU M4 output: {error}")))?;
    let m4 = build_m4(&request, &source_root, &output_root)?;
    let observations = run_m4_smokes(&request, &source_root, &m4)?;
    let outputs = collect_outputs(&m4, &observations)?;
    validate_expected_outputs(&outputs)?;
    let report = build_inventory_report(&request, configured_source_digest_blake3, outputs);
    write_inventory(request.scratch_dir, &report)?;
    assert_eq!(report.outputs.len(), M4_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn build_inventory_report(
    request: &M4InventoryRequest<'_>,
    configured_source_digest_blake3: String,
    outputs: Vec<M4OutputReport>,
) -> M4InventoryReport {
    let report = M4InventoryReport {
        format: M4_REPORT_FORMAT,
        configured_source_digest_blake3,
        source_compile_count: M4_SOURCE_COMPILE_COUNT,
        build_command_count: M4_BUILD_COMMAND_COUNT,
        smoke_command_count: M4_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: M4_NON_CLAIM,
    };
    assert_eq!(report.source_compile_count, M4_SOURCE_COMPILE_COUNT);
    assert_eq!(report.build_command_count, M4_BUILD_COMMAND_COUNT);
    report
}

fn write_inventory(scratch_dir: &Path, report: &M4InventoryReport) -> Result<(), StagexM4Error> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| StagexM4Error::Materialization(format!("serializing GNU M4 report: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&scratch_dir.join("m4-inventory.json"), &bytes)?;
    assert!(!bytes.is_empty());
    assert!(scratch_dir.join("m4-inventory.json").is_file());
    Ok(())
}

fn validate_bundle_path(bundle_path: &Path) -> Result<(), StagexM4Error> {
    if !bundle_path.is_absolute() || !bundle_path.is_file() {
        return Err(StagexM4Error::Materialization(format!(
            "source bundle is not an absolute file: {}",
            bundle_path.display()
        )));
    }
    assert!(bundle_path.is_absolute());
    assert!(bundle_path.is_file());
    Ok(())
}

fn validate_materialized_source(output_path: &Path) -> Result<(), StagexM4Error> {
    for relative in ["lib/cloexec.c", "src/m4.c", "src/builtin.c"] {
        if !output_path.join(relative).is_file() {
            return Err(StagexM4Error::Materialization(format!(
                "materialized GNU M4 tree lacks required source {relative}"
            )));
        }
    }
    assert!(output_path.is_absolute());
    assert!(output_path.join("lib").is_dir());
    Ok(())
}

fn validate_inventory_inputs(request: &M4InventoryRequest<'_>) -> Result<(), StagexM4Error> {
    if request.scratch_dir.exists() {
        return Err(StagexM4Error::Materialization(format!(
            "create-new GNU M4 scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("GNU M4 source", request.source_root),
        ("TinyCC musl-v2", request.tcc_musl_v2_root),
        ("native musl", request.musl_native_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(StagexM4Error::Materialization(format!(
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
    Ok(())
}

fn validate_native_musl_inputs(root: &Path) -> Result<(), StagexM4Error> {
    for (artifact_id, relative) in [("musl-native-crt1", "lib/crt1.o"), ("musl-native-libc", "lib/libc.a")] {
        let expected = crate::stagex_musl_native::EXPECTED_OUTPUTS
            .iter()
            .find(|output| output.artifact_id == artifact_id)
            .expect("native musl input identity is fixed")
            .digest_blake3;
        validate_file_digest(&root.join(relative), expected, artifact_id)?;
    }
    assert!(root.join("include").is_dir());
    assert!(root.join("lib/libc.a").is_file());
    Ok(())
}

fn validate_recipe_digest() -> Result<(), StagexM4Error> {
    let observed = blake3::hash(M4_RECIPE).to_hex().to_string();
    if observed != M4_RECIPE_BLAKE3 {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 recipe BLAKE3 mismatch: expected {M4_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!M4_RECIPE.is_empty());
    Ok(())
}

fn disable_ambient_shell_builtins(source_root: &Path) -> Result<(), StagexM4Error> {
    let builtin_path = source_root.join("src/builtin.c");
    let source = fs::read_to_string(&builtin_path)
        .map_err(|error| StagexM4Error::Materialization(format!("reading GNU M4 builtin source: {error}")))?;
    let rewritten = disable_ambient_shell_builtins_text(&source)?;
    fs::write(&builtin_path, rewritten)
        .map_err(|error| StagexM4Error::Materialization(format!("writing GNU M4 builtin source: {error}")))?;
    let installed = fs::read_to_string(&builtin_path)
        .map_err(|error| StagexM4Error::Materialization(format!("reading rewritten GNU M4 builtin source: {error}")))?;
    assert!(builtin_path.is_file());
    assert!(M4_DISABLED_BUILTINS.iter().all(|name| !has_builtin_table_entry(&installed, name)));
    Ok(())
}

fn disable_ambient_shell_builtins_text(source: &str) -> Result<String, StagexM4Error> {
    for name in M4_DISABLED_BUILTINS {
        let match_count = source.lines().filter(|line| has_builtin_table_entry(line, name)).count();
        if match_count != 1 {
            return Err(StagexM4Error::Materialization(format!(
                "GNU M4 builtin table entry {name} occurs {match_count} times, expected once"
            )));
        }
    }
    let mut rewritten = source
        .lines()
        .filter(|line| !M4_DISABLED_BUILTINS.iter().any(|name| has_builtin_table_entry(line, name)))
        .collect::<Vec<_>>()
        .join("\n");
    rewritten.push('\n');
    assert!(M4_DISABLED_BUILTINS.iter().all(|name| !has_builtin_table_entry(&rewritten, name)));
    assert!(!rewritten.is_empty());
    Ok(rewritten)
}

fn has_builtin_table_entry(line: &str, name: &str) -> bool {
    let marker = format!("{{ \"{name}\",");
    assert!(!name.is_empty());
    assert!(!marker.is_empty());
    line.contains(&marker)
}

fn validate_compile_source_set(source_root: &Path) -> Result<(), StagexM4Error> {
    let paths = compile_source_paths();
    let unique = paths.iter().collect::<BTreeSet<_>>();
    if paths.len() != usize::try_from(M4_SOURCE_COMPILE_COUNT).unwrap() || unique.len() != paths.len() {
        return Err(StagexM4Error::Materialization(
            "GNU M4 compile source set is incomplete or contains duplicates".to_string(),
        ));
    }
    for relative in &paths {
        if !source_root.join(relative).is_file() {
            return Err(StagexM4Error::Materialization(format!("GNU M4 compile source is missing: {relative}")));
        }
    }
    assert_eq!(paths.len(), unique.len());
    assert!(paths.iter().all(|path| path.ends_with(".c")));
    Ok(())
}

fn compile_source_paths() -> Vec<String> {
    let mut paths = Vec::with_capacity(usize::try_from(M4_SOURCE_COMPILE_COUNT).unwrap());
    paths.extend(M4_LIB_SOURCES.iter().map(|name| format!("lib/{name}.c")));
    paths.extend(M4_APP_SOURCES.iter().map(|name| format!("src/{name}.c")));
    assert_eq!(paths.len(), usize::try_from(M4_SOURCE_COMPILE_COUNT).unwrap());
    assert!(!paths.is_empty());
    paths
}

fn build_m4(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, StagexM4Error> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object_paths = compile_m4_objects(request, &compiler, source_root)?;
    let output = output_root.join("m4");
    let args = link_args(request.musl_native_root, &output, &object_paths)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("m4-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    validate_nonempty_file(&output, "GNU M4 1.4.7")?;
    assert_eq!(object_paths.len(), usize::try_from(M4_SOURCE_COMPILE_COUNT).unwrap());
    assert!(output.is_file());
    Ok(output)
}

fn compile_m4_objects(
    request: &M4InventoryRequest<'_>,
    compiler: &Path,
    source_root: &Path,
) -> Result<Vec<String>, StagexM4Error> {
    let source_paths = compile_source_paths();
    let mut object_paths = Vec::with_capacity(source_paths.len());
    for (index, source) in source_paths.iter().enumerate() {
        let object = source.strip_suffix(".c").expect("source suffix is fixed").to_string() + ".o";
        let args = compile_args(request.musl_native_root, source, &object)?;
        crate::stagex_mes_lib::run_bounded_process(
            compiler,
            &args,
            source_root,
            &BTreeMap::<String, String>::new(),
            &request.scratch_dir.join(format!("m4-compile-{index:02}.stderr.txt")),
        )?;
        let object_path = source_root.join(&object);
        normalize_m4_object_mode(&object_path, source)?;
        validate_nonempty_file(&object_path, source)?;
        object_paths.push(object);
    }
    assert_eq!(object_paths.len(), source_paths.len());
    assert!(object_paths.iter().all(|path| path.ends_with(".o")));
    Ok(object_paths)
}

fn normalize_m4_object_mode(path: &Path, source: &str) -> Result<(), StagexM4Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexM4Error::Materialization(format!("reading GNU M4 object {source} metadata: {error}")))?;
    if !metadata.file_type().is_file() {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 object {source} is not a regular file: {}",
            path.display()
        )));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(M4_OBJECT_MODE)).map_err(|error| {
        StagexM4Error::Materialization(format!("setting GNU M4 object {source} permissions: {error}"))
    })?;
    let observed_mode = fs::symlink_metadata(path)
        .map_err(|error| {
            StagexM4Error::Materialization(format!("re-reading GNU M4 object {source} metadata: {error}"))
        })?
        .permissions()
        .mode()
        & M4_PERMISSION_MODE_MASK;
    if observed_mode != M4_OBJECT_MODE {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 object {source} mode mismatch: expected {M4_OBJECT_MODE:o}, observed {observed_mode:o}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed_mode, M4_OBJECT_MODE);
    Ok(())
}

fn compile_args(musl_root: &Path, source: &str, object: &str) -> Result<Vec<String>, StagexM4Error> {
    let include_path = musl_root.join("include");
    let include = crate::stagex_mes_lib::utf8_absolute(&include_path, "native musl include")?;
    let args = vec![
        "-Ilib".to_string(),
        "-Isrc".to_string(),
        format!("-I{include}"),
        "-DHAVE_CONFIG_H=1".to_string(),
        "-DSIZE_MAX=~(size_t)0".to_string(),
        "-c".to_string(),
        source.to_string(),
        "-o".to_string(),
        object.to_string(),
    ];
    assert!(source.ends_with(".c"));
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn link_args(musl_root: &Path, output: &Path, objects: &[String]) -> Result<Vec<String>, StagexM4Error> {
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "GNU M4 output")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&musl_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(crate::stagex_mes_lib::utf8_absolute(&musl_root.join("lib/libc.a"), "native musl libc")?.to_string());
    assert!(!objects.is_empty());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

#[derive(Debug)]
struct M4SmokeObservations {
    version: PathBuf,
    macro_output: PathBuf,
    prefix_output: PathBuf,
    rejection_receipt: PathBuf,
}

fn run_m4_smokes(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    m4: &Path,
) -> Result<M4SmokeObservations, StagexM4Error> {
    let smoke_root = source_root.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| StagexM4Error::Materialization(format!("creating GNU M4 smoke root: {error}")))?;
    let version = run_version_smoke(request, source_root, m4, &smoke_root)?;
    let macro_output = run_exact_smoke(request, source_root, m4, &smoke_root, MacroSmokeKind::Ordinary)?;
    let prefix_output = run_exact_smoke(request, source_root, m4, &smoke_root, MacroSmokeKind::Prefixed)?;
    run_prefix_negative_smoke(request, source_root, m4, &smoke_root)?;
    let rejection_receipt = run_malformed_smoke(request, source_root, m4, &smoke_root)?;
    assert!(version.is_file());
    assert!(rejection_receipt.is_file());
    Ok(M4SmokeObservations {
        version,
        macro_output,
        prefix_output,
        rejection_receipt,
    })
}

fn run_version_smoke(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    m4: &Path,
    smoke_root: &Path,
) -> Result<PathBuf, StagexM4Error> {
    let stdout = smoke_root.join("version.txt");
    run_capture(request, m4, &["--version"], source_root, &stdout, "version")?;
    let bytes = read_bounded_file(&stdout, "GNU M4 version output")?;
    if !bytes.windows(b"GNU M4 1.4.7".len()).any(|window| window == b"GNU M4 1.4.7") {
        return Err(StagexM4Error::Materialization("GNU M4 version output mismatch".to_string()));
    }
    assert!(!bytes.is_empty());
    assert!(stdout.is_file());
    Ok(stdout)
}

#[derive(Debug, Clone, Copy)]
enum MacroSmokeKind {
    Ordinary,
    Prefixed,
}

fn run_exact_smoke(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    m4: &Path,
    smoke_root: &Path,
    kind: MacroSmokeKind,
) -> Result<PathBuf, StagexM4Error> {
    let (name, input_bytes, expected, prefixed) = match kind {
        MacroSmokeKind::Ordinary => ("macro", M4_MACRO_INPUT, M4_MACRO_EXPECTED, false),
        MacroSmokeKind::Prefixed => ("prefix", M4_PREFIX_INPUT, M4_PREFIX_EXPECTED, true),
    };
    let input = smoke_root.join(format!("{name}.m4"));
    let output = smoke_root.join(format!("{name}.out"));
    crate::stagex_mes_lib::write_create_new(&input, input_bytes)?;
    let mut args = Vec::with_capacity(M4_INVOCATION_ARG_COUNT_MAX);
    if prefixed {
        args.push("-P");
    }
    args.push(crate::stagex_mes_lib::utf8_absolute(&input, "GNU M4 smoke input")?);
    run_capture(request, m4, &args, source_root, &output, name)?;
    require_exact_bytes(&output, expected, name)?;
    assert!(output.is_file());
    assert!(!args.is_empty());
    Ok(output)
}

fn run_prefix_negative_smoke(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    m4: &Path,
    smoke_root: &Path,
) -> Result<(), StagexM4Error> {
    let input = smoke_root.join("prefix-negative.m4");
    let output = smoke_root.join("prefix-negative.out");
    crate::stagex_mes_lib::write_create_new(&input, M4_PREFIX_NEGATIVE_INPUT)?;
    let input_arg = crate::stagex_mes_lib::utf8_absolute(&input, "GNU M4 prefix-negative input")?;
    run_capture(request, m4, &["-P", input_arg], source_root, &output, "prefix-negative")?;
    let observed = read_bounded_file(&output, "GNU M4 prefix-negative output")?;
    if observed == M4_PREFIX_NEGATIVE_FORBIDDEN {
        return Err(StagexM4Error::Materialization("GNU M4 -P accepted the unprefixed define builtin".to_string()));
    }
    assert!(!observed.is_empty());
    assert_ne!(observed, M4_PREFIX_NEGATIVE_FORBIDDEN);
    Ok(())
}

fn run_malformed_smoke(
    request: &M4InventoryRequest<'_>,
    source_root: &Path,
    m4: &Path,
    smoke_root: &Path,
) -> Result<PathBuf, StagexM4Error> {
    let input = smoke_root.join("malformed.m4");
    let output = smoke_root.join("malformed.out");
    crate::stagex_mes_lib::write_create_new(&input, M4_MALFORMED_INPUT)?;
    let input_arg = crate::stagex_mes_lib::utf8_absolute(&input, "GNU M4 malformed input")?;
    let result = run_capture(request, m4, &[input_arg], source_root, &output, "malformed");
    require_rejected_process(result)?;
    let receipt = smoke_root.join("malformed-rejection.json");
    crate::stagex_mes_lib::write_create_new(&receipt, M4_REJECTION_RECEIPT)?;
    assert!(receipt.is_file());
    assert_eq!(fs::read(&receipt).unwrap_or_default(), M4_REJECTION_RECEIPT);
    Ok(receipt)
}

fn run_capture<S: AsRef<std::ffi::OsStr>>(
    request: &M4InventoryRequest<'_>,
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    stdout: &Path,
    label: &str,
) -> Result<(), StagexM4Error> {
    crate::stagex_mes_lib::run_bounded_process_capturing_stdout(
        executable,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        stdout,
        M4_FILE_BYTES_MAX,
        &request.scratch_dir.join(format!("m4-{label}.stderr.txt")),
    )?;
    assert!(executable.is_file());
    assert!(stdout.is_file());
    Ok(())
}

fn require_rejected_process(result: Result<(), StagexM4Error>) -> Result<(), StagexM4Error> {
    match result {
        Err(StagexM4Error::Runtime(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(exit_code),
            stderr,
            ..
        })) if exit_code != 0 && !stderr.trim().is_empty() => {
            assert_ne!(exit_code, 0);
            assert!(!stderr.trim().is_empty());
            Ok(())
        }
        Ok(()) => Err(StagexM4Error::Materialization("GNU M4 accepted malformed quoted input".to_string())),
        Err(error) => Err(error),
    }
}

fn collect_outputs(m4: &Path, observations: &M4SmokeObservations) -> Result<Vec<M4OutputReport>, StagexM4Error> {
    let specs = [
        ("m4-1.4.7", m4),
        ("m4-version-observation", observations.version.as_path()),
        ("m4-macro-observation", observations.macro_output.as_path()),
        ("m4-prefix-observation", observations.prefix_output.as_path()),
        ("m4-malformed-rejection", observations.rejection_receipt.as_path()),
    ];
    let mut outputs = Vec::with_capacity(M4_OUTPUT_COUNT);
    for (artifact_id, path) in specs {
        let metadata = fs::metadata(path)
            .map_err(|error| StagexM4Error::Materialization(format!("reading {artifact_id} metadata: {error}")))?;
        outputs.push(M4OutputReport {
            artifact_id: artifact_id.to_string(),
            path: path.to_path_buf(),
            bytes_len: metadata.len(),
            digest_blake3: blake3_file_hex(path, artifact_id)?,
        });
    }
    assert_eq!(outputs.len(), M4_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[M4OutputReport]) -> Result<(), StagexM4Error> {
    let observed = outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), output.digest_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mismatches = M4_EXPECTED_OUTPUTS
        .iter()
        .filter_map(|expected| {
            let actual = observed.get(expected.artifact_id).copied().unwrap_or("missing");
            (actual != expected.digest_blake3).then(|| format!("{}={actual}", expected.artifact_id))
        })
        .collect::<Vec<_>>();
    if !mismatches.is_empty() {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 output BLAKE3 mismatches: {}",
            mismatches.join(", ")
        )));
    }
    assert_eq!(outputs.len(), M4_EXPECTED_OUTPUTS.len());
    assert!(mismatches.is_empty());
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexM4Error> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(M4_RECORD_NAME))
        .ok_or(StagexM4Error::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexM4Error> {
    if record.kind != SourceRecordKind::FixedUrl || record.content_blake3 != M4_SOURCE_CONTENT_BLAKE3 {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 source record {} has substituted kind or content identity",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexM4Error::Materialization(
            "GNU M4 source record must contain exactly one archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.content_blake3, M4_SOURCE_CONTENT_BLAKE3);
    Ok(())
}

pub(crate) fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-m4-configured-source-v1\0");
    hasher.update(M4_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(M4_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(M4_CONFIG_H);
    for name in M4_DISABLED_BUILTINS {
        hasher.update(name.as_bytes());
        hasher.update(b"\0");
    }
    for value in M4_COMMON_FLAGS.iter().chain(M4_LIB_SOURCES.iter()).chain(M4_APP_SOURCES.iter()) {
        hasher.update(value.as_bytes());
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    digest
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexM4Error> {
    let observed = read_bounded_file(path, label)?;
    if observed != expected {
        return Err(StagexM4Error::Materialization(format!(
            "GNU M4 {label} output mismatch: observed {:?}",
            String::from_utf8_lossy(&observed)
        )));
    }
    assert_eq!(observed, expected);
    assert!(!observed.is_empty());
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexM4Error> {
    let observed = blake3_file_hex(path, label)?;
    if observed != expected {
        return Err(StagexM4Error::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(path.is_file());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexM4Error> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexM4Error::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > M4_FILE_BYTES_MAX {
        return Err(StagexM4Error::Materialization(format!(
            "{label} is empty, oversized, or not a file: {}",
            path.display()
        )));
    }
    assert!(metadata.len() > 0);
    assert!(metadata.len() <= M4_FILE_BYTES_MAX);
    Ok(())
}

fn read_bounded_file(path: &Path, label: &str) -> Result<Vec<u8>, StagexM4Error> {
    validate_nonempty_file(path, label)?;
    let bytes =
        fs::read(path).map_err(|error| StagexM4Error::Materialization(format!("reading {label} bytes: {error}")))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > M4_FILE_BYTES_MAX {
        return Err(StagexM4Error::Materialization(format!("{label} exceeds {M4_FILE_BYTES_MAX} bytes")));
    }
    assert!(!bytes.is_empty());
    assert!(u64::try_from(bytes.len()).unwrap() <= M4_FILE_BYTES_MAX);
    Ok(bytes)
}

fn blake3_file_hex(path: &Path, label: &str) -> Result<String, StagexM4Error> {
    let bytes = read_bounded_file(path, label)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!bytes.is_empty());
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_bundle::SourceFileEntry;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_M4_SOURCE_ROOT";
    const RETAINED_TCC_V2_ROOT_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_V2_ROOT";
    const RETAINED_MUSL_NATIVE_ROOT_ENV: &str = "MANTLE_STAGE_X_MUSL_NATIVE_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_M4_BUILD_SCRATCH";

    fn source_record(kind: SourceRecordKind, digest: &str) -> SourceRecord {
        SourceRecord {
            kind,
            identity: "fixed-url-m4-test".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([("name".to_string(), M4_RECORD_NAME.to_string())]),
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
    fn validates_exact_m4_source_record_and_recipe() {
        let record = source_record(SourceRecordKind::FixedUrl, M4_SOURCE_CONTENT_BLAKE3);
        validate_source_record(&record).unwrap();
        validate_recipe_digest().unwrap();
        assert_eq!(record.kind, SourceRecordKind::FixedUrl);
        assert_eq!(source_artifact_digests().len(), M4_SOURCE_ARTIFACT_COUNT);
    }

    #[test]
    fn rejects_substituted_m4_source_record() {
        let wrong_kind = source_record(SourceRecordKind::VcsSnapshot, M4_SOURCE_CONTENT_BLAKE3);
        let wrong_digest = source_record(SourceRecordKind::FixedUrl, &"a".repeat(BLAKE3_HEX_CHAR_COUNT));
        assert!(validate_source_record(&wrong_kind).is_err());
        assert!(validate_source_record(&wrong_digest).is_err());
    }

    #[test]
    fn compile_source_set_is_complete_and_unique() {
        let paths = compile_source_paths();
        let unique = paths.iter().collect::<BTreeSet<_>>();
        assert_eq!(paths.len(), usize::try_from(M4_SOURCE_COMPILE_COUNT).unwrap());
        assert_eq!(paths.len(), unique.len());
    }

    #[test]
    fn compile_source_presence_is_required() {
        let source_root = tempfile::tempdir().unwrap();
        let missing_relative = compile_source_paths().first().unwrap().clone();
        for relative in compile_source_paths() {
            let path = source_root.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, []).unwrap();
        }
        validate_compile_source_set(source_root.path()).unwrap();
        let missing_path = source_root.path().join(&missing_relative);
        fs::remove_file(&missing_path).unwrap();
        let error = validate_compile_source_set(source_root.path()).unwrap_err();
        assert!(error.to_string().contains("compile source is missing"));
        assert!(error.to_string().contains(&missing_relative));
    }

    #[test]
    fn normalizes_m4_object_permissions_before_linking() {
        let temp = tempfile::tempdir().unwrap();
        let object = temp.path().join("cloexec.o");
        fs::write(&object, b"object").unwrap();
        fs::set_permissions(&object, fs::Permissions::from_mode(0o1150)).unwrap();

        normalize_m4_object_mode(&object, "lib/cloexec.c").unwrap();

        let observed_mode = fs::metadata(&object).unwrap().permissions().mode() & M4_PERMISSION_MODE_MASK;
        assert_eq!(observed_mode, M4_OBJECT_MODE);
        assert_eq!(fs::read(&object).unwrap(), b"object");
    }

    #[test]
    fn rejects_non_regular_m4_object_before_mode_change() {
        let temp = tempfile::tempdir().unwrap();
        let error = normalize_m4_object_mode(temp.path(), "lib/cloexec.c").unwrap_err();
        assert!(error.to_string().contains("not a regular file"));
        assert!(temp.path().is_dir());
    }

    #[test]
    fn configuration_disables_ambient_shell_execution() {
        let config = std::str::from_utf8(M4_CONFIG_H).unwrap();
        assert!(config.contains("/__mantle_stagex_ambient_shell_disabled__"));
        assert!(!config.contains("/bin/sh"));
    }

    #[test]
    fn source_transform_removes_shell_builtins_and_rejects_shape_drift() {
        let source = "  { \"syscmd\", FALSE },\n  { \"esyscmd\", TRUE },\n  { \"eval\", FALSE },\n";
        let rewritten = disable_ambient_shell_builtins_text(source).unwrap();
        assert!(!has_builtin_table_entry(&rewritten, "syscmd"));
        assert!(!has_builtin_table_entry(&rewritten, "esyscmd"));
        assert!(has_builtin_table_entry(&rewritten, "eval"));
        let missing = disable_ambient_shell_builtins_text("  { \"syscmd\", FALSE },\n").unwrap_err();
        assert!(missing.to_string().contains("esyscmd"));
        assert!(missing.to_string().contains("expected once"));
    }

    #[test]
    fn configured_source_digest_is_bound() {
        let observed = configured_source_digest_blake3();
        assert_eq!(observed, M4_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(observed.len(), BLAKE3_HEX_CHAR_COUNT);
    }

    #[test]
    fn rejects_substituted_output_digest() {
        let mut outputs = M4_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| M4OutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_expected_outputs(&outputs).unwrap();
        let binary = outputs.iter_mut().find(|output| output.artifact_id == "m4-1.4.7").unwrap();
        binary.digest_blake3 = "f".repeat(BLAKE3_HEX_CHAR_COUNT);
        let error = validate_expected_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("m4-1.4.7"));
        assert!(!error.to_string().contains("accepted malformed"));
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_m4_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let report = materialize_authenticated_m4_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        assert_eq!(report.record_content_blake3, M4_SOURCE_CONTENT_BLAKE3);
        assert!(report.output_path.join("src/m4.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU M4 source, TinyCC musl-v2, and native musl"]
    fn derives_retained_m4_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tcc_musl_v2_root = PathBuf::from(std::env::var(RETAINED_TCC_V2_ROOT_ENV).unwrap());
        let musl_native_root = PathBuf::from(std::env::var(RETAINED_MUSL_NATIVE_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let report = derive_m4_inventory(M4InventoryRequest {
            source_root: &source_root,
            tcc_musl_v2_root: &tcc_musl_v2_root,
            musl_native_root: &musl_native_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.source_compile_count, M4_SOURCE_COMPILE_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
