use std::collections::BTreeMap;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde::Serialize;

const CONFIGURE_LINE_COUNT_MAX: usize = 2_048;
const CONFIGURE_VARIABLE_COUNT_MAX: usize = 128;
const CONFIGURE_VALUE_BYTES_MAX: usize = 128 * 1_024;
const CONFIGURE_CONDITION_DEPTH_MAX: usize = 16;
const LIBRARY_SOURCE_COUNT_MAX: usize = 512;
const MES_LIBRARY_COUNT: usize = 8;
const MES_LIBRARY_REPORT_FORMAT: &str = "mantle-stagex-mes-runtime-inventory-v1";
const MES_LOGICAL_PREFIX: &str = "/stagex/mes-runtime";
const MES_LIBRARY_NON_CLAIM: &str = "this inventory binds source-built mes-m2, regenerated NYACC tables, Mes modules, and Mes libc archives only; it does not prove TinyCC or provider admission";
const KIB_BYTES: u64 = 1_024;
const MIB_BYTES: u64 = KIB_BYTES * KIB_BYTES;
const MES_TREE_ENTRY_COUNT_MAX: u32 = 30_000;
const MES_FILE_BYTES_MAX: u64 = 64 * MIB_BYTES;
const MES_RECIPE_BYTES_MAX: u64 = 256 * KIB_BYTES;
const MES_PROCESS_STDERR_BYTES_MAX: u64 = MIB_BYTES;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const MES_PROCESS_TIMEOUT_MINUTES: u64 = 10;
const MES_PROCESS_TIMEOUT_MS: u64 = MES_PROCESS_TIMEOUT_MINUTES * SECONDS_PER_MINUTE * MILLISECONDS_PER_SECOND;
const MES_PROCESS_POLL_MS: u64 = 10;
const MES_ARENA_BYTES: &str = "30000000";
const MES_MAX_ARENA_BYTES: &str = "30000000";
const MES_STACK_BYTES: &str = "15000000";
const NYACC_DEFAULT_ARENA_BYTES: &str = "20000000";
const NYACC_C99_ARENA_BYTES: &str = "32000000";
const NYACC_MAX_ARENA_BYTES: &str = "20000000";
const NYACC_STACK_BYTES: &str = "6000000";
const NYACC_COMMAND_COUNT: usize = 3;
const MES_RUNTIME_ARCHIVE_COUNT: usize = 7;
const MES_RUNTIME_REQUIRED_OUTPUT_COUNT: usize = 13;
const MESCC_ASSERT_SYSTEM_OLD: &str = "(define (assert-system* . args)\n  (let ((status (apply system* args)))\n    (when (not (zero? status))\n      (format (current-error-port) \"mescc: failed: ~a\\n\" (string-join args))\n      (exit (status:exit-val status)))\n    status))";
const MESCC_ASSERT_SYSTEM_NEW: &str = "(define (assert-system* . args)\n  (apply system* args)\n  0)";
const MESCC_ARRAY_TYPE_OLD: &str = "(make-c-array (rank+= type rank) count)";
const MESCC_ARRAY_TYPE_NEW: &str = "(make-c-array (rank+= (ast->type type info) rank) count)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MesRuntimeExpectedOutput {
    pub artifact_id: &'static str,
    pub relative_path: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const MES_RUNTIME_EXPECTED_OUTPUTS: [MesRuntimeExpectedOutput; MES_RUNTIME_REQUIRED_OUTPUT_COUNT] = [
    MesRuntimeExpectedOutput {
        artifact_id: "mes-m2",
        relative_path: "bin/mes-m2",
        digest_blake3: "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mescc-entrypoint",
        relative_path: "bin/mescc.scm",
        digest_blake3: "34ecfa676a692c79fe14fe85882dbd3b325c6f6f732e475ab94b9240f18cf445",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-crt1",
        relative_path: "lib/x86_64-mes/crt1.o",
        digest_blake3: "e49c87f324b58369183ab753fb1f346aeb28fc2974956d2872e046f8ef27416d",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-x86_64-m1",
        relative_path: "lib/x86_64-mes/x86_64.M1",
        digest_blake3: "a1107fe39ca1c6486dc57bccf82ad40569fa13e34695e8f2c119fbb892b02892",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libc-mini",
        relative_path: "lib/x86_64-mes/libc-mini.a",
        digest_blake3: "12f9ee98f249da9a85158df6f242f5382c1e2d9a590dc84d521c70249d01dda1",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libmes",
        relative_path: "lib/x86_64-mes/libmes.a",
        digest_blake3: "2f7c7eb4d002097cf68ad3b84ffecbb107195563fd21f27672b9918c3cfad2bd",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libmescc",
        relative_path: "lib/x86_64-mes/libmescc.a",
        digest_blake3: "6a8d1e0a5f9dea02ae863552744e36f721833e57d054c993466d74c83eed7cdc",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libc",
        relative_path: "lib/x86_64-mes/libc.a",
        digest_blake3: "d70f11fe75e9eaa59b0aa4d0d92d8c663afe33a0e8f60e3864d364b45d0105c4",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libc-tcc",
        relative_path: "lib/x86_64-mes/libc+tcc.a",
        digest_blake3: "348f304db99daa3dc1eec807133cb232d28768b0d20a563bf8f6d68b493440d7",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libtcc1",
        relative_path: "lib/x86_64-mes/libtcc1.a",
        digest_blake3: "c11be20d8d70e8ed3a88f1cea0a54ade5c64f8035c50974859dccb77f1bea055",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-libgetopt",
        relative_path: "lib/x86_64-mes/libgetopt.a",
        digest_blake3: "04806a625d31f6a1ae4c92343e23ac4d355bcaa880f8aaa55b372f4a3785f135",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-elf-header",
        relative_path: "lib/linux/x86_64-mes/elf64-header.hex2",
        digest_blake3: "892b042fe8e7d17b9774e89c83979c35f79f067fa2bdfd2bcc7279ac2858d8d4",
    },
    MesRuntimeExpectedOutput {
        artifact_id: "mes-elf-footer",
        relative_path: "lib/linux/x86_64-mes/elf64-footer-single-main.hex2",
        digest_blake3: "b23189272376d99464531e041d1367b5a93086665513280645c1f41ce7ecda79",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MesLibrarySources {
    pub name: &'static str,
    pub source_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MesRuntimeInventoryRequest<'a> {
    pub mes_source_root: &'a Path,
    pub nyacc_source_root: &'a Path,
    pub stage0_root: &'a Path,
    pub output_root: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesRuntimeOutput {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesRuntimeInventoryReport {
    pub format: &'static str,
    pub configure_lib_digest_blake3: String,
    pub nyacc_command_count: u32,
    pub source_compile_count: u32,
    pub archive_count: u32,
    pub outputs: Vec<MesRuntimeOutput>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConditionFrame {
    parent_active: bool,
    condition_matches: bool,
    in_else: bool,
}

#[derive(Debug)]
pub(crate) enum MesLibraryPlanError {
    Parse(String),
    Limit(String),
    Io {
        action: String,
        source: io::Error,
    },
    ProcessFailure {
        executable: PathBuf,
        exit_code: Option<i32>,
        stderr: String,
    },
    ProcessTimeout {
        executable: PathBuf,
        timeout_ms: u64,
    },
}

impl std::fmt::Display for MesLibraryPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(message) => write!(formatter, "parsing Mes library source plan: {message}"),
            Self::Limit(message) => write!(formatter, "Mes library source plan limit: {message}"),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::ProcessFailure {
                executable,
                exit_code,
                stderr,
            } => {
                write!(formatter, "Mes process {} failed with status {exit_code:?}: {stderr}", executable.display())
            }
            Self::ProcessTimeout { executable, timeout_ms } => {
                write!(formatter, "Mes process {} exceeded {timeout_ms} ms", executable.display())
            }
        }
    }
}

impl std::error::Error for MesLibraryPlanError {}

pub(crate) fn derive_mes_library_sources(configure_lib: &str) -> Result<Vec<MesLibrarySources>, MesLibraryPlanError> {
    let mut variables = BTreeMap::from([
        ("mes_libc".to_string(), "mes".to_string()),
        ("mes_kernel".to_string(), "linux".to_string()),
        ("mes_cpu".to_string(), "x86_64".to_string()),
        ("compiler".to_string(), "mescc".to_string()),
        ("V".to_string(), "1".to_string()),
    ]);
    evaluate_configure_assignments(configure_lib, &mut variables)?;
    let specs = [
        ("libc-mini.a", "libc_mini_SOURCES"),
        ("libmes.a", "libmes_SOURCES"),
        ("libmescc.a", "libmescc_SOURCES"),
        ("libc.a", "libc_SOURCES"),
        ("libc+tcc.a", "libc_tcc_SOURCES"),
        ("libtcc1.a", "libtcc1_SOURCES"),
        ("libgetopt.a", "libgetopt_SOURCES"),
        ("crt1.o", "crt1_SOURCE"),
    ];
    variables.insert("libgetopt_SOURCES".to_string(), "lib/posix/getopt.c".to_string());
    variables.insert("crt1_SOURCE".to_string(), "lib/linux/x86_64-mes-mescc/crt1.c".to_string());
    let mut libraries = Vec::with_capacity(MES_LIBRARY_COUNT);
    for (name, variable) in specs {
        let value = variables
            .get(variable)
            .ok_or_else(|| MesLibraryPlanError::Parse(format!("required variable {variable} is missing")))?;
        let source_paths = bounded_source_paths(variable, value)?;
        libraries.push(MesLibrarySources { name, source_paths });
    }
    assert_eq!(libraries.len(), MES_LIBRARY_COUNT);
    assert!(libraries.iter().all(|library| !library.source_paths.is_empty()));
    Ok(libraries)
}

pub(crate) fn derive_mes_runtime_inventory(
    request: MesRuntimeInventoryRequest<'_>,
) -> Result<MesRuntimeInventoryReport, MesLibraryPlanError> {
    validate_runtime_inputs(&request)?;
    fs::create_dir(request.output_root).map_err(|source| io_error("creating create-new Mes runtime root", source))?;
    regenerate_nyacc_tables(&request)?;
    install_mes_runtime_skeleton(&request)?;
    let configure_path = request.mes_source_root.join("build-aux/configure-lib.sh");
    let configure_bytes = read_bounded_file(&configure_path, MES_RECIPE_BYTES_MAX, "configure-lib.sh")?;
    let configure = std::str::from_utf8(&configure_bytes)
        .map_err(|error| MesLibraryPlanError::Parse(format!("configure-lib.sh is not UTF-8: {error}")))?;
    let libraries = derive_mes_library_sources(configure)?;
    let (source_compile_count, archive_count) = build_mes_runtime_libraries(&request, &libraries)?;
    let outputs = collect_runtime_outputs(request.output_root)?;
    let report = MesRuntimeInventoryReport {
        format: MES_LIBRARY_REPORT_FORMAT,
        configure_lib_digest_blake3: blake3::hash(&configure_bytes).to_hex().to_string(),
        nyacc_command_count: u32::try_from(NYACC_COMMAND_COUNT)
            .map_err(|_| MesLibraryPlanError::Limit("NYACC command count does not fit u32".to_string()))?,
        source_compile_count,
        archive_count,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: MES_LIBRARY_NON_CLAIM,
    };
    write_json_create_new(&request.output_root.join("mes-runtime-inventory.json"), &report)?;
    assert_eq!(report.outputs.len(), MES_RUNTIME_REQUIRED_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_runtime_inputs(request: &MesRuntimeInventoryRequest<'_>) -> Result<(), MesLibraryPlanError> {
    if request.output_root.exists() {
        return Err(MesLibraryPlanError::Parse(format!(
            "create-new Mes output already exists: {}",
            request.output_root.display()
        )));
    }
    for (label, path) in [
        ("Mes source root", request.mes_source_root),
        ("NYACC source root", request.nyacc_source_root),
        ("full Stage0 root", request.stage0_root),
    ] {
        if !path.is_absolute() || !path.is_dir() {
            return Err(MesLibraryPlanError::Parse(format!(
                "{label} is not an absolute directory: {}",
                path.display()
            )));
        }
    }
    validate_digest(&request.mes_source_root.join("bin/mes-m2"), crate::stagex_mes::MES_M2_BLAKE3, "mes-m2")?;
    for artifact_id in ["stage0-m1", "stage0-full-blood-elf"] {
        let expected = find_stage0_expected(artifact_id)?;
        validate_digest(&request.stage0_root.join(expected.0), expected.1, artifact_id)?;
    }
    assert!(request.mes_source_root.join("build-aux/configure-lib.sh").is_file());
    assert!(request.nyacc_source_root.join("module").is_dir());
    Ok(())
}

fn regenerate_nyacc_tables(request: &MesRuntimeInventoryRequest<'_>) -> Result<(), MesLibraryPlanError> {
    let generated = [
        "module/nyacc/lang/c99/mach.d/c99-act.scm",
        "module/nyacc/lang/c99/mach.d/c99-tab.scm",
        "module/nyacc/lang/c99/mach.d/c99cx-act.scm",
        "module/nyacc/lang/c99/mach.d/c99cx-tab.scm",
        "module/nyacc/lang/c99/mach.d/c99x-act.scm",
        "module/nyacc/lang/c99/mach.d/c99x-tab.scm",
        "module/nyacc/lang/c99/mach.d/cpp-act.scm",
        "module/nyacc/lang/c99/mach.d/cpp-tab.scm",
    ];
    for relative in generated {
        remove_file_if_present(&request.nyacc_source_root.join(relative))?;
    }
    let mes = request.mes_source_root.join("bin/mes-m2");
    let load_path = format!(
        "{}/mes/module:{}/module:{}/module",
        utf8_absolute(request.mes_source_root, "Mes source root")?,
        utf8_absolute(request.mes_source_root, "Mes source root")?,
        utf8_absolute(request.nyacc_source_root, "NYACC source root")?
    );
    let commands = [
        ("nyacc-cpp", "gen-cpp-files.scm", NYACC_DEFAULT_ARENA_BYTES),
        ("nyacc-c99", "gen-c99-files.scm", NYACC_C99_ARENA_BYTES),
        ("nyacc-c99cx", "gen-c99cx-files.scm", NYACC_DEFAULT_ARENA_BYTES),
    ];
    for (label, script, arena) in commands {
        let stderr_path = request.output_root.join(format!("{label}.stderr.txt"));
        let args = ["-L", "module", script];
        let mut env =
            mes_environment(utf8_absolute(request.mes_source_root, "Mes source root")?, &load_path, arena, None, None);
        env.insert("MES_MAX_ARENA".to_string(), NYACC_MAX_ARENA_BYTES.to_string());
        env.insert("MES_STACK".to_string(), NYACC_STACK_BYTES.to_string());
        run_bounded_process(&mes, &args, request.nyacc_source_root, &env, &stderr_path)?;
    }
    for relative in generated {
        if !request.nyacc_source_root.join(relative).is_file() {
            return Err(MesLibraryPlanError::Parse(format!("NYACC regeneration did not produce {relative}")));
        }
    }
    assert_eq!(commands.len(), NYACC_COMMAND_COUNT);
    assert!(request.nyacc_source_root.join(generated[0]).is_file());
    Ok(())
}

fn install_mes_runtime_skeleton(request: &MesRuntimeInventoryRequest<'_>) -> Result<(), MesLibraryPlanError> {
    let output = request.output_root;
    for relative in ["bin", "lib/x86_64-mes", "lib/linux/x86_64-mes", "include", "mes"] {
        fs::create_dir_all(output.join(relative))
            .map_err(|source| io_error("creating Mes output directory", source))?;
    }
    copy_file_exact(&request.mes_source_root.join("bin/mes-m2"), &output.join("bin/mes-m2"))?;
    copy_file_exact(
        &request.mes_source_root.join("lib/x86_64-mes/x86_64.M1"),
        &output.join("lib/x86_64-mes/x86_64.M1"),
    )?;
    for file in ["elf64-header.hex2", "elf64-footer-single-main.hex2"] {
        copy_file_exact(
            &request.mes_source_root.join("lib/linux/x86_64-mes").join(file),
            &output.join("lib/linux/x86_64-mes").join(file),
        )?;
    }
    copy_tree_bounded(&request.mes_source_root.join("include"), &output.join("include/mes-include"))?;
    copy_tree_bounded(&request.mes_source_root.join("mes/module"), &output.join("mes/module"))?;
    copy_tree_bounded(&request.mes_source_root.join("module/mescc"), &output.join("mes/module/mescc"))?;
    copy_file_exact(&request.mes_source_root.join("module/mescc.scm"), &output.join("mes/module/mescc.scm"))?;
    for file in ["getopt-long.scm", "misc.scm", "test.scm"] {
        let source = request.mes_source_root.join("module/mes").join(file);
        if source.is_file() {
            copy_file_exact(&source, &output.join("mes/module/mes").join(file))?;
        }
    }
    merge_directory_children(&request.nyacc_source_root.join("module"), &output.join("mes/module"))?;
    restore_mes_safe_modules(request.mes_source_root, output)?;
    fs::create_dir_all(output.join("mes/module/ice-9"))
        .map_err(|source| io_error("creating Mes ice-9 module directory", source))?;
    write_create_new(&output.join("mes/module/ice-9/syncase.scm"), b"(define-module (ice-9 syncase))\n")?;
    generate_mescc_entrypoint(request.mes_source_root, output)?;
    patch_installed_mescc(output)?;
    copy_tree_bounded(&request.stage0_root.join("M2libc"), &output.join("lib/M2libc"))?;
    assert!(output.join("bin/mescc.scm").is_file());
    assert!(output.join("mes/module/mescc/compile.scm").is_file());
    Ok(())
}

fn restore_mes_safe_modules(mes_source_root: &Path, output: &Path) -> Result<(), MesLibraryPlanError> {
    let safe = [
        "nyacc/compat18.scm",
        "nyacc/lang/c99/pprint.scm",
        "rnrs/arithmetic/bitwise.scm",
        "system/foreign.scm",
    ];
    for relative in safe {
        copy_file_replace(
            &mes_source_root.join("mes/module").join(relative),
            &output.join("mes/module").join(relative),
        )?;
    }
    assert!(output.join("mes/module/system/foreign.scm").is_file());
    assert!(output.join("mes/module/nyacc/compat18.scm").is_file());
    Ok(())
}

fn generate_mescc_entrypoint(mes_source_root: &Path, output: &Path) -> Result<(), MesLibraryPlanError> {
    let template =
        read_bounded_file(&mes_source_root.join("scripts/mescc.scm.in"), MES_RECIPE_BYTES_MAX, "mescc template")?;
    let template = String::from_utf8(template)
        .map_err(|error| MesLibraryPlanError::Parse(format!("mescc template is not UTF-8: {error}")))?;
    let output_prefix = utf8_absolute(output, "Mes output root")?;
    let generated = render_mescc_entrypoint(&template, output_prefix)?;
    if generated.contains("@prefix@") || generated.contains("@GUILE@") {
        return Err(MesLibraryPlanError::Parse("mescc template retains required placeholders".to_string()));
    }
    write_create_new(&output.join("bin/mescc.scm"), generated.as_bytes())?;
    assert!(generated.contains(MES_LOGICAL_PREFIX));
    assert!(!generated.contains(output_prefix));
    assert!(!generated.contains("@VERSION@"));
    Ok(())
}

fn render_mescc_entrypoint(template: &str, output_prefix: &str) -> Result<String, MesLibraryPlanError> {
    let generated = template
        .replace("@prefix@", MES_LOGICAL_PREFIX)
        .replace("@VERSION@", "0.27.1")
        .replace("@mes_cpu@", "x86_64")
        .replace("@mes_kernel@", "linux")
        .replace("@guile_site_dir@", &format!("{MES_LOGICAL_PREFIX}/mes/module"))
        .replace("@guile_site_ccache_dir@", &format!("{MES_LOGICAL_PREFIX}/mes/module"))
        .replace("@GUILE@", &format!("{MES_LOGICAL_PREFIX}/bin/mes-m2"));
    if generated.contains("@prefix@") || generated.contains("@GUILE@") || generated.contains(output_prefix) {
        return Err(MesLibraryPlanError::Parse(
            "rendered mescc entrypoint retains a placeholder or physical path".to_string(),
        ));
    }
    assert!(generated.contains(MES_LOGICAL_PREFIX));
    assert!(!generated.contains(output_prefix));
    Ok(generated)
}

fn patch_installed_mescc(output: &Path) -> Result<(), MesLibraryPlanError> {
    replace_required_text(
        &output.join("mes/module/mescc/mescc.scm"),
        MESCC_ASSERT_SYSTEM_OLD,
        MESCC_ASSERT_SYSTEM_NEW,
    )?;
    replace_required_text(&output.join("mes/module/mescc/compile.scm"), MESCC_ARRAY_TYPE_OLD, MESCC_ARRAY_TYPE_NEW)?;
    assert!(
        read_bounded_file(&output.join("mes/module/mescc/mescc.scm"), MES_RECIPE_BYTES_MAX, "patched mescc")?
            .windows(MESCC_ASSERT_SYSTEM_NEW.len())
            .any(|window| window == MESCC_ASSERT_SYSTEM_NEW.as_bytes())
    );
    assert!(output.join("mes/module/mescc/compile.scm").is_file());
    Ok(())
}

fn evaluate_configure_assignments(
    configure_lib: &str,
    variables: &mut BTreeMap<String, String>,
) -> Result<(), MesLibraryPlanError> {
    let lines = configure_lib.lines().collect::<Vec<_>>();
    if lines.len() > CONFIGURE_LINE_COUNT_MAX {
        return Err(MesLibraryPlanError::Limit(format!(
            "configure-lib has {} lines; maximum is {CONFIGURE_LINE_COUNT_MAX}",
            lines.len()
        )));
    }
    let mut index = 0usize;
    let mut active = true;
    let mut conditions = Vec::<ConditionFrame>::new();
    while index < lines.len() {
        let line = lines[index].trim();
        if line.starts_with("if ") && line.ends_with("; then") {
            let condition_matches = evaluate_test_condition(line, variables)?;
            if conditions.len() >= CONFIGURE_CONDITION_DEPTH_MAX {
                return Err(MesLibraryPlanError::Limit(format!(
                    "condition nesting exceeds {CONFIGURE_CONDITION_DEPTH_MAX}"
                )));
            }
            conditions.push(ConditionFrame {
                parent_active: active,
                condition_matches,
                in_else: false,
            });
            active = active && condition_matches;
            index = index.saturating_add(1);
            continue;
        }
        if line == "else" {
            let frame = conditions
                .last_mut()
                .ok_or_else(|| MesLibraryPlanError::Parse("else without matching if".to_string()))?;
            if frame.in_else {
                return Err(MesLibraryPlanError::Parse("duplicate else in condition".to_string()));
            }
            frame.in_else = true;
            active = frame.parent_active && !frame.condition_matches;
            index = index.saturating_add(1);
            continue;
        }
        if line == "fi" {
            let frame =
                conditions.pop().ok_or_else(|| MesLibraryPlanError::Parse("fi without matching if".to_string()))?;
            active = frame.parent_active;
            index = index.saturating_add(1);
            continue;
        }
        if let Some((name, value, consumed_lines)) = parse_quoted_assignment(&lines, index)? {
            if active {
                let expanded = expand_variables(&value, variables)?;
                if expanded.len() > CONFIGURE_VALUE_BYTES_MAX {
                    return Err(MesLibraryPlanError::Limit(format!(
                        "expanded variable {name} exceeds {CONFIGURE_VALUE_BYTES_MAX} bytes"
                    )));
                }
                variables.insert(name, expanded);
                if variables.len() > CONFIGURE_VARIABLE_COUNT_MAX {
                    return Err(MesLibraryPlanError::Limit(format!(
                        "variable count exceeds {CONFIGURE_VARIABLE_COUNT_MAX}"
                    )));
                }
            }
            index = index
                .checked_add(consumed_lines)
                .ok_or_else(|| MesLibraryPlanError::Limit("line index overflow".to_string()))?;
            continue;
        }
        index = index.saturating_add(1);
    }
    if !conditions.is_empty() {
        return Err(MesLibraryPlanError::Parse("unterminated if block".to_string()));
    }
    assert!(index >= lines.len());
    assert!(conditions.is_empty());
    Ok(())
}

fn parse_quoted_assignment(
    lines: &[&str],
    start: usize,
) -> Result<Option<(String, String, usize)>, MesLibraryPlanError> {
    let line = lines[start].trim();
    let Some((name, right)) = line.split_once('=') else {
        return Ok(None);
    };
    if !valid_variable_name(name) || !right.starts_with('"') {
        return Ok(None);
    }
    let mut value = String::new();
    let first = &right[1..];
    if let Some(without_quote) = first.strip_suffix('"') {
        value.push_str(without_quote);
        return Ok(Some((name.to_string(), value, 1)));
    }
    value.push_str(first);
    let mut index = start.saturating_add(1);
    while index < lines.len() {
        value.push('\n');
        let candidate = lines[index];
        if let Some(without_quote) = candidate.strip_suffix('"') {
            value.push_str(without_quote);
            let consumed = index
                .checked_sub(start)
                .and_then(|distance| distance.checked_add(1))
                .ok_or_else(|| MesLibraryPlanError::Limit("assignment line count overflow".to_string()))?;
            return Ok(Some((name.to_string(), value, consumed)));
        }
        value.push_str(candidate);
        if value.len() > CONFIGURE_VALUE_BYTES_MAX {
            return Err(MesLibraryPlanError::Limit(format!(
                "quoted assignment {name} exceeds {CONFIGURE_VALUE_BYTES_MAX} bytes"
            )));
        }
        index = index.saturating_add(1);
    }
    Err(MesLibraryPlanError::Parse(format!("unterminated quoted assignment {name}")))
}

fn evaluate_test_condition(line: &str, variables: &BTreeMap<String, String>) -> Result<bool, MesLibraryPlanError> {
    let body = if let Some(body) = line.strip_prefix("if test ").and_then(|value| value.strip_suffix("; then")) {
        body
    } else if let Some(body) = line.strip_prefix("if [ ").and_then(|value| value.strip_suffix(" ]; then")) {
        body
    } else {
        return Err(MesLibraryPlanError::Parse(format!("unsupported if syntax: {line}")));
    };
    let fields = body.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 3 || fields[1] != "=" {
        return Err(MesLibraryPlanError::Parse(format!("unsupported test condition: {line}")));
    }
    let variable_token = fields[0].trim_matches('"');
    let variable_name = variable_token
        .strip_prefix('$')
        .ok_or_else(|| MesLibraryPlanError::Parse(format!("test does not reference a variable: {line}")))?;
    if !valid_variable_name(variable_name) {
        return Err(MesLibraryPlanError::Parse(format!("invalid test variable {variable_name}")));
    }
    let actual = variables
        .get(variable_name)
        .ok_or_else(|| MesLibraryPlanError::Parse(format!("test references unknown variable {variable_name}")))?;
    assert_eq!(fields.len(), 3);
    assert!(!fields[2].is_empty());
    Ok(actual == fields[2])
}

fn expand_variables(value: &str, variables: &BTreeMap<String, String>) -> Result<String, MesLibraryPlanError> {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'$' {
            output.push(char::from(bytes[index]));
            index = index.saturating_add(1);
            continue;
        }
        let (name, next) = parse_variable_reference(value, index)?;
        let replacement = variables
            .get(name)
            .ok_or_else(|| MesLibraryPlanError::Parse(format!("assignment references unknown variable {name}")))?;
        output.push_str(replacement);
        index = next;
        if output.len() > CONFIGURE_VALUE_BYTES_MAX {
            return Err(MesLibraryPlanError::Limit(format!(
                "expanded assignment exceeds {CONFIGURE_VALUE_BYTES_MAX} bytes"
            )));
        }
    }
    assert!(index >= bytes.len());
    assert!(output.len() <= CONFIGURE_VALUE_BYTES_MAX);
    Ok(output)
}

fn parse_variable_reference(value: &str, dollar: usize) -> Result<(&str, usize), MesLibraryPlanError> {
    let bytes = value.as_bytes();
    let start = dollar.saturating_add(1);
    if bytes.get(start) == Some(&b'{') {
        let name_start = start.saturating_add(1);
        let close = value[name_start..]
            .find('}')
            .map(|offset| name_start.saturating_add(offset))
            .ok_or_else(|| MesLibraryPlanError::Parse("unterminated braced variable".to_string()))?;
        let name = &value[name_start..close];
        if !valid_variable_name(name) {
            return Err(MesLibraryPlanError::Parse(format!("invalid variable reference {name}")));
        }
        return Ok((name, close.saturating_add(1)));
    }
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end = end.saturating_add(1);
    }
    let name = &value[start..end];
    if !valid_variable_name(name) {
        return Err(MesLibraryPlanError::Parse("empty or invalid variable reference".to_string()));
    }
    Ok((name, end))
}

fn bounded_source_paths(variable: &str, value: &str) -> Result<Vec<String>, MesLibraryPlanError> {
    let paths = value.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    if paths.is_empty() || paths.len() > LIBRARY_SOURCE_COUNT_MAX {
        return Err(MesLibraryPlanError::Limit(format!(
            "{variable} has {} sources; required range is 1..={LIBRARY_SOURCE_COUNT_MAX}",
            paths.len()
        )));
    }
    for path in &paths {
        if path.starts_with('/') || path.contains("..") || !path.ends_with(".c") {
            return Err(MesLibraryPlanError::Parse(format!("{variable} has unsafe source path '{path}'")));
        }
    }
    assert!(!paths.is_empty());
    assert!(paths.len() <= LIBRARY_SOURCE_COUNT_MAX);
    Ok(paths)
}

fn valid_variable_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| byte == b'_' || byte.is_ascii_alphabetic() || (index > 0 && byte.is_ascii_digit()))
}

