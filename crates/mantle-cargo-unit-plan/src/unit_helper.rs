//! Static-linkable Rust unit builder. It cannot discover a store or Cargo state:
//! every compiler, source, and direct dependency is supplied by a plan unit.
//!
//! Modes (first argument): `target-facts` records the declared rustc's target
//! cfg; `run-build-script` executes a compiled Cargo build script with only its
//! declared environment and writes `flags`; anything else is a rustc invocation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use mantle_rust_differential_core::waits::MAX_RECORD_BYTES;
use mantle_rust_differential_core::waits::NativeContext;
use mantle_rust_differential_core::waits::NativeEvent;
use mantle_rust_differential_core::waits::NativeFrame;
use mantle_rust_differential_core::waits::RunOutcome;
use mantle_rust_plan_core::BuildScriptFlagsFiles;
use mantle_rust_plan_core::ImmediateArgTarget;
use mantle_rust_plan_core::MAX_BUILD_SCRIPT_OUTPUT_BYTES;
use mantle_rust_plan_core::TARGET_FACTS_FILE;
use mantle_rust_plan_core::check_link_search_roots;
use mantle_rust_plan_core::merge_links_env;
use mantle_rust_plan_core::parse_build_script_output;
use mantle_rust_plan_core::parse_flags_env;
use mantle_rust_plan_core::parse_target_facts;
use mantle_rust_plan_core::parse_transitive_args;
use mantle_rust_plan_core::render_build_script_flags;
use mantle_rust_plan_core::render_target_facts;
use mantle_rust_plan_core::select_immediate_args;
use serde::Deserialize;
use serde::Serialize;

const MANIFEST: &str = "share/mantle/unit-dependencies-v1.json";
const MAX_CLOSURE: usize = 4096;
const SCRIPT_STDERR_MAX_BYTES: usize = 1_048_576;

struct NativeFrameBuffer {
    bytes: [u8; MAX_RECORD_BYTES],
    len: usize,
}

