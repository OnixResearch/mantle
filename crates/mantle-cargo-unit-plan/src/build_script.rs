//! Build-script lowering: one target-description unit per toolchain and triple,
//! one sandboxed execution unit per Cargo `run-custom-build` unit, and the
//! `out`/`flags` references of dependent compilations.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use mantle_rust_plan_core::ExistingUnitEffect;
use mantle_rust_plan_core::check_links_unique;
use mantle_rust_plan_core::is_reserved_build_script_env;

use crate::Blocker;
use crate::Derivation;
use crate::Input;
use crate::LoweringContext;
use crate::MAX_INPUTS_PER_UNIT;
use crate::PlanBindings;
use crate::Unit;
use crate::UnitBinding;
use crate::UnitPolicy;
use crate::blocked;
use crate::plan_id;
use crate::source_token;
use crate::validated_store_root;

/// Normalized core effect kind; Cargo's unit-graph mode remains `run-custom-build`.
pub const BUILD_SCRIPT_RUN: &str = "build-script-run";
/// Helper mode (and the only admitted effect argument) of an execution unit.
pub const RUN_BUILD_SCRIPT_MODE: &str = "run-build-script";
/// Helper mode of a target-description unit.
pub const TARGET_FACTS_MODE: &str = "target-facts";
const BUILD_SCRIPT_RUN_UNSUPPORTED: &str = "unit-plan-build-script-run-unsupported";
const MAX_ENV_VALUE_BYTES: usize = 16_384;
const MAX_ENV_ENTRIES: usize = 512;

/// Build-script topology derived from admitted core effects.
#[derive(Default)]
pub(crate) struct BuildScriptGraph<'a> {
    run_units: BTreeSet<&'a str>,
    /// Compile unit -> execution unit of its own package's build script.
    own_run: BTreeMap<&'a str, &'a str>,
    /// Compile unit -> execution units reachable through compile dependencies,
    /// ordered by dependency topology, then unit id.
    transitive: BTreeMap<&'a str, Vec<&'a str>>,
    /// Execution unit -> compiled build script unit.
    script_compile: BTreeMap<&'a str, &'a str>,
    /// Execution unit -> direct `links` dependency execution units, ordered.
    links_deps: BTreeMap<&'a str, Vec<&'a str>>,
    /// Execution unit -> declared native-input roots (validated when the
    /// execution unit is lowered; an invalid root blocks the whole plan).
    native_roots: BTreeMap<&'a str, &'a [String]>,
    pub(crate) triples: BTreeSet<&'a str>,
}

impl BuildScriptGraph<'_> {
    pub(crate) fn is_run(&self, id: &str) -> bool {
        self.run_units.contains(id)
    }
}

#[allow(tigerstyle::ambiguous_params)] // unit id vs output name: distinct placeholder parts
pub(crate) fn named_output_token(id: &str, output: &str) -> String {
    format!("{{{{mantle-unit-output:{id}:{output}}}}}")
}

fn json<T: serde::Serialize>(id: &str, value: &T) -> Result<String, Vec<Blocker>> {
    serde_json::to_string(value).map_err(|_| blocked("unit-plan-limit", id, "build-script environment serialization"))
}

type Ranks<'a> = BTreeMap<&'a str, u32>;
type EffectsById<'a> = BTreeMap<&'a str, &'a ExistingUnitEffect>;

