use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

const KIBIBYTE_BYTES: u64 = 1_024;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const BASH_FULL_ARTIFACT_MEBIBYTES_MAX: u64 = 64;
const BASH_FULL_ARTIFACT_BYTES_MAX: u64 = BASH_FULL_ARTIFACT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const BASH_FULL_EXECUTABLE_MODE: u32 = 0o755;
const BASH_FULL_REGULAR_MODE: u32 = 0o644;
const BASH_FULL_BUILTIN_DEFINITION_COUNT: usize = 43;
const BASH_FULL_GENERATED_FILE_COUNT: usize = 42;
const BASH_FULL_PIPE_SIZE_BYTES: u32 = 512;
const BASH_FULL_GENERATOR_BUILD_COMMAND_COUNT: u32 = 2;
const BASH_FULL_GENERATOR_COMMAND_COUNT: u32 = 40;
const BASH_FULL_GENERATOR_NON_CLAIM: &str = "this generator probe binds Bash 2.05b mkbuiltins output only; it does not prove a full shell, configure compatibility, binutils, or provider admission";
const BASH_FULL_STATIC_BUILTIN_SOURCE_COUNT: usize = 5;
const BASH_FULL_GENERATED_BUILTIN_SOURCE_COUNT: usize = 39;
const BASH_FULL_AUXILIARY_SOURCE_COUNT: usize = 2;
const BASH_FULL_OMITTED_SH_SOURCE_COUNT: usize = 2;
const BASH_FULL_TILDE_SOURCE_COUNT: usize = 1;
const BASH_FULL_SOURCE_COMPILE_COUNT: usize = crate::stagex_bash::CORE_SOURCES.len()
    + BASH_FULL_AUXILIARY_SOURCE_COUNT
    + crate::stagex_bash::GLOB_SOURCES.len()
    + BASH_FULL_TILDE_SOURCE_COUNT
    + crate::stagex_bash::SH_LIBRARY_SOURCES.len()
    - BASH_FULL_OMITTED_SH_SOURCE_COUNT
    + BASH_FULL_STATIC_BUILTIN_SOURCE_COUNT
    + BASH_FULL_GENERATED_BUILTIN_SOURCE_COUNT
    + 1;
const BASH_FULL_BUILD_COMMAND_COUNT: u32 = 133;
const BASH_FULL_SMOKE_COMMAND_COUNT: u32 = 5;
const BASH_FULL_OUTPUT_COUNT: usize = 6;
const BASH_FULL_NON_CLAIM: &str = "this inventory binds a non-interactive single-thread Bash 2.05b with generated real builtins, bounded native-musl compatibility, and declared shell observations only; it does not prove arbitrary shell semantics, a general threading runtime, binutils, or provider admission";
pub(crate) const BASH_FULL_GENERATOR_BLAKE3: &str = "725e682bc5f91728a92f72a81332dda87537ff0ec27b20134b9644b5f2cd43a7";
pub(crate) const BASH_FULL_GENERATED_TREE_BLAKE3: &str =
    "1797cb39c0507d6fbb40afbbd58299ffd9a0eb7d0e983bbe64742499a3fdc9c9";
pub(crate) const BASH_FULL_CONFIGURED_SOURCE_ARTIFACT_ID: &str = "bash-full-configured-source";
pub(crate) const BASH_FULL_GENERATOR_ARTIFACT_ID: &str = "bash-full-mkbuiltins";
pub(crate) const BASH_FULL_GENERATED_TREE_ARTIFACT_ID: &str = "bash-full-generated-builtins";
pub(crate) const BASH_FULL_CONFIGURED_SOURCE_BLAKE3: &str =
    "dc3fd01b5f34af5e2f3031c10cda243f43b3c9ce45a0fd2181693727fb629a2e";
pub(crate) const BASH_FULL_BINARY_BLAKE3: &str = "3f166d5bb28aee29982ec38ba07f8b1dca3c488390737a0e78d35950876e4d4e";
const BASH_FULL_VERSION_BLAKE3: &str = "1cb581932f76e9f3870af0a75060141d692acd438ef5663ebb182798642fa20e";
const BASH_FULL_FUNCTIONAL_BLAKE3: &str = "6687d0952325979e333ab55180b9aea8835ff490e46b7652ea5ec204c94b9c2a";
const BASH_FULL_NEGATIVE_BLAKE3: &str = "f1184a47cefae3e975232299de76d9a96ea5a9354c87f19f77413895583d8da3";
const BASH_FULL_EXTERNAL_CHILD_BLAKE3: &str = "deb003aad7485e87e9a7ec33306fe4e89ca6774d39eeea26371b78d21cd8c292";
const BASH_FULL_MAKE_SHELL_BLAKE3: &str = "97ce45a8f015be46bbbd66c2abd778940a5aed257069c4e9dfd432b751c968d8";
const BASH_FULL_STATIC_BUILTIN_SOURCES: &[&str] = &[
    "builtins/common",
    "builtins/evalstring",
    "builtins/evalfile",
    "builtins/bashgetopt",
    "builtins/getopt",
];

const BASH_FULL_BUILTIN_DEFINITIONS: &[&str] = &[
    "alias",
    "bind",
    "break",
    "builtin",
    "cd",
    "colon",
    "command",
    "declare",
    "echo",
    "enable",
    "eval",
    "getopts",
    "exec",
    "exit",
    "fc",
    "fg_bg",
    "hash",
    "help",
    "history",
    "jobs",
    "kill",
    "let",
    "read",
    "return",
    "set",
    "setattr",
    "shift",
    "source",
    "suspend",
    "test",
    "times",
    "trap",
    "type",
    "ulimit",
    "umask",
    "wait",
    "reserved",
    "pushd",
    "shopt",
    "printf",
    "common",
    "evalstring",
    "evalfile",
];

