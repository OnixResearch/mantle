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

use mantle_cargo_unit_plan::BUILD_SCRIPT_RUN;
use mantle_cargo_unit_plan::Blocker;
use mantle_cargo_unit_plan::BuildScriptRunBinding;
use mantle_cargo_unit_plan::ExternBinding;
use mantle_cargo_unit_plan::PlanBindings;
use mantle_cargo_unit_plan::RUN_BUILD_SCRIPT_MODE;
use mantle_cargo_unit_plan::SourceSlice;
use mantle_cargo_unit_plan::TargetLinkerBinding;
use mantle_cargo_unit_plan::UnitBinding;
use mantle_cargo_unit_plan::lower;
use mantle_rust_differential_core::Digest;
use mantle_rust_differential_core::waits::ArtifactRole;
use mantle_rust_differential_core::waits::Dependency;
use mantle_rust_differential_core::waits::ProcessRole;
use mantle_rust_differential_core::waits::UnitIdentity as WaitUnitIdentity;
use mantle_rust_plan_core::ExistingUnitFacts;
use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::ResolvedNativeInputs;
use mantle_rust_plan_core::plan_existing_unit_effects;
use mantle_rust_plan_core::resolve_native_inputs;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const MAX_ORACLE_BYTES: usize = 32 * 1024 * 1024;
/// Evaluated Nickel `native_inputs`, serialized by `lib/cargo_unit_plan.ncl`.
const NATIVE_INPUTS_ENV: &str = "MANTLE_UNIT_NATIVE_INPUTS";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    workspace: String,
    vendor: Option<String>,
    planner: String,
    cargo: String,
    rustc: String,
    linker: String,
    #[serde(default)]
    target_linker: Option<TargetLinkerBinding>,
    // Producer-only oracle toolchain. `rustc` must name a distinct compiler-only
    // store root, with its complete runtime/sysroot closure independently admitted.
    toolchain: String,
    helper: String,
    #[serde(default)]
    runtime_inputs: Vec<String>,
    store_prefix: String,
    system: String,
    host_triple: String,
    target_triple: String,
    profile: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    no_default_features: bool,
    /// Keyed by exact Cargo package id from the admitted graph.
    #[serde(default)]
    native_inputs: BTreeMap<String, NativeInputDeclaration>,
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
    mode: String,
    profile: Option<GraphProfile>,
    dependencies: Vec<GraphEdge>,
    features: Option<Vec<String>>,
}

