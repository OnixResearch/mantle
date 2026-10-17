//! Sandboxed offline Cargo oracle and per-package source-slice producer.
//! The declared planner supplies rust-plan unit identity and metadata; Cargo's
//! versioned unit graph supplies exact direct producer edges and extern names.

mod source_nar;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use mantle_cargo_unit_plan::Blocker;
use mantle_cargo_unit_plan::ExternBinding;
use mantle_cargo_unit_plan::NativeInput;
use mantle_cargo_unit_plan::PlanBindings;
use mantle_cargo_unit_plan::SourceSlice;
use mantle_cargo_unit_plan::UnitBinding;
use mantle_cargo_unit_plan::lower;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::plan_existing_unit_effects;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const MAX_ORACLE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    workspace: String,
    vendor: Option<String>,
    planner: String,
    cargo: String,
    rustc: String,
    linker: String,
    toolchain: String,
    helper: String,
    runtime_inputs: Vec<String>,
    store_prefix: String,
    system: String,
    host_triple: String,
    target_triple: String,
    profile: String,
    #[serde(default)]
    native_inputs: BTreeMap<String, NativeInput>,
}

#[derive(Debug, Deserialize)]
struct OracleGraph {
    version: u32,
    units: Vec<GraphUnit>,
    roots: Vec<usize>,
}
#[derive(Debug, Deserialize)]
struct GraphUnit {
    pkg_id: String,
    target: GraphTarget,
    #[serde(default)]
    features: Vec<String>,
    mode: String,
    profile: GraphProfile,
    dependencies: Vec<GraphEdge>,
}
#[derive(Debug, Deserialize)]
struct GraphTarget {
    kind: Vec<String>,
    name: String,
    src_path: String,
    edition: String,
}
#[derive(Debug, Deserialize)]
struct GraphProfile {
    name: String,
    opt_level: String,
    debuginfo: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GraphEdge {
    index: usize,
    extern_crate_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
}
#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    manifest_path: String,
    name: String,
    version: String,
    source: Option<String>,
    links: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlannerReceipt {
    schema_version: u32,
    cargo_mode: CargoMode,
    lockfile: Lockfile,
    unit_graph: GraphSummary,
    unit_derivation_graph: DerivationGraph,
    cargo_unit_derivation_graph: DerivationGraph,
}
#[derive(Debug, Deserialize)]
struct CargoMode {
    compatibility_class: String,
}
#[derive(Debug, Deserialize)]
struct Lockfile {
    blake3: String,
}
#[derive(Debug, Deserialize)]
struct GraphSummary {
    digest_blake3: String,
}
#[derive(Debug, Deserialize)]
struct DerivationGraph {
    ready: bool,
    derivations: Vec<PlannerUnit>,
    blockers: Vec<Value>,
}
#[derive(Debug, Clone, Deserialize)]
struct PlannerUnit {
    unit_id: String,
    package_id: String,
    target_name: String,
    target_kind: String,
    execution_kind: String,
    rustc_metadata_hash: String,
    derivation: PlannerDerivation,
}
#[derive(Debug, Clone, Deserialize)]
struct PlannerDerivation {
    args: Vec<String>,
    env: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct Evidence<'a> {
    schema: &'static str,
    evidence_class: &'static str,
    project_build_status: &'static str,
    lock_blake3: &'a str,
    unit_graph_blake3: &'a str,
    toolchain_blake3: String,
    plan_blake3: &'a str,
    unit_count: usize,
    non_claims: [&'static str; 5],
}

fn block(code: &'static str, subject: &str, detail: impl AsRef<str>) -> Vec<Blocker> {
    vec![Blocker {
        code,
        subject: subject.into(),
        detail: detail.as_ref().into(),
    }]
}
fn required(name: &str) -> Result<String, Vec<Blocker>> {
    env::var(name).map_err(|_| block("unit-plan-oracle-failure", name, "missing declared input"))
}
fn store_root(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix).and_then(|rest| rest.strip_prefix('/')).is_some_and(|rest| {
        rest.split('/')
            .next()
            .is_some_and(|name| name.len() >= 34 && name.as_bytes().get(32) == Some(&b'-'))
    })
}
fn in_declared_source(path: &Path, workspace: &Path, vendor: Option<&Path>) -> bool {
    path.starts_with(workspace) || vendor.is_some_and(|root| path.starts_with(root))
}

fn command_output(command: &mut Command, subject: &str) -> Result<Vec<u8>, Vec<Blocker>> {
    let output = command.output().map_err(|error| block("unit-plan-oracle-failure", subject, error.to_string()))?;
    if output.stdout.len() > MAX_ORACLE_BYTES || output.stderr.len() > MAX_ORACLE_BYTES {
        return Err(block("unit-plan-limit", subject, "oracle output exceeds 32 MiB"));
    }
    if !output.status.success() {
        return Err(block("unit-plan-oracle-failure", subject, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

fn command_env(command: &mut Command, config: &Config, root: &Path, cargo_home: &Path) {
    command
        .env_clear()
        .current_dir(root)
        .env("CARGO_HOME", cargo_home)
        .env("CARGO_TARGET_DIR", cargo_home.join("target"))
        .env("CARGO_NET_OFFLINE", "true")
        .env("RUSTC_BOOTSTRAP", "1")
        .env("RUSTC", &config.rustc)
        .env("HOME", cargo_home)
        .env("TMPDIR", cargo_home)
        .env("PATH", format!("{}/bin", config.toolchain));
}

fn normalized_graph_digest(bytes: &[u8]) -> Result<String, Vec<Blocker>> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "unit-graph", error.to_string()))?;
    let canonical = serde_json::to_vec(&value)
        .map_err(|error| block("unit-plan-oracle-failure", "unit-graph", error.to_string()))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn parse_unit_graph(bytes: &[u8]) -> Result<OracleGraph, Vec<Blocker>> {
    let graph: OracleGraph = serde_json::from_slice(bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "cargo unit-graph", error.to_string()))?;
    if graph.version != 1 {
        return Err(block("unit-plan-oracle-failure", "unit-graph", "unknown unit-graph version"));
    }
    Ok(graph)
}

fn run_oracle(
    config: &Config,
    workspace: &Path,
    cargo_home: &Path,
) -> Result<(Metadata, OracleGraph, PlannerReceipt), Vec<Blocker>> {
    let mut metadata = Command::new(&config.cargo);
    metadata.args(["metadata", "--format-version", "1", "--locked", "--offline"]);
    command_env(&mut metadata, config, workspace, cargo_home);
    let metadata = command_output(&mut metadata, "cargo metadata")?;
    let metadata: Metadata = serde_json::from_slice(&metadata)
        .map_err(|error| block("unit-plan-oracle-failure", "cargo metadata", error.to_string()))?;
    let mut graph = Command::new(&config.cargo);
    graph.args([
        "build",
        "-Z",
        "unstable-options",
        "--unit-graph",
        "--locked",
        "--offline",
        "--profile",
        &config.profile,
    ]);
    command_env(&mut graph, config, workspace, cargo_home);
    let graph_bytes = command_output(&mut graph, "cargo unit-graph")?;
    let graph_digest = normalized_graph_digest(&graph_bytes)?;
    let graph = parse_unit_graph(&graph_bytes)?;
    let mut planner = Command::new(&config.planner);
    planner.args(["--json", "rust-plan", "--root"]).arg(workspace).args([
        "--cargo",
        &config.cargo,
        "--rustc",
        &config.rustc,
        "--profile",
        &config.profile,
    ]);
    command_env(&mut planner, config, workspace, cargo_home);
    planner.env("MANTLE_UNIT_PLAN_CARGO_INDEXED_DERIVATIONS", "1");
    let receipt_bytes = command_output(&mut planner, "rust-plan")?;
    let receipt: PlannerReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "rust-plan", error.to_string()))?;
    if receipt.schema_version != 1
        || receipt.cargo_mode.compatibility_class != "cargo-oracle-evidence"
        || receipt.unit_graph.digest_blake3 != graph_digest
        || !receipt.unit_derivation_graph.ready
        || !receipt.cargo_unit_derivation_graph.ready
    {
        return Err(block(
            "unit-plan-oracle-failure",
            "rust-plan",
            "planner graph was blocked or disagreed with Cargo",
        ));
    }
    Ok((metadata, graph, receipt))
}