#[derive(Debug, Clone)]
pub(crate) struct BashFullGeneratorRequest<'a> {
    pub configured_source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub scratch_dir: &'a Path,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct BashFullGeneratorReport {
    pub format: &'static str,
    pub generator_path: PathBuf,
    pub generator_digest_blake3: String,
    pub generator_build_command_count: u32,
    pub generator_command_count: u32,
    pub generated_file_count: u32,
    pub generated_tree_digest_blake3: String,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct BashFullInventoryRequest<'a> {
    pub configured_source_root: &'a Path,
    pub tcc_musl_v2_root: &'a Path,
    pub musl_native_root: &'a Path,
    pub make: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BashFullExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const BASH_FULL_EXPECTED_OUTPUTS: [BashFullExpectedOutput; BASH_FULL_OUTPUT_COUNT] = [
    BashFullExpectedOutput {
        artifact_id: "bash-2.05b-full",
        digest_blake3: BASH_FULL_BINARY_BLAKE3,
    },
    BashFullExpectedOutput {
        artifact_id: "bash-full-version-observation",
        digest_blake3: BASH_FULL_VERSION_BLAKE3,
    },
    BashFullExpectedOutput {
        artifact_id: "bash-full-functional-observation",
        digest_blake3: BASH_FULL_FUNCTIONAL_BLAKE3,
    },
    BashFullExpectedOutput {
        artifact_id: "bash-full-negative-observation",
        digest_blake3: BASH_FULL_NEGATIVE_BLAKE3,
    },
    BashFullExpectedOutput {
        artifact_id: "bash-full-external-child-observation",
        digest_blake3: BASH_FULL_EXTERNAL_CHILD_BLAKE3,
    },
    BashFullExpectedOutput {
        artifact_id: "bash-full-make-shell-observation",
        digest_blake3: BASH_FULL_MAKE_SHELL_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct BashFullOutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct BashFullInventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub generator_path: PathBuf,
    pub generator_digest_blake3: String,
    pub generated_tree_digest_blake3: String,
    pub source_compile_count: u32,
    pub build_command_count: u32,
    pub generator_command_count: u32,
    pub smoke_command_count: u32,
    pub negative_exit_code: i32,
    pub outputs: Vec<BashFullOutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum StagexBashFullError {
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for StagexBashFullError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Materialization(message) => write!(formatter, "full Bash materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "full Bash runtime failed: {error}"),
        }
    }
}

impl std::error::Error for StagexBashFullError {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for StagexBashFullError {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

pub(crate) fn derive_bash_full_generator(
    request: BashFullGeneratorRequest<'_>,
) -> Result<BashFullGeneratorReport, StagexBashFullError> {
    validate_generator_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash scratch: {error}")))?;
    let source_root = request.scratch_dir.join("bash-2.05b-full");
    crate::stagex_mes_lib::copy_tree_bounded(request.configured_source_root, &source_root)?;
    normalize_full_config(&source_root)?;
    let generator = build_generator(&request, &source_root)?;
    generate_builtins(&request, &source_root, &generator)?;
    let generated_tree_digest_blake3 = generated_tree_digest(&source_root)?;
    let generator_digest_blake3 = digest_file(&generator, "Bash mkbuiltins")?;
    validate_digest(&generator_digest_blake3, BASH_FULL_GENERATOR_BLAKE3, "Bash mkbuiltins")?;
    validate_digest(&generated_tree_digest_blake3, BASH_FULL_GENERATED_TREE_BLAKE3, "generated Bash builtin tree")?;
    let generated_file_count = u32::try_from(BASH_FULL_GENERATED_FILE_COUNT)
        .map_err(|_| StagexBashFullError::Materialization("generated file count does not fit u32".to_string()))?;
    let report = BashFullGeneratorReport {
        format: "mantle-stagex-bash-2.05b-full-generator-v1",
        generator_path: generator,
        generator_digest_blake3,
        generator_build_command_count: BASH_FULL_GENERATOR_BUILD_COMMAND_COUNT,
        generator_command_count: BASH_FULL_GENERATOR_COMMAND_COUNT,
        generated_file_count,
        generated_tree_digest_blake3,
        fallback_events: Vec::new(),
        non_claim: BASH_FULL_GENERATOR_NON_CLAIM,
    };
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| {
        StagexBashFullError::Materialization(format!("serializing full Bash generator report: {error}"))
    })?;
    crate::stagex_mes_lib::write_create_new(&request.scratch_dir.join("bash-full-generator.json"), &bytes)?;
    assert_eq!(report.generated_file_count, generated_file_count);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

pub(crate) fn derive_bash_full_inventory(
    request: BashFullInventoryRequest<'_>,
) -> Result<BashFullInventoryReport, StagexBashFullError> {
    let generator_request = BashFullGeneratorRequest {
        configured_source_root: request.configured_source_root,
        tcc_musl_v2_root: request.tcc_musl_v2_root,
        musl_native_root: request.musl_native_root,
        scratch_dir: request.scratch_dir,
    };
    validate_generator_inputs(&generator_request)?;
    validate_file_digest(request.make, crate::stagex_make::MAKE_FINAL_BLAKE3, "GNU Make 3.82")?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash scratch: {error}")))?;
    let source_root = request.scratch_dir.join("bash-2.05b-full");
    crate::stagex_mes_lib::copy_tree_bounded(request.configured_source_root, &source_root)?;
    normalize_full_config(&source_root)?;
    let generator = build_generator(&generator_request, &source_root)?;
    generate_builtins(&generator_request, &source_root, &generator)?;
    let generated_tree_digest_blake3 = generated_tree_digest(&source_root)?;
    let generator_digest_blake3 = digest_file(&generator, "Bash mkbuiltins")?;
    validate_digest(&generator_digest_blake3, BASH_FULL_GENERATOR_BLAKE3, "Bash mkbuiltins")?;
    validate_digest(&generated_tree_digest_blake3, BASH_FULL_GENERATED_TREE_BLAKE3, "generated Bash builtin tree")?;
    normalize_fmtumax_source(&source_root)?;
    write_runtime_compat_source(&source_root)?;
    let configured_source_digest_blake3 = full_configured_source_digest(&source_root)?;
    validate_digest(
        &configured_source_digest_blake3,
        BASH_FULL_CONFIGURED_SOURCE_BLAKE3,
        "full Bash configured source",
    )?;
    let bash = build_full_shell(&request, &source_root)?;
    let observations = run_shell_smokes(&request, &bash)?;
    let outputs = collect_full_outputs(&bash, &observations)?;
    validate_full_outputs(&outputs)?;
    let source_compile_count = u32::try_from(BASH_FULL_SOURCE_COMPILE_COUNT).map_err(|_| {
        StagexBashFullError::Materialization("full Bash source compile count does not fit u32".to_string())
    })?;
    let report = BashFullInventoryReport {
        format: "mantle-stagex-bash-2.05b-full-inventory-v1",
        configured_source_digest_blake3,
        generator_path: generator,
        generator_digest_blake3,
        generated_tree_digest_blake3,
        source_compile_count,
        build_command_count: BASH_FULL_BUILD_COMMAND_COUNT,
        generator_command_count: BASH_FULL_GENERATOR_COMMAND_COUNT,
        smoke_command_count: BASH_FULL_SMOKE_COMMAND_COUNT,
        negative_exit_code: observations.negative_exit_code,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: BASH_FULL_NON_CLAIM,
    };
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| StagexBashFullError::Materialization(format!("serializing full Bash inventory: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&request.scratch_dir.join("bash-full-inventory.json"), &bytes)?;
    assert_eq!(report.outputs.len(), BASH_FULL_OUTPUT_COUNT);
    assert_ne!(report.negative_exit_code, 0);
    Ok(report)
}

fn normalize_full_config(source_root: &Path) -> Result<(), StagexBashFullError> {
    const REQUIRED_PREDECESSOR_MARKER: &str = "#define HAVE_TIMES 1\n";
    const REQUIRED_FULL_MARKER: &str = "#define HAVE_SYS_TIMES_H 1\n";
    const DISABLE_NETWORK_MARKER: &str = "#undef HAVE_SYS_SOCKET_H\n";
    let config_path = source_root.join("config.h");
    let bytes = crate::stagex_mes_lib::read_bounded_file(
        &config_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        "Bash full config header",
    )?;
    let mut config = String::from_utf8(bytes).map_err(|error| {
        StagexBashFullError::Materialization(format!("reading UTF-8 Bash full config header: {error}"))
    })?;
    if config.matches(REQUIRED_PREDECESSOR_MARKER).count() != 1 {
        return Err(StagexBashFullError::Materialization("unexpected Bash full config predecessor marker".to_string()));
    }
    if config.contains(REQUIRED_FULL_MARKER) {
        return Err(StagexBashFullError::Materialization(
            "Bash full config already contains the times-header marker".to_string(),
        ));
    }
    if config.contains(DISABLE_NETWORK_MARKER) {
        return Err(StagexBashFullError::Materialization(
            "Bash full config already contains the network marker".to_string(),
        ));
    }
    config.push_str(REQUIRED_FULL_MARKER);
    config.push_str(DISABLE_NETWORK_MARKER);
    fs::write(&config_path, config.as_bytes())
        .map_err(|error| StagexBashFullError::Materialization(format!("writing Bash full config header: {error}")))?;
    assert!(config.contains(REQUIRED_PREDECESSOR_MARKER));
    assert!(config.contains(REQUIRED_FULL_MARKER));
    assert!(config.ends_with(DISABLE_NETWORK_MARKER));
    Ok(())
}

fn normalize_fmtumax_source(source_root: &Path) -> Result<(), StagexBashFullError> {
    const INCLUDE_MARKER: &str = "#include \"fmtulong.c\"\n";
    let wrapper_path = source_root.join("lib/sh/fmtumax.c");
    let implementation_path = source_root.join("lib/sh/fmtulong.c");
    let wrapper_bytes =
        crate::stagex_mes_lib::read_bounded_file(&wrapper_path, BASH_FULL_ARTIFACT_BYTES_MAX, "Bash fmtumax wrapper")?;
    let implementation_bytes = crate::stagex_mes_lib::read_bounded_file(
        &implementation_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        "Bash fmtulong implementation",
    )?;
    let wrapper = String::from_utf8(wrapper_bytes).map_err(|error| {
        StagexBashFullError::Materialization(format!("reading UTF-8 Bash fmtumax wrapper: {error}"))
    })?;
    let implementation = String::from_utf8(implementation_bytes).map_err(|error| {
        StagexBashFullError::Materialization(format!("reading UTF-8 Bash fmtulong implementation: {error}"))
    })?;
    if wrapper.matches(INCLUDE_MARKER).count() != 1 {
        return Err(StagexBashFullError::Materialization("unexpected Bash fmtumax include count".to_string()));
    }
    if !wrapper.ends_with(INCLUDE_MARKER) {
        return Err(StagexBashFullError::Materialization("unexpected Bash fmtumax include position".to_string()));
    }
    let inlined = wrapper.replacen(INCLUDE_MARKER, &implementation, 1);
    const TYPE_MARKER: &str = "#define LONG\tintmax_t";
    if inlined.matches(TYPE_MARKER).count() != 1 {
        return Err(StagexBashFullError::Materialization("unexpected Bash fmtumax integer-type marker".to_string()));
    }
    let normalized = inlined.replacen(TYPE_MARKER, "#include <stdint.h>\n#define LONG\tintmax_t", 1);
    fs::write(&wrapper_path, normalized.as_bytes()).map_err(|error| {
        StagexBashFullError::Materialization(format!("writing normalized Bash fmtumax source: {error}"))
    })?;
    assert!(!normalized.contains(INCLUDE_MARKER));
    assert!(normalized.contains("#define fmtulong\tfmtumax"));
    Ok(())
}

fn write_runtime_compat_source(source_root: &Path) -> Result<(), StagexBashFullError> {
    const SOURCE: &str = r#"#include <errno.h>
#include <limits.h>
#include <semaphore.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <unistd.h>

int socket(int domain, int type, int protocol) {
  return (int)syscall(SYS_socket, domain, type, protocol);
}

int connect(int fd, const struct sockaddr *address, socklen_t length) {
  return (int)syscall(SYS_connect, fd, address, length);
}

ssize_t sendmsg(int fd, const struct msghdr *message, int flags) {
  return (ssize_t)syscall(SYS_sendmsg, fd, message, flags);
}

int sem_init(sem_t *sem, int pshared, unsigned value) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (pshared != 0) { errno = EINVAL; return -1; }
  if (value > INT_MAX) { errno = EINVAL; return -1; }
  sem->__val[0] = (int)value;
  return 0;
}

int sem_destroy(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  return 0;
}

int sem_post(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (sem->__val[0] == INT_MAX) { errno = EOVERFLOW; return -1; }
  sem->__val[0] += 1;
  return 0;
}

int sem_wait(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (sem->__val[0] <= 0) { errno = EAGAIN; return -1; }
  sem->__val[0] -= 1;
  return 0;
}
"#;
    let path = source_root.join("stagex-full-runtime-compat.c");
    crate::stagex_mes_lib::write_create_new(&path, SOURCE.as_bytes())?;
    assert!(SOURCE.contains("int sem_init"));
    assert!(SOURCE.contains("int sem_wait"));
    Ok(())
}

fn full_configured_source_digest(source_root: &Path) -> Result<String, StagexBashFullError> {
    const CONFIGURED_SOURCE_MEMBER_COUNT: usize = 3;
    const PATHS: [&str; CONFIGURED_SOURCE_MEMBER_COUNT] =
        ["config.h", "lib/sh/fmtumax.c", "stagex-full-runtime-compat.c"];
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-bash-full-configured-source-v1\0");
    hasher.update(crate::stagex_bash::BASH_CONFIGURED_SOURCE_BLAKE3.as_bytes());
    hasher.update(b"\0");
    for relative in PATHS {
        let bytes = crate::stagex_mes_lib::read_bounded_file(
            &source_root.join(relative),
            BASH_FULL_ARTIFACT_BYTES_MAX,
            "full Bash configured source member",
        )?;
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(PATHS.len(), CONFIGURED_SOURCE_MEMBER_COUNT);
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok(digest)
}

fn build_full_shell(
    request: &BashFullInventoryRequest<'_>,
    source_root: &Path,
) -> Result<PathBuf, StagexBashFullError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object_root = source_root.join("stagex-full-objects");
    fs::create_dir(&object_root)
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash object root: {error}")))?;
    let sources = full_shell_sources();
    if sources.len() != BASH_FULL_SOURCE_COMPILE_COUNT {
        return Err(StagexBashFullError::Materialization(format!(
            "full Bash source count mismatch: expected {BASH_FULL_SOURCE_COMPILE_COUNT}, observed {}",
            sources.len()
        )));
    }
    let mut objects = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        objects.push(compile_full_shell_source(request, source_root, &compiler, index, source)?);
    }
    let output_bin = request.scratch_dir.join("output/bin");
    fs::create_dir_all(&output_bin)
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash output root: {error}")))?;
    let bash = output_bin.join("bash-full");
    link_full_shell(request, source_root, &compiler, &objects, &bash)?;
    std::os::unix::fs::symlink("bash-full", output_bin.join("sh"))
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash sh alias: {error}")))?;
    assert_eq!(objects.len(), sources.len());
    assert!(output_bin.join("sh").is_symlink());
    Ok(bash)
}

fn full_shell_sources() -> Vec<String> {
    let mut sources = Vec::with_capacity(BASH_FULL_SOURCE_COMPILE_COUNT);
    sources.extend(crate::stagex_bash::CORE_SOURCES.iter().map(|source| format!("{source}.c")));
    sources.push("lib/readline/readline_stub.c".to_string());
    sources.push("stagex-full-runtime-compat.c".to_string());
    sources.push("lib/tilde/tilde.c".to_string());
    sources.extend(crate::stagex_bash::GLOB_SOURCES.iter().map(|source| format!("{source}.c")));
    sources.extend(
        crate::stagex_bash::SH_LIBRARY_SOURCES
            .iter()
            .filter(|source| !matches!(**source, "lib/sh/strtoimax" | "lib/sh/strtoumax"))
            .map(|source| format!("{source}.c")),
    );
    sources.extend(BASH_FULL_STATIC_BUILTIN_SOURCES.iter().map(|source| format!("{source}.c")));
    sources.extend(
        BASH_FULL_BUILTIN_DEFINITIONS
            .iter()
            .filter(|definition| !is_non_generated_definition(definition))
            .map(|definition| format!("builtins/{definition}.c")),
    );
    sources.push("builtins/builtins.c".to_string());
    assert_eq!(sources.len(), BASH_FULL_SOURCE_COMPILE_COUNT);
    assert!(sources.iter().all(|source| source.ends_with(".c")));
    sources
}

fn compile_full_shell_source(
    request: &BashFullInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    index: usize,
    source: &str,
) -> Result<String, StagexBashFullError> {
    const OBJECT_INDEX_WIDTH: usize = 3;
    let object = format!("stagex-full-objects/{index:0width$}.o", width = OBJECT_INDEX_WIDTH);
    let args = vec![
        "-c".to_string(),
        "-I.".to_string(),
        "-Iinclude".to_string(),
        "-Ilib".to_string(),
        "-Ibuiltins".to_string(),
        format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?),
        "-DHAVE_CONFIG_H".to_string(),
        source.to_string(),
        "-o".to_string(),
        object.clone(),
    ];
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("bash-full-compile-{index:03}.stderr.txt")),
    )?;
    let object_path = source_root.join(&object);
    fs::set_permissions(&object_path, fs::Permissions::from_mode(BASH_FULL_REGULAR_MODE))
        .map_err(|error| StagexBashFullError::Materialization(format!("setting full Bash object mode: {error}")))?;
    validate_nonempty_file(&object_path, "full Bash object")?;
    assert!(source_root.join(source).is_file());
    assert!(object_path.is_file());
    Ok(object)
}

