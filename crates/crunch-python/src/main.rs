#![feature(register_tool)]
#![register_tool(tigerstyle)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_python_core::Artifact;
use crunch_python_core::MAX_CONFIG_BYTES;
use crunch_python_core::MAX_INPUT_BYTES;
use crunch_python_core::Target;
use crunch_python_core::admit_artifacts;
use crunch_python_core::admit_selections;
use crunch_python_core::admit_target;
use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::resolve_native_inputs;
use serde::Deserialize;
use serde::Serialize;

const DRIVER: &str = include_str!("driver.py");
const MAX_TREE_ENTRIES: u32 = 16384;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema: u32,
    mode: String,
    store_prefix: String,
    python: String,
    uv: String,
    project: String,
    target: Target,
    extras: Vec<String>,
    groups: Vec<String>,
    members: Vec<String>,
    artifacts: Vec<Artifact>,
    packaging: String,
    build_requirements: Vec<String>,
    dynamic_requirements: Vec<String>,
    build_packages: Vec<String>,
    source_date_epoch: u64,
    native_inputs: NativeInputDeclaration,
    runtime_inputs: Vec<String>,
    /// Immutable Git snapshots, already admitted through the generic Git transport.
    git_sources: BTreeMap<String, GitSource>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GitSource {
    revision: String,
    path: String,
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("python-adapter: {message}");
    std::process::exit(1)
}

fn store_root<'a>(path: &'a str, prefix: &str) -> Result<&'a str, String> {
    let rest = path
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(|| format!("python-host-path: {path}"))?;
    let root = rest.split('/').next().unwrap_or_default();
    let valid_hash = root
        .as_bytes()
        .get(..32)
        .is_some_and(|hash| hash.iter().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(byte)));
    if !valid_hash
        || root.as_bytes().get(32) != Some(&b'-')
        || root.len() < 34
        || path.split('/').any(|part| part == ".." || part == ".")
    {
        return Err(format!("python-store-path-invalid: {path}"));
    }
    Ok(&path[..prefix.len() + 1 + root.len()])
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(format!("python-input-size-or-kind: {}", path.display()));
    }
    fs::read(path).map_err(|error| error.to_string())
}

fn hash_file(path: &Path, hasher: &mut blake3::Hasher) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err(format!("python-input-size-or-kind: {}", path.display()));
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    hasher.update_reader(file).map_err(|error| error.to_string())?;
    Ok(())
}

fn hash_tree(path: &Path, relative: &Path, hasher: &mut blake3::Hasher, entries: &mut u32) -> Result<(), String> {
    *entries = entries.checked_add(1).ok_or("python-tree-limit")?;
    if *entries > MAX_TREE_ENTRIES || relative.components().count() > 64 {
        return Err("python-tree-limit".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(format!("python-source-symlink-unsupported: {}", path.display()));
    }
    hasher.update(relative.to_string_lossy().as_bytes());
    hasher.update(&[0]);
    if metadata.is_dir() {
        let mut children = fs::read_dir(path)
            .map_err(|error| error.to_string())?
            .take(MAX_TREE_ENTRIES as usize + 1)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        if children.len() > MAX_TREE_ENTRIES as usize {
            return Err("python-tree-limit".into());
        }
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            hash_tree(&child.path(), &relative.join(child.file_name()), hasher, entries)?;
        }
    } else {
        hasher.update(&metadata.len().to_le_bytes());
        hash_file(path, hasher)?;
    }
    Ok(())
}

struct ValidatedConfig {
    config: Config,
    native: mantle_rust_plan_core::ResolvedNativeInputs,
}

fn validate(mut config: Config) -> Result<ValidatedConfig, String> {
    if config.schema != 1 || !["plan", "environment", "wheel"].contains(&config.mode.as_str()) {
        return Err("python-config-version-or-mode".into());
    }
    config.target = admit_target(config.target).map_err(|error| error.to_string())?;
    config.artifacts = admit_artifacts(config.artifacts).map_err(|error| error.to_string())?;
    config.extras = admit_selections(config.extras).map_err(|error| error.to_string())?;
    config.groups = admit_selections(config.groups).map_err(|error| error.to_string())?;
    config.members = admit_selections(config.members).map_err(|error| error.to_string())?;
    config.build_packages = admit_selections(config.build_packages).map_err(|error| error.to_string())?;
    if config.build_requirements.len() > 64
        || config.dynamic_requirements.len() > 64
        || config.runtime_inputs.len() > 1024
        || config.git_sources.len() > 256
    {
        return Err("python-input-limit".into());
    }
    if config.store_prefix.ends_with('/') || !config.store_prefix.starts_with('/') {
        return Err("python-store-prefix".into());
    }
    for path in [&config.python, &config.uv, &config.project] {
        store_root(path, &config.store_prefix)?;
    }
    for artifact in &config.artifacts {
        if config.mode != "plan" || !artifact.path.is_empty() {
            store_root(&artifact.path, &config.store_prefix)?;
        }
    }
    let native = resolve_native_inputs(&config.native_inputs).map_err(|error| error.to_string())?;
    for root in native
        .inputs
        .iter()
        .chain(&config.runtime_inputs)
        .chain(config.git_sources.values().map(|source| &source.path))
    {
        if store_root(root, &config.store_prefix)? != root {
            return Err(format!("python-input-not-root: {root}"));
        }
    }
    for key in native.env.keys() {
        if key.starts_with("PYTHON")
            || key.starts_with("UV_")
            || key.starts_with("PIP_")
            || key.starts_with("MANTLE_")
            || [
                "HOME",
                "TMPDIR",
                "SOURCE_DATE_EPOCH",
                "LD_PRELOAD",
                "LD_LIBRARY_PATH",
                "VIRTUAL_ENV",
                "XDG_CONFIG_HOME",
                "XDG_CACHE_HOME",
            ]
            .contains(&key.as_str())
        {
            return Err(format!("python-native-env-reserved: {key}"));
        }
    }
    if config.source_date_epoch < 315532800 || config.source_date_epoch > 4354819199 {
        return Err("python-wheel-epoch-range: requires ZIP-representable UTC epoch".into());
    }
    Ok(ValidatedConfig { config, native })
}

