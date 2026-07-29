use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::materialize_source_record_for_offline_use;
use crate::source_bundle::read_source_bundle;

const SOURCE_RECORD_NAME_KEY: &str = "name";
const GREP_RECORD_NAME: &str = "grep-2.4-src";
const GREP_RECORD_IDENTITY: &str = "fixed-url-55d3b8c817bb1fe5fda738e69dfbc6b2985166f020ab2c5d7ff56e208e96c821";
const GREP_SOURCE_OUTPUT_NAME: &str = "grep-2.4";
pub(crate) const GREP_SOURCE_ARTIFACT_ID: &str = "grep-2.4-source";
pub(crate) const GREP_SOURCE_CONTENT_BLAKE3: &str = "b40c519f58ef370f44bbafc37a76ae843a3cf621a44ef83158294fb29ada14d0";
pub(crate) const GREP_RECIPE_ARTIFACT_ID: &str = "grep-2.4-recipe-source";
pub(crate) const GREP_RECIPE_BLAKE3: &str = "15afa14595fb8c84cd5e1ce8d0807ed853a2177146a4e585bcedd90b3d589ad6";
pub(crate) const GREP_RUNNER_SOURCE_ARTIFACT_ID: &str = "grep-2.4-bridge-runner-source";
pub(crate) const GREP_RUNNER_SOURCE_BLAKE3: &str = "926cba44f7f96479db1e73a85e428346eccc380ca463704fa7f2e68905e9363f";
const GREP_RECIPE: &[u8] = include_bytes!("../bootstrap/grep-2.4-musl.ncl");
const GREP_RUNNER_SOURCE: &[u8] = include_bytes!("../bootstrap/stagex-grep-bridge-runner.c");
const GREP_SOURCE_REPORT_FORMAT: &str = "mantle-stagex-grep-source-materialization-v1";
const GREP_SOURCE_NON_CLAIM: &str =
    "grep source materialization proves authenticated offline archive identity and fixed-output parity only";
const GREP_REPORT_FORMAT: &str = "mantle-stagex-grep-2.4-inventory-v1";
const GREP_NON_CLAIM: &str = "this inventory binds the exact checked GNU grep 2.4 bridge as data, a native runner for its declared subset, and bounded literal, quiet, and anchored-pattern observations only; it does not prove shell execution, general regular-expression behavior, parser generators, binutils, native TinyCC, or provider admission";
const GREP_RECORD_HASH: &str = "sha256-v9xxK5dLi3FxvpCbRjPorpK+bPNcAXwj/jgX4eslWxI=";
const GREP_RECORD_URL: &str = "https://mirrors.kernel.org/gnu/grep/grep-2.4.tar.gz";
const GREP_RECORD_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const GREP_RECORD_UNPACK: &str = "1";
const GREP_BRIDGE_START: &[u8] = b"      $BB cat > grep <<'EOF'\n";
const GREP_BRIDGE_END: &[u8] = b"\nEOF\n";
const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const GREP_OBSERVATION_KIBIBYTES_MAX: u64 = 64;
const GREP_OBSERVATION_BYTES_MAX: u64 = GREP_OBSERVATION_KIBIBYTES_MAX * KIBIBYTE_BYTES;
const GREP_ARTIFACT_MEBIBYTES_MAX: u64 = 64;
const GREP_ARTIFACT_BYTES_MAX: u64 = GREP_ARTIFACT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const GREP_SOURCE_ARTIFACT_COUNT: usize = 3;
const GREP_BUILD_COMMAND_COUNT: u32 = 2;
const GREP_SMOKE_COMMAND_COUNT: u32 = 3;
const GREP_OUTPUT_COUNT: usize = 7;
const GREP_RUNNER_MODE: u32 = 0o755;
const GREP_BRIDGE_DATA_MODE: u32 = 0o644;
const GREP_OBJECT_MODE: u32 = 0o644;
const GREP_EXECUTE_MODE_MASK: u32 = 0o111;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHAR_COUNT: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const GREP_LITERAL_INPUT: &[u8] = b"hello\n";
const GREP_QUIET_INPUT: &[u8] = b"HELLO\n";
const GREP_ANCHORED_INPUT: &[u8] = b"abc\n";
const GREP_QUIET_OUTPUT: &[u8] = b"";

