#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::explicit_defaults)]

//! Pure adapter from admitted Rust-plan effects to the generic mantle-plan-v2 wire ABI.
//! Binding facts are supplied by the producer; this crate performs no store or host I/O.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::ExistingUnitEffect;
use serde::Deserialize;
use serde::Serialize;

mod build_script;

pub use build_script::BUILD_SCRIPT_RUN;
use build_script::BuildScriptGraph;
pub use build_script::RUN_BUILD_SCRIPT_MODE;
pub use build_script::TARGET_FACTS_MODE;

pub const MAX_PLAN_UNITS: usize = 4096;
pub const MAX_INPUTS_PER_UNIT: usize = 256;
pub const MAX_PLAN_BYTES: usize = 4_194_304;
pub const MANIFEST_FILE: &str = "share/mantle/unit-dependencies-v1.json";
/// Wait-unit provenance travels as numbered text chunks of at most this size.
const WAIT_PROVENANCE_CHUNK_BYTES: usize = 16_384;
/// A wait-unit table within `MAX_PLAN_BYTES` never needs more chunks than this.
const MAX_WAIT_PROVENANCE_CHUNKS: usize = MAX_PLAN_BYTES.div_ceil(WAIT_PROVENANCE_CHUNK_BYTES);

/// The producer owns these observations, including the source-slice NAR digest.
/// A source slice must refer to a separately admitted subtree of the producer output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSlice {
    pub id: String,
    pub producer_output: String,
    pub subpath: String,
    pub store_name: String,
    pub nar_blake3: String,
}

/// One direct rustc extern edge. `unit_id` must be a direct core-effect dependency.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternBinding {
    pub name: String,
    pub unit_id: String,
}

/// Producer-resolved facts of one build-script execution unit. Native inputs
/// are whole store roots already resolved from `{{role:<name>}}` markers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildScriptRunBinding {
    /// The package's Cargo `links` key, if any.
    pub links: Option<String>,
    pub native_inputs: Vec<String>,
    pub native_env: BTreeMap<String, String>,
    pub native_path: Vec<String>,
}

/// Side-effect facts not claimed by the pure core: reviewed toolchain and source
/// authority, the rust-plan metadata hash, and explicit dependency names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitBinding {
    pub unit_id: String,
    pub source_id: String,
    /// Admitted Cargo metadata source kind: path or registry.
    pub source_kind: String,
    pub source_entry: String,
    pub source_label: String,
    pub rustc_metadata_hash: String,
    pub triple: String,
    pub externs: Vec<ExternBinding>,
    /// Present exactly for `build-script-run` effects.
    pub build_script_run: Option<BuildScriptRunBinding>,
}

/// Target-only linker and its declared tool/sysroot roots. Host units always
/// retain `PlanBindings::linker`, even when host and target triples coincide.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLinkerBinding {
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBindings {
    pub store_prefix: String,
    pub system: String,
    pub target_triple: String,
    pub host_triple: String,
    pub helper: String,
    pub rustc: String,
    /// Producer/oracle identity only; never an execution input.
    pub toolchain: String,
    /// Whole producer-only Cargo/oracle roots, excluded from execution inputs.
    pub oracle_inputs: Vec<String>,
    pub linker: String,
    #[serde(default)]
    pub target_linker: Option<TargetLinkerBinding>,
    /// Explicit store roots required by the tools' ELF loaders and libraries.
    pub runtime_inputs: Vec<String>,
    /// Complete normalized compiler identities, declared identically off/on.
    pub wait_units: BTreeMap<String, mantle_rust_differential_core::waits::UnitIdentity>,
    pub sources: Vec<SourceSlice>,
    pub units: Vec<UnitBinding>,
    pub roots: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    pub code: &'static str,
    pub subject: String,
    pub detail: String,
}

