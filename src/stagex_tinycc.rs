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
const TINYCC_RECORD_NAME: &str = "tcc-0.9.26-src";
const TINYCC_SOURCE_ARTIFACT_ID: &str = "tinycc-0.9.26-source";
const TINYCC_SOURCE_CONTENT_BLAKE3: &str = "db8c11387fd0db5783b4d2349b578ba1edfc739bf13d0865ab8b6b5f83a0711b";
const TINYCC_SOURCE_OUTPUT_NAME: &str = "tinycc-0.9.26";
const TINYCC_MES_RECORD_NAME: &str = "mes-0.27.1-src";
const TINYCC_MES_SOURCE_ARTIFACT_ID: &str = "tinycc-mes-0.27.1-source";
const TINYCC_MES_SOURCE_CONTENT_BLAKE3: &str = "438536d1095c3cd1f19b2645572dbe4ec265353513076db6ba5c395e23e6841a";
const TINYCC_MES_SOURCE_OUTPUT_NAME: &str = "mes-0.27.1";
const TINYCC_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-tinycc-source-materialization-v1";
const TINYCC_SOURCE_RECORD_COUNT: usize = 2;
const TINYCC_SOURCE_NON_CLAIM: &str =
    "TinyCC source materialization proves authenticated offline archive identity and fixed-output parity only";
const TCC_MES_REPORT_FORMAT: &str = "mantle-stagex-tcc-mes-inventory-v1";
const TCC_MES_NON_CLAIM: &str = "this inventory binds the Mes-linked compiler, TinyCC runtime refresh, boot0, and final TinyCC compiler; it does not prove normalized provider admission or later compiler stages";
pub(crate) const TCC_MES_BLAKE3: &str = "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3";
pub(crate) const TCC_BOOT0_BLAKE3: &str = "8b3f20ef69f2356e64d513b82d26afb5c67bce9e51c045bbfbbf14784d13c66f";
pub(crate) const TINYCC_FINAL_BLAKE3: &str = "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3";
const TCC_MES_ARENA_BYTES: &str = "30000000";
const TCC_MES_STACK_BYTES: &str = "15000000";
const TCC_MES_BASE_ADDRESS: &str = "0x08048000";
const TCC_MES_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TCC_MES_PATCH_MATCH_MAX: usize = 32;
const TCC_MES_LOGICAL_PREFIX: &str = "/stagex/tinycc-0.9.26";
const TCC_MES_INITIAL_OUTPUT_COUNT: usize = 2;
const TCC_MES_OUTPUT_COUNT: usize = 9;
const TCC_MES_COMPILE_COMMAND_COUNT: u32 = 2;
const TCC_RUNTIME_REFRESH_COMMAND_COUNT: u32 = 7;
const TCC_BOOT0_COMMAND_COUNT: u32 = 2;
const TCC_FINAL_COMMAND_COUNT: u32 = 3;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TinyccExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

const TINYCC_EXPECTED_OUTPUT_COUNT: usize = 7;
pub(crate) const TINYCC_EXPECTED_OUTPUTS: [TinyccExpectedOutput; TINYCC_EXPECTED_OUTPUT_COUNT] = [
    TinyccExpectedOutput {
        artifact_id: "tcc-mes",
        digest_blake3: TCC_MES_BLAKE3,
    },
    TinyccExpectedOutput {
        artifact_id: "tcc-boot0",
        digest_blake3: TCC_BOOT0_BLAKE3,
    },
    TinyccExpectedOutput {
        artifact_id: "tinycc-0.9.26",
        digest_blake3: TINYCC_FINAL_BLAKE3,
    },
    TinyccExpectedOutput {
        artifact_id: "tinycc-crt1",
        digest_blake3: "d3fec15bd12fd72fdbdf4248d6df6b21a44cfb6a3e2c9d787a0d081ff7143df1",
    },
    TinyccExpectedOutput {
        artifact_id: "tinycc-libc",
        digest_blake3: "cbc10117c34515b9b3ad14f3e4dc73a37708c6206f36cd51dc6f1e8a98093f8c",
    },
    TinyccExpectedOutput {
        artifact_id: "tinycc-libtcc1",
        digest_blake3: "0e8b75458ad70ab03142b27a3014af68f57c03004f4d22a65702e60d531140e9",
    },
    TinyccExpectedOutput {
        artifact_id: "tinycc-libgetopt",
        digest_blake3: "1fb5e11a952e173ec451601f2f11c09b2703d48e20b6533d327dc83b6b520449",
    },
];

const ABORT_OBJECT: &[u8] = b"\n<\n:abort\n55\n4889e5\n4881ec A8200000\n48c7c7 06000000\n57\ne8 %raise\n4883c4 08\n48c7c7 00000000\n4829f8\n0f9cc0\n480fb6c0\n4885c0\n0f84 %_abort_1_break\n48c7c0 00000000\n488945 F8\n48c7c0 02000000\n488b7d F8\n8807\n4885c0\ne9 %_abort_1_break\n:_abort_1_break\n4889ec\n5d\nc3\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TinyccSourceMaterializationReport {
    pub format: &'static str,
    pub source_bundle_manifest_blake3: String,
    pub artifact_id: &'static str,
    pub record_name: &'static str,
    pub record_identity: String,
    pub record_content_blake3: String,
    pub output_path: PathBuf,
    pub mes_record_identity: String,
    pub mes_record_content_blake3: String,
    pub mes_output_path: PathBuf,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct TccMesInventoryRequest<'a> {
    pub tinycc_source_root: &'a Path,
    pub mes_source_root: &'a Path,
    pub mes_runtime_root: &'a Path,
    pub mes_m2_path: &'a Path,
    pub stage0_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMesOutput {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TccMesInventoryReport {
    pub format: &'static str,
    pub source_patch_digest_blake3: String,
    pub unified_libc_source_count: u32,
    pub compile_command_count: u32,
    pub runtime_refresh_command_count: u32,
    pub boot0_command_count: u32,
    pub final_command_count: u32,
    pub outputs: Vec<TccMesOutput>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum TinyccError {
    Bundle(crate::RunError),
    InvalidAuthority(String),
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for TinyccError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bundle(error) => write!(formatter, "reading TinyCC source bundle: {error}"),
            Self::InvalidAuthority(message) => write!(formatter, "invalid TinyCC source authority: {message}"),
            Self::Materialization(message) => write!(formatter, "materializing TinyCC source: {message}"),
            Self::Runtime(error) => write!(formatter, "building Mes-linked TinyCC: {error}"),
        }
    }
}