fn link_full_shell(
    request: &BashFullInventoryRequest<'_>,
    source_root: &Path,
    compiler: &Path,
    objects: &[String],
    bash: &Path,
) -> Result<(), StagexBashFullError> {
    let mut args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        utf8_absolute(bash, "full Bash output")?.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
    ];
    args.extend(objects.iter().cloned());
    args.push(utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string());
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        source_root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("bash-full-link.stderr.txt"),
    )?;
    fs::set_permissions(bash, fs::Permissions::from_mode(BASH_FULL_EXECUTABLE_MODE))
        .map_err(|error| StagexBashFullError::Materialization(format!("setting full Bash executable mode: {error}")))?;
    validate_nonempty_file(bash, "full Bash executable")?;
    assert_eq!(objects.len(), BASH_FULL_SOURCE_COMPILE_COUNT);
    assert!(bash.is_file());
    Ok(())
}

struct BashFullObservations {
    version: PathBuf,
    functional: PathBuf,
    negative: PathBuf,
    external_child: PathBuf,
    make_shell: PathBuf,
    negative_exit_code: i32,
}

fn run_shell_smokes(
    request: &BashFullInventoryRequest<'_>,
    bash: &Path,
) -> Result<BashFullObservations, StagexBashFullError> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| StagexBashFullError::Materialization(format!("creating full Bash smoke root: {error}")))?;
    let version_args = vec!["--version".to_string()];
    let version = run_shell_success(bash, &smoke, "version", &version_args, b"")?;
    require_contains(&version, b"GNU bash", "full Bash version observation")?;
    let script = "set -eu\nvalue=mantle\nfor item in one two; do case \"$item\" in one) printf '%s\\n' first ;; two) printf '%s\\n' second ;; esac; done\nprintf '%s\\n' \"$value\"\nread line\ntest \"$line\" = input\neval 'printf \"%s\\n\" eval-ok'\n";
    let functional_args = vec!["-c".to_string(), script.to_string()];
    let functional = run_shell_success(bash, &smoke, "functional", &functional_args, b"input\n")?;
    require_exact_bytes(&functional, b"first\nsecond\nmantle\neval-ok\n", "full Bash functional observation")?;
    let (negative, negative_exit_code) = run_shell_negative(bash, &smoke)?;
    let make = utf8_absolute(request.make, "GNU Make external child")?;
    let external_args = vec![
        "-c".to_string(),
        "\"$1\" --version".to_string(),
        "bash-full-external-child".to_string(),
        make.to_string(),
    ];
    let external_child = run_shell_success(bash, &smoke, "external-child", &external_args, b"")?;
    require_contains(&external_child, b"GNU Make 3.82", "full Bash external-child observation")?;
    let make_shell = run_make_shell_smoke(request.make, bash, &smoke)?;
    assert_ne!(negative_exit_code, 0);
    assert!(make_shell.is_file());
    Ok(BashFullObservations {
        version,
        functional,
        negative,
        external_child,
        make_shell,
        negative_exit_code,
    })
}

