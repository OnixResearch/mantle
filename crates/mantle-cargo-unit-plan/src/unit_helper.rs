//! Static-linkable Rust unit builder. It cannot discover a store or Cargo state:
//! every compiler, source, and direct dependency is supplied by a plan unit.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

const MANIFEST: &str = "share/mantle/unit-dependencies-v1.json";
const MAX_CLOSURE: usize = 4096;

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
        if arg.is_empty() || arg.chars().any(char::is_whitespace) {
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
        .envs(compile_env)
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
    };
    let path = out.join(MANIFEST);
    fs::create_dir_all(path.parent().ok_or("unit-helper-output: invalid manifest directory")?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    fs::write(path, serde_json::to_vec(&result).map_err(|error| format!("unit-helper-output: {error}"))?)
        .map_err(|error| format!("unit-helper-output: {error}"))?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stderr(), "{error}");
        std::process::exit(1);
    }
}