/// Kahn order over unit edges with `(execution_order, unit id)` tie-break, so
/// ranks are independent of the order effects were supplied in.
#[allow(tigerstyle::unbounded_collection_growth)] // BTree collections cannot reserve; MAX_PLAN_UNITS bounds entries.
fn topological_ranks<'a>(
    effects: &'a [ExistingUnitEffect],
    by_id: &EffectsById<'a>,
) -> Result<Ranks<'a>, Vec<Blocker>> {
    let mut indegree = BTreeMap::<&str, u32>::new();
    let mut consumers = BTreeMap::<&str, Vec<&ExistingUnitEffect>>::new();
    for effect in effects {
        let id = effect.unit_id.0.as_str();
        indegree.entry(id).or_insert(0);
        for dep in &effect.dependency_unit_ids {
            if !by_id.contains_key(dep.0.as_str()) || dep.0 == id {
                return Err(blocked("unit-plan-mode-unsupported", id, "unknown or self dependency"));
            }
            let count = indegree.entry(id).or_insert(0);
            *count = count.saturating_add(1);
            consumers.entry(dep.0.as_str()).or_default().push(effect);
        }
    }
    let mut ready = effects
        .iter()
        .filter(|effect| indegree.get(effect.unit_id.0.as_str()) == Some(&0))
        .map(|effect| (effect.execution_order, effect.unit_id.0.as_str()))
        .collect::<BTreeSet<_>>();
    let mut ranks = BTreeMap::new();
    while let Some((_, id)) = ready.pop_first() {
        let rank = u32::try_from(ranks.len()).map_err(|_| blocked("unit-plan-limit", id, "unit rank overflow"))?;
        ranks.insert(id, rank);
        for consumer in consumers.get(id).map(Vec::as_slice).unwrap_or_default() {
            let consumer_id = consumer.unit_id.0.as_str();
            let count = indegree.entry(consumer_id).or_insert(0);
            *count = count.saturating_sub(1);
            if *count == 0 {
                ready.insert((consumer.execution_order, consumer_id));
            }
        }
    }
    if ranks.len() != effects.len() {
        return Err(blocked("unit-plan-mode-unsupported", "plan", "unit dependencies contain a cycle"));
    }
    debug_assert!(ready.is_empty());
    debug_assert_eq!(ranks.len(), by_id.len());
    Ok(ranks)
}

/// Derive build-script topology. Plans without build-script execution units keep
/// their previous shape exactly.
// r[impl mantle.rust_unit_plan.build_scripts.run_units]
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
#[allow(tigerstyle::unbounded_collection_growth)] // BTree collections cannot reserve; effect count was checked.
pub(crate) fn build_script_graph<'a>(
    effects: &'a [ExistingUnitEffect],
    bindings: &BTreeMap<&'a str, &'a UnitBinding>,
) -> Result<BuildScriptGraph<'a>, Vec<Blocker>> {
    let run_units = effects
        .iter()
        .filter(|effect| effect.execution_kind == BUILD_SCRIPT_RUN)
        .map(|effect| effect.unit_id.0.as_str())
        .collect::<BTreeSet<_>>();
    if run_units.is_empty() {
        return Ok(BuildScriptGraph::default());
    }
    let by_id = effects.iter().map(|effect| (effect.unit_id.0.as_str(), effect)).collect::<EffectsById<'a>>();
    let ranks = topological_ranks(effects, &by_id)?;
    let mut graph = BuildScriptGraph {
        run_units,
        ..BuildScriptGraph::default()
    };
    let mut ordered = effects
        .iter()
        .map(|effect| Ok((rank_of(&ranks, effect.unit_id.0.as_str())?, effect)))
        .collect::<Result<Vec<_>, Vec<Blocker>>>()?;
    ordered.sort_by_key(|(rank, _)| *rank);
    let mut links = Vec::with_capacity(graph.run_units.len());
    for (_, effect) in ordered {
        let id = effect.unit_id.0.as_str();
        let binding: &'a UnitBinding = bindings
            .get(id)
            .copied()
            .ok_or_else(|| blocked("unit-plan-mode-unsupported", id, "missing binding"))?;
        if graph.is_run(id) {
            link_run_unit(&mut graph, effect, binding, &by_id, &ranks)?;
            if let Some(name) = binding.build_script_run.as_ref().and_then(|run| run.links.as_deref()) {
                links.push((effect.package_id.as_str(), name));
            }
        } else {
            link_compile_unit(&mut graph, effect, &by_id, &ranks)?;
        }
    }
    check_links_unique(&links).map_err(|error| blocked(error.code(), "links", &error.to_string()))?;
    debug_assert_eq!(graph.script_compile.len(), graph.run_units.len());
    debug_assert!(!graph.triples.is_empty());
    Ok(graph)
}

fn rank_of(ranks: &Ranks<'_>, id: &str) -> Result<u32, Vec<Blocker>> {
    ranks
        .get(id)
        .copied()
        .ok_or_else(|| blocked("unit-plan-mode-unsupported", id, "unit is absent from the topology"))
}