pub(crate) const GREP_CONFIGURED_SOURCE_BLAKE3: &str =
    "d86dde390ef4ab7b4456f0371cbd29afa8b0e31ca297e7acc85b5f9544775c40";
pub(crate) const GREP_RUNNER_BLAKE3: &str = "b4adac2bf1f29e3e3b8ec736207a0cdfc3eb5b51c658a2f9f8eff85aee4ef71d";
pub(crate) const GREP_BRIDGE_BLAKE3: &str = "3da22b084c98e1ebef50ddcbe404c681d6a7d81b29389d26151729b1c74127de";
const GREP_LITERAL_OBSERVATION_BLAKE3: &str = "8e4c7c1b99dbfd50e7a95185fead5ee1448fa904a2fdd778eaf5f2dbfd629a99";
const GREP_QUIET_OBSERVATION_BLAKE3: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
const GREP_ANCHORED_OBSERVATION_BLAKE3: &str = "aa95faeede7041e63c6056bdcf10e6fbf709a355e539259da51a067e5dd27802";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GrepExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const GREP_EXPECTED_OUTPUTS: [GrepExpectedOutput; GREP_OUTPUT_COUNT] = [
    GrepExpectedOutput {
        artifact_id: "grep-2.4-runner",
        digest_blake3: GREP_RUNNER_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "egrep-2.4-runner",
        digest_blake3: GREP_RUNNER_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "fgrep-2.4-runner",
        digest_blake3: GREP_RUNNER_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "grep-2.4-bridge-script",
        digest_blake3: GREP_BRIDGE_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "grep-literal-observation",
        digest_blake3: GREP_LITERAL_OBSERVATION_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "grep-quiet-observation",
        digest_blake3: GREP_QUIET_OBSERVATION_BLAKE3,
    },
    GrepExpectedOutput {
        artifact_id: "grep-anchored-observation",
        digest_blake3: GREP_ANCHORED_OBSERVATION_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GrepSourceMaterializationReport {
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
pub(crate) struct GrepOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GrepInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<GrepOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GrepInventoryRequest<'a> {
    pub source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum StagexGrepError {
    SourceRecordNotFound,
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexGrepError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRecordNotFound => write!(formatter, "GNU grep source record was not found"),
            Self::Materialization(message) => write!(formatter, "GNU grep materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "GNU grep runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexGrepError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexGrepError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); GREP_SOURCE_ARTIFACT_COUNT] {
    [
        (GREP_SOURCE_ARTIFACT_ID, GREP_SOURCE_CONTENT_BLAKE3),
        (GREP_RECIPE_ARTIFACT_ID, GREP_RECIPE_BLAKE3),
        (GREP_RUNNER_SOURCE_ARTIFACT_ID, GREP_RUNNER_SOURCE_BLAKE3),
    ]
}

pub(crate) fn materialize_authenticated_grep_source(
    bundle_path: &Path,
    expected_manifest_blake3: &str,
    scratch_dir: &Path,
) -> Result<GrepSourceMaterializationReport, StagexGrepError> {
    validate_bundle_path(bundle_path)?;
    let manifest = read_source_bundle(bundle_path)
        .map_err(|error| StagexGrepError::Materialization(format!("reading source bundle: {error}")))?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(StagexGrepError::Materialization(format!(
            "source-bundle manifest mismatch: expected {expected_manifest_blake3}, observed {}",
            manifest.manifest_blake3
        )));
    }
    let record = find_source_record(&manifest.records)?;
    validate_source_record(record)?;
    fs::create_dir(scratch_dir)
        .map_err(|error| StagexGrepError::Materialization(format!("creating grep source scratch: {error}")))?;
    let output_path = scratch_dir.join(GREP_SOURCE_OUTPUT_NAME);
    materialize_source_record_for_offline_use(record, &output_path)
        .map_err(|error| StagexGrepError::Materialization(format!("materializing grep source: {error}")))?;
    validate_materialized_source(&output_path)?;
    let materialization = GrepSourceMaterializationReport {
        format: GREP_SOURCE_REPORT_FORMAT,
        source_bundle_manifest_blake3: manifest.manifest_blake3,
        artifact_id: GREP_SOURCE_ARTIFACT_ID,
        record_name: GREP_RECORD_NAME,
        record_identity: record.identity.clone(),
        record_content_blake3: record.content_blake3.clone(),
        output_path,
        non_claim: GREP_SOURCE_NON_CLAIM,
    };
    assert_eq!(materialization.record_identity, GREP_RECORD_IDENTITY);
    assert!(materialization.output_path.join("src/grep.c").is_file());
    Ok(materialization)
}

pub(crate) fn derive_grep_inventory(request: GrepInventoryRequest<'_>) -> Result<GrepInventoryReport, StagexGrepError> {
    validate_inventory_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexGrepError::Materialization(format!("creating grep runtime scratch: {error}")))?;
    let retained_source = request.scratch_dir.join(GREP_SOURCE_OUTPUT_NAME);
    crate::stagex_mes_lib::copy_tree_bounded(request.source_root, &retained_source)?;
    let runner_source = retained_source.join("stagex-grep-bridge-runner.c");
    crate::stagex_mes_lib::write_create_new(&runner_source, GREP_RUNNER_SOURCE)?;
    let output_bin = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_bin)
        .map_err(|error| StagexGrepError::Materialization(format!("creating grep output: {error}")))?;
    let grep = build_native_runner(&request, &retained_source, &output_bin)?;
    let egrep = output_bin.join("egrep");
    let fgrep = output_bin.join("fgrep");
    symlink("grep", &egrep)
        .map_err(|error| StagexGrepError::Materialization(format!("creating egrep alias: {error}")))?;
    symlink("grep", &fgrep)
        .map_err(|error| StagexGrepError::Materialization(format!("creating fgrep alias: {error}")))?;
    let bridge = write_bound_bridge_data(request.scratch_dir)?;
    let observations = run_checked_smokes(&request, &grep)?;
    let outputs = collect_outputs(&grep, &egrep, &fgrep, &bridge, &observations)?;
    validate_expected_outputs(&outputs)?;
    let inventory = GrepInventoryReport {
        format: GREP_REPORT_FORMAT,
        configured_source_digest_blake3: configured_source_digest_blake3()?,
        build_command_count: GREP_BUILD_COMMAND_COUNT,
        smoke_command_count: GREP_SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: GREP_NON_CLAIM,
    };
    let inventory_path = request.scratch_dir.join("grep-inventory.json");
    let bytes = serde_json::to_vec_pretty(&inventory)
        .map_err(|error| StagexGrepError::Materialization(format!("serializing grep inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&inventory_path, &bytes)?;
    assert_eq!(inventory.outputs.len(), GREP_OUTPUT_COUNT);
    assert!(inventory.fallback_events.is_empty());
    Ok(inventory)
}

fn build_native_runner(
    request: &GrepInventoryRequest<'_>,
    source_root: &Path,
    output_bin: &Path,
) -> Result<PathBuf, StagexGrepError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object_relative = "stagex-grep-bridge-runner.o";
    let object = source_root.join(object_relative);
    let compile_args = vec![
        "-c".to_string(),
        format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?),
        "stagex-grep-bridge-runner.c".to_string(),
        "-o".to_string(),
        object_relative.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &compile_args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("grep-runner-compile.stderr.txt"),
    )?;
    fs::set_permissions(&object, fs::Permissions::from_mode(GREP_OBJECT_MODE))
        .map_err(|error| StagexGrepError::Materialization(format!("setting grep runner object mode: {error}")))?;
    validate_nonempty_artifact(&object, "grep runner object")?;
    let grep = output_bin.join("grep");
    let link_args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_absolute(&grep, "grep runner output")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
        object_relative.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &link_args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("grep-runner-link.stderr.txt"),
    )?;
    fs::set_permissions(&grep, fs::Permissions::from_mode(GREP_RUNNER_MODE))
        .map_err(|error| StagexGrepError::Materialization(format!("setting grep runner mode: {error}")))?;
    validate_nonempty_artifact(&grep, "grep runner")?;
    assert_eq!(object.metadata().map(|value| value.permissions().mode() & 0o777).unwrap_or(0), GREP_OBJECT_MODE);
    assert!(
        grep.metadata()
            .map(|value| value.permissions().mode() & GREP_EXECUTE_MODE_MASK != 0)
            .unwrap_or(false)
    );
    Ok(grep)
}

