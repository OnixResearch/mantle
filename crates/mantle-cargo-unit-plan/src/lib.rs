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
pub const MAX_PLAN_BYTES: usize = 4_194_304; // 4 MiB
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

struct BlockerContext<'a> {
    code: &'static str,
    subject: &'a str,
    detail: &'a str,
}

fn blocked(context: BlockerContext<'_>) -> Vec<Blocker> {
    vec![Blocker {
        code: context.code,
        subject: context.subject.to_string(),
        detail: context.detail.to_string(),
    }]
}

struct StoreRootLocation<'a> {
    path: &'a str,
    prefix: &'a str,
}

fn safe_store_root_name(root: &str) -> bool {
    if root.len() < 34 || root.as_bytes().get(32) != Some(&b'-') {
        return false;
    }
    let is_store_hash = root
        .get(..32)
        .is_some_and(|hash| hash.bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte)));
    is_store_hash && root.get(33..).is_some_and(safe_name)
}

fn validated_store_root<'a>(location: StoreRootLocation<'a>) -> Option<&'a str> {
    let rest = location.path.strip_prefix(location.prefix)?.strip_prefix('/')?;
    let mut parts = rest.splitn(2, '/');
    let root = parts.next()?;
    if !safe_store_root_name(root) || !parts.next().is_none_or(safe_relative) {
        return None;
    }
    let root_end = location.prefix.len().checked_add(1)?.checked_add(root.len())?;
    location.path.get(..root_end)
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

struct LowerContext<'a> {
    plan: &'a PlanBindings,
    tool_roots: &'a BTreeSet<&'a str>,
    source_ids: &'a BTreeSet<&'a str>,
    known_units: &'a BTreeSet<&'a str>,
}

struct LowerInputs<'a> {
    inputs: BTreeSet<Input>,
    dependencies: BTreeSet<&'a str>,
    externs: BTreeMap<String, String>,
}

fn validate_source_and_mode(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LowerContext<'_>,
) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let is_bound_source = binding.unit_id == id && context.source_ids.contains(binding.source_id.as_str());
    let is_safe_source = safe_relative(&binding.source_entry) && safe_name(&binding.source_label);
    if !is_bound_source || !is_safe_source {
        return Err(blocked(BlockerContext {
            code: "unit-plan-source-unsupported",
            subject: id,
            detail: "source binding is absent or invalid",
        }));
    }
    if binding.source_kind != "path" && binding.source_kind != "registry" {
        return Err(blocked(BlockerContext {
            code: "unit-plan-source-unsupported",
            subject: id,
            detail: &binding.source_kind,
        }));
    }
    if effect.execution_kind == "build-script-run" || effect.execution_kind == "run-custom-build" {
        return Err(blocked(BlockerContext {
            code: "unit-plan-build-script-run-unsupported",
            subject: id,
            detail: &effect.package_id,
        }));
    }
    if !matches!(effect.execution_kind.as_str(), "host" | "host-dependency" | "target") {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: id,
            detail: &effect.execution_kind,
        }));
    }
    if !matches!(effect.target_kind.as_str(), "lib" | "rlib" | "bin" | "proc-macro" | "custom-build") {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: id,
            detail: &effect.target_kind,
        }));
    }
    if binding.triple != context.plan.target_triple && binding.triple != context.plan.host_triple {
        return Err(blocked(BlockerContext {
            code: "unit-plan-triple-unsupported",
            subject: id,
            detail: &binding.triple,
        }));
    }
    if effect.execution_kind != "target" && binding.triple != context.plan.host_triple {
        return Err(blocked(BlockerContext {
            code: "unit-plan-triple-unsupported",
            subject: id,
            detail: "host unit has target triple",
        }));
    }
    if binding.rustc_metadata_hash.is_empty()
        || !binding.rustc_metadata_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: id,
            detail: "missing rust-plan metadata hash",
        }));
    }
    debug_assert!(is_bound_source && is_safe_source);
    debug_assert!(binding.triple == context.plan.host_triple || binding.triple == context.plan.target_triple);
    Ok(())
}