impl Blocker {
    fn new(code: &'static str, subject: &str, detail: &str) -> Self {
        Self {
            code,
            subject: subject.to_string(),
            detail: detail.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Plan<'a> {
    schema: &'static str,
    producer: Producer,
    sources: &'a [SourceSlice],
    units: Vec<Unit>,
    roots: &'a [String],
    provenance: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Producer {
    logical_name: &'static str,
    goal_hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Unit {
    id: String,
    derivation: Derivation,
    requested_outputs: Vec<&'static str>,
    policy: UnitPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Derivation {
    name: String,
    builder: String,
    system: String,
    args: Vec<String>,
    outputs: Vec<&'static str>,
    env: BTreeMap<String, String>,
    inputs: Vec<Input>,
    fixed_output: Option<()>,
    addressing_mode: &'static str,
    sandbox: &'static str,
    dynamic_plan_outputs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Input {
    StorePath { path: String },
    Source { source: String },
    UnitOutput { unit: String, output: &'static str },
}

impl Input {
    fn canonical_key(&self) -> (&'static str, &str, &str) {
        match self {
            Self::StorePath { path } => ("store_path", path, ""),
            Self::Source { source } => ("source", source, ""),
            Self::UnitOutput { unit, output } => ("unit_output", unit, output),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct UnitPolicy {
    sandbox: &'static str,
    substitutions: &'static str,
    store_prefix: &'static str,
    host_paths: &'static str,
}

#[allow(tigerstyle::ambiguous_params)] // stable code vs subject vs detail: distinct blocker fields
fn blocked(code: &'static str, subject: &str, detail: &str) -> Vec<Blocker> {
    vec![Blocker::new(code, subject, detail)]
}

#[allow(tigerstyle::ambiguous_params)] // candidate path vs declared store prefix: distinct domains
fn validated_store_root<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = path.strip_prefix(prefix)?.strip_prefix('/')?;
    let mut parts = rest.splitn(2, '/');
    let root = parts.next()?;
    if root.len() < 34 || root.as_bytes().get(32) != Some(&b'-') {
        return None;
    }
    let root_checks = [
        root.as_bytes()[..32].iter().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(byte)),
        safe_store_name(&root[33..]),
        parts.next().is_none_or(safe_relative),
    ];
    if root_checks.contains(&false) {
        return None;
    }
    let root_end = prefix.len().saturating_add(1).saturating_add(root.len());
    debug_assert!(root_end <= path.len());
    debug_assert!(root.len() >= 34);
    Some(&path[..root_end])
}

// Nix store names have a different wire bound and alphabet from unit labels.
// Keep this aligned with nix_compat::store_path::validate_name without adding
// the std store adapter as a dependency of this alloc-only lowering library.
fn safe_store_name(name: &str) -> bool {
    const MAX_STORE_NAME_BYTES: usize = 211;
    !name.is_empty()
        && name.len() <= MAX_STORE_NAME_BYTES
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._+-?=".contains(&byte))
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

fn safe_id(id: &str) -> bool {
    id.len() <= 128
        && id.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase())
        && id
            .bytes()
            .skip(1)
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte))
}

fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.contains('\0')
        && !path.contains('\\')
        && !path.starts_with('/')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn source_token(id: &str) -> String {
    format!("{{{{mantle-source:{id}}}}}")
}
fn output_token(id: &str) -> String {
    build_script::named_output_token(id, "out")
}
/// Cargo unit identities can exceed the generic plan's 128-byte identifier bound.
pub fn plan_id(id: &str) -> String {
    format!("u.{}", blake3::hash(id.as_bytes()).to_hex())
}

/// Plan-wide admitted facts shared by every lowered unit.
struct LoweringContext<'a> {
    plan: &'a PlanBindings,
    tool_roots: BTreeSet<&'a str>,
    source_ids: BTreeSet<&'a str>,
    known_units: BTreeSet<&'a str>,
}

fn lower_unit(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LoweringContext<'_>,
    scripts: &BuildScriptGraph<'_>,
) -> Result<Unit, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    check_unit_binding(effect, binding, context)?;
    check_unit_effect_values(effect)?;
    let source = source_token(&binding.source_id);
    let mut args = Vec::with_capacity(effect.arguments.len().saturating_add(4));
    args.extend_from_slice(&effect.arguments);
    args.push(format!("{source}/{}", binding.source_entry));
    args.push("-C".to_string());
    args.push(format!("metadata={}", binding.rustc_metadata_hash));
    args.push(format!("--remap-path-prefix={source}={}", binding.source_label));
    let (inputs, deps) = unit_inputs(effect, binding, context, scripts)?;
    let mut env = unit_environment(effect, binding, context.plan, source, &deps)?;
    env.extend(build_script::compile_script_env(scripts, effect)?);
    if env.len() > 512 || env.values().any(|value| value.len() > 16_384) {
        return Err(blocked("unit-plan-limit", id, "unit environment exceeds generic plan limits"));
    }
    debug_assert!(inputs.len() <= MAX_INPUTS_PER_UNIT);
    debug_assert_eq!(args.len(), effect.arguments.len().saturating_add(4));
    Ok(Unit {
        id: plan_id(id),
        derivation: Derivation {
            name: format!("rust-unit-{}", &blake3::hash(id.as_bytes()).to_hex().as_str()[..16]),
            builder: context.plan.helper.clone(),
            system: context.plan.system.clone(),
            args,
            outputs: vec!["out"],
            env,
            inputs,
            fixed_output: None,
            addressing_mode: "input-addressed",
            sandbox: "native",
            dynamic_plan_outputs: Vec::new(),
        },
        requested_outputs: vec!["out"],
        policy: UnitPolicy {
            sandbox: "inherit",
            substitutions: "inherit",
            store_prefix: "inherit",
            host_paths: "none",
        },
    })
}

/// Source, mode, triple, and identity admission for one unit binding.
fn check_unit_binding(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LoweringContext<'_>,
) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let plan = context.plan;
    let source_checks = [
        binding.unit_id == id,
        context.source_ids.contains(binding.source_id.as_str()),
        safe_relative(&binding.source_entry),
        safe_name(&binding.source_label),
    ];
    if source_checks.contains(&false) {
        return Err(blocked("unit-plan-source-unsupported", id, "source binding is absent or invalid"));
    }
    if binding.source_kind != "path" && binding.source_kind != "registry" {
        return Err(blocked("unit-plan-source-unsupported", id, &binding.source_kind));
    }
    let is_run = effect.execution_kind == build_script::BUILD_SCRIPT_RUN;
    if is_run != binding.build_script_run.is_some() {
        return Err(blocked("unit-plan-build-script-run-unsupported", id, &effect.package_id));
    }
    if !matches!(effect.execution_kind.as_str(), "host" | "host-dependency" | "target" | "build-script-run") {
        return Err(blocked("unit-plan-mode-unsupported", id, &effect.execution_kind));
    }
    if !matches!(effect.target_kind.as_str(), "lib" | "rlib" | "bin" | "proc-macro" | "custom-build" | "cdylib") {
        return Err(blocked("unit-plan-mode-unsupported", id, &effect.target_kind));
    }
    if binding.triple != plan.target_triple && binding.triple != plan.host_triple {
        return Err(blocked("unit-plan-triple-unsupported", id, &binding.triple));
    }
    if matches!(effect.execution_kind.as_str(), "host" | "host-dependency") && binding.triple != plan.host_triple {
        return Err(blocked("unit-plan-triple-unsupported", id, "host unit has target triple"));
    }
    if binding.rustc_metadata_hash.is_empty()
        || !binding.rustc_metadata_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(blocked("unit-plan-mode-unsupported", id, "missing rust-plan metadata hash"));
    }
    debug_assert_eq!(binding.unit_id, id);
    debug_assert!(!binding.rustc_metadata_hash.is_empty());
    Ok(())
}

/// Argument, environment, and output admission for core effect values.
fn check_unit_effect_values(effect: &ExistingUnitEffect) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let effect_overflow_checks = [
        effect.arguments.len() > 512,
        effect.arguments.iter().any(|arg| arg.len() > 16_384),
        effect.environment.len() > 256,
        effect.environment.iter().any(|(key, value)| key.len() > 16_384 || value.len() > 16_384),
    ];
    if effect_overflow_checks.contains(&true) {
        return Err(blocked("unit-plan-limit", id, "effect exceeds unit argument or environment limits"));
    }
    if let Some(arg) = effect.arguments.iter().find(|arg| is_host_path_argument(arg)) {
        return Err(blocked("unit-plan-host-path", id, arg));
    }
    if effect.environment.iter().any(|(key, value)| !safe_name(key) || is_host_path_value(value)) {
        return Err(blocked("unit-plan-host-path", id, "effect environment contains host paths"));
    }
    let expected_outputs: &[&str] = if effect.execution_kind == build_script::BUILD_SCRIPT_RUN {
        &["out", "flags"]
    } else {
        &["out"]
    };
    if effect.expected_outputs.as_slice() != expected_outputs {
        return Err(blocked("unit-plan-mode-unsupported", id, "unit requires exactly its declared outputs"));
    }
    debug_assert!(effect.arguments.len() <= 512);
    debug_assert!(effect.environment.len() <= 256);
    Ok(())
}

/// Host paths, placeholders, and adapter-owned output or extern flags may not
/// enter a plan argument; the helper supplies those from its environment.
fn is_host_path_argument(arg: &str) -> bool {
    let host_path_markers = [
        arg.contains('\0'),
        arg.starts_with('/'),
        arg.contains("=/"),
        arg.contains("@/"),
        arg.contains("{{mantle-"),
        arg.starts_with("--out-dir"),
        arg.starts_with("--extern"),
    ];
    host_path_markers.contains(&true)
}

fn is_host_path_value(value: &str) -> bool {
    value.starts_with('/') || value.contains("=/") || value.contains('\0')
}

type UnitInputs<'a> = (Vec<Input>, BTreeSet<&'a str>);