impl GraphUnit {
    /// Preserve the oracle's original semantic default without claiming observed metadata.
    fn semantic_features(&self) -> &[String] {
        self.features.as_deref().unwrap_or_default()
    }
}
#[derive(Debug, Deserialize)]
struct GraphTarget {
    kind: Vec<String>,
    name: String,
    src_path: String,
    edition: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct GraphProfile {
    name: String,
    opt_level: String,
    debuginfo: Option<u32>,
    /// Preserve every admitted Cargo profile setting, including codegen and panic policy.
    #[serde(flatten)]
    settings: BTreeMap<String, Value>,
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
    #[serde(default)]
    links: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlannerReceipt {
    schema_version: u32,
    cargo_mode: CargoMode,
    lockfile: Lockfile,
    unit_graph: GraphSummary,
    unit_derivation_graph: DerivationGraph,
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
#[derive(Debug, Deserialize)]
struct PlannerUnit {
    unit_id: String,
    package_id: String,
    target_name: String,
    target_kind: String,
    execution_kind: String,
    rustc_metadata_hash: String,
    derivation: PlannerDerivation,
}
#[derive(Debug, Deserialize)]
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
/// Maximum path components accepted when lexically normalising Cargo source paths.
const MAX_SOURCE_PATH_COMPONENTS: usize = 256;

/// Lexically resolve `.` and `..` in an absolute Cargo source path (the unit graph reports
/// path-dependency sources as e.g. `app/../bar/build.rs`). Returns `None` for relative
/// paths, paths that climb above the filesystem root, or over-long paths.
fn normalize_source_path(path: &Path) -> Option<PathBuf> {
    use std::path::Component;
    if !path.is_absolute() || path.components().count() > MAX_SOURCE_PATH_COMPONENTS {
        return None;
    }
    let mut normalized = PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(_) => return None,
        }
    }
    debug_assert!(normalized.is_absolute());
    debug_assert!(!normalized.components().any(|part| part == Component::ParentDir));
    Some(normalized)
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
    if graph.units.len() > mantle_cargo_unit_plan::MAX_PLAN_UNITS {
        return Err(block("unit-plan-limit", "unit-graph", "more than 4096 Cargo units"));
    }
    Ok(graph)
}

fn select_features(command: &mut Command, config: &Config) -> Result<(), Vec<Blocker>> {
    if config.features.len() > 256
        || config.features.iter().any(|feature| {
            feature.is_empty() || feature.len() > 128 || feature.starts_with('-') || feature.contains([',', '\0', ' '])
        })
    {
        return Err(block("unit-plan-mode-unsupported", "features", "invalid feature selection"));
    }
    if !config.features.is_empty() {
        command.arg("--features").arg(config.features.join(","));
    }
    if config.no_default_features {
        command.arg("--no-default-features");
    }
    Ok(())
}
fn retain_oracle(name: &str, bytes: &[u8]) -> Result<(), Vec<Blocker>> {
    let out = PathBuf::from(required("out")?);
    fs::create_dir_all(&out).map_err(|error| block("unit-plan-oracle-failure", "evidence", error.to_string()))?;
    fs::write(out.join(name), bytes).map_err(|error| block("unit-plan-oracle-failure", name, error.to_string()))
}

fn run_oracle(
    config: &Config,
    workspace: &Path,
    cargo_home: &Path,
) -> Result<(Metadata, OracleGraph, PlannerReceipt), Vec<Blocker>> {
    let mut metadata = Command::new(&config.cargo);
    metadata.args(["metadata", "--format-version", "1", "--locked", "--offline"]);
    select_features(&mut metadata, config)?;
    command_env(&mut metadata, config, workspace, cargo_home);
    let metadata = command_output(&mut metadata, "cargo metadata")?;
    retain_oracle("cargo-metadata.json", &metadata)?;
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
    select_features(&mut graph, config)?;
    if config.target_triple != config.host_triple {
        graph.args(["--target", &config.target_triple]);
    }
    command_env(&mut graph, config, workspace, cargo_home);
    let graph_bytes = command_output(&mut graph, "cargo unit-graph")?;
    retain_oracle("cargo-unit-graph.json", &graph_bytes)?;
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
    select_features(&mut planner, config)?;
    if config.target_triple != config.host_triple {
        planner.args(["--target", &config.target_triple]);
    }
    command_env(&mut planner, config, workspace, cargo_home);

    let receipt_bytes = command_output(&mut planner, "rust-plan")?;
    retain_oracle("rust-plan-receipt.json", &receipt_bytes)?;
    let receipt: PlannerReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "rust-plan", error.to_string()))?;
    if receipt.schema_version != 1
        || receipt.cargo_mode.compatibility_class != "cargo-oracle-evidence"
        || receipt.unit_graph.digest_blake3 != graph_digest
        || !receipt.unit_derivation_graph.ready
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
    nar_blake3: String,
    root: String,
    links: Option<String>,
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
    let package_roots = metadata
        .packages
        .iter()
        .map(|package| {
            normalize_source_path(Path::new(&package.manifest_path))
                .and_then(|manifest| manifest.parent().map(Path::to_path_buf))
                .ok_or_else(|| block("unit-plan-source-unsupported", &package.id, "invalid package manifest path"))
        })
        .collect::<Result<Vec<_>, _>>()?;
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
        let manifest = normalize_source_path(Path::new(&package.manifest_path))
            .ok_or_else(|| block("unit-plan-source-unsupported", &package.id, "invalid package manifest path"))?;
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
        source_nar::copy_package_slice(package_root, &copied, &package_roots)
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
            nar_blake3: digest.clone(),
        });
        let kind = if package.source.is_some() { "registry" } else { "path" };
        package_bindings.insert(package.id.clone(), PackageSource {
            source_id,
            kind: kind.into(),
            nar_blake3: digest,
            root: package_root.to_string_lossy().into_owned(),
            links: package.links.clone(),
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
        if word.ends_with(".rs") && word.starts_with('/') {
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
        if word == "--cfg" && unit.derivation.args.get(index + 1).is_some_and(|value| value.starts_with("feature=\"")) {
            index += 2;
            continue;
        }
        if word.starts_with("--cfg=feature=\"") {
            index += 1;
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
    for feature in graph.semantic_features().iter().collect::<BTreeSet<_>>() {
        if feature.contains('\"') || feature.as_bytes().contains(&0) || feature.len() > 128 {
            return Err(block("unit-plan-mode-unsupported", &unit.unit_id, "invalid Cargo feature name"));
        }
        args.extend(["--cfg".to_string(), format!("feature=\"{feature}\"")]);
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

/// Cargo's topology, not the target name, decides which resolver-2 host
/// dependency instance is requested. A package can have host and target
/// instances with different features even when both triples happen to agree.
fn host_dependencies(graph: &OracleGraph) -> Result<BTreeSet<usize>, Vec<Blocker>> {
    let mut pending = graph
        .units
        .iter()
        .enumerate()
        .filter(|(_, unit)| {
            unit.mode == "build"
                && matches!(unit.target.kind.first().map(String::as_str), Some("custom-build" | "proc-macro"))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mut host = BTreeSet::new();
    while let Some(index) = pending.pop() {
        if !host.insert(index) {
            continue;
        }
        for edge in &graph.units[index].dependencies {
            let child = graph.units.get(edge.index).ok_or_else(|| {
                block("unit-plan-oracle-failure", &graph.units[index].pkg_id, "dependency index outside graph")
            })?;
            if child.mode == "build" {
                pending.push(edge.index);
            }
        }
    }
    Ok(host)
}

fn planned_features(unit: &PlannerUnit) -> BTreeSet<String> {
    unit.derivation
        .args
        .windows(2)
        .filter(|pair| pair[0] == "--cfg")
        .filter_map(|pair| pair[1].strip_prefix("feature=\"").and_then(|feature| feature.strip_suffix('\"')))
        .map(ToString::to_string)
        .collect()
}

fn match_planner<'a>(
    unit: &GraphUnit,
    role: &str,
    receipt: &'a PlannerReceipt,
) -> Result<&'a PlannerUnit, Vec<Blocker>> {
    let candidates = receipt
        .unit_derivation_graph
        .derivations
        .iter()
        .filter(|candidate| {
            candidate.package_id == unit.pkg_id
                && candidate.target_name.replace('-', "_") == unit.target.name.replace('-', "_")
                && unit.target.kind.iter().any(|kind| kind == &candidate.target_kind)
                && candidate.execution_kind == role
        })
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [only] => Ok(*only), // Cargo's features replace planner features at the rustc boundary.
        [] => Err(block("unit-plan-oracle-failure", &unit.pkg_id, format!("missing Rust-plan {role} unit"))),
        _ => {
            let features = unit.semantic_features().iter().cloned().collect::<BTreeSet<_>>();
            let exact = candidates
                .iter()
                .copied()
                .filter(|candidate| planned_features(candidate) == features)
                .collect::<Vec<_>>();
            if let [only] = exact.as_slice() {
                return Ok(*only);
            }
            // Rust-plan can retain duplicate host instances with unified features.
            // Cargo's graph owns feature selection and dependency edges; the
            // remaining planner arguments and package facts must still agree.
            let first = candidates[0];
            let args = normalize_args(first, unit)?;
            for candidate in candidates.iter().skip(1) {
                let first_package_facts = first.derivation.env.iter().filter(|(key, _)| key.starts_with("CARGO_PKG_"));
                let candidate_package_facts =
                    candidate.derivation.env.iter().filter(|(key, _)| key.starts_with("CARGO_PKG_"));
                if normalize_args(candidate, unit)? != args || !first_package_facts.eq(candidate_package_facts) {
                    return Err(block(
                        "unit-plan-mode-unsupported",
                        &unit.pkg_id,
                        "ambiguous Rust-plan target identity",
                    ));
                }
            }
            Ok(*candidates.iter().min_by_key(|candidate| candidate.unit_id.as_str()).unwrap())
        }
    }
}

#[derive(Serialize)]
struct UnitIdentity {
    core_unit_id: String,
    package_id: String,
    target_name: String,
    target_kind: String,
    execution_kind: String,
    source_id: String,
    triple: String,
    features: Vec<String>,
}

struct PlannedUnitFacts {
    facts: Vec<ExistingUnitFacts>,
    bindings: Vec<UnitBinding>,
    identities: BTreeMap<String, UnitIdentity>,
    wait_units: BTreeMap<String, WaitUnitIdentity>,
    roots: Vec<String>,
}

fn is_run(unit: &GraphUnit) -> bool {
    unit.mode == "run-custom-build"
}

fn graph_kind(unit: &GraphUnit) -> Result<&str, Vec<Blocker>> {
    unit.target
        .kind
        .first()
        .map(String::as_str)
        .ok_or_else(|| block("unit-plan-mode-unsupported", &unit.pkg_id, "empty target kind"))
}

fn compile_execution_kind(unit: &GraphUnit, host_dependency: bool) -> &'static str {
    if matches!(unit.target.kind.first().map(String::as_str), Some("proc-macro" | "custom-build")) {
        "host"
    } else if host_dependency {
        "host-dependency"
    } else {
        "target"
    }
}

fn script_compiler_index(graph: &OracleGraph, index: usize) -> Result<usize, Vec<Blocker>> {
    let run = &graph.units[index];
    let mut matches = run.dependencies.iter().filter(|edge| {
        graph.units.get(edge.index).is_some_and(|unit| {
            unit.pkg_id == run.pkg_id
                && unit.mode == "build"
                && unit.target.kind.iter().any(|kind| kind == "custom-build")
        })
    });
    let found = matches.next().ok_or_else(|| {
        block("unit-plan-build-script-run-unsupported", &run.pkg_id, "missing compiled script dependency")
    })?;
    if matches.next().is_some() {
        return Err(block(
            "unit-plan-build-script-run-unsupported",
            &run.pkg_id,
            "ambiguous compiled script dependency",
        ));
    }
    Ok(found.index)
}

fn select_planned<'a>(graph: &OracleGraph, receipt: &'a PlannerReceipt) -> Result<Vec<&'a PlannerUnit>, Vec<Blocker>> {
    let host = host_dependencies(graph)?;
    let mut selected = Vec::with_capacity(graph.units.len());
    for (index, unit) in graph.units.iter().enumerate() {
        let kind = graph_kind(unit)?;
        if is_run(unit) && kind != "custom-build" {
            return Err(block(
                "unit-plan-build-script-run-unsupported",
                &unit.pkg_id,
                "run of a non build-script target",
            ));
        }
        if !is_run(unit) && unit.mode != "build" {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, &unit.mode));
        }
        if !matches!(kind, "lib" | "bin" | "proc-macro" | "custom-build") {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, kind));
        }
        let compiled = if is_run(unit) {
            script_compiler_index(graph, index)?
        } else {
            index
        };
        let candidate = &graph.units[compiled];
        let role = compile_execution_kind(candidate, host.contains(&compiled));
        selected.push(match_planner(candidate, role, receipt)?);
    }
    Ok(selected)
}