pub(crate) fn derive_mes_unified_libc_sources(configure_lib: &str) -> Result<Vec<String>, MesLibraryPlanError> {
    let mut variables = BTreeMap::from([
        ("mes_libc".to_string(), "mes".to_string()),
        ("mes_kernel".to_string(), "linux".to_string()),
        ("mes_cpu".to_string(), "x86_64".to_string()),
        ("compiler".to_string(), "gcc".to_string()),
        ("V".to_string(), "1".to_string()),
    ]);
    evaluate_configure_assignments(configure_lib, &mut variables)?;
    let value = variables
        .get("libc_gnu_SOURCES")
        .ok_or_else(|| MesLibraryPlanError::Parse("required variable libc_gnu_SOURCES is missing".to_string()))?;
    let paths = bounded_source_paths("libc_gnu_SOURCES", value)?;
    assert!(!paths.is_empty());
    assert!(paths.len() <= LIBRARY_SOURCE_COUNT_MAX);
    Ok(paths)
}

fn build_mes_runtime_libraries(
    request: &MesRuntimeInventoryRequest<'_>,
    libraries: &[MesLibrarySources],
) -> Result<(u32, u32), MesLibraryPlanError> {
    let build_root = request.output_root.join("runtime-build");
    fs::create_dir(&build_root).map_err(|source| io_error("creating Mes runtime build directory", source))?;
    let mes = request.mes_source_root.join("bin/mes-m2");
    let mescc = request.output_root.join("bin/mescc.scm");
    copy_tree_bounded(&request.mes_source_root.join("include"), &build_root.join("include"))?;
    let m1 = request.stage0_root.join(find_stage0_expected("stage0-m1")?.0);
    let blood_elf = request.stage0_root.join(find_stage0_expected("stage0-full-blood-elf")?.0);
    let load_path = format!("{}/mes/module", utf8_absolute(request.output_root, "Mes output root")?);
    let mut compiled = BTreeMap::<String, (PathBuf, PathBuf)>::new();
    let mut compile_count = 0u32;
    let mut archive_count = 0u32;
    for library in libraries {
        if library.name == "crt1.o" {
            let source = library.source_paths.first().expect("crt1 source is nonempty");
            let object = build_root.join("crt1.o");
            compile_mes_source(
                &mes,
                &mescc,
                &m1,
                &blood_elf,
                request.mes_source_root,
                &build_root,
                source,
                &object,
                &load_path,
                compile_count,
            )?;
            compile_count = compile_count
                .checked_add(1)
                .ok_or_else(|| MesLibraryPlanError::Limit("Mes compile count overflow".to_string()))?;
            copy_file_exact(&object, &request.output_root.join("lib/x86_64-mes/crt1.o"))?;
            let assembly = object.with_extension("s");
            if assembly.is_file() {
                copy_file_exact(&assembly, &request.output_root.join("lib/x86_64-mes/crt1.s"))?;
            }
            continue;
        }
        let mut objects = Vec::with_capacity(library.source_paths.len());
        for source in &library.source_paths {
            let (object, assembly) = if let Some(paths) = compiled.get(source) {
                paths.clone()
            } else {
                let object = build_root.join(object_name_for_source(source)?);
                compile_mes_source(
                    &mes,
                    &mescc,
                    &m1,
                    &blood_elf,
                    request.mes_source_root,
                    &build_root,
                    source,
                    &object,
                    &load_path,
                    compile_count,
                )?;
                let assembly = object.with_extension("s");
                if !assembly.is_file() {
                    return Err(MesLibraryPlanError::Parse(format!(
                        "mescc did not produce assembly for {source}: {}",
                        assembly.display()
                    )));
                }
                compile_count = compile_count
                    .checked_add(1)
                    .ok_or_else(|| MesLibraryPlanError::Limit("Mes compile count overflow".to_string()))?;
                compiled.insert(source.clone(), (object.clone(), assembly.clone()));
                (object, assembly)
            };
            objects.push((object, assembly));
        }
        let archive = build_root.join(library.name);
        let assembly_archive = archive.with_extension("s");
        concatenate_files_create_new(&archive, objects.iter().map(|(object, _)| object.as_path()))?;
        concatenate_files_create_new(&assembly_archive, objects.iter().map(|(_, assembly)| assembly.as_path()))?;
        copy_file_exact(&archive, &request.output_root.join("lib/x86_64-mes").join(library.name))?;
        copy_file_exact(
            &assembly_archive,
            &request
                .output_root
                .join("lib/x86_64-mes")
                .join(Path::new(library.name).with_extension("s").file_name().expect("archive assembly has file name")),
        )?;
        archive_count = archive_count
            .checked_add(1)
            .ok_or_else(|| MesLibraryPlanError::Limit("Mes archive count overflow".to_string()))?;
    }
    if usize::try_from(archive_count).ok() != Some(MES_RUNTIME_ARCHIVE_COUNT) {
        return Err(MesLibraryPlanError::Parse(format!(
            "expected {MES_RUNTIME_ARCHIVE_COUNT} Mes archives, produced {archive_count}"
        )));
    }
    assert_eq!(usize::try_from(archive_count).ok(), Some(MES_RUNTIME_ARCHIVE_COUNT));
    assert!(compile_count > 0);
    Ok((compile_count, archive_count))
}