impl std::error::Error for TinyccError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for TinyccError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::RunError> for TinyccError {
    fn from(error: crate::RunError) -> Self {
        Self::Bundle(error)
    }
}

pub(crate) fn tinycc_source_artifact_digests() -> [(&'static str, &'static str); TINYCC_SOURCE_RECORD_COUNT] {
    [
        (TINYCC_SOURCE_ARTIFACT_ID, TINYCC_SOURCE_CONTENT_BLAKE3),
        (TINYCC_MES_SOURCE_ARTIFACT_ID, TINYCC_MES_SOURCE_CONTENT_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_tinycc_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    source_root: &Path,
) -> Result<TinyccSourceMaterializationReport, TinyccError> {
    let manifest = read_source_bundle(bundle_path)?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(TinyccError::InvalidAuthority(format!(
            "source bundle BLAKE3 mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = select_tinycc_source_record(&manifest.records)?;
    let mes_record =
        select_fixed_source_record(&manifest.records, TINYCC_MES_RECORD_NAME, TINYCC_MES_SOURCE_CONTENT_BLAKE3)?;
    fs::create_dir(source_root).map_err(|error| {
        TinyccError::Materialization(format!(
            "creating create-new TinyCC source root {}: {error}",
            source_root.display()
        ))
    })?;
    let output_path = source_root.join(TINYCC_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path).map_err(|error| {
        TinyccError::Materialization(format!(
            "materializing {} at {}: {error}",
            TINYCC_RECORD_NAME,
            output_path.display()
        ))
    })?;
    let mes_output_path = source_root.join(TINYCC_MES_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(mes_record, &mes_output_path).map_err(|error| {
        TinyccError::Materialization(format!(
            "materializing {} at {}: {error}",
            TINYCC_MES_RECORD_NAME,
            mes_output_path.display()
        ))
    })?;
    assert!(output_path.is_dir());
    assert!(mes_output_path.is_dir());
    Ok(TinyccSourceMaterializationReport {
        format: TINYCC_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: TINYCC_SOURCE_ARTIFACT_ID,
        record_name: TINYCC_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        mes_record_identity: mes_record.identity.clone(),
        mes_record_content_blake3: mes_record.content_blake3.clone(),
        mes_output_path,
        non_claim: TINYCC_SOURCE_NON_CLAIM,
    })
}

pub(crate) fn derive_tcc_mes_inventory(
    request: TccMesInventoryRequest<'_>,
) -> Result<TccMesInventoryReport, TinyccError> {
    validate_tcc_mes_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| TinyccError::Materialization(format!("creating TCC Mes scratch: {error}")))?;
    let tcc_root = request.scratch_dir.join("tinycc-0.9.26");
    let mes_root = request.scratch_dir.join("mes-0.27.1");
    crate::stagex_mes_lib::copy_tree_bounded(request.tinycc_source_root, &tcc_root)?;
    crate::stagex_mes_lib::copy_tree_bounded(request.mes_source_root, &mes_root)?;
    make_tree_owner_writable(&tcc_root)?;
    make_tree_owner_writable(&mes_root)?;
    let prefix = request.scratch_dir.join("prefix");
    let mes_link = request.scratch_dir.join("mes-link");
    prepare_tcc_mes_runtime(&request, &mes_root, &prefix, &mes_link)?;
    patch_tinycc_sources(&tcc_root)?;
    let source_patch_digest_blake3 = patched_source_digest(&tcc_root)?;
    let unified_libc_source_count = create_unified_libc(&mes_root)?;
    let mut outputs = build_tcc_mes(&request, &tcc_root, &prefix, &mes_link)?;
    outputs.extend(finish_tinycc_runtime(&request, &tcc_root, &mes_root, &prefix, &mes_link)?);
    validate_expected_tinycc_outputs(&outputs)?;
    let report = TccMesInventoryReport {
        format: TCC_MES_REPORT_FORMAT,
        source_patch_digest_blake3,
        unified_libc_source_count,
        compile_command_count: TCC_MES_COMPILE_COMMAND_COUNT,
        runtime_refresh_command_count: TCC_RUNTIME_REFRESH_COMMAND_COUNT,
        boot0_command_count: TCC_BOOT0_COMMAND_COUNT,
        final_command_count: TCC_FINAL_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: TCC_MES_NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("tcc-mes-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| TinyccError::Materialization(format!("serializing tcc-mes report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), TCC_MES_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_expected_tinycc_outputs(outputs: &[TccMesOutput]) -> Result<(), TinyccError> {
    for expected in TINYCC_EXPECTED_OUTPUTS {
        let output = outputs
            .iter()
            .find(|output| output.artifact_id == expected.artifact_id)
            .ok_or_else(|| TinyccError::Materialization(format!("TinyCC report lacks {}", expected.artifact_id)))?;
        if output.digest_blake3 != expected.digest_blake3 {
            return Err(TinyccError::Materialization(format!(
                "TinyCC output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, output.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), TCC_MES_OUTPUT_COUNT);
    assert_eq!(TINYCC_EXPECTED_OUTPUTS.len(), TINYCC_EXPECTED_OUTPUT_COUNT);
    Ok(())
}

fn validate_tcc_mes_inputs(request: &TccMesInventoryRequest<'_>) -> Result<(), TinyccError> {
    if request.scratch_dir.exists() {
        return Err(TinyccError::Materialization(format!(
            "create-new tcc-mes scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    for (label, root) in [
        ("TinyCC source", request.tinycc_source_root),
        ("Mes source", request.mes_source_root),
        ("Mes runtime", request.mes_runtime_root),
        ("full Stage0", request.stage0_root),
    ] {
        if !root.is_absolute() || !root.is_dir() {
            return Err(TinyccError::Materialization(format!(
                "{label} root is not an absolute directory: {}",
                root.display()
            )));
        }
    }
    validate_file_digest(request.mes_m2_path, crate::stagex_mes::MES_M2_BLAKE3, "Mes runtime mes-m2")?;
    assert!(request.tinycc_source_root.join("tcc.c").is_file());
    assert!(request.mes_source_root.join("build-aux/configure-lib.sh").is_file());
    Ok(())
}

fn prepare_tcc_mes_runtime(
    request: &TccMesInventoryRequest<'_>,
    mes_root: &Path,
    prefix: &Path,
    mes_link: &Path,
) -> Result<(), TinyccError> {
    let libdir = prefix.join("lib/mes");
    let include = prefix.join("include/mes");
    for directory in [
        prefix.join("bin"),
        libdir.join("tcc"),
        prefix.join("include"),
        mes_link.join("linux"),
    ] {
        fs::create_dir_all(&directory)
            .map_err(|error| TinyccError::Materialization(format!("creating {}: {error}", directory.display())))?;
    }
    crate::stagex_mes_lib::copy_tree_bounded(&request.mes_runtime_root.join("include/mes-include"), &include)?;
    crate::stagex_mes_lib::copy_tree_bounded(&request.mes_runtime_root.join("lib/x86_64-mes"), &libdir)?;
    crate::stagex_mes_lib::copy_tree_bounded(
        &request.mes_runtime_root.join("lib/x86_64-mes"),
        &mes_link.join("x86_64-mes"),
    )?;
    crate::stagex_mes_lib::copy_tree_bounded(
        &request.mes_runtime_root.join("lib/linux/x86_64-mes"),
        &mes_link.join("linux/x86_64-mes"),
    )?;
    append_abort_object(&mes_link.join("x86_64-mes/libc+tcc.a"))?;
    fs::write(mes_root.join("include/mes/config.h"), b"#undef SYSTEM_LIBC\n#define MES_VERSION \"0.27.1\"\n")
        .map_err(|error| TinyccError::Materialization(format!("writing Mes config.h: {error}")))?;
    replace_required_text(&mes_root.join("include/mes/lib.h"), "int oputs (char const *s);\n", "")?;
    fs::create_dir_all(mes_root.join("include/arch"))
        .map_err(|error| TinyccError::Materialization(format!("creating Mes arch includes: {error}")))?;
    for file in ["kernel-stat.h", "signal.h", "syscall.h"] {
        crate::stagex_mes_lib::copy_file_replace(
            &mes_root.join("include/linux/x86_64").join(file),
            &mes_root.join("include/arch").join(file),
        )?;
    }
    assert!(libdir.join("libc+tcc.a").is_file());
    assert!(mes_link.join("x86_64-mes/libc+tcc.a").is_file());
    Ok(())
}

pub(crate) fn create_unified_libc(mes_root: &Path) -> Result<u32, TinyccError> {
    let configure = fs::read_to_string(mes_root.join("build-aux/configure-lib.sh"))
        .map_err(|error| TinyccError::Materialization(format!("reading Mes configure-lib.sh: {error}")))?;
    let sources = crate::stagex_mes_lib::derive_mes_unified_libc_sources(&configure)?;
    let mut bytes = Vec::new();
    for source in &sources {
        let source_bytes = crate::stagex_mes_lib::read_bounded_file(
            &mes_root.join(source),
            TCC_MES_FILE_BYTES_MAX,
            "unified Mes libc source",
        )?;
        let next_len = bytes
            .len()
            .checked_add(source_bytes.len())
            .ok_or_else(|| TinyccError::Materialization("unified Mes libc size overflow".to_string()))?;
        if u64::try_from(next_len).unwrap_or(u64::MAX) > TCC_MES_FILE_BYTES_MAX {
            return Err(TinyccError::Materialization(format!(
                "unified Mes libc exceeds {TCC_MES_FILE_BYTES_MAX} bytes"
            )));
        }
        bytes.extend_from_slice(&source_bytes);
    }
    crate::stagex_mes_lib::write_create_new(&mes_root.join("unified-libc.c"), &bytes)?;
    let count = u32::try_from(sources.len())
        .map_err(|_| TinyccError::Materialization("unified libc source count does not fit u32".to_string()))?;
    assert!(count > 0);
    assert!(!bytes.is_empty());
    Ok(count)
}

fn append_abort_object(archive: &Path) -> Result<(), TinyccError> {
    let mut bytes = crate::stagex_mes_lib::read_bounded_file(archive, TCC_MES_FILE_BYTES_MAX, "Mes libc+tcc archive")?;
    if bytes.windows(b":abort\n".len()).any(|window| window == b":abort\n") {
        return Ok(());
    }
    bytes.extend_from_slice(ABORT_OBJECT);
    fs::write(archive, &bytes)
        .map_err(|error| TinyccError::Materialization(format!("appending Mes abort object: {error}")))?;
    assert!(bytes.windows(b":abort\n".len()).any(|window| window == b":abort\n"));
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= TCC_MES_FILE_BYTES_MAX);
    Ok(())
}

fn build_tcc_mes(
    request: &TccMesInventoryRequest<'_>,
    tcc_root: &Path,
    prefix: &Path,
    mes_link: &Path,
) -> Result<Vec<TccMesOutput>, TinyccError> {
    fs::write(tcc_root.join("config.h"), [])
        .map_err(|error| TinyccError::Materialization(format!("writing TinyCC config.h: {error}")))?;
    let mes = request.mes_m2_path.to_path_buf();
    let mescc = request.mes_runtime_root.join("bin/mescc.scm");
    let include = prefix.join("include/mes");
    let tcc_s = tcc_root.join("tcc.s");
    let tcc_mes = tcc_root.join("tcc-mes");
    let compile_args = tcc_mes_compile_args(&mescc, &tcc_s, &include, tcc_root)?;
    let compile_env = tcc_mes_environment(request, None)?;
    crate::stagex_mes_lib::run_bounded_process(
        &mes,
        &compile_args,
        tcc_root,
        &compile_env,
        &request.scratch_dir.join("tcc-mes-compile.stderr.txt"),
    )?;
    let link_args = vec![
        "--no-auto-compile".to_string(),
        "-e".to_string(),
        "main".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&mescc, "mescc entrypoint")?.to_string(),
        "--".to_string(),
        "--base-address".to_string(),
        TCC_MES_BASE_ADDRESS.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tcc_mes, "tcc-mes output")?.to_string(),
        "-L".to_string(),
        crate::stagex_mes_lib::utf8_absolute(mes_link, "Mes link root")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tcc_s, "tcc-mes assembly")?.to_string(),
        "-l".to_string(),
        "c+tcc".to_string(),
    ];
    let link_env = tcc_mes_environment(
        request,
        Some(("libdir", crate::stagex_mes_lib::utf8_absolute(mes_link, "Mes link root")?)),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        &mes,
        &link_args,
        tcc_root,
        &link_env,
        &request.scratch_dir.join("tcc-mes-link.stderr.txt"),
    )?;
    set_owner_executable(&tcc_mes)?;
    crate::stagex_mes_lib::run_bounded_process(
        &tcc_mes,
        &["-version"],
        tcc_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("tcc-mes-version.stderr.txt"),
    )?;
    let mut outputs = Vec::with_capacity(TCC_MES_INITIAL_OUTPUT_COUNT);
    for (artifact_id, path) in [("tcc-mes-assembly", tcc_s), ("tcc-mes", tcc_mes)] {
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, TCC_MES_FILE_BYTES_MAX, artifact_id)?;
        outputs.push(TccMesOutput {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| TinyccError::Materialization("tcc-mes output size does not fit u64".to_string()))?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), TCC_MES_INITIAL_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn finish_tinycc_runtime(
    request: &TccMesInventoryRequest<'_>,
    tcc_root: &Path,
    mes_root: &Path,
    prefix: &Path,
    mes_link: &Path,
) -> Result<Vec<TccMesOutput>, TinyccError> {
    let tcc_mes = tcc_root.join("tcc-mes");
    let libdir = prefix.join("lib/mes");
    let tcc_runtime = libdir.join("tcc");
    let include = prefix.join("include/mes");
    let empty_env = BTreeMap::<String, String>::new();
    let runtime_build = request.scratch_dir.join("tinycc-runtime-build");
    fs::create_dir(&runtime_build)
        .map_err(|error| TinyccError::Materialization(format!("creating TinyCC runtime build root: {error}")))?;
    fs::create_dir(runtime_build.join("src"))
        .map_err(|error| TinyccError::Materialization(format!("creating TinyCC runtime source root: {error}")))?;
    fs::create_dir(runtime_build.join("obj"))
        .map_err(|error| TinyccError::Materialization(format!("creating TinyCC runtime object root: {error}")))?;
    let runtime_specs = [
        ("runtime-crt1", mes_root.join("lib/linux/x86_64-mes-gcc/crt1.c"), "crt1.c", "crt1.o", vec![
            "-D",
            "HAVE_CONFIG_H=1",
        ]),
        ("runtime-libtcc1", mes_root.join("lib/libtcc1.c"), "libtcc1.c", "libtcc1.o", vec![
            "-D",
            "HAVE_CONFIG_H=1",
            "-D",
            "HAVE_LONG_LONG=1",
            "-D",
            "HAVE_FLOAT=1",
        ]),
        ("runtime-libc", mes_root.join("unified-libc.c"), "unified-libc.c", "unified-libc.o", vec![
            "-D",
            "HAVE_CONFIG_H=1",
        ]),
        ("runtime-getopt", mes_root.join("lib/posix/getopt.c"), "getopt.c", "getopt.o", vec![
            "-D",
            "HAVE_CONFIG_H=1",
        ]),
    ];
    for (label, source, source_name, object_name, defines) in runtime_specs {
        crate::stagex_mes_lib::copy_file_exact(&source, &runtime_build.join("src").join(source_name))?;
        let mut args = vec!["-c".to_string()];
        args.extend(defines.into_iter().map(str::to_string));
        args.extend([
            "-I".to_string(),
            crate::stagex_mes_lib::utf8_absolute(&mes_root.join("include"), "Mes source include")?.to_string(),
            "-I".to_string(),
            crate::stagex_mes_lib::utf8_absolute(&mes_root.join("include/linux/x86_64"), "Mes Linux include")?
                .to_string(),
            "-o".to_string(),
            format!("obj/{object_name}"),
            format!("src/{source_name}"),
        ]);
        crate::stagex_mes_lib::run_bounded_process(
            &tcc_mes,
            &args,
            &runtime_build,
            &empty_env,
            &request.scratch_dir.join(format!("{label}.stderr.txt")),
        )?;
    }
    crate::stagex_mes_lib::copy_file_replace(&runtime_build.join("obj/crt1.o"), &libdir.join("crt1.o"))?;
    for empty in [libdir.join("crti.o"), libdir.join("crtn.o")] {
        crate::stagex_mes_lib::remove_path_if_present(&empty)?;
        crate::stagex_mes_lib::write_create_new(&empty, &[])?;
    }
    let archives = [
        ("runtime-libtcc1-ar", tcc_runtime.join("libtcc1.a"), "obj/libtcc1.o"),
        ("runtime-libc-ar", libdir.join("libc.a"), "obj/unified-libc.o"),
        ("runtime-getopt-ar", libdir.join("libgetopt.a"), "obj/getopt.o"),
    ];
    for (label, archive, object) in archives {
        crate::stagex_mes_lib::remove_path_if_present(&archive)?;
        let args = vec![
            "-ar".to_string(),
            "cr".to_string(),
            crate::stagex_mes_lib::utf8_absolute(&archive, "TinyCC runtime archive")?.to_string(),
            object.to_string(),
        ];
        crate::stagex_mes_lib::run_bounded_process(
            &tcc_mes,
            &args,
            &runtime_build,
            &empty_env,
            &request.scratch_dir.join(format!("{label}.stderr.txt")),
        )?;
    }
    for file in ["crt1.o", "crti.o", "crtn.o", "libc.a"] {
        crate::stagex_mes_lib::copy_file_replace(&libdir.join(file), &tcc_runtime.join(file))?;
    }
    let boot0 = tcc_root.join("tcc-boot0");
    let boot_args = tcc_boot_compile_args(&boot0, tcc_root, &include, &libdir, &tcc_runtime)?;
    crate::stagex_mes_lib::run_bounded_process(
        &tcc_mes,
        &boot_args,
        tcc_root,
        &empty_env,
        &request.scratch_dir.join("tcc-boot0.stderr.txt"),
    )?;
    set_owner_executable(&boot0)?;
    crate::stagex_mes_lib::run_bounded_process(
        &boot0,
        &["-version"],
        tcc_root,
        &empty_env,
        &request.scratch_dir.join("tcc-boot0-version.stderr.txt"),
    )?;
    let output_root = request.scratch_dir.join("output");
    for directory in [
        output_root.join("bin"),
        output_root.join("lib/mes/tcc"),
        output_root.join("include"),
    ] {
        fs::create_dir_all(&directory)
            .map_err(|error| TinyccError::Materialization(format!("creating final TinyCC directory: {error}")))?;
    }
    crate::stagex_mes_lib::copy_tree_bounded(&include, &output_root.join("include/mes"))?;
    crate::stagex_mes_lib::copy_tree_bounded(&include, &output_root.join("include/mes-include"))?;
    for file in ["crt1.o", "crti.o", "crtn.o", "libc.a", "libgetopt.a"] {
        crate::stagex_mes_lib::copy_file_exact(&libdir.join(file), &output_root.join("lib/mes").join(file))?;
    }
    crate::stagex_mes_lib::copy_file_exact(&tcc_runtime.join("libtcc1.a"), &output_root.join("lib/mes/tcc/libtcc1.a"))?;
    let final_s = tcc_root.join("tcc-final.s");
    let mes = request.mes_m2_path.to_path_buf();
    let mescc = request.mes_runtime_root.join("bin/mescc.scm");
    let final_args = tcc_mes_compile_args(&mescc, &final_s, &include, tcc_root)?;
    let compile_env = tcc_mes_environment(request, None)?;
    crate::stagex_mes_lib::run_bounded_process(
        &mes,
        &final_args,
        tcc_root,
        &compile_env,
        &request.scratch_dir.join("tcc-final-compile.stderr.txt"),
    )?;
    let final_tcc = output_root.join("bin/tcc-0.9.26");
    let link_args = vec![
        "--no-auto-compile".to_string(),
        "-e".to_string(),
        "main".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&mescc, "mescc entrypoint")?.to_string(),
        "--".to_string(),
        "--base-address".to_string(),
        TCC_MES_BASE_ADDRESS.to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&final_tcc, "final TinyCC")?.to_string(),
        "-L".to_string(),
        crate::stagex_mes_lib::utf8_absolute(mes_link, "Mes link root")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&final_s, "final TinyCC assembly")?.to_string(),
        "-l".to_string(),
        "c+tcc".to_string(),
    ];
    let link_env = tcc_mes_environment(
        request,
        Some(("libdir", crate::stagex_mes_lib::utf8_absolute(mes_link, "Mes link root")?)),
    )?;
    crate::stagex_mes_lib::run_bounded_process(
        &mes,
        &link_args,
        tcc_root,
        &link_env,
        &request.scratch_dir.join("tcc-final-link.stderr.txt"),
    )?;
    set_owner_executable(&final_tcc)?;
    crate::stagex_mes_lib::copy_file_exact(&final_tcc, &output_root.join("bin/tcc"))?;
    set_owner_executable(&output_root.join("bin/tcc"))?;
    crate::stagex_mes_lib::run_bounded_process(
        &output_root.join("bin/tcc"),
        &["-version"],
        tcc_root,
        &empty_env,
        &request.scratch_dir.join("tcc-final-version.stderr.txt"),
    )?;
    collect_final_tinycc_outputs(&boot0, &final_s, &final_tcc, &output_root)
}

fn tcc_boot_compile_args(
    output: &Path,
    tcc_root: &Path,
    include: &Path,
    libdir: &Path,
    tcc_runtime: &Path,
) -> Result<Vec<String>, TinyccError> {
    let args = vec![
        "-B".to_string(),
        crate::stagex_mes_lib::utf8_absolute(tcc_runtime, "TinyCC runtime")?.to_string(),
        "-static".to_string(),
        "-nostdlib".to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "TinyCC boot0")?.to_string(),
        "-D".to_string(),
        "BOOTSTRAP=1".to_string(),
        "-D".to_string(),
        "HAVE_FLOAT=1".to_string(),
        "-D".to_string(),
        "HAVE_BITFIELD=1".to_string(),
        "-D".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "-D".to_string(),
        "HAVE_SETJMP=1".to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(tcc_root, "TinyCC source")?.to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(include, "TinyCC include")?.to_string(),
        "-D".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCCDIR=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        format!("CONFIG_TCC_CRTPREFIX=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes\""),
        "-D".to_string(),
        "CONFIG_TCC_ELFINTERP=\"/mes/loader\"".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCC_LIBPATHS=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes:{TCC_MES_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{TCC_MES_LOGICAL_PREFIX}/include/mes\""),
        "-D".to_string(),
        format!("TCC_LIBGCC=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes/libc.a\""),
        "-D".to_string(),
        "TCC_LIBTCC1=\"libtcc1.a\"".to_string(),
        "-D".to_string(),
        "CONFIG_TCCBOOT=1".to_string(),
        "-D".to_string(),
        "CONFIG_TCC_STATIC=1".to_string(),
        "-D".to_string(),
        "CONFIG_USE_LIBGCC=1".to_string(),
        "-D".to_string(),
        "TCC_VERSION=\"0.9.26\"".to_string(),
        "-D".to_string(),
        "ONE_SOURCE=1".to_string(),
        "-L".to_string(),
        crate::stagex_mes_lib::utf8_absolute(tcc_root, "TinyCC source")?.to_string(),
        "-L".to_string(),
        crate::stagex_mes_lib::utf8_absolute(libdir, "TinyCC libdir")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tcc_root.join("tcc.c"), "TinyCC tcc.c")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("crt1.o"), "TinyCC crt1")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&libdir.join("libc.a"), "TinyCC libc")?.to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tcc_runtime.join("libtcc1.a"), "TinyCC libtcc1")?.to_string(),
    ];
    assert!(args.len() > 1);
    assert!(args.iter().all(|arg| !arg.is_empty()));
    Ok(args)
}

fn collect_final_tinycc_outputs(
    boot0: &Path,
    final_s: &Path,
    final_tcc: &Path,
    output_root: &Path,
) -> Result<Vec<TccMesOutput>, TinyccError> {
    let specs = [
        ("tcc-boot0", boot0.to_path_buf()),
        ("tcc-final-assembly", final_s.to_path_buf()),
        ("tinycc-0.9.26", final_tcc.to_path_buf()),
        ("tinycc-crt1", output_root.join("lib/mes/crt1.o")),
        ("tinycc-libc", output_root.join("lib/mes/libc.a")),
        ("tinycc-libtcc1", output_root.join("lib/mes/tcc/libtcc1.a")),
        ("tinycc-libgetopt", output_root.join("lib/mes/libgetopt.a")),
    ];
    let mut outputs = Vec::with_capacity(specs.len());
    for (artifact_id, path) in specs {
        let bytes = crate::stagex_mes_lib::read_bounded_file(&path, TCC_MES_FILE_BYTES_MAX, artifact_id)?;
        outputs.push(TccMesOutput {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| TinyccError::Materialization("final TinyCC output size does not fit u64".to_string()))?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), TCC_MES_OUTPUT_COUNT - TCC_MES_INITIAL_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn tcc_mes_compile_args(
    mescc: &Path,
    output: &Path,
    include: &Path,
    tcc_root: &Path,
) -> Result<Vec<String>, TinyccError> {
    let arguments = vec![
        "--no-auto-compile".to_string(),
        "-e".to_string(),
        "main".to_string(),
        crate::stagex_mes_lib::utf8_absolute(mescc, "mescc entrypoint")?.to_string(),
        "--".to_string(),
        "-S".to_string(),
        "-o".to_string(),
        crate::stagex_mes_lib::utf8_absolute(output, "tcc assembly")?.to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(include, "TinyCC Mes include")?.to_string(),
        "-D".to_string(),
        "BOOTSTRAP=1".to_string(),
        "-D".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "-I".to_string(),
        crate::stagex_mes_lib::utf8_absolute(tcc_root, "TinyCC source")?.to_string(),
        "-D".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        "-D".to_string(),
        "inline=".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCCDIR=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes/tcc\""),
        "-D".to_string(),
        "CONFIG_SYSROOT=\"/\"".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCC_CRTPREFIX=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes\""),
        "-D".to_string(),
        "CONFIG_TCC_ELFINTERP=\"/mes/loader\"".to_string(),
        "-D".to_string(),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{TCC_MES_LOGICAL_PREFIX}/include/mes\""),
        "-D".to_string(),
        format!("TCC_LIBGCC=\"{TCC_MES_LOGICAL_PREFIX}/lib/mes/libc.a\""),
        "-D".to_string(),
        "CONFIG_TCC_LIBTCC1_MES=0".to_string(),
        "-D".to_string(),
        "CONFIG_TCCBOOT=1".to_string(),
        "-D".to_string(),
        "CONFIG_TCC_STATIC=1".to_string(),
        "-D".to_string(),
        "CONFIG_USE_LIBGCC=1".to_string(),
        "-D".to_string(),
        "TCC_VERSION=\"0.9.26\"".to_string(),
        "-D".to_string(),
        "ONE_SOURCE=1".to_string(),
        crate::stagex_mes_lib::utf8_absolute(&tcc_root.join("tcc.c"), "TinyCC tcc.c")?.to_string(),
    ];
    assert!(arguments.len() > 1);
    assert!(arguments.iter().all(|argument| !argument.is_empty()));
    Ok(arguments)
}

fn tcc_mes_environment(
    request: &TccMesInventoryRequest<'_>,
    extra: Option<(&str, &str)>,
) -> Result<BTreeMap<String, String>, TinyccError> {
    let mut environment = BTreeMap::from([
        (
            "MES_PREFIX".to_string(),
            crate::stagex_mes_lib::utf8_absolute(request.mes_runtime_root, "Mes runtime")?.to_string(),
        ),
        (
            "GUILE_LOAD_PATH".to_string(),
            format!("{}/mes/module", crate::stagex_mes_lib::utf8_absolute(request.mes_runtime_root, "Mes runtime")?),
        ),
        ("MES_ARENA".to_string(), TCC_MES_ARENA_BYTES.to_string()),
        ("MES_MAX_ARENA".to_string(), TCC_MES_ARENA_BYTES.to_string()),
        ("MES_STACK".to_string(), TCC_MES_STACK_BYTES.to_string()),
        ("M1".to_string(), stage0_tool_path(request.stage0_root, "stage0-m1")?),
        ("HEX2".to_string(), stage0_tool_path(request.stage0_root, "stage0-hex2")?),
        ("BLOOD_ELF".to_string(), stage0_tool_path(request.stage0_root, "stage0-full-blood-elf")?),
    ]);
    if let Some((name, value)) = extra {
        environment.insert(name.to_string(), value.to_string());
    }
    assert!(environment.contains_key("M1"));
    assert!(environment.contains_key("HEX2"));
    Ok(environment)
}

fn stage0_tool_path(root: &Path, artifact_id: &str) -> Result<String, TinyccError> {
    if let Some(expected) = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok(crate::stagex_mes_lib::utf8_absolute(&root.join(expected.relative_path), artifact_id)?.to_string());
    }
    if let Some(expected) = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok(crate::stagex_mes_lib::utf8_absolute(&root.join(expected.relative_path), artifact_id)?.to_string());
    }
    Err(TinyccError::Materialization(format!("unknown Stage0 tool {artifact_id}")))
}

fn patch_tinycc_sources(root: &Path) -> Result<(), TinyccError> {
    patch_tcctools_archive(root)?;
    replace_required_text(&root.join("tccelf.c"), "const char filename[]", "const char *filename")?;
    replace_required_text(
        &root.join("x86_64-gen.c"),
        "SValue tmp = vtop[0];",
        "SValue tmp;\n            memcpy(&tmp, &vtop[0], sizeof(SValue));",
    )?;
    replace_required_text(&root.join("x86_64-gen.c"), "tmp = vtop[0];", "memcpy(&tmp, &vtop[0], sizeof(SValue));")?;
    replace_required_text(
        &root.join("x86_64-gen.c"),
        "vtop[0] = vtop[-i];",
        "memcpy(&vtop[0], &vtop[-i], sizeof(SValue));",
    )?;
    replace_required_text(&root.join("x86_64-gen.c"), "vtop[-i] = tmp;", "memcpy(&vtop[-i], &tmp, sizeof(SValue));")?;
    replace_required_text(&root.join("tccpp.c"), "size = size * 2;", "size = size + size;")?;
    replace_required_text(&root.join("tccelf.c"), "size = size * 2;", "size = size + size;")?;
    replace_required_text(&root.join("libtcc.c"), "nb_alloc = nb * 2;", "nb_alloc = nb + nb;")?;
    replace_required_text(&root.join("tccpp.c"), "next->size * 2", "next->size + next->size")?;
    replace_required_text(
        &root.join("tccelf.c"),
        "rebuild_hash(s, 2 * nbuckets);",
        "rebuild_hash(s, nbuckets + nbuckets);",
    )?;
    replace_required_text(
        &root.join("x86_64-gen.c"),
        "loc = 0;",
        "loc = 0;\n    ind = cur_text_section->data_offset;",
    )?;
    replace_required_text(&root.join("x86_64-gen.c"), "g(vtop->c.i & (ll ? 63 : 31));", "g(vtop->c.i);")?;
    patch_tinycc_varargs_paths(root)?;
    replace_required_text(
        &root.join("tccgen.c"),
        "if (!is_compatible_types(&sym->type, type))",
        "if (0 && !is_compatible_types(&sym->type, type))",
    )?;
    replace_required_text(
        &root.join("tccelf.c"),
        "    if (file_type == TCC_OUTPUT_EXE && s1->static_link)\n        fill_got(s1);",
        "    if (file_type == TCC_OUTPUT_EXE && s1->static_link) {\n        if (s1->plt)\n            relocate_plt(s1);\n        fill_got(s1);\n    }",
    )?;
    assert!(
        !fs::read_to_string(root.join("x86_64-gen.c"))
            .map_err(|error| TinyccError::Materialization(format!("reading patched x86_64-gen.c: {error}")))?
            .contains("g(vtop->c.i & (ll ? 63 : 31));")
    );
    assert!(
        fs::read_to_string(root.join("tccelf.c"))
            .map_err(|error| TinyccError::Materialization(format!("reading patched tccelf.c: {error}")))?
            .contains("relocate_plt(s1);")
    );
    Ok(())
}

fn patch_tcctools_archive(root: &Path) -> Result<(), TinyccError> {
    let path = root.join("tcctools.c");
    let old = "    if ((fh = fopen(argv[i_lib], \"wb\")) == NULL)\n    {\n        fprintf(stderr, \"tcc: ar: can't open file %s \\n\", argv[i_lib]);\n        goto the_end;\n    }\n\n";
    replace_required_text(&path, old, "")?;
    let marker = "    // write header";
    let insertion = "    if ((fh = fopen(argv[i_lib], \"wb\")) == NULL)\n    {\n        fprintf(stderr, \"tcc: ar: cannot open file %s \\n\", argv[i_lib]);\n        goto the_end;\n    }\n\n    // write header";
    replace_required_text(&path, marker, insertion)?;
    assert!(
        fs::read_to_string(&path)
            .map_err(|error| TinyccError::Materialization(format!("reading patched tcctools.c: {error}")))?
            .contains("cannot open file")
    );
    Ok(())
}

fn patch_tinycc_varargs_paths(root: &Path) -> Result<(), TinyccError> {
    replace_required_text(
        &root.join("libtcc.c"),
        "        snprintf(buf, sizeof(buf), fmt, paths[i], filename);",
        "        if (fmt[3] == 'l') {\n            strcpy(buf, paths[i]); strcat(buf, \"/lib\"); strcat(buf, filename); strcat(buf, \".a\");\n        } else {\n            strcpy(buf, paths[i]); strcat(buf, \"/\"); strcat(buf, filename);\n        }",
    )?;
    replace_required_text(
        &root.join("tccpp.c"),
        "    sprintf(buf, \"\\\"%s\\\"\", file->filename);",
        "    buf[0] = '\"'; strcpy(buf + 1, file->filename); strcat(buf, \"\\\"\");",
    )?;
    replace_required_text(
        &root.join("libtcc.c"),
        "ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\n{\n    if (-1 == tcc_add_library_internal(s, \"%s/%s\",\n        filename, 0, s->crt_paths, s->nb_crt_paths))\n        tcc_error_noabort(\"file '%s' not found\", filename);\n    return 0;\n}",
        "ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\n{\n    char buf[1024];\n    int i;\n    if (!strcmp(filename, \"crti.o\") || !strcmp(filename, \"crtn.o\"))\n        return 0;\n    for (i = 0; i < s->nb_crt_paths; i++) {\n        strcpy(buf, s->crt_paths[i]); strcat(buf, \"/\"); strcat(buf, filename);\n        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\n            return 0;\n    }\n    tcc_error_noabort(\"file '%s' not found\", filename);\n    return 0;\n}",
    )?;
    assert!(
        !fs::read_to_string(root.join("libtcc.c"))
            .map_err(|error| TinyccError::Materialization(format!("reading patched libtcc.c: {error}")))?
            .contains("snprintf(buf, sizeof(buf), fmt")
    );
    assert!(
        !fs::read_to_string(root.join("tccpp.c"))
            .map_err(|error| TinyccError::Materialization(format!("reading patched tccpp.c: {error}")))?
            .contains("sprintf(buf, \"\\\"%s\\\"\"")
    );
    Ok(())
}

pub(crate) fn replace_required_text(path: &Path, old: &str, new: &str) -> Result<(), TinyccError> {
    let text = fs::read_to_string(path).map_err(|error| {
        TinyccError::Materialization(format!("reading TinyCC patch target {}: {error}", path.display()))
    })?;
    let match_count = text.matches(old).count();
    if match_count == 0 || match_count > TCC_MES_PATCH_MATCH_MAX {
        return Err(TinyccError::Materialization(format!(
            "TinyCC patch {} has {match_count} matches; required range is 1..={TCC_MES_PATCH_MATCH_MAX}",
            path.display()
        )));
    }
    let patched = text.replace(old, new);
    fs::write(path, patched.as_bytes()).map_err(|error| {
        TinyccError::Materialization(format!("writing TinyCC patch target {}: {error}", path.display()))
    })?;
    assert!(new.is_empty() || patched.contains(new));
    assert!(new.contains(old) || !patched.contains(old));
    Ok(())
}

fn patched_source_digest(root: &Path) -> Result<String, TinyccError> {
    let files = [
        "libtcc.c",
        "tccelf.c",
        "tccgen.c",
        "tccpp.c",
        "tcctools.c",
        "x86_64-gen.c",
    ];
    let mut hasher = blake3::Hasher::new();
    for relative in files {
        let bytes = crate::stagex_mes_lib::read_bounded_file(
            &root.join(relative),
            TCC_MES_FILE_BYTES_MAX,
            "patched TinyCC source",
        )?;
        let path_len = u64::try_from(relative.len())
            .map_err(|_| TinyccError::Materialization("TinyCC patch path length does not fit u64".to_string()))?;
        let byte_len = u64::try_from(bytes.len())
            .map_err(|_| TinyccError::Materialization("TinyCC patch byte length does not fit u64".to_string()))?;
        hasher.update(&path_len.to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&byte_len.to_le_bytes());
        hasher.update(&bytes);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!digest.is_empty());
    Ok(digest)
}

pub(crate) fn make_tree_owner_writable(root: &Path) -> Result<(), TinyccError> {
    use std::os::unix::fs::PermissionsExt;
    const ENTRY_COUNT_MAX: u32 = 30_000;
    const OWNER_WRITE_MODE: u32 = 0o200;
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0u32;
    while let Some(path) = pending.pop() {
        count = count
            .checked_add(1)
            .ok_or_else(|| TinyccError::Materialization("TinyCC permission walk overflow".to_string()))?;
        if count > ENTRY_COUNT_MAX {
            return Err(TinyccError::Materialization(format!(
                "TinyCC permission walk exceeds {ENTRY_COUNT_MAX} entries"
            )));
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| TinyccError::Materialization(format!("reading TinyCC path metadata: {error}")))?;
        if !metadata.file_type().is_symlink() {
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | OWNER_WRITE_MODE);
            fs::set_permissions(&path, permissions)
                .map_err(|error| TinyccError::Materialization(format!("making TinyCC path writable: {error}")))?;
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&path)
                .map_err(|error| TinyccError::Materialization(format!("reading TinyCC source directory: {error}")))?
            {
                pending.push(
                    entry
                        .map_err(|error| TinyccError::Materialization(format!("reading TinyCC source entry: {error}")))?
                        .path(),
                );
            }
        }
    }
    assert!(count > 0);
    assert!(count <= ENTRY_COUNT_MAX);
    Ok(())
}