fn ordered_by_rank<'a>(ids: impl Iterator<Item = &'a str>, ranks: &Ranks<'a>) -> Result<Vec<&'a str>, Vec<Blocker>> {
    let mut ordered = ids.map(|id| Ok((rank_of(ranks, id)?, id))).collect::<Result<Vec<_>, Vec<Blocker>>>()?;
    ordered.sort_unstable();
    ordered.dedup();
    debug_assert!(ordered.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(ordered.len() <= ranks.len());
    Ok(ordered.into_iter().map(|(_, id)| id).collect())
}

fn link_run_unit<'a>(
    graph: &mut BuildScriptGraph<'a>,
    effect: &'a ExistingUnitEffect,
    binding: &'a UnitBinding,
    by_id: &EffectsById<'a>,
    ranks: &Ranks<'a>,
) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let shape_checks = [
        effect.target_kind == "custom-build",
        effect.arguments.as_slice() == [RUN_BUILD_SCRIPT_MODE],
        binding.build_script_run.is_some(),
    ];
    if shape_checks.contains(&false) {
        return Err(blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "run unit is not a declared custom-build execution"));
    }
    let mut compile = None;
    let mut links = Vec::with_capacity(effect.dependency_unit_ids.len());
    for dep in &effect.dependency_unit_ids {
        let dep_id = dep.0.as_str();
        let dep_effect =
            by_id.get(dep_id).ok_or_else(|| blocked("unit-plan-mode-unsupported", id, "unknown dependency"))?;
        if graph.is_run(dep_id) {
            links.push(dep_id);
            continue;
        }
        let is_own_script = dep_effect.target_kind == "custom-build" && dep_effect.package_id == effect.package_id;
        if !is_own_script || compile.replace(dep_id).is_some() {
            return Err(blocked(
                BUILD_SCRIPT_RUN_UNSUPPORTED,
                id,
                "run unit must depend on exactly its compiled script",
            ));
        }
    }
    let compile =
        compile.ok_or_else(|| blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "compiled build script is absent"))?;
    graph.script_compile.insert(id, compile);
    graph.links_deps.insert(id, ordered_by_rank(links.into_iter(), ranks)?);
    graph.triples.insert(binding.triple.as_str());
    if let Some(run) = binding.build_script_run.as_ref().filter(|run| !run.native_inputs.is_empty()) {
        graph.native_roots.insert(id, run.native_inputs.as_slice());
    }
    debug_assert!(graph.script_compile.contains_key(id));
    debug_assert!(!graph.is_run(compile));
    Ok(())
}

fn link_compile_unit<'a>(
    graph: &mut BuildScriptGraph<'a>,
    effect: &'a ExistingUnitEffect,
    by_id: &EffectsById<'a>,
    ranks: &Ranks<'a>,
) -> Result<(), Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let mut own = None;
    let mut reachable = Vec::with_capacity(effect.dependency_unit_ids.len());
    for dep in &effect.dependency_unit_ids {
        let dep_id = dep.0.as_str();
        if graph.is_run(dep_id) {
            let is_own_package = by_id.get(dep_id).is_some_and(|run| run.package_id == effect.package_id);
            if !is_own_package || own.replace(dep_id).is_some() {
                return Err(blocked(
                    BUILD_SCRIPT_RUN_UNSUPPORTED,
                    id,
                    "compile unit depends on a foreign build script",
                ));
            }
            continue;
        }
        reachable.extend(graph.own_run.get(dep_id).copied());
        reachable.extend(graph.transitive.get(dep_id).into_iter().flatten().copied());
    }
    let transitive = ordered_by_rank(reachable.into_iter().filter(|run| Some(*run) != own), ranks)?;
    if transitive.len() > MAX_INPUTS_PER_UNIT {
        return Err(blocked("unit-plan-limit", id, "transitive build-script flags exceed 256"));
    }
    if let Some(run) = own {
        graph.own_run.insert(id, run);
    }
    if !transitive.is_empty() {
        graph.transitive.insert(id, transitive);
    }
    debug_assert!(!graph.is_run(id));
    debug_assert!(graph.transitive.get(id).is_none_or(|runs| runs.iter().all(|run| graph.is_run(run))));
    Ok(())
}