/// Distinct Cargo feature/role instances must retain distinct compiler unit
/// identities even when rust-plan currently reports a unified unit id.
fn graph_unit_ids(graph: &OracleGraph, selected: &[&PlannerUnit]) -> Result<Vec<String>, Vec<Blocker>> {
    let mut counts = BTreeMap::<&str, u32>::new();
    for (unit, planned) in graph.units.iter().zip(selected) {
        if !is_run(unit) {
            *counts.entry(planned.unit_id.as_str()).or_default() += 1;
        }
    }
    let compiled = graph
        .units
        .iter()
        .zip(selected)
        .enumerate()
        .map(|(index, (unit, planned))| {
            if !is_run(unit) && counts[planned.unit_id.as_str()] > 1 {
                format!("{}:graph:{index}", planned.unit_id)
            } else {
                planned.unit_id.clone()
            }
        })
        .collect::<Vec<_>>();
    let mut run_counts = BTreeMap::<usize, u32>::new();
    for (index, unit) in graph.units.iter().enumerate() {
        if is_run(unit) {
            *run_counts.entry(script_compiler_index(graph, index)?).or_default() += 1;
        }
    }
    graph
        .units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            if is_run(unit) {
                let compile = script_compiler_index(graph, index)?;
                let suffix = if run_counts[&compile] > 1 {
                    format!(":{index}")
                } else {
                    String::new()
                };
                Ok(format!("{}:{BUILD_SCRIPT_RUN}{suffix}", compiled[compile]))
            } else {
                Ok(compiled[index].clone())
            }
        })
        .collect()
}

/// A build script's `TARGET` is the triple of the compilation that consumes it.
fn unit_triple(config: &Config, graph: &OracleGraph, selected: &[&PlannerUnit], index: usize) -> String {
    let consumer = if is_run(&graph.units[index]) {
        graph
            .units
            .iter()
            .enumerate()
            .find(|(_, unit)| !is_run(unit) && unit.dependencies.iter().any(|edge| edge.index == index))
            .map(|(index, _)| index)
    } else {
        Some(index)
    };
    if consumer.is_some_and(|index| matches!(selected[index].execution_kind.as_str(), "host" | "host-dependency")) {
        config.host_triple.clone()
    } else {
        config.target_triple.clone()
    }
}

fn feature_env_name(feature: &str) -> String {
    let name = feature
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("CARGO_FEATURE_{name}")
}

/// Cargo package facts for compilation, or the declared build-script environment.
fn unit_environment(
    config: &Config,
    unit: &GraphUnit,
    planned: &PlannerUnit,
    source_id: &str,
) -> Result<Vec<(String, String)>, Vec<Blocker>> {
    let run = is_run(unit);
    let mut env = Vec::new();
    for (key, value) in &planned.derivation.env {
        let is_package_fact = key.starts_with("CARGO_PKG_");
        if is_package_fact {
            if value.starts_with('/') || value.contains("=/") || value.contains('\0') {
                return Err(block("unit-plan-host-path", &planned.unit_id, key));
            }
            env.push((key.clone(), value.clone()));
        }
    }
    env.push(("CARGO_MANIFEST_DIR".into(), format!("{{{{mantle-source:{source_id}}}}}")));
    env.extend(
        unit.semantic_features()
            .iter()
            .map(|feature| (feature_env_name(feature), "1".to_string()))
            .collect::<BTreeMap<_, _>>(),
    );
    if run {
        let profile = unit.profile.as_ref().ok_or_else(|| {
            block("unit-plan-mode-unsupported", &planned.unit_id, "missing Cargo build-script profile")
        })?;
        if profile.name != config.profile
            || profile.opt_level.len() > 8
            || !profile.opt_level.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(block("unit-plan-mode-unsupported", &planned.unit_id, "invalid Cargo build-script profile"));
        }
        env.push(("PROFILE".into(), if config.profile == "dev" { "debug" } else { "release" }.into()));
        env.push(("OPT_LEVEL".into(), profile.opt_level.clone()));
        env.push(("DEBUG".into(), (profile.debuginfo.unwrap_or(0) > 0).to_string()));
        env.push(("NUM_JOBS".into(), "1".into()));
    }
    Ok(env)
}

/// Direct producer edges. Execution units are never rustc externs.
fn unit_edges(
    graph: &OracleGraph,
    ids: &[String],
    index: usize,
) -> Result<(Vec<String>, Vec<ExternBinding>), Vec<Blocker>> {
    let unit = &graph.units[index];
    let mut dependencies = Vec::with_capacity(unit.dependencies.len());
    let mut externs = Vec::new();
    for edge in &unit.dependencies {
        let dependency = ids
            .get(edge.index)
            .ok_or_else(|| block("unit-plan-oracle-failure", &ids[index], "dependency index outside graph"))?;
        dependencies.push(dependency.clone());
        let is_compile_edge = !is_run(unit) && !is_run(&graph.units[edge.index]);
        if let Some(name) = edge.extern_crate_name.as_ref().filter(|_| is_compile_edge) {
            externs.push(ExternBinding {
                name: name.clone(),
                unit_id: dependency.clone(),
            });
        }
    }
    Ok((dependencies, externs))
}

struct FactsInputs<'a> {
    config: &'a Config,
    graph: &'a OracleGraph,
    selected: &'a [&'a PlannerUnit],
    ids: &'a [String],
    package_sources: &'a BTreeMap<String, PackageSource>,
    native: &'a BTreeMap<String, ResolvedNativeInputs>,
}