#[allow(clippy::too_many_arguments)]
fn compile_mes_source(
    mes: &Path,
    mescc: &Path,
    m1: &Path,
    blood_elf: &Path,
    source_root: &Path,
    build_root: &Path,
    source_relative: &str,
    object: &Path,
    load_path: &str,
    compile_index: u32,
) -> Result<(), MesLibraryPlanError> {
    let source = source_root.join(source_relative);
    if !source.is_file() || !object.starts_with(build_root) {
        return Err(MesLibraryPlanError::Parse(format!(
            "invalid Mes compile source or output: {} -> {}",
            source.display(),
            object.display()
        )));
    }
    let staged_source_relative = Path::new("sources").join(source_relative);
    let staged_source = build_root.join(&staged_source_relative);
    if !staged_source.is_file() {
        copy_file_exact(&source, &staged_source)?;
    }
    let object_relative = object
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| MesLibraryPlanError::Parse(format!("Mes object name is not UTF-8: {}", object.display())))?;
    let staged_source_text = staged_source_relative.to_str().ok_or_else(|| {
        MesLibraryPlanError::Parse(format!("staged Mes source path is not UTF-8: {}", staged_source_relative.display()))
    })?;
    let args = vec![
        "--no-auto-compile".to_string(),
        "-e".to_string(),
        "main".to_string(),
        utf8_absolute(mescc, "mescc entrypoint")?.to_string(),
        "--".to_string(),
        "-c".to_string(),
        "-D".to_string(),
        "HAVE_CONFIG_H=1".to_string(),
        "-Iinclude".to_string(),
        "-Iinclude/linux/x86_64".to_string(),
        "-o".to_string(),
        object_relative.to_string(),
        staged_source_text.to_string(),
    ];
    let env = mes_environment(
        utf8_absolute(mescc.parent().and_then(Path::parent).expect("mescc output has prefix"), "Mes output prefix")?,
        load_path,
        MES_ARENA_BYTES,
        Some(("M1", utf8_absolute(m1, "Stage0 M1")?)),
        Some(("BLOOD_ELF", utf8_absolute(blood_elf, "Stage0 blood-elf")?)),
    );
    let stderr = build_root.join(format!("compile-{compile_index:04}.stderr.txt"));
    run_bounded_process(mes, &args, build_root, &env, &stderr)?;
    validate_nonempty_output(object, MES_FILE_BYTES_MAX, source_relative)?;
    let assembly = object.with_extension("s");
    validate_nonempty_output(&assembly, MES_FILE_BYTES_MAX, "Mes assembly")?;
    assert!(object.starts_with(build_root));
    assert!(assembly.starts_with(build_root));
    Ok(())
}

