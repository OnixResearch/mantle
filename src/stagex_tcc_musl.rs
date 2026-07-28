use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

pub(crate) const TCC_MUSL_RECIPE_ARTIFACT_ID: &str = "tcc-musl-recipe-source";
pub(crate) const TCC_MUSL_RECIPE_BLAKE3: &str = "8692c674f19ec7dcf7643365a984dbac570418dbb81989874f078792dd2668c4";
const TCC_MUSL_RECIPE: &[u8] = include_bytes!("../bootstrap/tcc-musl.ncl");
const TCC_MUSL_REPORT_FORMAT: &str = "mantle-stagex-tcc-musl-inventory-v1";
const TCC_MUSL_NON_CLAIM: &str = "this inventory binds the first TinyCC compiler configured for reduced musl and bounded compile/link observations only; it does not prove a complete musl toolchain or provider admission";
const TCC_MUSL_LOGICAL_PREFIX: &str = "/stagex/tcc-musl";
const MUSL_LOGICAL_PREFIX: &str = "/stagex/musl-1.1.24";
const TCC_MUSL_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TCC_MUSL_BUILD_COMMAND_COUNT: u32 = 2;
const TCC_MUSL_SMOKE_COMMAND_COUNT: u32 = 4;
const TCC_MUSL_OUTPUT_COUNT: usize = 5;
pub(crate) const TCC_MUSL_CONFIGURED_SOURCE_BLAKE3: &str =
    "679e40759989e2bf0886a430ac3cb00ef22b045dd18fa50fdeddba44230c7674";
pub(crate) const TCC_MUSL_FINAL_BLAKE3: &str = "053f36a66503797b3aaee40d2b0962dc643554e4ecf35d00ee2b7f947601f2af";
const TCC_MUSL_POSITIVE_OBJECT_BLAKE3: &str = "f7d6cd4379debd6239bbbcfffdd602d87ad9e8cfabe322902a85fa5ace902fc6";
const TCC_MUSL_POSITIVE_BINARY_BLAKE3: &str = "92ebe6be234ab289a07f5560ce0f25b1a696917d4515d2ec92c330fa0cd3a8a4";
const POSITIVE_SMOKE_SOURCE: &[u8] = b"int main(void) { return 0; }\n";
const MALFORMED_SMOKE_SOURCE: &[u8] = b"int broken( {\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TccMuslExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const TCC_MUSL_EXPECTED_OUTPUTS: [TccMuslExpectedOutput; TCC_MUSL_OUTPUT_COUNT] = [
    TccMuslExpectedOutput {
        artifact_id: "tcc-musl",
        digest_blake3: TCC_MUSL_FINAL_BLAKE3,
    },
    TccMuslExpectedOutput {
        artifact_id: "tcc-musl-alias",
        digest_blake3: TCC_MUSL_FINAL_BLAKE3,
    },
    TccMuslExpectedOutput {
        artifact_id: "tcc-musl-libtcc1",
        digest_blake3: "0e8b75458ad70ab03142b27a3014af68f57c03004f4d22a65702e60d531140e9",
    },
    TccMuslExpectedOutput {
        artifact_id: "tcc-musl-positive-object",
        digest_blake3: TCC_MUSL_POSITIVE_OBJECT_BLAKE3,
    },
    TccMuslExpectedOutput {
        artifact_id: "tcc-musl-positive-binary",
        digest_blake3: TCC_MUSL_POSITIVE_BINARY_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMuslOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMuslInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<TccMuslOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TccMuslInventoryRequest<'a> {
    pub tinycc27_source_root: &'a Path,
    pub tcc_musl_prep_root: &'a Path,
    pub musl_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum TccMuslError {
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for TccMuslError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Materialization(message) => write!(formatter, "musl-linked TinyCC materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "musl-linked TinyCC runtime failed: {error}"),
        }
    }
}