fn is_unsafe_argument(arg: &str) -> bool {
    let is_host_path = arg.contains('\0') || arg.starts_with('/') || arg.contains("=/");
    let is_interpolated_path = arg.contains("@/") || arg.contains("{{mantle-");
    let is_reserved_flag = arg.starts_with("--out-dir") || arg.starts_with("--extern");
    is_host_path || is_interpolated_path || is_reserved_flag
}

fn validate_effect_shape(effect: &ExistingUnitEffect) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let is_arguments_bounded = effect.arguments.len() <= 512 && effect.arguments.iter().all(|arg| arg.len() <= 16_384);
    let is_environment_bounded = effect.environment.len() <= 256
        && effect.environment.iter().all(|(key, value)| key.len() <= 16_384 && value.len() <= 16_384);
    if !is_arguments_bounded || !is_environment_bounded {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: id,
            detail: "effect exceeds unit argument or environment limits",
        }));
    }
    for arg in &effect.arguments {
        if is_unsafe_argument(arg) {
            return Err(blocked(BlockerContext {
                code: "unit-plan-host-path",
                subject: id,
                detail: arg,
            }));
        }
    }
    if effect
        .environment
        .iter()
        .any(|(key, value)| !safe_name(key) || value.starts_with('/') || value.contains("=/") || value.contains('\0'))
    {
        return Err(blocked(BlockerContext {
            code: "unit-plan-host-path",
            subject: id,
            detail: "effect environment contains host paths",
        }));
    }
    if effect.expected_outputs.as_slice() != ["out"] {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: id,
            detail: "unit requires exactly the out output",
        }));
    }
    debug_assert!(is_arguments_bounded && is_environment_bounded);
    debug_assert_eq!(effect.expected_outputs.as_slice(), ["out"]);
    Ok(())
}

fn lower_inputs<'a>(
    effect: &'a ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LowerContext<'_>,
) -> Result<LowerInputs<'a>, Vec<Blocker>> {
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
    let mut dependencies = BTreeSet::new();
    for dep in &effect.dependency_unit_ids {
        if !context.known_units.contains(dep.0.as_str()) || dep.0 == id {
            return Err(blocked(BlockerContext {
                code: "unit-plan-mode-unsupported",
                subject: id,
                detail: "unknown or self dependency",
            }));
        }
        dependencies.insert(dep.0.as_str());
        inputs.insert(Input::UnitOutput {
            unit: plan_id(&dep.0),
            output: "out",
        });
    }
    let mut externs = BTreeMap::new();
    for ext in &binding.externs {
        if !safe_name(&ext.name)
            || !dependencies.contains(ext.unit_id.as_str())
            || externs.insert(ext.name.clone(), output_token(&plan_id(&ext.unit_id))).is_some()
        {
            return Err(blocked(BlockerContext {
                code: "unit-plan-mode-unsupported",
                subject: id,
                detail: "extern is not a unique direct dependency",
            }));
        }
    }
    if inputs.len() > MAX_INPUTS_PER_UNIT {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: id,
            detail: "unit inputs exceed 256",
        }));
    }
    debug_assert!(inputs.len() <= MAX_INPUTS_PER_UNIT);
    debug_assert!(externs.len() <= binding.externs.len());
    Ok(LowerInputs {
        inputs,
        dependencies,
        externs,
    })
}