fn collect_runtime_outputs(output_root: &Path) -> Result<Vec<MesRuntimeOutput>, MesLibraryPlanError> {
    let mut outputs = Vec::with_capacity(MES_RUNTIME_REQUIRED_OUTPUT_COUNT);
    for expected in MES_RUNTIME_EXPECTED_OUTPUTS {
        let path = output_root.join(expected.relative_path);
        let bytes = read_bounded_file(&path, MES_FILE_BYTES_MAX, expected.artifact_id)?;
        let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
        if digest_blake3 != expected.digest_blake3 {
            return Err(MesLibraryPlanError::Parse(format!(
                "Mes output {} BLAKE3 mismatch: expected {}, observed {}",
                expected.artifact_id, expected.digest_blake3, digest_blake3
            )));
        }
        outputs.push(MesRuntimeOutput {
            artifact_id: expected.artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| MesLibraryPlanError::Limit("Mes runtime output size does not fit u64".to_string()))?,
            digest_blake3,
        });
    }
    assert_eq!(outputs.len(), MES_RUNTIME_REQUIRED_OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn object_name_for_source(source: &str) -> Result<String, MesLibraryPlanError> {
    if source.starts_with('/') || source.contains("..") || !source.ends_with(".c") {
        return Err(MesLibraryPlanError::Parse(format!("unsafe Mes source path '{source}'")));
    }
    let stem = source.trim_start_matches("./").strip_suffix(".c").expect("source suffix checked");
    let name = format!("{}.o", stem.replace('/', "-"));
    if name.len() > 255 {
        return Err(MesLibraryPlanError::Limit(format!("Mes object name exceeds 255 bytes: {name}")));
    }
    assert!(name.ends_with(".o"));
    assert!(!name.contains('/'));
    Ok(name)
}

fn concatenate_files_create_new<'a>(
    target: &Path,
    sources: impl Iterator<Item = &'a Path>,
) -> Result<(), MesLibraryPlanError> {
    let mut output = File::options()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|source| io_error("creating Mes archive", source))?;
    let mut source_count = 0u32;
    let mut total_bytes = 0u64;
    for source in sources {
        let bytes = read_bounded_file(source, MES_FILE_BYTES_MAX, "Mes archive member")?;
        output.write_all(&bytes).map_err(|error| io_error("writing Mes archive member", error))?;
        source_count = source_count
            .checked_add(1)
            .ok_or_else(|| MesLibraryPlanError::Limit("Mes archive member count overflow".to_string()))?;
        total_bytes = total_bytes
            .checked_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
            .ok_or_else(|| MesLibraryPlanError::Limit("Mes archive byte count overflow".to_string()))?;
        if total_bytes > MES_FILE_BYTES_MAX {
            return Err(MesLibraryPlanError::Limit(format!("Mes archive exceeds {MES_FILE_BYTES_MAX} bytes")));
        }
    }
    output.sync_all().map_err(|source| io_error("syncing Mes archive", source))?;
    if source_count == 0 || total_bytes == 0 {
        return Err(MesLibraryPlanError::Parse("Mes archive has no members or bytes".to_string()));
    }
    assert!(target.is_file());
    assert!(total_bytes <= MES_FILE_BYTES_MAX);
    Ok(())
}