impl std::error::Error for TccMuslError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for TccMuslError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_tinycc::TinyccError> for TccMuslError {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tinycc27::Tinycc27Error> for TccMuslError {
    fn from(error: crate::stagex_tinycc27::Tinycc27Error) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tcc_musl_prep::TccMuslPrepError> for TccMuslError {
    fn from(error: crate::stagex_tcc_musl_prep::TccMuslPrepError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); 1] {
    [(TCC_MUSL_RECIPE_ARTIFACT_ID, TCC_MUSL_RECIPE_BLAKE3)]
}

pub(crate) fn derive_tcc_musl_inventory(
    request: TccMuslInventoryRequest<'_>,
) -> Result<TccMuslInventoryReport, TccMuslError> {
    validate_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| TccMuslError::Materialization(format!("creating musl-linked TinyCC scratch: {error}")))?;
    let source_root = request.scratch_dir.join("tcc-0.9.27");
    crate::stagex_mes_lib::copy_tree_bounded(request.tinycc27_source_root, &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    let patched = crate::stagex_tinycc27::apply_tinycc27_source_patch(&source_root)?;
    let patch_digest = crate::stagex_tinycc27::patched_tinycc27_source_digest(&source_root, &patched)?;
    if patch_digest != crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3 {
        return Err(TccMuslError::Materialization(format!(
            "musl-linked TinyCC patch mismatch: expected {}, observed {patch_digest}",
            crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3
        )));
    }
    crate::stagex_tcc_musl_prep::apply_musl_prep_adjustments(&source_root)?;
    fs::write(source_root.join("config.h"), [])
        .map_err(|error| TccMuslError::Materialization(format!("writing musl-linked TinyCC config.h: {error}")))?;
    let configured_source_digest_blake3 = configured_source_digest_blake3();
    let output_root = request.scratch_dir.join("output");
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| TccMuslError::Materialization(format!("creating musl-linked TinyCC output: {error}")))?;
    fs::create_dir_all(output_root.join("lib/tcc"))
        .map_err(|error| TccMuslError::Materialization(format!("creating musl-linked TinyCC runtime: {error}")))?;
    let compiler = build_compiler(&request, &source_root, &output_root)?;
    crate::stagex_mes_lib::copy_file_exact(
        &request.tcc_musl_prep_root.join("lib/mes/tcc/libtcc1.a"),
        &output_root.join("lib/tcc/libtcc1.a"),
    )?;
    let (positive_object, positive_binary) = run_smokes(&request, &compiler, &output_root)?;
    let outputs = collect_outputs(&output_root, &compiler, &positive_object, &positive_binary)?;
    validate_expected_outputs(&outputs)?;
    let report = TccMuslInventoryReport {
        format: TCC_MUSL_REPORT_FORMAT,
        configured_source_digest_blake3,
        build_command_count: TCC_MUSL_BUILD_COMMAND_COUNT,
        smoke_command_count: TCC_MUSL_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: TCC_MUSL_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tcc-musl-inventory.json"),
        &serde_json::to_vec_pretty(&report).map_err(|error| {
            TccMuslError::Materialization(format!("serializing musl-linked TinyCC report: {error}"))
        })?,
    )?;
    assert_eq!(report.outputs.len(), TCC_MUSL_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inputs(request: &TccMuslInventoryRequest<'_>) -> Result<(), TccMuslError> {
    if request.scratch_dir.exists() {
        return Err(TccMuslError::Materialization(format!(
            "create-new musl-linked TinyCC scratch exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("TinyCC 0.9.27 source", request.tinycc27_source_root),
        ("TinyCC musl-prep", request.tcc_musl_prep_root),
        ("reduced musl", request.musl_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(TccMuslError::Materialization(format!(
                "{label} is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(
        &request.tcc_musl_prep_root.join("bin/tcc-musl-prep"),
        crate::stagex_tcc_musl_prep::TCC_MUSL_PREP_FINAL_BLAKE3,
        "TinyCC musl-prep compiler",
    )?;
    validate_file_digest(
        &request.musl_root.join("lib/libc.a"),
        crate::stagex_musl::MUSL_LIBC_BLAKE3,
        "reduced musl libc",
    )?;
    validate_bound_recipe()?;
    assert!(request.tinycc27_source_root.join("tcc.c").is_file());
    assert!(request.musl_root.join("include/stdio.h").is_file());
    Ok(())
}

fn build_compiler(
    request: &TccMuslInventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<PathBuf, TccMuslError> {
    let predecessor = request.tcc_musl_prep_root.join("bin/tcc-musl-prep");
    let object = source_root.join("tcc-musl.o");
    crate::stagex_mes_lib::run_bounded_process(
        &predecessor,
        &compile_args(request, &object)?,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-compile.stderr.txt"),
    )?;
    validate_nonempty_file(&object, "musl-linked TinyCC object")?;
    let compiler = output_root.join("bin/tcc-0.9.27-musl");
    crate::stagex_mes_lib::run_bounded_process(
        &predecessor,
        &link_args(request, &object, &compiler)?,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-musl-link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&compiler)?;
    crate::stagex_mes_lib::copy_file_exact(&compiler, &output_root.join("bin/tcc"))?;
    crate::stagex_tinycc::set_owner_executable(&output_root.join("bin/tcc"))?;
    assert!(object.is_file());
    assert!(compiler.is_file());
    Ok(compiler)
}

fn compile_args(request: &TccMuslInventoryRequest<'_>, object: &Path) -> Result<Vec<String>, TccMuslError> {
    let include = request.tcc_musl_prep_root.join("include/mes");
    let values = [
        "BOOTSTRAP=1".to_string(),
        "HAVE_BITFIELD=1".to_string(),
        "HAVE_FLOAT=1".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "HAVE_SETJMP=1".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        format!("CONFIG_TCCDIR=\"{TCC_MUSL_LOGICAL_PREFIX}/lib/tcc\""),
        format!("CONFIG_TCC_CRTPREFIX=\"{MUSL_LOGICAL_PREFIX}/lib\""),
        "CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\"".to_string(),
        format!("CONFIG_TCC_LIBPATHS=\"{MUSL_LOGICAL_PREFIX}/lib:{TCC_MUSL_LOGICAL_PREFIX}/lib/tcc\""),
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
        absolute_utf8(object, "musl-linked TinyCC object")?,
        "-I".to_string(),
        ".".to_string(),
        "-I".to_string(),
        absolute_utf8(&include, "musl-linked TinyCC include")?,
    ];
    for value in values {
        args.push("-D".to_string());
        args.push(value);
    }
    args.push("tcc.c".to_string());
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(args.len() > 1);
    Ok(args)
}

fn link_args(request: &TccMuslInventoryRequest<'_>, object: &Path, output: &Path) -> Result<Vec<String>, TccMuslError> {
    let libdir = request.tcc_musl_prep_root.join("lib/mes");
    let runtime = libdir.join("tcc");
    let args = vec![
        "-v".to_string(),
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-B".to_string(),
        absolute_utf8(&runtime, "musl-linked TinyCC predecessor runtime")?,
        "-o".to_string(),
        absolute_utf8(output, "musl-linked TinyCC output")?,
        absolute_utf8(&libdir.join("crt1.o"), "musl-linked TinyCC Mes crt1")?,
        absolute_utf8(&libdir.join("crti.o"), "musl-linked TinyCC Mes crti")?,
        absolute_utf8(object, "musl-linked TinyCC object")?,
        absolute_utf8(&libdir.join("libc.a"), "musl-linked TinyCC Mes libc")?,
        absolute_utf8(&runtime.join("libtcc1.a"), "musl-linked TinyCC Mes libtcc1")?,
        absolute_utf8(&libdir.join("crtn.o"), "musl-linked TinyCC Mes crtn")?,
    ];
    assert!(args.iter().all(|argument| !argument.is_empty()));
    assert!(args.len() > 1);
    Ok(args)
}

fn run_smokes(
    request: &TccMuslInventoryRequest<'_>,
    compiler: &Path,
    output_root: &Path,
) -> Result<(PathBuf, PathBuf), TccMuslError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| TccMuslError::Materialization(format!("creating musl-linked TinyCC smoke: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("positive.c"), POSITIVE_SMOKE_SOURCE)?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("malformed.c"), MALFORMED_SMOKE_SOURCE)?;
    let empty_env = BTreeMap::<String, String>::new();
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &["-version"],
        &smoke,
        &empty_env,
        &smoke.join("version.stderr.txt"),
    )?;
    let object = smoke.join("positive.o");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &[
            "-c",
            "-o",
            &absolute_utf8(&object, "musl-linked TinyCC smoke object")?,
            "positive.c",
        ],
        &smoke,
        &empty_env,
        &smoke.join("positive-compile.stderr.txt"),
    )?;
    let binary = smoke.join("positive");
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &[
            "-static",
            "-nostdlib",
            "-o",
            &absolute_utf8(&binary, "musl-linked TinyCC smoke binary")?,
            &absolute_utf8(&request.musl_root.join("lib/crt1.o"), "musl-linked TinyCC smoke crt1")?,
            &absolute_utf8(&object, "musl-linked TinyCC smoke object")?,
            &absolute_utf8(&request.musl_root.join("lib/libc.a"), "musl-linked TinyCC smoke libc")?,
            &absolute_utf8(&output_root.join("lib/tcc/libtcc1.a"), "musl-linked TinyCC smoke libtcc1")?,
        ],
        &smoke,
        &empty_env,
        &smoke.join("positive-link.stderr.txt"),
    )?;
    require_expected_failure(
        compiler,
        &["-c", "-o", "malformed.o", "malformed.c"],
        &smoke,
        &smoke.join("malformed.stderr.txt"),
    )?;
    validate_nonempty_file(&object, "musl-linked TinyCC smoke object")?;
    validate_nonempty_file(&binary, "musl-linked TinyCC smoke binary")?;
    assert!(!smoke.join("malformed.o").exists());
    Ok((object, binary))
}

fn require_expected_failure(
    compiler: &Path,
    args: &[&str],
    current_dir: &Path,
    stderr_path: &Path,
) -> Result<(), TccMuslError> {
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
        Ok(()) => Err(TccMuslError::Materialization("musl-linked TinyCC accepted malformed source".to_string())),
        Err(error) => Err(TccMuslError::Runtime(error)),
    }
}

fn collect_outputs(
    output_root: &Path,
    compiler: &Path,
    object: &Path,
    binary: &Path,
) -> Result<Vec<TccMuslOutputReport>, TccMuslError> {
    let specs = [
        ("tcc-musl", compiler.to_path_buf()),
        ("tcc-musl-alias", output_root.join("bin/tcc")),
        ("tcc-musl-libtcc1", output_root.join("lib/tcc/libtcc1.a")),
        ("tcc-musl-positive-object", object.to_path_buf()),
        ("tcc-musl-positive-binary", binary.to_path_buf()),
    ];
    let mut outputs = Vec::with_capacity(TCC_MUSL_OUTPUT_COUNT);
    for (artifact_id, path) in specs {
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, TCC_MUSL_FILE_BYTES_MAX, artifact_id)?;
        outputs.push(TccMuslOutputReport {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| TccMuslError::Materialization("musl-linked TinyCC output size overflow".to_string()))?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), TCC_MUSL_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn validate_expected_outputs(outputs: &[TccMuslOutputReport]) -> Result<(), TccMuslError> {
    let mut mismatches = Vec::new();
    for expected in TCC_MUSL_EXPECTED_OUTPUTS {
        let output = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            TccMuslError::Materialization(format!("musl-linked TinyCC report lacks {}", expected.artifact_id))
        })?;
        if output.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, output.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(TccMuslError::Materialization(format!(
            "musl-linked TinyCC output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(outputs.len(), TCC_MUSL_EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn configured_source_digest_blake3() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-tcc-musl-configured-source-v1\0");
    hasher.update(crate::stagex_tinycc27::TINYCC27_PATCHED_SOURCE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(TCC_MUSL_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(TCC_MUSL_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0");
    hasher.update(MUSL_LOGICAL_PREFIX.as_bytes());
    hasher.update(b"\0BOOTSTRAP=1\0HAVE_BITFIELD=1\0HAVE_FLOAT=1\0HAVE_LONG_LONG=1\0HAVE_SETJMP=1\0");
    hasher.finalize().to_hex().to_string()
}

fn validate_bound_recipe() -> Result<(), TccMuslError> {
    let observed = blake3::hash(TCC_MUSL_RECIPE).to_hex().to_string();
    if observed != TCC_MUSL_RECIPE_BLAKE3 {
        return Err(TccMuslError::Materialization(format!(
            "musl-linked TinyCC recipe mismatch: expected {TCC_MUSL_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert!(!TCC_MUSL_RECIPE.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), TccMuslError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TCC_MUSL_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(TccMuslError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), TccMuslError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TCC_MUSL_FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(TccMuslError::Materialization(format!("{label} is empty")));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, TccMuslError> {
    crate::stagex_mes_lib::utf8_absolute(path, label).map(str::to_string).map_err(TccMuslError::Runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETAINED_TINYCC27_SOURCE_ENV: &str = "MANTLE_STAGE_X_TINYCC27_SOURCE_ROOT";
    const RETAINED_PREP_ROOT_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_PREP_ROOT";
    const RETAINED_MUSL_ROOT_ENV: &str = "MANTLE_STAGE_X_MUSL_ROOT";
    const RETAINED_SCRATCH_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_SCRATCH";

    #[test]
    fn configured_source_digest_is_stable() {
        let digest = configured_source_digest_blake3();
        assert_eq!(digest.len(), blake3::OUT_LEN * 2);
        assert_eq!(digest, TCC_MUSL_CONFIGURED_SOURCE_BLAKE3);
    }

    #[test]
    fn substituted_recipe_is_rejected() {
        let observed = blake3::hash(b"substituted").to_hex().to_string();
        assert_ne!(observed, TCC_MUSL_RECIPE_BLAKE3);
        assert_eq!(TCC_MUSL_RECIPE_BLAKE3.len(), blake3::OUT_LEN * 2);
    }

    #[test]
    #[ignore = "requires retained TinyCC source, musl-prep, and musl"]
    fn derives_retained_tcc_musl_inventory() {
        let source = PathBuf::from(std::env::var(RETAINED_TINYCC27_SOURCE_ENV).unwrap());
        let prep = PathBuf::from(std::env::var(RETAINED_PREP_ROOT_ENV).unwrap());
        let musl = PathBuf::from(std::env::var(RETAINED_MUSL_ROOT_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SCRATCH_ENV).unwrap());
        let report = derive_tcc_musl_inventory(TccMuslInventoryRequest {
            tinycc27_source_root: &source,
            tcc_musl_prep_root: &prep,
            musl_root: &musl,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, TCC_MUSL_BUILD_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
