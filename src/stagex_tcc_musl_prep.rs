use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

pub(crate) const TCC_MUSL_PREP_RECIPE_ARTIFACT_ID: &str = "tcc-musl-prep-recipe-source";
pub(crate) const TCC_MUSL_PREP_RECIPE_BLAKE3: &str = "88f0a242b6e53d3618383af6d9fce83bbacaa4cf4f2403bba56baf9651901b4e";
const TCC_MUSL_PREP_RECIPE: &[u8] = include_bytes!("../bootstrap/tcc-musl-prep.ncl");
const TCC_MUSL_PREP_REPORT_FORMAT: &str = "mantle-stagex-tcc-musl-prep-inventory-v1";
const TCC_MUSL_PREP_NON_CLAIM: &str = "this inventory binds the Mes-linked TinyCC compiler prepared for the first musl handoff and bounded object-compilation observations only; it does not prove musl or provider admission";
const TCC_MUSL_PREP_LOGICAL_PREFIX: &str = "/stagex/tcc-musl-prep";
const MUSL_LOGICAL_PREFIX: &str = "/stagex/musl-1.1.24";
const TCC_MUSL_PREP_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TCC_MUSL_PREP_BUILD_COMMAND_COUNT: u32 = 2;
const TCC_MUSL_PREP_SMOKE_COMMAND_COUNT: u32 = 3;
const TCC_MUSL_PREP_OUTPUT_COUNT: usize = 7;
const TCC_MUSL_PREP_TREE_ENTRY_COUNT_MAX: usize = 8_192;
pub(crate) const TCC_MUSL_PREP_CONFIGURED_SOURCE_BLAKE3: &str =
    "29489150a78ce433dac24acfaf58ec66a4cd37d3eb93e223c5b9150f5c71a0aa";
pub(crate) const TCC_MUSL_PREP_FINAL_BLAKE3: &str = "4add5639d2016d79b1ac258aa502098fc941354ded0acd91bf8cafea42240a77";
const TCC_MUSL_PREP_POSITIVE_OBJECT_BLAKE3: &str = "41c08a115f275f00099aa397872090612e853be0fb5f13ba55ddb89737e42377";
const POSITIVE_SMOKE_SOURCE: &[u8] = b"int bridge_value(void) { return 24; }\n";
const MALFORMED_SMOKE_SOURCE: &[u8] = b"int broken( {\n";
const FIXED_WIDTH_ALIAS_INSERTION: &str = "#include <inttypes.h>\n#define uint8_t unsigned char\n#define uint16_t unsigned short\n#define uint32_t unsigned int\n#define uint64_t unsigned long long\n#define int32_t int\n#define int64_t long long\n#define CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES 1\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TccMuslPrepExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const TCC_MUSL_PREP_EXPECTED_OUTPUTS: [TccMuslPrepExpectedOutput; TCC_MUSL_PREP_OUTPUT_COUNT] = [
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep",
        digest_blake3: TCC_MUSL_PREP_FINAL_BLAKE3,
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-alias",
        digest_blake3: TCC_MUSL_PREP_FINAL_BLAKE3,
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-libc",
        digest_blake3: "bd46a4a350e17ec4a7870be6c6dea82f7327c513ee4e2720bebb7b6e26f20d85",
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-crt1",
        digest_blake3: "d3fec15bd12fd72fdbdf4248d6df6b21a44cfb6a3e2c9d787a0d081ff7143df1",
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-libtcc1",
        digest_blake3: "0e8b75458ad70ab03142b27a3014af68f57c03004f4d22a65702e60d531140e9",
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-headers",
        digest_blake3: "2577c43f136d52c8d7b61ba8f7cab614b31898586d3fab634bebe6e1fd14cb7d",
    },
    TccMuslPrepExpectedOutput {
        artifact_id: "tcc-musl-prep-positive-object",
        digest_blake3: TCC_MUSL_PREP_POSITIVE_OBJECT_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMuslPrepOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMuslPrepInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<TccMuslPrepOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TccMuslPrepInventoryRequest<'a> {
    pub tinycc27_source_root: &'a Path,
    pub tinycc27_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum TccMuslPrepError {
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for TccMuslPrepError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Materialization(message) => write!(formatter, "TinyCC musl prep materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "TinyCC musl prep runtime failed: {error}"),
        }
    }
}

impl std::error::Error for TccMuslPrepError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for TccMuslPrepError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for TccMuslPrepError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tinycc27::Tinycc27Error> for TccMuslPrepError {
    fn from(error: crate::stagex_tinycc27::Tinycc27Error) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); 1] {
    [(TCC_MUSL_PREP_RECIPE_ARTIFACT_ID, TCC_MUSL_PREP_RECIPE_BLAKE3)]
}