fn write_bound_bridge_data(scratch_dir: &Path) -> Result<PathBuf, StagexGrepError> {
    let share = scratch_dir.join("output/share/grep-2.4");
    fs::create_dir_all(&share)
        .map_err(|error| StagexGrepError::Materialization(format!("creating grep bridge data root: {error}")))?;
    let bridge = share.join("bootstrap-bridge.sh");
    crate::stagex_mes_lib::write_create_new(&bridge, &grep_bridge_bytes()?)?;
    fs::set_permissions(&bridge, fs::Permissions::from_mode(GREP_BRIDGE_DATA_MODE))
        .map_err(|error| StagexGrepError::Materialization(format!("setting grep bridge data mode: {error}")))?;
    assert_eq!(
        bridge.metadata().map(|value| value.permissions().mode() & 0o777).unwrap_or(0),
        GREP_BRIDGE_DATA_MODE
    );
    assert!(bridge.is_file());
    Ok(bridge)
}

struct GrepObservations {
    literal: PathBuf,
    quiet: PathBuf,
    anchored: PathBuf,
}

fn run_checked_smokes(request: &GrepInventoryRequest<'_>, grep: &Path) -> Result<GrepObservations, StagexGrepError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexGrepError::Materialization(format!("creating grep smoke root: {error}")))?;
    let literal = run_smoke(grep, &smoke, "literal", &["hello"], GREP_LITERAL_INPUT)?;
    require_exact_bytes(&literal, GREP_LITERAL_INPUT, "grep literal observation")?;
    let quiet = run_smoke(grep, &smoke, "quiet", &["-q", "HELLO"], GREP_QUIET_INPUT)?;
    require_exact_bytes(&quiet, GREP_QUIET_OUTPUT, "grep quiet observation")?;
    let anchored = run_smoke(grep, &smoke, "anchored", &["-E", "^ab"], GREP_ANCHORED_INPUT)?;
    require_exact_bytes(&anchored, GREP_ANCHORED_INPUT, "grep anchored observation")?;
    assert!(literal.is_file());
    assert!(anchored.is_file());
    Ok(GrepObservations {
        literal,
        quiet,
        anchored,
    })
}