/// Direct inputs only: declared tool roots, the owning source slice, and the
/// outputs of direct dependency units. Transitive reachability is carried by
/// dependency manifests, not by additional unit inputs.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree sets cannot reserve; MAX_INPUTS_PER_UNIT bounds the result.
fn unit_inputs<'a>(
    effect: &'a ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LoweringContext<'_>,
    scripts: &BuildScriptGraph<'_>,
) -> Result<UnitInputs<'a>, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let mut inputs = context
        .tool_roots
        .iter()
        .map(|path| Input::StorePath {
            path: (*path).to_string(),
        })
        .collect::<BTreeSet<_>>();
    inputs.insert(Input::Source {
        source: binding.source_id.clone(),
    });
    if effect.execution_kind == "target"
        && let Some(linker) = &context.plan.target_linker
    {
        for path in core::iter::once(&linker.executable).chain(linker.inputs.iter()) {
            let root = validated_store_root(path, &context.plan.store_prefix)
                .ok_or_else(|| blocked("unit-plan-host-path", id, path))?;
            inputs.insert(Input::StorePath { path: root.to_string() });
        }
    }
    let mut deps = BTreeSet::new();
    for dep in &effect.dependency_unit_ids {
        if !context.known_units.contains(dep.0.as_str()) || dep.0 == id {
            return Err(blocked("unit-plan-mode-unsupported", id, "unknown or self dependency"));
        }
        if scripts.is_run(dep.0.as_str()) {
            continue;
        }
        deps.insert(dep.0.as_str());
        inputs.insert(Input::UnitOutput {
            unit: plan_id(&dep.0),
            output: "out",
        });
    }
    inputs.extend(build_script::compile_script_inputs(scripts, id));
    if inputs.len() > MAX_INPUTS_PER_UNIT {
        return Err(blocked("unit-plan-limit", id, "unit inputs exceed 256"));
    }
    let mut inputs = inputs.into_iter().collect::<Vec<_>>();
    inputs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
    debug_assert!(deps.len() <= effect.dependency_unit_ids.len());
    debug_assert!(inputs.len() > deps.len());
    Ok((inputs, deps))
}

