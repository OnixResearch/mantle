use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const TINYCC27_RECORD_NAME: &str = "tcc-0.9.27-src";
const TINYCC27_SOURCE_OUTPUT_NAME: &str = "tinycc-0.9.27";
pub(crate) const TINYCC27_SOURCE_ARTIFACT_ID: &str = "tinycc-0.9.27-source";
pub(crate) const TINYCC27_SOURCE_CONTENT_BLAKE3: &str =
    "a3417d7e6218de60bfb3b30cab2db9fbe3e65891d9b3e765f09a4ac87539d03d";
const TINYCC27_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-tinycc27-source-materialization-v1";
const TINYCC27_SOURCE_NON_CLAIM: &str =
    "TinyCC 0.9.27 source materialization proves authenticated offline archive identity and fixed-output parity only";
const TINYCC27_PATCH_BYTES: &[u8] = include_bytes!("../bootstrap/patches/tcc-0.9.27-stagex.patch");
pub(crate) const TINYCC27_PATCH_BLAKE3: &str = "b05705f548352080041ae56e2cacdf2e0898aa7dd5c857888004cf808884fc7f";
const TINYCC27_PATCH_FILE_COUNT: usize = 8;
const TINYCC27_REPORT_FORMAT: &str = "mantle-stagex-tinycc27-inventory-v1";
const TINYCC27_NON_CLAIM: &str = "this inventory binds the TinyCC 0.9.27 source patch, Mes runtime ABI refresh, compiler, and bounded smoke observations only; it does not prove later toolchain stages or provider admission";
const TINYCC27_LOGICAL_PREFIX: &str = "/stagex/tinycc-0.9.27";
const TINYCC27_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TINYCC27_RUNTIME_COMMAND_COUNT: u32 = 3;
const TINYCC27_BUILD_COMMAND_COUNT: u32 = 1;
const TINYCC27_SMOKE_COMMAND_COUNT: u32 = 3;
const TINYCC27_OUTPUT_COUNT: usize = 5;
pub(crate) const TINYCC27_FINAL_BLAKE3: &str = "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39";
pub(crate) const TINYCC27_PATCHED_SOURCE_BLAKE3: &str =
    "684632508d70bc1cfd9941cb8f3ceea609507347f677eb29f148d86000d29711";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tinycc27ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const TINYCC27_EXPECTED_OUTPUTS: [Tinycc27ExpectedOutput; TINYCC27_OUTPUT_COUNT] = [
    Tinycc27ExpectedOutput {
        artifact_id: "tinycc-0.9.27",
        digest_blake3: TINYCC27_FINAL_BLAKE3,
    },
    Tinycc27ExpectedOutput {
        artifact_id: "tinycc27-va-list",
        digest_blake3: "0374e6d6c06eb731b2189d02283736cb93a0caa66ef4022737f2617ae037c7e8",
    },
    Tinycc27ExpectedOutput {
        artifact_id: "tinycc27-libc",
        digest_blake3: "bd46a4a350e17ec4a7870be6c6dea82f7327c513ee4e2720bebb7b6e26f20d85",
    },
    Tinycc27ExpectedOutput {
        artifact_id: "tinycc27-positive-object",
        digest_blake3: "19efdabfedafc9bf94f3ddc49967773a2a5e5b5f28128c6f95ac4f36184fdb92",
    },
    Tinycc27ExpectedOutput {
        artifact_id: "tinycc27-alias",
        digest_blake3: TINYCC27_FINAL_BLAKE3,
    },
];
const TINYCC27_STDARG_HEADER: &[u8] = br#"#ifndef __MES_STDARG_H
#define __MES_STDARG_H 1
#if defined(__TINYC__) && defined(__x86_64__)
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
typedef __va_list_struct va_list[1];
void __va_start(__va_list_struct *ap, void *fp);
void *__va_arg(__va_list_struct *ap, int arg_type, int size, int align);
#define va_start(ap, last) __va_start(ap, __builtin_frame_address(0))
#define va_arg(ap, type) (*(type *)(__va_arg(ap, __builtin_va_arg_types(type), sizeof(type), __alignof__(type))))
#define va_arg8(ap, type) va_arg(ap, type)
#define va_copy(dest, src) (*(dest) = *(src))
#define va_end(ap)
#else
#include_next <stdarg.h>
#define va_arg8(ap, type) va_arg(ap, type)
#endif
#endif
"#;
const TINYCC27_VA_LIST_SOURCE: &[u8] = br#"#if defined __x86_64__
extern void *memset(void *s, int c, unsigned long n);
extern void abort(void);
enum __va_arg_type { __va_gen_reg, __va_float_reg, __va_stack };
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
void __va_start(__va_list_struct *ap, void *fp) {
    memset(ap, 0, sizeof(__va_list_struct));
    *ap = *(__va_list_struct *)((char *)fp - 16);
    ap->overflow_arg_area = (char *)fp + ap->overflow_offset;
    ap->reg_save_area = (char *)fp - 176 - 16;
}
void *__va_arg(__va_list_struct *ap, enum __va_arg_type arg_type, int size, int align) {
    size = (size + 7) & ~7;
    align = (align + 7) & ~7;
    switch (arg_type) {
    case __va_gen_reg:
        if (ap->gp_offset + size <= 48) { ap->gp_offset += size; return ap->reg_save_area + ap->gp_offset - size; }
        goto use_overflow_area;
    case __va_float_reg:
        if (ap->fp_offset < 128 + 48) { ap->fp_offset += 16; return ap->reg_save_area + ap->fp_offset - 16; }
        size = 8;
        goto use_overflow_area;
    case __va_stack:
    use_overflow_area:
        ap->overflow_arg_area += size;
        ap->overflow_arg_area = (char*)((long long)(ap->overflow_arg_area + align - 1) & -align);
        return ap->overflow_arg_area - size;
    default:
        abort();
    }
}
#endif
"#;
const TINYCC27_POSITIVE_SMOKE_SOURCE: &[u8] =
    b"int stagex_tinycc27_add(int left, int right) { return left + right; }\n";