fn admit_evaluated_inputs(config: &Config, native: &mantle_rust_plan_core::ResolvedNativeInputs) -> Result<(), String> {
    let roots: Vec<String> =
        serde_json::from_str(&std::env::var("MANTLE_PYTHON_INPUTS").map_err(|_| "MANTLE_PYTHON_INPUTS is required")?)
            .map_err(|error| error.to_string())?;
    if roots.len() > 1024 {
        return Err("python-input-limit".into());
    }
    let paths = [&config.python, &config.uv, &config.project]
        .into_iter()
        .chain(config.artifacts.iter().filter(|artifact| !artifact.path.is_empty()).map(|artifact| &artifact.path))
        .chain(native.inputs.iter())
        .chain(config.runtime_inputs.iter())
        .chain(config.git_sources.values().map(|source| &source.path));
    for path in paths {
        let root = store_root(path, &config.store_prefix)?;
        if !roots.iter().any(|declared| declared == root) {
            return Err(format!("python-input-undeclared: {path}"));
        }
    }
    let declared =
        std::env::var("MANTLE_PYTHON_NATIVE_INPUTS").map_err(|_| "MANTLE_PYTHON_NATIVE_INPUTS is required")?;
    let declaration: NativeInputDeclaration = serde_json::from_str(&declared).map_err(|error| error.to_string())?;
    if declaration != config.native_inputs {
        return Err("python-native-declaration-mismatch".into());
    }
    let runtime: Vec<String> = serde_json::from_str(
        &std::env::var("MANTLE_PYTHON_RUNTIME_INPUTS").map_err(|_| "MANTLE_PYTHON_RUNTIME_INPUTS is required")?,
    )
    .map_err(|error| error.to_string())?;
    if runtime != config.runtime_inputs {
        return Err("python-runtime-declaration-mismatch".into());
    }
    Ok(())
}

fn execute() -> Result<(), String> {
    let config_path = std::env::var("MANTLE_PYTHON_CONFIG").map_err(|_| "MANTLE_PYTHON_CONFIG is required")?;
    let output = PathBuf::from(std::env::var("out").map_err(|_| "out is required")?);
    if output.exists() {
        return Err("python-output-already-exists".into());
    }
    let bytes = read_bounded(Path::new(&config_path), MAX_CONFIG_BYTES)?;
    let config: Config = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let ValidatedConfig { config, native } = validate(config)?;
    admit_evaluated_inputs(&config, &native)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-python-plan-v1\0");
    hasher.update(DRIVER.as_bytes());
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    hash_file(&executable, &mut hasher)?;
    hasher.update(&serde_json::to_vec(&config).map_err(|error| error.to_string())?);
    hash_tree(Path::new(&config.project), Path::new("project"), &mut hasher, &mut 0)?;
    for (name, source) in &config.git_sources {
        hash_tree(Path::new(&source.path), Path::new(name), &mut hasher, &mut 0)?;
    }
    for artifact in &config.artifacts {
        if !artifact.path.is_empty() {
            hash_file(Path::new(&artifact.path), &mut hasher)?;
        }
    }
    let identity = hasher.finalize().to_hex().to_string();
    let mut command = Command::new(&config.python);
    command.args(["-I", "-S", "-c", DRIVER, &config_path]);
    command.env_clear().envs(&native.env);
    command.env("PATH", native.path.join(":"));
    command.env("MANTLE_PYTHON_RESOLVED_NATIVE", serde_json::to_string(&native).map_err(|error| error.to_string())?);
    command.env("MANTLE_PYTHON_IDENTITY", &identity).env("out", &output);
    command.env("LC_ALL", "C").env("TZ", "UTC");
    let status = command.status().map_err(|error| format!("python-launch: {error}"))?;
    if !status.success() {
        return Err(format!("python-execution-failed: {status}"));
    }
    let manifest_path = output.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&read_bounded(&manifest_path, MAX_CONFIG_BYTES)?).map_err(|error| error.to_string())?;
    let mut outputs = BTreeMap::new();
    collect_output_digests(&output, &output, &mut outputs, &mut 0)?;
    manifest["output_blake3"] = serde_json::to_value(outputs).map_err(|error| error.to_string())?;
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    println!("python-adapter: {identity} {}", output.display());
    Ok(())
}

fn collect_output_digests(
    root: &Path,
    directory: &Path,
    digests: &mut BTreeMap<String, String>,
    entries: &mut u32,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        *entries = entries.saturating_add(1);
        if *entries > MAX_TREE_ENTRIES {
            return Err("python-output-limit".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_output_digests(root, &path, digests, entries)?;
        } else if path != root.join("manifest.json") {
            let mut hasher = blake3::Hasher::new();
            hash_file(&path, &mut hasher)?;
            let relative = path.strip_prefix(root).map_err(|error| error.to_string())?.to_string_lossy().into_owned();
            digests.insert(relative, hasher.finalize().to_hex().to_string());
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        fail(error);
    }
}