fn lower_environment(
    effect: &ExistingUnitEffect,
    context: &LowerContext<'_>,
    source: String,
    lowered: &LowerInputs<'_>,
) -> Result<BTreeMap<String, String>, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let compile_env = effect.environment.iter().cloned().collect::<BTreeMap<_, _>>();
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("MANTLE_UNIT_RUSTC".to_string(), context.plan.rustc.clone());
    env.insert("MANTLE_UNIT_LINKER".to_string(), context.plan.linker.clone());
    env.insert("MANTLE_UNIT_SOURCE".to_string(), source);
    env.insert(
        "MANTLE_UNIT_COMPILE_ENV".to_string(),
        serde_json::to_string(&compile_env).map_err(|_| {
            blocked(BlockerContext {
                code: "unit-plan-limit",
                subject: id,
                detail: "compile environment serialization",
            })
        })?,
    );
    env.insert(
        "MANTLE_UNIT_DEPENDENCIES".to_string(),
        serde_json::to_string(&lowered.dependencies.iter().map(|id| output_token(&plan_id(id))).collect::<Vec<_>>())
            .map_err(|_| {
                blocked(BlockerContext {
                    code: "unit-plan-limit",
                    subject: id,
                    detail: "dependency serialization",
                })
            })?,
    );
    env.insert(
        "MANTLE_UNIT_EXTERNS".to_string(),
        serde_json::to_string(&lowered.externs).map_err(|_| {
            blocked(BlockerContext {
                code: "unit-plan-limit",
                subject: id,
                detail: "extern serialization",
            })
        })?,
    );
    if env.len() > 512 || env.values().any(|value| value.len() > 16_384) {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: id,
            detail: "unit environment exceeds generic plan limits",
        }));
    }
    debug_assert!(env.len() <= 512);
    debug_assert!(env.values().all(|value| value.len() <= 16_384));
    Ok(env)
}

fn lower_unit(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LowerContext<'_>,
) -> Result<Unit, Vec<Blocker>> {
    validate_source_and_mode(effect, binding, context)?;
    validate_effect_shape(effect)?;
    let id = effect.unit_id.0.as_str();
    let source = source_token(&binding.source_id);
    let mut args = effect.arguments.clone();
    args.push(format!("{source}/{}", binding.source_entry));
    args.push("-C".to_string());
    args.push(format!("metadata={}", binding.rustc_metadata_hash));
    args.push(format!("--remap-path-prefix={source}={}", binding.source_label));
    let lowered = lower_inputs(effect, binding, context)?;
    let env = lower_environment(effect, context, source, &lowered)?;
    let mut inputs = lowered.inputs.into_iter().collect::<Vec<_>>();
    inputs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
    debug_assert!(args.len() <= 516);
    debug_assert!(inputs.len() <= MAX_INPUTS_PER_UNIT);
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

fn validate_slices(slices: &[SourceSlice]) -> Result<BTreeSet<&str>, Vec<Blocker>> {
    let mut source_ids = BTreeSet::new();
    for slice in slices {
        let is_valid_identity = safe_id(&slice.id) && safe_name(&slice.producer_output) && safe_name(&slice.store_name);
        let is_valid_digest = slice.nar_blake3.len() == 64
            && slice.nar_blake3.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
        let is_valid_slice = is_valid_identity && safe_relative(&slice.subpath) && is_valid_digest;
        if !is_valid_slice || !source_ids.insert(slice.id.as_str()) {
            return Err(blocked(BlockerContext {
                code: "unit-plan-source-unsupported",
                subject: &slice.id,
                detail: "invalid source slice",
            }));
        }
    }
    debug_assert_eq!(source_ids.len(), slices.len());
    debug_assert!(source_ids.len() <= 256);
    Ok(source_ids)
}

struct AdmittedBindings<'a> {
    bindings: BTreeMap<&'a str, &'a UnitBinding>,
    known_units: BTreeSet<&'a str>,
}

fn admit_bindings<'a>(
    effects: &'a [ExistingUnitEffect],
    plan: &'a PlanBindings,
) -> Result<AdmittedBindings<'a>, Vec<Blocker>> {
    let mut bindings = BTreeMap::new();
    for binding in &plan.units {
        if bindings.insert(binding.unit_id.as_str(), binding).is_some() {
            return Err(blocked(BlockerContext {
                code: "unit-plan-mode-unsupported",
                subject: &binding.unit_id,
                detail: "duplicate binding",
            }));
        }
    }
    let known_units = effects.iter().map(|effect| effect.unit_id.0.as_str()).collect::<BTreeSet<_>>();
    if known_units.len() != effects.len() || bindings.len() != effects.len() {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: "plan",
            detail: "unit binding set does not match core effects",
        }));
    }
    debug_assert_eq!(known_units.len(), effects.len());
    debug_assert_eq!(bindings.len(), effects.len());
    Ok(AdmittedBindings { bindings, known_units })
}

