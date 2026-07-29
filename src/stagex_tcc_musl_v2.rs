use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

const RECIPE_ARTIFACT_ID: &str = "tcc-musl-v2-recipe-source";
const RECIPE_BLAKE3: &str = "c82e4b5c375d3f9ba767c916ad4dba18e995d239755ea492ae94a007d2f49190";
const RECIPE: &[u8] = include_bytes!("../bootstrap/tcc-musl-v2.ncl");
const STDARG_RUNTIME_ARTIFACT_ID: &str = "tcc-musl-v2-stdarg-runtime-source";
const STDARG_RUNTIME_TARGET: &str = "/tmp/tcc-stdarg-runtime.c";
const STDARG_RUNTIME_BLAKE3: &str = "39655914dd22719e8f64182e281622ccaa4104ab7d901f728db84338659cc942";
const REPORT_FORMAT: &str = "mantle-stagex-tcc-musl-v2-inventory-v1";
const NON_CLAIM: &str = "this inventory binds TinyCC 0.9.27 rebuilt by the protected TinyCC 0.9.26 host, its extended variadic runtime, and bounded compile/link/execute observations only; it does not prove the later self-host, native runtime, or provider admission";
const TCC_LOGICAL_PREFIX: &str = "runtime/tcc-musl-v2";
const MUSL_LOGICAL_PREFIX: &str = "runtime/musl-1.1.24-pass2";
const FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const BUILD_COMMAND_COUNT: u32 = 4;
const SMOKE_COMMAND_COUNT: u32 = 5;
const OUTPUT_COUNT: usize = 8;
pub(crate) const CONFIGURED_SOURCE_BLAKE3: &str = "b68ec9f7c46ca1cbec5508a98476f771e4bcc4d90b4e088a237065aac1c6061b";
pub(crate) const COMPILER_BLAKE3: &str = "a17643b015db05bd452918841ba7811b8e063edfe29328672388f6b693a1b8ec";
pub(crate) const RUNTIME_BLAKE3: &str = "c201ffd35a5466b4171345fc924b64a75c650fabbe7d0c20d365cfe08faf0860";
const SOURCE_TREE_BLAKE3: &str = "5461df0d2a3503f2a7d124859d2ddedab87694f16bd81b644b9cdb8fffdfa367";
const HEADER_TREE_BLAKE3: &str = "2fa068749df350a62721e60696d625b6d5b0e3061019929a4c67970155833424";
const POSITIVE_OBJECT_BLAKE3: &str = "a507376f7e9af418c78cc9f9121e7b182c8b1ef3b741de98741d6320f6f301b2";
pub(crate) const POSITIVE_BINARY_BLAKE3: &str = "2a6733b887417ec6f848ece381d125e10d5d5544f0839dcfa896eacc88529fa6";
const POSITIVE_SOURCE: &[u8] = b"#include \"tcc-stdarg-prefix.h\"\nenum { EXPECTED_VARIADIC_VALUE = 7 };\nstatic int first_variadic(int ignored, ...) { va_list arguments; int value; va_start(arguments, ignored); value = va_arg(arguments, int); va_end(arguments); return value; }\nint main(void) { return first_variadic(0, EXPECTED_VARIADIC_VALUE) == EXPECTED_VARIADIC_VALUE ? 0 : 1; }\n";
const MALFORMED_SOURCE: &[u8] = b"#include \"tcc-stdarg-prefix.h\"\nstatic int malformed_variadic(int ignored, ...) { va_list arguments; va_start(arguments, ignored); return va_arg(arguments, ); }\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const EXPECTED_OUTPUTS: [ExpectedOutput; OUTPUT_COUNT] = [
    ExpectedOutput {
        artifact_id: "tcc-musl-v2",
        digest_blake3: COMPILER_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-alias",
        digest_blake3: COMPILER_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-libtcc1",
        digest_blake3: RUNTIME_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-source-tree",
        digest_blake3: SOURCE_TREE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-header-tree",
        digest_blake3: HEADER_TREE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-positive-object",
        digest_blake3: POSITIVE_OBJECT_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-positive-binary",
        digest_blake3: POSITIVE_BINARY_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-v2-positive-execution",
        digest_blake3: "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct InventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<OutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InventoryRequest<'a> {
    pub tinycc27_source_root: &'a Path,
    pub tinycc26_root: &'a Path,
    pub tcc_musl_prep_root: &'a Path,
    pub musl_pass2_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum Error {
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Materialization(message) => write!(formatter, "TinyCC musl-v2 materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "TinyCC musl-v2 runtime failed: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for Error {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tinycc27::Tinycc27Error> for Error {
    fn from(error: crate::stagex_tinycc27::Tinycc27Error) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tcc_musl_prep::TccMuslPrepError> for Error {
    fn from(error: crate::stagex_tcc_musl_prep::TccMuslPrepError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); 2] {
    [
        (RECIPE_ARTIFACT_ID, RECIPE_BLAKE3),
        (STDARG_RUNTIME_ARTIFACT_ID, STDARG_RUNTIME_BLAKE3),
    ]
}

pub(crate) fn derive_inventory(request: InventoryRequest<'_>) -> Result<InventoryReport, Error> {
    validate_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| Error::Materialization(format!("creating TinyCC musl-v2 scratch: {error}")))?;
    let source_root = request.scratch_dir.join("tcc-0.9.27");
    crate::stagex_mes_lib::copy_tree_bounded(request.tinycc27_source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    let patched = crate::stagex_tinycc27::apply_tinycc27_source_patch(&source_root)?;
    let patch_digest = crate::stagex_tinycc27::patched_tinycc27_source_digest(&source_root, &patched)?;
    if patch_digest != crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3 {
        return Err(Error::Materialization(format!("TinyCC musl-v2 patch mismatch: {patch_digest}")));
    }
    crate::stagex_tcc_musl_prep::apply_musl_prep_adjustments(&source_root)?;
    apply_v2_codegen_adjustments(&source_root)?;
    fs::write(source_root.join("config.h"), [])
        .map_err(|error| Error::Materialization(format!("writing TinyCC musl-v2 config.h: {error}")))?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output");
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| Error::Materialization(format!("creating TinyCC musl-v2 bin: {error}")))?;
    fs::create_dir_all(output_root.join("lib/tcc"))
        .map_err(|error| Error::Materialization(format!("creating TinyCC musl-v2 lib: {error}")))?;
    let compiler = build_compiler(&request, &source_root, &output_root)?;
    export_source_contract(&source_root, &output_root)?;
    build_runtime(&request, &source_root, &output_root)?;
    let (positive_object, positive_binary, execution) = run_smokes(&request, &output_root, &compiler)?;
    let outputs = collect_outputs(&output_root, &compiler, &positive_object, &positive_binary, &execution)?;
    validate_expected_outputs(&outputs)?;
    let report = InventoryReport {
        format: REPORT_FORMAT,
        configured_source_digest_blake3,
        build_command_count: BUILD_COMMAND_COUNT,
        smoke_command_count: SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tcc-musl-v2-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| Error::Materialization(format!("serializing TinyCC musl-v2 report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inputs(request: &InventoryRequest<'_>) -> Result<(), Error> {
    if request.scratch_dir.exists() {
        return Err(Error::Materialization(format!(
            "create-new TinyCC musl-v2 scratch exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("TinyCC 0.9.27 source", request.tinycc27_source_root),
        ("TinyCC 0.9.26 host", request.tinycc26_root),
        ("TinyCC musl-prep", request.tcc_musl_prep_root),
        ("second-musl", request.musl_pass2_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(Error::Materialization(format!("{label} is not an absolute directory: {}", root.display())));
        }
    }
    validate_file_digest(
        &request.tinycc26_root.join("bin/tcc-0.9.26"),
        crate::stagex_tinycc::TINYCC_FINAL_BLAKE3,
        "TinyCC 0.9.26 host",
    )?;
    validate_file_digest(
        &request.musl_pass2_root.join("lib/libc.a"),
        crate::stagex_musl::MUSL_PASS2_LIBC_BLAKE3,
        "second-musl libc",
    )?;
    validate_bound_sources()?;
    assert!(request.tcc_musl_prep_root.join("lib/mes/tcc/libtcc1.a").is_file());
    assert!(request.tinycc27_source_root.join("tcc.c").is_file());
    Ok(())
}

fn apply_v2_codegen_adjustments(source_root: &Path) -> Result<(), Error> {
    let codegen = source_root.join("x86_64-gen.c");
    crate::stagex_tinycc::replace_required_text(&codegen, "R_X86_64_PLT32", "R_X86_64_PC32")?;
    crate::stagex_tinycc::replace_required_text(
        &codegen,
        "    greloca(cur_text_section, sym, ind, R_X86_64_GOTPCREL, -4);",
        "    gen_addrpc32(r, sym, c); return;",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &codegen,
        "    is_got = (op_reg & TREG_MEM) && !(sym->type.t & VT_STATIC);",
        "    is_got = 0;",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &codegen,
        "    if ((fr & VT_VALMASK) == VT_CONST && (fr & VT_SYM) &&\n        (fr & VT_LVAL) && !(sv->sym->type.t & VT_STATIC)) {",
        "    if (0) {",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &codegen,
        "                if (sv->sym->type.t & VT_STATIC) {",
        "                if (1 || (sv->sym->type.t & VT_STATIC)) {",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &codegen,
        "    if (fr == VT_CONST && (v->r & VT_SYM)) {",
        "    if (0 && fr == VT_CONST && (v->r & VT_SYM)) {",
    )?;
    let library = source_root.join("libtcc.c");
    crate::stagex_tinycc::replace_required_text(
        &library,
        "    int pos = strlen(buf);\n    while (*fmt) {",
        "    int pos = strlen(buf);\n    (void)va_arg(ap, char *);\n    while (*fmt) {",
    )?;
    let tools = source_root.join("tcctools.c");
    crate::stagex_tinycc::replace_required_text(
        &tools,
        "static int ar_usage(int ret) {",
        "static void crunch_ar_size(char dst[10], unsigned long v)\n{\n    char tmp[20]; int i = 0, j = 0;\n    do { tmp[i++] = 48 + (v % 10); v /= 10; } while (v && i < 20);\n    while (i && j < 10) dst[j++] = tmp[--i];\n    while (j < 10) dst[j++] = 32;\n}\n\nstatic int ar_usage(int ret) {",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &tools,
        "        sprintf(stmp, \"%-10d\", fsize);\n        memcpy(&arhdro.ar_size, stmp, 10);",
        "        crunch_ar_size(arhdro.ar_size, (unsigned long)fsize);",
    )?;
    crate::stagex_tinycc::replace_required_text(
        &tools,
        "    sprintf(stmp, \"%-10d\", (int)(strpos + (funccnt+1) * sizeof(int)));\n    memcpy(&arhdr.ar_size, stmp, 10);",
        "    crunch_ar_size(arhdr.ar_size, (unsigned long)(strpos + (funccnt+1) * sizeof(int)));",
    )?;
    let generator = source_root.join("tccgen.c");
    let generator_text = fs::read_to_string(&generator)
        .map_err(|error| Error::Materialization(format!("reading TinyCC generator: {error}")))?;
    if !generator_text.contains("if (0 && !is_compatible_types(&sym->type, type))") {
        return Err(Error::Materialization(
            "TinyCC generator lacks the bound redefinition compatibility guard".to_string(),
        ));
    }
    assert!(codegen.is_file());
    assert!(library.is_file());
    assert!(tools.is_file());
    assert!(generator.is_file());
    Ok(())
}

fn build_compiler(request: &InventoryRequest<'_>, source_root: &Path, output_root: &Path) -> Result<PathBuf, Error> {
    let host = request.tinycc26_root.join("bin/tcc-0.9.26");
    let object = source_root.join("tcc-musl-v2.o");
    crate::stagex_mes_lib::run_bounded_process(
        &host,
        &compile_args(request, &object)?,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-v2-compile.stderr.txt"),
    )?;
    validate_nonempty_file(&object, "TinyCC musl-v2 object")?;
    let compiler = output_root.join("bin/tcc-0.9.27-musl-v2");
    crate::stagex_mes_lib::run_bounded_process(
        &host,
        &link_args(request, &object, &compiler)?,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-v2-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&compiler)?;
    crate::stagex_mes_lib::copy_file_exact(&compiler, &output_root.join("bin/tcc"))?;
    crate::stagex_tinycc::set_owner_executable(&output_root.join("bin/tcc"))?;
    assert!(object.is_file());
    assert!(compiler.is_file());
    Ok(compiler)
}

fn compile_args(request: &InventoryRequest<'_>, object: &Path) -> Result<Vec<String>, Error> {
    let values = [
        "BOOTSTRAP=1".to_string(),
        "HAVE_BITFIELD=1".to_string(),
        "HAVE_FLOAT=1".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "HAVE_SETJMP=1".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        format!("CONFIG_TCCDIR=\"{TCC_LOGICAL_PREFIX}/lib/tcc\""),
        format!("CONFIG_TCC_CRTPREFIX=\"{MUSL_LOGICAL_PREFIX}/lib\""),
        "CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\"".to_string(),
        format!("CONFIG_TCC_LIBPATHS=\"{MUSL_LOGICAL_PREFIX}/lib:{TCC_LOGICAL_PREFIX}/lib/tcc\""),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{MUSL_LOGICAL_PREFIX}/include\""),
        "CONFIG_SYSROOT=\"/\"".to_string(),
        format!("TCC_LIBGCC=\"{MUSL_LOGICAL_PREFIX}/lib/libc.a\""),
        "TCC_LIBTCC1=\"libtcc1.a\"".to_string(),
        "CONFIG_TCCBOOT=1".to_string(),
        "CONFIG_TCC_STATIC=1".to_string(),
        "CONFIG_USE_LIBGCC=1".to_string(),
        "TCC_VERSION=\"0.9.27\"".to_string(),
        "ONE_SOURCE=1".to_string(),
    ];
    let mut args = vec![
        "-v".to_string(),
        "-c".to_string(),
        "-o".to_string(),
        absolute_utf8(object, "TinyCC musl-v2 object")?,
        "-I".to_string(),
        ".".to_string(),
        "-I".to_string(),
        absolute_utf8(&request.tinycc26_root.join("include/mes"), "TinyCC 0.9.26 include")?,
    ];
    for value in values {
        args.push("-D".to_string());
        args.push(value);
    }
    args.push("tcc.c".to_string());
    assert!(args.len() > 1);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn link_args(request: &InventoryRequest<'_>, object: &Path, output: &Path) -> Result<Vec<String>, Error> {
    let libdir = request.tinycc26_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let args = vec![
        "-v".to_string(),
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-B".to_string(),
        absolute_utf8(&runtime, "TinyCC 0.9.26 runtime")?,
        "-o".to_string(),
        absolute_utf8(output, "TinyCC musl-v2 output")?,
        absolute_utf8(&libdir.join("crt1.o"), "TinyCC musl-v2 crt1")?,
        absolute_utf8(&libdir.join("crti.o"), "TinyCC musl-v2 crti")?,
        absolute_utf8(object, "TinyCC musl-v2 object")?,
        absolute_utf8(&libdir.join("libc.a"), "TinyCC musl-v2 Mes libc")?,
        absolute_utf8(&runtime.join("libtcc1.a"), "TinyCC musl-v2 Mes libtcc1")?,
        absolute_utf8(&libdir.join("crtn.o"), "TinyCC musl-v2 crtn")?,
    ];
    assert!(args.len() > 1);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn export_source_contract(source_root: &Path, output_root: &Path) -> Result<(), Error> {
    let target = output_root.join("share/tcc-source");
    fs::create_dir_all(&target)
        .map_err(|error| Error::Materialization(format!("creating TinyCC musl-v2 source export: {error}")))?;
    let mut count = 0u32;
    for entry in fs::read_dir(source_root)
        .map_err(|error| Error::Materialization(format!("reading TinyCC musl-v2 source root: {error}")))?
    {
        let path = entry
            .map_err(|error| Error::Materialization(format!("reading TinyCC musl-v2 source entry: {error}")))?
            .path();
        if !path.is_file() {
            continue;
        }
        if !matches!(path.extension().and_then(|value| value.to_str()), Some("c" | "h" | "def")) {
            continue;
        }
        let name = path
            .file_name()
            .ok_or_else(|| Error::Materialization("TinyCC source file lacks name".to_string()))?;
        crate::stagex_mes_lib::copy_file_exact(&path, &target.join(name))?;
        count = count
            .checked_add(1)
            .ok_or_else(|| Error::Materialization("TinyCC source count overflow".to_string()))?;
    }
    crate::stagex_mes_lib::copy_tree_bounded(&source_root.join("include"), &target.join("include"))?;
    crate::stagex_mes_lib::copy_tree_bounded(&source_root.join("include"), &output_root.join("lib/tcc/include"))?;
    if count == 0 || count > 128 {
        return Err(Error::Materialization(format!("TinyCC source export count outside bounds: {count}")));
    }
    assert!(target.join("tcc.c").is_file());
    assert!(target.join("include/stdarg.h").is_file());
    Ok(())
}

fn build_runtime(request: &InventoryRequest<'_>, source_root: &Path, output_root: &Path) -> Result<(), Error> {
    let source = source_root.join("tcc-stdarg-runtime.c");
    crate::stagex_mes_lib::write_create_new(&source, &extract_helper(RECIPE, STDARG_RUNTIME_TARGET)?)?;
    let object = source_root.join("tcc-stdarg-runtime.o");
    let host = request.tinycc26_root.join("bin/tcc-0.9.26");
    crate::stagex_mes_lib::run_bounded_process(
        &host,
        &[
            "-c",
            "-o",
            &absolute_utf8(&object, "TinyCC stdarg runtime object")?,
            "tcc-stdarg-runtime.c",
        ],
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-stdarg-runtime.stderr.txt"),
    )?;
    let archive = output_root.join("lib/tcc/libtcc1.a");
    crate::stagex_mes_lib::copy_file_exact(&request.tcc_musl_prep_root.join("lib/mes/tcc/libtcc1.a"), &archive)?;
    crate::stagex_mes_lib::run_bounded_process(
        &host,
        &[
            "-ar",
            "rcs",
            &absolute_utf8(&archive, "TinyCC musl-v2 runtime archive")?,
            &absolute_utf8(&object, "TinyCC stdarg runtime object")?,
        ],
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-stdarg-archive.stderr.txt"),
    )?;
    validate_nonempty_file(&archive, "TinyCC musl-v2 runtime archive")?;
    assert!(object.is_file());
    assert!(archive.is_file());
    Ok(())
}

fn run_smokes(
    request: &InventoryRequest<'_>,
    output_root: &Path,
    compiler: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), Error> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| Error::Materialization(format!("creating TinyCC musl-v2 smoke: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("variadic-positive.c"), POSITIVE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("variadic-negative.c"), MALFORMED_SOURCE)?;
    crate::stagex_mes_lib::copy_file_exact(
        &output_root.join("lib/tcc/include/stdarg.h"),
        &smoke.join("tcc-stdarg-prefix.h"),
    )?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-version"],
        &smoke,
        &empty_env,
        &smoke.join("version.stderr.txt"),
    )?;
    let object = smoke.join("variadic-positive.o");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &[
            "-c",
            "-o",
            &absolute_utf8(&object, "TinyCC musl-v2 positive object")?,
            "variadic-positive.c",
        ],
        &smoke,
        &empty_env,
        &smoke.join("positive-compile.stderr.txt"),
    )?;
    set_owner_read_write(&object)?;
    let staged_musl = smoke.join(MUSL_LOGICAL_PREFIX);
    crate::stagex_mes_lib::copy_tree_bounded(request.musl_pass2_root, &staged_musl)?;
    let staged_tcc = smoke.join(TCC_LOGICAL_PREFIX);
    fs::create_dir_all(&staged_tcc)
        .map_err(|error| Error::Materialization(format!("creating staged TinyCC runtime: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(&output_root.join("lib"), &staged_tcc.join("lib"))?;
    let binary = smoke.join("variadic-positive");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-static", "-o", "variadic-positive", "variadic-positive.c"],
        &smoke,
        &empty_env,
        &smoke.join("positive-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&binary)?;
    let execution = smoke.join("positive-execution.stdout.txt");
    crate::stagex_mes_lib::run_bounded_process(&binary, &[] as &[&str], &smoke, &empty_env, &execution)?;
    require_expected_failure(
        compiler,
        &["-c", "-o", "variadic-negative.o", "variadic-negative.c"],
        &smoke,
        &smoke.join("negative.stderr.txt"),
    )?;
    validate_nonempty_file(&object, "TinyCC musl-v2 positive object")?;
    validate_nonempty_file(&binary, "TinyCC musl-v2 positive binary")?;
    if smoke.join("variadic-negative.o").exists() {
        return Err(Error::Materialization("TinyCC musl-v2 retained malformed output".to_string()));
    }
    if !execution.is_file() {
        crate::stagex_mes_lib::write_create_new(&execution, b"")?;
    }
    assert!(binary.is_file());
    assert!(!smoke.join("variadic-negative.o").exists());
    Ok((object, binary, execution))
}

fn require_expected_failure(
    compiler: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), Error> {
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
        }) if exit_code != 0 && !stderr.trim().is_empty() => Ok(()),
        Ok(()) => Err(Error::Materialization("TinyCC musl-v2 accepted malformed source".to_string())),
        Err(error) => Err(Error::Runtime(error)),
    }
}

fn collect_outputs(
    output_root: &Path,
    compiler: &Path,
    object: &Path,
    binary: &Path,
    execution: &Path,
) -> Result<Vec<OutputReport>, Error> {
    let specs = [
        ("tcc-musl-v2", compiler.to_path_buf(), false),
        ("tcc-musl-v2-alias", output_root.join("bin/tcc"), false),
        ("tcc-musl-v2-libtcc1", output_root.join("lib/tcc/libtcc1.a"), false),
        ("tcc-musl-v2-source-tree", output_root.join("share/tcc-source"), true),
        ("tcc-musl-v2-header-tree", output_root.join("lib/tcc/include"), true),
        ("tcc-musl-v2-positive-object", object.to_path_buf(), false),
        ("tcc-musl-v2-positive-binary", binary.to_path_buf(), false),
        ("tcc-musl-v2-positive-execution", execution.to_path_buf(), false),
    ];
    let mut outputs = Vec::with_capacity(OUTPUT_COUNT);
    for (artifact_id, path, is_tree) in specs {
        let (bytes_len, digest_blake3) = if is_tree {
            (
                crate::stagex_musl::tree_bytes_len(&path).map_err(|error| Error::Materialization(error.to_string()))?,
                crate::stagex_musl::tree_digest_blake3(&path)
                    .map_err(|error| Error::Materialization(error.to_string()))?,
            )
        } else {
            let bytes = fs::read(&path)
                .map_err(|error| Error::Materialization(format!("reading TinyCC musl-v2 output: {error}")))?;
            if bytes.len() as u64 > FILE_BYTES_MAX {
                return Err(Error::Materialization(format!("TinyCC musl-v2 output exceeds bound: {artifact_id}")));
            }
            (bytes.len() as u64, blake3::hash(&bytes).to_hex().to_string())
        };
        outputs.push(OutputReport {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len,
            digest_blake3,
        });
    }
    assert_eq!(outputs.len(), OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[OutputReport]) -> Result<(), Error> {
    let mut mismatches = Vec::new();
    for expected in EXPECTED_OUTPUTS {
        let output = outputs
            .iter()
            .find(|output| output.artifact_id == expected.artifact_id)
            .ok_or_else(|| Error::Materialization(format!("TinyCC musl-v2 report lacks {}", expected.artifact_id)))?;
        if output.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, output.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(Error::Materialization(format!(
            "TinyCC musl-v2 output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-tcc-musl-v2-configured-source-v1\0");
    hasher.update(crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(TCC_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0");
    hasher.update(MUSL_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0BOOTSTRAP=1\0HAVE_BITFIELD=1\0HAVE_FLOAT=1\0HAVE_LONG_LONG=1\0HAVE_SETJMP=1\0ONE_SOURCE=1\0");
    hasher.finalize().to_hex().to_string()
}

fn validate_bound_sources() -> Result<(), Error> {
    let recipe_observed = blake3::hash(RECIPE).to_hex().to_string();
    let helper = extract_helper(RECIPE, STDARG_RUNTIME_TARGET)?;
    let helper_observed = blake3::hash(&helper).to_hex().to_string();
    let mut mismatches = Vec::new();
    if recipe_observed != RECIPE_BLAKE3 {
        mismatches.push(format!("recipe={recipe_observed}"));
    }
    if helper_observed != STDARG_RUNTIME_BLAKE3 {
        mismatches.push(format!("stdarg-runtime={helper_observed}"));
    }
    if !mismatches.is_empty() {
        return Err(Error::Materialization(format!(
            "TinyCC musl-v2 source BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert!(!RECIPE.is_empty());
    assert!(!helper.is_empty());
    Ok(())
}

fn extract_helper(source: &[u8], target: &str) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(source)
        .map_err(|error| Error::Materialization(format!("TinyCC musl-v2 recipe is not UTF-8: {error}")))?;
    let marker = format!("cat > {target} <<'EOF'\n");
    let starts = text.match_indices(&marker).map(|(index, _)| index).collect::<Vec<_>>();
    if starts.len() != 1 {
        return Err(Error::Materialization(format!("TinyCC musl-v2 helper {target} occurs {} times", starts.len())));
    }
    let start = starts[0]
        .checked_add(marker.len())
        .ok_or_else(|| Error::Materialization("TinyCC musl-v2 helper start overflow".to_string()))?;
    let relative_end = text[start..]
        .find("\nEOF\n")
        .ok_or_else(|| Error::Materialization(format!("TinyCC musl-v2 helper {target} lacks terminator")))?;
    let end = start
        .checked_add(relative_end)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Error::Materialization("TinyCC musl-v2 helper end overflow".to_string()))?;
    let bytes = source[start..end].to_vec();
    if bytes.is_empty() || !bytes.ends_with(b"\n") {
        return Err(Error::Materialization(format!("TinyCC musl-v2 helper {target} is empty")));
    }
    assert!(end <= source.len());
    assert!(bytes.ends_with(b"\n"));
    Ok(bytes)
}

fn set_owner_read_write(path: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    const OWNER_READ_WRITE_MODE: u32 = 0o600;
    const OWNER_PERMISSION_MASK: u32 = 0o700;
    fs::set_permissions(path, fs::Permissions::from_mode(OWNER_READ_WRITE_MODE))
        .map_err(|error| Error::Materialization(format!("setting owner read/write on {}: {error}", path.display())))?;
    assert!(path.is_file());
    assert_eq!(
        fs::metadata(path).map_err(|error| Error::Materialization(error.to_string()))?.permissions().mode()
            & OWNER_PERMISSION_MASK,
        OWNER_READ_WRITE_MODE
    );
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(Error::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(Error::Materialization(format!("{label} is empty")));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, Error> {
    crate::stagex_mes_lib::utf8_absolute(path, label).map(str::to_string).map_err(Error::Runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_ENV: &str = "MANTLE_STAGE_X_TINYCC27_SOURCE_ROOT";
    const TINYCC26_ENV: &str = "MANTLE_STAGE_X_TINYCC26_ROOT";
    const PREP_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_PREP_ROOT";
    const MUSL_ENV: &str = "MANTLE_STAGE_X_MUSL_PASS2_ROOT";
    const SCRATCH_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_V2_SCRATCH";

    #[test]
    fn source_bindings_are_exact() {
        validate_bound_sources().unwrap();
        assert_eq!(configured_source_digest_blake3(), CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn missing_helper_is_rejected() {
        let error = extract_helper(RECIPE, "missing.c").unwrap_err();
        assert!(error.to_string().contains("occurs 0 times"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained StageX compiler and libc roots"]
    fn derives_retained_inventory() {
        let source = PathBuf::from(std::env::var(SOURCE_ENV).unwrap());
        let tinycc26 = PathBuf::from(std::env::var(TINYCC26_ENV).unwrap());
        let prep = PathBuf::from(std::env::var(PREP_ENV).unwrap());
        let musl = PathBuf::from(std::env::var(MUSL_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(SCRATCH_ENV).unwrap());
        let report = derive_inventory(InventoryRequest {
            tinycc27_source_root: &source,
            tinycc26_root: &tinycc26,
            tcc_musl_prep_root: &prep,
            musl_pass2_root: &musl,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
