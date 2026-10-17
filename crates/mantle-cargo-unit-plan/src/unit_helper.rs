//! Static-linkable Rust unit builder. It cannot discover a store or Cargo state:
//! every compiler, source, and direct dependency is supplied by a plan unit.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use mantle_cargo_unit_plan::BuildScriptFlags;
use mantle_cargo_unit_plan::NativeInput;
use mantle_cargo_unit_plan::parse_directives;
use serde::Deserialize;
use serde::Serialize;

const MANIFEST: &str = "share/mantle/unit-dependencies-v1.json";
const MAX_CLOSURE: usize = 4096;
const SCRIPT_STDOUT_MAX_BYTES: usize = 4 * 1024 * 1024;
const SCRIPT_STDERR_MAX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DependencyManifest {
    schema: String,
    direct_dependencies: Vec<String>,
    artifacts: Vec<String>,
    #[serde(default)]
    flags: Option<String>,
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
        if arg.is_empty() || arg.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r')) {
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

fn read_flags(path: &str) -> Result<BuildScriptFlags, String> {
    let bytes = fs::read(Path::new(path).join("flags.json"))
        .map_err(|error| format!("unit-helper-missing-flags: {path}: {error}"))?;
    let parsed: BuildScriptFlags =
        serde_json::from_slice(&bytes).map_err(|error| format!("unit-helper-invalid-flags: {path}: {error}"))?;
    if parsed.schema != "mantle-build-script-flags-v1" {
        return Err(format!("unit-helper-invalid-flags: {path}: unknown schema"));
    }

    Ok(parsed)
}
fn scoped_link_args(arguments: &mut Vec<String>, flags: &BuildScriptFlags) {
    let kind = arguments
        .iter()
        .find_map(|word| word.strip_prefix("--crate-type="))
        .or_else(|| arguments.windows(2).find(|pair| pair[0] == "--crate-type").map(|pair| pair[1].as_str()));
    let name = arguments
        .iter()
        .find_map(|word| word.strip_prefix("--crate-name="))
        .or_else(|| arguments.windows(2).find(|pair| pair[0] == "--crate-name").map(|pair| pair[1].as_str()));
    let selected = flags
        .scoped_link_args
        .iter()
        .filter(|arg| match arg.kind.as_str() {
            "rustc-link-arg-bin" => kind == Some("bin") && arg.target.as_deref() == name,
            "rustc-link-arg-bins" => kind == Some("bin"),
            "rustc-link-arg-cdylib" => kind == Some("cdylib"),
            "rustc-link-arg-tests" => kind == Some("test"),
            "rustc-link-arg-examples" => kind == Some("example"),
            "rustc-link-arg-benches" => kind == Some("bench"),
            _ => false,
        })
        .map(|arg| arg.argument.clone())
        .collect::<Vec<_>>();
    for argument in selected {
        arguments.push("-C".into());
        arguments.push(format!("link-arg={argument}"));
    }
}

fn write_manifest(out: &Path, result: &DependencyManifest) -> Result<(), String> {
    let path = out.join(MANIFEST);
    fs::create_dir_all(path.parent().ok_or("unit-helper-output: invalid manifest directory")?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    fs::write(path, serde_json::to_vec(result).map_err(|error| format!("unit-helper-output: {error}"))?)
        .map_err(|error| format!("unit-helper-output: {error}"))
}

// r[impl mantle.rust_unit_plan.build_scripts.target_facts]
fn target_facts() -> Result<(), String> {
    let out = PathBuf::from(required("out")?);
    let rustc = required("MANTLE_UNIT_RUSTC")?;
    let target = required("MANTLE_UNIT_TARGET")?;
    let host = required("MANTLE_UNIT_HOST")?;
    let cfg = Command::new(&rustc)
        .args(["--print=cfg", "--target", &target])
        .env_clear()
        .output()
        .map_err(|error| format!("unit-helper-target-facts: {error}"))?;
    let version = Command::new(&rustc)
        .arg("-vV")
        .env_clear()
        .output()
        .map_err(|error| format!("unit-helper-target-facts: {error}"))?;
    if !cfg.status.success()
        || !version.status.success()
        || !String::from_utf8_lossy(&version.stdout).lines().any(|line| line == format!("host: {host}"))
    {
        return Err("unit-helper-target-facts: compiler host or cfg query failed".into());
    }
    let mut facts = BTreeMap::from([("TARGET".to_string(), target), ("HOST".to_string(), host)]);
    let stdout = String::from_utf8(cfg.stdout).map_err(|error| format!("unit-helper-target-facts: {error}"))?;
    for line in stdout.lines() {
        let (key, value) = line.split_once('=').map_or((line, ""), |(key, value)| (key, value.trim_matches('\"')));
        if !key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err("unit-helper-target-facts: invalid cfg key".into());
        }
        let name = format!("CARGO_CFG_{}", key.to_ascii_uppercase());
        facts
            .entry(name)
            .and_modify(|old| {
                old.push(',');
                old.push_str(value);
            })
            .or_insert_with(|| value.to_string());
    }
    fs::create_dir_all(&out).map_err(|error| format!("unit-helper-target-facts: {error}"))?;
    fs::write(out.join("target-facts.json"), serde_json::to_vec(&facts).map_err(|error| error.to_string())?)
        .map_err(|error| format!("unit-helper-target-facts: {error}"))
}

fn read_script_stream(reader: impl Read, limit: usize, name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("build-script-output-limit: reading {name}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("build-script-output-limit: {name} exceeds {limit} bytes"));
    }
    Ok(bytes)
}

fn capture_script(
    program: &Path,
    source: &Path,
    environment: BTreeMap<String, String>,
) -> Result<std::process::Output, String> {
    let mut child = Command::new(program)
        .env_clear()
        .envs(environment)
        .current_dir(source)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("build-script-nonzero-exit: {error}"))?;
    let stdout = child.stdout.take().expect("piped script stdout");
    let stderr = child.stderr.take().expect("piped script stderr");
    let (failure_tx, failure_rx) = std::sync::mpsc::channel::<()>();
    std::thread::scope(|scope| {
        let stdout_tx = failure_tx.clone();
        let stdout_reader = scope.spawn(move || {
            let result = read_script_stream(stdout, SCRIPT_STDOUT_MAX_BYTES, "stdout");
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
        let stdout = stdout_reader
            .join()
            .map_err(|_| "build-script-output-limit: stdout reader panicked".to_string())??;
        let stderr = stderr_reader
            .join()
            .map_err(|_| "build-script-output-limit: stderr reader panicked".to_string())??;
        let status = status.ok_or("build-script-output-limit: child stopped without stream diagnostic")?;
        Ok(std::process::Output { status, stdout, stderr })
    })
}

// r[impl mantle.rust_unit_plan.build_scripts.run_units]
fn run_build_script() -> Result<(), String> {
    let out = PathBuf::from(required("out")?);
    let flags_dir = PathBuf::from(required("flags")?);
    let script_out = required("MANTLE_UNIT_SCRIPT")?;
    let source = required("MANTLE_UNIT_SOURCE")?;
    let manifest = manifest(&script_out)?;
    let program = manifest
        .artifacts
        .iter()
        .filter(|path| {
            path.starts_with("lib/")
                && !path.ends_with(".rlib")
                && !path.ends_with(".rmeta")
                && !path.ends_with(".so")
                && !path.ends_with(".dylib")
        })
        .collect::<Vec<_>>();
    if program.len() != 1 {
        return Err("unit-helper-invalid-script: expected one compiled executable".into());
    }
    let program = Path::new(&script_out).join(program[0]);
    let facts_path = required("MANTLE_UNIT_TARGET_FACTS")?;
    let mut environment: BTreeMap<String, String> = serde_json::from_slice(
        &fs::read(Path::new(&facts_path).join("target-facts.json"))
            .map_err(|error| format!("unit-helper-target-facts: {error}"))?,
    )
    .map_err(|error| format!("unit-helper-target-facts: {error}"))?;
    let package: BTreeMap<String, String> = serde_json::from_str(&required("MANTLE_UNIT_COMPILE_ENV")?)
        .map_err(|error| format!("unit-helper-invalid-compile-env: {error}"))?;
    for (key, value) in package {
        if !key.starts_with("CARGO_PKG_")
            && !key.starts_with("CARGO_FEATURE_")
            && key != "CARGO_MANIFEST_DIR"
            && key != "PROFILE"
            && key != "OPT_LEVEL"
            && key != "DEBUG"
        {
            return Err(format!("unit-helper-invalid-compile-env: {key}"));
        }
        environment.insert(key, value);
    }
    for key in ["PROFILE", "OPT_LEVEL", "DEBUG"] {
        if !environment.contains_key(key) {
            return Err(format!("unit-helper-invalid-compile-env: missing {key}"));
        }
    }
    let native: NativeInput = serde_json::from_str(&required("MANTLE_UNIT_NATIVE")?)
        .map_err(|error| format!("unit-helper-invalid-native: {error}"))?;
    for (key, value) in native.env {
        if matches!(key.as_str(), "OUT_DIR" | "PATH" | "PROFILE" | "OPT_LEVEL" | "DEBUG" | "RUSTC" | "HOST")
            || key.starts_with("MANTLE_")
            || key.starts_with("CARGO_")
            || key.starts_with("DEP_")
            || key.contains('=')
        {
            return Err(format!("unit-helper-invalid-native: {key}"));
        }
        environment.insert(key, value);
    }
    let linked: BTreeMap<String, String> = serde_json::from_str(&required("MANTLE_UNIT_LINKED_FLAGS")?)
        .map_err(|error| format!("unit-helper-invalid-links: {error}"))?;
    for (links, flags) in linked {
        for (key, value) in read_flags(&flags)?.links {
            environment.insert(
                format!("DEP_{}_{}", links.replace('-', "_").to_ascii_uppercase(), key.to_ascii_uppercase()),
                value,
            );
        }
    }
    fs::create_dir_all(&out).map_err(|error| format!("unit-helper-output: {error}"))?;
    fs::create_dir_all(&flags_dir).map_err(|error| format!("unit-helper-output: {error}"))?;
    environment.insert("OUT_DIR".into(), out.display().to_string());
    environment.insert("RUSTC".into(), required("MANTLE_UNIT_RUSTC")?);
    environment
        .insert("HOST".into(), environment.get("HOST").cloned().ok_or("unit-helper-target-facts: missing HOST")?);
    environment.insert("NUM_JOBS".into(), "1".into());
    environment.insert("HOME".into(), out.display().to_string());
    environment.insert("TMPDIR".into(), out.display().to_string());
    let path = std::iter::once(format!("{}/bin", required("MANTLE_UNIT_TOOLCHAIN")?))
        .chain(native.path)
        .collect::<Vec<_>>()
        .join(":");
    environment.insert("PATH".into(), path);
    let result = capture_script(&program, Path::new(&source), environment)?;
    if !result.status.success() {
        return Err(format!(
            "build-script-nonzero-exit: {}: {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let stdout =
        String::from_utf8(result.stdout).map_err(|error| format!("build-script-directive-unsupported: {error}"))?;
    let mut allowed = native.inputs.into_values().collect::<Vec<_>>();
    allowed.push(out.display().to_string());
    let flags = parse_directives(&stdout, &allowed)
        .map_err(|blocker| format!("{}: {}: {}", blocker.code, blocker.subject, blocker.detail))?;
    if !flags.links.is_empty() && required("MANTLE_UNIT_LINKS_KEY")?.is_empty() {
        return Err("build-script-links-conflict: metadata requires a declared package links key".into());
    }
    fs::write(flags_dir.join("flags.json"), serde_json::to_vec(&flags).map_err(|error| error.to_string())?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    for (name, bytes) in [
        ("args-immediate", flags.args_immediate.join("\n")),
        ("args-transitive", flags.args_transitive.join("\n")),
        ("env", serde_json::to_string(&flags.env).map_err(|error| error.to_string())?),
        ("links", serde_json::to_string(&flags.links).map_err(|error| error.to_string())?),
        ("rerun", flags.rerun.join("\n")),
    ] {
        fs::write(flags_dir.join(name), bytes).map_err(|error| format!("unit-helper-output: {error}"))?;
    }
    let dependencies: Vec<String> = direct_paths()?;
    write_manifest(&out, &DependencyManifest {
        schema: "mantle-unit-dependencies-v1".into(),
        direct_dependencies: dependencies,
        artifacts: Vec::new(),
        flags: Some(flags_dir.display().to_string()),
    })
}

fn run() -> Result<(), String> {
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
    let mut environment = compile_env;
    let own_flags = env::var("MANTLE_UNIT_FLAGS").ok();
    if let Some(path) = &own_flags {
        let flags = read_flags(path)?;
        scoped_link_args(&mut arguments, &flags);
        arguments.extend(flags.args_immediate);
        environment.extend(flags.env);
        environment.insert("OUT_DIR".into(), required("MANTLE_UNIT_BUILD_OUT")?);
    }
    let links_binary = arguments
        .iter()
        .any(|arg| arg == "--crate-type=bin" || arg == "--crate-type=cdylib" || arg == "bin" || arg == "cdylib");
    if links_binary {
        let mut seen = BTreeSet::new();
        for path in &closure {
            if let Some(flags) = manifest(path)?.flags
                && own_flags.as_deref() != Some(flags.as_str())
                && seen.insert(flags.clone())
            {
                let flags = read_flags(&flags)?;
                scoped_link_args(&mut arguments, &flags);
                arguments.extend(flags.args_transitive);
            }
        }
    }
    if !arguments.iter().any(|arg| arg.starts_with(&source)) {
        return Err("unit-helper-source-absent: no declared source argument".into());
    }
    fs::create_dir_all(out.join("lib")).map_err(|error| format!("unit-helper-output: {error}"))?;
    arguments.push("--out-dir".into());
    arguments.push(out.join("lib").to_string_lossy().into_owned());
    let response = out.join("rustc-arguments.rsp");
    write_response_file(&response, &arguments)?;
    let status = Command::new(&rustc)
        .arg(format!("@{}", response.display()))
        .env_clear()
        .env("HOME", out.as_os_str())
        .env("TMPDIR", out.as_os_str())
        .envs(environment)
        .status()
        .map_err(|error| format!("unit-helper-rustc-failed: {error}"))?;
    if !status.success() {
        return Err(format!("unit-helper-rustc-failed: {status}"));
    }
    fs::remove_file(&response).map_err(|error| format!("unit-helper-argument-file: {error}"))?;
    let result = DependencyManifest {
        schema: "mantle-unit-dependencies-v1".into(),
        direct_dependencies: direct,
        artifacts: artifacts(&out)?,
        flags: own_flags,
    };
    write_manifest(&out, &result)?;
    Ok(())
}

fn main() {
    let mode = env::args().nth(1);
    let result = match mode.as_deref() {
        Some("target-facts") => target_facts(),
        Some("run-build-script") => run_build_script(),
        _ => run(),
    };
    if let Err(error) = result {
        let _ = writeln!(io::stderr(), "{error}");
        std::process::exit(1);
    }
}