struct PackageSource {
    source_id: String,
    kind: String,
    root: String,
}

struct SourcePaths {
    slices: Vec<SourceSlice>,
    package_bindings: BTreeMap<String, PackageSource>,
}

fn source_paths(
    config: &Config,
    root: &Path,
    vendor: Option<&Path>,
    graph: &OracleGraph,
    metadata: &Metadata,
    sources_output: &Path,
) -> Result<SourcePaths, Vec<Blocker>> {
    let mut packages = BTreeSet::new();
    let mut slices = Vec::new();
    let mut package_bindings = BTreeMap::new();
    for unit in &graph.units {
        if !packages.insert(unit.pkg_id.as_str()) {
            continue;
        }
        let package =
            metadata.packages.iter().find(|package| package.id == unit.pkg_id).ok_or_else(|| {
                block("unit-plan-source-unsupported", &unit.pkg_id, "package absent from Cargo metadata")
            })?;
        if package.source.as_ref().is_some_and(|source| !source.starts_with("registry+")) {
            return Err(block(
                "unit-plan-source-unsupported",
                &package.id,
                "only path and declared registry sources are supported",
            ));
        }
        let manifest = Path::new(&package.manifest_path);
        let package_root = manifest
            .parent()
            .ok_or_else(|| block("unit-plan-source-unsupported", &package.id, "manifest has no parent"))?;
        if !in_declared_source(package_root, root, vendor) {
            return Err(block(
                "unit-plan-source-unsupported",
                &package.id,
                "package root outside declared workspace/vendor",
            ));
        }
        let package_digest = blake3::hash(package.id.as_bytes()).to_hex().to_string();
        let source_id = format!("s.{package_digest}");
        let subpath = format!("packages/{package_digest}");
        let copied = sources_output.join(&subpath);
        source_nar::copy_package(package_root, &copied, 0)
            .map_err(|error| block("unit-plan-source-unsupported", &package.id, error))?;
        let digest = source_nar::nar_blake3(&copied)
            .map_err(|error| block("unit-plan-source-unsupported", &package.id, error))?;
        let name = package.name.replace('_', "-");
        let store_name = format!("crate-{}-{}", name, package.version);
        slices.push(SourceSlice {
            id: source_id.clone(),
            producer_output: "sources".into(),
            subpath,
            store_name,
            nar_blake3: digest,
        });
        let kind = if package.source.is_some() { "registry" } else { "path" };
        package_bindings.insert(package.id.clone(), PackageSource {
            source_id,
            kind: kind.into(),
            root: package_root.to_string_lossy().into_owned(),
        });
    }
    if slices.len() > 256 {
        return Err(block("unit-plan-limit", "sources", "more than 256 package slices"));
    }
    if !store_root(&config.workspace, &config.store_prefix) {
        return Err(block("unit-plan-host-path", "workspace", "source must be a declared store input"));
    }
    Ok(SourcePaths {
        slices,
        package_bindings,
    })
}