const TINYCC27_MALFORMED_SMOKE_SOURCE: &[u8] = b"int stagex_tinycc27_broken( {\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Tinycc27SourceMaterializationReport {
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
pub(crate) struct Tinycc27InventoryRequest<'a> {
    pub source_root: &'a Path,
    pub mes_source_root: &'a Path,
    pub tinycc26_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Tinycc27Output {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Tinycc27InventoryReport {
    pub format: &'static str,
    pub source_patch_digest_blake3: String,
    pub patched_files: Vec<String>,
    pub unified_libc_source_count: u32,
    pub runtime_command_count: u32,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<Tinycc27Output>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum Tinycc27Error {
    Bundle(crate::RunError),
    InvalidAuthority(String),
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for Tinycc27Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "reading TinyCC 0.9.27 source bundle: {error}"),
            Self::InvalidAuthority(message) => write!(formatter, "invalid TinyCC 0.9.27 source authority: {message}"),
            Self::Materialization(message) => write!(formatter, "materializing TinyCC 0.9.27 source: {message}"),
            Self::Runtime(error) => write!(formatter, "building TinyCC 0.9.27: {error}"),
        }
    }
}

impl std::error::Error for Tinycc27Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for Tinycc27Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for Tinycc27Error {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::RunError> for Tinycc27Error {
    fn from(error: crate::RunError) -> Self {
        Self::Bundle(error)
    }
}

pub(crate) fn source_artifact_digest() -> (&'static str, &'static str) {
    (TINYCC27_SOURCE_ARTIFACT_ID, TINYCC27_SOURCE_CONTENT_BLAKE3)
}

pub(crate) fn materialize_authenticated_tinycc27_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    source_root: &Path,
) -> Result<Tinycc27SourceMaterializationReport, Tinycc27Error> {
    let manifest = read_source_bundle(bundle_path)?;
    validate_manifest_identity(&manifest.manifest_blake3, expected_manifest_blake3)?;
    let record = select_tinycc27_source_record(&manifest.records)?;
    fs::create_dir(source_root).map_err(|error| {
        Tinycc27Error::Materialization(format!(
            "creating create-new TinyCC 0.9.27 source root {}: {error}",
            source_root.display()
        ))
    })?;
    let output_path = source_root.join(TINYCC27_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        Tinycc27Error::Materialization(format!(
            "materializing {TINYCC27_RECORD_NAME} at {}: {error}",
            output_path.display()
        ))
    })?;
    if !output_path.is_dir() || !output_path.join("tcc.c").is_file() {
        return Err(Tinycc27Error::Materialization(format!(
            "materialized TinyCC 0.9.27 tree lacks tcc.c: {}",
            output_path.display()
        )));
    }
    assert!(output_path.is_absolute());
    assert_eq!(record.content_blake3, TINYCC27_SOURCE_CONTENT_BLAKE3);
    Ok(Tinycc27SourceMaterializationReport {
        format: TINYCC27_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: TINYCC27_SOURCE_ARTIFACT_ID,
        record_name: TINYCC27_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: TINYCC27_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_tinycc27_inventory(
    request: Tinycc27InventoryRequest<'_>,
) -> Result<Tinycc27InventoryReport, Tinycc27Error> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir).map_err(|error| {
        Tinycc27Error::Materialization(format!("creating create-new TinyCC 0.9.27 scratch: {error}"))
    })?;
    let tcc_root = request.scratch_dir.join("tinycc-0.9.27");
    let mes_root = request.scratch_dir.join("mes-0.27.1");
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &tcc_root)?;
    crate::stagex_mes_lib::copy_tree_bounded(request.mes_source_root, &mes_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&tcc_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&mes_root)?;
    let patched_files = apply_tinycc27_source_patch(&tcc_root)?;
    let source_patch_digest_blake3 = patched_tinycc27_source_digest(&tcc_root, &patched_files)?;
    if source_patch_digest_blake3 != TINYCC27_PATCHED_SOURCE_BLAKE3 {
        return Err(Tinycc27Error::Materialization(format!(
            "patched TinyCC 0.9.27 source BLAKE3 mismatch: expected {TINYCC27_PATCHED_SOURCE_BLAKE3}, observed {source_patch_digest_blake3}"
        )));
    }
    let output_root = request.scratch_dir.join("output");
    prepare_predecessor_runtime(&request, &output_root)?;
    let unified_libc_source_count = rebuild_tinycc27_runtime(&request, &mes_root, &output_root)?;
    let final_tcc = build_tinycc27(&request, &tcc_root, &output_root)?;
    run_tinycc27_smokes(&request, &output_root.join("bin/tcc"))?;
    let outputs = collect_tinycc27_outputs(&request, &final_tcc, &output_root)?;
    validate_expected_tinycc27_outputs(&outputs)?;
    let report = Tinycc27InventoryReport {
        format: TINYCC27_REPORT_FORMAT,
        source_patch_digest_blake3,
        patched_files,
        unified_libc_source_count,
        runtime_command_count: TINYCC27_RUNTIME_COMMAND_COUNT,
        build_command_count: TINYCC27_BUILD_COMMAND_COUNT,
        smoke_command_count: TINYCC27_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: TINYCC27_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tinycc27-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| Tinycc27Error::Materialization(format!("serializing TinyCC 0.9.27 report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), TINYCC27_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inventory_inputs(request: &Tinycc27InventoryRequest<'_>) -> Result<(), Tinycc27Error> {
    if request.scratch_dir.exists() {
        return Err(Tinycc27Error::Materialization(format!(
            "create-new TinyCC 0.9.27 scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("TinyCC 0.9.27 source", request.source_root),
        ("Mes source", request.mes_source_root),
        ("TinyCC 0.9.26 runtime", request.tinycc26_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(Tinycc27Error::Materialization(format!(
                "{label} root is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.tinycc26_root.join("bin/tcc"),
        crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
        "TinyCC 0.9.26 compiler",
    )?;
    if !request.source_root.join("tcc.c").is_file()
        || !request.mes_source_root.join("build-aux/configure-lib.sh").is_file()
    {
        return Err(Tinycc27Error::Materialization(
            "TinyCC 0.9.27 or Mes source root lacks a required recipe file".to_string(),
        ));
    }
    assert!(request.tinycc26_root.join("lib/mes/tcc/libtcc1.a").is_file());
    assert!(request.tinycc26_root.join("include/mes").is_dir());
    Ok(())
}

fn prepare_predecessor_runtime(
    request: &Tinycc27InventoryRequest<'_>,
    output_root: &Path,
) -> Result<(), Tinycc27Error> {
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| Tinycc27Error::Materialization(format!("creating TinyCC 0.9.27 output bin: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(
        &request.tinycc26_root.join("include/mes"),
        &output_root.join("include/mes"),
    )?;
    crate::stagex_mes_lib::copy_tree_bounded(&request.tinycc26_root.join("lib/mes"), &output_root.join("lib/mes"))?;
    assert!(output_root.join("include/mes/stdarg.h").is_file());
    assert!(output_root.join("lib/mes/tcc/libtcc1.a").is_file());
    Ok(())
}

fn rebuild_tinycc27_runtime(
    request: &Tinycc27InventoryRequest<'_>,
    mes_root: &Path,
    output_root: &Path,
) -> Result<u32, Tinycc27Error> {
    let include = output_root.join("include/mes");
    let libdir = output_root.join("lib/mes");
    write_tinycc27_stdarg_headers(mes_root, &include)?;
    patch_mes_exit_source(mes_root)?;
    let unified_source_count = crate::stagex_tinycc::create_unified_libc(mes_root)?;
    let build_root = request.scratch_dir.join("runtime-build");
    fs::create_dir_all(build_root.join("src")).map_err(|error| {
        Tinycc27Error::Materialization(format!("creating TinyCC 0.9.27 runtime source root: {error}"))
    })?;
    fs::create_dir(build_root.join("obj")).map_err(|error| {
        Tinycc27Error::Materialization(format!("creating TinyCC 0.9.27 runtime object root: {error}"))
    })?;
    crate::stagex_mes_lib::write_create_new(&build_root.join("src/va_list.c"), TINYCC27_VA_LIST_SOURCE)?;
    crate::stagex_mes_lib::copy_file_exact(&mes_root.join("unified-libc.c"), &build_root.join("src/unified-libc.c"))?;
    let compiler = request.tinycc26_root.join("bin/tcc");
    compile_tinycc27_runtime_objects(&compiler, mes_root, &build_root, request.scratch_dir)?;
    crate::stagex_mes_lib::copy_file_replace(&build_root.join("obj/va_list.o"), &libdir.join("va_list.o"))?;
    crate::stagex_mes_lib::remove_path_if_present(&libdir.join("libc.a"))?;
    let archive_args = vec![
        "-ar".to_string(),
        "cr".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC 0.9.27 libc archive")?.to_string(),
        "obj/va_list.o".to_string(),
        "obj/unified-libc.o".to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &archive_args,
        &build_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("runtime-archive.stderr.txt"),
    )?;
    validate_nonempty_file(&libdir.join("libc.a"), "TinyCC 0.9.27 Mes libc")?;
    assert!(unified_source_count > 0);
    assert!(libdir.join("va_list.o").is_file());
    Ok(unified_source_count)
}

fn write_tinycc27_stdarg_headers(mes_root: &Path, include: &Path) -> Result<(), Tinycc27Error> {
    let mes_config = mes_root.join("include/mes/config.h");
    fs::create_dir_all(mes_config.parent().expect("Mes config has a parent")).map_err(|error| {
        Tinycc27Error::Materialization(format!("creating TinyCC 0.9.27 Mes config directory: {error}"))
    })?;
    fs::write(&mes_config, b"#undef SYSTEM_LIBC\n#define MES_VERSION \"0.27.1\"\n")
        .map_err(|error| Tinycc27Error::Materialization(format!("writing TinyCC 0.9.27 Mes config: {error}")))?;
    fs::write(mes_root.join("include/stdarg.h"), TINYCC27_STDARG_HEADER)
        .map_err(|error| Tinycc27Error::Materialization(format!("writing Mes stdarg.h: {error}")))?;
    fs::write(include.join("stdarg.h"), TINYCC27_STDARG_HEADER)
        .map_err(|error| Tinycc27Error::Materialization(format!("writing output stdarg.h: {error}")))?;
    let arch_include = mes_root.join("include/arch");
    fs::create_dir(&arch_include).map_err(|error| {
        Tinycc27Error::Materialization(format!("creating Mes architecture include directory: {error}"))
    })?;
    for file in ["kernel-stat.h", "signal.h", "syscall.h"] {
        crate::stagex_mes_lib::copy_file_exact(
            &mes_root.join("include/linux/x86_64").join(file),
            &arch_include.join(file),
        )?;
    }
    assert!(mes_config.is_file());
    assert_eq!(fs::read(include.join("stdarg.h")).ok().as_deref(), Some(TINYCC27_STDARG_HEADER));
    assert!(arch_include.join("syscall.h").is_file());
    Ok(())
}

fn patch_mes_exit_source(mes_root: &Path) -> Result<(), Tinycc27Error> {
    let exit_source = mes_root.join("lib/linux/x86_64-mes-gcc/_exit.c");
    crate::stagex_tinycc::replace_required_text(&exit_source, ": // no outputs \"=\" (r)", ": // no outputs")?;
    crate::stagex_tinycc::replace_required_text(
        &exit_source,
        ": \"rm\" (code)",
        ": \"r\" (code)\n       : \"rax\", \"rdi\"",
    )?;
    let text = fs::read_to_string(&exit_source)
        .map_err(|error| Tinycc27Error::Materialization(format!("reading patched Mes _exit.c: {error}")))?;
    assert!(text.contains(": \"rax\", \"rdi\""));
    assert!(!text.contains("=\" (r)"));
    Ok(())
}

fn compile_tinycc27_runtime_objects(
    compiler: &Path,
    mes_root: &Path,
    build_root: &Path,
    observation_root: &Path,
) -> Result<(), Tinycc27Error> {
    let include_path = mes_root.join("include");
    let linux_include_path = mes_root.join("include/linux/x86_64");
    let include = crate::stagex_mes_lib::utf8_absolute(&include_path, "Mes include")?;
    let linux_include = crate::stagex_mes_lib::utf8_absolute(&linux_include_path, "Mes Linux include")?;
    let common = [
        "-D".to_string(),
        "HAVE_CONFIG_H=1".to_string(),
        "-I".to_string(),
        include.to_string(),
        "-I".to_string(),
        linux_include.to_string(),
    ];
    let mut va_args = vec!["-c".to_string()];
    va_args.extend(common.iter().cloned());
    va_args.extend([
        "-o".to_string(),
        "obj/va_list.o".to_string(),
        "src/va_list.c".to_string(),
    ]);
    let mut libc_args = vec!["-w".to_string(), "-c".to_string()];
    libc_args.extend(common);
    libc_args.extend([
        "-o".to_string(),
        "obj/unified-libc.o".to_string(),
        "src/unified-libc.c".to_string(),
    ]);
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &va_args,
        build_root,
        &empty_env,
        &observation_root.join("runtime-va-list.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &libc_args,
        build_root,
        &empty_env,
        &observation_root.join("runtime-libc.stderr.txt"),
    )?;
    assert!(build_root.join("obj/va_list.o").is_file());
    assert!(build_root.join("obj/unified-libc.o").is_file());
    Ok(())
}

fn build_tinycc27(
    request: &Tinycc27InventoryRequest<'_>,
    tcc_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, Tinycc27Error> {
    fs::write(tcc_root.join("config.h"), [])
        .map_err(|error| Tinycc27Error::Materialization(format!("writing TinyCC 0.9.27 config.h: {error}")))?;
    let compiler = request.tinycc26_root.join("bin/tcc");
    let output = output_root.join("bin/tcc-0.9.27");
    let args = tinycc27_compile_args(tcc_root, output_root, &output)?;
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        tcc_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tinycc27-build.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&output)?;
    crate::stagex_mes_lib::copy_file_exact(&output, &output_root.join("bin/tcc"))?;
    crate::stagex_tinycc::set_owner_executable(&output_root.join("bin/tcc"))?;
    validate_nonempty_file(&output, "TinyCC 0.9.27 compiler")?;
    assert!(output_root.join("bin/tcc").is_file());
    Ok(output)
}

fn tinycc27_compile_args(tcc_root: &Path, output_root: &Path, output: &Path) -> Result<Vec<String>, Tinycc27Error> {
    let libdir = output_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let include = output_root.join("include/mes");
    let args = vec![
        "-v".to_string(),
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime, "TinyCC 0.9.27 runtime")?.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "TinyCC 0.9.27 output")?.to_string(),
        "-I".to_string(),
        ".".to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&include, "TinyCC 0.9.27 include")?.to_string(),
        "-D".to_string(),
        "BOOTSTRAP=1".to_string(),
        "-D".to_string(),
        "HAVE_BITFIELD=1".to_string(),
        "-D".to_string(),
        "HAVE_FLOAT=1".to_string(),
        "-D".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "-D".to_string(),
        "HAVE_SETJMP=1".to_string(),
        "-D".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCCDIR=\"{TINYCC27_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        format!("CONFIG_TCC_CRTPREFIX=\"{TINYCC27_LOGICAL_PREFIX}/lib/mes\""),
        "-D".to_string(),
        "CONFIG_TCC_ELFINTERP=\"/mes/loader\"".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCC_LIBPATHS=\"{TINYCC27_LOGICAL_PREFIX}/lib/mes:{TINYCC27_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{TINYCC27_LOGICAL_PREFIX}/include/mes\""),
        "-D".to_string(),
        "CONFIG_SYSROOT=\"/\"".to_string(),
        "-D".to_string(),
        format!("TCC_LIBGCC=\"{TINYCC27_LOGICAL_PREFIX}/lib/mes/libc.a\""),
        "-D".to_string(),
        "TCC_LIBTCC1=\"libtcc1.a\"".to_string(),
        "-D".to_string(),
        "CONFIG_TCCBOOT=1".to_string(),
        "-D".to_string(),
        "CONFIG_TCC_STATIC=1".to_string(),
        "-D".to_string(),
        "CONFIG_USE_LIBGCC=1".to_string(),
        "-D".to_string(),
        "TCC_VERSION=\"0.9.27\"".to_string(),
        "-D".to_string(),
        "ONE_SOURCE=1".to_string(),
        "tcc.c".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("va_list.o"), "TinyCC va_list object")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1 object")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc archive")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&runtime.join("libtcc1.a"), "TinyCC libtcc1 archive")?.to_string(),
    ];
    assert!(tcc_root.join("tcc.c").is_file());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn run_tinycc27_smokes(request: &Tinycc27InventoryRequest<'_>, compiler: &Path) -> Result<(), Tinycc27Error> {
    let smoke_root = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| Tinycc27Error::Materialization(format!("creating TinyCC 0.9.27 smoke root: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("positive.c"), TINYCC27_POSITIVE_SMOKE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.c"), TINYCC27_MALFORMED_SMOKE_SOURCE)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-version"],
        &smoke_root,
        &empty_env,
        &smoke_root.join("version.stderr.txt"),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-c", "-o", "positive.o", "positive.c"],
        &smoke_root,
        &empty_env,
        &smoke_root.join("positive.stderr.txt"),
    )?;
    require_expected_compile_failure(
        compiler,
        &["-c", "-o", "malformed.o", "malformed.c"],
        &smoke_root,
        &smoke_root.join("malformed.stderr.txt"),
    )?;
    validate_nonempty_file(&smoke_root.join("positive.o"), "TinyCC 0.9.27 positive smoke object")?;
    if smoke_root.join("malformed.o").exists() {
        return Err(Tinycc27Error::Materialization(
            "TinyCC 0.9.27 malformed smoke unexpectedly produced an object".to_string(),
        ));
    }
    assert!(smoke_root.join("positive.o").is_file());
    assert!(!smoke_root.join("malformed.o").exists());
    Ok(())
}

fn require_expected_compile_failure(
    compiler: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), Tinycc27Error> {
    match crate::stagex_mes_lib::run_bounded_process(
        compiler,
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
        Ok(()) => Err(Tinycc27Error::Materialization("TinyCC 0.9.27 accepted malformed smoke source".to_string())),
        Err(error) => Err(Tinycc27Error::Runtime(error)),
    }
}

fn collect_tinycc27_outputs(
    request: &Tinycc27InventoryRequest<'_>,
    compiler: &Path,
    output_root: &Path,
) -> Result<Vec<Tinycc27Output>, Tinycc27Error> {
    let specs = [
        ("tinycc-0.9.27", compiler.to_path_buf()),
        ("tinycc27-va-list", output_root.join("lib/mes/va_list.o")),
        ("tinycc27-libc", output_root.join("lib/mes/libc.a")),
        ("tinycc27-positive-object", request.scratch_dir.join("smoke/positive.o")),
        ("tinycc27-alias", output_root.join("bin/tcc")),
    ];
    let mut outputs = Vec::with_capacity(specs.len());
    for (artifact_id, path) in specs {
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, TINYCC27_FILE_BYTES_MAX, artifact_id)?;
        outputs.push(Tinycc27Output {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len()).map_err(|_| {
                Tinycc27Error::Materialization("TinyCC 0.9.27 output size does not fit u64".to_string())
            })?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), TINYCC27_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_tinycc27_outputs(outputs: &[Tinycc27Output]) -> Result<(), Tinycc27Error> {
    if outputs.len() != TINYCC27_OUTPUT_COUNT {
        return Err(Tinycc27Error::Materialization(format!(
            "TinyCC 0.9.27 report has {} outputs; expected {TINYCC27_OUTPUT_COUNT}",
            outputs.len()
        )));
    }
    for expected in TINYCC27_EXPECTED_OUTPUTS {
        let output = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            Tinycc27Error::Materialization(format!("TinyCC 0.9.27 report lacks {}", expected.artifact_id))
        })?;
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(Tinycc27Error::Materialization(format!(
                "TinyCC 0.9.27 output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), TINYCC27_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| !output.digest_blake3.is_empty()));
    Ok(())
}

pub(crate) fn patched_tinycc27_source_digest(root: &Path, files: &[String]) -> Result<String, Tinycc27Error> {
    let mut sorted = files.to_vec();
    sorted.sort();
    let mut hasher = blake3::Hasher::new();
    for relative in &sorted {
        let bytes = crate::stagex_mes_lib::read_bounded_file(
            &root.join(relative),
            TINYCC27_FILE_BYTES_MAX,
            "patched TinyCC 0.9.27 source",
        )?;
        let path_len = u64::try_from(relative.len())
            .map_err(|_| Tinycc27Error::Materialization("TinyCC 0.9.27 patch path length overflow".to_string()))?;
        let byte_len = u64::try_from(bytes.len())
            .map_err(|_| Tinycc27Error::Materialization("TinyCC 0.9.27 patch byte length overflow".to_string()))?;
        hasher.update(&path_len.to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&byte_len.to_le_bytes());
        hasher.update(&bytes);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert_eq!(sorted.len(), TINYCC27_PATCH_FILE_COUNT);
    Ok(digest)
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), Tinycc27Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TINYCC27_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(Tinycc27Error::InvalidAuthority(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed, expected);
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), Tinycc27Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TINYCC27_FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(Tinycc27Error::Materialization(format!("{label} is empty: {}", path.display())));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

pub(crate) fn apply_tinycc27_source_patch(root: &Path) -> Result<Vec<String>, Tinycc27Error> {
    validate_bound_bytes(TINYCC27_PATCH_BYTES, TINYCC27_PATCH_BLAKE3, "TinyCC 0.9.27 source patch")?;
    let patched = crate::stagex_patch::apply_unified_patch_tree(root, TINYCC27_PATCH_BYTES)
        .map_err(|error| Tinycc27Error::Materialization(error.to_string()))?;
    if patched.len() != TINYCC27_PATCH_FILE_COUNT {
        return Err(Tinycc27Error::Materialization(format!(
            "TinyCC 0.9.27 patch changed {} files; expected {TINYCC27_PATCH_FILE_COUNT}",
            patched.len()
        )));
    }
    assert_eq!(patched.len(), TINYCC27_PATCH_FILE_COUNT);
    assert!(patched.iter().all(|path| root.join(path).is_file()));
    Ok(patched)
}

fn validate_bound_bytes(bytes: &[u8], expected_blake3: &str, label: &str) -> Result<(), Tinycc27Error> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if expected_blake3.len() != blake3::OUT_LEN * 2 || observed != expected_blake3 {
        return Err(Tinycc27Error::InvalidAuthority(format!(
            "{label} BLAKE3 mismatch: expected {expected_blake3}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed, expected_blake3);
    Ok(())
}

fn validate_manifest_identity(observed: &str, expected: &str) -> Result<(), Tinycc27Error> {
    if expected.len() != blake3::OUT_LEN * 2 || observed != expected {
        return Err(Tinycc27Error::InvalidAuthority(format!(
            "source bundle BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert_eq!(observed, expected);
    Ok(())
}

fn select_tinycc27_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, Tinycc27Error> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(TINYCC27_RECORD_NAME))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(Tinycc27Error::InvalidAuthority(format!(
            "source bundle has {} records named {TINYCC27_RECORD_NAME}; expected exactly one",
            matches.len()
        )));
    }
    let record = matches[0];
    if record.kind != SourceRecordKind::FixedUrl
        || record.content_blake3 != TINYCC27_SOURCE_CONTENT_BLAKE3
        || record.files.is_empty()
    {
        return Err(Tinycc27Error::InvalidAuthority(format!(
            "source record {TINYCC27_RECORD_NAME} has substituted kind, digest, or empty payload: {}",
            record.identity
        )));
    }
    assert!(record.payload_bytes > 0);
    assert!(!record.identity.is_empty());
    Ok(record)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::source_bundle::SourceFileEntry;
    use crate::source_bundle::SourceFileType;

    const TEST_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn record() -> SourceRecord {
        SourceRecord {
            kind: SourceRecordKind::FixedUrl,
            identity: "fixed-url-tinycc27".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), TINYCC27_RECORD_NAME.to_string())]),
            payload_bytes: 1,
            content_blake3: TINYCC27_SOURCE_CONTENT_BLAKE3.to_string(),
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
    fn selects_exact_tinycc27_source_and_rejects_substitution() {
        let valid = record();
        let selected = select_tinycc27_source_record(std::slice::from_ref(&valid)).unwrap();
        assert_eq!(selected.content_blake3, TINYCC27_SOURCE_CONTENT_BLAKE3);
        assert_eq!(selected.kind, SourceRecordKind::FixedUrl);

        let mut substituted = record();
        substituted.content_blake3 = TEST_BLAKE3.to_string();
        let error = select_tinycc27_source_record(&[substituted]).unwrap_err();
        assert!(error.to_string().contains("substituted kind, digest, or empty payload"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn rejects_substituted_manifest_identity() {
        let error = validate_manifest_identity(TEST_BLAKE3, TINYCC27_SOURCE_CONTENT_BLAKE3).unwrap_err();
        assert!(error.to_string().contains("source bundle BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());

        let patch_error = validate_bound_bytes(b"substituted", TINYCC27_PATCH_BLAKE3, "test patch").unwrap_err();
        assert!(patch_error.to_string().contains("test patch BLAKE3 mismatch"));
        assert!(!patch_error.to_string().is_empty());
    }

    #[test]
    fn validates_exact_tinycc27_outputs_and_rejects_substitution() {
        let mut outputs = TINYCC27_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| Tinycc27Output {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_expected_tinycc27_outputs(&outputs).unwrap();
        assert_eq!(outputs.len(), TINYCC27_OUTPUT_COUNT);

        outputs[0].digest_blake3 = TEST_BLAKE3.to_string();
        let error = validate_expected_tinycc27_outputs(&outputs).unwrap_err();
        assert!(error.to_string().contains("output tinycc-0.9.27 BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires explicit materialized TinyCC 0.9.27 source and create-new scratch"]
    fn applies_retained_tinycc27_source_patch() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_SOURCE_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_PATCH_SCRATCH").unwrap());
        crate::stagex_mes_lib::copy_tree_bounded(&source, &scratch).unwrap();
        crate::stagex_tinycc::make_tree_owner_writable(&scratch).unwrap();

        let patched = apply_tinycc27_source_patch(&scratch).unwrap();

        assert_eq!(patched.len(), TINYCC27_PATCH_FILE_COUNT);
        assert!(scratch.join("tcc.c").is_file());
        assert!(scratch.join("x86_64-gen.c").is_file());
    }

    #[test]
    #[ignore = "requires explicit TinyCC 0.9.27 source, Mes source, TinyCC 0.9.26 runtime, and create-new scratch"]
    fn derives_retained_tinycc27_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_SOURCE_ROOT").unwrap());
        let mes_source = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_MES_SOURCE_ROOT").unwrap());
        let tinycc26 = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC26_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_BUILD_SCRATCH").unwrap());

        let report = derive_tinycc27_inventory(Tinycc27InventoryRequest {
            source_root: &source,
            mes_source_root: &mes_source,
            tinycc26_root: &tinycc26,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();

        assert_eq!(report.outputs.len(), TINYCC27_OUTPUT_COUNT);
        assert_eq!(report.runtime_command_count, TINYCC27_RUNTIME_COMMAND_COUNT);
        assert!(scratch.join("tinycc27-inventory.json").is_file());
        assert!(report.fallback_events.is_empty());
    }

    #[test]
    #[ignore = "requires explicit authenticated source bundle and create-new scratch"]
    fn materializes_retained_tinycc27_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC27_SOURCE_SCRATCH").unwrap());

        let report = materialize_authenticated_tinycc27_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        fs::write(scratch.join("tinycc27-source-materialization.json"), serde_json::to_vec_pretty(&report).unwrap())
            .unwrap();

        assert!(report.output_path.is_dir());
        assert_eq!(report.record_content_blake3, TINYCC27_SOURCE_CONTENT_BLAKE3);
        assert!(scratch.join("tinycc27-source-materialization.json").is_file());
    }
}