fn run_smoke(
    grep: &Path,
    smoke: &Path,
    label: &str,
    grep_args: &[&str],
    stdin_bytes: &[u8],
) -> Result<PathBuf, StagexGrepError> {
    let stdin_path = smoke.join(format!("{label}.in"));
    let stdout_path = smoke.join(format!("{label}.out"));
    let stderr_path = smoke.join(format!("{label}.stderr.txt"));
    crate::stagex_mes_lib::write_create_new(&stdin_path, stdin_bytes)?;
    let args = grep_args.iter().map(|argument| (*argument).to_string()).collect::<Vec<_>>();
    crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout(
        grep,
        &args,
        smoke,
        &BTreeMap::<String, String>::new(),
        &stdin_path,
        GREP_OBSERVATION_BYTES_MAX,
        &stdout_path,
        GREP_OBSERVATION_BYTES_MAX,
        &stderr_path,
    )?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(stdout_path)
}

fn collect_outputs(
    grep: &Path,
    egrep: &Path,
    fgrep: &Path,
    bridge: &Path,
    observations: &GrepObservations,
) -> Result<Vec<GrepOutputReport>, StagexGrepError> {
    let paths = [
        ("grep-2.4-runner", grep),
        ("egrep-2.4-runner", egrep),
        ("fgrep-2.4-runner", fgrep),
        ("grep-2.4-bridge-script", bridge),
        ("grep-literal-observation", observations.literal.as_path()),
        ("grep-quiet-observation", observations.quiet.as_path()),
        ("grep-anchored-observation", observations.anchored.as_path()),
    ];
    let outputs = paths
        .iter()
        .map(|(artifact_id, path)| output_report(artifact_id, path))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), GREP_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == BLAKE3_HEX_CHAR_COUNT));
    Ok(outputs)
}

