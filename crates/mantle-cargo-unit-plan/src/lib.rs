#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

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

pub const MAX_PLAN_UNITS: usize = 4096;
pub const MAX_INPUTS_PER_UNIT: usize = 256;
pub const MAX_PLAN_BYTES: usize = 4 * 1024 * 1024;
pub const MANIFEST_FILE: &str = "share/mantle/unit-dependencies-v1.json";

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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBindings {
    pub store_prefix: String,
    pub system: String,
    pub target_triple: String,
    pub host_triple: String,
    pub helper: String,
    pub rustc: String,
    pub toolchain: String,
    pub linker: String,
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

fn blocked(code: &'static str, subject: &str, detail: &str) -> Vec<Blocker> {
    vec![Blocker::new(code, subject, detail)]
}

fn validated_store_root<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = path.strip_prefix(prefix)?.strip_prefix('/')?;
    let mut parts = rest.splitn(2, '/');
    let root = parts.next()?;
    if root.len() < 34
        || root.as_bytes().get(32) != Some(&b'-')
        || !root.as_bytes()[..32].iter().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(byte))
        || !safe_name(&root[33..])
        || !parts.next().is_none_or(safe_relative)
    {
        return None;
    }
    Some(&path[..prefix.len() + 1 + root.len()])
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
    format!("{{{{mantle-unit-output:{id}:out}}}}")
}
/// Cargo unit identities can exceed the generic plan's 128-byte identifier bound.
fn plan_id(id: &str) -> String {
    format!("u.{}", blake3::hash(id.as_bytes()).to_hex())
}