fn unit_facts(inputs: &FactsInputs<'_>, index: usize) -> Result<(ExistingUnitFacts, UnitBinding), Vec<Blocker>> {
    let unit = &inputs.graph.units[index];
    let planned = inputs.selected[index];
    let unit_id = &inputs.ids[index];
    let source = inputs
        .package_sources
        .get(&unit.pkg_id)
        .ok_or_else(|| block("unit-plan-source-unsupported", &unit.pkg_id, "missing package slice"))?;
    let source_path = normalize_source_path(Path::new(&unit.target.src_path)).ok_or_else(|| {
        block("unit-plan-source-unsupported", &unit.pkg_id, "source path escapes the filesystem root")
    })?;
    let entry = source_path
        .strip_prefix(&source.root)
        .map_err(|_| block("unit-plan-source-unsupported", &unit.pkg_id, "source path outside package"))?
        .to_str()
        .ok_or_else(|| block("unit-plan-source-unsupported", &unit.pkg_id, "non-UTF8 source entry"))?;
    let (dependencies, externs) = unit_edges(inputs.graph, inputs.ids, index)?;
    let execution_kind = if is_run(unit) {
        BUILD_SCRIPT_RUN
    } else {
        planned.execution_kind.as_str()
    };
    let (arguments, expected_outputs) = if is_run(unit) {
        (vec![RUN_BUILD_SCRIPT_MODE.to_string()], vec!["out".to_string(), "flags".to_string()])
    } else {
        (normalize_args(planned, unit)?, vec!["out".to_string()])
    };
    let build_script_run = is_run(unit).then(|| {
        let native = inputs.native.get(&unit.pkg_id).cloned().unwrap_or_default();
        BuildScriptRunBinding {
            links: source.links.clone(),
            native_inputs: native.inputs,
            native_env: native.env,
            native_path: native.path,
        }
    });
    let facts = ExistingUnitFacts {
        unit_id: unit_id.clone(),
        package_id: unit.pkg_id.clone(),
        target_name: unit.target.name.clone(),
        target_kind: unit.target.kind[0].clone(),
        execution_kind: execution_kind.into(),
        dependency_unit_ids: dependencies,
        arguments,
        environment: unit_environment(inputs.config, unit, planned, &source.source_id)?,
        input_identities: vec![format!("source:{}", source.source_id)],
        expected_outputs,
        execution_order: u32::try_from(index).map_err(|_| block("unit-plan-limit", "units", "unit index overflow"))?,
    };
    let binding = UnitBinding {
        unit_id: unit_id.clone(),
        source_id: source.source_id.clone(),
        source_kind: source.kind.clone(),
        source_entry: entry.into(),
        source_label: format!("{}-{}", unit.target.name.replace('_', "-"), "source"),
        rustc_metadata_hash: if unit_id.contains(":graph:") {
            blake3::hash(format!("{}:{unit_id}", planned.rustc_metadata_hash).as_bytes()).to_hex().to_string()
        } else {
            planned.rustc_metadata_hash.clone()
        },
        triple: unit_triple(inputs.config, inputs.graph, inputs.selected, index),
        externs,
        build_script_run,
    };
    Ok((facts, binding))
}
fn build_facts(
    config: &Config,
    graph: &OracleGraph,
    receipt: &PlannerReceipt,
    package_sources: &BTreeMap<String, PackageSource>,
    native: &BTreeMap<String, ResolvedNativeInputs>,
) -> Result<PlannedUnitFacts, Vec<Blocker>> {
    let selected = select_planned(graph, receipt)?;
    let ids = graph_unit_ids(graph, &selected)?;
    let inputs = FactsInputs {
        config,
        graph,
        selected: &selected,
        ids: &ids,
        package_sources,
        native,
    };
    let mut facts = Vec::with_capacity(graph.units.len());
    let mut bindings = Vec::with_capacity(graph.units.len());
    let mut identities = BTreeMap::new();
    for index in 0..graph.units.len() {
        let (fact, binding) = unit_facts(&inputs, index)?;
        identities.insert(mantle_cargo_unit_plan::plan_id(&fact.unit_id), UnitIdentity {
            core_unit_id: fact.unit_id.clone(),
            package_id: fact.package_id.clone(),
            target_name: fact.target_name.clone(),
            target_kind: fact.target_kind.clone(),
            execution_kind: fact.execution_kind.clone(),
            source_id: binding.source_id.clone(),
            triple: binding.triple.clone(),
            features: graph.units[index]
                .semantic_features()
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        });
        facts.push(fact);
        bindings.push(binding);
    }
    let roots = graph
        .roots
        .iter()
        .map(|index| {
            ids.get(*index)
                .cloned()
                .ok_or_else(|| block("unit-plan-oracle-failure", "roots", "root index outside graph"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PlannedUnitFacts {
        // Optional observation metadata cannot veto an otherwise admitted semantic plan.
        wait_units: wait_units(config, graph, &facts, &bindings, package_sources).unwrap_or_default(),
        facts,
        bindings,
        roots,
        identities,
    })
}

/// Domain separation keeps opaque semantic facts distinct without retaining argv or paths.
fn wait_digest<T: Serialize>(role: &str, value: &T) -> Result<Digest, Vec<Blocker>> {
    mantle_rust_plan_core::domain_digest(role.as_bytes(), value)
        .map_err(|_| block("unit-plan-wait-identity-invalid", role, "semantic identity serialization"))
}

/// Traverse only execution-only nodes. Compiler nodes carry their own dependency closure.
fn wait_dependencies(
    graph: &OracleGraph,
    index: usize,
    digests: &[Option<Digest>],
) -> Result<Vec<Dependency>, Vec<Blocker>> {
    let mut pending = graph.units[index].dependencies.iter().map(|edge| edge.index).collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut dependencies = BTreeMap::new();
    while let Some(next) = pending.pop() {
        if !seen.insert(next) {
            continue;
        }
        let unit = graph
            .units
            .get(next)
            .ok_or_else(|| block("unit-plan-oracle-failure", "wait dependencies", "dependency outside graph"))?;
        if is_run(unit) {
            pending.extend(unit.dependencies.iter().map(|edge| edge.index));
            continue;
        }
        let digest = digests[next]
            .as_ref()
            .ok_or_else(|| block("unit-plan-wait-identity-invalid", &unit.pkg_id, "unresolved compiler dependency"))?;
        let role = if matches!(graph_kind(unit)?, "custom-build" | "proc-macro") {
            ArtifactRole::HostExecutable
        } else {
            ArtifactRole::Linkable
        };
        dependencies.insert((digest.clone(), role), ());
        if dependencies.len() > mantle_rust_differential_core::waits::MAX_PREREQUISITES {
            return Err(block("unit-plan-limit", &unit.pkg_id, "more than 256 compiler prerequisites"));
        }
    }
    Ok(dependencies.into_keys().map(|(unit, role)| Dependency { unit, role }).collect())
}

fn wait_input_digest(
    config: &Config,
    fact: &ExistingUnitFacts,
    binding: &UnitBinding,
    source: &PackageSource,
    dependencies: &[&Digest],
) -> Result<Digest, Vec<Blocker>> {
    let linker = config.target_linker.as_ref().filter(|_| fact.execution_kind == "target");
    wait_digest(
        "mantle-cargo-wait-inputs-v1",
        &(
            &source.nar_blake3,
            &fact.input_identities,
            &fact.environment,
            &fact.expected_outputs,
            &binding.source_entry,
            &binding.source_label,
            &binding.source_kind,
            &binding.rustc_metadata_hash,
            &binding.externs,
            &binding.build_script_run,
            dependencies,
            (&config.helper, &config.linker, linker, &config.runtime_inputs),
        ),
    )
}

fn wait_compiler_identity(
    config: &Config,
    unit: &GraphUnit,
    fact: &ExistingUnitFacts,
    binding: &UnitBinding,
    source: &PackageSource,
    inputs: Digest,
    dependencies: Vec<Dependency>,
) -> Result<WaitUnitIdentity, Vec<Blocker>> {
    let profile = unit
        .profile
        .as_ref()
        .ok_or_else(|| block("unit-plan-wait-identity-missing", &fact.unit_id, "missing Cargo compiler profile"))?;
    if profile.name.is_empty() || profile.opt_level.is_empty() || binding.rustc_metadata_hash.is_empty() {
        return Err(block("unit-plan-wait-identity-missing", &fact.unit_id, "incomplete compiler metadata"));
    }
    let role = match fact.execution_kind.as_str() {
        "host" | "host-dependency" => ProcessRole::HostCompiler,
        "target" => ProcessRole::TargetCompiler,
        _ => return Err(block("unit-plan-mode-unsupported", &fact.unit_id, "not a compiler execution")),
    };
    let observed_features = unit
        .features
        .as_ref()
        .ok_or_else(|| block("unit-plan-wait-identity-missing", &fact.unit_id, "missing Cargo feature metadata"))?;
    let mut features = observed_features
        .iter()
        .map(|feature| wait_digest("mantle-cargo-wait-feature-v1", feature))
        .collect::<Result<Vec<_>, _>>()?;
    features.sort();
    features.dedup();
    let identity = WaitUnitIdentity {
        package: wait_digest("mantle-cargo-wait-package-v1", &fact.package_id)?,
        source: wait_digest("mantle-cargo-wait-source-nar-v1", &source.nar_blake3)?,
        target: wait_digest("mantle-cargo-wait-target-v1", &(&fact.target_name, &binding.source_entry))?,
        target_kind: wait_digest("mantle-cargo-wait-target-kind-v1", &unit.target.kind)?,
        host: wait_digest("mantle-cargo-wait-host-v1", &config.host_triple)?,
        target_triple: wait_digest("mantle-cargo-wait-triple-v1", &binding.triple)?,
        role,
        features,
        profile: wait_digest("mantle-cargo-wait-profile-v1", profile)?,
        mode: wait_digest("mantle-cargo-wait-mode-v1", &unit.mode)?,
        compiler: wait_digest("mantle-cargo-wait-compiler-v1", &(&config.rustc, &config.toolchain))?,
        arguments: wait_digest("mantle-cargo-wait-arguments-v1", &fact.arguments)?,
        inputs,
        dependencies,
    };
    Ok(identity)
}

/// Kahn reduction includes build-script runs in input identities, never in compiler inventory.
/// All graph storage is bounded by the existing unit and per-unit prerequisite ceilings.
fn wait_units(
    config: &Config,
    graph: &OracleGraph,
    facts: &[ExistingUnitFacts],
    bindings: &[UnitBinding],
    sources: &BTreeMap<String, PackageSource>,
) -> Result<BTreeMap<String, WaitUnitIdentity>, Vec<Blocker>> {
    let mut remaining = Vec::with_capacity(graph.units.len());
    let mut consumers = vec![Vec::new(); graph.units.len()];
    for (index, unit) in graph.units.iter().enumerate() {
        let features = unit
            .features
            .as_ref()
            .ok_or_else(|| block("unit-plan-wait-identity-missing", &unit.pkg_id, "missing Cargo feature metadata"))?;
        if unit.dependencies.len() > mantle_cargo_unit_plan::MAX_INPUTS_PER_UNIT
            || features.len() > mantle_rust_differential_core::waits::MAX_PREREQUISITES
        {
            return Err(block("unit-plan-limit", &unit.pkg_id, "more than 256 prerequisites or features"));
        }
        remaining.push(unit.dependencies.len());
        for edge in &unit.dependencies {
            consumers
                .get_mut(edge.index)
                .ok_or_else(|| block("unit-plan-oracle-failure", &unit.pkg_id, "dependency outside graph"))?
                .push(index);
        }
    }
    let mut ready =
        remaining.iter().enumerate().filter_map(|(i, count)| (*count == 0).then_some(i)).collect::<Vec<_>>();
    let mut digests = vec![None; graph.units.len()];
    let mut inventory = BTreeMap::new();
    let mut completed = 0;
    while let Some(index) = ready.pop() {
        let fact = &facts[index];
        let unit = &graph.units[index];
        let source = sources
            .get(&unit.pkg_id)
            .ok_or_else(|| block("unit-plan-source-unsupported", &unit.pkg_id, "missing NAR identity"))?;
        let mut inputs = unit
            .dependencies
            .iter()
            .map(|edge| {
                digests[edge.index].as_ref().ok_or_else(|| {
                    block("unit-plan-wait-identity-invalid", &fact.unit_id, "unresolved dependency identity")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        inputs.sort();
        inputs.dedup();
        let input = wait_input_digest(config, fact, &bindings[index], source, &inputs)?;
        let digest = if is_run(unit) {
            wait_digest(
                "mantle-cargo-wait-build-script-run-v1",
                &(
                    input,
                    unit.semantic_features().iter().collect::<BTreeSet<_>>(),
                    &unit.profile,
                    &unit.mode,
                    &bindings[index].triple,
                ),
            )?
        } else {
            let dependencies = wait_dependencies(graph, index, &digests)?;
            let identity = wait_compiler_identity(config, unit, fact, &bindings[index], source, input, dependencies)?;
            let digest = identity
                .identity()
                .map_err(|_| block("unit-plan-wait-identity-invalid", &fact.unit_id, "invalid compiler identity"))?;
            if inventory.insert(fact.unit_id.clone(), identity).is_some() {
                return Err(block("unit-plan-wait-identity-invalid", &fact.unit_id, "duplicate compiler unit"));
            }
            digest
        };
        digests[index] = Some(digest);
        completed += 1;
        for consumer in &consumers[index] {
            remaining[*consumer] -= 1;
            if remaining[*consumer] == 0 {
                ready.push(*consumer);
            }
        }
    }
    if completed != graph.units.len() {
        return Err(block(
            "unit-plan-wait-identity-invalid",
            "unit-graph",
            "cyclic compiler or build-script dependencies",
        ));
    }
    Ok(inventory)
}

fn is_store_root(path: &str, prefix: &str) -> bool {
    store_root(path, prefix) && path.strip_prefix(prefix).is_some_and(|rest| rest.matches('/').count() == 1)
}

/// Resolve role markers per declaration and bind only an exact Cargo package id.
// r[impl mantle.rust_unit_plan.build_scripts.native_inputs]
fn native_bindings(config: &Config) -> Result<BTreeMap<String, ResolvedNativeInputs>, Vec<Blocker>> {
    let mut bound = BTreeMap::new();
    for (key, declaration) in &config.native_inputs {
        let resolved =
            resolve_native_inputs(declaration).map_err(|error| block(error.code(), key, error.to_string()))?;
        if let Some(input) = resolved.inputs.iter().find(|input| !is_store_root(input, &config.store_prefix)) {
            return Err(block("unit-plan-host-path", key, input));
        }
        bound.insert(key.clone(), resolved);
    }
    Ok(bound)
}

fn check_native_packages(
    graph: &OracleGraph,
    metadata: &Metadata,
    bound: &BTreeMap<String, ResolvedNativeInputs>,
) -> Result<(), Vec<Blocker>> {
    let scripted = graph
        .units
        .iter()
        .filter(|unit| is_run(unit))
        .map(|unit| unit.pkg_id.as_str())
        .collect::<BTreeSet<_>>();
    for key in bound.keys() {
        let count = metadata
            .packages
            .iter()
            .filter(|package| &package.id == key && scripted.contains(package.id.as_str()))
            .take(2)
            .count();
        if count != 1 {
            return Err(block(
                "native-input-invalid",
                key,
                "declaration must name exactly one package with a build script",
            ));
        }
    }
    Ok(())
}

/// The evaluated Nickel declaration (whose role paths are producer inputs)
/// must be the configuration the producer resolves.
fn check_declared_native_inputs(config: &Config, declared: Option<&str>) -> Result<(), Vec<Blocker>> {
    let Some(declared) = declared else {
        if config.native_inputs.is_empty() {
            return Ok(());
        }
        return Err(block("native-input-invalid", NATIVE_INPUTS_ENV, "native inputs are not declared producer inputs"));
    };
    let parsed: BTreeMap<String, NativeInputDeclaration> = serde_json::from_str(declared)
        .map_err(|error| block("native-input-invalid", NATIVE_INPUTS_ENV, error.to_string()))?;
    if parsed != config.native_inputs {
        return Err(block("native-input-invalid", NATIVE_INPUTS_ENV, "declaration differs from the producer config"));
    }
    Ok(())
}

fn check_declared_target_linker(config: &Config, declared: Option<&str>) -> Result<(), Vec<Blocker>> {
    let parsed: Option<TargetLinkerBinding> = match declared {
        Some(value) => serde_json::from_str(value)
            .map_err(|error| block("unit-plan-host-path", "target_linker", error.to_string()))?,
        None => None,
    };
    if parsed != config.target_linker {
        return Err(block(
            "unit-plan-host-path",
            "target_linker",
            "config and Nickel target linker declarations differ",
        ));
    }
    Ok(())
}

/// rust-plan discovers vendored sources only from the workspace Cargo config,
/// so the staged workspace declares the vendor directory there.
fn stage_vendor_config(workspace: &Path, vendor: Option<&Path>) -> Result<(), Vec<Blocker>> {
    let Some(vendor) = vendor else {
        return Ok(());
    };
    let cargo_dir = workspace.join(".cargo");
    if cargo_dir.join("config.toml").exists() || cargo_dir.join("config").exists() {
        return Err(block(
            "unit-plan-source-unsupported",
            "workspace",
            "workspace Cargo config conflicts with the declared vendor directory",
        ));
    }
    let writable = |path: &Path| -> Result<(), Vec<Blocker>> {
        let mut permissions = fs::metadata(path)
            .map_err(|error| block("unit-plan-oracle-failure", "vendor", error.to_string()))?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o200);
        fs::set_permissions(path, permissions)
            .map_err(|error| block("unit-plan-oracle-failure", "vendor", error.to_string()))
    };
    if !cargo_dir.exists() {
        writable(workspace)?;
    }
    fs::create_dir_all(&cargo_dir).map_err(|error| block("unit-plan-oracle-failure", "vendor", error.to_string()))?;
    writable(&cargo_dir)?;
    fs::write(
        cargo_dir.join("config.toml"),
        format!("[source.crates-io]\nreplace-with = \"declared-vendor\"\n\n[source.declared-vendor]\ndirectory = {vendor:?}\n"),
    )
    .map_err(|error| block("unit-plan-oracle-failure", "vendor", error.to_string()))
}

fn run() -> Result<(), Vec<Blocker>> {
    let config: Config = serde_json::from_slice(
        &fs::read(required("MANTLE_UNIT_CONFIG")?)
            .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?,
    )
    .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?;
    let declared_runtime: Vec<String> = serde_json::from_str(&required("MANTLE_UNIT_RUNTIME_INPUTS")?)
        .map_err(|error| block("unit-plan-host-path", "runtime_inputs", error.to_string()))?;
    if config.runtime_inputs != declared_runtime {
        return Err(block("unit-plan-host-path", "runtime_inputs", "config and Nickel runtime declarations differ"));
    }
    check_declared_native_inputs(&config, env::var(NATIVE_INPUTS_ENV).ok().as_deref())?;
    check_declared_target_linker(&config, env::var("MANTLE_UNIT_TARGET_LINKER").ok().as_deref())?;
    let native = native_bindings(&config)?;

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
    stage_vendor_config(&workspace, vendor)?;
    let lock_before = fs::read(workspace.join("Cargo.lock"))
        .map_err(|error| block("unit-plan-oracle-failure", "Cargo.lock", error.to_string()))?;
    let (metadata, graph, receipt) = run_oracle(&config, &workspace, &cargo_home)?;
    check_native_packages(&graph, &metadata, &native)?;
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
        identities,
        wait_units,
    } = build_facts(&config, &graph, &receipt, &package_bindings, &native)?;
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
    let mut oracle_inputs = [&config.cargo, &config.toolchain]
        .into_iter()
        .map(|path| {
            let name = path
                .strip_prefix(&config.store_prefix)
                .and_then(|suffix| suffix.strip_prefix('/'))
                .and_then(|suffix| suffix.split('/').next())
                .ok_or_else(|| block("unit-plan-host-path", "oracle_inputs", path))?;
            Ok::<_, Vec<Blocker>>(format!("{}/{name}", config.store_prefix))
        })
        .collect::<Result<Vec<_>, _>>()?;
    oracle_inputs.sort();
    oracle_inputs.dedup();
    let bindings = PlanBindings {
        store_prefix: config.store_prefix,
        system: config.system,
        target_triple: config.target_triple,
        host_triple: config.host_triple,
        helper: config.helper,
        rustc: config.rustc,
        linker: config.linker,
        target_linker: config.target_linker,
        toolchain: config.toolchain,
        oracle_inputs,
        runtime_inputs: config.runtime_inputs,
        wait_units,
        sources,
        units,
        roots,
    };
    let (bytes, digest) = lower(&effects, &bindings)?;
    fs::write(required("plan")?, &bytes)
        .map_err(|error| block("unit-plan-oracle-failure", "plan", error.to_string()))?;
    let out = PathBuf::from(required("out")?);
    fs::create_dir_all(&out).map_err(|error| block("unit-plan-oracle-failure", "evidence", error.to_string()))?;
    let identity_record = serde_json::json!({"schema": "mantle-cargo-unit-bindings-v1", "units": identities, "roots": bindings.roots.iter().map(|id| mantle_cargo_unit_plan::plan_id(id)).collect::<Vec<_>>()});
    fs::write(
        out.join("cargo-unit-bindings.json"),
        serde_json::to_vec(&identity_record)
            .map_err(|error| block("unit-plan-oracle-failure", "bindings", error.to_string()))?,
    )
    .map_err(|error| block("unit-plan-oracle-failure", "bindings", error.to_string()))?;
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
    use std::fs;

    use serde_json::json;

    use super::Config;
    use super::GraphUnit;
    use super::Metadata;
    use super::OracleGraph;
    use super::PlannerReceipt;
    use super::PlannerUnit;
    use super::check_declared_native_inputs;
    use super::check_native_packages;
    use super::graph_unit_ids;
    use super::match_planner;
    use super::native_bindings;
    use super::normalize_args;
    use super::normalize_source_path;
    use super::parse_unit_graph;
    use super::select_planned;
    use super::stage_vendor_config;

    #[test]
    fn path_dependency_sources_are_normalized_without_escaping_the_root() {
        let normalized = normalize_source_path(std::path::Path::new("/w/app/../bar/./build.rs"));
        assert_eq!(normalized.as_deref(), Some(std::path::Path::new("/w/bar/build.rs")));
        assert_eq!(normalize_source_path(std::path::Path::new("/../etc/passwd")), None);
        assert_eq!(normalize_source_path(std::path::Path::new("relative/build.rs")), None);
    }

    const ZLIB: &str = "/nix/store/00000000000000000000000000000000-zlib-1.3";

    fn config(native_inputs: serde_json::Value) -> Config {
        serde_json::from_value(json!({
            "workspace": "/nix/store/11111111111111111111111111111111-workspace",
            "vendor": null,
            "planner": "/nix/store/22222222222222222222222222222222-mantle/bin/mantle",
            "cargo": "/nix/store/33333333333333333333333333333333-rust/bin/cargo",
            "rustc": "/nix/store/33333333333333333333333333333333-rust/bin/rustc",
            "linker": "/nix/store/44444444444444444444444444444444-cc/bin/cc",
            "toolchain": "/nix/store/33333333333333333333333333333333-rust",
            "helper": "/nix/store/55555555555555555555555555555555-helper/bin/helper",
            "runtime_inputs": [],
            "store_prefix": "/nix/store",
            "system": "x86_64-linux",
            "host_triple": "x86_64-unknown-linux-gnu",
            "target_triple": "x86_64-unknown-linux-gnu",
            "profile": "dev",
            "native_inputs": native_inputs,
        }))
        .unwrap()
    }

    fn graph_and_metadata() -> (OracleGraph, Metadata) {
        let target = |kind: &str| json!({"kind": [kind], "name": "build-script-build", "src_path": "/w/z-sys/build.rs", "edition": "2021"});
        let graph = serde_json::from_value(json!({
            "version": 1,
            "units": [
                {"pkg_id": "z-sys 0.1.0", "target": target("custom-build"), "mode": "build", "dependencies": []},
                {"pkg_id": "z-sys 0.1.0", "target": target("custom-build"), "mode": "run-custom-build",
                 "dependencies": [{"index": 0, "extern_crate_name": "build_script_build"}], "features": ["static"]},
                {"pkg_id": "app 0.1.0", "target": target("bin"), "mode": "build", "dependencies": []},
            ],
            "roots": [2],
        }))
        .unwrap();
        let metadata = serde_json::from_value(json!({"packages": [
            {"id": "z-sys 0.1.0", "manifest_path": "/w/z-sys/Cargo.toml", "name": "z-sys", "version": "0.1.0", "source": null, "links": "z"},
            {"id": "app 0.1.0", "manifest_path": "/w/app/Cargo.toml", "name": "app", "version": "0.1.0", "source": null},
        ]}))
        .unwrap();
        (graph, metadata)
    }

    #[test]
    fn native_roles_resolve_to_declared_store_roots_for_the_named_package() {
        let (graph, metadata) = graph_and_metadata();
        let config = config(json!({"z-sys 0.1.0": {
            "inputs": {"zlib": ZLIB},
            "env": {"PKG_CONFIG_PATH": "{{role:zlib}}/lib/pkgconfig"},
            "path": [],
        }}));
        let bound = native_bindings(&config).unwrap();
        check_native_packages(&graph, &metadata, &bound).unwrap();
        assert_eq!(bound.len(), 1);
        assert_eq!(bound["z-sys 0.1.0"].inputs, vec![ZLIB.to_string()]);
        assert_eq!(bound["z-sys 0.1.0"].env["PKG_CONFIG_PATH"], format!("{ZLIB}/lib/pkgconfig"));
    }

    #[test]
    fn undeclared_role_marker_fails_with_a_stable_code() {
        let config = config(json!({"z-sys 0.1.0": {"inputs": {"zlib": ZLIB}, "env": {"ZLIB_DIR": "{{role:libz}}"}}}));
        let blockers = native_bindings(&config).unwrap_err();
        assert_eq!(blockers[0].code, "native-input-role-undeclared");
        assert!(blockers[0].detail.contains("libz"));
    }

    #[test]
    fn native_declarations_reject_host_paths_and_unknown_packages() {
        let (graph, metadata) = graph_and_metadata();
        let subpath = config(json!({"z-sys 0.1.0": {"inputs": {"zlib": format!("{ZLIB}/lib")}}}));
        assert_eq!(native_bindings(&subpath).unwrap_err()[0].code, "unit-plan-host-path");
        let host = config(json!({"z-sys 0.1.0": {"inputs": {"zlib": "/usr/lib"}}}));
        assert_eq!(native_bindings(&host).unwrap_err()[0].code, "unit-plan-host-path");
        let unscripted = config(json!({"app 0.1.0": {"inputs": {"zlib": ZLIB}}}));
        assert_eq!(
            check_native_packages(&graph, &metadata, &native_bindings(&unscripted).unwrap()).unwrap_err()[0].code,
            "native-input-invalid"
        );
        let alias = config(json!({"z-sys": {"inputs": {"zlib": ZLIB}}}));
        assert_eq!(
            check_native_packages(&graph, &metadata, &native_bindings(&alias).unwrap()).unwrap_err()[0].code,
            "native-input-invalid"
        );
    }

    #[test]
    fn config_native_inputs_must_match_the_evaluated_declaration() {
        let declared = json!({"z-sys 0.1.0": {"inputs": {"zlib": ZLIB}, "env": {}, "path": []}});
        let config = config(declared.clone());
        assert!(check_declared_native_inputs(&config, Some(&declared.to_string())).is_ok());
        assert_eq!(check_declared_native_inputs(&config, None).unwrap_err()[0].code, "native-input-invalid");
        let other = json!({"z-sys 0.1.0": {"inputs": {}, "env": {}, "path": []}}).to_string();
        assert_eq!(check_declared_native_inputs(&config, Some(&other)).unwrap_err()[0].code, "native-input-invalid");
        assert!(check_declared_native_inputs(&super::tests::config(json!({})), None).is_ok());
    }

    #[test]
    fn declared_vendor_directory_is_staged_in_the_workspace_cargo_config() {
        let root = std::env::temp_dir().join(format!("mantle-unit-vendor-stage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let vendor = std::path::Path::new("/nix/store/66666666666666666666666666666666-vendor");
        stage_vendor_config(&root, Some(vendor)).unwrap();
        let staged = fs::read_to_string(root.join(".cargo/config.toml")).unwrap();
        assert!(staged.contains("replace-with = \"declared-vendor\""));
        assert!(staged.contains("directory = \"/nix/store/66666666666666666666666666666666-vendor\""));
        let conflict = stage_vendor_config(&root, Some(vendor)).unwrap_err();
        assert_eq!(conflict[0].code, "unit-plan-source-unsupported");
        let untouched = root.join("no-vendor");
        fs::create_dir_all(&untouched).unwrap();
        stage_vendor_config(&untouched, None).unwrap();
        assert!(!untouched.join(".cargo").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unknown_cargo_unit_graph_version_blocks_before_lowering() {
        let blockers = parse_unit_graph(br#"{"version":2,"units":[],"roots":[]}"#).unwrap_err();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "unit-plan-oracle-failure");
        assert_eq!(blockers[0].subject, "unit-graph");
    }
    #[test]
    fn resolver_two_host_and_target_instances_select_distinct_roles_and_features() {
        let graph: OracleGraph = serde_json::from_value(json!({
            "version": 1,
            "units": [
                {"pkg_id":"macro 1", "target":{"kind":["custom-build"],"name":"build-script-build","src_path":"/w/build.rs","edition":"2021"},
                 "mode":"build","dependencies":[{"index":1}]},
                {"pkg_id":"proc-macro2 1", "target":{"kind":["lib"],"name":"proc_macro2","src_path":"/w/src/lib.rs","edition":"2021"},
                 "mode":"build","features":["proc-macro"],"dependencies":[]},
                {"pkg_id":"app 1", "target":{"kind":["bin"],"name":"app","src_path":"/w/src/main.rs","edition":"2021"},
                 "mode":"build","dependencies":[{"index":3}]},
                {"pkg_id":"proc-macro2 1", "target":{"kind":["lib"],"name":"proc_macro2","src_path":"/w/src/lib.rs","edition":"2021"},
                 "mode":"build","features":["default","proc-macro"],"dependencies":[]}
            ],
            "roots":[0,2]
        })).unwrap();
        let receipt: PlannerReceipt = serde_json::from_value(json!({
            "schema_version":1, "cargo_mode":{"compatibility_class":"cargo-oracle-evidence"},
            "lockfile":{"blake3":"abc"}, "unit_graph":{"digest_blake3":"def"},
            "unit_derivation_graph":{"ready":true,"blockers":[],"derivations":[
                {"unit_id":"u.macro","package_id":"macro 1","target_name":"build-script-build","target_kind":"custom-build",
                 "execution_kind":"host","rustc_metadata_hash":"abc","derivation":{"args":[],"env":{}}},
                {"unit_id":"u.shared","package_id":"proc-macro2 1","target_name":"proc_macro2","target_kind":"lib",
                 "execution_kind":"host-dependency","rustc_metadata_hash":"abc","derivation":{"args":["--cfg","feature=\"default\"","--cfg","feature=\"proc-macro\""],"env":{}}},
                {"unit_id":"u.app","package_id":"app 1","target_name":"app","target_kind":"bin",
                 "execution_kind":"target","rustc_metadata_hash":"abc","derivation":{"args":[],"env":{}}},
                {"unit_id":"u.shared","package_id":"proc-macro2 1","target_name":"proc_macro2","target_kind":"lib",
                 "execution_kind":"target","rustc_metadata_hash":"abc","derivation":{"args":["--cfg","feature=\"default\"","--cfg","feature=\"proc-macro\""],"env":{}}}
            ]}
        })).unwrap();
        let selected = select_planned(&graph, &receipt).unwrap();
        assert_eq!(selected[1].execution_kind, "host-dependency");
        assert_eq!(selected[3].execution_kind, "target");
        let ids = graph_unit_ids(&graph, &selected).unwrap();
        assert_eq!(ids[1], "u.shared:graph:1");
        assert_eq!(ids[3], "u.shared:graph:3");
        assert_ne!(ids[1], ids[3]);
        let mut wrong_role = receipt;
        wrong_role.unit_derivation_graph.derivations.retain(|unit| unit.execution_kind != "host-dependency");
        assert_eq!(select_planned(&graph, &wrong_role).unwrap_err()[0].code, "unit-plan-oracle-failure");
    }

    #[test]
    fn cargo_feature_partition_replaces_unified_planner_cfg_at_rustc_boundary() {
        let fixture = std::env::temp_dir().join(format!("mantle-unit-features-{}", std::process::id()));
        fs::create_dir_all(fixture.join("src")).unwrap();
        let src = fixture.join("src/lib.rs");
        fs::write(&src, "pub fn feature() {}").unwrap();
        let graph: GraphUnit = serde_json::from_value(json!({
            "pkg_id":"proc-macro2 1",
            "target":{"kind":["lib"],"name":"proc_macro2","src_path":src.to_str().unwrap(),"edition":"2021"},
            "mode":"build","features":["proc-macro"],"dependencies":[]
        }))
        .unwrap();
        let planner: PlannerUnit = serde_json::from_value(json!({
            "unit_id":"u.proc","package_id":"proc-macro2 1","target_name":"proc_macro2",
            "target_kind":"lib","execution_kind":"host-dependency","rustc_metadata_hash":"abc",
            "derivation":{"args":["--crate-name","proc_macro2","--crate-type","rlib",
                                  "--edition","2021","--cfg","feature=\"default\"",
                                  "--cfg","feature=\"proc-macro\"",src.to_str().unwrap()],
                          "env":{"CARGO_FEATURE_DEFAULT":"1","CARGO_FEATURE_PROC_MACRO":"1"}}
        }))
        .unwrap();
        let args = normalize_args(&planner, &graph).unwrap();
        assert!(args.windows(2).any(|pair| pair[0] == "--cfg" && pair[1] == "feature=\"proc-macro\""));
        assert!(!args.iter().any(|arg| arg.contains("feature=\"default\"")));
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn duplicate_planner_host_units_require_equivalent_compiler_inputs() {
        let fixture = std::env::temp_dir().join(format!("mantle-unit-duplicate-host-{}", std::process::id()));
        fs::create_dir_all(&fixture).unwrap();
        let src = fixture.join("build.rs");
        fs::write(&src, "fn main() {}\n").unwrap();
        let graph: GraphUnit = serde_json::from_value(json!({
            "pkg_id":"proc-macro2 1",
            "target":{"kind":["custom-build"],"name":"build-script-build","src_path":src.to_str().unwrap(),"edition":"2021"},
            "mode":"build","features":["proc-macro"],"dependencies":[]
        }))
        .unwrap();
        let mut receipt: PlannerReceipt = serde_json::from_value(json!({
            "schema_version":1,"cargo_mode":{"compatibility_class":"cargo-oracle-evidence"},
            "lockfile":{"blake3":"abc"},"unit_graph":{"digest_blake3":"def"},
            "unit_derivation_graph":{"ready":true,"blockers":[],"derivations":[
                {"unit_id":"u.z","package_id":"proc-macro2 1","target_name":"build-script-build",
                 "target_kind":"custom-build","execution_kind":"host","rustc_metadata_hash":"aaa",
                 "derivation":{"args":["--crate-name","build_script_build","--crate-type","bin",
                                       "--edition","2021",src.to_str().unwrap(),"-C","metadata=aaa",
                                       "--cfg","feature=\"default\"","--cfg","feature=\"proc-macro\""],
                               "env":{"CARGO_PKG_VERSION":"1"}}},
                {"unit_id":"u.a","package_id":"proc-macro2 1","target_name":"build-script-build",
                 "target_kind":"custom-build","execution_kind":"host","rustc_metadata_hash":"bbb",
                 "derivation":{"args":["--crate-name","build_script_build","--crate-type","bin",
                                       "--edition","2021",src.to_str().unwrap(),"-C","metadata=bbb",
                                       "--cfg","feature=\"default\"","--cfg","feature=\"proc-macro\""],
                               "env":{"CARGO_PKG_VERSION":"1"}}}
            ]}
        }))
        .unwrap();
        assert_eq!(match_planner(&graph, "host", &receipt).unwrap().unit_id, "u.a");
        receipt.unit_derivation_graph.derivations.reverse();
        assert_eq!(match_planner(&graph, "host", &receipt).unwrap().unit_id, "u.a");
        receipt.unit_derivation_graph.derivations[1]
            .derivation
            .env
            .insert("CARGO_PKG_VERSION".into(), "2".into());
        assert_eq!(match_planner(&graph, "host", &receipt).unwrap_err()[0].code, "unit-plan-mode-unsupported");
        fs::remove_dir_all(fixture).unwrap();
    }
}
