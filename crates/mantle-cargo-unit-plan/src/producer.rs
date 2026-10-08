//! Sandboxed offline Cargo oracle and per-package source-slice producer.
//! The declared planner supplies rust-plan unit identity and metadata; Cargo's
//! versioned unit graph supplies exact direct producer edges and extern names.

mod source_nar;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use mantle_cargo_unit_plan::Blocker;
use mantle_cargo_unit_plan::ExternBinding;
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
    store_prefix: String,
    system: String,
    host_triple: String,
    target_triple: String,
    profile: String,
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
    let receipt_bytes = command_output(&mut planner, "rust-plan")?;
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
        if word.ends_with(".rs") && word.contains("/src/") {
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

fn match_planner<'a>(graph: &GraphUnit, receipt: &'a PlannerReceipt) -> Result<&'a PlannerUnit, Vec<Blocker>> {
    let mut matches = receipt.unit_derivation_graph.derivations.iter().filter(|unit| {
        unit.package_id == graph.pkg_id
            && unit.target_name.replace('-', "_") == graph.target.name.replace('-', "_")
            && graph.target.kind.iter().any(|kind| kind == &unit.target_kind)
    });
    let found = matches
        .next()
        .ok_or_else(|| block("unit-plan-oracle-failure", &graph.pkg_id, "missing Rust-plan unit"))?;
    if matches.next().is_some() {
        return Err(block("unit-plan-mode-unsupported", &graph.pkg_id, "ambiguous Rust-plan target identity"));
    }
    Ok(found)
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
) -> Result<PlannedUnitFacts, Vec<Blocker>> {
    let mut selected = Vec::new();
    for unit in &graph.units {
        let kind = unit
            .target
            .kind
            .first()
            .ok_or_else(|| block("unit-plan-mode-unsupported", &unit.pkg_id, "empty target kind"))?;
        if kind == "custom-build" && unit.mode == "run-custom-build" {
            return Err(block("unit-plan-build-script-run-unsupported", &unit.pkg_id, "build script execution"));
        }
        if unit.mode != "build" {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, &unit.mode));
        }
        if !matches!(kind.as_str(), "lib" | "bin" | "proc-macro" | "custom-build") {
            return Err(block("unit-plan-mode-unsupported", &unit.pkg_id, kind));
        }
        selected.push(match_planner(unit, receipt)?);
    }
    let mut facts = Vec::new();
    let mut bindings = Vec::new();
    for (index, unit) in graph.units.iter().enumerate() {
        let planned = selected[index];
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
        for edge in &unit.dependencies {
            let dependency = selected
                .get(edge.index)
                .ok_or_else(|| block("unit-plan-oracle-failure", &planned.unit_id, "dependency index outside graph"))?;
            dependencies.push(dependency.unit_id.clone());
            if let Some(name) = &edge.extern_crate_name {
                externs.push(ExternBinding {
                    name: name.clone(),
                    unit_id: dependency.unit_id.clone(),
                });
            }
        }
        let execution_kind = if matches!(unit.target.kind[0].as_str(), "proc-macro" | "custom-build") {
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
        let arguments = normalize_args(planned, unit)?;
        let mut compile_env = Vec::new();
        for (key, value) in &planned.derivation.env {
            if key.starts_with("CARGO_PKG_") || key.starts_with("CARGO_FEATURE_") {
                if value.starts_with('/') || value.contains("=/") || value.contains('\0') {
                    return Err(block("unit-plan-host-path", &planned.unit_id, key));
                }
                compile_env.push((key.clone(), value.clone()));
            }
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
            expected_outputs: vec!["out".into()],
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

fn run() -> Result<(), Vec<Blocker>> {
    let config: Config = serde_json::from_slice(
        &fs::read(required("MANTLE_UNIT_CONFIG")?)
            .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?,
    )
    .map_err(|error| block("unit-plan-oracle-failure", "config", error.to_string()))?;
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
    if let Some(vendor) = vendor {
        fs::write(
            cargo_home.join("config.toml"),
            format!(
                "[source.crates-io]\nreplace-with='declared-vendor'\n[source.declared-vendor]\ndirectory={:?}\n",
                vendor
            ),
        )
        .map_err(|error| block("unit-plan-oracle-failure", "cargo-home", error.to_string()))?;
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
    } = build_facts(&config, &graph, &receipt, &package_bindings)?;
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
    use super::parse_unit_graph;

    #[test]
    fn unknown_cargo_unit_graph_version_blocks_before_lowering() {
        let blockers = parse_unit_graph(br#"{"version":2,"units":[],"roots":[]}"#).unwrap_err();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "unit-plan-oracle-failure");
        assert_eq!(blockers[0].subject, "unit-graph");
    }
}