fn run_shell_success(
    bash: &Path,
    smoke: &Path,
    label: &str,
    arguments: &[String],
    stdin_bytes: &[u8],
) -> Result<PathBuf, StagexBashFullError> {
    let stdin_path = smoke.join(format!("{label}.in"));
    let stdout_path = smoke.join(format!("{label}.out"));
    let stderr_path = smoke.join(format!("{label}.stderr.txt"));
    crate::stagex_mes_lib::write_create_new(&stdin_path, stdin_bytes)?;
    crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout(
        bash,
        arguments,
        smoke,
        &BTreeMap::<String, String>::new(),
        &stdin_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        &stdout_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        &stderr_path,
    )?;
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    Ok(stdout_path)
}

fn run_make_shell_smoke(make: &Path, bash: &Path, smoke: &Path) -> Result<PathBuf, StagexBashFullError> {
    let makefile_path = smoke.join("Makefile.full-shell");
    let output_path = smoke.join("make-shell.out");
    let makefile = format!(
        "SHELL := {}\n.SHELLFLAGS := -c\nall:\n\t@printf '%s\\n' make-shell-ok > {}\n",
        utf8_absolute(bash, "full Bash Make shell")?,
        utf8_absolute(&output_path, "full Bash Make output")?,
    );
    crate::stagex_mes_lib::write_create_new(&makefile_path, makefile.as_bytes())?;
    crate::stagex_mes_lib::run_bounded_process(
        make,
        &["-f", "Makefile.full-shell", "all"],
        smoke,
        &BTreeMap::<String, String>::new(),
        &smoke.join("make-shell.stderr.txt"),
    )?;
    require_exact_bytes(&output_path, b"make-shell-ok\n", "full Bash GNU Make shell observation")?;
    assert!(makefile.contains(".SHELLFLAGS := -c"));
    assert!(output_path.is_file());
    Ok(output_path)
}