impl core::fmt::Write for NativeFrameBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl NativeFrameBuffer {
    fn encode(frame: &NativeFrame) -> Result<Self, core::fmt::Error> {
        use core::fmt::Write as _;

        let mut output = Self {
            bytes: [0u8; MAX_RECORD_BYTES],
            len: 0,
        };
        // This closed DTO contains only validated lowercase hex digests, u64s,
        // and fixed tags. No field needs escaping; serde_json's std feature is
        // deliberately unnecessary for the lowerer's no_std dependency closure.
        write!(
            output,
            "{{\"invocation\":\"{}\",\"clock\":\"{}\",\"attempt\":\"{}\",\"sequence\":{},\"source_offset_ns\":{},\"event\":",
            frame.invocation.as_str(),
            frame.clock.as_str(),
            frame.attempt.as_str(),
            frame.sequence,
            frame.source_offset_ns,
        )?;
        match &frame.event {
            NativeEvent::Started => output.write_str(r#"{"kind":"started"}"#)?,
            NativeEvent::ErrorObserved => output.write_str(r#"{"kind":"error_observed"}"#)?,
            NativeEvent::Exited { outcome } => {
                let outcome = match outcome {
                    RunOutcome::Success => "success",
                    RunOutcome::Rejected => "rejected",
                    RunOutcome::Crash => "crash",
                    RunOutcome::Cancelled => "cancelled",
                    RunOutcome::Timeout => "timeout",
                    RunOutcome::Infrastructure => "infrastructure",
                    RunOutcome::Unrun => "unrun",
                };
                write!(output, "{{\"kind\":\"exited\",\"outcome\":\"{outcome}\"}}")?;
            }
            NativeEvent::Reaped => output.write_str(r#"{"kind":"reaped"}"#)?,
        }
        output.write_str("}")?;
        Ok(output)
    }
}

/// Execution-only transport: the adapter supplies an already admitted attempt.
/// No unit identity, readiness, or scheduler phase is inferred by this helper.
struct NativeObserver {
    socket: UnixDatagram,
    frame: NativeFrame,
    origin_ns: u64,
    available: bool,
}

impl NativeObserver {
    fn from_environment() -> Option<Self> {
        // Check the opt-in first: the ordinary path allocates no observer state.
        let path = env::var_os("MANTLE_NATIVE_WAIT_SOCKET")?;
        let context = env::var("MANTLE_NATIVE_WAIT_CONTEXT").ok()?;
        if context.len() > MAX_RECORD_BYTES {
            return None;
        }
        let context: NativeContext = serde_json::from_str(&context).ok()?;
        // Reject a foreign/future origin instead of saturating it into a timestamp.
        monotonic_ns()?.checked_sub(context.origin_ns)?;
        let socket = UnixDatagram::unbound().ok()?;
        socket.set_nonblocking(true).ok()?;
        socket.connect(path).ok()?;
        Some(Self {
            socket,
            frame: NativeFrame {
                invocation: context.invocation,
                clock: context.clock,
                attempt: context.attempt,
                sequence: 0,
                source_offset_ns: 0,
                event: NativeEvent::Started,
            },
            origin_ns: context.origin_ns,
            available: true,
        })
    }

    fn emit_at(&mut self, event: NativeEvent, now_ns: Option<u64>) {
        if !self.available {
            return;
        }
        let Some(offset) = now_ns.and_then(|now| now.checked_sub(self.origin_ns)) else {
            self.available = false;
            return;
        };
        // This source emits at most Started, Exited, Reaped, starting at zero.
        if self.frame.sequence > 2 || offset < self.frame.source_offset_ns {
            self.available = false;
            return;
        }
        self.frame.source_offset_ns = offset;
        self.frame.event = event;
        let Ok(encoded) = NativeFrameBuffer::encode(&self.frame) else {
            self.available = false;
            return;
        };
        // One bounded JSON datagram per frame; loss leaves an incomplete prefix.
        // Never retry/block, print, or change compilation on diagnostic failure.
        if self.socket.send(&encoded.bytes[..encoded.len]).ok() != Some(encoded.len) {
            self.available = false;
            return;
        }
        self.frame.sequence += 1;
    }
}

fn monotonic_ns() -> Option<u64> {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: time is writable and CLOCK_MONOTONIC is the adapter's host clock.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
        return None;
    }
    let seconds = u64::try_from(time.tv_sec).ok()?;
    let nanos = u64::try_from(time.tv_nsec).ok()?;
    if nanos >= 1_000_000_000 {
        return None;
    }
    seconds.checked_mul(1_000_000_000)?.checked_add(nanos)
}

fn rustc_status(command: &mut Command) -> io::Result<std::process::ExitStatus> {
    let Some(mut observer) = NativeObserver::from_environment() else {
        return command.status();
    };
    let mut child = command.spawn()?;
    observer.emit_at(NativeEvent::Started, monotonic_ns());
    // Preserve inherited stdin/stdout/stderr, including non-UTF-8 output and
    // ordinary signal behavior. Without a live structured stream there is no
    // ErrorObserved event: completed stderr must never fabricate error timing.
    let status = child.wait()?;
    let outcome = if status.success() {
        RunOutcome::Success
    } else if status.code().is_none() {
        RunOutcome::Crash
    } else {
        RunOutcome::Rejected
    };
    // wait observes both facts together, not an invented exit-to-reap interval.
    let observed_ns = monotonic_ns();
    observer.emit_at(NativeEvent::Exited { outcome }, observed_ns);
    observer.emit_at(NativeEvent::Reaped, observed_ns);
    Ok(status)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DependencyManifest {
    schema: String,
    direct_dependencies: Vec<String>,
    artifacts: Vec<String>,
}

fn required(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("unit-helper-missing-environment: {key}"))
}

fn direct_paths() -> Result<Vec<String>, String> {
    serde_json::from_str(&required("MANTLE_UNIT_DEPENDENCIES")?)
        .map_err(|error| format!("unit-helper-invalid-dependencies: {error}"))
}

fn manifest(path: &str) -> Result<DependencyManifest, String> {
    let file = Path::new(path).join(MANIFEST);
    let bytes = fs::read(&file).map_err(|error| format!("unit-helper-missing-manifest: {path}: {error}"))?;
    let parsed: DependencyManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("unit-helper-invalid-manifest: {path}: {error}"))?;
    if parsed.schema != "mantle-unit-dependencies-v1" {
        return Err(format!("unit-helper-invalid-manifest: {path}: unknown schema"));
    }
    Ok(parsed)
}

fn search_closure(direct: &[String], prefix: &str) -> Result<BTreeSet<String>, String> {
    let mut pending = direct.to_vec();
    let store_prefix = format!("{prefix}/");
    let mut visited = BTreeSet::new();
    while let Some(path) = pending.pop() {
        if !path.starts_with(&store_prefix) || !visited.insert(path.clone()) {
            if visited.contains(&path) {
                continue;
            }
            return Err(format!("unit-helper-invalid-manifest: undeclared path {path}"));
        }
        if visited.len() > MAX_CLOSURE {
            return Err("unit-helper-closure-limit: more than 4096 dependencies".into());
        }
        let found = manifest(&path)?;
        for ancestor in found.direct_dependencies {
            if !visited.contains(&ancestor) {
                pending.push(ancestor);
            }
        }
    }
    Ok(visited)
}

fn extern_artifact(output: &str) -> Result<PathBuf, String> {
    let found = manifest(output)?;
    let mut artifacts = found
        .artifacts
        .into_iter()
        .filter(|path| path.ends_with(".rlib") || path.ends_with(".so") || path.ends_with(".dylib"))
        .collect::<Vec<_>>();
    artifacts.sort();
    artifacts.dedup();
    if artifacts.len() != 1 || !artifacts[0].starts_with("lib/") || artifacts[0].contains("..") {
        return Err(format!("unit-helper-invalid-manifest: {output}: expected one linkable library"));
    }
    let path = Path::new(output).join(&artifacts[0]);
    if !path.is_file() {
        return Err(format!("unit-helper-missing-artifact: {}", path.display()));
    }
    Ok(path)
}

fn write_response_file(path: &Path, args: &[String]) -> Result<(), String> {
    let mut file = fs::File::create(path).map_err(|error| format!("unit-helper-argument-file: {error}"))?;
    for arg in args {
        // rustc reads one argument per line, so only line breaks and NUL are unsafe.
        if arg.is_empty() || arg.contains(['\n', '\r', '\0']) {
            return Err(format!("unit-helper-argument-unsupported: {arg}"));
        }
        writeln!(file, "{arg}").map_err(|error| format!("unit-helper-argument-file: {error}"))?;
    }
    file.sync_all().map_err(|error| format!("unit-helper-argument-file: {error}"))
}

fn artifacts(output: &Path) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    for entry in fs::read_dir(output.join("lib")).map_err(|error| format!("unit-helper-output: {error}"))? {
        let entry = entry.map_err(|error| format!("unit-helper-output: {error}"))?;
        if entry.file_type().map_err(|error| format!("unit-helper-output: {error}"))?.is_file() {
            let file = entry
                .file_name()
                .into_string()
                .map_err(|_| "unit-helper-output: non-UTF-8 artifact name".to_string())?;
            result.push(format!("lib/{file}"));
        }
    }
    result.sort();
    if result.is_empty() {
        return Err("unit-helper-output: rustc produced no artifacts".into());
    }
    Ok(result)
}