/// Inputs a compile unit needs from build scripts: its own script's `out`
/// (`OUT_DIR`) and `flags`, the `flags` of transitively reachable scripts, and
/// the declared native-input roots of every one of those scripts, because their
/// `-L`/`-l` flags name libraries inside those roots for this compilation's link.
// r[impl mantle.rust_unit_plan.build_scripts.native_inputs]
pub(crate) fn compile_script_inputs(graph: &BuildScriptGraph<'_>, id: &str) -> Vec<Input> {
    let own = graph.own_run.get(id).copied();
    let transitive = graph.transitive.get(id).map(Vec::as_slice).unwrap_or_default();
    let mut inputs = Vec::with_capacity(transitive.len().saturating_add(2));
    if let Some(run) = own {
        inputs.push(Input::UnitOutput {
            unit: plan_id(run),
            output: "out",
        });
        inputs.push(Input::UnitOutput {
            unit: plan_id(run),
            output: "flags",
        });
    }
    inputs.extend(transitive.iter().map(|run| Input::UnitOutput {
        unit: plan_id(run),
        output: "flags",
    }));
    let scripts = own.into_iter().chain(transitive.iter().copied());
    let native = scripts.filter_map(|run| graph.native_roots.get(run)).flat_map(|roots| roots.iter());
    inputs.extend(native.map(|path| Input::StorePath { path: path.clone() }));
    debug_assert!(inputs.len() >= transitive.len());
    debug_assert!(own.is_none() || inputs.len() >= 2);
    inputs
}

/// Environment through which the rustc helper mode reads `flags` files.
// r[impl mantle.rust_unit_plan.build_scripts.directive_propagation]
pub(crate) fn compile_script_env(
    graph: &BuildScriptGraph<'_>,
    effect: &ExistingUnitEffect,
) -> Result<Vec<(String, String)>, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let own = graph.own_run.get(id).copied();
    let transitive = graph.transitive.get(id).map(Vec::as_slice).unwrap_or_default();
    if own.is_none() && transitive.is_empty() {
        return Ok(Vec::new());
    }
    let mut env = Vec::with_capacity(5);
    if let Some(run) = own {
        env.push(("MANTLE_UNIT_BUILD_SCRIPT_OUT".to_string(), named_output_token(&plan_id(run), "out")));
        env.push(("MANTLE_UNIT_BUILD_SCRIPT_FLAGS".to_string(), named_output_token(&plan_id(run), "flags")));
    }
    let tokens = transitive.iter().map(|run| named_output_token(&plan_id(run), "flags")).collect::<Vec<_>>();
    env.push(("MANTLE_UNIT_TRANSITIVE_FLAGS".to_string(), json(id, &tokens)?));
    env.push(("MANTLE_UNIT_TARGET_KIND".to_string(), effect.target_kind.clone()));
    env.push(("MANTLE_UNIT_TARGET_NAME".to_string(), effect.target_name.clone()));
    debug_assert!(env.len() >= 3);
    debug_assert!(env.len() <= 5);
    Ok(env)
}

/// One target-description unit per toolchain, host, and triple.
pub(crate) fn target_facts_id(plan: &PlanBindings, triple: &str) -> String {
    let identity = format!("target-facts\0{}\0{}\0{}\0{triple}", plan.toolchain, plan.rustc, plan.host_triple);
    debug_assert!(!triple.is_empty());
    debug_assert!(identity.starts_with("target-facts"));
    plan_id(&identity)
}

/// Owned pieces of one lowered unit.
pub(crate) struct UnitParts {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
    pub(crate) outputs: Vec<&'static str>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) inputs: Vec<Input>,
}

pub(crate) fn finish_unit(parts: UnitParts, plan: &PlanBindings) -> Result<Unit, Vec<Blocker>> {
    let is_env_overflow =
        parts.env.len() > MAX_ENV_ENTRIES || parts.env.values().any(|value| value.len() > MAX_ENV_VALUE_BYTES);
    if is_env_overflow {
        return Err(blocked("unit-plan-limit", &parts.id, "unit environment exceeds generic plan limits"));
    }
    if parts.inputs.len() > MAX_INPUTS_PER_UNIT {
        return Err(blocked("unit-plan-limit", &parts.id, "unit inputs exceed 256"));
    }
    let mut inputs = parts.inputs;
    inputs.sort_by(|a, b| a.canonical_key().cmp(&b.canonical_key()));
    inputs.dedup();
    debug_assert!(!parts.outputs.is_empty());
    debug_assert!(inputs.windows(2).all(|pair| pair[0].canonical_key() < pair[1].canonical_key()));
    Ok(Unit {
        id: parts.id,
        derivation: Derivation {
            name: parts.name,
            builder: plan.helper.clone(),
            system: plan.system.clone(),
            args: parts.args,
            outputs: parts.outputs.clone(),
            env: parts.env,
            inputs,
            fixed_output: None,
            addressing_mode: "input-addressed",
            sandbox: "native",
            dynamic_plan_outputs: Vec::new(),
        },
        requested_outputs: parts.outputs,
        policy: UnitPolicy {
            sandbox: "inherit",
            substitutions: "inherit",
            store_prefix: "inherit",
            host_paths: "none",
        },
    })
}