fn validate_inventory_inputs(request: &GrepInventoryRequest<'_>) -> Result<(), StagexGrepError> {
    if request.scratch_dir.exists() {
        return Err(StagexGrepError::Materialization(format!(
            "grep runtime scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    validate_materialized_source(request.source_root)?;
    validate_recipe_digest()?;
    validate_runner_source_digest()?;
    let configured_source = configured_source_digest_blake3()?;
    if configured_source != GREP_CONFIGURED_SOURCE_BLAKE3 {
        return Err(StagexGrepError::Materialization(format!(
            "grep configured-source BLAKE3 mismatch: expected {GREP_CONFIGURED_SOURCE_BLAKE3}, observed {configured_source}"
        )));
    }
    validate_file_digest(
        &request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2"),
        crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        "TinyCC musl-v2",
    )?;
    let libc_digest = native_musl_digest("musl-native-libc")?;
    let crt1_digest = native_musl_digest("musl-native-crt1")?;
    validate_file_digest(&request.musl_native_root.join("lib/libc.a"), libc_digest, "native musl libc")?;
    validate_file_digest(&request.musl_native_root.join("lib/crt1.o"), crt1_digest, "native musl crt1")?;
    if !request.musl_native_root.join("include/stdio.h").is_file() {
        return Err(StagexGrepError::Materialization("native musl headers are missing".to_string()));
    }
    assert!(!request.scratch_dir.exists());
    assert!(request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2").is_file());
    Ok(())
}

fn validate_bundle_path(path: &Path) -> Result<(), StagexGrepError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(StagexGrepError::Materialization(format!(
            "source-bundle path is not an absolute file: {}",
            path.display()
        )));
    }
    assert!(path.is_absolute());
    assert!(path.is_file());
    Ok(())
}

fn validate_materialized_source(path: &Path) -> Result<(), StagexGrepError> {
    if !path.is_absolute() || !path.join("src/grep.c").is_file() || !path.join("src/dfa.c").is_file() {
        return Err(StagexGrepError::Materialization(format!("grep source lacks required files: {}", path.display())));
    }
    assert!(path.join("src/grep.c").is_file());
    assert!(path.join("src/dfa.c").is_file());
    Ok(())
}

fn find_source_record(records: &[SourceRecord]) -> Result<&SourceRecord, StagexGrepError> {
    records
        .iter()
        .find(|record| record.metadata.get(SOURCE_RECORD_NAME_KEY).map(String::as_str) == Some(GREP_RECORD_NAME))
        .ok_or(StagexGrepError::SourceRecordNotFound)
}

fn validate_source_record(record: &SourceRecord) -> Result<(), StagexGrepError> {
    let expected_metadata = [
        ("builder", "builtin:fetchurl"),
        ("hash", GREP_RECORD_HASH),
        ("hash_algo", "sha256"),
        ("hash_mode", "recursive"),
        ("name", GREP_RECORD_NAME),
        ("payload_encoding", GREP_RECORD_PAYLOAD_ENCODING),
        ("unpack", GREP_RECORD_UNPACK),
        ("url", GREP_RECORD_URL),
    ];
    let metadata_matches = expected_metadata
        .iter()
        .all(|(key, value)| record.metadata.get(*key).map(String::as_str) == Some(*value));
    if record.kind != SourceRecordKind::FixedUrl
        || record.identity != GREP_RECORD_IDENTITY
        || record.content_blake3 != GREP_SOURCE_CONTENT_BLAKE3
        || !metadata_matches
    {
        return Err(StagexGrepError::Materialization(format!(
            "grep source record {} has substituted authority",
            record.identity
        )));
    }
    if record.files.len() != 1 || record.files.iter().any(|entry| entry.file_type != SourceFileType::Regular) {
        return Err(StagexGrepError::Materialization(
            "grep source record must contain exactly one regular archive payload".to_string(),
        ));
    }
    assert_eq!(record.kind, SourceRecordKind::FixedUrl);
    assert_eq!(record.identity, GREP_RECORD_IDENTITY);
    Ok(())
}

fn validate_recipe_digest() -> Result<(), StagexGrepError> {
    let observed = blake3::hash(GREP_RECIPE).to_hex().to_string();
    if observed != GREP_RECIPE_BLAKE3 {
        return Err(StagexGrepError::Materialization(format!(
            "grep recipe BLAKE3 mismatch: expected {GREP_RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert_eq!(observed, GREP_RECIPE_BLAKE3);
    assert!(!GREP_RECIPE.is_empty());
    Ok(())
}

fn validate_runner_source_digest() -> Result<(), StagexGrepError> {
    let observed = blake3::hash(GREP_RUNNER_SOURCE).to_hex().to_string();
    if observed != GREP_RUNNER_SOURCE_BLAKE3 {
        return Err(StagexGrepError::Materialization(format!(
            "grep runner source BLAKE3 mismatch: expected {GREP_RUNNER_SOURCE_BLAKE3}, observed {observed}"
        )));
    }
    if GREP_RUNNER_SOURCE.windows("system(".len()).any(|window| window == b"system(") {
        return Err(StagexGrepError::Materialization(
            "grep runner source contains ambient shell execution".to_string(),
        ));
    }
    assert_eq!(observed, GREP_RUNNER_SOURCE_BLAKE3);
    assert!(GREP_RUNNER_SOURCE.starts_with(b"#include <assert.h>\n"));
    Ok(())
}

fn native_musl_digest(artifact_id: &str) -> Result<&'static str, StagexGrepError> {
    crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.digest_blake3)
        .ok_or_else(|| StagexGrepError::Materialization(format!("native musl identity {artifact_id} is missing")))
}

fn grep_bridge_bytes() -> Result<Vec<u8>, StagexGrepError> {
    let start = find_subslice(GREP_RECIPE, GREP_BRIDGE_START)
        .ok_or_else(|| StagexGrepError::Materialization("grep bridge start marker is missing".to_string()))?;
    let body_start = start
        .checked_add(GREP_BRIDGE_START.len())
        .ok_or_else(|| StagexGrepError::Materialization("grep bridge start offset overflow".to_string()))?;
    let relative_end = find_subslice(&GREP_RECIPE[body_start..], GREP_BRIDGE_END)
        .ok_or_else(|| StagexGrepError::Materialization("grep bridge end marker is missing".to_string()))?;
    let body_end = body_start
        .checked_add(relative_end)
        .ok_or_else(|| StagexGrepError::Materialization("grep bridge end offset overflow".to_string()))?;
    let bridge = GREP_RECIPE[body_start..body_end].to_vec();
    if bridge.is_empty() || !bridge.starts_with(b"#!/bin/sh\n") {
        return Err(StagexGrepError::Materialization("grep bridge bytes are malformed".to_string()));
    }
    assert!(bridge.windows(b"scan_stream".len()).any(|window| window == b"scan_stream"));
    assert!(bridge.ends_with(b"exit $status"));
    Ok(bridge)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}

pub(crate) fn configured_source_digest_blake3() -> Result<String, StagexGrepError> {
    let bridge = grep_bridge_bytes()?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-grep-configured-source-v1\0");
    hasher.update(GREP_SOURCE_CONTENT_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(GREP_RECIPE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(GREP_RUNNER_SOURCE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    hasher.update(&bridge);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn validate_expected_outputs(outputs: &[GrepOutputReport]) -> Result<(), StagexGrepError> {
    if outputs.len() != GREP_EXPECTED_OUTPUTS.len() {
        return Err(StagexGrepError::Materialization(format!(
            "expected {} grep outputs, observed {}",
            GREP_EXPECTED_OUTPUTS.len(),
            outputs.len()
        )));
    }
    let observed = outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), output.digest_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mismatches = GREP_EXPECTED_OUTPUTS
        .iter()
        .filter_map(|expected| {
            let actual = observed.get(expected.artifact_id).copied().unwrap_or("missing");
            (actual != expected.digest_blake3).then(|| format!("{}={actual}", expected.artifact_id))
        })
        .collect::<Vec<_>>();
    if !mismatches.is_empty() {
        return Err(StagexGrepError::Materialization(format!(
            "grep output BLAKE3 mismatches: {}",
            mismatches.join(", ")
        )));
    }
    assert!(mismatches.is_empty());
    assert_eq!(outputs.len(), GREP_OUTPUT_COUNT);
    Ok(())
}

fn output_report(artifact_id: &str, path: &Path) -> Result<GrepOutputReport, StagexGrepError> {
    let bytes = read_bounded_artifact(path, artifact_id)?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexGrepError::Materialization(format!("{artifact_id} byte count exceeds u64")))?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_CHAR_COUNT);
    assert!(path.is_file());
    Ok(GrepOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn validate_nonempty_artifact(path: &Path, label: &str) -> Result<(), StagexGrepError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexGrepError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > GREP_ARTIFACT_BYTES_MAX {
        return Err(StagexGrepError::Materialization(format!(
            "{label} is empty, not regular, or exceeds {GREP_ARTIFACT_BYTES_MAX} bytes"
        )));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
    Ok(())
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexGrepError> {
    let observed = read_bounded_file(path, label)?;
    if observed != expected {
        return Err(StagexGrepError::Materialization(format!("{label} bytes differ")));
    }
    assert_eq!(observed, expected);
    assert!(path.is_file());
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexGrepError> {
    let observed = blake3::hash(&read_bounded_artifact(path, label)?).to_hex().to_string();
    if observed != expected {
        return Err(StagexGrepError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed, expected);
    assert!(path.is_file());
    Ok(())
}

fn read_bounded_file(path: &Path, label: &str) -> Result<Vec<u8>, StagexGrepError> {
    read_bounded_file_with_max(path, label, GREP_OBSERVATION_BYTES_MAX)
}

fn read_bounded_artifact(path: &Path, label: &str) -> Result<Vec<u8>, StagexGrepError> {
    read_bounded_file_with_max(path, label, GREP_ARTIFACT_BYTES_MAX)
}

fn read_bounded_file_with_max(path: &Path, label: &str, bytes_max: u64) -> Result<Vec<u8>, StagexGrepError> {
    let bytes =
        fs::read(path).map_err(|error| StagexGrepError::Materialization(format!("reading {label}: {error}")))?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexGrepError::Materialization(format!("{label} byte count exceeds u64")))?;
    if bytes_len > bytes_max {
        return Err(StagexGrepError::Materialization(format!("{label} exceeds {bytes_max} bytes")));
    }
    assert!(bytes_len <= bytes_max);
    assert!(path.is_file());
    Ok(bytes)
}

fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexGrepError> {
    if !path.is_absolute() {
        return Err(StagexGrepError::Materialization(format!("{label} path is not absolute")));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexGrepError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_bundle::SourceFileEntry;

    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_SOURCE_ROOT_ENV: &str = "MANTLE_STAGE_X_GREP_SOURCE_ROOT";
    const RETAINED_TCC_V2_ROOT_ENV: &str = "MANTLE_STAGE_X_TCC_MUSL_V2_ROOT";
    const RETAINED_MUSL_NATIVE_ROOT_ENV: &str = "MANTLE_STAGE_X_MUSL_NATIVE_ROOT";
    const RETAINED_BUILD_SCRATCH_ENV: &str = "MANTLE_STAGE_X_GREP_BUILD_SCRATCH";

    fn source_record(kind: SourceRecordKind, identity: &str, content_blake3: &str) -> SourceRecord {
        SourceRecord {
            kind,
            identity: identity.to_string(),
            store_prefix: None,
            adapter: None,
            metadata: BTreeMap::from([
                ("builder".to_string(), "builtin:fetchurl".to_string()),
                ("hash".to_string(), GREP_RECORD_HASH.to_string()),
                ("hash_algo".to_string(), "sha256".to_string()),
                ("hash_mode".to_string(), "recursive".to_string()),
                ("name".to_string(), GREP_RECORD_NAME.to_string()),
                ("payload_encoding".to_string(), GREP_RECORD_PAYLOAD_ENCODING.to_string()),
                ("unpack".to_string(), GREP_RECORD_UNPACK.to_string()),
                ("url".to_string(), GREP_RECORD_URL.to_string()),
            ]),
            content_blake3: content_blake3.to_string(),
            payload_bytes: 1,
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
        }
    }

    #[test]
    fn validates_exact_grep_source_record_recipe_and_bridge() {
        let record = source_record(SourceRecordKind::FixedUrl, GREP_RECORD_IDENTITY, GREP_SOURCE_CONTENT_BLAKE3);
        validate_source_record(&record).unwrap();
        validate_recipe_digest().unwrap();
        validate_runner_source_digest().unwrap();
        let bridge = grep_bridge_bytes().unwrap();
        assert!(bridge.starts_with(b"#!/bin/sh\n"));
        assert!(bridge.ends_with(b"exit $status"));
    }

    #[test]
    fn rejects_substituted_grep_source_authority() {
        let wrong_kind = source_record(SourceRecordKind::VcsSnapshot, GREP_RECORD_IDENTITY, GREP_SOURCE_CONTENT_BLAKE3);
        let wrong_identity =
            source_record(SourceRecordKind::FixedUrl, "fixed-url-substituted", GREP_SOURCE_CONTENT_BLAKE3);
        let wrong_digest =
            source_record(SourceRecordKind::FixedUrl, GREP_RECORD_IDENTITY, &"a".repeat(BLAKE3_HEX_CHAR_COUNT));
        assert!(validate_source_record(&wrong_kind).is_err());
        assert!(validate_source_record(&wrong_identity).is_err());
        assert!(validate_source_record(&wrong_digest).is_err());
    }

    #[test]
    fn pins_deterministic_grep_identities() {
        let bridge = grep_bridge_bytes().unwrap();
        assert_eq!(configured_source_digest_blake3().unwrap(), GREP_CONFIGURED_SOURCE_BLAKE3);
        assert_eq!(blake3::hash(&bridge).to_hex().as_str(), GREP_BRIDGE_BLAKE3);
        assert_eq!(blake3::hash(GREP_LITERAL_INPUT).to_hex().as_str(), GREP_LITERAL_OBSERVATION_BLAKE3);
        assert_eq!(blake3::hash(GREP_QUIET_OUTPUT).to_hex().as_str(), GREP_QUIET_OBSERVATION_BLAKE3);
        assert_eq!(blake3::hash(GREP_ANCHORED_INPUT).to_hex().as_str(), GREP_ANCHORED_OBSERVATION_BLAKE3);
    }

    #[test]
    fn rejects_substituted_grep_output_identity() {
        let outputs = GREP_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| GrepOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(expected.artifact_id),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_expected_outputs(&outputs).unwrap();
        let mut substituted = outputs;
        substituted[0].digest_blake3 = "a".repeat(BLAKE3_HEX_CHAR_COUNT);
        assert!(validate_expected_outputs(&substituted).is_err());
    }

    #[test]
    #[ignore = "requires retained authenticated source bundle"]
    fn materializes_retained_grep_source() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let expected_manifest = read_source_bundle(&bundle).unwrap().manifest_blake3;
        let materialization = materialize_authenticated_grep_source(&bundle, &expected_manifest, &scratch).unwrap();
        assert_eq!(materialization.record_identity, GREP_RECORD_IDENTITY);
        assert!(materialization.output_path.join("src/grep.c").is_file());
    }

    #[test]
    #[ignore = "requires retained GNU grep source, TinyCC musl-v2, native musl, and create-new scratch"]
    fn derives_retained_grep_inventory() {
        let source_root = PathBuf::from(std::env::var(RETAINED_SOURCE_ROOT_ENV).unwrap());
        let tcc_musl_v2_root = PathBuf::from(std::env::var(RETAINED_TCC_V2_ROOT_ENV).unwrap());
        let musl_native_root = PathBuf::from(std::env::var(RETAINED_MUSL_NATIVE_ROOT_ENV).unwrap());
        let scratch_dir = PathBuf::from(std::env::var(RETAINED_BUILD_SCRATCH_ENV).unwrap());
        let inventory = derive_grep_inventory(GrepInventoryRequest {
            source_root: &source_root,
            tcc_musl_v2_root: &tcc_musl_v2_root,
            musl_native_root: &musl_native_root,
            scratch_dir: &scratch_dir,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(inventory.outputs.len(), GREP_OUTPUT_COUNT);
        assert!(inventory.fallback_events.is_empty());
    }
}