pub(crate) fn set_owner_executable(path: &Path) -> Result<(), TinyccError> {
    use std::os::unix::fs::PermissionsExt;
    const EXECUTABLE_MODE: u32 = 0o755;
    let mut permissions = fs::metadata(path)
        .map_err(|error| TinyccError::Materialization(format!("reading TinyCC executable mode: {error}")))?
        .permissions();
    permissions.set_mode(EXECUTABLE_MODE);
    fs::set_permissions(path, permissions)
        .map_err(|error| TinyccError::Materialization(format!("setting TinyCC executable mode: {error}")))?;
    assert!(path.is_file());
    assert_eq!(
        fs::metadata(path).map(|metadata| metadata.permissions().mode() & EXECUTABLE_MODE).ok(),
        Some(EXECUTABLE_MODE)
    );
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), TinyccError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, TCC_MES_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(TinyccError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn select_tinycc_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, TinyccError> {
    select_fixed_source_record(records, TINYCC_RECORD_NAME, TINYCC_SOURCE_CONTENT_BLAKE3)
}

fn select_fixed_source_record<'a>(
    records: &'a [SourceRecord],
    record_name: &str,
    expected_content_blake3: &str,
) -> Result<&'a SourceRecord, TinyccError> {
    let matches = records
        .iter()
        .filter(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(record_name))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(TinyccError::InvalidAuthority(format!(
            "source bundle has {} records named {record_name}; expected exactly one",
            matches.len()
        )));
    }
    let record = matches[0];
    if record.kind != SourceRecordKind::FixedUrl
        || record.content_blake3 != expected_content_blake3
        || record.files.is_empty()
    {
        return Err(TinyccError::InvalidAuthority(format!(
            "source record {record_name} has substituted kind, digest, or empty payload: {}",
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
            identity: "fixed-url-tinycc".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::from([(SOURCE_RECORD_NAME_KEY.to_string(), TINYCC_RECORD_NAME.to_string())]),
            payload_bytes: 1,
            content_blake3: TINYCC_SOURCE_CONTENT_BLAKE3.to_string(),
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
    fn selects_exact_tinycc_source_and_rejects_substitution() {
        let valid = record();
        let selected = select_tinycc_source_record(std::slice::from_ref(&valid)).unwrap();
        assert_eq!(selected.content_blake3, TINYCC_SOURCE_CONTENT_BLAKE3);
        assert_eq!(selected.kind, SourceRecordKind::FixedUrl);

        let mut substituted = record();
        substituted.content_blake3 = TEST_BLAKE3.to_string();
        let error = select_tinycc_source_record(&[substituted]).unwrap_err();
        assert!(error.to_string().contains("substituted kind, digest, or empty payload"));
        assert!(!error.to_string().is_empty());
    }

    #[test]
    #[ignore = "requires explicit TinyCC, Mes, Stage0, and create-new scratch inputs"]
    fn derives_retained_tcc_mes_inventory() {
        let tcc_source = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC_SOURCE_ROOT").unwrap());
        let mes_source = PathBuf::from(std::env::var("MANTLE_STAGE_X_MES_SOURCE_ROOT").unwrap());
        let mes_runtime = PathBuf::from(std::env::var("MANTLE_STAGE_X_MES_RUNTIME_ROOT").unwrap());
        let stage0 = PathBuf::from(std::env::var("MANTLE_STAGE_X_STAGE0_FULL_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TCC_MES_SCRATCH").unwrap());

        let report = derive_tcc_mes_inventory(TccMesInventoryRequest {
            tinycc_source_root: &tcc_source,
            mes_source_root: &mes_source,
            mes_runtime_root: &mes_runtime,
            mes_m2_path: &mes_runtime.join("bin/mes-m2"),
            stage0_root: &stage0,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();

        assert_eq!(report.outputs.len(), TCC_MES_OUTPUT_COUNT);
        assert_eq!(report.compile_command_count, TCC_MES_COMPILE_COMMAND_COUNT);
        assert!(report.outputs.iter().all(|output| output.bytes_len > 0));
        assert!(scratch.join("tcc-mes-inventory.json").is_file());
    }

    #[test]
    #[ignore = "requires explicit authenticated source bundle and create-new scratch"]
    fn materializes_retained_tinycc_source() {
        let bundle = PathBuf::from(std::env::var("MANTLE_STAGE_X_SOURCE_BUNDLE").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_TINYCC_SOURCE_SCRATCH").unwrap());

        let report = materialize_authenticated_tinycc_source(
            &bundle,
            crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            &scratch,
        )
        .unwrap();
        fs::write(scratch.join("tinycc-source-materialization.json"), serde_json::to_vec_pretty(&report).unwrap())
            .unwrap();

        assert!(report.output_path.is_dir());
        assert_eq!(report.record_content_blake3, TINYCC_SOURCE_CONTENT_BLAKE3);
        assert!(scratch.join("tinycc-source-materialization.json").is_file());
    }
}