fn lower_unit(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    plan: &PlanBindings,
    tool_roots: &BTreeSet<&str>,
    source_ids: &BTreeSet<&str>,
    known_units: &BTreeSet<&str>,
) -> Result<Unit, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    if binding.unit_id != id
        || !source_ids.contains(binding.source_id.as_str())
        || !safe_relative(&binding.source_entry)
        || !safe_name(&binding.source_label)
    {
        return Err(blocked("unit-plan-source-unsupported", id, "source binding is absent or invalid"));
    }
    if binding.source_kind != "path" && binding.source_kind != "registry" {
        return Err(blocked("unit-plan-source-unsupported", id, &binding.source_kind));
    }
    if effect.execution_kind == "build-script-run" || effect.execution_kind == "run-custom-build" {
        return Err(blocked("unit-plan-build-script-run-unsupported", id, &effect.package_id));
    }
    if !matches!(effect.execution_kind.as_str(), "host" | "host-dependency" | "target") {
        return Err(blocked("unit-plan-mode-unsupported", id, &effect.execution_kind));
    }
    if !matches!(effect.target_kind.as_str(), "lib" | "rlib" | "bin" | "proc-macro" | "custom-build") {
        return Err(blocked("unit-plan-mode-unsupported", id, &effect.target_kind));
    }
    if binding.triple != plan.target_triple && binding.triple != plan.host_triple {
        return Err(blocked("unit-plan-triple-unsupported", id, &binding.triple));
    }
    if effect.execution_kind != "target" && binding.triple != plan.host_triple {
        return Err(blocked("unit-plan-triple-unsupported", id, "host unit has target triple"));
    }
    if binding.rustc_metadata_hash.is_empty()
        || !binding.rustc_metadata_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(blocked("unit-plan-mode-unsupported", id, "missing rust-plan metadata hash"));
    }
    if effect.arguments.len() > 512
        || effect.arguments.iter().any(|arg| arg.len() > 16_384)
        || effect.environment.len() > 256
        || effect.environment.iter().any(|(key, value)| key.len() > 16_384 || value.len() > 16_384)
    {
        return Err(blocked("unit-plan-limit", id, "effect exceeds unit argument or environment limits"));
    }
    let mut args = effect.arguments.clone();
    for arg in &args {
        if arg.contains('\0')
            || arg.starts_with('/')
            || arg.contains("=/")
            || arg.contains("@/")
            || arg.contains("{{mantle-")
            || arg.starts_with("--out-dir")
            || arg.starts_with("--extern")
        {
            return Err(blocked("unit-plan-host-path", id, arg));
        }
    }
    if effect
        .environment
        .iter()
        .any(|(key, value)| !safe_name(key) || value.starts_with('/') || value.contains("=/") || value.contains('\0'))
    {
        return Err(blocked("unit-plan-host-path", id, "effect environment contains host paths"));
    }
    if effect.expected_outputs.as_slice() != ["out"] {
        return Err(blocked("unit-plan-mode-unsupported", id, "unit requires exactly the out output"));
    }
    let source = source_token(&binding.source_id);
    args.push(format!("{source}/{}", binding.source_entry));
    args.push("-C".to_string());
    args.push(format!("metadata={}", binding.rustc_metadata_hash));
    args.push(format!("--remap-path-prefix={source}={}", binding.source_label));
    let mut inputs = tool_roots
        .iter()
        .map(|path| Input::StorePath {
            path: (*path).to_string(),
        })
        .collect::<BTreeSet<_>>();
    inputs.insert(Input::Source {
        source: binding.source_id.clone(),
    });
    let mut deps = BTreeSet::new();
    for dep in &effect.dependency_unit_ids {
        if !known_units.contains(dep.0.as_str()) || dep.0 == id {
            return Err(blocked("unit-plan-mode-unsupported", id, "unknown or self dependency"));
        }
        deps.insert(dep.0.as_str());
        inputs.insert(Input::UnitOutput {
            unit: plan_id(&dep.0),
            output: "out",
        });
    }
    let mut externs = BTreeMap::new();
    for ext in &binding.externs {
        if !safe_name(&ext.name)
            || !deps.contains(ext.unit_id.as_str())
            || externs.insert(ext.name.clone(), output_token(&plan_id(&ext.unit_id))).is_some()
        {
            return Err(blocked("unit-plan-mode-unsupported", id, "extern is not a unique direct dependency"));
        }
    }
    if inputs.len() > MAX_INPUTS_PER_UNIT {
        return Err(blocked("unit-plan-limit", id, "unit inputs exceed 256"));
    }
    let compile_env = effect.environment.iter().cloned().collect::<BTreeMap<_, _>>();
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("MANTLE_UNIT_RUSTC".to_string(), plan.rustc.clone());
    env.insert("MANTLE_UNIT_LINKER".to_string(), plan.linker.clone());
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
    let mut inputs = inputs.into_iter().collect::<Vec<_>>();
    inputs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
    Ok(Unit {
        id: plan_id(id),
        derivation: Derivation {
            name: format!("rust-unit-{}", &blake3::hash(id.as_bytes()).to_hex().as_str()[..16]),
            builder: plan.helper.clone(),
            system: plan.system.clone(),
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
    let mut tool_roots = BTreeSet::new();
    for path in [&plan.helper, &plan.rustc, &plan.toolchain, &plan.linker] {
        let root = validated_store_root(path, &plan.store_prefix)
            .ok_or_else(|| blocked("unit-plan-host-path", "toolchain", path))?;
        tool_roots.insert(root);
    }
    let mut slices = plan.sources.clone();
    slices.sort_by(|a, b| a.id.cmp(&b.id));
    let mut source_ids = BTreeSet::new();
    for slice in &slices {
        if !safe_id(&slice.id)
            || !safe_name(&slice.producer_output)
            || !safe_name(&slice.store_name)
            || !safe_relative(&slice.subpath)
            || slice.nar_blake3.len() != 64
            || !slice.nar_blake3.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || !source_ids.insert(slice.id.as_str())
        {
            return Err(blocked("unit-plan-source-unsupported", &slice.id, "invalid source slice"));
        }
    }
    let mut bindings = BTreeMap::new();
    for binding in &plan.units {
        if bindings.insert(binding.unit_id.as_str(), binding).is_some() {
            return Err(blocked("unit-plan-mode-unsupported", &binding.unit_id, "duplicate binding"));
        }
    }
    let known_units = effects.iter().map(|effect| effect.unit_id.0.as_str()).collect::<BTreeSet<_>>();
    if known_units.len() != effects.len() || bindings.len() != effects.len() {
        return Err(blocked("unit-plan-mode-unsupported", "plan", "unit binding set does not match core effects"));
    }
    let mut units = Vec::with_capacity(effects.len());
    for effect in effects {
        let binding = bindings
            .get(effect.unit_id.0.as_str())
            .ok_or_else(|| blocked("unit-plan-mode-unsupported", &effect.unit_id.0, "missing binding"))?;
        units.push(lower_unit(effect, binding, plan, &tool_roots, &source_ids, &known_units)?);
    }
    units.sort_by(|a, b| a.id.cmp(&b.id));
    let mut roots = plan.roots.clone();
    roots.sort();
    roots.dedup();
    if roots.is_empty() || roots.iter().any(|root| !known_units.contains(root.as_str())) {
        return Err(blocked("unit-plan-mode-unsupported", "roots", "root is absent from core effects"));
    }
    let mut roots = roots.iter().map(|root| plan_id(root)).collect::<Vec<_>>();
    roots.sort();
    let wire = Plan {
        schema: "mantle-plan-v2",
        producer: Producer {
            logical_name: "cargo-unit-graph",
            goal_hint: None,
        },
        sources: &slices,
        units,
        roots: &roots,
        provenance: BTreeMap::new(),
    };
    let bytes = serde_json::to_vec(&wire).map_err(|_| blocked("unit-plan-limit", "plan", "plan serialization"))?;
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(blocked("unit-plan-limit", "plan", "plan bytes exceed 4 MiB"));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok((bytes, digest))
}