pub(crate) fn derive_tcc_musl_prep_inventory(
    request: TccMuslPrepInventoryRequest<'_>,
) -> Result<TccMuslPrepInventoryReport, TccMuslPrepError> {
    validate_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| TccMuslPrepError::Materialization(format!("creating TinyCC musl prep scratch: {error}")))?;
    let source_root = request.scratch_dir.join("tcc-0.9.27");
    crate::stagex_mes_lib::copy_tree_bounded(request.tinycc27_source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    let patched_files = crate::stagex_tinycc27::apply_tinycc27_source_patch(&source_root)?;
    let patched_digest = crate::stagex_tinycc27::patched_tinycc27_source_digest(&source_root, &patched_files)?;
    if patched_digest != crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3 {
        return Err(TccMuslPrepError::Materialization(format!(
            "TinyCC musl prep patched source mismatch: expected {}, observed {patched_digest}",
            crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3
        )));
    }
    apply_musl_prep_adjustments(&source_root)?;
    fs::write(source_root.join("config.h"), [])
        .map_err(|error| TccMuslPrepError::Materialization(format!("writing TinyCC musl prep config.h: {error}")))?;
    let output_root = request.scratch_dir.join("output");
    materialize_runtime(&request, &output_root)?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let compiler = build_tcc_musl_prep(&request, &source_root, &output_root)?;
    let positive_object = run_smokes(&request, &compiler, &output_root)?;
    let outputs = collect_outputs(&output_root, &compiler, &positive_object)?;
    validate_expected_outputs(&outputs)?;
    let report = TccMuslPrepInventoryReport {
        format: TCC_MUSL_PREP_REPORT_FORMAT,
        configured_source_digest_blake3,
        build_command_count: TCC_MUSL_PREP_BUILD_COMMAND_COUNT,
        smoke_command_count: TCC_MUSL_PREP_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: TCC_MUSL_PREP_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tcc-musl-prep-inventory.json"),
        &serde_json::to_vec_pretty(&report).map_err(|error| {
            TccMuslPrepError::Materialization(format!("serializing TinyCC musl prep report: {error}"))
        })?,
    )?;
    assert_eq!(report.outputs.len(), TCC_MUSL_PREP_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

pub(crate) fn apply_musl_prep_adjustments(source_root: &Path) -> Result<(), TccMuslPrepError> {
    crate::stagex_tinycc::replace_required_text(
        &source_root.join("elf.h"),
        "#include <inttypes.h>\n",
        FIXED_WIDTH_ALIAS_INSERTION,
    )?;
    let elf = fs::read_to_string(source_root.join("elf.h"))
        .map_err(|error| TccMuslPrepError::Materialization(format!("reading adjusted TinyCC elf.h: {error}")))?;
    if elf.matches("CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES").count() != 1 {
        return Err(TccMuslPrepError::Materialization(
            "TinyCC musl prep fixed-width aliases are missing or duplicated".to_string(),
        ));
    }
    assert!(elf.contains("#define uint64_t unsigned long long"));
    assert_eq!(elf.matches("CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES").count(), 1);
    Ok(())
}

fn validate_inputs(request: &TccMuslPrepInventoryRequest<'_>) -> Result<(), TccMuslPrepError> {
    if request.scratch_dir.exists() {
        return Err(TccMuslPrepError::Materialization(format!(
            "create-new TinyCC musl prep scratch exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("TinyCC 0.9.27 source", request.tinycc27_source_root),
        ("TinyCC 0.9.27 runtime", request.tinycc27_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(TccMuslPrepError::Materialization(format!(
                "{label} is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.tinycc27_root.join("bin/tcc"),
        crate::stagex_tinycc27::TINYCC27_FINAL_BLAKE3,
        "TinyCC 0.9.27 compiler",
    )?;
    validate_bound_bytes(TCC_MUSL_PREP_RECIPE, TCC_MUSL_PREP_RECIPE_BLAKE3, "TinyCC musl prep recipe")?;
    assert!(request.tinycc27_source_root.join("tcc.c").is_file());
    assert!(request.tinycc27_root.join("lib/mes/libc.a").is_file());
    Ok(())
}

fn materialize_runtime(request: &TccMuslPrepInventoryRequest<'_>, output_root: &Path) -> Result<(), TccMuslPrepError> {
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| TccMuslPrepError::Materialization(format!("creating TinyCC musl prep bin: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(&request.tinycc27_root.join("lib/mes"), &output_root.join("lib/mes"))?;
    crate::stagex_mes_lib::copy_tree_bounded(
        &request.tinycc27_root.join("include/mes"),
        &output_root.join("include/mes"),
    )?;
    assert!(output_root.join("lib/mes/libc.a").is_file());
    assert!(output_root.join("include/mes/stdio.h").is_file());
    Ok(())
}

fn build_tcc_musl_prep(
    request: &TccMuslPrepInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, TccMuslPrepError> {
    let predecessor = request.tinycc27_root.join("bin/tcc");
    let object = source_root.join("tcc-musl-prep.o");
    let compile_args = compile_args(request.tinycc27_root, &object)?;
    crate::stagex_mes_lib::run_bounded_process(
        &predecessor,
        &compile_args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-prep-compile.stderr.txt"),
    )?;
    validate_nonempty_file(&object, "TinyCC musl prep object")?;
    let compiler = output_root.join("bin/tcc-musl-prep");
    let link_args = link_args(request.tinycc27_root, &object, &compiler)?;
    crate::stagex_mes_lib::run_bounded_process(
        &predecessor,
        &link_args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-prep-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&compiler)?;
    crate::stagex_mes_lib::copy_file_exact(&compiler, &output_root.join("bin/tcc"))?;
    crate::stagex_tinycc::set_owner_executable(&output_root.join("bin/tcc"))?;
    assert!(object.is_file());
    assert!(compiler.is_file());
    Ok(compiler)
}

fn compile_args(tinycc27_root: &Path, object: &Path) -> Result<Vec<String>, TccMuslPrepError> {
    let include = tinycc27_root.join("include/mes");
    let args = vec![
        "-v".to_string(),
        "-c".to_string(),
        "-o".to_string(),
        absolute_utf8(object, "TinyCC musl prep object")?,
        "-I".to_string(),
        ".".to_string(),
        "-I".to_string(),
        absolute_utf8(&include, "TinyCC musl prep include")?,
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
        format!("CONFIG_TCCDIR=\"{TCC_MUSL_PREP_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        format!("CONFIG_TCC_CRTPREFIX=\"{MUSL_LOGICAL_PREFIX}/lib\""),
        "-D".to_string(),
        "CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\"".to_string(),
        "-D".to_string(),
        format!(
            "CONFIG_TCC_LIBPATHS=\"{TCC_MUSL_PREP_LOGICAL_PREFIX}/lib/mes:{TCC_MUSL_PREP_LOGICAL_PREFIX}/lib/mes/tcc:{MUSL_LOGICAL_PREFIX}/lib\""
        ),
        "-D".to_string(),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{MUSL_LOGICAL_PREFIX}/include\""),
        "-D".to_string(),
        "CONFIG_SYSROOT=\"/\"".to_string(),
        "-D".to_string(),
        format!("TCC_LIBGCC=\"{TCC_MUSL_PREP_LOGICAL_PREFIX}/lib/mes/libc.a\""),
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
    ];
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(args.len() > 1);
    Ok(args)
}

fn link_args(tinycc27_root: &Path, object: &Path, output: &Path) -> Result<Vec<String>, TccMuslPrepError> {
    let libdir = tinycc27_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let args = vec![
        "-v".to_string(),
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        absolute_utf8(&runtime, "TinyCC musl prep predecessor runtime")?,
        "-o".to_string(),
        absolute_utf8(output, "TinyCC musl prep compiler")?,
        absolute_utf8(object, "TinyCC musl prep object")?,
        absolute_utf8(&libdir.join("va_list.o"), "TinyCC musl prep va_list")?,
        absolute_utf8(&libdir.join("crt1.o"), "TinyCC musl prep crt1")?,
        absolute_utf8(&libdir.join("libc.a"), "TinyCC musl prep libc")?,
        absolute_utf8(&runtime.join("libtcc1.a"), "TinyCC musl prep libtcc1")?,
    ];
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(args.len() > 1);
    Ok(args)
}

fn run_smokes(
    request: &TccMuslPrepInventoryRequest<'_>,
    compiler: &Path,
    output_root: &Path,
) -> Result<PathBuf, TccMuslPrepError> {
    let smoke_root = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke_root)
        .map_err(|error| TccMuslPrepError::Materialization(format!("creating TinyCC musl prep smoke: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("positive.c"), POSITIVE_SMOKE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke_root.join("malformed.c"), MALFORMED_SMOKE_SOURCE)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-version"],
        &smoke_root,
        &empty_env,
        &smoke_root.join("version.stderr.txt"),
    )?;
    let include = absolute_utf8(&output_root.join("include/mes"), "TinyCC musl prep smoke include")?;
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-I", &include, "-c", "-o", "positive.o", "positive.c"],
        &smoke_root,
        &empty_env,
        &smoke_root.join("positive.stderr.txt"),
    )?;
    require_expected_compile_failure(
        compiler,
        &["-I", &include, "-c", "-o", "malformed.o", "malformed.c"],
        &smoke_root,
        &smoke_root.join("malformed.stderr.txt"),
    )?;
    let positive = smoke_root.join("positive.o");
    validate_nonempty_file(&positive, "TinyCC musl prep positive object")?;
    if smoke_root.join("malformed.o").exists() {
        return Err(TccMuslPrepError::Materialization(
            "TinyCC musl prep malformed smoke produced an object".to_string(),
        ));
    }
    assert!(positive.is_file());
    assert!(!smoke_root.join("malformed.o").exists());
    Ok(positive)
}

fn require_expected_compile_failure(
    compiler: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), TccMuslPrepError> {
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
        Ok(()) => {
            Err(TccMuslPrepError::Materialization("TinyCC musl prep accepted malformed smoke source".to_string()))
        }
        Err(error) => Err(TccMuslPrepError::Runtime(error)),
    }
}

fn collect_outputs(
    output_root: &Path,
    compiler: &Path,
    positive_object: &Path,
) -> Result<Vec<TccMuslPrepOutputReport>, TccMuslPrepError> {
    let specs = [
        ("tcc-musl-prep", compiler.to_path_buf(), false),
        ("tcc-musl-prep-alias", output_root.join("bin/tcc"), false),
        ("tcc-musl-prep-libc", output_root.join("lib/mes/libc.a"), false),
        ("tcc-musl-prep-crt1", output_root.join("lib/mes/crt1.o"), false),
        ("tcc-musl-prep-libtcc1", output_root.join("lib/mes/tcc/libtcc1.a"), false),
        ("tcc-musl-prep-headers", output_root.join("include/mes"), true),
        ("tcc-musl-prep-positive-object", positive_object.to_path_buf(), false),
    ];
    let mut outputs = Vec::with_capacity(TCC_MUSL_PREP_OUTPUT_COUNT);
    for (artifact_id, path, is_tree) in specs {
        let (bytes_len, digest_blake3) = if is_tree {
            let digest = tree_digest_blake3(&path)?;
            (tree_bytes_len(&path)?, digest)
        } else {
            let bytes = crate::stagex_mes_lib::read_bounded_file(&path, TCC_MUSL_PREP_FILE_BYTES_MAX, artifact_id)?;
            (
                u64::try_from(bytes.len())
                    .map_err(|_| TccMuslPrepError::Materialization("output size overflow".to_string()))?,
                blake3::hash(&bytes).to_hex().to_string(),
            )
        };
        outputs.push(TccMuslPrepOutputReport {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len,
            digest_blake3,
        });
    }
    assert_eq!(outputs.len(), TCC_MUSL_PREP_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[TccMuslPrepOutputReport]) -> Result<(), TccMuslPrepError> {
    let mut mismatches = Vec::new();
    for expected in TCC_MUSL_PREP_EXPECTED_OUTPUTS {
        let output = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            TccMuslPrepError::Materialization(format!("TinyCC musl prep report lacks {}", expected.artifact_id))
        })?;
        if output.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, output.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(TccMuslPrepError::Materialization(format!(
            "TinyCC musl prep output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), TCC_MUSL_PREP_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-tcc-musl-prep-configured-source-v1\0");
    hasher.update(crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(TCC_MUSL_PREP_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(TCC_MUSL_PREP_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0");
    hasher.update(MUSL_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0");
    hasher.update(FIXED_WIDTH_ALIAS_INSERTION.as_bytes());
    hasher.update(b"\0BOOTSTRAP=1\0HAVE_BITFIELD=1\0HAVE_FLOAT=1\0HAVE_LONG_LONG=1\0HAVE_SETJMP=1\0");
    hasher.finalize().to_hex().to_string()
}

fn tree_digest_blake3(root: &Path) -> Result<String, TccMuslPrepError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-tcc-musl-prep-tree-v1\0");
    for path in bounded_tree_paths(root)? {
        let relative = path
            .strip_prefix(root)
            .map_err(|error| TccMuslPrepError::Materialization(format!("finding header relative path: {error}")))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| TccMuslPrepError::Materialization("header path is not UTF-8".to_string()))?;
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            TccMuslPrepError::Materialization(format!("reading header metadata {}: {error}", path.display()))
        })?;
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        if metadata.file_type().is_file() {
            hasher.update(b"f\0");
            hasher.update(&fs::read(&path).map_err(|error| {
                TccMuslPrepError::Materialization(format!("reading header {}: {error}", path.display()))
            })?);
        } else if metadata.file_type().is_symlink() {
            hasher.update(b"l\0");
            let target = fs::read_link(&path).map_err(|error| {
                TccMuslPrepError::Materialization(format!("reading header link {}: {error}", path.display()))
            })?;
            hasher.update(target.as_os_str().as_encoded_bytes());
        } else if metadata.file_type().is_dir() {
            hasher.update(b"d\0");
        } else {
            return Err(TccMuslPrepError::Materialization(format!("unsupported header entry: {}", path.display())));
        }
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(root.is_dir());
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok(digest)
}

fn tree_bytes_len(root: &Path) -> Result<u64, TccMuslPrepError> {
    let mut total = 0_u64;
    for path in bounded_tree_paths(root)? {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            TccMuslPrepError::Materialization(format!("reading tree metadata {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_file() {
            total = total
                .checked_add(metadata.len())
                .ok_or_else(|| TccMuslPrepError::Materialization("tree byte count overflow".to_string()))?;
        }
    }
    if total == 0 || total > TCC_MUSL_PREP_FILE_BYTES_MAX {
        return Err(TccMuslPrepError::Materialization(format!("header tree byte count is outside bounds: {total}")));
    }
    assert!(root.is_dir());
    assert!(total > 0);
    Ok(total)
}

fn bounded_tree_paths(root: &Path) -> Result<Vec<PathBuf>, TccMuslPrepError> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| {
                TccMuslPrepError::Materialization(format!("reading tree directory {}: {error}", directory.display()))
            })?
            .map(|entry| {
                entry
                    .map(|value| value.path())
                    .map_err(|error| TccMuslPrepError::Materialization(format!("reading tree entry: {error}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        children.sort();
        for child in children.into_iter().rev() {
            let metadata = fs::symlink_metadata(&child).map_err(|error| {
                TccMuslPrepError::Materialization(format!("reading tree entry {}: {error}", child.display()))
            })?;
            if metadata.file_type().is_dir() {
                pending.push(child.clone());
            }
            paths.push(child);
            if paths.len() > TCC_MUSL_PREP_TREE_ENTRY_COUNT_MAX {
                return Err(TccMuslPrepError::Materialization(format!(
                    "tree exceeds {} entries",
                    TCC_MUSL_PREP_TREE_ENTRY_COUNT_MAX
                )));
            }
        }
    }
    paths.sort();
    assert!(paths.len() <= TCC_MUSL_PREP_TREE_ENTRY_COUNT_MAX);
    assert!(paths.iter().all(|path| path.starts_with(root)));
    Ok(paths)
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, TccMuslPrepError> {
    crate::stagex_mes_lib::utf8_absolute(path, label)
        .map(str::to_string)
        .map_err(TccMuslPrepError::Runtime)
}

fn validate_bound_bytes(bytes: &[u8], expected: &str, label: &str) -> Result<(), TccMuslPrepError> {
    let observed = blake3::hash(bytes).to_hex().to_string();
    if observed != expected {
        return Err(TccMuslPrepError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), TccMuslPrepError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TCC_MUSL_PREP_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(TccMuslPrepError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), TccMuslPrepError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TCC_MUSL_PREP_FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(TccMuslPrepError::Materialization(format!("{label} is empty")));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_TINYCC27_SOURCE_ENV: &str = "MANTLE_STAGE_X_TINYCC27_SOURCE_ROOT";
    const RETAINED_TINYCC27_ROOT_ENV: &str = "MANTLE_STAGE_X_TINYCC27_ROOT";
    const RETAINED_SCRATCH_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_PREP_SCRATCH";

    #[test]
    fn configured_source_digest_is_stable() {
        let digest = configured_source_digest_blake3();
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert_eq!(digest, TCC_MUSL_PREP_CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn substituted_recipe_is_rejected() {
        let error = validate_bound_bytes(b"substituted", TCC_MUSL_PREP_RECIPE_BLAKE3, "test recipe").unwrap_err();
        assert!(error.to_string().contains("BLAKE3 mismatch"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires retained TinyCC 0.9.27 source and runtime"]
    fn derives_retained_tcc_musl_prep_inventory() {
        let source = PathBuf::from(std::env::var(RETAINED_TINYCC27_SOURCE_ENV).unwrap());
        let runtime = PathBuf::from(std::env::var(RETAINED_TINYCC27_ROOT_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SCRATCH_ENV).unwrap());
        let report = derive_tcc_musl_prep_inventory(TccMuslPrepInventoryRequest {
            tinycc27_source_root: &source,
            tinycc27_root: &runtime,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, TCC_MUSL_PREP_BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