fn run_shell_negative(bash: &Path, smoke: &Path) -> Result<(PathBuf, i32), StagexBashFullError> {
    let stdin_path = smoke.join("negative.in");
    let stdout_path = smoke.join("negative.out");
    let stderr_path = smoke.join("negative.stderr.txt");
    crate::stagex_mes_lib::write_create_new(&stdin_path, b"")?;
    let args = ["-c".to_string(), "if then".to_string()];
    let exit_code = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        bash,
        &args,
        smoke,
        &BTreeMap::<String, String>::new(),
        &stdin_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        &stdout_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        &stderr_path,
    )?;
    if exit_code == 0 {
        return Err(StagexBashFullError::Materialization("full Bash accepted malformed shell input".to_string()));
    }
    require_exact_bytes(&stdout_path, b"", "full Bash negative stdout")?;
    validate_nonempty_file(&stderr_path, "full Bash negative diagnostic")?;
    let raw = crate::stagex_mes_lib::read_bounded_file(
        &stderr_path,
        BASH_FULL_ARTIFACT_BYTES_MAX,
        "full Bash negative diagnostic",
    )?;
    let raw = String::from_utf8(raw).map_err(|error| {
        StagexBashFullError::Materialization(format!("reading UTF-8 full Bash negative diagnostic: {error}"))
    })?;
    let bash_path = utf8_absolute(bash, "full Bash negative executable")?;
    const EXPECTED_PATH_OCCURRENCES: usize = 2;
    if raw.matches(bash_path).count() != EXPECTED_PATH_OCCURRENCES {
        return Err(StagexBashFullError::Materialization(
            "full Bash negative diagnostic had an unexpected executable-path count".to_string(),
        ));
    }
    let normalized_path = smoke.join("negative.normalized.stderr.txt");
    let normalized = raw.replace(bash_path, "bash-full");
    crate::stagex_mes_lib::write_create_new(&normalized_path, normalized.as_bytes())?;
    assert_ne!(exit_code, 0);
    assert!(!normalized.contains(bash_path));
    Ok((normalized_path, exit_code))
}