fn normalize_args(unit: &PlannerUnit, graph: &GraphUnit) -> Result<Vec<String>, Vec<Blocker>> {
    let mut args = Vec::new();
    let mut source_count = 0_u32;
    let mut index = 0;
    while index < unit.derivation.args.len() {
        let word = &unit.derivation.args[index];
        if word.ends_with(".rs") {
            let actual = fs::canonicalize(word)
                .map_err(|error| block("unit-plan-source-unsupported", &unit.unit_id, error.to_string()))?;
            let expected = fs::canonicalize(&graph.target.src_path)
                .map_err(|error| block("unit-plan-source-unsupported", &unit.unit_id, error.to_string()))?;
            if actual != expected {
                return Err(block(
                    "unit-plan-source-unsupported",
                    &unit.unit_id,
                    "planner source differs from Cargo target",
                ));
            }
            source_count += 1;
            index += 1;
            continue;
        }
        if word == "--extern" {
            if unit.derivation.args.get(index + 1).is_none() {
                return Err(block("unit-plan-mode-unsupported", &unit.unit_id, "unterminated --extern option"));
            }
            index += 2;
            continue;
        }
        if word == "-C" {
            let value = unit
                .derivation
                .args
                .get(index + 1)
                .ok_or_else(|| block("unit-plan-mode-unsupported", &unit.unit_id, "unterminated -C option"))?;
            if value.starts_with("metadata=") || value.starts_with("linker=") {
                index += 2;
                continue;
            }
        }
        if word.starts_with('/') || word.contains("=/") || word.contains("@/") || word.contains("{{mantle-") {
            return Err(block("unit-plan-host-path", &unit.unit_id, word));
        }
        args.push(word.clone());
        index += 1;
    }
    if source_count != 1
        || !args.iter().any(|arg| arg == "--crate-name")
        || !args.iter().any(|arg| arg == "--crate-type")
        || !args.windows(2).any(|pair| pair[0] == "--edition" && pair[1] == graph.target.edition)
    {
        return Err(block(
            "unit-plan-mode-unsupported",
            &unit.unit_id,
            "planner source, edition, or crate identity is ambiguous",
        ));
    }
    Ok(args)
}

fn match_planner<'a>(
    index: usize,
    graph: &GraphUnit,
    units: &[GraphUnit],
    receipt: &'a PlannerReceipt,
) -> Result<&'a PlannerUnit, Vec<Blocker>> {
    let compiled_index = if graph.mode == "run-custom-build" {
        let candidates = graph
            .dependencies
            .iter()
            .filter(|edge| {
                units.get(edge.index).is_some_and(|candidate| {
                    candidate.pkg_id == graph.pkg_id
                        && candidate.mode == "build"
                        && candidate.target.kind.first().is_some_and(|kind| kind == "custom-build")
                        && candidate.target.name == graph.target.name
                        && candidate.features == graph.features
                })
            })
            .map(|edge| edge.index)
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(block("unit-plan-mode-unsupported", &graph.pkg_id, "ambiguous compiled build script"));
        }
        candidates[0]
    } else {
        index
    };
    let prefix = format!("{compiled_index}:{}:{}:", graph.pkg_id, graph.target.name);
    let mut matches = receipt.cargo_unit_derivation_graph.derivations.iter().filter(|unit| {
        unit.unit_id.strip_prefix(&prefix).is_some_and(|suffix| {
            suffix == format!("{}:build", unit.target_kind)
                && graph.target.kind.iter().any(|kind| kind == &unit.target_kind)
        }) && unit.package_id == graph.pkg_id
            && unit.target_name == graph.target.name
    });
    let found = matches
        .next()
        .ok_or_else(|| block("unit-plan-oracle-failure", &graph.pkg_id, "missing Rust-plan unit at Cargo index"))?;
    if matches.next().is_some() {
        return Err(block("unit-plan-mode-unsupported", &graph.pkg_id, "ambiguous Rust-plan target identity"));
    }
    Ok(found)
}

fn resolve_native(config: &Config, package_id: &str) -> Result<NativeInput, Vec<Blocker>> {
    let Some(declaration) = config.native_inputs.get(package_id) else {
        return Ok(NativeInput::default());
    };
    if declaration.inputs.len() > 64 || declaration.env.len() > 128 || declaration.path.len() > 64 {
        return Err(block("unit-plan-limit", package_id, "native declaration too large"));
    }
    for (role, path) in &declaration.inputs {
        if role.is_empty()
            || !role.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || !store_root(path, &config.store_prefix)
            || !Path::new(path).exists()
        {
            return Err(block("native-input-role-undeclared", package_id, role));
        }
    }
    fn substitute(value: &str, package_id: &str, roots: &BTreeMap<String, String>) -> Result<String, Vec<Blocker>> {
        let mut resolved = String::new();
        let mut rest = value;
        while let Some((prefix, suffix)) = rest.split_once("{{role:") {
            resolved.push_str(prefix);
            let (role, after) =
                suffix.split_once("}}").ok_or_else(|| block("native-input-role-undeclared", package_id, suffix))?;
            resolved.push_str(roots.get(role).ok_or_else(|| block("native-input-role-undeclared", package_id, role))?);
            rest = after;
        }
        resolved.push_str(rest);
        let declared_path = |candidate: &str| {
            !candidate.split('/').any(|part| part == "." || part == "..")
                && roots.values().any(|root| {
                    candidate == root || candidate.strip_prefix(root).is_some_and(|rest| rest.starts_with('/'))
                })
        };
        if resolved.contains("{{role:")
            || resolved.contains('\0')
            || resolved.len() > 16_384
            || resolved.split(['=', ':', ',', ';', ' ']).any(|part| part.starts_with('/') && !declared_path(part))
        {
            return Err(block("native-input-role-undeclared", package_id, value));
        }
        Ok(resolved)
    }
    let mut native = declaration.clone();
    for value in native.env.values_mut() {
        *value = substitute(value, package_id, &declaration.inputs)?;
    }
    for value in &mut native.path {
        *value = substitute(value, package_id, &declaration.inputs)?;
    }
    Ok(native)
}