fn run_rustc() -> Result<(), String> {
    let out = PathBuf::from(required("out")?);
    let rustc = required("MANTLE_UNIT_RUSTC")?;
    let linker = required("MANTLE_UNIT_LINKER")?;
    let source = required("MANTLE_UNIT_SOURCE")?;
    let prefix = Path::new(&source)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| "unit-helper-invalid-source: expected store source root".to_string())?;
    let direct = direct_paths()?;
    let externs: BTreeMap<String, String> = serde_json::from_str(&required("MANTLE_UNIT_EXTERNS")?)
        .map_err(|error| format!("unit-helper-invalid-externs: {error}"))?;
    let compile_env: BTreeMap<String, String> = serde_json::from_str(&required("MANTLE_UNIT_COMPILE_ENV")?)
        .map_err(|error| format!("unit-helper-invalid-compile-env: {error}"))?;
    for (key, value) in &compile_env {
        if !key.starts_with("CARGO_PKG_") && !key.starts_with("CARGO_FEATURE_") && key != "CARGO_MANIFEST_DIR" {
            return Err(format!("unit-helper-invalid-compile-env: {key}"));
        }
        if key.contains('=') || key.contains('\0') || value.contains('\0') {
            return Err(format!("unit-helper-invalid-compile-env: {key}"));
        }
    }
    let closure = search_closure(&direct, prefix)?;
    let mut arguments = env::args().skip(1).collect::<Vec<_>>();
    arguments.push("-C".into());
    arguments.push(format!("linker={linker}"));
    if let Ok(value) = env::var("MANTLE_UNIT_LINK_ARGS") {
        let linker_args: Vec<String> =
            serde_json::from_str(&value).map_err(|error| format!("unit-helper-invalid-linker-arguments: {error}"))?;
        if linker_args.len() > 256
            || linker_args.iter().any(|arg| arg.is_empty() || arg.len() > 16_384 || arg.contains('\0'))
        {
            return Err("unit-helper-invalid-linker-arguments: argument bounds".into());
        }
        for argument in linker_args {
            arguments.push("-C".into());
            arguments.push(format!("link-arg={argument}"));
        }
    }
    for path in &closure {
        arguments.push(format!("-Ldependency={path}/lib"));
    }
    for (name, path) in externs {
        if !direct.contains(&path)
            || name.is_empty()
            || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(format!("unit-helper-invalid-externs: {name}"));
        }
        let artifact = extern_artifact(&path)?;
        arguments.push("--extern".into());
        arguments.push(format!("{name}={}", artifact.display()));
    }
    let (script_arguments, script_env) = build_script_flags(&arguments, prefix)?;
    arguments.extend(script_arguments);
    if !arguments.iter().any(|arg| arg.starts_with(&source)) {
        return Err("unit-helper-source-absent: no declared source argument".into());
    }
    fs::create_dir_all(out.join("lib")).map_err(|error| format!("unit-helper-output: {error}"))?;
    arguments.push("--out-dir".into());
    arguments.push(out.join("lib").to_string_lossy().into_owned());
    let response = out.join("rustc-arguments.rsp");
    write_response_file(&response, &arguments)?;
    let mut command = Command::new(&rustc);
    command
        .arg(format!("@{}", response.display()))
        .env_clear()
        .env("HOME", out.as_os_str())
        .env("TMPDIR", out.as_os_str())
        .envs(compile_env)
        .envs(script_env);
    let status = rustc_status(&mut command).map_err(|error| format!("unit-helper-rustc-failed: {error}"))?;
    if !status.success() {
        return Err(format!("unit-helper-rustc-failed: {status}"));
    }
    fs::remove_file(&response).map_err(|error| format!("unit-helper-argument-file: {error}"))?;
    let result = DependencyManifest {
        schema: "mantle-unit-dependencies-v1".into(),
        direct_dependencies: direct,
        artifacts: artifacts(&out)?,
    };
    let path = out.join(MANIFEST);
    fs::create_dir_all(path.parent().ok_or("unit-helper-output: invalid manifest directory")?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    fs::write(path, serde_json::to_vec(&result).map_err(|error| format!("unit-helper-output: {error}"))?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    Ok(())
}

fn json_env<T: serde::de::DeserializeOwned>(key: &str) -> Result<T, String> {
    serde_json::from_str(&required(key)?).map_err(|error| format!("unit-helper-invalid-environment: {key}: {error}"))
}

/// Read one `flags` file of a declared unit output under the store prefix.
fn read_flags_file(output: &str, prefix: &str, file: &str) -> Result<String, String> {
    if !output.starts_with(&format!("{prefix}/")) {
        return Err(format!("unit-helper-invalid-manifest: undeclared path {output}"));
    }
    let path = Path::new(output).join(file);
    let metadata =
        fs::metadata(&path).map_err(|error| format!("unit-helper-missing-flags: {}: {error}", path.display()))?;
    if metadata.len() > MAX_BUILD_SCRIPT_OUTPUT_BYTES as u64 {
        return Err(format!("build-script-output-limit: {}", path.display()));
    }
    fs::read_to_string(&path).map_err(|error| format!("unit-helper-missing-flags: {}: {error}", path.display()))
}

/// Extra rustc arguments and compile environment derived from `flags`.
type ScriptFlags = (Vec<String>, Vec<(String, String)>);

/// rustc arguments and environment from build-script `flags`: the immediate
/// arguments and `rustc-env` of the package's own script, `OUT_DIR`, and the
/// `-L` search paths of every transitively reachable script.
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
fn build_script_flags(arguments: &[String], prefix: &str) -> Result<ScriptFlags, String> {
    let mut extra = Vec::new();
    let mut extra_env = Vec::new();
    if let Ok(flags) = env::var("MANTLE_UNIT_BUILD_SCRIPT_FLAGS") {
        let out = required("MANTLE_UNIT_BUILD_SCRIPT_OUT")?;
        let kind = required("MANTLE_UNIT_TARGET_KIND")?;
        let name = required("MANTLE_UNIT_TARGET_NAME")?;
        let is_cdylib = arguments
            .windows(2)
            .any(|pair| pair[0] == "--crate-type" && pair[1].split(',').any(|ty| ty == "cdylib"))
            || arguments.iter().any(|arg| {
                arg.strip_prefix("--crate-type=").is_some_and(|types| types.split(',').any(|ty| ty == "cdylib"))
            });
        let target = ImmediateArgTarget {
            kind: &kind,
            name: &name,
            is_cdylib,
        };
        let immediate = read_flags_file(&flags, prefix, "args-immediate")?;
        extra.extend(select_immediate_args(&immediate, &target).map_err(|error| error.to_string())?);
        extra_env.extend(parse_flags_env(&read_flags_file(&flags, prefix, "env")?).map_err(|error| error.to_string())?);
        if !out.starts_with(&format!("{prefix}/")) {
            return Err(format!("unit-helper-invalid-manifest: undeclared path {out}"));
        }
        extra_env.push(("OUT_DIR".to_string(), out));
    }
    if env::var_os("MANTLE_UNIT_TRANSITIVE_FLAGS").is_some() {
        let transitive: Vec<String> = json_env("MANTLE_UNIT_TRANSITIVE_FLAGS")?;
        for flags in &transitive {
            let file = read_flags_file(flags, prefix, "args-transitive")?;
            extra.extend(parse_transitive_args(&file).map_err(|error| error.to_string())?);
        }
    }
    Ok((extra, extra_env))
}

fn rustc_stdout(rustc: &str, arguments: &[&str], scratch: &Path) -> Result<String, String> {
    let output = Command::new(rustc)
        .args(arguments)
        .env_clear()
        .env("HOME", scratch)
        .env("TMPDIR", scratch)
        .output()
        .map_err(|error| format!("unit-helper-rustc-failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "unit-helper-rustc-failed: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| "unit-helper-rustc-failed: non-UTF-8 output".to_string())
}

/// `target-facts`: one typed environment file per toolchain and triple.
// r[impl mantle.rust_unit_plan.build_scripts.target_facts]
fn run_target_facts() -> Result<(), String> {
    let out = PathBuf::from(required("out")?);
    let rustc = required("MANTLE_UNIT_RUSTC")?;
    let target = required("MANTLE_UNIT_TARGET")?;
    let host = required("MANTLE_UNIT_HOST")?;
    fs::create_dir_all(&out).map_err(|error| format!("unit-helper-output: {error}"))?;
    let version = rustc_stdout(&rustc, &["-vV"], &out)?;
    let observed = version.lines().find_map(|line| line.strip_prefix("host: ")).unwrap_or_default();
    if observed != host {
        return Err(format!("unit-helper-host-mismatch: declared {host}, rustc reports {observed}"));
    }
    let cfg = rustc_stdout(&rustc, &["--print", "cfg", "--target", &target], &out)?;
    let facts = render_target_facts(&cfg, &target, &host).map_err(|error| error.to_string())?;
    fs::write(out.join(TARGET_FACTS_FILE), facts).map_err(|error| format!("unit-helper-output: {error}"))
}

/// The compiled script is the single artifact recorded by its compile unit.
fn script_executable(unit: &str) -> Result<PathBuf, String> {
    let found = manifest(unit)?;
    let executables = found
        .artifacts
        .iter()
        .filter(|artifact| !artifact.ends_with(".d") && !artifact.ends_with(".rmeta"))
        .collect::<Vec<_>>();
    let [artifact] = executables.as_slice() else {
        return Err(format!("unit-helper-invalid-manifest: {unit}: expected one build-script executable"));
    };
    if !artifact.starts_with("lib/") || artifact.contains("..") {
        return Err(format!("unit-helper-invalid-manifest: {unit}: {artifact}"));
    }
    let path = Path::new(unit).join(artifact);
    if !path.is_file() {
        return Err(format!("unit-helper-missing-artifact: {}", path.display()));
    }
    Ok(path)
}

struct ScriptRun {
    script: PathBuf,
    source: String,
    out: PathBuf,
    env: Vec<(String, String)>,
    path: Vec<String>,
}

/// Environment of a build script: declared package facts and native values,
/// target facts, `DEP_*` from `links` dependencies, and `OUT_DIR`. Nothing is
/// inherited from the helper's own environment.
fn script_environment(out: &Path, prefix: &str) -> Result<Vec<(String, String)>, String> {
    let declared: BTreeMap<String, String> = json_env("MANTLE_UNIT_SCRIPT_ENV")?;
    let lane_owned = |key: &str| {
        matches!(key, "OUT_DIR" | "TARGET" | "HOST" | "PATH")
            || key.starts_with("DEP_")
            || key.starts_with("CARGO_CFG_")
    };
    if let Some(key) = declared.keys().find(|key| lane_owned(key.as_str()) || key.contains('=')) {
        return Err(format!("unit-helper-invalid-environment: {key}"));
    }
    let facts_unit = required("MANTLE_UNIT_TARGET_FACTS")?;
    let facts = parse_target_facts(&read_flags_file(&facts_unit, prefix, TARGET_FACTS_FILE)?)
        .map_err(|error| error.to_string())?;
    let links_units: Vec<String> = json_env("MANTLE_UNIT_LINKS_DEPENDENCIES")?;
    let mut links_files = Vec::with_capacity(links_units.len());
    for unit in &links_units {
        links_files.push(read_flags_file(unit, prefix, "links")?);
    }
    let links = merge_links_env(&links_files.iter().map(String::as_str).collect::<Vec<_>>())
        .map_err(|error| error.to_string())?;
    let mut env = declared.into_iter().collect::<Vec<_>>();
    env.extend(facts);
    env.extend(links);
    env.push(("OUT_DIR".to_string(), out.to_string_lossy().into_owned()));
    env.push(("HOME".to_string(), out.to_string_lossy().into_owned()));
    env.push(("TMPDIR".to_string(), out.to_string_lossy().into_owned()));
    Ok(env)
}

fn write_flags(flags: &Path, files: &BuildScriptFlagsFiles) -> Result<(), String> {
    fs::create_dir_all(flags).map_err(|error| format!("unit-helper-output: {error}"))?;
    for (name, contents) in [
        ("args-immediate", &files.args_immediate),
        ("args-transitive", &files.args_transitive),
        ("env", &files.env),
        ("links", &files.links),
        ("rerun", &files.rerun),
        ("warnings", &files.warnings),
    ] {
        fs::write(flags.join(name), contents).map_err(|error| format!("unit-helper-output: {error}"))?;
    }
    Ok(())
}

fn read_script_stream(reader: impl Read, limit: usize, name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let read_limit = u64::try_from(limit).expect("script output limit fits u64").saturating_add(1);
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("build-script-output-limit: reading {name}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("build-script-output-limit: {name} exceeds {limit} bytes"));
    }
    Ok(bytes)
}

fn capture_script(run: &ScriptRun) -> Result<std::process::Output, String> {
    let mut command = Command::new(&run.script);
    command.env_clear().current_dir(&run.source).envs(run.env.iter().map(|(key, value)| (key, value)));
    command.env("PATH", run.path.join(":"));
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| format!("build-script-nonzero-exit: spawn failed: {error}"))?;
    let stdout = child.stdout.take().expect("piped script stdout");
    let stderr = child.stderr.take().expect("piped script stderr");
    let (failure_tx, failure_rx) = std::sync::mpsc::channel::<()>();
    std::thread::scope(|scope| {
        let stdout_tx = failure_tx.clone();
        let stdout_reader = scope.spawn(move || {
            let result = read_script_stream(stdout, MAX_BUILD_SCRIPT_OUTPUT_BYTES, "stdout");
            if result.is_err() {
                let _ = stdout_tx.send(());
            }
            result
        });
        let stderr_reader = scope.spawn(move || {
            let result = read_script_stream(stderr, SCRIPT_STDERR_MAX_BYTES, "stderr");
            if result.is_err() {
                let _ = failure_tx.send(());
            }
            result
        });
        let status = loop {
            if failure_rx.try_recv().is_ok() {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("build-script-nonzero-exit: waiting for script: {error}"));
                }
            }
        };
        let stdout = stdout_reader.join().map_err(|_| "build-script-output-limit: stdout reader panicked")??;
        let stderr = stderr_reader.join().map_err(|_| "build-script-output-limit: stderr reader panicked")??;
        let status = status.ok_or("build-script-output-limit: child stopped without stream diagnostic")?;
        Ok(std::process::Output { status, stdout, stderr })
    })
}