pub(crate) fn run_bounded_process<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    environment: &BTreeMap<String, String>,
    stderr_path: &Path,
) -> Result<(), MesLibraryPlanError> {
    run_bounded_process_with_stdio(
        executable,
        args,
        current_dir,
        environment,
        Stdio::null(),
        Stdio::null(),
        None,
        stderr_path,
    )
}

pub(crate) fn run_bounded_process_capturing_stdout<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    environment: &BTreeMap<String, String>,
    stdout_path: &Path,
    stdout_bytes_max: u64,
    stderr_path: &Path,
) -> Result<(), MesLibraryPlanError> {
    let stdout_file = File::create(stdout_path).map_err(|source| io_error("creating process stdout", source))?;
    let result = run_bounded_process_with_stdio(
        executable,
        args,
        current_dir,
        environment,
        Stdio::null(),
        Stdio::from(stdout_file),
        Some((stdout_path, stdout_bytes_max)),
        stderr_path,
    );
    assert!(stdout_path.is_file());
    assert!(stderr_path.is_file());
    result
}

pub(crate) fn run_bounded_process_with_stdin_capturing_stdout<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    environment: &BTreeMap<String, String>,
    stdin_path: &Path,
    stdin_bytes_max: u64,
    stdout_path: &Path,
    stdout_bytes_max: u64,
    stderr_path: &Path,
) -> Result<(), MesLibraryPlanError> {
    let stdin_file = open_bounded_process_stdin(stdin_path, stdin_bytes_max)?;
    let stdout_file = File::create(stdout_path).map_err(|source| io_error("creating process stdout", source))?;
    let result = run_bounded_process_with_stdio(
        executable,
        args,
        current_dir,
        environment,
        Stdio::from(stdin_file),
        Stdio::from(stdout_file),
        Some((stdout_path, stdout_bytes_max)),
        stderr_path,
    );
    assert!(stdin_path.is_file());
    assert!(stdout_path.is_file());
    result
}