fn tool_root(plan: &PlanBindings, path: &str) -> Result<Input, Vec<Blocker>> {
    let root = validated_store_root(path, &plan.store_prefix)
        .ok_or_else(|| blocked("unit-plan-host-path", "toolchain", path))?;
    crate::check_execution_root(plan, root)?;
    debug_assert!(root.len() <= path.len());
    debug_assert!(path.starts_with(root));
    Ok(Input::StorePath { path: root.to_string() })
}

/// The shared target-description unit: `rustc --print cfg --target <triple>`
/// plus the declared host, recorded as a typed environment file.
// r[impl mantle.rust_unit_plan.build_scripts.target_facts]
pub(crate) fn target_facts_unit(plan: &PlanBindings, triple: &str) -> Result<Unit, Vec<Blocker>> {
    let id = target_facts_id(plan, triple);
    let mut inputs = Vec::with_capacity(plan.runtime_inputs.len().saturating_add(2));
    inputs.extend([tool_root(plan, &plan.helper)?, tool_root(plan, &plan.rustc)?]);
    for path in &plan.runtime_inputs {
        inputs.push(tool_root(plan, path)?);
    }
    let env = BTreeMap::from([
        ("MANTLE_UNIT_RUSTC".to_string(), plan.rustc.clone()),
        ("MANTLE_UNIT_TARGET".to_string(), triple.to_string()),
        ("MANTLE_UNIT_HOST".to_string(), plan.host_triple.clone()),
    ]);
    let name = format!("rust-target-facts-{}", &id[2..18]);
    debug_assert!(id.starts_with("u."));
    debug_assert_eq!(env.len(), 3);
    finish_unit(
        UnitParts {
            id,
            name,
            args: vec![TARGET_FACTS_MODE.to_string()],
            outputs: vec!["out"],
            env,
            inputs,
        },
        plan,
    )
}

/// One build-script execution unit: runs the compiled script through the
/// helper with only the declared environment and writes `out` and `flags`.
// r[impl mantle.rust_unit_plan.build_scripts.run_units]
// r[impl mantle.rust_unit_plan.build_scripts.sandbox_boundary]
pub(crate) fn lower_run_unit(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    context: &LoweringContext<'_>,
    graph: &BuildScriptGraph<'_>,
) -> Result<Unit, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let compile = graph.script_compile.get(id).copied();
    let links = graph.links_deps.get(id).map(Vec::as_slice).unwrap_or_default();
    let compile =
        compile.ok_or_else(|| blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "compiled build script is absent"))?;
    let facts_id = target_facts_id(context.plan, &binding.triple);
    let mut inputs = context
        .tool_roots
        .iter()
        .map(|path| Input::StorePath {
            path: (*path).to_string(),
        })
        .collect::<Vec<_>>();
    inputs.push(Input::Source {
        source: binding.source_id.clone(),
    });
    inputs.push(Input::UnitOutput {
        unit: plan_id(compile),
        output: "out",
    });
    inputs.push(Input::UnitOutput {
        unit: facts_id.clone(),
        output: "out",
    });
    inputs.extend(links.iter().map(|run| Input::UnitOutput {
        unit: plan_id(run),
        output: "flags",
    }));
    let native = native_input_roots(binding, context.plan)?;
    inputs.extend(native.iter().map(|path| Input::StorePath { path: path.clone() }));
    let mut env = run_unit_env(effect, binding, context.plan, &native)?;
    env.insert("MANTLE_UNIT_BUILD_SCRIPT".to_string(), named_output_token(&plan_id(compile), "out"));
    env.insert("MANTLE_UNIT_TARGET_FACTS".to_string(), named_output_token(&facts_id, "out"));
    env.insert("MANTLE_UNIT_SOURCE".to_string(), source_token(&binding.source_id));
    let tokens = links.iter().map(|run| named_output_token(&plan_id(run), "flags")).collect::<Vec<_>>();
    env.insert("MANTLE_UNIT_LINKS_DEPENDENCIES".to_string(), json(id, &tokens)?);
    debug_assert!(inputs.len() >= links.len().saturating_add(3));
    debug_assert!(env.contains_key("MANTLE_UNIT_SCRIPT_ENV"));
    finish_unit(
        UnitParts {
            id: plan_id(id),
            name: format!("rust-build-script-run-{}", &blake3::hash(id.as_bytes()).to_hex().as_str()[..16]),
            args: vec![RUN_BUILD_SCRIPT_MODE.to_string()],
            outputs: vec!["out", "flags"],
            env,
            inputs,
        },
        context.plan,
    )
}