fn execute_script(run: &ScriptRun) -> Result<String, String> {
    let result = capture_script(run)?;
    let _ = io::stderr().write_all(&result.stderr);
    if !result.status.success() {
        return Err(format!("build-script-nonzero-exit: {}", result.status));
    }
    String::from_utf8(result.stdout).map_err(|error| format!("build-script-directive-unsupported: {error}"))
}

/// `run-build-script`: execute the compiled script in its source slice with
/// only the declared environment, then parse and record its directives.
// r[impl mantle.rust_unit_plan.build_scripts.run_units]
// r[impl mantle.rust_unit_plan.build_scripts.sandbox_boundary]
fn run_build_script() -> Result<(), String> {
    let out = PathBuf::from(required("out")?);
    let flags = PathBuf::from(required("flags")?);
    let source = required("MANTLE_UNIT_SOURCE")?;
    let prefix = Path::new(&source)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| "unit-helper-invalid-source: expected store source root".to_string())?
        .to_string();
    let script_unit = required("MANTLE_UNIT_BUILD_SCRIPT")?;
    if !script_unit.starts_with(&format!("{prefix}/")) {
        return Err(format!("unit-helper-invalid-manifest: undeclared path {script_unit}"));
    }
    let native: Vec<String> = json_env("MANTLE_UNIT_NATIVE_INPUTS")?;
    let mut path: Vec<String> = json_env("MANTLE_UNIT_SCRIPT_PATH")?;
    if let Some(entry) = path.iter().find(|entry| {
        !native.iter().any(|root| {
            entry.strip_prefix(root.as_str()).is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
        }) || entry.contains(':')
    }) {
        return Err(format!("unit-helper-invalid-environment: PATH entry {entry} is not a native input"));
    }
    let rustc = required("MANTLE_UNIT_RUSTC")?;
    let compiler_bin = Path::new(&rustc)
        .parent()
        .filter(|path| path.is_absolute() && !rustc.contains(['\0', ':']))
        .ok_or_else(|| "unit-helper-invalid-environment: rustc must be an absolute path".to_string())?;
    path.insert(0, compiler_bin.to_string_lossy().into_owned());
    fs::create_dir_all(&out).map_err(|error| format!("unit-helper-output: {error}"))?;
    let run = ScriptRun {
        script: script_executable(&script_unit)?,
        env: script_environment(&out, &prefix)?,
        source,
        out,
        path,
    };
    let stdout = execute_script(&run)?;
    let directives = parse_build_script_output(&stdout).map_err(|error| error.to_string())?;
    let out_root = run.out.to_string_lossy();
    let mut roots = vec![out_root.as_ref(), run.source.as_str()];
    roots.extend(native.iter().map(String::as_str));
    check_link_search_roots(&directives, &roots).map_err(|error| error.to_string())?;
    let links = run.env.iter().find(|(key, _)| key == "CARGO_MANIFEST_LINKS").map(|(_, value)| value.as_str());
    let files = render_build_script_flags(&directives, links);
    for warning in &directives.warnings {
        let _ = writeln!(io::stderr(), "warning: {warning}");
    }
    write_flags(&flags, &files)
}

fn main() {
    let result = match env::args().nth(1).as_deref() {
        Some("target-facts") => run_target_facts(),
        Some("run-build-script") => run_build_script(),
        _ => run_rustc(),
    };
    if let Err(error) = result {
        let _ = writeln!(io::stderr(), "{error}");
        std::process::exit(1);
    }
}