#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; extern names are unique direct edges.
fn unit_externs(binding: &UnitBinding, deps: &BTreeSet<&str>) -> Result<BTreeMap<String, String>, Vec<Blocker>> {
    let mut externs = BTreeMap::new();
    for ext in &binding.externs {
        if !safe_name(&ext.name)
            || !deps.contains(ext.unit_id.as_str())
            || externs.insert(ext.name.clone(), output_token(&plan_id(&ext.unit_id))).is_some()
        {
            return Err(blocked(
                "unit-plan-mode-unsupported",
                &binding.unit_id,
                "extern is not a unique direct dependency",
            ));
        }
    }
    debug_assert_eq!(externs.len(), binding.externs.len());
    debug_assert!(externs.values().all(|token| token.starts_with("{{mantle-unit-output:")));
    Ok(externs)
}

fn unit_environment(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    plan: &PlanBindings,
    source: String,
    deps: &BTreeSet<&str>,
) -> Result<BTreeMap<String, String>, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let externs = unit_externs(binding, deps)?;
    let compile_env = effect.environment.iter().cloned().collect::<BTreeMap<_, _>>();
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("MANTLE_UNIT_RUSTC".to_string(), plan.rustc.clone());
    let wait_id = plan.wait_units.get(id).and_then(|unit| unit.identity().ok());
    env.insert(
        "MANTLE_UNIT_WAIT_TOOLS".into(),
        serde_json::to_string(&serde_json::json!({
            "helper": plan.helper,
            "rustc": plan.rustc,
            "unit": wait_id,
        }))
        .map_err(|_| blocked("unit-plan-limit", id, "wait inventory serialization"))?,
    );
    let target_linker = plan.target_linker.as_ref().filter(|_| effect.execution_kind == "target");
    env.insert(
        "MANTLE_UNIT_LINKER".to_string(),
        target_linker.map_or(&plan.linker, |linker| &linker.executable).clone(),
    );
    if let Some(linker) = target_linker {
        env.insert(
            "MANTLE_UNIT_LINK_ARGS".into(),
            serde_json::to_string(&linker.arguments)
                .map_err(|_| blocked("unit-plan-limit", id, "linker argument serialization"))?,
        );
    }
    env.insert("MANTLE_UNIT_SOURCE".to_string(), source);
    env.insert(
        "MANTLE_UNIT_COMPILE_ENV".to_string(),
        serde_json::to_string(&compile_env)
            .map_err(|_| blocked("unit-plan-limit", id, "compile environment serialization"))?,
    );
    env.insert(
        "MANTLE_UNIT_DEPENDENCIES".to_string(),
        serde_json::to_string(&deps.iter().map(|id| output_token(&plan_id(id))).collect::<Vec<_>>())
            .map_err(|_| blocked("unit-plan-limit", id, "dependency serialization"))?,
    );
    env.insert(
        "MANTLE_UNIT_EXTERNS".to_string(),
        serde_json::to_string(&externs).map_err(|_| blocked("unit-plan-limit", id, "extern serialization"))?,
    );
    if env.len() > 512 || env.values().any(|value| value.len() > 16_384) {
        return Err(blocked("unit-plan-limit", id, "unit environment exceeds generic plan limits"));
    }
    debug_assert!(env.len() >= 6);
    debug_assert!(env.values().all(|value| value.len() <= 16_384));
    Ok(env)
}