struct PlannedUnitFacts {
    facts: Vec<ExistingUnitFacts>,
    bindings: Vec<UnitBinding>,
    roots: Vec<String>,
}

fn build_facts(
    config: &Config,
    graph: &OracleGraph,
    receipt: &PlannerReceipt,
    package_sources: &BTreeMap<String, PackageSource>,
    metadata: &Metadata,
) -> Result<PlannedUnitFacts, Vec<Blocker>> {
    let mut selected = Vec::<PlannerUnit>::new();
    for (index, unit) in graph.units.iter().enumerate() {
        let kind = unit
            .target
            .kind
            .first()
            .ok_or_else(|| block("unit-plan-mode-unsupported", &unit.pkg_id, "empty target kind"))?;
        if unit.mode != "build" && !(kind == "custom-build" && unit.mode == "run-custom-build") {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, &unit.mode));
        }
        if !matches!(kind.as_str(), "lib" | "bin" | "proc-macro" | "custom-build") {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, kind));
        }
        let mut planned = match_planner(index, unit, &graph.units, receipt)?.clone();
        if unit.mode == "run-custom-build" {
            planned.unit_id = format!("{index}:{}:{}:custom-build:run-custom-build", unit.pkg_id, unit.target.name);
            planned.execution_kind = "build-script-run".into();
        }
        selected.push(planned);
    }
    let mut links_owners = BTreeMap::new();
    for package in &metadata.packages {
        if let Some(links) = &package.links {
            if !links.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
                return Err(block("build-script-links-conflict", links, "invalid Cargo links key"));
            }
            if let Some(existing) = links_owners.insert(links, &package.id)
                && existing != &package.id
            {
                return Err(block("build-script-links-conflict", links, "two packages declare one links key"));
            }
        }
    }
    for package_id in config.native_inputs.keys() {
        if !graph.units.iter().any(|unit| unit.pkg_id == *package_id && unit.mode == "run-custom-build") {
            return Err(block("native-input-role-undeclared", package_id, "no selected build-script execution unit"));
        }
    }
    let mut facts = Vec::new();
    let mut bindings = Vec::new();
    for (index, unit) in graph.units.iter().enumerate() {
        let planned = &selected[index];
        let source = package_sources
            .get(&unit.pkg_id)
            .ok_or_else(|| block("unit-plan-source-unsupported", &unit.pkg_id, "missing package slice"))?;
        let source_id = &source.source_id;
        let source_kind = &source.kind;
        let root = &source.root;
        let entry = Path::new(&unit.target.src_path)
            .strip_prefix(root)
            .map_err(|_| block("unit-plan-source-unsupported", &unit.pkg_id, "source path outside package"))?;
        let entry = entry
            .to_str()
            .ok_or_else(|| block("unit-plan-source-unsupported", &unit.pkg_id, "non-UTF8 source entry"))?;
        let mut externs = Vec::new();
        let mut dependencies = Vec::new();
        let is_run = unit.mode == "run-custom-build";
        let mut linked_runs = BTreeMap::new();
        let mut compiled_script = None;
        let mut script_run = None;
        for edge in &unit.dependencies {
            let dependency = selected
                .get(edge.index)
                .ok_or_else(|| block("unit-plan-oracle-failure", &planned.unit_id, "dependency index outside graph"))?;
            dependencies.push(dependency.unit_id.clone());
            let dep_graph = &graph.units[edge.index];
            if dep_graph.target.kind.first().is_some_and(|kind| kind == "custom-build") {
                if is_run && dep_graph.pkg_id == unit.pkg_id && dep_graph.mode == "build" {
                    compiled_script = Some(dependency.unit_id.clone());
                }
                if !is_run && dep_graph.pkg_id == unit.pkg_id && dep_graph.mode == "run-custom-build" {
                    script_run = Some(dependency.unit_id.clone());
                }
                if is_run && dep_graph.mode == "run-custom-build" && dep_graph.pkg_id != unit.pkg_id {
                    let package =
                        metadata.packages.iter().find(|pkg| pkg.id == dep_graph.pkg_id).ok_or_else(|| {
                            block("unit-plan-oracle-failure", &dep_graph.pkg_id, "missing links package")
                        })?;
                    if let Some(key) = &package.links
                        && linked_runs.insert(key.clone(), dependency.unit_id.clone()).is_some()
                    {
                        return Err(block("build-script-links-conflict", key, "duplicate links run"));
                    }
                }
            }
            if is_run
                && dep_graph.pkg_id != unit.pkg_id
                && let Some(key) =
                    metadata.packages.iter().find(|pkg| pkg.id == dep_graph.pkg_id).and_then(|pkg| pkg.links.as_ref())
            {
                let runs = graph
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, candidate)| {
                        candidate.pkg_id == dep_graph.pkg_id && candidate.mode == "run-custom-build"
                    })
                    .map(|(index, _)| &selected[index].unit_id)
                    .collect::<Vec<_>>();
                if runs.len() != 1 {
                    return Err(block(
                        "build-script-links-conflict",
                        key,
                        "missing or ambiguous dependency script run",
                    ));
                }
                linked_runs.insert(key.clone(), runs[0].clone());
                dependencies.push(runs[0].clone());
            }
            if !is_run
                && dep_graph.mode != "run-custom-build"
                && let Some(name) = &edge.extern_crate_name
            {
                externs.push(ExternBinding {
                    name: name.clone(),
                    unit_id: dependency.unit_id.clone(),
                });
            }
        }
        // Cargo's run-custom-build node may not expose the compiled executable as
        // an edge. Bind it explicitly by exact package/target/features rather
        // than relying on rust-plan's collapsed host-artifact identity.
        if is_run && compiled_script.is_none() {
            let candidates = graph
                .units
                .iter()
                .enumerate()
                .filter(|(_, candidate)| {
                    candidate.pkg_id == unit.pkg_id
                        && candidate.mode == "build"
                        && candidate.target.kind.first().is_some_and(|kind| kind == "custom-build")
                        && candidate.target.name == unit.target.name
                        && candidate.features == unit.features
                })
                .map(|(index, _)| &selected[index].unit_id)
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(block("unit-plan-mode-unsupported", &planned.unit_id, "ambiguous compiled build script"));
            }
            compiled_script = Some(candidates[0].clone());
            dependencies.push(candidates[0].clone());
        }
        if !is_run && unit.target.kind[0] != "custom-build" && script_run.is_none() {
            let candidates = graph
                .units
                .iter()
                .enumerate()
                .filter(|(_, candidate)| candidate.pkg_id == unit.pkg_id && candidate.mode == "run-custom-build")
                .map(|(index, _)| &selected[index].unit_id)
                .collect::<Vec<_>>();
            if candidates.len() > 1 {
                return Err(block("unit-plan-mode-unsupported", &planned.unit_id, "ambiguous script run"));
            }
            if let Some(run) = candidates.first() {
                script_run = Some((*run).clone());
                dependencies.push((*run).clone());
            }
        }
        dependencies.sort();
        dependencies.dedup();
        let execution_kind = if is_run {
            "build-script-run"
        } else if matches!(unit.target.kind[0].as_str(), "proc-macro" | "custom-build") {
            "host"
        } else {
            "target"
        };
        if planned.execution_kind != execution_kind {
            return Err(block(
                "unit-plan-triple-unsupported",
                &planned.unit_id,
                "planner and Cargo disagree on execution triple",
            ));
        }
        let arguments = if is_run {
            vec!["run-build-script".into()]
        } else {
            normalize_args(planned, unit)?
        };
        let mut compile_env = Vec::new();
        for (key, value) in &planned.derivation.env {
            if key.starts_with("CARGO_PKG_") || key.starts_with("CARGO_FEATURE_") {
                if value.starts_with('/') || value.contains("=/") || value.contains('\0') {
                    return Err(block("unit-plan-host-path", &planned.unit_id, key));
                }
                compile_env.push((key.clone(), value.clone()));
            }
        }
        let package = metadata
            .packages
            .iter()
            .find(|pkg| pkg.id == unit.pkg_id)
            .ok_or_else(|| block("unit-plan-oracle-failure", &unit.pkg_id, "missing package metadata"))?;
        let mut version = package.version.splitn(2, '-');
        let digits = version.next().unwrap_or_default().split('.').collect::<Vec<_>>();
        if digits.len() != 3 || digits.iter().any(|part| part.parse::<u32>().is_err()) {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, "invalid package version"));
        }
        compile_env.extend([
            ("CARGO_PKG_NAME".into(), package.name.clone()),
            ("CARGO_PKG_VERSION".into(), package.version.clone()),
            ("CARGO_PKG_VERSION_MAJOR".into(), digits[0].into()),
            ("CARGO_PKG_VERSION_MINOR".into(), digits[1].into()),
            ("CARGO_PKG_VERSION_PATCH".into(), digits[2].into()),
            ("CARGO_PKG_VERSION_PRE".into(), version.next().unwrap_or_default().into()),
        ]);
        for feature in &unit.features {
            let name = format!("CARGO_FEATURE_{}", feature.replace('-', "_").to_ascii_uppercase());
            if name.len() > 128 || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
                return Err(block("unit-plan-mode-unsupported", &planned.unit_id, "invalid feature name"));
            }
            compile_env.push((name, "1".into()));
        }
        if is_run {
            if unit.profile.name != config.profile
                || unit.profile.opt_level.len() > 8
                || !unit.profile.opt_level.bytes().all(|byte| byte.is_ascii_alphanumeric())
            {
                return Err(block(
                    "unit-plan-mode-unsupported",
                    &planned.unit_id,
                    "invalid Cargo build-script profile",
                ));
            }
            compile_env.extend([
                ("PROFILE".into(), if config.profile == "dev" { "debug" } else { "release" }.into()),
                ("OPT_LEVEL".into(), unit.profile.opt_level.clone()),
                ("DEBUG".into(), (unit.profile.debuginfo.unwrap_or(0) > 0).to_string()),
            ]);
        }
        compile_env.push(("CARGO_MANIFEST_DIR".into(), format!("{{{{mantle-source:{source_id}}}}}")));
        facts.push(ExistingUnitFacts {
            unit_id: planned.unit_id.clone(),
            package_id: unit.pkg_id.clone(),
            target_name: unit.target.name.clone(),
            target_kind: unit.target.kind[0].clone(),
            execution_kind: execution_kind.into(),
            dependency_unit_ids: dependencies,
            arguments,
            environment: compile_env,
            input_identities: vec![format!("source:{source_id}")],
            expected_outputs: if is_run {
                vec!["out".into(), "flags".into()]
            } else {
                vec!["out".into()]
            },
            execution_order: u32::try_from(index)
                .map_err(|_| block("unit-plan-limit", "units", "unit index overflow"))?,
        });
        bindings.push(UnitBinding {
            unit_id: planned.unit_id.clone(),
            source_id: source_id.clone(),
            source_kind: source_kind.clone(),
            source_entry: entry.into(),
            source_label: format!("{}-{}", unit.target.name.replace('_', "-"), "source"),
            rustc_metadata_hash: planned.rustc_metadata_hash.clone(),
            triple: if execution_kind == "host" {
                config.host_triple.clone()
            } else {
                config.target_triple.clone()
            },
            links: metadata.packages.iter().find(|pkg| pkg.id == unit.pkg_id).and_then(|pkg| pkg.links.clone()),
            native: if is_run {
                config.native_inputs.get(&unit.pkg_id).cloned().unwrap_or_default()
            } else {
                NativeInput::default()
            },
            compiled_script,
            script_run,
            linked_runs,
            externs,
        });
    }
    let roots = graph
        .roots
        .iter()
        .map(|index| {
            selected
                .get(*index)
                .map(|unit| unit.unit_id.clone())
                .ok_or_else(|| block("unit-plan-oracle-failure", "roots", "root index outside graph"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PlannedUnitFacts { facts, bindings, roots })
}

fn write_workspace_source_config(workspace: &Path, source_config: &str) -> Result<(), Vec<Blocker>> {
    let writable = |path: &Path| {
        let mut permissions = fs::metadata(path)
            .map_err(|error| block("unit-plan-oracle-failure", "workspace source config", error.to_string()))?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o200);
        fs::set_permissions(path, permissions)
            .map_err(|error| block("unit-plan-oracle-failure", "workspace source config", error.to_string()))
    };
    // Only the private workspace copy needs to accept the declared Cargo
    // source config; keep the immutable input and all other copied files intact.
    let config_dir = workspace.join(".cargo");
    if !config_dir.exists() {
        writable(workspace)?;
    }
    fs::create_dir_all(&config_dir)
        .map_err(|error| block("unit-plan-oracle-failure", "workspace source config", error.to_string()))?;
    writable(&config_dir)?;
    let config_path = config_dir.join("config.toml");
    match fs::remove_file(&config_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(block("unit-plan-oracle-failure", "workspace source config", error.to_string())),
    }
    fs::write(config_path, source_config)
        .map_err(|error| block("unit-plan-oracle-failure", "workspace source config", error.to_string()))
}

fn run() -> Result<(), Vec<Blocker>> {
    let mut config: Config = serde_json::from_slice(
        &fs::read(required("MANTLE_UNIT_CONFIG")?)
            .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?,
    )
    .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?;
    if let Ok(declared) = env::var("MANTLE_UNIT_NATIVE_INPUTS") {
        let native_inputs: BTreeMap<String, NativeInput> = serde_json::from_str(&declared)
            .map_err(|error| block("native-input-role-undeclared", "native_inputs", error.to_string()))?;
        if !config.native_inputs.is_empty() && config.native_inputs != native_inputs {
            return Err(block(
                "native-input-role-undeclared",
                "native_inputs",
                "config and Nickel declarations differ",
            ));
        }
        config.native_inputs = native_inputs;
    } else if !config.native_inputs.is_empty() {
        return Err(block("native-input-role-undeclared", "native_inputs", "missing Nickel declaration"));
    }
    let declared_runtime: Vec<String> = serde_json::from_str(&required("MANTLE_UNIT_RUNTIME_INPUTS")?)
        .map_err(|error| block("unit-plan-host-path", "runtime_inputs", error.to_string()))?;
    if config.runtime_inputs != declared_runtime {
        return Err(block("unit-plan-host-path", "runtime_inputs", "config and Nickel runtime declarations differ"));
    }
    if ![
        &config.workspace,
        &config.planner,
        &config.cargo,
        &config.rustc,
        &config.linker,
        &config.toolchain,
        &config.helper,
    ]
    .iter()
    .all(|path| store_root(path, &config.store_prefix))
        || config.vendor.as_ref().is_some_and(|path| !store_root(path, &config.store_prefix))
    {
        return Err(block("unit-plan-host-path", "inputs", "producer tools and sources must be declared store paths"));
    }
    if config.profile != "dev" && config.profile != "release" {
        return Err(block("unit-plan-mode-unsupported", "profile", &config.profile));
    }
    // Resolve declarations once, before copying sources or running the Cargo
    // oracle. The selected-run check in build_facts still rejects declarations
    // for packages without a build-script execution unit.
    let resolved_native = config
        .native_inputs
        .keys()
        .map(|package_id| resolve_native(&config, package_id).map(|native| (package_id.clone(), native)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    config.native_inputs = resolved_native;
    let scratch = PathBuf::from(required("TMPDIR")?).join("cargo-unit-plan-workspace");
    fs::create_dir(&scratch).map_err(|error| block("unit-plan-oracle-failure", "scratch", error.to_string()))?;
    // Nix fixes TMPDIR to /build; copying here keeps Cargo package identities
    // independent of the immutable workspace store path across source edits.
    let workspace = scratch.join("workspace");
    source_nar::copy_package(Path::new(&config.workspace), &workspace, 0)
        .map_err(|error| block("unit-plan-source-unsupported", "workspace", error))?;
    let vendor = config.vendor.as_deref().map(Path::new);
    let cargo_home = scratch.join("cargo-home");
    fs::create_dir(&cargo_home).map_err(|error| block("unit-plan-oracle-failure", "cargo-home", error.to_string()))?;
    if let Some(vendor) = vendor {
        let source_config = format!(
            "[source.crates-io]\nreplace-with='declared-vendor'\n[source.declared-vendor]\ndirectory={:?}\n",
            vendor
        );
        fs::write(cargo_home.join("config.toml"), &source_config)
            .map_err(|error| block("unit-plan-oracle-failure", "cargo-home", error.to_string()))?;
        // Cargo sees CARGO_HOME/config.toml; rust-plan deliberately inspects the
        // copied workspace's declared source config, not ambient Cargo state.
        write_workspace_source_config(&workspace, &source_config)?;
    }
    let lock_before = fs::read(workspace.join("Cargo.lock"))
        .map_err(|error| block("unit-plan-oracle-failure", "Cargo.lock", error.to_string()))?;
    let (metadata, graph, receipt) = run_oracle(&config, &workspace, &cargo_home)?;
    if blake3::hash(&lock_before).to_hex().as_str() != receipt.lockfile.blake3
        || fs::read(workspace.join("Cargo.lock")).ok().as_deref() != Some(lock_before.as_slice())
    {
        return Err(block("unit-plan-oracle-failure", "Cargo.lock", "lock changed or planner lock identity differed"));
    }
    if !receipt.unit_derivation_graph.blockers.is_empty() {
        return Err(block("unit-plan-oracle-failure", "rust-plan", "unit derivation graph blocked"));
    }
    let sources_output = PathBuf::from(required("sources")?);
    fs::create_dir_all(sources_output.join("packages"))
        .map_err(|error| block("unit-plan-source-unsupported", "sources", error.to_string()))?;
    let SourcePaths {
        slices: sources,
        package_bindings,
    } = source_paths(&config, &workspace, vendor, &graph, &metadata, &sources_output)?;
    let PlannedUnitFacts {
        facts,
        bindings: units,
        roots,
    } = build_facts(&config, &graph, &receipt, &package_bindings, &metadata)?;
    let effects = plan_existing_unit_effects(facts).map_err(|blockers| {
        blockers
            .iter()
            .map(|blocker| Blocker {
                code: "unit-plan-mode-unsupported",
                subject: blocker.subject.clone(),
                detail: blocker.message.clone(),
            })
            .collect::<Vec<_>>()
    })?;
    let bindings = PlanBindings {
        store_prefix: config.store_prefix,
        system: config.system,
        target_triple: config.target_triple,
        host_triple: config.host_triple,
        helper: config.helper,
        rustc: config.rustc,
        linker: config.linker,
        toolchain: config.toolchain,
        runtime_inputs: config.runtime_inputs,
        sources,
        units,
        roots,
    };
    let (bytes, digest) = lower(&effects, &bindings)?;
    fs::write(required("plan")?, &bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "plan", error.to_string()))?;
    let out = PathBuf::from(required("out")?);
    fs::create_dir_all(&out).map_err(|error| block("unit-plan-oracle-failure", "evidence", error.to_string()))?;
    let evidence = Evidence {
        schema: "mantle-cargo-unit-plan-evidence-v1",
        evidence_class: "cargo-unit-graph-dynamic-plan",
        project_build_status: "opt-in-project-build-lane",
        lock_blake3: &receipt.lockfile.blake3,
        unit_graph_blake3: &receipt.unit_graph.digest_blake3,
        toolchain_blake3: blake3::hash(bindings.toolchain.as_bytes()).to_hex().to_string(),
        plan_blake3: &digest,
        unit_count: effects.len(),
        non_claims: [
            "not-cargo-free-execution",
            "not-full-cargo-compatibility",
            "not-compiler-correctness",
            "not-release-reproducibility",
            "not-bootstrap-correctness",
        ],
    };
    fs::write(
        out.join("cargo-unit-plan-evidence.json"),
        serde_json::to_vec(&evidence)
            .map_err(|error| block("unit-plan-oracle-failure", "evidence", error.to_string()))?,
    )
    .map_err(|error| block("unit-plan-oracle-failure", "evidence", error.to_string()))?;
    Ok(())
}

fn main() {
    if let Err(blockers) = run() {
        if let Ok(out) = env::var("out") {
            let _ = fs::create_dir_all(&out);
            let _ = fs::write(
                Path::new(&out).join("cargo-unit-plan-blockers.json"),
                serde_json::to_vec(&blockers).unwrap_or_default(),
            );
        }
        eprintln!("{}", serde_json::to_string(&blockers).unwrap_or_default());
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undeclared_native_role_does_not_become_a_host_path() {
        let package_id = "path+file:///build/workspace/native-answer-sys#0.1.0";
        let mut config = Config {
            workspace: String::new(),
            vendor: None,
            planner: String::new(),
            cargo: String::new(),
            rustc: String::new(),
            linker: String::new(),
            toolchain: String::new(),
            helper: String::new(),
            runtime_inputs: Vec::new(),
            store_prefix: "/nix/store".into(),
            system: "x86_64-linux".into(),
            host_triple: "x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            profile: "dev".into(),
            native_inputs: BTreeMap::from([(package_id.into(), NativeInput {
                inputs: BTreeMap::new(),
                env: BTreeMap::from([("NATIVE_ANSWER_ROOT".into(), "{{role:missing}}".into())]),
                path: Vec::new(),
            })]),
        };
        let blockers = resolve_native(&config, package_id).unwrap_err();
        assert_eq!(blockers[0].code, "native-input-role-undeclared");
        assert_eq!(blockers[0].subject, package_id);
        config
            .native_inputs
            .get_mut(package_id)
            .unwrap()
            .env
            .insert("NATIVE_ANSWER_ROOT".into(), "LIBRARY_DIR=/usr/lib".into());
        let blockers = resolve_native(&config, package_id).unwrap_err();
        assert_eq!(blockers[0].code, "native-input-role-undeclared");
    }

    #[test]
    fn unknown_cargo_unit_graph_version_blocks_before_lowering() {
        let blockers = parse_unit_graph(br#"{"version":2,"units":[],"roots":[]}"#).unwrap_err();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "unit-plan-oracle-failure");
        assert_eq!(blockers[0].subject, "unit-graph");
    }

    #[test]
    fn two_cargo_packages_cannot_claim_the_same_links_key() {
        let config: Config = serde_json::from_value(serde_json::json!({
            "workspace": "", "vendor": null, "planner": "", "cargo": "",
            "rustc": "", "linker": "", "toolchain": "", "helper": "",
            "runtime_inputs": [], "store_prefix": "/nix/store",
            "system": "x86_64-linux", "host_triple": "x86_64-unknown-linux-gnu",
            "target_triple": "x86_64-unknown-linux-gnu", "profile": "dev",
        }))
        .unwrap();
        let graph = OracleGraph {
            version: 1,
            units: Vec::new(),
            roots: Vec::new(),
        };
        let receipt: PlannerReceipt = serde_json::from_value(serde_json::json!({
            "schema_version": 1, "cargo_mode": {"compatibility_class": ""},
            "lockfile": {"blake3": ""}, "unit_graph": {"digest_blake3": ""},
            "unit_derivation_graph": {"ready": true, "derivations": [], "blockers": []},
            "cargo_unit_derivation_graph": {"ready": true, "derivations": [], "blockers": []},
        }))
        .unwrap();
        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "packages": [
                {
                    "id": "path+file:///workspace/a#0.1.0",
                    "manifest_path": "/workspace/a/Cargo.toml",
                    "name": "a", "version": "0.1.0", "source": null,
                    "links": "shared_native",
                },
                {
                    "id": "path+file:///workspace/b#0.1.0",
                    "manifest_path": "/workspace/b/Cargo.toml",
                    "name": "b", "version": "0.1.0", "source": null,
                    "links": "shared_native",
                },
            ],
        }))
        .unwrap();

        let blockers = build_facts(&config, &graph, &receipt, &BTreeMap::new(), &metadata)
            .err()
            .expect("duplicate Cargo links must reject the unit graph");
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "build-script-links-conflict");
        assert_eq!(blockers[0].subject, "shared_native");
    }

    #[test]
    fn copied_readonly_workspace_accepts_only_declared_source_config() {
        use std::time::SystemTime;
        use std::time::UNIX_EPOCH;

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("unit-plan-workspace-config-{}-{unique}", std::process::id()));
        fs::create_dir(&root).unwrap();
        for existing_config in [false, true] {
            let case = root.join(if existing_config { "existing" } else { "missing" });
            let source = case.join("immutable");
            let copied = case.join("workspace");
            fs::create_dir_all(&source).unwrap();
            fs::write(source.join("Cargo.toml"), "[package]\nname='readonly'\nversion='0.1.0'\n").unwrap();
            fs::set_permissions(source.join("Cargo.toml"), fs::Permissions::from_mode(0o444)).unwrap();
            if existing_config {
                fs::create_dir(source.join(".cargo")).unwrap();
                fs::write(source.join(".cargo/config.toml"), "[source.crates-io]\n").unwrap();
                fs::set_permissions(source.join(".cargo/config.toml"), fs::Permissions::from_mode(0o444)).unwrap();
                fs::set_permissions(source.join(".cargo"), fs::Permissions::from_mode(0o555)).unwrap();
            }
            fs::set_permissions(&source, fs::Permissions::from_mode(0o555)).unwrap();
            let source_digest = source_nar::nar_blake3(&source).unwrap();
            source_nar::copy_package(&source, &copied, 0).unwrap();
            let source_config = "[source.crates-io]\nreplace-with='declared-vendor'\n";
            write_workspace_source_config(&copied, source_config).unwrap();
            assert_eq!(fs::read_to_string(copied.join(".cargo/config.toml")).unwrap(), source_config);
            assert_eq!(source_nar::nar_blake3(&source).unwrap(), source_digest);
            assert_eq!(fs::metadata(copied.join("Cargo.toml")).unwrap().permissions().mode() & 0o777, 0o444);
            assert_eq!(fs::metadata(&source).unwrap().permissions().mode() & 0o777, 0o555);
            if existing_config {
                assert_eq!(fs::metadata(&copied).unwrap().permissions().mode() & 0o777, 0o555);
                assert_eq!(fs::read_to_string(source.join(".cargo/config.toml")).unwrap(), "[source.crates-io]\n");
                fs::set_permissions(source.join(".cargo"), fs::Permissions::from_mode(0o755)).unwrap();
            } else {
                assert!(!source.join(".cargo").exists());
            }
            fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(&copied, fs::Permissions::from_mode(0o755)).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
}