/// Declared native inputs must be whole store roots.
// r[impl mantle.rust_unit_plan.build_scripts.native_inputs]
fn native_input_roots(binding: &UnitBinding, plan: &PlanBindings) -> Result<Vec<String>, Vec<Blocker>> {
    let id = binding.unit_id.as_str();
    let run = binding
        .build_script_run
        .as_ref()
        .ok_or_else(|| blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "run binding is absent"))?;
    if run.native_inputs.len() > 64 {
        return Err(blocked("unit-plan-limit", id, "native inputs exceed 64"));
    }
    let mut roots = Vec::with_capacity(run.native_inputs.len());
    for path in &run.native_inputs {
        if validated_store_root(path, &plan.store_prefix) != Some(path.as_str()) {
            return Err(blocked("unit-plan-host-path", id, path));
        }
        crate::check_execution_root(plan, path)?;
        roots.push(path.clone());
    }
    roots.sort();
    roots.dedup();
    debug_assert!(roots.len() <= run.native_inputs.len());
    debug_assert!(roots.iter().all(|root| root.starts_with(&plan.store_prefix)));
    Ok(roots)
}

/// The script's declared environment: Cargo package facts from the effect,
/// `RUSTC`, `CARGO_MANIFEST_LINKS`, and resolved native-input values. `OUT_DIR`,
/// `TARGET`, `HOST`, `CARGO_CFG_*`, and `DEP_*` come only from the helper.
fn run_unit_env(
    effect: &ExistingUnitEffect,
    binding: &UnitBinding,
    plan: &PlanBindings,
    native: &[String],
) -> Result<BTreeMap<String, String>, Vec<Blocker>> {
    let id = effect.unit_id.0.as_str();
    let run = binding
        .build_script_run
        .as_ref()
        .ok_or_else(|| blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "run binding is absent"))?;
    let mut script_env = effect.environment.iter().cloned().collect::<BTreeMap<_, _>>();
    let lane_owned = |key: &str| {
        matches!(key, "OUT_DIR" | "TARGET" | "HOST" | "PATH")
            || key.starts_with("DEP_")
            || key.starts_with("CARGO_CFG_")
    };
    if script_env.keys().any(|key| lane_owned(key.as_str())) {
        return Err(blocked(BUILD_SCRIPT_RUN_UNSUPPORTED, id, "script environment declares a lane-owned key"));
    }
    script_env.insert("RUSTC".to_string(), plan.rustc.clone());
    if let Some(links) = &run.links {
        script_env.insert("CARGO_MANIFEST_LINKS".to_string(), links.clone());
    }
    for (key, value) in &run.native_env {
        if is_reserved_build_script_env(key)
            || value.contains('\0')
            || script_env.insert(key.clone(), value.clone()).is_some()
        {
            return Err(blocked("native-input-env-reserved", id, key));
        }
    }
    let is_under_native = |entry: &str| {
        native
            .iter()
            .any(|root| entry.strip_prefix(root.as_str()).is_some_and(|rest| rest.is_empty() || rest.starts_with('/')))
    };
    if let Some(entry) = run.native_path.iter().find(|entry| !is_under_native(entry.as_str()) || entry.contains(':')) {
        return Err(blocked("unit-plan-host-path", id, entry));
    }
    let env = BTreeMap::from([
        ("MANTLE_UNIT_SCRIPT_ENV".to_string(), json(id, &script_env)?),
        ("MANTLE_UNIT_SCRIPT_PATH".to_string(), json(id, &run.native_path)?),
        ("MANTLE_UNIT_NATIVE_INPUTS".to_string(), json(id, &native)?),
        ("MANTLE_UNIT_RUSTC".to_string(), plan.rustc.clone()),
    ]);
    debug_assert!(script_env.contains_key("RUSTC"));
    debug_assert_eq!(env.len(), 4);
    Ok(env)
}