fn is_valid_source_slice(slice: &SourceSlice) -> bool {
    let slice_checks = [
        safe_id(&slice.id),
        safe_name(&slice.producer_output),
        safe_name(&slice.store_name),
        safe_relative(&slice.subpath),
        slice.nar_blake3.len() == 64,
        slice.nar_blake3.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
    ];
    !slice_checks.contains(&false)
}
/// Lower only reviewed core effects. No partial plan is returned on any blocker.
/// The caller must pass producer-observed source digests and existing store paths.
// r[impl mantle.rust_unit_plan.lowering]
// r[impl mantle.rust_unit_plan.dependency_closure]
pub fn lower(effects: &[ExistingUnitEffect], plan: &PlanBindings) -> Result<(Vec<u8>, String), Vec<Blocker>> {
    if effects.is_empty() || effects.len() > MAX_PLAN_UNITS {
        return Err(blocked("unit-plan-limit", "plan", "unit count outside 1..4096"));
    }
    if plan.sources.len() > 256 || plan.sources.is_empty() {
        return Err(blocked("unit-plan-limit", "sources", "slice count outside 1..256"));
    }
    let mut slices = plan.sources.clone();
    slices.sort_by(|a, b| a.id.cmp(&b.id));
    let context = lowering_context(effects, plan)?;
    let bindings = unit_bindings(effects, plan, &context)?;
    let scripts = build_script::build_script_graph(effects, &bindings)?;
    let units = lower_units(effects, &bindings, &context, &scripts)?;
    let mut roots = plan.roots.clone();
    roots.sort();
    roots.dedup();
    if roots.is_empty() || roots.iter().any(|root| !context.known_units.contains(root.as_str())) {
        return Err(blocked("unit-plan-mode-unsupported", "roots", "root is absent from core effects"));
    }
    let mut roots = roots.iter().map(|root| plan_id(root)).collect::<Vec<_>>();
    roots.sort();
    assert!(units.len() >= effects.len());
    assert!(!roots.is_empty());
    let provenance = wait_provenance(plan);
    let mut wire = Plan {
        schema: "mantle-plan-v2",
        producer: Producer {
            logical_name: "cargo-unit-graph",
            goal_hint: None,
        },
        sources: &slices,
        units,
        roots: &roots,
        provenance,
    };
    let mut bytes = serde_json::to_vec(&wire).map_err(|_| blocked("unit-plan-limit", "plan", "plan serialization"))?;
    if bytes.len() > MAX_PLAN_BYTES && !wire.provenance.is_empty() {
        wire.provenance.clear();
        bytes = serde_json::to_vec(&wire).map_err(|_| blocked("unit-plan-limit", "plan", "plan serialization"))?;
    }
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(blocked("unit-plan-limit", "plan", "plan bytes exceed 4 MiB"));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok((bytes, digest))
}