fn collect_full_outputs(
    bash: &Path,
    observations: &BashFullObservations,
) -> Result<Vec<BashFullOutputReport>, StagexBashFullError> {
    let specs = [
        ("bash-2.05b-full", bash),
        ("bash-full-version-observation", observations.version.as_path()),
        ("bash-full-functional-observation", observations.functional.as_path()),
        ("bash-full-negative-observation", observations.negative.as_path()),
        ("bash-full-external-child-observation", observations.external_child.as_path()),
        ("bash-full-make-shell-observation", observations.make_shell.as_path()),
    ];
    let outputs = specs
        .iter()
        .map(|(artifact_id, path)| full_output_report(artifact_id, path))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(outputs.len(), BASH_FULL_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(outputs)
}

fn full_output_report(artifact_id: &str, path: &Path) -> Result<BashFullOutputReport, StagexBashFullError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, BASH_FULL_ARTIFACT_BYTES_MAX, artifact_id)?;
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| StagexBashFullError::Materialization(format!("full Bash artifact is too large: {artifact_id}")))?;
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    assert!(bytes_len <= BASH_FULL_ARTIFACT_BYTES_MAX);
    assert_eq!(digest_blake3.len(), blake3::OUT_LEN * 2);
    Ok(BashFullOutputReport {
        artifact_id: artifact_id.to_string(),
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3,
    })
}

fn validate_full_outputs(outputs: &[BashFullOutputReport]) -> Result<(), StagexBashFullError> {
    for expected in BASH_FULL_EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            StagexBashFullError::Materialization(format!("full Bash output is missing: {}", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            return Err(StagexBashFullError::Materialization(format!(
                "full Bash output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, observed.digest_blake3
            )));
        }
    }
    assert_eq!(outputs.len(), BASH_FULL_OUTPUT_COUNT);
    assert_eq!(BASH_FULL_EXPECTED_OUTPUTS.len(), BASH_FULL_OUTPUT_COUNT);
    Ok(())
}