fn run_bounded_process_with_stdio<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    args: &[S],
    current_dir: &Path,
    environment: &BTreeMap<String, String>,
    stdin: Stdio,
    stdout: Stdio,
    stdout_limit: Option<(&Path, u64)>,
    stderr_path: &Path,
) -> Result<(), MesLibraryPlanError> {
    let stderr_file = File::create(stderr_path).map_err(|source| io_error("creating process stderr", source))?;
    let mut child = Command::new(executable)
        .args(args)
        .current_dir(current_dir)
        .env_clear()
        .envs(environment)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|source| io_error(format!("spawning {}", executable.display()), source))?;
    let started = Instant::now();
    loop {
        if let Err(error) = enforce_optional_observation_file_limit(stdout_limit) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for process", source))? {
            return classify_process_status(executable, status, stderr_path);
        }
        if started.elapsed() >= Duration::from_millis(MES_PROCESS_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(MesLibraryPlanError::ProcessTimeout {
                executable: executable.to_path_buf(),
                timeout_ms: MES_PROCESS_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(MES_PROCESS_POLL_MS));
    }
}

fn open_bounded_process_stdin(path: &Path, bytes_max: u64) -> Result<File, MesLibraryPlanError> {
    let metadata = fs::metadata(path).map_err(|source| io_error("reading process stdin metadata", source))?;
    if !metadata.is_file() {
        return Err(MesLibraryPlanError::Limit(format!("process stdin is not a regular file: {}", path.display())));
    }
    if metadata.len() > bytes_max {
        return Err(MesLibraryPlanError::Limit(format!(
            "process stdin exceeds {bytes_max} bytes: {} has {} bytes",
            path.display(),
            metadata.len()
        )));
    }
    let file = File::open(path).map_err(|source| io_error("opening process stdin", source))?;
    assert!(metadata.len() <= bytes_max);
    assert!(path.is_file());
    Ok(file)
}

fn enforce_optional_observation_file_limit(stdout_limit: Option<(&Path, u64)>) -> Result<(), MesLibraryPlanError> {
    let Some((stdout_path, stdout_bytes_max)) = stdout_limit else {
        return Ok(());
    };
    enforce_observation_file_limit(stdout_path, stdout_bytes_max)
}

fn enforce_observation_file_limit(path: &Path, bytes_max: u64) -> Result<(), MesLibraryPlanError> {
    let bytes_len = fs::metadata(path).map_err(|source| io_error("reading process stdout metadata", source))?.len();
    if bytes_len > bytes_max {
        return Err(MesLibraryPlanError::Limit(format!(
            "process stdout exceeds {bytes_max} bytes: {} has {bytes_len} bytes",
            path.display()
        )));
    }
    assert!(bytes_len <= bytes_max);
    assert!(path.is_file());
    Ok(())
}