fn wait_provenance(plan: &PlanBindings) -> BTreeMap<String, String> {
    if plan.wait_units.len() > MAX_PLAN_UNITS || plan.wait_units.values().any(|unit| unit.identity().is_err()) {
        return BTreeMap::new();
    }
    let Ok(table) = serde_json::to_vec(&plan.wait_units.values().collect::<Vec<_>>()) else {
        return BTreeMap::new();
    };
    if table.len() > MAX_PLAN_BYTES {
        return BTreeMap::new();
    }
    let mut provenance = BTreeMap::new();
    for (index, chunk) in table.chunks(WAIT_PROVENANCE_CHUNK_BYTES).enumerate() {
        if provenance.len() >= MAX_WAIT_PROVENANCE_CHUNKS {
            return BTreeMap::new();
        }
        let Ok(text) = core::str::from_utf8(chunk) else {
            return BTreeMap::new();
        };
        provenance.insert(format!("mantle-rust-wait-units-v1.{index:04}"), text.to_string());
    }
    debug_assert!(provenance.len() <= MAX_WAIT_PROVENANCE_CHUNKS);
    debug_assert!(!provenance.values().any(|chunk| chunk.is_empty() || chunk.len() > WAIT_PROVENANCE_CHUNK_BYTES));
    provenance
}