fn require_exact_bytes(path: &Path, expected: &[u8], label: &str) -> Result<(), StagexBashFullError> {
    let metadata = fs::metadata(path)
        .map_err(|error| StagexBashFullError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() > BASH_FULL_ARTIFACT_BYTES_MAX {
        return Err(StagexBashFullError::Materialization(format!("{label} is not a bounded regular file")));
    }
    let observed =
        fs::read(path).map_err(|error| StagexBashFullError::Materialization(format!("reading {label}: {error}")))?;
    if observed != expected {
        return Err(StagexBashFullError::Materialization(format!("{label} bytes differ")));
    }
    assert_eq!(observed, expected);
    assert!(u64::try_from(observed.len()).unwrap_or(u64::MAX) <= BASH_FULL_ARTIFACT_BYTES_MAX);
    Ok(())
}

fn require_contains(path: &Path, needle: &[u8], label: &str) -> Result<(), StagexBashFullError> {
    let observed = crate::stagex_mes_lib::read_bounded_file(path, BASH_FULL_ARTIFACT_BYTES_MAX, label)?;
    if !observed.windows(needle.len()).any(|window| window == needle) {
        return Err(StagexBashFullError::Materialization(format!("{label} lacks required bytes")));
    }
    assert!(!needle.is_empty());
    assert!(!observed.is_empty());
    Ok(())
}

fn validate_generator_inputs(request: &BashFullGeneratorRequest<'_>) -> Result<(), StagexBashFullError> {
    if request.scratch_dir.exists() {
        return Err(StagexBashFullError::Materialization(format!(
            "full Bash scratch already exists: {}",
            request.scratch_dir.display()
        )));
    }
    if !request.configured_source_root.is_absolute() || !request.configured_source_root.is_dir() {
        return Err(StagexBashFullError::Materialization(
            "configured Bash source root is not an absolute directory".to_string(),
        ));
    }
    for required in ["config.h", "shell.c", "builtins/mkbuiltins.c", "builtins/alias.def"] {
        if !request.configured_source_root.join(required).is_file() {
            return Err(StagexBashFullError::Materialization(format!("configured Bash source is missing {required}")));
        }
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
    assert_eq!(BASH_FULL_BUILTIN_DEFINITIONS.len(), BASH_FULL_BUILTIN_DEFINITION_COUNT);
    assert!(!request.scratch_dir.exists());
    Ok(())
}

fn build_generator(request: &BashFullGeneratorRequest<'_>, source_root: &Path) -> Result<PathBuf, StagexBashFullError> {
    let compiler = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object = "builtins/stagex-full-mkbuiltins.o";
    let generator = source_root.join("builtins/stagex-full-mkbuiltins");
    let compile_args = vec![
        "-c".to_string(),
        "-I.".to_string(),
        "-Iinclude".to_string(),
        "-Ilib".to_string(),
        "-Ibuiltins".to_string(),
        format!("-I{}", utf8_absolute(&request.musl_native_root.join("include"), "native musl include")?),
        "-DHAVE_CONFIG_H".to_string(),
        "builtins/mkbuiltins.c".to_string(),
        "-o".to_string(),
        object.to_string(),
    ];
    run_compiler(request, source_root, &compiler, &compile_args, "bash-full-mkbuiltins-compile")?;
    let object_path = source_root.join(object);
    fs::set_permissions(&object_path, fs::Permissions::from_mode(BASH_FULL_REGULAR_MODE))
        .map_err(|error| StagexBashFullError::Materialization(format!("setting mkbuiltins object mode: {error}")))?;
    let link_args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        "builtins/stagex-full-mkbuiltins".to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/crt1.o"), "native musl crt1")?.to_string(),
        object.to_string(),
        utf8_absolute(&request.musl_native_root.join("lib/libc.a"), "native musl libc")?.to_string(),
    ];
    run_compiler(request, source_root, &compiler, &link_args, "bash-full-mkbuiltins-link")?;
    fs::set_permissions(&generator, fs::Permissions::from_mode(BASH_FULL_EXECUTABLE_MODE)).map_err(|error| {
        StagexBashFullError::Materialization(format!("setting mkbuiltins executable mode: {error}"))
    })?;
    validate_nonempty_file(&generator, "Bash mkbuiltins")?;
    assert!(object_path.is_file());
    assert!(generator.is_file());
    Ok(generator)
}

fn generate_builtins(
    request: &BashFullGeneratorRequest<'_>,
    source_root: &Path,
    generator: &Path,
) -> Result<(), StagexBashFullError> {
    let builtins_root = source_root.join("builtins");
    let pipesize_header = builtins_root.join("pipesize.h");
    let pipesize = format!(
        "/* generated from the authenticated Bash psize.sh fallback */\n#define PIPESIZE {BASH_FULL_PIPE_SIZE_BYTES}\n"
    );
    fs::write(&pipesize_header, pipesize.as_bytes())
        .map_err(|error| StagexBashFullError::Materialization(format!("writing Bash pipesize header: {error}")))?;
    assert!(pipesize.contains("#define PIPESIZE"));
    assert!(pipesize.ends_with('\n'));
    remove_if_file(&builtins_root.join("builtext.h"))?;
    remove_if_file(&builtins_root.join("builtins.c"))?;
    for definition in BASH_FULL_BUILTIN_DEFINITIONS {
        if is_non_generated_definition(definition) {
            continue;
        }
        let generated = builtins_root.join(format!("{definition}.c"));
        remove_if_file(&generated)?;
        let args = vec!["-D".to_string(), ".".to_string(), format!("{definition}.def")];
        run_generator(request, generator, &builtins_root, &args, definition)?;
        validate_nonempty_file(&generated, "generated Bash builtin source")?;
    }
    let definitions = BASH_FULL_BUILTIN_DEFINITIONS
        .iter()
        .filter(|definition| !matches!(**definition, "common" | "evalstring" | "evalfile"))
        .map(|definition| format!("{definition}.def"))
        .collect::<Vec<_>>();
    let mut args = vec![
        "-externfile".to_string(),
        "builtext.h".to_string(),
        "-structfile".to_string(),
        "builtins.c".to_string(),
        "-noproduction".to_string(),
        "-D".to_string(),
        ".".to_string(),
    ];
    args.extend(definitions);
    run_generator(request, generator, &builtins_root, &args, "table")?;
    validate_nonempty_file(&builtins_root.join("builtext.h"), "generated Bash builtin declarations")?;
    validate_nonempty_file(&builtins_root.join("builtins.c"), "generated Bash builtin table")?;
    assert!(builtins_root.join("eval.c").is_file());
    assert!(!builtins_root.join("reserved.c").exists());
    assert!(builtins_root.join("builtins.c").is_file());
    Ok(())
}

fn is_non_generated_definition(definition: &str) -> bool {
    let non_generated = matches!(definition, "common" | "evalstring" | "evalfile" | "reserved");
    assert!(!definition.is_empty());
    assert!(definition.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_'));
    non_generated
}

fn run_generator(
    request: &BashFullGeneratorRequest<'_>,
    generator: &Path,
    cwd: &Path,
    args: &[String],
    label: &str,
) -> Result<(), StagexBashFullError> {
    crate::stagex_mes_lib::run_bounded_process(
        generator,
        args,
        cwd,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("bash-full-generate-{label}.stderr.txt")),
    )?;
    assert!(generator.is_absolute());
    assert!(cwd.is_absolute());
    Ok(())
}

fn run_compiler(
    request: &BashFullGeneratorRequest<'_>,
    cwd: &Path,
    compiler: &Path,
    args: &[String],
    label: &str,
) -> Result<(), StagexBashFullError> {
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        args,
        cwd,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("{label}.stderr.txt")),
    )?;
    assert!(compiler.is_absolute());
    assert!(cwd.is_absolute());
    Ok(())
}

fn generated_tree_digest(source_root: &Path) -> Result<String, StagexBashFullError> {
    let builtins_root = source_root.join("builtins");
    let mut paths = BASH_FULL_BUILTIN_DEFINITIONS
        .iter()
        .filter(|definition| !is_non_generated_definition(definition))
        .map(|definition| builtins_root.join(format!("{definition}.c")))
        .collect::<Vec<_>>();
    paths.push(builtins_root.join("builtext.h"));
    paths.push(builtins_root.join("builtins.c"));
    paths.push(builtins_root.join("pipesize.h"));
    paths.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-stagex-bash-full-generated-builtins-v1\0");
    for path in &paths {
        let relative = path
            .strip_prefix(source_root)
            .map_err(|_| StagexBashFullError::Materialization("generated Bash path escaped source root".to_string()))?;
        let bytes =
            crate::stagex_mes_lib::read_bounded_file(path, BASH_FULL_ARTIFACT_BYTES_MAX, "generated Bash source")?;
        hasher.update(relative.as_os_str().as_encoded_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(paths.len(), BASH_FULL_GENERATED_FILE_COUNT);
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok(digest)
}

fn remove_if_file(path: &Path) -> Result<(), StagexBashFullError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(StagexBashFullError::Materialization(format!(
            "removing stale full Bash generated file {}: {error}",
            path.display()
        ))),
    }
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), StagexBashFullError> {
    let metadata = path
        .metadata()
        .map_err(|error| StagexBashFullError::Materialization(format!("reading {label} metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > BASH_FULL_ARTIFACT_BYTES_MAX {
        return Err(StagexBashFullError::Materialization(format!("invalid {label}: {}", path.display())));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() <= BASH_FULL_ARTIFACT_BYTES_MAX);
    Ok(())
}

fn digest_file(path: &Path, label: &str) -> Result<String, StagexBashFullError> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, BASH_FULL_ARTIFACT_BYTES_MAX, label)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(digest)
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexBashFullError> {
    let observed = digest_file(path, label)?;
    validate_digest(&observed, expected, label)
}

fn validate_digest(observed: &str, expected: &str, label: &str) -> Result<(), StagexBashFullError> {
    if observed != expected {
        return Err(StagexBashFullError::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert_ne!(observed, "0".repeat(blake3::OUT_LEN * 2));
    Ok(())
}

fn native_musl_digest(artifact_id: &str) -> Result<&'static str, StagexBashFullError> {
    let digest = crate::stagex_musl_native::EXPECTED_OUTPUTS
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| output.digest_blake3)
        .ok_or_else(|| {
            StagexBashFullError::Materialization(format!("native musl inventory is missing {artifact_id}"))
        })?;
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!digest.is_empty());
    Ok(digest)
}

fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexBashFullError> {
    if !path.is_absolute() {
        return Err(StagexBashFullError::Materialization(format!("{label} path is not absolute: {}", path.display())));
    }
    let value = path
        .to_str()
        .ok_or_else(|| StagexBashFullError::Materialization(format!("{label} path is not UTF-8")))?;
    assert!(path.is_absolute());
    assert!(!value.is_empty());
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_definition_inventory_is_closed() {
        let unique = BASH_FULL_BUILTIN_DEFINITIONS.iter().copied().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(BASH_FULL_BUILTIN_DEFINITIONS.len(), BASH_FULL_BUILTIN_DEFINITION_COUNT);
        assert_eq!(unique.len(), BASH_FULL_BUILTIN_DEFINITION_COUNT);
        assert!(unique.contains("exec"));
        assert!(unique.contains("read"));
    }

    #[test]
    fn full_shell_source_inventory_is_closed() {
        let sources = full_shell_sources();
        let unique = sources.iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(sources.len(), BASH_FULL_SOURCE_COMPILE_COUNT);
        assert_eq!(unique.len(), BASH_FULL_SOURCE_COMPILE_COUNT);
        assert!(sources.contains(&"builtins/exec.c".to_string()));
        assert!(sources.contains(&"builtins/read.c".to_string()));
    }

    #[test]
    fn output_validation_accepts_exact_identities_and_rejects_substitution() {
        let outputs = BASH_FULL_EXPECTED_OUTPUTS
            .iter()
            .map(|expected| BashFullOutputReport {
                artifact_id: expected.artifact_id.to_string(),
                path: PathBuf::from(format!("/stagex/{}", expected.artifact_id)),
                bytes_len: 1,
                digest_blake3: expected.digest_blake3.to_string(),
            })
            .collect::<Vec<_>>();
        validate_full_outputs(&outputs).unwrap();
        let mut substituted = outputs;
        substituted[0].digest_blake3 = "a".repeat(blake3::OUT_LEN * 2);
        assert!(validate_full_outputs(&substituted).unwrap_err().to_string().contains("BLAKE3 mismatch"));
        assert_ne!(substituted[0].digest_blake3, BASH_FULL_BINARY_BLAKE3);
    }

    #[test]
    fn generator_rejects_existing_scratch_before_input_use() {
        let temp = tempfile::tempdir().unwrap();
        let error = derive_bash_full_generator(BashFullGeneratorRequest {
            configured_source_root: Path::new("relative-source"),
            tcc_musl_v2_root: Path::new("relative-tcc"),
            musl_native_root: Path::new("relative-musl"),
            scratch_dir: temp.path(),
        })
        .unwrap_err();
        assert!(error.to_string().contains("scratch already exists"));
        assert!(temp.path().is_dir());
    }

    #[test]
    #[ignore = "requires configured Bash source, TinyCC musl-v2, native musl, GNU Make, and create-new scratch"]
    fn derives_retained_full_shell_inventory() {
        let source = std::env::var("MANTLE_STAGE_X_BASH_CONFIGURED_SOURCE").unwrap();
        let tcc = std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap();
        let musl = std::env::var("MANTLE_STAGE_X_MUSL_NATIVE_ROOT").unwrap();
        let make = std::env::var("MANTLE_STAGE_X_MAKE").unwrap();
        let scratch = std::env::var("MANTLE_STAGE_X_BASH_FULL_SCRATCH").unwrap();
        let report = derive_bash_full_inventory(BashFullInventoryRequest {
            configured_source_root: Path::new(&source),
            tcc_musl_v2_root: Path::new(&tcc),
            musl_native_root: Path::new(&musl),
            make: Path::new(&make),
            scratch_dir: Path::new(&scratch),
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.build_command_count, BASH_FULL_BUILD_COMMAND_COUNT);
        assert_eq!(report.generator_command_count, BASH_FULL_GENERATOR_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }

    #[test]
    #[ignore = "requires configured Bash source, TinyCC musl-v2, native musl, and create-new scratch"]
    fn generates_retained_full_builtin_sources() {
        let source = std::env::var("MANTLE_STAGE_X_BASH_CONFIGURED_SOURCE").unwrap();
        let tcc = std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap();
        let musl = std::env::var("MANTLE_STAGE_X_MUSL_NATIVE_ROOT").unwrap();
        let scratch = std::env::var("MANTLE_STAGE_X_BASH_FULL_SCRATCH").unwrap();
        let report = derive_bash_full_generator(BashFullGeneratorRequest {
            configured_source_root: Path::new(&source),
            tcc_musl_v2_root: Path::new(&tcc),
            musl_native_root: Path::new(&musl),
            scratch_dir: Path::new(&scratch),
        })
        .unwrap();
        assert_eq!(report.generator_build_command_count, BASH_FULL_GENERATOR_BUILD_COMMAND_COUNT);
        assert_eq!(report.generator_command_count, BASH_FULL_GENERATOR_COMMAND_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