fn classify_process_status(
    executable: &Path,
    status: ExitStatus,
    stderr_path: &Path,
) -> Result<(), MesLibraryPlanError> {
    let stderr = read_observation(stderr_path, MES_PROCESS_STDERR_BYTES_MAX, "Mes process stderr")?;
    if status.success() {
        return Ok(());
    }
    Err(MesLibraryPlanError::ProcessFailure {
        executable: executable.to_path_buf(),
        exit_code: status.code(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

fn mes_environment(
    mes_prefix: &str,
    load_path: &str,
    arena_bytes: &str,
    first_tool: Option<(&str, &str)>,
    second_tool: Option<(&str, &str)>,
) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::from([
        ("MES_PREFIX".to_string(), mes_prefix.to_string()),
        ("GUILE_LOAD_PATH".to_string(), load_path.to_string()),
        ("MES_ARENA".to_string(), arena_bytes.to_string()),
        ("MES_MAX_ARENA".to_string(), MES_MAX_ARENA_BYTES.to_string()),
        ("MES_STACK".to_string(), MES_STACK_BYTES.to_string()),
    ]);
    if let Some((name, value)) = first_tool {
        environment.insert(name.to_string(), value.to_string());
    }
    if let Some((name, value)) = second_tool {
        environment.insert(name.to_string(), value.to_string());
    }
    assert!(environment.contains_key("MES_PREFIX"));
    assert!(environment.contains_key("GUILE_LOAD_PATH"));
    assert!(environment.contains_key("MES_ARENA"));
    environment
}

fn find_stage0_expected(artifact_id: &str) -> Result<(&'static str, &'static str), MesLibraryPlanError> {
    if let Some(expected) = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok((expected.relative_path, expected.digest_blake3));
    }
    if let Some(expected) = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
    {
        return Ok((expected.relative_path, expected.digest_blake3));
    }
    Err(MesLibraryPlanError::Parse(format!("unknown Stage0 artifact {artifact_id}")))
}

pub(crate) fn copy_tree_bounded(source: &Path, target: &Path) -> Result<(), MesLibraryPlanError> {
    let mut pending = vec![(source.to_path_buf(), target.to_path_buf())];
    let mut entry_count = 0u32;
    while let Some((from, to)) = pending.pop() {
        entry_count = entry_count
            .checked_add(1)
            .ok_or_else(|| MesLibraryPlanError::Limit("Mes copy entry count overflow".to_string()))?;
        if entry_count > MES_TREE_ENTRY_COUNT_MAX {
            return Err(MesLibraryPlanError::Limit(format!("Mes copy exceeds {MES_TREE_ENTRY_COUNT_MAX} entries")));
        }
        let metadata = fs::symlink_metadata(&from).map_err(|source| io_error("reading Mes copy metadata", source))?;
        if metadata.file_type().is_symlink() {
            copy_symlink_replace(&from, &to)?;
        } else if metadata.is_file() {
            copy_file_replace(&from, &to)?;
        } else if metadata.is_dir() {
            fs::create_dir_all(&to).map_err(|source| io_error("creating Mes copy directory", source))?;
            let mut entries = fs::read_dir(&from)
                .map_err(|source| io_error("reading Mes copy directory", source))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|source| io_error("reading Mes copy entry", source))?;
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries.into_iter().rev() {
                pending.push((entry.path(), to.join(entry.file_name())));
            }
        } else {
            return Err(MesLibraryPlanError::Parse(format!("unsupported Mes copy path type: {}", from.display())));
        }
    }
    assert!(entry_count > 0);
    assert!(entry_count <= MES_TREE_ENTRY_COUNT_MAX);
    Ok(())
}

fn merge_directory_children(source: &Path, target: &Path) -> Result<(), MesLibraryPlanError> {
    let mut entries = fs::read_dir(source)
        .map_err(|error| io_error("reading Mes merge directory", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("reading Mes merge entry", error))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    if entries.is_empty() || entries.len() > usize::try_from(MES_TREE_ENTRY_COUNT_MAX).unwrap_or(usize::MAX) {
        return Err(MesLibraryPlanError::Limit("Mes merge child count is empty or excessive".to_string()));
    }
    for entry in entries {
        copy_tree_bounded(&entry.path(), &target.join(entry.file_name()))?;
    }
    assert!(target.is_dir());
    assert!(
        !fs::read_dir(target)
            .map_err(|error| io_error("checking Mes merge", error))?
            .collect::<Vec<_>>()
            .is_empty()
    );
    Ok(())
}

pub(crate) fn copy_file_exact(source: &Path, target: &Path) -> Result<(), MesLibraryPlanError> {
    if target.exists() {
        return Err(MesLibraryPlanError::Parse(format!("create-new Mes copy target exists: {}", target.display())));
    }
    copy_file_replace(source, target)
}

pub(crate) fn copy_file_replace(source: &Path, target: &Path) -> Result<(), MesLibraryPlanError> {
    let metadata = fs::metadata(source).map_err(|error| io_error("reading Mes copy source metadata", error))?;
    if !metadata.is_file() || metadata.len() > MES_FILE_BYTES_MAX {
        return Err(MesLibraryPlanError::Limit(format!(
            "Mes copy source must be a regular file of at most {MES_FILE_BYTES_MAX} bytes, observed {} at {}",
            metadata.len(),
            source.display()
        )));
    }
    let bytes = fs::read(source).map_err(|error| io_error("reading Mes copy source", error))?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error("creating Mes copy parent", error))?;
    }
    remove_path_if_present(target)?;
    write_create_new(target, &bytes)?;
    let permissions = fs::metadata(source)
        .map_err(|error| io_error("reading Mes source permissions", error))?
        .permissions();
    fs::set_permissions(target, permissions).map_err(|error| io_error("setting Mes copy permissions", error))?;
    assert!(target.is_file());
    assert_eq!(fs::metadata(target).map(|metadata| metadata.len()).ok(), u64::try_from(bytes.len()).ok());
    Ok(())
}

fn copy_symlink_replace(source: &Path, target: &Path) -> Result<(), MesLibraryPlanError> {
    let link = fs::read_link(source).map_err(|error| io_error("reading Mes symlink", error))?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error("creating Mes symlink parent", error))?;
    }
    remove_path_if_present(target)?;
    std::os::unix::fs::symlink(&link, target).map_err(|error| io_error("creating Mes symlink", error))?;
    assert!(fs::symlink_metadata(target).map(|metadata| metadata.file_type().is_symlink()).unwrap_or(false));
    assert_eq!(fs::read_link(target).ok().as_ref(), Some(&link));
    Ok(())
}

pub(crate) fn remove_path_if_present(path: &Path) -> Result<(), MesLibraryPlanError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error("reading Mes replacement target", error)),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(|error| io_error("removing Mes replacement directory", error))?;
    } else {
        fs::remove_file(path).map_err(|error| io_error("removing Mes replacement path", error))?;
    }
    assert!(!path.exists());
    assert!(fs::symlink_metadata(path).is_err());
    Ok(())
}

fn remove_file_if_present(path: &Path) -> Result<(), MesLibraryPlanError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("removing Mes generated file", error)),
    }
}

fn replace_required_text(path: &Path, old: &str, new: &str) -> Result<(), MesLibraryPlanError> {
    let bytes = read_bounded_file(path, MES_RECIPE_BYTES_MAX, "Mes patch target")?;
    let text = String::from_utf8(bytes)
        .map_err(|error| MesLibraryPlanError::Parse(format!("Mes patch target is not UTF-8: {error}")))?;
    if !text.contains(old) {
        return Err(MesLibraryPlanError::Parse(format!("Mes patch target lacks required text: {}", path.display())));
    }
    const PATCH_MATCH_COUNT_MAX: usize = 16;
    let match_count = text.matches(old).count();
    if match_count == 0 || match_count > PATCH_MATCH_COUNT_MAX {
        return Err(MesLibraryPlanError::Limit(format!(
            "Mes patch match count {match_count} is outside 1..={PATCH_MATCH_COUNT_MAX}"
        )));
    }
    let patched = text.replace(old, new);
    fs::write(path, patched.as_bytes()).map_err(|error| io_error("writing Mes patch", error))?;
    assert!(patched.contains(new));
    assert!(!patched.contains(old));
    Ok(())
}

fn validate_digest(path: &Path, expected: &str, label: &str) -> Result<(), MesLibraryPlanError> {
    let bytes = read_bounded_file(path, MES_FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(MesLibraryPlanError::Parse(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_output(path: &Path, max_bytes: u64, label: &str) -> Result<(), MesLibraryPlanError> {
    let bytes = read_bounded_file(path, max_bytes, label)?;
    assert!(!bytes.is_empty());
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= max_bytes);
    Ok(())
}

pub(crate) fn read_bounded_file(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, MesLibraryPlanError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {label} metadata"), source))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(MesLibraryPlanError::Limit(format!(
            "{label} must be a nonempty regular file of at most {max_bytes} bytes, observed {} at {}",
            metadata.len(),
            path.display()
        )));
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len())
            .map_err(|_| MesLibraryPlanError::Limit(format!("{label} size does not fit usize")))?,
    );
    File::open(path)
        .and_then(|file| file.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes))
        .map_err(|source| io_error(format!("reading {label}"), source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(MesLibraryPlanError::Limit(format!("{label} exceeds {max_bytes} bytes")));
    }
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    assert!(!bytes.is_empty());
    Ok(bytes)
}