/// Lower every core effect plus one target-description unit per build-script
/// triple, sorted by wire identity.
// r[impl mantle.rust_unit_plan.build_scripts.target_facts]
fn lower_units(
    effects: &[ExistingUnitEffect],
    bindings: &BTreeMap<&str, &UnitBinding>,
    context: &LoweringContext<'_>,
    scripts: &BuildScriptGraph<'_>,
) -> Result<Vec<Unit>, Vec<Blocker>> {
    let mut units = Vec::with_capacity(effects.len().saturating_add(scripts.triples.len()));
    for effect in effects {
        let binding = bindings
            .get(effect.unit_id.0.as_str())
            .ok_or_else(|| blocked("unit-plan-mode-unsupported", &effect.unit_id.0, "missing binding"))?;
        if scripts.is_run(&effect.unit_id.0) {
            check_unit_binding(effect, binding, context)?;
            check_unit_effect_values(effect)?;
            units.push(build_script::lower_run_unit(effect, binding, context, scripts)?);
        } else {
            units.push(lower_unit(effect, binding, context, scripts)?);
        }
    }
    for triple in &scripts.triples {
        units.push(build_script::target_facts_unit(context.plan, triple)?);
    }
    if units.len() > MAX_PLAN_UNITS {
        return Err(blocked("unit-plan-limit", "plan", "lowered units exceed 4096"));
    }
    units.sort_by(|a, b| a.id.cmp(&b.id));
    debug_assert_eq!(units.len(), effects.len().saturating_add(scripts.triples.len()));
    debug_assert!(units.windows(2).all(|pair| pair[0].id < pair[1].id));
    Ok(units)
}

/// Store roots are indivisible capabilities: an oracle root may not be mounted
/// for compilation or script execution, even when its PATH entry is omitted.
fn check_execution_root(plan: &PlanBindings, root: &str) -> Result<(), Vec<Blocker>> {
    if plan
        .oracle_inputs
        .iter()
        .any(|oracle| validated_store_root(oracle, &plan.store_prefix) == Some(root))
    {
        return Err(blocked("unit-plan-oracle-execution-overlap", "execution_inputs", root));
    }
    debug_assert!(root.starts_with(&plan.store_prefix));
    debug_assert!(!plan.oracle_inputs.iter().any(|oracle| oracle == root));
    Ok(())
}

#[allow(tigerstyle::unbounded_collection_growth)] // BTree sets cannot reserve; slice and unit counts were checked.
fn lowering_context<'a>(
    effects: &'a [ExistingUnitEffect],
    plan: &'a PlanBindings,
) -> Result<LoweringContext<'a>, Vec<Blocker>> {
    debug_assert!(!effects.is_empty());
    debug_assert!(plan.sources.len() <= 256);
    if plan.oracle_inputs.is_empty() || plan.oracle_inputs.len() > 64 {
        return Err(blocked("unit-plan-limit", "oracle_inputs", "declare 1 to 64 producer-only store roots"));
    }
    for root in &plan.oracle_inputs {
        if validated_store_root(root, &plan.store_prefix) != Some(root.as_str()) {
            return Err(blocked("unit-plan-host-path", "oracle_inputs", root));
        }
    }
    let oracle_toolchain = validated_store_root(&plan.toolchain, &plan.store_prefix)
        .ok_or_else(|| blocked("unit-plan-host-path", "toolchain", &plan.toolchain))?;
    if !plan.oracle_inputs.iter().any(|root| root == oracle_toolchain) {
        return Err(blocked("unit-plan-host-path", "oracle_inputs", "oracle toolchain root is not declared"));
    }
    let mut tool_roots = BTreeSet::new();
    for path in [&plan.helper, &plan.rustc, &plan.linker] {
        let root = validated_store_root(path, &plan.store_prefix)
            .ok_or_else(|| blocked("unit-plan-host-path", "toolchain", path))?;
        tool_roots.insert(root);
        check_execution_root(plan, root)?;
    }
    if plan.runtime_inputs.len() > 64 {
        return Err(blocked("unit-plan-limit", "runtime_inputs", "runtime tool roots exceed 64"));
    }
    let mut declared_runtime = BTreeSet::new();
    for path in &plan.runtime_inputs {
        let root = validated_store_root(path, &plan.store_prefix)
            .ok_or_else(|| blocked("unit-plan-host-path", "runtime_inputs", path))?;
        if root != path || !declared_runtime.insert(root) {
            return Err(blocked("unit-plan-host-path", "runtime_inputs", "runtime input must be a unique store root"));
        }
        check_execution_root(plan, root)?;
        tool_roots.insert(root);
    }
    validate_target_linker(plan)?;
    let mut ordered_slices = plan.sources.iter().collect::<Vec<_>>();
    ordered_slices.sort_by(|a, b| a.id.cmp(&b.id));
    let mut source_ids = BTreeSet::new();
    for slice in ordered_slices {
        if !is_valid_source_slice(slice) || !source_ids.insert(slice.id.as_str()) {
            return Err(blocked("unit-plan-source-unsupported", &slice.id, "invalid source slice"));
        }
    }
    let known_units = effects.iter().map(|effect| effect.unit_id.0.as_str()).collect::<BTreeSet<_>>();
    Ok(LoweringContext {
        plan,
        tool_roots,
        source_ids,
        known_units,
    })
}