fn serialize_plan(wire: &Plan<'_>) -> Result<(Vec<u8>, String), Vec<Blocker>> {
    let bytes = serde_json::to_vec(wire).map_err(|_| {
        blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: "plan",
            detail: "plan serialization",
        })
    })?;
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: "plan",
            detail: "plan bytes exceed 4 MiB",
        }));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok((bytes, digest))
}

fn lower_units(
    effects: &[ExistingUnitEffect],
    bindings: &BTreeMap<&str, &UnitBinding>,
    context: &LowerContext<'_>,
) -> Result<Vec<Unit>, Vec<Blocker>> {
    let mut units = Vec::with_capacity(effects.len());
    for effect in effects {
        let binding = bindings.get(effect.unit_id.0.as_str()).ok_or_else(|| {
            blocked(BlockerContext {
                code: "unit-plan-mode-unsupported",
                subject: &effect.unit_id.0,
                detail: "missing binding",
            })
        })?;
        units.push(lower_unit(effect, binding, context)?);
    }
    units.sort_by(|left, right| left.id.cmp(&right.id));
    debug_assert_eq!(units.len(), effects.len());
    debug_assert!(units.windows(2).all(|pair| pair[0].id <= pair[1].id));
    Ok(units)
}

/// Lower only reviewed core effects. No partial plan is returned on any blocker.
/// The caller must pass producer-observed source digests and existing store paths.
// r[impl mantle.rust_unit_plan.lowering]
// r[impl mantle.rust_unit_plan.dependency_closure]
pub fn lower(effects: &[ExistingUnitEffect], plan: &PlanBindings) -> Result<(Vec<u8>, String), Vec<Blocker>> {
    if effects.is_empty() || effects.len() > MAX_PLAN_UNITS {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: "plan",
            detail: "unit count outside 1..4096",
        }));
    }
    if plan.sources.len() > 256 || plan.sources.is_empty() {
        return Err(blocked(BlockerContext {
            code: "unit-plan-limit",
            subject: "sources",
            detail: "slice count outside 1..256",
        }));
    }
    debug_assert!(effects.len() <= MAX_PLAN_UNITS);
    debug_assert!(plan.sources.len() <= 256);
    let mut tool_roots = BTreeSet::new();
    for path in [&plan.helper, &plan.rustc, &plan.toolchain, &plan.linker] {
        let root = validated_store_root(StoreRootLocation {
            path,
            prefix: &plan.store_prefix,
        })
        .ok_or_else(|| {
            blocked(BlockerContext {
                code: "unit-plan-host-path",
                subject: "toolchain",
                detail: path,
            })
        })?;
        tool_roots.insert(root);
    }
    let mut slices = plan.sources.clone();
    slices.sort_by(|a, b| a.id.cmp(&b.id));
    let source_ids = validate_slices(&slices)?;
    let AdmittedBindings { bindings, known_units } = admit_bindings(effects, plan)?;
    let context = LowerContext {
        plan,
        tool_roots: &tool_roots,
        source_ids: &source_ids,
        known_units: &known_units,
    };
    let units = lower_units(effects, &bindings, &context)?;
    let mut roots = plan.roots.clone();
    roots.sort();
    roots.dedup();
    if roots.is_empty() || roots.iter().any(|root| !known_units.contains(root.as_str())) {
        return Err(blocked(BlockerContext {
            code: "unit-plan-mode-unsupported",
            subject: "roots",
            detail: "root is absent from core effects",
        }));
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
    serialize_plan(&wire)
}