fn read_observation(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, MesLibraryPlanError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {label} metadata"), source))?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(MesLibraryPlanError::Limit(format!(
            "{label} must be a regular file of at most {max_bytes} bytes, observed {}",
            metadata.len()
        )));
    }
    let bytes = fs::read(path).map_err(|source| io_error(format!("reading {label}"), source))?;
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= max_bytes);
    Ok(bytes)
}

pub(crate) fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), MesLibraryPlanError> {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error(format!("creating {}", path.display()), source))?;
    file.write_all(bytes).map_err(|source| io_error(format!("writing {}", path.display()), source))?;
    file.sync_all().map_err(|source| io_error(format!("syncing {}", path.display()), source))?;
    assert!(path.is_file());
    assert_eq!(fs::metadata(path).map(|metadata| metadata.len()).ok(), u64::try_from(bytes.len()).ok());
    Ok(())
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), MesLibraryPlanError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| MesLibraryPlanError::Parse(format!("serializing {}: {error}", path.display())))?;
    write_create_new(path, &bytes)
}

pub(crate) fn utf8_absolute<'a>(path: &'a Path, label: &str) -> Result<&'a str, MesLibraryPlanError> {
    if !path.is_absolute() {
        return Err(MesLibraryPlanError::Parse(format!("{label} is not absolute: {}", path.display())));
    }
    let text = path
        .to_str()
        .ok_or_else(|| MesLibraryPlanError::Parse(format!("{label} is not UTF-8: {}", path.display())))?;
    assert!(text.starts_with('/'));
    assert!(!text.is_empty());
    Ok(text)
}

fn io_error(action: impl Into<String>, source: io::Error) -> MesLibraryPlanError {
    MesLibraryPlanError::Io {
        action: action.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_linux_x86_64_mescc_source_sets() {
        let configure = "libc_mini_SOURCES=\"lib/mes/eputs.c\"\n\
libmes_SOURCES=\"$libc_mini_SOURCES lib/mes/itoa.c\"\n\
libmescc_SOURCES=\"lib/$mes_kernel/$mes_cpu-mes-$compiler/syscall.c\"\n\
libc_SOURCES=\"$libmes_SOURCES lib/string/memcpy.c\"\n\
if test $mes_kernel = linux; then\n\
libc_SOURCES=\"$libc_SOURCES lib/linux/wait4.c\"\n\
else\n\
libc_SOURCES=\"$libc_SOURCES lib/stub/wait4.c\"\n\
fi\n\
libc_tcc_SOURCES=\"$libc_SOURCES lib/string/strcat.c\"\n\
libtcc1_SOURCES=\"lib/libtcc1.c\"\n";

        let libraries = derive_mes_library_sources(configure).unwrap();

        assert_eq!(libraries.len(), MES_LIBRARY_COUNT);
        assert!(libraries.iter().find(|library| library.name == "libc+tcc.a").unwrap().source_paths.len() >= 4);
        assert!(
            libraries
                .iter()
                .find(|library| library.name == "libc.a")
                .unwrap()
                .source_paths
                .contains(&"lib/linux/wait4.c".to_string())
        );
        assert!(
            libraries
                .iter()
                .find(|library| library.name == "libmescc.a")
                .unwrap()
                .source_paths
                .contains(&"lib/linux/x86_64-mes-mescc/syscall.c".to_string())
        );
    }

    #[test]
    #[ignore = "requires explicit Mes, NYACC, full Stage0, and create-new output roots"]
    fn derives_retained_mes_runtime_inventory() {
        let mes_root = std::path::PathBuf::from(std::env::var("MANTLE_STAGE_X_MES_SOURCE_ROOT").unwrap());
        let nyacc_root = std::path::PathBuf::from(std::env::var("MANTLE_STAGE_X_NYACC_SOURCE_ROOT").unwrap());
        let stage0_root = std::path::PathBuf::from(std::env::var("MANTLE_STAGE_X_STAGE0_FULL_ROOT").unwrap());
        let output_root = std::path::PathBuf::from(std::env::var("MANTLE_STAGE_X_MES_RUNTIME_OUTPUT").unwrap());

        let report = derive_mes_runtime_inventory(MesRuntimeInventoryRequest {
            mes_source_root: &mes_root,
            nyacc_source_root: &nyacc_root,
            stage0_root: &stage0_root,
            output_root: &output_root,
            protected_exec_enforced: false,
        })
        .unwrap();

        assert_eq!(report.outputs.len(), MES_RUNTIME_REQUIRED_OUTPUT_COUNT);
        assert_eq!(usize::try_from(report.archive_count).unwrap(), MES_RUNTIME_ARCHIVE_COUNT);
        assert!(report.source_compile_count > 0);
        assert!(output_root.join("mes-runtime-inventory.json").is_file());
    }

    #[test]
    #[ignore = "requires an explicit authenticated Mes source root"]
    fn parses_retained_authenticated_mes_recipe() {
        let root = std::path::PathBuf::from(std::env::var("MANTLE_STAGE_X_MES_SOURCE_ROOT").unwrap());
        let configure = std::fs::read_to_string(root.join("build-aux/configure-lib.sh")).unwrap();

        let libraries = derive_mes_library_sources(&configure).unwrap();
        let template = std::fs::read_to_string(root.join("scripts/mescc.scm.in")).unwrap();
        let entrypoint = render_mescc_entrypoint(&template, "/physical/output").unwrap();
        println!("mescc-entrypoint={}", blake3::hash(entrypoint.as_bytes()).to_hex());

        for library in &libraries {
            println!("{}={}", library.name, library.source_paths.len());
        }
        assert_eq!(libraries.len(), MES_LIBRARY_COUNT);
        assert!(libraries.iter().all(|library| library.source_paths.iter().all(|path| root.join(path).is_file())));
    }

    #[test]
    fn stdout_capture_limit_accepts_bound_and_rejects_overflow() {
        const STDOUT_BYTES_MAX: u64 = 1;
        const OVER_LIMIT_STDOUT: &[u8] = b"xx";
        let temp = tempfile::tempdir().unwrap();
        let stdout = temp.path().join("stdout.txt");
        fs::write(&stdout, b"x").unwrap();
        enforce_observation_file_limit(&stdout, STDOUT_BYTES_MAX).unwrap();
        fs::write(&stdout, OVER_LIMIT_STDOUT).unwrap();
        let error = enforce_observation_file_limit(&stdout, STDOUT_BYTES_MAX).unwrap_err();
        assert!(error.to_string().contains("process stdout exceeds"));
        assert!(error.to_string().contains(stdout.to_str().unwrap()));
    }

    #[test]
    fn process_stdin_limit_accepts_bound_and_rejects_overflow() {
        const STDIN_BYTES_MAX: u64 = 1;
        const OVER_LIMIT_STDIN: &[u8] = b"xx";
        let temp = tempfile::tempdir().unwrap();
        let stdin = temp.path().join("stdin.txt");
        fs::write(&stdin, b"x").unwrap();
        drop(open_bounded_process_stdin(&stdin, STDIN_BYTES_MAX).unwrap());
        fs::write(&stdin, OVER_LIMIT_STDIN).unwrap();
        let error = open_bounded_process_stdin(&stdin, STDIN_BYTES_MAX).unwrap_err();
        assert!(error.to_string().contains("process stdin exceeds"));
        assert!(error.to_string().contains(stdin.to_str().unwrap()));
    }

    #[test]
    fn rejects_unknown_variables_unsafe_paths_and_unterminated_conditions() {
        let unknown = derive_mes_library_sources("libc_mini_SOURCES=\"$missing\"").unwrap_err();
        assert!(unknown.to_string().contains("unknown variable"));

        let unsafe_error = bounded_source_paths("bad", "../host.c").unwrap_err();
        assert!(unsafe_error.to_string().contains("unsafe source path"));

        let mut variables = BTreeMap::from([("mes_libc".to_string(), "mes".to_string())]);
        let condition_error =
            evaluate_configure_assignments("if test $mes_libc = mes; then", &mut variables).unwrap_err();
        assert!(condition_error.to_string().contains("unterminated if block"));
    }
}