/// Validate every target linker path before any unit is emitted. Linker options
/// are data, never a shell command; path-bearing tokens must name declared roots.
fn validate_target_linker(plan: &PlanBindings) -> Result<(), Vec<Blocker>> {
    let Some(linker) = &plan.target_linker else {
        return Ok(());
    };
    if linker.inputs.len() > 64 || linker.arguments.len() > 256 {
        return Err(blocked("unit-plan-limit", "target_linker", "linker inputs/arguments exceed bounds"));
    }
    let executable_root = validated_store_root(&linker.executable, &plan.store_prefix)
        .ok_or_else(|| blocked("unit-plan-host-path", "target_linker", &linker.executable))?;
    check_execution_root(plan, executable_root)?;
    for input in &linker.inputs {
        if validated_store_root(input, &plan.store_prefix) != Some(input.as_str()) {
            return Err(blocked("unit-plan-host-path", "target_linker", input));
        }
        check_execution_root(plan, input)?;
    }
    for argument in &linker.arguments {
        let is_empty_argument = argument.is_empty();
        let is_excessive_argument = argument.len() > 16_384;
        if is_empty_argument || is_excessive_argument {
            return Err(blocked("unit-plan-host-path", "target_linker", argument));
        }
        let has_nul_byte = argument.contains('\0');
        let has_template_marker = argument.contains("{{");
        let has_at_marker = argument.contains('@');
        if has_nul_byte || has_template_marker || has_at_marker {
            return Err(blocked("unit-plan-host-path", "target_linker", argument));
        }
        for token in argument.split(['=', ',', ' ']) {
            let path = if token.starts_with("-L/") || token.starts_with("-I/") {
                &token[2..]
            } else {
                token
            };
            if path.contains('/') && !path.starts_with('/') {
                return Err(blocked("unit-plan-host-path", "target_linker", argument));
            }
            if path.starts_with('/') {
                let root = validated_store_root(path, &plan.store_prefix)
                    .ok_or_else(|| blocked("unit-plan-host-path", "target_linker", argument))?;
                if root != executable_root && !linker.inputs.iter().any(|input| input == root) {
                    return Err(blocked("unit-plan-host-path", "target_linker", "linker path has no declared input"));
                }
            }
        }
    }
    debug_assert!(!linker.executable.is_empty());
    debug_assert!(linker.arguments.len() <= 256);
    Ok(())
}

#[allow(tigerstyle::unbounded_collection_growth)] // BTree maps cannot reserve; the binding count must equal effects.
fn unit_bindings<'a>(
    effects: &[ExistingUnitEffect],
    plan: &'a PlanBindings,
    context: &LoweringContext<'_>,
) -> Result<BTreeMap<&'a str, &'a UnitBinding>, Vec<Blocker>> {
    let mut bindings = BTreeMap::new();
    for binding in &plan.units {
        if bindings.insert(binding.unit_id.as_str(), binding).is_some() {
            return Err(blocked("unit-plan-mode-unsupported", &binding.unit_id, "duplicate binding"));
        }
    }
    if context.known_units.len() != effects.len() || bindings.len() != effects.len() {
        return Err(blocked("unit-plan-mode-unsupported", "plan", "unit binding set does not match core effects"));
    }
    debug_assert_eq!(bindings.len(), plan.units.len());
    debug_assert_eq!(bindings.len(), context.known_units.len());
    Ok(bindings)
}
